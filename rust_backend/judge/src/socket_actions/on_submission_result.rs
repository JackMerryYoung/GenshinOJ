use crate::global::*;

use mysql_async::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnSubmissionResult {
    submission_id: i64,
    request_key: String,
}

#[derive(serde::Serialize)]
struct ContentInSubmissionNotFound {
    submission_id: i64,
    result: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SubmissionNotFoundResult {
    r#type: String,
    content: ContentInSubmissionNotFound,
}

pub async fn on_submission_result(msg: SocketJsonMessageWithWsId) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnSubmissionResult>(
            msg.content
        )
    {
        let guard_mysql_database_pool: tokio::sync::MutexGuard<
            '_,
            mysql_async::Pool
        > = MYSQL_DATABASE_POOL.lock().await;
        let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
        let row_result: Result<
            Option<(String, i64, String, i32, String, String, String, String)>,
            _
        > = conn
            .exec_first(
                "SELECT username, problem_number, result, general_score, statuses, scores, code, language
                FROM RsOJ.submissions
                WHERE submission_id = :submission_id",
                mysql_async::params! { "submission_id" => content.submission_id }
            )
            .await;
        drop(conn);
        drop(guard_mysql_database_pool);

        match row_result {
            Ok(Some((owner_username, problem_number, result, general_score, statuses, scores, code, language))) => {
                let rpc_request_key: String = uuid::Uuid::new_v4().to_string();

                let mut guard_pending_requests = PENDING_SUBMISSION_RESULT_FETCH_REQUESTS.lock().await;
                guard_pending_requests.insert(rpc_request_key.clone(), PendingSubmissionResultFetchRequest {
                    requester_ws_id: msg.ws_id.clone(),
                    submission_id: content.submission_id,
                    owner_username,
                    problem_number,
                    result,
                    general_score,
                    statuses: serde_json::from_str(&statuses).unwrap_or_default(),
                    scores: serde_json::from_str(&scores).unwrap_or_default(),
                    code: serde_json::from_str(&code).unwrap_or_default(),
                    language,
                    original_request_key: content.request_key,
                });
                drop(guard_pending_requests);

                #[derive(serde::Serialize)]
                struct ContentOnUsernameByWsId {
                    ws_id: String,
                    request_key: String,
                }

                let msg_to_send: SocketJsonMessage = SocketJsonMessage {
                    r#type: String::from("on_username_by_ws_id"),
                    content: serde_json
                        ::to_value(ContentOnUsernameByWsId {
                            ws_id: msg.ws_id,
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
            }
            Ok(None) => {
                let not_found_result = SubmissionNotFoundResult {
                    r#type: String::from("submission_result"),
                    content: ContentInSubmissionNotFound {
                        submission_id: content.submission_id,
                        result: String::from("SNF"),
                        request_key: content.request_key,
                    },
                };
                let json_msg: SocketJsonMessage = SocketJsonMessage {
                    r#type: String::from("on_send_msg"),
                    content: serde_json
                        ::to_value(SocketJsonMessageContentOnSendMsg {
                            ws_id: msg.ws_id,
                            msg_to_send: serde_json::to_value(not_found_result).unwrap(),
                        })
                        .unwrap(),
                    request_key: msg.request_key,
                    from_protocol: String::from("std_judge"),
                };
                send_socket_json_message(
                    &serde_json::to_value(json_msg).unwrap(),
                    "std_ws_server"
                ).await;
            }
            Err(e) => {
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to fetch the submission `{}` (The SQL query is not correct).",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            content.submission_id
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
