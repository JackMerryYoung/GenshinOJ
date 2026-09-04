use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnQuit {
    username: String,
    session_token: String,
}

pub async fn on_quit(msg: SocketJsonMessageWithWsId) {
    let Ok(content) = serde_json::from_value::<ContentOnQuit>(msg.content) else {
        return;
    };

    let _session_state = SESSION_STATE_LOCK.lock().await;
    let username_for_ws = {
        let usernames_by_ws_id = USERNAMES_BY_WS_ID.lock().await;
        usernames_by_ws_id.get(&msg.ws_id).cloned()
    };
    if username_for_ws.as_deref() != Some(content.username.as_str()) {
        return;
    }

    let token_matches = {
        let session_tokens = SESSION_TOKENS_BY_USERNAME.lock().await;
        session_tokens.get(&content.username) == Some(&content.session_token)
    };
    if !token_matches {
        return;
    }

    SESSION_TOKENS_BY_USERNAME
        .lock()
        .await
        .remove(&content.username);
    LOGGED_IN_USERNAMES.lock().await.remove(&content.username);
    USERNAMES_BY_WS_ID.lock().await.remove(&msg.ws_id);

    let mut ws_ids_by_username = WS_IDS_BY_USERNAME.lock().await;
    if ws_ids_by_username
        .get(&content.username)
        .map(String::as_str)
        == Some(msg.ws_id.as_str())
    {
        ws_ids_by_username.remove(&content.username);
    }
}
