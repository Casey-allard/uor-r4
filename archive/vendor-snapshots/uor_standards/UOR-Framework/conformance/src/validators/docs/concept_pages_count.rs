//! v0.2.2 T1.5 validator: `website/content/concepts/*.md` count matches
//! `uor_ontology::counts::CONCEPT_PAGES`.
//!
//! Prevents drift between the website's authoritative concept-source
//! directory and the centralised count constant. The constant drifted
//! silently through Phases A–J because no validator enforced it;
//! `website/pages/concepts` only asserted `>=`, not `==`. This validator
//! asserts exact equality, excluding `prism.md` (merged into the pipeline
//! page by the website generator).

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "docs/concept_pages_count";

/// Walks `website/content/concepts/*.md` (excluding `prism.md`) and asserts
/// the count equals `uor_ontology::counts::CONCEPT_PAGES`.
///
/// # Errors
///
/// Returns an error if the concepts directory cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let concepts_dir = workspace.join("website/content/concepts");
    let entries = match std::fs::read_dir(&concepts_dir) {
        Ok(it) => it,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", concepts_dir.display()),
            ));
            return Ok(report);
        }
    };

    let mut actual_count = 0usize;
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                report.push(TestResult::fail(
                    VALIDATOR,
                    format!("failed to read dir entry: {e}"),
                ));
                return Ok(report);
            }
        };
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        // `prism.md` is merged into the pipeline page — exclude it.
        if path.file_stem().and_then(|s| s.to_str()) == Some("prism") {
            continue;
        }
        actual_count += 1;
    }

    let expected = uor_ontology::counts::CONCEPT_PAGES;
    if actual_count == expected {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "website/content/concepts/*.md count (excluding prism.md) matches \
                 CONCEPT_PAGES = {expected}"
            ),
        ));
    } else {
        report.push(TestResult::fail(
            VALIDATOR,
            format!(
                "website/content/concepts/ has {actual_count} .md files (excluding prism.md) \
                 but CONCEPT_PAGES = {expected}; update spec/src/counts.rs to match"
            ),
        ));
    }

    Ok(report)
}
