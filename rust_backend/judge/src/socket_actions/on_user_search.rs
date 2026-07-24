use crate::global::*;
use crate::notifications;

#[derive(serde::Deserialize)]
struct Content {
    prefix: String,
    request_key: String,
}

// Public @mention autocomplete lookup. No identity needed, so it answers directly.
pub async fn on_user_search(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<Content>(msg.content) {
        notifications::user_search(msg.ws_id, content.request_key, content.prefix).await;
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
