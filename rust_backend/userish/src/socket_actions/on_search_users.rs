use crate::global::*;

use mysql_async::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnSearchUsers {
    query: String,
    exclude_username: Option<String>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct ContentInSearchUsersResult {
    users: Vec<String>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SearchUsersResult {
    r#type: String,
    content: ContentInSearchUsersResult,
}

const SEARCH_USERS_LIMIT: i64 = 20;

pub async fn on_search_users(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<ContentOnSearchUsers>(msg.content) {
        // Escape LIKE wildcards in the user-supplied query so `%`/`_` are matched literally
        // instead of behaving as wildcards.
        let escaped_query: String = content.query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        let pattern: String = format!("%{}%", escaped_query);
        let exclude_username: String = content.exclude_username.unwrap_or_default();

        let guard_mysql_database_pool: tokio::sync::MutexGuard<
            '_,
            mysql_async::Pool
        > = MYSQL_DATABASE_POOL.lock().await;
        let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
        let results: Result<Vec<String>, _> = conn
            .exec(
                "SELECT username FROM GenshinOJ.users
                WHERE username LIKE :pattern AND username != :exclude_username
                ORDER BY username ASC
                LIMIT :limit",
                mysql_async::params! {
                    "pattern" => pattern,
                    "exclude_username" => exclude_username,
                    "limit" => SEARCH_USERS_LIMIT,
                }
            )
            .await;
        drop(conn);
        drop(guard_mysql_database_pool);

        match results {
            Ok(users) => {
                let search_users_result = SearchUsersResult {
                    r#type: String::from("search_users_result"),
                    content: ContentInSearchUsersResult {
                        users,
                        request_key: content.request_key,
                    },
                };

                let json_msg: SocketJsonMessage = SocketJsonMessage {
                    r#type: String::from("on_send_msg"),
                    content: serde_json
                        ::to_value(SocketJsonMessageContentOnSendMsg {
                            ws_id: msg.ws_id,
                            msg_to_send: serde_json::to_value(search_users_result).unwrap(),
                        })
                        .unwrap(),
                    request_key: msg.request_key,
                    from_protocol: String::from("std_userish"),
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
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to search users (The SQL query is not correct).",
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
