//! Model-agnostic local generation through the established R4/Spin causal-softmax seam.
//!
//! Unlike the frozen #973 reference wrapper, this module accepts any local
//! Hugging Face Llama checkpoint admitted by `HuggingFaceLlamaOracle`. It has
//! no network/provider fallback and keeps the checkpoint tree identity
//! distinct from the loader's weights-only CID.

use std::fmt;
use std::io;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use uor_r4_core::helm_d_r4_attention::{
    R4SpinCausalAttentionTransport, R4SpinTransportEvidence, R4SpinTransportIntervention,
    HELM_D_R4_GAUGE_SOFTMAX_POLICY,
};
use uor_r4_core::transformerless::hf_bpe::{HfBpeTokenizer, TokenizerAdapter};
use uor_r4_model_source::attention::CausalAttentionLayerSelection;
use uor_r4_model_source::conformance::FixtureTolerances;
use uor_r4_model_source::{
    CausalAttentionOutputPolicy, CausalAttentionOutputPolicyAudit, ExactBackendReport,
    HuggingFaceLlamaOracle, TeacherExecutionConfig, TeacherExecutionSnapshot, TeacherOracle,
};

use crate::geometric_decoder::{greedy_token, short_cycle_period, RolloutTranscript};
use crate::model::{
    build_source_manifest, SourceManifestFile, SourceSnapshotInfo,
    SOURCE_EXECUTION_MODE_OFFLINE_COMPILER_INPUT,
};
use crate::r4_softmax_reference_generation::{
    expected_causal_audit, expected_projection_audit, expected_r4_audit, model_shape,
    AttentionAuditEvidence, CausalAttentionAuditRecord, GenerationStopReason, ModelShape,
    ProjectionAuditRecord,
};

pub const REPORT_SCHEMA: &str = "uor-r4.r4-softmax-local-generation/1";
pub const POLICY_SCHEMA: &str = "R4SoftmaxLocalGeneratorV1";
pub const DEFAULT_MAX_NEW_TOKENS: usize = 128;
pub const MAX_NEW_TOKENS: usize = 128;
pub const DEFAULT_WORKERS: usize = 4;
pub const QUALIFICATION_REPORT_SCHEMA: &str = "uor-r4.r4-softmax-local-qualification/1";
pub const PYTHON_PREFIX_LOGITS_SCHEMA: &str = "uor-r4.r4-softmax-python-prefix-logits/1";
pub const PREFIX_PARITY_TOKENS: usize = 32;
pub const SEEDED_SAMPLER_POLICY: &str =
    "r4-local-top-k-q32-splitmix64/1;temperature=0.8;top-k=40;rank=logit-desc-token-asc";
pub const GREEDY_SAMPLER_POLICY: &str = "r4-local-greedy-argmax-token-asc/1";

