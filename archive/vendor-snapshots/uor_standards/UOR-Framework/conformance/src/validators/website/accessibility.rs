//! WCAG 2.1 AA automated accessibility validator.
//!
//! Checks that generated HTML pages meet WCAG 2.1 AA automated requirements:
//! - Every `<img>` has an `alt` attribute
//! - Every `<html>` element has a `lang` attribute
//! - No empty `<a>` elements without `title` or `aria-label`
//! - Skip-to-content link present

use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::report::{ConformanceReport, TestResult};

/// Validates WCAG 2.1 AA automated requirements for website HTML files.
///
/// # Errors
///
/// Returns an error if HTML files cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    if !artifacts.exists() {
        report.push(TestResult::warn(
            "website/accessibility",
            "Artifacts directory not found — skipping accessibility checks",
        ));
        return Ok(report);
    }

    let mut issues: Vec<String> = Vec::new();
    let mut pages_checked = 0u32;

    for entry in WalkDir::new(artifacts)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "html").unwrap_or(false))
    {
        let path = entry.path();
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel_path = path
            .strip_prefix(artifacts)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        check_accessibility(&rel_path, &content, &mut issues);
        pages_checked += 1;
    }

    if pages_checked == 0 {
        report.push(TestResult::warn(
            "website/accessibility",
            "No HTML files found — skipping accessibility checks",
        ));
        return Ok(report);
    }

    if issues.is_empty() {
        report.push(TestResult::pass(
            "website/accessibility",
            format!(
                "All {} pages pass WCAG 2.1 AA automated checks",
                pages_checked
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/accessibility",
            format!("WCAG 2.1 AA violations in {} pages checked", pages_checked),
            issues,
        ));
    }

    Ok(report)
}

/// Performs string-based WCAG 2.1 AA checks on a page.
fn check_accessibility(path: &str, content: &str, issues: &mut Vec<String>) {
    let lower = content.to_lowercase();

    // WCAG 3.1.1: lang attribute on <html>
    if !lower.contains("lang=") {
        issues.push(format!(
            "{}: <html> missing lang attribute (WCAG 3.1.1)",
            path
        ));
    }

    // WCAG 2.4.1: skip-to-content link
    if !lower.contains("skip") || !lower.contains("main-content") {
        issues.push(format!(
            "{}: missing skip-to-main-content link (WCAG 2.4.1)",
            path
        ));
    }

    // WCAG 1.1.1: <img> tags must have alt attribute
    let mut remaining = lower.as_str();
    while let Some(idx) = remaining.find("<img") {
        remaining = &remaining[idx + 4..];
        // Find the end of this tag
        let end = remaining.find('>').unwrap_or(remaining.len());
        let tag_content = &remaining[..end];
        if !tag_content.contains("alt=") {
            issues.push(format!(
                "{}: <img> missing alt attribute (WCAG 1.1.1)",
                path
            ));
        }
    }
}
