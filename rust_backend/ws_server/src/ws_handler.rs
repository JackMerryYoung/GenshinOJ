use crate::global::*;

pub async fn ip_handler(
    axum_client_ip::ClientIp(ip_addr): axum_client_ip::ClientIp,
    request: axum::extract::Request,
    next: axum::middleware::Next
) -> axum::response::Response {
    // Show IP when connected.
    println!(
        "{}",
        ansi_term::Color::Blue.paint(
            format!(
                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Connection from `{}` established.",
                MODULE_IDENTITY,
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
                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Websocket connection established.",
                MODULE_IDENTITY,
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
        // Receive messages.
        if let Ok(original_msg) = original_msg {
            match original_msg {
                axum::extract::ws::Message::Text(text) => {
                    if
                        let Ok(json_msg) = serde_json::from_str::<WebsocketServerJsonMessage>(
                            text.as_str()
                        )
                    {
                        // Received JSON message.
                        let guard_ws_server_external_listeners_by_command =
                            WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND.lock().await;
                        let json_msg_with_ws_id: SocketJsonMessageWithWsId =
                            SocketJsonMessageWithWsId {
                                r#type: json_msg.r#type,
                                content: json_msg.content,
                                request_key: uuid::Uuid::new_v4().to_string(),
                                from_protocol: String::from("std_ws_server@0.1.0"),
                                ws_id: ws_id.to_string(),
                            };
                        if
                            let Some(external_listener) =
                                (*guard_ws_server_external_listeners_by_command).get(
                                    &json_msg_with_ws_id.r#type
                                )
                        {
                            let json_msg_with_ws_id_value = serde_json
                                ::to_value(json_msg_with_ws_id)
                                .unwrap();
                            for protocol in &external_listener.protocols {
                                send_socket_json_message(
                                    &json_msg_with_ws_id_value,
                                    protocol
                                ).await;
                            }
                        } else {
                            println!(
                                "{}",
                                ansi_term::Color::Yellow.paint(
                                    format!(
                                        "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Someone tried to bind a listener whose command is not implemented.",
                                        MODULE_IDENTITY,
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
                                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Websocket connection closed.",
                                MODULE_IDENTITY,
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
