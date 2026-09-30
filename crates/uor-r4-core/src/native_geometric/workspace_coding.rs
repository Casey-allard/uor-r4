//! Explicit host workspace and compiler utilities; these functions do not learn,
//! synthesize, or repair code without a caller-supplied program or patch strategy.
//! They are not a sandbox for untrusted programs. Paths reject absolute paths,
//! traversal and symlinks; input/output limits and process deadlines bound work.

use super::{Error, Result};
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

pub const MAX_WORKSPACE_FILE_BYTES: usize = 1024 * 1024;
pub const MAX_WORKSPACE_FILES: usize = 1024;
pub const MAX_WORKSPACE_TOTAL_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_PROCESS_OUTPUT_BYTES: u64 = 1024 * 1024;
pub const PROCESS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

trait BoundedCommandOutput {
    fn bounded_output(&mut self) -> io::Result<Output>;
}
impl BoundedCommandOutput for Command {
    #[cfg(target_arch = "wasm32")]
    fn bounded_output(&mut self) -> io::Result<Output> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Host process execution is unavailable in WASM",
        ))
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn bounded_output(&mut self) -> io::Result<Output> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        struct Capture(std::path::PathBuf);
        impl Drop for Capture {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
            }
        }
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let stem = format!("uor-process-{}-{serial}", std::process::id());
        let out_path = std::env::temp_dir().join(format!("{stem}.stdout"));
        let err_path = std::env::temp_dir().join(format!("{stem}.stderr"));
        let create = |path: &Path| {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
        };
        let out_file = create(&out_path)?;
        let out = Capture(out_path);
        let err_file = create(&err_path)?;
        let err = Capture(err_path);
        self.stdin(Stdio::null()).stdout(out_file).stderr(err_file);
        let mut child = self.spawn()?;
        let started = std::time::Instant::now();
        let status = loop {
            let oversized = [&out.0, &err.0]
                .iter()
                .any(|p| std::fs::metadata(p).is_ok_and(|m| m.len() > MAX_PROCESS_OUTPUT_BYTES));
            if oversized || started.elapsed() >= PROCESS_TIMEOUT {
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
                Ok(Some(status)) => break status,
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(error);
                }
            }
        };
        let read = |path: &Path| -> io::Result<Vec<u8>> {
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(MAX_PROCESS_OUTPUT_BYTES + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 > MAX_PROCESS_OUTPUT_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "process output limit exceeded",
                ));
            }
            Ok(bytes)
        };
        Ok(Output {
            status,
            stdout: read(&out.0)?,
            stderr: read(&err.0)?,
        })
    }
}

/// Canonical schema for workspace coding evaluations.
pub const WORKSPACE_CODING_SCHEMA: &str = "uor-r4.workspace-coding/1";

/// Cryptographic and quantitative summary of workspace state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceRevision {
    /// Deterministic hexadecimal digest of file contents and paths.
    pub revision_id: String,
    /// Number of tracked files in workspace.
    pub file_count: usize,
    /// Total bytes across all tracked files.
    pub total_bytes: u64,
}

/// Controlled workspace environment for bounded file operations.
#[derive(Debug, Clone)]
pub struct WorkspaceEnvironment {
    root_dir: PathBuf,
}

impl WorkspaceEnvironment {
    /// Initialize workspace rooted at `root_dir`.
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    /// Root directory path.
    pub fn root(&self) -> &Path {
        &self.root_dir
    }

