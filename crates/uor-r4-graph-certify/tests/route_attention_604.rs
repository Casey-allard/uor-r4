//! `R4RouteAttentionV1` differential + witness + property tests (#604).
//!
//! The operator under test is DORMANT (`r4-route-attention-dormant` in
//! `model/ledger.toml`): these tests construct it directly; no serving
//! path is exercised or changed. The reference specification lives in
//! `uor-r4-graph-certify::route_attention`, the packed lowering in
//! `uor-r4-graph-runtime::route_attention`, the canonical instance
//! substrate in `uor-r4-graph-format::route_attention`.
//!
//! Fixture discipline: every fixture below is deterministic ramp
//! arithmetic — no RNG, no clock, no iteration-order dependence — so
//! the pinned expectations (selections, distances, aggregates, digests)
//! are stable bytes-for-bytes across runs and machines.

use uor_r4_graph_certify::route_attention::{
    digest_string, expected_route_census, replay_route_witness, run_packed,
    RouteAttentionReference, RouteAttentionWitness, RouteReplayError, RouteSelection,
    ROUTE_WITNESS_FORMAT,
};
use uor_r4_graph_format::route_attention::{
    build_route_attention_instance, RouteAttentionView, RouteOpCensus, ROUTE_CODE_BYTES,
    ROUTE_MAX_TOP_M,
};
use uor_r4_graph_format::{FormatError, NotAProduct, ObjectKind, ScoreQ};
use uor_r4_graph_runtime::route_attention::{route_attention_step, RouteState};

// ------------------------------------------------------------ fixtures --

/// Deterministic synthetic fixture (no RNG): a structured mask with a
/// moving hole, ramp candidate codes and contributions, ramp queries.
fn synthetic_route_fixture(
    variant: usize,
    candidate_count: usize,
    top_m: u32,
    query_count: usize,
) -> (Vec<u8>, Vec<[u8; ROUTE_CODE_BYTES]>) {
    let mut mask = [0u8; ROUTE_CODE_BYTES];
    for (position, byte) in mask.iter_mut().enumerate() {
        *byte = match (position + variant) % 5 {
            0 => 0x7f,
            1 => 0xff,
            2 => 0xfe,
            3 => 0xf7,
            _ => 0xbf,
        };
    }
    let mut codes = Vec::with_capacity(candidate_count);
    let mut contributions = Vec::with_capacity(candidate_count);
    for candidate in 0..candidate_count {
        let mut code = [0u8; ROUTE_CODE_BYTES];
        for (position, byte) in code.iter_mut().enumerate() {
            *byte = ((position * 31 + candidate * 97 + variant * 13) % 251) as u8;
        }
        codes.push(code);
        let raw = ((candidate * 7919 + variant * 101) % 40_000) as i32 - 20_000;
        contributions.push(ScoreQ::from_raw(raw));
    }
    let bytes = build_route_attention_instance(&mask, &codes, &contributions, top_m)
        .expect("synthetic fixture parameters are within the declared bounds");
    let mut queries = Vec::with_capacity(query_count);
    for step in 0..query_count {
        let mut query = [0u8; ROUTE_CODE_BYTES];
        for (position, byte) in query.iter_mut().enumerate() {
            *byte = ((position * 17 + step * 53 + variant * 29) % 249) as u8;
        }
        queries.push(query);
    }
    (bytes, queries)
}

/// The one PINNED fixture of the differential/witness suite: variant 0,
/// N = 16 candidates, M = 3, 4 steps.
fn pinned_fixture() -> (Vec<u8>, Vec<[u8; ROUTE_CODE_BYTES]>) {
    synthetic_route_fixture(0, 16, 3, 4)
}

// ------------------------------------------------- differential + pins --

