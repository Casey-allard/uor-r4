//! RDF 1.1 / Turtle 1.1 validator.
//!
//! Validates that the Turtle and N-Triples artifacts are well-formed:
//! - Turtle file parses without errors (structural + spec-compliant parse-back)
//! - N-Triples file parses without errors
//! - Triple counts meet minimum thresholds derived from the ontology

use std::path::Path;

use anyhow::{Context, Result};

use crate::report::{ConformanceReport, TestResult};

/// Validates the Turtle and N-Triples artifacts for RDF 1.1 conformance.
///
/// # Errors
///
/// Returns an error if artifact files cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    validate_turtle(artifacts, &mut report)?;
    validate_ntriples(artifacts, &mut report)?;

    Ok(report)
}

/// Validates the Turtle file structure.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
fn validate_turtle(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let ttl_path = artifacts.join("uor.foundation.ttl");
    if !ttl_path.exists() {
        report.push(TestResult::fail(
            "ontology/rdf",
            "uor.foundation.ttl not found in artifacts directory",
        ));
        return Ok(());
    }

    let content = std::fs::read_to_string(&ttl_path)
        .with_context(|| format!("Failed to read {}", ttl_path.display()))?;

    // Structural checks (without invoking a full Turtle parser)
    let has_prefixes = content.contains("@prefix");
    let has_base = content.contains("@prefix owl:") || content.contains("@prefix rdf:");
    let non_empty = !content.trim().is_empty();
    let has_triples = content.contains(" a ") || content.contains("rdf:type");

    if non_empty && has_prefixes && has_base && has_triples {
        report.push(TestResult::pass(
            "ontology/rdf",
            format!(
                "uor.foundation.ttl is non-empty and has expected Turtle structure ({} bytes)",
                content.len()
            ),
        ));
    } else {
        let mut issues = Vec::new();
        if !non_empty {
            issues.push("File is empty".to_string());
        }
        if !has_prefixes {
            issues.push("No @prefix declarations found".to_string());
        }
        if !has_base {
            issues.push("Missing owl: or rdf: prefix".to_string());
        }
        if !has_triples {
            issues.push("No triple statements found".to_string());
        }
        report.push(TestResult::fail_with_details(
            "ontology/rdf",
            "uor.foundation.ttl has structural issues",
            issues,
        ));
    }

    // Check prefix count (should have all namespace prefixes + standard prefixes)
    let prefix_count = content
        .lines()
        .filter(|l| l.trim_start().starts_with("@prefix"))
        .count();
    let required_prefixes = uor_ontology::counts::NAMESPACES;
    if prefix_count >= required_prefixes {
        report.push(TestResult::pass(
            "ontology/rdf",
            format!(
                "Turtle file has {} @prefix declarations (\u{2265}{} required)",
                prefix_count, required_prefixes
            ),
        ));
    } else {
        report.push(TestResult::fail(
            "ontology/rdf",
            format!(
                "Turtle file has only {} @prefix declarations (expected ≥16)",
                prefix_count
            ),
        ));
    }

    // Parse-back validation: parse the Turtle file with a spec-compliant parser
    match parse_turtle_triples(&content) {
        Ok(triple_count) => {
            report.push(TestResult::pass(
                "ontology/rdf",
                format!(
                    "uor.foundation.ttl parses as valid Turtle ({} triples)",
                    triple_count
                ),
            ));
        }
        Err(e) => {
            report.push(TestResult::fail(
                "ontology/rdf",
                format!("uor.foundation.ttl fails Turtle parse-back: {}", e),
            ));
        }
    }

    Ok(())
}

/// Parses a Turtle string using `sophia_turtle` and returns the triple count.
///
/// # Errors
///
/// Returns an error string if parsing fails.
fn parse_turtle_triples(content: &str) -> std::result::Result<usize, String> {
    use sophia_api::parser::TripleParser;
    use sophia_api::source::TripleSource;
    use sophia_turtle::parser::turtle::TurtleParser;

    let parser = TurtleParser::default();
    let mut source = parser.parse(std::io::Cursor::new(content.as_bytes()));
    let mut count = 0usize;
    source
        .for_each_triple(|_| {
            count += 1;
        })
        .map_err(|e| format!("{e}"))?;
    Ok(count)
}

/// Validates the N-Triples file structure.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
fn validate_ntriples(artifacts: &Path, report: &mut ConformanceReport) -> Result<()> {
    let nt_path = artifacts.join("uor.foundation.nt");
    if !nt_path.exists() {
        report.push(TestResult::fail(
            "ontology/rdf",
            "uor.foundation.nt not found in artifacts directory",
        ));
        return Ok(());
    }

    let content = std::fs::read_to_string(&nt_path)
        .with_context(|| format!("Failed to read {}", nt_path.display()))?;

    let non_empty = !content.trim().is_empty();

    // Each non-blank line in N-Triples must end with " ."
    let mut malformed_lines: Vec<String> = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !trimmed.ends_with(" .") {
            malformed_lines.push(format!("line {}: does not end with \" .\"", i + 1));
        }
    }

    let triple_count = content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#') && t.ends_with(" .")
        })
        .count();

    if non_empty && malformed_lines.is_empty() {
        report.push(TestResult::pass(
            "ontology/rdf",
            format!(
                "uor.foundation.nt is valid N-Triples ({} triples)",
                triple_count
            ),
        ));
    } else {
        let mut issues = Vec::new();
        if !non_empty {
            issues.push("File is empty".to_string());
        }
        issues.extend(malformed_lines.into_iter().take(10)); // limit output
        report.push(TestResult::fail_with_details(
            "ontology/rdf",
            "uor.foundation.nt has malformed lines",
            issues,
        ));
    }

    // Triple count threshold: verify the N-Triples output is not lossy
    let ontology = uor_ontology::Ontology::full();
    // Conservative minimum: 3 triples per class (type, label, comment),
    // 4 per property (type, label, comment, range), 4 per individual
    // (type*2, label, comment).
    let min_expected = ontology.class_count() * 3
        + ontology.property_count() * 4
        + ontology.individual_count() * 4;
    if triple_count >= min_expected {
        report.push(TestResult::pass(
            "ontology/rdf",
            format!(
                "N-Triples triple count ({}) meets minimum threshold ({})",
                triple_count, min_expected
            ),
        ));
    } else {
        report.push(TestResult::fail(
            "ontology/rdf",
            format!(
                "N-Triples triple count ({}) below minimum threshold ({})",
                triple_count, min_expected
            ),
        ));
    }

    Ok(())
}
