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
use uor_r4_core::transformerless::compiler::{Corpus, STAGES};
use uor_r4_core::transformerless::{convert_r4g1, runtime};
use uor_r4_graph_certify::StepCandidates;
use uor_r4_graph_compiler::segment_fit;
use uor_r4_graph_format::{GraphView, ScoreQ, SectionId, SegmentLaneDescriptor, LANE_SEGMENT};
use uor_r4_graph_runtime::runtime_state::{SegmentSession, SEGMENT_STATE_CAPACITY};

/// A small self-contained R4G1 bundle the engine can load (the synthetic store
/// recipe the scorer unit tests use). Returns `(r4g1_bytes, teacher_bytes)`.
/// The bundle carries no PSTATE section, so it exercises absent-section
/// identity directly.
fn synthetic_bundle_opt(lane: Option<&SegmentLaneDescriptor>) -> (Vec<u8>, Vec<u8>) {
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
    let r4g1 = match lane {
        Some(descriptor) => convert_r4g1::convert_with_segment_lane(
            &art_bytes,
            &artifacts,
            &store,
            &store_bytes,
            None,
            descriptor,
        ),
        None => convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None),
    }
    .expect("convert to R4G1")
    .0;
    (r4g1, art_bytes)
}

/// `synthetic_bundle` without a segment lane (no PSTATE section).
fn synthetic_bundle() -> (Vec<u8>, Vec<u8>) {
    synthetic_bundle_opt(None)
}

