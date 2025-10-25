use crate::global::*;
use crate::socket_actions;
use std::str::FromStr;
use mysql_async::prelude::*;
use rand::Rng;
use tokio::io::{ AsyncReadExt, AsyncWriteExt };

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct SocketJsonMessage {
    pub r#type: String,
    pub content: serde_json::Value,
    pub request_key: String,
    pub from_protocol: String,
}

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

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnCloseConnection {
    ws_id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnLoginCheckResult {
    check_result: bool,
}

pub async fn socket_message_processing() {
    let guard_simple_authenticator_socket: tokio::sync::MutexGuard<
        '_,
        tokio::net::TcpListener
    > = SIMPLE_AUTHENTICATOR_SOCKET.get().unwrap().lock().await;
    loop {
        let (mut client, _) = guard_simple_authenticator_socket.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf: bytes::BytesMut = bytes::BytesMut::with_capacity(1024);
            client.read_buf(&mut buf).await.unwrap();
            let msg: Result<
                SocketJsonMessage,
                serde_json::Error
            > = serde_json::from_slice::<SocketJsonMessage>(&buf);
            if let Ok(msg) = msg {
                println!(
                    "{}",
                    ansi_term::Color::Blue.paint(
                        format!(
                            "[CHAT_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Received socket message: {:?}",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            msg
                        )
                    )
                );
                if msg.r#type == "on_login" {
                    if
                        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnLogin>(
                            msg.content
                        )
                    {
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
                        let mut conn: mysql_async::Conn = guard_mysql_database_pool
                            .get_conn().await
                            .unwrap();
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
                                        guard_logged_in_usernames.insert(
                                            unwrapped_content.username.clone()
                                        );
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
                } else if msg.r#type == "on_login_check" {
                    let msg: SocketJsonMessage = SocketJsonMessage {
                        r#type: String::from("on_login_check"),
                        content: serde_json
                            ::to_value(SocketJsonMessageContentOnLoginCheckResult {
                                check_result: false,
                            })
                            .unwrap(),
                        request_key: uuid::Uuid::new_v4().to_string(),
                        from_protocol: String::from("std_authenticator@0.1.0"),
                    };
                    let mut socket: tokio::net::TcpStream = crate::simple_authenticator_socket::get_socket_by_protocol(
                        &msg.from_protocol
                    ).await;
                    socket.writable().await.unwrap();
                    socket
                        .write_all(serde_json::to_string(&msg).unwrap().as_bytes()).await
                        .unwrap();
                    socket.flush().await.unwrap();
                } else if msg.r#type == "on_quit" {
                    if
                        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnQuit>(
                            msg.content
                        )
                    {
                        let mut guard_usernames_by_ws_id: tokio::sync::MutexGuard<
                            '_,
                            std::collections::HashMap<uuid::Uuid, String>
                        > = USERNAMES_BY_WS_ID.lock().await;
                        if
                            let Some(username_by_ws_id) = guard_usernames_by_ws_id.get(
                                &uuid::Uuid::from_str(&content.ws_id).unwrap()
                            )
                        {
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
                                    &unwrapped_content.username == username_by_ws_id &&
                                    &unwrapped_content.session_token ==
                                        guard_session_tokens_by_username
                                            .get(username_by_ws_id)
                                            .unwrap_or(&String::from(""))
                                {
                                    println!(
                                        "{}",
                                        ansi_term::Color::Blue.paint(
                                            format!(
                                                "[SIMPLE_AUTHENTICATOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` quitted with session token: `{}`.",
                                                std::thread::current().id().as_u64(),
                                                file!(),
                                                line!(),
                                                username_by_ws_id,
                                                guard_session_tokens_by_username
                                                    .get(username_by_ws_id)
                                                    .unwrap_or(&String::from(""))
                                            )
                                        )
                                    );
                                    guard_session_tokens_by_username.remove(username_by_ws_id);
                                    guard_logged_in_usernames.remove(username_by_ws_id);
                                    guard_usernames_by_ws_id.remove(
                                        &uuid::Uuid::from_str(&content.ws_id).unwrap()
                                    );
                                } else {
                                    println!(
                                        "{}",
                                        ansi_term::Color::Yellow.paint(
                                            format!(
                                                "[SIMPLE_AUTHENTICATOR] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to quit (The user wanted to quit with a fake session token).",
                                                std::thread::current().id().as_u64(),
                                                file!(),
                                                line!(),
                                                &unwrapped_content.username
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
                } else if msg.r#type == "on_close_connection" {
                    if
                        let Ok(content) =
                            serde_json::from_value::<SocketJsonMessageContentOnCloseConnection>(
                                msg.content
                            )
                    {
                        let mut guard_usernames_by_ws_id: tokio::sync::MutexGuard<
                            '_,
                            std::collections::HashMap<uuid::Uuid, String>
                        > = USERNAMES_BY_WS_ID.lock().await;
                        if
                            let Some(username_by_ws_id) = guard_usernames_by_ws_id.get(
                                &uuid::Uuid::from_str(&content.ws_id).unwrap()
                            )
                        {
                            let mut guard_logged_in_usernames: tokio::sync::MutexGuard<
                                '_,
                                std::collections::HashSet<String>
                            > = LOGGED_IN_USERNAMES.lock().await;
                            if guard_logged_in_usernames.contains(username_by_ws_id) {
                                let mut guard_session_tokens_by_username: tokio::sync::MutexGuard<
                                    '_,
                                    std::collections::HashMap<String, String>
                                > = SESSION_TOKENS_BY_USERNAME.lock().await;
                                println!(
                                    "{}",
                                    ansi_term::Color::Blue.paint(
                                        format!(
                                            "[SIMPLE_AUTHENTICATOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` quitted with session token: `{}`.",
                                            std::thread::current().id().as_u64(),
                                            file!(),
                                            line!(),
                                            username_by_ws_id,
                                            guard_session_tokens_by_username
                                                .get(username_by_ws_id)
                                                .unwrap_or(&String::from(""))
                                        )
                                    )
                                );
                                guard_session_tokens_by_username.remove(username_by_ws_id);
                                guard_logged_in_usernames.remove(username_by_ws_id);
                                drop(guard_session_tokens_by_username);
                            }
                            drop(guard_logged_in_usernames);
                            guard_usernames_by_ws_id.remove(
                                &uuid::Uuid::from_str(&content.ws_id).unwrap()
                            );
                        }
                        drop(guard_usernames_by_ws_id);
                    }
                } else if msg.r#type == "on_online_user" {
                    socket_actions::on_online_user::on_online_user(msg).await;
                } else {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[SIMPLE_AUTHENTICATOR] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is in wrong format.",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            )
                        )
                    );
                }
            } else {
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[SIMPLE_AUTHENTICATOR] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is in wrong format.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
            }
        });
    }
}
