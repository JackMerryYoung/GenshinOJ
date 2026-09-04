use futures_util::SinkExt;

use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnSendMsg {
    pub ws_id: String,
    pub msg_to_send: serde_json::Value,
}

pub async fn on_send_msg(msg: SocketJsonMessage) {
    if let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnSendMsg>(msg.content) {
        // Clone the per-connection sender and release the registry lock before awaiting the
        // socket write. A slow client must not block inserts/removals or sends to other clients.
        let ws = {
            let guard_ws_server_connections_by_ws_id = WS_SERVER_CONNECTIONS_BY_WS_ID.lock().await;
            guard_ws_server_connections_by_ws_id.get(content.ws_id.as_str()).cloned()
        };
        if let Some(ws) = ws {
            let mut guard_ws: tokio::sync::MutexGuard<
                '_,
                futures_util::stream::SplitSink<
                    axum::extract::ws::WebSocket,
                    axum::extract::ws::Message
                >
            > = ws.lock().await;
            let send_result = guard_ws
                .send(
                    axum::extract::ws::Message::from(
                        serde_json::to_string(&content.msg_to_send).unwrap()
                    )
                ).await;
            if send_result.is_err() {
                eprintln!(
                    "{}",
                    ansi_term::Color::Red.paint(
                        format!(
                            "[{}] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to send JSON Message.",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
            } else {
                println!(
                    "{}",
                    ansi_term::Color::Blue.paint(
                        format!(
                            "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Successfully send JSON Message.",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
            }
        } else {
            println!(
                "{}",
                ansi_term::Color::Yellow.paint(
                    format!(
                        "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The Websocket sender has been dropped.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    )
                )
            );
        }
    } else {
        println!(
            "{}",
            ansi_term::Color::Yellow.paint(
                format!(
                    "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is in wrong format.",
                    MODULE_IDENTITY,
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!()
                )
            )
        );
    }
}
