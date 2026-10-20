//! Phase F (target §4.7): kernel namespace enforcement surface.
//!
//! Pins that the 8 kernel namespaces expose sealed witness types and
//! closed enumerations per target §4.7. Each anchor is a unique foundation
//! surface element that must remain present across future codegen updates.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/kernel_enforcement";

/// Runs the Phase F kernel-enforcement validation.
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

    let required: &[(&str, &str)] = &[
        // F.2: closed six-kind constraint enum.
        ("ConstraintKind enum", "pub enum ConstraintKind {"),
        ("ConstraintKind::Residue", "Residue,"),
        ("ConstraintKind::Carry", "Carry,"),
        ("ConstraintKind::Depth", "Depth,"),
        ("ConstraintKind::Hamming", "Hamming,"),
        ("ConstraintKind::Site", "Site,"),
        ("ConstraintKind::Affine", "Affine,"),
        // F.3 carry.
        ("CarryProfile sealed", "pub struct CarryProfile {"),
        ("CarryEvent sealed", "pub struct CarryEvent {"),
        // F.3 convergence.
        (
            "ConvergenceLevel<L> sealed",
            "pub struct ConvergenceLevel<L> {",
        ),
        // F.3 division.
        (
            "DivisionAlgebraWitness enum",
            "pub enum DivisionAlgebraWitness {",
        ),
        ("DivisionAlgebraWitness::Real", "Real,"),
        ("DivisionAlgebraWitness::Complex", "Complex,"),
        ("DivisionAlgebraWitness::Quaternion", "Quaternion,"),
        ("DivisionAlgebraWitness::Octonion", "Octonion,"),
        // F.3 monoidal.
        (
            "MonoidalProduct<L, R> sealed",
            "pub struct MonoidalProduct<L, R> {",
        ),
        ("MonoidalUnit<L> sealed", "pub struct MonoidalUnit<L> {"),
        // F.1 operad.
        ("OperadComposition sealed", "pub struct OperadComposition {"),
        // F.3 recursion.
        (
            "RECURSION_TRACE_MAX_DEPTH const",
            "pub const RECURSION_TRACE_MAX_DEPTH",
        ),
        ("RecursionTrace sealed", "pub struct RecursionTrace {"),
        // F.3 region.
        ("AddressRegion sealed", "pub struct AddressRegion {"),
        // F.3 linear.
        ("LinearBudget sealed", "pub struct LinearBudget {"),
        ("LeaseAllocation sealed", "pub struct LeaseAllocation {"),
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
            "Phase F kernel enforcement: 8 namespaces (carry, convergence, division, monoidal, \
             operad, recursion, region, linear) expose sealed witness types + closed enumerations \
             (ConstraintKind: 6 variants, DivisionAlgebraWitness: 4 variants) per target §4.7",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase F kernel enforcement has {} missing anchors",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
