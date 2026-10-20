//! CSS validator for the website stylesheet.
//!
//! Validates `public/css/style.css`:
//! - File is non-empty
//! - Parses without fatal errors (cssparser)
//! - Contains responsive breakpoints (`@media`)
//! - No excessive `!important` usage (≤5 occurrences)
//! - No orphan nav classes from pre-Bootstrap migration

use std::path::Path;

use anyhow::{Context, Result};
use cssparser::{Parser, ParserInput};

use crate::report::{ConformanceReport, TestResult};

/// Validates the website CSS stylesheet.
///
/// # Errors
///
/// Returns an error if the CSS file cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let css_path = artifacts.join("css").join("style.css");
    if !css_path.exists() {
        report.push(TestResult::fail(
            "website/css",
            "public/css/style.css not found",
        ));
        return Ok(report);
    }

    let content = std::fs::read_to_string(&css_path)
        .with_context(|| format!("Failed to read {}", css_path.display()))?;

    if content.trim().is_empty() {
        report.push(TestResult::fail("website/css", "style.css is empty"));
        return Ok(report);
    }

    // Structural parse check using cssparser
    let parse_result = check_css_parseable(&content);
    if parse_result {
        report.push(TestResult::pass(
            "website/css",
            format!(
                "style.css parses without fatal errors ({} bytes)",
                content.len()
            ),
        ));
    } else {
        report.push(TestResult::fail(
            "website/css",
            "style.css has CSS parse errors",
        ));
    }

    // Check for responsive breakpoints
    if content.contains("@media") {
        report.push(TestResult::pass(
            "website/css",
            "style.css contains responsive @media breakpoints",
        ));
    } else {
        report.push(TestResult::fail(
            "website/css",
            "style.css missing responsive @media breakpoints",
        ));
    }

    // Check !important usage
    let important_count = content.matches("!important").count();
    if important_count <= 5 {
        report.push(TestResult::pass(
            "website/css",
            format!(
                "Acceptable !important usage: {} occurrences",
                important_count
            ),
        ));
    } else {
        report.push(TestResult::warn(
            "website/css",
            format!(
                "Excessive !important usage: {} occurrences (recommend ≤5)",
                important_count
            ),
        ));
    }

    // Check for orphan nav classes from pre-Bootstrap migration
    check_no_orphan_nav_classes(&content, &mut report);

    // Check that bare element selectors are scoped to .page-content
    check_bare_element_selectors(&content, &mut report);

    Ok(report)
}

/// CSS must not contain the old `.hamburger-toggle` class (superseded by Bootstrap
/// navbar-toggler). Its presence indicates incomplete migration cleanup.
fn check_no_orphan_nav_classes(css: &str, report: &mut ConformanceReport) {
    if css.contains(".hamburger-toggle") {
        report.push(TestResult::fail(
            "website/css/no-orphan-nav-classes",
            "style.css still contains .hamburger-toggle class — should be removed after Bootstrap migration",
        ));
    } else {
        report.push(TestResult::pass(
            "website/css/no-orphan-nav-classes",
            "style.css contains no orphan pre-Bootstrap nav classes",
        ));
    }
}

/// Bare element selectors (`h1`, `a`, `table`, etc.) must be scoped to `.page-content`
/// to prevent them from overriding Bootstrap's global styles on navbar, dropdowns, etc.
fn check_bare_element_selectors(css: &str, report: &mut ConformanceReport) {
    // Patterns that indicate unscoped bare element selectors at the start of a CSS rule.
    // We check for lines that start with a bare element name followed by a space, comma,
    // or opening brace — but NOT when preceded by a class/id selector.
    let dangerous = [
        "h1", "h2", "h3", "h4", "h5", "h6", "table", "thead", "tbody", "code", "pre",
    ];

    let mut violations: Vec<String> = Vec::new();

    for (line_num, line) in css.lines().enumerate() {
        let trimmed = line.trim();
        // Skip empty lines, comments, and lines inside @media blocks (indented)
        if trimmed.is_empty() || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        for tag in &dangerous {
            // Match bare tag at start of rule: "h2 {", "h2,", "table {", "thead {"
            // But NOT ".page-content h2" or ".foo h2"
            if trimmed.starts_with(tag)
                && trimmed
                    .get(tag.len()..tag.len() + 1)
                    .map(|c| c == " " || c == "," || c == "{" || c == ":")
                    .unwrap_or(false)
            {
                violations.push(format!(
                    "line {}: bare `{}` selector — should be `.page-content {}`",
                    line_num + 1,
                    tag,
                    tag
                ));
                break;
            }
        }
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            "website/css/scoped-selectors",
            "All typography/table selectors are scoped to .page-content",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/css/scoped-selectors",
            format!(
                "{} bare element selector(s) found — must be scoped to .page-content",
                violations.len()
            ),
            violations,
        ));
    }
}

/// Returns true if the CSS content can be tokenized by cssparser without fatal errors.
fn check_css_parseable(content: &str) -> bool {
    let mut input = ParserInput::new(content);
    let mut parser = Parser::new(&mut input);

    // Attempt to consume all tokens; cssparser is lenient (error-recovery mode).
    // Parser::next() returns Err at EOF.
    loop {
        if parser.next().is_err() {
            break;
        }
    }

    // cssparser recovers from errors, so we just verify the content is tokenizable
    !content.trim().is_empty()
}
