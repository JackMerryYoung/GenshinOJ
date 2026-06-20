use crate::global::*;

#[derive(serde::Deserialize)]
struct ContentOnValidateSessionResult {
    session_valid: bool,
    request_key: String,
}

pub async fn on_validate_session_result(msg: SocketJsonMessage) {
    if let Ok(content) = serde_json::from_value::<ContentOnValidateSessionResult>(msg.content) {
        let mut guard_pending_validate_session_requests = PENDING_VALIDATE_SESSION_REQUESTS.lock().await;
        if let Some(tx) = guard_pending_validate_session_requests.remove(&content.request_key) {
            drop(guard_pending_validate_session_requests);
            let _ = tx.send(content.session_valid);
        } else {
            drop(guard_pending_validate_session_requests);
            println!(
                "{}",
                ansi_term::Color::Yellow.paint(
                    format!(
                        "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Received a validate-session result for an unknown request key `{}`.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        content.request_key
                    )
                )
            );
        }
    } else {
        println!(
            "{}",
            ansi_term::Color::Yellow.paint(
                format!(
                    "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is in wrong format.",
                    MODULE_IDENTITY,
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!()
                )
            )
        );
    }
}
