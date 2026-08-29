//! Phase-I construction and geometry freeze for `ConnectionGaugeCovarianceV4`.
//!
//! This suite contains only the 16 already-public construction documents and
//! binds only the representation-covariance mechanism and its geometry.

use std::collections::BTreeSet;

use uor_r4_core::direct_causal_geometric_attention::{
    ConnectionGaugeCovarianceV4, ConnectionGaugeCovarianceV4Arm,
    ConnectionGaugeCovarianceV4Intervention, ConnectionGaugeCovarianceV4ParameterCoordinate,
    ConnectionGaugeCovarianceV4Role, DirectCausalGeometricAttentionConfig,
    CONNECTION_GAUGE_COVARIANCE_V4_FINITE_DIFFERENCE_ABSOLUTE_TOLERANCE,
    CONNECTION_GAUGE_COVARIANCE_V4_FINITE_DIFFERENCE_RELATIVE_TOLERANCE,
    CONNECTION_GAUGE_COVARIANCE_V4_FINITE_DIFFERENCE_SCALE,
    CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY,
    CONNECTION_GAUGE_COVARIANCE_V4_GRADIENT_ABSOLUTE_TOLERANCE,
    CONNECTION_GAUGE_COVARIANCE_V4_GRADIENT_RELATIVE_TOLERANCE,
    CONNECTION_GAUGE_COVARIANCE_V4_MAXIMUM_CURRENT_ONLY_CORRECT,
    CONNECTION_GAUGE_COVARIANCE_V4_POLICY,
    CONNECTION_GAUGE_COVARIANCE_V4_REQUIRED_CONSTRUCTION_CORRECT,
    CONNECTION_GAUGE_COVARIANCE_V4_REQUIRED_CONTROL_DROP,
    CONNECTION_GAUGE_COVARIANCE_V4_REQUIRED_VALIDATION_CORRECT,
    CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_ABSOLUTE_TOLERANCE,
    CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_RELATIVE_TOLERANCE,
    CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE,
    CONNECTION_GAUGE_COVARIANCE_V4_UNIT_MARGIN,
};
use uor_r4_core::geometric_gated_delta_retention::{
    GeometricRetentionConstructionSequence, GeometricRetentionConstructionStep,
    GeometricRetentionSupportBinding,
};

const MAXIMUM_TOKEN_ID: u32 = 12;
const QUERY_TOKEN: u32 = 1;
const RECALL_SUPPORT: [u32; 2] = [5, 6];
const EXPECTED_CONSTRUCTION_EVENT_COUNT: u64 = 116;
const EXPECTED_PARAMETER_COUNT_PER_ARM: usize = 13 * 4 * 3;
const FROZEN_CONSTRUCTION_DECISION_GAP: f64 = 1.0e-8;
const FROZEN_PHASE_I_ARTIFACT_CID: &str =
    "blake3:0ed7bf62074857df80045ac3b8bee13ee5f367be4b2b971748631b606ab5985a";
const FROZEN_PHASE_I_CORE_FREEZE_CID: &str =
    "blake3:4c7c33d8de40dd6bd7424c9e6360183f672d55453c257a83fa554e045b6b1d1a";
const FROZEN_PHASE_I_INITIALIZATION_CID: &str =
    "blake3:8f91f8d05cbde422593860cffdc3153007fb5b3b2946217ef0015668d3ac34d0";
const FROZEN_PHASE_I_CONSTRUCTION_KAPPA: &str =
    "blake3:446e4f16c9aff5b5dee4c342bf45847e6e8332d6bed8d4a9a21bfc99f82dbe39";
const FROZEN_PHASE_I_FRAME_MANIFEST_CID: &str =
    "blake3:205ee0d1b9aebbee2475d97de3b95d359ff2ee8220334995cfe4c7a71ead5920";
const FROZEN_PHASE_I_PREFLIGHT_ROOT_CID: &str =
    "blake3:be3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e";

const FROZEN_CONFIG: DirectCausalGeometricAttentionConfig = DirectCausalGeometricAttentionConfig {
    epochs: 80,
    learning_rate: 0.04,
    temperature: 0.30,
};

