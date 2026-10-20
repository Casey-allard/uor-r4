//! Standards document count validator.
//!
//! Validates that the conformance standards markdown files contain correct
//! ontology counts, preventing drift between the live spec and normative
//! documentation.

use std::path::Path;

use anyhow::{Context, Result};

use crate::report::{ConformanceReport, TestResult};

/// Validates that conformance standards documents contain correct ontology counts.
///
/// Checks `conformance/standards/{docs,jsonld,website}.md` for count claims
/// (e.g., "175 classes") and asserts they match the live ontology.
///
/// # Errors
///
/// Returns an error if a standards file cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let ontology = uor_ontology::Ontology::full();

    let expected_classes = ontology.class_count();
    let expected_properties = ontology.property_count();
    let expected_individuals = ontology.individual_count();

    let standards_dir = workspace.join("conformance/standards");

    validate_standards_file(
        &standards_dir.join("docs.md"),
        "ontology/standards/docs",
        expected_classes,
        expected_properties,
        expected_individuals,
        &mut report,
    )?;

    validate_standards_file(
        &standards_dir.join("jsonld.md"),
        "ontology/standards/jsonld",
        expected_classes,
        expected_properties,
        expected_individuals,
        &mut report,
    )?;

    validate_standards_file(
        &standards_dir.join("website.md"),
        "ontology/standards/website",
        expected_classes,
        expected_properties,
        expected_individuals,
        &mut report,
    )?;

    Ok(report)
}

/// Validates a single standards file for correct count claims.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
fn validate_standards_file(
    path: &Path,
    validator: &str,
    expected_classes: usize,
    expected_properties: usize,
    expected_individuals: usize,
    report: &mut ConformanceReport,
) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let mut errors: Vec<String> = Vec::new();

    // Check class count claims (patterns: "N classes", "N class labels")
    check_count_claim(
        &content,
        &["classes", "class labels"],
        expected_classes,
        "classes",
        &mut errors,
    );

    // Check property count claims (patterns: "N properties", "N property labels")
    check_count_claim(
        &content,
        &["properties", "property labels"],
        expected_properties,
        "properties",
        &mut errors,
    );

    // Check individual count claims (patterns: "N individuals", "N named individuals",
    // "N individual labels")
    check_count_claim(
        &content,
        &["individuals", "named individuals", "individual labels"],
        expected_individuals,
        "individuals",
        &mut errors,
    );

    if errors.is_empty() {
        report.push(TestResult::pass(
            validator,
            format!(
                "All count claims in {} are correct",
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            validator,
            format!(
                "Stale count claims in {}",
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            ),
            errors,
        ));
    }

    Ok(())
}

/// Checks that all occurrences of "N <suffix>" in the content match the expected count.
fn check_count_claim(
    content: &str,
    suffixes: &[&str],
    expected: usize,
    label: &str,
    errors: &mut Vec<String>,
) {
    for suffix in suffixes {
        // Find patterns like "175 classes" or "130 class labels"
        let pattern = format!(" {}", suffix);
        for (i, line) in content.lines().enumerate() {
            if let Some(pos) = line.find(&pattern) {
                // Walk backward from the match to extract the number
                let before = &line[..pos];
                let num_str: String = before
                    .chars()
                    .rev()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                if let Ok(found) = num_str.parse::<usize>() {
                    if found != expected {
                        errors.push(format!(
                            "Line {}: found \"{} {}\" but expected {} {}",
                            i + 1,
                            found,
                            suffix,
                            expected,
                            label
                        ));
                    }
                }
            }
        }
    }
}
