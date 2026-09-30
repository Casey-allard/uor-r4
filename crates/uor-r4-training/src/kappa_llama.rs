//! Curvature-homotopy conversion of a Llama-architecture checkpoint's attention
//! to hyperbolic (Lorentz) attention.
//!
//! This is an offline floating-point training and measurement tool, never a
//! serving path. It loads a Hugging Face Llama checkpoint (for example
//! SmolLM2-135M/360M-Instruct), keeps every checkpoint weight, and replaces
//! only the attention score of each head.
//!
//! Scores for one head, with head width `r`, curvature `kappa = eps^2` and
//! scaled coordinates `a = eps q`, `b = eps k` lifted to the unit hyperboloid
//! `x -> (sqrt(1 + |x|^2), x)` whose geodesic distance is `d`:
//!
//! - `Dot`: `<q, k> / sqrt(r)`, the checkpoint's own score.
//! - `KeyNorm`: `(|b|^2 - d(a, b)^2) / (2 eps^2 sqrt(r))`.
//! - `Intrinsic`: `(d(o, b)^2 - d(a, b)^2) / (2 eps^2 sqrt(r))`, where `o` is
//!   the hyperboloid origin, so `d(o, b)` is the key's hyperbolic radius.
//!
//! Near the origin `d(a, b)^2 = |a - b|^2 + O(eps^4)`, and the polarization
//! identity `2 <q, k> = |k|^2 + |q|^2 - |q - k|^2` makes both curved scores equal
//! `<q, k> / sqrt(r)` minus a per-query constant, which softmax ignores. These
//! are flat-limit reparametrisations: they reproduce the checkpoint only as
//! `kappa -> 0`, where to first order each adds `kappa F(q, k) / sqrt(r)` with
//! the fixed quartic feature `F = (|q|^2 - |k|^2)^2 / 8 + |q - k|^4 / 24`
//! (minus `|k|^4 / 6` for `Intrinsic`). The `*Linear` kinds are exactly that
//! first-order model; they give the flat-limit curvature gradient in one
//! backward pass and serve as the matched-capacity control for curved runs.
//! The dimensionless curvature of a head is `t = kappa * mean |k|^2`; a head
//! is genuinely hyperbolic only near `t ~ 1`. An exact start tends to stay in
//! the flat basin, so curved runs can anneal curvature upward. RoPE rotates
//! spatial coordinates only, which is an isometry of the hyperboloid, so the
//! curved scores stay relative-position scores. Per-head learnable curvature
//! with an exact Euclidean limit is known (FPS-T, arXiv 2309.04082; kappa-GCN,
//! 1911.05076), as is RoPE as a spatial Lorentz rotation (HELM, 2505.24722).
//!
//! Every head also carries a learnable log inverse temperature (`log_beta`), in
//! every score kind, so a `Dot` control has the same non-curvature freedom.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use candle_core::{Device, Tensor, Var, D};
use safetensors::{Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{invalid, Result};

pub use crate::flock::{
    flock_row_weights, flock_select, rank_table, raw_rank_weights, softmax_over_support,
    FlockEntry, FlockScan, FlockSelect, FlockSelection, FlockSlot, FlockSpec, FlockStats,
    FlockWeights, FLOCK_SELECTOR_VERSION,
};

/// Name of the per-head log curvature scale (`kappa = exp(2 log_eps)`).
pub const LOG_EPS: &str = "curvature.log_eps";
/// Name of the per-head log inverse temperature.
pub const LOG_BETA: &str = "curvature.log_beta";
/// Name of the per-head first-order curvature coefficient of the `*Linear`
/// kinds, in units of the dimensionless curvature `t`.
pub const LAMBDA: &str = "curvature.lambda";
/// Frozen per-head `mean |k|^2` that converts `LAMBDA` (in `t`) to `kappa`.
pub const NORM_SCALE: &str = "curvature.norm_scale";
/// Below this argument the squared arcosh uses its four-term series (relative
/// truncation error below 5e-8 here), which has finite gradients at zero; above
/// it the closed form's f32 rounding error is below 1e-6 relative.
pub const SERIES_LIMIT: f32 = 0.05;

/// Attention score used by every converted head.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreKind {
    Dot,
    KeyNorm,
    Intrinsic,
    /// First-order (in `kappa`) expansion of `KeyNorm` around the flat limit.
    KeyNormLinear,
    /// First-order (in `kappa`) expansion of `Intrinsic` around the flat limit.
    IntrinsicLinear,
}

impl ScoreKind {
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "dot" => Ok(Self::Dot),
            "key_norm" => Ok(Self::KeyNorm),
            "intrinsic" => Ok(Self::Intrinsic),
            "key_norm_linear" => Ok(Self::KeyNormLinear),
            "intrinsic_linear" => Ok(Self::IntrinsicLinear),
            _ => Err(invalid(format!(
                "score must be dot, key_norm, intrinsic, key_norm_linear or intrinsic_linear, not {text}"
            ))),
        }
    }

    /// Hyperbolic scores parametrized by `log_eps`.
    pub fn is_curved(self) -> bool {
        matches!(self, Self::KeyNorm | Self::Intrinsic)
    }

    /// First-order scores parametrized by `LAMBDA`.
    pub fn is_linear(self) -> bool {
        matches!(self, Self::KeyNormLinear | Self::IntrinsicLinear)
    }
}

/// Flock attention probabilities `(batch, heads, time, time)`, training-free.
///
/// Keys are lifted to the Lorentz model as `x_L = (sqrt(1 + |x|^2), x)` and
/// ranked by the Minkowski product `-q0 k0 + q . k`, which is monotone
/// (`-cosh d`); **no `arcosh` is evaluated**. The `sqrt` is the ordinary f32
/// tensor square root. The support and both weight arms come from the shared
/// pure selector in [`crate::flock`]; this adapter only lifts the rows and
/// writes the dense probability tensor.
pub fn flock_probabilities(
    query: &Tensor,
    key: &Tensor,
    spec: FlockSpec,
    stats: &mut FlockStats,
) -> Result<Tensor> {
    if spec.k == 0 {
        return Err(invalid("flock k must be positive"));
    }
    let (batch, heads, time, head_dim) = query.dims4()?;
    if key.dims4()? != (batch, heads, time, head_dim) {
        return Err(invalid("flock query and key shapes differ"));
    }
    let key_t = key.transpose(2, 3)?.contiguous()?;
    let dot = query.matmul(&key_t)?;
    let q0 = query.sqr()?.sum_keepdim(3)?.affine(1.0, 1.0)?.sqrt()?;
    let k0 = key.sqr()?.sum_keepdim(3)?.affine(1.0, 1.0)?.sqrt()?;
    let k0_t = k0.transpose(2, 3)?.contiguous()?;
    // -q0 k0 + q . k
    let lorentz = dot.sub(&q0.matmul(&k0_t)?)?;
    // Arm S scores: the checkpoint's scaled dot product. The learned per-head
    // log_beta (applied on the dense path below) is deliberately not applied
    // here, so this equals the checkpoint's own softmax only when log_beta == 0.
    let scaled = dot.affine(1.0 / (head_dim as f64).sqrt(), 0.0)?;
    let cpu = Device::Cpu;
    let lorentz = lorentz.to_device(&cpu)?.flatten_all()?.to_vec1::<f32>()?;
    let scaled = scaled.to_device(&cpu)?.flatten_all()?.to_vec1::<f32>()?;
    let mut probability = vec![0f32; batch * heads * time * time];
    for row in 0..batch * heads {
        let matrix = row * time * time;
        for position in 0..time {
            let base = matrix + position * time;
            let rank_row = &lorentz[base..base + position + 1];
            let model_row = &scaled[base..base + position + 1];
            let weighted = flock_row_weights(rank_row, model_row, position, &spec)?;
            stats.record(&weighted.selection.scan);
            probability[base..base + position + 1].copy_from_slice(&weighted.weights);
        }
    }
    Ok(Tensor::from_vec(
        probability,
        (batch, heads, time, time),
        query.device(),
    )?)
}

/// Counter-based window sampler (SplitMix64), shared by the probe and the G1a
/// exactness test so both draw identical held-out windows.
pub struct WindowSampler(u64);

impl WindowSampler {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// `count` counter-seeded window starts inside the final `held_out` tokens.
/// Held-out windows are drawn at runtime and never tuned on.
pub fn held_out_starts(
    len: usize,
    time: usize,
    count: usize,
    held_out: usize,
    seed: u64,
) -> Result<Vec<usize>> {
    if count == 0 {
        return Err(invalid("window count must be positive"));
    }
    if held_out > len || held_out < time + 2 {
        return Err(invalid(format!(
            "held-out region of {held_out} tokens cannot hold windows of {time} from {len} tokens"
        )));
    }
    let origin = len - held_out;
    let span = held_out - time - 1;
    let mut sampler = WindowSampler::new(seed);
    Ok((0..count)
        .map(|_| origin + (sampler.next() % span as u64) as usize)
        .collect())
}

/// Where a weight map reads its input (see [`KappaLlama::forward_with_capture`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Site {
    /// Input of `q_proj`, `k_proj` and `v_proj` in a layer.
    Attention(usize),
    /// Input of `o_proj`.
    Output(usize),
    /// Input of `gate_proj` and `up_proj`.
    Mlp(usize),
    /// Input of `down_proj`.
    Down(usize),
    /// Input of the output head.
    Head,
}

/// Curvature parameters of one layer, each `(1, heads, 1, 1)`.
pub enum LayerCurvature<'a> {
    Flat,
    LogEps(&'a Tensor),
    /// First-order coefficient `kappa` per head.
    Linear(&'a Tensor),
}

/// Architecture of a Hugging Face Llama checkpoint, as far as this tool uses it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LlamaShape {
    pub vocab: usize,
    pub width: usize,
    pub layers: usize,
    pub heads: usize,
    pub kv_heads: usize,
    pub head_dim: usize,
    pub ffn: usize,
    pub rope_theta: f64,
    pub rms_eps: f64,
    pub tied_embeddings: bool,
}

