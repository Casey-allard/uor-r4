//! Bounded product-Hyperbolic-4 full-decoder decision harness for issue #973.
//!
//! The first ignored test freezes a deterministic, document-disjoint
//! SimpleWiki partition without running a model. The second ignored test
//! consumes only that frozen manifest, fits curved and flat attention
//! operators from construction traces, applies the construction-validation
//! gate, and opens the D3-held-out token sequences only after that gate passes.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use uor_r4_core::helm_d_r4_attention::{
    helm_d_lorentz_causal_row, intrinsic_r4_score_feature, intrinsic_r4_weighted_centroid,
    intrinsic_stable_softmax_into, HelmDLorentzReferenceConfig,
    IntrinsicLorentzR4AttentionParameters, IntrinsicR4AttentionEvidence,
    IntrinsicR4AttentionIntervention, IntrinsicR4AttentionMetric,
    IntrinsicR4CausalAttentionTransport, R4SpinCausalAttentionTransport, R4SpinFrameAtlas,
    R4SpinTransportAudit, R4SpinTransportEvidence, R4SpinTransportIntervention,
    HELM_D_R4_GAUGE_SOFTMAX_POLICY, HELM_D_UPSTREAM_COMMIT, INTRINSIC_FLAT_R4_ATTENTION_POLICY,
    INTRINSIC_LORENTZ_R4_ATTENTION_POLICY,
};
use uor_r4_core::source_free_table::d3_is_held_out;
use uor_r4_core::transformerless::scenarios::Tokenizer;
use uor_r4_model_source::attention::{
    CausalAttentionLayerSelection, CausalAttentionTransportAudit,
};
use uor_r4_model_source::{
    BehaviorSource, HuggingFaceLlamaOracle, TeacherExecutionConfig, TeacherExecutionPreparation,
    TeacherExecutionSnapshot, TeacherOracle, TraceCaptureRequest, TraceCaptureSinks,
};

const MODEL_ENV: &str = "UOR_R4_973_INTRINSIC_MODEL";
const TOKENIZER_ENV: &str = "UOR_R4_973_INTRINSIC_TOKENIZER";
const CORPUS_ENV: &str = "UOR_R4_973_INTRINSIC_CORPUS";
const MANIFEST_OUTPUT_ENV: &str = "UOR_R4_973_INTRINSIC_MANIFEST_OUTPUT";
const MANIFEST_ENV: &str = "UOR_R4_973_INTRINSIC_MANIFEST";
const RESULT_OUTPUT_ENV: &str = "UOR_R4_973_INTRINSIC_OUTPUT";
const WORKERS_ENV: &str = "UOR_R4_973_INTRINSIC_WORKERS";
const CANONICAL_DETERMINISTIC_ENV: &str = "TLESS_CANONICAL_DETERMINISTIC";
const IMPLEMENTATION_REVISION_ENV: &str = "UOR_R4_973_INTRINSIC_IMPLEMENTATION_REVISION";
const COMPILED_IMPLEMENTATION_REVISION: Option<&str> =
    option_env!("UOR_R4_973_INTRINSIC_IMPLEMENTATION_REVISION");

const COMPILED_CORE_SOURCE: &[u8] = include_bytes!("../src/helm_d_r4_attention.rs");
const COMPILED_HARNESS_SOURCE: &[u8] = include_bytes!("intrinsic_lorentz_r4_attention_973.rs");
const COMPILED_MODEL_ATTENTION_SOURCE: &[u8] =
    include_bytes!("../../uor-r4-model-source/src/attention.rs");
const COMPILED_MODEL_SOURCE: &[u8] = include_bytes!("../../uor-r4-model-source/src/lib.rs");
const COMPILED_EXACT_EXECUTOR_SOURCE: &[u8] =
    include_bytes!("../../uor-r4-model-source/src/exact_executor.rs");
const COMPILED_CONTRACT: &[u8] =
    include_bytes!("../../../docs/intrinsic_lorentz_r4_attention_manifest_973.json");
const COMPILED_PARTITION: &[u8] =
    include_bytes!("../../../docs/intrinsic_lorentz_r4_attention_partition_973.json");

const DEFAULT_MODEL: &str = "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct";
const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/compiled/smollm2-135m-instruct/tokenizer.bin";
const DEFAULT_CORPUS: &str =
    "/Users/casey.allard/uor-r4/.uor-models/corpora/simple-wiki-20231101/articles.jsonl";
const CANONICAL_EVIDENCE_ROOT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/research/issue-973-intrinsic-lorentz-r4";

const CORPUS_CID: &str = "blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf";
const DONOR_CID: &str = "blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5";
const CORPUS_DOCUMENTS: usize = 3_000;
const EXCLUDED_HELDOUT_ID: &str = "12";

const FIT_DOCUMENTS: usize = 16;
const VALIDATION_DOCUMENTS: usize = 4;
const HELDOUT_DOCUMENTS: usize = 8;
const REQUIRED_TOKENS: usize = 17;
const INPUT_POSITIONS: usize = 16;
const SCORE_START: usize = 8;
const SCORE_POSITIONS: usize = 8;
const GENERATED_TOKENS: usize = 8;
const DEFAULT_WORKERS: usize = 8;
const EXPERIMENT_DEADLINE_SECONDS: u64 = 75 * 60;
const PARTITION_SCHEMA: &str = "uor-r4.intrinsic-lorentz-r4-attention-partition/1";

const EXPECTED_LAYERS: usize = 30;
const EXPECTED_HEADS: usize = 9;
const EXPECTED_KV_HEADS: usize = 3;
const EXPECTED_HEAD_WIDTH: usize = 64;
const R4_WIDTH: usize = 4;
const BLOCKS: usize = EXPECTED_HEAD_WIDTH / R4_WIDTH;

const NNLS_SWEEPS: usize = 128;
const RIDGE: f64 = 1.0e-6;
const COEFFICIENT_FLOOR: f64 = 0.0;
const OUTPUT_SCALE_FLOOR: f64 = 1.0e-6;
const EXPECTED_FIT_CAUSAL_ROWS: u64 = 34_560;
const EXPECTED_FIT_CAUSAL_SOURCE_PAIRS: u64 = 432_000;
const EXPECTED_FIT_GEOMETRIC_ROW_EVALUATIONS: u64 = 69_120;
const EXPECTED_FIT_GEOMETRIC_SOURCE_PAIR_EVALUATIONS: u64 = 864_000;
const EXPECTED_FIT_FEATURE_BLOCK_EVALUATIONS: u64 = 13_824_000;
const EXPECTED_FIT_CENTROID_SOURCE_BLOCK_EVALUATIONS: u64 = 6_912_000;
const EXPECTED_FIT_OUTPUT_SCALE_LANE_ACCUMULATIONS: u64 = 2_211_840;
const EXPECTED_FIT_NNLS_COORDINATE_UPDATES: u64 = 552_960;
const EXPECTED_FIT_PARAMETER_SCALARS: usize = 8_640;

const VALIDATION_DONOR_MARGIN: f64 = 0.05;
const VALIDATION_FLAT_MARGIN: f64 = 0.05;
const LIVE_ATTENTION_DELTA: f64 = 1.0e-4;
const FINAL_REFERENCE_MARGIN: f64 = 0.02;
const FINAL_FLAT_MARGIN: f64 = 0.01;
const FINAL_CONTROL_MARGIN: f64 = 0.02;

const PASS_TERMINAL: &str = "PASS_INTRINSIC_LORENTZ_R4_ADVANCE_TO_MULTI_RESONANCE";
const RETAIN_TERMINAL: &str =
    "RETAIN_INTRINSIC_FUNCTIONAL_PARITY_NO_CURVATURE_ADVANTAGE_STOP_BEFORE_RESONANCE";
const FAIL_TERMINAL: &str = "FAIL_INTRINSIC_LORENTZ_R4_REVISE_DISTANCE_CENTROID_OR_TRAINING_SEAM";
const VALIDATION_FAIL_TERMINAL: &str =
    "FAIL_INTRINSIC_LORENTZ_R4_CONSTRUCTION_VALIDATION_STOP_BEFORE_HELD_OUT";
const UNAVAILABLE_TERMINAL: &str = "UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT";
const POST_REVEAL_INVALID_TERMINAL: &str = "INVALID_INTRINSIC_LORENTZ_R4_POST_REVEAL_EVIDENCE";
const ATTEMPT_TWO_RESULT_FILE: &str = "result.attempt-02-checkpoint-float-roundtrip.json";

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
type Vector4 = [f64; R4_WIDTH];

#[derive(Debug)]
struct ExperimentDeadlineExceeded {
    stage: String,
    elapsed_seconds: f64,
    deadline_seconds: u64,
}

impl std::fmt::Display for ExperimentDeadlineExceeded {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "experiment deadline exceeded at stage {}: elapsed={:.6}s deadline={}s",
            self.stage, self.elapsed_seconds, self.deadline_seconds
        )
    }
}

impl Error for ExperimentDeadlineExceeded {}

struct ExperimentDeadline {
    started: Instant,
    limit: Duration,
    stage: RefCell<String>,
    heldout_opened: Cell<bool>,
}

impl ExperimentDeadline {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            limit: Duration::from_secs(EXPERIMENT_DEADLINE_SECONDS),
            stage: RefCell::new("initialization".to_owned()),
            heldout_opened: Cell::new(false),
        }
    }

    fn check(&self, stage: &str) -> TestResult {
        *self.stage.borrow_mut() = stage.to_owned();
        let elapsed = self.started.elapsed();
        if elapsed >= self.limit {
            return Err(Box::new(ExperimentDeadlineExceeded {
                stage: stage.to_owned(),
                elapsed_seconds: elapsed.as_secs_f64(),
                deadline_seconds: self.limit.as_secs(),
            }));
        }
        Ok(())
    }

    fn mark_heldout_opened(&self) {
        self.heldout_opened.set(true);
    }

    fn heldout_opened(&self) -> bool {
        self.heldout_opened.get()
    }

    fn elapsed_seconds(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    fn is_exceeded(&self) -> bool {
        self.started.elapsed() >= self.limit
    }

    fn stage(&self) -> String {
        self.stage.borrow().clone()
    }
}

