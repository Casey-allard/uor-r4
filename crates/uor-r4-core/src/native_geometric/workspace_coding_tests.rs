//! Tests for executable Rust coding and controlled workspace use (#1088).

use super::workspace_coding::*;
use std::path::{Path, PathBuf};

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        if self.0.is_dir() {
            let _ = std::fs::remove_dir_all(&self.0);
        } else {
            let _ = std::fs::remove_file(&self.0);
        }
    }
}

fn setup_temp_workspace(test_name: &str) -> (TempDirGuard, WorkspaceEnvironment) {
    let now_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir_path = std::env::temp_dir().join(format!(
        "uor_r4_{}_{}_{}",
        test_name,
        std::process::id(),
        now_nanos
    ));
    std::fs::create_dir_all(&dir_path).expect("create tempdir");
    let env = WorkspaceEnvironment::new(dir_path.clone());
    (TempDirGuard(dir_path), env)
}

/// A compiled-binary path outside every workspace root.
///
/// A workspace tracks every file under its root and rejects any above
/// `MAX_WORKSPACE_FILE_BYTES` when it computes a revision. A default `rustc` binary exceeds that
/// 1 MiB limit on Linux, so compiled test artifacts must be created outside the workspace.
fn setup_temp_binary(test_name: &str) -> (TempDirGuard, PathBuf) {
    let now_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!(
        "uor_r4_bin_{}_{}_{}",
        test_name,
        std::process::id(),
        now_nanos
    ));
    (TempDirGuard(path.clone()), path)
}

#[test]
fn native_workspace_file_read_write_patch() {
    let (_dir, env) = setup_temp_workspace("file_ops");

    // 1. Write file in nested path
    let rel_path = Path::new("src/lib.rs");
    env.write_file(rel_path, "pub fn original_fn() -> i32 { 1 }\n")
        .expect("write_file");

    // 2. Read file
    let content = env.read_file(rel_path).expect("read_file");
    assert_eq!(content, "pub fn original_fn() -> i32 { 1 }\n");

    // 3. Compute initial revision
    let rev1 = env.compute_revision().expect("compute_revision");
    assert_eq!(rev1.file_count, 1);
    assert!(rev1.total_bytes > 0);

    // 4. Apply patch
    env.apply_patch(rel_path, "1", "42").expect("apply_patch");
    let updated = env.read_file(rel_path).expect("read_file after patch");
    assert_eq!(updated, "pub fn original_fn() -> i32 { 42 }\n");

    // 5. Revision changes deterministically
    let rev2 = env.compute_revision().expect("compute_revision");
    assert_ne!(rev1.revision_id, rev2.revision_id);

    // 6. Path traversal rejection
    let traversal_path = Path::new("../escaped.rs");
    assert!(env.write_file(traversal_path, "malicious").is_err());
    assert!(env.read_file(traversal_path).is_err());
}

#[test]
fn native_workspace_single_file_synthesis_and_run() {
    let (_dir, env) = setup_temp_workspace("synthesis_run");

    let src_path = Path::new("src/main.rs");
    let code = r#"
fn main() {
    let factor_alpha: i64 = 17;
    let factor_beta: i64 = 3;
    let result = factor_alpha * factor_beta;
    assert_eq!(result, 51);
    println!("VERIFIED:{}", result);
}
"#;
    env.write_file(src_path, code).expect("write code");

    let full_src = env.root().join(src_path);
    let bin_path = env.root().join("target_bin");

    let report =
        WorkspaceCodingEngine::compile_single(&full_src, &bin_path).expect("compile single");

    assert!(
        report.success,
        "Compilation must succeed: {}",
        report.stderr
    );
    assert_eq!(report.exit_code, 0);

    let output = WorkspaceCodingEngine::execute_binary(&bin_path).expect("execute binary");

    assert!(output.contains("VERIFIED:51"));
}

#[test]
fn native_workspace_compiler_feedback_diagnosis() {
    let (_dir, env) = setup_temp_workspace("diagnosis");

    let src_path = Path::new("src/main.rs");
    // Deliberate type error: assigning &str to i64
    let broken_code = r#"
fn main() {
    let value: i64 = "type_mismatch_error";
    println!("{}", value);
}
"#;
    env.write_file(src_path, broken_code)
        .expect("write broken code");

    let full_src = env.root().join(src_path);
    let bin_path = env.root().join("target_bin");

    let report =
        WorkspaceCodingEngine::compile_single(&full_src, &bin_path).expect("compile single");

    assert!(!report.success, "Compilation must fail");
    assert!(!report.diagnostics.is_empty(), "Must parse diagnostics");

    let diag = &report.diagnostics[0];
    assert_eq!(diag.level, "error");
    assert_eq!(diag.code.as_deref(), Some("E0308"));
    assert_eq!(diag.line, Some(3));
}

