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
    /// Immutable once-prepared dense bounds and tied-head transpose. All
    /// mutable proof/intermediate storage belongs to `Gpt2State`, so one
    /// loaded model may be shared across independent callers without a
    /// mutable-model race.
    dense_prepared: Gpt2ProductionDensePrepared,
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
        let dense_prepared = Gpt2ProductionDensePrepared::prepare(&cfg, &wte, &layers)
            .ok_or_else(|| SourceUnavailable::new("GPT-2 dense preparation geometry overflow"))?;
        Ok(Self {
            cfg,
            wte,
            wpe,
            layers,
            ln_f_w,
            ln_f_b,
            kappa: snapshot.kappa().to_owned(),
            source_bytes: snapshot.source_bytes(),
            dense_prepared,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DenseCertificationRejection {
    Nonfinite,
    Zero,
    Overflow,
    Cell,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum DenseCertificationOutcome {
    Certified(f32),
    Rejected(DenseCertificationRejection),
}

#[inline]
fn dense_next_up_f64(value: f64) -> f64 {
    if value.is_nan() || value == f64::INFINITY {
        return value;
    }
    if value == 0.0 {
        return f64::from_bits(1);
    }
    let bits = value.to_bits();
    f64::from_bits(if value > 0.0 { bits + 1 } else { bits - 1 })
}

#[inline]
fn dense_next_down_f64(value: f64) -> f64 {
    if value.is_nan() || value == f64::NEG_INFINITY {
        return value;
    }
    if value == 0.0 {
        return f64::from_bits((1u64 << 63) | 1);
    }
    let bits = value.to_bits();
    f64::from_bits(if value > 0.0 { bits - 1 } else { bits + 1 })
}

#[inline]
fn dense_next_up_f32(value: f32) -> f32 {
    if value.is_nan() || value == f32::INFINITY {
        return value;
    }
    if value == 0.0 {
        return f32::from_bits(1);
    }
    let bits = value.to_bits();
    f32::from_bits(if value > 0.0 { bits + 1 } else { bits - 1 })
}

#[inline]
fn dense_next_down_f32(value: f32) -> f32 {
    if value.is_nan() || value == f32::NEG_INFINITY {
        return value;
    }
    if value == 0.0 {
        return f32::from_bits((1u32 << 31) | 1);
    }
    let bits = value.to_bits();
    f32::from_bits(if value > 0.0 { bits - 1 } else { bits + 1 })
}

/// Outward `gamma_k = ku / (1-ku)` for binary64 round-to-nearest addition.
#[inline]
fn dense_gamma(k: usize) -> Option<f64> {
    if (k as u128) > (1u128 << f64::MANTISSA_DIGITS) {
        return None;
    }
    const UNIT_ROUNDOFF: f64 = 1.0 / ((1u64 << 53) as f64);
    let mu = dense_next_up_f64((k as f64) * UNIT_ROUNDOFF);
    if mu >= 1.0 {
        return None;
    }
    let denominator = dense_next_down_f64(1.0 - mu);
    (denominator > 0.0).then(|| dense_next_up_f64(mu / denominator))
}

/// Convert a rounded nonnegative binary64 sum into an outward upper bound on
/// its exact sum. For `Ahat = fl(sum a_i)`, `|Ahat-A| <= gamma_k A`, hence
/// `A <= Ahat/(1-gamma_k)`.
#[inline]
fn dense_positive_sum_upper(approximate: f64, k: usize) -> Option<f64> {
    if !approximate.is_finite() || approximate < 0.0 {
        return None;
    }
    let gamma = dense_gamma(k)?;
    let denominator = dense_next_down_f64(1.0 - gamma);
    if denominator <= 0.0 {
        return None;
    }
    let upper = dense_next_up_f64(approximate / denominator);
    upper.is_finite().then_some(upper)
}

#[inline]
fn dense_rounding_cell_contains(
    approximate: f64,
    lower: f64,
    upper: f64,
) -> DenseCertificationOutcome {
    if !approximate.is_finite() || !lower.is_finite() || !upper.is_finite() {
        return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Nonfinite);
    }
    let candidate = approximate as f32;
    if candidate == 0.0 {
        return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Zero);
    }
    if !candidate.is_finite() || candidate.abs() == f32::MAX {
        return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Overflow);
    }
    let previous = dense_next_down_f32(candidate);
    let next = dense_next_up_f32(candidate);
    if !previous.is_finite() || !next.is_finite() {
        return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Overflow);
    }
    // Both binary32 values and their midpoint are represented exactly in
    // binary64. Strict containment rejects midpoint ties without a parity arm.
    let cell_lower = (f64::from(previous) + f64::from(candidate)) * 0.5;
    let cell_upper = (f64::from(candidate) + f64::from(next)) * 0.5;
    if lower > cell_lower && upper < cell_upper {
        DenseCertificationOutcome::Certified(candidate)
    } else {
        DenseCertificationOutcome::Rejected(DenseCertificationRejection::Cell)
    }
}

/// Certify a binary64 left fold using an independently supplied outward upper
/// bound on `sum_i |x_i*w_i|`.
#[inline]
fn certify_dense_sum(approximate: f64, sum_abs_upper: f64, k: usize) -> DenseCertificationOutcome {
    if !approximate.is_finite() || !sum_abs_upper.is_finite() {
        return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Nonfinite);
    }
    let Some(gamma) = dense_gamma(k) else {
        return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Cell);
    };
    let error = dense_next_up_f64(gamma * sum_abs_upper);
    if !error.is_finite() {
        return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Cell);
    }
    let lower = dense_next_down_f64(approximate - error);
    let upper = dense_next_up_f64(approximate + error);
    dense_rounding_cell_contains(approximate, lower, upper)
}

/// Knuth TwoSum: absent overflow, `sum + error == left + right` exactly.
#[inline]
fn dense_two_sum(left: f64, right: f64) -> (f64, f64) {
    let sum = left + right;
    let right_virtual = sum - left;
    let left_virtual = sum - right_virtual;
    let right_error = right - right_virtual;
    let left_error = left - left_virtual;
    (sum, left_error + right_error)
}

/// Scalar refinement for a lane rejected by the coarse weight-magnitude
/// bound. TwoSum records every primary-fold error exactly. Only the secondary
/// fold of those errors rounds, so its much smaller residual is bounded with a
/// second gamma enclosure before the same strict binary32 cell test.
fn refine_dense_lane(
    x: &[f32],
    w: &[f32],
    out_dim: usize,
    lane: usize,
) -> DenseCertificationOutcome {
    let mut high = 0.0f64;
    let mut correction = 0.0f64;
    let mut correction_abs = 0.0f64;
    for (input_index, &activation) in x.iter().enumerate() {
        let weight = w[input_index * out_dim + lane];
        if !activation.is_finite() || !weight.is_finite() {
            return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Nonfinite);
        }
        // A finite binary32 product has at most 48 significant bits and is
        // therefore exact in binary64.
        let product = f64::from(activation) * f64::from(weight);
        let (next, error) = dense_two_sum(high, product);
        high = next;
        correction += error;
        correction_abs += error.abs();
    }
    if !high.is_finite() || !correction.is_finite() || !correction_abs.is_finite() {
        return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Nonfinite);
    }
    let Some(correction_abs_upper) = dense_positive_sum_upper(correction_abs, x.len()) else {
        return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Cell);
    };
    let Some(gamma) = dense_gamma(x.len()) else {
        return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Cell);
    };
    let error = dense_next_up_f64(gamma * correction_abs_upper);
    if !error.is_finite() {
        return DenseCertificationOutcome::Rejected(DenseCertificationRejection::Cell);
    }

    // `high + correction` itself is represented as a TwoSum pair. Outward
    // evaluation of that exact pair avoids silently losing its low word.
    let (center_high, center_low) = dense_two_sum(high, correction);
    let approximate = center_high + center_low;
    let tail_lower = dense_next_down_f64(center_low - error);
    let tail_upper = dense_next_up_f64(center_low + error);
    let lower = dense_next_down_f64(center_high + tail_lower);
    let upper = dense_next_up_f64(center_high + tail_upper);
    dense_rounding_cell_contains(approximate, lower, upper)
}

fn exact_dense_lane(x: &[f32], w: &[f32], out_dim: usize, lane: usize) -> f32 {
    if x.is_empty() {
        return 0.0;
    }
    let stride = isize::try_from(out_dim).expect("validated dense row stride fits isize");
    let left = uor_matmul::MatView::row_major(x, 1, x.len())
        .expect("validated dense activation row has its declared extent");
    let right = uor_matmul::MatView::new(
        &w[lane..],
        x.len(),
        1,
        uor_matmul::Strides { rs: stride, cs: 1 },
    )
    .expect("validated immutable dense weight column has its declared extent");
    let mut value = [0.0f32];
    let sink = uor_matmul::MatViewMut::row_major(&mut value, 1, 1)
        .expect("one dense output cell has its declared extent");
    let mut product = uor_matmul::Triple::new(left, right, sink)
        .expect("validated dense row and column form a product");
    uor_matmul::driver::gemm_float(
        &mut product,
        &uor_matmul::Linear::OVERWRITE,
        uor_matmul::GemmOptions::default(),
    );
    value[0]
}

fn exact_dense_projection(out: &mut [f32], x: &[f32], w: &[f32], out_dim: usize) {
    uor_matmul::slice::gemm_float(1, x.len(), out_dim, x, w, out, &mut [], &mut [])
        .expect("validated contiguous dense operands form a product");
}

fn certified_dense_dots_with_scratch(
    out: &mut [f32],
    x: &[f32],
    w: &[f32],
    out_dim: usize,
    sums: &mut [f64],
    sum_abs_weight_upper: &[f64],
) -> Gpt2DenseCanaryCensus {
    sums.fill(0.0);
    let mut max_activation_abs = 0.0f64;
    for (input_index, &activation) in x.iter().enumerate() {
        let activation = f64::from(activation);
        max_activation_abs = max_activation_abs.max(activation.abs());
        let row = &w[input_index * out_dim..(input_index + 1) * out_dim];
        for output_index in 0..out_dim {
            sums[output_index] += activation * f64::from(row[output_index]);
        }
    }

    let mut census = Gpt2DenseCanaryCensus {
        lanes: out_dim,
        ..Gpt2DenseCanaryCensus::default()
    };
    for lane in 0..out_dim {
        let sum_abs_upper = dense_next_up_f64(max_activation_abs * sum_abs_weight_upper[lane]);
        let dot = match certify_dense_sum(sums[lane], sum_abs_upper, x.len()) {
            DenseCertificationOutcome::Certified(value) => {
                census.fast_certified += 1;
                value
            }
            DenseCertificationOutcome::Rejected(DenseCertificationRejection::Nonfinite) => {
                census.reject(DenseCertificationRejection::Nonfinite);
                exact_dense_lane(x, w, out_dim, lane)
            }
            DenseCertificationOutcome::Rejected(_) => {
                match refine_dense_lane(x, w, out_dim, lane) {
                    DenseCertificationOutcome::Certified(value) => {
                        census.refined_certified += 1;
                        value
                    }
                    DenseCertificationOutcome::Rejected(reason) => {
                        census.reject(reason);
                        exact_dense_lane(x, w, out_dim, lane)
                    }
                }
            }
        };
        out[lane] = dot;
    }
    debug_assert_eq!(
        census.fast_certified + census.refined_certified + census.fallbacks().unwrap_or(usize::MAX),
        census.lanes
    );
    census
}

fn certified_dense_projection(
    out: &mut [f32],
    x: &[f32],
    w: &[f32],
    bias: &[f32],
    workspace: &mut Gpt2DenseCanaryWorkspace,
) -> Gpt2DenseCanaryCensus {
    let census = certified_dense_dots_with_scratch(
        out,
        x,
        w,
        workspace.out_dim,
        &mut workspace.sums,
        &workspace.sum_abs_weight_upper,
    );
    // #704's Conv1D semantics are one correctly-rounded dot followed by the
    // historical single binary32 bias addition.
    for (value, &addend) in out.iter_mut().zip(bias) {
        *value += addend;
    }
    census
}

fn conventional_dense_dots(out: &mut [f32], x: &[f32], w: &[f32], out_dim: usize) {
    out[..out_dim].fill(0.0);
    for (input_index, &activation) in x.iter().enumerate() {
        if activation == 0.0 {
            continue;
        }
        let row = &w[input_index * out_dim..(input_index + 1) * out_dim];
        for output_index in 0..out_dim {
            out[output_index] += activation * row[output_index];
        }
    }
}

fn conventional_lm_head(out: &mut [f32], hidden: &[f32], wte: &[f32], width: usize) {
    for (vocabulary_index, output) in out.iter_mut().enumerate() {
        let row = &wte[vocabulary_index * width..(vocabulary_index + 1) * width];
        let mut accumulator = 0.0f32;
        for input_index in 0..width {
            accumulator += hidden[input_index] * row[input_index];
        }
        *output = accumulator;
    }
}

fn controlled_dense_projection(
    out: &mut [f32],
    x: &[f32],
    weights: &[f32],
    bias: Option<&[f32]>,
    prepared: &mut DensePreparedMatrix,
    mode: Gpt2DenseCanaryMode,
) -> Gpt2DenseCanaryCensus {
    let out_dim = prepared.out_dim;
    let mut census = Gpt2DenseCanaryCensus {
        lanes: out_dim,
        ..Gpt2DenseCanaryCensus::default()
    };
    match mode {
        Gpt2DenseCanaryMode::Conventional => {
            if let Some(bias) = bias {
                conv1d(x, weights, bias, out_dim, out);
            } else {
                conventional_dense_dots(out, x, weights, out_dim);
            }
            census.conventional = out_dim;
        }
        Gpt2DenseCanaryMode::Exact => {
            exact_dense_projection(out, x, weights, out_dim);
            if let Some(bias) = bias {
                for (value, &addend) in out.iter_mut().zip(bias) {
                    *value += addend;
                }
            }
            census.exact_control = out_dim;
        }
        Gpt2DenseCanaryMode::CertifiedNative => {
            census = certified_dense_dots_with_scratch(
                out,
                x,
                weights,
                out_dim,
                &mut prepared.sums,
                &prepared.sum_abs_weight_upper,
            );
            if let Some(bias) = bias {
                for (value, &addend) in out.iter_mut().zip(bias) {
                    *value += addend;
                }
            }
        }
    }
    census
}

