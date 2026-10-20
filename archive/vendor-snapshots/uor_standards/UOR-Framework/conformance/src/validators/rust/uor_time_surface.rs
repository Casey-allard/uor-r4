//! v0.2.2 Phase A: `rust/uor_time_surface` validator.
//!
//! Asserts that the UorTime infrastructure is in the public-API snapshot
//! and that the foundation crate's `enforcement` module exposes the
//! expected six entries: `UorTime`, `LandauerBudget`, `Calibration`,
//! `CalibrationError`, `Nanos`, `calibrations` (the preset module).
//! Also asserts the four foundation calibration presets exist.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/uor_time_surface";

/// The Phase A surface entries that must appear in `enforcement.rs`.
/// Each tuple: `(scan-substring, human-readable name)`.
const REQUIRED_SYMBOLS: &[(&str, &str)] = &[
    ("pub struct UorTime", "UorTime"),
    ("pub struct LandauerBudget", "LandauerBudget"),
    ("pub struct Calibration", "Calibration"),
    ("pub enum CalibrationError", "CalibrationError"),
    ("pub struct Nanos", "Nanos"),
    ("pub mod calibrations", "calibrations"),
];

/// The four Calibration presets that must exist in `enforcement::calibrations`.
const REQUIRED_PRESETS: &[&str] = &[
    "X86_SERVER",
    "ARM_MOBILE",
    "CORTEX_M_EMBEDDED",
    "CONSERVATIVE_WORST_CASE",
];

/// The two clock accessors required on `UorTime`.
const REQUIRED_UOR_TIME_ACCESSORS: &[(&str, &str)] = &[
    ("pub const fn landauer_nats", "landauer_nats"),
    ("pub const fn rewrite_steps", "rewrite_steps"),
];

/// Runs the v0.2.2 Phase A `uor_time_surface` validator.
///
/// # Errors
///
/// Returns an error if the foundation source files cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let enforcement_path = workspace
        .join("foundation")
        .join("src")
        .join("enforcement.rs");
    let content = match std::fs::read_to_string(&enforcement_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("cannot read foundation/src/enforcement.rs: {e}"),
            ));
            return Ok(report);
        }
    };

    let mut missing: Vec<String> = Vec::new();

    for (needle, name) in REQUIRED_SYMBOLS {
        if !content.contains(needle) {
            missing.push(format!("Phase A symbol missing: `{name}` (`{needle}`)"));
        }
    }

    for preset in REQUIRED_PRESETS {
        // Each preset is a `pub const PRESET_NAME: Calibration = ...` inside
        // the `pub mod calibrations` module.
        let needle = format!("pub const {preset}: Calibration");
        if !content.contains(&needle) {
            missing.push(format!("Phase A preset missing: `calibrations::{preset}`"));
        }
    }

    for (needle, name) in REQUIRED_UOR_TIME_ACCESSORS {
        if !content.contains(needle) {
            missing.push(format!(
                "Phase A UorTime accessor missing: `{name}` (`{needle}`)"
            ));
        }
    }

    // PartialOrd is required on UorTime; Ord is forbidden. Phase 9
    // parameterized over `<H>`, so the impls now read
    // `impl<H: HostTypes> PartialOrd for UorTime<H>`.
    if !content.contains("impl<H: HostTypes> PartialOrd for UorTime<H>") {
        missing.push("UorTime<H> must implement PartialOrd (component-wise)".to_string());
    }
    if content.contains("impl<H: HostTypes> Ord for UorTime<H>")
        || content.contains("impl Ord for UorTime ")
    {
        missing.push(
            "UorTime must NOT implement Ord — two UorTime values from unrelated \
             computations are genuinely incomparable (Q1 commitment)"
                .to_string(),
        );
    }

    // LandauerBudget<H> must implement Ord (its Decimal backing has NaN
    // excluded by construction, so the foundation provides a total order).
    if !content.contains("impl<H: HostTypes> Ord for LandauerBudget<H>") {
        missing.push(
            "LandauerBudget<H> must implement Ord (NaN excluded by construction)".to_string(),
        );
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "Phase A UorTime infrastructure complete: \
             UorTime / LandauerBudget / Calibration / CalibrationError / Nanos / \
             calibrations::* all present; \
             accessors and Ord/PartialOrd discipline correct",
        ));
    } else {
        let mut summary = format!(
            "Phase A UorTime surface incomplete: {} issue(s):",
            missing.len()
        );
        for m in &missing {
            summary.push_str("\n       - ");
            summary.push_str(m);
        }
        report.push(TestResult::fail(VALIDATOR, summary));
    }

    Ok(report)
}
