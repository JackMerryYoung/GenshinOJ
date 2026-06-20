use crate::global::*;

#[derive(serde::Deserialize)]
struct ContentOnValidateSessionResult {
    session_valid: bool,
    request_key: String,
}

pub async fn on_validate_session_result(msg: SocketJsonMessage) {
    let Ok(content) = serde_json::from_value::<ContentOnValidateSessionResult>(msg.content) else {
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
        return;
    };

    let mut guard_pending_follow_requests = PENDING_FOLLOW_REQUESTS.lock().await;
    if let Some(pending_request) = guard_pending_follow_requests.remove(&content.request_key) {
        drop(guard_pending_follow_requests);
        if !content.session_valid {
            crate::socket_actions::on_follow::send_result(pending_request.ws_id, pending_request.request_key, false).await;
            return;
        }
        let success: bool = crate::socket_actions::on_follow::perform_follow(
            &pending_request.follower_username,
            &pending_request.followee_username
        ).await;
        crate::socket_actions::on_follow::send_result(pending_request.ws_id, pending_request.request_key, success).await;
        return;
    }
    drop(guard_pending_follow_requests);

    let mut guard_pending_unfollow_requests = PENDING_UNFOLLOW_REQUESTS.lock().await;
    if let Some(pending_request) = guard_pending_unfollow_requests.remove(&content.request_key) {
        drop(guard_pending_unfollow_requests);
        if !content.session_valid {
            crate::socket_actions::on_unfollow::send_result(pending_request.ws_id, pending_request.request_key, false).await;
            return;
        }
        let success: bool = crate::socket_actions::on_unfollow::perform_unfollow(
            &pending_request.follower_username,
            &pending_request.followee_username
        ).await;
        crate::socket_actions::on_unfollow::send_result(pending_request.ws_id, pending_request.request_key, success).await;
        return;
    }
    drop(guard_pending_unfollow_requests);

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
