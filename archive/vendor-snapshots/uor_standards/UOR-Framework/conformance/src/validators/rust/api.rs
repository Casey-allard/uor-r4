//! Rust public API surface validator.
//!
//! Validates that all public types and functions in library crates are
//! documented and that error types implement the standard traits.

use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::report::{ConformanceReport, TestResult};

/// Validates the public API surface of all library crates.
///
/// # Errors
///
/// Returns an error if workspace sources cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    check_pub_items_documented(workspace, &mut report)?;
    check_error_types(workspace, &mut report)?;

    Ok(report)
}

/// Checks that `pub` items in `lib.rs` and public modules have doc comments.
///
/// This is a heuristic check: every `pub fn`, `pub struct`, `pub enum`, `pub trait`
/// declaration must be preceded by a `///` doc comment within 5 lines.
///
/// # Errors
///
/// Returns an error if source files cannot be read.
fn check_pub_items_documented(workspace: &Path, report: &mut ConformanceReport) -> Result<()> {
    let mut undocumented: Vec<String> = Vec::new();

    for entry in WalkDir::new(workspace)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map(|x| x == "rs").unwrap_or(false)
                && !e.path().to_string_lossy().contains("target")
        })
    {
        let path = entry.path();
        // Only check library sources (not test code)
        let path_str = path.to_string_lossy();
        if path_str.contains("/tests/") || path_str.ends_with("_test.rs") {
            continue;
        }

        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Detect public item declarations
            if (trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub struct ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("pub trait ")
                || trimmed.starts_with("pub type "))
                && !trimmed.contains("use ")
            {
                // Check that a doc comment appears within the 5 preceding
                // lines. Some declarations have multiple `#[derive]` /
                // `#[non_exhaustive]` / `#[allow(...)]` attribute lines
                // between the doc and the `pub` keyword; 5 lines covers
                // all in-tree cases.
                let start = i.saturating_sub(5);
                let has_doc = lines[start..i]
                    .iter()
                    .any(|l| l.trim().starts_with("///") || l.trim().starts_with("#[doc"));
                if !has_doc {
                    undocumented.push(format!("{}:{}: {}", path.display(), i + 1, trimmed));
                }
            }
        }
    }

    if undocumented.is_empty() {
        report.push(TestResult::pass(
            "rust/api",
            "All public items have documentation comments",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "rust/api",
            "Public items missing documentation comments",
            undocumented,
        ));
    }

    Ok(())
}

/// Checks that types ending in `Error` implement `std::error::Error`.
///
/// This is a heuristic: looks for `struct *Error` or `enum *Error` declarations
/// and verifies there is an `impl` block using `thiserror` or a manual impl.
///
/// **`#![no_std]` exception**: files in crates that begin with `#![no_std]`
/// (or have any module file that declares `#![no_std]`) cannot implement
/// `std::error::Error`. `core::error::Error` is stable from Rust 1.81; with
/// the workspace's MSRV at 1.70 (per `CLAUDE.md` §Toolchain), neither is
/// available. Such files are exempt from this check; the foundation defines
/// error types as plain types and downstream consumers convert them via
/// `From` impls in the application layer where `std::error::Error` is
/// available.
///
/// # Errors
///
/// Returns an error if source files cannot be read.
fn check_error_types(workspace: &Path, report: &mut ConformanceReport) -> Result<()> {
    let mut issues: Vec<String> = Vec::new();

    for entry in WalkDir::new(workspace)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map(|x| x == "rs").unwrap_or(false)
                && !e.path().to_string_lossy().contains("target")
        })
    {
        let path = entry.path();
        let content = std::fs::read_to_string(path)?;

        // Check for error types using thiserror (acceptable pattern)
        if content.contains("thiserror::Error") || content.contains("use thiserror") {
            continue;
        }

        // `#![no_std]` crates cannot implement std::error::Error and (with
        // MSRV 1.70) cannot implement core::error::Error either. Skip files
        // belonging to such crates: walk up to find the crate root's lib.rs
        // and check for the no_std attribute.
        if file_in_no_std_crate(path) {
            continue;
        }

        // Look for manually declared error types without thiserror
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if (trimmed.starts_with("pub struct") || trimmed.starts_with("pub enum"))
                && trimmed.contains("Error")
            {
                // Verify the file contains impl std::error::Error
                if !content.contains("impl std::error::Error")
                    && !content.contains("impl Error for")
                    && !content.contains("thiserror")
                {
                    issues.push(format!(
                        "{}:{}: error type without std::error::Error impl",
                        path.display(),
                        i + 1
                    ));
                }
            }
        }
    }

    if issues.is_empty() {
        report.push(TestResult::pass(
            "rust/api",
            "All error types implement std::error::Error",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "rust/api",
            "Error types missing std::error::Error implementation",
            issues,
        ));
    }

    Ok(())
}

/// Returns true if the given file belongs to a crate whose `lib.rs` declares
/// `#![no_std]`. Walks up the directory tree from the file looking for a `src`
/// directory whose parent is a crate root; checks `src/lib.rs` for the
/// `#![no_std]` attribute. Returns false if no enclosing crate is found or
/// the crate root has no `lib.rs` (e.g., binary-only crates).
fn file_in_no_std_crate(file: &Path) -> bool {
    // Walk parents until we find one whose name is "src" (suggesting that
    // its parent is the crate root).
    let mut cursor = file.parent();
    while let Some(dir) = cursor {
        if dir.file_name().map(|n| n == "src").unwrap_or(false) {
            let lib_rs = dir.join("lib.rs");
            if let Ok(content) = std::fs::read_to_string(&lib_rs) {
                return content.contains("#![no_std]");
            }
            return false;
        }
        cursor = dir.parent();
    }
    false
}
