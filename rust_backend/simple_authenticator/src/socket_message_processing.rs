use crate::global::*;
use crate::socket_actions;

use tokio::io::AsyncReadExt;

pub async fn socket_message_processing() {
    let guard_simple_authenticator_socket: tokio::sync::MutexGuard<
        '_,
        tokio::net::TcpListener
    > = SIMPLE_AUTHENTICATOR_SOCKET.get().unwrap().lock().await;
    loop {
        let (mut client, _) = guard_simple_authenticator_socket.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf: bytes::BytesMut = bytes::BytesMut::with_capacity(1024);
            client.read_buf(&mut buf).await.unwrap();
            let msg: Result<serde_json::Value, serde_json::Error> = serde_json::from_slice(&buf);
            if let Ok(msg) = msg {
                if msg.get("ws_id").is_some() {
                    // The JSON message is a `SocketJsonMessageWithWsId` value
                    if let Ok(msg) = serde_json::from_value::<SocketJsonMessageWithWsId>(msg) {
                        println!(
                            "{}",
                            ansi_term::Color::Blue.paint(
                                format!(
                                    "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Received socket message: {:?}",
                                    MODULE_IDENTITY,
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    msg
                                )
                            )
                        );
                        if msg.r#type == "on_login" {
                            socket_actions::on_login::on_login(msg).await;
                        } else if msg.r#type == "on_login_check" {
                            socket_actions::on_login_check::on_login_check(msg).await;
                        } else if msg.r#type == "on_quit" {
                            socket_actions::on_quit::on_quit(msg).await;
                        } else if msg.r#type == "on_close_connection" {
                            socket_actions::on_close_connection::on_close_connection(msg).await;
                        } else if msg.r#type == "on_online_user" {
                            socket_actions::on_online_user::on_online_user(msg).await;
                        } else if msg.r#type == "on_register" {
                            socket_actions::on_register::on_register(msg).await;
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
        });
    }
}