fn config_usize(config: &Value, key: &str) -> Result<usize> {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| invalid(format!("config.json needs a non-negative integer {key}")))
}

fn config_f64(config: &Value, key: &str, default: f64) -> Result<f64> {
    match config.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(value) => value
            .as_f64()
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or_else(|| invalid(format!("config.json {key} must be a positive number"))),
    }
}

fn config_flag(config: &Value, key: &str) -> Result<bool> {
    match config.get(key) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(flag)) => Ok(*flag),
        Some(_) => Err(invalid(format!("config.json {key} must be a boolean"))),
    }
}

impl LlamaShape {
    /// Parse and validate a Hugging Face `config.json`. Features this tool does
    /// not implement (biases, RoPE scaling, interleaved RoPE, other
    /// activations) are refused rather than silently ignored.
    pub fn from_config(config: &Value) -> Result<Self> {
        let heads = config_usize(config, "num_attention_heads")?;
        let width = config_usize(config, "hidden_size")?;
        let kv_heads = match config.get("num_key_value_heads") {
            None | Some(Value::Null) => heads,
            Some(_) => config_usize(config, "num_key_value_heads")?,
        };
        if config.get("hidden_act").and_then(Value::as_str) != Some("silu") {
            return Err(invalid("only silu Llama checkpoints are supported"));
        }
        for key in ["attention_bias", "mlp_bias", "rope_interleaved"] {
            if config_flag(config, key)? {
                return Err(invalid(format!("{key}=true is not supported")));
            }
        }
        if !matches!(config.get("rope_scaling"), None | Some(Value::Null)) {
            return Err(invalid("rope_scaling is not supported"));
        }
        let shape = Self {
            vocab: config_usize(config, "vocab_size")?,
            width,
            layers: config_usize(config, "num_hidden_layers")?,
            heads,
            kv_heads,
            head_dim: width.checked_div(heads).unwrap_or(0),
            ffn: config_usize(config, "intermediate_size")?,
            rope_theta: config_f64(config, "rope_theta", 10_000.0)?,
            rms_eps: config_f64(config, "rms_norm_eps", 1e-6)?,
            tied_embeddings: config_flag(config, "tie_word_embeddings")?,
        };
        shape.validate()?;
        Ok(shape)
    }

    pub fn validate(&self) -> Result<()> {
        let positive = [
            self.vocab,
            self.width,
            self.layers,
            self.heads,
            self.kv_heads,
            self.head_dim,
            self.ffn,
        ];
        if positive.contains(&0)
            || self.width != self.heads * self.head_dim
            || !self.heads.is_multiple_of(self.kv_heads)
            || !self.head_dim.is_multiple_of(2)
            || self.vocab > 1 << 20
            || self.width > 1 << 14
            || self.layers > 256
        {
            return Err(invalid("inconsistent or unsupported Llama shape"));
        }
        Ok(())
    }

    /// Every checkpoint tensor this tool requires, with its row-major shape.
    pub fn expected_tensors(&self) -> BTreeMap<String, Vec<usize>> {
        let q_rows = self.heads * self.head_dim;
        let kv_rows = self.kv_heads * self.head_dim;
        let mut shapes = BTreeMap::from([
            (
                "model.embed_tokens.weight".to_owned(),
                vec![self.vocab, self.width],
            ),
            ("model.norm.weight".to_owned(), vec![self.width]),
        ]);
        if !self.tied_embeddings {
            shapes.insert("lm_head.weight".to_owned(), vec![self.vocab, self.width]);
        }
        for layer in 0..self.layers {
            let prefix = format!("model.layers.{layer}");
            for name in ["input_layernorm", "post_attention_layernorm"] {
                shapes.insert(format!("{prefix}.{name}.weight"), vec![self.width]);
            }
            let attention = [
                ("q_proj", vec![q_rows, self.width]),
                ("k_proj", vec![kv_rows, self.width]),
                ("v_proj", vec![kv_rows, self.width]),
                ("o_proj", vec![self.width, q_rows]),
            ];
            for (name, shape) in attention {
                shapes.insert(format!("{prefix}.self_attn.{name}.weight"), shape);
            }
            for name in ["gate_proj", "up_proj"] {
                shapes.insert(
                    format!("{prefix}.mlp.{name}.weight"),
                    vec![self.ffn, self.width],
                );
            }
            shapes.insert(
                format!("{prefix}.mlp.down_proj.weight"),
                vec![self.width, self.ffn],
            );
        }
        shapes
    }
}

fn decode_values(name: &str, dtype: SafeDtype, bytes: &[u8]) -> Result<Vec<f32>> {
    let values: Vec<f32> = match dtype {
        SafeDtype::F32 => bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect(),
        // bfloat16 is the upper half of an IEEE f32.
        SafeDtype::BF16 => bytes
            .chunks_exact(2)
            .map(|b| f32::from_bits(u32::from(u16::from_le_bytes([b[0], b[1]])) << 16))
            .collect(),
        other => {
            return Err(invalid(format!(
                "{name}: dtype {other:?} is not supported (use F32 or BF16)"
            )))
        }
    };
    if values.iter().any(|value| !value.is_finite()) {
        return Err(invalid(format!("{name}: nonfinite coefficient")));
    }
    Ok(values)
}

/// A checkpoint read into f32 tensors, with its shape and weights-file digest.
/// Cloning shares the tensor storage.
#[derive(Clone)]
pub struct Checkpoint {
    pub shape: LlamaShape,
    pub tensors: BTreeMap<String, Tensor>,
    pub weights_sha256: String,
}

/// Safe in-memory safetensors deserialization of `DIR/config.json` and
/// `DIR/model.safetensors`. No pickle, mmap, network or unsafe code. The tensor
/// inventory must equal the expected Llama inventory exactly.
pub fn load_checkpoint(directory: &Path, device: &Device) -> Result<Checkpoint> {
    let config: Value = serde_json::from_slice(&fs::read(directory.join("config.json"))?)?;
    let shape = LlamaShape::from_config(&config)?;
    let weights_path = directory.join("model.safetensors");
    let weights_sha256 = crate::sha256_file(&weights_path)?;
    let bytes = fs::read(&weights_path)?;
    let file = SafeTensors::deserialize(&bytes)?;
    let expected = shape.expected_tensors();
    let names: BTreeSet<String> = file.names().into_iter().map(str::to_owned).collect();
    let wanted: BTreeSet<String> = expected.keys().cloned().collect();
    if names != wanted {
        let missing: Vec<_> = wanted.difference(&names).take(4).collect();
        let extra: Vec<_> = names.difference(&wanted).take(4).collect();
        return Err(invalid(format!(
            "tensor inventory differs from the Llama shape: missing {missing:?}, unexpected {extra:?}"
        )));
    }
    let mut tensors = BTreeMap::new();
    for (name, dims) in expected {
        let view = file.tensor(&name)?;
        if view.shape() != dims.as_slice() {
            return Err(invalid(format!(
                "{name}: shape {:?}, expected {dims:?}",
                view.shape()
            )));
        }
        let values = decode_values(&name, view.dtype(), view.data())?;
        tensors.insert(name, Tensor::from_vec(values, dims.as_slice(), device)?);
    }
    Ok(Checkpoint {
        shape,
        tensors,
        weights_sha256,
    })
}

/// Squared hyperbolic distance `arcosh(1 + z)^2` for `z >= 0` (negative inputs,
/// which only arise from rounding, are clamped). The series
/// `2z - z^2/3 + 4z^3/45 - z^4/35` (from inverting `z = cosh(d) - 1`) is used
/// below [`SERIES_LIMIT`]; the closed form's argument is clamped at the limit
/// from below and the series argument from above, so neither unused branch
/// produces a nonfinite gradient.
pub fn arcosh1p_squared(z: &Tensor) -> Result<Tensor> {
    let z = z.clamp(0f32, f32::MAX)?;
    let small = z.le(SERIES_LIMIT)?;
    let wide = z.clamp(SERIES_LIMIT, f32::MAX)?;
    let root = wide.mul(&wide.affine(1.0, 2.0)?)?.sqrt()?;
    let exact = wide.affine(1.0, 1.0)?.add(&root)?.log()?.sqr()?;
    let near = z.clamp(0f32, SERIES_LIMIT)?;
    let cubic = near.affine(-1.0 / 35.0, 4.0 / 45.0)?;
    let quadratic = near.mul(&cubic)?.affine(1.0, -1.0 / 3.0)?;
    let series = near.mul(&near.mul(&quadratic)?.affine(1.0, 2.0)?)?;
    Ok(small.where_cond(&series, &exact)?)
}

