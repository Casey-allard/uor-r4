//! `MsaStructuredSelectorV1` grounding, reference, and witness tests
//! (#643).
//!
//! The operator under test is DORMANT: these tests construct it
//! directly through `uor-r4-graph-certify::msa_selector`; no serving
//! path is exercised or changed, and no packed R4G1 lowering exists yet
//! (#604's own two-stage plan: reference semantics land first).
//!
//! # Pre-registered exit rule (issue #643 — binding)
//!
//! This operator is a candidate to REPLACE or supplement
//! `r4-route-attention/1` as the target selection mechanism. The two
//! are plug-compatible by construction (identical `value_aggregation`
//! and `tie_breaking` shape, module docs of both operators) precisely
//! so a fair A/B is possible once a shared evaluation harness exists.
//!
//! POSITIVE iff, on the SAME held-out evaluation corpus and candidate
//! set `r4-route-attention/1` is measured against, `msa-structured-
//! selector/1` achieves top-1 next-token accuracy >= r4-route-
//! attention/1's top-1 accuracy + 0.02 absolute, AND does not increase
//! bits-per-token versus r4-route-attention/1, AND beats the #457
//! unigram floor. NEGATIVE (including "no measurable difference") ⇒
//! close #643 with the comparison table and register a
//! `msa-structured-selector-dormant` lane in `model/ledger.toml`, the
//! same disposition `r4-route-attention/1` itself would receive on a
//! negative result.
//!
//! This criterion is posted before any run, per the #626 convention.
//! Running it requires wiring `msa-structured-selector/1` into the same
//! held-out evaluation loop `r4-route-attention/1` uses (a packed
//! lowering, or at minimum a harness adapter) — that wiring is
//! explicitly OUT OF SCOPE for this slice (see #643's pinned scope
//! comment) and is the next concrete action once this reference lands.

use uor_r4_graph_certify::msa_selector::{
    classify, expected_msa_selector_census, replay_msa_selector_witness, MsaClassification,
    MsaSelectorReference, ROLE_GEN, ROLE_MAN, ROLE_MED, ROLE_ZERO,
};
use uor_r4_graph_format::ScoreQ;
use uor_r4_model_source::attention::{operator_spec, AttentionOperatorSpec};

// ------------------------------------------------------- grounding tests --

/// MSA7 ("The 11-Theorem") + Theorem M4 ("The 11-Cascade Theorem"),
/// transcribed literally: `mod_11(γ)=2` at cascade position 0 (Gen),
/// `mod_11(μ)=4` at position 1 (Med), `mod_11(ε)=8` at position 2
/// (Man). These three (residue, role, position) triples are the ONLY
/// ones the paper proves; everything else this module derives is an
/// explicit extension (module docs).
#[test]
fn role_anchors_match_msa7_and_the_11_theorem() {
    assert_eq!(
        classify(2),
        MsaClassification {
            role_rank: ROLE_GEN,
            cascade_position: 0
        },
        "mod_11(gamma) = 2 is the Gen anchor at cascade position 0"
    );
    assert_eq!(
        classify(4),
        MsaClassification {
            role_rank: ROLE_MED,
            cascade_position: 1
        },
        "mod_11(mu) = 4 is the Med anchor at cascade position 1"
    );
    assert_eq!(
        classify(8),
        MsaClassification {
            role_rank: ROLE_MAN,
            cascade_position: 2
        },
        "mod_11(epsilon) = 8 is the Man anchor at cascade position 2"
    );
}

/// Theorem M4's literal cascade `(2, 4, 8, 5, 10, 9, 7, 3, 6, 1)`,
/// transcribed as the full residue -> cascade_position table. This is
/// the paper's own worked proof (module docs of `route_attention.rs`'s
/// counterpart theorem citations follow the same "transcribe, then
/// test" discipline).
#[test]
fn cascade_orbit_matches_theorem_m4_exactly() {
    let expected_positions = [
        (2u32, 0u8),
        (4, 1),
        (8, 2),
        (5, 3),
        (10, 4),
        (9, 5),
        (7, 6),
        (3, 7),
        (6, 8),
        (1, 9),
    ];
    for (residue, position) in expected_positions {
        assert_eq!(
            classify(residue).cascade_position,
            position,
            "residue {residue} should sit at cascade position {position}"
        );
    }
}

