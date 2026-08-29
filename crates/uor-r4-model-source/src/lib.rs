//! The TEACHER: faithful Rust port of karpathy run.c forward pass (v0 checkpoint).
//! Arithmetic order mirrors the C exactly: sequential adds in matmul rows,
//! rmsnorm/softmax/RoPE/SwiGLU op-for-op, libm via glibc on gnu targets.
//! The Safetensors adapter also loads pinned Hugging Face SmolLM2 weights
//! into this same source-only teacher surface. Llama/shared projections use the
//! pinned exact `uor-matmul` owner. GPT-2 uses its declared
//! certified-native/exact-fallback dense and attention owners; canonical mode
//! separately selects the remaining libm and ordered-reduction family.

pub mod attention;
pub mod conformance;
pub mod dense;
mod exact_executor;
mod exact_probe;
pub mod geometric_decoder;
pub mod geometric_training;
pub mod geometry;
#[cfg(not(target_arch = "wasm32"))]
pub mod gpt2;
/// #804 measurement-only BLAS exception — opt-in feature, macOS only,
/// pinned by the `matrix_operation_census` gate. See the module docs.
#[cfg(all(feature = "observation-blas-exception", target_os = "macos"))]
mod observation_blas_exception;
pub mod progress;
#[cfg(not(target_arch = "wasm32"))]
pub mod teacher;

#[cfg(all(test, not(target_arch = "wasm32")))]
use exact_executor::AtomicTeacherExecutionProgress;
use exact_executor::ExactExecutor;
pub use exact_executor::{
    exact_backend_report, ExactBackendReport, TeacherExecutionConfig, TeacherExecutionObserver,
    TeacherExecutionPreparation, TeacherExecutionSnapshot, UOR_MATMUL_REVISION,
};
#[cfg(not(target_arch = "wasm32"))]
pub use exact_probe::production_admission_component_cids;
pub use exact_probe::{
    exact_executor_contract_cid, exact_probe_host_identity, ExactMulticoreProbeEventsBinding,
    ExactMulticoreProbeExpectation, ExactMulticoreProbeExpectationShapes, ExactMulticoreProbeHost,
    ExactMulticoreProbePrestart, ExactMulticoreProbeReport, ExactMulticoreProbeResources,
    ExactMulticoreProbeRun, ExactMulticoreProbeSelection, ExactMulticoreProbeSource,
    ExactMulticoreProbeStatus, ExactMulticoreProbeTraceShape, ExactMulticoreProbeValidationError,
    ExactMulticoreProbeVerdict, ExactMulticoreProbeWork, ExactMulticoreProbeWorkerPlan,
    EXACT_MULTICORE_PROBE_CONTEXT_ASSUMPTION, EXACT_MULTICORE_PROBE_DEADLINE_POLICY,
    EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_LANES,
    EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_TOKENS,
    EXACT_MULTICORE_PROBE_REGISTERED_MAX_SEQUENCE_POSITION,
    EXACT_MULTICORE_PROBE_REGISTERED_STATE_SEQUENCE_CAPACITY,
    EXACT_MULTICORE_PROBE_REGISTERED_TRANSCRIPT_BATCH_WIDTHS,
    EXACT_MULTICORE_PROBE_REGISTERED_TRANSCRIPT_FORWARDS, EXACT_MULTICORE_PROBE_SCHEMA,
    EXACT_MULTICORE_PROBE_SELECTION_POLICY, EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS,
    PRODUCTION_ADMISSION_COMPONENTS,
};
#[cfg(not(target_arch = "wasm32"))]
pub use gpt2::HuggingFaceGpt2Oracle;
#[cfg(not(target_arch = "wasm32"))]
pub use teacher::Teacher;
pub struct Config {
    pub dim: usize,
    pub hidden: usize,
    pub n_layers: usize,
    pub n_heads: usize,
    pub n_kv_heads: usize,
    pub vocab: usize,
    pub seq_len: usize,
    pub rope_theta: f32,
    pub rms_norm_eps: f32,
    pub rope_interleaved: bool,
    pub r4_attention: bool,
}

/// Exact executor work owned by one serial or batched teacher forward.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ExactForwardPlan {
    /// Independent sequence states advanced together through shared weights.
    pub batch_width: usize,
    /// Exact matrix calls (`7 * layers + vocabulary projection`).
    pub matrix_calls: u64,
    /// Disjoint output-row tiles owned by the executor.
    pub row_tiles: u64,
    /// Scheduler tasks; one task owns each complete output-row tile.
    pub worker_tasks: u64,
    /// Output cells completed across every batch lane.
    pub output_cells: u64,
    /// Scalar product terms absorbed into complete exact accumulators.
    pub scalar_terms: u64,
}

/// Why an exact forward counter plan cannot be represented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExactForwardPlanError {
    /// A forward batch must contain at least one independent sequence.
    EmptyBatch,
    /// A trace-state capacity must be nonzero and within model context.
    InvalidSequenceCapacity,
    /// Model geometry or the requested batch exceeds the counter domain.
    ArithmeticOverflow,
    /// This build routes teacher projections through a non-exact exception.
    ExactBackendUnavailable,
}

impl std::fmt::Display for ExactForwardPlanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "exact forward plan unavailable: {self:?}")
    }
}

impl std::error::Error for ExactForwardPlanError {}

fn exact_forward_plan_for_geometry(
    cfg: &Config,
    batch_width: usize,
    mut row_tiles_for: impl FnMut(usize) -> usize,
) -> Result<ExactForwardPlan, ExactForwardPlanError> {
    use ExactForwardPlanError as Error;

    if batch_width == 0 {
        return Err(Error::EmptyBatch);
    }
    if exact_backend_report().arithmetic_owner != "uor-matmul exact GEMM" {
        return Err(Error::ExactBackendUnavailable);
    }
    let checked_mul =
        |left: usize, right: usize| left.checked_mul(right).ok_or(Error::ArithmeticOverflow);
    let checked_add =
        |left: usize, right: usize| left.checked_add(right).ok_or(Error::ArithmeticOverflow);
    let dim = cfg.dim;
    let hidden = cfg.hidden;
    let layers = cfg.n_layers;
    let kv_dim = checked_mul(dim, cfg.n_kv_heads)?
        .checked_div(cfg.n_heads)
        .ok_or(Error::ArithmeticOverflow)?;
    let matrix_calls = checked_add(checked_mul(layers, 7)?, 1)?;
    let dim_tiles = row_tiles_for(dim);
    let kv_tiles = row_tiles_for(kv_dim);
    let hidden_tiles = row_tiles_for(hidden);
    let layer_tiles = checked_add(
        checked_add(checked_mul(dim_tiles, 3)?, checked_mul(kv_tiles, 2)?)?,
        checked_mul(hidden_tiles, 2)?,
    )?;
    let row_tiles = checked_add(checked_mul(layer_tiles, layers)?, row_tiles_for(cfg.vocab))?;
    let layer_output_rows = checked_add(
        checked_add(checked_mul(dim, 3)?, checked_mul(kv_dim, 2)?)?,
        checked_mul(hidden, 2)?,
    )?;
    let output_rows = checked_add(checked_mul(layer_output_rows, layers)?, cfg.vocab)?;
    let output_cells = checked_mul(output_rows, batch_width)?;
    let dim_squared = checked_mul(dim, dim)?;
    let kv_terms = checked_mul(kv_dim, dim)?;
    let hidden_terms = checked_mul(hidden, dim)?;
    let layer_scalar_terms = checked_add(
        checked_add(checked_mul(dim_squared, 2)?, checked_mul(kv_terms, 2)?)?,
        checked_mul(hidden_terms, 3)?,
    )?;
    let vocabulary_terms = checked_mul(cfg.vocab, dim)?;
    let scalar_terms = checked_mul(
        checked_add(checked_mul(layer_scalar_terms, layers)?, vocabulary_terms)?,
        batch_width,
    )?;
    let as_u64 = |value| u64::try_from(value).map_err(|_| Error::ArithmeticOverflow);
    let row_tiles = as_u64(row_tiles)?;
    Ok(ExactForwardPlan {
        batch_width,
        matrix_calls: as_u64(matrix_calls)?,
        row_tiles,
        worker_tasks: row_tiles,
        output_cells: as_u64(output_cells)?,
        scalar_terms: as_u64(scalar_terms)?,
    })
}

fn exact_probe_trace_shape_for_geometry(
    cfg: &Config,
    sequence_capacity: usize,
    positions: usize,
    batch_width: usize,
    top_k: usize,
) -> Result<ExactMulticoreProbeTraceShape, ExactForwardPlanError> {
    use ExactForwardPlanError as Error;

    if positions == 0 || batch_width == 0 {
        return Err(Error::EmptyBatch);
    }
    if sequence_capacity == 0 || sequence_capacity > cfg.seq_len {
        return Err(Error::InvalidSequenceCapacity);
    }
    let checked_mul =
        |left: usize, right: usize| left.checked_mul(right).ok_or(Error::ArithmeticOverflow);
    let checked_add =
        |left: usize, right: usize| left.checked_add(right).ok_or(Error::ArithmeticOverflow);
    let kv_dim = checked_mul(cfg.dim, cfg.n_kv_heads)?
        .checked_div(cfg.n_heads)
        .ok_or(Error::ArithmeticOverflow)?;
    let cache_words = checked_mul(checked_mul(cfg.n_layers, sequence_capacity)?, kv_dim)?;
    let persistent_state_words_per_state = checked_add(
        checked_add(cfg.dim, checked_mul(cache_words, 2)?)?,
        cfg.vocab,
    )?;
    let state_records = checked_mul(positions, batch_width)?;
    let logit_words = checked_mul(state_records, cfg.vocab)?;
    let persistent_state_words = checked_mul(state_records, persistent_state_words_per_state)?;
    let top_k = top_k.min(cfg.vocab);
    let top_tokens = checked_mul(state_records, top_k)?;
    let as_u64 = |value| u64::try_from(value).map_err(|_| Error::ArithmeticOverflow);
    Ok(ExactMulticoreProbeTraceShape {
        positions,
        streams_per_position: batch_width,
        sequence_capacity,
        state_records: as_u64(state_records)?,
        logits_per_state: cfg.vocab,
        logit_words: as_u64(logit_words)?,
        logit_bytes: as_u64(checked_mul(logit_words, std::mem::size_of::<u32>())?)?,
        persistent_state_words_per_state,
        persistent_state_words: as_u64(persistent_state_words)?,
        greedy_tokens: as_u64(state_records)?,
        top_k,
        top_tokens: as_u64(top_tokens)?,
    })
}

#[derive(Default)]
struct BatchForwardWorkspace {
    norm: Vec<f32>,
    q: Vec<f32>,
    ktmp: Vec<f32>,
    vtmp: Vec<f32>,
    attn: Vec<f32>,
    o: Vec<f32>,
    hb: Vec<f32>,
    hb2: Vec<f32>,
    ffn: Vec<f32>,
    xstack: Vec<f32>,
    logits_stacked: Vec<f32>,
}

impl BatchForwardWorkspace {
    fn grow(values: &mut Vec<f32>, length: usize, executor: &ExactExecutor) {
        if values.len() >= length {
            return;
        }
        let before = values.capacity();
        values.resize(length, 0.0);
        executor.record_workspace_growth_bytes(
            values
                .capacity()
                .saturating_sub(before)
                .saturating_mul(std::mem::size_of::<f32>()),
        );
    }

    fn ensure(&mut self, cfg: &Config, batch: usize, executor: &ExactExecutor) {
        let kv_dim = cfg.dim * cfg.n_kv_heads / cfg.n_heads;
        let dim_words = batch
            .checked_mul(cfg.dim)
            .expect("batched teacher dimension workspace must fit usize");
        let kv_words = batch
            .checked_mul(kv_dim)
            .expect("batched teacher KV workspace must fit usize");
        let hidden_words = batch
            .checked_mul(cfg.hidden)
            .expect("batched teacher hidden workspace must fit usize");
        let logit_words = batch
            .checked_mul(cfg.vocab)
            .expect("batched teacher logit workspace must fit usize");
        Self::grow(&mut self.norm, dim_words, executor);
        Self::grow(&mut self.q, dim_words, executor);
        Self::grow(&mut self.ktmp, kv_words, executor);
        Self::grow(&mut self.vtmp, kv_words, executor);
        Self::grow(&mut self.attn, dim_words, executor);
        Self::grow(&mut self.o, dim_words, executor);
        Self::grow(&mut self.hb, hidden_words, executor);
        Self::grow(&mut self.hb2, hidden_words, executor);
        Self::grow(&mut self.ffn, dim_words, executor);
        Self::grow(&mut self.xstack, dim_words, executor);
        Self::grow(&mut self.logits_stacked, logit_words, executor);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn capacity_bytes(&self) -> usize {
        [
            &self.norm,
            &self.q,
            &self.ktmp,
            &self.vtmp,
            &self.attn,
            &self.o,
            &self.hb,
            &self.hb2,
            &self.ffn,
            &self.xstack,
            &self.logits_stacked,
        ]
        .into_iter()
        .fold(0usize, |bytes, values| {
            bytes.saturating_add(values.capacity().saturating_mul(std::mem::size_of::<f32>()))
        })
    }
}

pub struct Llama {
    pub cfg: Config,
    w: Vec<f32>,
    // RoPE angles depend only on the position and head dimension.  Keeping
    // them here avoids recomputing pow/cos/sin once per layer and token.
    rope_cos: Vec<f32>,
    rope_sin: Vec<f32>,
    // float offsets into w
    emb: usize,
    rms_att: usize,
    wq: usize,
    wk: usize,
    wv: usize,
    wo: usize,
    rms_ffn: usize,
    w1: usize,
    w2: usize,
    w3: usize,
    rms_final: usize,
    wcls: usize,
    /// Use the portable pure-Rust math path required by D2 canonical mode.
    canonical_math: bool,
    /// Persistent bounded owner of exact output-row work.
    exact_executor: ExactExecutor,
    /// One physical teacher forward owns the shared exact pool at a time. The
    /// scientific multi-stream dimension is the explicit batch, never nested
    /// outer forward concurrency.
    forward_gate: std::sync::Mutex<()>,
    /// Retained shape-bounded scratch shared by successive batched forwards.
    batch_workspace: std::sync::Mutex<Box<BatchForwardWorkspace>>,
}

#[derive(Clone)]
pub struct State {
    pub x: Vec<f32>,
    xb: Vec<f32>,
    xb2: Vec<f32>,
    hb: Vec<f32>,
    hb2: Vec<f32>,
    q: Vec<f32>,
    att: Vec<f32>,
    key_cache: Vec<f32>,
    value_cache: Vec<f32>,
    pub logits: Vec<f32>,
    sequence_capacity: usize,
}

/// Invalid bounded sequence-state capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TeacherStateCapacityError {
    Zero,
    ExceedsModel { requested: usize, maximum: usize },
    BoundedAllocationUnavailable { requested: usize, model: usize },
    ArithmeticOverflow,
}

impl std::fmt::Display for TeacherStateCapacityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid teacher state capacity: {self:?}")
    }
}

impl std::error::Error for TeacherStateCapacityError {}

impl State {
    pub fn new(c: &Config) -> Self {
        Self::allocate(
            c,
            c.seq_len,
            c.n_layers * c.seq_len * (c.dim * c.n_kv_heads / c.n_heads),
        )
    }

    /// Allocate sequence state only for the actual prompt/generation horizon.
    pub fn new_bounded(
        c: &Config,
        sequence_capacity: usize,
    ) -> Result<Self, TeacherStateCapacityError> {
        use TeacherStateCapacityError as Error;
        if sequence_capacity == 0 {
            return Err(Error::Zero);
        }
        if sequence_capacity > c.seq_len {
            return Err(Error::ExceedsModel {
                requested: sequence_capacity,
                maximum: c.seq_len,
            });
        }
        let kv_dim = c
            .dim
            .checked_mul(c.n_kv_heads)
            .and_then(|value| value.checked_div(c.n_heads))
            .ok_or(Error::ArithmeticOverflow)?;
        let cache_words = c
            .n_layers
            .checked_mul(sequence_capacity)
            .and_then(|value| value.checked_mul(kv_dim))
            .ok_or(Error::ArithmeticOverflow)?;
        Ok(Self::allocate(c, sequence_capacity, cache_words))
    }

    fn allocate(c: &Config, sequence_capacity: usize, cache_words: usize) -> Self {
        State {
            x: vec![0.0; c.dim],
            xb: vec![0.0; c.dim],
            xb2: vec![0.0; c.dim],
            hb: vec![0.0; c.hidden],
            hb2: vec![0.0; c.hidden],
            q: vec![0.0; c.dim],
            att: vec![0.0; c.n_heads * sequence_capacity],
            key_cache: vec![0.0; cache_words],
            value_cache: vec![0.0; cache_words],
            logits: vec![0.0; c.vocab],
            sequence_capacity,
        }
    }

    /// Maximum position count owned by this private sequence state.
    pub fn sequence_capacity(&self) -> usize {
        self.sequence_capacity
    }

    /// Deterministic content identity of the causal state retained across
    /// teacher forwards.
    ///
    /// Overwrite-only projection scratch is deliberately excluded. The
    /// identity binds the bounded sequence capacity plus the exact bits of the
    /// residual stream, both KV caches, and exposed logits. It is suitable for
    /// proving that cloned transcript templates start equal and then evolve in
    /// private storage.
    pub fn persistent_state_cid(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"uor-r4.teacher-persistent-state/1");
        hasher.update(
            &u64::try_from(self.sequence_capacity)
                .unwrap_or(u64::MAX)
                .to_le_bytes(),
        );
        for (label, values) in [
            ("x", &self.x),
            ("key_cache", &self.key_cache),
            ("value_cache", &self.value_cache),
            ("logits", &self.logits),
        ] {
            hasher.update(&u64::try_from(label.len()).unwrap_or(u64::MAX).to_le_bytes());
            hasher.update(label.as_bytes());
            hasher.update(
                &u64::try_from(values.len())
                    .unwrap_or(u64::MAX)
                    .to_le_bytes(),
            );
            for value in values {
                hasher.update(&value.to_bits().to_le_bytes());
            }
        }
        format!("blake3:{}", hasher.finalize().to_hex())
    }

    /// Begin a new sequence by zeroing state buffers and the KV cache.
    pub fn reset(&mut self) {
        self.x.fill(0.0);
        self.xb.fill(0.0);
        self.xb2.fill(0.0);
        self.hb.fill(0.0);
        self.hb2.fill(0.0);
        self.q.fill(0.0);
        self.att.fill(0.0);
        self.key_cache.fill(0.0);
        self.value_cache.fill(0.0);
        self.logits.fill(0.0);
    }
}

/// One private full-decoder state plus an injected coordinate transport for
/// ordinary dense causal attention.
///
/// The transport is intentionally type-erased at this boundary so
/// `uor-r4-core` can implement the object-safe trait without making this
/// source-model crate depend on the core crate. Construct sessions through
/// [`HuggingFaceLlamaOracle::new_causal_attention_transport_session`].
pub struct CausalAttentionTransportSession {
    state: State,
    transport: Box<dyn attention::CausalAttentionTransport>,
    selected_layers: Vec<bool>,
    scratch: CausalAttentionTransportScratch,
    audit: attention::CausalAttentionTransportAudit,
    pre_rope_projection_audit: attention::CausalAttentionProjectionAudit,
    source_cid: String,
    next_position: usize,
}

impl CausalAttentionTransportSession {
    /// Stable identity of the injected transport implementation and policy.
    pub fn policy_identity(&self) -> &str {
        self.transport.policy_identity()
    }

    /// Current implementation health. A failed status is also surfaced by
    /// the public step API before any logits are copied to the caller.
    pub fn transport_status(&self) -> Result<(), String> {
        self.transport.status()
    }

    /// Deterministic implementation-owned evidence, when the injected
    /// transport supplies a geometry-specific audit snapshot.
    pub fn transport_implementation_evidence(&self) -> Result<Option<String>, String> {
        self.transport.implementation_evidence()
    }

    /// Decoder-owned proof that the hook was exercised over the full causal
    /// prefix and never requested a future position.
    pub fn audit(&self) -> attention::CausalAttentionTransportAudit {
        self.audit
    }

    /// Decoder-owned proof that the optional learned Q/K/V projection hook was
    /// invoked once per selected layer and received complete projected vectors.
    pub fn pre_rope_projection_audit(&self) -> attention::CausalAttentionProjectionAudit {
        self.pre_rope_projection_audit
    }

    /// Clear counters without changing model, KV-cache, or transport state.
    pub fn clear_audit(&mut self) {
        self.audit = attention::CausalAttentionTransportAudit::default();
        self.pre_rope_projection_audit = attention::CausalAttentionProjectionAudit::default();
    }

    /// Deterministic identity of the retained residual, KV-cache, and logits.
    pub fn persistent_state_cid(&self) -> String {
        self.state.persistent_state_cid()
    }

    /// Maximum position count owned by this private sequence state.
    pub fn sequence_capacity(&self) -> usize {
        self.state.sequence_capacity()
    }

    /// Number of decoder layers whose attention uses the injected transport.
    pub fn selected_layer_count(&self) -> usize {
        self.selected_layers
            .iter()
            .filter(|selected| **selected)
            .count()
    }

    /// Whether one zero-based decoder layer uses the injected transport.
    pub fn layer_is_selected(&self, layer: usize) -> bool {
        self.selected_layers.get(layer).copied().unwrap_or(false)
    }

    /// Reset the source state, transport state, causal cursor, and audit for a
    /// fresh independent sequence.
    pub fn reset(&mut self) {
        self.state.reset();
        self.transport.reset();
        self.scratch.clear();
        self.audit = attention::CausalAttentionTransportAudit::default();
        self.pre_rope_projection_audit = attention::CausalAttentionProjectionAudit::default();
        self.next_position = 0;
    }
}

struct CausalAttentionTransportScratch {
    query: Vec<f32>,
    keys: Vec<f32>,
    values: Vec<f32>,
    aggregate: Vec<f32>,
}

impl CausalAttentionTransportScratch {
    fn new(
        head_size: usize,
        sequence_capacity: usize,
    ) -> Result<Self, attention::CausalAttentionTransportError> {
        let cache_words = head_size
            .checked_mul(sequence_capacity)
            .ok_or(attention::CausalAttentionTransportError::ArithmeticOverflow)?;
        Ok(Self {
            query: vec![0.0; head_size],
            keys: vec![0.0; cache_words],
            values: vec![0.0; cache_words],
            aggregate: vec![0.0; head_size],
        })
    }

    fn clear(&mut self) {
        self.query.fill(0.0);
        self.keys.fill(0.0);
        self.values.fill(0.0);
        self.aggregate.fill(0.0);
    }
}

struct CausalAttentionLayerOverride<'a> {
    transport: &'a mut dyn attention::CausalAttentionTransport,
    scratch: &'a mut CausalAttentionTransportScratch,
    audit: &'a mut attention::CausalAttentionTransportAudit,
    pre_rope_projection_audit: &'a mut attention::CausalAttentionProjectionAudit,
}

fn causal_attention_layer_mask(
    selection: attention::CausalAttentionLayerSelection,
    layer_count: usize,
) -> Result<Vec<bool>, attention::CausalAttentionTransportError> {
    use attention::{CausalAttentionLayerSelection as Selection, CausalAttentionTransportError};

    if layer_count == 0 {
        return Err(CausalAttentionTransportError::EmptyLayerSelection);
    }
    match selection {
        Selection::All => Ok(vec![true; layer_count]),
        Selection::Selected(layers) => {
            if layers.is_empty() {
                return Err(CausalAttentionTransportError::EmptyLayerSelection);
            }
            let mut selected = vec![false; layer_count];
            for layer in layers {
                if layer >= layer_count {
                    return Err(CausalAttentionTransportError::LayerOutOfRange {
                        requested: layer,
                        layers: layer_count,
                    });
                }
                if selected[layer] {
                    return Err(CausalAttentionTransportError::DuplicateLayer(layer));
                }
                selected[layer] = true;
            }
            Ok(selected)
        }
    }
}

