use crate::global::*;

pub async fn ip_handler(
    axum_client_ip::ClientIp(ip_addr): axum_client_ip::ClientIp,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    println!(
        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Connection from `{}` established.",
        std::thread::current().id().as_u64(),
        file!(),
        line!(),
        ip_addr
    );
    next.run(request).await
}

pub async fn ws_handler(ws_upgrade: axum::extract::ws::WebSocketUpgrade) -> axum::response::Response {
    ws_upgrade.on_upgrade(ws_callback)
}

pub async fn ws_callback(mut ws: axum::extract::ws::WebSocket) {
    println!(
        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Websocket connection established.",
        std::thread::current().id().as_u64(),
        file!(),
        line!()
    );

    let mut guard_ws_server_connections_cnt: tokio::sync::MutexGuard<'_, usize> =
        WS_SERVER_CONNECTIONS_CNT.lock().await;
    *guard_ws_server_connections_cnt += 1;
    drop(guard_ws_server_connections_cnt);

    let ws_id: uuid::Uuid = uuid::Uuid::new_v4();
    while let Some(original_msg) = ws.recv().await {
        if let Ok(original_msg) = original_msg {
            match original_msg {
                axum::extract::ws::Message::Text(text) => {
                    if let Ok(mut json_msg) =
                        serde_json::from_str::<WebsocketServerJsonMessage>(text.as_str())
                    {
                        let taken_json_msg_content: AsyncModifiable<serde_json::Value> =
                            std::sync::Arc::new(tokio::sync::Mutex::new(json_msg.content.take()));
                        let guard_ws_server_applications_libraries: tokio::sync::MutexGuard<
                            '_,
                            Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>,
                        > = WS_SERVER_APPLICATIONS_LIBRARIES.lock().await;
                        for library in guard_ws_server_applications_libraries.iter() {
                            if let Ok(callback_function) = unsafe {
                                library.0.symbol::<unsafe extern "Rust" fn(
                                    &tokio::runtime::Runtime,
                                    (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
                                    AsyncModifiable<serde_json::Value>,
                                )
                                    -> ()>(
                                    &(String::from("on_") + &json_msg.r#type)
                                )
                            } {
                                unsafe {
                                    callback_function(
                                        &library.1,
                                        (&mut ws, &ws_id),
                                        taken_json_msg_content.clone(),
                                    );
                                }
                            };
                        }
                    };
                }
                axum::extract::ws::Message::Close(_) => {
                    println!(
                        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Websocket connection closed.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    );
                    let mut guard_ws_server_connections_cnt: tokio::sync::MutexGuard<'_, usize> =
                        WS_SERVER_CONNECTIONS_CNT.lock().await;
                    *guard_ws_server_connections_cnt -= 1;
                    drop(guard_ws_server_connections_cnt);
                    let guard_ws_server_applications_libraries: tokio::sync::MutexGuard<
                        '_,
                        Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>,
                    > = WS_SERVER_APPLICATIONS_LIBRARIES.lock().await;
                    for library in guard_ws_server_applications_libraries.iter() {
                        if let Ok(callback_function) = unsafe {
                            library.0.symbol::<unsafe extern "Rust" fn(
                                &tokio::runtime::Runtime,
                                (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
                            ) -> ()>(&String::from(
                                "on_close_connection",
                            ))
                        } {
                            unsafe {
                                callback_function(&library.1, (&mut ws, &ws_id));
                            }
                        };
                    }
                }
                _ => {}
            }
        }
    }
}
