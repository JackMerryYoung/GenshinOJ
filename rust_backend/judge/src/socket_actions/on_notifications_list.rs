use crate::global::*;
use crate::notifications;

#[derive(serde::Deserialize)]
struct Content {
    index: i64,
    request_key: String,
}

pub async fn on_notifications_list(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<Content>(msg.content) {
        notifications::request_username_for_lookup(
            msg.ws_id,
            content.request_key,
            PendingNotificationLookup::NotificationsList { page_index: content.index }
        ).await;
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
