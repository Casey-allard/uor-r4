//! v0.2.2 Phase E validator: bridge namespace completion.
//!
//! Asserts that the foundation crate exposes the sealed bridge-namespace
//! surface introduced by Phase E: the Query/Coordinate/BindingQuery/Partition/
//! PartitionComponent/Trace/TraceEvent types, the six BaseMetric accessors on
//! `Grounded<T, Tag>`, the `MAX_BETTI_DIMENSION` / `JACOBIAN_MAX_SITES`
//! constants, the `SigmaValue` and `JacobianMetric<L>` sealed carriers, the
//! `HomologyClass<FP_MAX>` / `CohomologyClass<FP_MAX>` fingerprint-width-
//! parametric classes, the `Derivation::replay` accessor, and the `InteractionDeclarationBuilder`
//! entry point.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/bridge_namespace_completion";

/// Runs the bridge namespace completion check.
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
        // Constants. Wiki ADR-037 names `HostBounds::BETTI_DIMENSION_MAX` /
        // `JACOBIAN_SITES_MAX` as the canonical capacity bounds; these
        // foundation-internal `pub const`s carry conservative defaults for
        // stable-Rust array-size positions (ADR-060 removed `DefaultHostBounds`).
        (
            "MAX_BETTI_DIMENSION constant",
            "pub const MAX_BETTI_DIMENSION: usize =",
        ),
        (
            "JACOBIAN_MAX_SITES constant",
            "pub const JACOBIAN_MAX_SITES: usize =",
        ),
        // Trace event-count ceiling is carried by `HostBounds` per the wiki's
        // ADR-018 — there is no free-standing `TRACE_MAX_EVENTS` constant; the
        // canonical surface is `Trace<const TR_MAX: usize = 256>`.
        (
            "Trace const-generic carrier",
            "pub struct Trace<const TR_MAX: usize",
        ),
        // Sealed BaseMetric carriers.
        ("SigmaValue sealed type", "pub struct SigmaValue"),
        (
            "JacobianMetric<L> sealed type",
            "pub struct JacobianMetric<L>",
        ),
        ("PartitionComponent enum", "pub enum PartitionComponent"),
        // Bridge surface.
        ("Query sealed type", "pub struct Query"),
        ("Coordinate<L> sealed type", "pub struct Coordinate<L>"),
        ("BindingQuery sealed type", "pub struct BindingQuery"),
        ("Partition sealed type", "pub struct Partition"),
        ("TraceEvent sealed type", "pub struct TraceEvent"),
        ("Trace sealed type", "pub struct Trace"),
        // Phase X.2: dimension-as-runtime-field carriers + cup-product algebra.
        // ADR-018/060: cohomology/homology classes carry the application's
        // fingerprint width `FP_MAX` (default 32) — they are minted through
        // the consumer's `Hasher<FP_MAX>`, so the digest survives at full width.
        (
            "HomologyClass",
            "pub struct HomologyClass<const FP_MAX: usize = 32> {",
        ),
        (
            "CohomologyClass",
            "pub struct CohomologyClass<const FP_MAX: usize = 32> {",
        ),
        ("CohomologyError", "pub enum CohomologyError {"),
        (
            "mint_cohomology_class",
            "pub fn mint_cohomology_class<H: Hasher<FP_MAX>, const FP_MAX: usize>",
        ),
        (
            "mint_homology_class",
            "pub fn mint_homology_class<H: Hasher<FP_MAX>, const FP_MAX: usize>",
        ),
        (
            "fold_cup_product",
            "pub fn fold_cup_product<const FP_MAX: usize, H: Hasher<FP_MAX>>",
        ),
        (
            "MAX_COHOMOLOGY_DIMENSION",
            "pub const MAX_COHOMOLOGY_DIMENSION: u32 = 32;",
        ),
        // Target §3 Sink/Sinking hardening.
        (
            "Sinking trait",
            "pub trait Sinking<const INLINE_BYTES: usize> {",
        ),
        ("MorphismKind trait", "pub trait MorphismKind:"),
        ("ProjectionMapKind trait", "pub trait ProjectionMapKind:"),
        (
            "EmitThrough trait",
            "pub trait EmitThrough<const INLINE_BYTES: usize, H: crate::HostTypes>:",
        ),
        (
            "IntegerProjectionMap marker",
            "pub struct IntegerProjectionMap;",
        ),
        ("Utf8ProjectionMap marker", "pub struct Utf8ProjectionMap;"),
        ("JsonProjectionMap marker", "pub struct JsonProjectionMap;"),
        (
            "DigestProjectionMap marker",
            "pub struct DigestProjectionMap;",
        ),
        (
            "BinaryProjectionMap marker",
            "pub struct BinaryProjectionMap;",
        ),
        (
            "InteractionDeclarationBuilder",
            "pub struct InteractionDeclarationBuilder",
        ),
        // Phase A.4: sealed BaseMetric newtype carriers.
        ("DDeltaMetric sealed type", "pub struct DDeltaMetric"),
        ("EulerMetric sealed type", "pub struct EulerMetric"),
        ("ResidualMetric sealed type", "pub struct ResidualMetric"),
        ("BettiMetric sealed type", "pub struct BettiMetric"),
        // Phase A.3: sealed stratum newtype.
        ("Stratum<L> sealed type", "pub struct Stratum<L>"),
        // Six BaseMetric accessors on Grounded now return sealed newtypes.
        (
            "Grounded::d_delta accessor",
            "pub const fn d_delta(&self) -> DDeltaMetric",
        ),
        (
            "Grounded::sigma accessor",
            "pub fn sigma(&self) -> SigmaValue",
        ),
        // v0.2.2 T2.6 (cleanup): BaseMetric field storage anchors.
        ("Grounded::sigma_ppm field", "sigma_ppm: u32"),
        (
            "Grounded::jacobian_entries field",
            "jacobian_entries: [i64; JACOBIAN_MAX_SITES]",
        ),
        (
            "Grounded::betti_numbers field",
            "betti_numbers: [u32; MAX_BETTI_DIMENSION]",
        ),
        (
            "Grounded::jacobian accessor",
            "pub fn jacobian(&self) -> JacobianMetric<T>",
        ),
        (
            "Grounded::betti accessor",
            "pub const fn betti(&self) -> BettiMetric",
        ),
        (
            "Grounded::euler accessor",
            "pub const fn euler(&self) -> EulerMetric",
        ),
        (
            "Grounded::residual accessor",
            "pub const fn residual(&self) -> ResidualMetric",
        ),
        // Phase A.1: uor_time accessors on Grounded and Certified.
        (
            "Grounded::uor_time accessor",
            "pub const fn uor_time(&self) -> UorTime",
        ),
        (
            "Grounded::triad accessor",
            "pub const fn triad(&self) -> Triad<T>",
        ),
        // Derivation::replay accessor — parametric over `<const TR_MAX: usize>`
        // per ADR-018; the trace event-count ceiling flows from the
        // application's selected `HostBounds`.
        (
            "Derivation::replay accessor",
            "pub fn replay<const TR_MAX: usize>(&self) -> Trace<TR_MAX, FP_MAX>",
        ),
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
            "Phase E bridge namespace completion: MAX_BETTI_DIMENSION, \
             SigmaValue, JacobianMetric<L>, Query/Coordinate/BindingQuery/\
             Partition/Trace/TraceEvent/HomologyClass/CohomologyClass, six \
             BaseMetric accessors on Grounded, Derivation::replay, and \
             InteractionDeclarationBuilder all present",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase E bridge namespace completion has {} missing anchors",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
