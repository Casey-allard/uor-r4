//! GPT-2 (`openai-community/gpt2`) source executor for #607 — the first
//! genuinely non-Llama adapter behind the architecture-neutral compiler
//! boundary.
//!
//! Every GPT-2-specific fact lives in this module: the `config.json` key
//! schema (`n_embd`/`n_head`/`n_layer`/`n_positions`), the bare
//! `wte`/`wpe`/`h.<l>.*`/`ln_f` tensor names, the fused `c_attn` QKV split,
//! the Conv1D weight orientation (`[in, out]`, applied as `x @ W` with no
//! transpose), learned absolute positions, LayerNorm-with-bias, the
//! `gelu_new` activation, and tied embeddings. Downstream crates see only
//! the [`crate::TeacherOracle`] two-surface trait, never any of these names.
//!
//! The executor is a faithful f32 reference: it reproduces an independent
//! numpy GPT-2 forward within the #599 fixture tolerances (see the
//! generated-fixture test), so "parity" is measured against a second
//! implementation, not against itself.
#![cfg(not(target_arch = "wasm32"))]

use crate::{SafetensorsSnapshot, SourceIngestKind, SourceUnavailable, TensorRequirement};

/// Parsed, validated GPT-2 geometry from `config.json`. The GPT-2 family
/// spells these fields differently from Llama; the #599 conformance gate
/// (`AdapterFeatures::huggingface_gpt2`) has already validated the raw
/// configuration against the adapter's declaration before this parse.
#[derive(Debug, Clone)]
pub struct Gpt2Config {
    /// Residual / hidden width (`n_embd`).
    pub n_embd: usize,
    /// Attention heads (`n_head`); GPT-2 is plain multi-head (kv == heads).
    pub n_head: usize,
    /// Transformer blocks (`n_layer`).
    pub n_layer: usize,
    /// Learned position-table rows (`n_positions`): the maximum context.
    pub n_positions: usize,
    /// MLP inner width. GPT-2 uses `n_inner` when present, else `4 * n_embd`.
    pub n_inner: usize,
    /// Vocabulary size (`vocab_size`).
    pub vocab: usize,
    /// LayerNorm epsilon (`layer_norm_epsilon`).
    pub layer_norm_eps: f32,
    /// Working sequence length (bounded teacher context; <= n_positions).
    pub seq_len: usize,
    /// `bos_token_id`.
    pub bos: usize,
    /// `eos_token_id`.
    pub eos: usize,
}

impl Gpt2Config {
    /// Head width (`n_embd / n_head`).
    pub fn head_size(&self) -> usize {
        self.n_embd / self.n_head
    }

    /// Parse and range-check the GPT-2 fields. Presence/geometry
    /// relationships the #599 gate already checked are re-asserted here as
    /// typed-parse errors so a direct executor construction is also safe.
    pub fn from_json(
        config: &serde_json::Value,
        sequence_length: Option<usize>,
    ) -> Result<Self, SourceUnavailable> {
        let uint = |field: &str| -> Result<usize, SourceUnavailable> {
            config
                .get(field)
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as usize)
                .ok_or_else(|| {
                    SourceUnavailable::from(SourceIngestKind::Unreadable {
                        path: "config.json".to_owned(),
                        reason: format!("GPT-2 config is missing an integer `{field}`"),
                    })
                })
        };
        let n_embd = uint("n_embd")?;
        let n_head = uint("n_head")?;
        let n_layer = uint("n_layer")?;
        let n_positions = uint("n_positions")?;
        let vocab = uint("vocab_size")?;
        let n_inner = config
            .get("n_inner")
            .and_then(serde_json::Value::as_u64)
            .map(|value| value as usize)
            .filter(|inner| *inner > 0)
            .unwrap_or(4 * n_embd);
        let layer_norm_eps = config
            .get("layer_norm_epsilon")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(1e-5) as f32;
        let bos = uint("bos_token_id").unwrap_or(
            config
                .get("bos_token_id")
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(0),
        );
        let eos = config
            .get("eos_token_id")
            .and_then(serde_json::Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(bos);
        if n_head == 0 || n_embd % n_head != 0 {
            return Err(SourceIngestKind::Unreadable {
                path: "config.json".to_owned(),
                reason: format!("GPT-2 n_embd {n_embd} is not divisible by n_head {n_head}"),
            }
            .into());
        }
        let seq_len = sequence_length
            .unwrap_or(n_positions)
            .min(n_positions)
            .max(1);
        Ok(Self {
            n_embd,
            n_head,
            n_layer,
            n_positions,
            n_inner,
            vocab,
            layer_norm_eps,
            seq_len,
            bos,
            eos,
        })
    }
}

/// One transformer block's weights, in the bare GPT-2 tensor layout. Conv1D
/// weights are stored `[in, out]` exactly as the checkpoint carries them,
/// applied as `y[o] = b[o] + sum_i x[i] * w[i * out + o]` with no transpose.
pub(crate) struct Gpt2Layer {
    pub(crate) ln1_w: Vec<f32>,
    pub(crate) ln1_b: Vec<f32>,
    /// `c_attn` fused QKV projection: `[n_embd, 3 * n_embd]`.
    pub(crate) c_attn_w: Vec<f32>,
    pub(crate) c_attn_b: Vec<f32>,
    /// attention output projection `c_proj`: `[n_embd, n_embd]`.
    pub(crate) c_proj_w: Vec<f32>,
    pub(crate) c_proj_b: Vec<f32>,
    pub(crate) ln2_w: Vec<f32>,
    pub(crate) ln2_b: Vec<f32>,
    /// MLP `c_fc`: `[n_embd, n_inner]`.
    pub(crate) fc_w: Vec<f32>,
    pub(crate) fc_b: Vec<f32>,
    /// MLP `c_proj`: `[n_inner, n_embd]`.
    pub(crate) mlp_w: Vec<f32>,
    pub(crate) mlp_b: Vec<f32>,
}

/// The GPT-2 executor: token + learned-position embeddings, a stack of
/// LayerNorm/attention/MLP blocks, a final LayerNorm, and a tied lm-head.
pub struct Gpt2 {
    pub cfg: Gpt2Config,
    /// Token embeddings `wte`: `[vocab, n_embd]` (also the tied lm-head).
    pub(crate) wte: Vec<f32>,
    /// Learned position embeddings `wpe`: `[n_positions, n_embd]`.
    pub(crate) wpe: Vec<f32>,
    pub(crate) layers: Vec<Gpt2Layer>,
    pub(crate) ln_f_w: Vec<f32>,
    pub(crate) ln_f_b: Vec<f32>,
    pub(crate) kappa: String,
    pub(crate) source_bytes: usize,
}

/// The tensors the GPT-2 geometry requires, with the shapes `config.json`
/// implies. Bare names (no `transformer.` prefix), matching the pinned
/// `openai-community/gpt2` checkpoint. The causal-mask buffer
/// `h.<l>.attn.bias` is deliberately not required: masking is applied
/// directly, and a single-file snapshot ignores unrequired tensors.
pub(crate) fn required_gpt2_tensors(cfg: &Gpt2Config) -> Vec<TensorRequirement> {
    let (d, inner, three) = (cfg.n_embd, cfg.n_inner, 3 * cfg.n_embd);
    let mut req = vec![
        TensorRequirement {
            name: "wte.weight".to_owned(),
            shape: vec![cfg.vocab, d],
        },
        TensorRequirement {
            name: "wpe.weight".to_owned(),
            shape: vec![cfg.n_positions, d],
        },
    ];
    for l in 0..cfg.n_layer {
        for (suffix, shape) in [
            ("ln_1.weight", vec![d]),
            ("ln_1.bias", vec![d]),
            ("attn.c_attn.weight", vec![d, three]),
            ("attn.c_attn.bias", vec![three]),
            ("attn.c_proj.weight", vec![d, d]),
            ("attn.c_proj.bias", vec![d]),
            ("ln_2.weight", vec![d]),
            ("ln_2.bias", vec![d]),
            ("mlp.c_fc.weight", vec![d, inner]),
            ("mlp.c_fc.bias", vec![inner]),
            ("mlp.c_proj.weight", vec![inner, d]),
            ("mlp.c_proj.bias", vec![d]),
        ] {
            req.push(TensorRequirement {
                name: format!("h.{l}.{suffix}"),
                shape,
            });
        }
    }
    req.push(TensorRequirement {
        name: "ln_f.weight".to_owned(),
        shape: vec![d],
    });
    req.push(TensorRequirement {
        name: "ln_f.bias".to_owned(),
        shape: vec![d],
    });
    req
}

impl Gpt2 {
    /// Load the executor from a snapshot directory: `config.json` plus a
    /// single `model.safetensors` (or #598 indexed shards). The raw config
    /// passes the #599 GPT-2 conformance gate before the typed parse, and
    /// all shape/dtype/byte checks happen at the one
    /// [`SafetensorsSnapshot::open`] boundary before any weight is read.
    pub fn load(
        source: impl AsRef<std::path::Path>,
        sequence_length: Option<usize>,
    ) -> Result<Self, SourceUnavailable> {
        let source = source.as_ref();
        let config_bytes = std::fs::read(source.join("config.json"))?;
        let raw: serde_json::Value = serde_json::from_slice(&config_bytes)?;
        crate::conformance::AdapterFeatures::huggingface_gpt2().validate_config(&raw)?;
        let cfg = Gpt2Config::from_json(&raw, sequence_length)?;
        let required = required_gpt2_tensors(&cfg);
        let snapshot = SafetensorsSnapshot::open(source, &required)?;

        let load = |name: &str| -> Result<Vec<f32>, SourceUnavailable> {
            let mut out = Vec::new();
            snapshot.tensor_f32_into(name, &mut out)?;
            Ok(out)
        };
        let wte = load("wte.weight")?;
        let wpe = load("wpe.weight")?;
        let mut layers = Vec::with_capacity(cfg.n_layer);
        for l in 0..cfg.n_layer {
            layers.push(Gpt2Layer {
                ln1_w: load(&format!("h.{l}.ln_1.weight"))?,
                ln1_b: load(&format!("h.{l}.ln_1.bias"))?,
                c_attn_w: load(&format!("h.{l}.attn.c_attn.weight"))?,
                c_attn_b: load(&format!("h.{l}.attn.c_attn.bias"))?,
                c_proj_w: load(&format!("h.{l}.attn.c_proj.weight"))?,
                c_proj_b: load(&format!("h.{l}.attn.c_proj.bias"))?,
                ln2_w: load(&format!("h.{l}.ln_2.weight"))?,
                ln2_b: load(&format!("h.{l}.ln_2.bias"))?,
                fc_w: load(&format!("h.{l}.mlp.c_fc.weight"))?,
                fc_b: load(&format!("h.{l}.mlp.c_fc.bias"))?,
                mlp_w: load(&format!("h.{l}.mlp.c_proj.weight"))?,
                mlp_b: load(&format!("h.{l}.mlp.c_proj.bias"))?,
            });
        }
        let ln_f_w = load("ln_f.weight")?;
        let ln_f_b = load("ln_f.bias")?;
        Ok(Self {
            cfg,
            wte,
            wpe,
            layers,
            ln_f_w,
            ln_f_b,
            kappa: snapshot.kappa().to_owned(),
            source_bytes: snapshot.source_bytes(),
        })
    }

