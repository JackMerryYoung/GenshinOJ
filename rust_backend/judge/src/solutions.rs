use crate::global::*;

use mysql_async::prelude::*;

// Wrap a payload in the `on_send_msg` envelope and hand it to ws_server for delivery to a single
// client. Mirrors the private helpers in on_validate_session_result.rs / on_username_by_ws_id_result.rs.
pub async fn send_json_msg_to_ws_server(ws_id: String, msg_to_send: serde_json::Value) {
    let json_msg: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_send_msg"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnSendMsg { ws_id, msg_to_send })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_judge"),
    };
    send_socket_json_message(&serde_json::to_value(json_msg).unwrap(), "std_ws_server").await;
}

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
struct SolutionListItem {
    solution_id: i64,
    problem_number: i64,
    username: String,
    title: String,
    is_official: bool,
    likes: i64,
    dislikes: i64,
    created_at: i64,
    my_vote: i64,
}

#[derive(serde::Serialize)]
struct SolutionsListResult {
    r#type: String,
    content: SolutionsListResultContent,
}

#[derive(serde::Serialize)]
struct SolutionsListResultContent {
    solutions_list: Vec<SolutionListItem>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SolutionResult {
    r#type: String,
    content: SolutionResultContent,
}

#[derive(serde::Serialize)]
struct SolutionResultContent {
    solution_id: i64,
    problem_number: i64,
    username: String,
    title: String,
    content: Vec<String>,
    is_official: bool,
    likes: i64,
    dislikes: i64,
    created_at: i64,
    my_vote: i64,
    request_key: String,
}

#[derive(serde::Serialize)]
struct SolutionNotFoundResult {
    r#type: String,
    content: SolutionNotFoundContent,
}

#[derive(serde::Serialize)]
struct SolutionNotFoundContent {
    solution_id: i64,
    result: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct CommentListItem {
    comment_id: i64,
    username: String,
    content: Vec<String>,
    likes: i64,
    dislikes: i64,
    created_at: i64,
    my_vote: i64,
}

#[derive(serde::Serialize)]
struct CommentsListResult {
    r#type: String,
    content: CommentsListResultContent,
}

#[derive(serde::Serialize)]
struct CommentsListResultContent {
    comments_list: Vec<CommentListItem>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct IdResult {
    r#type: String,
    content: IdResultContent,
}

#[derive(serde::Serialize)]
struct IdResultContent {
    // One of `solution_id` / `comment_id` depending on `field`, chosen at build time.
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
// Request initiators. Both the read and write paths need simple_authenticator to
// resolve the requester's true identity first (judge can't trust the client), so
// each parks its pending work keyed by an RPC request_key and fires the lookup.
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

// Read path: stash the pending lookup, then ask simple_authenticator who owns this ws_id.
pub async fn request_username_for_lookup(
    requester_ws_id: String,
    original_request_key: String,
    lookup: PendingSolutionLookup
) {
    let rpc_request_key: String = uuid::Uuid::new_v4().to_string();
    let ws_id: String = requester_ws_id.clone();
    PENDING_SOLUTION_LOOKUP_REQUESTS.lock().await.insert(
        rpc_request_key.clone(),
        PendingSolutionLookupRequest { requester_ws_id, original_request_key, lookup }
    );

    let msg_to_send: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_username_by_ws_id"),
        content: serde_json
            ::to_value(ContentOnUsernameByWsId {
                ws_id,
                request_key: rpc_request_key.clone(),
            })
            .unwrap(),
        request_key: rpc_request_key,
        from_protocol: String::from("std_judge"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_authenticator").await;
}

// Write path: stash the pending authed action, then ask simple_authenticator to validate the session.
pub async fn request_session_for_action(
    requester_ws_id: String,
    username: String,
    session_token: String,
    original_request_key: String,
    action: PendingSolutionAuthedAction
) {
    let rpc_request_key: String = uuid::Uuid::new_v4().to_string();
    PENDING_SOLUTION_AUTHED_REQUESTS.lock().await.insert(
        rpc_request_key.clone(),
        PendingSolutionAuthedRequest {
            requester_ws_id,
            username: username.clone(),
            original_request_key,
            action,
        }
    );

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
// Read-side handlers (dispatched from on_username_by_ws_id_result once the
// requester's username is known).
// ---------------------------------------------------------------------------

pub async fn handle_lookup(
    requester_username: Option<String>,
    req: PendingSolutionLookupRequest
) {
    match req.lookup {
        PendingSolutionLookup::SolutionsList { problem_number, page_index, sort_by_likes } => {
            solutions_list(
                req.requester_ws_id,
                req.original_request_key,
                requester_username,
                problem_number,
                page_index,
                sort_by_likes
            ).await;
        }
        PendingSolutionLookup::SolutionFetch { solution_id } => {
            solution_fetch(
                req.requester_ws_id,
                req.original_request_key,
                requester_username,
                solution_id
            ).await;
        }
        PendingSolutionLookup::SolutionCommentsList { solution_id, page_index, sort_by_likes } => {
            comments_list(
                req.requester_ws_id,
                req.original_request_key,
                requester_username,
                solution_id,
                page_index,
                sort_by_likes
            ).await;
        }
    }
}

async fn solutions_list(
    requester_ws_id: String,
    original_request_key: String,
    requester_username: Option<String>,
    problem_number: i64,
    page_index: i64,
    sort_by_likes: bool
) {
    let requester: String = requester_username.unwrap_or_default();
    let page_index: i64 = std::cmp::max(1, page_index);
    let offset: i64 = (page_index - 1) * SOLUTIONS_LIST_PAGE_SIZE;
    // Official ("starred") solutions are pinned to the top regardless of sort; the chosen sort
    // orders the rest. The sort column is a fixed value chosen here, never client text.
    let sort_column: &str = if sort_by_likes { "s.likes" } else { "s.created_at" };

    let guard_pool = MYSQL_DATABASE_POOL.lock().await;
    let mut conn: mysql_async::Conn = guard_pool.get_conn().await.unwrap();
    let results: Result<Vec<(i64, i64, String, String, bool, i64, i64, i64, i64)>, _> = conn
        .exec(
            format!(
                "SELECT s.solution_id, s.problem_number, s.username, s.title, s.is_official, s.likes, s.dislikes, s.created_at, COALESCE(v.vote, 0) AS my_vote
                FROM RsOJ.solutions s
                LEFT JOIN RsOJ.solution_votes v ON v.solution_id = s.solution_id AND v.username = :requester
                WHERE s.problem_number = :problem_number
                ORDER BY s.is_official DESC, {sort_column} DESC, s.solution_id DESC
                LIMIT :limit OFFSET :offset"
            ),
            mysql_async::params! {
                "requester" => &requester,
                "problem_number" => problem_number,
                "limit" => SOLUTIONS_LIST_PAGE_SIZE,
                "offset" => offset,
            }
        )
        .await;
    drop(conn);
    drop(guard_pool);

    let rows = match results {
        Ok(rows) => rows,
        Err(e) => {
            warn(line!(), &format!("Failed to fetch the solutions list: {e}"));
            return;
        }
    };

    let solutions_list: Vec<SolutionListItem> = rows
        .into_iter()
        .map(
            |
                (
                    solution_id,
                    problem_number,
                    username,
                    title,
                    is_official,
                    likes,
                    dislikes,
                    created_at,
                    my_vote,
                )
            | SolutionListItem {
                solution_id,
                problem_number,
                username,
                title,
                is_official,
                likes,
                dislikes,
                created_at,
                my_vote,
            }
        )
        .collect();

    let result = SolutionsListResult {
        r#type: String::from("solutions_list"),
        content: SolutionsListResultContent {
            solutions_list,
            request_key: original_request_key,
        },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
}

async fn solution_fetch(
    requester_ws_id: String,
    original_request_key: String,
    requester_username: Option<String>,
    solution_id: i64
) {
    let requester: String = requester_username.unwrap_or_default();

    let guard_pool = MYSQL_DATABASE_POOL.lock().await;
    let mut conn: mysql_async::Conn = guard_pool.get_conn().await.unwrap();
    let row_result: Result<Option<(i64, String, String, String, bool, i64, i64, i64)>, _> = conn
        .exec_first(
            "SELECT problem_number, username, title, content, is_official, likes, dislikes, created_at
            FROM RsOJ.solutions
            WHERE solution_id = :solution_id",
            mysql_async::params! { "solution_id" => solution_id }
        )
        .await;
    let my_vote: i64 = conn
        .exec_first::<i64, _, _>(
            "SELECT vote FROM RsOJ.solution_votes WHERE solution_id = :solution_id AND username = :requester",
            mysql_async::params! { "solution_id" => solution_id, "requester" => &requester }
        )
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    drop(conn);
    drop(guard_pool);

    match row_result {
        Ok(Some((problem_number, username, title, content, is_official, likes, dislikes, created_at))) => {
            let result = SolutionResult {
                r#type: String::from("solution"),
                content: SolutionResultContent {
                    solution_id,
                    problem_number,
                    username,
                    title,
                    content: serde_json::from_str(&content).unwrap_or_default(),
                    is_official,
                    likes,
                    dislikes,
                    created_at,
                    my_vote,
                    request_key: original_request_key,
                },
            };
            send_json_msg_to_ws_server(
                requester_ws_id,
                serde_json::to_value(result).unwrap()
            ).await;
        }
        Ok(None) => {
            let result = SolutionNotFoundResult {
                r#type: String::from("solution"),
                content: SolutionNotFoundContent {
                    solution_id,
                    result: String::from("SNF"),
                    request_key: original_request_key,
                },
            };
            send_json_msg_to_ws_server(
                requester_ws_id,
                serde_json::to_value(result).unwrap()
            ).await;
        }
        Err(e) => {
            warn(line!(), &format!("Failed to fetch the solution `{solution_id}`: {e}"));
        }
    }
}

async fn comments_list(
    requester_ws_id: String,
    original_request_key: String,
    requester_username: Option<String>,
    solution_id: i64,
    page_index: i64,
    sort_by_likes: bool
) {
    let requester: String = requester_username.unwrap_or_default();
    let page_index: i64 = std::cmp::max(1, page_index);
    let offset: i64 = (page_index - 1) * SOLUTION_COMMENTS_LIST_PAGE_SIZE;
    let sort_column: &str = if sort_by_likes { "c.likes" } else { "c.created_at" };

    let guard_pool = MYSQL_DATABASE_POOL.lock().await;
    let mut conn: mysql_async::Conn = guard_pool.get_conn().await.unwrap();
    let results: Result<Vec<(i64, String, String, i64, i64, i64, i64)>, _> = conn
        .exec(
            format!(
                "SELECT c.comment_id, c.username, c.content, c.likes, c.dislikes, c.created_at, COALESCE(v.vote, 0) AS my_vote
                FROM RsOJ.solution_comments c
                LEFT JOIN RsOJ.solution_comment_votes v ON v.comment_id = c.comment_id AND v.username = :requester
                WHERE c.solution_id = :solution_id
                ORDER BY {sort_column} DESC, c.comment_id DESC
                LIMIT :limit OFFSET :offset"
            ),
            mysql_async::params! {
                "requester" => &requester,
                "solution_id" => solution_id,
                "limit" => SOLUTION_COMMENTS_LIST_PAGE_SIZE,
                "offset" => offset,
            }
        )
        .await;
    drop(conn);
    drop(guard_pool);

    let rows = match results {
        Ok(rows) => rows,
        Err(e) => {
            warn(line!(), &format!("Failed to fetch the comments list: {e}"));
            return;
        }
    };

    let comments_list: Vec<CommentListItem> = rows
        .into_iter()
        .map(|(comment_id, username, content, likes, dislikes, created_at, my_vote)| CommentListItem {
            comment_id,
            username,
            content: serde_json::from_str(&content).unwrap_or_default(),
            likes,
            dislikes,
            created_at,
            my_vote,
        })
        .collect();

    let result = CommentsListResult {
        r#type: String::from("solution_comments_list"),
        content: CommentsListResultContent {
            comments_list,
            request_key: original_request_key,
        },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
}

// ---------------------------------------------------------------------------
// Write-side handlers (dispatched from on_validate_session_result once the
// requester's session has been confirmed valid).
// ---------------------------------------------------------------------------

pub async fn handle_authed_action(req: PendingSolutionAuthedRequest) {
    let requester_ws_id: String = req.requester_ws_id;
    let username: String = req.username;
    let original_request_key: String = req.original_request_key;
    match req.action {
        PendingSolutionAuthedAction::PostSolution { problem_number, title, content, is_official } => {
            post_solution(
                requester_ws_id,
                original_request_key,
                username,
                problem_number,
                title,
                content,
                is_official
            ).await;
        }
        PendingSolutionAuthedAction::VoteSolution { solution_id, vote } => {
            vote_solution(requester_ws_id, original_request_key, username, solution_id, vote).await;
        }
        PendingSolutionAuthedAction::PostComment { solution_id, content } => {
            post_comment(requester_ws_id, original_request_key, username, solution_id, content).await;
        }
        PendingSolutionAuthedAction::VoteComment { comment_id, vote } => {
            vote_comment(requester_ws_id, original_request_key, username, comment_id, vote).await;
        }
    }
}

// Tell the requester their write was rejected because the session didn't validate. The result
// `type` must match the variant so the right frontend handler picks it up by request_key.
pub async fn handle_invalid_session(req: PendingSolutionAuthedRequest) {
    let type_name: &str = match req.action {
        PendingSolutionAuthedAction::PostSolution { .. } => "solution_post_failure",
        PendingSolutionAuthedAction::VoteSolution { .. } => "solution_vote_result",
        PendingSolutionAuthedAction::PostComment { .. } => "solution_comment_post_failure",
        PendingSolutionAuthedAction::VoteComment { .. } => "solution_comment_vote_result",
    };
    send_failure(req.requester_ws_id, type_name, "invalid_session", req.original_request_key).await;
}

async fn send_failure(ws_id: String, type_name: &str, reason: &str, request_key: String) {
    let result = FailureResult {
        r#type: String::from(type_name),
        content: FailureResultContent {
            reason: String::from(reason),
            request_key,
        },
    };
    send_json_msg_to_ws_server(ws_id, serde_json::to_value(result).unwrap()).await;
}

async fn post_solution(
    requester_ws_id: String,
    original_request_key: String,
    username: String,
    problem_number: i64,
    title: String,
    content: Vec<String>,
    is_official: bool
) {
    if title.trim().is_empty() || content.iter().all(|line| line.trim().is_empty()) {
        send_failure(requester_ws_id, "solution_post_failure", "empty", original_request_key).await;
        return;
    }

    let guard_pool = MYSQL_DATABASE_POOL.lock().await;
    let mut conn: mysql_async::Conn = guard_pool.get_conn().await.unwrap();
    let insert_result = conn
        .exec_drop(
            "INSERT INTO RsOJ.solutions
            (problem_number, username, title, content, is_official, likes, dislikes, created_at)
            VALUES (:problem_number, :username, :title, :content, :is_official, 0, 0, :created_at)",
            mysql_async::params! {
                "problem_number" => problem_number,
                "username" => &username,
                "title" => &title,
                "content" => serde_json::to_string(&content).unwrap(),
                "is_official" => is_official,
                "created_at" => chrono::Utc::now().timestamp(),
            }
        )
        .await;
    let solution_id: i64 = conn.last_insert_id().unwrap_or(0) as i64;
    drop(conn);
    drop(guard_pool);

    if insert_result.is_err() {
        warn(line!(), "Failed to insert the solution row.");
        send_failure(requester_ws_id, "solution_post_failure", "internal_error", original_request_key).await;
        return;
    }

    let mut id: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    id.insert(String::from("solution_id"), solution_id);
    let result = IdResult {
        r#type: String::from("solution_id"),
        content: IdResultContent { id, request_key: original_request_key },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;

    // A top-level solution has no parent author, so notifications come only from @mentions.
    crate::notifications::create_post_notifications(
        &username,
        &content,
        None,
        "solution",
        &format!("/solution/{solution_id}")
    ).await;
}

async fn post_comment(
    requester_ws_id: String,
    original_request_key: String,
    username: String,
    solution_id: i64,
    content: Vec<String>
) {
    if content.iter().all(|line| line.trim().is_empty()) {
        send_failure(requester_ws_id, "solution_comment_post_failure", "empty", original_request_key).await;
        return;
    }

    let guard_pool = MYSQL_DATABASE_POOL.lock().await;
    let mut conn: mysql_async::Conn = guard_pool.get_conn().await.unwrap();
    // Guard against commenting on a non-existent solution; also grab the author so we can notify them.
    let solution_owner: Option<String> = conn
        .exec_first(
            "SELECT username FROM RsOJ.solutions WHERE solution_id = :solution_id",
            mysql_async::params! { "solution_id" => solution_id }
        )
        .await
        .ok()
        .flatten();
    let Some(solution_owner) = solution_owner else {
        drop(conn);
        drop(guard_pool);
        send_failure(requester_ws_id, "solution_comment_post_failure", "solution_not_found", original_request_key).await;
        return;
    };

    let insert_result = conn
        .exec_drop(
            "INSERT INTO RsOJ.solution_comments
            (solution_id, username, content, likes, dislikes, created_at)
            VALUES (:solution_id, :username, :content, 0, 0, :created_at)",
            mysql_async::params! {
                "solution_id" => solution_id,
                "username" => &username,
                "content" => serde_json::to_string(&content).unwrap(),
                "created_at" => chrono::Utc::now().timestamp(),
            }
        )
        .await;
    let comment_id: i64 = conn.last_insert_id().unwrap_or(0) as i64;
    drop(conn);
    drop(guard_pool);

    if insert_result.is_err() {
        warn(line!(), "Failed to insert the solution comment row.");
        send_failure(requester_ws_id, "solution_comment_post_failure", "internal_error", original_request_key).await;
        return;
    }

    let mut id: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    id.insert(String::from("comment_id"), comment_id);
    let result = IdResult {
        r#type: String::from("comment_id"),
        content: IdResultContent { id, request_key: original_request_key },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;

    // Notify the solution's author (a 'reply'), plus anyone @mentioned in the comment body.
    crate::notifications::create_post_notifications(
        &username,
        &content,
        Some(&solution_owner),
        "solution_comment",
        &format!("/solution/{solution_id}")
    ).await;
}

// Apply a like/dislike/clear against a votes table, keeping the denormalized `likes`/`dislikes`
// totals on the parent row in sync. `target_table`/`votes_table`/`id_column` are fixed identifiers
// chosen by the caller (never client text). Returns (likes, dislikes, my_vote) after the change, or
// None if the target row doesn't exist / a query failed.
pub async fn apply_vote(
    target_table: &str,
    votes_table: &str,
    id_column: &str,
    username: &str,
    target_id: i64,
    // 1 = like, -1 = dislike, anything else = clear.
    vote: i8
) -> Option<(i64, i64, i64)> {
    let new_vote: i64 = match vote {
        1 => 1,
        -1 => -1,
        _ => 0,
    };

    let guard_pool = MYSQL_DATABASE_POOL.lock().await;
    let mut conn: mysql_async::Conn = guard_pool.get_conn().await.unwrap();

    // The target must exist before we touch any totals.
    let exists: Option<i64> = conn
        .exec_first(
            format!("SELECT {id_column} FROM RsOJ.{target_table} WHERE {id_column} = :id"),
            mysql_async::params! { "id" => target_id }
        )
        .await
        .ok()
        .flatten();
    exists?;

    let old_vote: i64 = conn
        .exec_first::<i64, _, _>(
            format!("SELECT vote FROM RsOJ.{votes_table} WHERE {id_column} = :id AND username = :username"),
            mysql_async::params! { "id" => target_id, "username" => username }
        )
        .await
        .ok()
        .flatten()
        .unwrap_or(0);

    // No change — just report the current totals.
    if old_vote == new_vote {
        let totals: Option<(i64, i64)> = conn
            .exec_first(
                format!("SELECT likes, dislikes FROM RsOJ.{target_table} WHERE {id_column} = :id"),
                mysql_async::params! { "id" => target_id }
            )
            .await
            .ok()
            .flatten();
        drop(conn);
        drop(guard_pool);
        return totals.map(|(likes, dislikes)| (likes, dislikes, new_vote));
    }

    // Persist the per-user vote.
    if new_vote == 0 {
        let _ = conn
            .exec_drop(
                format!("DELETE FROM RsOJ.{votes_table} WHERE {id_column} = :id AND username = :username"),
                mysql_async::params! { "id" => target_id, "username" => username }
            )
            .await;
    } else {
        let _ = conn
            .exec_drop(
                format!(
                    "INSERT INTO RsOJ.{votes_table} ({id_column}, username, vote)
                    VALUES (:id, :username, :vote)
                    ON DUPLICATE KEY UPDATE vote = :vote"
                ),
                mysql_async::params! { "id" => target_id, "username" => username, "vote" => new_vote }
            )
            .await;
    }

    // Move the denormalized totals by the difference between the old and new vote.
    let likes_delta: i64 = ((new_vote == 1) as i64) - ((old_vote == 1) as i64);
    let dislikes_delta: i64 = ((new_vote == -1) as i64) - ((old_vote == -1) as i64);
    let _ = conn
        .exec_drop(
            format!(
                "UPDATE RsOJ.{target_table}
                SET likes = likes + :likes_delta, dislikes = dislikes + :dislikes_delta
                WHERE {id_column} = :id"
            ),
            mysql_async::params! {
                "likes_delta" => likes_delta,
                "dislikes_delta" => dislikes_delta,
                "id" => target_id,
            }
        )
        .await;

    let totals: Option<(i64, i64)> = conn
        .exec_first(
            format!("SELECT likes, dislikes FROM RsOJ.{target_table} WHERE {id_column} = :id"),
            mysql_async::params! { "id" => target_id }
        )
        .await
        .ok()
        .flatten();
    drop(conn);
    drop(guard_pool);

    totals.map(|(likes, dislikes)| (likes, dislikes, new_vote))
}

async fn vote_solution(
    requester_ws_id: String,
    original_request_key: String,
    username: String,
    solution_id: i64,
    vote: i8
) {
    match apply_vote("solutions", "solution_votes", "solution_id", &username, solution_id, vote).await {
        Some((likes, dislikes, my_vote)) => {
            let mut id: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            id.insert(String::from("solution_id"), solution_id);
            let result = VoteResult {
                r#type: String::from("solution_vote_result"),
                content: VoteResultContent { id, likes, dislikes, my_vote, request_key: original_request_key },
            };
            send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
        }
        None => {
            send_failure(requester_ws_id, "solution_vote_result", "solution_not_found", original_request_key).await;
        }
    }
}

async fn vote_comment(
    requester_ws_id: String,
    original_request_key: String,
    username: String,
    comment_id: i64,
    vote: i8
) {
    match apply_vote("solution_comments", "solution_comment_votes", "comment_id", &username, comment_id, vote).await {
        Some((likes, dislikes, my_vote)) => {
            let mut id: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            id.insert(String::from("comment_id"), comment_id);
            let result = VoteResult {
                r#type: String::from("solution_comment_vote_result"),
                content: VoteResultContent { id, likes, dislikes, my_vote, request_key: original_request_key },
            };
            send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
        }
        None => {
            send_failure(requester_ws_id, "solution_comment_vote_result", "comment_not_found", original_request_key).await;
        }
    }
}
