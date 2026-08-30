//! Strict three-phase harness for the bounded R4 softmax trace state student.
//!
//! Construction consumes only the frozen construction trace, predecessor
//! freeze, and predecessor suffix artifact. Sealing consumes only the compiled
//! state artifact and a causal-input-only JSON document. Reveal consumes only
//! that immutable seal and the already-frozen #1010 judge JSON.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use uor_r4_core::helm_d_r4_attention::{
    R4SpinCausalAttentionTransport, R4SpinTransportIntervention,
};
use uor_r4_core::r4_softmax_trace_state_student::{
    compile_r4_softmax_trace_state_student, signed_reduce_final_layer_r4,
    R4SoftmaxTraceReductionRole, R4SoftmaxTraceStateArm, R4SoftmaxTraceStateFitConfig,
    R4SoftmaxTraceStateFitEvent, R4SoftmaxTraceStateFitSequence, R4SoftmaxTraceStatePrediction,
    R4SoftmaxTraceStateRuntimeAudit, R4SoftmaxTraceStateStudentArtifact,
    R4_SOFTMAX_TRACE_STATE_BYTES, R4_SOFTMAX_TRACE_STATE_FITTED_VALUES_PER_ARM,
    R4_SOFTMAX_TRACE_STATE_PARAMETER_BYTES_PER_ARM,
    R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM,
};
use uor_r4_core::r4_softmax_trace_student::{
    R4SoftmaxTraceSequence, R4SoftmaxTraceStudentArtifact, R4_SOFTMAX_TRACE_Q16_TOTAL,
};
use uor_r4_model_source::attention::CausalAttentionTransport;

use crate::geometric_decoder::{PINNED_EOS_TOKEN_ID, PINNED_TOKENIZER_CID};
use crate::r4_softmax_teacher_trace::{R4SoftmaxHeadTrace, R4SoftmaxTeacherTraceBundle};
use crate::r4_softmax_trace_experiment::{teacher_distribution, R4SoftmaxTraceFreeze};

pub const STATE_FREEZE_SCHEMA: &str = "uor-r4.r4-softmax-trace-state-freeze/1";
pub const STATE_SEAL_SCHEMA: &str = "uor-r4.r4-softmax-trace-state-seal/1";
pub const STATE_RESULT_SCHEMA: &str = "uor-r4.r4-softmax-trace-state-result/1";

pub const PREDECESSOR_FREEZE_CID: &str =
    "blake3:bb19fc6f6976aca6dfd8c67c470fd1fb70a1e1e74763800fdcb635f135325df7";
pub const PREDECESSOR_RESULT_CID: &str =
    "blake3:e48b4172e02fc84eef9e00024ac6602b790d8230026a979d2f71c552ddca0cd4";
pub const PREDECESSOR_SUFFIX_ARTIFACT_CID: &str =
    "blake3:e3b48b8bd113bf71be2fe9ecb64257b4eb1516303966d9d6c2c5cbe9d46adfac";
pub const CONSTRUCTION_TRACE_BUNDLE_CID: &str =
    "blake3:2de2affeff0be3dee3cc8fcd88bd83c5f049f81390870a3c78eea485c0fd62eb";
pub const CAUSAL_INPUT_CID: &str =
    "blake3:bc21446c5a2df6d715df206d8090c95cb49f128a3211bbca848f20fb8ba4ea28";
pub const PREDECESSOR_COVERED_CROSS_ENTROPY_NATS: f64 = 2.660_721_076_003_211;
pub const PREDECESSOR_TEACHER_TOP1: u64 = 3;
pub const PREDECESSOR_ACTUAL_TOP1: u64 = 2;
pub const FROZEN_CONTEXT_POSITIONS: u64 = 9;
pub const MATERIAL_CONTROL_CE_DELTA_NATS: f64 = 0.10;
pub const FROZEN_TEACHER_MASS_COVERED_Q16: u64 = 422_875;

const CONSTRUCTION_DOCUMENTS: usize = 4;
const CONSTRUCTION_POSITIONS: u64 = 38;
const CONSTRUCTION_TRACE_BUNDLE_BYTES: usize = 45_205_493;
const PREDECESSOR_SUFFIX_ARTIFACT_BYTES: usize = 39_648;
const STATE_ARTIFACT_BYTES: usize = 40_692;
const MAXIMUM_TOKEN_ID: u32 = 49_151;
const HELD_OUT_DOCUMENT_TEXT_CID: &str =
    "blake3:8afb2aa96f18a3a6eaf7cf4d721b16add2e3eb051b004ec974259ccf34bf5a67";
const EXPECTED_DOCUMENT_IDS: [&str; 4] = ["14", "657", "4579", "5121"];
const EXPECTED_DOCUMENT_TRACE_CIDS: [&str; 4] = [
    "blake3:62bb33d4605914541443bea0c972990031a0821449e932e19a929db1369f2960",
    "blake3:4a31dec9c955c5bca5513673cb0fa5c2fe3b7eb93f27a177838142abd00b4704",
    "blake3:3a51132fd40387f0e0f55e1fd50a7bfc05b8f1376e6fe877d63f9ab60bfd45ad",
    "blake3:bc47e6f862978effb616da762f935cbcc97b71631bb2cd6c650a6e6be1e91efc",
];
const ARMS: [R4SoftmaxTraceStateArm; 4] = [
    R4SoftmaxTraceStateArm::Suffix,
    R4SoftmaxTraceStateArm::PlainRecurrent,
    R4SoftmaxTraceStateArm::GeometricRecurrent,
    R4SoftmaxTraceStateArm::TransportPermutedControl,
];

#[derive(Clone, Debug)]
pub struct R4SoftmaxTraceStateCompileConfig {
    pub implementation_revision: String,
    pub trace_bundle: PathBuf,
    pub predecessor_freeze: PathBuf,
    pub suffix_artifact: PathBuf,
    pub artifact_output: PathBuf,
    pub freeze_output: PathBuf,
}

#[derive(Clone, Debug)]
pub struct R4SoftmaxTraceStateSealConfig {
    pub artifact: PathBuf,
    pub freeze: PathBuf,
    pub causal_input: PathBuf,
    pub seal_output: PathBuf,
}

#[derive(Clone, Debug)]
pub struct R4SoftmaxTraceStateRevealConfig {
    pub seal: PathBuf,
    pub judge: PathBuf,
    pub result_output: PathBuf,
}

#[derive(Debug)]
pub enum R4SoftmaxTraceStateExperimentError {
    Invalid(String),
    Io(std::io::Error),
    Serialization(String),
    Trace(String),
    Student(String),
}

impl fmt::Display for R4SoftmaxTraceStateExperimentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "invalid state experiment: {reason}"),
            Self::Io(error) => write!(formatter, "state experiment I/O failed: {error}"),
            Self::Serialization(reason) => {
                write!(formatter, "state experiment serialization failed: {reason}")
            }
            Self::Trace(reason) => write!(formatter, "state experiment trace failed: {reason}"),
            Self::Student(reason) => {
                write!(formatter, "state experiment student failed: {reason}")
            }
        }
    }
}

impl std::error::Error for R4SoftmaxTraceStateExperimentError {}

