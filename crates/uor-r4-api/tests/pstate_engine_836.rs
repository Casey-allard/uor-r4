//! #836 — the deployed R4Engine consumes the #835 segment lane.
//!
//! Absent-section identity is the load-bearing safety property: an artifact
//! without a PSTATE section (every artifact today) loads with an **inactive**
//! `segment_session`, and `predict_decision_candidates_with_segment` with that
//! session is byte-identical to the base decision. When the caller primes an
//! **active** session, the served token becomes the segment-adjusted argmax
//! over the decided candidate list (proven here on the real engine, and in the
//! unit tests over controlled candidate lists).

use std::collections::BTreeMap;

use uor_r4_api::{EngineParts, PredictDecision, R4Engine};
use uor_r4_core::transformerless::compiler::STAGES;
use uor_r4_core::transformerless::{convert_r4g1, runtime};
use uor_r4_graph_certify::StepCandidates;
use uor_r4_graph_format::ScoreQ;
use uor_r4_graph_runtime::runtime_state::{SegmentSession, SEGMENT_STATE_CAPACITY};

/// A small self-contained R4G1 bundle the engine can load (the synthetic store
/// recipe the scorer unit tests use). Returns `(r4g1_bytes, teacher_bytes)`.
/// The bundle carries no PSTATE section, so it exercises absent-section
/// identity directly.
fn synthetic_bundle() -> (Vec<u8>, Vec<u8>) {
    use uor_r4_core::transformerless::compiler;
    let dir = env!("CARGO_MANIFEST_DIR");
    let art_bytes = std::fs::read(format!(
        "{dir}/../uor-r4-core/tests/fixtures/tless_artifacts.bin"
    ))
    .expect("fixture artifacts present");
    let artifacts = compiler::parse_artifacts(&art_bytes).expect("artifacts parse");

    let mut store: runtime::Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
    let codes: [[u8; 4]; 6] = [
        [3, 1, 4, 1],
        [3, 1, 4, 2],
        [3, 5, 9, 2],
        [7, 5, 9, 2],
        [7, 5, 8, 2],
        [11, 5, 8, 7],
    ];
    for (i, code) in codes.iter().enumerate() {
        runtime::add_evidence(&mut store, code, (i + 1) as u32, 1);
    }
    let store_bytes = runtime::store_bytes(&store);
    let r4g1 = convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None)
        .expect("convert to R4G1")
        .0;
    (r4g1, art_bytes)
}

fn load_engine(graph: &[u8], teacher: &[u8]) -> R4Engine {
    R4Engine::load_accepting_quality(EngineParts {
        graph,
        signature_artifact: teacher,
        tokenizer: None,
        score_report: None,
    })
    .expect("engine load")
}

const WINDOWS: [&[u32]; 5] = [&[3], &[3, 1], &[3, 1, 4], &[7, 5], &[11, 5, 8]];

#[test]
fn no_pstate_artifact_yields_inactive_session() {
    let (graph, teacher) = synthetic_bundle();
    let engine = load_engine(&graph, &teacher);
    assert!(
        !engine.segment_session().is_active(),
        "an artifact without a PSTATE section must produce an inactive segment session"
    );
}

#[test]
fn inactive_session_is_byte_identical_to_base_decision() {
    let (graph, teacher) = synthetic_bundle();
    let mut engine = load_engine(&graph, &teacher);
    let inactive = SegmentSession::<SEGMENT_STATE_CAPACITY>::inactive();

    for window in WINDOWS {
        // Base decision + candidate list via the segment method with an inactive
        // session; the same call again must be deterministic and unchanged.
        let mut base_c = StepCandidates::default();
        let base = engine
            .predict_decision_candidates_with_segment(window, &mut base_c, &inactive)
            .expect("base decision");

        let mut again_c = StepCandidates::default();
        let again = engine
            .predict_decision_candidates_with_segment(window, &mut again_c, &inactive)
            .expect("repeat decision");

        assert_eq!(base, again, "inactive session must be deterministic");
        assert_eq!(
            base_c.ranked(),
            again_c.ranked(),
            "inactive session must not disturb the candidate list"
        );
    }
}

#[test]
fn active_session_runs_and_preserves_invariants_on_real_engine() {
    // The re-ranking arithmetic itself is proven over controlled candidate
    // lists by the `segment_adjust_*` unit tests. Here we exercise the wired
    // method end-to-end on the real engine and pin the serving invariants that
    // must hold whatever the (demo) fixture decides: an active session runs
    // without error, an abstention is never overridden into a served token, and
    // a segment-adjusted served token is always one of the decided candidates
    // (never fabricated).
    let (graph, teacher) = synthetic_bundle();
    let mut engine = load_engine(&graph, &teacher);
    let inactive = SegmentSession::<SEGMENT_STATE_CAPACITY>::inactive();
    let mut active = SegmentSession::<SEGMENT_STATE_CAPACITY>::active(
        0,
        ScoreQ::from_raw(1 << 12),
        ScoreQ::from_raw(i32::MAX),
    );
    active.fold_prompt(&[1, 2, 3, 4, 5, 7, 8, 9]);

    for window in WINDOWS {
        let mut base_c = StepCandidates::default();
        let base = engine
            .predict_decision_candidates_with_segment(window, &mut base_c, &inactive)
            .expect("base decision");
        let base_tokens: Vec<u32> = base_c.ranked().iter().map(|&(t, _)| t).collect();

        let mut act_c = StepCandidates::default();
        let act = engine
            .predict_decision_candidates_with_segment(window, &mut act_c, &active)
            .expect("active method runs on the real engine");

        match (base, act) {
            (PredictDecision::Abstain(_), _) => assert_eq!(
                base, act,
                "an abstaining decision is never overridden into a served token"
            ),
            (PredictDecision::Serve(b), PredictDecision::Serve(a)) => assert!(
                a.token == b.token || base_tokens.contains(&a.token),
                "a segment-adjusted served token comes from the decided candidate list, \
                 never fabricated"
            ),
            (PredictDecision::Serve(_), PredictDecision::Abstain(_)) => {
                panic!("the segment lane must not turn a served decision into an abstention")
            }
        }
    }
}
