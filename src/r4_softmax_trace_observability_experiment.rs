//! Construction-only leave-one-document-out observability for #1012.
//!
//! Each fold compiles fresh suffix and recurrent artifacts from exactly three
//! construction documents. The fourth document is used only after fitting for
//! the diagnostic evaluation. No #1011 held-out document, source checkpoint,
//! or four-document-fitted state artifact is accepted by this module.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uor_r4_core::r4_softmax_trace_state_student::{
    compile_r4_softmax_trace_state_student, signed_reduce_final_layer_r4,
    R4SoftmaxTraceReductionRole, R4SoftmaxTraceStateArm, R4SoftmaxTraceStateFitConfig,
    R4SoftmaxTraceStateFitEvent, R4SoftmaxTraceStateFitSequence,
    R4SoftmaxTraceStateStudentArtifact, R4_SOFTMAX_TRACE_STATE_FITTED_VALUES_PER_ARM,
};
use uor_r4_core::r4_softmax_trace_student::{
    compile_r4_softmax_trace_student, R4SoftmaxTraceSequence, R4SoftmaxTraceStudentArm,
    R4SoftmaxTraceStudentArtifact, R4SoftmaxTraceStudentConfig, TeacherTopDistributionQ16,
    R4_SOFTMAX_TRACE_Q16_TOTAL,
};

use crate::r4_softmax_teacher_trace::{R4SoftmaxHeadTrace, R4SoftmaxTeacherTraceBundle};
use crate::r4_softmax_trace_experiment::{teacher_distribution, R4SoftmaxTraceFreeze};
use crate::r4_softmax_trace_state_experiment::{
    CONSTRUCTION_TRACE_BUNDLE_CID, PREDECESSOR_FREEZE_CID,
};

pub const OBSERVABILITY_RESULT_SCHEMA: &str = "uor-r4.r4-softmax-trace-observability-result/1";

const ISSUE: u32 = 1012;
const EXPECTED_TRACE_BYTES: usize = 45_205_493;
const EXPECTED_DOCUMENT_IDS: [&str; 4] = ["14", "657", "4579", "5121"];
const EXPECTED_DOCUMENT_TRACE_CIDS: [&str; 4] = [
    "blake3:62bb33d4605914541443bea0c972990031a0821449e932e19a929db1369f2960",
    "blake3:4a31dec9c955c5bca5513673cb0fa5c2fe3b7eb93f27a177838142abd00b4704",
    "blake3:3a51132fd40387f0e0f55e1fd50a7bfc05b8f1376e6fe877d63f9ab60bfd45ad",
    "blake3:bc47e6f862978effb616da762f935cbcc97b71631bb2cd6c650a6e6be1e91efc",
];
const EXPECTED_DOCUMENT_POSITIONS: [usize; 4] = [8, 8, 10, 12];
const EXPECTED_PRIMARY_PER_FOLD: [usize; 4] = [7, 3, 9, 7];
const EXPECTED_SECONDARY_PER_FOLD: [usize; 4] = [7, 7, 9, 11];
const EXPECTED_POSITIONS: usize = 38;
const EXPECTED_PRIMARY_EVENTS: usize = 26;
const EXPECTED_SECONDARY_EVENTS: usize = 34;
const FULL_QKV_WIDTH: usize = 1_728;
const REDUCED_QKV_WIDTH: usize = 12;
const PROBE_WIDTH: usize = 64;
const PROBE_STEPS: usize = 512;
const PROBE_BACKTRACKS: usize = 16;
const PROBE_INITIAL_RATE: f64 = 1.0 / 16.0;
const PROBE_REGULARIZATION: f64 = 1.0 / 1024.0;
const PROBE_RMS_EPSILON: f64 = 1.0e-12;
const SUPPORT_CAP: usize = 32;
const MAXIMUM_TOKEN_ID: u32 = 49_151;
const MATERIAL_CE_DELTA_NATS: f64 = 0.10;
const REQUIRED_DIRECTION_FOLDS: u64 = 3;

const DIAGNOSTIC_BOUNDARIES: [&str; 5] = [
    "full_final_layer_qkv_signed_sketch",
    "signed_576_to_4_qkv_sketch",
    "plain_natural_state_tensor",
    "geometric_natural_state_tensor",
    "transport_permuted_natural_state_tensor",
];

const EXACT_BOUNDARIES: [&str; 4] = [
    "suffix_exact_base_logits",
    "plain_exact_base_plus_residual_logits",
    "geometric_exact_base_plus_residual_logits",
    "transport_permuted_exact_base_plus_residual_logits",
];

#[derive(Clone, Debug)]
pub struct R4SoftmaxTraceObservabilityConfig {
    pub implementation_revision: String,
    pub trace_bundle: PathBuf,
    pub predecessor_freeze: PathBuf,
    pub result_output: PathBuf,
}

#[derive(Debug)]
pub enum R4SoftmaxTraceObservabilityError {
    Invalid(String),
    Io(std::io::Error),
    Serialization(String),
    Trace(String),
    Student(String),
}

impl fmt::Display for R4SoftmaxTraceObservabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => {
                write!(formatter, "invalid observability experiment: {reason}")
            }
            Self::Io(error) => write!(formatter, "observability I/O failed: {error}"),
            Self::Serialization(reason) => {
                write!(formatter, "observability serialization failed: {reason}")
            }
            Self::Trace(reason) => write!(formatter, "observability trace failed: {reason}"),
            Self::Student(reason) => write!(formatter, "observability student failed: {reason}"),
        }
    }
}

impl std::error::Error for R4SoftmaxTraceObservabilityError {}

impl From<std::io::Error> for R4SoftmaxTraceObservabilityError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservabilityInputAudit {
    pub trace_bundle_reads: u64,
    pub predecessor_freeze_reads: u64,
    pub source_model_reads: u64,
    pub source_model_forwards: u64,
    pub held_out_document_13_reads: u64,
    pub canonical_bundle_reload_exact: bool,
    pub nested_document_cids_exact: bool,
    pub document_order_exact: bool,
    pub construction_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeContract {
    pub weights: usize,
    pub bias: bool,
    pub regularization: String,
    pub steps: usize,
    pub initial_rate: String,
    pub maximum_backtracks_per_step: usize,
    pub full_batch: bool,
    pub candidate_conditioned_sketch: bool,
    pub support_cap: usize,
    pub fit_loss_weighting: String,
    pub headline_loss_weighting: String,
    pub paired_loss_weighting: String,
    pub feature_standardization: String,
    pub feature_standardization_epsilon: String,
    pub signed_projection: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoldManifest {
    pub ordinal: usize,
    pub held_out_document_id: String,
    pub held_out_document_trace_cid: String,
    pub training_document_ids: Vec<String>,
    pub training_document_trace_cids: Vec<String>,
    pub training_positions: usize,
    pub training_non_bos_events: usize,
    pub primary_exact_prefix_novel_events: usize,
    pub secondary_non_bos_events: usize,
    pub fold_input_cid: String,
    pub support_artifact_cid: String,
    pub state_artifact_cid: String,
    pub training_support_cid: String,
    pub label_control_mapping_cid: String,
    pub label_control_mass_audit: LabelControlMassAudit,
    pub training_fit_identity_cid: String,
    pub held_label_substitution_fit_identity_exact: bool,
    pub held_label_substitution_mutated_events: u64,
    pub held_labels_original_cid: String,
    pub held_labels_substituted_cid: String,
    pub held_out_events_used_for_fit: u64,
    pub state_resets: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabelControlMassAudit {
    pub rows: u64,
    pub original_covered_mass_q16: u64,
    pub permuted_covered_mass_q16: u64,
    pub mass_preserved_exactly: bool,
    pub invalid_support_rows: u64,
    pub unchanged_label_rows: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureAudit {
    pub raw_width: usize,
    pub probe_width: usize,
    pub fitted_weights: usize,
    pub bias: bool,
    pub candidate_feature_rows: usize,
    pub unique_feature_rows: usize,
    pub exact_collision_rows: usize,
    pub zero_overlap_rows: usize,
    pub active_lanes: usize,
    pub numerical_rank: usize,
    pub total_variance: f64,
    pub all_finite: bool,
    pub target_blind_preprocessing: bool,
    pub feature_cid: String,
    pub standardization_stats_cid: String,
    pub train_only_unit_rms_standardization: bool,
    pub standardized_unit_rms_lanes: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSummary {
    pub positions: u64,
    pub teacher_mass_covered_q16: u64,
    pub covered_mass_fraction: Option<f64>,
    pub covered_teacher_cross_entropy_nats: Option<f64>,
    pub teacher_top1_agreements: u64,
    pub actual_next_top1_correct: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedUncertainty {
    pub positions: u64,
    pub candidate_loss_delta_control_minus_real_nats: Option<f64>,
    pub real_wins: u64,
    pub control_wins: u64,
    pub ties: u64,
    pub exact_two_sided_sign_p: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticBoundaryResult {
    pub boundary: String,
    pub feature_audit: FeatureAudit,
    pub fitted_weight_cid: String,
    pub label_permuted_weight_cid: String,
    pub label_control_mapping_cid: String,
    pub label_permuted_training_rows_changed: u64,
    pub optimizer_backtracks: u64,
    pub label_permuted_optimizer_backtracks: u64,
    pub primary: MetricSummary,
    pub primary_label_permuted: MetricSummary,
    pub primary_paired: PairedUncertainty,
    pub secondary: MetricSummary,
    pub secondary_label_permuted: MetricSummary,
    pub secondary_paired: PairedUncertainty,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResidualScaleTrial {
    pub alpha: String,
    pub training_covered_teacher_cross_entropy_nats: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResidualScaleDiagnostic {
    pub trials: Vec<ResidualScaleTrial>,
    pub selected_alpha: String,
    pub selection_uses_training_only: bool,
    pub deterministic_tie_break_prefers_smaller_alpha: bool,
    pub selected_primary: MetricSummary,
    pub selected_secondary: MetricSummary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactBoundaryResult {
    pub boundary: String,
    pub fitted_parameter_values: usize,
    pub actual_alpha: String,
    pub primary: MetricSummary,
    pub secondary: MetricSummary,
    pub residual_scale_diagnostic: Option<ResidualScaleDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoldResult {
    pub manifest: FoldManifest,
    pub diagnostic_boundaries: Vec<DiagnosticBoundaryResult>,
    pub exact_boundaries: Vec<ExactBoundaryResult>,
    pub candidate_support_cid: String,
    pub candidate_support_rows: usize,
    pub candidate_support_maximum: usize,
    pub candidate_support_train_only: bool,
    pub runtime_source_reads: u64,
    pub runtime_future_reads: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateDiagnosticResult {
    pub boundary: String,
    pub primary: MetricSummary,
    pub primary_label_permuted: MetricSummary,
    pub primary_paired: PairedUncertainty,
    pub secondary: MetricSummary,
    pub secondary_label_permuted: MetricSummary,
    pub secondary_paired: PairedUncertainty,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateExactResult {
    pub boundary: String,
    pub primary: MetricSummary,
    pub secondary: MetricSummary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionComparison {
    pub comparison: String,
    pub aggregate_delta_nats: f64,
    pub directional_folds: u64,
    pub required_delta_nats: f64,
    pub required_directional_folds: u64,
    pub material_and_stable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionAudit {
    pub aggregate_primary_coverage_fraction: f64,
    pub minimum_primary_fold_coverage_fraction: f64,
    pub support_coverage_sufficient: bool,
    pub full_improvement_vs_suffix: DecisionComparison,
    pub full_improvement_vs_label_control: DecisionComparison,
    pub reduction_or_projection_loss_vs_full: DecisionComparison,
    pub geometric_state_loss_vs_reduced: DecisionComparison,
    pub current_readout_loss_vs_geometric_probe: DecisionComparison,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkLedger {
    pub logical_folds: u64,
    pub execution_passes: u64,
    pub replay_passes: u64,
    pub fold_executions: u64,
    pub suffix_compiles: u64,
    pub state_compiles: u64,
    pub probe_fits: u64,
    pub probe_steps: u64,
    pub probe_backtracks: u64,
    pub probe_objective_evaluations: u64,
    pub held_label_substitution_audits: u64,
    pub training_event_fit_incidences: u64,
    pub training_alpha_score_row_evaluations: u64,
    pub primary_score_row_evaluations: u64,
    pub secondary_score_row_evaluations: u64,
    pub state_observations: u64,
    pub state_resets: u64,
}

impl WorkLedger {
    fn combine_with_replay(&self, replay: &Self) -> Result<Self, R4SoftmaxTraceObservabilityError> {
        if self.logical_folds != replay.logical_folds
            || self.execution_passes != 1
            || replay.execution_passes != 1
            || self.replay_passes != 0
            || replay.replay_passes != 0
        {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "per-pass work ledgers do not have the frozen replay shape".to_owned(),
            ));
        }
        let add = |left: u64, right: u64, field: &str| {
            left.checked_add(right).ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(format!(
                    "combined work-ledger field {field} overflowed"
                ))
            })
        };
        Ok(Self {
            logical_folds: self.logical_folds,
            execution_passes: add(
                self.execution_passes,
                replay.execution_passes,
                "execution_passes",
            )?,
            replay_passes: 1,
            fold_executions: add(
                self.fold_executions,
                replay.fold_executions,
                "fold_executions",
            )?,
            suffix_compiles: add(
                self.suffix_compiles,
                replay.suffix_compiles,
                "suffix_compiles",
            )?,
            state_compiles: add(self.state_compiles, replay.state_compiles, "state_compiles")?,
            probe_fits: add(self.probe_fits, replay.probe_fits, "probe_fits")?,
            probe_steps: add(self.probe_steps, replay.probe_steps, "probe_steps")?,
            probe_backtracks: add(
                self.probe_backtracks,
                replay.probe_backtracks,
                "probe_backtracks",
            )?,
            probe_objective_evaluations: add(
                self.probe_objective_evaluations,
                replay.probe_objective_evaluations,
                "probe_objective_evaluations",
            )?,
            held_label_substitution_audits: add(
                self.held_label_substitution_audits,
                replay.held_label_substitution_audits,
                "held_label_substitution_audits",
            )?,
            training_event_fit_incidences: add(
                self.training_event_fit_incidences,
                replay.training_event_fit_incidences,
                "training_event_fit_incidences",
            )?,
            training_alpha_score_row_evaluations: add(
                self.training_alpha_score_row_evaluations,
                replay.training_alpha_score_row_evaluations,
                "training_alpha_score_row_evaluations",
            )?,
            primary_score_row_evaluations: add(
                self.primary_score_row_evaluations,
                replay.primary_score_row_evaluations,
                "primary_score_row_evaluations",
            )?,
            secondary_score_row_evaluations: add(
                self.secondary_score_row_evaluations,
                replay.secondary_score_row_evaluations,
                "secondary_score_row_evaluations",
            )?,
            state_observations: add(
                self.state_observations,
                replay.state_observations,
                "state_observations",
            )?,
            state_resets: add(self.state_resets, replay.state_resets, "state_resets")?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4SoftmaxTraceObservabilityResult {
    pub schema: String,
    pub issue: u32,
    pub implementation_revision: String,
    pub predecessor_freeze_cid: String,
    pub construction_trace_bundle_cid: String,
    pub construction_trace_bundle_bytes: usize,
    pub construction_document_trace_cids: Vec<String>,
    pub input_audit: ObservabilityInputAudit,
    pub probe_contract: ProbeContract,
    pub primary_event_count: usize,
    pub secondary_event_count: usize,
    pub folds: Vec<FoldResult>,
    pub aggregate_diagnostic_boundaries: Vec<AggregateDiagnosticResult>,
    pub aggregate_exact_boundaries: Vec<AggregateExactResult>,
    pub decision_audit: DecisionAudit,
    pub work_ledger: WorkLedger,
    pub exact_replay: bool,
    pub terminal: String,
    pub next_action: String,
    pub result_cid: String,
    pub nonclaims: Vec<String>,
}

#[derive(Clone)]
struct Document {
    id: String,
    trace_cid: String,
    text_cid: String,
    input_tokens: Vec<u32>,
    events: Vec<Event>,
}

#[derive(Clone)]
struct Event {
    position: usize,
    observed_token: u32,
    actual_next_token: u32,
    frame_table_offset: u16,
    teacher: TeacherTopDistributionQ16,
    full_qkv: Vec<f64>,
    reduced_qkv: [f64; REDUCED_QKV_WIDTH],
}

#[derive(Clone)]
struct CandidateView {
    token: u32,
    teacher_q16: u16,
    full_features: [f64; PROBE_WIDTH],
    reduced_features: [f64; PROBE_WIDTH],
    plain_features: [f64; PROBE_WIDTH],
    geometric_features: [f64; PROBE_WIDTH],
    permuted_features: [f64; PROBE_WIDTH],
    suffix_logit: f64,
    plain_residual_logit: f64,
    plain_logit: f64,
    geometric_residual_logit: f64,
    geometric_logit: f64,
    permuted_residual_logit: f64,
    permuted_logit: f64,
}

#[derive(Clone)]
struct Row {
    document_id: String,
    position: usize,
    actual_next_token: u32,
    teacher_top_token: u32,
    teacher_top32_q16: BTreeMap<u32, u16>,
    candidates: Vec<CandidateView>,
    exact_prefix_novel: bool,
}

#[derive(Clone)]
struct ProbeRow {
    document_id: String,
    position: usize,
    actual_next_token: u32,
    teacher_top_token: u32,
    teacher_top32_q16: BTreeMap<u32, u16>,
    tokens: Vec<u32>,
    teacher_q16: Vec<u16>,
    base_logits: Vec<f64>,
    features: Vec<[f64; PROBE_WIDTH]>,
}

#[derive(Clone)]
struct ExactRow {
    document_id: String,
    position: usize,
    actual_next_token: u32,
    teacher_top_token: u32,
    tokens: Vec<u32>,
    teacher_q16: Vec<u16>,
    logits: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
struct ScoredPosition {
    document_id: String,
    position: usize,
    teacher_mass_q16: u64,
    covered_loss_nats: f64,
    teacher_top1: bool,
    actual_top1: bool,
}

#[derive(Clone)]
struct FittedProbe {
    weights: [f64; PROBE_WIDTH],
    backtracks: u64,
    changed_label_rows: u64,
}

struct FeatureStandardization {
    stats_cid: String,
    unit_rms_lanes: usize,
}

struct LabelControlPlan {
    row_identities: BTreeSet<(String, usize)>,
    mapping_cid: String,
}

#[derive(Default)]
struct SignedProjectionCache {
    source_lane_masks: BTreeMap<(String, u32, usize), Vec<u64>>,
}

impl SignedProjectionCache {
    fn project(
        &mut self,
        boundary: &str,
        candidate: u32,
        values: &[f64],
    ) -> Result<[f64; PROBE_WIDTH], R4SoftmaxTraceObservabilityError> {
        if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "signed-projection input is empty or non-finite".to_owned(),
            ));
        }
        let key = (boundary.to_owned(), candidate, values.len());
        if !self.source_lane_masks.contains_key(&key) {
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"uor-r4/1012/candidate-signed-projection-xof-u64/v1");
            hasher.update(&(boundary.len() as u64).to_le_bytes());
            hasher.update(boundary.as_bytes());
            hasher.update(&candidate.to_le_bytes());
            hasher.update(&(values.len() as u64).to_le_bytes());
            let mut reader = hasher.finalize_xof();
            let byte_len = values.len().checked_mul(8).ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "signed-projection mask length overflowed".to_owned(),
                )
            })?;
            let mut bytes = vec![0_u8; byte_len];
            reader.fill(&mut bytes);
            let masks = bytes
                .chunks_exact(8)
                .map(|chunk| {
                    let mut mask = [0_u8; 8];
                    mask.copy_from_slice(chunk);
                    u64::from_le_bytes(mask)
                })
                .collect::<Vec<_>>();
            self.source_lane_masks.insert(key.clone(), masks);
        }
        let masks = self.source_lane_masks.get(&key).ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "signed-projection cache insertion failed".to_owned(),
            )
        })?;
        let scale = 1.0 / (values.len() as f64).sqrt();
        let mut output = [0.0_f64; PROBE_WIDTH];
        for (output_lane, output_value) in output.iter_mut().enumerate() {
            for (source, value) in values.iter().enumerate() {
                let sign = if masks[source] & (1_u64 << output_lane) == 0 {
                    1.0
                } else {
                    -1.0
                };
                *output_value += sign * *value * scale;
            }
        }
        if output.iter().any(|value| !value.is_finite()) {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "signed-projection output is non-finite".to_owned(),
            ));
        }
        Ok(output)
    }
}

/// Run the complete four-fold construction-only audit and write its canonical
/// structured result.
pub fn run_observability(
    config: &R4SoftmaxTraceObservabilityConfig,
) -> Result<R4SoftmaxTraceObservabilityResult, R4SoftmaxTraceObservabilityError> {
    validate_revision(&config.implementation_revision)?;
    validate_distinct_paths(&[
        &config.trace_bundle,
        &config.predecessor_freeze,
        &config.result_output,
    ])?;

    let freeze_bytes = fs::read(&config.predecessor_freeze)?;
    let freeze_value: serde_json::Value = serde_json::from_slice(&freeze_bytes)
        .map_err(|error| R4SoftmaxTraceObservabilityError::Serialization(error.to_string()))?;
    let freeze: R4SoftmaxTraceFreeze = serde_json::from_slice(&freeze_bytes)
        .map_err(|error| R4SoftmaxTraceObservabilityError::Serialization(error.to_string()))?;
    validate_predecessor_freeze(freeze_value, &freeze)?;

    let trace_bytes = fs::read(&config.trace_bundle)?;
    let expected_cids = EXPECTED_DOCUMENT_TRACE_CIDS
        .iter()
        .map(|cid| (*cid).to_owned())
        .collect::<Vec<_>>();
    let bundle = R4SoftmaxTeacherTraceBundle::from_bytes_with_expected_cids(
        &trace_bytes,
        CONSTRUCTION_TRACE_BUNDLE_CID,
        &expected_cids,
    )
    .map_err(|error| R4SoftmaxTraceObservabilityError::Trace(error.to_string()))?;
    if trace_bytes.len() != EXPECTED_TRACE_BYTES
        || bundle
            .canonical_bytes()
            .map_err(|error| R4SoftmaxTraceObservabilityError::Trace(error.to_string()))?
            != trace_bytes
        || bundle
            .document_trace_cids()
            .map_err(|error| R4SoftmaxTraceObservabilityError::Trace(error.to_string()))?
            != expected_cids
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "construction bundle bytes, canonical replay, or nested CIDs differ from the freeze"
                .to_owned(),
        ));
    }
    let documents = extract_documents(&bundle, &expected_cids)?;
    let mut first = execute_all_folds(&config.implementation_revision, &documents)?;
    let replay = execute_all_folds(&config.implementation_revision, &documents)?;
    if first != replay {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "observability computation did not replay exactly".to_owned(),
        ));
    }
    first.work_ledger = first.work_ledger.combine_with_replay(&replay.work_ledger)?;
    first.exact_replay = true;
    first.input_audit = ObservabilityInputAudit {
        trace_bundle_reads: 1,
        predecessor_freeze_reads: 1,
        source_model_reads: 0,
        source_model_forwards: 0,
        held_out_document_13_reads: 0,
        canonical_bundle_reload_exact: true,
        nested_document_cids_exact: true,
        document_order_exact: true,
        construction_only: true,
    };
    first.result_cid = canonical_json_cid_omitting_result_cid(&first)?;
    validate_result(&first)?;
    write_json_atomic(&config.result_output, &first)?;
    let reloaded_bytes = fs::read(&config.result_output)?;
    let reloaded: R4SoftmaxTraceObservabilityResult = serde_json::from_slice(&reloaded_bytes)
        .map_err(|error| R4SoftmaxTraceObservabilityError::Serialization(error.to_string()))?;
    if reloaded != first || canonical_json_cid_omitting_result_cid(&reloaded)? != first.result_cid {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "observability result reload or CID validation failed".to_owned(),
        ));
    }
    Ok(first)
}

