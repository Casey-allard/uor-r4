//! Construction-only qualification for `GeometricGatedDeltaRetentionR4V1`.
//!
//! This harness uses independently frozen synthetic token sequences and
//! caller-supplied matched supports. It exercises the recurrent mechanism and
//! its controls without claiming that the supports came from #953, opening a
//! held-out language-model population, decoding autonomous text, or proving an
//! integer/table runtime lowering.

use std::collections::BTreeSet;
use std::error::Error;

use serde::Serialize;
use uor_r4_core::geometric_gated_delta_retention::{
    GeometricGatedDeltaRetentionConfig, GeometricGatedDeltaRetentionR4V1,
    GeometricRetentionConstructionSequence, GeometricRetentionConstructionStep,
    GeometricRetentionControl, GeometricRetentionSupportBinding, CANONICAL_PRIME_SPIN_LEAF_POLICY,
    GEOMETRIC_GATED_DELTA_RETENTION_POLICY, GEOMETRIC_RETENTION_SUPPORT_POLICY,
};

const MAXIMUM_TOKEN_ID: u32 = 8;
const CONSTRUCTION_PARTITION_IDENTITY: &str = "uor-r4.ggdr-973.synthetic-construction-partition/1";
const EVENT_ORDER_PERMUTATION: [usize; 7] = [2, 5, 0, 3, 6, 1, 4];
const FROZEN_FIXTURE_KAPPA: &str =
    "blake3:b32a94caaa60c97f2f3df346b65ddf3d7d0e7bab81d786f599fb62c32e2762f5";
const FROZEN_CONSTRUCTION_POPULATION_KAPPA: &str =
    "blake3:f2d07a091936e0619ad29db6053ac097c1469d3b9bd2ef64ff038d5dbe1e51c4";
const FROZEN_EVENT_ORDER_PERMUTATION_KAPPA: &str =
    "blake3:1ec330fbff9d8b644c7c1f69afc48dcb6343b38665946412be18454528a0db11";
const FROZEN_ARTIFACT_CID: &str =
    "blake3:6bf22c9c5283b971d8a9e5e7f4bce067424064a49394f5c7b02a2174e6f38973";
const FROZEN_SMOKE_REPORT_CID: &str =
    "blake3:f99b9815044f139ec0380b5a82502aaf4e159e25761626c0c25eee39173816e1";
const FROZEN_MEASUREMENTS: [(&str, &str, u64, u64, u64, u64, u64); 7] = [
    (
        "full_geometric",
        "frozen_natural_event_order",
        28,
        16,
        112,
        55,
        0,
    ),
    (
        "plain_delta",
        "frozen_natural_event_order",
        28,
        23,
        112,
        98,
        0,
    ),
    (
        "no_delta_overwrite",
        "frozen_natural_event_order",
        28,
        15,
        112,
        58,
        0,
    ),
    (
        "transport_permuted",
        "frozen_natural_event_order",
        28,
        15,
        112,
        62,
        0,
    ),
    (
        "left_fold_route",
        "frozen_natural_event_order",
        28,
        16,
        112,
        55,
        0,
    ),
    (
        "last_only",
        "frozen_natural_event_order",
        28,
        15,
        112,
        52,
        0,
    ),
    (
        "order_shuffled_events",
        "prebound_permutation_2_5_0_3_6_1_4",
        28,
        17,
        112,
        54,
        0,
    ),
];
const CONTROLS: [GeometricRetentionControl; 6] = [
    GeometricRetentionControl::FullGeometric,
    GeometricRetentionControl::PlainDelta,
    GeometricRetentionControl::NoDeltaOverwrite,
    GeometricRetentionControl::TransportPermuted,
    GeometricRetentionControl::LeftFoldRoute,
    GeometricRetentionControl::LastOnly,
];

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct QueryMeasurement {
    arm: String,
    core_control: GeometricRetentionControl,
    event_order: &'static str,
    next_token_queries: u64,
    next_token_correct: u64,
    support_mismatches: u64,
    association_queries: u64,
    association_margin_wins: u64,
    final_state_checksums: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ConstructionSmokeReport {
    schema: u32,
    domain: &'static str,
    evidence_status: &'static str,
    mechanism_scope: &'static str,
    fixture_kind: &'static str,
    fixture_kappa: String,
    artifact_cid: String,
    artifact_bytes: usize,
    byte_identical_recompile: bool,
    reversed_input_replay_identical: bool,
    support_table_cid: String,
    support_overlay_cid: String,
    construction_partition_identity: String,
    construction_population_kappa: String,
    event_order_permutation_kappa: String,
    h4_root_table_kappa: String,
    h4_product_table_kappa: String,
    exact_leaf_map_kappa: String,
    construction_documents: usize,
    construction_events: u64,
    learned_update_counts: [[u64; 3]; 2],
    pairwise_distinct_kvq_tokens: u64,
    bounded_state_scalars: usize,
    controls: Vec<QueryMeasurement>,
    corpus_qualification: &'static str,
    separate_hierarchy_inputs: &'static str,
    typed_953_support_origin: &'static str,
    held_out_language_model_evidence: &'static str,
    decoded_generation_evidence: &'static str,
    runtime_lowering_evidence: &'static str,
}

fn fixture_cid(label: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(label).to_hex())
}

