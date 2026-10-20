//! Lean 4 build validator.
//!
//! Runs `lake build` in the `lean4/` directory to verify that all generated
//! Lean 4 code compiles successfully. This is a stronger check than text
//! pattern matching — it proves the generated code is well-typed.

use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "lean4/build";

/// Validates that the generated Lean 4 code compiles via `lake build`.
///
/// # Errors
///
/// Returns an error if the lean4 directory does not exist or if an
/// unexpected I/O error occurs (other than `lake` not being installed).
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let lean_dir = workspace.join("lean4");

    if !lean_dir.exists() {
        report.push(TestResult::fail(VALIDATOR, "lean4/ directory not found"));
        return Ok(report);
    }

    match Command::new("lake")
        .arg("build")
        .current_dir(workspace)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                report.push(TestResult::pass(
                    VALIDATOR,
                    "lake build succeeded \u{2014} all generated Lean 4 code compiles",
                ));
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let lines: Vec<String> = stderr.lines().take(50).map(|l| l.to_string()).collect();
                report.push(TestResult::fail_with_details(
                    VALIDATOR,
                    "lake build failed \u{2014} generated Lean 4 code has compilation errors",
                    lines,
                ));
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            report.push(TestResult::fail(
                VALIDATOR,
                "lake not found \u{2014} install Lean 4 via elan: \
                 curl https://elan.lean-lang.org/elan-init.sh -sSf | sh",
            ));
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to run lake build: {e}"));
        }
    }

    Ok(report)
}