#[derive(Clone, Debug)]
pub struct R4SoftmaxLocalGeneratorConfig {
    pub model: PathBuf,
    pub prompt: String,
    pub max_new_tokens: usize,
    pub workers: NonZeroUsize,
    pub attention_off: bool,
    pub seed: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct R4SoftmaxLocalQualificationConfig {
    pub model: PathBuf,
    pub python_prefix_logits: PathBuf,
    pub reveal_manifest: Option<PathBuf>,
    pub workers: NonZeroUsize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointTreeBinding {
    pub schema: String,
    pub checkpoint_tree_cid: String,
    pub files: Vec<SourceManifestFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalCheckpointBinding {
    pub model_path: String,
    pub checkpoint_tree_cid: String,
    pub config_cid: String,
    pub tokenizer_cid: String,
    pub weights_cid: String,
    pub weights_cid_scope: String,
    pub files: Vec<SourceManifestFile>,
    pub tokenizer: TokenizerAdapter,
    pub bos_token_id: u32,
    pub eos_token_id: u32,
    pub exact_backend: ExactBackendReport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReadAudit {
    pub checkpoint_tree_scans: u64,
    pub checkpoint_tree_file_reads: u64,
    pub tokenizer_loads: u64,
    pub oracle_loads: u64,
    pub local_checkpoint_forward_steps: u64,
    pub provider_calls: u64,
    pub ollama_calls: u64,
    pub prior_trace_reads: u64,
    pub tree_unchanged_across_execution: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalDecodeAudit {
    pub selection: String,
    pub deterministic_greedy: bool,
    pub exact_tie_break: String,
    pub sampler_policy: String,
    pub seed: Option<u64>,
    pub bos_policy: String,
    pub bos_insertions: u64,
    pub utf8_decodable: bool,
    pub short_cycle_period: Option<usize>,
    pub cycles_checked: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttentionOutputPolicyAuditRecord {
    pub policy: String,
    pub applications: u64,
    pub enabled_applications: u64,
    pub zeroed_applications: u64,
    pub output_lanes: u64,
    pub nonzero_lanes_before_policy: u64,
    pub nonzero_lanes_after_policy: u64,
    pub applications_by_layer: Vec<u64>,
    pub maximum_query_position: Option<usize>,
    pub exact: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimingReport {
    pub source_load_seconds: f64,
    pub generation_seconds: f64,
    pub total_seconds: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct R4SoftmaxLocalGenerationReport {
    pub schema: String,
    pub decision_cid: String,
    pub generation_policy_cid: String,
    pub output_cid: String,
    pub audit_cid: String,
    pub claim_scope: String,
    pub checkpoint: LocalCheckpointBinding,
    pub model_shape: ModelShape,
    pub prompt: String,
    pub prompt_token_ids: Vec<u32>,
    pub transcript: RolloutTranscript,
    pub stop_reason: GenerationStopReason,
    pub persistent_state_cid: String,
    pub attention_audit: AttentionAuditEvidence,
    pub attention_output_policy_audit: AttentionOutputPolicyAuditRecord,
    pub decode_audit: LocalDecodeAudit,
    pub source_read_audit: SourceReadAudit,
    pub execution: TeacherExecutionSnapshot,
    pub timing: TimingReport,
    pub nonclaims: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonPrefixLogitsReference {
    pub schema: String,
    pub token_store_cid: String,
    pub weights_cid: String,
    pub prefix_token_ids: Vec<u32>,
    pub maximum_absolute_logit_delta_limit: f64,
    pub enabled: PythonPrefixArmReference,
    pub attention_off: PythonPrefixArmReference,
    pub result_cid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonPrefixArmReference {
    pub top1_token_id: u32,
    pub logits: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualificationProvenance {
    pub export_manifest_cid: String,
    pub export_tree_cid: String,
    pub dataset_manifest_cid: String,
    pub training_view_manifest_cid: String,
    pub split_policy_cid: String,
    pub run_contract_cid: String,
    pub training_result_cid: String,
    pub selected_checkpoint_cid: String,
    pub config_cid: String,
    pub tokenizer_cid: String,
    pub weights_cid: String,
    pub reveal_manifest_cid: Option<String>,
    pub reveal_tree_cid: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualificationInputBinding {
    pub token_store_cid: String,
    pub python_prefix_logits_path: String,
    pub python_prefix_logits_cid: String,
    pub python_prefix_result_cid: String,
    pub prefix_token_ids: Vec<u32>,
    pub sources_unchanged_across_execution: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualificationArmAudit {
    pub sessions: u64,
    pub positions_per_session: usize,
    pub total_positions: u64,
    pub selected_layer_count: usize,
    pub all_layers_selected: bool,
    pub causal_audits_exact: u64,
    pub projection_audits_exact: u64,
    pub r4_audits_exact: u64,
    pub output_policy_audits_exact: u64,
    pub future_reads: u64,
    pub output_policy_applications: u64,
    pub enabled_applications: u64,
    pub zeroed_applications: u64,
    pub output_lanes: u64,
    pub nonzero_lanes_before_policy: u64,
    pub nonzero_lanes_after_policy: u64,
    pub applications_by_layer: Vec<u64>,
    pub state_ledger_cid: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QualificationArmResult {
    pub attention_output_policy: String,
    pub policy_cid: String,
    pub top1_token_id: u32,
    pub output_cid: String,
    pub audit_cid: String,
    pub audit: QualificationArmAudit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrefixParityEvidence {
    pub attention_output_policy: String,
    pub python_top1_token_id: u32,
    pub rust_top1_token_id: u32,
    pub identical_top1: bool,
    pub maximum_absolute_logit_delta: f64,
    pub maximum_absolute_logit_delta_limit: f64,
    pub maximum_absolute_logit_delta_within_limit: bool,
    pub python_logits: Vec<f32>,
    pub rust_logits: Vec<f32>,
    pub passed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct R4SoftmaxLocalQualificationReport {
    pub schema: String,
    pub issue: u32,
    pub decision_cid: String,
    pub checkpoint: LocalCheckpointBinding,
    pub provenance: QualificationProvenance,
    pub model_shape: ModelShape,
    pub evaluation_input: QualificationInputBinding,
    pub enabled: QualificationArmResult,
    pub attention_off: QualificationArmResult,
    pub enabled_prefix_parity: PrefixParityEvidence,
    pub attention_off_prefix_parity: PrefixParityEvidence,
    pub qualification_passed: bool,
    pub source_read_audit: SourceReadAudit,
    pub execution: TeacherExecutionSnapshot,
    pub timing: TimingReport,
    pub nonclaims: Vec<String>,
}

#[derive(Serialize)]
struct DecisionIdentity<'a> {
    schema: &'static str,
    policy: &'static str,
    checkpoint_tree_cid: &'a str,
    config_cid: &'a str,
    tokenizer_cid: &'a str,
    weights_cid: &'a str,
    model_shape: ModelShape,
    prompt: &'a str,
    prompt_token_ids: &'a [u32],
    generated_token_ids: &'a [u32],
    sampler_policy: &'a str,
    seed: Option<u64>,
    stop_reason: &'a GenerationStopReason,
    persistent_state_cid: &'a str,
    attention_audit: &'a AttentionAuditEvidence,
    attention_output_policy_audit: &'a AttentionOutputPolicyAuditRecord,
    source_read_audit: SourceReadAudit,
}

struct QualificationArmExecution {
    result: QualificationArmResult,
    prefix_logits: Vec<f32>,
}

struct VerifiedManifest {
    value: serde_json::Value,
    manifest_cid: String,
    tree_cid: String,
}

struct ExportProvenance {
    export_manifest_cid: String,
    export_tree_cid: String,
    dataset_manifest_cid: String,
    training_view_manifest_cid: String,
    split_policy_cid: String,
    run_contract_cid: String,
    training_result_cid: String,
    selected_checkpoint_cid: String,
    config_cid: String,
    tokenizer_cid: String,
    weights_cid: String,
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }
}

fn sample_top_k_q32(
    logits: &[f32],
    sampler: &mut SplitMix64,
) -> Result<u32, R4SoftmaxLocalGenerationError> {
    if logits.is_empty() || logits.iter().any(|value| !value.is_finite()) {
        return Err(R4SoftmaxLocalGenerationError::Source(
            "seeded sampler requires nonempty finite logits".to_owned(),
        ));
    }
    let mut ranked = logits.iter().copied().enumerate().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    ranked.truncate(40.min(ranked.len()));
    let maximum = f64::from(ranked[0].1);
    let mut weighted = Vec::with_capacity(ranked.len());
    let mut total = 0_u64;
    for (token, logit) in ranked {
        let probability_ratio = ((f64::from(logit) - maximum) / 0.8).exp();
        let weight = (probability_ratio * 4_294_967_296.0)
            .round()
            .clamp(1.0, u64::MAX as f64) as u64;
        total = total.checked_add(weight).ok_or_else(|| {
            R4SoftmaxLocalGenerationError::Source(
                "seeded sampler Q32 weight total overflowed".to_owned(),
            )
        })?;
        weighted.push((token, weight));
    }
    let threshold = ((u128::from(sampler.next_u64()) * u128::from(total)) >> 64) as u64;
    let mut cumulative = 0_u64;
    for (token, weight) in &weighted {
        cumulative = cumulative.checked_add(*weight).ok_or_else(|| {
            R4SoftmaxLocalGenerationError::Source(
                "seeded sampler Q32 cumulative weight overflowed".to_owned(),
            )
        })?;
        if threshold < cumulative {
            return u32::try_from(*token).map_err(|_| {
                R4SoftmaxLocalGenerationError::Source("seeded sampler token exceeds u32".to_owned())
            });
        }
    }
    let token = weighted.last().map(|(token, _)| *token).ok_or_else(|| {
        R4SoftmaxLocalGenerationError::Source("seeded sampler top-k is empty".to_owned())
    })?;
    u32::try_from(token).map_err(|_| {
        R4SoftmaxLocalGenerationError::Source("seeded sampler token exceeds u32".to_owned())
    })
}

#[derive(Debug)]
pub enum R4SoftmaxLocalGenerationError {
    InvalidRequest(String),
    InvalidCheckpoint(String),
    Tokenizer(String),
    Source(String),
    Attention(String),
    Audit(String),
    Io(io::Error),
}

impl fmt::Display for R4SoftmaxLocalGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(reason) => {
                write!(formatter, "invalid local generation request: {reason}")
            }
            Self::InvalidCheckpoint(reason) => {
                write!(formatter, "invalid local checkpoint: {reason}")
            }
            Self::Tokenizer(reason) => write!(formatter, "local tokenizer unavailable: {reason}"),
            Self::Source(reason) => write!(formatter, "local decoder unavailable: {reason}"),
            Self::Attention(reason) => {
                write!(formatter, "R4 causal attention unavailable: {reason}")
            }
            Self::Audit(reason) => write!(formatter, "local generation audit failed: {reason}"),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for R4SoftmaxLocalGenerationError {}

impl From<io::Error> for R4SoftmaxLocalGenerationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn run_r4_softmax_local_generation(
    config: &R4SoftmaxLocalGeneratorConfig,
) -> Result<R4SoftmaxLocalGenerationReport, R4SoftmaxLocalGenerationError> {
    validate_request(config)?;
    let total_started = Instant::now();
    let before = checkpoint_tree_binding(&config.model)?;
    let tokenizer = HfBpeTokenizer::from_dir(&config.model)
        .map_err(|error| R4SoftmaxLocalGenerationError::Tokenizer(error.to_string()))?;
    let tokenizer_cid = tokenizer.address();
    let config_cid = required_file_cid(&before, "config.json")?;
    let bound_tokenizer_cid = required_file_cid(&before, "tokenizer.json")?;
    if tokenizer_cid != bound_tokenizer_cid {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "tokenizer parser identity differs from checkpoint-tree tokenizer bytes".to_owned(),
        ));
    }
    require_weight_files(&before)?;

    let content_prompt_token_ids = tokenizer.encode(&config.prompt);
    if content_prompt_token_ids.is_empty() {
        return Err(R4SoftmaxLocalGenerationError::Tokenizer(
            "--prompt encoded to zero tokens".to_owned(),
        ));
    }
    let sequence_capacity = content_prompt_token_ids
        .len()
        .checked_add(1)
        .and_then(|length| length.checked_add(config.max_new_tokens))
        .ok_or_else(|| {
            R4SoftmaxLocalGenerationError::InvalidRequest(
                "prompt plus generation horizon overflowed".to_owned(),
            )
        })?;

    let load_started = Instant::now();
    let oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
        &config.model,
        sequence_capacity,
        TeacherExecutionConfig::fixed_workers(config.workers),
    )
    .map_err(|error| R4SoftmaxLocalGenerationError::Source(error.to_string()))?;
    let source_load_seconds = load_started.elapsed().as_secs_f64();
    if oracle.cfg().seq_len != sequence_capacity {
        return Err(R4SoftmaxLocalGenerationError::InvalidRequest(format!(
            "requested horizon {sequence_capacity} exceeds checkpoint capacity {}",
            oracle.cfg().seq_len
        )));
    }
    if oracle.cfg().vocab != tokenizer.vocab_size() {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "model vocabulary {} != tokenizer vocabulary {}",
            oracle.cfg().vocab,
            tokenizer.vocab_size()
        )));
    }
    let eos_token_id = u32::try_from(TeacherOracle::eos_token(&oracle)).map_err(|_| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "checkpoint EOS token exceeds the u32 token namespace".to_owned(),
        )
    })?;
    let bos_token_id = u32::try_from(TeacherOracle::bos_token(&oracle)).map_err(|_| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "checkpoint BOS token exceeds the u32 token namespace".to_owned(),
        )
    })?;
    let shape = model_shape(&oracle, sequence_capacity)
        .map_err(|error| R4SoftmaxLocalGenerationError::InvalidCheckpoint(error.to_string()))?;
    let maximum_token = u32::try_from(shape.vocabulary.checked_sub(1).ok_or_else(|| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "checkpoint vocabulary is empty".to_owned(),
        )
    })?)
    .map_err(|_| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "checkpoint vocabulary exceeds the u32 token namespace".to_owned(),
        )
    })?;
    if bos_token_id > maximum_token || eos_token_id > maximum_token {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "checkpoint BOS or EOS token lies outside its vocabulary".to_owned(),
        ));
    }
    let mut prompt_token_ids = Vec::with_capacity(content_prompt_token_ids.len() + 1);
    prompt_token_ids.push(bos_token_id);
    prompt_token_ids.extend_from_slice(&content_prompt_token_ids);
    let transport = R4SpinCausalAttentionTransport::new(
        maximum_token,
        sequence_capacity,
        R4SpinTransportIntervention::Coherent,
    )
    .map_err(|error| R4SoftmaxLocalGenerationError::Attention(error.to_string()))?;
    let attention_output_policy = if config.attention_off {
        CausalAttentionOutputPolicy::ZeroPostWoBeforeResidual
    } else {
        CausalAttentionOutputPolicy::Enabled
    };
    let mut session = oracle
        .new_causal_attention_transport_session_with_output_policy(
            Box::new(transport),
            CausalAttentionLayerSelection::All,
            sequence_capacity,
            attention_output_policy,
        )
        .map_err(|error| R4SoftmaxLocalGenerationError::Attention(error.to_string()))?;
    let all_layers_selected = session.selected_layer_count() == shape.layers
        && (0..shape.layers).all(|layer| session.layer_is_selected(layer));
    if !all_layers_selected {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "not every decoder attention layer was selected".to_owned(),
        ));
    }

    let generation_started = Instant::now();
    let mut logits = vec![0.0_f32; shape.vocabulary];
    for (position, &token) in prompt_token_ids.iter().enumerate() {
        oracle
            .step_causal_attention_transport(&mut session, token as usize, position, &mut logits)
            .map_err(|error| R4SoftmaxLocalGenerationError::Attention(error.to_string()))?;
    }
    let mut positions_executed = prompt_token_ids.len();
    let mut generated_token_ids = Vec::with_capacity(config.max_new_tokens);
    let mut stop_reason = GenerationStopReason::MaximumNewTokens;
    let mut sampler = config.seed.map(SplitMix64::new);
    for decision in 0..config.max_new_tokens {
        let token = if let Some(sampler) = &mut sampler {
            sample_top_k_q32(&logits, sampler)?
        } else {
            greedy_token(&logits)
                .map_err(|error| R4SoftmaxLocalGenerationError::Source(error.to_string()))?
        };
        generated_token_ids.push(token);
        if token == eos_token_id {
            stop_reason = GenerationStopReason::Eos;
            break;
        }
        if let Some(period) = short_cycle_period(&generated_token_ids) {
            stop_reason = GenerationStopReason::ShortCycle { period };
            break;
        }
        if decision + 1 == config.max_new_tokens {
            break;
        }
        let position = prompt_token_ids
            .len()
            .checked_add(decision)
            .ok_or_else(|| {
                R4SoftmaxLocalGenerationError::InvalidRequest(
                    "generated position overflowed".to_owned(),
                )
            })?;
        oracle
            .step_causal_attention_transport(&mut session, token as usize, position, &mut logits)
            .map_err(|error| R4SoftmaxLocalGenerationError::Attention(error.to_string()))?;
        positions_executed += 1;
    }
    let generation_seconds = generation_started.elapsed().as_secs_f64();

    session
        .transport_status()
        .map_err(R4SoftmaxLocalGenerationError::Attention)?;
    if session.policy_identity() != HELM_D_R4_GAUGE_SOFTMAX_POLICY {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "unexpected R4 attention policy identity".to_owned(),
        ));
    }
    let implementation: R4SpinTransportEvidence = serde_json::from_str(
        &session
            .transport_implementation_evidence()
            .map_err(R4SoftmaxLocalGenerationError::Attention)?
            .ok_or_else(|| {
                R4SoftmaxLocalGenerationError::Audit(
                    "R4 transport emitted no implementation evidence".to_owned(),
                )
            })?,
    )
    .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    let observed_causal: CausalAttentionAuditRecord = session.audit().into();
    let observed_projection: ProjectionAuditRecord = session.pre_rope_projection_audit().into();
    let expected_causal = expected_causal_audit(positions_executed, &shape)
        .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    let expected_projection = expected_projection_audit(positions_executed, &shape)
        .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    let expected_r4 = expected_r4_audit(positions_executed, &shape)
        .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    let causal_audit_exact = observed_causal == expected_causal;
    let projection_audit_exact = observed_projection == expected_projection;
    let r4_audit_exact = implementation.policy_identity == HELM_D_R4_GAUGE_SOFTMAX_POLICY
        && implementation.intervention == R4SpinTransportIntervention::Coherent
        && implementation.frame_table_offsets.len() == positions_executed
        && implementation.audit == expected_r4;
    let zero_future_reads =
        observed_causal.future_reads == 0 && implementation.audit.future_position_reads == 0;
    if !(causal_audit_exact && projection_audit_exact && r4_audit_exact && zero_future_reads) {
        return Err(R4SoftmaxLocalGenerationError::Audit(format!(
            "audit mismatch: causal={causal_audit_exact}, projection={projection_audit_exact}, R4={r4_audit_exact}, zero_future_reads={zero_future_reads}"
        )));
    }
    let attention_audit = AttentionAuditEvidence {
        selected_layer_count: session.selected_layer_count(),
        positions_executed,
        observed_causal,
        expected_causal,
        causal_audit_exact,
        observed_projection,
        expected_projection,
        projection_audit_exact,
        r4_implementation: implementation,
        expected_r4,
        r4_audit_exact,
        zero_future_reads,
        all_layers_selected,
    };
    let attention_output_policy_audit = output_policy_audit_record(
        session.output_policy_audit(),
        positions_executed,
        &shape,
        attention_output_policy,
    )?;
    let persistent_state_cid = session.persistent_state_cid();
    let execution = oracle.execution_snapshot();
    let after = checkpoint_tree_binding(&config.model)?;
    if before != after {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "checkpoint tree changed during local generation".to_owned(),
        ));
    }
    let file_reads = u64::try_from(before.files.len())
        .ok()
        .and_then(|count| count.checked_mul(2))
        .ok_or_else(|| {
            R4SoftmaxLocalGenerationError::Audit("source-read count overflowed".to_owned())
        })?;
    let source_read_audit = SourceReadAudit {
        checkpoint_tree_scans: 2,
        checkpoint_tree_file_reads: file_reads,
        tokenizer_loads: 1,
        oracle_loads: 1,
        local_checkpoint_forward_steps: u64::try_from(positions_executed).map_err(|_| {
            R4SoftmaxLocalGenerationError::Audit("forward-step count overflowed".to_owned())
        })?,
        provider_calls: 0,
        ollama_calls: 0,
        prior_trace_reads: 0,
        tree_unchanged_across_execution: true,
    };

    let transcript = local_transcript(
        &tokenizer,
        &config.prompt,
        prompt_token_ids.len(),
        eos_token_id,
        generated_token_ids,
    );
    let sampler_policy = if config.seed.is_some() {
        SEEDED_SAMPLER_POLICY
    } else {
        GREEDY_SAMPLER_POLICY
    };
    let decode_audit = LocalDecodeAudit {
        selection: if config.seed.is_some() {
            "deterministic seeded temperature/top-k sampling over local checkpoint logits"
                .to_owned()
        } else {
            "greedy argmax over local checkpoint logits".to_owned()
        },
        deterministic_greedy: config.seed.is_none(),
        exact_tie_break: "lower token id wins an exact logit tie".to_owned(),
        sampler_policy: sampler_policy.to_owned(),
        seed: config.seed,
        bos_policy: "prepend the checkpoint BOS token exactly once before raw prompt tokens"
            .to_owned(),
        bos_insertions: 1,
        utf8_decodable: transcript.utf8_decodable,
        short_cycle_period: transcript.short_cycle_period,
        cycles_checked: vec![1, 2, 3, 4],
    };
    let checkpoint = LocalCheckpointBinding {
        model_path: config.model.display().to_string(),
        checkpoint_tree_cid: before.checkpoint_tree_cid.clone(),
        config_cid,
        tokenizer_cid,
        weights_cid: oracle.source_cid().to_owned(),
        weights_cid_scope: "Safetensors shard bytes in the loader's canonical shard order; not the checkpoint-tree CID".to_owned(),
        files: before.files,
        tokenizer: tokenizer.adapter(),
        bos_token_id,
        eos_token_id,
        exact_backend: oracle.exact_backend_report(),
    };
    let decision_cid = decision_cid(&DecisionIdentity {
        schema: REPORT_SCHEMA,
        policy: POLICY_SCHEMA,
        checkpoint_tree_cid: &checkpoint.checkpoint_tree_cid,
        config_cid: &checkpoint.config_cid,
        tokenizer_cid: &checkpoint.tokenizer_cid,
        weights_cid: &checkpoint.weights_cid,
        model_shape: shape,
        prompt: &config.prompt,
        prompt_token_ids: &prompt_token_ids,
        generated_token_ids: &transcript.generated_token_ids,
        sampler_policy,
        seed: config.seed,
        stop_reason: &stop_reason,
        persistent_state_cid: &persistent_state_cid,
        attention_audit: &attention_audit,
        attention_output_policy_audit: &attention_output_policy_audit,
        source_read_audit,
    })?;
    let generation_policy_cid = cid_serializable(&(
        POLICY_SCHEMA,
        HELM_D_R4_GAUGE_SOFTMAX_POLICY,
        attention_output_policy.identity(),
        sampler_policy,
        config.seed,
        config.max_new_tokens,
    ))?;
    let output_cid = cid_serializable(&(
        &transcript.generated_token_ids,
        &transcript.raw_decoded,
        &transcript.response_text,
        &stop_reason,
    ))?;
    let audit_cid = cid_serializable(&(
        &attention_audit,
        &attention_output_policy_audit,
        &decode_audit,
        source_read_audit,
    ))?;
    Ok(R4SoftmaxLocalGenerationReport {
        schema: REPORT_SCHEMA.to_owned(),
        decision_cid,
        generation_policy_cid,
        output_cid,
        audit_cid,
        claim_scope: "local checkpoint generation through every learned layer's coherent R4/Spin transported ordinary causal dot-product/stable-softmax attention".to_owned(),
        checkpoint,
        model_shape: shape,
        prompt: config.prompt.clone(),
        prompt_token_ids,
        transcript,
        stop_reason,
        persistent_state_cid,
        attention_audit,
        attention_output_policy_audit,
        decode_audit,
        source_read_audit,
        execution,
        timing: TimingReport {
            source_load_seconds,
            generation_seconds,
            total_seconds: total_started.elapsed().as_secs_f64(),
        },
        nonclaims: vec![
            "This floating-point local checkpoint path is not transformerless or multiplication-free.".to_owned(),
            "Coherent R4/Spin execution does not establish a geometry advantage over ordinary coordinates.".to_owned(),
            "One rollout does not establish general text quality, reasoning, scaling, WASM, or release readiness.".to_owned(),
        ],
    })
}

