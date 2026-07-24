use crate::global::*;
use crate::discussions;

#[derive(serde::Deserialize)]
struct Content {
    username: String,
    session_token: String,
    discussion_id: i64,
    // 1 = like, -1 = dislike, 0 = clear the requester's existing vote.
    vote: i8,
    request_key: String,
}

pub async fn on_discussion_vote(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<Content>(msg.content) {
        discussions::request_session_for_action(
            msg.ws_id,
            content.username,
            content.session_token,
            content.request_key,
            PendingDiscussionAuthedAction::VoteDiscussion {
                discussion_id: content.discussion_id,
                vote: content.vote,
            }
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