    /// Reject absolute paths, traversal and symlink components before file access.
    /// This is path hygiene for a trusted local workspace, not race-proof isolation.
    fn resolve_path(&self, relative_path: &Path) -> Result<PathBuf> {
        if relative_path.as_os_str().is_empty()
            || relative_path.is_absolute()
            || relative_path.components().any(|c| {
                matches!(
                    c,
                    std::path::Component::ParentDir
                        | std::path::Component::RootDir
                        | std::path::Component::Prefix(_)
                )
            })
        {
            return Err(Error(format!(
                "Path traversal or absolute path rejected: {relative_path:?}"
            )));
        }
        let mut joined = self.root_dir.clone();
        for component in relative_path.components() {
            joined.push(component);
            match std::fs::symlink_metadata(&joined) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(Error(format!("Symlink path rejected: {relative_path:?}")))
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(Error(format!("Path inspection failed: {error}"))),
            }
        }
        Ok(joined)
    }

    pub fn read_file(&self, relative_path: &Path) -> Result<String> {
        let path = self.resolve_path(relative_path)?;
        let mut bytes = Vec::new();
        std::fs::File::open(&path)
            .and_then(|f| {
                f.take(MAX_WORKSPACE_FILE_BYTES as u64 + 1)
                    .read_to_end(&mut bytes)
            })
            .map_err(|e| Error(format!("Failed to read {relative_path:?}: {e}")))?;
        if bytes.len() > MAX_WORKSPACE_FILE_BYTES {
            return Err(Error("Workspace file byte limit exceeded".into()));
        }
        String::from_utf8(bytes).map_err(|e| Error(format!("Workspace file is not UTF-8: {e}")))
    }

    pub fn write_file(&self, relative_path: &Path, content: &str) -> Result<()> {
        if content.len() > MAX_WORKSPACE_FILE_BYTES {
            return Err(Error("Workspace file byte limit exceeded".into()));
        }
        let path = self.resolve_path(relative_path)?;
        let files = self.list_files()?;
        if !path.exists() && files.len() >= MAX_WORKSPACE_FILES {
            return Err(Error("Workspace file count limit exceeded".into()));
        }
        let mut resulting_bytes = content.len() as u64;
        for existing in files {
            let existing_path = self.resolve_path(&existing)?;
            if existing_path != path {
                let size = std::fs::metadata(&existing_path)
                    .map_err(|e| Error(e.to_string()))?
                    .len();
                resulting_bytes = resulting_bytes
                    .checked_add(size)
                    .ok_or_else(|| Error("Workspace size overflow".into()))?;
            }
        }
        if resulting_bytes > MAX_WORKSPACE_TOTAL_BYTES {
            return Err(Error("Workspace total byte limit exceeded".into()));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error(format!("Failed to create parent: {e}")))?;
        }
        std::fs::write(path, content)
            .map_err(|e| Error(format!("Failed to write {relative_path:?}: {e}")))
    }

    /// Apply a surgical patch replacing an exact unique occurrence of `search` with `replacement`.
    pub fn apply_patch(&self, relative_path: &Path, search: &str, replacement: &str) -> Result<()> {
        let existing = self.read_file(relative_path)?;
        let count = existing.matches(search).count();
        if count == 0 {
            return Err(Error(format!(
                "Search pattern not found in {:?}: '{}'",
                relative_path, search
            )));
        }
        if count > 1 {
            return Err(Error(format!(
                "Search pattern occurs {} times in {:?}; expected exactly one",
                count, relative_path
            )));
        }
        let updated = existing.replace(search, replacement);
        self.write_file(relative_path, &updated)
    }

    /// Recursively list all files relative to workspace root.
    pub fn list_files(&self) -> Result<Vec<PathBuf>> {
        let mut results = Vec::new();
        self.collect_files(&self.root_dir, &mut results)?;
        results.sort();
        Ok(results)
    }

    fn collect_files(&self, dir: &Path, acc: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        if dir
            .strip_prefix(&self.root_dir)
            .map_or(true, |p| p.components().count() > 32)
        {
            return Err(Error("Workspace directory depth limit exceeded".into()));
        }
        let entries = std::fs::read_dir(dir)
            .map_err(|e| Error(format!("Failed to read dir {:?}: {}", dir, e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| Error(format!("Dir entry error: {}", e)))?;
            let path = entry.path();
            if entry
                .file_type()
                .map_err(|e| Error(e.to_string()))?
                .is_symlink()
            {
                return Err(Error(format!("Symlink path rejected: {path:?}")));
            }
            if acc.len() >= MAX_WORKSPACE_FILES {
                return Err(Error("Workspace file count limit exceeded".into()));
            }
            if path.is_dir() {
                self.collect_files(&path, acc)?;
            } else if path.is_file() {
                if let Ok(rel) = path.strip_prefix(&self.root_dir) {
                    acc.push(rel.to_path_buf());
                }
            }
        }
        Ok(())
    }

    /// Compute deterministic cryptographic revision over all files in the workspace.
    pub fn compute_revision(&self) -> Result<WorkspaceRevision> {
        let files = self.list_files()?;
        let mut total_bytes = 0u64;
        let mut hasher = blake3::Hasher::new();

        for rel in &files {
            let path = self.resolve_path(rel)?;
            let mut bytes = Vec::new();
            std::fs::File::open(&path)
                .and_then(|f| {
                    f.take(MAX_WORKSPACE_FILE_BYTES as u64 + 1)
                        .read_to_end(&mut bytes)
                })
                .map_err(|e| Error(format!("Failed to read bytes for {rel:?}: {e}")))?;
            if bytes.len() > MAX_WORKSPACE_FILE_BYTES {
                return Err(Error("Workspace file byte limit exceeded".into()));
            }
            total_bytes += bytes.len() as u64;
            if total_bytes > MAX_WORKSPACE_TOTAL_BYTES {
                return Err(Error("Workspace total byte limit exceeded".into()));
            }
            hasher.update(rel.to_string_lossy().as_bytes());
            hasher.update(b":");
            hasher.update(&bytes);
            hasher.update(b"\n");
        }

        let revision_id = hasher.finalize().to_hex().to_string();
        Ok(WorkspaceRevision {
            revision_id,
            file_count: files.len(),
            total_bytes,
        })
    }
}

