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
                } else {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is not supported.",
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
