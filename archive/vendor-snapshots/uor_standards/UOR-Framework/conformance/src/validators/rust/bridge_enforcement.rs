//! Phase E (target §4.6): bridge namespace enforcement.
//!
//! Pins the bridge-namespace completeness additions:
//! - `InteractionDeclarationBuilder::validate` + `validate_const` against
//!   `conformance:InteractionShape` (+ the `InteractionShape` result type).
//! - Observability subscribe API gated behind `#[cfg(feature = "observability")]`.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/bridge_enforcement";

/// Runs the Phase E bridge-enforcement check.
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
        (
            "InteractionShape result type",
            "pub struct InteractionShape {",
        ),
        (
            "InteractionDeclarationBuilder::validate",
            "pub fn validate(&self) -> Result<Validated<InteractionShape>",
        ),
        (
            "InteractionDeclarationBuilder::validate_const",
            // Wrapped across lines by rustfmt; anchor the return type only.
            "Result<Validated<InteractionShape, CompileTime>, ShapeViolation>",
        ),
        (
            "subscribe_trace_events (observability-gated)",
            "pub fn subscribe_trace_events",
        ),
        (
            "ObservabilitySubscription (observability-gated)",
            "pub struct ObservabilitySubscription<F",
        ),
        (
            "observability feature gate on subscribe",
            "#[cfg(feature = \"observability\")]",
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
            "Phase E bridge enforcement: InteractionDeclarationBuilder::validate/validate_const \
             against conformance:InteractionShape, subscribe_trace_events gated behind \
             #[cfg(feature = \"observability\")] (target §4.6 / §7.4)",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase E bridge enforcement has {} missing anchors",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
