use crate::global::*;

use mysql_async::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnFriendsList {
    username: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct ContentInFriendsListResult {
    friends: Vec<String>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct FriendsListResult {
    r#type: String,
    content: ContentInFriendsListResult,
}

pub async fn on_friends_list(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<ContentOnFriendsList>(msg.content) {
        let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
        let results: Result<Vec<String>, _> = conn
            .exec(
                "SELECT followee_username FROM RsOJ.follows
                WHERE follower_username = :username
                ORDER BY followee_username ASC",
                mysql_async::params! { "username" => &content.username }
            )
            .await;
        drop(conn);

        match results {
            Ok(friends) => {
                let friends_list_result = FriendsListResult {
                    r#type: String::from("friends_list"),
                    content: ContentInFriendsListResult {
                        friends,
                        request_key: content.request_key,
                    },
                };

                let json_msg: SocketJsonMessage = SocketJsonMessage {
                    r#type: String::from("on_send_msg"),
                    content: serde_json
                        ::to_value(SocketJsonMessageContentOnSendMsg {
                            ws_id: msg.ws_id,
                            msg_to_send: serde_json::to_value(friends_list_result).unwrap(),
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
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to fetch the friends list (The SQL query is not correct).",
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
