//! SHACL shapes artifact validator.
//!
//! Validates that the generated `uor.shapes.ttl` file is well-formed and
//! contains the expected number of NodeShapes.

use std::path::Path;

use anyhow::{Context, Result};

use crate::report::{ConformanceReport, TestResult};

/// Validates the SHACL shapes artifact for structural correctness.
///
/// Checks that `uor.shapes.ttl` exists, is non-empty, contains the
/// expected prefix declarations, and has exactly one `sh:NodeShape` per
/// ontology class.
///
/// # Errors
///
/// Returns an error if the artifact file cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let validator = "ontology/shacl_shapes";

    let shacl_path = artifacts.join("uor.shapes.ttl");
    if !shacl_path.exists() {
        report.push(TestResult::fail(
            validator,
            "uor.shapes.ttl not found in artifacts directory",
        ));
        return Ok(report);
    }

    let content = std::fs::read_to_string(&shacl_path)
        .with_context(|| format!("Failed to read {}", shacl_path.display()))?;

    let mut issues: Vec<String> = Vec::new();

    // Structure
    if content.trim().is_empty() {
        issues.push("File is empty".to_string());
    }
    if !content.contains("@prefix sh:") {
        issues.push("Missing @prefix sh: declaration".to_string());
    }
    if !content.contains("@prefix uor-sh:") {
        issues.push("Missing @prefix uor-sh: declaration".to_string());
    }

    // NodeShape count
    let ontology = uor_ontology::Ontology::full();
    let expected = ontology.class_count();
    let actual = content.matches("sh:NodeShape").count();
    if actual != expected {
        issues.push(format!(
            "NodeShape count: {} (expected {})",
            actual, expected
        ));
    }

    if issues.is_empty() {
        report.push(TestResult::pass(
            validator,
            format!(
                "uor.shapes.ttl is well-formed \
                 ({} bytes, {} NodeShapes)",
                content.len(),
                actual
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            validator,
            "uor.shapes.ttl has structural issues",
            issues,
        ));
    }

    Ok(report)
}
