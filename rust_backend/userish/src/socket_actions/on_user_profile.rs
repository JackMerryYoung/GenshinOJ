use crate::global::*;

use mysql_async::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnUserProfile {
    username: String,
    #[serde(default)]
    viewer_username: Option<String>,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentInUserProfileResult {
    username: String,
    accepted: i32,
    test_accepted: i32,
    general: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_following: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_followed_by: Option<bool>,
    request_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct UserProfileResult {
    r#type: String,
    content: ContentInUserProfileResult,
}

pub async fn on_user_profile(msg: SocketJsonMessageWithWsId) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnUserProfile>(
            msg.content
        )
    {
        let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
        let results: Result<Vec<(i32, i32, i32)>, _> = conn
            .exec(
                "SELECT accepted, test_accepted, general FROM RsOJ.users WHERE username = :username",
                mysql_async::params! { "username" => &content.username }
            )
            .await;

        let (is_following, is_followed_by) = match &content.viewer_username {
            Some(viewer_username) if viewer_username != &content.username => {
                let following: Vec<i32> = conn
                    .exec(
                        "SELECT 1 FROM RsOJ.follows WHERE follower_username = :viewer AND followee_username = :target",
                        mysql_async::params! { "viewer" => viewer_username, "target" => &content.username }
                    )
                    .await
                    .unwrap_or_default();
                let followed_by: Vec<i32> = conn
                    .exec(
                        "SELECT 1 FROM RsOJ.follows WHERE follower_username = :target AND followee_username = :viewer",
                        mysql_async::params! { "viewer" => viewer_username, "target" => &content.username }
                    )
                    .await
                    .unwrap_or_default();
                (Some(!following.is_empty()), Some(!followed_by.is_empty()))
            }
            _ => (None, None),
        };

        drop(conn);

        match results {
            Ok(results_unwrapped) => {
                if let Some((accepted, test_accepted, general)) = results_unwrapped.into_iter().next() {
                    let user_profile_result = UserProfileResult {
                        r#type: String::from("user_profile"),
                        content: ContentInUserProfileResult {
                            username: content.username.clone(),
                            accepted,
                            test_accepted,
                            general,
                            is_following,
                            is_followed_by,
                            request_key: content.request_key,
                        },
                    };

                    let msg_to_send: SocketJsonMessage = SocketJsonMessage {
                        r#type: String::from("on_send_msg"),
                        content: serde_json
                            ::to_value(SocketJsonMessageContentOnSendMsg {
                                ws_id: msg.ws_id,
                                msg_to_send: serde_json::to_value(user_profile_result).unwrap(),
                            })
                            .unwrap(),
                        request_key: msg.request_key,
                        from_protocol: String::from("std_userish"),
                    };
                    send_socket_json_message(
                        &serde_json::to_value(msg_to_send).unwrap(),
                        "std_ws_server"
                    ).await;
                } else {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Someone tried to fetch the profile of the user `{}`, who doesn't exist.",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                content.username
                            )
                        )
                    );
                }
            }
            Err(e) => {
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to fetch the profile of the user `{}` (The SQL query is not correct).",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            content.username
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
    }
}
