//! Website new-page existence validator.
//!
//! Checks that all new PRISM-era pages exist and contain the expected structural
//! elements: concepts index, explore page, pipeline page, identities page, and
//! download page.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

/// Validates that all new PRISM pages exist and have required content.
///
/// # Errors
///
/// Returns an error if artifact files cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    check_concepts_page(artifacts, &mut report)?;
    check_explore_page(artifacts, &mut report)?;
    check_pipeline_page(artifacts, &mut report)?;
    check_identities_page(artifacts, &mut report)?;
    check_download_page(artifacts, &mut report)?;
    check_about_page(artifacts, &mut report)?;

    Ok(report)
}

/// concepts/index.html must exist and contain concept page links.
fn check_concepts_page(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let page = artifacts.join("concepts").join("index.html");
    if !page.exists() {
        report.push(TestResult::fail(
            "website/pages/concepts",
            "concepts/index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&page)?;
    let required_count = uor_ontology::counts::CONCEPT_PAGES;
    let found = count_concept_hrefs(&html);

    if found >= required_count {
        report.push(TestResult::pass(
            "website/pages/concepts",
            format!(
                "concepts/index.html contains {found} concept links (required: {required_count})"
            ),
        ));
    } else {
        report.push(TestResult::fail(
            "website/pages/concepts",
            format!(
                "concepts/index.html has only {found} concept links; expected at least {required_count}"
            ),
        ));
    }

    Ok(())
}

/// Count occurrences of `href=` attributes that reference `/concepts/` paths.
fn count_concept_hrefs(html: &str) -> usize {
    html.match_indices("href=")
        .filter(|(pos, _)| {
            let tail = &html[*pos..];
            // Check that within the next 300 chars there's "concepts/"
            // (300 accommodates PUBLIC_BASE_PATH prefixed URLs)
            tail.get(..tail.len().min(300))
                .map(|s| s.contains("concepts/"))
                .unwrap_or(false)
        })
        .count()
}

/// explore/index.html must exist and contain the ontology explorer element.
fn check_explore_page(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let page = artifacts.join("explore").join("index.html");
    if !page.exists() {
        report.push(TestResult::fail(
            "website/pages/explore",
            "explore/index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&page)?;

    if html.contains("id=\"ontology-explorer\"") {
        report.push(TestResult::pass(
            "website/pages/explore",
            "explore/index.html contains ontology explorer element",
        ));
    } else {
        report.push(TestResult::fail(
            "website/pages/explore",
            "explore/index.html missing id=\"ontology-explorer\" element",
        ));
    }

    Ok(())
}

/// pipeline/index.html must exist and contain all three stage section IDs.
fn check_pipeline_page(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let page = artifacts.join("pipeline").join("index.html");
    if !page.exists() {
        report.push(TestResult::fail(
            "website/pages/pipeline",
            "pipeline/index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&page)?;
    let required = [
        "id=\"stage-define\"",
        "id=\"stage-resolve\"",
        "id=\"stage-certify\"",
    ];
    let missing: Vec<String> = required
        .iter()
        .filter(|&&id| !html.contains(id))
        .map(|&s| s.to_string())
        .collect();

    if missing.is_empty() {
        report.push(TestResult::pass(
            "website/pages/pipeline",
            "pipeline/index.html contains all three stage section IDs",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/pages/pipeline",
            "pipeline/index.html missing stage section IDs",
            missing,
        ));
    }

    Ok(())
}

/// identities/index.html must exist and contain identity table rows.
fn check_identities_page(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let page = artifacts.join("identities").join("index.html");
    if !page.exists() {
        report.push(TestResult::fail(
            "website/pages/identities",
            "identities/index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&page)?;

    if html.contains("class=\"identity-row\"") {
        report.push(TestResult::pass(
            "website/pages/identities",
            "identities/index.html contains identity table rows",
        ));
    } else {
        report.push(TestResult::fail(
            "website/pages/identities",
            "identities/index.html missing class=\"identity-row\" table rows",
        ));
    }

    Ok(())
}

/// about/index.html must exist.
fn check_about_page(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let page = artifacts.join("about").join("index.html");
    if page.exists() {
        report.push(TestResult::pass(
            "website/pages/about",
            "about/index.html exists in generated website",
        ));
    } else {
        report.push(TestResult::fail(
            "website/pages/about",
            "about/index.html not found in generated website",
        ));
    }
    Ok(())
}

/// download/index.html must exist and reference all artifact formats.
fn check_download_page(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let page = artifacts.join("download").join("index.html");
    if !page.exists() {
        report.push(TestResult::fail(
            "website/pages/download",
            "download/index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&page)?;
    let required = [
        ".jsonld",
        ".ttl",
        ".nt",
        ".owl",
        ".schema.json",
        ".shapes.ttl",
        ".ebnf",
    ];
    let missing: Vec<String> = required
        .iter()
        .filter(|&&ext| !html.contains(ext))
        .map(|&s| s.to_string())
        .collect();

    if missing.is_empty() {
        report.push(TestResult::pass(
            "website/pages/download",
            "download/index.html references all artifact formats \
             (.jsonld, .ttl, .nt, .owl, .schema.json, .shapes.ttl, .ebnf)",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/pages/download",
            "download/index.html missing artifact format references",
            missing,
        ));
    }

    Ok(())
}
