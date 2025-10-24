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
    MYSQL_DATABASE_POOL.set(new_async_modifiable(mysql_async::Pool::new(""))).unwrap();
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
pub extern "Rust" fn on_close_connection(
    self_rt: &tokio::runtime::Runtime,
    (_ws, ws_id): (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
) {
    self_rt.block_on(async move {
        
    });
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_quit(
    self_rt: &tokio::runtime::Runtime,
    (_ws, ws_id): (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
    content: AsyncModifiable<serde_json::Value>,
) {
    self_rt.block_on(async move {
        
    });
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnOnlineUser {
    request_key: String,
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