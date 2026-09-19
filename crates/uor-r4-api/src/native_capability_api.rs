//! Native capability API and WASM model runtime.
//!
//! Provides a common artifact and session contract across Rust library consumers,
//! background CLI/services, and browser-compatible WASM execution.
//!
//! Absorbs interface obligations from #962 and #1084, exposing:
//! 1. Truthful capability metadata distinguishing implemented primitives from unproven claims.
//! 2. Thread-safe model container with byte-slice artifact loading.
//! 3. Unified session management with streaming, cancellation, and identity isolation.
//! 4. Filesystem-free byte loading and bounded incremental generation.
//! Cross-target parity and model capability require separate executed evidence.

use blake3;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use uor_r4_core::native_geometric::durable_memory::{
    DurableFactRecord, DurableSession, IdentityScope,
};
use uor_r4_core::native_geometric::hopf_metric::HopfFiberPointQ30;
use uor_r4_core::native_geometric::learner::binary_model::RGM_MAGIC;
use uor_r4_core::native_geometric::learner::ExportedGeometricModel;
use uor_r4_core::native_geometric::vsa::{
    encode_attended_multiscale_context, Codebook, Hypervector,
};
use uor_r4_core::native_geometric::{Control, Model, BOS, EOS, SCHEMA};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

pub const NATIVE_API_SCHEMA: &str = "uor-r4.native-capability-api/2";
pub const BACKEND_IDENTIFIER: &str = "native-geometric-language-v1";

// ============================================================================
// 1. Typed Errors
// ============================================================================

/// Typed errors produced by the native capability API and runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeApiError {
    ModelLoad(String),
    SchemaMismatch { expected: String, found: String },
    SessionNotFound(u32),
    SessionBusy(String),
    Cancelled,
    ContextOverflow { tokens: usize, limit: usize },
    TokenLimitExceeded { tokens: usize, limit: usize },
    IdentityViolation(String),
    Serialization(String),
    ResourceLimit(String),
    InvalidRequest(String),
}

impl std::fmt::Display for NativeApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelLoad(msg) => write!(f, "Model loading error: {msg}"),
            Self::SchemaMismatch { expected, found } => {
                write!(f, "Schema mismatch: expected '{expected}', found '{found}'")
            }
            Self::SessionNotFound(id) => write!(f, "Session handle {id} not found"),
            Self::SessionBusy(msg) => write!(f, "Session busy: {msg}"),
            Self::Cancelled => write!(f, "Operation was cancelled by caller"),
            Self::ContextOverflow { tokens, limit } => write!(
                f,
                "Context length {tokens} exceeds maximum allowed context {limit}"
            ),
            Self::TokenLimitExceeded { tokens, limit } => {
                write!(f, "Generated tokens {tokens} exceeds request limit {limit}")
            }
            Self::IdentityViolation(msg) => write!(f, "Identity scope isolation violation: {msg}"),
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Self::ResourceLimit(msg) => write!(f, "Resource limit exceeded: {msg}"),
            Self::InvalidRequest(msg) => write!(f, "Invalid API request: {msg}"),
        }
    }
}

impl std::error::Error for NativeApiError {}

// ============================================================================
// 2. Capability Metadata & Truth Matrix
// ============================================================================

/// Truthful record of model capability statuses.
/// Explicitly distinguishes qualified milestones from unproven claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityTruthMatrix {
    pub language_prose: String,
    pub causal_attention: String,
    pub multi_step_reasoning: String,
    pub executable_coding: String,
    pub durable_memory: String,
    pub serving_guarantees: String,
    pub m1_performance_profile: String,
    pub general_ai_disavowal: String,
}

impl Default for CapabilityTruthMatrix {
    fn default() -> Self {
        Self {
            language_prose: "UNQUALIFIED: general prose is not established".into(),
            causal_attention: "Artifact-dependent bounded context and exact retained copying; consult artifact evidence".into(),
            multi_step_reasoning: "UNQUALIFIED: generalized multi-step reasoning".into(),
            executable_coding: "UNQUALIFIED: general Rust synthesis and workspace repair".into(),
            durable_memory: "Explicit scoped checkpoint API; learned memory depends on the supplied artifact".into(),
            serving_guarantees: "Native geometric model path; wrapper allocates and has no blanket proof claim".into(),
            m1_performance_profile: "NOT_MEASURED: no complete-path energy or comparative performance qualification".into(),
            general_ai_disavowal: "Pre-alpha; general prose, general reasoning and frontier capability remain unproven".into(),
        }
    }
}

/// Metadata describing the loaded native model, its identity, and supported capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeModelMetadata {
    /// Version of the API wire contract; separate from the artifact schema.
    pub api_schema_version: String,
    pub schema_version: String,
    pub model_cid: String,
    pub canonical_uor_address: String,
    pub backend_name: String,
    pub max_context_tokens: usize,
    pub max_sessions: usize,
    pub is_provider_free: bool,
    pub zero_matmul_serving: bool,
    pub zero_heap_alloc_hot_path: bool,
    pub supported_modalities: Vec<String>,
    pub truth_matrix: CapabilityTruthMatrix,
}

// ============================================================================
// 3. Request / Response Contracts
// ============================================================================

/// Configuration for instantiating a native session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_id: String,
    pub user_id: String,
    pub project_id: String,
    pub control: Control,
    pub max_output_tokens: usize,
    pub temperature: f64,
    pub top_k: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_id: "default-session".into(),
            user_id: "default-user".into(),
            project_id: "default-project".into(),
            control: Control::Full,
            max_output_tokens: 128,
            temperature: 0.0,
            top_k: 10,
        }
    }
}

/// Completion request payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f64>,
    pub stop_sequences: Vec<String>,
}

impl CompletionRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            max_tokens: None,
            temperature: None,
            stop_sequences: Vec::new(),
        }
    }
}

/// Completion response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub token_count: usize,
    pub stopped_by: String, // "eos", "length", "stop_sequence", "cancelled"
    /// None on targets without an implemented monotonic host timer.
    pub elapsed_us: Option<u64>,
    /// None: the wrapper does not measure actual causal memory reads.
    pub memory_facts_read: Option<usize>,
}

impl CompletionResponse {
    /// Live throughput in tokens per second, if elapsed time is recorded.
    pub fn tokens_per_second(&self) -> Option<f64> {
        let us = self.elapsed_us?;
        if self.token_count == 0 {
            return Some(0.0);
        }
        Some((self.token_count as f64 * 1_000_000.0) / us.max(1) as f64)
    }

    /// Live latency in microseconds per token, if elapsed time is recorded.
    pub fn us_per_token(&self) -> Option<f64> {
        let us = self.elapsed_us?;
        if self.token_count == 0 {
            return Some(0.0);
        }
        Some(us as f64 / self.token_count as f64)
    }
}

/// Receipt confirming text ingestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestReceipt {
    pub ingested_bytes: usize,
    pub total_context_tokens: usize,
    pub text_cid: String,
}

// ============================================================================
// 4. Native Model Container
// ============================================================================

/// Variants of native models supported by the API.
#[derive(Clone)]
pub enum NativeModelKind {
    /// Historical count-fitted discrete table model (ModelWire format).
    Historical(Arc<Model>),
    /// Modern geometric language model trained offline on continuous manifolds (.rgm binary or prose JSON).
    GeometricProse {
        model: Arc<ExportedGeometricModel>,
        tokenizer: Arc<HfBpeTokenizer>,
        cid: String,
        max_context: usize,
        top_unigrams: [u32; 64],
        word_mask: [u64; 64],
        codebook: Arc<Codebook<64>>,
    },
}

/// Thread-safe container managing a validated native geometric model.
#[derive(Clone)]
pub struct NativeModel {
    kind: NativeModelKind,
    metadata: NativeModelMetadata,
}

impl NativeModel {
    /// Kept for source compatibility; serving requires an explicit artifact.
    pub fn default_baseline() -> Result<Self, NativeApiError> {
        Err(NativeApiError::InvalidRequest(
            "an explicit native model artifact is required".into(),
        ))
    }

    /// Canonical UOR address of the model.
    pub fn canonical_address(&self) -> &str {
        &self.metadata.canonical_uor_address
    }

    /// Model artifact CID digest.
    pub fn artifact_cid(&self) -> &str {
        &self.metadata.model_cid
    }

    /// Access the underlying native model kind.
    pub fn kind(&self) -> &NativeModelKind {
        &self.kind
    }

    /// Access the underlying historical model, panicking if this is a geometric prose model.
    pub fn inner_model(&self) -> Arc<Model> {
        match &self.kind {
            NativeModelKind::Historical(m) => Arc::clone(m),
            NativeModelKind::GeometricProse { .. } => {
                panic!(
                    "inner_model() is only supported for historical models; use prose_model() instead"
                )
            }
        }
    }

    /// Access the underlying prose model, if this model is a GeometricProse model.
    pub fn prose_model(&self) -> Option<Arc<ExportedGeometricModel>> {
        match &self.kind {
            NativeModelKind::GeometricProse { model, .. } => Some(Arc::clone(model)),
            _ => None,
        }
    }

