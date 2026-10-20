//! Phase H (target §1.6): libm always-on dependency + transcendentals
//! routing module.
//!
//! Pins:
//! - `libm` appears as a non-optional dependency in `foundation/Cargo.toml`
//!   (not under `[features]`, not under `[dev-dependencies]`).
//! - `pub mod transcendentals` exists in the foundation with `ln`, `exp`,
//!   `sqrt`, `entropy_term_nats` wrappers.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/libm_dependency";

/// Runs the Phase H libm dependency validation.
///
/// # Errors
///
/// Returns an error if the workspace cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let cargo_path = workspace.join("foundation/Cargo.toml");
    let cargo = match std::fs::read_to_string(&cargo_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", cargo_path.display()),
            ));
            return Ok(report);
        }
    };

    let mut missing: Vec<String> = Vec::new();

    // libm must appear under [dependencies] (not [features], not optional).
    let (deps_section, _rest) = match cargo.split_once("[dev-dependencies]") {
        Some(parts) => parts,
        None => (cargo.as_str(), ""),
    };
    let deps_section = match deps_section.split_once("[dependencies]") {
        Some((_, after)) => after,
        None => "",
    };
    if !deps_section.contains("libm = \"") && !deps_section.contains("libm = {") {
        missing.push("libm = ... under [dependencies] in foundation/Cargo.toml".to_string());
    }
    if deps_section.contains("libm") && deps_section.contains("optional = true") {
        missing.push("libm must be non-optional — target §1.6 rejects feature gating".to_string());
    }

    // The transcendentals module must exist in the foundation.
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
    // Phase 9: transcendentals dispatch generically via DecimalTranscendental;
    // the libm calls live in the f64 / f32 trait impls (foundation/src/lib.rs)
    // rather than in the transcendentals module itself.
    let lib_path = workspace.join("foundation/src/lib.rs");
    let lib_content = match std::fs::read_to_string(&lib_path) {
        Ok(s) => s,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", lib_path.display()),
            ));
            return Ok(report);
        }
    };

    let anchors: &[(&str, &str, &str)] = &[
        // (label, content_haystack, anchor_string).
        // "enf" = foundation/src/enforcement.rs, "lib" = foundation/src/lib.rs
        ("transcendentals module", "enf", "pub mod transcendentals {"),
        (
            "transcendentals::ln (generic)",
            "enf",
            "pub fn ln<D: DecimalTranscendental>(x: D) -> D",
        ),
        (
            "transcendentals::exp (generic)",
            "enf",
            "pub fn exp<D: DecimalTranscendental>(x: D) -> D",
        ),
        (
            "transcendentals::sqrt (generic)",
            "enf",
            "pub fn sqrt<D: DecimalTranscendental>(x: D) -> D",
        ),
        (
            "transcendentals::entropy_term_nats (generic)",
            "enf",
            "pub fn entropy_term_nats<D: DecimalTranscendental>(p: D) -> D",
        ),
        ("libm::log call (f64 impl)", "lib", "libm::log("),
        ("libm::exp call (f64 impl)", "lib", "libm::exp("),
        ("libm::sqrt call (f64 impl)", "lib", "libm::sqrt("),
        ("libm::logf call (f32 impl)", "lib", "libm::logf("),
        ("libm::expf call (f32 impl)", "lib", "libm::expf("),
        ("libm::sqrtf call (f32 impl)", "lib", "libm::sqrtf("),
    ];
    for (label, source, anchor) in anchors {
        let haystack = match *source {
            "enf" => &content,
            "lib" => &lib_content,
            _ => continue,
        };
        if !haystack.contains(*anchor) {
            missing.push((*label).to_string());
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "Phase H libm dependency: non-optional libm under [dependencies], \
             transcendentals module exports ln/exp/sqrt/entropy_term_nats routing \
             through libm — target §1.6 satisfied",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase H libm dependency has {} missing anchors",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
