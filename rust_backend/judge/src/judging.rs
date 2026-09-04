use crate::global::*;

use tokio::io::AsyncReadExt;

const MAX_JUDGE_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
const COMPILE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(serde::Deserialize)]
struct TestcaseConfig {
    score: i32,
    input: String,
    answer: String,
    time_limit: f64,
    memory_limit: i64,
}

#[derive(serde::Deserialize)]
struct TestcaseConfigFile {
    testcases: Vec<TestcaseConfig>,
}

pub struct JudgeOutcome {
    pub result: String,
    pub general_score: i32,
    pub statuses: Vec<String>,
    pub scores: Vec<i32>,
}

fn all_failed(testcase_count: usize, status: &str) -> JudgeOutcome {
    JudgeOutcome {
        result: String::from(status),
        general_score: 0,
        statuses: (0..testcase_count).map(|_| String::from(status)).collect(),
        scores: (0..testcase_count).map(|_| 0).collect(),
    }
}

fn normalize_output(output: &str) -> Vec<&str> {
    output
        .lines()
        .map(|line| line.trim_end())
        .collect()
}

// Note: this provides only a basic wall-clock timeout and a `ulimit -v` memory cap via the
// wrapping shell. It does NOT sandbox the process (no chroot/cgroups/seccomp/network isolation) —
// arbitrary code from any authenticated user runs directly on the host. Acceptable for a trusted
// dev/class environment, not for running submissions from untrusted strangers.
async fn run_with_limits(
    command: &str,
    args: &[&str],
    cwd: &std::path::Path,
    stdin_content: &str,
    time_limit: f64,
    memory_limit_mb: i64
) -> Result<std::process::Output, String> {
    let memory_limit_kb = memory_limit_mb
        .checked_mul(1024)
        .filter(|value| *value > 0)
        .ok_or_else(|| String::from("invalid memory limit"))?;
    let full_command = format!(
        "ulimit -v {}; exec {} {}",
        memory_limit_kb,
        command,
        args.join(" ")
    );

    let mut child = tokio::process::Command
        ::new("sh")
        .arg("-c")
        .arg(&full_command)
        .current_dir(cwd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        // stderr is not part of the judging result. Discarding it prevents a submission that
        // writes continuously to stderr from consuming an unbounded buffer.
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    let Some(stdout) = child.stdout.take() else {
        let _ = child.start_kill();
        return Err(String::from("failed to open child stdout"));
    };
    let mut stdout_task = tokio::spawn(async move {
        let mut stdout = stdout.take((MAX_JUDGE_OUTPUT_BYTES + 1) as u64);
        let mut bytes = Vec::new();
        let result = stdout.read_to_end(&mut bytes).await;
        (result, bytes)
    });

    // Keep stdin writing concurrent with stdout draining. Otherwise a submission that writes a
    // full pipe and then reads stdin can deadlock the judge before the time limit starts.
    let Some(mut stdin) = child.stdin.take() else {
        stdout_task.abort();
        let _ = child.start_kill();
        let _ = child.wait().await;
        return Err(String::from("failed to open child stdin"));
    };
    let stdin_content = stdin_content.to_owned();
    let stdin_task = tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        let _ = stdin.write_all(stdin_content.as_bytes()).await;
        drop(stdin);
    });

    let timeout_duration = std::time::Duration::from_secs_f64(time_limit.max(0.001));
    let wait_result = tokio::time::timeout(timeout_duration, child.wait()).await;
    let status = match wait_result {
        Ok(Ok(status)) => status,
        Ok(Err(error)) => {
            let _ = child.start_kill();
            let _ = tokio::time::timeout(std::time::Duration::from_secs(1), child.wait()).await;
            stdin_task.abort();
            stdout_task.abort();
            return Err(format!("Failed to wait for process: {}", error));
        }
        Err(_) => {
            // `kill_on_drop` is a final backstop, but explicitly request termination before
            // dropping the child so a timed-out submission does not remain in the process table.
            let _ = child.start_kill();
            let _ = tokio::time::timeout(std::time::Duration::from_secs(1), child.wait()).await;
            stdin_task.abort();
            stdout_task.abort();
            return Err(String::from("TLE"));
        }
    };

    stdin_task.abort();

    let (read_result, stdout) = match tokio::time::timeout(
        std::time::Duration::from_secs(1),
        &mut stdout_task
    ).await {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => return Err(format!("Failed to read process output: {}", error)),
        Err(_) => {
            stdout_task.abort();
            return Err(String::from("RE"));
        }
    };
    if let Err(error) = read_result {
        return Err(format!("Failed to read process output: {}", error));
    }
    if stdout.len() > MAX_JUDGE_OUTPUT_BYTES {
        return Err(String::from("OLE"));
    }

    Ok(std::process::Output { status, stdout, stderr: Vec::new() })
}