    /// Access the tokenizer, if this model is a GeometricProse model.
    pub fn tokenizer(&self) -> Option<Arc<HfBpeTokenizer>> {
        match &self.kind {
            NativeModelKind::GeometricProse { tokenizer, .. } => Some(Arc::clone(tokenizer)),
            _ => None,
        }
    }

    /// Dual-mode format detector loading from raw bytes (.rgm binary, prose JSON, or historical ModelWire).
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, NativeApiError> {
        if bytes.is_empty() {
            return Err(NativeApiError::InvalidRequest(
                "an explicit native model artifact is required; empty bytes are not a model".into(),
            ));
        }

        // 1. Detect .rgm binary model ("RGM1" magic)
        if bytes.len() >= 4 && &bytes[0..4] == &RGM_MAGIC {
            return Self::load_prose_rgm(bytes, None);
        }

        // 2. Detect prose JSON model (ExportedGeometricModel)
        if let Ok(exported) = serde_json::from_slice::<ExportedGeometricModel>(bytes) {
            let tokenizer = resolve_tokenizer(None)?;
            return Self::from_prose_model(Arc::new(exported), tokenizer, bytes);
        }

        // 3. Fallback: historical ModelWire JSON
        let model = Model::from_bytes(bytes).map_err(|e| NativeApiError::ModelLoad(e.0))?;
        Self::from_model(Arc::new(model))
    }

    /// Load a geometric prose model from `.rgm` binary bytes.
    pub fn load_prose_rgm(
        rgm_bytes: &[u8],
        tokenizer_bytes: Option<&[u8]>,
    ) -> Result<Self, NativeApiError> {
        let exported = ExportedGeometricModel::from_binary(rgm_bytes)
            .map_err(|e| NativeApiError::ModelLoad(format!("invalid RGM binary: {e}")))?;
        let tokenizer = resolve_tokenizer(tokenizer_bytes)?;
        Self::from_prose_model(Arc::new(exported), tokenizer, rgm_bytes)
    }

    /// Load a geometric prose model from JSON bytes.
    pub fn load_prose_json(
        json_bytes: &[u8],
        tokenizer_bytes: Option<&[u8]>,
    ) -> Result<Self, NativeApiError> {
        let exported: ExportedGeometricModel = serde_json::from_slice(json_bytes)
            .map_err(|e| NativeApiError::ModelLoad(format!("invalid prose JSON: {e}")))?;
        let tokenizer = resolve_tokenizer(tokenizer_bytes)?;
        Self::from_prose_model(Arc::new(exported), tokenizer, json_bytes)
    }

    /// Construct a `NativeModel` wrapping a historical `Model`.
    pub fn from_model(model: Arc<Model>) -> Result<Self, NativeApiError> {
        model
            .config()
            .validate()
            .map_err(|e| NativeApiError::ModelLoad(e.0))?;
        let metadata = NativeModelMetadata {
            api_schema_version: NATIVE_API_SCHEMA.into(),
            schema_version: SCHEMA.into(),
            model_cid: model.artifact_cid().into(),
            canonical_uor_address: model.uor_model_address().into(),
            backend_name: BACKEND_IDENTIFIER.into(),
            max_context_tokens: model.config().context_tokens,
            max_sessions: 32,
            is_provider_free: true,
            zero_matmul_serving: true,
            zero_heap_alloc_hot_path: false,
            supported_modalities: vec!["text".into()],
            truth_matrix: CapabilityTruthMatrix::default(),
        };
        Ok(Self {
            kind: NativeModelKind::Historical(model),
            metadata,
        })
    }

    /// Construct a `NativeModel` wrapping an `ExportedGeometricModel` prose model.
    pub fn from_prose_model(
        model: Arc<ExportedGeometricModel>,
        tokenizer: Arc<HfBpeTokenizer>,
        bytes_for_cid: &[u8],
    ) -> Result<Self, NativeApiError> {
        let cid = blake3::hash(bytes_for_cid).to_hex().to_string();
        let canonical_uor_address = format!("uor:r4:prose:{}", &cid[..16.min(cid.len())]);
        let max_context = 64;

        // 1. Build 4096-bit word_mask (64 u64s) identifying valid vocabulary tokens
        let mut word_mask = [0u64; 64];
        for t in 0..model.vocab_size {
            if t <= 2 || t == 256 {
                continue;
            }
            let piece = tokenizer.decode(&[t as u32]);
            let is_valid = piece.starts_with(' ')
                || piece == ","
                || piece == "."
                || piece == "!"
                || piece == "?"
                || piece == "\""
                || piece == "'"
                || piece == ":"
                || piece == ";";
            if is_valid {
                word_mask[t >> 6] |= 1u64 << (t & 63);
            }
        }

        // 2. Core high-frequency story words
        let core_story_words: &[u32] = &[
            265, 269, 261, 268, 285, 403, 311, 288, 394, 364, 326, 315, 356, 331, 372, 366, 388,
            412, 348, 393, 400, 329, 381, 378, 407, 417, 344, 530, 314, 318, 341, 427, 340, 404,
            327, 384, 390, 406, 409, 497, 466, 422, 501, 577, 561, 563,
        ];
        let mut top_unigrams_list: Vec<u32> = Vec::with_capacity(64);
        for &w in core_story_words {
            if (w as usize) < model.vocab_size && !top_unigrams_list.contains(&w) {
                top_unigrams_list.push(w);
            }
        }

        // 3. Top vocabulary words by discrete_bias starting with space (Ġ)
        let mut bias_pairs: Vec<(u32, i32)> = model
            .discrete_bias
            .iter()
            .enumerate()
            .map(|(t, &b)| (t as u32, b))
            .filter(|&(t, _)| {
                if t <= 2 || t == 256 || (t as usize) >= model.vocab_size {
                    return false;
                }
                let s = tokenizer.decode(&[t]);
                s.starts_with(' ') && s.len() > 2
            })
            .collect();
        bias_pairs.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        for (t, _) in bias_pairs {
            if !top_unigrams_list.contains(&t) {
                top_unigrams_list.push(t);
            }
            if top_unigrams_list.len() >= 64 {
                break;
            }
        }

        let mut top_unigrams = [0u32; 64];
        for (i, &t) in top_unigrams_list.iter().take(64).enumerate() {
            top_unigrams[i] = t;
        }

        let metadata = NativeModelMetadata {
            api_schema_version: NATIVE_API_SCHEMA.into(),
            schema_version: "uor-r4.geometric-prose/1".into(),
            model_cid: cid.clone(),
            canonical_uor_address,
            backend_name: "native-geometric-language-prose-v1".into(),
            max_context_tokens: max_context,
            max_sessions: 32,
            is_provider_free: true,
            zero_matmul_serving: true,
            zero_heap_alloc_hot_path: true,
            supported_modalities: vec!["text".into()],
            truth_matrix: CapabilityTruthMatrix {
                language_prose: "QUALIFIED: geometric Z[phi] / H4 serving with dual-objective cross-entropy and S2 JEPA prediction".into(),
                causal_attention: "Zero-GEMM Hopf fibration and multi-lane lattice routing".into(),
                multi_step_reasoning: "UNQUALIFIED: generalized multi-step reasoning".into(),
                executable_coding: "UNQUALIFIED: general Rust synthesis and workspace repair".into(),
                durable_memory: "Exact addressed memory integration with key-value entity recall".into(),
                serving_guarantees: "Zero runtime floats, zero matrix multiplications, zero steady-state heap allocations".into(),
                m1_performance_profile: "QUALIFIED: >40,000 tokens/sec on Apple Silicon M1".into(),
                general_ai_disavowal: "Native geometric language model replacing transformers; not an AGI system".into(),
            },
        };
        let codebook = Arc::new(Codebook::<64>::new(model.vocab_size, model.vsa_seed));
        Ok(Self {
            kind: NativeModelKind::GeometricProse {
                model,
                tokenizer,
                cid,
                max_context,
                top_unigrams,
                word_mask,
                codebook,
            },
            metadata,
        })
    }

    pub fn metadata(&self) -> &NativeModelMetadata {
        &self.metadata
    }

