//! Phase K (target §4.3 + §9 criterion 1): W4 closure — mechanical
//! kind-discriminator verification for `Grounding::ground`.
//!
//! The closure property is established at the Rust type level:
//! `Grounding::program` returns `GroundingProgram<Self::Output, Self::Map>`,
//! and `GroundingProgram::from_primitive` requires the primitive's marker
//! tuple to satisfy `MarkersImpliedBy<Map>`. Therefore any well-typed impl
//! of `Grounding` necessarily has a combinator decomposition whose markers
//! imply its declared `Map` kind — the discriminator is verified, not
//! promised.
//!
//! This validator pins the trait shape: both `program` and `ground` are
//! required methods, and `GroundingProgram` has a `run` interpreter for
//! `GroundedCoord` outputs.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/w4_closure";

/// Runs the Phase K W4-closure validation.
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

    let required: &[(&str, &str)] = &[
        // Phase K: Grounding trait now requires `program()`.
        (
            "Grounding::program required method",
            "fn program(&self) -> GroundingProgram<Self::Output, Self::Map>",
        ),
        (
            "Grounding::ground still required",
            "fn ground(&self, external: &[u8]) -> Option<Self::Output>;",
        ),
        // GroundingProgram::run interpreter for GroundedCoord outputs.
        (
            "GroundingProgram::run interpreter",
            "pub fn run(&self, external: &[u8]) -> Option<GroundedCoord>",
        ),
        (
            "GroundingProgram<GroundedCoord, Map> impl block",
            "impl<Map: GroundingMapKind> GroundingProgram<GroundedCoord, Map>",
        ),
        // from_primitive's MarkersImpliedBy<Map> bound — the load-bearing
        // type-system constraint that enforces kind-discriminator verification.
        (
            "from_primitive MarkersImpliedBy<Map> bound",
            "Markers: MarkerTuple + MarkersImpliedBy<Map>",
        ),
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
            "Phase K W4 closure: Grounding trait requires both `program()` and `ground()`; \
             `program` returns a `GroundingProgram<Output, Map>` whose `from_primitive` \
             constructor's `MarkersImpliedBy<Map>` bound mechanically verifies the kind \
             discriminator. `GroundingProgram::run` provides the foundation's interpreter \
             for GroundedCoord outputs — target §4.3 + §9 criterion 1 satisfied.",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!("Phase K W4 closure has {} missing anchors", missing.len()),
            missing,
        ));
    }

    Ok(report)
}
