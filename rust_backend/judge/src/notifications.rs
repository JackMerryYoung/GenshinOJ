use crate::global::*;
use crate::solutions::send_json_msg_to_ws_server;

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
struct NotificationListItem {
    notification_id: i64,
    actor: String,
    kind: String,
    source_type: String,
    target_url: String,
    excerpt: String,
    is_read: bool,
    created_at: i64,
}

#[derive(serde::Serialize)]
struct NotificationsListResult {
    r#type: String,
    content: NotificationsListResultContent,
}

#[derive(serde::Serialize)]
struct NotificationsListResultContent {
    notifications_list: Vec<NotificationListItem>,
    request_key: String,
}

#[derive(serde::Serialize)]
struct TotalIndexResult {
    r#type: String,
    content: TotalIndexResultContent,
}

#[derive(serde::Serialize)]
struct TotalIndexResultContent {
    total_notifications_list_index: i64,
    request_key: String,
}

#[derive(serde::Serialize)]
struct UnreadCountResult {
    r#type: String,
    content: UnreadCountResultContent,
}

#[derive(serde::Serialize)]
struct UnreadCountResultContent {
    unread_count: i64,
    request_key: String,
}

#[derive(serde::Serialize)]
struct MarkReadResult {
    r#type: String,
    content: MarkReadResultContent,
}

#[derive(serde::Serialize)]
struct MarkReadResultContent {
    result: String,
    request_key: String,
}

#[derive(serde::Serialize)]
struct UserSearchResult {
    r#type: String,
    content: UserSearchResultContent,
}

#[derive(serde::Serialize)]
struct UserSearchResultContent {
    usernames: Vec<String>,
    request_key: String,
}

// ---------------------------------------------------------------------------
// @mention parsing + notification creation. Called by the post handlers in
// solutions.rs / discussions.rs after a row is successfully inserted.
// ---------------------------------------------------------------------------

// Pull out every distinct `@name` token (name = ASCII alphanumerics / underscore) from the posted
// markdown lines. Deliberately conservative: anything outside that character class ends a token, so
// punctuation and email-like text won't spuriously match, and unknown names are filtered later by a
// users-table existence check.
fn extract_mentions(lines: &[String]) -> Vec<String> {
    let mut found: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for line in lines {
        let chars: Vec<char> = line.chars().collect();
        let mut i: usize = 0;
        while i < chars.len() {
            if chars[i] == '@' {
                let mut j: usize = i + 1;
                let mut name: String = String::new();
                while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                    name.push(chars[j]);
                    j += 1;
                }
                if !name.is_empty() {
                    found.insert(name);
                }
                i = j;
            } else {
                i += 1;
            }
        }
    }
    found.into_iter().collect()
}

// Build a short single-line preview of the posted text for the notification list.
fn build_excerpt(lines: &[String]) -> String {
    let joined: String = lines.join(" ");
    let trimmed: &str = joined.trim();
    let truncated: String = trimmed.chars().take(120).collect();
    if trimmed.chars().count() > 120 {
        format!("{truncated}…")
    } else {
        truncated
    }
}

