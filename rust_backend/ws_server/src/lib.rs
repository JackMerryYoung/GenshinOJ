#[no_mangle]
extern "Rust" fn on_init(tokio_runtime: &tokio::runtime::Runtime) {
    println!("[WS_SERVER] [INFO] Initializing the Websocket server...");
    tokio_runtime.block_on(async {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
        let app: axum::Router = axum::Router::new()
            .route("/ws", axum::routing::get(ws_handler))
            .layer(tower_http::trace::TraceLayer::new_for_http());
        let listener_result: Result<tokio::net::TcpListener, std::io::Error> =
            tokio::net::TcpListener::bind("0.0.0.0:9982").await;
        let listener: tokio::net::TcpListener = match listener_result {
            Ok(x) => {
                println!("[WS_SERVER] [INFO] Initialized the Websocket server.");
                x
            }
            Err(_e) => {
                eprintln!(
                    "[WS_SERVER] [ERROR] Encountered error when starting the Websocket server."
                );
                panic!();
            }
        };
        axum::serve(listener, app).await.unwrap();
    });
}

#[no_mangle]
extern "Rust" fn on_unload() {
    println!("[WS_SERVER] [INFO] Unloading the Websocket server...");
    println!("[WS_SERVER] [INFO] Unloaded the Websocket server.");
}

async fn ws_handler(ws: axum::extract::ws::WebSocketUpgrade) -> axum::response::Response {
    ws.on_upgrade(ws_handle_socket)
}

async fn ws_handle_socket(mut socket: axum::extract::ws::WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg: axum::extract::ws::Message = if let Ok(msg) = msg {
            msg
        } else {
            return;
        };

        if let axum::extract::ws::Message::Text(text) = &msg {
            println!("[WS_SERVER] [INFO] Echoed message: {}", text)
        };

        if socket.send(msg).await.is_err() {
            return;
        }
    }
}
