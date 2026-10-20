//! Documentation completeness validator.
//!
//! Verifies that every ontology term (class, property, individual) is documented
//! in at least one file under `public/docs/`.

use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::report::{ConformanceReport, TestResult};

/// Validates that every spec term has documentation coverage in `public/docs/`.
///
/// # Errors
///
/// Returns an error if the docs directory cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let docs_dir = artifacts.join("docs");
    if !docs_dir.exists() {
        report.push(TestResult::fail(
            "docs/completeness",
            "public/docs/ directory not found — run uor-docs first",
        ));
        return Ok(report);
    }

    // Collect all HTML content from the docs output
    let mut all_content = String::new();
    for entry in WalkDir::new(&docs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "html").unwrap_or(false))
    {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            all_content.push_str(&content);
        }
    }

    if all_content.is_empty() {
        report.push(TestResult::fail(
            "docs/completeness",
            "No HTML files found in public/docs/",
        ));
        return Ok(report);
    }

    let ontology = uor_ontology::Ontology::full();
    let mut missing_classes: Vec<String> = Vec::new();
    let mut missing_properties: Vec<String> = Vec::new();
    let mut missing_individuals: Vec<String> = Vec::new();

    // Check every class IRI appears in the docs
    for module in &ontology.namespaces {
        for class in &module.classes {
            if !all_content.contains(class.id) && !all_content.contains(class.label) {
                missing_classes.push(format!("{} ({})", class.label, class.id));
            }
        }
        for prop in &module.properties {
            if !all_content.contains(prop.id) && !all_content.contains(prop.label) {
                missing_properties.push(format!("{} ({})", prop.label, prop.id));
            }
        }
        for ind in &module.individuals {
            if !all_content.contains(ind.id) && !all_content.contains(ind.label) {
                missing_individuals.push(format!("{} ({})", ind.label, ind.id));
            }
        }
    }

    if missing_classes.is_empty() {
        report.push(TestResult::pass(
            "docs/completeness",
            format!(
                "All {} ontology classes are documented",
                ontology.class_count()
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "docs/completeness",
            format!(
                "{} classes missing from documentation",
                missing_classes.len()
            ),
            missing_classes,
        ));
    }

    if missing_properties.is_empty() {
        report.push(TestResult::pass(
            "docs/completeness",
            format!(
                "All {} ontology properties are documented",
                ontology.property_count() - 1
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "docs/completeness",
            format!(
                "{} properties missing from documentation",
                missing_properties.len()
            ),
            missing_properties,
        ));
    }

    if missing_individuals.is_empty() {
        report.push(TestResult::pass(
            "docs/completeness",
            format!(
                "All {} named individuals are documented",
                ontology.individual_count()
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "docs/completeness",
            format!(
                "{} individuals missing from documentation",
                missing_individuals.len()
            ),
            missing_individuals,
        ));
    }

    Ok(report)
}
