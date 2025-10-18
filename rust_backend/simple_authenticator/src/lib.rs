#![feature(thread_id_value)]

use std::str::FromStr;

use crypto::digest::Digest;
use mysql_async::prelude::*;
use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

fn new_async_modifiable<T>(x: T) -> AsyncModifiable<T> {
    std::sync::Arc::new(tokio::sync::Mutex::new(x))
}

#[derive(Debug)]
pub struct ModuleStatus {
    initialized: bool,
    panicked: bool,
    socket_port: AsyncModifiable<u16>,
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    _rt: &'static tokio::runtime::Runtime,
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
    >,
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let simple_authenticator_runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL
        .set(global_module_statuses_by_protocol.clone())
        .unwrap();
    let simple_authenticator_status: ModuleStatus = ModuleStatus {
        initialized: true,
        panicked: false,
        socket_port: new_async_modifiable(0),
    };
    let simple_authenticator_status: AsyncModifiable<ModuleStatus> =
        new_async_modifiable(simple_authenticator_status);
    {
        let simple_authenticator_status: AsyncModifiable<ModuleStatus> =
            simple_authenticator_status.clone();
        simple_authenticator_runtime.spawn(async move {
            let guard_simple_authenticator_status: tokio::sync::MutexGuard<
                '_,
                ModuleStatus
            > = simple_authenticator_status.lock().await; // Get the status of the server.
            // Try to establish a socket for messaging.
            let mut simple_authenticator_socket_port: u16 = 9000;
            let mut guard_simple_authenticator_status_socket_port: tokio::sync::MutexGuard<
                '_,
                u16
            > = guard_simple_authenticator_status.socket_port.lock().await;
            let simple_authenticator_socket: tokio::net::TcpListener;
            (simple_authenticator_socket, *guard_simple_authenticator_status_socket_port) = loop {
                let simple_authenticator_socket_result: Result<
                    tokio::net::TcpListener,
                    std::io::Error
                > = tokio::net::TcpListener::bind(
                    format!("localhost:{}", &simple_authenticator_socket_port)
                ).await;
                if let Ok(x) = simple_authenticator_socket_result {
                    println!(
                        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the socket on port {}.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        simple_authenticator_socket_port
                    );
                    break (x, simple_authenticator_socket_port);
                } else {
                    println!(
                        "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to open the socket on port {}. Retrying...",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        simple_authenticator_socket_port
                    );
                }
                if simple_authenticator_socket_port == u16::MAX {
                    eprintln!(
                        "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Exceeded maximum retry times. Now quitting... ",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    );
                    panic!();
                }
                simple_authenticator_socket_port += 1;
            };
            drop(guard_simple_authenticator_status_socket_port);
            drop(guard_simple_authenticator_status);
            SIMPLE_AUTHENTICATOR_SOCKET.set(new_async_modifiable(simple_authenticator_socket)).unwrap();
        });
    }

    {
        let simple_authenticator_status: AsyncModifiable<ModuleStatus> =
            simple_authenticator_status.clone();
        simple_authenticator_runtime.spawn(async move {
            loop {
                // Waiting for the initialization to be completed.
                let guard_simple_authenticator_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
                    simple_authenticator_status.lock().await;
                if guard_simple_authenticator_status.initialized {
                    drop(guard_simple_authenticator_status);
                    break;
                }
                drop(guard_simple_authenticator_status);
                fake_yield_now().await;
            }
            // Now processing socket message
            socket_message_processing().await;
        });
    }

    {
        let simple_authenticator_status: AsyncModifiable<ModuleStatus> =
            simple_authenticator_status.clone();
        simple_authenticator_runtime.spawn(self_management(simple_authenticator_status));
    }
    (simple_authenticator_runtime, simple_authenticator_status)
}

static GLOBAL_MODULE_STATUSES_BY_PROTOCOL: std::sync::OnceLock<
    AsyncModifiable<std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>>,
> = std::sync::OnceLock::new();

