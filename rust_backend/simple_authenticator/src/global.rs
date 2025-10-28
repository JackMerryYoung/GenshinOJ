use crypto::digest::Digest;
use tokio::io::AsyncWriteExt;

pub static MODULE_IDENTITY: &str = "SIMPLE_AUTHENTICATOR";

pub type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

pub fn new_async_modifiable<T>(x: T) -> AsyncModifiable<T> {
    std::sync::Arc::new(tokio::sync::Mutex::new(x))
}

#[derive(Debug)]
pub struct ModuleStatus {
    pub initialized: bool,
    pub panicked: bool,
    pub socket_port: AsyncModifiable<u16>,
}

pub static GLOBAL_MODULE_STATUSES_BY_PROTOCOL: std::sync::OnceLock<
    AsyncModifiable<std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>>
> = std::sync::OnceLock::new();

pub static SIMPLE_AUTHENTICATOR_SOCKET: std::sync::OnceLock<AsyncModifiable<tokio::net::TcpListener>> = std::sync::OnceLock::new();

pub const MYSQL_DATABASE_URL: &str = "mysql://root:123456@127.0.0.1:3306/GenshinOJ";

pub static MYSQL_DATABASE_POOL: std::sync::LazyLock<tokio::sync::Mutex<mysql_async::Pool>> = std::sync::LazyLock::new(
    || { tokio::sync::Mutex::new(mysql_async::Pool::new(MYSQL_DATABASE_URL)) }
);

pub static SESSION_TOKENS_BY_USERNAME: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, String>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

pub static LOGGED_IN_USERNAMES: std::sync::LazyLock<tokio::sync::Mutex<std::collections::HashSet<String>>> = std::sync::LazyLock::new(
    || tokio::sync::Mutex::new(std::collections::HashSet::new())
);

pub static USERNAMES_BY_WS_ID: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, String>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct SocketJsonMessage {
    pub r#type: String,
    pub content: serde_json::Value,
    pub request_key: String,
    pub from_protocol: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
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

pub fn generate_session_token(session_token_seed: u32) -> String {
    if session_token_seed > 1 {
        let generated_session_token: String =
            char
                ::from_u32((session_token_seed % 26) + ('a' as u32))
                .unwrap()
                .to_string() +
            char
                ::from_u32(((session_token_seed * 3) % 26) + ('a' as u32))
                .unwrap()
                .to_string()
                .as_str() +
            char
                ::from_u32(((session_token_seed * 5) % 26) + ('a' as u32))
                .unwrap()
                .to_string()
                .as_str() +
            char
                ::from_u32(((session_token_seed * 7) % 26) + ('a' as u32))
                .unwrap()
                .to_string()
                .as_str() +
            char
                ::from_u32(((session_token_seed * 9) % 26) + ('a' as u32))
                .unwrap()
                .to_string()
                .as_str() +
            char
                ::from_u32(((session_token_seed * 11) % 26) + ('a' as u32))
                .unwrap()
                .to_string()
                .as_str() +
            char
                ::from_u32(((session_token_seed * 13) % 26) + ('a' as u32))
                .unwrap()
                .to_string()
                .as_str() +
            char
                ::from_u32(((session_token_seed * 15) % 26) + ('a' as u32))
                .unwrap()
                .to_string()
                .as_str();
        return generated_session_token + generate_session_token(session_token_seed / 5).as_str();
    }
    String::from("s")
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnSendMsg {
    pub ws_id: String,
    pub msg_to_send: serde_json::Value,
}

pub async fn get_socket_port_by_protocol(protocol: &str) -> u16 {
    let guard_global_module_statuses_by_protocol: tokio::sync::MutexGuard<
        '_,
        std::collections::HashMap<String, std::sync::Arc<tokio::sync::Mutex<ModuleStatus>>>
    > = GLOBAL_MODULE_STATUSES_BY_PROTOCOL.get().unwrap().lock().await;
    let guard_status: tokio::sync::MutexGuard<
        '_,
        ModuleStatus
    > = guard_global_module_statuses_by_protocol.get(protocol).unwrap().lock().await;
    *guard_status.socket_port.lock().await
}

pub async fn get_socket_by_protocol(protocol: &str) -> tokio::net::TcpStream {
    let socket_port: u16 = get_socket_port_by_protocol(protocol).await;
    match tokio::net::TcpStream::connect(format!("127.0.0.1:{socket_port}")).await {
        Ok(socket) => socket,
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

pub async fn send_socket_json_message(msg_to_send: &serde_json::Value, to_protocol: &String) {
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

const FAKE_YIELD_NOW_MILLISECONDS: u64 = 100;

pub async fn fake_yield_now(tm: u64) {
    if tm == 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(FAKE_YIELD_NOW_MILLISECONDS)).await;
    } else {
        tokio::time::sleep(tokio::time::Duration::from_millis(tm)).await;
    }
}
