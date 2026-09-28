//! A multi-layer native geometric language model trained a whole window at a
//! time (the geometric stack), and an ordinary transformer control with the
//! retained #1017 reference's shape. Both are trained from scratch on the same
//! data. Offline training only; nothing here is a serving path.
//!
//! The retained native model (`joint_model`) is one gated recurrent layer with
//! one read, trained position by position. This module tests whether a stack of
//! the same kinds of operators closes the gap to an ordinary transformer of
//! equal size. Its layers take their gates and projections from the layer input
//! alone, so each projection is one matrix product per window; only the cheap
//! recurrence scan runs position by position, batched and parallel over
//! windows.
//!
//! Geometric layers are pre-norm and residual, and each ends with a SwiGLU MLP.
//! The temporal mixer is one of two kinds, chosen per layer by `pattern`:
//! - `r`, quaternion transport recurrence. Each lane of four channels carries a
//!   quaternion state `h_t = lambda_t (u_t * h_{t-1}) + sqrt(1 - lambda_t^2) a_t`.
//!   `u_t` is an input-dependent unit quaternion (near the identity at
//!   initialization), `lambda_t` a gated decay in (0, 1), and `a_t` the input
//!   after a width-4 causal convolution. The window runs through one exact scan
//!   ([`quaternion_scan`]) whose backward is the reverse scan.
//! - `a`, a multi-head read over positions `<= t`. It has a Lorentz
//!   (hyperbolic) or Dot score, a learned age term per head and distance, and a
//!   NoRead slot with a zero value. One fused op ([`fused_read`]) computes it,
//!   with an exact backward.
//!
//! The control layer is RoPE softmax attention plus a SwiGLU MLP, #1017's
//! layout. Its attention uses the same fused op with the Dot score and no NoRead
//! or age, so both models run on the same kernels.
//!
//! Quantization-aware training: with a served representation set
//! ([`StackModel::set_served_representation`]), a geometric stack's forward
//! pass reads exactly the values its integer export writes
//! ([`crate::stack_export::export_stack`]): weight maps through a
//! [`MapCodec`] (by default [`D11Interim`], the export's own 4-bit groups)
//! with the norm gains folded in, per-channel scalars through their grid codes
//! and biases, age tables and Lorentz offsets in fixed point. Gradients reach
//! the float variables by a straight-through estimator.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use candle_core::{CpuStorage, CustomOp2, CustomOp3, DType, Device, Layout, Shape, Tensor, Var};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uor_r4_lut::GROUP;

use crate::lut_export::{dequantize_matrix, quantize_matrix};
use crate::stack_export::{
    block, decay_of_rate, decay_rate, fixed, fixed_value, fold_columns, grid_code, grid_value, pad,
};
use crate::stack_memory::{keys_aux_len, product_key_memory, MemoryConfig, MemoryScore};
use crate::{invalid, Result};

/// Lower bound on `z - 1` in the Lorentz score. Below it the distance is
/// clamped and carries no gradient.
const LORENTZ_MIN_EXCESS: f64 = 1e-7;
const RMS_EPSILON: f64 = 1e-5;
const ROPE_THETA: f64 = 10_000.0;
/// Griffin's recurrence-gate exponent: `log lambda_t = c r_t log a`.
const DECAY_EXPONENT: f64 = 8.0;
/// Timescales (tokens) of the fastest and slowest lanes at a fully open gate.
const DECAY_TIMESCALES: (f64, f64) = (2.0, 1000.0);
const CONVOLUTION_WIDTH: usize = 4;
const INITIAL_STD: f64 = 0.02;
const ROTATION_STD: f64 = 0.002;
/// Initial Lorentz offset: a typical initial distance, so scores start flat.
const INITIAL_LORENTZ_OFFSET: f64 = 2.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StackArch {
    Geometric,
    Transformer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadScore {
    Lorentz,
    Dot,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StackConfig {
    pub arch: StackArch,
    pub vocab_size: usize,
    pub width: usize,
    pub heads: usize,
    pub mlp_hidden: usize,
    pub context: usize,
    /// One character per layer: `r` recurrence, `a` read. Geometric only.
    pub pattern: String,
    /// Score of the geometric reads.
    pub read: ReadScore,
    /// Learned quaternion transport; `false` fixes `u_t` to the identity, which
    /// leaves a real gated linear recurrence with one decay per lane.
    pub rotation: bool,
    pub seed: u64,
    /// Product-key memories in place of some layers' MLPs
    /// ([`crate::stack_memory`]); absent from configurations without one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryConfig>,
}

impl StackConfig {
    /// #1017's shape: width 288, 6 layers, 6 heads, SwiGLU 768, context 256.
    pub fn transformer_control(seed: u64) -> Self {
        Self {
            arch: StackArch::Transformer,
            vocab_size: 4096,
            width: 288,
            heads: 6,
            mlp_hidden: 768,
            context: 256,
            pattern: "aaaaaa".into(),
            read: ReadScore::Dot,
            rotation: false,
            seed,
            memory: None,
        }
    }

    /// A geometric stack with the control's width, depth and heads, whose MLP
    /// width is chosen so the parameter count matches the control's.
    pub fn geometric_matched(
        pattern: &str,
        read: ReadScore,
        rotation: bool,
        seed: u64,
    ) -> Result<Self> {
        Self::geometric_matched_to(&Self::transformer_control(seed), pattern, read, rotation)
    }

    /// A transformer control of any shape: `layers` attention layers.
    pub fn transformer(
        width: usize,
        heads: usize,
        layers: usize,
        mlp_hidden: usize,
        context: usize,
        seed: u64,
    ) -> Result<Self> {
        let config = Self {
            arch: StackArch::Transformer,
            vocab_size: 4096,
            width,
            heads,
            mlp_hidden,
            context,
            pattern: "a".repeat(layers),
            read: ReadScore::Dot,
            rotation: false,
            seed,
            memory: None,
        };
        config.validate()?;
        Ok(config)
    }

    /// A geometric stack with `control`'s width, heads, depth and context,
    /// whose MLP width makes its parameter count match `control`'s.
    pub fn geometric_matched_to(
        control: &Self,
        pattern: &str,
        read: ReadScore,
        rotation: bool,
    ) -> Result<Self> {
        let mut config = Self {
            arch: StackArch::Geometric,
            pattern: pattern.into(),
            read,
            rotation,
            ..control.clone()
        };
        if pattern.len() != control.layers() {
            return Err(invalid(
                "a matched geometric stack needs one pattern letter per control layer",
            ));
        }
        config.mlp_hidden = matched_mlp_hidden(&config, control.parameter_count()?)?;
        config.validate()?;
        Ok(config)
    }

    pub fn layers(&self) -> usize {
        self.pattern.len()
    }

    pub fn head_width(&self) -> usize {
        self.width / self.heads.max(1)
    }

    pub fn validate(&self) -> Result<()> {
        let head = self.head_width();
        if self.vocab_size == 0
            || self.width == 0
            || !self.width.is_multiple_of(4)
            || self.heads == 0
            || !self.width.is_multiple_of(self.heads)
            || !head.is_multiple_of(2)
            || self.mlp_hidden == 0
            || !(2..=4096).contains(&self.context)
            || self.pattern.is_empty()
        {
            return Err(invalid(
                "stack config needs width divisible by 4 and by heads, an even head width, context 2..4096 and at least one layer",
            ));
        }
        match self.arch {
            StackArch::Transformer => {
                if self.pattern.chars().any(|c| c != 'a') {
                    return Err(invalid("the transformer control has attention layers only"));
                }
            }
            StackArch::Geometric => {
                if self.pattern.chars().any(|c| c != 'a' && c != 'r') {
                    return Err(invalid(
                        "geometric pattern letters are r (recurrence) or a (read)",
                    ));
                }
            }
        }
        if let Some(memory) = &self.memory {
            memory.validate(self.layers())?;
        }
        Ok(())
    }

    fn layer_kind(&self, layer: usize) -> char {
        self.pattern.as_bytes()[layer] as char
    }

    /// Whether `layer`'s MLP is a product-key memory.
    pub fn memory_layer(&self, layer: usize) -> bool {
        self.memory
            .as_ref()
            .is_some_and(|memory| memory.layers.contains(&layer))
    }

    fn rotation_rows(&self) -> usize {
        if self.rotation {
            self.width
        } else {
            0
        }
    }

    pub fn shapes(&self) -> BTreeMap<String, Vec<usize>> {
        let d = self.width;
        let m = self.mlp_hidden;
        let lanes = d / 4;
        let mut shapes = BTreeMap::from([
            ("embedding.weight".to_owned(), vec![self.vocab_size, d]),
            ("final_norm.weight".to_owned(), vec![d]),
        ]);
        for layer in 0..self.layers() {
            let name = |suffix: &str| format!("layers.{layer:02}.{suffix}");
            shapes.insert(name("mlp_norm.weight"), vec![d]);
            match &self.memory {
                Some(memory) if memory.layers.contains(&layer) => {
                    let half = memory.key_dim / 2;
                    shapes.insert(
                        name("memory.query.weight"),
                        vec![memory.heads * memory.key_dim, d],
                    );
                    shapes.insert(
                        name("memory.keys"),
                        vec![memory.heads * 2 * memory.sub_keys, half],
                    );
                    shapes.insert(name("memory.values"), vec![memory.slots(), d]);
                    if memory.score == MemoryScore::Lorentz {
                        shapes.insert(name("memory.log_beta"), vec![memory.heads]);
                    }
                }
                _ => {
                    shapes.insert(name("mlp.gate.weight"), vec![m, d]);
                    shapes.insert(name("mlp.up.weight"), vec![m, d]);
                    shapes.insert(name("mlp.down.weight"), vec![d, m]);
                }
            }
            match (self.arch, self.layer_kind(layer)) {
                (StackArch::Transformer, _) => {
                    shapes.insert(name("attn_norm.weight"), vec![d]);
                    for part in ["q", "k", "v", "o"] {
                        shapes.insert(name(&format!("attn.{part}.weight")), vec![d, d]);
                    }
                }
                (StackArch::Geometric, 'r') => {
                    let gates = lanes + self.rotation_rows();
                    shapes.insert(name("rec_norm.weight"), vec![d]);
                    shapes.insert(name("rec.in.weight"), vec![2 * d, d]);
                    shapes.insert(name("rec.conv.weight"), vec![CONVOLUTION_WIDTH, d]);
                    shapes.insert(name("rec.conv.bias"), vec![d]);
                    shapes.insert(name("rec.gate.weight"), vec![gates, d]);
                    shapes.insert(name("rec.gate.bias"), vec![gates]);
                    shapes.insert(name("rec.decay"), vec![lanes]);
                    shapes.insert(name("rec.out.weight"), vec![d, d]);
                }
                (StackArch::Geometric, _) => {
                    shapes.insert(name("read_norm.weight"), vec![d]);
                    for part in ["query", "key", "value", "out"] {
                        shapes.insert(name(&format!("read.{part}.weight")), vec![d, d]);
                    }
                    shapes.insert(name("read.null.weight"), vec![self.heads, d]);
                    shapes.insert(name("read.null.bias"), vec![self.heads]);
                    shapes.insert(name("read.age"), vec![self.heads, self.context]);
                    if self.read == ReadScore::Lorentz {
                        shapes.insert(name("read.log_beta"), vec![self.heads]);
                        shapes.insert(name("read.offset"), vec![self.heads]);
                    }
                }
            }
        }
        shapes
    }

    pub fn parameter_count(&self) -> Result<usize> {
        self.validate()?;
        Ok(self
            .shapes()
            .values()
            .map(|shape| shape.iter().product::<usize>())
            .sum())
    }

    /// Parameters a token reads: all of them except memory value rows outside
    /// its selections.
    pub fn active_parameter_count(&self) -> Result<usize> {
        let idle = self.memory.as_ref().map_or(0, |memory| {
            memory.layers.len() * memory.idle_parameters(self.width)
        });
        Ok(self.parameter_count()? - idle)
    }
}

/// The MLP width whose total parameter count is closest to `target`.
fn matched_mlp_hidden(config: &StackConfig, target: usize) -> Result<usize> {
    let mut probe = config.clone();
    probe.mlp_hidden = 1;
    let base = probe.parameter_count()?;
    let per_unit = 3 * config.width * config.layers();
    if target <= base {
        return Err(invalid("the matched stack has no room for an MLP"));
    }
    let hidden = ((target - base) as f64 / per_unit as f64).round() as usize + 1;
    Ok(hidden.max(1))
}

/// SplitMix64 with Box-Muller normals: a deterministic initializer.
/// Where a weight map reads its input ([`StackModel::hidden_with_capture`]),
/// for calibrating the rounding of the integer export.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StackSite {
    /// The normalized state before its gain, read by a recurrence's input and
    /// gate maps.
    Recurrence(usize),
    /// The recurrence core's output, read by `rec.out`.
    RecurrenceOut(usize),
    /// The normalized state before its gain, read by a read layer's query,
    /// key, value and NoRead maps, or by the control's attention.
    Read(usize),
    /// The merged heads, read by `read.out` or the control's `attn.o`.
    ReadOut(usize),
    /// The normalized state before its gain, read by the MLP's gate and up maps.
    Mlp(usize),
    /// The SwiGLU activation, read by `mlp.down`.
    Down(usize),
    /// The final normalized state before its gain, read by the head.
    Head,
}

/// The capture of [`StackModel::hidden_with_capture`], if any.
type Capture<'a> = Option<&'a mut dyn FnMut(StackSite, &Tensor) -> Result<()>>;

/// Present a map's input, as `(rows, columns)`, to the capture. `input` runs
/// only when there is one, so an uncaptured forward pass does no extra work.
fn tap(
    capture: &mut Capture<'_>,
    site: StackSite,
    input: impl FnOnce() -> Result<Tensor>,
) -> Result<()> {
    if let Some(capture) = capture {
        let x = input()?.detach();
        let cols = x.dim(x.rank() - 1)?;
        capture(site, &x.reshape((x.elem_count() / cols, cols))?)?;
    }
    Ok(())
}

struct Initializer(u64);

impl Initializer {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn uniform(&mut self) -> f64 {
        ((self.next() >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }

    fn normal(&mut self) -> f64 {
        let (u, v) = (self.uniform(), self.uniform());
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }
}

pub struct StackModel {
    pub config: StackConfig,
    variables: BTreeMap<String, Var>,
    device: Device,
    /// The served representation the forward pass reads, if set
    /// ([`Self::set_served_representation`]).
    served: Option<ServedState>,
}

impl StackModel {
    pub fn new(config: StackConfig, device: &Device) -> Result<Self> {
        config.validate()?;
        let mut rng = Initializer(config.seed ^ 0x6765_6F6D_5354_4143);
        let residual_std = INITIAL_STD / (2.0 * config.layers() as f64).sqrt();
        let lanes = config.width / 4;
        let mut variables = BTreeMap::new();
        for (name, shape) in config.shapes() {
            let count: usize = shape.iter().product();
            let suffix = name
                .rsplit_once("layers.")
                .map_or(name.as_str(), |(_, rest)| &rest[3..]);
            let values: Vec<f32> = match suffix {
                "final_norm.weight" | "mlp_norm.weight" | "attn_norm.weight"
                | "rec_norm.weight" | "read_norm.weight" => vec![1.0; count],
                "rec.conv.weight" => (0..count)
                    .map(|index| if index < config.width { 1.0 } else { 0.0 })
                    .collect(),
                "rec.conv.bias" | "read.null.bias" | "read.log_beta" | "memory.log_beta" => {
                    vec![0.0; count]
                }
                // Sub-keys near unit norm, so first scores are of order one.
                "memory.keys" => match config.memory.as_ref().and_then(|m| m.codebook) {
                    // Every head and side holds the whole codebook, in its order.
                    Some(codebook) => {
                        let vectors = codebook.vectors();
                        (0..count / (codebook.size() * codebook.dim()))
                            .flat_map(|_| vectors.iter().flatten().map(|&v| v as f32))
                            .collect()
                    }
                    None => {
                        let half = shape[1].max(1) as f64;
                        (0..count)
                            .map(|_| (rng.normal() / half.sqrt()) as f32)
                            .collect()
                    }
                },
                "read.offset" => vec![INITIAL_LORENTZ_OFFSET as f32; count],
                "rec.gate.bias" => (0..count)
                    .map(|index| {
                        // Rotation rows start at the identity quaternion (1, 0, 0, 0).
                        if index >= lanes && (index - lanes).is_multiple_of(4) {
                            1.0
                        } else {
                            0.0
                        }
                    })
                    .collect(),
                "rec.decay" => (0..count)
                    .map(|lane| {
                        let fraction = lane as f64 / (count.max(2) - 1) as f64;
                        let (fast, slow) = DECAY_TIMESCALES;
                        let tau = fast * (slow / fast).powf(fraction);
                        let a = (-1.0 / (DECAY_EXPONENT * tau)).exp();
                        (a / (1.0 - a)).ln() as f32
                    })
                    .collect(),
                "read.age" => (0..count)
                    .map(|index| {
                        let (head, distance) = (index / config.context, index % config.context);
                        let slope = 2f64.powf(-8.0 * (head + 1) as f64 / config.heads as f64);
                        (-slope * distance as f64) as f32
                    })
                    .collect(),
                "rec.gate.weight" => (0..count)
                    .map(|index| {
                        let std = if index / config.width >= lanes {
                            ROTATION_STD
                        } else {
                            INITIAL_STD
                        };
                        (rng.normal() * std) as f32
                    })
                    .collect(),
                "attn.o.weight" | "read.out.weight" | "rec.out.weight" | "mlp.down.weight" => (0
                    ..count)
                    .map(|_| (rng.normal() * residual_std) as f32)
                    .collect(),
                _ => (0..count)
                    .map(|_| (rng.normal() * INITIAL_STD) as f32)
                    .collect(),
            };
            variables.insert(name, Var::from_vec(values, shape.as_slice(), device)?);
        }
        Ok(Self {
            config,
            variables,
            device: device.clone(),
            served: None,
        })
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn parameter_count(&self) -> usize {
        self.variables.values().map(|var| var.elem_count()).sum()
    }

    /// Variables with and without weight decay (see `decayed`).
    pub fn optimizer_groups(&self) -> (Vec<Var>, Vec<Var>) {
        let mut with_decay = Vec::new();
        let mut plain = Vec::new();
        for (name, var) in &self.variables {
            if decayed(name, var.rank()) {
                with_decay.push(var.clone());
            } else {
                plain.push(var.clone());
            }
        }
        (with_decay, plain)
    }

    pub fn variables(&self) -> &BTreeMap<String, Var> {
        &self.variables
    }

    /// The tensors a forward pass reads: the variables, or in served mode the
    /// served view of their current values.
    fn params(&self) -> Result<Params<'_>> {
        match &self.served {
            None => Ok(Params::Float(&self.variables)),
            Some(state) => Ok(Params::Served(self.served_view(state)?)),
        }
    }

    /// RMSNorm with the gain `gain`; in served mode the gain is folded into
    /// the maps that read the norm, so the norm itself is unit.
    fn norm(&self, p: &Params<'_>, input: &Tensor, gain: &str) -> Result<Tensor> {
        if p.folded_gains() {
            self.unit_norm(input)
        } else {
            self.rms_norm(input, p.get(gain)?)
        }
    }

    fn rms_norm(&self, input: &Tensor, weight: &Tensor) -> Result<Tensor> {
        Ok(input
            .contiguous()?
            .apply_op2(&weight.contiguous()?, RmsNorm)?)
    }

    /// The normalized state before its gain: what a map with the gain folded
    /// in reads.
    fn unit_norm(&self, input: &Tensor) -> Result<Tensor> {
        let ones = Tensor::ones(self.config.width, DType::F32, &self.device)?;
        self.rms_norm(input, &ones)
    }

    /// `input @ weight^T` over the last dimension, as one matrix product.
    fn linear(input: &Tensor, weight: &Tensor) -> Result<Tensor> {
        let dims = input.dims().to_vec();
        let (rows, features) = (
            dims[..dims.len() - 1].iter().product::<usize>(),
            dims[dims.len() - 1],
        );
        let output = input.reshape((rows, features))?.matmul(&weight.t()?)?;
        let mut shape = dims;
        let last = shape.len() - 1;
        shape[last] = weight.dim(0)?;
        Ok(output.reshape(shape)?)
    }

    fn mlp(
        &self,
        p: &Params<'_>,
        layer: usize,
        x: &Tensor,
        capture: &mut Capture<'_>,
    ) -> Result<Tensor> {
        tap(capture, StackSite::Mlp(layer), || self.unit_norm(x))?;
        let u = self.norm(p, x, &layer_name(layer, "mlp_norm.weight"))?;
        if let Some(memory) = self
            .config
            .memory
            .as_ref()
            .filter(|m| m.layers.contains(&layer))
        {
            return self.memory(p, layer, &u, memory);
        }
        let gate = Self::linear(&u, p.layer(layer, "mlp.gate.weight")?)?;
        let up = Self::linear(&u, p.layer(layer, "mlp.up.weight")?)?;
        let mixed = gate.contiguous()?.apply_op2(&up.contiguous()?, SwiGlu)?;
        tap(capture, StackSite::Down(layer), || Ok(mixed.clone()))?;
        Self::linear(&mixed, p.layer(layer, "mlp.down.weight")?)
    }

    /// The product-key memory in place of `layer`'s MLP, on the normalized
    /// input `u` [batch, time, width].
    fn memory(
        &self,
        p: &Params<'_>,
        layer: usize,
        u: &Tensor,
        memory: &MemoryConfig,
    ) -> Result<Tensor> {
        let (batch, time, width) = u.dims3()?;
        let query = Self::linear(u, p.layer(layer, "memory.query.weight")?)?
            .reshape((batch * time, memory.heads * memory.key_dim))?;
        let keys = p.layer(layer, "memory.keys")?.flatten_all()?;
        // Fixed codebook keys carry no gradient, so they never move.
        let mut aux = vec![if memory.codebook.is_some() {
            keys.detach()
        } else {
            keys
        }];
        if memory.score == MemoryScore::Lorentz {
            aux.push(p.layer(layer, "memory.log_beta")?.exp()?);
        }
        let aux = Tensor::cat(&aux, 0)?;
        if aux.elem_count() != keys_aux_len(memory) {
            return Err(invalid("memory key layout differs from its configuration"));
        }
        Ok(
            product_key_memory(&query, &aux, p.layer(layer, "memory.values")?, memory)?
                .reshape((batch, time, width))?,
        )
    }

    /// Splits [batch, time, width] into [batch, heads, time, head_width].
    fn heads(&self, x: &Tensor, batch: usize, time: usize) -> Result<Tensor> {
        Ok(
            x.reshape((batch, time, self.config.heads, self.config.head_width()))?
                .transpose(1, 2)?
                .contiguous()?,
        )
    }

    fn merge_heads(&self, x: &Tensor, batch: usize, time: usize) -> Result<Tensor> {
        Ok(x.transpose(1, 2)?
            .reshape((batch, time, self.config.width))?)
    }

    fn attention(
        &self,
        p: &Params<'_>,
        layer: usize,
        x: &Tensor,
        capture: &mut Capture<'_>,
    ) -> Result<Tensor> {
        let (batch, time, _) = x.dims3()?;
        tap(capture, StackSite::Read(layer), || self.unit_norm(x))?;
        let u = self.norm(p, x, &layer_name(layer, "attn_norm.weight"))?;
        let project = |part: &str| -> Result<Tensor> {
            self.heads(
                &Self::linear(&u, p.layer(layer, &format!("attn.{part}.weight"))?)?,
                batch,
                time,
            )
        };
        let (query, key, value) = (project("q")?, project("k")?, project("v")?);
        let aux = Tensor::zeros(1, DType::F32, &self.device)?;
        let read = fused_read(
            &query,
            &key,
            &value,
            &aux,
            ReadScore::Dot,
            false,
            false,
            true,
        )?;
        let merged = self.merge_heads(&read, batch, time)?;
        tap(capture, StackSite::ReadOut(layer), || Ok(merged.clone()))?;
        Self::linear(&merged, p.layer(layer, "attn.o.weight")?)
    }

    fn geometric_read(
        &self,
        p: &Params<'_>,
        layer: usize,
        x: &Tensor,
        capture: &mut Capture<'_>,
    ) -> Result<Tensor> {
        let (batch, time, _) = x.dims3()?;
        let heads = self.config.heads;
        tap(capture, StackSite::Read(layer), || self.unit_norm(x))?;
        let u = self.norm(p, x, &layer_name(layer, "read_norm.weight"))?;
        let project = |part: &str| -> Result<Tensor> {
            self.heads(
                &Self::linear(&u, p.layer(layer, &format!("read.{part}.weight"))?)?,
                batch,
                time,
            )
        };
        let (query, key, value) = (project("query")?, project("key")?, project("value")?);
        let null = Self::linear(&u, p.layer(layer, "read.null.weight")?)?
            .broadcast_add(p.layer(layer, "read.null.bias")?)?
            .transpose(1, 2)?
            .flatten_all()?;
        let age = p
            .layer(layer, "read.age")?
            .narrow(1, 0, time)?
            .flatten_all()?;
        let mut aux = vec![null, age];
        if self.config.read == ReadScore::Lorentz {
            aux.push(p.layer(layer, "read.log_beta")?.exp()?);
            aux.push(p.layer(layer, "read.offset")?.clone());
        }
        let aux = Tensor::cat(&aux, 0)?;
        if aux.dim(0)? != fused_aux_len(batch, heads, time, self.config.read, true, true) {
            return Err(invalid("fused read auxiliary layout differs"));
        }
        let read = fused_read(
            &query,
            &key,
            &value,
            &aux,
            self.config.read,
            true,
            true,
            false,
        )?;
        let merged = self.merge_heads(&read, batch, time)?;
        tap(capture, StackSite::ReadOut(layer), || Ok(merged.clone()))?;
        Self::linear(&merged, p.layer(layer, "read.out.weight")?)
    }

    fn recurrence(
        &self,
        p: &Params<'_>,
        layer: usize,
        x: &Tensor,
        capture: &mut Capture<'_>,
    ) -> Result<Tensor> {
        let (batch, time, width) = x.dims3()?;
        tap(capture, StackSite::Recurrence(layer), || self.unit_norm(x))?;
        let u = self.norm(p, x, &layer_name(layer, "rec_norm.weight"))?;
        let branches = Self::linear(&u, p.layer(layer, "rec.in.weight")?)?;
        let gates = Self::linear(&u, p.layer(layer, "rec.gate.weight")?)?
            .broadcast_add(p.layer(layer, "rec.gate.bias")?)?;
        let parameters = Tensor::cat(
            &[
                &p.layer(layer, "rec.conv.weight")?.flatten_all()?,
                p.layer(layer, "rec.conv.bias")?,
                p.layer(layer, "rec.decay")?,
            ],
            0,
        )?;
        let core = branches.contiguous()?.apply_op3(
            &gates.contiguous()?,
            &parameters,
            RecurrenceCore {
                batch,
                time,
                width,
                rotation: self.config.rotation,
            },
        )?;
        tap(
            capture,
            StackSite::RecurrenceOut(layer),
            || Ok(core.clone()),
        )?;
        Self::linear(&core, p.layer(layer, "rec.out.weight")?)
    }

    /// Logits [batch * time, vocabulary] for a batch of windows. Positions see
    /// only themselves and earlier positions of the same window.
    pub fn forward(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        let p = self.params()?;
        let hidden = self.hidden_hooked(&p, ids, batch, time, &mut None)?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// The final normalized states [batch * time, width], which the tied
    /// embedding (in served mode, the served head) maps to logits.
    pub fn hidden(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        let p = self.params()?;
        self.hidden_hooked(&p, ids, batch, time, &mut None)
    }

    /// [`hidden`](Self::hidden), presenting the input of every weight map to
    /// `capture` as detached `(batch * time, columns)` tensors (see
    /// [`StackSite`]). Maps that read a normalized state see it before its gain,
    /// as the export folds the gains into them. The embedding is a lookup and
    /// has no site.
    pub fn hidden_with_capture(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        capture: &mut dyn FnMut(StackSite, &Tensor) -> Result<()>,
    ) -> Result<Tensor> {
        let p = self.params()?;
        self.hidden_hooked(&p, ids, batch, time, &mut Some(capture))
    }

    fn hidden_hooked(
        &self,
        p: &Params<'_>,
        ids: &[u32],
        batch: usize,
        time: usize,
        capture: &mut Capture<'_>,
    ) -> Result<Tensor> {
        let x = self.embed_with(p, ids, batch, time)?;
        self.layers_hooked(p, x, capture)
    }

    /// The token embeddings [batch, time, width] that the first layer reads.
    pub fn embed(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        let p = self.params()?;
        self.embed_with(&p, ids, batch, time)
    }

    fn embed_with(&self, p: &Params<'_>, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        if ids.len() != batch * time || time == 0 || time > self.config.context {
            return Err(invalid(
                "stack forward needs batch * time ids within the context",
            ));
        }
        if ids.iter().any(|&id| id as usize >= self.config.vocab_size) {
            return Err(invalid("token id outside the vocabulary"));
        }
        let embedding = p.get("embedding.weight")?;
        let index = Tensor::from_vec(ids.to_vec(), batch * time, &self.device)?;
        Ok(embedding
            .index_select(&index, 0)?
            .reshape((batch, time, self.config.width))?)
    }

    /// The final normalized states [batch * time, width] from a first-layer
    /// input [batch, time, width]: [`hidden`](Self::hidden) for callers that add
    /// a side channel to the embeddings (`stack_tracking`).
    pub fn hidden_from_input(&self, x: Tensor) -> Result<Tensor> {
        let (_, time, width) = x.dims3()?;
        if time == 0 || time > self.config.context || width != self.config.width {
            return Err(invalid(
                "stack input needs [batch, time, width] within the context",
            ));
        }
        let p = self.params()?;
        self.layers_hooked(&p, x, &mut None)
    }

    /// Logits [rows, vocabulary] from final states, through the tied embedding
    /// (in served mode, the served head).
    pub fn head(&self, hidden: &Tensor) -> Result<Tensor> {
        let p = self.params()?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// The residual stream [batch, time, width] after running `layers` on
    /// `x`, without the final norm: for callers that inject a side channel
    /// between layers (`stack_tracking`). [`finish`](Self::finish) completes it.
    pub fn run_layers(&self, x: Tensor, layers: std::ops::Range<usize>) -> Result<Tensor> {
        let (_, time, width) = x.dims3()?;
        if time == 0
            || time > self.config.context
            || width != self.config.width
            || layers.start > layers.end
            || layers.end > self.config.layers()
        {
            return Err(invalid(
                "stack layers need [batch, time, width] within the context and a valid layer range",
            ));
        }
        let p = self.params()?;
        self.layer_range_hooked(&p, x, layers, &mut None)
    }

    /// The final normalized states [batch * time, width] from the residual
    /// stream after the last layer.
    pub fn finish(&self, x: Tensor) -> Result<Tensor> {
        let p = self.params()?;
        self.finish_hooked(&p, x, &mut None)
    }

    fn layers_hooked(
        &self,
        p: &Params<'_>,
        x: Tensor,
        capture: &mut Capture<'_>,
    ) -> Result<Tensor> {
        let x = self.layer_range_hooked(p, x, 0..self.config.layers(), capture)?;
        self.finish_hooked(p, x, capture)
    }

    fn layer_range_hooked(
        &self,
        p: &Params<'_>,
        mut x: Tensor,
        layers: std::ops::Range<usize>,
        capture: &mut Capture<'_>,
    ) -> Result<Tensor> {
        for layer in layers {
            let mixed = match (self.config.arch, self.config.layer_kind(layer)) {
                (StackArch::Transformer, _) => self.attention(p, layer, &x, capture)?,
                (StackArch::Geometric, 'r') => self.recurrence(p, layer, &x, capture)?,
                (StackArch::Geometric, _) => self.geometric_read(p, layer, &x, capture)?,
            };
            x = x.add(&mixed)?;
            x = x.add(&self.mlp(p, layer, &x, capture)?)?;
        }
        Ok(x)
    }

    fn finish_hooked(
        &self,
        p: &Params<'_>,
        x: Tensor,
        capture: &mut Capture<'_>,
    ) -> Result<Tensor> {
        let (batch, time, _) = x.dims3()?;
        tap(capture, StackSite::Head, || self.unit_norm(&x))?;
        let x = self.norm(p, &x, "final_norm.weight")?;
        Ok(x.reshape((batch * time, self.config.width))?)
    }

    /// Mean next-token negative log-likelihood (nats) over all targets.
    pub fn loss(&self, ids: &[u32], targets: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        if targets.len() != ids.len() {
            return Err(invalid("one target per input id"));
        }
        if targets
            .iter()
            .any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid("target id outside the vocabulary"));
        }
        let logits = self.forward(ids, batch, time)?;
        Ok(logits.apply_op1(CrossEntropy {
            targets: targets.to_vec(),
            weights: None,
        })?)
    }

    /// Weighted mean next-token negative log-likelihood (nats):
    /// `sum_i w_i nll_i / sum_i w_i`, for a loss on chosen targets only (a
    /// dialogue's responses). Weights are finite and nonnegative with a
    /// positive sum; rows of weight zero cost no loss or gradient work.
    pub fn weighted_loss(
        &self,
        ids: &[u32],
        targets: &[u32],
        weights: &[f32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        if targets.len() != ids.len() || weights.len() != ids.len() {
            return Err(invalid("one target and one weight per input id"));
        }
        if targets
            .iter()
            .any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid("target id outside the vocabulary"));
        }
        if weights.iter().any(|w| !w.is_finite() || *w < 0.0)
            || weights.iter().map(|&w| f64::from(w)).sum::<f64>() <= 0.0
        {
            return Err(invalid(
                "loss weights must be finite, nonnegative and not all zero",
            ));
        }
        let logits = self.forward(ids, batch, time)?;
        Ok(logits.apply_op1(CrossEntropy {
            targets: targets.to_vec(),
            weights: Some(weights.to_vec()),
        })?)
    }

    /// Per-target negative log-likelihoods (nats), without a backward graph.
    pub fn target_nll(
        &self,
        ids: &[u32],
        targets: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<Vec<f64>> {
        if targets
            .iter()
            .any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid("target id outside the vocabulary"));
        }
        row_nll(&self.forward(ids, batch, time)?.detach(), targets)
    }

    pub fn save(&self, directory: &Path) -> Result<()> {
        fs::create_dir_all(directory)?;
        let tensors: std::collections::HashMap<String, Tensor> = self
            .variables
            .iter()
            .map(|(name, var)| (name.clone(), var.as_tensor().clone()))
            .collect();
        candle_core::safetensors::save(&tensors, directory.join("model.safetensors"))?;
        fs::write(
            directory.join("config.json"),
            serde_json::to_vec_pretty(&self.config)?,
        )?;
        Ok(())
    }

    pub fn load(directory: &Path, device: &Device) -> Result<Self> {
        let config: StackConfig =
            serde_json::from_slice(&fs::read(directory.join("config.json"))?)?;
        config.validate()?;
        let tensors = candle_core::safetensors::load(directory.join("model.safetensors"), device)?;
        let shapes = config.shapes();
        if tensors.len() != shapes.len() {
            return Err(invalid("saved stack tensors differ from the configuration"));
        }
        let mut variables = BTreeMap::new();
        for (name, shape) in shapes {
            let tensor = tensors
                .get(&name)
                .ok_or_else(|| invalid(format!("saved stack lacks {name}")))?;
            if tensor.dims() != shape.as_slice() || tensor.dtype() != DType::F32 {
                return Err(invalid(format!("saved stack shape differs for {name}")));
            }
            variables.insert(name, Var::from_tensor(tensor)?);
        }
        Ok(Self {
            config,
            variables,
            device: device.clone(),
            served: None,
        })
    }
}

