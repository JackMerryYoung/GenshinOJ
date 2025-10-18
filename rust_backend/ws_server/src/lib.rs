#![feature(thread_id_value)]
use std::io::Read;

use tokio::io::AsyncReadExt;

macro_rules! retry {
    ($f:expr, $count:expr, $interval:expr, $retry_func:expr) => {
        {
        let mut retries = 0;
        let result = loop {
            let result = $f;
            if result.is_ok() {
                break result;
            } else if retries > $count {
                break result;
            } else {
                retries += 1;
                $retry_func
                tokio::time::sleep(std::time::Duration::from_millis($interval)).await;
            }
        };
        result
        }
    };
    ($f:expr) => {
        retry!($f, 2, 1000, {})
    };
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SingleWebsocketApplicationConfigJson {
    name: String,
    id: String,
    enabled: bool,
    dependencies: Vec<String>,
    unload_timeout: usize,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct WebsocketServerApplicationsConfigJson {
    ws_server_applications: std::collections::HashMap<String, SingleWebsocketApplicationConfigJson>,
    restricted_mode: bool,
}

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

static GLOBAL_MODULE_STATUSES_BY_PROTOCOL: std::sync::OnceLock<
    AsyncModifiable<std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>>,
> = std::sync::OnceLock::new();

#[derive(serde::Deserialize, serde::Serialize)]
struct WebsocketServerJsonMessage {
    r#type: String,
    content: serde_json::Value,
}

static WS_SERVER_APPLICATIONS_LIBRARIES: std::sync::LazyLock<
    tokio::sync::Mutex<Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>>,
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(vec![]));

static WS_SERVER_SOCKET: std::sync::OnceLock<AsyncModifiable<tokio::net::TcpListener>> =
    std::sync::OnceLock::new();

static WS_SERVER_CONNECTIONS_CNT: std::sync::LazyLock<tokio::sync::Mutex<usize>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(0));

