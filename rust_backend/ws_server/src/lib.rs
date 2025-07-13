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

// static WS_SERVER_APPLICATIONS_LIBRARIES: std::sync::atomic::

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

struct WebsocketServerStatus {
    initialized: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct WebsocketServerJsonMessage {
    r#type: String,
    content: serde_json::Value,
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(_rt: &'static tokio::runtime::Runtime) -> tokio::runtime::Runtime {
    let ws_server_runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let ws_server_status: WebsocketServerStatus = WebsocketServerStatus { initialized: false };
    let ws_server_status: std::sync::Arc<tokio::sync::Mutex<WebsocketServerStatus>> =
        std::sync::Arc::new(tokio::sync::Mutex::new(ws_server_status));
    {
        let ws_server_status: std::sync::Arc<tokio::sync::Mutex<WebsocketServerStatus>> =
            ws_server_status.clone();
        ws_server_runtime.spawn(async move {
            println!(
                "[WS_SERVER] [INFO] [THREAD {}] Initializing the Websocket server...",
                std::thread::current().id().as_u64()
            );
            let ws_server_app: axum::Router = axum::Router::new()
                .route("/", axum::routing::get(|| async { "Rust Backend Test" }))
                .route("/ws", axum::routing::get(ws_handler));
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
                    let mut guard: tokio::sync::MutexGuard<'_, WebsocketServerStatus> =
                    ws_server_status.lock().await;
                    guard.initialized = true;
                    listener
                }
                Err(_) => {
                    eprintln!(
                        "[WS_SERVER] [ERROR] [THREAD {}] Exceeded maximum retry times. Now quitting...",
                        std::thread::current().id().as_u64()
                    );
                    on_unload();
                    panic!()
                }
            };
            axum::serve(listener, ws_server_app).await.unwrap();
        });
    }

    let ws_server_applications_config_json: WebsocketServerApplicationsConfigJson =
        parse_ws_server_applications_config_json();
    {
        let ws_server_status: std::sync::Arc<tokio::sync::Mutex<WebsocketServerStatus>> =
            ws_server_status.clone();
        ws_server_runtime.spawn(async move {
            loop {
                let guard: tokio::sync::MutexGuard<'_, WebsocketServerStatus> =
                    ws_server_status.lock().await;
                if guard.initialized {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
            // load_ws_server_applications(
            //     &mut WS_SERVER_APPLICATIONS_LIBRARIES,
            //     &ws_server_applications_config_json,
            // );
        });
    }

    ws_server_runtime
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

async fn ws_handler(ws_upgrade: axum::extract::ws::WebSocketUpgrade) -> axum::response::Response {
    ws_upgrade.on_upgrade(ws_callback)
}

async fn ws_callback(mut ws: axum::extract::ws::WebSocket) {
    while let Some(original_msg) = ws.recv().await {
        let original_msg: axum::extract::ws::Message = original_msg.unwrap();
        let json_msg: WebsocketServerJsonMessage =
            serde_json::from_str(&original_msg.into_text().unwrap()).unwrap();
        // for library in
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
                "[WS_SERVER] [THREAD {}] [ERROR] {}",
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
    ws_server_applications_libraries: &mut Vec<dlopen2::symbor::Library>,
    ws_server_applications_config_json: &WebsocketServerApplicationsConfigJson,
) {
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

                    ws_server_applications_libraries.push(ws_server_application_library);
                }
                Err(_) => {
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
                        panic!();
                    }
                }
            }
        }
    }
}