impl From<std::io::Error> for R4SoftmaxTraceStateExperimentError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateCompilerInputAudit {
    pub construction_trace_bundle_reads: u64,
    pub predecessor_freeze_reads: u64,
    pub suffix_artifact_reads: u64,
    pub source_model_reads: u64,
    pub held_out_causal_input_reads: u64,
    pub judge_reads: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4SoftmaxTraceStateFreeze {
    pub schema: String,
    pub issue: u32,
    pub implementation_revision: String,
    pub predecessor_freeze_cid: String,
    pub predecessor_result_cid_expected_at_reveal: String,
    pub construction_trace_bundle_cid: String,
    pub construction_trace_bundle_bytes: usize,
    pub construction_document_trace_cids: Vec<String>,
    pub suffix_artifact_cid: String,
    pub suffix_artifact_bytes: usize,
    pub state_artifact_cid: String,
    pub state_artifact_bytes: usize,
    pub state_artifact_payload_bytes_before_headers: usize,
    pub construction_digest: String,
    pub construction_documents: u32,
    pub construction_positions: u64,
    pub runtime_state_bytes_per_arm: usize,
    pub parameter_values_per_fitted_arm: usize,
    pub fitted_parameter_values_per_arm: usize,
    pub parameter_bytes_per_fitted_arm: usize,
    pub deterministic_compile_bytes_exact: bool,
    pub deterministic_compile_cid_exact: bool,
    pub artifact_reload_bytes_exact: bool,
    pub artifact_reload_cid_exact: bool,
    pub compiler_input_audit: StateCompilerInputAudit,
    pub compile_seconds: f64,
    pub freeze_cid: String,
    pub nonclaims: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CausalInputDocument {
    id: String,
    partition: String,
    text_cid: String,
    input_tokens: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CausalContinuationInput {
    prompt: String,
    bos_token_id: u32,
    prompt_token_ids: Vec<u32>,
    maximum_new_tokens: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CausalInput {
    schema: String,
    issue: u32,
    tokenizer_cid: String,
    held_out_document: CausalInputDocument,
    continuation: CausalContinuationInput,
    forbidden_fields: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateContinuationSeal {
    pub prompt: String,
    pub bos_token_id: u32,
    pub prompt_token_ids: Vec<u32>,
    pub maximum_new_tokens: usize,
    pub generated_token_ids: Vec<u32>,
    pub predictions: Vec<R4SoftmaxTraceStatePrediction>,
    pub frame_table_offsets: Vec<u16>,
    pub terminal_period_one_cycle: bool,
    pub terminal_period_two_cycle: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateArmSeal {
    pub arm: R4SoftmaxTraceStateArm,
    pub held_out_predictions: Vec<R4SoftmaxTraceStatePrediction>,
    pub held_out_runtime_audit: R4SoftmaxTraceStateRuntimeAudit,
    pub continuation: StateContinuationSeal,
    pub continuation_runtime_audit: R4SoftmaxTraceStateRuntimeAudit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateSealInputAudit {
    pub artifact_file_reads: u64,
    pub state_freeze_file_reads: u64,
    pub causal_input_file_reads: u64,
    pub construction_trace_reads: u64,
    pub predecessor_freeze_reads: u64,
    pub source_model_reads: u64,
    pub source_model_forwards: u64,
    pub judge_reads: u64,
    pub target_reads: u64,
    pub future_token_reads: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4SoftmaxTraceStateSeal {
    pub schema: String,
    pub issue: u32,
    pub state_artifact_cid: String,
    pub state_artifact_bytes: usize,
    pub state_freeze_cid: String,
    pub maximum_token_id: u32,
    pub causal_input_cid: String,
    pub allowed_runtime_input_digest: String,
    pub held_out_document_id: String,
    pub held_out_document_text_cid: String,
    pub held_out_input_positions: usize,
    pub held_out_input_tokens: Vec<u32>,
    pub held_out_frame_table_offsets: Vec<u16>,
    pub arms: Vec<StateArmSeal>,
    pub exact_replay: bool,
    pub input_audit: StateSealInputAudit,
    pub seal_seconds: f64,
    pub seal_cid: String,
    pub nonclaims: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateArmMetrics {
    pub arm: R4SoftmaxTraceStateArm,
    pub positions: u64,
    pub teacher_mass_covered_q16: u64,
    pub teacher_top1_agreements: u64,
    pub actual_next_top1_correct: u64,
    pub covered_teacher_cross_entropy_nats: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4SoftmaxTraceStateResult {
    pub schema: String,
    pub issue: u32,
    pub state_seal_cid: String,
    pub state_artifact_cid: String,
    pub state_freeze_cid: String,
    pub predecessor_result_cid: String,
    pub predecessor_freeze_cid: String,
    pub causal_input_cid: String,
    pub metrics: Vec<StateArmMetrics>,
    pub context_positions: u64,
    pub geometry_control_distinct_witnesses: u64,
    pub geometric_ce_below_predecessor: bool,
    pub geometric_ce_below_plain: bool,
    pub geometric_ce_below_permuted: bool,
    pub geometric_teacher_top1_above_predecessor: bool,
    pub geometric_actual_top1_above_predecessor: bool,
    pub material_permuted_ce_loss: bool,
    pub material_permuted_teacher_top1_loss: bool,
    pub geometry_control_state_and_output_distinct: bool,
    pub exact_replay: bool,
    pub zero_forbidden_runtime_reads: bool,
    pub geometric_continuation_has_no_period_one_or_two_cycle: bool,
    pub promotion_passed: bool,
    pub terminal: String,
    pub sealed_arms: Vec<StateArmSeal>,
    pub reveal_seconds: f64,
    pub result_cid: String,
    pub nonclaims: Vec<String>,
}

#[derive(Deserialize)]
struct PredecessorJudge {
    result_cid: String,
    construction_freeze_cid: String,
    artifact_cid_before_reveal: String,
    held_out_judge_sequence: R4SoftmaxTraceSequence,
}

/// Phase 1: compile construction traces into a deterministic state artifact.
pub fn compile_construction(
    config: &R4SoftmaxTraceStateCompileConfig,
) -> Result<R4SoftmaxTraceStateFreeze, R4SoftmaxTraceStateExperimentError> {
    let started = Instant::now();
    validate_revision(&config.implementation_revision)?;
    validate_distinct_paths(&[
        &config.trace_bundle,
        &config.predecessor_freeze,
        &config.suffix_artifact,
        &config.artifact_output,
        &config.freeze_output,
    ])?;

    let predecessor_freeze_bytes = fs::read(&config.predecessor_freeze)?;
    let predecessor_freeze: R4SoftmaxTraceFreeze =
        serde_json::from_slice(&predecessor_freeze_bytes).map_err(|error| {
            R4SoftmaxTraceStateExperimentError::Serialization(error.to_string())
        })?;
    validate_predecessor_freeze(&predecessor_freeze_bytes, &predecessor_freeze)?;

    let trace_bytes = fs::read(&config.trace_bundle)?;
    let expected_document_cids = EXPECTED_DOCUMENT_TRACE_CIDS
        .iter()
        .map(|cid| (*cid).to_owned())
        .collect::<Vec<_>>();
    let trace_bundle = R4SoftmaxTeacherTraceBundle::from_bytes_with_expected_cids(
        &trace_bytes,
        CONSTRUCTION_TRACE_BUNDLE_CID,
        &expected_document_cids,
    )
    .map_err(|error| R4SoftmaxTraceStateExperimentError::Trace(error.to_string()))?;
    if trace_bytes.len() != predecessor_freeze.trace_bundle_bytes
        || predecessor_freeze.trace_bundle_cid != CONSTRUCTION_TRACE_BUNDLE_CID
        || predecessor_freeze.document_trace_cids != expected_document_cids
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "construction trace does not match the predecessor freeze".to_owned(),
        ));
    }

    let suffix_bytes = fs::read(&config.suffix_artifact)?;
    let suffix = R4SoftmaxTraceStudentArtifact::from_bytes(&suffix_bytes)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?;
    if suffix.artifact_cid() != PREDECESSOR_SUFFIX_ARTIFACT_CID
        || predecessor_freeze.artifact_cid != PREDECESSOR_SUFFIX_ARTIFACT_CID
        || suffix_bytes.len() != predecessor_freeze.artifact_bytes
        || suffix.construction_document_count() != CONSTRUCTION_DOCUMENTS as u32
        || suffix.construction_position_count() != CONSTRUCTION_POSITIONS
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "suffix artifact does not match the frozen #1010 predecessor".to_owned(),
        ));
    }

    let (maximum_token_id, fit_sequences) = fit_sequences(&trace_bundle)?;
    if maximum_token_id != MAXIMUM_TOKEN_ID {
        return Err(R4SoftmaxTraceStateExperimentError::Trace(format!(
            "construction maximum token id is {maximum_token_id}; {MAXIMUM_TOKEN_ID} required"
        )));
    }
    let fit_config = R4SoftmaxTraceStateFitConfig { maximum_token_id };
    let artifact = compile_r4_softmax_trace_state_student(fit_config, &suffix, &fit_sequences)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?;
    let replay_artifact =
        compile_r4_softmax_trace_state_student(fit_config, &suffix, &fit_sequences)
            .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?;
    let artifact_bytes = artifact.to_bytes();
    let replay_bytes = replay_artifact.to_bytes();
    let artifact_cid = artifact.artifact_cid();
    let deterministic_compile_bytes_exact = artifact_bytes == replay_bytes;
    let deterministic_compile_cid_exact = artifact_cid == replay_artifact.artifact_cid();
    let reloaded = R4SoftmaxTraceStateStudentArtifact::from_bytes_with_expected_cid(
        &artifact_bytes,
        &artifact_cid,
    )
    .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?;
    let artifact_reload_bytes_exact = reloaded.to_bytes() == artifact_bytes;
    let artifact_reload_cid_exact = reloaded.artifact_cid() == artifact_cid;
    if !(deterministic_compile_bytes_exact
        && deterministic_compile_cid_exact
        && artifact_reload_bytes_exact
        && artifact_reload_cid_exact)
    {
        return Err(R4SoftmaxTraceStateExperimentError::Student(
            "state artifact determinism or canonical reload failed".to_owned(),
        ));
    }

    write_atomic(&config.artifact_output, &artifact_bytes)?;
    let mut freeze = R4SoftmaxTraceStateFreeze {
        schema: STATE_FREEZE_SCHEMA.to_owned(),
        issue: 1011,
        implementation_revision: config.implementation_revision.clone(),
        predecessor_freeze_cid: PREDECESSOR_FREEZE_CID.to_owned(),
        predecessor_result_cid_expected_at_reveal: PREDECESSOR_RESULT_CID.to_owned(),
        construction_trace_bundle_cid: CONSTRUCTION_TRACE_BUNDLE_CID.to_owned(),
        construction_trace_bundle_bytes: trace_bytes.len(),
        construction_document_trace_cids: expected_document_cids,
        suffix_artifact_cid: PREDECESSOR_SUFFIX_ARTIFACT_CID.to_owned(),
        suffix_artifact_bytes: suffix_bytes.len(),
        state_artifact_cid: artifact_cid,
        state_artifact_bytes: artifact_bytes.len(),
        state_artifact_payload_bytes_before_headers: artifact.payload_bytes_before_headers(),
        construction_digest: format!("blake3:{}", hex::encode(artifact.construction_digest())),
        construction_documents: artifact.construction_document_count(),
        construction_positions: artifact.construction_position_count(),
        runtime_state_bytes_per_arm: R4_SOFTMAX_TRACE_STATE_BYTES,
        parameter_values_per_fitted_arm: R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM,
        fitted_parameter_values_per_arm: R4_SOFTMAX_TRACE_STATE_FITTED_VALUES_PER_ARM,
        parameter_bytes_per_fitted_arm: R4_SOFTMAX_TRACE_STATE_PARAMETER_BYTES_PER_ARM,
        deterministic_compile_bytes_exact,
        deterministic_compile_cid_exact,
        artifact_reload_bytes_exact,
        artifact_reload_cid_exact,
        compiler_input_audit: StateCompilerInputAudit {
            construction_trace_bundle_reads: 1,
            predecessor_freeze_reads: 1,
            suffix_artifact_reads: 1,
            source_model_reads: 0,
            held_out_causal_input_reads: 0,
            judge_reads: 0,
        },
        compile_seconds: started.elapsed().as_secs_f64(),
        freeze_cid: String::new(),
        nonclaims: vec![
            "This construction-only fit does not establish held-out quality, coherent generation, reasoning, or exact deployed lowering."
                .to_owned(),
            "No source checkpoint or held-out judge is accepted by this command.".to_owned(),
        ],
    };
    freeze.freeze_cid =
        canonical_json_cid_omitting_fields(&freeze, &["compile_seconds", "freeze_cid"])?;
    write_json_atomic(&config.freeze_output, &freeze)?;
    Ok(freeze)
}

/// Phase 2: execute and seal every arm from causal token inputs only.
pub fn seal_source_free(
    config: &R4SoftmaxTraceStateSealConfig,
) -> Result<R4SoftmaxTraceStateSeal, R4SoftmaxTraceStateExperimentError> {
    let started = Instant::now();
    validate_distinct_paths(&[
        &config.artifact,
        &config.freeze,
        &config.causal_input,
        &config.seal_output,
    ])?;
    let artifact_bytes = fs::read(&config.artifact)?;
    let artifact = R4SoftmaxTraceStateStudentArtifact::from_bytes(&artifact_bytes)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?;
    if artifact.construction_document_count() != CONSTRUCTION_DOCUMENTS as u32
        || artifact.construction_position_count() != CONSTRUCTION_POSITIONS
        || artifact.suffix_artifact().artifact_cid() != PREDECESSOR_SUFFIX_ARTIFACT_CID
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "state artifact is not the frozen four-document successor".to_owned(),
        ));
    }
    let freeze_bytes = fs::read(&config.freeze)?;
    let freeze_value: serde_json::Value = serde_json::from_slice(&freeze_bytes)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Serialization(error.to_string()))?;
    validate_state_freeze_json_shape(&freeze_value)?;
    let raw_freeze_cid =
        canonical_value_cid_omitting_fields(freeze_value, &["compile_seconds", "freeze_cid"])?;
    let freeze: R4SoftmaxTraceStateFreeze = serde_json::from_slice(&freeze_bytes)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Serialization(error.to_string()))?;
    validate_state_freeze(&freeze, &raw_freeze_cid, &artifact, artifact_bytes.len())?;
    let causal_bytes = fs::read(&config.causal_input)?;
    if bytes_cid(&causal_bytes) != CAUSAL_INPUT_CID {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "causal-input-only JSON CID is not the frozen #1011 input".to_owned(),
        ));
    }
    let causal: CausalInput = serde_json::from_slice(&causal_bytes)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Serialization(error.to_string()))?;
    validate_causal_input(&causal, artifact.maximum_token_id())?;
    let held_out_frames = canonical_frame_offsets(
        &causal.held_out_document.input_tokens,
        artifact.maximum_token_id(),
    )?;
    let allowed_runtime_input_digest = canonical_json_cid(&serde_json::json!({
        "artifact_cid": artifact.artifact_cid(),
        "state_freeze_cid": &freeze.freeze_cid,
        "causal_input_cid": CAUSAL_INPUT_CID,
        "held_out_tokens": &causal.held_out_document.input_tokens,
        "held_out_frames": &held_out_frames,
        "continuation": &causal.continuation,
    }))?;
    let arms = execute_all_arms(&artifact, &causal, &held_out_frames)?;
    let replay_arms = execute_all_arms(&artifact, &causal, &held_out_frames)?;
    let exact_replay = arms == replay_arms;
    validate_prediction_sets(&arms, artifact.maximum_token_id())?;
    if !exact_replay || arms.iter().any(|arm| !runtime_audits_are_source_free(arm)) {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "source-free state execution failed replay or provenance audit".to_owned(),
        ));
    }
    let mut seal = R4SoftmaxTraceStateSeal {
        schema: STATE_SEAL_SCHEMA.to_owned(),
        issue: 1011,
        state_artifact_cid: artifact.artifact_cid(),
        state_artifact_bytes: artifact_bytes.len(),
        state_freeze_cid: freeze.freeze_cid,
        maximum_token_id: artifact.maximum_token_id(),
        causal_input_cid: CAUSAL_INPUT_CID.to_owned(),
        allowed_runtime_input_digest,
        held_out_document_id: causal.held_out_document.id,
        held_out_document_text_cid: causal.held_out_document.text_cid,
        held_out_input_positions: causal.held_out_document.input_tokens.len(),
        held_out_input_tokens: causal.held_out_document.input_tokens,
        held_out_frame_table_offsets: held_out_frames,
        arms,
        exact_replay,
        input_audit: StateSealInputAudit {
            artifact_file_reads: 1,
            state_freeze_file_reads: 1,
            causal_input_file_reads: 1,
            construction_trace_reads: 0,
            predecessor_freeze_reads: 0,
            source_model_reads: 0,
            source_model_forwards: 0,
            judge_reads: 0,
            target_reads: 0,
            future_token_reads: 0,
        },
        seal_seconds: started.elapsed().as_secs_f64(),
        seal_cid: String::new(),
        nonclaims: vec![
            "Sealed predictions are unscored until the independently frozen #1010 judge is opened."
                .to_owned(),
            "Token-ID continuation is not a claim of coherent decoded text or reasoning."
                .to_owned(),
        ],
    };
    seal.seal_cid = canonical_json_cid_omitting_fields(&seal, &["seal_seconds", "seal_cid"])?;
    validate_seal(&seal)?;
    write_json_atomic(&config.seal_output, &seal)?;
    Ok(seal)
}

