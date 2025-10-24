use md5::Digest;

pub type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

pub fn new_async_modifiable<T>(x: T) -> AsyncModifiable<T> {
    std::sync::Arc::new(tokio::sync::Mutex::new(x))
}

#[derive(Debug)]
pub struct ModuleStatus {
    _initialized: bool,
    _panicked: bool,
    _socket_port: AsyncModifiable<u16>,
}

pub static GLOBAL_MODULE_STATUSES_BY_PROTOCOL: std::sync::OnceLock<
    AsyncModifiable<std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>>,
> = std::sync::OnceLock::new();

pub static LOGGED_IN_USERNAMES: std::sync::OnceLock<
    AsyncModifiable<std::collections::HashSet<String>>,
> = std::sync::OnceLock::new();

pub static MYSQL_DATABASE_POOL: std::sync::OnceLock<
    AsyncModifiable<mysql_async::Pool>,
> = std::sync::OnceLock::new();

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ContentOnLogin {
    pub username: String,
    pub password: String,
    pub request_key: String,
}

pub fn get_hash(x: &String) -> String {
    let mut hasher = md5::Md5::new();
    hasher.update((String::from("add-some-salt") + x).as_bytes());
    String::from_utf8(Vec::from(hasher.finalize().as_slice())).unwrap()
}