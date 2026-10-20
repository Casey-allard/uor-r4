//! First source-free student compiled from the established R4/Spin softmax oracle.
//!
//! The live experiment is deliberately split in two. [`preflight`] reads only
//! the pinned tokenizer and the frozen natural-language texts, proving that the
//! held-out document can reach construction-fitted suffix rows before any
//! source weights execute. The model-backed trace/compile/score half is added
//! beside this seam and must preserve that partition.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use uor_r4_core::helm_d_r4_attention::{
    R4SpinCausalAttentionTransport, R4SpinTransportEvidence, R4SpinTransportIntervention,
    HELM_D_R4_GAUGE_SOFTMAX_POLICY,
};
use uor_r4_core::r4_softmax_trace_student::{
    compile_r4_softmax_trace_student, R4SoftmaxTraceSequence, R4SoftmaxTraceStudentArm,
    R4SoftmaxTraceStudentArtifact, R4SoftmaxTraceStudentConfig, R4SoftmaxTraceStudentDistribution,
    R4SoftmaxTraceStudentEvaluation, R4SoftmaxTraceStudentRuntime, TeacherTopDistributionQ16,
    TeacherTopTokenQ16, R4_SOFTMAX_TRACE_Q16_TOTAL,
};
use uor_r4_core::source_free_table::d3_is_held_out;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_model_source::attention::{CausalAttentionLayerSelection, CausalAttentionTransport};
use uor_r4_model_source::{
    CausalAttentionTransportSession, HuggingFaceLlamaOracle, TeacherExecutionConfig,
    TeacherExecutionSnapshot, TeacherOracle,
};

use crate::geometric_decoder::{
    validate_source, PINNED_BOS_TOKEN_ID, PINNED_EOS_TOKEN_ID, PINNED_SOURCE_CID,
    PINNED_SOURCE_REVISION, PINNED_TOKENIZER_CID,
};
use crate::r4_softmax_reference_generation::{
    expected_causal_audit, expected_projection_audit, expected_r4_audit, model_shape,
    AttentionAuditEvidence, CausalAttentionAuditRecord, ModelShape, ProjectionAuditRecord,
};
use crate::r4_softmax_teacher_trace::{
    R4SoftmaxTeacherTrace, R4SoftmaxTeacherTraceBounds, R4SoftmaxTeacherTraceBundle,
    R4SoftmaxTeacherTraceIdentity, TracingR4SpinTransport,
};

pub const PREFLIGHT_SCHEMA: &str = "uor-r4.r4-softmax-trace-student-preflight/1";
pub const TRACE_SCHEMA: &str = "R4SoftmaxTeacherTraceV1";
pub const COMPILER_SCHEMA: &str = "R4SoftmaxTraceCompilerV1";
pub const FREEZE_SCHEMA: &str = "uor-r4.r4-softmax-trace-student-freeze/1";
pub const RESULT_SCHEMA: &str = "uor-r4.r4-softmax-trace-student-result/1";
pub const MAX_SUFFIX_DEPTH: usize = 4;
pub const ATTENTION_SUPPORT: usize = 8;
pub const LOGIT_SUPPORT: usize = 32;

const CONSTRUCTION_DOCUMENTS: [FrozenDocument; 4] = [
    FrozenDocument {
        id: "14",
        text: "She was born in Ottawa, Canada.",
    },
    FrozenDocument {
        id: "657",
        text: "He was born in Bombay, India.",
    },
    FrozenDocument {
        id: "4579",
        text: "Alexander Graham Bell was born in Edinburgh, Scotland.",
    },
    FrozenDocument {
        id: "5121",
        text: "He was born in Shrewsbury, Shropshire.",
    },
];

const HELD_OUT_DOCUMENT: FrozenDocument = FrozenDocument {
    id: "13",
    text: "Alan Mathison Turing OBE FRS (London, 23 June 1912 – Wilmslow, Cheshire, 7 June 1954) was an English mathematician and computer scientist. He was born in Maida Vale, London.",
};

#[derive(Clone, Copy)]
struct FrozenDocument {
    id: &'static str,
    text: &'static str,
}

