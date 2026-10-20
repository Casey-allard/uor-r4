//! Layer 3: test-quality gate.
//!
//! Scans `foundation/tests/behavior_*.rs` for anti-patterns that indicate
//! a test is ceremonial rather than behavioral. Fails the conformance run
//! if any anti-pattern appears:
//!
//! - `fn _takes<...>(_: Type) {}` — addressability-only no-op. No behavioral assertion.
//! - `let _ = <call>` — discarded return value from a contract-bearing call.
//! - A `#[test]` function body containing zero `assert!` / `assert_eq!` /
//!   `assert_ne!` / `.expect(` / `.unwrap(` calls.
//!
//! Only `behavior_*.rs` is scanned; legacy `phase_*.rs` tests and other
//! test files are exempt.

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/test_assertion_depth";

/// Runs the test-quality gate over `foundation/tests/behavior_*.rs`.
///
/// # Errors
///
/// Returns an error if the test directory cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let tests_dir = workspace.join("foundation/tests");
    let entries = match fs::read_dir(&tests_dir) {
        Ok(e) => e,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("cannot read {}: {e}", tests_dir.display()),
            ));
            return Ok(report);
        }
    };

    let mut violations: Vec<String> = Vec::new();
    let mut files_scanned = 0usize;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        // Only scan behavior_*.rs files.
        if !name.starts_with("behavior_") || !name.ends_with(".rs") {
            continue;
        }
        files_scanned += 1;
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Pattern 1: `fn _takes<...>(_: Type) {}` — addressability no-op.
        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            // Permissive match for the structural pattern: `fn _takes`
            // followed by an underscore-pattern binding and either a
            // unit-return-type empty body or no body at all.
            if trimmed.starts_with("fn _takes")
                && (trimmed.ends_with("{}") || trimmed.ends_with("{ }"))
            {
                violations.push(format!(
                    "{name}:{}: addressability no-op `fn _takes<...>(_: Type) {{}}` \u{2014} \
                     behavior tests must assert behavior, not presence",
                    line_no + 1
                ));
            }
        }

        // Pattern 2: `let _ = <expr>;` where `<expr>` is a raw discarded
        // function call without an `.expect(` or `.unwrap(` check along
        // the chain. The full expression can span multiple lines (rustfmt
        // breaks long builder chains), so we scan from the `let _ =`
        // anchor through the matching `;`.
        let mut i = 0usize;
        while i < content.len() {
            if let Some(rel) = content[i..].find("let _ =") {
                let start = i + rel;
                // Find the terminating `;` at depth 0 relative to start.
                let rest = &content[start..];
                let mut depth: i32 = 0;
                let mut end = rest.len();
                for (off, ch) in rest.char_indices() {
                    match ch {
                        '(' | '[' | '{' => depth += 1,
                        ')' | ']' | '}' => depth -= 1,
                        ';' if depth <= 0 => {
                            end = off;
                            break;
                        }
                        _ => {}
                    }
                }
                let expr = &rest[..end];
                let has_call = expr.contains('(');
                let has_check = expr.contains(".expect(")
                    || expr.contains(".unwrap(")
                    || expr.contains(".expect_err(")
                    || expr.contains(".unwrap_err(");
                if has_call && !has_check {
                    // Report the line where the `let _ =` starts.
                    let line_no = content[..start].chars().filter(|c| *c == '\n').count();
                    violations.push(format!(
                        "{name}:{}: `let _ = <call>` discards a contract-bearing return value \
                         \u{2014} inspect the result with assert!/expect/unwrap",
                        line_no + 1
                    ));
                }
                i = start + end + 1;
            } else {
                break;
            }
        }

        // Pattern 3: a `#[test]` function body with zero assertion calls.
        // We scan each top-level `fn <ident>()` that follows a `#[test]`
        // attribute line, and verify the body (until its matching `}`)
        // contains at least one assertion-family call. This is a rough
        // scan; false negatives (assertions inside helper calls) are OK
        // as long as the direct contract-bearing test body has some
        // assertion presence.
        let bytes = content.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            // Find `#[test]` anchor.
            if let Some(pos) = content[i..].find("#[test]") {
                let abs = i + pos;
                // Skip to next `fn ` after the attribute.
                if let Some(fn_pos) = content[abs..].find("\nfn ") {
                    let fn_abs = abs + fn_pos + 1; // point at `fn`
                                                   // Find the opening `{` of the function body.
                    if let Some(open_pos) = content[fn_abs..].find('{') {
                        let body_start = fn_abs + open_pos;
                        // Find the matching closing brace (naive — count
                        // depth).
                        let mut depth: i32 = 0;
                        let mut end = body_start;
                        for (off, ch) in content[body_start..].char_indices() {
                            if ch == '{' {
                                depth += 1;
                            }
                            if ch == '}' {
                                depth -= 1;
                                if depth == 0 {
                                    end = body_start + off;
                                    break;
                                }
                            }
                        }
                        let body = &content[body_start..=end];
                        // Extract the fn name (rough) for the violation
                        // message.
                        let fn_header = &content[fn_abs..body_start];
                        let fn_name: String = fn_header
                            .trim_start_matches("fn ")
                            .chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect();
                        // Check for assertion tokens.
                        let has_assert = body.contains("assert!")
                            || body.contains("assert_eq!")
                            || body.contains("assert_ne!")
                            || body.contains(".expect(")
                            || body.contains(".unwrap(")
                            || body.contains(".expect_err(")
                            || body.contains(".unwrap_err(");
                        if !has_assert {
                            violations.push(format!(
                                "{name}: #[test] fn {fn_name} has no assert!/assert_eq!/\
                                 assert_ne!/.expect/.unwrap in its body \u{2014} the test \
                                 asserts nothing",
                            ));
                        }
                        i = end + 1;
                        continue;
                    }
                }
            }
            break;
        }
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "test_assertion_depth: scanned {files_scanned} behavior_*.rs files, no \
                 addressability-only no-ops, discarded return values, or zero-assertion \
                 tests detected"
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "test_assertion_depth: {} anti-pattern violations in behavior_*.rs \
                 (ceremonial tests are not behavioral)",
                violations.len()
            ),
            violations,
        ));
    }

    Ok(report)
}
