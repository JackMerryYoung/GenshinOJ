use crate::global::*;
use crate::solutions;

#[derive(serde::Deserialize)]
struct Content {
    problem_number: i64,
    index: i64,
    // "likes" sorts by like count; anything else (e.g. "time") sorts newest-first.
    #[serde(default)]
    sort: String,
    request_key: String,
}

pub async fn on_solutions_list(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<Content>(msg.content) {
        solutions::request_username_for_lookup(
            msg.ws_id,
            content.request_key,
            PendingSolutionLookup::SolutionsList {
                problem_number: content.problem_number,
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