/// Phase 3: reveal the immutable #1010 judge and score the immutable seal.
pub fn reveal(
    config: &R4SoftmaxTraceStateRevealConfig,
) -> Result<R4SoftmaxTraceStateResult, R4SoftmaxTraceStateExperimentError> {
    let started = Instant::now();
    validate_distinct_paths(&[&config.seal, &config.judge, &config.result_output])?;
    let seal_bytes = fs::read(&config.seal)?;
    let seal_value: serde_json::Value = serde_json::from_slice(&seal_bytes)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Serialization(error.to_string()))?;
    validate_state_seal_json_shape(&seal_value)?;
    let seal: R4SoftmaxTraceStateSeal = serde_json::from_slice(&seal_bytes)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Serialization(error.to_string()))?;
    validate_seal(&seal)?;

    let judge_bytes = fs::read(&config.judge)?;
    let judge_value: serde_json::Value = serde_json::from_slice(&judge_bytes)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Serialization(error.to_string()))?;
    let computed_judge_cid =
        canonical_value_cid_omitting_fields(judge_value, &["reveal_seconds", "result_cid"])?;
    let judge: PredecessorJudge = serde_json::from_slice(&judge_bytes)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Serialization(error.to_string()))?;
    if judge.result_cid != PREDECESSOR_RESULT_CID
        || computed_judge_cid != PREDECESSOR_RESULT_CID
        || judge.construction_freeze_cid != PREDECESSOR_FREEZE_CID
        || judge.artifact_cid_before_reveal != PREDECESSOR_SUFFIX_ARTIFACT_CID
        || judge.held_out_judge_sequence.document_id != seal.held_out_document_id
        || judge.held_out_judge_sequence.input_tokens.len() != seal.held_out_input_positions
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "revealed judge does not match the frozen #1010 result".to_owned(),
        ));
    }
    validate_judge_alignment(&seal, &judge.held_out_judge_sequence)?;

    let metrics = seal
        .arms
        .iter()
        .map(|arm| score_arm(arm, &judge.held_out_judge_sequence))
        .collect::<Result<Vec<_>, _>>()?;
    let geometric = metric(&metrics, R4SoftmaxTraceStateArm::GeometricRecurrent)?;
    let plain = metric(&metrics, R4SoftmaxTraceStateArm::PlainRecurrent)?;
    let permuted = metric(&metrics, R4SoftmaxTraceStateArm::TransportPermutedControl)?;
    let suffix = metric(&metrics, R4SoftmaxTraceStateArm::Suffix)?;
    if metrics
        .iter()
        .any(|metric| metric.teacher_mass_covered_q16 != FROZEN_TEACHER_MASS_COVERED_Q16)
        || suffix.positions != FROZEN_CONTEXT_POSITIONS
        || suffix.teacher_top1_agreements != PREDECESSOR_TEACHER_TOP1
        || suffix.actual_next_top1_correct != PREDECESSOR_ACTUAL_TOP1
        || !approximately_equal(
            suffix.covered_teacher_cross_entropy_nats,
            Some(PREDECESSOR_COVERED_CROSS_ENTROPY_NATS),
            1.0e-5,
        )
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "sealed suffix arm does not reproduce the frozen predecessor baseline".to_owned(),
        ));
    }
    let geometric_ce = required_ce(geometric)?;
    let plain_ce = required_ce(plain)?;
    let permuted_ce = required_ce(permuted)?;
    let geometry_control_distinct_witnesses = distinct_geometry_control_witnesses(&seal)?;
    let geometric_ce_below_predecessor = geometric_ce < PREDECESSOR_COVERED_CROSS_ENTROPY_NATS;
    let geometric_ce_below_plain = geometric_ce < plain_ce;
    let geometric_ce_below_permuted = geometric_ce < permuted_ce;
    let geometric_teacher_top1_above_predecessor =
        geometric.teacher_top1_agreements > PREDECESSOR_TEACHER_TOP1;
    let geometric_actual_top1_above_predecessor =
        geometric.actual_next_top1_correct > PREDECESSOR_ACTUAL_TOP1;
    let material_permuted_ce_loss = permuted_ce - geometric_ce >= MATERIAL_CONTROL_CE_DELTA_NATS;
    let material_permuted_teacher_top1_loss =
        geometric.teacher_top1_agreements > permuted.teacher_top1_agreements;
    let geometry_control_state_and_output_distinct = geometry_control_distinct_witnesses > 0;
    let zero_forbidden_runtime_reads = seal.input_audit.source_model_reads == 0
        && seal.input_audit.source_model_forwards == 0
        && seal.input_audit.judge_reads == 0
        && seal.input_audit.target_reads == 0
        && seal.input_audit.future_token_reads == 0
        && seal.arms.iter().all(runtime_audits_are_source_free);
    let geometric_seal = arm_seal(&seal, R4SoftmaxTraceStateArm::GeometricRecurrent)?;
    let geometric_continuation_has_no_period_one_or_two_cycle =
        !has_terminal_cycle(&geometric_seal.continuation.generated_token_ids, 1)
            && !has_terminal_cycle(&geometric_seal.continuation.generated_token_ids, 2);
    let promotion_passed = geometric_ce_below_predecessor
        && geometric_ce_below_plain
        && geometric_ce_below_permuted
        && geometric_teacher_top1_above_predecessor
        && geometric_actual_top1_above_predecessor
        && material_permuted_ce_loss
        && material_permuted_teacher_top1_loss
        && geometry_control_state_and_output_distinct
        && seal.exact_replay
        && zero_forbidden_runtime_reads
        && geometric_continuation_has_no_period_one_or_two_cycle;

    let mut result = R4SoftmaxTraceStateResult {
        schema: STATE_RESULT_SCHEMA.to_owned(),
        issue: 1011,
        state_seal_cid: seal.seal_cid,
        state_artifact_cid: seal.state_artifact_cid,
        state_freeze_cid: seal.state_freeze_cid,
        predecessor_result_cid: PREDECESSOR_RESULT_CID.to_owned(),
        predecessor_freeze_cid: PREDECESSOR_FREEZE_CID.to_owned(),
        causal_input_cid: CAUSAL_INPUT_CID.to_owned(),
        metrics,
        context_positions: FROZEN_CONTEXT_POSITIONS,
        geometry_control_distinct_witnesses,
        geometric_ce_below_predecessor,
        geometric_ce_below_plain,
        geometric_ce_below_permuted,
        geometric_teacher_top1_above_predecessor,
        geometric_actual_top1_above_predecessor,
        material_permuted_ce_loss,
        material_permuted_teacher_top1_loss,
        geometry_control_state_and_output_distinct,
        exact_replay: seal.exact_replay,
        zero_forbidden_runtime_reads,
        geometric_continuation_has_no_period_one_or_two_cycle,
        promotion_passed,
        terminal: if promotion_passed {
            "PASS_R4_SOFTMAX_TRACE_STATE_STUDENT_ADVANCE_LARGER_QUALIFICATION".to_owned()
        } else {
            "STOP_R4_SOFTMAX_TRACE_STATE_STUDENT_REPAIR_OR_RETIRE_REPRESENTATION".to_owned()
        },
        sealed_arms: seal.arms,
        reveal_seconds: started.elapsed().as_secs_f64(),
        result_cid: String::new(),
        nonclaims: vec![
            "This is a nine-position shared-support result, not full-vocabulary likelihood or general language modeling."
                .to_owned(),
            "A positive result would establish only bounded recurrent representation evidence, not reasoning or deployed exact runtime."
                .to_owned(),
        ],
    };
    result.result_cid =
        canonical_json_cid_omitting_fields(&result, &["reveal_seconds", "result_cid"])?;
    write_json_atomic(&config.result_output, &result)?;
    Ok(result)
}

