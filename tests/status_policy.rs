//! Probe suite for the ResolutionStatus-driven deployed path (issue #78,
//! decision D4). Against a hand-built scored artifact (helpers shared with
//! the census in `tests/status_policy_common/`, mirroring
//! `crates/uor-r4-core/tests/score.rs`):
//!
//! - (a) an out-of-distribution probe resolves Novel, widens once, then
//!   abstains with the status recorded;
//! - (b) covered probes serve with ExactContext / Graph statuses, and the
//!   deployed allocation-free step matches the reference scorer exactly;
//! - (c) a second identical Novel probe does NOT widen again — the
//!   widen-once bound holds (counters asserted);
//! - (d) adversarial repetition of the same probe is deterministic;
//! - generation stops at the first abstention (count + status returned);
//! - the manifest policy is data (defaults + score-report override);
//! - the delimited policy block in `crates/uor-r4-api/src/engine.rs`
//!   (moved from `src/r4g1.rs`, which now wraps the library engine) is
//!   integer-only by source
//!   scan (the P-4 pattern; `score_runtime.rs` is covered whole-file by
//!   `tests/score.rs`'s scan in the core crate).

mod status_policy_common;

use status_policy_common as fixture;

use uor_r4_graph_certify::{ScoreStatus, TOP_M, WIDENED_TOP_M};
use uor_r4_wasm_router::r4g1::{
    AbstainOutcome, PolicyStatus, PredictDecision, PredictOutcome, StatusAction, StatusPolicy,
};

// ------------------------------------------------------- (a) OOD probe --

#[test]
fn ood_probe_widens_once_then_abstains_with_status_recorded() {
    let fixture = fixture::signature_fixture(None);
    let state = fixture.load();
    let decision = state
        .predict_signature_status(&fixture.ood_sig)
        .expect("decision");
    assert_eq!(
        decision,
        PredictDecision::Abstain(AbstainOutcome {
            status: ScoreStatus::Novel,
            widened: true,
        }),
        "OOD input resolves Novel, widens once, then abstains"
    );
    let counters = state.policy_counters();
    assert_eq!(counters.predicts, 1);
    assert_eq!(counters.serves, 0);
    assert_eq!(counters.abstains, 1);
    assert_eq!(counters.widen_attempts, 1);
    assert_eq!(counters.widen_skipped_seen, 0);
}

// --------------------------------------------------- (b) covered probe --

#[test]
fn covered_probes_serve_with_exact_context_and_graph_status() {
    let fixture = fixture::signature_fixture(None);
    let state = fixture.load();

    // ExactContext: the covered signature carries exact-context evidence
    // with support ≥ EXCT_SUPPORT_MIN (Rule 2); token 10 is the only
    // admitted entry.
    let exact = state
        .predict_signature_status(&fixture.covered_sig)
        .expect("decision");
    assert_eq!(
        exact,
        PredictDecision::Serve(PredictOutcome {
            token: 10,
            status: ScoreStatus::ExactContext,
            widened: false,
        })
    );

    // Graph: the all-ones signature is covered by region 1 (Rule 1):
    // S(20) = B(20) + ΔE(2,20) = 200 + 2000 = 2200 is the argmax.
    let graph = state
        .predict_signature_status(&fixture.graph_sig)
        .expect("decision");
    assert_eq!(
        graph,
        PredictDecision::Serve(PredictOutcome {
            token: 20,
            status: ScoreStatus::Graph,
            widened: false,
        })
    );

    let counters = state.policy_counters();
    assert_eq!(counters.serves, 2);
    assert_eq!(counters.abstains, 0);
    assert_eq!(counters.widen_attempts, 0);
}