/// Scores for one layer. `query` and `key` are post-RoPE `(batch, heads, time,
/// head_dim)`; the curvature parameters and `log_beta` are `(1, heads, 1, 1)`.
/// Returns `(batch, heads, time, time)` before the causal mask.
pub fn head_scores(
    kind: ScoreKind,
    query: &Tensor,
    key: &Tensor,
    curvature: LayerCurvature<'_>,
    log_beta: &Tensor,
) -> Result<Tensor> {
    let head_dim = query.dim(D::Minus1)? as f64;
    let beta = log_beta.exp()?;
    if kind == ScoreKind::Dot || kind.is_linear() {
        let key_t = key.transpose(2, 3)?.contiguous()?;
        let dot = query.matmul(&key_t)?;
        let mut scores = dot.affine(1.0 / head_dim.sqrt(), 0.0)?;
        if kind.is_linear() {
            let LayerCurvature::Linear(kappa) = curvature else {
                return Err(invalid("a first-order score needs its coefficient"));
            };
            let q2 = query.sqr()?.sum_keepdim(3)?;
            let k2 = key.sqr()?.sum_keepdim(3)?.transpose(2, 3)?.contiguous()?;
            let distance_sq = dot
                .affine(-2.0, 0.0)?
                .broadcast_add(&q2)?
                .broadcast_add(&k2)?;
            let mut feature = q2
                .broadcast_sub(&k2)?
                .sqr()?
                .affine(1.0 / 8.0, 0.0)?
                .add(&distance_sq.sqr()?.affine(1.0 / 24.0, 0.0)?)?;
            if kind == ScoreKind::IntrinsicLinear {
                feature = feature.broadcast_sub(&k2.sqr()?.affine(1.0 / 6.0, 0.0)?)?;
            }
            scores = scores.add(
                &feature
                    .broadcast_mul(kappa)?
                    .affine(1.0 / head_dim.sqrt(), 0.0)?,
            )?;
        }
        return Ok(scores.broadcast_mul(&beta)?);
    }
    let LayerCurvature::LogEps(log_eps) = curvature else {
        return Err(invalid("a curved score needs log_eps"));
    };
    let eps = log_eps.exp()?;
    let a = query.broadcast_mul(&eps)?;
    let b = key.broadcast_mul(&eps)?;
    let a2 = a.sqr()?.sum_keepdim(3)?;
    let b2 = b.sqr()?.sum_keepdim(3)?.transpose(2, 3)?.contiguous()?;
    // x0 - 1 = |x|^2 / (1 + sqrt(1 + |x|^2)), free of cancellation near the origin.
    let lift = |s: &Tensor| -> Result<Tensor> {
        Ok(s.div(&s.affine(1.0, 1.0)?.sqrt()?.affine(1.0, 1.0)?)?)
    };
    let xm = lift(&a2)?;
    let ym = lift(&b2)?;
    let ab = a.matmul(&b.transpose(2, 3)?.contiguous()?)?;
    // -<x, y>_L - 1 = (x0 - 1) + (y0 - 1) + (x0 - 1)(y0 - 1) - <a, b>.
    let z = xm
        .broadcast_add(&ym)?
        .broadcast_add(&xm.broadcast_mul(&ym)?)?
        .sub(&ab)?;
    let distance_sq = arcosh1p_squared(&z)?;
    let bias = match kind {
        ScoreKind::KeyNorm => b2,
        ScoreKind::Intrinsic => arcosh1p_squared(&ym)?,
        _ => return Err(invalid("unreachable flat or first-order branch")),
    };
    let denominator = log_eps
        .affine(2.0, 0.0)?
        .exp()?
        .affine(2.0 * head_dim.sqrt(), 0.0)?;
    Ok(bias
        .broadcast_sub(&distance_sq)?
        .broadcast_div(&denominator)?
        .broadcast_mul(&beta)?)
}

/// Half-split RoPE (Hugging Face Llama layout) on `(batch, heads, time, head_dim)`.
pub fn rope(input: &Tensor, cosine: &Tensor, sine: &Tensor) -> Result<Tensor> {
    let half = input.dim(D::Minus1)? / 2;
    let first = input.narrow(3, 0, half)?;
    let second = input.narrow(3, half, half)?;
    let a = first
        .broadcast_mul(cosine)?
        .sub(&second.broadcast_mul(sine)?)?;
    let b = second
        .broadcast_mul(cosine)?
        .add(&first.broadcast_mul(sine)?)?;
    Ok(Tensor::cat(&[&a, &b], 3)?)
}

/// RoPE tables `(1, 1, time, head_dim / 2)` for positions `offset..offset + time`.
pub fn rope_tables(
    theta: f64,
    head_dim: usize,
    offset: usize,
    time: usize,
    device: &Device,
) -> Result<(Tensor, Tensor)> {
    let half = head_dim / 2;
    let mut cosine = Vec::with_capacity(time * half);
    let mut sine = Vec::with_capacity(time * half);
    for position in offset..offset + time {
        for pair in 0..half {
            let inverse = 1.0 / theta.powf((2 * pair) as f64 / head_dim as f64);
            let angle = position as f64 * inverse;
            cosine.push(angle.cos() as f32);
            sine.push(angle.sin() as f32);
        }
    }
    Ok((
        Tensor::from_vec(cosine, (1, 1, time, half), device)?,
        Tensor::from_vec(sine, (1, 1, time, half), device)?,
    ))
}

/// Which checkpoint weights become trainable in addition to the per-head
/// curvature and temperature scalars.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trainable {
    /// Only `log_eps` (curved scores) and `log_beta`.
    Scalars,
    /// Scalars plus every layer's query and key projections.
    QueryKey,
}

impl Trainable {
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "scalars" => Ok(Self::Scalars),
            "query_key" => Ok(Self::QueryKey),
            _ => Err(invalid(format!(
                "trainable must be scalars or query_key, not {text}"
            ))),
        }
    }
}

/// A checkpoint whose attention scores are replaced by [`ScoreKind`].
pub struct KappaLlama {
    shape: LlamaShape,
    score: ScoreKind,
    frozen: BTreeMap<String, Tensor>,
    variables: BTreeMap<String, Var>,
    device: Device,
    flock: Option<FlockSpec>,
    flock_stats: RefCell<FlockStats>,
}

impl KappaLlama {
    /// Wrap loaded checkpoint tensors. Curved scores start at `init_log_eps` in
    /// every head (`kappa = exp(2 init_log_eps)`); every `log_beta` starts at 0.
    pub fn new(
        checkpoint: Checkpoint,
        score: ScoreKind,
        init_log_eps: f32,
        trainable: Trainable,
        device: &Device,
    ) -> Result<Self> {
        let Checkpoint { shape, tensors, .. } = checkpoint;
        shape.validate()?;
        if !init_log_eps.is_finite() || !(-20.0..=5.0).contains(&init_log_eps) {
            return Err(invalid("init_log_eps must be finite and in [-20, 5]"));
        }
        let expected = shape.expected_tensors();
        for (name, dims) in &expected {
            let tensor = tensors
                .get(name)
                .ok_or_else(|| invalid(format!("missing checkpoint tensor {name}")))?;
            if tensor.dims() != dims.as_slice() {
                return Err(invalid(format!("{name}: wrong shape")));
            }
        }
        let mut frozen = BTreeMap::new();
        let mut variables = BTreeMap::new();
        for (name, tensor) in tensors {
            if !expected.contains_key(&name) {
                return Err(invalid(format!("unexpected checkpoint tensor {name}")));
            }
            let projection = name.ends_with("self_attn.q_proj.weight")
                || name.ends_with("self_attn.k_proj.weight");
            if trainable == Trainable::QueryKey && projection {
                variables.insert(name, Var::from_tensor(&tensor.to_device(device)?)?);
            } else {
                frozen.insert(name, tensor.to_device(device)?.detach());
            }
        }
        let grid = (shape.layers, shape.heads);
        variables.insert(
            LOG_BETA.to_owned(),
            Var::from_tensor(&Tensor::zeros(grid, candle_core::DType::F32, device)?)?,
        );
        if score.is_curved() {
            variables.insert(
                LOG_EPS.to_owned(),
                Var::from_tensor(&Tensor::full(init_log_eps, grid, device)?)?,
            );
        }
        if score.is_linear() {
            variables.insert(
                LAMBDA.to_owned(),
                Var::from_tensor(&Tensor::zeros(grid, candle_core::DType::F32, device)?)?,
            );
            frozen.insert(
                NORM_SCALE.to_owned(),
                Tensor::ones(grid, candle_core::DType::F32, device)?,
            );
        }
        Ok(Self {
            shape,
            score,
            frozen,
            variables,
            device: device.clone(),
            flock: None,
            flock_stats: RefCell::new(FlockStats::default()),
        })
    }

    /// Enable (or clear) training-free flock attention on the checkpoint's own
    /// dot scores. The model's weights are never modified.
    pub fn set_flock(&mut self, flock: Option<FlockSpec>) -> Result<()> {
        if let Some(spec) = flock {
            if self.score != ScoreKind::Dot {
                return Err(invalid(
                    "flock attention is defined on the checkpoint's own (dot) scores",
                ));
            }
            if spec.k == 0 || spec.window == 0 {
                return Err(invalid("flock k and window must be positive"));
            }
        }
        self.flock = flock;
        Ok(())
    }

    /// Supports collected since the previous call, for the probe report.
    pub fn take_flock_stats(&self) -> FlockStats {
        self.flock_stats.replace(FlockStats::default())
    }

    pub fn shape(&self) -> &LlamaShape {
        &self.shape
    }