    /// Loaded model digest exposed only for hard-binding the #704 canary.
    #[doc(hidden)]
    pub fn attention_control_source_kappa(&self) -> &str {
        &self.kappa
    }

    /// Loaded model byte count exposed only for hard-binding the #704 canary.
    #[doc(hidden)]
    pub fn attention_control_source_bytes(&self) -> usize {
        self.source_bytes
    }
}

/// GPT-2's `gelu_new` (the tanh approximation the checkpoint was trained
/// with): `0.5 x (1 + tanh(sqrt(2/pi) (x + 0.044715 x^3)))`.
#[inline]
fn gelu_new(x: f32) -> f32 {
    const C: f32 = 0.797_884_6; // sqrt(2/pi)
    0.5 * x * (1.0 + (C * (x + 0.044_715 * x * x * x)).tanh())
}

/// LayerNorm with a learned scale and bias, into `out`:
/// `out = (x - mean) / sqrt(var + eps) * w + b`, population variance.
fn layer_norm(x: &[f32], w: &[f32], b: &[f32], eps: f32, out: &mut [f32]) {
    let n = x.len() as f32;
    let mean = x.iter().sum::<f32>() / n;
    let var = x.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n;
    let inv = 1.0 / (var + eps).sqrt();
    for i in 0..x.len() {
        out[i] = (x[i] - mean) * inv * w[i] + b[i];
    }
}

/// Conv1D as GPT-2 stores it: weight `[in, out]` (row-major), applied
/// `out[o] = bias[o] + sum_i x[i] * w[i * out_dim + o]` — no transpose.
fn conv1d(x: &[f32], w: &[f32], bias: &[f32], out_dim: usize, out: &mut [f32]) {
    out[..out_dim].copy_from_slice(&bias[..out_dim]);
    for (i, &xi) in x.iter().enumerate() {
        if xi == 0.0 {
            continue;
        }
        let row = &w[i * out_dim..(i + 1) * out_dim];
        for o in 0..out_dim {
            out[o] += xi * row[o];
        }
    }
}

/// Batched Conv1D: `b` input vectors of length `in_dim` through weight
/// `[in_dim, out_dim]` → `b` output vectors of length `out_dim`, both in
/// sequence-major layout (`b * in_dim` in, `b * out_dim` out). Each weight
/// row `w[i * out_dim..]` is read once per input index `i` and reused
/// across all `b` sequences — the memory-amortization that lifts batched
/// teacher inference off the per-token bandwidth wall — while every output
/// accumulates over `i` in the SAME order, with the same bias
/// initialization and the same bit-neutral zero-input skip, as the serial
/// [`conv1d`]. So `conv1d_batched` is bit-identical to calling `conv1d` on
/// each sequence separately.
fn conv1d_batched(
    out: &mut [f32],
    x: &[f32],
    w: &[f32],
    bias: &[f32],
    out_dim: usize,
    in_dim: usize,
    b: usize,
) {
    for bi in 0..b {
        out[bi * out_dim..(bi + 1) * out_dim].copy_from_slice(&bias[..out_dim]);
    }
    for i in 0..in_dim {
        let row = &w[i * out_dim..(i + 1) * out_dim];
        for bi in 0..b {
            let xi = x[bi * in_dim + i];
            if xi == 0.0 {
                continue;
            }
            let ob = &mut out[bi * out_dim..(bi + 1) * out_dim];
            for o in 0..out_dim {
                ob[o] += xi * row[o];
            }
        }
    }
}

/// Preserve GPT-2's scalar post-dot order: multiply every raw Q·K result by
/// one precomputed reciprocal square-root, then use one reciprocal of the
/// softmax sum and multiply every normalized weight by it.
fn scale_and_normalize_attention_scores(scores: &mut [f32], scale: f32) {
    let mut max = f32::NEG_INFINITY;
    for score in scores.iter_mut() {
        *score *= scale;
        if *score > max {
            max = *score;
        }
    }
    let mut sum = 0.0f32;
    for score in scores.iter_mut() {
        *score = (*score - max).exp();
        sum += *score;
    }
    let inverse = 1.0 / sum;
    for score in scores.iter_mut() {
        *score *= inverse;
    }
}

/// GPT-2 Q·K control preserving its reciprocal-multiply normalization order.
#[doc(hidden)]
pub(crate) fn attention_weights_with_arithmetic(
    scores: &mut [f32],
    q: &[f32],
    keys: &[f32],
    key_offset: usize,
    key_stride: usize,
    arithmetic: crate::attention::AttentionArithmetic,
) -> crate::attention::AttentionDotCensus {
    let census = crate::attention::head_dot_products_with_arithmetic(
        scores,
        q,
        keys,
        key_offset,
        key_stride,
        q.len(),
        arithmetic,
    );
    let scale = 1.0 / (q.len() as f32).sqrt();
    scale_and_normalize_attention_scores(scores, scale);
    census
}

/// Recurrent decode state: per-layer key/value caches, the working
/// residual, the final hidden state, and the last step's logits.
#[derive(Clone, Debug, PartialEq)]
pub struct Gpt2State {
    k_cache: Vec<f32>,
    v_cache: Vec<f32>,
    /// Logits of the last [`Gpt2::forward`] step (`vocab`).
    pub logits: Vec<f32>,
    /// Final hidden state (post `ln_f`) of the last step (`n_embd`).
    pub hidden: Vec<f32>,
    x: Vec<f32>,
}

/// Arithmetic arm selected by the checked, evidence-only GPT-2 attention
/// canary façade. Raw strided attention controls remain crate-private.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpt2AttentionCanaryMode {
    Conventional,
    Exact,
    CertifiedNative,
}

/// Certified-lane and exact-fallback counts returned by the checked canary
/// façade. Unlike the crate-private proof census, this report exposes no raw
/// layout control.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gpt2AttentionCanaryDotCensus {
    lanes: usize,
    certified: usize,
    fallback_nonfinite: usize,
    fallback_zero: usize,
    fallback_overflow: usize,
    fallback_cell: usize,
}

impl Gpt2AttentionCanaryDotCensus {
    pub const fn lanes(self) -> usize {
        self.lanes
    }

    pub const fn certified(self) -> usize {
        self.certified
    }

    pub const fn fallback_nonfinite(self) -> usize {
        self.fallback_nonfinite
    }

    pub const fn fallback_zero(self) -> usize {
        self.fallback_zero
    }

    pub const fn fallback_overflow(self) -> usize {
        self.fallback_overflow
    }

    pub const fn fallback_cell(self) -> usize {
        self.fallback_cell
    }

    /// Total fallback count, or `None` when the private counters do not form a
    /// `usize` product.
    pub fn fallbacks(self) -> Option<usize> {
        checked_canary_count(self.fallback_nonfinite, self.fallback_zero)
            .and_then(|total| checked_canary_count(total, self.fallback_overflow))
            .and_then(|total| checked_canary_count(total, self.fallback_cell))
    }

    fn checked_merge(self, other: Self) -> Option<Self> {
        Some(Self {
            lanes: checked_canary_count(self.lanes, other.lanes)?,
            certified: checked_canary_count(self.certified, other.certified)?,
            fallback_nonfinite: checked_canary_count(
                self.fallback_nonfinite,
                other.fallback_nonfinite,
            )?,
            fallback_zero: checked_canary_count(self.fallback_zero, other.fallback_zero)?,
            fallback_overflow: checked_canary_count(
                self.fallback_overflow,
                other.fallback_overflow,
            )?,
            fallback_cell: checked_canary_count(self.fallback_cell, other.fallback_cell)?,
        })
    }
}

impl From<crate::attention::AttentionDotCensus> for Gpt2AttentionCanaryDotCensus {
    fn from(value: crate::attention::AttentionDotCensus) -> Self {
        Self {
            lanes: value.lanes,
            certified: value.certified,
            fallback_nonfinite: value.fallback_nonfinite,
            fallback_zero: value.fallback_zero,
            fallback_overflow: value.fallback_overflow,
            fallback_cell: value.fallback_cell,
        }
    }
}

/// Aggregate QK and weighted-value census for a checked GPT-2 canary story.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gpt2AttentionCanaryCensus {
    qk: Gpt2AttentionCanaryDotCensus,
    value: Gpt2AttentionCanaryDotCensus,
}

impl Gpt2AttentionCanaryCensus {
    pub const fn qk(self) -> Gpt2AttentionCanaryDotCensus {
        self.qk
    }

    pub const fn value(self) -> Gpt2AttentionCanaryDotCensus {
        self.value
    }

    /// Merge a story census atomically. `None` means the combined counters do
    /// not form a `usize` product and leaves `self` byte-identical.
    pub fn merge(&mut self, other: Self) -> Option<()> {
        let qk = self.qk.checked_merge(other.qk)?;
        let value = self.value.checked_merge(other.value)?;
        self.qk = qk;
        self.value = value;
        Some(())
    }
}

