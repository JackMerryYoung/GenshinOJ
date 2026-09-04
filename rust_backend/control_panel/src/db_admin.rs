use crate::global::*;
use mysql_async::prelude::*;

// Wipe all data from the RsOJ database without dropping it.
//
// We deliberately TRUNCATE every base table rather than DROP the whole database: the backend is a
// long-running process and each module only creates its tables once, at startup. Dropping the
// database out from under a running server would leave every subsequent query failing against
// missing tables until a full restart. Truncating preserves the schema the running modules depend
// on and only clears the rows.
//
// Returns the number of tables truncated.
pub async fn clear_database() -> Result<usize, String> {
    let mut conn: mysql_async::Conn = MYSQL_DATABASE_POOL
        .get_conn().await
        .map_err(|e| e.to_string())?;

    // Fresh-install path: make sure the database exists before we try to inspect it.
    format!("CREATE DATABASE IF NOT EXISTS {DATABASE_NAME}")
        .ignore(&mut conn).await
        .map_err(|e| e.to_string())?;

    // Enumerate tables from information_schema so this stays correct as modules add new tables.
    let tables: Vec<String> = conn
        .exec(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema = :schema AND table_type = 'BASE TABLE'",
            params! { "schema" => DATABASE_NAME }
        ).await
        .map_err(|e| e.to_string())?;

    // Never truncate the control panel's own tables (visit analytics) — wiping
    // OJ data must not delete the traffic history.
    let tables: Vec<String> = tables
        .into_iter()
        .filter(|table| !table.starts_with(CONTROL_PANEL_TABLE_PREFIX))
        .collect();

    // Disable FK checks so truncation order doesn't matter across related tables.
    "SET FOREIGN_KEY_CHECKS = 0".ignore(&mut conn).await.map_err(|e| e.to_string())?;
    for table in &tables {
        // `table` comes from information_schema for our own schema, not from user input; the
        // backtick-quoting is belt-and-suspenders.
        format!("TRUNCATE TABLE `{DATABASE_NAME}`.`{table}`")
            .ignore(&mut conn).await
            .map_err(|e| e.to_string())?;
    }
    "SET FOREIGN_KEY_CHECKS = 1".ignore(&mut conn).await.map_err(|e| e.to_string())?;

    drop(conn);
    Ok(tables.len())
}
