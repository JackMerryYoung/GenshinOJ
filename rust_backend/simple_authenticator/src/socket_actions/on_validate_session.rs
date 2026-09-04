use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnValidateSession {
    username: String,
    session_token: String,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnValidateSessionResult {
    session_valid: bool,
    request_key: String,
}

pub async fn on_validate_session(msg: SocketJsonMessage) {
    if let Ok(content) = serde_json::from_value::<ContentOnValidateSession>(msg.content) {
        let _session_state = SESSION_STATE_LOCK.lock().await;
        let guard_session_tokens_by_username = SESSION_TOKENS_BY_USERNAME.lock().await;
        let session_valid: bool =
            guard_session_tokens_by_username.get(&content.username) ==
            Some(&content.session_token);
        drop(guard_session_tokens_by_username);
        drop(_session_state);

        let msg_to_send: SocketJsonMessage = SocketJsonMessage {
            r#type: String::from("on_validate_session_result"),
            content: serde_json
                ::to_value(ContentOnValidateSessionResult {
                    session_valid,
                    request_key: content.request_key,
                })
                .unwrap(),
            request_key: uuid::Uuid::new_v4().to_string(),
            from_protocol: String::from("std_authenticator"),
        };
        // Reply to whichever module actually asked (judge for submissions, ws_server for avatar
        // uploads, etc.) instead of a single hardcoded caller.
        send_socket_json_message(
            &serde_json::to_value(msg_to_send).unwrap(),
            &msg.from_protocol
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
