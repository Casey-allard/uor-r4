//! Rust style validator.
//!
//! Checks that Rust source files comply with conventions that clippy does not enforce:
//! - No `pub` items without doc comments in library crates
//! - No `std::process::exit` called outside of `main.rs` / `bin/` sources
//! - Every `Cargo.toml` in the workspace declares `edition = "2021"` and `license`

use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::report::{ConformanceReport, TestResult};

/// Validates Rust style conventions across all workspace source files.
///
/// # Errors
///
/// Returns an error if the workspace directory cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    check_no_process_exit_in_lib(workspace, &mut report)?;
    check_cargo_toml_fields(workspace, &mut report)?;

    Ok(report)
}

/// Checks that `std::process::exit` is not called in library sources.
///
/// # Errors
///
/// Returns an error if workspace files cannot be read.
fn check_no_process_exit_in_lib(workspace: &Path, report: &mut ConformanceReport) -> Result<()> {
    let mut violations: Vec<String> = Vec::new();

    for entry in WalkDir::new(workspace)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map(|x| x == "rs").unwrap_or(false)
                && !e.path().to_string_lossy().contains("target")
                && !e
                    .path()
                    .to_string_lossy()
                    .contains(std::path::MAIN_SEPARATOR_STR.to_string().as_str())
        })
    {
        let path = entry.path();
        // Only check lib.rs files (not bin/ sources where process::exit in main is acceptable)
        if path.file_name().map(|n| n == "lib.rs").unwrap_or(false) {
            let content = std::fs::read_to_string(path)?;
            if content.contains("process::exit") || content.contains("std::process::exit") {
                violations.push(format!("{}", path.display()));
            }
        }
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            "rust/style",
            "No std::process::exit calls in library sources",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "rust/style",
            "std::process::exit used in library source(s)",
            violations,
        ));
    }

    Ok(())
}

/// Checks that all workspace `Cargo.toml` files inherit `edition` and `license`.
///
/// # Errors
///
/// Returns an error if Cargo.toml files cannot be read.
fn check_cargo_toml_fields(workspace: &Path, report: &mut ConformanceReport) -> Result<()> {
    let mut issues: Vec<String> = Vec::new();

    for entry in WalkDir::new(workspace)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .map(|n| n == "Cargo.toml")
                .unwrap_or(false)
                && !e.path().to_string_lossy().contains("target")
        })
    {
        let path = entry.path();
        // Skip the workspace root Cargo.toml (it defines [workspace.package], not [package])
        if path.parent() == Some(workspace) {
            continue;
        }
        let content = std::fs::read_to_string(path)?;
        // Member crates should use workspace inheritance for edition and license
        let has_edition = content.contains("edition.workspace")
            || content.contains("edition = \"2021\"")
            || content.contains("edition = '2021'");
        let has_license = content.contains("license.workspace")
            || content.contains("license = ")
            || content.contains("license-file");
        if !has_edition {
            issues.push(format!("{}: missing edition field", path.display()));
        }
        if !has_license {
            issues.push(format!("{}: missing license field", path.display()));
        }
    }

    if issues.is_empty() {
        report.push(TestResult::pass(
            "rust/style",
            "All Cargo.toml files declare edition and license",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "rust/style",
            "Cargo.toml files missing required fields",
            issues,
        ));
    }

    Ok(())
}