pub fn run_r4_softmax_local_qualification(
    config: &R4SoftmaxLocalQualificationConfig,
) -> Result<R4SoftmaxLocalQualificationReport, R4SoftmaxLocalGenerationError> {
    let total_started = Instant::now();
    let before = checkpoint_tree_binding(&config.model)?;
    require_weight_files(&before)?;
    let export = verify_export_provenance(&config.model)?;

    let reveal = if let Some(path) = &config.reveal_manifest {
        let manifest = verify_manifest_envelope(path)?;
        if require_json_string(&manifest.value, "schema", "reveal manifest")?
            != "uor-r4-softmax-trainer-reveal-manifest/1"
        {
            return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
                "unexpected reveal manifest schema".to_owned(),
            ));
        }
        let research_root = path.parent().and_then(Path::parent).ok_or_else(|| {
            R4SoftmaxLocalGenerationError::InvalidCheckpoint(
                "reveal manifest must be <research-root>/reveal/reveal-manifest.json".to_owned(),
            )
        })?;
        require_same_file_path(&config.model, &research_root.join("export"), "model export")?;
        require_same_file_path(
            &config.python_prefix_logits,
            &research_root.join("reveal/python-prefix-logits.json"),
            "Python prefix reference",
        )?;
        verify_manifest_artifacts(&manifest.value, research_root)?;
        Some(manifest)
    } else {
        None
    };

    let python_reference_bytes =
        read_regular_file(&config.python_prefix_logits, "Python prefix reference")?;
    let python_prefix_logits_cid = raw_cid(&python_reference_bytes);
    if let Some(manifest) = &reveal {
        verify_manifest_artifact(
            &manifest.value,
            "reveal/python-prefix-logits.json",
            &python_reference_bytes,
        )?;
    }
    let python_reference_value: serde_json::Value = serde_json::from_slice(&python_reference_bytes)
        .map_err(|error| {
            R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
                "invalid Python prefix reference: {error}"
            ))
        })?;
    verify_embedded_cid_bytes(
        &python_reference_bytes,
        "result_cid",
        require_json_string(
            &python_reference_value,
            "result_cid",
            "Python prefix reference",
        )?
        .as_str(),
        "Python prefix reference",
    )?;
    let python_reference: PythonPrefixLogitsReference =
        serde_json::from_value(python_reference_value).map_err(|error| {
            R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
                "invalid Python prefix reference: {error}"
            ))
        })?;
    validate_python_prefix_reference_envelope(&python_reference)?;

    let tokenizer = HfBpeTokenizer::from_dir(&config.model)
        .map_err(|error| R4SoftmaxLocalGenerationError::Tokenizer(error.to_string()))?;
    let tokenizer_cid = tokenizer.address();
    let config_cid = required_file_cid(&before, "config.json")?;
    if tokenizer_cid != required_file_cid(&before, "tokenizer.json")? {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "tokenizer parser identity differs from checkpoint-tree tokenizer bytes".to_owned(),
        ));
    }
    let load_started = Instant::now();
    let oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
        &config.model,
        PREFIX_PARITY_TOKENS,
        TeacherExecutionConfig::fixed_workers(config.workers),
    )
    .map_err(|error| R4SoftmaxLocalGenerationError::Source(error.to_string()))?;
    let source_load_seconds = load_started.elapsed().as_secs_f64();
    if oracle.cfg().seq_len != PREFIX_PARITY_TOKENS {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "#1014 prefix qualifier could not allocate exactly 32 positions".to_owned(),
        ));
    }
    let shape = model_shape(&oracle, PREFIX_PARITY_TOKENS)
        .map_err(|error| R4SoftmaxLocalGenerationError::InvalidCheckpoint(error.to_string()))?;
    enforce_issue_1014_shape(&config.model, &shape, tokenizer.vocab_size())?;
    let weights_cid = oracle.source_cid().to_owned();
    if python_reference.weights_cid != weights_cid {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "Python prefix reference does not bind the loaded Safetensors weights".to_owned(),
        ));
    }
    if export.weights_cid != weights_cid
        || export.tokenizer_cid != tokenizer_cid
        || export.config_cid != config_cid
    {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "verified export manifest component identities do not match the loaded checkpoint"
                .to_owned(),
        ));
    }
    if let Some(manifest) = &reveal {
        for (field, expected) in [
            ("weights_cid", weights_cid.as_str()),
            ("tokenizer_cid", tokenizer_cid.as_str()),
            ("dataset_manifest_cid", export.dataset_manifest_cid.as_str()),
            (
                "training_view_manifest_cid",
                export.training_view_manifest_cid.as_str(),
            ),
            ("split_policy_cid", export.split_policy_cid.as_str()),
        ] {
            require_reveal_identity(&manifest.value, field, expected)?;
        }
    }

    let maximum_token = u32::try_from(shape.vocabulary - 1).map_err(|_| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "checkpoint vocabulary exceeds the u32 token namespace".to_owned(),
        )
    })?;
    for (index, &token) in python_reference.prefix_token_ids.iter().enumerate() {
        if token > maximum_token {
            return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
                "prefix token {token} at index {index} exceeds vocabulary maximum {maximum_token}"
            )));
        }
    }

    let qualification_started = Instant::now();
    let enabled_execution = run_qualification_arm(
        &oracle,
        &python_reference.prefix_token_ids,
        &shape,
        maximum_token,
        CausalAttentionOutputPolicy::Enabled,
    )?;
    let attention_off_execution = run_qualification_arm(
        &oracle,
        &python_reference.prefix_token_ids,
        &shape,
        maximum_token,
        CausalAttentionOutputPolicy::ZeroPostWoBeforeResidual,
    )?;
    let generation_seconds = qualification_started.elapsed().as_secs_f64();
    let enabled_prefix_parity = prefix_parity_evidence(
        CausalAttentionOutputPolicy::Enabled,
        python_reference.enabled,
        enabled_execution.prefix_logits,
    )?;
    let attention_off_prefix_parity = prefix_parity_evidence(
        CausalAttentionOutputPolicy::ZeroPostWoBeforeResidual,
        python_reference.attention_off,
        attention_off_execution.prefix_logits,
    )?;
    let qualification_passed = enabled_prefix_parity.passed && attention_off_prefix_parity.passed;

    let after = checkpoint_tree_binding(&config.model)?;
    if before != after {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "checkpoint tree changed during local qualification".to_owned(),
        ));
    }
    let python_reference_after = read_regular_file(
        &config.python_prefix_logits,
        "Python prefix reference replay",
    )?;
    if raw_cid(&python_reference_after) != python_prefix_logits_cid {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "Python prefix reference changed during execution".to_owned(),
        ));
    }
    let checkpoint_file_reads = u64::try_from(before.files.len())
        .ok()
        .and_then(|count| count.checked_mul(2))
        .ok_or_else(|| {
            R4SoftmaxLocalGenerationError::Audit("source-read count overflowed".to_owned())
        })?;
    let source_read_audit = SourceReadAudit {
        checkpoint_tree_scans: 2,
        checkpoint_tree_file_reads: checkpoint_file_reads,
        tokenizer_loads: 1,
        oracle_loads: 1,
        local_checkpoint_forward_steps: (PREFIX_PARITY_TOKENS as u64) * 2,
        provider_calls: 0,
        ollama_calls: 0,
        prior_trace_reads: 0,
        tree_unchanged_across_execution: true,
    };
    let bos_token_id = u32::try_from(TeacherOracle::bos_token(&oracle)).map_err(|_| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "checkpoint BOS token exceeds the u32 namespace".to_owned(),
        )
    })?;
    if python_reference.prefix_token_ids.first().copied() != Some(bos_token_id) {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "32-token qualification prefix does not begin with checkpoint BOS".to_owned(),
        ));
    }
    let eos_token_id = u32::try_from(TeacherOracle::eos_token(&oracle)).map_err(|_| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "checkpoint EOS token exceeds the u32 namespace".to_owned(),
        )
    })?;
    let checkpoint = LocalCheckpointBinding {
        model_path: config.model.display().to_string(),
        checkpoint_tree_cid: before.checkpoint_tree_cid,
        config_cid,
        tokenizer_cid,
        weights_cid,
        weights_cid_scope: "Safetensors shard bytes in the loader's canonical shard order; not the checkpoint-tree CID".to_owned(),
        files: before.files,
        tokenizer: tokenizer.adapter(),
        bos_token_id,
        eos_token_id,
        exact_backend: oracle.exact_backend_report(),
    };
    let provenance = QualificationProvenance {
        export_manifest_cid: export.export_manifest_cid,
        export_tree_cid: export.export_tree_cid,
        dataset_manifest_cid: export.dataset_manifest_cid,
        training_view_manifest_cid: export.training_view_manifest_cid,
        split_policy_cid: export.split_policy_cid,
        run_contract_cid: export.run_contract_cid,
        training_result_cid: export.training_result_cid,
        selected_checkpoint_cid: export.selected_checkpoint_cid,
        config_cid: export.config_cid,
        tokenizer_cid: export.tokenizer_cid,
        weights_cid: export.weights_cid,
        reveal_manifest_cid: reveal
            .as_ref()
            .map(|manifest| manifest.manifest_cid.clone()),
        reveal_tree_cid: reveal.as_ref().map(|manifest| manifest.tree_cid.clone()),
    };
    let evaluation_input = QualificationInputBinding {
        token_store_cid: python_reference.token_store_cid,
        python_prefix_logits_path: config.python_prefix_logits.display().to_string(),
        python_prefix_logits_cid,
        python_prefix_result_cid: python_reference.result_cid,
        prefix_token_ids: python_reference.prefix_token_ids,
        sources_unchanged_across_execution: true,
    };
    let enabled = enabled_execution.result;
    let attention_off = attention_off_execution.result;
    let decision_cid = qualification_decision_cid(
        &checkpoint,
        &provenance,
        &evaluation_input,
        &enabled,
        &attention_off,
        &enabled_prefix_parity,
        &attention_off_prefix_parity,
    )?;
    Ok(R4SoftmaxLocalQualificationReport {
        schema: QUALIFICATION_REPORT_SCHEMA.to_owned(),
        issue: 1014,
        decision_cid,
        checkpoint,
        provenance,
        model_shape: shape,
        evaluation_input,
        enabled,
        attention_off,
        enabled_prefix_parity,
        attention_off_prefix_parity,
        qualification_passed,
        source_read_audit,
        execution: oracle.execution_snapshot(),
        timing: TimingReport {
            source_load_seconds,
            generation_seconds,
            total_seconds: total_started.elapsed().as_secs_f64(),
        },
        nonclaims: vec![
            "This 32-token exporter/loader gate does not substitute for the Python MPS full sealed-test NLL comparison.".to_owned(),
            "Passing parity establishes faithful Rust loading and R4 execution of the trained checkpoint, not geometric advantage.".to_owned(),
            "This floating-point checkpoint path is not the multiplication-free deployed runtime.".to_owned(),
        ],
    })
}
pub fn write_json_report(
    path: &Path,
    report: &R4SoftmaxLocalGenerationReport,
) -> Result<(), R4SoftmaxLocalGenerationError> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| R4SoftmaxLocalGenerationError::Io(io::Error::other(error)))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn write_qualification_json_report(
    path: &Path,
    report: &R4SoftmaxLocalQualificationReport,
) -> Result<(), R4SoftmaxLocalGenerationError> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| R4SoftmaxLocalGenerationError::Io(io::Error::other(error)))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