/// Reference and packed paths agree bit-for-bit on selections,
/// distances, aggregates, the census, and the whole witness — and the
/// pinned fixture's first/last steps match hard-coded expectations, so
/// a semantics drift in EITHER path fails loudly here.
#[test]
fn reference_and_packed_agree_bit_for_bit_on_the_pinned_fixture() {
    let (instance, queries) = pinned_fixture();
    let reference =
        RouteAttentionReference::from_instance_bytes(&instance).expect("fixture instance parses");
    let (reference_records, reference_witness) = reference.run(&queries);
    let (packed_records, packed_witness) =
        run_packed(&instance, &queries).expect("packed run succeeds");

    assert_eq!(
        reference_records, packed_records,
        "selections, distances, and aggregates must agree bit-for-bit"
    );
    assert_eq!(
        reference_witness, packed_witness,
        "the whole witness must agree bit-for-bit"
    );
    // Byte-level agreement of the serialized witnesses (serde/ciborium).
    let mut reference_bytes = Vec::new();
    ciborium::into_writer(&reference_witness, &mut reference_bytes).expect("witness serializes");
    let mut packed_bytes = Vec::new();
    ciborium::into_writer(&packed_witness, &mut packed_bytes).expect("witness serializes");
    assert_eq!(reference_bytes, packed_bytes);

    // Census equals its closed form (and is data-independent of the
    // query values by construction).
    assert_eq!(
        reference_witness.census,
        expected_route_census(16, 3, queries.len())
    );

    // Pinned expectations (computed once from the deterministic
    // fixture and hard-coded; any drift is a semantics change and must
    // arrive as a new operator version).
    let step0: Vec<(u32, u32)> = reference_records[0]
        .selected
        .iter()
        .map(|selection| (selection.candidate, selection.distance))
        .collect();
    let step3: Vec<(u32, u32)> = reference_records[3]
        .selected
        .iter()
        .map(|selection| (selection.candidate, selection.distance))
        .collect();
    assert_eq!(step0, PINNED_STEP0_SELECTED.to_vec(), "step 0 selection");
    assert_eq!(step3, PINNED_STEP3_SELECTED.to_vec(), "step 3 selection");
    assert_eq!(
        reference_records[0].aggregate.raw(),
        PINNED_STEP0_AGGREGATE_RAW,
        "step 0 aggregate"
    );
    assert_eq!(
        reference_records[3].aggregate.raw(),
        PINNED_STEP3_AGGREGATE_RAW,
        "step 3 aggregate"
    );
    assert_eq!(
        reference_witness.instance_digest, PINNED_INSTANCE_DIGEST,
        "canonical instance bytes are pinned"
    );
    assert_eq!(
        reference_witness.inputs_digest, PINNED_INPUTS_DIGEST,
        "inputs digest is pinned"
    );
}

/// Pinned values of the variant-0 fixture (see the test above).
const PINNED_STEP0_SELECTED: [(u32, u32); 3] = [(3, 113), (0, 117), (12, 118)];
const PINNED_STEP3_SELECTED: [(u32, u32); 3] = [(4, 109), (7, 115), (10, 120)];
const PINNED_STEP0_AGGREGATE_RAW: i32 = -21_215;
const PINNED_STEP3_AGGREGATE_RAW: i32 = 26_299;
const PINNED_INSTANCE_DIGEST: &str =
    "blake3:939d40da65eec1d054ffadc738c727028e3fbda72d710a032648a981455a8993";
const PINNED_INPUTS_DIGEST: &str =
    "blake3:99bd7eb4e07a3488ca702bfdcddd3b23004aee7761878c02651aa16fb4020386";

