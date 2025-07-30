#![feature(thread_id_value)]

use std::io::Read;

macro_rules! retry {
    ($f:expr, $count:expr, $interval:expr, $retry_func: expr) => {{
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
    }};
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

pub struct ModuleStatus {
    initialized: bool,
    panicked: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct WebsocketServerJsonMessage {
    r#type: String,
    content: serde_json::Value,
}

static WS_SERVER_APPLICATIONS_LIBRARIES: std::sync::LazyLock<
    tokio::sync::Mutex<Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>>,
> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(vec![]));

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    rt: &'static tokio::runtime::Runtime,
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let ws_server_runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let ws_server_status: ModuleStatus = ModuleStatus {
        initialized: false,
        panicked: false,
    };
    let ws_server_status: AsyncModifiable<ModuleStatus> =
        std::sync::Arc::new(tokio::sync::Mutex::new(ws_server_status));
    {
        let ws_server_status: AsyncModifiable<ModuleStatus> = ws_server_status.clone();
        ws_server_runtime.spawn(async move {
            println!(
                "[WS_SERVER] [INFO] [THREAD {}] Initializing the Websocket server...",
                std::thread::current().id().as_u64()
            );
            let ws_server_app: axum::Router = axum::Router::new()
                .route("/", axum::routing::get(|| async { "Rust Backend Test" }))
                .merge(axum::Router::new().route("/ws", axum::routing::get(ws_handler)).layer(tower::ServiceBuilder::new().layer(axum_client_ip::ClientIpSource::ConnectInfo.into_extension()).layer(axum::middleware::from_fn(ip_handler))));
            let listener: Result<tokio::net::TcpListener, std::io::Error> = retry!(
                tokio::net::TcpListener::bind("0.0.0.0:9983").await,
                2,
                1000,
                {
                    eprintln!(
                        "[WS_SERVER] [ERROR] [THREAD {}] Failed to initialize the Websocket server. Retrying...",
                        std::thread::current().id().as_u64()
                    );
                }
            );
            let listener: tokio::net::TcpListener = match listener {
                Ok(listener) => {
                    println!(
                        "[WS_SERVER] [INFO] [THREAD {}] Initialized the Websocket server.",
                        std::thread::current().id().as_u64()
                    );

                    let mut guard_ws_server_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
                    ws_server_status.lock().await;
                    guard_ws_server_status.initialized = true;
                    drop(guard_ws_server_status);
                    listener
                }
                Err(_) => {
                    eprintln!(
                        "[WS_SERVER] [ERROR] [THREAD {}] Exceeded maximum retry times. Now quitting...",
                        std::thread::current().id().as_u64()
                    );

                    on_unload();
                    let mut guard_ws_server_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
                    ws_server_status.lock().await;
                    guard_ws_server_status.panicked = true;
                    drop(guard_ws_server_status); // Avoiding poisoned mutex lock
                    panic!();
                }
            };
            axum::serve(listener, ws_server_app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap();
        });
    };

    let ws_server_applications_config_json: WebsocketServerApplicationsConfigJson =
        parse_ws_server_applications_config_json();
    {
        let ws_server_status: AsyncModifiable<ModuleStatus> = ws_server_status.clone();
        ws_server_runtime.spawn(async move {
            loop {
                let guard_ws_server_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
                    ws_server_status.lock().await;
                if guard_ws_server_status.initialized {
                    drop(guard_ws_server_status);
                    break;
                }
                drop(guard_ws_server_status);
                fake_yield_now().await;
            }
            let mut guard_ws_server_applications_libraries: tokio::sync::MutexGuard<
                '_,
                Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>,
            > = WS_SERVER_APPLICATIONS_LIBRARIES.lock().await;
            if load_ws_server_applications(
                rt,
                &mut guard_ws_server_applications_libraries,
                &ws_server_applications_config_json,
            )
            .is_err()
            {
                drop(guard_ws_server_applications_libraries);
                panic!();
            };

            drop(guard_ws_server_applications_libraries);
            fake_yield_now().await;
        });
    }

    {
        let ws_server_status: AsyncModifiable<ModuleStatus> = ws_server_status.clone();
        ws_server_runtime.spawn(async move {
            loop {
                let guard_ws_server_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
                    ws_server_status.lock().await;
                if guard_ws_server_status.panicked {
                    drop(guard_ws_server_status);
                    break;
                }
                drop(guard_ws_server_status);
                fake_yield_now().await;
            }
        });
    }
    (ws_server_runtime, ws_server_status)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    println!(
        "[WS_SERVER] [INFO] [THREAD {}] Unloading the Websocket server...",
        std::thread::current().id().as_u64()
    );
    println!(
        "[WS_SERVER] [INFO] [THREAD {}] Unloaded the Websocket server.",
        std::thread::current().id().as_u64()
    );
}

async fn ip_handler(
    axum_client_ip::ClientIp(ip_addr): axum_client_ip::ClientIp,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    println!(
        "[WS_SERVER] [INFO] [THREAD {}] Connection from `{}` established.",
        std::thread::current().id().as_u64(),
        ip_addr
    );
    next.run(request).await
}

async fn ws_handler(ws_upgrade: axum::extract::ws::WebSocketUpgrade) -> axum::response::Response {
    ws_upgrade.on_upgrade(ws_callback)
}

async fn ws_callback(mut ws: axum::extract::ws::WebSocket) {
    println!(
        "[WS_SERVER] [INFO] [THREAD {}] Websocket connection established.",
        std::thread::current().id().as_u64()
    );

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
                                    )
                                };
                            };
                        }
                    };
                }
                axum::extract::ws::Message::Close(_) => {
                    println!(
                        "[WS_SERVER] [INFO] [THREAD {}] Websocket connection closed.",
                        std::thread::current().id().as_u64(),
                    );
                    let guard_ws_server_applications_libraries: tokio::sync::MutexGuard<
                        '_,
                        Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>,
                    > = WS_SERVER_APPLICATIONS_LIBRARIES.lock().await;
                    for library in guard_ws_server_applications_libraries.iter() {
                        if let Ok(callback_function) = unsafe {
                            library.0.symbol::<unsafe extern "Rust" fn(
                                &tokio::runtime::Runtime,
                                (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
                            ) -> ()>(
                                &(String::from("on_close_connection"))
                            )
                        } {
                            unsafe { callback_function(&library.1, (&mut ws, &ws_id)) };
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
                "[WS_SERVER] [ERROR] [THREAD {}] Failed to open ws_server_config_rs.json from `{}`. Maybe the file doesn't exist?",
                std::thread::current().id().as_u64(),
                ws_server_applications_config_json_file_path
            );
            eprintln!(
                "[WS_SERVER] [ERROR] [THREAD {}] {}",
                e,
                std::thread::current().id().as_u64()
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
    ws_server_applications_config_json: &WebsocketServerApplicationsConfigJson,
) -> Result<(), dlopen2::Error> {
    for (name, config) in &ws_server_applications_config_json.ws_server_applications {
        if config.enabled {
            println!(
                "[WS_SERVER] [INFO] [THREAD {}] Loading websocket server application {}...",
                std::thread::current().id().as_u64(),
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
                        "[WS_SERVER] [INFO] [THREAD {}] Successfully loaded websocket server application `{}` from `{}`.",
                        std::thread::current().id().as_u64(),
                        name,
                        &ws_server_application_library_file_path
                    );

                    if let Ok(callback) = unsafe {
                        ws_server_application_library
                            .symbol::<unsafe extern "Rust" fn(
                            &tokio::runtime::Runtime,
                        ) -> tokio::runtime::Runtime>("on_init")
                    } {
                        let ws_server_application_library_runtime: tokio::runtime::Runtime =
                            unsafe { callback(rt) };
                        ws_server_applications_libraries.push((
                            ws_server_application_library,
                            ws_server_application_library_runtime,
                        ));
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[WS_SERVER] [ERROR] [THREAD {}] Failed to load websocket server application `{}` from `{}`. Maybe the websocket server application file doesn't exist or is not a valid shared object?",
                        std::thread::current().id().as_u64(),
                        name,
                        &ws_server_application_library_file_path
                    );

                    if ws_server_applications_config_json.restricted_mode {
                        eprintln!(
                            "[WS_SERVER] [ERROR] [THREAD {}] Due to the restricted mode, the websocket server now is shutting down.",
                            std::thread::current().id().as_u64()
                        );
                        return Err(e);
                    }
                }
            }
        }
    }

    Ok(())
}

const FAKE_YIELD_NOW_MILLISECONDS: u64 = 1;

async fn fake_yield_now() {
    tokio::time::sleep(tokio::time::Duration::from_millis(
        FAKE_YIELD_NOW_MILLISECONDS,
    ))
    .await;
}