#[derive(Clone, Debug)]
pub struct R4SoftmaxTraceExperimentConfig {
    pub source: PathBuf,
    pub source_revision: String,
    /// Exact uor-r4 Git commit whose implementation produced the evidence.
    pub implementation_revision: String,
    pub workers: NonZeroUsize,
    pub artifact_output: PathBuf,
    pub trace_output: PathBuf,
    pub freeze_output: PathBuf,
    pub result_output: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FrozenDocumentBinding {
    pub id: String,
    pub partition: String,
    pub text_cid: String,
    pub target_tokens: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SuffixReachability {
    pub held_out_positions: usize,
    pub positions_with_nonzero_suffix: usize,
    pub non_bos_positions_with_nonzero_suffix: usize,
    pub positions_with_depth_two_or_more: usize,
    pub positions_with_depth_three_or_more: usize,
    /// Exact longest reachable suffix depth, indexed by depth 0..=4.
    pub longest_depth_histogram: [usize; MAX_SUFFIX_DEPTH + 1],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct R4SoftmaxTracePreflight {
    pub schema: String,
    pub issue: u32,
    pub trace_policy: String,
    pub compiler_policy: String,
    pub source_revision: String,
    pub tokenizer_cid: String,
    pub maximum_suffix_depth: usize,
    pub attention_support: usize,
    pub logit_support: usize,
    pub construction_positions: usize,
    pub held_out_positions: usize,
    pub source_forward_positions: usize,
    pub documents: Vec<FrozenDocumentBinding>,
    pub reachability: SuffixReachability,
    pub frozen_shape_exact: bool,
    pub construction_held_out_text_cids_disjoint: bool,
    pub construction_partition_valid: bool,
    pub held_out_partition_valid: bool,
    pub proceed_to_source_trace: bool,
    pub decision: String,
    pub preflight_cid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct R4SoftmaxTraceFreeze {
    pub schema: String,
    pub issue: u32,
    pub claim_scope: String,
    pub implementation_revision: String,
    pub source_revision: String,
    pub source_cid: String,
    pub tokenizer_cid: String,
    pub attention_policy: String,
    pub attention_policy_cid: String,
    pub construction_manifest_cid: String,
    pub construction_documents: Vec<FrozenDocumentBinding>,
    pub construction_positions: usize,
    pub trace_bundle_cid: String,
    pub trace_bundle_bytes: usize,
    pub document_trace_cids: Vec<String>,
    pub artifact_cid: String,
    pub artifact_bytes: usize,
    pub artifact_construction_digest: String,
    pub artifact_rows: usize,
    pub artifact_rows_by_depth: [usize; MAX_SUFFIX_DEPTH + 1],
    pub artifact_reload_bytes_exact: bool,
    pub artifact_reload_cid_exact: bool,
    pub construction_audits: Vec<AttentionAuditEvidence>,
    pub source_execution: TeacherExecutionSnapshot,
    pub held_out_teacher_scored: bool,
    pub held_out_identity_bound_into_artifact: bool,
    pub compile_seconds: f64,
    pub freeze_cid: String,
    pub nonclaims: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextArmEvaluation {
    pub positions: u64,
    pub teacher_mass_covered_q16: u64,
    pub teacher_top1_agreements: u64,
    pub actual_next_top1_correct: u64,
    pub covered_teacher_cross_entropy_nats: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextBearingEvaluation {
    pub excluded_shared_bos_positions: u64,
    pub teacher_distilled: ContextArmEvaluation,
    pub observed_count: ContextArmEvaluation,
    pub document_permuted_control: ContextArmEvaluation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceFreeContinuation {
    pub prompt: String,
    pub prompt_token_ids: Vec<u32>,
    pub generated_token_ids: Vec<u32>,
    pub decoded_text: String,
    pub replay_token_ids: Vec<u32>,
    pub replay_exact: bool,
    pub source_execution_unchanged: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct R4SoftmaxTraceResult {
    pub schema: String,
    pub issue: u32,
    pub claim_scope: String,
    pub implementation_revision: String,
    pub evaluation_preflight: R4SoftmaxTracePreflight,
    pub construction_freeze_cid: String,
    pub artifact_cid_before_reveal: String,
    pub artifact_cid_after_reveal: String,
    pub artifact_unchanged_across_reveal: bool,
    pub held_out_document: FrozenDocumentBinding,
    pub held_out_trace_cid: String,
    /// Compact held-out judge input retained for source-free metric replay.
    /// This sequence is revealed only after the construction artifact is frozen.
    pub held_out_judge_sequence: R4SoftmaxTraceSequence,
    pub held_out_audit: AttentionAuditEvidence,
    pub source_execution_after_held_out_trace: TeacherExecutionSnapshot,
    pub held_out_evaluation: R4SoftmaxTraceStudentEvaluation,
    pub context_bearing_evaluation: ContextBearingEvaluation,
    pub continuation: SourceFreeContinuation,
    pub distilled_beats_count_cross_entropy: bool,
    pub distilled_beats_count_teacher_top1: bool,
    pub distilled_actual_top1_not_worse: bool,
    pub permuted_control_loses_cross_entropy: bool,
    pub permuted_control_loses_teacher_top1: bool,
    pub exact_replay: bool,
    pub zero_runtime_source_forwards: bool,
    pub terminal: String,
    pub reveal_seconds: f64,
    pub result_cid: String,
    pub nonclaims: Vec<String>,
}

#[derive(Debug)]
pub enum R4SoftmaxTraceExperimentError {
    InvalidRequest(String),
    Tokenizer(String),
    Source(String),
    Attention(String),
    Trace(String),
    Student(String),
    Serialization(String),
    Io(std::io::Error),
}

impl fmt::Display for R4SoftmaxTraceExperimentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(reason) => write!(formatter, "invalid trace experiment: {reason}"),
            Self::Tokenizer(reason) => write!(formatter, "trace tokenizer unavailable: {reason}"),
            Self::Source(reason) => write!(formatter, "trace source unavailable: {reason}"),
            Self::Attention(reason) => write!(formatter, "R4 trace attention failed: {reason}"),
            Self::Trace(reason) => write!(formatter, "R4 teacher trace failed: {reason}"),
            Self::Student(reason) => {
                write!(formatter, "source-free trace student failed: {reason}")
            }
            Self::Serialization(reason) => {
                write!(formatter, "trace evidence serialization failed: {reason}")
            }
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for R4SoftmaxTraceExperimentError {}

impl From<std::io::Error> for R4SoftmaxTraceExperimentError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TokenizedDocument {
    pub id: String,
    pub text: String,
    pub targets: Vec<u32>,
    pub inputs: Vec<u32>,
}

pub fn preflight(
    config: &R4SoftmaxTraceExperimentConfig,
) -> Result<R4SoftmaxTracePreflight, R4SoftmaxTraceExperimentError> {
    if config.source_revision != PINNED_SOURCE_REVISION {
        return Err(R4SoftmaxTraceExperimentError::InvalidRequest(format!(
            "source revision {} != pinned {PINNED_SOURCE_REVISION}",
            config.source_revision
        )));
    }
    let tokenizer = HfBpeTokenizer::from_dir(&config.source)
        .map_err(|error| R4SoftmaxTraceExperimentError::Tokenizer(error.to_string()))?;
    let tokenizer_cid = tokenizer.address();
    if tokenizer_cid != PINNED_TOKENIZER_CID {
        return Err(R4SoftmaxTraceExperimentError::Tokenizer(format!(
            "tokenizer CID {tokenizer_cid} != pinned {PINNED_TOKENIZER_CID}"
        )));
    }

    let construction = CONSTRUCTION_DOCUMENTS
        .iter()
        .map(|document| tokenize_document(&tokenizer, *document))
        .collect::<Result<Vec<_>, _>>()?;
    let held_out = tokenize_document(&tokenizer, HELD_OUT_DOCUMENT)?;
    let construction_positions = construction
        .iter()
        .try_fold(0usize, |sum, document| {
            sum.checked_add(document.targets.len())
        })
        .ok_or_else(|| {
            R4SoftmaxTraceExperimentError::InvalidRequest(
                "construction position count overflowed".to_owned(),
            )
        })?;
    let held_out_positions = held_out.targets.len();

    let mut construction_keys: [BTreeSet<Vec<u32>>; MAX_SUFFIX_DEPTH + 1] =
        std::array::from_fn(|_| BTreeSet::new());
    for document in &construction {
        for position in 0..document.inputs.len() {
            for (depth, keys) in construction_keys
                .iter_mut()
                .enumerate()
                .take(MAX_SUFFIX_DEPTH.min(position + 1) + 1)
                .skip(1)
            {
                keys.insert(document.inputs[position + 1 - depth..=position].to_vec());
            }
        }
    }
    let mut longest_depth_histogram = [0usize; MAX_SUFFIX_DEPTH + 1];
    let mut positions_with_nonzero_suffix = 0usize;
    let mut non_bos_positions_with_nonzero_suffix = 0usize;
    for position in 0..held_out.inputs.len() {
        let mut longest = 0usize;
        for (depth, keys) in construction_keys
            .iter()
            .enumerate()
            .take(MAX_SUFFIX_DEPTH.min(position + 1) + 1)
            .skip(1)
        {
            if keys.contains(&held_out.inputs[position + 1 - depth..=position]) {
                longest = depth;
            }
        }
        longest_depth_histogram[longest] += 1;
        positions_with_nonzero_suffix += usize::from(longest > 0);
        non_bos_positions_with_nonzero_suffix += usize::from(position > 0 && longest > 0);
    }
    let positions_with_depth_two_or_more = longest_depth_histogram[2..].iter().sum();
    let positions_with_depth_three_or_more = longest_depth_histogram[3..].iter().sum();

    let construction_text_cids = construction
        .iter()
        .map(|document| text_cid(&document.text))
        .collect::<BTreeSet<_>>();
    let held_out_text_cid = text_cid(&held_out.text);
    let construction_partition_valid = construction
        .iter()
        .all(|document| !d3_is_held_out(&document.id));
    let held_out_partition_valid = d3_is_held_out(&held_out.id);
    let construction_held_out_text_cids_disjoint =
        !construction_text_cids.contains(&held_out_text_cid);
    let expected_frozen_shape = construction_positions == 38
        && held_out_positions == 57
        && longest_depth_histogram == [47, 8, 1, 1, 0];
    let proceed_to_source_trace = expected_frozen_shape
        && non_bos_positions_with_nonzero_suffix > 0
        && positions_with_depth_two_or_more > 0
        && positions_with_depth_three_or_more > 0
        && longest_depth_histogram[MAX_SUFFIX_DEPTH] == 0
        && construction_partition_valid
        && held_out_partition_valid
        && construction_held_out_text_cids_disjoint;

    let mut documents = construction
        .iter()
        .map(|document| binding(document, "construction"))
        .collect::<Vec<_>>();
    documents.push(binding(&held_out, "held_out"));
    let reachability = SuffixReachability {
        held_out_positions,
        positions_with_nonzero_suffix,
        non_bos_positions_with_nonzero_suffix,
        positions_with_depth_two_or_more,
        positions_with_depth_three_or_more,
        longest_depth_histogram,
    };
    let mut report = R4SoftmaxTracePreflight {
        schema: PREFLIGHT_SCHEMA.to_owned(),
        issue: 973,
        trace_policy: TRACE_SCHEMA.to_owned(),
        compiler_policy: COMPILER_SCHEMA.to_owned(),
        source_revision: config.source_revision.clone(),
        tokenizer_cid,
        maximum_suffix_depth: MAX_SUFFIX_DEPTH,
        attention_support: ATTENTION_SUPPORT,
        logit_support: LOGIT_SUPPORT,
        construction_positions,
        held_out_positions,
        source_forward_positions: construction_positions + held_out_positions,
        documents,
        reachability,
        frozen_shape_exact: expected_frozen_shape,
        construction_held_out_text_cids_disjoint,
        construction_partition_valid,
        held_out_partition_valid,
        proceed_to_source_trace,
        decision: if proceed_to_source_trace {
            "PROCEED_TO_BOUNDED_R4_SOFTMAX_TRACE".to_owned()
        } else {
            "STOP_UNREACHABLE_OR_INVALID_PARTITION".to_owned()
        },
        preflight_cid: String::new(),
    };
    let bytes = serde_json::to_vec(&report).map_err(|error| {
        R4SoftmaxTraceExperimentError::InvalidRequest(format!(
            "serialize trace preflight identity: {error}"
        ))
    })?;
    report.preflight_cid = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    Ok(report)
}

#[derive(Serialize)]
struct ConstructionManifestIdentity<'a> {
    schema: &'static str,
    source_revision: &'a str,
    source_cid: &'static str,
    tokenizer_cid: &'a str,
    attention_policy_cid: &'a str,
    maximum_suffix_depth: usize,
    attention_support: usize,
    logit_support: usize,
    documents: Vec<ConstructionDocumentIdentity<'a>>,
}

#[derive(Serialize)]
struct ConstructionDocumentIdentity<'a> {
    id: &'a str,
    text_cid: String,
    input_tokens: &'a [u32],
    actual_next_tokens: &'a [u32],
}

struct TraceDocumentOutcome {
    trace: R4SoftmaxTeacherTrace,
    sequence: R4SoftmaxTraceSequence,
    audit: AttentionAuditEvidence,
}

pub fn compile_construction(
    config: &R4SoftmaxTraceExperimentConfig,
) -> Result<R4SoftmaxTraceFreeze, R4SoftmaxTraceExperimentError> {
    let started = Instant::now();
    validate_distinct_output_paths(config)?;
    validate_implementation_revision(&config.implementation_revision)?;
    validate_source(&config.source, &config.source_revision)
        .map_err(|error| R4SoftmaxTraceExperimentError::Source(error.to_string()))?;
    let tokenizer = load_tokenizer(config)?;
    let construction = CONSTRUCTION_DOCUMENTS
        .iter()
        .map(|document| tokenize_document(&tokenizer, *document))
        .collect::<Result<Vec<_>, _>>()?;
    let construction_positions = construction
        .iter()
        .map(|document| document.targets.len())
        .sum();
    if construction_positions != 38
        || construction
            .iter()
            .any(|document| d3_is_held_out(&document.id))
    {
        return Err(R4SoftmaxTraceExperimentError::InvalidRequest(format!(
            "frozen construction partition drifted: positions={construction_positions}"
        )));
    }
    let maximum_positions = construction
        .iter()
        .map(|document| document.targets.len())
        .max()
        .ok_or_else(|| {
            R4SoftmaxTraceExperimentError::InvalidRequest(
                "construction partition is empty".to_owned(),
            )
        })?;
    let attention_policy_cid = policy_cid();
    let construction_manifest_cid = construction_manifest_cid(
        config,
        &tokenizer.address(),
        &attention_policy_cid,
        &construction,
    )?;
    let oracle = load_oracle(config, &tokenizer, maximum_positions)?;

    let mut outcomes = Vec::with_capacity(construction.len());
    for document in &construction {
        outcomes.push(trace_document(
            &oracle,
            document,
            &tokenizer.address(),
            &attention_policy_cid,
            &construction_manifest_cid,
            "D3-construction/14-657-4579-5121",
        )?);
    }
    let execution = oracle.execution_snapshot();
    if execution.streams_completed != construction_positions as u64 {
        return Err(R4SoftmaxTraceExperimentError::Source(format!(
            "construction execution completed {} streams, expected {construction_positions}",
            execution.streams_completed
        )));
    }

    let traces = outcomes
        .iter()
        .map(|outcome| outcome.trace.clone())
        .collect::<Vec<_>>();
    let typed_trace_bundle = R4SoftmaxTeacherTraceBundle::new(traces)
        .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
    let trace_bundle = typed_trace_bundle
        .canonical_bytes()
        .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
    let trace_bundle_cid = typed_trace_bundle
        .bundle_cid()
        .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
    let document_trace_cids = typed_trace_bundle
        .document_trace_cids()
        .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
    let sequences = outcomes
        .iter()
        .map(|outcome| outcome.sequence.clone())
        .collect::<Vec<_>>();
    let artifact = compile_r4_softmax_trace_student(
        R4SoftmaxTraceStudentConfig::new(LOGIT_SUPPORT)
            .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?,
        &sequences,
    )
    .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?;
    let artifact_bytes = artifact.to_bytes();
    let artifact_cid = artifact.artifact_cid();
    let reloaded = R4SoftmaxTraceStudentArtifact::from_bytes(&artifact_bytes)
        .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?;
    let artifact_reload_bytes_exact = reloaded.to_bytes() == artifact_bytes;
    let artifact_reload_cid_exact = reloaded.artifact_cid() == artifact_cid;
    if !(artifact_reload_bytes_exact && artifact_reload_cid_exact) {
        return Err(R4SoftmaxTraceExperimentError::Student(
            "canonical artifact reload changed bytes or CID".to_owned(),
        ));
    }

    write_atomic(&config.trace_output, &trace_bundle)?;
    write_atomic(&config.artifact_output, &artifact_bytes)?;
    let construction_documents = construction
        .iter()
        .map(|document| binding(document, "construction"))
        .collect::<Vec<_>>();
    let artifact_rows_by_depth = std::array::from_fn(|depth| artifact.rows_at_depth(depth as u8));
    let construction_audits = outcomes
        .into_iter()
        .map(|outcome| outcome.audit)
        .collect::<Vec<_>>();
    let mut freeze = R4SoftmaxTraceFreeze {
        schema: FREEZE_SCHEMA.to_owned(),
        issue: 973,
        claim_scope: "construction-only R4/Spin softmax trace and deterministic source-free Q16 suffix student"
            .to_owned(),
        implementation_revision: config.implementation_revision.clone(),
        source_revision: config.source_revision.clone(),
        source_cid: oracle.source_cid().to_owned(),
        tokenizer_cid: tokenizer.address(),
        attention_policy: HELM_D_R4_GAUGE_SOFTMAX_POLICY.to_owned(),
        attention_policy_cid,
        construction_manifest_cid,
        construction_documents,
        construction_positions,
        trace_bundle_cid,
        trace_bundle_bytes: trace_bundle.len(),
        document_trace_cids,
        artifact_cid,
        artifact_bytes: artifact_bytes.len(),
        artifact_construction_digest: format!(
            "blake3:{}",
            hex::encode(artifact.construction_digest())
        ),
        artifact_rows: artifact.row_count(),
        artifact_rows_by_depth,
        artifact_reload_bytes_exact,
        artifact_reload_cid_exact,
        construction_audits,
        source_execution: execution,
        held_out_teacher_scored: false,
        held_out_identity_bound_into_artifact: false,
        compile_seconds: started.elapsed().as_secs_f64(),
        freeze_cid: String::new(),
        nonclaims: vec![
            "No held-out teacher output, held-out text identity, or evaluation-preflight CID entered the artifact compiler."
                .to_owned(),
            "This suffix student is a geometry-free source-free baseline compiled from a geometry-qualified teacher; it is not yet geometric attention-state inference."
                .to_owned(),
            "This construction result does not establish coherent generation, reasoning, softmax removal, or deployed WASM readiness."
                .to_owned(),
        ],
    };
    freeze.freeze_cid =
        canonical_json_cid_omitting_fields(&freeze, &["compile_seconds", "freeze_cid"])?;
    write_json_atomic(&config.freeze_output, &freeze)?;
    Ok(freeze)
}

pub fn reveal_held_out(
    config: &R4SoftmaxTraceExperimentConfig,
) -> Result<R4SoftmaxTraceResult, R4SoftmaxTraceExperimentError> {
    let started = Instant::now();
    validate_distinct_output_paths(config)?;
    let evaluation_preflight = preflight(config)?;
    if !evaluation_preflight.proceed_to_source_trace {
        return Err(R4SoftmaxTraceExperimentError::InvalidRequest(format!(
            "evaluation preflight refused reveal: {}",
            evaluation_preflight.decision
        )));
    }
    let freeze_bytes = fs::read(&config.freeze_output)?;
    let freeze: R4SoftmaxTraceFreeze = serde_json::from_slice(&freeze_bytes)
        .map_err(|error| R4SoftmaxTraceExperimentError::Serialization(error.to_string()))?;
    validate_freeze(config, &freeze)?;

    let artifact_bytes_before = fs::read(&config.artifact_output)?;
    let runtime = R4SoftmaxTraceStudentRuntime::from_bytes(&artifact_bytes_before)
        .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?;
    let artifact_cid_before_reveal = runtime.artifact_cid();
    if artifact_cid_before_reveal != freeze.artifact_cid {
        return Err(R4SoftmaxTraceExperimentError::Student(format!(
            "artifact CID {} != frozen {}",
            artifact_cid_before_reveal, freeze.artifact_cid
        )));
    }

    let tokenizer = load_tokenizer(config)?;
    let held_out = tokenize_document(&tokenizer, HELD_OUT_DOCUMENT)?;
    if held_out.targets.len() != 57 || !d3_is_held_out(&held_out.id) {
        return Err(R4SoftmaxTraceExperimentError::InvalidRequest(
            "held-out document binding drifted after freeze".to_owned(),
        ));
    }
    let oracle = load_oracle(config, &tokenizer, held_out.targets.len())?;
    let held_out_outcome = trace_document(
        &oracle,
        &held_out,
        &tokenizer.address(),
        &freeze.attention_policy_cid,
        &evaluation_preflight.preflight_cid,
        "D3-held-out/13",
    )?;
    let source_execution_after_held_out_trace = oracle.execution_snapshot();
    if source_execution_after_held_out_trace.streams_completed != held_out.targets.len() as u64 {
        return Err(R4SoftmaxTraceExperimentError::Source(format!(
            "held-out execution completed {} streams, expected {}",
            source_execution_after_held_out_trace.streams_completed,
            held_out.targets.len()
        )));
    }
    let held_out_trace_cid = held_out_outcome
        .trace
        .trace_cid()
        .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
    let held_out_sequence = held_out_outcome.sequence;
    let held_out_audit = held_out_outcome.audit;
    let held_out_evaluation = runtime
        .evaluate(std::slice::from_ref(&held_out_sequence))
        .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?;
    let context_bearing_evaluation = evaluate_context_bearing(&runtime, &held_out_sequence)?;

    let prompt = "He was born";
    let prompt_token_ids = tokenizer.encode(prompt);
    let mut prompt_history = Vec::with_capacity(prompt_token_ids.len() + 1);
    prompt_history.push(PINNED_BOS_TOKEN_ID);
    prompt_history.extend_from_slice(&prompt_token_ids);
    let source_before_student = oracle.execution_snapshot();
    let generated_token_ids = source_free_continue(
        &runtime,
        &prompt_history,
        R4SoftmaxTraceStudentArm::TeacherDistilled,
        16,
    )?;
    let replay_runtime = R4SoftmaxTraceStudentRuntime::from_bytes(&artifact_bytes_before)
        .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?;
    let replay_token_ids = source_free_continue(
        &replay_runtime,
        &prompt_history,
        R4SoftmaxTraceStudentArm::TeacherDistilled,
        16,
    )?;
    let replay_evaluation = replay_runtime
        .evaluate(std::slice::from_ref(&held_out_sequence))
        .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?;
    let source_after_student = oracle.execution_snapshot();
    let source_execution_unchanged = source_before_student == source_after_student;
    let replay_exact = generated_token_ids == replay_token_ids
        && held_out_evaluation == replay_evaluation
        && runtime.artifact_cid() == replay_runtime.artifact_cid();
    let continuation = SourceFreeContinuation {
        prompt: prompt.to_owned(),
        prompt_token_ids,
        decoded_text: tokenizer.decode(&generated_token_ids),
        generated_token_ids,
        replay_token_ids,
        replay_exact,
        source_execution_unchanged,
    };

    let artifact_bytes_after = fs::read(&config.artifact_output)?;
    let artifact_cid_after_reveal = bytes_cid(&artifact_bytes_after);
    let artifact_unchanged_across_reveal = artifact_bytes_before == artifact_bytes_after
        && artifact_cid_before_reveal == artifact_cid_after_reveal;
    let distilled = &context_bearing_evaluation.teacher_distilled;
    let count = &context_bearing_evaluation.observed_count;
    let permuted = &context_bearing_evaluation.document_permuted_control;
    let distilled_beats_count_cross_entropy = lower_optional(
        distilled.covered_teacher_cross_entropy_nats,
        count.covered_teacher_cross_entropy_nats,
    );
    let distilled_beats_count_teacher_top1 =
        distilled.teacher_top1_agreements > count.teacher_top1_agreements;
    let distilled_actual_top1_not_worse =
        distilled.actual_next_top1_correct >= count.actual_next_top1_correct;
    let permuted_control_loses_cross_entropy = lower_optional(
        distilled.covered_teacher_cross_entropy_nats,
        permuted.covered_teacher_cross_entropy_nats,
    );
    let permuted_control_loses_teacher_top1 =
        distilled.teacher_top1_agreements > permuted.teacher_top1_agreements;
    let exact_replay = replay_exact && artifact_unchanged_across_reveal;
    let zero_runtime_source_forwards = source_execution_unchanged;
    let positive = distilled_beats_count_cross_entropy
        && distilled_beats_count_teacher_top1
        && distilled_actual_top1_not_worse
        && permuted_control_loses_cross_entropy
        && permuted_control_loses_teacher_top1
        && exact_replay
        && zero_runtime_source_forwards
        && held_out_audit.causal_audit_exact
        && held_out_audit.projection_audit_exact
        && held_out_audit.r4_audit_exact
        && held_out_audit.zero_future_reads;
    let mut result = R4SoftmaxTraceResult {
        schema: RESULT_SCHEMA.to_owned(),
        issue: 973,
        claim_scope: "held-out comparison of a frozen source-free suffix student against equal-support count and document-permuted controls"
            .to_owned(),
        implementation_revision: config.implementation_revision.clone(),
        evaluation_preflight,
        construction_freeze_cid: freeze.freeze_cid,
        artifact_cid_before_reveal,
        artifact_cid_after_reveal,
        artifact_unchanged_across_reveal,
        held_out_document: binding(&held_out, "held_out"),
        held_out_trace_cid,
        held_out_judge_sequence: held_out_sequence,
        held_out_audit,
        source_execution_after_held_out_trace,
        held_out_evaluation,
        context_bearing_evaluation,
        continuation,
        distilled_beats_count_cross_entropy,
        distilled_beats_count_teacher_top1,
        distilled_actual_top1_not_worse,
        permuted_control_loses_cross_entropy,
        permuted_control_loses_teacher_top1,
        exact_replay,
        zero_runtime_source_forwards,
        terminal: if positive {
            "PASS_SOURCE_FREE_TRACE_STUDENT_ADVANCE_GEOMETRIC_STATE_COMPILER".to_owned()
        } else {
            "STOP_SOURCE_FREE_TRACE_STUDENT_REPAIR_REPRESENTATION".to_owned()
        },
        reveal_seconds: started.elapsed().as_secs_f64(),
        result_cid: String::new(),
        nonclaims: vec![
            "A positive result establishes only bounded source-free trace distillation, not geometric advantage."
                .to_owned(),
            "The frozen corpus has only nine non-BOS context-bearing held-out positions; it cannot establish general generation or reasoning."
                .to_owned(),
            "Held-out document 13 plaintext and token targets were known to the tokenizer-only reachability preflight; only its teacher outputs were held back until after artifact freeze. This is construction-disjoint, teacher-output-held-back transfer, not blind unseen-text generalization."
                .to_owned(),
            "The student runtime is integer/table based, but no production R4G1, WASM, or multiplication-free release claim is made here."
                .to_owned(),
        ],
    };
    result.result_cid =
        canonical_json_cid_omitting_fields(&result, &["reveal_seconds", "result_cid"])?;
    write_json_atomic(&config.result_output, &result)?;
    Ok(result)
}

fn load_tokenizer(
    config: &R4SoftmaxTraceExperimentConfig,
) -> Result<HfBpeTokenizer, R4SoftmaxTraceExperimentError> {
    let tokenizer = HfBpeTokenizer::from_dir(&config.source)
        .map_err(|error| R4SoftmaxTraceExperimentError::Tokenizer(error.to_string()))?;
    let tokenizer_cid = tokenizer.address();
    if tokenizer_cid != PINNED_TOKENIZER_CID {
        return Err(R4SoftmaxTraceExperimentError::Tokenizer(format!(
            "tokenizer CID {tokenizer_cid} != pinned {PINNED_TOKENIZER_CID}"
        )));
    }
    Ok(tokenizer)
}

fn load_oracle(
    config: &R4SoftmaxTraceExperimentConfig,
    tokenizer: &HfBpeTokenizer,
    sequence_capacity: usize,
) -> Result<HuggingFaceLlamaOracle, R4SoftmaxTraceExperimentError> {
    validate_source(&config.source, &config.source_revision)
        .map_err(|error| R4SoftmaxTraceExperimentError::Source(error.to_string()))?;
    let oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
        &config.source,
        sequence_capacity,
        TeacherExecutionConfig::fixed_workers(config.workers),
    )
    .map_err(|error| R4SoftmaxTraceExperimentError::Source(error.to_string()))?;
    if oracle.source_cid() != PINNED_SOURCE_CID
        || oracle.cfg().vocab != tokenizer.vocab_size()
        || oracle.cfg().seq_len != sequence_capacity
    {
        return Err(R4SoftmaxTraceExperimentError::Source(format!(
            "pinned source binding mismatch: cid={}, vocab={}/{}, capacity={}/{}",
            oracle.source_cid(),
            oracle.cfg().vocab,
            tokenizer.vocab_size(),
            oracle.cfg().seq_len,
            sequence_capacity
        )));
    }
    let bos = u32::try_from(TeacherOracle::bos_token(&oracle)).map_err(|_| {
        R4SoftmaxTraceExperimentError::Source(
            "source BOS token exceeds the u32 namespace".to_owned(),
        )
    })?;
    let eos = u32::try_from(TeacherOracle::eos_token(&oracle)).map_err(|_| {
        R4SoftmaxTraceExperimentError::Source(
            "source EOS token exceeds the u32 namespace".to_owned(),
        )
    })?;
    if bos != PINNED_BOS_TOKEN_ID || eos != PINNED_EOS_TOKEN_ID {
        return Err(R4SoftmaxTraceExperimentError::Source(format!(
            "source BOS/EOS ({bos},{eos}) != pinned ({PINNED_BOS_TOKEN_ID},{PINNED_EOS_TOKEN_ID})"
        )));
    }
    Ok(oracle)
}

fn construction_manifest_cid(
    config: &R4SoftmaxTraceExperimentConfig,
    tokenizer_cid: &str,
    attention_policy_cid: &str,
    construction: &[TokenizedDocument],
) -> Result<String, R4SoftmaxTraceExperimentError> {
    let documents = construction
        .iter()
        .map(|document| ConstructionDocumentIdentity {
            id: &document.id,
            text_cid: text_cid(&document.text),
            input_tokens: &document.inputs,
            actual_next_tokens: &document.targets,
        })
        .collect::<Vec<_>>();
    let identity = ConstructionManifestIdentity {
        schema: COMPILER_SCHEMA,
        source_revision: &config.source_revision,
        source_cid: PINNED_SOURCE_CID,
        tokenizer_cid,
        attention_policy_cid,
        maximum_suffix_depth: MAX_SUFFIX_DEPTH,
        attention_support: ATTENTION_SUPPORT,
        logit_support: LOGIT_SUPPORT,
        documents,
    };
    canonical_json_cid(&identity)
}

fn trace_document(
    oracle: &HuggingFaceLlamaOracle,
    document: &TokenizedDocument,
    tokenizer_cid: &str,
    attention_policy_cid: &str,
    corpus_cid: &str,
    partition_id: &str,
) -> Result<TraceDocumentOutcome, R4SoftmaxTraceExperimentError> {
    let positions = document.targets.len();
    let shape = model_shape(oracle, positions)
        .map_err(|error| R4SoftmaxTraceExperimentError::Source(error.to_string()))?;
    let inner = R4SpinCausalAttentionTransport::new(
        u32::try_from(shape.vocabulary - 1).map_err(|_| {
            R4SoftmaxTraceExperimentError::Source(
                "source vocabulary exceeds the u32 namespace".to_owned(),
            )
        })?,
        positions,
        R4SpinTransportIntervention::Coherent,
    )
    .map_err(|error| R4SoftmaxTraceExperimentError::Attention(error.to_string()))?;
    let identity = R4SoftmaxTeacherTraceIdentity {
        source_cid: oracle.source_cid().to_owned(),
        tokenizer_cid: tokenizer_cid.to_owned(),
        attention_policy_cid: attention_policy_cid.to_owned(),
        corpus_cid: corpus_cid.to_owned(),
        construction_partition_id: partition_id.to_owned(),
        document_id: document.id.clone(),
        document_text_cid: text_cid(&document.text),
    };
    let bounds = R4SoftmaxTeacherTraceBounds {
        maximum_positions: positions,
        layers: shape.layers,
        query_heads: shape.query_heads,
        key_value_heads: shape.key_value_heads,
        head_size: shape.head_size,
        vocabulary: shape.vocabulary,
    };
    let (transport, handle) = TracingR4SpinTransport::new(inner, identity, bounds)
        .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
    let mut session = oracle
        .new_causal_attention_transport_session(
            Box::new(transport),
            CausalAttentionLayerSelection::All,
            positions,
        )
        .map_err(|error| R4SoftmaxTraceExperimentError::Attention(error.to_string()))?;
    if session.selected_layer_count() != shape.layers
        || (0..shape.layers).any(|layer| !session.layer_is_selected(layer))
    {
        return Err(R4SoftmaxTraceExperimentError::Attention(
            "trace session did not select every decoder layer".to_owned(),
        ));
    }
    let mut logits = vec![0.0_f32; shape.vocabulary];
    for (position, (&input, &target)) in document.inputs.iter().zip(&document.targets).enumerate() {
        oracle
            .step_causal_attention_transport(&mut session, input as usize, position, &mut logits)
            .map_err(|error| R4SoftmaxTraceExperimentError::Attention(error.to_string()))?;
        handle
            .complete_position(position, target, &logits)
            .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
    }
    handle
        .status()
        .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
    session
        .transport_status()
        .map_err(R4SoftmaxTraceExperimentError::Attention)?;
    let audit = audit_session(&session, &shape, positions)?;
    let trace = handle
        .snapshot()
        .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
    let teacher_top_distributions = trace
        .positions
        .iter()
        .map(|position| teacher_distribution(&position.logits))
        .collect::<Result<Vec<_>, _>>()?;
    let sequence = R4SoftmaxTraceSequence::new(
        document.id.clone(),
        document.inputs.clone(),
        document.targets.clone(),
        teacher_top_distributions,
    );
    Ok(TraceDocumentOutcome {
        trace,
        sequence,
        audit,
    })
}

fn audit_session(
    session: &CausalAttentionTransportSession,
    shape: &ModelShape,
    positions: usize,
) -> Result<AttentionAuditEvidence, R4SoftmaxTraceExperimentError> {
    let implementation_json = session
        .transport_implementation_evidence()
        .map_err(R4SoftmaxTraceExperimentError::Attention)?
        .ok_or_else(|| {
            R4SoftmaxTraceExperimentError::Attention(
                "R4 trace transport emitted no implementation evidence".to_owned(),
            )
        })?;
    let implementation: R4SpinTransportEvidence = serde_json::from_str(&implementation_json)
        .map_err(|error| R4SoftmaxTraceExperimentError::Serialization(error.to_string()))?;
    let observed_causal = CausalAttentionAuditRecord::from(session.audit());
    let expected_causal = expected_causal_audit(positions, shape)
        .map_err(|error| R4SoftmaxTraceExperimentError::Attention(error.to_string()))?;
    let observed_projection = ProjectionAuditRecord::from(session.pre_rope_projection_audit());
    let expected_projection = expected_projection_audit(positions, shape)
        .map_err(|error| R4SoftmaxTraceExperimentError::Attention(error.to_string()))?;
    let expected_r4 = expected_r4_audit(positions, shape)
        .map_err(|error| R4SoftmaxTraceExperimentError::Attention(error.to_string()))?;
    let causal_audit_exact = observed_causal == expected_causal;
    let projection_audit_exact = observed_projection == expected_projection;
    let r4_audit_exact = implementation.policy_identity == HELM_D_R4_GAUGE_SOFTMAX_POLICY
        && implementation.intervention == R4SpinTransportIntervention::Coherent
        && implementation.frame_table_offsets.len() == positions
        && implementation.audit == expected_r4;
    let zero_future_reads =
        observed_causal.future_reads == 0 && implementation.audit.future_position_reads == 0;
    let all_layers_selected = session.selected_layer_count() == shape.layers
        && (0..shape.layers).all(|layer| session.layer_is_selected(layer));
    if !(causal_audit_exact
        && projection_audit_exact
        && r4_audit_exact
        && zero_future_reads
        && all_layers_selected)
    {
        return Err(R4SoftmaxTraceExperimentError::Attention(format!(
            "trace audit mismatch: causal={causal_audit_exact}, projection={projection_audit_exact}, R4={r4_audit_exact}, zero_future={zero_future_reads}, all_layers={all_layers_selected}"
        )));
    }
    Ok(AttentionAuditEvidence {
        selected_layer_count: session.selected_layer_count(),
        positions_executed: positions,
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
    })
}

pub(crate) fn teacher_distribution(
    logits: &crate::r4_softmax_teacher_trace::R4SoftmaxLogitTrace,
) -> Result<TeacherTopDistributionQ16, R4SoftmaxTraceExperimentError> {
    if logits.top_logits.len() != LOGIT_SUPPORT {
        return Err(R4SoftmaxTraceExperimentError::Trace(format!(
            "teacher logit support {} != frozen {LOGIT_SUPPORT}",
            logits.top_logits.len()
        )));
    }
    let maximum = logits
        .top_logits
        .iter()
        .map(|entry| f64::from(entry.logit()))
        .fold(f64::NEG_INFINITY, f64::max);
    let raw = logits
        .top_logits
        .iter()
        .map(|entry| (f64::from(entry.logit()) - maximum).exp())
        .collect::<Vec<_>>();
    let total = raw.iter().sum::<f64>();
    if !total.is_finite() || total <= 0.0 {
        return Err(R4SoftmaxTraceExperimentError::Trace(
            "teacher top-logit normalization is non-finite".to_owned(),
        ));
    }
    let floor_total = u32::try_from(raw.len()).map_err(|_| {
        R4SoftmaxTraceExperimentError::Trace("teacher support exceeds u32".to_owned())
    })?;
    let distributable = u32::from(R4_SOFTMAX_TRACE_Q16_TOTAL)
        .checked_sub(floor_total)
        .ok_or_else(|| {
            R4SoftmaxTraceExperimentError::Trace(
                "teacher support exceeds Q16 normalization".to_owned(),
            )
        })?;
    let mut weights = Vec::with_capacity(raw.len());
    let mut remainders = Vec::with_capacity(raw.len());
    let mut assigned = floor_total;
    for (index, (&value, ranked)) in raw.iter().zip(&logits.top_logits).enumerate() {
        let scaled = value / total * f64::from(distributable);
        let floor = scaled.floor() as u32;
        assigned = assigned.checked_add(floor).ok_or_else(|| {
            R4SoftmaxTraceExperimentError::Trace("teacher Q16 sum overflowed".to_owned())
        })?;
        weights.push(floor + 1);
        remainders.push((scaled - f64::from(floor), ranked.token, index));
    }
    let remaining = u32::from(R4_SOFTMAX_TRACE_Q16_TOTAL)
        .checked_sub(assigned)
        .ok_or_else(|| {
            R4SoftmaxTraceExperimentError::Trace("teacher Q16 sum exceeded total".to_owned())
        })?;
    remainders.sort_by(|left, right| {
        right
            .0
            .total_cmp(&left.0)
            .then_with(|| left.1.cmp(&right.1))
    });
    if remaining as usize > remainders.len() {
        return Err(R4SoftmaxTraceExperimentError::Trace(
            "teacher Q16 remainder exceeded support".to_owned(),
        ));
    }
    for &(_, _, index) in remainders.iter().take(remaining as usize) {
        weights[index] += 1;
    }
    let entries = logits
        .top_logits
        .iter()
        .zip(weights)
        .map(|(ranked, weight)| {
            Ok(TeacherTopTokenQ16::new(
                ranked.token,
                u16::try_from(weight).map_err(|_| {
                    R4SoftmaxTraceExperimentError::Trace(
                        "teacher Q16 weight exceeds u16".to_owned(),
                    )
                })?,
            ))
        })
        .collect::<Result<Vec<_>, R4SoftmaxTraceExperimentError>>()?;
    TeacherTopDistributionQ16::new(entries)
        .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))
}

#[derive(Default)]
struct ContextAccumulator {
    positions: u64,
    teacher_mass_covered_q16: u64,
    teacher_top1_agreements: u64,
    actual_next_top1_correct: u64,
    cross_entropy: f64,
}

impl ContextAccumulator {
    fn record(
        &mut self,
        student: &R4SoftmaxTraceStudentDistribution,
        teacher: &TeacherTopDistributionQ16,
        actual: u32,
    ) -> Result<(), R4SoftmaxTraceExperimentError> {
        self.positions += 1;
        let prediction = student
            .scores
            .iter()
            .max_by(|left, right| {
                left.weight_q16
                    .cmp(&right.weight_q16)
                    .then_with(|| right.token.cmp(&left.token))
            })
            .ok_or_else(|| {
                R4SoftmaxTraceExperimentError::Student(
                    "context-bearing student row is empty".to_owned(),
                )
            })?;
        if prediction.token
            == teacher
                .top_token()
                .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?
        {
            self.teacher_top1_agreements += 1;
        }
        if prediction.token == actual {
            self.actual_next_top1_correct += 1;
        }
        for entry in &teacher.entries {
            let Some(score) = student
                .scores
                .iter()
                .find(|score| score.token == entry.token)
            else {
                continue;
            };
            self.teacher_mass_covered_q16 += u64::from(entry.probability_q16);
            let teacher_probability =
                f64::from(entry.probability_q16) / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL);
            let student_probability =
                f64::from(score.weight_q16) / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL);
            self.cross_entropy -= teacher_probability * student_probability.ln();
        }
        Ok(())
    }

    fn finish(self) -> ContextArmEvaluation {
        let covered_teacher_cross_entropy_nats = (self.teacher_mass_covered_q16 != 0).then_some(
            self.cross_entropy
                / (self.teacher_mass_covered_q16 as f64 / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL)),
        );
        ContextArmEvaluation {
            positions: self.positions,
            teacher_mass_covered_q16: self.teacher_mass_covered_q16,
            teacher_top1_agreements: self.teacher_top1_agreements,
            actual_next_top1_correct: self.actual_next_top1_correct,
            covered_teacher_cross_entropy_nats,
        }
    }
}

