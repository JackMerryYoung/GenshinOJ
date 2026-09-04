use crate::global::*;

#[derive(serde::Deserialize)]
struct ContentOnSessionRestore {
    username: String,
    session_token: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SessionRestored {
    username: String,
    session_token: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SessionRestoreFailed {
    reason: String,
    request_key: String,
}

async fn send_restore_response(ws_id: String, response_type: &str, content: serde_json::Value) {
    let msg_to_send = SocketJsonMessage {
        r#type: String::from("on_send_msg"),
        content: serde_json::to_value(SocketJsonMessageContentOnSendMsg {
            ws_id,
            msg_to_send: serde_json::json!({
                "type": response_type,
                "content": content,
            }),
        })
        .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_authenticator"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_ws_server").await;
}

pub async fn on_session_restore(msg: SocketJsonMessageWithWsId) {
    let Ok(content) = serde_json::from_value::<ContentOnSessionRestore>(msg.content) else {
        return;
    };

    let _session_state = SESSION_STATE_LOCK.lock().await;
    let session_valid = {
        let tokens = SESSION_TOKENS_BY_USERNAME.lock().await;
        tokens.get(&content.username) == Some(&content.session_token)
    };

    if !session_valid {
        drop(_session_state);
        send_restore_response(
            msg.ws_id,
            "session_restore_failed",
            serde_json::to_value(SessionRestoreFailed {
                reason: String::from("invalid_session"),
                request_key: content.request_key,
            })
            .unwrap(),
        )
        .await;
        return;
    }

    unbind_ws_identity(&msg.ws_id).await;
    let old_ws_id = {
        let mut ws_ids_by_username = WS_IDS_BY_USERNAME.lock().await;
        ws_ids_by_username.insert(content.username.clone(), msg.ws_id.clone())
    };
    {
        let mut usernames_by_ws_id = USERNAMES_BY_WS_ID.lock().await;
        if let Some(old_ws_id) = old_ws_id
            && old_ws_id != msg.ws_id
        {
            usernames_by_ws_id.remove(&old_ws_id);
        }
        usernames_by_ws_id.insert(msg.ws_id.clone(), content.username.clone());
    }
    LOGGED_IN_USERNAMES.lock().await.insert(content.username.clone());
    drop(_session_state);

    send_restore_response(
        msg.ws_id,
        "session_restored",
        serde_json::to_value(SessionRestored {
            username: content.username,
            session_token: content.session_token,
            request_key: content.request_key,
        })
        .unwrap(),
    )
    .await;
}
