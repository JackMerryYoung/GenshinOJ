#![feature(thread_id_value)]

use mysql_async::prelude::*;

type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

#[derive(Debug)]
pub struct ModuleStatus {
    _initialized: bool,
    _panicked: bool,
    _socket_port: AsyncModifiable<u16>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnLogin {
    username: String,
    password: String,
    request_key: String,
}

static GLOBAL_MODULE_STATUSES_BY_PROTOCOL: std::sync::OnceLock<
    AsyncModifiable<std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>>,
> = std::sync::OnceLock::new();

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    _rt: &'static tokio::runtime::Runtime,
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
    >,
) -> tokio::runtime::Runtime {
    let simple_authenticator_application_runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL
    .set(global_module_statuses_by_protocol)
    .unwrap();
    simple_authenticator_application_runtime
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    println!(
        "{}", ansi_term::Color::Blue.paint(
            format!(
                "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloading the Simple Websocket Server Application...",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
    println!(
        "{}", ansi_term::Color::Blue.paint(
            format!(
                "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloaded the Simple Websocket Server Application.",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_login(
    self_rt: &tokio::runtime::Runtime,
    (ws, ws_id): (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
    content: AsyncModifiable<serde_json::Value>,
) {
    let content: AsyncModifiable<serde_json::Value> = content.clone();
    self_rt.block_on(async move {
        let guard_content: tokio::sync::MutexGuard<'_, serde_json::Value> = content.lock().await;
        let cloned_content: serde_json::Value = guard_content.clone();
        let failed_content_reserved: serde_json::Value = guard_content.clone();
        drop(guard_content);
        match serde_json::from_value::<ContentOnLogin>(cloned_content) {
            Ok(unwrapped_content) => {
                
            }
            Err(_) => {
                println!(
                    "{}", ansi_term::Color::Yellow.paint(
                        format!(
                            "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (The JSON message received is in wrong format).",
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
                            "reason": "authentication_failure",
                            "request_key": "{}"
                        }}
                    }}
                    "#,
                    failed_content_reserved["request_key"].as_str().unwrap_or_default()
                );
                ws.send(axum::extract::ws::Message::from(response)).await.unwrap_or_default();
            }
        };
    });
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_close_connection(
    self_rt: &tokio::runtime::Runtime,
    (_ws, ws_id): (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
) {
    self_rt.block_on(async move {
        
    });
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_quit(
    self_rt: &tokio::runtime::Runtime,
    (_ws, ws_id): (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
    content: AsyncModifiable<serde_json::Value>,
) {
    self_rt.block_on(async move {
        
    });
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnOnlineUser {
    request_key: String,
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_online_user(
    self_rt: &tokio::runtime::Runtime,
    (ws, _ws_id): (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
    content: AsyncModifiable<serde_json::Value>,
) {
    self_rt.block_on(async move {
        let guard_content: tokio::sync::MutexGuard<'_, serde_json::Value> = content.lock().await;
        let cloned_content: serde_json::Value = guard_content.clone();
        drop(guard_content);
        if let Ok(unwrapped_content) = serde_json::from_value::<ContentOnOnlineUser>(cloned_content)
        {
            let guard_logged_in_usernames: tokio::sync::MutexGuard<
                '_,
                std::collections::HashSet<String>,
            > = LOGGED_IN_USERNAMES.lock().await;
            let response: String = format!(
                r#"
                {{
                    "type": "online_user",
                    "content": {{
                        "online_users": {:?},
                        "request_key": "{}"
                    }}
                }}
                "#,
                guard_logged_in_usernames.iter().collect::<Vec<&String>>(),
                &unwrapped_content.request_key
            );
            drop(guard_logged_in_usernames);
            ws.send(axum::extract::ws::Message::from(response))
                .await
                .unwrap_or_default();
        }
    });
}

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
        match serde_json::from_value::<ContentOnLogin>(cloned_content) {
            Ok(unwrapped_content) => {
                let password_hash: String = get_hash(unwrapped_content.password.as_str());
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
                        if results.is_empty() {
                            let guard_mysql_database_pool: tokio::sync::MutexGuard<
                                '_,
                                mysql_async::Pool
                            > = MYSQL_DATABASE_POOL.lock().await;
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