impl From<crate::attention::AttentionArithmeticCensus> for Gpt2AttentionCanaryCensus {
    fn from(value: crate::attention::AttentionArithmeticCensus) -> Self {
        Self {
            qk: value.qk.into(),
            value: value.value.into(),
        }
    }
}

fn checked_canary_count(left: usize, right: usize) -> Option<usize> {
    left.checked_add(right)
}

impl Gpt2State {
    pub fn new(cfg: &Gpt2Config) -> Self {
        let cache = cfg.n_layer * cfg.seq_len * cfg.n_embd;
        Self {
            k_cache: vec![0.0; cache],
            v_cache: vec![0.0; cache],
            logits: vec![0.0; cfg.vocab],
            hidden: vec![0.0; cfg.n_embd],
            x: vec![0.0; cfg.n_embd],
        }
    }

    /// Begin a new sequence: zero the caches and working buffers.
    pub fn reset(&mut self) {
        self.k_cache.fill(0.0);
        self.v_cache.fill(0.0);
        self.logits.fill(0.0);
        self.hidden.fill(0.0);
        self.x.fill(0.0);
    }
}

/// Opaque caller-owned scratch for the checked #704 attention canary façade.
/// Construction is outside the hot path; every checked step validates and
/// reuses these buffers without allocation.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub struct Gpt2AttentionCanaryWorkspace {
    normed: Vec<f32>,
    qkv: Vec<f32>,
    attn: Vec<f32>,
    proj: Vec<f32>,
    inner: Vec<f32>,
    mlp_out: Vec<f32>,
    scores: Vec<f32>,
}

impl Gpt2AttentionCanaryWorkspace {
    fn new(config: &Gpt2Config) -> Self {
        Self {
            normed: vec![0.0; config.n_embd],
            qkv: vec![0.0; 3 * config.n_embd],
            attn: vec![0.0; config.n_embd],
            proj: vec![0.0; config.n_embd],
            inner: vec![0.0; config.n_inner],
            mlp_out: vec![0.0; config.n_embd],
            scores: vec![0.0; config.seq_len],
        }
    }
}

fn canary_product(component: &'static str, factors: &[usize]) -> Result<usize, SourceUnavailable> {
    factors
        .iter()
        .try_fold(1usize, |product, &factor| product.checked_mul(factor))
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "GPT-2 attention canary geometry overflows usize: {component}"
            ))
        })
}

fn require_canary_length(
    component: &'static str,
    layer: Option<usize>,
    actual: usize,
    expected: usize,
) -> Result<(), SourceUnavailable> {
    if actual == expected {
        Ok(())
    } else {
        let layer = layer.map_or_else(String::new, |layer| format!(" layer {layer}"));
        Err(SourceUnavailable::new(format!(
            "GPT-2 attention canary{layer} {component} length {actual}, expected {expected}"
        )))
    }
}

impl Gpt2 {
    fn validate_attention_canary_model(&self) -> Result<(), SourceUnavailable> {
        let config = &self.cfg;
        if config.n_embd == 0 {
            return Err(SourceUnavailable::new(
                "invalid GPT-2 attention canary geometry: n_embd must be nonzero",
            ));
        }
        if config.n_layer == 0 {
            return Err(SourceUnavailable::new(
                "invalid GPT-2 attention canary geometry: n_layer must be nonzero",
            ));
        }
        if config.vocab == 0 {
            return Err(SourceUnavailable::new(
                "invalid GPT-2 attention canary geometry: vocab must be nonzero",
            ));
        }
        if config.n_head == 0 || !config.n_embd.is_multiple_of(config.n_head) {
            return Err(SourceUnavailable::new(
                "invalid GPT-2 attention canary geometry: n_head must be nonzero and divide n_embd",
            ));
        }
        if config.seq_len == 0 || config.seq_len > config.n_positions {
            return Err(SourceUnavailable::new(
                "invalid GPT-2 attention canary geometry: seq_len must be in 1..=n_positions",
            ));
        }

        let d = config.n_embd;
        let three_d = canary_product("3 * n_embd", &[3, d])?;
        let token_table = canary_product("vocab * n_embd", &[config.vocab, d])?;
        let position_table = canary_product("n_positions * n_embd", &[config.n_positions, d])?;
        let square = canary_product("n_embd * n_embd", &[d, d])?;
        let attention_projection = canary_product("n_embd * 3 * n_embd", &[d, three_d])?;
        let feed_forward = canary_product("n_embd * n_inner", &[d, config.n_inner])?;
        canary_product(
            "n_layer * seq_len * n_embd",
            &[config.n_layer, config.seq_len, d],
        )?;

        require_canary_length("model.wte", None, self.wte.len(), token_table)?;
        require_canary_length("model.wpe", None, self.wpe.len(), position_table)?;
        require_canary_length("model.layers", None, self.layers.len(), config.n_layer)?;
        require_canary_length("model.ln_f_w", None, self.ln_f_w.len(), d)?;
        require_canary_length("model.ln_f_b", None, self.ln_f_b.len(), d)?;
        for layer_index in 0..self.layers.len() {
            let layer = Some(layer_index);
            require_canary_length("ln1_w", layer, self.layers[layer_index].ln1_w.len(), d)?;
            require_canary_length("ln1_b", layer, self.layers[layer_index].ln1_b.len(), d)?;
            require_canary_length(
                "c_attn_w",
                layer,
                self.layers[layer_index].c_attn_w.len(),
                attention_projection,
            )?;
            require_canary_length(
                "c_attn_b",
                layer,
                self.layers[layer_index].c_attn_b.len(),
                three_d,
            )?;
            require_canary_length(
                "c_proj_w",
                layer,
                self.layers[layer_index].c_proj_w.len(),
                square,
            )?;
            require_canary_length(
                "c_proj_b",
                layer,
                self.layers[layer_index].c_proj_b.len(),
                d,
            )?;
            require_canary_length("ln2_w", layer, self.layers[layer_index].ln2_w.len(), d)?;
            require_canary_length("ln2_b", layer, self.layers[layer_index].ln2_b.len(), d)?;
            require_canary_length(
                "fc_w",
                layer,
                self.layers[layer_index].fc_w.len(),
                feed_forward,
            )?;
            require_canary_length(
                "fc_b",
                layer,
                self.layers[layer_index].fc_b.len(),
                config.n_inner,
            )?;
            require_canary_length(
                "mlp_w",
                layer,
                self.layers[layer_index].mlp_w.len(),
                feed_forward,
            )?;
            require_canary_length("mlp_b", layer, self.layers[layer_index].mlp_b.len(), d)?;
        }
        Ok(())
    }

    fn validate_attention_canary_inputs(
        &self,
        state: &Gpt2State,
        workspace: &Gpt2AttentionCanaryWorkspace,
        token: usize,
        pos: usize,
    ) -> Option<()> {
        self.validate_attention_canary_model().ok()?;
        let config = &self.cfg;
        if token >= config.vocab {
            return None;
        }
        if pos >= config.seq_len {
            return None;
        }

        let d = config.n_embd;
        let three_d = canary_product("3 * n_embd", &[3, d]).ok()?;
        let cache = canary_product(
            "n_layer * seq_len * n_embd",
            &[config.n_layer, config.seq_len, d],
        )
        .ok()?;
        [
            (state.k_cache.len(), cache),
            (state.v_cache.len(), cache),
            (state.logits.len(), config.vocab),
            (state.hidden.len(), d),
            (state.x.len(), d),
            (workspace.normed.len(), d),
            (workspace.qkv.len(), three_d),
            (workspace.attn.len(), d),
            (workspace.proj.len(), d),
            (workspace.inner.len(), config.n_inner),
            (workspace.mlp_out.len(), d),
            (workspace.scores.len(), config.seq_len),
        ]
        .into_iter()
        .all(|(actual, expected)| actual == expected)
        .then_some(())
    }

    /// One teacher-forced forward step at `pos` (0-based), leaving logits
    /// and the final hidden state in `st`. `capture` receives the
    /// post-block residual stream for each declared layer index, in
    /// ascending order — the #599 conformance-trace tap (a no-op closure
    /// captures nothing).
    pub fn forward(
        &self,
        st: &mut Gpt2State,
        token: usize,
        pos: usize,
        capture: &[usize],
        sink: &mut dyn FnMut(usize, &[f32]),
    ) {
        let d = self.cfg.n_embd;
        // token + learned absolute position embedding.
        for i in 0..d {
            st.x[i] = self.wte[token * d + i] + self.wpe[pos * d + i];
        }
        // scratch buffers reused across layers.
        let mut normed = vec![0.0f32; d];
        let mut qkv = vec![0.0f32; 3 * d];
        let mut attn = vec![0.0f32; d];
        let mut proj = vec![0.0f32; d];
        let mut inner = vec![0.0f32; self.cfg.n_inner];
        let mut mlp_out = vec![0.0f32; d];
        for l in 0..self.cfg.n_layer {
            self.block_forward(
                st,
                l,
                pos,
                &mut normed,
                &mut qkv,
                &mut attn,
                &mut proj,
                &mut inner,
                &mut mlp_out,
                None,
                None,
            );
            if capture.contains(&l) {
                sink(l, &st.x);
            }
        }
        self.finish_forward(st);
    }

    fn forward_with_attention_arithmetic_unchecked(
        &self,
        st: &mut Gpt2State,
        workspace: &mut Gpt2AttentionCanaryWorkspace,
        token: usize,
        pos: usize,
        arithmetic: crate::attention::AttentionArithmetic,
    ) -> crate::attention::AttentionArithmeticCensus {
        let d = self.cfg.n_embd;
        for i in 0..d {
            st.x[i] = self.wte[token * d + i] + self.wpe[pos * d + i];
        }
        let mut census = crate::attention::AttentionArithmeticCensus::default();
        for layer in 0..self.cfg.n_layer {
            let block = self.block_forward_with_attention_arithmetic(
                st,
                layer,
                pos,
                &mut workspace.normed,
                &mut workspace.qkv,
                &mut workspace.attn,
                &mut workspace.proj,
                &mut workspace.inner,
                &mut workspace.mlp_out,
                &mut workspace.scores[..=pos],
                arithmetic,
            );
            census.merge(block);
        }
        self.finish_forward_without_allocation(st);
        census
    }

