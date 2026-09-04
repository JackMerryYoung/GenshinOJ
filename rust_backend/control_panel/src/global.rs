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

// Set role-scoped credentials in the backend process environment. Keeping them outside the
// database means clearing OJ data cannot invalidate or expose control-panel credentials.
pub static CONTROL_PANEL_ADMIN_TOKEN: std::sync::LazyLock<Option<String>> =
    std::sync::LazyLock::new(|| {
        std::env::var("CONTROL_PANEL_ADMIN_TOKEN")
            .ok()
            .filter(|token| !token.trim().is_empty())
    });

pub static CONTROL_PANEL_PROBLEM_ADMIN_TOKEN: std::sync::LazyLock<Option<String>> =
    std::sync::LazyLock::new(|| {
        std::env::var("CONTROL_PANEL_PROBLEM_ADMIN_TOKEN")
            .ok()
            .filter(|token| !token.trim().is_empty())
    });

pub static CONTROL_PANEL_COMMUNITY_ADMIN_TOKEN: std::sync::LazyLock<Option<String>> =
    std::sync::LazyLock::new(|| {
        std::env::var("CONTROL_PANEL_COMMUNITY_ADMIN_TOKEN")
            .ok()
            .filter(|token| !token.trim().is_empty())
    });

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminRole {
    Super,
    Problem,
    Community,
}

impl AdminRole {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Super => "super_admin",
            Self::Problem => "problem_admin",
            Self::Community => "community_admin",
        }
    }
}