impl StackModel {
    /// Evaluation-only diagnostic (roadmap S1.0b): next-token logits
    /// `[batch * time, vocabulary]` with every transport quaternion replaced,
    /// before its scaling by lambda, by the nearest of `snap` (for example the
    /// 120 unit icosians of 2I). The recurrences run through their composed
    /// Candle reference; with `snap = None` this equals the fused forward
    /// within float tolerance. Not a training or serving path.
    pub fn logits_with_transport(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        snap: Option<&[[f32; 4]]>,
    ) -> Result<Tensor> {
        if self.config.arch != StackArch::Geometric {
            return Err(invalid("transport snapping needs a geometric stack"));
        }
        if snap.is_some() && !self.config.rotation {
            return Err(invalid("transport snapping needs a rotating stack"));
        }
        let p = self.params()?;
        let mut x = self.embed_with(&p, ids, batch, time)?;
        for layer in 0..self.config.layers() {
            let mixed = match self.config.layer_kind(layer) {
                'r' => self.composed_recurrence(&p, layer, &x, snap)?,
                _ => self.geometric_read(&p, layer, &x, &mut None)?,
            };
            x = x.add(&mixed)?;
            x = x.add(&self.mlp(&p, layer, &x, &mut None)?)?;
        }
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// The recurrence mixer composed from Candle operations: the reference
    /// for the fused core. `snap` replaces each unit transport quaternion by
    /// its nearest root (see [`Self::logits_with_transport`]).
    fn composed_recurrence(
        &self,
        p: &Params<'_>,
        layer: usize,
        x: &Tensor,
        snap: Option<&[[f32; 4]]>,
    ) -> Result<Tensor> {
        let (batch, time, width) = x.dims3()?;
        let lanes = width / 4;
        let u = self.norm(p, x, &layer_name(layer, "rec_norm.weight"))?;
        let branches = Self::linear(&u, p.layer(layer, "rec.in.weight")?)?;
        let input = branches.narrow(2, 0, width)?;
        let gate = branches.narrow(2, width, width)?;
        // Width-4 causal depthwise convolution over time.
        let weights = p.layer(layer, "rec.conv.weight")?;
        let mut convolved = p
            .layer(layer, "rec.conv.bias")?
            .broadcast_as((batch, time, width))?
            .contiguous()?;
        for shift in 0..CONVOLUTION_WIDTH.min(time) {
            let shifted = if shift == 0 {
                input.clone()
            } else {
                Tensor::cat(
                    &[
                        &Tensor::zeros((batch, shift, width), DType::F32, &self.device)?,
                        &input.narrow(1, 0, time - shift)?,
                    ],
                    1,
                )?
            };
            convolved = convolved.add(&shifted.broadcast_mul(&weights.get(shift)?)?)?;
        }
        let gates = Self::linear(&u, p.layer(layer, "rec.gate.weight")?)?
            .broadcast_add(p.layer(layer, "rec.gate.bias")?)?;
        let opening = candle_nn::ops::sigmoid(&gates.narrow(2, 0, lanes)?)?;
        // log a = -softplus(-decay) keeps a in (0, 1); log lambda = c r log a.
        let log_a = p
            .layer(layer, "rec.decay")?
            .neg()?
            .exp()?
            .affine(1.0, 1.0)?
            .log()?
            .neg()?;
        let log_lambda = opening.broadcast_mul(&log_a)?.affine(DECAY_EXPONENT, 0.0)?;
        let lambda = log_lambda.exp()?.unsqueeze(3)?;
        let keep = lambda
            .sqr()?
            .affine(-1.0, 1.0)?
            .clamp(1e-6f32, 1f32)?
            .sqrt()?;
        let transition = if self.config.rotation {
            let raw = gates
                .narrow(2, lanes, width)?
                .reshape((batch, time, lanes, 4))?;
            let norm = raw.sqr()?.sum_keepdim(3)?.affine(1.0, 1e-6)?.sqrt()?;
            let unit = raw.broadcast_div(&norm)?;
            let unit = match snap {
                None => unit,
                Some(roots) => snap_to_roots(&unit, roots)?,
            };
            unit.broadcast_mul(&lambda)?
        } else {
            let zeros = Tensor::zeros((batch, time, lanes, 3), DType::F32, &self.device)?;
            Tensor::cat(&[&lambda, &zeros], 3)?
        };
        let drive = convolved
            .reshape((batch, time, lanes, 4))?
            .broadcast_mul(&keep)?;
        let state = quaternion_scan(&transition.contiguous()?, &drive.contiguous()?)?
            .reshape((batch, time, width))?;
        Self::linear(
            &state.mul(&gate.gelu()?)?,
            p.layer(layer, "rec.out.weight")?,
        )
    }
}

/// Each quaternion of a `[.., 4]` tensor replaced by the root with the
/// largest dot product: the nearest root, as every root is a unit.
fn snap_to_roots(unit: &Tensor, roots: &[[f32; 4]]) -> Result<Tensor> {
    if roots.is_empty() {
        return Err(invalid("snapping needs at least one root"));
    }
    let shape = unit.shape().clone();
    let values = unit.flatten_all()?.to_vec1::<f32>()?;
    let snapped: Vec<[f32; 4]> = values
        .par_chunks(4)
        .map(|q| {
            let mut best = roots[0];
            let mut best_dot = f32::NEG_INFINITY;
            for root in roots {
                let dot = q[0] * root[0] + q[1] * root[1] + q[2] * root[2] + q[3] * root[3];
                if dot > best_dot {
                    best_dot = dot;
                    best = *root;
                }
            }
            best
        })
        .collect();
    Ok(Tensor::from_vec(
        snapped.into_iter().flatten().collect::<Vec<f32>>(),
        shape,
        unit.device(),
    )?)
}

// ---------------------------------------------------------------------------
// The served representation: training with the export's values in the loop.

/// A representation of the weight maps whose values a forward pass can read
/// ([`StackModel::set_served_representation`]): its round trip returns the
/// dequantized values a serving artifact would hold. [`D11Interim`] is the
/// stack export's own; a geometry-coded 4-bit codec (roadmap D4) plugs in here.
pub trait MapCodec: Send + Sync {
    /// A stable name for reports.
    fn name(&self) -> &str;

    /// Round-trip a row-major `rows x cols` matrix through the codec
    /// (dequantized values). `cols` is a multiple of `uor_r4_lut::GROUP`.
    fn round_trip(&self, values: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>>;
}

/// The D11 interim weight format exactly as
/// [`crate::stack_export::export_stack`] writes it without calibration: 4-bit
/// values in groups of `GROUP` columns with a one-byte grid scale per group,
/// rounded to nearest ([`quantize_matrix`], then [`dequantize_matrix`]).
#[derive(Clone, Copy, Debug, Default)]
pub struct D11Interim;

impl MapCodec for D11Interim {
    fn name(&self) -> &str {
        "d11-interim-4bit-g32-round-to-nearest"
    }

    fn round_trip(&self, values: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>> {
        let packed = quantize_matrix(values, rows, cols)?;
        dequantize_matrix(rows, cols, packed.exp_base, &packed.nibbles, &packed.scales)
    }
}

/// The served view's name for the output map: the embedding with the final
/// norm's gain folded in, which the export writes as its own matrix.
const SERVED_HEAD: &str = "head";

fn layer_name(layer: usize, suffix: &str) -> String {
    format!("layers.{layer:02}.{suffix}")
}

/// How the export writes one tensor.
#[derive(Clone, Debug)]
enum ServedKind {
    /// A `rows x cols` weight map through the codec: its columns times the
    /// norm gain `gain` first, if any, and padded with zeros to `padded` (the
    /// MLP's units) as the export pads.
    Map {
        gain: Option<String>,
        rows: usize,
        cols: usize,
        padded: (usize, usize),
    },
    /// Grid codes of the values (convolution taps).
    Taps,
    /// Grid codes of the rate `8 softplus(-decay)`, read back as a decay.
    DecayRate,
    /// Grid codes of `beta = exp(log_beta)`, read back as `ln beta`.
    LorentzScale,
    /// Integers at `2^exp`.
    Fixed(i32),
}

/// One tensor of the served view.
#[derive(Clone, Debug)]
struct ServedTensor {
    /// Its name in the view: its variable's, or [`SERVED_HEAD`].
    name: String,
    /// The variable it stands for, which its gradient reaches.
    source: String,
    kind: ServedKind,
}

impl ServedTensor {
    /// The variables its value is computed from.
    fn sources(&self) -> impl Iterator<Item = &str> {
        let gain = match &self.kind {
            ServedKind::Map { gain, .. } => gain.as_deref(),
            _ => None,
        };
        std::iter::once(self.source.as_str()).chain(gain)
    }