async fn compile_source(
    command: &str,
    args: &[&str],
    cwd: &std::path::Path
) -> bool {
    let mut child = match tokio::process::Command::new(command)
        .args(args)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn() {
        Ok(child) => child,
        Err(_) => return false,
    };
    match tokio::time::timeout(COMPILE_TIMEOUT, child.wait()).await {
        Ok(Ok(status)) => status.success(),
        Ok(Err(_)) | Err(_) => {
            let _ = child.start_kill();
            let _ = tokio::time::timeout(std::time::Duration::from_secs(1), child.wait()).await;
            false
        }
    }
}

pub async fn judge_submission(
    problem_number: i64,
    language: &str,
    code: &[String]
) -> JudgeOutcome {
    let problem_dir = format!("{}/{}", get_problem_dir_path(), problem_number);
    let testcase_config_path = format!("{}/problem_testcase_config.json", problem_dir);

    let testcase_config_string = match std::fs::read_to_string(&testcase_config_path) {
        Ok(x) => x,
        Err(_) => {
            return all_failed(0, "JE");
        }
    };
    let testcase_config: TestcaseConfigFile = match
        serde_json::from_str(&testcase_config_string)
    {
        Ok(x) => x,
        Err(_) => {
            return all_failed(0, "JE");
        }
    };

    let tmp_dir = std::env::temp_dir().join(format!("rsoj_judge_{}", uuid::Uuid::new_v4()));
    if std::fs::create_dir_all(&tmp_dir).is_err() {
        return all_failed(testcase_config.testcases.len(), "JE");
    }

    let source_code = code.join("\n");
    let (source_file_name, run_command, run_args, compile_result): (
        &str,
        String,
        Vec<String>,
        Result<(), ()>,
    ) = match language {
        "c" => {
            let source_path = tmp_dir.join("main.c");
            let _ = std::fs::write(&source_path, &source_code);
            let ok = compile_source("gcc", &["main.c", "-O2", "-o", "main"], &tmp_dir).await;
            ("main.c", String::from("./main"), vec![], if ok { Ok(()) } else { Err(()) })
        }
        "cpp" => {
            let source_path = tmp_dir.join("main.cpp");
            let _ = std::fs::write(&source_path, &source_code);
            let ok = compile_source("g++", &["main.cpp", "-O2", "-o", "main"], &tmp_dir).await;
            ("main.cpp", String::from("./main"), vec![], if ok { Ok(()) } else { Err(()) })
        }
        "java" => {
            let source_path = tmp_dir.join("Main.java");
            let _ = std::fs::write(&source_path, &source_code);
            let ok = compile_source("javac", &["Main.java"], &tmp_dir).await;
            ("Main.java", String::from("java"), vec![String::from("Main")], if ok {
                Ok(())
            } else {
                Err(())
            })
        }
        "py" => {
            let source_path = tmp_dir.join("main.py");
            let _ = std::fs::write(&source_path, &source_code);
            ("main.py", String::from("python3"), vec![String::from("main.py")], Ok(()))
        }
        _ => {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return all_failed(testcase_config.testcases.len(), "JE");
        }
    };
    let _ = source_file_name;

    if compile_result.is_err() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return all_failed(testcase_config.testcases.len(), "CE");
    }

    let mut statuses: Vec<String> = Vec::with_capacity(testcase_config.testcases.len());
    let mut scores: Vec<i32> = Vec::with_capacity(testcase_config.testcases.len());
    let mut general_score: i32 = 0;
    let mut all_accepted = true;

    let run_args_str: Vec<&str> = run_args.iter().map(String::as_str).collect();

    for testcase in &testcase_config.testcases {
        let input_path = format!("{}/input/{}", problem_dir, testcase.input);
        let answer_path = format!("{}/answer/{}", problem_dir, testcase.answer);

        let input_content = std::fs::read_to_string(&input_path).unwrap_or_default();
        let answer_content = std::fs::read_to_string(&answer_path).unwrap_or_default();

        let status = match
            run_with_limits(
                &run_command,
                &run_args_str,
                &tmp_dir,
                &input_content,
                testcase.time_limit,
                testcase.memory_limit
            ).await
        {
            Ok(output) => {
                if !output.status.success() {
                    String::from("RE")
                } else {
                    let actual_stdout = String::from_utf8_lossy(&output.stdout);
                    if normalize_output(&actual_stdout) == normalize_output(&answer_content) {
                        String::from("AC")
                    } else {
                        String::from("WA")
                    }
                }
            }
            Err(reason) => {
                match reason.as_str() {
                    "TLE" => String::from("TLE"),
                    "OLE" => String::from("OLE"),
                    _ => String::from("RE"),
                }
            }
        };

        if status == "AC" {
            general_score += testcase.score;
            scores.push(testcase.score);
        } else {
            all_accepted = false;
            scores.push(0);
        }
        statuses.push(status);
    }

    let _ = std::fs::remove_dir_all(&tmp_dir);

    JudgeOutcome {
        result: if all_accepted {
            String::from("AC")
        } else {
            statuses
                .iter()
                .find(|s| s.as_str() != "AC")
                .cloned()
                .unwrap_or_else(|| String::from("WA"))
        },
        general_score,
        statuses,
        scores,
    }
}