fn require_healthy_causal_attention_transport(
    transport: &dyn attention::CausalAttentionTransport,
) -> Result<(), attention::CausalAttentionTransportError> {
    let policy_identity = transport.policy_identity().to_owned();
    transport.status().map_err(
        |reason| attention::CausalAttentionTransportError::TransportFault {
            policy_identity,
            reason,
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn apply_full_prefix_causal_attention_transport(
    causal: &mut CausalAttentionLayerOverride<'_>,
    layer: usize,
    query_position: usize,
    query_heads: usize,
    head_size: usize,
    kv_mul: usize,
    sequence_capacity: usize,
    kv_stride: usize,
    queries: &[f32],
    keys: &[f32],
    values: &[f32],
    attention_weights: &mut [f32],
    output: &mut [f32],
    canonical_math: bool,
) {
    causal.audit.layers = causal.audit.layers.saturating_add(1);
    let prefix_len = query_position + 1;
    let prefix_words = prefix_len * head_size;

    for head in 0..query_heads {
        causal.audit.heads = causal.audit.heads.saturating_add(1);
        let head_context = attention::CausalAttentionHeadContext {
            layer,
            head,
            query_position,
        };
        let query = &queries[head * head_size..(head + 1) * head_size];
        causal
            .transport
            .transform_query(head_context, query, &mut causal.scratch.query);
        causal.audit.query_transforms = causal.audit.query_transforms.saturating_add(1);

        let kv_head_offset = (head / kv_mul) * head_size;
        for source_position in 0..prefix_len {
            let source_context = attention::CausalAttentionSourceContext {
                layer,
                head,
                query_position,
                source_position,
            };
            let source_start = source_position * kv_stride + kv_head_offset;
            let source_end = source_start + head_size;
            let transformed_start = source_position * head_size;
            let transformed_end = transformed_start + head_size;
            causal.transport.transport_key(
                source_context,
                &keys[source_start..source_end],
                &mut causal.scratch.keys[transformed_start..transformed_end],
            );
            causal.transport.transport_value(
                source_context,
                &values[source_start..source_end],
                &mut causal.scratch.values[transformed_start..transformed_end],
            );
            causal.audit.key_transports = causal.audit.key_transports.saturating_add(1);
            causal.audit.value_transports = causal.audit.value_transports.saturating_add(1);
            if source_position > query_position {
                causal.audit.future_reads = causal.audit.future_reads.saturating_add(1);
            }
            causal.audit.maximum_source_position = Some(
                causal
                    .audit
                    .maximum_source_position
                    .map_or(source_position, |maximum| maximum.max(source_position)),
            );
        }

        let attention =
            &mut attention_weights[head * sequence_capacity..head * sequence_capacity + prefix_len];
        causal.transport.score_and_normalize(
            head_context,
            &causal.scratch.query,
            &causal.scratch.keys[..prefix_words],
            attention,
            canonical_math,
        );
        causal.transport.weighted_value_centroid(
            head_context,
            attention,
            &causal.scratch.values[..prefix_words],
            &mut causal.scratch.aggregate,
        );
        causal.transport.output_to_model_frame(
            head_context,
            &causal.scratch.aggregate,
            &mut output[head * head_size..(head + 1) * head_size],
        );
        causal.audit.output_transforms = causal.audit.output_transforms.saturating_add(1);
    }
}

#[inline]
pub(crate) fn sqrtf(value: f32, canonical: bool) -> f32 {
    if canonical {
        libm::sqrtf(value)
    } else {
        value.sqrt()
    }
}

#[inline]
fn expf(value: f32, canonical: bool) -> f32 {
    if canonical {
        libm::expf(value)
    } else {
        value.exp()
    }
}

#[inline]
fn powf(base: f32, exponent: f32, canonical: bool) -> f32 {
    if canonical {
        libm::powf(base, exponent)
    } else {
        base.powf(exponent)
    }
}

#[inline]
fn sinf(value: f32, canonical: bool) -> f32 {
    if canonical {
        libm::sinf(value)
    } else {
        value.sin()
    }
}

#[inline]
fn cosf(value: f32, canonical: bool) -> f32 {
    if canonical {
        libm::cosf(value)
    } else {
        value.cos()
    }
}

fn rmsnorm_with_mode(o: &mut [f32], x: &[f32], weight: &[f32], canonical: bool) {
    let size = x.len();
    let mut ss = x.iter().map(|value| value * value).sum::<f32>();
    ss /= size as f32;
    ss += 1e-5f32;
    ss = 1.0f32 / sqrtf(ss, canonical);
    for ((output, value), weight) in o.iter_mut().zip(x).zip(weight) {
        *output = *weight * (ss * *value);
    }
}

/// In-place variant matching C's rmsnorm(x, x, w): C computes ss from x
/// first, then writes; identical here.
fn rmsnorm_inplace_with_mode(x: &mut [f32], weight: &[f32], canonical: bool) {
    let size = x.len();
    let mut ss = x.iter().map(|value| value * value).sum::<f32>();
    ss /= size as f32;
    ss += 1e-5f32;
    ss = 1.0f32 / sqrtf(ss, canonical);
    for (value, weight) in x.iter_mut().zip(weight) {
        *value = *weight * (ss * *value);
    }
}

pub(crate) fn softmax_with_mode(x: &mut [f32], canonical: bool) {
    let mut max_val = x[0];
    for &value in x.iter().skip(1) {
        if value > max_val {
            max_val = value;
        }
    }
    let mut sum = 0.0f32;
    for value in x.iter_mut() {
        *value = expf(*value - max_val, canonical);
        sum += *value;
    }
    for value in x.iter_mut() {
        *value /= sum;
    }
}

/// W (d,n) @ x (n,) -> xout (d,), computed by the pinned `uor-matmul` exact
/// GEMM (#655-B2). `gemm_float` accumulates every product into a complete
/// accumulator and rounds once. The enforced output-bit contract is exercised
/// across fixed worker counts by `exact_matmul_matches_serial_bits_for_1_2_4_8_workers`
/// and by the fixture-present exact probe. The former
/// `fast` (Accelerate BLAS `sgemv`) / hand-rolled canonical paths are gone;
/// `_fast` is retained in the signature only so callers need not change.
fn matmul(
    _executor: &ExactExecutor,
    xout: &mut [f32],
    x: &[f32],
    w: &[f32],
    n: usize,
    _fast: bool,
) {
    // #804 measurement-only exception (maintainer-approved 2026-08-18):
    // under the opt-in feature, observation builds route through Apple
    // Accelerate — see `observation_blas_exception`'s module docs. Every
    // default build takes the owned exact GEMM below.
    #[cfg(all(feature = "observation-blas-exception", target_os = "macos"))]
    {
        observation_blas_exception::matmul(xout, x, w, n);
    }
    #[cfg(not(all(feature = "observation-blas-exception", target_os = "macos")))]
    {
        _executor.matmul(xout, x, w, n);
    }
}

/// Batched matmul: `batch` input vectors of length `n` through weight
/// `W[rows, n]` → `batch` output vectors of length `rows`, laid out
/// sequence-major (`x` is `batch * n`, `xout` is `batch * rows`). Computes
/// `C[batch, rows] = X[batch, n] · W[rows, n]ᵀ` with the pinned `uor-matmul`
/// exact GEMM (#655-B2), replacing the former Accelerate BLAS `sgemm` /
/// hand-rolled `dot_fast` reuse. Exact serial/batched output-bit identity is an
/// enforced contract covered by `exact_batched_matmul_matches_serial_bits_for_1_2_4_8_workers`
/// and the fixture-present probe; unavailable live evidence is not a pass.
fn matmul_batched(
    _executor: &ExactExecutor,
    xout: &mut [f32],
    x: &[f32],
    w: &[f32],
    n: usize,
    batch: usize,
) {
    debug_assert!(batch > 0);
    debug_assert_eq!(xout.len() % batch, 0);
    let rows = xout.len() / batch;
    debug_assert!(w.len() >= rows * n);
    debug_assert_eq!(x.len(), batch * n);
    // #804 measurement-only exception — see `matmul` above and the
    // `observation_blas_exception` module docs.
    #[cfg(all(feature = "observation-blas-exception", target_os = "macos"))]
    {
        observation_blas_exception::matmul_batched(xout, x, w, n, batch);
    }
    #[cfg(not(all(feature = "observation-blas-exception", target_os = "macos")))]
    {
        _executor.matmul_batched(xout, x, w, n, batch);
    }
}

/// The teacher's matrix-operation backend, for the "teacher model ready"
/// diagnostic. Since #655-B2 every Llama/shared weight projection is the
/// pinned `uor-matmul` exact GEMM. Hosted builds may select different exact
/// kernels through runtime CPU-feature detection while retaining the pinned
/// arithmetic/output-bit contract; wasm uses the portable fallback. GPT-2
/// owns its separate declared dense implementation.
fn fast_matmul_backend() -> &'static str {
    #[cfg(all(feature = "observation-blas-exception", target_os = "macos"))]
    {
        // Loud per-run provenance: every "teacher model ready" line names
        // the exception so no corpus can be produced under it silently.
        "Accelerate cblas (observation-only exception #804)"
    }
    #[cfg(not(all(feature = "observation-blas-exception", target_os = "macos")))]
    {
        "uor-matmul exact GEMM"
    }
}

impl Llama {
    fn build_rope_cache(cfg: &Config, canonical_math: bool) -> (Vec<f32>, Vec<f32>) {
        let head_size = cfg.dim / cfg.n_heads;
        let half = head_size / 2;
        let mut cos = Vec::with_capacity(cfg.seq_len * half);
        let mut sin = Vec::with_capacity(cfg.seq_len * half);
        for pos in 0..cfg.seq_len {
            for i in 0..half {
                let freq = 1.0f32
                    / powf(
                        cfg.rope_theta,
                        (2 * i) as f32 / head_size as f32,
                        canonical_math,
                    );
                let angle = pos as f32 * freq;
                cos.push(cosf(angle, canonical_math));
                sin.push(sinf(angle, canonical_math));
            }
        }
        (cos, sin)
    }

    fn rebuild_rope_cache(&mut self) {
        (self.rope_cos, self.rope_sin) = Self::build_rope_cache(&self.cfg, self.canonical_math);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(path: &str) -> Llama {
        let raw = std::fs::read(path).expect("checkpoint");
        let i32at = |o: usize| i32::from_le_bytes(raw[o..o + 4].try_into().unwrap());
        let vocab_raw = i32at(20);
        let cfg = Config {
            dim: i32at(0) as usize,
            hidden: i32at(4) as usize,
            n_layers: i32at(8) as usize,
            n_heads: i32at(12) as usize,
            n_kv_heads: i32at(16) as usize,
            vocab: vocab_raw.unsigned_abs() as usize,
            seq_len: i32at(24) as usize,
            rope_theta: 10_000.0,
            rms_norm_eps: 1e-5,
            rope_interleaved: true,
            r4_attention: false,
        };
        let shared = vocab_raw > 0;
        let nf = (raw.len() - 28) / 4;
        let mut w = vec![0.0f32; nf];
        for (i, value) in w.iter_mut().enumerate() {
            let o = 28 + i * 4;
            *value = f32::from_le_bytes(raw[o..o + 4].try_into().unwrap());
        }
        let (dim, hid, nl, hs) = (cfg.dim, cfg.hidden, cfg.n_layers, cfg.dim / cfg.n_heads);
        let kv_dim = cfg.dim * cfg.n_kv_heads / cfg.n_heads;
        let mut p = 0usize;
        let emb = p;
        p += cfg.vocab * dim;
        let rms_att = p;
        p += nl * dim;
        let wq = p;
        p += nl * dim * dim;
        let wk = p;
        p += nl * dim * kv_dim;
        let wv = p;
        p += nl * dim * kv_dim;
        let wo = p;
        p += nl * dim * dim;
        let rms_ffn = p;
        p += nl * dim;
        let w1 = p;
        p += nl * dim * hid;
        let w2 = p;
        p += nl * hid * dim;
        let w3 = p;
        p += nl * dim * hid;
        let rms_final = p;
        p += dim;
        p += cfg.seq_len * hs / 2; // skip legacy freq_cis_real
        p += cfg.seq_len * hs / 2; // skip legacy freq_cis_imag
        let wcls = if shared { emb } else { p };
        let mut model = Llama {
            cfg,
            w,
            rope_cos: Vec::new(),
            rope_sin: Vec::new(),
            emb,
            rms_att,
            wq,
            wk,
            wv,
            wo,
            rms_ffn,
            w1,
            w2,
            w3,
            rms_final,
            wcls,
            canonical_math: false,
            exact_executor: ExactExecutor::new(TeacherExecutionConfig::default())
                .expect("the one-worker exact executor must build"),
            forward_gate: std::sync::Mutex::new(()),
            batch_workspace: std::sync::Mutex::new(Box::new(BatchForwardWorkspace::default())),
        };
        model.rebuild_rope_cache();
        model
    }

    fn from_flat(cfg: Config, w: Vec<f32>, shared: bool) -> Self {
        let (dim, hid, nl) = (cfg.dim, cfg.hidden, cfg.n_layers);
        let kv_dim = cfg.dim * cfg.n_kv_heads / cfg.n_heads;
        let mut p = 0usize;
        let emb = p;
        p += cfg.vocab * dim;
        let rms_att = p;
        p += nl * dim;
        let wq = p;
        p += nl * dim * dim;
        let wk = p;
        p += nl * dim * kv_dim;
        let wv = p;
        p += nl * dim * kv_dim;
        let wo = p;
        p += nl * dim * dim;
        let rms_ffn = p;
        p += nl * dim;
        let w1 = p;
        p += nl * dim * hid;
        let w2 = p;
        p += nl * hid * dim;
        let w3 = p;
        p += nl * dim * hid;
        let rms_final = p;
        p += dim;
        let wcls = if shared { emb } else { p };
        assert_eq!(w.len(), if shared { p } else { p + cfg.vocab * dim });
        let mut model = Self {
            cfg,
            w,
            rope_cos: Vec::new(),
            rope_sin: Vec::new(),
            emb,
            rms_att,
            wq,
            wk,
            wv,
            wo,
            rms_ffn,
            w1,
            w2,
            w3,
            rms_final,
            wcls,
            canonical_math: false,
            exact_executor: ExactExecutor::new(TeacherExecutionConfig::default())
                .expect("the one-worker exact executor must build"),
            forward_gate: std::sync::Mutex::new(()),
            batch_workspace: std::sync::Mutex::new(Box::new(BatchForwardWorkspace::default())),
        };
        model.rebuild_rope_cache();
        model
    }

    /// Replace the bounded exact executor while holding exclusive model access.
    ///
    /// Weights and numerical configuration are unchanged. Requiring `&mut`
    /// makes replacement mutually exclusive with every forward and resets the
    /// execution counters for the new run.
    pub(crate) fn set_execution_config(
        &mut self,
        config: TeacherExecutionConfig,
    ) -> Result<(), SourceUnavailable> {
        self.exact_executor = ExactExecutor::new(config)?;
        Ok(())
    }

    pub(crate) fn begin_measured_execution(&mut self, observer: TeacherExecutionObserver) {
        self.exact_executor.begin_measured_execution(observer);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn prestart_exact_execution(
        &self,
        batch_width: usize,
    ) -> Result<TeacherExecutionPreparation, SourceUnavailable> {
        let started = std::time::Instant::now();
        let _forward_guard = self
            .forward_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let batch_capacity_bytes = {
            let mut workspace = self
                .batch_workspace
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            workspace.ensure(&self.cfg, batch_width, &self.exact_executor);
            workspace.capacity_bytes()
        };
        let maximum_k = self.cfg.dim.max(self.cfg.hidden);
        let kv_dim = self.cfg.dim * self.cfg.n_kv_heads / self.cfg.n_heads;
        let maximum_rows = self
            .cfg
            .dim
            .max(self.cfg.hidden)
            .max(kv_dim)
            .max(self.cfg.vocab);
        let mut evidence = self
            .exact_executor
            .prestart(batch_width, maximum_k, maximum_rows)?;
        evidence.elapsed_seconds = started.elapsed().as_secs_f64();
        evidence.workspace_capacity_bytes = evidence
            .workspace_capacity_bytes
            .saturating_add(u64::try_from(batch_capacity_bytes).unwrap_or(u64::MAX));
        Ok(evidence)
    }

    /// Current bounded-execution counters.
    pub fn execution_snapshot(&self) -> TeacherExecutionSnapshot {
        self.exact_executor.snapshot()
    }

    /// Counter oracle for one forward at the current geometry and tiling.
    ///
    /// Output rows are the sole scheduler partition. Batch width therefore
    /// scales output cells and exact scalar terms, but not matrix calls or row
    /// tiles: all lanes share each weight tile in one exact GEMM.
    pub fn exact_forward_plan(
        &self,
        batch_width: usize,
    ) -> Result<ExactForwardPlan, ExactForwardPlanError> {
        exact_forward_plan_for_geometry(&self.cfg, batch_width, |rows| {
            self.exact_executor.row_tiles(rows)
        })
    }

    /// Complete raw-output/state trace dimensions for a bounded probe.
    pub fn exact_probe_trace_shape(
        &self,
        positions: usize,
        batch_width: usize,
        top_k: usize,
    ) -> Result<ExactMulticoreProbeTraceShape, ExactForwardPlanError> {
        exact_probe_trace_shape_for_geometry(
            &self.cfg,
            self.cfg.seq_len,
            positions,
            batch_width,
            top_k,
        )
    }

    /// Complete trace dimensions for explicitly bounded private states.
    pub fn exact_probe_trace_shape_bounded(
        &self,
        sequence_capacity: usize,
        positions: usize,
        batch_width: usize,
        top_k: usize,
    ) -> Result<ExactMulticoreProbeTraceShape, ExactForwardPlanError> {
        exact_probe_trace_shape_for_geometry(
            &self.cfg,
            sequence_capacity,
            positions,
            batch_width,
            top_k,
        )
    }

    /// One forward step. After return, st.x holds the post-final-rmsnorm
    /// hidden state (the kNN-LM context vector) and st.logits the logits.
    pub fn forward(&self, st: &mut State, token: usize, pos: usize, fast_matmul: bool) {
        let _forward_guard = self
            .forward_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            pos < st.sequence_capacity,
            "teacher state capacity exceeded"
        );
        self.exact_executor.begin_forward(1);
        let dim = self.cfg.dim;
        st.x.copy_from_slice(&self.w[self.emb + token * dim..self.emb + (token + 1) * dim]);
        for l in 0..self.cfg.n_layers {
            self.layer_forward(st, l, pos, fast_matmul);
        }
        self.finish_forward(st, fast_matmul);
        self.exact_executor.complete_forward(1);
    }

    /// Advance one independent experimental geometric-decoder session.
    /// Exactly one declared layer bypasses source Q/K/V, source attention,
    /// and source output projection in favor of the bounded R4 mixer.  All
    /// other layers and the final LM head remain the ordinary source path.
    fn forward_geometric(
        &self,
        st: &mut State,
        token: usize,
        pos: usize,
        fast_matmul: bool,
        runtime: &mut geometric_decoder::GeometricRuntime,
    ) {
        let _forward_guard = self
            .forward_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        debug_assert!(pos < st.sequence_capacity);
        debug_assert!(token < self.cfg.vocab);
        self.exact_executor.begin_forward(1);
        let dim = self.cfg.dim;
        st.x.copy_from_slice(&self.w[self.emb + token * dim..self.emb + (token + 1) * dim]);
        for layer in 0..self.cfg.n_layers {
            if layer == runtime.target_layer()
                && runtime.intervention() != geometric_decoder::GeometryIntervention::Disabled
            {
                self.layer_forward_with_geometry(
                    st,
                    layer,
                    pos,
                    fast_matmul,
                    Some(runtime),
                    None,
                    None,
                );
            } else {
                self.layer_forward(st, layer, pos, fast_matmul);
            }
        }
        self.finish_forward(st, fast_matmul);
        self.exact_executor.complete_forward(1);
    }

    /// Advance one independent session whose selected layers retain the
    /// checkpoint's Q/K/V, RoPE, output projection, and surrounding decoder
    /// while an injected query-local operator transports coordinates and may
    /// replace the row score/normalization/value centroid.
    fn forward_causal_attention_transport(
        &self,
        session: &mut CausalAttentionTransportSession,
        token: usize,
        pos: usize,
        fast_matmul: bool,
    ) {
        let _forward_guard = self
            .forward_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        debug_assert!(pos < session.state.sequence_capacity);
        debug_assert!(token < self.cfg.vocab);

        let CausalAttentionTransportSession {
            state,
            transport,
            selected_layers,
            scratch,
            audit,
            pre_rope_projection_audit,
            ..
        } = session;
        transport.begin_position(token, pos);
        audit.positions = audit.positions.saturating_add(1);
        audit.maximum_query_position = Some(
            audit
                .maximum_query_position
                .map_or(pos, |maximum| maximum.max(pos)),
        );

        self.exact_executor.begin_forward(1);
        let dim = self.cfg.dim;
        state
            .x
            .copy_from_slice(&self.w[self.emb + token * dim..self.emb + (token + 1) * dim]);
        for (layer, selected) in selected_layers.iter().copied().enumerate() {
            if selected {
                let mut causal_attention = CausalAttentionLayerOverride {
                    transport: transport.as_mut(),
                    scratch,
                    audit,
                    pre_rope_projection_audit,
                };
                self.layer_forward_with_geometry(
                    state,
                    layer,
                    pos,
                    fast_matmul,
                    None,
                    None,
                    Some(&mut causal_attention),
                );
            } else {
                self.layer_forward(state, layer, pos, fast_matmul);
            }
        }
        self.finish_forward(state, fast_matmul);
        self.exact_executor.complete_forward(1);
    }

    /// One ordinary source step with the focused G1 layer seam copied through
    /// the existing residual/Q/K/V/attention/logit executor. The callback is a
    /// read-only trace tap: source weights and recurrent state follow the same
    /// branch as [`Llama::forward`].
    fn forward_capturing_geometric_source(
        &self,
        st: &mut State,
        token: usize,
        pos: usize,
        fast_matmul: bool,
        target_layer: usize,
        capture: &mut geometric_training::SourceLayerCapture,
    ) {
        let _forward_guard = self
            .forward_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        debug_assert!(pos < st.sequence_capacity);
        debug_assert!(token < self.cfg.vocab);
        debug_assert!(target_layer < self.cfg.n_layers);
        self.exact_executor.begin_forward(1);
        let dim = self.cfg.dim;
        st.x.copy_from_slice(&self.w[self.emb + token * dim..self.emb + (token + 1) * dim]);
        for layer in 0..self.cfg.n_layers {
            if layer == target_layer {
                self.layer_forward_with_geometry(
                    st,
                    layer,
                    pos,
                    fast_matmul,
                    None,
                    Some(capture),
                    None,
                );
            } else {
                self.layer_forward(st, layer, pos, fast_matmul);
            }
        }
        self.finish_forward(st, fast_matmul);
        capture.logits.clear();
        capture.logits.extend_from_slice(&st.logits);
        self.exact_executor.complete_forward(1);
    }

    /// One forward step with the residual stream captured after each layer in
    /// `capture_layers` (#599 conformance trace). This IS the exact executor:
    /// embedding, per-layer body, and final norm/logits are the same
    /// [`Llama::layer_forward`]/[`Llama::finish_forward`] path `forward`
    /// takes, so a captured run and a plain run produce identical bits. The
    /// capture is bounded by construction: only the declared layer indices
    /// are copied out, once per step.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn forward_capturing(
        &self,
        st: &mut State,
        token: usize,
        pos: usize,
        fast_matmul: bool,
        capture_layers: &[usize],
        sink: &mut dyn FnMut(usize, &[f32]),
    ) {
        let _forward_guard = self
            .forward_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            pos < st.sequence_capacity,
            "teacher state capacity exceeded"
        );
        self.exact_executor.begin_forward(1);
        let dim = self.cfg.dim;
        st.x.copy_from_slice(&self.w[self.emb + token * dim..self.emb + (token + 1) * dim]);
        for l in 0..self.cfg.n_layers {
            self.layer_forward(st, l, pos, fast_matmul);
            if capture_layers.contains(&l) {
                sink(l, &st.x);
            }
        }
        self.finish_forward(st, fast_matmul);
        self.exact_executor.complete_forward(1);
    }

    /// One forward step with the #603 teacher-trace lanes captured at
    /// declared layer indices. This IS the exact executor — the same
    /// [`Llama::layer_forward`]/[`Llama::finish_forward`] path `forward`
    /// and `forward_capturing` take, so a traced step and a plain step
    /// produce identical bits; the taps only READ state the layer body
    /// already produced. Bounded by construction: each lane copies out
    /// only its declared layer indices, once per step.
    ///
    /// Taps, all read after `layer_forward(l)` returns:
    ///
    /// - residual: `st.x` (the post-layer residual stream, the same slice
    ///   `forward_capturing` sinks);
    /// - q/k/v: `st.q` (the current position's RoPE-rotated query) and the
    ///   layer's key/value cache rows at `pos` — the exact vectors the
    ///   #602 attention operators consumed;
    /// - attention support: per head `h`, the softmax-normalized weights
    ///   `st.att[h*seq_len .. h*seq_len+pos+1]` the #602 factored per-head
    ///   weight functions just produced for layer `l`. Read before the
    ///   next layer overwrites the shared buffer.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn forward_capturing_trace(
        &self,
        st: &mut State,
        token: usize,
        pos: usize,
        fast_matmul: bool,
        request: &TraceCaptureRequest<'_>,
        sinks: &mut TraceCaptureSinks<'_, '_>,
    ) {
        let _forward_guard = self
            .forward_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            pos < st.sequence_capacity,
            "teacher state capacity exceeded"
        );
        self.exact_executor.begin_forward(1);
        let dim = self.cfg.dim;
        let kv_dim = self.cfg.dim * self.cfg.n_kv_heads / self.cfg.n_heads;
        st.x.copy_from_slice(&self.w[self.emb + token * dim..self.emb + (token + 1) * dim]);
        for l in 0..self.cfg.n_layers {
            self.layer_forward(st, l, pos, fast_matmul);
            if request.attention_layers.contains(&l) {
                for h in 0..self.cfg.n_heads {
                    (sinks.attention)(
                        l,
                        h,
                        &st.att[h * st.sequence_capacity..h * st.sequence_capacity + pos + 1],
                    );
                }
            }
            if request.qkv_layers.contains(&l) {
                let loff = l * st.sequence_capacity * kv_dim;
                let k = &st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                let v = &st.value_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                (sinks.qkv)(l, &st.q, k, v);
            }
            if request.residual_layers.contains(&l) {
                (sinks.residual)(l, &st.x);
            }
        }
        self.finish_forward(st, fast_matmul);
        self.exact_executor.complete_forward(1);
    }

    /// One transformer layer of the exact forward step, factored out of
    /// [`Llama::forward`] so the #599 conformance runner can observe the
    /// residual stream at declared layer indices through the very same
    /// executor path. Operation order and arithmetic are unchanged from the
    /// original in-line loop body.
    fn layer_forward(&self, st: &mut State, l: usize, pos: usize, fast_matmul: bool) {
        self.layer_forward_with_geometry(st, l, pos, fast_matmul, None, None, None);
    }

    /// Shared transformer-layer body.  `geometry = Some` replaces only this
    /// layer's source-attention seam; the residual/MLP stack stays identical.
    #[allow(clippy::too_many_arguments)]
    fn layer_forward_with_geometry(
        &self,
        st: &mut State,
        l: usize,
        pos: usize,
        fast_matmul: bool,
        geometry: Option<&mut geometric_decoder::GeometricRuntime>,
        mut source_capture: Option<&mut geometric_training::SourceLayerCapture>,
        mut causal_attention: Option<&mut CausalAttentionLayerOverride<'_>>,
    ) {
        let c = &self.cfg;
        let (dim, hid) = (c.dim, c.hidden);
        let kv_dim = c.dim * c.n_kv_heads / c.n_heads;
        let kv_mul = c.n_heads / c.n_kv_heads;
        let head_size = dim / c.n_heads;
        let w = &self.w;
        {
            if let Some(capture) = source_capture.as_deref_mut() {
                capture.base_residual.clear();
                capture.base_residual.extend_from_slice(&st.x);
            }
            rmsnorm_with_mode(
                &mut st.xb,
                &st.x,
                &w[self.rms_att + l * dim..self.rms_att + (l + 1) * dim],
                self.canonical_math,
            );
            if let Some(capture) = source_capture.as_deref_mut() {
                capture.normalized_residual.clear();
                capture.normalized_residual.extend_from_slice(&st.xb);
            }

            if let Some(runtime) = geometry {
                runtime.mix(
                    &self.exact_executor,
                    &st.xb,
                    pos,
                    &mut st.xb2,
                    self.canonical_math,
                );
            } else {
                let loff = l * st.sequence_capacity * kv_dim;
                matmul(
                    &self.exact_executor,
                    &mut st.q,
                    &st.xb,
                    &w[self.wq + l * dim * dim..],
                    dim,
                    fast_matmul,
                );
                {
                    let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                    matmul(
                        &self.exact_executor,
                        k,
                        &st.xb,
                        &w[self.wk + l * dim * kv_dim..],
                        dim,
                        fast_matmul,
                    );
                }
                {
                    let v = &mut st.value_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                    matmul(
                        &self.exact_executor,
                        v,
                        &st.xb,
                        &w[self.wv + l * dim * kv_dim..],
                        dim,
                        fast_matmul,
                    );
                }

                if let Some(causal_attention) = causal_attention.as_deref_mut() {
                    let context = attention::CausalAttentionProjectionContext {
                        layer: l,
                        query_position: pos,
                        query_heads: c.n_heads,
                        key_value_heads: c.n_kv_heads,
                        head_size,
                    };
                    let key = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                    let value = &mut st.value_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                    causal_attention
                        .transport
                        .transform_projected_qkv_before_rope(context, &mut st.q, key, value);
                    causal_attention
                        .pre_rope_projection_audit
                        .record(context, dim, kv_dim, kv_dim);
                }

                // RoPE: converted llama2.c checkpoints interleave pairs; native
                // Hugging Face Safetensors rotate the two head halves.
                if c.rope_interleaved {
                    let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                    let rope_offset = pos * (head_size / 2);
                    let mut i = 0usize;
                    while i < dim {
                        let angle_index = rope_offset + (i % head_size) / 2;
                        let fcr = self.rope_cos[angle_index];
                        let fci = self.rope_sin[angle_index];
                        let rotn = if i < kv_dim { 2 } else { 1 };
                        for v in 0..rotn {
                            let vec: &mut [f32] = if v == 0 { &mut st.q } else { &mut *k };
                            let v0 = vec[i];
                            let v1 = vec[i + 1];
                            vec[i] = v0 * fcr - v1 * fci;
                            vec[i + 1] = v0 * fci + v1 * fcr;
                        }
                        i += 2;
                    }
                } else {
                    let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                    for vector in [&mut st.q[..], &mut k[..]] {
                        for head in vector.chunks_exact_mut(head_size) {
                            let half = head_size / 2;
                            for i in 0..half {
                                let angle_index = pos * half + i;
                                let cos = self.rope_cos[angle_index];
                                let sin = self.rope_sin[angle_index];
                                let first = head[i];
                                let second = head[i + half];
                                head[i] = first * cos - second * sin;
                                head[i + half] = second * cos + first * sin;
                            }
                        }
                    }
                }

                // Source multihead attention. The injected path is presented
                // the same complete causal prefix. Its default row hooks retain
                // standard stable softmax and the linear value aggregate;
                // implementation-owned evidence binds any replacement score
                // or geometric centroid.
                if let Some(causal_attention) = causal_attention {
                    apply_full_prefix_causal_attention_transport(
                        causal_attention,
                        l,
                        pos,
                        c.n_heads,
                        head_size,
                        kv_mul,
                        st.sequence_capacity,
                        kv_dim,
                        &st.q,
                        &st.key_cache[loff..],
                        &st.value_cache[loff..],
                        &mut st.att,
                        &mut st.xb,
                        self.canonical_math,
                    );
                } else {
                    for h in 0..c.n_heads {
                        let q = &st.q[h * head_size..(h + 1) * head_size];
                        let att = &mut st.att
                            [h * st.sequence_capacity..h * st.sequence_capacity + pos + 1];
                        let kv_head_offset = (h / kv_mul) * head_size;

                        if c.r4_attention {
                            attention::experimental_r4_head_attention_weights(
                                att,
                                q,
                                &st.key_cache[loff..],
                                kv_head_offset,
                                kv_dim,
                                self.canonical_math,
                            );
                        } else {
                            attention::standard_head_attention_weights(
                                att,
                                q,
                                &st.key_cache[loff..],
                                kv_head_offset,
                                kv_dim,
                                self.canonical_math,
                            );
                        }

                        let att =
                            &st.att[h * st.sequence_capacity..h * st.sequence_capacity + pos + 1];
                        let xb = &mut st.xb[h * head_size..(h + 1) * head_size];
                        attention::head_attention_value_aggregate(
                            xb,
                            att,
                            &st.value_cache[loff..],
                            kv_head_offset,
                            kv_dim,
                        );
                    }
                }

                matmul(
                    &self.exact_executor,
                    &mut st.xb2,
                    &st.xb,
                    &w[self.wo + l * dim * dim..],
                    dim,
                    fast_matmul,
                );
                if let Some(capture) = source_capture {
                    capture.attention_output.clear();
                    capture.attention_output.extend_from_slice(&st.xb2);
                    capture.q.clear();
                    capture.q.extend_from_slice(&st.q);
                    let loff = l * st.sequence_capacity * kv_dim;
                    capture.k.clear();
                    capture.k.extend_from_slice(
                        &st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim],
                    );
                    capture.v.clear();
                    capture.v.extend_from_slice(
                        &st.value_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim],
                    );
                    capture.mean_attention_support.clear();
                    capture.mean_attention_support.resize(pos + 1, 0.0);
                    for head in 0..c.n_heads {
                        let attention = &st.att
                            [head * st.sequence_capacity..head * st.sequence_capacity + pos + 1];
                        for (mean, weight) in
                            capture.mean_attention_support.iter_mut().zip(attention)
                        {
                            *mean += *weight / c.n_heads as f32;
                        }
                    }
                }
            }
            for i in 0..dim {
                st.x[i] += st.xb2[i];
            }

            rmsnorm_with_mode(
                &mut st.xb,
                &st.x,
                &w[self.rms_ffn + l * dim..self.rms_ffn + (l + 1) * dim],
                self.canonical_math,
            );
            matmul(
                &self.exact_executor,
                &mut st.hb,
                &st.xb,
                &w[self.w1 + l * dim * hid..],
                dim,
                fast_matmul,
            );
            matmul(
                &self.exact_executor,
                &mut st.hb2,
                &st.xb,
                &w[self.w3 + l * dim * hid..],
                dim,
                fast_matmul,
            );
            for i in 0..hid {
                let mut val = st.hb[i];
                val *= 1.0f32 / (1.0f32 + expf(-val, self.canonical_math));
                val *= st.hb2[i];
                st.hb[i] = val;
            }
            matmul(
                &self.exact_executor,
                &mut st.xb,
                &st.hb,
                &w[self.w2 + l * hid * dim..],
                hid,
                fast_matmul,
            );
            for i in 0..dim {
                st.x[i] += st.xb[i];
            }
        }
    }

    /// The tail of the exact forward step (final in-place rmsnorm and the
    /// vocabulary matmul), factored out of [`Llama::forward`] unchanged.
    fn finish_forward(&self, st: &mut State, fast_matmul: bool) {
        let dim = self.cfg.dim;
        let w = &self.w;
        let rf = self.rms_final;
        // C: rmsnorm(x, x, w) — in-place with pre-read ss.
        {
            let (wslice, x) = (&w[rf..rf + dim], &mut st.x);
            rmsnorm_inplace_with_mode(x, wslice, self.canonical_math);
        }
        matmul(
            &self.exact_executor,
            &mut st.logits,
            &st.x,
            &w[self.wcls..],
            dim,
            fast_matmul,
        );
    }

    /// Batched forward: advance `states.len()` independent sequences by one
    /// position each — sequence `b` steps `tokens[b]` at `positions[b]` against
    /// its own KV cache in `states[b]`. The memory-bound weight matmuls (Q/K/V,
    /// output, MLP, vocab) run once over the whole batch via [`matmul_batched`]
    /// instead of once per sequence, so B sequences cost one weight sweep — the
    /// amortization that lifts the teacher off the per-token memory-bandwidth
    /// wall. Every per-sequence op (rmsnorm, RoPE, attention, SwiGLU, residual)
    /// mirrors [`Llama::forward`] exactly. `matmul_batched` and the serial path
    /// share the pinned exact `uor-matmul` owner; exact bit agreement is an
    /// enforced contract checked by focused per-target tests and the live
    /// probe, not a universal cross-target claim. `fast_matmul` is a
    /// compatibility parameter and does not select another arithmetic owner.
    pub fn forward_batch(
        &self,
        states: &mut [State],
        tokens: &[usize],
        positions: &[usize],
        fast_matmul: bool,
    ) {
        assert!(
            !states.is_empty(),
            "batched teacher forward requires at least one state"
        );
        assert_eq!(
            tokens.len(),
            states.len(),
            "batched teacher token/state lengths must match"
        );
        assert_eq!(
            positions.len(),
            states.len(),
            "batched teacher position/state lengths must match"
        );
        let _forward_guard = self
            .forward_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _ = fast_matmul;
        let c = &self.cfg;
        let (dim, hid) = (c.dim, c.hidden);
        let kv_dim = c.dim * c.n_kv_heads / c.n_heads;
        let kv_mul = c.n_heads / c.n_kv_heads;
        let head_size = dim / c.n_heads;
        let w = &self.w;
        let b = states.len();
        assert!(
            states
                .iter()
                .zip(positions)
                .all(|(state, &position)| position < state.sequence_capacity),
            "teacher state capacity exceeded"
        );
        self.exact_executor.prepare_workspace(
            dim.max(hid),
            dim.max(hid).max(kv_dim).max(c.vocab),
            b,
        );
        self.exact_executor.begin_forward(b);

        // Shape-bounded sequence-major buffers persist across physical
        // forwards. Preparation grows them once; steady-state forwards reuse
        // the same capacities without changing any arithmetic or lane order.
        let mut workspace = self
            .batch_workspace
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        workspace.ensure(c, b, &self.exact_executor);
        let BatchForwardWorkspace {
            norm,
            q,
            ktmp,
            vtmp,
            attn,
            o,
            hb,
            hb2,
            ffn,
            xstack,
            logits_stacked,
        } = &mut **workspace;
        let dim_words = b * dim;
        let kv_words = b * kv_dim;
        let hidden_words = b * hid;
        let logit_words = b * c.vocab;
        let norm = &mut norm[..dim_words];
        let q = &mut q[..dim_words];
        let ktmp = &mut ktmp[..kv_words];
        let vtmp = &mut vtmp[..kv_words];
        let attn = &mut attn[..dim_words];
        let o = &mut o[..dim_words];
        let hb = &mut hb[..hidden_words];
        let hb2 = &mut hb2[..hidden_words];
        let ffn = &mut ffn[..dim_words];
        let xstack = &mut xstack[..dim_words];
        let logits_stacked = &mut logits_stacked[..logit_words];

        for bi in 0..b {
            let token = tokens[bi];
            states[bi]
                .x
                .copy_from_slice(&w[self.emb + token * dim..self.emb + (token + 1) * dim]);
        }

        for l in 0..c.n_layers {
            for bi in 0..b {
                rmsnorm_with_mode(
                    &mut norm[bi * dim..(bi + 1) * dim],
                    &states[bi].x,
                    &w[self.rms_att + l * dim..self.rms_att + (l + 1) * dim],
                    self.canonical_math,
                );
            }
            matmul_batched(
                &self.exact_executor,
                q,
                norm,
                &w[self.wq + l * dim * dim..],
                dim,
                b,
            );
            matmul_batched(
                &self.exact_executor,
                ktmp,
                norm,
                &w[self.wk + l * dim * kv_dim..],
                dim,
                b,
            );
            matmul_batched(
                &self.exact_executor,
                vtmp,
                norm,
                &w[self.wv + l * dim * kv_dim..],
                dim,
                b,
            );
            for bi in 0..b {
                let loff = l * states[bi].sequence_capacity * kv_dim;
                let dst = loff + positions[bi] * kv_dim;
                states[bi].key_cache[dst..dst + kv_dim]
                    .copy_from_slice(&ktmp[bi * kv_dim..(bi + 1) * kv_dim]);
                states[bi].value_cache[dst..dst + kv_dim]
                    .copy_from_slice(&vtmp[bi * kv_dim..(bi + 1) * kv_dim]);
            }

            for bi in 0..b {
                let pos = positions[bi];
                let qb = &mut q[bi * dim..(bi + 1) * dim];
                let out = &mut attn[bi * dim..(bi + 1) * dim];
                let st = &mut states[bi];
                let loff = l * st.sequence_capacity * kv_dim;

                if c.rope_interleaved {
                    let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                    let rope_offset = pos * (head_size / 2);
                    let mut i = 0usize;
                    while i < dim {
                        let angle_index = rope_offset + (i % head_size) / 2;
                        let fcr = self.rope_cos[angle_index];
                        let fci = self.rope_sin[angle_index];
                        let rotn = if i < kv_dim { 2 } else { 1 };
                        for v in 0..rotn {
                            let vec: &mut [f32] = if v == 0 { &mut *qb } else { &mut *k };
                            let v0 = vec[i];
                            let v1 = vec[i + 1];
                            vec[i] = v0 * fcr - v1 * fci;
                            vec[i + 1] = v0 * fci + v1 * fcr;
                        }
                        i += 2;
                    }
                } else {
                    let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                    for vector in [&mut qb[..], &mut k[..]] {
                        for head in vector.chunks_exact_mut(head_size) {
                            let half = head_size / 2;
                            for i in 0..half {
                                let angle_index = pos * half + i;
                                let cos = self.rope_cos[angle_index];
                                let sin = self.rope_sin[angle_index];
                                let first = head[i];
                                let second = head[i + half];
                                head[i] = first * cos - second * sin;
                                head[i + half] = second * cos + first * sin;
                            }
                        }
                    }
                }

                // Same #602 reference functions as the serial
                // [`Llama::layer_forward`] path: the `r4_attention`
                // switch selects between the two registered operators.
                for h in 0..c.n_heads {
                    let qh = &qb[h * head_size..(h + 1) * head_size];
                    let att =
                        &mut st.att[h * st.sequence_capacity..h * st.sequence_capacity + pos + 1];
                    let kv_head_offset = (h / kv_mul) * head_size;
                    if c.r4_attention {
                        attention::experimental_r4_head_attention_weights(
                            att,
                            qh,
                            &st.key_cache[loff..],
                            kv_head_offset,
                            kv_dim,
                            self.canonical_math,
                        );
                    } else {
                        attention::standard_head_attention_weights(
                            att,
                            qh,
                            &st.key_cache[loff..],
                            kv_head_offset,
                            kv_dim,
                            self.canonical_math,
                        );
                    }
                    let att = &st.att[h * st.sequence_capacity..h * st.sequence_capacity + pos + 1];
                    let outh = &mut out[h * head_size..(h + 1) * head_size];
                    attention::head_attention_value_aggregate(
                        outh,
                        att,
                        &st.value_cache[loff..],
                        kv_head_offset,
                        kv_dim,
                    );
                }
            }

            matmul_batched(
                &self.exact_executor,
                o,
                attn,
                &w[self.wo + l * dim * dim..],
                dim,
                b,
            );
            for bi in 0..b {
                for i in 0..dim {
                    states[bi].x[i] += o[bi * dim + i];
                }
            }

            for bi in 0..b {
                rmsnorm_with_mode(
                    &mut norm[bi * dim..(bi + 1) * dim],
                    &states[bi].x,
                    &w[self.rms_ffn + l * dim..self.rms_ffn + (l + 1) * dim],
                    self.canonical_math,
                );
            }
            matmul_batched(
                &self.exact_executor,
                hb,
                norm,
                &w[self.w1 + l * dim * hid..],
                dim,
                b,
            );
            matmul_batched(
                &self.exact_executor,
                hb2,
                norm,
                &w[self.w3 + l * dim * hid..],
                dim,
                b,
            );
            for idx in 0..b * hid {
                let mut val = hb[idx];
                val *= 1.0f32 / (1.0f32 + expf(-val, self.canonical_math));
                val *= hb2[idx];
                hb[idx] = val;
            }
            matmul_batched(
                &self.exact_executor,
                ffn,
                hb,
                &w[self.w2 + l * hid * dim..],
                hid,
                b,
            );
            for bi in 0..b {
                for i in 0..dim {
                    states[bi].x[i] += ffn[bi * dim + i];
                }
            }
        }

        let rf = self.rms_final;
        for bi in 0..b {
            let (wslice, x) = (&w[rf..rf + dim], &mut states[bi].x);
            rmsnorm_inplace_with_mode(x, wslice, self.canonical_math);
            xstack[bi * dim..(bi + 1) * dim].copy_from_slice(x);
        }
        matmul_batched(
            &self.exact_executor,
            logits_stacked,
            xstack,
            &w[self.wcls..],
            dim,
            b,
        );
        for bi in 0..b {
            states[bi]
                .logits
                .copy_from_slice(&logits_stacked[bi * c.vocab..(bi + 1) * c.vocab]);
        }
        self.exact_executor.complete_forward(b);
    }
}

pub trait RepresentationSource {
    fn vocab_size(&self) -> usize;
    fn source_dimension(&self) -> usize;
    fn tokenizer_address(&self) -> &str;
    /// Copy the embedding rows in `range` into `output`. Total: returns
    /// `None` when the caller's `output` buffer cannot hold the requested
    /// rows (a property of the caller's chosen instantiation), `Some(())`
    /// once the rows are written.
    fn read_embedding_rows(&self, range: std::ops::Range<usize>, output: &mut [f32]) -> Option<()>;
}

pub trait BehaviorSource {
    fn reset(&mut self);
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]);
}

/// The bounded dimensions a trace-capturing oracle exposes for the #603
/// teacher-trace lanes: how many layers exist (bounding the declarable
/// layer indices), how many attention heads and kv heads each layer has
/// (bounding the attention-support and k/v lane widths), and the width of
/// the residual stream the capture taps (`cfg.dim`, the SOURCE width — for
/// the Hugging Face adapter this is 576, not the compiled 288 that
/// [`TeacherOracle::dim`] presents).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceCaptureGeometry {
    /// Number of transformer layers (declared capture indices must be
    /// strictly below this).
    pub layers: usize,
    /// Attention heads per layer.
    pub heads: usize,
    /// Key/value heads per layer (grouped query).
    pub kv_heads: usize,
    /// Residual-stream width of the captured `st.x` / q vectors (the
    /// source `cfg.dim`). K/v rows are
    /// `residual_width * kv_heads / heads` wide.
    pub residual_width: usize,
}

/// Which layer indices each #603 trace lane captures during one
/// [`TeacherOracle::step_with_trace_capture`]. Empty slices capture
/// nothing for that lane; indices outside the model's layer range are
/// never matched (the caller validates them against
/// [`TraceCaptureGeometry::layers`] up front).
#[derive(Debug, Clone, Copy)]
pub struct TraceCaptureRequest<'a> {
    /// Post-layer residual-stream capture indices.
    pub residual_layers: &'a [usize],
    /// Current-position q/k/v capture indices.
    pub qkv_layers: &'a [usize],
    /// Per-head attention-weight capture indices.
    pub attention_layers: &'a [usize],
}

/// `(layer, post-layer residual stream)` sink of one traced step.
pub type ResidualSink<'a> = dyn FnMut(usize, &[f32]) + 'a;
/// `(layer, rotated q at the current position, k cache row, v cache
/// row)` sink of one traced step.
pub type QkvSink<'a> = dyn FnMut(usize, &[f32], &[f32], &[f32]) + 'a;
/// `(layer, head, softmax weights over positions 0..=pos)` sink of one
/// traced step.
pub type AttentionSupportSink<'a> = dyn FnMut(usize, usize, &[f32]) + 'a;

/// The capture sinks one traced step feeds. Each is called only for the
/// declared layer indices, in ascending layer order (heads ascending
/// within a layer for the attention sink), once per step.
pub struct TraceCaptureSinks<'a, 'b> {
    /// See [`ResidualSink`].
    pub residual: &'a mut ResidualSink<'b>,
    /// See [`QkvSink`].
    pub qkv: &'a mut QkvSink<'b>,
    /// See [`AttentionSupportSink`].
    pub attention: &'a mut AttentionSupportSink<'b>,
}

/// The TWO-SURFACE interface every source architecture must expose to the
/// compiler: the embedding table (representation) and a sequential
/// next-token forward (behavior). The compiler is written against this
/// trait and CANNOT touch anything else — the architecture-generality
/// claim (PROOF.md P4) is enforced by construction, not by inspection.
/// A qwen- or phi-class source implements this trait and nothing
/// downstream changes.
pub trait TeacherOracle: RepresentationSource + BehaviorSource {
    fn vocab(&self) -> usize;
    fn dim(&self) -> usize;
    fn seq_len(&self) -> usize;
    fn bos_token(&self) -> usize {
        1
    }
    fn eos_token(&self) -> usize {
        1
    }
    /// κ of the source artifact this oracle wraps.
    fn kappa(&self) -> String;
    /// Size in bytes of the source artifact (compression accounting).
    fn source_bytes(&self) -> usize;
    /// Copy the embedding row of `token` into `out` (len == dim).
    fn embedding(&self, token: usize, out: &mut [f32]);

    /// The #600 typed record of the source→compiled geometry projection
    /// this oracle applies inside [`TeacherOracle::embedding`], when it
    /// applies one. `None` means embedding rows pass through at the
    /// source width ([`TeacherOracle::dim`] ==
    /// [`RepresentationSource::source_dimension`]); the default keeps
    /// every existing oracle unaffected.
    fn geometry_projection(&self) -> Option<geometry::GeometryProjection> {
        None
    }

