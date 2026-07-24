use crate::global::*;
use crate::solutions;

#[derive(serde::Deserialize)]
struct Content {
    username: String,
    session_token: String,
    problem_number: i64,
    title: String,
    content: Vec<String>,
    // Reserved for the future admin system; clients can't grant themselves this yet, but the field
    // is carried end-to-end so the star can be set once authorization exists.
    #[serde(default)]
    is_official: bool,
    request_key: String,
}

pub async fn on_solution_post(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<Content>(msg.content) {
        solutions::request_session_for_action(
            msg.ws_id,
            content.username,
            content.session_token,
            content.request_key,
            PendingSolutionAuthedAction::PostSolution {
                problem_number: content.problem_number,
                title: content.title,
                content: content.content,
                is_official: content.is_official,
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