fn fixture_binding() -> GeometricRetentionSupportBinding {
    GeometricRetentionSupportBinding::new(
        fixture_cid(b"uor-r4.ggdr-973.synthetic-matched-table/1"),
        fixture_cid(b"uor-r4.ggdr-973.synthetic-matched-overlay/1"),
        CONSTRUCTION_PARTITION_IDENTITY,
    )
    .expect("synthetic matched-support binding")
}

fn step(support: [u32; 2], observed_token: u32) -> GeometricRetentionConstructionStep {
    GeometricRetentionConstructionStep {
        admitted_support: support.to_vec(),
        observed_token,
    }
}

fn construction_sequences() -> Vec<GeometricRetentionConstructionSequence> {
    vec![
        GeometricRetentionConstructionSequence {
            document_id: "ggdr-construction-a".to_owned(),
            initial_token: 1,
            steps: vec![
                step([2, 6], 2),
                step([3, 7], 3),
                step([4, 8], 4),
                step([1, 5], 1),
                step([2, 6], 2),
                step([3, 7], 3),
                step([4, 8], 4),
            ],
        },
        GeometricRetentionConstructionSequence {
            document_id: "ggdr-construction-b".to_owned(),
            initial_token: 5,
            steps: vec![
                step([2, 6], 6),
                step([3, 7], 7),
                step([4, 8], 8),
                step([1, 5], 5),
                step([2, 6], 6),
                step([3, 7], 7),
                step([4, 8], 8),
            ],
        },
        GeometricRetentionConstructionSequence {
            document_id: "ggdr-construction-c".to_owned(),
            initial_token: 2,
            steps: vec![
                step([3, 7], 3),
                step([4, 8], 4),
                step([1, 5], 1),
                step([2, 6], 2),
                step([3, 7], 3),
                step([4, 8], 4),
                step([1, 5], 1),
            ],
        },
        GeometricRetentionConstructionSequence {
            document_id: "ggdr-construction-d".to_owned(),
            initial_token: 6,
            steps: vec![
                step([3, 7], 7),
                step([4, 8], 8),
                step([1, 5], 5),
                step([2, 6], 6),
                step([3, 7], 7),
                step([4, 8], 8),
                step([1, 5], 5),
            ],
        },
    ]
}

fn order_shuffled_event_sequences() -> Vec<GeometricRetentionConstructionSequence> {
    construction_sequences()
        .into_iter()
        .map(|mut sequence| {
            assert_eq!(sequence.steps.len(), EVENT_ORDER_PERMUTATION.len());
            let original = sequence.steps;
            sequence.steps = EVENT_ORDER_PERMUTATION
                .iter()
                .map(|index| original[*index].clone())
                .collect();
            sequence
        })
        .collect()
}