fn validate_predecessor_freeze(
    bytes: &[u8],
    freeze: &R4SoftmaxTraceFreeze,
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Serialization(error.to_string()))?;
    let computed = canonical_value_cid_omitting_fields(value, &["compile_seconds", "freeze_cid"])?;
    if freeze.freeze_cid != PREDECESSOR_FREEZE_CID
        || computed != PREDECESSOR_FREEZE_CID
        || freeze.trace_bundle_cid != CONSTRUCTION_TRACE_BUNDLE_CID
        || freeze.artifact_cid != PREDECESSOR_SUFFIX_ARTIFACT_CID
        || freeze.construction_positions != CONSTRUCTION_POSITIONS as usize
        || freeze.construction_documents.len() != CONSTRUCTION_DOCUMENTS
        || freeze.held_out_teacher_scored
        || freeze.held_out_identity_bound_into_artifact
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "predecessor freeze identity or construction boundary is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn validate_state_freeze(
    freeze: &R4SoftmaxTraceStateFreeze,
    raw_freeze_cid: &str,
    artifact: &R4SoftmaxTraceStateStudentArtifact,
    artifact_bytes: usize,
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    validate_revision(&freeze.implementation_revision)?;
    let computed = canonical_json_cid_omitting_fields(freeze, &["compile_seconds", "freeze_cid"])?;
    let expected_document_cids = EXPECTED_DOCUMENT_TRACE_CIDS
        .iter()
        .map(|cid| (*cid).to_owned())
        .collect::<Vec<_>>();
    if freeze.schema != STATE_FREEZE_SCHEMA
        || freeze.issue != 1011
        || freeze.freeze_cid != computed
        || freeze.freeze_cid != raw_freeze_cid
        || freeze.predecessor_freeze_cid != PREDECESSOR_FREEZE_CID
        || freeze.predecessor_result_cid_expected_at_reveal != PREDECESSOR_RESULT_CID
        || freeze.construction_trace_bundle_cid != CONSTRUCTION_TRACE_BUNDLE_CID
        || freeze.construction_trace_bundle_bytes != CONSTRUCTION_TRACE_BUNDLE_BYTES
        || freeze.construction_document_trace_cids != expected_document_cids
        || freeze.suffix_artifact_cid != PREDECESSOR_SUFFIX_ARTIFACT_CID
        || freeze.suffix_artifact_bytes != PREDECESSOR_SUFFIX_ARTIFACT_BYTES
        || freeze.state_artifact_cid != artifact.artifact_cid()
        || artifact.maximum_token_id() != MAXIMUM_TOKEN_ID
        || freeze.state_artifact_bytes != STATE_ARTIFACT_BYTES
        || freeze.state_artifact_bytes != artifact_bytes
        || freeze.state_artifact_payload_bytes_before_headers
            != artifact.payload_bytes_before_headers()
        || freeze.construction_digest
            != format!("blake3:{}", hex::encode(artifact.construction_digest()))
        || freeze.construction_documents != CONSTRUCTION_DOCUMENTS as u32
        || freeze.construction_documents != artifact.construction_document_count()
        || freeze.construction_positions != CONSTRUCTION_POSITIONS
        || freeze.construction_positions != artifact.construction_position_count()
        || freeze.runtime_state_bytes_per_arm != R4_SOFTMAX_TRACE_STATE_BYTES
        || freeze.parameter_values_per_fitted_arm != R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM
        || freeze.fitted_parameter_values_per_arm != R4_SOFTMAX_TRACE_STATE_FITTED_VALUES_PER_ARM
        || freeze.parameter_bytes_per_fitted_arm != R4_SOFTMAX_TRACE_STATE_PARAMETER_BYTES_PER_ARM
        || !(freeze.deterministic_compile_bytes_exact
            && freeze.deterministic_compile_cid_exact
            && freeze.artifact_reload_bytes_exact
            && freeze.artifact_reload_cid_exact)
        || freeze.compiler_input_audit.construction_trace_bundle_reads != 1
        || freeze.compiler_input_audit.predecessor_freeze_reads != 1
        || freeze.compiler_input_audit.suffix_artifact_reads != 1
        || freeze.compiler_input_audit.source_model_reads != 0
        || freeze.compiler_input_audit.held_out_causal_input_reads != 0
        || freeze.compiler_input_audit.judge_reads != 0
        || !freeze.compile_seconds.is_finite()
        || freeze.compile_seconds < 0.0
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "state freeze does not bind the exact phase-1 artifact and leak-free provenance"
                .to_owned(),
        ));
    }
    Ok(())
}

fn fit_sequences(
    bundle: &R4SoftmaxTeacherTraceBundle,
) -> Result<(u32, Vec<R4SoftmaxTraceStateFitSequence>), R4SoftmaxTraceStateExperimentError> {
    if bundle.traces().len() != CONSTRUCTION_DOCUMENTS {
        return Err(R4SoftmaxTraceStateExperimentError::Trace(
            "construction bundle must contain four documents".to_owned(),
        ));
    }
    let mut vocabulary = None;
    let mut sequences = Vec::with_capacity(CONSTRUCTION_DOCUMENTS);
    let mut total_positions = 0_u64;
    for (ordinal, trace) in bundle.traces().iter().enumerate() {
        if trace.identity.document_id != EXPECTED_DOCUMENT_IDS[ordinal]
            || trace.identity.document_id == "13"
        {
            return Err(R4SoftmaxTraceStateExperimentError::Trace(
                "construction document order or partition is invalid".to_owned(),
            ));
        }
        if vocabulary
            .replace(trace.bounds.vocabulary)
            .is_some_and(|previous| previous != trace.bounds.vocabulary)
        {
            return Err(R4SoftmaxTraceStateExperimentError::Trace(
                "construction trace vocabularies differ".to_owned(),
            ));
        }
        let mut events = Vec::with_capacity(trace.positions.len());
        for position in &trace.positions {
            let final_layer = position.layers.last().ok_or_else(|| {
                R4SoftmaxTraceStateExperimentError::Trace(
                    "construction position has no final layer".to_owned(),
                )
            })?;
            events.push(R4SoftmaxTraceStateFitEvent {
                position: position.position,
                observed_token: position.input_token,
                actual_next_token: position.logits.target_token,
                frame_table_offset: position.frame_table_offset,
                query_trace_r4: reduce_heads(
                    &final_layer.heads,
                    R4SoftmaxTraceReductionRole::Query,
                )?,
                key_trace_r4: reduce_heads(&final_layer.heads, R4SoftmaxTraceReductionRole::Key)?,
                value_trace_r4: reduce_heads(
                    &final_layer.heads,
                    R4SoftmaxTraceReductionRole::Value,
                )?,
                teacher_top_distribution: teacher_distribution(&position.logits).map_err(
                    |error| R4SoftmaxTraceStateExperimentError::Trace(error.to_string()),
                )?,
            });
        }
        total_positions = total_positions
            .checked_add(u64::try_from(events.len()).map_err(|_| {
                R4SoftmaxTraceStateExperimentError::Invalid(
                    "construction position census exceeds u64".to_owned(),
                )
            })?)
            .ok_or_else(|| {
                R4SoftmaxTraceStateExperimentError::Invalid(
                    "construction position census overflowed".to_owned(),
                )
            })?;
        sequences.push(R4SoftmaxTraceStateFitSequence {
            document_id: trace.identity.document_id.clone(),
            events,
        });
    }
    if total_positions != CONSTRUCTION_POSITIONS {
        return Err(R4SoftmaxTraceStateExperimentError::Trace(format!(
            "construction trace contains {total_positions} positions; {CONSTRUCTION_POSITIONS} required"
        )));
    }
    let vocabulary = vocabulary.ok_or_else(|| {
        R4SoftmaxTraceStateExperimentError::Trace("construction vocabulary is absent".to_owned())
    })?;
    let maximum_token_id = u32::try_from(vocabulary.checked_sub(1).ok_or_else(|| {
        R4SoftmaxTraceStateExperimentError::Trace("construction vocabulary is empty".to_owned())
    })?)
    .map_err(|_| {
        R4SoftmaxTraceStateExperimentError::Trace("construction vocabulary exceeds u32".to_owned())
    })?;
    Ok((maximum_token_id, sequences))
}

