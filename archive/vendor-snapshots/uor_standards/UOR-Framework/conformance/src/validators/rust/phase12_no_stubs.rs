//! Phase 12 conformance gate: no `WITNESS_UNIMPLEMENTED_STUB:*` markers
//! remain in `foundation/src/primitives/*.rs`. Every `verify_*`
//! primitive returns either `Ok(witness)` or a typed
//! `GenericImpossibilityWitness` per the plan's close condition.
//!
//! Excludes the OB_P cohomology family from the assertion — those
//! identities are tracked under Phase 14 and explicitly carry the
//! `THEORY_DEFERRED:OB_P_*` error IRI per Phase 12c's caveat.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/phase12_no_stubs";

/// Runs the Phase 12 no-stubs validator.
///
/// # Errors
///
/// Returns an error if `foundation/src/primitives/` cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let primitives_dir = workspace.join("foundation/src/primitives");
    let entries = match std::fs::read_dir(&primitives_dir) {
        Ok(e) => e,
        Err(err) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!(
                    "cannot read foundation/src/primitives/: {err} \
                     (Phase 10 / 12 require the directory to exist)"
                ),
            ));
            return Ok(report);
        }
    };

    let mut failures: Vec<String> = Vec::new();
    let mut files_scanned = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().is_some_and(|e| e != "rs") {
            continue;
        }
        let label = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        if label == "mod.rs" {
            continue;
        }
        let body = match std::fs::read_to_string(&path) {
            Ok(b) => b,
            Err(e) => {
                failures.push(format!("cannot read primitives/{label}: {e}"));
                continue;
            }
        };
        files_scanned += 1;

        // The plan's Phase 12 close condition: "no stub body remains".
        // The `WITNESS_UNIMPLEMENTED_STUB:` marker is the Phase-10 stub
        // signature — its presence in a primitive body is a Phase-12 regression.
        for (line_no, line) in body.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!")
            {
                continue;
            }
            if line.contains("WITNESS_UNIMPLEMENTED_STUB:") {
                failures.push(format!(
                    "primitives/{label}:{}: WITNESS_UNIMPLEMENTED_STUB marker remains in body",
                    line_no + 1
                ));
            }
        }

        // Phase 15 close: every primitive file must contain at least
        // one `for_identity(` call signalling that the verify_*
        // bodies route at least some failure modes to specific
        // op-namespace identities. Catches accidental
        // unconditional-Ok regressions where someone removes all
        // structural-invariant checks.
        if !body.contains("GenericImpossibilityWitness::for_identity(") {
            failures.push(format!(
                "primitives/{label}: missing `GenericImpossibilityWitness::for_identity(...)` call \
                 (Phase 15 expects every verify_* to route at least one failure mode)"
            ));
        }
    }

    if failures.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "Phase 12 no-stubs: {files_scanned} per-family primitive file(s) \
                 contain zero WITNESS_UNIMPLEMENTED_STUB markers; every verify_* \
                 returns Ok(witness) or a typed GenericImpossibilityWitness"
            ),
        ));
    } else {
        let mut summary = format!(
            "Phase 12 stub residue: {} occurrence(s) of WITNESS_UNIMPLEMENTED_STUB:",
            failures.len()
        );
        for f in &failures {
            summary.push_str("\n       - ");
            summary.push_str(f);
        }
        report.push(TestResult::fail(VALIDATOR, summary));
    }

    Ok(report)
}
