use futures_util::StreamExt;

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
    loop {
        if
            WS_SERVER_CONNECTIONS_CNT.try_lock().is_ok() &&
            WS_SERVER_CONNECTIONS_BY_WS_ID.try_lock().is_ok()
        {
            break;
        }
        fake_yield_now(0).await;
    }
    ws_upgrade.on_upgrade(ws_callback)
}

#[allow(unused_parens)]
pub async fn ws_callback(ws: axum::extract::ws::WebSocket) {
    let (ws_sender, mut ws_receiver) = ws.split();
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
    let mut guard_ws_server_connections_by_ws_id: tokio::sync::MutexGuard<
        '_,
        std::collections::HashMap<
            String,
            AsyncModifiable<
                futures_util::stream::SplitSink<
                    axum::extract::ws::WebSocket,
                    axum::extract::ws::Message
                >
            >
        >
    > = WS_SERVER_CONNECTIONS_BY_WS_ID.lock().await;
    guard_ws_server_connections_by_ws_id.insert(ws_id.to_string(), new_async_modifiable(ws_sender));
    drop(guard_ws_server_connections_by_ws_id);
    while let Some(original_msg) = ws_receiver.next().await {
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
                        let ws_server_external_listeners_by_command =
                            WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND.get().unwrap();
                        let guard_ws_server_external_listeners_by_command =
                            ws_server_external_listeners_by_command.lock().await;
                        let json_msg_with_ws_id: SocketJsonMessageWithWsId =
                            SocketJsonMessageWithWsId {
                                r#type: String::from("on_") + &json_msg.r#type,
                                content: json_msg.content,
                                request_key: uuid::Uuid::new_v4().to_string(),
                                from_protocol: String::from("std_ws_server"),
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
                                        "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Someone tried to call the command `{}`, which is not implemented.",
                                        MODULE_IDENTITY,
                                        std::thread::current().id().as_u64(),
                                        file!(),
                                        line!(),
                                        json_msg_with_ws_id.r#type
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
                    drop(ws_receiver); // Clear receiver.
                    let mut guard_ws_server_connections_by_ws_id: tokio::sync::MutexGuard<
                        '_,
                        std::collections::HashMap<
                            String,
                            AsyncModifiable<
                                futures_util::stream::SplitSink<
                                    axum::extract::ws::WebSocket,
                                    axum::extract::ws::Message
                                >
                            >
                        >
                    > = WS_SERVER_CONNECTIONS_BY_WS_ID.lock().await;
                    guard_ws_server_connections_by_ws_id.remove(&ws_id.to_string()); // Clear sender.
                    drop(guard_ws_server_connections_by_ws_id);
                    let mut guard_ws_server_connections_cnt: tokio::sync::MutexGuard<
                        '_,
                        usize
                    > = WS_SERVER_CONNECTIONS_CNT.lock().await;
                    *guard_ws_server_connections_cnt -= 1; // Decrease connections count.
                    drop(guard_ws_server_connections_cnt);
                    return;
                }
                _ => {}
            }
        }
    }
}
