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
pub static WS_SERVER_CONNECTIONS_BY_WS_ID: std::sync::LazyLock<
    tokio::sync::Mutex<
        std::collections::HashMap<String, AsyncModifiable<axum::extract::ws::WebSocket>>
    >
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

#[derive(Debug)]
pub struct ExternalListener {
    pub command: String, // Represents what command they listen to.
    pub protocols: std::collections::HashSet<String>, // List of protocols implemented by modules which listen to this command.
}

/**
    # `WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND`

    Represents several external modules' listeners.

    **External listeners can be indexed by specifying commands**.
 */
pub static WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND: std::sync::LazyLock<
    AsyncModifiable<std::collections::HashMap<String, ExternalListener>>
> = std::sync::LazyLock::new(|| new_async_modifiable(std::collections::HashMap::new()));

#[derive(serde::Deserialize, serde::Serialize)]
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

pub fn get_parent_path() -> String {
    // Get the parent path
    let mut pwd: std::path::PathBuf = std::env::current_dir().unwrap();
    pwd.pop();
    String::from(pwd.to_str().unwrap())
}

const FAKE_YIELD_NOW_MILLISECONDS: u64 = 100;

pub async fn fake_yield_now(tm: u64) {
    if tm == 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(FAKE_YIELD_NOW_MILLISECONDS)).await;
    } else {
        tokio::time::sleep(tokio::time::Duration::from_millis(tm)).await;
    }
}
