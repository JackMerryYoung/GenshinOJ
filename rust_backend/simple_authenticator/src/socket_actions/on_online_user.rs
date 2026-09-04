use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnOnlineUser {
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentInOnlineUserResult {
    online_users: Vec<String>,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct OnlineUserResult {
    r#type: String,
    content: ContentInOnlineUserResult,
}

pub async fn on_online_user(msg: SocketJsonMessageWithWsId) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnOnlineUser>(
            msg.content
        )
    {
        let _session_state = SESSION_STATE_LOCK.lock().await;
        let guard_logged_in_usernames: tokio::sync::MutexGuard<
            '_,
            std::collections::HashSet<String>
        > = LOGGED_IN_USERNAMES.lock().await;
        let online_users: Vec<String> = guard_logged_in_usernames.iter().cloned().collect();
        drop(guard_logged_in_usernames);
        drop(_session_state);

        let online_user_result = OnlineUserResult {
            r#type: String::from("online_user"),
            content: ContentInOnlineUserResult {
                online_users,
                request_key: content.request_key,
            },
        };

        let msg_to_send: SocketJsonMessage = SocketJsonMessage {
            r#type: String::from("on_send_msg"),
            content: serde_json
                ::to_value(SocketJsonMessageContentOnSendMsg {
                    ws_id: msg.ws_id,
                    msg_to_send: serde_json::to_value(online_user_result).unwrap(),
                })
                .unwrap(),
            request_key: msg.request_key,
            from_protocol: String::from("std_authenticator"),
        };
        send_socket_json_message(
            &serde_json::to_value(msg_to_send).unwrap(),
            "std_ws_server"
        ).await;
    }
}
