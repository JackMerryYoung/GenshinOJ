use crate::global::*;

use mysql_async::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnRegister {
    username: String,
    password: String,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentInMsgToSendQuitOnRegister {
    reason: String,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct MsgToSendQuitOnRegister {
    r#type: String,
    content: ContentInMsgToSendQuitOnRegister,
}

fn generate_msg_to_send_quit_on_register_success(request_key: String) -> MsgToSendQuitOnRegister {
    MsgToSendQuitOnRegister {
        r#type: String::from("quit"),
        content: ContentInMsgToSendQuitOnRegister {
            reason: String::from("registration_success"),
            request_key,
        },
    }
}

fn generate_msg_to_send_quit_on_register_failure(request_key: String) -> MsgToSendQuitOnRegister {
    MsgToSendQuitOnRegister {
        r#type: String::from("quit"),
        content: ContentInMsgToSendQuitOnRegister {
            reason: String::from("registration_failure"),
            request_key,
        },
    }
}

pub async fn on_register(msg: SocketJsonMessageWithWsId) {
    if let Ok(unwrapped_content) = serde_json::from_value::<ContentOnRegister>(msg.content) {
        let password_hash: String = get_hash(unwrapped_content.password.as_str());
        println!(
            "{}",
            ansi_term::Color::Blue.paint(
                format!(
                    "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` try to register with the hash: `{}`.",
                    MODULE_IDENTITY,
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
        let results: Result<Vec<String>, _> = conn.query(
            format!(
                "SELECT password FROM GenshinOJ.users WHERE username = \"{}\"",
                &unwrapped_content.username
            )
        ).await;
        match results {
            Ok(results_unwrapped) => {
                if results_unwrapped.is_empty() {
                    format!(
                        "INSERT INTO GenshinOJ.users (username, password) VALUES (\"{}\", \"{}\")",
                        &unwrapped_content.username,
                        password_hash
                    )
                        .ignore(&mut conn).await
                        .unwrap();
                    let json_msg: SocketJsonMessage = SocketJsonMessage {
                        r#type: String::from("on_send_msg"),
                        content: serde_json
                            ::to_value(SocketJsonMessageContentOnSendMsg {
                                ws_id: msg.ws_id,
                                msg_to_send: serde_json
                                    ::to_value(
                                        generate_msg_to_send_quit_on_register_success(
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
                    send_socket_json_message(&json_msg_value, &String::from("std_ws_server")).await;
                    println!(
                        "{}",
                        ansi_term::Color::Blue.paint(
                            format!(
                                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` registered successfully.",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                &unwrapped_content.username
                            )
                        )
                    );
                } else {
                    let json_msg: SocketJsonMessage = SocketJsonMessage {
                        r#type: String::from("on_send_msg"),
                        content: serde_json
                            ::to_value(SocketJsonMessageContentOnSendMsg {
                                ws_id: msg.ws_id,
                                msg_to_send: serde_json
                                    ::to_value(
                                        generate_msg_to_send_quit_on_register_failure(
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
                    send_socket_json_message(&json_msg_value, &String::from("std_ws_server")).await;
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to register (The user already exists).",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                &unwrapped_content.username
                            )
                        )
                    );
                }
            }
            Err(e) => {
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to register (The SQL query is not correct).",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            &unwrapped_content.username
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
        drop(conn);
        drop(guard_mysql_database_pool);
    }
}
