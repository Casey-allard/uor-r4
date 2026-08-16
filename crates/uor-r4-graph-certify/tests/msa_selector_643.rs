//! `MsaStructuredSelectorV1` grounding, reference, and witness tests
//! (#643).
//!
//! The operator under test is DORMANT (`msa-structured-selector-dormant`
//! in `model/ledger.toml`): these tests construct it directly through
//! `uor-r4-graph-certify::msa_selector` (scalar reference) and
//! `uor-r4-graph-runtime::msa_selector` (packed R4G1 lowering, via the
//! certify crate's `run_packed`); no serving path is exercised or
//! changed. The canonical instance substrate lives in
//! `uor-r4-graph-format::msa_selector`.
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
//! The packed R4G1 lowering now exists (`uor-r4-graph-runtime::msa_selector`,
//! differentially tested against the reference below), which was the
//! precondition this comment originally named. Running the actual A/B
//! still requires wiring `msa-structured-selector/1` into the same
//! held-out evaluation loop `r4-route-attention/1` uses (a harness
//! adapter) — that wiring remains OUT OF SCOPE for this slice and is
//! the next concrete action once it lands.

use uor_r4_graph_certify::msa_selector::{
    classify, expected_msa_selector_census, replay_msa_selector_witness, run_packed,
    MsaClassification, MsaSelectorOpCensus, MsaSelectorReference, ROLE_GEN, ROLE_MAN, ROLE_MED,
    ROLE_ZERO,
};
use uor_r4_graph_format::{
    FormatError, MsaSelectorView, NotAProduct, ObjectKind, ScoreQ, MSA_MAX_CANDIDATES,
    MSA_MAX_TOP_M,
};
use uor_r4_graph_runtime::msa_selector::{msa_selector_step, MsaSelectorState};
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

/// Malformed instances are refused before any reference construction,
/// on the sanctioned R5 surface (the same `NotAProduct` shape
/// `run_packed` and `MsaSelectorView::parse` refuse with).
#[test]
fn malformed_instances_are_refused() {
    assert!(MsaSelectorReference::new(vec![], vec![], 1).is_err());
    assert!(MsaSelectorReference::new(vec![1, 2], vec![ScoreQ::ZERO], 1).is_err());
    assert!(MsaSelectorReference::new(vec![1, 2], vec![ScoreQ::ZERO, ScoreQ::ZERO], 0).is_err());
    assert!(MsaSelectorReference::new(vec![1, 2], vec![ScoreQ::ZERO, ScoreQ::ZERO], 3).is_err());
    assert!(matches!(
        MsaSelectorReference::new(vec![], vec![], 1),
        Err(NotAProduct {
            object: ObjectKind::MsaSelectorInstance,
            reason: FormatError::MsaCandidateCountOutOfBounds {
                declared: 0,
                max: 64
            },
        })
    ));
}

// -------------------------------------------------- packed differential --

/// Deterministic synthetic fixture (no RNG): ramp candidate ids and
/// contributions, spanning multiple residue classes — the packed-lowering
/// counterpart of `synthetic_fixture` above, parameterized identically
/// so the same shapes drive both the reference-only tests and the
/// differential tests below.
fn packed_fixture(candidate_count: u32, top_m: u16) -> (Vec<u32>, Vec<ScoreQ>) {
    let candidate_ids: Vec<u32> = (0..candidate_count).map(|i| i * 13 + 5).collect();
    let contributions: Vec<ScoreQ> = (0..candidate_count)
        .map(|i| ScoreQ::from_raw(((i * 917) % 40_000) as i32 - 20_000))
        .collect();
    let _ = top_m;
    (candidate_ids, contributions)
}

/// Reference and packed paths agree bit-for-bit on selections,
/// aggregates, the census, and the whole witness, over the SAME
/// canonical instance bytes.
#[test]
fn reference_and_packed_agree_bit_for_bit_on_the_pinned_fixture() {
    let (candidate_ids, contributions) = packed_fixture(20, 4);
    let reference = MsaSelectorReference::new(candidate_ids.clone(), contributions.clone(), 4)
        .expect("fixture is a valid instance");
    let (reference_records, reference_witness) = reference.run(5);
    let (packed_records, packed_witness) =
        run_packed(&candidate_ids, &contributions, 4, 5).expect("packed run succeeds");

    assert_eq!(
        reference_records, packed_records,
        "selections and aggregates must agree bit-for-bit"
    );
    assert_eq!(
        reference_witness, packed_witness,
        "the whole witness must agree bit-for-bit"
    );
    let mut reference_bytes = Vec::new();
    ciborium::into_writer(&reference_witness, &mut reference_bytes).expect("witness serializes");
    let mut packed_bytes = Vec::new();
    ciborium::into_writer(&packed_witness, &mut packed_bytes).expect("witness serializes");
    assert_eq!(reference_bytes, packed_bytes);

    assert_eq!(
        reference_witness.census,
        expected_msa_selector_census(20, 4, 5)
    );
    assert_eq!(
        replay_msa_selector_witness(&candidate_ids, &contributions, 4, 5, &reference_witness),
        None,
        "the reference witness replays"
    );
    assert_eq!(
        replay_msa_selector_witness(&candidate_ids, &contributions, 4, 5, &packed_witness),
        None,
        "the packed witness replays too — it is bit-identical to the reference witness"
    );
}

