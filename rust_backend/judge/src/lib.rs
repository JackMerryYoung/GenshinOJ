#![feature(thread_id_value)]

mod global;
mod socket_actions;
mod self_management;
mod judge_socket;
mod judging;

use crate::global::*;

use mysql_async::prelude::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>
    >
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let judge_runtime: tokio::runtime::Runtime = tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL.set(global_module_statuses_by_protocol.clone()).unwrap();
    let judge_status: ModuleStatus = ModuleStatus {
        initialized: false,
        panicked: false,
        socket_port: new_async_modifiable(0),
        init_notify: std::sync::Arc::new(tokio::sync::Notify::new()),
    };
    let judge_status: AsyncModifiable<ModuleStatus> = new_async_modifiable(judge_status);

    {
        let judge_status: AsyncModifiable<ModuleStatus> = judge_status.clone();
        judge_runtime.spawn(async move {
            let guard_mysql_database_pool: tokio::sync::MutexGuard<
                '_,
                mysql_async::Pool
            > = MYSQL_DATABASE_POOL.lock().await;
            let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
            "CREATE DATABASE IF NOT EXISTS GenshinOJ".ignore(&mut conn).await.unwrap();
            "USE GenshinOJ".ignore(&mut conn).await.unwrap();
            "CREATE TABLE IF NOT EXISTS submissions (
                submission_id INT AUTO_INCREMENT PRIMARY KEY NOT NULL,
                username VARCHAR(256) NOT NULL,
                problem_number INT NOT NULL,
                result VARCHAR(16) NOT NULL DEFAULT 'PD',
                general_score INT NOT NULL DEFAULT 0,
                statuses TEXT NOT NULL,
                scores TEXT NOT NULL,
                code TEXT NOT NULL,
                language VARCHAR(16) NOT NULL DEFAULT '',
                is_test_submission_mode BOOLEAN NOT NULL DEFAULT FALSE,
                created_at BIGINT NOT NULL
            )"
                .ignore(&mut conn).await
                .unwrap();
            drop(conn);
            drop(guard_mysql_database_pool);

            let mut guard_judge_status: tokio::sync::MutexGuard<
                '_,
                ModuleStatus
            > = judge_status.lock().await; // Get the status of the server.
            // Try to establish a socket for messaging.
            let mut judge_socket_port: u16 = 9003;
            let mut guard_judge_status_socket_port: tokio::sync::MutexGuard<
                '_,
                u16
            > = guard_judge_status.socket_port.lock().await;
            let judge_socket: tokio::net::TcpListener;
            (judge_socket, *guard_judge_status_socket_port) = loop {
                let judge_socket_result: Result<
                    tokio::net::TcpListener,
                    std::io::Error
                > = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", judge_socket_port)).await;
                if let Ok(x) = judge_socket_result {
                    println!(
                        "{}",
                        ansi_term::Color::Green.paint(
                            format!(
                                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the socket on port {}.",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                judge_socket_port
                            )
                        )
                    );
                    break (x, judge_socket_port);
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
                                judge_socket_port
                            )
                        )
                    );
                }
                if judge_socket_port == u16::MAX {
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
                judge_socket_port += 1;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            };
            JUDGE_SOCKET.set(new_async_modifiable(judge_socket)).unwrap();
            drop(guard_judge_status_socket_port);
            guard_judge_status.initialized = true;
            // notify_waiters(), not notify_one(): there are two independent waiters on this same
            // init_notify — main_backend's module-loading wait, and this module's own internal
            // wait below before it starts processing its socket. notify_one() only wakes one of
            // them, permanently starving the other.
            guard_judge_status.init_notify.notify_waiters();
            drop(guard_judge_status);
        });
    }

    {
        let judge_status: AsyncModifiable<ModuleStatus> = judge_status.clone();
        judge_runtime.spawn(async move {
            // Wait for initialization, notified instead of polled. The setter uses
            // notify_waiters() (not notify_one()) because main_backend's own module-loading wait
            // is a second, independent waiter on this same init_notify; notify_waiters() only
            // reaches waiters already registered at the moment it's called, so `enable()` must
            // run here before the flag check to register us immediately.
            let init_notify: std::sync::Arc<tokio::sync::Notify> = judge_status
                .lock().await.init_notify
                .clone();
            let notified = init_notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let already_initialized: bool = judge_status.lock().await.initialized;
            if !already_initialized {
                notified.await;
            }

            // Now processing socket message
            judge_socket::socket_message_processing().await;
        });
    }

    {
        judge_runtime.spawn(async move {
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

            judge_socket::connect_to_ws_server().await;
        });
    }

    {
        let judge_status: AsyncModifiable<ModuleStatus> = judge_status.clone();
        judge_runtime.spawn(self_management::self_management(judge_status));
    }
    (judge_runtime, judge_status)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    let judge_runtime_on_unload: tokio::runtime::Runtime = tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Unloading the judge...",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );

    judge_runtime_on_unload.spawn(async move {
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
            judge_socket::disconnect_from_ws_server().await;
        }
    });

    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Unloaded the judge.",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
}