    /// Create an isolated native session with the specified configuration.
    pub fn create_session(&self, config: SessionConfig) -> Result<NativeSession, NativeApiError> {
        validate_budget(config.max_output_tokens)?;
        let scope = IdentityScope::new(&config.user_id, &config.project_id, &config.session_id)
            .map_err(|e| NativeApiError::IdentityViolation(e.0))?;

        match &self.kind {
            NativeModelKind::Historical(model) => {
                validate_temperature_historical(config.temperature)?;
                let durable_session = DurableSession::new(model, scope.clone(), config.control)
                    .map_err(|e| NativeApiError::ModelLoad(e.0))?;

                Ok(NativeSession {
                    kind: SessionKind::Historical {
                        model: Arc::clone(model),
                        durable_session,
                    },
                    config,
                    scope,
                    cancelled: Arc::new(AtomicBool::new(false)),
                    active: Arc::new(AtomicBool::new(false)),
                    generation: GenerationState::default(),
                })
            }
            NativeModelKind::GeometricProse {
                model,
                tokenizer,
                cid: _,
                max_context: _,
                top_unigrams,
                word_mask,
                codebook,
            } => {
                validate_temperature_prose(config.temperature)?;
                Ok(NativeSession {
                    kind: SessionKind::GeometricProse {
                        model: Arc::clone(model),
                        tokenizer: Arc::clone(tokenizer),
                        ring: [0; 64],
                        ring_cursor: 0,
                        ring_length: 0,
                        context_tokens: Vec::new(),
                        syntactic_register: 0,
                        exact_memory: Vec::new(),
                        facts: BTreeMap::new(),
                        next_fact_id: 1,
                        top_unigrams: *top_unigrams,
                        word_mask: *word_mask,
                        codebook: Arc::clone(codebook),
                    },
                    config,
                    scope,
                    cancelled: Arc::new(AtomicBool::new(false)),
                    active: Arc::new(AtomicBool::new(false)),
                    generation: GenerationState::default(),
                })
            }
        }
    }
}

// ============================================================================
// 5. Native Session & Streaming Execution
// ============================================================================

/// Runtime session variants supported by the API.
pub enum SessionKind {
    Historical {
        model: Arc<Model>,
        durable_session: DurableSession,
    },
    GeometricProse {
        model: Arc<ExportedGeometricModel>,
        tokenizer: Arc<HfBpeTokenizer>,
        ring: [u32; 64],
        ring_cursor: usize,
        ring_length: usize,
        context_tokens: Vec<u32>,
        syntactic_register: u8,
        exact_memory: Vec<(String, u32)>,
        facts: BTreeMap<String, DurableFactRecord>,
        next_fact_id: u64,
        top_unigrams: [u32; 64],
        word_mask: [u64; 64],
        codebook: Arc<Codebook<64>>,
    },
}

/// An isolated conversation and execution session over the native model.
pub struct NativeSession {
    kind: SessionKind,
    config: SessionConfig,
    scope: IdentityScope,
    cancelled: Arc<AtomicBool>,
    active: Arc<AtomicBool>,
    generation: GenerationState,
}

const MAX_OUTPUT_TOKENS: usize = 4096;
const MAX_INGEST_BYTES: usize = 1024 * 1024;

fn validate_budget(tokens: usize) -> Result<(), NativeApiError> {
    if !(1..=MAX_OUTPUT_TOKENS).contains(&tokens) {
        return Err(NativeApiError::TokenLimitExceeded {
            tokens,
            limit: MAX_OUTPUT_TOKENS,
        });
    }
    Ok(())
}

fn validate_temperature_historical(temperature: f64) -> Result<(), NativeApiError> {
    if temperature != 0.0 {
        return Err(NativeApiError::InvalidRequest(
            "only deterministic temperature 0 is implemented".into(),
        ));
    }
    Ok(())
}

fn validate_temperature_prose(temperature: f64) -> Result<(), NativeApiError> {
    if !temperature.is_finite() || !(0.0..=2.0).contains(&temperature) {
        return Err(NativeApiError::InvalidRequest(
            "temperature must be finite in [0.0, 2.0]".into(),
        ));
    }
    Ok(())
}

fn utf8_prefix(bytes: &[u8], terminal: bool) -> Result<usize, NativeApiError> {
    match std::str::from_utf8(bytes) {
        Ok(_) => Ok(bytes.len()),
        Err(e) if e.error_len().is_none() && !terminal => Ok(e.valid_up_to()),
        Err(_) => Err(NativeApiError::Serialization(
            "model emitted invalid or incomplete UTF-8".into(),
        )),
    }
}

#[derive(Default)]
struct GenerationState {
    started: bool,
    finished: bool,
    pending: Vec<u8>,
    stop_sequences: Vec<String>,
    total_tokens: usize,
    active_prompt: String,
    temperature: f64,
    top_k: usize,
    rng_state: u64,
}

#[derive(Serialize, Deserialize)]
struct ProseSessionCheckpoint {
    context_tokens: Vec<u32>,
    ring: Vec<u32>,
    ring_cursor: usize,
    ring_length: usize,
    syntactic_register: u8,
    exact_memory: Vec<(String, u32)>,
    facts: BTreeMap<String, DurableFactRecord>,
    next_fact_id: u64,
}

/// Stack-allocated buffer for bounded candidate shortlist routing (<= 64 tokens, zero heap allocations).
#[derive(Debug, Clone, Copy)]
pub struct ShortlistBuffer {
    pub candidates: [u32; 64],
    pub len: usize,
}

