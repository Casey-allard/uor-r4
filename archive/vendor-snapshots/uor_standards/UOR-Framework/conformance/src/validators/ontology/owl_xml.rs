//! OWL RDF/XML artifact validator.
//!
//! Validates that the generated `uor.foundation.owl` file is well-formed
//! and contains the expected ontology structure.

use std::path::Path;

use anyhow::{Context, Result};

use crate::report::{ConformanceReport, TestResult};

/// Validates the OWL RDF/XML artifact for structural correctness.
///
/// Checks that `uor.foundation.owl` exists, is non-empty, contains the
/// expected XML structure, OWL declarations, version string, and all
/// namespace `xmlns:` declarations.
///
/// # Errors
///
/// Returns an error if the artifact file cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let validator = "ontology/owl_xml";

    let owl_path = artifacts.join("uor.foundation.owl");
    if !owl_path.exists() {
        report.push(TestResult::fail(
            validator,
            "uor.foundation.owl not found in artifacts directory",
        ));
        return Ok(report);
    }

    let content = std::fs::read_to_string(&owl_path)
        .with_context(|| format!("Failed to read {}", owl_path.display()))?;

    let mut issues: Vec<String> = Vec::new();

    // Structure
    if content.trim().is_empty() {
        issues.push("File is empty".to_string());
    }
    if !content.contains("<?xml") {
        issues.push("Missing XML declaration".to_string());
    }
    if !content.contains("<rdf:RDF") {
        issues.push("Missing <rdf:RDF root element".to_string());
    }
    if !content.contains("</rdf:RDF>") {
        issues.push("Missing </rdf:RDF> closing element".to_string());
    }

    // OWL declarations
    if !content.contains("<owl:Ontology") {
        issues.push("Missing owl:Ontology declaration".to_string());
    }
    if !content.contains("<owl:Class") {
        issues.push("Missing owl:Class declarations".to_string());
    }
    if !content.contains("<owl:NamedIndividual") {
        issues.push("Missing owl:NamedIndividual declarations".to_string());
    }

    // Version string
    let ontology = uor_ontology::Ontology::full();
    let version_tag = format!("<owl:versionInfo>{}</owl:versionInfo>", ontology.version);
    if !content.contains(&version_tag) {
        issues.push(format!("Missing version string: {}", ontology.version));
    }

    // Namespace xmlns declarations
    for module in &ontology.namespaces {
        let decl = format!(
            "xmlns:{}=\"{}\"",
            module.namespace.prefix, module.namespace.iri
        );
        if !content.contains(&decl) {
            issues.push(format!(
                "Missing xmlns declaration for '{}'",
                module.namespace.prefix
            ));
        }
    }

    if issues.is_empty() {
        report.push(TestResult::pass(
            validator,
            format!(
                "uor.foundation.owl is well-formed ({} bytes)",
                content.len()
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            validator,
            "uor.foundation.owl has structural issues",
            issues,
        ));
    }

    Ok(report)
}