fn validate_predecessor_freeze(
    value: serde_json::Value,
    freeze: &R4SoftmaxTraceFreeze,
) -> Result<(), R4SoftmaxTraceObservabilityError> {
    let computed = canonical_value_cid_omitting_fields(value, &["compile_seconds", "freeze_cid"])?;
    let ids = freeze
        .construction_documents
        .iter()
        .map(|document| document.id.as_str())
        .collect::<Vec<_>>();
    let positions = freeze
        .construction_documents
        .iter()
        .map(|document| document.target_tokens)
        .collect::<Vec<_>>();
    let expected_cids = EXPECTED_DOCUMENT_TRACE_CIDS
        .iter()
        .map(|cid| (*cid).to_owned())
        .collect::<Vec<_>>();
    if freeze.freeze_cid != PREDECESSOR_FREEZE_CID
        || computed != PREDECESSOR_FREEZE_CID
        || freeze.trace_bundle_cid != CONSTRUCTION_TRACE_BUNDLE_CID
        || freeze.trace_bundle_bytes != EXPECTED_TRACE_BYTES
        || freeze.document_trace_cids != expected_cids
        || ids != EXPECTED_DOCUMENT_IDS
        || positions != EXPECTED_DOCUMENT_POSITIONS
        || freeze.construction_positions != EXPECTED_POSITIONS
        || freeze.held_out_teacher_scored
        || freeze.held_out_identity_bound_into_artifact
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "#1010 predecessor freeze does not bind the exact construction-only trace input"
                .to_owned(),
        ));
    }
    if freeze
        .construction_documents
        .iter()
        .any(|document| document.id == "13" || document.partition != "construction")
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "predecessor freeze contains a non-construction or burned document".to_owned(),
        ));
    }
    Ok(())
}

fn extract_documents(
    bundle: &R4SoftmaxTeacherTraceBundle,
    expected_cids: &[String],
) -> Result<Vec<Document>, R4SoftmaxTraceObservabilityError> {
    if bundle.traces().len() != EXPECTED_DOCUMENT_IDS.len() {
        return Err(R4SoftmaxTraceObservabilityError::Trace(
            "observability requires exactly four construction documents".to_owned(),
        ));
    }
    let mut documents = Vec::with_capacity(bundle.traces().len());
    let mut total_positions = 0_usize;
    for (ordinal, trace) in bundle.traces().iter().enumerate() {
        if trace.identity.document_id != EXPECTED_DOCUMENT_IDS[ordinal]
            || trace.identity.document_id == "13"
            || trace.positions.len() != EXPECTED_DOCUMENT_POSITIONS[ordinal]
            || trace.bounds.layers != 30
            || trace.bounds.query_heads != 9
            || trace.bounds.head_size != 64
            || trace.bounds.vocabulary != usize::try_from(MAXIMUM_TOKEN_ID + 1).unwrap_or(0)
        {
            return Err(R4SoftmaxTraceObservabilityError::Trace(format!(
                "construction trace document {ordinal} has an invalid identity, order, or shape"
            )));
        }
        let mut input_tokens = Vec::with_capacity(trace.positions.len());
        let mut events = Vec::with_capacity(trace.positions.len());
        for (expected_position, position) in trace.positions.iter().enumerate() {
            if usize::try_from(position.position).ok() != Some(expected_position)
                || usize::from(position.frame_table_offset) >= 120
                || position.input_token > MAXIMUM_TOKEN_ID
                || position.logits.target_token > MAXIMUM_TOKEN_ID
            {
                return Err(R4SoftmaxTraceObservabilityError::Trace(
                    "construction event order, frame, or token namespace is invalid".to_owned(),
                ));
            }
            let final_layer = position.layers.last().ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Trace(
                    "construction event has no final trace layer".to_owned(),
                )
            })?;
            if usize::try_from(final_layer.layer).ok() != Some(trace.bounds.layers - 1) {
                return Err(R4SoftmaxTraceObservabilityError::Trace(
                    "construction event final layer is not canonical".to_owned(),
                ));
            }
            let full_qkv = full_ordered_qkv(&final_layer.heads)?;
            let reduced_qkv = reduced_ordered_qkv(&final_layer.heads)?;
            let teacher = teacher_distribution(&position.logits)
                .map_err(|error| R4SoftmaxTraceObservabilityError::Trace(error.to_string()))?;
            input_tokens.push(position.input_token);
            events.push(Event {
                position: expected_position,
                observed_token: position.input_token,
                actual_next_token: position.logits.target_token,
                frame_table_offset: position.frame_table_offset,
                teacher,
                full_qkv,
                reduced_qkv,
            });
        }
        total_positions = total_positions.checked_add(events.len()).ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid("position census overflowed".to_owned())
        })?;
        documents.push(Document {
            id: trace.identity.document_id.clone(),
            trace_cid: expected_cids[ordinal].clone(),
            text_cid: trace.identity.document_text_cid.clone(),
            input_tokens,
            events,
        });
    }
    if total_positions != EXPECTED_POSITIONS {
        return Err(R4SoftmaxTraceObservabilityError::Trace(format!(
            "construction census is {total_positions}; {EXPECTED_POSITIONS} required"
        )));
    }
    Ok(documents)
}

fn full_ordered_qkv(
    heads: &[R4SoftmaxHeadTrace],
) -> Result<Vec<f64>, R4SoftmaxTraceObservabilityError> {
    if heads.len() != 9 {
        return Err(R4SoftmaxTraceObservabilityError::Trace(
            "final layer must contain exactly nine heads".to_owned(),
        ));
    }
    let mut values = Vec::with_capacity(FULL_QKV_WIDTH);
    for role in 0..3 {
        for (head_index, head) in heads.iter().enumerate() {
            if usize::try_from(head.head).ok() != Some(head_index) {
                return Err(R4SoftmaxTraceObservabilityError::Trace(
                    "final-layer head order is noncanonical".to_owned(),
                ));
            }
            let bits = match role {
                0 => &head.query_gauge_bits,
                1 => &head.current_key_query_gauge_bits,
                _ => &head.current_value_query_gauge_bits,
            };
            if bits.len() != 64 {
                return Err(R4SoftmaxTraceObservabilityError::Trace(
                    "final-layer Q/K/V head width is not 64".to_owned(),
                ));
            }
            for &bits in bits {
                let value = f32::from_bits(bits);
                if !value.is_finite() {
                    return Err(R4SoftmaxTraceObservabilityError::Trace(
                        "final-layer Q/K/V contains a non-finite value".to_owned(),
                    ));
                }
                values.push(f64::from(value));
            }
        }
    }
    if values.len() != FULL_QKV_WIDTH {
        return Err(R4SoftmaxTraceObservabilityError::Trace(
            "full Q/K/V boundary is not exactly 1728 scalars".to_owned(),
        ));
    }
    Ok(values)
}

fn reduced_ordered_qkv(
    heads: &[R4SoftmaxHeadTrace],
) -> Result<[f64; REDUCED_QKV_WIDTH], R4SoftmaxTraceObservabilityError> {
    let mut output = [0.0_f64; REDUCED_QKV_WIDTH];
    for (role_index, role) in [
        R4SoftmaxTraceReductionRole::Query,
        R4SoftmaxTraceReductionRole::Key,
        R4SoftmaxTraceReductionRole::Value,
    ]
    .into_iter()
    .enumerate()
    {
        let reduced = reduce_heads(heads, role)?;
        for lane in 0..4 {
            output[role_index * 4 + lane] = f64::from(reduced[lane]);
        }
    }
    Ok(output)
}

fn reduce_heads(
    heads: &[R4SoftmaxHeadTrace],
    role: R4SoftmaxTraceReductionRole,
) -> Result<[f32; 4], R4SoftmaxTraceObservabilityError> {
    if heads.len() != 9 {
        return Err(R4SoftmaxTraceObservabilityError::Trace(
            "reduction requires exactly nine heads".to_owned(),
        ));
    }
    let mut blocks = Vec::with_capacity(9 * 16);
    for head in heads {
        let bits = match role {
            R4SoftmaxTraceReductionRole::Query => &head.query_gauge_bits,
            R4SoftmaxTraceReductionRole::Key => &head.current_key_query_gauge_bits,
            R4SoftmaxTraceReductionRole::Value => &head.current_value_query_gauge_bits,
        };
        if bits.len() != 64 {
            return Err(R4SoftmaxTraceObservabilityError::Trace(
                "reduction requires sixteen R4 blocks per head".to_owned(),
            ));
        }
        for chunk in bits.chunks_exact(4) {
            blocks.push([
                f32::from_bits(chunk[0]),
                f32::from_bits(chunk[1]),
                f32::from_bits(chunk[2]),
                f32::from_bits(chunk[3]),
            ]);
        }
    }
    signed_reduce_final_layer_r4(role, &blocks)
        .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))
}

fn execute_all_folds(
    implementation_revision: &str,
    documents: &[Document],
) -> Result<R4SoftmaxTraceObservabilityResult, R4SoftmaxTraceObservabilityError> {
    let mut folds = Vec::with_capacity(documents.len());
    let mut projection_cache = SignedProjectionCache::default();
    for held_out in 0..documents.len() {
        folds.push(execute_fold(held_out, documents, &mut projection_cache)?);
    }
    if folds
        .iter()
        .map(|fold| fold.manifest.primary_exact_prefix_novel_events)
        .sum::<usize>()
        != EXPECTED_PRIMARY_EVENTS
        || folds
            .iter()
            .map(|fold| fold.manifest.secondary_non_bos_events)
            .sum::<usize>()
            != EXPECTED_SECONDARY_EVENTS
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "fold event census differs from the frozen 26/34 contract".to_owned(),
        ));
    }
    let aggregate_diagnostic_boundaries = aggregate_diagnostic(&folds)?;
    let aggregate_exact_boundaries = aggregate_exact(&folds)?;
    let (decision_audit, terminal, next_action) = decide_next_action(
        &folds,
        &aggregate_diagnostic_boundaries,
        &aggregate_exact_boundaries,
    )?;
    let training_non_bos_events = folds.iter().try_fold(0_u64, |total, fold| {
        total
            .checked_add(fold.manifest.training_non_bos_events as u64)
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "training event ledger overflowed".to_owned(),
                )
            })
    })?;
    let state_resets = folds.iter().try_fold(0_u64, |total, fold| {
        total
            .checked_add(fold.manifest.state_resets)
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "state reset ledger overflowed".to_owned(),
                )
            })
    })?;
    let state_observations = folds.iter().try_fold(0_u64, |total, fold| {
        let document_positions = fold
            .manifest
            .training_positions
            .checked_add(EXPECTED_DOCUMENT_POSITIONS[fold.manifest.ordinal])
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "state observation position ledger overflowed".to_owned(),
                )
            })?;
        let all_arms = u64::try_from(document_positions)
            .ok()
            .and_then(|positions| positions.checked_mul(3))
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "state observation arm ledger overflowed".to_owned(),
                )
            })?;
        total.checked_add(all_arms).ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "state observation ledger overflowed".to_owned(),
            )
        })
    })?;
    let multiply = |left: u64, right: u64, field: &str| {
        left.checked_mul(right).ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(format!(
                "per-pass work-ledger field {field} overflowed"
            ))
        })
    };
    let fold_count = u64::try_from(folds.len()).map_err(|_| {
        R4SoftmaxTraceObservabilityError::Invalid(
            "logical fold count does not fit the work ledger".to_owned(),
        )
    })?;
    let diagnostic_count = DIAGNOSTIC_BOUNDARIES.len() as u64;
    let exact_recurrent_count = (EXACT_BOUNDARIES.len() - 1) as u64;
    // Each diagnostic is fit once against true labels and once against the
    // fixed within-support label rotation. The held-label audit repeats both fits, but
    // must reproduce the same training-only identity.
    let fits_per_fold = multiply(diagnostic_count, 4, "fits_per_fold")?;
    let probe_fits = multiply(fold_count, fits_per_fold, "probe_fits")?;
    let probe_steps = multiply(probe_fits, PROBE_STEPS as u64, "probe_steps")?;
    let normal_backtracks = folds.iter().try_fold(0_u64, |total, fold| {
        fold.diagnostic_boundaries
            .iter()
            .try_fold(total, |subtotal, boundary| {
                subtotal
                    .checked_add(boundary.optimizer_backtracks)
                    .and_then(|value| {
                        value.checked_add(boundary.label_permuted_optimizer_backtracks)
                    })
                    .ok_or_else(|| {
                        R4SoftmaxTraceObservabilityError::Invalid(
                            "per-pass probe backtrack ledger overflowed".to_owned(),
                        )
                    })
            })
    })?;
    let probe_backtracks = multiply(
        normal_backtracks,
        2,
        "probe_backtracks_including_held_label_audit",
    )?;
    // Every optimizer step evaluates the objective once with a gradient and
    // at least once at a trial point; each backtrack adds one trial.
    let probe_objective_evaluations = multiply(probe_steps, 2, "probe_objectives")?
        .checked_add(probe_backtracks)
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "per-pass probe objective ledger overflowed".to_owned(),
            )
        })?;
    let training_event_fit_incidences = multiply(
        training_non_bos_events,
        fits_per_fold,
        "training_event_fit_incidences",
    )?;
    let training_alpha_score_row_evaluations = multiply(
        multiply(
            training_non_bos_events,
            exact_recurrent_count,
            "training_alpha_boundaries",
        )?,
        (RESIDUAL_ALPHA_GRID.len() * 2) as u64,
        "training_alpha_score_row_evaluations",
    )?;
    // A held row is scored by two arms at each diagnostic boundary, once at
    // the suffix, and at actual/selected alpha for each recurrent exact arm.
    // The held-label substitution audit repeats that complete scoring path.
    let held_scores_per_row = multiply(
        diagnostic_count
            .checked_mul(2)
            .and_then(|value| value.checked_add(1 + exact_recurrent_count * 2))
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "held score multiplicity overflowed".to_owned(),
                )
            })?,
        2,
        "held_scores_including_label_audit",
    )?;
    let primary_score_row_evaluations = multiply(
        EXPECTED_PRIMARY_EVENTS as u64,
        held_scores_per_row,
        "primary_score_row_evaluations",
    )?;
    let secondary_score_row_evaluations = multiply(
        EXPECTED_SECONDARY_EVENTS as u64,
        held_scores_per_row,
        "secondary_score_row_evaluations",
    )?;
    Ok(R4SoftmaxTraceObservabilityResult {
        schema: OBSERVABILITY_RESULT_SCHEMA.to_owned(),
        issue: ISSUE,
        implementation_revision: implementation_revision.to_owned(),
        predecessor_freeze_cid: PREDECESSOR_FREEZE_CID.to_owned(),
        construction_trace_bundle_cid: CONSTRUCTION_TRACE_BUNDLE_CID.to_owned(),
        construction_trace_bundle_bytes: EXPECTED_TRACE_BYTES,
        construction_document_trace_cids: EXPECTED_DOCUMENT_TRACE_CIDS
            .iter()
            .map(|cid| (*cid).to_owned())
            .collect(),
        input_audit: ObservabilityInputAudit {
            trace_bundle_reads: 1,
            predecessor_freeze_reads: 1,
            source_model_reads: 0,
            source_model_forwards: 0,
            held_out_document_13_reads: 0,
            canonical_bundle_reload_exact: true,
            nested_document_cids_exact: true,
            document_order_exact: true,
            construction_only: true,
        },
        probe_contract: ProbeContract {
            weights: PROBE_WIDTH,
            bias: false,
            regularization: "1/1024".to_owned(),
            steps: PROBE_STEPS,
            initial_rate: "1/16".to_owned(),
            maximum_backtracks_per_step: PROBE_BACKTRACKS,
            full_batch: true,
            candidate_conditioned_sketch: true,
            support_cap: SUPPORT_CAP,
            fit_loss_weighting: "raw covered Q16 mass normalized globally; targets renormalized within each row"
                .to_owned(),
            headline_loss_weighting: "raw covered Q16 mass, normalized after aggregation"
                .to_owned(),
            paired_loss_weighting: "equal held positions after per-row covered-Q16 renormalization"
                .to_owned(),
            feature_standardization: "train-only per-lane uncentered RMS; same scales applied to held rows"
                .to_owned(),
            feature_standardization_epsilon: "1e-12".to_owned(),
            signed_projection: "one BLAKE3 XOF per boundary/candidate/width; one canonical u64 mask per source; bit j is output-lane j sign; divide by sqrt(width)"
                .to_owned(),
        },
        primary_event_count: EXPECTED_PRIMARY_EVENTS,
        secondary_event_count: EXPECTED_SECONDARY_EVENTS,
        folds,
        aggregate_diagnostic_boundaries,
        aggregate_exact_boundaries,
        decision_audit,
        work_ledger: WorkLedger {
            logical_folds: fold_count,
            execution_passes: 1,
            replay_passes: 0,
            fold_executions: fold_count,
            suffix_compiles: fold_count,
            state_compiles: fold_count,
            probe_fits,
            probe_steps,
            probe_backtracks,
            probe_objective_evaluations,
            held_label_substitution_audits: fold_count,
            training_event_fit_incidences,
            training_alpha_score_row_evaluations,
            primary_score_row_evaluations,
            secondary_score_row_evaluations,
            state_observations,
            state_resets,
        },
        exact_replay: false,
        terminal,
        next_action,
        result_cid: String::new(),
        nonclaims: vec![
            "This is a four-document construction-only observability diagnostic, not held-out model promotion."
                .to_owned(),
            "Revealed document 13 is excluded and cannot qualify any repair.".to_owned(),
            "No result here establishes coherent generation, reasoning, scale, exact lowering, WASM, or release readiness."
                .to_owned(),
            "Boundary-specific signed projections remain fixed but do not eliminate all cross-boundary sketch-variance confounding."
                .to_owned(),
        ],
    })
}

