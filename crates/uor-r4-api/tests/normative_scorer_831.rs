//! #831 — normative R4G1 scorer designation: machine-checked evidence.
//!
//! Decision record: `docs/adr/0001-normative-r4g1-scorer.md`; evidence record:
//! `docs/normative_scorer_831.md`.
//!
//! These tests are the fail-closed teeth behind ADR-0001. They exercise real
//! production code from three crates and prove a planted divergence from the
//! normative scoring semantics is detected:
//!   * normative specification — `uor-r4-graph-format::scoring_semantics`,
//!   * deployed runtime accumulator/selector — `uor-r4-graph-runtime::scoring`,
//!   * reference/certifier scorer — `uor-r4-graph-certify::GraphScorer`.

use std::cmp::Ordering;

use uor_r4_graph_certify::{GraphScorer, DEFAULT_EXCT_TOP_X, DEFAULT_ROOT_TOP_B};
use uor_r4_graph_format::scoring_semantics::{
    ResidualContribution, ResidualContributionKind, ScoreAccumulator, ScoringSemanticsVerifier,
    ScoringSemanticsVersion,
};
use uor_r4_graph_format::ScoreQ;
use uor_r4_graph_runtime::scoring::{
    accumulate_reference, select_best, OrderedScore, ResidualKind, TypedContribution,
};
use uor_r4_graph_runtime::R4G1Runtime;

// --- normative oracle vs deployed runtime: accumulation ----------------------

/// Oracle total for a duplicate-free residual set: the normative
/// `scoring_semantics` accumulator (graph-format crate).
fn oracle_total(items: &[(u32, i32)]) -> i32 {
    let mut acc = ScoreAccumulator::<32>::new();
    for &(id, raw) in items {
        acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::TokenEmission,
            contribution_id: id,
            raw_value: raw,
        })
        .expect("within evidence capacity");
    }
    acc.score()
}

/// Deployed runtime total for the same set: `accumulate_reference`
/// (graph-runtime crate). `None` when the set repeats an evidence id.
fn runtime_total(items: &[(u32, i32)]) -> Option<ScoreQ> {
    let contributions: Vec<TypedContribution> = items
        .iter()
        .map(|&(id, raw)| TypedContribution {
            evidence_id: id,
            kind: ResidualKind::Emission,
            value: ScoreQ::from_raw(raw),
        })
        .collect();
    accumulate_reference(ScoreQ::ZERO, &contributions)
}

const ACC_CASES: &[&[(u32, i32)]] = &[
    &[(1, 100), (2, -25), (3, -10)],
    &[(1, 0)],
    &[(5, i32::MAX - 5), (6, 10)],  // positive saturation
    &[(7, i32::MIN + 5), (8, -10)], // negative saturation
    &[(9, 250), (10, -250), (11, 1)],
    &[(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)],
];

#[test]
fn spec_and_runtime_accumulators_agree_on_disjoint_sets() {
    for &case in ACC_CASES {
        let oracle = oracle_total(case);
        let runtime = runtime_total(case).expect("duplicate-free set accumulates");
        assert_eq!(
            runtime.raw(),
            oracle,
            "normative spec and deployed runtime accumulator diverged on {case:?}"
        );
    }
}

// --- normative oracle vs deployed runtime: selection -------------------------

/// Normative winner via the spec `compare_candidates` (score desc, id asc).
fn oracle_winner(cands: &[(u32, i32)]) -> u32 {
    let mut best = cands[0];
    for &c in &cands[1..] {
        if ScoreAccumulator::<4>::compare_candidates(c.1, c.0, best.1, best.0) == Ordering::Less {
            best = c;
        }
    }
    best.0
}

/// Deployed runtime winner via `select_best` (graph-runtime crate).
fn runtime_winner(cands: &[(u32, i32)]) -> u32 {
    let ordered: Vec<(u32, OrderedScore)> = cands
        .iter()
        .map(|&(id, raw)| (id, OrderedScore::Real(ScoreQ::from_raw(raw))))
        .collect();
    select_best(&ordered).expect("non-empty candidate set").0
}

const SELECT_CASES: &[&[(u32, i32)]] = &[
    &[(10, 500), (20, 500)],         // tie -> lowest id (10)
    &[(3, 900), (1, 100), (2, 900)], // tie at top -> lowest id (2)
    &[(7, -5), (8, -5), (9, -6)],    // negative scores, tie -> lowest id (7)
    &[(2, i32::MAX), (1, i32::MAX)], // saturated tie -> lowest id (1)
];

#[test]
fn spec_and_runtime_selectors_agree() {
    for &case in SELECT_CASES {
        assert_eq!(
            runtime_winner(case),
            oracle_winner(case),
            "normative spec and deployed runtime selector diverged on {case:?}"
        );
    }
}

// --- no-double-counting, honored by both (in each one's documented way) ------

#[test]
fn no_double_counting_enforced_by_both() {
    // Spec accumulator: a repeated contribution id is ignored; score unchanged.
    let mut acc = ScoreAccumulator::<8>::new();
    let item = ResidualContribution {
        kind: ResidualContributionKind::InteractionResidual,
        contribution_id: 42,
        raw_value: 250,
    };
    assert!(acc.accumulate(&item).expect("within capacity"));
    assert!(
        !acc.accumulate(&item).expect("within capacity"),
        "spec must ignore a repeated contribution id"
    );
    assert_eq!(acc.score(), 250);
    assert_eq!(acc.evidence_count(), 1);

    // Runtime accumulator: a set carrying a duplicated evidence id is rejected.
    let dup = [
        TypedContribution {
            evidence_id: 42,
            kind: ResidualKind::Emission,
            value: ScoreQ::from_raw(250),
        },
        TypedContribution {
            evidence_id: 42,
            kind: ResidualKind::Emission,
            value: ScoreQ::from_raw(250),
        },
    ];
    assert!(
        accumulate_reference(ScoreQ::ZERO, &dup).is_none(),
        "runtime accumulate_reference must reject a duplicated evidence id"
    );
}

