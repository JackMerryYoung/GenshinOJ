use crate::global::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn on_login(
    self_rt: &tokio::runtime::Runtime,
    (ws, ws_id): (&mut axum::extract::ws::WebSocket, &uuid::Uuid),
    content: AsyncModifiable<serde_json::Value>,
) {
    let content: AsyncModifiable<serde_json::Value> = content.clone();
    self_rt.block_on(async move {
        let guard_content: tokio::sync::MutexGuard<'_, serde_json::Value> = content.lock().await;
        let cloned_content: serde_json::Value = guard_content.clone();
        let failed_content_reserved: serde_json::Value = guard_content.clone();
        drop(guard_content);
        match serde_json::from_value::<ContentOnLogin>(cloned_content) {
            Ok(unwrapped_content) => {
                
            }
            Err(_) => {
                println!(
                    "{}", ansi_term::Color::Yellow.paint(
                        format!(
                            "[WS_SERVER::SIMPLE_AUTHENTICATOR_APPLICATION] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to login (The JSON message received is in wrong format).",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            failed_content_reserved["username"].as_str().unwrap_or_default()
                        )
                    )
                );

                let response: String = format!(
                    r#"
                    {{
                        "type": "quit",
                        "content": {{
                            "reason": "authentication_failure",
                            "request_key": "{}"
                        }}
                    }}
                    "#,
                    failed_content_reserved["request_key"].as_str().unwrap_or_default()
                );
                ws.send(axum::extract::ws::Message::from(response)).await.unwrap_or_default();
            }
        };
    });
}