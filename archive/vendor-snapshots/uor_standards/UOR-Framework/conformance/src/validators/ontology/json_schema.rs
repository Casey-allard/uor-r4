//! JSON Schema artifact validator.
//!
//! Validates that the generated `uor.foundation.schema.json` file is
//! well-formed and contains the expected schema structure.

use std::path::Path;

use anyhow::{Context, Result};

use crate::report::{ConformanceReport, TestResult};

/// Validates the JSON Schema artifact for structural correctness.
///
/// Checks that `uor.foundation.schema.json` exists, parses as valid JSON,
/// contains the expected `$schema`, `$defs`, class count, and enum class
/// entries.
///
/// # Errors
///
/// Returns an error if the artifact file cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let validator = "ontology/json_schema";

    let schema_path = artifacts.join("uor.foundation.schema.json");
    if !schema_path.exists() {
        report.push(TestResult::fail(
            validator,
            "uor.foundation.schema.json not found in artifacts directory",
        ));
        return Ok(report);
    }

    let content = std::fs::read_to_string(&schema_path)
        .with_context(|| format!("Failed to read {}", schema_path.display()))?;

    let mut issues: Vec<String> = Vec::new();

    // Parse JSON
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            issues.push(format!("Invalid JSON: {e}"));
            report.push(TestResult::fail_with_details(
                validator,
                "uor.foundation.schema.json has structural issues",
                issues,
            ));
            return Ok(report);
        }
    };

    // $schema key
    if json.get("$schema").and_then(|v| v.as_str())
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        issues.push("Missing or incorrect $schema value".to_string());
    }

    // $defs
    let ontology = uor_ontology::Ontology::full();
    let expected_count = ontology.class_count();

    if let Some(defs) = json.get("$defs").and_then(|v| v.as_object()) {
        if defs.len() != expected_count {
            issues.push(format!(
                "$defs has {} entries, expected {}",
                defs.len(),
                expected_count
            ));
        }

        // Enum classes must have "enum" key (keys are qualified: "prefix/Name")
        for name in uor_ontology::Ontology::enum_class_names() {
            let suffix = format!("/{}", name);
            let found = defs.iter().find(|(k, _)| k.ends_with(&suffix));
            if let Some((_, entry)) = found {
                if entry.get("enum").is_none() {
                    issues.push(format!("Enum class '{}' missing 'enum' keyword", name));
                }
            } else {
                issues.push(format!("Missing $defs entry for enum class '{}'", name));
            }
        }
    } else {
        issues.push("Missing $defs object".to_string());
    }

    // Version in description
    if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
        if !desc.contains(ontology.version) {
            issues.push(format!(
                "Version '{}' not found in description",
                ontology.version
            ));
        }
    } else {
        issues.push("Missing description field".to_string());
    }

    if issues.is_empty() {
        report.push(TestResult::pass(
            validator,
            format!(
                "uor.foundation.schema.json is well-formed \
                 ({} bytes, {} $defs entries)",
                content.len(),
                expected_count
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            validator,
            "uor.foundation.schema.json has structural issues",
            issues,
        ));
    }

    // $ref resolution: every $ref must point to an existing $defs key
    if let Some(defs) = json.get("$defs").and_then(|v| v.as_object()) {
        let mut broken_refs: Vec<String> = Vec::new();
        collect_and_check_refs(&json, defs, &mut broken_refs);
        if broken_refs.is_empty() {
            report.push(TestResult::pass(
                validator,
                "All $ref pointers resolve to existing $defs entries",
            ));
        } else {
            report.push(TestResult::fail_with_details(
                validator,
                "Broken $ref pointers found in JSON Schema",
                broken_refs,
            ));
        }
    }

    Ok(report)
}

/// Recursively collects all `$ref` values and checks they resolve against `$defs`.
fn collect_and_check_refs(
    value: &serde_json::Value,
    defs: &serde_json::Map<String, serde_json::Value>,
    broken: &mut Vec<String>,
) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(r)) = map.get("$ref") {
                if let Some(escaped_key) = r.strip_prefix("#/$defs/") {
                    // Unescape JSON Pointer per RFC 6901: ~1 -> /, ~0 -> ~
                    let key = escaped_key.replace("~1", "/").replace("~0", "~");
                    if !defs.contains_key(&key) {
                        broken.push(format!("$ref '{}' -> key '{}' not found", r, key));
                    }
                }
            }
            for v in map.values() {
                collect_and_check_refs(v, defs, broken);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_and_check_refs(v, defs, broken);
            }
        }
        _ => {}
    }
}
