use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnChatUser {
    from: String,
    to: String,
    messages: String,
    session_token: String,
    #[allow(dead_code)]
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnValidateSessionAndLocate {
    from_username: String,
    session_token: String,
    to_username: String,
    request_key: String,
}

pub async fn on_chat_user(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<ContentOnChatUser>(msg.content) {
        let request_key: String = uuid::Uuid::new_v4().to_string();

        let mut guard_pending_chat_requests: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<String, PendingChatRequest>
        > = PENDING_CHAT_REQUESTS.lock().await;
        guard_pending_chat_requests.insert(request_key.clone(), PendingChatRequest {
            from_ws_id: msg.ws_id,
            from_username: content.from.clone(),
            to_username: content.to.clone(),
            messages: content.messages,
        });
        drop(guard_pending_chat_requests);
        expire_pending(&*PENDING_CHAT_REQUESTS, request_key.clone());

        let msg_to_send: SocketJsonMessage = SocketJsonMessage {
            r#type: String::from("on_validate_session_and_locate"),
            content: serde_json
                ::to_value(ContentOnValidateSessionAndLocate {
                    from_username: content.from,
                    session_token: content.session_token,
                    to_username: content.to,
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
