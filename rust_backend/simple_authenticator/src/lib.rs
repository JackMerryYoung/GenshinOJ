#![feature(thread_id_value)]

mod global;
mod socket_actions;
mod self_management;
mod socket_message_processing;
mod simple_authenticator_socket;

use mysql_async::prelude::{ Query, Queryable };

use crate::global::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>
    >
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let simple_authenticator_runtime: tokio::runtime::Runtime = tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL.set(global_module_statuses_by_protocol.clone()).unwrap();
    let simple_authenticator_status: ModuleStatus = ModuleStatus {
        initialized: false,
        panicked: false,
        socket_port: new_async_modifiable(0),
    };
    let simple_authenticator_status: AsyncModifiable<ModuleStatus> = new_async_modifiable(
        simple_authenticator_status
    );
    {
        let simple_authenticator_status: AsyncModifiable<ModuleStatus> =
            simple_authenticator_status.clone();
        simple_authenticator_runtime.spawn(async move {
            let guard_mysql_database_pool: tokio::sync::MutexGuard<
                '_,
                mysql_async::Pool
            > = MYSQL_DATABASE_POOL.lock().await;
            let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
            let tmp: Vec<String> = conn.query("SHOW DATABASES LIKE \'GenshinOJ\'").await.unwrap();
            if !tmp.iter().any(|x| x == "GenshinOJ") {
                "CREATE DATABASE GenshinOJ".ignore(&mut conn).await.unwrap();
            }
            "USE GenshinOJ".ignore(&mut conn).await.unwrap();
            let tmp: Vec<String> = conn.query("SHOW TABLES LIKE \'users\'").await.unwrap();
            if !tmp.iter().any(|x| x == "users") {
                "CREATE TABLE users (
                    id INT AUTO_INCREMENT PRIMARY KEY NOT NULL,
                    username VARCHAR(256) NOT NULL,
                    password VARCHAR(256) NOT NULL
                )"
                    .ignore(&mut conn).await
                    .unwrap();
            }
            drop(conn);
            drop(guard_mysql_database_pool);
            let mut guard_simple_authenticator_status: tokio::sync::MutexGuard<
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
                    format!("127.0.0.1:{}", &simple_authenticator_socket_port)
                ).await;
                if let Ok(x) = simple_authenticator_socket_result {
                    println!(
                        "{}",
                        ansi_term::Color::Green.paint(
                            format!(
                                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the socket on port {}.",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                simple_authenticator_socket_port
                            )
                        )
                    );
                    break (x, simple_authenticator_socket_port);
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
                                simple_authenticator_socket_port
                            )
                        )
                    );
                }
                if simple_authenticator_socket_port == u16::MAX {
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
                simple_authenticator_socket_port += 1;
                fake_yield_now(0).await;
            };
            SIMPLE_AUTHENTICATOR_SOCKET.set(
                new_async_modifiable(simple_authenticator_socket)
            ).unwrap();
            drop(guard_simple_authenticator_status_socket_port);
            guard_simple_authenticator_status.initialized = true;
            drop(guard_simple_authenticator_status);
        });
    }

    {
        let simple_authenticator_status: AsyncModifiable<ModuleStatus> =
            simple_authenticator_status.clone();
        simple_authenticator_runtime.spawn(async move {
            loop {
                // Waiting for the initialization to be completed.
                let guard_simple_authenticator_status: tokio::sync::MutexGuard<
                    '_,
                    ModuleStatus
                > = simple_authenticator_status.lock().await;
                if guard_simple_authenticator_status.initialized {
                    drop(guard_simple_authenticator_status);
                    break;
                }
                drop(guard_simple_authenticator_status);
                fake_yield_now(1000).await;
            }

            // Now processing socket message
            crate::socket_message_processing::socket_message_processing().await;
        });
    }

    {
        simple_authenticator_runtime.spawn(async move {
            loop {
                // Waiting for the initialization of ws_server to be completed.
                let guard_global_module_statuses_by_protocol =
                    global_module_statuses_by_protocol.lock().await;
                if
                    let Some(ws_server_status) =
                        guard_global_module_statuses_by_protocol.get("std_ws_server")
                {
                    let guard_ws_server_status = ws_server_status.lock().await;
                    if guard_ws_server_status.initialized {
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
                        break;
                    } else {
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
                    }
                } else {
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
                }
                fake_yield_now(1000).await;
            }

            simple_authenticator_socket::connect_to_ws_server().await;
        });
    }

    {
        let simple_authenticator_status: AsyncModifiable<ModuleStatus> =
            simple_authenticator_status.clone();
        simple_authenticator_runtime.spawn(
            self_management::self_management(simple_authenticator_status)
        );
    }
    (simple_authenticator_runtime, simple_authenticator_status)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    let simple_authenticator_runtime_on_unload: tokio::runtime::Runtime = tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Unloading the simple authenticator...",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );

    simple_authenticator_runtime_on_unload.spawn(async move {
        let guard_global_module_statuses_by_protocol = GLOBAL_MODULE_STATUSES_BY_PROTOCOL.get()
            .unwrap()
            .lock().await;
        if
            let Some(ws_server_status) =
                guard_global_module_statuses_by_protocol.get("std_ws_server")
        {
            let guard_ws_server_status = ws_server_status.lock().await;
            if guard_ws_server_status.initialized {
                simple_authenticator_socket::disconnect_from_ws_server().await;
            }
        }

        // TODO: To tell main backend to drop this module.
    });

    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Unloaded the simple authenticator.",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
}