fn event_order_permutation_kappa(sequences: &[GeometricRetentionConstructionSequence]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.ggdr-973.prebound-event-order-permutation/1\0");
    for index in EVENT_ORDER_PERMUTATION {
        hasher.update(&(index as u64).to_le_bytes());
    }
    hasher.update(&serde_json::to_vec(sequences).expect("permuted fixture serializes"));
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn control_name(control: GeometricRetentionControl) -> &'static str {
    match control {
        GeometricRetentionControl::FullGeometric => "full_geometric",
        GeometricRetentionControl::PlainDelta => "plain_delta",
        GeometricRetentionControl::NoDeltaOverwrite => "no_delta_overwrite",
        GeometricRetentionControl::TransportPermuted => "transport_permuted",
        GeometricRetentionControl::LeftFoldRoute => "left_fold_route",
        GeometricRetentionControl::LastOnly => "last_only",
    }
}

fn fixture_kappa(sequences: &[GeometricRetentionConstructionSequence]) -> String {
    let bytes = serde_json::to_vec(sequences).expect("fixture serializes");
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}

fn fixture_model() -> GeometricGatedDeltaRetentionR4V1 {
    GeometricGatedDeltaRetentionR4V1::compile(
        MAXIMUM_TOKEN_ID,
        &construction_sequences(),
        GeometricGatedDeltaRetentionConfig::default(),
        fixture_binding(),
    )
    .expect("construction-only GGDR fixture compiles")
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn measure_control(
    model: &GeometricGatedDeltaRetentionR4V1,
    sequences: &[GeometricRetentionConstructionSequence],
    control: GeometricRetentionControl,
    arm: impl Into<String>,
    event_order: &'static str,
) -> TestResult<QueryMeasurement> {
    let mut next_token_queries = 0_u64;
    let mut next_token_correct = 0_u64;
    let mut support_mismatches = 0_u64;
    let mut association_queries = 0_u64;
    let mut association_margin_wins = 0_u64;
    let mut final_state_checksums = Vec::new();

    for sequence in sequences {
        let mut state = model.start_state(control)?;
        model.observe(&mut state, sequence.initial_token)?;
        let mut associations = Vec::new();
        let mut context = sequence.initial_token;
        for construction_step in &sequence.steps {
            let before_observation = model.predict(&state, &construction_step.admitted_support)?;
            next_token_queries = next_token_queries.saturating_add(1);
            if before_observation.selected_token == construction_step.observed_token {
                next_token_correct = next_token_correct.saturating_add(1);
            }
            if before_observation.admitted_support != construction_step.admitted_support
                || before_observation.scores.len() != construction_step.admitted_support.len()
                || before_observation
                    .scores
                    .iter()
                    .map(|score| score.token)
                    .collect::<Vec<_>>()
                    != construction_step.admitted_support
            {
                support_mismatches = support_mismatches.saturating_add(1);
            }
            associations.push((
                context,
                construction_step.observed_token,
                construction_step.admitted_support.clone(),
            ));
            model.observe(&mut state, construction_step.observed_token)?;
            context = construction_step.observed_token;
        }
        assert_eq!(state.bounded_scalar_state_len(), 64);
        assert_eq!(state.observations(), sequence.steps.len() as u64 + 1);

        for (key, target, support) in associations {
            for bank_index in 0..4 {
                let target_score = model.association_score(&state, bank_index, key, target)?;
                let distractor_score = support
                    .iter()
                    .copied()
                    .filter(|candidate| *candidate != target)
                    .map(|candidate| model.association_score(&state, bank_index, key, candidate))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .fold(f64::NEG_INFINITY, f64::max);
                association_queries = association_queries.saturating_add(1);
                if target_score > distractor_score {
                    association_margin_wins = association_margin_wins.saturating_add(1);
                }
            }
        }
        final_state_checksums.push(model.state_checksum(&state));
    }

    Ok(QueryMeasurement {
        arm: arm.into(),
        core_control: control,
        event_order,
        next_token_queries,
        next_token_correct,
        support_mismatches,
        association_queries,
        association_margin_wins,
        final_state_checksums,
    })
}

#[test]
fn artifact_recompile_and_bindings_are_deterministic() {
    let sequences = construction_sequences();
    assert_eq!(fixture_kappa(&sequences), FROZEN_FIXTURE_KAPPA);
    let first = fixture_model();
    let second = fixture_model();
    let mut reversed = sequences.clone();
    reversed.reverse();
    let reverse_order_replay = GeometricGatedDeltaRetentionR4V1::compile(
        MAXIMUM_TOKEN_ID,
        &reversed,
        GeometricGatedDeltaRetentionConfig::default(),
        fixture_binding(),
    )
    .expect("reverse-input replay compiles");

    assert_eq!(first.to_bytes(), second.to_bytes());
    assert_eq!(first.artifact_cid(), second.artifact_cid());
    assert_eq!(first.artifact_cid(), FROZEN_ARTIFACT_CID);
    assert_eq!(first.to_bytes(), reverse_order_replay.to_bytes());
    assert_eq!(first.artifact_cid(), reverse_order_replay.artifact_cid());
    assert_eq!(
        first.construction_population_kappa(),
        FROZEN_CONSTRUCTION_POPULATION_KAPPA
    );
    assert_eq!(first.construction_document_ids().len(), sequences.len());
    assert!(first
        .construction_document_ids()
        .windows(2)
        .all(|ids| ids[0] < ids[1]));

    let artifact = first.to_bytes();
    for binding in [
        first.support_binding().table_artifact_cid(),
        first.support_binding().overlay_artifact_cid(),
        first.support_binding().construction_partition_identity(),
        first.construction_population_kappa(),
        first.h4_root_table_kappa(),
        first.h4_product_table_kappa(),
        first.exact_leaf_map_kappa(),
        GEOMETRIC_GATED_DELTA_RETENTION_POLICY,
        GEOMETRIC_RETENTION_SUPPORT_POLICY,
        CANONICAL_PRIME_SPIN_LEAF_POLICY,
    ] {
        assert!(
            contains_bytes(&artifact, binding.as_bytes()),
            "canonical artifact omitted binding {binding}"
        );
    }

    let alternate_binding = GeometricRetentionSupportBinding::new(
        fixture_cid(b"uor-r4.ggdr-973.synthetic-matched-table/1"),
        fixture_cid(b"uor-r4.ggdr-973.synthetic-matched-overlay/1"),
        "uor-r4.ggdr-973.alternate-construction-partition/1",
    )
    .unwrap();
    let alternate = GeometricGatedDeltaRetentionR4V1::compile(
        MAXIMUM_TOKEN_ID,
        &sequences,
        GeometricGatedDeltaRetentionConfig::default(),
        alternate_binding,
    )
    .unwrap();
    assert_ne!(first.to_bytes(), alternate.to_bytes());
    assert_ne!(first.artifact_cid(), alternate.artifact_cid());
    assert_ne!(
        first.construction_population_kappa(),
        alternate.construction_population_kappa()
    );

    let mut target_outside_support = sequences.clone();
    target_outside_support[0].steps[0].observed_token = 3;
    let error = GeometricGatedDeltaRetentionR4V1::compile(
        MAXIMUM_TOKEN_ID,
        &target_outside_support,
        GeometricGatedDeltaRetentionConfig::default(),
        fixture_binding(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("not in its admitted support"));

    let extended_namespace = GeometricGatedDeltaRetentionR4V1::compile(
        MAXIMUM_TOKEN_ID + 1,
        &sequences,
        GeometricGatedDeltaRetentionConfig::default(),
        fixture_binding(),
    )
    .unwrap();
    assert_eq!(
        first.h4_root_table_kappa(),
        extended_namespace.h4_root_table_kappa()
    );
    assert_eq!(
        first.h4_product_table_kappa(),
        extended_namespace.h4_product_table_kappa()
    );
    assert_ne!(
        first.exact_leaf_map_kappa(),
        extended_namespace.exact_leaf_map_kappa()
    );
    assert_ne!(first.artifact_cid(), extended_namespace.artifact_cid());
}

#[test]
fn prediction_is_target_isolated_support_preserving_and_bounded() -> TestResult {
    let model = fixture_model();
    let support = [2, 6];
    let mut causal = model.start_state(GeometricRetentionControl::FullGeometric)?;
    model.observe(&mut causal, 1)?;

    let before_target_two = model.predict(&causal, &support)?;
    let before_target_six = model.predict(&causal, &support)?;
    assert_eq!(before_target_two, before_target_six);
    assert_eq!(before_target_two.admitted_support, support);
    assert_eq!(
        before_target_two.construction_partition_identity,
        CONSTRUCTION_PARTITION_IDENTITY
    );
    assert_eq!(
        before_target_two
            .scores
            .iter()
            .map(|score| score.token)
            .collect::<Vec<_>>(),
        support
    );
    assert_eq!(before_target_two.bank_reads, 4);
    assert_eq!(before_target_two.dot_products, support.len() as u64);
    assert_eq!(causal.bounded_scalar_state_len(), 64);

    let mut observed_two = causal.clone();
    let mut observed_six = causal.clone();
    model.observe(&mut observed_two, 2)?;
    model.observe(&mut observed_six, 6)?;
    assert_ne!(
        model.state_checksum(&observed_two),
        model.state_checksum(&observed_six)
    );
    assert!(model.predict(&causal, &[2, 2]).is_err());
    assert!(model.predict(&causal, &[6, 2]).is_err());
    assert!(model.predict(&causal, &[2, 9]).is_err());
    Ok(())
}

#[test]
fn learned_kvq_controls_and_multi_query_measurements_are_exercised() -> TestResult {
    let sequences = construction_sequences();
    let model = fixture_model();
    let update_counts = model.learning_update_counts();
    assert!(update_counts.iter().flatten().all(|count| *count > 0));

    let mut pairwise_distinct_kvq_tokens = 0_u64;
    for token in 0..=model.maximum_token_id() {
        let geometric = model.placement_trace(token, false)?;
        let plain = model.placement_trace(token, true)?;
        assert!(geometric.pairwise_distinct);
        assert!(plain.pairwise_distinct);
        assert_ne!(geometric.key, geometric.value);
        assert_ne!(geometric.key, geometric.query);
        assert_ne!(geometric.value, geometric.query);
        pairwise_distinct_kvq_tokens = pairwise_distinct_kvq_tokens.saturating_add(1);
    }

    let mut measurements = CONTROLS
        .into_iter()
        .map(|control| {
            measure_control(
                &model,
                &sequences,
                control,
                control_name(control),
                "frozen_natural_event_order",
            )
        })
        .collect::<TestResult<Vec<_>>>()?;
    let order_shuffled_events = order_shuffled_event_sequences();
    assert_eq!(
        event_order_permutation_kappa(&order_shuffled_events),
        FROZEN_EVENT_ORDER_PERMUTATION_KAPPA
    );
    measurements.push(measure_control(
        &model,
        &order_shuffled_events,
        GeometricRetentionControl::FullGeometric,
        "order_shuffled_events",
        "prebound_permutation_2_5_0_3_6_1_4",
    )?);
    let observed_measurements = measurements
        .iter()
        .map(|row| {
            (
                row.arm.as_str(),
                row.event_order,
                row.next_token_queries,
                row.next_token_correct,
                row.association_queries,
                row.association_margin_wins,
                row.support_mismatches,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(observed_measurements.as_slice(), &FROZEN_MEASUREMENTS);
    assert!(measurements.iter().all(|row| row.next_token_queries == 28));
    assert!(measurements
        .iter()
        .all(|row| row.association_queries == 112));
    assert!(measurements.iter().all(|row| row.support_mismatches == 0));
    assert!(measurements
        .iter()
        .all(|row| row.final_state_checksums.len() == sequences.len()));
    assert!(measurements.iter().all(|row| {
        row.final_state_checksums
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            == sequences.len()
    }));
    let control_checksum_sets = measurements
        .iter()
        .map(|row| row.final_state_checksums.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(control_checksum_sets.len(), measurements.len());

    let artifact = model.to_bytes();
    assert_eq!(artifact.len(), 4_233);
    assert_eq!(model.artifact_cid(), FROZEN_ARTIFACT_CID);
    let report = ConstructionSmokeReport {
        schema: 1,
        domain: "uor-r4.geometric-gated-delta-retention-construction-smoke/1",
        evidence_status: "EXERCISED_CONSTRUCTION_SMOKE_ONLY",
        mechanism_scope: "bounded-multirate-last-context-core",
        fixture_kind: "independently-frozen-synthetic-token-sequences-and-matched-supports",
        fixture_kappa: {
            let observed = fixture_kappa(&sequences);
            assert_eq!(observed, FROZEN_FIXTURE_KAPPA);
            observed
        },
        artifact_cid: model.artifact_cid(),
        artifact_bytes: artifact.len(),
        byte_identical_recompile: fixture_model().to_bytes() == artifact,
        reversed_input_replay_identical: {
            let mut reversed = sequences.clone();
            reversed.reverse();
            GeometricGatedDeltaRetentionR4V1::compile(
                MAXIMUM_TOKEN_ID,
                &reversed,
                GeometricGatedDeltaRetentionConfig::default(),
                fixture_binding(),
            )?
            .to_bytes()
                == artifact
        },
        support_table_cid: model.support_binding().table_artifact_cid().to_owned(),
        support_overlay_cid: model.support_binding().overlay_artifact_cid().to_owned(),
        construction_partition_identity: model
            .support_binding()
            .construction_partition_identity()
            .to_owned(),
        construction_population_kappa: model.construction_population_kappa().to_owned(),
        event_order_permutation_kappa: event_order_permutation_kappa(&order_shuffled_events),
        h4_root_table_kappa: model.h4_root_table_kappa().to_owned(),
        h4_product_table_kappa: model.h4_product_table_kappa().to_owned(),
        exact_leaf_map_kappa: model.exact_leaf_map_kappa().to_owned(),
        construction_documents: sequences.len(),
        construction_events: model.construction_event_count(),
        learned_update_counts: update_counts,
        pairwise_distinct_kvq_tokens,
        bounded_state_scalars: model
            .start_state(GeometricRetentionControl::FullGeometric)?
            .bounded_scalar_state_len(),
        controls: measurements,
        corpus_qualification: "NOT_RUN",
        separate_hierarchy_inputs: "NOT_RUN",
        typed_953_support_origin: "NOT_RUN_SYNTHETIC_MATCHED_SUPPORT_ONLY",
        held_out_language_model_evidence: "NOT_RUN",
        decoded_generation_evidence: "NOT_RUN",
        runtime_lowering_evidence: "NOT_RUN",
    };
    let report_bytes = serde_json::to_vec(&report)?;
    assert_eq!(serde_json::to_vec(&report)?, report_bytes);
    let report_cid = format!("blake3:{}", blake3::hash(&report_bytes).to_hex());
    assert_eq!(report_cid, FROZEN_SMOKE_REPORT_CID);
    println!("{}", std::str::from_utf8(&report_bytes)?);
    eprintln!("#973 ggdr_smoke_report_cid={report_cid}");
    Ok(())
}
