use crate::global::*;

use mysql_async::prelude::*;

#[derive(serde::Deserialize)]
struct Content {
    problem_number: i64,
    request_key: String,
}

#[derive(serde::Serialize)]
struct ContentInResult {
    total_solutions_list_index: i64,
    request_key: String,
}

#[derive(serde::Serialize)]
struct Result_ {
    r#type: String,
    content: ContentInResult,
}

pub async fn on_total_solutions_list_index(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<Content>(msg.content) {
        let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
        let results: std::result::Result<Vec<i64>, _> = conn
            .exec(
                "SELECT COUNT(*) FROM RsOJ.solutions WHERE problem_number = :problem_number",
                mysql_async::params! { "problem_number" => content.problem_number }
            )
            .await;
        drop(conn);

        match results {
            Ok(counts) => {
                let total_solutions: i64 = counts.into_iter().next().unwrap_or(0);
                // At least one page even with no solutions, matching the submissions convention and
                // the frontend's "page X / max(total, 1)" display.
                let total_solutions_list_index: i64 = std::cmp::max(
                    1,
                    (total_solutions + SOLUTIONS_LIST_PAGE_SIZE - 1) / SOLUTIONS_LIST_PAGE_SIZE
                );

                let result = Result_ {
                    r#type: String::from("total_solutions_list_index"),
                    content: ContentInResult {
                        total_solutions_list_index,
                        request_key: content.request_key,
                    },
                };
                let msg_to_send: SocketJsonMessage = SocketJsonMessage {
                    r#type: String::from("on_send_msg"),
                    content: serde_json
                        ::to_value(SocketJsonMessageContentOnSendMsg {
                            ws_id: msg.ws_id,
                            msg_to_send: serde_json::to_value(result).unwrap(),
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
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to count the total solutions: {}",
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
    }
}
