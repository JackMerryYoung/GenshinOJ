use crate::global::*;
use crate::solutions;

#[derive(serde::Deserialize)]
struct Content {
    solution_id: i64,
    index: i64,
    #[serde(default)]
    sort: String,
    request_key: String,
}

pub async fn on_solution_comments_list(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<Content>(msg.content) {
        solutions::request_username_for_lookup(
            msg.ws_id,
            content.request_key,
            PendingSolutionLookup::SolutionCommentsList {
                solution_id: content.solution_id,
                page_index: content.index,
                sort_by_likes: content.sort == "likes",
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