/// The deployed allocation-free step (`score_step`) matches the
/// witness-emitting reference scorer exactly — selected token, score,
/// status, candidate count — on every probe signature, at both the base
/// and the widened membership width (widening only enlarges the
/// membership lists, so the two-region fixture resolves identically).
#[test]
fn deployed_step_matches_the_reference_scorer_exactly() {
    let fixture = fixture::signature_fixture(None);
    let scorer = fixture.reference_scorer();
    let mut step_state = scorer.step_state(WIDENED_TOP_M).expect("step state");
    for sig in [fixture.covered_sig, fixture.graph_sig, fixture.ood_sig] {
        let reference = scorer.score_candidates(&sig, &[]).expect("reference");
        let recent = [reference.selected];
        let penalized_reference = scorer
            .score_candidates(&sig, &recent)
            .expect("penalized reference");
        for top_m in [TOP_M, WIDENED_TOP_M] {
            let step = scorer
                .score_step(&sig, top_m, &mut step_state)
                .expect("step");
            assert_eq!(step.selected, reference.selected, "top_m {top_m}");
            assert_eq!(
                step.selected_score, reference.selected_score,
                "top_m {top_m}"
            );
            assert_eq!(step.status, reference.witness.status, "top_m {top_m}");
            assert_eq!(
                step.candidate_count, reference.witness.candidate_count,
                "top_m {top_m}"
            );
            let penalized_step = scorer
                .score_step_with_recent(&sig, top_m, &mut step_state, &recent)
                .expect("penalized step");
            assert_eq!(
                penalized_step.selected, penalized_reference.selected,
                "penalized top_m {top_m}"
            );
            assert_eq!(
                penalized_step.selected_score, penalized_reference.selected_score,
                "penalized top_m {top_m}"
            );
        }
    }
}

// ------------------------------------------- (c) widen-once bound -----

#[test]
fn widen_once_bound_holds_for_repeated_novel_probes() {
    let fixture = fixture::signature_fixture(None);
    let state = fixture.load();
    let first = state
        .predict_signature_status(&fixture.ood_sig)
        .expect("first");
    assert_eq!(
        first,
        PredictDecision::Abstain(AbstainOutcome {
            status: ScoreStatus::Novel,
            widened: true,
        })
    );
    // A second identical Novel input does NOT widen again: the bounded
    // widen-once memory answers and the probe abstains immediately.
    let second = state
        .predict_signature_status(&fixture.ood_sig)
        .expect("second");
    assert_eq!(
        second,
        PredictDecision::Abstain(AbstainOutcome {
            status: ScoreStatus::Novel,
            widened: false,
        })
    );
    let counters = state.policy_counters();
    assert_eq!(
        counters.widen_attempts, 1,
        "the second identical probe must not widen again"
    );
    assert_eq!(counters.widen_skipped_seen, 1);
    assert_eq!(counters.abstains, 2);
    assert_eq!(counters.serves, 0);
}

// --------------------------------------- (d) adversarial repetition --

#[test]
fn adversarial_repetition_is_deterministic() {
    let fixture = fixture::signature_fixture(None);
    let state = fixture.load();
    let mut outcomes = Vec::new();
    for _ in 0..16 {
        outcomes.push(
            state
                .predict_signature_status(&fixture.ood_sig)
                .expect("decision"),
        );
    }
    for (i, decision) in outcomes.iter().enumerate() {
        assert!(
            matches!(
                decision,
                PredictDecision::Abstain(AbstainOutcome {
                    status: ScoreStatus::Novel,
                    ..
                })
            ),
            "repetition {i} abstains with status Novel"
        );
    }
    assert!(
        outcomes.iter().skip(1).all(|d| d == &outcomes[1]),
        "every repetition after the first is identical"
    );
    let counters = state.policy_counters();
    assert_eq!(
        counters.widen_attempts, 1,
        "sixteen adversarial repetitions widen at most once"
    );
    assert_eq!(counters.abstains, 16);
    assert_eq!(counters.serves, 0);
    // Interleaved served probes are unaffected and identical too.
    for _ in 0..4 {
        let served = state
            .predict_signature_status(&fixture.covered_sig)
            .expect("served");
        assert_eq!(
            served,
            PredictDecision::Serve(PredictOutcome {
                token: 10,
                status: ScoreStatus::ExactContext,
                widened: false,
            })
        );
    }
}

// ------------------------------------------- generation abstention --

#[test]
fn generation_stops_at_the_first_abstain() {
    let fixture = fixture::window_fixture();
    let state = fixture.load();
    let ood_window = fixture::find_window_by_status(&fixture, ScoreStatus::Novel);

    // An out-of-distribution seed abstains at step one: zero tokens, the
    // abstaining status recorded, no guessed token emitted.
    let mut out = [0u32; 8];
    let outcome = state
        .generate_into_status(&ood_window, &mut out)
        .expect("generate");
    assert_eq!(outcome.count, 0);
    assert!(outcome.abstained);
    assert_eq!(outcome.status, Some(ScoreStatus::Novel));

    // The covered seed serves its first token (Graph rule, token 10) —
    // the served path is intact. Later steps follow the same per-step
    // policy, so the run may abstain after the first token; the first
    // token and the absence of a guessed substitute are what matter here.
    let mut out = [0u32; 8];
    let outcome = state
        .generate_into_status(&fixture.covered_window, &mut out)
        .expect("generate");
    assert!(!outcome.abstained || outcome.count >= 1);
    assert!(outcome.count >= 1);
    assert_eq!(out[0], 10);

    // The legacy delegates keep their signatures: the window delegate
    // errors on abstention instead of guessing, the generate delegate
    // returns the same count.
    assert!(state.predict_window(&ood_window).is_err());
    assert_eq!(state.predict_window(&fixture.covered_window).ok(), Some(10));
    let mut legacy_out = [0u32; 8];
    let legacy_count = state
        .generate_into(&fixture.covered_window, &mut legacy_out)
        .expect("legacy generate");
    assert_eq!(legacy_count, outcome.count);
    assert_eq!(legacy_out[..legacy_count], out[..outcome.count]);
}