fn evaluate_context_bearing(
    runtime: &R4SoftmaxTraceStudentRuntime,
    sequence: &R4SoftmaxTraceSequence,
) -> Result<ContextBearingEvaluation, R4SoftmaxTraceExperimentError> {
    let mut distilled = ContextAccumulator::default();
    let mut count = ContextAccumulator::default();
    let mut permuted = ContextAccumulator::default();
    let mut excluded_shared_bos_positions = 0_u64;
    for position in 0..sequence.input_tokens.len() {
        let history = &sequence.input_tokens[..=position];
        let teacher = &sequence.teacher_top_distributions[position];
        let actual = sequence.actual_next_tokens[position];
        let distilled_row = runtime
            .distribution(history, R4SoftmaxTraceStudentArm::TeacherDistilled)
            .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?;
        let count_row = runtime
            .distribution(history, R4SoftmaxTraceStudentArm::ObservedCount)
            .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?;
        let permuted_row = runtime
            .distribution(history, R4SoftmaxTraceStudentArm::DocumentPermutedControl)
            .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?;
        if distilled_row.suffix_depth != count_row.suffix_depth
            || distilled_row.suffix_depth != permuted_row.suffix_depth
        {
            return Err(R4SoftmaxTraceExperimentError::Student(
                "matched arms resolved different suffix depths".to_owned(),
            ));
        }
        if distilled_row.suffix_depth == 0 {
            continue;
        }
        if position == 0 && sequence.input_tokens[position] == PINNED_BOS_TOKEN_ID {
            excluded_shared_bos_positions += 1;
            continue;
        }
        distilled.record(&distilled_row, teacher, actual)?;
        count.record(&count_row, teacher, actual)?;
        permuted.record(&permuted_row, teacher, actual)?;
    }
    let evaluation = ContextBearingEvaluation {
        excluded_shared_bos_positions,
        teacher_distilled: distilled.finish(),
        observed_count: count.finish(),
        document_permuted_control: permuted.finish(),
    };
    if evaluation.excluded_shared_bos_positions != 1
        || evaluation.teacher_distilled.positions != 9
        || evaluation.observed_count.positions != 9
        || evaluation.document_permuted_control.positions != 9
    {
        return Err(R4SoftmaxTraceExperimentError::Student(format!(
            "context-bearing evaluation drifted: BOS={}, distilled={}, count={}, permuted={}",
            evaluation.excluded_shared_bos_positions,
            evaluation.teacher_distilled.positions,
            evaluation.observed_count.positions,
            evaluation.document_permuted_control.positions
        )));
    }
    Ok(evaluation)
}

