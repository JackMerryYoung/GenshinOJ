use crate::global::*;
use crate::socket_actions;

use tokio::io::AsyncReadExt;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnBindListener {
    pub protocol: String,
    pub version: String,
    pub commands_to_bind: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnUnbindListener {
    pub protocol: String,
    pub version: String,
    pub commands_to_unbind: Vec<String>,
}

pub async fn connect_to_ws_server() {
    let msg_to_send = SocketJsonMessage {
        r#type: String::from("on_bind_listener"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnBindListener {
                protocol: String::from("std_judge"),
                version: String::from(env!("CARGO_PKG_VERSION")),
                commands_to_bind: vec![
                    String::from("on_problem_set"),
                    String::from("on_problem_statement"),
                    String::from("on_total_submissions_list_index"),
                    String::from("on_submissions_list"),
                    String::from("on_submission"),
                    String::from("on_submission_result"),
                    String::from("on_solutions_list"),
                    String::from("on_total_solutions_list_index"),
                    String::from("on_solution"),
                    String::from("on_solution_post"),
                    String::from("on_solution_vote"),
                    String::from("on_solution_comments_list"),
                    String::from("on_total_solution_comments_list_index"),
                    String::from("on_solution_comment_post"),
                    String::from("on_solution_comment_vote"),
                    String::from("on_discussions_list"),
                    String::from("on_total_discussions_list_index"),
                    String::from("on_discussion"),
                    String::from("on_discussion_post"),
                    String::from("on_discussion_vote"),
                    String::from("on_discussion_replies_list"),
                    String::from("on_total_discussion_replies_list_index"),
                    String::from("on_discussion_reply_post"),
                    String::from("on_discussion_reply_vote"),
                    String::from("on_notifications_list"),
                    String::from("on_total_notifications_list_index"),
                    String::from("on_notifications_unread_count"),
                    String::from("on_notification_mark_read"),
                    String::from("on_user_search")
                ],
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_judge"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_ws_server").await;
}

pub async fn disconnect_from_ws_server() {
    let msg_to_send = SocketJsonMessage {
        r#type: String::from("on_unbind_listener"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnUnbindListener {
                protocol: String::from("std_judge"),
                version: String::from(env!("CARGO_PKG_VERSION")),
                commands_to_unbind: vec![
                    String::from("on_problem_set"),
                    String::from("on_problem_statement"),
                    String::from("on_total_submissions_list_index"),
                    String::from("on_submissions_list"),
                    String::from("on_submission"),
                    String::from("on_submission_result"),
                    String::from("on_solutions_list"),
                    String::from("on_total_solutions_list_index"),
                    String::from("on_solution"),
                    String::from("on_solution_post"),
                    String::from("on_solution_vote"),
                    String::from("on_solution_comments_list"),
                    String::from("on_total_solution_comments_list_index"),
                    String::from("on_solution_comment_post"),
                    String::from("on_solution_comment_vote"),
                    String::from("on_discussions_list"),
                    String::from("on_total_discussions_list_index"),
                    String::from("on_discussion"),
                    String::from("on_discussion_post"),
                    String::from("on_discussion_vote"),
                    String::from("on_discussion_replies_list"),
                    String::from("on_total_discussion_replies_list_index"),
                    String::from("on_discussion_reply_post"),
                    String::from("on_discussion_reply_vote"),
                    String::from("on_notifications_list"),
                    String::from("on_total_notifications_list_index"),
                    String::from("on_notifications_unread_count"),
                    String::from("on_notification_mark_read"),
                    String::from("on_user_search")
                ],
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_judge"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_ws_server").await;
}

pub async fn socket_message_processing() {
    let guard_judge_socket: tokio::sync::MutexGuard<
        '_,
        tokio::net::TcpListener
    > = JUDGE_SOCKET.get().unwrap().lock().await;
    loop {
        let (mut client, _) = guard_judge_socket.accept().await.unwrap();
        client.set_nodelay(true).ok();
        tokio::spawn(async move {
            let mut buf: bytes::BytesMut = bytes::BytesMut::with_capacity(1024);
            client.read_buf(&mut buf).await.unwrap();
            let msg: Result<serde_json::Value, serde_json::Error> = serde_json::from_slice(&buf);
            if let Ok(msg) = msg {
                if msg.get("ws_id").is_some() {
                    // The JSON message is a `SocketJsonMessageWithWsId` value, forwarded from
                    // ws_server (a real client sent it).
                    if let Ok(msg) = serde_json::from_value::<SocketJsonMessageWithWsId>(msg) {
                        println!(
                            "{}",
                            ansi_term::Color::Blue.paint(
                                format!(
                                    "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Received socket message: {:?}",
                                    MODULE_IDENTITY,
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    msg
                                )
                            )
                        );
                        if msg.r#type == "on_problem_set" {
                            socket_actions::on_problem_set::on_problem_set(msg).await;
                        } else if msg.r#type == "on_problem_statement" {
                            socket_actions::on_problem_statement::on_problem_statement(msg).await;
                        } else if msg.r#type == "on_total_submissions_list_index" {
                            socket_actions::on_total_submissions_list_index::on_total_submissions_list_index(
                                msg
                            ).await;
                        } else if msg.r#type == "on_submissions_list" {
                            socket_actions::on_submissions_list::on_submissions_list(msg).await;
                        } else if msg.r#type == "on_submission" {
                            socket_actions::on_submission::on_submission(msg).await;
                        } else if msg.r#type == "on_submission_result" {
                            socket_actions::on_submission_result::on_submission_result(msg).await;
                        } else if msg.r#type == "on_solutions_list" {
                            socket_actions::on_solutions_list::on_solutions_list(msg).await;
                        } else if msg.r#type == "on_total_solutions_list_index" {
                            socket_actions::on_total_solutions_list_index::on_total_solutions_list_index(
                                msg
                            ).await;
                        } else if msg.r#type == "on_solution" {
                            socket_actions::on_solution::on_solution(msg).await;
                        } else if msg.r#type == "on_solution_post" {
                            socket_actions::on_solution_post::on_solution_post(msg).await;
                        } else if msg.r#type == "on_solution_vote" {
                            socket_actions::on_solution_vote::on_solution_vote(msg).await;
                        } else if msg.r#type == "on_solution_comments_list" {
                            socket_actions::on_solution_comments_list::on_solution_comments_list(
                                msg
                            ).await;
                        } else if msg.r#type == "on_total_solution_comments_list_index" {
                            socket_actions::on_total_solution_comments_list_index::on_total_solution_comments_list_index(
                                msg
                            ).await;
                        } else if msg.r#type == "on_solution_comment_post" {
                            socket_actions::on_solution_comment_post::on_solution_comment_post(
                                msg
                            ).await;
                        } else if msg.r#type == "on_solution_comment_vote" {
                            socket_actions::on_solution_comment_vote::on_solution_comment_vote(
                                msg
                            ).await;
                        } else if msg.r#type == "on_discussions_list" {
                            socket_actions::on_discussions_list::on_discussions_list(msg).await;
                        } else if msg.r#type == "on_total_discussions_list_index" {
                            socket_actions::on_total_discussions_list_index::on_total_discussions_list_index(
                                msg
                            ).await;
                        } else if msg.r#type == "on_discussion" {
                            socket_actions::on_discussion::on_discussion(msg).await;
                        } else if msg.r#type == "on_discussion_post" {
                            socket_actions::on_discussion_post::on_discussion_post(msg).await;
                        } else if msg.r#type == "on_discussion_vote" {
                            socket_actions::on_discussion_vote::on_discussion_vote(msg).await;
                        } else if msg.r#type == "on_discussion_replies_list" {
                            socket_actions::on_discussion_replies_list::on_discussion_replies_list(
                                msg
                            ).await;
                        } else if msg.r#type == "on_total_discussion_replies_list_index" {
                            socket_actions::on_total_discussion_replies_list_index::on_total_discussion_replies_list_index(
                                msg
                            ).await;
                        } else if msg.r#type == "on_discussion_reply_post" {
                            socket_actions::on_discussion_reply_post::on_discussion_reply_post(
                                msg
                            ).await;
                        } else if msg.r#type == "on_discussion_reply_vote" {
                            socket_actions::on_discussion_reply_vote::on_discussion_reply_vote(
                                msg
                            ).await;
                        } else if msg.r#type == "on_notifications_list" {
                            socket_actions::on_notifications_list::on_notifications_list(msg).await;
                        } else if msg.r#type == "on_total_notifications_list_index" {
                            socket_actions::on_total_notifications_list_index::on_total_notifications_list_index(
                                msg
                            ).await;
                        } else if msg.r#type == "on_notifications_unread_count" {
                            socket_actions::on_notifications_unread_count::on_notifications_unread_count(
                                msg
                            ).await;
                        } else if msg.r#type == "on_notification_mark_read" {
                            socket_actions::on_notification_mark_read::on_notification_mark_read(
                                msg
                            ).await;
                        } else if msg.r#type == "on_user_search" {
                            socket_actions::on_user_search::on_user_search(msg).await;
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
                } else if let Ok(msg) = serde_json::from_value::<SocketJsonMessage>(msg) {
                    // A direct module-to-module message (no `ws_id`) — the result of asking
                    // simple_authenticator to resolve a username from a ws_id.
                    println!(
                        "{}",
                        ansi_term::Color::Blue.paint(
                            format!(
                                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Received socket message: {:?}",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                msg
                            )
                        )
                    );
                    if msg.r#type == "on_username_by_ws_id_result" {
                        socket_actions::on_username_by_ws_id_result::on_username_by_ws_id_result(
                            msg
                        ).await;
                    } else if msg.r#type == "on_validate_session_result" {
                        socket_actions::on_validate_session_result::on_validate_session_result(
                            msg
                        ).await;
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
        });
    }
}
