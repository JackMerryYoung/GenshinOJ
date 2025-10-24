#![feature(thread_id_value)]

mod global;
mod actions;
mod self_management;
mod socket_message_processing;
mod simple_authenticator_socket;

use crate::global::*;

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
        initialized: false,
        panicked: false,
        socket_port: new_async_modifiable(0),
    };
    let simple_authenticator_status: AsyncModifiable<ModuleStatus> =
        new_async_modifiable(simple_authenticator_status);
    {
        let simple_authenticator_status: AsyncModifiable<ModuleStatus> =
            simple_authenticator_status.clone();
        simple_authenticator_runtime.spawn(async move {
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
                        "{}", ansi_term::Color::Blue.paint(
                            format!(
                                "[SIMPLE_AUTHENTICATOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the socket on port {}.",
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
                        "{}", ansi_term::Color::Yellow.paint(
                            format!(
                                "[SIMPLE_AUTHENTICATOR] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to open the socket on port {}. Retrying...",
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
                        "{}", ansi_term::Color::Red.paint(
                            format!(
                                "[SIMPLE_AUTHENTICATOR] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Exceeded maximum retry times. Now quitting... ",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            )
                        )
                    );
                    panic!();
                }
                simple_authenticator_socket_port += 1;
                fake_yield_now().await;
            };
            SIMPLE_AUTHENTICATOR_SOCKET.set(new_async_modifiable(simple_authenticator_socket)).unwrap();
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
            crate::socket_message_processing::socket_message_processing().await;
        });
    }

    {
        let simple_authenticator_status: AsyncModifiable<ModuleStatus> =
            simple_authenticator_status.clone();
        simple_authenticator_runtime.spawn(self_management::self_management(simple_authenticator_status));
    }
    (simple_authenticator_runtime, simple_authenticator_status)
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