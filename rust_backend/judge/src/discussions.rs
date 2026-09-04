use crate::global::*;
use crate::solutions::{ apply_vote, send_json_msg_to_ws_server };

use mysql_async::prelude::*;

fn warn(line: u32, message: &str) {
    println!(
        "{}",
        ansi_term::Color::Yellow.paint(
            format!(
                "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] {}",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line,
                message
            )
        )
    );
}

// ---------------------------------------------------------------------------
// Serialized payloads sent back to the frontend.
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
struct DiscussionListItem {
    discussion_id: i64,
    username: String,
    title: String,
    likes: i64,
    dislikes: i64,
    created_at: i64,
    my_vote: i64,
}

#[derive(serde::Serialize)]
struct DiscussionsListResult {
    r#type: String,
    content: DiscussionsListResultContent,
}

#[derive(serde::Serialize)]
struct DiscussionsListResultContent {
    discussions_list: Vec<DiscussionListItem>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct DiscussionResult {
    r#type: String,
    content: DiscussionResultContent,
}

#[derive(serde::Serialize)]
struct DiscussionResultContent {
    discussion_id: i64,
    username: String,
    title: String,
    content: Vec<String>,
    likes: i64,
    dislikes: i64,
    created_at: i64,
    my_vote: i64,
    request_key: String,
}

#[derive(serde::Serialize)]
struct DiscussionNotFoundResult {
    r#type: String,
    content: DiscussionNotFoundContent,
}

#[derive(serde::Serialize)]
struct DiscussionNotFoundContent {
    discussion_id: i64,
    result: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct ReplyListItem {
    reply_id: i64,
    username: String,
    content: Vec<String>,
    likes: i64,
    dislikes: i64,
    created_at: i64,
    my_vote: i64,
}

#[derive(serde::Serialize)]
struct RepliesListResult {
    r#type: String,
    content: RepliesListResultContent,
}

#[derive(serde::Serialize)]
struct RepliesListResultContent {
    replies_list: Vec<ReplyListItem>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct IdResult {
    r#type: String,
    content: IdResultContent,
}

#[derive(serde::Serialize)]
struct IdResultContent {
    #[serde(flatten)]
    id: std::collections::HashMap<String, i64>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct FailureResult {
    r#type: String,
    content: FailureResultContent,
}

#[derive(serde::Serialize)]
struct FailureResultContent {
    reason: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct VoteResult {
    r#type: String,
    content: VoteResultContent,
}

#[derive(serde::Serialize)]
struct VoteResultContent {
    #[serde(flatten)]
    id: std::collections::HashMap<String, i64>,
    likes: i64,
    dislikes: i64,
    my_vote: i64,
    request_key: String,
}

// ---------------------------------------------------------------------------
// Request initiators (resolve the requester's identity via simple_authenticator first).
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
struct ContentOnUsernameByWsId {
    ws_id: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct ContentOnValidateSession {
    username: String,
    session_token: String,
    request_key: String,
}

pub async fn request_username_for_lookup(
    requester_ws_id: String,
    original_request_key: String,
    lookup: PendingDiscussionLookup
) {
    let rpc_request_key: String = uuid::Uuid::new_v4().to_string();
    let ws_id: String = requester_ws_id.clone();
    PENDING_DISCUSSION_LOOKUP_REQUESTS.lock().await.insert(
        rpc_request_key.clone(),
        PendingDiscussionLookupRequest { requester_ws_id, original_request_key, lookup }
    );
    expire_pending(&*PENDING_DISCUSSION_LOOKUP_REQUESTS, rpc_request_key.clone());

    let msg_to_send: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_username_by_ws_id"),
        content: serde_json
            ::to_value(ContentOnUsernameByWsId { ws_id, request_key: rpc_request_key.clone() })
            .unwrap(),
        request_key: rpc_request_key,
        from_protocol: String::from("std_judge"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_authenticator").await;
}

pub async fn request_session_for_action(
    requester_ws_id: String,
    username: String,
    session_token: String,
    original_request_key: String,
    action: PendingDiscussionAuthedAction
) {
    let rpc_request_key: String = uuid::Uuid::new_v4().to_string();
    PENDING_DISCUSSION_AUTHED_REQUESTS.lock().await.insert(
        rpc_request_key.clone(),
        PendingDiscussionAuthedRequest {
            requester_ws_id,
            username: username.clone(),
            original_request_key,
            action,
        }
    );
    expire_pending(&*PENDING_DISCUSSION_AUTHED_REQUESTS, rpc_request_key.clone());

    let msg_to_send: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_validate_session"),
        content: serde_json
            ::to_value(ContentOnValidateSession {
                username,
                session_token,
                request_key: rpc_request_key,
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_judge"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_authenticator").await;
}

// ---------------------------------------------------------------------------
// Read-side handlers (dispatched from on_username_by_ws_id_result).
// ---------------------------------------------------------------------------

pub async fn handle_lookup(
    requester_username: Option<String>,
    req: PendingDiscussionLookupRequest
) {
    match req.lookup {
        PendingDiscussionLookup::DiscussionsList { page_index, sort_by_likes } => {
            discussions_list(
                req.requester_ws_id,
                req.original_request_key,
                requester_username,
                page_index,
                sort_by_likes
            ).await;
        }
        PendingDiscussionLookup::DiscussionFetch { discussion_id } => {
            discussion_fetch(
                req.requester_ws_id,
                req.original_request_key,
                requester_username,
                discussion_id
            ).await;
        }
        PendingDiscussionLookup::DiscussionRepliesList { discussion_id, page_index, sort_by_likes } => {
            replies_list(
                req.requester_ws_id,
                req.original_request_key,
                requester_username,
                discussion_id,
                page_index,
                sort_by_likes
            ).await;
        }
    }
}

async fn discussions_list(
    requester_ws_id: String,
    original_request_key: String,
    requester_username: Option<String>,
    page_index: i64,
    sort_by_likes: bool
) {
    let requester: String = requester_username.unwrap_or_default();
    let page_index: i64 = std::cmp::max(1, page_index);
    let offset: i64 = (page_index - 1) * DISCUSSIONS_LIST_PAGE_SIZE;
    let sort_column: &str = if sort_by_likes { "d.likes" } else { "d.created_at" };

    let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
    type DiscussionListType = Result<Vec<(i64, String, String, i64, i64, i64, i64)>, mysql_async::Error>;
    let results: DiscussionListType = conn
        .exec(
            format!(
                "SELECT d.discussion_id, d.username, d.title, d.likes, d.dislikes, d.created_at, COALESCE(v.vote, 0) AS my_vote
                FROM RsOJ.discussions d
                LEFT JOIN RsOJ.discussion_votes v ON v.discussion_id = d.discussion_id AND v.username = :requester
                ORDER BY {sort_column} DESC, d.discussion_id DESC
                LIMIT :limit OFFSET :offset"
            ),
            mysql_async::params! {
                "requester" => &requester,
                "limit" => DISCUSSIONS_LIST_PAGE_SIZE,
                "offset" => offset,
            }
        )
        .await;
    drop(conn);

    let rows = match results {
        Ok(rows) => rows,
        Err(e) => {
            warn(line!(), &format!("Failed to fetch the discussions list: {e}"));
            return;
        }
    };

    let discussions_list: Vec<DiscussionListItem> = rows
        .into_iter()
        .map(|(discussion_id, username, title, likes, dislikes, created_at, my_vote)| DiscussionListItem {
            discussion_id,
            username,
            title,
            likes,
            dislikes,
            created_at,
            my_vote,
        })
        .collect();

    let result = DiscussionsListResult {
        r#type: String::from("discussions_list"),
        content: DiscussionsListResultContent { discussions_list, request_key: original_request_key },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
}

async fn discussion_fetch(
    requester_ws_id: String,
    original_request_key: String,
    requester_username: Option<String>,
    discussion_id: i64
) {
    let requester: String = requester_username.unwrap_or_default();

    let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
    type DiscussionFetchResult = Result<Option<(String, String, String, i64, i64, i64)>, mysql_async::Error>;
    let row_result: DiscussionFetchResult = conn
        .exec_first(
            "SELECT username, title, content, likes, dislikes, created_at
            FROM RsOJ.discussions
            WHERE discussion_id = :discussion_id",
            mysql_async::params! { "discussion_id" => discussion_id }
        )
        .await;
    let my_vote: i64 = conn
        .exec_first::<i64, _, _>(
            "SELECT vote FROM RsOJ.discussion_votes WHERE discussion_id = :discussion_id AND username = :requester",
            mysql_async::params! { "discussion_id" => discussion_id, "requester" => &requester }
        )
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    drop(conn);

    match row_result {
        Ok(Some((username, title, content, likes, dislikes, created_at))) => {
            let result = DiscussionResult {
                r#type: String::from("discussion"),
                content: DiscussionResultContent {
                    discussion_id,
                    username,
                    title,
                    content: serde_json::from_str(&content).unwrap_or_default(),
                    likes,
                    dislikes,
                    created_at,
                    my_vote,
                    request_key: original_request_key,
                },
            };
            send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
        }
        Ok(None) => {
            let result = DiscussionNotFoundResult {
                r#type: String::from("discussion"),
                content: DiscussionNotFoundContent {
                    discussion_id,
                    result: String::from("DNF"),
                    request_key: original_request_key,
                },
            };
            send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
        }
        Err(e) => {
            warn(line!(), &format!("Failed to fetch the discussion `{discussion_id}`: {e}"));
        }
    }
}

async fn replies_list(
    requester_ws_id: String,
    original_request_key: String,
    requester_username: Option<String>,
    discussion_id: i64,
    page_index: i64,
    sort_by_likes: bool
) {
    let requester: String = requester_username.unwrap_or_default();
    let page_index: i64 = std::cmp::max(1, page_index);
    let offset: i64 = (page_index - 1) * DISCUSSION_REPLIES_LIST_PAGE_SIZE;
    let sort_column: &str = if sort_by_likes { "r.likes" } else { "r.created_at" };

    let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
    type RepliesListResultType = Result<Vec<(i64, String, String, i64, i64, i64, i64)>, mysql_async::Error>;
    let results: RepliesListResultType = conn
        .exec(
            format!(
                "SELECT r.reply_id, r.username, r.content, r.likes, r.dislikes, r.created_at, COALESCE(v.vote, 0) AS my_vote
                FROM RsOJ.discussion_replies r
                LEFT JOIN RsOJ.discussion_reply_votes v ON v.reply_id = r.reply_id AND v.username = :requester
                WHERE r.discussion_id = :discussion_id
                ORDER BY {sort_column} DESC, r.reply_id DESC
                LIMIT :limit OFFSET :offset"
            ),
            mysql_async::params! {
                "requester" => &requester,
                "discussion_id" => discussion_id,
                "limit" => DISCUSSION_REPLIES_LIST_PAGE_SIZE,
                "offset" => offset,
            }
        )
        .await;
    drop(conn);

    let rows = match results {
        Ok(rows) => rows,
        Err(e) => {
            warn(line!(), &format!("Failed to fetch the replies list: {e}"));
            return;
        }
    };

    let replies_list: Vec<ReplyListItem> = rows
        .into_iter()
        .map(|(reply_id, username, content, likes, dislikes, created_at, my_vote)| ReplyListItem {
            reply_id,
            username,
            content: serde_json::from_str(&content).unwrap_or_default(),
            likes,
            dislikes,
            created_at,
            my_vote,
        })
        .collect();

    let result = RepliesListResult {
        r#type: String::from("discussion_replies_list"),
        content: RepliesListResultContent { replies_list, request_key: original_request_key },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
}

// ---------------------------------------------------------------------------
// Write-side handlers (dispatched from on_validate_session_result).
// ---------------------------------------------------------------------------

pub async fn handle_authed_action(req: PendingDiscussionAuthedRequest) {
    let requester_ws_id: String = req.requester_ws_id;
    let username: String = req.username;
    let original_request_key: String = req.original_request_key;
    match req.action {
        PendingDiscussionAuthedAction::PostDiscussion { title, content } => {
            post_discussion(requester_ws_id, original_request_key, username, title, content).await;
        }
        PendingDiscussionAuthedAction::VoteDiscussion { discussion_id, vote } => {
            vote_discussion(requester_ws_id, original_request_key, username, discussion_id, vote).await;
        }
        PendingDiscussionAuthedAction::PostReply { discussion_id, content } => {
            post_reply(requester_ws_id, original_request_key, username, discussion_id, content).await;
        }
        PendingDiscussionAuthedAction::VoteReply { reply_id, vote } => {
            vote_reply(requester_ws_id, original_request_key, username, reply_id, vote).await;
        }
    }
}

pub async fn handle_invalid_session(req: PendingDiscussionAuthedRequest) {
    let type_name: &str = match req.action {
        PendingDiscussionAuthedAction::PostDiscussion { .. } => "discussion_post_failure",
        PendingDiscussionAuthedAction::VoteDiscussion { .. } => "discussion_vote_result",
        PendingDiscussionAuthedAction::PostReply { .. } => "discussion_reply_post_failure",
        PendingDiscussionAuthedAction::VoteReply { .. } => "discussion_reply_vote_result",
    };
    send_failure(req.requester_ws_id, type_name, "invalid_session", req.original_request_key).await;
}

async fn send_failure(ws_id: String, type_name: &str, reason: &str, request_key: String) {
    let result = FailureResult {
        r#type: String::from(type_name),
        content: FailureResultContent { reason: String::from(reason), request_key },
    };
    send_json_msg_to_ws_server(ws_id, serde_json::to_value(result).unwrap()).await;
}

async fn post_discussion(
    requester_ws_id: String,
    original_request_key: String,
    username: String,
    title: String,
    content: Vec<String>
) {
    if title.trim().is_empty() || content.iter().all(|line| line.trim().is_empty()) {
        send_failure(requester_ws_id, "discussion_post_failure", "empty", original_request_key).await;
        return;
    }

    let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
    let insert_result = conn
        .exec_drop(
            "INSERT INTO RsOJ.discussions
            (username, title, content, likes, dislikes, created_at)
            VALUES (:username, :title, :content, 0, 0, :created_at)",
            mysql_async::params! {
                "username" => &username,
                "title" => &title,
                "content" => serde_json::to_string(&content).unwrap(),
                "created_at" => chrono::Utc::now().timestamp(),
            }
        )
        .await;
    let discussion_id: i64 = conn.last_insert_id().unwrap_or(0) as i64;
    drop(conn);

    if insert_result.is_err() {
        warn(line!(), "Failed to insert the discussion row.");
        send_failure(requester_ws_id, "discussion_post_failure", "internal_error", original_request_key).await;
        return;
    }

    let mut id: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    id.insert(String::from("discussion_id"), discussion_id);
    let result = IdResult {
        r#type: String::from("discussion_id"),
        content: IdResultContent { id, request_key: original_request_key },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;

    // A top-level thread has no parent author, so notifications come only from @mentions.
    crate::notifications::create_post_notifications(
        &username,
        &content,
        None,
        "discussion",
        &format!("/discussion/{discussion_id}")
    ).await;
}

async fn post_reply(
    requester_ws_id: String,
    original_request_key: String,
    username: String,
    discussion_id: i64,
    content: Vec<String>
) {
    if content.iter().all(|line| line.trim().is_empty()) {
        send_failure(requester_ws_id, "discussion_reply_post_failure", "empty", original_request_key).await;
        return;
    }

    let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
    let discussion_owner: Option<String> = conn
        .exec_first(
            "SELECT username FROM RsOJ.discussions WHERE discussion_id = :discussion_id",
            mysql_async::params! { "discussion_id" => discussion_id }
        )
        .await
        .ok()
        .flatten();
    let Some(discussion_owner) = discussion_owner else {
        drop(conn);
        send_failure(requester_ws_id, "discussion_reply_post_failure", "discussion_not_found", original_request_key).await;
        return;
    };

    let insert_result = conn
        .exec_drop(
            "INSERT INTO RsOJ.discussion_replies
            (discussion_id, username, content, likes, dislikes, created_at)
            VALUES (:discussion_id, :username, :content, 0, 0, :created_at)",
            mysql_async::params! {
                "discussion_id" => discussion_id,
                "username" => &username,
                "content" => serde_json::to_string(&content).unwrap(),
                "created_at" => chrono::Utc::now().timestamp(),
            }
        )
        .await;
    let reply_id: i64 = conn.last_insert_id().unwrap_or(0) as i64;
    drop(conn);

    if insert_result.is_err() {
        warn(line!(), "Failed to insert the discussion reply row.");
        send_failure(requester_ws_id, "discussion_reply_post_failure", "internal_error", original_request_key).await;
        return;
    }

    let mut id: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    id.insert(String::from("reply_id"), reply_id);
    let result = IdResult {
        r#type: String::from("reply_id"),
        content: IdResultContent { id, request_key: original_request_key },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;

    // Notify the thread's author (a 'reply'), plus anyone @mentioned in the reply body.
    crate::notifications::create_post_notifications(
        &username,
        &content,
        Some(&discussion_owner),
        "discussion_reply",
        &format!("/discussion/{discussion_id}")
    ).await;
}

async fn vote_discussion(
    requester_ws_id: String,
    original_request_key: String,
    username: String,
    discussion_id: i64,
    vote: i8
) {
    match apply_vote("discussions", "discussion_votes", "discussion_id", &username, discussion_id, vote).await {
        Some((likes, dislikes, my_vote)) => {
            let mut id: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            id.insert(String::from("discussion_id"), discussion_id);
            let result = VoteResult {
                r#type: String::from("discussion_vote_result"),
                content: VoteResultContent { id, likes, dislikes, my_vote, request_key: original_request_key },
            };
            send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
        }
        None => {
            send_failure(requester_ws_id, "discussion_vote_result", "discussion_not_found", original_request_key).await;
        }
    }
}

async fn vote_reply(
    requester_ws_id: String,
    original_request_key: String,
    username: String,
    reply_id: i64,
    vote: i8
) {
    match apply_vote("discussion_replies", "discussion_reply_votes", "reply_id", &username, reply_id, vote).await {
        Some((likes, dislikes, my_vote)) => {
            let mut id: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            id.insert(String::from("reply_id"), reply_id);
            let result = VoteResult {
                r#type: String::from("discussion_reply_vote_result"),
                content: VoteResultContent { id, likes, dislikes, my_vote, request_key: original_request_key },
            };
            send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
        }
        None => {
            send_failure(requester_ws_id, "discussion_reply_vote_result", "reply_not_found", original_request_key).await;
        }
    }
}
