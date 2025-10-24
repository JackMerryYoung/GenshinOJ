#![feature(thread_id_value)]
type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

fn new_async_modifiable<T>(x: T) -> AsyncModifiable<T> {
    std::sync::Arc::new(tokio::sync::Mutex::new(x))
}

#[derive(Debug)]
pub struct ModuleStatus {
    initialized: bool,
    panicked: bool,
    socket_port: AsyncModifiable<u16>,
}

static GLOBAL_MODULE_STATUSES_BY_PROTOCOL: std::sync::OnceLock<
    AsyncModifiable<std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>>,
> = std::sync::OnceLock::new();

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    _rt: &'static tokio::runtime::Runtime,
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
    >,
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let db_connector_runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let db_connector_status: ModuleStatus = ModuleStatus {
        initialized: false,
        panicked: false,
        socket_port: new_async_modifiable(0),
    };
    let db_connector_status: AsyncModifiable<ModuleStatus> =
        new_async_modifiable(db_connector_status);
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL
        .set(global_module_statuses_by_protocol)
        .unwrap();

    {
        let db_connector_status: AsyncModifiable<ModuleStatus> = db_connector_status.clone();
        db_connector_runtime.spawn(async move {
            loop {
                // Waiting for the initialization to be completed.
                let guard_db_connector_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
                db_connector_status.lock().await;
                if guard_db_connector_status.initialized {
                    fake_yield_now(0).await;
                    drop(guard_db_connector_status);
                    break;
                }
                drop(guard_db_connector_status);
                fake_yield_now(1000).await;
            }
            // Now start self management
            self_management(db_connector_status).await
        });
    }
    (db_connector_runtime, db_connector_status)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    println!(
        "{}", ansi_term::Color::Blue.paint(
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
        ansi_term::Color::Blue.paint(format!(
            "[DB_CONNECTOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloaded the database connector.",
            std::thread::current().id().as_u64(),
            file!(),
            line!()
        ))
    );
}

async fn self_management(db_connector_status: AsyncModifiable<ModuleStatus>) {
    println!(
        "{}", ansi_term::Color::Blue.paint(
            format!(
                "[DB_CONNECTOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] Started self management.",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
    let global_module_statuses_by_protocol = GLOBAL_MODULE_STATUSES_BY_PROTOCOL.get().unwrap();
    let mut guard_global_module_statuses_by_protocol = global_module_statuses_by_protocol.lock().await;
    guard_global_module_statuses_by_protocol.insert(String::from("db_connect"), db_connector_status.clone());
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
                    "{}", ansi_term::Color::Blue.paint(
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
            fake_yield_now(0).await;
        } else {
            fake_yield_now(1000).await;
        }
    }
}

const FAKE_YIELD_NOW_MILLISECONDS: u64 = 100;

async fn fake_yield_now(tm: u64) {
    if tm == 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(
            FAKE_YIELD_NOW_MILLISECONDS,
        ))
        .await;
    } else {
        tokio::time::sleep(tokio::time::Duration::from_millis(
            tm,
        ))
        .await;
    }
}
