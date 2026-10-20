//! Website visual elements (SVG) validator.
//!
//! Checks that SVG diagrams are present on the pipeline page, explore page,
//! namespace pages, and identities page.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

/// Validates that SVG visual elements are present on key pages.
///
/// # Errors
///
/// Returns an error if artifact files cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    check_pipeline_svg(artifacts, &mut report)?;
    check_namespace_graph_svg(artifacts, &mut report)?;
    check_class_hierarchy_svgs(artifacts, &mut report)?;
    check_identity_distribution_svg(artifacts, &mut report)?;

    Ok(report)
}

/// pipeline/index.html must contain the PRISM pipeline SVG.
fn check_pipeline_svg(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let page = artifacts.join("pipeline").join("index.html");
    if !page.exists() {
        report.push(TestResult::fail(
            "website/visual/svg-pipeline",
            "pipeline/index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&page)?;
    let has_class = html.contains("class=\"prism-pipeline\"");
    let has_svg = html.contains("<svg");

    if has_class && has_svg {
        report.push(TestResult::pass(
            "website/visual/svg-pipeline",
            "pipeline/index.html contains PRISM pipeline SVG diagram",
        ));
    } else {
        report.push(TestResult::fail(
            "website/visual/svg-pipeline",
            format!(
                "pipeline/index.html missing pipeline SVG (class=\"prism-pipeline\": {has_class}, <svg: {has_svg})"
            ),
        ));
    }

    Ok(())
}

/// explore/index.html must contain the namespace dependency graph SVG.
fn check_namespace_graph_svg(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let page = artifacts.join("explore").join("index.html");
    if !page.exists() {
        report.push(TestResult::fail(
            "website/visual/svg-namespace-graph",
            "explore/index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&page)?;
    let has_class = html.contains("class=\"ns-dependency-graph\"");
    let has_svg = html.contains("<svg");

    if has_class && has_svg {
        report.push(TestResult::pass(
            "website/visual/svg-namespace-graph",
            "explore/index.html contains namespace dependency graph SVG",
        ));
    } else {
        report.push(TestResult::fail(
            "website/visual/svg-namespace-graph",
            format!(
                "explore/index.html missing dependency graph SVG (class=\"ns-dependency-graph\": {has_class}, <svg: {has_svg})"
            ),
        ));
    }

    Ok(())
}

/// Namespace pages with >= MIN_HIERARCHY_CLASSES classes must contain a class hierarchy SVG.
fn check_class_hierarchy_svgs(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let min_classes = uor_ontology::counts::MIN_HIERARCHY_CLASSES;
    let ontology = uor_ontology::Ontology::full();
    let mut failures: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for module in &ontology.namespaces {
        if module.classes.len() < min_classes {
            continue;
        }
        checked += 1;
        let page = artifacts
            .join("namespaces")
            .join(module.namespace.prefix)
            .join("index.html");
        if !page.exists() {
            failures.push(format!(
                "namespaces/{}/index.html not found",
                module.namespace.prefix
            ));
            continue;
        }
        let html = match std::fs::read_to_string(&page) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!(
                    "namespaces/{}/index.html: read error: {e}",
                    module.namespace.prefix
                ));
                continue;
            }
        };
        if !html.contains("class=\"class-hierarchy\"") || !html.contains("<svg") {
            failures.push(format!(
                "namespaces/{}/index.html missing class hierarchy SVG ({} classes)",
                module.namespace.prefix,
                module.classes.len()
            ));
        }
    }

    if failures.is_empty() {
        report.push(TestResult::pass(
            "website/visual/svg-class-hierarchy",
            format!(
                "All {checked} namespace pages (with >= {min_classes} classes) contain class hierarchy SVG"
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/visual/svg-class-hierarchy",
            format!(
                "{} namespace page(s) missing class hierarchy SVG",
                failures.len()
            ),
            failures,
        ));
    }

    Ok(())
}

/// identities/index.html must contain the identity distribution SVG.
fn check_identity_distribution_svg(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let page = artifacts.join("identities").join("index.html");
    if !page.exists() {
        report.push(TestResult::fail(
            "website/visual/identity-distribution",
            "identities/index.html not found in generated website",
        ));
        return Ok(());
    }

    let html = std::fs::read_to_string(&page)?;
    let has_class = html.contains("class=\"identity-distribution\"");
    let has_figure = html.contains("<figure");
    let has_svg = html.contains("<svg");

    if has_class && has_figure && has_svg {
        report.push(TestResult::pass(
            "website/visual/identity-distribution",
            "identities/index.html contains identity distribution SVG figure",
        ));
    } else {
        report.push(TestResult::fail(
            "website/visual/identity-distribution",
            format!(
                "identities/index.html missing identity distribution SVG \
                 (class=\"identity-distribution\": {has_class}, <figure: {has_figure}, <svg: {has_svg})"
            ),
        ));
    }

    Ok(())
}