/// Differential agreement holds across a deterministic grid of shapes
/// (every N/M corner: N = 1, N = M, N at the cap, M at the cap).
#[test]
fn reference_and_packed_agree_across_the_shape_grid() {
    for (variant, candidate_count, top_m, steps) in [
        (1usize, 1usize, 1u32, 3usize),
        (2, 2, 2, 3),
        (3, 5, 1, 2),
        (4, 8, 8, 2),
        (5, 64, 8, 2),
        (6, 64, 1, 1),
        (7, 9, 4, 5),
    ] {
        let (instance, queries) = synthetic_route_fixture(variant, candidate_count, top_m, steps);
        let reference =
            RouteAttentionReference::from_instance_bytes(&instance).expect("grid instance parses");
        let (reference_records, reference_witness) = reference.run(&queries);
        let (packed_records, packed_witness) =
            run_packed(&instance, &queries).expect("packed run succeeds");
        assert_eq!(reference_records, packed_records, "grid shape {variant}");
        assert_eq!(reference_witness, packed_witness, "grid shape {variant}");
        assert_eq!(
            reference_witness.census,
            expected_route_census(candidate_count as u32, top_m as u16, steps),
            "closed-form census, grid shape {variant}"
        );
        assert_eq!(
            replay_route_witness(&instance, &queries, &reference_witness),
            None,
            "witness replays, grid shape {variant}"
        );
    }
}

// ------------------------------------------------------ witness replay --

/// The independent replayer accepts the true witness and rejects every
/// tampered field with the named failure.
#[test]
fn witness_replay_accepts_truth_and_rejects_tampering() {
    let (instance, queries) = pinned_fixture();
    let reference =
        RouteAttentionReference::from_instance_bytes(&instance).expect("fixture instance parses");
    let (_, witness) = reference.run(&queries);
    assert_eq!(
        replay_route_witness(&instance, &queries, &witness),
        None,
        "true witness replays"
    );

    // Format / identity tampering.
    let mut tampered = witness.clone();
    tampered.format = "uor-r4-route-attention-witness/9".to_owned();
    assert_eq!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::FormatTagMismatch)
    );
    let mut tampered = witness.clone();
    tampered.operator_version = 2;
    assert_eq!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::OperatorMismatch)
    );
    let mut tampered = witness.clone();
    tampered.instance_digest = digest_string(&[0u8; 32]);
    assert_eq!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::InstanceDigestMismatch)
    );
    let mut tampered = witness.clone();
    tampered.inputs_digest = digest_string(&[0u8; 32]);
    assert_eq!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::InputsDigestMismatch)
    );

    // Step-shape tampering.
    let mut tampered = witness.clone();
    tampered.steps.pop();
    assert_eq!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::StepCountMismatch {
            declared: 3,
            actual: 4
        })
    );
    let mut tampered = witness.clone();
    tampered.steps[0].selected.pop();
    assert_eq!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::SelectionWidthMismatch { step: 0 })
    );

    // Selection tampering.
    let mut tampered = witness.clone();
    tampered.steps[1].selected[0].distance += 1;
    assert_eq!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::DistanceMismatch { step: 1, slot: 0 })
    );
    let mut tampered = witness.clone();
    tampered.steps[1].selected[0].candidate = 999;
    assert_eq!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::CandidateOutOfRange { step: 1, slot: 0 })
    );
    let mut tampered = witness.clone();
    tampered.steps[2].selected.swap(0, 1);
    assert_eq!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::SelectionOrderViolation { step: 2, slot: 1 })
    );
    // Replace the WORST selected slot with a truthful record of a
    // strictly worse candidate: every recorded entry stays truthful and
    // ordered, but a genuinely better candidate is now unselected —
    // only the optimality check can catch this.
    let mut tampered = witness.clone();
    {
        let view = RouteAttentionView::parse(&instance).expect("parses");
        // All candidate distances for step 0, computed independently of
        // the operator (test-local masked popcount).
        let mask = view.mask();
        let mut pairs: Vec<(u32, u32)> = Vec::new();
        for (candidate, code) in view.codes().chunks_exact(ROUTE_CODE_BYTES).enumerate() {
            let mut distance = 0u32;
            for position in 0..ROUTE_CODE_BYTES {
                distance += ((queries[0][position] ^ code[position]) & mask[position]).count_ones();
            }
            pairs.push((distance, candidate as u32));
        }
        pairs.sort_unstable();
        // The 4th-best candidate is strictly outside the true top-3.
        let (worse_distance, worse_candidate) = pairs[3];
        let step = &mut tampered.steps[0];
        step.selected[2] = RouteSelection {
            candidate: worse_candidate,
            distance: worse_distance,
        };
        step.selected.sort_by_key(|s| (s.distance, s.candidate));
        // Keep the aggregate consistent with the tampered selection so
        // the optimality check (not the aggregate check) must fire.
        let mut aggregate = ScoreQ::ZERO;
        for selection in &step.selected {
            aggregate =
                aggregate.saturating_add(view.contribution(selection.candidate).expect("in range"));
        }
        step.aggregate_raw = aggregate.raw();
    }
    assert!(matches!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::SelectionNotOptimal { step: 0, .. })
    ));

    // Output tampering.
    let mut tampered = witness.clone();
    tampered.steps[3].aggregate_raw += 1;
    assert_eq!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::AggregateMismatch { step: 3 })
    );

    // Census tampering.
    let mut tampered = witness.clone();
    tampered.census.adds += 1;
    assert_eq!(
        replay_route_witness(&instance, &queries, &tampered),
        Some(RouteReplayError::CensusMismatch)
    );

    // Fixture substitution: the same witness replayed against a
    // different instance or different queries fails on the digests.
    let (other_instance, _) = synthetic_route_fixture(9, 16, 3, 4);
    assert_eq!(
        replay_route_witness(&other_instance, &queries, &witness),
        Some(RouteReplayError::InstanceDigestMismatch)
    );
    let mut other_queries = queries.clone();
    other_queries[0][0] ^= 0xff;
    assert_eq!(
        replay_route_witness(&instance, &other_queries, &witness),
        Some(RouteReplayError::InputsDigestMismatch)
    );
}

