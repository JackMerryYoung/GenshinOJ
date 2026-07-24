use crate::global::*;
use mysql_async::prelude::*;

// Matches the actual `problems` table schema:
//   problem_number (PK), problem_name, difficulty (INT), problem_statement
// No id / title / is_hidden / created_at / time_limit / memory_limit columns.
#[derive(serde::Serialize)]
pub struct ProblemInfo {
    pub problem_number: i64,
    pub problem_name: String,
    pub difficulty: i64,
    pub submission_count: i64,
    pub accepted_count: i64,
    pub acceptance_rate: f64,
}

#[derive(serde::Deserialize)]
pub struct ProblemListQuery {
    pub page: i64,
    pub page_size: i64,
    pub search: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ProblemListResponse {
    pub problems: Vec<ProblemInfo>,
    pub total: i64,
    pub page: i64,
    pub total_pages: i64,
}

pub async fn get_problem_list(query: ProblemListQuery) -> Result<ProblemListResponse, String> {
    let guard = MYSQL_DATABASE_POOL.lock().await;
    let mut conn = guard.get_conn().await.map_err(|e| e.to_string())?;

    let offset = (query.page - 1) * query.page_size;

    // Filter out empty search strings to treat them as None
    let search_term = query.search.as_ref().filter(|s| !s.trim().is_empty());

    let (total, rows): (i64, Vec<(i64, String, i64)>) = if let Some(search) = search_term {
        // Try to parse as problem number first; fall back to name search.
        let pattern = format!("%{}%", search);
        let search_num = search.parse::<i64>().ok();

        let total: Option<i64> = conn.exec_first(
            format!(
                "SELECT COUNT(*) FROM `{DATABASE_NAME}`.`problems`
                 WHERE problem_name LIKE :pat OR problem_number = :num"
            ),
            params! { "pat" => &pattern, "num" => search_num.unwrap_or(-1) },
        ).await.map_err(|e| e.to_string())?;

        let rows = conn.exec(
            format!(
                "SELECT problem_number, problem_name, difficulty
                 FROM `{DATABASE_NAME}`.`problems`
                 WHERE problem_name LIKE :pat OR problem_number = :num
                 ORDER BY problem_number DESC LIMIT :lim OFFSET :off"
            ),
            params! {
                "pat" => &pattern,
                "num" => search_num.unwrap_or(-1),
                "lim" => query.page_size,
                "off" => offset,
            },
        ).await.map_err(|e| e.to_string())?;

        (total.unwrap_or(0), rows)
    } else {
        let total: Option<i64> = conn
            .query_first(format!("SELECT COUNT(*) FROM `{DATABASE_NAME}`.`problems`"))
            .await.map_err(|e| e.to_string())?;

        let rows = conn.exec(
            format!(
                "SELECT problem_number, problem_name, difficulty
                 FROM `{DATABASE_NAME}`.`problems`
                 ORDER BY problem_number ASC LIMIT :lim OFFSET :off"
            ),
            params! { "lim" => query.page_size, "off" => offset },
        ).await.map_err(|e| e.to_string())?;

        (total.unwrap_or(0), rows)
    };

    let mut problems = Vec::with_capacity(rows.len());
    for (problem_number, problem_name, difficulty) in rows {
        let sub: Option<i64> = conn
            .exec_first(
                format!(
                    "SELECT COUNT(*) FROM `{DATABASE_NAME}`.`submissions`
                     WHERE problem_number = :n"
                ),
                params! { "n" => problem_number },
            ).await.unwrap_or(Some(0));

        let acc: Option<i64> = conn
            .exec_first(
                format!(
                    "SELECT COUNT(*) FROM `{DATABASE_NAME}`.`submissions`
                     WHERE problem_number = :n AND result = 'AC'"
                ),
                params! { "n" => problem_number },
            ).await.unwrap_or(Some(0));

        let sub_count = sub.unwrap_or(0);
        let acc_count = acc.unwrap_or(0);
        problems.push(ProblemInfo {
            problem_number,
            problem_name,
            difficulty,
            submission_count: sub_count,
            accepted_count: acc_count,
            acceptance_rate: if sub_count > 0 {
                (acc_count as f64 / sub_count as f64) * 100.0
            } else {
                0.0
            },
        });
    }

    drop(conn);
    drop(guard);

    Ok(ProblemListResponse {
        problems,
        total,
        page: query.page,
        total_pages: (total + query.page_size - 1) / query.page_size,
    })
}

#[derive(serde::Deserialize)]
pub struct CreateProblemRequest {
    pub problem_number: i64,
    pub problem_name: String,
    pub difficulty: i64,
    pub problem_statement: String,
    pub testcase_config: Option<String>,
}

pub async fn create_problem(request: CreateProblemRequest) -> Result<(), String> {
    let guard = MYSQL_DATABASE_POOL.lock().await;
    let mut conn = guard.get_conn().await.map_err(|e| e.to_string())?;

    // Check if problem number already exists
    let exists: Option<i64> = conn
        .exec_first(
            format!(
                "SELECT COUNT(*) FROM `{DATABASE_NAME}`.`problems` WHERE problem_number = :n"
            ),
            params! { "n" => request.problem_number },
        )
        .await
        .map_err(|e| e.to_string())?;

    if exists.unwrap_or(0) > 0 {
        return Err(format!("Problem number {} already exists", request.problem_number));
    }

    // Validate difficulty (0-7: Unknown/Beginner/Primary/Junior/Senior/Advanced/Hard/Grand)
    if request.difficulty < 0 || request.difficulty > 7 {
        return Err("Difficulty must be between 0 (Unknown) and 7 (Grand)".to_string());
    }

    // Validate testcase_config JSON if provided
    if let Some(ref tc_config) = request.testcase_config {
        if !tc_config.trim().is_empty() {
            serde_json::from_str::<serde_json::Value>(tc_config)
                .map_err(|e| format!("Invalid testcase_config JSON: {}", e))?;
        }
    }

    // Insert the new problem
    conn.exec_drop(
        format!(
            "INSERT INTO `{DATABASE_NAME}`.`problems`
             (problem_number, problem_name, difficulty, problem_statement, testcase_config)
             VALUES (:num, :name, :diff, :stmt, :tc_config)"
        ),
        params! {
            "num" => request.problem_number,
            "name" => &request.problem_name,
            "diff" => request.difficulty,
            "stmt" => &request.problem_statement,
            "tc_config" => &request.testcase_config,
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    drop(conn);
    drop(guard);
    Ok(())
}

// Get single problem details
#[derive(serde::Serialize)]
pub struct ProblemDetail {
    pub problem_number: i64,
    pub problem_name: String,
    pub difficulty: i64,
    pub problem_statement: String,
    pub testcase_config: Option<String>,
}

pub async fn get_problem_detail(problem_number: i64) -> Result<ProblemDetail, String> {
    let guard = MYSQL_DATABASE_POOL.lock().await;
    let mut conn = guard.get_conn().await.map_err(|e| e.to_string())?;

    let result: Option<(i64, String, i64, String, Option<String>)> = conn
        .exec_first(
            format!(
                "SELECT problem_number, problem_name, difficulty, problem_statement, testcase_config
                 FROM `{DATABASE_NAME}`.`problems` WHERE problem_number = :n"
            ),
            params! { "n" => problem_number },
        )
        .await
        .map_err(|e| e.to_string())?;

    drop(conn);
    drop(guard);

    match result {
        Some((problem_number, problem_name, difficulty, problem_statement, testcase_config)) => {
            Ok(ProblemDetail {
                problem_number,
                problem_name,
                difficulty,
                problem_statement,
                testcase_config,
            })
        }
        None => Err(format!("Problem {} not found", problem_number)),
    }
}

// Update problem
#[derive(serde::Deserialize)]
pub struct UpdateProblemRequest {
    pub problem_name: String,
    pub difficulty: i64,
    pub problem_statement: String,
    pub testcase_config: Option<String>,
}

pub async fn update_problem(
    problem_number: i64,
    request: UpdateProblemRequest,
) -> Result<(), String> {
    let guard = MYSQL_DATABASE_POOL.lock().await;
    let mut conn = guard.get_conn().await.map_err(|e| e.to_string())?;

    // Check if problem exists
    let exists: Option<i64> = conn
        .exec_first(
            format!(
                "SELECT COUNT(*) FROM `{DATABASE_NAME}`.`problems` WHERE problem_number = :n"
            ),
            params! { "n" => problem_number },
        )
        .await
        .map_err(|e| e.to_string())?;

    if exists.unwrap_or(0) == 0 {
        return Err(format!("Problem {} not found", problem_number));
    }

    // Validate difficulty (0-7: Unknown/Beginner/Primary/Junior/Senior/Advanced/Hard/Grand)
    if request.difficulty < 0 || request.difficulty > 7 {
        return Err("Difficulty must be between 0 (Unknown) and 7 (Grand)".to_string());
    }

    // Validate testcase_config JSON if provided
    if let Some(ref tc_config) = request.testcase_config {
        if !tc_config.trim().is_empty() {
            serde_json::from_str::<serde_json::Value>(tc_config)
                .map_err(|e| format!("Invalid testcase_config JSON: {}", e))?;
        }
    }

    // Update the problem
    conn.exec_drop(
        format!(
            "UPDATE `{DATABASE_NAME}`.`problems`
             SET problem_name = :name, difficulty = :diff, problem_statement = :stmt, testcase_config = :tc_config
             WHERE problem_number = :num"
        ),
        params! {
            "name" => &request.problem_name,
            "diff" => request.difficulty,
            "stmt" => &request.problem_statement,
            "tc_config" => &request.testcase_config,
            "num" => problem_number,
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    drop(conn);
    drop(guard);

    // The judge reads test config from the file, not the DB column, so mirror it
    // to `<project_root>/problem/<num>/problem_testcase_config.json`. Modules run
    // from `rust_backend/`, so the problem dir is one level up.
    if let Some(ref tc_config) = request.testcase_config {
        if !tc_config.trim().is_empty() {
            let dir = format!("../problem/{}", problem_number);
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create problem dir: {}", e))?;
            std::fs::write(format!("{}/problem_testcase_config.json", dir), tc_config)
                .map_err(|e| format!("Failed to write testcase config file: {}", e))?;
        }
    }

    Ok(())
}

pub async fn delete_problem(problem_number: i64) -> Result<(), String> {
    let guard = MYSQL_DATABASE_POOL.lock().await;
    let mut conn = guard.get_conn().await.map_err(|e| e.to_string())?;

    "SET FOREIGN_KEY_CHECKS = 0".ignore(&mut conn).await.map_err(|e| e.to_string())?;

    conn.exec_drop(
        format!(
            "DELETE FROM `{DATABASE_NAME}`.`submissions` WHERE problem_number = :n"
        ),
        params! { "n" => problem_number },
    ).await.map_err(|e| e.to_string())?;

    conn.exec_drop(
        format!(
            "DELETE FROM `{DATABASE_NAME}`.`solutions` WHERE problem_number = :n"
        ),
        params! { "n" => problem_number },
    ).await.map_err(|e| e.to_string())?;

    conn.exec_drop(
        format!(
            "DELETE FROM `{DATABASE_NAME}`.`problems` WHERE problem_number = :n"
        ),
        params! { "n" => problem_number },
    ).await.map_err(|e| e.to_string())?;

    "SET FOREIGN_KEY_CHECKS = 1".ignore(&mut conn).await.map_err(|e| e.to_string())?;

    drop(conn);
    drop(guard);
    Ok(())
}
