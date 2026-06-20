use tokio::io::AsyncWriteExt;

pub static MODULE_IDENTITY: &str = "JUDGE";

pub const SUBMISSIONS_LIST_PAGE_SIZE: i64 = 20;

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

pub static JUDGE_SOCKET: std::sync::OnceLock<AsyncModifiable<tokio::net::TcpListener>> = std::sync::OnceLock::new();

// The `problem` directory lives at the project root, one level above `rust_backend` (where every
// module's process actually runs from), mirroring `main_backend`'s `get_parent_path()` convention.
pub fn get_problem_dir_path() -> String {
    let mut pwd: std::path::PathBuf = std::env::current_dir().unwrap();
    pwd.pop();
    pwd.push("problem");
    String::from(pwd.to_str().unwrap())
}

#[derive(serde::Deserialize)]
pub struct ProblemSetJson {
    pub problem_set: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct ProblemStatementJson {
    pub problem_number: i64,
    pub difficulty: i32,
    pub problem_name: String,
    pub problem_statement: Vec<String>,
}

pub const MYSQL_DATABASE_URL: &str = "mysql://root:123456@127.0.0.1:3306/";

pub static MYSQL_DATABASE_POOL: std::sync::LazyLock<tokio::sync::Mutex<mysql_async::Pool>> = std::sync::LazyLock::new(
    || { tokio::sync::Mutex::new(mysql_async::Pool::new(MYSQL_DATABASE_URL)) }
);

// In-memory map of pending `submissions_list` requests, keyed by the request_key used when
// asking simple_authenticator for the requester's username (resolved from their ws_id). Needed
// because that lookup is an async round trip — when the result comes back, we need to recall
// who asked and which page they wanted.
pub struct PendingSubmissionsListRequest {
    pub requester_ws_id: String,
    pub page_index: i64,
    pub original_request_key: String,
}

pub static PENDING_SUBMISSIONS_LIST_REQUESTS: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, PendingSubmissionsListRequest>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

// In-memory map of pending `submission` requests, keyed by the request_key used when asking
// simple_authenticator to validate the requester's session. Needed because that lookup is an
// async round trip — when the result comes back, we need to recall the original submission.
pub struct PendingSubmissionRequest {
    pub requester_ws_id: String,
    pub username: String,
    pub problem_number: i64,
    pub language: String,
    pub code: Vec<String>,
    pub is_test_submission_mode: bool,
    pub original_request_key: String,
}

pub static PENDING_SUBMISSION_REQUESTS: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, PendingSubmissionRequest>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

// In-memory map of pending `submission_result` (fetch-by-id) requests, keyed by the request_key
// used when asking simple_authenticator to resolve the requester's username from their ws_id.
// Needed to decide whether the requester is the submission's own author (who alone may see
// `code`) before replying.
pub struct PendingSubmissionResultFetchRequest {
    pub requester_ws_id: String,
    pub submission_id: i64,
    pub owner_username: String,
    pub problem_number: i64,
    pub result: String,
    pub general_score: i32,
    pub statuses: Vec<String>,
    pub scores: Vec<i32>,
    pub code: Vec<String>,
    pub language: String,
    pub original_request_key: String,
}

pub static PENDING_SUBMISSION_RESULT_FETCH_REQUESTS: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, PendingSubmissionResultFetchRequest>>
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