fn reduce_heads(
    heads: &[R4SoftmaxHeadTrace],
    role: R4SoftmaxTraceReductionRole,
) -> Result<[f32; 4], R4SoftmaxTraceStateExperimentError> {
    if heads.len() != 9 {
        return Err(R4SoftmaxTraceStateExperimentError::Trace(format!(
            "final trace layer has {} heads; 9 required",
            heads.len()
        )));
    }
    let mut blocks = Vec::with_capacity(9 * 16);
    for head in heads {
        let bits = match role {
            R4SoftmaxTraceReductionRole::Query => &head.query_gauge_bits,
            R4SoftmaxTraceReductionRole::Key => &head.current_key_query_gauge_bits,
            R4SoftmaxTraceReductionRole::Value => &head.current_value_query_gauge_bits,
        };
        if bits.len() != 64 {
            return Err(R4SoftmaxTraceStateExperimentError::Trace(
                "final trace head is not sixteen R4 blocks".to_owned(),
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
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))
}

fn validate_causal_input(
    input: &CausalInput,
    maximum_token_id: u32,
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    let required_forbidden = [
        "actual_next_tokens",
        "source_weights",
        "teacher_logits",
        "teacher_qkv",
        "teacher_top_distributions",
    ];
    let observed_forbidden = input
        .forbidden_fields
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if input.schema != "uor-r4.r4-softmax-trace-state-causal-input/1"
        || input.issue != 1011
        || input.tokenizer_cid != PINNED_TOKENIZER_CID
        || input.held_out_document.id != "13"
        || input.held_out_document.partition != "held_out_causal_inputs_only"
        || input.held_out_document.text_cid != HELD_OUT_DOCUMENT_TEXT_CID
        || input.held_out_document.input_tokens.len() != 57
        || input.held_out_document.input_tokens.first().copied() != Some(1)
        || input
            .held_out_document
            .input_tokens
            .iter()
            .any(|token| *token > maximum_token_id)
        || input.continuation.prompt != "He was born"
        || input.continuation.bos_token_id != 1
        || input.continuation.prompt_token_ids != [3681, 436, 3988]
        || input.continuation.maximum_new_tokens != 16
        || observed_forbidden.len() != required_forbidden.len()
        || required_forbidden
            .iter()
            .any(|field| !observed_forbidden.contains(field))
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "causal-input-only JSON identity or boundary is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn execute_all_arms(
    artifact: &R4SoftmaxTraceStateStudentArtifact,
    causal: &CausalInput,
    held_out_frames: &[u16],
) -> Result<Vec<StateArmSeal>, R4SoftmaxTraceStateExperimentError> {
    ARMS.iter()
        .map(|&arm| execute_arm(artifact, causal, held_out_frames, arm))
        .collect()
}

fn execute_arm(
    artifact: &R4SoftmaxTraceStateStudentArtifact,
    causal: &CausalInput,
    held_out_frames: &[u16],
    arm: R4SoftmaxTraceStateArm,
) -> Result<StateArmSeal, R4SoftmaxTraceStateExperimentError> {
    let mut runtime = artifact
        .runtime(arm)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?;
    let held_out_predictions = causal
        .held_out_document
        .input_tokens
        .iter()
        .zip(held_out_frames)
        .map(|(&token, &frame)| {
            runtime
                .observe_and_predict(token, frame)
                .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let held_out_runtime_audit = runtime.audit();
    let (continuation, continuation_runtime_audit) = run_continuation(artifact, causal, arm)?;
    Ok(StateArmSeal {
        arm,
        held_out_predictions,
        held_out_runtime_audit,
        continuation,
        continuation_runtime_audit,
    })
}

fn run_continuation(
    artifact: &R4SoftmaxTraceStateStudentArtifact,
    causal: &CausalInput,
    arm: R4SoftmaxTraceStateArm,
) -> Result<
    (StateContinuationSeal, R4SoftmaxTraceStateRuntimeAudit),
    R4SoftmaxTraceStateExperimentError,
> {
    let context = std::iter::once(causal.continuation.bos_token_id)
        .chain(causal.continuation.prompt_token_ids.iter().copied())
        .collect::<Vec<_>>();
    let capacity = context
        .len()
        .checked_add(causal.continuation.maximum_new_tokens)
        .ok_or_else(|| {
            R4SoftmaxTraceStateExperimentError::Invalid(
                "continuation capacity overflowed".to_owned(),
            )
        })?;
    let mut frame_oracle = R4SpinCausalAttentionTransport::new(
        artifact.maximum_token_id(),
        capacity,
        R4SpinTransportIntervention::Coherent,
    )
    .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?;
    let mut runtime = artifact
        .runtime(arm)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?;
    let mut position = 0;
    let mut current_prediction = None;
    let mut frame_table_offsets = Vec::with_capacity(capacity);
    for token in context {
        let frame = prepare_frame(&mut frame_oracle, token, position)?;
        frame_table_offsets.push(frame);
        current_prediction = Some(
            runtime
                .observe_and_predict(token, frame)
                .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?,
        );
        position += 1;
    }
    let mut generated_token_ids = Vec::with_capacity(causal.continuation.maximum_new_tokens);
    let mut predictions = Vec::with_capacity(causal.continuation.maximum_new_tokens);
    for index in 0..causal.continuation.maximum_new_tokens {
        let prediction = current_prediction.take().ok_or_else(|| {
            R4SoftmaxTraceStateExperimentError::Invalid(
                "continuation context produced no prediction".to_owned(),
            )
        })?;
        let token = prediction.token;
        predictions.push(prediction);
        generated_token_ids.push(token);
        if token == PINNED_EOS_TOKEN_ID || index + 1 == causal.continuation.maximum_new_tokens {
            break;
        }
        let frame = prepare_frame(&mut frame_oracle, token, position)?;
        frame_table_offsets.push(frame);
        current_prediction = Some(
            runtime
                .observe_and_predict(token, frame)
                .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?,
        );
        position += 1;
    }
    let terminal_period_one_cycle = has_terminal_cycle(&generated_token_ids, 1);
    let terminal_period_two_cycle = has_terminal_cycle(&generated_token_ids, 2);
    Ok((
        StateContinuationSeal {
            prompt: causal.continuation.prompt.clone(),
            bos_token_id: causal.continuation.bos_token_id,
            prompt_token_ids: causal.continuation.prompt_token_ids.clone(),
            maximum_new_tokens: causal.continuation.maximum_new_tokens,
            generated_token_ids,
            predictions,
            frame_table_offsets,
            terminal_period_one_cycle,
            terminal_period_two_cycle,
        },
        runtime.audit(),
    ))
}

fn canonical_frame_offsets(
    tokens: &[u32],
    maximum_token_id: u32,
) -> Result<Vec<u16>, R4SoftmaxTraceStateExperimentError> {
    let mut oracle = R4SpinCausalAttentionTransport::new(
        maximum_token_id,
        tokens.len(),
        R4SpinTransportIntervention::Coherent,
    )
    .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?;
    tokens
        .iter()
        .enumerate()
        .map(|(position, &token)| prepare_frame(&mut oracle, token, position))
        .collect()
}

fn prepare_frame(
    oracle: &mut R4SpinCausalAttentionTransport,
    token: u32,
    position: usize,
) -> Result<u16, R4SoftmaxTraceStateExperimentError> {
    CausalAttentionTransport::begin_position(oracle, token as usize, position);
    CausalAttentionTransport::status(oracle)
        .map_err(R4SoftmaxTraceStateExperimentError::Student)?;
    oracle
        .frame_table_offset(position)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))
}

fn runtime_audits_are_source_free(arm: &StateArmSeal) -> bool {
    [arm.held_out_runtime_audit, arm.continuation_runtime_audit]
        .iter()
        .all(|audit| {
            audit.source_model_forwards == 0
                && audit.source_trace_reads == 0
                && audit.teacher_distribution_reads == 0
                && audit.target_reads == 0
                && audit.future_token_reads == 0
                && audit.prior_state_reads == audit.observed_token_reads
                && audit.prior_state_reads == audit.canonical_frame_reads
                && audit.prior_state_reads == audit.artifact_reads
        })
}

fn validate_prediction_sets(
    arms: &[StateArmSeal],
    maximum_token_id: u32,
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    if arms.len() != ARMS.len()
        || arms
            .iter()
            .zip(ARMS)
            .any(|(arm, expected)| arm.arm != expected)
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "sealed arm census or order is invalid".to_owned(),
        ));
    }
    for arm in arms {
        for prediction in arm
            .held_out_predictions
            .iter()
            .chain(&arm.continuation.predictions)
        {
            validate_prediction(prediction, maximum_token_id)?;
        }
    }
    let reference = &arms[0].held_out_predictions;
    for arm in &arms[1..] {
        if arm.held_out_predictions.len() != reference.len() {
            return Err(R4SoftmaxTraceStateExperimentError::Invalid(
                "held-out prediction counts differ across matched arms".to_owned(),
            ));
        }
        for (reference, candidate) in reference.iter().zip(&arm.held_out_predictions) {
            if reference.suffix_depth != candidate.suffix_depth
                || reference
                    .candidates
                    .iter()
                    .map(|score| score.token)
                    .ne(candidate.candidates.iter().map(|score| score.token))
            {
                return Err(R4SoftmaxTraceStateExperimentError::Invalid(
                    "matched arms do not expose identical ordered candidate support".to_owned(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_prediction(
    prediction: &R4SoftmaxTraceStatePrediction,
    maximum_token_id: u32,
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    if prediction.candidates.is_empty()
        || prediction.token > maximum_token_id
        || prediction.suffix_depth > 4
        || !valid_blake3_cid(&prediction.state_checksum)
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "sealed prediction identity or shape is invalid".to_owned(),
        ));
    }
    let mut tokens = BTreeSet::new();
    let mut probability_sum = 0.0_f64;
    for candidate in &prediction.candidates {
        if candidate.token > maximum_token_id
            || !tokens.insert(candidate.token)
            || !candidate.probability.is_finite()
            || !(0.0..=1.0).contains(&candidate.probability)
            || candidate.probability == 0.0
            || !candidate.logit.is_finite()
        {
            return Err(R4SoftmaxTraceStateExperimentError::Invalid(
                "sealed candidate support is duplicated, out of range, or non-finite".to_owned(),
            ));
        }
        probability_sum += f64::from(candidate.probability);
    }
    if (probability_sum - 1.0).abs() > 1.0e-5 {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(format!(
            "sealed candidate probabilities sum to {probability_sum}, not one"
        )));
    }
    let winner = prediction
        .candidates
        .iter()
        .max_by(|left, right| {
            left.probability
                .total_cmp(&right.probability)
                .then_with(|| right.token.cmp(&left.token))
        })
        .expect("nonempty candidates checked above");
    if winner.token != prediction.token
        || winner.probability.to_bits() != prediction.probability.to_bits()
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "sealed prediction does not match its deterministic candidate winner".to_owned(),
        ));
    }
    Ok(())
}

