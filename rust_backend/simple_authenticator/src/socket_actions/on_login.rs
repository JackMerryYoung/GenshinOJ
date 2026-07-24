use crate::global::*;

use rand::Rng;
use mysql_async::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnLogin {
    username: String,
    password: String,
    request_key: String,
}

mod msg_to_send_generator {
    #[derive(serde::Deserialize, serde::Serialize)]
    struct ContentInQuitOnLoginFailure {
        reason: String,
        request_key: String,
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    struct ContentInSessionTokenOnLoginSuccess {
        session_token: String,
        request_key: String,
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct SessionTokenOnLoginSuccess {
        r#type: String,
        content: ContentInSessionTokenOnLoginSuccess,
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct QuitOnLoginFailure {
        r#type: String,
        content: ContentInQuitOnLoginFailure,
    }

    pub fn generate_quit_on_login_failure(request_key: String) -> QuitOnLoginFailure {
        QuitOnLoginFailure {
            r#type: String::from("quit"),
            content: ContentInQuitOnLoginFailure {
                reason: String::from("authentication_failure"),
                request_key,
            },
        }
    }

    pub fn generate_session_token_on_login_success(
        session_token: String,
        request_key: String
    ) -> SessionTokenOnLoginSuccess {
        SessionTokenOnLoginSuccess {
            r#type: String::from("session_token"),
            content: ContentInSessionTokenOnLoginSuccess {
                session_token,
                request_key,
            },
        }
    }
}

async fn send_json_msg_to_ws_server_on_login_failure(
    original_ws_id: String,
    unwrapped_content: ContentOnLogin
) {
    let json_msg: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_send_msg"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnSendMsg {
                ws_id: original_ws_id,
                msg_to_send: serde_json
                    ::to_value(
                        msg_to_send_generator::generate_quit_on_login_failure(
                            unwrapped_content.request_key
                        )
                    )
                    .unwrap(),
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_authenticator"),
    };
    let json_msg_value = serde_json::to_value(json_msg).unwrap();
    send_socket_json_message(&json_msg_value, "std_ws_server").await;
}

async fn send_json_msg_to_ws_server_on_login_success(
    original_ws_id: String,
    unwrapped_content: ContentOnLogin,
    session_token: String
) {
    let json_msg: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_send_msg"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnSendMsg {
                ws_id: original_ws_id,
                msg_to_send: serde_json
                    ::to_value(
                        msg_to_send_generator::generate_session_token_on_login_success(
                            session_token,
                            unwrapped_content.request_key
                        )
                    )
                    .unwrap(),
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_authenticator"),
    };
    let json_msg_value = serde_json::to_value(json_msg).unwrap();
    send_socket_json_message(&json_msg_value, "std_ws_server").await;
}

pub async fn on_login(msg: SocketJsonMessageWithWsId) {
    if let Ok(unwrapped_content) = serde_json::from_value::<ContentOnLogin>(msg.content) {
        let password_hash: String = get_hash(unwrapped_content.password.as_str());
        println!(
            "{}",
            ansi_term::Color::Blue.paint(
                format!(
                    "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` try to login with the hash: `{}`.",
                    MODULE_IDENTITY,
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                    unwrapped_content.username,
                    password_hash
                )
            )
        );

        let guard_mysql_database_pool: tokio::sync::MutexGuard<
            '_,
            mysql_async::Pool
        > = MYSQL_DATABASE_POOL.lock().await;
        let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
        let results: Result<Vec<String>, _> = conn.exec(
            "SELECT password FROM RsOJ.users WHERE username = :username",
            mysql_async::params! { "username" => &unwrapped_content.username }
        ).await;
        drop(conn);
        drop(guard_mysql_database_pool);
        match results {
            Ok(results_unwrapped) => {
                if let Some(real_password_hash) = results_unwrapped.first() {
                    if real_password_hash == &password_hash {
                        let new_session_token: String = generate_session_token(
                            rand::rng().random_range(u32::MAX / 4..=u32::MAX)
                        );

                        println!(
                            "{}",
                            ansi_term::Color::Blue.paint(
                                format!(
                                    "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` logged in successfully.",
                                    MODULE_IDENTITY,
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    unwrapped_content.username
                                )
                            )
                        );
                        println!(
                            "{}",
                            ansi_term::Color::Blue.paint(
                                format!(
                                    "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] The session token: `{}`.",
                                    MODULE_IDENTITY,
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    new_session_token
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
                            std::collections::HashMap<String, String>
                        > = USERNAMES_BY_WS_ID.lock().await;
                        guard_usernames_by_ws_id.insert(
                            msg.ws_id.clone(),
                            unwrapped_content.username.clone()
                        );
                        drop(guard_usernames_by_ws_id);

                        let mut guard_ws_ids_by_username: tokio::sync::MutexGuard<
                            '_,
                            std::collections::HashMap<String, String>
                        > = WS_IDS_BY_USERNAME.lock().await;
                        guard_ws_ids_by_username.insert(
                            unwrapped_content.username.clone(),
                            msg.ws_id.clone()
                        );
                        drop(guard_ws_ids_by_username);

                        let mut guard_session_tokens: tokio::sync::MutexGuard<
                            '_,
                            std::collections::HashMap<String, String>
                        > = SESSION_TOKENS_BY_USERNAME.lock().await;
                        guard_session_tokens.insert(
                            unwrapped_content.username.clone(),
                            new_session_token.clone()
                        );
                        drop(guard_session_tokens);
                        send_json_msg_to_ws_server_on_login_success(
                            msg.ws_id,
                            unwrapped_content,
                            new_session_token
                        ).await;
                    } else {
                        println!(
                            "{}",
                            ansi_term::Color::Yellow.paint(
                                format!(
                                    "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (The user tried to login with a wrong password).",
                                    MODULE_IDENTITY,
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    unwrapped_content.username
                                )
                            )
                        );
                        send_json_msg_to_ws_server_on_login_failure(
                            msg.ws_id,
                            unwrapped_content
                        ).await;
                    }
                } else {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (The user doesn't exist).",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                unwrapped_content.username
                            )
                        )
                    );
                    send_json_msg_to_ws_server_on_login_failure(msg.ws_id, unwrapped_content).await;
                }
            }
            Err(e) => {
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (The SQL query is not correct).",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            unwrapped_content.username
                        )
                    )
                );

                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] {}",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            e
                        )
                    )
                );
            }
        }
    }
}