    /// The #602 typed record of the attention operator this oracle's
    /// source executor computes during [`BehaviorSource::step`], when the
    /// oracle declares one. The boolean `r4_attention` switch maps to
    /// exactly the two registered operators
    /// (`standard-source-attention/2` when off,
    /// `experimental-r4-source-attention/2` when on — see
    /// [`attention::operator_for_r4_switch`]). `None` means the oracle
    /// predates the record (the legacy interpretation documented in
    /// `docs/MODEL_LIFECYCLE.md`); the default keeps every existing
    /// oracle unaffected, mirroring
    /// [`TeacherOracle::geometry_projection`].
    fn attention_operator_spec(&self) -> Option<attention::AttentionOperatorSpec> {
        None
    }

    /// Typed identity of the source-dense arithmetic executed by
    /// [`BehaviorSource::step`], when the oracle declares one. `None` keeps
    /// historical and non-dense-registry teachers compatible without
    /// relabelling them.
    fn dense_operator_spec(&self) -> Option<dense::DenseOperatorSpec> {
        None
    }

    /// Optional compiler-only trace surface (graph-compiler plan §5 Phase
    /// 2): the final hidden state (post-final-rmsnorm activation) of the
    /// last `step`, if the oracle retains it. Defaults to `None` so
    /// existing oracles are unaffected.
    fn hidden_state(&self) -> Option<&[f32]> {
        None
    }

    /// Optional compiler-only trace surface: the top-k (token, probability)
    /// pairs of the last `step`'s softmax distribution, ordered by
    /// descending probability with a canonical tie-break (higher
    /// probability, then lower token id). Writes at most
    /// `min(k, out.len())` pairs and returns the count written. Defaults
    /// to 0 so existing oracles are unaffected.
    fn top_k(&self, k: usize, out: &mut [(u32, f32)]) -> usize {
        let _ = (k, out);
        0
    }

    /// Optional compiler-only #603 trace-capture surface: the bounded
    /// capture dimensions this oracle exposes, when it supports the
    /// traced step. `None` means the oracle has no bounded per-layer
    /// capture path (richer trace profiles must be refused, never
    /// zero-filled); the default keeps every existing oracle unaffected,
    /// mirroring [`TeacherOracle::hidden_state`].
    fn trace_capture_geometry(&self) -> Option<TraceCaptureGeometry> {
        None
    }

    /// Optional compiler-only #603 trace-capture step: one forward step
    /// with the declared lanes captured through the production executor path
    /// (`Llama::forward_capturing_trace`, the #599 `forward_capturing`
    /// discipline extended with the q/k/v and per-head attention taps).
    /// Returns `true` when the oracle captured through its executor;
    /// the default performs a plain [`BehaviorSource::step`] and returns
    /// `false` — the caller must treat `false` as "this oracle has no
    /// capture surface" and refuse the richer profile rather than emit
    /// absent lanes as zeros.
    fn step_with_trace_capture(
        &mut self,
        token: usize,
        pos: usize,
        logits: &mut [f32],
        request: &TraceCaptureRequest<'_>,
        sinks: &mut TraceCaptureSinks<'_, '_>,
    ) -> bool {
        let _ = (request, sinks);
        self.step(token, pos, logits);
        false
    }
}

/// Shared top-k trace computation over the raw logits a llama-family
/// `State` retains after `step`. Softmax is computed in the same f32
/// max-subtracted form the corpus generator uses; ordering is canonical
/// (probability descending, token id ascending on ties).
pub(crate) fn top_k_from_logits(
    logits: &[f32],
    k: usize,
    out: &mut [(u32, f32)],
    canonical_math: bool,
) -> usize {
    let count = k.min(out.len()).min(logits.len());
    if count == 0 {
        return 0;
    }
    let mut max = logits[0];
    for &logit in &logits[1..] {
        if logit > max {
            max = logit;
        }
    }
    let mut sum = 0.0f32;
    let mut probs = vec![0.0f32; logits.len()];
    for (prob, &logit) in probs.iter_mut().zip(logits.iter()) {
        *prob = expf(logit - max, canonical_math);
        sum += *prob;
    }
    for prob in probs.iter_mut() {
        *prob /= sum;
    }
    let mut order: Vec<u32> = (0..logits.len() as u32).collect();
    order.sort_by(|a, b| {
        probs[*b as usize]
            .total_cmp(&probs[*a as usize])
            .then_with(|| a.cmp(b))
    });
    for (dest, &token) in out.iter_mut().zip(order.iter()).take(count) {
        *dest = (token, probs[token as usize]);
    }
    count
}

/// The llama-family adapter: `Llama` plus its recurrent state.
pub struct LlamaOracle {
    pub model: Llama,
    state: State,
    kappa: String,
    source_bytes: usize,
}

impl LlamaOracle {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(path: &str) -> Self {
        let bytes = std::fs::read(path).expect("source checkpoint");
        let kappa = format!("blake3:{}", blake3::hash(&bytes).to_hex());
        let source_bytes = bytes.len();
        let model = Llama::load(path);
        let state = State::new(&model.cfg);
        LlamaOracle {
            model,
            state,
            kappa,
            source_bytes,
        }
    }
}

impl RepresentationSource for LlamaOracle {
    fn vocab_size(&self) -> usize {
        self.model.cfg.vocab
    }
    fn source_dimension(&self) -> usize {
        self.model.cfg.dim
    }
    fn tokenizer_address(&self) -> &str {
        "local-llama-tokenizer"
    }
    fn read_embedding_rows(&self, range: std::ops::Range<usize>, output: &mut [f32]) -> Option<()> {
        let d = self.model.cfg.dim;
        let count = range.end - range.start;
        if output.len() < count * d {
            return None;
        }
        let start_offset = self.model.emb + range.start * d;
        let end_offset = self.model.emb + range.end * d;
        output[..count * d].copy_from_slice(&self.model.w[start_offset..end_offset]);
        Some(())
    }
}

impl BehaviorSource for LlamaOracle {
    fn reset(&mut self) {
        self.state.reset();
    }
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]) {
        self.model.forward(&mut self.state, token, pos, false);
        logits.copy_from_slice(&self.state.logits);
    }
}

impl TeacherOracle for LlamaOracle {
    fn vocab(&self) -> usize {
        self.model.cfg.vocab
    }
    fn dim(&self) -> usize {
        self.model.cfg.dim
    }
    fn seq_len(&self) -> usize {
        self.model.cfg.seq_len
    }
    fn kappa(&self) -> String {
        self.kappa.clone()
    }
    fn source_bytes(&self) -> usize {
        self.source_bytes
    }
    fn embedding(&self, token: usize, out: &mut [f32]) {
        let d = self.model.cfg.dim;
        out.copy_from_slice(
            &self.model.w[self.model.emb + token * d..self.model.emb + (token + 1) * d],
        );
    }
    fn attention_operator_spec(&self) -> Option<attention::AttentionOperatorSpec> {
        // #704: this legacy checkpoint adapter executes the same current
        // Llama attention implementation and public switch as the Safetensors
        // adapter. Bind the branch actually selected by its mutable config.
        // `None` is reserved for already-produced, pre-provenance corpora; a
        // new observation must not be mislabeled as implicit standard/1 by
        // omission when the current registry alias advances.
        Some(attention::operator_for_r4_switch(
            self.model.cfg.r4_attention,
        ))
    }
    fn hidden_state(&self) -> Option<&[f32]> {
        Some(&self.state.x)
    }
    fn top_k(&self, k: usize, out: &mut [(u32, f32)]) -> usize {
        top_k_from_logits(&self.state.logits, k, out, self.model.canonical_math)
    }
    fn trace_capture_geometry(&self) -> Option<TraceCaptureGeometry> {
        Some(TraceCaptureGeometry {
            layers: self.model.cfg.n_layers,
            heads: self.model.cfg.n_heads,
            kv_heads: self.model.cfg.n_kv_heads,
            residual_width: self.model.cfg.dim,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn step_with_trace_capture(
        &mut self,
        token: usize,
        pos: usize,
        logits: &mut [f32],
        request: &TraceCaptureRequest<'_>,
        sinks: &mut TraceCaptureSinks<'_, '_>,
    ) -> bool {
        // Same pinned exact matmul owner as `step`, so a traced step and a
        // plain step produce identical bits.
        self.model
            .forward_capturing_trace(&mut self.state, token, pos, false, request, sinks);
        logits.copy_from_slice(&self.state.logits);
        true
    }
}

#[derive(serde::Deserialize)]
struct HuggingFaceConfig {
    hidden_size: usize,
    intermediate_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: usize,
    vocab_size: usize,
    max_position_embeddings: usize,
    #[serde(default = "default_rope_theta")]
    rope_theta: f32,
    #[serde(default = "default_rms_epsilon")]
    rms_norm_eps: f32,
    #[serde(default)]
    tie_word_embeddings: bool,
    #[serde(default = "default_bos_token")]
    bos_token_id: usize,
    #[serde(default = "default_eos_token")]
    eos_token_id: usize,
    #[serde(default)]
    rope_interleaved: bool,
}

fn default_rope_theta() -> f32 {
    10_000.0
}
fn default_rms_epsilon() -> f32 {
    1e-5
}
fn default_bos_token() -> usize {
    1
}
fn default_eos_token() -> usize {
    2
}

/// Compute fail-closed probe admission geometry from `config.json` only.
///
/// This function never opens a Safetensors shard or allocates model weights,
/// so a caller can validate durable probe evidence before authorizing the
/// multi-gigabyte teacher load. The same owner functions used by a live
/// [`Llama`] compute every counter and trace dimension.
#[cfg(not(target_arch = "wasm32"))]
pub fn exact_probe_expectation_shapes_from_config(
    source: impl AsRef<std::path::Path>,
    sequence_length: usize,
    worker_counts: &[usize],
    tiles_per_worker: usize,
    streams: usize,
    probe_positions: usize,
    top_k: usize,
) -> Result<ExactMulticoreProbeExpectationShapes, SourceUnavailable> {
    if sequence_length == 0
        || streams == 0
        || probe_positions == 0
        || tiles_per_worker == 0
        || worker_counts.is_empty()
        || worker_counts.contains(&0)
    {
        return Err(SourceUnavailable::new(
            "exact probe config-only planner requires nonzero geometry and worker bounds",
        ));
    }
    let mut unique_workers = std::collections::BTreeSet::new();
    if !worker_counts
        .iter()
        .all(|workers| unique_workers.insert(*workers))
    {
        return Err(SourceUnavailable::new(
            "exact probe config-only planner worker counts must be unique",
        ));
    }
    let config_bytes = std::fs::read(source.as_ref().join("config.json"))?;
    let raw_config: serde_json::Value = serde_json::from_slice(&config_bytes)?;
    conformance::AdapterFeatures::huggingface_llama().validate_config(&raw_config)?;
    let config: HuggingFaceConfig = serde_json::from_slice(&config_bytes)?;
    let cfg = Config {
        dim: config.hidden_size,
        hidden: config.intermediate_size,
        n_layers: config.num_hidden_layers,
        n_heads: config.num_attention_heads,
        n_kv_heads: config.num_key_value_heads,
        vocab: config.vocab_size,
        seq_len: sequence_length.min(config.max_position_embeddings),
        rope_theta: config.rope_theta,
        rms_norm_eps: config.rms_norm_eps,
        rope_interleaved: config.rope_interleaved,
        r4_attention: false,
    };
    let forward_plans = worker_counts
        .iter()
        .map(|&workers| {
            exact_forward_plan_for_geometry(&cfg, streams, |rows| {
                exact_executor::exact_row_tiles_for(rows, workers, tiles_per_worker)
            })
            .map(|forward_plan| ExactMulticoreProbeWorkerPlan {
                workers,
                forward_plan,
            })
        })
        .collect::<Result<Vec<_>, ExactForwardPlanError>>()
        .map_err(|error| SourceUnavailable::new(error.to_string()))?;
    let trace_shape = exact_probe_trace_shape_for_geometry(
        &cfg,
        sequence_length,
        probe_positions,
        streams,
        top_k,
    )
    .map_err(|error| SourceUnavailable::new(error.to_string()))?;
    Ok(ExactMulticoreProbeExpectationShapes {
        forward_plans,
        trace_shape,
    })
}

/// Offline teacher adapter for Hugging Face Llama-family Safetensors
/// (BF16/F16/F32; single-file or #598 indexed shards, ingested through the
/// [`SafetensorsSnapshot`] validation boundary).
/// The full source model executes only while compiling; deployed inference
/// continues to use the multiplication-free [`super::runtime`] tables.
pub struct HuggingFaceLlamaOracle {
    model: Llama,
    state: State,
    kappa: String,
    source_bytes: usize,
    bos_token: usize,
    eos_token: usize,
    fast_matmul: bool,
}

/// The sanctioned host-ingestion boundary error (R5).
///
/// A declared external source artifact — a teacher directory, its
/// `model.safetensors`, `config.json`, an indexed shard set, or a named
/// tensor within — could not be ingested into a valid teacher at
/// construction. This is the host-side analogue of graph-format's
/// `NotAProduct`: the only reportable condition is that the requested object
/// does not exist as a valid product, reported at construction. It carries
/// the operator-facing diagnostic (which file, which tensor, what dtype)
/// rather than discarding it, and — since #598 — the structured
/// [`SourceIngestKind`] classifying which ingestion check refused the
/// artifact, so callers and tests can assert the exact failure class without
/// this crate growing a second error surface.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct SourceUnavailable {
    /// Human-facing description of why the source could not be ingested.
    pub reason: String,
    /// The #598 ingestion failure class, when a validation-boundary check
    /// produced this error; [`SourceIngestKind::Unspecified`] otherwise
    /// (I/O, JSON, or legacy construction sites).
    pub kind: SourceIngestKind,
}

#[cfg(not(target_arch = "wasm32"))]
impl SourceUnavailable {
    /// Construct from any displayable reason (failure class
    /// [`SourceIngestKind::Unspecified`]).
    pub fn new(reason: impl std::fmt::Display) -> Self {
        Self {
            reason: reason.to_string(),
            kind: SourceIngestKind::Unspecified,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for SourceUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "source unavailable: {}", self.reason)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::error::Error for SourceUnavailable {}

#[cfg(not(target_arch = "wasm32"))]
impl From<std::io::Error> for SourceUnavailable {
    fn from(error: std::io::Error) -> Self {
        Self::new(error)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<serde_json::Error> for SourceUnavailable {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<safetensors::SafeTensorError> for SourceUnavailable {
    fn from(error: safetensors::SafeTensorError) -> Self {
        Self::new(error)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<SourceIngestKind> for SourceUnavailable {
    fn from(kind: SourceIngestKind) -> Self {
        Self {
            reason: kind.to_string(),
            kind,
        }
    }
}

/// Filesystem-free counterpart of the sanctioned source-unavailable error.
/// Browser serving never performs host ingestion, but portable production
/// admission still needs the same typed failure boundary for malformed or
/// mismatched in-memory components.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct SourceUnavailable {
    pub reason: String,
}

#[cfg(target_arch = "wasm32")]
impl SourceUnavailable {
    pub fn new(reason: impl std::fmt::Display) -> Self {
        Self {
            reason: reason.to_string(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl std::fmt::Display for SourceUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "source unavailable: {}", self.reason)
    }
}

#[cfg(target_arch = "wasm32")]
impl std::error::Error for SourceUnavailable {}

/// Exact-widening source codecs (#598): BF16, F16, and F32 → `f32`.
///
/// Every BF16 and F16 value — normals, subnormals, ±0, ±infinity, and NaN —
/// is exactly representable in `f32`, so widening is a pure bit rewrite: no
/// rounding path exists in this module. NaN widening is payload-preserving:
/// the narrow mantissa payload (including the quiet bit) is shifted into the
/// top of the `f32` mantissa (BF16 by 16 bits, F16 by 13 bits) and the sign
/// is kept, so a BF16/F16 NaN widens to an `f32` NaN carrying the same
/// payload bits. F32 is decoded little-endian, bit-identically.
pub mod codec {
    /// Widen one BF16 bit pattern to the `f32` with the identical value.
    /// BF16 is the top half of an `f32`, so this is a 16-bit left shift.
    #[inline]
    pub const fn bf16_to_f32(bits: u16) -> f32 {
        f32::from_bits((bits as u32) << 16)
    }

    /// Widen one IEEE 754 binary16 bit pattern to the `f32` with the
    /// identical value. Normals re-bias the exponent (+112), subnormals are
    /// normalized (every F16 subnormal is an `f32` normal), and ±infinity
    /// and NaN map onto the `f32` exponent 0xFF with the mantissa payload
    /// shifted left by 13.
    #[inline]
    pub const fn f16_to_f32(bits: u16) -> f32 {
        let sign = ((bits as u32) & 0x8000) << 16;
        let exponent = ((bits >> 10) & 0x1F) as u32;
        let mantissa = (bits as u32) & 0x3FF;
        let widened = if exponent == 0x1F {
            // Infinity (mantissa 0) or NaN (payload shifted, preserved).
            sign | 0x7F80_0000 | (mantissa << 13)
        } else if exponent != 0 {
            // Normal: re-bias 15 → 127.
            sign | ((exponent + 112) << 23) | (mantissa << 13)
        } else if mantissa == 0 {
            // ±0.
            sign
        } else {
            // Subnormal: value = mantissa × 2⁻²⁴; normalize the leading one
            // away. With the mantissa's most significant bit at position p,
            // the shift is 10 − p and the biased f32 exponent is 103 + p.
            let shift = mantissa.leading_zeros() - 21;
            sign | ((113 - shift) << 23) | (((mantissa << shift) & 0x3FF) << 13)
        };
        f32::from_bits(widened)
    }

    /// Decode one little-endian F32 value, bit-identically.
    #[inline]
    pub const fn f32_from_le(bytes: [u8; 4]) -> f32 {
        f32::from_le_bytes(bytes)
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Append the exact `f32` widening of `data` (one tensor's raw bytes in
    /// `dtype`) to `out`. The caller guarantees `data.len()` is a multiple
    /// of the dtype's byte size — the #598 validation boundary checks every
    /// tensor's byte length against shape × dtype size before loading.
    pub(crate) fn append_widened(dtype: super::SourceDtype, data: &[u8], out: &mut Vec<f32>) {
        match dtype {
            super::SourceDtype::Bf16 => {
                for bytes in data.chunks_exact(2) {
                    out.push(bf16_to_f32(u16::from_le_bytes([bytes[0], bytes[1]])));
                }
            }
            super::SourceDtype::F16 => {
                for bytes in data.chunks_exact(2) {
                    out.push(f16_to_f32(u16::from_le_bytes([bytes[0], bytes[1]])));
                }
            }
            super::SourceDtype::F32 => {
                for bytes in data.chunks_exact(4) {
                    out.push(f32_from_le([bytes[0], bytes[1], bytes[2], bytes[3]]));
                }
            }
        }
    }
}

/// File name of the Hugging Face Safetensors shard index.
#[cfg(not(target_arch = "wasm32"))]
pub const SAFETENSORS_INDEX_FILE_NAME: &str = "model.safetensors.index.json";
/// File name of the single-file (unsharded) Safetensors weights artifact.
#[cfg(not(target_arch = "wasm32"))]
pub const SAFETENSORS_SINGLE_FILE_NAME: &str = "model.safetensors";

/// The source dtypes this crate can ingest (#598). Each widens exactly to
/// `f32` through [`codec`]; nothing else is admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceDtype {
    /// bfloat16 — the top half of an `f32`.
    Bf16,
    /// IEEE 754 binary16.
    F16,
    /// IEEE 754 binary32, stored little-endian.
    F32,
}

impl SourceDtype {
    /// Bytes per element in the Safetensors data section.
    pub const fn byte_size(self) -> usize {
        match self {
            Self::Bf16 | Self::F16 => 2,
            Self::F32 => 4,
        }
    }

    /// The Safetensors header spelling of this dtype.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bf16 => "BF16",
            Self::F16 => "F16",
            Self::F32 => "F32",
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn classify(declared: &str) -> Option<Self> {
        match declared {
            "BF16" => Some(Self::Bf16),
            "F16" => Some(Self::F16),
            "F32" => Some(Self::F32),
            _ => None,
        }
    }
}

/// The #598 source-ingestion failure class: one variant per check at the
/// [`SafetensorsSnapshot::open`] validation boundary, all detected before
/// any model construction. This is NOT a second error surface — the one
/// sanctioned host-ingestion error remains [`SourceUnavailable`] (R5);
/// this enum rides inside it as the structured `kind` field, so each
/// variant names the offending object and callers/tests can still assert
/// the exact failure class.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceIngestKind {
    /// No structured ingestion class: the error came from a plain I/O,
    /// JSON, or legacy construction site ([`SourceUnavailable::new`]).
    Unspecified,
    /// A snapshot file exists but could not be read or parsed (I/O error,
    /// malformed JSON header or index, non-portable shard name).
    Unreadable {
        /// The file (or file: tensor context) that failed.
        path: String,
        /// The underlying parse or I/O diagnostic.
        reason: String,
    },
    /// A shard file referenced by the index (or the single
    /// `model.safetensors`) is absent from the snapshot directory.
    MissingShardFile {
        /// The missing shard's file name.
        shard: String,
    },
    /// A tensor is declared (by the index or by the model geometry) but the
    /// shard that should carry it does not.
    MissingTensor {
        /// The declared tensor name.
        tensor: String,
        /// Where the tensor was expected (a shard file, or the index/single
        /// file for a geometry-required tensor missing everywhere).
        shard: String,
    },
    /// A tensor is claimed more than once: two shard headers both declare
    /// it, or the raw index JSON maps the same name twice.
    DuplicateTensor {
        /// The doubly-claimed tensor name.
        tensor: String,
        /// The first claimant (shard file, or the index file itself).
        first_shard: String,
        /// The second claimant.
        second_shard: String,
    },
    /// A shard header declares a tensor the index does not assign to any
    /// shard. Only possible in indexed mode; a single-file snapshot has no
    /// index, so extra tensors there are ignored exactly as before #598.
    UnexpectedTensor {
        /// The unindexed tensor name.
        tensor: String,
        /// The shard whose header declares it.
        shard: String,
    },
    /// A geometry-required tensor's declared shape does not match the shape
    /// `config.json` implies.
    ShapeMismatch {
        /// The tensor name.
        tensor: String,
        /// The shape the model geometry requires.
        expected: Vec<usize>,
        /// The shape the shard header declares.
        actual: Vec<usize>,
    },
    /// A declared byte length is inconsistent: a tensor's data span differs
    /// from shape × dtype size, the data spans are not contiguous, or the
    /// shard file's size differs from what its header claims.
    ByteLengthMismatch {
        /// Which check failed (shard file, or shard file + tensor).
        context: String,
        /// The byte length the header/shape claims.
        expected: u64,
        /// The byte length actually found.
        actual: u64,
    },
    /// The same tensor name is declared with two different dtypes.
    DtypeInconsistency {
        /// The tensor name.
        tensor: String,
        /// First declaration, as `DTYPE (in <shard>)`.
        first: String,
        /// Conflicting declaration, as `DTYPE (in <shard>)`.
        second: String,
    },
    /// A tensor declares a dtype outside BF16/F16/F32 — e.g. quantized I8,
    /// U8, or GPTQ/AWQ-style packed formats. Rejected by name, never
    /// silently approximated (#598 non-goal).
    UnsupportedDtype {
        /// The tensor name.
        tensor: String,
        /// The declared dtype/format string from the shard header.
        dtype: String,
    },
    /// #599 adapter-conformance gate: the parsed `config.json` declares a
    /// feature outside the adapter's typed
    /// [`conformance::AdapterFeatures`] declaration — an activation, norm
    /// epsilon, RoPE mode, head geometry, bias, embedding-tying, or token
    /// policy the source executor would otherwise silently misinterpret.
    /// Detected at oracle construction, before any tensor is read and
    /// before any observation can be generated (fail-closed).
    UnsupportedConfigFeature {
        /// Which declared feature the configuration falls outside of.
        feature: conformance::AdapterFeature,
        /// What the adapter declares it supports for this feature.
        declared: String,
        /// What `config.json` actually contains.
        actual: String,
    },
    /// #600 geometry registry: a caller named a source→compiled geometry
    /// projection `(id, version)` outside the versioned registry
    /// ([`geometry::projection_implementation`]), so no implementation
    /// exists to interpret it. Refused by name, never approximated by a
    /// "closest" algorithm or version.
    UnknownGeometryProjection {
        /// The requested projection algorithm id.
        id: String,
        /// The requested projection version.
        version: u32,
    },
    /// #601 tokenizer-adapter registry: a caller named a tokenizer
    /// adapter `(family, version)` outside the versioned registry
    /// (`uor_r4_core::transformerless::hf_bpe::adapter_constructor`),
    /// so no constructor exists to interpret it. Refused by name, never
    /// approximated by a "closest" family or version. Registered entries
    /// include `hf-byte-bpe/1` plus frozen `sentencepiece-unigram/1` and
    /// current `sentencepiece-unigram/2`; any unregistered family/version pair
    /// is rejected here.
    UnknownTokenizerAdapter {
        /// The requested adapter family.
        family: String,
        /// The requested adapter version.
        version: u32,
    },
    /// #602 attention-operator registry: a caller named a source
    /// attention operator `(id, version)` outside the versioned registry
    /// ([`attention::operator_spec`]), so no specification exists to
    /// interpret it. Refused by name, never approximated by a "closest"
    /// operator or version.
    UnknownAttentionOperator {
        /// The requested operator id.
        id: String,
        /// The requested operator version.
        version: u32,
    },
    /// Source-dense operator registry: a caller named an `(id, version)`
    /// outside [`dense::operator_spec`]. Unknown entries are never
    /// approximated by a nearby arithmetic version.
    UnknownDenseOperator {
        /// The requested dense operator id.
        id: String,
        /// The requested dense operator version.
        version: u32,
    },
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for SourceIngestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unspecified => write!(f, "no structured ingestion class"),
            Self::Unreadable { path, reason } => write!(f, "{path}: {reason}"),
            Self::MissingShardFile { shard } => {
                write!(
                    f,
                    "shard file {shard} is absent from the snapshot directory"
                )
            }
            Self::MissingTensor { tensor, shard } => {
                write!(f, "tensor {tensor} is declared but absent from {shard}")
            }
            Self::DuplicateTensor {
                tensor,
                first_shard,
                second_shard,
            } => write!(
                f,
                "tensor {tensor} is claimed twice ({first_shard} and {second_shard})"
            ),
            Self::UnexpectedTensor { tensor, shard } => write!(
                f,
                "tensor {tensor} appears in shard {shard} but not in the index"
            ),
            Self::ShapeMismatch {
                tensor,
                expected,
                actual,
            } => write!(
                f,
                "tensor {tensor} has shape {actual:?}, expected {expected:?}"
            ),
            Self::ByteLengthMismatch {
                context,
                expected,
                actual,
            } => write!(
                f,
                "byte-length mismatch at {context}: expected {expected} bytes, found {actual}"
            ),
            Self::DtypeInconsistency {
                tensor,
                first,
                second,
            } => write!(f, "tensor {tensor} is declared {first} and {second}"),
            Self::UnsupportedDtype { tensor, dtype } => write!(
                f,
                "tensor {tensor} is {dtype}; supported source dtypes are BF16, F16, F32 \
                 (quantized formats are rejected, not approximated)"
            ),
            Self::UnsupportedConfigFeature {
                feature,
                declared,
                actual,
            } => write!(
                f,
                "config.json feature {feature} is outside the adapter's declared support: \
                 adapter declares {declared}, configuration has {actual} \
                 (rejected before any observation)"
            ),
            Self::UnknownGeometryProjection { id, version } => write!(
                f,
                "geometry projection {id}/{version} is not in the versioned projection \
                 registry (known: {}/{})",
                geometry::GeometryProjection::BUCKET_AVERAGE_ID,
                geometry::GeometryProjection::BUCKET_AVERAGE_VERSION,
            ),
            Self::UnknownTokenizerAdapter { family, version } => write!(
                f,
                "tokenizer adapter {family}/{version} is not in the versioned adapter \
                 registry (registered entries include hf-byte-bpe/1, \
                 sentencepiece-unigram/1, and sentencepiece-unigram/2; unknown families or \
                 versions are never approximated)"
            ),
            Self::UnknownAttentionOperator { id, version } => write!(
                f,
                "attention operator {id}/{version} is not in the versioned operator \
                 registry (registered entries include {}/{}, {}/{}, {}/{}, {}/{}, \
                 {}/{}, {}/{}, and {}/{})",
                attention::AttentionOperatorSpec::STANDARD_ID,
                attention::AttentionOperatorSpec::STANDARD_V1_VERSION,
                attention::AttentionOperatorSpec::STANDARD_ID,
                attention::AttentionOperatorSpec::STANDARD_V2_VERSION,
                attention::AttentionOperatorSpec::EXPERIMENTAL_R4_ID,
                attention::AttentionOperatorSpec::EXPERIMENTAL_R4_V1_VERSION,
                attention::AttentionOperatorSpec::EXPERIMENTAL_R4_ID,
                attention::AttentionOperatorSpec::EXPERIMENTAL_R4_V2_VERSION,
                attention::AttentionOperatorSpec::LEARNED_ABSOLUTE_ID,
                attention::AttentionOperatorSpec::LEARNED_ABSOLUTE_V1_VERSION,
                attention::AttentionOperatorSpec::LEARNED_ABSOLUTE_ID,
                attention::AttentionOperatorSpec::LEARNED_ABSOLUTE_V2_VERSION,
                attention::AttentionOperatorSpec::R4_ROUTE_ID,
                attention::AttentionOperatorSpec::R4_ROUTE_VERSION,
            ),
            Self::UnknownDenseOperator { id, version } => write!(
                f,
                "dense operator {id}/{version} is not in the versioned operator registry \
                 (registered entries: {}/{}, {}/{})",
                dense::DenseOperatorSpec::GPT2_ID,
                dense::DenseOperatorSpec::GPT2_V1_VERSION,
                dense::DenseOperatorSpec::GPT2_ID,
                dense::DenseOperatorSpec::GPT2_V2_VERSION,
            ),
        }
    }
}

/// One tensor the model geometry requires from a snapshot: its name and the
/// shape `config.json` implies. [`SafetensorsSnapshot::open`] verifies
/// presence, shape, and (implicitly, via the global dtype check) codec
/// support for every requirement before the model is constructed.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TensorRequirement {
    /// The Safetensors tensor name.
    pub name: String,
    /// The required shape.
    pub shape: Vec<usize>,
}

/// One tensor's location and declared metadata within a loaded shard.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct TensorMeta {
    dtype: SourceDtype,
    shape: Vec<usize>,
    /// Byte offsets relative to the shard's data section.
    begin: usize,
    end: usize,
}

/// One validated shard: its file name, raw bytes, data-section offset, and
/// header-declared tensors.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct Shard {
    name: String,
    bytes: Vec<u8>,
    data_start: usize,
    tensors: std::collections::BTreeMap<String, TensorMeta>,
}

/// JSON map entries in document order, duplicate keys preserved. serde_json
/// maps silently drop duplicate keys, so both the shard-index `weight_map`
/// and the Safetensors headers are parsed through this visitor to make
/// duplicate tensor names detectable.
#[cfg(not(target_arch = "wasm32"))]
struct MapEntries(Vec<(String, serde_json::Value)>);

#[cfg(not(target_arch = "wasm32"))]
impl<'de> serde::Deserialize<'de> for MapEntries {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EntriesVisitor;
        impl<'de> serde::de::Visitor<'de> for EntriesVisitor {
            type Value = MapEntries;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a JSON object")
            }
            fn visit_map<D>(self, mut map: D) -> Result<Self::Value, D::Error>
            where
                D: serde::de::MapAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, serde_json::Value>()? {
                    entries.push(entry);
                }
                Ok(MapEntries(entries))
            }
        }
        deserializer.deserialize_map(EntriesVisitor)
    }
}

/// The raw `model.safetensors.index.json` document. Unknown fields (e.g.
/// `metadata.total_size`) are tolerated; `weight_map` keeps duplicate keys
/// visible for the duplicate-tensor check.
#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Deserialize)]
struct RawShardIndex {
    weight_map: MapEntries,
}

/// One tensor's declared header record: dtype spelling, shape, and data
/// span. Mirrors the Safetensors header schema.
#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Deserialize)]
struct RawTensorInfo {
    dtype: String,
    shape: Vec<u64>,
    data_offsets: (u64, u64),
}

/// A fully validated Safetensors snapshot: THE #598 ingestion boundary.
///
/// [`SafetensorsSnapshot::open`] is the single deterministic validation
/// entry point for both snapshot layouts:
///
/// * **Indexed shards** — `model.safetensors.index.json` is parsed and every
///   tensor is resolved to exactly one shard file.
/// * **Single file** — `model.safetensors` alone is treated as a one-shard
///   resolution with no index (the pre-#598 layout); it flows through the
///   same code path, so behavior and κ are unchanged.
///
/// Validation runs over all shard headers BEFORE any tensor data is decoded
/// or any model is constructed, and every failure class maps to one
/// [`SourceIngestKind`] variant: missing shard files and tensors,
/// duplicate and unexpected tensors, shape mismatches against the model
/// geometry, byte-length mismatches (tensor span vs shape × dtype size,
/// non-contiguous spans, shard file size vs header claim), dtype
/// inconsistencies, and unsupported (e.g. quantized) dtypes.
///
/// Seam with the #597 snapshot manifest: this boundary checks only that
/// every shard file the index references exists in the directory — the
/// byte-length checks above come from the shard headers themselves. The
/// full `source_manifest.json` digest cross-check (path/bytes/blake3 per
/// admitted file, root κ) belongs to the root crate
/// (`src/model.rs::read_source_manifest`); `uor-r4-model-source` stays free
/// of that dependency by design.
///
/// The snapshot κ is `blake3:<hex>` over the concatenated shard bytes in
/// lexicographic shard-name order. A single-file snapshot therefore keeps
/// exactly the pre-#598 κ (the hash of `model.safetensors` alone); the
/// index bytes are bound by the #597 manifest, not by this κ.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct SafetensorsSnapshot {
    /// Shards in lexicographic file-name order.
    shards: Vec<Shard>,
    /// Tensor name → index into `shards`.
    resolved: std::collections::BTreeMap<String, usize>,
    /// Where a geometry-required tensor is reported missing from: the index
    /// file name in indexed mode, the single file name otherwise.
    origin: String,
    kappa: String,
    source_bytes: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl SafetensorsSnapshot {
    /// Open and validate a snapshot directory. See the type-level docs for
    /// the full contract; this is the one deterministic #598 validation
    /// boundary, and it fails BEFORE any model construction.
    pub fn open(
        dir: impl AsRef<std::path::Path>,
        required: &[TensorRequirement],
    ) -> Result<Self, SourceUnavailable> {
        use std::collections::{BTreeMap, BTreeSet};
        let dir = dir.as_ref();
        let index_path = dir.join(SAFETENSORS_INDEX_FILE_NAME);

        // Resolve the shard set: parse the index, or synthesize the
        // single-file one-shard resolution.
        let (index_map, shard_names, origin) = if index_path.is_file() {
            let bytes = std::fs::read(&index_path)
                .map_err(|error| unreadable(&index_path.display().to_string(), &error))?;
            let raw: RawShardIndex = serde_json::from_slice(&bytes)
                .map_err(|error| unreadable(&index_path.display().to_string(), &error))?;
            let mut map = BTreeMap::new();
            let mut names = BTreeSet::new();
            for (tensor, value) in raw.weight_map.0 {
                let Some(shard) = value.as_str() else {
                    return Err(unreadable(
                        &index_path.display().to_string(),
                        &format!("weight_map entry {tensor} is not a string"),
                    )
                    .into());
                };
                if shard.is_empty() || shard.contains(['/', '\\']) || shard.contains("..") {
                    return Err(unreadable(
                        &index_path.display().to_string(),
                        &format!("shard name {shard:?} is not a bare file name"),
                    )
                    .into());
                }
                names.insert(shard.to_owned());
                if let Some(previous) = map.insert(tensor.clone(), shard.to_owned()) {
                    return Err(SourceIngestKind::DuplicateTensor {
                        tensor,
                        first_shard: previous,
                        second_shard: shard.to_owned(),
                    }
                    .into());
                }
            }
            (Some(map), names, SAFETENSORS_INDEX_FILE_NAME.to_owned())
        } else {
            let mut names = BTreeSet::new();
            names.insert(SAFETENSORS_SINGLE_FILE_NAME.to_owned());
            (None, names, SAFETENSORS_SINGLE_FILE_NAME.to_owned())
        };

        // Pass 1 — load every shard and validate its header in isolation.
        let mut shards = Vec::with_capacity(shard_names.len());
        for name in &shard_names {
            let path = dir.join(name);
            if !path.is_file() {
                return Err(SourceIngestKind::MissingShardFile {
                    shard: name.clone(),
                }
                .into());
            }
            let bytes = crate::progress::read_file(&path, "loading Safetensors")
                .map_err(|error| unreadable(&path.display().to_string(), &error.reason))?;
            shards.push(parse_and_validate_shard(name, bytes)?);
        }

        // Pass 2 — global resolution: every tensor to exactly one shard.
        let mut resolved: BTreeMap<String, usize> = BTreeMap::new();
        for (index, shard) in shards.iter().enumerate() {
            for (tensor, meta) in &shard.tensors {
                if let Some(&previous) = resolved.get(tensor) {
                    let previous_shard = &shards[previous];
                    let previous_meta = &previous_shard.tensors[tensor];
                    if previous_meta.dtype != meta.dtype {
                        return Err(SourceIngestKind::DtypeInconsistency {
                            tensor: tensor.clone(),
                            first: format!(
                                "{} (in {})",
                                previous_meta.dtype.name(),
                                previous_shard.name
                            ),
                            second: format!("{} (in {})", meta.dtype.name(), shard.name),
                        }
                        .into());
                    }
                    return Err(SourceIngestKind::DuplicateTensor {
                        tensor: tensor.clone(),
                        first_shard: previous_shard.name.clone(),
                        second_shard: shard.name.clone(),
                    }
                    .into());
                }
                resolved.insert(tensor.clone(), index);
            }
        }

        // Pass 3 — cross-check the index against the shard headers.
        if let Some(index_map) = &index_map {
            for (tensor, shard_name) in index_map {
                let missing = match resolved.get(tensor) {
                    Some(&index) => &shards[index].name != shard_name,
                    None => true,
                };
                if missing {
                    return Err(SourceIngestKind::MissingTensor {
                        tensor: tensor.clone(),
                        shard: shard_name.clone(),
                    }
                    .into());
                }
            }
            for shard in &shards {
                for tensor in shard.tensors.keys() {
                    if !index_map.contains_key(tensor) {
                        return Err(SourceIngestKind::UnexpectedTensor {
                            tensor: tensor.clone(),
                            shard: shard.name.clone(),
                        }
                        .into());
                    }
                }
            }
        }

        // Pass 4 — the model geometry's requirements: presence and shape.
        // (Dtype support was already validated for every declared tensor.)
        for requirement in required {
            let Some(&index) = resolved.get(&requirement.name) else {
                return Err(SourceIngestKind::MissingTensor {
                    tensor: requirement.name.clone(),
                    shard: origin.clone(),
                }
                .into());
            };
            let meta = &shards[index].tensors[&requirement.name];
            if meta.shape != requirement.shape {
                return Err(SourceIngestKind::ShapeMismatch {
                    tensor: requirement.name.clone(),
                    expected: requirement.shape.clone(),
                    actual: meta.shape.clone(),
                }
                .into());
            }
        }

        // Identity: blake3 over concatenated shard bytes in name order.
        // With one shard (the single-file layout) this is exactly the
        // pre-#598 κ of `model.safetensors`.
        let mut hasher = blake3::Hasher::new();
        let mut source_bytes = 0usize;
        for shard in &shards {
            hasher.update(&shard.bytes);
            source_bytes += shard.bytes.len();
        }
        let kappa = format!("blake3:{}", hasher.finalize().to_hex());
        Ok(Self {
            shards,
            resolved,
            origin,
            kappa,
            source_bytes,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SafetensorsSnapshot {
    /// The snapshot κ: `blake3:<hex>` over the concatenated shard bytes in
    /// lexicographic shard-name order (single-file: the file's own hash,
    /// unchanged from before #598).
    pub fn kappa(&self) -> &str {
        &self.kappa
    }

    /// Total bytes across all shard files.
    pub fn source_bytes(&self) -> usize {
        self.source_bytes
    }

    /// Append tensor `name`, widened exactly to `f32` through [`codec`],
    /// to `out`. Fails only for a name outside the validated resolution.
    pub fn tensor_f32_into(&self, name: &str, out: &mut Vec<f32>) -> Result<(), SourceUnavailable> {
        let Some(&index) = self.resolved.get(name) else {
            return Err(SourceIngestKind::MissingTensor {
                tensor: name.to_owned(),
                shard: self.origin.clone(),
            }
            .into());
        };
        let shard = &self.shards[index];
        let meta = &shard.tensors[name];
        let data = &shard.bytes[shard.data_start + meta.begin..shard.data_start + meta.end];
        codec::append_widened(meta.dtype, data, out);
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unreadable(path: &str, reason: &impl std::fmt::Display) -> SourceIngestKind {
    SourceIngestKind::Unreadable {
        path: path.to_owned(),
        reason: reason.to_string(),
    }
}

/// Parse one shard's Safetensors header and validate it in isolation:
/// dtype support, tensor span vs shape × dtype size, span contiguity, and
/// file size vs header claim. Duplicate names within one header are
/// detected on the raw JSON (before map collapse).
#[cfg(not(target_arch = "wasm32"))]
fn parse_and_validate_shard(name: &str, bytes: Vec<u8>) -> Result<Shard, SourceUnavailable> {
    let file_len = bytes.len() as u64;
    if bytes.len() < 8 {
        return Err(SourceIngestKind::ByteLengthMismatch {
            context: format!("{name}: header length prefix"),
            expected: 8,
            actual: file_len,
        }
        .into());
    }
    let header_len = u64::from_le_bytes(bytes[..8].try_into().expect("eight bytes"));
    let data_start = header_len
        .checked_add(8)
        .filter(|start| *start <= file_len)
        .ok_or(SourceIngestKind::ByteLengthMismatch {
            context: format!("{name}: header"),
            expected: header_len.saturating_add(8),
            actual: file_len,
        })?;
    let header = std::str::from_utf8(&bytes[8..data_start as usize])
        .map_err(|error| unreadable(name, &error))?;
    let entries: MapEntries =
        serde_json::from_str(header).map_err(|error| unreadable(name, &error))?;

    let mut tensors: std::collections::BTreeMap<String, TensorMeta> =
        std::collections::BTreeMap::new();
    for (tensor, value) in entries.0 {
        if tensor == "__metadata__" {
            continue;
        }
        let info: RawTensorInfo = serde_json::from_value(value)
            .map_err(|error| unreadable(&format!("{name}: tensor {tensor}"), &error))?;
        let Some(dtype) = SourceDtype::classify(&info.dtype) else {
            return Err(SourceIngestKind::UnsupportedDtype {
                tensor,
                dtype: info.dtype,
            }
            .into());
        };
        let mut shape = Vec::with_capacity(info.shape.len());
        for &extent in &info.shape {
            let extent = usize::try_from(extent)
                .map_err(|error| unreadable(&format!("{name}: tensor {tensor}"), &error))?;
            shape.push(extent);
        }
        let elements = shape
            .iter()
            .copied()
            .try_fold(1usize, usize::checked_mul)
            .ok_or_else(|| {
                unreadable(
                    &format!("{name}: tensor {tensor}"),
                    &"shape element count overflows",
                )
            })?;
        let claimed = elements as u64 * dtype.byte_size() as u64;
        let (begin, end) = info.data_offsets;
        if end < begin || end - begin != claimed {
            return Err(SourceIngestKind::ByteLengthMismatch {
                context: format!("{name}: tensor {tensor} data length vs shape×dtype size"),
                expected: claimed,
                actual: end.saturating_sub(begin),
            }
            .into());
        }
        let meta = TensorMeta {
            dtype,
            shape,
            begin: usize::try_from(begin)
                .map_err(|error| unreadable(&format!("{name}: tensor {tensor}"), &error))?,
            end: usize::try_from(end)
                .map_err(|error| unreadable(&format!("{name}: tensor {tensor}"), &error))?,
        };
        if tensors.insert(tensor.clone(), meta).is_some() {
            return Err(SourceIngestKind::DuplicateTensor {
                tensor,
                first_shard: name.to_owned(),
                second_shard: name.to_owned(),
            }
            .into());
        }
    }

    // Spans must tile the data section contiguously from zero.
    let mut spans: Vec<(usize, usize, &String)> = tensors
        .iter()
        .map(|(tensor, meta)| (meta.begin, meta.end, tensor))
        .collect();
    spans.sort_unstable();
    let mut cursor = 0usize;
    for (begin, end, tensor) in spans {
        if begin != cursor {
            return Err(SourceIngestKind::ByteLengthMismatch {
                context: format!("{name}: tensor {tensor} data offsets are not contiguous"),
                expected: cursor as u64,
                actual: begin as u64,
            }
            .into());
        }
        cursor = end;
    }
    let claimed_file_len = data_start + cursor as u64;
    if claimed_file_len != file_len {
        return Err(SourceIngestKind::ByteLengthMismatch {
            context: format!("{name}: file size vs header claim"),
            expected: claimed_file_len,
            actual: file_len,
        }
        .into());
    }
    Ok(Shard {
        name: name.to_owned(),
        bytes,
        data_start: data_start as usize,
        tensors,
    })
}

/// The tensors (name + shape) a Llama-family teacher requires, in the
/// flattened append order `load_inner` uses. Derived entirely from the
/// `config.json` geometry, so shape validation happens at the #598 boundary
/// before the model is constructed.
#[cfg(not(target_arch = "wasm32"))]
fn required_llama_tensors(cfg: &Config, tie_word_embeddings: bool) -> Vec<TensorRequirement> {
    let (dim, hidden, vocab) = (cfg.dim, cfg.hidden, cfg.vocab);
    let kv_dim = cfg.dim * cfg.n_kv_heads / cfg.n_heads;
    let mut required = Vec::new();
    let mut push = |name: String, shape: Vec<usize>| {
        required.push(TensorRequirement { name, shape });
    };
    push("model.embed_tokens.weight".to_owned(), vec![vocab, dim]);
    for (suffix, shape) in [
        ("input_layernorm.weight", vec![dim]),
        ("self_attn.q_proj.weight", vec![dim, dim]),
        ("self_attn.k_proj.weight", vec![kv_dim, dim]),
        ("self_attn.v_proj.weight", vec![kv_dim, dim]),
        ("self_attn.o_proj.weight", vec![dim, dim]),
        ("post_attention_layernorm.weight", vec![dim]),
        ("mlp.gate_proj.weight", vec![hidden, dim]),
        ("mlp.down_proj.weight", vec![dim, hidden]),
        ("mlp.up_proj.weight", vec![hidden, dim]),
    ] {
        for layer in 0..cfg.n_layers {
            push(format!("model.layers.{layer}.{suffix}"), shape.clone());
        }
    }
    push("model.norm.weight".to_owned(), vec![dim]);
    if !tie_word_embeddings {
        push("lm_head.weight".to_owned(), vec![vocab, dim]);
    }
    required
}

/// A teacher that can advance a batch of independent sequences one position
/// each through a single memory-amortized forward. Implemented by the HF oracle
/// (the deployed teacher) and mockable for tests, so the batched observe driver
/// need not name a concrete oracle.
pub trait BatchedTeacher {
    /// The per-sequence decode state this teacher advances. Generic so a
    /// non-Llama architecture (GPT-2) can carry its own state through the
    /// same batched observe driver instead of the Llama `State`.
    type State;
    /// A fresh per-sequence state for this teacher.
    fn new_state(&self) -> Self::State;
    /// A state bounded to an actual prompt/generation position count.
    /// Implementations that cannot allocate a smaller private state refuse
    /// rather than silently allocating their model maximum.
    fn new_state_bounded(
        &self,
        sequence_capacity: usize,
    ) -> Result<Self::State, TeacherStateCapacityError> {
        if sequence_capacity == 0 {
            return Err(TeacherStateCapacityError::Zero);
        }
        if sequence_capacity > self.seq_len() {
            return Err(TeacherStateCapacityError::ExceedsModel {
                requested: sequence_capacity,
                maximum: self.seq_len(),
            });
        }
        if sequence_capacity != self.seq_len() {
            return Err(TeacherStateCapacityError::BoundedAllocationUnavailable {
                requested: sequence_capacity,
                model: self.seq_len(),
            });
        }
        Ok(self.new_state())
    }
    /// Reset a state to begin a new sequence (zero its caches/buffers).
    fn reset_state(&self, state: &mut Self::State);
    /// Mutable view of the logits the last
    /// [`BatchedTeacher::forward_batch_into`] left in `state` — the observe
    /// driver encodes each position in place.
    fn logits_mut<'a>(&self, state: &'a mut Self::State) -> &'a mut [f32];
    /// Maximum context length (teacher-forced positions per article).
    fn seq_len(&self) -> usize;
    /// Vocabulary size (logits length).
    fn vocab(&self) -> usize;
    /// The source-to-compiled geometry identity applied by this batched
    /// teacher, when one exists. The default keeps external/mock teachers
    /// compatible while allowing the observation boundary to bind shipped
    /// producers before writing rows.
    fn geometry_projection(&self) -> Option<geometry::GeometryProjection> {
        None
    }
    /// The registered source-attention identity executed by this batched
    /// teacher. A producer that returns `None` may create only an unbound
    /// legacy corpus and cannot resume one with an explicit recorded era.
    fn attention_operator_spec(&self) -> Option<attention::AttentionOperatorSpec> {
        None
    }
    /// Registered source-dense identity executed by this batched producer.
    /// `None` denotes an unbound or inapplicable source family.
    fn dense_operator_spec(&self) -> Option<dense::DenseOperatorSpec> {
        None
    }
    /// Advance `states.len()` sequences one position each: sequence `b` steps
    /// `tokens[b]` at `positions[b]`, leaving its logits reachable through
    /// [`BatchedTeacher::logits_mut`].
    fn forward_batch_into(&self, states: &mut [Self::State], tokens: &[usize], positions: &[usize]);
}

impl BatchedTeacher for HuggingFaceLlamaOracle {
    type State = State;
    fn new_state(&self) -> State {
        State::new(&self.model.cfg)
    }
    fn new_state_bounded(
        &self,
        sequence_capacity: usize,
    ) -> Result<State, TeacherStateCapacityError> {
        State::new_bounded(&self.model.cfg, sequence_capacity)
    }
    fn reset_state(&self, state: &mut State) {
        state.reset();
    }
    fn logits_mut<'a>(&self, state: &'a mut State) -> &'a mut [f32] {
        &mut state.logits
    }
    fn seq_len(&self) -> usize {
        self.model.cfg.seq_len
    }
    fn vocab(&self) -> usize {
        self.model.cfg.vocab
    }
    fn geometry_projection(&self) -> Option<geometry::GeometryProjection> {
        <Self as TeacherOracle>::geometry_projection(self)
    }
    fn attention_operator_spec(&self) -> Option<attention::AttentionOperatorSpec> {
        <Self as TeacherOracle>::attention_operator_spec(self)
    }
    fn dense_operator_spec(&self) -> Option<dense::DenseOperatorSpec> {
        <Self as TeacherOracle>::dense_operator_spec(self)
    }
    fn forward_batch_into(&self, states: &mut [State], tokens: &[usize], positions: &[usize]) {
        self.model
            .forward_batch(states, tokens, positions, self.fast_matmul);
    }
}

impl HuggingFaceLlamaOracle {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(source: impl AsRef<std::path::Path>) -> Result<Self, SourceUnavailable> {
        Self::load_inner(source, None, TeacherExecutionConfig::default())
    }

    /// Load a teacher with an explicit bounded exact-execution policy.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_with_execution(
        source: impl AsRef<std::path::Path>,
        execution: TeacherExecutionConfig,
    ) -> Result<Self, SourceUnavailable> {
        Self::load_inner(source, None, execution)
    }

    /// This teacher's configuration (dims, heads, vocab, sequence length).
    pub fn cfg(&self) -> &Config {
        &self.model.cfg
    }

    /// Content identity of the exact source weights backing this oracle.
    pub fn source_cid(&self) -> &str {
        &self.kappa
    }

    /// Allocate one private sequence state at the actual prompt/generation
    /// horizon rather than the model's maximum context.
    pub fn new_state_bounded(
        &self,
        sequence_capacity: usize,
    ) -> Result<State, TeacherStateCapacityError> {
        State::new_bounded(&self.model.cfg, sequence_capacity)
    }

    /// Construct an independent full decoder whose selected attention layers
    /// use an injected coordinate transport over every causal source.
    ///
    /// The source checkpoint continues to own Q/K/V and Wo, RoPE,
    /// residual/FFN blocks, and the LM head. Default row hooks retain its
    /// stable softmax and linear value aggregate; an implementation may
    /// replace those two operations and must bind that policy in its evidence.
    /// The session fails closed unless each head is a non-zero exact sequence
    /// of four-lane R4 blocks.
    pub fn new_causal_attention_transport_session(
        &self,
        mut transport: Box<dyn attention::CausalAttentionTransport>,
        selection: attention::CausalAttentionLayerSelection,
        sequence_capacity: usize,
    ) -> Result<CausalAttentionTransportSession, attention::CausalAttentionTransportError> {
        use attention::CausalAttentionTransportError as Error;

        let config = &self.model.cfg;
        if config.n_heads == 0 || config.dim == 0 || !config.dim.is_multiple_of(config.n_heads) {
            return Err(Error::InvalidHeadLayout {
                dimension: config.dim,
                heads: config.n_heads,
            });
        }
        if config.n_kv_heads == 0 || !config.n_heads.is_multiple_of(config.n_kv_heads) {
            return Err(Error::InvalidGroupedQueryLayout {
                query_heads: config.n_heads,
                kv_heads: config.n_kv_heads,
            });
        }
        let head_size = config.dim / config.n_heads;
        if head_size == 0 || !head_size.is_multiple_of(4) {
            return Err(Error::HeadSizeNotDivisibleByFour { head_size });
        }
        let selected_layers = causal_attention_layer_mask(selection, config.n_layers)?;
        let state = State::new_bounded(config, sequence_capacity)
            .map_err(|error| Error::SequenceCapacity(error.to_string()))?;
        let scratch = CausalAttentionTransportScratch::new(head_size, sequence_capacity)?;
        transport.reset();
        require_healthy_causal_attention_transport(transport.as_ref())?;
        Ok(CausalAttentionTransportSession {
            state,
            transport,
            selected_layers,
            scratch,
            audit: attention::CausalAttentionTransportAudit::default(),
            pre_rope_projection_audit: attention::CausalAttentionProjectionAudit::default(),
            source_cid: self.kappa.clone(),
            next_position: 0,
        })
    }

    /// Advance one full-prefix causal attention operator session by exactly
    /// one token and copy its resulting full-decoder logits to `logits`.
    ///
    /// Positions are deliberately sequential: cumulative geometric frames
    /// and the source KV cache must admit the same causal history.
    pub fn step_causal_attention_transport(
        &self,
        session: &mut CausalAttentionTransportSession,
        token: usize,
        position: usize,
        logits: &mut [f32],
    ) -> Result<(), attention::CausalAttentionTransportError> {
        use attention::CausalAttentionTransportError as Error;

        if session.source_cid != self.kappa {
            return Err(Error::SourceBindingMismatch);
        }
        require_healthy_causal_attention_transport(session.transport.as_ref())?;
        if token >= self.model.cfg.vocab {
            return Err(Error::TokenOutOfRange(token));
        }
        let capacity = session.state.sequence_capacity();
        if position >= capacity {
            return Err(Error::PositionOutOfRange { position, capacity });
        }
        if position != session.next_position {
            return Err(Error::PositionOutOfOrder {
                requested: position,
                expected: session.next_position,
            });
        }
        if logits.len() != self.model.cfg.vocab {
            return Err(Error::LogitShape {
                requested: logits.len(),
                expected: self.model.cfg.vocab,
            });
        }
        self.model
            .forward_causal_attention_transport(session, token, position, self.fast_matmul);
        require_healthy_causal_attention_transport(session.transport.as_ref())?;
        logits.copy_from_slice(&session.state.logits);
        session.next_position = position + 1;
        Ok(())
    }

    /// Construct one independent experimental decoder arm.  Persistent
    /// memory must already carry the exact tokenizer and deterministic
    /// adapter identity; both are revalidated here against the checkpoint.
    pub fn new_geometric_session(
        &self,
        mixer: geometric_decoder::GeometricMixer,
        context: geometric_decoder::GeometryContext,
        intervention: geometric_decoder::GeometryIntervention,
        sequence_capacity: usize,
    ) -> Result<geometric_decoder::GeometricDecoderSession, geometric_decoder::GeometricDecoderError>
    {
        use geometric_decoder::GeometricDecoderError as Error;

        if mixer.source_width != self.model.cfg.dim {
            return Err(Error::InvalidSourceWidth(mixer.source_width));
        }
        if mixer.layer >= self.model.cfg.n_layers {
            return Err(Error::TargetLayerOutOfRange {
                requested: mixer.layer,
                layers: self.model.cfg.n_layers,
            });
        }
        if context.provenance.source_cid != self.kappa {
            return Err(Error::SourceBindingMismatch);
        }
        let expected_adapter = mixer.memory_adapter_identity(&self.kappa, &context.tokenizer_cid);
        if context.adapter_identity != expected_adapter {
            return Err(Error::AdapterBindingMismatch);
        }
        let state = State::new_bounded(&self.model.cfg, sequence_capacity)
            .map_err(|error| Error::SequenceCapacity(error.to_string()))?;
        let dim = self.model.cfg.dim;
        let vocabulary = self.model.cfg.vocab;
        let embeddings =
            &self.model.w[self.model.emb..self.model.emb + vocabulary.saturating_mul(dim)];
        let runtime = geometric_decoder::GeometricRuntime::prepare(
            mixer,
            context,
            intervention,
            sequence_capacity,
            &self.model.exact_executor,
            embeddings,
            vocabulary,
        )?;
        Ok(geometric_decoder::GeometricDecoderSession { state, runtime })
    }

    /// Advance a caller-owned ordinary source state through the exact local
    /// control path.  Unlike the historical trait method, this focused G0
    /// surface validates token/position/output bounds instead of panicking.
    pub fn step_state(
        &self,
        state: &mut State,
        token: usize,
        position: usize,
        logits: &mut [f32],
    ) -> Result<(), geometric_decoder::GeometricDecoderError> {
        self.validate_session_step(state.sequence_capacity(), token, position, logits.len())?;
        self.model.forward(state, token, position, self.fast_matmul);
        logits.copy_from_slice(&state.logits);
        Ok(())
    }

    /// Advance a caller-owned treatment arm through the one-layer seam.
    pub fn step_geometric(
        &self,
        session: &mut geometric_decoder::GeometricDecoderSession,
        token: usize,
        position: usize,
        logits: &mut [f32],
    ) -> Result<(), geometric_decoder::GeometricDecoderError> {
        self.validate_session_step(
            session.state.sequence_capacity(),
            token,
            position,
            logits.len(),
        )?;
        self.model.forward_geometric(
            &mut session.state,
            token,
            position,
            self.fast_matmul,
            &mut session.runtime,
        );
        logits.copy_from_slice(&session.state.logits);
        Ok(())
    }

    fn validate_session_step(
        &self,
        capacity: usize,
        token: usize,
        position: usize,
        logits: usize,
    ) -> Result<(), geometric_decoder::GeometricDecoderError> {
        use geometric_decoder::GeometricDecoderError as Error;

        if token >= self.model.cfg.vocab {
            return Err(Error::TokenOutOfRange(token));
        }
        if position >= capacity {
            return Err(Error::PositionOutOfRange { position, capacity });
        }
        if logits != self.model.cfg.vocab {
            return Err(Error::LogitShape {
                requested: logits,
                expected: self.model.cfg.vocab,
            });
        }
        Ok(())
    }

    /// Load an offline teacher with a bounded context allocation. Compilation
    /// only needs short trajectories because the deployed runtime consumes an
    /// eight-token window; bounding teacher stories avoids quadratic attention
    /// work at source-model maximum context lengths.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_with_sequence_length(
        source: impl AsRef<std::path::Path>,
        sequence_length: usize,
    ) -> Result<Self, SourceUnavailable> {
        if sequence_length == 0 {
            return Err(SourceUnavailable::new(
                "teacher sequence length must be greater than zero",
            ));
        }
        Self::load_inner(
            source,
            Some(sequence_length),
            TeacherExecutionConfig::default(),
        )
    }

    /// Load a teacher with both bounded context storage and an explicit
    /// bounded exact-execution policy.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_with_sequence_length_and_execution(
        source: impl AsRef<std::path::Path>,
        sequence_length: usize,
        execution: TeacherExecutionConfig,
    ) -> Result<Self, SourceUnavailable> {
        if sequence_length == 0 {
            return Err(SourceUnavailable::new(
                "teacher sequence length must be greater than zero",
            ));
        }
        Self::load_inner(source, Some(sequence_length), execution)
    }

    /// Replace only the exact execution policy, retaining the loaded weights.
    ///
    /// Exclusive access prevents a pool replacement from racing a forward or
    /// [`HuggingFaceLlamaOracle::set_r4_attention`] configuration mutation.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_execution_config(
        &mut self,
        execution: TeacherExecutionConfig,
    ) -> Result<(), SourceUnavailable> {
        self.model.set_execution_config(execution)
    }

