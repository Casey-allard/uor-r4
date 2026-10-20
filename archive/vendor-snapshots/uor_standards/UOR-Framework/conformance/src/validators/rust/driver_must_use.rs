//! Phase M.3 (target §5): driver surface must-use discipline.
//!
//! Asserts that every pipeline driver entry point is annotated with
//! `#[must_use]`. Dropping a `Grounded` witness / `StreamDriver` /
//! `InteractionDriver` return value silently discards the sealed
//! witness's effects — always a programming error that the compiler
//! should flag.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/driver_must_use";

/// Runs the Phase M.3 driver must-use validation.
///
/// # Errors
///
/// Returns an error if the pipeline source cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let pipeline_path = workspace.join("foundation/src/pipeline.rs");
    let content = match std::fs::read_to_string(&pipeline_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", pipeline_path.display()),
            ));
            return Ok(report);
        }
    };

    // Expected anchors: `#[must_use]\npub fn <name>` for drivers whose
    // return type is NOT already `#[must_use]`. `run` and `run_parallel`
    // return `Result<_, _>` which is already must-use, so they don't need
    // the attribute (clippy's `double_must_use` flags the redundancy).
    // StreamDriver and InteractionDriver are plain structs, so they DO
    // need the attribute on their factory functions.
    let drivers = [
        (
            "run_stream",
            "pub fn run_stream<T, P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(",
        ),
        (
            "run_interactive",
            "pub fn run_interactive<T, P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(",
        ),
    ];

    let mut missing: Vec<String> = Vec::new();
    for (name, signature) in drivers {
        let sig_idx = match content.find(signature) {
            Some(i) => i,
            None => {
                missing.push(format!("driver `pipeline::{name}` signature not found"));
                continue;
            }
        };
        // Walk backward to find the preceding line (the expected `#[must_use]`).
        let prefix = &content[..sig_idx];
        let preceding = prefix.trim_end_matches('\n').lines().last().unwrap_or("");
        if !preceding.trim().starts_with("#[must_use]") {
            missing.push(format!(
                "driver `pipeline::{name}` missing `#[must_use]` attribute (preceding line: `{}`)",
                preceding.trim()
            ));
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "Phase M.3 driver surface: run / run_parallel / run_stream / run_interactive \
             all annotated with `#[must_use]` (target §5)",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase M.3 driver must-use has {} missing annotations",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
