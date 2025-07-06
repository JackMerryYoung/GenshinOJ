#![feature(thread_id_value)]

#[no_mangle]
pub extern "Rust" fn on_init(_rt: &'static tokio::runtime::Runtime) -> tokio::runtime::Runtime {
    let ws_server_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    ws_server_runtime.spawn(async move {
        println!(
            "[WS_SERVER] [INFO] [THREAD {}] Initializing the Websocket server...",
            std::thread::current().id().as_u64()
        );
        let ws_server_app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "Hello, world!" }))
            .route("/ws", axum::routing::get(ws_handler));
        let listener = tokio::net::TcpListener::bind("0.0.0.0:9983").await.unwrap();
        axum::serve(listener, ws_server_app).await.unwrap();
    });
    ws_server_runtime
}

#[no_mangle]
pub extern "Rust" fn on_unload() {
    println!(
        "[WS_SERVER] [INFO] [THREAD {}] Unloading the Websocket server...",
        std::thread::current().id().as_u64()
    );
    println!(
        "[WS_SERVER] [INFO] [THREAD {}] Unloaded the Websocket server.",
        std::thread::current().id().as_u64()
    );
}

async fn ws_handler(ws_upgrade: axum::extract::ws::WebSocketUpgrade) -> axum::response::Response {
    ws_upgrade.on_upgrade(ws_callback)
}

async fn ws_callback(mut ws: axum::extract::ws::WebSocket) {
    while let Some(msg) = ws.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            return;
        };

        if ws.send(msg).await.is_err() {
            return;
        }
    }
}
