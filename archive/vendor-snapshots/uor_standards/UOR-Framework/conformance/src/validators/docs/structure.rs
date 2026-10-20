//! Documentation structure validator.
//!
//! Verifies that the generated documentation follows the Diataxis framework:
//! - Reference section: `public/docs/namespaces/` (one page per namespace)
//! - Explanation section: `public/docs/concepts/` (concept pages)
//! - How-to section: `public/docs/guides/` (guide pages)
//! - Entry point: `public/docs/index.html`

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

/// Required files in the docs structure.
const REQUIRED_SECTIONS: &[(&str, &str)] = &[
    ("docs/index.html", "Documentation index page"),
    (
        "docs/namespaces",
        "Namespace reference section (Diataxis: reference)",
    ),
    (
        "docs/concepts",
        "Concept explanation section (Diataxis: explanation)",
    ),
    ("docs/guides", "How-to guide section (Diataxis: how-to)"),
];

/// Validates the Diataxis documentation structure.
///
/// # Errors
///
/// Returns an error if the artifacts directory cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    // Check required sections exist
    for (rel_path, description) in REQUIRED_SECTIONS {
        let path = artifacts.join(rel_path);
        if path.exists() {
            report.push(TestResult::pass(
                "docs/structure",
                format!("Present: {} ({})", rel_path, description),
            ));
        } else {
            report.push(TestResult::fail(
                "docs/structure",
                format!("Missing: {} ({})", rel_path, description),
            ));
        }
    }

    // Check all namespace reference pages (derived from live ontology)
    let ontology = uor_ontology::Ontology::full();
    let mut missing_namespaces: Vec<String> = Vec::new();
    for module in &ontology.namespaces {
        let page = artifacts
            .join("docs")
            .join("namespaces")
            .join(format!("{}.html", module.namespace.prefix));
        if !page.exists() {
            missing_namespaces.push(format!("{}.html", module.namespace.prefix));
        }
    }

    if missing_namespaces.is_empty() {
        report.push(TestResult::pass(
            "docs/structure",
            format!(
                "All {} namespace reference pages present",
                ontology.namespaces.len()
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "docs/structure",
            "Missing namespace reference pages",
            missing_namespaces,
        ));
    }

    // Check that concepts/ and guides/ are non-empty
    let concepts_dir = artifacts.join("docs").join("concepts");
    if concepts_dir.exists() {
        let count = std::fs::read_dir(&concepts_dir)
            .map(|rd| rd.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        if count > 0 {
            report.push(TestResult::pass(
                "docs/structure",
                format!("concepts/ section has {} pages", count),
            ));
        } else {
            report.push(TestResult::fail(
                "docs/structure",
                "concepts/ section is empty (Diataxis explanation section required)",
            ));
        }
    }

    let guides_dir = artifacts.join("docs").join("guides");
    if guides_dir.exists() {
        let count = std::fs::read_dir(&guides_dir)
            .map(|rd| rd.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        if count > 0 {
            report.push(TestResult::pass(
                "docs/structure",
                format!("guides/ section has {} pages", count),
            ));
        } else {
            report.push(TestResult::fail(
                "docs/structure",
                "guides/ section is empty (Diataxis how-to section required)",
            ));
        }
    }

    Ok(report)
}
