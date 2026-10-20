//! v0.2.2 Phase H validator: all-features build check.
//!
//! Shells to `cargo check -p uor-foundation --all-features` and asserts
//! exit 0. Confirms that the union of all feature gates compiles cleanly.

use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/all_features_build_check";

/// Runs the all-features build check.
///
/// # Errors
///
/// Returns an error if the cargo command cannot be launched.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let output = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .args(["check", "-p", "uor-foundation", "--all-features", "--quiet"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            report.push(TestResult::pass(
                VALIDATOR,
                "uor-foundation builds cleanly with --all-features",
            ));
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            report.push(TestResult::fail(
                VALIDATOR,
                format!("uor-foundation all-features build failed: {stderr}"),
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