#[test]
fn out_of_vocabulary_window_is_a_typed_error_not_a_panic() {
    let fixture = fixture::window_fixture();
    let state = fixture.load();
    // The fixture teacher carries 64 token rows; id 64 and above are
    // rejected at the prediction boundary (the HTTP endpoints accept
    // arbitrary client-supplied token ids).
    let err = state
        .predict_window_status(&[5, 64])
        .expect_err("out-of-vocabulary window rejected");
    assert!(err.contains("outside the teacher vocabulary"), "{err}");
    let err = state
        .generate_into_status(&[999_999], &mut [0u32; 4])
        .expect_err("out-of-vocabulary seed rejected");
    assert!(err.contains("outside the teacher vocabulary"), "{err}");
    // The boundary itself still works (a decision, not the vocab error).
    assert!(state.predict_window_status(&[63]).is_ok());
    assert!(state.predict_window_status(&[5, 63, 0]).is_ok());
}

// ------------------------------------------------- the manifest policy --

#[test]
fn manifest_defaults_implement_d4() {
    let policy = StatusPolicy::default();
    assert_eq!(
        policy.action(PolicyStatus::ExactContext),
        StatusAction::Serve
    );
    assert_eq!(policy.action(PolicyStatus::Graph), StatusAction::Serve);
    assert_eq!(policy.action(PolicyStatus::Novel), StatusAction::WidenOnce);
    assert_eq!(
        policy.action(PolicyStatus::Contradictory),
        StatusAction::Abstain,
        "the Contradictory arm is declared and enforced (reserved status)"
    );
    assert_eq!(PolicyStatus::Contradictory.label(), "contradictory");
    assert_eq!(PolicyStatus::ExactContext.label(), "exact_context");
    assert_eq!(PolicyStatus::Graph.label(), "graph");
    assert_eq!(PolicyStatus::Novel.label(), "novel");
}

#[test]
fn score_report_override_replaces_rows_leniently() {
    let report = serde_json::json!({
        "config": {
            "status_policy": {
                "novel": "abstain",
                "graph": "not-an-action",
            }
        }
    });
    let policy = StatusPolicy::from_report(Some(&report));
    assert_eq!(policy.action(PolicyStatus::Novel), StatusAction::Abstain);
    assert_eq!(
        policy.action(PolicyStatus::Graph),
        StatusAction::Serve,
        "an unknown value keeps the default for that row"
    );
    assert_eq!(
        policy.action(PolicyStatus::ExactContext),
        StatusAction::Serve
    );
    assert_eq!(
        policy.action(PolicyStatus::Contradictory),
        StatusAction::Abstain
    );
    // A missing report and an empty object both keep every default.
    assert_eq!(StatusPolicy::from_report(None), StatusPolicy::default());
    let empty = serde_json::json!({});
    assert_eq!(
        StatusPolicy::from_report(Some(&empty)),
        StatusPolicy::default()
    );
}

#[test]
fn override_abstain_on_novel_skips_widening() {
    let fixture = fixture::signature_fixture(Some(serde_json::json!({
        "novel": "abstain"
    })));
    let state = fixture.load();
    assert_eq!(
        state.policy().action(PolicyStatus::Novel),
        StatusAction::Abstain,
        "load() wires the score-report override"
    );
    let decision = state
        .predict_signature_status(&fixture.ood_sig)
        .expect("decision");
    assert_eq!(
        decision,
        PredictDecision::Abstain(AbstainOutcome {
            status: ScoreStatus::Novel,
            widened: false,
        })
    );
    assert_eq!(state.policy_counters().widen_attempts, 0);
    // The served path is unaffected by the override.
    let served = state
        .predict_signature_status(&fixture.covered_sig)
        .expect("served");
    assert!(matches!(served, PredictDecision::Serve(_)));
}

