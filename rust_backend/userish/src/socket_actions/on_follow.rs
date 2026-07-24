use crate::global::*;

use mysql_async::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnFollow {
    username: String,
    session_token: String,
    target_username: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct ContentInFollowResult {
    success: bool,
    request_key: String,
}

#[derive(serde::Serialize)]
struct FollowResult {
    r#type: String,
    content: ContentInFollowResult,
}

#[derive(serde::Serialize)]
struct ContentOnValidateSession {
    username: String,
    session_token: String,
    request_key: String,
}

pub async fn send_result(ws_id: String, request_key: String, success: bool) {
    let follow_result = FollowResult {
        r#type: String::from("follow_result"),
        content: ContentInFollowResult { success, request_key },
    };
    let json_msg: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_send_msg"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnSendMsg {
                ws_id,
                msg_to_send: serde_json::to_value(follow_result).unwrap(),
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_userish"),
    };
    send_socket_json_message(&serde_json::to_value(json_msg).unwrap(), "std_ws_server").await;
}

pub async fn on_follow(msg: SocketJsonMessageWithWsId) {
    if let Ok(content) = serde_json::from_value::<ContentOnFollow>(msg.content) {
        if content.username == content.target_username {
            send_result(msg.ws_id, content.request_key, false).await;
            return;
        }

        // No local view of session state in this module — ask simple_authenticator via the same
        // `on_validate_session` RPC `judge` uses, and resume in `on_validate_session_result` once
        // the reply arrives.
        let validate_request_key: String = uuid::Uuid::new_v4().to_string();
        {
            let mut guard_pending_follow_requests = PENDING_FOLLOW_REQUESTS.lock().await;
            guard_pending_follow_requests.insert(validate_request_key.clone(), PendingFollowRequest {
                ws_id: msg.ws_id,
                follower_username: content.username.clone(),
                followee_username: content.target_username,
                request_key: content.request_key,
            });
        }

        let validate_msg: SocketJsonMessage = SocketJsonMessage {
            r#type: String::from("on_validate_session"),
            content: serde_json
                ::to_value(ContentOnValidateSession {
                    username: content.username,
                    session_token: content.session_token,
                    request_key: validate_request_key,
                })
                .unwrap(),
            request_key: uuid::Uuid::new_v4().to_string(),
            from_protocol: String::from("std_userish"),
        };
        send_socket_json_message(&serde_json::to_value(validate_msg).unwrap(), "std_authenticator").await;
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

pub async fn perform_follow(follower_username: &str, followee_username: &str) -> bool {
    let guard_mysql_database_pool: tokio::sync::MutexGuard<
        '_,
        mysql_async::Pool
    > = MYSQL_DATABASE_POOL.lock().await;
    let mut conn: mysql_async::Conn = guard_mysql_database_pool.get_conn().await.unwrap();
    let result = conn
        .exec_drop(
            "INSERT IGNORE INTO RsOJ.follows (follower_username, followee_username)
            VALUES (:follower, :followee)",
            mysql_async::params! {
                "follower" => follower_username,
                "followee" => followee_username,
            }
        )
        .await;
    drop(conn);
    drop(guard_mysql_database_pool);
    result.is_ok()
}
