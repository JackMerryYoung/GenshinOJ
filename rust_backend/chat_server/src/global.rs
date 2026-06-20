use tokio::io::AsyncWriteExt;

pub static MODULE_IDENTITY: &str = "CHAT_SERVER";

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

pub static CHAT_SERVER_SOCKET: std::sync::OnceLock<AsyncModifiable<tokio::net::TcpListener>> = std::sync::OnceLock::new();

pub const MYSQL_DATABASE_URL: &str = "mysql://root:123456@127.0.0.1:3306/";

pub static MYSQL_DATABASE_POOL: std::sync::LazyLock<tokio::sync::Mutex<mysql_async::Pool>> = std::sync::LazyLock::new(
    || { tokio::sync::Mutex::new(mysql_async::Pool::new(MYSQL_DATABASE_URL)) }
);

// In-memory map of pending `chat_user` requests, keyed by the request_key used when asking
// simple_authenticator to validate the session and locate the recipient. Needed because the
// validate-and-locate round trip is async — when the result comes back, we need to recall who
// asked and what they sent.
pub struct PendingChatRequest {
    pub from_ws_id: String,
    pub from_username: String,
    pub to_username: String,
    pub messages: String,
}

// Same idea as `PendingChatRequest`, but for `on_chat_history` requests: the session-validation
// round trip to simple_authenticator is async, so we need to recall who asked and what page of
// history they wanted once the result comes back.
pub struct PendingChatHistoryRequest {
    pub requester_ws_id: String,
    pub requester_username: String,
    pub with_username: String,
    pub before_id: Option<i64>,
}

pub static PENDING_CHAT_HISTORY_REQUESTS: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, PendingChatHistoryRequest>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

pub struct ChatMessageRow {
    pub id: i64,
    pub from_username: String,
    pub messages: String,
    pub created_at: i64,
}

pub async fn store_chat_message(from_username: &str, to_username: &str, messages: &str, created_at: i64) {
    use mysql_async::prelude::*;
    let guard_mysql_database_pool: tokio::sync::MutexGuard<
        '_,
        mysql_async::Pool
    > = MYSQL_DATABASE_POOL.lock().await;
    let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
    conn.exec_drop(
        "INSERT INTO GenshinOJ.chat_messages (from_username, to_username, messages, created_at) VALUES (:from_username, :to_username, :messages, :created_at)",
        mysql_async::params! {
            "from_username" => from_username,
            "to_username" => to_username,
            "messages" => messages,
            "created_at" => created_at,
        }
    ).await
        .unwrap();
}

// Fetches up to 10 messages exchanged between `username_a` and `username_b`, ordered oldest
// first, optionally only those older than `before_id` (for "load 10 more" pagination). Returns
// them already in chronological order so the caller can prepend/render directly.
pub async fn fetch_chat_history(
    username_a: &str,
    username_b: &str,
    before_id: Option<i64>
) -> Vec<ChatMessageRow> {
    use mysql_async::prelude::*;
    let guard_mysql_database_pool: tokio::sync::MutexGuard<
        '_,
        mysql_async::Pool
    > = MYSQL_DATABASE_POOL.lock().await;
    let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
    let rows: Vec<(i64, String, String, i64)> = match before_id {
        Some(before_id) => {
            conn.exec(
                "SELECT id, from_username, messages, created_at FROM GenshinOJ.chat_messages
                WHERE ((from_username = :username_a AND to_username = :username_b)
                    OR (from_username = :username_b AND to_username = :username_a))
                    AND id < :before_id
                ORDER BY id DESC LIMIT 10",
                mysql_async::params! { "username_a" => username_a, "username_b" => username_b, "before_id" => before_id }
            ).await
                .unwrap()
        }
        None => {
            conn.exec(
                "SELECT id, from_username, messages, created_at FROM GenshinOJ.chat_messages
                WHERE (from_username = :username_a AND to_username = :username_b)
                    OR (from_username = :username_b AND to_username = :username_a)
                ORDER BY id DESC LIMIT 10",
                mysql_async::params! { "username_a" => username_a, "username_b" => username_b }
            ).await
                .unwrap()
        }
    };
    rows.into_iter()
        .rev()
        .map(|(id, from_username, messages, created_at)| ChatMessageRow { id, from_username, messages, created_at })
        .collect()
}

pub static PENDING_CHAT_REQUESTS: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, PendingChatRequest>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

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
