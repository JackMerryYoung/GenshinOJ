use crate::global::*;

pub async fn ip_handler(
    axum_client_ip::ClientIp(ip_addr): axum_client_ip::ClientIp,
    request: axum::extract::Request,
    next: axum::middleware::Next
) -> axum::response::Response {
    println!(
        "{}",
        ansi_term::Color::Blue.paint(
            format!(
                "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Connection from `{}` established.",
                std::thread::current().id().as_u64(),
                file!(),
                line!(),
                ip_addr
            )
        )
    );
    next.run(request).await
}

pub async fn ws_handler(
    ws_upgrade: axum::extract::ws::WebSocketUpgrade
) -> axum::response::Response {
    ws_upgrade.on_upgrade(ws_callback)
}

pub async fn ws_callback(mut ws: axum::extract::ws::WebSocket) {
    println!(
        "{}",
        ansi_term::Color::Blue.paint(
            format!(
                "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Websocket connection established.",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );

    let mut guard_ws_server_connections_cnt: tokio::sync::MutexGuard<
        '_,
        usize
    > = WS_SERVER_CONNECTIONS_CNT.lock().await;
    *guard_ws_server_connections_cnt += 1;
    drop(guard_ws_server_connections_cnt);

    //TODO: Finish `ws_callback()`.
    let ws_id: uuid::Uuid = uuid::Uuid::new_v4();
    while let Some(original_msg) = ws.recv().await {
        if let Ok(original_msg) = original_msg {
            match original_msg {
                axum::extract::ws::Message::Text(text) => {
                    if
                        let Ok(mut json_msg) = serde_json::from_str::<WebsocketServerJsonMessage>(
                            text.as_str()
                        )
                    {
                        json_msg.r#type;
                    };
                }
                axum::extract::ws::Message::Close(_) => {
                    println!(
                        "{}",
                        ansi_term::Color::Blue.paint(
                            format!(
                                "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Websocket connection closed.",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            )
                        )
                    );
                    let mut guard_ws_server_connections_cnt: tokio::sync::MutexGuard<
                        '_,
                        usize
                    > = WS_SERVER_CONNECTIONS_CNT.lock().await;
                    *guard_ws_server_connections_cnt -= 1;
                    drop(guard_ws_server_connections_cnt);
                }
                _ => {}
            }
        }
    }
}
