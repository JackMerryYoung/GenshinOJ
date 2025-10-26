use tokio::io::AsyncWriteExt;

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

    {
        let mut guard_ws_server_connections_cnt: tokio::sync::MutexGuard<
            '_,
            usize
        > = WS_SERVER_CONNECTIONS_CNT.lock().await;
        *guard_ws_server_connections_cnt += 1;
        drop(guard_ws_server_connections_cnt);
    }

    let ws_id: uuid::Uuid = uuid::Uuid::new_v4();
    while let Some(original_msg) = ws.recv().await {
        if let Ok(original_msg) = original_msg {
            match original_msg {
                axum::extract::ws::Message::Text(text) => {
                    if
                        let Ok(json_msg) = serde_json::from_str::<WebsocketServerJsonMessage>(
                            text.as_str()
                        )
                    {
                        let ws_server_external_listeners_by_command =
                            WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND.get().unwrap();
                        let guard_ws_server_external_listeners_by_command =
                            ws_server_external_listeners_by_command.lock().await;
                        let json_msg_with_ws_id = WebsocketServerJsonMessageWithWsId {
                            r#type: json_msg.r#type,
                            content: json_msg.content,
                            ws_id: ws_id.to_string(),
                        };
                        if
                            let Some(external_listener) =
                                (*guard_ws_server_external_listeners_by_command).get(
                                    &json_msg_with_ws_id.r#type
                                )
                        {
                            for protocol in &external_listener.protocols {
                                let mut socket =
                                    crate::ws_server_socket::get_socket_by_protocol(protocol).await;
                                let json_msg_str = serde_json
                                    ::to_string(&json_msg_with_ws_id)
                                    .unwrap();
                                let x = socket.write_all(json_msg_str.as_bytes()).await;
                                if x.is_err() {
                                    println!(
                                        "{}",
                                        ansi_term::Color::Yellow.paint(
                                            format!(
                                                "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to send message to protocol `{}`.",
                                                std::thread::current().id().as_u64(),
                                                &protocol,
                                                file!(),
                                                line!()
                                            )
                                        )
                                    );
                                }
                            }
                        } else {
                            println!(
                                "{}",
                                ansi_term::Color::Yellow.paint(
                                    format!(
                                        "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Someone tried to bind a listener whose command is not implemented.",
                                        std::thread::current().id().as_u64(),
                                        file!(),
                                        line!()
                                    )
                                )
                            );
                        }
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