/// Witness serde round-trip (ciborium — the certify byte format), plus
/// serde-default backfill of partial documents.
#[test]
fn witness_serde_round_trips_and_defaults_fill() {
    let (instance, queries) = pinned_fixture();
    let reference =
        RouteAttentionReference::from_instance_bytes(&instance).expect("fixture instance parses");
    let (_, witness) = reference.run(&queries);
    let mut bytes = Vec::new();
    ciborium::into_writer(&witness, &mut bytes).expect("witness serializes");
    let back: RouteAttentionWitness =
        ciborium::from_reader(bytes.as_slice()).expect("witness deserializes");
    assert_eq!(witness, back);
    // A replayed round-tripped witness still verifies.
    assert_eq!(
        replay_route_witness(&instance, &queries, &back),
        None,
        "round-tripped witness replays"
    );

    // serde(default): a partial document parses with absent fields
    // defaulted (era discipline). Serialize a one-field document and
    // read it back as a full witness.
    #[derive(serde::Serialize)]
    struct PartialWitnessDocument {
        format: String,
    }
    let mut partial_bytes = Vec::new();
    ciborium::into_writer(
        &PartialWitnessDocument {
            format: ROUTE_WITNESS_FORMAT.to_owned(),
        },
        &mut partial_bytes,
    )
    .expect("partial document serializes");
    let partial: RouteAttentionWitness =
        ciborium::from_reader(partial_bytes.as_slice()).expect("defaults fill");
    assert_eq!(partial.format, ROUTE_WITNESS_FORMAT);
    assert_eq!(partial.operator_version, 0);
    assert!(partial.steps.is_empty());
    assert_eq!(partial.census, RouteOpCensus::default());
}

// ------------------------------------------------------ property tests --

