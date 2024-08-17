#[no_mangle]
async extern "Rust" fn on_init() {
    println!("[WS_SERVER] [INFO] Initializing the Websocket server...");
    let app: axum::Router = axum::Router::new().route("/ws", axum::routing::get(ws_handler));
    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind("0.0.0.0:9982").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    println!("[WS_SERVER] [INFO] Initialized the Websocket server.");
}

#[no_mangle]
async extern "Rust" fn on_unload() {
    println!("[WS_SERVER] [INFO] Unloading the Websocket server...");
    println!("[WS_SERVER] [INFO] Unloaded the Websocket server.");
}

async fn ws_handler(ws: axum::extract::ws::WebSocketUpgrade) -> axum::response::Response {
    ws.on_upgrade(ws_handle_socket)
}

async fn ws_handle_socket(mut socket: axum::extract::ws::WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }
}