fn validate_request(
    config: &R4SoftmaxLocalGeneratorConfig,
) -> Result<(), R4SoftmaxLocalGenerationError> {
    if config.prompt.is_empty() {
        return Err(R4SoftmaxLocalGenerationError::InvalidRequest(
            "--prompt must not be empty".to_owned(),
        ));
    }
    if config.max_new_tokens == 0 || config.max_new_tokens > MAX_NEW_TOKENS {
        return Err(R4SoftmaxLocalGenerationError::InvalidRequest(format!(
            "--max-new-tokens must be in 1..={MAX_NEW_TOKENS}"
        )));
    }
    Ok(())
}

fn checkpoint_tree_binding(
    model: &Path,
) -> Result<CheckpointTreeBinding, R4SoftmaxLocalGenerationError> {
    if !model.is_dir() {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "{} is not a checkpoint directory",
            model.display()
        )));
    }
    let manifest = build_source_manifest(
        model,
        &SourceSnapshotInfo {
            repository: "local://uor-r4/issue-1014".to_owned(),
            revision: "content-addressed-local-checkpoint".to_owned(),
            license: None,
            source_execution_mode: SOURCE_EXECUTION_MODE_OFFLINE_COMPILER_INPUT.to_owned(),
        },
    )
    .map_err(|error| R4SoftmaxLocalGenerationError::InvalidCheckpoint(error.to_string()))?;
    if manifest.files.is_empty() {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "checkpoint tree has no admitted files".to_owned(),
        ));
    }
    #[derive(Serialize)]
    struct TreeIdentity<'a> {
        schema: &'static str,
        files: &'a [SourceManifestFile],
    }
    let identity = TreeIdentity {
        schema: "uor-r4.r4-softmax-local-checkpoint-tree/1",
        files: &manifest.files,
    };
    let bytes = serde_json::to_vec(&identity)
        .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    Ok(CheckpointTreeBinding {
        schema: identity.schema.to_owned(),
        checkpoint_tree_cid: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
        files: manifest.files,
    })
}