    pub fn score(&self) -> ScoreKind {
        self.score
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Trainable variables by name (for the optimizer and checkpoints).
    pub fn variables(&self) -> &BTreeMap<String, Var> {
        &self.variables
    }

    /// Access a frozen tensor by name.
    pub fn get_tensor(&self, name: &str) -> Option<&Tensor> {
        self.frozen.get(name)
    }

    /// Set or replace a frozen tensor by name.
    pub fn set_tensor(&mut self, name: &str, tensor: Tensor) -> Result<()> {
        if !self.frozen.contains_key(name) {
            return Err(invalid(format!("tensor {name} not found in model")));
        }
        self.frozen.insert(name.to_string(), tensor);
        Ok(())
    }

    fn tensor(&self, name: &str, detached: bool) -> Result<Tensor> {
        if let Some(variable) = self.variables.get(name) {
            let tensor = variable.as_tensor();
            return Ok(if detached {
                tensor.detach()
            } else {
                tensor.clone()
            });
        }
        self.frozen
            .get(name)
            .cloned()
            .ok_or_else(|| invalid(format!("missing tensor {name}")))
    }

    fn linear(&self, input: &Tensor, name: &str, detached: bool) -> Result<Tensor> {
        Ok(input.matmul(&self.tensor(name, detached)?.t()?)?)
    }

    /// RMSNorm before its gain: `x / sqrt(mean(x^2) + eps)`.
    fn rms_unit(&self, input: &Tensor) -> Result<Tensor> {
        // Primitive composition: Candle's fused RMSNorm is inference-only.
        let denominator = input
            .sqr()?
            .mean_keepdim(D::Minus1)?
            .affine(1.0, self.shape.rms_eps)?
            .sqrt()?;
        Ok(input.broadcast_div(&denominator)?)
    }

    fn apply_gain(&self, unit: &Tensor, name: &str) -> Result<Tensor> {
        Ok(unit.broadcast_mul(&self.tensor(name, true)?)?)
    }

    /// Per-head scalars of one layer as `(1, heads, 1, 1)`.
    fn layer_scalars(&self, name: &str, layer: usize, detached: bool) -> Result<Tensor> {
        Ok(self.tensor(name, detached)?.narrow(0, layer, 1)?.reshape((
            1,
            self.shape.heads,
            1,
            1,
        ))?)
    }

    /// Causal logits `(batch, time, vocab)` for `batch` rows of `time` tokens.
    /// `detached` evaluates without recording a backward graph.
    pub fn forward(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        detached: bool,
    ) -> Result<Tensor> {
        self.forward_with_probe(ids, batch, time, detached, &mut |_, _, _, _| Ok(()))
    }

    /// [`forward`](Self::forward) that also presents every layer's post-RoPE
    /// query and (grouped-query repeated) key `(batch, heads, time, head_dim)`
    /// and attention probabilities `(batch, heads, time, time)` to `probe`.
    pub fn forward_with_probe(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        detached: bool,
        probe: &mut dyn FnMut(usize, &Tensor, &Tensor, &Tensor) -> Result<()>,
    ) -> Result<Tensor> {
        self.forward_hooked(ids, batch, time, detached, probe, &mut |_, _| Ok(()))
    }

    /// Detached [`forward`](Self::forward) that presents the input of every
    /// weight map to `capture` as `(batch * time, columns)`: the RMSNorm output
    /// before its gain for the attention, MLP and head sites (the exporter folds
    /// the gains into those weights), the attention output for `o_proj` and the
    /// gated MLP activation for `down_proj`.
    pub fn forward_with_capture(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        capture: &mut dyn FnMut(Site, &Tensor) -> Result<()>,
    ) -> Result<Tensor> {
        self.forward_hooked(ids, batch, time, true, &mut |_, _, _, _| Ok(()), capture)
    }

    fn forward_hooked(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        detached: bool,
        probe: &mut dyn FnMut(usize, &Tensor, &Tensor, &Tensor) -> Result<()>,
        capture: &mut dyn FnMut(Site, &Tensor) -> Result<()>,
    ) -> Result<Tensor> {
        let s = &self.shape;
        if batch == 0
            || time == 0
            || ids.len() != batch * time
            || ids.iter().any(|&token| token as usize >= s.vocab)
        {
            return Err(invalid("forward token bounds"));
        }
        let (cosine, sine) = rope_tables(s.rope_theta, s.head_dim, 0, time, &self.device)?;
        let causal: Vec<u8> = (0..time)
            .flat_map(|row| (0..time).map(move |column| u8::from(column > row)))
            .collect();
        let mask = Tensor::from_vec(causal, (1, 1, time, time), &self.device)?
            .broadcast_as((batch, s.heads, time, time))?;
        let excluded = Tensor::full(
            f32::NEG_INFINITY,
            (batch, s.heads, time, time),
            &self.device,
        )?;
        let ids = Tensor::new(ids, &self.device)?;
        let mut state = self
            .tensor("model.embed_tokens.weight", true)?
            .index_select(&ids, 0)?;
        let group = s.heads / s.kv_heads;
        for layer in 0..s.layers {
            let prefix = format!("model.layers.{layer}");
            let unit = self.rms_unit(&state)?;
            capture(Site::Attention(layer), &unit)?;
            let normalized = self.apply_gain(&unit, &format!("{prefix}.input_layernorm.weight"))?;
            let project = |name: &str, heads: usize| -> Result<Tensor> {
                Ok(self
                    .linear(
                        &normalized,
                        &format!("{prefix}.self_attn.{name}.weight"),
                        detached,
                    )?
                    .reshape((batch, time, heads, s.head_dim))?
                    .transpose(1, 2)?
                    .contiguous()?)
            };
            let repeat = |tensor: Tensor| -> Result<Tensor> {
                if group == 1 {
                    return Ok(tensor);
                }
                Ok(tensor
                    .unsqueeze(2)?
                    .broadcast_as((batch, s.kv_heads, group, time, s.head_dim))?
                    .contiguous()?
                    .reshape((batch, s.heads, time, s.head_dim))?)
            };
            let query = rope(&project("q_proj", s.heads)?, &cosine, &sine)?;
            let key = repeat(rope(&project("k_proj", s.kv_heads)?, &cosine, &sine)?)?;
            let value = repeat(project("v_proj", s.kv_heads)?)?;
            let log_eps: Tensor;
            let kappa: Tensor;
            let curvature = if self.score.is_curved() {
                log_eps = self.layer_scalars(LOG_EPS, layer, detached)?;
                LayerCurvature::LogEps(&log_eps)
            } else if self.score.is_linear() {
                kappa = self
                    .layer_scalars(LAMBDA, layer, detached)?
                    .div(&self.layer_scalars(NORM_SCALE, layer, true)?)?;
                LayerCurvature::Linear(&kappa)
            } else {
                LayerCurvature::Flat
            };
            let log_beta = self.layer_scalars(LOG_BETA, layer, detached)?;
            let probability = if let Some(spec) = self.flock {
                if !detached {
                    return Err(invalid(
                        "flock attention is a detached measurement path (its support is not differentiable)",
                    ));
                }
                let mut stats = self.flock_stats.borrow_mut();
                flock_probabilities(&query, &key, spec, &mut stats)?
            } else {
                let scores = head_scores(self.score, &query, &key, curvature, &log_beta)?;
                let masked = mask.where_cond(&excluded, &scores)?;
                candle_nn::ops::softmax(&masked, 3)?
            };
            probe(layer, &query, &key, &probability)?;
            let attended = probability
                .matmul(&value)?
                .transpose(1, 2)?
                .contiguous()?
                .reshape((batch * time, s.width))?;
            capture(Site::Output(layer), &attended)?;
            state = state.add(&self.linear(
                &attended,
                &format!("{prefix}.self_attn.o_proj.weight"),
                true,
            )?)?;
            let unit = self.rms_unit(&state)?;
            capture(Site::Mlp(layer), &unit)?;
            let normalized =
                self.apply_gain(&unit, &format!("{prefix}.post_attention_layernorm.weight"))?;
            let gate = self
                .linear(&normalized, &format!("{prefix}.mlp.gate_proj.weight"), true)?
                .silu()?;
            let up = self.linear(&normalized, &format!("{prefix}.mlp.up_proj.weight"), true)?;
            let gated = gate.mul(&up)?;
            capture(Site::Down(layer), &gated)?;
            state = state.add(&self.linear(
                &gated,
                &format!("{prefix}.mlp.down_proj.weight"),
                true,
            )?)?;
        }
        let unit = self.rms_unit(&state)?;
        capture(Site::Head, &unit)?;
        let hidden = self.apply_gain(&unit, "model.norm.weight")?;
        let head = if s.tied_embeddings {
            "model.embed_tokens.weight"
        } else {
            "lm_head.weight"
        };
        Ok(self
            .linear(&hidden, head, true)?
            .reshape((batch, time, s.vocab))?)
    }

    /// Curvature `kappa = exp(2 log_eps)` per layer and head (empty for `Dot`).
    pub fn curvature(&self) -> Result<Vec<Vec<f32>>> {
        match self.variables.get(LOG_EPS) {
            None => Ok(Vec::new()),
            Some(log_eps) => Ok(log_eps.as_tensor().affine(2.0, 0.0)?.exp()?.to_vec2()?),
        }
    }

    /// Inverse temperature `exp(log_beta)` per layer and head.
    pub fn temperature(&self) -> Result<Vec<Vec<f32>>> {
        Ok(self.tensor(LOG_BETA, true)?.exp()?.to_vec2()?)
    }

    /// Overwrite every head's `log_eps` (probes and tests).
    pub fn set_log_eps(&self, value: f32) -> Result<()> {
        let variable = self
            .variables
            .get(LOG_EPS)
            .ok_or_else(|| invalid("a dot model has no curvature"))?;
        variable.set(&Tensor::full(value, variable.shape(), &self.device)?)?;
        Ok(())
    }

    fn grid_tensor(&self, grid: &[Vec<f32>]) -> Result<Tensor> {
        let (layers, heads) = (self.shape.layers, self.shape.heads);
        if grid.len() != layers || grid.iter().any(|row| row.len() != heads) {
            return Err(invalid(format!(
                "a per-head grid must be {layers} x {heads}"
            )));
        }
        let flat: Vec<f32> = grid.iter().flatten().copied().collect();
        if flat.iter().any(|value| !value.is_finite()) {
            return Err(invalid("per-head grid values must be finite"));
        }
        Ok(Tensor::from_vec(flat, (layers, heads), &self.device)?)
    }

    /// Overwrite `log_eps` head by head (`grid[layer][head]`).
    pub fn set_log_eps_grid(&self, grid: &[Vec<f32>]) -> Result<()> {
        let variable = self
            .variables
            .get(LOG_EPS)
            .ok_or_else(|| invalid("this score kind has no log_eps"))?;
        variable.set(&self.grid_tensor(grid)?)?;
        Ok(())
    }

    /// Project `log_eps` onto a per-head floor (`floor[layer][head]`).
    pub fn floor_log_eps_grid(&self, floor: &[Vec<f32>]) -> Result<()> {
        let variable = self
            .variables
            .get(LOG_EPS)
            .ok_or_else(|| invalid("this score kind has no log_eps"))?;
        variable.set(&variable.as_tensor().maximum(&self.grid_tensor(floor)?)?)?;
        Ok(())
    }

    /// Set the frozen per-head `mean |k|^2` of a first-order score, so that
    /// `LAMBDA` is measured in the dimensionless curvature `t`.
    pub fn set_norm_scale(&mut self, scale: &[Vec<f32>]) -> Result<()> {
        if !self.score.is_linear() {
            return Err(invalid("only first-order scores have a norm scale"));
        }
        if scale.iter().flatten().any(|value| *value <= 0.0) {
            return Err(invalid("norm scales must be positive"));
        }
        let tensor = self.grid_tensor(scale)?;
        self.frozen.insert(NORM_SCALE.to_owned(), tensor);
        Ok(())
    }

    /// First-order coefficient per layer and head, in units of `t`.
    pub fn first_order_coefficient(&self) -> Result<Vec<Vec<f32>>> {
        match self.variables.get(LAMBDA) {
            None => Ok(Vec::new()),
            Some(lambda) => Ok(lambda.as_tensor().to_vec2()?),
        }
    }

    /// Project every head's `log_eps` onto `[floor, inf)`. An exact flat-limit
    /// start can sit in the flat basin, where learned curvature never grows; a
    /// rising floor anneals the heads into the curved regime while the rest of
    /// the model adapts.
    pub fn floor_log_eps(&self, floor: f32) -> Result<()> {
        if !floor.is_finite() {
            return Err(invalid("curvature floor must be finite"));
        }
        let variable = self
            .variables
            .get(LOG_EPS)
            .ok_or_else(|| invalid("a dot model has no curvature"))?;
        variable.set(&variable.as_tensor().maximum(floor)?)?;
        Ok(())
    }
}

/// Per-head mean of `|k|^2` over batch and time, for keys `(batch, heads, time,
/// head_dim)`. Multiplying a head's curvature by it gives the dimensionless
/// curvature `t`, which is invariant to rescaling that head's keys.
pub fn mean_key_sq(key: &Tensor) -> Result<Vec<f32>> {
    Ok(key.sqr()?.sum(3)?.mean((0, 2))?.to_vec1()?)
}

/// Attention statistics per layer and head (`[layer][head]`).
#[derive(Clone, Debug, Serialize)]
pub struct HeadStatistics {
    /// Mean `|k|^2`: the unit of the dimensionless curvature `t = kappa * mean |k|^2`.
    pub mean_key_sq: Vec<Vec<f32>>,
    /// Median `|k|^2`.
    pub median_key_sq: Vec<Vec<f32>>,
    /// Mean cosine between a query and its highest-weight key.
    pub top1_cosine: Vec<Vec<f32>>,
    /// Mean attention mass held by the top `fractions[i]` of the causal keys,
    /// the ceiling for any index that scores that fraction exactly.
    pub top_mass: Vec<Vec<Vec<f32>>>,
    pub fractions: Vec<f64>,
    /// Queries with fewer causal keys than this are skipped for cosine and mass.
    pub min_candidates: usize,
    /// Query rows averaged per head.
    pub rows: u64,
}

/// Accumulates [`HeadStatistics`] over batches of one model's attention.
pub struct HeadStatisticsAccumulator {
    fractions: Vec<f64>,
    min_candidates: usize,
    key_sq: Vec<Vec<Vec<f32>>>,
    cosine: Vec<Vec<f64>>,
    mass: Vec<Vec<Vec<f64>>>,
    rows: Vec<Vec<u64>>,
}

fn dot64(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| f64::from(*x) * f64::from(*y))
        .sum()
}

