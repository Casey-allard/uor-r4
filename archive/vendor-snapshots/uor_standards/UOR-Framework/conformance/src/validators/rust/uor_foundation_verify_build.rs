//! v0.2.2 Phase H validator: uor-foundation-verify crate build check.
//!
//! Shells to `cargo check -p uor-foundation-verify` and asserts exit 0.
//! The crate is a required v0.2.2 deliverable (Phase E trace replay) and
//! must build cleanly in its default (no_std) configuration.

use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/uor_foundation_verify_build";

/// Runs the uor-foundation-verify build check.
///
/// # Errors
///
/// Returns an error if the cargo command cannot be launched.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let output = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .args(["check", "-p", "uor-foundation-verify", "--quiet"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            report.push(TestResult::pass(
                VALIDATOR,
                "uor-foundation-verify builds cleanly",
            ));
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            report.push(TestResult::fail(
                VALIDATOR,
                format!("uor-foundation-verify build failed: {stderr}"),
            ));
        }
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to spawn cargo check: {e}"),
            ));
        }
    }

    Ok(report)
}
