use crate::global::*;
use crate::socket_actions;

use tokio::io::AsyncReadExt;

pub async fn socket_message_processing() {
    let guard_ws_server_socket: tokio::sync::MutexGuard<
        '_,
        tokio::net::TcpListener
    > = WS_SERVER_SOCKET.get().unwrap().lock().await;
    loop {
        let (mut client, _) = guard_ws_server_socket.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf: bytes::BytesMut = bytes::BytesMut::with_capacity(65536);
            client.read_buf(&mut buf).await.unwrap();
            let msg: Result<
                SocketJsonMessage,
                serde_json::Error
            > = serde_json::from_slice::<SocketJsonMessage>(&buf);
            if let Ok(msg) = msg {
                println!(
                    "{}",
                    ansi_term::Color::Blue.paint(
                        format!(
                            "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Received socket message: {:?}",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            msg
                        )
                    )
                );
                if msg.r#type == "on_send_msg" {
                    socket_actions::on_send_msg::on_send_msg(msg).await;
                } else if msg.r#type == "on_bind_listener" {
                    socket_actions::on_bind_listener::on_bind_listener(msg).await;
                } else if msg.r#type == "on_unbind_listener" {
                    socket_actions::on_unbind_listener::on_unbind_listener(msg).await;
                } else {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is not implemented.",
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

pub async fn get_socket_port_by_protocol(protocol: &str) -> u16 {
    let guard_global_module_statuses_by_protocol: tokio::sync::MutexGuard<
        '_,
        std::collections::HashMap<String, std::sync::Arc<tokio::sync::Mutex<ModuleStatus>>>
    > = GLOBAL_MODULE_STATUSES_BY_PROTOCOL.get().unwrap().lock().await;
    let guard_status: tokio::sync::MutexGuard<
        '_,
        ModuleStatus
    > = guard_global_module_statuses_by_protocol.get(protocol).unwrap().lock().await;
    *guard_status.socket_port.lock().await
}

pub async fn get_socket_by_protocol(protocol: &str) -> tokio::net::TcpStream {
    let socket_port: u16 = get_socket_port_by_protocol(protocol).await;
    match tokio::net::TcpStream::connect(format!("127.0.0.1:{socket_port}")).await {
        Ok(socket) => socket,
        Err(_) => {
            eprintln!(
                "{}",
                ansi_term::Color::Red.paint(
                    format!(
                        "[SIMPLE_AUTHENTICATOR] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to connect to the socket of the module implemented protocol `{}` on port {}.",
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