impl HeadStatisticsAccumulator {
    pub fn new(shape: &LlamaShape, fractions: &[f64], min_candidates: usize) -> Result<Self> {
        if fractions.is_empty()
            || fractions.iter().any(|f| !(*f > 0.0 && *f <= 1.0))
            || min_candidates == 0
        {
            return Err(invalid(
                "fractions must lie in (0, 1] and min_candidates must be positive",
            ));
        }
        Ok(Self {
            fractions: fractions.to_vec(),
            min_candidates,
            key_sq: vec![vec![Vec::new(); shape.heads]; shape.layers],
            cosine: vec![vec![0.0; shape.heads]; shape.layers],
            mass: vec![vec![vec![0.0; fractions.len()]; shape.heads]; shape.layers],
            rows: vec![vec![0; shape.heads]; shape.layers],
        })
    }

    /// Run `model` on one batch without a backward graph, accumulate every
    /// head, and return the batch's logits `(batch, time, vocab)`.
    pub fn add(
        &mut self,
        model: &KappaLlama,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        let cpu = Device::Cpu;
        let mut probe =
            |layer: usize, query: &Tensor, key: &Tensor, probability: &Tensor| -> Result<()> {
                let (b_len, h_len, t_len, width) = key.dims4()?;
                let q: Vec<f32> = query.to_device(&cpu)?.flatten_all()?.to_vec1()?;
                let k: Vec<f32> = key.to_device(&cpu)?.flatten_all()?.to_vec1()?;
                let p: Vec<f32> = probability.to_device(&cpu)?.flatten_all()?.to_vec1()?;
                let mut row = Vec::with_capacity(t_len);
                for b in 0..b_len {
                    for h in 0..h_len {
                        let base = (b * h_len + h) * t_len;
                        for t in 0..t_len {
                            let kt = &k[(base + t) * width..(base + t + 1) * width];
                            self.key_sq[layer][h].push(dot64(kt, kt) as f32);
                        }
                        for t in self.min_candidates.saturating_sub(1)..t_len {
                            let start = (base + t) * t_len;
                            let weights = &p[start..start + t + 1];
                            let mut top = 0;
                            for (i, w) in weights.iter().enumerate() {
                                if *w > weights[top] {
                                    top = i;
                                }
                            }
                            let qt = &q[(base + t) * width..(base + t + 1) * width];
                            let kt = &k[(base + top) * width..(base + top + 1) * width];
                            let norms = (dot64(qt, qt) * dot64(kt, kt)).sqrt();
                            if norms > 0.0 {
                                self.cosine[layer][h] += dot64(qt, kt) / norms;
                            }
                            row.clear();
                            row.extend_from_slice(weights);
                            row.sort_by(|a, b| b.total_cmp(a));
                            for (i, fraction) in self.fractions.iter().enumerate() {
                                let keep =
                                    ((fraction * (t + 1) as f64).ceil() as usize).clamp(1, t + 1);
                                self.mass[layer][h][i] +=
                                    row[..keep].iter().map(|w| f64::from(*w)).sum::<f64>();
                            }
                            self.rows[layer][h] += 1;
                        }
                    }
                }
                Ok(())
            };
        model.forward_with_probe(ids, batch, time, true, &mut probe)
    }

    pub fn finish(mut self) -> Result<HeadStatistics> {
        let rows = self.rows.iter().flatten().copied().min().unwrap_or(0);
        if rows == 0 || self.key_sq.iter().flatten().any(Vec::is_empty) {
            return Err(invalid(
                "no query had enough causal keys for head statistics",
            ));
        }
        let mut mean_key_sq = Vec::with_capacity(self.key_sq.len());
        let mut median_key_sq = Vec::with_capacity(self.key_sq.len());
        for layer in &mut self.key_sq {
            let mut means = Vec::with_capacity(layer.len());
            let mut medians = Vec::with_capacity(layer.len());
            for values in layer.iter_mut() {
                let sum: f64 = values.iter().map(|v| f64::from(*v)).sum();
                means.push((sum / values.len() as f64) as f32);
                values.sort_by(f32::total_cmp);
                medians.push(values[values.len() / 2]);
            }
            mean_key_sq.push(means);
            median_key_sq.push(medians);
        }
        let mut top1_cosine = Vec::with_capacity(self.cosine.len());
        let mut top_mass = Vec::with_capacity(self.mass.len());
        for ((cosines, masses), counts) in self.cosine.iter().zip(&self.mass).zip(&self.rows) {
            top1_cosine.push(
                cosines
                    .iter()
                    .zip(counts)
                    .map(|(sum, n)| (sum / *n as f64) as f32)
                    .collect(),
            );
            top_mass.push(
                masses
                    .iter()
                    .zip(counts)
                    .map(|(sums, n)| sums.iter().map(|sum| (sum / *n as f64) as f32).collect())
                    .collect(),
            );
        }
        Ok(HeadStatistics {
            mean_key_sq,
            median_key_sq,
            top1_cosine,
            top_mass,
            fractions: self.fractions,
            min_candidates: self.min_candidates,
            rows,
        })
    }
}

/// Mean next-token negative log-likelihood (nats) of `(batch, time, vocab)`
/// logits against `targets` of length `batch * time`.
pub fn next_token_nll(logits: &Tensor, targets: &[u32]) -> Result<Tensor> {
    let (batch, time, vocab) = logits.dims3()?;
    if targets.len() != batch * time || targets.iter().any(|&t| t as usize >= vocab) {
        return Err(invalid("target bounds"));
    }
    let rows = logits.reshape((batch * time, vocab))?;
    let targets = Tensor::new(targets, logits.device())?.unsqueeze(1)?;
    Ok(candle_nn::ops::log_softmax(&rows, 1)?
        .gather(&targets, 1)?
        .mean_all()?
        .neg()?)
}

/// Mean per-position `KL(teacher || student)` in nats. The teacher is detached.
pub fn distillation_kl(student: &Tensor, teacher: &Tensor) -> Result<Tensor> {
    if student.dims() != teacher.dims() {
        return Err(invalid("student and teacher logits differ in shape"));
    }
    let vocab = student.dim(D::Minus1)?;
    let rows = student.elem_count() / vocab;
    let teacher_log = candle_nn::ops::log_softmax(&teacher.detach().reshape((rows, vocab))?, 1)?;
    let student_log = candle_nn::ops::log_softmax(&student.reshape((rows, vocab))?, 1)?;
    Ok(teacher_log
        .exp()?
        .mul(&teacher_log.sub(&student_log)?)?
        .sum(1)?
        .mean_all()?)
}

