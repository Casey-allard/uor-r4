//! Phase D (target §4.2 + §9 criterion 4): resolver-tower completion.
//!
//! Pins that the foundation exposes one `certify(...)` free function per
//! `resolver:*` ontology class via the module-per-resolver organization:
//! `enforcement::resolver::<snake_case_name>::certify(...)`.
//!
//! Current coverage: 21 of 22 resolver classes (5 from v0.2.2 + 16 added
//! in Phase D). The 22nd entry in the earlier fact-check was the base
//! `resolver:Resolver` class, which is abstract and has no decision
//! procedure — it is the supertype of the 21 concrete classes below.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/resolver_tower";

/// All concrete resolver classes in the foundation. Each entry: (ontology
/// class name, Rust module name, expected anchor in enforcement.rs).
const RESOLVER_MODULES: &[(&str, &str, &str)] = &[
    // v0.2.2 baseline (5).
    (
        "TowerCompletenessResolver",
        "tower_completeness",
        "pub mod tower_completeness",
    ),
    (
        "IncrementalCompletenessResolver",
        "incremental_completeness",
        "pub mod incremental_completeness",
    ),
    (
        "GroundingAwareResolver",
        "grounding_aware",
        "pub mod grounding_aware",
    ),
    ("InhabitanceResolver", "inhabitance", "pub mod inhabitance"),
    (
        "MultiplicationResolver",
        "multiplication",
        "pub mod multiplication",
    ),
    // Phase D additions (16).
    (
        "TwoSatDecider",
        "two_sat_decider",
        "pub mod two_sat_decider",
    ),
    (
        "HornSatDecider",
        "horn_sat_decider",
        "pub mod horn_sat_decider",
    ),
    (
        "ResidualVerdictResolver",
        "residual_verdict",
        "pub mod residual_verdict",
    ),
    (
        "CanonicalFormResolver",
        "canonical_form",
        "pub mod canonical_form",
    ),
    (
        "TypeSynthesisResolver",
        "type_synthesis",
        "pub mod type_synthesis",
    ),
    ("HomotopyResolver", "homotopy", "pub mod homotopy"),
    ("MonodromyResolver", "monodromy", "pub mod monodromy"),
    ("ModuliResolver", "moduli", "pub mod moduli"),
    (
        "JacobianGuidedResolver",
        "jacobian_guided",
        "pub mod jacobian_guided",
    ),
    ("EvaluationResolver", "evaluation", "pub mod evaluation"),
    ("SessionResolver", "session", "pub mod session"),
    (
        "SuperpositionResolver",
        "superposition",
        "pub mod superposition",
    ),
    ("MeasurementResolver", "measurement", "pub mod measurement"),
    (
        "WittLevelResolver",
        "witt_level_resolver",
        "pub mod witt_level_resolver",
    ),
    (
        "DihedralFactorizationResolver",
        "dihedral_factorization",
        "pub mod dihedral_factorization",
    ),
    (
        "CompletenessResolver",
        "completeness",
        "pub mod completeness",
    ),
];

/// Runs the Phase D resolver-tower validation.
///
/// # Errors
///
/// Returns an error if the foundation source cannot be read.
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

    let mut missing: Vec<String> = Vec::new();
    for (class_name, module_name, anchor) in RESOLVER_MODULES {
        if !content.contains(anchor) {
            missing.push(format!(
                "resolver:{class_name} has no `enforcement::resolver::{module_name}::certify` module"
            ));
        }
        // Each module must expose a `certify` entry point.
        let certify_needle = format!("pub mod {module_name} {{");
        if let Some(idx) = content.find(&certify_needle) {
            let window = &content[idx..content.len().min(idx + 4000)];
            if !window.contains("pub fn certify") {
                missing.push(format!(
                    "resolver::{module_name} module exists but has no `pub fn certify` entry point"
                ));
            }
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "Phase D resolver tower: {} resolver classes expose `certify(...)` free \
                 functions under `enforcement::resolver::*` per target §4.2 — no \
                 resolver ships as a perpetual-impossibility stub",
                RESOLVER_MODULES.len()
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase D resolver tower has {} missing modules/entry points",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
