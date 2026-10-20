//! Bootstrap framework integration validator.
//!
//! Checks that the website correctly integrates Bootstrap 5 for responsive
//! navigation, replacing the custom hamburger toggle with Bootstrap's native
//! navbar collapse pattern.

use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::report::{ConformanceReport, TestResult};

/// Validates Bootstrap 5 integration across the generated website.
///
/// # Errors
///
/// Returns an error if HTML files cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    if !artifacts.exists() {
        report.push(TestResult::warn(
            "website/bootstrap",
            "Artifacts directory not found — skipping Bootstrap validation",
        ));
        return Ok(report);
    }

    check_cdn_link(artifacts, &mut report)?;
    check_navbar_structure(artifacts, &mut report)?;
    check_toggler(artifacts, &mut report)?;
    check_dropdown_menus(artifacts, &mut report)?;
    check_responsive_collapse(artifacts, &mut report)?;
    check_sri_hash(artifacts, &mut report)?;

    Ok(report)
}

/// Every HTML page must include a Bootstrap CSS CDN link.
fn check_cdn_link(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let mut missing: Vec<String> = Vec::new();
    let mut pages_checked = 0u32;

    for entry in WalkDir::new(artifacts)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "html").unwrap_or(false))
    {
        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        pages_checked += 1;

        let lower = content.to_lowercase();
        // Check for a <link referencing bootstrap CSS
        if !(lower.contains("<link") && lower.contains("bootstrap")) {
            let rel = entry
                .path()
                .strip_prefix(artifacts)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .to_string();
            missing.push(rel);
        }
    }

    if pages_checked == 0 {
        report.push(TestResult::warn(
            "website/bootstrap/cdn-link",
            "No HTML files found — skipping Bootstrap CDN check",
        ));
        return Ok(());
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            "website/bootstrap/cdn-link",
            format!("All {pages_checked} pages include Bootstrap CSS CDN link"),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/bootstrap/cdn-link",
            format!("{} page(s) missing Bootstrap CSS CDN link", missing.len()),
            missing,
        ));
    }

    Ok(())
}

/// Homepage must contain Bootstrap navbar structure classes.
fn check_navbar_structure(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let index = artifacts.join("index.html");
    if !index.exists() {
        report.push(TestResult::fail(
            "website/bootstrap/navbar-structure",
            "index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&index)?;
    let mut missing: Vec<String> = Vec::new();

    for class in &["navbar", "navbar-expand-", "navbar-collapse"] {
        if !html.contains(class) {
            missing.push(format!("class containing \"{class}\" not found"));
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            "website/bootstrap/navbar-structure",
            "index.html contains Bootstrap navbar structure (navbar, navbar-expand-*, navbar-collapse)",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/bootstrap/navbar-structure",
            "index.html missing Bootstrap navbar classes",
            missing,
        ));
    }

    Ok(())
}

/// Homepage must contain Bootstrap toggler with data-bs-toggle collapse.
fn check_toggler(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let index = artifacts.join("index.html");
    if !index.exists() {
        report.push(TestResult::fail(
            "website/bootstrap/toggler",
            "index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&index)?;
    let mut missing: Vec<String> = Vec::new();

    if !html.contains("navbar-toggler") {
        missing.push("navbar-toggler class not found".to_string());
    }
    if !html.contains("data-bs-toggle=\"collapse\"") {
        missing.push("data-bs-toggle=\"collapse\" attribute not found".to_string());
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            "website/bootstrap/toggler",
            "index.html contains Bootstrap navbar-toggler with data-bs-toggle collapse",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/bootstrap/toggler",
            "index.html missing Bootstrap toggler elements",
            missing,
        ));
    }

    Ok(())
}

/// Homepage must contain Bootstrap dropdown classes for nav items with children.
fn check_dropdown_menus(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let index = artifacts.join("index.html");
    if !index.exists() {
        report.push(TestResult::fail(
            "website/bootstrap/dropdown-menus",
            "index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&index)?;
    let mut missing: Vec<String> = Vec::new();

    if !html.contains("dropdown-menu") {
        missing.push("dropdown-menu class not found".to_string());
    }
    if !html.contains("dropdown-toggle") {
        missing.push("dropdown-toggle class not found".to_string());
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            "website/bootstrap/dropdown-menus",
            "index.html contains Bootstrap dropdown-menu and dropdown-toggle classes",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/bootstrap/dropdown-menus",
            "index.html missing Bootstrap dropdown classes",
            missing,
        ));
    }

    Ok(())
}

/// Homepage must NOT contain the old custom hamburger-toggle class (clean migration).
fn check_responsive_collapse(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let index = artifacts.join("index.html");
    if !index.exists() {
        report.push(TestResult::fail(
            "website/bootstrap/responsive-collapse",
            "index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&index)?;

    if html.contains("hamburger-toggle") {
        report.push(TestResult::fail(
            "website/bootstrap/responsive-collapse",
            "index.html still contains old hamburger-toggle class — should use Bootstrap navbar-toggler",
        ));
    } else {
        report.push(TestResult::pass(
            "website/bootstrap/responsive-collapse",
            "index.html uses Bootstrap responsive collapse (no legacy hamburger-toggle)",
        ));
    }

    Ok(())
}

/// The Bootstrap CSS integrity hash must match the known-good value for the
/// pinned CDN version. A wrong hash causes browsers to reject the stylesheet,
/// leaving the entire site unstyled.
fn check_sri_hash(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let index = artifacts.join("index.html");
    if !index.exists() {
        report.push(TestResult::fail(
            "website/bootstrap/sri-hash",
            "index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&index)?;

    // Known-good integrity hash for Bootstrap 5.3.3 CSS from jsDelivr CDN.
    let expected = "sha384-QWTKZyjpPEjISv5WaRU9OFeRpok6YctnYmDr5pNlyT2bRjXh0JMhjY6hW+ALEwIH";

    if html.contains(expected) {
        report.push(TestResult::pass(
            "website/bootstrap/sri-hash",
            "Bootstrap CSS integrity hash matches known-good value for 5.3.3",
        ));
    } else {
        report.push(TestResult::fail(
            "website/bootstrap/sri-hash",
            format!("Bootstrap CSS integrity hash mismatch — expected {expected} in index.html"),
        ));
    }

    Ok(())
}