fn execute_fold(
    held_out: usize,
    documents: &[Document],
    projection_cache: &mut SignedProjectionCache,
) -> Result<FoldResult, R4SoftmaxTraceObservabilityError> {
    let training_indices = (0..documents.len())
        .filter(|index| *index != held_out)
        .collect::<Vec<_>>();
    if training_indices.len() != 3 || documents[held_out].id == "13" {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "LODO fold is not a disjoint three-train/one-held split".to_owned(),
        ));
    }
    let training_sequences = training_indices
        .iter()
        .map(|&index| trace_sequence(&documents[index]))
        .collect::<Vec<_>>();
    let suffix = compile_r4_softmax_trace_student(
        R4SoftmaxTraceStudentConfig::new(SUPPORT_CAP)
            .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?,
        &training_sequences,
    )
    .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
    if suffix.candidate_cap() as usize != SUPPORT_CAP
        || suffix.construction_document_count() != 3
        || suffix.construction_position_count()
            != training_sequences
                .iter()
                .map(|sequence| sequence.input_tokens.len() as u64)
                .sum::<u64>()
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "fold suffix artifact census or support cap is invalid".to_owned(),
        ));
    }
    let fit_sequences = training_indices
        .iter()
        .map(|&index| state_fit_sequence(&documents[index]))
        .collect::<Vec<_>>();
    let state = compile_r4_softmax_trace_state_student(
        R4SoftmaxTraceStateFitConfig {
            maximum_token_id: MAXIMUM_TOKEN_ID,
        },
        &suffix,
        &fit_sequences,
    )
    .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
    if state.construction_document_count() != 3
        || state.construction_position_count() != suffix.construction_position_count()
        || state.suffix_artifact().artifact_cid() != suffix.artifact_cid()
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "fold state artifact imports data outside its three training documents".to_owned(),
        ));
    }

    let mut selected_indices = training_indices.clone();
    selected_indices.push(held_out);
    let rows = collect_rows(
        &selected_indices,
        &training_indices,
        documents,
        &suffix,
        &state,
        projection_cache,
    )?;
    let train_rows = rows
        .iter()
        .filter(|row| {
            row.position != 0
                && training_indices
                    .iter()
                    .any(|index| documents[*index].id == row.document_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    let secondary_rows = rows
        .iter()
        .filter(|row| row.position != 0 && row.document_id == documents[held_out].id)
        .cloned()
        .collect::<Vec<_>>();
    let primary_rows = secondary_rows
        .iter()
        .filter(|row| row.exact_prefix_novel)
        .cloned()
        .collect::<Vec<_>>();
    if primary_rows.len() != EXPECTED_PRIMARY_PER_FOLD[held_out]
        || secondary_rows.len() != EXPECTED_SECONDARY_PER_FOLD[held_out]
        || train_rows.len()
            != training_indices
                .iter()
                .map(|index| documents[*index].events.len() - 1)
                .sum::<usize>()
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
            "fold {} event census violates the frozen primary/secondary contract",
            documents[held_out].id
        )));
    }

    let label_control = build_label_control_plan(
        train_rows
            .iter()
            .map(|row| (row.document_id.clone(), row.position)),
    )?;
    let label_control_mass_audit = label_control_mass_audit(&train_rows, &label_control)?;

    let mut diagnostics = Vec::with_capacity(DIAGNOSTIC_BOUNDARIES.len());
    for boundary in DIAGNOSTIC_BOUNDARIES {
        diagnostics.push(run_diagnostic_boundary(
            boundary,
            &train_rows,
            &primary_rows,
            &secondary_rows,
            &label_control,
        )?);
    }
    let mut exact = Vec::with_capacity(EXACT_BOUNDARIES.len());
    for boundary in EXACT_BOUNDARIES {
        exact.push(run_exact_boundary(
            boundary,
            &train_rows,
            &primary_rows,
            &secondary_rows,
        )?);
    }

    let training_support_cid = support_identity_cid(&train_rows)?;
    let fit_identity_cid = training_fit_identity_cid(
        &suffix.artifact_cid(),
        &state.artifact_cid(),
        &training_support_cid,
        &label_control.mapping_cid,
        &diagnostics,
        &exact,
    )?;
    let held_labels_original_cid = held_labels_cid(&secondary_rows)?;
    let mut substituted_secondary = secondary_rows.clone();
    let substituted_events = substitute_held_labels(&mut substituted_secondary)?;
    let substituted_primary = substituted_secondary
        .iter()
        .filter(|row| row.exact_prefix_novel)
        .cloned()
        .collect::<Vec<_>>();
    let mut substituted_diagnostics = Vec::with_capacity(DIAGNOSTIC_BOUNDARIES.len());
    for boundary in DIAGNOSTIC_BOUNDARIES {
        substituted_diagnostics.push(run_diagnostic_boundary(
            boundary,
            &train_rows,
            &substituted_primary,
            &substituted_secondary,
            &label_control,
        )?);
    }
    let mut substituted_exact = Vec::with_capacity(EXACT_BOUNDARIES.len());
    for boundary in EXACT_BOUNDARIES {
        substituted_exact.push(run_exact_boundary(
            boundary,
            &train_rows,
            &substituted_primary,
            &substituted_secondary,
        )?);
    }
    let substituted_fit_identity_cid = training_fit_identity_cid(
        &suffix.artifact_cid(),
        &state.artifact_cid(),
        &support_identity_cid(&train_rows)?,
        &label_control.mapping_cid,
        &substituted_diagnostics,
        &substituted_exact,
    )?;
    if substituted_fit_identity_cid != fit_identity_cid {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "held-label substitution changed the training-fit identity".to_owned(),
        ));
    }
    let held_labels_substituted_cid = held_labels_cid(&substituted_secondary)?;
    if held_labels_substituted_cid == held_labels_original_cid {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "held-label substitution did not change its bound label CID".to_owned(),
        ));
    }

    let support_cid = support_identity_cid(&rows)?;
    let candidate_support_maximum = rows
        .iter()
        .map(|row| row.candidates.len())
        .max()
        .unwrap_or(0);
    if candidate_support_maximum == 0 || candidate_support_maximum > SUPPORT_CAP {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "fold candidate support is empty or exceeds 32".to_owned(),
        ));
    }
    let training_positions = training_indices
        .iter()
        .map(|index| documents[*index].events.len())
        .sum::<usize>();
    let training_non_bos_events = training_indices
        .iter()
        .map(|index| documents[*index].events.len() - 1)
        .sum::<usize>();
    let fold_input_cid = canonical_json_cid(&serde_json::json!({
        "schema": "uor-r4.r4-softmax-trace-observability-fold-input/1",
        "held_out_document_id": documents[held_out].id,
        "held_out_document_trace_cid": documents[held_out].trace_cid,
        "held_out_document_text_cid": documents[held_out].text_cid,
        "training_document_ids": training_indices.iter().map(|index| documents[*index].id.as_str()).collect::<Vec<_>>(),
        "training_document_trace_cids": training_indices.iter().map(|index| documents[*index].trace_cid.as_str()).collect::<Vec<_>>(),
        "training_document_text_cids": training_indices.iter().map(|index| documents[*index].text_cid.as_str()).collect::<Vec<_>>(),
        "trace_bundle_cid": CONSTRUCTION_TRACE_BUNDLE_CID,
        "predecessor_freeze_cid": PREDECESSOR_FREEZE_CID,
        "support_cap": SUPPORT_CAP,
        "full_scalar_order": "role-major(Q,K,V)/head-major(0..9)/lane-major(0..64)",
        "sketch_domain": "uor-r4/1012/candidate-signed-projection-xof-u64/v1",
    }))?;
    Ok(FoldResult {
        manifest: FoldManifest {
            ordinal: held_out,
            held_out_document_id: documents[held_out].id.clone(),
            held_out_document_trace_cid: documents[held_out].trace_cid.clone(),
            training_document_ids: training_indices
                .iter()
                .map(|index| documents[*index].id.clone())
                .collect(),
            training_document_trace_cids: training_indices
                .iter()
                .map(|index| documents[*index].trace_cid.clone())
                .collect(),
            training_positions,
            training_non_bos_events,
            primary_exact_prefix_novel_events: primary_rows.len(),
            secondary_non_bos_events: secondary_rows.len(),
            fold_input_cid,
            support_artifact_cid: suffix.artifact_cid(),
            state_artifact_cid: state.artifact_cid(),
            training_support_cid,
            label_control_mapping_cid: label_control.mapping_cid,
            label_control_mass_audit,
            training_fit_identity_cid: fit_identity_cid,
            held_label_substitution_fit_identity_exact: true,
            held_label_substitution_mutated_events: substituted_events,
            held_labels_original_cid,
            held_labels_substituted_cid,
            held_out_events_used_for_fit: 0,
            state_resets: 12,
        },
        diagnostic_boundaries: diagnostics,
        exact_boundaries: exact,
        candidate_support_cid: support_cid,
        candidate_support_rows: rows.len(),
        candidate_support_maximum,
        candidate_support_train_only: true,
        runtime_source_reads: 0,
        runtime_future_reads: 0,
    })
}

fn training_fit_identity_cid(
    suffix_artifact_cid: &str,
    state_artifact_cid: &str,
    training_support_cid: &str,
    label_control_mapping_cid: &str,
    diagnostics: &[DiagnosticBoundaryResult],
    exact: &[ExactBoundaryResult],
) -> Result<String, R4SoftmaxTraceObservabilityError> {
    canonical_json_cid(&serde_json::json!({
        "schema": "uor-r4.r4-softmax-trace-observability-training-fit-identity/1",
        "suffix_artifact_cid": suffix_artifact_cid,
        "state_artifact_cid": state_artifact_cid,
        "training_support_cid": training_support_cid,
        "label_control_mapping_cid": label_control_mapping_cid,
        "boundaries": diagnostics.iter().map(|boundary| serde_json::json!({
            "boundary": boundary.boundary,
            "standardization_stats_cid": boundary.feature_audit.standardization_stats_cid,
            "training_feature_cid": boundary.feature_audit.feature_cid,
            "real_weight_cid": boundary.fitted_weight_cid,
            "control_weight_cid": boundary.label_permuted_weight_cid,
            "label_control_mapping_cid": boundary.label_control_mapping_cid,
        })).collect::<Vec<_>>(),
        "exact_train_only": exact.iter().map(|boundary| serde_json::json!({
            "boundary": boundary.boundary,
            "fitted_parameter_values": boundary.fitted_parameter_values,
            "alpha_trials": boundary.residual_scale_diagnostic.as_ref().map(|diagnostic| &diagnostic.trials),
            "selected_alpha": boundary.residual_scale_diagnostic.as_ref().map(|diagnostic| diagnostic.selected_alpha.as_str()),
        })).collect::<Vec<_>>(),
    }))
}

fn label_control_mass_audit(
    rows: &[Row],
    plan: &LabelControlPlan,
) -> Result<LabelControlMassAudit, R4SoftmaxTraceObservabilityError> {
    let identities = rows
        .iter()
        .map(|row| (row.document_id.clone(), row.position))
        .collect::<BTreeSet<_>>();
    if identities.len() != rows.len() || identities != plan.row_identities {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "label-control mass audit identities differ from the frozen plan".to_owned(),
        ));
    }
    let mut original_covered_mass_q16 = 0_u64;
    let mut permuted_covered_mass_q16 = 0_u64;
    let mut invalid_support_rows = 0_u64;
    let mut unchanged_label_rows = 0_u64;
    for row in rows {
        let original = row
            .candidates
            .iter()
            .map(|candidate| candidate.teacher_q16)
            .collect::<Vec<_>>();
        if original.len() < 2 || original.iter().all(|weight| *weight == 0) {
            invalid_support_rows += 1;
            continue;
        }
        let mut permuted = original.clone();
        permuted.rotate_left(1);
        unchanged_label_rows += u64::from(permuted == original);
        let original_mass = original
            .iter()
            .map(|weight| u64::from(*weight))
            .sum::<u64>();
        let permuted_mass = permuted
            .iter()
            .map(|weight| u64::from(*weight))
            .sum::<u64>();
        original_covered_mass_q16 = original_covered_mass_q16
            .checked_add(original_mass)
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "label-control original-mass ledger overflowed".to_owned(),
                )
            })?;
        permuted_covered_mass_q16 = permuted_covered_mass_q16
            .checked_add(permuted_mass)
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "label-control permuted-mass ledger overflowed".to_owned(),
                )
            })?;
    }
    if invalid_support_rows != 0
        || unchanged_label_rows != 0
        || original_covered_mass_q16 != permuted_covered_mass_q16
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "fixed within-support label rotation is invalid, unchanged, or not mass-preserving"
                .to_owned(),
        ));
    }
    Ok(LabelControlMassAudit {
        rows: rows.len() as u64,
        original_covered_mass_q16,
        permuted_covered_mass_q16,
        mass_preserved_exactly: true,
        invalid_support_rows,
        unchanged_label_rows,
    })
}

fn rotate_control_labels(labels: &[u16]) -> Result<Vec<u16>, R4SoftmaxTraceObservabilityError> {
    if labels.len() < 2 || labels.iter().all(|weight| *weight == 0) {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "fixed label rotation requires at least two supported slots and nonzero mass"
                .to_owned(),
        ));
    }
    let mut permuted = labels.to_vec();
    permuted.rotate_left(1);
    if permuted == labels {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "fixed label rotation leaves the row unchanged".to_owned(),
        ));
    }
    Ok(permuted)
}

fn held_labels_cid(rows: &[Row]) -> Result<String, R4SoftmaxTraceObservabilityError> {
    canonical_json_cid(&serde_json::json!({
        "schema": "uor-r4.r4-softmax-trace-observability-held-labels/1",
        "rows": rows.iter().map(|row| serde_json::json!({
            "document_id": row.document_id,
            "position": row.position,
            "actual_next_token": row.actual_next_token,
            "teacher_top32": row.teacher_top32_q16,
        })).collect::<Vec<_>>(),
    }))
}

fn substitute_held_labels(rows: &mut [Row]) -> Result<u64, R4SoftmaxTraceObservabilityError> {
    let mut mutated = 0_u64;
    for row in rows {
        if row.teacher_top32_q16.len() < 2 {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "held-label substitution requires at least two recorded teacher labels".to_owned(),
            ));
        }
        let original = row.teacher_top32_q16.clone();
        let mut weights = original.values().copied().collect::<Vec<_>>();
        weights.rotate_right(1);
        if weights == original.values().copied().collect::<Vec<_>>() {
            weights.reverse();
        }
        if weights == original.values().copied().collect::<Vec<_>>() {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "held teacher distribution cannot be mutated deterministically".to_owned(),
            ));
        }
        for ((_, weight), replacement) in row.teacher_top32_q16.iter_mut().zip(weights) {
            *weight = replacement;
        }
        row.teacher_top_token = row
            .teacher_top32_q16
            .iter()
            .max_by(|(left_token, left_weight), (right_token, right_weight)| {
                left_weight
                    .cmp(right_weight)
                    .then_with(|| right_token.cmp(left_token))
            })
            .map(|(token, _)| *token)
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "mutated teacher distribution is empty".to_owned(),
                )
            })?;
        for candidate in &mut row.candidates {
            candidate.teacher_q16 = row
                .teacher_top32_q16
                .get(&candidate.token)
                .copied()
                .unwrap_or(0);
        }
        if row
            .candidates
            .iter()
            .all(|candidate| candidate.teacher_q16 == 0)
        {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "held-label substitution removed all measured support overlap".to_owned(),
            ));
        }
        row.actual_next_token = if row.actual_next_token == MAXIMUM_TOKEN_ID {
            0
        } else {
            row.actual_next_token + 1
        };
        if row.teacher_top32_q16 == original {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "held-label substitution left teacher labels unchanged".to_owned(),
            ));
        }
        mutated += 1;
    }
    Ok(mutated)
}

fn trace_sequence(document: &Document) -> R4SoftmaxTraceSequence {
    R4SoftmaxTraceSequence::new(
        document.id.clone(),
        document.input_tokens.clone(),
        document
            .events
            .iter()
            .map(|event| event.actual_next_token)
            .collect(),
        document
            .events
            .iter()
            .map(|event| event.teacher.clone())
            .collect(),
    )
}

fn state_fit_sequence(document: &Document) -> R4SoftmaxTraceStateFitSequence {
    R4SoftmaxTraceStateFitSequence {
        document_id: document.id.clone(),
        events: document
            .events
            .iter()
            .map(|event| R4SoftmaxTraceStateFitEvent {
                position: event.position as u32,
                observed_token: event.observed_token,
                actual_next_token: event.actual_next_token,
                frame_table_offset: event.frame_table_offset,
                query_trace_r4: [
                    event.reduced_qkv[0] as f32,
                    event.reduced_qkv[1] as f32,
                    event.reduced_qkv[2] as f32,
                    event.reduced_qkv[3] as f32,
                ],
                key_trace_r4: [
                    event.reduced_qkv[4] as f32,
                    event.reduced_qkv[5] as f32,
                    event.reduced_qkv[6] as f32,
                    event.reduced_qkv[7] as f32,
                ],
                value_trace_r4: [
                    event.reduced_qkv[8] as f32,
                    event.reduced_qkv[9] as f32,
                    event.reduced_qkv[10] as f32,
                    event.reduced_qkv[11] as f32,
                ],
                teacher_top_distribution: event.teacher.clone(),
            })
            .collect(),
    }
}

