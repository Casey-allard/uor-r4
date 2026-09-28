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

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use candle_core::{CpuStorage, CustomOp2, CustomOp3, DType, Device, Layout, Shape, Tensor, Var};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

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
        Ok(())
    }

    fn layer_kind(&self, layer: usize) -> char {
        self.pattern.as_bytes()[layer] as char
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
            shapes.insert(name("mlp.gate.weight"), vec![m, d]);
            shapes.insert(name("mlp.up.weight"), vec![m, d]);
            shapes.insert(name("mlp.down.weight"), vec![d, m]);
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
                "rec.conv.bias" | "read.null.bias" | "read.log_beta" => vec![0.0; count],
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

    fn weight(&self, name: &str) -> Result<&Tensor> {
        self.variables
            .get(name)
            .map(Var::as_tensor)
            .ok_or_else(|| invalid(format!("missing stack variable {name}")))
    }

    fn layer_weight(&self, layer: usize, suffix: &str) -> Result<&Tensor> {
        self.weight(&format!("layers.{layer:02}.{suffix}"))
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

    fn mlp(&self, layer: usize, x: &Tensor, capture: &mut Capture<'_>) -> Result<Tensor> {
        tap(capture, StackSite::Mlp(layer), || self.unit_norm(x))?;
        let u = self.rms_norm(x, self.layer_weight(layer, "mlp_norm.weight")?)?;
        let gate = Self::linear(&u, self.layer_weight(layer, "mlp.gate.weight")?)?;
        let up = Self::linear(&u, self.layer_weight(layer, "mlp.up.weight")?)?;
        let mixed = gate.contiguous()?.apply_op2(&up.contiguous()?, SwiGlu)?;
        tap(capture, StackSite::Down(layer), || Ok(mixed.clone()))?;
        Self::linear(&mixed, self.layer_weight(layer, "mlp.down.weight")?)
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

    fn attention(&self, layer: usize, x: &Tensor, capture: &mut Capture<'_>) -> Result<Tensor> {
        let (batch, time, _) = x.dims3()?;
        tap(capture, StackSite::Read(layer), || self.unit_norm(x))?;
        let u = self.rms_norm(x, self.layer_weight(layer, "attn_norm.weight")?)?;
        let project = |part: &str| -> Result<Tensor> {
            self.heads(
                &Self::linear(
                    &u,
                    self.layer_weight(layer, &format!("attn.{part}.weight"))?,
                )?,
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
        Self::linear(&merged, self.layer_weight(layer, "attn.o.weight")?)
    }

    fn geometric_read(
        &self,
        layer: usize,
        x: &Tensor,
        capture: &mut Capture<'_>,
    ) -> Result<Tensor> {
        let (batch, time, _) = x.dims3()?;
        let heads = self.config.heads;
        tap(capture, StackSite::Read(layer), || self.unit_norm(x))?;
        let u = self.rms_norm(x, self.layer_weight(layer, "read_norm.weight")?)?;
        let project = |part: &str| -> Result<Tensor> {
            self.heads(
                &Self::linear(
                    &u,
                    self.layer_weight(layer, &format!("read.{part}.weight"))?,
                )?,
                batch,
                time,
            )
        };
        let (query, key, value) = (project("query")?, project("key")?, project("value")?);
        let null = Self::linear(&u, self.layer_weight(layer, "read.null.weight")?)?
            .broadcast_add(self.layer_weight(layer, "read.null.bias")?)?
            .transpose(1, 2)?
            .flatten_all()?;
        let age = self
            .layer_weight(layer, "read.age")?
            .narrow(1, 0, time)?
            .flatten_all()?;
        let mut aux = vec![null, age];
        if self.config.read == ReadScore::Lorentz {
            aux.push(self.layer_weight(layer, "read.log_beta")?.exp()?);
            aux.push(self.layer_weight(layer, "read.offset")?.clone());
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
        Self::linear(&merged, self.layer_weight(layer, "read.out.weight")?)
    }

    fn recurrence(&self, layer: usize, x: &Tensor, capture: &mut Capture<'_>) -> Result<Tensor> {
        let (batch, time, width) = x.dims3()?;
        tap(capture, StackSite::Recurrence(layer), || self.unit_norm(x))?;
        let u = self.rms_norm(x, self.layer_weight(layer, "rec_norm.weight")?)?;
        let branches = Self::linear(&u, self.layer_weight(layer, "rec.in.weight")?)?;
        let gates = Self::linear(&u, self.layer_weight(layer, "rec.gate.weight")?)?
            .broadcast_add(self.layer_weight(layer, "rec.gate.bias")?)?;
        let parameters = Tensor::cat(
            &[
                &self.layer_weight(layer, "rec.conv.weight")?.flatten_all()?,
                self.layer_weight(layer, "rec.conv.bias")?,
                self.layer_weight(layer, "rec.decay")?,
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
        Self::linear(&core, self.layer_weight(layer, "rec.out.weight")?)
    }

    /// Logits [batch * time, vocabulary] for a batch of windows. Positions see
    /// only themselves and earlier positions of the same window.
    pub fn forward(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        let embedding = self.weight("embedding.weight")?;
        Ok(self.hidden(ids, batch, time)?.matmul(&embedding.t()?)?)
    }

    /// The final normalized states [batch * time, width], which the tied
    /// embedding maps to logits.
    pub fn hidden(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        self.hidden_hooked(ids, batch, time, &mut None)
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
        self.hidden_hooked(ids, batch, time, &mut Some(capture))
    }

    fn hidden_hooked(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        capture: &mut Capture<'_>,
    ) -> Result<Tensor> {
        let x = self.embed(ids, batch, time)?;
        self.layers_hooked(x, capture)
    }

    /// The token embeddings [batch, time, width] that the first layer reads.
    pub fn embed(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        if ids.len() != batch * time || time == 0 || time > self.config.context {
            return Err(invalid(
                "stack forward needs batch * time ids within the context",
            ));
        }
        if ids.iter().any(|&id| id as usize >= self.config.vocab_size) {
            return Err(invalid("token id outside the vocabulary"));
        }
        let embedding = self.weight("embedding.weight")?;
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
        self.layers_hooked(x, &mut None)
    }

    /// Logits [rows, vocabulary] from final states, through the tied embedding.
    pub fn head(&self, hidden: &Tensor) -> Result<Tensor> {
        Ok(hidden.matmul(&self.weight("embedding.weight")?.t()?)?)
    }

    fn layers_hooked(&self, mut x: Tensor, capture: &mut Capture<'_>) -> Result<Tensor> {
        let (batch, time, _) = x.dims3()?;
        for layer in 0..self.config.layers() {
            let mixed = match (self.config.arch, self.config.layer_kind(layer)) {
                (StackArch::Transformer, _) => self.attention(layer, &x, capture)?,
                (StackArch::Geometric, 'r') => self.recurrence(layer, &x, capture)?,
                (StackArch::Geometric, _) => self.geometric_read(layer, &x, capture)?,
            };
            x = x.add(&mixed)?;
            x = x.add(&self.mlp(layer, &x, capture)?)?;
        }
        tap(capture, StackSite::Head, || self.unit_norm(&x))?;
        let x = self.rms_norm(&x, self.weight("final_norm.weight")?)?;
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
        })
    }
}

#[cfg(test)]
impl StackModel {
    /// The recurrence mixer composed from Candle operations: the reference
    /// for the fused core.
    pub(super) fn composed_recurrence(&self, layer: usize, x: &Tensor) -> Result<Tensor> {
        let (batch, time, width) = x.dims3()?;
        let lanes = width / 4;
        let u = self.rms_norm(x, self.layer_weight(layer, "rec_norm.weight")?)?;
        let branches = Self::linear(&u, self.layer_weight(layer, "rec.in.weight")?)?;
        let input = branches.narrow(2, 0, width)?;
        let gate = branches.narrow(2, width, width)?;
        // Width-4 causal depthwise convolution over time.
        let weights = self.layer_weight(layer, "rec.conv.weight")?;
        let mut convolved = self
            .layer_weight(layer, "rec.conv.bias")?
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
        let gates = Self::linear(&u, self.layer_weight(layer, "rec.gate.weight")?)?
            .broadcast_add(self.layer_weight(layer, "rec.gate.bias")?)?;
        let opening = candle_nn::ops::sigmoid(&gates.narrow(2, 0, lanes)?)?;
        // log a = -softplus(-decay) keeps a in (0, 1); log lambda = c r log a.
        let log_a = self
            .layer_weight(layer, "rec.decay")?
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
            raw.broadcast_div(&norm)?.broadcast_mul(&lambda)?
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
            self.layer_weight(layer, "rec.out.weight")?,
        )
    }
}

/// Matrices take weight decay; norms, biases, decays, age tables, the
/// convolution taps and the Lorentz scalars do not.
fn decayed(name: &str, rank: usize) -> bool {
    name.ends_with(".weight") && rank == 2 && !name.ends_with("conv.weight")
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
            let grad = grad.affine(scale, 0.0)?;
            let m_next = m
                .as_tensor()
                .affine(self.beta1, 0.0)?
                .add(&grad.affine(1.0 - self.beta1, 0.0)?)?;
            let v_next = v
                .as_tensor()
                .affine(self.beta2, 0.0)?
                .add(&grad.sqr()?.affine(1.0 - self.beta2, 0.0)?)?;
            let step = m_next.affine(1.0 / first, 0.0)?.div(
                &v_next
                    .affine(1.0 / second, 0.0)?
                    .sqrt()?
                    .affine(1.0, self.epsilon)?,
            )?;
            let keep = if decayed(name, var.rank()) {
                1.0 - lr * self.weight_decay
            } else {
                1.0
            };
            var.set(
                &var.as_tensor()
                    .affine(keep, 0.0)?
                    .sub(&step.affine(lr, 0.0)?)?,
            )?;
            m.set(&m_next)?;
            v.set(&v_next)?;
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
            let fused = model.recurrence(0, &x, &mut None)?;
            let composed = model.composed_recurrence(0, &x)?;
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
                .recurrence(0, &x, &mut None)?
                .mul(&weights)?
                .sum_all()?;
            let composed_loss = model.composed_recurrence(0, &x)?.mul(&weights)?.sum_all()?;
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
        }
    }

    #[test]
    fn stacks_are_causal() -> Result<()> {
        for config in [
            tiny(StackArch::Transformer, "aa", ReadScore::Dot, false),
            tiny(StackArch::Geometric, "ra", ReadScore::Lorentz, true),
            tiny(StackArch::Geometric, "ar", ReadScore::Dot, false),
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
}
