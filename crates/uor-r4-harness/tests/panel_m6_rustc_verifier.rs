//! Finite integration tests for the frozen M6 CodeTasks Rust verifier.

use uor_r4_harness::panels::m6_reasoning::generate_frozen_m6_panel;
use uor_r4_harness::panels::m6_rustc_verifier::{
    verify_all_frozen_code_tasks, verify_row, VerifyOutcome,
};

#[test]
fn passing_source_compiles_and_passes() {
    let source = r#"
pub fn solve(x: i64) -> i64 { x + 1 }

#[test]
fn verify_vectors() {
    assert_eq!(solve(1i64), 2i64);
    assert_eq!(solve(10i64), 11i64);
}
"#;
    let result = verify_row("test-pass", source);
    assert!(
        matches!(result.outcome, VerifyOutcome::Pass { passed: 1 }),
        "expected one passing test, got {:?}",
        result.outcome
    );
}

#[test]
fn compile_error_is_reported() {
    let result = verify_row("test-compile-err", "this is not valid rust");
    assert!(
        matches!(result.outcome, VerifyOutcome::CompileError { .. }),
        "expected CompileError, got {:?}",
        result.outcome
    );
}

#[test]
fn failing_assertion_is_reported() {
    let source = r#"
pub fn solve(x: i64) -> i64 { x }

#[test]
fn verify_vectors() {
    assert_eq!(solve(1i64), 999i64);
}
"#;
    let result = verify_row("test-fail", source);
    assert!(
        matches!(result.outcome, VerifyOutcome::TestFailure { .. }),
        "expected TestFailure, got {:?}",
        result.outcome
    );
}

#[test]
fn timeout_is_reported() {
    let source = r#"
pub fn solve(_x: i64) -> i64 { loop {} }

#[test]
fn verify_vectors() {
    solve(0);
}
"#;
    let result = verify_row("test-timeout", source);
    assert!(
        matches!(result.outcome, VerifyOutcome::Timeout { phase: "run" }),
        "expected a run timeout, got {:?}",
        result.outcome
    );
}

#[test]
fn success_without_a_libtest_result_fails_closed() {
    let source = r#"
#[test]
fn verify_vectors() {
    std::process::exit(0);
}
"#;
    let result = verify_row("test-no-result", source);
    assert!(
        matches!(result.outcome, VerifyOutcome::UnexpectedOutput { .. }),
        "a successful process without libtest output must not pass, got {:?}",
        result.outcome
    );
}

#[test]
fn zero_test_source_fails_closed() {
    let source = r#"
pub fn solve(x: i64) -> i64 { x }
"#;
    let result = verify_row("test-zero", source);
    assert!(
        matches!(result.outcome, VerifyOutcome::TestFailure { .. }),
        "a source with no tests must not pass, got {:?}",
        result.outcome
    );
}

#[test]
fn all_80_frozen_rows_pass() {
    generate_frozen_m6_panel()
        .verify_zero_overlap()
        .expect("the frozen panel and sandbox oracle remain valid");
    let results = verify_all_frozen_code_tasks();
    assert_eq!(results.len(), 80, "expected 80 CodeTasks rows");
    let failures: Vec<_> = results
        .iter()
        .filter(|row| !matches!(row.outcome, VerifyOutcome::Pass { passed: 1 }))
        .collect();
    assert!(failures.is_empty(), "frozen rows failed: {failures:#?}");
    assert!(
        results.iter().all(|row| row.row_id.starts_with("m6-code-")),
        "every result must be a frozen CodeTasks row"
    );
}