#[test]
fn native_workspace_iterative_compiler_repair() {
    let (_dir, env) = setup_temp_workspace("iterative_repair");

    let src_path = Path::new("src/main.rs");
    let broken_code = r#"
fn main() {
    let computation_total: i64 = "wrong_operand";
    assert_eq!(computation_total, 100);
    println!("REPAIRED:{}", computation_total);
}
"#;
    env.write_file(src_path, broken_code)
        .expect("write broken code");
    let (_bin_guard, bin_path) = setup_temp_binary("iterative_repair");

    let report = WorkspaceCodingEngine::iterative_repair(
        "task-repair-type-mismatch",
        &env,
        src_path,
        &bin_path,
        3,
        |diag, current_code| {
            // Model repair generator: inspects diagnostic and proposes targeted patch
            if diag.code.as_deref() == Some("E0308") && current_code.contains("\"wrong_operand\"") {
                Some(PatchOperation {
                    file: PathBuf::from("src/main.rs"),
                    search: "\"wrong_operand\"".to_string(),
                    replacement: "100".to_string(),
                })
            } else {
                None
            }
        },
    )
    .expect("iterative_repair execution");

    assert!(report.final_compile_success, "Final compile must succeed");
    assert!(
        report.final_execution_success,
        "Final execution must succeed"
    );
    assert!(report.execution_output.contains("REPAIRED:100"));
    assert_eq!(report.iterations.len(), 2); // iteration 0 (fail -> patch) + iteration 1 (success)
    assert_ne!(
        report.initial_revision.revision_id,
        report.final_revision.revision_id
    );
    assert_eq!(
        report.final_revision.file_count, 1,
        "the compiled binary must live outside the workspace and not be tracked in the revision"
    );
}

#[test]
fn native_workspace_multi_file_interface_repair() {
    let (_dir, env) = setup_temp_workspace("multi_file");

    // 1. Library file: exports `compute_product`
    let lib_path = Path::new("src/math.rs");
    let lib_code = r#"
pub fn compute_product(left: i64, right: i64) -> i64 {
    left * right
}
"#;
    env.write_file(lib_path, lib_code).expect("write lib");

    // 2. Binary file: attempts to call non-existent `compute_mult`
    let bin_src_path = Path::new("src/main.rs");
    let broken_main = r#"
fn main() {
    let total = math::compute_mult(6, 7);
    assert_eq!(total, 42);
    println!("MULTI_FILE_PASS:{}", total);
}
"#;
    env.write_file(bin_src_path, broken_main)
        .expect("write main");

    let full_lib = env.root().join(lib_path);
    let full_main = env.root().join(bin_src_path);
    let lib_rlib = env.root().join("libmath.rlib");
    let bin_path = env.root().join("multi_bin");

    // Initial compilation fails due to missing function `compute_mult`
    let initial_report = WorkspaceCodingEngine::compile_multi_file(
        &full_lib, &lib_rlib, &full_main, &bin_path, "math",
    )
    .expect("compile multi file");

    assert!(!initial_report.success);
    assert!(initial_report
        .stderr
        .contains("cannot find function `compute_mult`"));

    // Apply interface repair patch to align with lib interface
    env.apply_patch(bin_src_path, "compute_mult", "compute_product")
        .expect("apply interface repair patch");

    // Second compilation succeeds
    let repaired_report = WorkspaceCodingEngine::compile_multi_file(
        &full_lib, &lib_rlib, &full_main, &bin_path, "math",
    )
    .expect("compile multi file after repair");

    assert!(
        repaired_report.success,
        "Must compile after repair: {}",
        repaired_report.stderr
    );

    let output =
        WorkspaceCodingEngine::execute_binary(&bin_path).expect("execute multi file binary");

    assert!(output.contains("MULTI_FILE_PASS:42"));
}

#[test]
fn native_workspace_iteration_limit_enforcement() {
    let (_dir, env) = setup_temp_workspace("limit_enforcement");

    let src_path = Path::new("src/main.rs");
    // Persistent unfixable error
    let unfixable_code = r#"
fn main() {
    compile_error_that_cannot_be_fixed();
}
"#;
    env.write_file(src_path, unfixable_code)
        .expect("write unfixable code");
    let bin_path = env.root().join("unfixable_bin");

    let report = WorkspaceCodingEngine::iterative_repair(
        "task-unfixable-limit",
        &env,
        src_path,
        &bin_path,
        2,                   // max 2 iterations
        |_diag, _code| None, // No patch produced
    )
    .expect("iterative_repair execution");

    assert!(!report.final_compile_success, "Must not succeed");
    assert!(!report.final_execution_success, "Must not execute");
    assert_eq!(report.iterations.len(), 1); // Exits immediately when no patch is produced
    assert_eq!(report.iterations[0].compile_success, false);
}

