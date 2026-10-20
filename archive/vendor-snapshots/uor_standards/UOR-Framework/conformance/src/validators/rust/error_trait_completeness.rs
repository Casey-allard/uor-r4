//! v0.2.2 T6.22 validator: every foundation error type implements both
//! `core::fmt::Display` and `core::error::Error`.
//!
//! Greps the foundation source for every `pub enum *Error`, `pub struct
//! *Violation`, `pub struct *Failure`, `pub struct *Witness`, and asserts
//! each has both `impl core::fmt::Display for ` and `impl core::error::Error
//! for ` blocks.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/error_trait_completeness";

fn extract_type_names<'a>(content: &'a str, prefix: &str, suffixes: &[&str]) -> Vec<&'a str> {
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix(prefix) {
            // Extract the identifier up to the first non-ident character.
            let name_end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            let name = &rest[..name_end];
            if suffixes.iter().any(|s| name.ends_with(*s)) {
                out.push(name);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Runs the error-trait-completeness check.
///
/// # Errors
///
/// Returns an error if the foundation enforcement source cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let enforcement_path = workspace.join("foundation/src/enforcement.rs");
    let content = match std::fs::read_to_string(&enforcement_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", enforcement_path.display()),
            ));
            return Ok(report);
        }
    };

    let mut candidates: Vec<String> = Vec::new();
    for name in extract_type_names(&content, "pub enum ", &["Error", "Failure"]) {
        candidates.push(name.to_string());
    }
    for name in extract_type_names(
        &content,
        "pub struct ",
        &["Error", "Violation", "Failure", "Witness"],
    ) {
        candidates.push(name.to_string());
    }
    candidates.sort();
    candidates.dedup();

    let mut missing: Vec<String> = Vec::new();
    for name in &candidates {
        let display_anchor = format!("impl core::fmt::Display for {name}");
        let error_anchor = format!("impl core::error::Error for {name}");
        if !content.contains(&display_anchor) {
            missing.push(format!("{name}: missing `impl core::fmt::Display`"));
        }
        if !content.contains(&error_anchor) {
            missing.push(format!("{name}: missing `impl core::error::Error`"));
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "T6.22 error trait completeness: {} error/violation/failure/witness \
                 types all implement `core::fmt::Display` and `core::error::Error`",
                candidates.len()
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "T6.22 error trait completeness: {} missing trait impls across {} types",
                missing.len(),
                candidates.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