fn required_file_cid(
    tree: &CheckpointTreeBinding,
    path: &str,
) -> Result<String, R4SoftmaxLocalGenerationError> {
    tree.files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.kappa.clone())
        .ok_or_else(|| {
            R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
                "checkpoint tree has no {path}"
            ))
        })
}

fn require_weight_files(tree: &CheckpointTreeBinding) -> Result<(), R4SoftmaxLocalGenerationError> {
    let has_single = tree
        .files
        .iter()
        .any(|file| file.path == "model.safetensors");
    let has_index = tree
        .files
        .iter()
        .any(|file| file.path == "model.safetensors.index.json");
    if has_single || has_index {
        Ok(())
    } else {
        Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "checkpoint tree has neither model.safetensors nor model.safetensors.index.json"
                .to_owned(),
        ))
    }
}

fn raw_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn canonical_json_bytes(
    value: &serde_json::Value,
) -> Result<Vec<u8>, R4SoftmaxLocalGenerationError> {
    let mut bytes = serde_json::to_vec(value)
        .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn cid_serializable<T: Serialize>(value: &T) -> Result<String, R4SoftmaxLocalGenerationError> {
    let value = serde_json::to_value(value)
        .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    Ok(raw_cid(&canonical_json_bytes(&value)?))
}

fn read_regular_file(path: &Path, label: &str) -> Result<Vec<u8>, R4SoftmaxLocalGenerationError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "cannot inspect {label} {}: {error}",
            path.display()
        ))
    })?;
    if !metadata.file_type().is_file() {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "{label} {} is not a regular non-symlink file",
            path.display()
        )));
    }
    std::fs::read(path).map_err(R4SoftmaxLocalGenerationError::Io)
}

fn require_json_string(
    value: &serde_json::Value,
    field: &str,
    label: &str,
) -> Result<String, R4SoftmaxLocalGenerationError> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
                "{label} has no nonempty {field}"
            ))
        })
}

fn verify_manifest_envelope(
    path: &Path,
) -> Result<VerifiedManifest, R4SoftmaxLocalGenerationError> {
    let bytes = read_regular_file(path, "bound manifest")?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "invalid manifest {}: {error}",
            path.display()
        ))
    })?;
    let manifest_cid = require_json_string(&value, "manifest_cid", "bound manifest")?;
    verify_embedded_cid_bytes(&bytes, "manifest_cid", &manifest_cid, "bound manifest")?;
    let artifacts = value
        .get("artifacts")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            R4SoftmaxLocalGenerationError::InvalidCheckpoint(
                "bound manifest has no artifact array".to_owned(),
            )
        })?;
    let mut sorted = artifacts.clone();
    sorted.sort_by(|left, right| {
        left.get("path")
            .and_then(serde_json::Value::as_str)
            .cmp(&right.get("path").and_then(serde_json::Value::as_str))
    });
    if sorted != *artifacts {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "bound manifest artifacts are not in canonical path order".to_owned(),
        ));
    }
    let tree_cid = require_json_string(&value, "tree_cid", "bound manifest")?;
    if raw_cid(&canonical_json_bytes(&serde_json::Value::Array(sorted))?) != tree_cid {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "bound manifest tree CID does not reproduce".to_owned(),
        ));
    }
    Ok(VerifiedManifest {
        value,
        manifest_cid,
        tree_cid,
    })
}

fn manifest_artifact_record<'a>(
    manifest: &'a serde_json::Value,
    relative_path: &str,
) -> Result<&'a serde_json::Value, R4SoftmaxLocalGenerationError> {
    manifest
        .get("artifacts")
        .and_then(serde_json::Value::as_array)
        .and_then(|records| {
            records.iter().find(|record| {
                record.get("path").and_then(serde_json::Value::as_str) == Some(relative_path)
            })
        })
        .ok_or_else(|| {
            R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
                "manifest does not commit {relative_path}"
            ))
        })
}

fn verify_manifest_artifact(
    manifest: &serde_json::Value,
    relative_path: &str,
    bytes: &[u8],
) -> Result<(), R4SoftmaxLocalGenerationError> {
    let record = manifest_artifact_record(manifest, relative_path)?;
    let expected_bytes = record
        .get("bytes")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
                "manifest artifact {relative_path} has no byte length"
            ))
        })?;
    let actual_bytes = u64::try_from(bytes.len()).map_err(|_| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "artifact byte length exceeds u64".to_owned(),
        )
    })?;
    let expected_cid = require_json_string(record, "cid", "manifest artifact")?;
    if expected_bytes != actual_bytes || expected_cid != raw_cid(bytes) {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "manifest artifact {relative_path} does not reproduce"
        )));
    }
    Ok(())
}

fn verify_manifest_artifacts(
    manifest: &serde_json::Value,
    root: &Path,
) -> Result<(), R4SoftmaxLocalGenerationError> {
    let records = manifest
        .get("artifacts")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            R4SoftmaxLocalGenerationError::InvalidCheckpoint(
                "bound manifest has no artifact array".to_owned(),
            )
        })?;
    for record in records {
        let relative = require_json_string(record, "path", "manifest artifact")?;
        let path = Path::new(&relative);
        if path.is_absolute()
            || path
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
                "manifest artifact path {relative:?} is not a safe relative path"
            )));
        }
        let bytes = read_regular_file(&root.join(path), "manifest artifact")?;
        verify_manifest_artifact(manifest, &relative, &bytes)?;
    }
    Ok(())
}

fn require_same_file_path(
    observed: &Path,
    expected: &Path,
    label: &str,
) -> Result<(), R4SoftmaxLocalGenerationError> {
    let observed = observed
        .canonicalize()
        .map_err(R4SoftmaxLocalGenerationError::Io)?;
    let expected = expected
        .canonicalize()
        .map_err(R4SoftmaxLocalGenerationError::Io)?;
    if observed != expected {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "{label} path {} != committed path {}",
            observed.display(),
            expected.display()
        )));
    }
    Ok(())
}

fn require_reveal_identity(
    reveal: &serde_json::Value,
    field: &str,
    expected: &str,
) -> Result<(), R4SoftmaxLocalGenerationError> {
    let observed = require_json_string(reveal, field, "reveal manifest")?;
    if observed != expected {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "reveal manifest {field} does not match the loaded export"
        )));
    }
    Ok(())
}