fn validate_continuation(
    continuation: &StateContinuationSeal,
    maximum_token_id: u32,
) -> Result<usize, R4SoftmaxTraceStateExperimentError> {
    if continuation.prompt != "He was born"
        || continuation.bos_token_id != 1
        || continuation.prompt_token_ids != [3681, 436, 3988]
        || continuation.maximum_new_tokens != 16
        || continuation.generated_token_ids.is_empty()
        || continuation.generated_token_ids.len() > continuation.maximum_new_tokens
        || continuation.predictions.len() != continuation.generated_token_ids.len()
        || continuation
            .predictions
            .iter()
            .map(|prediction| prediction.token)
            .ne(continuation.generated_token_ids.iter().copied())
        || continuation
            .generated_token_ids
            .iter()
            .any(|token| *token > maximum_token_id)
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "sealed continuation identity, length, or prediction alignment is invalid".to_owned(),
        ));
    }
    let eos_position = continuation
        .generated_token_ids
        .iter()
        .position(|token| *token == PINNED_EOS_TOKEN_ID);
    if eos_position.is_some_and(|position| position + 1 != continuation.generated_token_ids.len())
        || (eos_position.is_none()
            && continuation.generated_token_ids.len() != continuation.maximum_new_tokens)
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "sealed continuation does not stop exactly at EOS or its frozen cap".to_owned(),
        ));
    }
    let observed_tokens = std::iter::once(continuation.bos_token_id)
        .chain(continuation.prompt_token_ids.iter().copied())
        .chain(
            continuation.generated_token_ids[..continuation.generated_token_ids.len() - 1]
                .iter()
                .copied(),
        )
        .collect::<Vec<_>>();
    let expected_frames = canonical_frame_offsets(&observed_tokens, maximum_token_id)?;
    if continuation.frame_table_offsets != expected_frames
        || continuation.terminal_period_one_cycle
            != has_terminal_cycle(&continuation.generated_token_ids, 1)
        || continuation.terminal_period_two_cycle
            != has_terminal_cycle(&continuation.generated_token_ids, 2)
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "sealed continuation frame or cycle evidence is not independently reproducible"
                .to_owned(),
        ));
    }
    Ok(observed_tokens.len())
}

fn validate_runtime_ledger(
    arm: &StateArmSeal,
    held_out_observations: usize,
    continuation_observations: usize,
) -> bool {
    let expected_held_out_transports = match arm.arm {
        R4SoftmaxTraceStateArm::GeometricRecurrent
        | R4SoftmaxTraceStateArm::TransportPermutedControl => {
            held_out_observations.saturating_sub(1)
        }
        _ => 0,
    };
    let expected_continuation_transports = match arm.arm {
        R4SoftmaxTraceStateArm::GeometricRecurrent
        | R4SoftmaxTraceStateArm::TransportPermutedControl => {
            continuation_observations.saturating_sub(1)
        }
        _ => 0,
    };
    let expected_held_out_permutations =
        if arm.arm == R4SoftmaxTraceStateArm::TransportPermutedControl {
            expected_held_out_transports
        } else {
            0
        };
    let expected_continuation_permutations =
        if arm.arm == R4SoftmaxTraceStateArm::TransportPermutedControl {
            expected_continuation_transports
        } else {
            0
        };
    audit_matches_work(
        arm.held_out_runtime_audit,
        held_out_observations,
        expected_held_out_transports,
        expected_held_out_permutations,
    ) && audit_matches_work(
        arm.continuation_runtime_audit,
        continuation_observations,
        expected_continuation_transports,
        expected_continuation_permutations,
    )
}

fn audit_matches_work(
    audit: R4SoftmaxTraceStateRuntimeAudit,
    observations: usize,
    transports: usize,
    permutations: usize,
) -> bool {
    let Ok(observations) = u64::try_from(observations) else {
        return false;
    };
    let Ok(transports) = u64::try_from(transports) else {
        return false;
    };
    let Ok(permutations) = u64::try_from(permutations) else {
        return false;
    };
    audit.prior_state_reads == observations
        && audit.observed_token_reads == observations
        && audit.canonical_frame_reads == observations
        && audit.artifact_reads == observations
        && audit.state_transports == transports
        && audit.transport_permutations == permutations
        && audit.source_model_forwards == 0
        && audit.source_trace_reads == 0
        && audit.teacher_distribution_reads == 0
        && audit.target_reads == 0
        && audit.future_token_reads == 0
}

fn validate_seal(seal: &R4SoftmaxTraceStateSeal) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    let computed = canonical_json_cid_omitting_fields(seal, &["seal_seconds", "seal_cid"])?;
    if seal.schema != STATE_SEAL_SCHEMA
        || seal.issue != 1011
        || !valid_blake3_cid(&seal.state_artifact_cid)
        || seal.state_artifact_bytes != STATE_ARTIFACT_BYTES
        || !valid_blake3_cid(&seal.state_freeze_cid)
        || seal.maximum_token_id != MAXIMUM_TOKEN_ID
        || seal.causal_input_cid != CAUSAL_INPUT_CID
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "source-free seal identity or bound CIDs are invalid".to_owned(),
        ));
    }
    if seal.seal_cid != computed {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(format!(
            "source-free seal CID is not canonical (declared {}, computed {computed})",
            seal.seal_cid
        )));
    }
    let recomputed_frames =
        canonical_frame_offsets(&seal.held_out_input_tokens, seal.maximum_token_id)?;
    if seal.held_out_document_id != "13"
        || seal.held_out_document_text_cid != HELD_OUT_DOCUMENT_TEXT_CID
        || seal.held_out_input_positions != 57
        || seal.held_out_input_tokens.len() != 57
        || seal.held_out_input_tokens.first().copied() != Some(1)
        || seal.held_out_frame_table_offsets.len() != 57
        || recomputed_frames != seal.held_out_frame_table_offsets
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "source-free seal causal token or frame census is invalid".to_owned(),
        ));
    }
    if seal.arms.len() != ARMS.len() || !seal.exact_replay {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "source-free seal arm census or exact replay flag is invalid".to_owned(),
        ));
    }
    let first_continuation = seal.arms.first().map(|arm| &arm.continuation);
    let continuation_observations = seal
        .arms
        .iter()
        .map(|arm| validate_continuation(&arm.continuation, seal.maximum_token_id))
        .collect::<Result<Vec<_>, _>>()?;
    let recomputed_input_digest = first_continuation
        .map(|continuation| {
            canonical_json_cid(&serde_json::json!({
                "artifact_cid": &seal.state_artifact_cid,
                "state_freeze_cid": &seal.state_freeze_cid,
                "causal_input_cid": CAUSAL_INPUT_CID,
                "held_out_tokens": &seal.held_out_input_tokens,
                "held_out_frames": &seal.held_out_frame_table_offsets,
                "continuation": {
                    "prompt": &continuation.prompt,
                    "bos_token_id": continuation.bos_token_id,
                    "prompt_token_ids": &continuation.prompt_token_ids,
                    "maximum_new_tokens": continuation.maximum_new_tokens,
                },
            }))
        })
        .transpose()?;
    validate_prediction_sets(&seal.arms, seal.maximum_token_id)?;
    if recomputed_input_digest.as_deref() != Some(&seal.allowed_runtime_input_digest) {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "source-free seal allowed-input digest is not independently reproducible".to_owned(),
        ));
    }
    let expected_input_audit = StateSealInputAudit {
        artifact_file_reads: 1,
        state_freeze_file_reads: 1,
        causal_input_file_reads: 1,
        construction_trace_reads: 0,
        predecessor_freeze_reads: 0,
        source_model_reads: 0,
        source_model_forwards: 0,
        judge_reads: 0,
        target_reads: 0,
        future_token_reads: 0,
    };
    if seal.input_audit != expected_input_audit {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "source-free seal input-read ledger is not exact".to_owned(),
        ));
    }
    let first_continuation = first_continuation.ok_or_else(|| {
        R4SoftmaxTraceStateExperimentError::Invalid(
            "source-free seal contains no continuation".to_owned(),
        )
    })?;
    for ((arm, expected), &continuation_observations) in
        seal.arms.iter().zip(ARMS).zip(&continuation_observations)
    {
        if arm.arm != expected || arm.held_out_predictions.len() != 57 {
            return Err(R4SoftmaxTraceStateExperimentError::Invalid(
                "source-free seal arm order or held-out census is invalid".to_owned(),
            ));
        }
        if arm.continuation.prompt != first_continuation.prompt
            || arm.continuation.bos_token_id != first_continuation.bos_token_id
            || arm.continuation.prompt_token_ids != first_continuation.prompt_token_ids
            || arm.continuation.maximum_new_tokens != first_continuation.maximum_new_tokens
        {
            return Err(R4SoftmaxTraceStateExperimentError::Invalid(
                "source-free seal continuation inputs differ across arms".to_owned(),
            ));
        }
        if !runtime_audits_are_source_free(arm)
            || !validate_runtime_ledger(arm, 57, continuation_observations)
        {
            return Err(R4SoftmaxTraceStateExperimentError::Invalid(format!(
                "source-free seal runtime ledger for {:?} is not exact",
                arm.arm
            )));
        }
    }
    Ok(())
}

