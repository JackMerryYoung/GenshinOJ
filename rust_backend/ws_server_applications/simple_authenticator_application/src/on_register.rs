use crate::global::*;
use mysql_async::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnRegister {
    username: String,
    password: String,
    request_key: String,
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_register(
    self_rt: &tokio::runtime::Runtime,
    (ws, _ws_id): (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
    content: AsyncModifiable<serde_json::Value>,
) {
    let content: AsyncModifiable<serde_json::Value> = content.clone();
    self_rt.block_on(async move {
        let guard_content: tokio::sync::MutexGuard<'_, serde_json::Value> = content.lock().await;
        let cloned_content: serde_json::Value = guard_content.clone();
        let failed_content_reserved: serde_json::Value = guard_content.clone();
        drop(guard_content);
        match serde_json::from_value::<ContentOnRegister>(cloned_content) {
            Ok(unwrapped_content) => {
                let password_hash: String = get_hash(&unwrapped_content.password);
                println!(
                    "{}", ansi_term::Color::Blue.paint(
                        format!(
                            "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` try to register with the hash: `{}`.",
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
                > = MYSQL_DATABASE_POOL.get().unwrap().lock().await;
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
                        if results.is_empty() {
                            let guard_mysql_database_pool: tokio::sync::MutexGuard<
                                '_,
                                mysql_async::Pool
                            > = MYSQL_DATABASE_POOL.get().unwrap().lock().await;
                            let mut conn: mysql_async::Conn = guard_mysql_database_pool
                                .get_conn().await
                                .unwrap();
                            let result: Result<(), mysql_async::Error> = conn.exec_drop(
                                "INSERT INTO users (username, password) VALUES (:username, :password)",
                                mysql_async::params! {
                                    "username" => &unwrapped_content.username,
                                    "password" => &password_hash
                                }
                            ).await;
                            drop(conn);
                            drop(guard_mysql_database_pool);
                            if result.is_ok() {
                                println!(
                                    "{}", ansi_term::Color::Blue.paint(
                                        format!(
                                            "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` registered successfully.",
                                            std::thread::current().id().as_u64(),
                                            file!(),
                                            line!(),
                                            &unwrapped_content.username
                                        )
                                    )
                                );

                                let response: String = format!(
                                    r#"
                                    {{
                                        "type": "quit",
                                        "content": {{
                                            "reason": "registration_success",
                                            "request_key": "{}"
                                        }}
                                    }}
                                    "#,
                                    &unwrapped_content.request_key
                                );
                                ws.send(
                                    axum::extract::ws::Message::from(response)
                                ).await.unwrap_or_default();
                            } else {
                                println!(
                                    "{}", ansi_term::Color::Yellow.paint(
                                        format!(
                                            "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to register (Failed to get queries from the database).",
                                            file!(),
                                            line!(),
                                            std::thread::current().id().as_u64(),
                                            &unwrapped_content.username
                                        )
                                    )
                                );

                                let response: String = format!(
                                    r#"
                                    {{
                                        "type": "quit",
                                        "content": {{
                                            "reason": "registration_failure",
                                            "request_key": "{}"
                                        }}
                                    }}
                                    "#,
                                    &unwrapped_content.request_key
                                );
                                ws.send(
                                    axum::extract::ws::Message::from(response)
                                ).await.unwrap_or_default();
                            }
                        } else {
                            println!(
                                "{}", ansi_term::Color::Yellow.paint(
                                    format!(
                                        "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to register (The username already exists).",
                                        std::thread::current().id().as_u64(),
                                        file!(),
                                        line!(),
                                        &unwrapped_content.username
                                    )
                                )
                            );

                            let response: String = format!(
                                r#"
                                {{
                                    "type": "quit",
                                    "content": {{
                                        "reason": "registration_failure",
                                        "request_key": "{}"
                                    }}
                                }}
                                "#,
                                &unwrapped_content.request_key
                            );
                            ws.send(
                                axum::extract::ws::Message::from(response)
                            ).await.unwrap_or_default();
                        }
                    }
                    Err(_) => {
                        println!(
                            "{}", ansi_term::Color::Yellow.paint(
                                format!(
                                    "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to register (Failed to get queries from the database).",
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    &unwrapped_content.username
                                )
                            )
                        );

                        let response: String = format!(
                            r#"
                            {{
                                "type": "quit",
                                "content": {{
                                    "reason": "registration_failure",
                                    "request_key": "{}"
                                }}
                            }}
                            "#,
                            &unwrapped_content.request_key
                        );
                        ws.send(
                            axum::extract::ws::Message::from(response)
                        ).await.unwrap_or_default();
                    }
                }
            }
            Err(_) => {
                println!(
                    "{}", ansi_term::Color::Yellow.paint(
                        format!(
                            "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to register (The JSON message received is in wrong format).",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            failed_content_reserved["username"].as_str().unwrap_or_default()
                        )
                    )
                );

                let response: String = format!(
                    r#"
                    {{
                        "type": "quit",
                        "content": {{
                            "reason": "registration_failure",
                            "request_key": "{}"
                        }}
                    }}
                    "#,
                    failed_content_reserved["request_key"].as_str().unwrap_or_default()
                );
                ws.send(axum::extract::ws::Message::from(response)).await.unwrap_or_default();
            }
        }
    });
}