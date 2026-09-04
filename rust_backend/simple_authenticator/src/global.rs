use crypto::digest::Digest;
use rand::Rng;
use tokio::io::AsyncWriteExt;

pub static MODULE_IDENTITY: &str = "SIMPLE_AUTHENTICATOR";

pub type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

pub fn new_async_modifiable<T>(x: T) -> AsyncModifiable<T> {
    std::sync::Arc::new(tokio::sync::Mutex::new(x))
}

#[derive(Debug, Clone)]
pub struct ModuleStatus {
    pub initialized: bool,
    pub panicked: bool,
    pub socket_port: AsyncModifiable<u16>,
    pub init_notify: std::sync::Arc<tokio::sync::Notify>,
}

pub static GLOBAL_MODULE_STATUSES_BY_PROTOCOL: std::sync::OnceLock<
    AsyncModifiable<std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>>
> = std::sync::OnceLock::new();

pub static SIMPLE_AUTHENTICATOR_SOCKET: std::sync::OnceLock<AsyncModifiable<tokio::net::TcpListener>> = std::sync::OnceLock::new();

pub const MYSQL_DATABASE_URL: &str = "mysql://root:123456@127.0.0.1:3306/";

pub static MYSQL_DATABASE_POOL: std::sync::LazyLock<mysql_async::Pool> =
    std::sync::LazyLock::new(|| mysql_async::Pool::new(MYSQL_DATABASE_URL));

// Serializes operations spanning the token, online-user, and websocket identity maps. The maps
// remain separate to keep the existing call sites small, but they form one logical session state
// and must never be observed or updated halfway through a restore/logout transition.
pub static SESSION_STATE_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

pub static MODULE_RUNTIME_HANDLE: std::sync::OnceLock<tokio::runtime::Handle> =
    std::sync::OnceLock::new();

pub fn run_shutdown_task<F>(task: F, timeout: std::time::Duration) -> bool
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    if let Some(handle) = MODULE_RUNTIME_HANDLE.get() {
        let (complete_tx, complete_rx) = std::sync::mpsc::channel();
        drop(handle.spawn(async move {
            task.await;
            let _ = complete_tx.send(());
        }));
        complete_rx.recv_timeout(timeout).is_ok()
    } else {
        eprintln!("[{}] [WARNING] Runtime handle is unavailable during shutdown", MODULE_IDENTITY);
        false
    }
}

pub async fn disconnect_database_pool() {
    if let Err(error) = MYSQL_DATABASE_POOL.clone().disconnect().await {
        eprintln!(
            "[{}] [WARNING] Failed to disconnect the database pool during shutdown: {}",
            MODULE_IDENTITY, error
        );
    }
}

pub async fn get_db_conn() -> Result<mysql_async::Conn, mysql_async::Error> {
    MYSQL_DATABASE_POOL.get_conn().await
}

pub static SESSION_TOKENS_BY_USERNAME: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, String>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

pub static LOGGED_IN_USERNAMES: std::sync::LazyLock<tokio::sync::Mutex<std::collections::HashSet<String>>> = std::sync::LazyLock::new(
    || tokio::sync::Mutex::new(std::collections::HashSet::new())
);

pub static USERNAMES_BY_WS_ID: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, String>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

pub static WS_IDS_BY_USERNAME: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, String>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

// Caller must hold SESSION_STATE_LOCK. Removes the identity currently attached to a websocket
// without invalidating its reusable session token.
pub async fn unbind_ws_identity(ws_id: &str) {
    let previous_username = USERNAMES_BY_WS_ID.lock().await.remove(ws_id);
    let Some(previous_username) = previous_username else {
        return;
    };

    let was_current_connection = {
        let mut ws_ids_by_username = WS_IDS_BY_USERNAME.lock().await;
        if ws_ids_by_username.get(&previous_username).map(String::as_str) == Some(ws_id) {
            ws_ids_by_username.remove(&previous_username);
            true
        } else {
            false
        }
    };
    if was_current_connection {
        LOGGED_IN_USERNAMES.lock().await.remove(&previous_username);
    }
}

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct SocketJsonMessage {
    pub r#type: String,
    pub content: serde_json::Value,
    pub request_key: String,
    pub from_protocol: String,
}

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct SocketJsonMessageWithWsId {
    pub r#type: String,
    pub content: serde_json::Value,
    pub request_key: String,
    pub from_protocol: String,
    pub ws_id: String,
}

pub fn get_hash(text: &str) -> String {
    let mut md5_hasher = crypto::md5::Md5::new();
    md5_hasher.input_str("add-some-salt");
    md5_hasher.input_str(text);
    md5_hasher.result_str()
}

pub fn generate_session_token(_session_token_seed: u32) -> String {
    // A session token must be unpredictable even if an attacker can observe login timing or
    // knows the old seed. Keep the argument for source compatibility with existing callers.
    let mut token_bytes = [0u8; 32];
    rand::rng().fill(&mut token_bytes);
    token_bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnSendMsg {
    pub ws_id: String,
    pub msg_to_send: serde_json::Value,
}

pub async fn get_socket_port_by_protocol(protocol: &str) -> Option<u16> {
    let guard_global_module_statuses_by_protocol: tokio::sync::MutexGuard<
        '_,
        std::collections::HashMap<String, std::sync::Arc<tokio::sync::Mutex<ModuleStatus>>>
    > = GLOBAL_MODULE_STATUSES_BY_PROTOCOL.get()?.lock().await;
    let guard_status: tokio::sync::MutexGuard<
        '_,
        ModuleStatus
    > = guard_global_module_statuses_by_protocol.get(protocol)?.lock().await;
    Some(*guard_status.socket_port.lock().await)
}

pub async fn get_socket_by_protocol(protocol: &str) -> tokio::net::TcpStream {
    let socket_port: u16 = match get_socket_port_by_protocol(protocol).await {
        Some(x) => x,
        None => {
            eprintln!(
                "{}",
                ansi_term::Color::Red.paint(
                    format!(
                        "[{}] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to get the port of the module implemented protocol `{}`.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        protocol
                    )
                )
            );
            panic!();
        }
    };
    match tokio::net::TcpStream::connect(format!("127.0.0.1:{socket_port}")).await {
        Ok(socket) => {
            // Without this, Nagle's algorithm + the peer's delayed ACK can stall these
            // one-shot connect-write-drop control messages by hundreds of ms to seconds.
            socket.set_nodelay(true).ok();
            socket
        }
        Err(_) => {
            eprintln!(
                "{}",
                ansi_term::Color::Red.paint(
                    format!(
                        "[{}] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to connect to the socket of the module implemented protocol `{}` on port {}.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        protocol,
                        socket_port
                    )
                )
            );
            panic!();
        }
    }
}

pub async fn send_socket_json_message(msg_to_send: &serde_json::Value, to_protocol: &str) {
    let mut socket = get_socket_by_protocol(to_protocol).await;
    let json_msg_str = serde_json::to_string(&msg_to_send).unwrap();
    let result = socket.write_all(json_msg_str.as_bytes()).await;
    if result.is_err() {
        println!(
            "{}",
            ansi_term::Color::Yellow.paint(
                format!(
                    "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to send message to protocol `{}`.",
                    MODULE_IDENTITY,
                    std::thread::current().id().as_u64(),
                    to_protocol,
                    file!(),
                    line!()
                )
            )
        );
    }
}
