//! v0.2.2 Phase H validator: strictly no_std build check.
//!
//! Shells to `cargo check -p uor-foundation --no-default-features` and
//! asserts exit 0. The foundation crate is no_std by default per the v0.2.1
//! invariants; this check fails loudly if any regression introduces an
//! unconditional std or alloc dependency.

use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/no_std_build_check";

/// Runs the no_std build check.
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
            "--quiet",
        ])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            report.push(TestResult::pass(
                VALIDATOR,
                "uor-foundation builds cleanly with --no-default-features",
            ));
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            report.push(TestResult::fail(
                VALIDATOR,
                format!("uor-foundation no_std build failed: {stderr}"),
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