// Fan out notifications for a freshly posted solution / comment / discussion / reply.
//
// `parent_owner` is the author of the thing being replied to (solution / discussion), or None for a
// top-level post — they get a 'reply' notification. Every @mentioned *existing* user gets a
// 'mention' notification (which takes priority if they're also the parent owner). The actor never
// notifies themselves.
pub async fn create_post_notifications(
    actor: &str,
    content_lines: &[String],
    parent_owner: Option<&str>,
    source_type: &str,
    target_url: &str
) {
    let excerpt: String = build_excerpt(content_lines);
    let mentions: Vec<String> = extract_mentions(content_lines);

    let mut conn: mysql_async::Conn = match get_db_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            warn(line!(), &format!("Failed to get a connection for notifications: {e}"));
            return;
        }
    };

    // recipient -> kind, with 'mention' overriding 'reply' for the same person.
    let mut recipients: std::collections::BTreeMap<String, &str> = std::collections::BTreeMap::new();

    if let Some(owner) = parent_owner
        && owner != actor {
            recipients.insert(owner.to_string(), "reply");
        }

    for name in &mentions {
        if name == actor {
            continue;
        }
        // Only notify names that resolve to a real account.
        let exists: Option<String> = conn
            .exec_first(
                "SELECT username FROM RsOJ.users WHERE username = :username",
                mysql_async::params! { "username" => name }
            )
            .await
            .ok()
            .flatten();
        if exists.is_some() {
            recipients.insert(name.clone(), "mention");
        }
    }

    let now: i64 = chrono::Utc::now().timestamp();
    for (recipient, kind) in recipients {
        let insert_result = conn
            .exec_drop(
                "INSERT INTO RsOJ.notifications
                (recipient, actor, kind, source_type, target_url, excerpt, is_read, created_at)
                VALUES (:recipient, :actor, :kind, :source_type, :target_url, :excerpt, FALSE, :created_at)",
                mysql_async::params! {
                    "recipient" => &recipient,
                    "actor" => actor,
                    "kind" => kind,
                    "source_type" => source_type,
                    "target_url" => target_url,
                    "excerpt" => &excerpt,
                    "created_at" => now,
                }
            )
            .await;
        if insert_result.is_err() {
            warn(line!(), &format!("Failed to insert a notification for `{recipient}`."));
        }
    }

    drop(conn);
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
    lookup: PendingNotificationLookup
) {
    let rpc_request_key: String = uuid::Uuid::new_v4().to_string();
    let ws_id: String = requester_ws_id.clone();
    PENDING_NOTIFICATION_LOOKUP_REQUESTS.lock().await.insert(
        rpc_request_key.clone(),
        PendingNotificationLookupRequest { requester_ws_id, original_request_key, lookup }
    );
    expire_pending(&*PENDING_NOTIFICATION_LOOKUP_REQUESTS, rpc_request_key.clone());

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
    action: PendingNotificationAuthedAction
) {
    let rpc_request_key: String = uuid::Uuid::new_v4().to_string();
    PENDING_NOTIFICATION_AUTHED_REQUESTS.lock().await.insert(
        rpc_request_key.clone(),
        PendingNotificationAuthedRequest {
            requester_ws_id,
            username: username.clone(),
            original_request_key,
            action,
        }
    );
    expire_pending(&*PENDING_NOTIFICATION_AUTHED_REQUESTS, rpc_request_key.clone());

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
    req: PendingNotificationLookupRequest
) {
    // Every inbox read requires a confirmed identity; an unresolved ws_id gets an empty answer.
    let requester: String = match requester_username {
        Some(name) if !name.is_empty() => name,
        _ => {
            send_empty(req).await;
            return;
        }
    };
    match req.lookup {
        PendingNotificationLookup::NotificationsList { page_index } => {
            notifications_list(req.requester_ws_id, req.original_request_key, requester, page_index).await;
        }
        PendingNotificationLookup::TotalNotificationsListIndex => {
            total_notifications_list_index(req.requester_ws_id, req.original_request_key, requester).await;
        }
        PendingNotificationLookup::UnreadCount => {
            unread_count(req.requester_ws_id, req.original_request_key, requester).await;
        }
    }
}

// Anonymous / unresolved requester — answer with the relevant empty shape so the frontend's
// request_key still resolves instead of hanging.
async fn send_empty(req: PendingNotificationLookupRequest) {
    match req.lookup {
        PendingNotificationLookup::NotificationsList { .. } => {
            let result = NotificationsListResult {
                r#type: String::from("notifications_list"),
                content: NotificationsListResultContent {
                    notifications_list: vec![],
                    request_key: req.original_request_key,
                },
            };
            send_json_msg_to_ws_server(req.requester_ws_id, serde_json::to_value(result).unwrap()).await;
        }
        PendingNotificationLookup::TotalNotificationsListIndex => {
            let result = TotalIndexResult {
                r#type: String::from("total_notifications_list_index"),
                content: TotalIndexResultContent {
                    total_notifications_list_index: 1,
                    request_key: req.original_request_key,
                },
            };
            send_json_msg_to_ws_server(req.requester_ws_id, serde_json::to_value(result).unwrap()).await;
        }
        PendingNotificationLookup::UnreadCount => {
            let result = UnreadCountResult {
                r#type: String::from("notifications_unread_count"),
                content: UnreadCountResultContent {
                    unread_count: 0,
                    request_key: req.original_request_key,
                },
            };
            send_json_msg_to_ws_server(req.requester_ws_id, serde_json::to_value(result).unwrap()).await;
        }
    }
}

