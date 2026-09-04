use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnUsernameByWsId {
    ws_id: String,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnUsernameByWsIdResult {
    username: Option<String>,
    request_key: String,
}

pub async fn on_username_by_ws_id(msg: SocketJsonMessage) {
    if let Ok(content) = serde_json::from_value::<ContentOnUsernameByWsId>(msg.content) {
        let _session_state = SESSION_STATE_LOCK.lock().await;
        let guard_usernames_by_ws_id: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<String, String>
        > = USERNAMES_BY_WS_ID.lock().await;
        let username: Option<String> = guard_usernames_by_ws_id.get(&content.ws_id).cloned();
        drop(guard_usernames_by_ws_id);
        drop(_session_state);

        let msg_to_send: SocketJsonMessage = SocketJsonMessage {
            r#type: String::from("on_username_by_ws_id_result"),
            content: serde_json
                ::to_value(ContentOnUsernameByWsIdResult {
                    username,
                    request_key: content.request_key,
                })
                .unwrap(),
            request_key: uuid::Uuid::new_v4().to_string(),
            from_protocol: String::from("std_authenticator"),
        };
        send_socket_json_message(
            &serde_json::to_value(msg_to_send).unwrap(),
            "std_judge"
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