/// Differential agreement holds across a deterministic grid of shapes
/// (every N/M corner: N = 1, N = M, N at the cap, M at the cap).
#[test]
fn reference_and_packed_agree_across_the_shape_grid() {
    for (candidate_count, top_m, steps) in [
        (1u32, 1u16, 2usize),
        (2, 2, 2),
        (5, 1, 3),
        (8, 8, 2),
        (MSA_MAX_CANDIDATES as u32, MSA_MAX_TOP_M as u16, 2),
        (MSA_MAX_CANDIDATES as u32, 1, 1),
        (9, 4, 4),
    ] {
        let (candidate_ids, contributions) = packed_fixture(candidate_count, top_m);
        let reference =
            MsaSelectorReference::new(candidate_ids.clone(), contributions.clone(), top_m as usize)
                .expect("grid fixture is a valid instance");
        let (reference_records, reference_witness) = reference.run(steps);
        let (packed_records, packed_witness) =
            run_packed(&candidate_ids, &contributions, top_m, steps).expect("packed run succeeds");
        assert_eq!(
            reference_records, packed_records,
            "grid shape N={candidate_count} M={top_m}"
        );
        assert_eq!(
            reference_witness, packed_witness,
            "grid shape N={candidate_count} M={top_m}"
        );
        assert_eq!(
            reference_witness.census,
            expected_msa_selector_census(candidate_count, top_m, steps),
            "closed-form census, grid shape N={candidate_count} M={top_m}"
        );
    }
}

/// The caller-owned packed state is reusable across steps and instances
/// (the epoch stamp advances once per step), and every step examines
/// exactly the declared `top_m` slots.
#[test]
fn packed_state_epoch_advances_and_state_is_reusable() {
    let (candidate_ids, contributions) = packed_fixture(12, 3);
    let instance =
        uor_r4_graph_format::build_msa_selector_instance(&candidate_ids, &contributions, 3)
            .expect("fixture builds");
    let view = MsaSelectorView::parse(&instance).expect("parses");
    let mut state = MsaSelectorState::new();
    let mut census = MsaSelectorOpCensus::default();
    assert_eq!(state.epoch(), 0);
    for step in 1..=4u64 {
        let _ = msa_selector_step(&view, &mut state, &mut census);
        assert_eq!(state.epoch(), step, "epoch stamps each step");
        assert_eq!(state.selected_len(), 3);
        assert!(state.selected(3).is_none());
    }
    // Reuse the same state on a different instance shape.
    let (other_ids, other_contributions) = packed_fixture(5, 1);
    let other_instance =
        uor_r4_graph_format::build_msa_selector_instance(&other_ids, &other_contributions, 1)
            .expect("fixture builds");
    let other_view = MsaSelectorView::parse(&other_instance).expect("parses");
    let _ = msa_selector_step(&other_view, &mut state, &mut census);
    assert_eq!(state.epoch(), 5);
    assert_eq!(state.selected_len(), 1);
}

/// Deterministic tie-breaking: candidates whose ids share a residue mod
/// 11 (so identical role_rank/cascade_position) select the LOWEST
/// index first, both on the reference and the packed path.
#[test]
fn property_ties_break_to_the_lowest_index_and_runs_are_deterministic() {
    // Every candidate id congruent to 2 mod 11 (role Gen, cascade
    // position 0) — every classification is identical, so ties are the
    // only thing that can order the selection.
    let candidate_ids: Vec<u32> = (0..10).map(|i| 2 + i * 11).collect();
    let contributions: Vec<ScoreQ> = (0..10).map(|i| ScoreQ::from_raw(i * 1000)).collect();
    let (records, witness) =
        run_packed(&candidate_ids, &contributions, 5, 1).expect("tie fixture runs");
    let selected: Vec<u32> = records[0]
        .selected
        .iter()
        .map(|selection| selection.candidate)
        .collect();
    assert_eq!(
        selected,
        vec![0, 1, 2, 3, 4],
        "equal classification selects the lowest indices, ascending"
    );
    assert_eq!(records[0].aggregate.raw(), 1000 + 2000 + 3000 + 4000);

    let (again_records, again_witness) =
        run_packed(&candidate_ids, &contributions, 5, 1).expect("second run");
    assert_eq!(records, again_records);
    assert_eq!(witness, again_witness);
    let reference =
        MsaSelectorReference::new(candidate_ids, contributions, 5).expect("tie instance is valid");
    let (reference_records, reference_witness) = reference.run(1);
    assert_eq!(records, reference_records);
    assert_eq!(witness, reference_witness);
}