fn collect_rows(
    selected_indices: &[usize],
    training_indices: &[usize],
    documents: &[Document],
    suffix: &R4SoftmaxTraceStudentArtifact,
    state: &R4SoftmaxTraceStateStudentArtifact,
    projection_cache: &mut SignedProjectionCache,
) -> Result<Vec<Row>, R4SoftmaxTraceObservabilityError> {
    let mut rows = Vec::new();
    for &document_index in selected_indices {
        let document = &documents[document_index];
        let mut plain = state
            .runtime(R4SoftmaxTraceStateArm::PlainRecurrent)
            .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
        let mut geometric = state
            .runtime(R4SoftmaxTraceStateArm::GeometricRecurrent)
            .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
        let mut permuted = state
            .runtime(R4SoftmaxTraceStateArm::TransportPermutedControl)
            .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
        let mut history = Vec::new();
        for event in &document.events {
            history.push(event.observed_token);
            let support = suffix
                .runtime()
                .distribution(&history, R4SoftmaxTraceStudentArm::TeacherDistilled)
                .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
            let plain_prediction = plain
                .observe_and_predict(event.observed_token, event.frame_table_offset)
                .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
            let geometric_prediction = geometric
                .observe_and_predict(event.observed_token, event.frame_table_offset)
                .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
            let permuted_prediction = permuted
                .observe_and_predict(event.observed_token, event.frame_table_offset)
                .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
            let plain_snapshot = plain
                .diagnostic_readout_snapshot()
                .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
            let geometric_snapshot = geometric
                .diagnostic_readout_snapshot()
                .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
            let permuted_snapshot = permuted
                .diagnostic_readout_snapshot()
                .map_err(|error| R4SoftmaxTraceObservabilityError::Student(error.to_string()))?;
            let support_tokens = support
                .scores
                .iter()
                .map(|candidate| candidate.token)
                .collect::<Vec<_>>();
            for (snapshot, prediction) in [
                (&plain_snapshot, &plain_prediction),
                (&geometric_snapshot, &geometric_prediction),
                (&permuted_snapshot, &permuted_prediction),
            ] {
                let snapshot_tokens = snapshot
                    .candidates
                    .iter()
                    .map(|candidate| candidate.token)
                    .collect::<Vec<_>>();
                let prediction_tokens = prediction
                    .candidates
                    .iter()
                    .map(|candidate| candidate.token)
                    .collect::<Vec<_>>();
                if snapshot_tokens != support_tokens
                    || prediction_tokens != support_tokens
                    || snapshot.suffix_depth != support.suffix_depth
                    || snapshot.state_checksum != prediction.state_checksum
                {
                    return Err(R4SoftmaxTraceObservabilityError::Invalid(
                        "fold boundaries do not share exact train-only support/state".to_owned(),
                    ));
                }
            }
            let teacher_top32_q16 = event
                .teacher
                .entries
                .iter()
                .map(|entry| (entry.token, entry.probability_q16))
                .collect::<BTreeMap<_, _>>();
            if teacher_top32_q16.len() != event.teacher.entries.len() {
                return Err(R4SoftmaxTraceObservabilityError::Invalid(
                    "recorded teacher top-32 contains a duplicate token".to_owned(),
                ));
            }
            let mut candidates = Vec::with_capacity(support.scores.len());
            for (candidate_index, suffix_candidate) in support.scores.iter().enumerate() {
                let token = suffix_candidate.token;
                let teacher_q16 = event
                    .teacher
                    .entries
                    .iter()
                    .find(|entry| entry.token == token)
                    .map_or(0, |entry| entry.probability_q16);
                let plain_candidate = &plain_snapshot.candidates[candidate_index];
                let geometric_candidate = &geometric_snapshot.candidates[candidate_index];
                let permuted_candidate = &permuted_snapshot.candidates[candidate_index];
                for (diagnostic, predicted) in [
                    (
                        plain_candidate,
                        &plain_prediction.candidates[candidate_index],
                    ),
                    (
                        geometric_candidate,
                        &geometric_prediction.candidates[candidate_index],
                    ),
                    (
                        permuted_candidate,
                        &permuted_prediction.candidates[candidate_index],
                    ),
                ] {
                    if diagnostic.token != token
                        || predicted.token != token
                        || diagnostic.total_logit.to_bits() != predicted.logit.to_bits()
                    {
                        return Err(R4SoftmaxTraceObservabilityError::Invalid(
                            "diagnostic snapshot differs from exact runtime logits".to_owned(),
                        ));
                    }
                }
                if plain_candidate.base_logit.to_bits() != geometric_candidate.base_logit.to_bits()
                    || plain_candidate.base_logit.to_bits()
                        != permuted_candidate.base_logit.to_bits()
                {
                    return Err(R4SoftmaxTraceObservabilityError::Invalid(
                        "matched recurrent arms expose different exact suffix base logits"
                            .to_owned(),
                    ));
                }
                let suffix_logit = f64::from(plain_candidate.base_logit);
                candidates.push(CandidateView {
                    token,
                    teacher_q16,
                    full_features: projection_cache.project(
                        "full_final_layer_qkv_signed_sketch",
                        token,
                        &event.full_qkv,
                    )?,
                    reduced_features: projection_cache.project(
                        "signed_576_to_4_qkv_sketch",
                        token,
                        &event.reduced_qkv,
                    )?,
                    plain_features: plain_candidate.readout_features.map(f64::from),
                    geometric_features: geometric_candidate.readout_features.map(f64::from),
                    permuted_features: permuted_candidate.readout_features.map(f64::from),
                    suffix_logit,
                    plain_residual_logit: f64::from(plain_candidate.residual_logit),
                    plain_logit: f64::from(plain_candidate.total_logit),
                    geometric_residual_logit: f64::from(geometric_candidate.residual_logit),
                    geometric_logit: f64::from(geometric_candidate.total_logit),
                    permuted_residual_logit: f64::from(permuted_candidate.residual_logit),
                    permuted_logit: f64::from(permuted_candidate.total_logit),
                });
            }
            if candidates.is_empty()
                || candidates.len() > SUPPORT_CAP
                || candidates
                    .iter()
                    .all(|candidate| candidate.teacher_q16 == 0)
            {
                return Err(R4SoftmaxTraceObservabilityError::Invalid(
                    "fold row has invalid candidate support or zero teacher overlap".to_owned(),
                ));
            }
            let prefix = &document.input_tokens[..=event.position];
            let exact_prefix_novel = event.position != 0
                && training_indices.iter().all(|index| {
                    let training = &documents[*index].input_tokens;
                    training.len() < prefix.len() || training[..prefix.len()] != *prefix
                });
            rows.push(Row {
                document_id: document.id.clone(),
                position: event.position,
                actual_next_token: event.actual_next_token,
                teacher_top_token: event.teacher.top_token().map_err(|error| {
                    R4SoftmaxTraceObservabilityError::Student(error.to_string())
                })?,
                teacher_top32_q16,
                candidates,
                exact_prefix_novel,
            });
        }
        validate_runtime_audit(document, &plain, &geometric, &permuted)?;
    }
    Ok(rows)
}

fn validate_runtime_audit(
    document: &Document,
    plain: &uor_r4_core::r4_softmax_trace_state_student::R4SoftmaxTraceStateRuntime,
    geometric: &uor_r4_core::r4_softmax_trace_state_student::R4SoftmaxTraceStateRuntime,
    permuted: &uor_r4_core::r4_softmax_trace_state_student::R4SoftmaxTraceStateRuntime,
) -> Result<(), R4SoftmaxTraceObservabilityError> {
    let observations = document.events.len() as u64;
    for runtime in [plain, geometric, permuted] {
        let audit = runtime.audit();
        if audit.observed_token_reads != observations
            || audit.prior_state_reads != observations
            || audit.canonical_frame_reads != observations
            || audit.artifact_reads != observations
            || audit.source_model_forwards != 0
            || audit.source_trace_reads != 0
            || audit.teacher_distribution_reads != 0
            || audit.target_reads != 0
            || audit.future_token_reads != 0
        {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "state feature extraction violated the causal runtime audit".to_owned(),
            ));
        }
    }
    let expected_transports = observations.saturating_sub(1);
    if plain.audit().state_transports != 0
        || plain.audit().transport_permutations != 0
        || geometric.audit().state_transports != expected_transports
        || geometric.audit().transport_permutations != 0
        || permuted.audit().state_transports != expected_transports
        || permuted.audit().transport_permutations != expected_transports
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "plain/geometric/permuted work ledgers are not the exact matched contract".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
fn signed_sketch<const N: usize>(
    boundary: &str,
    candidate: u32,
    values: &[f64; N],
) -> Result<[f64; PROBE_WIDTH], R4SoftmaxTraceObservabilityError> {
    signed_sketch_slice(boundary, candidate, values)
}

#[cfg(test)]
fn signed_sketch_slice(
    boundary: &str,
    candidate: u32,
    values: &[f64],
) -> Result<[f64; PROBE_WIDTH], R4SoftmaxTraceObservabilityError> {
    SignedProjectionCache::default().project(boundary, candidate, values)
}

fn run_diagnostic_boundary(
    boundary: &str,
    training: &[Row],
    primary: &[Row],
    secondary: &[Row],
    label_control: &LabelControlPlan,
) -> Result<DiagnosticBoundaryResult, R4SoftmaxTraceObservabilityError> {
    if !DIAGNOSTIC_BOUNDARIES.contains(&boundary) {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
            "unknown diagnostic boundary {boundary}"
        )));
    }
    let mut training_rows = probe_rows(boundary, training)?;
    let mut primary_rows = probe_rows(boundary, primary)?;
    let mut secondary_rows = probe_rows(boundary, secondary)?;
    let standardization = fit_and_apply_feature_standardization(
        boundary,
        &mut training_rows,
        &mut primary_rows,
        &mut secondary_rows,
    )?;
    let feature_audit = audit_features(boundary, &training_rows, &standardization)?;
    if !feature_audit.all_finite
        || feature_audit.active_lanes == 0
        || feature_audit.total_variance <= 0.0
        || feature_audit.candidate_feature_rows == 0
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
            "diagnostic boundary {boundary} failed its finite/liveness gate"
        )));
    }

    let fitted = fit_probe(&training_rows, None)?;
    let permuted = fit_probe(&training_rows, Some(label_control))?;
    if fitted.changed_label_rows != 0 || permuted.changed_label_rows == 0 {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
            "diagnostic boundary {boundary} did not construct a live label control"
        )));
    }
    let (primary_metrics, primary_positions) = score_probe_rows(&primary_rows, &fitted.weights)?;
    let (primary_control, primary_control_positions) =
        score_probe_rows(&primary_rows, &permuted.weights)?;
    let (secondary_metrics, secondary_positions) =
        score_probe_rows(&secondary_rows, &fitted.weights)?;
    let (secondary_control, secondary_control_positions) =
        score_probe_rows(&secondary_rows, &permuted.weights)?;

    Ok(DiagnosticBoundaryResult {
        boundary: boundary.to_owned(),
        feature_audit,
        fitted_weight_cid: weight_cid(boundary, "real", &fitted.weights)?,
        label_permuted_weight_cid: weight_cid(boundary, "label-permuted", &permuted.weights)?,
        label_control_mapping_cid: label_control.mapping_cid.clone(),
        label_permuted_training_rows_changed: permuted.changed_label_rows,
        optimizer_backtracks: fitted.backtracks,
        label_permuted_optimizer_backtracks: permuted.backtracks,
        primary: primary_metrics,
        primary_label_permuted: primary_control,
        primary_paired: paired_uncertainty(&primary_positions, &primary_control_positions)?,
        secondary: secondary_metrics,
        secondary_label_permuted: secondary_control,
        secondary_paired: paired_uncertainty(&secondary_positions, &secondary_control_positions)?,
    })
}

fn probe_rows(
    boundary: &str,
    rows: &[Row],
) -> Result<Vec<ProbeRow>, R4SoftmaxTraceObservabilityError> {
    rows.iter()
        .map(|row| {
            if row.candidates.is_empty() || row.candidates.len() > SUPPORT_CAP {
                return Err(R4SoftmaxTraceObservabilityError::Invalid(
                    "probe row has an invalid candidate support".to_owned(),
                ));
            }
            let features = row
                .candidates
                .iter()
                .map(|candidate| match boundary {
                    "full_final_layer_qkv_signed_sketch" => Ok(candidate.full_features),
                    "signed_576_to_4_qkv_sketch" => Ok(candidate.reduced_features),
                    "plain_natural_state_tensor" => Ok(candidate.plain_features),
                    "geometric_natural_state_tensor" => Ok(candidate.geometric_features),
                    "transport_permuted_natural_state_tensor" => Ok(candidate.permuted_features),
                    _ => Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                        "unknown diagnostic boundary {boundary}"
                    ))),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ProbeRow {
                document_id: row.document_id.clone(),
                position: row.position,
                actual_next_token: row.actual_next_token,
                teacher_top_token: row.teacher_top_token,
                teacher_top32_q16: row.teacher_top32_q16.clone(),
                tokens: row
                    .candidates
                    .iter()
                    .map(|candidate| candidate.token)
                    .collect(),
                teacher_q16: row
                    .candidates
                    .iter()
                    .map(|candidate| candidate.teacher_q16)
                    .collect(),
                base_logits: row
                    .candidates
                    .iter()
                    .map(|candidate| candidate.suffix_logit)
                    .collect(),
                features,
            })
        })
        .collect()
}

fn fit_and_apply_feature_standardization(
    boundary: &str,
    training: &mut [ProbeRow],
    primary: &mut [ProbeRow],
    secondary: &mut [ProbeRow],
) -> Result<FeatureStandardization, R4SoftmaxTraceObservabilityError> {
    let candidate_rows = training.iter().map(|row| row.features.len()).sum::<usize>();
    if candidate_rows == 0 {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "feature standardization has no training candidate rows".to_owned(),
        ));
    }
    let mut rms = [0.0_f64; PROBE_WIDTH];
    for row in training.iter() {
        for features in &row.features {
            for lane in 0..PROBE_WIDTH {
                rms[lane] += features[lane] * features[lane];
            }
        }
    }
    for value in &mut rms {
        *value = (*value / candidate_rows as f64).sqrt();
        if !value.is_finite() {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "feature RMS is non-finite".to_owned(),
            ));
        }
    }
    let stats_cid = canonical_json_cid(&serde_json::json!({
        "schema": "uor-r4.r4-softmax-trace-observability-feature-rms/1",
        "boundary": boundary,
        "training_candidate_rows": candidate_rows,
        "epsilon_f64_bits": PROBE_RMS_EPSILON.to_bits(),
        "rms_f64_bits": rms.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
    }))?;
    apply_feature_standardization(training, &rms);
    apply_feature_standardization(primary, &rms);
    apply_feature_standardization(secondary, &rms);
    let mut standardized_rms = [0.0_f64; PROBE_WIDTH];
    for row in training.iter() {
        for features in &row.features {
            for lane in 0..PROBE_WIDTH {
                standardized_rms[lane] += features[lane] * features[lane];
            }
        }
    }
    let unit_rms_lanes = standardized_rms
        .iter()
        .filter(|sum| {
            let lane_rms = (**sum / candidate_rows as f64).sqrt();
            (lane_rms - 1.0).abs() <= 1.0e-9
        })
        .count();
    if rms.iter().any(|value| *value > PROBE_RMS_EPSILON) && unit_rms_lanes == 0 {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "feature standardization did not produce any unit-RMS lane".to_owned(),
        ));
    }
    Ok(FeatureStandardization {
        stats_cid,
        unit_rms_lanes,
    })
}

fn apply_feature_standardization(rows: &mut [ProbeRow], rms: &[f64; PROBE_WIDTH]) {
    for row in rows {
        for features in &mut row.features {
            for lane in 0..PROBE_WIDTH {
                features[lane] /= rms[lane].max(PROBE_RMS_EPSILON);
            }
        }
    }
}

fn audit_features(
    boundary: &str,
    rows: &[ProbeRow],
    standardization: &FeatureStandardization,
) -> Result<FeatureAudit, R4SoftmaxTraceObservabilityError> {
    let raw_width = match boundary {
        "full_final_layer_qkv_signed_sketch" => FULL_QKV_WIDTH,
        "signed_576_to_4_qkv_sketch" => REDUCED_QKV_WIDTH,
        "plain_natural_state_tensor"
        | "geometric_natural_state_tensor"
        | "transport_permuted_natural_state_tensor" => PROBE_WIDTH,
        _ => {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                "unknown diagnostic boundary {boundary}"
            )))
        }
    };
    let mut all = Vec::new();
    let mut zero_overlap_rows = 0_usize;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4/1012/target-blind-feature-cid/v1");
    hasher.update(boundary.as_bytes());
    for row in rows {
        if row.teacher_q16.iter().all(|weight| *weight == 0) {
            zero_overlap_rows += 1;
        }
        hasher.update(&(row.document_id.len() as u64).to_le_bytes());
        hasher.update(row.document_id.as_bytes());
        hasher.update(&(row.position as u64).to_le_bytes());
        for (&token, features) in row.tokens.iter().zip(&row.features) {
            hasher.update(&token.to_le_bytes());
            for feature in features {
                hasher.update(&feature.to_bits().to_le_bytes());
            }
            all.push(*features);
        }
    }
    let all_finite = all
        .iter()
        .all(|features| features.iter().all(|value| value.is_finite()));
    if !all_finite {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
            "diagnostic boundary {boundary} contains non-finite features"
        )));
    }
    let mut unique = BTreeSet::new();
    for features in &all {
        unique.insert(features.map(f64::to_bits));
    }
    let mut means = [0.0_f64; PROBE_WIDTH];
    for features in &all {
        for (mean, feature) in means.iter_mut().zip(features) {
            *mean += *feature;
        }
    }
    if !all.is_empty() {
        for mean in &mut means {
            *mean /= all.len() as f64;
        }
    }
    let mut variances = [0.0_f64; PROBE_WIDTH];
    for features in &all {
        for lane in 0..PROBE_WIDTH {
            let centered = features[lane] - means[lane];
            variances[lane] += centered * centered;
        }
    }
    if !all.is_empty() {
        for variance in &mut variances {
            *variance /= all.len() as f64;
        }
    }
    let active_lanes = variances.iter().filter(|variance| **variance > 0.0).count();
    let total_variance = variances.iter().sum();
    Ok(FeatureAudit {
        raw_width,
        probe_width: PROBE_WIDTH,
        fitted_weights: PROBE_WIDTH,
        bias: false,
        candidate_feature_rows: all.len(),
        unique_feature_rows: unique.len(),
        exact_collision_rows: all.len().saturating_sub(unique.len()),
        zero_overlap_rows,
        active_lanes,
        numerical_rank: numerical_rank(&all, means),
        total_variance,
        all_finite,
        target_blind_preprocessing: true,
        feature_cid: format!("blake3:{}", hasher.finalize().to_hex()),
        standardization_stats_cid: standardization.stats_cid.clone(),
        train_only_unit_rms_standardization: true,
        standardized_unit_rms_lanes: standardization.unit_rms_lanes,
    })
}

fn numerical_rank(rows: &[[f64; PROBE_WIDTH]], means: [f64; PROBE_WIDTH]) -> usize {
    if rows.is_empty() {
        return 0;
    }
    let mut gram = [[0.0_f64; PROBE_WIDTH]; PROBE_WIDTH];
    for row in rows {
        for left in 0..PROBE_WIDTH {
            let left_value = row[left] - means[left];
            for right in left..PROBE_WIDTH {
                gram[left][right] += left_value * (row[right] - means[right]);
            }
        }
    }
    for left in 0..PROBE_WIDTH {
        let (prior_rows, current_and_later) = gram.split_at_mut(left);
        let current = &mut current_and_later[0];
        for (right, prior) in prior_rows.iter().enumerate() {
            current[right] = prior[left];
        }
    }
    let scale = gram
        .iter()
        .flat_map(|row| row.iter())
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return 0;
    }
    let tolerance = scale * 1.0e-10;
    let mut rank = 0_usize;
    for column in 0..PROBE_WIDTH {
        let pivot = (rank..PROBE_WIDTH)
            .max_by(|left, right| {
                gram[*left][column]
                    .abs()
                    .total_cmp(&gram[*right][column].abs())
            })
            .unwrap_or(rank);
        if gram[pivot][column].abs() <= tolerance {
            continue;
        }
        gram.swap(rank, pivot);
        let divisor = gram[rank][column];
        for value in gram[rank].iter_mut().skip(column) {
            *value /= divisor;
        }
        let pivot_row = gram[rank];
        for (row_index, row) in gram.iter_mut().enumerate() {
            if row_index == rank {
                continue;
            }
            let factor = row[column];
            for (value, pivot) in row.iter_mut().zip(&pivot_row).skip(column) {
                *value -= factor * *pivot;
            }
        }
        rank += 1;
        if rank == PROBE_WIDTH {
            break;
        }
    }
    rank
}

