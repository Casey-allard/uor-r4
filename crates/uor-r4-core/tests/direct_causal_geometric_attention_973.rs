//! Frozen #973 qualification for the dense direct-attention oracle.
//!
//! Every scored event uses current query key 1. Its correct value changes
//! across documents and is determined only by an earlier causal key/value
//! binding. Construction and V3 validation prefix inputs are disjoint, only
//! recall events are scored, and validation inputs and labels have separate
//! kappas. V2 is retained only as a budget-mismatched historical reveal: its
//! plain/current raw vectors had two effective degrees of freedom while the
//! geometric raw vectors had three. This remains an H4-only offline reference,
//! not an E8 hierarchy binding or evidence of geometric advantage.

use std::collections::BTreeSet;

use uor_r4_core::direct_causal_geometric_attention::{
    DirectAttentionTransportKind, DirectCausalGeometricAttentionConfig,
    DirectCausalGeometricAttentionControl, DirectCausalGeometricAttentionR4V1,
    DIRECT_CAUSAL_GEOMETRIC_ATTENTION_V3_ARM_POLICY,
};
use uor_r4_core::geometric_gated_delta_retention::{
    GeometricRetentionConstructionSequence, GeometricRetentionConstructionStep,
    GeometricRetentionSupportBinding,
};

const MAXIMUM_TOKEN_ID: u32 = 12;
const QUERY_TOKEN: u32 = 1;
const RECALL_SUPPORT: [u32; 2] = [5, 6];

// V3 configuration and decision thresholds. These constants, the fresh V3
// input and label kappas, and the arm-policy/seed identity are frozen before
// the first V3 prediction. Any change is a V4 rung, never a V3 repair.
const FROZEN_CONFIG: DirectCausalGeometricAttentionConfig = DirectCausalGeometricAttentionConfig {
    epochs: 80,
    learning_rate: 0.04,
    temperature: 0.30,
};
const REQUIRED_V3_FULL_CORRECT: usize = 9;
const MAXIMUM_V3_CURRENT_ONLY_CORRECT: usize = 6;
const REQUIRED_V3_CONTROL_DROP: usize = 3;

// Append-only V2 identities/result. The result was revealed under unequal
// effective parameterization and therefore is historical, not promotable.
const FROZEN_V2_VALIDATION_INPUT_KAPPA: &str =
    "blake3:2b2448e51821b2c003ca5cdede0d667fd22def6880003da8f54c38c74a80c09c";
const FROZEN_V2_VALIDATION_LABEL_KAPPA: &str =
    "blake3:e5bc1d8e5e6e390e62c823e51a139ca13bf094e2858d51357ae94104eb3b838a";
const FROZEN_V2_ARTIFACT_CID: &str =
    "blake3:64d4187570e275864a9bf543ba9c27b9eb103426924971d8735bc582b8837c39";
const FROZEN_V2_CONSTRUCTION_POPULATION_KAPPA: &str =
    "blake3:550c303ac607ade900055d2e68690d62e173f284e4a02c5b8bd01868b879d784";
const FROZEN_V2_BUDGET_MISMATCHED_RESULT: &str = concat!(
    "full=8/8;plain=8/8;seed-disabled=8/8;current-only=4/8;",
    "alternative=8/8;key-isometry=6/8;order-shuffled=4/8;value-permuted=0/8"
);

// Populated from the input-only freeze test before the first V3 prediction.
// These placeholders intentionally prevent a V3 scoring test from running
// successfully until all pre-reveal identities have been reviewed and bound.
const FROZEN_V3_VALIDATION_INPUT_KAPPA: &str =
    "blake3:c6c5d6d3ec1af4aaa419ce1857bfe5e389d4a3e7a963d6a87b16d2161809829d";
const FROZEN_V3_VALIDATION_LABEL_KAPPA: &str =
    "blake3:1ec85cd0956b1237df63595f95b525b4d0b9c86a25a47d0453145addbeb9d260";
const FROZEN_V3_EXPERIMENT_KAPPA: &str =
    "blake3:8ff5c7f584f82c8fc7cf39ebbd28274140f4de3faf05eff370bc22e7aa429785";

// The model artifact and construction identity are frozen without reading V3.
const FROZEN_V3_ARTIFACT_CID: &str =
    "blake3:136f0bac7361ca77a30946d8f120843b2d42eacccedbb1f64ab604af3cdf50f3";
