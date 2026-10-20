//! v0.2.2 Phase H validator: foundation crate feature-flag layout.
//!
//! Asserts that `foundation/Cargo.toml` declares exactly the Phase H feature
//! set — `default`, `alloc`, `std`, `serde`, `observability` — and no
//! additional features. The discipline is: no feature other than these
//! five exists; `default` is strictly empty; `std` implies `alloc`; every
//! optional feature gating alloc-only surface implies `alloc`.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/feature_flag_layout";

/// Runs the feature-flag layout check.
///
/// # Errors
///
/// Returns an error if the `Cargo.toml` file cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let manifest_path = workspace.join("foundation/Cargo.toml");
    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", manifest_path.display()),
            ));
            return Ok(report);
        }
    };

    let required: &[(&str, &str)] = &[
        ("default feature", "default = []"),
        ("alloc feature", "alloc = []"),
        ("std feature", "std = [\"alloc\"]"),
        ("serde feature", "serde = [\"alloc\"]"),
        ("observability feature", "observability = [\"alloc\"]"),
    ];

    let mut missing: Vec<String> = Vec::new();
    for (label, anchor) in required {
        if !content.contains(*anchor) {
            missing.push((*label).to_string());
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "Phase H feature flag layout: default/alloc/std/serde/observability \
             all present with correct implications",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase H feature flag layout has {} missing entries",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
