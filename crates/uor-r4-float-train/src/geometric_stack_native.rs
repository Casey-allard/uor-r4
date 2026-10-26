//! Qualified native stack extracted from geometric_stack.rs.
//! Historical source remains inactive; only CUDA FBS, geometric recurrence and shared writer execute.
#![allow(
    dead_code,
    unused_variables,
    unused_imports,
    clippy::derivable_impls,
    clippy::excessive_precision,
    clippy::items_after_test_module,
    clippy::manual_is_multiple_of,
    clippy::needless_range_loop
)]
use crate::{invalid, Result};
#[cfg(feature = "cuda")]
use candle_core::CudaStorage;
use candle_core::{CpuStorage, CustomOp2, DType, Device, Layout, Shape, Tensor, Var};
use serde::{Deserialize, Serialize};
#[cfg(feature = "cuda")]
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, sync::Arc};
#[cfg(feature = "cuda")]
#[path = "geometric_stack_native_cuda.rs"]
pub(crate) mod cuda_ops;
const CONVOLUTION_WIDTH: usize = 4;
const RMS_EPSILON: f64 = 1e-5;
const INITIAL_STD: f64 = 0.02;
const ROTATION_STD: f64 = 0.002;
pub const QUATERNION_WEIGHTS_FIELD: &str = "quaternion_weights";
pub const ACTIVATION_SNAP_FIELD: &str = "activation_snap";
pub const QUATERNION_SNAP_FIELD: &str = "quaternion_snap";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StackArch {
    Geometric,
    Transformer,
    /// Recurrence layers from `pattern=` with **ordinary attention** reads: the
    /// attribution control that separates the geometric read from the recurrent
    /// stack (`pattern` letters `r` and `a` are honoured; `a` layers run attention).
    Hybrid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Precision {
    /// Every activation and kernel in f32 (default).
    F32,
    /// Activations and matmul operands in bf16, accumulated in f32.
    Bf16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadScore {
    Lorentz,
    Dot,
    /// Matched flat control of [`ReadScore::Lorentz`]: `-beta (|q - k| -
    /// offset)`, the Euclidean distance in place of the hyperbolic one, with
    /// the same learned `beta` and `offset` per head (so the same parameter
    /// count). The squared distance is clamped below at `L2_MIN_SQUARED`.
    L2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotationGroup {
    /// Unit quaternions (S3): non-commuting lane transitions.
    Quaternion,
    /// Matched abelian control: the raw rotation quaternion's `j` and `k`
    /// components are zeroed before normalization, so every transport lies in
    /// the commutative subgroup `{a + b i}` (U(1)) and all transitions commute.
    /// Same gate parameters as [`RotationGroup::Quaternion`]; the zeroed
    /// components receive gradient exactly 0.
    U1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Objective {
    /// Cross-entropy over normalized-exponential logits (the implicit
    /// normalization is prohibited for new runs by D23 and retained so
    /// existing artifacts load and reproduce).
    Ce,
    /// `mean_k relu(1 − z_target + z_k)` over the top-`rivals` non-target
    /// logits of each row: a rank hinge with no exponential of any kind.
    MarginHinge { rivals: usize },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gate {
    /// `silu(g) · u` (implicit sigmoid; prohibited for new runs and retained
    /// so existing artifacts load and reproduce).
    Swiglu,
    /// `(g / |g|) ⊗ u` per 4-block — a rotation gate with no exponential of
    /// any kind; the hidden width must be divisible by 4.
    Hamilton,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecGates {
    /// Sigmoid opening, `exp` rate, tanh-GELU output gate.
    Historical,
    /// Clamped-linear opening, exact dyadic rate selection, Hamilton gate.
    Geometric,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuaternionWeights {
    /// Dense maps (the default).
    None,
    /// Every map of [`QUATERNION_MAPS`] quaternion-valued.
    All,
    /// Exact 8D SpiralCore BFS transport through retained FBS binders.
    SpiralcoreFbs,
}
impl Precision {
    /// `precision=` on the command line.
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "f32" => Ok(Precision::F32),
            "bf16" => Ok(Precision::Bf16),
            other => Err(invalid(format!("precision must be f32|bf16, got {other}"))),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Precision::F32 => "f32",
            Precision::Bf16 => "bf16",
        }
    }

    /// Whether the activations are bf16.
    pub fn is_bf16(self) -> bool {
        self == Precision::Bf16
    }

    /// The dtype of this precision's activations.
    pub fn activation_dtype(self) -> DType {
        match self {
            Precision::F32 => DType::F32,
            Precision::Bf16 => DType::BF16,
        }
    }

    /// The dtype of a parameter or moment (f32 in both precisions).
    pub fn parameter_dtype(self) -> DType {
        DType::F32
    }
}
impl Objective {
    /// The objective's own name, for reports and validation errors.
    pub fn name(self) -> &'static str {
        match self {
            Objective::Ce => "ce",
            Objective::MarginHinge { .. } => "margin_hinge",
        }
    }

    /// [`Objective::Ce`], the serialization default (a `Ce` configuration
    /// writes no field, so older configs stay byte-identical).
    pub fn is_ce(&self) -> bool {
        matches!(self, Objective::Ce)
    }
}
impl Gate {
    /// The gate's own name, for reports, flags and validation errors.
    pub fn name(self) -> &'static str {
        match self {
            Gate::Swiglu => "swiglu",
            Gate::Hamilton => "hamilton",
        }
    }

    /// [`Gate::Swiglu`], the serialization default (a `Swiglu` configuration
    /// writes no field, so older configs stay byte-identical).
    pub fn is_swiglu(&self) -> bool {
        matches!(self, Gate::Swiglu)
    }
}
impl RecGates {
    /// The mode's own name, for reports, flags and validation errors.
    pub fn name(self) -> &'static str {
        match self {
            RecGates::Historical => "historical",
            RecGates::Geometric => "geometric",
        }
    }

    /// [`RecGates::Historical`], the serialization default (only the geometric
    /// form writes the config field, so older configs stay byte-identical).
    pub fn is_historical(&self) -> bool {
        matches!(self, RecGates::Historical)
    }
}
impl QuaternionWeights {
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::All => "all",
            Self::SpiralcoreFbs => "spiralcore_fbs",
        }
    }

    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "none" => Ok(Self::None),
            "all" => Ok(Self::All),
            "spiralcore_fbs" => Ok(Self::SpiralcoreFbs),
            other => Err(invalid(format!(
                "invalid projection mode={other} (none, all, or spiralcore_fbs)"
            ))),
        }
    }
}
impl Default for Precision {
    fn default() -> Self {
        Self::F32
    }
}
impl Default for RotationGroup {
    fn default() -> Self {
        Self::Quaternion
    }
}
impl Default for Objective {
    fn default() -> Self {
        Self::Ce
    }
}
impl Default for Gate {
    fn default() -> Self {
        Self::Swiglu
    }
}
impl Default for RecGates {
    fn default() -> Self {
        Self::Historical
    }
}
fn is_quaternion(group: &RotationGroup) -> bool {
    *group == RotationGroup::Quaternion
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
fn layer_name(layer: usize, suffix: &str) -> String {
    format!("layers.{layer:02}.{suffix}")
}
fn quaternion_name(name: &str) -> Option<String> {
    let (_, rest) = name.split_once("layers.")?;
    let suffix = rest.get(3..)?;
    (QUATERNION_MAPS.contains(&suffix) || NATIVE_STARLING_MAPS.contains(&suffix))
        .then(|| format!("{}.quaternion", &name[..name.len() - ".weight".len()]))
}
fn spiralcore_fbs_shapes(
    shapes: BTreeMap<String, Vec<usize>>,
) -> Result<(BTreeMap<String, Vec<usize>>, NativePlans)> {
    let mut native = BTreeMap::new();
    let mut plans = BTreeMap::new();
    for (name, shape) in shapes {
        if quaternion_name(&name).is_some() {
            let [out, input] = shape.as_slice() else {
                return Err(invalid("native projection requires fresh dense-shape metadata, never old learned quaternion arrays"));
            };
            let plan = crate::spiralcore_fbs::ProjectionPlan::new(*input, *out)?;
            let stem = name
                .strip_suffix(".weight")
                .ok_or_else(|| invalid("invalid projection variable name"))?;
            native.insert(
                format!("{stem}.spiralcore_fbs"),
                plan.query_shape().to_vec(),
            );
            plans.insert((*input, *out), plan);
        } else {
            native.insert(name, shape);
        }
    }
    Ok((native, plans))
}
fn spiralcore_fbs_binding(plans: &NativePlans) -> serde_json::Value {
    serde_json::json!({"schema": crate::spiralcore_fbs::SCHEMA,
        "fbs_source_sha256": crate::spiralcore_fbs::FBS_SOURCE_SHA256,
        "plans": plans.iter().map(|(&(input, output), plan)| (format!("{input}/{output}"), plan.fingerprint.clone())).collect::<BTreeMap<_, _>>()})
}
fn cuda_gradient_clip(tensors: &[&Tensor], clip: f64) -> Result<(Tensor, Tensor)> {
    let first = tensors
        .first()
        .ok_or_else(|| invalid("no CUDA gradients to clip"))?;
    if !first.device().is_cuda()
        || tensors.iter().any(|tensor| {
            tensor.dtype() != DType::F32 || !tensor.device().same_device(first.device())
        })
        || !clip.is_finite()
        || clip < 0.0
        || (clip > 0.0 && clip < f32::MIN_POSITIVE as f64)
        || clip > f64::from(f32::MAX)
        || (clip > 0.0 && clip as f32 == 0.0)
    {
        return Err(invalid(
            "gradient clipping requires F32-representable configuration and same-device F32 CUDA gradients",
        ));
    }
    let squares = tensors
        .iter()
        .map(|grad| grad.sqr()?.sum_all()?.reshape(1))
        .collect::<candle_core::Result<Vec<_>>>()?;
    let norm = Tensor::cat(&squares, 0)?.sum_all()?.sqrt()?;
    let scale = if clip > 0.0 {
        Tensor::full(clip as f32, (), first.device())?
            .broadcast_div(&norm.clamp(clip, f32::MAX as f64)?)?
    } else {
        Tensor::ones((), DType::F32, first.device())?
    };
    Ok((norm, scale))
}
pub fn hamilton_gate(gate: &Tensor, up: &Tensor) -> Result<Tensor> {
    if gate.dtype() != DType::F32 || up.dtype() != DType::F32 {
        return Err(invalid("hamilton_gate requires F32 tensors"));
    }
    if gate.shape() != up.shape() {
        return Err(invalid("hamilton_gate gate and up shapes must match"));
    }
    if gate.elem_count() == 0 {
        return Err(invalid("hamilton_gate inputs must not be empty"));
    }
    if gate.elem_count() % 4 != 0 {
        return Err(invalid("hamilton_gate rotates 4-blocks"));
    }
    Ok(gate
        .contiguous()?
        .apply_op2(&up.contiguous()?, HamiltonGate)?)
}
pub fn rate_select(score: &Tensor) -> Result<Tensor> {
    if score.dtype() != DType::F32 {
        return Err(invalid("rate_select requires F32 tensors"));
    }
    if score.elem_count() == 0 {
        return Err(invalid("rate_select requires non-empty scores"));
    }
    Ok(score.contiguous()?.apply_op1(RateSelect)?)
}
pub fn quaternion_scan(transition: &Tensor, drive: &Tensor) -> Result<Tensor> {
    // The scan reads bf16 storage and multiplies in f32 under
    // `precision=bf16` (see [`Precision`]).
    if !matches!(transition.dtype(), DType::F32 | DType::BF16)
        || drive.dtype() != transition.dtype()
    {
        return Err(invalid(
            "quaternion_scan needs one f32 or bf16 dtype on both inputs",
        ));
    }
    let (batch, time, lanes, four) = transition.dims4()?;
    if four != 4 || drive.shape() != transition.shape() {
        return Err(invalid(
            "quaternion_scan needs matching [batch, time, lanes, 4] inputs",
        ));
    }
    if batch == 0 || time == 0 || lanes == 0 {
        return Err(invalid("quaternion_scan requires positive dimensions"));
    }
    Ok(transition
        .contiguous()?
        .apply_op2(&drive.contiguous()?, QuaternionScan)?)
}
fn decayed(name: &str, rank: usize) -> bool {
    (name.ends_with(".weight") && rank == 2 && !name.ends_with("conv.weight"))
        || (name.ends_with(".quaternion") && rank == 3)
}
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
pub struct StackAdamW {
    pub beta1: f64,
    pub beta2: f64,
    pub epsilon: f64,
    pub weight_decay: f64,
    pub clip: f64,
    pub step: usize,
    moments: BTreeMap<String, (Var, Var)>,
}
pub const QUATERNION_MAPS: [&str; 10] = [
    "rec.in.weight",
    "rec.gate.weight",
    "rec.out.weight",
    "read.query.weight",
    "read.key.weight",
    "read.value.weight",
    "read.out.weight",
    "mlp.gate.weight",
    "mlp.up.weight",
    "mlp.down.weight",
];
pub const NATIVE_STARLING_MAPS: [&str; 2] = [
    "read.native_starling.query_address.weight",
    "read.native_starling.key_address.weight",
];
pub const NATIVE_STARLING_FIELD: &str = "native_starling_read";
type NativePlans = BTreeMap<(usize, usize), Arc<crate::spiralcore_fbs::ProjectionPlan>>;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PointerConfig {
    pub dim: usize,
    #[serde(default)]
    pub score: ReadScore,
    #[serde(default)]
    pub identity: Option<serde_json::Value>,
    #[serde(default)]
    pub select: Option<serde_json::Value>,
    #[serde(default)]
    pub init_seed: Option<u64>,
    #[serde(default)]
    pub route: Option<serde_json::Value>,
}
impl PointerConfig {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            score: ReadScore::Dot,
            identity: None,
            select: None,
            init_seed: None,
            route: None,
        }
    }
}
impl Default for ReadScore {
    fn default() -> Self {
        Self::Dot
    }
}
/// Historical diagnostic metadata only; probability computation is excluded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerSelection {
    pub source: usize,
    pub attention: f64,
    pub gate: f64,
    pub raw_gate: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StackConfig {
    pub arch: StackArch,
    pub vocab_size: usize,
    pub width: usize,
    pub heads: usize,
    pub mlp_hidden: usize,
    pub context: usize,
    pub pattern: String,
    pub read: ReadScore,
    pub rotation: bool,
    #[serde(default, skip_serializing_if = "is_quaternion")]
    pub rotation_group: RotationGroup,
    pub seed: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub select: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer: Option<PointerConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_router: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Objective::is_ce")]
    pub objective: Objective,
    #[serde(default, skip_serializing_if = "Gate::is_swiglu")]
    pub gate: Gate,
    #[serde(default, skip_serializing_if = "RecGates::is_historical")]
    pub rec_gates: RecGates,
}
impl StackConfig {
    /// Historical shape constructor retained as metadata only. Model creation
    /// separately requires the native geometric configuration.
    pub fn transformer(
        width: usize,
        heads: usize,
        layers: usize,
        mlp_hidden: usize,
        context: usize,
        seed: u64,
    ) -> Result<Self> {
        let config = Self {
            width,
            heads,
            mlp_hidden,
            context,
            pattern: "a".repeat(layers),
            ..Self::transformer_control(seed)
        };
        config.validate()?;
        Ok(config)
    }
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
            rotation_group: RotationGroup::Quaternion,
            seed,
            memory: None,
            select: None,
            pointer: None,
            read_router: None,
            objective: Objective::Ce,
            gate: Gate::Swiglu,
            rec_gates: RecGates::Historical,
        }
    }
    pub fn layers(&self) -> usize {
        self.pattern.len()
    }
    pub fn head_width(&self) -> usize {
        self.width / self.heads.max(1)
    }
    pub fn validate(&self) -> Result<()> {
        if self.vocab_size == 0
            || self.width == 0
            || !self.width.is_multiple_of(4)
            || self.heads == 0
            || !self.width.is_multiple_of(self.heads)
            || !self.head_width().is_multiple_of(2)
            || self.mlp_hidden == 0
            || !(2..=4096).contains(&self.context)
            || self.pattern.is_empty()
        {
            return Err(invalid("stack shape is invalid"));
        }
        if !self.pattern.chars().all(|kind| kind == 'r' || kind == 'a') {
            return Err(invalid("stack pattern needs recurrence/read metadata"));
        }
        if self.arch == StackArch::Transformer && self.pattern.chars().any(|kind| kind != 'a') {
            return Err(invalid(
                "transformer metadata permits attention layers only",
            ));
        }
        if self.rotation_group == RotationGroup::U1 && !self.rotation {
            return Err(invalid("rotation_group=u1 requires rotation=true"));
        }
        if let Objective::MarginHinge { rivals } = self.objective {
            if !(1..=64).contains(&rivals) || self.pointer.is_some() {
                return Err(invalid(
                    "margin objective needs 1..64 rivals and no probability pointer",
                ));
            }
        }
        if self.rec_gates == RecGates::Geometric && self.arch != StackArch::Geometric {
            return Err(invalid(
                "geometric rate gates require geometric architecture",
            ));
        }
        if self.gate == Gate::Hamilton && !self.mlp_hidden.is_multiple_of(4) {
            return Err(invalid("Hamilton gate requires whole 4-blocks"));
        }
        if self.memory.is_some() || self.select.is_some() || self.read_router.is_some() {
            return Err(invalid(
                "historical memory/selection/router metadata is not admitted",
            ));
        }
        Ok(())
    }
    fn validate_native_config(&self) -> Result<()> {
        self.validate()?;
        if self.arch != StackArch::Geometric
            || !self.pattern.chars().all(|c| c == 'r')
            || self.memory.is_some()
            || self.select.is_some()
            || self.pointer.is_some()
            || self.read_router.is_some()
        {
            return Err(invalid("qualified stack requires recurrence-only geometric configuration without historical read/memory/pointer"));
        }
        if self.gate != Gate::Hamilton
            || self.rec_gates != RecGates::Geometric
            || !self.mlp_hidden.is_multiple_of(4)
        {
            return Err(invalid(
                "qualified stack requires geometric rate and Hamilton gates",
            ));
        }
        if let Objective::MarginHinge { rivals } = self.objective {
            if rivals == 0 {
                return Err(invalid("margin needs positive rivals"));
            }
        }
        Ok(())
    }
    pub fn shapes(&self) -> BTreeMap<String, Vec<usize>> {
        let (d, m, lanes) = (self.width, self.mlp_hidden, self.width / 4);
        let mut shapes = BTreeMap::from([
            ("embedding.weight".into(), vec![self.vocab_size, d]),
            ("final_norm.weight".into(), vec![d]),
        ]);
        for layer in 0..self.layers() {
            let n = |s: &str| layer_name(layer, s);
            shapes.insert(n("mlp_norm.weight"), vec![d]);
            for part in ["gate", "up"] {
                shapes.insert(n(&format!("mlp.{part}.weight")), vec![m, d]);
            }
            shapes.insert(n("mlp.down.weight"), vec![d, m]);
            let kind = self.pattern.as_bytes()[layer] as char;
            if self.arch == StackArch::Transformer
                || (self.arch == StackArch::Hybrid && kind != 'r')
            {
                shapes.insert(n("attn_norm.weight"), vec![d]);
                for part in ["q", "k", "v", "o"] {
                    shapes.insert(n(&format!("attn.{part}.weight")), vec![d, d]);
                }
                continue;
            }
            if kind != 'r' {
                shapes.insert(n("read_norm.weight"), vec![d]);
                for part in ["query", "key", "value", "out"] {
                    shapes.insert(n(&format!("read.{part}.weight")), vec![d, d]);
                }
                shapes.insert(n("read.null.weight"), vec![self.heads, d]);
                shapes.insert(n("read.null.bias"), vec![self.heads]);
                shapes.insert(n("read.age"), vec![self.heads, self.context]);
                if self.read != ReadScore::Dot {
                    shapes.insert(n("read.log_beta"), vec![self.heads]);
                    shapes.insert(n("read.offset"), vec![self.heads]);
                }
                continue;
            }
            let gates = lanes + if self.rotation { d } else { 0 };
            shapes.insert(n("rec_norm.weight"), vec![d]);
            shapes.insert(n("rec.in.weight"), vec![2 * d, d]);
            shapes.insert(n("rec.conv.weight"), vec![CONVOLUTION_WIDTH, d]);
            shapes.insert(n("rec.conv.bias"), vec![d]);
            shapes.insert(n("rec.gate.weight"), vec![gates, d]);
            shapes.insert(n("rec.gate.bias"), vec![gates]);
            shapes.insert(n("rec.decay"), vec![lanes]);
            shapes.insert(n("rec.out.weight"), vec![d, d]);
        }
        if let Some(pointer) = &self.pointer {
            shapes.insert("pointer.query.weight".into(), vec![pointer.dim, d]);
            shapes.insert("pointer.key.weight".into(), vec![pointer.dim, d]);
            shapes.insert("pointer.gate.weight".into(), vec![1, d]);
            shapes.insert("pointer.gate.bias".into(), vec![1]);
            if pointer.score == ReadScore::Lorentz {
                shapes.insert("pointer.log_beta".into(), vec![1]);
            }
        }
        shapes
    }
    pub fn parameter_count(&self) -> Result<usize> {
        self.validate()?;
        Ok(self
            .shapes()
            .values()
            .map(|s| s.iter().product::<usize>())
            .sum())
    }
    pub fn active_parameter_count(&self) -> Result<usize> {
        self.parameter_count()
    }
}
fn initial_variable_values(
    config: &StackConfig,
    rng: &mut Initializer,
    shape: &[usize],
    suffix: &str,
    residual_std: f64,
    lanes: usize,
) -> Vec<f32> {
    let count = shape.iter().product();
    match suffix {
        "final_norm.weight" | "mlp_norm.weight" | "rec_norm.weight" => vec![1.; count],
        "rec.conv.weight" => (0..count)
            .map(|index| if index < config.width { 1. } else { 0. })
            .collect(),
        "rec.conv.bias" => vec![0.; count],
        "rec.gate.bias" => (0..count)
            .map(|index| {
                if index >= lanes && (index - lanes).is_multiple_of(4) {
                    1.
                } else {
                    0.
                }
            })
            .collect(),
        "rec.decay" => (0..count)
            .map(|lane| ((lane % RATE_ENTRIES) as f32 + 0.5) / RATE_ENTRIES as f32)
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
        "rec.out.weight" | "mlp.down.weight" => (0..count)
            .map(|_| (rng.normal() * residual_std) as f32)
            .collect(),
        _ => (0..count)
            .map(|_| (rng.normal() * INITIAL_STD) as f32)
            .collect(),
    }
}
struct Params<'a> {
    vars: &'a BTreeMap<String, Var>,
    routes: BTreeMap<String, Tensor>,
}
impl Params<'_> {
    fn get(&self, name: &str) -> Result<&Tensor> {
        self.routes
            .get(name)
            .or_else(|| self.vars.get(name).map(Var::as_tensor))
            .ok_or_else(|| invalid(format!("missing parameter {name}")))
    }
    fn layer(&self, layer: usize, suffix: &str) -> Result<&Tensor> {
        self.get(&layer_name(layer, suffix))
    }
    fn head(&self) -> Result<&Tensor> {
        self.get("embedding.weight")
    }
}
pub struct StackModel {
    pub config: StackConfig,
    precision: Precision,
    variables: BTreeMap<String, Var>,
    device: Device,
    quaternion_weights: QuaternionWeights,
    spiralcore_fbs_plans: NativePlans,
    native_vector_head: Option<crate::native_vector_writer::WriterSpec>,
    native_starling: Option<crate::native_starling_spec::NativeStarling>,
}
impl StackModel {
    pub fn new(config: StackConfig, device: &Device) -> Result<Self> {
        config.validate_native_config()?;
        let mut rng = Initializer(config.seed ^ 0x6765_6F6D_5354_4143);
        let residual_std = INITIAL_STD / (2. * config.layers() as f64).sqrt();
        let lanes = config.width / 4;
        let mut variables = BTreeMap::new();
        for (name, shape) in config.shapes() {
            let suffix = name
                .rsplit_once("layers.")
                .map_or(name.as_str(), |(_, rest)| &rest[3..]);
            let values =
                initial_variable_values(&config, &mut rng, &shape, suffix, residual_std, lanes);
            variables.insert(name, Var::from_vec(values, shape.as_slice(), device)?);
        }
        Ok(Self {
            config,
            variables,
            device: device.clone(),
            precision: Precision::F32,
            quaternion_weights: QuaternionWeights::None,
            spiralcore_fbs_plans: BTreeMap::new(),
            native_vector_head: None,
            native_starling: None,
        })
    }
    pub fn quaternion_weights(&self) -> QuaternionWeights {
        self.quaternion_weights
    }
    pub fn set_quaternion_weights(&mut self, mode: QuaternionWeights) -> Result<()> {
        if mode == self.quaternion_weights {
            return Ok(());
        }
        if mode != QuaternionWeights::SpiralcoreFbs
            || self.quaternion_weights != QuaternionWeights::None
        {
            return Err(invalid(
                "only fresh SpiralCore/FBS projection construction is admitted",
            ));
        }
        self.config.validate_native_config()?;
        let shapes = self
            .variables
            .iter()
            .map(|(name, var)| (name.clone(), var.dims().to_vec()))
            .collect();
        let (shapes, plans) = spiralcore_fbs_shapes(shapes)?;
        let mut rng = Initializer(self.config.seed ^ 0x5350_4952_4642_5301);
        let mut replacements = Vec::new();
        for (name, shape) in &shapes {
            if let Some(stem) = name.strip_suffix(".spiralcore_fbs") {
                let values = (0..shape.iter().product::<usize>())
                    .map(|_| rng.normal() as f32)
                    .collect::<Vec<_>>();
                replacements.push((
                    format!("{stem}.weight"),
                    name.clone(),
                    Var::from_vec(values, shape.as_slice(), &self.device)?,
                ));
            }
        }
        for (old, new, variable) in replacements {
            self.variables.remove(&old);
            self.variables.insert(new, variable);
        }
        self.spiralcore_fbs_plans = plans;
        self.quaternion_weights = mode;
        Ok(())
    }
    /// The `Fbs_2Atom_Mix` arm of spiralcore-fbs-offline-float-sheet (#2891): every SpiralCore/FBS
    /// query `[outputs, stages, 8]` becomes `[outputs, 2 stages, 8]`. Each output's first path keeps
    /// its seed values; its second path is fresh, independently seeded normals. Run it only with
    /// that arm set (`spiralcore_fbs_arms::set_fbs_arm`), which reads both paths.
    pub fn widen_fbs_queries_two_path(&mut self) -> Result<()> {
        if self.quaternion_weights != QuaternionWeights::SpiralcoreFbs {
            return Err(invalid("two-path queries need a SpiralCore/FBS model"));
        }
        let mut rng = Initializer(self.config.seed ^ 0x3241_544F_4D5F_4D49);
        let names: Vec<String> = self
            .variables
            .keys()
            .filter(|name| name.ends_with(".spiralcore_fbs"))
            .cloned()
            .collect();
        for name in names {
            let var = &self.variables[&name];
            let &[outputs, stages, eight] = var.dims() else {
                return Err(invalid(
                    "SpiralCore/FBS queries must be [outputs, stages, 8]",
                ));
            };
            let first = var.as_tensor().flatten_all()?.to_vec1::<f32>()?;
            let row = stages * eight;
            let mut values = Vec::with_capacity(outputs * 2 * row);
            for output in 0..outputs {
                values.extend_from_slice(&first[output * row..(output + 1) * row]);
                values.extend((0..row).map(|_| rng.normal() as f32));
            }
            let widened = Var::from_vec(values, (outputs, 2 * stages, eight), &self.device)?;
            self.variables.insert(name, widened);
        }
        Ok(())
    }
    fn params(&self) -> Result<Params<'_>> {
        if !self.device.is_cuda()
            || self.precision != Precision::F32
            || self.quaternion_weights != QuaternionWeights::SpiralcoreFbs
        {
            return Err(invalid("qualified model execution requires F32 CUDA SpiralCore/FBS; no host or dense fallback"));
        }
        let routes = self
            .variables
            .iter()
            .filter_map(|(name, var)| {
                name.strip_suffix(".spiralcore_fbs")
                    .map(|stem| (format!("{stem}.weight"), var.as_tensor().clone()))
            })
            .collect();
        Ok(Params {
            vars: &self.variables,
            routes,
        })
    }
    pub fn validate_native_learning_path(&self) -> Result<()> {
        if self.native_starling.is_some() {
            return Err(invalid(
                "native murmuration read integration is pending; retired rank read cannot execute",
            ));
        }
        self.config.validate_native_config()?;
        self.params()?;
        self.native_vector_writer()?;
        if !matches!(self.config.objective, Objective::MarginHinge { .. }) {
            return Err(invalid("native learning requires margin objective"));
        }
        Ok(())
    }
    /// Canonical typed seam reserved for the approved murmuration integration.
    /// It cannot initialize or activate the retired rank/null table mechanism.
    pub fn set_native_starling_read(
        &mut self,
        spec: crate::native_starling_spec::NativeStarling,
    ) -> Result<()> {
        spec.validate()?;
        Err(invalid(
            "native murmuration read integration is pending; no historical rank read fallback",
        ))
    }
    pub fn native_starling_read(&self) -> Option<crate::native_starling_spec::NativeStarling> {
        self.native_starling
    }
    pub fn saved_native_starling(
        directory: &Path,
    ) -> Result<Option<crate::native_starling_spec::NativeStarling>> {
        let config: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.join("config.json"))?)?;
        let Some(value) = config.get(NATIVE_STARLING_FIELD) else {
            return Ok(None);
        };
        let spec: crate::native_starling_spec::NativeStarling =
            serde_json::from_value(value.clone())?;
        spec.validate()?;
        Ok(Some(spec))
    }
    /// Future device-only read seam. The S1 implementation must supply the
    /// actual addressed murmuration operator before this method can execute.
    fn native_starling_layer(
        &self,
        _addresses: &Tensor,
        _values: &Tensor,
        _ids: &[u32],
        spec: &crate::native_starling_spec::NativeStarling,
    ) -> Result<Tensor> {
        spec.validate()?;
        Err(invalid(
            "native murmuration read is not wired; no retired rank/null computation",
        ))
    }
    pub fn native_vector_writer(&self) -> Result<crate::native_vector_writer::NativeVectorWriter> {
        let spec = self
            .native_vector_head
            .as_ref()
            .ok_or_else(|| invalid("native head is not configured; use head=spiralcore_product"))?;
        if self.quaternion_weights != QuaternionWeights::SpiralcoreFbs
            || self.config.arch != StackArch::Geometric
            || self.config.pointer.is_some()
            || spec.width != self.config.width
            || spec.vocabulary != self.config.vocab_size
        {
            return Err(invalid("native writer configuration changed: geometry/projection, vector shape and no-pointer binding are required"));
        }
        let table = self
            .variables
            .get("embedding.weight")
            .ok_or_else(|| invalid("shared native vector table absent"))?;
        crate::native_vector_writer::NativeVectorWriter::new(table.as_tensor(), spec.clone())
    }
    pub fn native_vector_head_spec(&self) -> Option<&crate::native_vector_writer::WriterSpec> {
        self.native_vector_head.as_ref()
    }
    pub fn set_native_vector_head(&mut self, kept_cells: usize) -> Result<()> {
        if self.quaternion_weights != QuaternionWeights::SpiralcoreFbs
            || self.config.pointer.is_some()
        {
            return Err(invalid(
                "head=spiralcore_product requires native projection, no legacy pointer",
            ));
        }
        let spec = crate::native_vector_writer::WriterSpec {
            vocabulary: self.config.vocab_size,
            width: self.config.width,
            kept_cells,
        };
        spec.validate()?;
        self.native_vector_head = Some(spec);
        Ok(())
    }
    fn linear(&self, input: &Tensor, weight: &Tensor) -> Result<Tensor> {
        if weight.rank() == 3 {
            let input_width = *input
                .dims()
                .last()
                .ok_or_else(|| invalid("native projection needs carrier axis"))?;
            let output_width = weight
                .dim(0)?
                .checked_mul(8)
                .ok_or_else(|| invalid("native width overflow"))?;
            let plan = self
                .spiralcore_fbs_plans
                .get(&(input_width, output_width))
                .ok_or_else(|| invalid("missing bound native projection"))?;
            // A SpiralCore/FBS sheet arm (spiralcore-fbs-offline-float-sheet), when one is set.
            if let Some(arm) = crate::spiralcore_fbs_arms::fbs_arm() {
                return crate::spiralcore_fbs_arms::project(plan, weight, input, arm);
            }
            return plan.forward(weight, input);
        }
        if weight.dims() == [self.config.vocab_size, self.config.width]
            && self.native_vector_head.is_some()
        {
            let rows = input.elem_count() / self.config.width;
            return self.native_vector_writer()?.compatibility_scores(
                &input
                    .to_dtype(DType::F32)?
                    .reshape((rows, self.config.width))?,
            );
        }
        Err(invalid(
            "native projections cannot fall through to dense map",
        ))
    }
    fn norm(&self, p: &Params<'_>, input: &Tensor, gain: &str) -> Result<Tensor> {
        self.rms_norm(input, p.get(gain)?)
    }
    pub fn embed(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        self.params()?;
        if batch == 0
            || time == 0
            || time > self.config.context
            || batch.checked_mul(time) != Some(ids.len())
            || ids.iter().any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid(
                "stack forward needs batch*time vocabulary ids within context",
            ));
        }
        Ok(self
            .native_vector_writer()?
            .encode(ids)?
            .reshape((batch, time, self.config.width))?)
    }
    pub fn head(&self, hidden: &Tensor) -> Result<Tensor> {
        let p = self.params()?;
        self.linear(hidden, p.head()?)
    }
    pub fn hidden(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        self.validate_native_learning_path()?;
        let p = self.params()?;
        let mut x = self.embed(ids, batch, time)?;
        for layer in 0..self.config.layers() {
            x = x.add(&self.composed_recurrence(&p, layer, &x)?)?;
            x = x.add(&self.mlp(&p, layer, &x)?)?;
        }
        let x = self.norm(&p, &x, "final_norm.weight")?;
        Ok(x.reshape((batch * time, self.config.width))?)
    }
    fn mlp(&self, p: &Params<'_>, layer: usize, x: &Tensor) -> Result<Tensor> {
        let u = self.norm(p, x, &layer_name(layer, "mlp_norm.weight"))?;
        let gate = self.linear(&u, p.layer(layer, "mlp.gate.weight")?)?;
        let up = self.linear(&u, p.layer(layer, "mlp.up.weight")?)?;
        let mixed = gate
            .contiguous()?
            .apply_op2(&up.contiguous()?, HamiltonGate)?;
        let down = self.linear(&mixed, p.layer(layer, "mlp.down.weight")?)?;
        if crate::hamilton_sheet::is_capturing_gate_telemetry() {
            let unrotated = self.linear(&up, p.layer(layer, "mlp.down.weight")?)?;
            crate::hamilton_sheet::record_downstream_projection_norms(&down, &unrotated)?;
        }
        Ok(down)
    }
    pub fn forward(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        self.head(&self.hidden(ids, batch, time)?)
    }
    pub fn loss(&self, ids: &[u32], targets: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        let rivals = match self.config.objective {
            Objective::MarginHinge { rivals } => rivals,
            _ => return Err(invalid("historical likelihood is excluded")),
        };
        self.native_head_loss(&self.hidden(ids, batch, time)?, targets, rivals)
    }
    pub fn logits_with_transport(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        snap: Option<&[[f32; 4]]>,
    ) -> Result<Tensor> {
        if snap.is_some() {
            return Err(invalid("native projection has no host snap"));
        }
        self.forward(ids, batch, time)
    }
    pub fn add_pointer(&mut self, _pointer: PointerConfig, _seed: u64) -> Result<bool> {
        Err(invalid("legacy probability pointer is excluded"))
    }
    pub fn target_nll(
        &self,
        _ids: &[u32],
        _targets: &[u32],
        _batch: usize,
        _time: usize,
    ) -> Result<Vec<f64>> {
        Err(invalid("native vector writer does not expose likelihood"))
    }
    pub fn next_scores(&self, _ids: &[u32]) -> Result<Vec<f32>> {
        Err(invalid(
            "native writer emits device token ids, no host scores",
        ))
    }
    pub fn next_scores_with_pointer(
        &self,
        _ids: &[u32],
    ) -> Result<(Vec<f32>, Option<PointerSelection>)> {
        Err(invalid("legacy probability pointer is excluded"))
    }
    pub fn last_pointer_selection(&self, _ids: &[u32]) -> Result<Option<PointerSelection>> {
        Err(invalid("legacy probability pointer is excluded"))
    }
    pub fn native_head_candidates(
        &self,
        hidden: &Tensor,
    ) -> Result<crate::native_vector_writer::VectorCandidates> {
        self.native_vector_writer()?.candidates(hidden)
    }
    pub fn native_vector_channel(
        &self,
    ) -> Result<crate::native_vector_writer::NativeVectorChannel> {
        Ok(crate::native_vector_writer::NativeVectorChannel::new(
            self.native_vector_writer()?,
        ))
    }
    pub fn native_head_loss(
        &self,
        hidden: &Tensor,
        targets: &[u32],
        rivals: usize,
    ) -> Result<Tensor> {
        let writer = self.native_vector_writer()?;
        writer.margin(&writer.candidates(hidden)?, targets, rivals)
    }
    pub fn native_next_token_ids(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        let writer = self.native_vector_writer()?;
        writer.write(&self.hidden(ids, batch, time)?)
    }
    pub fn variables(&self) -> &BTreeMap<String, Var> {
        &self.variables
    }
    pub fn projection_plan_for_variable(
        &self,
        name: &str,
    ) -> Option<&Arc<crate::spiralcore_fbs::ProjectionPlan>> {
        let var = self.variables.get(name)?;
        if var.rank() != 3 || !name.ends_with(".spiralcore_fbs") {
            return None;
        }
        let output_width = var.dim(0).ok()? * 8;
        let input_width = if name.ends_with("mlp.down.spiralcore_fbs") {
            self.config.mlp_hidden
        } else {
            self.config.width
        };
        self.spiralcore_fbs_plans.get(&(input_width, output_width))
    }
    pub fn device(&self) -> &Device {
        &self.device
    }
    pub fn parameter_count(&self) -> usize {
        self.variables.values().map(|var| var.elem_count()).sum()
    }
    pub fn copy_variables_from(&self, source: &StackModel) -> Result<()> {
        if self.variables.len() != source.variables.len() {
            return Err(invalid("replica and source hold different variable sets"));
        }
        for (name, var) in &self.variables {
            let from = source
                .variables
                .get(name)
                .ok_or_else(|| invalid(format!("the source has no variable {name}")))?;
            if from.shape() != var.shape() {
                return Err(invalid(format!("variable {name} differs in shape")));
            }
            var.set(&from.as_tensor().to_device(&self.device)?)?;
        }
        Ok(())
    }
    pub fn precision(&self) -> Precision {
        self.precision
    }
    pub fn set_precision(&mut self, precision: Precision) {
        self.precision = precision;
    }
    pub fn scored_in_f32<T>(
        &mut self,
        scored: impl FnOnce(&mut StackModel) -> Result<T>,
    ) -> Result<T> {
        let precision = self.precision;
        self.precision = Precision::F32;
        let result = scored(self);
        self.precision = precision;
        result
    }
    fn rms_norm(&self, input: &Tensor, weight: &Tensor) -> Result<Tensor> {
        Ok(input
            .contiguous()?
            .apply_op2(&weight.contiguous()?, RmsNorm)?)
    }
    fn unit_norm(&self, input: &Tensor) -> Result<Tensor> {
        let ones = Tensor::ones(self.config.width, DType::F32, &self.device)?;
        self.rms_norm(input, &ones)
    }
    pub fn saved_quaternion_weights(directory: &Path) -> Result<QuaternionWeights> {
        let config: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.join("config.json"))?)?;
        if config.get("spiralcore_fbs_projection").is_some()
            && config
                .get(QUATERNION_WEIGHTS_FIELD)
                .and_then(|v| v.as_str())
                != Some("spiralcore_fbs")
        {
            return Err(invalid(
                "native projection binding requires the declared SpiralCore/FBS mode",
            ));
        }
        match config.get(QUATERNION_WEIGHTS_FIELD) {
            None => Ok(QuaternionWeights::None),
            Some(value) if value.as_str() == Some("all") => Ok(QuaternionWeights::All),
            Some(value) if value.as_str() == Some("spiralcore_fbs") => {
                Ok(QuaternionWeights::SpiralcoreFbs)
            }
            Some(_) => Err(invalid(format!(
                "config.json's {QUATERNION_WEIGHTS_FIELD} must be absent, all, or spiralcore_fbs"
            ))),
        }
    }
    fn composed_recurrence(&self, p: &Params<'_>, layer: usize, x: &Tensor) -> Result<Tensor> {
        let (batch, time, width) = x.dims3()?;
        let lanes = width / 4;
        let u = self.norm(p, x, &layer_name(layer, "rec_norm.weight"))?;
        let branches = self.linear(&u, p.layer(layer, "rec.in.weight")?)?;
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
        let gates = self
            .linear(&u, p.layer(layer, "rec.gate.weight")?)?
            .broadcast_add(p.layer(layer, "rec.gate.bias")?)?;
        // The decay rate. `RecGates::Geometric`: a clamped-linear opening score
        // plus the per-lane decay parameter read **raw** as a rate score selects
        // one entry of the fixed dyadic table ([`rate_select`]); no sigmoid, no
        // softplus and no `exp` in either direction. `RecGates::Historical`
        // keeps the exponential chain so existing configs and checkpoints
        // reproduce exactly (prohibited for new runs).
        let score = gates
            .narrow(2, 0, lanes)?
            .broadcast_add(&p.layer(layer, "rec.decay")?.reshape((1, 1, lanes))?)?
            .clamp(0., 1.)?;
        let selected = rate_select(&score)?;
        let (lambda, keep) = (selected.narrow(3, 0, 1)?, selected.narrow(3, 1, 1)?);
        let transition = if self.config.rotation {
            let raw = gates
                .narrow(2, lanes, width)?
                .reshape((batch, time, lanes, 4))?;
            let raw = match self.config.rotation_group {
                RotationGroup::Quaternion => raw,
                // The U(1) control: j and k zeroed before normalization.
                RotationGroup::U1 => {
                    raw.broadcast_mul(&Tensor::new(&[1f32, 1.0, 0.0, 0.0], &self.device)?)?
                }
            };
            let norm = raw.sqr()?.sum_keepdim(3)?.affine(1.0, 1e-6)?.sqrt()?;
            let unit = raw.broadcast_div(&norm)?;
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
        // The output gate. `RecGates::Geometric` rotates the recurrence output
        // by the gate's 4-lane quaternions ([`hamilton_gate`], OP1); the
        // historical mode keeps the tanh-GELU gate.
        let mixed = hamilton_gate(&gate, &state)?;
        let out = self.linear(&mixed, p.layer(layer, "rec.out.weight")?)?;
        if crate::hamilton_sheet::is_capturing_gate_telemetry() {
            let unrotated = self.linear(&state, p.layer(layer, "rec.out.weight")?)?;
            crate::hamilton_sheet::record_downstream_projection_norms(&out, &unrotated)?;
        }
        Ok(out)
    }
    pub fn save(&self, directory: &Path) -> Result<()> {
        self.config.validate_native_config()?;
        fs::create_dir_all(directory)?;
        let tensors: std::collections::HashMap<String, Tensor> = self
            .variables
            .iter()
            .map(|(name, var)| (name.clone(), var.as_tensor().clone()))
            .collect();
        candle_core::safetensors::save(&tensors, directory.join("model.safetensors"))?;
        let mut config = serde_json::to_vec_pretty(&self.config)?;
        if self.quaternion_weights != QuaternionWeights::None {
            let mut with_field: serde_json::Value = serde_json::from_slice(&config)?;
            with_field[QUATERNION_WEIGHTS_FIELD] = serde_json::to_value(self.quaternion_weights)?;
            if self.quaternion_weights == QuaternionWeights::SpiralcoreFbs {
                with_field["spiralcore_fbs_projection"] =
                    spiralcore_fbs_binding(&self.spiralcore_fbs_plans);
            }
            if let Some(spec) = &self.native_vector_head {
                with_field["head"] = serde_json::json!("spiralcore_product");
                with_field["native_vector_head"] = spec.binding()?;
            }
            config = serde_json::to_vec_pretty(&with_field)?;
        }
        if let Some(spec) = &self.native_starling {
            let mut with_field: serde_json::Value = serde_json::from_slice(&config)?;
            with_field[NATIVE_STARLING_FIELD] = serde_json::to_value(spec)?;
            config = serde_json::to_vec_pretty(&with_field)?;
        }
        fs::write(directory.join("config.json"), config)?;
        for record in [
            "read-identity-carry.json",
            "read-identity-latch.json",
            "geometric-address.json",
            "geometric-span.json",
            "transport.json",
        ] {
            match fs::remove_file(directory.join(record)) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
    pub fn load(directory: &Path, device: &Device) -> Result<Self> {
        let bytes = fs::read(directory.join("config.json"))?;
        let config: StackConfig = serde_json::from_slice(&bytes)?;
        config.validate_native_config()?;
        let saved: serde_json::Value = serde_json::from_slice(&bytes)?;
        for field in [
            "served_representation",
            "read_identity_carry",
            "read_identity_latch",
            "geometric_address",
            "geometric_span",
            "read_key_shift",
            "read_route",
            "read_weighting",
            "read_lineage",
            "activation_snap",
            "quaternion_snap",
            "e8_memory",
        ] {
            if saved.get(field).is_some() {
                return Err(invalid(format!(
                    "qualified artifact refuses historical field {field}"
                )));
            }
        }
        if Self::saved_native_starling(directory)?.is_some() {
            return Err(invalid("native read artifact requires the pending murmuration integration; retired rank read cannot load"));
        }
        if directory.join("transport.json").exists() {
            return Err(invalid(
                "qualified artifact refuses historical transport sidecar",
            ));
        }
        let mode = Self::saved_quaternion_weights(directory)?;
        if mode != QuaternionWeights::SpiralcoreFbs {
            return Err(invalid(
                "artifact must bind native SpiralCore/FBS projection",
            ));
        }
        let (shapes, plans) = spiralcore_fbs_shapes(config.shapes())?;
        if saved.get("spiralcore_fbs_projection") != Some(&spiralcore_fbs_binding(&plans)) {
            return Err(invalid(
                "native projection compiler/operator/FBS identity is missing or stale",
            ));
        }
        let tensors = candle_core::safetensors::load(directory.join("model.safetensors"), device)?;
        if tensors.len() != shapes.len() {
            return Err(invalid("saved stack tensors differ from configuration"));
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
        let mut model = Self {
            config,
            variables,
            device: device.clone(),
            precision: Precision::F32,
            quaternion_weights: mode,
            spiralcore_fbs_plans: plans,
            native_vector_head: None,
            native_starling: None,
        };
        if saved.get("head").and_then(|value| value.as_str()) == Some("spiralcore_product") {
            let binding = saved
                .get("native_vector_head")
                .ok_or_else(|| invalid("native vector head binding missing"))?;
            let spec: crate::native_vector_writer::WriterSpec =
                serde_json::from_value(binding["spec"].clone())?;
            if binding != &spec.binding()?
                || spec.vocabulary != model.config.vocab_size
                || spec.width != model.config.width
            {
                return Err(invalid(
                    "shared native vector head artifact identity missing or stale",
                ));
            }
            model.set_native_vector_head(spec.kept_cells)?;
        } else if saved.get("native_vector_head").is_some() || saved.get("head").is_some() {
            return Err(invalid("undeclared or unsupported native head"));
        }
        Ok(model)
    }
}
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
        candle_core::bail!("D24: native model operation requires CUDA; no CPU reference")
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        cuda_ops::quaternion_scan_fwd(s1, l1, s2, l2)
    }
    fn bwd(
        &self,
        transition: &Tensor,
        _drive: &Tensor,
        state: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        #[cfg(feature = "cuda")]
        if let Device::Cuda(device) = transition.device() {
            let (first, second) = cuda_ops::quaternion_scan_bwd(device, transition, state, grad)?;
            return Ok((Some(first), Some(second)));
        }
        candle_core::bail!("D24: native model gradient requires CUDA; no CPU reference")
    }
}
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
        candle_core::bail!("D24: native model operation requires CUDA; no CPU reference")
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        cuda_ops::rms_norm_fwd(s1, l1, s2, l2)
    }
    fn bwd(
        &self,
        x: &Tensor,
        w: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        #[cfg(feature = "cuda")]
        if let Device::Cuda(device) = x.device() {
            let (first, second) = cuda_ops::rms_norm_bwd(device, x, w, grad)?;
            return Ok((Some(first), Some(second)));
        }
        candle_core::bail!("D24: native model gradient requires CUDA; no CPU reference")
    }
}
struct HamiltonGate;
impl CustomOp2 for HamiltonGate {
    fn name(&self) -> &'static str {
        "geometric-stack-hamilton-gate"
    }
    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        candle_core::bail!("D24: native model operation requires CUDA; no CPU reference")
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        cuda_ops::hamilton_gate_fwd(s1, l1, s2, l2)
    }
    fn bwd(
        &self,
        gate: &Tensor,
        up: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        #[cfg(feature = "cuda")]
        if let Device::Cuda(device) = gate.device() {
            let (first, second) = cuda_ops::hamilton_gate_bwd(device, gate, up, grad)?;
            return Ok((Some(first), Some(second)));
        }
        candle_core::bail!("D24: native model gradient requires CUDA; no CPU reference")
    }
}
struct RateSelect;
impl candle_core::CustomOp1 for RateSelect {
    fn name(&self) -> &'static str {
        "geometric-stack-rate-select"
    }
    fn cpu_fwd(
        &self,
        storage: &CpuStorage,
        layout: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        candle_core::bail!("D24: native model operation requires CUDA; no CPU reference")
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        storage: &CudaStorage,
        layout: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        cuda_ops::rate_select_fwd(storage, layout)
    }
    fn bwd(
        &self,
        score: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<Option<Tensor>> {
        #[cfg(feature = "cuda")]
        if let Device::Cuda(device) = score.device() {
            return Ok(Some(cuda_ops::rate_select_bwd(device, score, grad)?));
        }
        candle_core::bail!("D24: native model gradient requires CUDA; no CPU reference")
    }
}
/// two in f32.
pub const RATE_TABLE: [f32; 16] = [
    0.5,
    0.25,
    0.125,
    0.0625,
    0.03125,
    0.015625,
    0.0078125,
    0.00390625,
    0.001953125,
    0.0009765625,
    0.00048828125,
    0.000244140625,
    0.0001220703125,
    6.103515625e-05,
    3.0517578125e-05,
    1.52587890625e-05,
];

