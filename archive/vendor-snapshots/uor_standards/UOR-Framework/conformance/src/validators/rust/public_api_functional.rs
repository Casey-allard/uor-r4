//! v0.2.2 T2.0 (cleanup): public API functional verification gate.
//!
//! Shells to two `cargo test --test ...` invocations and asserts both
//! exit 0:
//!
//! 1. `cargo test -p uor-foundation --test public_api_e2e` — the
//!    foundation's end-to-end test exercising every previously-hardcoded
//!    public endpoint with input-dependence assertions.
//! 2. `cargo test -p uor-foundation-verify --test round_trip` — the
//!    uor-foundation-verify crate's round-trip tests using the
//!    test-helpers crate.
//!
//! Two passing checks contribute to the conformance suite gate.

use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const FOUNDATION_E2E: &str = "rust/public_api_functional/foundation_e2e";
const VERIFY_ROUND_TRIP: &str = "rust/public_api_functional/verify_round_trip";

/// Runs the public-API functional test gate.
///
/// # Errors
///
/// Returns an error if cargo cannot be launched.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    // 1. Foundation end-to-end test binary.
    let output = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .args([
            "test",
            "-p",
            "uor-foundation",
            "--test",
            "public_api_e2e",
            "--quiet",
        ])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            report.push(TestResult::pass(
                FOUNDATION_E2E,
                "uor-foundation public_api_e2e exits 0 (all hardcoded endpoints \
                 functional and input-dependent)",
            ));
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            report.push(TestResult::fail(
                FOUNDATION_E2E,
                format!("public_api_e2e failed: {stderr}"),
            ));
        }
        Err(e) => {
            report.push(TestResult::fail(
                FOUNDATION_E2E,
                format!("failed to spawn cargo test: {e}"),
            ));
        }
    }

    // 2. uor-foundation-verify round-trip test binary.
    let output = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .args([
            "test",
            "-p",
            "uor-foundation-verify",
            "--test",
            "round_trip",
            "--quiet",
        ])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            report.push(TestResult::pass(
                VERIFY_ROUND_TRIP,
                "uor-foundation-verify round_trip exits 0 (verify_trace round-trips \
                 against test-helpers-constructed Traces)",
            ));
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            report.push(TestResult::fail(
                VERIFY_ROUND_TRIP,
                format!("round_trip failed: {stderr}"),
            ));
        }
        Err(e) => {
            report.push(TestResult::fail(
                VERIFY_ROUND_TRIP,
                format!("failed to spawn cargo test: {e}"),
            ));
        }
    }

    Ok(report)
}
