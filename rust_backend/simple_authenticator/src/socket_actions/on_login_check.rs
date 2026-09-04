use crate::global::*;

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
        let _session_state = SESSION_STATE_LOCK.lock().await;
        let guard_logged_in_usernames = LOGGED_IN_USERNAMES.lock().await;
        let check_result = (*guard_logged_in_usernames).contains(&content.username);
        drop(guard_logged_in_usernames);
        drop(_session_state);

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
            from_protocol: String::from("std_authenticator"),
        };
        send_socket_json_message(
            &serde_json::to_value(msg_to_send).unwrap(),
            "std_ws_server"
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
