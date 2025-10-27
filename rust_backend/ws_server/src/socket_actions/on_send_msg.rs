use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnSendMsg {
    pub ws_id: String,
    pub msg_to_send: serde_json::Value,
}

pub async fn on_send_msg(msg: SocketJsonMessage) {
    if let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnSendMsg>(msg.content) {
        let guard_ws_server_connections_by_ws_id: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<
                String,
                std::sync::Arc<tokio::sync::Mutex<axum::extract::ws::WebSocket>>
            >
        > = WS_SERVER_CONNECTIONS_BY_WS_ID.lock().await;
        let ws: &std::sync::Arc<tokio::sync::Mutex<axum::extract::ws::WebSocket>> = guard_ws_server_connections_by_ws_id
            .get(content.ws_id.as_str())
            .unwrap();
        let mut guard_ws: tokio::sync::MutexGuard<
            '_,
            axum::extract::ws::WebSocket
        > = ws.lock().await;
        if
            guard_ws
                .send(
                    axum::extract::ws::Message::from(
                        serde_json::to_string(&content.msg_to_send).unwrap()
                    )
                ).await
                .is_err()
        {
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
        }

        drop(guard_ws);
        drop(guard_ws_server_connections_by_ws_id);
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
