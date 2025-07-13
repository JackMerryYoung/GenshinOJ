#![feature(thread_id_value)]

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(_rt: &'static tokio::runtime::Runtime) -> tokio::runtime::Runtime {
    let simple_ws_server_application_runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
    simple_ws_server_application_runtime
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    println!(
        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] Unloading the Simple Websocket Server Application...",
        std::thread::current().id().as_u64()
    );
    println!(
        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] Unloaded the Simple Websocket Server Application.",
        std::thread::current().id().as_u64()
    );
}
