//! v0.2.2 Phase F validator: driver shape.
//!
//! Asserts that the foundation crate exposes the four named-driver entry
//! points (`pipeline::run`, `run_parallel`, `run_stream`, `run_interactive`)
//! and their sealed supporting types (`StreamDriver`, `InteractionDriver`,
//! `StepResult`, `PeerInput`, `PeerPayload`, `CommutatorState`,
//! `ParallelDeclaration`, `StreamDeclaration`, `InteractionDeclaration`) with
//! the expected signatures.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/driver_shape";

/// Runs the driver shape check.
///
/// # Errors
///
/// Returns an error if the foundation source file cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let pipeline_path = workspace.join("foundation/src/pipeline.rs");
    let content = match std::fs::read_to_string(&pipeline_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", pipeline_path.display()),
            ));
            return Ok(report);
        }
    };

    let required: &[(&str, &str)] = &[
        // Declaration markers.
        ("ParallelDeclaration", "pub struct ParallelDeclaration"),
        ("StreamDeclaration", "pub struct StreamDeclaration"),
        (
            "InteractionDeclaration",
            "pub struct InteractionDeclaration",
        ),
        // v0.2.2 T2.7 (cleanup): payload accessors prove input-dependence.
        (
            "ParallelDeclaration::site_count",
            "pub const fn site_count(&self) -> u64",
        ),
        (
            "StreamDeclaration::productivity_bound",
            "pub const fn productivity_bound(&self) -> u64",
        ),
        (
            "InteractionDeclaration::convergence_seed",
            "pub const fn convergence_seed(&self) -> u64",
        ),
        (
            "InteractionDriver::commutator_acc field",
            "commutator_acc: [u64; 4]",
        ),
        ("StreamDriver::seed field", "seed: u64"),
        // Sealed peer and commutator types.
        ("PeerPayload sealed", "pub struct PeerPayload"),
        ("PeerInput sealed", "pub struct PeerInput"),
        ("StepResult enum", "pub enum StepResult"),
        ("CommutatorState<L>", "pub struct CommutatorState<L>"),
        // Drivers.
        ("StreamDriver<T, P>", "pub struct StreamDriver<"),
        ("InteractionDriver<T, P>", "pub struct InteractionDriver<"),
        (
            "StreamDriver Iterator impl",
            "Iterator for StreamDriver<T, P, H, INLINE_BYTES, FP_MAX>",
        ),
        // Pipeline entry points.
        // v0.2.2 T6.1: every driver entry point threads an `H: Hasher<FP_MAX>`
        // type parameter for the parametric substrate fingerprint. ADR-060 adds
        // the `const INLINE_BYTES: usize` carrier-width parameter; ADR-018/060
        // adds the `const FP_MAX: usize` fingerprint-width parameter so any
        // `Hasher<FP_MAX>` (e.g. SHA-512 at 64) flows through every driver.
        (
            "pipeline::run",
            "pub fn run<T, P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(",
        ),
        (
            "pipeline::run_parallel",
            "pub fn run_parallel<T, P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(",
        ),
        (
            "pipeline::run_stream",
            "pub fn run_stream<T, P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(",
        ),
        (
            "pipeline::run_interactive",
            "pub fn run_interactive<T, P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(",
        ),
        // InteractionDriver methods.
        (
            "InteractionDriver::step",
            "pub fn step(&mut self, input: PeerInput) -> StepResult<T, INLINE_BYTES, FP_MAX>",
        ),
        (
            "InteractionDriver::is_converged",
            "pub const fn is_converged(&self) -> bool",
        ),
        (
            "InteractionDriver::finalize",
            "pub fn finalize(self) -> Result<Grounded<'static, T, INLINE_BYTES, FP_MAX>, PipelineFailure>",
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
            "Phase F driver shape: run/run_parallel/run_stream/run_interactive \
             entry points, StreamDriver + InteractionDriver sealed types, \
             PeerInput, PeerPayload, StepResult, CommutatorState all present",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!("Phase F driver shape has {} missing anchors", missing.len()),
            missing,
        ));
    }

    Ok(report)
}