fn fit_probe(
    rows: &[ProbeRow],
    label_control: Option<&LabelControlPlan>,
) -> Result<FittedProbe, R4SoftmaxTraceObservabilityError> {
    if rows.is_empty() {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "probe fit has no training rows".to_owned(),
        ));
    }
    let (fit_rows, changed_label_rows) = if let Some(plan) = label_control {
        slot_permuted_training_rows(rows, plan)?
    } else {
        (rows.to_vec(), 0)
    };
    let mut weights = [0.0_f64; PROBE_WIDTH];
    let mut backtracks = 0_u64;
    for _ in 0..PROBE_STEPS {
        let (objective, gradient) = probe_objective_and_gradient(&fit_rows, &weights)?;
        let mut accepted = None;
        for backtrack in 0..=PROBE_BACKTRACKS {
            let rate = PROBE_INITIAL_RATE / 2.0_f64.powi(backtrack as i32);
            let mut trial = weights;
            for lane in 0..PROBE_WIDTH {
                trial[lane] -= rate * gradient[lane];
            }
            let trial_objective = probe_objective(&fit_rows, &trial)?;
            if trial_objective <= objective + 1.0e-15 {
                accepted = Some((trial, backtrack));
                break;
            }
        }
        if let Some((trial, used_backtracks)) = accepted {
            weights = trial;
            backtracks = backtracks
                .checked_add(used_backtracks as u64)
                .ok_or_else(|| {
                    R4SoftmaxTraceObservabilityError::Invalid(
                        "optimizer backtrack ledger overflowed".to_owned(),
                    )
                })?;
        } else {
            backtracks = backtracks
                .checked_add(PROBE_BACKTRACKS as u64)
                .ok_or_else(|| {
                    R4SoftmaxTraceObservabilityError::Invalid(
                        "optimizer backtrack ledger overflowed".to_owned(),
                    )
                })?;
        }
    }
    if weights.iter().any(|weight| !weight.is_finite()) {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "probe fit produced a non-finite weight".to_owned(),
        ));
    }
    Ok(FittedProbe {
        weights,
        backtracks,
        changed_label_rows,
    })
}

fn slot_permuted_training_rows(
    rows: &[ProbeRow],
    plan: &LabelControlPlan,
) -> Result<(Vec<ProbeRow>, u64), R4SoftmaxTraceObservabilityError> {
    let identities = rows
        .iter()
        .map(|row| (row.document_id.clone(), row.position))
        .collect::<BTreeSet<_>>();
    if identities.len() != rows.len() || identities != plan.row_identities {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "fixed label-rotation plan and training rows differ".to_owned(),
        ));
    }
    let mut permuted = rows.to_vec();
    let mut changed = 0_u64;
    for output in &mut permuted {
        let rotated = rotate_control_labels(&output.teacher_q16)?;
        let original_mass = output
            .teacher_q16
            .iter()
            .map(|weight| u64::from(*weight))
            .sum::<u64>();
        let rotated_mass = rotated.iter().map(|weight| u64::from(*weight)).sum::<u64>();
        if original_mass != rotated_mass {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "fixed label rotation changed covered Q16 mass".to_owned(),
            ));
        }
        output.teacher_q16 = rotated;
        output.teacher_top32_q16 = output
            .tokens
            .iter()
            .copied()
            .zip(output.teacher_q16.iter().copied())
            .collect();
        output.teacher_top_token = output
            .tokens
            .iter()
            .copied()
            .zip(output.teacher_q16.iter().copied())
            .max_by(|(left_token, left_weight), (right_token, right_weight)| {
                left_weight
                    .cmp(right_weight)
                    .then_with(|| right_token.cmp(left_token))
            })
            .map(|(token, _)| token)
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "fixed label rotation has no candidate token".to_owned(),
                )
            })?;
        changed += 1;
    }
    Ok((permuted, changed))
}

fn build_label_control_plan(
    identities: impl IntoIterator<Item = (String, usize)>,
) -> Result<LabelControlPlan, R4SoftmaxTraceObservabilityError> {
    let mut canonical = identities.into_iter().collect::<Vec<_>>();
    canonical.sort_by_key(|(document_id, position)| {
        (
            EXPECTED_DOCUMENT_IDS
                .iter()
                .position(|expected| *expected == document_id)
                .unwrap_or(usize::MAX),
            *position,
        )
    });
    if canonical.len() < 2
        || canonical.windows(2).any(|window| window[0] == window[1])
        || canonical
            .iter()
            .any(|(document_id, _)| !EXPECTED_DOCUMENT_IDS.contains(&document_id.as_str()))
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "label-control identities are empty, singleton, or duplicated".to_owned(),
        ));
    }
    let document_count = canonical
        .iter()
        .map(|(document_id, _)| document_id)
        .collect::<BTreeSet<_>>()
        .len();
    if document_count != 3 {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "label-control plan requires exactly three training documents".to_owned(),
        ));
    }
    let row_identities = canonical.iter().cloned().collect::<BTreeSet<_>>();
    let mapping_cid = canonical_json_cid(&serde_json::json!({
        "schema": "uor-r4.r4-softmax-trace-observability-label-control-mapping/1",
        "policy": "within-row-candidate-slot-rotate-left-one",
        "shift": 1,
        "rows": canonical.iter().map(|identity| serde_json::json!({
            "document_id": identity.0.as_str(),
            "position": identity.1,
        })).collect::<Vec<_>>(),
    }))?;
    Ok(LabelControlPlan {
        row_identities,
        mapping_cid,
    })
}

fn probe_objective_and_gradient(
    rows: &[ProbeRow],
    weights: &[f64; PROBE_WIDTH],
) -> Result<(f64, [f64; PROBE_WIDTH]), R4SoftmaxTraceObservabilityError> {
    let mut objective = 0.0_f64;
    let mut gradient = [0.0_f64; PROBE_WIDTH];
    let total_mass = probe_total_covered_mass(rows)?;
    for row in rows {
        let mass = row
            .teacher_q16
            .iter()
            .map(|weight| u64::from(*weight))
            .sum::<u64>();
        if mass == 0 {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "probe training row has zero support overlap".to_owned(),
            ));
        }
        let row_weight = mass as f64 / total_mass as f64;
        let logits = row
            .base_logits
            .iter()
            .zip(&row.features)
            .map(|(base, features)| *base + dot(weights, features))
            .collect::<Vec<_>>();
        let probabilities = stable_softmax(&logits)?;
        for (candidate, probability) in probabilities.iter().enumerate() {
            let target = f64::from(row.teacher_q16[candidate]) / mass as f64;
            if target > 0.0 {
                objective -= row_weight * target * probability.ln();
            }
            let error = row_weight * (*probability - target);
            for (gradient, feature) in gradient.iter_mut().zip(&row.features[candidate]) {
                *gradient += error * *feature;
            }
        }
    }
    for lane in 0..PROBE_WIDTH {
        objective += 0.5 * PROBE_REGULARIZATION * weights[lane] * weights[lane];
        gradient[lane] += PROBE_REGULARIZATION * weights[lane];
    }
    if !objective.is_finite() || gradient.iter().any(|value| !value.is_finite()) {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "probe objective or gradient is non-finite".to_owned(),
        ));
    }
    Ok((objective, gradient))
}

fn probe_objective(
    rows: &[ProbeRow],
    weights: &[f64; PROBE_WIDTH],
) -> Result<f64, R4SoftmaxTraceObservabilityError> {
    let mut objective = 0.0_f64;
    let total_mass = probe_total_covered_mass(rows)?;
    for row in rows {
        let mass = row
            .teacher_q16
            .iter()
            .map(|weight| u64::from(*weight))
            .sum::<u64>();
        if mass == 0 {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "probe training row has zero support overlap".to_owned(),
            ));
        }
        let row_weight = mass as f64 / total_mass as f64;
        let logits = row
            .base_logits
            .iter()
            .zip(&row.features)
            .map(|(base, features)| *base + dot(weights, features))
            .collect::<Vec<_>>();
        let probabilities = stable_softmax(&logits)?;
        for (candidate, probability) in probabilities.iter().enumerate() {
            let target = f64::from(row.teacher_q16[candidate]) / mass as f64;
            if target > 0.0 {
                objective -= row_weight * target * probability.ln();
            }
        }
    }
    objective +=
        0.5 * PROBE_REGULARIZATION * weights.iter().map(|weight| weight * weight).sum::<f64>();
    if !objective.is_finite() {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "probe objective is non-finite".to_owned(),
        ));
    }
    Ok(objective)
}

fn probe_total_covered_mass(rows: &[ProbeRow]) -> Result<u64, R4SoftmaxTraceObservabilityError> {
    let mut total = 0_u64;
    for row in rows {
        total = total
            .checked_add(
                row.teacher_q16
                    .iter()
                    .map(|weight| u64::from(*weight))
                    .sum::<u64>(),
            )
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "probe covered-mass ledger overflowed".to_owned(),
                )
            })?;
    }
    if total == 0 {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "probe fit has zero total covered teacher mass".to_owned(),
        ));
    }
    Ok(total)
}

fn dot(left: &[f64; PROBE_WIDTH], right: &[f64; PROBE_WIDTH]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn weight_cid(
    boundary: &str,
    arm: &str,
    weights: &[f64; PROBE_WIDTH],
) -> Result<String, R4SoftmaxTraceObservabilityError> {
    canonical_json_cid(&serde_json::json!({
        "schema": "uor-r4.r4-softmax-trace-observability-probe-weights/1",
        "boundary": boundary,
        "arm": arm,
        "weights_f64_bits": weights.iter().map(|weight| weight.to_bits()).collect::<Vec<_>>(),
        "bias": false,
    }))
}

fn run_exact_boundary(
    boundary: &str,
    training: &[Row],
    primary: &[Row],
    secondary: &[Row],
) -> Result<ExactBoundaryResult, R4SoftmaxTraceObservabilityError> {
    if !EXACT_BOUNDARIES.contains(&boundary) {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
            "unknown exact boundary {boundary}"
        )));
    }
    let primary_actual = exact_rows(boundary, primary, 1.0)?;
    let secondary_actual = exact_rows(boundary, secondary, 1.0)?;
    let (primary_metrics, _) = score_exact_rows(&primary_actual)?;
    let (secondary_metrics, _) = score_exact_rows(&secondary_actual)?;
    let recurrent = boundary != "suffix_exact_base_logits";
    let residual_scale_diagnostic = if recurrent {
        Some(residual_scale_diagnostic(
            boundary, training, primary, secondary,
        )?)
    } else {
        None
    };
    Ok(ExactBoundaryResult {
        boundary: boundary.to_owned(),
        fitted_parameter_values: if recurrent {
            R4_SOFTMAX_TRACE_STATE_FITTED_VALUES_PER_ARM
        } else {
            0
        },
        actual_alpha: if recurrent { "1" } else { "0" }.to_owned(),
        primary: primary_metrics,
        secondary: secondary_metrics,
        residual_scale_diagnostic,
    })
}

const RESIDUAL_ALPHA_GRID: [(f32, &str); 10] = [
    (0.0, "0"),
    (1.0 / 16.0, "1/16"),
    (1.0 / 8.0, "1/8"),
    (1.0 / 4.0, "1/4"),
    (1.0 / 2.0, "1/2"),
    (1.0, "1"),
    (2.0, "2"),
    (4.0, "4"),
    (8.0, "8"),
    (16.0, "16"),
];

fn residual_scale_diagnostic(
    boundary: &str,
    training: &[Row],
    primary: &[Row],
    secondary: &[Row],
) -> Result<ResidualScaleDiagnostic, R4SoftmaxTraceObservabilityError> {
    let mut trials = Vec::with_capacity(RESIDUAL_ALPHA_GRID.len());
    let mut selected = None::<(f32, &str, f64)>;
    for (alpha, name) in RESIDUAL_ALPHA_GRID {
        let rows = exact_rows(boundary, training, alpha)?;
        let (metrics, _) = score_exact_rows(&rows)?;
        let cross_entropy = metrics.covered_teacher_cross_entropy_nats;
        trials.push(ResidualScaleTrial {
            alpha: name.to_owned(),
            training_covered_teacher_cross_entropy_nats: cross_entropy,
        });
        if let Some(value) = cross_entropy.filter(|value| value.is_finite()) {
            if selected.is_none_or(|(_, _, best)| value + 1.0e-12 < best) {
                selected = Some((alpha, name, value));
            }
        }
    }
    let (selected_alpha, selected_name, _) = selected.ok_or_else(|| {
        R4SoftmaxTraceObservabilityError::Invalid(format!(
            "exact boundary {boundary} has no finite train-only alpha trial"
        ))
    })?;
    let (selected_primary, _) = score_exact_rows(&exact_rows(boundary, primary, selected_alpha)?)?;
    let (selected_secondary, _) =
        score_exact_rows(&exact_rows(boundary, secondary, selected_alpha)?)?;
    Ok(ResidualScaleDiagnostic {
        trials,
        selected_alpha: selected_name.to_owned(),
        selection_uses_training_only: true,
        deterministic_tie_break_prefers_smaller_alpha: true,
        selected_primary,
        selected_secondary,
    })
}

fn exact_rows(
    boundary: &str,
    rows: &[Row],
    alpha: f32,
) -> Result<Vec<ExactRow>, R4SoftmaxTraceObservabilityError> {
    if !alpha.is_finite() || alpha < 0.0 {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "residual alpha is invalid".to_owned(),
        ));
    }
    rows.iter()
        .map(|row| {
            let mut logits = Vec::with_capacity(row.candidates.len());
            for candidate in &row.candidates {
                let logit = match boundary {
                    "suffix_exact_base_logits" => candidate.suffix_logit,
                    "plain_exact_base_plus_residual_logits" => {
                        scaled_exact_logit(candidate, alpha, "plain")?
                    }
                    "geometric_exact_base_plus_residual_logits" => {
                        scaled_exact_logit(candidate, alpha, "geometric")?
                    }
                    "transport_permuted_exact_base_plus_residual_logits" => {
                        scaled_exact_logit(candidate, alpha, "permuted")?
                    }
                    _ => {
                        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                            "unknown exact boundary {boundary}"
                        )))
                    }
                };
                if !logit.is_finite() {
                    return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                        "exact boundary {boundary} contains a non-finite logit"
                    )));
                }
                logits.push(logit);
            }
            Ok(ExactRow {
                document_id: row.document_id.clone(),
                position: row.position,
                actual_next_token: row.actual_next_token,
                teacher_top_token: row.teacher_top_token,
                tokens: row
                    .candidates
                    .iter()
                    .map(|candidate| candidate.token)
                    .collect(),
                teacher_q16: row
                    .candidates
                    .iter()
                    .map(|candidate| candidate.teacher_q16)
                    .collect(),
                logits,
            })
        })
        .collect()
}

fn scaled_exact_logit(
    candidate: &CandidateView,
    alpha: f32,
    arm: &str,
) -> Result<f64, R4SoftmaxTraceObservabilityError> {
    let (residual, actual) = match arm {
        "plain" => (candidate.plain_residual_logit, candidate.plain_logit),
        "geometric" => (
            candidate.geometric_residual_logit,
            candidate.geometric_logit,
        ),
        "permuted" => (candidate.permuted_residual_logit, candidate.permuted_logit),
        _ => {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                "unknown exact recurrent arm {arm}"
            )))
        }
    };
    let base = candidate.suffix_logit as f32;
    let residual = residual as f32;
    let scaled = f64::from(base + alpha * residual);
    if alpha.to_bits() == 1.0_f32.to_bits() && scaled.to_bits() != actual.to_bits() {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
            "alpha=1 diagnostic differs from the exact {arm} runtime logit"
        )));
    }
    Ok(scaled)
}

fn score_probe_rows(
    rows: &[ProbeRow],
    weights: &[f64; PROBE_WIDTH],
) -> Result<(MetricSummary, Vec<ScoredPosition>), R4SoftmaxTraceObservabilityError> {
    let exact = rows
        .iter()
        .map(|row| ExactRow {
            document_id: row.document_id.clone(),
            position: row.position,
            actual_next_token: row.actual_next_token,
            teacher_top_token: row.teacher_top_token,
            tokens: row.tokens.clone(),
            teacher_q16: row.teacher_q16.clone(),
            logits: row
                .base_logits
                .iter()
                .zip(&row.features)
                .map(|(base, features)| *base + dot(weights, features))
                .collect(),
        })
        .collect::<Vec<_>>();
    score_exact_rows(&exact)
}

fn score_exact_rows(
    rows: &[ExactRow],
) -> Result<(MetricSummary, Vec<ScoredPosition>), R4SoftmaxTraceObservabilityError> {
    let mut positions = Vec::with_capacity(rows.len());
    let mut total_mass = 0_u64;
    let mut total_raw_loss = 0.0_f64;
    let mut teacher_top1 = 0_u64;
    let mut actual_top1 = 0_u64;
    for row in rows {
        if row.tokens.is_empty()
            || row.tokens.len() != row.teacher_q16.len()
            || row.tokens.len() != row.logits.len()
            || row.tokens.len() > SUPPORT_CAP
        {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "scoring row shapes are invalid".to_owned(),
            ));
        }
        let probabilities = stable_softmax(&row.logits)?;
        let mass = row
            .teacher_q16
            .iter()
            .map(|weight| u64::from(*weight))
            .sum::<u64>();
        if mass == 0 {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "scoring row has zero teacher/support overlap".to_owned(),
            ));
        }
        let mut raw_loss = 0.0_f64;
        for (&teacher, probability) in row.teacher_q16.iter().zip(&probabilities) {
            if teacher != 0 {
                raw_loss -=
                    (f64::from(teacher) / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL)) * probability.ln();
            }
        }
        let winner = row
            .logits
            .iter()
            .enumerate()
            .max_by(|(left_index, left), (right_index, right)| {
                left.total_cmp(right)
                    .then_with(|| row.tokens[*right_index].cmp(&row.tokens[*left_index]))
            })
            .map(|(index, _)| row.tokens[index])
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "scoring row has no winning token".to_owned(),
                )
            })?;
        let teacher_agreement = winner == row.teacher_top_token;
        let actual_correct = winner == row.actual_next_token;
        teacher_top1 += u64::from(teacher_agreement);
        actual_top1 += u64::from(actual_correct);
        total_mass = total_mass.checked_add(mass).ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid("teacher mass ledger overflowed".to_owned())
        })?;
        total_raw_loss += raw_loss;
        positions.push(ScoredPosition {
            document_id: row.document_id.clone(),
            position: row.position,
            teacher_mass_q16: mass,
            covered_loss_nats: raw_loss / (mass as f64 / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL)),
            teacher_top1: teacher_agreement,
            actual_top1: actual_correct,
        });
    }
    let cross_entropy = (total_mass != 0)
        .then_some(total_raw_loss / (total_mass as f64 / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL)));
    Ok((
        MetricSummary {
            positions: rows.len() as u64,
            teacher_mass_covered_q16: total_mass,
            covered_mass_fraction: (!rows.is_empty()).then_some(
                total_mass as f64 / (rows.len() as f64 * f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL)),
            ),
            covered_teacher_cross_entropy_nats: cross_entropy,
            teacher_top1_agreements: teacher_top1,
            actual_next_top1_correct: actual_top1,
        },
        positions,
    ))
}

fn stable_softmax(logits: &[f64]) -> Result<Vec<f64>, R4SoftmaxTraceObservabilityError> {
    if logits.is_empty() || logits.iter().any(|logit| !logit.is_finite()) {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "softmax logits are empty or non-finite".to_owned(),
        ));
    }
    let maximum = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut probabilities = logits
        .iter()
        .map(|logit| (*logit - maximum).exp())
        .collect::<Vec<_>>();
    let normalizer = probabilities.iter().sum::<f64>();
    if !normalizer.is_finite() || normalizer <= 0.0 {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "softmax normalizer is invalid".to_owned(),
        ));
    }
    for probability in &mut probabilities {
        *probability /= normalizer;
        if !probability.is_finite() || *probability <= 0.0 {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "softmax probability is non-finite or non-positive".to_owned(),
            ));
        }
    }
    Ok(probabilities)
}

