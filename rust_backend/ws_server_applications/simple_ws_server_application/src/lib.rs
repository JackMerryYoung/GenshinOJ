#![feature(thread_id_value)]

use crypto::digest::Digest;
use mysql_async::prelude::*;
use rand::prelude::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(_rt: &'static tokio::runtime::Runtime) -> tokio::runtime::Runtime {
    let simple_ws_server_application_runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
    simple_ws_server_application_runtime
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    println!(
        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloading the Simple Websocket Server Application...",
        std::thread::current().id().as_u64(),
        file!(),
        line!()
    );
    println!(
        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloaded the Simple Websocket Server Application.",
        std::thread::current().id().as_u64(),
        file!(),
        line!()
    );
}

const MYSQL_DATABASE_URL: &str = "mysql://root:123456@localhost:3306/GenshinOJ";

static MYSQL_DATABASE_POOL: std::sync::LazyLock<tokio::sync::Mutex<mysql_async::Pool>> =
    std::sync::LazyLock::new(|| {
        tokio::sync::Mutex::new(mysql_async::Pool::new(MYSQL_DATABASE_URL))
    });

static SESSION_TOKENS_BY_USERNAME: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<String, String>>,
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

static LOGGED_IN_USERNAMES: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashSet<String>>,
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashSet::new()));

static USERNAMES_BY_WS_ID: std::sync::LazyLock<
    tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, String>>,
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnLogin {
    username: String,
    password: String,
    request_key: String,
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
                let password_hash: String = get_hash(unwrapped_content.password.as_str());
                println!(
                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` try to login with the hash: `{}`.",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                    &unwrapped_content.username,
                    &password_hash
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
                                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` logged in successfully.",
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    &unwrapped_content.username
                                );
                                println!(
                                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The session token: `{}`.",
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    &new_session_token
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
                                    *ws_id,
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

                                let response: String = format!(
                                    r#"
                                    {{
                                        "type": "session_token",
                                        "content": {{
                                            "session_token": "{}",
                                            "request_key": "{}"
                                        }}
                                    }}
                                    "#,
                                    &new_session_token,
                                    &unwrapped_content.request_key
                                );
                                ws.send(
                                    axum::extract::ws::Message::from(response)
                                ).await.unwrap_or_default();
                            } else {
                                println!(
                                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (The user tried to login with a fake password).",
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    &unwrapped_content.username
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
                                    &unwrapped_content.request_key
                                );
                                ws.send(
                                    axum::extract::ws::Message::from(response)
                                ).await.unwrap_or_default();
                            }
                        } else {
                            println!(
                                "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (Failed to get queries from the database).",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                &unwrapped_content.username
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
                                &unwrapped_content.request_key
                            );
                            ws.send(
                                axum::extract::ws::Message::from(response)
                            ).await.unwrap_or_default();
                        }
                    }
                    Err(_) => {
                        println!(
                            "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (Failed to get queries from the database).",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            &unwrapped_content.username
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
                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (The JSON message received is in wrong format).",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                    failed_content_reserved["username"].as_str().unwrap_or_default()
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
        let mut guard_usernames_by_ws_id: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<uuid::Uuid, String>
        > = USERNAMES_BY_WS_ID.lock().await;
        if let Some(username_by_ws_id) = guard_usernames_by_ws_id.get(ws_id) {
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
                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` quitted with session token: `{}`.",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                    username_by_ws_id,
                    guard_session_tokens_by_username
                        .get(username_by_ws_id)
                        .unwrap_or(&String::from(""))
                );
                guard_session_tokens_by_username.remove(username_by_ws_id);
                guard_logged_in_usernames.remove(username_by_ws_id);
                drop(guard_session_tokens_by_username);
            }
            drop(guard_logged_in_usernames);
            guard_usernames_by_ws_id.remove(ws_id);
        }
        drop(guard_usernames_by_ws_id);
    });
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnQuit {
    username: String,
    session_token: String,
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_quit(
    self_rt: &tokio::runtime::Runtime,
    (_ws, ws_id): (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
    content: AsyncModifiable<serde_json::Value>,
) {
    self_rt.block_on(async move {
        let mut guard_usernames_by_ws_id: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<uuid::Uuid, String>
        > = USERNAMES_BY_WS_ID.lock().await;
        if let Some(username_by_ws_id) = guard_usernames_by_ws_id.get(ws_id) {
            let guard_content: tokio::sync::MutexGuard<
                '_,
                serde_json::Value
            > = content.lock().await;
            let cloned_content: serde_json::Value = guard_content.clone();
            let failed_content_reserved: serde_json::Value = guard_content.clone();
            drop(guard_content);
            match serde_json::from_value::<ContentOnQuit>(cloned_content) {
                Ok(unwrapped_content) => {
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
                                "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` quitted with session token: `{}`.",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                username_by_ws_id,
                                guard_session_tokens_by_username
                                    .get(username_by_ws_id)
                                    .unwrap_or(&String::from(""))
                            );
                            guard_session_tokens_by_username.remove(username_by_ws_id);
                            guard_logged_in_usernames.remove(username_by_ws_id);
                            guard_usernames_by_ws_id.remove(ws_id);
                        } else {
                            println!(
                                "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to quit (The user wanted to quit with a fake session token).",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                &unwrapped_content.username
                            );
                        }
                        drop(guard_session_tokens_by_username);
                    }
                    drop(guard_logged_in_usernames);
                }
                Err(_) => {
                    println!(
                        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to quit (The JSON message received is in wrong format).",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        failed_content_reserved["username"].as_str().unwrap_or_default()
                    );
                }
            }
            drop(guard_usernames_by_ws_id);
        }
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
                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` try to register with the hash: `{}`.",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                    &unwrapped_content.username,
                    &password_hash
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
                                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` registered successfully.",
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    &unwrapped_content.username
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
                                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to register (Failed to get queries from the database).",
                                    file!(),
                                    line!(),
                                    std::thread::current().id().as_u64(),
                                    &unwrapped_content.username
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
                                "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to register (The username already exists).",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                &unwrapped_content.username
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
                            "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to register (Failed to get queries from the database).",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            &unwrapped_content.username
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
                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to register (The JSON message received is in wrong format).",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                    failed_content_reserved["username"].as_str().unwrap_or_default()
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