#[derive(Debug, Deserialize)]
struct Article {
    id: String,
    title: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    article_count: usize,
    corpus_cid: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenDocumentCommitment {
    id: String,
    title: String,
    selection_digest: String,
    token_cid: String,
    input_cid: String,
    target_cid: String,
    corpus_byte_offset: u64,
    corpus_byte_length: u64,
}

#[derive(Clone, Debug)]
struct FrozenDocument {
    id: String,
    title: String,
    tokens: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenPartitionManifest {
    schema: String,
    issue: u32,
    selection_policy: String,
    corpus_cid: String,
    corpus_documents: usize,
    donor_source_cid: String,
    tokenizer_cid: String,
    required_tokens_per_document: usize,
    input_positions: usize,
    scored_positions: Vec<usize>,
    construction_fit: Vec<FrozenDocumentCommitment>,
    construction_validation: Vec<FrozenDocumentCommitment>,
    d3_heldout: Vec<FrozenDocumentCommitment>,
    d3_target_commitment_cid: String,
    partition_cid: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenManifestEnvelope {
    manifest_cid: String,
    manifest: FrozenPartitionManifest,
}

#[derive(Debug)]
struct Candidate {
    selection_digest: [u8; 32],
    document: FrozenDocumentCommitment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PartitionAccess {
    Construction,
    Heldout,
}

#[derive(Clone, Debug)]
struct QkvCapture {
    query: Vec<f32>,
    key: Vec<f32>,
    value: Vec<f32>,
}

#[derive(Clone, Debug)]
struct LayerCapture {
    qkv: QkvCapture,
    attention: Vec<Vec<f32>>,
}

#[derive(Debug)]
struct CapturedDocument {
    document: FrozenDocument,
    positions: Vec<Vec<LayerCapture>>,
    logits: Vec<Vec<f32>>,
    atlas: R4SpinFrameAtlas,
}

#[derive(Clone)]
struct NormalEquation {
    gram: [[f64; BLOCKS]; BLOCKS],
    correlation: [f64; BLOCKS],
    target_square: f64,
    rows: u64,
    source_pairs: u64,
}

impl Default for NormalEquation {
    fn default() -> Self {
        Self {
            gram: [[0.0; BLOCKS]; BLOCKS],
            correlation: [0.0; BLOCKS],
            target_square: 0.0,
            rows: 0,
            source_pairs: 0,
        }
    }
}

struct GeometricRow {
    donor_weights: Vec<f64>,
    features: Vec<[f64; BLOCKS]>,
    values: Vec<[Vector4; BLOCKS]>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FitReport {
    metric: IntrinsicR4AttentionMetric,
    construction_document_count: usize,
    causal_rows: u64,
    causal_source_pairs: u64,
    geometric_row_evaluations: u64,
    geometric_source_pair_evaluations: u64,
    feature_block_evaluations: u64,
    centroid_source_block_evaluations: u64,
    output_scale_lane_accumulations: u64,
    nnls_sweeps: usize,
    nnls_coordinate_updates: u64,
    ridge: f64,
    coefficient_floor: f64,
    output_scale_floor: f64,
    parameter_scalars: usize,
    active_metric_coefficients: usize,
    row_centered_objective: f64,
    construction_trace_cid: String,
    parameter_json_cid: String,
    fit_report_cid: String,
}

#[derive(Clone)]
struct FittedArm {
    parameters: IntrinsicLorentzR4AttentionParameters,
    report: FitReport,
    parameter_json: Vec<u8>,
}

struct FitStageOutcome {
    curved: FittedArm,
    flat: FittedArm,
    replay: FitReplayEvidence,
    checkpoint_cid: String,
    resumed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ImplementationIdentity {
    revision: String,
    executable_cid: String,
    core_source_cid: String,
    harness_source_cid: String,
    model_attention_source_cid: String,
    model_source_cid: String,
    exact_executor_source_cid: String,
    contract_cid: String,
    compiled_partition_bytes_cid: String,
}

struct PartitionRunLock {
    _file: fs::File,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FitReplayEvidence {
    parameter_replay_exact: bool,
    fit_report_replay_exact: bool,
    fit_work_and_shape_valid: bool,
    curved_primary_parameter_cid: String,
    curved_replay_parameter_cid: String,
    flat_primary_parameter_cid: String,
    flat_replay_parameter_cid: String,
    curved_primary_fit_report_cid: String,
    curved_replay_fit_report_cid: String,
    flat_primary_fit_report_cid: String,
    flat_replay_fit_report_cid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FitCheckpoint {
    schema: String,
    issue: u32,
    manifest_cid: String,
    partition_cid: String,
    implementation_identity: ImplementationIdentity,
    curved_parameters: IntrinsicLorentzR4AttentionParameters,
    flat_parameters: IntrinsicLorentzR4AttentionParameters,
    curved_fit: FitReport,
    flat_fit: FitReport,
    replay: FitReplayEvidence,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FitCheckpointEnvelope {
    checkpoint_cid: String,
    checkpoint: FitCheckpoint,
}

#[derive(Debug, Serialize)]
struct D3RevealMarker {
    schema: &'static str,
    issue: u32,
    manifest_cid: String,
    partition_cid: String,
    fit_checkpoint_cid: String,
    implementation_identity: ImplementationIdentity,
}

#[derive(Clone, Debug, Serialize)]
struct PositionResult {
    query_position: usize,
    input_token: u32,
    target_token: u32,
    target_nll_nats: f64,
    top1_token: u32,
    top1_hit: bool,
    top8_hit: bool,
}

#[derive(Clone, Debug, Serialize)]
struct AuditReport {
    positions: u64,
    layers: u64,
    heads: u64,
    query_transforms: u64,
    key_transports: u64,
    value_transports: u64,
    output_transforms: u64,
    future_reads: u64,
    maximum_query_position: Option<usize>,
    maximum_source_position: Option<usize>,
}

impl From<CausalAttentionTransportAudit> for AuditReport {
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

#[derive(Clone, Debug, Serialize)]
struct DocumentResult {
    document_id: String,
    title: String,
    positions: Vec<PositionResult>,
    mean_nll_nats: f64,
    top1_hits: usize,
    top8_hits: usize,
    logits_cid: String,
    state_cid: String,
    causal_audit: Option<AuditReport>,
    implementation_evidence: Option<serde_json::Value>,
    implementation_evidence_cid: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ArmReport {
    arm: String,
    documents: Vec<DocumentResult>,
    scored_positions: usize,
    mean_nll_nats: f64,
    perplexity: f64,
    top1_hits: usize,
    top8_hits: usize,
    top1_accuracy: f64,
    top8_accuracy: f64,
    logits_cid: String,
    state_cid: String,
    audit_cid: String,
    evidence_cid: String,
}

struct ArmExecution {
    report: ArmReport,
    logits: Vec<Vec<Vec<f32>>>,
}

#[derive(Clone, Debug, Serialize)]
struct PositionDifference {
    document_id: String,
    query_position: usize,
    nll_delta_nats: f64,
}

#[derive(Clone, Debug, Serialize)]
struct ArmComparison {
    reference_arm: String,
    candidate_arm: String,
    candidate_minus_reference_mean_nll: f64,
    candidate_worse_document_count: usize,
    reference_worse_document_count: usize,
    mean_donor_kl_nats: f64,
    maximum_absolute_logit_delta: f64,
    mean_absolute_logit_delta: f64,
    position_nll_deltas: Vec<PositionDifference>,
    document_nll_deltas: Vec<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct DecodeReport {
    arm: String,
    generated_tokens: Vec<u32>,
    generated_text: String,
    generated_token_cid: String,
    state_cid: String,
    causal_audit: Option<AuditReport>,
    evidence_cid: Option<String>,
    no_period_one_or_two_cycle: bool,
}

#[derive(Clone, Debug, Serialize)]
struct GeometryPreflight {
    exercised_blocks: u64,
    maximum_hyperboloid_residual: f64,
    maximum_distance_invariance_delta: f64,
    maximum_barycenter_covariance_delta: f64,
    minimum_timelike_denominator_squared: f64,
    maximum_softmax_sum_delta: f64,
    helm_d_golden_reproduced: bool,
    passed: bool,
}

#[derive(Clone, Debug, Serialize)]
struct ValidationGateReport {
    passed: bool,
    curved_minus_donor_nll: f64,
    curved_minus_flat_nll: f64,
    maximum_curved_vs_flat_attention_delta: f64,
    parameter_replay_exact: bool,
    fit_report_replay_exact: bool,
    fit_work_and_shape_valid: bool,
    trace_matches_donor_decoder: bool,
    zero_faults_and_future_reads: bool,
    geometry_preflight: GeometryPreflight,
    failures: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct Thresholds {
    validation_donor_margin: f64,
    validation_flat_margin: f64,
    live_attention_delta: f64,
    final_reference_margin: f64,
    final_flat_margin: f64,
    final_control_margin: f64,
    required_document_wins: usize,
    top1_reference_shortfall_tokens: usize,
}

#[derive(Clone, Debug, Serialize)]
struct HeldoutReport {
    ordinary_donor: ArmReport,
    gauge_r4_reference: ArmReport,
    intrinsic_lorentz_r4: ArmReport,
    flat_r4_distance: ArmReport,
    source_frame_permuted: ArmReport,
    value_permuted: ArmReport,
    curved_replay: ArmReport,
    curved_replay_exact: bool,
    comparisons_to_donor: Vec<ArmComparison>,
    curved_vs_gauge: ArmComparison,
    flat_vs_curved: ArmComparison,
    source_frame_permuted_vs_curved: ArmComparison,
    value_permuted_vs_curved: ArmComparison,
    decodes: Vec<DecodeReport>,
    curved_decode_replay: DecodeReport,
    curved_decode_replay_exact: bool,
    scientific_result_cid: String,
}

#[derive(Debug, Serialize)]
struct TimingReport {
    trace_and_fit_seconds: f64,
    validation_seconds: f64,
    heldout_seconds: Option<f64>,
    total_seconds: f64,
    deadline_seconds: u64,
    deadline_exceeded: bool,
}

#[derive(Debug, Serialize)]
struct ResultPayload {
    schema: &'static str,
    issue: u32,
    terminal: &'static str,
    helm_d_upstream_commit: &'static str,
    manifest_cid: String,
    partition_cid: String,
    donor_source_cid: String,
    tokenizer_cid: String,
    worker_policy: String,
    implementation_identity: ImplementationIdentity,
    fit_checkpoint_cid: String,
    d3_reveal_marker_cid: Option<String>,
    model_shape: ModelShape,
    thresholds: Thresholds,
    curved_parameters: IntrinsicLorentzR4AttentionParameters,
    flat_parameters: IntrinsicLorentzR4AttentionParameters,
    curved_fit: FitReport,
    flat_fit: FitReport,
    validation_gate: ValidationGateReport,
    validation_donor: ArmReport,
    validation_curved: ArmReport,
    validation_flat: ArmReport,
    heldout: Option<HeldoutReport>,
    nonclaims: [&'static str; 8],
}

#[derive(Debug, Serialize)]
struct OperationalTelemetry {
    schema: &'static str,
    execution_preparation: TeacherExecutionPreparation,
    execution_snapshot: TeacherExecutionSnapshot,
    fit_checkpoint_resumed: bool,
    timing: TimingReport,
}

#[derive(Debug, Serialize)]
struct ResultEnvelope {
    result_cid: String,
    result: ResultPayload,
    operational_telemetry: OperationalTelemetry,
}

#[derive(Debug, Serialize)]
struct FailureResult {
    schema: &'static str,
    issue: u32,
    terminal: &'static str,
    manifest_cid: String,
    partition_cid: String,
    d3_reveal_marker_cid: Option<String>,
    stage: String,
    error: String,
    elapsed_seconds: f64,
    deadline_seconds: u64,
    deadline_exceeded: bool,
    heldout_opened: bool,
}

#[derive(Debug, Serialize)]
struct FailureEnvelope {
    result_cid: String,
    result: FailureResult,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct ModelShape {
    dimension: usize,
    layers: usize,
    query_heads: usize,
    kv_heads: usize,
    head_width: usize,
    vocabulary: usize,
}

#[derive(Clone, Copy)]
enum ArmSpec<'a> {
    Donor,
    Gauge,
    Intrinsic {
        parameters: &'a IntrinsicLorentzR4AttentionParameters,
        metric: IntrinsicR4AttentionMetric,
        intervention: IntrinsicR4AttentionIntervention,
    },
}

#[test]
#[ignore = "freezes the pinned tokenizer and 3,000-document SimpleWiki partition"]
fn freeze_intrinsic_lorentz_r4_partition_manifest() -> TestResult {
    let tokenizer_path = path_from_env(TOKENIZER_ENV, DEFAULT_TOKENIZER);
    let corpus_path = path_from_env(CORPUS_ENV, DEFAULT_CORPUS);
    require_file(&tokenizer_path)?;
    require_file(&corpus_path)?;
    verify_corpus(&corpus_path)?;

    let tokenizer = Tokenizer::try_load(&tokenizer_path)?;
    let tokenizer_cid = file_cid(&tokenizer_path)?;
    let mut fit = Vec::new();
    let mut validation = Vec::new();
    let mut heldout = Vec::new();
    let mut observed_ids = HashSet::new();
    let mut observed_documents = 0usize;

    let mut corpus = BufReader::new(fs::File::open(&corpus_path)?);
    let mut line = Vec::new();
    let mut corpus_byte_offset = 0u64;
    loop {
        line.clear();
        let bytes_read = corpus.read_until(b'\n', &mut line)?;
        if bytes_read == 0 {
            break;
        }
        let line_offset = corpus_byte_offset;
        let line_length = u64::try_from(bytes_read)?;
        corpus_byte_offset = corpus_byte_offset
            .checked_add(line_length)
            .ok_or("corpus byte offset overflow")?;
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        observed_documents = observed_documents
            .checked_add(1)
            .ok_or("corpus document count overflow")?;
        let article: Article = serde_json::from_slice(&line)?;
        if !observed_ids.insert(article.id.clone()) {
            return Err(format!("duplicate SimpleWiki document id {}", article.id).into());
        }
        let tokens = tokenizer.encode(&format!("{}\n\n{}", article.title, article.text));
        if tokens.len() < REQUIRED_TOKENS {
            continue;
        }
        let selection_digest = selection_digest(&article.id);
        let document = freeze_document(
            article,
            selection_digest,
            &tokens[..REQUIRED_TOKENS],
            line_offset,
            line_length,
        )?;
        if d3_is_held_out(&document.id) {
            if document.id != EXCLUDED_HELDOUT_ID {
                heldout.push(Candidate {
                    selection_digest,
                    document,
                });
            }
        } else if selection_digest[0].is_multiple_of(5) {
            validation.push(Candidate {
                selection_digest,
                document,
            });
        } else {
            fit.push(Candidate {
                selection_digest,
                document,
            });
        }
    }
    if observed_documents != CORPUS_DOCUMENTS {
        return Err(format!(
            "SimpleWiki document count mismatch: expected {CORPUS_DOCUMENTS}, observed {observed_documents}"
        )
        .into());
    }

    sort_candidates(&mut fit);
    sort_candidates(&mut validation);
    sort_candidates(&mut heldout);
    let construction_fit = take_candidates(fit, FIT_DOCUMENTS, "construction fit")?;
    let construction_validation =
        take_candidates(validation, VALIDATION_DOCUMENTS, "construction validation")?;
    let d3_heldout = take_candidates(heldout, HELDOUT_DOCUMENTS, "D3 heldout")?;

    let d3_target_commitment_cid = aggregate_d3_target_commitment(&d3_heldout);
    let mut manifest = FrozenPartitionManifest {
        schema: PARTITION_SCHEMA.to_owned(),
        issue: 973,
        selection_policy:
            "blake3(domain-nul-utf8-id); D3; eligible>=17; 16-fit/4-validation/8-heldout; exclude-id-12"
                .to_owned(),
        corpus_cid: CORPUS_CID.to_owned(),
        corpus_documents: CORPUS_DOCUMENTS,
        donor_source_cid: DONOR_CID.to_owned(),
        tokenizer_cid,
        required_tokens_per_document: REQUIRED_TOKENS,
        input_positions: INPUT_POSITIONS,
        scored_positions: (SCORE_START..INPUT_POSITIONS).collect(),
        construction_fit,
        construction_validation,
        d3_heldout,
        d3_target_commitment_cid,
        partition_cid: String::new(),
    };
    validate_manifest_documents(&manifest)?;
    manifest.partition_cid = partition_cid(&manifest)?;
    let manifest_bytes = serde_json::to_vec(&manifest)?;
    let manifest_cid = cid_bytes(&manifest_bytes);
    let envelope = FrozenManifestEnvelope {
        manifest_cid: manifest_cid.clone(),
        manifest,
    };

    match env::var_os(MANIFEST_OUTPUT_ENV) {
        Some(path) => write_pretty_json(Path::new(&path), &envelope)?,
        None => eprintln!(
            "intrinsic R4 partition frozen: manifest_cid={manifest_cid} partition_cid={}",
            envelope.manifest.partition_cid
        ),
    }
    Ok(())
}

#[test]
#[ignore = "requires the frozen manifest, local SmolLM2 source, and bounded exact full-decoder run"]
fn intrinsic_lorentz_r4_full_decoder_decision() -> TestResult {
    let output_path = required_path_from_env(RESULT_OUTPUT_ENV)?;
    let manifest_path = required_path_from_env(MANIFEST_ENV)?;
    require_file(&manifest_path)?;
    let manifest_bytes = fs::read(&manifest_path)?;
    let observed_partition_bytes_cid = cid_bytes(&manifest_bytes);
    let manifest_envelope = parse_frozen_manifest(&manifest_bytes)?;
    let canonical_output_path = canonical_result_path(&manifest_envelope.manifest)?;
    if output_path != canonical_output_path {
        return Err(format!(
            "{RESULT_OUTPUT_ENV} must equal the partition-scoped canonical ledger path {}; observed {}",
            canonical_output_path.display(),
            output_path.display()
        )
        .into());
    }
    let _run_lock = acquire_partition_run_lock(&canonical_output_path)?;
    let deadline = ExperimentDeadline::new();
    if let Some(result_cid) =
        reconcile_interrupted_reveal(&output_path, &deadline, &manifest_envelope)?
    {
        return Err(format!(
            "reconciled interrupted post-reveal run as invalid evidence: result_cid={result_cid} report={}",
            output_path.display()
        )
        .into());
    }
    ensure_fresh_run_target(&output_path)?;
    let implementation_identity = implementation_identity()?;
    verify_committed_implementation(&implementation_identity)?;
    match run_intrinsic_lorentz_r4_full_decoder_decision(
        &output_path,
        &deadline,
        &manifest_envelope,
        &observed_partition_bytes_cid,
        implementation_identity,
    ) {
        Ok(()) => Ok(()),
        Err(error) => {
            match write_failure(
                &output_path,
                &output_path,
                &deadline,
                error.as_ref(),
                &manifest_envelope,
            ) {
                Ok(result_cid) => eprintln!(
                    "intrinsic R4 decision failed: stage={} result_cid={result_cid} report={}",
                    deadline.stage(),
                    output_path.display()
                ),
                Err(write_error) => {
                    return Err(format!(
                        "{error}; additionally failed to write unavailable result to {}: {write_error}",
                        output_path.display()
                    )
                    .into());
                }
            }
            Err(error)
        }
    }
}

fn run_intrinsic_lorentz_r4_full_decoder_decision(
    output_path: &Path,
    deadline: &ExperimentDeadline,
    manifest_envelope: &FrozenManifestEnvelope,
    observed_partition_bytes_cid: &str,
    implementation_identity: ImplementationIdentity,
) -> TestResult {
    deadline.check("initialization")?;
    if env::var(CANONICAL_DETERMINISTIC_ENV).as_deref() != Ok("1") {
        return Err(
            format!("intrinsic R4 decision requires {CANONICAL_DETERMINISTIC_ENV}=1").into(),
        );
    }
    let model_path = path_from_env(MODEL_ENV, DEFAULT_MODEL);
    let tokenizer_path = path_from_env(TOKENIZER_ENV, DEFAULT_TOKENIZER);
    let corpus_path = path_from_env(CORPUS_ENV, DEFAULT_CORPUS);
    let workers = positive_usize_env(WORKERS_ENV, DEFAULT_WORKERS)?;
    if workers != DEFAULT_WORKERS {
        return Err(format!(
            "frozen intrinsic R4 decision requires exactly {DEFAULT_WORKERS} donor/full-decoder workers; observed {workers}"
        )
        .into());
    }
    let workers = NonZeroUsize::new(workers).ok_or("worker count must be positive")?;
    deadline.check("inputs.validate")?;
    require_file(&tokenizer_path)?;
    require_file(&corpus_path)?;
    if !model_path.is_dir() {
        return Err(format!("model directory is unavailable: {}", model_path.display()).into());
    }
    deadline.check("inputs.manifest_validated")?;
    let manifest = &manifest_envelope.manifest;
    if observed_partition_bytes_cid != implementation_identity.compiled_partition_bytes_cid {
        return Err(format!(
            "runtime partition bytes differ from the compiled implementation identity: compiled {}, observed {observed_partition_bytes_cid}",
            implementation_identity.compiled_partition_bytes_cid
        )
        .into());
    }
    let observed_tokenizer_cid = file_cid(&tokenizer_path)?;
    if observed_tokenizer_cid != manifest.tokenizer_cid {
        return Err(format!(
            "tokenizer CID mismatch: frozen {}, observed {observed_tokenizer_cid}",
            manifest.tokenizer_cid
        )
        .into());
    }
    let tokenizer = Tokenizer::try_load(&tokenizer_path)?;
    deadline.check("inputs.tokenizer_loaded")?;
    let execution = TeacherExecutionConfig::fixed_workers(workers);
    let mut oracle = HuggingFaceLlamaOracle::load_with_execution(&model_path, execution)?;
    deadline.check("donor.loaded")?;
    if oracle.source_cid() != DONOR_CID || oracle.source_cid() != manifest.donor_source_cid {
        return Err(format!(
            "donor source CID mismatch: frozen {}, observed {}",
            manifest.donor_source_cid,
            oracle.source_cid()
        )
        .into());
    }
    let config = oracle.cfg();
    let head_width = config.dim / config.n_heads;
    if config.r4_attention
        || config.n_layers != EXPECTED_LAYERS
        || config.n_heads != EXPECTED_HEADS
        || config.n_kv_heads != EXPECTED_KV_HEADS
        || head_width != EXPECTED_HEAD_WIDTH
        || head_width / R4_WIDTH != BLOCKS
        || config.seq_len < INPUT_POSITIONS + GENERATED_TOKENS - 1
    {
        return Err(format!(
            "donor geometry/switch mismatch: dim={} layers={} heads={} kv_heads={} head_width={} seq_len={} r4_switch={}",
            config.dim,
            config.n_layers,
            config.n_heads,
            config.n_kv_heads,
            head_width,
            config.seq_len,
            config.r4_attention
        )
        .into());
    }
    let model_shape = ModelShape {
        dimension: config.dim,
        layers: config.n_layers,
        query_heads: config.n_heads,
        kv_heads: config.n_kv_heads,
        head_width,
        vocabulary: config.vocab,
    };
    let execution_preparation = oracle.prepare_exact_execution(1)?;
    deadline.check("donor.prepared")?;
    if execution_preparation.workers_observed != workers.get()
        || !execution_preparation.backend_exercised
    {
        return Err("fixed exact worker pool did not complete bounded preparation".into());
    }

    let fit_started = Instant::now();
    let fit_checkpoint_path = sidecar_path(output_path, ".fit-checkpoint.json");
    let fit_stage = prepare_fit_checkpoint(
        &fit_checkpoint_path,
        &mut oracle,
        &corpus_path,
        &tokenizer,
        manifest_envelope,
        &implementation_identity,
        deadline,
    )?;
    let curved_fit = fit_stage.curved;
    let flat_fit = fit_stage.flat;
    let fit_replay = fit_stage.replay;
    let fit_checkpoint_cid = fit_stage.checkpoint_cid;
    let fit_checkpoint_resumed = fit_stage.resumed;
    let trace_and_fit_seconds = fit_started.elapsed().as_secs_f64();

    // The validation corpus is intentionally not materialized until the full
    // fitted oracle has been persisted, re-read, and identity-checked.
    deadline.check("construction.fit.checkpoint_verified")?;
    let validation_documents = materialize_committed_documents(
        &corpus_path,
        &tokenizer,
        &manifest.construction_validation,
        PartitionAccess::Construction,
        deadline,
        "construction.validation.materialize",
    )?;

    let validation_started = Instant::now();
    let validation_captures = capture_documents(
        &mut oracle,
        &validation_documents,
        deadline,
        "construction.validation.trace_capture",
    )?;
    let validation_donor = run_arm(
        &oracle,
        &validation_documents,
        ArmSpec::Donor,
        "ordinary_donor",
        deadline,
        "construction.validation.donor",
    )?;
    let validation_curved = run_arm(
        &oracle,
        &validation_documents,
        ArmSpec::Intrinsic {
            parameters: &curved_fit.parameters,
            metric: IntrinsicR4AttentionMetric::Lorentz,
            intervention: IntrinsicR4AttentionIntervention::Coherent,
        },
        "intrinsic_lorentz_r4",
        deadline,
        "construction.validation.curved",
    )?;
    let validation_flat = run_arm(
        &oracle,
        &validation_documents,
        ArmSpec::Intrinsic {
            parameters: &flat_fit.parameters,
            metric: IntrinsicR4AttentionMetric::Flat,
            intervention: IntrinsicR4AttentionIntervention::Coherent,
        },
        "flat_r4_distance",
        deadline,
        "construction.validation.flat",
    )?;
    let validation_gate = validation_gate(
        &validation_donor,
        &validation_curved,
        &validation_flat,
        &validation_captures,
        &curved_fit,
        &flat_fit,
        &fit_replay,
        deadline,
    )?;
    let validation_seconds = validation_started.elapsed().as_secs_f64();
    drop(validation_captures);

    let mut heldout_seconds = None;
    let mut d3_reveal_marker_cid = None;
    let (terminal, heldout) = if validation_gate.passed {
        deadline.check("heldout.admission")?;
        let marker = D3RevealMarker {
            schema: "uor-r4.intrinsic-lorentz-r4-attention-d3-reveal/1",
            issue: 973,
            manifest_cid: manifest_envelope.manifest_cid.clone(),
            partition_cid: manifest.partition_cid.clone(),
            fit_checkpoint_cid: fit_checkpoint_cid.clone(),
            implementation_identity: implementation_identity.clone(),
        };
        let marker_path = partition_reveal_path(output_path)?;
        let marker_cid = match write_content_addressed_exclusive(&marker_path, &marker) {
            Ok(marker_cid) => marker_cid,
            Err(error) => {
                // hard_link publishes the final path before temporary-file cleanup and
                // directory sync. Once that path exists, every later failure is
                // post-reveal even if publication returned an error.
                if marker_path.is_file() {
                    deadline.mark_heldout_opened();
                }
                return Err(error);
            }
        };
        deadline.mark_heldout_opened();
        d3_reveal_marker_cid = Some(marker_cid);
        deadline.check("heldout.corpus_verify")?;
        verify_corpus(&corpus_path)?;
        deadline.check("heldout.corpus_verified")?;
        let heldout_documents = materialize_committed_documents(
            &corpus_path,
            &tokenizer,
            &manifest.d3_heldout,
            PartitionAccess::Heldout,
            deadline,
            "heldout.materialize",
        )?;
        let materialized_target_commitment =
            aggregate_materialized_d3_target_commitment(&heldout_documents)?;
        if materialized_target_commitment != manifest.d3_target_commitment_cid {
            return Err(format!(
                "materialized D3 target commitment mismatch: frozen {}, observed {materialized_target_commitment}",
                manifest.d3_target_commitment_cid
            )
            .into());
        }
        let heldout_started = Instant::now();
        let ordinary_donor = run_arm(
            &oracle,
            &heldout_documents,
            ArmSpec::Donor,
            "ordinary_donor",
            deadline,
            "heldout.ordinary_donor",
        )?;
        let gauge_r4_reference = run_arm(
            &oracle,
            &heldout_documents,
            ArmSpec::Gauge,
            "gauge_r4_reference",
            deadline,
            "heldout.gauge_r4_reference",
        )?;
        let intrinsic_lorentz_r4 = run_arm(
            &oracle,
            &heldout_documents,
            ArmSpec::Intrinsic {
                parameters: &curved_fit.parameters,
                metric: IntrinsicR4AttentionMetric::Lorentz,
                intervention: IntrinsicR4AttentionIntervention::Coherent,
            },
            "intrinsic_lorentz_r4",
            deadline,
            "heldout.intrinsic_lorentz_r4",
        )?;
        let flat_r4_distance = run_arm(
            &oracle,
            &heldout_documents,
            ArmSpec::Intrinsic {
                parameters: &flat_fit.parameters,
                metric: IntrinsicR4AttentionMetric::Flat,
                intervention: IntrinsicR4AttentionIntervention::Coherent,
            },
            "flat_r4_distance",
            deadline,
            "heldout.flat_r4_distance",
        )?;
        let source_frame_permuted = run_arm(
            &oracle,
            &heldout_documents,
            ArmSpec::Intrinsic {
                parameters: &curved_fit.parameters,
                metric: IntrinsicR4AttentionMetric::Lorentz,
                intervention: IntrinsicR4AttentionIntervention::SourceFramePermuted,
            },
            "source_frame_permuted",
            deadline,
            "heldout.source_frame_permuted",
        )?;
        let value_permuted = run_arm(
            &oracle,
            &heldout_documents,
            ArmSpec::Intrinsic {
                parameters: &curved_fit.parameters,
                metric: IntrinsicR4AttentionMetric::Lorentz,
                intervention: IntrinsicR4AttentionIntervention::ValuePermuted,
            },
            "value_permuted",
            deadline,
            "heldout.value_permuted",
        )?;
        let curved_replay = run_arm(
            &oracle,
            &heldout_documents,
            ArmSpec::Intrinsic {
                parameters: &curved_fit.parameters,
                metric: IntrinsicR4AttentionMetric::Lorentz,
                intervention: IntrinsicR4AttentionIntervention::Coherent,
            },
            "intrinsic_lorentz_r4",
            deadline,
            "heldout.curved_replay",
        )?;
        deadline.check("heldout.replay_compare")?;
        let curved_replay_exact = exact_arm_replay(&intrinsic_lorentz_r4, &curved_replay)?;

        let donor_vs_gauge = compare_arms(
            &ordinary_donor,
            &gauge_r4_reference,
            deadline,
            "heldout.compare.donor_gauge",
        )?;
        let donor_vs_curved = compare_arms(
            &ordinary_donor,
            &intrinsic_lorentz_r4,
            deadline,
            "heldout.compare.donor_curved",
        )?;
        let donor_vs_flat = compare_arms(
            &ordinary_donor,
            &flat_r4_distance,
            deadline,
            "heldout.compare.donor_flat",
        )?;
        let donor_vs_source = compare_arms(
            &ordinary_donor,
            &source_frame_permuted,
            deadline,
            "heldout.compare.donor_source_permuted",
        )?;
        let donor_vs_value = compare_arms(
            &ordinary_donor,
            &value_permuted,
            deadline,
            "heldout.compare.donor_value_permuted",
        )?;
        let curved_vs_gauge = compare_arms(
            &gauge_r4_reference,
            &intrinsic_lorentz_r4,
            deadline,
            "heldout.compare.gauge_curved",
        )?;
        let flat_vs_curved = compare_arms(
            &intrinsic_lorentz_r4,
            &flat_r4_distance,
            deadline,
            "heldout.compare.curved_flat",
        )?;
        let source_frame_permuted_vs_curved = compare_arms(
            &intrinsic_lorentz_r4,
            &source_frame_permuted,
            deadline,
            "heldout.compare.curved_source_permuted",
        )?;
        let value_permuted_vs_curved = compare_arms(
            &intrinsic_lorentz_r4,
            &value_permuted,
            deadline,
            "heldout.compare.curved_value_permuted",
        )?;

        let first = heldout_documents
            .first()
            .ok_or("heldout manifest has no first document")?;
        let donor_decode = run_decode(
            &oracle,
            &tokenizer,
            first,
            ArmSpec::Donor,
            "ordinary_donor",
            deadline,
            "heldout.decode.donor",
        )?;
        let gauge_decode = run_decode(
            &oracle,
            &tokenizer,
            first,
            ArmSpec::Gauge,
            "gauge_r4_reference",
            deadline,
            "heldout.decode.gauge",
        )?;
        let curved_decode = run_decode(
            &oracle,
            &tokenizer,
            first,
            ArmSpec::Intrinsic {
                parameters: &curved_fit.parameters,
                metric: IntrinsicR4AttentionMetric::Lorentz,
                intervention: IntrinsicR4AttentionIntervention::Coherent,
            },
            "intrinsic_lorentz_r4",
            deadline,
            "heldout.decode.curved",
        )?;
        let flat_decode = run_decode(
            &oracle,
            &tokenizer,
            first,
            ArmSpec::Intrinsic {
                parameters: &flat_fit.parameters,
                metric: IntrinsicR4AttentionMetric::Flat,
                intervention: IntrinsicR4AttentionIntervention::Coherent,
            },
            "flat_r4_distance",
            deadline,
            "heldout.decode.flat",
        )?;
        let source_decode = run_decode(
            &oracle,
            &tokenizer,
            first,
            ArmSpec::Intrinsic {
                parameters: &curved_fit.parameters,
                metric: IntrinsicR4AttentionMetric::Lorentz,
                intervention: IntrinsicR4AttentionIntervention::SourceFramePermuted,
            },
            "source_frame_permuted",
            deadline,
            "heldout.decode.source_permuted",
        )?;
        let value_decode = run_decode(
            &oracle,
            &tokenizer,
            first,
            ArmSpec::Intrinsic {
                parameters: &curved_fit.parameters,
                metric: IntrinsicR4AttentionMetric::Lorentz,
                intervention: IntrinsicR4AttentionIntervention::ValuePermuted,
            },
            "value_permuted",
            deadline,
            "heldout.decode.value_permuted",
        )?;
        let curved_decode_replay = run_decode(
            &oracle,
            &tokenizer,
            first,
            ArmSpec::Intrinsic {
                parameters: &curved_fit.parameters,
                metric: IntrinsicR4AttentionMetric::Lorentz,
                intervention: IntrinsicR4AttentionIntervention::Coherent,
            },
            "intrinsic_lorentz_r4",
            deadline,
            "heldout.decode.curved_replay",
        )?;
        deadline.check("heldout.decode.replay_compare")?;
        let curved_decode_replay_exact =
            exact_decode_replay(&curved_decode, &curved_decode_replay)?;

        let retention = donor_vs_curved.candidate_minus_reference_mean_nll
            <= FINAL_REFERENCE_MARGIN
            && curved_vs_gauge.candidate_minus_reference_mean_nll <= FINAL_REFERENCE_MARGIN
            && intrinsic_lorentz_r4.report.top1_hits + 1 >= ordinary_donor.report.top1_hits
            && intrinsic_lorentz_r4.report.top1_hits + 1 >= gauge_r4_reference.report.top1_hits;
        let curvature_separation = flat_vs_curved.candidate_minus_reference_mean_nll
            >= FINAL_FLAT_MARGIN
            && flat_vs_curved.candidate_worse_document_count >= 7
            && intrinsic_lorentz_r4.report.top1_hits >= flat_r4_distance.report.top1_hits;
        let control_separation = source_frame_permuted_vs_curved.candidate_minus_reference_mean_nll
            >= FINAL_CONTROL_MARGIN
            && value_permuted_vs_curved.candidate_minus_reference_mean_nll >= FINAL_CONTROL_MARGIN
            && source_frame_permuted_vs_curved.candidate_worse_document_count >= 7
            && value_permuted_vs_curved.candidate_worse_document_count >= 7;
        let contract_integrity = curved_replay_exact
            && curved_decode_replay_exact
            && curved_decode.no_period_one_or_two_cycle;
        let terminal =
            if retention && curvature_separation && control_separation && contract_integrity {
                PASS_TERMINAL
            } else if retention && contract_integrity {
                RETAIN_TERMINAL
            } else {
                FAIL_TERMINAL
            };
        let mut report = HeldoutReport {
            ordinary_donor: ordinary_donor.report,
            gauge_r4_reference: gauge_r4_reference.report,
            intrinsic_lorentz_r4: intrinsic_lorentz_r4.report,
            flat_r4_distance: flat_r4_distance.report,
            source_frame_permuted: source_frame_permuted.report,
            value_permuted: value_permuted.report,
            curved_replay: curved_replay.report,
            curved_replay_exact,
            comparisons_to_donor: vec![
                donor_vs_gauge,
                donor_vs_curved,
                donor_vs_flat,
                donor_vs_source,
                donor_vs_value,
            ],
            curved_vs_gauge,
            flat_vs_curved,
            source_frame_permuted_vs_curved,
            value_permuted_vs_curved,
            decodes: vec![
                donor_decode,
                gauge_decode,
                curved_decode,
                flat_decode,
                source_decode,
                value_decode,
            ],
            curved_decode_replay,
            curved_decode_replay_exact,
            scientific_result_cid: String::new(),
        };
        report.scientific_result_cid = cid_bytes(&serde_json::to_vec(&report)?);
        heldout_seconds = Some(heldout_started.elapsed().as_secs_f64());
        (terminal, Some(report))
    } else {
        (validation_failure_terminal(&validation_gate), None)
    };

    let result = ResultPayload {
        schema: "uor-r4.intrinsic-lorentz-r4-attention-result/1",
        issue: 973,
        terminal,
        helm_d_upstream_commit: HELM_D_UPSTREAM_COMMIT,
        manifest_cid: manifest_envelope.manifest_cid.clone(),
        partition_cid: manifest.partition_cid.clone(),
        donor_source_cid: oracle.source_cid().to_owned(),
        tokenizer_cid: observed_tokenizer_cid,
        worker_policy: format!(
            "donor_and_full_decoder_fixed_workers:{};fitter_fixed_order_serial",
            workers.get()
        ),
        implementation_identity,
        fit_checkpoint_cid,
        d3_reveal_marker_cid,
        model_shape,
        thresholds: thresholds(),
        curved_parameters: curved_fit.parameters,
        flat_parameters: flat_fit.parameters,
        curved_fit: curved_fit.report,
        flat_fit: flat_fit.report,
        validation_gate,
        validation_donor: validation_donor.report,
        validation_curved: validation_curved.report,
        validation_flat: validation_flat.report,
        heldout,
        nonclaims: [
            "not softmax removal",
            "not subquadratic inference",
            "not bounded recurrence",
            "not exact table-native lowering",
            "not transformerless serving",
            "not broad language quality",
            "not correctness or reasoning",
            "not chat or release readiness",
        ],
    };
    let operational_telemetry = OperationalTelemetry {
        schema: "uor-r4.intrinsic-lorentz-r4-attention-operational-telemetry/1",
        execution_preparation,
        execution_snapshot: oracle.execution_snapshot(),
        fit_checkpoint_resumed,
        timing: TimingReport {
            trace_and_fit_seconds,
            validation_seconds,
            heldout_seconds,
            total_seconds: deadline.elapsed_seconds(),
            deadline_seconds: EXPERIMENT_DEADLINE_SECONDS,
            deadline_exceeded: false,
        },
    };
    deadline.check("result.write")?;
    let result_cid = write_result(output_path, result, operational_telemetry)?;
    eprintln!(
        "intrinsic R4 decision: terminal={terminal} result_cid={result_cid} report={}",
        output_path.display()
    );
    Ok(())
}

fn path_from_env(name: &str, default: &str) -> PathBuf {
    env::var_os(name).map_or_else(|| PathBuf::from(default), PathBuf::from)
}

fn required_path_from_env(name: &str) -> TestResult<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("{name} must name an explicit path").into())
}

fn positive_usize_env(name: &str, default: usize) -> TestResult<usize> {
    let value = env::var(name)
        .ok()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .unwrap_or(default);
    if value == 0 {
        return Err(format!("{name} must be positive").into());
    }
    Ok(value)
}

fn implementation_identity() -> TestResult<ImplementationIdentity> {
    let revision = COMPILED_IMPLEMENTATION_REVISION.ok_or_else(|| {
        format!(
            "decision binary was not compiled with {IMPLEMENTATION_REVISION_ENV}=<40-character lowercase git revision>"
        )
    })?;
    if revision.len() != 40
        || !revision
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
    {
        return Err(format!(
            "compiled {IMPLEMENTATION_REVISION_ENV} is not a 40-character lowercase git revision"
        )
        .into());
    }
    let executable_path = env::current_exe()?;
    Ok(ImplementationIdentity {
        revision: revision.to_owned(),
        executable_cid: file_cid(&executable_path)?,
        core_source_cid: cid_bytes(COMPILED_CORE_SOURCE),
        harness_source_cid: cid_bytes(COMPILED_HARNESS_SOURCE),
        model_attention_source_cid: cid_bytes(COMPILED_MODEL_ATTENTION_SOURCE),
        model_source_cid: cid_bytes(COMPILED_MODEL_SOURCE),
        exact_executor_source_cid: cid_bytes(COMPILED_EXACT_EXECUTOR_SOURCE),
        contract_cid: cid_bytes(COMPILED_CONTRACT),
        compiled_partition_bytes_cid: cid_bytes(COMPILED_PARTITION),
    })
}

fn repository_root() -> TestResult<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "compiled Cargo manifest path has no repository root".into())
}

fn verify_committed_implementation(identity: &ImplementationIdentity) -> TestResult {
    let repository = repository_root()?;
    let revision_check = Command::new("git")
        .arg("-C")
        .arg(&repository)
        .args([
            "cat-file",
            "-e",
            &format!("{}^{{commit}}", identity.revision),
        ])
        .output()?;
    if !revision_check.status.success() {
        return Err(format!(
            "compiled implementation revision {} is not a local git commit: {}",
            identity.revision,
            String::from_utf8_lossy(&revision_check.stderr).trim()
        )
        .into());
    }
    let head = Command::new("git")
        .arg("-C")
        .arg(&repository)
        .args(["rev-parse", "HEAD"])
        .output()?;
    let observed_head = String::from_utf8(head.stdout)?.trim().to_owned();
    if !head.status.success() || observed_head != identity.revision {
        return Err(format!(
            "compiled implementation revision {} is not the checked-out HEAD {observed_head}",
            identity.revision
        )
        .into());
    }
    let tracked_status = Command::new("git")
        .arg("-C")
        .arg(&repository)
        .args(["status", "--porcelain=v1", "--untracked-files=no"])
        .output()?;
    if !tracked_status.status.success() || !tracked_status.stdout.is_empty() {
        return Err(format!(
            "tracked implementation checkout is not clean at revision {}: {}",
            identity.revision,
            String::from_utf8_lossy(&tracked_status.stdout).trim()
        )
        .into());
    }
    for (path, compiled_cid) in [
        (
            "crates/uor-r4-core/src/helm_d_r4_attention.rs",
            identity.core_source_cid.as_str(),
        ),
        (
            "crates/uor-r4-core/tests/intrinsic_lorentz_r4_attention_973.rs",
            identity.harness_source_cid.as_str(),
        ),
        (
            "crates/uor-r4-model-source/src/attention.rs",
            identity.model_attention_source_cid.as_str(),
        ),
        (
            "crates/uor-r4-model-source/src/lib.rs",
            identity.model_source_cid.as_str(),
        ),
        (
            "crates/uor-r4-model-source/src/exact_executor.rs",
            identity.exact_executor_source_cid.as_str(),
        ),
        (
            "docs/intrinsic_lorentz_r4_attention_manifest_973.json",
            identity.contract_cid.as_str(),
        ),
        (
            "docs/intrinsic_lorentz_r4_attention_partition_973.json",
            identity.compiled_partition_bytes_cid.as_str(),
        ),
    ] {
        let committed = Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args(["show", &format!("{}:{path}", identity.revision)])
            .output()?;
        if !committed.status.success() {
            return Err(format!(
                "implementation source {path} is absent from commit {}: {}",
                identity.revision,
                String::from_utf8_lossy(&committed.stderr).trim()
            )
            .into());
        }
        let committed_cid = cid_bytes(&committed.stdout);
        if committed_cid != compiled_cid {
            return Err(format!(
                "compiled source {path} differs from commit {}: compiled {compiled_cid}, committed {committed_cid}",
                identity.revision
            )
            .into());
        }
    }
    Ok(())
}

fn canonical_result_path(manifest: &FrozenPartitionManifest) -> TestResult<PathBuf> {
    let partition_digest = manifest
        .partition_cid
        .strip_prefix("blake3:")
        .filter(|digest| {
            digest.len() == 64
                && digest
                    .as_bytes()
                    .iter()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
        })
        .ok_or("partition CID cannot name the canonical evidence ledger")?;
    Ok(Path::new(CANONICAL_EVIDENCE_ROOT)
        .join(partition_digest)
        .join(ATTEMPT_TWO_RESULT_FILE))
}

fn acquire_partition_run_lock(output_path: &Path) -> TestResult<PartitionRunLock> {
    let directory = output_path
        .parent()
        .ok_or("canonical result path has no evidence directory")?;
    fs::create_dir_all(directory)?;
    let lock_path = directory.join("run.lock");
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)?;
    file.try_lock().map_err(|error| {
        format!(
            "another process holds the partition-scoped intrinsic R4 run lock {}: {error}",
            lock_path.display()
        )
    })?;
    Ok(PartitionRunLock { _file: file })
}

fn sidecar_path(result_path: &Path, suffix: &str) -> PathBuf {
    let mut path = result_path.as_os_str().to_os_string();
    path.push(suffix);
    PathBuf::from(path)
}

fn partition_reveal_path(result_path: &Path) -> TestResult<PathBuf> {
    let directory = result_path
        .parent()
        .ok_or("canonical result path has no evidence directory")?;
    Ok(directory.join("d3-revealed.json"))
}

fn ensure_fresh_run_target(output_path: &Path) -> TestResult {
    if output_path.exists() {
        return Err(format!(
            "refusing to overwrite existing intrinsic R4 result: {}",
            output_path.display()
        )
        .into());
    }
    let reveal_path = partition_reveal_path(output_path)?;
    if reveal_path.exists() {
        return Err(format!(
            "refusing to refit or rerun after durable D3 reveal marker: {}",
            reveal_path.display()
        )
        .into());
    }
    Ok(())
}

fn reconcile_interrupted_reveal(
    output_path: &Path,
    deadline: &ExperimentDeadline,
    manifest_envelope: &FrozenManifestEnvelope,
) -> TestResult<Option<String>> {
    let reveal_path = partition_reveal_path(output_path)?;
    if !reveal_path.exists() {
        return Ok(None);
    }
    if output_path.exists()
        && valid_post_reveal_terminal_cid(output_path, manifest_envelope, &reveal_path)?.is_some()
    {
        return Ok(None);
    }
    let reconciliation_path = if output_path.exists() {
        sidecar_path(output_path, ".post-reveal-invalid.json")
    } else {
        output_path.to_path_buf()
    };
    if reconciliation_path.exists() {
        if let Some(result_cid) =
            valid_post_reveal_terminal_cid(&reconciliation_path, manifest_envelope, &reveal_path)?
        {
            return Ok(Some(result_cid));
        }
        return Err(format!(
            "existing post-reveal reconciliation is not a valid bound terminal: {}",
            reconciliation_path.display()
        )
        .into());
    }
    deadline.check("startup.reconcile_interrupted_post_reveal")?;
    deadline.mark_heldout_opened();
    let reveal_cid = canonical_json_file_cid(&reveal_path)
        .unwrap_or_else(|_| "UNREADABLE_DURABLE_REVEAL_MARKER".to_owned());
    let error = std::io::Error::other(format!(
        "durable D3 reveal marker {reveal_cid} for manifest {} and partition {} exists without a structurally valid terminal result; prior run was interrupted after admission and D3 will not be reopened",
        manifest_envelope.manifest_cid, manifest_envelope.manifest.partition_cid
    ));
    let result_cid = write_failure(
        &reconciliation_path,
        output_path,
        deadline,
        &error,
        manifest_envelope,
    )?;
    if valid_post_reveal_terminal_cid(&reconciliation_path, manifest_envelope, &reveal_path)?
        .as_deref()
        != Some(result_cid.as_str())
    {
        return Err(format!(
            "published post-reveal reconciliation failed its own binding validation: {}",
            reconciliation_path.display()
        )
        .into());
    }
    Ok(Some(result_cid))
}

fn valid_post_reveal_terminal_cid(
    path: &Path,
    manifest_envelope: &FrozenManifestEnvelope,
    reveal_path: &Path,
) -> TestResult<Option<String>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(None),
    };
    let value: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let Some(result_value) = value.get("result") else {
        return Ok(None);
    };
    let Some(result) = result_value.as_object() else {
        return Ok(None);
    };
    let issue_valid = result
        .get("issue")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|issue| issue == 973);
    let Some(terminal) = result.get("terminal").and_then(serde_json::Value::as_str) else {
        return Ok(None);
    };
    let regular_terminal = [PASS_TERMINAL, RETAIN_TERMINAL, FAIL_TERMINAL].contains(&terminal);
    let invalid_terminal = terminal == POST_REVEAL_INVALID_TERMINAL;
    let schema_valid = result
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|schema| {
            (regular_terminal && schema == "uor-r4.intrinsic-lorentz-r4-attention-result/1")
                || (invalid_terminal
                    && schema == "uor-r4.intrinsic-lorentz-r4-attention-invalid-post-reveal/1")
        });
    let manifest_valid = result
        .get("manifest_cid")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|cid| cid == manifest_envelope.manifest_cid);
    let partition_valid = result
        .get("partition_cid")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|cid| cid == manifest_envelope.manifest.partition_cid);
    let Some((reveal_cid, reveal_value)) = valid_reveal_marker(reveal_path, manifest_envelope)?
    else {
        return Ok(None);
    };
    let reveal_valid = result
        .get("d3_reveal_marker_cid")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|cid| cid == reveal_cid);
    let reveal_payload_valid = if regular_terminal {
        result.get("fit_checkpoint_cid") == reveal_value.get("fit_checkpoint_cid")
            && result.get("implementation_identity") == reveal_value.get("implementation_identity")
    } else {
        true
    };
    let terminal_shape_valid = (regular_terminal
        && result
            .get("heldout")
            .is_some_and(|heldout| !heldout.is_null()))
        || (invalid_terminal
            && result
                .get("heldout_opened")
                .and_then(serde_json::Value::as_bool)
                .is_some_and(|opened| opened));
    let Some(declared_result_cid) = value.get("result_cid").and_then(serde_json::Value::as_str)
    else {
        return Ok(None);
    };
    let computed_result_cid = canonical_json_cid(result_value)?;
    let valid = issue_valid
        && (regular_terminal || invalid_terminal)
        && schema_valid
        && manifest_valid
        && partition_valid
        && reveal_valid
        && reveal_payload_valid
        && terminal_shape_valid
        && is_blake3_cid(declared_result_cid)
        && declared_result_cid == computed_result_cid;
    Ok(valid.then(|| declared_result_cid.to_owned()))
}

fn valid_reveal_marker(
    path: &Path,
    manifest_envelope: &FrozenManifestEnvelope,
) -> TestResult<Option<(String, serde_json::Value)>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(None),
    };
    let value: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let Some(marker) = value.as_object() else {
        return Ok(None);
    };
    let valid = marker
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|schema| schema == "uor-r4.intrinsic-lorentz-r4-attention-d3-reveal/1")
        && marker
            .get("issue")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|issue| issue == 973)
        && marker
            .get("manifest_cid")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|cid| cid == manifest_envelope.manifest_cid)
        && marker
            .get("partition_cid")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|cid| cid == manifest_envelope.manifest.partition_cid)
        && marker
            .get("fit_checkpoint_cid")
            .and_then(serde_json::Value::as_str)
            .is_some_and(is_blake3_cid)
        && marker
            .get("implementation_identity")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|identity| !identity.is_empty());
    if !valid {
        return Ok(None);
    }
    Ok(Some((canonical_json_cid(&value)?, value)))
}

