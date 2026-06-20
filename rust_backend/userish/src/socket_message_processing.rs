use crate::global::*;
use crate::socket_actions;

use tokio::io::AsyncReadExt;

pub async fn socket_message_processing() {
    let guard_userish_socket: tokio::sync::MutexGuard<
        '_,
        tokio::net::TcpListener
    > = USERISH_SOCKET.get().unwrap().lock().await;
    loop {
        let (mut client, _) = guard_userish_socket.accept().await.unwrap();
        client.set_nodelay(true).ok();
        tokio::spawn(async move {
            let mut buf: bytes::BytesMut = bytes::BytesMut::with_capacity(1024);
            client.read_buf(&mut buf).await.unwrap();
            let msg: Result<serde_json::Value, serde_json::Error> = serde_json::from_slice(&buf);
            if let Ok(msg) = msg {
                if msg.get("ws_id").is_some() {
                    // The JSON message is a `SocketJsonMessageWithWsId` value (forwarded by
                    // ws_server on behalf of a client).
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
                        if msg.r#type == "on_user_profile" {
                            socket_actions::on_user_profile::on_user_profile(msg).await;
                        } else if msg.r#type == "on_follow" {
                            socket_actions::on_follow::on_follow(msg).await;
                        } else if msg.r#type == "on_unfollow" {
                            socket_actions::on_unfollow::on_unfollow(msg).await;
                        } else if msg.r#type == "on_friends_list" {
                            socket_actions::on_friends_list::on_friends_list(msg).await;
                        } else if msg.r#type == "on_search_users" {
                            socket_actions::on_search_users::on_search_users(msg).await;
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
                } else if let Ok(msg) = serde_json::from_value::<SocketJsonMessage>(msg) {
                    // A direct module-to-module message (e.g. `simple_authenticator`'s reply to
                    // our own `on_validate_session` RPC).
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
                    if msg.r#type == "on_validate_session_result" {
                        socket_actions::on_validate_session_result::on_validate_session_result(msg).await;
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
