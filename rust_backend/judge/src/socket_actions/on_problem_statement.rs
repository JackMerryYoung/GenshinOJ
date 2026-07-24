use crate::global::*;
use mysql_async::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnProblemStatement {
    problem_number: i64,
    request_key: String,
}

#[derive(serde::Serialize)]
struct ContentInProblemStatementResult {
    problem_number: i64,
    difficulty: i32,
    problem_name: String,
    problem_statement: Vec<String>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct ProblemStatementResult {
    r#type: String,
    content: ContentInProblemStatementResult,
}

pub async fn on_problem_statement(msg: SocketJsonMessageWithWsId) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnProblemStatement>(
            msg.content
        )
    {
        // Read problem statement from database instead of JSON file
        let guard = MYSQL_DATABASE_POOL.lock().await;
        match guard.get_conn().await {
            Ok(mut conn) => {
                let query = format!(
                    "SELECT problem_number, problem_name, difficulty, problem_statement
                     FROM `{DATABASE_NAME}`.`problems`
                     WHERE problem_number = {}",
                    content.problem_number
                );

                match conn.query_first::<(i64, String, i32, String), _>(query).await {
                    Ok(Some((problem_number, problem_name, difficulty, problem_statement_json))) => {
                        // Parse problem_statement from JSON array
                        match serde_json::from_str::<Vec<String>>(&problem_statement_json) {
                            Ok(problem_statement) => {
                                let problem_statement_result = ProblemStatementResult {
                                    r#type: String::from("problem_statement"),
                                    content: ContentInProblemStatementResult {
                                        problem_number,
                                        difficulty,
                                        problem_name,
                                        problem_statement,
                                        request_key: content.request_key,
                                    },
                                };

                                let msg_to_send: SocketJsonMessage = SocketJsonMessage {
                                    r#type: String::from("on_send_msg"),
                                    content: serde_json
                                        ::to_value(SocketJsonMessageContentOnSendMsg {
                                            ws_id: msg.ws_id,
                                            msg_to_send: serde_json
                                                ::to_value(problem_statement_result)
                                                .unwrap(),
                                        })
                                        .unwrap(),
                                    request_key: msg.request_key,
                                    from_protocol: String::from("std_judge"),
                                };
                                send_socket_json_message(
                                    &serde_json::to_value(msg_to_send).unwrap(),
                                    "std_ws_server"
                                ).await;
                            }
                            Err(e) => {
                                println!(
                                    "{}",
                                    ansi_term::Color::Yellow.paint(
                                        format!(
                                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to parse problem_statement JSON for problem {}: {}",
                                            MODULE_IDENTITY,
                                            std::thread::current().id().as_u64(),
                                            file!(),
                                            line!(),
                                            content.problem_number,
                                            e
                                        )
                                    )
                                );
                            }
                        }
                    }
                    Ok(None) => {
                        println!(
                            "{}",
                            ansi_term::Color::Yellow.paint(
                                format!(
                                    "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Someone tried to fetch the statement of problem `{}`, which doesn't exist.",
                                    MODULE_IDENTITY,
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    content.problem_number
                                )
                            )
                        );
                    }
                    Err(e) => {
                        println!(
                            "{}",
                            ansi_term::Color::Yellow.paint(
                                format!(
                                    "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to query problem statement from database: {}",
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
                drop(conn);
            }
            Err(e) => {
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to connect to database: {}",
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
        drop(guard);
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
