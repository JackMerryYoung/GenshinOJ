use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnProblemSet {
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentInProblemSetResult {
    problem_set: Vec<String>,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ProblemSetResult {
    r#type: String,
    content: ContentInProblemSetResult,
}

pub async fn on_problem_set(msg: SocketJsonMessageWithWsId) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnProblemSet>(
            msg.content
        )
    {
        let problem_set_json_path = get_problem_dir_path() + "/problem_set.json";
        match std::fs::read_to_string(&problem_set_json_path) {
            Ok(problem_set_json_string) => {
                match serde_json::from_str::<ProblemSetJson>(&problem_set_json_string) {
                    Ok(problem_set_json) => {
                        let problem_set_result = ProblemSetResult {
                            r#type: String::from("problem_set"),
                            content: ContentInProblemSetResult {
                                problem_set: problem_set_json.problem_set,
                                request_key: content.request_key,
                            },
                        };

                        let msg_to_send: SocketJsonMessage = SocketJsonMessage {
                            r#type: String::from("on_send_msg"),
                            content: serde_json
                                ::to_value(SocketJsonMessageContentOnSendMsg {
                                    ws_id: msg.ws_id,
                                    msg_to_send: serde_json::to_value(problem_set_result).unwrap(),
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
                                    problem_set_json_path,
                                    e
                                )
                            )
                        );
                    }
                }
            }
            Err(e) => {
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to read `{}`: {}",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            problem_set_json_path,
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
