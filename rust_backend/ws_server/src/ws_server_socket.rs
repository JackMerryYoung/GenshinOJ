use crate::global::*;

use tokio::io::AsyncReadExt;

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct SocketJsonMessage {
    pub r#type: String,
    pub content: serde_json::Value,
    pub request_key: String,
    pub from_protocol: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnSendMsg {
    pub ws_id: String,
    pub json_msg: serde_json::Value,
}

pub async fn socket_message_processing() {
    let guard_ws_server_socket: tokio::sync::MutexGuard<'_, tokio::net::TcpListener> =
        WS_SERVER_SOCKET.get().unwrap().lock().await;
    loop {
        let (mut client, _) = guard_ws_server_socket.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf: bytes::BytesMut = bytes::BytesMut::with_capacity(1024);
            client.read_buf(&mut buf).await.unwrap();
            let msg: Result<SocketJsonMessage, serde_json::Error> =
                serde_json::from_slice::<SocketJsonMessage>(&buf);
            if let Ok(msg) = msg {
                println!(
                    "{}", ansi_term::Color::Blue.paint(
                        format!(
                            "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Received socket message: {:?}",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            msg
                        )
                    )
                );
                if msg.r#type == "on_send_msg"
                    && let Ok(content) =
                        serde_json::from_value::<SocketJsonMessageContentOnSendMsg>(msg.content)
                {
                    let guard_ws_server_connections_by_ws_id: tokio::sync::MutexGuard<
                        '_,
                        std::collections::HashMap<
                            String,
                            std::sync::Arc<tokio::sync::Mutex<axum::extract::ws::WebSocket>>,
                        >,
                    > = WS_SERVER_CONNECTIONS_BY_WS_ID.lock().await;
                    let ws: &std::sync::Arc<tokio::sync::Mutex<axum::extract::ws::WebSocket>> =
                        guard_ws_server_connections_by_ws_id
                            .get(content.ws_id.as_str())
                            .unwrap();
                    let mut guard_ws: tokio::sync::MutexGuard<'_, axum::extract::ws::WebSocket> =
                        ws.lock().await;
                    if guard_ws
                        .send(axum::extract::ws::Message::from(
                            serde_json::to_string(&content.json_msg).unwrap(),
                        ))
                        .await
                        .is_err()
                    {
                        eprintln!(
                            "{}", ansi_term::Color::Red.paint(
                                format!(
                                    "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to send JSON Message.",
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!()
                                )
                            )
                        );
                    }
                    drop(guard_ws);
                    drop(guard_ws_server_connections_by_ws_id);
                }
            } else {
                println!(
                    "{}", ansi_term::Color::Yellow.paint(
                        format!(
                            "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is in wrong format.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
            }
        });
    }
}