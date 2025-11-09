use crate::global::*;

pub async fn get_socket_port_by_protocol(protocol: &str) -> u16 {
    let guard_global_module_statuses_by_protocol: tokio::sync::MutexGuard<
        '_,
        std::collections::HashMap<String, std::sync::Arc<tokio::sync::Mutex<ModuleStatus>>>
    > = GLOBAL_MODULE_STATUSES_BY_PROTOCOL.get().unwrap().lock().await;
    let guard_status: tokio::sync::MutexGuard<
        '_,
        ModuleStatus
    > = guard_global_module_statuses_by_protocol.get(protocol).unwrap().lock().await;
    *guard_status.socket_port.lock().await
}

pub async fn get_socket_by_protocol(protocol: &str) -> tokio::net::TcpStream {
    let socket_port: u16 = get_socket_port_by_protocol(protocol).await;
    match tokio::net::TcpStream::connect(format!("127.0.0.1:{socket_port}")).await {
        Ok(socket) => socket,
        Err(_) => {
            eprintln!(
                "{}",
                ansi_term::Color::Red.paint(
                    format!(
                        "[{}] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to connect to the socket of the module implemented protocol `{}` on port {}.",
                        MODULE_IDENTITY,
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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnBindListener {
    pub protocol: String,
    pub commands_to_bind: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnUnbindListener {
    pub protocol: String,
    pub commands_to_unbind: Vec<String>,
}

pub async fn connect_to_ws_server() {
    let msg_to_send = SocketJsonMessage {
        r#type: String::from("on_bind_listener"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnBindListener {
                protocol: String::from("std_authenticator"),
                commands_to_bind: vec![
                    String::from("on_close_connection"),
                    String::from("on_login_check"),
                    String::from("on_login"),
                    String::from("on_online_user"),
                    String::from("on_quit"),
                    String::from("on_register")
                ],
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_authenticator"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_ws_server").await;
}

pub async fn disconnect_from_ws_server() {
    let msg_to_send = SocketJsonMessage {
        r#type: String::from("on_unbind_listener"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnUnbindListener {
                protocol: String::from("std_authenticator@0.1.0"),
                commands_to_unbind: vec![
                    String::from("on_close_connection"),
                    String::from("on_login_check"),
                    String::from("on_login"),
                    String::from("on_online_user"),
                    String::from("on_quit"),
                    String::from("on_register")
                ],
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_authenticator@0.1.0"),
    };
    send_socket_json_message(
        &serde_json::to_value(msg_to_send).unwrap(),
        "std_ws_server@0.1.0"
    ).await;
}