async fn notifications_list(
    requester_ws_id: String,
    original_request_key: String,
    requester: String,
    page_index: i64
) {
    let page_index: i64 = std::cmp::max(1, page_index);
    let offset: i64 = (page_index - 1) * NOTIFICATIONS_LIST_PAGE_SIZE;

    let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
    type NotificationListResultType = Result<Vec<(i64, String, String, String, String, String, bool, i64)>, mysql_async::Error>;
    let results: NotificationListResultType = conn
        .exec(
            "SELECT notification_id, actor, kind, source_type, target_url, excerpt, is_read, created_at
            FROM RsOJ.notifications
            WHERE recipient = :recipient
            ORDER BY notification_id DESC
            LIMIT :limit OFFSET :offset",
            mysql_async::params! {
                "recipient" => &requester,
                "limit" => NOTIFICATIONS_LIST_PAGE_SIZE,
                "offset" => offset,
            }
        )
        .await;
    drop(conn);

    let rows = match results {
        Ok(rows) => rows,
        Err(e) => {
            warn(line!(), &format!("Failed to fetch the notifications list: {e}"));
            return;
        }
    };

    let notifications_list: Vec<NotificationListItem> = rows
        .into_iter()
        .map(|(notification_id, actor, kind, source_type, target_url, excerpt, is_read, created_at)| NotificationListItem {
            notification_id,
            actor,
            kind,
            source_type,
            target_url,
            excerpt,
            is_read,
            created_at,
        })
        .collect();

    let result = NotificationsListResult {
        r#type: String::from("notifications_list"),
        content: NotificationsListResultContent { notifications_list, request_key: original_request_key },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
}

async fn total_notifications_list_index(
    requester_ws_id: String,
    original_request_key: String,
    requester: String
) {
    let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
    let count: i64 = conn
        .exec_first::<i64, _, _>(
            "SELECT COUNT(*) FROM RsOJ.notifications WHERE recipient = :recipient",
            mysql_async::params! { "recipient" => &requester }
        )
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    drop(conn);

    let total_index: i64 = std::cmp::max(1, (count + NOTIFICATIONS_LIST_PAGE_SIZE - 1) / NOTIFICATIONS_LIST_PAGE_SIZE);
    let result = TotalIndexResult {
        r#type: String::from("total_notifications_list_index"),
        content: TotalIndexResultContent {
            total_notifications_list_index: total_index,
            request_key: original_request_key,
        },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
}

async fn unread_count(
    requester_ws_id: String,
    original_request_key: String,
    requester: String
) {
    let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
    let count: i64 = conn
        .exec_first::<i64, _, _>(
            "SELECT COUNT(*) FROM RsOJ.notifications WHERE recipient = :recipient AND is_read = FALSE",
            mysql_async::params! { "recipient" => &requester }
        )
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    drop(conn);

    let result = UnreadCountResult {
        r#type: String::from("notifications_unread_count"),
        content: UnreadCountResultContent { unread_count: count, request_key: original_request_key },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
}

// ---------------------------------------------------------------------------
// Write-side handlers (dispatched from on_validate_session_result).
// ---------------------------------------------------------------------------

pub async fn handle_authed_action(req: PendingNotificationAuthedRequest) {
    match req.action {
        PendingNotificationAuthedAction::MarkRead { notification_id } => {
            mark_read(req.requester_ws_id, req.original_request_key, req.username, notification_id).await;
        }
    }
}

pub async fn handle_invalid_session(req: PendingNotificationAuthedRequest) {
    let result = MarkReadResult {
        r#type: String::from("notification_mark_read_result"),
        content: MarkReadResultContent {
            result: String::from("invalid_session"),
            request_key: req.original_request_key,
        },
    };
    send_json_msg_to_ws_server(req.requester_ws_id, serde_json::to_value(result).unwrap()).await;
}

async fn mark_read(
    requester_ws_id: String,
    original_request_key: String,
    requester: String,
    notification_id: i64
) {
    let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
    // The `recipient = requester` predicate is what stops a user from touching anyone else's inbox.
    let update_result = if notification_id == 0 {
        conn
            .exec_drop(
                "UPDATE RsOJ.notifications SET is_read = TRUE WHERE recipient = :recipient AND is_read = FALSE",
                mysql_async::params! { "recipient" => &requester }
            )
            .await
    } else {
        conn
            .exec_drop(
                "UPDATE RsOJ.notifications SET is_read = TRUE WHERE recipient = :recipient AND notification_id = :notification_id",
                mysql_async::params! { "recipient" => &requester, "notification_id" => notification_id }
            )
            .await
    };
    drop(conn);

    let result_str: &str = if update_result.is_err() {
        warn(line!(), "Failed to mark notification(s) read.");
        "internal_error"
    } else {
        "ok"
    };
    let result = MarkReadResult {
        r#type: String::from("notification_mark_read_result"),
        content: MarkReadResultContent { result: String::from(result_str), request_key: original_request_key },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
}

// ---------------------------------------------------------------------------
// @mention autocomplete — a public username prefix search. No identity needed, so it answers
// directly without the simple_authenticator round trip.
// ---------------------------------------------------------------------------

pub async fn user_search(requester_ws_id: String, original_request_key: String, prefix: String) {
    let prefix: String = prefix.trim().to_string();
    let usernames: Vec<String> = if prefix.is_empty() {
        vec![]
    } else {
        // Escape LIKE wildcards in the user-supplied prefix so `%`/`_` are matched literally.
        let escaped: String = prefix
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern: String = format!("{escaped}%");

        let mut conn: mysql_async::Conn = get_db_conn().await.unwrap();
        let rows: Vec<String> = conn
            .exec(
                "SELECT username FROM RsOJ.users WHERE username LIKE :pattern ESCAPE '\\\\'
                ORDER BY username LIMIT :limit",
                mysql_async::params! { "pattern" => &pattern, "limit" => USER_SEARCH_LIMIT }
            )
            .await
            .unwrap_or_default();
        drop(conn);
        rows
    };

    let result = UserSearchResult {
        r#type: String::from("user_search"),
        content: UserSearchResultContent { usernames, request_key: original_request_key },
    };
    send_json_msg_to_ws_server(requester_ws_id, serde_json::to_value(result).unwrap()).await;
}