fn paired_uncertainty(
    real: &[ScoredPosition],
    control: &[ScoredPosition],
) -> Result<PairedUncertainty, R4SoftmaxTraceObservabilityError> {
    if real.len() != control.len() {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "paired score lengths differ".to_owned(),
        ));
    }
    let mut delta = 0.0_f64;
    let mut real_wins = 0_u64;
    let mut control_wins = 0_u64;
    let mut ties = 0_u64;
    for (real, control) in real.iter().zip(control) {
        if real.document_id != control.document_id
            || real.position != control.position
            || real.teacher_mass_q16 != control.teacher_mass_q16
        {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "paired score identities or support mass differ".to_owned(),
            ));
        }
        let difference = control.covered_loss_nats - real.covered_loss_nats;
        delta += difference;
        if difference > 1.0e-12 {
            real_wins += 1;
        } else if difference < -1.0e-12 {
            control_wins += 1;
        } else {
            ties += 1;
        }
    }
    let non_ties = real_wins + control_wins;
    Ok(PairedUncertainty {
        positions: real.len() as u64,
        candidate_loss_delta_control_minus_real_nats: (!real.is_empty())
            .then_some(delta / real.len() as f64),
        real_wins,
        control_wins,
        ties,
        exact_two_sided_sign_p: (non_ties != 0)
            .then_some(exact_two_sided_sign_p(real_wins, control_wins)),
    })
}

fn exact_two_sided_sign_p(real_wins: u64, control_wins: u64) -> f64 {
    let trials = real_wins + control_wins;
    let tail = real_wins.min(control_wins);
    let mut coefficient = 1.0_f64;
    let mut cumulative = 1.0_f64;
    for successes in 1..=tail {
        coefficient *= (trials - successes + 1) as f64 / successes as f64;
        cumulative += coefficient;
    }
    (2.0 * cumulative / 2.0_f64.powi(trials as i32)).min(1.0)
}

fn aggregate_diagnostic(
    folds: &[FoldResult],
) -> Result<Vec<AggregateDiagnosticResult>, R4SoftmaxTraceObservabilityError> {
    DIAGNOSTIC_BOUNDARIES
        .iter()
        .map(|boundary| {
            let entries = folds
                .iter()
                .map(|fold| diagnostic_boundary(fold, boundary))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(AggregateDiagnosticResult {
                boundary: (*boundary).to_owned(),
                primary: aggregate_metrics(entries.iter().map(|entry| &entry.primary))?,
                primary_label_permuted: aggregate_metrics(
                    entries.iter().map(|entry| &entry.primary_label_permuted),
                )?,
                primary_paired: aggregate_paired(
                    entries.iter().map(|entry| &entry.primary_paired),
                )?,
                secondary: aggregate_metrics(entries.iter().map(|entry| &entry.secondary))?,
                secondary_label_permuted: aggregate_metrics(
                    entries.iter().map(|entry| &entry.secondary_label_permuted),
                )?,
                secondary_paired: aggregate_paired(
                    entries.iter().map(|entry| &entry.secondary_paired),
                )?,
            })
        })
        .collect()
}

fn aggregate_exact(
    folds: &[FoldResult],
) -> Result<Vec<AggregateExactResult>, R4SoftmaxTraceObservabilityError> {
    EXACT_BOUNDARIES
        .iter()
        .map(|boundary| {
            let entries = folds
                .iter()
                .map(|fold| exact_boundary(fold, boundary))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(AggregateExactResult {
                boundary: (*boundary).to_owned(),
                primary: aggregate_metrics(entries.iter().map(|entry| &entry.primary))?,
                secondary: aggregate_metrics(entries.iter().map(|entry| &entry.secondary))?,
            })
        })
        .collect()
}

fn diagnostic_boundary<'a>(
    fold: &'a FoldResult,
    boundary: &str,
) -> Result<&'a DiagnosticBoundaryResult, R4SoftmaxTraceObservabilityError> {
    fold.diagnostic_boundaries
        .iter()
        .find(|entry| entry.boundary == boundary)
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(format!(
                "fold {} lacks diagnostic boundary {boundary}",
                fold.manifest.held_out_document_id
            ))
        })
}

fn exact_boundary<'a>(
    fold: &'a FoldResult,
    boundary: &str,
) -> Result<&'a ExactBoundaryResult, R4SoftmaxTraceObservabilityError> {
    fold.exact_boundaries
        .iter()
        .find(|entry| entry.boundary == boundary)
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(format!(
                "fold {} lacks exact boundary {boundary}",
                fold.manifest.held_out_document_id
            ))
        })
}

fn aggregate_metrics<'a>(
    metrics: impl Iterator<Item = &'a MetricSummary>,
) -> Result<MetricSummary, R4SoftmaxTraceObservabilityError> {
    let mut positions = 0_u64;
    let mut mass = 0_u64;
    let mut raw_loss = 0.0_f64;
    let mut teacher_top1 = 0_u64;
    let mut actual_top1 = 0_u64;
    for metric in metrics {
        positions = positions.checked_add(metric.positions).ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "aggregate position ledger overflowed".to_owned(),
            )
        })?;
        mass = mass
            .checked_add(metric.teacher_mass_covered_q16)
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "aggregate teacher-mass ledger overflowed".to_owned(),
                )
            })?;
        if let Some(cross_entropy) = metric.covered_teacher_cross_entropy_nats {
            raw_loss += cross_entropy
                * (metric.teacher_mass_covered_q16 as f64 / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL));
        } else if metric.teacher_mass_covered_q16 != 0 {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "nonzero aggregate mass lacks a cross-entropy".to_owned(),
            ));
        }
        teacher_top1 = teacher_top1
            .checked_add(metric.teacher_top1_agreements)
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "aggregate teacher top-1 ledger overflowed".to_owned(),
                )
            })?;
        actual_top1 = actual_top1
            .checked_add(metric.actual_next_top1_correct)
            .ok_or_else(|| {
                R4SoftmaxTraceObservabilityError::Invalid(
                    "aggregate actual top-1 ledger overflowed".to_owned(),
                )
            })?;
    }
    Ok(MetricSummary {
        positions,
        teacher_mass_covered_q16: mass,
        covered_mass_fraction: (positions != 0)
            .then_some(mass as f64 / (positions as f64 * f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL))),
        covered_teacher_cross_entropy_nats: (mass != 0)
            .then_some(raw_loss / (mass as f64 / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL))),
        teacher_top1_agreements: teacher_top1,
        actual_next_top1_correct: actual_top1,
    })
}

fn aggregate_paired<'a>(
    entries: impl Iterator<Item = &'a PairedUncertainty>,
) -> Result<PairedUncertainty, R4SoftmaxTraceObservabilityError> {
    let mut positions = 0_u64;
    let mut delta_sum = 0.0_f64;
    let mut real_wins = 0_u64;
    let mut control_wins = 0_u64;
    let mut ties = 0_u64;
    for entry in entries {
        positions = positions.checked_add(entry.positions).ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "aggregate paired position ledger overflowed".to_owned(),
            )
        })?;
        if let Some(delta) = entry.candidate_loss_delta_control_minus_real_nats {
            delta_sum += delta * entry.positions as f64;
        } else if entry.positions != 0 {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "nonempty paired entry lacks a loss delta".to_owned(),
            ));
        }
        real_wins += entry.real_wins;
        control_wins += entry.control_wins;
        ties += entry.ties;
    }
    let non_ties = real_wins + control_wins;
    Ok(PairedUncertainty {
        positions,
        candidate_loss_delta_control_minus_real_nats: (positions != 0)
            .then_some(delta_sum / positions as f64),
        real_wins,
        control_wins,
        ties,
        exact_two_sided_sign_p: (non_ties != 0)
            .then_some(exact_two_sided_sign_p(real_wins, control_wins)),
    })
}

fn required_cross_entropy(
    metric: &MetricSummary,
    boundary: &str,
) -> Result<f64, R4SoftmaxTraceObservabilityError> {
    metric
        .covered_teacher_cross_entropy_nats
        .filter(|value| value.is_finite())
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(format!(
                "boundary {boundary} lacks finite covered cross-entropy"
            ))
        })
}

fn decide_next_action(
    folds: &[FoldResult],
    diagnostics: &[AggregateDiagnosticResult],
    exact: &[AggregateExactResult],
) -> Result<(DecisionAudit, String, String), R4SoftmaxTraceObservabilityError> {
    let suffix = exact
        .iter()
        .find(|entry| entry.boundary == "suffix_exact_base_logits")
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "aggregate suffix exact boundary is absent".to_owned(),
            )
        })?;
    let full = diagnostics
        .iter()
        .find(|entry| entry.boundary == "full_final_layer_qkv_signed_sketch")
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "aggregate full-trace diagnostic boundary is absent".to_owned(),
            )
        })?;
    let reduced = diagnostics
        .iter()
        .find(|entry| entry.boundary == "signed_576_to_4_qkv_sketch")
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "aggregate reduced diagnostic boundary is absent".to_owned(),
            )
        })?;
    let geometric = diagnostics
        .iter()
        .find(|entry| entry.boundary == "geometric_natural_state_tensor")
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "aggregate geometric-state diagnostic boundary is absent".to_owned(),
            )
        })?;
    let current = exact
        .iter()
        .find(|entry| entry.boundary == "geometric_exact_base_plus_residual_logits")
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "aggregate geometric exact boundary is absent".to_owned(),
            )
        })?;
    let suffix_ce = required_cross_entropy(&suffix.primary, &suffix.boundary)?;
    let full_ce = required_cross_entropy(&full.primary, &full.boundary)?;
    let full_control_ce = required_cross_entropy(&full.primary_label_permuted, &full.boundary)?;
    let reduced_ce = required_cross_entropy(&reduced.primary, &reduced.boundary)?;
    let geometric_ce = required_cross_entropy(&geometric.primary, &geometric.boundary)?;
    let current_ce = required_cross_entropy(&current.primary, &current.boundary)?;
    let aggregate_primary_coverage_fraction =
        suffix.primary.covered_mass_fraction.ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "aggregate primary support coverage is absent".to_owned(),
            )
        })?;

    let mut full_vs_suffix_folds = 0_u64;
    let mut full_vs_control_folds = 0_u64;
    let mut reduction_loss_folds = 0_u64;
    let mut state_loss_folds = 0_u64;
    let mut readout_loss_folds = 0_u64;
    let mut minimum_primary_fold_coverage_fraction = 1.0_f64;
    for fold in folds {
        let fold_suffix = exact_boundary(fold, "suffix_exact_base_logits")?;
        let fold_full = diagnostic_boundary(fold, "full_final_layer_qkv_signed_sketch")?;
        let fold_reduced = diagnostic_boundary(fold, "signed_576_to_4_qkv_sketch")?;
        let fold_geometric = diagnostic_boundary(fold, "geometric_natural_state_tensor")?;
        let fold_current = exact_boundary(fold, "geometric_exact_base_plus_residual_logits")?;
        let fold_suffix_ce = required_cross_entropy(&fold_suffix.primary, &fold_suffix.boundary)?;
        let fold_coverage = fold_suffix.primary.covered_mass_fraction.ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(format!(
                "fold {} primary support coverage is absent",
                fold.manifest.held_out_document_id
            ))
        })?;
        minimum_primary_fold_coverage_fraction =
            minimum_primary_fold_coverage_fraction.min(fold_coverage);
        let fold_full_ce = required_cross_entropy(&fold_full.primary, &fold_full.boundary)?;
        let fold_control_ce =
            required_cross_entropy(&fold_full.primary_label_permuted, &fold_full.boundary)?;
        let fold_reduced_ce =
            required_cross_entropy(&fold_reduced.primary, &fold_reduced.boundary)?;
        let fold_geometric_ce =
            required_cross_entropy(&fold_geometric.primary, &fold_geometric.boundary)?;
        let fold_current_ce =
            required_cross_entropy(&fold_current.primary, &fold_current.boundary)?;
        full_vs_suffix_folds += u64::from(fold_suffix_ce > fold_full_ce);
        full_vs_control_folds += u64::from(fold_control_ce > fold_full_ce);
        reduction_loss_folds += u64::from(fold_reduced_ce > fold_full_ce);
        state_loss_folds += u64::from(fold_geometric_ce > fold_reduced_ce);
        readout_loss_folds += u64::from(fold_current_ce > fold_geometric_ce);
    }
    let support_coverage_sufficient =
        aggregate_primary_coverage_fraction >= 0.5 && minimum_primary_fold_coverage_fraction >= 0.5;
    let audit = DecisionAudit {
        aggregate_primary_coverage_fraction,
        minimum_primary_fold_coverage_fraction,
        support_coverage_sufficient,
        full_improvement_vs_suffix: decision_comparison(
            "suffix_ce_minus_full_trace_ce",
            suffix_ce - full_ce,
            full_vs_suffix_folds,
        ),
        full_improvement_vs_label_control: decision_comparison(
            "label_control_ce_minus_full_trace_ce",
            full_control_ce - full_ce,
            full_vs_control_folds,
        ),
        reduction_or_projection_loss_vs_full: decision_comparison(
            "reduction_or_projection_ce_minus_full_trace_ce",
            reduced_ce - full_ce,
            reduction_loss_folds,
        ),
        geometric_state_loss_vs_reduced: decision_comparison(
            "geometric_state_ce_minus_signed_reduction_ce",
            geometric_ce - reduced_ce,
            state_loss_folds,
        ),
        current_readout_loss_vs_geometric_probe: decision_comparison(
            "current_geometric_readout_ce_minus_geometric_probe_ce",
            current_ce - geometric_ce,
            readout_loss_folds,
        ),
    };

    if !audit.support_coverage_sufficient {
        return Ok((
            audit,
            "INSUFFICIENT_SUPPORT_COVERAGE".to_owned(),
            "Stop boundary attribution: matched train-only support covers less than half of recorded primary Q16 mass in aggregate or in at least one fold."
                .to_owned(),
        ));
    }
    if !audit.full_improvement_vs_suffix.material_and_stable
        || !audit.full_improvement_vs_label_control.material_and_stable
    {
        return Ok((audit,
            "FULL_TRACE_OBSERVABILITY_NOT_ESTABLISHED".to_owned(),
            "Stop this bounded current-step final-layer Q/K/V trace-distillation path and specify end-to-end cell training with a new untouched holdout."
                .to_owned(),
        ));
    }
    if audit
        .reduction_or_projection_loss_vs_full
        .material_and_stable
    {
        return Ok((audit,
            "SIGNAL_LOSS_AT_REDUCTION_OR_PROJECTION".to_owned(),
            "Open one reduction/projection disambiguation issue; preserve the full-trace probe as the upstream witness and do not assign causal blame to reduction alone."
                .to_owned(),
        ));
    }
    if audit.geometric_state_loss_vs_reduced.material_and_stable {
        return Ok((audit,
            "SIGNAL_LOSS_AT_GEOMETRIC_STATE".to_owned(),
            "Open one repair issue for token-map/state induction and transport; preserve the reduced probe as its upstream witness."
                .to_owned(),
        ));
    }
    if audit
        .current_readout_loss_vs_geometric_probe
        .material_and_stable
    {
        return Ok((audit,
            "SIGNAL_LOSS_AT_CURRENT_READOUT".to_owned(),
            "Open one repair issue for the current 64-value recurrent readout and use the train-only alpha diagnostic only to separate calibration from feature loss."
                .to_owned(),
        ));
    }
    Ok((audit,
        "NO_MATERIAL_NESTED_BOUNDARY_LOSS_ON_CONSTRUCTION_LODO".to_owned(),
        "Do not promote; freeze this construction-only result and design the next independently held causal qualification rung."
            .to_owned(),
    ))
}

fn decision_comparison(
    comparison: &str,
    aggregate_delta_nats: f64,
    directional_folds: u64,
) -> DecisionComparison {
    DecisionComparison {
        comparison: comparison.to_owned(),
        aggregate_delta_nats,
        directional_folds,
        required_delta_nats: MATERIAL_CE_DELTA_NATS,
        required_directional_folds: REQUIRED_DIRECTION_FOLDS,
        material_and_stable: aggregate_delta_nats >= MATERIAL_CE_DELTA_NATS
            && directional_folds >= REQUIRED_DIRECTION_FOLDS,
    }
}

fn support_identity_cid(rows: &[Row]) -> Result<String, R4SoftmaxTraceObservabilityError> {
    let value = serde_json::json!({
        "schema": "uor-r4.r4-softmax-trace-observability-supports/1",
        "rows": rows.iter().map(|row| serde_json::json!({
            "document_id": row.document_id,
            "position": row.position,
            "candidate_tokens": row.candidates.iter().map(|candidate| candidate.token).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    });
    canonical_json_cid(&value)
}

fn validate_metric_summary(
    metric: &MetricSummary,
    expected_positions: u64,
    context: &str,
) -> Result<(), R4SoftmaxTraceObservabilityError> {
    let maximum_mass = expected_positions
        .checked_mul(u64::from(R4_SOFTMAX_TRACE_Q16_TOTAL))
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(format!(
                "{context} maximum covered-mass ledger overflowed"
            ))
        })?;
    let expected_fraction = if expected_positions == 0 {
        None
    } else {
        Some(metric.teacher_mass_covered_q16 as f64 / maximum_mass as f64)
    };
    let fraction_exact = match (metric.covered_mass_fraction, expected_fraction) {
        (Some(actual), Some(expected)) => {
            actual.is_finite()
                && (0.0..=1.0).contains(&actual)
                && actual.to_bits() == expected.to_bits()
        }
        (None, None) => true,
        _ => false,
    };
    let cross_entropy_valid = match metric.covered_teacher_cross_entropy_nats {
        Some(value) => metric.teacher_mass_covered_q16 != 0 && value.is_finite() && value >= 0.0,
        None => metric.teacher_mass_covered_q16 == 0,
    };
    if metric.positions != expected_positions
        || metric.teacher_mass_covered_q16 > maximum_mass
        || (expected_positions != 0 && metric.teacher_mass_covered_q16 == 0)
        || !fraction_exact
        || !cross_entropy_valid
        || metric.teacher_top1_agreements > expected_positions
        || metric.actual_next_top1_correct > expected_positions
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
            "{context} metric range, mass, or census is invalid"
        )));
    }
    Ok(())
}

fn validate_paired_uncertainty(
    paired: &PairedUncertainty,
    expected_positions: u64,
    context: &str,
) -> Result<(), R4SoftmaxTraceObservabilityError> {
    let outcome_count = paired
        .real_wins
        .checked_add(paired.control_wins)
        .and_then(|value| value.checked_add(paired.ties))
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(format!(
                "{context} paired outcome ledger overflowed"
            ))
        })?;
    let delta_valid = match paired.candidate_loss_delta_control_minus_real_nats {
        Some(value) => expected_positions != 0 && value.is_finite(),
        None => expected_positions == 0,
    };
    let non_ties = paired
        .real_wins
        .checked_add(paired.control_wins)
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(format!(
                "{context} paired non-tie ledger overflowed"
            ))
        })?;
    let sign_p_valid = match paired.exact_two_sided_sign_p {
        Some(value) => {
            non_ties != 0
                && value.is_finite()
                && (0.0..=1.0).contains(&value)
                && value.to_bits()
                    == exact_two_sided_sign_p(paired.real_wins, paired.control_wins).to_bits()
        }
        None => non_ties == 0,
    };
    if paired.positions != expected_positions
        || outcome_count != expected_positions
        || !delta_valid
        || !sign_p_valid
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
            "{context} paired uncertainty is invalid"
        )));
    }
    Ok(())
}

fn require_matched_mass(
    reference: &MetricSummary,
    candidate: &MetricSummary,
    context: &str,
) -> Result<(), R4SoftmaxTraceObservabilityError> {
    if reference.positions != candidate.positions
        || reference.teacher_mass_covered_q16 != candidate.teacher_mass_covered_q16
        || reference.covered_mass_fraction.map(f64::to_bits)
            != candidate.covered_mass_fraction.map(f64::to_bits)
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
            "{context} changes the matched teacher support mass"
        )));
    }
    Ok(())
}

