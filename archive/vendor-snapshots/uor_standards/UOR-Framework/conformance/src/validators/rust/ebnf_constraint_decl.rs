//! v0.2.2 T2.3 (cleanup): EBNF constraint-decl production validator.
//!
//! Reads `public/uor.conformance.ebnf` and asserts the parametric Phase D
//! `constraint-decl` / `conjunction-decl` productions and the 6 legacy-sugar
//! forms are all present. Prevents drift between the codegen emitter and
//! the published grammar artifact.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/ebnf_constraint_decl";

/// Runs the EBNF constraint-decl production check.
///
/// # Errors
///
/// Returns an error if the EBNF artifact cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let ebnf_path = workspace.join("public/uor.conformance.ebnf");
    let content = match std::fs::read_to_string(&ebnf_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", ebnf_path.display()),
            ));
            return Ok(report);
        }
    };

    let required: &[(&str, &str)] = &[
        ("constraint-decl production", "constraint-decl        ::="),
        ("observable-iri production", "observable-iri         ::="),
        ("bound-shape-iri production", "bound-shape-iri        ::="),
        ("arg-list production", "arg-list               ::="),
        ("conjunction-decl production", "conjunction-decl       ::="),
        ("residue-sugar production", "residue-sugar          ::="),
        ("hamming-sugar production", "hamming-sugar          ::="),
        ("depth-sugar production", "depth-sugar            ::="),
        ("carry-sugar production", "carry-sugar            ::="),
        ("site-sugar production", "site-sugar             ::="),
        ("affine-sugar production", "affine-sugar           ::="),
    ];

    let mut missing: Vec<String> = Vec::new();
    for (label, anchor) in required {
        if !content.contains(*anchor) {
            missing.push((*label).to_string());
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "Phase D EBNF: constraint-decl + observable-iri + bound-shape-iri \
             + conjunction-decl + 6 legacy-sugar productions all present in \
             public/uor.conformance.ebnf",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase D EBNF constraint-decl has {} missing productions",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}