/// `keep = sqrt(1 - lambda^2)` of [`RATE_TABLE`], precomputed to the nearest
/// f32 so the host and the device read identical bit patterns (`|keep^2 +
/// lambda^2 - 1| <= 6e-8` for every entry). The last four entries round to `1`
/// because `1 - 2^-26` is below f32 resolution.
pub const KEEP_TABLE: [f32; 16] = [
    0.8660253882408142,
    0.9682458639144897,
    0.9921567440032959,
    0.9980449676513672,
    0.9995115995407104,
    0.9998779296875,
    0.999969482421875,
    0.9999923706054688,
    0.9999980926513672,
    0.9999995231628418,
    0.9999998807907104,
    0.9999999403953552,
    1.0,
    1.0,
    1.0,
    1.0,
];

/// The number of rate-table entries: the index is `floor(score * RATE_ENTRIES)`
/// clamped to the last entry, so a score of exactly `k / RATE_ENTRIES` selects
/// entry `k`.
pub const RATE_ENTRIES: usize = 16;

/// The table index a clamped score selects.
#[inline]
pub fn rate_index(score: f32) -> usize {
    let index = (score * RATE_ENTRIES as f32) as usize;
    index.min(RATE_ENTRIES - 1)
}

/// Exact rate selection. Forward: `score -> (lambda, keep)` by table read.
#[cfg(test)]
mod tests {
    use super::*;

