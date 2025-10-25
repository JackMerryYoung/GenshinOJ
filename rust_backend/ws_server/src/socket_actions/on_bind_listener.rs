use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnBindListener {
    pub protocol: String,
    pub commands_to_bind: Vec<String>,
}

pub async fn on_bind_listener(msg: SocketJsonMessage) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnBindListener>(
            msg.content
        )
    {
        // let guard_ws_server_connections_by_ws_id: tokio::sync::MutexGuard<
        //     '_,
        //     std::collections::HashMap<
        //         String,
        //         std::sync::Arc<tokio::sync::Mutex<axum::extract::ws::WebSocket>>
        //     >
        // > = WS_SERVER_CONNECTIONS_BY_WS_ID.lock().await;
        let ws_server_external_listeners_by_protocol =
            WS_SERVER_EXTERNAL_LISTENERS_BY_PROTOCOL.get().unwrap();
        let mut guard_ws_server_external_listeners_by_protocol =
            ws_server_external_listeners_by_protocol.lock().await;
        let external_listener = (*guard_ws_server_external_listeners_by_protocol)
            .get_mut(&content.protocol)
            .unwrap();
        for command_to_bind in content.commands_to_bind {
            external_listener.commands_listening.insert(command_to_bind);
        }
    } else {
        println!(
            "{}",
            ansi_term::Color::Yellow.paint(
                format!(
                    "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is in wrong format.",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!()
                )
            )
        );
    }
}
