use crate::global::*;

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
    let memory_limit_kb = memory_limit_mb * 1024;
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
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    {
        use tokio::io::AsyncWriteExt;
        let mut stdin = child.stdin.take().unwrap();
        let _ = stdin.write_all(stdin_content.as_bytes()).await;
        drop(stdin);
    }

    match
        tokio::time::timeout(
            std::time::Duration::from_secs_f64(time_limit),
            child.wait_with_output()
        ).await
    {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(e)) => Err(format!("Failed to wait for process: {}", e)),
        Err(_) => Err(String::from("TLE")),
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

    let tmp_dir = std::env::temp_dir().join(format!("genshinoj_judge_{}", uuid::Uuid::new_v4()));
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
            let compile_output = tokio::process::Command
                ::new("gcc")
                .arg("main.c")
                .arg("-O2")
                .arg("-o")
                .arg("main")
                .current_dir(&tmp_dir)
                .output().await;
            let ok = matches!(compile_output, Ok(ref o) if o.status.success());
            ("main.c", String::from("./main"), vec![], if ok { Ok(()) } else { Err(()) })
        }
        "cpp" => {
            let source_path = tmp_dir.join("main.cpp");
            let _ = std::fs::write(&source_path, &source_code);
            let compile_output = tokio::process::Command
                ::new("g++")
                .arg("main.cpp")
                .arg("-O2")
                .arg("-o")
                .arg("main")
                .current_dir(&tmp_dir)
                .output().await;
            let ok = matches!(compile_output, Ok(ref o) if o.status.success());
            ("main.cpp", String::from("./main"), vec![], if ok { Ok(()) } else { Err(()) })
        }
        "java" => {
            let source_path = tmp_dir.join("Main.java");
            let _ = std::fs::write(&source_path, &source_code);
            let compile_output = tokio::process::Command
                ::new("javac")
                .arg("Main.java")
                .current_dir(&tmp_dir)
                .output().await;
            let ok = matches!(compile_output, Ok(ref o) if o.status.success());
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
                if reason == "TLE" {
                    String::from("TLE")
                } else {
                    String::from("RE")
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
