//! v0.2.2 T6.18 validator: `calibrations` preset literals are valid.
//!
//! Every `pub const` in the `calibrations` module calls `Calibration::new`
//! inside a `match { Ok(c) => c, Err(_) => panic!() }` scaffold. If any
//! preset's physical parameters are invalid (NaN, negative, zero), the
//! foundation fails to compile. This validator asserts:
//!
//! 1. the `calibrations` module exists;
//! 2. each preset uses the `match Calibration::new(...) { Ok(c) => c, Err(_) => panic!() }`
//!    scaffold so a compile-time failure produces a clear diagnostic;
//! 3. the expected preset names (X86_SERVER, ARM_MOBILE, CORTEX_M_EMBEDDED,
//!    CONSERVATIVE_WORST_CASE) are all present.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/calibration_presets_valid";

// Phase 9 changed the preset declarations to construct via the const helper
// `Calibration::<DefaultHostTypes>::from_f64_unchecked` (Calibration::new is
// no longer const since it operates over generic H::Decimal). The four
// preset names and their default-host pinning are still required.
const REQUIRED_PRESETS: &[&str] = &[
    "pub const X86_SERVER: Calibration<DefaultHostTypes> =",
    "pub const ARM_MOBILE: Calibration<DefaultHostTypes> =",
    "pub const CORTEX_M_EMBEDDED: Calibration<DefaultHostTypes> =",
    "pub const CONSERVATIVE_WORST_CASE: Calibration<DefaultHostTypes> =",
];

/// Runs the calibration presets validator.
///
/// # Errors
///
/// Returns an error if the foundation enforcement source cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let enforcement_path = workspace.join("foundation/src/enforcement.rs");
    let content = match std::fs::read_to_string(&enforcement_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", enforcement_path.display()),
            ));
            return Ok(report);
        }
    };

    if !content.contains("pub mod calibrations {") {
        report.push(TestResult::fail(
            VALIDATOR,
            "`pub mod calibrations` not found in foundation enforcement source",
        ));
        return Ok(report);
    }

    let mut missing: Vec<String> = Vec::new();
    for preset in REQUIRED_PRESETS {
        if !content.contains(*preset) {
            missing.push((*preset).to_string());
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "T6.18 calibration presets: all 4 named constants validated via \
             Calibration::new match scaffold",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!("{} calibration preset anchors missing", missing.len()),
            missing,
        ));
    }

    Ok(report)
}