    /// Scalar 4-vector Hamilton product `p ⊗ q` matching `hamilton_gate_fwd` in `cuda_stack.cu:184-187`.
    #[inline]
    fn hamilton_product_4(p: [f32; 4], q: [f32; 4]) -> [f32; 4] {
        let ([a1, b1, c1, d1], [a2, b2, c2, d2]) = (p, q);
        [
            a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2,
            a1 * b2 + b1 * a2 + c1 * d2 - d1 * c2,
            a1 * c2 - b1 * d2 + c1 * a2 + d1 * b2,
            a1 * d2 + b1 * c2 - c1 * b2 + d1 * a2,
        ]
    }

    #[inline]
    fn norm4(v: [f32; 4]) -> f32 {
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2] + v[3] * v[3]).sqrt()
    }

    /// Closed-form scalar amplitude ratio `|g| / sqrt(|g|^2 + 1e-12)` in `f32`.
    #[inline]
    fn hamilton_gate_scalar_ratio(g_norm: f32) -> f32 {
        let r = (g_norm * g_norm + 1e-12_f32).sqrt();
        g_norm / r
    }

    /// Exact per-4-block forward arithmetic of `hamilton_gate_fwd` (`cuda_stack.cu:178-187`).
    #[inline]
    fn hamilton_gate_block_fwd(g: [f32; 4], u: [f32; 4]) -> ([f32; 4], f32, [f32; 4]) {
        let r = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2] + g[3] * g[3] + 1e-12_f32).sqrt();
        let q = [g[0] / r, g[1] / r, g[2] / r, g[3] / r];
        let out = hamilton_product_4(q, u);
        (out, r, q)
    }

    /// Exact per-4-block backward arithmetic of `hamilton_gate_bwd` (`cuda_stack.cu:199-219`).
    #[inline]
    fn hamilton_gate_block_bwd(g: [f32; 4], u: [f32; 4], d: [f32; 4]) -> ([f32; 4], [f32; 4], f32) {
        let r = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2] + g[3] * g[3] + 1e-12_f32).sqrt();
        let q = [g[0] / r, g[1] / r, g[2] / r, g[3] / r];
        let d_up = [
            q[0] * d[0] + q[1] * d[1] + q[2] * d[2] + q[3] * d[3],
            q[0] * d[1] - q[1] * d[0] - q[2] * d[3] + q[3] * d[2],
            q[0] * d[2] + q[1] * d[3] - q[2] * d[0] - q[3] * d[1],
            q[0] * d[3] - q[1] * d[2] + q[2] * d[1] - q[3] * d[0],
        ];
        let s = [
            d[0] * u[0] + d[1] * u[1] + d[2] * u[2] + d[3] * u[3],
            -d[0] * u[1] + d[1] * u[0] - d[2] * u[3] + d[3] * u[2],
            -d[0] * u[2] + d[1] * u[3] + d[2] * u[0] - d[3] * u[1],
            -d[0] * u[3] - d[1] * u[2] + d[2] * u[1] + d[3] * u[0],
        ];
        let dot = q[0] * s[0] + q[1] * s[1] + q[2] * s[2] + q[3] * s[3];
        let d_gate = [
            (s[0] - q[0] * dot) / r,
            (s[1] - q[1] * dot) / r,
            (s[2] - q[2] * dot) / r,
            (s[3] - q[3] * dot) / r,
        ];
        (d_gate, d_up, r)
    }

    #[test]
    fn hamilton_gate_suppression_closed_form_and_backward_hazard_proof() -> Result<()> {
        let gate_dirs: [[f32; 4]; 5] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.5, 0.5, 0.5, 0.5],
            [
                1.0 / 10.0_f32.sqrt(),
                -2.0 / 10.0_f32.sqrt(),
                2.0 / 10.0_f32.sqrt(),
                -1.0 / 10.0_f32.sqrt(),
            ],
            [3.0 / 13.0, 4.0 / 13.0, -12.0 / 13.0, 0.0],
        ];
        let value_blocks: [[f32; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.5, -1.5, 2.0, -0.5],
            [-3.0, 0.25, 1.25, 4.0],
            [0.125, 0.25, -0.375, 0.5],
        ];
        let decades: [f32; 13] = [
            1e-9, 1e-8, 1e-7, 1e-6, 1e-5, 1e-4, 1e-3, 1e-2, 1e-1, 1e0, 1e1, 1e2, 1e3,
        ];

        let mut min_decade_ratio = f32::INFINITY;
        for &g_norm in &decades {
            let expected_ratio = hamilton_gate_scalar_ratio(g_norm);
            for &g_dir in &gate_dirs {
                let g = [
                    g_dir[0] * g_norm,
                    g_dir[1] * g_norm,
                    g_dir[2] * g_norm,
                    g_dir[3] * g_norm,
                ];
                for &u in &value_blocks {
                    let (out, r, q) = hamilton_gate_block_fwd(g, u);
                    let u_norm = norm4(u);
                    let measured_ratio = norm4(out) / u_norm;
                    let q_norm = norm4(q);
                    assert!(
                        (measured_ratio - expected_ratio).abs() <= 1e-6,
                        "measured ratio {measured_ratio} != closed-form {expected_ratio} at |g|={g_norm}"
                    );
                    assert!(
                        (q_norm - expected_ratio).abs() <= 1e-6,
                        "|q| {q_norm} != closed-form {expected_ratio} at |g|={g_norm}"
                    );
                    let coupling = q_norm * q_norm + 1e-12_f32 / (r * r);
                    assert!(
                        (coupling - 1.0).abs() <= 2e-6,
                        "algebraic coupling |q|^2 + 1e-12/r^2 = {coupling} != 1 at |g|={g_norm}"
                    );
                    min_decade_ratio = min_decade_ratio.min(measured_ratio);
                }
            }
            if g_norm >= 1e-2 {
                // In f32, for |g| >= 1e-2, |g|^2 >= 1e-4 so adding 1e-12 is below 1 ULP.
                assert_eq!(
                    (g_norm * g_norm + 1e-12_f32).to_bits(),
                    (g_norm * g_norm).to_bits()
                );
                assert!((expected_ratio - 1.0).abs() <= 1e-7);
            }
        }
        println!(
            "hamilton_gate decade sweep [1e-9..1e3]: smallest ratio reached = {min_decade_ratio:.7e} (at |g|=1e-9); at |g|=0 ratio = 0.0"
        );
        assert!((min_decade_ratio - 1e-3).abs() <= 1e-6);

        // Exact zero gate block gives zero output and backward scale 1/r = 1e6.
        let (zero_out, zero_r, _) = hamilton_gate_block_fwd([0.0; 4], value_blocks[1]);
        assert_eq!(zero_out, [0.0; 4]);
        assert!((1.0 / zero_r - 1e6).abs() <= 1.0);

        // At |g| = 1e-6, ratio = 1/sqrt(2) ~= 0.70710677.
        let ratio_1e6 = hamilton_gate_scalar_ratio(1e-6_f32);
        assert!((ratio_1e6 - std::f32::consts::FRAC_1_SQRT_2).abs() <= 1e-6);

        // Registered Supp_Hamilton threshold tau = 0.5:
        // |out|/|u| <= 0.5 iff |g| <= 1e-6 / sqrt(3) ~= 5.7735027e-7,
        // where backward Jacobian norm on q^perp is 1/r >= (sqrt(3)/2) * 1e6 ~= 8.660254e5.
        let g_half = 1e-6_f32 / 3.0_f32.sqrt();
        let ratio_half = hamilton_gate_scalar_ratio(g_half);
        assert!((ratio_half - 0.5).abs() <= 1e-6);
        assert!(hamilton_gate_scalar_ratio(g_half * 1.01) > 0.5);

        for &scale in &[0.0_f32, 0.25, 0.5, 1.0] {
            let g = [g_half * scale, 0.0, 0.0, 0.0];
            let u = [1.0_f32, 0.0, 0.0, 0.0];
            let (out, r, _) = hamilton_gate_block_fwd(g, u);
            assert!(norm4(out) <= 0.5 + 1e-6);
            // Incoming gradient d = [0, 1, 0, 0] is orthogonal to q = [|q|, 0, 0, 0] when u = [1, 0, 0, 0],
            // so s = d ⊗ conj(u) = [0, 1, 0, 0] has q . s = 0 and |d_gate| = |s| / r = 1 / r.
            let (d_gate, d_up, _) = hamilton_gate_block_bwd(g, u, [0.0, 1.0, 0.0, 0.0]);
            let grad_norm = norm4(d_gate);
            let min_hazard = 0.5_f32 * 3.0_f32.sqrt() * 1e6_f32;
            assert!(
                (grad_norm - 1.0 / r).abs() <= 1.0,
                "|d_gate| {grad_norm} != 1/r {}",
                1.0 / r
            );
            assert!(
                grad_norm >= min_hazard - 1.0,
                "suppression region |g|<={g_half} must have |d_gate| >= {min_hazard}, got {grad_norm}"
            );
            assert!((norm4(d_up) - norm4(out)).abs() <= 1e-6);
        }

        // Candidate (b) contrast: unit gate q = [0, 1, 0, 0] steers u = [1, 0, 0, 0] orthogonal to
        // a single scalar read direction w = [1, 0, 0, 0] (<w, out> = 0), but preserves full 4-block norm |out| = |u| = 1.
        let (steered, _, _) = hamilton_gate_block_fwd([0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]);
        let w = [1.0_f32, 0.0, 0.0, 0.0];
        let dot_w = w[0] * steered[0] + w[1] * steered[1] + w[2] * steered[2] + w[3] * steered[3];
        assert_eq!(dot_w, 0.0);
        assert!((norm4(steered) - 1.0).abs() <= 1e-6);

        // Tensor entry point validation and D24 host refusal.
        let cpu = Device::Cpu;
        let g_tensor = Tensor::from_vec(vec![1.0_f32, 0.0, 0.0, 0.0], 4, &cpu)?;
        let u_tensor = Tensor::from_vec(vec![1.0_f32, 0.0, 0.0, 0.0], 4, &cpu)?;
        assert!(hamilton_gate(&g_tensor, &u_tensor).is_err());
        let bad_len = Tensor::from_vec(vec![1.0_f32, 0.0, 0.0], 3, &cpu)?;
        assert!(hamilton_gate(&bad_len, &bad_len).is_err());

        Ok(())
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn geometric_rate_cuda_uses_signed_edges_and_exact_keep_table() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let scores = Var::from_tensor(&Tensor::from_vec(vec![0.0f32, 0.1, 0.95], 3, &device)?)?;
        let selected = rate_select(scores.as_tensor())?;
        let incoming = Tensor::from_vec(vec![1.0f32, 0.0, 1.0, 0.0, 1.0, 0.0], (3, 2), &device)?;
        let gradients = selected.mul(&incoming)?.sum_all()?.backward()?;
        let actual = gradients
            .get(scores.as_tensor())
            .ok_or_else(|| invalid("CUDA rate score credit missing"))?
            .to_vec1::<f32>()?;
        assert_eq!(actual, vec![-4.0f32, -3.0, -0.000244140625]);

        // Score constants enumerate the declared table addresses, not a host
        // evaluation of the forward or its reference arithmetic.
        let addresses = Tensor::from_vec(
            (0..RATE_ENTRIES)
                .map(|i| i as f32 / 16.0)
                .collect::<Vec<_>>(),
            RATE_ENTRIES,
            &device,
        )?;
        let values = rate_select(&addresses)?.to_vec2::<f32>()?;
        for (index, value) in values.iter().enumerate() {
            assert_eq!(value[0].to_bits(), RATE_TABLE[index].to_bits());
            assert_eq!(value[1].to_bits(), KEEP_TABLE[index].to_bits());
        }
        Ok(())
    }
    #[cfg(feature = "cuda")]
    #[test]
    fn geometric_cuda_clipping_matches_stored_analytic_values() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let first = Tensor::from_vec(vec![3.0f32], 1, &device)?;
        let second = Tensor::from_vec(vec![4.0f32], 1, &device)?;
        let (norm, scale) = cuda_gradient_clip(&[&first, &second], 2.5)?;
        assert!(norm.device().is_cuda() && scale.device().is_cuda());
        assert_eq!(norm.to_scalar::<f32>()?, 5.0);
        assert_eq!(scale.to_scalar::<f32>()?, 0.5);
        assert_eq!(first.broadcast_mul(&scale)?.to_vec1::<f32>()?, vec![1.5]);
        assert_eq!(second.broadcast_mul(&scale)?.to_vec1::<f32>()?, vec![2.0]);
        for limit in [0.0, 10.0] {
            let (norm, scale) = cuda_gradient_clip(&[&first, &second], limit)?;
            assert_eq!(norm.to_scalar::<f32>()?, 5.0);
            assert_eq!(scale.to_scalar::<f32>()?, 1.0);
        }
        let zero = Tensor::zeros(2, DType::F32, &device)?;
        let (norm, scale) = cuda_gradient_clip(&[&zero], 2.5)?;
        assert_eq!(norm.to_scalar::<f32>()?, 0.0);
        assert_eq!(scale.to_scalar::<f32>()?, 1.0);
        let host_constant = Tensor::from_vec(vec![3.0f32], 1, &Device::Cpu)?;
        assert!(cuda_gradient_clip(&[&host_constant], 2.5).is_err());
        assert!(cuda_gradient_clip(&[&first], f64::NAN).is_err());
        assert!(cuda_gradient_clip(&[&first], 1e-100).is_err());
        assert!(cuda_gradient_clip(&[&zero], 1e-40).is_err());
        assert!(cuda_gradient_clip(&[&first], f64::from(f32::MAX) * 2.0).is_err());
        Ok(())
    }
    #[cfg(feature = "cuda")]
    #[test]
    fn geometric_rung0_cuda_ordinary_path_has_learning_credit() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let mut config = StackConfig::transformer_control(42);
        config.arch = StackArch::Geometric;
        config.pattern = "rrrrrr".into();
        config.width = 256;
        config.heads = 4;
        config.mlp_hidden = 384;
        config.context = 384;
        config.vocab_size = 4096;
        config.rotation = true;
        config.gate = Gate::Hamilton;
        config.rec_gates = RecGates::Geometric;
        config.objective = Objective::MarginHinge { rivals: 8 };
        let mut model = StackModel::new(config, &device)?;
        model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
        model.set_native_vector_head(8)?;
        let batch = 16;
        let time = 384;
        // Deterministic fixture token IDs, not a corpus or quality panel.
        let cycle = [0u32, 1, 0, 2, 1, 3, 2, 4];
        let next_cycle = [1u32, 0, 2, 1, 3, 2, 4, 5];
        let ids = (0..batch * time)
            .map(|i| cycle[i % cycle.len()])
            .collect::<Vec<_>>();
        let targets = (0..batch * time)
            .map(|i| next_cycle[i % next_cycle.len()])
            .collect::<Vec<_>>();
        let loss = model.loss(&ids, &targets, batch, time)?;
        assert!(loss.to_scalar::<f32>()?.is_finite());
        let gradients = loss.backward()?;
        for (name, variable) in model.variables() {
            let gradient = gradients
                .get(variable.as_tensor())
                .ok_or_else(|| invalid(format!("ordinary rung0 credit missing: {name}")))?;
            assert!(
                gradient.device().is_cuda(),
                "gradient moved off CUDA: {name}"
            );
            assert_eq!(gradient.shape(), variable.shape(), "gradient shape: {name}");
            assert_eq!(
                gradient.abs()?.le(f32::MAX)?.min_all()?.to_scalar::<u8>()?,
                1,
                "nonfinite ordinary rung0 credit: {name}"
            );
            if name.ends_with("rec.decay") || name.ends_with("rec.gate.spiralcore_fbs") {
                assert!(
                    gradient.abs()?.sum_all()?.to_scalar::<f32>()? > 0.0,
                    "geometric rate/opening credit disconnected at fresh initialization: {name}"
                );
            }
        }
        Ok(())
    }
    #[cfg(feature = "cuda")]
    #[test]
    fn geometric_rung0_cuda_composition_matches_zero_carrier_golden() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let mut config = StackConfig::transformer_control(42);
        config.arch = StackArch::Geometric;
        config.pattern = "rrrrrr".into();
        config.width = 256;
        config.heads = 4;
        config.mlp_hidden = 384;
        config.context = 384;
        config.vocab_size = 4096;
        config.rotation = true;
        config.gate = Gate::Hamilton;
        config.rec_gates = RecGates::Geometric;
        config.objective = Objective::MarginHinge { rivals: 1 };
        let mut model = StackModel::new(config, &device)?;
        model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
        model.set_native_vector_head(8)?;
        for variable in model.variables().values() {
            variable.set(&Tensor::zeros(variable.shape(), DType::F32, &device)?)?;
        }
        model.variables()["final_norm.weight"].set(&Tensor::ones(256, DType::F32, &device)?)?;
        let table = Tensor::cat(
            &[
                Tensor::zeros((1, 256), DType::F32, &device)?,
                Tensor::ones((4095, 256), DType::F32, &device)?,
            ],
            0,
        )?;
        model.variables()["embedding.weight"].set(&table)?;
        let hidden = model.hidden(&[0, 0], 1, 2)?;
        assert_eq!(hidden.dims(), &[2, 256]);
        assert_eq!(hidden.abs()?.sum_all()?.to_scalar::<f32>()?, 0.0);
        let writer = model.native_vector_writer()?;
        let candidates = writer.candidates(&hidden)?;
        assert_eq!(
            candidates.cell_ids().to_vec2::<u32>()?,
            vec![vec![0, 1, 2, 3, 4, 5, 6, 7]; 2]
        );
        assert_eq!(writer.write(&hidden)?.to_vec1::<u32>()?, vec![0, 0]);
        let admitted = writer.margin_components(&candidates, &[0, 0], 1)?;
        assert_eq!(admitted.token_margin.to_scalar::<f32>()?, 0.0);
        assert_eq!(admitted.address_margin.to_scalar::<f32>()?, 1.0);
        let omitted = writer.margin_components(&candidates, &[8, 8], 1)?;
        assert_eq!(omitted.token_margin.to_scalar::<f32>()?, 2.0);
        assert_eq!(omitted.address_margin.to_scalar::<f32>()?, 1.0);
        assert_eq!(model.loss(&[0, 0], &[8, 8], 1, 2)?.to_scalar::<f32>()?, 3.0);

        // These mutations must refuse before their historical/read paths run.
        model.config.rec_gates = RecGates::Historical;
        assert!(model.hidden(&[0], 1, 1).is_err());
        assert!(model.logits_with_transport(&[0], 1, 1, None).is_err());
        model.config.rec_gates = RecGates::Geometric;
        model.config.gate = Gate::Swiglu;
        assert!(model.hidden(&[0], 1, 1).is_err());
        assert!(model.logits_with_transport(&[0], 1, 1, None).is_err());
        model.config.gate = Gate::Hamilton;
        model.config.pattern = "rrrrra".into();
        assert!(model.hidden(&[0], 1, 1).is_err());
        assert!(model.logits_with_transport(&[0], 1, 1, None).is_err());
        model.config.pattern = "rrrrrr".into();
        model.set_precision(Precision::Bf16);
        assert!(model.hidden(&[0], 1, 1).is_err());
        Ok(())
    }
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
        self.update_filtered(model, grads, lr, |_| true)
    }

    /// One update at learning rate `lr` restricted to variables matching `include(name)`;
    /// returns the gradient norm across the included variable group before clipping.
    pub fn update_filtered(
        &mut self,
        model: &StackModel,
        grads: &candle_core::backprop::GradStore,
        lr: f64,
        include: impl Fn(&str) -> bool,
    ) -> Result<f64> {
        model.validate_native_learning_path()?;
        let mut total = 0f64;
        let mut device_scale = None;
        if model.device().is_cuda() {
            let present: Vec<&Tensor> = model
                .variables()
                .iter()
                .filter(|(name, _)| include(name))
                .filter_map(|(_, var)| grads.get(var.as_tensor()))
                .collect();
            if !present.is_empty() {
                let (norm, scale) = cuda_gradient_clip(&present, self.clip)?;
                // Reporting/error admission only: this scalar does not compute
                // or feed clipping, gradients, moments or parameter values.
                total = f64::from(norm.to_scalar::<f32>()?);
                device_scale = Some(scale);
            }
        } else {
            return Err(invalid(
                "D24: optimizer requires CUDA; no host model update",
            ));
        }
        let norm = if model.device().is_cuda() {
            total
        } else {
            total.sqrt()
        };
        if !norm.is_finite() {
            return Err(invalid("nonfinite gradient norm"));
        }
        let scale = if device_scale.is_some() {
            1.0
        } else if self.clip > 0.0 && norm > self.clip {
            self.clip / norm
        } else {
            1.0
        };
        self.step += 1;
        let first = 1.0 - self.beta1.powi(self.step as i32);
        let second = 1.0 - self.beta2.powi(self.step as i32);
        for (name, var) in model.variables() {
            if !include(name) {
                continue;
            }
            let Some(grad) = grads.get(var.as_tensor()) else {
                continue;
            };
            let clipped;
            let grad = if let Some(scale) = &device_scale {
                clipped = grad.broadcast_mul(scale)?;
                &clipped
            } else {
                grad
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
            #[cfg(feature = "cuda")]
            if let Device::Cuda(device) = var.device() {
                cuda_ops::adam_step_cuda(device, var, grad, m, v, &constants)?;
                continue;
            }
            return Err(invalid(
                "D24: optimizer requires CUDA; no host model update",
            ));
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
