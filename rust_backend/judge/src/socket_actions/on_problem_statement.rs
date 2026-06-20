use crate::global::*;

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
        let problem_statement_json_path =
            get_problem_dir_path() +
            &format!("/{}/problem_statement.json", content.problem_number);

        match std::fs::read_to_string(&problem_statement_json_path) {
            Ok(problem_statement_json_string) => {
                match
                    serde_json::from_str::<ProblemStatementJson>(&problem_statement_json_string)
                {
                    Ok(problem_statement_json) => {
                        let problem_statement_result = ProblemStatementResult {
                            r#type: String::from("problem_statement"),
                            content: ContentInProblemStatementResult {
                                problem_number: problem_statement_json.problem_number,
                                difficulty: problem_statement_json.difficulty,
                                problem_name: problem_statement_json.problem_name,
                                problem_statement: problem_statement_json.problem_statement,
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
                                    "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to parse `{}`: {}",
                                    MODULE_IDENTITY,
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    problem_statement_json_path,
                                    e
                                )
                            )
                        );
                    }
                }
            }
            Err(_) => {
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