    /// Allocate opaque scratch for the checked attention canary façade.
    #[doc(hidden)]
    pub fn attention_canary_workspace(
        &self,
    ) -> Result<Gpt2AttentionCanaryWorkspace, SourceUnavailable> {
        self.validate_attention_canary_model()?;
        Ok(Gpt2AttentionCanaryWorkspace::new(&self.cfg))
    }

    /// Execute one matched attention-canary step after validating every model,
    /// state, workspace, token, position, and derived slice extent. Validation
    /// completes before any mutable byte is touched, so `None` is failure
    /// atomic. Production [`Self::forward`] remains the normal executor.
    #[doc(hidden)]
    pub fn forward_attention_canary(
        &self,
        st: &mut Gpt2State,
        workspace: &mut Gpt2AttentionCanaryWorkspace,
        token: usize,
        pos: usize,
        mode: Gpt2AttentionCanaryMode,
    ) -> Option<Gpt2AttentionCanaryCensus> {
        self.validate_attention_canary_inputs(st, workspace, token, pos)?;
        let arithmetic = match mode {
            Gpt2AttentionCanaryMode::Conventional => {
                crate::attention::AttentionArithmetic::Conventional
            }
            Gpt2AttentionCanaryMode::Exact => crate::attention::AttentionArithmetic::Exact,
            Gpt2AttentionCanaryMode::CertifiedNative => {
                crate::attention::AttentionArithmetic::CertifiedNative
            }
        };
        Some(
            self.forward_with_attention_arithmetic_unchecked(st, workspace, token, pos, arithmetic)
                .into(),
        )
    }

    fn finish_forward_without_allocation(&self, st: &mut Gpt2State) {
        let d = self.cfg.n_embd;
        layer_norm(
            &st.x,
            &self.ln_f_w,
            &self.ln_f_b,
            self.cfg.layer_norm_eps,
            &mut st.hidden,
        );
        let hidden = &st.hidden;
        for (vocabulary_index, output) in st.logits.iter_mut().enumerate() {
            let row = &self.wte[vocabulary_index * d..(vocabulary_index + 1) * d];
            let mut accumulator = 0.0f32;
            for i in 0..d {
                accumulator += hidden[i] * row[i];
            }
            *output = accumulator;
        }
    }

    /// Final `ln_f` LayerNorm into `st.hidden` and the tied lm-head into
    /// `st.logits`. Shared by [`Gpt2::forward`] and
    /// [`Gpt2::forward_capturing_trace`] so a traced step finishes through
    /// the exact same arithmetic.
    fn finish_forward(&self, st: &mut Gpt2State) {
        let d = self.cfg.n_embd;
        layer_norm(
            &st.x,
            &self.ln_f_w,
            &self.ln_f_b,
            self.cfg.layer_norm_eps,
            &mut st.hidden,
        );
        let hidden = st.hidden.clone();
        for v in 0..self.cfg.vocab {
            let row = &self.wte[v * d..(v + 1) * d];
            let mut acc = 0.0f32;
            for i in 0..d {
                acc += hidden[i] * row[i];
            }
            st.logits[v] = acc;
        }
    }

    /// One teacher-forced forward step at `pos` with the #603 trace lanes
    /// captured through the exact executor path: the post-block residual
    /// stream, the current-position q/k/v, and the per-head softmax
    /// attention weights, each for the layer indices `request` declares.
    /// A traced step leaves the same logits, hidden state, and k/v caches
    /// as [`Gpt2::forward`] — the taps read the executor's own
    /// intermediates, they never recompute. Sinks fire in ascending layer
    /// order (heads ascending within a layer for the attention lane).
    pub fn forward_capturing_trace(
        &self,
        st: &mut Gpt2State,
        token: usize,
        pos: usize,
        request: &crate::TraceCaptureRequest<'_>,
        sinks: &mut crate::TraceCaptureSinks<'_, '_>,
    ) {
        let d = self.cfg.n_embd;
        // token + learned absolute position embedding.
        for i in 0..d {
            st.x[i] = self.wte[token * d + i] + self.wpe[pos * d + i];
        }
        let mut normed = vec![0.0f32; d];
        let mut qkv = vec![0.0f32; 3 * d];
        let mut attn = vec![0.0f32; d];
        let mut proj = vec![0.0f32; d];
        let mut inner = vec![0.0f32; self.cfg.n_inner];
        let mut mlp_out = vec![0.0f32; d];
        for l in 0..self.cfg.n_layer {
            self.block_forward(
                st,
                l,
                pos,
                &mut normed,
                &mut qkv,
                &mut attn,
                &mut proj,
                &mut inner,
                &mut mlp_out,
                Some(request),
                Some(&mut *sinks),
            );
            if request.residual_layers.contains(&l) {
                (sinks.residual)(l, &st.x);
            }
        }
        self.finish_forward(st);
    }

    #[allow(clippy::too_many_arguments)]
    fn block_forward_with_attention_arithmetic(
        &self,
        st: &mut Gpt2State,
        layer_index: usize,
        pos: usize,
        normed: &mut [f32],
        qkv: &mut [f32],
        attn: &mut [f32],
        proj: &mut [f32],
        inner: &mut [f32],
        mlp_out: &mut [f32],
        scores: &mut [f32],
        arithmetic: crate::attention::AttentionArithmetic,
    ) -> crate::attention::AttentionArithmeticCensus {
        let d = self.cfg.n_embd;
        let layer = &self.layers[layer_index];

        layer_norm(
            &st.x,
            &layer.ln1_w,
            &layer.ln1_b,
            self.cfg.layer_norm_eps,
            normed,
        );
        conv1d(normed, &layer.c_attn_w, &layer.c_attn_b, 3 * d, qkv);
        let census = self.block_attention_with_arithmetic(
            st,
            layer_index,
            pos,
            qkv,
            attn,
            scores,
            arithmetic,
        );
        conv1d(attn, &layer.c_proj_w, &layer.c_proj_b, d, proj);
        for (value, &projection) in st.x.iter_mut().zip(&proj[..d]) {
            *value += projection;
        }

        layer_norm(
            &st.x,
            &layer.ln2_w,
            &layer.ln2_b,
            self.cfg.layer_norm_eps,
            normed,
        );
        conv1d(normed, &layer.fc_w, &layer.fc_b, self.cfg.n_inner, inner);
        for value in inner.iter_mut() {
            *value = gelu_new(*value);
        }
        conv1d(inner, &layer.mlp_w, &layer.mlp_b, d, mlp_out);
        for (value, &mlp) in st.x.iter_mut().zip(&mlp_out[..d]) {
            *value += mlp;
        }
        census
    }

