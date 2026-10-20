//! Website coverage validator.
//!
//! Verifies that the website covers all ontology terms:
//! - Every namespace has a landing page under `public/namespaces/<prefix>/index.html`
//! - `search-index.json` contains all class labels
//! - `sitemap.xml` is present

use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::report::{ConformanceReport, TestResult};

/// Validates that all ontology namespaces have website pages and search coverage.
///
/// # Errors
///
/// Returns an error if artifact files cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    check_namespace_pages(artifacts, &mut report);
    check_search_index(artifacts, &mut report)?;
    check_sitemap(artifacts, &mut report);

    Ok(report)
}

/// Checks that each namespace has a `public/namespaces/<prefix>/index.html`.
fn check_namespace_pages(artifacts: &Path, report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let mut missing: Vec<String> = Vec::new();

    for module in &ontology.namespaces {
        let page = artifacts
            .join("namespaces")
            .join(module.namespace.prefix)
            .join("index.html");
        if !page.exists() {
            missing.push(format!("namespaces/{}/index.html", module.namespace.prefix));
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            "website/coverage",
            format!(
                "All {} namespace pages present in website",
                uor_ontology::counts::NAMESPACES
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/coverage",
            "Missing namespace pages in website",
            missing,
        ));
    }
}

/// Checks that `search-index.json` contains all class labels.
///
/// # Errors
///
/// Returns an error if the search index file cannot be read.
fn check_search_index(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let index_path = artifacts.join("search-index.json");
    if !index_path.exists() {
        report.push(TestResult::fail(
            "website/coverage",
            "search-index.json not found in public/",
        ));
        return Ok(());
    }

    let content = std::fs::read_to_string(&index_path)
        .with_context(|| format!("Failed to read {}", index_path.display()))?;

    let index: Value =
        serde_json::from_str(&content).with_context(|| "Failed to parse search-index.json")?;

    let ontology = uor_ontology::Ontology::full();
    let mut missing: Vec<String> = Vec::new();

    let index_str = index.to_string();
    for module in &ontology.namespaces {
        for class in &module.classes {
            if !index_str.contains(class.label) && !index_str.contains(class.id) {
                missing.push(format!("{} ({})", class.label, class.id));
            }
        }
    }

    if missing.is_empty() {
        let class_count = ontology
            .namespaces
            .iter()
            .map(|m| m.classes.len())
            .sum::<usize>();
        report.push(TestResult::pass(
            "website/coverage",
            format!("All {} classes present in search-index.json", class_count),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/coverage",
            format!("{} classes missing from search-index.json", missing.len()),
            missing,
        ));
    }

    Ok(())
}

/// Checks that `sitemap.xml` is present.
fn check_sitemap(artifacts: &Path, report: &mut ConformanceReport) {
    let sitemap = artifacts.join("sitemap.xml");
    if sitemap.exists() {
        report.push(TestResult::pass(
            "website/coverage",
            "sitemap.xml present in public/",
        ));
    } else {
        report.push(TestResult::fail(
            "website/coverage",
            "sitemap.xml not found in public/",
        ));
    }
}
