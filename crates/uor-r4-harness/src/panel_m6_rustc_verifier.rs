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
use std::sync::atomic::{AtomicU64, Ordering};

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

/// Per-process sequence for [`TempDir::create`]: two calls in one process never share a name,
/// even when the clock reads the same nanosecond (issue #3876).
static TEMP_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

/// Attempts before [`TempDir::create`] gives up on names that already exist.
const TEMP_DIR_ATTEMPTS: u32 = 16;

struct TempDir(PathBuf);

impl TempDir {
    /// Makes a fresh directory owned by this caller alone.
    ///
    /// The name is pid + label + per-process sequence + clock nanoseconds, and `create_dir`
    /// refuses an existing path, so a collision is retried under a new sequence number instead of
    /// silently sharing another caller's files.
    fn create(label: &str) -> io::Result<Self> {
        Self::create_in(&std::env::temp_dir(), label)
    }

    fn create_in(root: &Path, label: &str) -> io::Result<Self> {
        std::fs::create_dir_all(root)?;
        for _ in 0..TEMP_DIR_ATTEMPTS {
            let path = root.join(format!(
                "uor-verify-{}-{}-{}-{}",
                std::process::id(),
                label,
                TEMP_DIR_SEQ.fetch_add(1, Ordering::Relaxed),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("no unused verifier temp dir name for label {label:?} after {TEMP_DIR_ATTEMPTS} attempts"),
        ))
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
    capture: &Path,
) -> io::Result<std::process::Output> {
    let out_path = capture.with_extension("stdout");
    let err_path = capture.with_extension("stderr");

    let out_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&out_path)?;
    let err_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&err_path)?;

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
        &tmp.path().join("compile"),
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
        &tmp.path().join("run"),
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

#[cfg(test)]
mod tests {
    use super::TempDir;
    use std::collections::HashSet;

    /// Issue #3876: two directories made under one label in one process must be distinct,
    /// so one test's `task.rs` can never be read as another's.
    #[test]
    fn same_label_temp_dirs_never_share_a_path() {
        let a = TempDir::create("ct").expect("first temp dir");
        let b = TempDir::create("ct").expect("second temp dir");
        assert_ne!(a.path(), b.path());
        assert!(a.path().is_dir() && b.path().is_dir());
    }

    #[test]
    fn missing_temp_root_is_created_and_survives_child_cleanup() {
        let parent = TempDir::create("test-root").expect("parent");
        let root = parent.path().join("missing").join("nested");
        let child = TempDir::create_in(&root, "ct").expect("fresh child");
        assert_eq!(child.path().parent(), Some(root.as_path()));
        assert!(child.path().is_dir());
        drop(child);
        assert!(root.is_dir());
    }

    #[test]
    fn parallel_same_label_temp_dirs_are_all_distinct() {
        let dirs: Vec<TempDir> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..16)
                .map(|_| scope.spawn(|| TempDir::create("ct").expect("temp dir")))
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("thread"))
                .collect()
        });
        let unique: HashSet<_> = dirs.iter().map(|d| d.path().to_path_buf()).collect();
        assert_eq!(unique.len(), dirs.len());
    }

    #[test]
    fn dropping_a_temp_dir_removes_it() {
        let dir = TempDir::create("ct").expect("temp dir");
        let path = dir.path().to_path_buf();
        std::fs::write(path.join("task.rs"), "fn main() {}").expect("write");
        drop(dir);
        assert!(!path.exists());
    }
}