    #[allow(clippy::too_many_arguments)]
    fn block_forward(
        &self,
        st: &mut Gpt2State,
        l: usize,
        pos: usize,
        normed: &mut [f32],
        qkv: &mut [f32],
        attn: &mut [f32],
        proj: &mut [f32],
        inner: &mut [f32],
        mlp_out: &mut [f32],
        request: Option<&crate::TraceCaptureRequest<'_>>,
        sinks: Option<&mut crate::TraceCaptureSinks<'_, '_>>,
    ) {
        let d = self.cfg.n_embd;
        let layer = &self.layers[l];

        // --- attention ---
        layer_norm(
            &st.x,
            &layer.ln1_w,
            &layer.ln1_b,
            self.cfg.layer_norm_eps,
            normed,
        );
        conv1d(normed, &layer.c_attn_w, &layer.c_attn_b, 3 * d, qkv);
        self.block_attention(st, l, pos, qkv, attn, request, sinks);
        conv1d(attn, &layer.c_proj_w, &layer.c_proj_b, d, proj);
        for (xi, &p) in st.x.iter_mut().zip(&proj[..d]) {
            *xi += p;
        }

        // --- MLP ---
        layer_norm(
            &st.x,
            &layer.ln2_w,
            &layer.ln2_b,
            self.cfg.layer_norm_eps,
            normed,
        );
        conv1d(normed, &layer.fc_w, &layer.fc_b, self.cfg.n_inner, inner);
        for v in inner.iter_mut() {
            *v = gelu_new(*v);
        }
        conv1d(inner, &layer.mlp_w, &layer.mlp_b, d, mlp_out);
        for (xi, &m) in st.x.iter_mut().zip(&mlp_out[..d]) {
            *xi += m;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn block_attention_with_arithmetic(
        &self,
        st: &mut Gpt2State,
        layer: usize,
        pos: usize,
        qkv: &[f32],
        attn: &mut [f32],
        scores: &mut [f32],
        arithmetic: crate::attention::AttentionArithmetic,
    ) -> crate::attention::AttentionArithmeticCensus {
        let d = self.cfg.n_embd;
        let head_size = self.cfg.head_size();
        let sequence_length = self.cfg.seq_len;
        let base = (layer * sequence_length + pos) * d;
        st.k_cache[base..base + d].copy_from_slice(&qkv[d..2 * d]);
        st.v_cache[base..base + d].copy_from_slice(&qkv[2 * d..3 * d]);

        debug_assert_eq!(scores.len(), pos + 1);
        let mut census = crate::attention::AttentionArithmeticCensus::default();
        let key_cache = &st.k_cache[layer * sequence_length * d..(layer + 1) * sequence_length * d];
        let value_cache =
            &st.v_cache[layer * sequence_length * d..(layer + 1) * sequence_length * d];
        for head in 0..self.cfg.n_head {
            let query = &qkv[head * head_size..(head + 1) * head_size];
            census.qk.merge(attention_weights_with_arithmetic(
                scores,
                query,
                key_cache,
                head * head_size,
                d,
                arithmetic,
            ));
            let output = &mut attn[head * head_size..(head + 1) * head_size];
            census.value.merge(
                crate::attention::head_attention_value_aggregate_with_arithmetic(
                    output,
                    scores,
                    value_cache,
                    head * head_size,
                    d,
                    arithmetic,
                ),
            );
        }
        census
    }

    /// The per-sequence attention sub-block shared by [`Gpt2::block_forward`]
    /// (serial) and [`Gpt2::forward_batch`] (batched): cache this position's
    /// k/v from the fused `qkv` projection, then for each head compute the
    /// scaled dot-product scores, the max-subtracted softmax (normalized in
    /// place), and the value aggregation into `attn`. Optional #603 taps
    /// (q/k/v and the per-head weights) fire for a requested layer. Sharing
    /// it keeps the serial and batched executors bit-identical by
    /// construction.
    #[allow(clippy::too_many_arguments)]
    fn block_attention(
        &self,
        st: &mut Gpt2State,
        l: usize,
        pos: usize,
        qkv: &[f32],
        attn: &mut [f32],
        request: Option<&crate::TraceCaptureRequest<'_>>,
        mut sinks: Option<&mut crate::TraceCaptureSinks<'_, '_>>,
    ) {
        let d = self.cfg.n_embd;
        let hs = self.cfg.head_size();
        let seq = self.cfg.seq_len;
        // store this position's k, v (thirds 1 and 2 of the fused qkv).
        let base = (l * seq + pos) * d;
        st.k_cache[base..base + d].copy_from_slice(&qkv[d..2 * d]);
        st.v_cache[base..base + d].copy_from_slice(&qkv[2 * d..3 * d]);
        // #603 q/k/v tap at this position (q = all heads, then the just-cached
        // k and v rows), emitted only for a requested layer. GPT-2 is plain
        // MHA (kv heads == query heads), so every row is `d` wide.
        if let Some(request) = request {
            if request.qkv_layers.contains(&l) {
                if let Some(sinks) = sinks.as_deref_mut() {
                    (sinks.qkv)(l, &qkv[..d], &qkv[d..2 * d], &qkv[2 * d..3 * d]);
                }
            }
        }
        let mut scores = vec![0.0f32; pos + 1];
        let key_cache = &st.k_cache[l * seq * d..(l + 1) * seq * d];
        let value_cache = &st.v_cache[l * seq * d..(l + 1) * seq * d];
        for h in 0..self.cfg.n_head {
            let qh = &qkv[h * hs..(h + 1) * hs];
            let _ = attention_weights_with_arithmetic(
                &mut scores,
                qh,
                key_cache,
                h * hs,
                d,
                crate::attention::AttentionArithmetic::CertifiedNative,
            );
            // #603 per-head attention-weight tap over positions 0..=pos.
            if let Some(request) = request {
                if request.attention_layers.contains(&l) {
                    if let Some(sinks) = sinks.as_deref_mut() {
                        (sinks.attention)(l, h, &scores);
                    }
                }
            }
            let ao = &mut attn[h * hs..(h + 1) * hs];
            let _ = crate::attention::head_attention_value_aggregate_with_arithmetic(
                ao,
                &scores,
                value_cache,
                h * hs,
                d,
                crate::attention::AttentionArithmetic::CertifiedNative,
            );
        }
    }

    /// Batched forward: advance `states.len()` independent sequences by one
    /// position each — sequence `bi` steps `tokens[bi]` at `positions[bi]`
    /// against its own k/v cache in `states[bi]`. The projection Conv1Ds
    /// (`c_attn`, `c_proj`, `fc`, `mlp`) run once over the whole batch via
    /// the amortized [`conv1d_batched`], and the tied lm-head reuses each
    /// `wte` row across the batch — the same memory-amortization Llama's
    /// `forward_batch` gets from `matmul_batched`. Every per-sequence op
    /// (embedding, LayerNorm, attention via the shared
    /// [`Gpt2::block_attention`], GELU, residual) mirrors [`Gpt2::forward`]
    /// and keeps the serial accumulation order, so this is bit-identical to
    /// calling `forward` on each sequence.
    // Sequence-major index loops are deliberate here: the accumulation order
    // must match the serial `forward` byte-for-byte, and the offsets index
    // several stacked scratch buffers in lockstep.
    #[allow(clippy::needless_range_loop)]
    pub fn forward_batch(&self, states: &mut [Gpt2State], tokens: &[usize], positions: &[usize]) {
        let d = self.cfg.n_embd;
        let inner_dim = self.cfg.n_inner;
        let b = states.len();
        debug_assert_eq!(tokens.len(), b);
        debug_assert_eq!(positions.len(), b);
        if b == 0 {
            return;
        }

        let mut normed = vec![0.0f32; b * d];
        let mut qkv = vec![0.0f32; b * 3 * d];
        let mut attn = vec![0.0f32; b * d];
        let mut proj = vec![0.0f32; b * d];
        let mut inner = vec![0.0f32; b * inner_dim];
        let mut mlp_out = vec![0.0f32; b * d];

        // token + learned absolute position embedding, per sequence.
        for bi in 0..b {
            let (token, pos) = (tokens[bi], positions[bi]);
            let x = &mut states[bi].x;
            for i in 0..d {
                x[i] = self.wte[token * d + i] + self.wpe[pos * d + i];
            }
        }

        for l in 0..self.cfg.n_layer {
            let layer = &self.layers[l];
            for bi in 0..b {
                layer_norm(
                    &states[bi].x,
                    &layer.ln1_w,
                    &layer.ln1_b,
                    self.cfg.layer_norm_eps,
                    &mut normed[bi * d..(bi + 1) * d],
                );
            }
            conv1d_batched(
                &mut qkv,
                &normed,
                &layer.c_attn_w,
                &layer.c_attn_b,
                3 * d,
                d,
                b,
            );
            for bi in 0..b {
                self.block_attention(
                    &mut states[bi],
                    l,
                    positions[bi],
                    &qkv[bi * 3 * d..(bi + 1) * 3 * d],
                    &mut attn[bi * d..(bi + 1) * d],
                    None,
                    None,
                );
            }
            conv1d_batched(&mut proj, &attn, &layer.c_proj_w, &layer.c_proj_b, d, d, b);
            for bi in 0..b {
                let x = &mut states[bi].x;
                for i in 0..d {
                    x[i] += proj[bi * d + i];
                }
            }
            for bi in 0..b {
                layer_norm(
                    &states[bi].x,
                    &layer.ln2_w,
                    &layer.ln2_b,
                    self.cfg.layer_norm_eps,
                    &mut normed[bi * d..(bi + 1) * d],
                );
            }
            conv1d_batched(
                &mut inner,
                &normed,
                &layer.fc_w,
                &layer.fc_b,
                inner_dim,
                d,
                b,
            );
            for v in inner.iter_mut() {
                *v = gelu_new(*v);
            }
            conv1d_batched(
                &mut mlp_out,
                &inner,
                &layer.mlp_w,
                &layer.mlp_b,
                d,
                inner_dim,
                b,
            );
            for bi in 0..b {
                let x = &mut states[bi].x;
                for i in 0..d {
                    x[i] += mlp_out[bi * d + i];
                }
            }
        }

        // final ln_f into each state's hidden, then the tied lm-head with
        // each wte row reused across the batch (naive sequential dot per
        // sequence — bit-identical to `finish_forward`).
        for bi in 0..b {
            let st = &mut states[bi];
            layer_norm(
                &st.x,
                &self.ln_f_w,
                &self.ln_f_b,
                self.cfg.layer_norm_eps,
                &mut st.hidden,
            );
        }
        for v in 0..self.cfg.vocab {
            let row = &self.wte[v * d..(v + 1) * d];
            for bi in 0..b {
                let st = &mut states[bi];
                let mut acc = 0.0f32;
                for i in 0..d {
                    acc += st.hidden[i] * row[i];
                }
                st.logits[v] = acc;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gpt2-tiny")
    }

    fn assert_canary_unchanged(
        state: &Gpt2State,
        state_before: &Gpt2State,
        workspace: &Gpt2AttentionCanaryWorkspace,
        workspace_before: &Gpt2AttentionCanaryWorkspace,
    ) {
        assert_eq!(state, state_before, "invalid canary call mutated state");
        assert_eq!(
            workspace, workspace_before,
            "invalid canary call mutated workspace"
        );
    }

    #[test]
    fn attention_control_preserves_gpt2_reciprocal_normalization_order() {
        let raw = [
            f32::from_bits(0xc2b3_f6f4),
            f32::from_bits(0xc2bc_870f),
            f32::from_bits(0xc213_2cb3),
            f32::from_bits(0x416b_d2a8),
            f32::from_bits(0xc15f_22ae),
        ];
        let query = [1.0f32, 0.0];
        let keys: Vec<f32> = raw.iter().flat_map(|&score| [score, 0.0]).collect();
        let mut exact = [0.0f32; 5];
        attention_weights_with_arithmetic(
            &mut exact,
            &query,
            &keys,
            0,
            2,
            crate::attention::AttentionArithmetic::Exact,
        );
        let mut certified = [0.0f32; 5];
        attention_weights_with_arithmetic(
            &mut certified,
            &query,
            &keys,
            0,
            2,
            crate::attention::AttentionArithmetic::CertifiedNative,
        );

        let reciprocal = 1.0 / 2.0f32.sqrt();
        let mut expected = raw;
        let mut max = f32::NEG_INFINITY;
        for score in &mut expected {
            *score *= reciprocal;
            if *score > max {
                max = *score;
            }
        }
        let mut sum = 0.0f32;
        for score in &mut expected {
            *score = (*score - max).exp();
            sum += *score;
        }
        let inverse = 1.0 / sum;
        for score in &mut expected {
            *score *= inverse;
        }
        assert_eq!(exact.map(f32::to_bits), expected.map(f32::to_bits));
        assert_eq!(certified.map(f32::to_bits), exact.map(f32::to_bits));
    }

    #[test]
    fn checked_attention_canary_rejects_every_invalid_input_before_mutation() {
        let mut model = Gpt2::load(fixture_dir(), None).expect("load tiny GPT-2 fixture");

        let mut state = Gpt2State::new(&model.cfg);
        let mut workspace = model
            .attention_canary_workspace()
            .expect("valid model admits canary scratch");
        let state_before = state.clone();
        let workspace_before = workspace.clone();
        let result = model.forward_attention_canary(
            &mut state,
            &mut workspace,
            model.cfg.vocab,
            0,
            Gpt2AttentionCanaryMode::CertifiedNative,
        );
        assert_eq!(result, None, "out-of-range token must have no product");
        assert_canary_unchanged(&state, &state_before, &workspace, &workspace_before);

        let result = model.forward_attention_canary(
            &mut state,
            &mut workspace,
            0,
            model.cfg.seq_len,
            Gpt2AttentionCanaryMode::CertifiedNative,
        );
        assert_eq!(result, None, "out-of-range position must have no product");
        assert_canary_unchanged(&state, &state_before, &workspace, &workspace_before);

        state.logits.pop();
        let state_before = state.clone();
        let workspace_before = workspace.clone();
        let result = model.forward_attention_canary(
            &mut state,
            &mut workspace,
            0,
            0,
            Gpt2AttentionCanaryMode::CertifiedNative,
        );
        assert_eq!(result, None, "invalid state must have no canary product");
        assert_canary_unchanged(&state, &state_before, &workspace, &workspace_before);

        state = Gpt2State::new(&model.cfg);
        workspace.scores.pop();
        let state_before = state.clone();
        let workspace_before = workspace.clone();
        let result = model.forward_attention_canary(
            &mut state,
            &mut workspace,
            0,
            0,
            Gpt2AttentionCanaryMode::CertifiedNative,
        );
        assert_eq!(
            result, None,
            "invalid workspace must have no canary product"
        );
        assert_canary_unchanged(&state, &state_before, &workspace, &workspace_before);

        workspace = Gpt2AttentionCanaryWorkspace::new(&model.cfg);
        let state_before = state.clone();
        let workspace_before = workspace.clone();
        let original_head_count = model.cfg.n_head;
        model.cfg.n_head = 0;
        let error = model
            .attention_canary_workspace()
            .expect_err("invalid model geometry must refuse workspace construction");
        assert_eq!(
            error.reason,
            "invalid GPT-2 attention canary geometry: n_head must be nonzero and divide n_embd"
        );
        let result = model.forward_attention_canary(
            &mut state,
            &mut workspace,
            0,
            0,
            Gpt2AttentionCanaryMode::CertifiedNative,
        );
        assert_eq!(result, None, "invalid model must have no canary product");
        assert_canary_unchanged(&state, &state_before, &workspace, &workspace_before);

        model.cfg.n_head = original_head_count;
        let original_layer_count = model.cfg.n_layer;
        model.cfg.n_layer = 0;
        let error = model
            .attention_canary_workspace()
            .expect_err("zero-layer model must refuse workspace construction");
        assert_eq!(
            error.reason,
            "invalid GPT-2 attention canary geometry: n_layer must be nonzero"
        );
        let result = model.forward_attention_canary(
            &mut state,
            &mut workspace,
            0,
            0,
            Gpt2AttentionCanaryMode::CertifiedNative,
        );
        assert_eq!(
            result, None,
            "zero-layer canary must be non-vacuously absent"
        );
        assert_canary_unchanged(&state, &state_before, &workspace, &workspace_before);

        model.cfg.n_layer = original_layer_count;
        model.cfg.vocab = 0;
        let error = model
            .attention_canary_workspace()
            .expect_err("zero-vocabulary model must refuse workspace construction");
        assert_eq!(
            error.reason,
            "invalid GPT-2 attention canary geometry: vocab must be nonzero"
        );
        let result = model.forward_attention_canary(
            &mut state,
            &mut workspace,
            0,
            0,
            Gpt2AttentionCanaryMode::CertifiedNative,
        );
        assert_eq!(
            result, None,
            "zero-vocabulary canary must be non-vacuously absent"
        );
        assert_canary_unchanged(&state, &state_before, &workspace, &workspace_before);
    }

    #[test]
    fn public_canary_census_overflow_fails_closed_and_atomically() {
        let overflowing_fallbacks = Gpt2AttentionCanaryDotCensus {
            fallback_nonfinite: usize::MAX,
            fallback_zero: 1,
            ..Gpt2AttentionCanaryDotCensus::default()
        };
        assert_eq!(overflowing_fallbacks.fallbacks(), None);

        let mut census = Gpt2AttentionCanaryCensus {
            qk: Gpt2AttentionCanaryDotCensus {
                lanes: 7,
                certified: 7,
                ..Gpt2AttentionCanaryDotCensus::default()
            },
            value: Gpt2AttentionCanaryDotCensus {
                lanes: usize::MAX,
                certified: 11,
                ..Gpt2AttentionCanaryDotCensus::default()
            },
        };
        let before = census;
        let other = Gpt2AttentionCanaryCensus {
            qk: Gpt2AttentionCanaryDotCensus {
                lanes: 1,
                certified: 1,
                ..Gpt2AttentionCanaryDotCensus::default()
            },
            value: Gpt2AttentionCanaryDotCensus {
                lanes: 1,
                ..Gpt2AttentionCanaryDotCensus::default()
            },
        };
        assert_eq!(census.merge(other), None);
        assert_eq!(census, before, "failed merge mutated the public census");
    }

    fn f32_vec(value: &serde_json::Value) -> Vec<f32> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect()
    }

    fn assert_close(got: &[f32], want: &[f32], tol: f32, ctx: &str) {
        assert_eq!(got.len(), want.len(), "{ctx}: length");
        let mut worst = 0.0f32;
        for (i, (&g, &w)) in got.iter().zip(want).enumerate() {
            let d = (g - w).abs();
            if d > worst {
                worst = d;
            }
            assert!(d <= tol, "{ctx}[{i}]: got {g}, want {w}, |Δ|={d} > {tol}");
        }
        eprintln!("{ctx}: worst |Δ| = {worst:e}");
    }

    fn argmax(xs: &[f32]) -> usize {
        let mut best = 0;
        for i in 1..xs.len() {
            if xs[i] > xs[best] {
                best = i;
            }
        }
        best
    }

    /// The Rust GPT-2 executor reproduces the INDEPENDENT numpy reference
    /// (scripts/gen_gpt2_tiny_fixture.py) within tolerance across every
    /// per-layer residual, the final hidden state, the full logit vector,
    /// and the argmax token — for multiple prompt lengths. This is the
    /// parity that makes "faithful executor" a measured claim.
    #[test]
    fn tiny_gpt2_matches_numpy_reference() {
        let dir = fixture_dir();
        let golden: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("golden.json")).unwrap()).unwrap();
        let model = Gpt2::load(&dir, None).expect("load tiny gpt2 snapshot");
        let n_layer = model.cfg.n_layer;
        let mut st = Gpt2State::new(&model.cfg);
        let capture: Vec<usize> = (0..n_layer).collect();

        for case in golden["cases"].as_array().unwrap() {
            st.reset();
            let tokens: Vec<usize> = case["tokens"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as usize)
                .collect();
            let mut captured: Vec<Vec<f32>> = vec![Vec::new(); n_layer];
            for (pos, &tok) in tokens.iter().enumerate() {
                if pos + 1 == tokens.len() {
                    model.forward(&mut st, tok, pos, &capture, &mut |l, x| {
                        captured[l] = x.to_vec();
                    });
                } else {
                    model.forward(&mut st, tok, pos, &[], &mut |_, _| {});
                }
            }

            let per_layer = case["per_layer"].as_array().unwrap();
            for (l, layer_golden) in per_layer.iter().enumerate() {
                assert_close(
                    &captured[l],
                    &f32_vec(layer_golden),
                    2e-3,
                    &format!("tokens {tokens:?} per_layer[{l}]"),
                );
            }
            assert_close(&st.hidden, &f32_vec(&case["hidden"]), 2e-3, "hidden");
            assert_close(&st.logits, &f32_vec(&case["logits"]), 3e-3, "logits");

            let top_token = case["top_k"].as_array().unwrap()[0].as_array().unwrap()[0]
                .as_u64()
                .unwrap() as usize;
            assert_eq!(argmax(&st.logits), top_token, "top-1 token for {tokens:?}");
        }
    }

    /// #668: the GPT-2 oracle reports the truthful learned-absolute
    /// operator record, and it resolves through the versioned registry.
    /// Its positional action is NOT RoPE, so it is a distinct identity
    /// from current `standard-source-attention/2` — reusing that record would be
    /// a false operator identity.
    #[test]
    fn gpt2_oracle_reports_learned_absolute_operator() {
        use crate::attention::{operator_spec, AttentionOperatorSpec};
        use crate::TeacherOracle;

        let oracle = HuggingFaceGpt2Oracle::load(fixture_dir()).expect("load tiny gpt2 oracle");
        let spec = oracle
            .attention_operator_spec()
            .expect("gpt2 oracle declares an attention operator");

        assert_eq!(
            spec,
            AttentionOperatorSpec::learned_absolute_source_attention()
        );
        assert_eq!(spec.id, AttentionOperatorSpec::LEARNED_ABSOLUTE_ID);
        assert_eq!(
            spec.version,
            AttentionOperatorSpec::LEARNED_ABSOLUTE_VERSION
        );

        // Resolvable through the registry it is registered in.
        let resolved = operator_spec(&spec.id, spec.version).expect("registered operator");
        assert_eq!(resolved, spec);

        // A distinct identity from the RoPE standard operator: the
        // positional action is the discriminating field, and the digest
        // over the declared identity differs accordingly.
        let standard = AttentionOperatorSpec::standard();
        assert_ne!(spec.positional_action, standard.positional_action);
        assert_ne!(spec.implementation_digest, standard.implementation_digest);
    }

    /// #601 (item 4 of #657): the GPT-2 oracle declares the byte-level BPE
    /// adapter family `hf-byte-bpe/1` it actually tokenizes with, distinct
    /// from the generic `huggingface-tokenizer` the Llama oracle reports —
    /// so a compiled GPT-2 bundle's tokenizer identity is not conflated with
    /// any other HF source. The `(family, version)` rendering matches the
    /// core `hf_bpe` registry that refuses every other pair by name.
    #[test]
    fn gpt2_oracle_reports_byte_bpe_tokenizer_identity() {
        use crate::RepresentationSource;

        let oracle = HuggingFaceGpt2Oracle::load(fixture_dir()).expect("load tiny gpt2 oracle");
        assert_eq!(oracle.tokenizer_address(), "hf-byte-bpe/1");
        // Distinct from the generic Hugging Face tokenizer identity the Llama
        // oracle declares — the whole point of item 4.
        assert_ne!(oracle.tokenizer_address(), "huggingface-tokenizer");
    }

    /// #667: a #603-traced GPT-2 step produces logits bit-identical to the
    /// plain forward, and the residual/qkv/attention taps fire only for the
    /// requested layers with the declared shapes (residual `n_embd`; q/k/v
    /// each `n_embd` wide — plain MHA; per-head attention weights over
    /// `0..=pos`, summing to 1).
    #[test]
    fn forward_capturing_trace_matches_plain_forward_and_bounds_taps() {
        let dir = fixture_dir();
        let model = Gpt2::load(&dir, None).expect("load tiny gpt2 snapshot");
        let cfg_layers = model.cfg.n_layer;
        let heads = model.cfg.n_head;
        let d = model.cfg.n_embd;
        // Safe token ids and positions for the tiny fixture.
        let steps = 5.min(model.cfg.seq_len);
        let tokens: Vec<usize> = (0..steps).map(|i| i % model.cfg.vocab).collect();

        // Reference: plain forward logits at each position.
        let mut plain = Gpt2State::new(&model.cfg);
        plain.reset();
        let mut plain_logits: Vec<Vec<f32>> = Vec::new();
        for (pos, &tok) in tokens.iter().enumerate() {
            model.forward(&mut plain, tok, pos, &[], &mut |_, _| {});
            plain_logits.push(plain.logits.clone());
        }

        // Bounded capture request.
        let residual_layers: Vec<usize> = vec![0, cfg_layers - 1];
        let qkv_layers: Vec<usize> = vec![0];
        let attn_layer = if cfg_layers > 1 { 1 } else { 0 };
        let attention_layers: Vec<usize> = vec![attn_layer];
        let request = crate::TraceCaptureRequest {
            residual_layers: &residual_layers,
            qkv_layers: &qkv_layers,
            attention_layers: &attention_layers,
        };

        let mut traced = Gpt2State::new(&model.cfg);
        traced.reset();
        for (pos, &tok) in tokens.iter().enumerate() {
            let mut residual_hits: Vec<usize> = Vec::new();
            let mut qkv_hits: Vec<(usize, usize, usize, usize)> = Vec::new();
            let mut attention_hits: Vec<(usize, usize, usize, f32)> = Vec::new();
            {
                let mut residual = |l: usize, x: &[f32]| {
                    assert_eq!(x.len(), d, "residual width is n_embd");
                    residual_hits.push(l);
                };
                let mut qkv = |l: usize, q: &[f32], k: &[f32], v: &[f32]| {
                    qkv_hits.push((l, q.len(), k.len(), v.len()));
                };
                let mut attention = |l: usize, h: usize, w: &[f32]| {
                    let sum: f32 = w.iter().sum();
                    attention_hits.push((l, h, w.len(), sum));
                };
                let mut sinks = crate::TraceCaptureSinks {
                    residual: &mut residual,
                    qkv: &mut qkv,
                    attention: &mut attention,
                };
                model.forward_capturing_trace(&mut traced, tok, pos, &request, &mut sinks);
            }

            // Logits are bit-identical to the plain forward.
            let want: Vec<u32> = plain_logits[pos].iter().map(|v| v.to_bits()).collect();
            let got: Vec<u32> = traced.logits.iter().map(|v| v.to_bits()).collect();
            assert_eq!(
                got, want,
                "traced logits at pos {pos} equal the plain forward"
            );

            // Residual tap fires exactly for the requested layers, in order.
            assert_eq!(
                residual_hits, residual_layers,
                "residual layers at pos {pos}"
            );

            // q/k/v tap: once for the one requested layer, each row n_embd wide.
            assert_eq!(qkv_hits.len(), 1, "one qkv hit at pos {pos}");
            assert_eq!(qkv_hits[0], (0, d, d, d), "qkv layer + widths at pos {pos}");

            // Attention tap: one hit per head for the requested layer, weights
            // over 0..=pos and summing to 1.
            assert_eq!(attention_hits.len(), heads, "one attention hit per head");
            for (idx, (l, h, wlen, sum)) in attention_hits.iter().enumerate() {
                assert_eq!(*l, attn_layer, "attention layer");
                assert_eq!(*h, idx, "heads ascending");
                assert_eq!(*wlen, pos + 1, "attention weights span 0..=pos");
                assert!((sum - 1.0).abs() < 1e-4, "softmax weights sum to 1: {sum}");
            }
        }
    }

    /// #667: GPT-2's batched forward is bit-identical to advancing each
    /// sequence through the serial `forward` — the amortized conv1d/lm-head
    /// keep the serial accumulation order, and each sequence reads its own
    /// `positions[bi]` and k/v cache. Covers a lockstep batch (all at the
    /// same position) and a divergent batch (each sequence at a different
    /// position).
    #[test]
    #[allow(clippy::needless_range_loop)]
    fn forward_batch_matches_serial_forward() {
        let dir = fixture_dir();
        let model = Gpt2::load(&dir, None).expect("load tiny gpt2 snapshot");
        let steps = 4.min(model.cfg.seq_len).max(1);
        let vocab = model.cfg.vocab;
        let seqs: [Vec<usize>; 3] = [
            (0..steps).map(|i| i % vocab).collect(),
            (0..steps).map(|i| (i * 2 + 1) % vocab).collect(),
            (0..steps).map(|i| (i * 3 + 2) % vocab).collect(),
        ];

        // Serial reference: each sequence's logits at each position.
        let mut serial: Vec<Vec<Vec<f32>>> = Vec::new();
        for seq in &seqs {
            let mut st = Gpt2State::new(&model.cfg);
            st.reset();
            let mut per_pos = Vec::new();
            for (pos, &tok) in seq.iter().enumerate() {
                model.forward(&mut st, tok, pos, &[], &mut |_, _| {});
                per_pos.push(st.logits.clone());
            }
            serial.push(per_pos);
        }

        let fresh = || {
            let mut s = Gpt2State::new(&model.cfg);
            s.reset();
            s
        };
        let bits = |xs: &[f32]| -> Vec<u32> { xs.iter().map(|v| v.to_bits()).collect() };

        // Lockstep: advance all three together, position by position.
        {
            let mut states: Vec<Gpt2State> = (0..seqs.len()).map(|_| fresh()).collect();
            for pos in 0..steps {
                let tokens: Vec<usize> = seqs.iter().map(|s| s[pos]).collect();
                let positions = vec![pos; seqs.len()];
                model.forward_batch(&mut states, &tokens, &positions);
                for (b, st) in states.iter().enumerate() {
                    assert_eq!(
                        bits(&st.logits),
                        bits(&serial[b][pos]),
                        "lockstep batched seq {b} pos {pos} equals serial"
                    );
                }
            }
        }

        // Divergent positions in one batch: seq b targets a different pos.
        if steps >= 3 {
            let targets = [2usize, 1, 0];
            let mut states: Vec<Gpt2State> = (0..seqs.len()).map(|_| fresh()).collect();
            // Pre-fill each state's k/v cache serially up to its target - 1.
            for (b, &target) in targets.iter().enumerate() {
                for pos in 0..target {
                    model.forward(&mut states[b], seqs[b][pos], pos, &[], &mut |_, _| {});
                }
            }
            let tokens: Vec<usize> = targets
                .iter()
                .enumerate()
                .map(|(b, &t)| seqs[b][t])
                .collect();
            let positions = targets.to_vec();
            model.forward_batch(&mut states, &tokens, &positions);
            for (b, &target) in targets.iter().enumerate() {
                assert_eq!(
                    bits(&states[b].logits),
                    bits(&serial[b][target]),
                    "divergent batched seq {b} pos {target} equals serial"
                );
            }
        }
    }

    /// Loading a snapshot whose config declares a non-GPT-2 architecture is
    /// refused at the #599 gate before any weight is read.
    #[test]
    fn non_gpt2_config_is_refused() {
        let tmp = std::env::temp_dir().join(format!("uor-r4-gpt2-neg-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("config.json"),
            br#"{"model_type":"llama","n_embd":32,"n_head":4,"n_layer":2,"n_positions":16,"vocab_size":24}"#,
        )
        .unwrap();
        let err = Gpt2::load(&tmp, None)
            .err()
            .expect("a llama config must be refused by the GPT-2 adapter");
        assert!(
            matches!(err.kind, SourceIngestKind::UnsupportedConfigFeature { .. }),
            "expected UnsupportedConfigFeature, got {:?}",
            err.kind
        );
    }
}

/// The GPT-2 adapter: a [`Gpt2`] executor plus its recurrent state, exposed
/// through the architecture-neutral [`crate::TeacherOracle`] two-surface
/// trait. Nothing GPT-2-specific escapes this type; the compiler consumes
/// only the trait, so no downstream crate branches on `gpt2`, `c_attn`, or
/// `wte`. The compiled representation width it presents is
/// [`crate::geometry::COMPILED_WIDTH`] (288); `embedding` projects the
/// source-width rows down through the #600 `bucket-average/1` projection,
/// exactly as the Llama adapter does for its own source width.
///
/// The full #599–#606 source-parity surface is now declared (all items
/// enumerated in issue #657 have landed): the #602 attention-operator record
/// ([`Self::attention_operator_spec`] → learned-absolute), the #603
/// trace-capture surface ([`Self::trace_capture_geometry`] /
/// [`Self::step_with_trace_capture`] over [`Gpt2::forward_capturing_trace`]),
/// the #601 tokenizer identity ([`Self::tokenizer_address`] →
/// `hf-byte-bpe/1`), and the graph-cli/compiler dispatch that lets a pinned
/// GPT-2 source compile to an R4G1 artifact end to end.
pub struct HuggingFaceGpt2Oracle {
    model: Gpt2,
    state: Gpt2State,
    bos_token: usize,
    eos_token: usize,
}

impl HuggingFaceGpt2Oracle {
    /// Load the GPT-2 teacher from a snapshot directory.
    pub fn load(source: impl AsRef<std::path::Path>) -> Result<Self, SourceUnavailable> {
        Self::build(Gpt2::load(source, None)?)
    }

    /// Load with a bounded teacher context (short trajectories are all the
    /// compiler needs; the deployed runtime consumes an eight-token window).
    pub fn load_with_sequence_length(
        source: impl AsRef<std::path::Path>,
        sequence_length: usize,
    ) -> Result<Self, SourceUnavailable> {
        if sequence_length == 0 {
            return Err(SourceUnavailable::new(
                "teacher sequence length must be greater than zero",
            ));
        }
        Self::build(Gpt2::load(source, Some(sequence_length))?)
    }

    fn build(model: Gpt2) -> Result<Self, SourceUnavailable> {
        let state = Gpt2State::new(&model.cfg);
        let (bos_token, eos_token) = (model.cfg.bos, model.cfg.eos);
        Ok(Self {
            model,
            state,
            bos_token,
            eos_token,
        })
    }

    /// This teacher's GPT-2 configuration.
    pub fn cfg(&self) -> &Gpt2Config {
        &self.model.cfg
    }
}

impl crate::RepresentationSource for HuggingFaceGpt2Oracle {
    fn vocab_size(&self) -> usize {
        self.model.cfg.vocab
    }
    fn source_dimension(&self) -> usize {
        self.model.cfg.n_embd
    }
    fn tokenizer_address(&self) -> &str {
        // #601: GPT-2 tokenizes with Hugging Face byte-level BPE (vocab.json +
        // merges.txt / tokenizer.json), so it declares the versioned adapter
        // family that implements exactly that rule — `hf-byte-bpe/1` — rather
        // than the generic `huggingface-tokenizer` the Llama oracle reports.
        // The string is the `(family, version)` rendering of the core registry
        // (`hf_bpe::TokenizerAdapter::HF_BYTE_BPE_FAMILY` / `_VERSION`), spelled
        // literally here because `uor-r4-core` depends on this crate (the
        // constant cannot be imported without a cycle). The pinned tokenizer
        // *content* CID is bound by that core `TokenizerAdapter.tokenizer_cid`
        // (blake3 over the tokenizer bytes) where the pipeline builds it; this
        // family identity is the oracle's #601 surface.
        "hf-byte-bpe/1"
    }
    fn read_embedding_rows(&self, range: std::ops::Range<usize>, output: &mut [f32]) -> Option<()> {
        let d = self.model.cfg.n_embd;
        let count = range.end.checked_sub(range.start)?;
        if output.len() < count * d || range.end > self.model.cfg.vocab {
            return None;
        }
        output[..count * d].copy_from_slice(&self.model.wte[range.start * d..range.end * d]);
        Some(())
    }
}

impl crate::BehaviorSource for HuggingFaceGpt2Oracle {
    fn reset(&mut self) {
        self.state.reset();
    }
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]) {
        self.model
            .forward(&mut self.state, token, pos, &[], &mut |_, _| {});
        logits.copy_from_slice(&self.state.logits);
    }
}