fn prepare_fit_checkpoint(
    checkpoint_path: &Path,
    oracle: &mut HuggingFaceLlamaOracle,
    corpus_path: &Path,
    tokenizer: &Tokenizer,
    manifest_envelope: &FrozenManifestEnvelope,
    implementation_identity: &ImplementationIdentity,
    deadline: &ExperimentDeadline,
) -> TestResult<FitStageOutcome> {
    if checkpoint_path.exists() {
        deadline.check("construction.fit.checkpoint_resume")?;
        let envelope =
            read_fit_checkpoint(checkpoint_path, manifest_envelope, implementation_identity)?;
        return fit_stage_from_checkpoint(envelope, true);
    }

    let fit_documents = materialize_committed_documents(
        corpus_path,
        tokenizer,
        &manifest_envelope.manifest.construction_fit,
        PartitionAccess::Construction,
        deadline,
        "construction.fit.materialize",
    )?;
    let fit_captures = capture_documents(
        oracle,
        &fit_documents,
        deadline,
        "construction.fit.trace_capture",
    )?;
    let curved_fit = fit_intrinsic_parameters(
        &fit_captures,
        IntrinsicR4AttentionMetric::Lorentz,
        deadline,
        "construction.fit.curved.primary",
    )?;
    let curved_replay = fit_intrinsic_parameters(
        &fit_captures,
        IntrinsicR4AttentionMetric::Lorentz,
        deadline,
        "construction.fit.curved.replay",
    )?;
    let flat_fit = fit_intrinsic_parameters(
        &fit_captures,
        IntrinsicR4AttentionMetric::Flat,
        deadline,
        "construction.fit.flat.primary",
    )?;
    let flat_replay = fit_intrinsic_parameters(
        &fit_captures,
        IntrinsicR4AttentionMetric::Flat,
        deadline,
        "construction.fit.flat.replay",
    )?;
    let replay = fit_replay_evidence(&curved_fit, &curved_replay, &flat_fit, &flat_replay)?;
    let checkpoint = FitCheckpoint {
        schema: "uor-r4.intrinsic-lorentz-r4-attention-fit-checkpoint/1".to_owned(),
        issue: 973,
        manifest_cid: manifest_envelope.manifest_cid.clone(),
        partition_cid: manifest_envelope.manifest.partition_cid.clone(),
        implementation_identity: implementation_identity.clone(),
        curved_parameters: curved_fit.parameters,
        flat_parameters: flat_fit.parameters,
        curved_fit: curved_fit.report,
        flat_fit: flat_fit.report,
        replay,
    };
    drop(fit_captures);
    deadline.check("construction.fit.checkpoint_write")?;
    let written_cid = write_fit_checkpoint(checkpoint_path, checkpoint)?;
    let envelope =
        read_fit_checkpoint(checkpoint_path, manifest_envelope, implementation_identity)?;
    if envelope.checkpoint_cid != written_cid {
        return Err("fit checkpoint changed between exclusive write and readback".into());
    }
    fit_stage_from_checkpoint(envelope, false)
}

fn fit_replay_evidence(
    curved_fit: &FittedArm,
    curved_replay: &FittedArm,
    flat_fit: &FittedArm,
    flat_replay: &FittedArm,
) -> TestResult<FitReplayEvidence> {
    let parameter_replay_exact = curved_fit.parameter_json == curved_replay.parameter_json
        && flat_fit.parameter_json == flat_replay.parameter_json
        && curved_fit.report.parameter_json_cid == curved_replay.report.parameter_json_cid
        && flat_fit.report.parameter_json_cid == flat_replay.report.parameter_json_cid;
    let fit_report_replay_exact = serde_json::to_vec(&curved_fit.report)?
        == serde_json::to_vec(&curved_replay.report)?
        && serde_json::to_vec(&flat_fit.report)? == serde_json::to_vec(&flat_replay.report)?;
    let fit_work_and_shape_valid = fit_work_and_shape_valid(curved_fit)
        && fit_work_and_shape_valid(curved_replay)
        && fit_work_and_shape_valid(flat_fit)
        && fit_work_and_shape_valid(flat_replay);
    Ok(FitReplayEvidence {
        parameter_replay_exact,
        fit_report_replay_exact,
        fit_work_and_shape_valid,
        curved_primary_parameter_cid: curved_fit.report.parameter_json_cid.clone(),
        curved_replay_parameter_cid: curved_replay.report.parameter_json_cid.clone(),
        flat_primary_parameter_cid: flat_fit.report.parameter_json_cid.clone(),
        flat_replay_parameter_cid: flat_replay.report.parameter_json_cid.clone(),
        curved_primary_fit_report_cid: curved_fit.report.fit_report_cid.clone(),
        curved_replay_fit_report_cid: curved_replay.report.fit_report_cid.clone(),
        flat_primary_fit_report_cid: flat_fit.report.fit_report_cid.clone(),
        flat_replay_fit_report_cid: flat_replay.report.fit_report_cid.clone(),
    })
}

fn write_fit_checkpoint(path: &Path, checkpoint: FitCheckpoint) -> TestResult<String> {
    let checkpoint_cid = cid_bytes(&serde_json::to_vec(&checkpoint)?);
    let envelope = FitCheckpointEnvelope {
        checkpoint_cid: checkpoint_cid.clone(),
        checkpoint,
    };
    write_pretty_json_exclusive(path, &envelope)?;
    Ok(checkpoint_cid)
}

