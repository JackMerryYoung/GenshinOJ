use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnCloseConnection {
    #[serde(default)]
    invalidate_session: bool,
    request_key: String,
}

pub async fn on_close_connection(msg: SocketJsonMessageWithWsId) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnCloseConnection>(
            msg.content
        )
    {
        let _session_state = SESSION_STATE_LOCK.lock().await;
        let username = {
            let usernames_by_ws_id = USERNAMES_BY_WS_ID.lock().await;
            usernames_by_ws_id.get(&msg.ws_id).cloned()
        };
        let Some(username) = username else {
            return;
        };

        // A user can restore a session on a new websocket before the old socket finishes
        // closing. Only the current ws_id may invalidate the session; otherwise an old close
        // would log the user out of the new connection.
        let is_current_connection = {
            let ws_ids_by_username = WS_IDS_BY_USERNAME.lock().await;
            ws_ids_by_username.get(&username).map(String::as_str) == Some(msg.ws_id.as_str())
        };

        USERNAMES_BY_WS_ID.lock().await.remove(&msg.ws_id);
        if !is_current_connection {
            return;
        }

        LOGGED_IN_USERNAMES.lock().await.remove(&username);
        WS_IDS_BY_USERNAME.lock().await.remove(&username);
        if content.invalidate_session {
            SESSION_TOKENS_BY_USERNAME.lock().await.remove(&username);
        }

        println!(
            "{}",
            ansi_term::Color::Blue.paint(
                format!(
                    "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` disconnected{}.",
                    MODULE_IDENTITY,
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                    username,
                    if content.invalidate_session {
                        " and the session was invalidated"
                    } else {
                        ""
                    }
                )
            )
        );
    }
}