    /// The values the export writes for this tensor, read back as floats
    /// (what [`crate::stack_export::stack_grid_reference`] holds), from the
    /// variables' current `values`.
    fn served_values(
        &self,
        values: &BTreeMap<String, Vec<f32>>,
        codec: &dyn MapCodec,
    ) -> Result<Vec<f32>> {
        let get = |name: &str| {
            values
                .get(name)
                .ok_or_else(|| invalid(format!("missing stack variable {name}")))
        };
        let source = get(&self.source)?;
        let scalars = |f: &dyn Fn(f64) -> Result<f64>| -> Result<Vec<f32>> {
            source
                .iter()
                .map(|&v| Ok(f(f64::from(v))? as f32))
                .collect()
        };
        match &self.kind {
            ServedKind::Map {
                gain,
                rows,
                cols,
                padded,
            } => {
                let mut map = source.clone();
                if let Some(gain) = gain {
                    fold_columns(&mut map, *cols, get(gain)?);
                }
                let (to_rows, to_cols) = *padded;
                let same = (to_rows, to_cols) == (*rows, *cols);
                if !same {
                    map = pad(&map, *rows, *cols, to_rows, to_cols);
                }
                let served = codec.round_trip(&map, to_rows, to_cols)?;
                if served.len() != to_rows * to_cols || served.iter().any(|v| !v.is_finite()) {
                    return Err(invalid(format!(
                        "codec {} returned {} values or a nonfinite one for {} ({to_rows} x {to_cols})",
                        codec.name(),
                        served.len(),
                        self.name
                    )));
                }
                Ok(if same {
                    served
                } else {
                    block(&served, to_cols, *rows, *cols)
                })
            }
            ServedKind::Taps => scalars(&|v| Ok(grid_value(grid_code(v)?))),
            ServedKind::DecayRate => {
                scalars(&|v| Ok(decay_of_rate(grid_value(grid_code(decay_rate(v))?))))
            }
            ServedKind::LorentzScale => scalars(&|v| Ok(grid_value(grid_code(v.exp())?).ln())),
            ServedKind::Fixed(exp) => scalars(&|v| Ok(fixed_value(fixed(v, *exp)?, *exp))),
        }
    }
}

/// Every tensor the stack export writes, in the forward pass's terms: for a
/// geometric stack without memories whose width is a multiple of `GROUP`.
fn served_plan(config: &StackConfig) -> Result<Vec<ServedTensor>> {
    if config.arch != StackArch::Geometric || config.memory.is_some() {
        return Err(invalid(
            "the served representation is the geometric stack export's; it has no transformer or memory layers",
        ));
    }
    if !config.width.is_multiple_of(GROUP) {
        return Err(invalid(format!(
            "the served representation needs a width that is a multiple of {GROUP}"
        )));
    }
    let (d, vocab) = (config.width, config.vocab_size);
    let m = config.mlp_hidden;
    let padded_mlp = m.div_ceil(GROUP) * GROUP;
    let map = |name: String, gain: Option<String>, rows: usize, cols: usize, padded| ServedTensor {
        source: name.clone(),
        name,
        kind: ServedKind::Map {
            gain,
            rows,
            cols,
            padded,
        },
    };
    let scalar = |name: String, kind: ServedKind| ServedTensor {
        source: name.clone(),
        name,
        kind,
    };
    let mut plan = vec![
        map("embedding.weight".into(), None, vocab, d, (vocab, d)),
        ServedTensor {
            name: SERVED_HEAD.into(),
            source: "embedding.weight".into(),
            kind: ServedKind::Map {
                gain: Some("final_norm.weight".into()),
                rows: vocab,
                cols: d,
                padded: (vocab, d),
            },
        },
    ];
    for layer in 0..config.layers() {
        let n = |suffix: &str| layer_name(layer, suffix);
        if config.layer_kind(layer) == 'r' {
            let gain = Some(n("rec_norm.weight"));
            let gate_rows = d / 4 + config.rotation_rows();
            plan.push(map(n("rec.in.weight"), gain.clone(), 2 * d, d, (2 * d, d)));
            plan.push(map(
                n("rec.gate.weight"),
                gain,
                gate_rows,
                d,
                (gate_rows, d),
            ));
            plan.push(map(n("rec.out.weight"), None, d, d, (d, d)));
            plan.push(scalar(n("rec.conv.weight"), ServedKind::Taps));
            plan.push(scalar(n("rec.conv.bias"), ServedKind::Fixed(-16)));
            plan.push(scalar(n("rec.gate.bias"), ServedKind::Fixed(-16)));
            plan.push(scalar(n("rec.decay"), ServedKind::DecayRate));
        } else {
            let gain = Some(n("read_norm.weight"));
            for part in ["query", "key", "value"] {
                let name = n(&format!("read.{part}.weight"));
                plan.push(map(name, gain.clone(), d, d, (d, d)));
            }
            let heads = config.heads;
            plan.push(map(n("read.null.weight"), gain, heads, d, (heads, d)));
            plan.push(map(n("read.out.weight"), None, d, d, (d, d)));
            plan.push(scalar(n("read.null.bias"), ServedKind::Fixed(-16)));
            plan.push(scalar(n("read.age"), ServedKind::Fixed(-16)));
            if config.read == ReadScore::Lorentz {
                plan.push(scalar(n("read.log_beta"), ServedKind::LorentzScale));
                plan.push(scalar(n("read.offset"), ServedKind::Fixed(-24)));
            }
        }
        let gain = Some(n("mlp_norm.weight"));
        plan.push(map(
            n("mlp.gate.weight"),
            gain.clone(),
            m,
            d,
            (padded_mlp, d),
        ));
        plan.push(map(n("mlp.up.weight"), gain, m, d, (padded_mlp, d)));
        plan.push(map(n("mlp.down.weight"), None, d, m, (d, padded_mlp)));
    }
    Ok(plan)
}

/// Straight-through estimator: the output is the second input's values
/// exactly (the served values) and the gradient passes to the first input (the
/// float expression they were computed from) unchanged. This is
/// `w + (q(w) - w).detach()` without the rounding of that sum.
struct StraightThrough;

impl CustomOp2 for StraightThrough {
    fn name(&self) -> &'static str {
        "geometric-stack-straight-through"
    }

    fn cpu_fwd(
        &self,
        _s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        if l1.shape() != l2.shape() {
            candle_core::bail!("straight-through inputs must have one shape");
        }
        Ok((
            CpuStorage::F32(contiguous(s2, l2)?.to_vec()),
            l2.shape().clone(),
        ))
    }

    fn bwd(
        &self,
        _source: &Tensor,
        _served: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        Ok((Some(grad.clone()), None))
    }
}

/// Work of the served representation so far ([`StackModel::served_statistics`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct ServedStatistics {
    /// Forward passes that compared the variables with the served view's.
    pub checks: usize,
    /// Checks that found changed variables and recomputed served tensors.
    pub refreshes: usize,
    /// Served tensors recomputed.
    pub tensors: usize,
    /// Seconds spent comparing and recomputing.
    pub seconds: f64,
}

/// A served view and the variable values it was computed from.
#[derive(Clone)]
struct ServedCache {
    sources: Arc<BTreeMap<String, Vec<f32>>>,
    tensors: Arc<BTreeMap<String, Tensor>>,
}

/// Served mode ([`StackModel::set_served_representation`]).
struct ServedState {
    codec: Arc<dyn MapCodec>,
    plan: Vec<ServedTensor>,
    cache: Mutex<Option<ServedCache>>,
    statistics: Mutex<ServedStatistics>,
}

/// The tensors one forward pass reads: the variables, or in served mode the
/// served view of their current values.
enum Params<'a> {
    Float(&'a BTreeMap<String, Var>),
    Served(Arc<BTreeMap<String, Tensor>>),
}

impl Params<'_> {
    fn get(&self, name: &str) -> Result<&Tensor> {
        let found = match self {
            Self::Float(variables) => variables.get(name).map(Var::as_tensor),
            Self::Served(tensors) => tensors.get(name),
        };
        found.ok_or_else(|| invalid(format!("missing stack variable {name}")))
    }

    fn layer(&self, layer: usize, suffix: &str) -> Result<&Tensor> {
        self.get(&layer_name(layer, suffix))
    }

    /// The output map: the tied embedding, or the served head.
    fn head(&self) -> Result<&Tensor> {
        match self {
            Self::Float(_) => self.get("embedding.weight"),
            Self::Served(_) => self.get(SERVED_HEAD),
        }
    }

    /// Whether the norm gains are folded into the maps that read the norms.
    fn folded_gains(&self) -> bool {
        matches!(self, Self::Served(_))
    }
}

/// Bitwise equality of two float slices.
fn same_bits(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
}

impl StackModel {
    /// Train and evaluate with the served representation (`Some`), or in
    /// float (`None`, the default). In served mode every forward pass
    /// ([`forward`](Self::forward), [`loss`](Self::loss) and the rest) reads
    /// exactly the values [`crate::stack_export::export_stack`] writes:
    /// weight maps through `codec` with the norm gains folded in (the norms
    /// themselves unit, the head a separate map of the embedding times the
    /// final gain, the MLP padded to whole groups), per-channel scalars
    /// through their grid codes (convolution taps; decays through their rates;
    /// Lorentz scales through `beta`), and biases, age tables and Lorentz
    /// offsets in fixed point, each rounded as the export rounds it. Gradients
    /// reach the float variables by a straight-through estimator, the gains'
    /// through the folded products. Served values are recomputed, in
    /// parallel, whenever a variable they come from changed. Geometric stacks
    /// without memories whose width is a multiple of `GROUP` only.
    pub fn set_served_representation(&mut self, codec: Option<Arc<dyn MapCodec>>) -> Result<()> {
        self.served = match codec {
            None => None,
            Some(codec) => {
                let plan = served_plan(&self.config)?;
                self.check_served_plan(&plan)?;
                Some(ServedState {
                    codec,
                    plan,
                    cache: Mutex::new(None),
                    statistics: Mutex::new(ServedStatistics::default()),
                })
            }
        };
        Ok(())
    }

    /// The served representation's codec, in served mode.
    pub fn served_codec(&self) -> Option<&dyn MapCodec> {
        self.served.as_ref().map(|state| state.codec.as_ref())
    }

    /// The served representation's work so far, in served mode.
    pub fn served_statistics(&self) -> Result<Option<ServedStatistics>> {
        self.served
            .as_ref()
            .map(|state| {
                state
                    .statistics
                    .lock()
                    .map(|statistics| *statistics)
                    .map_err(|_| invalid("the served statistics are poisoned"))
            })
            .transpose()
    }

    /// `f` on this model in float, with served mode (and its computed values)
    /// restored afterwards.
    pub fn with_float_forward<T>(&mut self, f: impl FnOnce(&Self) -> Result<T>) -> Result<T> {
        let served = self.served.take();
        let result = f(self);
        self.served = served;
        result
    }

    /// Every served tensor's variables exist with the export's shapes, and
    /// every variable is served or folded into a served map.
    fn check_served_plan(&self, plan: &[ServedTensor]) -> Result<()> {
        let dims = |name: &str| -> Result<&[usize]> {
            self.variables
                .get(name)
                .map(|var| var.dims())
                .ok_or_else(|| invalid(format!("the served representation needs {name}")))
        };
        let mut covered = BTreeSet::new();
        for tensor in plan {
            for source in tensor.sources() {
                dims(source)?;
                covered.insert(source);
            }
            if let ServedKind::Map {
                gain, rows, cols, ..
            } = &tensor.kind
            {
                if dims(&tensor.source)? != [*rows, *cols]
                    || gain
                        .as_deref()
                        .map(|gain| dims(gain).map(|g| g != [*cols]))
                        .transpose()?
                        .unwrap_or(false)
                {
                    return Err(invalid(format!(
                        "{} differs from its served shape",
                        tensor.name
                    )));
                }
            }
        }
        if let Some(name) = self
            .variables
            .keys()
            .find(|name| !covered.contains(name.as_str()))
        {
            return Err(invalid(format!(
                "the served representation leaves {name} in float"
            )));
        }
        Ok(())
    }

    /// The served view of the current variables. Served tensors whose
    /// variables changed since the last view are recomputed, in parallel; the
    /// rest are reused.
    fn served_view(&self, state: &ServedState) -> Result<Arc<BTreeMap<String, Tensor>>> {
        let started = Instant::now();
        // No lock is held while the parallel work runs.
        let previous = state
            .cache
            .lock()
            .map_err(|_| invalid("the served cache is poisoned"))?
            .clone();
        let names: Vec<&String> = self.variables.keys().collect();
        let current: Vec<(Vec<f32>, bool)> = names
            .par_iter()
            .map(|name| -> Result<(Vec<f32>, bool)> {
                let values = self.variables[*name]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let changed = previous
                    .as_ref()
                    .and_then(|cache| cache.sources.get(*name))
                    .is_none_or(|old| !same_bits(old, &values));
                Ok((values, changed))
            })
            .collect::<Result<_>>()?;
        let changed: BTreeSet<&str> = names
            .iter()
            .zip(&current)
            .filter(|(_, (_, changed))| *changed)
            .map(|(name, _)| name.as_str())
            .collect();
        let mut statistics = ServedStatistics {
            checks: 1,
            ..ServedStatistics::default()
        };
        let view = match &previous {
            Some(cache) if changed.is_empty() => cache.tensors.clone(),
            _ => {
                let sources: BTreeMap<String, Vec<f32>> = names
                    .iter()
                    .map(|name| (*name).clone())
                    .zip(current.into_iter().map(|(values, _)| values))
                    .collect();
                let stale: Vec<&ServedTensor> = state
                    .plan
                    .iter()
                    .filter(|tensor| {
                        previous.is_none() || tensor.sources().any(|s| changed.contains(s))
                    })
                    .collect();
                let rebuilt: Vec<(String, Tensor)> = stale
                    .par_iter()
                    .map(|tensor| -> Result<(String, Tensor)> {
                        let values = tensor.served_values(&sources, state.codec.as_ref())?;
                        Ok((tensor.name.clone(), self.straight_through(tensor, values)?))
                    })
                    .collect::<Result<_>>()?;
                let mut tensors = previous
                    .as_ref()
                    .map(|cache| (*cache.tensors).clone())
                    .unwrap_or_default();
                tensors.extend(rebuilt);
                let tensors = Arc::new(tensors);
                *state
                    .cache
                    .lock()
                    .map_err(|_| invalid("the served cache is poisoned"))? = Some(ServedCache {
                    sources: Arc::new(sources),
                    tensors: tensors.clone(),
                });
                statistics.refreshes = 1;
                statistics.tensors = stale.len();
                tensors
            }
        };
        let mut total = state
            .statistics
            .lock()
            .map_err(|_| invalid("the served statistics are poisoned"))?;
        total.checks += statistics.checks;
        total.refreshes += statistics.refreshes;
        total.tensors += statistics.tensors;
        total.seconds += started.elapsed().as_secs_f64();
        Ok(view)
    }

    /// `values` in the forward pass, with the gradient of `tensor`'s float
    /// expression: its variable, times its norm gain for a folded map.
    fn straight_through(&self, tensor: &ServedTensor, values: Vec<f32>) -> Result<Tensor> {
        let variable = |name: &str| {
            self.variables
                .get(name)
                .map(Var::as_tensor)
                .ok_or_else(|| invalid(format!("missing stack variable {name}")))
        };
        let source = variable(&tensor.source)?;
        let input = match &tensor.kind {
            ServedKind::Map {
                gain: Some(gain), ..
            } => source.broadcast_mul(variable(gain)?)?,
            _ => source.clone(),
        };
        let served = Tensor::from_vec(values, source.shape(), source.device())?;
        Ok(input.apply_op2(&served, StraightThrough)?)
    }
}

/// Matrices take weight decay; norms, biases, decays, age tables, the
/// convolution taps and the Lorentz scalars do not.
fn decayed(name: &str, rank: usize) -> bool {
    name.ends_with(".weight") && rank == 2 && !name.ends_with("conv.weight")
}

/// One update's constants, in f32 as Candle's `affine` applies them.
struct AdamConstants {
    scale: f32,
    beta1: f32,
    rest1: f32,
    beta2: f32,
    rest2: f32,
    correct1: f32,
    correct2: f32,
    epsilon: f32,
    keep: f32,
    lr: f32,
}

/// The AdamW step of one variable in one parallel pass. It performs the same
/// f32 operations, in the same order, as the composition of Candle operations
/// it replaced (kept in the tests), so updates are bit-identical.
fn adam_step(
    parameters: &mut [f32],
    gradient: &[f32],
    first_moment: &mut [f32],
    second_moment: &mut [f32],
    c: &AdamConstants,
) {
    const CHUNK: usize = 1 << 14;
    parameters
        .par_chunks_mut(CHUNK)
        .zip(first_moment.par_chunks_mut(CHUNK))
        .zip(second_moment.par_chunks_mut(CHUNK))
        .zip(gradient.par_chunks(CHUNK))
        .for_each(|(((p, m), v), g)| {
            for i in 0..p.len() {
                let grad = g[i] * c.scale + 0.0;
                m[i] = (m[i] * c.beta1 + 0.0) + (grad * c.rest1 + 0.0);
                v[i] = (v[i] * c.beta2 + 0.0) + ((grad * grad) * c.rest2 + 0.0);
                let step = (m[i] * c.correct1 + 0.0)
                    / (((v[i] * c.correct2 + 0.0).sqrt() * 1.0) + c.epsilon);
                p[i] = (p[i] * c.keep + 0.0) - (step * c.lr + 0.0);
            }
        });
}

/// AdamW with global gradient-norm clipping and resumable moments.
pub struct StackAdamW {
    pub beta1: f64,
    pub beta2: f64,
    pub epsilon: f64,
    pub weight_decay: f64,
    pub clip: f64,
    pub step: usize,
    moments: BTreeMap<String, (Var, Var)>,
}

impl StackAdamW {
    pub fn new(model: &StackModel, weight_decay: f64, clip: f64) -> Result<Self> {
        let mut moments = BTreeMap::new();
        for (name, var) in model.variables() {
            moments.insert(
                name.clone(),
                (
                    Var::zeros(var.shape(), DType::F32, model.device())?,
                    Var::zeros(var.shape(), DType::F32, model.device())?,
                ),
            );
        }
        Ok(Self {
            beta1: 0.9,
            beta2: 0.95,
            epsilon: 1e-8,
            weight_decay,
            clip,
            step: 0,
            moments,
        })
    }

    /// One update at learning rate `lr`; returns the gradient norm before clipping.
    pub fn update(
        &mut self,
        model: &StackModel,
        grads: &candle_core::backprop::GradStore,
        lr: f64,
    ) -> Result<f64> {
        let mut total = 0f64;
        for var in model.variables().values() {
            if let Some(grad) = grads.get(var.as_tensor()) {
                total += f64::from(grad.sqr()?.sum_all()?.to_scalar::<f32>()?);
            }
        }
        let norm = total.sqrt();
        if !norm.is_finite() {
            return Err(invalid("nonfinite gradient norm"));
        }
        let scale = if self.clip > 0.0 && norm > self.clip {
            self.clip / norm
        } else {
            1.0
        };
        self.step += 1;
        let first = 1.0 - self.beta1.powi(self.step as i32);
        let second = 1.0 - self.beta2.powi(self.step as i32);
        for (name, var) in model.variables() {
            let Some(grad) = grads.get(var.as_tensor()) else {
                continue;
            };
            let (m, v) = self
                .moments
                .get(name)
                .ok_or_else(|| invalid(format!("missing moments for {name}")))?;
            let keep = if decayed(name, var.rank()) {
                1.0 - lr * self.weight_decay
            } else {
                1.0
            };
            let constants = AdamConstants {
                scale: scale as f32,
                beta1: self.beta1 as f32,
                rest1: (1.0 - self.beta1) as f32,
                beta2: self.beta2 as f32,
                rest2: (1.0 - self.beta2) as f32,
                correct1: (1.0 / first) as f32,
                correct2: (1.0 / second) as f32,
                epsilon: self.epsilon as f32,
                keep: keep as f32,
                lr: lr as f32,
            };
            let mut parameters = var.as_tensor().flatten_all()?.to_vec1::<f32>()?;
            let gradient = grad.flatten_all()?.to_vec1::<f32>()?;
            let mut first_moment = m.as_tensor().flatten_all()?.to_vec1::<f32>()?;
            let mut second_moment = v.as_tensor().flatten_all()?.to_vec1::<f32>()?;
            adam_step(
                &mut parameters,
                &gradient,
                &mut first_moment,
                &mut second_moment,
                &constants,
            );
            let (shape, device) = (var.shape().clone(), var.device().clone());
            var.set(&Tensor::from_vec(parameters, &shape, &device)?)?;
            m.set(&Tensor::from_vec(first_moment, &shape, &device)?)?;
            v.set(&Tensor::from_vec(second_moment, &shape, &device)?)?;
        }
        Ok(norm)
    }

    pub fn save(&self, directory: &Path) -> Result<()> {
        fs::create_dir_all(directory)?;
        let mut tensors = std::collections::HashMap::new();
        for (name, (m, v)) in &self.moments {
            tensors.insert(format!("m.{name}"), m.as_tensor().clone());
            tensors.insert(format!("v.{name}"), v.as_tensor().clone());
        }
        candle_core::safetensors::save(&tensors, directory.join("optimizer.safetensors"))?;
        fs::write(
            directory.join("optimizer.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "step": self.step,
                "beta1": self.beta1,
                "beta2": self.beta2,
                "epsilon": self.epsilon,
                "weight_decay": self.weight_decay,
                "clip": self.clip,
            }))?,
        )?;
        Ok(())
    }

    pub fn load(directory: &Path, model: &StackModel) -> Result<Self> {
        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.join("optimizer.json"))?)?;
        let number = |key: &str| -> Result<f64> {
            meta[key]
                .as_f64()
                .ok_or_else(|| invalid(format!("optimizer state lacks {key}")))
        };
        let mut optimizer = Self::new(model, number("weight_decay")?, number("clip")?)?;
        optimizer.beta1 = number("beta1")?;
        optimizer.beta2 = number("beta2")?;
        optimizer.epsilon = number("epsilon")?;
        optimizer.step = meta["step"]
            .as_u64()
            .ok_or_else(|| invalid("optimizer state lacks step"))?
            as usize;
        let tensors = candle_core::safetensors::load(
            directory.join("optimizer.safetensors"),
            model.device(),
        )?;
        for (name, (m, v)) in &optimizer.moments {
            for (prefix, var) in [("m", m), ("v", v)] {
                let tensor = tensors
                    .get(&format!("{prefix}.{name}"))
                    .ok_or_else(|| invalid(format!("optimizer state lacks {prefix}.{name}")))?;
                if tensor.dims() != var.dims() {
                    return Err(invalid(format!("optimizer state shape differs for {name}")));
                }
                var.set(tensor)?;
            }
        }
        Ok(optimizer)
    }
}

// ---------------------------------------------------------------------------
// Quaternion transport scan.