/// Largest absolute logit difference between two equally shaped logit tensors.
pub fn max_abs_difference(a: &Tensor, b: &Tensor) -> Result<f32> {
    Ok(a.sub(b)?.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::backprop::GradStore;

    fn tiny_shape() -> LlamaShape {
        LlamaShape {
            vocab: 40,
            width: 32,
            layers: 2,
            heads: 4,
            kv_heads: 2,
            head_dim: 8,
            ffn: 48,
            rope_theta: 10_000.0,
            rms_eps: 1e-5,
            tied_embeddings: true,
        }
    }

    /// Deterministic pseudo-random checkpoint with trained-like magnitudes.
    fn tiny_checkpoint(device: &Device) -> Checkpoint {
        let shape = tiny_shape();
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        let mut tensors = BTreeMap::new();
        for (name, dims) in shape.expected_tensors() {
            let count: usize = dims.iter().product();
            let norm = name.ends_with("norm.weight");
            let scale = if name.contains("q_proj") || name.contains("k_proj") {
                0.6
            } else {
                0.25
            };
            let values: Vec<f32> = (0..count)
                .map(|_| {
                    seed = seed
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1_442_695_040_888_963_407);
                    let unit = ((seed >> 40) as f32) / ((1u64 << 24) as f32) - 0.5;
                    if norm {
                        1.0 + 0.2 * unit
                    } else {
                        scale * unit
                    }
                })
                .collect();
            tensors.insert(
                name,
                Tensor::from_vec(values, dims.as_slice(), device).expect("tensor"),
            );
        }
        Checkpoint {
            shape,
            tensors,
            weights_sha256: String::new(),
        }
    }

    fn ids(batch: usize, time: usize) -> Vec<u32> {
        (0..batch * time)
            .map(|i| ((i * 7 + 3) % 40) as u32)
            .collect()
    }

    fn model(score: ScoreKind, log_eps: f32, trainable: Trainable) -> KappaLlama {
        let device = Device::Cpu;
        KappaLlama::new(tiny_checkpoint(&device), score, log_eps, trainable, &device)
            .expect("model")
    }

    #[test]
    fn config_parsing_accepts_smollm2_and_refuses_unsupported_features() {
        let smollm2: Value = serde_json::json!({
            "hidden_act": "silu", "hidden_size": 576, "intermediate_size": 1536,
            "num_attention_heads": 9, "num_hidden_layers": 30, "num_key_value_heads": 3,
            "rms_norm_eps": 1e-5, "rope_theta": 100000, "tie_word_embeddings": true,
            "vocab_size": 49152, "attention_bias": false, "mlp_bias": false,
            "rope_interleaved": false, "rope_scaling": null
        });
        let shape = LlamaShape::from_config(&smollm2).expect("SmolLM2-135M shape");
        assert_eq!((shape.head_dim, shape.kv_heads), (64, 3));
        let params: usize = shape
            .expected_tensors()
            .values()
            .map(|dims| dims.iter().product::<usize>())
            .sum();
        assert_eq!(params, 134_515_008);
        let mut biased = smollm2.clone();
        biased["attention_bias"] = Value::Bool(true);
        assert!(LlamaShape::from_config(&biased).is_err());
        let mut scaled = smollm2;
        scaled["rope_scaling"] = serde_json::json!({"type": "linear", "factor": 2.0});
        assert!(LlamaShape::from_config(&scaled).is_err());
    }

    #[test]
    fn arcosh_series_and_closed_form_agree_at_the_switch() {
        let device = Device::Cpu;
        let points = [0.0f32, 1e-6, 1e-3, 0.0499, 0.0501, 0.5, 3.0];
        let z = Tensor::new(&points, &device).expect("z");
        let got: Vec<f32> = arcosh1p_squared(&z)
            .expect("arcosh")
            .to_vec1()
            .expect("vec");
        for (value, zz) in got.iter().zip(points) {
            let want = (1.0 + f64::from(zz)).acosh().powi(2);
            assert!(
                (f64::from(*value) - want).abs() <= 2e-6 * want.max(1e-6),
                "z={zz}: {value} vs {want}"
            );
        }
    }

    #[test]
    fn curved_scores_converge_to_dot_attention_as_curvature_vanishes() {
        let dot = model(ScoreKind::Dot, 0.0, Trainable::Scalars);
        let reference = dot.forward(&ids(2, 12), 2, 12, true).expect("dot");
        for score in [ScoreKind::KeyNorm, ScoreKind::Intrinsic] {
            let curved = model(score, -8.0, Trainable::Scalars);
            let mut errors = Vec::new();
            for log_eps in [-8.0f32, -6.0, -4.0, -1.0] {
                curved.set_log_eps(log_eps).expect("set");
                let logits = curved.forward(&ids(2, 12), 2, 12, true).expect("curved");
                errors.push(max_abs_difference(&logits, &reference).expect("diff"));
            }
            assert!(errors[0] < 2e-4, "{score:?} flat limit: {errors:?}");
            assert!(errors[1] < 2e-3, "{score:?}: {errors:?}");
            assert!(
                errors[3] > 10.0 * errors[1],
                "{score:?} curvature must matter: {errors:?}"
            );
        }
    }

    #[test]
    fn curved_scores_depend_on_relative_rope_position_only() {
        let device = Device::Cpu;
        let values: Vec<f32> = (0..2 * 3 * 8)
            .map(|i| ((i * 37 % 23) as f32 - 11.0) / 4.0)
            .collect();
        let base = Tensor::from_vec(values, (1, 2, 3, 8), &device).expect("base");
        let log_eps = Tensor::full(-0.5f32, (1, 2, 1, 1), &device).expect("eps");
        let log_beta = Tensor::zeros((1, 2, 1, 1), candle_core::DType::F32, &device).expect("beta");
        for kind in [ScoreKind::KeyNorm, ScoreKind::Intrinsic] {
            let mut rows = Vec::new();
            for offset in [0usize, 17] {
                let (cosine, sine) = rope_tables(1e4, 8, offset, 3, &device).expect("rope");
                let q = rope(&base, &cosine, &sine).expect("q");
                let k = rope(&base.affine(0.5, 0.25).expect("k"), &cosine, &sine).expect("k");
                let scores = head_scores(kind, &q, &k, LayerCurvature::LogEps(&log_eps), &log_beta)
                    .expect("scores");
                rows.push(scores);
            }
            let shift = max_abs_difference(&rows[0], &rows[1]).expect("diff");
            assert!(
                shift < 1e-4,
                "{kind:?}: a common RoPE offset changed scores by {shift}"
            );
        }
    }

    #[test]
    fn language_loss_reaches_curvature_and_query_key_weights() {
        let curved = model(ScoreKind::Intrinsic, -1.0, Trainable::QueryKey);
        let tokens = ids(2, 13);
        let inputs: Vec<u32> = tokens
            .chunks(13)
            .flat_map(|row| row[..12].to_vec())
            .collect();
        let targets: Vec<u32> = tokens
            .chunks(13)
            .flat_map(|row| row[1..].to_vec())
            .collect();
        let logits = curved.forward(&inputs, 2, 12, false).expect("forward");
        let loss = next_token_nll(&logits, &targets).expect("loss");
        let grads: GradStore = loss.backward().expect("backward");
        for name in [LOG_EPS, LOG_BETA, "model.layers.1.self_attn.k_proj.weight"] {
            let variable = &curved.variables()[name];
            let grad = grads.get(variable.as_tensor()).expect("gradient present");
            let norm = grad
                .sqr()
                .expect("sq")
                .sum_all()
                .expect("sum")
                .to_scalar::<f32>()
                .expect("s");
            assert!(
                norm.is_finite() && norm > 0.0,
                "{name}: gradient norm {norm}"
            );
        }
    }

    #[test]
    fn curvature_floor_projects_every_head() {
        let curved = model(ScoreKind::KeyNorm, -6.0, Trainable::Scalars);
        curved.floor_log_eps(-2.0).expect("floor");
        let kappa = curved.curvature().expect("kappa");
        let want = (-4.0f32).exp();
        assert!(kappa
            .iter()
            .flatten()
            .all(|k| (k - want).abs() <= 1e-6 * want));
        curved
            .floor_log_eps(-3.0)
            .expect("lower floor keeps higher values");
        assert!(curved
            .curvature()
            .expect("kappa")
            .iter()
            .flatten()
            .all(|k| (k - want).abs() <= 1e-6 * want));
        assert!(model(ScoreKind::Dot, 0.0, Trainable::Scalars)
            .floor_log_eps(0.0)
            .is_err());
    }

    #[test]
    fn first_order_scores_match_curved_scores_to_first_order() {
        let device = Device::Cpu;
        let values: Vec<f32> = (0..2 * 3 * 8)
            .map(|i| ((i * 29 % 31) as f32 - 15.0) / 5.0)
            .collect();
        let q = Tensor::from_vec(values.clone(), (1, 2, 3, 8), &device).expect("q");
        let k = q.affine(-0.7, 0.3).expect("k");
        let beta = Tensor::zeros((1, 2, 1, 1), candle_core::DType::F32, &device).expect("beta");
        let dot = head_scores(ScoreKind::Dot, &q, &k, LayerCurvature::Flat, &beta).expect("dot");
        let q2 = q.sqr().expect("sq").sum_keepdim(3).expect("sum");
        for (curved, linear) in [
            (ScoreKind::KeyNorm, ScoreKind::KeyNormLinear),
            (ScoreKind::Intrinsic, ScoreKind::IntrinsicLinear),
        ] {
            let kappa = 1e-4f32;
            let log_eps = Tensor::full(0.5 * kappa.ln(), (1, 2, 1, 1), &device).expect("eps");
            let coefficient = Tensor::full(kappa, (1, 2, 1, 1), &device).expect("kappa");
            // Curved scores carry the per-query constant -|q|^2 / (2 sqrt r); add it back.
            let shift = q2.affine(0.5 / 8f64.sqrt(), 0.0).expect("shift");
            let curved = head_scores(curved, &q, &k, LayerCurvature::LogEps(&log_eps), &beta)
                .expect("curved")
                .broadcast_add(&shift)
                .expect("shifted");
            let first = head_scores(linear, &q, &k, LayerCurvature::Linear(&coefficient), &beta)
                .expect("first order");
            let correction = max_abs_difference(&first, &dot).expect("correction");
            let residual = max_abs_difference(&curved, &first).expect("residual");
            assert!(
                correction > 1e-3,
                "{linear:?}: the first-order term must matter: {correction}"
            );
            assert!(
                residual < 0.05 * correction,
                "{linear:?}: residual {residual} vs {correction}"
            );
        }
    }

    #[test]
    fn head_statistics_are_well_formed() {
        let dot = model(ScoreKind::Dot, 0.0, Trainable::Scalars);
        let mut stats =
            HeadStatisticsAccumulator::new(dot.shape(), &[0.1, 0.5, 1.0], 4).expect("accumulator");
        stats.add(&dot, &ids(2, 12), 2, 12).expect("batch");
        let stats = stats.finish().expect("finish");
        assert_eq!(stats.rows, 2 * 9);
        for layer in 0..2 {
            for head in 0..4 {
                assert!(stats.median_key_sq[layer][head] > 0.0);
                assert!(stats.top1_cosine[layer][head].abs() <= 1.0 + 1e-6);
                let mass = &stats.top_mass[layer][head];
                assert!(mass[0] <= mass[1] + 1e-6 && mass[1] <= mass[2] + 1e-6);
                assert!(
                    (mass[2] - 1.0).abs() < 1e-4,
                    "all keys hold all mass: {mass:?}"
                );
            }
        }
    }

    #[test]
    fn flat_limit_curvature_drive_is_one_backward_pass() {
        let linear = model(ScoreKind::IntrinsicLinear, 0.0, Trainable::Scalars);
        let dot = model(ScoreKind::Dot, 0.0, Trainable::Scalars);
        let tokens = ids(2, 13);
        let inputs: Vec<u32> = tokens
            .chunks(13)
            .flat_map(|row| row[..12].to_vec())
            .collect();
        let targets: Vec<u32> = tokens
            .chunks(13)
            .flat_map(|row| row[1..].to_vec())
            .collect();
        let reference = dot.forward(&inputs, 2, 12, true).expect("dot");
        let logits = linear.forward(&inputs, 2, 12, false).expect("linear");
        assert!(max_abs_difference(&logits, &reference).expect("diff") < 1e-4);
        let lambda = linear.variables()[LAMBDA].as_tensor().clone();
        let nll = next_token_nll(&logits, &targets).expect("nll");
        let drive = nll.backward().expect("backward");
        let g: Vec<Vec<f32>> = drive.get(&lambda).expect("drive").to_vec2().expect("grid");
        assert!(g.iter().flatten().all(|v| v.is_finite()));
        assert!(g.iter().flatten().any(|v| v.abs() > 1e-6), "{g:?}");
        let logits = linear.forward(&inputs, 2, 12, false).expect("linear");
        let self_kl = distillation_kl(&logits, &reference).expect("kl");
        let sanity = self_kl.backward().expect("backward");
        let s: Vec<Vec<f32>> = sanity
            .get(&lambda)
            .expect("sanity")
            .to_vec2()
            .expect("grid");
        assert!(
            s.iter().flatten().all(|v| v.abs() < 1e-6),
            "self-distillation drive {s:?}"
        );
    }

    #[test]
    fn distillation_moves_curvature_toward_the_teacher() {
        let teacher = model(ScoreKind::Dot, 0.0, Trainable::Scalars);
        let student = model(ScoreKind::KeyNorm, 0.0, Trainable::Scalars);
        let batch = ids(2, 12);
        let target = teacher.forward(&batch, 2, 12, true).expect("teacher");
        let kl = |m: &KappaLlama| -> f32 {
            let logits = m.forward(&batch, 2, 12, true).expect("student");
            distillation_kl(&logits, &target)
                .expect("kl")
                .to_scalar()
                .expect("scalar")
        };
        let before = kl(&student);
        let variable = &student.variables()[LOG_EPS];
        for _ in 0..40 {
            let logits = student.forward(&batch, 2, 12, false).expect("forward");
            let loss = distillation_kl(&logits, &target).expect("kl");
            let grads = loss.backward().expect("backward");
            let grad = grads.get(variable.as_tensor()).expect("grad");
            // Sign descent: the flat teacher is reached by lowering curvature.
            let step = grad.sign().expect("sign").affine(-0.1, 0.0).expect("step");
            variable
                .set(&variable.as_tensor().add(&step).expect("add"))
                .expect("set");
        }
        let after = kl(&student);
        assert!(
            before > 1e-4,
            "the curved start must differ from the teacher: {before}"
        );
        assert!(after < 0.1 * before, "KL {before} -> {after}");
    }

    fn flock_model(k: usize, window: usize, weights: FlockWeights) -> KappaLlama {
        let mut model = model(ScoreKind::Dot, 0.0, Trainable::Scalars);
        model
            .set_flock(Some(FlockSpec { k, window, weights }))
            .expect("flock");
        model
    }

    #[test]
    fn flock_top1_is_the_exact_copy_pointer() {
        let device = Device::Cpu;
        let time = 12usize;
        let head_dim = 4usize;
        let query: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let mut key = vec![0f32; time * head_dim];
        for position in 0..time {
            for lane in 0..head_dim {
                key[position * head_dim + lane] = 0.25 + 0.1 * position as f32 + lane as f32;
            }
        }
        key[3 * head_dim..4 * head_dim].copy_from_slice(&query);
        let repeated: Vec<f32> = (0..time).flat_map(|_| query.clone()).collect();
        let query = Tensor::from_vec(repeated, (1, 1, time, head_dim), &device).expect("q");
        let key = Tensor::from_vec(key, (1, 1, time, head_dim), &device).expect("k");
        let spec = FlockSpec {
            k: 1,
            window: 2,
            weights: FlockWeights::Rank,
        };
        let mut stats = FlockStats::default();
        let probability = flock_probabilities(&query, &key, spec, &mut stats).expect("flock");
        let row = probability
            .narrow(2, time - 1, 1)
            .expect("row")
            .flatten_all()
            .expect("flat")
            .to_vec1::<f32>()
            .expect("vec");
        let selected: Vec<usize> = row
            .iter()
            .enumerate()
            .filter(|(_, value)| **value > 0.0)
            .map(|(index, _)| index)
            .collect();
        assert_eq!(
            selected,
            vec![0, 3, 10, 11],
            "the support must be sink + window + the top-1 copy pointer: {row:?}"
        );
        let top = row
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(index, _)| index)
            .expect("argmax");
        assert_eq!(top, 3, "the copy pointer must be rank 0: {row:?}");
        assert!(row.iter().all(|value| value.is_finite() && *value >= 0.0));
    }

    #[test]
    fn lorentz_rank_matches_the_acosh_distance_reference() {
        let query: Vec<f32> = vec![0.7, -0.4, 1.3, 0.2, -0.9];
        let mut rows = Vec::new();
        for position in 0..9usize {
            rows.push(vec![
                0.1 * position as f32 - 0.3,
                0.2 - 0.05 * position as f32,
                0.4 + 0.03 * position as f32,
                -0.6 + 0.07 * position as f32,
                0.25 * position as f32 - 0.5,
            ]);
        }
        let lorentz: Vec<f32> = rows
            .iter()
            .map(|k| {
                let q2: f32 = query.iter().map(|v| v * v).sum();
                let k2: f32 = k.iter().map(|v| v * v).sum();
                let dot: f32 = query.iter().zip(k).map(|(a, b)| a * b).sum();
                -(1.0 + q2).sqrt() * (1.0 + k2).sqrt() + dot
            })
            .collect();
        let selection = flock_select(
            &lorentz,
            8,
            FlockSelect {
                sink: 0,
                window: 1,
                k: 8,
            },
        )
        .expect("selection");
        let ranked: Vec<usize> = selection
            .entries
            .iter()
            .filter(|entry| entry.slot == FlockSlot::TopK)
            .map(|entry| entry.position)
            .collect();
        // Reference: true hyperbolic distance d = arcosh(q0 k0 - q . k).
        let distance = |k: &[f32]| -> f64 {
            let q2: f64 = query.iter().map(|v| (v * v) as f64).sum();
            let k2: f64 = k.iter().map(|v| (v * v) as f64).sum();
            let dot: f64 = query
                .iter()
                .zip(k)
                .map(|(a, b)| (*a as f64) * (*b as f64))
                .sum();
            (((1.0 + q2).sqrt() * (1.0 + k2).sqrt() - dot).max(1.0)).acosh()
        };
        let mut reference: Vec<usize> = (1..=7).collect();
        reference.sort_by(|&a, &b| distance(&rows[a]).total_cmp(&distance(&rows[b])));
        assert_eq!(
            ranked, reference,
            "the Minkowski ranking must agree with the true distance ordering"
        );
    }

    #[test]
    fn flock_softmax_over_full_support_matches_dense() {
        let dense = model(ScoreKind::Dot, 0.0, Trainable::Scalars);
        let flock = flock_model(64, 64, FlockWeights::Softmax);
        let tokens = ids(1, 8);
        let reference = dense.forward(&tokens, 1, 8, true).expect("dense");
        let logits = flock.forward(&tokens, 1, 8, true).expect("flock");
        let difference = max_abs_difference(&logits, &reference).expect("diff");
        assert!(
            difference <= 1e-4,
            "full-support softmax differs by {difference}"
        );
    }

    #[test]
    fn flock_attention_is_deterministic_and_leaves_weights_unchanged() {
        for weights in [FlockWeights::Softmax, FlockWeights::Rank] {
            let flock = flock_model(3, 4, weights);
            let frozen_before: Vec<(String, Vec<f32>)> = flock
                .frozen
                .iter()
                .map(|(name, tensor)| {
                    (
                        name.clone(),
                        tensor.flatten_all().expect("flat").to_vec1().expect("vec"),
                    )
                })
                .collect();
            let variables_before: Vec<(String, Vec<f32>)> = flock
                .variables
                .iter()
                .map(|(name, var)| {
                    (
                        name.clone(),
                        var.as_tensor()
                            .flatten_all()
                            .expect("flat")
                            .to_vec1()
                            .expect("vec"),
                    )
                })
                .collect();
            let tokens = ids(2, 10);
            let first = flock.forward(&tokens, 2, 10, true).expect("first");
            let second = flock.forward(&tokens, 2, 10, true).expect("second");
            assert_eq!(
                first
                    .flatten_all()
                    .expect("flat")
                    .to_vec1::<f32>()
                    .expect("vec"),
                second
                    .flatten_all()
                    .expect("flat")
                    .to_vec1::<f32>()
                    .expect("vec"),
                "{weights:?} forward is not deterministic"
            );
            let frozen_after: Vec<(String, Vec<f32>)> = flock
                .frozen
                .iter()
                .map(|(name, tensor)| {
                    (
                        name.clone(),
                        tensor.flatten_all().expect("flat").to_vec1().expect("vec"),
                    )
                })
                .collect();
            let variables_after: Vec<(String, Vec<f32>)> = flock
                .variables
                .iter()
                .map(|(name, var)| {
                    (
                        name.clone(),
                        var.as_tensor()
                            .flatten_all()
                            .expect("flat")
                            .to_vec1()
                            .expect("vec"),
                    )
                })
                .collect();
            assert_eq!(
                frozen_before, frozen_after,
                "{weights:?} changed frozen weights"
            );
            assert_eq!(
                variables_before, variables_after,
                "{weights:?} changed trainable weights"
            );
            assert!(
                !flock.variables.contains_key(LOG_EPS),
                "a flock run must not create curvature parameters"
            );
            let stats = flock.take_flock_stats();
            assert_eq!(stats.queries, 2 * 2 * 2 * 4 * 10);
            assert!(stats.mean_selected() > 1.0);
            assert!(
                (stats.sink_share() + stats.window_share() + stats.top_k_share() - 1.0).abs()
                    < 1e-9
            );
        }
    }

    #[test]
    fn flock_requires_dot_scores_and_a_detached_forward() {
        let mut curved = model(ScoreKind::Intrinsic, -1.0, Trainable::Scalars);
        assert!(curved
            .set_flock(Some(FlockSpec {
                k: 7,
                window: 64,
                weights: FlockWeights::Rank,
            }))
            .is_err());
        let mut dot = model(ScoreKind::Dot, 0.0, Trainable::Scalars);
        dot.set_flock(Some(FlockSpec {
            k: 7,
            window: 64,
            weights: FlockWeights::Rank,
        }))
        .expect("flock");
        assert!(dot.forward(&ids(1, 4), 1, 4, false).is_err());
        assert!(dot.forward(&ids(1, 4), 1, 4, true).is_ok());
    }

    #[test]
    #[ignore = "needs the pinned SmolLM2 checkpoint and u16 token file: UOR_B0_MODEL=/path UOR_B0_TOKENS=/path/x.u16 UOR_B0_HELD_OUT=1200000 cargo test -p uor-r4-training --release --offline --lib g1a -- --ignored --nocapture; the exact oracle takes about two hours over 4x512 positions on 8 cores (the default), or set UOR_B0_ORACLE_WORKERS=1 for one worker"]
    fn g1a_dense_path_matches_the_exact_f32_oracle() {
        use uor_r4_model_source::{BehaviorSource, HuggingFaceLlamaOracle, TeacherExecutionConfig};

        let model_dir = std::env::var("UOR_B0_MODEL")
            .expect("set UOR_B0_MODEL to the SmolLM2-135M-Instruct directory");
        let token_path = std::path::PathBuf::from(
            std::env::var("UOR_B0_TOKENS").expect("set UOR_B0_TOKENS to the pinned u16 file"),
        );
        let held_out: usize = std::env::var("UOR_B0_HELD_OUT")
            .expect("set UOR_B0_HELD_OUT to the held-out token count")
            .parse()
            .expect("UOR_B0_HELD_OUT is a token count");
        let device = Device::Cpu;
        let checkpoint = load_checkpoint(Path::new(&model_dir), &device).expect("checkpoint");
        let vocab = checkpoint.shape.vocab;
        let weights_sha = checkpoint.weights_sha256.clone();
        let tokens = read_u16_tokens(&token_path, vocab).expect("tokens");
        let window = 512usize;
        let starts = held_out_starts(tokens.len(), 2048, 4, held_out, 9001).expect("starts");
        let model = KappaLlama::new(checkpoint, ScoreKind::Dot, 0.0, Trainable::Scalars, &device)
            .expect("model");
        let execution = match std::env::var("UOR_B0_ORACLE_WORKERS").as_deref() {
            Ok(fixed) if fixed != "available" => TeacherExecutionConfig::fixed_workers(
                std::num::NonZeroUsize::new(fixed.parse().expect("UOR_B0_ORACLE_WORKERS"))
                    .expect("nonzero"),
            ),
            _ => TeacherExecutionConfig::available_parallelism(),
        };
        let mut oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
            &model_dir, window, execution,
        )
        .map_err(|error| invalid(format!("oracle: {error}")))
        .expect("oracle");
        let started = std::time::Instant::now();
        let mut maximum = 0f64;
        let mut agreement = 0usize;
        let mut positions = 0usize;
        let mut candle_total = 0f64;
        let mut oracle_total = 0f64;
        for (window_index, &start) in starts.iter().enumerate() {
            let inputs = &tokens[start..start + window];
            let targets = &tokens[start + 1..start + window + 1];
            let logits = model
                .forward(inputs, 1, window, true)
                .expect("dense forward")
                .reshape((window, vocab))
                .expect("rows")
                .to_vec2::<f32>()
                .expect("logits");
            eprintln!(
                "G1a window {window_index} start={start}: candle forward done at {:.0}s",
                started.elapsed().as_secs_f64()
            );
            let mut reference = vec![0f32; vocab];
            for (position, &token) in inputs.iter().enumerate() {
                oracle.step(token as usize, position, &mut reference);
                let row = &logits[position];
                let candle_top1 = row
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.total_cmp(b.1))
                    .map(|(index, _)| index)
                    .expect("candle top1");
                let oracle_top1 = reference
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.total_cmp(b.1))
                    .map(|(index, _)| index)
                    .expect("oracle top1");
                if candle_top1 == oracle_top1 {
                    agreement += 1;
                }
                for (a, b) in row.iter().zip(&reference) {
                    maximum = maximum.max((f64::from(*a) - f64::from(*b)).abs());
                }
                let target = targets[position] as usize;
                candle_total += logprob_nll(row, target);
                oracle_total += logprob_nll(&reference, target);
                positions += 1;
                if positions % 64 == 0 {
                    eprintln!(
                        "G1a position {positions}: {:.0}s elapsed, max_abs_logit_delta={maximum}, agreement={agreement}",
                        started.elapsed().as_secs_f64()
                    );
                }
            }
            eprintln!(
                "G1a window {window_index}/{} start={start} done: positions={positions} agreement={agreement} max_abs_logit_delta={maximum} at {:.0}s",
                starts.len(),
                started.elapsed().as_secs_f64()
            );
        }
        let candle_nll = candle_total / positions as f64;
        let oracle_nll = oracle_total / positions as f64;
        eprintln!(
            "G1a: starts={starts:?} window_starts_sha256={} weights={weights_sha} positions={positions} \
             argmax_agreement={agreement}/{positions} max_abs_logit_delta={maximum} \
             candle_nll={candle_nll} oracle_nll={oracle_nll} delta={}",
            crate::sha256_file(&token_path).expect("token sha"),
            (candle_nll - oracle_nll).abs()
        );
        assert_eq!(
            agreement, positions,
            "every argmax must agree with the exact oracle"
        );
        assert!(maximum <= 1e-4, "max |delta logit| {maximum} exceeds 1e-4");
        assert!(
            (candle_nll - oracle_nll).abs() <= 1e-4,
            "NLL differs by more than 1e-4 nats: candle {candle_nll}, oracle {oracle_nll}"
        );
    }

    fn logprob_nll(logits: &[f32], target: usize) -> f64 {
        let maximum = logits
            .iter()
            .fold(f64::NEG_INFINITY, |best, &value| best.max(f64::from(value)));
        let sum: f64 = logits
            .iter()
            .map(|&value| (f64::from(value) - maximum).exp())
            .sum();
        -(f64::from(logits[target]) - maximum - sum.ln())
    }

    fn read_u16_tokens(path: &Path, vocab: usize) -> Result<Vec<u32>> {
        let bytes = fs::read(path)?;
        if bytes.len() % 2 != 0 {
            return Err(invalid("token file is not u16"));
        }
        let tokens: Vec<u32> = bytes
            .chunks_exact(2)
            .map(|b| u32::from(u16::from_le_bytes([b[0], b[1]])))
            .collect();
        if tokens.iter().any(|&t| t as usize >= vocab) {
            return Err(invalid("token file has an out-of-vocabulary token"));
        }
        Ok(tokens)
    }
}
