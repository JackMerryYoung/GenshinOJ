use crate::global::*;

use rand::Rng;
use std::str::FromStr;
use mysql_async::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnLogin {
    username: String,
    password: String,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnLogin {
    ws_id: String,
    json_msg: ContentOnLogin,
}

pub async fn on_login(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnLogin>(msg.content) {
        let unwrapped_content = content.json_msg;
        let password_hash: String = get_hash(unwrapped_content.password.as_str());
        println!(
            "{}",
            ansi_term::Color::Blue.paint(
                format!(
                    "[SIMPLE_AUTHENTICATOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` try to login with the hash: `{}`.",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                    &unwrapped_content.username,
                    &password_hash
                )
            )
        );

        let guard_mysql_database_pool: tokio::sync::MutexGuard<
            '_,
            mysql_async::Pool
        > = MYSQL_DATABASE_POOL.lock().await;
        let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
        let tmp: Result<Vec<String>, _> = conn.query(
            format!(
                "SELECT password FROM users WHERE username = \"{}\"",
                &unwrapped_content.username
            )
        ).await;
        drop(conn);
        drop(guard_mysql_database_pool);
        match tmp {
            Ok(results) => {
                if let Some(real_password_hash) = results.first() {
                    if real_password_hash == &password_hash {
                        let new_session_token: String = generate_session_token(
                            rand::rng().random_range(u32::MAX / 4..=u32::MAX)
                        );

                        println!(
                            "{}",
                            ansi_term::Color::Blue.paint(
                                format!(
                                    "[SIMPLE_AUTHENTICATOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` logged in successfully.",
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    &unwrapped_content.username
                                )
                            )
                        );
                        println!(
                            "{}",
                            ansi_term::Color::Blue.paint(
                                format!(
                                    "[SIMPLE_AUTHENTICATOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] The session token: `{}`.",
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    &new_session_token
                                )
                            )
                        );

                        let mut guard_logged_in_usernames: tokio::sync::MutexGuard<
                            '_,
                            std::collections::HashSet<String>
                        > = LOGGED_IN_USERNAMES.lock().await;
                        guard_logged_in_usernames.insert(unwrapped_content.username.clone());
                        drop(guard_logged_in_usernames);

                        let mut guard_usernames_by_ws_id: tokio::sync::MutexGuard<
                            '_,
                            std::collections::HashMap<uuid::Uuid, String>
                        > = USERNAMES_BY_WS_ID.lock().await;
                        guard_usernames_by_ws_id.insert(
                            uuid::Uuid::from_str(&content.ws_id).unwrap(),
                            unwrapped_content.username.clone()
                        );
                        drop(guard_usernames_by_ws_id);

                        let mut guard_session_tokens: tokio::sync::MutexGuard<
                            '_,
                            std::collections::HashMap<String, String>
                        > = SESSION_TOKENS_BY_USERNAME.lock().await;
                        guard_session_tokens.insert(
                            unwrapped_content.username.clone(),
                            new_session_token.clone()
                        );
                        drop(guard_session_tokens);
                    } else {
                        println!(
                            "{}",
                            ansi_term::Color::Yellow.paint(
                                format!(
                                    "[SIMPLE_AUTHENTICATOR] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (The user tried to login with a fake password).",
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    &unwrapped_content.username
                                )
                            )
                        );
                    }
                } else {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[SIMPLE_AUTHENTICATOR] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (Failed to get queries from the database).",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                &unwrapped_content.username
                            )
                        )
                    );
                }
            }
            Err(_) => {
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[SIMPLE_AUTHENTICATOR] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (Failed to get queries from the database).",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            &unwrapped_content.username
                        )
                    )
                );
            }
        }
    }
}
