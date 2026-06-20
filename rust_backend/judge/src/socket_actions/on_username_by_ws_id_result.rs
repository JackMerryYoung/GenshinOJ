use crate::global::*;

use mysql_async::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnUsernameByWsIdResult {
    username: Option<String>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SubmissionResultOwn {
    submission_id: i64,
    result: String,
    general_score: i32,
    statuses: Vec<String>,
    scores: Vec<i32>,
    problem_number: i64,
}

#[derive(serde::Serialize)]
struct SubmissionResultOthers {
    submission_id: i64,
    result: String,
    problem_number: i64,
}

#[derive(serde::Serialize)]
struct ContentInSubmissionsListResult {
    submissions_list: Vec<serde_json::Value>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SubmissionsListResult {
    r#type: String,
    content: ContentInSubmissionsListResult,
}

#[derive(serde::Serialize)]
struct ContentInSubmissionResult {
    submission_id: i64,
    result: String,
    general_score: i32,
    statuses: Vec<String>,
    scores: Vec<i32>,
    problem_number: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<Vec<String>>,
    language: String,
    username: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SubmissionResultResult {
    r#type: String,
    content: ContentInSubmissionResult,
}

async fn send_json_msg_to_ws_server(ws_id: String, msg_to_send: serde_json::Value) {
    let json_msg: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_send_msg"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnSendMsg { ws_id, msg_to_send })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_judge"),
    };
    send_socket_json_message(&serde_json::to_value(json_msg).unwrap(), "std_ws_server").await;
}

async fn handle_submission_result_fetch(
    requester_username: Option<String>,
    pending_request: PendingSubmissionResultFetchRequest
) {
    let is_owner: bool = requester_username.as_deref() == Some(pending_request.owner_username.as_str());

    let submission_result_result = SubmissionResultResult {
        r#type: String::from("submission_result"),
        content: ContentInSubmissionResult {
            submission_id: pending_request.submission_id,
            result: pending_request.result,
            general_score: pending_request.general_score,
            statuses: pending_request.statuses,
            scores: pending_request.scores,
            problem_number: pending_request.problem_number,
            code: if is_owner { Some(pending_request.code) } else { None },
            language: pending_request.language,
            username: pending_request.owner_username,
            request_key: pending_request.original_request_key,
        },
    };

    send_json_msg_to_ws_server(
        pending_request.requester_ws_id,
        serde_json::to_value(submission_result_result).unwrap()
    ).await;
}

pub async fn on_username_by_ws_id_result(msg: SocketJsonMessage) {
    if
        let Ok(content) = serde_json::from_value::<ContentOnUsernameByWsIdResult>(msg.content)
    {
        let mut guard_pending_submission_result_fetch_requests: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<String, PendingSubmissionResultFetchRequest>
        > = PENDING_SUBMISSION_RESULT_FETCH_REQUESTS.lock().await;
        if
            let Some(pending_request) = guard_pending_submission_result_fetch_requests.remove(
                &content.request_key
            )
        {
            drop(guard_pending_submission_result_fetch_requests);
            handle_submission_result_fetch(content.username, pending_request).await;
            return;
        }
        drop(guard_pending_submission_result_fetch_requests);

        let mut guard_pending_submissions_list_requests: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<String, PendingSubmissionsListRequest>
        > = PENDING_SUBMISSIONS_LIST_REQUESTS.lock().await;
        let Some(pending_request) = guard_pending_submissions_list_requests.remove(
            &content.request_key
        ) else {
            drop(guard_pending_submissions_list_requests);
            println!(
                "{}",
                ansi_term::Color::Yellow.paint(
                    format!(
                        "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Received a username-by-ws_id result for an unknown request key `{}`.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        content.request_key
                    )
                )
            );
            return;
        };
        drop(guard_pending_submissions_list_requests);

        let requester_username: Option<String> = content.username;

        let page_index: i64 = std::cmp::max(1, pending_request.page_index);
        let offset: i64 = (page_index - 1) * SUBMISSIONS_LIST_PAGE_SIZE;

        let guard_mysql_database_pool: tokio::sync::MutexGuard<
            '_,
            mysql_async::Pool
        > = MYSQL_DATABASE_POOL.lock().await;
        let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
        let results: Result<
            Vec<(i64, String, i64, String, i32, String, String)>,
            _
        > = conn
            .exec(
                "SELECT submission_id, username, problem_number, result, general_score, statuses, scores
                FROM GenshinOJ.submissions
                ORDER BY submission_id DESC
                LIMIT :limit OFFSET :offset",
                mysql_async::params! { "limit" => SUBMISSIONS_LIST_PAGE_SIZE, "offset" => offset }
            )
            .await;
        drop(conn);
        drop(guard_mysql_database_pool);

        match results {
            Ok(rows) => {
                let submissions_list: Vec<serde_json::Value> = rows
                    .into_iter()
                    .map(
                        |
                            (
                                submission_id,
                                username,
                                problem_number,
                                result,
                                general_score,
                                statuses,
                                scores,
                            )
                        | {
                            if requester_username.as_deref() == Some(username.as_str()) {
                                serde_json
                                    ::to_value(SubmissionResultOwn {
                                        submission_id,
                                        result,
                                        general_score,
                                        statuses: serde_json
                                            ::from_str(&statuses)
                                            .unwrap_or_default(),
                                        scores: serde_json::from_str(&scores).unwrap_or_default(),
                                        problem_number,
                                    })
                                    .unwrap()
                            } else {
                                serde_json
                                    ::to_value(SubmissionResultOthers {
                                        submission_id,
                                        result,
                                        problem_number,
                                    })
                                    .unwrap()
                            }
                        }
                    )
                    .collect();

                let submissions_list_result = SubmissionsListResult {
                    r#type: String::from("submissions_list"),
                    content: ContentInSubmissionsListResult {
                        submissions_list,
                        request_key: pending_request.original_request_key,
                    },
                };

                send_json_msg_to_ws_server(
                    pending_request.requester_ws_id,
                    serde_json::to_value(submissions_list_result).unwrap()
                ).await;
            }
            Err(e) => {
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to fetch the submissions list (The SQL query is not correct).",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] {}",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            e
                        )
                    )
                );
            }
        }
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
