//! Lean 4 individual-proof conformance validator.
//!
//! Reads the `lean4/.uor-unproven.json` manifest produced by
//! `uor-lean-codegen` and fails if it is non-empty. The manifest
//! records every named individual that could not be fully derived
//! from its ontology assertions:
//!
//! - fields missing required assertions
//! - IRI references that don't resolve to any known individual /
//!   enum variant
//! - assertion value types that don't match the declared field type
//! - self-references or cycles that can't be represented as finite
//!   Lean data
//! - blocked (mutual-cluster) structures lacking assertions
//!
//! This check is how "conformance-first, no fallbacks" is enforced:
//! Lake build succeeds (typed defs fall back to `Inhabited` defaults
//! or Unit orphan placeholders), but this validator fails if any
//! individual is not fully proven against its declared class.

use std::path::Path;

use anyhow::{Context, Result};

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "lean4/individual_proof";

/// Validates that every non-enum named individual in the ontology is
/// emitted into Lean as a fully-proven typed `def`. A "fully proven"
/// individual has every field either directly assigned from an
/// ontology assertion or covered by an `Option`/`Array` absent-value
/// convention.
///
/// # Errors
///
/// Returns an error if the workspace layout is unexpected or the
/// manifest file is present but unparseable.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let manifest_path = workspace.join("lean4").join(".uor-unproven.json");

    if !manifest_path.exists() {
        report.push(TestResult::fail(
            VALIDATOR,
            "Missing lean4/.uor-unproven.json \u{2014} codegen did not emit the manifest; \
             run `cargo run --bin uor-lean`",
        ));
        return Ok(report);
    }

    let raw = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read {}", manifest_path.display()))?;
    let manifest: uor_lean_codegen::individuals::UnprovenManifest = serde_json::from_str(&raw)
        .with_context(|| {
            format!(
                "Failed to parse {} as UnprovenManifest JSON",
                manifest_path.display()
            )
        })?;

    if manifest.is_clean() {
        report.push(TestResult::pass(
            VALIDATOR,
            "All non-enum named individuals are fully proven against their declared classes",
        ));
        return Ok(report);
    }

    // Build a concise human-readable list of the first ~50 gaps.
    let mut details: Vec<String> = Vec::new();
    for f in manifest.unproven_fields.iter().take(50) {
        details.push(format!(
            "{} :: {} \u{2014} {}",
            f.individual,
            f.field,
            reason_short(&f.reason)
        ));
    }
    for iri in manifest.orphan_individuals.iter().take(10) {
        details.push(format!("{iri} :: orphan (no Inhabited path)"));
    }
    if manifest.unproven_fields.len() > 50 {
        details.push(format!(
            "\u{2026} ({} more unproven field(s) omitted)",
            manifest.unproven_fields.len() - 50
        ));
    }

    let indiv_count = manifest.unproven_individual_count();
    report.push(TestResult::fail_with_details(
        VALIDATOR,
        format!(
            "{} individuals unproven across {} field gaps ({} orphan placeholders)",
            indiv_count,
            manifest.unproven_fields.len(),
            manifest.orphan_individuals.len()
        ),
        details,
    ));

    Ok(report)
}

/// Short human-readable rendering of an `UnprovenReason`.
fn reason_short(reason: &uor_lean_codegen::individuals::UnprovenReason) -> String {
    use uor_lean_codegen::individuals::UnprovenReason;
    match reason {
        UnprovenReason::NoAssertion => "no assertion".to_string(),
        UnprovenReason::IriRefUnresolvable { iri } => format!("unresolved IRI <{iri}>"),
        UnprovenReason::TypeMismatch {
            value_kind,
            field_type,
        } => format!("type mismatch: {value_kind} !~ {field_type}"),
        UnprovenReason::CyclicSelfReference => "cyclic self-reference".to_string(),
        UnprovenReason::BlockedWithoutAssertion { field_type } => {
            format!("blocked field type {field_type} lacks assertion")
        }
    }
}
