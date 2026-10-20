//! v0.2.2 Phase D (Q4) validator: parametric constraint surface.
//!
//! Asserts that the foundation crate exposes:
//! - sealed `Observable` and `BoundShape` marker traits with the closed
//!   catalogue of impls (5 Observable unit structs + 6 BoundShape unit
//!   structs);
//! - the parametric `BoundConstraint<O: Observable, B: BoundShape>` carrier
//!   and its `BoundArguments` / `BoundArgValue` / `BoundArgEntry` fixed-size
//!   argument table;
//! - the parametric `Conjunction<const N: usize>` wrapper;
//! - the seven legacy constraint kind aliases over the parametric carrier.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/parametric_constraints";

/// Runs the parametric constraint surface check.
///
/// # Errors
///
/// Returns an error if the foundation source file cannot be read.
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
        // Sealed supertraits.
        (
            "sealed Observable supertrait",
            "mod bound_constraint_sealed",
        ),
        (
            "Observable trait",
            "pub trait Observable: bound_constraint_sealed::ObservableSealed",
        ),
        (
            "BoundShape trait",
            "pub trait BoundShape: bound_constraint_sealed::BoundShapeSealed",
        ),
        // Observable catalogue (5).
        ("ValueModObservable", "pub struct ValueModObservable;"),
        ("HammingMetric", "pub struct HammingMetric;"),
        (
            "DerivationDepthObservable",
            "pub struct DerivationDepthObservable;",
        ),
        ("CarryDepthObservable", "pub struct CarryDepthObservable;"),
        ("FreeRankObservable", "pub struct FreeRankObservable;"),
        // BoundShape catalogue (6).
        ("EqualBound", "pub struct EqualBound;"),
        ("LessEqBound", "pub struct LessEqBound;"),
        ("GreaterEqBound", "pub struct GreaterEqBound;"),
        ("RangeContainBound", "pub struct RangeContainBound;"),
        ("ResidueClassBound", "pub struct ResidueClassBound;"),
        ("AffineEqualBound", "pub struct AffineEqualBound;"),
        // Parametric carriers.
        (
            "BoundConstraint<O, B>",
            "pub struct BoundConstraint<O: Observable, B: BoundShape>",
        ),
        ("Conjunction<N>", "pub struct Conjunction<const N: usize>"),
        ("BoundArgValue enum", "pub enum BoundArgValue"),
        ("BoundArguments struct", "pub struct BoundArguments"),
        ("BoundArgEntry struct", "pub struct BoundArgEntry"),
        // Legacy type aliases (7).
        (
            "ResidueConstraint alias",
            "pub type ResidueConstraint = BoundConstraint<ValueModObservable, ResidueClassBound>;",
        ),
        (
            "HammingConstraint alias",
            "pub type HammingConstraint = BoundConstraint<HammingMetric, LessEqBound>;",
        ),
        (
            "DepthConstraint alias",
            "pub type DepthConstraint = BoundConstraint<DerivationDepthObservable, LessEqBound>;",
        ),
        (
            "CarryConstraint alias",
            "pub type CarryConstraint = BoundConstraint<CarryDepthObservable, LessEqBound>;",
        ),
        (
            "SiteConstraint alias",
            "pub type SiteConstraint = BoundConstraint<FreeRankObservable, LessEqBound>;",
        ),
        (
            "AffineConstraint alias",
            "pub type AffineConstraint = BoundConstraint<ValueModObservable, AffineEqualBound>;",
        ),
        (
            "CompositeConstraint alias",
            "pub type CompositeConstraint<const N: usize> = Conjunction<N>;",
        ),
    ];

    // v0.2.2 T2.2 (cleanup): pipeline-side parametric ConstraintRef anchors.
    // These live in foundation/src/pipeline.rs, not enforcement.rs.
    let pipeline_path = workspace.join("foundation/src/pipeline.rs");
    let pipeline_content = std::fs::read_to_string(&pipeline_path).unwrap_or_default();
    let pipeline_required: &[(&str, &str)] = &[
        (
            "ConstraintRef::Bound parametric variant",
            "observable_iri: &'static str,",
        ),
        (
            "ConstraintRef::Conjunction parametric variant",
            // Phase 17 — `Conjunction.conjuncts` is now a fixed-size
            // `[LeafConstraintRef; CONJUNCTION_MAX_TERMS]` array with
            // an active prefix length, not a variable-length slice.
            "conjuncts: [LeafConstraintRef; CONJUNCTION_MAX_TERMS],",
        ),
        (
            "encode_constraint_to_clauses dispatch",
            "pub(crate) const fn encode_constraint_to_clauses(",
        ),
    ];

    let mut missing: Vec<String> = Vec::new();
    for (label, anchor) in required {
        if !content.contains(*anchor) {
            missing.push((*label).to_string());
        }
    }
    for (label, anchor) in pipeline_required {
        if !pipeline_content.contains(*anchor) {
            missing.push((*label).to_string());
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "Phase D parametric constraint surface complete: sealed Observable + \
             BoundShape catalogues, BoundConstraint<O, B> carrier, Conjunction<N> \
             wrapper, 7 legacy type aliases, ConstraintRef parametric variants, \
             and pub(crate) encode_constraint_to_clauses dispatch all present",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase D parametric constraint surface has {} missing anchors",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
