use crate::global::*;

#[derive(serde::Serialize)]
struct ContentInSubmissionResult {
    submission_id: i64,
    result: String,
    general_score: i32,
    statuses: Vec<String>,
    scores: Vec<i32>,
    problem_number: i64,
    code: Vec<String>,
    language: String,
    username: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SubmissionResultResult {
    r#type: String,
    content: ContentInSubmissionResult,
}

// Pushed to the original submitter (the only one who can see their own `code`) right after the
// background judging task finishes, so they don't have to poll `on_submission_result` themselves
// if they're still on the page.
pub async fn push_submission_result(
    ws_id: String,
    submission_id: i64,
    problem_number: i64,
    code: Vec<String>,
    language: String,
    username: String,
    outcome: crate::judging::JudgeOutcome,
    request_key: String
) {
    let submission_result_result = SubmissionResultResult {
        r#type: String::from("submission_result"),
        content: ContentInSubmissionResult {
            submission_id,
            result: outcome.result,
            general_score: outcome.general_score,
            statuses: outcome.statuses,
            scores: outcome.scores,
            problem_number,
            code,
            language,
            username,
            request_key,
        },
    };

    let json_msg: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_send_msg"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnSendMsg {
                ws_id,
                msg_to_send: serde_json::to_value(submission_result_result).unwrap(),
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_judge"),
    };
    send_socket_json_message(&serde_json::to_value(json_msg).unwrap(), "std_ws_server").await;
}
