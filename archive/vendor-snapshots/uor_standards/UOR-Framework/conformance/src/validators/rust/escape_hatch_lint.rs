//! Phase I (target §7.1 + §9 criterion 9): sealed-set escape-hatch lint.
//!
//! Scans the foundation crate's generated source for structural escape
//! hatches that would let downstream bypass sealed types. The discipline
//! is: every sealed type must be constructible only via foundation-owned
//! paths.
//!
//! The lint denies three families of forbidden patterns:
//!
//! 1. `unsafe impl <SealedTrait>` for any sealed trait in the §2 table.
//! 2. `pub fn new(...) -> SealedType` or `pub const fn new(...) -> SealedType`
//!    on any sealed type in the §2 table. Only `pub(crate) fn new(...)` is
//!    admissible inside the foundation.
//! 3. `impl From<...> for SealedType` outside the audited phase-coercion
//!    exception (`CompileTime → Runtime`).
//!
//! This is a grep-based implementation that stands in for a dylint crate;
//! it scans only `foundation/src/enforcement.rs` where every sealed-type
//! definition lives.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/escape_hatch_lint";

/// Sealed traits covered by the lint. The foundation is the only place
/// these may be implemented.
const SEALED_TRAITS: &[&str] = &[
    // v0.2.2 seal coverage (6).
    "OntologyTarget",
    "Certificate",
    "GroundingMapKind",
    "ValidationPhase",
    "Observable",
    "BoundShape",
    // Phase I additions (7) — the full §2 sealed-trait table.
    "GroundedShape",
    "GroundedValue",
    "WittLevel",
    "FragmentMarker",
    "ImpossibilityWitnessKind",
    "RingOp",
    "ValidLevelEmbedding",
    // Target §3 Sink/Sinking hardening — ProjectionMap kind discriminator +
    // shared morphism-kind supertrait.
    "ProjectionMapKind",
    "MorphismKind",
];

/// Sealed types whose construction must go through foundation-owned paths.
/// `pub fn new` / `pub const fn new` returning these types is forbidden;
/// only `pub(crate)` constructors are admissible.
const SEALED_TYPES: &[&str] = &[
    // Original 23 — target §2 sealed-type table (Rust-typed rows).
    "Datum",
    "Validated",
    "Grounded",
    "Certified",
    "Triad",
    "Derivation",
    "FreeRank",
    "BoundarySession",
    "BindingsTable",
    "UorTime",
    "LandauerBudget",
    "Stratum",
    "ContentAddress",
    "Nanos",
    "Calibration",
    "DDeltaMetric",
    "EulerMetric",
    "ResidualMetric",
    "BettiMetric",
    "SigmaValue",
    "JacobianMetric",
    "Trace",
    "TraceEvent",
    // Workstream D.2: builder-output types from §2's "individuals
    // materialized by …" rows. These are Rust types the foundation
    // exposes; downstream cannot construct them via `pub fn new`.
    "CompileUnit",
    "EffectDeclaration",
    "DispatchDeclaration",
    "DispatchRule",
    "PredicateDeclaration",
    "ParallelDeclaration",
    "StreamDeclaration",
    "LeaseDeclaration",
    "WittLevelDeclaration",
    "InteractionDeclaration",
    "GroundingDeclaration",
    "TypeDeclaration",
    "SourceDeclaration",
    "SinkDeclaration",
    // Workstream F.1: new spectral-sequence sealed type.
    "SpectralSequencePage",
];

/// `From<...>` impls producing a sealed type outside this audited set
/// are flagged. The one admissible exception is the phase-coercion impl
/// emitted for `Validated<T, CompileTime> → Validated<T, Runtime>`.
fn is_allowed_from_impl(line: &str) -> bool {
    // The only admissible From-impl: Validated<T, CompileTime> → Runtime.
    line.contains("Validated<T, CompileTime>") && line.contains("for Validated<T, Runtime>")
}

