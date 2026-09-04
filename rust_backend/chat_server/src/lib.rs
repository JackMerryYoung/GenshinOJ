#![feature(thread_id_value)]

mod global;
mod socket_actions;
mod self_management;
mod chat_server_socket;

use crate::global::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>
    >
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let chat_server_runtime: tokio::runtime::Runtime = tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    MODULE_RUNTIME_HANDLE.set(chat_server_runtime.handle().clone()).unwrap();
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL.set(global_module_statuses_by_protocol.clone()).unwrap();
    let chat_server_status: ModuleStatus = ModuleStatus {
        initialized: false,
        panicked: false,
        socket_port: new_async_modifiable(0),
        init_notify: std::sync::Arc::new(tokio::sync::Notify::new()),
    };
    let chat_server_status: AsyncModifiable<ModuleStatus> = new_async_modifiable(chat_server_status);

    {
        let chat_server_status: AsyncModifiable<ModuleStatus> = chat_server_status.clone();
        chat_server_runtime.spawn(async move {
            use mysql_async::prelude::*;
            let mut conn: mysql_async::Conn = MYSQL_DATABASE_POOL.get_conn().await.unwrap();
            "CREATE DATABASE IF NOT EXISTS RsOJ".ignore(&mut conn).await.unwrap();
            "USE RsOJ".ignore(&mut conn).await.unwrap();
            "CREATE TABLE IF NOT EXISTS chat_messages (
                id INT AUTO_INCREMENT PRIMARY KEY NOT NULL,
                from_username VARCHAR(256) NOT NULL,
                to_username VARCHAR(256) NOT NULL,
                messages TEXT NOT NULL,
                created_at BIGINT NOT NULL
            )"
                .ignore(&mut conn).await
                .unwrap();
            // Conversation pagination filters both participants and then orders by id.
            let _ = "ALTER TABLE chat_messages ADD INDEX idx_chat_from_to_id (from_username, to_username, id)"
                .ignore(&mut conn).await;
            let _ = "ALTER TABLE chat_messages ADD INDEX idx_chat_to_from_id (to_username, from_username, id)"
                .ignore(&mut conn).await;
            drop(conn);

            let mut guard_chat_server_status: tokio::sync::MutexGuard<
                '_,
                ModuleStatus
            > = chat_server_status.lock().await; // Get the status of the server.
            // Try to establish a socket for messaging.
            let mut chat_server_socket_port: u16 = 9002;
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
                    format!("127.0.0.1:{}", chat_server_socket_port)
                ).await;
                if let Ok(x) = chat_server_socket_result {
                    println!(
                        "{}",
                        ansi_term::Color::Green.paint(
                            format!(
                                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the socket on port {}.",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                chat_server_socket_port
                            )
                        )
                    );
                    break (x, chat_server_socket_port);
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
                                chat_server_socket_port
                            )
                        )
                    );
                }
                if chat_server_socket_port == u16::MAX {
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
                chat_server_socket_port += 1;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            };
            CHAT_SERVER_SOCKET.set(new_async_modifiable(chat_server_socket)).unwrap();
            drop(guard_chat_server_status_socket_port);
            guard_chat_server_status.initialized = true;
            // notify_waiters(), not notify_one(): there are two independent waiters on this same
            // init_notify — main_backend's module-loading wait, and this module's own internal
            // wait below before it starts processing its socket. notify_one() only wakes one of
            // them, permanently starving the other.
            guard_chat_server_status.init_notify.notify_waiters();
            drop(guard_chat_server_status);
        });
    }

    {
        let chat_server_status: AsyncModifiable<ModuleStatus> = chat_server_status.clone();
        chat_server_runtime.spawn(async move {
            // Wait for initialization, notified instead of polled. The setter uses
            // notify_waiters() (not notify_one()) because main_backend's own module-loading wait
            // is a second, independent waiter on this same init_notify; notify_waiters() only
            // reaches waiters already registered at the moment it's called, so `enable()` must
            // run here before the flag check to register us immediately.
            let init_notify: std::sync::Arc<tokio::sync::Notify> = chat_server_status
                .lock().await.init_notify
                .clone();
            let notified = init_notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let already_initialized: bool = chat_server_status.lock().await.initialized;
            if !already_initialized {
                notified.await;
            }

            // Now processing socket message
            chat_server_socket::socket_message_processing().await;
        });
    }

    {
        chat_server_runtime.spawn(async move {
            // ws_server doesn't necessarily exist in the map yet (it may not have been loaded by
            // main_backend at all), so there's no event to wait on for that — poll until it
            // registers itself.
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
            // Now that ws_server is registered, wait for it to finish initializing — notified
            // instead of polled, same as init_notify everywhere else. In practice ws_server only
            // appears in the map after main_backend's own wait for it already completed, so
            // `already_initialized` will already be true here; `enable()` is kept for
            // consistency with the other waiters in case that ordering ever changes.
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

            chat_server_socket::connect_to_ws_server().await;
        });
    }

    {
        let chat_server_status: AsyncModifiable<ModuleStatus> = chat_server_status.clone();
        chat_server_runtime.spawn(self_management::self_management(chat_server_status));
    }
    (chat_server_runtime, chat_server_status)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload(unload_timeout_ms: usize) {
    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Unloading the chat server...",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );

    let cleanup_completed = run_shutdown_task(async move {
        // Resolve whether ws_server is up and release the lock before calling
        // disconnect_from_ws_server() below, since it locks GLOBAL_MODULE_STATUSES_BY_PROTOCOL
        // itself again internally (via get_socket_port_by_protocol) — holding it here too would
        // self-deadlock the task on tokio::sync::Mutex, which isn't reentrant.
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
            chat_server_socket::disconnect_from_ws_server().await;
        }
        disconnect_database_pool().await;
    }, std::time::Duration::from_millis(unload_timeout_ms as u64));

    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Chat server shutdown cleanup {}.",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!(),
                if cleanup_completed { "completed" } else { "timed out" }
            )
        )
    );
}