#[inline]
fn quaternion_product(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

#[inline]
fn conjugate(a: [f32; 4]) -> [f32; 4] {
    [a[0], -a[1], -a[2], -a[3]]
}

#[inline]
fn quad(values: &[f32], offset: usize) -> [f32; 4] {
    [
        values[offset],
        values[offset + 1],
        values[offset + 2],
        values[offset + 3],
    ]
}

fn contiguous<'a>(storage: &'a CpuStorage, layout: &Layout) -> candle_core::Result<&'a [f32]> {
    let values = storage.as_slice::<f32>()?;
    match layout.contiguous_offsets() {
        Some((start, end)) => Ok(&values[start..end]),
        None => candle_core::bail!("geometric stack kernels need contiguous inputs"),
    }
}

/// `h_t = q_t * h_{t-1} + b_t` for every lane, with `h_{-1} = 0`, where `*`
/// is the Hamilton product. Inputs and output are [batch, time, lanes, 4].
struct QuaternionScan;

impl CustomOp2 for QuaternionScan {
    fn name(&self) -> &'static str {
        "quaternion-scan"
    }

    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let (batch, time, lanes, four) = l1.shape().dims4()?;
        if four != 4 || l2.shape() != l1.shape() {
            candle_core::bail!("quaternion scan needs matching [batch, time, lanes, 4] inputs");
        }
        let (transition, drive) = (contiguous(s1, l1)?, contiguous(s2, l2)?);
        let block = time * lanes * 4;
        let mut state = vec![0f32; batch * block];
        state
            .par_chunks_mut(block)
            .enumerate()
            .for_each(|(window, out)| {
                let base = window * block;
                let mut previous = vec![[0f32; 4]; lanes];
                for t in 0..time {
                    for (lane, held) in previous.iter_mut().enumerate() {
                        let offset = (t * lanes + lane) * 4;
                        let moved = quaternion_product(quad(transition, base + offset), *held);
                        let next = [
                            moved[0] + drive[base + offset],
                            moved[1] + drive[base + offset + 1],
                            moved[2] + drive[base + offset + 2],
                            moved[3] + drive[base + offset + 3],
                        ];
                        out[offset..offset + 4].copy_from_slice(&next);
                        *held = next;
                    }
                }
            });
        Ok((CpuStorage::F32(state), l1.shape().clone()))
    }

    /// Reverse scan: `g_t = dh_t + conj(q_{t+1}) * g_{t+1}` is the total
    /// gradient of `h_t`; then `db_t = g_t` and `dq_t = g_t * conj(h_{t-1})`.
    fn bwd(
        &self,
        transition: &Tensor,
        _drive: &Tensor,
        state: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let (batch, time, lanes, _) = transition.dims4()?;
        let q = transition.flatten_all()?.to_vec1::<f32>()?;
        let h = state.flatten_all()?.to_vec1::<f32>()?;
        let dh = grad.flatten_all()?.to_vec1::<f32>()?;
        let block = time * lanes * 4;
        let mut dq = vec![0f32; batch * block];
        let mut db = vec![0f32; batch * block];
        dq.par_chunks_mut(block)
            .zip(db.par_chunks_mut(block))
            .enumerate()
            .for_each(|(window, (dq, db))| {
                let base = window * block;
                let mut carried = vec![[0f32; 4]; lanes];
                for t in (0..time).rev() {
                    for (lane, held) in carried.iter_mut().enumerate() {
                        let offset = (t * lanes + lane) * 4;
                        let mut total = quad(&dh, base + offset);
                        if t + 1 < time {
                            let next = (offset + lanes * 4) + base;
                            let back = quaternion_product(conjugate(quad(&q, next)), *held);
                            for c in 0..4 {
                                total[c] += back[c];
                            }
                        }
                        db[offset..offset + 4].copy_from_slice(&total);
                        if t > 0 {
                            let earlier = quad(&h, base + offset - lanes * 4);
                            dq[offset..offset + 4]
                                .copy_from_slice(&quaternion_product(total, conjugate(earlier)));
                        }
                        *held = total;
                    }
                }
            });
        let device = transition.device();
        Ok((
            Some(Tensor::from_vec(dq, transition.shape(), device)?),
            Some(Tensor::from_vec(db, transition.shape(), device)?),
        ))
    }
}

/// Runs the quaternion transport recurrence over whole windows.
pub fn quaternion_scan(transition: &Tensor, drive: &Tensor) -> Result<Tensor> {
    Ok(transition
        .contiguous()?
        .apply_op2(&drive.contiguous()?, QuaternionScan)?)
}

/// GELU, tanh approximation (Candle's `gelu`), and its derivative.
#[inline]
fn gelu(x: f32) -> (f32, f32) {
    const K: f32 = 0.797_884_6; // sqrt(2 / pi)
    const C: f32 = 0.044_715;
    let v = K * (x + C * x * x * x);
    let t = v.tanh();
    let value = 0.5 * x * (1.0 + t);
    let slope = 0.5 * (1.0 + t) + 0.5 * x * (1.0 - t * t) * K * (1.0 + 3.0 * C * x * x);
    (value, slope)
}

/// The recurrence mixer's core in one op: width-4 causal convolution, gated
/// decay, rotation normalization, the quaternion transport scan and the GELU
/// output gate, parallel over windows, with an exact backward.
///
/// Inputs: branches [batch, time, 2 width] (the scan drive `a`, then the gate
/// `g`); gates [batch, time, lanes (+ width with rotation)] (decay-gate logits,
/// then the raw rotation quaternions); parameters packed as the convolution
/// taps [4, width], its bias [width] and the lane decays [lanes]. Output:
/// `h * gelu(g)` [batch, time, width].
#[derive(Clone, Copy, Debug)]
struct RecurrenceCore {
    batch: usize,
    time: usize,
    width: usize,
    rotation: bool,
}

/// One lane's transition at one position.
#[derive(Clone, Copy, Default)]
struct Transition {
    /// Decay gate sigma(logit).
    opening: f32,
    lambda: f32,
    keep: f32,
    /// `1 - lambda^2` fell below the floor, so `keep` carries no gradient.
    clamped: bool,
    rotation: [f32; 4],
    norm: f32,
}

impl RecurrenceCore {
    fn lanes(&self) -> usize {
        self.width / 4
    }

    fn gate_width(&self) -> usize {
        self.lanes() + if self.rotation { self.width } else { 0 }
    }

    fn parameter_len(&self) -> usize {
        CONVOLUTION_WIDTH * self.width + self.width + self.lanes()
    }

    /// `log a` per lane: `-softplus(-decay)`.
    fn log_a(&self, parameters: &[f32]) -> Vec<f32> {
        let decay = &parameters[(CONVOLUTION_WIDTH + 1) * self.width..];
        decay
            .iter()
            .map(|&d| -((-f64::from(d)).exp().ln_1p()) as f32)
            .collect()
    }

    /// Forward pass of one window: convolved drives `c` [time, width], states
    /// `h` [time, width] and transitions [time, lanes].
    fn window(
        &self,
        branches: &[f32],
        gates: &[f32],
        parameters: &[f32],
        log_a: &[f32],
    ) -> (Vec<f32>, Vec<f32>, Vec<Transition>) {
        let (time, width, lanes, gate_width) =
            (self.time, self.width, self.lanes(), self.gate_width());
        let taps = &parameters[..CONVOLUTION_WIDTH * width];
        let bias = &parameters[CONVOLUTION_WIDTH * width..(CONVOLUTION_WIDTH + 1) * width];
        let mut drive = vec![0f32; time * width];
        let mut state = vec![0f32; time * width];
        let mut transitions = vec![Transition::default(); time * lanes];
        for t in 0..time {
            let c = &mut drive[t * width..(t + 1) * width];
            c.copy_from_slice(bias);
            for shift in 0..CONVOLUTION_WIDTH.min(t + 1) {
                let a = &branches[(t - shift) * 2 * width..(t - shift) * 2 * width + width];
                for ((c, &w), &a) in c
                    .iter_mut()
                    .zip(&taps[shift * width..(shift + 1) * width])
                    .zip(a)
                {
                    *c += w * a;
                }
            }
            let gate_row = &gates[t * gate_width..(t + 1) * gate_width];
            for lane in 0..lanes {
                let opening = sigmoid(gate_row[lane]);
                let lambda = (DECAY_EXPONENT as f32 * opening * log_a[lane]).exp();
                let complement = 1.0 - lambda * lambda;
                let (keep, clamped) = if complement < 1e-6 {
                    (1e-3, true)
                } else {
                    (complement.sqrt(), false)
                };
                let (rotation, norm) = if self.rotation {
                    let raw = quad(gate_row, lanes + 4 * lane);
                    let norm = (raw.iter().map(|v| v * v).sum::<f32>() + 1e-6).sqrt();
                    (
                        [raw[0] / norm, raw[1] / norm, raw[2] / norm, raw[3] / norm],
                        norm,
                    )
                } else {
                    ([1.0, 0.0, 0.0, 0.0], 1.0)
                };
                transitions[t * lanes + lane] = Transition {
                    opening,
                    lambda,
                    keep,
                    clamped,
                    rotation,
                    norm,
                };
                let q = rotation.map(|v| v * lambda);
                let previous = if t > 0 {
                    quad(&state, (t - 1) * width + 4 * lane)
                } else {
                    [0.0; 4]
                };
                let moved = quaternion_product(q, previous);
                let offset = t * width + 4 * lane;
                for k in 0..4 {
                    state[offset + k] = moved[k] + keep * drive[offset + k];
                }
            }
        }
        (drive, state, transitions)
    }
}

impl CustomOp3 for RecurrenceCore {
    fn name(&self) -> &'static str {
        "geometric-stack-recurrence"
    }

    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
        s3: &CpuStorage,
        l3: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let (branches, gates, parameters) = (
            contiguous(s1, l1)?,
            contiguous(s2, l2)?,
            contiguous(s3, l3)?,
        );
        let (time, width) = (self.time, self.width);
        if branches.len() != self.batch * time * 2 * width
            || gates.len() != self.batch * time * self.gate_width()
            || parameters.len() != self.parameter_len()
        {
            candle_core::bail!("recurrence core inputs have the wrong sizes");
        }
        let log_a = self.log_a(parameters);
        let mut out = vec![0f32; self.batch * time * width];
        out.par_chunks_mut(time * width)
            .enumerate()
            .for_each(|(window, out)| {
                let branches =
                    &branches[window * time * 2 * width..(window + 1) * time * 2 * width];
                let gates = &gates
                    [window * time * self.gate_width()..(window + 1) * time * self.gate_width()];
                let (_, state, _) = self.window(branches, gates, parameters, &log_a);
                for t in 0..time {
                    let g = &branches[t * 2 * width + width..(t + 1) * 2 * width];
                    for i in 0..width {
                        out[t * width + i] = state[t * width + i] * gelu(g[i]).0;
                    }
                }
            });
        Ok((CpuStorage::F32(out), Shape::from((self.batch, time, width))))
    }

    fn bwd(
        &self,
        branches: &Tensor,
        gates: &Tensor,
        parameters: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>, Option<Tensor>)> {
        let branch_values = branches.flatten_all()?.to_vec1::<f32>()?;
        let gate_values = gates.flatten_all()?.to_vec1::<f32>()?;
        let parameter_values = parameters.to_vec1::<f32>()?;
        let d_out = grad.flatten_all()?.to_vec1::<f32>()?;
        let (time, width, lanes, gate_width) =
            (self.time, self.width, self.lanes(), self.gate_width());
        let log_a = self.log_a(&parameter_values);
        let taps = &parameter_values[..CONVOLUTION_WIDTH * width];
        let exponent = DECAY_EXPONENT as f32;
        let mut d_branches = vec![0f32; branch_values.len()];
        let mut d_gates = vec![0f32; gate_values.len()];
        let partials: Vec<Vec<f64>> = d_branches
            .par_chunks_mut(time * 2 * width)
            .zip(d_gates.par_chunks_mut(time * gate_width))
            .enumerate()
            .map(|(window, (d_branch, d_gate))| {
                let branch =
                    &branch_values[window * time * 2 * width..(window + 1) * time * 2 * width];
                let gate =
                    &gate_values[window * time * gate_width..(window + 1) * time * gate_width];
                let dy = &d_out[window * time * width..(window + 1) * time * width];
                let (drive, state, transitions) =
                    self.window(branch, gate, &parameter_values, &log_a);
                // Partial parameter gradients: taps, bias, then log a per lane.
                let mut d_parameters = vec![0f64; self.parameter_len()];
                let mut d_log_a = vec![0f64; lanes];
                let mut carried = vec![[0f32; 4]; lanes];
                let mut d_drive = vec![0f32; width];
                for t in (0..time).rev() {
                    let g = &branch[t * 2 * width + width..(t + 1) * 2 * width];
                    for i in 0..width {
                        let (value, slope) = gelu(g[i]);
                        d_branch[t * 2 * width + width + i] =
                            dy[t * width + i] * state[t * width + i] * slope;
                        // Reuse d_drive as the direct state gradient for now.
                        d_drive[i] = dy[t * width + i] * value;
                    }
                    for (lane, held) in carried.iter_mut().enumerate() {
                        let offset = 4 * lane;
                        let mut total = quad(&d_drive, offset);
                        if t + 1 < time {
                            let next = transitions[(t + 1) * lanes + lane];
                            let q_next = next.rotation.map(|v| v * next.lambda);
                            let back = quaternion_product(conjugate(q_next), *held);
                            for k in 0..4 {
                                total[k] += back[k];
                            }
                        }
                        *held = total;
                        let transition = transitions[t * lanes + lane];
                        let c = quad(&drive, t * width + offset);
                        // x = keep c.
                        let d_keep: f32 = (0..4).map(|k| total[k] * c[k]).sum();
                        for k in 0..4 {
                            d_drive[offset + k] = transition.keep * total[k];
                        }
                        // q = lambda u, with h_{-1} = 0.
                        let dq = if t > 0 {
                            quaternion_product(
                                total,
                                conjugate(quad(&state, (t - 1) * width + offset)),
                            )
                        } else {
                            [0.0; 4]
                        };
                        let u = transition.rotation;
                        let mut d_lambda: f32 = (0..4).map(|k| dq[k] * u[k]).sum();
                        if !transition.clamped {
                            d_lambda -= d_keep * transition.lambda / transition.keep;
                        }
                        let d_log_lambda = d_lambda * transition.lambda;
                        let d_opening = d_log_lambda * exponent * log_a[lane];
                        d_log_a[lane] += f64::from(d_log_lambda * exponent * transition.opening);
                        d_gate[t * gate_width + lane] =
                            d_opening * transition.opening * (1.0 - transition.opening);
                        if self.rotation {
                            // u = raw / n, n = sqrt(|raw|^2 + eps).
                            let du = dq.map(|v| v * transition.lambda);
                            let projection: f32 = (0..4).map(|k| du[k] * u[k]).sum();
                            for k in 0..4 {
                                d_gate[t * gate_width + lanes + offset + k] =
                                    (du[k] - u[k] * projection) / transition.norm;
                            }
                        }
                    }
                    // Convolution: c_t = bias + sum_k taps_k * a_{t-k}.
                    for i in 0..width {
                        d_parameters[CONVOLUTION_WIDTH * width + i] += f64::from(d_drive[i]);
                    }
                    for shift in 0..CONVOLUTION_WIDTH.min(t + 1) {
                        let source = (t - shift) * 2 * width;
                        for i in 0..width {
                            d_parameters[shift * width + i] +=
                                f64::from(d_drive[i] * branch[source + i]);
                            d_branch[source + i] += taps[shift * width + i] * d_drive[i];
                        }
                    }
                }
                let decay_offset = (CONVOLUTION_WIDTH + 1) * width;
                d_parameters[decay_offset..decay_offset + lanes].copy_from_slice(&d_log_a);
                d_parameters
            })
            .collect();
        // d log a / d decay = sigma(-decay).
        let decay_offset = (CONVOLUTION_WIDTH + 1) * width;
        let mut d_parameters = vec![0f64; self.parameter_len()];
        for partial in &partials {
            for (a, b) in d_parameters.iter_mut().zip(partial) {
                *a += b;
            }
        }
        for lane in 0..lanes {
            let decay = f64::from(parameter_values[decay_offset + lane]);
            d_parameters[decay_offset + lane] *= 1.0 / (1.0 + decay.exp());
        }
        let d_parameters: Vec<f32> = d_parameters.into_iter().map(|v| v as f32).collect();
        Ok((
            Some(Tensor::from_vec(
                d_branches,
                branches.shape(),
                branches.device(),
            )?),
            Some(Tensor::from_vec(d_gates, gates.shape(), gates.device())?),
            Some(Tensor::from_vec(
                d_parameters,
                parameters.shape(),
                parameters.device(),
            )?),
        ))
    }
}

// ---------------------------------------------------------------------------
// Fused multi-head read.

/// Length of the packed auxiliary input of [`fused_read`]: NoRead logits
/// [batch, heads, time], then the age table [heads, time], then for Lorentz
/// the scales beta [heads] (already exponentiated) and offsets [heads].
pub fn fused_aux_len(
    batch: usize,
    heads: usize,
    time: usize,
    score: ReadScore,
    null: bool,
    age: bool,
) -> usize {
    (if null { batch * heads * time } else { 0 })
        + (if age { heads * time } else { 0 })
        + (if score == ReadScore::Lorentz {
            2 * heads
        } else {
            0
        })
}

/// Dot product with sixteen independent partial sums, so the compiler can
/// vectorize it without reordering a single floating-point accumulator.
#[inline]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    let mut partial = [0f32; 16];
    let mut chunks_a = a.chunks_exact(16);
    let mut chunks_b = b.chunks_exact(16);
    for (x, y) in chunks_a.by_ref().zip(chunks_b.by_ref()) {
        for i in 0..16 {
            partial[i] += x[i] * y[i];
        }
    }
    let mut total: f32 = partial.iter().sum();
    for (x, y) in chunks_a.remainder().iter().zip(chunks_b.remainder()) {
        total += x * y;
    }
    total
}

/// `y += alpha * x`.
#[inline]
fn axpy(alpha: f32, x: &[f32], y: &mut [f32]) {
    for (y, &x) in y.iter_mut().zip(x) {
        *y += alpha * x;
    }
}

/// The whole multi-head read in one op: inner products, the score transform,
/// the causal mask, the NoRead slot, the softmax and the value mix, parallel
/// over (window, head) blocks, with an exact backward that recomputes the
/// probabilities. Inner loops run over positions or features as `axpy`, so
/// they vectorize.
#[derive(Clone, Copy, Debug)]
struct FusedRead {
    batch: usize,
    heads: usize,
    time: usize,
    key: usize,
    value: usize,
    score: ReadScore,
    null: bool,
    age: bool,
    /// Rotary position embedding of queries and keys (the control's attention).
    rope: bool,
}

/// One (window, head) block: queries and keys (rotated with RoPE) in row
/// layout, keys and values also transposed, and the per-head parameters.
struct Block<'a> {
    query: Vec<f32>,
    key_rows: Vec<f32>,
    key_columns: Vec<f32>,
    value_rows: Vec<f32>,
    value_columns: Vec<f32>,
    null: Option<&'a [f32]>,
    age: Option<&'a [f32]>,
    /// Lifts sqrt(1 + |x|^2) of queries and keys, for Lorentz.
    query_lift: Vec<f64>,
    key_lift: Vec<f64>,
    beta: f64,
    offset: f64,
}

/// Scratch for a tile of `TILE` rows, each `time` wide: the Lorentz
/// excesses and distances, and in the backward pass the probability
/// gradients.
struct Scratch {
    excess: Vec<f64>,
    distance: Vec<f64>,
    dp: Vec<f32>,
}

impl Scratch {
    fn new(time: usize) -> Self {
        Self {
            excess: vec![0f64; TILE * time],
            distance: vec![0f64; TILE * time],
            dp: vec![0f32; TILE * time],
        }
    }
}

/// Rows of a register tile of the read's products.
const TILE: usize = 4;
/// Columns of a register tile.
const LANES: usize = 16;

/// `out[r * out_stride + j] = sum_i a[r * a_stride + i] b[i * b_stride + j]`
/// for rows `r < rows` (at most `TILE`), columns `j < columns` and `i <
/// count`, each sum taken in ascending `i` from zero, exactly as an `axpy`
/// loop over `i` takes it. Full tiles hold their sums in registers.
#[allow(clippy::too_many_arguments)]
fn tile_product(
    a: &[f32],
    a_stride: usize,
    rows: usize,
    b: &[f32],
    b_stride: usize,
    columns: usize,
    count: usize,
    out: &mut [f32],
    out_stride: usize,
) {
    let mut j0 = 0;
    while j0 < columns {
        let width = LANES.min(columns - j0);
        if rows == TILE && width == LANES {
            let mut acc = [[0f32; LANES]; TILE];
            for i in 0..count {
                let row = &b[i * b_stride + j0..i * b_stride + j0 + LANES];
                let x = [
                    a[i],
                    a[a_stride + i],
                    a[2 * a_stride + i],
                    a[3 * a_stride + i],
                ];
                for (c, &y) in row.iter().enumerate() {
                    acc[0][c] += x[0] * y;
                    acc[1][c] += x[1] * y;
                    acc[2][c] += x[2] * y;
                    acc[3][c] += x[3] * y;
                }
            }
            for (r, acc) in acc.iter().enumerate() {
                out[r * out_stride + j0..r * out_stride + j0 + LANES].copy_from_slice(acc);
            }
        } else {
            for r in 0..rows {
                let target = &mut out[r * out_stride + j0..r * out_stride + j0 + width];
                target.fill(0.0);
                for i in 0..count {
                    axpy(
                        a[r * a_stride + i],
                        &b[i * b_stride + j0..i * b_stride + j0 + width],
                        target,
                    );
                }
            }
        }
        j0 += width;
    }
}

