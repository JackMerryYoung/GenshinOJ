#[macro_export] macro_rules! retry {
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

pub static GLOBAL_MODULE_STATUSES_BY_PROTOCOL: std::sync::OnceLock<
    AsyncModifiable<std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>>,
> = std::sync::OnceLock::new();

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SingleWebsocketApplicationConfigJson {
    pub name: String,
    pub id: String,
    pub enabled: bool,
    pub dependencies: Vec<String>,
    pub unload_timeout: usize,
}

pub static WS_SERVER_SOCKET: std::sync::OnceLock<AsyncModifiable<tokio::net::TcpListener>> =
    std::sync::OnceLock::new();

pub static WS_SERVER_CONNECTIONS_CNT: std::sync::LazyLock<tokio::sync::Mutex<usize>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(0));

pub static WS_SERVER_CONNECTIONS_BY_WS_ID: std::sync::LazyLock<
    tokio::sync::Mutex<
        std::collections::HashMap<String, AsyncModifiable<axum::extract::ws::WebSocket>>,
    >,
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WebsocketServerJsonMessage {
    pub r#type: String,
    pub content: serde_json::Value,
}

pub static WS_SERVER_APPLICATIONS_LIBRARIES: std::sync::LazyLock<
    tokio::sync::Mutex<Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>>,
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(vec![]));

pub fn get_parent_path() -> String {
    // Get the parent path
    let mut pwd: std::path::PathBuf = std::env::current_dir().unwrap();
    pwd.pop();
    String::from(pwd.to_str().unwrap())
}

pub const FAKE_YIELD_NOW_MILLISECONDS: u64 = 100;

pub async fn fake_yield_now() {
    tokio::time::sleep(tokio::time::Duration::from_millis(
        FAKE_YIELD_NOW_MILLISECONDS,
    ))
    .await;
}