/// Masked-out bits are never scored: flipping any bit OUTSIDE the mask
/// (in the query or any candidate code) leaves selections, aggregates,
/// and the whole witness bit-identical; flipping a bit INSIDE the mask
/// moves at least one recorded distance.
#[test]
fn property_mask_is_honored() {
    let (instance, queries) = pinned_fixture();
    let view = RouteAttentionView::parse(&instance).expect("parses");
    let mask: Vec<u8> = view.mask().to_vec();
    let baseline = run_packed(&instance, &queries).expect("baseline run");

    // Outside-mask perturbations: query bits.
    let mut queries_outside = queries.clone();
    let mut perturbed_any = false;
    for (position, byte) in queries_outside[0].iter_mut().enumerate() {
        let hole = !mask[position];
        if hole != 0 {
            *byte ^= hole;
            perturbed_any = true;
        }
    }
    assert!(perturbed_any, "the fixture mask must have holes to test");
    let outside = run_packed(&instance, &queries_outside).expect("outside-mask run");
    assert_eq!(
        baseline.0, outside.0,
        "unmasked query bits must never enter any distance"
    );

    // Outside-mask perturbations: candidate code bits (flip the hole
    // bits of every candidate's code in the instance bytes).
    let mut instance_outside = instance.clone();
    {
        let codes_start = 16 + ROUTE_CODE_BYTES;
        let n = view.candidate_count() as usize;
        for candidate in 0..n {
            for position in 0..ROUTE_CODE_BYTES {
                let hole = !mask[position];
                instance_outside[codes_start + candidate * ROUTE_CODE_BYTES + position] ^= hole;
            }
        }
    }
    let outside_codes = run_packed(&instance_outside, &queries).expect("outside-code run");
    assert_eq!(
        baseline.0, outside_codes.0,
        "unmasked candidate bits must never enter any distance"
    );

    // Inside-mask perturbation: flip one masked bit of the query.
    let mut queries_inside = queries.clone();
    let masked_position = mask
        .iter()
        .position(|&byte| byte != 0)
        .expect("mask has set bits");
    let lowest_set_bit = mask[masked_position] & mask[masked_position].wrapping_neg();
    queries_inside[0][masked_position] ^= lowest_set_bit;
    let inside = run_packed(&instance, &queries_inside).expect("inside-mask run");
    assert_ne!(
        baseline.0[0].selected, inside.0[0].selected,
        "a masked bit flip must move at least one step-0 distance"
    );
}

/// Top-M bounds hold on every step: exactly `top_m` selections, every
/// candidate in range, every distance within the 288-bit ceiling, and
/// the candidate census equals the examined bound.
#[test]
fn property_top_m_and_candidate_bounds_hold() {
    for (variant, candidate_count, top_m) in [(11usize, 7usize, 2u32), (12, 64, 8), (13, 3, 3)] {
        let (instance, queries) = synthetic_route_fixture(variant, candidate_count, top_m, 3);
        let (records, witness) = run_packed(&instance, &queries).expect("run succeeds");
        for record in &records {
            assert_eq!(record.selected.len(), top_m as usize);
            for selection in &record.selected {
                assert!((selection.candidate as usize) < candidate_count);
                assert!(selection.distance <= 288, "distance within the bit width");
            }
        }
        assert_eq!(
            witness.census.candidates_examined,
            (candidate_count * queries.len()) as u64,
            "every step examines exactly N candidates"
        );
    }
}

/// Deterministic tie-breaking: on equal masked distance the LOWEST
/// candidate index wins. All-identical candidate codes make every
/// distance equal, so the selection must be exactly indices 0..M in
/// order; and the whole run is bit-deterministic across repeats.
#[test]
fn property_ties_break_to_the_lowest_index_and_runs_are_deterministic() {
    let mask = [0xffu8; ROUTE_CODE_BYTES];
    let code = [0xa5u8; ROUTE_CODE_BYTES];
    let codes = vec![code; 12];
    let contributions: Vec<ScoreQ> = (0..12).map(|i| ScoreQ::from_raw(i * 1000)).collect();
    let instance = build_route_attention_instance(&mask, &codes, &contributions, 5)
        .expect("tie fixture builds");
    let query = [0x5au8; ROUTE_CODE_BYTES];
    let (records, witness) = run_packed(&instance, &[query]).expect("run succeeds");
    let selected: Vec<u32> = records[0]
        .selected
        .iter()
        .map(|selection| selection.candidate)
        .collect();
    assert_eq!(
        selected,
        vec![0, 1, 2, 3, 4],
        "equal distances select the lowest indices, ascending"
    );
    // All distances equal.
    let distances: Vec<u32> = records[0]
        .selected
        .iter()
        .map(|selection| selection.distance)
        .collect();
    assert!(distances.windows(2).all(|pair| pair[0] == pair[1]));
    // Aggregate = contributions 0+1000+2000+3000+4000.
    assert_eq!(records[0].aggregate.raw(), 10_000);

    // Bit-determinism across repeated runs (fresh state each run).
    let (again_records, again_witness) = run_packed(&instance, &[query]).expect("second run");
    assert_eq!(records, again_records);
    assert_eq!(witness, again_witness);
    // And the reference agrees.
    let reference =
        RouteAttentionReference::from_instance_bytes(&instance).expect("tie instance parses");
    let (reference_records, reference_witness) = reference.run(&[query]);
    assert_eq!(records, reference_records);
    assert_eq!(witness, reference_witness);
}

