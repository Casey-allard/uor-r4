//! Phase 9e conformance category: `HostTypesDiscipline`.
//!
//! Asserts the bounds and impl shapes that Phase 9 introduces:
//!
//! 1. `HostTypes::Decimal: DecimalTranscendental` is bounded.
//! 2. `DecimalTranscendental` carries the closed arithmetic + transcendental
//!    surface (Copy / Default / PartialOrd / Add/Sub/Mul/Div / ln / exp /
//!    sqrt / from_bits / to_bits / from_u32 / from_u64 / as_u64_saturating
//!    / entropy_term_nats).
//! 3. `impl DecimalTranscendental for f64` and `impl DecimalTranscendental
//!    for f32` are both present.
//! 4. The `transcendentals` module dispatches via `DecimalTranscendental`
//!    rather than a fixed f64 path.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/host_types_discipline";

/// Runs the Phase 9e validator.
///
/// # Errors
///
/// Returns an error if `foundation/src/lib.rs` or
/// `foundation/src/enforcement.rs` cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let lib_src = std::fs::read_to_string(workspace.join("foundation/src/lib.rs"))?;
    let enforcement_src = std::fs::read_to_string(workspace.join("foundation/src/enforcement.rs"))?;

    let mut failures: Vec<String> = Vec::new();

    // 1. HostTypes::Decimal carries the supertrait bound.
    if !lib_src.contains("type Decimal: DecimalTranscendental") {
        failures
            .push("HostTypes::Decimal must carry the `: DecimalTranscendental` bound".to_string());
    }

    // 2. DecimalTranscendental trait surface — every required member.
    let required_members: &[&str] = &[
        "fn ln(self) -> Self",
        "fn exp(self) -> Self",
        "fn sqrt(self) -> Self",
        "fn from_bits(bits: u64) -> Self",
        "fn to_bits(self) -> u64",
        "fn from_u32(value: u32) -> Self",
        "fn from_u64(value: u64) -> Self",
        "fn as_u64_saturating(self) -> u64",
        "fn entropy_term_nats(self) -> Self",
    ];
    for member in required_members {
        if !lib_src.contains(member) {
            failures.push(format!(
                "DecimalTranscendental must declare `{member}` (missing in foundation/src/lib.rs)"
            ));
        }
    }

    // 3. f64 + f32 impls present.
    if !lib_src.contains("impl DecimalTranscendental for f64 {") {
        failures.push("`impl DecimalTranscendental for f64` is missing".to_string());
    }
    if !lib_src.contains("impl DecimalTranscendental for f32 {") {
        failures.push("`impl DecimalTranscendental for f32` is missing".to_string());
    }

    // 4. transcendentals module dispatches generically.
    let dispatches_generically = enforcement_src.contains("pub fn ln<D: DecimalTranscendental>")
        && enforcement_src.contains("pub fn exp<D: DecimalTranscendental>")
        && enforcement_src.contains("pub fn sqrt<D: DecimalTranscendental>")
        && enforcement_src.contains("pub fn entropy_term_nats<D: DecimalTranscendental>");
    if !dispatches_generically {
        failures.push(
            "transcendentals::{ln,exp,sqrt,entropy_term_nats} must be generic over \
             `D: DecimalTranscendental` (not pinned to f64)"
                .to_string(),
        );
    }

    if failures.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "HostTypesDiscipline: HostTypes::Decimal: DecimalTranscendental + libm impls + \
             generic transcendentals dispatch",
        ));
    } else {
        let mut msg = String::from("HostTypesDiscipline drift:");
        for f in &failures {
            msg.push_str(&format!("\n    - {f}"));
        }
        report.push(TestResult::fail(VALIDATOR, msg));
    }

    Ok(report)
}