static SIMPLE_AUTHENTICATOR_SOCKET: std::sync::OnceLock<AsyncModifiable<tokio::net::TcpListener>> =
    std::sync::OnceLock::new();

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

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
struct SocketJsonMessage {
    r#type: String,
    content: serde_json::Value,
    request_key: String,
    from_protocol: String,
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

async fn send_to_simple_authenticator(msg: serde_json::Value) {
    let msg: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_send_msg"),
        content: msg,
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_authenticator"),
    };
    let msg: serde_json::Value = serde_json::to_value(&msg).unwrap();
    let msg: String = serde_json::to_string(&msg).unwrap();
    let mut socket: tokio::net::TcpStream =
        get_socket_by_protocol("std_simple_authenticator").await;
    socket.writable().await.unwrap();
    socket.write_all(msg.as_bytes()).await.unwrap();
    socket.flush().await.unwrap();
}

async fn socket_message_processing() {
    let guard_simple_authenticator_socket: tokio::sync::MutexGuard<'_, tokio::net::TcpListener> =
        SIMPLE_AUTHENTICATOR_SOCKET.get().unwrap().lock().await;
    loop {
        let (mut client, _) = guard_simple_authenticator_socket.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf: bytes::BytesMut = bytes::BytesMut::with_capacity(1024);
            client.read_buf(&mut buf).await.unwrap();
            let msg: Result<SocketJsonMessage, serde_json::Error> =
                serde_json::from_slice::<SocketJsonMessage>(&buf);
            if let Ok(msg) = msg {
                println!(
                    "[CHAT_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Received socket message: {:?}",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                    msg
                );
                if msg.r#type == "on_login" {
                    if let Ok(content) =
                        serde_json::from_value::<SocketJsonMessageContentOnLogin>(msg.content)
                    {
                        let unwrapped_content = content.json_msg;
                        let password_hash: String = get_hash(unwrapped_content.password.as_str());
                        println!(
                        "{}", ansi_term::Color::Blue.paint(
                            format!(
                                "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` try to login with the hash: `{}`.",
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
                            mysql_async::Pool,
                        > = MYSQL_DATABASE_POOL.lock().await;
                        let mut conn: mysql_async::Conn =
                            guard_mysql_database_pool.get_conn().await.unwrap();
                        let tmp: Result<Vec<String>, _> = conn
                            .query(format!(
                                "SELECT password FROM users WHERE username = \"{}\"",
                                &unwrapped_content.username
                            ))
                            .await;
                        drop(conn);
                        drop(guard_mysql_database_pool);
                        match tmp {
                            Ok(results) => {
                                if let Some(real_password_hash) = results.first() {
                                    if real_password_hash == &password_hash {
                                        let new_session_token: String = generate_session_token(
                                            rand::rng().random_range(u32::MAX / 4..=u32::MAX),
                                        );

                                        println!(
                                        "{}", ansi_term::Color::Blue.paint(
                                            format!(
                                                "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` logged in successfully.",
                                                std::thread::current().id().as_u64(),
                                                file!(),
                                                line!(),
                                                &unwrapped_content.username
                                            )
                                        )
                                    );
                                        println!(
                                        "{}", ansi_term::Color::Blue.paint(
                                            format!(
                                                "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The session token: `{}`.",
                                                std::thread::current().id().as_u64(),
                                                file!(),
                                                line!(),
                                                &new_session_token
                                            )
                                        )
                                    );

                                        let mut guard_logged_in_usernames: tokio::sync::MutexGuard<
                                            '_,
                                            std::collections::HashSet<String>,
                                        > = LOGGED_IN_USERNAMES.lock().await;
                                        guard_logged_in_usernames
                                            .insert(unwrapped_content.username.clone());
                                        drop(guard_logged_in_usernames);

                                        let mut guard_usernames_by_ws_id: tokio::sync::MutexGuard<
                                            '_,
                                            std::collections::HashMap<uuid::Uuid, String>,
                                        > = USERNAMES_BY_WS_ID.lock().await;
                                        guard_usernames_by_ws_id.insert(
                                            uuid::Uuid::from_str(&content.ws_id).unwrap(),
                                            unwrapped_content.username.clone(),
                                        );
                                        drop(guard_usernames_by_ws_id);

                                        let mut guard_session_tokens: tokio::sync::MutexGuard<
                                            '_,
                                            std::collections::HashMap<String, String>,
                                        > = SESSION_TOKENS_BY_USERNAME.lock().await;
                                        guard_session_tokens.insert(
                                            unwrapped_content.username.clone(),
                                            new_session_token.clone(),
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
                                            &new_session_token, &unwrapped_content.request_key
                                        );
                                        send_to_simple_authenticator(
                                            serde_json::from_str(&response).unwrap(),
                                        )
                                        .await;
                                    } else {
                                        println!(
                                        "{}", ansi_term::Color::Yellow.paint(
                                            format!(
                                                "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (The user tried to login with a fake password).",
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
                                                "reason": "authentication_failure",
                                                "request_key": "{}"
                                            }}
                                        }}
                                        "#,
                                            &unwrapped_content.request_key
                                        );
                                        send_to_simple_authenticator(
                                            serde_json::from_str(&response).unwrap(),
                                        )
                                        .await;
                                    }
                                } else {
                                    println!(
                                        "{}", ansi_term::Color::Yellow.paint(
                                            format!(
                                                "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (Failed to get queries from the database).",
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
                                                "reason": "authentication_failure",
                                                "request_key": "{}"
                                            }}
                                        }}
                                        "#,
                                        &unwrapped_content.request_key
                                    );
                                    send_to_simple_authenticator(
                                        serde_json::from_str(&response).unwrap(),
                                    )
                                    .await;
                                }
                            }
                            Err(_) => {
                                println!(
                                "{}", ansi_term::Color::Yellow.paint(
                                    format!(
                                        "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (Failed to get queries from the database).",
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
                                            "reason": "authentication_failure",
                                            "request_key": "{}"
                                        }}
                                    }}
                                    "#,
                                    &unwrapped_content.request_key
                                );
                                send_to_simple_authenticator(
                                    serde_json::from_str(&response).unwrap(),
                                )
                                .await
                            }
                        }
                    }
                } else if msg.r#type == "on_login_check" {
                    // let guard_ws;
                    let msg: SocketJsonMessage = SocketJsonMessage {
                        r#type: String::from("on_login_check"),
                        content: serde_json::to_value(SocketJsonMessageContentOnLoginCheckResult {
                            check_result: false,
                        })
                        .unwrap(),
                        request_key: uuid::Uuid::new_v4().to_string(),
                        from_protocol: String::from("std_chat_server"),
                    };
                    let mut socket: tokio::net::TcpStream =
                        get_socket_by_protocol(&msg.from_protocol).await;
                    socket.writable().await.unwrap();
                    socket
                        .write_all(serde_json::to_string(&msg).unwrap().as_bytes())
                        .await
                        .unwrap();
                    socket.flush().await.unwrap();
                } else if msg.r#type == "on_quit" {
                    if let Ok(content) =
                        serde_json::from_value::<SocketJsonMessageContentOnQuit>(msg.content)
                    {
                        let mut guard_usernames_by_ws_id: tokio::sync::MutexGuard<
                            '_,
                            std::collections::HashMap<uuid::Uuid, String>,
                        > = USERNAMES_BY_WS_ID.lock().await;
                        if let Some(username_by_ws_id) = guard_usernames_by_ws_id
                            .get(&uuid::Uuid::from_str(&content.ws_id).unwrap())
                        {
                            let unwrapped_content: ContentOnQuit = content.json_msg;
                            let mut guard_logged_in_usernames: tokio::sync::MutexGuard<
                                '_,
                                std::collections::HashSet<String>,
                            > = LOGGED_IN_USERNAMES.lock().await;
                            if guard_logged_in_usernames.contains(username_by_ws_id) {
                                let mut guard_session_tokens_by_username: tokio::sync::MutexGuard<
                                    '_,
                                    std::collections::HashMap<String, String>,
                                > = SESSION_TOKENS_BY_USERNAME.lock().await;
                                if &unwrapped_content.username == username_by_ws_id
                                    && &unwrapped_content.session_token
                                        == guard_session_tokens_by_username
                                            .get(username_by_ws_id)
                                            .unwrap_or(&String::from(""))
                                {
                                    println!(
                                        "{}", ansi_term::Color::Blue.paint(
                                            format!(
                                                "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` quitted with session token: `{}`.",
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
                                    guard_usernames_by_ws_id
                                        .remove(&uuid::Uuid::from_str(&content.ws_id).unwrap());
                                } else {
                                    println!(
                                        "{}", ansi_term::Color::Yellow.paint(
                                            format!(
                                                "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to quit (The user wanted to quit with a fake session token).",
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
                    if let Ok(content) = serde_json::from_value::<
                        SocketJsonMessageContentOnCloseConnection,
                    >(msg.content)
                    {
                        let mut guard_usernames_by_ws_id: tokio::sync::MutexGuard<
                            '_,
                            std::collections::HashMap<uuid::Uuid, String>,
                        > = USERNAMES_BY_WS_ID.lock().await;
                        if let Some(username_by_ws_id) = guard_usernames_by_ws_id
                            .get(&uuid::Uuid::from_str(&content.ws_id).unwrap())
                        {
                            let mut guard_logged_in_usernames: tokio::sync::MutexGuard<
                                '_,
                                std::collections::HashSet<String>,
                            > = LOGGED_IN_USERNAMES.lock().await;
                            if guard_logged_in_usernames.contains(username_by_ws_id) {
                                let mut guard_session_tokens_by_username: tokio::sync::MutexGuard<
                                    '_,
                                    std::collections::HashMap<String, String>,
                                > = SESSION_TOKENS_BY_USERNAME.lock().await;
                                println!(
                                    "{}", ansi_term::Color::Blue.paint(
                                        format!(
                                            "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` quitted with session token: `{}`.",
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
                            guard_usernames_by_ws_id
                                .remove(&uuid::Uuid::from_str(&content.ws_id).unwrap());
                        }
                        drop(guard_usernames_by_ws_id);
                    }
                } else {
                    println!(
                        "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is in wrong format.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    );
                }
            } else {
                println!(
                    "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is in wrong format.",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!()
                );
            }
        });
    }
}

async fn self_management(simple_authenticator_status: AsyncModifiable<ModuleStatus>) {
    let mut monitor_time_cnt: usize = 0;
    loop {
        if let Ok(guard_simple_authenticator_status) = simple_authenticator_status.try_lock() {
            if guard_simple_authenticator_status.panicked {
                drop(guard_simple_authenticator_status); // Avoid poisoning the mutex lock.
                panic!();
            } else if guard_simple_authenticator_status.initialized
                && SIMPLE_AUTHENTICATOR_SOCKET.get().is_none()
            {
                drop(guard_simple_authenticator_status); // Avoid poisoning the mutex lock.
                panic!();
            }
            drop(guard_simple_authenticator_status);
            monitor_time_cnt += 1;
            if monitor_time_cnt == 600 {
                // Show monitoring message per minute.
                println!(
                    "[SIMPLE_AUTHENTICATOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] Status reporting: Working very well.",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                );
                monitor_time_cnt = 0;
            }
            fake_yield_now().await;
        } else {
            fake_yield_now().await;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    println!(
        "{}", ansi_term::Color::Blue.paint(
            format!(
                "[SIMPLE_AUTHENTICATOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloading the simple authenticator...",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
    println!(
        "{}",
        ansi_term::Color::Blue.paint(format!(
            "[SIMPLE_AUTHENTICATOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloaded the simple authenticator.",
            std::thread::current().id().as_u64(),
            file!(),
            line!()
        ))
    );
}

const FAKE_YIELD_NOW_MILLISECONDS: u64 = 100;

async fn fake_yield_now() {
    tokio::time::sleep(tokio::time::Duration::from_millis(
        FAKE_YIELD_NOW_MILLISECONDS,
    ))
    .await;
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
