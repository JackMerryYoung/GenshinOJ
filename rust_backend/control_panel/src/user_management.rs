use crate::global::*;
use mysql_async::prelude::*;

// Matches the actual `users` table schema:
//   id, username, password, accepted, test_accepted, general, created_at
// No email / is_banned / last_login columns exist.
#[derive(serde::Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub created_at: i64,
    pub accepted: i64,       // AC count stored directly on the user row
    pub submission_count: i64,
    pub discussion_count: i64,
}

#[derive(serde::Deserialize)]
pub struct UserListQuery {
    pub page: i64,
    pub page_size: i64,
    pub search: Option<String>,
}

#[derive(serde::Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserInfo>,
    pub total: i64,
    pub page: i64,
    pub total_pages: i64,
}

pub async fn get_user_list(query: UserListQuery) -> Result<UserListResponse, String> {
    let mut conn = MYSQL_DATABASE_POOL.get_conn().await.map_err(|e| e.to_string())?;

    let offset = (query.page - 1) * query.page_size;

    let (total, users): (i64, Vec<(i64, String, i64, i64)>) = if let Some(ref search) = query.search {
        let pattern = format!("%{}%", search);
        let total: Option<i64> = conn
            .exec_first(
                format!("SELECT COUNT(*) FROM `{DATABASE_NAME}`.`users` WHERE username LIKE :p"),
                params! { "p" => &pattern },
            ).await.map_err(|e| e.to_string())?;

        let rows = conn
            .exec(
                format!(
                    "SELECT id, username, created_at, accepted
                     FROM `{DATABASE_NAME}`.`users`
                     WHERE username LIKE :p
                     ORDER BY id DESC LIMIT :lim OFFSET :off"
                ),
                params! { "p" => &pattern, "lim" => query.page_size, "off" => offset },
            ).await.map_err(|e| e.to_string())?;

        (total.unwrap_or(0), rows)
    } else {
        let total: Option<i64> = conn
            .query_first(format!("SELECT COUNT(*) FROM `{DATABASE_NAME}`.`users`"))
            .await.map_err(|e| e.to_string())?;

        let rows = conn
            .exec(
                format!(
                    "SELECT id, username, created_at, accepted
                     FROM `{DATABASE_NAME}`.`users`
                     ORDER BY id DESC LIMIT :lim OFFSET :off"
                ),
                params! { "lim" => query.page_size, "off" => offset },
            ).await.map_err(|e| e.to_string())?;

        (total.unwrap_or(0), rows)
    };

    let mut user_infos = Vec::with_capacity(users.len());
    for (id, username, created_at, accepted) in users {
        let submission_count: Option<i64> = conn
            .exec_first(
                format!("SELECT COUNT(*) FROM `{DATABASE_NAME}`.`submissions` WHERE username = :u"),
                params! { "u" => &username },
            ).await.unwrap_or(Some(0));

        let discussion_count: Option<i64> = conn
            .exec_first(
                format!("SELECT COUNT(*) FROM `{DATABASE_NAME}`.`discussions` WHERE username = :u"),
                params! { "u" => &username },
            ).await.unwrap_or(Some(0));

        user_infos.push(UserInfo {
            id,
            username,
            created_at,
            accepted,
            submission_count: submission_count.unwrap_or(0),
            discussion_count: discussion_count.unwrap_or(0),
        });
    }

    drop(conn);

    Ok(UserListResponse {
        users: user_infos,
        total,
        page: query.page,
        total_pages: (total + query.page_size - 1) / query.page_size,
    })
}

pub async fn delete_user(user_id: i64) -> Result<(), String> {
    let mut conn = MYSQL_DATABASE_POOL.get_conn().await.map_err(|e| e.to_string())?;

    // Resolve username first so we can clean related tables (keyed by username).
    let username: Option<String> = conn
        .exec_first(
            format!("SELECT username FROM `{DATABASE_NAME}`.`users` WHERE id = :id"),
            params! { "id" => user_id },
        ).await.map_err(|e| e.to_string())?;
    let username = username.ok_or_else(|| format!("User {} not found", user_id))?;

    "SET FOREIGN_KEY_CHECKS = 0".ignore(&mut conn).await.map_err(|e| e.to_string())?;

    for tbl in &[
        format!("`{DATABASE_NAME}`.`submissions`"),
        format!("`{DATABASE_NAME}`.`discussions`"),
        format!("`{DATABASE_NAME}`.`solutions`"),
    ] {
        conn.exec_drop(
            format!("DELETE FROM {tbl} WHERE username = :u"),
            params! { "u" => &username },
        ).await.map_err(|e| e.to_string())?;
    }

    conn.exec_drop(
        format!("DELETE FROM `{DATABASE_NAME}`.`users` WHERE id = :id"),
        params! { "id" => user_id },
    ).await.map_err(|e| e.to_string())?;

    "SET FOREIGN_KEY_CHECKS = 1".ignore(&mut conn).await.map_err(|e| e.to_string())?;

    drop(conn);
    Ok(())
}