fn source_free_continue(
    runtime: &R4SoftmaxTraceStudentRuntime,
    history: &[u32],
    arm: R4SoftmaxTraceStudentArm,
    maximum_new_tokens: usize,
) -> Result<Vec<u32>, R4SoftmaxTraceExperimentError> {
    let mut context = history.to_vec();
    let mut generated = Vec::with_capacity(maximum_new_tokens);
    for _ in 0..maximum_new_tokens {
        let token = runtime
            .predict(&context, arm)
            .map_err(|error| R4SoftmaxTraceExperimentError::Student(error.to_string()))?
            .token;
        generated.push(token);
        if token == PINNED_EOS_TOKEN_ID {
            break;
        }
        context.push(token);
    }
    Ok(generated)
}

fn validate_freeze(
    config: &R4SoftmaxTraceExperimentConfig,
    freeze: &R4SoftmaxTraceFreeze,
) -> Result<(), R4SoftmaxTraceExperimentError> {
    validate_implementation_revision(&config.implementation_revision)?;
    let recorded_cid = &freeze.freeze_cid;
    let computed_cid =
        canonical_json_cid_omitting_fields(freeze, &["compile_seconds", "freeze_cid"])?;
    let tokenizer = load_tokenizer(config)?;
    let construction = CONSTRUCTION_DOCUMENTS
        .iter()
        .map(|document| tokenize_document(&tokenizer, *document))
        .collect::<Result<Vec<_>, _>>()?;
    let expected_documents = construction
        .iter()
        .map(|document| binding(document, "construction"))
        .collect::<Vec<_>>();
    let expected_manifest_cid =
        construction_manifest_cid(config, &tokenizer.address(), &policy_cid(), &construction)?;
    let audit_census_exact = freeze.construction_audits.len() == expected_documents.len()
        && freeze
            .construction_audits
            .iter()
            .zip(&expected_documents)
            .all(|(audit, document)| audit.positions_executed == document.target_tokens);
    if freeze.schema != FREEZE_SCHEMA
        || recorded_cid != &computed_cid
        || freeze.implementation_revision != config.implementation_revision
        || freeze.source_revision != config.source_revision
        || freeze.source_cid != PINNED_SOURCE_CID
        || freeze.tokenizer_cid != PINNED_TOKENIZER_CID
        || freeze.attention_policy != HELM_D_R4_GAUGE_SOFTMAX_POLICY
        || freeze.attention_policy_cid != policy_cid()
        || freeze.construction_manifest_cid != expected_manifest_cid
        || freeze.construction_positions != 38
        || freeze.construction_documents != expected_documents
        || freeze.document_trace_cids.len() != CONSTRUCTION_DOCUMENTS.len()
        || freeze
            .document_trace_cids
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != CONSTRUCTION_DOCUMENTS.len()
        || !audit_census_exact
        || freeze.held_out_teacher_scored
        || freeze.held_out_identity_bound_into_artifact
        || !freeze.artifact_reload_bytes_exact
        || !freeze.artifact_reload_cid_exact
        || freeze.construction_audits.iter().any(|audit| {
            !(audit.causal_audit_exact
                && audit.projection_audit_exact
                && audit.r4_audit_exact
                && audit.zero_future_reads
                && audit.all_layers_selected)
        })
    {
        return Err(R4SoftmaxTraceExperimentError::InvalidRequest(
            "construction freeze identity or audit is invalid".to_owned(),
        ));
    }
    if freeze
        .construction_documents
        .iter()
        .any(|document| document.partition != "construction" || d3_is_held_out(&document.id))
    {
        return Err(R4SoftmaxTraceExperimentError::InvalidRequest(
            "construction freeze contains a held-out document binding".to_owned(),
        ));
    }
    let trace_bundle = fs::read(&config.trace_output)?;
    if trace_bundle.len() != freeze.trace_bundle_bytes {
        return Err(R4SoftmaxTraceExperimentError::Trace(
            "construction trace bundle byte length changed after freeze".to_owned(),
        ));
    }
    let typed_trace_bundle = R4SoftmaxTeacherTraceBundle::from_bytes_with_expected_cids(
        &trace_bundle,
        &freeze.trace_bundle_cid,
        &freeze.document_trace_cids,
    )
    .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
    validate_loaded_trace_bundle(
        &typed_trace_bundle,
        &construction,
        tokenizer.vocab_size(),
        &freeze.construction_manifest_cid,
        &freeze.construction_audits,
    )?;
    Ok(())
}