static WS_SERVER_CONNECTIONS_BY_WS_ID: std::sync::LazyLock<
    tokio::sync::Mutex<
        std::collections::HashMap<String, AsyncModifiable<axum::extract::ws::WebSocket>>,
    >,
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    rt: &'static tokio::runtime::Runtime,
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
    >,
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let ws_server_runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL
        .set(global_module_statuses_by_protocol.clone())
        .unwrap();
    let ws_server_status: ModuleStatus = ModuleStatus {
        initialized: false,
        panicked: false,
        socket_port: new_async_modifiable(0),
    };
    let ws_server_status: AsyncModifiable<ModuleStatus> = new_async_modifiable(ws_server_status);
    // Initialization
    {
        let ws_server_status: AsyncModifiable<ModuleStatus> = ws_server_status.clone();
        ws_server_runtime.spawn(async move {
            let guard_ws_server_status: tokio::sync::MutexGuard<
                '_,
                ModuleStatus
            > = ws_server_status.lock().await; // Get the status of the server.
            // Try to establish a socket for messaging.
            let mut ws_server_socket_port: u16 = 9000;
            let mut guard_ws_server_status_socket_port: tokio::sync::MutexGuard<
                '_,
                u16
            > = guard_ws_server_status.socket_port.lock().await;
            let ws_server_socket: tokio::net::TcpListener;
            (ws_server_socket, *guard_ws_server_status_socket_port) = loop {
                let ws_server_socket_result: Result<
                    tokio::net::TcpListener,
                    std::io::Error
                > = tokio::net::TcpListener::bind(
                    format!("localhost:{}", &ws_server_socket_port)
                ).await;
                if let Ok(x) = ws_server_socket_result {
                    println!(
                        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the socket on port {}.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        ws_server_socket_port
                    );
                    break (x, ws_server_socket_port);
                } else {
                    println!(
                        "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to open the socket on port {}. Retrying...",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        ws_server_socket_port
                    );
                }
                if ws_server_socket_port == u16::MAX {
                    eprintln!(
                        "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Exceeded maximum retry times. Now quitting... ",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    );
                    panic!();
                }
                ws_server_socket_port += 1;
            };
            drop(guard_ws_server_status_socket_port);
            drop(guard_ws_server_status);
            WS_SERVER_SOCKET.set(new_async_modifiable(ws_server_socket)).unwrap();
            println!(
                "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initializing the Websocket server...",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            );
            let ws_server_app: axum::Router = axum::Router
                ::new()
                .route(
                    "/",
                    axum::routing::get(|| async { "Rust Backend Test" })
                )
                .merge(
                    axum::Router
                        ::new()
                        .route("/ws", axum::routing::get(ws_handler))
                        .layer(
                            tower::ServiceBuilder
                                ::new()
                                .layer(axum_client_ip::ClientIpSource::ConnectInfo.into_extension())
                                .layer(axum::middleware::from_fn(ip_handler))
                        )
                );
            let listener: Result<tokio::net::TcpListener, std::io::Error> = retry!(
                tokio::net::TcpListener::bind("0.0.0.0:9983").await,
                2,
                1000,
                {
                    eprintln!(
                        "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to initialize the Websocket server. Retrying...",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    );
                }
            );
            let listener: tokio::net::TcpListener = match listener {
                Ok(listener) => {
                    println!(
                        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the Websocket server.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    );

                    let mut guard_ws_server_status: tokio::sync::MutexGuard<
                        '_,
                        ModuleStatus
                    > = ws_server_status.lock().await;
                    guard_ws_server_status.initialized = true;
                    drop(guard_ws_server_status);
                    listener
                }
                Err(_) => {
                    eprintln!(
                        "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Exceeded maximum retry times. Now quitting...",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    );

                    on_unload();
                    let mut guard_ws_server_status: tokio::sync::MutexGuard<
                        '_,
                        ModuleStatus
                    > = ws_server_status.lock().await;
                    guard_ws_server_status.panicked = true;
                    drop(guard_ws_server_status); // Avoid poisoning the mutex lock.
                    panic!();
                }
            };
            axum::serve(
                listener,
                ws_server_app.into_make_service_with_connect_info::<std::net::SocketAddr>()
            ).await.unwrap();
        });
    }

    let ws_server_applications_config_json: WebsocketServerApplicationsConfigJson =
        parse_ws_server_applications_config_json();
    {
        let ws_server_status: AsyncModifiable<ModuleStatus> = ws_server_status.clone();
        ws_server_runtime.spawn(async move {
            loop {
                // Waiting for the initialization to be completed.
                let guard_ws_server_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
                    ws_server_status.lock().await;
                if guard_ws_server_status.initialized {
                    drop(guard_ws_server_status);
                    break;
                }
                drop(guard_ws_server_status);
                fake_yield_now().await;
            }
            // Now load the applications.
            let mut guard_ws_server_applications_libraries: tokio::sync::MutexGuard<
                '_,
                Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>,
            > = WS_SERVER_APPLICATIONS_LIBRARIES.lock().await;
            if load_ws_server_applications(
                rt,
                &mut guard_ws_server_applications_libraries,
                global_module_statuses_by_protocol,
                &ws_server_applications_config_json,
            )
            .is_err()
            {
                drop(guard_ws_server_applications_libraries);
                panic!();
            }

            drop(guard_ws_server_applications_libraries);
            fake_yield_now().await;
        });
    }

    {
        let ws_server_status: AsyncModifiable<ModuleStatus> = ws_server_status.clone();
        ws_server_runtime.spawn(async move {
            loop {
                // Waiting for the initialization to be completed.
                let guard_ws_server_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
                    ws_server_status.lock().await;
                if guard_ws_server_status.initialized {
                    drop(guard_ws_server_status);
                    break;
                }
                drop(guard_ws_server_status);
                fake_yield_now().await;
            }
            // Now processing socket message
            socket_message_processing().await;
        });
    }

    {
        let ws_server_status: AsyncModifiable<ModuleStatus> = ws_server_status.clone();
        ws_server_runtime.spawn(self_management(ws_server_status));
    }
    (ws_server_runtime, ws_server_status)
}

