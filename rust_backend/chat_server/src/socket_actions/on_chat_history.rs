use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnChatHistory {
    username: String,
    with_username: String,
    session_token: String,
    before_id: Option<i64>,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnValidateSessionAndLocate {
    from_username: String,
    session_token: String,
    to_username: String,
    request_key: String,
}

pub async fn on_chat_history(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<ContentOnChatHistory>(msg.content) {
        let request_key: String = uuid::Uuid::new_v4().to_string();

        let mut guard_pending_chat_history_requests: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<String, PendingChatHistoryRequest>
        > = PENDING_CHAT_HISTORY_REQUESTS.lock().await;
        guard_pending_chat_history_requests.insert(request_key.clone(), PendingChatHistoryRequest {
            requester_ws_id: msg.ws_id,
            requester_username: content.username.clone(),
            with_username: content.with_username.clone(),
            before_id: content.before_id,
        });
        drop(guard_pending_chat_history_requests);
        expire_pending(&*PENDING_CHAT_HISTORY_REQUESTS, request_key.clone());

        let msg_to_send: SocketJsonMessage = SocketJsonMessage {
            r#type: String::from("on_validate_session_and_locate"),
            content: serde_json
                ::to_value(ContentOnValidateSessionAndLocate {
                    from_username: content.username,
                    session_token: content.session_token,
                    to_username: content.with_username,
                    request_key: request_key.clone(),
                })
                .unwrap(),
            request_key,
            from_protocol: String::from("std_chat_server"),
        };
        send_socket_json_message(
            &serde_json::to_value(msg_to_send).unwrap(),
            "std_authenticator"
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