const FROZEN_V3_CONSTRUCTION_POPULATION_KAPPA: &str =
    "blake3:61dd6791af2fc115fc0fcecfa4778bd3e8213f77abc1ee635f8847dace10cd89";

fn fixture_binding() -> GeometricRetentionSupportBinding {
    GeometricRetentionSupportBinding::new(
        format!(
            "blake3:{}",
            blake3::hash(b"dcga-973-binding-table-v3").to_hex()
        ),
        format!(
            "blake3:{}",
            blake3::hash(b"dcga-973-binding-overlay-v3").to_hex()
        ),
        "dcga-973-dynamic-binding-construction/3",
    )
    .expect("support binding")
}

fn recall_document(
    document_id: &str,
    causal_prefix: &[u32],
    target: u32,
) -> GeometricRetentionConstructionSequence {
    assert!(causal_prefix.len() >= 2);
    assert_eq!(causal_prefix.last().copied(), Some(QUERY_TOKEN));
    assert!(RECALL_SUPPORT.binary_search(&target).is_ok());
    let mut steps = causal_prefix[1..]
        .iter()
        .copied()
        .map(|token| GeometricRetentionConstructionStep {
            admitted_support: vec![token],
            observed_token: token,
        })
        .collect::<Vec<_>>();
    steps.push(GeometricRetentionConstructionStep {
        admitted_support: RECALL_SUPPORT.to_vec(),
        observed_token: target,
    });
    GeometricRetentionConstructionSequence {
        document_id: document_id.to_owned(),
        initial_token: causal_prefix[0],
        steps,
    }
}

fn construction_split() -> Vec<GeometricRetentionConstructionSequence> {
    vec![
        recall_document("construction-01", &[1, 5, 2, 6, 9, 1], 5),
        recall_document("construction-02", &[1, 6, 2, 5, 9, 1], 6),
        recall_document("construction-03", &[2, 6, 1, 5, 8, 1], 5),
        recall_document("construction-04", &[2, 5, 1, 6, 8, 1], 6),
        recall_document("construction-05", &[3, 1, 5, 4, 2, 6, 9, 1], 5),
        recall_document("construction-06", &[4, 1, 6, 3, 2, 5, 9, 1], 6),
        recall_document("construction-07", &[2, 6, 7, 1, 5, 8, 1], 5),
        recall_document("construction-08", &[2, 5, 7, 1, 6, 8, 1], 6),
        recall_document("construction-09", &[1, 5, 3, 2, 6, 4, 1], 5),
        recall_document("construction-10", &[1, 6, 3, 2, 5, 4, 1], 6),
        recall_document("construction-11", &[2, 6, 3, 1, 5, 7, 8, 1], 5),
        recall_document("construction-12", &[2, 5, 3, 1, 6, 7, 8, 1], 6),
        recall_document("construction-13", &[10, 1, 5, 11, 2, 6, 12, 1], 5),
        recall_document("construction-14", &[10, 1, 6, 11, 2, 5, 12, 1], 6),
        recall_document("construction-15", &[2, 6, 4, 1, 5, 3, 7, 1], 5),
        recall_document("construction-16", &[2, 5, 4, 1, 6, 3, 7, 1], 6),
    ]
}

fn validation_v2_split() -> Vec<GeometricRetentionConstructionSequence> {
    vec![
        recall_document("validation-01", &[3, 2, 6, 4, 1, 5, 7, 8, 1], 5),
        recall_document("validation-02", &[8, 2, 5, 3, 4, 1, 6, 7, 1], 6),
        recall_document("validation-03", &[10, 1, 5, 3, 4, 2, 6, 11, 12, 1], 5),
        recall_document("validation-04", &[11, 2, 5, 7, 1, 6, 3, 4, 1], 6),
        recall_document("validation-05", &[4, 2, 6, 8, 3, 1, 5, 10, 1], 5),
        recall_document("validation-06", &[12, 1, 6, 7, 3, 2, 5, 8, 4, 1], 6),
        recall_document("validation-07", &[2, 6, 10, 11, 1, 5, 3, 7, 8, 1], 5),
        recall_document("validation-08", &[3, 1, 6, 10, 2, 5, 11, 4, 7, 1], 6),
    ]
}