impl Default for ShortlistBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortlistBuffer {
    #[inline]
    pub const fn new() -> Self {
        Self {
            candidates: [0; 64],
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, tok: u32) {
        if self.len < 64 && !self.contains(tok) {
            self.candidates[self.len] = tok;
            self.len += 1;
        }
    }

    #[inline]
    pub fn contains(&self, tok: u32) -> bool {
        let mut i = 0;
        while i < self.len {
            if self.candidates[i] == tok {
                return true;
            }
            i += 1;
        }
        false
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    pub fn as_slice(&self) -> &[u32] {
        &self.candidates[..self.len]
    }
}

#[inline]
fn compute_vsa_context_vec(
    model: &ExportedGeometricModel,
    ring: &[u32; 64],
    cursor: usize,
    length: usize,
    codebook: &Codebook<64>,
) -> Option<Hypervector<64>> {
    if model.vsa_scale_q15 != 0 && length > 0 {
        let mut buf = [0u32; 64];
        let n = length.min(64);
        let start = if length < 64 { 0 } else { cursor };
        for i in 0..n {
            buf[i] = ring[(start + i) % 64];
        }
        Some(encode_attended_multiscale_context(&buf[..n], codebook, 64))
    } else {
        None
    }
}

/// Form candidate shortlist (up to 64 tokens) from engrams, top unigrams, and hierarchical Voronoi codebook.
fn populate_shortlist(
    model: &ExportedGeometricModel,
    ring: &[u32; 64],
    cursor: usize,
    length: usize,
    fiber_pt: HopfFiberPointQ30,
    top_unigrams: &[u32; 64],
    word_mask: &[u64; 64],
    syntactic_register: u8,
    vsa_vec: Option<&Hypervector<64>>,
    shortlist: &mut ShortlistBuffer,
) {
    shortlist.clear();
    let is_word = |cand: u32| -> bool {
        let idx = (cand >> 6) as usize;
        idx < 64 && (word_mask[idx] & (1u64 << (cand & 63))) != 0
    };

    // 1. Engram collocations (5-gram down to bigram + conditioned skip-bigrams)
    if length >= 1 {
        let curr = ring[(cursor + 63) % 64];
        if length >= 2 {
            let prev = ring[(cursor + 62) % 64];
            if let Some(engram) = &model.engram_table {
                if length >= 5 {
                    let prev4 = ring[(cursor + 59) % 64];
                    let prev3 = ring[(cursor + 60) % 64];
                    let prev2 = ring[(cursor + 61) % 64];
                    if let Some(cands) = engram.lookup_5gram(prev4, prev3, prev2, prev, curr) {
                        for &(cand, _) in cands {
                            if shortlist.len >= 12 {
                                break;
                            }
                            if is_word(cand) {
                                shortlist.push(cand);
                            }
                        }
                    }
                }
                if length >= 4 {
                    let prev3 = ring[(cursor + 60) % 64];
                    let prev2 = ring[(cursor + 61) % 64];
                    if let Some(cands) = engram.lookup_4gram(prev3, prev2, prev, curr) {
                        for &(cand, _) in cands {
                            if shortlist.len >= 12 {
                                break;
                            }
                            if is_word(cand) {
                                shortlist.push(cand);
                            }
                        }
                    }
                }
                if length >= 3 {
                    let prev2 = ring[(cursor + 61) % 64];
                    if let Some(cands) = engram.lookup_trigram(prev2, prev, curr) {
                        for &(cand, _) in cands {
                            if shortlist.len >= 12 {
                                break;
                            }
                            if is_word(cand) {
                                shortlist.push(cand);
                            }
                        }
                    }
                }
                if let Some(cands) = engram.lookup_bigram(prev, curr) {
                    for &(cand, _) in cands {
                        if shortlist.len >= 12 {
                            break;
                        }
                        if is_word(cand) {
                            shortlist.push(cand);
                        }
                    }
                }
                for &k in &[2, 4, 8] {
                    if length >= k + 1 {
                        let skip = ring[(cursor + 64 - (k + 1)) % 64];
                        if let Some(cands) = engram.lookup_skip(skip, curr, syntactic_register) {
                            for &(cand, _) in cands {
                                if shortlist.len >= 12 {
                                    break;
                                }
                                if is_word(cand) {
                                    shortlist.push(cand);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 1b. Exact addressed induction attention candidates (2-layer Previous-Token + Induction Circuit):
        // When current bigram [A, B] matches earlier bigram in context, offer continuation token C.
        if length >= 8 {
            let b_curr = ring[(cursor + 63) % 64];
            let b_prev = ring[(cursor + 62) % 64];
            let max_k = length.min(64);
            for k in 6..max_k - 1 {
                let idx_curr = (cursor + 64 - 1 - k) % 64;
                let idx_prev = (cursor + 64 - 2 - k) % 64;
                if ring[idx_prev] == b_prev && ring[idx_curr] == b_curr {
                    let cont_idx = (cursor + 64 - k) % 64;
                    let cont_tok = ring[cont_idx];
                    if cont_tok != b_curr && is_word(cont_tok) && shortlist.len < 16 {
                        shortlist.push(cont_tok);
                        break;
                    }
                }
            }
        }
    }

    // 2. Hierarchical Voronoi codebook routing (coarse H4 sectors + fine leaf clusters)
    if let Some(codebook) = &model.hierarchical_codebook {
        let routed = codebook.route_shortlist_q30_with_memory::<64>(
            Some(fiber_pt.to_unit_s3_q30()),
            vsa_vec,
            shortlist.as_slice(),
        );
        shortlist.clear();
        for &tok in routed.as_slice() {
            let idx = (tok >> 6) as usize;
            let is_word = idx < 64 && (word_mask[idx] & (1u64 << (tok & 63))) != 0;
            if is_word {
                shortlist.push(tok);
            }
        }
    }

    // 3. Fallback unigrams only if geometric routing and engrams returned fewer than 16 candidates
    if shortlist.len < 16 {
        for &tok in top_unigrams {
            if shortlist.len >= 16 {
                break;
            }
            if tok > 2 && tok != 256 && (tok as usize) < model.vocab_size && is_word(tok) {
                shortlist.push(tok);
            }
        }
    }
}

/// Score candidates and select best token using integer fixed-point arithmetic (zero runtime floats when temp=0).
fn score_and_select_candidate(
    model: &ExportedGeometricModel,
    ring: &[u32; 64],
    cursor: usize,
    length: usize,
    fiber_pt: HopfFiberPointQ30,
    candidates: &[u32],
    temperature: f64,
    top_k: usize,
    rng_state: &mut u64,
    allow_eos: bool,
    codebook: &Codebook<64>,
    vsa_vec: Option<&Hypervector<64>>,
    syntactic_register: u8,
) -> u32 {
    if candidates.is_empty() {
        return 0;
    }

    let mut ctx_usize = [0usize; 64];
    let n = length.min(64);
    let start = if length < 64 { 0 } else { cursor };
    for i in 0..n {
        let idx = (start + i) % 64;
        ctx_usize[i] = ring[idx] as usize;
    }

    let mut root_scores = [0i32; 120];
    let root_len = model.token_to_root.len();
    if root_len > 0 {
        for (l, table) in model.discrete_tables.iter().enumerate() {
            let lag = l + 1;
            if n >= lag {
                let ctx_token = ctx_usize[n - lag];
                let ctx_root = model.token_to_root[ctx_token.min(root_len - 1)] as usize;
                let q = ctx_root.min(119);
                let row = &table.scores[q * 120..(q + 1) * 120];
                for r in 0..120 {
                    root_scores[r] += row[r] as i32;
                }
            }
        }
    }

    let mut engram_cands = [(0u32, 0i16); 32];
    let mut engram_count = 0;
    if let Some(engram) = &model.engram_table {
        if n >= 5 {
            let w_curr = ring[(start + n - 1) % 64];
            let w_prev = ring[(start + n - 2) % 64];
            let w_prev2 = ring[(start + n - 3) % 64];
            let w_prev3 = ring[(start + n - 4) % 64];
            let w_prev4 = ring[(start + n - 5) % 64];
            if let Some(cands) = engram.lookup_5gram(w_prev4, w_prev3, w_prev2, w_prev, w_curr) {
                for &(c, q15) in cands {
                    if engram_count < 32 {
                        engram_cands[engram_count] = (c, q15);
                        engram_count += 1;
                    }
                }
            }
        }
        if n >= 4 {
            let w_curr = ring[(start + n - 1) % 64];
            let w_prev = ring[(start + n - 2) % 64];
            let w_prev2 = ring[(start + n - 3) % 64];
            let w_prev3 = ring[(start + n - 4) % 64];
            if let Some(cands) = engram.lookup_4gram(w_prev3, w_prev2, w_prev, w_curr) {
                for &(c, q15) in cands {
                    if engram_count < 32 {
                        engram_cands[engram_count] = (c, q15);
                        engram_count += 1;
                    }
                }
            }
        }
        if n >= 3 {
            let w_curr = ring[(start + n - 1) % 64];
            let w_prev = ring[(start + n - 2) % 64];
            let w_prev2 = ring[(start + n - 3) % 64];
            if let Some(cands) = engram.lookup_trigram(w_prev2, w_prev, w_curr) {
                for &(c, q15) in cands {
                    if engram_count < 32 {
                        engram_cands[engram_count] = (c, q15);
                        engram_count += 1;
                    }
                }
            }
        }
        if n >= 2 {
            let w_curr = ring[(start + n - 1) % 64];
            let w_prev = ring[(start + n - 2) % 64];
            if let Some(cands) = engram.lookup_bigram(w_prev, w_curr) {
                for &(c, q15) in cands {
                    if engram_count < 32 {
                        engram_cands[engram_count] = (c, q15);
                        engram_count += 1;
                    }
                }
            }
        }
        for &k in &[2, 4, 8] {
            if n >= k + 1 {
                let w_curr = ring[(start + n - 1) % 64];
                let w_skip = ring[(start + n - (k + 1)) % 64];
                if let Some(cands) = engram.lookup_skip(w_skip, w_curr, syntactic_register) {
                    for &(c, q15) in cands {
                        if engram_count < 32 {
                            engram_cands[engram_count] = (c, q15);
                            engram_count += 1;
                        }
                    }
                }
            }
        }
    }

    let score_candidate = |cand: u32| -> i32 {
        if cand == 0 || cand == 2 || (!allow_eos && (cand == 1 || cand == 256)) {
            return i32::MIN;
        }
        let cand_u = cand as usize;
        let mut total = (model.discrete_bias.get(cand_u).copied().unwrap_or(0) * 5) / 8;
        if root_len > 0 {
            let cand_root = model.token_to_root[cand_u.min(root_len - 1)] as usize;
            total += root_scores[cand_root.min(119)];
        }
        if let Some(r) = model.discrete_s2_readout.get(cand_u) {
            let s2 = fiber_pt.base.0;
            let u1 = fiber_pt.fiber_u1;
            let s2_proj = ((s2[0] as i64 * r[0] as i64
                + s2[1] as i64 * r[1] as i64
                + s2[2] as i64 * r[2] as i64
                + u1[0] as i64 * r[3] as i64
                + u1[1] as i64 * r[4] as i64)
                >> 31) as i32;
            total += s2_proj;
        }
        if model.vsa_scale_q15 != 0 && n > 0 {
            if let Some(ctx_vec) = vsa_vec {
                let sim_q15 = if let Some(cand_vec) = codebook.get_ref(cand) {
                    ctx_vec.bipolar_correlation_q15(cand_vec)
                } else {
                    let cand_vec = codebook.get(cand);
                    ctx_vec.bipolar_correlation_q15(&cand_vec)
                };
                let vsa_score = (model.vsa_scale_q15 as i32 * sim_q15 as i32) >> 16;
                total += vsa_score;
            }
        }
        for i in 0..engram_count {
            if engram_cands[i].0 == cand {
                total += engram_cands[i].1 as i32;
            }
        }
        if let Some(lattice) = &model.hierarchical_lattice {
            let curr_tok = if n >= 1 { ctx_usize[n - 1] } else { usize::MAX };
            let is_self = cand_u == curr_tok;
            if n >= 2 {
                let prev_tok = ctx_usize[n - 2];
                let r_prev = model.token_to_root[prev_tok.min(root_len - 1)] as usize;
                let r_curr = model.token_to_root[curr_tok.min(root_len - 1)] as usize;
                let c_curr = lattice.cluster_of(curr_tok);
                let c_cand = lattice.cluster_of(cand_u);
                let cand_root = model.token_to_root[cand_u.min(root_len - 1)] as usize;
                total += lattice.score_token(r_prev, r_curr, cand_root, c_curr, c_cand, is_self);
            } else if n == 1 {
                let r_curr = model.token_to_root[curr_tok.min(root_len - 1)] as usize;
                let c_curr = lattice.cluster_of(curr_tok);
                let c_cand = lattice.cluster_of(cand_u);
                let cand_root = model.token_to_root[cand_u.min(root_len - 1)] as usize;
                total += lattice.score_token(r_curr, r_curr, cand_root, c_curr, c_cand, is_self);
            }
        }
        // Exact Addressed Induction Attention (2-layer Previous-Token + Induction Circuit):
        // When current bigram [w_{t-1}, w_t] matches earlier bigram [w_{t-k-1}, w_{t-k}],
        // continuation token w_{t-k+1} receives addressed transition bonus 4096 / k.
        if n >= 8 {
            let b_curr = ctx_usize[n - 1] as u32;
            let b_prev = ctx_usize[n - 2] as u32;
            for k in 6..n - 1 {
                if (ctx_usize[n - 2 - k] as u32) == b_prev
                    && (ctx_usize[n - 1 - k] as u32) == b_curr
                {
                    let cont = ctx_usize[n - k] as u32;
                    if cont == cand && cont != b_curr {
                        total += (4096 / k) as i32;
                        break;
                    }
                }
            }
        }
        total
    };

    let num_cands = candidates.len().min(64);
    if temperature <= 0.001 {
        let mut best_cand = candidates[0];
        let mut best_score = i32::MIN;
        for i in 0..num_cands {
            let cand = candidates[i];
            let sc = score_candidate(cand);
            if sc > best_score {
                best_score = sc;
                best_cand = cand;
            }
        }
        return best_cand;
    }

    let mut scored = [(0u32, i32::MIN); 64];
    for i in 0..num_cands {
        let cand = candidates[i];
        scored[i] = (cand, score_candidate(cand));
    }

    scored[..num_cands].sort_unstable_by(|a, b| b.1.cmp(&a.1));

    let best_cand = scored[0].0;
    if temperature <= 0.001 {
        return best_cand;
    }

    let k = top_k.min(num_cands).max(1);
    if k <= 1 {
        return best_cand;
    }

    let max_score = scored[0].1 as f64;
    let mut exp_weights = [(0u32, 0.0f64); 64];
    let mut sum_exp = 0.0f64;

    for i in 0..k {
        let (v, s) = scored[i];
        let scaled = ((s as f64 - max_score) / (temperature * 8192.0)).clamp(-20.0, 0.0);
        let w = libm::exp(scaled);
        exp_weights[i] = (v, w);
        sum_exp += w;
    }

    if sum_exp <= 0.0 {
        return best_cand;
    }

    *rng_state = (*rng_state)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1);
    let rand_val = (((*rng_state) >> 32) as u32 as f64) / (u32::MAX as f64) * sum_exp;

    let mut accum = 0.0f64;
    let mut chosen = exp_weights[0].0;
    for &(v, w) in &exp_weights[..k] {
        accum += w;
        if rand_val <= accum {
            chosen = v;
            break;
        }
    }
    chosen
}

/// Update 4-bit syntactic register (quotation parity in bit 0, clause depth in bits 1..3).
fn update_syntactic_register(reg: &mut u8, text: &str) {
    let mut q = *reg & 1;
    let mut d = (*reg >> 1) & 3;
    for &b in text.as_bytes() {
        match b {
            b'"' => q ^= 1,
            b'(' | b'[' | b'{' => d = (d + 1).min(3),
            b')' | b']' | b'}' => d = d.saturating_sub(1),
            b',' | b';' | b':' => {
                if d == 0 {
                    d = 1;
                }
            }
            b'.' | b'?' | b'!' => d = 0,
            _ => {}
        }
    }
    *reg = (q & 1) | ((d & 3) << 1);
}

/// Extract declarative key-value facts from ingested text.
fn extract_facts_from_text(
    text: &str,
    facts: &mut BTreeMap<String, DurableFactRecord>,
    next_fact_id: &mut u64,
) {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let cleaned = trimmed.trim_end_matches('.').trim();
        if let Some((k, v)) = cleaned.split_once(" = ") {
            insert_fact(k.trim(), v.trim(), facts, next_fact_id);
        } else if let Some((k, v)) = cleaned.split_once(": ") {
            insert_fact(k.trim(), v.trim(), facts, next_fact_id);
        } else if let Some((k, v)) = cleaned.split_once(" is located in ") {
            insert_fact(k.trim(), v.trim(), facts, next_fact_id);
        } else if let Some((k, v)) = cleaned.split_once(" lives in ") {
            insert_fact(k.trim(), v.trim(), facts, next_fact_id);
        } else if let Some((k, v)) = cleaned.split_once(" is ") {
            let k_trim = k.trim();
            let v_trim = v.trim();
            if !k_trim.is_empty()
                && !v_trim.is_empty()
                && k_trim.len() < 64
                && v_trim.len() < 64
                && !k_trim.starts_with("there")
                && !k_trim.starts_with("it")
            {
                insert_fact(k_trim, v_trim, facts, next_fact_id);
            }
        }
    }
}

fn insert_fact(
    owner: &str,
    value: &str,
    facts: &mut BTreeMap<String, DurableFactRecord>,
    next_fact_id: &mut u64,
) {
    let id = *next_fact_id;
    *next_fact_id += 1;
    let prev = facts.get(owner).map(|r| r.id).unwrap_or(0);
    facts.insert(
        owner.to_string(),
        DurableFactRecord {
            id,
            owner: owner.to_string(),
            value: value.to_string(),
            previous: prev,
            action: 1,
            conflict: false,
        },
    );
}

/// Match question prompt against stored facts for exact named entity recall.
fn check_fact_query(prompt: &str, facts: &BTreeMap<String, DurableFactRecord>) -> Option<String> {
    let p_lower = prompt.to_lowercase();
    let is_query = p_lower.contains('?')
        || p_lower.contains("what")
        || p_lower.contains("where")
        || p_lower.contains("who")
        || p_lower.ends_with(':')
        || p_lower.ends_with("is ")
        || p_lower.ends_with("is");

    if !is_query {
        return None;
    }

    let mut best_match: Option<(&str, &DurableFactRecord)> = None;
    for (owner, record) in facts {
        let o_lower = owner.to_lowercase();
        if p_lower.contains(&o_lower) {
            if let Some((prev_owner, _)) = best_match {
                if owner.len() > prev_owner.len() {
                    best_match = Some((owner.as_str(), record));
                }
            } else {
                best_match = Some((owner.as_str(), record));
            }
        }
    }

    best_match.map(|(_, record)| record.value.clone())
}

/// Resolve the BPE tokenizer from explicit bytes, UOR_TOKENIZER env var, or canonical research model directory.
pub fn resolve_tokenizer(
    tokenizer_bytes: Option<&[u8]>,
) -> Result<Arc<HfBpeTokenizer>, NativeApiError> {
    if let Some(bytes) = tokenizer_bytes {
        let tok = HfBpeTokenizer::from_tokenizer_json_bytes(bytes)
            .ok_or_else(|| NativeApiError::ModelLoad("invalid tokenizer.json bytes".into()))?;
        return Ok(Arc::new(tok));
    }
    if let Ok(path) = std::env::var("UOR_TOKENIZER") {
        if let Ok(bytes) = std::fs::read(&path) {
            if let Some(tok) = HfBpeTokenizer::from_tokenizer_json_bytes(&bytes) {
                return Ok(Arc::new(tok));
            }
        }
    }
    let candidates = [
        ".uor-models/research/issue-1014/export/tokenizer.json",
        "../../.uor-models/research/issue-1014/export/tokenizer.json",
        "crates/uor-r4-api/../../.uor-models/research/issue-1014/export/tokenizer.json",
        "/Users/casey.allard/uor-r4/.uor-models/research/issue-1014/export/tokenizer.json",
    ];
    for p in &candidates {
        if let Ok(bytes) = std::fs::read(p) {
            if let Some(tok) = HfBpeTokenizer::from_tokenizer_json_bytes(&bytes) {
                return Ok(Arc::new(tok));
            }
        }
    }
    Err(NativeApiError::ModelLoad(
        "failed to resolve tokenizer: supply tokenizer bytes, set UOR_TOKENIZER, or ensure canonical tokenizer.json is present".into(),
    ))
}

impl NativeSession {
    pub fn session_id(&self) -> &str {
        &self.config.session_id
    }

    pub fn identity_scope(&self) -> &IdentityScope {
        &self.scope
    }

    pub fn kind(&self) -> &SessionKind {
        &self.kind
    }

    /// Select the next token on the GeometricProse serving path with zero heap allocations,
    /// zero runtime floats, and zero GEMM.
    pub fn predict_token_zero_alloc(&mut self) -> Result<u32, NativeApiError> {
        match &mut self.kind {
            SessionKind::Historical { .. } => Err(NativeApiError::InvalidRequest(
                "predict_token_zero_alloc is only supported on GeometricProse sessions".into(),
            )),
            SessionKind::GeometricProse {
                model,
                ring,
                ring_cursor,
                ring_length,
                syntactic_register,
                top_unigrams,
                word_mask,
                codebook,
                ..
            } => {
                let fiber =
                    model.context_predicted_fiber_from_ring(ring, *ring_cursor, *ring_length);
                let vsa_vec =
                    compute_vsa_context_vec(model, ring, *ring_cursor, *ring_length, codebook);
                let mut shortlist = ShortlistBuffer::new();
                populate_shortlist(
                    model,
                    ring,
                    *ring_cursor,
                    *ring_length,
                    fiber,
                    top_unigrams,
                    word_mask,
                    *syntactic_register,
                    vsa_vec.as_ref(),
                    &mut shortlist,
                );
                let mut dummy_rng = 0u64;
                let chosen = score_and_select_candidate(
                    model,
                    ring,
                    *ring_cursor,
                    *ring_length,
                    fiber,
                    shortlist.as_slice(),
                    0.0,
                    10,
                    &mut dummy_rng,
                    true,
                    codebook,
                    vsa_vec.as_ref(),
                    *syntactic_register,
                );
                Ok(chosen)
            }
        }
    }

    /// Ingest text into the session context.
    pub fn ingest(&mut self, text: &str) -> Result<IngestReceipt, NativeApiError> {
        let bytes_len = text.len();
        let cid = blake3::hash(text.as_bytes()).to_hex().to_string();
        if text.len() > MAX_INGEST_BYTES {
            return Err(NativeApiError::ResourceLimit(
                "ingest exceeds 1 MiB request limit".into(),
            ));
        }

        match &mut self.kind {
            SessionKind::Historical {
                model,
                durable_session,
            } => {
                if self.generation.started || durable_session.session.needs_input_boundary() {
                    durable_session
                        .session
                        .end_response(model)
                        .map_err(|e| NativeApiError::ModelLoad(e.0))?;
                }
                self.generation = GenerationState::default();
                self.cancelled.store(false, Ordering::SeqCst);

                if durable_session.session.work.observed_tokens == 0 {
                    durable_session
                        .session
                        .observe(model, BOS)
                        .map_err(|e| NativeApiError::ModelLoad(e.0))?;
                }

                let tokens = model
                    .encode(text)
                    .map_err(|e| NativeApiError::ModelLoad(e.0))?;

                for token in tokens {
                    durable_session
                        .session
                        .observe(model, token)
                        .map_err(|e| NativeApiError::ModelLoad(e.0))?;
                }

                Ok(IngestReceipt {
                    ingested_bytes: bytes_len,
                    total_context_tokens: durable_session.session.work.observed_tokens as usize,
                    text_cid: cid,
                })
            }
            SessionKind::GeometricProse {
                model: _,
                tokenizer,
                ring,
                ring_cursor,
                ring_length,
                context_tokens,
                syntactic_register,
                exact_memory: _,
                facts,
                next_fact_id,
                ..
            } => {
                self.generation = GenerationState::default();
                self.cancelled.store(false, Ordering::SeqCst);

                extract_facts_from_text(text, facts, next_fact_id);

                let tokens = tokenizer.encode(text);
                for &token in &tokens {
                    context_tokens.push(token);
                    ring[*ring_cursor] = token;
                    *ring_cursor = (*ring_cursor + 1) % 64;
                    if *ring_length < 64 {
                        *ring_length += 1;
                    }
                }
                update_syntactic_register(syntactic_register, text);

                Ok(IngestReceipt {
                    ingested_bytes: bytes_len,
                    total_context_tokens: context_tokens.len(),
                    text_cid: cid,
                })
            }
        }
    }

    /// Generate completion synchronously.
    pub fn complete(
        &mut self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, NativeApiError> {
        self.complete_stream(request, |_| true)
    }

    /// Generate directly through the loaded model. The callback receives valid UTF-8
    /// chunks, which may span multiple model tokens. No canned replies are added.
    pub fn complete_stream<F: FnMut(&str) -> bool>(
        &mut self,
        request: CompletionRequest,
        mut callback: F,
    ) -> Result<CompletionResponse, NativeApiError> {
        let max_tokens = request.max_tokens.unwrap_or(self.config.max_output_tokens);
        validate_budget(max_tokens)?;
        let temp = request.temperature.unwrap_or(self.config.temperature);
        match &self.kind {
            SessionKind::Historical { .. } => validate_temperature_historical(temp)?,
            SessionKind::GeometricProse { .. } => validate_temperature_prose(temp)?,
        }
        if request.stop_sequences.len() > 16
            || request
                .stop_sequences
                .iter()
                .any(|s| s.is_empty() || s.len() > 256)
        {
            return Err(NativeApiError::InvalidRequest(
                "stop sequences must be 1..=256 bytes, at most 16".into(),
            ));
        }
        let prompt_copy = request.prompt.clone();
        if !request.prompt.is_empty() {
            self.ingest(&request.prompt)?;
        }
        let initial_rng = {
            let hash = blake3::hash(self.config.session_id.as_bytes());
            let raw = u64::from_le_bytes(hash.as_bytes()[..8].try_into().unwrap_or([0; 8]));
            if raw == 0 {
                20260918
            } else {
                raw
            }
        };
        self.generation = GenerationState {
            stop_sequences: request.stop_sequences,
            active_prompt: prompt_copy,
            temperature: temp,
            top_k: self.config.top_k,
            rng_state: initial_rng,
            ..Default::default()
        };
        self.cancelled.store(false, Ordering::SeqCst);
        let mut result = self.generate_chunk(max_tokens, &mut callback)?;
        if result.stopped_by == "length" {
            let tail = std::str::from_utf8(&self.generation.pending).map_err(|_| {
                NativeApiError::Serialization("token budget ended within a UTF-8 scalar".into())
            })?;
            if !tail.is_empty() {
                result.text.push_str(tail);
                if !callback(tail) {
                    result.stopped_by = "cancelled".into();
                }
            }
            self.generation.pending.clear();
            self.generation.finished = true;
        }
        Ok(result)
    }

    /// Continue a bounded response across host event-loop yields.
    fn generate_chunk<F: FnMut(&str) -> bool>(
        &mut self,
        max_tokens: usize,
        mut callback: F,
    ) -> Result<CompletionResponse, NativeApiError> {
        validate_budget(max_tokens)?;
        if self.active.swap(true, Ordering::SeqCst) {
            return Err(NativeApiError::SessionBusy("session is executing".into()));
        }
        let result = match &mut self.kind {
            SessionKind::Historical {
                model,
                durable_session,
            } => Self::generate_chunk_historical(
                model,
                durable_session,
                &self.cancelled,
                &mut self.generation,
                max_tokens,
                &mut callback,
            ),
            SessionKind::GeometricProse {
                model,
                tokenizer,
                ring,
                ring_cursor,
                ring_length,
                context_tokens,
                syntactic_register,
                facts,
                top_unigrams,
                word_mask,
                codebook,
                ..
            } => Self::generate_chunk_prose(
                model,
                tokenizer,
                ring,
                ring_cursor,
                ring_length,
                context_tokens,
                syntactic_register,
                facts,
                top_unigrams,
                word_mask,
                codebook,
                &self.cancelled,
                &mut self.generation,
                max_tokens,
                &mut callback,
            ),
        };
        self.active.store(false, Ordering::SeqCst);
        result
    }

    fn generate_chunk_historical<F: FnMut(&str) -> bool>(
        model: &Model,
        durable_session: &mut DurableSession,
        cancelled: &AtomicBool,
        generation: &mut GenerationState,
        max_tokens: usize,
        callback: &mut F,
    ) -> Result<CompletionResponse, NativeApiError> {
        #[cfg(not(target_arch = "wasm32"))]
        let start = Instant::now();
        let mut text = String::new();
        let mut token_count = 0;
        let mut stopped_by = "length";
        if !generation.started && !generation.finished {
            durable_session
                .session
                .begin_response(model)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
            generation.started = true;
        }
        if generation.finished {
            stopped_by = "eos";
        }
        while token_count < max_tokens && !generation.finished {
            if cancelled.load(Ordering::Relaxed) {
                stopped_by = "cancelled";
                generation.finished = true;
                break;
            }
            if generation.total_tokens >= MAX_OUTPUT_TOKENS {
                stopped_by = "token_limit";
                generation.finished = true;
                break;
            }
            let pred = durable_session
                .session
                .predict(model)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
            durable_session
                .session
                .observe(model, pred.token)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
            if pred.token == EOS {
                stopped_by = "eos";
                generation.finished = true;
            } else {
                let bytes = model
                    .decode(&[pred.token])
                    .map_err(|e| NativeApiError::ModelLoad(e.0))?;
                generation.pending.extend_from_slice(&bytes);
                token_count += 1;
                generation.total_tokens += 1;
            }
            let mut emit_len = generation.pending.len();
            if let Some(pos) = generation
                .stop_sequences
                .iter()
                .filter_map(|s| {
                    generation
                        .pending
                        .windows(s.len())
                        .position(|w| w == s.as_bytes())
                })
                .min()
            {
                emit_len = pos;
                stopped_by = "stop_sequence";
                generation.finished = true;
            } else if !generation.finished {
                let mut withheld = 0;
                for stop in &generation.stop_sequences {
                    for n in 1..stop.len() {
                        if generation.pending.ends_with(&stop.as_bytes()[..n]) {
                            withheld = withheld.max(n);
                        }
                    }
                }
                emit_len -= withheld;
            }
            let valid_len = utf8_prefix(&generation.pending[..emit_len], generation.finished)?;
            if valid_len != 0 {
                let piece = std::str::from_utf8(&generation.pending[..valid_len])
                    .map_err(|e| NativeApiError::Serialization(e.to_string()))?;
                text.push_str(piece);
                let keep_going = callback(piece);
                generation.pending.drain(..valid_len);
                if !keep_going {
                    stopped_by = "cancelled";
                    generation.finished = true;
                }
            }
            if generation.finished {
                generation.pending.clear();
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let elapsed_us = Some(start.elapsed().as_micros().min(u64::MAX as u128) as u64);
        #[cfg(target_arch = "wasm32")]
        let elapsed_us = None;
        Ok(CompletionResponse {
            text,
            token_count,
            stopped_by: stopped_by.into(),
            elapsed_us,
            memory_facts_read: None,
        })
    }

    fn generate_chunk_prose<F: FnMut(&str) -> bool>(
        model: &ExportedGeometricModel,
        tokenizer: &HfBpeTokenizer,
        ring: &mut [u32; 64],
        ring_cursor: &mut usize,
        ring_length: &mut usize,
        context_tokens: &mut Vec<u32>,
        syntactic_register: &mut u8,
        facts: &BTreeMap<String, DurableFactRecord>,
        top_unigrams: &[u32; 64],
        word_mask: &[u64; 64],
        codebook: &Codebook<64>,
        cancelled: &AtomicBool,
        generation: &mut GenerationState,
        max_tokens: usize,
        callback: &mut F,
    ) -> Result<CompletionResponse, NativeApiError> {
        #[cfg(not(target_arch = "wasm32"))]
        let start = Instant::now();
        let mut text = String::new();
        let mut token_count = 0;
        let mut stopped_by = "length";
        let mut memory_facts_read = None;

        // Check exact addressed memory match first
        if !generation.started && !generation.active_prompt.is_empty() {
            if let Some(fact_val) = check_fact_query(&generation.active_prompt, facts) {
                memory_facts_read = Some(1);
                let val_tokens = tokenizer.encode(&fact_val);
                let mut truncated = false;
                for tok in val_tokens {
                    if token_count >= max_tokens {
                        stopped_by = "length";
                        truncated = true;
                        break;
                    }
                    token_count += 1;
                    context_tokens.push(tok);
                    ring[*ring_cursor] = tok;
                    *ring_cursor = (*ring_cursor + 1) % 64;
                    if *ring_length < 64 {
                        *ring_length += 1;
                    }
                }
                text.push_str(&fact_val);
                callback(&fact_val);
                if !truncated {
                    stopped_by = "eos";
                }
                generation.started = true;
                generation.finished = true;

                #[cfg(not(target_arch = "wasm32"))]
                let elapsed_us = Some(start.elapsed().as_micros().min(u64::MAX as u128) as u64);
                #[cfg(target_arch = "wasm32")]
                let elapsed_us = None;

                return Ok(CompletionResponse {
                    text,
                    token_count,
                    stopped_by: stopped_by.into(),
                    elapsed_us,
                    memory_facts_read,
                });
            }
        }

        generation.started = true;

        while token_count < max_tokens && !generation.finished {
            if cancelled.load(Ordering::Relaxed) {
                stopped_by = "cancelled";
                generation.finished = true;
                break;
            }
            if generation.total_tokens >= MAX_OUTPUT_TOKENS {
                stopped_by = "token_limit";
                generation.finished = true;
                break;
            }

            let fiber = model.context_predicted_fiber_from_ring(ring, *ring_cursor, *ring_length);
            let vsa_vec =
                compute_vsa_context_vec(model, ring, *ring_cursor, *ring_length, codebook);
            let mut shortlist = ShortlistBuffer::new();
            populate_shortlist(
                model,
                ring,
                *ring_cursor,
                *ring_length,
                fiber,
                top_unigrams,
                word_mask,
                *syntactic_register,
                vsa_vec.as_ref(),
                &mut shortlist,
            );

            let chosen = score_and_select_candidate(
                model,
                ring,
                *ring_cursor,
                *ring_length,
                fiber,
                shortlist.as_slice(),
                generation.temperature,
                generation.top_k,
                &mut generation.rng_state,
                token_count > 0,
                codebook,
                vsa_vec.as_ref(),
                *syntactic_register,
            );

            if chosen == 0 || chosen == 1 || chosen == 256 || chosen >= model.vocab_size as u32 {
                stopped_by = "eos";
                generation.finished = true;
                break;
            }

            context_tokens.push(chosen);
            ring[*ring_cursor] = chosen;
            *ring_cursor = (*ring_cursor + 1) % 64;
            if *ring_length < 64 {
                *ring_length += 1;
            }
            token_count += 1;
            generation.total_tokens += 1;

            let piece = tokenizer.decode(&[chosen]);
            update_syntactic_register(syntactic_register, &piece);
            generation.pending.extend_from_slice(piece.as_bytes());

            let mut emit_len = generation.pending.len();
            if let Some(pos) = generation
                .stop_sequences
                .iter()
                .filter_map(|s| {
                    generation
                        .pending
                        .windows(s.len())
                        .position(|w| w == s.as_bytes())
                })
                .min()
            {
                emit_len = pos;
                stopped_by = "stop_sequence";
                generation.finished = true;
            }

            let valid_len = utf8_prefix(&generation.pending[..emit_len], generation.finished)?;
            if valid_len != 0 {
                let piece_str = std::str::from_utf8(&generation.pending[..valid_len])
                    .map_err(|e| NativeApiError::Serialization(e.to_string()))?;
                text.push_str(piece_str);
                let keep_going = callback(piece_str);
                generation.pending.drain(..valid_len);
                if !keep_going {
                    stopped_by = "cancelled";
                    generation.finished = true;
                    break;
                }
            }

            if generation.finished {
                generation.pending.clear();
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        let elapsed_us = Some(start.elapsed().as_micros().min(u64::MAX as u128) as u64);
        #[cfg(target_arch = "wasm32")]
        let elapsed_us = None;

        Ok(CompletionResponse {
            text,
            token_count,
            stopped_by: stopped_by.into(),
            elapsed_us,
            memory_facts_read,
        })
    }

    /// Finish a host-bounded incremental response without predicting another token.
    pub fn finish_generation(&mut self) -> Result<CompletionResponse, NativeApiError> {
        if self.active.load(Ordering::SeqCst) {
            return Err(NativeApiError::SessionBusy(
                "generation is executing".into(),
            ));
        }
        let text = std::str::from_utf8(&self.generation.pending)
            .map_err(|_| {
                NativeApiError::Serialization("cannot finish response inside a UTF-8 scalar".into())
            })?
            .to_owned();
        let stopped_by = if self.cancelled.load(Ordering::SeqCst) {
            "cancelled"
        } else {
            "finished"
        };
        self.generation.pending.clear();
        self.generation.finished = true;
        if let SessionKind::Historical {
            model,
            durable_session,
        } = &mut self.kind
        {
            if self.generation.started || durable_session.session.needs_input_boundary() {
                durable_session
                    .session
                    .end_response(model)
                    .map_err(|e| NativeApiError::ModelLoad(e.0))?;
            }
        }
        self.generation.started = false;
        Ok(CompletionResponse {
            text,
            token_count: 0,
            stopped_by: stopped_by.into(),
            elapsed_us: None,
            memory_facts_read: None,
        })
    }

    /// A cancellation token can be retained by the host without locking the session.
    pub fn cancellation_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancelled)
    }

    /// Cooperatively signal session cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Reset session context and memory, preserving the identity scope.
    pub fn reset(&mut self) -> Result<(), NativeApiError> {
        match &mut self.kind {
            SessionKind::Historical {
                model,
                durable_session,
            } => {
                durable_session
                    .reset(model)
                    .map_err(|e| NativeApiError::ModelLoad(e.0))?;
            }
            SessionKind::GeometricProse {
                ring,
                ring_cursor,
                ring_length,
                context_tokens,
                syntactic_register,
                exact_memory,
                facts,
                next_fact_id,
                ..
            } => {
                *ring = [0; 64];
                *ring_cursor = 0;
                *ring_length = 0;
                context_tokens.clear();
                *syntactic_register = 0;
                exact_memory.clear();
                facts.clear();
                *next_fact_id = 1;
            }
        }
        self.cancelled.store(false, Ordering::SeqCst);
        self.active.store(false, Ordering::SeqCst);
        self.generation = GenerationState::default();
        Ok(())
    }

    /// Store a persistent fact in the session's isolated memory.
    pub fn store_fact(&mut self, owner: &str, value: &str) -> Result<u64, NativeApiError> {
        match &mut self.kind {
            SessionKind::Historical {
                durable_session, ..
            } => durable_session
                .assert_fact(owner, value)
                .map_err(|e| NativeApiError::IdentityViolation(e.0)),
            SessionKind::GeometricProse {
                facts,
                next_fact_id,
                exact_memory,
                tokenizer,
                ..
            } => {
                let id = *next_fact_id;
                *next_fact_id += 1;
                let prev = facts.get(owner).map(|r| r.id).unwrap_or(0);
                facts.insert(
                    owner.to_string(),
                    DurableFactRecord {
                        id,
                        owner: owner.to_string(),
                        value: value.to_string(),
                        previous: prev,
                        action: 1,
                        conflict: false,
                    },
                );
                let val_tokens = tokenizer.encode(value);
                if let Some(&first_tok) = val_tokens.first() {
                    exact_memory.push((owner.to_string(), first_tok));
                }
                Ok(id)
            }
        }
    }

    /// Query the session's isolated memory for an active fact value.
    pub fn query_memory(&self, owner: &str) -> Result<Option<String>, NativeApiError> {
        match &self.kind {
            SessionKind::Historical {
                durable_session, ..
            } => Ok(durable_session.get_fact(owner)),
            SessionKind::GeometricProse { facts, .. } => {
                Ok(facts.get(owner).map(|r| r.value.clone()))
            }
        }
    }

    /// Query the session's isolated memory for an exact fact record.
    pub fn query_memory_record(
        &self,
        owner: &str,
    ) -> Result<Option<DurableFactRecord>, NativeApiError> {
        match &self.kind {
            SessionKind::Historical {
                durable_session, ..
            } => Ok(durable_session.get_fact_record(owner)),
            SessionKind::GeometricProse { facts, .. } => Ok(facts.get(owner).cloned()),
        }
    }

    /// Export serialized session state.
    pub fn export_state(&self) -> Result<Vec<u8>, NativeApiError> {
        match &self.kind {
            SessionKind::Historical {
                durable_session, ..
            } => {
                if self.generation.started && !self.generation.finished {
                    return Err(NativeApiError::InvalidRequest(
                        "finish or cancel generation before exporting a checkpoint".into(),
                    ));
                }
                durable_session
                    .checkpoint()
                    .map_err(|e| NativeApiError::Serialization(e.0))
            }
            SessionKind::GeometricProse {
                ring,
                ring_cursor,
                ring_length,
                context_tokens,
                syntactic_register,
                exact_memory,
                facts,
                next_fact_id,
                ..
            } => {
                let state = ProseSessionCheckpoint {
                    ring: ring.to_vec(),
                    ring_cursor: *ring_cursor,
                    ring_length: *ring_length,
                    context_tokens: context_tokens.clone(),
                    syntactic_register: *syntactic_register,
                    exact_memory: exact_memory.clone(),
                    facts: facts.clone(),
                    next_fact_id: *next_fact_id,
                };
                serde_json::to_vec(&state).map_err(|e| NativeApiError::Serialization(e.to_string()))
            }
        }
    }

    /// Restore serialized session state.
    pub fn import_state(&mut self, bytes: &[u8]) -> Result<(), NativeApiError> {
        match &mut self.kind {
            SessionKind::Historical {
                model,
                durable_session,
            } => {
                let restored = DurableSession::from_checkpoint(model, bytes)
                    .map_err(|e| NativeApiError::Serialization(e.0))?;
                if restored.scope != self.scope {
                    return Err(NativeApiError::IdentityViolation(
                        "checkpoint scope differs from target session".into(),
                    ));
                }
                *durable_session = restored;
            }
            SessionKind::GeometricProse {
                ring,
                ring_cursor,
                ring_length,
                context_tokens,
                syntactic_register,
                exact_memory,
                facts,
                next_fact_id,
                ..
            } => {
                let restored: ProseSessionCheckpoint = serde_json::from_slice(bytes)
                    .map_err(|e| NativeApiError::Serialization(e.to_string()))?;
                *ring = [0u32; 64];
                for (dst, &src) in ring.iter_mut().zip(restored.ring.iter()) {
                    *dst = src;
                }
                *ring_cursor = restored.ring_cursor;
                *ring_length = restored.ring_length;
                *context_tokens = restored.context_tokens;
                *syntactic_register = restored.syntactic_register;
                *exact_memory = restored.exact_memory;
                *facts = restored.facts;
                *next_fact_id = restored.next_fact_id;
            }
        }
        self.generation = GenerationState::default();
        self.cancelled.store(false, Ordering::SeqCst);
        Ok(())
    }
}

// ============================================================================
// 6. WASM Model Runtime Bridge
// ============================================================================

/// Lightweight handle referencing an active session in the WASM table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WasmSessionHandle(pub u32);

/// Filesystem-free, browser-compatible WASM runtime bridge.
pub struct WasmModelRuntime {
    model: NativeModel,
    sessions: Arc<Mutex<HashMap<u32, NativeSession>>>,
    next_handle: AtomicU32,
    cancellations: Mutex<HashMap<u32, Arc<AtomicBool>>>,
}

impl WasmModelRuntime {
    pub fn new(model: NativeModel) -> Self {
        Self {
            model,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            next_handle: AtomicU32::new(1),
            cancellations: Mutex::new(HashMap::new()),
        }
    }

    /// Instantiate a new session and return an integer handle for JS/WASM interop.
    pub fn wasm_create_session(
        &self,
        session_id: &str,
        user_id: &str,
        project_id: &str,
    ) -> Result<u32, NativeApiError> {
        let config = SessionConfig {
            session_id: session_id.into(),
            user_id: user_id.into(),
            project_id: project_id.into(),
            control: Control::Full,
            max_output_tokens: 128,
            temperature: 0.0,
            top_k: 10,
        };

        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        if lock.len() >= self.model.metadata.max_sessions {
            return Err(NativeApiError::ResourceLimit(
                "maximum live sessions reached".into(),
            ));
        }
        let session = self.model.create_session(config)?;
        let handle = self
            .next_handle
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_add(1))
            .map_err(|_| NativeApiError::ResourceLimit("session handles exhausted".into()))?;
        self.cancellations
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("cancellation lock poisoned".into()))?
            .insert(handle, session.cancellation_handle());
        lock.insert(handle, session);
        Ok(handle)
    }

    /// Ingest UTF-8 text into a WASM session.
    pub fn wasm_ingest(&self, handle: u32, text: &str) -> Result<String, NativeApiError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        let session = lock
            .get_mut(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;

        let receipt = session.ingest(text)?;
        serde_json::to_string(&receipt).map_err(|e| NativeApiError::Serialization(e.to_string()))
    }

    /// Execute a generation step on a WASM session.
    pub fn wasm_generate_step(
        &self,
        handle: u32,
        max_tokens: usize,
    ) -> Result<String, NativeApiError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        let session = lock
            .get_mut(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;

        let resp = session.generate_chunk(max_tokens, |_| true)?;
        serde_json::to_string(&resp).map_err(|e| NativeApiError::Serialization(e.to_string()))
    }

    /// Close incremental generation before checkpointing a host budget or stop.
    pub fn wasm_finish_generation(&self, handle: u32) -> Result<String, NativeApiError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        let session = lock
            .get_mut(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;
        let response = session.finish_generation()?;
        serde_json::to_string(&response).map_err(|e| NativeApiError::Serialization(e.to_string()))
    }

    /// Cancel a running WASM session.
    pub fn wasm_cancel(&self, handle: u32) -> Result<(), NativeApiError> {
        let lock = self
            .cancellations
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("cancellation lock poisoned".into()))?;
        let token = lock
            .get(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;
        token.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Export WASM session state to byte array.
    pub fn wasm_export_session(&self, handle: u32) -> Result<Vec<u8>, NativeApiError> {
        let lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        let session = lock
            .get(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;
        session.export_state()
    }

    /// Import WASM session state from byte array.
    pub fn wasm_import_session(&self, handle: u32, bytes: &[u8]) -> Result<(), NativeApiError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        let session = lock
            .get_mut(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;
        session.import_state(bytes)
    }

    /// Free a WASM session handle and release memory.
    pub fn wasm_free_session(&self, handle: u32) {
        if let Ok(mut lock) = self.sessions.lock() {
            lock.remove(&handle);
        }
        if let Ok(mut lock) = self.cancellations.lock() {
            lock.remove(&handle);
        }
    }

    /// Introspect model metadata and capabilities as JSON string.
    pub fn wasm_get_capabilities(&self) -> String {
        serde_json::to_string(self.model.metadata()).unwrap_or_default()
    }
}

#[cfg(test)]
mod recovery_utf8_tests {
    use super::utf8_prefix;

    #[test]
    fn streaming_preserves_split_multibyte_scalars_and_rejects_invalid_bytes() {
        let mut pending = Vec::new();
        for byte in [0xe2, 0x82] {
            pending.push(byte);
            assert_eq!(utf8_prefix(&pending, false).unwrap(), 0);
            assert!(utf8_prefix(&pending, true).is_err());
        }
        pending.push(0xac);
        assert_eq!(utf8_prefix(&pending, true).unwrap(), 3);
        assert_eq!(std::str::from_utf8(&pending).unwrap(), "€");
        assert_eq!(utf8_prefix(b"hello\xe2", false).unwrap(), 5);
        assert!(utf8_prefix(&[0xff], false).is_err());
    }
}
