#![feature(thread_id_value)]

use crypto::digest::Digest;
use tokio::io::AsyncWriteExt;

type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

#[derive(Debug)]
pub struct ModuleStatus {
    _initialized: bool,
    _panicked: bool,
    socket_port: AsyncModifiable<u16>,
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
    let chat_ws_server_application_runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL
        .set(global_module_statuses_by_protocol)
        .unwrap();
    chat_ws_server_application_runtime
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    println!(
        "{}", ansi_term::Color::Blue.paint(
            format!(
                "[WS_SERVER::CHAT_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloading the Chat Websocket Server Application...",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
    println!(
        "{}", ansi_term::Color::Blue.paint(
            format!(
                "[WS_SERVER::CHAT_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloaded the Chat Websocket Server Application.",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnLogin {
    username: String,
    password: String,
    request_key: String,
}

async fn get_socket_port_by_protocol(protocol: &str) -> u16 {
    let guard_global_module_statuses_by_protocol: tokio::sync::MutexGuard<
        '_,
        std::collections::HashMap<String, std::sync::Arc<tokio::sync::Mutex<ModuleStatus>>>,
    > = GLOBAL_MODULE_STATUSES_BY_PROTOCOL
        .get()
        .unwrap()
        .lock()
        .await;
    let guard_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
        guard_global_module_statuses_by_protocol
            .get(protocol)
            .unwrap()
            .lock()
            .await;
    *(guard_status.socket_port.lock().await)
}

async fn get_socket_by_protocol(protocol: &str) -> tokio::net::TcpStream {
    let socket_port: u16 = get_socket_port_by_protocol(protocol).await;
    match tokio::net::TcpStream::connect(format!("localhost:{socket_port}")).await {
        Ok(socket) => socket,
        Err(_) => {
            eprintln!(
                "{}", ansi_term::Color::Red.paint(
                    format!(
                        "[WS_SERVER::CHAT_WS_SERVER_APPLICATION] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to connect to the socket of the module implemented protocol `{}` on port {}.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        protocol,
                        socket_port
                    )
                )
            );
            panic!();
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessage {
    r#type: String,
    content: serde_json::Value,
    request_key: String,
    from_protocol: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnLoginCheck {
    check_username: String,
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_login(
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
                            "[WS_SERVER::CHAT_WS_SERVER_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` try to login with the hash: `{}`.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            &unwrapped_content.username,
                            &password_hash
                        )
                    )
                );
                let msg: SocketJsonMessage = SocketJsonMessage {
                    r#type: String::from("on_login_check"),
                    content: serde_json::to_value(SocketJsonMessageContentOnLoginCheck {
                        check_username: unwrapped_content.username
                    }).unwrap(),
                    request_key: uuid::Uuid::new_v4().to_string(),
                    from_protocol: String::from("std_chat_server")
                };
                let mut socket: tokio::net::TcpStream = get_socket_by_protocol("std_authenticator").await;
                socket.writable().await.unwrap();
                socket.write_all(serde_json::to_string(&msg).unwrap().as_bytes()).await.unwrap();
                socket.flush().await.unwrap();
            }
            Err(_) => {
                println!(
                    "{}", ansi_term::Color::Yellow.paint(
                        format!(
                            "[WS_SERVER::CHAT_WS_SERVER_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (The JSON message received is in wrong format).",
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
        }
    });
}

fn get_hash(text: &str) -> String {
    let mut md5_hasher = crypto::md5::Md5::new();
    md5_hasher.input_str("add-some-salt");
    md5_hasher.input_str(text);
    md5_hasher.result_str()
}
