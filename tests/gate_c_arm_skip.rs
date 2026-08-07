//! Issue #471: the Gate C arm-skip knob does not move a number it still
//! reports.
//!
//! # What this test is for
//!
//! `R4_GATE_C_SKIP_ARMS=right_context` tells the Gate C evaluation not to run
//! the whole-corpus right-context code pass, and therefore not to build the
//! #446 M1 two-sided table or the #446 M2 latent tables. That is a large
//! saving (measured at 60% of a sampled run's wall clock on the 500k fixture,
//! and a growing share as the corpus grows) bought by dropping five rows.
//!
//! The bought-and-paid-for claim is narrow and exact: **every row the run
//! still prints must be bit-identical to what a full run prints.** A
//! performance knob that quietly perturbs `rule12_precedence` would be worse
//! than the 85-minute run it replaces, because the fast number would look
//! usable. So this test runs the shipped `r4 transformerless score` binary
//! twice over the same fixture with the same sample, and compares the two
//! `score_report.json` documents key by key.
//!
//! # Why it spawns the binary instead of calling the library
//!
//! The knob is read from the process environment. Setting a process
//! environment variable from a test thread races every other thread in the
//! harness, and under edition 2024 it is `unsafe` for exactly that reason.
//! Spawning the real binary with the variable set gives each arm its own
//! process, tests the surface a human actually uses, and needs no `unsafe`.
//!
//! # Why the control arm asserts before it compares
//!
//! An equivalence test between two runs that both measured nothing passes.
//! This repository has found five instruments that could not fail; the rule
//! that came out of that is that an all-zero arm set is a harness bug until
//! proven otherwise. So the control arm asserts, BEFORE any comparison, that
//! the right-context arms are present, scored a nonzero population, and moved
//! top-1 away from the Rule 1+2 baseline they are supposed to differ from. If
//! the fixture ever stops exercising those arms this test fails loudly
//! instead of turning green and meaningless.
//!
//! # Running it
//!
//! Ignored by default: two full 500k score runs cost minutes, which is more
//! than the merge queue should pay per PR. It is also the measurement
//! instrument for #471 — it prints both wall clocks and the saving.
//!
//! ```text
//! cargo test --release --test gate_c_arm_skip -- --ignored --nocapture
//! ```
//!
//! Knobs: `R4_ARM_SKIP_SAMPLE` (Gate C sample size, default 10,000),
//! `R4_ARM_SKIP_FIXTURE_DIR` (fixture directory, default the committed
//! `crates/uor-r4-core/tests/fixtures`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Keys of `gate_c` that a skipped run is EXPECTED to differ on. Everything
/// else must match exactly.
const EXPECTED_TO_DIFFER: [&str; 2] = ["right_context_arms", "skipped_arm_groups"];

fn fixture_dir() -> PathBuf {
    match std::env::var("R4_ARM_SKIP_FIXTURE_DIR") {
        Ok(dir) if !dir.trim().is_empty() => PathBuf::from(dir),
        _ => Path::new(env!("CARGO_MANIFEST_DIR")).join("crates/uor-r4-core/tests/fixtures"),
    }
}