fn read_fit_checkpoint(
    path: &Path,
    manifest_envelope: &FrozenManifestEnvelope,
    implementation_identity: &ImplementationIdentity,
) -> TestResult<FitCheckpointEnvelope> {
    let envelope: FitCheckpointEnvelope = serde_json::from_slice(&fs::read(path)?)?;
    let observed_cid = cid_bytes(&serde_json::to_vec(&envelope.checkpoint)?);
    if envelope.checkpoint_cid != observed_cid {
        return Err(format!(
            "fit checkpoint CID mismatch: declared {}, observed {observed_cid}",
            envelope.checkpoint_cid
        )
        .into());
    }
    let checkpoint = &envelope.checkpoint;
    if checkpoint.schema != "uor-r4.intrinsic-lorentz-r4-attention-fit-checkpoint/1"
        || checkpoint.issue != 973
        || checkpoint.manifest_cid != manifest_envelope.manifest_cid
        || checkpoint.partition_cid != manifest_envelope.manifest.partition_cid
        || checkpoint.implementation_identity != *implementation_identity
    {
        return Err(
            "fit checkpoint contract, partition, or implementation identity mismatch".into(),
        );
    }
    validate_checkpoint_arm(
        &checkpoint.curved_parameters,
        &checkpoint.curved_fit,
        IntrinsicR4AttentionMetric::Lorentz,
    )?;
    validate_checkpoint_arm(
        &checkpoint.flat_parameters,
        &checkpoint.flat_fit,
        IntrinsicR4AttentionMetric::Flat,
    )?;
    let replay = &checkpoint.replay;
    if !replay.parameter_replay_exact
        || !replay.fit_report_replay_exact
        || !replay.fit_work_and_shape_valid
        || replay.curved_primary_parameter_cid != checkpoint.curved_fit.parameter_json_cid
        || replay.curved_replay_parameter_cid != checkpoint.curved_fit.parameter_json_cid
        || replay.flat_primary_parameter_cid != checkpoint.flat_fit.parameter_json_cid
        || replay.flat_replay_parameter_cid != checkpoint.flat_fit.parameter_json_cid
        || replay.curved_primary_fit_report_cid != checkpoint.curved_fit.fit_report_cid
        || replay.curved_replay_fit_report_cid != checkpoint.curved_fit.fit_report_cid
        || replay.flat_primary_fit_report_cid != checkpoint.flat_fit.fit_report_cid
        || replay.flat_replay_fit_report_cid != checkpoint.flat_fit.fit_report_cid
    {
        return Err("fit checkpoint does not preserve an exact valid replay identity".into());
    }
    Ok(envelope)
}

fn validate_checkpoint_arm(
    parameters: &IntrinsicLorentzR4AttentionParameters,
    report: &FitReport,
    expected_metric: IntrinsicR4AttentionMetric,
) -> TestResult {
    let validated_parameters = IntrinsicLorentzR4AttentionParameters::new(
        parameters.layers(),
        parameters.heads(),
        parameters.blocks_per_head(),
        parameters.score_coefficients().to_vec(),
        parameters.output_block_scales().to_vec(),
    )?;
    let parameter_json = serde_json::to_vec(&validated_parameters)?;
    if report.metric != expected_metric
        || report.parameter_json_cid != cid_bytes(&parameter_json)
        || !report.row_centered_objective.is_finite()
    {
        return Err("fit checkpoint parameter or objective identity mismatch".into());
    }
    let mut unhashed_report = report.clone();
    let declared_report_cid = std::mem::take(&mut unhashed_report.fit_report_cid);
    if declared_report_cid != cid_bytes(&serde_json::to_vec(&unhashed_report)?) {
        return Err("fit checkpoint report CID mismatch".into());
    }
    let fitted = FittedArm {
        parameters: validated_parameters,
        report: report.clone(),
        parameter_json,
    };
    if !fit_work_and_shape_valid(&fitted) {
        return Err("fit checkpoint work or parameter shape mismatch".into());
    }
    Ok(())
}

fn fit_stage_from_checkpoint(
    envelope: FitCheckpointEnvelope,
    resumed: bool,
) -> TestResult<FitStageOutcome> {
    let checkpoint_cid = envelope.checkpoint_cid;
    let checkpoint = envelope.checkpoint;
    let curved_parameter_json = serde_json::to_vec(&checkpoint.curved_parameters)?;
    let flat_parameter_json = serde_json::to_vec(&checkpoint.flat_parameters)?;
    Ok(FitStageOutcome {
        curved: FittedArm {
            parameters: checkpoint.curved_parameters,
            report: checkpoint.curved_fit,
            parameter_json: curved_parameter_json,
        },
        flat: FittedArm {
            parameters: checkpoint.flat_parameters,
            report: checkpoint.flat_fit,
            parameter_json: flat_parameter_json,
        },
        replay: checkpoint.replay,
        checkpoint_cid,
        resumed,
    })
}

fn require_file(path: &Path) -> TestResult {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("required file is unavailable: {}", path.display()).into())
    }
}

fn verify_corpus(corpus_path: &Path) -> TestResult {
    let manifest_path = corpus_path.with_file_name("manifest.json");
    require_file(&manifest_path)?;
    let manifest: CorpusManifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    if manifest.article_count != CORPUS_DOCUMENTS || manifest.corpus_cid != CORPUS_CID {
        return Err(format!(
            "corpus manifest mismatch: count={} cid={}",
            manifest.article_count, manifest.corpus_cid
        )
        .into());
    }
    let observed = file_cid(corpus_path)?;
    if observed != CORPUS_CID {
        return Err(format!(
            "corpus byte CID mismatch: expected {CORPUS_CID}, observed {observed}"
        )
        .into());
    }
    Ok(())
}