fn validate_result(
    result: &R4SoftmaxTraceObservabilityResult,
) -> Result<(), R4SoftmaxTraceObservabilityError> {
    validate_revision(&result.implementation_revision)?;
    let expected_cids = EXPECTED_DOCUMENT_TRACE_CIDS
        .iter()
        .map(|cid| (*cid).to_owned())
        .collect::<Vec<_>>();
    if result.schema != OBSERVABILITY_RESULT_SCHEMA
        || result.issue != ISSUE
        || result.predecessor_freeze_cid != PREDECESSOR_FREEZE_CID
        || result.construction_trace_bundle_cid != CONSTRUCTION_TRACE_BUNDLE_CID
        || result.construction_trace_bundle_bytes != EXPECTED_TRACE_BYTES
        || result.construction_document_trace_cids != expected_cids
        || !result.exact_replay
        || result.primary_event_count != EXPECTED_PRIMARY_EVENTS
        || result.secondary_event_count != EXPECTED_SECONDARY_EVENTS
        || result.folds.len() != 4
        || result.aggregate_diagnostic_boundaries.len() != DIAGNOSTIC_BOUNDARIES.len()
        || result.aggregate_exact_boundaries.len() != EXACT_BOUNDARIES.len()
        || result.terminal.is_empty()
        || result.next_action.is_empty()
        || result.nonclaims.len() < 3
        || !valid_blake3_cid(&result.result_cid)
        || canonical_json_cid_omitting_result_cid(result)? != result.result_cid
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "result envelope violates the frozen observability schema".to_owned(),
        ));
    }
    if result.input_audit
        != (ObservabilityInputAudit {
            trace_bundle_reads: 1,
            predecessor_freeze_reads: 1,
            source_model_reads: 0,
            source_model_forwards: 0,
            held_out_document_13_reads: 0,
            canonical_bundle_reload_exact: true,
            nested_document_cids_exact: true,
            document_order_exact: true,
            construction_only: true,
        })
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "result input audit is not the exact source-free contract".to_owned(),
        ));
    }
    if result.probe_contract.weights != PROBE_WIDTH
        || result.probe_contract.bias
        || result.probe_contract.regularization != "1/1024"
        || result.probe_contract.steps != PROBE_STEPS
        || result.probe_contract.initial_rate != "1/16"
        || result.probe_contract.maximum_backtracks_per_step != PROBE_BACKTRACKS
        || !result.probe_contract.full_batch
        || !result.probe_contract.candidate_conditioned_sketch
        || result.probe_contract.support_cap != SUPPORT_CAP
        || result.probe_contract.fit_loss_weighting
            != "raw covered Q16 mass normalized globally; targets renormalized within each row"
        || result.probe_contract.headline_loss_weighting
            != "raw covered Q16 mass, normalized after aggregation"
        || result.probe_contract.paired_loss_weighting
            != "equal held positions after per-row covered-Q16 renormalization"
        || result.probe_contract.feature_standardization
            != "train-only per-lane uncentered RMS; same scales applied to held rows"
        || result.probe_contract.feature_standardization_epsilon != "1e-12"
        || result.probe_contract.signed_projection
            != "one BLAKE3 XOF per boundary/candidate/width; one canonical u64 mask per source; bit j is output-lane j sign; divide by sqrt(width)"
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "probe contract differs from the frozen matched budget".to_owned(),
        ));
    }

    for (ordinal, fold) in result.folds.iter().enumerate() {
        let expected_training_ids = EXPECTED_DOCUMENT_IDS
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != ordinal)
            .map(|(_, id)| (*id).to_owned())
            .collect::<Vec<_>>();
        let expected_training_trace_cids = EXPECTED_DOCUMENT_TRACE_CIDS
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != ordinal)
            .map(|(_, cid)| (*cid).to_owned())
            .collect::<Vec<_>>();
        let expected_training_positions = EXPECTED_POSITIONS - EXPECTED_DOCUMENT_POSITIONS[ordinal];
        let expected_training_events = expected_training_positions - 3;
        if fold.manifest.ordinal != ordinal
            || fold.manifest.held_out_document_id != EXPECTED_DOCUMENT_IDS[ordinal]
            || fold.manifest.held_out_document_trace_cid != EXPECTED_DOCUMENT_TRACE_CIDS[ordinal]
            || fold.manifest.training_document_ids != expected_training_ids
            || fold.manifest.training_document_trace_cids != expected_training_trace_cids
            || fold.manifest.training_positions != expected_training_positions
            || fold.manifest.training_non_bos_events != expected_training_events
            || fold.manifest.primary_exact_prefix_novel_events != EXPECTED_PRIMARY_PER_FOLD[ordinal]
            || fold.manifest.secondary_non_bos_events != EXPECTED_SECONDARY_PER_FOLD[ordinal]
            || fold.manifest.held_out_events_used_for_fit != 0
            || fold.manifest.state_resets != 12
            || fold.candidate_support_rows != EXPECTED_POSITIONS
            || fold.candidate_support_maximum == 0
            || fold.candidate_support_maximum > SUPPORT_CAP
            || !fold.candidate_support_train_only
            || fold.runtime_source_reads != 0
            || fold.runtime_future_reads != 0
            || !valid_blake3_cid(&fold.manifest.fold_input_cid)
            || !valid_blake3_cid(&fold.manifest.support_artifact_cid)
            || !valid_blake3_cid(&fold.manifest.state_artifact_cid)
            || !valid_blake3_cid(&fold.manifest.training_support_cid)
            || !valid_blake3_cid(&fold.manifest.label_control_mapping_cid)
            || !valid_blake3_cid(&fold.manifest.training_fit_identity_cid)
            || !fold.manifest.held_label_substitution_fit_identity_exact
            || fold.manifest.held_label_substitution_mutated_events
                != EXPECTED_SECONDARY_PER_FOLD[ordinal] as u64
            || !valid_blake3_cid(&fold.manifest.held_labels_original_cid)
            || !valid_blake3_cid(&fold.manifest.held_labels_substituted_cid)
            || fold.manifest.held_labels_original_cid == fold.manifest.held_labels_substituted_cid
            || fold.manifest.label_control_mass_audit.rows != expected_training_events as u64
            || fold
                .manifest
                .label_control_mass_audit
                .original_covered_mass_q16
                == 0
            || fold
                .manifest
                .label_control_mass_audit
                .original_covered_mass_q16
                != fold
                    .manifest
                    .label_control_mass_audit
                    .permuted_covered_mass_q16
            || !fold
                .manifest
                .label_control_mass_audit
                .mass_preserved_exactly
            || fold.manifest.label_control_mass_audit.invalid_support_rows != 0
            || fold.manifest.label_control_mass_audit.unchanged_label_rows != 0
            || !valid_blake3_cid(&fold.candidate_support_cid)
            || fold.diagnostic_boundaries.len() != DIAGNOSTIC_BOUNDARIES.len()
            || fold.exact_boundaries.len() != EXACT_BOUNDARIES.len()
        {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                "fold {ordinal} violates its frozen split, census, or provenance contract"
            )));
        }
        let reference_primary = &fold.exact_boundaries[0].primary;
        let reference_secondary = &fold.exact_boundaries[0].secondary;
        let expected_raw_widths = [
            FULL_QKV_WIDTH,
            REDUCED_QKV_WIDTH,
            PROBE_WIDTH,
            PROBE_WIDTH,
            PROBE_WIDTH,
        ];
        for (boundary_index, boundary) in fold.diagnostic_boundaries.iter().enumerate() {
            if boundary.boundary != DIAGNOSTIC_BOUNDARIES[boundary_index]
                || boundary.feature_audit.raw_width != expected_raw_widths[boundary_index]
                || boundary.feature_audit.probe_width != PROBE_WIDTH
                || boundary.feature_audit.fitted_weights != PROBE_WIDTH
                || boundary.feature_audit.bias
                || !boundary.feature_audit.all_finite
                || !boundary.feature_audit.target_blind_preprocessing
                || !boundary.feature_audit.train_only_unit_rms_standardization
                || boundary.feature_audit.standardized_unit_rms_lanes == 0
                || boundary.feature_audit.candidate_feature_rows == 0
                || boundary.feature_audit.unique_feature_rows == 0
                || boundary.feature_audit.unique_feature_rows
                    > boundary.feature_audit.candidate_feature_rows
                || boundary.feature_audit.exact_collision_rows
                    != boundary
                        .feature_audit
                        .candidate_feature_rows
                        .saturating_sub(boundary.feature_audit.unique_feature_rows)
                || boundary.feature_audit.zero_overlap_rows != 0
                || boundary.feature_audit.active_lanes == 0
                || boundary.feature_audit.active_lanes > PROBE_WIDTH
                || boundary.feature_audit.numerical_rank > PROBE_WIDTH
                || !boundary.feature_audit.total_variance.is_finite()
                || boundary.feature_audit.total_variance <= 0.0
                || boundary.feature_audit.standardized_unit_rms_lanes > PROBE_WIDTH
                || boundary.label_permuted_training_rows_changed
                    != fold.manifest.training_non_bos_events as u64
                || !valid_blake3_cid(&boundary.feature_audit.feature_cid)
                || !valid_blake3_cid(&boundary.feature_audit.standardization_stats_cid)
                || !valid_blake3_cid(&boundary.fitted_weight_cid)
                || !valid_blake3_cid(&boundary.label_permuted_weight_cid)
                || boundary.label_control_mapping_cid != fold.manifest.label_control_mapping_cid
                || boundary.optimizer_backtracks > (PROBE_STEPS * PROBE_BACKTRACKS) as u64
                || boundary.label_permuted_optimizer_backtracks
                    > (PROBE_STEPS * PROBE_BACKTRACKS) as u64
                || boundary.primary.positions
                    != fold.manifest.primary_exact_prefix_novel_events as u64
                || boundary.secondary.positions != fold.manifest.secondary_non_bos_events as u64
                || boundary.primary_label_permuted.positions != boundary.primary.positions
                || boundary.secondary_label_permuted.positions != boundary.secondary.positions
            {
                return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                    "fold {ordinal} diagnostic boundary {} violates its matched contract",
                    boundary.boundary
                )));
            }
            let primary_positions = fold.manifest.primary_exact_prefix_novel_events as u64;
            let secondary_positions = fold.manifest.secondary_non_bos_events as u64;
            validate_metric_summary(
                &boundary.primary,
                primary_positions,
                &format!("fold {ordinal} {} primary", boundary.boundary),
            )?;
            validate_metric_summary(
                &boundary.primary_label_permuted,
                primary_positions,
                &format!("fold {ordinal} {} primary control", boundary.boundary),
            )?;
            validate_metric_summary(
                &boundary.secondary,
                secondary_positions,
                &format!("fold {ordinal} {} secondary", boundary.boundary),
            )?;
            validate_metric_summary(
                &boundary.secondary_label_permuted,
                secondary_positions,
                &format!("fold {ordinal} {} secondary control", boundary.boundary),
            )?;
            require_matched_mass(
                reference_primary,
                &boundary.primary,
                &format!("fold {ordinal} {} primary", boundary.boundary),
            )?;
            require_matched_mass(
                reference_primary,
                &boundary.primary_label_permuted,
                &format!("fold {ordinal} {} primary control", boundary.boundary),
            )?;
            require_matched_mass(
                reference_secondary,
                &boundary.secondary,
                &format!("fold {ordinal} {} secondary", boundary.boundary),
            )?;
            require_matched_mass(
                reference_secondary,
                &boundary.secondary_label_permuted,
                &format!("fold {ordinal} {} secondary control", boundary.boundary),
            )?;
            validate_paired_uncertainty(
                &boundary.primary_paired,
                primary_positions,
                &format!("fold {ordinal} {} primary", boundary.boundary),
            )?;
            validate_paired_uncertainty(
                &boundary.secondary_paired,
                secondary_positions,
                &format!("fold {ordinal} {} secondary", boundary.boundary),
            )?;
        }
        for (boundary_index, boundary) in fold.exact_boundaries.iter().enumerate() {
            let recurrent = boundary_index != 0;
            if boundary.boundary != EXACT_BOUNDARIES[boundary_index]
                || boundary.fitted_parameter_values
                    != if recurrent {
                        R4_SOFTMAX_TRACE_STATE_FITTED_VALUES_PER_ARM
                    } else {
                        0
                    }
                || boundary.actual_alpha != if recurrent { "1" } else { "0" }
                || boundary.primary.positions
                    != fold.manifest.primary_exact_prefix_novel_events as u64
                || boundary.secondary.positions != fold.manifest.secondary_non_bos_events as u64
                || recurrent != boundary.residual_scale_diagnostic.is_some()
            {
                return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                    "fold {ordinal} exact boundary {} violates its budget or census",
                    boundary.boundary
                )));
            }
            let primary_positions = fold.manifest.primary_exact_prefix_novel_events as u64;
            let secondary_positions = fold.manifest.secondary_non_bos_events as u64;
            validate_metric_summary(
                &boundary.primary,
                primary_positions,
                &format!("fold {ordinal} {} primary", boundary.boundary),
            )?;
            validate_metric_summary(
                &boundary.secondary,
                secondary_positions,
                &format!("fold {ordinal} {} secondary", boundary.boundary),
            )?;
            require_matched_mass(
                reference_primary,
                &boundary.primary,
                &format!("fold {ordinal} {} primary", boundary.boundary),
            )?;
            require_matched_mass(
                reference_secondary,
                &boundary.secondary,
                &format!("fold {ordinal} {} secondary", boundary.boundary),
            )?;
            if let Some(diagnostic) = &boundary.residual_scale_diagnostic {
                let expected_names = RESIDUAL_ALPHA_GRID
                    .iter()
                    .map(|(_, name)| *name)
                    .collect::<Vec<_>>();
                if diagnostic
                    .trials
                    .iter()
                    .map(|trial| trial.alpha.as_str())
                    .collect::<Vec<_>>()
                    != expected_names
                    || !diagnostic.selection_uses_training_only
                    || !diagnostic.deterministic_tie_break_prefers_smaller_alpha
                    || !expected_names.contains(&diagnostic.selected_alpha.as_str())
                    || diagnostic.selected_primary.positions != boundary.primary.positions
                    || diagnostic.selected_secondary.positions != boundary.secondary.positions
                {
                    return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                        "fold {ordinal} exact boundary {} has an invalid alpha diagnostic",
                        boundary.boundary
                    )));
                }
                for trial in &diagnostic.trials {
                    if trial
                        .training_covered_teacher_cross_entropy_nats
                        .is_none_or(|value| !value.is_finite() || value < 0.0)
                    {
                        return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                            "fold {ordinal} exact boundary {} has an invalid train-only alpha loss",
                            boundary.boundary
                        )));
                    }
                }
                validate_metric_summary(
                    &diagnostic.selected_primary,
                    primary_positions,
                    &format!(
                        "fold {ordinal} {} selected-alpha primary",
                        boundary.boundary
                    ),
                )?;
                validate_metric_summary(
                    &diagnostic.selected_secondary,
                    secondary_positions,
                    &format!(
                        "fold {ordinal} {} selected-alpha secondary",
                        boundary.boundary
                    ),
                )?;
                require_matched_mass(
                    reference_primary,
                    &diagnostic.selected_primary,
                    &format!(
                        "fold {ordinal} {} selected-alpha primary",
                        boundary.boundary
                    ),
                )?;
                require_matched_mass(
                    reference_secondary,
                    &diagnostic.selected_secondary,
                    &format!(
                        "fold {ordinal} {} selected-alpha secondary",
                        boundary.boundary
                    ),
                )?;
            }
        }
        let recomputed_fit_identity_cid = training_fit_identity_cid(
            &fold.manifest.support_artifact_cid,
            &fold.manifest.state_artifact_cid,
            &fold.manifest.training_support_cid,
            &fold.manifest.label_control_mapping_cid,
            &fold.diagnostic_boundaries,
            &fold.exact_boundaries,
        )?;
        if recomputed_fit_identity_cid != fold.manifest.training_fit_identity_cid {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                "fold {ordinal} training-fit identity does not replay from its train-only fields"
            )));
        }
    }
    let aggregate_reference_primary = &result.aggregate_exact_boundaries[0].primary;
    let aggregate_reference_secondary = &result.aggregate_exact_boundaries[0].secondary;
    for (index, aggregate) in result.aggregate_diagnostic_boundaries.iter().enumerate() {
        if aggregate.boundary != DIAGNOSTIC_BOUNDARIES[index]
            || aggregate.primary.positions != EXPECTED_PRIMARY_EVENTS as u64
            || aggregate.secondary.positions != EXPECTED_SECONDARY_EVENTS as u64
            || aggregate.primary_label_permuted.positions != EXPECTED_PRIMARY_EVENTS as u64
            || aggregate.secondary_label_permuted.positions != EXPECTED_SECONDARY_EVENTS as u64
        {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                "aggregate diagnostic boundary {index} violates order or census"
            )));
        }
        validate_metric_summary(
            &aggregate.primary,
            EXPECTED_PRIMARY_EVENTS as u64,
            &format!("aggregate {} primary", aggregate.boundary),
        )?;
        validate_metric_summary(
            &aggregate.primary_label_permuted,
            EXPECTED_PRIMARY_EVENTS as u64,
            &format!("aggregate {} primary control", aggregate.boundary),
        )?;
        validate_metric_summary(
            &aggregate.secondary,
            EXPECTED_SECONDARY_EVENTS as u64,
            &format!("aggregate {} secondary", aggregate.boundary),
        )?;
        validate_metric_summary(
            &aggregate.secondary_label_permuted,
            EXPECTED_SECONDARY_EVENTS as u64,
            &format!("aggregate {} secondary control", aggregate.boundary),
        )?;
        require_matched_mass(
            aggregate_reference_primary,
            &aggregate.primary,
            &format!("aggregate {} primary", aggregate.boundary),
        )?;
        require_matched_mass(
            aggregate_reference_primary,
            &aggregate.primary_label_permuted,
            &format!("aggregate {} primary control", aggregate.boundary),
        )?;
        require_matched_mass(
            aggregate_reference_secondary,
            &aggregate.secondary,
            &format!("aggregate {} secondary", aggregate.boundary),
        )?;
        require_matched_mass(
            aggregate_reference_secondary,
            &aggregate.secondary_label_permuted,
            &format!("aggregate {} secondary control", aggregate.boundary),
        )?;
        validate_paired_uncertainty(
            &aggregate.primary_paired,
            EXPECTED_PRIMARY_EVENTS as u64,
            &format!("aggregate {} primary", aggregate.boundary),
        )?;
        validate_paired_uncertainty(
            &aggregate.secondary_paired,
            EXPECTED_SECONDARY_EVENTS as u64,
            &format!("aggregate {} secondary", aggregate.boundary),
        )?;
    }
    for (index, aggregate) in result.aggregate_exact_boundaries.iter().enumerate() {
        if aggregate.boundary != EXACT_BOUNDARIES[index]
            || aggregate.primary.positions != EXPECTED_PRIMARY_EVENTS as u64
            || aggregate.secondary.positions != EXPECTED_SECONDARY_EVENTS as u64
        {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(format!(
                "aggregate exact boundary {index} violates order or census"
            )));
        }
        validate_metric_summary(
            &aggregate.primary,
            EXPECTED_PRIMARY_EVENTS as u64,
            &format!("aggregate {} primary", aggregate.boundary),
        )?;
        validate_metric_summary(
            &aggregate.secondary,
            EXPECTED_SECONDARY_EVENTS as u64,
            &format!("aggregate {} secondary", aggregate.boundary),
        )?;
        require_matched_mass(
            aggregate_reference_primary,
            &aggregate.primary,
            &format!("aggregate {} primary", aggregate.boundary),
        )?;
        require_matched_mass(
            aggregate_reference_secondary,
            &aggregate.secondary,
            &format!("aggregate {} secondary", aggregate.boundary),
        )?;
    }
    if aggregate_diagnostic(&result.folds)? != result.aggregate_diagnostic_boundaries
        || aggregate_exact(&result.folds)? != result.aggregate_exact_boundaries
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "serialized aggregates do not replay exactly from fold evidence".to_owned(),
        ));
    }
    let (decision_audit, terminal, next_action) = decide_next_action(
        &result.folds,
        &result.aggregate_diagnostic_boundaries,
        &result.aggregate_exact_boundaries,
    )?;
    if result.decision_audit != decision_audit
        || result.terminal != terminal
        || result.next_action != next_action
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "decision audit or terminal does not replay from the measured boundaries".to_owned(),
        ));
    }
    let normal_backtracks = result.folds.iter().try_fold(0_u64, |total, fold| {
        fold.diagnostic_boundaries
            .iter()
            .try_fold(total, |subtotal, boundary| {
                subtotal
                    .checked_add(boundary.optimizer_backtracks)
                    .and_then(|value| {
                        value.checked_add(boundary.label_permuted_optimizer_backtracks)
                    })
                    .ok_or_else(|| {
                        R4SoftmaxTraceObservabilityError::Invalid(
                            "validated probe backtrack ledger overflowed".to_owned(),
                        )
                    })
            })
    })?;
    let expected_backtracks = normal_backtracks.checked_mul(4).ok_or_else(|| {
        R4SoftmaxTraceObservabilityError::Invalid(
            "two-pass audited probe backtrack ledger overflowed".to_owned(),
        )
    })?;
    let expected_probe_steps = 160 * PROBE_STEPS as u64;
    let expected_objectives = expected_probe_steps
        .checked_mul(2)
        .and_then(|value| value.checked_add(expected_backtracks))
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(
                "two-pass probe objective ledger overflowed".to_owned(),
            )
        })?;
    if result.work_ledger
        != (WorkLedger {
            logical_folds: 4,
            execution_passes: 2,
            replay_passes: 1,
            fold_executions: 8,
            suffix_compiles: 8,
            state_compiles: 8,
            probe_fits: 160,
            probe_steps: expected_probe_steps,
            probe_backtracks: expected_backtracks,
            probe_objective_evaluations: expected_objectives,
            held_label_substitution_audits: 8,
            training_event_fit_incidences: 4_080,
            training_alpha_score_row_evaluations: 12_240,
            primary_score_row_evaluations: 1_768,
            secondary_score_row_evaluations: 2_312,
            state_observations: 912,
            state_resets: 96,
        })
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "work ledger differs from the exact four-fold computation".to_owned(),
        ));
    }
    Ok(())
}