impl crate::TeacherOracle for HuggingFaceGpt2Oracle {
    fn vocab(&self) -> usize {
        self.model.cfg.vocab
    }
    fn dim(&self) -> usize {
        // The compiled geometry this adapter presents (D = 288), not the
        // GPT-2 source width; `embedding` projects rows down to it (#600).
        crate::geometry::COMPILED_WIDTH as usize
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
        self.model.kappa.clone()
    }
    fn source_bytes(&self) -> usize {
        self.model.source_bytes
    }
    fn embedding(&self, token: usize, out: &mut [f32]) {
        let d = self.model.cfg.n_embd;
        let row = &self.model.wte[token * d..(token + 1) * d];
        crate::geometry::bucket_average_project(row, out);
    }
    fn geometry_projection(&self) -> Option<crate::geometry::GeometryProjection> {
        u32::try_from(self.model.cfg.n_embd)
            .ok()
            .map(|source_width| {
                crate::geometry::GeometryProjection::bucket_average(
                    source_width,
                    crate::geometry::COMPILED_WIDTH,
                )
            })
    }
    fn attention_operator_spec(&self) -> Option<crate::attention::AttentionOperatorSpec> {
        // #668: the truthful GPT-2 operator record. `Gpt2Model::layer_forward`
        // (this module) is its implementation — a scaled dot product with the
        // standard max-subtracted softmax, but learned absolute positions (no
        // RoPE on q/k) and GPT-2's fused-`c_attn`/`c_proj` Conv1D projections.
        // Reusing `standard-source-attention/2` would misdeclare the positional
        // action, so this is its own registered `(id, version)`.
        Some(crate::attention::AttentionOperatorSpec::learned_absolute_source_attention())
    }
    fn hidden_state(&self) -> Option<&[f32]> {
        Some(&self.state.hidden)
    }
    fn top_k(&self, k: usize, out: &mut [(u32, f32)]) -> usize {
        crate::top_k_from_logits(&self.state.logits, k, out, false)
    }
    fn trace_capture_geometry(&self) -> Option<crate::TraceCaptureGeometry> {
        Some(crate::TraceCaptureGeometry {
            layers: self.model.cfg.n_layer,
            heads: self.model.cfg.n_head,
            // GPT-2 is plain multi-head: kv heads == query heads.
            kv_heads: self.model.cfg.n_head,
            // The SOURCE residual width the taps expose (n_embd), not the
            // compiled width `dim()` presents. K/v rows are the same width
            // (kv_heads == heads).
            residual_width: self.model.cfg.n_embd,
        })
    }
    fn step_with_trace_capture(
        &mut self,
        token: usize,
        pos: usize,
        logits: &mut [f32],
        request: &crate::TraceCaptureRequest<'_>,
        sinks: &mut crate::TraceCaptureSinks<'_, '_>,
    ) -> bool {
        // #603: capture through the exact executor path — a traced step
        // leaves the same logits as the plain `step`.
        self.model
            .forward_capturing_trace(&mut self.state, token, pos, request, sinks);
        logits.copy_from_slice(&self.state.logits);
        true
    }
}