fn validate_judge_alignment(
    seal: &R4SoftmaxTraceStateSeal,
    judge: &R4SoftmaxTraceSequence,
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    if judge.input_tokens.len() != 57
        || judge.input_tokens != seal.held_out_input_tokens
        || judge.actual_next_tokens.len() != 57
        || judge.teacher_top_distributions.len() != 57
        || seal
            .arms
            .iter()
            .any(|arm| arm.held_out_predictions.len() != judge.input_tokens.len())
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "judge and sealed prediction shapes differ".to_owned(),
        ));
    }
    Ok(())
}

fn score_arm(
    arm: &StateArmSeal,
    judge: &R4SoftmaxTraceSequence,
) -> Result<StateArmMetrics, R4SoftmaxTraceStateExperimentError> {
    let mut positions = 0_u64;
    let mut teacher_mass_covered_q16 = 0_u64;
    let mut teacher_top1_agreements = 0_u64;
    let mut actual_next_top1_correct = 0_u64;
    let mut cross_entropy = 0.0_f64;
    for (position, ((prediction, teacher), &actual)) in arm
        .held_out_predictions
        .iter()
        .zip(&judge.teacher_top_distributions)
        .zip(&judge.actual_next_tokens)
        .enumerate()
    {
        if prediction.suffix_depth == 0 || position == 0 {
            continue;
        }
        positions += 1;
        if prediction.token
            == teacher
                .top_token()
                .map_err(|error| R4SoftmaxTraceStateExperimentError::Student(error.to_string()))?
        {
            teacher_top1_agreements += 1;
        }
        if prediction.token == actual {
            actual_next_top1_correct += 1;
        }
        for entry in &teacher.entries {
            let Some(candidate) = prediction
                .candidates
                .iter()
                .find(|candidate| candidate.token == entry.token)
            else {
                continue;
            };
            if !candidate.probability.is_finite() || candidate.probability <= 0.0 {
                return Err(R4SoftmaxTraceStateExperimentError::Invalid(
                    "sealed candidate probability is non-finite or non-positive".to_owned(),
                ));
            }
            teacher_mass_covered_q16 += u64::from(entry.probability_q16);
            let teacher_probability =
                f64::from(entry.probability_q16) / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL);
            cross_entropy -= teacher_probability * f64::from(candidate.probability).ln();
        }
    }
    let covered_teacher_cross_entropy_nats = (teacher_mass_covered_q16 != 0).then_some(
        cross_entropy / (teacher_mass_covered_q16 as f64 / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL)),
    );
    if positions != FROZEN_CONTEXT_POSITIONS {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(format!(
            "arm {:?} exposed {positions} context positions; {FROZEN_CONTEXT_POSITIONS} required",
            arm.arm
        )));
    }
    Ok(StateArmMetrics {
        arm: arm.arm,
        positions,
        teacher_mass_covered_q16,
        teacher_top1_agreements,
        actual_next_top1_correct,
        covered_teacher_cross_entropy_nats,
    })
}

fn metric(
    metrics: &[StateArmMetrics],
    arm: R4SoftmaxTraceStateArm,
) -> Result<&StateArmMetrics, R4SoftmaxTraceStateExperimentError> {
    metrics
        .iter()
        .find(|metric| metric.arm == arm)
        .ok_or_else(|| {
            R4SoftmaxTraceStateExperimentError::Invalid(format!("metric for {arm:?} is absent"))
        })
}

fn arm_seal(
    seal: &R4SoftmaxTraceStateSeal,
    arm: R4SoftmaxTraceStateArm,
) -> Result<&StateArmSeal, R4SoftmaxTraceStateExperimentError> {
    seal.arms
        .iter()
        .find(|entry| entry.arm == arm)
        .ok_or_else(|| {
            R4SoftmaxTraceStateExperimentError::Invalid(format!("seal for {arm:?} is absent"))
        })
}

fn required_ce(metric: &StateArmMetrics) -> Result<f64, R4SoftmaxTraceStateExperimentError> {
    metric
        .covered_teacher_cross_entropy_nats
        .filter(|value| value.is_finite())
        .ok_or_else(|| {
            R4SoftmaxTraceStateExperimentError::Invalid(format!(
                "arm {:?} has no finite covered cross-entropy",
                metric.arm
            ))
        })
}

fn distinct_geometry_control_witnesses(
    seal: &R4SoftmaxTraceStateSeal,
) -> Result<u64, R4SoftmaxTraceStateExperimentError> {
    let geometric = arm_seal(seal, R4SoftmaxTraceStateArm::GeometricRecurrent)?;
    let permuted = arm_seal(seal, R4SoftmaxTraceStateArm::TransportPermutedControl)?;
    Ok(geometric
        .held_out_predictions
        .iter()
        .zip(&permuted.held_out_predictions)
        .enumerate()
        .filter(|(position, (geometric, permuted))| {
            *position != 0
                && geometric.suffix_depth > 0
                && geometric.state_checksum != permuted.state_checksum
                && geometric.candidates != permuted.candidates
        })
        .count() as u64)
}

fn has_terminal_cycle(tokens: &[u32], period: usize) -> bool {
    let width = period.saturating_mul(2);
    tokens.len() >= width
        && tokens[tokens.len() - width..tokens.len() - period] == tokens[tokens.len() - period..]
}

fn approximately_equal(left: Option<f64>, right: Option<f64>, tolerance: f64) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => (left - right).abs() <= tolerance,
        (None, None) => true,
        _ => false,
    }
}

fn validate_revision(revision: &str) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(R4SoftmaxTraceStateExperimentError::Invalid(
            "implementation revision must be an exact lowercase 40-character Git commit".to_owned(),
        ));
    }
    Ok(())
}

fn validate_distinct_paths(paths: &[&Path]) -> Result<(), R4SoftmaxTraceStateExperimentError> {
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
            return Err(R4SoftmaxTraceStateExperimentError::Invalid(
                "input and output paths must be pairwise distinct".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_state_freeze_json_shape(
    value: &serde_json::Value,
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    reject_unknown_fields(
        value,
        &[
            "schema",
            "issue",
            "implementation_revision",
            "predecessor_freeze_cid",
            "predecessor_result_cid_expected_at_reveal",
            "construction_trace_bundle_cid",
            "construction_trace_bundle_bytes",
            "construction_document_trace_cids",
            "suffix_artifact_cid",
            "suffix_artifact_bytes",
            "state_artifact_cid",
            "state_artifact_bytes",
            "state_artifact_payload_bytes_before_headers",
            "construction_digest",
            "construction_documents",
            "construction_positions",
            "runtime_state_bytes_per_arm",
            "parameter_values_per_fitted_arm",
            "fitted_parameter_values_per_arm",
            "parameter_bytes_per_fitted_arm",
            "deterministic_compile_bytes_exact",
            "deterministic_compile_cid_exact",
            "artifact_reload_bytes_exact",
            "artifact_reload_cid_exact",
            "compiler_input_audit",
            "compile_seconds",
            "freeze_cid",
            "nonclaims",
        ],
        "state freeze",
    )?;
    reject_unknown_fields(
        required_field(value, "compiler_input_audit", "state freeze")?,
        &[
            "construction_trace_bundle_reads",
            "predecessor_freeze_reads",
            "suffix_artifact_reads",
            "source_model_reads",
            "held_out_causal_input_reads",
            "judge_reads",
        ],
        "state freeze compiler input audit",
    )
}

fn validate_state_seal_json_shape(
    value: &serde_json::Value,
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    reject_unknown_fields(
        value,
        &[
            "schema",
            "issue",
            "state_artifact_cid",
            "state_artifact_bytes",
            "state_freeze_cid",
            "maximum_token_id",
            "causal_input_cid",
            "allowed_runtime_input_digest",
            "held_out_document_id",
            "held_out_document_text_cid",
            "held_out_input_positions",
            "held_out_input_tokens",
            "held_out_frame_table_offsets",
            "arms",
            "exact_replay",
            "input_audit",
            "seal_seconds",
            "seal_cid",
            "nonclaims",
        ],
        "state seal",
    )?;
    reject_unknown_fields(
        required_field(value, "input_audit", "state seal")?,
        &[
            "artifact_file_reads",
            "state_freeze_file_reads",
            "causal_input_file_reads",
            "construction_trace_reads",
            "predecessor_freeze_reads",
            "source_model_reads",
            "source_model_forwards",
            "judge_reads",
            "target_reads",
            "future_token_reads",
        ],
        "state seal input audit",
    )?;
    let arms = required_field(value, "arms", "state seal")?
        .as_array()
        .ok_or_else(|| {
            R4SoftmaxTraceStateExperimentError::Serialization(
                "state seal arms is not an array".to_owned(),
            )
        })?;
    for arm in arms {
        reject_unknown_fields(
            arm,
            &[
                "arm",
                "held_out_predictions",
                "held_out_runtime_audit",
                "continuation",
                "continuation_runtime_audit",
            ],
            "state seal arm",
        )?;
        validate_runtime_audit_json_shape(required_field(
            arm,
            "held_out_runtime_audit",
            "state seal arm",
        )?)?;
        validate_runtime_audit_json_shape(required_field(
            arm,
            "continuation_runtime_audit",
            "state seal arm",
        )?)?;
        validate_prediction_array_json_shape(required_field(
            arm,
            "held_out_predictions",
            "state seal arm",
        )?)?;
        let continuation = required_field(arm, "continuation", "state seal arm")?;
        reject_unknown_fields(
            continuation,
            &[
                "prompt",
                "bos_token_id",
                "prompt_token_ids",
                "maximum_new_tokens",
                "generated_token_ids",
                "predictions",
                "frame_table_offsets",
                "terminal_period_one_cycle",
                "terminal_period_two_cycle",
            ],
            "state seal continuation",
        )?;
        validate_prediction_array_json_shape(required_field(
            continuation,
            "predictions",
            "state seal continuation",
        )?)?;
    }
    Ok(())
}

fn validate_runtime_audit_json_shape(
    value: &serde_json::Value,
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    reject_unknown_fields(
        value,
        &[
            "prior_state_reads",
            "observed_token_reads",
            "canonical_frame_reads",
            "artifact_reads",
            "state_transports",
            "transport_permutations",
            "source_model_forwards",
            "source_trace_reads",
            "teacher_distribution_reads",
            "target_reads",
            "future_token_reads",
        ],
        "state runtime audit",
    )
}

fn validate_prediction_array_json_shape(
    value: &serde_json::Value,
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    let predictions = value.as_array().ok_or_else(|| {
        R4SoftmaxTraceStateExperimentError::Serialization(
            "state predictions is not an array".to_owned(),
        )
    })?;
    for prediction in predictions {
        reject_unknown_fields(
            prediction,
            &[
                "token",
                "probability",
                "suffix_depth",
                "candidates",
                "state_checksum",
            ],
            "state prediction",
        )?;
        let candidates = required_field(prediction, "candidates", "state prediction")?
            .as_array()
            .ok_or_else(|| {
                R4SoftmaxTraceStateExperimentError::Serialization(
                    "state prediction candidates is not an array".to_owned(),
                )
            })?;
        for candidate in candidates {
            reject_unknown_fields(
                candidate,
                &["token", "probability", "logit"],
                "state prediction candidate",
            )?;
        }
    }
    Ok(())
}

fn required_field<'a>(
    value: &'a serde_json::Value,
    field: &str,
    path: &str,
) -> Result<&'a serde_json::Value, R4SoftmaxTraceStateExperimentError> {
    value.get(field).ok_or_else(|| {
        R4SoftmaxTraceStateExperimentError::Serialization(format!("{path} field {field} is absent"))
    })
}

fn reject_unknown_fields(
    value: &serde_json::Value,
    allowed: &[&str],
    path: &str,
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    let object = value.as_object().ok_or_else(|| {
        R4SoftmaxTraceStateExperimentError::Serialization(format!("{path} is not an object"))
    })?;
    if let Some(field) = object
        .keys()
        .find(|field| !allowed.contains(&field.as_str()))
    {
        return Err(R4SoftmaxTraceStateExperimentError::Serialization(format!(
            "{path} contains unknown field {field}"
        )));
    }
    Ok(())
}

fn canonical_json_cid<T: Serialize>(
    value: &T,
) -> Result<String, R4SoftmaxTraceStateExperimentError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Serialization(error.to_string()))?;
    Ok(bytes_cid(&bytes))
}