#[allow(clippy::too_many_arguments)]
fn certified_dense_dots_batched_with_scratch(
    out: &mut [f32],
    x: &[f32],
    weights: &[f32],
    batch: usize,
    in_dim: usize,
    out_dim: usize,
    sums: &mut [f64],
    max_activation_abs: &mut [f64],
    sum_abs_weight_upper: &[f64],
) -> Gpt2DenseCanaryCensus {
    sums[..batch * out_dim].fill(0.0);
    max_activation_abs[..batch].fill(0.0);
    for input_index in 0..in_dim {
        let row = &weights[input_index * out_dim..(input_index + 1) * out_dim];
        for batch_index in 0..batch {
            let activation = f64::from(x[batch_index * in_dim + input_index]);
            max_activation_abs[batch_index] = max_activation_abs[batch_index].max(activation.abs());
            let output = &mut sums[batch_index * out_dim..(batch_index + 1) * out_dim];
            for output_index in 0..out_dim {
                output[output_index] += activation * f64::from(row[output_index]);
            }
        }
    }

    let mut census = Gpt2DenseCanaryCensus {
        lanes: batch * out_dim,
        ..Gpt2DenseCanaryCensus::default()
    };
    for batch_index in 0..batch {
        let input = &x[batch_index * in_dim..(batch_index + 1) * in_dim];
        for (lane, &weight_bound) in sum_abs_weight_upper.iter().enumerate().take(out_dim) {
            let index = batch_index * out_dim + lane;
            let sum_abs_upper = dense_next_up_f64(max_activation_abs[batch_index] * weight_bound);
            let dot = match certify_dense_sum(sums[index], sum_abs_upper, in_dim) {
                DenseCertificationOutcome::Certified(value) => {
                    census.fast_certified += 1;
                    value
                }
                DenseCertificationOutcome::Rejected(DenseCertificationRejection::Nonfinite) => {
                    census.reject(DenseCertificationRejection::Nonfinite);
                    exact_dense_lane(input, weights, out_dim, lane)
                }
                DenseCertificationOutcome::Rejected(_) => {
                    match refine_dense_lane(input, weights, out_dim, lane) {
                        DenseCertificationOutcome::Certified(value) => {
                            census.refined_certified += 1;
                            value
                        }
                        DenseCertificationOutcome::Rejected(reason) => {
                            census.reject(reason);
                            exact_dense_lane(input, weights, out_dim, lane)
                        }
                    }
                }
            };
            out[index] = dot;
        }
    }
    debug_assert_eq!(
        census.fast_certified + census.refined_certified + census.fallbacks().unwrap_or(usize::MAX),
        census.lanes
    );
    census
}

#[allow(clippy::too_many_arguments)]
fn controlled_dense_projection_batched(
    out: &mut [f32],
    x: &[f32],
    weights: &[f32],
    bias: Option<&[f32]>,
    batch: usize,
    prepared: &DensePreparedMatrix,
    sums: &mut [f64],
    max_activation_abs: &mut [f64],
    mode: Gpt2DenseCanaryMode,
) -> Gpt2DenseCanaryCensus {
    let in_dim = prepared.in_dim;
    let out_dim = prepared.out_dim;
    let lanes = batch * out_dim;
    let mut census = Gpt2DenseCanaryCensus {
        lanes,
        batch_rows: batch,
        ..Gpt2DenseCanaryCensus::default()
    };
    match mode {
        Gpt2DenseCanaryMode::Conventional => {
            if let Some(bias) = bias {
                conv1d_batched(out, x, weights, bias, out_dim, in_dim, batch);
            } else {
                out[..lanes].fill(0.0);
                for input_index in 0..in_dim {
                    let row = &weights[input_index * out_dim..(input_index + 1) * out_dim];
                    for batch_index in 0..batch {
                        let activation = x[batch_index * in_dim + input_index];
                        if activation == 0.0 {
                            continue;
                        }
                        let output = &mut out[batch_index * out_dim..(batch_index + 1) * out_dim];
                        for output_index in 0..out_dim {
                            output[output_index] += activation * row[output_index];
                        }
                    }
                }
            }
            census.conventional = lanes;
        }
        Gpt2DenseCanaryMode::Exact => {
            for batch_index in 0..batch {
                let input = &x[batch_index * in_dim..(batch_index + 1) * in_dim];
                for lane in 0..out_dim {
                    out[batch_index * out_dim + lane] =
                        exact_dense_lane(input, weights, out_dim, lane);
                }
            }
            if let Some(bias) = bias {
                for output in out[..lanes].chunks_exact_mut(out_dim) {
                    for (value, &addend) in output.iter_mut().zip(bias) {
                        *value += addend;
                    }
                }
            }
            census.exact_control = lanes;
        }
        Gpt2DenseCanaryMode::CertifiedNative => {
            census = certified_dense_dots_batched_with_scratch(
                out,
                x,
                weights,
                batch,
                in_dim,
                out_dim,
                sums,
                max_activation_abs,
                &prepared.sum_abs_weight_upper,
            );
            census.batch_rows = batch;
            if let Some(bias) = bias {
                for output in out[..lanes].chunks_exact_mut(out_dim) {
                    for (value, &addend) in output.iter_mut().zip(bias) {
                        *value += addend;
                    }
                }
            }
        }
    }
    census
}

/// Production-shaped certified projection over caller-owned state lanes.
/// Each immutable weight row is fetched once, then reused across the batch;
/// within every lane the f64 accumulation still visits input indices in the
/// same ascending order as the serial one-row call.
fn production_dense_projection_batched(
    states: &mut [Gpt2State],
    weights: &[f32],
    bias: Option<&[f32]>,
    metadata: &DenseWeightMetadata,
) {
    let in_dim = metadata.in_dim;
    let out_dim = metadata.out_dim;
    debug_assert_eq!(weights.len(), in_dim * out_dim);
    debug_assert_eq!(metadata.sum_abs_weight_upper.len(), out_dim);
    debug_assert!(bias.is_none_or(|bias| bias.len() >= out_dim));

    for state in states.iter_mut() {
        debug_assert!(state.dense_scratch.input.len() >= in_dim);
        debug_assert!(state.dense_scratch.output.len() >= out_dim);
        debug_assert!(state.dense_scratch.sums.len() >= out_dim);
        state.dense_scratch.sums[..out_dim].fill(0.0);
        state.dense_scratch.max_activation_abs = 0.0;
    }

    for input_index in 0..in_dim {
        let row = &weights[input_index * out_dim..(input_index + 1) * out_dim];
        for state in states.iter_mut() {
            let activation = f64::from(state.dense_scratch.input[input_index]);
            state.dense_scratch.max_activation_abs =
                state.dense_scratch.max_activation_abs.max(activation.abs());
            let sums = &mut state.dense_scratch.sums[..out_dim];
            for output_index in 0..out_dim {
                sums[output_index] += activation * f64::from(row[output_index]);
            }
        }
    }

    for state in states {
        let Gpt2ProductionDenseScratch {
            input,
            output,
            sums,
            max_activation_abs,
            ..
        } = &mut state.dense_scratch;
        let input = &input[..in_dim];
        for lane in 0..out_dim {
            let sum_abs_upper =
                dense_next_up_f64(*max_activation_abs * metadata.sum_abs_weight_upper[lane]);
            let dot = match certify_dense_sum(sums[lane], sum_abs_upper, in_dim) {
                DenseCertificationOutcome::Certified(value) => value,
                DenseCertificationOutcome::Rejected(DenseCertificationRejection::Nonfinite) => {
                    exact_dense_lane(input, weights, out_dim, lane)
                }
                DenseCertificationOutcome::Rejected(_) => {
                    match refine_dense_lane(input, weights, out_dim, lane) {
                        DenseCertificationOutcome::Certified(value) => value,
                        DenseCertificationOutcome::Rejected(_) => {
                            exact_dense_lane(input, weights, out_dim, lane)
                        }
                    }
                }
            };
            output[lane] = bias.map_or(dot, |bias| dot + bias[lane]);
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
#[derive(Clone, Debug)]
pub struct Gpt2State {
    k_cache: Vec<f32>,
    v_cache: Vec<f32>,
    /// Logits of the last [`Gpt2::forward`] step (`vocab`).
    pub logits: Vec<f32>,
    /// Final hidden state (post `ln_f`) of the last step (`n_embd`).
    pub hidden: Vec<f32>,
    x: Vec<f32>,
    dense_scratch: Gpt2ProductionDenseScratch,
}

// `Gpt2State` exposed logical equality before production dense scratch became
// state-local. Preserve that public contract: opaque workspace residue is not
// recurrent model state and must not make two otherwise identical states
// unequal.
impl PartialEq for Gpt2State {
    fn eq(&self, other: &Self) -> bool {
        self.k_cache == other.k_cache
            && self.v_cache == other.v_cache
            && self.logits == other.logits
            && self.hidden == other.hidden
            && self.x == other.x
    }
}

/// Per-state production scratch. Keeping it with the recurrent state makes
/// the shared model immutable and gives an arbitrary-size batch one scratch
/// lane per caller-owned sequence without a shared capacity or lock.
#[derive(Clone, Debug)]
struct Gpt2ProductionDenseScratch {
    input: Vec<f32>,
    output: Vec<f32>,
    sums: Vec<f64>,
    max_activation_abs: f64,
    attention: Vec<f32>,
    scores: Vec<f32>,
}

impl Gpt2ProductionDenseScratch {
    fn new(cfg: &Gpt2Config) -> Self {
        let maximum_input = cfg.n_inner.max(cfg.n_embd);
        let maximum_output = cfg
            .vocab
            .max(cfg.n_inner)
            .max(3usize.saturating_mul(cfg.n_embd));
        Self {
            input: vec![0.0; maximum_input],
            output: vec![0.0; maximum_output],
            sums: vec![0.0; maximum_output],
            max_activation_abs: 0.0,
            attention: vec![0.0; cfg.n_embd],
            scores: vec![0.0; cfg.seq_len],
        }
    }

    fn reset(&mut self) {
        self.input.fill(0.0);
        self.output.fill(0.0);
        self.sums.fill(0.0);
        self.max_activation_abs = 0.0;
        self.attention.fill(0.0);
        self.scores.fill(0.0);
    }
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

/// Arithmetic arm selected by the checked GPT-2 dense differential controls.
/// Production executes the certified-native v2 arm; conventional and exact
/// remain explicit comparison owners.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpt2DenseCanaryMode {
    /// Historical bias-seeded sequential binary32 Conv1D.
    Conventional,
    /// Pinned correctly-rounded exact dot, followed by one binary32 bias add.
    Exact,
    /// Native binary64 candidate with a proof/refinement/exact-fallback chain.
    CertifiedNative,
}

/// Per-output verdict census for the checked #704 dense canary.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gpt2DenseCanaryCensus {
    lanes: usize,
    batch_rows: usize,
    conventional: usize,
    exact_control: usize,
    fast_certified: usize,
    refined_certified: usize,
    fallback_nonfinite: usize,
    fallback_zero: usize,
    fallback_overflow: usize,
    fallback_cell: usize,
}

impl Gpt2DenseCanaryCensus {
    pub const fn lanes(self) -> usize {
        self.lanes
    }

    /// Sequence rows processed by the production-shaped row-reuse batch
    /// kernel. Serial controls report zero.
    pub const fn batch_rows(self) -> usize {
        self.batch_rows
    }

    pub const fn conventional(self) -> usize {
        self.conventional
    }

    pub const fn exact_control(self) -> usize {
        self.exact_control
    }

    pub const fn fast_certified(self) -> usize {
        self.fast_certified
    }

    pub const fn refined_certified(self) -> usize {
        self.refined_certified
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

    pub fn fallbacks(self) -> Option<usize> {
        self.fallback_nonfinite
            .checked_add(self.fallback_zero)?
            .checked_add(self.fallback_overflow)?
            .checked_add(self.fallback_cell)
    }

    /// Merge a projection census atomically. `None` leaves `self` unchanged.
    pub fn merge(&mut self, other: Self) -> Option<()> {
        let merged = Self {
            lanes: self.lanes.checked_add(other.lanes)?,
            batch_rows: self.batch_rows.checked_add(other.batch_rows)?,
            conventional: self.conventional.checked_add(other.conventional)?,
            exact_control: self.exact_control.checked_add(other.exact_control)?,
            fast_certified: self.fast_certified.checked_add(other.fast_certified)?,
            refined_certified: self
                .refined_certified
                .checked_add(other.refined_certified)?,
            fallback_nonfinite: self
                .fallback_nonfinite
                .checked_add(other.fallback_nonfinite)?,
            fallback_zero: self.fallback_zero.checked_add(other.fallback_zero)?,
            fallback_overflow: self
                .fallback_overflow
                .checked_add(other.fallback_overflow)?,
            fallback_cell: self.fallback_cell.checked_add(other.fallback_cell)?,
        };
        *self = merged;
        Some(())
    }

    fn reject(&mut self, reason: DenseCertificationRejection) {
        match reason {
            DenseCertificationRejection::Nonfinite => self.fallback_nonfinite += 1,
            DenseCertificationRejection::Zero => self.fallback_zero += 1,
            DenseCertificationRejection::Overflow => self.fallback_overflow += 1,
            DenseCertificationRejection::Cell => self.fallback_cell += 1,
        }
    }
}

/// Dense owner selected by the checked matrix-level and whole-model controls.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpt2DenseControlSite {
    CAttn,
    AttentionCProj,
    MlpCFc,
    MlpCProj,
    LmHead,
}

/// Four projection censuses for one GPT-2 transformer layer.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gpt2DenseLayerCensus {
    c_attn: Gpt2DenseCanaryCensus,
    attention_c_proj: Gpt2DenseCanaryCensus,
    mlp_c_fc: Gpt2DenseCanaryCensus,
    mlp_c_proj: Gpt2DenseCanaryCensus,
}

impl Gpt2DenseLayerCensus {
    pub const fn c_attn(self) -> Gpt2DenseCanaryCensus {
        self.c_attn
    }

    pub const fn attention_c_proj(self) -> Gpt2DenseCanaryCensus {
        self.attention_c_proj
    }

    pub const fn mlp_c_fc(self) -> Gpt2DenseCanaryCensus {
        self.mlp_c_fc
    }

    pub const fn mlp_c_proj(self) -> Gpt2DenseCanaryCensus {
        self.mlp_c_proj
    }

    pub fn merge(&mut self, other: Self) -> Option<()> {
        let mut c_attn = self.c_attn;
        let mut attention_c_proj = self.attention_c_proj;
        let mut mlp_c_fc = self.mlp_c_fc;
        let mut mlp_c_proj = self.mlp_c_proj;
        c_attn.merge(other.c_attn)?;
        attention_c_proj.merge(other.attention_c_proj)?;
        mlp_c_fc.merge(other.mlp_c_fc)?;
        mlp_c_proj.merge(other.mlp_c_proj)?;
        *self = Self {
            c_attn,
            attention_c_proj,
            mlp_c_fc,
            mlp_c_proj,
        };
        Some(())
    }
}

/// Borrowed, allocation-free census from one checked serial, trace, or batch
/// control call. The slice has exactly one row per model layer.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct Gpt2DenseControlCensus<'a> {
    layers: &'a [Gpt2DenseLayerCensus],
    lm_head: Gpt2DenseCanaryCensus,
}

impl<'a> Gpt2DenseControlCensus<'a> {
    pub const fn layers(self) -> &'a [Gpt2DenseLayerCensus] {
        self.layers
    }

    pub const fn lm_head(self) -> Gpt2DenseCanaryCensus {
        self.lm_head
    }
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
            dense_scratch: Gpt2ProductionDenseScratch::new(cfg),
        }
    }

    /// Begin a new sequence: zero the caches and working buffers.
    pub fn reset(&mut self) {
        self.k_cache.fill(0.0);
        self.v_cache.fill(0.0);
        self.logits.fill(0.0);
        self.hidden.fill(0.0);
        self.x.fill(0.0);
        self.dense_scratch.reset();
    }

    /// Bitwise comparison for the evidence-only dense controls, including
    /// recurrent caches and the private residual buffer.
    #[doc(hidden)]
    pub fn dense_control_bit_identical(&self, other: &Self) -> bool {
        let bits_equal = |left: &[f32], right: &[f32]| {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| left.to_bits() == right.to_bits())
        };
        bits_equal(&self.k_cache, &other.k_cache)
            && bits_equal(&self.v_cache, &other.v_cache)
            && bits_equal(&self.logits, &other.logits)
            && bits_equal(&self.hidden, &other.hidden)
            && bits_equal(&self.x, &other.x)
    }

    /// Test-only comparison of every logical and scratch bit. Public
    /// [`PartialEq`] deliberately excludes the opaque production workspace,
    /// while failure-atomicity tests must still prove that rejected calls do
    /// not alter it.
    #[cfg(test)]
    fn full_storage_bit_identical(&self, other: &Self) -> bool {
        let f32_bits_equal = |left: &[f32], right: &[f32]| {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| left.to_bits() == right.to_bits())
        };
        let f64_bits_equal = |left: &[f64], right: &[f64]| {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| left.to_bits() == right.to_bits())
        };
        self.dense_control_bit_identical(other)
            && f32_bits_equal(&self.dense_scratch.input, &other.dense_scratch.input)
            && f32_bits_equal(&self.dense_scratch.output, &other.dense_scratch.output)
            && f64_bits_equal(&self.dense_scratch.sums, &other.dense_scratch.sums)
            && self.dense_scratch.max_activation_abs.to_bits()
                == other.dense_scratch.max_activation_abs.to_bits()
            && f32_bits_equal(
                &self.dense_scratch.attention,
                &other.dense_scratch.attention,
            )
            && f32_bits_equal(&self.dense_scratch.scores, &other.dense_scratch.scores)
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