async fn get_socket_port_by_protocol(protocol: &String) -> u16 {
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

async fn get_socket_by_protocol(protocol: &String) -> tokio::net::TcpStream {
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
struct SocketJsonMessageContentOnSendMsg {
    ws_id: String,
    json_msg: serde_json::Value,
}

async fn socket_message_processing() {
    let guard_ws_server_socket: tokio::sync::MutexGuard<'_, tokio::net::TcpListener> =
        WS_SERVER_SOCKET.get().unwrap().lock().await;
    loop {
        let (mut client, _) = guard_ws_server_socket.accept().await.unwrap();
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
                if msg.r#type == "on_send_msg"
                    && let Ok(content) =
                        serde_json::from_value::<SocketJsonMessageContentOnSendMsg>(msg.content)
                {
                    let guard_ws_server_connections_by_ws_id: tokio::sync::MutexGuard<
                        '_,
                        std::collections::HashMap<
                            String,
                            std::sync::Arc<tokio::sync::Mutex<axum::extract::ws::WebSocket>>,
                        >,
                    > = WS_SERVER_CONNECTIONS_BY_WS_ID.lock().await;
                    let ws: &std::sync::Arc<tokio::sync::Mutex<axum::extract::ws::WebSocket>> =
                        guard_ws_server_connections_by_ws_id
                            .get(content.ws_id.as_str())
                            .unwrap();
                    let mut guard_ws: tokio::sync::MutexGuard<'_, axum::extract::ws::WebSocket> =
                        ws.lock().await;
                    if guard_ws
                        .send(axum::extract::ws::Message::from(
                            serde_json::to_string(&content.json_msg).unwrap(),
                        ))
                        .await
                        .is_err()
                    {
                        eprintln!(
                            "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to send JSON Message.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        );
                    }
                    drop(guard_ws);
                    drop(guard_ws_server_connections_by_ws_id);
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

async fn self_management(ws_server_status: AsyncModifiable<ModuleStatus>) {
    let mut monitor_time_cnt: usize = 0;
    loop {
        if let Ok(guard_ws_server_status) = ws_server_status.try_lock() {
            if guard_ws_server_status.panicked {
                drop(guard_ws_server_status); // Avoid poisoning the mutex lock.
                panic!();
            } else if guard_ws_server_status.initialized && WS_SERVER_SOCKET.get().is_none() {
                drop(guard_ws_server_status); // Avoid poisoning the mutex lock.
                panic!();
            }
            drop(guard_ws_server_status);
            monitor_time_cnt += 1;
            if monitor_time_cnt == 600 {
                // Show monitoring message per minute.
                let guard_ws_server_connections_cnt: tokio::sync::MutexGuard<'_, usize> =
                    WS_SERVER_CONNECTIONS_CNT.lock().await;
                println!(
                    "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Status reporting: Working very well with {} connections in total.",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!(),
                    *guard_ws_server_connections_cnt
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
        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloading the Websocket server...",
        std::thread::current().id().as_u64(),
        file!(),
        line!()
    );
    println!(
        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloaded the Websocket server.",
        std::thread::current().id().as_u64(),
        file!(),
        line!()
    );
}

async fn ip_handler(
    axum_client_ip::ClientIp(ip_addr): axum_client_ip::ClientIp,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    println!(
        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Connection from `{}` established.",
        std::thread::current().id().as_u64(),
        file!(),
        line!(),
        ip_addr
    );
    next.run(request).await
}

async fn ws_handler(ws_upgrade: axum::extract::ws::WebSocketUpgrade) -> axum::response::Response {
    ws_upgrade.on_upgrade(ws_callback)
}

async fn ws_callback(mut ws: axum::extract::ws::WebSocket) {
    println!(
        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Websocket connection established.",
        std::thread::current().id().as_u64(),
        file!(),
        line!()
    );

    let mut guard_ws_server_connections_cnt: tokio::sync::MutexGuard<'_, usize> =
        WS_SERVER_CONNECTIONS_CNT.lock().await;
    *guard_ws_server_connections_cnt += 1;
    drop(guard_ws_server_connections_cnt);

    let ws_id: uuid::Uuid = uuid::Uuid::new_v4();
    while let Some(original_msg) = ws.recv().await {
        if let Ok(original_msg) = original_msg {
            match original_msg {
                axum::extract::ws::Message::Text(text) => {
                    if let Ok(mut json_msg) =
                        serde_json::from_str::<WebsocketServerJsonMessage>(text.as_str())
                    {
                        let taken_json_msg_content: AsyncModifiable<serde_json::Value> =
                            std::sync::Arc::new(tokio::sync::Mutex::new(json_msg.content.take()));
                        let guard_ws_server_applications_libraries: tokio::sync::MutexGuard<
                            '_,
                            Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>,
                        > = WS_SERVER_APPLICATIONS_LIBRARIES.lock().await;
                        for library in guard_ws_server_applications_libraries.iter() {
                            if let Ok(callback_function) = unsafe {
                                library.0.symbol::<unsafe extern "Rust" fn(
                                    &tokio::runtime::Runtime,
                                    (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
                                    AsyncModifiable<serde_json::Value>,
                                )
                                    -> ()>(
                                    &(String::from("on_") + &json_msg.r#type)
                                )
                            } {
                                unsafe {
                                    callback_function(
                                        &library.1,
                                        (&mut ws, &ws_id),
                                        taken_json_msg_content.clone(),
                                    );
                                }
                            };
                        }
                    };
                }
                axum::extract::ws::Message::Close(_) => {
                    println!(
                        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Websocket connection closed.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    );
                    let mut guard_ws_server_connections_cnt: tokio::sync::MutexGuard<'_, usize> =
                        WS_SERVER_CONNECTIONS_CNT.lock().await;
                    *guard_ws_server_connections_cnt -= 1;
                    drop(guard_ws_server_connections_cnt);
                    let guard_ws_server_applications_libraries: tokio::sync::MutexGuard<
                        '_,
                        Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>,
                    > = WS_SERVER_APPLICATIONS_LIBRARIES.lock().await;
                    for library in guard_ws_server_applications_libraries.iter() {
                        if let Ok(callback_function) = unsafe {
                            library.0.symbol::<unsafe extern "Rust" fn(
                                &tokio::runtime::Runtime,
                                (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
                            ) -> ()>(&String::from(
                                "on_close_connection",
                            ))
                        } {
                            unsafe {
                                callback_function(&library.1, (&mut ws, &ws_id));
                            }
                        };
                    }
                }
                _ => {}
            }
        }
    }
}

fn parse_ws_server_applications_config_json() -> WebsocketServerApplicationsConfigJson {
    let mut ws_server_applications_config_json_string: String = String::new();
    let ws_server_applications_config_json_file_path: String =
        get_parent_path() + "/rust_backend/ws_server/ws_server_config_rs.json";
    let mut ws_server_applications_config_json_file: std::fs::File = match std::fs::File::open(
        &ws_server_applications_config_json_file_path,
    ) {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to open ws_server_config_rs.json from `{}`. Maybe the file doesn't exist?",
                std::thread::current().id().as_u64(),
                file!(),
                line!(),
                ws_server_applications_config_json_file_path
            );
            eprintln!(
                "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] {}",
                file!(),
                line!(),
                std::thread::current().id().as_u64(),
                e
            );
            panic!();
        }
    };
    ws_server_applications_config_json_file
        .read_to_string(&mut ws_server_applications_config_json_string)
        .unwrap(); // Read module config json file to string.

    serde_json::from_str(ws_server_applications_config_json_string.as_str()).unwrap()
}

fn get_parent_path() -> String {
    // Get the parent path
    let mut pwd: std::path::PathBuf = std::env::current_dir().unwrap();
    pwd.pop();
    String::from(pwd.to_str().unwrap())
}

fn load_ws_server_applications(
    rt: &tokio::runtime::Runtime,
    ws_server_applications_libraries: &mut Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>,
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
    >,
    ws_server_applications_config_json: &WebsocketServerApplicationsConfigJson,
) -> Result<(), dlopen2::Error> {
    for (name, config) in &ws_server_applications_config_json.ws_server_applications {
        if config.enabled {
            println!(
                "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Loading websocket server application {}...",
                std::thread::current().id().as_u64(),
                file!(),
                line!(),
                name
            );
            let ws_server_application_library_file_path: String = get_parent_path()
                + "/rust_backend/modules/ws_server/assets/lib/lib"
                + &config.id
                + ".so"; // Get the path of the module.
            let ws_server_application_library: Result<dlopen2::symbor::Library, dlopen2::Error> =
                dlopen2::symbor::Library::open(&ws_server_application_library_file_path);

            match ws_server_application_library {
                Ok(ws_server_application_library) => {
                    println!(
                        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Successfully loaded websocket server application `{}` from `{}`.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        name,
                        &ws_server_application_library_file_path
                    );

                    if let Ok(callback) = unsafe {
                        ws_server_application_library
                            .symbol::<unsafe extern "Rust" fn(
                            &tokio::runtime::Runtime,
                            AsyncModifiable<
                                std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
                            >,
                        ) -> tokio::runtime::Runtime>("on_init")
                    } {
                        let ws_server_application_library_runtime: tokio::runtime::Runtime =
                            unsafe { callback(rt, global_module_statuses_by_protocol.clone()) };
                        ws_server_applications_libraries.push((
                            ws_server_application_library,
                            ws_server_application_library_runtime,
                        ));
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to load websocket server application `{}` from `{}`. Maybe the websocket server application file doesn't exist or is not a valid shared object?",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        name,
                        &ws_server_application_library_file_path
                    );

                    if ws_server_applications_config_json.restricted_mode {
                        eprintln!(
                            "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Due to the restricted mode, the websocket server now is shutting down.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        );
                        return Err(e);
                    }
                }
            }
        }
    }

    Ok(())
}

const FAKE_YIELD_NOW_MILLISECONDS: u64 = 100;

async fn fake_yield_now() {
    tokio::time::sleep(tokio::time::Duration::from_millis(
        FAKE_YIELD_NOW_MILLISECONDS,
    ))
    .await;
}
