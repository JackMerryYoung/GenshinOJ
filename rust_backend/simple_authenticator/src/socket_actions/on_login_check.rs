use crate::global::*;

use tokio::io::AsyncWriteExt;

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnLoginCheck {
    username: String,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnLoginCheckResult {
    check_result: bool,
    request_key: String,
}

pub async fn on_login_check(msg: SocketJsonMessageWithWsId) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnLoginCheck>(
            msg.content
        )
    {
        let guard_logged_in_usernames = LOGGED_IN_USERNAMES.lock().await;
        let check_result = (*guard_logged_in_usernames).contains(&content.username);
        drop(guard_logged_in_usernames);
        let msg_to_send: SocketJsonMessage = SocketJsonMessage {
            r#type: String::from("on_send_msg"),
            content: serde_json
                ::to_value(SocketJsonMessageContentOnSendMsg {
                    ws_id: msg.ws_id,
                    msg_to_send: serde_json
                        ::to_value(SocketJsonMessageContentOnLoginCheckResult {
                            check_result,
                            request_key: content.request_key,
                        })
                        .unwrap(),
                })
                .unwrap(),
            request_key: msg.request_key,
            from_protocol: String::from("std_authenticator@0.1.0"),
        };

        let mut socket: tokio::net::TcpStream = crate::simple_authenticator_socket::get_socket_by_protocol(
            &msg_to_send.from_protocol
        ).await;
        socket.writable().await.unwrap();
        socket.write_all(serde_json::to_string(&msg_to_send).unwrap().as_bytes()).await.unwrap();
        socket.flush().await.unwrap();
    }
}
