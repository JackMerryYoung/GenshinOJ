use crate::global::*;

use tokio::io::AsyncReadExt;

pub async fn get_socket_port_by_protocol(protocol: &String) -> u16 {
    let guard_global_module_statuses_by_protocol: tokio::sync::MutexGuard<
        '_,
        std::collections::HashMap<String, std::sync::Arc<tokio::sync::Mutex<ModuleStatus>>>,
    > = crate::GLOBAL_MODULE_STATUSES_BY_PROTOCOL
        .get()
        .unwrap()
        .lock()
        .await;
    let guard_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
        guard_global_module_statuses_by_protocol
            .get(protocol)
            .unwrap()
            .lock()
            .await;
    *(guard_status.socket_port.lock().await)
}

pub async fn get_socket_by_protocol(protocol: &String) -> tokio::net::TcpStream {
    let socket_port: u16 = get_socket_port_by_protocol(protocol).await;
    match tokio::net::TcpStream::connect(format!("localhost:{socket_port}")).await {
        Ok(socket) => socket,
        Err(_) => {
            eprintln!(
                "{}", ansi_term::Color::Red.paint(
                    format!(
                        "[WS_SERVER::CHAT_WS_SERVER_APPLICATION] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to connect to the socket of the module implemented protocol `{}` on port {}.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        protocol,
                        socket_port
                    )
                )
            );
            panic!();
        }
    }
}

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
                    "[CHAT_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Received socket message: {:?}",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                    msg
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
                            "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to send JSON Message.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        );
                    }
                    drop(guard_ws);
                    drop(guard_ws_server_connections_by_ws_id);
                }
            } else {
                println!(
                    "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is in wrong format.",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!()
                );
            }
        });
    }
}