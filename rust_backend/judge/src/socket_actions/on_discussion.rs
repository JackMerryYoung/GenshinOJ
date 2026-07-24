use crate::global::*;
use crate::discussions;

#[derive(serde::Deserialize)]
struct Content {
    discussion_id: i64,
    request_key: String,
}

pub async fn on_discussion(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<Content>(msg.content) {
        discussions::request_username_for_lookup(
            msg.ws_id,
            content.request_key,
            PendingDiscussionLookup::DiscussionFetch { discussion_id: content.discussion_id }
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