fn sample_size() -> String {
    std::env::var("R4_ARM_SKIP_SAMPLE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "10000".to_owned())
}

/// One `r4 transformerless score` run. Returns the parsed report and the
/// wall clock, so the test doubles as the #471 before/after measurement.
fn score_run(out: &Path, skip_arms: Option<&str>) -> (serde_json::Value, f64) {
    let fixtures = fixture_dir();
    let _ = std::fs::remove_dir_all(out);
    let mut command = Command::new(env!("CARGO_BIN_EXE_r4"));
    command
        .args(["transformerless", "score"])
        .arg("--corpus-meta")
        .arg(fixtures.join("c_meta.bin"))
        .arg("--corpus-recs")
        .arg(fixtures.join("c_recs.bin"))
        .arg("--artifacts")
        .arg(fixtures.join("tless_artifacts.bin"))
        .arg("--out")
        .arg(out)
        .env("R4_GATE_C_SAMPLE", sample_size());
    match skip_arms {
        Some(groups) => {
            command.env("R4_GATE_C_SKIP_ARMS", groups);
        }
        // Explicitly REMOVED rather than merely unset by omission: an
        // inherited value from the caller's shell would silently turn the
        // control arm into a second skipped arm, and the comparison would
        // then pass by comparing a run to itself.
        None => {
            command.env_remove("R4_GATE_C_SKIP_ARMS");
        }
    }
    let started = Instant::now();
    let status = command.status().expect("failed to spawn the r4 binary");
    let seconds = started.elapsed().as_secs_f64();
    assert!(status.success(), "score run failed: {status}");
    let text = std::fs::read_to_string(out.join("score_report.json"))
        .expect("score run produced no report");
    (
        serde_json::from_str(&text).expect("score report is not valid JSON"),
        seconds,
    )
}

#[test]
#[ignore = "two full score runs over the 500k fixture; minutes, not seconds"]
fn skipping_the_right_context_arms_changes_no_row_it_still_reports() {
    let (control, control_seconds) =
        score_run(&PathBuf::from("/tmp/gate_c_arm_skip_control"), None);
    let (skipped, skipped_seconds) = score_run(
        &PathBuf::from("/tmp/gate_c_arm_skip_skipped"),
        Some("right_context"),
    );

    let control_gate = control["gate_c"]
        .as_object()
        .expect("control report has no gate_c object");
    let skipped_gate = skipped["gate_c"]
        .as_object()
        .expect("skipped report has no gate_c object");

    // ---- anti-vacuity: prove the control arm measured something ----
    let arms = control_gate["right_context_arms"].as_object().expect(
        "control run must EVALUATE the right-context arms; if this is null the \
                 control arm was itself skipped and every comparison below is vacuous",
    );
    let two_sided_positions = arms["rule12_twosided"]["positions"]
        .as_u64()
        .expect("two-sided row has no position count");
    assert!(
        two_sided_positions > 0,
        "control run scored zero two-sided positions — the fixture no longer exercises \
         this arm, so the equivalence assertion below would compare two empty runs"
    );
    let two_sided_top1 = arms["rule12_twosided"]["top1_agreement"]
        .as_f64()
        .expect("two-sided row has no top-1 rate");
    let baseline_top1 = control_gate["rule12_precedence"]["top1_agreement"]
        .as_f64()
        .expect("rule12_precedence row has no top-1 rate");
    assert!(
        (two_sided_top1 - baseline_top1).abs() > f64::EPSILON,
        "control two-sided top-1 ({two_sided_top1}) equals the Rule 1+2 baseline \
         ({baseline_top1}) — the arm was inert at every position, so a skipped run \
         would be indistinguishable from a full one for reasons that have nothing to \
         do with the knob under test"
    );

    // ---- the skip is declared, not silent ----
    assert!(
        skipped_gate["right_context_arms"].is_null(),
        "a skipped run must report the arms as ABSENT (null), never as zeroed rows"
    );
    assert_eq!(
        skipped_gate["skipped_arm_groups"],
        serde_json::json!(["right_context"]),
        "a skipped run must NAME the group it skipped"
    );
    assert_eq!(
        control_gate["skipped_arm_groups"],
        serde_json::json!([]),
        "a control run must report an empty skip list"
    );

    // ---- the claim: every retained row is bit-identical ----
    assert_eq!(
        control_gate.keys().collect::<Vec<_>>(),
        skipped_gate.keys().collect::<Vec<_>>(),
        "the two runs disagree on which gate_c keys exist; a skipped run must drop \
         VALUES, never the schema"
    );
    let mut compared = 0usize;
    for (key, control_value) in control_gate {
        if EXPECTED_TO_DIFFER.contains(&key.as_str()) {
            continue;
        }
        assert_eq!(
            control_value, &skipped_gate[key],
            "gate_c.{key} differs between a full run and a run that skipped the \
             right-context arms; the skip is a cost knob and must not move a number"
        );
        compared += 1;
    }
    assert!(
        compared > 20,
        "only {compared} gate_c keys were compared — the report shrank unexpectedly \
         and this test is no longer covering what it claims to"
    );

    // The artifact is upstream of Gate C entirely, so a skip that changed it
    // would mean the knob leaked out of the evaluation.
    assert_eq!(
        control["inputs"], skipped["inputs"],
        "the skip changed a compile input κ; it must touch the evaluation only"
    );
    assert_eq!(
        control["graph"], skipped["graph"],
        "the skip changed the emitted graph; it must touch the evaluation only"
    );

    println!(
        "#471 arm skip — control {control_seconds:.1}s, skipped {skipped_seconds:.1}s, \
         saving {:.1}s ({:.1}% of the control run, whole-pipeline); {compared} gate_c \
         keys compared and identical",
        control_seconds - skipped_seconds,
        100.0 * (control_seconds - skipped_seconds) / control_seconds.max(f64::EPSILON),
    );
}
