use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnValidateSessionAndLocate {
    from_username: String,
    session_token: String,
    to_username: String,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnValidateSessionAndLocateResult {
    session_valid: bool,
    to_ws_id: Option<String>,
    request_key: String,
}

pub async fn on_validate_session_and_locate(msg: SocketJsonMessage) {
    if
        let Ok(content) = serde_json::from_value::<ContentOnValidateSessionAndLocate>(
            msg.content
        )
    {
        let _session_state = SESSION_STATE_LOCK.lock().await;
        let guard_session_tokens_by_username: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<String, String>
        > = SESSION_TOKENS_BY_USERNAME.lock().await;
        let session_valid: bool =
            guard_session_tokens_by_username.get(&content.from_username) ==
            Some(&content.session_token);
        drop(guard_session_tokens_by_username);

        let guard_ws_ids_by_username: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<String, String>
        > = WS_IDS_BY_USERNAME.lock().await;
        let to_ws_id: Option<String> = guard_ws_ids_by_username.get(&content.to_username).cloned();
        drop(guard_ws_ids_by_username);
        drop(_session_state);

        let msg_to_send: SocketJsonMessage = SocketJsonMessage {
            r#type: String::from("on_validate_session_and_locate_result"),
            content: serde_json
                ::to_value(ContentOnValidateSessionAndLocateResult {
                    session_valid,
                    to_ws_id,
                    request_key: content.request_key,
                })
                .unwrap(),
            request_key: uuid::Uuid::new_v4().to_string(),
            from_protocol: String::from("std_authenticator"),
        };
        send_socket_json_message(
            &serde_json::to_value(msg_to_send).unwrap(),
            "std_chat_server"
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