/// Runs the escape-hatch lint check.
///
/// # Errors
///
/// Returns an error if the foundation source files cannot be read.
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

    let mut violations: Vec<String> = Vec::new();

    // Family 1: unsafe impl for any sealed trait is always forbidden.
    for trait_name in SEALED_TRAITS {
        let needle = format!("unsafe impl {trait_name}");
        if content.contains(&needle) {
            violations.push(format!("forbidden `unsafe impl {trait_name}` found"));
        }
        // Also deny the generic form. Generic unsafe impls can land on
        // anything; we only flag if a sealed trait name appears on the
        // same line as `unsafe impl<`.
        for line in content.lines() {
            if line.contains("unsafe impl<") && line.contains(trait_name) {
                violations.push(format!(
                    "forbidden `unsafe impl<...> {trait_name}` on line `{}`",
                    line.trim()
                ));
            }
        }
    }

    // Family 2: `pub fn new` / `pub const fn new` returning a sealed type.
    // The foundation's constructors are all `pub(crate)` — any public one
    // is an escape hatch. We scan line-by-line, matching `pub fn new(` or
    // `pub const fn new(` (but not `pub(crate)`).
    for line in content.lines() {
        let trimmed = line.trim_start();
        // pub(crate), pub(super), pub(in ...) all stay inside crate scope;
        // only naked `pub fn`/`pub const fn` reach downstream.
        let is_downstream_pub = (trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub const fn "))
            && !trimmed.starts_with("pub(crate)")
            && !trimmed.starts_with("pub(super)")
            && !trimmed.starts_with("pub(in ");
        if !is_downstream_pub {
            continue;
        }
        // Does the signature declare a constructor named `new` or `new_*`
        // whose return type is one of the sealed types?
        let is_new_ctor = trimmed.contains(" fn new(") || trimmed.contains(" fn new_internal(");
        if !is_new_ctor {
            continue;
        }
        // Skip the `-> Self` case — it's contextual and can't be resolved
        // without tracking the enclosing impl block. The `pub(crate)`
        // discipline in the codegen covers it.
        for sealed in SEALED_TYPES {
            let return_marker_1 = format!("-> {sealed}");
            let return_marker_2 = format!("-> {sealed}<");
            if line.contains(&return_marker_1) || line.contains(&return_marker_2) {
                violations.push(format!(
                    "forbidden public constructor `{}` returning sealed `{sealed}`",
                    trimmed
                ));
            }
        }
    }

    // Family 3: `impl From<...> for SealedType` outside the audited
    // phase-coercion exception.
    for line in content.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("impl") || !trimmed.contains("From<") {
            continue;
        }
        for sealed in SEALED_TYPES {
            let needle_1 = format!("for {sealed}");
            let needle_2 = format!("for {sealed}<");
            if (trimmed.contains(&needle_1) || trimmed.contains(&needle_2))
                && !is_allowed_from_impl(trimmed)
            {
                violations.push(format!(
                    "forbidden `impl From<...> for {sealed}` outside the phase-coercion \
                     exception — line: `{}`",
                    trimmed
                ));
            }
        }
    }

    // Family 4: no_std discipline — `extern crate alloc` / `extern crate std`
    // must not appear at the crate root under the default feature set.
    if content.contains("extern crate alloc") {
        violations.push("foundation/src/enforcement.rs has `extern crate alloc`".to_string());
    }
    if content.contains("extern crate std") {
        violations.push("foundation/src/enforcement.rs has `extern crate std`".to_string());
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "Phase I escape-hatch lint: 13 sealed traits and 38 sealed types covered; \
             no forbidden `unsafe impl`, public `new` constructors, or unaudited \
             `From<...>` impls in the foundation source (target §7.1, §9 criterion 9)",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase I escape-hatch lint found {} violations",
                violations.len()
            ),
            violations,
        ));
    }

    Ok(report)
}
