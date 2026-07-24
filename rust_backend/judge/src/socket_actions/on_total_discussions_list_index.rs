use crate::global::*;

use mysql_async::prelude::*;

#[derive(serde::Deserialize)]
struct Content {
    request_key: String,
}

#[derive(serde::Serialize)]
struct ContentInResult {
    total_discussions_list_index: i64,
    request_key: String,
}

#[derive(serde::Serialize)]
struct Result_ {
    r#type: String,
    content: ContentInResult,
}

pub async fn on_total_discussions_list_index(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<Content>(msg.content) {
        let guard_pool = MYSQL_DATABASE_POOL.lock().await;
        let mut conn: mysql_async::Conn = guard_pool.get_conn().await.unwrap();
        let results: std::result::Result<Vec<i64>, _> = conn
            .query("SELECT COUNT(*) FROM RsOJ.discussions")
            .await;
        drop(conn);
        drop(guard_pool);

        match results {
            Ok(counts) => {
                let total: i64 = counts.into_iter().next().unwrap_or(0);
                let total_discussions_list_index: i64 = std::cmp::max(
                    1,
                    (total + DISCUSSIONS_LIST_PAGE_SIZE - 1) / DISCUSSIONS_LIST_PAGE_SIZE
                );

                let result = Result_ {
                    r#type: String::from("total_discussions_list_index"),
                    content: ContentInResult {
                        total_discussions_list_index,
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
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to count the total discussions: {}",
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
