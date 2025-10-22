#![feature(thread_id_value)]
use tokio::io::AsyncReadExt;

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

static CHAT_SERVER_SOCKET: std::sync::OnceLock<AsyncModifiable<tokio::net::TcpListener>> =
    std::sync::OnceLock::new();

async fn wait_for_initialized(chat_server_status: AsyncModifiable<ModuleStatus>) {
    loop {
        // Waiting for the initialization to be completed.
        let guard_chat_server_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
            chat_server_status.lock().await;
        if guard_chat_server_status.initialized && CHAT_SERVER_SOCKET.get().is_some() {
            drop(guard_chat_server_status);
            break;
        }
        drop(guard_chat_server_status);
        fake_yield_now().await;
    }
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
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let chat_server_runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let chat_server_status: ModuleStatus = ModuleStatus {
        initialized: true,
        panicked: false,
        socket_port: new_async_modifiable(0),
    };
    let chat_server_status: AsyncModifiable<ModuleStatus> =
        new_async_modifiable(chat_server_status);
    
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL
        .set(global_module_statuses_by_protocol)
        .unwrap();
    // Initialization
    {
        let chat_server_status: AsyncModifiable<ModuleStatus> = chat_server_status.clone();
        chat_server_runtime.spawn(async move {
            let guard_chat_server_status: tokio::sync::MutexGuard<
                '_,
                ModuleStatus
            > = chat_server_status.lock().await; // Get the status of the server.
            // Try to establish a socket for messaging.
            let mut chat_server_socket_port: u16 = 9000;
            let mut guard_chat_server_status_socket_port: tokio::sync::MutexGuard<
                '_,
                u16
            > = guard_chat_server_status.socket_port.lock().await;
            let chat_server_socket: tokio::net::TcpListener;
            (chat_server_socket, *guard_chat_server_status_socket_port) = loop {
                let chat_server_socket_result: Result<
                    tokio::net::TcpListener,
                    std::io::Error
                > = tokio::net::TcpListener::bind(
                    format!("localhost:{}", &chat_server_socket_port)
                ).await;
                if let Ok(x) = chat_server_socket_result {
                    println!(
                        "[CHAT_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the socket on port {}.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        chat_server_socket_port
                    );
                    break (x, chat_server_socket_port);
                } else {
                    println!(
                        "[CHAT_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to open the socket on port {}. Retrying...",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        chat_server_socket_port
                    );
                }
                if chat_server_socket_port == u16::MAX {
                    eprintln!(
                        "[CHAT_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Exceeded maximum retry times. Now quitting... ",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    );
                    panic!();
                }
                chat_server_socket_port += 1;
            };
            drop(guard_chat_server_status_socket_port);
            drop(guard_chat_server_status);
            CHAT_SERVER_SOCKET.set(new_async_modifiable(chat_server_socket)).unwrap();
        });
    }

    {
        let chat_server_status: AsyncModifiable<ModuleStatus> = chat_server_status.clone();
        chat_server_runtime.spawn(async move {
            wait_for_initialized(chat_server_status).await;
            // Now processing socket message
            socket_message_processing().await;
        });
    }

    {
        let chat_server_status: AsyncModifiable<ModuleStatus> = chat_server_status.clone();
        chat_server_runtime.spawn(async move {
            wait_for_initialized(chat_server_status.clone()).await;
            // Now start self management
            self_management(chat_server_status).await
        });
    }
    (chat_server_runtime, chat_server_status)
}

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
struct SocketJsonMessage {
    r#type: String,
    content: serde_json::Value,
    request_key: String,
    from_protocol: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnSendMsg {
    ws_id: String,
    json_msg: serde_json::Value,
}

async fn socket_message_processing() {
    let guard_chat_server_socket: tokio::sync::MutexGuard<'_, tokio::net::TcpListener> =
        CHAT_SERVER_SOCKET.get().unwrap().lock().await;
    loop {
        let (mut client, _) = guard_chat_server_socket.accept().await.unwrap();
        tokio::spawn(async move {
            client.readable().await.unwrap();
            let mut buf: bytes::BytesMut = bytes::BytesMut::with_capacity(1024);
            client.read_buf(&mut buf).await.unwrap();
            println!(
                "[CHAT_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Received socket message: {:?}",
                std::thread::current().id().as_u64(),
                file!(),
                line!(),
                serde_json::from_slice::<SocketJsonMessage>(&buf).unwrap()
            );
        });
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    println!(
        "[CHAT_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloading the chat server...",
        std::thread::current().id().as_u64(),
        file!(),
        line!()
    );
    println!(
        "[CHAT_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloaded the chat server.",
        std::thread::current().id().as_u64(),
        file!(),
        line!()
    );
}

async fn self_management(chat_server_status: AsyncModifiable<ModuleStatus>) {
    let global_module_statuses_by_protocol = GLOBAL_MODULE_STATUSES_BY_PROTOCOL.get().unwrap();
    let mut guard_global_module_statuses_by_protocol = global_module_statuses_by_protocol.lock().await;
    guard_global_module_statuses_by_protocol.insert(String::from("chat_server"), chat_server_status.clone());
    let mut monitor_time_cnt: usize = 0;
    loop {
        if let Ok(guard_chat_server_status) = chat_server_status.try_lock() {
            if guard_chat_server_status.panicked || (guard_chat_server_status.initialized && CHAT_SERVER_SOCKET.get().is_none()) {
                drop(guard_chat_server_status); // Avoid poisoning the mutex lock.
                panic!();
            }
            drop(guard_chat_server_status);
            monitor_time_cnt += 1;
            if monitor_time_cnt == 600 {
                // Show monitoring message per minute.
                println!(
                    "[CHAT_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Status reporting: Working very well.",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!()
                );
                monitor_time_cnt = 0;
            }
            fake_yield_now().await;
        } else {
            fake_yield_now().await;
        }
    }
}

const FAKE_YIELD_NOW_MILLISECONDS: u64 = 100;

async fn fake_yield_now() {
    tokio::time::sleep(tokio::time::Duration::from_millis(
        FAKE_YIELD_NOW_MILLISECONDS,
    ))
    .await;
}
