//! v0.2.2 Phase C.4 validator: multiplication resolver surface.
//!
//! Asserts that the foundation crate exposes:
//! - `MulContext` struct with public `stack_budget_bytes`, `const_eval`,
//!   `limb_count` fields.
//! - `MultiplicationEvidence` struct with `splitting_factor`,
//!   `sub_multiplication_count`, `landauer_cost_nats` accessors.
//! - `MultiplicationCertificate` sealed shim.
//! - `resolver::multiplication::certify` free function.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/multiplication_resolver";

/// Runs the multiplication resolver surface check.
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
        ("MulContext struct", "pub struct MulContext"),
        (
            "MulContext stack_budget_bytes field",
            "pub stack_budget_bytes: u64",
        ),
        ("MulContext const_eval field", "pub const_eval: bool"),
        ("MulContext limb_count field", "pub limb_count: usize"),
        (
            "MultiplicationEvidence struct",
            "pub struct MultiplicationEvidence",
        ),
        (
            "MultiplicationEvidence splitting_factor accessor",
            "pub const fn splitting_factor(&self) -> u32",
        ),
        (
            "MultiplicationEvidence sub_multiplication_count accessor",
            "pub const fn sub_multiplication_count(&self) -> u32",
        ),
        (
            "MultiplicationEvidence landauer_cost_nats_bits accessor (Phase 9 bit pattern)",
            "pub const fn landauer_cost_nats_bits(&self) -> u64",
        ),
        (
            "MultiplicationCertificate shim",
            "pub struct MultiplicationCertificate",
        ),
        (
            "resolver::multiplication module",
            "pub mod multiplication {",
        ),
        (
            "resolver::multiplication::certify free function",
            "pub fn certify<H: crate::enforcement::Hasher<FP_MAX>, const FP_MAX: usize>(",
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
            "Phase C.4 multiplication resolver surface complete: MulContext, \
             MultiplicationEvidence, MultiplicationCertificate, and \
             resolver::multiplication::certify free function all present",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase C.4 multiplication resolver surface has {} missing anchors",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
