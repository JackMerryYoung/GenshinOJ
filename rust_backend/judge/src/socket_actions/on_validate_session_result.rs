use crate::global::*;

use mysql_async::prelude::*;

#[derive(serde::Deserialize)]
struct ContentOnValidateSessionResult {
    session_valid: bool,
    request_key: String,
}

#[derive(serde::Serialize)]
struct ContentInSubmissionId {
    submission_id: i64,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SubmissionIdResult {
    r#type: String,
    content: ContentInSubmissionId,
}

#[derive(serde::Serialize)]
struct ContentInSubmissionFailure {
    reason: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SubmissionFailureResult {
    r#type: String,
    content: ContentInSubmissionFailure,
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

pub async fn on_validate_session_result(msg: SocketJsonMessage) {
    if let Ok(content) = serde_json::from_value::<ContentOnValidateSessionResult>(msg.content) {
        let mut guard_pending_submission_requests = PENDING_SUBMISSION_REQUESTS.lock().await;
        let Some(pending_request) = guard_pending_submission_requests.remove(
            &content.request_key
        ) else {
            drop(guard_pending_submission_requests);
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
            return;
        };
        drop(guard_pending_submission_requests);

        if !content.session_valid {
            let failure_result = SubmissionFailureResult {
                r#type: String::from("submission_failure"),
                content: ContentInSubmissionFailure {
                    reason: String::from("invalid_session"),
                    request_key: pending_request.original_request_key,
                },
            };
            send_json_msg_to_ws_server(
                pending_request.requester_ws_id,
                serde_json::to_value(failure_result).unwrap()
            ).await;
            return;
        }

        let guard_mysql_database_pool: tokio::sync::MutexGuard<
            '_,
            mysql_async::Pool
        > = MYSQL_DATABASE_POOL.lock().await;
        let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
        let insert_result = conn
            .exec_drop(
                "INSERT INTO GenshinOJ.submissions
                (username, problem_number, result, general_score, statuses, scores, code, language, is_test_submission_mode, created_at)
                VALUES (:username, :problem_number, 'PD', 0, '[]', '[]', :code, :language, :is_test_submission_mode, :created_at)",
                mysql_async::params! {
                    "username" => &pending_request.username,
                    "problem_number" => pending_request.problem_number,
                    "code" => serde_json::to_string(&pending_request.code).unwrap(),
                    "language" => &pending_request.language,
                    "is_test_submission_mode" => pending_request.is_test_submission_mode,
                    "created_at" => chrono::Utc::now().timestamp(),
                }
            )
            .await;

        if insert_result.is_err() {
            drop(conn);
            drop(guard_mysql_database_pool);
            println!(
                "{}",
                ansi_term::Color::Yellow.paint(
                    format!(
                        "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to insert the submission row (The SQL query is not correct).",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    )
                )
            );
            let failure_result = SubmissionFailureResult {
                r#type: String::from("submission_failure"),
                content: ContentInSubmissionFailure {
                    reason: String::from("internal_error"),
                    request_key: pending_request.original_request_key,
                },
            };
            send_json_msg_to_ws_server(
                pending_request.requester_ws_id,
                serde_json::to_value(failure_result).unwrap()
            ).await;
            return;
        }

        let submission_id: i64 = conn.last_insert_id().unwrap_or(0) as i64;

        // `general` counts every submission attempt, TSM included.
        let general_update_result = conn
            .exec_drop(
                "UPDATE GenshinOJ.users SET general = general + 1 WHERE username = :username",
                mysql_async::params! { "username" => &pending_request.username }
            )
            .await;
        if general_update_result.is_err() {
            println!(
                "{}",
                ansi_term::Color::Yellow.paint(
                    format!(
                        "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to increment `general` for user `{}`.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        pending_request.username
                    )
                )
            );
        }

        drop(conn);
        drop(guard_mysql_database_pool);

        let submission_id_result = SubmissionIdResult {
            r#type: String::from("submission_id"),
            content: ContentInSubmissionId {
                submission_id,
                request_key: pending_request.original_request_key.clone(),
            },
        };
        send_json_msg_to_ws_server(
            pending_request.requester_ws_id.clone(),
            serde_json::to_value(submission_id_result).unwrap()
        ).await;

        let problem_number = pending_request.problem_number;
        let language = pending_request.language;
        let code = pending_request.code;
        let username = pending_request.username;
        let is_test_submission_mode = pending_request.is_test_submission_mode;
        let requester_ws_id = pending_request.requester_ws_id;
        let original_request_key = pending_request.original_request_key;
        tokio::spawn(async move {
            let outcome = crate::judging::judge_submission(problem_number, &language, &code).await;

            let guard_mysql_database_pool: tokio::sync::MutexGuard<
                '_,
                mysql_async::Pool
            > = MYSQL_DATABASE_POOL.lock().await;
            let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
            let update_result = conn
                .exec_drop(
                    "UPDATE GenshinOJ.submissions
                    SET result = :result, general_score = :general_score, statuses = :statuses, scores = :scores
                    WHERE submission_id = :submission_id",
                    mysql_async::params! {
                        "result" => &outcome.result,
                        "general_score" => outcome.general_score,
                        "statuses" => serde_json::to_string(&outcome.statuses).unwrap(),
                        "scores" => serde_json::to_string(&outcome.scores).unwrap(),
                        "submission_id" => submission_id,
                    }
                )
                .await;

            if update_result.is_err() {
                drop(conn);
                drop(guard_mysql_database_pool);
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to update the submission `{}` with judging results.",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            submission_id
                        )
                    )
                );
                return;
            }

            if outcome.result == "AC" {
                let counter_column = if is_test_submission_mode { "test_accepted" } else { "accepted" };
                let counter_update_result = conn
                    .exec_drop(
                        format!(
                            "UPDATE GenshinOJ.users SET {} = {} + 1 WHERE username = :username",
                            counter_column,
                            counter_column
                        ),
                        mysql_async::params! { "username" => &username }
                    )
                    .await;
                if counter_update_result.is_err() {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to increment `{}` for user `{}`.",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                counter_column,
                                username
                            )
                        )
                    );
                }
            }

            drop(conn);
            drop(guard_mysql_database_pool);

            crate::socket_actions::push_submission_result::push_submission_result(
                requester_ws_id,
                submission_id,
                problem_number,
                code,
                language,
                username,
                outcome,
                original_request_key
            ).await;
        });
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