fn canonical_json_cid_omitting_fields<T: Serialize>(
    value: &T,
    fields: &[&str],
) -> Result<String, R4SoftmaxTraceStateExperimentError> {
    let mut value = serde_json::to_value(value)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Serialization(error.to_string()))?;
    let object = value.as_object_mut().ok_or_else(|| {
        R4SoftmaxTraceStateExperimentError::Serialization(
            "CID-bearing JSON is not an object".to_owned(),
        )
    })?;
    for field in fields {
        if object.remove(*field).is_none() {
            return Err(R4SoftmaxTraceStateExperimentError::Serialization(format!(
                "CID field {field} is absent"
            )));
        }
    }
    canonical_json_cid(&value)
}

fn canonical_value_cid_omitting_fields(
    mut value: serde_json::Value,
    fields: &[&str],
) -> Result<String, R4SoftmaxTraceStateExperimentError> {
    let object = value.as_object_mut().ok_or_else(|| {
        R4SoftmaxTraceStateExperimentError::Serialization(
            "CID-bearing JSON is not an object".to_owned(),
        )
    })?;
    for field in fields {
        if object.remove(*field).is_none() {
            return Err(R4SoftmaxTraceStateExperimentError::Serialization(format!(
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
) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| R4SoftmaxTraceStateExperimentError::Serialization(error.to_string()))?;
    write_atomic(path, &bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), R4SoftmaxTraceStateExperimentError> {
    let parent = path.parent().ok_or_else(|| {
        R4SoftmaxTraceStateExperimentError::Invalid(format!(
            "output {} has no parent directory",
            path.display()
        ))
    })?;
    fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            R4SoftmaxTraceStateExperimentError::Invalid(format!(
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

    #[test]
    fn frozen_causal_input_is_exact_and_contains_no_judge_fields() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs/r4_softmax_trace_state_causal_input_1011.json");
        let bytes = fs::read(path).expect("frozen causal input");
        assert_eq!(bytes_cid(&bytes), CAUSAL_INPUT_CID);
        let input: CausalInput = serde_json::from_slice(&bytes).expect("strict causal input");
        validate_causal_input(&input, 49_151).expect("valid frozen causal input");
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("actual_next_tokens\":"));
        assert!(!text.contains("teacher_top_distributions\":"));
    }

    #[test]
    fn terminal_cycle_checks_distinguish_period_one_and_two() {
        assert!(has_terminal_cycle(&[1, 2, 2], 1));
        assert!(has_terminal_cycle(&[1, 2, 3, 2, 3], 2));
        assert!(!has_terminal_cycle(&[1, 2, 3, 4], 1));
        assert!(!has_terminal_cycle(&[1, 2, 3, 4], 2));
    }

    #[test]
    fn evidence_cids_exclude_elapsed_time_and_self_identity() {
        #[derive(Serialize)]
        struct Evidence<'a> {
            stable: &'a str,
            elapsed: f64,
            cid: &'a str,
        }
        let first = Evidence {
            stable: "same",
            elapsed: 1.0,
            cid: "first",
        };
        let second = Evidence {
            stable: "same",
            elapsed: 99.0,
            cid: "second",
        };
        assert_eq!(
            canonical_json_cid_omitting_fields(&first, &["elapsed", "cid"]).unwrap(),
            canonical_json_cid_omitting_fields(&second, &["elapsed", "cid"]).unwrap()
        );
    }

    #[test]
    fn seal_shape_rejects_top_level_and_nested_unknown_fields() {
        let top_level = serde_json::json!({
            "input_audit": {},
            "arms": [],
            "forbidden_teacher_rows": [],
        });
        assert!(validate_state_seal_json_shape(&top_level).is_err());

        let runtime_audit = serde_json::json!({
            "prior_state_reads": 0,
            "observed_token_reads": 0,
            "canonical_frame_reads": 0,
            "artifact_reads": 0,
            "state_transports": 0,
            "transport_permutations": 0,
            "source_model_forwards": 0,
            "source_trace_reads": 0,
            "teacher_distribution_reads": 0,
            "target_reads": 0,
            "future_token_reads": 0,
        });
        let nested = serde_json::json!({
            "input_audit": {},
            "arms": [{
                "held_out_predictions": [{
                    "token": 28,
                    "probability": 1.0,
                    "suffix_depth": 1,
                    "candidates": [{"token": 28, "probability": 1.0, "logit": 0.0}],
                    "state_checksum": format!("blake3:{}", "0".repeat(64)),
                    "forbidden_teacher_rows": [],
                }],
                "held_out_runtime_audit": runtime_audit.clone(),
                "continuation": {"predictions": []},
                "continuation_runtime_audit": runtime_audit,
            }],
        });
        assert!(validate_state_seal_json_shape(&nested).is_err());
    }

    #[test]
    fn prediction_and_continuation_derived_fields_fail_closed() {
        let malformed: R4SoftmaxTraceStatePrediction = serde_json::from_value(serde_json::json!({
            "token": 28,
            "probability": 0.5,
            "suffix_depth": 1,
            "candidates": [
                {"token": 28, "probability": 0.5, "logit": 0.0},
                {"token": 28, "probability": 0.5, "logit": 0.0}
            ],
            "state_checksum": format!("blake3:{}", "0".repeat(64)),
        }))
        .unwrap();
        assert!(validate_prediction(&malformed, MAXIMUM_TOKEN_ID).is_err());

        let prediction: R4SoftmaxTraceStatePrediction = serde_json::from_value(serde_json::json!({
            "token": 28,
            "probability": 1.0,
            "suffix_depth": 1,
            "candidates": [{"token": 28, "probability": 1.0, "logit": 0.0}],
            "state_checksum": format!("blake3:{}", "0".repeat(64)),
        }))
        .unwrap();
        let generated_token_ids = vec![28; 16];
        let observed_tokens = std::iter::once(1)
            .chain([3681, 436, 3988])
            .chain(generated_token_ids[..15].iter().copied())
            .collect::<Vec<_>>();
        let mut continuation = StateContinuationSeal {
            prompt: "He was born".to_owned(),
            bos_token_id: 1,
            prompt_token_ids: vec![3681, 436, 3988],
            maximum_new_tokens: 16,
            generated_token_ids,
            predictions: vec![prediction; 16],
            frame_table_offsets: canonical_frame_offsets(&observed_tokens, MAXIMUM_TOKEN_ID)
                .unwrap(),
            terminal_period_one_cycle: true,
            terminal_period_two_cycle: true,
        };
        assert_eq!(
            validate_continuation(&continuation, MAXIMUM_TOKEN_ID).unwrap(),
            19
        );
        continuation.generated_token_ids[0] = 29;
        assert!(validate_continuation(&continuation, MAXIMUM_TOKEN_ID).is_err());
    }
}
