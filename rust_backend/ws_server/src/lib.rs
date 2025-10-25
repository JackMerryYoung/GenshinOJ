#![feature(thread_id_value)]

mod global;
mod ws_handler;
mod socket_actions;
mod self_management;
mod ws_server_socket;

use global::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>
    >
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let ws_server_runtime: tokio::runtime::Runtime = tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL.set(global_module_statuses_by_protocol.clone()).unwrap();
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
                    format!("127.0.0.1:{}", &ws_server_socket_port)
                ).await;
                if let Ok(x) = ws_server_socket_result {
                    println!(
                        "{}",
                        ansi_term::Color::Blue.paint(
                            format!(
                                "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the socket on port {}.",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                ws_server_socket_port
                            )
                        )
                    );
                    break (x, ws_server_socket_port);
                } else {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to open the socket on port {}. Retrying...",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                ws_server_socket_port
                            )
                        )
                    );
                }
                if ws_server_socket_port == u16::MAX {
                    eprintln!(
                        "{}",
                        ansi_term::Color::Red.paint(
                            format!(
                                "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Exceeded maximum retry times. Now quitting... ",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            )
                        )
                    );
                    panic!();
                }
                ws_server_socket_port += 1;
                fake_yield_now().await;
            };
            drop(guard_ws_server_status_socket_port);
            drop(guard_ws_server_status);
            WS_SERVER_SOCKET.set(new_async_modifiable(ws_server_socket)).unwrap();
            println!(
                "{}",
                ansi_term::Color::Blue.paint(
                    format!(
                        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initializing the Websocket server...",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    )
                )
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
                        .route("/ws", axum::routing::get(crate::ws_handler::ws_handler))
                        .layer(
                            tower::ServiceBuilder
                                ::new()
                                .layer(axum_client_ip::ClientIpSource::ConnectInfo.into_extension())
                                .layer(axum::middleware::from_fn(crate::ws_handler::ip_handler))
                        )
                );
            let listener: Result<tokio::net::TcpListener, std::io::Error> = retry!(
                tokio::net::TcpListener::bind("0.0.0.0:9983").await,
                2,
                1000,
                {
                    eprintln!(
                        "{}",
                        ansi_term::Color::Red.paint(
                            format!(
                                "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to initialize the Websocket server. Retrying...",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            )
                        )
                    );
                }
            );
            let listener: tokio::net::TcpListener = match listener {
                Ok(listener) => {
                    println!(
                        "{}",
                        ansi_term::Color::Blue.paint(
                            format!(
                                "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the Websocket server.",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            )
                        )
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
                        "{}",
                        ansi_term::Color::Red.paint(
                            format!(
                                "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Exceeded maximum retry times. Now quitting...",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            )
                        )
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

    {
        let ws_server_status: AsyncModifiable<ModuleStatus> = ws_server_status.clone();
        ws_server_runtime.spawn(async move {
            loop {
                // Waiting for the initialization to be completed.
                let guard_ws_server_status: tokio::sync::MutexGuard<
                    '_,
                    ModuleStatus
                > = ws_server_status.lock().await;
                if guard_ws_server_status.initialized {
                    drop(guard_ws_server_status);
                    break;
                }
                drop(guard_ws_server_status);
                fake_yield_now().await;
            }
            WS_SERVER_EXTERNAL_LISTENERS_BY_PROTOCOL.set(
                new_async_modifiable(std::collections::HashMap::new())
            ).unwrap();
            // Now processing socket message
            crate::ws_server_socket::socket_message_processing().await;
        });
    }

    {
        let ws_server_status: AsyncModifiable<ModuleStatus> = ws_server_status.clone();
        ws_server_runtime.spawn(crate::self_management::self_management(ws_server_status));
    }
    (ws_server_runtime, ws_server_status)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    println!(
        "{}",
        ansi_term::Color::Blue.paint(
            format!(
                "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloading the Websocket server...",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
    println!(
        "{}",
        ansi_term::Color::Blue.paint(
            format!(
                "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloaded the Websocket server.",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
}
