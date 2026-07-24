use crate::global::*;
use mysql_async::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

// Global counters for real-time metrics
static ACTIVE_WS_CONNECTIONS: AtomicU64 = AtomicU64::new(0);

pub fn increment_ws_connections() {
    ACTIVE_WS_CONNECTIONS.fetch_add(1, Ordering::Relaxed);
}

pub fn decrement_ws_connections() {
    ACTIVE_WS_CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
}

#[derive(serde::Serialize)]
pub struct SystemStatus {
    pub database_connected: bool,
    pub database_pool_size: usize,
    pub active_ws_connections: u64,
    pub uptime_seconds: u64,
    pub memory_usage_mb: Option<f64>,
}

#[derive(serde::Serialize)]
pub struct ModuleInfo {
    pub name: String,
    pub status: String,
    pub port: u16,
}

lazy_static::lazy_static! {
    static ref START_TIME: std::time::Instant = std::time::Instant::now();
}

pub async fn get_system_status() -> Result<SystemStatus, String> {
    // Test database connection
    let guard_mysql_database_pool = MYSQL_DATABASE_POOL.lock().await;
    let database_connected = guard_mysql_database_pool.get_conn().await.is_ok();

    // Note: mysql_async Pool doesn't expose a status() method in this version
    // We'll use a fixed value for now
    let database_pool_size = 10; // Default pool size
    drop(guard_mysql_database_pool);

    // Get memory usage (Linux-specific via /proc/self/statm)
    let memory_usage_mb = get_memory_usage();

    // Get uptime
    let uptime_seconds = START_TIME.elapsed().as_secs();

    Ok(SystemStatus {
        database_connected,
        database_pool_size,
        active_ws_connections: ACTIVE_WS_CONNECTIONS.load(Ordering::Relaxed),
        uptime_seconds,
        memory_usage_mb,
    })
}

fn get_memory_usage() -> Option<f64> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/self/statm") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if let Some(rss_pages) = parts.get(1) {
                if let Ok(pages) = rss_pages.parse::<u64>() {
                    // Convert pages to MB (page size is typically 4KB)
                    return Some((pages * 4096) as f64 / 1024.0 / 1024.0);
                }
            }
        }
    }
    None
}

pub async fn get_recent_errors(limit: usize) -> Result<Vec<String>, String> {
    // This is a placeholder - you'd need to implement error logging first
    // For now, return empty array
    Ok(vec![])
}

#[derive(serde::Serialize)]
pub struct DatabaseStats {
    pub total_users: i64,
    pub total_problems: i64,
    pub total_submissions: i64,
    pub total_discussions: i64,
    pub database_size_mb: Option<f64>,
}

pub async fn get_database_stats() -> Result<DatabaseStats, String> {
    let guard_mysql_database_pool = MYSQL_DATABASE_POOL.lock().await;
    let mut conn = guard_mysql_database_pool
        .get_conn().await
        .map_err(|e| e.to_string())?;

    let total_users: Option<i64> = conn
        .query_first(format!("SELECT COUNT(*) FROM `{DATABASE_NAME}`.`users`"))
        .await.unwrap_or(Some(0));

    // problems PK is problem_number
    let total_problems: Option<i64> = conn
        .query_first(format!("SELECT COUNT(*) FROM `{DATABASE_NAME}`.`problems`"))
        .await.unwrap_or(Some(0));

    // submissions PK is submission_id
    let total_submissions: Option<i64> = conn
        .query_first(format!("SELECT COUNT(*) FROM `{DATABASE_NAME}`.`submissions`"))
        .await.unwrap_or(Some(0));

    let total_discussions: Option<i64> = conn
        .query_first(format!("SELECT COUNT(*) FROM `{DATABASE_NAME}`.`discussions`"))
        .await.unwrap_or(Some(0));

    let database_size_mb: Option<f64> = conn
        .query_first(format!(
            "SELECT SUM(data_length + index_length) / 1024.0 / 1024.0
             FROM information_schema.tables
             WHERE table_schema = '{DATABASE_NAME}'"
        ))
        .await.unwrap_or(None);

    drop(conn);
    drop(guard_mysql_database_pool);

    Ok(DatabaseStats {
        total_users: total_users.unwrap_or(0),
        total_problems: total_problems.unwrap_or(0),
        total_submissions: total_submissions.unwrap_or(0),
        total_discussions: total_discussions.unwrap_or(0),
        database_size_mb,
    })
}