fn verify_export_provenance(
    model: &Path,
) -> Result<ExportProvenance, R4SoftmaxLocalGenerationError> {
    let manifest = verify_manifest_envelope(&model.join("export-manifest.json"))?;
    if require_json_string(&manifest.value, "schema", "export manifest")?
        != "uor-r4-softmax-trainer-export/1"
    {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "unexpected #1014 export manifest schema".to_owned(),
        ));
    }
    verify_manifest_artifacts(&manifest.value, model)?;
    for path in ["config.json", "model.safetensors", "tokenizer.json"] {
        manifest_artifact_record(&manifest.value, path)?;
    }
    Ok(ExportProvenance {
        export_manifest_cid: manifest.manifest_cid,
        export_tree_cid: manifest.tree_cid,
        dataset_manifest_cid: require_json_string(
            &manifest.value,
            "dataset_manifest_cid",
            "export manifest",
        )?,
        training_view_manifest_cid: require_json_string(
            &manifest.value,
            "training_view_manifest_cid",
            "export manifest",
        )?,
        split_policy_cid: require_json_string(
            &manifest.value,
            "split_policy_cid",
            "export manifest",
        )?,
        run_contract_cid: require_json_string(
            &manifest.value,
            "run_contract_cid",
            "export manifest",
        )?,
        training_result_cid: require_json_string(
            &manifest.value,
            "training_result_cid",
            "export manifest",
        )?,
        selected_checkpoint_cid: require_json_string(
            &manifest.value,
            "selected_checkpoint_cid",
            "export manifest",
        )?,
        config_cid: require_json_string(&manifest.value, "config_cid", "export manifest")?,
        tokenizer_cid: require_json_string(&manifest.value, "tokenizer_cid", "export manifest")?,
        weights_cid: require_json_string(&manifest.value, "weights_cid", "export manifest")?,
    })
}

fn validate_python_prefix_reference_envelope(
    reference: &PythonPrefixLogitsReference,
) -> Result<(), R4SoftmaxLocalGenerationError> {
    if reference.schema != PYTHON_PREFIX_LOGITS_SCHEMA {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "unexpected Python prefix schema {}",
            reference.schema
        )));
    }
    if reference.prefix_token_ids.len() != PREFIX_PARITY_TOKENS {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "Python prefix has {} tokens, expected {PREFIX_PARITY_TOKENS}",
            reference.prefix_token_ids.len()
        )));
    }
    let tolerance = FixtureTolerances::default().logit_abs;
    if reference.maximum_absolute_logit_delta_limit != tolerance {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "Python prefix tolerance {} != frozen {tolerance}",
            reference.maximum_absolute_logit_delta_limit
        )));
    }
    for (label, arm) in [
        ("enabled", &reference.enabled),
        ("attention_off", &reference.attention_off),
    ] {
        if arm.logits.len() != 4096 || arm.logits.iter().any(|value| !value.is_finite()) {
            return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
                "Python {label} logits must be 4096 finite values"
            )));
        }
        let top1 = greedy_token(&arm.logits)
            .map_err(|error| R4SoftmaxLocalGenerationError::InvalidCheckpoint(error.to_string()))?;
        if top1 != arm.top1_token_id {
            return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
                "Python {label} top-1 does not reproduce its logits"
            )));
        }
    }
    Ok(())
}

fn verify_embedded_cid_bytes(
    bytes: &[u8],
    field: &str,
    expected: &str,
    label: &str,
) -> Result<(), R4SoftmaxLocalGenerationError> {
    let text = std::str::from_utf8(bytes).map_err(|error| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "{label} is not UTF-8 JSON: {error}"
        ))
    })?;
    let needle = format!("\"{field}\":\"{expected}\"");
    let start = text.find(&needle).ok_or_else(|| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "{label} does not contain its canonical {field}"
        ))
    })?;
    if text[start + needle.len()..].contains(&needle) {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "{label} contains duplicate {field} values"
        )));
    }
    let mut unsigned = bytes.to_vec();
    let end = start + needle.len();
    if start > 0 && unsigned[start - 1] == b',' {
        unsigned.drain(start - 1..end);
    } else if unsigned.get(end) == Some(&b',') {
        unsigned.drain(start..=end);
    } else {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "{label} {field} is not one field of a canonical JSON object"
        )));
    }
    if raw_cid(&unsigned) != expected {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "{label} {field} does not reproduce"
        )));
    }
    Ok(())
}

fn enforce_issue_1014_shape(
    model: &Path,
    shape: &ModelShape,
    tokenizer_vocabulary: usize,
) -> Result<(), R4SoftmaxLocalGenerationError> {
    let expected = ModelShape {
        dimension: 288,
        hidden_dimension: 768,
        layers: 6,
        query_heads: 6,
        key_value_heads: 6,
        head_size: 48,
        vocabulary: 4096,
        sequence_capacity: PREFIX_PARITY_TOKENS,
    };
    if *shape != expected || tokenizer_vocabulary != expected.vocabulary {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "checkpoint shape {shape:?} is not the frozen #1014 shape {expected:?}"
        )));
    }
    let config_bytes = read_regular_file(&model.join("config.json"), "checkpoint config")?;
    let config: serde_json::Value = serde_json::from_slice(&config_bytes).map_err(|error| {
        R4SoftmaxLocalGenerationError::InvalidCheckpoint(format!(
            "invalid checkpoint config: {error}"
        ))
    })?;
    let exact = config
        .get("max_position_embeddings")
        .and_then(serde_json::Value::as_u64)
        == Some(256)
        && config
            .get("tie_word_embeddings")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && config
            .get("bos_token_id")
            .and_then(serde_json::Value::as_u64)
            == Some(0)
        && config
            .get("eos_token_id")
            .and_then(serde_json::Value::as_u64)
            == Some(1);
    if !exact {
        return Err(R4SoftmaxLocalGenerationError::InvalidCheckpoint(
            "checkpoint config violates frozen #1014 context/tied-head/BOS/EOS contract".to_owned(),
        ));
    }
    Ok(())
}

fn output_policy_audit_record(
    audit: CausalAttentionOutputPolicyAudit,
    positions: usize,
    shape: &ModelShape,
    expected_policy: CausalAttentionOutputPolicy,
) -> Result<AttentionOutputPolicyAuditRecord, R4SoftmaxLocalGenerationError> {
    let positions_u64 = u64::try_from(positions).map_err(|_| {
        R4SoftmaxLocalGenerationError::Audit("position count overflowed".to_owned())
    })?;
    let layers_u64 = u64::try_from(shape.layers)
        .map_err(|_| R4SoftmaxLocalGenerationError::Audit("layer count overflowed".to_owned()))?;
    let dimension_u64 = u64::try_from(shape.dimension).map_err(|_| {
        R4SoftmaxLocalGenerationError::Audit("model dimension overflowed".to_owned())
    })?;
    let applications = positions_u64.checked_mul(layers_u64).ok_or_else(|| {
        R4SoftmaxLocalGenerationError::Audit("attention-output census overflowed".to_owned())
    })?;
    let output_lanes = applications.checked_mul(dimension_u64).ok_or_else(|| {
        R4SoftmaxLocalGenerationError::Audit("attention-output lane census overflowed".to_owned())
    })?;
    let policy_counts_exact = match expected_policy {
        CausalAttentionOutputPolicy::Enabled => {
            audit.enabled_applications == applications
                && audit.zeroed_applications == 0
                && audit.nonzero_lanes_after_policy == audit.nonzero_lanes_before_policy
        }
        CausalAttentionOutputPolicy::ZeroPostWoBeforeResidual => {
            audit.enabled_applications == 0
                && audit.zeroed_applications == applications
                && audit.nonzero_lanes_after_policy == 0
        }
    };
    let exact = audit.policy == expected_policy
        && audit.applications == applications
        && audit.output_lanes == output_lanes
        && audit.applications_by_layer == vec![positions_u64; shape.layers]
        && audit.maximum_query_position == positions.checked_sub(1)
        && policy_counts_exact;
    let record = AttentionOutputPolicyAuditRecord {
        policy: audit.policy.identity().to_owned(),
        applications: audit.applications,
        enabled_applications: audit.enabled_applications,
        zeroed_applications: audit.zeroed_applications,
        output_lanes: audit.output_lanes,
        nonzero_lanes_before_policy: audit.nonzero_lanes_before_policy,
        nonzero_lanes_after_policy: audit.nonzero_lanes_after_policy,
        applications_by_layer: audit.applications_by_layer,
        maximum_query_position: audit.maximum_query_position,
        exact,
    };
    if !record.exact {
        return Err(R4SoftmaxLocalGenerationError::Audit(format!(
            "post-Wo attention-output policy census is not exact for {}",
            expected_policy.identity()
        )));
    }
    Ok(record)
}