fn file_cid(path: &Path) -> TestResult<String> {
    let mut input = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn selection_digest(id: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.intrinsic-lorentz-r4-attention/1\0");
    hasher.update(id.as_bytes());
    *hasher.finalize().as_bytes()
}

fn freeze_document(
    article: Article,
    digest: [u8; 32],
    tokens: &[u32],
    corpus_byte_offset: u64,
    corpus_byte_length: u64,
) -> TestResult<FrozenDocumentCommitment> {
    if tokens.len() != REQUIRED_TOKENS {
        return Err("frozen document must contain exactly 17 tokens".into());
    }
    if corpus_byte_length == 0 || corpus_byte_offset.checked_add(corpus_byte_length).is_none() {
        return Err("frozen document corpus span is empty or overflows".into());
    }
    Ok(FrozenDocumentCommitment {
        id: article.id,
        title: article.title,
        selection_digest: format!("blake3:{}", hex::encode(digest)),
        token_cid: token_cid(b"uor-r4.intrinsic.tokens/1", tokens),
        input_cid: token_cid(b"uor-r4.intrinsic.inputs/1", &tokens[..INPUT_POSITIONS]),
        target_cid: token_cid(
            b"uor-r4.intrinsic.targets/1",
            &tokens[SCORE_START + 1..INPUT_POSITIONS + 1],
        ),
        corpus_byte_offset,
        corpus_byte_length,
    })
}

fn token_cid(domain: &[u8], tokens: &[u32]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(
        &u64::try_from(tokens.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for token in tokens {
        hasher.update(&token.to_le_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn sort_candidates(candidates: &mut [Candidate]) {
    candidates.sort_by(|left, right| {
        left.selection_digest
            .cmp(&right.selection_digest)
            .then_with(|| {
                left.document
                    .id
                    .as_bytes()
                    .cmp(right.document.id.as_bytes())
            })
    });
}

fn take_candidates(
    candidates: Vec<Candidate>,
    count: usize,
    label: &str,
) -> TestResult<Vec<FrozenDocumentCommitment>> {
    if candidates.len() < count {
        return Err(format!(
            "{label} has {} eligible documents; {count} required",
            candidates.len()
        )
        .into());
    }
    Ok(candidates
        .into_iter()
        .take(count)
        .map(|candidate| candidate.document)
        .collect())
}

fn validate_manifest_documents(manifest: &FrozenPartitionManifest) -> TestResult {
    if manifest.schema != PARTITION_SCHEMA
        || manifest.issue != 973
        || manifest.corpus_cid != CORPUS_CID
        || manifest.corpus_documents != CORPUS_DOCUMENTS
        || manifest.donor_source_cid != DONOR_CID
        || manifest.required_tokens_per_document != REQUIRED_TOKENS
        || manifest.input_positions != INPUT_POSITIONS
        || manifest.scored_positions != (SCORE_START..INPUT_POSITIONS).collect::<Vec<_>>()
        || manifest.construction_fit.len() != FIT_DOCUMENTS
        || manifest.construction_validation.len() != VALIDATION_DOCUMENTS
        || manifest.d3_heldout.len() != HELDOUT_DOCUMENTS
    {
        return Err("frozen intrinsic R4 manifest shape or binding mismatch".into());
    }
    if !is_blake3_cid(&manifest.tokenizer_cid)
        || !is_blake3_cid(&manifest.d3_target_commitment_cid)
        || manifest.d3_target_commitment_cid != aggregate_d3_target_commitment(&manifest.d3_heldout)
    {
        return Err("frozen intrinsic R4 manifest commitment mismatch".into());
    }
    let mut ids = HashSet::new();
    let mut inputs = HashSet::new();
    let mut spans = Vec::with_capacity(FIT_DOCUMENTS + VALIDATION_DOCUMENTS + HELDOUT_DOCUMENTS);
    for (partition, documents) in [
        ("fit", manifest.construction_fit.as_slice()),
        ("validation", manifest.construction_validation.as_slice()),
        ("heldout", manifest.d3_heldout.as_slice()),
    ] {
        let mut prior_key: Option<([u8; 32], Vec<u8>)> = None;
        for document in documents {
            if document.id.is_empty()
                || document.title.is_empty()
                || !is_blake3_cid(&document.token_cid)
                || !is_blake3_cid(&document.input_cid)
                || !is_blake3_cid(&document.target_cid)
            {
                return Err(format!(
                    "{partition} document {} commitment is malformed",
                    document.id
                )
                .into());
            }
            let digest = selection_digest(&document.id);
            if document.selection_digest != format!("blake3:{}", hex::encode(digest)) {
                return Err(format!(
                    "{partition} document {} selection digest mismatch",
                    document.id
                )
                .into());
            }
            let is_heldout = d3_is_held_out(&document.id);
            let partition_valid = match partition {
                "fit" => !is_heldout && !digest[0].is_multiple_of(5),
                "validation" => !is_heldout && digest[0].is_multiple_of(5),
                "heldout" => is_heldout && document.id != EXCLUDED_HELDOUT_ID,
                _ => false,
            };
            if !partition_valid {
                return Err(format!("document {} violates {partition} rule", document.id).into());
            }
            let key = (digest, document.id.as_bytes().to_vec());
            if prior_key.as_ref().is_some_and(|prior| prior >= &key) {
                return Err(format!("{partition} documents are not canonically ordered").into());
            }
            prior_key = Some(key);
            if !ids.insert(document.id.clone()) || !inputs.insert(document.input_cid.clone()) {
                return Err("duplicate selected document id or encoded-input CID".into());
            }
            let span_end = document
                .corpus_byte_offset
                .checked_add(document.corpus_byte_length)
                .filter(|_| document.corpus_byte_length > 0)
                .ok_or_else(|| {
                    format!(
                        "{partition} document {} has an invalid corpus span",
                        document.id
                    )
                })?;
            spans.push((document.corpus_byte_offset, span_end, document.id.as_str()));
        }
    }
    spans.sort_unstable_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.as_bytes().cmp(right.2.as_bytes()))
    });
    for pair in spans.windows(2) {
        if pair[0].1 > pair[1].0 {
            return Err(format!(
                "selected corpus spans overlap between documents {} and {}",
                pair[0].2, pair[1].2
            )
            .into());
        }
    }
    Ok(())
}

fn is_blake3_cid(value: &str) -> bool {
    value.strip_prefix("blake3:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .as_bytes()
                .iter()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
    })
}

fn aggregate_d3_target_commitment(documents: &[FrozenDocumentCommitment]) -> String {
    aggregate_target_commitment_cids(
        documents
            .iter()
            .map(|document| document.target_cid.as_str()),
    )
}

fn aggregate_materialized_d3_target_commitment(documents: &[FrozenDocument]) -> TestResult<String> {
    let target_cids = documents
        .iter()
        .map(|document| {
            if document.tokens.len() != REQUIRED_TOKENS {
                return Err(format!(
                    "materialized D3 document {} has {} tokens; {REQUIRED_TOKENS} required",
                    document.id,
                    document.tokens.len()
                )
                .into());
            }
            Ok(token_cid(
                b"uor-r4.intrinsic.targets/1",
                &document.tokens[SCORE_START + 1..INPUT_POSITIONS + 1],
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    Ok(aggregate_target_commitment_cids(
        target_cids.iter().map(String::as_str),
    ))
}

fn aggregate_target_commitment_cids<'a>(
    target_cids: impl ExactSizeIterator<Item = &'a str>,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.intrinsic.d3-target-commitments/1\0");
    hasher.update(
        &u64::try_from(target_cids.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for (index, target_cid) in target_cids.enumerate() {
        hasher.update(&u64::try_from(index).unwrap_or(u64::MAX).to_le_bytes());
        hasher.update(
            &u64::try_from(target_cid.len())
                .unwrap_or(u64::MAX)
                .to_le_bytes(),
        );
        hasher.update(target_cid.as_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn partition_cid(manifest: &FrozenPartitionManifest) -> TestResult<String> {
    let mut commitment = manifest.clone();
    commitment.partition_cid.clear();
    Ok(cid_bytes(&serde_json::to_vec(&commitment)?))
}

fn parse_frozen_manifest(bytes: &[u8]) -> TestResult<FrozenManifestEnvelope> {
    let envelope: FrozenManifestEnvelope = serde_json::from_slice(bytes)?;
    validate_manifest_documents(&envelope.manifest)?;
    let expected_partition_cid = partition_cid(&envelope.manifest)?;
    if envelope.manifest.partition_cid != expected_partition_cid {
        return Err(format!(
            "partition CID mismatch: declared {}, computed {expected_partition_cid}",
            envelope.manifest.partition_cid
        )
        .into());
    }
    let expected_manifest_cid = cid_bytes(&serde_json::to_vec(&envelope.manifest)?);
    if envelope.manifest_cid != expected_manifest_cid {
        return Err(format!(
            "manifest CID mismatch: declared {}, computed {expected_manifest_cid}",
            envelope.manifest_cid
        )
        .into());
    }
    Ok(envelope)
}

fn cid_bytes(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn canonical_json_bytes(value: &impl Serialize) -> TestResult<Vec<u8>> {
    let mut value = serde_json::to_value(value)?;
    sort_json_object_keys(&mut value);
    Ok(serde_json::to_vec(&value)?)
}

fn sort_json_object_keys(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                sort_json_object_keys(value);
            }
        }
        serde_json::Value::Object(object) => {
            let mut entries = std::mem::take(object).into_iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            for (key, mut value) in entries {
                sort_json_object_keys(&mut value);
                object.insert(key, value);
            }
        }
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {}
    }
}

fn canonical_json_cid(value: &impl Serialize) -> TestResult<String> {
    Ok(cid_bytes(&canonical_json_bytes(value)?))
}

fn canonical_json_file_cid(path: &Path) -> TestResult<String> {
    let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    canonical_json_cid(&value)
}

fn write_pretty_json(path: &Path, value: &impl Serialize) -> TestResult {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

fn write_pretty_json_exclusive(path: &Path, value: &impl Serialize) -> TestResult {
    let parent = path
        .parent()
        .ok_or("exclusive evidence path has no parent directory")?;
    fs::create_dir_all(parent)?;
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("evidence");
    let mut temporary_path = None;
    let mut temporary_file = None;
    for attempt in 0..16 {
        let candidate = parent.join(format!(
            ".{file_name}.tmp-{}-{nonce}-{attempt}",
            std::process::id()
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                temporary_path = Some(candidate);
                temporary_file = Some(file);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    let temporary_path = temporary_path.ok_or("could not reserve an atomic evidence temp file")?;
    let mut output = temporary_file.ok_or("atomic evidence temp file handle is unavailable")?;
    output.write_all(&bytes)?;
    output.sync_all()?;
    drop(output);
    if let Err(error) = fs::hard_link(&temporary_path, path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(error.into());
    }
    fs::remove_file(&temporary_path)?;
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}

fn write_content_addressed_exclusive(path: &Path, value: &impl Serialize) -> TestResult<String> {
    let content_cid = canonical_json_cid(value)?;
    write_pretty_json_exclusive(path, value)?;
    Ok(content_cid)
}

fn materialize_committed_documents(
    corpus_path: &Path,
    tokenizer: &Tokenizer,
    commitments: &[FrozenDocumentCommitment],
    access: PartitionAccess,
    deadline: &ExperimentDeadline,
    stage: &str,
) -> TestResult<Vec<FrozenDocument>> {
    deadline.check(stage)?;
    let mut corpus = fs::File::open(corpus_path)?;
    let corpus_byte_length = corpus.metadata()?.len();
    let mut documents = Vec::with_capacity(commitments.len());
    for commitment in commitments {
        deadline.check(stage)?;
        let record = read_committed_corpus_record(
            &mut corpus,
            corpus_byte_length,
            commitment,
            access,
            deadline.heldout_opened(),
        )?;
        documents.push(materialize_committed_document(
            tokenizer, commitment, &record, access,
        )?);
    }
    deadline.check(stage)?;
    Ok(documents)
}

fn read_committed_corpus_record<R: Read + Seek>(
    corpus: &mut R,
    corpus_byte_length: u64,
    commitment: &FrozenDocumentCommitment,
    access: PartitionAccess,
    heldout_opened: bool,
) -> TestResult<Vec<u8>> {
    if access == PartitionAccess::Heldout && !heldout_opened {
        return Err("heldout corpus materialization attempted before validation admission".into());
    }
    let is_heldout = d3_is_held_out(&commitment.id);
    match access {
        PartitionAccess::Construction if is_heldout => {
            return Err(format!(
                "construction materialization rejected D3 document {}",
                commitment.id
            )
            .into());
        }
        PartitionAccess::Heldout if !is_heldout || commitment.id == EXCLUDED_HELDOUT_ID => {
            return Err(format!(
                "heldout materialization rejected non-D3 or excluded document {}",
                commitment.id
            )
            .into());
        }
        _ => {}
    }
    let end = commitment
        .corpus_byte_offset
        .checked_add(commitment.corpus_byte_length)
        .filter(|end| commitment.corpus_byte_length > 0 && *end <= corpus_byte_length)
        .ok_or_else(|| format!("document {} corpus span is out of bounds", commitment.id))?;
    let byte_length = usize::try_from(commitment.corpus_byte_length).map_err(|_| {
        format!(
            "document {} corpus span exceeds address space",
            commitment.id
        )
    })?;
    let mut record = Vec::new();
    record.try_reserve_exact(byte_length).map_err(|error| {
        format!(
            "document {} corpus span allocation failed: {error}",
            commitment.id
        )
    })?;
    record.resize(byte_length, 0);
    corpus.seek(SeekFrom::Start(commitment.corpus_byte_offset))?;
    corpus.read_exact(&mut record)?;
    let observed_end = corpus.stream_position()?;
    if observed_end != end {
        return Err(format!(
            "document {} corpus span ended at {observed_end}; expected {end}",
            commitment.id
        )
        .into());
    }
    Ok(record)
}

fn materialize_committed_document(
    tokenizer: &Tokenizer,
    commitment: &FrozenDocumentCommitment,
    record: &[u8],
    access: PartitionAccess,
) -> TestResult<FrozenDocument> {
    let article: Article = serde_json::from_slice(record)?;
    if article.id != commitment.id || article.title != commitment.title {
        return Err(format!(
            "committed {:?} document identity mismatch: expected id={} title={:?}, observed id={} title={:?}",
            access, commitment.id, commitment.title, article.id, article.title
        )
        .into());
    }
    let digest = selection_digest(&article.id);
    if commitment.selection_digest != format!("blake3:{}", hex::encode(digest)) {
        return Err(format!(
            "committed {:?} document {} selection digest mismatch",
            access, article.id
        )
        .into());
    }
    let encoded = tokenizer.encode(&format!("{}\n\n{}", article.title, article.text));
    if encoded.len() < REQUIRED_TOKENS {
        return Err(format!(
            "committed {:?} document {} now has {} tokens; {REQUIRED_TOKENS} required",
            access,
            article.id,
            encoded.len()
        )
        .into());
    }
    let tokens = encoded[..REQUIRED_TOKENS].to_vec();
    let observed_token_cid = token_cid(b"uor-r4.intrinsic.tokens/1", &tokens);
    let observed_input_cid = token_cid(b"uor-r4.intrinsic.inputs/1", &tokens[..INPUT_POSITIONS]);
    let observed_target_cid = token_cid(
        b"uor-r4.intrinsic.targets/1",
        &tokens[SCORE_START + 1..INPUT_POSITIONS + 1],
    );
    if observed_token_cid != commitment.token_cid
        || observed_input_cid != commitment.input_cid
        || observed_target_cid != commitment.target_cid
    {
        return Err(format!(
            "committed {:?} document {} token/input/target CID mismatch",
            access, article.id
        )
        .into());
    }
    Ok(FrozenDocument {
        id: article.id,
        title: article.title,
        tokens,
    })
}

fn capture_documents(
    oracle: &mut HuggingFaceLlamaOracle,
    documents: &[FrozenDocument],
    deadline: &ExperimentDeadline,
    stage: &str,
) -> TestResult<Vec<CapturedDocument>> {
    let mut captures = Vec::with_capacity(documents.len());
    for document in documents {
        deadline.check(stage)?;
        captures.push(capture_document(oracle, document, deadline, stage)?);
    }
    deadline.check(stage)?;
    Ok(captures)
}

fn capture_document(
    oracle: &mut HuggingFaceLlamaOracle,
    document: &FrozenDocument,
    deadline: &ExperimentDeadline,
    stage: &str,
) -> TestResult<CapturedDocument> {
    deadline.check(stage)?;
    let layers = oracle.cfg().n_layers;
    let heads = oracle.cfg().n_heads;
    let dimension = oracle.cfg().dim;
    let kv_width = dimension * oracle.cfg().n_kv_heads / heads;
    let vocabulary = oracle.cfg().vocab;
    let maximum_token_id = u32::try_from(vocabulary.checked_sub(1).ok_or("empty vocabulary")?)?;
    let all_layers = (0..layers).collect::<Vec<_>>();
    let request = TraceCaptureRequest {
        residual_layers: &[],
        qkv_layers: &all_layers,
        attention_layers: &all_layers,
    };

    BehaviorSource::reset(oracle);
    let mut positions = Vec::with_capacity(INPUT_POSITIONS);
    let mut all_logits = Vec::with_capacity(INPUT_POSITIONS);
    let mut logits = vec![0.0; vocabulary];
    for position in 0..INPUT_POSITIONS {
        deadline.check(stage)?;
        let mut qkv_slots: Vec<Option<QkvCapture>> = vec![None; layers];
        let mut attention_slots: Vec<Vec<Option<Vec<f32>>>> = vec![vec![None; heads]; layers];
        let mut residual_sink = |_layer: usize, _residual: &[f32]| {};
        let mut qkv_sink = |layer: usize, query: &[f32], key: &[f32], value: &[f32]| {
            qkv_slots[layer] = Some(QkvCapture {
                query: query.to_vec(),
                key: key.to_vec(),
                value: value.to_vec(),
            });
        };
        let mut attention_sink = |layer: usize, head: usize, weights: &[f32]| {
            attention_slots[layer][head] = Some(weights.to_vec());
        };
        let captured = TeacherOracle::step_with_trace_capture(
            oracle,
            document.tokens[position] as usize,
            position,
            &mut logits,
            &request,
            &mut TraceCaptureSinks {
                residual: &mut residual_sink,
                qkv: &mut qkv_sink,
                attention: &mut attention_sink,
            },
        );
        deadline.check(stage)?;
        if !captured {
            return Err("frozen donor does not expose the required Q/K/V attention trace".into());
        }
        let mut layer_rows = Vec::with_capacity(layers);
        for layer in 0..layers {
            let qkv = qkv_slots[layer]
                .take()
                .ok_or_else(|| format!("missing Q/K/V capture at layer {layer}"))?;
            if qkv.query.len() != dimension
                || qkv.key.len() != kv_width
                || qkv.value.len() != kv_width
            {
                return Err(format!("Q/K/V capture shape mismatch at layer {layer}").into());
            }
            let attention = attention_slots[layer]
                .iter_mut()
                .enumerate()
                .map(|(head, weights)| {
                    let weights = weights.take().ok_or_else(|| {
                        format!("missing attention capture at layer {layer} head {head}")
                    })?;
                    if weights.len() != position + 1
                        || weights
                            .iter()
                            .any(|weight| !weight.is_finite() || *weight < 0.0)
                    {
                        return Err(format!(
                            "attention capture shape/value mismatch at layer {layer} head {head}"
                        ));
                    }
                    let sum = weights.iter().map(|weight| f64::from(*weight)).sum::<f64>();
                    if (sum - 1.0).abs() > 1.0e-6 {
                        return Err(format!(
                            "attention row sum {sum} at layer {layer} head {head} exceeds tolerance"
                        ));
                    }
                    Ok(weights)
                })
                .collect::<Result<Vec<_>, String>>()?;
            layer_rows.push(LayerCapture { qkv, attention });
        }
        positions.push(layer_rows);
        all_logits.push(logits.clone());
    }

    let mut atlas = R4SpinFrameAtlas::new(maximum_token_id, INPUT_POSITIONS)?;
    for position in 0..INPUT_POSITIONS {
        deadline.check(stage)?;
        atlas.begin_position(document.tokens[position], position)?;
    }
    deadline.check(stage)?;
    Ok(CapturedDocument {
        document: document.clone(),
        positions,
        logits: all_logits,
        atlas,
    })
}

fn geometric_row(
    document: &CapturedDocument,
    atlas: &mut R4SpinFrameAtlas,
    position: usize,
    layer: usize,
    head: usize,
    metric: IntrinsicR4AttentionMetric,
) -> TestResult<GeometricRow> {
    let prefix = position + 1;
    let query_capture = &document.positions[position][layer];
    let donor_weights = query_capture.attention[head]
        .iter()
        .map(|weight| f64::from(*weight))
        .collect::<Vec<_>>();
    if donor_weights.len() != prefix || donor_weights.iter().any(|weight| *weight <= 0.0) {
        return Err(format!(
            "donor attention row at document {} position {position} layer {layer} head {head} contains a zero/nonpositive weight",
            document.document.id
        )
        .into());
    }
    let query_base = head
        .checked_mul(EXPECTED_HEAD_WIDTH)
        .ok_or("query-head offset overflow")?;
    let kv_mul = EXPECTED_HEADS / EXPECTED_KV_HEADS;
    let kv_head = head / kv_mul;
    let kv_base = kv_head
        .checked_mul(EXPECTED_HEAD_WIDTH)
        .ok_or("KV-head offset overflow")?;
    let mut query_blocks = [[0.0; R4_WIDTH]; BLOCKS];
    for (block, query) in query_blocks.iter_mut().enumerate() {
        let offset = query_base + block * R4_WIDTH;
        *query = quantize_live_block(
            atlas.encode_model_block(position, read_block(&query_capture.qkv.query, offset)?)?,
        )?;
    }
    let mut features = Vec::with_capacity(prefix);
    let mut values = Vec::with_capacity(prefix);
    for source in 0..prefix {
        let source_capture = &document.positions[source][layer];
        let mut source_features = [0.0; BLOCKS];
        let mut source_values = [[0.0; R4_WIDTH]; BLOCKS];
        for block in 0..BLOCKS {
            let offset = kv_base + block * R4_WIDTH;
            let key_local =
                atlas.encode_model_block(source, read_block(&source_capture.qkv.key, offset)?)?;
            let key = quantize_live_block(atlas.transport_local_block(
                source,
                position,
                key_local,
                R4SpinTransportIntervention::Coherent,
                false,
            )?)?;
            source_features[block] = intrinsic_r4_score_feature(metric, query_blocks[block], key)?;

            let value_local =
                atlas.encode_model_block(source, read_block(&source_capture.qkv.value, offset)?)?;
            source_values[block] = quantize_live_block(atlas.transport_local_block(
                source,
                position,
                value_local,
                R4SpinTransportIntervention::Coherent,
                true,
            )?)?;
        }
        features.push(source_features);
        values.push(source_values);
    }
    Ok(GeometricRow {
        donor_weights,
        features,
        values,
    })
}

fn quantize_live_block(block: Vector4) -> TestResult<Vector4> {
    let mut quantized = [0.0; R4_WIDTH];
    for (target, source) in quantized.iter_mut().zip(block) {
        let live = source as f32;
        if !live.is_finite() {
            return Err("R4 block overflowed the live f32 attention seam".into());
        }
        *target = f64::from(live);
    }
    Ok(quantized)
}

fn read_block(values: &[f32], offset: usize) -> TestResult<Vector4> {
    let block = values
        .get(offset..offset + R4_WIDTH)
        .ok_or("R4 block exceeds captured vector")?;
    Ok([
        f64::from(block[0]),
        f64::from(block[1]),
        f64::from(block[2]),
        f64::from(block[3]),
    ])
}

fn fit_intrinsic_parameters(
    documents: &[CapturedDocument],
    metric: IntrinsicR4AttentionMetric,
    deadline: &ExperimentDeadline,
    stage: &str,
) -> TestResult<FittedArm> {
    deadline.check(stage)?;
    if documents.len() != FIT_DOCUMENTS {
        return Err("intrinsic fitter requires exactly 16 construction documents".into());
    }
    let layer_heads = EXPECTED_LAYERS
        .checked_mul(EXPECTED_HEADS)
        .ok_or("layer/head count overflow")?;
    let mut equations = vec![NormalEquation::default(); layer_heads];
    for document in documents {
        deadline.check(stage)?;
        let mut atlas = document.atlas.clone();
        for position in SCORE_START..INPUT_POSITIONS {
            deadline.check(stage)?;
            for layer in 0..EXPECTED_LAYERS {
                deadline.check(stage)?;
                for head in 0..EXPECTED_HEADS {
                    let row = geometric_row(document, &mut atlas, position, layer, head, metric)?;
                    accumulate_normal_equation(
                        &mut equations[layer * EXPECTED_HEADS + head],
                        &row,
                    )?;
                }
            }
        }
    }

    let parameter_count = layer_heads
        .checked_mul(BLOCKS)
        .ok_or("parameter count overflow")?;
    let mut score_coefficients = vec![0.0; parameter_count];
    for layer in 0..EXPECTED_LAYERS {
        deadline.check(stage)?;
        for head in 0..EXPECTED_HEADS {
            let equation = &equations[layer * EXPECTED_HEADS + head];
            let fitted = solve_nonnegative_coordinates(equation)?;
            let offset = (layer * EXPECTED_HEADS + head) * BLOCKS;
            for (target, coefficient) in score_coefficients[offset..offset + BLOCKS]
                .iter_mut()
                .zip(fitted)
            {
                *target = coefficient;
            }
        }
    }

    let mut numerator = vec![0.0; parameter_count];
    let mut denominator = vec![0.0; parameter_count];
    for document in documents {
        deadline.check(stage)?;
        let mut atlas = document.atlas.clone();
        for position in SCORE_START..INPUT_POSITIONS {
            deadline.check(stage)?;
            for layer in 0..EXPECTED_LAYERS {
                deadline.check(stage)?;
                for head in 0..EXPECTED_HEADS {
                    let row = geometric_row(document, &mut atlas, position, layer, head, metric)?;
                    let offset = (layer * EXPECTED_HEADS + head) * BLOCKS;
                    let weights = fitted_weights(
                        &row.features,
                        &score_coefficients[offset..offset + BLOCKS],
                    )?;
                    for block in 0..BLOCKS {
                        let values = row
                            .values
                            .iter()
                            .map(|source| source[block])
                            .collect::<Vec<_>>();
                        let centroid = intrinsic_r4_weighted_centroid(metric, &values, &weights)?;
                        let mut donor_aggregate = [0.0; R4_WIDTH];
                        for (value, weight) in values.iter().zip(&row.donor_weights) {
                            for lane in 0..R4_WIDTH {
                                donor_aggregate[lane] += *weight * value[lane];
                            }
                        }
                        for lane in 0..R4_WIDTH {
                            numerator[offset + block] += centroid[lane] * donor_aggregate[lane];
                            denominator[offset + block] += centroid[lane] * centroid[lane];
                        }
                    }
                }
            }
        }
    }
    deadline.check(stage)?;
    let output_scales = solve_output_scales(&numerator, &denominator)?;
    let parameters = IntrinsicLorentzR4AttentionParameters::new(
        EXPECTED_LAYERS,
        EXPECTED_HEADS,
        BLOCKS,
        score_coefficients,
        output_scales,
    )?;
    let parameter_json = serde_json::to_vec(&parameters)?;
    let parameter_json_cid = cid_bytes(&parameter_json);
    let causal_rows = equations.iter().map(|equation| equation.rows).sum::<u64>();
    let causal_source_pairs = equations
        .iter()
        .map(|equation| equation.source_pairs)
        .sum::<u64>();
    let geometric_row_evaluations = causal_rows
        .checked_mul(2)
        .ok_or("geometric-row work count overflow")?;
    let geometric_source_pair_evaluations = causal_source_pairs
        .checked_mul(2)
        .ok_or("geometric-source work count overflow")?;
    let feature_block_evaluations = geometric_source_pair_evaluations
        .checked_mul(u64::try_from(BLOCKS)?)
        .ok_or("feature-work count overflow")?;
    let centroid_source_block_evaluations = causal_source_pairs
        .checked_mul(u64::try_from(BLOCKS)?)
        .ok_or("centroid-work count overflow")?;
    let output_scale_lane_accumulations = causal_rows
        .checked_mul(u64::try_from(BLOCKS)?)
        .and_then(|value| value.checked_mul(u64::try_from(R4_WIDTH).ok()?))
        .ok_or("output-scale work count overflow")?;
    let coordinate_updates = u64::try_from(layer_heads)?
        .checked_mul(u64::try_from(BLOCKS)?)
        .and_then(|value| value.checked_mul(u64::try_from(NNLS_SWEEPS).ok()?))
        .ok_or("NNLS update count overflow")?;
    let objective = equations
        .iter()
        .enumerate()
        .map(|(index, equation)| {
            let offset = index * BLOCKS;
            centered_objective(
                equation,
                &parameters.score_coefficients()[offset..offset + BLOCKS],
            )
        })
        .sum::<f64>();
    if !objective.is_finite() {
        return Err("fitted row-centered objective is non-finite".into());
    }
    let construction_trace_cid = capture_trace_cid(documents);
    let mut report = FitReport {
        metric,
        construction_document_count: documents.len(),
        causal_rows,
        causal_source_pairs,
        geometric_row_evaluations,
        geometric_source_pair_evaluations,
        feature_block_evaluations,
        centroid_source_block_evaluations,
        output_scale_lane_accumulations,
        nnls_sweeps: NNLS_SWEEPS,
        nnls_coordinate_updates: coordinate_updates,
        ridge: RIDGE,
        coefficient_floor: COEFFICIENT_FLOOR,
        output_scale_floor: OUTPUT_SCALE_FLOOR,
        parameter_scalars: parameters.score_coefficients().len()
            + parameters.output_block_scales().len(),
        active_metric_coefficients: parameters
            .score_coefficients()
            .iter()
            .filter(|coefficient| **coefficient > 0.0)
            .count(),
        row_centered_objective: objective,
        construction_trace_cid,
        parameter_json_cid,
        fit_report_cid: String::new(),
    };
    report.fit_report_cid = cid_bytes(&serde_json::to_vec(&report)?);
    deadline.check(stage)?;
    Ok(FittedArm {
        parameters,
        report,
        parameter_json,
    })
}

fn solve_output_scales(numerator: &[f64], denominator: &[f64]) -> TestResult<Vec<f64>> {
    if numerator.is_empty() || numerator.len() != denominator.len() {
        return Err("output-scale normal equations are empty or misaligned".into());
    }
    numerator
        .iter()
        .zip(denominator)
        .enumerate()
        .map(|(index, (numerator, denominator))| {
            if !numerator.is_finite() || !denominator.is_finite() {
                return Err(format!("output-scale normal equation {index} is non-finite").into());
            }
            let scale = *numerator / (*denominator + RIDGE);
            if !scale.is_finite() {
                return Err(format!("fitted output scale {index} is non-finite").into());
            }
            Ok(scale.max(OUTPUT_SCALE_FLOOR))
        })
        .collect()
}

fn accumulate_normal_equation(equation: &mut NormalEquation, row: &GeometricRow) -> TestResult {
    let count = row.donor_weights.len();
    if count == 0 || row.features.len() != count {
        return Err("construction row is empty or misaligned".into());
    }
    let mut targets = row
        .donor_weights
        .iter()
        .map(|weight| libm::log(*weight))
        .collect::<Vec<_>>();
    if targets.iter().any(|target| !target.is_finite()) {
        return Err("row-centered donor log weight is non-finite".into());
    }
    let target_mean = targets.iter().sum::<f64>() / count as f64;
    for target in &mut targets {
        *target -= target_mean;
    }
    let mut feature_means = [0.0; BLOCKS];
    for features in &row.features {
        for block in 0..BLOCKS {
            feature_means[block] += features[block];
        }
    }
    for mean in &mut feature_means {
        *mean /= count as f64;
    }
    for (source, target) in row.features.iter().zip(targets) {
        let mut centered = [0.0; BLOCKS];
        for block in 0..BLOCKS {
            centered[block] = source[block] - feature_means[block];
            equation.correlation[block] += centered[block] * target;
        }
        for row_block in 0..BLOCKS {
            for column_block in 0..BLOCKS {
                equation.gram[row_block][column_block] +=
                    centered[row_block] * centered[column_block];
            }
        }
        equation.target_square += target * target;
    }
    equation.rows = equation.rows.saturating_add(1);
    equation.source_pairs = equation.source_pairs.saturating_add(u64::try_from(count)?);
    Ok(())
}

fn solve_nonnegative_coordinates(equation: &NormalEquation) -> TestResult<[f64; BLOCKS]> {
    let mut coefficients = [0.0; BLOCKS];
    for _ in 0..NNLS_SWEEPS {
        for coordinate in 0..BLOCKS {
            let mut other = 0.0;
            for (block, coefficient) in coefficients.iter().copied().enumerate() {
                if block != coordinate {
                    other += equation.gram[coordinate][block] * coefficient;
                }
            }
            let candidate = (equation.correlation[coordinate] - other)
                / (equation.gram[coordinate][coordinate] + RIDGE);
            if !candidate.is_finite() {
                return Err(format!(
                    "NNLS coordinate {coordinate} produced a non-finite candidate"
                )
                .into());
            }
            coefficients[coordinate] = candidate.max(0.0);
        }
    }
    Ok(coefficients)
}

fn centered_objective(equation: &NormalEquation, coefficients: &[f64]) -> f64 {
    let mut value = equation.target_square;
    for row in 0..BLOCKS {
        value -= 2.0 * coefficients[row] * equation.correlation[row];
        value += RIDGE * coefficients[row] * coefficients[row];
        for column in 0..BLOCKS {
            value += coefficients[row] * equation.gram[row][column] * coefficients[column];
        }
    }
    value
}

fn fitted_weights(features: &[[f64; BLOCKS]], coefficients: &[f64]) -> TestResult<Vec<f64>> {
    if features.is_empty() || coefficients.len() != BLOCKS {
        return Err("fitted softmax feature/coefficient shape mismatch".into());
    }
    let logits = features
        .iter()
        .map(|source| {
            source
                .iter()
                .zip(coefficients)
                .map(|(feature, coefficient)| feature * coefficient)
                .sum::<f64>()
        })
        .collect::<Vec<_>>();
    stable_softmax(&logits)
}

fn stable_softmax(logits: &[f64]) -> TestResult<Vec<f64>> {
    if logits.is_empty() || logits.iter().any(|logit| !logit.is_finite()) {
        return Err("softmax received empty or non-finite logits".into());
    }
    let mut scratch = logits.to_vec();
    let mut live_weights = vec![0.0f32; logits.len()];
    intrinsic_stable_softmax_into(&mut scratch, &mut live_weights)?;
    Ok(live_weights.into_iter().map(f64::from).collect())
}

fn capture_trace_cid(documents: &[CapturedDocument]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.intrinsic-construction-trace/1");
    for document in documents {
        hasher.update(document.document.id.as_bytes());
        for position in &document.positions {
            for layer in position {
                for value in layer
                    .qkv
                    .query
                    .iter()
                    .chain(&layer.qkv.key)
                    .chain(&layer.qkv.value)
                    .chain(layer.attention.iter().flatten())
                {
                    hasher.update(&value.to_bits().to_le_bytes());
                }
            }
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn run_arm(
    oracle: &HuggingFaceLlamaOracle,
    documents: &[FrozenDocument],
    spec: ArmSpec<'_>,
    arm: &str,
    deadline: &ExperimentDeadline,
    stage: &str,
) -> TestResult<ArmExecution> {
    deadline.check(stage)?;
    let mut reports = Vec::with_capacity(documents.len());
    let mut logits = Vec::with_capacity(documents.len());
    for document in documents {
        deadline.check(stage)?;
        let (report, rows) = match spec {
            ArmSpec::Donor => run_donor_document(oracle, document, deadline, stage)?,
            ArmSpec::Gauge => {
                run_transport_document(oracle, document, ArmSpec::Gauge, deadline, stage)?
            }
            ArmSpec::Intrinsic {
                parameters,
                metric,
                intervention,
            } => run_transport_document(
                oracle,
                document,
                ArmSpec::Intrinsic {
                    parameters,
                    metric,
                    intervention,
                },
                deadline,
                stage,
            )?,
        };
        reports.push(report);
        logits.push(rows);
    }
    deadline.check(stage)?;
    let scored_positions = reports
        .iter()
        .map(|document| document.positions.len())
        .sum::<usize>();
    if scored_positions != documents.len() * SCORE_POSITIONS {
        return Err("arm report omitted scored positions".into());
    }
    let total_nll = reports
        .iter()
        .flat_map(|document| &document.positions)
        .map(|position| position.target_nll_nats)
        .sum::<f64>();
    let top1_hits = reports.iter().map(|document| document.top1_hits).sum();
    let top8_hits = reports.iter().map(|document| document.top8_hits).sum();
    let mean_nll_nats = total_nll / scored_positions as f64;
    let perplexity = libm::exp(mean_nll_nats);
    if !total_nll.is_finite() || !mean_nll_nats.is_finite() || !perplexity.is_finite() {
        return Err("arm NLL or perplexity is non-finite".into());
    }
    let logits_cid = cid_bytes(&serde_json::to_vec(
        &reports
            .iter()
            .map(|document| &document.logits_cid)
            .collect::<Vec<_>>(),
    )?);
    let state_cid = cid_bytes(&serde_json::to_vec(
        &reports
            .iter()
            .map(|document| &document.state_cid)
            .collect::<Vec<_>>(),
    )?);
    let audit_cid = cid_bytes(&serde_json::to_vec(
        &reports
            .iter()
            .map(|document| &document.causal_audit)
            .collect::<Vec<_>>(),
    )?);
    let evidence_cid = cid_bytes(&serde_json::to_vec(
        &reports
            .iter()
            .map(|document| &document.implementation_evidence_cid)
            .collect::<Vec<_>>(),
    )?);
    Ok(ArmExecution {
        report: ArmReport {
            arm: arm.to_owned(),
            documents: reports,
            scored_positions,
            mean_nll_nats,
            perplexity,
            top1_hits,
            top8_hits,
            top1_accuracy: top1_hits as f64 / scored_positions as f64,
            top8_accuracy: top8_hits as f64 / scored_positions as f64,
            logits_cid,
            state_cid,
            audit_cid,
            evidence_cid,
        },
        logits,
    })
}

fn run_donor_document(
    oracle: &HuggingFaceLlamaOracle,
    document: &FrozenDocument,
    deadline: &ExperimentDeadline,
    stage: &str,
) -> TestResult<(DocumentResult, Vec<Vec<f32>>)> {
    deadline.check(stage)?;
    let mut state = oracle.new_state_bounded(INPUT_POSITIONS)?;
    let mut logits = vec![0.0; oracle.cfg().vocab];
    let mut results = Vec::with_capacity(SCORE_POSITIONS);
    let mut scored_logits = Vec::with_capacity(SCORE_POSITIONS);
    for position in 0..INPUT_POSITIONS {
        deadline.check(stage)?;
        oracle.step_state(
            &mut state,
            document.tokens[position] as usize,
            position,
            &mut logits,
        )?;
        deadline.check(stage)?;
        if position >= SCORE_START {
            results.push(position_result(document, position, &logits)?);
            scored_logits.push(logits.clone());
        }
    }
    let report = document_report(
        document,
        results,
        &scored_logits,
        state.persistent_state_cid(),
        None,
        None,
    )?;
    Ok((report, scored_logits))
}

fn run_transport_document(
    oracle: &HuggingFaceLlamaOracle,
    document: &FrozenDocument,
    spec: ArmSpec<'_>,
    deadline: &ExperimentDeadline,
    stage: &str,
) -> TestResult<(DocumentResult, Vec<Vec<f32>>)> {
    deadline.check(stage)?;
    let maximum_token_id = u32::try_from(oracle.cfg().vocab.checked_sub(1).ok_or("empty vocab")?)?;
    let transport: Box<dyn uor_r4_model_source::attention::CausalAttentionTransport> = match spec {
        ArmSpec::Gauge => Box::new(R4SpinCausalAttentionTransport::new(
            maximum_token_id,
            INPUT_POSITIONS,
            R4SpinTransportIntervention::Coherent,
        )?),
        ArmSpec::Intrinsic {
            parameters,
            metric,
            intervention,
        } => Box::new(IntrinsicR4CausalAttentionTransport::new(
            maximum_token_id,
            INPUT_POSITIONS,
            parameters.clone(),
            metric,
            intervention,
        )?),
        ArmSpec::Donor => return Err("donor cannot use the transport runner".into()),
    };
    let mut session = oracle.new_causal_attention_transport_session(
        transport,
        CausalAttentionLayerSelection::All,
        INPUT_POSITIONS,
    )?;
    let mut logits = vec![0.0; oracle.cfg().vocab];
    let mut results = Vec::with_capacity(SCORE_POSITIONS);
    let mut scored_logits = Vec::with_capacity(SCORE_POSITIONS);
    for position in 0..INPUT_POSITIONS {
        deadline.check(stage)?;
        oracle.step_causal_attention_transport(
            &mut session,
            document.tokens[position] as usize,
            position,
            &mut logits,
        )?;
        deadline.check(stage)?;
        if position >= SCORE_START {
            results.push(position_result(document, position, &logits)?);
            scored_logits.push(logits.clone());
        }
    }
    deadline.check(stage)?;
    session.transport_status()?;
    let causal_audit = session.audit();
    if causal_audit != expected_causal_audit(INPUT_POSITIONS)? {
        return Err(format!(
            "{} causal audit differs from exact 16-position work ledger",
            document.id
        )
        .into());
    }
    let evidence_text = session
        .transport_implementation_evidence()?
        .ok_or("transport omitted implementation evidence")?;
    let evidence: serde_json::Value = serde_json::from_str(&evidence_text)?;
    validate_implementation_evidence(&evidence, spec, INPUT_POSITIONS)?;
    let report = document_report(
        document,
        results,
        &scored_logits,
        session.persistent_state_cid(),
        Some(causal_audit),
        Some(evidence),
    )?;
    Ok((report, scored_logits))
}

fn document_report(
    document: &FrozenDocument,
    positions: Vec<PositionResult>,
    logits: &[Vec<f32>],
    state_cid: String,
    causal_audit: Option<CausalAttentionTransportAudit>,
    evidence: Option<serde_json::Value>,
) -> TestResult<DocumentResult> {
    if positions.is_empty() || logits.len() != positions.len() {
        return Err("document result has empty or misaligned scored positions".into());
    }
    let mean_nll_nats = positions
        .iter()
        .map(|position| position.target_nll_nats)
        .sum::<f64>()
        / positions.len() as f64;
    if !mean_nll_nats.is_finite() {
        return Err("document mean NLL is non-finite".into());
    }
    let top1_hits = positions
        .iter()
        .filter(|position| position.top1_hit)
        .count();
    let top8_hits = positions
        .iter()
        .filter(|position| position.top8_hit)
        .count();
    let implementation_evidence_cid = evidence
        .as_ref()
        .map(|evidence| serde_json::to_vec(evidence).map(|bytes| cid_bytes(&bytes)))
        .transpose()?;
    Ok(DocumentResult {
        document_id: document.id.clone(),
        title: document.title.clone(),
        positions,
        mean_nll_nats,
        top1_hits,
        top8_hits,
        logits_cid: logits_cid(logits),
        state_cid,
        causal_audit: causal_audit.map(AuditReport::from),
        implementation_evidence: evidence,
        implementation_evidence_cid,
    })
}

fn position_result(
    document: &FrozenDocument,
    position: usize,
    logits: &[f32],
) -> TestResult<PositionResult> {
    let target = *document
        .tokens
        .get(position + 1)
        .ok_or("scored position has no causal target token")?;
    let target_index = usize::try_from(target)?;
    validate_logit_row(logits, Some(target_index))?;
    let top1 = argmax(logits)?;
    Ok(PositionResult {
        query_position: position,
        input_token: document.tokens[position],
        target_token: target,
        target_nll_nats: cross_entropy(logits, target_index)?,
        top1_token: u32::try_from(top1).unwrap_or(u32::MAX),
        top1_hit: top1 == target_index,
        top8_hit: target_rank(logits, target_index)? <= 8,
    })
}

fn validate_logit_row(logits: &[f32], target: Option<usize>) -> TestResult {
    if logits.is_empty() || logits.iter().any(|value| !value.is_finite()) {
        return Err("decoder logit row is empty or non-finite".into());
    }
    if target.is_some_and(|target| target >= logits.len()) {
        return Err("decoder target token exceeds logit width".into());
    }
    Ok(())
}

fn argmax(values: &[f32]) -> TestResult<usize> {
    validate_logit_row(values, None)?;
    let mut winner = 0;
    for index in 1..values.len() {
        if values[index] > values[winner] {
            winner = index;
        }
    }
    Ok(winner)
}

fn target_rank(logits: &[f32], target: usize) -> TestResult<usize> {
    validate_logit_row(logits, Some(target))?;
    let target_value = logits[target];
    Ok(1 + logits
        .iter()
        .enumerate()
        .filter(|(index, value)| {
            **value > target_value || (**value == target_value && *index < target)
        })
        .count())
}

fn cross_entropy(logits: &[f32], target: usize) -> TestResult<f64> {
    validate_logit_row(logits, Some(target))?;
    let maximum = f64::from(logits.iter().copied().fold(f32::NEG_INFINITY, f32::max));
    let normalizer = logits
        .iter()
        .map(|logit| libm::exp(f64::from(*logit) - maximum))
        .sum::<f64>();
    if !normalizer.is_finite() || normalizer <= 0.0 {
        return Err("decoder logit normalizer is not positive and finite".into());
    }
    let log_normalizer = libm::log(normalizer) + maximum;
    let nll = log_normalizer - f64::from(logits[target]);
    if !nll.is_finite() || nll < 0.0 {
        return Err("decoder target NLL is invalid".into());
    }
    Ok(nll)
}

fn logits_cid(logits: &[Vec<f32>]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.intrinsic-full-vocabulary-logits/1");
    for row in logits {
        hasher.update(&u64::try_from(row.len()).unwrap_or(u64::MAX).to_le_bytes());
        for value in row {
            hasher.update(&value.to_bits().to_le_bytes());
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn expected_causal_audit(positions: usize) -> TestResult<CausalAttentionTransportAudit> {
    let positions = u64::try_from(positions)?;
    let layers = u64::try_from(EXPECTED_LAYERS)?;
    let heads = u64::try_from(EXPECTED_HEADS)?;
    let layer_calls = positions.checked_mul(layers).ok_or("layer-work overflow")?;
    let head_calls = layer_calls.checked_mul(heads).ok_or("head-work overflow")?;
    let sources = positions
        .checked_mul(positions + 1)
        .and_then(|value| value.checked_div(2))
        .and_then(|value| value.checked_mul(layers))
        .and_then(|value| value.checked_mul(heads))
        .ok_or("source-work overflow")?;
    Ok(CausalAttentionTransportAudit {
        positions,
        layers: layer_calls,
        heads: head_calls,
        query_transforms: head_calls,
        key_transports: sources,
        value_transports: sources,
        output_transforms: head_calls,
        future_reads: 0,
        maximum_query_position: Some(usize::try_from(positions)? - 1),
        maximum_source_position: Some(usize::try_from(positions)? - 1),
    })
}

fn expected_r4_audit(positions: usize, source_permuted: bool) -> TestResult<R4SpinTransportAudit> {
    let positions = u64::try_from(positions)?;
    let layer_heads = u64::try_from(EXPECTED_LAYERS * EXPECTED_HEADS)?;
    let blocks = u64::try_from(BLOCKS)?;
    let head_calls = positions
        .checked_mul(layer_heads)
        .ok_or("R4 head-work overflow")?;
    let query_blocks = head_calls
        .checked_mul(blocks)
        .ok_or("R4 query-block overflow")?;
    let prefix_sources = positions
        .checked_mul(positions + 1)
        .and_then(|value| value.checked_div(2))
        .ok_or("R4 prefix-work overflow")?;
    let source_blocks = prefix_sources
        .checked_mul(layer_heads)
        .and_then(|value| value.checked_mul(blocks))
        .ok_or("R4 source-block overflow")?;
    let source_frame_permutations = if source_permuted {
        prefix_sources
            .saturating_sub(1)
            .checked_mul(layer_heads)
            .and_then(|value| value.checked_mul(blocks))
            .and_then(|value| value.checked_mul(2))
            .ok_or("R4 source-permutation overflow")?
    } else {
        0
    };
    Ok(R4SpinTransportAudit {
        positions_prepared: positions,
        r4_blocks_encoded: query_blocks
            .checked_add(source_blocks.checked_mul(2).ok_or("R4 encode overflow")?)
            .ok_or("R4 encode overflow")?,
        key_blocks_transported: source_blocks,
        value_blocks_transported: source_blocks,
        output_blocks_decoded: query_blocks,
        future_position_reads: 0,
        source_frame_permutations,
    })
}

fn validate_implementation_evidence(
    evidence: &serde_json::Value,
    spec: ArmSpec<'_>,
    positions: usize,
) -> TestResult {
    match spec {
        ArmSpec::Gauge => {
            let evidence: R4SpinTransportEvidence = serde_json::from_value(evidence.clone())?;
            if evidence.schema != "uor-r4.r4-spin-transport-evidence/1"
                || evidence.policy_identity != HELM_D_R4_GAUGE_SOFTMAX_POLICY
                || evidence.intervention != R4SpinTransportIntervention::Coherent
                || evidence.frame_table_offsets.len() != positions
                || evidence.audit != expected_r4_audit(positions, false)?
            {
                return Err("gauge R4 implementation evidence differs from exact work".into());
            }
        }
        ArmSpec::Intrinsic {
            parameters,
            metric,
            intervention,
        } => {
            let evidence: IntrinsicR4AttentionEvidence = serde_json::from_value(evidence.clone())?;
            let parameter_identity = cid_bytes(&serde_json::to_vec(parameters)?);
            let source_permuted =
                intervention == IntrinsicR4AttentionIntervention::SourceFramePermuted;
            let positions_u64 = u64::try_from(positions)?;
            let layer_heads = u64::try_from(EXPECTED_LAYERS * EXPECTED_HEADS)?;
            let head_rows = positions_u64
                .checked_mul(layer_heads)
                .ok_or("intrinsic head-row overflow")?;
            let prefix_sources = positions_u64
                .checked_mul(positions_u64 + 1)
                .and_then(|value| value.checked_div(2))
                .and_then(|value| value.checked_mul(layer_heads))
                .ok_or("intrinsic source-row overflow")?;
            let score_blocks = prefix_sources
                .checked_mul(u64::try_from(BLOCKS)?)
                .ok_or("intrinsic score-block overflow")?;
            let centroid_blocks = head_rows
                .checked_mul(u64::try_from(BLOCKS)?)
                .ok_or("intrinsic centroid-block overflow")?;
            let value_permutations =
                if intervention == IntrinsicR4AttentionIntervention::ValuePermuted {
                    positions_u64
                        .checked_mul(positions_u64 + 1)
                        .and_then(|value| value.checked_div(2))
                        .map(|value| value.saturating_sub(1))
                        .and_then(|value| value.checked_mul(layer_heads))
                        .and_then(|value| value.checked_mul(u64::try_from(BLOCKS).ok()?))
                        .ok_or("intrinsic value-permutation overflow")?
                } else {
                    0
                };
            let expected_policy = match metric {
                IntrinsicR4AttentionMetric::Lorentz => INTRINSIC_LORENTZ_R4_ATTENTION_POLICY,
                IntrinsicR4AttentionMetric::Flat => INTRINSIC_FLAT_R4_ATTENTION_POLICY,
            };
            if evidence.schema != "uor-r4.intrinsic-r4-attention-evidence/1"
                || evidence.policy_identity != expected_policy
                || evidence.parameter_identity != parameter_identity
                || evidence.metric != metric
                || evidence.intervention != intervention
                || evidence.frame_table_offsets.len() != positions
                || evidence.transport_audit != expected_r4_audit(positions, source_permuted)?
                || evidence.intrinsic_audit.score_rows != head_rows
                || evidence.intrinsic_audit.compatibility_pairs != prefix_sources
                || evidence.intrinsic_audit.score_blocks != score_blocks
                || evidence.intrinsic_audit.centroid_rows != head_rows
                || evidence.intrinsic_audit.centroid_source_pairs != prefix_sources
                || evidence.intrinsic_audit.centroid_blocks != centroid_blocks
                || evidence.intrinsic_audit.value_permutations != value_permutations
                || evidence.intrinsic_audit.arithmetic_failures != 0
                || (metric == IntrinsicR4AttentionMetric::Flat
                    && evidence.intrinsic_audit.lorentz_domain_clamps != 0)
            {
                return Err("intrinsic R4 implementation evidence differs from exact work".into());
            }
        }
        ArmSpec::Donor => return Err("donor has no transport implementation evidence".into()),
    }
    Ok(())
}

fn compare_arms(
    reference: &ArmExecution,
    candidate: &ArmExecution,
    deadline: &ExperimentDeadline,
    stage: &str,
) -> TestResult<ArmComparison> {
    deadline.check(stage)?;
    if reference.report.documents.len() != candidate.report.documents.len()
        || reference.logits.len() != candidate.logits.len()
        || reference.report.documents.is_empty()
        || !reference.report.mean_nll_nats.is_finite()
        || !candidate.report.mean_nll_nats.is_finite()
    {
        return Err("arm comparison inputs are empty, misaligned, or non-finite".into());
    }
    let mut position_differences = Vec::new();
    let mut document_differences = Vec::new();
    let mut maximum_absolute = 0.0f64;
    let mut absolute_sum = 0.0f64;
    let mut compared_logits = 0u64;
    let mut total_kl = 0.0;
    let mut compared_rows = 0u64;
    for document_index in 0..reference.report.documents.len() {
        deadline.check(stage)?;
        let left = &reference.report.documents[document_index];
        let right = &candidate.report.documents[document_index];
        if left.document_id != right.document_id
            || left.positions.len() != right.positions.len()
            || reference.logits[document_index].len() != candidate.logits[document_index].len()
        {
            return Err("arm comparison document/position identity mismatch".into());
        }
        let document_delta = right.mean_nll_nats - left.mean_nll_nats;
        if !document_delta.is_finite() {
            return Err("arm comparison document NLL delta is non-finite".into());
        }
        document_differences.push(document_delta);
        for position_index in 0..left.positions.len() {
            deadline.check(stage)?;
            let left_position = &left.positions[position_index];
            let right_position = &right.positions[position_index];
            if left_position.query_position != right_position.query_position
                || left_position.target_token != right_position.target_token
            {
                return Err("arm comparison target identity mismatch".into());
            }
            let position_delta = right_position.target_nll_nats - left_position.target_nll_nats;
            if !position_delta.is_finite() {
                return Err("arm comparison position NLL delta is non-finite".into());
            }
            position_differences.push(PositionDifference {
                document_id: left.document_id.clone(),
                query_position: left_position.query_position,
                nll_delta_nats: position_delta,
            });
            let left_logits = &reference.logits[document_index][position_index];
            let right_logits = &candidate.logits[document_index][position_index];
            if left_logits.len() != right_logits.len() || left_logits.is_empty() {
                return Err("arm comparison logit widths differ".into());
            }
            total_kl += donor_kl(left_logits, right_logits)?;
            compared_rows = compared_rows.saturating_add(1);
            for (left_value, right_value) in left_logits.iter().zip(right_logits) {
                let difference = (f64::from(*right_value) - f64::from(*left_value)).abs();
                maximum_absolute = maximum_absolute.max(difference);
                absolute_sum += difference;
                compared_logits = compared_logits.saturating_add(1);
            }
        }
    }
    deadline.check(stage)?;
    if compared_rows == 0 || compared_logits == 0 {
        return Err("arm comparison performed no finite work".into());
    }
    let candidate_minus_reference_mean_nll =
        candidate.report.mean_nll_nats - reference.report.mean_nll_nats;
    let mean_donor_kl_nats = total_kl / compared_rows as f64;
    let mean_absolute_logit_delta = absolute_sum / compared_logits as f64;
    if !candidate_minus_reference_mean_nll.is_finite()
        || !total_kl.is_finite()
        || !mean_donor_kl_nats.is_finite()
        || !maximum_absolute.is_finite()
        || !mean_absolute_logit_delta.is_finite()
    {
        return Err("arm comparison aggregate is non-finite".into());
    }
    Ok(ArmComparison {
        reference_arm: reference.report.arm.clone(),
        candidate_arm: candidate.report.arm.clone(),
        candidate_minus_reference_mean_nll,
        candidate_worse_document_count: document_differences
            .iter()
            .filter(|difference| **difference > 0.0)
            .count(),
        reference_worse_document_count: document_differences
            .iter()
            .filter(|difference| **difference < 0.0)
            .count(),
        mean_donor_kl_nats,
        maximum_absolute_logit_delta: maximum_absolute,
        mean_absolute_logit_delta,
        position_nll_deltas: position_differences,
        document_nll_deltas: document_differences,
    })
}

fn donor_kl(donor: &[f32], candidate: &[f32]) -> TestResult<f64> {
    if donor.is_empty()
        || donor.len() != candidate.len()
        || donor
            .iter()
            .chain(candidate)
            .any(|value| !value.is_finite())
    {
        return Err("KL inputs are empty, misaligned, or non-finite".into());
    }
    let donor_max = f64::from(donor.iter().copied().fold(f32::NEG_INFINITY, f32::max));
    let candidate_max = f64::from(candidate.iter().copied().fold(f32::NEG_INFINITY, f32::max));
    let donor_sum = donor
        .iter()
        .map(|value| libm::exp(f64::from(*value) - donor_max))
        .sum::<f64>();
    let candidate_sum = candidate
        .iter()
        .map(|value| libm::exp(f64::from(*value) - candidate_max))
        .sum::<f64>();
    if !donor_sum.is_finite()
        || !candidate_sum.is_finite()
        || donor_sum <= 0.0
        || candidate_sum <= 0.0
    {
        return Err("KL normalizer is not positive and finite".into());
    }
    let donor_log_z = libm::log(donor_sum) + donor_max;
    let candidate_log_z = libm::log(candidate_sum) + candidate_max;
    let mut kl = 0.0;
    for (donor_logit, candidate_logit) in donor.iter().zip(candidate) {
        let log_p = f64::from(*donor_logit) - donor_log_z;
        let log_q = f64::from(*candidate_logit) - candidate_log_z;
        kl += libm::exp(log_p) * (log_p - log_q);
    }
    if !kl.is_finite() {
        return Err("KL divergence is non-finite".into());
    }
    Ok(kl)
}

fn maximum_live_attention_delta(
    documents: &[CapturedDocument],
    curved_parameters: &IntrinsicLorentzR4AttentionParameters,
    flat_parameters: &IntrinsicLorentzR4AttentionParameters,
    deadline: &ExperimentDeadline,
    stage: &str,
) -> TestResult<f64> {
    deadline.check(stage)?;
    let mut maximum = 0.0f64;
    for document in documents {
        deadline.check(stage)?;
        let mut curved_atlas = document.atlas.clone();
        let mut flat_atlas = document.atlas.clone();
        for position in SCORE_START..INPUT_POSITIONS {
            deadline.check(stage)?;
            for layer in 0..EXPECTED_LAYERS {
                deadline.check(stage)?;
                for head in 0..EXPECTED_HEADS {
                    let curved_row = geometric_row(
                        document,
                        &mut curved_atlas,
                        position,
                        layer,
                        head,
                        IntrinsicR4AttentionMetric::Lorentz,
                    )?;
                    let flat_row = geometric_row(
                        document,
                        &mut flat_atlas,
                        position,
                        layer,
                        head,
                        IntrinsicR4AttentionMetric::Flat,
                    )?;
                    let offset = (layer * EXPECTED_HEADS + head) * BLOCKS;
                    let curved_weights = fitted_weights(
                        &curved_row.features,
                        &curved_parameters.score_coefficients()[offset..offset + BLOCKS],
                    )?;
                    let flat_weights = fitted_weights(
                        &flat_row.features,
                        &flat_parameters.score_coefficients()[offset..offset + BLOCKS],
                    )?;
                    for (curved, flat) in curved_weights.iter().zip(&flat_weights) {
                        maximum = maximum.max((*curved - *flat).abs());
                    }
                }
            }
        }
    }
    Ok(maximum)
}

fn geometry_preflight(
    documents: &[CapturedDocument],
    parameters: &IntrinsicLorentzR4AttentionParameters,
    deadline: &ExperimentDeadline,
    stage: &str,
) -> TestResult<GeometryPreflight> {
    deadline.check(stage)?;
    let golden = helm_d_lorentz_causal_row(
        &[0.25, -0.5, 0.75],
        &[vec![0.1, -0.2, 0.3], vec![-0.4, 0.2, 0.6]],
        &[vec![0.7, 0.1, -0.2], vec![-0.3, 0.8, 0.4]],
        HelmDLorentzReferenceConfig {
            curvature: 1.0,
            learned_scale: 2.5,
            bias: -0.125,
        },
    )?;
    let helm_d_golden_reproduced = (golden.logits[0] + 0.214_615_321_377_075_56).abs() <= 1.0e-15
        && (golden.logits[1] + 0.493_210_510_118_965_55).abs() <= 1.0e-15
        && (golden.weights[0] - 0.569_201_782_148_292_1).abs() <= 1.0e-15
        && (golden.weights[1] - 0.430_798_217_851_707_9).abs() <= 1.0e-15;

    let mut exercised_blocks = 0u64;
    let mut maximum_hyperboloid_residual = 0.0f64;
    let mut maximum_distance_invariance_delta = 0.0f64;
    let mut maximum_barycenter_covariance_delta = 0.0f64;
    let mut minimum_timelike_denominator_squared = f64::INFINITY;
    let mut maximum_softmax_sum_delta = 0.0f64;
    for document in documents {
        deadline.check(stage)?;
        let mut atlas = document.atlas.clone();
        for position in SCORE_START..INPUT_POSITIONS {
            deadline.check(stage)?;
            for layer in 0..EXPECTED_LAYERS {
                deadline.check(stage)?;
                for head in 0..EXPECTED_HEADS {
                    let row = geometric_row(
                        document,
                        &mut atlas,
                        position,
                        layer,
                        head,
                        IntrinsicR4AttentionMetric::Lorentz,
                    )?;
                    let parameter_offset = (layer * EXPECTED_HEADS + head) * BLOCKS;
                    let curved_weights = fitted_weights(
                        &row.features,
                        &parameters.score_coefficients()
                            [parameter_offset..parameter_offset + BLOCKS],
                    )?;
                    let softmax_delta = (curved_weights.iter().sum::<f64>() - 1.0).abs();
                    maximum_softmax_sum_delta = maximum_softmax_sum_delta.max(softmax_delta);

                    let query_capture = &document.positions[position][layer];
                    let query_base = head * EXPECTED_HEAD_WIDTH;
                    let kv_head = head / (EXPECTED_HEADS / EXPECTED_KV_HEADS);
                    let kv_base = kv_head * EXPECTED_HEAD_WIDTH;
                    for block in 0..BLOCKS {
                        let query_model =
                            read_block(&query_capture.qkv.query, query_base + block * R4_WIDTH)?;
                        let query_gauge = atlas.encode_model_block(position, query_model)?;
                        maximum_hyperboloid_residual = maximum_hyperboloid_residual
                            .max(hyperboloid_residual(query_model))
                            .max(hyperboloid_residual(query_gauge));
                        let mut original_values = Vec::with_capacity(position + 1);
                        let mut transported_values = Vec::with_capacity(position + 1);
                        for source in 0..=position {
                            let source_capture = &document.positions[source][layer];
                            let key_model =
                                read_block(&source_capture.qkv.key, kv_base + block * R4_WIDTH)?;
                            let key_local = atlas.encode_model_block(source, key_model)?;
                            let key_gauge = atlas.transport_local_block(
                                source,
                                position,
                                key_local,
                                R4SpinTransportIntervention::Coherent,
                                false,
                            )?;
                            let original_feature = intrinsic_r4_score_feature(
                                IntrinsicR4AttentionMetric::Lorentz,
                                query_model,
                                key_model,
                            )?;
                            let gauge_feature = intrinsic_r4_score_feature(
                                IntrinsicR4AttentionMetric::Lorentz,
                                query_gauge,
                                key_gauge,
                            )?;
                            maximum_distance_invariance_delta = maximum_distance_invariance_delta
                                .max((original_feature - gauge_feature).abs());
                            maximum_hyperboloid_residual = maximum_hyperboloid_residual
                                .max(hyperboloid_residual(key_model))
                                .max(hyperboloid_residual(key_gauge));

                            let value_model =
                                read_block(&source_capture.qkv.value, kv_base + block * R4_WIDTH)?;
                            original_values.push(value_model);
                            transported_values.push(row.values[source][block]);
                            maximum_hyperboloid_residual = maximum_hyperboloid_residual
                                .max(hyperboloid_residual(value_model))
                                .max(hyperboloid_residual(row.values[source][block]));
                            exercised_blocks = exercised_blocks.saturating_add(1);
                        }
                        let original_centroid = intrinsic_r4_weighted_centroid(
                            IntrinsicR4AttentionMetric::Lorentz,
                            &original_values,
                            &curved_weights,
                        )?;
                        let expected_centroid =
                            atlas.encode_model_block(position, original_centroid)?;
                        let transported_centroid = intrinsic_r4_weighted_centroid(
                            IntrinsicR4AttentionMetric::Lorentz,
                            &transported_values,
                            &curved_weights,
                        )?;
                        for lane in 0..R4_WIDTH {
                            maximum_barycenter_covariance_delta =
                                maximum_barycenter_covariance_delta.max(
                                    (expected_centroid[lane] - transported_centroid[lane]).abs(),
                                );
                        }
                        minimum_timelike_denominator_squared = minimum_timelike_denominator_squared
                            .min(timelike_norm_squared(&transported_values, &curved_weights)?);
                    }
                }
            }
        }
    }
    deadline.check(stage)?;
    let passed = helm_d_golden_reproduced
        && exercised_blocks > 0
        && maximum_hyperboloid_residual <= 1.0e-9
        && maximum_distance_invariance_delta <= 1.0e-9
        && maximum_barycenter_covariance_delta <= 1.0e-8
        && minimum_timelike_denominator_squared >= 1.0e-12
        && maximum_softmax_sum_delta <= 1.0e-6;
    Ok(GeometryPreflight {
        exercised_blocks,
        maximum_hyperboloid_residual,
        maximum_distance_invariance_delta,
        maximum_barycenter_covariance_delta,
        minimum_timelike_denominator_squared,
        maximum_softmax_sum_delta,
        helm_d_golden_reproduced,
        passed,
    })
}

fn hyperboloid_residual(spatial: Vector4) -> f64 {
    let norm = spatial.iter().map(|value| value * value).sum::<f64>();
    let time = libm::sqrt(1.0 + norm);
    (time * time - norm - 1.0).abs()
}

fn timelike_norm_squared(values: &[Vector4], weights: &[f64]) -> TestResult<f64> {
    if values.len() != weights.len() || values.is_empty() {
        return Err("timelike norm inputs are empty or misaligned".into());
    }
    let weight_sum = weights.iter().sum::<f64>();
    let mut average = [0.0; R4_WIDTH + 1];
    for (value, weight) in values.iter().zip(weights) {
        let norm = value.iter().map(|lane| lane * lane).sum::<f64>();
        let normalized_weight = *weight / weight_sum;
        average[0] += normalized_weight * libm::sqrt(1.0 + norm);
        for lane in 0..R4_WIDTH {
            average[lane + 1] += normalized_weight * value[lane];
        }
    }
    let result =
        average[0] * average[0] - average[1..].iter().map(|value| value * value).sum::<f64>();
    if !result.is_finite() {
        return Err("timelike norm is non-finite".into());
    }
    Ok(result)
}

// Every argument is a separately frozen construction-gate evidence source.
#[allow(clippy::too_many_arguments)]
fn validation_gate(
    donor: &ArmExecution,
    curved: &ArmExecution,
    flat: &ArmExecution,
    validation_captures: &[CapturedDocument],
    curved_fit: &FittedArm,
    flat_fit: &FittedArm,
    fit_replay: &FitReplayEvidence,
    deadline: &ExperimentDeadline,
) -> TestResult<ValidationGateReport> {
    deadline.check("construction.validation.gate")?;
    let geometry_preflight = geometry_preflight(
        validation_captures,
        &curved_fit.parameters,
        deadline,
        "construction.validation.geometry_preflight",
    )?;
    let maximum_curved_vs_flat_attention_delta = maximum_live_attention_delta(
        validation_captures,
        &curved_fit.parameters,
        &flat_fit.parameters,
        deadline,
        "construction.validation.live_attention_delta",
    )?;
    let parameter_replay_exact = fit_replay.parameter_replay_exact
        && fit_replay.curved_primary_parameter_cid == curved_fit.report.parameter_json_cid
        && fit_replay.curved_replay_parameter_cid == curved_fit.report.parameter_json_cid
        && fit_replay.flat_primary_parameter_cid == flat_fit.report.parameter_json_cid
        && fit_replay.flat_replay_parameter_cid == flat_fit.report.parameter_json_cid;
    let fit_report_replay_exact = fit_replay.fit_report_replay_exact
        && fit_replay.curved_primary_fit_report_cid == curved_fit.report.fit_report_cid
        && fit_replay.curved_replay_fit_report_cid == curved_fit.report.fit_report_cid
        && fit_replay.flat_primary_fit_report_cid == flat_fit.report.fit_report_cid
        && fit_replay.flat_replay_fit_report_cid == flat_fit.report.fit_report_cid;
    let fit_work_and_shape_valid = fit_replay.fit_work_and_shape_valid
        && fit_work_and_shape_valid(curved_fit)
        && fit_work_and_shape_valid(flat_fit);
    let trace_matches_donor_decoder =
        validation_captures
            .iter()
            .enumerate()
            .all(|(document_index, capture)| {
                capture.logits[SCORE_START..INPUT_POSITIONS]
                    .iter()
                    .flatten()
                    .zip(donor.logits[document_index].iter().flatten())
                    .all(|(captured, replayed)| captured.to_bits() == replayed.to_bits())
            });
    let curved_minus_donor_nll = curved.report.mean_nll_nats - donor.report.mean_nll_nats;
    let curved_minus_flat_nll = curved.report.mean_nll_nats - flat.report.mean_nll_nats;
    let zero_faults_and_future_reads = [curved, flat].iter().all(|arm| {
        arm.report.documents.iter().all(|document| {
            document
                .causal_audit
                .as_ref()
                .is_some_and(|audit| audit.future_reads == 0)
        })
    });
    let mut failures = Vec::new();
    if !geometry_preflight.passed {
        failures.push("Lorentz/Spin numerical and covariance preflight failed".to_owned());
    }
    if !parameter_replay_exact || !fit_report_replay_exact {
        failures.push("parameter or fit-report replay was not byte-identical".to_owned());
    }
    if !trace_matches_donor_decoder {
        failures.push("Q/K/V trace executor logits differed from ordinary donor replay".to_owned());
    }
    if !fit_work_and_shape_valid {
        failures.push("fit work or 8,640-scalar parameter shape mismatch".to_owned());
    }
    if curved_minus_donor_nll > VALIDATION_DONOR_MARGIN {
        failures.push(format!(
            "curved validation NLL exceeded donor by {curved_minus_donor_nll}"
        ));
    }
    if curved_minus_flat_nll > VALIDATION_FLAT_MARGIN {
        failures.push(format!(
            "curved validation NLL exceeded flat by {curved_minus_flat_nll}"
        ));
    }
    if maximum_curved_vs_flat_attention_delta < LIVE_ATTENTION_DELTA {
        failures.push(format!(
            "curved versus flat attention was not live: maximum delta {maximum_curved_vs_flat_attention_delta}"
        ));
    }
    if !zero_faults_and_future_reads {
        failures.push("construction-validation fault or future read".to_owned());
    }
    deadline.check("construction.validation.gate")?;
    Ok(ValidationGateReport {
        passed: failures.is_empty(),
        curved_minus_donor_nll,
        curved_minus_flat_nll,
        maximum_curved_vs_flat_attention_delta,
        parameter_replay_exact,
        fit_report_replay_exact,
        fit_work_and_shape_valid,
        trace_matches_donor_decoder,
        zero_faults_and_future_reads,
        geometry_preflight,
        failures,
    })
}

fn fit_work_and_shape_valid(fit: &FittedArm) -> bool {
    let score_coefficients = fit.parameters.score_coefficients();
    let output_scales = fit.parameters.output_block_scales();
    let expected_vector_scalars = EXPECTED_FIT_PARAMETER_SCALARS / 2;
    let active_metric_coefficients = score_coefficients
        .iter()
        .filter(|coefficient| **coefficient > COEFFICIENT_FLOOR)
        .count();
    fit.report.construction_document_count == FIT_DOCUMENTS
        && fit.report.causal_rows == EXPECTED_FIT_CAUSAL_ROWS
        && fit.report.causal_source_pairs == EXPECTED_FIT_CAUSAL_SOURCE_PAIRS
        && fit.report.geometric_row_evaluations == EXPECTED_FIT_GEOMETRIC_ROW_EVALUATIONS
        && fit.report.geometric_source_pair_evaluations
            == EXPECTED_FIT_GEOMETRIC_SOURCE_PAIR_EVALUATIONS
        && fit.report.feature_block_evaluations == EXPECTED_FIT_FEATURE_BLOCK_EVALUATIONS
        && fit.report.centroid_source_block_evaluations
            == EXPECTED_FIT_CENTROID_SOURCE_BLOCK_EVALUATIONS
        && fit.report.output_scale_lane_accumulations
            == EXPECTED_FIT_OUTPUT_SCALE_LANE_ACCUMULATIONS
        && fit.report.nnls_sweeps == NNLS_SWEEPS
        && fit.report.nnls_coordinate_updates == EXPECTED_FIT_NNLS_COORDINATE_UPDATES
        && fit.report.ridge.to_bits() == RIDGE.to_bits()
        && fit.report.coefficient_floor.to_bits() == COEFFICIENT_FLOOR.to_bits()
        && fit.report.output_scale_floor.to_bits() == OUTPUT_SCALE_FLOOR.to_bits()
        && fit.report.parameter_scalars == EXPECTED_FIT_PARAMETER_SCALARS
        && fit.report.active_metric_coefficients == active_metric_coefficients
        && active_metric_coefficients > 0
        && score_coefficients.len() == expected_vector_scalars
        && output_scales.len() == expected_vector_scalars
        && score_coefficients
            .iter()
            .all(|coefficient| coefficient.is_finite() && *coefficient >= COEFFICIENT_FLOOR)
        && output_scales
            .iter()
            .all(|scale| scale.is_finite() && *scale >= OUTPUT_SCALE_FLOOR)
}

fn validation_evidence_valid(report: &ValidationGateReport) -> bool {
    report.geometry_preflight.passed
        && report.parameter_replay_exact
        && report.fit_report_replay_exact
        && report.fit_work_and_shape_valid
        && report.trace_matches_donor_decoder
        && report.zero_faults_and_future_reads
}

fn validation_failure_terminal(report: &ValidationGateReport) -> &'static str {
    if validation_evidence_valid(report) {
        VALIDATION_FAIL_TERMINAL
    } else {
        UNAVAILABLE_TERMINAL
    }
}

fn run_decode(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &Tokenizer,
    document: &FrozenDocument,
    spec: ArmSpec<'_>,
    arm: &str,
    deadline: &ExperimentDeadline,
    stage: &str,
) -> TestResult<DecodeReport> {
    deadline.check(stage)?;
    let sequence_capacity = INPUT_POSITIONS
        .checked_add(GENERATED_TOKENS - 1)
        .ok_or("decode sequence capacity overflow")?;
    match spec {
        ArmSpec::Donor => {
            let mut state = oracle.new_state_bounded(sequence_capacity)?;
            let mut logits = vec![0.0; oracle.cfg().vocab];
            for position in 0..INPUT_POSITIONS {
                deadline.check(stage)?;
                oracle.step_state(
                    &mut state,
                    document.tokens[position] as usize,
                    position,
                    &mut logits,
                )?;
                deadline.check(stage)?;
            }
            let mut generated = vec![u32::try_from(argmax(&logits)?)?];
            for offset in 1..GENERATED_TOKENS {
                deadline.check(stage)?;
                oracle.step_state(
                    &mut state,
                    generated[offset - 1] as usize,
                    INPUT_POSITIONS + offset - 1,
                    &mut logits,
                )?;
                deadline.check(stage)?;
                generated.push(u32::try_from(argmax(&logits)?)?);
            }
            Ok(DecodeReport {
                arm: arm.to_owned(),
                generated_token_cid: token_cid(b"uor-r4.intrinsic.decode/1", &generated),
                generated_text: tokenizer.decode(&generated),
                no_period_one_or_two_cycle: !has_short_cycle(&generated),
                generated_tokens: generated,
                state_cid: state.persistent_state_cid(),
                causal_audit: None,
                evidence_cid: None,
            })
        }
        ArmSpec::Gauge | ArmSpec::Intrinsic { .. } => {
            let maximum_token_id =
                u32::try_from(oracle.cfg().vocab.checked_sub(1).ok_or("empty vocab")?)?;
            let transport: Box<dyn uor_r4_model_source::attention::CausalAttentionTransport> =
                match spec {
                    ArmSpec::Gauge => Box::new(R4SpinCausalAttentionTransport::new(
                        maximum_token_id,
                        sequence_capacity,
                        R4SpinTransportIntervention::Coherent,
                    )?),
                    ArmSpec::Intrinsic {
                        parameters,
                        metric,
                        intervention,
                    } => Box::new(IntrinsicR4CausalAttentionTransport::new(
                        maximum_token_id,
                        sequence_capacity,
                        parameters.clone(),
                        metric,
                        intervention,
                    )?),
                    ArmSpec::Donor => unreachable!(),
                };
            let mut session = oracle.new_causal_attention_transport_session(
                transport,
                CausalAttentionLayerSelection::All,
                sequence_capacity,
            )?;
            let mut logits = vec![0.0; oracle.cfg().vocab];
            for position in 0..INPUT_POSITIONS {
                deadline.check(stage)?;
                oracle.step_causal_attention_transport(
                    &mut session,
                    document.tokens[position] as usize,
                    position,
                    &mut logits,
                )?;
                deadline.check(stage)?;
            }
            let mut generated = vec![u32::try_from(argmax(&logits)?)?];
            for offset in 1..GENERATED_TOKENS {
                deadline.check(stage)?;
                oracle.step_causal_attention_transport(
                    &mut session,
                    generated[offset - 1] as usize,
                    INPUT_POSITIONS + offset - 1,
                    &mut logits,
                )?;
                deadline.check(stage)?;
                generated.push(u32::try_from(argmax(&logits)?)?);
            }
            deadline.check(stage)?;
            session.transport_status()?;
            let audit = session.audit();
            if audit != expected_causal_audit(sequence_capacity)? {
                return Err("decode causal audit differs from exact work".into());
            }
            let evidence = session
                .transport_implementation_evidence()?
                .ok_or("decode transport omitted evidence")?;
            let evidence_value: serde_json::Value = serde_json::from_str(&evidence)?;
            validate_implementation_evidence(&evidence_value, spec, sequence_capacity)?;
            Ok(DecodeReport {
                arm: arm.to_owned(),
                generated_token_cid: token_cid(b"uor-r4.intrinsic.decode/1", &generated),
                generated_text: tokenizer.decode(&generated),
                no_period_one_or_two_cycle: !has_short_cycle(&generated),
                generated_tokens: generated,
                state_cid: session.persistent_state_cid(),
                causal_audit: Some(AuditReport::from(audit)),
                evidence_cid: Some(cid_bytes(evidence.as_bytes())),
            })
        }
    }
}

fn has_short_cycle(tokens: &[u32]) -> bool {
    if tokens.len() < 2 {
        return false;
    }
    let period_one = tokens.iter().all(|token| *token == tokens[0]);
    let period_two = tokens.len() >= 4
        && tokens
            .iter()
            .enumerate()
            .all(|(index, token)| *token == tokens[index % 2]);
    period_one || period_two
}

fn exact_arm_replay(left: &ArmExecution, right: &ArmExecution) -> TestResult<bool> {
    Ok(
        serde_json::to_vec(&left.report)? == serde_json::to_vec(&right.report)?
            && left.logits.len() == right.logits.len()
            && left
                .logits
                .iter()
                .flatten()
                .flatten()
                .zip(right.logits.iter().flatten().flatten())
                .all(|(left, right)| left.to_bits() == right.to_bits()),
    )
}

fn exact_decode_replay(left: &DecodeReport, right: &DecodeReport) -> TestResult<bool> {
    Ok(serde_json::to_vec(left)? == serde_json::to_vec(right)?)
}

fn thresholds() -> Thresholds {
    Thresholds {
        validation_donor_margin: VALIDATION_DONOR_MARGIN,
        validation_flat_margin: VALIDATION_FLAT_MARGIN,
        live_attention_delta: LIVE_ATTENTION_DELTA,
        final_reference_margin: FINAL_REFERENCE_MARGIN,
        final_flat_margin: FINAL_FLAT_MARGIN,
        final_control_margin: FINAL_CONTROL_MARGIN,
        required_document_wins: 7,
        top1_reference_shortfall_tokens: 1,
    }
}

fn write_result(
    path: &Path,
    result: ResultPayload,
    operational_telemetry: OperationalTelemetry,
) -> TestResult<String> {
    let result_cid = canonical_json_cid(&result)?;
    let envelope = ResultEnvelope {
        result_cid: result_cid.clone(),
        result,
        operational_telemetry,
    };
    write_pretty_json_exclusive(path, &envelope)?;
    Ok(result_cid)
}

fn write_failure(
    path: &Path,
    canonical_output_path: &Path,
    deadline: &ExperimentDeadline,
    error: &(dyn Error + 'static),
    manifest_envelope: &FrozenManifestEnvelope,
) -> TestResult<String> {
    let reveal_path = partition_reveal_path(canonical_output_path)?;
    let d3_reveal_marker_cid = if reveal_path.exists() {
        Some(canonical_json_file_cid(&reveal_path)?)
    } else {
        None
    };
    if deadline.heldout_opened() && d3_reveal_marker_cid.is_none() {
        return Err(format!(
            "post-reveal failure cannot be published without the durable marker {}",
            reveal_path.display()
        )
        .into());
    }
    let result = failure_result(deadline, error, manifest_envelope, d3_reveal_marker_cid);
    let result_cid = canonical_json_cid(&result)?;
    let envelope = FailureEnvelope {
        result_cid: result_cid.clone(),
        result,
    };
    write_pretty_json_exclusive(path, &envelope)?;
    Ok(result_cid)
}

fn failure_result(
    deadline: &ExperimentDeadline,
    error: &(dyn Error + 'static),
    manifest_envelope: &FrozenManifestEnvelope,
    d3_reveal_marker_cid: Option<String>,
) -> FailureResult {
    let heldout_opened = deadline.heldout_opened();
    FailureResult {
        schema: if heldout_opened {
            "uor-r4.intrinsic-lorentz-r4-attention-invalid-post-reveal/1"
        } else {
            "uor-r4.intrinsic-lorentz-r4-attention-unavailable/1"
        },
        issue: 973,
        terminal: if heldout_opened {
            POST_REVEAL_INVALID_TERMINAL
        } else {
            UNAVAILABLE_TERMINAL
        },
        manifest_cid: manifest_envelope.manifest_cid.clone(),
        partition_cid: manifest_envelope.manifest.partition_cid.clone(),
        d3_reveal_marker_cid,
        stage: deadline.stage(),
        error: error.to_string(),
        elapsed_seconds: deadline.elapsed_seconds(),
        deadline_seconds: deadline.limit.as_secs(),
        deadline_exceeded: deadline.is_exceeded(),
        heldout_opened,
    }
}

#[test]
fn expired_deadline_stops_before_heldout_and_builds_unavailable_record() {
    let deadline = ExperimentDeadline {
        started: Instant::now() - Duration::from_secs(2),
        limit: Duration::from_secs(1),
        stage: RefCell::new("initialization".to_owned()),
        heldout_opened: Cell::new(false),
    };

    let error = deadline
        .check("heldout.admission")
        .expect_err("expired deadline must reject heldout admission");
    assert!(!deadline.heldout_opened.get());
    let manifest_envelope = test_manifest_envelope();
    let result = failure_result(&deadline, error.as_ref(), &manifest_envelope, None);
    assert_eq!(result.terminal, UNAVAILABLE_TERMINAL);
    assert_eq!(result.stage, "heldout.admission");
    assert!(result.deadline_exceeded);
    assert!(!result.heldout_opened);
    assert!(result.error.contains("experiment deadline exceeded"));

    deadline.mark_heldout_opened();
    let reveal_cid = cid_bytes(b"test-reveal-marker");
    let post_reveal = failure_result(
        &deadline,
        error.as_ref(),
        &manifest_envelope,
        Some(reveal_cid.clone()),
    );
    assert_eq!(post_reveal.terminal, POST_REVEAL_INVALID_TERMINAL);
    assert_eq!(
        post_reveal.schema,
        "uor-r4.intrinsic-lorentz-r4-attention-invalid-post-reveal/1"
    );
    assert!(post_reveal.heldout_opened);
    assert_eq!(
        post_reveal.d3_reveal_marker_cid.as_deref(),
        Some(reveal_cid.as_str())
    );
}

#[test]
fn fit_evidence_controls_unavailable_vs_valid_frozen_negative() {
    let mut gate = ValidationGateReport {
        passed: false,
        curved_minus_donor_nll: VALIDATION_DONOR_MARGIN + 0.01,
        curved_minus_flat_nll: 0.0,
        maximum_curved_vs_flat_attention_delta: LIVE_ATTENTION_DELTA,
        parameter_replay_exact: true,
        fit_report_replay_exact: true,
        fit_work_and_shape_valid: false,
        trace_matches_donor_decoder: true,
        zero_faults_and_future_reads: true,
        geometry_preflight: GeometryPreflight {
            exercised_blocks: 1,
            maximum_hyperboloid_residual: 0.0,
            maximum_distance_invariance_delta: 0.0,
            maximum_barycenter_covariance_delta: 0.0,
            minimum_timelike_denominator_squared: 1.0,
            maximum_softmax_sum_delta: 0.0,
            helm_d_golden_reproduced: true,
            passed: true,
        },
        failures: vec!["curved validation NLL exceeded donor margin".to_owned()],
    };

    assert_eq!(validation_failure_terminal(&gate), UNAVAILABLE_TERMINAL);
    let serialized = serde_json::to_value(&gate).expect("serialize validation gate");
    assert!(serialized
        .get("maximum_curved_vs_flat_attention_delta")
        .is_some());
    assert!(serialized
        .get("maximum_curved_vs_gauge_attention_delta")
        .is_none());
    gate.fit_work_and_shape_valid = true;
    assert_eq!(validation_failure_terminal(&gate), VALIDATION_FAIL_TERMINAL);
}

#[test]
fn fitter_softmax_matches_live_f32_weights_not_unrounded_f64() -> TestResult {
    let logits = [0.123_456_789, -0.987_654_321, 2.345_678_901];
    let fitted = stable_softmax(&logits)?;
    let maximum = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let raw_exp = logits.map(|logit| libm::exp(logit - maximum));
    let denominator = raw_exp.iter().sum::<f64>();
    let raw = raw_exp.map(|value| value / denominator);

    for (actual, expected) in fitted.iter().zip(raw) {
        assert_eq!(actual.to_bits(), f64::from(expected as f32).to_bits());
    }
    assert!(fitted
        .iter()
        .zip(raw)
        .any(|(actual, unrounded)| actual.to_bits() != unrounded.to_bits()));
    Ok(())
}

#[test]
fn fitter_matches_live_block_quantization_and_rejects_nonfinite_math() -> TestResult {
    let source = [1.0 / 3.0, -1.0 / 7.0, 1.0e-30, 1.0e30];
    let quantized = quantize_live_block(source)?;
    for (actual, expected) in quantized.into_iter().zip(source) {
        assert_eq!(actual.to_bits(), f64::from(expected as f32).to_bits());
    }
    assert!(quantize_live_block([f64::MAX, 0.0, 0.0, 0.0]).is_err());
    assert!(quantize_live_block([f64::NAN, 0.0, 0.0, 0.0]).is_err());

    let mut equation = NormalEquation::default();
    equation.correlation[0] = f64::NAN;
    assert!(solve_nonnegative_coordinates(&equation).is_err());
    assert!(solve_output_scales(&[f64::NAN], &[1.0]).is_err());
    assert!(solve_output_scales(&[1.0], &[f64::INFINITY]).is_err());
    Ok(())
}

#[test]
fn decoder_metrics_reject_nonfinite_or_misaligned_logits() {
    assert!(argmax(&[]).is_err());
    assert!(argmax(&[0.0, f32::NAN]).is_err());
    assert!(target_rank(&[0.0], 1).is_err());
    assert!(cross_entropy(&[0.0, f32::INFINITY], 0).is_err());
}

fn test_partition_id(heldout: bool) -> String {
    (0u64..10_000)
        .map(|id| id.to_string())
        .find(|id| d3_is_held_out(id) == heldout && (!heldout || id != EXCLUDED_HELDOUT_ID))
        .expect("test must find both D3 and construction ids")
}

fn test_commitment(id: String, offset: u64, length: u64, seed: &[u8]) -> FrozenDocumentCommitment {
    FrozenDocumentCommitment {
        selection_digest: format!("blake3:{}", hex::encode(selection_digest(&id))),
        id,
        title: "sealed title".to_owned(),
        token_cid: cid_bytes(&[seed, b"token"].concat()),
        input_cid: cid_bytes(&[seed, b"input"].concat()),
        target_cid: cid_bytes(&[seed, b"target"].concat()),
        corpus_byte_offset: offset,
        corpus_byte_length: length,
    }
}

#[test]
fn partition_schema_serializes_only_commitments() -> TestResult {
    assert_eq!(
        PARTITION_SCHEMA,
        "uor-r4.intrinsic-lorentz-r4-attention-partition/1"
    );
    assert_ne!(
        PARTITION_SCHEMA,
        "uor-r4.intrinsic-lorentz-r4-attention-manifest/1"
    );
    let commitment = test_commitment(test_partition_id(false), 123, 456, b"redacted");
    let encoded = serde_json::to_value(&commitment)?;
    let object = encoded
        .as_object()
        .ok_or("serialized commitment must be a JSON object")?;
    let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "corpus_byte_length",
            "corpus_byte_offset",
            "id",
            "input_cid",
            "selection_digest",
            "target_cid",
            "title",
            "token_cid",
        ]
    );
    assert!(!object.contains_key("tokens"));

    let mut injected = encoded;
    injected
        .as_object_mut()
        .ok_or("serialized commitment must remain a JSON object")?
        .insert("tokens".to_owned(), serde_json::json!([1, 2, 3]));
    assert!(serde_json::from_value::<FrozenDocumentCommitment>(injected).is_err());
    Ok(())
}

#[test]
fn ordered_d3_target_commitment_detects_reorder_and_target_change() -> TestResult {
    let first_id = test_partition_id(true);
    let second_id = (0u64..10_000)
        .map(|id| id.to_string())
        .find(|id| id != &first_id && id != EXCLUDED_HELDOUT_ID && d3_is_held_out(id))
        .ok_or("test must find a second D3 id")?;
    let first = test_commitment(first_id, 0, 10, b"first");
    let second = test_commitment(second_id, 10, 10, b"second");
    let ordered = aggregate_d3_target_commitment(&[first.clone(), second.clone()]);
    assert_eq!(
        ordered,
        aggregate_d3_target_commitment(&[first.clone(), second.clone()])
    );
    assert_ne!(
        ordered,
        aggregate_d3_target_commitment(&[second.clone(), first.clone()])
    );
    let mut changed = second;
    changed.target_cid = cid_bytes(b"changed-target");
    assert_ne!(ordered, aggregate_d3_target_commitment(&[first, changed]));
    Ok(())
}

#[test]
fn heldout_corpus_span_stays_unread_until_admission() -> TestResult {
    let record = b"{\"id\":\"heldout\",\"title\":\"sealed title\",\"text\":\"secret\"}\n";
    let commitment = test_commitment(
        test_partition_id(true),
        0,
        u64::try_from(record.len())?,
        b"lazy",
    );
    let mut corpus = std::io::Cursor::new(record.to_vec());
    let error = read_committed_corpus_record(
        &mut corpus,
        u64::try_from(record.len())?,
        &commitment,
        PartitionAccess::Heldout,
        false,
    )
    .expect_err("heldout read must fail before validation admission");
    assert!(error.to_string().contains("before validation admission"));
    assert_eq!(
        corpus.position(),
        0,
        "failed admission must perform no seek or read"
    );

    let revealed = read_committed_corpus_record(
        &mut corpus,
        u64::try_from(record.len())?,
        &commitment,
        PartitionAccess::Heldout,
        true,
    )?;
    assert_eq!(revealed, record);
    assert_eq!(corpus.position(), u64::try_from(record.len())?);
    Ok(())
}

fn temporary_evidence_path(label: &str) -> TestResult<PathBuf> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!("uor-r4-973-{label}-{}-{nonce}", std::process::id())))
}

#[test]
fn exclusive_evidence_files_refuse_overwrite_and_durable_reveal_refuses_rerun() -> TestResult {
    let directory = temporary_evidence_path("exclusive-guards")?;
    let attempt_one_path = directory.join("result.json");
    let output_path = directory.join(ATTEMPT_TWO_RESULT_FILE);
    let run_lock = acquire_partition_run_lock(&output_path)?;
    assert!(acquire_partition_run_lock(&output_path).is_err());
    write_content_addressed_exclusive(
        &attempt_one_path,
        &serde_json::json!({"attempt": 1, "terminal": UNAVAILABLE_TERMINAL}),
    )?;
    let attempt_one_bytes = fs::read(&attempt_one_path)?;
    ensure_fresh_run_target(&output_path)?;
    write_content_addressed_exclusive(
        &output_path,
        &serde_json::json!({"attempt": 2, "first": true}),
    )?;
    assert!(
        write_content_addressed_exclusive(&output_path, &serde_json::json!({"second": true}))
            .is_err()
    );
    assert!(ensure_fresh_run_target(&output_path).is_err());
    assert_eq!(fs::read(&attempt_one_path)?, attempt_one_bytes);

    fs::remove_file(&output_path)?;
    let reveal_path = partition_reveal_path(&output_path)?;
    write_content_addressed_exclusive(&reveal_path, &serde_json::json!({"revealed": true}))?;
    assert!(ensure_fresh_run_target(&output_path).is_err());
    let later_attempt_path = directory.join("result.attempt-03.json");
    assert!(ensure_fresh_run_target(&later_attempt_path).is_err());
    assert_eq!(fs::read(&attempt_one_path)?, attempt_one_bytes);

    fs::remove_file(reveal_path)?;
    fs::remove_file(attempt_one_path)?;
    drop(run_lock);
    fs::remove_file(directory.join("run.lock"))?;
    fs::remove_dir(directory)?;
    Ok(())
}

fn test_fit_arm(metric: IntrinsicR4AttentionMetric) -> TestResult<FittedArm> {
    let parameters = IntrinsicLorentzR4AttentionParameters::uniform(
        EXPECTED_LAYERS,
        EXPECTED_HEADS,
        BLOCKS,
        1.0,
        1.0,
    )?;
    let parameter_json = serde_json::to_vec(&parameters)?;
    let mut report = FitReport {
        metric,
        construction_document_count: FIT_DOCUMENTS,
        causal_rows: EXPECTED_FIT_CAUSAL_ROWS,
        causal_source_pairs: EXPECTED_FIT_CAUSAL_SOURCE_PAIRS,
        geometric_row_evaluations: EXPECTED_FIT_GEOMETRIC_ROW_EVALUATIONS,
        geometric_source_pair_evaluations: EXPECTED_FIT_GEOMETRIC_SOURCE_PAIR_EVALUATIONS,
        feature_block_evaluations: EXPECTED_FIT_FEATURE_BLOCK_EVALUATIONS,
        centroid_source_block_evaluations: EXPECTED_FIT_CENTROID_SOURCE_BLOCK_EVALUATIONS,
        output_scale_lane_accumulations: EXPECTED_FIT_OUTPUT_SCALE_LANE_ACCUMULATIONS,
        nnls_sweeps: NNLS_SWEEPS,
        nnls_coordinate_updates: EXPECTED_FIT_NNLS_COORDINATE_UPDATES,
        ridge: RIDGE,
        coefficient_floor: COEFFICIENT_FLOOR,
        output_scale_floor: OUTPUT_SCALE_FLOOR,
        parameter_scalars: EXPECTED_FIT_PARAMETER_SCALARS,
        active_metric_coefficients: EXPECTED_FIT_PARAMETER_SCALARS / 2,
        row_centered_objective: 1.0,
        construction_trace_cid: cid_bytes(b"test-construction-trace"),
        parameter_json_cid: cid_bytes(&parameter_json),
        fit_report_cid: String::new(),
    };
    report.fit_report_cid = cid_bytes(&serde_json::to_vec(&report)?);
    Ok(FittedArm {
        parameters,
        report,
        parameter_json,
    })
}

fn test_implementation_identity() -> ImplementationIdentity {
    ImplementationIdentity {
        revision: "a".repeat(40),
        executable_cid: cid_bytes(b"executable"),
        core_source_cid: cid_bytes(b"core"),
        harness_source_cid: cid_bytes(b"harness"),
        model_attention_source_cid: cid_bytes(b"attention"),
        model_source_cid: cid_bytes(b"model"),
        exact_executor_source_cid: cid_bytes(b"exact-executor"),
        contract_cid: cid_bytes(b"contract"),
        compiled_partition_bytes_cid: cid_bytes(b"partition-bytes"),
    }
}

fn test_manifest_envelope() -> FrozenManifestEnvelope {
    FrozenManifestEnvelope {
        manifest_cid: cid_bytes(b"test-manifest"),
        manifest: FrozenPartitionManifest {
            schema: PARTITION_SCHEMA.to_owned(),
            issue: 973,
            selection_policy: "test-only".to_owned(),
            corpus_cid: CORPUS_CID.to_owned(),
            corpus_documents: CORPUS_DOCUMENTS,
            donor_source_cid: DONOR_CID.to_owned(),
            tokenizer_cid: cid_bytes(b"tokenizer"),
            required_tokens_per_document: REQUIRED_TOKENS,
            input_positions: INPUT_POSITIONS,
            scored_positions: (SCORE_START..INPUT_POSITIONS).collect(),
            construction_fit: Vec::new(),
            construction_validation: Vec::new(),
            d3_heldout: Vec::new(),
            d3_target_commitment_cid: cid_bytes(b"targets"),
            partition_cid: cid_bytes(b"test-partition"),
        },
    }
}

fn test_reveal_marker(manifest_envelope: &FrozenManifestEnvelope) -> D3RevealMarker {
    D3RevealMarker {
        schema: "uor-r4.intrinsic-lorentz-r4-attention-d3-reveal/1",
        issue: 973,
        manifest_cid: manifest_envelope.manifest_cid.clone(),
        partition_cid: manifest_envelope.manifest.partition_cid.clone(),
        fit_checkpoint_cid: cid_bytes(b"test-fit-checkpoint"),
        implementation_identity: test_implementation_identity(),
    }
}

#[test]
fn interrupted_reveal_is_reconciled_without_reopening_d3() -> TestResult {
    let directory = temporary_evidence_path("reveal-reconciliation")?;
    let output_path = directory.join("result.json");
    let reveal_path = partition_reveal_path(&output_path)?;
    let manifest_envelope = test_manifest_envelope();
    write_content_addressed_exclusive(&reveal_path, &test_reveal_marker(&manifest_envelope))?;
    let deadline = ExperimentDeadline::new();
    let result_cid = reconcile_interrupted_reveal(&output_path, &deadline, &manifest_envelope)?
        .ok_or("missing post-reveal reconciliation result")?;
    assert!(is_blake3_cid(&result_cid));
    assert!(deadline.heldout_opened());
    let envelope: serde_json::Value = serde_json::from_slice(&fs::read(&output_path)?)?;
    assert_eq!(envelope["result"]["terminal"], POST_REVEAL_INVALID_TERMINAL);
    assert_eq!(
        valid_post_reveal_terminal_cid(&output_path, &manifest_envelope, &reveal_path)?.as_deref(),
        Some(result_cid.as_str())
    );
    assert_eq!(
        reconcile_interrupted_reveal(&output_path, &deadline, &manifest_envelope)?.as_deref(),
        None,
        "a valid canonical post-reveal terminal must not be replaced"
    );

    fs::remove_file(output_path)?;
    fs::remove_file(reveal_path)?;

    let invalid_output_path = directory.join("invalid-result.json");
    let invalid_reveal_path = partition_reveal_path(&invalid_output_path)?;
    write_content_addressed_exclusive(
        &invalid_reveal_path,
        &test_reveal_marker(&manifest_envelope),
    )?;
    fs::write(&invalid_output_path, b"{truncated")?;
    let invalid_deadline = ExperimentDeadline::new();
    assert!(reconcile_interrupted_reveal(
        &invalid_output_path,
        &invalid_deadline,
        &manifest_envelope,
    )?
    .is_some());
    let reconciliation_path = sidecar_path(&invalid_output_path, ".post-reveal-invalid.json");
    let reconciliation: serde_json::Value =
        serde_json::from_slice(&fs::read(&reconciliation_path)?)?;
    assert_eq!(
        reconciliation["result"]["terminal"],
        POST_REVEAL_INVALID_TERMINAL
    );
    let reconciliation_cid = reconciliation["result_cid"]
        .as_str()
        .ok_or("reconciliation result CID missing")?;
    assert_eq!(
        valid_post_reveal_terminal_cid(
            &reconciliation_path,
            &manifest_envelope,
            &invalid_reveal_path,
        )?
        .as_deref(),
        Some(reconciliation_cid)
    );
    assert_eq!(
        reconcile_interrupted_reveal(&invalid_output_path, &invalid_deadline, &manifest_envelope,)?
            .as_deref(),
        Some(reconciliation_cid),
        "restart must reuse the bound no-clobber reconciliation"
    );
    fs::remove_file(invalid_output_path)?;
    fs::remove_file(invalid_reveal_path)?;
    fs::remove_file(reconciliation_path)?;
    fs::remove_dir(directory)?;
    Ok(())
}

#[test]
fn post_reveal_terminal_validation_rejects_tampering_and_pre_reveal_terminals() -> TestResult {
    let directory = temporary_evidence_path("post-reveal-binding")?;
    let output_path = directory.join("result.json");
    let reveal_path = partition_reveal_path(&output_path)?;
    let manifest_envelope = test_manifest_envelope();
    let reveal_cid =
        write_content_addressed_exclusive(&reveal_path, &test_reveal_marker(&manifest_envelope))?;
    let error = std::io::Error::other("test post-reveal interruption");
    let deadline = ExperimentDeadline::new();
    deadline.mark_heldout_opened();
    let result = failure_result(&deadline, &error, &manifest_envelope, Some(reveal_cid));
    let result_cid = canonical_json_cid(&result)?;
    let mut envelope = serde_json::json!({"result_cid": result_cid, "result": result});
    write_pretty_json(&output_path, &envelope)?;
    assert!(
        valid_post_reveal_terminal_cid(&output_path, &manifest_envelope, &reveal_path)?.is_some()
    );

    envelope["result"]["error"] = serde_json::json!("tampered without updating the CID");
    write_pretty_json(&output_path, &envelope)?;
    assert!(
        valid_post_reveal_terminal_cid(&output_path, &manifest_envelope, &reveal_path)?.is_none()
    );

    envelope["result"]["manifest_cid"] = serde_json::json!(cid_bytes(b"wrong-manifest"));
    let rebound_result_cid = canonical_json_cid(&envelope["result"])?;
    envelope["result_cid"] = serde_json::json!(rebound_result_cid);
    write_pretty_json(&output_path, &envelope)?;
    assert!(
        valid_post_reveal_terminal_cid(&output_path, &manifest_envelope, &reveal_path)?.is_none()
    );

    let pre_reveal_deadline = ExperimentDeadline::new();
    let pre_reveal = failure_result(&pre_reveal_deadline, &error, &manifest_envelope, None);
    let pre_reveal_cid = canonical_json_cid(&pre_reveal)?;
    write_pretty_json(
        &output_path,
        &serde_json::json!({"result_cid": pre_reveal_cid, "result": pre_reveal}),
    )?;
    assert!(
        valid_post_reveal_terminal_cid(&output_path, &manifest_envelope, &reveal_path)?.is_none(),
        "pre-reveal UNAVAILABLE cannot satisfy post-reveal reconciliation"
    );

    fs::remove_file(output_path)?;
    fs::remove_file(reveal_path)?;
    fs::remove_dir(directory)?;
    Ok(())
}

#[test]
fn fit_checkpoint_round_trips_full_oracle_and_rejects_tampering() -> TestResult {
    let directory = temporary_evidence_path("fit-checkpoint")?;
    let checkpoint_path = directory.join("checkpoint.json");
    let tampered_path = directory.join("tampered.json");
    let curved = test_fit_arm(IntrinsicR4AttentionMetric::Lorentz)?;
    let flat = test_fit_arm(IntrinsicR4AttentionMetric::Flat)?;
    let replay = fit_replay_evidence(&curved, &curved, &flat, &flat)?;
    let identity = test_implementation_identity();
    let manifest = test_manifest_envelope();
    let checkpoint = FitCheckpoint {
        schema: "uor-r4.intrinsic-lorentz-r4-attention-fit-checkpoint/1".to_owned(),
        issue: 973,
        manifest_cid: manifest.manifest_cid.clone(),
        partition_cid: manifest.manifest.partition_cid.clone(),
        implementation_identity: identity.clone(),
        curved_parameters: curved.parameters,
        flat_parameters: flat.parameters,
        curved_fit: curved.report,
        flat_fit: flat.report,
        replay,
    };
    let checkpoint_cid = write_fit_checkpoint(&checkpoint_path, checkpoint)?;
    let restored = read_fit_checkpoint(&checkpoint_path, &manifest, &identity)?;
    assert_eq!(restored.checkpoint_cid, checkpoint_cid);
    let restored = fit_stage_from_checkpoint(restored, true)?;
    assert_eq!(
        restored.curved.report.metric,
        IntrinsicR4AttentionMetric::Lorentz
    );
    assert_eq!(
        restored.flat.report.metric,
        IntrinsicR4AttentionMetric::Flat
    );
    assert_eq!(
        restored.curved.parameters.score_coefficients().len(),
        EXPECTED_FIT_PARAMETER_SCALARS / 2
    );

    let mut tampered: FitCheckpointEnvelope = serde_json::from_slice(&fs::read(&checkpoint_path)?)?;
    tampered.checkpoint.curved_fit.row_centered_objective = 2.0;
    write_pretty_json(&tampered_path, &tampered)?;
    assert!(read_fit_checkpoint(&tampered_path, &manifest, &identity).is_err());

    fs::remove_file(checkpoint_path)?;
    fs::remove_file(tampered_path)?;
    fs::remove_dir(directory)?;
    Ok(())
}

#[test]
fn fit_checkpoint_json_float_parsing_is_bit_exact() -> TestResult {
    let original = 0.101_468_630_291_144_97_f64;
    let encoded = serde_json::to_vec(&original)?;
    let decoded: f64 = serde_json::from_slice(&encoded)?;

    assert_eq!(encoded, b"0.10146863029114497");
    assert_eq!(decoded.to_bits(), original.to_bits());
    Ok(())
}
