//! `rustc --test` verifier for frozen M6 CodeTasks rows.
//!
//! Compiles each row's `SandboxProgram::to_rust_source` output with `rustc --edition 2021 --test`,
//! runs the resulting binary under a wall-clock deadline, and reports pass/fail per row. This is
//! instrument QA for the bar and its transpiler, not a capability result.
//!
//! Process handling follows L-005: only the spawned child's own pid is killed, never a pattern.

use std::io::{self, Read as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const COMPILE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const RUN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const MAX_OUTPUT_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyOutcome {
    Pass { passed: usize },
    CompileError { stderr: String },
    TestFailure { stdout: String, stderr: String },
    UnexpectedOutput { stdout: String, stderr: String },
    Timeout { phase: &'static str },
    RunError { detail: String },
}

#[derive(Debug, Clone)]
pub struct RowVerification {
    pub row_id: String,
    pub outcome: VerifyOutcome,
}

struct TempDir(PathBuf);

impl TempDir {
    fn create(label: &str) -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "uor-verify-{}-{}-{}",
            std::process::id(),
            label,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn env_without_git() -> Vec<(String, String)> {
    std::env::vars()
        .filter(|(k, _)| !k.starts_with("GIT_"))
        .collect()
}

fn run_bounded(
    cmd: &mut Command,
    timeout: std::time::Duration,
) -> io::Result<std::process::Output> {
    let serial = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let stem = format!("uor-verify-io-{}-{serial}", std::process::id());
    let out_path = std::env::temp_dir().join(format!("{stem}.stdout"));
    let err_path = std::env::temp_dir().join(format!("{stem}.stderr"));

    struct FileGuard(PathBuf);
    impl Drop for FileGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    let out_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&out_path)?;
    let _out_guard = FileGuard(out_path.clone());
    let err_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&err_path)?;
    let _err_guard = FileGuard(err_path.clone());

    cmd.stdin(Stdio::null()).stdout(out_file).stderr(err_file);

    let mut child = cmd.spawn()?;
    let started = std::time::Instant::now();

    let status = loop {
        let oversized = [&out_path, &err_path]
            .iter()
            .any(|p| std::fs::metadata(p).is_ok_and(|m| m.len() > MAX_OUTPUT_BYTES));
        if oversized || started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(
                if oversized {
                    io::ErrorKind::InvalidData
                } else {
                    io::ErrorKind::TimedOut
                },
                if oversized {
                    "process output limit exceeded"
                } else {
                    "process deadline exceeded"
                },
            ));
        }
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e);
            }
        }
    };

    let read_capped = |path: &Path| -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(MAX_OUTPUT_BYTES + 1)
            .read_to_end(&mut bytes)?;
        bytes.truncate(MAX_OUTPUT_BYTES as usize);
        Ok(bytes)
    };

    Ok(std::process::Output {
        status,
        stdout: read_capped(&out_path)?,
        stderr: read_capped(&err_path)?,
    })
}

fn parse_test_result(stdout: &str) -> Option<(usize, usize)> {
    for line in stdout.lines().rev() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("test result: ") {
            let after_dot = if let Some(a) = rest.strip_prefix("ok. ") {
                a
            } else if let Some(a) = rest.strip_prefix("FAILED. ") {
                a
            } else {
                continue;
            };
            let mut passed = None;
            let mut failed = None;
            for part in after_dot.split(';') {
                let part = part.trim();
                if let Some(n) = part.strip_suffix(" passed") {
                    passed = n.trim().parse::<usize>().ok();
                } else if let Some(n) = part.strip_suffix(" failed") {
                    failed = n.trim().parse::<usize>().ok();
                }
            }
            if let (Some(p), Some(f)) = (passed, failed) {
                return Some((p, f));
            }
        }
    }
    None
}

/// Verify already-generated, trusted Rust source.
///
/// This is not a sandbox: the compiled test binary has the host process's
/// filesystem, network and environment access, except that `GIT_*` variables
/// are removed. Never pass model-emitted or untrusted source.
pub fn verify_row(row_id: &str, rust_source: &str) -> RowVerification {
    let outcome = match verify_source(rust_source) {
        Ok(o) => o,
        Err(e) => VerifyOutcome::RunError {
            detail: e.to_string(),
        },
    };
    RowVerification {
        row_id: row_id.to_string(),
        outcome,
    }
}

fn verify_source(rust_source: &str) -> io::Result<VerifyOutcome> {
    let tmp = TempDir::create("ct")?;
    let src_path = tmp.path().join("task.rs");
    let bin_path = tmp.path().join("task_test");
    std::fs::write(&src_path, rust_source)?;

    let env = env_without_git();
    let compile = run_bounded(
        Command::new("rustc")
            .arg("--edition")
            .arg("2021")
            .arg("--test")
            .arg(&src_path)
            .arg("-o")
            .arg(&bin_path)
            .env_clear()
            .envs(env.iter().map(|(k, v)| (k.as_str(), v.as_str()))),
        COMPILE_TIMEOUT,
    );

    match compile {
        Err(e) if e.kind() == io::ErrorKind::TimedOut => {
            return Ok(VerifyOutcome::Timeout { phase: "compile" });
        }
        Err(e) => return Err(e),
        Ok(output) if !output.status.success() => {
            return Ok(VerifyOutcome::CompileError {
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }
        Ok(_) => {}
    }

    let env = env_without_git();
    let run = run_bounded(
        Command::new(&bin_path)
            .env_clear()
            .envs(env.iter().map(|(k, v)| (k.as_str(), v.as_str()))),
        RUN_TIMEOUT,
    );

    match run {
        Err(e) if e.kind() == io::ErrorKind::TimedOut => {
            Ok(VerifyOutcome::Timeout { phase: "run" })
        }
        Err(e) => Err(e),
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if output.status.success() {
                match parse_test_result(&stdout) {
                    Some((passed, 0)) if passed > 0 => Ok(VerifyOutcome::Pass { passed }),
                    Some(_) => Ok(VerifyOutcome::TestFailure { stdout, stderr }),
                    None => Ok(VerifyOutcome::UnexpectedOutput { stdout, stderr }),
                }
            } else {
                Ok(VerifyOutcome::TestFailure { stdout, stderr })
            }
        }
    }
}

pub fn verify_all_frozen_code_tasks() -> Vec<RowVerification> {
    use super::m6_reasoning::{generate_frozen_m6_panel, M6RowSpec};

    let panel = generate_frozen_m6_panel();
    let mut results = Vec::new();

    for row in &panel.rows {
        if let M6RowSpec::CodeTasks(spec) = &row.spec {
            let source = spec
                .canonical_program
                .to_rust_source("solve", &spec.test_vectors);
            results.push(verify_row(&row.row_id, &source));
        }
    }
    results
}