/// Positions 0..9 mod 3, the mod-3 role extension confirmed by Casey
/// (2026-08-16) for the 7 residues MSA7 does not itself assign a role
/// to.
#[test]
fn role_extension_is_cascade_position_mod_3() {
    let expected_roles = [
        (5u32, ROLE_GEN), // position 3, 3 % 3 == 0
        (10, ROLE_MED),   // position 4, 4 % 3 == 1
        (9, ROLE_MAN),    // position 5, 5 % 3 == 2
        (7, ROLE_GEN),    // position 6, 6 % 3 == 0
        (3, ROLE_MED),    // position 7, 7 % 3 == 1
        (6, ROLE_MAN),    // position 8, 8 % 3 == 2
        (1, ROLE_GEN),    // position 9, 9 % 3 == 0
    ];
    for (residue, role) in expected_roles {
        assert_eq!(
            classify(residue).role_rank,
            role,
            "residue {residue}'s extended role should be {role}"
        );
    }
}

/// Residue 0 is outside `(Z/11Z)*` (Lagrange's theorem, which the
/// cascade-periodicity theorem invokes, only covers the multiplicative
/// group) and gets its own sentinel class, not silently folded into one
/// of the three proven anchors.
#[test]
fn residue_zero_is_its_own_sentinel_class() {
    for candidate in [0u32, 11, 22, 110] {
        let classification = classify(candidate);
        assert_eq!(classification.role_rank, ROLE_ZERO);
        assert_eq!(classification.cascade_position, 10);
    }
}

/// Classification depends only on the residue mod 11 — periodic by
/// construction.
#[test]
fn classification_is_periodic_mod_11() {
    for base in 0u32..11 {
        let a = classify(base);
        let b = classify(base + 11);
        let c = classify(base + 110);
        assert_eq!(a, b);
        assert_eq!(a, c);
    }
}

/// Every nonzero residue gets a real orbit position (< the residue-0
/// sentinel of 10) — the classification is total and the orbit is a
/// complete permutation of the 10 nonzero residues (Theorem M4:
/// maximal period 10 means every nonzero residue appears exactly once).
#[test]
fn every_nonzero_residue_gets_a_real_cascade_position() {
    for residue in 1u32..11 {
        assert!(
            classify(residue).cascade_position < 10,
            "residue {residue} should have a real (non-sentinel) cascade position"
        );
    }
}

// ------------------------------------------------------- registry tests --

/// The `msa-structured-selector/1` record is reachable through the
/// #602 registry by (id, version), exactly like `r4-route-attention/1`.
#[test]
fn registry_resolves_msa_structured_selector_v1() {
    let record = operator_spec(
        AttentionOperatorSpec::MSA_STRUCTURED_SELECTOR_ID,
        AttentionOperatorSpec::MSA_STRUCTURED_SELECTOR_VERSION,
    )
    .expect("registered MSA-structured-selector target operator");
    assert_eq!(record, AttentionOperatorSpec::msa_structured_selector_v1());
    assert_eq!(record.id, "msa-structured-selector");
    assert_eq!(record.version, 1);
}

/// The declared identity digest is a real blake3 digest and is stable
/// across repeated construction (canonical_bytes is deterministic).
#[test]
fn registry_digest_is_stable() {
    let a = AttentionOperatorSpec::msa_structured_selector_v1();
    let b = AttentionOperatorSpec::msa_structured_selector_v1();
    assert_eq!(a.implementation_digest, b.implementation_digest);
    assert!(a.implementation_digest.starts_with("blake3:"));
}

// ------------------------------------------------------- reference/witness --

/// Deterministic synthetic fixture: ramp candidate ids and contributions
/// (no RNG), spanning multiple residue classes.
fn synthetic_fixture(candidate_count: u32, top_m: usize) -> (Vec<u32>, Vec<ScoreQ>) {
    let candidate_ids: Vec<u32> = (0..candidate_count).map(|i| i * 7 + 3).collect();
    let contributions: Vec<ScoreQ> = (0..candidate_count)
        .map(|i| ScoreQ::from_raw((i as i32) * 100 - 500))
        .collect();
    let _ = top_m;
    (candidate_ids, contributions)
}

#[test]
fn reference_run_is_deterministic_across_repeats() {
    let (candidate_ids, contributions) = synthetic_fixture(20, 4);
    let reference =
        MsaSelectorReference::new(candidate_ids, contributions, 4).expect("valid instance");
    let (records_a, witness_a) = reference.run(3);
    let (records_b, witness_b) = reference.run(3);
    assert_eq!(records_a, records_b);
    assert_eq!(witness_a, witness_b);
    // Query-independence: every step within one run is identical too.
    assert_eq!(witness_a.steps[0], witness_a.steps[1]);
    assert_eq!(witness_a.steps[1], witness_a.steps[2]);
}