fn fixture_binding() -> GeometricRetentionSupportBinding {
    GeometricRetentionSupportBinding::new(
        format!(
            "blake3:{}",
            blake3::hash(b"cgcv-973-binding-table-v4").to_hex()
        ),
        format!(
            "blake3:{}",
            blake3::hash(b"cgcv-973-binding-overlay-v4").to_hex()
        ),
        "cgcv-973-construction-only/4",
    )
    .expect("V4 construction support binding")
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

fn causal_prefix_and_target(sequence: &GeometricRetentionConstructionSequence) -> (Vec<u32>, u32) {
    let recall_index = sequence
        .steps
        .iter()
        .position(|step| step.admitted_support.len() > 1)
        .expect("exactly one construction recall event");
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

fn fixture_model() -> ConnectionGaugeCovarianceV4 {
    ConnectionGaugeCovarianceV4::compile(
        MAXIMUM_TOKEN_ID,
        &construction_split(),
        FROZEN_CONFIG,
        fixture_binding(),
    )
    .expect("V4 construction-only model compiles")
}

fn close(actual: f64, expected: f64, absolute: f64, relative: f64) -> bool {
    let scale = actual.abs().max(expected.abs());
    (actual - expected).abs() <= absolute + relative * scale
}

fn assert_scalar_covariance(actual: f64, expected: f64, context: &str) {
    assert!(
        close(
            actual,
            expected,
            CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_ABSOLUTE_TOLERANCE,
            CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_RELATIVE_TOLERANCE,
        ),
        "{context}: {actual:?} != {expected:?}"
    );
}

fn assert_gradient_close(actual: f64, expected: f64, context: &str) {
    assert!(
        close(
            actual,
            expected,
            CONNECTION_GAUGE_COVARIANCE_V4_FINITE_DIFFERENCE_ABSOLUTE_TOLERANCE,
            CONNECTION_GAUGE_COVARIANCE_V4_FINITE_DIFFERENCE_RELATIVE_TOLERANCE,
        ),
        "{context}: analytical={actual:?} finite_difference={expected:?} delta={:?}",
        (actual - expected).abs()
    );
}

fn score(
    trace: &uor_r4_core::direct_causal_geometric_attention::ConnectionGaugeCovarianceV4Trace,
    token: u32,
) -> f64 {
    trace
        .scores
        .iter()
        .find(|candidate| candidate.token == token)
        .expect("candidate is present")
        .score
}

fn assert_exact_work_ledger(
    trace: &uor_r4_core::direct_causal_geometric_attention::ConnectionGaugeCovarianceV4Trace,
    causal_prefix_len: usize,
    current_only: bool,
) {
    let attended = if current_only { 1 } else { causal_prefix_len };
    assert_eq!(trace.input_position_count, causal_prefix_len);
    assert_eq!(trace.query_position + 1, causal_prefix_len);
    assert_eq!(trace.causal_prefix_position_count, causal_prefix_len);
    assert_eq!(trace.masked_future_position_count, 0);
    assert_eq!(trace.maximum_position_read, trace.query_position);
    assert_eq!(trace.future_token_reads, 0);
    assert_eq!(trace.causal_token_value_reads, attended as u64);
    assert_eq!(trace.positions.len(), attended);
    assert_eq!(trace.q_projections, 1);
    assert_eq!(trace.k_projections, attended as u64);
    assert_eq!(trace.v_projections, attended as u64);
    assert_eq!(trace.o_projections, RECALL_SUPPORT.len() as u64);
    assert_eq!(trace.key_transports, attended as u64);
    assert_eq!(trace.value_transports, attended as u64);
    assert_eq!(trace.output_transports, RECALL_SUPPORT.len() as u64);
    assert_eq!(
        trace.stored_scalar_parameter_count,
        EXPECTED_PARAMETER_COUNT_PER_ARM
    );
    assert_eq!(
        trace.learned_effective_degree_count,
        EXPECTED_PARAMETER_COUNT_PER_ARM
    );
    assert_eq!(trace.query_token, QUERY_TOKEN);
    assert_eq!(trace.admitted_support, RECALL_SUPPORT);
    assert!(
        (trace.softmax_weight_sum - 1.0).abs()
            <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE
    );
    assert!(trace.query_tangent_residual <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE);
    let maximum_key_residual = trace
        .positions
        .iter()
        .map(|position| position.transported_key_tangent_residual)
        .fold(0.0_f64, f64::max);
    let maximum_value_residual = trace
        .positions
        .iter()
        .map(|position| position.transported_value_tangent_residual)
        .fold(0.0_f64, f64::max);
    let maximum_output_residual = trace
        .scores
        .iter()
        .map(|candidate| candidate.output_tangent_residual)
        .fold(0.0_f64, f64::max);
    assert!(
        maximum_key_residual <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE,
        "{:?} key tangency residual {maximum_key_residual:?}",
        trace.arm
    );
    assert!(
        maximum_value_residual <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE,
        "{:?} value tangency residual {maximum_value_residual:?}",
        trace.arm
    );
    assert!(
        maximum_output_residual <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE,
        "{:?} output tangency residual {maximum_output_residual:?}",
        trace.arm
    );
}

fn assert_forward_covariance(
    reference: &uor_r4_core::direct_causal_geometric_attention::ConnectionGaugeCovarianceV4Trace,
    actual: &uor_r4_core::direct_causal_geometric_attention::ConnectionGaugeCovarianceV4Trace,
) {
    assert_eq!(actual.selected_token, reference.selected_token);
    assert_eq!(actual.query_token, reference.query_token);
    assert_eq!(actual.admitted_support, reference.admitted_support);
    assert_eq!(actual.positions.len(), reference.positions.len());
    assert_eq!(actual.scores.len(), reference.scores.len());
    for (left, right) in actual.positions.iter().zip(&reference.positions) {
        assert_eq!(left.attended_position, right.attended_position);
        assert_eq!(left.observed_token, right.observed_token);
        assert_eq!(left.key_source_token, right.key_source_token);
        assert_eq!(left.value_source_token, right.value_source_token);
        assert_scalar_covariance(
            left.attention_logit,
            right.attention_logit,
            "attention logit",
        );
        assert_scalar_covariance(
            left.attention_weight,
            right.attention_weight,
            "attention weight",
        );
        for (left_theta, right_theta) in left
            .key_theta
            .coefficients
            .into_iter()
            .zip(right.key_theta.coefficients)
        {
            assert_scalar_covariance(left_theta, right_theta, "key theta");
        }
        for (left_theta, right_theta) in left
            .value_theta
            .coefficients
            .into_iter()
            .zip(right.value_theta.coefficients)
        {
            assert_scalar_covariance(left_theta, right_theta, "value theta");
        }
    }
    for (left, right) in actual.scores.iter().zip(&reference.scores) {
        assert_eq!(left.token, right.token);
        assert_scalar_covariance(left.score, right.score, "candidate score");
        for (left_theta, right_theta) in left
            .output_theta
            .coefficients
            .into_iter()
            .zip(right.output_theta.coefficients)
        {
            assert_scalar_covariance(left_theta, right_theta, "output theta");
        }
    }
    for (left, right) in actual
        .query_theta
        .coefficients
        .into_iter()
        .zip(reference.query_theta.coefficients)
    {
        assert_scalar_covariance(left, right, "query theta");
    }
    for (left, right) in actual
        .aggregate_local_coordinates
        .into_iter()
        .zip(reference.aggregate_local_coordinates)
    {
        assert_scalar_covariance(left, right, "aggregate local coordinate");
    }
}

fn trace_has_exact_work_shape(
    trace: &uor_r4_core::direct_causal_geometric_attention::ConnectionGaugeCovarianceV4Trace,
    causal_prefix_len: usize,
    current_only: bool,
) -> bool {
    let attended = if current_only { 1 } else { causal_prefix_len };
    trace.query_position + 1 == causal_prefix_len
        && trace.causal_prefix_position_count == causal_prefix_len
        && trace.maximum_position_read == trace.query_position
        && trace.future_token_reads == 0
        && trace.causal_token_value_reads == attended as u64
        && trace.positions.len() == attended
        && trace.q_projections == 1
        && trace.k_projections == attended as u64
        && trace.v_projections == attended as u64
        && trace.o_projections == RECALL_SUPPORT.len() as u64
        && trace.key_transports == attended as u64
        && trace.value_transports == attended as u64
        && trace.output_transports == RECALL_SUPPORT.len() as u64
        && trace.stored_scalar_parameter_count == EXPECTED_PARAMETER_COUNT_PER_ARM
        && trace.learned_effective_degree_count == EXPECTED_PARAMETER_COUNT_PER_ARM
}

fn trace_is_finite_and_tangent(
    trace: &uor_r4_core::direct_causal_geometric_attention::ConnectionGaugeCovarianceV4Trace,
) -> bool {
    trace.query_tangent_residual.is_finite()
        && trace.query_tangent_residual <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE
        && trace.softmax_weight_sum.is_finite()
        && trace.aggregate_value.into_iter().all(f64::is_finite)
        && trace
            .aggregate_local_coordinates
            .into_iter()
            .all(f64::is_finite)
        && trace.positions.iter().all(|position| {
            position.attention_logit.is_finite()
                && position.attention_weight.is_finite()
                && position
                    .key_theta
                    .coefficients
                    .into_iter()
                    .all(f64::is_finite)
                && position
                    .value_theta
                    .coefficients
                    .into_iter()
                    .all(f64::is_finite)
                && position.transported_key_tangent_residual.is_finite()
                && position.transported_key_tangent_residual
                    <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE
                && position.transported_value_tangent_residual.is_finite()
                && position.transported_value_tangent_residual
                    <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE
        })
        && trace.scores.iter().all(|candidate| {
            candidate.score.is_finite()
                && candidate
                    .output_theta
                    .coefficients
                    .into_iter()
                    .all(f64::is_finite)
                && candidate.output_tangent_residual.is_finite()
                && candidate.output_tangent_residual
                    <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE
        })
}

#[derive(Debug, Clone, Copy)]
struct ConstructionEvidence {
    correct: [u64; 4],
    forward_covariant: bool,
    exact_work_shape: bool,
    zero_future_reads: bool,
    finite_and_tangent: bool,
}

fn construction_evidence(model: &ConnectionGaugeCovarianceV4) -> ConstructionEvidence {
    let mut evidence = ConstructionEvidence {
        correct: [0; 4],
        forward_covariant: true,
        exact_work_shape: true,
        zero_future_reads: true,
        finite_and_tangent: true,
    };
    for sequence in construction_split() {
        let (prefix, target) = causal_prefix_and_target(&sequence);
        let traces = ConnectionGaugeCovarianceV4Arm::MAIN.map(|arm| {
            model
                .predict_prefix(
                    &prefix,
                    &RECALL_SUPPORT,
                    arm,
                    ConnectionGaugeCovarianceV4Intervention::None,
                )
                .expect("Phase-I construction evidence main arm")
        });
        for (arm_index, trace) in traces.iter().enumerate() {
            evidence.correct[arm_index] += u64::from(trace.selected_token == target);
            evidence.exact_work_shape &= trace_has_exact_work_shape(trace, prefix.len(), false);
            evidence.zero_future_reads &= trace.future_token_reads == 0;
            evidence.finite_and_tangent &= trace_is_finite_and_tangent(trace);
        }
        assert_forward_covariance(&traces[0], &traces[1]);
        assert_forward_covariance(&traces[0], &traces[2]);
        let current = model
            .predict_prefix(
                &prefix,
                &RECALL_SUPPORT,
                ConnectionGaugeCovarianceV4Arm::CurrentTokenOnly,
                ConnectionGaugeCovarianceV4Intervention::None,
            )
            .expect("Phase-I construction evidence current-only arm");
        evidence.correct[3] += u64::from(current.selected_token == target);
        evidence.exact_work_shape &= trace_has_exact_work_shape(&current, prefix.len(), true);
        evidence.zero_future_reads &= current.future_token_reads == 0;
        evidence.finite_and_tangent &= trace_is_finite_and_tangent(&current);
    }
    evidence
}

#[derive(Debug, Clone, Copy)]
struct FiniteDifferenceEvidence {
    checked_coordinate_count: u64,
    maximum_absolute_residual: f64,
    maximum_tolerance_ratio: f64,
}

fn finite_difference_evidence(model: &ConnectionGaugeCovarianceV4) -> FiniteDifferenceEvidence {
    let (prefix, target) = causal_prefix_and_target(&construction_split()[0]);
    let negative = if target == RECALL_SUPPORT[0] {
        RECALL_SUPPORT[1]
    } else {
        RECALL_SUPPORT[0]
    };
    let arm = ConnectionGaugeCovarianceV4Arm::H4Compatible;
    let intervention = ConnectionGaugeCovarianceV4Intervention::None;
    let analytical = model
        .local_objective_and_analytic_gradient(
            &prefix,
            &RECALL_SUPPORT,
            target,
            negative,
            arm,
            intervention,
        )
        .expect("Phase-I preflight analytical gradient");
    let trace = model
        .predict_prefix(&prefix, &RECALL_SUPPORT, arm, intervention)
        .expect("Phase-I preflight representative trace");
    let unique_key_tokens = trace
        .positions
        .iter()
        .map(|position| position.key_source_token)
        .collect::<BTreeSet<_>>();
    let unique_value_tokens = trace
        .positions
        .iter()
        .map(|position| position.value_source_token)
        .collect::<BTreeSet<_>>();
    let mut active = BTreeSet::new();
    for component in 0_u8..3 {
        active.insert(ConnectionGaugeCovarianceV4ParameterCoordinate {
            arm,
            token: QUERY_TOKEN,
            role: ConnectionGaugeCovarianceV4Role::Query,
            component,
        });
        for &token in &unique_key_tokens {
            active.insert(ConnectionGaugeCovarianceV4ParameterCoordinate {
                arm,
                token,
                role: ConnectionGaugeCovarianceV4Role::Key,
                component,
            });
        }
        for &token in &unique_value_tokens {
            active.insert(ConnectionGaugeCovarianceV4ParameterCoordinate {
                arm,
                token,
                role: ConnectionGaugeCovarianceV4Role::Value,
                component,
            });
        }
        for token in RECALL_SUPPORT {
            active.insert(ConnectionGaugeCovarianceV4ParameterCoordinate {
                arm,
                token,
                role: ConnectionGaugeCovarianceV4Role::Output,
                component,
            });
        }
    }
    let snapshot = model.parameter_snapshot(arm);
    let mut evidence = FiniteDifferenceEvidence {
        checked_coordinate_count: 0,
        maximum_absolute_residual: 0.0,
        maximum_tolerance_ratio: 0.0,
    };
    for entry in analytical
        .gradients
        .iter()
        .filter(|entry| active.contains(&entry.coordinate))
    {
        let theta = snapshot
            .iter()
            .find(|parameter| parameter.coordinate == entry.coordinate)
            .expect("active preflight coordinate appears in snapshot")
            .value;
        let step = ConnectionGaugeCovarianceV4::finite_difference_step(theta);
        let plus = model
            .with_parameter_perturbation(entry.coordinate, step)
            .expect("positive preflight perturbation")
            .local_contrastive_objective(
                &prefix,
                &RECALL_SUPPORT,
                target,
                negative,
                arm,
                intervention,
            )
            .expect("positive preflight objective");
        let minus = model
            .with_parameter_perturbation(entry.coordinate, -step)
            .expect("negative preflight perturbation")
            .local_contrastive_objective(
                &prefix,
                &RECALL_SUPPORT,
                target,
                negative,
                arm,
                intervention,
            )
            .expect("negative preflight objective");
        let finite_difference = (plus - minus) / (2.0 * step);
        let residual = (entry.value - finite_difference).abs();
        let allowance = CONNECTION_GAUGE_COVARIANCE_V4_FINITE_DIFFERENCE_ABSOLUTE_TOLERANCE
            + CONNECTION_GAUGE_COVARIANCE_V4_FINITE_DIFFERENCE_RELATIVE_TOLERANCE
                * entry.value.abs().max(finite_difference.abs());
        evidence.checked_coordinate_count += 1;
        evidence.maximum_absolute_residual = evidence.maximum_absolute_residual.max(residual);
        evidence.maximum_tolerance_ratio =
            evidence.maximum_tolerance_ratio.max(residual / allowance);
    }
    evidence
}

#[derive(Debug, Clone, Copy)]
struct ControlShapeEvidence {
    all_execute: bool,
    exact_work_shape: bool,
    finite_and_tangent: bool,
    order_differs: bool,
    value_sources_permuted: bool,
    value_logits_unchanged: bool,
    mismatch_logits_changed: bool,
}

fn control_shape_evidence(model: &ConnectionGaugeCovarianceV4) -> ControlShapeEvidence {
    let (prefix, _) = causal_prefix_and_target(&construction_split()[0]);
    let predict = |intervention| {
        model
            .predict_prefix(
                &prefix,
                &RECALL_SUPPORT,
                ConnectionGaugeCovarianceV4Arm::H4Compatible,
                intervention,
            )
            .expect("Phase-I control-shape trace")
    };
    let baseline = predict(ConnectionGaugeCovarianceV4Intervention::None);
    let order = predict(ConnectionGaugeCovarianceV4Intervention::OrderShuffled);
    let value = predict(ConnectionGaugeCovarianceV4Intervention::ValuePermuted);
    let mismatch = predict(ConnectionGaugeCovarianceV4Intervention::SourceGaugeMismatched);
    let traces = [&order, &value, &mismatch];
    ControlShapeEvidence {
        all_execute: true,
        exact_work_shape: traces
            .iter()
            .all(|trace| trace_has_exact_work_shape(trace, prefix.len(), false)),
        finite_and_tangent: traces
            .iter()
            .all(|trace| trace_is_finite_and_tangent(trace)),
        order_differs: order
            .positions
            .iter()
            .zip(&baseline.positions)
            .any(|(left, right)| {
                left.observed_token != right.observed_token
                    || left.key_source_token != right.key_source_token
                    || !close(
                        left.attention_logit,
                        right.attention_logit,
                        CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_ABSOLUTE_TOLERANCE,
                        CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_RELATIVE_TOLERANCE,
                    )
            }),
        value_sources_permuted: value
            .positions
            .iter()
            .zip(&baseline.positions)
            .any(|(left, right)| left.value_source_token != right.value_source_token),
        value_logits_unchanged: value
            .positions
            .iter()
            .zip(&baseline.positions)
            .all(|(left, right)| left.attention_logit.to_bits() == right.attention_logit.to_bits()),
        mismatch_logits_changed: mismatch.positions.iter().zip(&baseline.positions).any(
            |(left, right)| {
                !close(
                    left.attention_logit,
                    right.attention_logit,
                    CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_ABSOLUTE_TOLERANCE,
                    CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_RELATIVE_TOLERANCE,
                )
            },
        ),
    }
}

fn push_preflight_bytes(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}

fn push_preflight_bool(target: &mut Vec<u8>, value: bool) {
    target.push(u8::from(value));
}

fn push_preflight_u64(target: &mut Vec<u8>, value: usize) {
    target.extend_from_slice(&(value as u64).to_le_bytes());
}

fn push_preflight_f64(target: &mut Vec<u8>, value: f64) {
    target.extend_from_slice(&value.to_bits().to_le_bytes());
}

#[test]
fn phase_i_artifact_construction_and_manifest_bytes_replay() {
    let left = fixture_model();
    let right = fixture_model();

    assert_eq!(
        left.policy_identity(),
        CONNECTION_GAUGE_COVARIANCE_V4_POLICY
    );
    assert_eq!(left.maximum_token_id(), MAXIMUM_TOKEN_ID);
    assert_eq!(
        left.construction_event_count(),
        EXPECTED_CONSTRUCTION_EVENT_COUNT
    );
    assert_eq!(left.construction_document_ids().len(), 16);
    assert!(left
        .construction_document_ids()
        .windows(2)
        .all(|pair| pair[0] < pair[1]));
    assert_eq!(left.support_binding(), &fixture_binding());
    assert_eq!(left.to_bytes(), right.to_bytes());
    assert_eq!(left.artifact_cid(), right.artifact_cid());
    assert_eq!(left.artifact_cid(), FROZEN_PHASE_I_ARTIFACT_CID);
    assert_eq!(left.core_freeze_bytes(), right.core_freeze_bytes());
    assert_eq!(left.core_freeze_cid(), right.core_freeze_cid());
    assert_eq!(left.core_freeze_cid(), FROZEN_PHASE_I_CORE_FREEZE_CID);
    assert_eq!(
        left.construction_population_kappa(),
        right.construction_population_kappa()
    );
    assert_eq!(
        left.construction_population_kappa(),
        FROZEN_PHASE_I_CONSTRUCTION_KAPPA
    );
    assert_eq!(
        left.canonical_frame_manifest_bytes(),
        right.canonical_frame_manifest_bytes()
    );
    assert_eq!(
        left.canonical_frame_manifest_cid(),
        right.canonical_frame_manifest_cid()
    );
    assert_eq!(
        left.canonical_frame_manifest_cid(),
        FROZEN_PHASE_I_FRAME_MANIFEST_CID
    );
    assert_eq!(
        left.artifact_cid(),
        format!("blake3:{}", blake3::hash(&left.to_bytes()).to_hex())
    );
    assert_eq!(
        left.canonical_frame_manifest_cid(),
        format!(
            "blake3:{}",
            blake3::hash(&left.canonical_frame_manifest_bytes()).to_hex()
        )
    );
    assert_eq!(
        left.stored_scalar_parameter_count_per_arm(),
        EXPECTED_PARAMETER_COUNT_PER_ARM
    );
    assert_eq!(
        left.learned_effective_degree_count_per_arm(),
        EXPECTED_PARAMETER_COUNT_PER_ARM
    );
    let initialization_cids =
        ConnectionGaugeCovarianceV4Arm::MAIN.map(|arm| left.initialization_cid(arm));
    assert_eq!(initialization_cids[0], initialization_cids[1]);
    assert_eq!(initialization_cids[0], initialization_cids[2]);
    assert_eq!(initialization_cids[0], FROZEN_PHASE_I_INITIALIZATION_CID);
    assert_eq!(
        initialization_cids[0],
        right.initialization_cid(ConnectionGaugeCovarianceV4Arm::H4Compatible)
    );
    let initial_snapshots =
        ConnectionGaugeCovarianceV4Arm::MAIN.map(|arm| left.initial_parameter_snapshot(arm));
    for comparator in [&initial_snapshots[1], &initial_snapshots[2]] {
        assert_eq!(initial_snapshots[0].len(), comparator.len());
        for (reference, actual) in initial_snapshots[0].iter().zip(comparator) {
            assert_eq!(reference.coordinate.token, actual.coordinate.token);
            assert_eq!(reference.coordinate.role, actual.coordinate.role);
            assert_eq!(reference.coordinate.component, actual.coordinate.component);
            assert_eq!(reference.value.to_bits(), actual.value.to_bits());
        }
    }
    let update_counts = left.learning_update_counts();
    assert!(update_counts
        .iter()
        .all(|role_counts| role_counts.iter().all(|count| *count > 0)));
    assert_eq!(update_counts[0], update_counts[1]);
    assert_eq!(update_counts[0], update_counts[2]);

    eprintln!("CGCV_973_PHASE_I_ARTIFACT_CID={}", left.artifact_cid());
    eprintln!(
        "CGCV_973_PHASE_I_CORE_FREEZE_CID={}",
        left.core_freeze_cid()
    );
    eprintln!(
        "CGCV_973_PHASE_I_INITIALIZATION_CID={}",
        initialization_cids[0]
    );
    eprintln!(
        "CGCV_973_PHASE_I_CONSTRUCTION_KAPPA={}",
        left.construction_population_kappa()
    );
    eprintln!(
        "CGCV_973_PHASE_I_FRAME_MANIFEST_CID={}",
        left.canonical_frame_manifest_cid()
    );
    eprintln!("CGCV_973_PHASE_I_UPDATE_COUNTS={update_counts:?}");
}

#[test]
fn all_main_arms_fit_construction_with_forward_and_decision_covariance() {
    let model = fixture_model();
    let construction = construction_split();
    let mut correct = [0_usize; 3];
    let mut current_only_correct = 0_usize;

    for sequence in &construction {
        let (prefix, target) = causal_prefix_and_target(sequence);
        let negative = if target == RECALL_SUPPORT[0] {
            RECALL_SUPPORT[1]
        } else {
            RECALL_SUPPORT[0]
        };
        let traces = ConnectionGaugeCovarianceV4Arm::MAIN.map(|arm| {
            model
                .predict_prefix(
                    &prefix,
                    &RECALL_SUPPORT,
                    arm,
                    ConnectionGaugeCovarianceV4Intervention::None,
                )
                .expect("construction main-arm prediction")
        });
        for (arm_index, trace) in traces.iter().enumerate() {
            assert_exact_work_ledger(trace, prefix.len(), false);
            assert_eq!(
                trace.selected_token, target,
                "{} {arm_index}",
                sequence.document_id
            );
            assert!(
                score(trace, target) - score(trace, negative) >= FROZEN_CONSTRUCTION_DECISION_GAP,
                "{} {arm_index} must not pass through a numerical tie",
                sequence.document_id
            );
            correct[arm_index] += usize::from(trace.selected_token == target);
        }
        assert_forward_covariance(&traces[0], &traces[1]);
        assert_forward_covariance(&traces[0], &traces[2]);

        let current = model
            .predict_prefix(
                &prefix,
                &RECALL_SUPPORT,
                ConnectionGaugeCovarianceV4Arm::CurrentTokenOnly,
                ConnectionGaugeCovarianceV4Intervention::None,
            )
            .expect("construction current-only prediction");
        assert_exact_work_ledger(&current, prefix.len(), true);
        current_only_correct += usize::from(current.selected_token == target);
    }

    assert_eq!(correct, [16, 16, 16]);
    assert_eq!(
        current_only_correct, 8,
        "balanced same-query construction caps current-only lookup at chance"
    );
    eprintln!(
        "CGCV_973_PHASE_I_CONSTRUCTION h4={}/16 alternative={}/16 plain={}/16 current_only={current_only_correct}/16",
        correct[0], correct[1], correct[2]
    );
}

#[test]
fn phase_i_controls_are_live_and_preserve_causal_work_shape() {
    let evidence = control_shape_evidence(&fixture_model());
    assert!(evidence.all_execute);
    assert!(evidence.exact_work_shape);
    assert!(evidence.finite_and_tangent);
    assert!(evidence.order_differs);
    assert!(evidence.value_sources_permuted);
    assert!(evidence.value_logits_unchanged);
    assert!(evidence.mismatch_logits_changed);
    eprintln!("CGCV_973_PHASE_I_CONTROL_SHAPE={evidence:?}");
}

#[test]
fn canonical_120_frame_manifest_and_every_ordered_pair_pass() {
    let model = fixture_model();
    let records = model
        .canonical_frame_manifest_records()
        .expect("canonical 120-frame manifest");
    assert_eq!(records.len(), 120);
    assert_eq!(
        records
            .iter()
            .map(|record| record.h4_table_offset)
            .collect::<BTreeSet<_>>(),
        (0_u16..120).collect::<BTreeSet<_>>()
    );
    assert_eq!(
        records
            .iter()
            .map(|record| record.scaled_zphi_quaternion)
            .collect::<BTreeSet<_>>()
            .len(),
        120
    );
    for record in &records {
        for row in 0..4 {
            assert_scalar_covariance(
                record.h4_full_frame[row][0],
                record.base[row],
                "H4 base column",
            );
            assert_scalar_covariance(
                record.alternative_full_frame[row][0],
                record.base[row],
                "alternative base column",
            );
        }
    }

    let audit = model
        .exhaustive_connection_audit()
        .expect("120-frame exhaustive connection audit");
    assert_eq!(audit.frame_count, 120);
    assert_eq!(audit.ordered_pair_count, 120 * 120);
    assert!(audit.passes(), "{audit:#?}");
    assert!(
        audit.maximum_h4_left_action_residual
            <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE,
        "H4 endpoint-basis connection must reproduce the existing left action"
    );
    eprintln!("CGCV_973_PHASE_I_CONNECTION_AUDIT={audit:?}");
}

#[test]
fn every_active_representative_qkvo_coordinate_matches_central_difference() {
    let model = fixture_model();
    let (prefix, target) = causal_prefix_and_target(&construction_split()[0]);
    let negative = if target == RECALL_SUPPORT[0] {
        RECALL_SUPPORT[1]
    } else {
        RECALL_SUPPORT[0]
    };
    let arm = ConnectionGaugeCovarianceV4Arm::H4Compatible;
    let intervention = ConnectionGaugeCovarianceV4Intervention::None;
    let analytical = model
        .local_objective_and_analytic_gradient(
            &prefix,
            &RECALL_SUPPORT,
            target,
            negative,
            arm,
            intervention,
        )
        .expect("analytical local gradient");
    assert_eq!(analytical.arm, arm);
    assert_eq!(analytical.intervention, intervention);
    assert_eq!(analytical.target, target);
    assert_eq!(analytical.negative, negative);
    let all_coordinates = analytical
        .gradients
        .iter()
        .map(|entry| entry.coordinate)
        .collect::<BTreeSet<_>>();
    assert_eq!(all_coordinates.len(), analytical.gradients.len());
    for role in [
        ConnectionGaugeCovarianceV4Role::Query,
        ConnectionGaugeCovarianceV4Role::Key,
        ConnectionGaugeCovarianceV4Role::Value,
        ConnectionGaugeCovarianceV4Role::Output,
    ] {
        assert!(all_coordinates
            .iter()
            .any(|coordinate| coordinate.role == role));
    }

    let trace = model
        .predict_prefix(&prefix, &RECALL_SUPPORT, arm, intervention)
        .expect("representative trace");
    let unique_key_tokens = trace
        .positions
        .iter()
        .map(|position| position.key_source_token)
        .collect::<BTreeSet<_>>();
    let unique_value_tokens = trace
        .positions
        .iter()
        .map(|position| position.value_source_token)
        .collect::<BTreeSet<_>>();
    let mut expected_active_coordinates = BTreeSet::new();
    for component in 0_u8..3 {
        expected_active_coordinates.insert(ConnectionGaugeCovarianceV4ParameterCoordinate {
            arm,
            token: QUERY_TOKEN,
            role: ConnectionGaugeCovarianceV4Role::Query,
            component,
        });
        for &token in &unique_key_tokens {
            expected_active_coordinates.insert(ConnectionGaugeCovarianceV4ParameterCoordinate {
                arm,
                token,
                role: ConnectionGaugeCovarianceV4Role::Key,
                component,
            });
        }
        for &token in &unique_value_tokens {
            expected_active_coordinates.insert(ConnectionGaugeCovarianceV4ParameterCoordinate {
                arm,
                token,
                role: ConnectionGaugeCovarianceV4Role::Value,
                component,
            });
        }
        for token in RECALL_SUPPORT {
            expected_active_coordinates.insert(ConnectionGaugeCovarianceV4ParameterCoordinate {
                arm,
                token,
                role: ConnectionGaugeCovarianceV4Role::Output,
                component,
            });
        }
    }
    assert_eq!(
        expected_active_coordinates.len(),
        3 * (1 + unique_key_tokens.len() + unique_value_tokens.len() + RECALL_SUPPORT.len()),
        "repeated-token K/V contributions must be accumulated into one parameter coordinate"
    );
    assert!(expected_active_coordinates.is_subset(&all_coordinates));

    let snapshot = model.parameter_snapshot(arm);
    for entry in analytical
        .gradients
        .iter()
        .filter(|entry| expected_active_coordinates.contains(&entry.coordinate))
    {
        let theta = snapshot
            .iter()
            .find(|parameter| parameter.coordinate == entry.coordinate)
            .expect("active coordinate appears in parameter snapshot")
            .value;
        let step = CONNECTION_GAUGE_COVARIANCE_V4_FINITE_DIFFERENCE_SCALE * theta.abs().max(1.0);
        let plus = model
            .with_parameter_perturbation(entry.coordinate, step)
            .expect("positive finite-difference perturbation")
            .local_contrastive_objective(
                &prefix,
                &RECALL_SUPPORT,
                target,
                negative,
                arm,
                intervention,
            )
            .expect("positive objective");
        let minus = model
            .with_parameter_perturbation(entry.coordinate, -step)
            .expect("negative finite-difference perturbation")
            .local_contrastive_objective(
                &prefix,
                &RECALL_SUPPORT,
                target,
                negative,
                arm,
                intervention,
            )
            .expect("negative objective");
        let finite_difference = (plus - minus) / (2.0 * step);
        assert_gradient_close(
            entry.value,
            finite_difference,
            &format!("{:?}", entry.coordinate),
        );
    }
}

#[test]
fn local_gradients_and_one_step_update_deltas_are_gauge_covariant() {
    let model = fixture_model();
    let (prefix, target) = causal_prefix_and_target(&construction_split()[0]);
    let negative = if target == RECALL_SUPPORT[0] {
        RECALL_SUPPORT[1]
    } else {
        RECALL_SUPPORT[0]
    };
    let audit = model
        .covariance_update_delta_audit(&prefix, &RECALL_SUPPORT, target, negative)
        .expect("local-gradient and update-delta covariance audit");
    assert_eq!(audit.compared_arm_count, 3);
    assert!(audit.passes(), "{audit:#?}");
    assert!(audit.maximum_scalar_tolerance_ratio <= 1.0);
    assert!(audit.maximum_gradient_tolerance_ratio <= 1.0);
    eprintln!("CGCV_973_PHASE_I_COVARIANCE_AUDIT={audit:?}");
}

#[test]
fn causal_boundary_ignores_suffix_values_and_reports_zero_future_reads() {
    let model = fixture_model();
    let (prefix, _) = causal_prefix_and_target(&construction_split()[0]);
    let query_position = prefix.len() - 1;
    let baseline = model
        .predict_at(
            &prefix,
            query_position,
            &RECALL_SUPPORT,
            ConnectionGaugeCovarianceV4Arm::H4Compatible,
            ConnectionGaugeCovarianceV4Intervention::None,
        )
        .expect("baseline causal trace");
    let mut with_opaque_suffix = prefix.clone();
    with_opaque_suffix.extend([u32::MAX, u32::MAX - 1, u32::MAX - 2]);
    let suffix = model
        .predict_at(
            &with_opaque_suffix,
            query_position,
            &RECALL_SUPPORT,
            ConnectionGaugeCovarianceV4Arm::H4Compatible,
            ConnectionGaugeCovarianceV4Intervention::None,
        )
        .expect("suffix token values remain unread");
    assert_eq!(baseline.positions, suffix.positions);
    assert_eq!(baseline.aggregate_value, suffix.aggregate_value);
    assert_eq!(
        baseline.aggregate_local_coordinates,
        suffix.aggregate_local_coordinates
    );
    assert_eq!(baseline.scores, suffix.scores);
    assert_eq!(baseline.selected_token, suffix.selected_token);
    assert_eq!(suffix.input_position_count, with_opaque_suffix.len());
    assert_eq!(suffix.causal_prefix_position_count, prefix.len());
    assert_eq!(suffix.masked_future_position_count, 3);
    assert_eq!(suffix.maximum_position_read, query_position);
    assert_eq!(suffix.future_token_reads, 0);
}

#[test]
fn phase_i_preflight_root_binds_complete_construction_evidence() {
    let model = fixture_model();
    let replay = fixture_model();
    assert_eq!(
        model.policy_identity(),
        CONNECTION_GAUGE_COVARIANCE_V4_POLICY
    );
    assert_eq!(
        model.generator_policy_identity(),
        CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY
    );

    let artifact_cid = model.artifact_cid();
    let core_freeze_cid = model.core_freeze_cid();
    let initialization_cid = model
        .initialization_cid(ConnectionGaugeCovarianceV4Arm::H4Compatible)
        .to_owned();
    let construction_kappa = model.construction_population_kappa().to_owned();
    let frame_manifest_cid = model.canonical_frame_manifest_cid();

    let construction = construction_evidence(&model);
    assert_eq!(construction.correct, [16, 16, 16, 8]);
    assert!(construction.forward_covariant);
    assert!(construction.exact_work_shape);
    assert!(construction.zero_future_reads);
    assert!(construction.finite_and_tangent);

    let update_counts = model.learning_update_counts();
    let main_update_counts_equal =
        update_counts[0] == update_counts[1] && update_counts[0] == update_counts[2];
    assert!(main_update_counts_equal);
    assert!(update_counts
        .iter()
        .all(|role_counts| role_counts.iter().all(|count| *count > 0)));

    let initial_snapshots =
        ConnectionGaugeCovarianceV4Arm::MAIN.map(|arm| model.initial_parameter_snapshot(arm));
    let identical_initialization = [&initial_snapshots[1], &initial_snapshots[2]]
        .into_iter()
        .all(|comparator| {
            initial_snapshots[0].len() == comparator.len()
                && initial_snapshots[0]
                    .iter()
                    .zip(comparator)
                    .all(|(reference, actual)| {
                        reference.coordinate.token == actual.coordinate.token
                            && reference.coordinate.role == actual.coordinate.role
                            && reference.coordinate.component == actual.coordinate.component
                            && reference.value.to_bits() == actual.value.to_bits()
                    })
        });
    assert!(identical_initialization);

    let frame_records = model
        .canonical_frame_manifest_records()
        .expect("Phase-I preflight frame records");
    let exact_frame_manifest = frame_records.len() == 120
        && frame_records
            .iter()
            .map(|record| record.h4_table_offset)
            .collect::<BTreeSet<_>>()
            == (0_u16..120).collect::<BTreeSet<_>>()
        && frame_records
            .iter()
            .map(|record| record.scaled_zphi_quaternion)
            .collect::<BTreeSet<_>>()
            .len()
            == 120;
    assert!(exact_frame_manifest);
    let frame_audit = model
        .exhaustive_connection_audit()
        .expect("Phase-I preflight exhaustive frame audit");
    assert!(frame_audit.passes(), "{frame_audit:#?}");

    let (representative_prefix, representative_target) =
        causal_prefix_and_target(&construction_split()[0]);
    let representative_negative = if representative_target == RECALL_SUPPORT[0] {
        RECALL_SUPPORT[1]
    } else {
        RECALL_SUPPORT[0]
    };
    let covariance_audit = model
        .covariance_update_delta_audit(
            &representative_prefix,
            &RECALL_SUPPORT,
            representative_target,
            representative_negative,
        )
        .expect("Phase-I preflight covariance audit");
    assert!(covariance_audit.passes(), "{covariance_audit:#?}");

    let finite_difference = finite_difference_evidence(&model);
    assert_eq!(finite_difference.checked_coordinate_count, 39);
    assert!(finite_difference.maximum_tolerance_ratio <= 1.0);

    let controls = control_shape_evidence(&model);
    assert!(controls.all_execute);
    assert!(controls.exact_work_shape);
    assert!(controls.finite_and_tangent);
    assert!(controls.order_differs);
    assert!(controls.value_sources_permuted);
    assert!(controls.value_logits_unchanged);
    assert!(controls.mismatch_logits_changed);

    let query_position = representative_prefix.len() - 1;
    let causal = model
        .predict_at(
            &representative_prefix,
            query_position,
            &RECALL_SUPPORT,
            ConnectionGaugeCovarianceV4Arm::H4Compatible,
            ConnectionGaugeCovarianceV4Intervention::None,
        )
        .expect("Phase-I preflight causal baseline");
    let mut opaque_suffix = representative_prefix.clone();
    opaque_suffix.extend([u32::MAX, u32::MAX - 1, u32::MAX - 2]);
    let suffix = model
        .predict_at(
            &opaque_suffix,
            query_position,
            &RECALL_SUPPORT,
            ConnectionGaugeCovarianceV4Arm::H4Compatible,
            ConnectionGaugeCovarianceV4Intervention::None,
        )
        .expect("Phase-I preflight suffix remains unread");
    let suffix_value_invariant = causal.positions == suffix.positions
        && causal.aggregate_value == suffix.aggregate_value
        && causal.aggregate_local_coordinates == suffix.aggregate_local_coordinates
        && causal.scores == suffix.scores
        && causal.selected_token == suffix.selected_token
        && suffix.future_token_reads == 0
        && suffix.maximum_position_read == query_position
        && suffix.masked_future_position_count == 3;
    assert!(suffix_value_invariant);

    let byte_replay = model.to_bytes() == replay.to_bytes()
        && model.core_freeze_bytes() == replay.core_freeze_bytes()
        && model.canonical_frame_manifest_bytes() == replay.canonical_frame_manifest_bytes()
        && artifact_cid == replay.artifact_cid()
        && core_freeze_cid == replay.core_freeze_cid()
        && frame_manifest_cid == replay.canonical_frame_manifest_cid();
    assert!(byte_replay);

    // Canonical, append-only field order for the public Phase-I evidence root.
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"CGPR0004");
    push_preflight_bytes(&mut bytes, model.policy_identity().as_bytes());
    push_preflight_bytes(&mut bytes, model.generator_policy_identity().as_bytes());
    bytes.extend_from_slice(&MAXIMUM_TOKEN_ID.to_le_bytes());
    bytes.extend_from_slice(&QUERY_TOKEN.to_le_bytes());
    push_preflight_u64(&mut bytes, RECALL_SUPPORT.len());
    for token in RECALL_SUPPORT {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    bytes.extend_from_slice(&FROZEN_CONFIG.epochs.to_le_bytes());
    push_preflight_f64(&mut bytes, FROZEN_CONFIG.learning_rate);
    push_preflight_f64(&mut bytes, FROZEN_CONFIG.temperature);
    for tolerance in [
        CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE,
        CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_ABSOLUTE_TOLERANCE,
        CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_RELATIVE_TOLERANCE,
        CONNECTION_GAUGE_COVARIANCE_V4_GRADIENT_ABSOLUTE_TOLERANCE,
        CONNECTION_GAUGE_COVARIANCE_V4_GRADIENT_RELATIVE_TOLERANCE,
        CONNECTION_GAUGE_COVARIANCE_V4_FINITE_DIFFERENCE_ABSOLUTE_TOLERANCE,
        CONNECTION_GAUGE_COVARIANCE_V4_FINITE_DIFFERENCE_RELATIVE_TOLERANCE,
        CONNECTION_GAUGE_COVARIANCE_V4_FINITE_DIFFERENCE_SCALE,
        CONNECTION_GAUGE_COVARIANCE_V4_UNIT_MARGIN,
        FROZEN_CONSTRUCTION_DECISION_GAP,
    ] {
        push_preflight_f64(&mut bytes, tolerance);
    }
    for threshold in [
        CONNECTION_GAUGE_COVARIANCE_V4_REQUIRED_CONSTRUCTION_CORRECT,
        CONNECTION_GAUGE_COVARIANCE_V4_REQUIRED_VALIDATION_CORRECT,
        CONNECTION_GAUGE_COVARIANCE_V4_MAXIMUM_CURRENT_ONLY_CORRECT,
        CONNECTION_GAUGE_COVARIANCE_V4_REQUIRED_CONTROL_DROP,
    ] {
        push_preflight_u64(&mut bytes, threshold);
    }
    for cid in [
        artifact_cid.as_str(),
        core_freeze_cid.as_str(),
        initialization_cid.as_str(),
        construction_kappa.as_str(),
        frame_manifest_cid.as_str(),
    ] {
        push_preflight_bytes(&mut bytes, cid.as_bytes());
    }
    push_preflight_u64(&mut bytes, model.construction_document_ids().len());
    for document_id in model.construction_document_ids() {
        push_preflight_bytes(&mut bytes, document_id.as_bytes());
    }
    bytes.extend_from_slice(&model.construction_event_count().to_le_bytes());
    for count in construction.correct {
        bytes.extend_from_slice(&count.to_le_bytes());
    }
    for role_counts in update_counts {
        for count in role_counts {
            bytes.extend_from_slice(&count.to_le_bytes());
        }
    }
    push_preflight_u64(&mut bytes, model.stored_scalar_parameter_count_per_arm());
    push_preflight_u64(&mut bytes, model.learned_effective_degree_count_per_arm());
    push_preflight_u64(&mut bytes, frame_audit.frame_count);
    push_preflight_u64(&mut bytes, frame_audit.ordered_pair_count);
    for residual in [
        frame_audit.maximum_frame_orthogonality_residual,
        frame_audit.maximum_frame_orientation_residual,
        frame_audit.maximum_tangent_residual,
        frame_audit.maximum_base_mapping_residual,
        frame_audit.maximum_connection_orthogonality_residual,
        frame_audit.maximum_tangent_composition_residual,
        frame_audit.maximum_connection_composition_residual,
        frame_audit.maximum_h4_left_action_residual,
        frame_audit.maximum_local_gauge_orthogonality_residual,
        frame_audit.maximum_tangent_basis_mapping_residual,
        frame_audit.maximum_source_tangent_projector_residual,
        frame_audit.maximum_destination_tangent_projector_residual,
        frame_audit.maximum_tangent_transpose_reciprocity_residual,
    ] {
        push_preflight_f64(&mut bytes, residual);
    }
    push_preflight_u64(&mut bytes, covariance_audit.compared_arm_count);
    push_preflight_bool(&mut bytes, covariance_audit.decision_parity);
    for residual in [
        covariance_audit.maximum_logit_absolute_delta,
        covariance_audit.maximum_weight_absolute_delta,
        covariance_audit.maximum_score_absolute_delta,
        covariance_audit.maximum_objective_absolute_delta,
        covariance_audit.maximum_gradient_absolute_delta,
        covariance_audit.maximum_update_delta_absolute_delta,
        covariance_audit.maximum_scalar_tolerance_ratio,
        covariance_audit.maximum_gradient_tolerance_ratio,
    ] {
        push_preflight_f64(&mut bytes, residual);
    }
    bytes.extend_from_slice(&finite_difference.checked_coordinate_count.to_le_bytes());
    push_preflight_f64(&mut bytes, finite_difference.maximum_absolute_residual);
    push_preflight_f64(&mut bytes, finite_difference.maximum_tolerance_ratio);
    for flag in [
        construction.forward_covariant,
        construction.exact_work_shape,
        construction.zero_future_reads,
        construction.finite_and_tangent,
        identical_initialization,
        main_update_counts_equal,
        exact_frame_manifest,
        frame_audit.passes(),
        covariance_audit.passes(),
        finite_difference.maximum_tolerance_ratio <= 1.0,
        controls.all_execute,
        controls.exact_work_shape,
        controls.finite_and_tangent,
        controls.order_differs,
        controls.value_sources_permuted,
        controls.value_logits_unchanged,
        controls.mismatch_logits_changed,
        suffix_value_invariant,
        byte_replay,
    ] {
        push_preflight_bool(&mut bytes, flag);
    }
    let preflight_root = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    eprintln!("CGCV_973_PHASE_I_ARTIFACT_CID={artifact_cid}");
    eprintln!("CGCV_973_PHASE_I_CORE_FREEZE_CID={core_freeze_cid}");
    eprintln!("CGCV_973_PHASE_I_INITIALIZATION_CID={initialization_cid}");
    eprintln!("CGCV_973_PHASE_I_CONSTRUCTION_KAPPA={construction_kappa}");
    eprintln!("CGCV_973_PHASE_I_FRAME_MANIFEST_CID={frame_manifest_cid}");
    eprintln!("CGCV_973_PHASE_I_PREFLIGHT_ROOT_CID={preflight_root}");
    eprintln!("CGCV_973_PHASE_I_PREFLIGHT_FD={finite_difference:?}");
    eprintln!("CGCV_973_PHASE_I_PREFLIGHT_CONTROLS={controls:?}");
    eprintln!("CGCV_973_PHASE_I_PREFLIGHT_UPDATE_COUNTS={update_counts:?}");
    assert_eq!(preflight_root, FROZEN_PHASE_I_PREFLIGHT_ROOT_CID);
}
