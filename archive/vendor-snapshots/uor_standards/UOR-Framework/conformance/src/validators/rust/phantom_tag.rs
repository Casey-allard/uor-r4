//! v0.2.2 Phase B (Q3): `rust/phantom_tag` validator.
//!
//! Asserts the foundation crate exposes `Grounded<T, Tag = T>` (with the
//! default type parameter) and a `pub fn tag<NewTag>(self) -> Grounded<T, NewTag>`
//! coercion. The phantom Tag is the v0.2.2 mechanism for downstream type-level
//! distinction without any new sealing — see the Q3 commitment in the plan.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/phantom_tag";

/// Required exact-string anchors in `foundation/src/enforcement.rs` for Phase B.
const REQUIRED_ANCHORS: &[(&str, &str)] = &[
    (
        // ADR-018/060: `FP_MAX` (default 32) is threaded between `INLINE_BYTES`
        // and `Tag`; the struct signature is rustfmt-wrapped, so the anchor
        // pins the trailing const-default + `Tag = T` default pair — unique to
        // `Grounded` (no other foundation struct carries a `Tag = T` default).
        "    const FP_MAX: usize = 32,\n    Tag = T,\n> {",
        "Grounded must declare `FP_MAX` (default 32) + `Tag = T` default parameters",
    ),
    (
        "_tag: PhantomData<Tag>,",
        "Grounded must hold a `_tag: PhantomData<Tag>` field",
    ),
    (
        "impl<'a, T: GroundedShape, const INLINE_BYTES: usize, const FP_MAX: usize, Tag>",
        "Grounded impl block must take T, INLINE_BYTES, FP_MAX, and Tag generic parameters",
    ),
    (
        "pub fn tag<NewTag>(self) -> Grounded<'a, T, INLINE_BYTES, FP_MAX, NewTag>",
        "Grounded::tag::<NewTag>() coercion must be public",
    ),
];

/// Forbidden substrings (regression guards).
const FORBIDDEN_ANCHORS: &[(&str, &str)] = &[
    // No `unsafe impl` for the phantom tag mechanism — the tag is purely
    // decoration and shouldn't unlock unsafe paths.
    (
        "unsafe impl<T, Tag> Grounded<T, Tag>",
        "Grounded impl must not be marked unsafe",
    ),
];

/// Runs the Phase B `phantom_tag` validator.
///
/// # Errors
///
/// Returns an error if the foundation source file cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let enforcement_path = workspace
        .join("foundation")
        .join("src")
        .join("enforcement.rs");
    let content = match std::fs::read_to_string(&enforcement_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("cannot read foundation/src/enforcement.rs: {e}"),
            ));
            return Ok(report);
        }
    };

    let mut issues: Vec<String> = Vec::new();

    for (needle, reason) in REQUIRED_ANCHORS {
        if !content.contains(needle) {
            issues.push(format!("missing: {reason} (`{needle}`)"));
        }
    }

    for (needle, reason) in FORBIDDEN_ANCHORS {
        if content.contains(needle) {
            issues.push(format!("forbidden: {reason} (`{needle}`)"));
        }
    }

    if issues.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "Phase B phantom Tag complete: Grounded<T, Tag = T> + tag::<NewTag>() coercion present; no unsafe phantom-tag impls",
        ));
    } else {
        let mut summary = format!(
            "Phase B phantom Tag surface incomplete: {} issue(s):",
            issues.len()
        );
        for i in &issues {
            summary.push_str("\n       - ");
            summary.push_str(i);
        }
        report.push(TestResult::fail(VALIDATOR, summary));
    }

    Ok(report)
}
