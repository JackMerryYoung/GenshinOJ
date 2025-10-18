#![feature(thread_id_value)]
type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

fn new_async_modifiable<T>(x: T) -> AsyncModifiable<T> {
    std::sync::Arc::new(tokio::sync::Mutex::new(x))
}

pub struct ModuleStatus {
    initialized: bool,
    panicked: bool,
    socket_port: AsyncModifiable<u16>,
}

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
        initialized: true,
        panicked: false,
        socket_port: new_async_modifiable(0),
    };
    let db_connector_status: AsyncModifiable<ModuleStatus> =
        new_async_modifiable(db_connector_status);
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