fn run_qualification_arm(
    oracle: &HuggingFaceLlamaOracle,
    prefix_token_ids: &[u32],
    shape: &ModelShape,
    maximum_token: u32,
    output_policy: CausalAttentionOutputPolicy,
) -> Result<QualificationArmExecution, R4SoftmaxLocalGenerationError> {
    if prefix_token_ids.len() != PREFIX_PARITY_TOKENS {
        return Err(R4SoftmaxLocalGenerationError::InvalidRequest(
            "qualification arm requires exactly 32 prefix tokens".to_owned(),
        ));
    }
    let transport = R4SpinCausalAttentionTransport::new(
        maximum_token,
        PREFIX_PARITY_TOKENS,
        R4SpinTransportIntervention::Coherent,
    )
    .map_err(|error| R4SoftmaxLocalGenerationError::Attention(error.to_string()))?;
    let mut session = oracle
        .new_causal_attention_transport_session_with_output_policy(
            Box::new(transport),
            CausalAttentionLayerSelection::All,
            PREFIX_PARITY_TOKENS,
            output_policy,
        )
        .map_err(|error| R4SoftmaxLocalGenerationError::Attention(error.to_string()))?;
    let all_layers_selected = session.selected_layer_count() == shape.layers
        && (0..shape.layers).all(|layer| session.layer_is_selected(layer));
    if !all_layers_selected {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "qualification did not select every decoder layer".to_owned(),
        ));
    }
    let mut logits = vec![0.0_f32; shape.vocabulary];
    for (position, &token) in prefix_token_ids.iter().enumerate() {
        oracle
            .step_causal_attention_transport(&mut session, token as usize, position, &mut logits)
            .map_err(|error| R4SoftmaxLocalGenerationError::Attention(error.to_string()))?;
    }
    session
        .transport_status()
        .map_err(R4SoftmaxLocalGenerationError::Attention)?;
    if session.policy_identity() != HELM_D_R4_GAUGE_SOFTMAX_POLICY
        || session.output_policy() != output_policy
    {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "qualification session policy identity drifted".to_owned(),
        ));
    }
    let implementation: R4SpinTransportEvidence = serde_json::from_str(
        &session
            .transport_implementation_evidence()
            .map_err(R4SoftmaxLocalGenerationError::Attention)?
            .ok_or_else(|| {
                R4SoftmaxLocalGenerationError::Audit(
                    "R4 transport emitted no implementation evidence".to_owned(),
                )
            })?,
    )
    .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    let observed_causal: CausalAttentionAuditRecord = session.audit().into();
    let observed_projection: ProjectionAuditRecord = session.pre_rope_projection_audit().into();
    let expected_causal = expected_causal_audit(PREFIX_PARITY_TOKENS, shape)
        .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    let expected_projection = expected_projection_audit(PREFIX_PARITY_TOKENS, shape)
        .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    let expected_r4 = expected_r4_audit(PREFIX_PARITY_TOKENS, shape)
        .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    if observed_causal != expected_causal
        || observed_projection != expected_projection
        || implementation.policy_identity != HELM_D_R4_GAUGE_SOFTMAX_POLICY
        || implementation.intervention != R4SpinTransportIntervention::Coherent
        || implementation.frame_table_offsets.len() != PREFIX_PARITY_TOKENS
        || implementation.audit != expected_r4
        || observed_causal.future_reads != 0
        || implementation.audit.future_position_reads != 0
    {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "qualification causal/projection/R4 audit was not exact".to_owned(),
        ));
    }
    let output_audit = output_policy_audit_record(
        session.output_policy_audit(),
        PREFIX_PARITY_TOKENS,
        shape,
        output_policy,
    )?;
    let top1_token_id = greedy_token(&logits)
        .map_err(|error| R4SoftmaxLocalGenerationError::Source(error.to_string()))?;
    let state_ledger_cid = cid_serializable(&[session.persistent_state_cid()])?;
    let audit = QualificationArmAudit {
        sessions: 1,
        positions_per_session: PREFIX_PARITY_TOKENS,
        total_positions: PREFIX_PARITY_TOKENS as u64,
        selected_layer_count: session.selected_layer_count(),
        all_layers_selected,
        causal_audits_exact: 1,
        projection_audits_exact: 1,
        r4_audits_exact: 1,
        output_policy_audits_exact: 1,
        future_reads: 0,
        output_policy_applications: output_audit.applications,
        enabled_applications: output_audit.enabled_applications,
        zeroed_applications: output_audit.zeroed_applications,
        output_lanes: output_audit.output_lanes,
        nonzero_lanes_before_policy: output_audit.nonzero_lanes_before_policy,
        nonzero_lanes_after_policy: output_audit.nonzero_lanes_after_policy,
        applications_by_layer: output_audit.applications_by_layer,
        state_ledger_cid,
    };
    let policy_cid = cid_serializable(&(
        HELM_D_R4_GAUGE_SOFTMAX_POLICY,
        output_policy.identity(),
        "all-decoder-layers",
    ))?;
    let output_cid = logits_output_cid(output_policy, top1_token_id, &logits)?;
    let audit_cid = cid_serializable(&audit)?;
    Ok(QualificationArmExecution {
        result: QualificationArmResult {
            attention_output_policy: output_policy.identity().to_owned(),
            policy_cid,
            top1_token_id,
            output_cid,
            audit_cid,
            audit,
        },
        prefix_logits: logits,
    })
}