/// The causal product of a tile: row `r` (position `t0 + r`) of `out` gets
/// `sum_{j <= t0 + r} a[r * a_stride + j] b[j * b_stride + ..width]`, summed in
/// ascending `j` from zero as `axpy` over `j` does; `out` rows are `width` wide.
#[allow(clippy::too_many_arguments)]
fn causal_rows(
    a: &[f32],
    a_stride: usize,
    rows: usize,
    t0: usize,
    b: &[f32],
    b_stride: usize,
    width: usize,
    out: &mut [f32],
) {
    let shared = t0 + 1;
    let mut c0 = 0;
    while c0 < width {
        let lanes = LANES.min(width - c0);
        if rows == TILE && lanes == LANES {
            let mut acc = [[0f32; LANES]; TILE];
            for j in 0..shared {
                let row = &b[j * b_stride + c0..j * b_stride + c0 + LANES];
                let x = [
                    a[j],
                    a[a_stride + j],
                    a[2 * a_stride + j],
                    a[3 * a_stride + j],
                ];
                for (c, &y) in row.iter().enumerate() {
                    acc[0][c] += x[0] * y;
                    acc[1][c] += x[1] * y;
                    acc[2][c] += x[2] * y;
                    acc[3][c] += x[3] * y;
                }
            }
            for (r, acc) in acc.iter_mut().enumerate() {
                for j in shared..=t0 + r {
                    let row = &b[j * b_stride + c0..j * b_stride + c0 + LANES];
                    let x = a[r * a_stride + j];
                    for (slot, &y) in acc.iter_mut().zip(row) {
                        *slot += x * y;
                    }
                }
                out[r * width + c0..r * width + c0 + LANES].copy_from_slice(acc);
            }
        } else {
            for r in 0..rows {
                let target = &mut out[r * width + c0..r * width + c0 + lanes];
                target.fill(0.0);
                for j in 0..=t0 + r {
                    axpy(
                        a[r * a_stride + j],
                        &b[j * b_stride + c0..j * b_stride + c0 + lanes],
                        target,
                    );
                }
            }
        }
        c0 += lanes;
    }
}

/// The transposed causal product of a whole block: row `j` of `out` is
/// `sum_{t >= j} a[t * time + j] x[t * width + ..width]`, summed in ascending
/// `t` from zero, as successive rows' `axpy` calls add them. Full tiles of
/// four output rows by sixteen columns hold their sums in registers.
fn causal_transpose(a: &[f32], x: &[f32], time: usize, width: usize, out: &mut [f32]) {
    for j0 in (0..time).step_by(TILE) {
        let rows = TILE.min(time - j0);
        let mut c0 = 0;
        while c0 < width {
            let lanes = LANES.min(width - c0);
            if rows == TILE && lanes == LANES {
                let mut acc = [[0f32; LANES]; TILE];
                // The triangle: position t reaches rows j <= t of the tile.
                for t in j0..j0 + TILE {
                    let row = &x[t * width + c0..t * width + c0 + LANES];
                    for (r, acc) in acc.iter_mut().enumerate().take(t - j0 + 1) {
                        let g = a[t * time + j0 + r];
                        for (slot, &y) in acc.iter_mut().zip(row) {
                            *slot += g * y;
                        }
                    }
                }
                for t in j0 + TILE..time {
                    let row = &x[t * width + c0..t * width + c0 + LANES];
                    let g = &a[t * time + j0..t * time + j0 + TILE];
                    for (c, &y) in row.iter().enumerate() {
                        acc[0][c] += g[0] * y;
                        acc[1][c] += g[1] * y;
                        acc[2][c] += g[2] * y;
                        acc[3][c] += g[3] * y;
                    }
                }
                for (r, acc) in acc.iter().enumerate() {
                    out[(j0 + r) * width + c0..(j0 + r) * width + c0 + LANES].copy_from_slice(acc);
                }
            } else {
                for j in j0..j0 + rows {
                    let target = &mut out[j * width + c0..j * width + c0 + lanes];
                    target.fill(0.0);
                    for t in j..time {
                        axpy(
                            a[t * time + j],
                            &x[t * width + c0..t * width + c0 + lanes],
                            target,
                        );
                    }
                }
            }
            c0 += lanes;
        }
    }
}

impl FusedRead {
    fn width(&self) -> usize {
        self.key + self.value
    }

    fn block<'a>(
        &self,
        query: &[f32],
        kv: &[f32],
        aux: &'a [f32],
        tables: Option<&(Vec<f32>, Vec<f32>)>,
        index: usize,
    ) -> Block<'a> {
        let (time, key, value, width, head) = (
            self.time,
            self.key,
            self.value,
            self.width(),
            index % self.heads,
        );
        let mut query = query[index * time * key..(index + 1) * time * key].to_vec();
        let kv = &kv[index * time * width..(index + 1) * time * width];
        let mut key_rows = Vec::with_capacity(time * key);
        let mut value_rows = Vec::with_capacity(time * value);
        for t in 0..time {
            key_rows.extend_from_slice(&kv[t * width..t * width + key]);
            value_rows.extend_from_slice(&kv[t * width + key..(t + 1) * width]);
        }
        if let Some(tables) = tables {
            for t in 0..time {
                rope_rotate(&mut query[t * key..(t + 1) * key], tables, t, false);
                rope_rotate(&mut key_rows[t * key..(t + 1) * key], tables, t, false);
            }
        }
        let transpose = |rows: &[f32], columns: usize| {
            let mut out = vec![0f32; rows.len()];
            for t in 0..time {
                for i in 0..columns {
                    out[i * time + t] = rows[t * columns + i];
                }
            }
            out
        };
        let key_columns = transpose(&key_rows, key);
        let value_columns = transpose(&value_rows, value);
        let mut cursor = 0;
        let null = if self.null {
            cursor = self.batch * self.heads * time;
            Some(&aux[index * time..(index + 1) * time])
        } else {
            None
        };
        let age = if self.age {
            let slice = &aux[cursor + head * time..cursor + (head + 1) * time];
            cursor += self.heads * time;
            Some(slice)
        } else {
            None
        };
        let lift = |row: &[f32]| (1.0 + f64::from(dot(row, row))).sqrt();
        let (mut query_lift, mut key_lift, mut beta, mut offset) =
            (Vec::new(), Vec::new(), 0.0, 0.0);
        if self.score == ReadScore::Lorentz {
            query_lift = (0..time)
                .map(|t| lift(&query[t * key..(t + 1) * key]))
                .collect();
            key_lift = (0..time)
                .map(|t| lift(&key_rows[t * key..(t + 1) * key]))
                .collect();
            beta = f64::from(aux[cursor + head]);
            offset = f64::from(aux[cursor + self.heads + head]);
        }
        Block {
            query,
            key_rows,
            key_columns,
            value_rows,
            value_columns,
            null,
            age,
            query_lift,
            key_lift,
            beta,
            offset,
        }
    }

    /// Row `t`'s key probabilities, in place of its inner products
    /// `row[..=t]`; for Lorentz, `z - 1` in `excess[..=t]` and the distances
    /// in `distance[..=t]`. Returns the NoRead probability.
    fn transform(
        &self,
        block: &Block,
        t: usize,
        row: &mut [f32],
        excess: &mut [f64],
        distance: &mut [f64],
    ) -> f32 {
        let row = &mut row[..=t];
        let scale = 1.0 / (self.key as f32).sqrt();
        let mut maximum = block.null.map_or(f32::NEG_INFINITY, |null| null[t]);
        for j in 0..=t {
            let age = block.age.map_or(0.0, |age| age[t - j]);
            let score = match self.score {
                ReadScore::Dot => row[j] * scale + age,
                ReadScore::Lorentz => {
                    let e = block.query_lift[t] * block.key_lift[j] - f64::from(row[j]) - 1.0;
                    let d = lorentz_distance(e);
                    excess[j] = e;
                    distance[j] = d;
                    (-block.beta * (d - block.offset)) as f32 + age
                }
            };
            row[j] = score;
            maximum = maximum.max(score);
        }
        let null_weight = block.null.map_or(0.0, |null| (null[t] - maximum).exp());
        let mut total = null_weight;
        for value in row.iter_mut() {
            *value = (*value - maximum).exp();
            total += *value;
        }
        let inverse = 1.0 / total;
        for value in row.iter_mut() {
            *value *= inverse;
        }
        null_weight * inverse
    }

    /// Probabilities of the tile of rows `t0..t0 + rows` in `probabilities`
    /// (rows `time` wide), with the Lorentz excesses and distances in
    /// `scratch`; returns the NoRead probabilities.
    fn tile(
        &self,
        block: &Block,
        t0: usize,
        rows: usize,
        probabilities: &mut [f32],
        scratch: &mut Scratch,
    ) -> [f32; TILE] {
        let (key, time) = (self.key, self.time);
        tile_product(
            &block.query[t0 * key..],
            key,
            rows,
            &block.key_columns,
            time,
            t0 + rows,
            key,
            probabilities,
            time,
        );
        let mut null = [0f32; TILE];
        for (r, null) in null.iter_mut().enumerate().take(rows) {
            let span = r * time..(r + 1) * time;
            *null = self.transform(
                block,
                t0 + r,
                &mut probabilities[span.clone()],
                &mut scratch.excess[span.clone()],
                &mut scratch.distance[span],
            );
        }
        null
    }
}

/// RoPE cosine and sine tables [time, key / 2], theta 10,000, #1017's layout.
fn rope_tables(time: usize, key: usize) -> (Vec<f32>, Vec<f32>) {
    let half = key / 2;
    let mut cosine = Vec::with_capacity(time * half);
    let mut sine = Vec::with_capacity(time * half);
    for position in 0..time {
        for index in 0..half {
            let frequency = ROPE_THETA.powf(-((2 * index) as f64) / key as f64);
            let angle = position as f64 * frequency;
            cosine.push(angle.cos() as f32);
            sine.push(angle.sin() as f32);
        }
    }
    (cosine, sine)
}

/// Rotates one row in place by position `t` (rotate-half pairing `i`, `i + half`);
/// `inverse` applies the transpose, which also maps gradients back.
#[inline]
fn rope_rotate(row: &mut [f32], tables: &(Vec<f32>, Vec<f32>), t: usize, inverse: bool) {
    let half = row.len() / 2;
    let (cosine, sine) = (
        &tables.0[t * half..(t + 1) * half],
        &tables.1[t * half..(t + 1) * half],
    );
    let sign = if inverse { -1.0 } else { 1.0 };
    for i in 0..half {
        let (a, b) = (row[i], row[i + half]);
        let (c, s) = (cosine[i], sign * sine[i]);
        row[i] = a * c - b * s;
        row[i + half] = b * c + a * s;
    }
}

/// arcosh(1 + e), clamped at `LORENTZ_MIN_EXCESS`.
#[inline]
fn lorentz_distance(excess: f64) -> f64 {
    let e = excess.max(LORENTZ_MIN_EXCESS);
    (e + (e * (e + 2.0)).sqrt()).ln_1p()
}

impl CustomOp3 for FusedRead {
    fn name(&self) -> &'static str {
        "geometric-stack-read"
    }

    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
        s3: &CpuStorage,
        l3: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let (query, kv, aux) = (
            contiguous(s1, l1)?,
            contiguous(s2, l2)?,
            contiguous(s3, l3)?,
        );
        let (time, value) = (self.time, self.value);
        let tables = self.rope.then(|| rope_tables(time, self.key));
        let mut out = vec![0f32; self.batch * self.heads * time * value];
        out.par_chunks_mut(time * value)
            .enumerate()
            .for_each(|(index, out)| {
                let block = self.block(query, kv, aux, tables.as_ref(), index);
                let mut scratch = Scratch::new(time);
                let mut probabilities = vec![0f32; TILE * time];
                for t0 in (0..time).step_by(TILE) {
                    let rows = TILE.min(time - t0);
                    self.tile(&block, t0, rows, &mut probabilities, &mut scratch);
                    causal_rows(
                        &probabilities,
                        time,
                        rows,
                        t0,
                        &block.value_rows,
                        value,
                        value,
                        &mut out[t0 * value..],
                    );
                }
            });
        Ok((
            CpuStorage::F32(out),
            Shape::from((self.batch, self.heads, time, value)),
        ))
    }

    fn bwd(
        &self,
        query: &Tensor,
        kv: &Tensor,
        aux: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>, Option<Tensor>)> {
        let q_values = query.flatten_all()?.to_vec1::<f32>()?;
        let kv_values = kv.flatten_all()?.to_vec1::<f32>()?;
        let aux_values = aux.flatten_all()?.to_vec1::<f32>()?;
        let d_out = grad.flatten_all()?.to_vec1::<f32>()?;
        let (time, key, value, width) = (self.time, self.key, self.value, self.width());
        let lorentz = self.score == ReadScore::Lorentz;
        let scale = 1.0 / (key as f64).sqrt();
        let tables = self.rope.then(|| rope_tables(time, key));
        let mut dq = vec![0f32; q_values.len()];
        let mut dkv = vec![0f32; kv_values.len()];
        struct Partial {
            dnull: Vec<f64>,
            dage: Vec<f64>,
            dbeta: f64,
            doffset: f64,
        }
        let partials: Vec<Partial> = dq
            .par_chunks_mut(time * key)
            .zip(dkv.par_chunks_mut(time * width))
            .enumerate()
            .map(|(index, (dq, dkv))| {
                let block = self.block(&q_values, &kv_values, &aux_values, tables.as_ref(), index);
                let d_block = &d_out[index * time * value..(index + 1) * time * value];
                let mut partial = Partial {
                    dnull: vec![0.0; if self.null { time } else { 0 }],
                    dage: vec![0.0; if self.age { time } else { 0 }],
                    dbeta: 0.0,
                    doffset: 0.0,
                };
                let mut scratch = Scratch::new(time);
                // The whole block's probabilities and inner-product gradients,
                // for the key and value gradients after the rows.
                let mut probabilities = vec![0f32; time * time];
                let mut inner_grads = vec![0f32; time * time];
                // Keys and values in row layout; keys are mapped back through
                // RoPE at the end.
                let mut dk_rows = vec![0f32; time * key];
                let mut dv_rows = vec![0f32; time * value];
                // Lorentz: coefficients of each key's own direction, applied once.
                let mut key_self = vec![0f64; if lorentz { time } else { 0 }];
                for t0 in (0..time).step_by(TILE) {
                    let rows = TILE.min(time - t0);
                    let null_probability = self.tile(
                        &block,
                        t0,
                        rows,
                        &mut probabilities[t0 * time..],
                        &mut scratch,
                    );
                    tile_product(
                        &d_block[t0 * value..],
                        value,
                        rows,
                        &block.value_columns,
                        time,
                        t0 + rows,
                        value,
                        &mut scratch.dp,
                        time,
                    );
                    let mut query_self = [0f64; TILE];
                    for r in 0..rows {
                        let t = t0 + r;
                        let span = r * time..r * time + t + 1;
                        let p = &probabilities[t * time..t * time + t + 1];
                        let dp = &scratch.dp[span.clone()];
                        let row_dot: f64 = p
                            .iter()
                            .zip(dp.iter())
                            .map(|(&a, &b)| f64::from(a) * f64::from(b))
                            .sum();
                        if self.null {
                            partial.dnull[t] = -f64::from(null_probability[r]) * row_dot;
                        }
                        let (excess, distance) = (
                            &scratch.excess[span.clone()],
                            &scratch.distance[span.clone()],
                        );
                        let inner_grad = &mut inner_grads[t * time..t * time + t + 1];
                        for j in 0..=t {
                            let ds = f64::from(p[j]) * (f64::from(dp[j]) - row_dot);
                            if self.age {
                                partial.dage[t - j] += ds;
                            }
                            inner_grad[j] = match self.score {
                                ReadScore::Dot => (ds * scale) as f32,
                                ReadScore::Lorentz => {
                                    let e = excess[j];
                                    partial.dbeta -= ds * (distance[j] - block.offset);
                                    partial.doffset += ds * block.beta;
                                    if e > LORENTZ_MIN_EXCESS {
                                        // e = lift_q lift_k - <q, k> - 1 and lift = sqrt(1 + |x|^2).
                                        let de = -block.beta * ds / (e * (e + 2.0)).sqrt();
                                        let (lq, lk) = (block.query_lift[t], block.key_lift[j]);
                                        query_self[r] += de * lk / lq;
                                        key_self[j] += de * lq / lk;
                                        -de as f32
                                    } else {
                                        0.0
                                    }
                                }
                            };
                        }
                    }
                    causal_rows(
                        &inner_grads[t0 * time..],
                        time,
                        rows,
                        t0,
                        &block.key_rows,
                        key,
                        key,
                        &mut dq[t0 * key..],
                    );
                    if lorentz {
                        for (r, &coefficient) in query_self.iter().enumerate().take(rows) {
                            if coefficient != 0.0 {
                                let t = t0 + r;
                                axpy(
                                    coefficient as f32,
                                    &block.query[t * key..(t + 1) * key],
                                    &mut dq[t * key..(t + 1) * key],
                                );
                            }
                        }
                    }
                }
                causal_transpose(&inner_grads, &block.query, time, key, &mut dk_rows);
                causal_transpose(&probabilities, d_block, time, value, &mut dv_rows);
                for (j, &coefficient) in key_self.iter().enumerate() {
                    if coefficient != 0.0 {
                        let (rows, grads) = (
                            &block.key_rows[j * key..(j + 1) * key],
                            &mut dk_rows[j * key..(j + 1) * key],
                        );
                        axpy(coefficient as f32, rows, grads);
                    }
                }
                if let Some(tables) = tables.as_ref() {
                    // Gradients of the rotated rows map back by the inverse rotation.
                    for t in 0..time {
                        rope_rotate(&mut dq[t * key..(t + 1) * key], tables, t, true);
                        rope_rotate(&mut dk_rows[t * key..(t + 1) * key], tables, t, true);
                    }
                }
                for t in 0..time {
                    dkv[t * width..t * width + key]
                        .copy_from_slice(&dk_rows[t * key..(t + 1) * key]);
                    dkv[t * width + key..(t + 1) * width]
                        .copy_from_slice(&dv_rows[t * value..(t + 1) * value]);
                }
                partial
            })
            .collect();
        let mut d_aux = vec![0f64; aux_values.len()];
        let null_len = if self.null {
            self.batch * self.heads * time
        } else {
            0
        };
        let age_len = if self.age { self.heads * time } else { 0 };
        for (index, partial) in partials.iter().enumerate() {
            let head = index % self.heads;
            for (t, &v) in partial.dnull.iter().enumerate() {
                d_aux[index * time + t] += v;
            }
            for (distance, &v) in partial.dage.iter().enumerate() {
                d_aux[null_len + head * time + distance] += v;
            }
            if lorentz {
                d_aux[null_len + age_len + head] += partial.dbeta;
                d_aux[null_len + age_len + self.heads + head] += partial.doffset;
            }
        }
        let device = query.device();
        let d_aux: Vec<f32> = d_aux.into_iter().map(|v| v as f32).collect();
        Ok((
            Some(Tensor::from_vec(dq, query.shape(), device)?),
            Some(Tensor::from_vec(dkv, kv.shape(), device)?),
            Some(Tensor::from_vec(d_aux, aux.shape(), device)?),
        ))
    }
}

/// Multi-head causal read over [batch, heads, time, head] queries, keys and
/// values. Row `t` reads positions `0..=t`, plus a NoRead slot with a zero
/// value when `null` is set. `aux` is packed as [`fused_aux_len`] describes;
/// with neither NoRead nor age and the Dot score it is ignored (pass one zero).
/// `rope` rotates queries and keys by position first (the control's attention).
#[allow(clippy::too_many_arguments)]
pub fn fused_read(
    query: &Tensor,
    key: &Tensor,
    value: &Tensor,
    aux: &Tensor,
    score: ReadScore,
    null: bool,
    age: bool,
    rope: bool,
) -> Result<Tensor> {
    let (batch, heads, time, key_width) = query.dims4()?;
    let (b2, h2, t2, k2) = key.dims4()?;
    let (b3, h3, t3, value_width) = value.dims4()?;
    if (b2, h2, t2, k2) != (batch, heads, time, key_width) || (b3, h3, t3) != (batch, heads, time) {
        return Err(invalid(
            "fused read needs matching query, key and value shapes",
        ));
    }
    let expected = fused_aux_len(batch, heads, time, score, null, age);
    if aux.rank() != 1 || aux.dim(0)? != expected.max(1) {
        return Err(invalid("fused read auxiliary input has the wrong length"));
    }
    let op = FusedRead {
        batch,
        heads,
        time,
        key: key_width,
        value: value_width,
        score,
        null,
        age,
        rope,
    };
    if rope && key_width % 2 != 0 {
        return Err(invalid("RoPE needs an even head width"));
    }
    let kv = Tensor::cat(&[key, value], 3)?.contiguous()?;
    Ok(query.contiguous()?.apply_op3(&kv, &aux.contiguous()?, op)?)
}

// ---------------------------------------------------------------------------
// Fused RMSNorm and SwiGLU, parallel over rows.

/// `x w / sqrt(mean(x^2) + eps)` over the last dimension.
struct RmsNorm;

impl CustomOp2 for RmsNorm {
    fn name(&self) -> &'static str {
        "geometric-stack-rms-norm"
    }

    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let (x, w) = (contiguous(s1, l1)?, contiguous(s2, l2)?);
        let width = w.len();
        if l1.shape().dims().last() != Some(&width) {
            candle_core::bail!("RMSNorm weight must match the last dimension");
        }
        let mut out = vec![0f32; x.len()];
        out.par_chunks_mut(width)
            .zip(x.par_chunks(width))
            .for_each(|(out, row)| {
                let mean = row
                    .iter()
                    .map(|&v| f64::from(v) * f64::from(v))
                    .sum::<f64>()
                    / width as f64;
                let r = (1.0 / (mean + RMS_EPSILON).sqrt()) as f32;
                for ((o, &v), &g) in out.iter_mut().zip(row).zip(w) {
                    *o = v * r * g;
                }
            });
        Ok((CpuStorage::F32(out), l1.shape().clone()))
    }

    fn bwd(
        &self,
        x: &Tensor,
        w: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let xs = x.flatten_all()?.to_vec1::<f32>()?;
        let ws = w.to_vec1::<f32>()?;
        let gs = grad.flatten_all()?.to_vec1::<f32>()?;
        let width = ws.len();
        let mut dx = vec![0f32; xs.len()];
        let partials: Vec<Vec<f64>> = dx
            .par_chunks_mut(width * 64)
            .zip(xs.par_chunks(width * 64))
            .zip(gs.par_chunks(width * 64))
            .map(|((dx, x), g)| {
                let mut dw = vec![0f64; width];
                for ((dx, x), g) in dx
                    .chunks_mut(width)
                    .zip(x.chunks(width))
                    .zip(g.chunks(width))
                {
                    let mean =
                        x.iter().map(|&v| f64::from(v) * f64::from(v)).sum::<f64>() / width as f64;
                    let r = 1.0 / (mean + RMS_EPSILON).sqrt();
                    // dx = r (g w - xhat mean(g w xhat)), xhat = x r.
                    let mut projection = 0.0;
                    for i in 0..width {
                        let xhat = f64::from(x[i]) * r;
                        dw[i] += f64::from(g[i]) * xhat;
                        projection += f64::from(g[i]) * f64::from(ws[i]) * xhat;
                    }
                    projection /= width as f64;
                    for i in 0..width {
                        let xhat = f64::from(x[i]) * r;
                        dx[i] =
                            (r * (f64::from(g[i]) * f64::from(ws[i]) - xhat * projection)) as f32;
                    }
                }
                dw
            })
            .collect();
        let mut dw = vec![0f64; width];
        for partial in &partials {
            for (a, b) in dw.iter_mut().zip(partial) {
                *a += b;
            }
        }
        let dw: Vec<f32> = dw.into_iter().map(|v| v as f32).collect();
        Ok((
            Some(Tensor::from_vec(dx, x.shape(), x.device())?),
            Some(Tensor::from_vec(dw, w.shape(), w.device())?),
        ))
    }
}

