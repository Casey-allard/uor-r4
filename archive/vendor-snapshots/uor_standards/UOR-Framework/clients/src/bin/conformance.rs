//! `uor-conformance` — Validates all UOR Framework artifacts against professional standards.
//!
//! Runs the complete conformance suite across:
//! - Rust source (API docs, style conventions)
//! - Ontology artifacts (JSON-LD 1.1, OWL 2 DL, RDF 1.1, Turtle 1.1, SHACL, inventory)
//! - Documentation artifacts (completeness, accuracy, Diataxis structure)
//! - Website artifacts (HTML5, WCAG 2.1 AA, CSS, coverage)
//!
//! **Usage:**
//! ```
//! uor-conformance [--artifacts <path>] [--workspace <path>]
//! ```
//!
//! Exits non-zero if any conformance check fails.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::Parser;
use uor_conformance::{run_all, WorkspacePaths};

/// Run the UOR Framework conformance suite.
#[derive(Parser)]
#[command(
    name = "uor-conformance",
    about = "Validate UOR Framework artifacts against professional standards"
)]
struct Args {
    /// Path to the built artifacts directory (default: public/).
    #[arg(long, default_value = "public")]
    artifacts: PathBuf,

    /// Path to the workspace root (default: current directory).
    #[arg(long, default_value = ".")]
    workspace: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let paths = WorkspacePaths {
        workspace: args.workspace,
        artifacts: args.artifacts,
    };

    let report = run_all(&paths)?;

    // Verify total check count has not drifted
    let expected_total_checks = uor_ontology::counts::CONFORMANCE_CHECKS;
    let actual_checks = report.results.len();
    if actual_checks != expected_total_checks {
        eprintln!(
            "CHECK COUNT DRIFT: expected {} checks but got {}. \
             Update CONFORMANCE_CHECKS in spec/src/counts.rs.",
            expected_total_checks, actual_checks
        );
        process::exit(2);
    }

    // Print results
    println!("UOR Framework Conformance Report");
    println!("================================");
    println!();

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut warned = 0usize;

    for result in &report.results {
        let status = match result.severity {
            uor_conformance::Severity::Pass => {
                passed += 1;
                "PASS"
            }
            uor_conformance::Severity::Warning => {
                warned += 1;
                "WARN"
            }
            uor_conformance::Severity::Failure => {
                failed += 1;
                "FAIL"
            }
        };
        println!("[{}] {} — {}", status, result.validator, result.message);
        for detail in &result.details {
            println!("       {}", detail);
        }
    }

    println!();
    println!(
        "Summary: {} passed, {} warnings, {} failed",
        passed, warned, failed
    );

    // Meta-audit results (Amendment 45: self-auditing validators)
    if !report.meta_results.is_empty() {
        println!();
        println!("Meta-Audit Results (ontology-derived)");
        println!("=====================================");
        println!();

        let mut meta_passed = 0usize;
        let mut meta_failed = 0usize;

        for result in &report.meta_results {
            let status = match result.severity {
                uor_conformance::Severity::Pass => {
                    meta_passed += 1;
                    "PASS"
                }
                uor_conformance::Severity::Warning => "WARN",
                uor_conformance::Severity::Failure => {
                    meta_failed += 1;
                    "FAIL"
                }
            };
            println!(
                "[{}] {} \u{2014} {}",
                status, result.validator, result.message
            );
            for detail in &result.details {
                println!("       {}", detail);
            }
        }

        println!();
        println!(
            "Meta-audit: {} passed, {} gaps detected (informational)",
            meta_passed, meta_failed
        );
    }

    if failed > 0 {
        eprintln!("Conformance FAILED: {} check(s) did not pass.", failed);
        process::exit(1);
    }

    println!("Conformance PASSED.");
    Ok(())
}