fn validate_loaded_trace_bundle(
    bundle: &R4SoftmaxTeacherTraceBundle,
    construction: &[TokenizedDocument],
    vocabulary: usize,
    construction_manifest_cid: &str,
    construction_audits: &[AttentionAuditEvidence],
) -> Result<(), R4SoftmaxTraceExperimentError> {
    if bundle.traces().len() != construction.len()
        || construction_audits.len() != construction.len()
    {
        return Err(R4SoftmaxTraceExperimentError::Trace(
            "construction trace bundle census does not match the frozen manifest".to_owned(),
        ));
    }
    let maximum_token_id = u32::try_from(vocabulary.checked_sub(1).ok_or_else(|| {
        R4SoftmaxTraceExperimentError::Trace(
            "construction tokenizer vocabulary is empty".to_owned(),
        )
    })?)
    .map_err(|_| {
        R4SoftmaxTraceExperimentError::Trace(
            "construction tokenizer vocabulary exceeds the u32 namespace".to_owned(),
        )
    })?;
    let maximum_positions = construction
        .iter()
        .map(|document| document.targets.len())
        .max()
        .ok_or_else(|| {
            R4SoftmaxTraceExperimentError::Trace("construction trace manifest is empty".to_owned())
        })?;

    for ((trace, document), audit) in bundle
        .traces()
        .iter()
        .zip(construction)
        .zip(construction_audits)
    {
        let expected_text_cid = text_cid(&document.text);
        if trace.identity.source_cid != PINNED_SOURCE_CID
            || trace.identity.tokenizer_cid != PINNED_TOKENIZER_CID
            || trace.identity.attention_policy_cid != policy_cid()
            || trace.identity.corpus_cid != construction_manifest_cid
            || trace.identity.construction_partition_id != "D3-construction/14-657-4579-5121"
            || trace.identity.document_id != document.id
            || trace.identity.document_text_cid != expected_text_cid
            || trace.bounds.maximum_positions != maximum_positions
            || trace.bounds.vocabulary != vocabulary
            || trace.bounds.layers != audit.selected_layer_count
            || trace.positions.len() != document.targets.len()
        {
            return Err(R4SoftmaxTraceExperimentError::Trace(format!(
                "trace document {} does not match its frozen manifest identity or shape",
                document.id
            )));
        }

        let mut frame_oracle = R4SpinCausalAttentionTransport::new(
            maximum_token_id,
            document.inputs.len(),
            R4SpinTransportIntervention::Coherent,
        )
        .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
        let mut trace_frame_offsets = Vec::with_capacity(document.inputs.len());
        for (position, ((trace_position, &input_token), &target_token)) in trace
            .positions
            .iter()
            .zip(&document.inputs)
            .zip(&document.targets)
            .enumerate()
        {
            if trace_position.position as usize != position
                || trace_position.input_token != input_token
                || trace_position.logits.target_token != target_token
            {
                return Err(R4SoftmaxTraceExperimentError::Trace(format!(
                    "trace document {} token row {position} does not match the frozen manifest",
                    document.id
                )));
            }
            CausalAttentionTransport::begin_position(
                &mut frame_oracle,
                input_token as usize,
                position,
            );
            CausalAttentionTransport::status(&frame_oracle)
                .map_err(R4SoftmaxTraceExperimentError::Trace)?;
            let expected_frame = frame_oracle
                .frame_table_offset(position)
                .map_err(|error| R4SoftmaxTraceExperimentError::Trace(error.to_string()))?;
            if trace_position.frame_table_offset != expected_frame {
                return Err(R4SoftmaxTraceExperimentError::Trace(format!(
                    "trace document {} frame row {position} is {}, expected {expected_frame}",
                    document.id, trace_position.frame_table_offset
                )));
            }
            trace_frame_offsets.push(trace_position.frame_table_offset);
        }
        if trace_frame_offsets != audit.r4_implementation.frame_table_offsets {
            return Err(R4SoftmaxTraceExperimentError::Trace(format!(
                "trace document {} frame addresses do not match the frozen transport audit",
                document.id
            )));
        }
    }
    Ok(())
}