fn logits_output_cid(
    output_policy: CausalAttentionOutputPolicy,
    top1_token_id: u32,
    logits: &[f32],
) -> Result<String, R4SoftmaxLocalGenerationError> {
    if logits.iter().any(|value| !value.is_finite()) {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "qualification emitted a non-finite logit".to_owned(),
        ));
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(output_policy.identity().as_bytes());
    hasher.update(&top1_token_id.to_le_bytes());
    for value in logits {
        hasher.update(&value.to_bits().to_le_bytes());
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn prefix_parity_evidence(
    output_policy: CausalAttentionOutputPolicy,
    python: PythonPrefixArmReference,
    rust_logits: Vec<f32>,
) -> Result<PrefixParityEvidence, R4SoftmaxLocalGenerationError> {
    if python.logits.len() != rust_logits.len() || rust_logits.len() != 4096 {
        return Err(R4SoftmaxLocalGenerationError::Audit(
            "Python/Rust prefix logit shapes differ".to_owned(),
        ));
    }
    let rust_top1_token_id = greedy_token(&rust_logits)
        .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    let maximum_absolute_logit_delta = python
        .logits
        .iter()
        .zip(&rust_logits)
        .map(|(left, right)| (f64::from(*left) - f64::from(*right)).abs())
        .fold(0.0_f64, f64::max);
    let maximum_absolute_logit_delta_limit = FixtureTolerances::default().logit_abs;
    let identical_top1 = python.top1_token_id == rust_top1_token_id;
    let maximum_absolute_logit_delta_within_limit =
        maximum_absolute_logit_delta <= maximum_absolute_logit_delta_limit;
    Ok(PrefixParityEvidence {
        attention_output_policy: output_policy.identity().to_owned(),
        python_top1_token_id: python.top1_token_id,
        rust_top1_token_id,
        identical_top1,
        maximum_absolute_logit_delta,
        maximum_absolute_logit_delta_limit,
        maximum_absolute_logit_delta_within_limit,
        python_logits: python.logits,
        rust_logits,
        passed: identical_top1 && maximum_absolute_logit_delta_within_limit,
    })
}

fn qualification_decision_cid(
    checkpoint: &LocalCheckpointBinding,
    provenance: &QualificationProvenance,
    input: &QualificationInputBinding,
    enabled: &QualificationArmResult,
    attention_off: &QualificationArmResult,
    enabled_parity: &PrefixParityEvidence,
    attention_off_parity: &PrefixParityEvidence,
) -> Result<String, R4SoftmaxLocalGenerationError> {
    cid_serializable(&serde_json::json!({
        "schema": QUALIFICATION_REPORT_SCHEMA,
        "checkpoint_tree_cid": checkpoint.checkpoint_tree_cid,
        "weights_cid": checkpoint.weights_cid,
        "provenance": provenance,
        "token_store_cid": input.token_store_cid,
        "python_prefix_logits_cid": input.python_prefix_logits_cid,
        "python_prefix_result_cid": input.python_prefix_result_cid,
        "prefix_token_ids": input.prefix_token_ids,
        "enabled": enabled,
        "attention_off": attention_off,
        "enabled_parity": enabled_parity,
        "attention_off_parity": attention_off_parity,
    }))
}

fn local_transcript(
    tokenizer: &HfBpeTokenizer,
    prompt: &str,
    input_tokens: usize,
    eos_token_id: u32,
    generated_token_ids: Vec<u32>,
) -> RolloutTranscript {
    let first_eos_offset = generated_token_ids
        .iter()
        .position(|&token| token == eos_token_id);
    let response_end = first_eos_offset.unwrap_or(generated_token_ids.len());
    let raw_bytes = tokenizer.decode_bytes(&generated_token_ids);
    let response_bytes = tokenizer.decode_bytes(&generated_token_ids[..response_end]);
    RolloutTranscript {
        prompt_id: "R4-SOFTMAX-LOCAL".to_owned(),
        prompt: prompt.to_owned(),
        input_tokens,
        generated_token_ids: generated_token_ids.clone(),
        raw_decoded: String::from_utf8_lossy(&raw_bytes).into_owned(),
        response_text: String::from_utf8_lossy(&response_bytes).trim().to_owned(),
        first_eos_offset,
        utf8_decodable: String::from_utf8(raw_bytes).is_ok()
            && String::from_utf8(response_bytes).is_ok(),
        short_cycle_period: short_cycle_period(&generated_token_ids),
    }
}

fn decision_cid(identity: &DecisionIdentity<'_>) -> Result<String, R4SoftmaxLocalGenerationError> {
    let bytes = serde_json::to_vec(identity)
        .map_err(|error| R4SoftmaxLocalGenerationError::Audit(error.to_string()))?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TinyCheckpoint(PathBuf);

    impl TinyCheckpoint {
        fn new() -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "uor-r4-1014-local-generation-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).expect("create tiny checkpoint");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TinyCheckpoint {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn bf16_data(elements: usize, salt: u16) -> Vec<u8> {
        (0..elements)
            .flat_map(|index| {
                (0x3F00_u16 | ((index as u16).wrapping_mul(31).wrapping_add(salt) & 0x7F))
                    .to_le_bytes()
            })
            .collect()
    }

    fn safetensors_bytes(tensors: &[(&str, &[usize], Vec<u8>)]) -> Vec<u8> {
        let mut entries = Vec::with_capacity(tensors.len());
        let mut data = Vec::new();
        let mut offset = 0_usize;
        for (name, shape, bytes) in tensors {
            let end = offset + bytes.len();
            let shape = shape
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            entries.push(format!(
                "\"{name}\":{{\"dtype\":\"BF16\",\"shape\":[{shape}],\"data_offsets\":[{offset},{end}]}}"
            ));
            data.extend_from_slice(bytes);
            offset = end;
        }
        let header = format!("{{{}}}", entries.join(","));
        let mut shard = (header.len() as u64).to_le_bytes().to_vec();
        shard.extend_from_slice(header.as_bytes());
        shard.extend_from_slice(&data);
        shard
    }

    fn write_tiny_checkpoint() -> TinyCheckpoint {
        let checkpoint = TinyCheckpoint::new();
        std::fs::write(
            checkpoint.path().join("config.json"),
            br#"{"hidden_size":8,"intermediate_size":16,"num_hidden_layers":1,"num_attention_heads":2,"num_key_value_heads":2,"vocab_size":10,"max_position_embeddings":8,"tie_word_embeddings":true}"#,
        )
        .expect("write config");
        std::fs::write(
            checkpoint.path().join("tokenizer.json"),
            br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"a":0,"b":1,"c":2,"d":3,"e":4,"f":5,"g":6,"h":7,"i":8,"j":9},"merges":[]}}"#,
        )
        .expect("write tokenizer");
        let tensors = vec![
            ("model.embed_tokens.weight", &[10, 8][..], bf16_data(80, 20)),
            (
                "model.layers.0.input_layernorm.weight",
                &[8][..],
                bf16_data(8, 21),
            ),
            (
                "model.layers.0.self_attn.q_proj.weight",
                &[8, 8][..],
                bf16_data(64, 22),
            ),
            (
                "model.layers.0.self_attn.k_proj.weight",
                &[8, 8][..],
                bf16_data(64, 23),
            ),
            (
                "model.layers.0.self_attn.v_proj.weight",
                &[8, 8][..],
                bf16_data(64, 24),
            ),
            (
                "model.layers.0.self_attn.o_proj.weight",
                &[8, 8][..],
                bf16_data(64, 25),
            ),
            (
                "model.layers.0.post_attention_layernorm.weight",
                &[8][..],
                bf16_data(8, 26),
            ),
            (
                "model.layers.0.mlp.gate_proj.weight",
                &[16, 8][..],
                bf16_data(128, 27),
            ),
            (
                "model.layers.0.mlp.down_proj.weight",
                &[8, 16][..],
                bf16_data(128, 28),
            ),
            (
                "model.layers.0.mlp.up_proj.weight",
                &[16, 8][..],
                bf16_data(128, 29),
            ),
            ("model.norm.weight", &[8][..], bf16_data(8, 30)),
        ];
        std::fs::write(
            checkpoint.path().join("model.safetensors"),
            safetensors_bytes(&tensors),
        )
        .expect("write weights");
        checkpoint
    }

    fn config(prompt: &str, max_new_tokens: usize) -> R4SoftmaxLocalGeneratorConfig {
        R4SoftmaxLocalGeneratorConfig {
            model: PathBuf::from("/not-opened-by-request-tests"),
            prompt: prompt.to_owned(),
            max_new_tokens,
            workers: NonZeroUsize::new(DEFAULT_WORKERS).expect("default workers are nonzero"),
            attention_off: false,
            seed: None,
        }
    }

    #[test]
    fn request_bounds_fail_before_checkpoint_access() {
        assert!(validate_request(&config("", 1)).is_err());
        assert!(validate_request(&config("a", 0)).is_err());
        assert!(validate_request(&config("a", MAX_NEW_TOKENS + 1)).is_err());
        assert!(validate_request(&config("a", MAX_NEW_TOKENS)).is_ok());
    }

    #[test]
    fn transcript_uses_checkpoint_eos_and_reports_utf8_and_cycles() {
        let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(
            br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"a":0,"b":1,"c":2},"merges":[]}}"#,
        )
        .expect("tiny byte BPE");
        let transcript = local_transcript(&tokenizer, "a", 1, 1, vec![0, 1, 2]);
        assert_eq!(transcript.first_eos_offset, Some(1));
        assert_eq!(transcript.response_text, "a");
        assert!(transcript.utf8_decodable);
        assert_eq!(
            local_transcript(&tokenizer, "a", 1, 2, vec![0; 3]).short_cycle_period,
            Some(1)
        );
    }

    #[test]
    fn seeded_sampler_and_two_arm_prefix_fixture_are_deterministic_and_fail_closed() {
        let logits = (0..64)
            .map(|token| (token as f32 - 32.0) / 7.0)
            .collect::<Vec<_>>();
        let mut left = SplitMix64::new(1014);
        let mut right = SplitMix64::new(1014);
        let left_tokens = (0..16)
            .map(|_| sample_top_k_q32(&logits, &mut left).expect("sample left"))
            .collect::<Vec<_>>();
        let right_tokens = (0..16)
            .map(|_| sample_top_k_q32(&logits, &mut right).expect("sample right"))
            .collect::<Vec<_>>();
        assert_eq!(left_tokens, right_tokens);

        let mut enabled = vec![-1.0_f32; 4096];
        enabled[7] = 2.0;
        let mut attention_off = vec![-1.0_f32; 4096];
        attention_off[8] = 2.0;
        let mut reference = PythonPrefixLogitsReference {
            schema: PYTHON_PREFIX_LOGITS_SCHEMA.to_owned(),
            token_store_cid: raw_cid(b"tokens"),
            weights_cid: raw_cid(b"weights"),
            prefix_token_ids: (0..PREFIX_PARITY_TOKENS as u32).collect(),
            maximum_absolute_logit_delta_limit: FixtureTolerances::default().logit_abs,
            enabled: PythonPrefixArmReference {
                top1_token_id: 7,
                logits: enabled,
            },
            attention_off: PythonPrefixArmReference {
                top1_token_id: 8,
                logits: attention_off,
            },
            result_cid: String::new(),
        };
        let mut unsigned = serde_json::to_value(&reference).expect("serialize fixture");
        unsigned
            .as_object_mut()
            .expect("fixture is an object")
            .remove("result_cid");
        reference.result_cid =
            raw_cid(&canonical_json_bytes(&unsigned).expect("canonical fixture"));
        let signed_bytes = canonical_json_bytes(
            &serde_json::to_value(&reference).expect("serialize signed fixture"),
        )
        .expect("canonical signed fixture");
        verify_embedded_cid_bytes(
            &signed_bytes,
            "result_cid",
            &reference.result_cid,
            "test fixture",
        )
        .expect("fixture CID reproduces");
        validate_python_prefix_reference_envelope(&reference).expect("valid two-arm fixture");
        reference.enabled.top1_token_id = 9;
        assert!(validate_python_prefix_reference_envelope(&reference).is_err());
    }

    #[test]
    fn tiny_checkpoint_runs_all_layers_deterministically_with_enabled_and_off_audits() {
        let checkpoint = write_tiny_checkpoint();
        let mut request = R4SoftmaxLocalGeneratorConfig {
            model: checkpoint.path().to_owned(),
            prompt: "a".to_owned(),
            max_new_tokens: 1,
            workers: NonZeroUsize::new(1).expect("one worker is nonzero"),
            attention_off: false,
            seed: Some(1014),
        };
        let first = run_r4_softmax_local_generation(&request).expect("enabled generation");
        let replay = run_r4_softmax_local_generation(&request).expect("enabled replay");
        assert_eq!(first.decision_cid, replay.decision_cid);
        assert_eq!(
            first.transcript.generated_token_ids,
            replay.transcript.generated_token_ids
        );
        assert!(first.attention_audit.all_layers_selected);
        assert!(first.attention_audit.causal_audit_exact);
        assert!(first.attention_audit.projection_audit_exact);
        assert!(first.attention_audit.r4_audit_exact);
        assert!(first.attention_audit.zero_future_reads);
        assert_eq!(first.prompt_token_ids, vec![1, 0]);
        assert_eq!(first.checkpoint.bos_token_id, 1);
        assert_eq!(first.decode_audit.bos_insertions, 1);
        assert_eq!(first.decode_audit.seed, Some(1014));
        assert_eq!(first.decode_audit.sampler_policy, SEEDED_SAMPLER_POLICY);
        assert_eq!(first.attention_output_policy_audit.applications, 2);
        assert_eq!(first.attention_output_policy_audit.enabled_applications, 2);
        assert_eq!(first.attention_output_policy_audit.zeroed_applications, 0);
        assert_eq!(
            first.attention_output_policy_audit.applications_by_layer,
            vec![2]
        );
        assert!(first.attention_output_policy_audit.exact);
        assert_eq!(first.source_read_audit.provider_calls, 0);
        assert_eq!(first.source_read_audit.ollama_calls, 0);
        assert_eq!(first.source_read_audit.prior_trace_reads, 0);
        assert_eq!(first.checkpoint.files.len(), 3);
        assert_ne!(
            first.checkpoint.checkpoint_tree_cid,
            first.checkpoint.weights_cid
        );

        request.attention_off = true;
        let off = run_r4_softmax_local_generation(&request).expect("attention-off generation");
        assert_eq!(
            off.attention_output_policy_audit.policy,
            CausalAttentionOutputPolicy::ZeroPostWoBeforeResidual
                .identity()
                .to_owned()
        );
        assert_eq!(off.attention_output_policy_audit.applications, 2);
        assert_eq!(off.attention_output_policy_audit.enabled_applications, 0);
        assert_eq!(off.attention_output_policy_audit.zeroed_applications, 2);
        assert_eq!(
            off.attention_output_policy_audit.nonzero_lanes_after_policy,
            0
        );
        assert!(off.attention_output_policy_audit.exact);
    }
}
