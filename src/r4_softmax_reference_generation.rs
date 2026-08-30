//! Bounded native generation through the established R4/Spin causal-softmax seam.
//!
//! This is a reference and product-smoke path, not the deployed table runtime.
//! The pinned SmolLM2 checkpoint retains learned Q/K/V, RoPE, residual/MLP
//! blocks, stable causal softmax, and its LM head. Every decoder attention
//! layer expresses its complete causal prefix through exact R4/Spin frames.

use std::fmt;
use std::io;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use uor_r4_core::helm_d_r4_attention::{
    R4SpinCausalAttentionTransport, R4SpinTransportAudit, R4SpinTransportEvidence,
    R4SpinTransportIntervention, HELM_D_R4_GAUGE_SOFTMAX_POLICY, HELM_D_UPSTREAM_COMMIT,
};
use uor_r4_core::transformerless::hf_bpe::{HfBpeTokenizer, TokenizerAdapter};
use uor_r4_model_source::attention::{
    CausalAttentionLayerSelection, CausalAttentionProjectionAudit, CausalAttentionTransportAudit,
};
use uor_r4_model_source::{
    ExactBackendReport, HuggingFaceLlamaOracle, TeacherExecutionConfig, TeacherExecutionSnapshot,
    TeacherOracle,
};

use crate::geometric_decoder::{
    greedy_token, is_eos_token, read_chat_template, render_chat_prompt, short_cycle_period,
    transcript, validate_source, RolloutTranscript, PINNED_CHAT_TEMPLATE, PINNED_EOS_TOKEN_ID,
    PINNED_SOURCE_CID, PINNED_TOKENIZER_CID, SOURCE_REPOSITORY,
};

pub const REPORT_SCHEMA: &str = "uor-r4.r4-softmax-reference-generation/1";
pub const POLICY_SCHEMA: &str = "R4SoftmaxReferenceGeneratorV1";
pub const DEFAULT_MAX_NEW_TOKENS: usize = 32;
pub const MAX_NEW_TOKENS: usize = 128;

