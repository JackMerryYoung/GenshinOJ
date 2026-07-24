use crate::global::*;
use crate::notifications;

#[derive(serde::Deserialize)]
struct Content {
    username: String,
    session_token: String,
    // 0 marks every unread notification for the requester; otherwise a single id.
    #[serde(default)]
    notification_id: i64,
    request_key: String,
}

pub async fn on_notification_mark_read(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<Content>(msg.content) {
        notifications::request_session_for_action(
            msg.ws_id,
            content.username,
            content.session_token,
            content.request_key,
            PendingNotificationAuthedAction::MarkRead { notification_id: content.notification_id }
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