/// Opaque caller-owned scratch for #704's layer-0 fused-QKV experiment.
/// Construction scans the immutable weight once and is outside the measured
/// hot path. A checked call then reuses the single binary64 sum vector without
/// allocation or weight rewriting.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub struct Gpt2DenseCanaryWorkspace {
    weight_address: usize,
    weight_len: usize,
    source_kappa: String,
    in_dim: usize,
    out_dim: usize,
    sums: Vec<f64>,
    sum_abs_weight_upper: Vec<f64>,
}

const DENSE_CONTROL_SITES_PER_LAYER: usize = 4;

/// Immutable proof metadata for one production `[in,out]` dense weight.
#[derive(Debug)]
struct DenseWeightMetadata {
    in_dim: usize,
    out_dim: usize,
    sum_abs_weight_upper: Vec<f64>,
}

impl DenseWeightMetadata {
    fn prepare(weights: &[f32], in_dim: usize, out_dim: usize) -> Option<Self> {
        if weights.len() != in_dim.checked_mul(out_dim)? {
            return None;
        }
        let mut sum_abs_weight_upper = vec![0.0f64; out_dim];
        for input_index in 0..in_dim {
            let row = &weights[input_index * out_dim..(input_index + 1) * out_dim];
            for output_index in 0..out_dim {
                sum_abs_weight_upper[output_index] += f64::from(row[output_index]).abs();
            }
        }
        for bound in &mut sum_abs_weight_upper {
            *bound = dense_positive_sum_upper(*bound, in_dim).unwrap_or(f64::INFINITY);
        }
        Some(Self {
            in_dim,
            out_dim,
            sum_abs_weight_upper,
        })
    }
}

/// Immutable production preparation for all 48 Conv1Ds and the tied head.
#[derive(Debug)]
struct Gpt2ProductionDensePrepared {
    geometry: Gpt2ProductionGeometry,
    matrices: Vec<DenseWeightMetadata>,
    lm_head_transposed: Vec<f32>,
    lm_head: DenseWeightMetadata,
}

/// Load-time geometry bound to immutable weights and dense preparation.
/// `Gpt2::cfg` predates the prepared executor and remains public for API
/// compatibility, so every production call compares it with this snapshot
/// before touching caller state.
#[derive(Clone, Copy, Debug)]
struct Gpt2ProductionGeometry {
    n_embd: usize,
    n_head: usize,
    n_layer: usize,
    n_positions: usize,
    n_inner: usize,
    vocab: usize,
    layer_norm_eps_bits: u32,
}

impl Gpt2ProductionGeometry {
    fn from_config(cfg: &Gpt2Config) -> Self {
        Self {
            n_embd: cfg.n_embd,
            n_head: cfg.n_head,
            n_layer: cfg.n_layer,
            n_positions: cfg.n_positions,
            n_inner: cfg.n_inner,
            vocab: cfg.vocab,
            layer_norm_eps_bits: cfg.layer_norm_eps.to_bits(),
        }
    }

    fn matches_config(self, cfg: &Gpt2Config) -> bool {
        self.n_embd == cfg.n_embd
            && self.n_head == cfg.n_head
            && self.n_layer == cfg.n_layer
            && self.n_positions == cfg.n_positions
            && self.n_inner == cfg.n_inner
            && self.vocab == cfg.vocab
            && self.layer_norm_eps_bits == cfg.layer_norm_eps.to_bits()
    }
}

