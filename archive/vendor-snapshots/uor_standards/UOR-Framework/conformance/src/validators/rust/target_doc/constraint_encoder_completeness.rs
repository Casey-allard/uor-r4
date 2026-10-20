//! Target-doc cross-ref A.3: every `ConstraintRef` variant has a
//! non-`None` arm in `encode_constraint_to_clauses`.
//!
//! Authority: target §1.5 + §4.7 (closed six-kind constraint set).
//! Fails with the list of variants still routing to `None`.

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/target_doc/constraint_encoder_completeness";

/// Runs the constraint-encoder completeness check.
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

    let variants = extract_constraint_ref_variants(&content);
    if variants.is_empty() {
        report.push(TestResult::fail(
            VALIDATOR,
            "could not parse `ConstraintRef` enum variants from foundation/src/pipeline.rs",
        ));
        return Ok(report);
    }

    let encoder_body = match extract_encoder_body(&content) {
        Some(b) => b,
        None => {
            report.push(TestResult::fail(
                VALIDATOR,
                "could not locate `encode_constraint_to_clauses` in foundation/src/pipeline.rs",
            ));
            return Ok(report);
        }
    };

    // Reject a wildcard-to-None arm outright.
    let has_wildcard_none = encoder_body.contains("_ => None")
        || encoder_body.contains("_ => {\n        None")
        || encoder_body.contains("_ => {\n            None");

    let mut uncovered: Vec<String> = Vec::new();
    for variant in &variants {
        if !encoder_arm_is_some(&encoder_body, variant) {
            uncovered.push(variant.clone());
        }
    }

    if uncovered.is_empty() && !has_wildcard_none {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "closed six-kind constraint set: all {} `ConstraintRef` variants have explicit `Some(...)` arms in `encode_constraint_to_clauses` (no `_ => None` catch-all)",
                variants.len()
            ),
        ));
    } else {
        let mut details = uncovered;
        if has_wildcard_none {
            details.push(
                "encode_constraint_to_clauses contains a `_ => None` catch-all — every variant must be explicit"
                    .to_string(),
            );
        }
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "constraint-encoder completeness: {} issue(s)",
                details.len()
            ),
            details,
        ));
    }

    Ok(report)
}

/// Extract variant names from the `pub enum ConstraintRef { ... }` block.
fn extract_constraint_ref_variants(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Some(idx) = src.find("pub enum ConstraintRef") else {
        return out;
    };
    let Some(brace_off) = src[idx..].find('{') else {
        return out;
    };
    let start = idx + brace_off + 1;
    // Find matching close brace.
    let bytes = src.as_bytes();
    let mut depth: i32 = 1;
    let mut j = start;
    while j < bytes.len() && depth > 0 {
        match bytes[j] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        j += 1;
    }
    let body = &src[start..j.saturating_sub(1)];
    // Each variant: line starts with an uppercase-led identifier followed
    // by '{' or '(' or ',' or whitespace.
    for raw_line in body.lines() {
        let line = raw_line.trim_start();
        if line.starts_with("//") || line.is_empty() {
            continue;
        }
        let Some(first_char) = line.chars().next() else {
            continue;
        };
        if !first_char.is_uppercase() {
            continue;
        }
        let ident: String = line
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !ident.is_empty() {
            out.push(ident);
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Extract the body of `encode_constraint_to_clauses`.
fn extract_encoder_body(src: &str) -> Option<String> {
    let idx = src.find("pub(crate) const fn encode_constraint_to_clauses")?;
    let brace_off = src[idx..].find('{')?;
    let start = idx + brace_off + 1;
    let bytes = src.as_bytes();
    let mut depth: i32 = 1;
    let mut j = start;
    while j < bytes.len() && depth > 0 {
        match bytes[j] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        j += 1;
    }
    Some(src[start..j.saturating_sub(1)].to_string())
}

/// Does the encoder body have an explicit `Some(...)` arm for the given
/// variant? Accept either `ConstraintRef::<Name>` or bare `<Name>` (when
/// imported via `use ConstraintRef as C`) followed somewhere later by
/// `Some(`.
fn encoder_arm_is_some(body: &str, variant: &str) -> bool {
    let anchor_qualified = format!("ConstraintRef::{variant}");
    let anchor_bare = format!("C::{variant}");
    for anchor in [&anchor_qualified, &anchor_bare] {
        let mut search_from = 0usize;
        while let Some(off) = body[search_from..].find(anchor.as_str()) {
            let arm_start = search_from + off;
            // Look ahead ≤300 chars for a `Some(` before the next `,` at depth 0
            // (which would end the arm). Simpler heuristic: slice next 300
            // chars and check.
            let snippet_end = (arm_start + 300).min(body.len());
            let snippet = &body[arm_start..snippet_end];
            // Rule out the arm that is itself inside a fused pattern like
            // `ConstraintRef::A | ConstraintRef::B => ...` where the RHS
            // is `Some(...)` — any `Some(` in the snippet counts.
            if snippet.contains("Some(") {
                return true;
            }
            search_from = arm_start + anchor.len();
        }
    }
    false
}
