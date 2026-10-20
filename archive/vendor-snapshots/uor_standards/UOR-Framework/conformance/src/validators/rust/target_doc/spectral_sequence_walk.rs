//! Target-doc cross-ref A.5: `run_incremental_completeness` walks a
//! `SpectralSequencePage` sequence.
//!
//! Authority: ontology's `IncrementalCompletenessResolver` contract at
//! `spec/src/namespaces/resolver.rs`. Asserts:
//!
//! - `pub struct SpectralSequencePage` is declared in
//!   `foundation/src/pipeline.rs`.
//! - `run_incremental_completeness`'s body references the
//!   `SpectralSequencePage` identifier.

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/target_doc/spectral_sequence_walk";

/// Runs the spectral-sequence-walk structural check.
///
/// # Errors
///
/// Returns an error if `foundation/src/pipeline.rs` cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let path = workspace.join("foundation/src/pipeline.rs");
    let content = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", path.display()),
            ));
            return Ok(report);
        }
    };

    let mut violations: Vec<String> = Vec::new();

    // 1) `pub struct SpectralSequencePage` must exist.
    if !content.contains("pub struct SpectralSequencePage") {
        violations.push(
            "`pub struct SpectralSequencePage` not declared in foundation/src/pipeline.rs"
                .to_string(),
        );
    }

    // 2) `run_incremental_completeness` body references `SpectralSequencePage`.
    if let Some(body) = extract_fn_body(&content, "run_incremental_completeness") {
        if !body.contains("SpectralSequencePage") {
            violations.push(
                "`run_incremental_completeness` body does not reference `SpectralSequencePage` — ontology requires reading the page sequence"
                    .to_string(),
            );
        }
    } else {
        violations
            .push("could not locate `run_incremental_completeness` function body".to_string());
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "ontology contract: `run_incremental_completeness` walks `SpectralSequencePage` sequence; sealed type declared",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "spectral-sequence walk not implemented: {} gap(s)",
                violations.len()
            ),
            violations,
        ));
    }

    Ok(report)
}

fn extract_fn_body(src: &str, fn_name: &str) -> Option<String> {
    let anchor_a = format!("pub fn {fn_name}<");
    let anchor_b = format!("pub fn {fn_name}(");
    let off = src.find(&anchor_a).or_else(|| src.find(&anchor_b))?;
    let brace_rel = src[off..].find('{')?;
    let body_start = off + brace_rel + 1;
    let bytes = src.as_bytes();
    let mut depth: i32 = 1;
    let mut j = body_start;
    while j < bytes.len() && depth > 0 {
        match bytes[j] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        j += 1;
    }
    Some(src[body_start..j.saturating_sub(1)].to_string())
}