fn validate_implementation_revision(revision: &str) -> Result<(), R4SoftmaxTraceExperimentError> {
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(R4SoftmaxTraceExperimentError::InvalidRequest(
            "--implementation-revision must be an exact 40-character lowercase Git commit"
                .to_owned(),
        ));
    }
    Ok(())
}

fn validate_distinct_output_paths(
    config: &R4SoftmaxTraceExperimentConfig,
) -> Result<(), R4SoftmaxTraceExperimentError> {
    let paths = [
        &config.artifact_output,
        &config.trace_output,
        &config.freeze_output,
        &config.result_output,
    ];
    let mut identities = BTreeSet::new();
    for path in paths {
        let identity = output_path_identity(path)?;
        if !identities.insert(identity) {
            return Err(R4SoftmaxTraceExperimentError::InvalidRequest(
                "artifact, trace, freeze, and result outputs must be distinct".to_owned(),
            ));
        }
    }
    Ok(())
}

fn output_path_identity(path: &Path) -> Result<PathBuf, R4SoftmaxTraceExperimentError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    if absolute.exists() {
        return Ok(fs::canonicalize(absolute)?);
    }
    let parent = absolute.parent().ok_or_else(|| {
        R4SoftmaxTraceExperimentError::InvalidRequest(format!(
            "output {} has no parent directory",
            path.display()
        ))
    })?;
    let file_name = absolute.file_name().ok_or_else(|| {
        R4SoftmaxTraceExperimentError::InvalidRequest(format!(
            "output {} has no file name",
            path.display()
        ))
    })?;
    let parent = fs::canonicalize(parent).unwrap_or_else(|_| normalize_path(parent));
    Ok(parent.join(file_name))
}

fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn lower_optional(left: Option<f64>, right: Option<f64>) -> bool {
    matches!((left, right), (Some(left), Some(right)) if left < right)
}

fn policy_cid() -> String {
    bytes_cid(HELM_D_R4_GAUGE_SOFTMAX_POLICY.as_bytes())
}

fn bytes_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn canonical_json_cid<T: Serialize>(value: &T) -> Result<String, R4SoftmaxTraceExperimentError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| R4SoftmaxTraceExperimentError::Serialization(error.to_string()))?;
    Ok(bytes_cid(&bytes))
}

/// Hash a canonical JSON object after removing non-identity metadata and its
/// self-CID. This keeps evidence identities stable across identical reruns.
fn canonical_json_cid_omitting_fields<T: Serialize>(
    value: &T,
    omitted_fields: &[&str],
) -> Result<String, R4SoftmaxTraceExperimentError> {
    let mut value = serde_json::to_value(value)
        .map_err(|error| R4SoftmaxTraceExperimentError::Serialization(error.to_string()))?;
    let object = value.as_object_mut().ok_or_else(|| {
        R4SoftmaxTraceExperimentError::Serialization(
            "canonical identity must serialize as a JSON object".to_owned(),
        )
    })?;
    for field in omitted_fields {
        if object.remove(*field).is_none() {
            return Err(R4SoftmaxTraceExperimentError::Serialization(format!(
                "canonical identity field {field} is absent"
            )));
        }
    }
    canonical_json_cid(&value)
}