/// ScoreQ saturation: contributions at the rails saturate instead of
/// wrapping, in the pinned selection-order fold, identically on both
/// paths — and the saturated fold is order-dependent by design, which
/// is why the fold order is part of the specification.
#[test]
fn property_scoreq_aggregation_saturates() {
    let mask = [0xffu8; ROUTE_CODE_BYTES];
    // Three candidates at increasing distance from the all-zero query:
    // index 0 nearest (0 bits), then 1, then 2.
    let mut codes = vec![[0u8; ROUTE_CODE_BYTES]; 3];
    codes[1][0] = 0x01;
    codes[2][0] = 0x03;
    let contributions = vec![
        ScoreQ::from_raw(i32::MAX),
        ScoreQ::from_raw(i32::MAX),
        ScoreQ::from_raw(i32::MIN),
    ];
    let instance = build_route_attention_instance(&mask, &codes, &contributions, 3)
        .expect("saturation fixture builds");
    let query = [0u8; ROUTE_CODE_BYTES];
    let (records, _) = run_packed(&instance, &[query]).expect("run succeeds");
    // Selection order: 0 (dist 0), 1 (dist 1), 2 (dist 2).
    // Fold: 0 +sat MAX = MAX; MAX +sat MAX = MAX; MAX +sat MIN = -1.
    assert_eq!(records[0].aggregate.raw(), -1);
    let reference =
        RouteAttentionReference::from_instance_bytes(&instance).expect("instance parses");
    let mut census = RouteOpCensus::default();
    let reference_record = reference.reference_step(&query, &mut census);
    assert_eq!(reference_record.aggregate.raw(), -1);

    // Pure positive saturation: MAX + MAX stays MAX (no wrap).
    let contributions = vec![
        ScoreQ::from_raw(i32::MAX),
        ScoreQ::from_raw(i32::MAX),
        ScoreQ::from_raw(1),
    ];
    let instance = build_route_attention_instance(&mask, &codes, &contributions, 2)
        .expect("saturation fixture builds");
    let (records, _) = run_packed(&instance, &[query]).expect("run succeeds");
    assert_eq!(records[0].aggregate.raw(), i32::MAX);
}

