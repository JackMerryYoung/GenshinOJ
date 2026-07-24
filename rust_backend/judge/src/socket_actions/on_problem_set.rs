use crate::global::*;
use mysql_async::prelude::*;

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
        // Read problem set from database instead of JSON file
        let guard = MYSQL_DATABASE_POOL.lock().await;
        match guard.get_conn().await {
            Ok(mut conn) => {
                let query = format!(
                    "SELECT problem_number FROM `{DATABASE_NAME}`.`problems` ORDER BY problem_number ASC"
                );

                match conn.query::<i64, _>(query).await {
                    Ok(problem_numbers) => {
                        let problem_set: Vec<String> = problem_numbers
                            .into_iter()
                            .map(|n| n.to_string())
                            .collect();

                        let problem_set_result = ProblemSetResult {
                            r#type: String::from("problem_set"),
                            content: ContentInProblemSetResult {
                                problem_set,
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
                                    "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to query problem set from database: {}",
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
