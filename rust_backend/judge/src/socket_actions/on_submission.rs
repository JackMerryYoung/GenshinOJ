use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnSubmission {
    username: String,
    session_token: String,
    problem_number: i64,
    language: String,
    code: Vec<String>,
    #[serde(default)]
    is_test_submission_mode: bool,
    request_key: String,
}

#[derive(serde::Serialize)]
struct ContentOnValidateSession {
    username: String,
    session_token: String,
    request_key: String,
}

pub async fn on_submission(msg: SocketJsonMessageWithWsId) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnSubmission>(
            msg.content
        )
    {
        let rpc_request_key: String = uuid::Uuid::new_v4().to_string();

        let mut guard_pending_submission_requests = PENDING_SUBMISSION_REQUESTS.lock().await;
        guard_pending_submission_requests.insert(rpc_request_key.clone(), PendingSubmissionRequest {
            requester_ws_id: msg.ws_id,
            username: content.username.clone(),
            problem_number: content.problem_number,
            language: content.language,
            code: content.code,
            is_test_submission_mode: content.is_test_submission_mode,
            original_request_key: content.request_key,
        });
        drop(guard_pending_submission_requests);
        expire_pending(&*PENDING_SUBMISSION_REQUESTS, rpc_request_key.clone());

        let msg_to_send: SocketJsonMessage = SocketJsonMessage {
            r#type: String::from("on_validate_session"),
            content: serde_json
                ::to_value(ContentOnValidateSession {
                    username: content.username,
                    session_token: content.session_token,
                    request_key: rpc_request_key,
                })
                .unwrap(),
            request_key: uuid::Uuid::new_v4().to_string(),
            from_protocol: String::from("std_judge"),
        };
        send_socket_json_message(
            &serde_json::to_value(msg_to_send).unwrap(),
            "std_authenticator"
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