impl crate::BatchedTeacher for HuggingFaceGpt2Oracle {
    type State = Gpt2State;
    fn new_state(&self) -> Gpt2State {
        Gpt2State::new(&self.model.cfg)
    }
    fn reset_state(&self, state: &mut Gpt2State) {
        state.reset();
    }
    fn logits_mut<'a>(&self, state: &'a mut Gpt2State) -> &'a mut [f32] {
        &mut state.logits
    }
    fn seq_len(&self) -> usize {
        self.model.cfg.seq_len
    }
    fn vocab(&self) -> usize {
        self.model.cfg.vocab
    }
    fn geometry_projection(&self) -> Option<crate::geometry::GeometryProjection> {
        <Self as crate::TeacherOracle>::geometry_projection(self)
    }
    fn attention_operator_spec(&self) -> Option<crate::attention::AttentionOperatorSpec> {
        <Self as crate::TeacherOracle>::attention_operator_spec(self)
    }
    fn forward_batch_into(&self, states: &mut [Gpt2State], tokens: &[usize], positions: &[usize]) {
        self.model.forward_batch(states, tokens, positions);
    }
}

#[cfg(test)]
mod real_tests {
    use super::*;
    use crate::{BehaviorSource, TeacherOracle};
    use std::path::PathBuf;