#[test]
fn reference_selection_is_top_m_wide_and_sorted() {
    let (candidate_ids, contributions) = synthetic_fixture(15, 5);
    let reference =
        MsaSelectorReference::new(candidate_ids, contributions, 5).expect("valid instance");
    let (records, _witness) = reference.run(1);
    let record = &records[0];
    assert_eq!(record.selected.len(), 5);
    let mut previous: Option<(u8, u8, u32)> = None;
    for selection in &record.selected {
        let key = (
            selection.role_rank,
            selection.cascade_position,
            selection.candidate,
        );
        if let Some(previous_key) = previous {
            assert!(previous_key < key, "selection must be strictly ordered");
        }
        previous = Some(key);
    }
}

#[test]
fn witness_census_matches_closed_form() {
    let (candidate_ids, contributions) = synthetic_fixture(30, 6);
    let candidate_count = candidate_ids.len() as u32;
    let reference =
        MsaSelectorReference::new(candidate_ids, contributions, 6).expect("valid instance");
    let (_records, witness) = reference.run(4);
    assert_eq!(
        witness.census,
        expected_msa_selector_census(candidate_count, 6, 4)
    );
}

#[test]
fn witness_replays_successfully() {
    let (candidate_ids, contributions) = synthetic_fixture(25, 4);
    let reference = MsaSelectorReference::new(candidate_ids.clone(), contributions.clone(), 4)
        .expect("valid instance");
    let (_records, witness) = reference.run(2);
    let outcome = replay_msa_selector_witness(&candidate_ids, &contributions, 4, 2, &witness);
    assert_eq!(
        outcome, None,
        "a genuine witness must replay clean: {outcome:?}"
    );
}

#[test]
fn replay_rejects_a_tampered_aggregate() {
    let (candidate_ids, contributions) = synthetic_fixture(12, 3);
    let reference = MsaSelectorReference::new(candidate_ids.clone(), contributions.clone(), 3)
        .expect("valid instance");
    let (_records, mut witness) = reference.run(1);
    witness.steps[0].aggregate_raw = witness.steps[0].aggregate_raw.wrapping_add(1);
    let outcome = replay_msa_selector_witness(&candidate_ids, &contributions, 3, 1, &witness);
    assert!(outcome.is_some(), "a tampered aggregate must fail replay");
}

#[test]
fn replay_rejects_a_wrong_operator_version() {
    let (candidate_ids, contributions) = synthetic_fixture(12, 3);
    let reference = MsaSelectorReference::new(candidate_ids.clone(), contributions.clone(), 3)
        .expect("valid instance");
    let (_records, mut witness) = reference.run(1);
    witness.operator_version = 99;
    let outcome = replay_msa_selector_witness(&candidate_ids, &contributions, 3, 1, &witness);
    assert!(
        outcome.is_some(),
        "a wrong operator version must fail replay"
    );
}

#[test]
fn replay_rejects_a_reordered_selection() {
    let (candidate_ids, contributions) = synthetic_fixture(12, 3);
    let reference = MsaSelectorReference::new(candidate_ids.clone(), contributions.clone(), 3)
        .expect("valid instance");
    let (_records, mut witness) = reference.run(1);
    witness.steps[0].selected.swap(0, 1);
    let outcome = replay_msa_selector_witness(&candidate_ids, &contributions, 3, 1, &witness);
    assert!(
        outcome.is_some(),
        "an out-of-order selection must fail replay"
    );
}

/// `top_m == candidate_count` (select everything) is a legal boundary.
#[test]
fn top_m_equal_to_candidate_count_selects_every_candidate() {
    let (candidate_ids, contributions) = synthetic_fixture(6, 6);
    let reference =
        MsaSelectorReference::new(candidate_ids, contributions, 6).expect("valid instance");
    let (records, _witness) = reference.run(1);
    assert_eq!(records[0].selected.len(), 6);
}

/// Malformed instances are refused before any reference construction.
#[test]
fn malformed_instances_are_refused() {
    assert!(MsaSelectorReference::new(vec![], vec![], 1).is_none());
    assert!(MsaSelectorReference::new(vec![1, 2], vec![ScoreQ::ZERO], 1).is_none());
    assert!(MsaSelectorReference::new(vec![1, 2], vec![ScoreQ::ZERO, ScoreQ::ZERO], 0).is_none());
    assert!(MsaSelectorReference::new(vec![1, 2], vec![ScoreQ::ZERO, ScoreQ::ZERO], 3).is_none());
}