/// `ScoreQ` saturation: contributions at the rails saturate instead of
/// wrapping, in the pinned selection-order fold, identically on both
/// paths.
#[test]
fn property_scoreq_aggregation_saturates() {
    // Three candidates, each in a DISTINCT role class so the selection
    // order is fixed by classification, not by tie-breaking: id 2 (Gen,
    // position 0), id 4 (Med, position 1), id 8 (Man, position 2).
    let candidate_ids = vec![2u32, 4, 8];
    let contributions = vec![
        ScoreQ::from_raw(i32::MAX),
        ScoreQ::from_raw(i32::MAX),
        ScoreQ::from_raw(i32::MIN),
    ];
    let (records, _) = run_packed(&candidate_ids, &contributions, 3, 1).expect("run succeeds");
    // Fold order: id 2 (Gen) first, then id 4 (Med), then id 8 (Man).
    // MAX +sat MAX = MAX; MAX +sat MIN = -1.
    assert_eq!(records[0].aggregate.raw(), -1);
    let reference = MsaSelectorReference::new(candidate_ids, contributions, 3)
        .expect("saturation fixture is valid");
    let mut census = MsaSelectorOpCensus::default();
    let reference_record = reference.reference_step(&mut census);
    assert_eq!(reference_record.aggregate.raw(), -1);
}

/// Hard caps refuse with the sanctioned error naming the observed value
/// and the bound (R5).
#[test]
fn property_caps_refuse_with_sanctioned_errors() {
    let ids_over_cap: Vec<u32> = (0..(MSA_MAX_CANDIDATES as u32 + 1)).collect();
    let contributions_over_cap = vec![ScoreQ::ZERO; MSA_MAX_CANDIDATES + 1];
    assert!(matches!(
        run_packed(&ids_over_cap, &contributions_over_cap, 1, 1),
        Err(NotAProduct {
            object: ObjectKind::MsaSelectorInstance,
            reason: FormatError::MsaCandidateCountOutOfBounds {
                declared,
                max: 64,
            },
        }) if declared == MSA_MAX_CANDIDATES as u32 + 1
    ));
    let ids = vec![2u32; 10];
    let contributions = vec![ScoreQ::ZERO; 10];
    assert!(matches!(
        run_packed(&ids, &contributions, MSA_MAX_TOP_M as u16 + 1, 1),
        Err(NotAProduct {
            reason: FormatError::MsaTopMOutOfBounds {
                declared: 9,
                max: 8,
            },
            ..
        })
    ));
    let ids = vec![2u32; 2];
    let contributions = vec![ScoreQ::ZERO; 2];
    assert!(matches!(
        run_packed(&ids, &contributions, 3, 1),
        Err(NotAProduct {
            reason: FormatError::MsaTopMOutOfBounds {
                declared: 3,
                max: 2,
            },
            ..
        })
    ));
}

// -------------------------------------------------------- source scans --

/// Comment- and string-stripped source scan for value `*` `/` `%`
/// operators and float types on the packed lowering — the by-
/// construction zero-float/zero-multiply/zero-divide/zero-modulo claim,
/// machine-checked on every test run. This mirrors
/// `route_attention_604.rs`'s equivalent scan and the P-4 extension in
/// `uor-r4-core::transformerless::mod.rs` (which also covers this file
/// as a contract-owned graph-runtime module).
fn scan_source_for_forbidden_ops(source: &str) -> Vec<String> {
    let mut offenders = Vec::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let mut stripped = String::with_capacity(raw_line.len());
        let mut in_string = false;
        let mut escaped = false;
        for ch in raw_line.chars() {
            if escaped {
                escaped = false;
                continue;
            }
            if in_string {
                if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_string = false;
                }
                continue;
            }
            if ch == '"' {
                in_string = true;
                continue;
            }
            stripped.push(ch);
        }
        let code = match stripped.find("//") {
            Some(comment_start) => &stripped[..comment_start],
            None => stripped.as_str(),
        };
        if code.trim().is_empty() {
            continue;
        }
        if code.contains("f32") || code.contains("f64") {
            offenders.push(format!("line {}: float type: {}", line_number + 1, code));
            continue;
        }
        for needle in [
            "wrapping_mul(",
            "saturating_mul(",
            "checked_mul(",
            ".mul(",
            "wrapping_div(",
            "saturating_div(",
            "checked_div(",
            ".div(",
            "wrapping_rem(",
            "saturating_rem(",
            "checked_rem(",
            ".rem(",
        ] {
            if code.contains(needle) {
                offenders.push(format!("line {}: {}", line_number + 1, code));
            }
        }
        let bytes = code.as_bytes();
        for (index, &byte) in bytes.iter().enumerate() {
            if byte != b'*' && byte != b'/' && byte != b'%' {
                continue;
            }
            let operand = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b')' || c == b']';
            let operand_right = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b'(';
            let prev = if index >= 2 && bytes[index - 1] == b' ' {
                bytes[index - 2]
            } else if index >= 1 {
                bytes[index - 1]
            } else {
                b' '
            };
            let next = if index + 2 < bytes.len() && bytes[index + 1] == b' ' {
                bytes[index + 2]
            } else if index + 1 < bytes.len() {
                bytes[index + 1]
            } else {
                b' '
            };
            if operand(prev) && operand_right(next) {
                offenders.push(format!("line {}: {}", line_number + 1, code));
                break;
            }
        }
    }
    offenders
}