    /// Reset exact counters after excluded executor prestart while retaining
    /// the same dedicated worker pool, then install the measured-run observer.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn begin_measured_execution(&mut self, observer: TeacherExecutionObserver) {
        self.model.begin_measured_execution(observer);
    }

    /// Prepare all shape-bounded model/executor workspaces, wake the dedicated
    /// pool, and exercise a tiny exact GEMM without a model forward. Call
    /// [`Self::begin_measured_execution`] afterwards to exclude preparation
    /// growth and wall time from measured teacher work.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn prepare_exact_execution(
        &self,
        batch_width: usize,
    ) -> Result<TeacherExecutionPreparation, SourceUnavailable> {
        self.model.prestart_exact_execution(batch_width)
    }

    /// Current exact-execution progress and bounded-concurrency evidence.
    pub fn execution_snapshot(&self) -> TeacherExecutionSnapshot {
        self.model.execution_snapshot()
    }

    /// Exact counter plan for one forward at `batch_width` under the current
    /// model geometry and executor tiling.
    pub fn exact_forward_plan(
        &self,
        batch_width: usize,
    ) -> Result<ExactForwardPlan, ExactForwardPlanError> {
        self.model.exact_forward_plan(batch_width)
    }

    /// Complete raw-output/state trace dimensions for a bounded probe.
    pub fn exact_probe_trace_shape(
        &self,
        positions: usize,
        batch_width: usize,
        top_k: usize,
    ) -> Result<ExactMulticoreProbeTraceShape, ExactForwardPlanError> {
        self.model
            .exact_probe_trace_shape(positions, batch_width, top_k)
    }

    /// Complete trace dimensions for explicitly bounded private states.
    pub fn exact_probe_trace_shape_bounded(
        &self,
        sequence_capacity: usize,
        positions: usize,
        batch_width: usize,
        top_k: usize,
    ) -> Result<ExactMulticoreProbeTraceShape, ExactForwardPlanError> {
        self.model
            .exact_probe_trace_shape_bounded(sequence_capacity, positions, batch_width, top_k)
    }

    /// Exact arithmetic owner and observable hosted-kernel availability.
    pub fn exact_backend_report(&self) -> ExactBackendReport {
        exact_backend_report()
    }

    /// Enable or disable the experimental attention variant
    /// (`experimental-r4-source-attention/2`, #602/#704): a correctly-rounded
    /// exact-real dot over the floor-multiple-of-four head prefix — truncating
    /// the trailing `head_size mod 4` dimensions — followed by the same
    /// softmax the standard operator applies. Despite the flag's historical
    /// name it is neither quaternionic nor softmax-bypassing (#515 audit).
    pub fn set_r4_attention(&mut self, enable: bool) {
        self.model.cfg.r4_attention = enable;
    }

    /// Check if the experimental attention variant
    /// (`experimental-r4-source-attention/2`) is enabled.
    pub fn r4_attention(&self) -> bool {
        self.model.cfg.r4_attention
    }

    /// One teacher step with the residual stream captured after each layer
    /// in `capture_layers` (#599 conformance trace). Delegates to the exact
    /// executor path [`Llama::forward_capturing`] with this oracle's own
    /// matmul selection, so a traced step and a plain [`BehaviorSource::step`]
    /// produce identical state, logits, and top-k.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn step_with_layer_capture(
        &mut self,
        token: usize,
        pos: usize,
        capture_layers: &[usize],
        sink: &mut dyn FnMut(usize, &[f32]),
    ) {
        self.model.forward_capturing(
            &mut self.state,
            token,
            pos,
            self.fast_matmul,
            capture_layers,
            sink,
        );
    }

    /// The logits the last step (plain or capturing) left in this oracle's
    /// state (#599 conformance trace surface).
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn last_logits(&self) -> &[f32] {
        &self.state.logits
    }

    /// Load the teacher from a snapshot directory: `config.json` plus the
    /// weights as either a single `model.safetensors` or #598 indexed
    /// shards (`model.safetensors.index.json` + shard files). Both layouts
    /// flow through the one [`SafetensorsSnapshot::open`] validation
    /// boundary — resolution, shape, byte-length, and dtype checks all fail
    /// there, before any model construction — and tensors are widened
    /// exactly from BF16/F16/F32 to `f32` through [`codec`]. κ is blake3
    /// over the shard bytes in shard-name order, so single-file snapshots
    /// keep their pre-#598 κ unchanged.
    #[cfg(not(target_arch = "wasm32"))]
    fn load_inner(
        source: impl AsRef<std::path::Path>,
        sequence_length: Option<usize>,
        execution: TeacherExecutionConfig,
    ) -> Result<Self, SourceUnavailable> {
        let source = source.as_ref();
        let config_bytes = std::fs::read(source.join("config.json"))?;
        // #599: fail-closed adapter-conformance gate. The raw configuration
        // is validated against this adapter's typed feature declaration
        // BEFORE the typed parse, before the #598 snapshot boundary, and
        // therefore before any tensor is read or observation generated. Any
        // feature outside the declaration is a focused
        // [`SourceIngestKind::UnsupportedConfigFeature`] failure.
        let raw_config: serde_json::Value = serde_json::from_slice(&config_bytes)?;
        conformance::AdapterFeatures::huggingface_llama().validate_config(&raw_config)?;
        let config: HuggingFaceConfig = serde_json::from_slice(&config_bytes)?;
        let cfg = Config {
            dim: config.hidden_size,
            hidden: config.intermediate_size,
            n_layers: config.num_hidden_layers,
            n_heads: config.num_attention_heads,
            n_kv_heads: config.num_key_value_heads,
            vocab: config.vocab_size,
            seq_len: sequence_length
                .unwrap_or(config.max_position_embeddings)
                .min(config.max_position_embeddings),
            rope_theta: config.rope_theta,
            rms_norm_eps: config.rms_norm_eps,
            rope_interleaved: config.rope_interleaved,
            r4_attention: false,
        };
        eprintln!(
            "model geometry: vocab={} hidden={} layers={} heads={} kv_heads={}",
            cfg.vocab, cfg.dim, cfg.n_layers, cfg.n_heads, cfg.n_kv_heads
        );
        // #598: one validation boundary for single-file and indexed sharded
        // snapshots; every failure class errors here, before construction.
        let required = required_llama_tensors(&cfg, config.tie_word_embeddings);
        let snapshot = SafetensorsSnapshot::open(source, &required)?;
        eprintln!("widening source tensors (BF16/F16/F32) to the compiler teacher layout...");
        let flattened_len = required
            .iter()
            .map(|requirement| requirement.shape.iter().product::<usize>())
            .sum();
        let mut weights = Vec::with_capacity(flattened_len);
        append_tensor(&snapshot, "model.embed_tokens.weight", &mut weights)?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "input_layernorm.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "self_attn.q_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "self_attn.k_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "self_attn.v_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "self_attn.o_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "post_attention_layernorm.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "mlp.gate_proj.weight",
            &mut weights,
        )?;
        append_layers(
            &snapshot,
            cfg.n_layers,
            "mlp.down_proj.weight",
            &mut weights,
        )?;
        append_layers(&snapshot, cfg.n_layers, "mlp.up_proj.weight", &mut weights)?;
        append_tensor(&snapshot, "model.norm.weight", &mut weights)?;
        if !config.tie_word_embeddings {
            append_tensor(&snapshot, "lm_head.weight", &mut weights)?;
        }
        let kappa = snapshot.kappa().to_owned();
        let source_bytes = snapshot.source_bytes();
        let canonical_math =
            std::env::var("TLESS_CANONICAL_DETERMINISTIC").is_ok_and(|value| value != "0");
        let mut model = Llama::from_flat(cfg, weights, config.tie_word_embeddings);
        model
            .set_execution_config(execution)
            .map_err(SourceUnavailable::new)?;
        model.canonical_math = canonical_math;
        // `from_flat` builds the fast native-math cache first; rebuild it if
        // D2 canonical math was requested so the cache uses the same libm
        // implementation as the rest of the forward pass.
        if canonical_math {
            model.rebuild_rope_cache();
        }
        let state = State::new(&model.cfg);
        let fast_matmul = !canonical_math && std::env::var("TLESS_EXACT_SCALAR").is_err();
        let backend = if canonical_math {
            "uor-matmul exact GEMM + canonical libm scalar (D2)"
        } else {
            fast_matmul_backend()
        };
        eprintln!(
            "teacher model ready (κ {kappa}, matmul={backend}, exact_workers={})",
            model.execution_snapshot().effective_workers
        );
        Ok(Self {
            model,
            state,
            kappa,
            source_bytes,
            bos_token: config.bos_token_id,
            eos_token: config.eos_token_id,
            fast_matmul,
        })
    }
}

/// Append one tensor per layer (`model.layers.<n>.<suffix>`) from the
/// validated snapshot, in layer order.
#[cfg(not(target_arch = "wasm32"))]
fn append_layers(
    snapshot: &SafetensorsSnapshot,
    layers: usize,
    suffix: &str,
    out: &mut Vec<f32>,
) -> Result<(), SourceUnavailable> {
    for layer in 0..layers {
        append_tensor(snapshot, &format!("model.layers.{layer}.{suffix}"), out)?;
    }
    Ok(())
}

/// Append tensor `name` from the validated snapshot, widened exactly from
/// its declared source dtype (BF16, F16, or F32) to `f32` through
/// [`codec`]. Dtype support, shape, and byte lengths were already checked
/// at the [`SafetensorsSnapshot::open`] boundary (#598), so this only fails
/// for a name outside the validated resolution.
#[cfg(not(target_arch = "wasm32"))]
fn append_tensor(
    snapshot: &SafetensorsSnapshot,
    name: &str,
    out: &mut Vec<f32>,
) -> Result<(), SourceUnavailable> {
    snapshot.tensor_f32_into(name, out)
}

impl RepresentationSource for HuggingFaceLlamaOracle {
    fn vocab_size(&self) -> usize {
        self.model.cfg.vocab
    }
    fn source_dimension(&self) -> usize {
        self.model.cfg.dim
    }
    fn tokenizer_address(&self) -> &str {
        "huggingface-tokenizer"
    }
    fn read_embedding_rows(&self, range: std::ops::Range<usize>, output: &mut [f32]) -> Option<()> {
        let d = self.model.cfg.dim;
        let count = range.end - range.start;
        if output.len() < count * d {
            return None;
        }
        let start_offset = self.model.emb + range.start * d;
        let end_offset = self.model.emb + range.end * d;
        output[..count * d].copy_from_slice(&self.model.w[start_offset..end_offset]);
        Some(())
    }
}

impl BehaviorSource for HuggingFaceLlamaOracle {
    fn reset(&mut self) {
        self.state.reset();
    }
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]) {
        self.model
            .forward(&mut self.state, token, pos, self.fast_matmul);
        logits.copy_from_slice(&self.state.logits);
    }
}

