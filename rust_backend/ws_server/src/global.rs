use tokio::io::AsyncWriteExt;

pub static MODULE_IDENTITY: &str = "WS_SERVER";

/**
    # `retry!()`

    A macro simplifies the process of executing a task which may fail, and retrying the same task until it succeeds, if it doesn't succeed at first.

    ## Arguments

    ### `retry!($f, $count, $interval, $retry_func)`

    Execute `$f` and get returned result first, if result is not `Ok(T)`, retry after `$interval` ms, until exceeding `$count$` time(s). 

    Also Execute `$retry_func` after every failure.

    ### `retry!($f)`

    Equal to `retry!($f, 2, 1000, {})`.

    ## Demo

    ``` rs
    let listener: Result<tokio::net::TcpListener, std::io::Error> = retry!(
        tokio::net::TcpListener::bind("0.0.0.0:9983").await,
        2,
        1000,
        {
            eprintln!(
                "{}",
                ansi_term::Color::Red.paint(
                    format!(
                        "[{}] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to initialize the Websocket server. Retrying...",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    )
                )
            );
        }
    );
    ```
 */
#[macro_export]
macro_rules! retry {
    ($f:expr, $count:expr, $interval:expr, $retry_func:expr) => {
        {
        let mut retries = 0;
        let result = loop {
            let result = $f;
            if result.is_ok() {
                break result;
            } else if retries > $count {
                break result;
            } else {
                retries += 1;
                $retry_func
                tokio::time::sleep(std::time::Duration::from_millis($interval)).await;
            }
        };
        result
        }
    };
    ($f:expr) => {
        retry!($f, 2, 1000, {})
    };
}

pub type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

pub fn new_async_modifiable<T>(x: T) -> AsyncModifiable<T> {
    std::sync::Arc::new(tokio::sync::Mutex::new(x))
}

#[derive(Debug)]
pub struct ModuleStatus {
    pub initialized: bool,
    pub panicked: bool,
    pub socket_port: AsyncModifiable<u16>,
    pub init_notify: std::sync::Arc<tokio::sync::Notify>,
}

/**
    # `GLOBAL_MODULE_STATUSES_BY_PROTOCOL`

    Represents statuses of all the modules.

    **Module status can be indexed by specifying protocol.**
 */
pub static GLOBAL_MODULE_STATUSES_BY_PROTOCOL: std::sync::OnceLock<
    AsyncModifiable<std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>>
> = std::sync::OnceLock::new();

/**
    # `WS_SERVER_CONNECTIONS_CNT`

    Represents the count of Websocket connections.

    Used in self-management task when showing the count of Websocket connections.
 */
pub static WS_SERVER_CONNECTIONS_CNT: std::sync::LazyLock<tokio::sync::Mutex<usize>> = std::sync::LazyLock::new(
    || tokio::sync::Mutex::new(0)
);

/**
    # `WS_SERVER_CONNECTIONS_BY_WS_ID`

    Represents Websocket connections.

    **Websocket connections can be indexed by specifying `ws_id` (a `String` value, which is unique to each Websocket connection).**
 */
type AsyncWsSender = AsyncModifiable<
    futures_util::stream::SplitSink<axum::extract::ws::WebSocket, axum::extract::ws::Message>
>;
pub static WS_SERVER_CONNECTIONS_BY_WS_ID: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, AsyncWsSender>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

/*
 * This part is about cross-module communicating socket of this Websocket Server.
 */

pub static WS_SERVER_SOCKET: std::sync::OnceLock<AsyncModifiable<tokio::net::TcpListener>> = std::sync::OnceLock::new();

/**
    # `ExternalListener`

    Represents a external module's listener, which serves as a record of those modules that can receive specific command from this Websocket Server.

    **When there's a command received by this Websocket Server, it'll be forwarded to those modules that can received this kind of commands.**
 */

#[derive(Debug, Clone)]
pub struct ExternalListener {
    pub command: String, // Represents what command they listen to.
    pub required_protocol_version: semver::VersionReq, // Version range a protocol must satisfy to bind to this command.
    pub protocols: std::collections::HashMap<String, semver::Version>, // Protocols (by name) implemented by modules which listen to this command, with the version they bound with.
}

/**
    # `WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND`

    Represents several external modules' listeners.

    **External listeners can be indexed by specifying commands**.
 */
pub static WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND: std::sync::OnceLock<
    std::sync::RwLock<std::sync::Arc<std::collections::HashMap<String, ExternalListener>>>
> = std::sync::OnceLock::new();

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct WebsocketServerJsonMessage {
    pub r#type: String,
    pub content: serde_json::Value,
}

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
            panic!(); // TODO: Not panicking here but returning a Result would be better.
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

pub fn get_pwd() -> String {
    let pwd: std::path::PathBuf = std::env::current_dir().unwrap();
    String::from(pwd.to_str().unwrap())
}

// The `avatars` directory lives at the project root, one level above `rust_backend` (where every
// module's process actually runs from) — mirrors `judge`'s `get_problem_dir_path()` convention.
pub fn get_avatars_dir_path() -> String {
    let mut pwd: std::path::PathBuf = std::env::current_dir().unwrap();
    pwd.pop();
    pwd.push("avatars");
    String::from(pwd.to_str().unwrap())
}

pub const ALLOWED_AVATAR_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];

pub const MAX_AVATAR_SIZE_BYTES: usize = 1024 * 1024; // 1 MiB

// In-flight `on_validate_session` RPCs issued by ws_server itself (e.g. to authorize an avatar
// upload), keyed by request_key. The oneshot sender is fired by
// `socket_actions::on_validate_session_result` once simple_authenticator's reply arrives.
pub static PENDING_VALIDATE_SESSION_REQUESTS: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, tokio::sync::oneshot::Sender<bool>>>
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));