// --- planted negative: the differential has teeth ----------------------------

/// Planted-negative selector: higher-id-wins on ties (violates id-ascending).
fn divergent_winner(cands: &[(u32, i32)]) -> u32 {
    let mut best = cands[0];
    for &c in &cands[1..] {
        if c.1 > best.1 || (c.1 == best.1 && c.0 > best.0) {
            best = c;
        }
    }
    best.0
}

/// Planted-negative accumulator: wrapping add (violates saturation).
fn wrapping_total(items: &[(u32, i32)]) -> i32 {
    let mut total: i32 = 0;
    for &(_, raw) in items {
        total = total.wrapping_add(raw);
    }
    total
}

#[test]
fn planted_divergence_is_detected() {
    // Tie case: normative winner is the lower id; the planted higher-id-wins
    // selector must disagree — proving the differential would catch a real
    // scorer that violated the normative tie-break.
    let tie: &[(u32, i32)] = &[(10, 500), (20, 500)];
    assert_eq!(oracle_winner(tie), 10);
    assert_eq!(runtime_winner(tie), 10);
    assert_ne!(
        divergent_winner(tie),
        oracle_winner(tie),
        "planted higher-id-wins selector must diverge from the normative tie-break"
    );

    // Saturation case: normative total clamps; the planted wrapping accumulator
    // must diverge.
    let sat: &[(u32, i32)] = &[(1, i32::MAX - 5), (2, 10)];
    assert_eq!(oracle_total(sat), i32::MAX, "spec saturates high");
    assert_eq!(
        runtime_total(sat).unwrap().raw(),
        i32::MAX,
        "runtime saturates high"
    );
    assert_ne!(
        wrapping_total(sat),
        oracle_total(sat),
        "planted wrapping accumulator must diverge from the normative saturation"
    );
}

// --- single-source scorer status space + spec pin ----------------------------

#[test]
fn scorer_status_is_single_source_and_spec_pinned() {
    // Compiles only if `uor-r4-api::ResolutionStatus` is the *same type* as the
    // certifier scorer's `ScoreStatus` — the single-source scorer status space.
    let coerce: fn(uor_r4_api::ResolutionStatus) -> uor_r4_graph_certify::ScoreStatus = |s| s;
    let _ = coerce;

    assert_eq!(
        ScoringSemanticsVerifier::version(),
        ScoringSemanticsVersion::V1_0_0
    );
    assert!(
        ScoringSemanticsVerifier::audit_scoring_compliance().is_none(),
        "normative scoring-semantics self-audit must report no violation"
    );
}

// --- reachability + fail-closed boundary -------------------------------------

/// A small, self-contained R4G1 bundle both scorers can read (the synthetic
/// store recipe used by the runtime unit tests). Returns `(r4g1_bytes,
/// teacher_bytes)`; the teacher container carries the pinned `teacher_cid`
/// that the reference scorer's EXCT path verifies fail-closed.
fn synthetic_bundle() -> (Vec<u8>, Vec<u8>) {
    use std::collections::BTreeMap;
    use uor_r4_core::transformerless::compiler::{self, STAGES};
    use uor_r4_core::transformerless::{convert_r4g1, runtime};

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

#[test]
fn reference_and_normative_reachable_from_same_artifact() {
    let (bytes, teacher) = synthetic_bundle();

    // Deployed normative scorer is reachable and deterministic.
    let runtime = R4G1Runtime::parse(&bytes).expect("normative deployed scorer parses");
    assert!(runtime.node_count() > 0);
    let mut ns1 = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let mut ns2 = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let a = runtime.predict_distribution(&[1, 2, 3], None, &mut ns1);
    let b = runtime.predict_distribution(&[1, 2, 3], None, &mut ns2);
    assert_eq!(
        a, b,
        "normative scorer must be deterministic on identical input"
    );

    // Reference/certifier scorer is reachable from the *same* artifact bytes
    // (with the pinned teacher container its EXCT path verifies fail-closed).
    let scorer = GraphScorer::from_artifact(
        &bytes,
        Some(&teacher),
        DEFAULT_ROOT_TOP_B,
        DEFAULT_EXCT_TOP_X,
    );
    assert!(
        scorer.is_some(),
        "reference scorer must construct from the same normative artifact bytes"
    );
}

#[test]
fn incompatible_artifacts_fail_closed() {
    let garbage = [0u8; 32];

    // Both the normative and the reference scorer reject non-artifact bytes.
    assert!(
        R4G1Runtime::parse(&garbage).is_err(),
        "normative scorer must reject non-artifact bytes"
    );
    assert!(
        GraphScorer::from_artifact(&garbage, None, DEFAULT_ROOT_TOP_B, DEFAULT_EXCT_TOP_X)
            .is_none(),
        "reference scorer must reject non-artifact bytes"
    );

    // Patch owner: an incompatible patch is rejected (fail closed).
    let (bytes, _teacher) = synthetic_bundle();
    let mut runtime = R4G1Runtime::parse(&bytes).expect("parse normative artifact");
    assert!(
        runtime.try_push_patch(&garbage).is_some(),
        "incompatible patch bytes must fail closed with a rejection reason"
    );
}