impl TeacherOracle for HuggingFaceLlamaOracle {
    fn vocab(&self) -> usize {
        self.model.cfg.vocab
    }
    fn dim(&self) -> usize {
        // The compiled geometry this adapter presents (D = 288), not the
        // source width; `embedding` projects rows down to it (#600).
        geometry::COMPILED_WIDTH as usize
    }
    fn seq_len(&self) -> usize {
        self.model.cfg.seq_len
    }
    fn bos_token(&self) -> usize {
        self.bos_token
    }
    fn eos_token(&self) -> usize {
        self.eos_token
    }
    fn kappa(&self) -> String {
        self.kappa.clone()
    }
    fn source_bytes(&self) -> usize {
        self.source_bytes
    }
    fn embedding(&self, token: usize, out: &mut [f32]) {
        let dim = self.model.cfg.dim;
        let row = &self.model.w[self.model.emb + token * dim..self.model.emb + (token + 1) * dim];
        // #600: the explicitly named `bucket-average/1` projection — the
        // exact arithmetic this method always performed, factored into
        // the versioned geometry module (see `geometry_projection`).
        geometry::bucket_average_project(row, out);
    }
    fn geometry_projection(&self) -> Option<geometry::GeometryProjection> {
        u32::try_from(self.model.cfg.dim).ok().map(|source_width| {
            geometry::GeometryProjection::bucket_average(source_width, geometry::COMPILED_WIDTH)
        })
    }
    fn attention_operator_spec(&self) -> Option<attention::AttentionOperatorSpec> {
        // #602: the boolean switch maps to exactly the two registered
        // operators — this is the one boundary where the legacy flag
        // becomes a versioned identity.
        Some(attention::operator_for_r4_switch(
            self.model.cfg.r4_attention,
        ))
    }
    fn hidden_state(&self) -> Option<&[f32]> {
        Some(&self.state.x)
    }
    fn top_k(&self, k: usize, out: &mut [(u32, f32)]) -> usize {
        top_k_from_logits(&self.state.logits, k, out, self.model.canonical_math)
    }
    fn trace_capture_geometry(&self) -> Option<TraceCaptureGeometry> {
        Some(TraceCaptureGeometry {
            layers: self.model.cfg.n_layers,
            heads: self.model.cfg.n_heads,
            kv_heads: self.model.cfg.n_kv_heads,
            residual_width: self.model.cfg.dim,
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn step_with_trace_capture(
        &mut self,
        token: usize,
        pos: usize,
        logits: &mut [f32],
        request: &TraceCaptureRequest<'_>,
        sinks: &mut TraceCaptureSinks<'_, '_>,
    ) -> bool {
        // This oracle's own matmul selection, exactly as
        // `step_with_layer_capture` (#599) delegates it, so a traced
        // step and a plain step produce identical bits.
        self.model.forward_capturing_trace(
            &mut self.state,
            token,
            pos,
            self.fast_matmul,
            request,
            sinks,
        );
        logits.copy_from_slice(&self.state.logits);
        true
    }
}

/// Backward-compatible name for the first supported Hugging Face model.
pub type SmolLm2Oracle = HuggingFaceLlamaOracle;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn unknown_tokenizer_adapter_diagnostic_names_current_registry() {
        let error: SourceUnavailable = SourceIngestKind::UnknownTokenizerAdapter {
            family: "unknown-family".to_owned(),
            version: 9,
        }
        .into();
        assert!(matches!(
            error.kind,
            SourceIngestKind::UnknownTokenizerAdapter {
                ref family,
                version: 9,
            } if family == "unknown-family"
        ));
        assert!(error.reason.contains("hf-byte-bpe/1"));
        assert!(error.reason.contains("sentencepiece-unigram/1"));
        assert!(error.reason.contains("sentencepiece-unigram/2"));
        assert!(!error.reason.contains("stays rejected"));
    }

    #[test]
    fn canonical_math_delegates_to_portable_libm() {
        let values = [-7.25f32, -1.0, 0.125, 1.0, 7.25];
        for &value in &values {
            assert_eq!(
                sqrtf(value.abs() + 0.5, true).to_bits(),
                libm::sqrtf(value.abs() + 0.5).to_bits()
            );
            assert_eq!(expf(value, true).to_bits(), libm::expf(value).to_bits());
            assert_eq!(sinf(value, true).to_bits(), libm::sinf(value).to_bits());
            assert_eq!(cosf(value, true).to_bits(), libm::cosf(value).to_bits());
            assert_eq!(
                powf(10_000.0, value / 8.0, true).to_bits(),
                libm::powf(10_000.0, value / 8.0).to_bits()
            );
        }
    }

    #[test]
    fn rope_cache_preserves_angle_bits() {
        let cfg = Config {
            dim: 12,
            hidden: 16,
            n_layers: 1,
            n_heads: 3,
            n_kv_heads: 1,
            vocab: 8,
            seq_len: 5,
            rope_theta: 10_000.0,
            rms_norm_eps: 1e-5,
            rope_interleaved: false,
            r4_attention: false,
        };
        let (cos, sin) = Llama::build_rope_cache(&cfg, false);
        let half = cfg.dim / cfg.n_heads / 2;
        for pos in 0..cfg.seq_len {
            for i in 0..half {
                let freq = 1.0f32 / cfg.rope_theta.powf((2 * i) as f32 / (2 * half) as f32);
                let angle = pos as f32 * freq;
                assert_eq!(cos[pos * half + i].to_bits(), angle.cos().to_bits());
                assert_eq!(sin[pos * half + i].to_bits(), angle.sin().to_bits());
            }
        }
    }

    #[test]
    fn canonical_softmax_is_repeatable() {
        let input = [3.0f32, -1.0, 0.25, 7.0];
        let mut first = input;
        let mut second = input;
        softmax_with_mode(&mut first, true);
        softmax_with_mode(&mut second, true);
        assert_eq!(first, second);
        assert!((first.iter().sum::<f32>() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn matmul_flag_is_a_bit_exact_compatibility_noop() {
        const ROWS: usize = 67;
        const COLUMNS: usize = 73;
        let input: Vec<f32> = (0..COLUMNS)
            .map(|index| ((index * 17 % 31) as f32 - 15.0) / 16.0)
            .collect();
        let weights: Vec<f32> = (0..ROWS * COLUMNS)
            .map(|index| ((index * 29 % 43) as f32 - 21.0) / 32.0)
            .collect();
        let executor = ExactExecutor::new(TeacherExecutionConfig::sequential())
            .expect("serial exact executor");
        let mut exact = [0.0f32; ROWS];
        let mut fast = [0.0f32; ROWS];
        matmul(&executor, &mut exact, &input, &weights, COLUMNS, false);
        matmul(&executor, &mut fast, &input, &weights, COLUMNS, true);
        assert_eq!(
            exact.map(f32::to_bits),
            fast.map(f32::to_bits),
            "the compatibility flag must not select another matmul owner"
        );
    }

    #[test]
    fn matmul_batched_matches_serial_exact_bits() {
        const N: usize = 73;
        const ROWS: usize = 40;
        const BATCH: usize = 6;
        let weights: Vec<f32> = (0..ROWS * N)
            .map(|index| ((index * 29 % 43) as f32 - 21.0) / 32.0)
            .collect();
        let x: Vec<f32> = (0..BATCH * N)
            .map(|index| ((index * 17 % 31) as f32 - 15.0) / 16.0)
            .collect();
        let executor = ExactExecutor::new(TeacherExecutionConfig::sequential())
            .expect("serial exact executor");
        let mut batched = vec![0.0f32; BATCH * ROWS];
        matmul_batched(&executor, &mut batched, &x, &weights, N, BATCH);
        for bi in 0..BATCH {
            let mut serial = [0.0f32; ROWS];
            matmul(
                &executor,
                &mut serial,
                &x[bi * N..(bi + 1) * N],
                &weights,
                N,
                true,
            );
            for row in 0..ROWS {
                let want = serial[row];
                let got = batched[bi * ROWS + row];
                // Both shapes use the pinned exact owner and must agree in
                // symbol bits on every target.
                assert_eq!(
                    got.to_bits(),
                    want.to_bits(),
                    "batched != serial at b{bi} row{row}"
                );
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn teacher_execution_is_sequential_unless_parallelism_is_explicit() {
        let sequential = ExactExecutor::new(TeacherExecutionConfig::default())
            .expect("the one-worker exact executor must build");
        assert_eq!(sequential.snapshot().requested_workers, 1);
        assert_eq!(sequential.snapshot().effective_workers, 1);

        let discovered = ExactExecutor::new(TeacherExecutionConfig::available_parallelism())
            .expect("the explicitly requested host-sized executor must build");
        assert_eq!(
            discovered.snapshot().effective_workers,
            std::thread::available_parallelism().map_or(1, usize::from)
        );
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        not(all(feature = "observation-blas-exception", target_os = "macos"))
    ))]
    #[test]
    fn hosted_uor_matmul_uses_runtime_feature_detection() {
        let report = exact_backend_report();
        assert_eq!(report.arithmetic_owner, "uor-matmul exact GEMM");
        assert!(report.std_runtime_detection_enabled);
        assert_eq!(report.target_arch, std::env::consts::ARCH);
        assert_eq!(report.target_os, std::env::consts::OS);
        assert!(report
            .available_backends
            .iter()
            .any(|backend| backend == "portable"));
        assert_eq!(report.uor_matmul_revision, UOR_MATMUL_REVISION);
        assert_eq!(report.selected_backend, None);
        assert!(report.selection_status.starts_with("UNAVAILABLE:"));

        #[cfg(target_arch = "aarch64")]
        assert_eq!(
            uor_matmul::kernels::isa::arm::dotprod_available(),
            std::arch::is_aarch64_feature_detected!("dotprod")
        );
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        assert_eq!(
            uor_matmul::kernels::isa::x86::avx2_available(),
            std::arch::is_x86_feature_detected!("avx2")
        );
    }

    #[cfg(all(target_os = "macos", feature = "observation-blas-exception"))]
    #[test]
    fn observation_blas_exception_reports_non_exact_owner() {
        let report = exact_backend_report();
        assert_eq!(
            report.arithmetic_owner,
            "Apple Accelerate observation BLAS exception"
        );
        assert_eq!(report.selected_backend.as_deref(), Some("Apple Accelerate"));
        assert!(report.selection_status.starts_with("AVAILABLE:"));
        assert_eq!(report.target_os, std::env::consts::OS);
        assert_eq!(report.uor_matmul_revision, UOR_MATMUL_REVISION);
    }

    #[test]
    fn declared_uor_matmul_revision_matches_both_target_dependencies() {
        let manifest = include_str!("../Cargo.toml");
        let pin = format!("rev = \"{UOR_MATMUL_REVISION}\"");
        assert_eq!(
            manifest.matches(&pin).count(),
            2,
            "hosted and wasm exact arithmetic dependencies must share the declared revision"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn exact_matmul_matches_serial_bits_for_1_2_4_8_workers() {
        use std::num::NonZeroUsize;

        const ROWS: usize = 67;
        const COLUMNS: usize = 73;
        let input: Vec<f32> = (0..COLUMNS)
            .map(|index| ((index * 17 % 31) as f32 - 15.0) / 16.0)
            .collect();
        let weights: Vec<f32> = (0..ROWS * COLUMNS)
            .map(|index| ((index * 29 % 43) as f32 - 21.0) / 32.0)
            .collect();
        let serial = ExactExecutor::new(TeacherExecutionConfig::sequential())
            .expect("serial exact executor");
        let mut expected = [0.0f32; ROWS];
        serial.matmul(&mut expected, &input, &weights, COLUMNS);

        for workers in [1usize, 2, 4, 8] {
            let executor = ExactExecutor::new(TeacherExecutionConfig::fixed_workers(
                NonZeroUsize::new(workers).expect("worker count is nonzero"),
            ))
            .expect("fixed exact executor");
            let mut actual = [0.0f32; ROWS];
            executor.matmul(&mut actual, &input, &weights, COLUMNS);
            assert_eq!(
                actual.map(f32::to_bits),
                expected.map(f32::to_bits),
                "worker count {workers} changed exact matmul bits"
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn exact_batched_matmul_matches_serial_bits_for_1_2_4_8_workers() {
        use std::num::NonZeroUsize;

        const N: usize = 73;
        const ROWS: usize = 67;
        const BATCH: usize = 5;
        let weights: Vec<f32> = (0..ROWS * N)
            .map(|index| ((index * 29 % 43) as f32 - 21.0) / 32.0)
            .collect();
        let x: Vec<f32> = (0..BATCH * N)
            .map(|index| ((index * 17 % 31) as f32 - 15.0) / 16.0)
            .collect();
        let serial = ExactExecutor::new(TeacherExecutionConfig::sequential())
            .expect("serial exact executor");
        let mut expected = vec![0.0f32; BATCH * ROWS];
        serial.matmul_batched(&mut expected, &x, &weights, N, BATCH);

        for workers in [1usize, 2, 4, 8] {
            let executor = ExactExecutor::new(TeacherExecutionConfig::fixed_workers(
                NonZeroUsize::new(workers).expect("worker count is nonzero"),
            ))
            .expect("fixed exact executor");
            let mut actual = vec![0.0f32; BATCH * ROWS];
            executor.matmul_batched(&mut actual, &x, &weights, N, BATCH);
            assert_eq!(
                actual
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                expected
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                "worker count {workers} changed exact batched-matmul bits"
            );
            if workers == 8 {
                let snapshot = executor.snapshot();
                assert!(
                    snapshot.max_active_workers >= 2,
                    "shared-weight batched GEMM did not overlap output-row tiles"
                );
                assert!(snapshot.max_active_workers <= snapshot.effective_workers);
                assert!(snapshot.tiles_completed > 1);
            }
        }
    }

    /// Bounded synthetic decision instrument for the caller-owned Atlas
    /// A-panel offer. The shapes are the row tiles one W=8 SmolLM2-135M
    /// forward presents to `uor-matmul` at batch width eight. The weighted
    /// aggregate uses their calls per layer/forward, including the single
    /// vocabulary tile, without loading model weights.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    #[ignore = "bounded exact A-panel cache benchmark; run explicitly with --nocapture"]
    fn bench_exact_a_panel_cache_rows_on_smollm2_tiles() {
        use std::hint::black_box;
        use std::time::{Duration, Instant};
        use uor_matmul::PackedCode;

        #[derive(Clone, Copy)]
        struct Case {
            label: &'static str,
            m: usize,
            k: usize,
            calls_per_forward: u32,
            samples: usize,
        }

        fn measure(
            case: Case,
            a: &[f32],
            b: &[f32],
            c: &mut [f32],
            pa: &mut [PackedCode],
            pb: &mut [PackedCode],
        ) -> Duration {
            let started = Instant::now();
            uor_matmul::slice::gemm_float(case.m, case.k, 8, a, b, c, pa, pb)
                .expect("representative exact product is conformant");
            let elapsed = started.elapsed();
            black_box(c);
            elapsed
        }

        let cases = [
            Case {
                label: "q_o",
                m: 18,
                k: 576,
                calls_per_forward: 60,
                samples: 5,
            },
            Case {
                label: "k_v",
                m: 6,
                k: 576,
                calls_per_forward: 60,
                samples: 5,
            },
            Case {
                label: "w1_w3",
                m: 48,
                k: 576,
                calls_per_forward: 60,
                samples: 5,
            },
            Case {
                label: "w2",
                m: 18,
                k: 1_536,
                calls_per_forward: 30,
                samples: 5,
            },
            Case {
                label: "vocab",
                m: 1_536,
                k: 576,
                calls_per_forward: 1,
                samples: 3,
            },
        ];

        let mut weighted_one_row_ns = 0u128;
        let mut weighted_eight_row_ns = 0u128;
        for case in cases {
            let a: Vec<f32> = (0..case.m * case.k)
                .map(|index| ((index * 29 % 43) as f32 - 21.0) / 32.0)
                .collect();
            let b: Vec<f32> = (0..case.k * 8)
                .map(|index| ((index * 17 % 31) as f32 - 15.0) / 16.0)
                .collect();
            let mut one_row_output = vec![0.0f32; case.m * 8];
            let mut eight_row_output = vec![0.0f32; case.m * 8];
            let mut one_row_pa = vec![PackedCode::default(); case.k];
            let mut eight_row_pa = vec![PackedCode::default(); case.k * case.m.min(8)];
            let mut one_row_pb = vec![PackedCode::default(); case.k * 8];
            let mut eight_row_pb = vec![PackedCode::default(); case.k * 8];

            // Excluded per-offer warm-up resolves the same pinned backend and
            // fills each retained cache before the interleaved samples.
            measure(
                case,
                &a,
                &b,
                &mut one_row_output,
                &mut one_row_pa,
                &mut one_row_pb,
            );
            measure(
                case,
                &a,
                &b,
                &mut eight_row_output,
                &mut eight_row_pa,
                &mut eight_row_pb,
            );
            assert_eq!(
                one_row_output
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                eight_row_output
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                "A-panel offer changed exact bits for {}",
                case.label
            );

            let mut one_row = Vec::with_capacity(case.samples);
            let mut eight_row = Vec::with_capacity(case.samples);
            for sample in 0..case.samples {
                if sample % 2 == 0 {
                    one_row.push(measure(
                        case,
                        &a,
                        &b,
                        &mut one_row_output,
                        &mut one_row_pa,
                        &mut one_row_pb,
                    ));
                    eight_row.push(measure(
                        case,
                        &a,
                        &b,
                        &mut eight_row_output,
                        &mut eight_row_pa,
                        &mut eight_row_pb,
                    ));
                } else {
                    eight_row.push(measure(
                        case,
                        &a,
                        &b,
                        &mut eight_row_output,
                        &mut eight_row_pa,
                        &mut eight_row_pb,
                    ));
                    one_row.push(measure(
                        case,
                        &a,
                        &b,
                        &mut one_row_output,
                        &mut one_row_pa,
                        &mut one_row_pb,
                    ));
                }
                assert_eq!(
                    one_row_output
                        .iter()
                        .map(|value| value.to_bits())
                        .collect::<Vec<_>>(),
                    eight_row_output
                        .iter()
                        .map(|value| value.to_bits())
                        .collect::<Vec<_>>(),
                    "sample {sample} changed exact bits for {}",
                    case.label
                );
            }
            one_row.sort_unstable();
            eight_row.sort_unstable();
            let one_row_median = one_row[case.samples / 2];
            let eight_row_median = eight_row[case.samples / 2];
            weighted_one_row_ns = weighted_one_row_ns.saturating_add(
                one_row_median
                    .as_nanos()
                    .saturating_mul(u128::from(case.calls_per_forward)),
            );
            weighted_eight_row_ns = weighted_eight_row_ns.saturating_add(
                eight_row_median
                    .as_nanos()
                    .saturating_mul(u128::from(case.calls_per_forward)),
            );
            println!(
                "EXACT_A_PANEL_CACHE label={} m={} k={} n=8 calls={} pa1_ns={} pa8_ns={} speedup={:.4}",
                case.label,
                case.m,
                case.k,
                case.calls_per_forward,
                one_row_median.as_nanos(),
                eight_row_median.as_nanos(),
                one_row_median.as_secs_f64() / eight_row_median.as_secs_f64()
            );
        }

        println!(
            "EXACT_A_PANEL_CACHE weighted_pa1_ns={} weighted_pa8_ns={} weighted_speedup={:.4}",
            weighted_one_row_ns,
            weighted_eight_row_ns,
            weighted_one_row_ns as f64 / weighted_eight_row_ns as f64
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn exact_executor_reports_real_bounded_concurrency_and_progress() {
        use std::num::NonZeroUsize;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::{Arc, Barrier, Mutex};

        const N: usize = 257;
        const ROWS: usize = 128;
        let callbacks = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&callbacks);
        // The planted product is intentionally small, so an unconstrained
        // scheduler can occasionally finish one row tile before another pool
        // worker starts. Hold only the first two real task-entry callbacks at
        // a barrier. Both tasks have already incremented `active_workers` and
        // retain their `ActiveTask` guards, making the observed overlap real
        // while leaving production scheduling and arithmetic untouched.
        let entry_barrier = Arc::new(Barrier::new(2));
        let observer_barrier = Arc::clone(&entry_barrier);
        let entry_arrivals = Arc::new(AtomicUsize::new(0));
        let observer_arrivals = Arc::clone(&entry_arrivals);
        let config = TeacherExecutionConfig::fixed_workers(
            NonZeroUsize::new(4).expect("worker count is nonzero"),
        )
        .with_tiles_per_worker(NonZeroUsize::new(4).expect("tile count is nonzero"))
        .with_observer(Arc::new(move |snapshot| {
            seen.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(snapshot.observer_epoch);
            if snapshot.active_workers > 0 && snapshot.tiles_completed == 0 {
                let arrival = observer_arrivals.fetch_add(1, Ordering::AcqRel);
                if arrival < 2 {
                    observer_barrier.wait();
                }
            }
        }));
        let executor = ExactExecutor::new(config).expect("fixed exact executor");
        let input: Vec<f32> = (0..N)
            .map(|index| ((index * 17 % 31) as f32 - 15.0) / 16.0)
            .collect();
        let weights: Vec<f32> = (0..ROWS * N)
            .map(|index| ((index * 29 % 43) as f32 - 21.0) / 32.0)
            .collect();
        let mut output = [0.0f32; ROWS];
        executor.matmul(&mut output, &input, &weights, N);

        let snapshot = executor.snapshot();
        assert_eq!(snapshot.effective_workers, 4);
        assert!(
            entry_arrivals.load(Ordering::Acquire) >= 2,
            "fewer than two real exact tasks reached the planted start barrier"
        );
        assert!(
            snapshot.max_active_workers >= 2,
            "no overlapping exact work"
        );
        assert!(snapshot.max_active_workers <= snapshot.effective_workers);
        assert_eq!(snapshot.active_workers, 0);
        assert_eq!(snapshot.matrix_calls, 1);
        assert!(snapshot.tiles_completed > 1);
        assert_eq!(snapshot.output_cells_completed, ROWS as u64);
        assert_eq!(snapshot.scalar_terms_completed, (ROWS * N) as u64);
        let mut callback_epochs = callbacks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert!(!callback_epochs.is_empty());
        assert!(
            callback_epochs.len() < snapshot.tiles_completed as usize,
            "observer publication unexpectedly returned to one callback per timed tile"
        );
        let tile_progress_bound =
            1usize.saturating_add((snapshot.tiles_completed as usize).div_ceil(4));
        let task_entry_bound = snapshot.effective_workers;
        assert!(
            callback_epochs.len() <= tile_progress_bound.saturating_add(task_entry_bound),
            "observer publication exceeded one callback per worker wave plus bounded task entry"
        );
        assert!(callback_epochs.iter().all(|&epoch| epoch > 0));
        let callback_count = callback_epochs.len();
        callback_epochs.sort_unstable();
        callback_epochs.dedup();
        assert_eq!(callback_epochs.len(), callback_count);
        assert_eq!(
            snapshot.observer_epoch,
            callback_epochs.last().copied().unwrap()
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn every_physical_forward_republishes_actual_multiworker_occupancy() {
        use std::collections::BTreeMap;
        use std::num::NonZeroUsize;
        use std::sync::{Arc, Mutex};

        const N: usize = 257;
        const ROWS: usize = 128;
        const STREAMS: usize = 8;
        let observed_peaks = Arc::new(Mutex::new(BTreeMap::<u64, usize>::new()));
        let observer_peaks = Arc::clone(&observed_peaks);
        let config = TeacherExecutionConfig::fixed_workers(
            NonZeroUsize::new(4).expect("worker count is nonzero"),
        )
        .with_tiles_per_worker(NonZeroUsize::new(4).expect("tile count is nonzero"))
        .with_observer(Arc::new(move |snapshot| {
            observer_peaks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .entry(snapshot.forward_calls)
                .and_modify(|peak| *peak = (*peak).max(snapshot.active_workers))
                .or_insert(snapshot.active_workers);
        }));
        let executor = ExactExecutor::new(config).expect("fixed exact executor");
        let input: Vec<f32> = (0..STREAMS * N)
            .map(|index| ((index * 17 % 31) as f32 - 15.0) / 16.0)
            .collect();
        let weights: Vec<f32> = (0..ROWS * N)
            .map(|index| ((index * 29 % 43) as f32 - 21.0) / 32.0)
            .collect();

        for _ in 0..2 {
            let mut output = vec![0.0f32; STREAMS * ROWS];
            let before = executor.snapshot();
            executor.begin_forward(STREAMS);
            executor.matmul_batched(&mut output, &input, &weights, N, STREAMS);
            executor.complete_forward(STREAMS);
            let after = executor.snapshot();
            assert_eq!(after.forward_calls - before.forward_calls, 1);
            assert_eq!(
                after.multiworker_forward_calls - before.multiworker_forward_calls,
                1
            );
            assert!(after.forward_max_active_workers >= 2);
            assert!(after.forward_max_active_workers <= after.effective_workers);
        }

        let observed_peaks = observed_peaks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for forward in [1, 2] {
            assert!(
                observed_peaks.get(&forward).copied().unwrap_or(0) >= 2,
                "physical forward {forward} never republished overlapping row workers"
            );
        }
    }

    #[cfg(not(all(target_os = "macos", feature = "observation-blas-exception")))]
    #[test]
    fn repeated_batched_forwards_reuse_every_exact_workspace_capacity() {
        use std::num::NonZeroUsize;

        let mut model = tiny_llama();
        model
            .set_execution_config(
                TeacherExecutionConfig::fixed_workers(
                    NonZeroUsize::new(4).expect("worker count is nonzero"),
                )
                .with_tiles_per_worker(NonZeroUsize::new(4).expect("tiles per worker is nonzero")),
            )
            .expect("fixed exact executor");
        let tokens = [1usize, 2, 3, 4, 5, 6, 7, 8];
        let positions = [0usize; 8];

        let mut first_states: Vec<State> = (0..8).map(|_| State::new(&model.cfg)).collect();
        model.forward_batch(&mut first_states, &tokens, &positions, true);
        let after_first = model.execution_snapshot();
        assert!(after_first.workspace_growth_events > 0);
        assert!(after_first.workspace_growth_bytes > 0);

        let mut second_states: Vec<State> = (0..8).map(|_| State::new(&model.cfg)).collect();
        model.forward_batch(&mut second_states, &tokens, &positions, true);
        let after_second = model.execution_snapshot();
        assert_eq!(
            after_second.workspace_growth_events, after_first.workspace_growth_events,
            "steady-state batched forward grew a retained workspace"
        );
        assert_eq!(
            after_second.workspace_growth_bytes, after_first.workspace_growth_bytes,
            "steady-state batched forward allocated new workspace capacity"
        );
        assert_eq!(
            first_states
                .iter()
                .flat_map(|state| state.logits.iter().map(|value| value.to_bits()))
                .collect::<Vec<_>>(),
            second_states
                .iter()
                .flat_map(|state| state.logits.iter().map(|value| value.to_bits()))
                .collect::<Vec<_>>(),
            "workspace reuse changed teacher output bits"
        );
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        not(all(target_os = "macos", feature = "observation-blas-exception"))
    ))]
    #[test]
    fn every_adaptive_candidate_prepares_workspace_outside_measurement() {
        use std::num::NonZeroUsize;
        use std::sync::Arc;

        let mut model = tiny_llama();
        let tokens = [1usize, 2, 3, 4, 5, 6, 7, 8];
        let positions = [0usize; 8];
        let mut reference_logits = None;
        for workers in [8usize, 4] {
            model
                .set_execution_config(
                    TeacherExecutionConfig::fixed_workers(
                        NonZeroUsize::new(workers).expect("worker count is nonzero"),
                    )
                    .with_tiles_per_worker(
                        NonZeroUsize::new(4).expect("tiles per worker is nonzero"),
                    ),
                )
                .expect("adaptive exact executor");
            let preparation = model
                .prestart_exact_execution(8)
                .expect("excluded exact workspace preparation");
            assert_eq!(preparation.workers_observed, workers);
            assert!(preparation.workspace_capacity_bytes > 0);
            assert!(preparation.workspace_growth_events > 0);
            assert!(preparation.workspace_growth_bytes > 0);

            model.begin_measured_execution(Arc::new(|_| {}));
            let mut states: Vec<State> = (0..8).map(|_| State::new(&model.cfg)).collect();
            model.forward_batch(&mut states, &tokens, &positions, true);
            let measured = model.execution_snapshot();
            assert_eq!(measured.workspace_growth_events, 0);
            assert_eq!(measured.workspace_growth_bytes, 0);
            let logits = states
                .iter()
                .flat_map(|state| state.logits.iter().map(|value| value.to_bits()))
                .collect::<Vec<_>>();
            if let Some(reference) = &reference_logits {
                assert_eq!(
                    &logits, reference,
                    "candidate worker count changed exact bits"
                );
            } else {
                reference_logits = Some(logits);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn atomic_progress_bridge_ignores_stale_epochs_and_publishes_exact_final_snapshot() {
        let bridge = AtomicTeacherExecutionProgress::default();
        let newest = TeacherExecutionSnapshot {
            observer_epoch: 9,
            effective_workers: 8,
            max_active_workers: 8,
            forward_calls: 3,
            streams_started: 24,
            streams_completed: 24,
            max_active_streams: 8,
            matrix_calls: 17,
            batched_matrix_calls: 17,
            max_matrix_batch_width: 8,
            tiles_completed: 99,
            output_cells_completed: 1234,
            scalar_terms_completed: 5678,
            ..TeacherExecutionSnapshot::default()
        };
        bridge.publish(newest);
        bridge.publish(TeacherExecutionSnapshot {
            observer_epoch: 8,
            tiles_completed: 1,
            ..TeacherExecutionSnapshot::default()
        });
        assert_eq!(bridge.snapshot(), newest);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn measured_execution_excludes_cheap_pool_backend_prestart() {
        use std::num::NonZeroUsize;
        use std::sync::Arc;

        let mut executor = ExactExecutor::new(TeacherExecutionConfig::fixed_workers(
            NonZeroUsize::new(4).expect("worker count is nonzero"),
        ))
        .expect("fixed exact executor");
        let prestart = executor.prestart(8, 32, 64).expect("cheap exact prestart");
        assert_eq!(prestart.workers_observed, 4);
        assert_eq!(prestart.batch_width, 8);
        assert!(prestart.backend_exercised);
        assert!(prestart.workspace_capacity_bytes > 0);
        assert!(prestart.workspace_growth_events > 0);
        assert!(prestart.workspace_growth_bytes > 0);
        assert_eq!(executor.snapshot().forward_calls, 0);
        assert_eq!(executor.snapshot().matrix_calls, 1);

        executor.begin_measured_execution(Arc::new(|_| {}));
        assert_eq!(executor.snapshot().matrix_calls, 0);
        assert_eq!(executor.snapshot().workspace_growth_events, 0);
        assert_eq!(executor.snapshot().workspace_growth_bytes, 0);
        let input = [0.25f32; 32];
        let weights = [0.5f32; 64 * 32];
        let mut output = [0.0f32; 64];
        executor.matmul(&mut output, &input, &weights, 32);
        let measured = executor.snapshot();
        assert_eq!(measured.requested_workers, 4);
        assert_eq!(measured.effective_workers, 4);
        assert_eq!(measured.matrix_calls, 1);
        assert_eq!(measured.workspace_growth_events, 0);
        assert_eq!(measured.workspace_growth_bytes, 0);
        assert!(measured.observer_epoch > 0);
    }

    /// A tiny synthetic Llama with deterministic weights, for exercising the
    /// forward path without loading a real checkpoint.
    fn tiny_llama() -> Llama {
        let (dim, hid, nl, kv_dim, vocab, seq_len) =
            (8usize, 16usize, 2usize, 8usize, 10usize, 8usize);
        let cfg = Config {
            dim,
            hidden: hid,
            n_layers: nl,
            n_heads: 2,
            n_kv_heads: 2,
            vocab,
            seq_len,
            rope_theta: 10_000.0,
            rms_norm_eps: 1e-5,
            rope_interleaved: true,
            r4_attention: false,
        };
        let emb = 0;
        let rms_att = emb + vocab * dim;
        let wq = rms_att + nl * dim;
        let wk = wq + nl * dim * dim;
        let wv = wk + nl * dim * kv_dim;
        let wo = wv + nl * dim * kv_dim;
        let rms_ffn = wo + nl * dim * dim;
        let w1 = rms_ffn + nl * dim;
        let w2 = w1 + nl * dim * hid;
        let w3 = w2 + nl * hid * dim;
        let rms_final = w3 + nl * dim * hid;
        let total = rms_final + dim;
        let w: Vec<f32> = (0..total)
            .map(|i| (((i * 131 + 7) % 251) as f32 / 251.0 - 0.5) * 0.2)
            .collect();
        let mut model = Llama {
            cfg,
            w,
            rope_cos: Vec::new(),
            rope_sin: Vec::new(),
            emb,
            rms_att,
            wq,
            wk,
            wv,
            wo,
            rms_ffn,
            w1,
            w2,
            w3,
            rms_final,
            wcls: emb,
            canonical_math: false,
            exact_executor: ExactExecutor::new(TeacherExecutionConfig::default())
                .expect("the one-worker exact executor must build"),
            forward_gate: std::sync::Mutex::new(()),
            batch_workspace: std::sync::Mutex::new(Box::new(BatchForwardWorkspace::default())),
        };
        model.rebuild_rope_cache();
        model
    }

    fn tiny_geometric_oracle() -> HuggingFaceLlamaOracle {
        let model = tiny_llama();
        let state = State::new(&model.cfg);
        HuggingFaceLlamaOracle {
            model,
            state,
            kappa: "blake3:synthetic-source".to_owned(),
            source_bytes: 0,
            bos_token: 1,
            eos_token: 2,
            fast_matmul: true,
        }
    }

    fn tiny_geometry_context(
        mixer: &geometric_decoder::GeometricMixer,
    ) -> geometric_decoder::GeometryContext {
        let tokenizer_cid = "blake3:synthetic-tokenizer";
        let adapter_identity =
            mixer.memory_adapter_identity("blake3:synthetic-source", tokenizer_cid);
        geometric_decoder::GeometryContext::new(
            "alice",
            tokenizer_cid,
            adapter_identity.clone(),
            [0.2, -0.3, 0.4, 0.5],
            vec![geometric_decoder::GeometryMemorySpan {
                sequence: 0,
                role: "user".to_owned(),
                text: "remember token".to_owned(),
                token_ids: vec![1, 2],
                tokenizer_cid: tokenizer_cid.to_owned(),
                adapter_identity,
                r4_coordinates: [0.1, 0.7, -0.2, 0.3],
                provenance: "blake3:memory".to_owned(),
            }],
            geometric_decoder::GeometryProvenance {
                source_cid: "blake3:synthetic-source".to_owned(),
                router_state_cid: "blake3:router".to_owned(),
                memory_source: "focused-test".to_owned(),
            },
        )
        .expect("valid geometry context")
    }

    #[derive(Default)]
    struct IdentityCausalAttentionTransport;

    impl attention::CausalAttentionTransport for IdentityCausalAttentionTransport {
        fn policy_identity(&self) -> &str {
            "identity-full-prefix-control/1"
        }

        fn begin_position(&mut self, _token: usize, _position: usize) {}

        fn transform_query(
            &mut self,
            _context: attention::CausalAttentionHeadContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }

        fn transport_key(
            &mut self,
            _context: attention::CausalAttentionSourceContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }

        fn transport_value(
            &mut self,
            _context: attention::CausalAttentionSourceContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }

        fn output_to_model_frame(
            &mut self,
            _context: attention::CausalAttentionHeadContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }
    }

    #[derive(Debug, Default)]
    struct PreRopeProjectionObservations {
        contexts: Vec<attention::CausalAttentionProjectionContext>,
        query_at_position_one: Option<[u32; 2]>,
        key_at_position_one: Option<[u32; 2]>,
        value_at_position_one: Option<[u32; 2]>,
    }

    struct PreRopeProjectionProbe {
        observations: std::sync::Arc<std::sync::Mutex<PreRopeProjectionObservations>>,
    }

    impl attention::CausalAttentionTransport for PreRopeProjectionProbe {
        fn policy_identity(&self) -> &str {
            "pre-rope-projection-probe/1"
        }

        fn begin_position(&mut self, _token: usize, _position: usize) {}

        fn transform_projected_qkv_before_rope(
            &mut self,
            context: attention::CausalAttentionProjectionContext,
            query: &mut [f32],
            key: &mut [f32],
            value: &mut [f32],
        ) {
            self.observations
                .lock()
                .expect("projection observations lock")
                .contexts
                .push(context);
            query.fill(0.0);
            key.fill(0.0);
            value.fill(0.0);
            query[0] = 1.0;
            key[0] = 1.0;
            value[0] = 3.0;
        }

        fn transform_query(
            &mut self,
            context: attention::CausalAttentionHeadContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            if context.query_position == 1 && context.head == 0 {
                self.observations
                    .lock()
                    .expect("query observations lock")
                    .query_at_position_one = Some([input[0].to_bits(), input[1].to_bits()]);
            }
            output.copy_from_slice(input);
        }

        fn transport_key(
            &mut self,
            context: attention::CausalAttentionSourceContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            if context.query_position == 1 && context.source_position == 1 && context.head == 0 {
                self.observations
                    .lock()
                    .expect("key observations lock")
                    .key_at_position_one = Some([input[0].to_bits(), input[1].to_bits()]);
            }
            output.copy_from_slice(input);
        }

        fn transport_value(
            &mut self,
            context: attention::CausalAttentionSourceContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            if context.query_position == 1 && context.source_position == 1 && context.head == 0 {
                self.observations
                    .lock()
                    .expect("value observations lock")
                    .value_at_position_one = Some([input[0].to_bits(), input[1].to_bits()]);
            }
            output.copy_from_slice(input);
        }

        fn output_to_model_frame(
            &mut self,
            _context: attention::CausalAttentionHeadContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }
    }

    #[derive(Default)]
    struct FaultingCausalAttentionTransport {
        faulted: bool,
    }

    impl attention::CausalAttentionTransport for FaultingCausalAttentionTransport {
        fn reset(&mut self) {
            self.faulted = false;
        }

        fn policy_identity(&self) -> &str {
            "faulting-full-prefix-control/1"
        }

        fn status(&self) -> Result<(), String> {
            if self.faulted {
                Err("injected transport fault".to_owned())
            } else {
                Ok(())
            }
        }

        fn begin_position(&mut self, _token: usize, _position: usize) {}

        fn transform_projected_qkv_before_rope(
            &mut self,
            _context: attention::CausalAttentionProjectionContext,
            _query: &mut [f32],
            _key: &mut [f32],
            _value: &mut [f32],
        ) {
            self.faulted = true;
        }

        fn transform_query(
            &mut self,
            _context: attention::CausalAttentionHeadContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }

        fn transport_key(
            &mut self,
            _context: attention::CausalAttentionSourceContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }

        fn transport_value(
            &mut self,
            _context: attention::CausalAttentionSourceContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }

        fn output_to_model_frame(
            &mut self,
            _context: attention::CausalAttentionHeadContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }
    }

    struct ReplacingCausalAttentionOperator {
        score_calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        centroid_calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl attention::CausalAttentionTransport for ReplacingCausalAttentionOperator {
        fn policy_identity(&self) -> &str {
            "replacing-causal-attention-operator/1"
        }

        fn begin_position(&mut self, _token: usize, _position: usize) {}

        fn transform_query(
            &mut self,
            _context: attention::CausalAttentionHeadContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }

        fn transport_key(
            &mut self,
            _context: attention::CausalAttentionSourceContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }

        fn transport_value(
            &mut self,
            _context: attention::CausalAttentionSourceContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }

        fn score_and_normalize(
            &mut self,
            _context: attention::CausalAttentionHeadContext,
            query: &[f32],
            packed_keys: &[f32],
            output_weights: &mut [f32],
            _canonical_math: bool,
        ) {
            use std::sync::atomic::Ordering;

            assert_eq!(packed_keys.len(), output_weights.len() * query.len());
            self.score_calls.fetch_add(1, Ordering::Relaxed);
            output_weights.fill(0.0);
            *output_weights
                .last_mut()
                .expect("the causal prefix is nonempty") = 1.0;
        }

        fn weighted_value_centroid(
            &mut self,
            _context: attention::CausalAttentionHeadContext,
            weights: &[f32],
            packed_values: &[f32],
            output: &mut [f32],
        ) {
            use std::sync::atomic::Ordering;

            assert_eq!(packed_values.len(), weights.len() * output.len());
            self.centroid_calls.fetch_add(1, Ordering::Relaxed);
            let output_width = output.len();
            attention::head_attention_value_aggregate(
                output,
                weights,
                packed_values,
                0,
                output_width,
            );
            for coordinate in output {
                *coordinate = -*coordinate;
            }
        }

        fn output_to_model_frame(
            &mut self,
            _context: attention::CausalAttentionHeadContext,
            input: &[f32],
            output: &mut [f32],
        ) {
            output.copy_from_slice(input);
        }
    }

    #[test]
    fn causal_attention_transport_preserves_full_decoder_and_audits_dense_prefix() {
        let oracle = tiny_geometric_oracle();
        let mut plain = oracle.new_state_bounded(6).expect("plain bounded state");
        let mut transported = oracle
            .new_causal_attention_transport_session(
                Box::new(IdentityCausalAttentionTransport),
                attention::CausalAttentionLayerSelection::All,
                6,
            )
            .expect("identity transport session");
        assert_eq!(
            transported.policy_identity(),
            "identity-full-prefix-control/1"
        );
        assert_eq!(transported.selected_layer_count(), oracle.cfg().n_layers);
        assert!(transported.layer_is_selected(0));
        assert!(transported.layer_is_selected(1));

        let mut plain_logits = vec![0.0; oracle.cfg().vocab];
        let mut transported_logits = vec![0.0; oracle.cfg().vocab];
        for (position, token) in [1usize, 3, 5].into_iter().enumerate() {
            oracle
                .step_state(&mut plain, token, position, &mut plain_logits)
                .expect("ordinary decoder step");
            oracle
                .step_causal_attention_transport(
                    &mut transported,
                    token,
                    position,
                    &mut transported_logits,
                )
                .expect("transported decoder step");
            assert_eq!(
                plain_logits
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                transported_logits
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(
            plain.persistent_state_cid(),
            transported.persistent_state_cid()
        );

        let audit = transported.audit();
        assert_eq!(audit.positions, 3);
        assert_eq!(audit.layers, 6);
        assert_eq!(audit.heads, 12);
        assert_eq!(audit.query_transforms, 12);
        assert_eq!(audit.key_transports, 24);
        assert_eq!(audit.value_transports, 24);
        assert_eq!(audit.output_transforms, 12);
        assert_eq!(audit.future_reads, 0);
        assert_eq!(audit.maximum_query_position, Some(2));
        assert_eq!(audit.maximum_source_position, Some(2));

        let projection_audit = transported.pre_rope_projection_audit();
        assert_eq!(projection_audit.hook_calls, 6);
        assert_eq!(projection_audit.query_vectors, 12);
        assert_eq!(projection_audit.key_vectors, 12);
        assert_eq!(projection_audit.value_vectors, 12);
        assert_eq!(projection_audit.query_lanes, 48);
        assert_eq!(projection_audit.key_lanes, 48);
        assert_eq!(projection_audit.value_lanes, 48);
    }

    #[test]
    fn pre_rope_projection_hook_runs_once_per_layer_before_rope() {
        let oracle = tiny_geometric_oracle();
        let head_size = oracle.cfg().dim / oracle.cfg().n_heads;
        let first_angle = head_size / 2;
        let expected_query_key = [
            oracle.model.rope_cos[first_angle].to_bits(),
            oracle.model.rope_sin[first_angle].to_bits(),
        ];
        let observations = std::sync::Arc::new(std::sync::Mutex::new(
            PreRopeProjectionObservations::default(),
        ));
        let mut session = oracle
            .new_causal_attention_transport_session(
                Box::new(PreRopeProjectionProbe {
                    observations: observations.clone(),
                }),
                attention::CausalAttentionLayerSelection::All,
                4,
            )
            .expect("pre-RoPE projection session");
        let mut logits = vec![0.0; oracle.cfg().vocab];
        for (position, token) in [1usize, 3].into_iter().enumerate() {
            oracle
                .step_causal_attention_transport(&mut session, token, position, &mut logits)
                .expect("pre-RoPE projection step");
        }

        let observations = observations.lock().expect("projection observations lock");
        assert_eq!(
            observations.contexts,
            vec![
                attention::CausalAttentionProjectionContext {
                    layer: 0,
                    query_position: 0,
                    query_heads: 2,
                    key_value_heads: 2,
                    head_size: 4,
                },
                attention::CausalAttentionProjectionContext {
                    layer: 1,
                    query_position: 0,
                    query_heads: 2,
                    key_value_heads: 2,
                    head_size: 4,
                },
                attention::CausalAttentionProjectionContext {
                    layer: 0,
                    query_position: 1,
                    query_heads: 2,
                    key_value_heads: 2,
                    head_size: 4,
                },
                attention::CausalAttentionProjectionContext {
                    layer: 1,
                    query_position: 1,
                    query_heads: 2,
                    key_value_heads: 2,
                    head_size: 4,
                },
            ]
        );
        assert_eq!(observations.query_at_position_one, Some(expected_query_key));
        assert_eq!(observations.key_at_position_one, Some(expected_query_key));
        assert_eq!(
            observations.value_at_position_one,
            Some([3.0f32.to_bits(), 0.0f32.to_bits()])
        );
        drop(observations);

        let projection_audit = session.pre_rope_projection_audit();
        assert_eq!(projection_audit.hook_calls, 4);
        assert_eq!(projection_audit.query_vectors, 8);
        assert_eq!(projection_audit.key_vectors, 8);
        assert_eq!(projection_audit.value_vectors, 8);
        assert_eq!(projection_audit.query_lanes, 32);
        assert_eq!(projection_audit.key_lanes, 32);
        assert_eq!(projection_audit.value_lanes, 32);
        assert_eq!(session.audit().future_reads, 0);
    }

    #[test]
    fn causal_attention_operator_hooks_are_invoked_and_change_decoder_behavior() {
        use std::sync::atomic::Ordering;

        let oracle = tiny_geometric_oracle();
        let mut plain = oracle.new_state_bounded(6).expect("plain bounded state");
        let score_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let centroid_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut replaced = oracle
            .new_causal_attention_transport_session(
                Box::new(ReplacingCausalAttentionOperator {
                    score_calls: score_calls.clone(),
                    centroid_calls: centroid_calls.clone(),
                }),
                attention::CausalAttentionLayerSelection::All,
                6,
            )
            .expect("replacement attention session");
        let mut plain_logits = vec![0.0; oracle.cfg().vocab];
        let mut replaced_logits = vec![0.0; oracle.cfg().vocab];
        for (position, token) in [1usize, 3, 5].into_iter().enumerate() {
            oracle
                .step_state(&mut plain, token, position, &mut plain_logits)
                .expect("ordinary decoder step");
            oracle
                .step_causal_attention_transport(
                    &mut replaced,
                    token,
                    position,
                    &mut replaced_logits,
                )
                .expect("replacement attention step");
        }

        let expected_calls = 3 * oracle.cfg().n_layers * oracle.cfg().n_heads;
        assert_eq!(score_calls.load(Ordering::Relaxed), expected_calls);
        assert_eq!(centroid_calls.load(Ordering::Relaxed), expected_calls);
        assert!(plain_logits
            .iter()
            .zip(&replaced_logits)
            .any(|(plain, replaced)| plain.to_bits() != replaced.to_bits()));
        assert_ne!(
            plain.persistent_state_cid(),
            replaced.persistent_state_cid()
        );
        assert_eq!(replaced.audit().future_reads, 0);
    }

    #[test]
    fn causal_attention_transport_fails_closed_on_shape_order_and_health() {
        use attention::{
            CausalAttentionLayerSelection as Selection, CausalAttentionTransportError,
        };

        let mut invalid_shape = tiny_geometric_oracle();
        invalid_shape.model.cfg.n_heads = 4;
        invalid_shape.model.cfg.n_kv_heads = 4;
        let shape_error = invalid_shape.new_causal_attention_transport_session(
            Box::new(IdentityCausalAttentionTransport),
            Selection::All,
            4,
        );
        assert!(matches!(
            shape_error,
            Err(CausalAttentionTransportError::HeadSizeNotDivisibleByFour { head_size: 2 })
        ));

        let oracle = tiny_geometric_oracle();
        let layer_error = oracle.new_causal_attention_transport_session(
            Box::new(IdentityCausalAttentionTransport),
            Selection::Selected(vec![oracle.cfg().n_layers]),
            4,
        );
        assert!(matches!(
            layer_error,
            Err(CausalAttentionTransportError::LayerOutOfRange {
                requested: 2,
                layers: 2
            })
        ));

        let mut session = oracle
            .new_causal_attention_transport_session(
                Box::new(FaultingCausalAttentionTransport::default()),
                Selection::Selected(vec![0]),
                4,
            )
            .expect("fault probe session");
        let mut logits = vec![123.0; oracle.cfg().vocab];
        let health_error = oracle.step_causal_attention_transport(&mut session, 1, 0, &mut logits);
        assert!(matches!(
            health_error,
            Err(CausalAttentionTransportError::TransportFault {
                ref policy_identity,
                ref reason,
            }) if policy_identity == "faulting-full-prefix-control/1"
                && reason == "injected transport fault"
        ));
        assert!(logits
            .iter()
            .all(|value| value.to_bits() == 123.0f32.to_bits()));
        assert!(session.transport_status().is_err());
        assert_eq!(session.pre_rope_projection_audit().hook_calls, 1);
        session.reset();
        assert_eq!(session.transport_status(), Ok(()));
        assert_eq!(session.pre_rope_projection_audit().hook_calls, 0);

        let order_error = oracle.step_causal_attention_transport(&mut session, 1, 1, &mut logits);
        assert!(matches!(
            order_error,
            Err(CausalAttentionTransportError::PositionOutOfOrder {
                requested: 1,
                expected: 0
            })
        ));
    }

    #[test]
    fn one_layer_geometry_is_disabled_exact_causal_bounded_and_reachable() {
        use geometric_decoder::{GeometricMixer, GeometryIntervention, DEFAULT_SUPPORT_BUDGET};

        let oracle = tiny_geometric_oracle();
        let mixer = GeometricMixer::deterministic(0, oracle.cfg().dim, b"issue-950-test")
            .expect("deterministic mixer");
        let context = tiny_geometry_context(&mixer);

        // Disabled mode is exactly the ordinary source control for the same
        // token/state sequence.
        let mut plain = oracle.new_state_bounded(6).expect("bounded source state");
        let mut disabled = oracle
            .new_geometric_session(
                mixer.clone(),
                context.clone(),
                GeometryIntervention::Disabled,
                6,
            )
            .expect("disabled session");
        let mut plain_logits = vec![0.0; oracle.cfg().vocab];
        let mut disabled_logits = vec![0.0; oracle.cfg().vocab];
        for (position, token) in [1usize, 3].into_iter().enumerate() {
            oracle
                .step_state(&mut plain, token, position, &mut plain_logits)
                .expect("plain step");
            oracle
                .step_geometric(&mut disabled, token, position, &mut disabled_logits)
                .expect("disabled step");
            assert_eq!(
                plain_logits
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                disabled_logits
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            );
        }
        assert!(disabled.traces().is_empty());

        // Clone after an identical real prefix, then change only the
        // candidate-coordinate intervention for the next causal step.
        let mut real = oracle
            .new_geometric_session(mixer, context, GeometryIntervention::Real, 6)
            .expect("real session");
        let mut logits = vec![0.0; oracle.cfg().vocab];
        for (position, token) in [1usize, 3].into_iter().enumerate() {
            oracle
                .step_geometric(&mut real, token, position, &mut logits)
                .expect("real prefix step");
        }
        let mut permuted = real.clone();
        real.clear_traces();
        permuted.clear_traces();
        permuted.set_intervention(GeometryIntervention::PermutedCoordinates);
        let mut real_logits = vec![0.0; oracle.cfg().vocab];
        let mut permuted_logits = vec![0.0; oracle.cfg().vocab];
        oracle
            .step_geometric(&mut real, 5, 2, &mut real_logits)
            .expect("real treatment step");
        oracle
            .step_geometric(&mut permuted, 5, 2, &mut permuted_logits)
            .expect("permuted treatment step");

        let real_trace = real.traces().last().expect("real trace");
        let permuted_trace = permuted.traces().last().expect("permuted trace");
        assert_eq!(real_trace.layer, 0);
        assert_eq!(real_trace.position, 2);
        assert_eq!(real_trace.prefix_candidates, 3);
        assert_eq!(real_trace.memory_candidates, 1);
        assert!(real_trace.selected_support.len() <= DEFAULT_SUPPORT_BUDGET);
        assert_eq!(real_trace.source_attention_calls, 0);
        assert!(!real_trace.dense_full_prefix_qk);
        assert_ne!(real_trace.support_cid, permuted_trace.support_cid);
        assert!(real_logits
            .iter()
            .zip(permuted_logits.iter())
            .any(|(left, right)| left.to_bits() != right.to_bits()));
        assert_eq!(real.context().position_states.last().unwrap().position, 2);
    }

    #[cfg(not(all(target_os = "macos", feature = "observation-blas-exception")))]
    #[test]
    fn exact_forward_plan_matches_observed_tiny_batch_delta() {
        use std::num::NonZeroUsize;

        let mut model = tiny_llama();
        model
            .set_execution_config(
                TeacherExecutionConfig::fixed_workers(
                    NonZeroUsize::new(4).expect("worker count is nonzero"),
                )
                .with_tiles_per_worker(NonZeroUsize::new(4).expect("tiles per worker is nonzero")),
            )
            .expect("fixed exact executor");
        let batch_width = 3usize;
        let plan = model
            .exact_forward_plan(batch_width)
            .expect("tiny exact forward plan");
        let before = model.execution_snapshot();
        let mut states: Vec<State> = (0..batch_width).map(|_| State::new(&model.cfg)).collect();
        model.forward_batch(&mut states, &[1, 2, 3], &[0, 0, 0], true);
        let after = model.execution_snapshot();

        assert_eq!(after.matrix_calls - before.matrix_calls, plan.matrix_calls);
        assert_eq!(
            after.tiles_completed - before.tiles_completed,
            plan.row_tiles
        );
        assert_eq!(plan.worker_tasks, plan.row_tiles);
        assert_eq!(
            after.output_cells_completed - before.output_cells_completed,
            plan.output_cells
        );
        assert_eq!(
            after.scalar_terms_completed - before.scalar_terms_completed,
            plan.scalar_terms
        );
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        not(all(feature = "observation-blas-exception", target_os = "macos"))
    ))]
    #[test]
    fn eight_stream_forward_keeps_private_states_and_observes_row_workers() {
        use std::num::NonZeroUsize;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let saw_streams_and_workers = Arc::new(AtomicBool::new(false));
        let observer_flag = Arc::clone(&saw_streams_and_workers);
        let mut model = tiny_llama();
        model
            .set_execution_config(
                TeacherExecutionConfig::fixed_workers(
                    NonZeroUsize::new(8).expect("worker count is nonzero"),
                )
                .with_tiles_per_worker(NonZeroUsize::new(4).expect("tiles per worker is nonzero"))
                .with_observer(Arc::new(move |snapshot| {
                    if snapshot.active_streams == 8 && snapshot.active_workers > 0 {
                        observer_flag.store(true, Ordering::Release);
                    }
                })),
            )
            .expect("eight-worker exact executor");
        let mut states: Vec<State> = (0..8).map(|_| State::new(&model.cfg)).collect();
        model.forward_batch(
            &mut states,
            &[1, 2, 3, 4, 5, 6, 7, 8],
            &[0, 0, 0, 0, 0, 0, 0, 0],
            true,
        );

        let snapshot = model.execution_snapshot();
        assert_eq!(snapshot.effective_workers, 8);
        assert_eq!(snapshot.streams_started, 8);
        assert_eq!(snapshot.streams_completed, 8);
        assert_eq!(snapshot.active_streams, 0);
        assert_eq!(snapshot.max_active_streams, 8);
        assert_eq!(snapshot.batched_matrix_calls, snapshot.matrix_calls);
        assert_eq!(snapshot.max_matrix_batch_width, 8);
        assert!(snapshot.observer_epoch > 0);
        assert!(snapshot.max_active_workers >= 2);
        assert!(snapshot.max_active_workers <= 8);
        assert!(saw_streams_and_workers.load(Ordering::Acquire));
        let state_bits: Vec<Vec<u32>> = states.iter().map(persistent_state_bits).collect();
        for left in 0..state_bits.len() {
            for right in left + 1..state_bits.len() {
                assert_ne!(
                    state_bits[left], state_bits[right],
                    "independent stream states {left} and {right} collapsed"
                );
            }
        }
        assert_eq!(
            model
                .exact_forward_plan(8)
                .expect("eight-stream exact plan")
                .batch_width,
            8
        );
    }

    #[test]
    fn legacy_llama_oracle_declares_the_selected_attention_branch() {
        let model = tiny_llama();
        let state = State::new(&model.cfg);
        let mut oracle = LlamaOracle {
            model,
            state,
            kappa: "blake3:synthetic".to_owned(),
            source_bytes: 0,
        };

        assert_eq!(
            oracle.attention_operator_spec(),
            Some(attention::AttentionOperatorSpec::standard())
        );
        assert_eq!(oracle.dense_operator_spec(), None);
        oracle.model.cfg.r4_attention = true;
        assert_eq!(
            oracle.attention_operator_spec(),
            Some(attention::AttentionOperatorSpec::experimental_r4())
        );
        assert_eq!(oracle.dense_operator_spec(), None);
    }

    /// #603: the traced forward IS the exact executor — a
    /// `forward_capturing_trace` stream produces bit-identical logits and
    /// hidden state to a plain `forward` stream, and its taps are bounded
    /// to the declared layer indices: residuals once per declared layer,
    /// q/k/v rows at the current position, and per-head attention weights
    /// that are softmax-normalized over the prefix (each head's captured
    /// row sums to 1 and has `pos + 1` entries).
    #[test]
    fn forward_capturing_trace_matches_plain_forward_with_bounded_taps() {
        let model = tiny_llama();
        let tokens = [1usize, 3, 5, 2, 7];
        let kv_dim = model.cfg.dim * model.cfg.n_kv_heads / model.cfg.n_heads;

        let mut plain = State::new(&model.cfg);
        plain.reset();
        let mut traced = State::new(&model.cfg);
        traced.reset();
        let request = TraceCaptureRequest {
            residual_layers: &[1],
            qkv_layers: &[0],
            attention_layers: &[1],
        };
        for (pos, &token) in tokens.iter().enumerate() {
            model.forward(&mut plain, token, pos, false);

            let mut residuals: Vec<(usize, Vec<f32>)> = Vec::new();
            let mut qkv: Vec<(usize, usize, usize, usize)> = Vec::new();
            let mut attention: Vec<(usize, usize, Vec<f32>)> = Vec::new();
            let mut residual_sink = |layer: usize, x: &[f32]| residuals.push((layer, x.to_vec()));
            let mut qkv_sink = |layer: usize, q: &[f32], k: &[f32], v: &[f32]| {
                qkv.push((layer, q.len(), k.len(), v.len()));
            };
            let mut attention_sink = |layer: usize, head: usize, att: &[f32]| {
                attention.push((layer, head, att.to_vec()))
            };
            model.forward_capturing_trace(
                &mut traced,
                token,
                pos,
                false,
                &request,
                &mut TraceCaptureSinks {
                    residual: &mut residual_sink,
                    qkv: &mut qkv_sink,
                    attention: &mut attention_sink,
                },
            );

            let plain_bits: Vec<u32> = plain.logits.iter().map(|v| v.to_bits()).collect();
            let traced_bits: Vec<u32> = traced.logits.iter().map(|v| v.to_bits()).collect();
            assert_eq!(plain_bits, traced_bits, "logits diverged at pos {pos}");
            let plain_x: Vec<u32> = plain.x.iter().map(|v| v.to_bits()).collect();
            let traced_x: Vec<u32> = traced.x.iter().map(|v| v.to_bits()).collect();
            assert_eq!(plain_x, traced_x, "hidden state diverged at pos {pos}");

            // Bounded taps: exactly the declared layers, once per step.
            assert_eq!(residuals.len(), 1);
            assert_eq!(residuals[0].0, 1);
            assert_eq!(residuals[0].1.len(), model.cfg.dim);
            assert_eq!(qkv, vec![(0, model.cfg.dim, kv_dim, kv_dim)]);
            assert_eq!(attention.len(), model.cfg.n_heads);
            for (head, (layer, got_head, att)) in attention.iter().enumerate() {
                assert_eq!(*layer, 1);
                assert_eq!(*got_head, head);
                assert_eq!(att.len(), pos + 1, "prefix-bounded weights");
                let total: f32 = att.iter().sum();
                assert!((total - 1.0).abs() < 1e-5, "head {head} not normalized");
            }
        }
    }

    fn persistent_state_bits(state: &State) -> Vec<u32> {
        // xb/xb2/hb/hb2/q/att are overwrite-only forward scratch and the
        // batched executor intentionally keeps their equivalents in stacked
        // private buffers. x, both KV caches, and logits are the persistent
        // sequence state consumed or exposed after the call.
        [
            &state.x,
            &state.key_cache,
            &state.value_cache,
            &state.logits,
        ]
        .into_iter()
        .flat_map(|values| values.iter().map(|value| value.to_bits()))
        .collect()
    }

    #[test]
    fn cloned_teacher_state_preserves_bits_and_owns_private_storage() {
        let model = tiny_llama();
        let mut template = State::new_bounded(&model.cfg, 4).expect("bounded template state");
        model.forward(&mut template, 1, 0, true);
        model.forward(&mut template, 3, 1, true);

        let template_cid = template.persistent_state_cid();
        let mut clone = template.clone();
        assert_eq!(clone.persistent_state_cid(), template_cid);
        assert_eq!(
            persistent_state_bits(&clone),
            persistent_state_bits(&template)
        );

        clone.key_cache[0] = f32::from_bits(clone.key_cache[0].to_bits() ^ 1);
        assert_eq!(template.persistent_state_cid(), template_cid);
        assert_ne!(clone.persistent_state_cid(), template_cid);

        let mut continuation = template.clone();
        model.forward(&mut continuation, 5, 2, true);
        assert_eq!(template.persistent_state_cid(), template_cid);
        assert_ne!(continuation.persistent_state_cid(), template_cid);
    }

    /// Exact output-row tiling must preserve every persistent sequence-state
    /// bit, including on macOS, at each supported fixed worker count.
    #[test]
    #[allow(clippy::needless_range_loop)]
    fn forward_batch_matches_serial_forward_for_1_2_4_8_workers() {
        use std::num::NonZeroUsize;

        let serial_model = tiny_llama();
        let seqs: [[usize; 4]; 3] = [[1, 3, 5, 2], [4, 0, 7, 8], [2, 9, 1, 6]];
        let len = 4;

        // Serial reference: one State per sequence, stepped token by token.
        let mut serial: Vec<Vec<u32>> = Vec::new();
        let mut sstates: Vec<State> = (0..3).map(|_| State::new(&serial_model.cfg)).collect();
        sstates.iter_mut().for_each(State::reset);
        for pos in 0..len {
            for (b, st) in sstates.iter_mut().enumerate() {
                serial_model.forward(st, seqs[b][pos], pos, true);
                serial.push(persistent_state_bits(st));
            }
        }

        for workers in [1usize, 2, 4, 8] {
            let mut model = tiny_llama();
            model
                .set_execution_config(TeacherExecutionConfig::fixed_workers(
                    NonZeroUsize::new(workers).expect("worker count is nonzero"),
                ))
                .expect("fixed exact executor");
            let mut bstates: Vec<State> = (0..3).map(|_| State::new(&model.cfg)).collect();
            bstates.iter_mut().for_each(State::reset);
            for pos in 0..len {
                let tokens: Vec<usize> = (0..3).map(|b| seqs[b][pos]).collect();
                let positions = vec![pos; 3];
                model.forward_batch(&mut bstates, &tokens, &positions, true);
                for (b, st) in bstates.iter().enumerate() {
                    assert_eq!(
                        persistent_state_bits(st),
                        serial[pos * 3 + b],
                        "state differs with {workers} workers at pos {pos} seq {b}"
                    );
                }
            }
        }
    }

    #[test]
    #[should_panic(expected = "batched teacher forward requires at least one state")]
    fn forward_batch_rejects_an_empty_cohort_before_executor_work() {
        let model = tiny_llama();
        model.forward_batch(&mut [], &[], &[], true);
    }

    #[test]
    #[should_panic(expected = "batched teacher token/state lengths must match")]
    fn forward_batch_rejects_mismatched_cohort_shapes_in_release_builds() {
        let model = tiny_llama();
        let mut states = vec![State::new(&model.cfg)];
        model.forward_batch(&mut states, &[], &[0], true);
    }

    #[test]
    fn bounded_state_matches_full_state_and_rejects_invalid_bounds() {
        let model = tiny_llama();
        assert_eq!(
            State::new_bounded(&model.cfg, 0).err(),
            Some(TeacherStateCapacityError::Zero)
        );
        assert_eq!(
            State::new_bounded(&model.cfg, model.cfg.seq_len + 1).err(),
            Some(TeacherStateCapacityError::ExceedsModel {
                requested: model.cfg.seq_len + 1,
                maximum: model.cfg.seq_len,
            })
        );

        let horizon = 4usize;
        let full_shape = model
            .exact_probe_trace_shape(1, 8, 8)
            .expect("full trace shape");
        let bounded_shape = model
            .exact_probe_trace_shape_bounded(horizon, 1, 8, 8)
            .expect("bounded trace shape");
        assert_eq!(full_shape.sequence_capacity, model.cfg.seq_len);
        assert_eq!(bounded_shape.sequence_capacity, horizon);
        assert!(bounded_shape.persistent_state_words < full_shape.persistent_state_words);
        let mut full = State::new(&model.cfg);
        let mut bounded = State::new_bounded(&model.cfg, horizon).expect("bounded state");
        assert_eq!(bounded.sequence_capacity(), horizon);
        assert_eq!(bounded.att.len(), model.cfg.n_heads * horizon);
        let kv_dim = model.cfg.dim * model.cfg.n_kv_heads / model.cfg.n_heads;
        assert_eq!(
            bounded.key_cache.len(),
            model.cfg.n_layers * horizon * kv_dim
        );
        for pos in 0..horizon {
            model.forward(&mut full, pos + 1, pos, true);
            model.forward(&mut bounded, pos + 1, pos, true);
            assert_eq!(
                full.x
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                bounded
                    .x
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                full.logits
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                bounded
                    .logits
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            );
            for layer in 0..model.cfg.n_layers {
                let full_offset = layer * model.cfg.seq_len * kv_dim;
                let bounded_offset = layer * horizon * kv_dim;
                let words = (pos + 1) * kv_dim;
                assert_eq!(
                    &full.key_cache[full_offset..full_offset + words],
                    &bounded.key_cache[bounded_offset..bounded_offset + words]
                );
                assert_eq!(
                    &full.value_cache[full_offset..full_offset + words],
                    &bounded.value_cache[bounded_offset..bounded_offset + words]
                );
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn direct_probe_budget_contract_error(
        generation_tokens_per_lane: usize,
        max_wall_seconds: usize,
    ) -> Option<String> {
        if generation_tokens_per_lane > EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_TOKENS
            || !generation_tokens_per_lane.is_power_of_two()
        {
            return Some(format!(
                "R4_PARITY_GEN_TOKENS must be a power of two in 1..={EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_TOKENS}; got {generation_tokens_per_lane}"
            ));
        }
        if max_wall_seconds > 28_800 {
            return Some(format!(
                "R4_PARITY_MAX_WALL_SECS must be in 1..=28800; got {max_wall_seconds}"
            ));
        }
        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn direct_probe_budget_bounds_match_the_binding_bdd_owner() {
        for tokens in [1, 2, 4, 8] {
            assert_eq!(direct_probe_budget_contract_error(tokens, 28_800), None);
        }
        for tokens in [3, 9, 128] {
            assert!(direct_probe_budget_contract_error(tokens, 28_800)
                .is_some_and(|reason| reason.contains("R4_PARITY_GEN_TOKENS")));
        }
        assert!(direct_probe_budget_contract_error(8, 28_801)
            .is_some_and(|reason| reason.contains("R4_PARITY_MAX_WALL_SECS")));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn direct_probe_revalidates_production_generation_immediately_before_teacher_load() {
        let source = include_str!("lib.rs");
        let start = source
            .rfind("fn live_exact_multicore_probe_emits_json_and_preserves_bits()")
            .expect("direct probe exists");
        let body = &source[start..];
        let load = body
            .find(
                "let mut oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(",
            )
            .expect("teacher load exists");
        let validations = body[..load]
            .match_indices("exact_probe::validate_teacher_free_preflight")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        assert_eq!(validations.len(), 2);
        let final_validation = *validations.last().expect("final validation");
        let final_interval = &body[final_validation..load];
        assert!(final_interval.contains("current_preflight != preflight"));
        assert!(final_interval.contains("production generation changed"));
    }

    /// Cheap live proof harness for #932. It loads source weights once, times
    /// identical eight-stream work at four and all available exact workers,
    /// selects the lowest projected wall time, compares every output and
    /// persistent-state bit, flushes JSONL progress, and atomically publishes
    /// typed admission evidence. Missing fixtures are a truthful unavailable
    /// result, never a skip.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    #[ignore = "requires the pinned live SmolLM2 source fixture"]
    fn live_exact_multicore_probe_emits_json_and_preserves_bits() {
        use std::fs::File;
        use std::io::Write;
        use std::num::NonZeroUsize;
        use std::path::PathBuf;
        use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
        use std::sync::{Arc, Mutex};
        use std::time::{Duration, Instant};

        fn try_emit_probe_record(
            events: &Arc<Mutex<File>>,
            record: &serde_json::Value,
            durable: bool,
        ) -> std::io::Result<()> {
            {
                let mut file = events
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                serde_json::to_writer(&mut *file, record)
                    .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
                file.write_all(b"\n")?;
                file.flush()?;
                if durable {
                    file.sync_all()?;
                }
            }
            let stderr = std::io::stderr();
            let mut output = stderr.lock();
            serde_json::to_writer(&mut output, record)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            output.write_all(b"\n")?;
            output.flush()?;
            Ok(())
        }

        fn emit_probe_record(events: &Arc<Mutex<File>>, record: &serde_json::Value) {
            try_emit_probe_record(events, record, false)
                .expect("write and flush exact probe JSONL record");
        }

        fn write_probe_state_atomic(
            path: &std::path::Path,
            state: &serde_json::Value,
        ) -> std::io::Result<()> {
            let parent = exact_probe::normalized_report_parent(path);
            let name = path
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("exact-multicore-probe.json");
            let temporary = parent.join(format!(".{name}.{}.state.tmp", std::process::id()));
            let mut file = File::create(&temporary)?;
            serde_json::to_writer_pretty(&mut file, state)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            file.write_all(b"\n")?;
            file.flush()?;
            file.sync_all()?;
            std::fs::rename(&temporary, path)?;
            File::open(parent)?.sync_all()?;
            Ok(())
        }

        #[derive(Clone, Copy)]
        enum ProbeTerminalKind {
            Aborted,
            Unavailable,
            Refused,
            Failed,
        }

        impl ProbeTerminalKind {
            fn event(self) -> &'static str {
                match self {
                    Self::Aborted => "ABORTED",
                    Self::Unavailable => "UNAVAILABLE",
                    Self::Refused => "NOT_RUN",
                    Self::Failed => "FAIL",
                }
            }

            fn status(self) -> &'static str {
                match self {
                    Self::Aborted => "ABORTED",
                    Self::Unavailable => "UNAVAILABLE",
                    Self::Refused => "REFUSE_FULL_RUN",
                    Self::Failed => "FAIL",
                }
            }
        }

        fn terminal_probe(
            events: &Arc<Mutex<File>>,
            report_path: &std::path::Path,
            elapsed_seconds: f64,
            reason: &str,
            kind: ProbeTerminalKind,
            overall_heartbeat: &mut ProbeOverallHeartbeat,
        ) -> ! {
            let heartbeat_failure = overall_heartbeat.stop_and_join().err();
            let terminal_reason = heartbeat_failure.as_ref().map_or_else(
                || reason.to_owned(),
                |heartbeat_reason| {
                    format!("{reason}; terminal heartbeat join failed: {heartbeat_reason}")
                },
            );
            let state = serde_json::json!({
                "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                "record": "EXACT_MULTICORE_PROBE_STATE",
                "event": kind.event(),
                "status": kind.status(),
                "qualifies_full_run": false,
                "probe_wall_ceiling_seconds": EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS,
                "probe_deadline_policy": EXACT_MULTICORE_PROBE_DEADLINE_POLICY,
                "probe_elapsed_seconds": elapsed_seconds,
                "reason": terminal_reason,
            });
            let state_error = write_probe_state_atomic(report_path, &state).err();
            emit_probe_record(events, &state);
            if let Some(error) = state_error {
                panic!(
                    "NOT_RUN / {}: {terminal_reason}; durable terminal state write failed: {error}",
                    kind.status()
                );
            }
            panic!("NOT_RUN / {}: {terminal_reason}", kind.status());
        }

        fn abort_probe(
            events: &Arc<Mutex<File>>,
            report_path: &std::path::Path,
            elapsed_seconds: f64,
            reason: &str,
            overall_heartbeat: &mut ProbeOverallHeartbeat,
        ) -> ! {
            terminal_probe(
                events,
                report_path,
                elapsed_seconds,
                reason,
                ProbeTerminalKind::Aborted,
                overall_heartbeat,
            )
        }

        fn unavailable_probe(
            events: &Arc<Mutex<File>>,
            report_path: &std::path::Path,
            elapsed_seconds: f64,
            reason: &str,
            overall_heartbeat: &mut ProbeOverallHeartbeat,
        ) -> ! {
            terminal_probe(
                events,
                report_path,
                elapsed_seconds,
                reason,
                ProbeTerminalKind::Unavailable,
                overall_heartbeat,
            )
        }

        fn refuse_probe(
            events: &Arc<Mutex<File>>,
            report_path: &std::path::Path,
            elapsed_seconds: f64,
            reason: &str,
            overall_heartbeat: &mut ProbeOverallHeartbeat,
        ) -> ! {
            terminal_probe(
                events,
                report_path,
                elapsed_seconds,
                reason,
                ProbeTerminalKind::Refused,
                overall_heartbeat,
            )
        }

        fn fail_probe(
            events: &Arc<Mutex<File>>,
            report_path: &std::path::Path,
            elapsed_seconds: f64,
            reason: &str,
            overall_heartbeat: &mut ProbeOverallHeartbeat,
        ) -> ! {
            terminal_probe(
                events,
                report_path,
                elapsed_seconds,
                reason,
                ProbeTerminalKind::Failed,
                overall_heartbeat,
            )
        }

        #[derive(Clone, Copy, Debug, serde::Serialize)]
        struct ProbeProcessSample {
            cpu_time_seconds: f64,
            resident_set_bytes: u64,
        }

        #[cfg(target_os = "macos")]
        fn process_sample() -> Result<ProbeProcessSample, String> {
            fn parse_cpu_time(raw: &str) -> Result<f64, String> {
                let (days, clock) = match raw.split_once('-') {
                    Some((days, clock)) => (
                        days.parse::<u64>()
                            .map_err(|error| format!("ps TIME days {days:?}: {error}"))?,
                        clock,
                    ),
                    None => (0, raw),
                };
                let fields: Vec<&str> = clock.split(':').collect();
                let (hours, minutes, seconds) = match fields.as_slice() {
                    [minutes, seconds] => (0, *minutes, *seconds),
                    [hours, minutes, seconds] => (
                        hours
                            .parse::<u64>()
                            .map_err(|error| format!("ps TIME hours {hours:?}: {error}"))?,
                        *minutes,
                        *seconds,
                    ),
                    _ => return Err(format!("ps TIME had unexpected shape {raw:?}")),
                };
                let minutes = minutes
                    .parse::<u64>()
                    .map_err(|error| format!("ps TIME minutes {minutes:?}: {error}"))?;
                let seconds = seconds
                    .parse::<f64>()
                    .map_err(|error| format!("ps TIME seconds {seconds:?}: {error}"))?;
                let total = days as f64 * 86_400.0
                    + hours as f64 * 3_600.0
                    + minutes as f64 * 60.0
                    + seconds;
                (total.is_finite() && total >= 0.0)
                    .then_some(total)
                    .ok_or_else(|| format!("ps TIME was invalid: {raw:?}"))
            }

            let process_id = std::process::id().to_string();
            let output = std::process::Command::new("/bin/ps")
                .args(["-o", "rss=", "-o", "time=", "-p", &process_id])
                .output()
                .map_err(|error| format!("ps process sample: {error}"))?;
            if !output.status.success() {
                return Err(format!("ps process sample exited {}", output.status));
            }
            let text = String::from_utf8(output.stdout)
                .map_err(|error| format!("ps returned non-UTF-8 output: {error}"))?;
            let mut fields = text.split_whitespace();
            let resident_kib = fields
                .next()
                .ok_or_else(|| "ps omitted RSS".to_owned())?
                .parse::<u64>()
                .map_err(|error| format!("ps RSS parse: {error}"))?;
            let cpu_time_seconds = parse_cpu_time(
                fields
                    .next()
                    .ok_or_else(|| "ps omitted process CPU time".to_owned())?,
            )?;
            if fields.next().is_some() {
                return Err(format!("ps returned unexpected fields: {text:?}"));
            }
            let resident_set_bytes = resident_kib
                .checked_mul(1024)
                .ok_or_else(|| "ps RSS byte conversion overflow".to_owned())?;
            Ok(ProbeProcessSample {
                cpu_time_seconds,
                resident_set_bytes,
            })
        }

        #[cfg(not(target_os = "macos"))]
        fn process_sample() -> Result<ProbeProcessSample, String> {
            Err(format!(
                "safe exact-probe process sampler is not implemented for {}",
                std::env::consts::OS
            ))
        }

        fn completed_resources(
            start: &Result<ProbeProcessSample, String>,
            end: &Result<ProbeProcessSample, String>,
            max_sampled_rss_bytes: u64,
            elapsed_seconds: f64,
            measurement_scope: &str,
            cpu_time_consumed_override: Option<f64>,
        ) -> ExactMulticoreProbeResources {
            match (start, end) {
                (Ok(start), Ok(end))
                    if end.cpu_time_seconds >= start.cpu_time_seconds
                        && elapsed_seconds.is_finite()
                        && elapsed_seconds > 0.0 =>
                {
                    let max_sampled_rss_bytes = max_sampled_rss_bytes
                        .max(start.resident_set_bytes)
                        .max(end.resident_set_bytes);
                    let cpu_time_consumed_seconds = cpu_time_consumed_override
                        .unwrap_or(end.cpu_time_seconds - start.cpu_time_seconds);
                    let mean_cpu_core_equivalents = cpu_time_consumed_seconds / elapsed_seconds;
                    ExactMulticoreProbeResources {
                        status: "PARTIAL".to_owned(),
                        measurement_scope: measurement_scope.to_owned(),
                        cpu_time_start_seconds: Some(start.cpu_time_seconds),
                        cpu_time_end_seconds: Some(end.cpu_time_seconds),
                        cpu_time_consumed_seconds: Some(cpu_time_consumed_seconds),
                        current_rss_bytes: Some(end.resident_set_bytes),
                        max_sampled_rss_bytes: Some(max_sampled_rss_bytes),
                        peak_rss_bytes: None,
                        mean_cpu_core_equivalents: Some(mean_cpu_core_equivalents),
                        mean_cpu_percent: Some(mean_cpu_core_equivalents * 100.0),
                        reason: Some(
                            "safe macOS ps sampling exposes current/max-sampled RSS but not an OS-maintained process peak RSS"
                                .to_owned(),
                        ),
                    }
                }
                (Ok(_), Ok(_)) => ExactMulticoreProbeResources::unavailable(
                    "process CPU time moved backwards between ps samples",
                ),
                (Err(start), Err(end)) => ExactMulticoreProbeResources::unavailable(format!(
                    "start sample: {start}; end sample: {end}"
                )),
                (Err(reason), _) => {
                    ExactMulticoreProbeResources::unavailable(format!("start sample: {reason}"))
                }
                (_, Err(reason)) => {
                    ExactMulticoreProbeResources::unavailable(format!("end sample: {reason}"))
                }
            }
        }

        fn process_sample_value(sample: &Result<ProbeProcessSample, String>) -> serde_json::Value {
            match sample {
                Ok(sample) => serde_json::json!({
                    "status": "AVAILABLE",
                    "sample": sample,
                }),
                Err(reason) => serde_json::json!({
                    "status": "UNAVAILABLE",
                    "reason": reason,
                }),
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        struct ProbeTracePoint {
            logits_bits: Vec<u32>,
            greedy_token: u32,
            top_tokens: Vec<u32>,
            persistent_state_bits: Vec<u32>,
        }

        struct ProbeRunMeasurement {
            workers: usize,
            prestart: ExactMulticoreProbePrestart,
            elapsed_ms: u64,
            elapsed_seconds: f64,
            aggregate_forwards_per_second: f64,
            equal_to_reference: bool,
            snapshot: TeacherExecutionSnapshot,
            output_trace_cid: String,
            resources: ExactMulticoreProbeResources,
            forward_plan: ExactForwardPlan,
            trace_shape: ExactMulticoreProbeTraceShape,
        }

        fn canonical_top_tokens(logits: &[f32], k: usize) -> Vec<u32> {
            let mut best = Vec::with_capacity(k.min(logits.len()));
            for token in 0..logits.len() {
                let insertion = best.iter().position(|&incumbent| {
                    logits[token].total_cmp(&logits[incumbent as usize]).is_gt()
                        || (logits[token].to_bits() == logits[incumbent as usize].to_bits()
                            && token < incumbent as usize)
                });
                if let Some(index) = insertion {
                    best.insert(index, u32::try_from(token).unwrap_or(u32::MAX));
                } else if best.len() < k {
                    best.push(u32::try_from(token).unwrap_or(u32::MAX));
                }
                best.truncate(k);
            }
            best
        }

        fn capture_trace_point(state: &State) -> ProbeTracePoint {
            let top_tokens = canonical_top_tokens(&state.logits, 8);
            ProbeTracePoint {
                logits_bits: state.logits.iter().map(|value| value.to_bits()).collect(),
                greedy_token: top_tokens.first().copied().unwrap_or(0),
                top_tokens,
                persistent_state_bits: persistent_state_bits(state),
            }
        }

        fn output_trace_cid(trace: &[Vec<ProbeTracePoint>]) -> String {
            let mut hasher = blake3::Hasher::new();
            for (position, streams) in trace.iter().enumerate() {
                hasher.update(&u64::try_from(position).unwrap_or(u64::MAX).to_le_bytes());
                hasher.update(
                    &u64::try_from(streams.len())
                        .unwrap_or(u64::MAX)
                        .to_le_bytes(),
                );
                for state in streams {
                    hasher.update(
                        &u64::try_from(state.logits_bits.len())
                            .unwrap_or(u64::MAX)
                            .to_le_bytes(),
                    );
                    for bits in &state.logits_bits {
                        hasher.update(&bits.to_le_bytes());
                    }
                    hasher.update(&state.greedy_token.to_le_bytes());
                    hasher.update(
                        &u64::try_from(state.top_tokens.len())
                            .unwrap_or(u64::MAX)
                            .to_le_bytes(),
                    );
                    for token in &state.top_tokens {
                        hasher.update(&token.to_le_bytes());
                    }
                    hasher.update(
                        &u64::try_from(state.persistent_state_bits.len())
                            .unwrap_or(u64::MAX)
                            .to_le_bytes(),
                    );
                    for bits in &state.persistent_state_bits {
                        hasher.update(&bits.to_le_bytes());
                    }
                }
            }
            format!("blake3:{}", hasher.finalize().to_hex())
        }

        #[derive(Default)]
        struct ProbeOverallPhase(Mutex<(String, Option<usize>)>);

        impl ProbeOverallPhase {
            fn set(&self, phase: &str, workers: Option<usize>) {
                *self
                    .0
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    (phase.to_owned(), workers);
            }

            fn get(&self) -> (String, Option<usize>) {
                self.0
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone()
            }
        }

        struct ProbeOverallHeartbeat {
            stop: Arc<AtomicBool>,
            handle: Option<std::thread::JoinHandle<()>>,
        }

        impl ProbeOverallHeartbeat {
            fn idle() -> Self {
                Self {
                    stop: Arc::new(AtomicBool::new(false)),
                    handle: None,
                }
            }

            fn stop_and_join(&mut self) -> Result<(), String> {
                self.stop.store(true, Ordering::Release);
                let Some(handle) = self.handle.take() else {
                    return Ok(());
                };
                handle.thread().unpark();
                handle
                    .join()
                    .map_err(|_| "overall exact-probe heartbeat thread panicked".to_owned())
            }
        }

        impl Drop for ProbeOverallHeartbeat {
            fn drop(&mut self) {
                self.stop.store(true, Ordering::Release);
                if let Some(handle) = self.handle.take() {
                    handle.thread().unpark();
                    let _ = handle.join();
                }
            }
        }

        let report_path = exact_probe::resolve_direct_probe_path(
            std::env::var_os("R4_EXACT_PROBE_REPORT")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from("target/teacher-parity/exact-multicore-probe.json")
                }),
        );
        if report_path.as_os_str().is_empty() {
            let state = serde_json::json!({
                "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                "record": "EXACT_MULTICORE_PROBE_STATE",
                "event": "NOT_RUN",
                "status": "REFUSE_FULL_RUN",
                "qualifies_full_run": false,
                "reason": "R4_EXACT_PROBE_REPORT must be a nonempty path",
            });
            eprintln!("{state}");
            panic!("NOT_RUN / REFUSE_FULL_RUN: R4_EXACT_PROBE_REPORT must be a nonempty path");
        }
        let report_parent = exact_probe::normalized_report_parent(&report_path);
        std::fs::create_dir_all(report_parent).unwrap_or_else(|error| {
            let state = serde_json::json!({
                "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                "record": "EXACT_MULTICORE_PROBE_STATE",
                "event": "UNAVAILABLE",
                "status": "UNAVAILABLE",
                "qualifies_full_run": false,
                "reason": format!("create probe report directory {}: {error}", report_parent.display()),
            });
            eprintln!("{state}");
            panic!("UNAVAILABLE: cannot create exact probe report directory: {error}");
        });
        let report_stem = report_path
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("exact-multicore-probe");
        let events_path = report_parent.join(format!("{report_stem}.events.jsonl"));
        let events_file = File::create(&events_path).unwrap_or_else(|error| {
            let state = serde_json::json!({
                "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                "record": "EXACT_MULTICORE_PROBE_STATE",
                "event": "UNAVAILABLE",
                "status": "UNAVAILABLE",
                "qualifies_full_run": false,
                "reason": format!("create durable probe JSONL {}: {error}", events_path.display()),
            });
            let _ = write_probe_state_atomic(&report_path, &state);
            eprintln!("{state}");
            panic!("UNAVAILABLE: cannot create exact probe JSONL: {error}");
        });
        let events = Arc::new(Mutex::new(events_file));
        let mut overall_heartbeat = ProbeOverallHeartbeat::idle();
        let probe_started = Instant::now();
        let overall_process_start = process_sample();
        if let Err(error) = write_probe_state_atomic(
            &report_path,
            &serde_json::json!({
                "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                "record": "EXACT_MULTICORE_PROBE_STATE",
                "event": "RUNNING",
                "status": "NOT_QUALIFIED",
                "qualifies_full_run": false,
                "probe_wall_ceiling_seconds": EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS,
                "probe_deadline_policy": EXACT_MULTICORE_PROBE_DEADLINE_POLICY,
            }),
        ) {
            let reason = format!("publish initial RUNNING probe state: {error}");
            unavailable_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                &reason,
                &mut overall_heartbeat,
            );
        }
        let progress_every_seconds = match std::env::var("R4_PARITY_PROGRESS_EVERY_SECS") {
            Ok(raw) => raw
                .parse::<u64>()
                .ok()
                .filter(|value| *value > 0)
                .unwrap_or_else(|| {
                    let reason =
                        format!("R4_PARITY_PROGRESS_EVERY_SECS={raw:?} must be a positive integer");
                    refuse_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                }),
            Err(std::env::VarError::NotPresent) => 10,
            Err(std::env::VarError::NotUnicode(_)) => refuse_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                "R4_PARITY_PROGRESS_EVERY_SECS is not valid Unicode",
                &mut overall_heartbeat,
            ),
        };
        let progress_every = Duration::from_secs(progress_every_seconds);
        let overall_phase = Arc::new(ProbeOverallPhase::default());
        overall_phase.set("TEACHER_FREE_PREFLIGHT", None);
        let overall_max_sampled_rss = Arc::new(AtomicU64::new(
            overall_process_start
                .as_ref()
                .map_or(0, |sample| sample.resident_set_bytes),
        ));
        let overall_heartbeat_stop = Arc::new(AtomicBool::new(false));
        overall_heartbeat.stop = Arc::clone(&overall_heartbeat_stop);
        {
            let worker_stop = Arc::clone(&overall_heartbeat_stop);
            let phase = Arc::clone(&overall_phase);
            let heartbeat_events = Arc::clone(&events);
            let max_rss = Arc::clone(&overall_max_sampled_rss);
            let handle = match std::thread::Builder::new()
                .name("r4-exact-probe-overall-heartbeat".to_owned())
                .spawn(move || loop {
                    std::thread::park_timeout(progress_every);
                    if worker_stop.load(Ordering::Acquire) {
                        break;
                    }
                    let elapsed_seconds = probe_started.elapsed().as_secs_f64();
                    let process = process_sample();
                    if let Ok(sample) = &process {
                        max_rss.fetch_max(sample.resident_set_bytes, Ordering::AcqRel);
                    }
                    let (phase, current_workers) = phase.get();
                    emit_probe_record(
                        &heartbeat_events,
                        &serde_json::json!({
                            "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                            "record": "EXACT_MULTICORE_PROBE",
                            "event": "OVERALL_PROGRESS",
                            "phase": phase,
                            "current_workers": current_workers,
                            "elapsed_seconds": elapsed_seconds,
                            "deadline_seconds": EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS,
                            "deadline_policy": EXACT_MULTICORE_PROBE_DEADLINE_POLICY,
                            "deadline_remaining_seconds": (EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS as f64 - elapsed_seconds).max(0.0),
                            "process": process_sample_value(&process),
                        }),
                    );
                })
            {
                Ok(handle) => handle,
                Err(error) => {
                    let reason = format!("spawn overall exact probe heartbeat: {error}");
                    unavailable_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                }
            };
            overall_heartbeat.handle = Some(handle);
        }

        let source = exact_probe::resolve_direct_probe_path(
            std::env::var_os("R4_PARITY_SOURCE")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(".uor-models/sources/smollm2-135m-instruct")),
        );
        let bundle = exact_probe::resolve_direct_probe_path(
            std::env::var_os("R4_PARITY_BUNDLE")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(".uor-models/compiled/smollm2-135m-instruct")),
        );
        let preflight_path = exact_probe::resolve_direct_probe_path(
            std::env::var_os("R4_PARITY_PREFLIGHT_REPORT")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from("target/teacher-parity/teacher-free-preflight.json")
                }),
        );
        let preflight =
            match exact_probe::validate_teacher_free_preflight(&preflight_path, &source, &bundle) {
                Ok(preflight) => preflight,
                Err(exact_probe::TeacherFreePreflightAdmissionError::Unavailable(reason)) => {
                    unavailable_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                }
                Err(exact_probe::TeacherFreePreflightAdmissionError::Refused(reason)) => {
                    refuse_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                }
                Err(exact_probe::TeacherFreePreflightAdmissionError::Failed(reason)) => fail_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    &reason,
                    &mut overall_heartbeat,
                ),
            };
        emit_probe_record(
            &events,
            &serde_json::json!({
                "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                "record": "EXACT_MULTICORE_PROBE",
                "event": "TEACHER_FREE_PREFLIGHT_VALIDATED",
                "preflight_report_path": &preflight_path,
                "preflight": &preflight,
                "teacher_source_opened": false,
                "teacher_forwards": 0,
            }),
        );
        overall_phase.set("LOAD", None);
        emit_probe_record(
            &events,
            &serde_json::json!({
                "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                "record": "EXACT_MULTICORE_PROBE",
                "event": "LOAD_START",
                "source": &source,
                "report_path": &report_path,
                "events_path": &events_path,
                "probe_wall_ceiling_seconds": EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS,
                "probe_deadline_policy": EXACT_MULTICORE_PROBE_DEADLINE_POLICY,
                "process_start": process_sample_value(&overall_process_start),
            }),
        );
        let backend = exact_backend_report();
        if backend.arithmetic_owner != "uor-matmul exact GEMM" {
            refuse_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                "the observation BLAS exception does not execute the exact uor-matmul owner",
                &mut overall_heartbeat,
            );
        }
        if !source.join("config.json").is_file() {
            let reason = format!(
                "live teacher source fixture does not exist at {}",
                source.display()
            );
            unavailable_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                &reason,
                &mut overall_heartbeat,
            );
        }
        let positions = match std::env::var("R4_EXACT_PROBE_POSITIONS") {
            Ok(raw) => raw.parse::<usize>().unwrap_or_else(|error| {
                let reason = format!("R4_EXACT_PROBE_POSITIONS={raw:?} is invalid: {error}");
                refuse_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    &reason,
                    &mut overall_heartbeat,
                )
            }),
            Err(std::env::VarError::NotPresent) => 1,
            Err(std::env::VarError::NotUnicode(_)) => refuse_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                "R4_EXACT_PROBE_POSITIONS is not valid Unicode",
                &mut overall_heartbeat,
            ),
        };
        if !(1..=8).contains(&positions) {
            refuse_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                "R4_EXACT_PROBE_POSITIONS must be in 1..=8",
                &mut overall_heartbeat,
            );
        }
        let available = std::thread::available_parallelism()
            .unwrap_or(NonZeroUsize::MIN)
            .get();
        if available < 4 {
            let reason = format!(
                "the adaptive exact probe requires at least 4 available workers; host reports {available}"
            );
            unavailable_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                &reason,
                &mut overall_heartbeat,
            );
        }
        let mut suite_budget = |name: &str, default: usize| {
            let value = match std::env::var(name) {
                Ok(raw) => raw.parse::<usize>().unwrap_or_else(|error| {
                    let reason = format!("{name}={raw:?} is invalid: {error}");
                    refuse_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                }),
                Err(std::env::VarError::NotPresent) => default,
                Err(std::env::VarError::NotUnicode(_)) => {
                    let reason = format!("{name} is not valid Unicode");
                    refuse_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                }
            };
            if value == 0 {
                let reason = format!("{name} must be greater than zero");
                refuse_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    &reason,
                    &mut overall_heartbeat,
                );
            }
            value
        };
        let tiles_per_worker = suite_budget("R4_PARITY_BATCH_PER_WORKER", 4);
        let transcript_logical_forwards = suite_budget("R4_PARITY_POSITIONS", 256)
            .min(EXACT_MULTICORE_PROBE_REGISTERED_TRANSCRIPT_FORWARDS);
        let generation_tokens_per_lane = suite_budget(
            "R4_PARITY_GEN_TOKENS",
            EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_TOKENS,
        );
        let generation_lanes = suite_budget("R4_PARITY_STREAMS", 8);
        if generation_lanes != 8 {
            let reason = format!(
                "the adaptive probe and optimized suite require exactly 8 independent lanes; R4_PARITY_STREAMS={generation_lanes}"
            );
            refuse_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                &reason,
                &mut overall_heartbeat,
            );
        }
        let configured_max_wall = suite_budget("R4_PARITY_MAX_WALL_SECS", 28_800);
        if let Some(reason) =
            direct_probe_budget_contract_error(generation_tokens_per_lane, configured_max_wall)
        {
            refuse_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                &reason,
                &mut overall_heartbeat,
            );
        }
        let configured_max_wall = u64::try_from(configured_max_wall)
            .expect("validated maximum wall seconds fit into u64");
        let mut configured_suite_work = ExactMulticoreProbeWork {
            transcript_logical_forwards,
            generation_tokens_per_lane,
            generation_lanes,
            logical_forwards: 0,
            transcript_physical_batches: 0,
            generation_physical_batches: 0,
            physical_batches: 0,
            max_sequence_position: EXACT_MULTICORE_PROBE_REGISTERED_MAX_SEQUENCE_POSITION,
            state_sequence_capacity: EXACT_MULTICORE_PROBE_REGISTERED_STATE_SEQUENCE_CAPACITY,
        };
        configured_suite_work.logical_forwards = configured_suite_work.derived_logical_forwards();
        configured_suite_work.transcript_physical_batches =
            configured_suite_work.derived_transcript_physical_batches();
        configured_suite_work.generation_physical_batches = generation_tokens_per_lane;
        configured_suite_work.physical_batches = configured_suite_work.derived_physical_batches();
        let probe_context_ceiling_tokens =
            configured_suite_work.derived_probe_context_ceiling_tokens();
        let probe_position_indices = vec![probe_context_ceiling_tokens - 1; positions];
        let streams = 8usize;
        // Close the interval between initial teacher-free admission and the
        // first teacher-weight read. Rehash the preflight, compiled inputs,
        // and complete production generation immediately before load, and
        // require byte-for-byte admission identity equality with the token we
        // validated above.
        let current_preflight =
            match exact_probe::validate_teacher_free_preflight(&preflight_path, &source, &bundle) {
                Ok(current) => current,
                Err(exact_probe::TeacherFreePreflightAdmissionError::Unavailable(reason)) => {
                    unavailable_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                }
                Err(exact_probe::TeacherFreePreflightAdmissionError::Refused(reason)) => {
                    refuse_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                }
                Err(exact_probe::TeacherFreePreflightAdmissionError::Failed(reason)) => fail_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    &reason,
                    &mut overall_heartbeat,
                ),
            };
        if current_preflight != preflight {
            refuse_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                "teacher-free preflight or production generation changed between admission and teacher load",
                &mut overall_heartbeat,
            );
        }
        let mut oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
            &source,
            probe_context_ceiling_tokens,
            TeacherExecutionConfig::sequential(),
        )
        .unwrap_or_else(|error| {
            let reason = format!("live teacher source fixture could not load: {error}");
            unavailable_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                &reason,
                &mut overall_heartbeat,
            )
        });
        let config_bytes = std::fs::read(source.join("config.json")).unwrap_or_else(|error| {
            let reason = format!("live teacher config identity is unavailable: {error}");
            unavailable_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                &reason,
                &mut overall_heartbeat,
            )
        });
        let source_identity = ExactMulticoreProbeSource {
            model_kappa: oracle.kappa(),
            config_cid: format!("blake3:{}", blake3::hash(&config_bytes).to_hex()),
            source_bytes: u64::try_from(oracle.source_bytes()).unwrap_or(u64::MAX),
        };
        let host_identity = exact_probe_host_identity();
        if host_identity.available_parallelism != available
            || host_identity.cpu_model.is_none()
            || host_identity.physical_core_count.is_none()
        {
            let reason = format!(
                "exact probe host model/capacity identity is incomplete or changed: {:?}",
                host_identity.topology_unavailable_reason
            );
            unavailable_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                &reason,
                &mut overall_heartbeat,
            );
        }
        emit_probe_record(
            &events,
            &serde_json::json!({
                "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                "record": "EXACT_MULTICORE_PROBE",
                "event": "LOAD_COMPLETE",
                "source_identity": &source_identity,
                "host": &host_identity,
            }),
        );
        overall_phase.set("BETWEEN_CONFIGS", None);
        let vocab = oracle.cfg().vocab;
        let reference_workers = available;
        let mut reference_trace: Option<Vec<Vec<ProbeTracePoint>>> = None;
        let mut runs = Vec::new();
        let mut exact_equality = true;

        // Measure the likely fastest point first. Four workers remains the
        // bounded comparison because the M1 performance-core cluster can beat
        // mixed performance/efficiency scheduling. Both candidates execute
        // identical eight-stream work; a four-core host deduplicates them.
        let mut worker_counts = Vec::new();
        for workers in [available, 4usize] {
            if !worker_counts.contains(&workers) {
                worker_counts.push(workers);
            }
        }
        for workers in worker_counts {
            if probe_started.elapsed()
                >= Duration::from_secs(EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS)
            {
                abort_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    "the cheap exact multicore probe reached its 60-minute wall ceiling before starting the next configuration",
                    &mut overall_heartbeat,
                );
            }
            overall_phase.set("PRESTART", Some(workers));
            emit_probe_record(
                &events,
                &serde_json::json!({
                    "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                    "record": "EXACT_MULTICORE_PROBE",
                    "event": "PRESTART_BEGIN",
                    "workers": workers,
                    "tiles_per_worker": tiles_per_worker,
                    "streams": streams,
                    "batch_width": streams,
                    "model_forward": false,
                    "excluded_from_measurement": true,
                    "backend": &backend,
                }),
            );
            oracle
                .set_execution_config(
                    TeacherExecutionConfig::fixed_workers(
                        NonZeroUsize::new(workers).expect("worker count is nonzero"),
                    )
                    .with_tiles_per_worker(
                        NonZeroUsize::new(tiles_per_worker).expect("tiles per worker is nonzero"),
                    ),
                )
                .unwrap_or_else(|error| {
                    let reason = format!(
                        "construct fixed {workers}-worker exact executor for adaptive candidate: {error}"
                    );
                    unavailable_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                });
            let prestart = oracle
                .prepare_exact_execution(streams)
                .map(|evidence| ExactMulticoreProbePrestart {
                    elapsed_seconds: evidence.elapsed_seconds,
                    workers_observed: evidence.workers_observed,
                    batch_width: evidence.batch_width,
                    backend_exercised: evidence.backend_exercised,
                    workspace_capacity_bytes: evidence.workspace_capacity_bytes,
                    workspace_growth_events: evidence.workspace_growth_events,
                    workspace_growth_bytes: evidence.workspace_growth_bytes,
                    excluded_from_measurement: true,
                })
                .unwrap_or_else(|error| {
                    let reason = format!(
                        "prestart fixed {workers}-worker pool and exact backend without a model forward: {error}"
                    );
                    unavailable_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                });
            emit_probe_record(
                &events,
                &serde_json::json!({
                    "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                    "record": "EXACT_MULTICORE_PROBE",
                    "event": "PRESTART_COMPLETE",
                    "workers": workers,
                    "streams": streams,
                    "prestart": &prestart,
                    "model_forward": false,
                    "excluded_from_measurement": true,
                }),
            );
            if probe_started.elapsed()
                >= Duration::from_secs(EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS)
            {
                abort_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    "the cheap exact multicore probe reached its 60-minute wall ceiling after excluded pool/backend prestart",
                    &mut overall_heartbeat,
                );
            }
            let live = Arc::new(AtomicTeacherExecutionProgress::default());
            let live_observer = Arc::clone(&live);
            oracle.begin_measured_execution(Arc::new(move |snapshot| {
                live_observer.publish(snapshot);
            }));
            overall_phase.set("MEASURED_FORWARD_AND_TRACE", Some(workers));
            let forward_plan = oracle.exact_forward_plan(streams).unwrap_or_else(|error| {
                let reason = format!("owner exact shared-weight forward plan: {error}");
                unavailable_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    &reason,
                    &mut overall_heartbeat,
                )
            });
            let trace_shape = oracle
                .exact_probe_trace_shape_bounded(
                    probe_context_ceiling_tokens,
                    positions,
                    streams,
                    8,
                )
                .unwrap_or_else(|error| {
                    let reason = format!("owner complete exact probe trace shape: {error}");
                    unavailable_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                });
            let mut states: Vec<State> = (0..streams)
                .map(|_| oracle.new_state_bounded(probe_context_ceiling_tokens))
                .collect::<Result<_, _>>()
                .unwrap_or_else(|error| {
                    let reason = format!("allocate bounded private probe states: {error}");
                    unavailable_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                });
            let process_start = process_sample();
            emit_probe_record(
                &events,
                &serde_json::json!({
                    "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                    "record": "EXACT_MULTICORE_PROBE",
                    "event": "START",
                    "measurement_scope": "EXACT_FORWARD_INTERVALS_ONLY",
                    "workers": workers,
                    "tiles_per_worker": tiles_per_worker,
                    "streams": streams,
                    "batch_width": streams,
                    "positions": positions,
                    "position_indices": &probe_position_indices,
                    "backend": &backend,
                    "process_start": process_sample_value(&process_start),
                }),
            );
            let heartbeat_live = Arc::clone(&live);
            let heartbeat_events = Arc::clone(&events);
            let heartbeat_stop = Arc::new(AtomicBool::new(false));
            let heartbeat_stop_worker = Arc::clone(&heartbeat_stop);
            let max_sampled_rss = Arc::new(AtomicU64::new(
                process_start
                    .as_ref()
                    .map_or(0, |sample| sample.resident_set_bytes),
            ));
            let heartbeat_max_rss = Arc::clone(&max_sampled_rss);
            let heartbeat_started = Instant::now();
            let heartbeat = std::thread::Builder::new()
                .name(format!("r4-exact-probe-heartbeat-{workers}"))
                .spawn(move || loop {
                    std::thread::park_timeout(progress_every);
                    if heartbeat_stop_worker.load(Ordering::Acquire) {
                        break;
                    }
                    let snapshot = heartbeat_live.snapshot();
                    let process = process_sample();
                    if let Ok(sample) = &process {
                        heartbeat_max_rss
                            .fetch_max(sample.resident_set_bytes, Ordering::AcqRel);
                    }
                    emit_probe_record(
                        &heartbeat_events,
                        &serde_json::json!({
                            "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                            "record": "EXACT_MULTICORE_PROBE",
                            "event": "PROGRESS",
                            "elapsed_ms": u64::try_from(heartbeat_started.elapsed().as_millis()).unwrap_or(u64::MAX),
                            "workers": workers,
                            "tiles_per_worker": tiles_per_worker,
                            "streams": streams,
                            "observer_epoch": snapshot.observer_epoch,
                            "forward_calls": snapshot.forward_calls,
                            "streams_started": snapshot.streams_started,
                            "streams_completed": snapshot.streams_completed,
                            "active_streams": snapshot.active_streams,
                            "peak_streams": snapshot.max_active_streams,
                            "matrix_calls": snapshot.matrix_calls,
                            "batched_matrix_calls": snapshot.batched_matrix_calls,
                            "max_matrix_batch_width": snapshot.max_matrix_batch_width,
                            "tiles": snapshot.tiles_completed,
                            "output_cells": snapshot.output_cells_completed,
                            "scalar_terms": snapshot.scalar_terms_completed,
                            "active_workers": snapshot.active_workers,
                            "peak_workers": snapshot.max_active_workers,
                            "forward_peak_workers": snapshot.forward_max_active_workers,
                            "multiworker_forward_calls": snapshot.multiworker_forward_calls,
                            "workspace_growth_events": snapshot.workspace_growth_events,
                            "workspace_growth_bytes": snapshot.workspace_growth_bytes,
                            "process": process_sample_value(&process),
                        }),
                    );
                })
                .unwrap_or_else(|error| {
                    let reason = format!("spawn exact probe heartbeat for {workers} workers: {error}");
                    unavailable_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        &reason,
                        &mut overall_heartbeat,
                    )
                });
            let config_started = Instant::now();
            let mut forward_elapsed = Duration::ZERO;
            let mut forward_cpu_seconds = 0.0f64;
            let mut forward_sample_error: Option<String> = None;
            let mut first_forward_sample: Option<Result<ProbeProcessSample, String>> = None;
            let mut last_forward_sample: Option<Result<ProbeProcessSample, String>> = None;
            let mut output_trace: Vec<Vec<ProbeTracePoint>> = Vec::with_capacity(positions);
            let mut wall_ceiling_reached = false;
            for (step, &pos) in probe_position_indices.iter().enumerate() {
                if probe_started.elapsed()
                    >= Duration::from_secs(EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS)
                {
                    wall_ceiling_reached = true;
                    break;
                }
                let tokens: Vec<usize> = (0..streams)
                    .map(|stream| ((stream + step + 1) % vocab).max(1))
                    .collect();
                let position_batch = vec![pos; streams];
                let interval_start = process_sample();
                if let Ok(sample) = &interval_start {
                    max_sampled_rss.fetch_max(sample.resident_set_bytes, Ordering::AcqRel);
                }
                if first_forward_sample.is_none() {
                    first_forward_sample = Some(interval_start.clone());
                }
                let forward_started = Instant::now();
                oracle.forward_batch_into(&mut states, &tokens, &position_batch);
                forward_elapsed = forward_elapsed.saturating_add(forward_started.elapsed());
                let interval_end = process_sample();
                if let Ok(sample) = &interval_end {
                    max_sampled_rss.fetch_max(sample.resident_set_bytes, Ordering::AcqRel);
                }
                match (&interval_start, &interval_end) {
                    (Ok(start), Ok(end)) if end.cpu_time_seconds >= start.cpu_time_seconds => {
                        forward_cpu_seconds += end.cpu_time_seconds - start.cpu_time_seconds;
                    }
                    (Ok(_), Ok(_)) => {
                        forward_sample_error =
                            Some("process CPU time moved backwards within a forward".to_owned());
                    }
                    (Err(reason), _) | (_, Err(reason)) => {
                        forward_sample_error = Some(reason.clone());
                    }
                }
                last_forward_sample = Some(interval_end);
                output_trace.push(states.iter().map(capture_trace_point).collect());
                if probe_started.elapsed()
                    >= Duration::from_secs(EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS)
                {
                    wall_ceiling_reached = true;
                    break;
                }
            }
            let inclusive_config_elapsed = config_started.elapsed();
            heartbeat_stop.store(true, Ordering::Release);
            heartbeat.thread().unpark();
            if heartbeat.join().is_err() {
                let reason = format!(
                    "exact probe heartbeat for {workers} workers panicked during measurement"
                );
                unavailable_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    &reason,
                    &mut overall_heartbeat,
                );
            }
            if wall_ceiling_reached {
                abort_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    "the cheap exact multicore probe reached its 60-minute wall ceiling; the in-flight forward completed and no additional work was admitted",
                    &mut overall_heartbeat,
                );
            }
            let elapsed_ms = u64::try_from(forward_elapsed.as_millis()).unwrap_or(u64::MAX);
            let aggregate_forwards = streams.saturating_mul(positions);
            let forwards_per_second =
                aggregate_forwards as f64 / forward_elapsed.as_secs_f64().max(f64::MIN_POSITIVE);
            if output_trace.len() != trace_shape.positions {
                fail_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    "exact probe trace omitted one or more measured positions",
                    &mut overall_heartbeat,
                );
            }
            for position_trace in &output_trace {
                if position_trace.len() != trace_shape.streams_per_position {
                    fail_probe(
                        &events,
                        &report_path,
                        probe_started.elapsed().as_secs_f64(),
                        "exact probe trace omitted one or more independent streams",
                        &mut overall_heartbeat,
                    );
                }
                for point in position_trace {
                    if point.logits_bits.len() != trace_shape.logits_per_state
                        || point.persistent_state_bits.len()
                            != trace_shape.persistent_state_words_per_state
                        || point.top_tokens.len() != trace_shape.top_k
                        || point.top_tokens.first().copied() != Some(point.greedy_token)
                    {
                        fail_probe(
                            &events,
                            &report_path,
                            probe_started.elapsed().as_secs_f64(),
                            "exact probe trace shape or canonical token evidence is incomplete",
                            &mut overall_heartbeat,
                        );
                    }
                }
            }
            let equal_to_reference = if let Some(reference) = &reference_trace {
                &output_trace == reference
            } else {
                reference_trace = Some(output_trace.clone());
                true
            };
            let output_trace_cid = output_trace_cid(&output_trace);
            exact_equality &= equal_to_reference;
            let snapshot = oracle.execution_snapshot();
            let mut resource_start = first_forward_sample.unwrap_or_else(|| process_start.clone());
            let resource_end = last_forward_sample.unwrap_or_else(|| process_start.clone());
            if let Some(reason) = forward_sample_error {
                resource_start = Err(reason);
            }
            let resources = completed_resources(
                &resource_start,
                &resource_end,
                max_sampled_rss.load(Ordering::Acquire),
                forward_elapsed.as_secs_f64(),
                "EXACT_FORWARD_INTERVALS_ONLY",
                Some(forward_cpu_seconds),
            );
            emit_probe_record(
                &events,
                &serde_json::json!({
                    "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                    "record": "EXACT_MULTICORE_PROBE",
                    "event": "CONFIG_COMPLETE",
                    "elapsed_ms": elapsed_ms,
                    "elapsed_seconds": forward_elapsed.as_secs_f64(),
                    "inclusive_config_elapsed_seconds": inclusive_config_elapsed.as_secs_f64(),
                    "measurement_scope": "EXACT_FORWARD_INTERVALS_ONLY",
                    "workers": workers,
                    "tiles_per_worker": tiles_per_worker,
                    "streams": streams,
                    "aggregate_forwards_per_second": forwards_per_second,
                    "equal_to_reference": equal_to_reference,
                    "reference_workers": reference_workers,
                    "output_trace_cid": &output_trace_cid,
                    "trace_shape": trace_shape,
                    "forward_plan": forward_plan,
                    "all_workers_active": snapshot.max_active_workers == snapshot.effective_workers,
                    "all_streams_active": snapshot.active_streams == 0 && snapshot.max_active_streams == streams,
                    "snapshot": snapshot,
                    "resources": &resources,
                }),
            );
            runs.push(ProbeRunMeasurement {
                workers,
                prestart,
                elapsed_ms,
                elapsed_seconds: forward_elapsed.as_secs_f64(),
                aggregate_forwards_per_second: forwards_per_second,
                equal_to_reference,
                snapshot,
                output_trace_cid,
                resources,
                forward_plan,
                trace_shape,
            });
            overall_phase.set("BETWEEN_CONFIGS", None);
        }

        let reference_trace_cid = runs
            .first()
            .map(|run| run.output_trace_cid.as_str())
            .unwrap_or_else(|| {
                fail_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    "the adaptive exact probe produced no reference trace",
                    &mut overall_heartbeat,
                )
            });
        let worker4_rate = runs
            .iter()
            .find(|run| run.workers == 4)
            .map(|run| run.aggregate_forwards_per_second)
            .unwrap_or_else(|| {
                fail_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    "the adaptive exact probe omitted its required four-worker candidate",
                    &mut overall_heartbeat,
                )
            });
        exact_equality &= runs
            .iter()
            .all(|run| run.equal_to_reference && run.output_trace_cid == reference_trace_cid);
        let qualification_wall_seconds = configured_max_wall.min(28_800);
        let projection_safety_factor = 1.25f64;
        let run_records: Vec<ExactMulticoreProbeRun> = runs
            .iter()
            .map(|run| {
                let raw_projection = configured_suite_work.physical_batches as f64
                    * run.elapsed_seconds
                    / positions as f64;
                ExactMulticoreProbeRun {
                    workers: run.workers,
                    batch_width: streams,
                    prestart: run.prestart.clone(),
                    elapsed_ms: run.elapsed_ms,
                    elapsed_seconds: run.elapsed_seconds,
                    aggregate_forwards_per_second: run.aggregate_forwards_per_second,
                    relative_throughput_vs_worker4: run.aggregate_forwards_per_second
                        / worker4_rate,
                    equal_to_reference: run.output_trace_cid == reference_trace_cid,
                    all_workers_active: run.snapshot.requested_workers == run.workers
                        && run.snapshot.effective_workers == run.workers
                        && run.snapshot.active_workers == 0
                        && run.snapshot.max_active_workers == run.workers,
                    all_streams_active: run.snapshot.active_streams == 0
                        && run.snapshot.max_active_streams == streams
                        && run.snapshot.streams_started
                            == u64::try_from(streams.saturating_mul(positions)).unwrap_or(u64::MAX)
                        && run.snapshot.streams_completed
                            == u64::try_from(streams.saturating_mul(positions)).unwrap_or(u64::MAX),
                    output_trace_cid: run.output_trace_cid.clone(),
                    trace_shape: run.trace_shape,
                    forward_plan: run.forward_plan,
                    raw_projected_suite_seconds: raw_projection,
                    safety_adjusted_projected_suite_seconds: raw_projection
                        * projection_safety_factor,
                    snapshot: run.snapshot,
                    resources: run.resources.clone(),
                }
            })
            .collect();
        let best = run_records
            .iter()
            .min_by(|left, right| {
                left.safety_adjusted_projected_suite_seconds
                    .total_cmp(&right.safety_adjusted_projected_suite_seconds)
                    .then_with(|| left.workers.cmp(&right.workers))
            })
            .unwrap_or_else(|| {
                fail_probe(
                    &events,
                    &report_path,
                    probe_started.elapsed().as_secs_f64(),
                    "the exact probe produced no fixed-worker measurements",
                    &mut overall_heartbeat,
                )
            });
        let selected_best_config = ExactMulticoreProbeSelection {
            workers: best.workers,
            tiles_per_worker,
            aggregate_forwards_per_second: best.aggregate_forwards_per_second,
            raw_projected_suite_seconds: best.raw_projected_suite_seconds,
            safety_adjusted_projected_suite_seconds: best.safety_adjusted_projected_suite_seconds,
        };
        let configured_execution = selected_best_config.clone();
        let raw_projected_suite_seconds = best.raw_projected_suite_seconds;
        let safety_adjusted_projected_suite_seconds = best.safety_adjusted_projected_suite_seconds;
        let all_workers_active = run_records.iter().all(|run| run.all_workers_active);
        let all_streams_active = run_records.iter().all(|run| run.all_streams_active);
        let registered_binding_work = configured_suite_work.is_registered_binding_work();
        emit_probe_record(
            &events,
            &serde_json::json!({
                "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                "record": "EXACT_MULTICORE_PROBE",
                "event": "SELECTION",
                "selection_policy": EXACT_MULTICORE_PROBE_SELECTION_POLICY,
                "selected": &selected_best_config,
                "candidate_count": run_records.len(),
                "physical_batches": configured_suite_work.physical_batches,
                "logical_forwards": configured_suite_work.logical_forwards,
                "registered_binding_work": registered_binding_work,
            }),
        );
        overall_phase.set("PUBLISH", None);
        let overall_process_end = process_sample();
        let probe_elapsed_seconds = probe_started.elapsed().as_secs_f64();
        if probe_elapsed_seconds >= EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS as f64 {
            abort_probe(
                &events,
                &report_path,
                probe_elapsed_seconds,
                "the cheap exact multicore probe exceeded its 60-minute wall ceiling before publication",
                &mut overall_heartbeat,
            );
        }
        let overall_max_rss_bytes = run_records
            .iter()
            .filter_map(|run| run.resources.max_sampled_rss_bytes)
            .chain(
                overall_process_start
                    .as_ref()
                    .ok()
                    .map(|sample| sample.resident_set_bytes),
            )
            .chain(
                overall_process_end
                    .as_ref()
                    .ok()
                    .map(|sample| sample.resident_set_bytes),
            )
            .chain(Some(overall_max_sampled_rss.load(Ordering::Acquire)))
            .max()
            .unwrap_or(0);
        let overall_resources = completed_resources(
            &overall_process_start,
            &overall_process_end,
            overall_max_rss_bytes,
            probe_elapsed_seconds,
            "FULL_PROBE_WALL",
            None,
        );
        let qualifies_full_run = registered_binding_work
            && exact_equality
            && all_streams_active
            && safety_adjusted_projected_suite_seconds < qualification_wall_seconds as f64;
        let mut report = ExactMulticoreProbeReport {
            schema: EXACT_MULTICORE_PROBE_SCHEMA.to_owned(),
            executor_contract_cid: exact_executor_contract_cid(),
            source: source_identity,
            host: host_identity,
            backend,
            probe_positions: positions,
            probe_context_ceiling_tokens,
            probe_position_indices,
            probe_streams: streams,
            probe_wall_ceiling_seconds: EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS,
            probe_deadline_policy: EXACT_MULTICORE_PROBE_DEADLINE_POLICY.to_owned(),
            events: ExactMulticoreProbeEventsBinding {
                file_name: events_path
                    .file_name()
                    .and_then(std::ffi::OsStr::to_str)
                    .unwrap_or("exact-multicore-probe.events.jsonl")
                    .to_owned(),
                content_cid: "PENDING".to_owned(),
                byte_len: 0,
                record_count: 0,
                final_record_number: 0,
                final_event: "PENDING".to_owned(),
                final_status: if qualifies_full_run {
                    ExactMulticoreProbeStatus::Qualified
                } else {
                    ExactMulticoreProbeStatus::RefuseFullRun
                },
                final_qualifies_full_run: false,
                report_body_cid: "PENDING".to_owned(),
            },
            probe_elapsed_seconds,
            runs: run_records,
            reference_workers,
            selected_best_config,
            configured_execution,
            exact_equality,
            all_workers_active,
            all_streams_active,
            configured_suite_work,
            raw_projected_suite_seconds,
            projection_safety_factor,
            projection_context_assumption: EXACT_MULTICORE_PROBE_CONTEXT_ASSUMPTION.to_owned(),
            safety_adjusted_projected_suite_seconds,
            binding_verdict: ExactMulticoreProbeVerdict {
                status: if qualifies_full_run {
                    ExactMulticoreProbeStatus::Qualified
                } else {
                    ExactMulticoreProbeStatus::RefuseFullRun
                },
                selection_policy: EXACT_MULTICORE_PROBE_SELECTION_POLICY.to_owned(),
                configured_max_wall_seconds: configured_max_wall,
                qualification_wall_seconds,
                qualifies_full_run,
            },
            resources: overall_resources,
        };
        if let Err(reason) = overall_heartbeat.stop_and_join() {
            unavailable_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                &reason,
                &mut overall_heartbeat,
            );
        }
        let final_status = report.binding_verdict.status;
        let final_qualifies_full_run = report.binding_verdict.qualifies_full_run;
        let finalization = report.write_after_durable_final(
            &report_path,
            &events_path,
            |report_body_cid, final_record_number| {
                try_emit_probe_record(
                    &events,
                    &serde_json::json!({
                        "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                        "record": "EXACT_MULTICORE_PROBE",
                        "event": "FINAL",
                        "sequence": final_record_number,
                        "source_path": &source,
                        "report_path": &report_path,
                        "events_path": &events_path,
                        "report_body_cid": report_body_cid,
                        "status": final_status,
                        "qualifies_full_run": final_qualifies_full_run,
                    }),
                    true,
                )
            },
        );
        if let Err(error) = finalization {
            // Every progress producer has already been joined. A failure after
            // FINAL therefore may append only this terminal non-PASS record;
            // no PROGRESS record can appear after either terminal event.
            let reason = format!(
                "durably append FINAL before publishing exact multicore probe report: {error}"
            );
            unavailable_probe(
                &events,
                &report_path,
                probe_started.elapsed().as_secs_f64(),
                &reason,
                &mut overall_heartbeat,
            );
        }
        assert!(
            exact_equality,
            "live exact outputs or persistent state changed across worker counts"
        );
    }

    /// Raw teacher throughput: serial `step` vs batched `forward_batch_into` on
    /// a real teacher. Ignored by default; run against a model directory:
    ///   TLESS_BENCH_MODEL=.uor-models/sources/smollm2-360m-instruct \
    ///   cargo test -p uor-r4-model-source --release bench_forward_batch \
    ///     -- --ignored --nocapture
    /// Optionally TLESS_BENCH_B=16 (batch), TLESS_BENCH_STEPS=128 (positions).
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    #[ignore]
    fn bench_forward_batch() {
        use std::time::Instant;
        let dir = std::env::var("TLESS_BENCH_MODEL")
            .expect("set TLESS_BENCH_MODEL to a teacher source directory");
        let steps: usize = std::env::var("TLESS_BENCH_STEPS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(128);
        let batch: usize = std::env::var("TLESS_BENCH_B")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(16);
        let mut oracle =
            HuggingFaceLlamaOracle::load_with_sequence_length(&dir, 128).expect("load teacher");
        let vocab = oracle.cfg().vocab;

        // Serial baseline: one sequence, `steps` positions via step().
        let mut logits = vec![0f32; vocab];
        oracle.reset();
        let t = Instant::now();
        for pos in 0..steps {
            oracle.step((pos % vocab).max(1), pos, &mut logits);
        }
        let serial_s = t.elapsed().as_secs_f64();
        let serial_tps = steps as f64 / serial_s;

        // Batched: `batch` sequences, `steps` positions each.
        let mut states: Vec<State> = (0..batch).map(|_| oracle.new_state()).collect();
        states.iter_mut().for_each(State::reset);
        let t = Instant::now();
        for pos in 0..steps {
            let tokens: Vec<usize> = (0..batch).map(|b| ((pos + b) % vocab).max(1)).collect();
            let positions = vec![pos; batch];
            oracle.forward_batch_into(&mut states, &tokens, &positions);
        }
        let batched_s = t.elapsed().as_secs_f64();
        let batched_tps = (steps * batch) as f64 / batched_s;

        eprintln!(
            "BENCH serial: {steps} tok in {serial_s:.3}s = {serial_tps:.1} tok/s | \
             batched B={batch}: {} tok in {batched_s:.3}s = {batched_tps:.1} tok/s | \
             speedup {:.2}x",
            steps * batch,
            batched_tps / serial_tps
        );
    }

    /// #598 source-codec bit-pattern tables: exact widening of BF16, F16,
    /// and F32 against hand-written hex constants — normals, subnormals,
    /// ±0, ±infinity, and payload-carrying NaNs. Equality is on raw bits.
    mod codec_bits {
        use crate::codec;

        /// (bf16 bits, expected f32 bits). BF16 → f32 is a 16-bit shift, so
        /// every class widens exactly, including NaN payloads.
        const BF16_WIDENING: &[(u16, u32)] = &[
            (0x0000, 0x0000_0000), // +0
            (0x8000, 0x8000_0000), // -0
            (0x3F80, 0x3F80_0000), // 1.0
            (0xBF80, 0xBF80_0000), // -1.0
            (0x4000, 0x4000_0000), // 2.0
            (0x3FC0, 0x3FC0_0000), // 1.5
            (0xC2F7, 0xC2F7_0000), // -123.5
            (0x0001, 0x0001_0000), // min subnormal (widens to an f32 subnormal)
            (0x007F, 0x007F_0000), // max subnormal
            (0x0080, 0x0080_0000), // min normal 2^-126
            (0x7F7F, 0x7F7F_0000), // max finite
            (0x7F80, 0x7F80_0000), // +inf
            (0xFF80, 0xFF80_0000), // -inf
            (0x7FC0, 0x7FC0_0000), // quiet NaN
            (0x7F81, 0x7F81_0000), // NaN, payload 0x01 preserved
            (0xFFC1, 0xFFC1_0000), // negative NaN, payload preserved
        ];

        /// (f16 bits, expected f32 bits). Normals re-bias (+112),
        /// subnormals normalize to f32 normals, NaN payloads shift by 13.
        const F16_WIDENING: &[(u16, u32)] = &[
            (0x0000, 0x0000_0000), // +0
            (0x8000, 0x8000_0000), // -0
            (0x3C00, 0x3F80_0000), // 1.0
            (0xBC00, 0xBF80_0000), // -1.0
            (0x4000, 0x4000_0000), // 2.0
            (0x3E00, 0x3FC0_0000), // 1.5
            (0x3555, 0x3EAA_A000), // 0.333251953125
            (0x7BFF, 0x477F_E000), // 65504.0 (max finite)
            (0xFBFF, 0xC77F_E000), // -65504.0
            (0x0400, 0x3880_0000), // min normal 2^-14
            (0x0001, 0x3380_0000), // min subnormal 2^-24
            (0x8001, 0xB380_0000), // -min subnormal
            (0x03FF, 0x387F_C000), // max subnormal 1023×2^-24
            (0x7C00, 0x7F80_0000), // +inf
            (0xFC00, 0xFF80_0000), // -inf
            (0x7E00, 0x7FC0_0000), // quiet NaN
            (0x7C01, 0x7F80_2000), // NaN, payload 0x001 → mantissa 0x001<<13
            (0xFE2A, 0xFFC5_4000), // negative NaN, payload 0x22A preserved
        ];

        /// F32 little-endian decode is bit-identity.
        const F32_IDENTITY: &[u32] = &[
            0x0000_0000, // +0
            0x8000_0000, // -0
            0x3F80_0000, // 1.0
            0xC2F6_E979, // -123.456...
            0x0000_0001, // min subnormal
            0x7F80_0000, // +inf
            0xFF80_0000, // -inf
            0x7FC0_0001, // quiet NaN with payload
            0x7F80_0001, // signaling NaN bit pattern
        ];

        #[test]
        fn bf16_widening_bit_patterns() {
            for &(bits, expected) in BF16_WIDENING {
                assert_eq!(
                    codec::bf16_to_f32(bits).to_bits(),
                    expected,
                    "bf16 {bits:#06x}"
                );
            }
        }

        #[test]
        fn f16_widening_bit_patterns() {
            for &(bits, expected) in F16_WIDENING {
                assert_eq!(
                    codec::f16_to_f32(bits).to_bits(),
                    expected,
                    "f16 {bits:#06x}"
                );
            }
        }

        #[test]
        fn f32_decode_is_bit_identity() {
            for &bits in F32_IDENTITY {
                assert_eq!(
                    codec::f32_from_le(bits.to_le_bytes()).to_bits(),
                    bits,
                    "f32 {bits:#010x}"
                );
            }
        }

        /// Exact round-trip: an f32 value representable in the narrow type
        /// survives narrow → widen unchanged, bit for bit.
        #[test]
        fn bf16_round_trip_values_are_exact() {
            for value in [1.0f32, -2.5, 0.156_25, 3.875, -0.007_812_5] {
                let narrow = (value.to_bits() >> 16) as u16;
                assert_eq!(codec::bf16_to_f32(narrow).to_bits(), value.to_bits());
            }
        }

        #[test]
        fn f16_round_trip_values_are_exact() {
            for (narrow, value) in [
                (0x3C00u16, 1.0f32),
                (0xB800, -0.5),
                (0x7BFF, 65504.0),
                (0x0400, f32::from_bits(0x3880_0000)), // 2^-14
            ] {
                assert_eq!(codec::f16_to_f32(narrow).to_bits(), value.to_bits());
            }
        }

        /// The byte-stream widening path used by tensor loading.
        #[cfg(not(target_arch = "wasm32"))]
        #[test]
        fn append_widened_decodes_each_dtype() {
            let mut out = Vec::new();
            codec::append_widened(
                crate::SourceDtype::Bf16,
                &[0x80, 0x3F, 0x00, 0xC0], // 1.0, -2.0
                &mut out,
            );
            codec::append_widened(
                crate::SourceDtype::F16,
                &[0x00, 0x3C, 0x00, 0xB8], // 1.0, -0.5
                &mut out,
            );
            codec::append_widened(crate::SourceDtype::F32, &1.5f32.to_le_bytes(), &mut out);
            assert_eq!(out, [1.0, -2.0, 1.0, -0.5, 1.5]);
        }
    }

    /// #598 ingestion-boundary tests: synthetic Safetensors shards and
    /// indexes built in-test (std only), one test per failure class, plus
    /// the sharded-equals-single-file round trip.
    #[cfg(not(target_arch = "wasm32"))]
    mod shard_ingestion {
        use crate::{
            exact_probe_expectation_shapes_from_config, HuggingFaceLlamaOracle,
            SafetensorsSnapshot, SourceIngestKind, TensorRequirement,
        };
        use std::path::{Path, PathBuf};
        use std::sync::atomic::{AtomicUsize, Ordering};

        const SHARD_1: &str = "model-00001-of-00002.safetensors";
        const SHARD_2: &str = "model-00002-of-00002.safetensors";

        fn temp_snapshot_dir(tag: &str) -> PathBuf {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "uor-r4-i598-{tag}-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&dir).expect("create synthetic snapshot dir");
            dir
        }

        fn raw_shard(header: &str, data: &[u8]) -> Vec<u8> {
            let mut out = (header.len() as u64).to_le_bytes().to_vec();
            out.extend_from_slice(header.as_bytes());
            out.extend_from_slice(data);
            out
        }

        /// Build a well-formed shard: contiguous offsets in entry order.
        fn shard_bytes(tensors: &[(&str, &str, &[usize], &[u8])]) -> Vec<u8> {
            let mut entries = Vec::new();
            let mut data = Vec::new();
            let mut offset = 0usize;
            for (name, dtype, shape, bytes) in tensors {
                let end = offset + bytes.len();
                let dims = shape
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                entries.push(format!(
                    "\"{name}\":{{\"dtype\":\"{dtype}\",\"shape\":[{dims}],\
                     \"data_offsets\":[{offset},{end}]}}"
                ));
                data.extend_from_slice(bytes);
                offset = end;
            }
            raw_shard(&format!("{{{}}}", entries.join(",")), &data)
        }

        fn index_bytes(entries: &[(&str, &str)]) -> Vec<u8> {
            let body = entries
                .iter()
                .map(|(tensor, shard)| format!("\"{tensor}\":\"{shard}\""))
                .collect::<Vec<_>>()
                .join(",");
            format!("{{\"metadata\":{{\"total_size\":0}},\"weight_map\":{{{body}}}}}").into_bytes()
        }

        fn write(dir: &Path, name: &str, bytes: &[u8]) {
            std::fs::write(dir.join(name), bytes).expect("write synthetic snapshot file");
        }

        fn req(name: &str, shape: &[usize]) -> TensorRequirement {
            TensorRequirement {
                name: name.to_owned(),
                shape: shape.to_vec(),
            }
        }

        /// Deterministic finite BF16 payload bytes for `n` elements.
        fn bf16_data(n: usize, salt: u16) -> Vec<u8> {
            (0..n)
                .flat_map(|i| {
                    (0x3F00u16 | ((i as u16).wrapping_mul(31).wrapping_add(salt) & 0x7F))
                        .to_le_bytes()
                })
                .collect()
        }

        #[test]
        fn missing_shard_file_is_detected() {
            let dir = temp_snapshot_dir("missing-shard");
            write(
                &dir,
                "model.safetensors.index.json",
                &index_bytes(&[("a", SHARD_1), ("b", SHARD_2)]),
            );
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 1))]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::MissingShardFile {
                    shard: SHARD_2.to_owned()
                }
            );
        }

        #[test]
        fn tensor_missing_from_its_mapped_shard_is_detected() {
            let dir = temp_snapshot_dir("missing-tensor");
            write(
                &dir,
                "model.safetensors.index.json",
                &index_bytes(&[("a", SHARD_1), ("b", SHARD_1)]),
            );
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 2))]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::MissingTensor {
                    tensor: "b".to_owned(),
                    shard: SHARD_1.to_owned()
                }
            );
        }

        #[test]
        fn required_tensor_absent_everywhere_is_detected() {
            let dir = temp_snapshot_dir("missing-required");
            write(
                &dir,
                "model.safetensors",
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 3))]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[req("b", &[2])]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::MissingTensor {
                    tensor: "b".to_owned(),
                    shard: "model.safetensors".to_owned()
                }
            );
        }

        #[test]
        fn duplicate_tensor_across_shards_is_detected() {
            let dir = temp_snapshot_dir("dup-shards");
            write(
                &dir,
                "model.safetensors.index.json",
                &index_bytes(&[("a", SHARD_1), ("b", SHARD_2)]),
            );
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 4))]),
            );
            write(
                &dir,
                SHARD_2,
                &shard_bytes(&[
                    ("a", "BF16", &[2], &bf16_data(2, 4)),
                    ("b", "BF16", &[2], &bf16_data(2, 12)),
                ]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::DuplicateTensor {
                    tensor: "a".to_owned(),
                    first_shard: SHARD_1.to_owned(),
                    second_shard: SHARD_2.to_owned()
                }
            );
        }

        #[test]
        fn duplicate_index_mapping_is_detected() {
            let dir = temp_snapshot_dir("dup-index");
            // Hand-written raw JSON: the same key appears twice in
            // weight_map; a plain serde_json map would silently drop one.
            let index = format!("{{\"weight_map\":{{\"a\":\"{SHARD_1}\",\"a\":\"{SHARD_1}\"}}}}");
            write(&dir, "model.safetensors.index.json", index.as_bytes());
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 5))]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::DuplicateTensor {
                    tensor: "a".to_owned(),
                    first_shard: SHARD_1.to_owned(),
                    second_shard: SHARD_1.to_owned()
                }
            );
        }

        #[test]
        fn unexpected_tensor_outside_index_is_detected() {
            let dir = temp_snapshot_dir("unexpected");
            write(
                &dir,
                "model.safetensors.index.json",
                &index_bytes(&[("a", SHARD_1)]),
            );
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[
                    ("a", "BF16", &[2], &bf16_data(2, 6)),
                    ("b", "BF16", &[2], &bf16_data(2, 7)),
                ]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::UnexpectedTensor {
                    tensor: "b".to_owned(),
                    shard: SHARD_1.to_owned()
                }
            );
        }

        #[test]
        fn shape_mismatch_against_geometry_is_detected() {
            let dir = temp_snapshot_dir("shape");
            write(
                &dir,
                "model.safetensors",
                &shard_bytes(&[("a", "BF16", &[4], &bf16_data(4, 8))]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[req("a", &[2, 2])]).unwrap_err();
            assert_eq!(
                error.kind,
                SourceIngestKind::ShapeMismatch {
                    tensor: "a".to_owned(),
                    expected: vec![2, 2],
                    actual: vec![4]
                }
            );
        }

        #[test]
        fn tensor_span_shorter_than_shape_dtype_size_is_detected() {
            let dir = temp_snapshot_dir("span");
            // Shape [2,2] BF16 claims 8 bytes but the span declares 6.
            let header = "{\"a\":{\"dtype\":\"BF16\",\"shape\":[2,2],\"data_offsets\":[0,6]}}";
            write(&dir, "model.safetensors", &raw_shard(header, &[0u8; 6]));
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            match error.kind {
                SourceIngestKind::ByteLengthMismatch {
                    context,
                    expected: 8,
                    actual: 6,
                } => assert!(context.contains("shape×dtype"), "{context}"),
                other => panic!("expected tensor-span ByteLengthMismatch, got {other:?}"),
            }
        }

        #[test]
        fn shard_file_size_disagreeing_with_header_is_detected() {
            let dir = temp_snapshot_dir("filesize");
            let mut shard = shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 9))]);
            shard.extend_from_slice(&[0u8; 4]); // trailing bytes the header never claimed
            write(&dir, "model.safetensors", &shard);
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            match error.kind {
                SourceIngestKind::ByteLengthMismatch { context, .. } => {
                    assert!(context.contains("file size"), "{context}");
                }
                other => panic!("expected file-size ByteLengthMismatch, got {other:?}"),
            }
        }

        #[test]
        fn non_contiguous_data_offsets_are_detected() {
            let dir = temp_snapshot_dir("gap");
            let header = "{\"a\":{\"dtype\":\"BF16\",\"shape\":[2],\"data_offsets\":[4,8]}}";
            write(&dir, "model.safetensors", &raw_shard(header, &[0u8; 8]));
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            match error.kind {
                SourceIngestKind::ByteLengthMismatch { context, .. } => {
                    assert!(context.contains("contiguous"), "{context}");
                }
                other => panic!("expected contiguity ByteLengthMismatch, got {other:?}"),
            }
        }

        #[test]
        fn dtype_inconsistency_across_shards_is_detected() {
            let dir = temp_snapshot_dir("dtype-mix");
            write(
                &dir,
                "model.safetensors.index.json",
                &index_bytes(&[("a", SHARD_1), ("b", SHARD_2)]),
            );
            write(
                &dir,
                SHARD_1,
                &shard_bytes(&[("a", "BF16", &[2], &bf16_data(2, 10))]),
            );
            write(
                &dir,
                SHARD_2,
                &shard_bytes(&[
                    ("a", "F16", &[2], &[0u8; 4]),
                    ("b", "BF16", &[2], &bf16_data(2, 13)),
                ]),
            );
            let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
            match error.kind {
                SourceIngestKind::DtypeInconsistency {
                    tensor,
                    first,
                    second,
                } => {
                    assert_eq!(tensor, "a");
                    assert!(first.contains("BF16"), "{first}");
                    assert!(second.contains("F16"), "{second}");
                }
                other => panic!("expected DtypeInconsistency, got {other:?}"),
            }
        }

        #[test]
        fn quantized_and_unknown_dtypes_are_rejected_by_name() {
            for dtype in ["I8", "U8", "Q4_GPTQ"] {
                let dir = temp_snapshot_dir("unsupported");
                let header = format!(
                    "{{\"a\":{{\"dtype\":\"{dtype}\",\"shape\":[4],\"data_offsets\":[0,4]}}}}"
                );
                write(&dir, "model.safetensors", &raw_shard(&header, &[0u8; 4]));
                let error = SafetensorsSnapshot::open(&dir, &[]).unwrap_err();
                assert_eq!(
                    error.kind,
                    SourceIngestKind::UnsupportedDtype {
                        tensor: "a".to_owned(),
                        dtype: dtype.to_owned()
                    }
                );
                assert!(error.to_string().contains(dtype));
            }
        }

        #[test]
        fn single_file_snapshot_keeps_its_kappa_and_widens_exactly() {
            let dir = temp_snapshot_dir("single");
            let shard = shard_bytes(&[(
                "a",
                "BF16",
                &[2],
                &[0x80, 0x3F, 0x00, 0xC0], // 1.0, -2.0
            )]);
            write(&dir, "model.safetensors", &shard);
            let snapshot = SafetensorsSnapshot::open(&dir, &[req("a", &[2])]).expect("open");
            // The single-file κ is the pre-#598 hash of model.safetensors.
            assert_eq!(
                snapshot.kappa(),
                format!("blake3:{}", blake3::hash(&shard).to_hex())
            );
            assert_eq!(snapshot.source_bytes(), shard.len());
            let mut out = Vec::new();
            snapshot.tensor_f32_into("a", &mut out).expect("tensor");
            assert_eq!(out, [1.0, -2.0]);
        }

        #[test]
        fn extra_tensor_without_index_is_tolerated() {
            let dir = temp_snapshot_dir("extra");
            write(
                &dir,
                "model.safetensors",
                &shard_bytes(&[
                    ("a", "BF16", &[2], &bf16_data(2, 11)),
                    ("rotary.inv_freq", "F32", &[1], &1.0f32.to_le_bytes()),
                ]),
            );
            // No index: extra tensors are ignored exactly as before #598.
            SafetensorsSnapshot::open(&dir, &[req("a", &[2])]).expect("open single-file");
        }

        /// The tiny synthetic Llama snapshot used by the round-trip test:
        /// mixed BF16/F16/F32 tensors matching `tiny_config_json`.
        fn tiny_model_tensors() -> Vec<(String, &'static str, Vec<usize>, Vec<u8>)> {
            let f16_data = |n: usize| -> Vec<u8> {
                (0..n)
                    .flat_map(|i| (0x3400u16 | ((i as u16).wrapping_mul(17) & 0xFF)).to_le_bytes())
                    .collect()
            };
            let f32_data = |n: usize| -> Vec<u8> {
                (0..n)
                    .flat_map(|i| (i as f32 * 0.031_25 - 1.0).to_le_bytes())
                    .collect()
            };
            let mut tensors = Vec::new();
            let mut push = |name: &str, dtype: &'static str, shape: &[usize], data: Vec<u8>| {
                tensors.push((name.to_owned(), dtype, shape.to_vec(), data));
            };
            push(
                "model.embed_tokens.weight",
                "BF16",
                &[10, 8],
                bf16_data(80, 20),
            );
            push(
                "model.layers.0.input_layernorm.weight",
                "F16",
                &[8],
                f16_data(8),
            );
            push(
                "model.layers.0.self_attn.q_proj.weight",
                "BF16",
                &[8, 8],
                bf16_data(64, 21),
            );
            push(
                "model.layers.0.self_attn.k_proj.weight",
                "BF16",
                &[8, 8],
                bf16_data(64, 22),
            );
            push(
                "model.layers.0.self_attn.v_proj.weight",
                "BF16",
                &[8, 8],
                bf16_data(64, 23),
            );
            push(
                "model.layers.0.self_attn.o_proj.weight",
                "BF16",
                &[8, 8],
                bf16_data(64, 24),
            );
            push(
                "model.layers.0.post_attention_layernorm.weight",
                "BF16",
                &[8],
                bf16_data(8, 25),
            );
            push(
                "model.layers.0.mlp.gate_proj.weight",
                "BF16",
                &[16, 8],
                bf16_data(128, 26),
            );
            push(
                "model.layers.0.mlp.down_proj.weight",
                "BF16",
                &[8, 16],
                bf16_data(128, 27),
            );
            push(
                "model.layers.0.mlp.up_proj.weight",
                "BF16",
                &[16, 8],
                bf16_data(128, 28),
            );
            push("model.norm.weight", "F32", &[8], f32_data(8));
            tensors
        }

        const TINY_CONFIG: &str = r#"{
            "hidden_size": 8,
            "intermediate_size": 16,
            "num_hidden_layers": 1,
            "num_attention_heads": 2,
            "num_key_value_heads": 2,
            "vocab_size": 10,
            "max_position_embeddings": 8,
            "tie_word_embeddings": true
        }"#;

        #[test]
        fn config_only_probe_planner_never_requires_weight_files() {
            let dir = temp_snapshot_dir("probe-plan-config-only");
            write(&dir, "config.json", TINY_CONFIG.as_bytes());
            assert!(!dir.join("model.safetensors").exists());
            let shapes = exact_probe_expectation_shapes_from_config(&dir, 8, &[4, 8], 4, 8, 1, 8)
                .expect("config-only exact probe plan");
            assert_eq!(shapes.forward_plans.len(), 2);
            assert!(shapes
                .forward_plans
                .iter()
                .all(|plan| plan.forward_plan.batch_width == 8));
            assert!(shapes
                .forward_plans
                .iter()
                .all(|plan| plan.forward_plan.matrix_calls == 8));
            assert_eq!(shapes.trace_shape.positions, 1);
            assert_eq!(shapes.trace_shape.streams_per_position, 8);
            assert_eq!(shapes.trace_shape.logits_per_state, 10);
            assert_eq!(shapes.trace_shape.state_records, 8);
        }

        fn as_entries<'a>(
            tensors: &'a [(String, &'static str, Vec<usize>, Vec<u8>)],
        ) -> Vec<(&'a str, &'a str, &'a [usize], &'a [u8])> {
            tensors
                .iter()
                .map(|(name, dtype, shape, data)| {
                    (name.as_str(), *dtype, shape.as_slice(), data.as_slice())
                })
                .collect()
        }

        /// The #598 round trip: a valid 2-shard indexed snapshot loads to
        /// the same flattened f32 weights — and the same step logits — as
        /// the equivalent single-file snapshot, through one code path.
        #[test]
        fn two_shard_index_round_trip_equals_single_file_load() {
            let tensors = tiny_model_tensors();

            let single = temp_snapshot_dir("roundtrip-single");
            write(&single, "config.json", TINY_CONFIG.as_bytes());
            write(
                &single,
                "model.safetensors",
                &shard_bytes(&as_entries(&tensors)),
            );

            let sharded = temp_snapshot_dir("roundtrip-sharded");
            write(&sharded, "config.json", TINY_CONFIG.as_bytes());
            let (first, second) = tensors.split_at(5);
            write(&sharded, SHARD_1, &shard_bytes(&as_entries(first)));
            write(&sharded, SHARD_2, &shard_bytes(&as_entries(second)));
            let index: Vec<(&str, &str)> = first
                .iter()
                .map(|(name, ..)| (name.as_str(), SHARD_1))
                .chain(second.iter().map(|(name, ..)| (name.as_str(), SHARD_2)))
                .collect();
            write(
                &sharded,
                "model.safetensors.index.json",
                &index_bytes(&index),
            );

            let mut from_single =
                HuggingFaceLlamaOracle::load(&single).expect("load single-file snapshot");
            let mut from_shards =
                HuggingFaceLlamaOracle::load(&sharded).expect("load sharded snapshot");
            assert_eq!(
                from_single.model.w, from_shards.model.w,
                "flattened f32 weights must be identical"
            );

            use crate::BehaviorSource;
            let vocab = from_single.model.cfg.vocab;
            let mut single_logits = vec![0.0f32; vocab];
            let mut sharded_logits = vec![0.0f32; vocab];
            from_single.reset();
            from_shards.reset();
            from_single.step(1, 0, &mut single_logits);
            from_shards.step(1, 0, &mut sharded_logits);
            assert!(single_logits.iter().all(|logit| logit.is_finite()));
            assert_eq!(single_logits, sharded_logits);
        }
    }
}
