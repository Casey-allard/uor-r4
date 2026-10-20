//! v0.2.2 T6 validator: const-fn frontier.
//!
//! Asserts that the pipeline crate exposes the const-fn surface for the four
//! builder validators, plus the structural anchors documenting that the
//! four `validate_*_const` companions are kept in lockstep with their runtime
//! equivalents. Post-T6:
//!
//! - the `certify_*_const` companions are no longer `const fn` (trait method
//!   dispatch on `H: Hasher` is not const-eval-friendly under MSRV 1.81);
//! - `run_const<T, H>` threads the consumer-supplied `H: Hasher` via
//!   `fold_unit_digest` and returns `Result<Grounded<T>, PipelineFailure>`;
//! - `run_const_zero` is gone (no legacy fallback path);
//! - `fnv1a_u128_const` and `hash_constraints` are gone (foundation no longer
//!   picks a hash function);
//! - `with_level_const` is gone (only `with_level_and_fingerprint_const`
//!   survives on the cert shims).

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/const_fn_frontier";

/// Runs the const-fn frontier check.
///
/// # Errors
///
/// Returns an error if the pipeline source file cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let pipeline_path = workspace.join("foundation/src/pipeline.rs");
    let enforcement_path = workspace.join("foundation/src/enforcement.rs");
    let pipeline_content = match std::fs::read_to_string(&pipeline_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", pipeline_path.display()),
            ));
            return Ok(report);
        }
    };
    let enforcement_content = match std::fs::read_to_string(&enforcement_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", enforcement_path.display()),
            ));
            return Ok(report);
        }
    };
    let content = format!("{pipeline_content}\n{enforcement_content}");

    let required: &[(&str, &str)] = &[
        // validate_*_const family (4). Each is `const fn` and shares the
        // runtime validate() field set (T6.13 dual-path consistency).
        ("validate_lease_const", "pub const fn validate_lease_const<"),
        (
            "validate_compile_unit_const",
            "pub const fn validate_compile_unit_const",
        ),
        (
            "validate_parallel_const",
            "pub const fn validate_parallel_const",
        ),
        (
            "validate_stream_const",
            "pub const fn validate_stream_const",
        ),
        // Phase C.2: run_const carries `M: Total + Invertible` to enforce
        // const-eval admissibility — see target §6.
        ("pipeline::run_const", "pub fn run_const<T, M, H"),
        // Const-fn accessors on CompileUnitBuilder.
        (
            "CompileUnitBuilder::witt_level_option",
            "pub const fn witt_level_option(&self) -> Option<WittLevel>",
        ),
        (
            "CompileUnitBuilder::budget_option",
            "pub const fn budget_option(&self) -> Option<u64>",
        ),
        (
            "CompileUnitBuilder::has_root_term_const",
            "pub const fn has_root_term_const(&self) -> bool",
        ),
        (
            "CompileUnitBuilder::has_target_domains_const",
            "pub const fn has_target_domains_const(&self) -> bool",
        ),
        (
            "CompileUnitBuilder::result_type_iri_const",
            "pub const fn result_type_iri_const(&self) -> Option<&'static str>",
        ),
        // T6.11: CompileUnit::from_parts_const now takes a result_type_iri.
        (
            "CompileUnit::from_parts_const",
            "pub(crate) const fn from_parts_const(",
        ),
        // T6.7: only `with_level_and_fingerprint_const` survives.
        (
            "GroundingCertificate::with_level_and_fingerprint_const",
            "pub(crate) const fn with_level_and_fingerprint_const(",
        ),
        // run_const threads the consumer-supplied Hasher via fold_unit_digest.
        (
            "run_const threads Hasher via fold_unit_digest",
            "crate::enforcement::fold_unit_digest(",
        ),
    ];

    let mut missing: Vec<String> = Vec::new();
    for (label, anchor) in required {
        if !content.contains(*anchor) {
            missing.push((*label).to_string());
        }
    }

    // T6.14 / T6.4 / T6.7 negative anchors: these identifiers must not
    // appear in the foundation source any longer.
    let forbidden: &[(&str, &str)] = &[
        ("fnv1a_u128_const", "fnv1a_u128_const"),
        ("hash_constraints", "hash_constraints"),
        ("run_const_zero", "run_const_zero"),
        ("with_level_const (legacy)", "with_level_const("),
        ("ZeroHasher", "ZeroHasher"),
    ];
    let mut present: Vec<String> = Vec::new();
    for (label, needle) in forbidden {
        if content.contains(*needle) {
            present.push((*label).to_string());
        }
    }

    if missing.is_empty() && present.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "T6 const-fn frontier: 4 validate_*_const + run_const<T, H> \
             + accessors + no legacy hash helpers",
        ));
    } else {
        let mut details: Vec<String> = Vec::new();
        for m in &missing {
            details.push(format!("missing anchor: {m}"));
        }
        for p in &present {
            details.push(format!("forbidden identifier still present: {p}"));
        }
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "T6 const-fn frontier has {} issues",
                missing.len() + present.len()
            ),
            details,
        ));
    }

    Ok(report)
}
