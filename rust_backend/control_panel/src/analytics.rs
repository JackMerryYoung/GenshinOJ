use crate::global::*;
use mysql_async::prelude::*;

// Create the visits table (if missing). Called once at startup, after the database exists.
pub async fn ensure_schema() -> Result<(), String> {
    let mut conn: mysql_async::Conn = MYSQL_DATABASE_POOL
        .get_conn().await
        .map_err(|e| e.to_string())?;

    format!(
        "CREATE TABLE IF NOT EXISTS `{DATABASE_NAME}`.`{VISITS_TABLE_NAME}` (
            id INT AUTO_INCREMENT PRIMARY KEY NOT NULL,
            visited_at BIGINT NOT NULL
        )"
    )
        .ignore(&mut conn).await
        .map_err(|e| e.to_string())?;

    drop(conn);
    Ok(())
}

// Record a single visit at the current time.
pub async fn record_visit() -> Result<(), String> {
    let mut conn: mysql_async::Conn = MYSQL_DATABASE_POOL
        .get_conn().await
        .map_err(|e| e.to_string())?;

    let visited_at: i64 = chrono::Utc::now().timestamp_millis();
    conn
        .exec_drop(
            format!(
                "INSERT INTO `{DATABASE_NAME}`.`{VISITS_TABLE_NAME}` (visited_at) VALUES (:visited_at)"
            ),
            params! { "visited_at" => visited_at }
        ).await
        .map_err(|e| e.to_string())?;

    drop(conn);
    Ok(())
}

pub struct MetricPoint {
    pub label: String,
    pub daily: i64,
    pub cumulative: i64,
}

pub struct MetricSeries {
    pub key: String,
    pub title: String,
    pub points: Vec<MetricPoint>,
}

// Daily and cumulative counts for one metric over the last `days` days.
//
// `daily` is the per-day count; `cumulative` is the all-time running total up to and including
// that day (a baseline COUNT of everything before the window is added to the running sum, so the
// cumulative line reflects the true total, not just activity within the window).
//
// Days are bucketed in UTC both here (chrono) and in SQL (the connection's time_zone is pinned to
// +00:00) so the generated day labels line up with the grouped rows. The day axis is filled
// continuously, so days with no activity show up as zeros.
async fn get_metric_series(
    table: &str,
    timestamp_column: &str,
    days: i64
) -> Result<Vec<MetricPoint>, String> {
    let mut conn: mysql_async::Conn = MYSQL_DATABASE_POOL
        .get_conn().await
        .map_err(|e| e.to_string())?;

    "SET time_zone = '+00:00'".ignore(&mut conn).await.map_err(|e| e.to_string())?;

    let today: chrono::NaiveDate = chrono::Utc::now().date_naive();
    let start_date: chrono::NaiveDate = today - chrono::Duration::days(days - 1);
    let since: i64 = start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp_millis();

    let baseline: Option<i64> = conn
        .query_first(
            format!("SELECT COUNT(*) FROM {table} WHERE {timestamp_column} < {since}")
        ).await
        .map_err(|e| e.to_string())?;
    let baseline: i64 = baseline.unwrap_or(0);

    let rows: Vec<(String, i64)> = conn
        .exec(
            format!(
                "SELECT DATE_FORMAT(FROM_UNIXTIME({timestamp_column} / 1000), '%Y-%m-%d') AS day,
                        COUNT(*) AS cnt
                 FROM {table}
                 WHERE {timestamp_column} >= :since
                 GROUP BY day"
            ),
            params! { "since" => since }
        ).await
        .map_err(|e| e.to_string())?;

    drop(conn);

    let mut counts: std::collections::HashMap<String, i64> = rows.into_iter().collect();
    let mut points: Vec<MetricPoint> = Vec::with_capacity(days as usize);
    let mut running: i64 = baseline;
    for offset in 0..days {
        let date: chrono::NaiveDate = start_date + chrono::Duration::days(offset);
        let label: String = date.format("%Y-%m-%d").to_string();
        let daily: i64 = counts.remove(&label).unwrap_or(0);
        running += daily;
        points.push(MetricPoint { label, daily, cumulative: running });
    }
    Ok(points)
}

// All dashboard metrics. A per-metric query failure (e.g. a table another module hasn't created
// yet) yields an empty series for that metric rather than failing the whole dashboard.
pub async fn get_all_stats(days: i64) -> Vec<MetricSeries> {
    let metrics: [(&str, &str, String, &str); 5] = [
        ("visits", "Site visits", format!("`{DATABASE_NAME}`.`{VISITS_TABLE_NAME}`"), "visited_at"),
        ("registrations", "Registrations", format!("`{DATABASE_NAME}`.`users`"), "created_at"),
        ("discussions", "Discussions", format!("`{DATABASE_NAME}`.`discussions`"), "created_at"),
        ("solutions", "Solutions", format!("`{DATABASE_NAME}`.`solutions`"), "created_at"),
        ("submissions", "Submissions", format!("`{DATABASE_NAME}`.`submissions`"), "created_at"),
    ];

    let mut series: Vec<MetricSeries> = Vec::with_capacity(metrics.len());
    for (key, title, table, timestamp_column) in metrics {
        let points: Vec<MetricPoint> = get_metric_series(&table, timestamp_column, days).await
            .unwrap_or_default();
        series.push(MetricSeries {
            key: String::from(key),
            title: String::from(title),
            points,
        });
    }
    series
}