    /// Presence-gated (#599 three-state) canary over the REAL pinned
    /// openai-community/gpt2 124M snapshot: the 548 MB source is a
    /// dev/local compiler input, never a CI download, so when it is absent
    /// this reports UNAVAILABLE and is a no-op success — never a silent
    /// skip of a real failure. When present, the oracle must reproduce the
    /// independent numpy reference (scripts/gen_gpt2_real_golden.py):
    /// identical argmax and top-5 token set on every prompt, and a final
    /// hidden state within tolerance.
    #[test]
    fn real_gpt2_matches_numpy_reference() {
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src = crate_dir.join("../../.uor-models/sources/gpt2-124m");
        let golden_path = crate_dir.join("tests/fixtures/gpt2-real/golden.json");
        if !src.join("model.safetensors").exists() || !golden_path.exists() {
            eprintln!(
                "UNAVAILABLE: real gpt2 snapshot absent at {} — presence-gated canary skipped",
                src.display()
            );
            return;
        }

        let golden: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&golden_path).unwrap()).unwrap();
        let mut oracle = HuggingFaceGpt2Oracle::load(&src).expect("load real gpt2 oracle");
        let vocab = oracle.vocab();
        let mut logits = vec![0.0f32; vocab];

        for case in golden["cases"].as_array().unwrap() {
            oracle.reset();
            let tokens: Vec<usize> = case["tokens"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as usize)
                .collect();
            for (pos, &tok) in tokens.iter().enumerate() {
                oracle.step(tok, pos, &mut logits);
            }

            // final hidden state within tolerance of the numpy reference.
            let hidden_golden: Vec<f32> = case["hidden"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect();
            let hidden = oracle.hidden_state().expect("hidden state");
            let mut worst = 0.0f32;
            for (&g, &w) in hidden.iter().zip(&hidden_golden) {
                worst = worst.max((g - w).abs());
            }
            eprintln!("tokens {tokens:?}: hidden worst |Δ| = {worst:e}");
            assert!(
                worst < 5e-2,
                "hidden worst |Δ| {worst} too large for {tokens:?}"
            );

            // argmax and top-5 token set must match exactly.
            let mut topk = [(0u32, 0.0f32); 10];
            let n = oracle.top_k(10, &mut topk);
            assert!(n >= 5);
            let argmax = case["argmax"].as_u64().unwrap() as u32;
            assert_eq!(topk[0].0, argmax, "argmax mismatch for {tokens:?}");

            let golden_top5: std::collections::BTreeSet<u32> = case["top_k"]
                .as_array()
                .unwrap()
                .iter()
                .take(5)
                .map(|e| e.as_array().unwrap()[0].as_u64().unwrap() as u32)
                .collect();
            let mine_top5: std::collections::BTreeSet<u32> =
                topk[..5].iter().map(|&(t, _)| t).collect();
            assert_eq!(
                mine_top5, golden_top5,
                "top-5 token set mismatch for {tokens:?}"
            );
        }
    }
}