fn get_hash(text: &str) -> String {
    let mut md5_hasher = crypto::md5::Md5::new();
    md5_hasher.input_str("add-some-salt");
    md5_hasher.input_str(text);
    md5_hasher.result_str()
}

fn generate_session_token(session_token_seed: u32) -> String {
    if session_token_seed > 1 {
        let generated_session_token: String =
            char::from_u32((session_token_seed % 26) + ('a' as u32))
                .unwrap()
                .to_string()
                + char::from_u32(((session_token_seed * 3) % 26) + ('a' as u32))
                    .unwrap()
                    .to_string()
                    .as_str()
                + char::from_u32(((session_token_seed * 5) % 26) + ('a' as u32))
                    .unwrap()
                    .to_string()
                    .as_str()
                + char::from_u32(((session_token_seed * 7) % 26) + ('a' as u32))
                    .unwrap()
                    .to_string()
                    .as_str()
                + char::from_u32(((session_token_seed * 9) % 26) + ('a' as u32))
                    .unwrap()
                    .to_string()
                    .as_str()
                + char::from_u32(((session_token_seed * 11) % 26) + ('a' as u32))
                    .unwrap()
                    .to_string()
                    .as_str()
                + char::from_u32(((session_token_seed * 13) % 26) + ('a' as u32))
                    .unwrap()
                    .to_string()
                    .as_str()
                + char::from_u32(((session_token_seed * 15) % 26) + ('a' as u32))
                    .unwrap()
                    .to_string()
                    .as_str();
        return generated_session_token + generate_session_token(session_token_seed / 5).as_str();
    }
    String::from("s")
}
