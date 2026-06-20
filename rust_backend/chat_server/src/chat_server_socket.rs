use crate::global::*;
use crate::socket_actions;

use tokio::io::AsyncReadExt;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnBindListener {
    pub protocol: String,
    pub version: String,
    pub commands_to_bind: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnUnbindListener {
    pub protocol: String,
    pub version: String,
    pub commands_to_unbind: Vec<String>,
}

pub async fn connect_to_ws_server() {
    let msg_to_send = SocketJsonMessage {
        r#type: String::from("on_bind_listener"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnBindListener {
                protocol: String::from("std_chat_server"),
                version: String::from(env!("CARGO_PKG_VERSION")),
                commands_to_bind: vec![String::from("on_chat_user"), String::from("on_chat_history")],
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_chat_server"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_ws_server").await;
}

pub async fn disconnect_from_ws_server() {
    let msg_to_send = SocketJsonMessage {
        r#type: String::from("on_unbind_listener"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnUnbindListener {
                protocol: String::from("std_chat_server"),
                version: String::from(env!("CARGO_PKG_VERSION")),
                commands_to_unbind: vec![String::from("on_chat_user"), String::from("on_chat_history")],
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_chat_server"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_ws_server").await;
}

pub async fn socket_message_processing() {
    let guard_chat_server_socket: tokio::sync::MutexGuard<
        '_,
        tokio::net::TcpListener
    > = CHAT_SERVER_SOCKET.get().unwrap().lock().await;
    loop {
        let (mut client, _) = guard_chat_server_socket.accept().await.unwrap();
        client.set_nodelay(true).ok();
        tokio::spawn(async move {
            let mut buf: bytes::BytesMut = bytes::BytesMut::with_capacity(1024);
            client.read_buf(&mut buf).await.unwrap();
            let msg: Result<serde_json::Value, serde_json::Error> = serde_json::from_slice(&buf);
            if let Ok(msg) = msg {
                if msg.get("ws_id").is_some() {
                    // The JSON message is a `SocketJsonMessageWithWsId` value, forwarded from
                    // ws_server (a real client sent it).
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
                        if msg.r#type == "on_chat_user" {
                            socket_actions::on_chat_user::on_chat_user(msg).await;
                        } else if msg.r#type == "on_chat_history" {
                            socket_actions::on_chat_history::on_chat_history(msg).await;
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
                    // A direct module-to-module message (no `ws_id`) — e.g. the result of asking
                    // simple_authenticator to validate a session and locate a recipient.
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
                    if msg.r#type == "on_validate_session_and_locate_result" {
                        socket_actions::on_validate_session_and_locate_result::on_validate_session_and_locate_result(
                            msg
                        ).await;
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
