use futures_util::SinkExt;
use futures_util::StreamExt;

use crate::global::*;

// Application-defined WebSocket close code (4000-4999 is the private-use range) used to tell the
// frontend that a connection was dropped specifically because of the AFK idle timeout, so it can
// show a dedicated notice rather than treating it as a generic disconnect.
const AFK_CLOSE_CODE: u16 = 4000;

const AFK_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(60 * 10);

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

async fn remove_ws_connection(ws_id: &str) {
    {
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
        guard_ws_server_connections_by_ws_id.remove(ws_id); // Clear sender.
    }

    {
        let mut guard_ws_server_connections_cnt: tokio::sync::MutexGuard<
            '_,
            usize
        > = WS_SERVER_CONNECTIONS_CNT.lock().await;
        *guard_ws_server_connections_cnt -= 1;
    }
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
    }

    // Best-effort: report this connection as a site visit to the control panel, if it's loaded.
    // Spawned so reporting never delays handling the websocket.
    tokio::spawn(report_visit_to_control_panel());

    let ws_id: uuid::Uuid = uuid::Uuid::new_v4();
    let ws_id_string: String = ws_id.to_string(); // Formatted once and reused; ws_id never changes for the life of this connection.
    {
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

        guard_ws_server_connections_by_ws_id.insert(ws_id_string.clone(), new_async_modifiable(ws_sender));
    }
    
    loop {
        let next_msg = match tokio::time::timeout(AFK_TIMEOUT, ws_receiver.next()).await {
            Ok(next_msg) => next_msg,
            Err(_) => {
                // No message received within the AFK timeout. Send a close frame carrying a
                // dedicated code/reason first, so the frontend can show an "idle disconnect"
                // notice (ordinary drops carry no such code). The sender lives in the connections
                // map until remove_ws_connection() below, so fetch it before clearing.
                let ws_sender: Option<
                    AsyncModifiable<
                        futures_util::stream::SplitSink<
                            axum::extract::ws::WebSocket,
                            axum::extract::ws::Message
                        >
                    >
                > = {
                    let guard_ws_server_connections_by_ws_id = WS_SERVER_CONNECTIONS_BY_WS_ID
                        .lock().await;
                    guard_ws_server_connections_by_ws_id.get(&ws_id_string).cloned()
                };
                if let Some(ws_sender) = ws_sender {
                    let mut guard_ws_sender = ws_sender.lock().await;
                    let _ = guard_ws_sender
                        .send(
                            axum::extract::ws::Message::Close(
                                Some(axum::extract::ws::CloseFrame {
                                    code: AFK_CLOSE_CODE,
                                    reason: "afk_timeout".into(),
                                })
                            )
                        ).await;
                }
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Websocket connection closed. (Due to AFKing for over ten minutes)",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
                drop(ws_receiver); // Clear receiver.
                remove_ws_connection(&ws_id_string).await;
                return;
            }
        };
        let Some(original_msg) = next_msg else {
            break; // Stream ended.
        };
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
                        let arc_from_guard_ws_server_external_listeners_by_command: std::sync::Arc<
                            std::collections::HashMap<String, ExternalListener>
                        > = {
                            let ws_server_external_listeners_by_command =
                                WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND.get().unwrap();
                            let guard_ws_server_external_listeners_by_command: std::sync::RwLockReadGuard<'_, std::sync::Arc<std::collections::HashMap<String, ExternalListener>>> =
                                ws_server_external_listeners_by_command.read().unwrap();
                            std::sync::Arc::clone(&*guard_ws_server_external_listeners_by_command)
                        };
                        let json_msg_with_ws_id: SocketJsonMessageWithWsId =
                            SocketJsonMessageWithWsId {
                                r#type: String::from("on_") + &json_msg.r#type,
                                content: json_msg.content,
                                request_key: uuid::Uuid::new_v4().to_string(),
                                from_protocol: String::from("std_ws_server"),
                                ws_id: ws_id_string.clone(),
                            };
                        if
                            let Some(external_listener) =
                                (arc_from_guard_ws_server_external_listeners_by_command).get(
                                    &json_msg_with_ws_id.r#type
                                )
                        {
                            let json_msg_with_ws_id_value = serde_json
                                ::to_value(json_msg_with_ws_id)
                                .unwrap();
                            for protocol in external_listener.protocols.keys() {
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
                    remove_ws_connection(&ws_id_string).await;
                    return;
                }
                _ => {}
            }
        }
    }

    println!(
        "{}",
        ansi_term::Color::Blue.paint(
            format!(
                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Websocket connection terminated.",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
    remove_ws_connection(&ws_id_string).await;
}
