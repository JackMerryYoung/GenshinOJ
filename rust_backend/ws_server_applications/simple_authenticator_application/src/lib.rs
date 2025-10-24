#![feature(thread_id_value)]

mod global;
mod on_login;
mod on_register;

use global::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    _rt: &'static tokio::runtime::Runtime,
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
    >,
) -> tokio::runtime::Runtime {
    let simple_authenticator_application_runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL
    .set(global_module_statuses_by_protocol)
    .unwrap();
    simple_authenticator_application_runtime
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload() {
    println!(
        "{}", ansi_term::Color::Blue.paint(
            format!(
                "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloading the Simple Websocket Server Application...",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
    println!(
        "{}", ansi_term::Color::Blue.paint(
            format!(
                "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [INFO] [THREAD {}] [FILE `{}` LINE {}] Unloaded the Simple Websocket Server Application.",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_online_user(
    self_rt: &tokio::runtime::Runtime,
    (ws, _ws_id): (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
    content: AsyncModifiable<serde_json::Value>,
) {
    self_rt.block_on(async move {
        let guard_content: tokio::sync::MutexGuard<'_, serde_json::Value> = content.lock().await;
        let cloned_content: serde_json::Value = guard_content.clone();
        drop(guard_content);
        if let Ok(unwrapped_content) = serde_json::from_value::<ContentOnOnlineUser>(cloned_content)
        {
            let guard_logged_in_usernames: tokio::sync::MutexGuard<
                '_,
                std::collections::HashSet<String>,
            > = LOGGED_IN_USERNAMES.get().unwrap().lock().await;
            let response: String = format!(
                r#"
                {{
                    "type": "online_user",
                    "content": {{
                        "online_users": {:?},
                        "request_key": "{}"
                    }}
                }}
                "#,
                guard_logged_in_usernames.iter().collect::<Vec<&String>>(),
                &unwrapped_content.request_key
            );
            drop(guard_logged_in_usernames);
            ws.send(axum::extract::ws::Message::from(response))
                .await
                .unwrap_or_default();
        }
    });
}

async fn send_to_simple_authenticator(msg: serde_json::Value) {
    let msg: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_send_msg"),
        content: msg,
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_authenticator"),
    };
    let msg: serde_json::Value = serde_json::to_value(&msg).unwrap();
    let msg: String = serde_json::to_string(&msg).unwrap();
    let mut socket: tokio::net::TcpStream =
        simple_authenticator_socket::get_socket_by_protocol("std_simple_authenticator").await;
    socket.writable().await.unwrap();
    socket.write_all(msg.as_bytes()).await.unwrap();
    socket.flush().await.unwrap();
}