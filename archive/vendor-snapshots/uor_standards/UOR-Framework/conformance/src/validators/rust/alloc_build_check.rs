//! v0.2.2 Phase H validator: alloc-featured build check.
//!
//! Shells to `cargo check -p uor-foundation --no-default-features
//! --features alloc` and asserts exit 0.

use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/alloc_build_check";

/// Runs the alloc-featured build check.
///
/// # Errors
///
/// Returns an error if the cargo command cannot be launched.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let output = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .args([
            "check",
            "-p",
            "uor-foundation",
            "--no-default-features",
            "--features",
            "alloc",
            "--quiet",
        ])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            report.push(TestResult::pass(
                VALIDATOR,
                "uor-foundation builds cleanly with --features alloc",
            ));
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            report.push(TestResult::fail(
                VALIDATOR,
                format!("uor-foundation alloc build failed: {stderr}"),
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
