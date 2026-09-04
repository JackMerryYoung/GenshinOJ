#![feature(thread_id_value)]

mod global;
mod socket_actions;
mod self_management;
mod socket_message_processing;
mod userish_socket;

use crate::global::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>
    >
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let userish_runtime: tokio::runtime::Runtime = tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    MODULE_RUNTIME_HANDLE.set(userish_runtime.handle().clone()).unwrap();
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL.set(global_module_statuses_by_protocol.clone()).unwrap();
    let userish_status: ModuleStatus = ModuleStatus {
        initialized: false,
        panicked: false,
        socket_port: new_async_modifiable(0),
        init_notify: std::sync::Arc::new(tokio::sync::Notify::new()),
    };
    let userish_status: AsyncModifiable<ModuleStatus> = new_async_modifiable(userish_status);
    {
        let userish_status: AsyncModifiable<ModuleStatus> = userish_status.clone();
        userish_runtime.spawn(async move {
            // `users`/`follows` are created by simple_authenticator (which this module depends
            // on, per module_config_rs.json's load ordering) — no schema setup needed here.
            let mut guard_userish_status: tokio::sync::MutexGuard<
                '_,
                ModuleStatus
            > = userish_status.lock().await;
            let mut userish_socket_port: u16 = 9100;
            let mut guard_userish_status_socket_port: tokio::sync::MutexGuard<
                '_,
                u16
            > = guard_userish_status.socket_port.lock().await;
            let userish_socket: tokio::net::TcpListener;
            (userish_socket, *guard_userish_status_socket_port) = loop {
                let userish_socket_result: Result<
                    tokio::net::TcpListener,
                    std::io::Error
                > = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", userish_socket_port)).await;
                if let Ok(x) = userish_socket_result {
                    println!(
                        "{}",
                        ansi_term::Color::Green.paint(
                            format!(
                                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the socket on port {}.",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                userish_socket_port
                            )
                        )
                    );
                    break (x, userish_socket_port);
                } else {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to open the socket on port {}. Retrying...",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                userish_socket_port
                            )
                        )
                    );
                }
                if userish_socket_port == u16::MAX {
                    eprintln!(
                        "{}",
                        ansi_term::Color::Red.paint(
                            format!(
                                "[{}] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Exceeded maximum retry times. Now quitting... ",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            )
                        )
                    );
                    panic!();
                }
                userish_socket_port += 1;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            };
            USERISH_SOCKET.set(new_async_modifiable(userish_socket)).unwrap();
            drop(guard_userish_status_socket_port);
            guard_userish_status.initialized = true;
            // notify_waiters(), not notify_one(): main_backend's own module-loading wait and this
            // module's internal wait below are two independent waiters on this same init_notify.
            guard_userish_status.init_notify.notify_waiters();
            drop(guard_userish_status);
        });
    }

    {
        let userish_status: AsyncModifiable<ModuleStatus> = userish_status.clone();
        userish_runtime.spawn(async move {
            let init_notify: std::sync::Arc<tokio::sync::Notify> = userish_status
                .lock().await.init_notify
                .clone();
            let notified = init_notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let already_initialized: bool = userish_status.lock().await.initialized;
            if !already_initialized {
                notified.await;
            }

            crate::socket_message_processing::socket_message_processing().await;
        });
    }

    {
        userish_runtime.spawn(async move {
            // ws_server may not have been loaded yet — poll until it registers itself.
            let ws_server_status: AsyncModifiable<ModuleStatus> = loop {
                let guard_global_module_statuses_by_protocol =
                    global_module_statuses_by_protocol.lock().await;
                if
                    let Some(ws_server_status) =
                        guard_global_module_statuses_by_protocol.get("std_ws_server")
                {
                    let ws_server_status: AsyncModifiable<ModuleStatus> = ws_server_status.clone();
                    drop(guard_global_module_statuses_by_protocol);
                    break ws_server_status;
                }
                drop(guard_global_module_statuses_by_protocol);
                println!(
                    "{}",
                    ansi_term::Color::Blue.paint(
                        format!(
                            "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Waiting for the Websocket server to be initialized...",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            };
            let init_notify: std::sync::Arc<tokio::sync::Notify> = ws_server_status
                .lock().await.init_notify
                .clone();
            let notified = init_notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let already_initialized: bool = ws_server_status.lock().await.initialized;
            if !already_initialized {
                notified.await;
            }
            println!(
                "{}",
                ansi_term::Color::Green.paint(
                    format!(
                        "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] The Websocket server has been initialized.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    )
                )
            );

            userish_socket::connect_to_ws_server().await;
        });
    }

    {
        let userish_status: AsyncModifiable<ModuleStatus> = userish_status.clone();
        userish_runtime.spawn(self_management::self_management(userish_status));
    }
    (userish_runtime, userish_status)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload(unload_timeout_ms: usize) {
    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Unloading userish...",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );

    let cleanup_completed = run_shutdown_task(async move {
        let ws_server_initialized: bool = {
            let guard_global_module_statuses_by_protocol = GLOBAL_MODULE_STATUSES_BY_PROTOCOL.get()
                .unwrap()
                .lock().await;
            match guard_global_module_statuses_by_protocol.get("std_ws_server") {
                Some(ws_server_status) => ws_server_status.lock().await.initialized,
                None => false,
            }
        };
        if ws_server_initialized {
            userish_socket::disconnect_from_ws_server().await;
        }
        disconnect_database_pool().await;
    }, std::time::Duration::from_millis(unload_timeout_ms as u64));

    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Userish shutdown cleanup {}.",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!(),
                if cleanup_completed { "completed" } else { "timed out" }
            )
        )
    );
}
