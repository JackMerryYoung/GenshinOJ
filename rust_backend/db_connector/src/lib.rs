#![feature(thread_id_value)]
type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

fn new_async_modifiable<T>(x: T) -> AsyncModifiable<T> {
    std::sync::Arc::new(tokio::sync::Mutex::new(x))
}

#[derive(Debug)]
pub struct ModuleStatus {
    initialized: bool,
    panicked: bool,
    // The database connector does not expose an inter-module socket, but this field is required
    // to keep ModuleStatus layout-compatible with main_backend's dynamic-library ABI.
    #[allow(dead_code)]
    socket_port: AsyncModifiable<u16>,
    init_notify: std::sync::Arc<tokio::sync::Notify>,
}

static GLOBAL_MODULE_STATUSES_BY_PROTOCOL: std::sync::OnceLock<
    AsyncModifiable<std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>>
> = std::sync::OnceLock::new();

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>
    >
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let db_connector_runtime: tokio::runtime::Runtime = tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let db_connector_status: ModuleStatus = ModuleStatus {
        initialized: false,
        panicked: false,
        socket_port: new_async_modifiable(0),
        init_notify: std::sync::Arc::new(tokio::sync::Notify::new()),
    };
    let db_connector_status: AsyncModifiable<ModuleStatus> =
        new_async_modifiable(db_connector_status);
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL.set(global_module_statuses_by_protocol).unwrap();

    {
        let db_connector_status: AsyncModifiable<ModuleStatus> = db_connector_status.clone();
        db_connector_runtime.spawn(async move {
            // Wait for initialization, notified instead of polled. The setter uses
            // notify_waiters() (not notify_one()) because main_backend's own module-loading wait
            // is a second, independent waiter on this same init_notify; notify_waiters() only
            // reaches waiters already registered at the moment it's called, so `enable()` must
            // run here before the flag check to register us immediately. (This module never
            // actually sets `initialized`, so this branch never fires today — kept consistent
            // with the other modules regardless.)
            let init_notify: std::sync::Arc<tokio::sync::Notify> = db_connector_status
                .lock().await.init_notify
                .clone();
            let notified = init_notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let already_initialized: bool = db_connector_status.lock().await.initialized;
            if !already_initialized {
                notified.await;
            }
            // Now start self management
            self_management(db_connector_status).await
        });
    }
    (db_connector_runtime, db_connector_status)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload(_unload_timeout_ms: usize) {
    println!(
        "{}",
        ansi_term::Color::Blue.paint(
            format!(
                "[DB_CONNECTOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloading the database connector...",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
    println!(
        "{}",
        ansi_term::Color::Blue.paint(
            format!(
                "[DB_CONNECTOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloaded the database connector.",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
}

async fn self_management(db_connector_status: AsyncModifiable<ModuleStatus>) {
    println!(
        "{}",
        ansi_term::Color::Blue.paint(
            format!(
                "[DB_CONNECTOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] Started self management.",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
    let global_module_statuses_by_protocol = GLOBAL_MODULE_STATUSES_BY_PROTOCOL.get().unwrap();
    let mut guard_global_module_statuses_by_protocol =
        global_module_statuses_by_protocol.lock().await;
    guard_global_module_statuses_by_protocol.insert(
        String::from("db_connect"),
        db_connector_status.clone()
    );
    let mut monitor_time_cnt: usize = 0;
    loop {
        if let Ok(guard_db_connector_status) = db_connector_status.try_lock() {
            if guard_db_connector_status.panicked {
                drop(guard_db_connector_status); // Avoid poisoning the mutex lock.
                panic!();
            }
            drop(guard_db_connector_status);
            monitor_time_cnt += 1;
            if monitor_time_cnt == 600 {
                // Show monitoring message per minute.
                println!(
                    "{}",
                    ansi_term::Color::Blue.paint(
                        format!(
                            "[DB_CONNECTOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] Status reporting: Working very well.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
                monitor_time_cnt = 0;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        } else {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    }
}
