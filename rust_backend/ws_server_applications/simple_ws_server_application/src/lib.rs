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
        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] Unloading the Simple Websocket Server Application...",
        std::thread::current().id().as_u64()
    );
    println!(
        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] Unloaded the Simple Websocket Server Application.",
        std::thread::current().id().as_u64()
    );
}

const MYSQL_DATABASE_URL: &str = "mysql://root:123456@localhost:3306/GenshinOJ";

static MYSQL_DATABASE_POOL: std::sync::LazyLock<tokio::sync::Mutex<mysql_async::Pool>> =
    std::sync::LazyLock::new(|| {
        tokio::sync::Mutex::new(mysql_async::Pool::new(MYSQL_DATABASE_URL))
    });

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnLogin {
    username: String,
    password: String,
    request_key: String,
}

type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

#[unsafe(no_mangle)]
pub extern "Rust" fn on_login(
    self_rt: &tokio::runtime::Runtime,
    ws: &mut axum::extract::ws::WebSocket,
    content: AsyncModifiable<serde_json::Value>,
) {
    {
        let content: AsyncModifiable<serde_json::Value> = content.clone();
        self_rt.block_on(async move {
            let guard_content: tokio::sync::MutexGuard<'_, serde_json::Value> = content.lock().await;
            let cloned_content: serde_json::Value = guard_content.clone();
            let failed_content_reserved: serde_json::Value = guard_content.clone();
            drop(guard_content);
            match serde_json::from_value::<ContentOnLogin>(cloned_content) {
                Ok(unwrapped_content) => {
                    let guard_mysql_database_pool: tokio::sync::MutexGuard<'_, mysql_async::Pool> =
                    MYSQL_DATABASE_POOL.lock().await;
                    let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
                    let password_hash: String = get_hash(unwrapped_content.password.as_str());
                    println!(
                        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] The user {} try to login with the hash: {}.",
                        std::thread::current().id().as_u64(),
                        &unwrapped_content.username,
                        &password_hash
                    );
                    let tmp: Result<Vec<String>, _> = conn
                        .query(format!(
                            "SELECT password FROM users WHERE username = {}",
                            &unwrapped_content.username
                        ))
                        .await;
                    match tmp {
                        Ok(results) => {
                            if let Some(real_password_hash) = results.first() {
                                println!(
                                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] Comparing {} (original) and {} (input).",
                                    std::thread::current().id().as_u64(),
                                    real_password_hash,
                                    &password_hash
                                );
                                if real_password_hash == &password_hash {
                                    let new_session_token: String = generate_session_token(
                                        rand::rng().random_range(u32::MAX / 4..=u32::MAX)
                                    );
                                    println!(
                                        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] The user {} logged in successfully.", 
                                        std::thread::current().id().as_u64(),
                                        &unwrapped_content.username
                                    );
                                    println!(
                                        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [INFO] [THREAD {}] The session token: {}.", 
                                        std::thread::current().id().as_u64(),
                                        &new_session_token
                                    );
                                    let response: String = format!(
                                        r#"
                                        {{
                                            "type": "session_token",
                                            "content": {{
                                                "session_token": {},
                                                "request_key": {},
                                            }},
                                        }}
                                        "#,
                                        &new_session_token,
                                        &unwrapped_content.request_key
                                    );
                                    ws.send(axum::extract::ws::Message::from(response))
                                        .await
                                        .unwrap_or_default();
                                }
                                else {
                                    println!(
                                        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] The user {} failed to login.",
                                        std::thread::current().id().as_u64(),
                                        &unwrapped_content.username
                                    );
                                    let response: String = format!(
                                        r#"
                                        {{
                                            "type": "quit",
                                            "content": {{
                                                "reason": "authentication_failure",
                                                "request_key": {},
                                            }},
                                        }}
                                        "#,
                                        &unwrapped_content.request_key
                                    );
                                    ws.send(axum::extract::ws::Message::from(response))
                                        .await
                                        .unwrap_or_default();
                                }
                            } else {
                                println!(
                                    "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] The user {} failed to login.",
                                    std::thread::current().id().as_u64(),
                                    &unwrapped_content.username
                                );
                            }
                        }
                        Err(_) => {
                            println!(
                                "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] The user {} failed to login.",
                                std::thread::current().id().as_u64(),
                                &unwrapped_content.username
                            );
                            let response: String = format!(
                                r#"
                            {{
                                "type": "quit",
                                "content": {{
                                    "reason": "authentication_failure",
                                    "request_key": {},
                                }},
                            }}
                            "#,
                                &unwrapped_content.request_key
                            );
                            ws.send(axum::extract::ws::Message::from(response))
                                .await
                                .unwrap_or_default();
                        }
                    };
                }
                Err(_) => {
                    println!(
                        "[WS_SERVER::SIMPLE_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] The user {} failed to login.",
                        std::thread::current().id().as_u64(),
                        failed_content_reserved["username"].as_str().unwrap_or_default()
                    );
                    let response: String = format!(
                        r#"
                        {{
                            "type": "quit",
                            "content": {{
                                "reason": "authentication_failure",
                                "request_key": {},
                            }},
                        }}
                        "#,
                        failed_content_reserved["request_key"].as_str().unwrap_or_default()
                    );
                    ws.send(axum::extract::ws::Message::from(response))
                        .await
                        .unwrap_or_default();
                }
            };
        });
    }
}

fn get_hash(text: &str) -> String {
    let mut md5_hasher = crypto::md5::Md5::new();
    md5_hasher.input_str("add-some-salt");
    md5_hasher.input_str(text);
    md5_hasher.result_str()
}

fn generate_session_token(session_token_seed: u32) -> String {
    if session_token_seed > 1 {
        let generated_session_token: String = char::from_u32(session_token_seed % 26 + 'a' as u32)
            .unwrap()
            .to_string()
            + char::from_u32(session_token_seed * 3 % 26 + 'a' as u32)
                .unwrap()
                .to_string()
                .as_str()
            + char::from_u32(session_token_seed * 5 % 26 + 'a' as u32)
                .unwrap()
                .to_string()
                .as_str()
            + char::from_u32(session_token_seed * 7 % 26 + 'a' as u32)
                .unwrap()
                .to_string()
                .as_str()
            + char::from_u32(session_token_seed * 9 % 26 + 'a' as u32)
                .unwrap()
                .to_string()
                .as_str()
            + char::from_u32(session_token_seed * 11 % 26 + 'a' as u32)
                .unwrap()
                .to_string()
                .as_str()
            + char::from_u32(session_token_seed * 13 % 26 + 'a' as u32)
                .unwrap()
                .to_string()
                .as_str()
            + char::from_u32(session_token_seed * 15 % 26 + 'a' as u32)
                .unwrap()
                .to_string()
                .as_str();
        return generated_session_token + generate_session_token(session_token_seed / 5).as_str();
    }
    String::from("s")
}