/// A structured compiler diagnostic parsed from `rustc` output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerDiagnostic {
    pub level: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub code: Option<String>,
}

/// The outcome of invoking `rustc` or `cargo`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompileReport {
    pub success: bool,
    pub exit_code: i32,
    pub diagnostics: Vec<CompilerDiagnostic>,
    pub stdout: String,
    pub stderr: String,
}

/// An exact surgical patch operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchOperation {
    pub file: PathBuf,
    pub search: String,
    pub replacement: String,
}

/// Record of an individual compile-repair iteration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepairIteration {
    pub iteration: usize,
    pub diagnostic_addressed: Option<CompilerDiagnostic>,
    pub patch_applied: Option<PatchOperation>,
    pub compile_success: bool,
}

/// Comprehensive provenance and verification report for a workspace coding session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceCodingReport {
    pub task_id: String,
    pub initial_revision: WorkspaceRevision,
    pub final_revision: WorkspaceRevision,
    pub source_context: Vec<String>,
    pub iterations: Vec<RepairIteration>,
    pub final_compile_success: bool,
    pub final_execution_success: bool,
    pub execution_output: String,
}

/// Engine executing bounded compilation, diagnostic parsing, and iterative repair.
pub struct WorkspaceCodingEngine;

impl WorkspaceCodingEngine {
    /// Parse `rustc` diagnostic output from stderr into structured `CompilerDiagnostic`s.
    pub fn parse_diagnostics(stderr: &str) -> Vec<CompilerDiagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = stderr.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("error[") || line.starts_with("error:") {
                let level = "error".to_string();
                let (code, message) = if let Some(bracket_idx) = line.find(']') {
                    let code_str = line[6..bracket_idx].to_string();
                    let msg = line[bracket_idx + 1..]
                        .trim_start_matches(':')
                        .trim()
                        .to_string();
                    (Some(code_str), msg)
                } else {
                    let msg = line.trim_start_matches("error:").trim().to_string();
                    (None, msg)
                };

                let mut file = None;
                let mut line_num = None;
                let mut col_num = None;

                // Check next line for location pointer (--> src/file.rs:10:5)
                if i + 1 < lines.len() {
                    let next_line = lines[i + 1].trim();
                    if next_line.starts_with("-->") {
                        let loc_str = next_line.trim_start_matches("-->").trim();
                        let parts: Vec<&str> = loc_str.split(':').collect();
                        if parts.len() >= 3 {
                            file = Some(parts[0].to_string());
                            line_num = parts[1].parse::<usize>().ok();
                            col_num = parts[2].parse::<usize>().ok();
                        } else if parts.len() >= 1 {
                            file = Some(parts[0].to_string());
                        }
                    }
                }

                diagnostics.push(CompilerDiagnostic {
                    level,
                    message,
                    file,
                    line: line_num,
                    column: col_num,
                    code,
                });
            }
            i += 1;
        }

        diagnostics
    }

    /// Compile a single Rust source file into an executable binary using `rustc --edition=2021`.
    pub fn compile_single(src_path: &Path, bin_path: &Path) -> Result<CompileReport> {
        let output = Command::new("rustc")
            .args([
                "--edition=2021",
                src_path
                    .to_str()
                    .ok_or_else(|| Error("Invalid src path".into()))?,
                "-o",
                bin_path
                    .to_str()
                    .ok_or_else(|| Error("Invalid bin path".into()))?,
            ])
            .bounded_output()
            .map_err(|e| Error(format!("Failed to execute rustc: {}", e)))?;

        let success = output.status.success();
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let diagnostics = Self::parse_diagnostics(&stderr);

        Ok(CompileReport {
            success,
            exit_code,
            diagnostics,
            stdout,
            stderr,
        })
    }

    /// Compile a multi-file Rust library and binary together.
    pub fn compile_multi_file(
        lib_src: &Path,
        lib_rlib: &Path,
        bin_src: &Path,
        bin_path: &Path,
        crate_name: &str,
    ) -> Result<CompileReport> {
        // Step 1: Compile library crate as rlib
        let lib_output = Command::new("rustc")
            .args([
                "--edition=2021",
                "--crate-type=rlib",
                "--crate-name",
                crate_name,
                lib_src
                    .to_str()
                    .ok_or_else(|| Error("Invalid lib path".into()))?,
                "-o",
                lib_rlib
                    .to_str()
                    .ok_or_else(|| Error("Invalid rlib path".into()))?,
            ])
            .bounded_output()
            .map_err(|e| Error(format!("Failed to execute rustc for lib: {}", e)))?;

        if !lib_output.status.success() {
            let stderr = String::from_utf8_lossy(&lib_output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&lib_output.stdout).to_string();
            return Ok(CompileReport {
                success: false,
                exit_code: lib_output.status.code().unwrap_or(-1),
                diagnostics: Self::parse_diagnostics(&stderr),
                stdout,
                stderr,
            });
        }

        // Step 2: Compile binary linking with the compiled library
        let bin_output = Command::new("rustc")
            .args([
                "--edition=2021",
                "--extern",
                &format!(
                    "{}={}",
                    crate_name,
                    lib_rlib
                        .to_str()
                        .ok_or_else(|| Error("Invalid library output path".into()))?
                ),
                bin_src
                    .to_str()
                    .ok_or_else(|| Error("Invalid bin src path".into()))?,
                "-o",
                bin_path
                    .to_str()
                    .ok_or_else(|| Error("Invalid bin out path".into()))?,
            ])
            .bounded_output()
            .map_err(|e| Error(format!("Failed to execute rustc for bin: {}", e)))?;

        let success = bin_output.status.success();
        let exit_code = bin_output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&bin_output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&bin_output.stderr).to_string();
        let diagnostics = Self::parse_diagnostics(&stderr);

        Ok(CompileReport {
            success,
            exit_code,
            diagnostics,
            stdout,
            stderr,
        })
    }

    /// Execute a compiled binary and verify that it exits successfully with code 0.
    pub fn execute_binary(bin_path: &Path) -> Result<String> {
        let output = Command::new(bin_path)
            .bounded_output()
            .map_err(|e| Error(format!("Failed to run binary {:?}: {}", bin_path, e)))?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error(format!(
                "Binary exited with failure status {}: {}",
                code, stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Execute an iterative compile-and-repair loop on a target file in the workspace.
    ///
    /// `bin_path` must point outside the workspace root: [`WorkspaceEnvironment::compute_revision`]
    /// rejects any tracked file above [`MAX_WORKSPACE_FILE_BYTES`], and a default `rustc` binary
    /// exceeds that 1 MiB limit on Linux.
    pub fn iterative_repair<F>(
        task_id: &str,
        workspace: &WorkspaceEnvironment,
        target_file: &Path,
        bin_path: &Path,
        max_iterations: usize,
        mut patch_generator: F,
    ) -> Result<WorkspaceCodingReport>
    where
        F: FnMut(&CompilerDiagnostic, &str) -> Option<PatchOperation>,
    {
        if max_iterations > 16 {
            return Err(Error("Repair iteration limit is 16".into()));
        }
        let initial_revision = workspace.compute_revision()?;
        let mut source_context = Vec::new();
        source_context.push(target_file.to_string_lossy().to_string());

        let mut iterations = Vec::new();
        let full_target_path = workspace.resolve_path(target_file)?;

        let mut final_compile_success = false;
        let mut final_execution_success = false;
        let mut execution_output = String::new();

        for iter_idx in 0..=max_iterations {
            let compile_report = Self::compile_single(&full_target_path, bin_path)?;

            if compile_report.success {
                final_compile_success = true;
                // Test run the compiled binary
                match Self::execute_binary(bin_path) {
                    Ok(stdout) => {
                        final_execution_success = true;
                        execution_output = stdout;
                    }
                    Err(e) => {
                        execution_output = format!("Execution error: {}", e);
                    }
                }
                iterations.push(RepairIteration {
                    iteration: iter_idx,
                    diagnostic_addressed: None,
                    patch_applied: None,
                    compile_success: true,
                });
                break;
            }

            if iter_idx == max_iterations {
                // Reached max iterations without passing
                iterations.push(RepairIteration {
                    iteration: iter_idx,
                    diagnostic_addressed: compile_report.diagnostics.first().cloned(),
                    patch_applied: None,
                    compile_success: false,
                });
                break;
            }

            // Ingest diagnostic
            let diagnostic =
                compile_report.diagnostics.first().cloned().ok_or_else(|| {
                    Error("Compilation failed but no diagnostic was parsed".into())
                })?;

            let current_content = workspace.read_file(target_file)?;
            if let Some(patch) = patch_generator(&diagnostic, &current_content) {
                workspace.apply_patch(&patch.file, &patch.search, &patch.replacement)?;
                iterations.push(RepairIteration {
                    iteration: iter_idx,
                    diagnostic_addressed: Some(diagnostic),
                    patch_applied: Some(patch),
                    compile_success: false,
                });
            } else {
                // Generator could not synthesize a patch
                iterations.push(RepairIteration {
                    iteration: iter_idx,
                    diagnostic_addressed: Some(diagnostic),
                    patch_applied: None,
                    compile_success: false,
                });
                break;
            }
        }

        let final_revision = workspace.compute_revision()?;

        Ok(WorkspaceCodingReport {
            task_id: task_id.to_string(),
            initial_revision,
            final_revision,
            source_context,
            iterations,
            final_compile_success,
            final_execution_success,
            execution_output,
        })
    }
}