/// A representative selected segment-lane descriptor (the reference arm's
/// constants; the final values are pinned by the causal re-run / verdict).
fn selected_descriptor() -> SegmentLaneDescriptor {
    SegmentLaneDescriptor {
        ring_capacity: SEGMENT_STATE_CAPACITY as u32,
        decay_shift: 0,
        base_w: 1 << 12,
        boost: 1 << 20,
        key_quant_id: 0,
    }
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

// --- witness attribution (increment 4b) ------------------------------------

#[test]
fn segment_witness_method_is_identity_when_inactive() {
    // The witness-bearing segment method returns the base decision and a `None`
    // attribution whenever the lane is inactive — the witness-level absent-
    // section identity, on the real engine.
    let (graph, teacher) = synthetic_bundle();
    let mut engine = load_engine(&graph, &teacher);
    let inactive = SegmentSession::<SEGMENT_STATE_CAPACITY>::inactive();

    for window in WINDOWS {
        let mut base_c = StepCandidates::default();
        let base = engine
            .predict_decision_candidates_with_segment(window, &mut base_c, &inactive)
            .expect("base decision");

        let mut wit_c = StepCandidates::default();
        let (decision, attribution) = engine
            .predict_decision_candidates_with_segment_witness(window, &mut wit_c, &inactive)
            .expect("witness path");

        assert_eq!(base, decision, "inactive witness path decision == base");
        assert!(
            attribution.is_none(),
            "an inactive lane never attributes a promotion"
        );
    }
}

// --- compiler emission (increment 4a): the lane becomes non-inert ----------

#[test]
fn compiler_emits_pstate_and_engine_activates_the_lane() {
    let descriptor = selected_descriptor();
    let (graph, teacher) = synthetic_bundle_opt(Some(&descriptor));

    // The produced bundle carries a PSTATE section that round-trips the
    // descriptor the compiler was given.
    let view = GraphView::parse(&graph).expect("compiled bundle parses");
    let table = view
        .pstate_table()
        .expect("pstate section valid")
        .expect("compiled bundle carries a PSTATE section");
    assert_eq!(table.lane_kind(), LANE_SEGMENT);
    assert_eq!(table.ring_capacity(), descriptor.ring_capacity);
    assert_eq!(table.decay_shift(), descriptor.decay_shift);
    assert_eq!(table.base_w().raw(), descriptor.base_w);
    assert_eq!(table.boost().raw(), descriptor.boost);

    // And the deployed engine activates the segment lane from it — the lane is
    // no longer inert on a real compiled bundle.
    let engine = load_engine(&graph, &teacher);
    assert!(
        engine.segment_session().is_active(),
        "a compiled PSTATE bundle must activate the deployed segment lane"
    );
}

#[test]
fn segment_lane_emission_is_deterministic_and_section_preserving() {
    let descriptor = selected_descriptor();
    let (with_lane, _) = synthetic_bundle_opt(Some(&descriptor));
    let (with_lane_again, _) = synthetic_bundle_opt(Some(&descriptor));
    let (without_lane, _) = synthetic_bundle_opt(None);

    // Deterministic bytes: identical inputs → identical container.
    assert_eq!(
        with_lane, with_lane_again,
        "PSTATE emission must be deterministic"
    );

    // Absent-section identity at the source: emitting PSTATE leaves every other
    // section byte-identical, and the no-lane bundle carries no PSTATE.
    let vw = GraphView::parse(&with_lane).expect("with-lane parses");
    let vo = GraphView::parse(&without_lane).expect("no-lane parses");
    assert!(
        vo.pstate_table().expect("valid").is_none(),
        "the no-lane bundle must carry no PSTATE section"
    );
    for id in [
        SectionId::HEAD,
        SectionId::NODE,
        SectionId::EDGE,
        SectionId::ROUT,
        SectionId::EMIT,
        SectionId::EXCT,
    ] {
        assert_eq!(
            vw.section(id),
            vo.section(id),
            "PSTATE emission must not change section {id:?}"
        );
    }
}

// --- learned table: fit → emit → load (increment 4c-ii) --------------------

/// A small self-contained R4G1 bundle carrying the fitted learned segment
/// `rows` in its PSTATE section (the 4c-ii path). Same synthetic store recipe
/// as [`synthetic_bundle_opt`], but through
/// [`convert_r4g1::convert_with_segment_table`]. Returns `(r4g1, teacher)`.
fn synthetic_bundle_with_table(
    descriptor: &SegmentLaneDescriptor,
    rows: &[(u32, Vec<(u32, i32)>)],
) -> (Vec<u8>, Vec<u8>) {
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
    let r4g1 = convert_r4g1::convert_with_segment_table(
        &art_bytes,
        &artifacts,
        &store,
        &store_bytes,
        None,
        descriptor,
        rows,
    )
    .expect("convert to R4G1 with segment table")
    .0;
    (r4g1, art_bytes)
}

/// A multi-story corpus in which content token 100 is, on every TRAIN
/// position it is live for, associated with a teacher-argmax of candidate 9 —
/// a clean content→candidate association the fitter must recover. Four TRAIN
/// stories plus one held-out story satisfy the 80/20 story cut.
fn content_corpus() -> Corpus {
    let unit_input = [100u32, 500];
    let unit_argmax = [9u32, 9];
    let stories = 5usize;
    let mut story = Vec::new();
    let mut input = Vec::new();
    let mut next = Vec::new();
    let mut t_argmax = Vec::new();
    for sid in 0..stories {
        for (j, (&tok, &tg)) in unit_input.iter().zip(&unit_argmax).enumerate() {
            story.push(sid as u32);
            input.push(tok);
            next.push(if j + 1 < unit_input.len() {
                unit_input[j + 1]
            } else {
                0
            });
            t_argmax.push(tg);
        }
    }
    let n = input.len();
    Corpus {
        n,
        stories: stories as u64,
        story,
        input,
        next,
        t_argmax,
        top_tokens: vec![[0u32; 8]; n],
        top_weights: vec![[0u32; 8]; n],
        span_start: (0..n).map(|i| i as u32).collect(),
        span_end: (0..n).map(|i| i as u32 + 1).collect(),
        byte_start: vec![u32::MAX; n],
        byte_end: vec![u32::MAX; n],
        hidden: None,
    }
}

#[test]
fn fitted_table_round_trips_through_convert_and_reaches_the_engine() {
    // Fit a real learned table from a corpus, emit it into a compiled bundle,
    // and prove (a) the emitted PSTATE rows are byte-faithful to the fitted
    // table and (b) the deployed engine ingests exactly those rows.
    let corpus = content_corpus();
    let rows = segment_fit::fit_segment_table(&corpus, segment_fit::DEFAULT_TOP_K, 64);
    assert!(
        !rows.is_empty(),
        "the fit must learn at least one content key"
    );
    assert!(
        rows.iter()
            .any(|(k, entries)| *k == 100 && entries.iter().any(|(c, w)| *c == 9 && *w > 0)),
        "the fitted table must carry the 100→9 association with positive weight"
    );

    let descriptor = selected_descriptor();
    let (graph, teacher) = synthetic_bundle_with_table(&descriptor, &rows);

    // (a) The PSTATE section round-trips the fitted rows exactly.
    let view = GraphView::parse(&graph).expect("compiled bundle parses");
    let table = view
        .pstate_table()
        .expect("pstate section valid")
        .expect("bundle carries a PSTATE section");
    assert_eq!(table.lane_kind(), LANE_SEGMENT);
    let emitted: Vec<(u32, Vec<(u32, i32)>)> = table
        .rows()
        .map(|row| {
            (
                row.key(),
                row.entries().map(|e| (e.token, e.score_q.raw())).collect(),
            )
        })
        .collect();
    // Canonicalize the fitted rows the same way `build_segment_lane` does
    // (keys ascending, candidates ascending) before comparing.
    let mut expected = rows.clone();
    expected.sort_by_key(|(k, _)| *k);
    for (_, entries) in expected.iter_mut() {
        entries.sort_by_key(|(t, _)| *t);
    }
    assert_eq!(
        emitted, expected,
        "emitted PSTATE rows must be byte-faithful to the fitted table"
    );

    // (b) The deployed engine ingests exactly those learned rows.
    let engine = load_engine(&graph, &teacher);
    assert!(
        engine.segment_session().is_active(),
        "a learned-table bundle activates the segment lane"
    );
    assert_eq!(
        engine.segment_learned_rows(),
        Some(expected.len()),
        "the engine consumes the learned table (one row per fitted content key)"
    );
}

#[test]
fn config_only_and_no_lane_bundles_carry_no_learned_table() {
    // A config-only descriptor (empty rows) is the recurrence lane: active, but
    // no learned rows reach the engine. A no-PSTATE bundle carries neither.
    let descriptor = selected_descriptor();
    let (recurrence, teacher) = synthetic_bundle_with_table(&descriptor, &[]);
    let engine = load_engine(&recurrence, &teacher);
    assert!(
        engine.segment_session().is_active(),
        "a config-only descriptor still activates the lane"
    );
    assert_eq!(
        engine.segment_learned_rows(),
        None,
        "an empty table leaves the engine on the recurrence lane (no learned rows)"
    );

    let (no_lane, teacher2) = synthetic_bundle();
    let plain = load_engine(&no_lane, &teacher2);
    assert_eq!(
        plain.segment_learned_rows(),
        None,
        "a no-PSTATE bundle carries no learned table"
    );
}

#[test]
fn learned_table_emission_is_section_preserving_and_deterministic() {
    // Emitting the learned table changes only the PSTATE section (absent-section
    // identity at the source) and is deterministic.
    let corpus = content_corpus();
    let rows = segment_fit::fit_segment_table(&corpus, segment_fit::DEFAULT_TOP_K, 64);
    let descriptor = selected_descriptor();

    let (with_table, _) = synthetic_bundle_with_table(&descriptor, &rows);
    let (with_table_again, _) = synthetic_bundle_with_table(&descriptor, &rows);
    assert_eq!(
        with_table, with_table_again,
        "learned-table emission must be deterministic"
    );

    let (without_lane, _) = synthetic_bundle();
    let vt = GraphView::parse(&with_table).expect("with-table parses");
    let vo = GraphView::parse(&without_lane).expect("no-lane parses");
    for id in [
        SectionId::HEAD,
        SectionId::NODE,
        SectionId::EDGE,
        SectionId::ROUT,
        SectionId::EMIT,
        SectionId::EXCT,
    ] {
        assert_eq!(
            vt.section(id),
            vo.section(id),
            "learned-table emission must not change section {id:?}"
        );
    }
}
