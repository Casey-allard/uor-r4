//! Phase 13c R16 conformance category: TaxonomyCoverage.
//!
//! Asserts that the Phase 0 classification report at
//! `docs/orphan-closure/classification_report.md` matches the live
//! `uor_codegen::classification::classify_all(Ontology::full())`
//! output cell-for-cell. Also asserts the per-class counts in the
//! report's totals table match `spec/src/counts.rs::CLASSIFICATION_*`.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/taxonomy_coverage";

/// Runs the Phase 13c TaxonomyCoverage validator.
///
/// # Errors
///
/// Returns an error if the classification report file cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let report_path = workspace.join("docs/orphan-closure/classification_report.md");
    let report_body = match std::fs::read_to_string(&report_path) {
        Ok(b) => b,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("cannot read {}: {e}", report_path.display()),
            ));
            return Ok(report);
        }
    };

    let ontology = uor_ontology::Ontology::full();
    let entries = uor_codegen::classification::classify_all(ontology);
    let counts = uor_codegen::classification::count(&entries);

    let mut failures: Vec<String> = Vec::new();

    // 1. Total count rows in the report's `## Totals` table must match
    // the live counts.
    let total_row_pairs: &[(&str, usize)] = &[
        ("| Skip |", counts.skip),
        ("| AlreadyImplemented |", counts.already_implemented),
        ("| Path1HandleResolver |", counts.path1),
        ("| Path2TheoremWitness |", counts.path2),
        ("| Path3PrimitiveBacked |", counts.path3),
        ("| Path4TheoryDeferred |", counts.path4),
    ];
    for (prefix, expected) in total_row_pairs {
        let needle = format!("{prefix} {expected} |");
        if !report_body.contains(&needle) {
            failures.push(format!(
                "report Totals table missing row `{prefix} {expected} |` — \
                 live classifier produced `{expected}` for that bucket"
            ));
        }
    }

    // 2. Cross-check counts.rs constants against live counts.
    if uor_ontology::counts::CLASSIFICATION_SKIP != counts.skip {
        failures.push(format!(
            "spec::counts::CLASSIFICATION_SKIP = {} != live {} (drift)",
            uor_ontology::counts::CLASSIFICATION_SKIP,
            counts.skip
        ));
    }
    if uor_ontology::counts::CLASSIFICATION_ALREADY_IMPLEMENTED != counts.already_implemented {
        failures.push(format!(
            "spec::counts::CLASSIFICATION_ALREADY_IMPLEMENTED = {} != live {} (drift)",
            uor_ontology::counts::CLASSIFICATION_ALREADY_IMPLEMENTED,
            counts.already_implemented,
        ));
    }
    if uor_ontology::counts::CLASSIFICATION_PATH1 != counts.path1 {
        failures.push(format!(
            "spec::counts::CLASSIFICATION_PATH1 = {} != live {} (drift)",
            uor_ontology::counts::CLASSIFICATION_PATH1,
            counts.path1
        ));
    }
    if uor_ontology::counts::CLASSIFICATION_PATH2 != counts.path2 {
        failures.push(format!(
            "spec::counts::CLASSIFICATION_PATH2 = {} != live {} (drift)",
            uor_ontology::counts::CLASSIFICATION_PATH2,
            counts.path2
        ));
    }
    if uor_ontology::counts::CLASSIFICATION_PATH3 != counts.path3 {
        failures.push(format!(
            "spec::counts::CLASSIFICATION_PATH3 = {} != live {} (drift)",
            uor_ontology::counts::CLASSIFICATION_PATH3,
            counts.path3
        ));
    }
    if uor_ontology::counts::CLASSIFICATION_PATH4 != counts.path4 {
        failures.push(format!(
            "spec::counts::CLASSIFICATION_PATH4 = {} != live {} (drift)",
            uor_ontology::counts::CLASSIFICATION_PATH4,
            counts.path4
        ));
    }

    // 3. Total row.
    let total_needle = format!("| **Total** | **{}** |", counts.total());
    if !report_body.contains(&total_needle) {
        failures.push(format!(
            "report Totals total row missing `{total_needle}` (live total = {})",
            counts.total()
        ));
    }

    // 4. Per-class table sanity — every classified class must appear in
    // the per-class table at least once. Use the class local name
    // wrapped in backticks (the report's column format).
    for entry in &entries {
        let needle = format!(" `{}` |", entry.class_local);
        if !report_body.contains(&needle) {
            failures.push(format!(
                "Per-class report row missing for `{}` (`{}`)",
                entry.class_local, entry.class_iri,
            ));
            // Cap reporting at first 5 missing rows.
            if failures.len() > 16 {
                break;
            }
        }
    }

    if failures.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "TaxonomyCoverage: classification report agrees with live classifier — \
                 {} entries; Skip={} AlreadyImplemented={} Path1={} Path2={} Path3={} Path4={}",
                counts.total(),
                counts.skip,
                counts.already_implemented,
                counts.path1,
                counts.path2,
                counts.path3,
                counts.path4,
            ),
        ));
    } else {
        let mut summary = format!("TaxonomyCoverage drift: {} issue(s):", failures.len());
        for f in failures.iter().take(20) {
            summary.push_str("\n       - ");
            summary.push_str(f);
        }
        if failures.len() > 20 {
            summary.push_str(&format!("\n       - ... ({} more)", failures.len() - 20));
        }
        report.push(TestResult::fail(VALIDATOR, summary));
    }

    Ok(report)
}