// ------------------------------------- manual verification helper --

/// Manual verification helper (not part of the suite): writes the window
/// fixture bundle to `$R4_STATUS_FIXTURE_DIR` (default
/// `/tmp/r4-status-fixture`) WITHOUT cleanup, so a live server can load
/// it — used to verify the HTTP abstain/served response shapes end to
/// end. Run with:
/// `cargo test -p uor-r4-wasm-router --test status_policy -- --ignored --nocapture`
#[test]
#[ignore]
fn materialize_window_fixture_bundle() {
    let dir = std::env::var("R4_STATUS_FIXTURE_DIR")
        .unwrap_or_else(|_| "/tmp/r4-status-fixture".to_owned());
    let fixture = fixture::window_fixture();
    let target = std::path::Path::new(&dir);
    std::fs::create_dir_all(target).expect("create target dir");
    std::fs::copy(fixture.dir.join("score.r4g1"), target.join("score.r4g1")).expect("copy graph");
    std::fs::copy(
        fixture.dir.join("tless_artifacts.bin"),
        target.join("tless_artifacts.bin"),
    )
    .expect("copy teacher");
    let ood = fixture::find_window_by_status(&fixture, ScoreStatus::Novel);
    println!("fixture bundle written to {dir}");
    println!("covered window: {:?}", fixture.covered_window);
    println!("ood window: {ood:?}");
    // Keep the temporary source alive until after the copies above; the
    // fixture's Drop removes it at the end of the test as usual.
    drop(fixture);
}

// ------------------------------------------- integer-only source scan --

/// The delimited status-policy block of `crates/uor-r4-api/src/engine.rs`
/// (moved from `src/r4g1.rs`; the root module is now a thin wrapper over
/// the library engine) carries no
/// `f32`/`f64`, no `*` `/` `%` value arithmetic, and no unsafe code
/// (the P-4 scan pattern of `transformerless/mod.rs`, mirrored from
/// `crates/uor-r4-core/tests/score.rs`; the deployed `score_step` code in
/// `score_runtime.rs` is covered by that crate's whole-file scan).
#[test]
fn deployed_status_path_is_integer_only_by_source_scan() {
    let src = include_str!("../crates/uor-r4-api/src/engine.rs");
    let mut block = String::new();
    let mut in_block = false;
    let mut blocks = 0u32;
    for line in src.lines() {
        if line.contains("BEGIN DEPLOYED STATUS POLICY (INTEGER-ONLY)") {
            in_block = true;
            blocks += 1;
            continue;
        }
        if line.contains("END DEPLOYED STATUS POLICY (INTEGER-ONLY)") {
            in_block = false;
            continue;
        }
        if in_block {
            block.push_str(line);
            block.push('\n');
        }
    }
    assert_eq!(blocks, 1, "exactly one delimited status-policy block");
    assert!(!block.is_empty(), "the block carries the policy code");
    for (ln, line) in block.lines().enumerate() {
        let code = line.trim_start();
        if code.starts_with("//") {
            continue;
        }
        assert!(
            !code.contains("f32") && !code.contains("f64"),
            "float type in the deployed status path, line {}: {}",
            ln + 1,
            code
        );
    }
    let mut offenders = Vec::new();
    for (ln, line) in block.lines().enumerate() {
        let code = line.trim_start();
        if code.starts_with("//") {
            continue;
        }
        let b = code.as_bytes();
        for (i, &ch) in b.iter().enumerate() {
            if ch != b'*' && ch != b'/' && ch != b'%' {
                continue;
            }
            if ch == b'/' && ((i + 1 < b.len() && b[i + 1] == b'/') || (i >= 1 && b[i - 1] == b'/'))
            {
                continue; // comment slashes
            }
            let prev = if i >= 2 && b[i - 1] == b' ' {
                b[i - 2]
            } else if i >= 1 {
                b[i - 1]
            } else {
                b' '
            };
            let next = if i + 2 < b.len() && b[i + 1] == b' ' {
                b[i + 2]
            } else if i + 1 < b.len() {
                b[i + 1]
            } else {
                b' '
            };
            let operand_l =
                |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b')' || c == b']';
            let operand_r = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b'(';
            if operand_l(prev) && operand_r(next) {
                offenders.push(format!("line {}: {}", ln + 1, code));
                break;
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "value arithmetic in the deployed status path:\n{}",
        offenders.join("\n")
    );
    assert!(!block.contains("unsafe"), "no unsafe in the status path");
}
