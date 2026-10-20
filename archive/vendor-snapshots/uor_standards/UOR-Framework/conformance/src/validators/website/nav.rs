//! Website navigation structure validator.
//!
//! Checks that every page exposes the PRISM pipeline stage names, that namespace
//! pages are grouped by space, and that namespace pages appear in assembly order.

use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::report::{ConformanceReport, TestResult};

/// Validates the website navigation structure.
///
/// # Errors
///
/// Returns an error if artifact files cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    check_prism_structure(artifacts, &mut report)?;
    check_space_group_labels(artifacts, &mut report)?;
    check_assembly_order(artifacts, &mut report)?;

    Ok(report)
}

/// Every HTML file must contain "define", "resolve", and "certify" (pipeline stage names
/// embedded in nav links). Docs pages are included because they now use the same shared nav.
fn check_prism_structure(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let pipeline_page = artifacts.join("pipeline").join("index.html");
    if !pipeline_page.exists() {
        report.push(TestResult::fail(
            "website/nav/prism-structure",
            "pipeline/index.html not found in generated website",
        ));
        return Ok(());
    }

    let mut missing_files: Vec<String> = Vec::new();

    for entry in WalkDir::new(artifacts)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "html").unwrap_or(false))
    {
        let html = match std::fs::read_to_string(entry.path()) {
            Ok(s) => s.to_lowercase(),
            Err(_) => continue,
        };
        let has_define = html.contains("define");
        let has_resolve = html.contains("resolve");
        let has_certify = html.contains("certify");
        if !has_define || !has_resolve || !has_certify {
            missing_files.push(format!(
                "{}: define={has_define}, resolve={has_resolve}, certify={has_certify}",
                entry.path().display()
            ));
        }
    }

    if missing_files.is_empty() {
        report.push(TestResult::pass(
            "website/nav/prism-structure",
            "All HTML files contain PRISM pipeline stage names (define/resolve/certify)",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/nav/prism-structure",
            format!(
                "{} HTML file(s) missing PRISM stage names in nav",
                missing_files.len()
            ),
            missing_files,
        ));
    }

    Ok(())
}

/// Website pages must contain data-space attributes for all three spaces and the cert space.
///
/// pipeline/index.html checks for kernel, bridge, cert (the three PRISM stages).
/// explore/index.html checks for kernel, bridge, user (the three ontology spaces).
fn check_space_group_labels(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let pipeline_page = artifacts.join("pipeline").join("index.html");
    let explore_page = artifacts.join("explore").join("index.html");
    if !pipeline_page.exists() || !explore_page.exists() {
        report.push(TestResult::fail(
            "website/nav/space-groups",
            "pipeline/index.html or explore/index.html not found in generated website",
        ));
        return Ok(());
    }

    let pipeline_html = std::fs::read_to_string(&pipeline_page)?;
    let explore_html = std::fs::read_to_string(&explore_page)?;
    let mut missing: Vec<String> = Vec::new();

    // Pipeline page covers the three PRISM stages (kernel, bridge, cert)
    for space in &["kernel", "bridge", "cert"] {
        let attr = format!("data-space=\"{space}\"");
        if !pipeline_html.contains(&attr) {
            missing.push(format!("pipeline/index.html: {attr}"));
        }
    }
    // Explore page covers the three ontology spaces (kernel, bridge, user)
    for space in &["kernel", "bridge", "user"] {
        let attr = format!("data-space=\"{space}\"");
        if !explore_html.contains(&attr) {
            missing.push(format!("explore/index.html: {attr}"));
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            "website/nav/space-groups",
            "pipeline and explore pages contain data-space attributes for all spaces",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/nav/space-groups",
            "website pages missing data-space attributes",
            missing,
        ));
    }

    Ok(())
}

/// namespaces/index.html must list namespace prefixes in assembly order.
fn check_assembly_order(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let ns_index = artifacts.join("namespaces").join("index.html");
    if !ns_index.exists() {
        report.push(TestResult::warn(
            "website/nav/assembly-order",
            "namespaces/index.html not found; skipping assembly-order check",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&ns_index)?;
    let ontology = uor_ontology::Ontology::full();
    let prefixes: Vec<&str> = ontology
        .namespaces
        .iter()
        .map(|m| m.namespace.prefix)
        .collect();

    let mut last_pos: usize = 0;
    let mut out_of_order: Vec<String> = Vec::new();

    for prefix in &prefixes {
        match html[last_pos..].find(prefix) {
            Some(rel_pos) => {
                last_pos += rel_pos + prefix.len();
            }
            None => {
                out_of_order.push(format!(
                    "prefix '{prefix}' not found after position {last_pos}"
                ));
            }
        }
    }

    if out_of_order.is_empty() {
        report.push(TestResult::pass(
            "website/nav/assembly-order",
            "namespaces/index.html lists namespaces in assembly order",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/nav/assembly-order",
            "namespaces/index.html does not list namespaces in assembly order",
            out_of_order,
        ));
    }

    Ok(())
}
