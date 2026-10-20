//! Structural cross-ref A.1: every core sealed type appears in
//! `rust/escape_hatch_lint`'s `SEALED_TYPES`.
//!
//! The authoritative sealed-type set is maintained as a static snapshot
//! below. Any PR that seals a new witness type updates this list
//! alongside `SEALED_TYPES` in `escape_hatch_lint`; this validator
//! cross-checks the two so lint coverage can't drift away from the
//! intended seal surface.

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/target_doc/sealed_type_coverage";

/// Authoritative snapshot of the core sealed-type surface. Updates to
/// this list are reviewed PRs paired with a matching edit to
/// `escape_hatch_lint::SEALED_TYPES`.
const REQUIRED_SEALED_TYPES: &[&str] = &[
    // Core sealed witnesses.
    "Datum",
    "Validated",
    "Grounded",
    "Certified",
    "Triad",
    "Derivation",
    "FreeRank",
    "BoundarySession",
    "BindingsTable",
    // UorTime surface.
    "UorTime",
    "LandauerBudget",
    "Stratum",
    "ContentAddress",
    "Nanos",
];

/// Runs the sealed-type cross-reference check.
///
/// # Errors
///
/// Returns an error if the lint source cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let lint_path = workspace.join("conformance/src/validators/rust/escape_hatch_lint.rs");

    let lint_src = match fs::read_to_string(&lint_path) {
        Ok(s) => s,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", lint_path.display()),
            ));
            return Ok(report);
        }
    };

    let missing: Vec<String> = REQUIRED_SEALED_TYPES
        .iter()
        .filter(|ty| !lint_mentions_type(&lint_src, ty))
        .map(|s| (*s).to_string())
        .collect();

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "sealed-type coverage: all {} core seal types present in escape_hatch_lint::SEALED_TYPES",
                REQUIRED_SEALED_TYPES.len()
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "sealed-type coverage: {} required seal types missing from escape_hatch_lint::SEALED_TYPES",
                missing.len()
            ),
            missing,
        ));
    }
    Ok(report)
}

/// Does the escape-hatch lint source mention the type name as a quoted
/// string entry (inside `SEALED_TYPES`)? Scan for the literal `"<Name>"`.
fn lint_mentions_type(lint_src: &str, type_name: &str) -> bool {
    let quoted = format!("\"{type_name}\"");
    lint_src.contains(&quoted)
}
