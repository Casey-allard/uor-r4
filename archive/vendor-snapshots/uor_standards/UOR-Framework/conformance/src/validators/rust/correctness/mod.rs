//! Layer 2: correctness validators.
//!
//! Each validator invokes one `foundation/tests/behavior_*.rs` test binary
//! via `cargo test --test <name>` and maps the exit status to a
//! conformance check. This is the shell-out pattern from
//! `public_api_functional.rs`, parameterized over behavior-test name.
//!
//! Every behavior test contributes exactly one conformance check. A
//! failure in any behavior test becomes a `[FAIL]` in the conformance
//! output with a precise contract-violation message.

use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

/// Runs a single behavior test as a conformance check.
///
/// # Errors
///
/// Returns an error if the cargo invocation cannot be spawned.
pub fn run_behavior_test(
    workspace: &Path,
    validator_name: &'static str,
    test_name: &'static str,
    pass_message: &'static str,
    features: &[&str],
) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let mut args: Vec<String> = vec![
        "test".into(),
        "-p".into(),
        "uor-foundation".into(),
        "--test".into(),
        test_name.into(),
        "--quiet".into(),
    ];
    for feat in features {
        args.push("--features".into());
        args.push((*feat).into());
    }
    let output = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .args(&args)
        .output();
    match output {
        Ok(o) if o.status.success() => {
            report.push(TestResult::pass(validator_name, pass_message));
        }
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Extract the failure tail (last ~400 chars of stdout, often
            // contains the `assertion failed:` message).
            let tail: String = stdout
                .lines()
                .rev()
                .take(15)
                .collect::<Vec<&str>>()
                .into_iter()
                .rev()
                .collect::<Vec<&str>>()
                .join(" | ");
            report.push(TestResult::fail(
                validator_name,
                format!("behavior test {test_name} failed: {tail}"),
            ));
        }
        Err(e) => {
            report.push(TestResult::fail(
                validator_name,
                format!("failed to spawn cargo test {test_name}: {e}"),
            ));
        }
    }
    Ok(report)
}

/// Runs all 14 behavioral correctness tests as conformance checks.
///
/// # Errors
///
/// Returns an error only if cargo invocation itself fails (not on test
/// failures, which are reported as `fail` check results).
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    // One entry per behavior test: (validator-name, test-name, pass-message, features).
    let behaviors: &[(&str, &str, &str, &[&str])] = &[
        (
            "rust/correctness/calibration",
            "behavior_calibration",
            "Calibration::new rejects every CalibrationError trigger; 4 presets validate",
            &[],
        ),
        (
            "rust/correctness/uor_time",
            "behavior_uor_time",
            "UorTime accessors, min_wall_clock, PartialOrd, LandauerBudget::Ord all correct",
            &[],
        ),
        (
            "rust/correctness/builder_rejection",
            "behavior_builder_rejection",
            "9 builders reject each missing required field with the exact property_iri",
            &[],
        ),
        (
            "rust/correctness/witness_accessors",
            "behavior_witness_accessors",
            "Grounded + Certified accessors return populated fields; content-deterministic",
            &[],
        ),
        (
            "rust/correctness/pipeline_determinism",
            "behavior_pipeline_determinism",
            "run/run_const/run_parallel/run_stream produce content-deterministic witnesses",
            &[],
        ),
        (
            "rust/correctness/grounding_interpreter",
            "behavior_grounding_interpreter",
            "GroundingProgram::run handles all 12 combinator ops",
            &[],
        ),
        (
            "rust/correctness/observability",
            "behavior_observability",
            "ObservabilitySubscription dispatches TraceEvents to registered handlers",
            &["observability"],
        ),
        (
            "rust/correctness/const_ring_eval",
            "behavior_const_ring_eval",
            "const_ring_eval_w{n} correct ring arithmetic at every shipped WittLevel",
            &[],
        ),
        (
            "rust/correctness/ring_ops_identities",
            "behavior_ring_ops_identities",
            "RingOp<L> and UnaryRingOp<L> identities (Neg(BNot(x))=Succ(x), etc.) hold",
            &[],
        ),
        (
            "rust/correctness/embedding_preserves_value",
            "behavior_embedding_preserves_value",
            "Embed<From, To>::apply preserves numeric value (zero-extension) for every pair",
            &[],
        ),
        (
            "rust/correctness/grammar_surface_roundtrip",
            "behavior_grammar_surface_roundtrip",
            "9 grammar builders roundtrip validated Decl with correct conformance:*Shape IRI",
            &[],
        ),
        (
            "rust/correctness/sat_deciders",
            "behavior_sat_deciders",
            "decide_two_sat and decide_horn_sat produce correct verdicts on representative inputs",
            &[],
        ),
        (
            "rust/correctness/resolver_multiplication",
            "behavior_resolver_multiplication",
            "multiplication::certify rejects zero stack, fingerprint discriminates by context",
            &[],
        ),
        (
            "rust/correctness/resolver_tower",
            "behavior_resolver_tower",
            "Distinct resolvers produce distinct fingerprints for the same input",
            &[],
        ),
    ];

    for (name, test, msg, features) in behaviors {
        report.extend(run_behavior_test(workspace, name, test, msg, features)?);
    }

    Ok(report)
}
