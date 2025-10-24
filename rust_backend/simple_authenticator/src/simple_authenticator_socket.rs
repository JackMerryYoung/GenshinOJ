use crate::global::*;

pub async fn get_socket_port_by_protocol(protocol: &str) -> u16 {
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

pub async fn get_socket_by_protocol(protocol: &str) -> tokio::net::TcpStream {
    let socket_port: u16 = get_socket_port_by_protocol(protocol).await;
    match tokio::net::TcpStream::connect(format!("127.0.0.1:{socket_port}")).await {
        Ok(socket) => socket,
        Err(_) => {
            eprintln!(
                "{}", ansi_term::Color::Red.paint(
                    format!(
                        "[SIMPLE_AUTHENTICATOR] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to connect to the socket of the module implemented protocol `{}` on port {}.",
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