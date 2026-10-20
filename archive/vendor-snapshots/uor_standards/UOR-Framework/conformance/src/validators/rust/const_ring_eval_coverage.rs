//! Phase L.2 (target §4.5 + §9 criterion 5): const_ring_eval coverage
//! validator. Asserts that every shipped WittLevel has a corresponding
//! `const_ring_eval_w{n}` helper emitted in `foundation/src/enforcement.rs`.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/const_ring_eval_coverage";

/// All 32 shipped WittLevel bit-widths (native W8..W128 + Limbs-backed
/// W160..W32768). The set is ontology-driven; regenerating the list from
/// `schema:WittLevel` individuals is future work — today we pin the set
/// explicitly to catch per-level emission drift.
const SHIPPED_LEVELS: &[u32] = &[
    // Native-backed (16).
    8, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96, 104, 112, 120, 128,
    // Limbs-backed (16).
    160, 192, 224, 256, 384, 448, 512, 520, 528, 1024, 2048, 4096, 8192, 12288, 16384, 32768,
];

/// Runs the Phase L.2 const-ring-eval coverage check.
///
/// # Errors
///
/// Returns an error if the foundation source cannot be read.
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

    let mut missing: Vec<String> = Vec::new();
    for bits in SHIPPED_LEVELS {
        let anchor = format!("pub const fn const_ring_eval_w{bits}(");
        if !content.contains(&anchor) {
            missing.push(format!(
                "const_ring_eval_w{bits} helper not emitted (target §4.5 L.2 coverage)"
            ));
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "Phase L.2 const-ring-eval coverage: all {} shipped WittLevel individuals \
                 have `const_ring_eval_w{{n}}` helpers (target §4.5 + §9 criterion 5)",
                SHIPPED_LEVELS.len()
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase L.2 const-ring-eval coverage has {} missing helpers",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