/// Hard caps refuse with the sanctioned error naming the observed value
/// and the bound (R5: `NotAProduct` is the graph substrate's sanctioned
/// refusal; the reason carries `declared`/`max`).
#[test]
fn property_caps_refuse_with_sanctioned_errors() {
    let mask = [0u8; ROUTE_CODE_BYTES];
    // Candidate cap.
    let codes = vec![[0u8; ROUTE_CODE_BYTES]; 65];
    let contributions = vec![ScoreQ::ZERO; 65];
    assert!(matches!(
        build_route_attention_instance(&mask, &codes, &contributions, 1),
        Err(NotAProduct {
            object: ObjectKind::RouteAttentionInstance,
            reason: FormatError::RouteCandidateCountOutOfBounds {
                declared: 65,
                max: 64,
            },
        })
    ));
    // Selection cap (M > 8) and M > N.
    let codes = vec![[0u8; ROUTE_CODE_BYTES]; 10];
    let contributions = vec![ScoreQ::ZERO; 10];
    assert!(matches!(
        build_route_attention_instance(&mask, &codes, &contributions, ROUTE_MAX_TOP_M as u32 + 1),
        Err(NotAProduct {
            reason: FormatError::RouteTopMOutOfBounds {
                declared: 9,
                max: 8,
            },
            ..
        })
    ));
    let codes = vec![[0u8; ROUTE_CODE_BYTES]; 2];
    let contributions = vec![ScoreQ::ZERO; 2];
    assert!(matches!(
        build_route_attention_instance(&mask, &codes, &contributions, 3),
        Err(NotAProduct {
            reason: FormatError::RouteTopMOutOfBounds {
                declared: 3,
                max: 2,
            },
            ..
        })
    ));
    // Step-time query-width refusal (packed path).
    let (instance, _) = pinned_fixture();
    let view = RouteAttentionView::parse(&instance).expect("parses");
    let mut state = RouteState::new();
    let mut census = RouteOpCensus::default();
    let short_query = [0u8; 8];
    assert!(matches!(
        route_attention_step(&view, &short_query, &mut state, &mut census),
        Err(NotAProduct {
            object: ObjectKind::RouteAttentionStep,
            reason: FormatError::RouteQueryWidthMismatch { actual: 8 },
        })
    ));
}

/// The caller-owned state is reusable across steps and instances (the
/// epoch stamp advances once per step) and the packed kernel touches
/// only the declared selection slots.
#[test]
fn packed_state_epoch_advances_and_state_is_reusable() {
    let (instance, queries) = pinned_fixture();
    let view = RouteAttentionView::parse(&instance).expect("parses");
    let mut state = RouteState::new();
    let mut census = RouteOpCensus::default();
    assert_eq!(state.epoch(), 0);
    for (step, query) in queries.iter().enumerate() {
        let _ = route_attention_step(&view, query, &mut state, &mut census).expect("step succeeds");
        assert_eq!(state.epoch(), (step + 1) as u64, "epoch stamps each step");
        assert_eq!(state.selected_len(), usize::from(view.top_m()));
        assert!(state.selected(usize::from(view.top_m())).is_none());
    }
    // Reuse the same state on a different instance shape.
    let (other_instance, other_queries) = synthetic_route_fixture(21, 5, 1, 1);
    let other_view = RouteAttentionView::parse(&other_instance).expect("parses");
    let _ = route_attention_step(&other_view, &other_queries[0], &mut state, &mut census)
        .expect("step succeeds");
    assert_eq!(state.epoch(), 5);
    assert_eq!(state.selected_len(), 1);
}

// -------------------------------------------------------- source scans --

/// Comment- and string-stripped source scan for value `*` `/` `%`
/// operators and float types: the by-construction zero-float /
/// zero-multiply / zero-divide claim of BOTH #604 implementations,
/// machine-checked on every test run. This mirrors the P-4 scan in
/// `uor-r4-core::transformerless` (which additionally covers the packed
/// file as a contract-owned graph-runtime module, strings included);
/// this copy also strips string literals so data tokens like the
/// witness format tag (`…/1`) are not misread as division.
fn scan_source_for_forbidden_ops(source: &str) -> Vec<String> {
    let mut offenders = Vec::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        // Strip string literals (naive, line-local: the scanned modules
        // keep every string on one line), then line comments.
        let mut stripped = String::with_capacity(raw_line.len());
        let mut in_string = false;
        let mut escaped = false;
        let mut previous = '\0';
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
            previous = ch;
        }
        let _ = previous;
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

/// The certify-side reference module carries no float type and no value
/// multiplication/division/modulo — by construction, machine-checked.
#[test]
fn reference_source_is_integer_only_by_construction() {
    let source = include_str!("../src/route_attention.rs");
    let offenders = scan_source_for_forbidden_ops(source);
    assert!(
        offenders.is_empty(),
        "forbidden arithmetic in the #604 reference module:\n{}",
        offenders.join("\n")
    );
}