fn write_json_atomic<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), R4SoftmaxTraceExperimentError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| R4SoftmaxTraceExperimentError::Serialization(error.to_string()))?;
    write_atomic(path, &bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), R4SoftmaxTraceExperimentError> {
    let parent = path.parent().ok_or_else(|| {
        R4SoftmaxTraceExperimentError::InvalidRequest(format!(
            "output {} has no parent directory",
            path.display()
        ))
    })?;
    fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            R4SoftmaxTraceExperimentError::InvalidRequest(format!(
                "output {} has no UTF-8 file name",
                path.display()
            ))
        })?;
    let temporary = parent.join(format!(".{file_name}.tmp"));
    fs::write(&temporary, bytes)?;
    fs::rename(&temporary, path)?;
    Ok(())
}

fn tokenize_document(
    tokenizer: &HfBpeTokenizer,
    document: FrozenDocument,
) -> Result<TokenizedDocument, R4SoftmaxTraceExperimentError> {
    let targets = tokenizer.encode(document.text);
    if targets.is_empty() {
        return Err(R4SoftmaxTraceExperimentError::Tokenizer(format!(
            "document {} encoded to zero tokens",
            document.id
        )));
    }
    let mut inputs = Vec::with_capacity(targets.len());
    inputs.push(PINNED_BOS_TOKEN_ID);
    inputs.extend(targets.iter().copied().take(targets.len() - 1));
    Ok(TokenizedDocument {
        id: document.id.to_owned(),
        text: document.text.to_owned(),
        targets,
        inputs,
    })
}

fn binding(document: &TokenizedDocument, partition: &str) -> FrozenDocumentBinding {
    FrozenDocumentBinding {
        id: document.id.clone(),
        partition: partition.to_owned(),
        text_cid: text_cid(&document.text),
        target_tokens: document.targets.len(),
    }
}

fn text_cid(text: &str) -> String {
    format!("blake3:{}", blake3::hash(text.as_bytes()).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_partition_is_disjoint_and_canonical() {
        assert!(CONSTRUCTION_DOCUMENTS
            .iter()
            .all(|document| !d3_is_held_out(document.id)));
        assert!(d3_is_held_out(HELD_OUT_DOCUMENT.id));
        let construction = CONSTRUCTION_DOCUMENTS
            .iter()
            .map(|document| text_cid(document.text))
            .collect::<BTreeSet<_>>();
        assert!(!construction.contains(&text_cid(HELD_OUT_DOCUMENT.text)));
    }

    #[test]
    fn evidence_identity_excludes_elapsed_time_and_self_cid() {
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
            elapsed: 999.0,
            cid: "second",
        };
        assert_eq!(
            canonical_json_cid_omitting_fields(&first, &["elapsed", "cid"])
                .expect("first identity"),
            canonical_json_cid_omitting_fields(&second, &["elapsed", "cid"])
                .expect("second identity")
        );
    }

    #[test]
    fn output_paths_must_be_pairwise_distinct_before_writes() {
        let shared = PathBuf::from("/tmp/r4-softmax-trace-shared-output");
        let config = R4SoftmaxTraceExperimentConfig {
            source: PathBuf::from("unused"),
            source_revision: PINNED_SOURCE_REVISION.to_owned(),
            implementation_revision: "0123456789abcdef0123456789abcdef01234567".to_owned(),
            workers: NonZeroUsize::new(1).expect("nonzero"),
            artifact_output: shared.clone(),
            trace_output: shared,
            freeze_output: PathBuf::from("/tmp/r4-softmax-trace-freeze"),
            result_output: PathBuf::from("/tmp/r4-softmax-trace-result"),
        };
        assert!(validate_distinct_output_paths(&config).is_err());
    }
}