#[test]
fn native_workspace_provenance_and_revision_binding() {
    let (_dir, env) = setup_temp_workspace("provenance");

    let src_path = Path::new("src/main.rs");
    let code = r#"
fn main() {
    let val: i64 = "needs_fix";
    assert_eq!(val, 999);
}
"#;
    env.write_file(src_path, code).expect("write code");
    let (_bin_guard, bin_path) = setup_temp_binary("provenance");

    let report = WorkspaceCodingEngine::iterative_repair(
        "task-provenance-binding-999",
        &env,
        src_path,
        &bin_path,
        3,
        |_diag, _code| {
            Some(PatchOperation {
                file: PathBuf::from("src/main.rs"),
                search: "\"needs_fix\"".into(),
                replacement: "999".into(),
            })
        },
    )
    .expect("iterative_repair execution");

    assert_eq!(report.task_id, "task-provenance-binding-999");
    assert_eq!(report.source_context, vec!["src/main.rs"]);
    assert_ne!(
        report.initial_revision.revision_id,
        report.final_revision.revision_id
    );
    assert_eq!(
        report.final_revision.file_count, 1,
        "the compiled binary must live outside the workspace and not be tracked in the revision"
    );
    assert!(report.final_compile_success);
    assert!(report.final_execution_success);

    // Verify iteration provenance
    assert_eq!(report.iterations.len(), 2);
    let iter0 = &report.iterations[0];
    assert!(!iter0.compile_success);
    assert!(iter0.diagnostic_addressed.is_some());
    let diag = iter0.diagnostic_addressed.as_ref().unwrap();
    assert_eq!(diag.code.as_deref(), Some("E0308"));
    assert_eq!(diag.line, Some(3));

    let patch = iter0.patch_applied.as_ref().unwrap();
    assert_eq!(patch.search, "\"needs_fix\"");
    assert_eq!(patch.replacement, "999");

    let iter1 = &report.iterations[1];
    assert!(iter1.compile_success);
}

#[test]
fn workspace_rejects_absolute_paths_and_oversized_files() {
    let (_guard, env) = setup_temp_workspace("limits");
    assert!(env
        .write_file(&env.root().join("absolute.rs"), "x")
        .is_err());
    assert!(env.read_file(Path::new("/etc/hosts")).is_err());
    let oversized = "x".repeat(MAX_WORKSPACE_FILE_BYTES + 1);
    assert!(env.write_file(Path::new("too-large"), &oversized).is_err());
    std::fs::write(env.root().join("existing-large"), oversized).unwrap();
    assert!(env.read_file(Path::new("existing-large")).is_err());
    assert!(env.compute_revision().is_err());
}

#[cfg(unix)]
#[test]
fn workspace_rejects_symlink_escape() {
    let (_inside, env) = setup_temp_workspace("symlink_inside");
    let (_outside, outside) = setup_temp_workspace("symlink_outside");
    outside.write_file(Path::new("secret"), "preserve").unwrap();
    std::os::unix::fs::symlink(outside.root(), env.root().join("alias")).unwrap();
    assert!(env.read_file(Path::new("alias/secret")).is_err());
    assert!(env
        .write_file(Path::new("alias/new"), "must not write")
        .is_err());
    assert!(env.list_files().is_err());
    assert!(!outside.root().join("new").exists());
    assert_eq!(outside.read_file(Path::new("secret")).unwrap(), "preserve");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn workspace_host_program_has_a_deadline() {
    let (_guard, env) = setup_temp_workspace("deadline");
    env.write_file(
        Path::new("main.rs"),
        "fn main() { std::thread::sleep(std::time::Duration::from_secs(60)); }",
    )
    .unwrap();
    let binary = env.root().join("program");
    assert!(
        WorkspaceCodingEngine::compile_single(&env.root().join("main.rs"), &binary)
            .unwrap()
            .success
    );
    let start = std::time::Instant::now();
    let error = WorkspaceCodingEngine::execute_binary(&binary).unwrap_err();
    assert!(error.0.contains("deadline"));
    assert!(start.elapsed() < PROCESS_TIMEOUT + std::time::Duration::from_secs(3));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn workspace_host_program_output_is_bounded() {
    let (_guard, env) = setup_temp_workspace("output_limit");
    env.write_file(Path::new("main.rs"), "fn main() { use std::io::Write; let _ = std::io::stdout().write_all(&vec![b'x'; 2 * 1024 * 1024]); }").unwrap();
    let binary = env.root().join("program");
    assert!(
        WorkspaceCodingEngine::compile_single(&env.root().join("main.rs"), &binary)
            .unwrap()
            .success
    );
    let error = WorkspaceCodingEngine::execute_binary(&binary).unwrap_err();
    assert!(error.0.contains("output limit"));
}
