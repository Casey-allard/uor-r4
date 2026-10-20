//! v0.2.2 Phase C validator: Witt tower completeness.
//!
//! Asserts that every `schema:WittLevel` individual declared in the ontology
//! has a corresponding `pub struct Wn;` marker struct emitted into
//! `foundation/src/enforcement.rs`, plus a `RingOp<Wn>` binary impl and a
//! `UnaryRingOp<Wn>` impl. Drift is a hard failure: adding a Witt level
//! individual without regenerating the foundation crate (or vice versa) is
//! caught here.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/witt_tower_completeness";

/// Runs the Witt tower completeness check.
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

    let ontology = uor_ontology::Ontology::full();
    let witt_level_type = "https://uor.foundation/schema/WittLevel";
    let mut levels: Vec<String> = Vec::new();
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            if ind.type_ == witt_level_type {
                if let Some(name) = ind.id.rsplit('/').next() {
                    levels.push(name.to_string());
                }
            }
        }
    }

    let mut missing: Vec<String> = Vec::new();
    for lvl in &levels {
        let struct_marker = format!("pub struct {lvl};");
        let ringop_binary = format!("impl RingOp<{lvl}>");
        let unary_marker = format!("impl UnaryRingOp<{lvl}>");
        if !content.contains(&struct_marker) {
            missing.push(format!("struct {lvl}"));
            continue;
        }
        if !content.contains(&ringop_binary) {
            missing.push(format!("RingOp<{lvl}>"));
            continue;
        }
        if !content.contains(&unary_marker) {
            missing.push(format!("UnaryRingOp<{lvl}>"));
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "Witt tower complete: all {} `schema:WittLevel` individuals have \
                 marker struct + RingOp + UnaryRingOp impls",
                levels.len()
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Witt tower drift: {} missing codegen artifacts for declared WittLevel individuals",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