#[derive(Clone, Debug)]
pub struct R4SoftmaxReferenceGeneratorConfig {
    pub source: PathBuf,
    pub source_revision: String,
    pub prompt: String,
    pub max_new_tokens: usize,
    pub workers: NonZeroUsize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceBinding {
    pub repository: String,
    pub revision: String,
    pub source_path: String,
    pub weights_cid: String,
    pub tokenizer_cid: String,
    pub tokenizer: TokenizerAdapter,
    pub chat_template_cid: String,
    pub exact_backend: ExactBackendReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelShape {
    pub dimension: usize,
    pub hidden_dimension: usize,
    pub layers: usize,
    pub query_heads: usize,
    pub key_value_heads: usize,
    pub head_size: usize,
    pub vocabulary: usize,
    pub sequence_capacity: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyBinding {
    pub generator: String,
    pub attention_transport: String,
    pub attention_transport_cid: String,
    pub selected_layers: String,
    pub score: String,
    pub selector: String,
    pub aggregate: String,
    pub token_selection: String,
    pub eos_token_id: u32,
    pub cycle_policy: String,
    pub maximum_new_tokens: usize,
    pub helm_d_upstream_commit: String,
    pub helm_d_role: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStopReason {
    Eos,
    ShortCycle { period: usize },
    MaximumNewTokens,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CausalAttentionAuditRecord {
    pub positions: u64,
    pub layers: u64,
    pub heads: u64,
    pub query_transforms: u64,
    pub key_transports: u64,
    pub value_transports: u64,
    pub output_transforms: u64,
    pub future_reads: u64,
    pub maximum_query_position: Option<usize>,
    pub maximum_source_position: Option<usize>,
}

impl From<CausalAttentionTransportAudit> for CausalAttentionAuditRecord {
    fn from(audit: CausalAttentionTransportAudit) -> Self {
        Self {
            positions: audit.positions,
            layers: audit.layers,
            heads: audit.heads,
            query_transforms: audit.query_transforms,
            key_transports: audit.key_transports,
            value_transports: audit.value_transports,
            output_transforms: audit.output_transforms,
            future_reads: audit.future_reads,
            maximum_query_position: audit.maximum_query_position,
            maximum_source_position: audit.maximum_source_position,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectionAuditRecord {
    pub hook_calls: u64,
    pub query_vectors: u64,
    pub key_vectors: u64,
    pub value_vectors: u64,
    pub query_lanes: u64,
    pub key_lanes: u64,
    pub value_lanes: u64,
}

impl From<CausalAttentionProjectionAudit> for ProjectionAuditRecord {
    fn from(audit: CausalAttentionProjectionAudit) -> Self {
        Self {
            hook_calls: audit.hook_calls,
            query_vectors: audit.query_vectors,
            key_vectors: audit.key_vectors,
            value_vectors: audit.value_vectors,
            query_lanes: audit.query_lanes,
            key_lanes: audit.key_lanes,
            value_lanes: audit.value_lanes,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttentionAuditEvidence {
    pub selected_layer_count: usize,
    pub positions_executed: usize,
    pub observed_causal: CausalAttentionAuditRecord,
    pub expected_causal: CausalAttentionAuditRecord,
    pub causal_audit_exact: bool,
    pub observed_projection: ProjectionAuditRecord,
    pub expected_projection: ProjectionAuditRecord,
    pub projection_audit_exact: bool,
    pub r4_implementation: R4SpinTransportEvidence,
    pub expected_r4: R4SpinTransportAudit,
    pub r4_audit_exact: bool,
    pub zero_future_reads: bool,
    pub all_layers_selected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimingReport {
    pub source_load_seconds: f64,
    pub generation_seconds: f64,
    pub total_seconds: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct R4SoftmaxReferenceGenerationReport {
    pub schema: String,
    /// Stable replay identity. Deliberately excludes local paths and timing.
    pub decision_cid: String,
    pub issue: u32,
    pub claim_scope: String,
    pub source: SourceBinding,
    pub model_shape: ModelShape,
    pub policy: PolicyBinding,
    pub rendered_prompt: String,
    pub prompt_token_ids: Vec<u32>,
    pub transcript: RolloutTranscript,
    pub stop_reason: GenerationStopReason,
    pub persistent_state_cid: String,
    pub audit: AttentionAuditEvidence,
    pub execution: TeacherExecutionSnapshot,
    pub timing: TimingReport,
    pub nonclaims: Vec<String>,
}

#[derive(Serialize)]
struct DecisionIdentity<'a> {
    schema: &'static str,
    generator: &'static str,
    source_revision: &'a str,
    weights_cid: &'a str,
    tokenizer_cid: &'a str,
    attention_transport_cid: &'a str,
    model_shape: ModelShape,
    prompt: &'a str,
    rendered_prompt: &'a str,
    prompt_token_ids: &'a [u32],
    generated_token_ids: &'a [u32],
    stop_reason: &'a GenerationStopReason,
    persistent_state_cid: &'a str,
    audit: &'a AttentionAuditEvidence,
}

#[derive(Debug)]
pub enum R4SoftmaxReferenceGenerationError {
    InvalidRequest(String),
    InvalidSource(String),
    Tokenizer(String),
    Source(String),
    Attention(String),
    Audit(String),
    Io(io::Error),
}

impl fmt::Display for R4SoftmaxReferenceGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(reason) => {
                write!(formatter, "invalid generation request: {reason}")
            }
            Self::InvalidSource(reason) => write!(formatter, "invalid pinned source: {reason}"),
            Self::Tokenizer(reason) => write!(formatter, "source tokenizer unavailable: {reason}"),
            Self::Source(reason) => write!(formatter, "source decoder unavailable: {reason}"),
            Self::Attention(reason) => {
                write!(formatter, "R4 causal attention unavailable: {reason}")
            }
            Self::Audit(reason) => write!(formatter, "R4 causal attention audit failed: {reason}"),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for R4SoftmaxReferenceGenerationError {}

impl From<io::Error> for R4SoftmaxReferenceGenerationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn run_r4_softmax_reference_generation(
    config: &R4SoftmaxReferenceGeneratorConfig,
) -> Result<R4SoftmaxReferenceGenerationReport, R4SoftmaxReferenceGenerationError> {
    validate_request(config)?;
    let total_started = Instant::now();
    validate_source(&config.source, &config.source_revision)
        .map_err(|error| R4SoftmaxReferenceGenerationError::InvalidSource(error.to_string()))?;

    let tokenizer = HfBpeTokenizer::from_dir(&config.source)
        .map_err(|error| R4SoftmaxReferenceGenerationError::Tokenizer(error.to_string()))?;
    let tokenizer_cid = tokenizer.address();
    if tokenizer_cid != PINNED_TOKENIZER_CID {
        return Err(R4SoftmaxReferenceGenerationError::InvalidSource(format!(
            "tokenizer CID {tokenizer_cid} != pinned {PINNED_TOKENIZER_CID}"
        )));
    }
    let chat_template = read_chat_template(&config.source)
        .map_err(|error| R4SoftmaxReferenceGenerationError::Tokenizer(error.to_string()))?;
    if chat_template != PINNED_CHAT_TEMPLATE {
        return Err(R4SoftmaxReferenceGenerationError::InvalidSource(
            "tokenizer chat template does not match the pinned SmolLM2 template".to_owned(),
        ));
    }
    let rendered_prompt = render_chat_prompt(&config.prompt);
    let prompt_token_ids = tokenizer.encode(&rendered_prompt);
    if prompt_token_ids.is_empty() {
        return Err(R4SoftmaxReferenceGenerationError::Tokenizer(
            "rendered chat prompt encoded to zero tokens".to_owned(),
        ));
    }
    let sequence_capacity = checked_horizon(prompt_token_ids.len(), config.max_new_tokens)?;
    let maximum_sequence_capacity = source_max_position_embeddings(&config.source)?;
    if sequence_capacity > maximum_sequence_capacity {
        return Err(R4SoftmaxReferenceGenerationError::InvalidRequest(format!(
            "requested horizon {sequence_capacity} exceeds model capacity {maximum_sequence_capacity}"
        )));
    }

    let load_started = Instant::now();
    let execution_policy = TeacherExecutionConfig::fixed_workers(config.workers);
    let oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
        &config.source,
        sequence_capacity,
        execution_policy,
    )
    .map_err(|error| R4SoftmaxReferenceGenerationError::Source(error.to_string()))?;
    let source_load_seconds = load_started.elapsed().as_secs_f64();
    if oracle.source_cid() != PINNED_SOURCE_CID {
        return Err(R4SoftmaxReferenceGenerationError::InvalidSource(format!(
            "weights CID {} != pinned {PINNED_SOURCE_CID}",
            oracle.source_cid()
        )));
    }
    if oracle.cfg().vocab != tokenizer.vocab_size() {
        return Err(R4SoftmaxReferenceGenerationError::InvalidSource(format!(
            "model vocab {} != tokenizer vocab {}",
            oracle.cfg().vocab,
            tokenizer.vocab_size()
        )));
    }
    if oracle.cfg().seq_len != sequence_capacity {
        return Err(R4SoftmaxReferenceGenerationError::InvalidRequest(format!(
            "requested horizon {sequence_capacity} exceeds model capacity {}",
            oracle.cfg().seq_len
        )));
    }
    let source_eos = u32::try_from(TeacherOracle::eos_token(&oracle)).map_err(|_| {
        R4SoftmaxReferenceGenerationError::InvalidSource(
            "source EOS token does not fit the u32 token namespace".to_owned(),
        )
    })?;
    if source_eos != PINNED_EOS_TOKEN_ID {
        return Err(R4SoftmaxReferenceGenerationError::InvalidSource(format!(
            "source EOS token {source_eos} != pinned {PINNED_EOS_TOKEN_ID}"
        )));
    }

    let shape = model_shape(&oracle, sequence_capacity)?;
    let transport = R4SpinCausalAttentionTransport::new(
        u32::try_from(shape.vocabulary - 1).map_err(|_| {
            R4SoftmaxReferenceGenerationError::InvalidSource(
                "source vocabulary exceeds the u32 token namespace".to_owned(),
            )
        })?,
        sequence_capacity,
        R4SpinTransportIntervention::Coherent,
    )
    .map_err(|error| R4SoftmaxReferenceGenerationError::Attention(error.to_string()))?;
    let mut session = oracle
        .new_causal_attention_transport_session(
            Box::new(transport),
            CausalAttentionLayerSelection::All,
            sequence_capacity,
        )
        .map_err(|error| R4SoftmaxReferenceGenerationError::Attention(error.to_string()))?;
    let all_layers_selected = session.selected_layer_count() == shape.layers
        && (0..shape.layers).all(|layer| session.layer_is_selected(layer));
    if !all_layers_selected {
        return Err(R4SoftmaxReferenceGenerationError::Audit(
            "the generator did not select every decoder attention layer".to_owned(),
        ));
    }

    let generation_started = Instant::now();
    let mut logits = vec![0.0; shape.vocabulary];
    for (position, &token) in prompt_token_ids.iter().enumerate() {
        oracle
            .step_causal_attention_transport(&mut session, token as usize, position, &mut logits)
            .map_err(|error| R4SoftmaxReferenceGenerationError::Attention(error.to_string()))?;
    }
    let mut positions_executed = prompt_token_ids.len();
    let mut generated_token_ids = Vec::with_capacity(config.max_new_tokens);
    let mut stop_reason = GenerationStopReason::MaximumNewTokens;
    for decision in 0..config.max_new_tokens {
        let token = greedy_token(&logits)
            .map_err(|error| R4SoftmaxReferenceGenerationError::Source(error.to_string()))?;
        generated_token_ids.push(token);
        if is_eos_token(token) {
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
                R4SoftmaxReferenceGenerationError::InvalidRequest(
                    "generated position overflow".to_owned(),
                )
            })?;
        oracle
            .step_causal_attention_transport(&mut session, token as usize, position, &mut logits)
            .map_err(|error| R4SoftmaxReferenceGenerationError::Attention(error.to_string()))?;
        positions_executed += 1;
    }
    let generation_seconds = generation_started.elapsed().as_secs_f64();

    session
        .transport_status()
        .map_err(R4SoftmaxReferenceGenerationError::Attention)?;
    let policy_identity = session.policy_identity().to_owned();
    if policy_identity != HELM_D_R4_GAUGE_SOFTMAX_POLICY {
        return Err(R4SoftmaxReferenceGenerationError::Audit(format!(
            "transport policy {policy_identity:?} != established policy"
        )));
    }
    let implementation_json = session
        .transport_implementation_evidence()
        .map_err(R4SoftmaxReferenceGenerationError::Attention)?
        .ok_or_else(|| {
            R4SoftmaxReferenceGenerationError::Audit(
                "transport emitted no implementation evidence".to_owned(),
            )
        })?;
    let implementation: R4SpinTransportEvidence = serde_json::from_str(&implementation_json)
        .map_err(|error| {
            R4SoftmaxReferenceGenerationError::Audit(format!(
                "invalid R4 implementation evidence: {error}"
            ))
        })?;
    let observed_causal = CausalAttentionAuditRecord::from(session.audit());
    let expected_causal = expected_causal_audit(positions_executed, &shape)?;
    let observed_projection = ProjectionAuditRecord::from(session.pre_rope_projection_audit());
    let expected_projection = expected_projection_audit(positions_executed, &shape)?;
    let expected_r4 = expected_r4_audit(positions_executed, &shape)?;
    let causal_audit_exact = observed_causal == expected_causal;
    let projection_audit_exact = observed_projection == expected_projection;
    let r4_audit_exact = implementation.policy_identity == HELM_D_R4_GAUGE_SOFTMAX_POLICY
        && implementation.intervention == R4SpinTransportIntervention::Coherent
        && implementation.frame_table_offsets.len() == positions_executed
        && implementation.audit == expected_r4;
    let zero_future_reads =
        observed_causal.future_reads == 0 && implementation.audit.future_position_reads == 0;
    if !(causal_audit_exact && projection_audit_exact && r4_audit_exact && zero_future_reads) {
        return Err(R4SoftmaxReferenceGenerationError::Audit(format!(
            "exact audit mismatch: causal={causal_audit_exact}, projection={projection_audit_exact}, R4={r4_audit_exact}, zero_future_reads={zero_future_reads}"
        )));
    }

    let rollout = transcript(
        &tokenizer,
        "R4-SOFTMAX-REFERENCE",
        &config.prompt,
        prompt_token_ids.len(),
        generated_token_ids,
    );
    let tokenizer_adapter = tokenizer.adapter();
    let exact_backend = oracle.exact_backend_report();
    let persistent_state_cid = session.persistent_state_cid();
    let execution = oracle.execution_snapshot();
    let policy_cid = format!(
        "blake3:{}",
        blake3::hash(HELM_D_R4_GAUGE_SOFTMAX_POLICY.as_bytes()).to_hex()
    );
    let source = SourceBinding {
        repository: SOURCE_REPOSITORY.to_owned(),
        revision: config.source_revision.clone(),
        source_path: config.source.display().to_string(),
        weights_cid: oracle.source_cid().to_owned(),
        tokenizer_cid,
        tokenizer: tokenizer_adapter,
        chat_template_cid: format!("blake3:{}", blake3::hash(chat_template.as_bytes()).to_hex()),
        exact_backend,
    };
    let policy = PolicyBinding {
        generator: POLICY_SCHEMA.to_owned(),
        attention_transport: policy_identity,
        attention_transport_cid: policy_cid,
        selected_layers: "all decoder layers".to_owned(),
        score: "checkpoint learned Q/K; unchanged scaled dot product in query gauge".to_owned(),
        selector: "unchanged stable causal softmax over the complete prefix".to_owned(),
        aggregate: "unchanged weighted value sum in query gauge; compiler-side R4/Spin gauge decode before Wo"
            .to_owned(),
        token_selection: "deterministic greedy argmax; lower token id wins an exact tie".to_owned(),
        eos_token_id: PINNED_EOS_TOKEN_ID,
        cycle_policy: "stop after three repeated copies of a period-one-through-four tail"
            .to_owned(),
        maximum_new_tokens: config.max_new_tokens,
        helm_d_upstream_commit: HELM_D_UPSTREAM_COMMIT.to_owned(),
        helm_d_role: "credited MIT-licensed architectural reference; no HELM checkpoint or paper-result inheritance".to_owned(),
    };
    let audit = AttentionAuditEvidence {
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
    let decision_cid = decision_cid(&DecisionIdentity {
        schema: REPORT_SCHEMA,
        generator: POLICY_SCHEMA,
        source_revision: &source.revision,
        weights_cid: &source.weights_cid,
        tokenizer_cid: &source.tokenizer_cid,
        attention_transport_cid: &policy.attention_transport_cid,
        model_shape: shape,
        prompt: &config.prompt,
        rendered_prompt: &rendered_prompt,
        prompt_token_ids: &prompt_token_ids,
        generated_token_ids: &rollout.generated_token_ids,
        stop_reason: &stop_reason,
        persistent_state_cid: &persistent_state_cid,
        audit: &audit,
    })?;
    Ok(R4SoftmaxReferenceGenerationReport {
        schema: REPORT_SCHEMA.to_owned(),
        decision_cid,
        issue: 973,
        claim_scope: "bounded native reference generation through all-layer R4/Spin transported ordinary causal dot-product/softmax attention".to_owned(),
        source,
        model_shape: shape,
        policy,
        rendered_prompt,
        prompt_token_ids,
        transcript: rollout,
        stop_reason,
        persistent_state_cid,
        audit,
        execution,
        timing: TimingReport {
            source_load_seconds,
            generation_seconds,
            total_seconds: total_started.elapsed().as_secs_f64(),
        },
        nonclaims: vec![
            "not source-free or transformerless; the full pinned source decoder executes".to_owned(),
            "not multiplication-free, no_std, allocation-free, browser-WASM, or compiled-runtime inference".to_owned(),
            "does not establish geometry advantage or replace dot-product/softmax".to_owned(),
            "one decoded sample does not establish general text quality, meaning, reasoning, or release readiness".to_owned(),
        ],
    })
}

fn decision_cid(
    identity: &DecisionIdentity<'_>,
) -> Result<String, R4SoftmaxReferenceGenerationError> {
    let bytes = serde_json::to_vec(identity).map_err(|error| {
        R4SoftmaxReferenceGenerationError::Audit(format!(
            "decision identity serialization failed: {error}"
        ))
    })?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}

pub fn write_json_report(
    path: &Path,
    report: &R4SoftmaxReferenceGenerationReport,
) -> Result<(), R4SoftmaxReferenceGenerationError> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| R4SoftmaxReferenceGenerationError::Io(io::Error::other(error)))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

fn validate_request(
    config: &R4SoftmaxReferenceGeneratorConfig,
) -> Result<(), R4SoftmaxReferenceGenerationError> {
    if config.prompt.is_empty() {
        return Err(R4SoftmaxReferenceGenerationError::InvalidRequest(
            "--prompt must not be empty".to_owned(),
        ));
    }
    if config.max_new_tokens == 0 || config.max_new_tokens > MAX_NEW_TOKENS {
        return Err(R4SoftmaxReferenceGenerationError::InvalidRequest(format!(
            "--max-tokens must be in 1..={MAX_NEW_TOKENS}"
        )));
    }
    Ok(())
}

fn checked_horizon(
    prompt_tokens: usize,
    max_new_tokens: usize,
) -> Result<usize, R4SoftmaxReferenceGenerationError> {
    prompt_tokens.checked_add(max_new_tokens).ok_or_else(|| {
        R4SoftmaxReferenceGenerationError::InvalidRequest(
            "prompt plus generation horizon overflow".to_owned(),
        )
    })
}

fn source_max_position_embeddings(
    source: &Path,
) -> Result<usize, R4SoftmaxReferenceGenerationError> {
    let bytes = std::fs::read(source.join("config.json"))?;
    let config: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        R4SoftmaxReferenceGenerationError::InvalidSource(format!(
            "config.json is not valid JSON: {error}"
        ))
    })?;
    let capacity = config
        .get("max_position_embeddings")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            R4SoftmaxReferenceGenerationError::InvalidSource(
                "config.json has no unsigned max_position_embeddings".to_owned(),
            )
        })?;
    usize::try_from(capacity).map_err(|_| {
        R4SoftmaxReferenceGenerationError::InvalidSource(
            "max_position_embeddings exceeds this host's index domain".to_owned(),
        )
    })
}

pub(crate) fn model_shape(
    oracle: &HuggingFaceLlamaOracle,
    sequence_capacity: usize,
) -> Result<ModelShape, R4SoftmaxReferenceGenerationError> {
    let cfg = oracle.cfg();
    if cfg.n_heads == 0 || !cfg.dim.is_multiple_of(cfg.n_heads) {
        return Err(R4SoftmaxReferenceGenerationError::InvalidSource(
            "invalid source query-head layout".to_owned(),
        ));
    }
    Ok(ModelShape {
        dimension: cfg.dim,
        hidden_dimension: cfg.hidden,
        layers: cfg.n_layers,
        query_heads: cfg.n_heads,
        key_value_heads: cfg.n_kv_heads,
        head_size: cfg.dim / cfg.n_heads,
        vocabulary: cfg.vocab,
        sequence_capacity,
    })
}

pub(crate) fn expected_causal_audit(
    positions: usize,
    shape: &ModelShape,
) -> Result<CausalAttentionAuditRecord, R4SoftmaxReferenceGenerationError> {
    let positions = as_u64(positions, "positions")?;
    let layers = as_u64(shape.layers, "layers")?;
    let heads = as_u64(shape.query_heads, "query heads")?;
    let layer_calls = checked_mul(positions, layers, "causal layer calls")?;
    let head_calls = checked_mul(layer_calls, heads, "causal head calls")?;
    let prefix_sources = triangular(positions, "causal prefix sources")?;
    let source_calls = checked_mul(
        checked_mul(prefix_sources, layers, "causal source layers")?,
        heads,
        "causal source heads",
    )?;
    Ok(CausalAttentionAuditRecord {
        positions,
        layers: layer_calls,
        heads: head_calls,
        query_transforms: head_calls,
        key_transports: source_calls,
        value_transports: source_calls,
        output_transforms: head_calls,
        future_reads: 0,
        maximum_query_position: usize::try_from(positions)
            .ok()
            .and_then(|value| value.checked_sub(1)),
        maximum_source_position: usize::try_from(positions)
            .ok()
            .and_then(|value| value.checked_sub(1)),
    })
}

pub(crate) fn expected_projection_audit(
    positions: usize,
    shape: &ModelShape,
) -> Result<ProjectionAuditRecord, R4SoftmaxReferenceGenerationError> {
    let positions = as_u64(positions, "positions")?;
    let layers = as_u64(shape.layers, "layers")?;
    let query_heads = as_u64(shape.query_heads, "query heads")?;
    let key_value_heads = as_u64(shape.key_value_heads, "key/value heads")?;
    let head_size = as_u64(shape.head_size, "head size")?;
    let hook_calls = checked_mul(positions, layers, "projection hook calls")?;
    let query_vectors = checked_mul(hook_calls, query_heads, "projection query vectors")?;
    let key_vectors = checked_mul(hook_calls, key_value_heads, "projection key vectors")?;
    Ok(ProjectionAuditRecord {
        hook_calls,
        query_vectors,
        key_vectors,
        value_vectors: key_vectors,
        query_lanes: checked_mul(query_vectors, head_size, "projection query lanes")?,
        key_lanes: checked_mul(key_vectors, head_size, "projection key lanes")?,
        value_lanes: checked_mul(key_vectors, head_size, "projection value lanes")?,
    })
}

pub(crate) fn expected_r4_audit(
    positions: usize,
    shape: &ModelShape,
) -> Result<R4SpinTransportAudit, R4SoftmaxReferenceGenerationError> {
    let positions = as_u64(positions, "positions")?;
    let layers = as_u64(shape.layers, "layers")?;
    let heads = as_u64(shape.query_heads, "query heads")?;
    let blocks = as_u64(shape.head_size / 4, "R4 blocks per head")?;
    let query_blocks = checked_mul(
        checked_mul(
            checked_mul(positions, layers, "R4 query layers")?,
            heads,
            "R4 query heads",
        )?,
        blocks,
        "R4 query blocks",
    )?;
    let source_blocks = checked_mul(
        checked_mul(
            checked_mul(
                triangular(positions, "R4 prefix sources")?,
                layers,
                "R4 source layers",
            )?,
            heads,
            "R4 source heads",
        )?,
        blocks,
        "R4 source blocks",
    )?;
    let encoded = query_blocks
        .checked_add(checked_mul(source_blocks, 2, "R4 key/value encodings")?)
        .ok_or_else(audit_overflow)?;
    Ok(R4SpinTransportAudit {
        positions_prepared: positions,
        r4_blocks_encoded: encoded,
        key_blocks_transported: source_blocks,
        value_blocks_transported: source_blocks,
        output_blocks_decoded: query_blocks,
        future_position_reads: 0,
        source_frame_permutations: 0,
    })
}

fn as_u64(value: usize, label: &str) -> Result<u64, R4SoftmaxReferenceGenerationError> {
    u64::try_from(value).map_err(|_| {
        R4SoftmaxReferenceGenerationError::Audit(format!("{label} exceeds the audit domain"))
    })
}

fn checked_mul(
    left: u64,
    right: u64,
    label: &str,
) -> Result<u64, R4SoftmaxReferenceGenerationError> {
    left.checked_mul(right)
        .ok_or_else(|| R4SoftmaxReferenceGenerationError::Audit(format!("{label} overflowed")))
}

fn triangular(value: u64, label: &str) -> Result<u64, R4SoftmaxReferenceGenerationError> {
    value
        .checked_mul(value.checked_add(1).ok_or_else(audit_overflow)?)
        .and_then(|product| product.checked_div(2))
        .ok_or_else(|| R4SoftmaxReferenceGenerationError::Audit(format!("{label} overflowed")))
}

fn audit_overflow() -> R4SoftmaxReferenceGenerationError {
    R4SoftmaxReferenceGenerationError::Audit("audit arithmetic overflowed".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_decoder::PINNED_SOURCE_REVISION;

    fn config(prompt: &str, max_new_tokens: usize) -> R4SoftmaxReferenceGeneratorConfig {
        R4SoftmaxReferenceGeneratorConfig {
            source: PathBuf::from("/not-opened-by-pure-tests"),
            source_revision: PINNED_SOURCE_REVISION.to_owned(),
            prompt: prompt.to_owned(),
            max_new_tokens,
            workers: NonZeroUsize::new(4).expect("four is nonzero"),
        }
    }

    fn shape() -> ModelShape {
        ModelShape {
            dimension: 32,
            hidden_dimension: 64,
            layers: 3,
            query_heads: 4,
            key_value_heads: 2,
            head_size: 8,
            vocabulary: 128,
            sequence_capacity: 16,
        }
    }

    #[test]
    fn request_bounds_fail_before_any_source_access() {
        assert!(validate_request(&config("", 8)).is_err());
        assert!(validate_request(&config("hello", 0)).is_err());
        assert!(validate_request(&config("hello", MAX_NEW_TOKENS + 1)).is_err());
        assert!(validate_request(&config("hello", DEFAULT_MAX_NEW_TOKENS)).is_ok());
        assert!(checked_horizon(usize::MAX, 1).is_err());
    }

    #[test]
    fn source_capacity_is_rejected_before_weight_loading() {
        let directory =
            std::env::temp_dir().join(format!("uor-r4-r4-softmax-capacity-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("create temporary source");
        std::fs::write(
            directory.join("config.json"),
            br#"{"max_position_embeddings":2048}"#,
        )
        .expect("write temporary config");
        assert_eq!(
            source_max_position_embeddings(&directory).expect("read source capacity"),
            2048
        );
        std::fs::remove_dir_all(directory).expect("remove temporary source");
    }

    #[test]
    fn exact_audit_formulas_cover_every_prefix_vector_and_r4_block() {
        let shape = shape();
        let causal = expected_causal_audit(2, &shape).expect("causal audit");
        assert_eq!(causal.layers, 6);
        assert_eq!(causal.heads, 24);
        assert_eq!(causal.key_transports, 36);
        assert_eq!(causal.future_reads, 0);
        assert_eq!(causal.maximum_query_position, Some(1));

        let projection = expected_projection_audit(2, &shape).expect("projection audit");
        assert_eq!(projection.hook_calls, 6);
        assert_eq!(projection.query_vectors, 24);
        assert_eq!(projection.key_vectors, 12);
        assert_eq!(projection.query_lanes, 192);
        assert_eq!(projection.key_lanes, 96);

        let r4 = expected_r4_audit(2, &shape).expect("R4 audit");
        assert_eq!(r4.positions_prepared, 2);
        assert_eq!(r4.r4_blocks_encoded, 192);
        assert_eq!(r4.key_blocks_transported, 72);
        assert_eq!(r4.output_blocks_decoded, 48);
        assert_eq!(r4.future_position_reads, 0);
    }

    #[test]
    fn stop_reason_wire_labels_are_stable() {
        assert_eq!(
            serde_json::to_string(&GenerationStopReason::Eos).expect("serialize EOS"),
            "\"eos\""
        );
        assert_eq!(
            serde_json::to_string(&GenerationStopReason::ShortCycle { period: 2 })
                .expect("serialize cycle"),
            "{\"short_cycle\":{\"period\":2}}"
        );
    }

    #[test]
    fn decision_identity_is_exact_and_changes_with_a_generated_token() {
        let shape = shape();
        let expected_causal = expected_causal_audit(2, &shape).expect("causal audit");
        let expected_projection = expected_projection_audit(2, &shape).expect("projection audit");
        let expected_r4 = expected_r4_audit(2, &shape).expect("R4 audit");
        let audit = AttentionAuditEvidence {
            selected_layer_count: shape.layers,
            positions_executed: 2,
            observed_causal: expected_causal,
            expected_causal,
            causal_audit_exact: true,
            observed_projection: expected_projection,
            expected_projection,
            projection_audit_exact: true,
            r4_implementation: R4SpinTransportEvidence {
                schema: "uor-r4.r4-spin-transport-evidence/1".to_owned(),
                policy_identity: HELM_D_R4_GAUGE_SOFTMAX_POLICY.to_owned(),
                intervention: R4SpinTransportIntervention::Coherent,
                frame_table_offsets: vec![0, 1],
                audit: expected_r4,
            },
            expected_r4,
            r4_audit_exact: true,
            zero_future_reads: true,
            all_layers_selected: true,
        };
        let prompt_tokens = [1, 2];
        let first_generated = [3, 4];
        let changed_generated = [3, 5];
        let stop = GenerationStopReason::MaximumNewTokens;
        fn identity<'a>(
            generated_token_ids: &'a [u32],
            prompt_token_ids: &'a [u32],
            stop_reason: &'a GenerationStopReason,
            audit: &'a AttentionAuditEvidence,
            model_shape: ModelShape,
        ) -> DecisionIdentity<'a> {
            DecisionIdentity {
                schema: REPORT_SCHEMA,
                generator: POLICY_SCHEMA,
                source_revision: PINNED_SOURCE_REVISION,
                weights_cid: PINNED_SOURCE_CID,
                tokenizer_cid: PINNED_TOKENIZER_CID,
                attention_transport_cid: "blake3:policy",
                model_shape,
                prompt: "hello",
                rendered_prompt: "rendered hello",
                prompt_token_ids,
                generated_token_ids,
                stop_reason,
                persistent_state_cid: "blake3:state",
                audit,
            }
        }
        let first = decision_cid(&identity(
            &first_generated,
            &prompt_tokens,
            &stop,
            &audit,
            shape,
        ))
        .expect("first CID");
        let replay = decision_cid(&identity(
            &first_generated,
            &prompt_tokens,
            &stop,
            &audit,
            shape,
        ))
        .expect("replay CID");
        let changed = decision_cid(&identity(
            &changed_generated,
            &prompt_tokens,
            &stop,
            &audit,
            shape,
        ))
        .expect("changed CID");
        assert_eq!(first, replay);
        assert_ne!(first, changed);
    }
}