/// Fresh post-correction V3 population. Every prefix has exactly one earlier
/// key-1 binding to its target and ends with the same current query token 1.
/// The population was authored without running the corrected model and is
/// input-disjoint from construction and the already-revealed V2 population.
fn validation_v3_split() -> Vec<GeometricRetentionConstructionSequence> {
    vec![
        recall_document(
            "validation-v3-01",
            &[7, 1, 5, 12, 4, 9, 2, 10, 6, 3, 8, 1],
            5,
        ),
        recall_document(
            "validation-v3-02",
            &[5, 8, 1, 6, 11, 3, 9, 2, 12, 4, 7, 1],
            6,
        ),
        recall_document("validation-v3-03", &[11, 4, 2, 1, 5, 10, 6, 7, 3, 12, 1], 5),
        recall_document("validation-v3-04", &[10, 3, 7, 1, 6, 12, 5, 8, 2, 11, 1], 6),
        recall_document(
            "validation-v3-05",
            &[6, 9, 3, 8, 1, 5, 12, 2, 4, 10, 7, 1],
            5,
        ),
        recall_document(
            "validation-v3-06",
            &[4, 12, 9, 2, 1, 6, 8, 3, 10, 5, 7, 1],
            6,
        ),
        recall_document(
            "validation-v3-07",
            &[12, 3, 7, 2, 9, 1, 5, 4, 11, 6, 8, 10, 1],
            5,
        ),
        recall_document(
            "validation-v3-08",
            &[8, 5, 2, 11, 7, 1, 6, 3, 12, 4, 9, 10, 1],
            6,
        ),
        recall_document(
            "validation-v3-09",
            &[4, 10, 1, 5, 8, 12, 3, 6, 11, 2, 9, 7, 1],
            5,
        ),
        recall_document(
            "validation-v3-10",
            &[3, 9, 1, 6, 7, 11, 4, 5, 10, 2, 12, 8, 1],
            6,
        ),
        recall_document(
            "validation-v3-11",
            &[9, 2, 11, 6, 4, 1, 5, 10, 3, 8, 12, 7, 1],
            5,
        ),
        recall_document(
            "validation-v3-12",
            &[12, 7, 4, 10, 2, 1, 6, 9, 5, 11, 3, 8, 1],
            6,
        ),
    ]
}

fn fixture_model() -> DirectCausalGeometricAttentionR4V1 {
    DirectCausalGeometricAttentionR4V1::compile(
        MAXIMUM_TOKEN_ID,
        &construction_split(),
        FROZEN_CONFIG,
        fixture_binding(),
    )
    .expect("direct attention compiles")
}

fn causal_prefix_and_target(sequence: &GeometricRetentionConstructionSequence) -> (Vec<u32>, u32) {
    let recall_index = sequence
        .steps
        .iter()
        .position(|step| step.admitted_support.len() > 1)
        .expect("one recall event");
    assert_eq!(
        sequence
            .steps
            .iter()
            .filter(|step| step.admitted_support.len() > 1)
            .count(),
        1
    );
    let mut prefix = vec![sequence.initial_token];
    prefix.extend(
        sequence.steps[..recall_index]
            .iter()
            .map(|step| step.observed_token),
    );
    (prefix, sequence.steps[recall_index].observed_token)
}

fn qualify(
    model: &DirectCausalGeometricAttentionR4V1,
    sequences: &[GeometricRetentionConstructionSequence],
    control: DirectCausalGeometricAttentionControl,
) -> (usize, usize) {
    let mut correct = 0;
    for sequence in sequences {
        let (prefix, target) = causal_prefix_and_target(sequence);
        assert_eq!(prefix.last().copied(), Some(QUERY_TOKEN));
        let trace = model
            .predict_prefix(&prefix, &RECALL_SUPPORT, control)
            .expect("recall prediction");
        assert_eq!(trace.query_token, QUERY_TOKEN);
        assert_eq!(trace.admitted_support, RECALL_SUPPORT);
        assert_eq!(trace.future_token_reads, 0);
        assert_eq!(
            trace.causal_token_value_reads,
            if control == DirectCausalGeometricAttentionControl::CurrentTokenOnly {
                1
            } else {
                prefix.len() as u64
            }
        );
        assert_eq!(trace.maximum_position_read + 1, prefix.len());
        assert_eq!(
            trace.positions.len(),
            if control == DirectCausalGeometricAttentionControl::CurrentTokenOnly {
                1
            } else {
                prefix.len()
            },
            "standard inclusive i<=t installs the current query row in K/V memory"
        );
        assert!((trace.softmax_weight_sum - 1.0).abs() <= 1.0e-12);
        correct += usize::from(trace.selected_token == target);
    }
    (correct, sequences.len())
}

