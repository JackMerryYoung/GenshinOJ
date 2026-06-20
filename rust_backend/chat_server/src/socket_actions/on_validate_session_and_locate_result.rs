use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct ContentOnValidateSessionAndLocateResult {
    session_valid: bool,
    to_ws_id: Option<String>,
    request_key: String,
}

mod msg_to_send_generator {
    #[derive(serde::Serialize)]
    struct ContentInChatEchoSuccess {
        status: u8,
        messages: String,
        created_at: i64,
    }

    #[derive(serde::Serialize)]
    pub struct ChatEchoSuccess {
        r#type: String,
        content: ContentInChatEchoSuccess,
    }

    #[derive(serde::Serialize)]
    struct ContentInChatEchoFailure {
        status: u8,
        reason: String,
    }

    #[derive(serde::Serialize)]
    pub struct ChatEchoFailure {
        r#type: String,
        content: ContentInChatEchoFailure,
    }

    #[derive(serde::Serialize)]
    pub struct ChatMessage {
        r#type: String,
        from: String,
        content: String,
        created_at: i64,
    }

    #[derive(serde::Serialize)]
    struct ChatHistoryMessage {
        id: i64,
        from: String,
        content: String,
        created_at: i64,
    }

    #[derive(serde::Serialize)]
    struct ContentInChatHistoryResult {
        with_username: String,
        messages: Vec<ChatHistoryMessage>,
        has_more: bool,
    }

    #[derive(serde::Serialize)]
    pub struct ChatHistoryResult {
        r#type: String,
        content: ContentInChatHistoryResult,
    }

    #[derive(serde::Serialize)]
    struct ContentInChatHistoryFailure {
        reason: String,
    }

    #[derive(serde::Serialize)]
    pub struct ChatHistoryFailure {
        r#type: String,
        content: ContentInChatHistoryFailure,
    }

    pub fn generate_chat_history_result(
        with_username: String,
        messages: Vec<(i64, String, String, i64)>
    ) -> ChatHistoryResult {
        // Asked for up to 10 rows; if we got a full page, there might be more below it.
        let has_more: bool = messages.len() >= 10;
        ChatHistoryResult {
            r#type: String::from("chat_history_result"),
            content: ContentInChatHistoryResult {
                with_username,
                messages: messages
                    .into_iter()
                    .map(|(id, from, content, created_at)| ChatHistoryMessage { id, from, content, created_at })
                    .collect(),
                has_more,
            },
        }
    }

    pub fn generate_chat_history_failure(reason: String) -> ChatHistoryFailure {
        ChatHistoryFailure {
            r#type: String::from("chat_history_result_failure"),
            content: ContentInChatHistoryFailure { reason },
        }
    }

    pub fn generate_chat_echo_success(messages: String, created_at: i64) -> ChatEchoSuccess {
        ChatEchoSuccess {
            r#type: String::from("chat_echo"),
            content: ContentInChatEchoSuccess { status: 1, messages, created_at },
        }
    }

    pub fn generate_chat_echo_failure(reason: String) -> ChatEchoFailure {
        ChatEchoFailure {
            r#type: String::from("chat_echo"),
            content: ContentInChatEchoFailure { status: 0, reason },
        }
    }

    pub fn generate_chat_message(from: String, content: String, created_at: i64) -> ChatMessage {
        ChatMessage { r#type: String::from("chat_message"), from, content, created_at }
    }
}

async fn send_json_msg_to_ws_server(ws_id: String, msg_to_send: serde_json::Value) {
    let json_msg: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_send_msg"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnSendMsg { ws_id, msg_to_send })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_chat_server"),
    };
    send_socket_json_message(&serde_json::to_value(json_msg).unwrap(), "std_ws_server").await;
}

async fn handle_chat_history_result(
    content: ContentOnValidateSessionAndLocateResult,
    pending_request: PendingChatHistoryRequest
) {
    if !content.session_valid {
        send_json_msg_to_ws_server(
            pending_request.requester_ws_id,
            serde_json
                ::to_value(
                    msg_to_send_generator::generate_chat_history_failure(
                        String::from("invalid_session")
                    )
                )
                .unwrap()
        ).await;
        return;
    }

    let rows: Vec<ChatMessageRow> = fetch_chat_history(
        &pending_request.requester_username,
        &pending_request.with_username,
        pending_request.before_id
    ).await;

    send_json_msg_to_ws_server(
        pending_request.requester_ws_id,
        serde_json
            ::to_value(
                msg_to_send_generator::generate_chat_history_result(
                    pending_request.with_username,
                    rows
                        .into_iter()
                        .map(|row| (row.id, row.from_username, row.messages, row.created_at))
                        .collect()
                )
            )
            .unwrap()
    ).await;
}

pub async fn on_validate_session_and_locate_result(msg: SocketJsonMessage) {
    if
        let Ok(content) = serde_json::from_value::<ContentOnValidateSessionAndLocateResult>(
            msg.content
        )
    {
        let mut guard_pending_chat_history_requests: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<String, PendingChatHistoryRequest>
        > = PENDING_CHAT_HISTORY_REQUESTS.lock().await;
        if
            let Some(pending_history_request) = guard_pending_chat_history_requests.remove(
                &content.request_key
            )
        {
            drop(guard_pending_chat_history_requests);
            handle_chat_history_result(content, pending_history_request).await;
            return;
        }
        drop(guard_pending_chat_history_requests);

        let mut guard_pending_chat_requests: tokio::sync::MutexGuard<
            '_,
            std::collections::HashMap<String, PendingChatRequest>
        > = PENDING_CHAT_REQUESTS.lock().await;
        let Some(pending_request) = guard_pending_chat_requests.remove(&content.request_key) else {
            drop(guard_pending_chat_requests);
            println!(
                "{}",
                ansi_term::Color::Yellow.paint(
                    format!(
                        "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Received a validate-and-locate result for an unknown request key `{}`.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        content.request_key
                    )
                )
            );
            return;
        };
        drop(guard_pending_chat_requests);

        if !content.session_valid {
            println!(
                "{}",
                ansi_term::Color::Yellow.paint(
                    format!(
                        "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The user `{}` failed to send a chat message (invalid session token).",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        pending_request.from_username
                    )
                )
            );
            send_json_msg_to_ws_server(
                pending_request.from_ws_id,
                serde_json
                    ::to_value(
                        msg_to_send_generator::generate_chat_echo_failure(
                            String::from("invalid_session")
                        )
                    )
                    .unwrap()
            ).await;
            return;
        }

        let created_at: i64 = chrono::Utc::now().timestamp_millis();

        store_chat_message(
            &pending_request.from_username,
            &pending_request.to_username,
            &pending_request.messages,
            created_at
        ).await;

        send_json_msg_to_ws_server(
            pending_request.from_ws_id,
            serde_json
                ::to_value(
                    msg_to_send_generator::generate_chat_echo_success(
                        pending_request.messages.clone(),
                        created_at
                    )
                )
                .unwrap()
        ).await;

        if let Some(to_ws_id) = content.to_ws_id {
            send_json_msg_to_ws_server(
                to_ws_id,
                serde_json
                    ::to_value(
                        msg_to_send_generator::generate_chat_message(
                            pending_request.from_username,
                            pending_request.messages,
                            created_at
                        )
                    )
                    .unwrap()
            ).await;
        } else {
            println!(
                "{}",
                ansi_term::Color::Blue.paint(
                    format!(
                        "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] The user `{}` is not online, so the chat message from `{}` was not delivered live.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        pending_request.to_username,
                        pending_request.from_username
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
}
