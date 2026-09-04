use tokio::io::AsyncWriteExt;

pub static MODULE_IDENTITY: &str = "USERISH";

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

pub static USERISH_SOCKET: std::sync::OnceLock<AsyncModifiable<tokio::net::TcpListener>> = std::sync::OnceLock::new();

pub const MYSQL_DATABASE_URL: &str = "mysql://root:123456@127.0.0.1:3306/";

pub static MYSQL_DATABASE_POOL: std::sync::LazyLock<mysql_async::Pool> =
    std::sync::LazyLock::new(|| mysql_async::Pool::new(MYSQL_DATABASE_URL));

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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnSendMsg {
    pub ws_id: String,
    pub msg_to_send: serde_json::Value,
}

// `on_follow`/`on_unfollow` need to validate the caller's session, but unlike
// `simple_authenticator` (which owns `SESSION_TOKENS_BY_USERNAME` in-process), this module has no
// local view of session state — it asks `simple_authenticator` via the same `on_validate_session`
// RPC `judge` already uses, and resumes here (in `on_validate_session_result`) once the reply
// arrives. Keyed by the request_key sent with the RPC, not the original client request_key.
pub struct PendingFollowRequest {
    pub ws_id: String,
    pub follower_username: String,
    pub followee_username: String,
    pub request_key: String,
}

pub struct PendingUnfollowRequest {
    pub ws_id: String,
    pub follower_username: String,
    pub followee_username: String,
    pub request_key: String,
}

pub static PENDING_FOLLOW_REQUESTS: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, PendingFollowRequest>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

pub static PENDING_UNFOLLOW_REQUESTS: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, PendingUnfollowRequest>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

pub fn expire_pending<T: Send + 'static>(
    map: &'static tokio::sync::Mutex<std::collections::HashMap<String, T>>,
    request_key: String,
) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        map.lock().await.remove(&request_key);
    });
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
