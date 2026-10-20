//! Documentation accuracy validator.
//!
//! Verifies that auto-generated namespace reference pages accurately
//! reflect the spec (class/property/individual data matches the live ontology).

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

/// Validates that namespace reference pages exist for all namespaces.
///
/// # Errors
///
/// Returns an error if the docs directory cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let namespaces_dir = artifacts.join("docs").join("namespaces");
    if !namespaces_dir.exists() {
        report.push(TestResult::fail(
            "docs/accuracy",
            "public/docs/namespaces/ directory not found",
        ));
        return Ok(report);
    }

    let ontology = uor_ontology::Ontology::full();
    let mut missing: Vec<String> = Vec::new();
    let mut inaccurate: Vec<String> = Vec::new();

    for module in &ontology.namespaces {
        let page_path = namespaces_dir.join(format!("{}.html", module.namespace.prefix));

        if !page_path.exists() {
            missing.push(format!(
                "Missing namespace page: {}.html",
                module.namespace.prefix
            ));
            continue;
        }

        // Verify the page contains all class and property labels
        let content = match std::fs::read_to_string(&page_path) {
            Ok(c) => c,
            Err(e) => {
                inaccurate.push(format!(
                    "Cannot read {}.html: {}",
                    module.namespace.prefix, e
                ));
                continue;
            }
        };

        for class in &module.classes {
            if !content.contains(class.label) && !content.contains(class.id) {
                inaccurate.push(format!(
                    "{}.html missing class: {} ({})",
                    module.namespace.prefix, class.label, class.id
                ));
            }
        }

        for prop in &module.properties {
            if !content.contains(prop.label) && !content.contains(prop.id) {
                inaccurate.push(format!(
                    "{}.html missing property: {} ({})",
                    module.namespace.prefix, prop.label, prop.id
                ));
            }
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            "docs/accuracy",
            format!(
                "All {} namespace reference pages exist",
                ontology.namespaces.len()
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "docs/accuracy",
            "Missing namespace reference pages",
            missing,
        ));
    }

    if inaccurate.is_empty() {
        report.push(TestResult::pass(
            "docs/accuracy",
            "All namespace pages accurately reflect the spec",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "docs/accuracy",
            "Namespace pages missing spec terms",
            inaccurate,
        ));
    }

    Ok(report)
}
