use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnQuit {
    username: String,
    session_token: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnQuit {
    ws_id: String,
    json_msg: ContentOnQuit,
}

pub async fn on_quit(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnQuit>(msg.content) {
        let mut guard_usernames_by_ws_id: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<String, String>
        > = USERNAMES_BY_WS_ID.lock().await;
        if let Some(username_by_ws_id) = guard_usernames_by_ws_id.get(&content.ws_id) {
            let unwrapped_content: ContentOnQuit = content.json_msg;
            let mut guard_logged_in_usernames: tokio::sync::MutexGuard<
                '_,
                std::collections::HashSet<String>
            > = LOGGED_IN_USERNAMES.lock().await;
            if guard_logged_in_usernames.contains(username_by_ws_id) {
                let mut guard_session_tokens_by_username: tokio::sync::MutexGuard<
                    '_,
                    std::collections::HashMap<String, String>
                > = SESSION_TOKENS_BY_USERNAME.lock().await;
                if
                    let Some(session_token) =
                        guard_session_tokens_by_username.get(username_by_ws_id)
                {
                    if
                        &unwrapped_content.username == username_by_ws_id &&
                        &unwrapped_content.session_token == session_token
                    {
                        println!(
                            "{}",
                            ansi_term::Color::Blue.paint(
                                format!(
                                    "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` quitted with session token: `{}`.",
                                    MODULE_IDENTITY,
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    username_by_ws_id,
                                    session_token
                                )
                            )
                        );
                        guard_session_tokens_by_username.remove(username_by_ws_id);
                        guard_logged_in_usernames.remove(username_by_ws_id);
                        guard_usernames_by_ws_id.remove(&content.ws_id);
                    } else {
                        println!(
                            "{}",
                            ansi_term::Color::Yellow.paint(
                                format!(
                                    "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to quit (The user tried to quit with a fake session token).",
                                    MODULE_IDENTITY,
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    username_by_ws_id
                                )
                            )
                        );
                    }
                } else {
                    println!(
                        "{}",
                        ansi_term::Color::Blue.paint(
                            format!(
                                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to quit (The user tried to quit without session token).",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                username_by_ws_id
                            )
                        )
                    );
                }
                drop(guard_session_tokens_by_username);
            }
            drop(guard_logged_in_usernames);
        }
        drop(guard_usernames_by_ws_id);
    }
}