/// `silu(gate) * up`.
struct SwiGlu;

#[inline]
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

impl CustomOp2 for SwiGlu {
    fn name(&self) -> &'static str {
        "geometric-stack-swiglu"
    }

    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let (gate, up) = (contiguous(s1, l1)?, contiguous(s2, l2)?);
        if l1.shape() != l2.shape() {
            candle_core::bail!("SwiGLU inputs must match");
        }
        let mut out = vec![0f32; gate.len()];
        out.par_chunks_mut(4096)
            .zip(gate.par_chunks(4096).zip(up.par_chunks(4096)))
            .for_each(|(out, (gate, up))| {
                for ((o, &g), &u) in out.iter_mut().zip(gate).zip(up) {
                    *o = g * sigmoid(g) * u;
                }
            });
        Ok((CpuStorage::F32(out), l1.shape().clone()))
    }

    fn bwd(
        &self,
        gate: &Tensor,
        up: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let gs = gate.flatten_all()?.to_vec1::<f32>()?;
        let us = up.flatten_all()?.to_vec1::<f32>()?;
        let ds = grad.flatten_all()?.to_vec1::<f32>()?;
        let mut d_gate = vec![0f32; gs.len()];
        let mut d_up = vec![0f32; gs.len()];
        d_gate
            .par_chunks_mut(4096)
            .zip(d_up.par_chunks_mut(4096))
            .zip(
                gs.par_chunks(4096)
                    .zip(us.par_chunks(4096))
                    .zip(ds.par_chunks(4096)),
            )
            .for_each(|((d_gate, d_up), ((g, u), d))| {
                for i in 0..g.len() {
                    let s = sigmoid(g[i]);
                    d_up[i] = d[i] * g[i] * s;
                    d_gate[i] = d[i] * u[i] * s * (1.0 + g[i] * (1.0 - s));
                }
            });
        Ok((
            Some(Tensor::from_vec(d_gate, gate.shape(), gate.device())?),
            Some(Tensor::from_vec(d_up, up.shape(), up.device())?),
        ))
    }
}

// ---------------------------------------------------------------------------
// Fused cross-entropy.

/// Mean next-token cross-entropy of [rows, vocabulary] logits, parallel over rows.
struct CrossEntropy {
    targets: Vec<u32>,
    /// Per-row weights of a weighted mean; `None` weighs every row equally.
    weights: Option<Vec<f32>>,
}

fn row_log_sum_exp(row: &[f32]) -> f64 {
    let maximum = row.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let total: f64 = row.iter().map(|&v| f64::from(v - maximum).exp()).sum();
    f64::from(maximum) + total.ln()
}

impl candle_core::CustomOp1 for CrossEntropy {
    fn name(&self) -> &'static str {
        "geometric-stack-cross-entropy"
    }

    fn cpu_fwd(
        &self,
        storage: &CpuStorage,
        layout: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let (rows, vocabulary) = layout.shape().dims2()?;
        let logits = contiguous(storage, layout)?;
        let mean = match &self.weights {
            None => {
                let total: f64 = logits
                    .par_chunks(vocabulary)
                    .zip(self.targets.par_iter())
                    .map(|(row, &target)| row_log_sum_exp(row) - f64::from(row[target as usize]))
                    .sum();
                total / rows as f64
            }
            Some(weights) => {
                let total: f64 = logits
                    .par_chunks(vocabulary)
                    .zip(self.targets.par_iter())
                    .zip(weights.par_iter())
                    .filter(|item| *item.1 != 0.0)
                    .map(|((row, &target), &weight)| {
                        f64::from(weight) * (row_log_sum_exp(row) - f64::from(row[target as usize]))
                    })
                    .sum();
                total / weights.iter().map(|&w| f64::from(w)).sum::<f64>()
            }
        };
        Ok((CpuStorage::F32(vec![mean as f32]), Shape::from(())))
    }

    fn bwd(
        &self,
        logits: &Tensor,
        _loss: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<Option<Tensor>> {
        let (rows, vocabulary) = logits.dims2()?;
        let values = logits.flatten_all()?.to_vec1::<f32>()?;
        let mut out = vec![0f32; values.len()];
        let grad = f64::from(grad.to_scalar::<f32>()?);
        let fill = |out: &mut [f32], row: &[f32], target: u32, scale: f64| {
            let lse = row_log_sum_exp(row);
            for (slot, &v) in out.iter_mut().zip(row) {
                *slot = ((f64::from(v) - lse).exp() * scale) as f32;
            }
            out[target as usize] -= scale as f32;
        };
        match &self.weights {
            None => {
                let scale = grad / rows as f64;
                out.par_chunks_mut(vocabulary)
                    .zip(values.par_chunks(vocabulary))
                    .zip(self.targets.par_iter())
                    .for_each(|((out, row), &target)| fill(out, row, target, scale));
            }
            Some(weights) => {
                let total: f64 = weights.iter().map(|&w| f64::from(w)).sum();
                out.par_chunks_mut(vocabulary)
                    .zip(values.par_chunks(vocabulary))
                    .zip(self.targets.par_iter())
                    .zip(weights.par_iter())
                    .filter(|item| *item.1 != 0.0)
                    .for_each(|(((out, row), &target), &weight)| {
                        fill(out, row, target, grad * f64::from(weight) / total)
                    });
            }
        }
        Ok(Some(Tensor::from_vec(
            out,
            logits.shape(),
            logits.device(),
        )?))
    }
}

/// Mean (or weighted mean) cross-entropy of `[rows, classes]` logits against
/// `targets`, with the fused forward and backward of
/// [`StackModel::weighted_loss`]: for callers that adjust the logits first, or
/// score a small auxiliary head. Weights, when given, are finite and
/// nonnegative with a positive sum.
pub fn logits_cross_entropy(
    logits: &Tensor,
    targets: &[u32],
    weights: Option<&[f32]>,
) -> Result<Tensor> {
    let (rows, classes) = logits.dims2()?;
    if targets.len() != rows || weights.is_some_and(|w| w.len() != rows) {
        return Err(invalid("one target and one weight per logit row"));
    }
    if targets.iter().any(|&t| t as usize >= classes) {
        return Err(invalid("target outside the logit classes"));
    }
    if let Some(weights) = weights {
        if weights.iter().any(|w| !w.is_finite() || *w < 0.0)
            || weights.iter().map(|&w| f64::from(w)).sum::<f64>() <= 0.0
        {
            return Err(invalid(
                "loss weights must be finite, nonnegative and not all zero",
            ));
        }
    }
    Ok(logits.contiguous()?.apply_op1(CrossEntropy {
        targets: targets.to_vec(),
        weights: weights.map(<[f32]>::to_vec),
    })?)
}