fn input_kappa_v2(sequences: &[GeometricRetentionConstructionSequence]) -> String {
    input_kappa(b"uor-r4.dcga-973-validation-input/2\0", sequences)
}

fn input_kappa_v3(sequences: &[GeometricRetentionConstructionSequence]) -> String {
    input_kappa(b"uor-r4.dcga-973-validation-input/3\0", sequences)
}

fn input_kappa(domain: &[u8], sequences: &[GeometricRetentionConstructionSequence]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for sequence in sequences {
        let (prefix, _) = causal_prefix_and_target(sequence);
        hasher.update(&(sequence.document_id.len() as u64).to_le_bytes());
        hasher.update(sequence.document_id.as_bytes());
        hasher.update(&(prefix.len() as u64).to_le_bytes());
        for token in prefix {
            hasher.update(&token.to_le_bytes());
        }
        for token in RECALL_SUPPORT {
            hasher.update(&token.to_le_bytes());
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn label_kappa_v2(sequences: &[GeometricRetentionConstructionSequence]) -> String {
    label_kappa(b"uor-r4.dcga-973-validation-label/2\0", sequences)
}

fn label_kappa_v3(sequences: &[GeometricRetentionConstructionSequence]) -> String {
    label_kappa(b"uor-r4.dcga-973-validation-label/3\0", sequences)
}

fn label_kappa(domain: &[u8], sequences: &[GeometricRetentionConstructionSequence]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for sequence in sequences {
        let (_, target) = causal_prefix_and_target(sequence);
        hasher.update(&(sequence.document_id.len() as u64).to_le_bytes());
        hasher.update(sequence.document_id.as_bytes());
        hasher.update(&target.to_le_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn v3_experiment_kappa(validation_inputs: &str, validation_labels: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.dcga-973-equal-effective-dof-experiment/3\0");
    hasher.update(DIRECT_CAUSAL_GEOMETRIC_ATTENTION_V3_ARM_POLICY.as_bytes());
    hasher.update(&MAXIMUM_TOKEN_ID.to_le_bytes());
    hasher.update(&QUERY_TOKEN.to_le_bytes());
    for token in RECALL_SUPPORT {
        hasher.update(&token.to_le_bytes());
    }
    hasher.update(&FROZEN_CONFIG.epochs.to_le_bytes());
    hasher.update(&FROZEN_CONFIG.learning_rate.to_bits().to_le_bytes());
    hasher.update(&FROZEN_CONFIG.temperature.to_bits().to_le_bytes());
    hasher.update(&(REQUIRED_V3_FULL_CORRECT as u64).to_le_bytes());
    hasher.update(&(MAXIMUM_V3_CURRENT_ONLY_CORRECT as u64).to_le_bytes());
    hasher.update(&(REQUIRED_V3_CONTROL_DROP as u64).to_le_bytes());
    hasher.update(&(validation_inputs.len() as u64).to_le_bytes());
    hasher.update(validation_inputs.as_bytes());
    hasher.update(&(validation_labels.len() as u64).to_le_bytes());
    hasher.update(validation_labels.as_bytes());
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn mechanism_signature(
    model: &DirectCausalGeometricAttentionR4V1,
    prefix: &[u32],
    control: DirectCausalGeometricAttentionControl,
) -> Vec<u64> {
    let trace = model
        .predict_prefix(prefix, &RECALL_SUPPORT, control)
        .expect("control prediction");
    let mut signature = Vec::new();
    signature.extend(trace.positions.iter().flat_map(|position| {
        [
            position.attention_logit.to_bits(),
            position.attention_weight.to_bits(),
        ]
    }));
    signature.extend(trace.aggregate_value.map(f64::to_bits));
    signature.extend(trace.scores.iter().map(|score| score.score.to_bits()));
    signature
}

#[test]
fn v2_history_is_preserved_as_budget_mismatched_and_not_promotable() {
    let validation = validation_v2_split();
    assert_eq!(
        input_kappa_v2(&validation),
        FROZEN_V2_VALIDATION_INPUT_KAPPA
    );
    assert_eq!(
        label_kappa_v2(&validation),
        FROZEN_V2_VALIDATION_LABEL_KAPPA
    );
    assert_eq!(
        FROZEN_V2_BUDGET_MISMATCHED_RESULT,
        concat!(
            "full=8/8;plain=8/8;seed-disabled=8/8;current-only=4/8;",
            "alternative=8/8;key-isometry=6/8;order-shuffled=4/8;value-permuted=0/8"
        )
    );
    assert!(FROZEN_V2_ARTIFACT_CID.starts_with("blake3:"));
    assert!(FROZEN_V2_CONSTRUCTION_POPULATION_KAPPA.starts_with("blake3:"));
}

#[test]
fn v3_population_is_prefix_input_disjoint_unique_and_kappa_bound_before_reveal() {
    let construction = construction_split();
    let validation_v2 = validation_v2_split();
    let validation_v3 = validation_v3_split();
    let construction_ids = construction
        .iter()
        .map(|sequence| sequence.document_id.as_str())
        .collect::<BTreeSet<_>>();
    let validation_v2_ids = validation_v2
        .iter()
        .map(|sequence| sequence.document_id.as_str())
        .collect::<BTreeSet<_>>();
    let validation_v3_ids = validation_v3
        .iter()
        .map(|sequence| sequence.document_id.as_str())
        .collect::<BTreeSet<_>>();
    assert!(construction_ids.is_disjoint(&validation_v2_ids));
    assert!(construction_ids.is_disjoint(&validation_v3_ids));
    assert!(validation_v2_ids.is_disjoint(&validation_v3_ids));

    // The operative input is prefix plus unchanged support. Labels are
    // deliberately excluded from this disjointness audit.
    let construction_prefix_inputs = construction
        .iter()
        .map(|sequence| causal_prefix_and_target(sequence).0)
        .collect::<BTreeSet<_>>();
    let validation_v2_prefix_inputs = validation_v2
        .iter()
        .map(|sequence| causal_prefix_and_target(sequence).0)
        .collect::<BTreeSet<_>>();
    let validation_v3_prefix_inputs = validation_v3
        .iter()
        .map(|sequence| causal_prefix_and_target(sequence).0)
        .collect::<BTreeSet<_>>();
    assert_eq!(construction_prefix_inputs.len(), construction.len());
    assert_eq!(validation_v2_prefix_inputs.len(), validation_v2.len());
    assert_eq!(validation_v3_prefix_inputs.len(), validation_v3.len());
    assert!(construction_prefix_inputs.is_disjoint(&validation_v2_prefix_inputs));
    assert!(construction_prefix_inputs.is_disjoint(&validation_v3_prefix_inputs));
    assert!(validation_v2_prefix_inputs.is_disjoint(&validation_v3_prefix_inputs));
    assert!(validation_v3
        .iter()
        .all(|sequence| causal_prefix_and_target(sequence).0.last() == Some(&QUERY_TOKEN)));
    for sequence in &validation_v3 {
        let (prefix, target) = causal_prefix_and_target(sequence);
        assert_eq!(
            prefix[..prefix.len() - 1]
                .windows(2)
                .filter(|pair| pair[0] == QUERY_TOKEN && pair[1] == target)
                .count(),
            1,
            "{} must contain exactly one earlier query-key/target-value binding",
            sequence.document_id
        );
        assert_eq!(
            prefix[..prefix.len() - 1]
                .iter()
                .filter(|token| **token == QUERY_TOKEN)
                .count(),
            1,
            "{} has an ambiguous earlier query key",
            sequence.document_id
        );
    }

    let inputs = input_kappa_v3(&validation_v3);
    let labels = label_kappa_v3(&validation_v3);
    let experiment = v3_experiment_kappa(&inputs, &labels);
    let model = fixture_model();
    eprintln!("DCGA_973_FROZEN_V3_VALIDATION_INPUT_KAPPA={inputs}");
    eprintln!("DCGA_973_FROZEN_V3_VALIDATION_LABEL_KAPPA={labels}");
    eprintln!("DCGA_973_FROZEN_V3_EXPERIMENT_KAPPA={experiment}");
    eprintln!("DCGA_973_FROZEN_V3_ARTIFACT_CID={}", model.artifact_cid());
    eprintln!(
        "DCGA_973_FROZEN_V3_CONSTRUCTION_POPULATION_KAPPA={}",
        model.construction_population_kappa()
    );
    assert_eq!(inputs, FROZEN_V3_VALIDATION_INPUT_KAPPA);
    assert_eq!(labels, FROZEN_V3_VALIDATION_LABEL_KAPPA);
    assert_eq!(experiment, FROZEN_V3_EXPERIMENT_KAPPA);
    assert_eq!(model.artifact_cid(), FROZEN_V3_ARTIFACT_CID);
    assert_eq!(
        model.construction_population_kappa(),
        FROZEN_V3_CONSTRUCTION_POPULATION_KAPPA
    );
}

#[test]
fn construction_recall_result_is_bound_without_retuning_after_v3() {
    let model = fixture_model();
    let construction = construction_split();
    let full = qualify(
        &model,
        &construction,
        DirectCausalGeometricAttentionControl::FullGeometric,
    );
    let plain = qualify(
        &model,
        &construction,
        DirectCausalGeometricAttentionControl::PlainEuclidean,
    );
    let seed_disabled = qualify(
        &model,
        &construction,
        DirectCausalGeometricAttentionControl::GeometricSeedDisabled,
    );
    let current_only = qualify(
        &model,
        &construction,
        DirectCausalGeometricAttentionControl::CurrentTokenOnly,
    );
    eprintln!(
        "DCGA_973_FROZEN_V3_CONSTRUCTION full={full:?} plain={plain:?} seed_disabled={seed_disabled:?} current_only={current_only:?}"
    );
    // Exact post-reveal audit values are appended here; no optimizer or
    // configuration change is authorized by a construction miss after V3.
    assert_eq!(full, (13, construction.len()));
    assert_eq!(plain, (16, construction.len()));
    assert_eq!(seed_disabled, (16, construction.len()));
    assert_eq!(
        current_only,
        (construction.len() / 2, construction.len()),
        "balanced same-query labels must cap current-token lookup at chance"
    );
}

#[test]
fn coherent_transport_controls_preserve_tangent_norm_and_work_shape() {
    let model = fixture_model();
    let (prefix, _) = causal_prefix_and_target(&construction_split()[0]);
    let full = model
        .predict_prefix(
            &prefix,
            &RECALL_SUPPORT,
            DirectCausalGeometricAttentionControl::FullGeometric,
        )
        .expect("full trace");
    for control in [
        DirectCausalGeometricAttentionControl::AlternativeConnection,
        DirectCausalGeometricAttentionControl::KeyTangentIsometryPermuted,
    ] {
        let trace = model
            .predict_prefix(&prefix, &RECALL_SUPPORT, control)
            .expect("coherent control trace");
        assert_eq!(trace.positions.len(), full.positions.len());
        assert_eq!(trace.q_projections, full.q_projections);
        assert_eq!(trace.k_projections, full.k_projections);
        assert_eq!(trace.v_projections, full.v_projections);
        assert_eq!(trace.o_projections, full.o_projections);
        assert_eq!(trace.key_transports, full.key_transports);
        assert_eq!(trace.value_transports, full.value_transports);
        assert_eq!(trace.output_transports, full.output_transports);
        assert_eq!(
            trace.stored_scalar_parameter_count,
            full.stored_scalar_parameter_count
        );
        assert_eq!(
            trace.learned_effective_degree_count,
            full.learned_effective_degree_count
        );
        assert!(trace.query_tangent_residual <= 1.0e-9);
        assert!(trace.positions.iter().all(|position| {
            position.transported_key_tangent_residual <= 1.0e-9
                && position.transported_value_tangent_residual <= 1.0e-9
        }));
        assert!(trace
            .scores
            .iter()
            .all(|candidate| candidate.output_tangent_residual <= 1.0e-9));
    }
    let alternative = model
        .predict_prefix(
            &prefix,
            &RECALL_SUPPORT,
            DirectCausalGeometricAttentionControl::AlternativeConnection,
        )
        .expect("alternative trace");
    assert!(alternative.positions.iter().all(|position| {
        position.key_transport_kind
            == DirectAttentionTransportKind::AlternativeOrthonormalTrivialization
            && position.value_transport_kind
                == DirectAttentionTransportKind::AlternativeOrthonormalTrivialization
    }));
}

#[test]
fn matched_controls_and_separately_trained_arms_are_mechanically_distinct() {
    let model = fixture_model();
    let (prefix, _) = causal_prefix_and_target(&construction_split()[0]);
    let full = mechanism_signature(
        &model,
        &prefix,
        DirectCausalGeometricAttentionControl::FullGeometric,
    );
    for control in [
        DirectCausalGeometricAttentionControl::PlainEuclidean,
        DirectCausalGeometricAttentionControl::AlternativeConnection,
        DirectCausalGeometricAttentionControl::KeyTangentIsometryPermuted,
        DirectCausalGeometricAttentionControl::OrderShuffled,
        DirectCausalGeometricAttentionControl::ValuePermuted,
        DirectCausalGeometricAttentionControl::GeometricSeedDisabled,
        DirectCausalGeometricAttentionControl::CurrentTokenOnly,
    ] {
        assert_ne!(
            full,
            mechanism_signature(&model, &prefix, control),
            "{control:?}"
        );
    }
    for counts in model.learning_update_counts() {
        assert!(counts.iter().all(|count| *count > 0));
    }

    let current = model
        .predict_prefix(
            &prefix,
            &RECALL_SUPPORT,
            DirectCausalGeometricAttentionControl::CurrentTokenOnly,
        )
        .expect("current-only baseline");
    let mut mutated_prior = vec![u32::MAX; prefix.len()];
    *mutated_prior.last_mut().expect("current position") = QUERY_TOKEN;
    let mutated = model
        .predict_prefix(
            &mutated_prior,
            &RECALL_SUPPORT,
            DirectCausalGeometricAttentionControl::CurrentTokenOnly,
        )
        .expect("current-only ignores all prior token values");
    assert_eq!(current, mutated);
    assert_eq!(current.causal_token_value_reads, 1);
    assert_eq!(
        current.stored_scalar_parameter_count,
        model.stored_scalar_parameter_count_per_arm()
    );
    assert_eq!(
        current.learned_effective_degree_count,
        model.learned_effective_degree_count_per_arm()
    );
    assert_eq!(
        model.stored_scalar_parameter_count_per_arm(),
        (MAXIMUM_TOKEN_ID as usize + 1) * 4 * 4
    );
    assert_eq!(
        model.learned_effective_degree_count_per_arm(),
        (MAXIMUM_TOKEN_ID as usize + 1) * 4 * 3
    );
    assert!(current.query_tangent_residual <= 1.0e-9);
    assert!(current.positions.iter().all(|position| {
        position.transported_key_tangent_residual <= 1.0e-9
            && position.transported_value_tangent_residual <= 1.0e-9
    }));
    assert!(current
        .scores
        .iter()
        .all(|candidate| candidate.output_tangent_residual <= 1.0e-9));

    let plain = model
        .predict_prefix(
            &prefix,
            &RECALL_SUPPORT,
            DirectCausalGeometricAttentionControl::PlainEuclidean,
        )
        .expect("plain fixed-frame trace");
    assert_eq!(
        plain.learned_effective_degree_count,
        model.learned_effective_degree_count_per_arm()
    );
    assert!(plain.query_tangent_residual <= 1.0e-9);
    assert!(plain.positions.iter().all(|position| {
        position.transported_key_tangent_residual <= 1.0e-9
            && position.transported_value_tangent_residual <= 1.0e-9
    }));
}

// Append-only first corrected V3 reveal. These exact counts were recorded
// without changing the already-frozen mechanism, fixture, or thresholds.
const FROZEN_V3_FULL_CORRECT: usize = 3;
const FROZEN_V3_PLAIN_CORRECT: usize = 12;
const FROZEN_V3_SEED_DISABLED_CORRECT: usize = 7;
const FROZEN_V3_CURRENT_ONLY_CORRECT: usize = 6;
const FROZEN_V3_ALTERNATIVE_CORRECT: usize = 10;
const FROZEN_V3_KEY_ISOMETRY_CORRECT: usize = 7;
const FROZEN_V3_ORDER_SHUFFLED_CORRECT: usize = 5;
const FROZEN_V3_VALUE_PERMUTED_CORRECT: usize = 8;
const FROZEN_V3_VERDICT: &str = "FAIL_EQUAL_DOF_H4_DIRECT_ATTENTION_NOT_LOAD_BEARING_ON_FRESH_V3";

#[test]
fn v3_first_frozen_validation_records_negative_attention_verdict() {
    let validation = validation_v3_split();
    let inputs = input_kappa_v3(&validation);
    let labels = label_kappa_v3(&validation);
    assert_eq!(inputs, FROZEN_V3_VALIDATION_INPUT_KAPPA);
    assert_eq!(labels, FROZEN_V3_VALIDATION_LABEL_KAPPA);
    assert_eq!(
        v3_experiment_kappa(&inputs, &labels),
        FROZEN_V3_EXPERIMENT_KAPPA
    );
    let model = fixture_model();
    assert_eq!(model.artifact_cid(), FROZEN_V3_ARTIFACT_CID);
    assert_eq!(
        model.construction_population_kappa(),
        FROZEN_V3_CONSTRUCTION_POPULATION_KAPPA
    );

    let full = qualify(
        &model,
        &validation,
        DirectCausalGeometricAttentionControl::FullGeometric,
    );
    let plain = qualify(
        &model,
        &validation,
        DirectCausalGeometricAttentionControl::PlainEuclidean,
    );
    let seed_disabled = qualify(
        &model,
        &validation,
        DirectCausalGeometricAttentionControl::GeometricSeedDisabled,
    );
    let current_only = qualify(
        &model,
        &validation,
        DirectCausalGeometricAttentionControl::CurrentTokenOnly,
    );
    let alternative = qualify(
        &model,
        &validation,
        DirectCausalGeometricAttentionControl::AlternativeConnection,
    );
    let key_isometry = qualify(
        &model,
        &validation,
        DirectCausalGeometricAttentionControl::KeyTangentIsometryPermuted,
    );
    let order_shuffled = qualify(
        &model,
        &validation,
        DirectCausalGeometricAttentionControl::OrderShuffled,
    );
    let value_permuted = qualify(
        &model,
        &validation,
        DirectCausalGeometricAttentionControl::ValuePermuted,
    );

    eprintln!(
        "DCGA_973_FROZEN V3 first validation: full={full:?} plain={plain:?} seed_disabled={seed_disabled:?} current_only={current_only:?} alternative={alternative:?} key_isometry={key_isometry:?} order_shuffled={order_shuffled:?} value_permuted={value_permuted:?} artifact={} construction_population={} validation_inputs={} validation_labels={} experiment={}",
        model.artifact_cid(),
        model.construction_population_kappa(),
        FROZEN_V3_VALIDATION_INPUT_KAPPA,
        FROZEN_V3_VALIDATION_LABEL_KAPPA,
        FROZEN_V3_EXPERIMENT_KAPPA,
    );

    assert_eq!(full, (FROZEN_V3_FULL_CORRECT, validation.len()));
    assert_eq!(plain, (FROZEN_V3_PLAIN_CORRECT, validation.len()));
    assert_eq!(
        seed_disabled,
        (FROZEN_V3_SEED_DISABLED_CORRECT, validation.len())
    );
    assert_eq!(
        current_only,
        (FROZEN_V3_CURRENT_ONLY_CORRECT, validation.len())
    );
    assert_eq!(
        alternative,
        (FROZEN_V3_ALTERNATIVE_CORRECT, validation.len())
    );
    assert_eq!(
        key_isometry,
        (FROZEN_V3_KEY_ISOMETRY_CORRECT, validation.len())
    );
    assert_eq!(
        order_shuffled,
        (FROZEN_V3_ORDER_SHUFFLED_CORRECT, validation.len())
    );
    assert_eq!(
        value_permuted,
        (FROZEN_V3_VALUE_PERMUTED_CORRECT, validation.len())
    );

    assert_eq!(full.1, validation.len());
    let destructive_controls = [key_isometry, order_shuffled, value_permuted];
    let qualifies = full.0 >= REQUIRED_V3_FULL_CORRECT
        && current_only.0 <= MAXIMUM_V3_CURRENT_ONLY_CORRECT
        && full.0 >= current_only.0 + REQUIRED_V3_CONTROL_DROP
        && destructive_controls
            .iter()
            .all(|control| full.0 >= control.0 + REQUIRED_V3_CONTROL_DROP);
    assert!(
        !qualifies,
        "frozen negative V3 result unexpectedly qualified"
    );
    assert_eq!(
        FROZEN_V3_VERDICT,
        "FAIL_EQUAL_DOF_H4_DIRECT_ATTENTION_NOT_LOAD_BEARING_ON_FRESH_V3"
    );
    assert!(full.0 < REQUIRED_V3_FULL_CORRECT);
    assert!(full.0 < current_only.0);
    assert!(
        destructive_controls
            .iter()
            .all(|control| full.0 < control.0),
        "every frozen geometry-destroying control actually beat the H4 arm"
    );
    // Plain, alternative-connection, and independently trained nongeometric
    // seed arms are reported comparators, not thresholds for a geometry claim.
}

#[test]
fn artifact_replay_is_deterministic_without_a_second_validation_reveal() {
    let left = fixture_model();
    let right = fixture_model();
    assert_eq!(left.to_bytes(), right.to_bytes());
    assert_eq!(left.artifact_cid(), right.artifact_cid());
}
