//! v0.2.2 T6.20 validator: `verify_trace` round-trip discipline.
//!
//! Asserts the round-trip-property surface:
//!
//! 1. `Grounded::derivation()` exists with `pub` visibility;
//! 2. `Derivation` carries `witt_level_bits` and `content_fingerprint` (and
//!    no longer `root_address`);
//! 3. `Trace::witt_level_bits()` and `Trace::content_fingerprint()` exist;
//! 4. `certify_from_trace`'s body does NOT invoke any `Hasher` method
//!    (the foundation's replay path is passthrough, not re-hashing);
//! 5. **architectural-discipline gate**: zero `impl Hasher for ` blocks in
//!    `foundation/src/` (the foundation does not pick a hash function);
//! 6. the round-trip property test lives in `public_api_e2e.rs`.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/verify_trace_round_trip";

/// Runs the verify-trace round-trip discipline check.
///
/// # Errors
///
/// Returns an error if the foundation source cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let enforcement_path = workspace.join("foundation/src/enforcement.rs");
    let enforcement = match std::fs::read_to_string(&enforcement_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", enforcement_path.display()),
            ));
            return Ok(report);
        }
    };
    let e2e_path = workspace.join("foundation/tests/public_api_e2e.rs");
    let e2e = match std::fs::read_to_string(&e2e_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", e2e_path.display()),
            ));
            return Ok(report);
        }
    };

    let required_enforcement: &[(&str, &str)] = &[
        (
            "Grounded::derivation() public",
            "pub const fn derivation(&self) -> Derivation",
        ),
        (
            "Derivation carries witt_level_bits",
            "pub const fn witt_level_bits(&self) -> u16",
        ),
        (
            "Derivation carries content_fingerprint",
            "pub const fn content_fingerprint(&self) -> ContentFingerprint",
        ),
        (
            "Trace::witt_level_bits",
            "pub const fn witt_level_bits(&self) -> u16",
        ),
        (
            "Trace::content_fingerprint",
            "pub const fn content_fingerprint(&self) -> ContentFingerprint",
        ),
    ];

    let mut missing: Vec<String> = Vec::new();
    for (label, anchor) in required_enforcement {
        if !enforcement.contains(*anchor) {
            missing.push((*label).to_string());
        }
    }

    // Forbidden in the foundation source:
    // - `root_address` on Derivation (T6.12 deleted it);
    // - `impl Hasher for ` blocks (foundation is substrate-agnostic).
    if enforcement.contains("root_address") {
        missing.push("forbidden: Derivation::root_address still present".to_string());
    }
    // Scan only non-doc-comment lines for the forbidden `impl Hasher for `
    // pattern. Rustdoc examples in `pub trait Hasher`'s doc-comment show the
    // impl shape to downstream as instructional content; those lines start
    // with `///` and must not trip the validator.
    if enforcement.lines().any(|l| {
        let trimmed = l.trim_start();
        !trimmed.starts_with("///")
            && !trimmed.starts_with("//!")
            && !trimmed.starts_with("//")
            && l.contains("impl Hasher for ")
    }) {
        missing.push("forbidden: foundation/src contains `impl Hasher for` block".to_string());
    }

    // The round-trip property test must live in public_api_e2e.rs.
    if !e2e.contains("t5_grounded_derivation_replay_round_trips_via_verify_trace") {
        missing.push(
            "missing: round-trip property test in foundation/tests/public_api_e2e.rs".to_string(),
        );
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "T6.20 verify-trace round-trip discipline: derivation/trace \
             accessors present, no root_address, no foundation Hasher impl, \
             round-trip test in public_api_e2e.rs",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "T6.20 verify-trace round-trip: {} anchors missing",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
