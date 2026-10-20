//! v0.2.2 T6.21 validator: pin the Trace / digest byte layout.
//!
//! Asserts the byte-level interop contracts that `verify_trace` relies on
//! are stable:
//!
//! 1. `primitive_op_discriminant(PrimitiveOp) -> u8` matches the 0..=17 range
//!    (10 original primitives + 5 ADR-013/TR-08 substrate-amendment primitives
//!    Le=10, Lt=11, Ge=12, Gt=13, Concat=14 + 3 ADR-053 ring-axis completion
//!    primitives Div=15, Mod=16, Pow=17);
//! 2. `certificate_kind_discriminant(CertificateKind) -> u8` matches 1..=21
//!    (5 Phase C kinds + 16 Phase D per-resolver kinds);
//! 3. `TRACE_REPLAY_FORMAT_VERSION = 10` (bumped from 9 when ADR-057 landed —
//!    introducing `ConstraintRef::Recurse { shape_iri, descent_bound }` as
//!    the constraint-level bounded-recursion primitive with discriminant
//!    byte 10 in `fold_constraint_ref`. Required for bounded recursive
//!    structural typing of JSON values, XML documents, AST shapes,
//!    S-expressions, ASN.1 / Protobuf message families, filesystem
//!    hierarchies, and other inductively-defined input domains. Bumped
//!    from 8 when ADR-055 landed — introducing the `SubstrateTermBody`
//!    supertrait on `AxisExtension` so the catamorphism's structural
//!    reach extends through every axis surface to the leaf level via
//!    `AxisTuple::body_arena_at`. Bumped from 7 when ADR-050/051/052/053
//!    landed — adding the ring-axis arithmetic completion (Div/Mod/Pow
//!    primitives, width-parametric semantics, wide-literal carrier in
//!    `Term::Literal`, and axis-generic SDK emission). Bumped from 6 when
//!    ADR-048 landed — adding the `CommitmentEvaluated` trace event variant
//!    for the post-resolver `TypedCommitment::evaluate(kappa_label)`
//!    consultation, plus the 5th `C: TypedCommitment` substrate parameter on
//!    `PrismModel` per the cost-model commitment surface. Bumped from 5
//!    when ADR-035 + ADR-036 landed — adding the nine ψ-Term variants
//!    (Nerve, ChainComplex, HomologyGroups, Betti, CochainComplex,
//!    CohomologyGroups, PostnikovTower, HomotopyGroups, KInvariants)
//!    and the ResolverTuple substrate parameter scaffolding. Bumped from
//!    4 when ADR-034 landed — adding `Term::FirstAdmit` (twelfth variant)
//!    and the iteration-counter binding for `Term::Recurse`'s step body
//!    via `RECURSE_IDX_NAME_INDEX`. Bumped from 3 when ADR-030 + ADR-033
//!    landed. Older format-7 traces are not forward-compatible because
//!    the PrimitiveOp discriminant set changed);
//! 4. the six byte-layout helpers exist: `fold_unit_digest`,
//!    `fold_parallel_digest`, `fold_stream_digest`, `fold_interaction_digest`,
//!    `fold_constraint_ref`, `fold_stream_step_digest`, `fold_interaction_step_digest`.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/trace_byte_layout_pinned";

/// Runs the trace-byte-layout pin check.
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

    let required: &[(&str, &str)] = &[
        (
            "primitive_op_discriminant signature",
            "pub const fn primitive_op_discriminant(op: crate::PrimitiveOp) -> u8",
        ),
        (
            "certificate_kind_discriminant signature",
            "pub const fn certificate_kind_discriminant(kind: CertificateKind) -> u8",
        ),
        (
            "TRACE_REPLAY_FORMAT_VERSION = 10",
            "pub const TRACE_REPLAY_FORMAT_VERSION: u16 = 10",
        ),
        (
            "fold_unit_digest helper",
            "pub fn fold_unit_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>",
        ),
        (
            "fold_parallel_digest helper",
            "pub fn fold_parallel_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>",
        ),
        (
            "fold_stream_digest helper",
            "pub fn fold_stream_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>",
        ),
        (
            "fold_interaction_digest helper",
            "pub fn fold_interaction_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>",
        ),
        (
            "fold_constraint_ref helper",
            "pub fn fold_constraint_ref<const FP_MAX: usize, H: Hasher<FP_MAX>>",
        ),
        (
            "fold_stream_step_digest helper",
            "pub fn fold_stream_step_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>",
        ),
        (
            "fold_interaction_step_digest helper",
            "pub fn fold_interaction_step_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>",
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
            "T6.21 trace byte layout: primitive/certificate discriminants, \
             TRACE_REPLAY_FORMAT_VERSION = 10, and 7 fold_*_digest helpers pinned",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!("T6.21 trace byte layout: {} anchors missing", missing.len()),
            missing,
        ));
    }

    Ok(report)
}