/// The packed lowering carries no float type and no value
/// multiplication/division/modulo. (The P-4 extension scan in
/// `uor-r4-core::transformerless::mod.rs` also covers this file for
/// mul/div/mod as a contract-owned graph-runtime module; the float scan
/// here is additional.)
#[test]
fn packed_source_is_integer_only_by_construction() {
    let source = include_str!("../../uor-r4-graph-runtime/src/msa_selector.rs");
    let offenders = scan_source_for_forbidden_ops(source);
    assert!(
        offenders.is_empty(),
        "forbidden arithmetic in the #643 packed lowering:\n{}",
        offenders.join("\n")
    );
}

/// The format-crate substrate's `MsaSelectorView` (the packed lowering's
/// entire read path — [`MsaSelectorView::parse`] and its accessors) is
/// float-free and integer-only. `#[cfg(feature = "alloc")]`-gated
/// BUILD-time code above it (`cascade_position`, `role_rank`,
/// `classify_at_build_time`, `build_msa_selector_instance`) legitimately
/// computes `candidate_id % 11` and a capacity-sizing multiply — the
/// module docs explain why this is the one place the crate computes a
/// modulus, and it is never reached from the packed lowering's read
/// path (`uor-r4-graph-runtime`, P-4-scanned separately). This test
/// scans only from `MsaSelectorOpCensus` onward — the census, the view,
/// and its `parse`/accessors — excluding the alloc-gated build-time
/// helpers above it.
#[test]
fn view_and_census_source_is_integer_only_by_construction() {
    let source = include_str!("../../uor-r4-graph-format/src/msa_selector.rs");
    let view_onward = source
        .split("pub struct MsaSelectorOpCensus")
        .nth(1)
        .expect("module defines MsaSelectorOpCensus");
    // Stop before `build_msa_selector_instance` (alloc-gated build-time
    // code with the documented capacity-sizing multiply) and before the
    // `#[cfg(test)]` module (ordinary test fixture data).
    let view_only = view_onward
        .split("pub fn build_msa_selector_instance")
        .next()
        .expect("module defines build_msa_selector_instance after the view");
    let non_test = view_only
        .split("#[cfg(test)]")
        .next()
        .expect("module has a body after the census struct");
    let offenders = scan_source_for_forbidden_ops(non_test);
    assert!(
        offenders.is_empty(),
        "forbidden arithmetic in the #643 view/census substrate:\n{}",
        offenders.join("\n")
    );
}

/// The #602 registry entry and the #643 substrate agree on the operator
/// identity, and the registry resolves it with the truthful deployed
/// integer class.
#[test]
fn registry_identity_matches_the_substrate() {
    use uor_r4_graph_format::{MSA_SELECTOR_OPERATOR_ID, MSA_SELECTOR_OPERATOR_VERSION};
    assert_eq!(
        AttentionOperatorSpec::MSA_STRUCTURED_SELECTOR_ID,
        MSA_SELECTOR_OPERATOR_ID
    );
    assert_eq!(
        AttentionOperatorSpec::MSA_STRUCTURED_SELECTOR_VERSION,
        MSA_SELECTOR_OPERATOR_VERSION
    );
    let record = operator_spec(MSA_SELECTOR_OPERATOR_ID, MSA_SELECTOR_OPERATOR_VERSION)
        .expect("the target operator is registered");
    assert_eq!(
        record.permitted_operation_class,
        "deployed-integer-table-read-compare-add-no-runtime-modulo"
    );
    assert!(record.implementation_digest.starts_with("blake3:"));
    let (candidate_ids, contributions) = packed_fixture(10, 3);
    let (_, witness) = run_packed(&candidate_ids, &contributions, 3, 1).expect("run succeeds");
    assert_eq!(witness.operator_id, record.id);
    assert_eq!(witness.operator_version, record.version);
}