fn validate_revision(revision: &str) -> Result<(), R4SoftmaxTraceObservabilityError> {
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(R4SoftmaxTraceObservabilityError::Invalid(
            "implementation revision must be an exact lowercase 40-character Git commit".to_owned(),
        ));
    }
    Ok(())
}

fn validate_distinct_paths(paths: &[&Path]) -> Result<(), R4SoftmaxTraceObservabilityError> {
    let mut identities = BTreeSet::new();
    for path in paths {
        let absolute = if path.is_absolute() {
            (*path).to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        let identity = if absolute.exists() {
            fs::canonicalize(absolute)?
        } else {
            absolute
        };
        if !identities.insert(identity) {
            return Err(R4SoftmaxTraceObservabilityError::Invalid(
                "input and output paths must be pairwise distinct".to_owned(),
            ));
        }
    }
    Ok(())
}

fn canonical_json_cid<T: Serialize>(value: &T) -> Result<String, R4SoftmaxTraceObservabilityError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| R4SoftmaxTraceObservabilityError::Serialization(error.to_string()))?;
    Ok(bytes_cid(&bytes))
}

fn canonical_json_cid_omitting_result_cid(
    value: &R4SoftmaxTraceObservabilityResult,
) -> Result<String, R4SoftmaxTraceObservabilityError> {
    canonical_json_cid_omitting_fields(value, &["result_cid"])
}

fn canonical_json_cid_omitting_fields<T: Serialize>(
    value: &T,
    fields: &[&str],
) -> Result<String, R4SoftmaxTraceObservabilityError> {
    let value = serde_json::to_value(value)
        .map_err(|error| R4SoftmaxTraceObservabilityError::Serialization(error.to_string()))?;
    canonical_value_cid_omitting_fields(value, fields)
}

fn canonical_value_cid_omitting_fields(
    mut value: serde_json::Value,
    fields: &[&str],
) -> Result<String, R4SoftmaxTraceObservabilityError> {
    let object = value.as_object_mut().ok_or_else(|| {
        R4SoftmaxTraceObservabilityError::Serialization(
            "CID-bearing JSON is not an object".to_owned(),
        )
    })?;
    for field in fields {
        if object.remove(*field).is_none() {
            return Err(R4SoftmaxTraceObservabilityError::Serialization(format!(
                "CID field {field} is absent"
            )));
        }
    }
    canonical_json_cid(&value)
}

fn bytes_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn valid_blake3_cid(cid: &str) -> bool {
    cid.len() == 71
        && cid.starts_with("blake3:")
        && cid[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn write_json_atomic<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), R4SoftmaxTraceObservabilityError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| R4SoftmaxTraceObservabilityError::Serialization(error.to_string()))?;
    let parent = path.parent().ok_or_else(|| {
        R4SoftmaxTraceObservabilityError::Invalid(format!(
            "output {} has no parent directory",
            path.display()
        ))
    })?;
    fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            R4SoftmaxTraceObservabilityError::Invalid(format!(
                "output {} has no UTF-8 file name",
                path.display()
            ))
        })?;
    let temporary = parent.join(format!(".{file_name}.tmp"));
    fs::write(&temporary, bytes)?;
    fs::rename(&temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feature(lane: usize, value: f64) -> [f64; PROBE_WIDTH] {
        let mut output = [0.0_f64; PROBE_WIDTH];
        output[lane] = value;
        output
    }

    fn probe_row(
        document_id: &str,
        position: usize,
        teacher_q16: [u16; 3],
        feature_offset: usize,
    ) -> ProbeRow {
        ProbeRow {
            document_id: document_id.to_owned(),
            position,
            actual_next_token: 20,
            teacher_top_token: 20,
            teacher_top32_q16: [10_u32, 20, 30].into_iter().zip(teacher_q16).collect(),
            tokens: vec![10, 20, 30],
            teacher_q16: teacher_q16.to_vec(),
            base_logits: vec![0.0; 3],
            features: vec![
                feature(feature_offset, 1.0),
                feature((feature_offset + 1) % PROBE_WIDTH, 1.0),
                feature((feature_offset + 2) % PROBE_WIDTH, 1.0),
            ],
        }
    }

    #[test]
    fn work_ledger_combines_two_physical_passes_and_one_replay() {
        let pass = WorkLedger {
            logical_folds: 4,
            execution_passes: 1,
            replay_passes: 0,
            fold_executions: 4,
            suffix_compiles: 4,
            state_compiles: 4,
            probe_fits: 80,
            probe_steps: 40_960,
            probe_backtracks: 12,
            probe_objective_evaluations: 81_932,
            held_label_substitution_audits: 4,
            training_event_fit_incidences: 2_040,
            training_alpha_score_row_evaluations: 6_120,
            primary_score_row_evaluations: 884,
            secondary_score_row_evaluations: 1_156,
            state_observations: 456,
            state_resets: 48,
        };
        let combined = pass.combine_with_replay(&pass).expect("combined ledger");
        assert_eq!(combined.logical_folds, 4);
        assert_eq!(combined.execution_passes, 2);
        assert_eq!(combined.replay_passes, 1);
        assert_eq!(combined.fold_executions, 8);
        assert_eq!(combined.probe_fits, 160);
        assert_eq!(combined.probe_steps, 81_920);
        assert_eq!(combined.probe_backtracks, 24);
        assert_eq!(combined.probe_objective_evaluations, 163_864);
        assert_eq!(combined.primary_score_row_evaluations, 1_768);
        assert_eq!(combined.secondary_score_row_evaluations, 2_312);
        assert_eq!(combined.state_observations, 912);
        assert_eq!(combined.state_resets, 96);
    }

    #[test]
    fn dense_projection_is_deterministic_and_candidate_conditioned() {
        let input = [1.0_f64, -2.0, 0.5];
        let first = signed_sketch("boundary", 7, &input).expect("projection");
        let replay = signed_sketch("boundary", 7, &input).expect("replay");
        let other = signed_sketch("boundary", 8, &input).expect("other candidate");
        assert_eq!(first, replay);
        assert_ne!(first, other);
        assert!(first.iter().all(|value| value.is_finite()));
        assert!(first.iter().all(|value| *value != 0.0));
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"uor-r4/1012/candidate-signed-projection-xof-u64/v1");
        hasher.update(&("boundary".len() as u64).to_le_bytes());
        hasher.update(b"boundary");
        hasher.update(&7_u32.to_le_bytes());
        hasher.update(&(input.len() as u64).to_le_bytes());
        let mut bytes = [0_u8; 24];
        hasher.finalize_xof().fill(&mut bytes);
        let masks = bytes
            .chunks_exact(8)
            .map(|chunk| {
                let mut mask = [0_u8; 8];
                mask.copy_from_slice(chunk);
                u64::from_le_bytes(mask)
            })
            .collect::<Vec<_>>();
        for (lane, actual) in first.iter().enumerate() {
            let expected = input
                .iter()
                .enumerate()
                .map(|(source, value)| {
                    let sign = if masks[source] & (1_u64 << lane) == 0 {
                        1.0
                    } else {
                        -1.0
                    };
                    sign * *value / (input.len() as f64).sqrt()
                })
                .sum::<f64>();
            assert_eq!(actual.to_bits(), expected.to_bits(), "lane {lane}");
        }
    }

    #[test]
    fn slot_label_rotation_is_boundary_independent_and_mass_preserving() {
        let rows = vec![
            probe_row("14", 1, [60_000, 5_000, 535], 0),
            probe_row("657", 1, [535, 60_000, 5_000], 3),
            probe_row("4579", 1, [5_000, 535, 60_000], 6),
        ];
        let plan = build_label_control_plan(
            rows.iter()
                .map(|row| (row.document_id.clone(), row.position)),
        )
        .expect("mapping");
        let (first, changed) = slot_permuted_training_rows(&rows, &plan).expect("permutation");
        let mut changed_features = rows.clone();
        for row in &mut changed_features {
            row.features.reverse();
        }
        let (second, replay_changed) =
            slot_permuted_training_rows(&changed_features, &plan).expect("replay permutation");
        assert_eq!(changed, 3);
        assert_eq!(replay_changed, 3);
        assert_eq!(
            first.iter().map(|row| &row.teacher_q16).collect::<Vec<_>>(),
            second
                .iter()
                .map(|row| &row.teacher_q16)
                .collect::<Vec<_>>()
        );
        for (control, real) in first.iter().zip(&rows) {
            let mut expected = real.teacher_q16.clone();
            expected.rotate_left(1);
            assert_eq!(control.teacher_q16, expected);
            assert_eq!(
                control
                    .teacher_q16
                    .iter()
                    .map(|weight| u64::from(*weight))
                    .sum::<u64>(),
                real.teacher_q16
                    .iter()
                    .map(|weight| u64::from(*weight))
                    .sum::<u64>()
            );
        }
    }

    #[test]
    fn slot_rotation_handles_unequal_documents_and_heterogeneous_supports() {
        let document_weights = BTreeMap::from([
            (
                "14",
                BTreeMap::from([(10, 40_000), (20, 15_000), (30, 8_000), (40, 2_535)]),
            ),
            (
                "657",
                BTreeMap::from([(10, 2_535), (20, 40_000), (30, 15_000), (40, 8_000)]),
            ),
            (
                "4579",
                BTreeMap::from([(10, 8_000), (20, 2_535), (30, 40_000), (40, 15_000)]),
            ),
        ]);
        let mut rows = Vec::new();
        for (document, length) in [("14", 2_usize), ("657", 3), ("4579", 4)] {
            for position in 1..=length {
                let tokens = if position % 2 == 0 {
                    vec![10, 30, 40]
                } else {
                    vec![10, 20]
                };
                let full = document_weights.get(document).unwrap().clone();
                rows.push(ProbeRow {
                    document_id: document.to_owned(),
                    position,
                    actual_next_token: 10,
                    teacher_top_token: 10,
                    teacher_top32_q16: full.clone(),
                    teacher_q16: tokens
                        .iter()
                        .map(|token| full.get(token).copied().unwrap_or(0))
                        .collect(),
                    base_logits: vec![0.0; tokens.len()],
                    features: (0..tokens.len()).map(|lane| feature(lane, 1.0)).collect(),
                    tokens,
                });
            }
        }
        let plan = build_label_control_plan(
            rows.iter()
                .map(|row| (row.document_id.clone(), row.position)),
        )
        .expect("unequal-length mapping");
        assert_eq!(plan.row_identities.len(), rows.len());
        let (permuted, changed) =
            slot_permuted_training_rows(&rows, &plan).expect("aligned control");
        assert_eq!(changed, rows.len() as u64);
        assert!(permuted
            .iter()
            .zip(&rows)
            .all(|(control, real)| control.tokens == real.tokens
                && control.teacher_q16 != real.teacher_q16));
        for (control, real) in permuted.iter().zip(&rows) {
            let mut expected = real.teacher_q16.clone();
            expected.rotate_left(1);
            assert_eq!(control.teacher_q16, expected);
            assert_eq!(
                control.teacher_top32_q16,
                control
                    .tokens
                    .iter()
                    .copied()
                    .zip(control.teacher_q16.iter().copied())
                    .collect()
            );
            assert_eq!(
                control
                    .teacher_q16
                    .iter()
                    .map(|weight| u64::from(*weight))
                    .sum::<u64>(),
                real.teacher_q16
                    .iter()
                    .map(|weight| u64::from(*weight))
                    .sum::<u64>()
            );
        }
    }

    #[test]
    fn exact_scorer_uses_lower_token_tie_break_and_raw_mass_headline() {
        let rows = vec![
            ExactRow {
                document_id: "a".to_owned(),
                position: 1,
                actual_next_token: 3,
                teacher_top_token: 3,
                tokens: vec![9, 3],
                teacher_q16: vec![0, 65_535],
                logits: vec![0.0, 0.0],
            },
            ExactRow {
                document_id: "b".to_owned(),
                position: 1,
                actual_next_token: 4,
                teacher_top_token: 4,
                tokens: vec![4, 8],
                teacher_q16: vec![32_768, 0],
                logits: vec![0.0, 0.0],
            },
        ];
        let (metrics, scored) = score_exact_rows(&rows).expect("score");
        assert_eq!(metrics.teacher_top1_agreements, 2);
        assert_eq!(metrics.actual_next_top1_correct, 2);
        assert_eq!(scored.len(), 2);
        assert!((metrics.covered_teacher_cross_entropy_nats.unwrap() - 2.0_f64.ln()).abs() < 1e-12);
    }

    #[test]
    fn probe_fit_runs_exact_fixed_budget_without_bias() {
        let rows = vec![
            probe_row("14", 1, [60_000, 5_000, 535], 0),
            probe_row("657", 1, [535, 60_000, 5_000], 3),
            probe_row("4579", 1, [5_000, 535, 60_000], 6),
        ];
        let plan = build_label_control_plan(
            rows.iter()
                .map(|row| (row.document_id.clone(), row.position)),
        )
        .expect("mapping");
        let fitted = fit_probe(&rows, None).expect("real fit");
        let control = fit_probe(&rows, Some(&plan)).expect("control fit");
        assert_eq!(fitted.changed_label_rows, 0);
        assert_eq!(control.changed_label_rows, 3);
        assert!(fitted.weights.iter().all(|weight| weight.is_finite()));
        assert_ne!(fitted.weights, control.weights);
        assert!(fitted.backtracks <= (PROBE_STEPS * PROBE_BACKTRACKS) as u64);
    }

    #[test]
    fn probe_objective_uses_raw_covered_mass_weighting() {
        let mut high_mass = probe_row("14", 1, [60_000, 0, 0], 0);
        high_mass.base_logits = vec![0.0, 0.0, 0.0];
        let mut low_mass = probe_row("657", 1, [0, 6_000, 0], 3);
        low_mass.base_logits = vec![0.0, 3.0_f64.ln(), 0.0];
        let objective = probe_objective(&[high_mass, low_mass], &[0.0; PROBE_WIDTH])
            .expect("mass-weighted objective");
        let expected = (60_000.0 * 3.0_f64.ln() + 6_000.0 * (5.0_f64 / 3.0).ln()) / 66_000.0;
        assert!((objective - expected).abs() < 1.0e-12);
    }

    #[test]
    fn held_label_substitution_preserves_causal_candidate_inputs() {
        let candidate = |token, teacher_q16, lane| CandidateView {
            token,
            teacher_q16,
            full_features: feature(lane, 1.0),
            reduced_features: feature(lane, 2.0),
            plain_features: feature(lane, 3.0),
            geometric_features: feature(lane, 4.0),
            permuted_features: feature(lane, 5.0),
            suffix_logit: 0.25,
            plain_residual_logit: 0.5,
            plain_logit: 0.75,
            geometric_residual_logit: 1.0,
            geometric_logit: 1.25,
            permuted_residual_logit: 1.5,
            permuted_logit: 1.75,
        };
        let mut rows = vec![Row {
            document_id: "5121".to_owned(),
            position: 1,
            actual_next_token: 20,
            teacher_top_token: 10,
            teacher_top32_q16: BTreeMap::from([(10, 40_000), (20, 20_000), (30, 5_535)]),
            candidates: vec![candidate(10, 40_000, 0), candidate(20, 20_000, 1)],
            exact_prefix_novel: true,
        }];
        let original_candidates = rows[0].candidates.clone();
        let original_distribution = rows[0].teacher_top32_q16.clone();
        let original_mass = original_distribution
            .values()
            .map(|weight| u64::from(*weight))
            .sum::<u64>();
        assert_eq!(substitute_held_labels(&mut rows).expect("substitution"), 1);
        assert_ne!(rows[0].actual_next_token, 20);
        assert_ne!(rows[0].teacher_top32_q16, original_distribution);
        assert_eq!(
            rows[0]
                .teacher_top32_q16
                .values()
                .map(|weight| u64::from(*weight))
                .sum::<u64>(),
            original_mass
        );
        for (mutated, original) in rows[0].candidates.iter().zip(&original_candidates) {
            assert_eq!(mutated.token, original.token);
            assert_eq!(mutated.full_features, original.full_features);
            assert_eq!(mutated.reduced_features, original.reduced_features);
            assert_eq!(mutated.plain_features, original.plain_features);
            assert_eq!(mutated.geometric_features, original.geometric_features);
            assert_eq!(mutated.permuted_features, original.permuted_features);
            assert_eq!(mutated.suffix_logit, original.suffix_logit);
            assert_eq!(
                mutated.teacher_q16,
                rows[0]
                    .teacher_top32_q16
                    .get(&mutated.token)
                    .copied()
                    .unwrap_or(0)
            );
        }
    }

    #[test]
    fn residual_alpha_grid_is_exact_and_terminal_at_sixteen() {
        assert_eq!(
            RESIDUAL_ALPHA_GRID
                .iter()
                .map(|(_, name)| *name)
                .collect::<Vec<_>>(),
            vec!["0", "1/16", "1/8", "1/4", "1/2", "1", "2", "4", "8", "16"]
        );
    }

    #[test]
    fn decision_comparison_requires_materiality_and_three_folds() {
        assert!(decision_comparison("x", 0.10, 3).material_and_stable);
        assert!(!decision_comparison("x", 0.099, 4).material_and_stable);
        assert!(!decision_comparison("x", 0.20, 2).material_and_stable);
    }

    #[test]
    fn exact_sign_test_is_symmetric() {
        assert_eq!(exact_two_sided_sign_p(3, 0), 0.25);
        assert_eq!(exact_two_sided_sign_p(0, 3), 0.25);
        assert_eq!(exact_two_sided_sign_p(2, 2), 1.0);
    }
}
