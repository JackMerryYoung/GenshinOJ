pub static MODULE_IDENTITY: &str = "CONTROL_PANEL";

pub type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

pub fn new_async_modifiable<T>(x: T) -> AsyncModifiable<T> {
    std::sync::Arc::new(tokio::sync::Mutex::new(x))
}

// Must stay structurally identical to main_backend's `ModuleStatus` (same fields, same order,
// same types): it is handed back across the `extern "Rust"` dylib boundary as
// `AsyncModifiable<ModuleStatus>`, and main_backend reads it through its own definition.
#[derive(Debug, Clone)]
pub struct ModuleStatus {
    pub initialized: bool,
    pub panicked: bool,
    pub socket_port: AsyncModifiable<u16>,
    pub init_notify: std::sync::Arc<tokio::sync::Notify>,
}

pub const MYSQL_DATABASE_URL: &str = "mysql://root:123456@127.0.0.1:3306/";

pub static MYSQL_DATABASE_POOL: std::sync::LazyLock<tokio::sync::Mutex<mysql_async::Pool>> = std::sync::LazyLock::new(
    || { tokio::sync::Mutex::new(mysql_async::Pool::new(MYSQL_DATABASE_URL)) }
);

pub const DATABASE_NAME: &str = "RsOJ";

// Tables owned by the control panel itself. Start with `control_panel_`; the "clear
// database" truncation skips every table with that prefix, so wiping OJ data never destroys
// the visit analytics.
pub const CONTROL_PANEL_TABLE_PREFIX: &str = "control_panel_";

// Table recording one row per site visit (a new websocket connection reported by ws_server).
pub const VISITS_TABLE_NAME: &str = "control_panel_visits";

// The control panel serves an HTTP admin UI here. Bound to 127.0.0.1 on purpose: clearing the
// database is destructive, so the panel must not be reachable from the public network. Reach it
// from the server host directly, or via an SSH tunnel. If the port is taken, on_init walks
// upward until it finds a free one.
pub const CONTROL_PANEL_HTTP_HOST: &str = "127.0.0.1";
pub const CONTROL_PANEL_HTTP_PORT: u16 = 9990;