impl Gpt2ProductionDensePrepared {
    fn prepare(cfg: &Gpt2Config, wte: &[f32], layers: &[Gpt2Layer]) -> Option<Self> {
        let d = cfg.n_embd;
        let expected_layers = cfg.n_layer;
        if layers.len() != expected_layers || wte.len() != cfg.vocab.checked_mul(d)? {
            return None;
        }
        let mut matrices =
            Vec::with_capacity(expected_layers.checked_mul(DENSE_CONTROL_SITES_PER_LAYER)?);
        for layer in layers {
            matrices.push(DenseWeightMetadata::prepare(
                &layer.c_attn_w,
                d,
                3usize.checked_mul(d)?,
            )?);
            matrices.push(DenseWeightMetadata::prepare(&layer.c_proj_w, d, d)?);
            matrices.push(DenseWeightMetadata::prepare(&layer.fc_w, d, cfg.n_inner)?);
            matrices.push(DenseWeightMetadata::prepare(&layer.mlp_w, cfg.n_inner, d)?);
        }

        let mut lm_head_transposed = vec![0.0f32; d.checked_mul(cfg.vocab)?];
        for vocabulary_index in 0..cfg.vocab {
            let row = &wte[vocabulary_index * d..(vocabulary_index + 1) * d];
            for input_index in 0..d {
                lm_head_transposed[input_index * cfg.vocab + vocabulary_index] = row[input_index];
            }
        }
        let lm_head = DenseWeightMetadata::prepare(&lm_head_transposed, d, cfg.vocab)?;
        Some(Self {
            geometry: Gpt2ProductionGeometry::from_config(cfg),
            matrices,
            lm_head_transposed,
            lm_head,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct DensePreparedMatrix {
    in_dim: usize,
    out_dim: usize,
    sums: Vec<f64>,
    sum_abs_weight_upper: Vec<f64>,
}

impl DensePreparedMatrix {
    fn prepare(weights: &[f32], in_dim: usize, out_dim: usize) -> Option<Self> {
        if weights.len() != in_dim.checked_mul(out_dim)? {
            return None;
        }
        let mut sum_abs_weight_upper = vec![0.0f64; out_dim];
        for input_index in 0..in_dim {
            let row = &weights[input_index * out_dim..(input_index + 1) * out_dim];
            for output_index in 0..out_dim {
                sum_abs_weight_upper[output_index] += f64::from(row[output_index]).abs();
            }
        }
        for bound in &mut sum_abs_weight_upper {
            *bound = dense_positive_sum_upper(*bound, in_dim).unwrap_or(f64::INFINITY);
        }
        Some(Self {
            in_dim,
            out_dim,
            sums: vec![0.0; out_dim],
            sum_abs_weight_upper,
        })
    }

    fn has_shape(&self, in_dim: usize, out_dim: usize) -> bool {
        self.in_dim == in_dim
            && self.out_dim == out_dim
            && self.sums.len() == out_dim
            && self.sum_abs_weight_upper.len() == out_dim
    }
}

/// Caller-owned, model-bound scratch for the evidence-only whole-GPT-2 dense
/// control. Every projection bound and the tied-lm-head transpose are prepared
/// once at construction; serial, trace, and batch hot calls allocate nothing.
#[doc(hidden)]
#[derive(Debug)]
pub struct Gpt2DenseControlWorkspace {
    source_kappa: String,
    max_batch: usize,
    model_weight_addresses: Vec<usize>,
    model_weight_lengths: Vec<usize>,
    wte_address: usize,
    wte_len: usize,
    prepared: Vec<DensePreparedMatrix>,
    lm_head_transposed: Vec<f32>,
    lm_head: DensePreparedMatrix,
    normed: Vec<f32>,
    qkv: Vec<f32>,
    attn: Vec<f32>,
    proj: Vec<f32>,
    inner: Vec<f32>,
    mlp_out: Vec<f32>,
    scores: Vec<f32>,
    batch_normed: Vec<f32>,
    batch_qkv: Vec<f32>,
    batch_attn: Vec<f32>,
    batch_proj: Vec<f32>,
    batch_inner: Vec<f32>,
    batch_mlp_out: Vec<f32>,
    batch_logits: Vec<f32>,
    batch_sums: Vec<f64>,
    batch_max_activation_abs: Vec<f64>,
    step_layers: Vec<Gpt2DenseLayerCensus>,
    step_lm_head: Gpt2DenseCanaryCensus,
    batch_layers: Vec<Gpt2DenseLayerCensus>,
    batch_lm_head: Gpt2DenseCanaryCensus,
}

/// Logical owned-capacity accounting for a prepared whole-model dense
/// workspace. Allocator bookkeeping and the caller's recurrent state are not
/// included; the reported total is the workspace value plus every heap byte
/// its vectors and string have reserved.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gpt2DenseControlWorkspaceBytes {
    lm_head_transpose: usize,
    matrix_bounds: usize,
    f64_sum_scratch: usize,
    intermediate_scratch: usize,
    metadata: usize,
    total: usize,
}

impl Gpt2DenseControlWorkspaceBytes {
    pub const fn lm_head_transpose(self) -> usize {
        self.lm_head_transpose
    }

    pub const fn matrix_bounds(self) -> usize {
        self.matrix_bounds
    }

    pub const fn f64_sum_scratch(self) -> usize {
        self.f64_sum_scratch
    }

    pub const fn intermediate_scratch(self) -> usize {
        self.intermediate_scratch
    }

    pub const fn metadata(self) -> usize {
        self.metadata
    }

    pub const fn total(self) -> usize {
        self.total
    }
}

/// Deterministic digest over every logical workspace field and vector
/// capacity. This permits failure-atomicity checks without cloning the
/// 154-MB-class tied-head transpose.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gpt2DenseControlWorkspaceFingerprint([u8; 32]);

impl Gpt2DenseControlWorkspaceFingerprint {
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

fn dense_fingerprint_usize(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&value.to_le_bytes());
}

fn dense_fingerprint_f32_slice(hasher: &mut blake3::Hasher, values: &[f32], capacity: usize) {
    dense_fingerprint_usize(hasher, values.len());
    dense_fingerprint_usize(hasher, capacity);
    for value in values {
        hasher.update(&value.to_bits().to_le_bytes());
    }
}

fn dense_fingerprint_f64_slice(hasher: &mut blake3::Hasher, values: &[f64], capacity: usize) {
    dense_fingerprint_usize(hasher, values.len());
    dense_fingerprint_usize(hasher, capacity);
    for value in values {
        hasher.update(&value.to_bits().to_le_bytes());
    }
}

fn dense_fingerprint_census(hasher: &mut blake3::Hasher, census: Gpt2DenseCanaryCensus) {
    for value in [
        census.lanes,
        census.batch_rows,
        census.conventional,
        census.exact_control,
        census.fast_certified,
        census.refined_certified,
        census.fallback_nonfinite,
        census.fallback_zero,
        census.fallback_overflow,
        census.fallback_cell,
    ] {
        dense_fingerprint_usize(hasher, value);
    }
}

fn dense_fingerprint_layer_census(hasher: &mut blake3::Hasher, layer: Gpt2DenseLayerCensus) {
    dense_fingerprint_census(hasher, layer.c_attn);
    dense_fingerprint_census(hasher, layer.attention_c_proj);
    dense_fingerprint_census(hasher, layer.mlp_c_fc);
    dense_fingerprint_census(hasher, layer.mlp_c_proj);
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

    #[allow(clippy::too_many_arguments)]
    fn block_attention_production(
        &self,
        key_cache: &mut [f32],
        value_cache: &mut [f32],
        layer_index: usize,
        position: usize,
        qkv: &[f32],
        attention: &mut [f32],
        scores: &mut [f32],
        request: Option<&crate::TraceCaptureRequest<'_>>,
        mut sinks: Option<&mut crate::TraceCaptureSinks<'_, '_>>,
    ) {
        let d = self.cfg.n_embd;
        let head_size = self.cfg.head_size();
        let sequence_length = self.cfg.seq_len;
        let base = (layer_index * sequence_length + position) * d;
        key_cache[base..base + d].copy_from_slice(&qkv[d..2 * d]);
        value_cache[base..base + d].copy_from_slice(&qkv[2 * d..3 * d]);

        if request.is_some_and(|request| request.qkv_layers.contains(&layer_index)) {
            if let Some(sinks) = sinks.as_deref_mut() {
                (sinks.qkv)(layer_index, &qkv[..d], &qkv[d..2 * d], &qkv[2 * d..3 * d]);
            }
        }

        debug_assert_eq!(scores.len(), position + 1);
        let layer_keys =
            &key_cache[layer_index * sequence_length * d..(layer_index + 1) * sequence_length * d];
        let layer_values = &value_cache
            [layer_index * sequence_length * d..(layer_index + 1) * sequence_length * d];
        for head in 0..self.cfg.n_head {
            let query = &qkv[head * head_size..(head + 1) * head_size];
            let _ = attention_weights_with_arithmetic(
                scores,
                query,
                layer_keys,
                head * head_size,
                d,
                crate::attention::AttentionArithmetic::CertifiedNative,
            );
            if request.is_some_and(|request| request.attention_layers.contains(&layer_index)) {
                if let Some(sinks) = sinks.as_deref_mut() {
                    (sinks.attention)(layer_index, head, scores);
                }
            }
            let output = &mut attention[head * head_size..(head + 1) * head_size];
            let _ = crate::attention::head_attention_value_aggregate_with_arithmetic(
                output,
                scores,
                layer_values,
                head * head_size,
                d,
                crate::attention::AttentionArithmetic::CertifiedNative,
            );
        }
    }

    fn block_forward_production(
        &self,
        state: &mut Gpt2State,
        layer_index: usize,
        position: usize,
        request: Option<&crate::TraceCaptureRequest<'_>>,
        sinks: Option<&mut crate::TraceCaptureSinks<'_, '_>>,
    ) {
        let d = self.cfg.n_embd;
        let inner_dim = self.cfg.n_inner;
        let layer = &self.layers[layer_index];
        let base = layer_index * DENSE_CONTROL_SITES_PER_LAYER;

        layer_norm(
            &state.x,
            &layer.ln1_w,
            &layer.ln1_b,
            self.cfg.layer_norm_eps,
            &mut state.dense_scratch.input[..d],
        );
        production_dense_projection_batched(
            std::slice::from_mut(state),
            &layer.c_attn_w,
            Some(&layer.c_attn_b),
            &self.dense_prepared.matrices[base],
        );
        {
            let Gpt2State {
                k_cache,
                v_cache,
                dense_scratch,
                ..
            } = state;
            self.block_attention_production(
                k_cache,
                v_cache,
                layer_index,
                position,
                &dense_scratch.output[..3 * d],
                &mut dense_scratch.attention[..d],
                &mut dense_scratch.scores[..=position],
                request,
                sinks,
            );
        }

        state.dense_scratch.input[..d].copy_from_slice(&state.dense_scratch.attention[..d]);
        production_dense_projection_batched(
            std::slice::from_mut(state),
            &layer.c_proj_w,
            Some(&layer.c_proj_b),
            &self.dense_prepared.matrices[base + 1],
        );
        for input_index in 0..d {
            state.x[input_index] += state.dense_scratch.output[input_index];
        }

        layer_norm(
            &state.x,
            &layer.ln2_w,
            &layer.ln2_b,
            self.cfg.layer_norm_eps,
            &mut state.dense_scratch.input[..d],
        );
        production_dense_projection_batched(
            std::slice::from_mut(state),
            &layer.fc_w,
            Some(&layer.fc_b),
            &self.dense_prepared.matrices[base + 2],
        );
        for value in &mut state.dense_scratch.output[..inner_dim] {
            *value = gelu_new(*value);
        }
        state.dense_scratch.input[..inner_dim]
            .copy_from_slice(&state.dense_scratch.output[..inner_dim]);
        production_dense_projection_batched(
            std::slice::from_mut(state),
            &layer.mlp_w,
            Some(&layer.mlp_b),
            &self.dense_prepared.matrices[base + 3],
        );
        for input_index in 0..d {
            state.x[input_index] += state.dense_scratch.output[input_index];
        }
    }

    fn finish_forward_production(&self, state: &mut Gpt2State) {
        let d = self.cfg.n_embd;
        layer_norm(
            &state.x,
            &self.ln_f_w,
            &self.ln_f_b,
            self.cfg.layer_norm_eps,
            &mut state.hidden,
        );
        state.dense_scratch.input[..d].copy_from_slice(&state.hidden);
        production_dense_projection_batched(
            std::slice::from_mut(state),
            &self.dense_prepared.lm_head_transposed,
            None,
            &self.dense_prepared.lm_head,
        );
        state
            .logits
            .copy_from_slice(&state.dense_scratch.output[..self.cfg.vocab]);
    }

    /// One teacher-forced forward step at `pos` (0-based), leaving logits
    /// and the final hidden state in `st`. `capture` receives the
    /// post-block residual stream for each declared layer index, in
    /// ascending order — the #599 conformance-trace tap (a no-op closure
    /// captures nothing). Invalid state/token/position/capture extents are
    /// rejected before mutation or a sink call.
    pub fn forward(
        &self,
        st: &mut Gpt2State,
        token: usize,
        pos: usize,
        capture: &[usize],
        sink: &mut dyn FnMut(usize, &[f32]),
    ) {
        if self.validate_production_step(st, token, pos).is_none()
            || capture.iter().any(|&layer| layer >= self.cfg.n_layer)
        {
            return;
        }
        let d = self.cfg.n_embd;
        // token + learned absolute position embedding.
        for i in 0..d {
            st.x[i] = self.wte[token * d + i] + self.wpe[pos * d + i];
        }
        for l in 0..self.cfg.n_layer {
            self.block_forward_production(st, l, pos, None, None);
            if capture.contains(&l) {
                sink(l, &st.x);
            }
        }
        self.finish_forward_production(st);
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

    /// Prepare model-bound scratch and an outward per-column
    /// `sum_i |weight_i|` bound for the real layer-0 fused QKV projection.
    /// This one-time weight scan is deliberately outside the candidate hot
    /// path. The returned workspace cannot be reused with another model.
    #[doc(hidden)]
    pub fn dense_canary_workspace(&self) -> Option<Gpt2DenseCanaryWorkspace> {
        self.validate_production_geometry()?;
        self.validate_attention_canary_model().ok()?;
        let in_dim = self.cfg.n_embd;
        let out_dim = canary_product("3 * n_embd", &[3, in_dim]).ok()?;
        let layer = self.layers.first()?;
        let mut sum_abs_weight_upper = vec![0.0f64; out_dim];
        for input_index in 0..in_dim {
            let row = &layer.c_attn_w[input_index * out_dim..(input_index + 1) * out_dim];
            for output_index in 0..out_dim {
                sum_abs_weight_upper[output_index] += f64::from(row[output_index]).abs();
            }
        }
        for bound in &mut sum_abs_weight_upper {
            *bound = dense_positive_sum_upper(*bound, in_dim).unwrap_or(f64::INFINITY);
        }
        Some(Gpt2DenseCanaryWorkspace {
            weight_address: layer.c_attn_w.as_ptr() as usize,
            weight_len: layer.c_attn_w.len(),
            source_kappa: self.kappa.clone(),
            in_dim,
            out_dim,
            sums: vec![0.0; out_dim],
            sum_abs_weight_upper,
        })
    }

    fn validate_dense_canary_inputs(
        &self,
        input: &[f32],
        output: &[f32],
        workspace: &Gpt2DenseCanaryWorkspace,
    ) -> Option<()> {
        self.validate_production_geometry()?;
        self.validate_attention_canary_model().ok()?;
        let layer = self.layers.first()?;
        let in_dim = self.cfg.n_embd;
        let out_dim = canary_product("3 * n_embd", &[3, in_dim]).ok()?;
        (input.len() == in_dim
            && output.len() == out_dim
            && workspace.weight_address == layer.c_attn_w.as_ptr() as usize
            && workspace.weight_len == layer.c_attn_w.len()
            && workspace.source_kappa == self.kappa
            && workspace.in_dim == in_dim
            && workspace.out_dim == out_dim
            && workspace.sums.len() == out_dim
            && workspace.sum_abs_weight_upper.len() == out_dim)
            .then_some(())
    }

    /// Materialize the real token-plus-position then layer-0 LayerNorm input
    /// consumed by fused `c_attn`. Validation completes before `out` changes.
    #[doc(hidden)]
    pub fn layer0_c_attn_canary_input(
        &self,
        token: usize,
        position: usize,
        out: &mut [f32],
    ) -> Option<()> {
        self.validate_production_geometry()?;
        self.validate_attention_canary_model().ok()?;
        let layer = self.layers.first()?;
        let d = self.cfg.n_embd;
        if token >= self.cfg.vocab || position >= self.cfg.n_positions || out.len() != d {
            return None;
        }
        for (index, value) in out.iter_mut().enumerate() {
            *value = self.wte[token * d + index] + self.wpe[position * d + index];
        }
        // LayerNorm cannot be safely in-place because it reads the complete
        // input to determine mean and variance. Reproduce its scalar order
        // directly while retaining the embedding vector in caller scratch.
        let n = d as f32;
        let mean = out.iter().sum::<f32>() / n;
        let variance = out
            .iter()
            .map(|value| (*value - mean) * (*value - mean))
            .sum::<f32>()
            / n;
        let inverse = 1.0 / (variance + self.cfg.layer_norm_eps).sqrt();
        for (index, value) in out.iter_mut().enumerate() {
            *value = (*value - mean) * inverse * layer.ln1_w[index] + layer.ln1_b[index];
        }
        Some(())
    }

    /// Execute one checked real layer-0 fused-QKV projection. Caller shape,
    /// model/workspace binding, and all derived extents are validated before
    /// either output or workspace is mutated; invalid input returns `None`.
    #[doc(hidden)]
    pub fn layer0_c_attn_canary(
        &self,
        input: &[f32],
        output: &mut [f32],
        workspace: &mut Gpt2DenseCanaryWorkspace,
        mode: Gpt2DenseCanaryMode,
    ) -> Option<Gpt2DenseCanaryCensus> {
        self.validate_dense_canary_inputs(input, output, workspace)?;
        let layer = self.layers.first()?;
        let out_dim = workspace.out_dim;
        let mut census = Gpt2DenseCanaryCensus {
            lanes: out_dim,
            ..Gpt2DenseCanaryCensus::default()
        };
        match mode {
            Gpt2DenseCanaryMode::Conventional => {
                conv1d(input, &layer.c_attn_w, &layer.c_attn_b, out_dim, output);
                census.conventional = out_dim;
            }
            Gpt2DenseCanaryMode::Exact => {
                exact_dense_projection(output, input, &layer.c_attn_w, out_dim);
                for (value, &bias) in output.iter_mut().zip(&layer.c_attn_b) {
                    *value += bias;
                }
                census.exact_control = out_dim;
            }
            Gpt2DenseCanaryMode::CertifiedNative => {
                census = certified_dense_projection(
                    output,
                    input,
                    &layer.c_attn_w,
                    &layer.c_attn_b,
                    workspace,
                );
            }
        }
        Some(census)
    }

    fn dense_control_layer_matrix(
        &self,
        layer_index: usize,
        site_offset: usize,
    ) -> Option<(&[f32], &[f32], usize, usize)> {
        let layer = self.layers.get(layer_index)?;
        let d = self.cfg.n_embd;
        match site_offset {
            0 => Some((&layer.c_attn_w, &layer.c_attn_b, d, 3 * d)),
            1 => Some((&layer.c_proj_w, &layer.c_proj_b, d, d)),
            2 => Some((&layer.fc_w, &layer.fc_b, d, self.cfg.n_inner)),
            3 => Some((&layer.mlp_w, &layer.mlp_b, self.cfg.n_inner, d)),
            _ => None,
        }
    }

    fn dense_control_prepared_index(layer_index: usize, site_offset: usize) -> Option<usize> {
        layer_index
            .checked_mul(DENSE_CONTROL_SITES_PER_LAYER)?
            .checked_add(site_offset)
    }

    /// Prepare every remaining GPT-2 dense owner for a serial checked control.
    #[doc(hidden)]
    pub fn dense_control_workspace(&self) -> Option<Gpt2DenseControlWorkspace> {
        self.dense_control_workspace_for_batch(1)
    }

    /// Prepare every dense owner and caller scratch for up to `max_batch`
    /// sequence-major rows. This includes a `[n_embd, vocab]` transpose of
    /// tied `wte`, so the exact and certified lm-heads use the same `[in,out]`
    /// row-reuse kernel without a hot-path gather or weight rewrite.
    #[doc(hidden)]
    pub fn dense_control_workspace_for_batch(
        &self,
        max_batch: usize,
    ) -> Option<Gpt2DenseControlWorkspace> {
        self.validate_production_geometry()?;
        self.validate_attention_canary_model().ok()?;
        if max_batch == 0 {
            return None;
        }
        let d = self.cfg.n_embd;
        let matrix_count = self
            .cfg
            .n_layer
            .checked_mul(DENSE_CONTROL_SITES_PER_LAYER)?;
        let mut prepared = Vec::with_capacity(matrix_count);
        let mut model_weight_addresses = Vec::with_capacity(matrix_count);
        let mut model_weight_lengths = Vec::with_capacity(matrix_count);
        for layer_index in 0..self.cfg.n_layer {
            for site_offset in 0..DENSE_CONTROL_SITES_PER_LAYER {
                let (weights, _, in_dim, out_dim) =
                    self.dense_control_layer_matrix(layer_index, site_offset)?;
                prepared.push(DensePreparedMatrix::prepare(weights, in_dim, out_dim)?);
                model_weight_addresses.push(weights.as_ptr() as usize);
                model_weight_lengths.push(weights.len());
            }
        }

        let transpose_len = d.checked_mul(self.cfg.vocab)?;
        let mut lm_head_transposed = vec![0.0f32; transpose_len];
        for vocabulary_index in 0..self.cfg.vocab {
            let row = &self.wte[vocabulary_index * d..(vocabulary_index + 1) * d];
            for input_index in 0..d {
                lm_head_transposed[input_index * self.cfg.vocab + vocabulary_index] =
                    row[input_index];
            }
        }
        let lm_head = DensePreparedMatrix::prepare(&lm_head_transposed, d, self.cfg.vocab)?;
        let maximum_output = self
            .cfg
            .vocab
            .max(self.cfg.n_inner)
            .max(3usize.checked_mul(d)?);

        Some(Gpt2DenseControlWorkspace {
            source_kappa: self.kappa.clone(),
            max_batch,
            model_weight_addresses,
            model_weight_lengths,
            wte_address: self.wte.as_ptr() as usize,
            wte_len: self.wte.len(),
            prepared,
            lm_head_transposed,
            lm_head,
            normed: vec![0.0; d],
            qkv: vec![0.0; 3 * d],
            attn: vec![0.0; d],
            proj: vec![0.0; d],
            inner: vec![0.0; self.cfg.n_inner],
            mlp_out: vec![0.0; d],
            scores: vec![0.0; self.cfg.seq_len],
            batch_normed: vec![0.0; max_batch.checked_mul(d)?],
            batch_qkv: vec![0.0; max_batch.checked_mul(3usize.checked_mul(d)?)?],
            batch_attn: vec![0.0; max_batch.checked_mul(d)?],
            batch_proj: vec![0.0; max_batch.checked_mul(d)?],
            batch_inner: vec![0.0; max_batch.checked_mul(self.cfg.n_inner)?],
            batch_mlp_out: vec![0.0; max_batch.checked_mul(d)?],
            batch_logits: vec![0.0; max_batch.checked_mul(self.cfg.vocab)?],
            batch_sums: vec![0.0; max_batch.checked_mul(maximum_output)?],
            batch_max_activation_abs: vec![0.0; max_batch],
            step_layers: vec![Gpt2DenseLayerCensus::default(); self.cfg.n_layer],
            step_lm_head: Gpt2DenseCanaryCensus::default(),
            batch_layers: vec![Gpt2DenseLayerCensus::default(); self.cfg.n_layer],
            batch_lm_head: Gpt2DenseCanaryCensus::default(),
        })
    }

    /// Report the prepared workspace's owned capacity by purpose. This is a
    /// deterministic byte census for the transpose tradeoff, not an RSS or
    /// allocator-overhead measurement.
    #[doc(hidden)]
    pub fn dense_control_workspace_bytes(
        &self,
        workspace: &Gpt2DenseControlWorkspace,
    ) -> Option<Gpt2DenseControlWorkspaceBytes> {
        self.validate_dense_control_workspace(workspace)?;
        let bytes = |count: usize, element: usize| count.checked_mul(element);
        let add = |left: usize, right: usize| left.checked_add(right);

        let mut bound_elements = workspace.lm_head.sum_abs_weight_upper.capacity();
        let mut sum_elements = workspace.lm_head.sums.capacity();
        for matrix in &workspace.prepared {
            bound_elements = bound_elements.checked_add(matrix.sum_abs_weight_upper.capacity())?;
            sum_elements = sum_elements.checked_add(matrix.sums.capacity())?;
        }
        let matrix_bounds = bytes(bound_elements, std::mem::size_of::<f64>())?;
        sum_elements = sum_elements
            .checked_add(workspace.batch_sums.capacity())?
            .checked_add(workspace.batch_max_activation_abs.capacity())?;
        let f64_sum_scratch = bytes(sum_elements, std::mem::size_of::<f64>())?;
        let lm_head_transpose = bytes(
            workspace.lm_head_transposed.capacity(),
            std::mem::size_of::<f32>(),
        )?;

        let intermediate_elements = workspace
            .normed
            .capacity()
            .checked_add(workspace.qkv.capacity())?
            .checked_add(workspace.attn.capacity())?
            .checked_add(workspace.proj.capacity())?
            .checked_add(workspace.inner.capacity())?
            .checked_add(workspace.mlp_out.capacity())?
            .checked_add(workspace.scores.capacity())?
            .checked_add(workspace.batch_normed.capacity())?
            .checked_add(workspace.batch_qkv.capacity())?
            .checked_add(workspace.batch_attn.capacity())?
            .checked_add(workspace.batch_proj.capacity())?
            .checked_add(workspace.batch_inner.capacity())?
            .checked_add(workspace.batch_mlp_out.capacity())?
            .checked_add(workspace.batch_logits.capacity())?;
        let intermediate_scratch = bytes(intermediate_elements, std::mem::size_of::<f32>())?;

        let mut metadata = std::mem::size_of::<Gpt2DenseControlWorkspace>();
        metadata = add(metadata, workspace.source_kappa.capacity())?;
        metadata = add(
            metadata,
            bytes(
                workspace.model_weight_addresses.capacity(),
                std::mem::size_of::<usize>(),
            )?,
        )?;
        metadata = add(
            metadata,
            bytes(
                workspace.model_weight_lengths.capacity(),
                std::mem::size_of::<usize>(),
            )?,
        )?;
        metadata = add(
            metadata,
            bytes(
                workspace.prepared.capacity(),
                std::mem::size_of::<DensePreparedMatrix>(),
            )?,
        )?;
        metadata = add(
            metadata,
            bytes(
                workspace.step_layers.capacity(),
                std::mem::size_of::<Gpt2DenseLayerCensus>(),
            )?,
        )?;
        metadata = add(
            metadata,
            bytes(
                workspace.batch_layers.capacity(),
                std::mem::size_of::<Gpt2DenseLayerCensus>(),
            )?,
        )?;
        let total = lm_head_transpose
            .checked_add(matrix_bounds)?
            .checked_add(f64_sum_scratch)?
            .checked_add(intermediate_scratch)?
            .checked_add(metadata)?;
        Some(Gpt2DenseControlWorkspaceBytes {
            lm_head_transpose,
            matrix_bounds,
            f64_sum_scratch,
            intermediate_scratch,
            metadata,
            total,
        })
    }

    /// Fingerprint all prepared metadata, proof scratch, batch scratch, and
    /// census storage without allocating or copying the tied-head transpose.
    #[doc(hidden)]
    pub fn dense_control_workspace_fingerprint(
        &self,
        workspace: &Gpt2DenseControlWorkspace,
    ) -> Option<Gpt2DenseControlWorkspaceFingerprint> {
        self.validate_dense_control_workspace(workspace)?;
        let mut hasher = blake3::Hasher::new();
        dense_fingerprint_usize(&mut hasher, workspace.source_kappa.len());
        dense_fingerprint_usize(&mut hasher, workspace.source_kappa.capacity());
        hasher.update(workspace.source_kappa.as_bytes());
        dense_fingerprint_usize(&mut hasher, workspace.max_batch);
        dense_fingerprint_usize(&mut hasher, workspace.model_weight_addresses.len());
        dense_fingerprint_usize(&mut hasher, workspace.model_weight_addresses.capacity());
        for &value in &workspace.model_weight_addresses {
            dense_fingerprint_usize(&mut hasher, value);
        }
        dense_fingerprint_usize(&mut hasher, workspace.model_weight_lengths.len());
        dense_fingerprint_usize(&mut hasher, workspace.model_weight_lengths.capacity());
        for &value in &workspace.model_weight_lengths {
            dense_fingerprint_usize(&mut hasher, value);
        }
        dense_fingerprint_usize(&mut hasher, workspace.wte_address);
        dense_fingerprint_usize(&mut hasher, workspace.wte_len);
        dense_fingerprint_usize(&mut hasher, workspace.prepared.len());
        dense_fingerprint_usize(&mut hasher, workspace.prepared.capacity());
        for matrix in &workspace.prepared {
            dense_fingerprint_usize(&mut hasher, matrix.in_dim);
            dense_fingerprint_usize(&mut hasher, matrix.out_dim);
            dense_fingerprint_f64_slice(&mut hasher, &matrix.sums, matrix.sums.capacity());
            dense_fingerprint_f64_slice(
                &mut hasher,
                &matrix.sum_abs_weight_upper,
                matrix.sum_abs_weight_upper.capacity(),
            );
        }
        dense_fingerprint_f32_slice(
            &mut hasher,
            &workspace.lm_head_transposed,
            workspace.lm_head_transposed.capacity(),
        );
        dense_fingerprint_usize(&mut hasher, workspace.lm_head.in_dim);
        dense_fingerprint_usize(&mut hasher, workspace.lm_head.out_dim);
        dense_fingerprint_f64_slice(
            &mut hasher,
            &workspace.lm_head.sums,
            workspace.lm_head.sums.capacity(),
        );
        dense_fingerprint_f64_slice(
            &mut hasher,
            &workspace.lm_head.sum_abs_weight_upper,
            workspace.lm_head.sum_abs_weight_upper.capacity(),
        );
        for values in [
            &workspace.normed,
            &workspace.qkv,
            &workspace.attn,
            &workspace.proj,
            &workspace.inner,
            &workspace.mlp_out,
            &workspace.scores,
            &workspace.batch_normed,
            &workspace.batch_qkv,
            &workspace.batch_attn,
            &workspace.batch_proj,
            &workspace.batch_inner,
            &workspace.batch_mlp_out,
            &workspace.batch_logits,
        ] {
            dense_fingerprint_f32_slice(&mut hasher, values, values.capacity());
        }
        dense_fingerprint_f64_slice(
            &mut hasher,
            &workspace.batch_sums,
            workspace.batch_sums.capacity(),
        );
        dense_fingerprint_f64_slice(
            &mut hasher,
            &workspace.batch_max_activation_abs,
            workspace.batch_max_activation_abs.capacity(),
        );
        dense_fingerprint_usize(&mut hasher, workspace.step_layers.len());
        dense_fingerprint_usize(&mut hasher, workspace.step_layers.capacity());
        for &layer in &workspace.step_layers {
            dense_fingerprint_layer_census(&mut hasher, layer);
        }
        dense_fingerprint_census(&mut hasher, workspace.step_lm_head);
        dense_fingerprint_usize(&mut hasher, workspace.batch_layers.len());
        dense_fingerprint_usize(&mut hasher, workspace.batch_layers.capacity());
        for &layer in &workspace.batch_layers {
            dense_fingerprint_layer_census(&mut hasher, layer);
        }
        dense_fingerprint_census(&mut hasher, workspace.batch_lm_head);
        Some(Gpt2DenseControlWorkspaceFingerprint(
            *hasher.finalize().as_bytes(),
        ))
    }

    fn validate_dense_control_workspace(
        &self,
        workspace: &Gpt2DenseControlWorkspace,
    ) -> Option<()> {
        self.validate_production_geometry()?;
        self.validate_attention_canary_model().ok()?;
        let d = self.cfg.n_embd;
        let matrix_count = self
            .cfg
            .n_layer
            .checked_mul(DENSE_CONTROL_SITES_PER_LAYER)?;
        let maximum_output = self
            .cfg
            .vocab
            .max(self.cfg.n_inner)
            .max(3usize.checked_mul(d)?);
        if workspace.source_kappa != self.kappa
            || workspace.max_batch == 0
            || workspace.model_weight_addresses.len() != matrix_count
            || workspace.model_weight_lengths.len() != matrix_count
            || workspace.prepared.len() != matrix_count
            || workspace.wte_address != self.wte.as_ptr() as usize
            || workspace.wte_len != self.wte.len()
            || workspace.lm_head_transposed.len() != d.checked_mul(self.cfg.vocab)?
            || !workspace.lm_head.has_shape(d, self.cfg.vocab)
            || workspace.normed.len() != d
            || workspace.qkv.len() != 3usize.checked_mul(d)?
            || workspace.attn.len() != d
            || workspace.proj.len() != d
            || workspace.inner.len() != self.cfg.n_inner
            || workspace.mlp_out.len() != d
            || workspace.scores.len() != self.cfg.seq_len
            || workspace.batch_normed.len() != workspace.max_batch.checked_mul(d)?
            || workspace.batch_qkv.len()
                != workspace.max_batch.checked_mul(3usize.checked_mul(d)?)?
            || workspace.batch_attn.len() != workspace.max_batch.checked_mul(d)?
            || workspace.batch_proj.len() != workspace.max_batch.checked_mul(d)?
            || workspace.batch_inner.len() != workspace.max_batch.checked_mul(self.cfg.n_inner)?
            || workspace.batch_mlp_out.len() != workspace.max_batch.checked_mul(d)?
            || workspace.batch_logits.len() != workspace.max_batch.checked_mul(self.cfg.vocab)?
            || workspace.batch_sums.len() != workspace.max_batch.checked_mul(maximum_output)?
            || workspace.batch_max_activation_abs.len() != workspace.max_batch
            || workspace.step_layers.len() != self.cfg.n_layer
            || workspace.batch_layers.len() != self.cfg.n_layer
        {
            return None;
        }
        for layer_index in 0..self.cfg.n_layer {
            for site_offset in 0..DENSE_CONTROL_SITES_PER_LAYER {
                let index = Self::dense_control_prepared_index(layer_index, site_offset)?;
                let (weights, _, in_dim, out_dim) =
                    self.dense_control_layer_matrix(layer_index, site_offset)?;
                if workspace.model_weight_addresses[index] != weights.as_ptr() as usize
                    || workspace.model_weight_lengths[index] != weights.len()
                    || !workspace.prepared[index].has_shape(in_dim, out_dim)
                {
                    return None;
                }
            }
        }
        Some(())
    }

    fn validate_dense_control_state(&self, state: &Gpt2State) -> Option<()> {
        let d = self.cfg.n_embd;
        let cache = canary_product(
            "n_layer * seq_len * n_embd",
            &[self.cfg.n_layer, self.cfg.seq_len, d],
        )
        .ok()?;
        (state.k_cache.len() == cache
            && state.v_cache.len() == cache
            && state.logits.len() == self.cfg.vocab
            && state.hidden.len() == d
            && state.x.len() == d)
            .then_some(())
    }

    /// O(1) load-geometry binding for the production executor. The public
    /// config remains mutable for source compatibility, but weights and dense
    /// proof preparation are immutable products of the load-time geometry.
    /// Reject any drift before cfg-derived indexing or arithmetic can run.
    fn validate_production_geometry(&self) -> Option<()> {
        let geometry = self.dense_prepared.geometry;
        if !geometry.matches_config(&self.cfg) {
            return None;
        }
        let embedding_values = geometry.vocab.checked_mul(geometry.n_embd)?;
        let position_values = geometry.n_positions.checked_mul(geometry.n_embd)?;
        let prepared_matrices = geometry
            .n_layer
            .checked_mul(DENSE_CONTROL_SITES_PER_LAYER)?;
        (geometry.n_head != 0
            && geometry.n_embd.is_multiple_of(geometry.n_head)
            && self.cfg.seq_len != 0
            && self.cfg.seq_len <= geometry.n_positions
            && self.wte.len() == embedding_values
            && self.wpe.len() == position_values
            && self.layers.len() == geometry.n_layer
            && self.ln_f_w.len() == geometry.n_embd
            && self.ln_f_b.len() == geometry.n_embd
            && self.dense_prepared.matrices.len() == prepared_matrices
            && self.dense_prepared.lm_head_transposed.len() == embedding_values
            && self.dense_prepared.lm_head.in_dim == geometry.n_embd
            && self.dense_prepared.lm_head.out_dim == geometry.vocab
            && self.dense_prepared.lm_head.sum_abs_weight_upper.len() == geometry.vocab)
            .then_some(())
    }

    fn validate_production_state(&self, state: &Gpt2State) -> Option<()> {
        self.validate_production_geometry()?;
        self.validate_dense_control_state(state)?;
        let d = self.cfg.n_embd;
        let maximum_input = self.cfg.n_inner.max(d);
        let maximum_output = self
            .cfg
            .vocab
            .max(self.cfg.n_inner)
            .max(3usize.checked_mul(d)?);
        (state.dense_scratch.input.len() == maximum_input
            && state.dense_scratch.output.len() == maximum_output
            && state.dense_scratch.sums.len() == maximum_output
            && state.dense_scratch.attention.len() == d
            && state.dense_scratch.scores.len() == self.cfg.seq_len)
            .then_some(())
    }

    fn validate_production_step(
        &self,
        state: &Gpt2State,
        token: usize,
        position: usize,
    ) -> Option<()> {
        self.validate_production_state(state)?;
        (token < self.cfg.vocab && position < self.cfg.seq_len).then_some(())
    }

    fn validate_dense_control_step(
        &self,
        state: &Gpt2State,
        workspace: &Gpt2DenseControlWorkspace,
        token: usize,
        position: usize,
    ) -> Option<()> {
        self.validate_dense_control_workspace(workspace)?;
        self.validate_dense_control_state(state)?;
        (token < self.cfg.vocab && position < self.cfg.seq_len).then_some(())
    }

    fn validate_dense_trace_request(&self, request: &crate::TraceCaptureRequest<'_>) -> Option<()> {
        request
            .residual_layers
            .iter()
            .chain(request.qkv_layers)
            .chain(request.attention_layers)
            .all(|&layer| layer < self.cfg.n_layer)
            .then_some(())
    }

    /// Checked matrix-level seam used to differentially exercise every real
    /// projection shape through the same arithmetic owner as the whole-model
    /// serial, trace, and batch controls.
    #[doc(hidden)]
    pub fn dense_control_matrix_canary(
        &self,
        workspace: &mut Gpt2DenseControlWorkspace,
        layer_index: Option<usize>,
        site: Gpt2DenseControlSite,
        input: &[f32],
        output: &mut [f32],
        mode: Gpt2DenseCanaryMode,
    ) -> Option<Gpt2DenseCanaryCensus> {
        self.validate_dense_control_workspace(workspace)?;
        match site {
            Gpt2DenseControlSite::LmHead => {
                if layer_index.is_some()
                    || input.len() != self.cfg.n_embd
                    || output.len() != self.cfg.vocab
                {
                    return None;
                }
                if mode == Gpt2DenseCanaryMode::Conventional {
                    conventional_lm_head(output, input, &self.wte, self.cfg.n_embd);
                    Some(Gpt2DenseCanaryCensus {
                        lanes: self.cfg.vocab,
                        conventional: self.cfg.vocab,
                        ..Gpt2DenseCanaryCensus::default()
                    })
                } else {
                    Some(controlled_dense_projection(
                        output,
                        input,
                        &workspace.lm_head_transposed,
                        None,
                        &mut workspace.lm_head,
                        mode,
                    ))
                }
            }
            _ => {
                let layer_index = layer_index?;
                let site_offset = match site {
                    Gpt2DenseControlSite::CAttn => 0,
                    Gpt2DenseControlSite::AttentionCProj => 1,
                    Gpt2DenseControlSite::MlpCFc => 2,
                    Gpt2DenseControlSite::MlpCProj => 3,
                    Gpt2DenseControlSite::LmHead => unreachable!(),
                };
                let index = Self::dense_control_prepared_index(layer_index, site_offset)?;
                let (weights, bias, in_dim, out_dim) =
                    self.dense_control_layer_matrix(layer_index, site_offset)?;
                if input.len() != in_dim || output.len() != out_dim {
                    return None;
                }
                Some(controlled_dense_projection(
                    output,
                    input,
                    weights,
                    Some(bias),
                    &mut workspace.prepared[index],
                    mode,
                ))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn block_attention_dense_control(
        &self,
        state: &mut Gpt2State,
        layer_index: usize,
        position: usize,
        qkv: &[f32],
        attention: &mut [f32],
        scores: &mut [f32],
        request: Option<&crate::TraceCaptureRequest<'_>>,
        mut sinks: Option<&mut crate::TraceCaptureSinks<'_, '_>>,
    ) {
        let d = self.cfg.n_embd;
        let head_size = self.cfg.head_size();
        let sequence_length = self.cfg.seq_len;
        let base = (layer_index * sequence_length + position) * d;
        state.k_cache[base..base + d].copy_from_slice(&qkv[d..2 * d]);
        state.v_cache[base..base + d].copy_from_slice(&qkv[2 * d..3 * d]);

        if request.is_some_and(|request| request.qkv_layers.contains(&layer_index)) {
            if let Some(sinks) = sinks.as_deref_mut() {
                (sinks.qkv)(layer_index, &qkv[..d], &qkv[d..2 * d], &qkv[2 * d..3 * d]);
            }
        }

        debug_assert_eq!(scores.len(), position + 1);
        let key_cache = &state.k_cache
            [layer_index * sequence_length * d..(layer_index + 1) * sequence_length * d];
        let value_cache = &state.v_cache
            [layer_index * sequence_length * d..(layer_index + 1) * sequence_length * d];
        for head in 0..self.cfg.n_head {
            let query = &qkv[head * head_size..(head + 1) * head_size];
            let _ = attention_weights_with_arithmetic(
                scores,
                query,
                key_cache,
                head * head_size,
                d,
                crate::attention::AttentionArithmetic::CertifiedNative,
            );
            if request.is_some_and(|request| request.attention_layers.contains(&layer_index)) {
                if let Some(sinks) = sinks.as_deref_mut() {
                    (sinks.attention)(layer_index, head, scores);
                }
            }
            let output = &mut attention[head * head_size..(head + 1) * head_size];
            let _ = crate::attention::head_attention_value_aggregate_with_arithmetic(
                output,
                scores,
                value_cache,
                head * head_size,
                d,
                crate::attention::AttentionArithmetic::CertifiedNative,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn forward_dense_control_unchecked(
        &self,
        state: &mut Gpt2State,
        workspace: &mut Gpt2DenseControlWorkspace,
        token: usize,
        position: usize,
        mode: Gpt2DenseCanaryMode,
        request: Option<&crate::TraceCaptureRequest<'_>>,
        mut sinks: Option<&mut crate::TraceCaptureSinks<'_, '_>>,
    ) {
        let d = self.cfg.n_embd;
        workspace.step_layers.fill(Gpt2DenseLayerCensus::default());
        workspace.step_lm_head = Gpt2DenseCanaryCensus::default();

        for input_index in 0..d {
            state.x[input_index] =
                self.wte[token * d + input_index] + self.wpe[position * d + input_index];
        }

        for layer_index in 0..self.cfg.n_layer {
            let layer = &self.layers[layer_index];
            layer_norm(
                &state.x,
                &layer.ln1_w,
                &layer.ln1_b,
                self.cfg.layer_norm_eps,
                &mut workspace.normed,
            );

            let c_attn_index = layer_index * DENSE_CONTROL_SITES_PER_LAYER;
            workspace.step_layers[layer_index].c_attn = controlled_dense_projection(
                &mut workspace.qkv,
                &workspace.normed,
                &layer.c_attn_w,
                Some(&layer.c_attn_b),
                &mut workspace.prepared[c_attn_index],
                mode,
            );
            self.block_attention_dense_control(
                state,
                layer_index,
                position,
                &workspace.qkv,
                &mut workspace.attn,
                &mut workspace.scores[..=position],
                request,
                sinks.as_deref_mut(),
            );

            let attention_projection_index = layer_index * DENSE_CONTROL_SITES_PER_LAYER + 1;
            workspace.step_layers[layer_index].attention_c_proj = controlled_dense_projection(
                &mut workspace.proj,
                &workspace.attn,
                &layer.c_proj_w,
                Some(&layer.c_proj_b),
                &mut workspace.prepared[attention_projection_index],
                mode,
            );
            for (value, &projection) in state.x.iter_mut().zip(&workspace.proj) {
                *value += projection;
            }

            layer_norm(
                &state.x,
                &layer.ln2_w,
                &layer.ln2_b,
                self.cfg.layer_norm_eps,
                &mut workspace.normed,
            );
            let fc_index = layer_index * DENSE_CONTROL_SITES_PER_LAYER + 2;
            workspace.step_layers[layer_index].mlp_c_fc = controlled_dense_projection(
                &mut workspace.inner,
                &workspace.normed,
                &layer.fc_w,
                Some(&layer.fc_b),
                &mut workspace.prepared[fc_index],
                mode,
            );
            for value in &mut workspace.inner {
                *value = gelu_new(*value);
            }

            let mlp_projection_index = layer_index * DENSE_CONTROL_SITES_PER_LAYER + 3;
            workspace.step_layers[layer_index].mlp_c_proj = controlled_dense_projection(
                &mut workspace.mlp_out,
                &workspace.inner,
                &layer.mlp_w,
                Some(&layer.mlp_b),
                &mut workspace.prepared[mlp_projection_index],
                mode,
            );
            for (value, &projection) in state.x.iter_mut().zip(&workspace.mlp_out) {
                *value += projection;
            }
            if request.is_some_and(|request| request.residual_layers.contains(&layer_index)) {
                if let Some(sinks) = sinks.as_deref_mut() {
                    (sinks.residual)(layer_index, &state.x);
                }
            }
        }

        layer_norm(
            &state.x,
            &self.ln_f_w,
            &self.ln_f_b,
            self.cfg.layer_norm_eps,
            &mut state.hidden,
        );
        workspace.step_lm_head = if mode == Gpt2DenseCanaryMode::Conventional {
            conventional_lm_head(&mut state.logits, &state.hidden, &self.wte, d);
            Gpt2DenseCanaryCensus {
                lanes: self.cfg.vocab,
                conventional: self.cfg.vocab,
                ..Gpt2DenseCanaryCensus::default()
            }
        } else {
            controlled_dense_projection(
                &mut state.logits,
                &state.hidden,
                &workspace.lm_head_transposed,
                None,
                &mut workspace.lm_head,
                mode,
            )
        };
    }

    /// Run one evidence-only whole-model dense control. Attention arithmetic
    /// is hard-bound to the production certified-native path for every arm;
    /// only the five dense owner classes vary.
    #[doc(hidden)]
    pub fn forward_dense_control<'workspace>(
        &self,
        state: &mut Gpt2State,
        workspace: &'workspace mut Gpt2DenseControlWorkspace,
        token: usize,
        position: usize,
        mode: Gpt2DenseCanaryMode,
    ) -> Option<Gpt2DenseControlCensus<'workspace>> {
        self.validate_dense_control_step(state, workspace, token, position)?;
        self.forward_dense_control_unchecked(state, workspace, token, position, mode, None, None);
        Some(Gpt2DenseControlCensus {
            layers: &workspace.step_layers,
            lm_head: workspace.step_lm_head,
        })
    }

    /// Trace-capable form of [`Self::forward_dense_control`]. All requested
    /// layer extents are validated before state, workspace, or any sink is
    /// touched.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn forward_dense_control_capturing_trace<'workspace>(
        &self,
        state: &mut Gpt2State,
        workspace: &'workspace mut Gpt2DenseControlWorkspace,
        token: usize,
        position: usize,
        mode: Gpt2DenseCanaryMode,
        request: &crate::TraceCaptureRequest<'_>,
        sinks: &mut crate::TraceCaptureSinks<'_, '_>,
    ) -> Option<Gpt2DenseControlCensus<'workspace>> {
        self.validate_dense_control_step(state, workspace, token, position)?;
        self.validate_dense_trace_request(request)?;
        self.forward_dense_control_unchecked(
            state,
            workspace,
            token,
            position,
            mode,
            Some(request),
            Some(sinks),
        );
        Some(Gpt2DenseControlCensus {
            layers: &workspace.step_layers,
            lm_head: workspace.step_lm_head,
        })
    }

    /// Differential row-reuse batch control. Every projection visits inputs
    /// in the serial order for each sequence while sharing each immutable
    /// weight row across the batch. The production batch uses the same
    /// certified projection owner with state-local scratch.
    #[doc(hidden)]
    // Indexed sequence-major loops keep every residual/cache/scratch row in
    // lockstep while each projection reuses immutable weight rows.
    #[allow(clippy::needless_range_loop)]
    pub fn forward_batch_dense_control<'workspace>(
        &self,
        states: &mut [Gpt2State],
        tokens: &[usize],
        positions: &[usize],
        workspace: &'workspace mut Gpt2DenseControlWorkspace,
        mode: Gpt2DenseCanaryMode,
    ) -> Option<Gpt2DenseControlCensus<'workspace>> {
        self.validate_dense_control_workspace(workspace)?;
        if states.len() != tokens.len() || states.len() != positions.len() {
            return None;
        }
        if states.len() > workspace.max_batch {
            return None;
        }
        for ((state, &token), &position) in states.iter().zip(tokens).zip(positions) {
            self.validate_dense_control_state(state)?;
            if token >= self.cfg.vocab || position >= self.cfg.seq_len {
                return None;
            }
        }
        for layer_index in 0..self.cfg.n_layer {
            for site_offset in 0..DENSE_CONTROL_SITES_PER_LAYER {
                let (_, _, _, out_dim) =
                    self.dense_control_layer_matrix(layer_index, site_offset)?;
                out_dim.checked_mul(states.len())?;
            }
        }
        self.cfg.vocab.checked_mul(states.len())?;

        let batch = states.len();
        let d = self.cfg.n_embd;
        let inner_dim = self.cfg.n_inner;
        workspace.batch_layers.fill(Gpt2DenseLayerCensus::default());
        workspace.batch_lm_head = Gpt2DenseCanaryCensus::default();
        if batch == 0 {
            return Some(Gpt2DenseControlCensus {
                layers: &workspace.batch_layers,
                lm_head: workspace.batch_lm_head,
            });
        }

        for batch_index in 0..batch {
            for input_index in 0..d {
                states[batch_index].x[input_index] = self.wte
                    [tokens[batch_index] * d + input_index]
                    + self.wpe[positions[batch_index] * d + input_index];
            }
        }

        for layer_index in 0..self.cfg.n_layer {
            let layer = &self.layers[layer_index];
            for batch_index in 0..batch {
                layer_norm(
                    &states[batch_index].x,
                    &layer.ln1_w,
                    &layer.ln1_b,
                    self.cfg.layer_norm_eps,
                    &mut workspace.batch_normed[batch_index * d..(batch_index + 1) * d],
                );
            }

            let c_attn_index = layer_index * DENSE_CONTROL_SITES_PER_LAYER;
            workspace.batch_layers[layer_index].c_attn = controlled_dense_projection_batched(
                &mut workspace.batch_qkv[..batch * 3 * d],
                &workspace.batch_normed[..batch * d],
                &layer.c_attn_w,
                Some(&layer.c_attn_b),
                batch,
                &workspace.prepared[c_attn_index],
                &mut workspace.batch_sums[..batch * 3 * d],
                &mut workspace.batch_max_activation_abs[..batch],
                mode,
            );
            for batch_index in 0..batch {
                self.block_attention_dense_control(
                    &mut states[batch_index],
                    layer_index,
                    positions[batch_index],
                    &workspace.batch_qkv[batch_index * 3 * d..(batch_index + 1) * 3 * d],
                    &mut workspace.batch_attn[batch_index * d..(batch_index + 1) * d],
                    &mut workspace.scores[..=positions[batch_index]],
                    None,
                    None,
                );
            }

            let attention_projection_index = layer_index * DENSE_CONTROL_SITES_PER_LAYER + 1;
            workspace.batch_layers[layer_index].attention_c_proj =
                controlled_dense_projection_batched(
                    &mut workspace.batch_proj[..batch * d],
                    &workspace.batch_attn[..batch * d],
                    &layer.c_proj_w,
                    Some(&layer.c_proj_b),
                    batch,
                    &workspace.prepared[attention_projection_index],
                    &mut workspace.batch_sums[..batch * d],
                    &mut workspace.batch_max_activation_abs[..batch],
                    mode,
                );
            for batch_index in 0..batch {
                for input_index in 0..d {
                    states[batch_index].x[input_index] +=
                        workspace.batch_proj[batch_index * d + input_index];
                }
                layer_norm(
                    &states[batch_index].x,
                    &layer.ln2_w,
                    &layer.ln2_b,
                    self.cfg.layer_norm_eps,
                    &mut workspace.batch_normed[batch_index * d..(batch_index + 1) * d],
                );
            }

            let fc_index = layer_index * DENSE_CONTROL_SITES_PER_LAYER + 2;
            workspace.batch_layers[layer_index].mlp_c_fc = controlled_dense_projection_batched(
                &mut workspace.batch_inner[..batch * inner_dim],
                &workspace.batch_normed[..batch * d],
                &layer.fc_w,
                Some(&layer.fc_b),
                batch,
                &workspace.prepared[fc_index],
                &mut workspace.batch_sums[..batch * inner_dim],
                &mut workspace.batch_max_activation_abs[..batch],
                mode,
            );
            for value in &mut workspace.batch_inner[..batch * inner_dim] {
                *value = gelu_new(*value);
            }

            let mlp_projection_index = layer_index * DENSE_CONTROL_SITES_PER_LAYER + 3;
            workspace.batch_layers[layer_index].mlp_c_proj = controlled_dense_projection_batched(
                &mut workspace.batch_mlp_out[..batch * d],
                &workspace.batch_inner[..batch * inner_dim],
                &layer.mlp_w,
                Some(&layer.mlp_b),
                batch,
                &workspace.prepared[mlp_projection_index],
                &mut workspace.batch_sums[..batch * d],
                &mut workspace.batch_max_activation_abs[..batch],
                mode,
            );
            for batch_index in 0..batch {
                for input_index in 0..d {
                    states[batch_index].x[input_index] +=
                        workspace.batch_mlp_out[batch_index * d + input_index];
                }
            }
        }

        for batch_index in 0..batch {
            layer_norm(
                &states[batch_index].x,
                &self.ln_f_w,
                &self.ln_f_b,
                self.cfg.layer_norm_eps,
                &mut workspace.batch_normed[batch_index * d..(batch_index + 1) * d],
            );
            states[batch_index]
                .hidden
                .copy_from_slice(&workspace.batch_normed[batch_index * d..(batch_index + 1) * d]);
        }
        workspace.batch_lm_head = if mode == Gpt2DenseCanaryMode::Conventional {
            for vocabulary_index in 0..self.cfg.vocab {
                let row = &self.wte[vocabulary_index * d..(vocabulary_index + 1) * d];
                for batch_index in 0..batch {
                    let hidden = &workspace.batch_normed[batch_index * d..(batch_index + 1) * d];
                    let mut accumulator = 0.0f32;
                    for input_index in 0..d {
                        accumulator += hidden[input_index] * row[input_index];
                    }
                    workspace.batch_logits[batch_index * self.cfg.vocab + vocabulary_index] =
                        accumulator;
                }
            }
            Gpt2DenseCanaryCensus {
                lanes: batch * self.cfg.vocab,
                batch_rows: batch,
                conventional: batch * self.cfg.vocab,
                ..Gpt2DenseCanaryCensus::default()
            }
        } else {
            controlled_dense_projection_batched(
                &mut workspace.batch_logits[..batch * self.cfg.vocab],
                &workspace.batch_normed[..batch * d],
                &workspace.lm_head_transposed,
                None,
                batch,
                &workspace.lm_head,
                &mut workspace.batch_sums[..batch * self.cfg.vocab],
                &mut workspace.batch_max_activation_abs[..batch],
                mode,
            )
        };
        for batch_index in 0..batch {
            states[batch_index].logits.copy_from_slice(
                &workspace.batch_logits
                    [batch_index * self.cfg.vocab..(batch_index + 1) * self.cfg.vocab],
            );
        }
        Some(Gpt2DenseControlCensus {
            layers: &workspace.batch_layers,
            lm_head: workspace.batch_lm_head,
        })
    }

    /// Execute one matched attention-canary step after validating every model,
    /// state, workspace, token, position, and derived slice extent. Validation
    /// completes before any mutable byte is touched, so `None` is failure
    /// atomic. Its dense arithmetic intentionally stays conventional so the
    /// historical attention-only experiment continues to vary attention only.
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

    /// One teacher-forced forward step at `pos` with the #603 trace lanes
    /// captured through the production v2 executor path: the post-block residual
    /// stream, the current-position q/k/v, and the per-head softmax
    /// attention weights, each for the layer indices `request` declares.
    /// A traced step leaves the same logits, hidden state, and k/v caches
    /// as [`Gpt2::forward`] — the taps read the executor's own
    /// intermediates, they never recompute. Sinks fire in ascending layer
    /// order (heads ascending within a layer for the attention lane). Invalid
    /// caller extents leave state and sinks untouched.
    pub fn forward_capturing_trace(
        &self,
        st: &mut Gpt2State,
        token: usize,
        pos: usize,
        request: &crate::TraceCaptureRequest<'_>,
        sinks: &mut crate::TraceCaptureSinks<'_, '_>,
    ) {
        if self.validate_production_step(st, token, pos).is_none()
            || self.validate_dense_trace_request(request).is_none()
        {
            return;
        }
        let d = self.cfg.n_embd;
        // token + learned absolute position embedding.
        for i in 0..d {
            st.x[i] = self.wte[token * d + i] + self.wpe[pos * d + i];
        }
        for l in 0..self.cfg.n_layer {
            self.block_forward_production(st, l, pos, Some(request), Some(&mut *sinks));
            if request.residual_layers.contains(&l) {
                (sinks.residual)(l, &st.x);
            }
        }
        self.finish_forward_production(st);
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

    /// Batched forward: advance `states.len()` independent sequences by one
    /// position each — sequence `bi` steps `tokens[bi]` at `positions[bi]`
    /// against its own k/v cache in `states[bi]`. Every projection, including
    /// the tied lm-head, runs through the certified row-reuse kernel while
    /// preserving each sequence's serial input-index accumulation order. The
    /// complete batch is validated before the first state byte changes.
    // Sequence-major index loops are deliberate here: the accumulation order
    // must match the serial `forward` byte-for-byte, and the offsets index
    // several stacked scratch buffers in lockstep.
    #[allow(clippy::needless_range_loop)]
    pub fn forward_batch(&self, states: &mut [Gpt2State], tokens: &[usize], positions: &[usize]) {
        let d = self.cfg.n_embd;
        let inner_dim = self.cfg.n_inner;
        let b = states.len();
        if tokens.len() != b
            || positions.len() != b
            || states
                .iter()
                .zip(tokens)
                .zip(positions)
                .any(|((state, &token), &position)| {
                    self.validate_production_step(state, token, position)
                        .is_none()
                })
        {
            return;
        }
        if b == 0 {
            return;
        }

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
            let base = l * DENSE_CONTROL_SITES_PER_LAYER;
            for bi in 0..b {
                layer_norm(
                    &states[bi].x,
                    &layer.ln1_w,
                    &layer.ln1_b,
                    self.cfg.layer_norm_eps,
                    &mut states[bi].dense_scratch.input[..d],
                );
            }
            production_dense_projection_batched(
                states,
                &layer.c_attn_w,
                Some(&layer.c_attn_b),
                &self.dense_prepared.matrices[base],
            );
            for bi in 0..b {
                let Gpt2State {
                    k_cache,
                    v_cache,
                    dense_scratch,
                    ..
                } = &mut states[bi];
                self.block_attention_production(
                    k_cache,
                    v_cache,
                    l,
                    positions[bi],
                    &dense_scratch.output[..3 * d],
                    &mut dense_scratch.attention[..d],
                    &mut dense_scratch.scores[..=positions[bi]],
                    None,
                    None,
                );
                dense_scratch.input[..d].copy_from_slice(&dense_scratch.attention[..d]);
            }
            production_dense_projection_batched(
                states,
                &layer.c_proj_w,
                Some(&layer.c_proj_b),
                &self.dense_prepared.matrices[base + 1],
            );
            for bi in 0..b {
                let x = &mut states[bi].x;
                for i in 0..d {
                    x[i] += states[bi].dense_scratch.output[i];
                }
            }
            for bi in 0..b {
                layer_norm(
                    &states[bi].x,
                    &layer.ln2_w,
                    &layer.ln2_b,
                    self.cfg.layer_norm_eps,
                    &mut states[bi].dense_scratch.input[..d],
                );
            }
            production_dense_projection_batched(
                states,
                &layer.fc_w,
                Some(&layer.fc_b),
                &self.dense_prepared.matrices[base + 2],
            );
            for state in states.iter_mut() {
                for value in &mut state.dense_scratch.output[..inner_dim] {
                    *value = gelu_new(*value);
                }
                state.dense_scratch.input[..inner_dim]
                    .copy_from_slice(&state.dense_scratch.output[..inner_dim]);
            }
            production_dense_projection_batched(
                states,
                &layer.mlp_w,
                Some(&layer.mlp_b),
                &self.dense_prepared.matrices[base + 3],
            );
            for bi in 0..b {
                let x = &mut states[bi].x;
                for i in 0..d {
                    x[i] += states[bi].dense_scratch.output[i];
                }
            }
        }

        // Final LayerNorm followed by the same row-reuse certified kernel over
        // the immutable input-major tied-head preparation.
        for bi in 0..b {
            let st = &mut states[bi];
            layer_norm(
                &st.x,
                &self.ln_f_w,
                &self.ln_f_b,
                self.cfg.layer_norm_eps,
                &mut st.hidden,
            );
            st.dense_scratch.input[..d].copy_from_slice(&st.hidden);
        }
        production_dense_projection_batched(
            states,
            &self.dense_prepared.lm_head_transposed,
            None,
            &self.dense_prepared.lm_head,
        );
        for state in states {
            state
                .logits
                .copy_from_slice(&state.dense_scratch.output[..self.cfg.vocab]);
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

    fn assert_production_geometry_rejected_without_mutation(
        model: &Gpt2,
        template: &Gpt2State,
        context: &str,
    ) {
        let mut serial = template.clone();
        let serial_before = serial.clone();
        let serial_sink_hits = std::cell::Cell::new(0usize);
        let mut serial_sink = |_: usize, _: &[f32]| {
            serial_sink_hits.set(serial_sink_hits.get() + 1);
        };
        model.forward(&mut serial, 0, 0, &[0], &mut serial_sink);
        assert_eq!(serial_sink_hits.get(), 0, "{context}: serial sink fired");
        assert!(
            serial.full_storage_bit_identical(&serial_before),
            "{context}: rejected serial call changed logical or scratch state"
        );

        let mut traced = template.clone();
        let traced_before = traced.clone();
        let trace_sink_hits = std::cell::Cell::new(0usize);
        let mut residual = |_: usize, _: &[f32]| {
            trace_sink_hits.set(trace_sink_hits.get() + 1);
        };
        let mut qkv = |_: usize, _: &[f32], _: &[f32], _: &[f32]| {
            trace_sink_hits.set(trace_sink_hits.get() + 1);
        };
        let mut attention = |_: usize, _: usize, _: &[f32]| {
            trace_sink_hits.set(trace_sink_hits.get() + 1);
        };
        let request = crate::TraceCaptureRequest {
            residual_layers: &[0],
            qkv_layers: &[0],
            attention_layers: &[0],
        };
        let mut sinks = crate::TraceCaptureSinks {
            residual: &mut residual,
            qkv: &mut qkv,
            attention: &mut attention,
        };
        model.forward_capturing_trace(&mut traced, 0, 0, &request, &mut sinks);
        assert_eq!(trace_sink_hits.get(), 0, "{context}: trace sink fired");
        assert!(
            traced.full_storage_bit_identical(&traced_before),
            "{context}: rejected trace call changed logical or scratch state"
        );

        let mut batch = vec![template.clone(), template.clone()];
        let batch_before = batch.clone();
        model.forward_batch(&mut batch, &[0, 0], &[0, 0]);
        assert!(
            batch
                .iter()
                .zip(&batch_before)
                .all(|(state, before)| state.full_storage_bit_identical(before)),
            "{context}: rejected batch call changed logical or scratch state"
        );
    }

    #[test]
    fn state_partial_eq_excludes_only_opaque_production_scratch() {
        let model = Gpt2::load(fixture_dir(), None).unwrap();
        let state = Gpt2State::new(&model.cfg);
        let mut scratch_changed = state.clone();
        scratch_changed.dense_scratch.input[0] = 1.0;
        scratch_changed.dense_scratch.output[0] = -2.0;
        scratch_changed.dense_scratch.sums[0] = 3.0;
        scratch_changed.dense_scratch.max_activation_abs = 4.0;
        scratch_changed.dense_scratch.attention[0] = -5.0;
        scratch_changed.dense_scratch.scores[0] = 6.0;
        assert_eq!(
            state, scratch_changed,
            "opaque dense workspace residue changed logical state equality"
        );
        assert!(
            !state.full_storage_bit_identical(&scratch_changed),
            "full-storage test helper did not observe scratch drift"
        );

        for logical_field in 0..5 {
            let mut changed = state.clone();
            match logical_field {
                0 => changed.k_cache[0] = 1.0,
                1 => changed.v_cache[0] = 1.0,
                2 => changed.logits[0] = 1.0,
                3 => changed.hidden[0] = 1.0,
                4 => changed.x[0] = 1.0,
                _ => unreachable!(),
            }
            assert_ne!(
                state, changed,
                "historical logical field {logical_field} was omitted from equality"
            );
        }
    }

    #[test]
    fn production_preflight_binds_load_geometry_before_all_mutation() {
        let mut zero_head = Gpt2::load(fixture_dir(), None).unwrap();
        let mut existing_state = Gpt2State::new(&zero_head.cfg);
        zero_head.forward(&mut existing_state, 1, 0, &[], &mut |_, _| {});
        zero_head.cfg.n_head = 0;
        assert_production_geometry_rejected_without_mutation(
            &zero_head,
            &existing_state,
            "zero-head drift",
        );

        let mut layer_drift = Gpt2::load(fixture_dir(), None).unwrap();
        let mut existing_state = Gpt2State::new(&layer_drift.cfg);
        layer_drift.forward(&mut existing_state, 1, 0, &[], &mut |_, _| {});
        layer_drift.cfg.n_layer += 1;
        assert_production_geometry_rejected_without_mutation(
            &layer_drift,
            &existing_state,
            "layer-count drift",
        );

        let mut width_drift = Gpt2::load(fixture_dir(), None).unwrap();
        width_drift.cfg.n_embd += width_drift.cfg.n_head;
        let newly_constructed_for_mutated_cfg = Gpt2State::new(&width_drift.cfg);
        assert_production_geometry_rejected_without_mutation(
            &width_drift,
            &newly_constructed_for_mutated_cfg,
            "width drift with newly constructed state",
        );

        // BOS/EOS are caller token-selection policy, not executor geometry.
        // Preserve the historical ability to adjust them without disabling a
        // direct explicit-token forward call.
        let baseline = Gpt2::load(fixture_dir(), None).unwrap();
        let mut policy_adjusted = Gpt2::load(fixture_dir(), None).unwrap();
        policy_adjusted.cfg.bos = (policy_adjusted.cfg.bos + 1) % policy_adjusted.cfg.vocab;
        policy_adjusted.cfg.eos = (policy_adjusted.cfg.eos + 2) % policy_adjusted.cfg.vocab;
        let mut baseline_state = Gpt2State::new(&baseline.cfg);
        let mut policy_state = Gpt2State::new(&policy_adjusted.cfg);
        baseline.forward(&mut baseline_state, 1, 0, &[], &mut |_, _| {});
        policy_adjusted.forward(&mut policy_state, 1, 0, &[], &mut |_, _| {});
        assert!(
            policy_state.full_storage_bit_identical(&baseline_state),
            "BOS/EOS policy changes altered explicit-token production execution"
        );

        // The working context is caller-selected execution capacity rather
        // than checkpoint geometry. A safe in-range adjustment with a newly
        // sized state must behave exactly like loading at that context length.
        let full_context = Gpt2::load(fixture_dir(), None).unwrap();
        let shorter_context = (full_context.cfg.seq_len / 2).max(1);
        assert!(shorter_context < full_context.cfg.seq_len);
        let reference_short = Gpt2::load(fixture_dir(), Some(shorter_context)).unwrap();
        let mut adjusted_context = full_context;
        adjusted_context.cfg.seq_len = shorter_context;
        let mut reference_state = Gpt2State::new(&reference_short.cfg);
        let mut adjusted_state = Gpt2State::new(&adjusted_context.cfg);
        reference_short.forward(&mut reference_state, 1, 0, &[], &mut |_, _| {});
        adjusted_context.forward(&mut adjusted_state, 1, 0, &[], &mut |_, _| {});
        assert!(
            adjusted_state.full_storage_bit_identical(&reference_state),
            "safe working-context adjustment diverged from bounded load"
        );
    }

    fn assert_dense_evidence_rejects_post_workspace_geometry_drift(
        mut model: Gpt2,
        drift: impl FnOnce(&mut Gpt2Config),
        context: &str,
    ) {
        let original_cfg = model.cfg.clone();
        let mut layer0_workspace = model.dense_canary_workspace().unwrap();
        let layer0_workspace_before = layer0_workspace.clone();
        let mut whole_workspace = model.dense_control_workspace_for_batch(2).unwrap();
        let whole_workspace_before = model
            .dense_control_workspace_fingerprint(&whole_workspace)
            .unwrap();

        let mut state = Gpt2State::new(&model.cfg);
        model.forward(&mut state, 1, 0, &[], &mut |_, _| {});
        let state_before = state.clone();
        let mut traced = state.clone();
        let traced_before = traced.clone();
        let mut batch = vec![state.clone(), state.clone()];
        let batch_before = batch.clone();

        let d = model.cfg.n_embd;
        let mut layer0_input = vec![f32::from_bits(0x7fc0_0704); d];
        let layer0_input_before = layer0_input.clone();
        let dense_input = vec![0.25f32; d];
        let mut dense_output = vec![f32::from_bits(0x7fc0_0704); 3 * d];
        let dense_output_before = dense_output.clone();

        drift(&mut model.cfg);

        assert!(
            model.dense_canary_workspace().is_none(),
            "{context}: layer-0 workspace construction accepted cfg drift"
        );
        assert!(
            model.dense_control_workspace().is_none(),
            "{context}: whole workspace construction accepted cfg drift"
        );
        assert!(
            model
                .layer0_c_attn_canary_input(1, 0, &mut layer0_input)
                .is_none(),
            "{context}: layer-0 input helper accepted cfg drift"
        );
        assert_dense_bits(
            &layer0_input,
            &layer0_input_before,
            &format!("{context}: rejected layer-0 input helper"),
        );
        assert!(
            model
                .layer0_c_attn_canary(
                    &dense_input,
                    &mut dense_output,
                    &mut layer0_workspace,
                    Gpt2DenseCanaryMode::CertifiedNative,
                )
                .is_none(),
            "{context}: layer-0 projection accepted cfg drift"
        );
        assert_dense_bits(
            &dense_output,
            &dense_output_before,
            &format!("{context}: rejected layer-0 projection"),
        );
        assert_eq!(
            layer0_workspace, layer0_workspace_before,
            "{context}: rejected layer-0 projection mutated workspace"
        );

        assert!(
            model
                .forward_dense_control(
                    &mut state,
                    &mut whole_workspace,
                    1,
                    1,
                    Gpt2DenseCanaryMode::CertifiedNative,
                )
                .is_none(),
            "{context}: serial dense control accepted cfg drift"
        );
        assert!(
            state.full_storage_bit_identical(&state_before),
            "{context}: rejected serial dense control mutated state"
        );

        let sink_hits = std::cell::Cell::new(0usize);
        let mut residual = |_: usize, _: &[f32]| sink_hits.set(sink_hits.get() + 1);
        let mut qkv = |_: usize, _: &[f32], _: &[f32], _: &[f32]| {
            sink_hits.set(sink_hits.get() + 1);
        };
        let mut attention = |_: usize, _: usize, _: &[f32]| {
            sink_hits.set(sink_hits.get() + 1);
        };
        let request = crate::TraceCaptureRequest {
            residual_layers: &[0],
            qkv_layers: &[0],
            attention_layers: &[0],
        };
        let mut sinks = crate::TraceCaptureSinks {
            residual: &mut residual,
            qkv: &mut qkv,
            attention: &mut attention,
        };
        assert!(
            model
                .forward_dense_control_capturing_trace(
                    &mut traced,
                    &mut whole_workspace,
                    1,
                    1,
                    Gpt2DenseCanaryMode::CertifiedNative,
                    &request,
                    &mut sinks,
                )
                .is_none(),
            "{context}: traced dense control accepted cfg drift"
        );
        assert_eq!(sink_hits.get(), 0, "{context}: trace sink fired");
        assert!(
            traced.full_storage_bit_identical(&traced_before),
            "{context}: rejected traced dense control mutated state"
        );

        assert!(
            model
                .forward_batch_dense_control(
                    &mut batch,
                    &[1, 2],
                    &[1, 1],
                    &mut whole_workspace,
                    Gpt2DenseCanaryMode::CertifiedNative,
                )
                .is_none(),
            "{context}: batch dense control accepted cfg drift"
        );
        assert!(
            batch
                .iter()
                .zip(&batch_before)
                .all(|(state, before)| state.full_storage_bit_identical(before)),
            "{context}: rejected batch dense control mutated state"
        );

        assert!(
            model
                .dense_control_workspace_fingerprint(&whole_workspace)
                .is_none(),
            "{context}: workspace validator accepted cfg drift"
        );
        model.cfg = original_cfg;
        assert_eq!(
            model
                .dense_control_workspace_fingerprint(&whole_workspace)
                .unwrap(),
            whole_workspace_before,
            "{context}: rejected dense controls mutated whole workspace"
        );
    }

    #[test]
    fn dense_evidence_workspaces_bind_frozen_arithmetic_geometry() {
        assert_dense_evidence_rejects_post_workspace_geometry_drift(
            Gpt2::load(fixture_dir(), None).unwrap(),
            |cfg| cfg.layer_norm_eps = f32::from_bits(cfg.layer_norm_eps.to_bits() ^ 1),
            "layer-norm epsilon drift",
        );
        assert_dense_evidence_rejects_post_workspace_geometry_drift(
            Gpt2::load(fixture_dir(), None).unwrap(),
            |cfg| {
                cfg.n_head = (1..=cfg.n_embd)
                    .find(|&heads| heads != cfg.n_head && cfg.n_embd.is_multiple_of(heads))
                    .expect("tiny fixture has an alternate valid head count");
            },
            "alternate valid head-count drift",
        );
    }

    fn dense_test_workspace(
        weights: &[f32],
        in_dim: usize,
        out_dim: usize,
    ) -> Gpt2DenseCanaryWorkspace {
        let mut sum_abs_weight_upper = vec![0.0f64; out_dim];
        for input_index in 0..in_dim {
            for output_index in 0..out_dim {
                sum_abs_weight_upper[output_index] +=
                    f64::from(weights[input_index * out_dim + output_index]).abs();
            }
        }
        for bound in &mut sum_abs_weight_upper {
            *bound = dense_positive_sum_upper(*bound, in_dim).expect("finite test weight bound");
        }
        Gpt2DenseCanaryWorkspace {
            weight_address: weights.as_ptr() as usize,
            weight_len: weights.len(),
            source_kappa: "test".to_owned(),
            in_dim,
            out_dim,
            sums: vec![0.0; out_dim],
            sum_abs_weight_upper,
        }
    }

    fn assert_dense_bits(got: &[f32], expected: &[f32], context: &str) {
        for (lane, (&got, &expected)) in got.iter().zip(expected).enumerate() {
            assert_eq!(got.to_bits(), expected.to_bits(), "{context}: lane {lane}");
        }
    }

    #[test]
    fn dense_refinement_and_exact_fallback_are_bit_identical_to_exact_owner() {
        let input = [1.0f32, -2.0, 0.25, 4.0];
        let weights = [
            0.5f32, -0.25, 2.0, 0.75, 0.5, -1.0, -2.0, 4.0, 0.125, 0.25, -0.5, 0.75,
        ];
        let bias = [0.125f32, -0.25, 0.5];
        let mut workspace = dense_test_workspace(&weights, input.len(), bias.len());
        // A deliberately loose but still valid upper bound forces every fast
        // lane through the TwoSum refinement without changing the proof.
        workspace.sum_abs_weight_upper.fill(1.0e300);
        let mut candidate = [0.0f32; 3];
        let census =
            certified_dense_projection(&mut candidate, &input, &weights, &bias, &mut workspace);
        let mut exact = [0.0f32; 3];
        exact_dense_projection(&mut exact, &input, &weights, bias.len());
        for (value, addend) in exact.iter_mut().zip(bias) {
            *value += addend;
        }
        assert_dense_bits(&candidate, &exact, "TwoSum refinement");
        assert_eq!(census.fast_certified, 0);
        assert_eq!(census.refined_certified, bias.len());
        assert_eq!(census.fallbacks(), Some(0));

        // 1 + 2^-24 is exactly the midpoint between 1 and its successor.
        // Strict containment must refuse both cells and delegate ties-to-even
        // to the pinned exact owner.
        let tie_term = f32::from_bits(115 << 23);
        let tie_input = [1.0f32, tie_term];
        let tie_weights = [1.0f32, tie_term];
        let tie_bias = [f32::from_bits(0x3380_0000)];
        let mut tie_workspace = dense_test_workspace(&tie_weights, 2, 1);
        let mut tie_candidate = [0.0f32];
        let tie_census = certified_dense_projection(
            &mut tie_candidate,
            &tie_input,
            &tie_weights,
            &tie_bias,
            &mut tie_workspace,
        );
        let mut tie_exact = [0.0f32];
        exact_dense_projection(&mut tie_exact, &tie_input, &tie_weights, 1);
        tie_exact[0] += tie_bias[0];
        assert_dense_bits(&tie_candidate, &tie_exact, "midpoint fallback");
        assert_eq!(tie_census.fallback_cell, 1);

        let zero_input = [0.0f32, 0.0];
        let mut zero_candidate = [0.0f32];
        let zero_census = certified_dense_projection(
            &mut zero_candidate,
            &zero_input,
            &tie_weights,
            &tie_bias,
            &mut tie_workspace,
        );
        assert_eq!(zero_candidate.map(f32::to_bits), tie_bias.map(f32::to_bits));
        assert_eq!(zero_census.fallback_zero, 1);

        let overflow_input = [f32::MAX];
        let overflow_weights = [f32::MAX];
        let overflow_bias = [0.0f32];
        let mut overflow_workspace = dense_test_workspace(&overflow_weights, 1, 1);
        let mut overflow_candidate = [0.0f32];
        let overflow_census = certified_dense_projection(
            &mut overflow_candidate,
            &overflow_input,
            &overflow_weights,
            &overflow_bias,
            &mut overflow_workspace,
        );
        let mut overflow_exact = [0.0f32];
        exact_dense_projection(&mut overflow_exact, &overflow_input, &overflow_weights, 1);
        assert_dense_bits(&overflow_candidate, &overflow_exact, "overflow fallback");
        assert_eq!(overflow_census.fallback_overflow, 1);
    }

    #[test]
    fn dense_row_reuse_batch_dispatch_preserves_cancellation_bits() {
        let in_dim = 3;
        let out_dim = 2;
        let batch = 3;
        let weights = [
            1.0f32,
            -1.0,
            f32::from_bits(0x3380_0000),
            f32::from_bits(0xb380_0000),
            -1.0,
            1.0,
        ];
        let bias = [0.25f32, -0.5];
        let inputs = [1.0f32, 1.0, 1.0, 1.0, -1.0, 0.5, 0.0, 0.0, 0.0];
        let prepared = DensePreparedMatrix::prepare(&weights, in_dim, out_dim).unwrap();
        let mut batch_sums = vec![0.0f64; batch * out_dim];
        let mut batch_max = vec![0.0f64; batch];
        let mut batched = vec![0.0f32; batch * out_dim];
        let batch_census = controlled_dense_projection_batched(
            &mut batched,
            &inputs,
            &weights,
            Some(&bias),
            batch,
            &prepared,
            &mut batch_sums,
            &mut batch_max,
            Gpt2DenseCanaryMode::CertifiedNative,
        );
        assert_eq!(batch_census.batch_rows, batch);
        assert_eq!(batch_census.lanes, batch * out_dim);
        assert!(batch_census.fallback_zero > 0);

        let mut serial_prepared = prepared.clone();
        for batch_index in 0..batch {
            let mut serial = [0.0f32; 2];
            let serial_census = controlled_dense_projection(
                &mut serial,
                &inputs[batch_index * in_dim..(batch_index + 1) * in_dim],
                &weights,
                Some(&bias),
                &mut serial_prepared,
                Gpt2DenseCanaryMode::CertifiedNative,
            );
            assert_eq!(serial_census.batch_rows, 0);
            assert_dense_bits(
                &batched[batch_index * out_dim..(batch_index + 1) * out_dim],
                &serial,
                "row-reuse batch/serial cancellation",
            );
        }

        let mut exact = vec![0.0f32; batch * out_dim];
        let mut exact_sums = vec![0.0f64; batch * out_dim];
        let mut exact_max = vec![0.0f64; batch];
        let exact_census = controlled_dense_projection_batched(
            &mut exact,
            &inputs,
            &weights,
            Some(&bias),
            batch,
            &prepared,
            &mut exact_sums,
            &mut exact_max,
            Gpt2DenseCanaryMode::Exact,
        );
        assert_eq!(exact_census.batch_rows, batch);
        assert_dense_bits(&batched, &exact, "row-reuse batch/exact cancellation");
    }

    #[test]
    fn dense_census_overflow_is_atomic_and_full_controls_leave_weights_immutable() {
        let mut census = Gpt2DenseCanaryCensus {
            lanes: usize::MAX,
            ..Gpt2DenseCanaryCensus::default()
        };
        let before = census;
        assert_eq!(
            census.merge(Gpt2DenseCanaryCensus {
                lanes: 1,
                ..Gpt2DenseCanaryCensus::default()
            }),
            None
        );
        assert_eq!(census, before, "overflowing census merge was not atomic");
        let mut layer_census = Gpt2DenseLayerCensus {
            c_attn: before,
            ..Gpt2DenseLayerCensus::default()
        };
        let layer_before = layer_census;
        assert_eq!(
            layer_census.merge(Gpt2DenseLayerCensus {
                c_attn: Gpt2DenseCanaryCensus {
                    lanes: 1,
                    ..Gpt2DenseCanaryCensus::default()
                },
                ..Gpt2DenseLayerCensus::default()
            }),
            None
        );
        assert_eq!(
            layer_census, layer_before,
            "overflowing layer census merge was not atomic"
        );

        let model = Gpt2::load(fixture_dir(), None).unwrap();
        assert!(model.dense_control_workspace_for_batch(0).is_none());
        let wte = model.wte.clone();
        let layer_weights: Vec<_> = model
            .layers
            .iter()
            .map(|layer| {
                (
                    layer.c_attn_w.clone(),
                    layer.c_proj_w.clone(),
                    layer.fc_w.clone(),
                    layer.mlp_w.clone(),
                )
            })
            .collect();
        let production_transpose = model.dense_prepared.lm_head_transposed.clone();
        let production_bounds: Vec<Vec<f64>> = model
            .dense_prepared
            .matrices
            .iter()
            .map(|matrix| matrix.sum_abs_weight_upper.clone())
            .chain(std::iter::once(
                model.dense_prepared.lm_head.sum_abs_weight_upper.clone(),
            ))
            .collect();
        let mut workspace = model.dense_control_workspace_for_batch(2).unwrap();
        let mut serial = Gpt2State::new(&model.cfg);
        let _ = model
            .forward_dense_control(
                &mut serial,
                &mut workspace,
                1,
                0,
                Gpt2DenseCanaryMode::CertifiedNative,
            )
            .unwrap();
        let mut states = vec![Gpt2State::new(&model.cfg); 2];
        let _ = model
            .forward_batch_dense_control(
                &mut states,
                &[1, 2],
                &[0, 0],
                &mut workspace,
                Gpt2DenseCanaryMode::CertifiedNative,
            )
            .unwrap();
        let mut production = Gpt2State::new(&model.cfg);
        model.forward(&mut production, 1, 0, &[], &mut |_, _| {});
        let mut production_batch = vec![Gpt2State::new(&model.cfg); 2];
        model.forward_batch(&mut production_batch, &[1, 2], &[0, 0]);
        assert_dense_bits(&model.wte, &wte, "tied embedding weights");
        for (layer, expected) in model.layers.iter().zip(layer_weights) {
            assert_dense_bits(&layer.c_attn_w, &expected.0, "c_attn weights");
            assert_dense_bits(&layer.c_proj_w, &expected.1, "attention c_proj weights");
            assert_dense_bits(&layer.fc_w, &expected.2, "MLP c_fc weights");
            assert_dense_bits(&layer.mlp_w, &expected.3, "MLP c_proj weights");
        }
        assert_dense_bits(
            &model.dense_prepared.lm_head_transposed,
            &production_transpose,
            "production tied-head preparation",
        );
        for (matrix, expected) in model
            .dense_prepared
            .matrices
            .iter()
            .chain(std::iter::once(&model.dense_prepared.lm_head))
            .zip(production_bounds)
        {
            assert_eq!(
                matrix
                    .sum_abs_weight_upper
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                expected
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                "production dense bounds mutated",
            );
        }
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
        assert_eq!(
            model.dense_canary_workspace(),
            None,
            "dense workspace construction uses the checked Option seam"
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

    /// The GPT-2 oracle reports the truthful learned-absolute attention and
    /// dense-v2 execution pair, and both resolve through their registries.
    /// Its positional action is NOT RoPE, so it is a distinct identity
    /// from current `standard-source-attention/2` — reusing that record would be
    /// a false operator identity.
    #[test]
    fn gpt2_oracle_reports_registered_source_execution_pair() {
        use crate::attention::{operator_spec, AttentionOperatorSpec};
        use crate::dense::{self, DenseOperatorSpec};
        use crate::{BatchedTeacher, TeacherOracle};

        let oracle = HuggingFaceGpt2Oracle::load(fixture_dir()).expect("load tiny gpt2 oracle");
        let spec = TeacherOracle::attention_operator_spec(&oracle)
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

        let dense_spec = TeacherOracle::dense_operator_spec(&oracle)
            .expect("gpt2 oracle declares dense provenance");
        assert_eq!(dense_spec, DenseOperatorSpec::gpt2_source_dense());
        assert_eq!(
            <HuggingFaceGpt2Oracle as BatchedTeacher>::dense_operator_spec(&oracle),
            Some(dense_spec.clone())
        );
        assert_eq!(
            dense::operator_spec(&dense_spec.id, dense_spec.version).expect("registered dense"),
            dense_spec
        );
        dense::validate_source_execution_pair(Some(&spec), Some(&dense_spec))
            .expect("current GPT-2 execution pair is registered");
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
    fn dense_operator_spec(&self) -> Option<crate::dense::DenseOperatorSpec> {
        Some(crate::dense::DenseOperatorSpec::gpt2_source_dense())
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
        // #603: capture through the production v2 path — a traced step
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
    fn dense_operator_spec(&self) -> Option<crate::dense::DenseOperatorSpec> {
        <Self as crate::TeacherOracle>::dense_operator_spec(self)
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