/// Per-row negative log-likelihood of `targets`, without a backward graph.
fn row_nll(logits: &Tensor, targets: &[u32]) -> Result<Vec<f64>> {
    let (rows, vocabulary) = logits.dims2()?;
    if rows != targets.len() {
        return Err(invalid("one target per logit row"));
    }
    let values = logits.flatten_all()?.to_vec1::<f32>()?;
    Ok(values
        .par_chunks(vocabulary)
        .zip(targets.par_iter())
        .map(|(row, &target)| row_log_sum_exp(row) - f64::from(row[target as usize]))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cpu() -> Device {
        Device::Cpu
    }

    fn random(rng: &mut Initializer, shape: &[usize], scale: f64) -> Tensor {
        let count: usize = shape.iter().product();
        let values: Vec<f32> = (0..count).map(|_| (rng.normal() * scale) as f32).collect();
        Tensor::from_vec(values, shape, &cpu()).expect("test tensor")
    }

    #[test]
    fn quaternion_scan_matches_the_sequential_recurrence() -> Result<()> {
        let mut rng = Initializer(7);
        let (batch, time, lanes) = (2, 9, 3);
        let q = random(&mut rng, &[batch, time, lanes, 4], 0.5);
        let b = random(&mut rng, &[batch, time, lanes, 4], 1.0);
        let h = quaternion_scan(&q, &b)?.flatten_all()?.to_vec1::<f32>()?;
        let (qv, bv) = (
            q.flatten_all()?.to_vec1::<f32>()?,
            b.flatten_all()?.to_vec1::<f32>()?,
        );
        for window in 0..batch {
            for lane in 0..lanes {
                let mut state = [0f64; 4];
                for t in 0..time {
                    let o = ((window * time + t) * lanes + lane) * 4;
                    let (a0, a1, a2, a3) = (
                        qv[o] as f64,
                        qv[o + 1] as f64,
                        qv[o + 2] as f64,
                        qv[o + 3] as f64,
                    );
                    let [b0, b1, b2, b3] = state;
                    state = [
                        a0 * b0 - a1 * b1 - a2 * b2 - a3 * b3 + bv[o] as f64,
                        a0 * b1 + a1 * b0 + a2 * b3 - a3 * b2 + bv[o + 1] as f64,
                        a0 * b2 - a1 * b3 + a2 * b0 + a3 * b1 + bv[o + 2] as f64,
                        a0 * b3 + a1 * b2 - a2 * b1 + a3 * b0 + bv[o + 3] as f64,
                    ];
                    for c in 0..4 {
                        assert!(
                            (h[o + c] as f64 - state[c]).abs() < 1e-5,
                            "scan differs at {o}+{c}"
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Directional finite difference of a scalar function of `vars` against the
    /// analytic gradient, in f64 on the perturbed direction.
    fn check_gradient(
        vars: &[Var],
        loss: impl Fn() -> Result<Tensor>,
        tolerance: f64,
    ) -> Result<()> {
        let grads = loss()?.backward()?;
        let mut rng = Initializer(99);
        let directions: Vec<Tensor> = vars
            .iter()
            .map(|var| random(&mut rng, var.dims(), 1.0))
            .collect();
        let mut analytic = 0.0;
        for (var, direction) in vars.iter().zip(&directions) {
            let grad = grads
                .get(var.as_tensor())
                .ok_or_else(|| invalid("missing gradient"))?;
            analytic += f64::from(grad.mul(direction)?.sum_all()?.to_scalar::<f32>()?);
        }
        let epsilon = 1e-3;
        let originals: Vec<Tensor> = vars
            .iter()
            .map(|var| var.as_tensor().copy())
            .collect::<std::result::Result<_, _>>()?;
        let evaluate = |sign: f64| -> Result<f64> {
            for ((var, original), direction) in vars.iter().zip(&originals).zip(&directions) {
                var.set(&original.add(&direction.affine(sign * epsilon, 0.0)?)?)?;
            }
            Ok(f64::from(loss()?.to_scalar::<f32>()?))
        };
        let numeric = (evaluate(1.0)? - evaluate(-1.0)?) / (2.0 * epsilon);
        for (var, original) in vars.iter().zip(&originals) {
            var.set(original)?;
        }
        let scale = analytic.abs().max(numeric.abs()).max(1e-3);
        assert!(
            (analytic - numeric).abs() / scale < tolerance,
            "analytic {analytic} numeric {numeric}"
        );
        Ok(())
    }

    #[test]
    fn quaternion_scan_backward_matches_finite_differences() -> Result<()> {
        let mut rng = Initializer(11);
        let (batch, time, lanes) = (2, 6, 2);
        let q = Var::from_tensor(&random(&mut rng, &[batch, time, lanes, 4], 0.4))?;
        let b = Var::from_tensor(&random(&mut rng, &[batch, time, lanes, 4], 1.0))?;
        let weights = random(&mut rng, &[batch, time, lanes, 4], 1.0);
        check_gradient(
            &[q.clone(), b.clone()],
            || {
                Ok(quaternion_scan(q.as_tensor(), b.as_tensor())?
                    .mul(&weights)?
                    .sum_all()?)
            },
            2e-3,
        )
    }

    /// The fused read computed with ordinary Candle operations.
    fn reference_read(
        query: &Tensor,
        key: &Tensor,
        value: &Tensor,
        null: Option<&Tensor>,
        age: Option<&Tensor>,
        lorentz: Option<(&Tensor, &Tensor)>,
    ) -> Result<Tensor> {
        let (batch, heads, time, width) = query.dims4()?;
        let inner = query.matmul(&key.transpose(2, 3)?)?;
        let mut scores = match lorentz {
            None => inner.affine(1.0 / (width as f64).sqrt(), 0.0)?,
            Some((beta, offset)) => {
                let lift = |x: &Tensor| -> Result<Tensor> {
                    Ok(x.sqr()?.sum_keepdim(3)?.affine(1.0, 1.0)?.sqrt()?)
                };
                let z = lift(query)?
                    .broadcast_mul(&lift(key)?.transpose(2, 3)?)?
                    .sub(&inner)?;
                let e = z
                    .affine(1.0, -1.0)?
                    .clamp(LORENTZ_MIN_EXCESS as f32, f32::MAX)?;
                let distance = e
                    .add(&e.mul(&e.affine(1.0, 2.0)?)?.sqrt()?)?
                    .affine(1.0, 1.0)?
                    .log()?;
                distance
                    .broadcast_sub(&offset.reshape((1, heads, 1, 1))?)?
                    .broadcast_mul(&beta.reshape((1, heads, 1, 1))?)?
                    .neg()?
            }
        };
        let mut mask = vec![0f32; time * time];
        let mut ages = vec![0u32; time * time];
        for t in 0..time {
            for j in 0..time {
                if j > t {
                    mask[t * time + j] = f32::NEG_INFINITY;
                } else {
                    ages[t * time + j] = (t - j) as u32;
                }
            }
        }
        if let Some(age) = age {
            let index = Tensor::from_vec(ages, time * time, &cpu())?;
            let table = age
                .index_select(&index, 1)?
                .reshape((1, heads, time, time))?;
            scores = scores.broadcast_add(&table)?;
        }
        scores = scores.broadcast_add(&Tensor::from_vec(mask, (1, 1, time, time), &cpu())?)?;
        let (scores, values) = match null {
            Some(null) => (
                Tensor::cat(&[&null.reshape((batch, heads, time, 1))?, &scores], 3)?,
                Tensor::cat(
                    &[
                        &Tensor::zeros((batch, heads, 1, value.dim(3)?), DType::F32, &cpu())?,
                        value,
                    ],
                    2,
                )?,
            ),
            None => (scores, value.clone()),
        };
        let probabilities = candle_nn::ops::softmax(&scores, 3)?;
        Ok(probabilities.matmul(&values)?)
    }

    #[test]
    fn tiled_read_products_equal_ascending_sums_bitwise() {
        let mut rng = Initializer(41);
        let mut values =
            |n: usize| -> Vec<f32> { (0..n).map(|_| (rng.normal() * 0.7) as f32).collect() };
        // Inner products: 4 rows (a full tile) and 3 (a short one), with
        // 37 columns (two full chunks and a remainder) and 23 terms.
        let (count, stride, columns) = (23, 40, 37);
        let (a, b) = (values(4 * count), values(count * stride));
        for rows in [4, 3] {
            let mut out = vec![0f32; rows * stride];
            tile_product(
                &a, count, rows, &b, stride, columns, count, &mut out, stride,
            );
            for r in 0..rows {
                for j in 0..columns {
                    let mut sum = 0f32;
                    for i in 0..count {
                        sum += a[r * count + i] * b[i * stride + j];
                    }
                    assert_eq!(
                        out[r * stride + j].to_bits(),
                        sum.to_bits(),
                        "product {r} {j}"
                    );
                }
            }
        }
        // Causal products of the tile at t0 = 9 for 33-wide rows.
        let (time, width, t0) = (16, 33, 9);
        let (p, v) = (values(4 * time), values(time * width));
        for rows in [4, 3] {
            let mut out = vec![0f32; rows * width];
            causal_rows(&p, time, rows, t0, &v, width, width, &mut out);
            for r in 0..rows {
                for c in 0..width {
                    let mut sum = 0f32;
                    for j in 0..=t0 + r {
                        sum += p[r * time + j] * v[j * width + c];
                    }
                    assert_eq!(out[r * width + c].to_bits(), sum.to_bits(), "rows {r} {c}");
                }
            }
        }
        // Transposed causal products of whole blocks: 16 and 18 positions
        // (full tiles, then a short one) by 33 columns.
        for time in [16, 18] {
            let (a, x) = (values(time * time), values(time * width));
            let mut out = vec![0f32; time * width];
            causal_transpose(&a, &x, time, width, &mut out);
            for j in 0..time {
                for c in 0..width {
                    let mut sum = 0f32;
                    for t in j..time {
                        sum += a[t * time + j] * x[t * width + c];
                    }
                    assert_eq!(
                        out[j * width + c].to_bits(),
                        sum.to_bits(),
                        "transpose {j} {c}"
                    );
                }
            }
        }
    }

    /// Small heads run the kernels' partial tiles only; `time` 11 with key
    /// width 20 and value width 33 also runs full tiles, the causal triangle
    /// and a short last tile.
    fn read_case(score: ReadScore) -> Result<()> {
        for (time, width, value_width) in [(7, 4, 5), (11, 20, 33)] {
            read_shape(score, time, width, value_width)?;
        }
        Ok(())
    }

    fn read_shape(score: ReadScore, time: usize, width: usize, value_width: usize) -> Result<()> {
        let mut rng = Initializer(match score {
            ReadScore::Dot => 3,
            ReadScore::Lorentz => 5,
        });
        let (batch, heads) = (2, 3);
        let q = Var::from_tensor(&random(&mut rng, &[batch, heads, time, width], 0.8))?;
        let k = Var::from_tensor(&random(&mut rng, &[batch, heads, time, width], 0.8))?;
        let v = Var::from_tensor(&random(&mut rng, &[batch, heads, time, value_width], 1.0))?;
        let null = Var::from_tensor(&random(&mut rng, &[batch, heads, time], 1.0))?;
        let age = Var::from_tensor(&random(&mut rng, &[heads, time], 0.5))?;
        let beta = Var::from_tensor(&random(&mut rng, &[heads], 0.3).affine(1.0, 1.0)?)?;
        let offset = Var::from_tensor(&random(&mut rng, &[heads], 0.5).affine(1.0, 2.0)?)?;
        let aux = |lorentz: bool| -> Result<Tensor> {
            let mut parts = vec![
                null.as_tensor().flatten_all()?,
                age.as_tensor().flatten_all()?,
            ];
            if lorentz {
                parts.push(beta.as_tensor().clone());
                parts.push(offset.as_tensor().clone());
            }
            Ok(Tensor::cat(&parts, 0)?)
        };
        let lorentz = score == ReadScore::Lorentz;
        let fused = fused_read(
            q.as_tensor(),
            k.as_tensor(),
            v.as_tensor(),
            &aux(lorentz)?,
            score,
            true,
            true,
            false,
        )?;
        let reference = reference_read(
            q.as_tensor(),
            k.as_tensor(),
            v.as_tensor(),
            Some(null.as_tensor()),
            Some(age.as_tensor()),
            lorentz.then_some((beta.as_tensor(), offset.as_tensor())),
        )?;
        let gap = fused
            .sub(&reference)?
            .abs()?
            .max_all()?
            .to_scalar::<f32>()?;
        assert!(gap < 1e-5, "fused read differs from the reference by {gap}");
        let weights = random(&mut rng, &[batch, heads, time, value_width], 1.0);
        let mut vars = vec![q.clone(), k.clone(), v.clone(), null.clone(), age.clone()];
        if lorentz {
            vars.push(beta.clone());
            vars.push(offset.clone());
        }
        // Every coordinate of the gradient against autograd through the
        // reference composition.
        let fused_grads = fused.mul(&weights)?.sum_all()?.backward()?;
        let reference_grads = reference.mul(&weights)?.sum_all()?.backward()?;
        for var in &vars {
            let (a, b) = (
                fused_grads
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("fused gradient"))?,
                reference_grads
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("reference gradient"))?,
            );
            let scale = b.abs()?.max_all()?.to_scalar::<f32>()?.max(1.0);
            let gap = a.sub(b)?.abs()?.max_all()?.to_scalar::<f32>()?;
            assert!(
                gap < 1e-4 * scale,
                "time {time}: gradient of {:?} differs by {gap} of {scale}",
                var.dims()
            );
        }
        if time > 7 {
            return Ok(());
        }
        check_gradient(
            &vars,
            || {
                Ok(fused_read(
                    q.as_tensor(),
                    k.as_tensor(),
                    v.as_tensor(),
                    &aux(lorentz)?,
                    score,
                    true,
                    true,
                    false,
                )?
                .mul(&weights)?
                .sum_all()?)
            },
            2e-3,
        )?;
        // Without NoRead or age: plain causal attention.
        let plain = fused_read(
            q.as_tensor(),
            k.as_tensor(),
            v.as_tensor(),
            &Tensor::zeros(1, DType::F32, &cpu())?,
            ReadScore::Dot,
            false,
            false,
            false,
        )?;
        let plain_reference = reference_read(
            q.as_tensor(),
            k.as_tensor(),
            v.as_tensor(),
            None,
            None,
            None,
        )?;
        let gap = plain
            .sub(&plain_reference)?
            .abs()?
            .max_all()?
            .to_scalar::<f32>()?;
        assert!(gap < 1e-5, "plain fused read differs by {gap}");
        // With RoPE: the reference rotates queries and keys with Candle ops.
        let rotate = |x: &Tensor| -> Result<Tensor> {
            let (cosine, sine) = rope_tables(time, width);
            let cosine = Tensor::from_vec(cosine, (time, width / 2), &cpu())?;
            let sine = Tensor::from_vec(sine, (time, width / 2), &cpu())?;
            let (a, b) = (
                x.narrow(3, 0, width / 2)?,
                x.narrow(3, width / 2, width / 2)?,
            );
            Ok(Tensor::cat(
                &[
                    &a.broadcast_mul(&cosine)?.sub(&b.broadcast_mul(&sine)?)?,
                    &b.broadcast_mul(&cosine)?.add(&a.broadcast_mul(&sine)?)?,
                ],
                3,
            )?)
        };
        let zero = Tensor::zeros(1, DType::F32, &cpu())?;
        let rotary = fused_read(
            q.as_tensor(),
            k.as_tensor(),
            v.as_tensor(),
            &zero,
            ReadScore::Dot,
            false,
            false,
            true,
        )?;
        let rotary_reference = reference_read(
            &rotate(q.as_tensor())?,
            &rotate(k.as_tensor())?,
            v.as_tensor(),
            None,
            None,
            None,
        )?;
        let gap = rotary
            .sub(&rotary_reference)?
            .abs()?
            .max_all()?
            .to_scalar::<f32>()?;
        assert!(gap < 1e-5, "rotary fused read differs by {gap}");
        check_gradient(
            &[q.clone(), k.clone(), v.clone()],
            || {
                Ok(fused_read(
                    q.as_tensor(),
                    k.as_tensor(),
                    v.as_tensor(),
                    &zero,
                    ReadScore::Dot,
                    false,
                    false,
                    true,
                )?
                .mul(&weights)?
                .sum_all()?)
            },
            2e-3,
        )?;
        Ok(())
    }

    #[test]
    fn fused_dot_read_matches_reference_and_finite_differences() -> Result<()> {
        read_case(ReadScore::Dot)
    }

    #[test]
    fn fused_lorentz_read_matches_reference_and_finite_differences() -> Result<()> {
        read_case(ReadScore::Lorentz)
    }

    #[test]
    fn transport_logits_equal_the_forward_until_snapped() -> Result<()> {
        let mut config = tiny(StackArch::Geometric, "rar", ReadScore::Lorentz, true);
        config.seed = 59;
        let model = StackModel::new(config, &cpu())?;
        for name in ["layers.00.rec.gate.weight", "layers.02.rec.gate.weight"] {
            let var = &model.variables()[name];
            var.set(&random(&mut Initializer(61), var.dims(), 0.5))?;
        }
        let vocab = model.config.vocab_size as u32;
        let ids: Vec<u32> = (0..18).map(|i| (i * 7 + 3) % vocab).collect();
        let fused = model.forward(&ids, 2, 9)?;
        let composed = model.logits_with_transport(&ids, 2, 9, None)?;
        let gap = fused.sub(&composed)?.abs()?.max_all()?.to_scalar::<f32>()?;
        assert!(gap < 1e-4, "the composed transport differs by {gap}");
        // Snapping to the identity alone removes every rotation, and snapping
        // to the 2I roots changes the logits but keeps them finite.
        let identity = [[1f32, 0.0, 0.0, 0.0]];
        let still = model.logits_with_transport(&ids, 2, 9, Some(&identity))?;
        assert!(still.sub(&fused)?.abs()?.max_all()?.to_scalar::<f32>()? > 1e-6);
        let roots: Vec<[f32; 4]> =
            uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots()
                .iter()
                .map(|r| {
                    let a = r.to_array();
                    [a[0] as f32, a[1] as f32, a[2] as f32, a[3] as f32]
                })
                .collect();
        let snapped = model
            .logits_with_transport(&ids, 2, 9, Some(&roots))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(snapped.iter().all(|v| v.is_finite()));
        Ok(())
    }

    #[test]
    fn fused_recurrence_matches_the_composed_mixer() -> Result<()> {
        for rotation in [true, false] {
            let mut config = tiny(StackArch::Geometric, "r", ReadScore::Lorentz, rotation);
            config.seed = 41;
            let model = StackModel::new(config, &cpu())?;
            // Larger gate and rotation weights than initialization, so every
            // path carries a visible signal.
            for name in ["layers.00.rec.gate.weight", "layers.00.rec.conv.weight"] {
                let var = &model.variables()[name];
                var.set(&random(&mut Initializer(43), var.dims(), 0.5))?;
            }
            let x = random(&mut Initializer(47), &[2, 9, 16], 1.0);
            let p = model.params()?;
            let fused = model.recurrence(&p, 0, &x, &mut None)?;
            let composed = model.composed_recurrence(&p, 0, &x, None)?;
            let gap = fused.sub(&composed)?.abs()?.max_all()?.to_scalar::<f32>()?;
            assert!(
                gap < 1e-5,
                "rotation {rotation}: fused recurrence differs by {gap}"
            );
            let weights = random(&mut Initializer(53), &[2, 9, 16], 1.0);
            let names = [
                "layers.00.rec.in.weight",
                "layers.00.rec.gate.weight",
                "layers.00.rec.gate.bias",
                "layers.00.rec.conv.weight",
                "layers.00.rec.conv.bias",
                "layers.00.rec.decay",
            ];
            let vars: Vec<Var> = names
                .iter()
                .map(|name| model.variables()[*name].clone())
                .collect();
            let fused_loss = model
                .recurrence(&p, 0, &x, &mut None)?
                .mul(&weights)?
                .sum_all()?;
            let composed_loss = model
                .composed_recurrence(&p, 0, &x, None)?
                .mul(&weights)?
                .sum_all()?;
            let (a, b) = (fused_loss.backward()?, composed_loss.backward()?);
            for (name, var) in names.iter().zip(&vars) {
                let ga = a
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("fused gradient"))?;
                let gb = b
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("composed gradient"))?;
                let scale = gb.abs()?.max_all()?.to_scalar::<f32>()?.max(1e-3);
                let gap = ga.sub(gb)?.abs()?.max_all()?.to_scalar::<f32>()? / scale;
                assert!(
                    gap < 1e-3,
                    "rotation {rotation}: {name} gradient differs by {gap} (relative)"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn fused_norm_and_swiglu_match_candle_compositions() -> Result<()> {
        let mut rng = Initializer(31);
        let x = Var::from_tensor(&random(&mut rng, &[5, 3, 8], 1.5))?;
        let w = Var::from_tensor(&random(&mut rng, &[8], 1.0))?;
        let fused = x.as_tensor().apply_op2(w.as_tensor(), RmsNorm)?;
        let denominator = x
            .as_tensor()
            .sqr()?
            .mean_keepdim(2)?
            .affine(1.0, RMS_EPSILON)?
            .sqrt()?;
        let reference = x
            .as_tensor()
            .broadcast_div(&denominator)?
            .broadcast_mul(w.as_tensor())?;
        assert!(
            fused
                .sub(&reference)?
                .abs()?
                .max_all()?
                .to_scalar::<f32>()?
                < 1e-5
        );
        let weights = random(&mut rng, &[5, 3, 8], 1.0);
        check_gradient(
            &[x.clone(), w.clone()],
            || {
                Ok(x.as_tensor()
                    .apply_op2(w.as_tensor(), RmsNorm)?
                    .mul(&weights)?
                    .sum_all()?)
            },
            2e-3,
        )?;
        let gate = Var::from_tensor(&random(&mut rng, &[4, 9], 2.0))?;
        let up = Var::from_tensor(&random(&mut rng, &[4, 9], 1.0))?;
        let fused = gate.as_tensor().apply_op2(up.as_tensor(), SwiGlu)?;
        let reference = gate.as_tensor().silu()?.mul(up.as_tensor())?;
        assert!(
            fused
                .sub(&reference)?
                .abs()?
                .max_all()?
                .to_scalar::<f32>()?
                < 1e-5
        );
        let weights = random(&mut rng, &[4, 9], 1.0);
        check_gradient(
            &[gate.clone(), up.clone()],
            || {
                Ok(gate
                    .as_tensor()
                    .apply_op2(up.as_tensor(), SwiGlu)?
                    .mul(&weights)?
                    .sum_all()?)
            },
            2e-3,
        )
    }

    #[test]
    fn fused_cross_entropy_matches_candle() -> Result<()> {
        let mut rng = Initializer(23);
        let logits = Var::from_tensor(&random(&mut rng, &[6, 11], 2.0))?;
        let targets: Vec<u32> = vec![0, 3, 10, 5, 5, 7];
        let fused = logits.as_tensor().apply_op1(CrossEntropy {
            targets: targets.clone(),
            weights: None,
        })?;
        let index = Tensor::from_vec(targets.clone(), 6, &cpu())?;
        let reference = candle_nn::loss::cross_entropy(logits.as_tensor(), &index)?;
        let gap = (fused.to_scalar::<f32>()? - reference.to_scalar::<f32>()?).abs();
        assert!(gap < 1e-6, "loss differs by {gap}");
        let (a, b) = (fused.backward()?, reference.backward()?);
        let ga = a
            .get(logits.as_tensor())
            .ok_or_else(|| invalid("fused gradient"))?;
        let gb = b
            .get(logits.as_tensor())
            .ok_or_else(|| invalid("reference gradient"))?;
        let gap = ga.sub(gb)?.abs()?.max_all()?.to_scalar::<f32>()?;
        assert!(gap < 1e-6, "gradient differs by {gap}");
        let rows = row_nll(logits.as_tensor(), &targets)?;
        let mean = rows.iter().sum::<f64>() / rows.len() as f64;
        assert!((mean - f64::from(reference.to_scalar::<f32>()?)).abs() < 1e-6);

        // Weighted: the weighted mean of the rows, with the Candle
        // composition's gradient; zero-weight rows get none.
        let weights = vec![0.0f32, 1.0, 2.5, 0.0, 1.0, 0.5];
        let fused = logits.as_tensor().apply_op1(CrossEntropy {
            targets: targets.clone(),
            weights: Some(weights.clone()),
        })?;
        let total: f64 = weights.iter().map(|&w| f64::from(w)).sum();
        let want: f64 = rows
            .iter()
            .zip(&weights)
            .map(|(nll, &w)| nll * f64::from(w))
            .sum::<f64>()
            / total;
        let gap = (f64::from(fused.to_scalar::<f32>()?) - want).abs();
        assert!(gap < 1e-6, "weighted loss differs by {gap}");
        let picked = candle_nn::ops::log_softmax(logits.as_tensor(), 1)?
            .gather(&index.unsqueeze(1)?, 1)?
            .squeeze(1)?;
        let reference = picked
            .mul(&Tensor::from_vec(weights.clone(), 6, &cpu())?)?
            .sum_all()?
            .affine(-1.0 / total, 0.0)?;
        let (a, b) = (fused.backward()?, reference.backward()?);
        let ga = a
            .get(logits.as_tensor())
            .ok_or_else(|| invalid("fused gradient"))?;
        let gb = b
            .get(logits.as_tensor())
            .ok_or_else(|| invalid("reference gradient"))?;
        let gap = ga.sub(gb)?.abs()?.max_all()?.to_scalar::<f32>()?;
        assert!(gap < 1e-6, "weighted gradient differs by {gap}");
        let zero_rows = ga.get(0)?.abs()?.max_all()?.to_scalar::<f32>()?
            + ga.get(3)?.abs()?.max_all()?.to_scalar::<f32>()?;
        assert_eq!(zero_rows, 0.0);
        Ok(())
    }

    fn tiny(arch: StackArch, pattern: &str, read: ReadScore, rotation: bool) -> StackConfig {
        StackConfig {
            arch,
            vocab_size: 37,
            width: 16,
            heads: 2,
            mlp_hidden: 24,
            context: 12,
            pattern: pattern.into(),
            read,
            rotation,
            seed: 5,
            memory: None,
        }
    }

    /// `config` with a small product-key memory in place of layer 1's MLP.
    fn with_memory(mut config: StackConfig, score: MemoryScore) -> StackConfig {
        config.memory = Some(MemoryConfig {
            layers: vec![1],
            sub_keys: 8,
            top_k: 3,
            heads: 2,
            key_dim: 8,
            score,
            codebook: None,
        });
        config
    }

    #[test]
    fn codebook_keys_are_fixed_and_the_rest_learns() -> Result<()> {
        use crate::stack_memory::Codebook;
        for codebook in [Codebook::H4, Codebook::E8] {
            let mut config = tiny(StackArch::Geometric, "ra", ReadScore::Dot, true);
            config.memory = Some(MemoryConfig {
                layers: vec![1],
                sub_keys: codebook.size(),
                top_k: 4,
                heads: 2,
                key_dim: 2 * codebook.dim(),
                score: MemoryScore::Dot,
                codebook: Some(codebook),
            });
            let model = StackModel::new(config, &cpu())?;
            let keys = model.variables()["layers.01.memory.keys"]
                .as_tensor()
                .to_vec2::<f32>()?;
            let vectors = codebook.vectors();
            for (row, key) in keys.iter().enumerate() {
                let want = &vectors[row % codebook.size()];
                assert!(key
                    .iter()
                    .zip(want)
                    .all(|(a, b)| (f64::from(*a) - b).abs() < 1e-6));
            }
            let ids: Vec<u32> = (0..10u32).map(|i| (i * 5 + 1) % 37).collect();
            let targets: Vec<u32> = (0..10u32).map(|i| (i * 3 + 2) % 37).collect();
            let grads = model.loss(&ids, &targets, 1, 10)?.backward()?;
            let names = model.variables();
            assert!(grads
                .get(names["layers.01.memory.keys"].as_tensor())
                .is_none());
            for name in ["layers.01.memory.query.weight", "layers.01.memory.values"] {
                assert!(grads.get(names[name].as_tensor()).is_some(), "{name}");
            }
        }
        Ok(())
    }

    #[test]
    fn stacks_are_causal() -> Result<()> {
        for config in [
            tiny(StackArch::Transformer, "aa", ReadScore::Dot, false),
            tiny(StackArch::Geometric, "ra", ReadScore::Lorentz, true),
            tiny(StackArch::Geometric, "ar", ReadScore::Dot, false),
            with_memory(
                tiny(StackArch::Geometric, "ra", ReadScore::Dot, true),
                MemoryScore::Dot,
            ),
            with_memory(
                tiny(StackArch::Geometric, "ar", ReadScore::Lorentz, false),
                MemoryScore::Lorentz,
            ),
        ] {
            let model = StackModel::new(config.clone(), &cpu())?;
            let time = 10;
            let ids: Vec<u32> = (0..time as u32).map(|i| (i * 7 + 3) % 37).collect();
            let mut changed = ids.clone();
            changed[6] = (changed[6] + 5) % 37;
            let a = model.forward(&ids, 1, time)?.to_vec2::<f32>()?;
            let b = model.forward(&changed, 1, time)?.to_vec2::<f32>()?;
            for t in 0..time {
                let gap = a[t]
                    .iter()
                    .zip(&b[t])
                    .map(|(x, y)| (x - y).abs())
                    .fold(0f32, f32::max);
                if t < 6 {
                    assert!(
                        gap < 1e-6,
                        "{:?}: position {t} sees a later token",
                        config.pattern
                    );
                } else if t == 6 {
                    assert!(
                        gap > 1e-4,
                        "{:?}: position {t} ignores its own token",
                        config.pattern
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn whole_model_gradient_matches_finite_differences() -> Result<()> {
        let model = StackModel::new(
            tiny(StackArch::Geometric, "ra", ReadScore::Lorentz, true),
            &cpu(),
        )?;
        let ids: Vec<u32> = (0..16u32).map(|i| (i * 5 + 1) % 37).collect();
        // Random logit weights give derivatives well above f32 rounding of the
        // function value; the cross-entropy's own backward is Candle's.
        let weights = random(&mut Initializer(17), &[16, 37], 1.0);
        let vars: Vec<Var> = [
            "layers.00.rec.gate.weight",
            "layers.00.rec.decay",
            "layers.00.rec.conv.weight",
            "layers.01.read.key.weight",
            "layers.01.read.offset",
            "layers.01.read.age",
        ]
        .iter()
        .map(|name| model.variables()[*name].clone())
        .collect();
        check_gradient(
            &vars,
            || Ok(model.forward(&ids, 2, 8)?.mul(&weights)?.sum_all()?),
            5e-3,
        )
    }

    #[test]
    fn matched_stack_has_the_control_parameter_count() -> Result<()> {
        let control = StackConfig::transformer_control(1).parameter_count()?;
        assert_eq!(
            control, 7_155_360,
            "the control has #1017's parameter count"
        );
        for (pattern, read, rotation) in [
            ("rrarra", ReadScore::Lorentz, true),
            ("rrarra", ReadScore::Dot, true),
            ("rrarra", ReadScore::Lorentz, false),
            ("aaaaaa", ReadScore::Lorentz, false),
        ] {
            let config = StackConfig::geometric_matched(pattern, read, rotation, 1)?;
            let count = config.parameter_count()?;
            let gap = (count as f64 - control as f64).abs() / control as f64;
            assert!(
                gap < 1e-3,
                "{pattern} {read:?} {rotation}: {count} against {control}"
            );
        }
        Ok(())
    }

    #[test]
    fn memory_layers_replace_the_mlp_train_and_round_trip() -> Result<()> {
        for score in [MemoryScore::Dot, MemoryScore::Lorentz] {
            let config = with_memory(
                tiny(StackArch::Geometric, "ra", ReadScore::Dot, true),
                score,
            );
            let model = StackModel::new(config.clone(), &cpu())?;
            let names = model.variables();
            assert!(names.contains_key("layers.01.memory.values"));
            assert!(!names.contains_key("layers.01.mlp.gate.weight"));
            assert!(names.contains_key("layers.00.mlp.gate.weight"));
            assert_eq!(
                names.contains_key("layers.01.memory.log_beta"),
                score == MemoryScore::Lorentz
            );
            // A token reads 2 heads x 3 of the 64 value rows of width 16.
            assert_eq!(
                config.parameter_count()? - config.active_parameter_count()?,
                (64 - 6) * 16
            );
            let ids: Vec<u32> = (0..10u32).map(|i| (i * 5 + 1) % 37).collect();
            let targets: Vec<u32> = (0..10u32).map(|i| (i * 3 + 2) % 37).collect();
            let grads = model.loss(&ids, &targets, 1, 10)?.backward()?;
            for name in [
                "layers.01.memory.query.weight",
                "layers.01.memory.keys",
                "layers.01.memory.values",
            ] {
                let grad = grads
                    .get(names[name].as_tensor())
                    .ok_or_else(|| invalid(format!("no gradient for {name}")))?;
                assert!(grad.abs()?.max_all()?.to_scalar::<f32>()? > 0.0, "{name}");
            }
            let directory = std::env::temp_dir().join(format!(
                "geometric-stack-memory-{score:?}-{}",
                std::process::id()
            ));
            model.save(&directory)?;
            let loaded = StackModel::load(&directory, &cpu())?;
            fs::remove_dir_all(&directory)?;
            assert_eq!(loaded.config, config);
            let a = model.forward(&ids, 1, 10)?;
            let b = loaded.forward(&ids, 1, 10)?;
            assert_eq!(a.sub(&b)?.abs()?.max_all()?.to_scalar::<f32>()?, 0.0);
        }
        Ok(())
    }

    #[test]
    fn fused_adam_step_is_bit_identical_to_the_candle_composition() -> Result<()> {
        let device = cpu();
        let n = 50_000;
        let mut seed = 3u64;
        let mut draw = |scale: f32| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((((seed >> 11) as f64) / ((1u64 << 53) as f64) - 0.5) as f32) * scale
        };
        let p: Vec<f32> = (0..n).map(|_| draw(0.2)).collect();
        let g: Vec<f32> = (0..n).map(|_| draw(0.01)).collect();
        let m: Vec<f32> = (0..n).map(|_| draw(0.001)).collect();
        let v: Vec<f32> = (0..n).map(|_| draw(1e-5).abs()).collect();
        let (beta1, beta2, epsilon, scale, lr, weight_decay) =
            (0.9f64, 0.95f64, 1e-8f64, 0.73f64, 0.004f64, 0.1f64);
        let (first, second) = (1.0 - beta1.powi(7), 1.0 - beta2.powi(7));
        let keep = 1.0 - lr * weight_decay;
        // The composition of Candle operations the fused step replaced.
        let t = |x: &Vec<f32>| Tensor::from_vec(x.clone(), n, &device);
        let grad = t(&g)?.affine(scale, 0.0)?;
        let m_next = t(&m)?
            .affine(beta1, 0.0)?
            .add(&grad.affine(1.0 - beta1, 0.0)?)?;
        let v_next = t(&v)?
            .affine(beta2, 0.0)?
            .add(&grad.sqr()?.affine(1.0 - beta2, 0.0)?)?;
        let step = m_next.affine(1.0 / first, 0.0)?.div(
            &v_next
                .affine(1.0 / second, 0.0)?
                .sqrt()?
                .affine(1.0, epsilon)?,
        )?;
        let p_next = t(&p)?.affine(keep, 0.0)?.sub(&step.affine(lr, 0.0)?)?;
        let (mut p2, mut m2, mut v2) = (p.clone(), m.clone(), v.clone());
        adam_step(
            &mut p2,
            &g,
            &mut m2,
            &mut v2,
            &AdamConstants {
                scale: scale as f32,
                beta1: beta1 as f32,
                rest1: (1.0 - beta1) as f32,
                beta2: beta2 as f32,
                rest2: (1.0 - beta2) as f32,
                correct1: (1.0 / first) as f32,
                correct2: (1.0 / second) as f32,
                epsilon: epsilon as f32,
                keep: keep as f32,
                lr: lr as f32,
            },
        );
        let bits = |x: &[f32]| x.iter().map(|v| v.to_bits()).collect::<Vec<u32>>();
        assert_eq!(bits(&p2), bits(&p_next.to_vec1::<f32>()?));
        assert_eq!(bits(&m2), bits(&m_next.to_vec1::<f32>()?));
        assert_eq!(bits(&v2), bits(&v_next.to_vec1::<f32>()?));
        Ok(())
    }

    #[test]
    fn save_and_load_reproduce_logits() -> Result<()> {
        let model = StackModel::new(
            tiny(StackArch::Geometric, "ar", ReadScore::Lorentz, true),
            &cpu(),
        )?;
        let directory =
            std::env::temp_dir().join(format!("geometric-stack-save-{}", std::process::id()));
        model.save(&directory)?;
        let loaded = StackModel::load(&directory, &cpu())?;
        fs::remove_dir_all(&directory)?;
        let ids: Vec<u32> = (0..8u32).collect();
        let a = model.forward(&ids, 1, 8)?;
        let b = loaded.forward(&ids, 1, 8)?;
        assert_eq!(a.sub(&b)?.abs()?.max_all()?.to_scalar::<f32>()?, 0.0);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // The served representation.

    /// A small geometric stack the export takes: its width a multiple of
    /// `GROUP` and an MLP the export pads (40 units to 64).
    fn exportable(pattern: &str, read: ReadScore, rotation: bool) -> StackConfig {
        StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 96,
            width: 64,
            heads: 2,
            mlp_hidden: 40,
            context: 16,
            pattern: pattern.into(),
            read,
            rotation,
            seed: 29,
            memory: None,
        }
    }

    /// Weights spread over many magnitudes, so the export uses many grid
    /// exponents: every row of a map has its own power-of-two scale (2^-4 to
    /// 2^3 of the base), some groups are zero, and some rows lie 2^-24 below
    /// the rest, under the export's exponent floor. Norm gains stay near one;
    /// decays, ages, gate biases and Lorentz offsets near their initial
    /// values; the convolution taps get scales of 2^-3 to 2^2.
    fn spread(model: &StackModel, seed: u64) -> Result<()> {
        let mut rng = Initializer(seed);
        for (name, var) in model.variables() {
            let cols = var.dims().last().copied().unwrap_or(1).max(1);
            let old = var.as_tensor().flatten_all()?.to_vec1::<f32>()?;
            let mut values = Vec::with_capacity(old.len());
            for (i, &v) in old.iter().enumerate() {
                let (row, col) = (i / cols, i % cols);
                let n = rng.normal();
                let near = |noise: f64| f64::from(v) + noise * n;
                let value = if name.ends_with("norm.weight") {
                    1.0 + 0.3 * n
                } else if ["rec.decay", "read.age", "rec.gate.bias", "read.offset"]
                    .iter()
                    .any(|suffix| name.ends_with(suffix))
                {
                    near(0.3)
                } else if name.ends_with("log_beta") || name.ends_with("bias") {
                    0.5 * n
                } else if name.ends_with("conv.weight") {
                    near(0.3 * 2f64.powi((i % 6) as i32 - 3))
                } else if var.rank() == 2 {
                    if row % 11 == 5 && col < GROUP {
                        0.0
                    } else if row % 13 == 7 {
                        0.05 * n * 2f64.powi(-24)
                    } else {
                        0.05 * n * 2f64.powi((row % 8) as i32 - 4)
                    }
                } else {
                    0.05 * n
                };
                values.push(value as f32);
            }
            var.set(&Tensor::from_vec(values, var.shape(), &cpu())?)?;
        }
        Ok(())
    }

    fn token_ids(count: usize, vocab: u32, seed: u64) -> Vec<u32> {
        let mut rng = Initializer(seed);
        (0..count)
            .map(|_| (rng.next() % u64::from(vocab)) as u32)
            .collect()
    }

    fn max_abs_gap(a: &Tensor, b: &Tensor) -> Result<f32> {
        Ok(a.sub(b)?.abs()?.max_all()?.to_scalar::<f32>()?)
    }

    fn bits(tensor: &Tensor) -> Result<Vec<u32>> {
        Ok(tensor
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .map(|v| v.to_bits())
            .collect())
    }

    /// The export of `model` (round to nearest, no calibration) and its grid
    /// reference.
    fn exported_reference(model: &StackModel) -> Result<crate::stack_export::GridReference> {
        let (bytes, _) =
            crate::stack_export::export_stack(model, serde_json::json!({"test": "qat"}), None)?;
        let artifact = uor_r4_lut::format::StackArtifact::parse(bytes)
            .map_err(|error| invalid(error.to_string()))?;
        crate::stack_export::stack_grid_reference(model, &artifact)
    }

    #[test]
    fn served_forward_equals_the_exported_reference() -> Result<()> {
        let mut worst = 0f32;
        for pattern in ["rar", "rrarra"] {
            for read in [ReadScore::Lorentz, ReadScore::Dot] {
                for rotation in [true, false] {
                    let mut model = StackModel::new(exportable(pattern, read, rotation), &cpu())?;
                    spread(&model, 71)?;
                    let reference = exported_reference(&model)?;
                    let time = model.config.context;
                    let ids = token_ids(2 * time, 96, 5);
                    let float = model.forward(&ids, 2, time)?;
                    model.set_served_representation(Some(Arc::new(D11Interim)))?;
                    let served = model.forward(&ids, 2, time)?;
                    let want = reference.logits(&ids, 2, time)?;
                    let gap = max_abs_gap(&served, &want)?;
                    let label = format!("{pattern} {read:?} rotation {rotation}");
                    assert!(
                        gap <= 1e-5,
                        "{label}: served logits differ from the exported reference by {gap}"
                    );
                    assert!(
                        max_abs_gap(&served, &float)? > 1e-3,
                        "{label}: the served representation left the logits unchanged"
                    );
                    worst = worst.max(gap);
                    // The piecewise entry points read the same served view.
                    let x =
                        model.run_layers(model.embed(&ids, 2, time)?, 0..model.config.layers())?;
                    let piecewise = model.head(&model.finish(x)?)?;
                    assert_eq!(bits(&piecewise)?, bits(&served)?, "{label}: piecewise");
                    let targets = token_ids(2 * time, 96, 6);
                    let loss = model.loss(&ids, &targets, 2, time)?.to_scalar::<f32>()?;
                    let reference_loss =
                        logits_cross_entropy(&want, &targets, None)?.to_scalar::<f32>()?;
                    assert!((loss - reference_loss).abs() <= 1e-6, "{label}: loss");
                }
            }
        }
        // The weights exercise many of the export's scale exponents.
        let model = StackModel::new(exportable("rar", ReadScore::Lorentz, true), &cpu())?;
        spread(&model, 71)?;
        let embedding = model.variables()["embedding.weight"]
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        let packed = quantize_matrix(&embedding, 96, 64)?;
        let exponents: BTreeSet<u8> = packed.scales.iter().map(|scale| scale >> 4).collect();
        assert!(
            exponents.len() >= 8,
            "only {} scale exponents",
            exponents.len()
        );
        eprintln!(
            "served forward against the exported reference: worst absolute logit gap {worst}"
        );
        Ok(())
    }

    /// The float forward composed directly from the variables with the
    /// stack's kernels, in the order the forward pass took before the served
    /// representation existed.
    fn float_reference_logits(
        model: &StackModel,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        let c = &model.config;
        let w = |name: &str| -> Result<Tensor> {
            Ok(model
                .variables()
                .get(name)
                .ok_or_else(|| invalid(name.to_owned()))?
                .as_tensor()
                .clone())
        };
        let l = |layer: usize, suffix: &str| w(&layer_name(layer, suffix));
        let norm = |x: &Tensor, gain: &Tensor| -> Result<Tensor> {
            Ok(x.contiguous()?.apply_op2(&gain.contiguous()?, RmsNorm)?)
        };
        let linear = StackModel::linear;
        let split = |x: &Tensor| -> Result<Tensor> {
            Ok(x.reshape((batch, time, c.heads, c.head_width()))?
                .transpose(1, 2)?
                .contiguous()?)
        };
        let index = Tensor::from_vec(ids.to_vec(), batch * time, model.device())?;
        let mut x = w("embedding.weight")?
            .index_select(&index, 0)?
            .reshape((batch, time, c.width))?;
        for layer in 0..c.layers() {
            let mixed = if c.layer_kind(layer) == 'r' {
                let u = norm(&x, &l(layer, "rec_norm.weight")?)?;
                let branches = linear(&u, &l(layer, "rec.in.weight")?)?;
                let gates = linear(&u, &l(layer, "rec.gate.weight")?)?
                    .broadcast_add(&l(layer, "rec.gate.bias")?)?;
                let parameters = Tensor::cat(
                    &[
                        &l(layer, "rec.conv.weight")?.flatten_all()?,
                        &l(layer, "rec.conv.bias")?,
                        &l(layer, "rec.decay")?,
                    ],
                    0,
                )?;
                let core = branches.contiguous()?.apply_op3(
                    &gates.contiguous()?,
                    &parameters,
                    RecurrenceCore {
                        batch,
                        time,
                        width: c.width,
                        rotation: c.rotation,
                    },
                )?;
                linear(&core, &l(layer, "rec.out.weight")?)?
            } else {
                let u = norm(&x, &l(layer, "read_norm.weight")?)?;
                let project = |part: &str| -> Result<Tensor> {
                    split(&linear(&u, &l(layer, &format!("read.{part}.weight"))?)?)
                };
                let null = linear(&u, &l(layer, "read.null.weight")?)?
                    .broadcast_add(&l(layer, "read.null.bias")?)?
                    .transpose(1, 2)?
                    .flatten_all()?;
                let age = l(layer, "read.age")?.narrow(1, 0, time)?.flatten_all()?;
                let mut aux = vec![null, age];
                if c.read == ReadScore::Lorentz {
                    aux.push(l(layer, "read.log_beta")?.exp()?);
                    aux.push(l(layer, "read.offset")?);
                }
                let read = fused_read(
                    &project("query")?,
                    &project("key")?,
                    &project("value")?,
                    &Tensor::cat(&aux, 0)?,
                    c.read,
                    true,
                    true,
                    false,
                )?;
                let merged = read.transpose(1, 2)?.reshape((batch, time, c.width))?;
                linear(&merged, &l(layer, "read.out.weight")?)?
            };
            x = x.add(&mixed)?;
            let u = norm(&x, &l(layer, "mlp_norm.weight")?)?;
            let gate = linear(&u, &l(layer, "mlp.gate.weight")?)?;
            let up = linear(&u, &l(layer, "mlp.up.weight")?)?;
            let mixed = gate.contiguous()?.apply_op2(&up.contiguous()?, SwiGlu)?;
            x = x.add(&linear(&mixed, &l(layer, "mlp.down.weight")?)?)?;
        }
        let hidden = norm(&x, &w("final_norm.weight")?)?.reshape((batch * time, c.width))?;
        Ok(hidden.matmul(&w("embedding.weight")?.t()?)?)
    }

    #[test]
    fn served_mode_off_is_bit_identical_to_the_float_forward() -> Result<()> {
        for (pattern, read, rotation) in [
            ("rar", ReadScore::Lorentz, true),
            ("rrarra", ReadScore::Dot, false),
            ("rrarra", ReadScore::Lorentz, true),
        ] {
            let mut model = StackModel::new(exportable(pattern, read, rotation), &cpu())?;
            spread(&model, 73)?;
            let time = model.config.context;
            let (ids, targets) = (token_ids(2 * time, 96, 9), token_ids(2 * time, 96, 10));
            let plain = bits(&model.forward(&ids, 2, time)?)?;
            let reference = float_reference_logits(&model, &ids, 2, time)?;
            assert_eq!(
                plain,
                bits(&reference)?,
                "{pattern}: the float forward changed"
            );
            // The backward too: every variable's gradient, bit for bit.
            let grads = model.loss(&ids, &targets, 2, time)?.backward()?;
            let reference_grads = logits_cross_entropy(&reference, &targets, None)?.backward()?;
            for (name, var) in model.variables() {
                let (a, b) = (
                    grads
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid(name.clone()))?,
                    reference_grads
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid(name.clone()))?,
                );
                assert_eq!(
                    bits(a)?,
                    bits(b)?,
                    "{pattern}: the gradient of {name} changed"
                );
            }
            // Served mode changes the logits; off again, or bypassed, it is gone.
            model.set_served_representation(Some(Arc::new(D11Interim)))?;
            assert_ne!(plain, bits(&model.forward(&ids, 2, time)?)?);
            let bypassed = model.with_float_forward(|m| m.forward(&ids, 2, time))?;
            assert_eq!(plain, bits(&bypassed)?, "{pattern}: with_float_forward");
            assert!(model.served_codec().is_some());
            model.set_served_representation(None)?;
            assert_eq!(
                plain,
                bits(&model.forward(&ids, 2, time)?)?,
                "{pattern}: off again"
            );
        }
        Ok(())
    }

    #[test]
    fn a_served_training_step_reaches_every_parameter() -> Result<()> {
        for (pattern, read, rotation) in [
            ("rar", ReadScore::Lorentz, true),
            ("rrarra", ReadScore::Dot, false),
        ] {
            let mut model = StackModel::new(exportable(pattern, read, rotation), &cpu())?;
            spread(&model, 79)?;
            // The whole context, so every age-table entry is read.
            let time = model.config.context;
            let (ids, targets) = (token_ids(2 * time, 96, 13), token_ids(2 * time, 96, 17));
            let reference = exported_reference(&model)?;
            model.set_served_representation(Some(Arc::new(D11Interim)))?;
            let loss = model.loss(&ids, &targets, 2, time)?;
            let grads = loss.backward()?;
            // The reference's gradients at the served values, its head a variable.
            let head = Var::from_tensor(&reference.head)?;
            let reference_logits = reference
                .model
                .hidden(&ids, 2, time)?
                .matmul(&head.as_tensor().t()?)?;
            let reference_loss = logits_cross_entropy(&reference_logits, &targets, None)?;
            assert!((loss.to_scalar::<f32>()? - reference_loss.to_scalar::<f32>()?).abs() <= 1e-6);
            let reference_grads = reference_loss.backward()?;
            // The straight-through gradient of every variable: the reference's
            // gradient of its served value, through the folded gain for a map.
            let variables = model.variables();
            let mut expected: BTreeMap<String, Tensor> = BTreeMap::new();
            for tensor in served_plan(&model.config)? {
                let served_grad = if tensor.name == SERVED_HEAD {
                    reference_grads.get(head.as_tensor())
                } else {
                    reference_grads.get(reference.model.variables()[&tensor.name].as_tensor())
                }
                .ok_or_else(|| invalid(format!("no reference gradient for {}", tensor.name)))?;
                let mut parts = Vec::new();
                match &tensor.kind {
                    ServedKind::Map {
                        gain: Some(gain), ..
                    } => {
                        let (w, g) = (
                            variables[&tensor.source].as_tensor(),
                            variables[gain].as_tensor(),
                        );
                        parts.push((tensor.source.clone(), served_grad.broadcast_mul(g)?));
                        parts.push((gain.clone(), served_grad.mul(w)?.sum(0)?));
                    }
                    _ => parts.push((tensor.source.clone(), served_grad.clone())),
                }
                for (name, part) in parts {
                    let total = match expected.remove(&name) {
                        Some(sum) => sum.add(&part)?,
                        None => part,
                    };
                    expected.insert(name, total);
                }
            }
            for (name, var) in variables {
                let got = grads
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid(format!("{name} has no gradient in served mode")))?;
                let values = got.flatten_all()?.to_vec1::<f32>()?;
                assert!(values.iter().all(|v| v.is_finite()), "{name}: nonfinite");
                let size = got.abs()?.max_all()?.to_scalar::<f32>()?;
                assert!(size > 0.0, "{pattern}: {name} has a zero gradient");
                let gap = max_abs_gap(got, &expected[name])?;
                assert!(
                    gap <= 1e-4 * size,
                    "{pattern}: {name}'s straight-through gradient differs by {gap} of {size}"
                );
            }
            // One update moves every variable, and the next served forward
            // reads the new values: it equals the new export's reference.
            let before: Vec<Vec<f32>> = variables
                .values()
                .map(|var| Ok(var.as_tensor().flatten_all()?.to_vec1::<f32>()?))
                .collect::<Result<_>>()?;
            let mut optimizer = StackAdamW::new(&model, 0.1, 1.0)?;
            optimizer.update(&model, &grads, 1e-3)?;
            for ((name, var), old) in variables.iter().zip(&before) {
                let new = var.as_tensor().flatten_all()?.to_vec1::<f32>()?;
                assert!(!same_bits(old, &new), "{pattern}: {name} did not move");
            }
            let updated = exported_reference(&model)?;
            let gap = max_abs_gap(
                &model.forward(&ids, 2, time)?,
                &updated.logits(&ids, 2, time)?,
            )?;
            assert!(
                gap <= 1e-5,
                "{pattern}: after an update the served forward is {gap} off"
            );
        }
        Ok(())
    }

    /// A lossless codec.
    struct Identity;

    impl MapCodec for Identity {
        fn name(&self) -> &str {
            "identity"
        }

        fn round_trip(&self, values: &[f32], _rows: usize, _cols: usize) -> Result<Vec<f32>> {
            Ok(values.to_vec())
        }
    }

    #[test]
    fn an_identity_codec_gives_the_plain_forward() -> Result<()> {
        for (pattern, read, rotation) in [
            ("rar", ReadScore::Lorentz, true),
            ("rrarra", ReadScore::Dot, false),
            ("rrarra", ReadScore::Lorentz, false),
        ] {
            let mut model = StackModel::new(exportable(pattern, read, rotation), &cpu())?;
            spread(&model, 83)?;
            // Norm gains of +-2^k: a gain folded into a map is then exact, as
            // it is applied to the normalized state, so the served forward
            // differs from the plain one in no rounding at all. (With other
            // gains the two round differently, and the Lorentz score amplifies
            // that; `served_forward_equals_the_exported_reference` covers them.)
            let mut rng = Initializer(85);
            for (name, var) in model.variables() {
                if name.ends_with("norm.weight") {
                    let gains: Vec<f32> = (0..var.elem_count())
                        .map(|_| {
                            let k = (rng.next() % 3) as i32 - 1;
                            let sign = if rng.next().is_multiple_of(5) {
                                -1.0
                            } else {
                                1.0
                            };
                            sign * 2f32.powi(k)
                        })
                        .collect();
                    var.set(&Tensor::from_vec(gains, var.shape(), &cpu())?)?;
                }
            }
            // The codec covers the maps; the scalars keep the export's grid
            // codes and fixed point. On their grid (where the scalar round
            // trip is the identity) the whole representation is lossless.
            let values = |model: &StackModel| -> Result<BTreeMap<String, Vec<f32>>> {
                model
                    .variables()
                    .iter()
                    .map(|(name, var)| {
                        Ok((
                            name.clone(),
                            var.as_tensor().flatten_all()?.to_vec1::<f32>()?,
                        ))
                    })
                    .collect()
            };
            let plan = served_plan(&model.config)?;
            let scalars: Vec<&ServedTensor> = plan
                .iter()
                .filter(|tensor| !matches!(tensor.kind, ServedKind::Map { .. }))
                .collect();
            let current = values(&model)?;
            for tensor in &scalars {
                let var = &model.variables()[&tensor.source];
                let on_grid = tensor.served_values(&current, &Identity)?;
                var.set(&Tensor::from_vec(on_grid, var.shape(), &cpu())?)?;
            }
            let current = values(&model)?;
            for tensor in &scalars {
                let again = tensor.served_values(&current, &Identity)?;
                assert!(
                    same_bits(&again, &current[&tensor.source]),
                    "{}",
                    tensor.name
                );
            }
            let time = model.config.context;
            let (ids, targets) = (token_ids(2 * time, 96, 19), token_ids(2 * time, 96, 23));
            let plain = model.forward(&ids, 2, time)?;
            let plain_grads = model.loss(&ids, &targets, 2, time)?.backward()?;
            model.set_served_representation(Some(Arc::new(Identity)))?;
            let label = format!("{pattern} {read:?} rotation {rotation}");
            assert_eq!(
                bits(&model.forward(&ids, 2, time)?)?,
                bits(&plain)?,
                "{label}: the identity codec's logits"
            );
            // Gradients agree too, up to summation order: a norm gain's
            // gradient gathers its terms through each map it is folded into.
            let served_grads = model.loss(&ids, &targets, 2, time)?.backward()?;
            for (name, var) in model.variables() {
                let (a, b) = (
                    served_grads
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid(name.clone()))?,
                    plain_grads
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid(name.clone()))?,
                );
                let size = b.abs()?.max_all()?.to_scalar::<f32>()?;
                let gap = max_abs_gap(a, b)?;
                assert!(
                    gap <= 1e-5 * size,
                    "{label}: {name}'s gradient differs by {gap} of {size}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn served_values_are_recomputed_exactly_when_their_variables_change() -> Result<()> {
        let mut model = StackModel::new(exportable("rar", ReadScore::Lorentz, true), &cpu())?;
        spread(&model, 89)?;
        let tensors = served_plan(&model.config)?.len();
        model.set_served_representation(Some(Arc::new(D11Interim)))?;
        let time = model.config.context;
        let ids = token_ids(time, 96, 29);
        let first = bits(&model.forward(&ids, 1, time)?)?;
        assert_eq!(first, bits(&model.forward(&ids, 1, time)?)?);
        let statistics = |model: &StackModel| -> Result<ServedStatistics> {
            model
                .served_statistics()?
                .ok_or_else(|| invalid("no served statistics"))
        };
        let seen = statistics(&model)?;
        assert_eq!((seen.checks, seen.refreshes, seen.tensors), (2, 1, tensors));
        // A decay feeds one served tensor; a norm gain the two maps it is
        // folded into; the embedding its lookup and the head.
        for (name, recomputed) in [
            ("layers.00.rec.decay", 1),
            ("layers.00.rec_norm.weight", 2),
            ("embedding.weight", 2),
        ] {
            let var = &model.variables()[name];
            var.set(&var.as_tensor().affine(1.25, 0.01)?)?;
            let before = statistics(&model)?;
            let logits = model.forward(&ids, 1, time)?;
            let after = statistics(&model)?;
            assert_eq!(after.refreshes, before.refreshes + 1, "{name}");
            assert_eq!(after.tensors, before.tensors + recomputed, "{name}");
            let gap = max_abs_gap(&logits, &exported_reference(&model)?.logits(&ids, 1, time)?)?;
            assert!(
                gap <= 1e-5,
                "after changing {name} the served forward is {gap} off"
            );
        }
        Ok(())
    }

    /// A codec that loses a value.
    struct Short;

    impl MapCodec for Short {
        fn name(&self) -> &str {
            "short"
        }

        fn round_trip(&self, values: &[f32], _rows: usize, _cols: usize) -> Result<Vec<f32>> {
            Ok(values[..values.len().saturating_sub(1)].to_vec())
        }
    }

    #[test]
    fn the_served_representation_takes_exportable_stacks_only() -> Result<()> {
        let codec: Arc<dyn MapCodec> = Arc::new(D11Interim);
        for config in [
            tiny(StackArch::Transformer, "aa", ReadScore::Dot, false),
            with_memory(exportable("ra", ReadScore::Dot, true), MemoryScore::Dot),
            // Width 16 is not a whole group.
            tiny(StackArch::Geometric, "ra", ReadScore::Dot, true),
        ] {
            let mut model = StackModel::new(config.clone(), &cpu())?;
            assert!(
                model
                    .set_served_representation(Some(codec.clone()))
                    .is_err(),
                "{config:?}"
            );
            assert!(model.served_codec().is_none());
        }
        let mut model = StackModel::new(exportable("ra", ReadScore::Dot, true), &cpu())?;
        model.set_served_representation(Some(Arc::new(Short)))?;
        assert!(model.forward(&token_ids(8, 96, 3), 1, 8).is_err());
        Ok(())
    }
}