/// The packed lowering carries no float type and no value
/// multiplication/division/modulo. (The P-4 extension scan in
/// `uor-r4-core::transformerless::mod.rs` also covers this file for
/// mul/div/mod as a contract-owned graph-runtime module; the float scan
/// here is additional.)
#[test]
fn packed_source_is_integer_only_by_construction() {
    let source = include_str!("../../uor-r4-graph-runtime/src/route_attention.rs");
    let offenders = scan_source_for_forbidden_ops(source);
    assert!(
        offenders.is_empty(),
        "forbidden arithmetic in the #604 packed lowering:\n{}",
        offenders.join("\n")
    );
}

/// The format-crate substrate (parse/validation and the shared popcount
/// table) is likewise float-free, and its only value products are the
/// scan-visible shift/add stride forms.
#[test]
fn substrate_source_is_integer_only_by_construction() {
    let source = include_str!("../../uor-r4-graph-format/src/route_attention.rs");
    // The substrate's #[cfg(test)] module uses ramp fixtures with
    // mul/mod (compiler-side test data, not operator code): scan only
    // the code above the test module marker.
    let non_test = source
        .split("#[cfg(test)]")
        .next()
        .expect("module has a body");
    let offenders = scan_source_for_forbidden_ops(non_test);
    assert!(
        offenders.is_empty(),
        "forbidden arithmetic in the #604 instance substrate:\n{}",
        offenders.join("\n")
    );
}

/// The #602 registry entry and the #604 substrate agree on the operator
/// identity, and the registry resolves it with the truthful deployed
/// integer class (this crate depends on both sides, so the sync is
/// asserted here).
#[test]
fn registry_identity_matches_the_substrate() {
    use uor_r4_graph_format::route_attention::{
        ROUTE_ATTENTION_OPERATOR_ID, ROUTE_ATTENTION_OPERATOR_VERSION,
    };
    use uor_r4_model_source::attention::{operator_spec, AttentionOperatorSpec};
    assert_eq!(
        AttentionOperatorSpec::R4_ROUTE_ID,
        ROUTE_ATTENTION_OPERATOR_ID
    );
    assert_eq!(
        AttentionOperatorSpec::R4_ROUTE_VERSION,
        ROUTE_ATTENTION_OPERATOR_VERSION
    );
    let record = operator_spec(
        ROUTE_ATTENTION_OPERATOR_ID,
        ROUTE_ATTENTION_OPERATOR_VERSION,
    )
    .expect("the target operator is registered");
    assert_eq!(record.compatibility_relation, "masked-xor-popcount");
    assert_eq!(
        record.selector_normalization,
        "none-bounded-top-m-selection"
    );
    assert_eq!(
        record.tie_breaking,
        "lowest-candidate-index-on-equal-masked-popcount-distance"
    );
    assert_eq!(
        record.permitted_operation_class,
        "deployed-integer-xor-popcount-add-compare-table-read"
    );
    assert!(record.implementation_digest.starts_with("blake3:"));
    // A witness carries exactly this identity.
    let (instance, queries) = pinned_fixture();
    let reference =
        RouteAttentionReference::from_instance_bytes(&instance).expect("fixture instance parses");
    let (_, witness) = reference.run(&queries);
    assert_eq!(witness.operator_id, record.id);
    assert_eq!(witness.operator_version, record.version);
}

/// The census type itself carries no float field: zero float ops "by
/// type" — the counters are all u64 and the operator's op vocabulary
/// has no float class to count.
#[test]
fn census_has_no_float_field_by_type() {
    // Compile-time shape pin: constructing the census exhaustively
    // names every field; all are u64.
    let census = RouteOpCensus {
        adds: 0u64,
        xors: 0u64,
        popcounts: 0u64,
        compares: 0u64,
        table_reads: 0u64,
        bytes_read: 0u64,
        candidates_examined: 0u64,
    };
    assert_eq!(census, RouteOpCensus::default());
}
