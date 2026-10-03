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
//! the float variables by a straight-through estimator. A model saved in
//! served mode records its codec in `config.json`
//! ([`SavedServedRepresentation`]), so that an export can refuse to write a
//! representation the model did not train against
//! ([`crate::stack_export::check_export_representation`]).
//!
//! Trained-in transport snap: with a snap set
//! ([`StackModel::set_transport_snap`]), every recurrence replaces each unit
//! transport quaternion `u_t`, before its scaling by `lambda_t`, by the
//! nearest of a fixed set of unit quaternions. For [`TransportSnap::Icosian`]
//! that set is the 120 unit icosians of the binary icosian group 2I, so every
//! transport is an exact element of 2I. The gradient passes the snap
//! unchanged to `u_t` and on through its normalization to the rotation logits
//! (straight-through, `u + (snap(u) - u).detach()`). The fused recurrence core
//! applies it, so training runs at the fused path's speed; with no snap set the
//! core computes exactly what it computed before the snap existed.
//!
//! Flock selection (A1): with [`StackConfig::select`] set, every read row `t`
//! of every `a` layer and head (and of the control's attention) softmaxes over
//! a kept subset of the sources `0..=t` only. The subset is the shared
//! selector's ([`crate::flock::flock_select`], sink 0): the sink, the last
//! `window` positions and the `k` best-scoring remaining sources on the same
//! total score the softmax uses (ties to the lowest position); the NoRead slot
//! is not a position and stays outside the selection. Unkept sources get
//! weight exactly 0 and, in the backward, gradient exactly 0; the backward
//! calls the same function on the same scores, so its selection is identical.
//! Selection is compare and select only, so it has a D11 port in principle; no
//! port exists yet, so the exports refuse a model with a flock. With `select:
//! None`, or a selection that keeps every source of a row, the read computes
//! what it computed before the flock existed, bit for bit. The reads' `select`
//! never applies to the pointer.
//!
//! Pointer-copy head (A1): with [`StackConfig::pointer`] set, the final hidden
//! state `h_t` also drives a copy distribution. `q_t = W_q h_t` and
//! `k_j = W_k h_j` (width to `dim`, no bias) give the pointer's own score of
//! each source `j <= t`: `q_t . k_j / sqrt(dim)` for [`ReadScore::Dot`], and
//! for [`ReadScore::Lorentz`] `-beta arcosh(1 + e)` with the excess `e =
//! lift(q_t) lift(k_j) - q_t . k_j - 1`, `lift(x) = sqrt(1 + |x|^2)` (the form
//! of the fused Lorentz read) and the learned positive scale `beta =
//! exp(pointer.log_beta)`. The attention `a_t` is the softmax of the scores
//! over the sources the pointer's **own** [`PointerConfig::select`] keeps
//! ([`PointerSelect`]: the shared flock, or the `k` best alone, `TopK(1)`
//! being the single-source pointer; `None` keeps every source), so a single
//! kept source has weight 1. `p_copy(v | t) = sum_j a_tj [x_j = v]` copies the
//! input tokens at the attended positions. A gate `g_t = sigmoid(w_g . h_t +
//! b_g)` (`b_g` starts at -2) mixes it with the ordinary distribution: `p(v |
//! t) = (1 - g_t) softmax(z_t)[v] + g_t p_copy(v | t)`. When no kept source
//! holds the target, `p_copy` is exactly 0 and the mixture is `(1 - g_t)
//! softmax(z_t)[v]`; there is no probability floor. [`StackModel::weighted_loss`],
//! [`StackModel::loss`] and [`StackModel::target_nll`] score that mixture (in
//! log space), and [`StackModel::next_scores`] is the greedy generation entry
//! point. [`StackModel::forward`] still returns the raw logits `z`. Training
//! may stay soft (`select: None`); [`StackModel::set_pointer_select`] applies a
//! selection to the same weights afterwards. The pointer has no served
//! representation: `qat=true` and the integer exports refuse a model with one.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use candle_core::backend::BackendStorage;
#[cfg(feature = "cuda")]
use candle_core::CudaStorage;
use candle_core::{CpuStorage, CustomOp2, CustomOp3, DType, Device, Layout, Shape, Tensor, Var};
#[cfg(feature = "metal")]
use candle_core::{MetalStorage, Storage};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots;
use uor_r4_lut::GROUP;

use crate::flock::{self, FlockSelect};
use crate::geometric_address::{self, AddressWeights, GeometricAddressConfig};
use crate::geometric_context::{self, CompiledContext, ContextWeights};
use crate::geometric_event::{self, CompiledEvents, EventWeights};
use crate::geometric_potential_native::{self, CompiledGeometricPotentials};
use crate::geometric_span::{self, GeometricSpanConfig, SpanPolicy};
use crate::geometric_span_native::{self, CompiledSpanActions};
use crate::lut_export::{dequantize_matrix, quantize_matrix};
use crate::stack_export::{
    block, decay_of_rate, decay_rate, fixed, fixed_value, fold_columns, grid_code, grid_value, pad,
};
use crate::stack_memory::{keys_aux_len, product_key_memory, MemoryConfig, MemoryScore};
use crate::stack_prime_route::PrimeRegistry;
use crate::{invalid, Result};
use uor_r4_integer::geometric_span::SpanAction;

#[cfg(feature = "cuda")]
mod cuda_ops;

/// Lower bound on `z - 1` in the Lorentz score. Below it the distance is
/// clamped and carries no gradient.
const LORENTZ_MIN_EXCESS: f64 = 1e-7;
/// Lower bound on `|q - k|^2` in the L2 score. Below it the distance is
/// clamped and carries no gradient.
const L2_MIN_SQUARED: f64 = 1e-7;
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
    /// Matched flat control of [`ReadScore::Lorentz`]: `-beta (|q - k| -
    /// offset)`, the Euclidean distance in place of the hyperbolic one, with
    /// the same learned `beta` and `offset` per head (so the same parameter
    /// count). The squared distance is clamped below at `L2_MIN_SQUARED`.
    L2,
}

impl ReadScore {
    /// Whether the score carries the learned per-head scale and offset
    /// (`read.log_beta`, `read.offset`): Lorentz and its flat L2 control.
    pub fn scaled(self) -> bool {
        matches!(self, ReadScore::Lorentz | ReadScore::L2)
    }
}

/// The group of a recurrence's per-lane transport quaternions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotationGroup {
    /// Unit quaternions (S3): non-commuting lane transitions.
    #[default]
    Quaternion,
    /// Matched abelian control: the raw rotation quaternion's `j` and `k`
    /// components are zeroed before normalization, so every transport lies in
    /// the commutative subgroup `{a + b i}` (U(1)) and all transitions commute.
    /// Same gate parameters as [`RotationGroup::Quaternion`]; the zeroed
    /// components receive gradient exactly 0.
    U1,
}

fn is_quaternion(group: &RotationGroup) -> bool {
    *group == RotationGroup::Quaternion
}

/// Matched identity-input mechanisms for geometric reads. Both use the same
/// learned gate and current-role plus identity projections. Only Held retains
/// identity across multiple intervening positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadIdentityLatch {
    Held,
    Local,
}

/// Per-call research counterfactual, deliberately absent from model state and
/// saved metadata. Hard decisions have no surrogate gate gradient.
#[derive(Clone, Copy)]
enum LatchGates<'a> {
    Soft,
    Hard,
    Replay(&'a Tensor),
    Projections(&'a Tensor, &'a Tensor),
}

/// Original selected token rows are separate from the contextual residual.
/// A side-channel caller without that provenance cannot construct span actions.
#[derive(Clone, Copy)]
enum SpanEvents<'a> {
    Training(&'a Tensor),
    TrainingNative {
        logits: &'a Tensor,
        actions: &'a [SpanAction],
        age: AgeTrainingMode,
    },
    Native(&'a [SpanAction]),
}

#[derive(Clone, Copy)]
enum AgeTrainingMode {
    None,
    Quarter,
    PriorResidual,
}

#[derive(Clone, Copy)]
enum ContextInput<'a> {
    Training(&'a Tensor),
    Native(&'a [uor_r4_integer::geometric_potential::AddressLane]),
}

#[derive(Clone, Copy)]
struct ReadSource<'a> {
    tokens: Option<&'a Tensor>,
    span_policy: SpanPolicy,
    native_span: Option<(&'a CompiledSpanActions, &'a [u32])>,
    native_potential: Option<&'a CompiledGeometricPotentials>,
    event_control: Option<SpanEvents<'a>>,
    context: Option<ContextInput<'a>>,
    native_reducer: Option<&'a crate::geometric_read_native::CompiledGeometricRead>,
    native_trace: NativeReadTraceSink<'a>,
    native_value_projection: Option<NativeValueProjection<'a>>,
    learned_values: Option<LearnedGeometricValues<'a>>,
    native_learned_values: Option<&'a [i32]>,
    no_read: NoReadSource<'a>,
    composition: Option<ReadComposition<'a>>,
    // Admitted incremental reader output, already in the residual frame.
    // Conversion to F32 occurs outside the integer session.
    integer_residual: Option<(usize, &'a Tensor)>,
}

#[derive(Clone, Copy)]
enum NoReadSource<'a> {
    LegacyFloat,
    LearnedTensor(&'a Tensor),
    NativeQ24(&'a [i64]),
    ContextCredit { raw: &'a [i64], live: &'a Tensor },
}
#[derive(Clone, Copy)]
enum NoReadMode<'a> {
    Legacy,
    Learned(&'a crate::geometric_no_read::NoReadWeights),
    Native(&'a crate::geometric_no_read_native::CompiledNoRead),
}

#[derive(Clone, Copy)]
enum CompositionMode<'a> {
    LearnedPotential {
        weights: &'a crate::geometric_potential_q4::PotentialQ4Weights,
        composition: &'a crate::geometric_composition_native::CompiledComposition,
        output: &'a std::cell::RefCell<Option<crate::geometric_potential_q4::PotentialQ4Output>>,
    },
    Learned(&'a crate::geometric_composition::CompositionWeights),
    Native(&'a crate::geometric_composition_native::CompiledComposition),
    FrozenValues {
        producer: &'a crate::geometric_value_producer::ValueProducerWeights,
        composition: &'a crate::geometric_composition::CompositionWeights,
        output:
            &'a std::cell::RefCell<Option<crate::geometric_value_producer::ValueProducerOutput>>,
    },
}
#[derive(Clone, Copy)]
enum ReadComposition<'a> {
    ContextCredit {
        values: &'a Tensor,
        potential: &'a crate::geometric_potential_q4::PotentialQ4Output,
    },
    LearnedPotential {
        values: &'a Tensor,
        weights: &'a crate::geometric_potential_q4::PotentialQ4Weights,
        output: &'a std::cell::RefCell<Option<crate::geometric_potential_q4::PotentialQ4Output>>,
    },
    Learned(&'a Tensor),
    Native {
        values: &'a [i64],
        compiled: &'a crate::geometric_composition_native::CompiledComposition,
        trace:
            &'a std::cell::RefCell<Option<crate::geometric_composition_native::ComposedReadTrace>>,
    },
}

type NativeReadTraceSink<'a> =
    Option<&'a std::cell::RefCell<Option<crate::geometric_read_native::NativeReadTrace>>>;

#[derive(Clone, Copy)]
struct NativeValueProjection<'a> {
    residual: bool,
    compiled: &'a crate::geometric_value_native::CompiledGeometricValues,
    trace: &'a std::cell::RefCell<Option<crate::geometric_value_native::ValueProjectionTrace>>,
}

/// Offline answer-credit bridge. Native compilation remains a separate boundary.
#[derive(Clone, Copy)]
struct LearnedGeometricValues<'a> {
    producer: &'a crate::geometric_value_producer::ValueProducerWeights,
    latent: &'a Tensor,
    ids: &'a [u32],
}

impl Default for ReadSource<'_> {
    fn default() -> Self {
        Self {
            tokens: None,
            span_policy: SpanPolicy::Ordered,
            native_span: None,
            native_potential: None,
            event_control: None,
            context: None,
            native_reducer: None,
            native_trace: None,
            native_value_projection: None,
            learned_values: None,
            native_learned_values: None,
            no_read: NoReadSource::LegacyFloat,
            composition: None,
            integer_residual: None,
        }
    }
}

impl LatchGates<'_> {
    fn apply(self, logits: &Tensor) -> Result<Tensor> {
        match self {
            Self::Soft => Ok(candle_nn::ops::sigmoid(logits)?),
            Self::Replay(_) | Self::Projections(..) => {
                Err(invalid("numerical replay has no gate policy"))
            }
            Self::Hard => {
                if logits
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .any(|z| !z.is_finite())
                {
                    return Err(invalid("hard read identity latch needs finite gate logits"));
                }
                // Compare the logit itself: a tiny negative logit can have a
                // rounded sigmoid of exactly 0.5. Both signed zeros capture.
                Ok(logits.detach().ge(0.0)?.to_dtype(logits.dtype())?)
            }
        }
    }
}

fn read_identity_latch_shapes(config: &StackConfig) -> BTreeMap<String, Vec<usize>> {
    let mut shapes = BTreeMap::new();
    for layer in 0..config.layers() {
        if config.layer_kind(layer) == 'a' {
            for part in ["query_identity", "key_identity"] {
                shapes.insert(
                    layer_name(layer, &format!("read.{part}.weight")),
                    vec![config.width, config.width],
                );
            }
            shapes.insert(
                layer_name(layer, "read.identity_gate.weight"),
                vec![1, 2 * config.width],
            );
            shapes.insert(layer_name(layer, "read.identity_gate.bias"), vec![1]);
        }
    }
    shapes
}

/// A training label for one query's exact historical source occurrences.
/// Positions are local to the input window, not token identities: equal token
/// IDs at other positions receive no positive mass. Sources must be unique,
/// nonempty and strictly earlier than `query`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadBinding {
    pub batch: usize,
    pub query: usize,
    pub sources: Vec<usize>,
}

/// Teacher-only binding labels for one declared geometric read layer/head.
/// The initial interface permits at most one labelled query per batch item.
/// It requires full source admission; excluded positives cannot learn through
/// a hard selection mask. This object is never a model or checkpoint field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadBindingTarget {
    pub layer: usize,
    pub head: usize,
    pub rows: Vec<ReadBinding>,
}

/// Losses from the same forward graph. The caller chooses the coefficient in
/// `language + lambda * binding`. `masses` follows the label-row order and
/// `binding` is its mean negative log; no clipping or hidden coefficient is
/// applied. Only the ordinary value channels reach the language computation.
pub struct StackBindingLoss {
    pub language: Tensor,
    pub binding: Tensor,
    pub masses: Tensor,
}

struct BindingCapture<'a> {
    target: &'a ReadBindingTarget,
    masses: Option<Tensor>,
}

/// Refuse a flock the reads cannot evaluate. The selection itself is the shared
/// selector's ([`crate::flock::flock_select`]); a read row's sink is its first
/// source (position 0), the window keeps at least the row's own position, and
/// at least one older source is kept by score (`flock_select` rejects a zero
/// window or `k`, and a sink beyond the first row's position, at run time).
pub fn validate_flock(select: &FlockSelect) -> Result<()> {
    if select.sink != 0 {
        return Err(invalid(
            "a flock's sink is position 0, the first source of every row",
        ));
    }
    if select.window == 0 {
        return Err(invalid("a flock window is at least 1 position"));
    }
    if select.k == 0 {
        return Err(invalid(
            "a flock keeps at least 1 top-scoring source (k >= 1)",
        ));
    }
    Ok(())
}

/// Parse `none` or `flock:<window>:<k>` (the reads' selection, sink 0).
pub fn parse_flock_select(text: &str) -> Result<Option<FlockSelect>> {
    let bad = || {
        invalid(format!(
            "invalid flock selection {text:?} (none or flock:<window>:<k>)"
        ))
    };
    if text == "none" {
        return Ok(None);
    }
    match text.split(':').collect::<Vec<_>>().as_slice() {
        ["flock", window, k] => {
            let select = FlockSelect {
                sink: 0,
                window: window.parse().map_err(|_| bad())?,
                k: k.parse().map_err(|_| bad())?,
            };
            validate_flock(&select)?;
            Ok(Some(select))
        }
        _ => Err(bad()),
    }
}

/// Which of a pointer's sources `0..=t` its own softmax runs over (A1): the
/// shared flock (sink 0, the last `window` positions and the `k` best of the
/// rest), or only the `k` best-scoring sources with no sink and no window
/// ([`crate::flock::top_k_select`]). `TopK(1)` is the single-source pointer,
/// whose attention is 1 on the best-scoring source. The selection is the
/// pointer's own: [`StackConfig::select`] never applies to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PointerSelect {
    Flock(FlockSelect),
    TopK(usize),
}

impl PointerSelect {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Flock(select) => validate_flock(select),
            Self::TopK(0) => Err(invalid("a pointer keeps at least 1 source (k >= 1)")),
            Self::TopK(_) => Ok(()),
        }
    }

    /// The sources of `scores[..=t]` this selection keeps, by the shared
    /// selector on the pointer's own scores.
    fn kept(&self, scores: &[f32], t: usize) -> Result<flock::FlockSelection> {
        match *self {
            Self::Flock(select) => flock::flock_select(scores, t, select),
            Self::TopK(k) => flock::top_k_select(scores, t, k),
        }
    }
}

/// Parse `none`, `flock:<window>:<k>` or `top:<k>` (a pointer's selection).
pub fn parse_pointer_select(text: &str) -> Result<Option<PointerSelect>> {
    let bad = || {
        invalid(format!(
            "invalid pointer selection {text:?} (none, flock:<window>:<k> or top:<k>)"
        ))
    };
    if text == "none" {
        return Ok(None);
    }
    let select = match text.split(':').collect::<Vec<_>>().as_slice() {
        ["flock", _, _] => PointerSelect::Flock(parse_flock_select(text)?.ok_or_else(|| bad())?),
        ["top", k] => PointerSelect::TopK(k.parse().map_err(|_| bad())?),
        _ => return Err(bad()),
    };
    select.validate()?;
    Ok(Some(select))
}

/// A pointer route from its command-line form: `none`, `prime:<window>` (the
/// exact route, admitting by a shared atom), `prime-ranked:<window>`, or
/// `ngram:<window>` / `ngram-ranked:<window>` (admitting by the longest
/// ordered n-let match).
pub fn parse_pointer_route(text: &str) -> Result<Option<PrimeRoute>> {
    if text == "none" {
        return Ok(None);
    }
    let route = match text.split_once(':') {
        Some((kind @ ("prime" | "prime-ranked" | "ngram" | "ngram-ranked"), window)) => {
            let window = window.parse().map_err(|_| {
                invalid(format!(
                    "invalid pointer route {text:?} (none, prime:<window>, prime-ranked:<window>, ngram:<window> or ngram-ranked:<window>)"
                ))
            })?;
            let route = if kind.ends_with("-ranked") {
                PrimeRoute::ranked(window)
            } else {
                PrimeRoute::exact(window)
            };
            if kind.starts_with("ngram") {
                route.admitting(RouteAdmission::Ngram)
            } else {
                route
            }
        }
        _ => {
            return Err(invalid(format!(
                "invalid pointer route {text:?} (none or prime:<window>)"
            )))
        }
    };
    route.validate()?;
    Ok(Some(route))
}

/// The pointer-copy head (A1): `dim` is the width of its query and key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointerConfig {
    pub dim: usize,
    /// The pointer's own score of a source: `Dot` (the default, and the only
    /// score a configuration saved before this field can mean) or `Lorentz`,
    /// the fused read's hyperboloid form with the learned scale
    /// `pointer.log_beta`.
    #[serde(default = "default_pointer_score", skip_serializing_if = "is_dot")]
    pub score: ReadScore,
    /// The sources the pointer softmaxes over; `None` keeps them all. It is the
    /// pointer's own: [`StackConfig::select`] never applies to it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub select: Option<PointerSelect>,
    /// The seed the head's weights were drawn from when the head was added to
    /// a model that had none ([`StackModel::add_pointer`]); absent when the
    /// head was built with its model. A resume verifies it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init_seed: Option<u64>,
    /// Exact prime routing of the pointer's sources ([`PrimeRoute`]); `None`
    /// (the default, and what a configuration saved before this field means)
    /// keeps the learned scores.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<PrimeRoute>,
}

/// Exact prime routing for the pointer (the prime router of ADR-0003,
/// `docs/adr/0003-fixed-zeta-prime-route-attention.md`): its sources come from
/// arithmetic on registered primes, not from learned query/key scores.
///
/// Every token id has a registered prime ([`token_prime`]; frequent, low ids
/// get small primes). A position's key is the product of the distinct primes
/// of its last `window` tokens. At position `t`, a source `j` is admitted when
/// the key ending at `j - 1` lies wholly before the query's own tokens
/// (`j - 1 <= t - window`; an overlapping key shares the query's positions,
/// not an earlier occurrence) and shares a factor with the key ending at `t`
/// (`gcd > 1`), and the pointer then copies the token at `j`: the value that
/// followed a matching key. An admitted source is scored by `ln gcd` (shared
/// rare atoms weigh more than shared common ones) plus a recency term below
/// `ln 2`, which only orders sources with the same shared atoms; the
/// attention is the softmax of [`ROUTE_SHARPNESS`] times that score over the
/// admitted sources, and all zero when none is admitted. The pointer's gate,
/// which mixes the route's copy with the generated distribution, stays
/// learned; an exact route has no learned parameters of its own (its query
/// and key widths are unused and receive no gradient).
///
/// A **ranked** route keeps the admission and hands the ranking to the
/// pointer's learned score: an admitted source scores its learned score plus
/// [`ROUTE_SHARPNESS`] times its route score, and the attention is the softmax
/// of that over the admitted sources; with none admitted it is the learned
/// pointer's softmax over every source, so a position no key matches (the
/// first token of a reply) can still copy by content. Admission is exact
/// identity; ranking among sources that share the same atoms (a fact and the
/// question that repeats its words) is learned, and the query and key get
/// their gradient as under a selection.
///
/// Admission is [`RouteAdmission::SharedAtom`] as above, or
/// [`RouteAdmission::Ngram`]: the ordered n-let identity of ADR-0003's
/// transition indexes with divisor fallback (I2 before I1). For the longest
/// `n <= window` with any match, a source `j` is admitted when the `n` tokens
/// before it equal, in order, the query's last `n` (on registered primes,
/// whose assignment is injective, this is equality of the ordered prime
/// sequences), the key again wholly before the query's tokens; shorter `n`
/// are tried only when no longer one matches. An admitted source scores the
/// matched n-let's prime weight (the sum of `ln p` over its atoms, so a rare
/// n-let outweighs a common one in the exact route's sharpness) plus recency.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimeRoute {
    /// Tokens in each key, 1 to [`MAX_ROUTE_WINDOW`].
    pub window: usize,
    /// Rank admitted sources by the learned score plus the route's; `false`
    /// (the default, and what a route saved before this field means) is the
    /// exact route.
    #[serde(default, skip_serializing_if = "is_false")]
    pub ranked: bool,
    /// Which sources the route admits; a route saved before this field
    /// admits by a shared atom.
    #[serde(default, skip_serializing_if = "RouteAdmission::is_shared_atom")]
    pub admission: RouteAdmission,
}

/// How a [`PrimeRoute`] admits a source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteAdmission {
    /// Any atom shared by the key before the source and the query's key
    /// (`gcd > 1`).
    #[default]
    SharedAtom,
    /// The longest ordered n-let match, backing off to shorter n-lets.
    Ngram,
}

impl RouteAdmission {
    fn is_shared_atom(&self) -> bool {
        *self == Self::SharedAtom
    }
}

fn is_false(flag: &bool) -> bool {
    !*flag
}

/// The longest key: six distinct primes below the table's bound fit a u128.
pub const MAX_ROUTE_WINDOW: usize = 6;
/// The fixed sharpness of the route's softmax over `ln gcd` scores.
pub const ROUTE_SHARPNESS: f64 = 4.0;
/// The weight of recency in a route score, below `ln 2`.
const ROUTE_RECENCY: f64 = 0.5;
/// How many token ids have a registered prime.
const ROUTE_PRIMES: usize = 65_536;

impl PrimeRoute {
    /// The exact route of `window`-token keys.
    pub fn exact(window: usize) -> Self {
        Self {
            window,
            ranked: false,
            admission: RouteAdmission::SharedAtom,
        }
    }

    /// The ranked route of `window`-token keys.
    pub fn ranked(window: usize) -> Self {
        Self {
            ranked: true,
            ..Self::exact(window)
        }
    }

    /// The same route admitting by `admission`.
    pub fn admitting(self, admission: RouteAdmission) -> Self {
        Self { admission, ..self }
    }

    pub fn validate(&self) -> Result<()> {
        if self.window == 0 || self.window > MAX_ROUTE_WINDOW {
            return Err(invalid(format!(
                "a prime route's key window is 1 to {MAX_ROUTE_WINDOW} tokens"
            )));
        }
        Ok(())
    }
}

/// The registered prime of token `id` (the relation store's
/// [`PrimeRegistry`]: the `(id + 1)`-th prime), for ids below
/// [`ROUTE_PRIMES`]; `None` above (such a token is no atom).
pub fn token_prime(id: u32) -> Option<u64> {
    static REGISTRY: OnceLock<Option<PrimeRegistry>> = OnceLock::new();
    REGISTRY
        .get_or_init(|| PrimeRegistry::new(ROUTE_PRIMES).ok())
        .as_ref()?
        .prime(id)
        .ok()
}

/// The product of the distinct primes of the `window` tokens of `ids` ending
/// at `end` (fewer at the start).
fn route_key(ids: &[u32], end: usize, window: usize) -> u128 {
    let start = (end + 1).saturating_sub(window);
    let mut key = 1u128;
    for &id in &ids[start..=end] {
        if let Some(p) = token_prime(id) {
            let p = u128::from(p);
            if !key.is_multiple_of(p) {
                key *= p;
            }
        }
    }
    key
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// The sources the route admits at position `t` of a window whose tokens are
/// `ids[..=t]` (see [`PrimeRoute`]), with their route scores, in position
/// order.
fn route_scores(ids: &[u32], t: usize, route: PrimeRoute) -> Vec<(usize, f64)> {
    match route.admission {
        RouteAdmission::SharedAtom => shared_atom_scores(ids, t, route.window),
        RouteAdmission::Ngram => ngram_scores(ids, t, route.window),
    }
}

/// The longest ordered n-let match (see [`RouteAdmission::Ngram`]).
fn ngram_scores(ids: &[u32], t: usize, window: usize) -> Vec<(usize, f64)> {
    for n in (1..=window.min(t + 1)).rev() {
        let query = &ids[t + 1 - n..=t];
        let weight: f64 = query
            .iter()
            .filter_map(|&id| token_prime(id))
            .map(|p| (p as f64).ln())
            .sum();
        // A source's n-let ends at `j - 1 <= t - n`, before the query's.
        let matches: Vec<(usize, f64)> = (n..=t + 1 - n)
            .filter(|&j| ids[j - n..j] == *query)
            .map(|j| (j, weight + ROUTE_RECENCY * j as f64 / (t + 1) as f64))
            .collect();
        if !matches.is_empty() {
            return matches;
        }
    }
    Vec::new()
}

/// Any shared atom (see [`RouteAdmission::SharedAtom`]): `ln gcd` plus recency.
fn shared_atom_scores(ids: &[u32], t: usize, window: usize) -> Vec<(usize, f64)> {
    let query = route_key(ids, t, window);
    // Sources whose key ends before the query's first token.
    let last = (t + 1).saturating_sub(window);
    (1..=last)
        .filter_map(|j| {
            let shared = gcd(query, route_key(ids, j - 1, window));
            (shared > 1).then(|| {
                (
                    j,
                    (shared as f64).ln() + ROUTE_RECENCY * j as f64 / (t + 1) as f64,
                )
            })
        })
        .collect()
}

/// The softmax of `scores` over their sources, as weights over `0..=t`
/// (exactly 0 elsewhere, and all 0 for no scores).
fn admitted_softmax(t: usize, scores: &[(usize, f64)]) -> Vec<f64> {
    let mut weights = vec![0f64; t + 1];
    let Some(top) = scores.iter().map(|&(_, s)| s).reduce(f64::max) else {
        return weights;
    };
    let mut total = 0.0;
    for &(j, score) in scores {
        weights[j] = (score - top).exp();
        total += weights[j];
    }
    for weight in &mut weights {
        *weight /= total;
    }
    weights
}

/// The exact route's attention of position `t` (see [`PrimeRoute`]): the
/// softmax of [`ROUTE_SHARPNESS`] times the route scores of the admitted
/// sources.
fn route_attention(ids: &[u32], t: usize, route: PrimeRoute) -> Vec<f64> {
    let scores: Vec<(usize, f64)> = route_scores(ids, t, route)
        .into_iter()
        .map(|(j, score)| (j, ROUTE_SHARPNESS * score))
        .collect();
    admitted_softmax(t, &scores)
}

/// The ranked route's attention of position `t` (see [`PrimeRoute`]) from the
/// pointer's learned scores of the sources `0..=t`.
fn ranked_attention(
    learned: Vec<f32>,
    ids: &[u32],
    t: usize,
    route: PrimeRoute,
) -> candle_core::Result<Vec<f64>> {
    let admitted = route_scores(ids, t, route);
    if admitted.is_empty() {
        return pointer_weights(learned, None);
    }
    let scores: Vec<(usize, f64)> = admitted
        .into_iter()
        .map(|(j, score)| (j, f64::from(learned[j]) + ROUTE_SHARPNESS * score))
        .collect();
    Ok(admitted_softmax(t, &scores))
}

fn default_pointer_score() -> ReadScore {
    ReadScore::Dot
}

fn is_dot(score: &ReadScore) -> bool {
    *score == ReadScore::Dot
}

impl PointerConfig {
    /// A Dot pointer of width `dim` that softmaxes over every source.
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            score: ReadScore::Dot,
            select: None,
            init_seed: None,
            route: None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.dim == 0 {
            return Err(invalid("the pointer head needs a positive query width"));
        }
        if let Some(select) = &self.select {
            select.validate()?;
        }
        if let Some(route) = &self.route {
            route.validate()?;
            if self.select.is_some() {
                return Err(invalid("a routed pointer has no selection"));
            }
        }
        if self.score == ReadScore::L2 {
            return Err(invalid(
                "the pointer head scores with dot or lorentz; the L2 control is a read score only",
            ));
        }
        Ok(())
    }
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
    /// The transport's group with `rotation`: quaternion (default, absent from
    /// configurations saved before the control existed) or the abelian U(1)
    /// control.
    #[serde(default, skip_serializing_if = "is_quaternion")]
    pub rotation_group: RotationGroup,
    pub seed: u64,
    /// Product-key memories in place of some layers' MLPs
    /// ([`crate::stack_memory`]); absent from configurations without one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryConfig>,
    /// Flock selection of every read (the geometric reads and the control's
    /// attention) by [`crate::flock::flock_select`], its sink at position 0;
    /// absent from configurations without one, which read every source as
    /// before. It does not apply to the pointer, which has its own
    /// ([`PointerConfig::select`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub select: Option<FlockSelect>,
    /// The pointer-copy head after the final norm; absent from configurations
    /// without one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer: Option<PointerConfig>,
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
            rotation_group: RotationGroup::Quaternion,
            seed,
            memory: None,
            select: None,
            pointer: None,
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
            rotation_group: RotationGroup::Quaternion,
            seed,
            memory: None,
            select: None,
            pointer: None,
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
        if let Some(select) = &self.select {
            validate_flock(select)?;
        }
        if let Some(pointer) = &self.pointer {
            pointer.validate()?;
        }
        if self.rotation_group == RotationGroup::U1 && !self.rotation {
            return Err(invalid(
                "rotation_group=u1 restricts the learned transport, so it needs rotation=true",
            ));
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
                    if self.read.scaled() {
                        shapes.insert(name("read.log_beta"), vec![self.heads]);
                        shapes.insert(name("read.offset"), vec![self.heads]);
                    }
                }
            }
        }
        if let Some(pointer) = &self.pointer {
            shapes.insert("pointer.query.weight".to_owned(), vec![pointer.dim, d]);
            shapes.insert("pointer.key.weight".to_owned(), vec![pointer.dim, d]);
            shapes.insert("pointer.gate.weight".to_owned(), vec![1, d]);
            shapes.insert("pointer.gate.bias".to_owned(), vec![1]);
            if pointer.score == ReadScore::Lorentz {
                shapes.insert("pointer.log_beta".to_owned(), vec![1]);
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
    /// key, value and NoRead maps, or by the control's attention. With identity
    /// carry enabled this remains the current state for value/NoRead; query
    /// and key projections use its causal predecessor (zero at position zero).
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

/// Names of the pointer head's variables start with this.
const POINTER_PREFIX: &str = "pointer.";
/// The pointer gate's initial bias: `sigmoid(-2)`, about 0.12.
const POINTER_GATE_BIAS: f32 = -2.0;
/// Initial scale of the pointer query and key, times `1 / sqrt(width)`.
const POINTER_INITIAL_SCALE: f64 = 0.5;
/// Mixed into the seed of the pointer head's initializer.
const POINTER_SEED_MIX: u64 = 0x504F_494E_5445_5221;

/// The pointer head's variables for `config`, drawn from a stream seeded by
/// `seed` alone: query and key at `POINTER_INITIAL_SCALE / sqrt(width)`, the
/// gate weight zero, its bias `POINTER_GATE_BIAS` and (Lorentz) the log scale
/// `pointer.log_beta` zero, as `read.log_beta` starts. The scale draws nothing
/// from the stream, so a Dot and a Lorentz head of one seed share their query
/// and key. Empty without a pointer.
fn pointer_variables(
    config: &StackConfig,
    seed: u64,
    device: &Device,
) -> Result<BTreeMap<String, Var>> {
    let mut variables = BTreeMap::new();
    let mut rng = Initializer(seed ^ POINTER_SEED_MIX);
    let std = POINTER_INITIAL_SCALE / (config.width as f64).sqrt();
    for (name, shape) in config.shapes() {
        if !name.starts_with(POINTER_PREFIX) {
            continue;
        }
        let count: usize = shape.iter().product();
        let values: Vec<f32> = match name.as_str() {
            "pointer.gate.weight" | "pointer.log_beta" => vec![0.0; count],
            "pointer.gate.bias" => vec![POINTER_GATE_BIAS; count],
            _ => (0..count).map(|_| (rng.normal() * std) as f32).collect(),
        };
        variables.insert(name, Var::from_vec(values, shape.as_slice(), device)?);
    }
    Ok(variables)
}

fn pointer_served_refusal() -> crate::TrainingError {
    invalid(
        "the pointer head has no served representation yet; train it in float (no qat=true) and \
         it has no D11 port",
    )
}

fn remove_address_projection_parameters<T>(parameters: &mut BTreeMap<String, T>) {
    for suffix in [
        "read.query.weight",
        "read.key.weight",
        "read.query_identity.weight",
        "read.key_identity.weight",
        "read.log_beta",
        "read.offset",
    ] {
        parameters.remove(&layer_name(2, suffix));
    }
}

fn geometric_span_shapes(config: &StackConfig) -> BTreeMap<String, Vec<usize>> {
    [
        (
            layer_name(2, "read.span_control.weight"),
            vec![4, 2 * config.width],
        ),
        (layer_name(2, "read.span_control.bias"), vec![4]),
    ]
    .into_iter()
    .collect()
}

pub struct StackModel {
    pub config: StackConfig,
    variables: BTreeMap<String, Var>,
    device: Device,
    /// The served representation the forward pass reads, if set
    /// ([`Self::set_served_representation`]).
    served: Option<ServedState>,
    /// The roots the recurrences snap their transport to, if set
    /// ([`Self::set_transport_snap`]).
    transport: Option<TransportSnap>,
    /// The q/k projection input is the preceding normalized state, when set.
    read_identity_carry: bool,
    read_identity_latch: Option<ReadIdentityLatch>,
    /// Direct paired content/context geometry, with no q/k projection maps.
    geometric_address: Option<GeometricAddressConfig>,
    geometric_span: Option<GeometricSpanConfig>,
}

impl StackModel {
    pub fn new(config: StackConfig, device: &Device) -> Result<Self> {
        config.validate()?;
        let mut rng = Initializer(config.seed ^ 0x6765_6F6D_5354_4143);
        let residual_std = INITIAL_STD / (2.0 * config.layers() as f64).sqrt();
        let lanes = config.width / 4;
        let mut variables = BTreeMap::new();
        for (name, shape) in config.shapes() {
            // The pointer head draws from its own stream, so adding one to a
            // saved model gives the weights a fresh construction would.
            if name.starts_with(POINTER_PREFIX) {
                continue;
            }
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
        variables.extend(pointer_variables(&config, config.seed, device)?);
        Ok(Self {
            config,
            variables,
            device: device.clone(),
            served: None,
            transport: None,
            read_identity_carry: false,
            read_identity_latch: None,
            geometric_address: None,
            geometric_span: None,
        })
    }

    /// Give the model a pointer-copy head ([`PointerConfig`]) whose weights are
    /// initialised fresh from `seed` (the weights a new model with that
    /// configuration and seed would start with), and record `seed` in the
    /// head's [`PointerConfig::init_seed`] so that a resume can verify it.
    /// `Ok(false)` if the model already has a head of that width and score
    /// (its weights, selection and recorded seed are untouched; see
    /// [`Self::set_pointer_select`]); a head of another width or score is
    /// refused. The optimizer of a model must be built after this.
    pub fn add_pointer(&mut self, pointer: PointerConfig, seed: u64) -> Result<bool> {
        if self.geometric_address.is_some() {
            return Err(invalid(
                "geometric addressing does not support a pointer head",
            ));
        }
        if let Some(existing) = self.config.pointer {
            if existing.dim == pointer.dim && existing.score == pointer.score {
                return Ok(false);
            }
            return Err(invalid(format!(
                "the model has a {:?} pointer head of width {}, not a {:?} one of width {}",
                existing.score, existing.dim, pointer.score, pointer.dim
            )));
        }
        if self.served.is_some() {
            return Err(pointer_served_refusal());
        }
        let mut config = self.config.clone();
        config.pointer = Some(PointerConfig {
            init_seed: Some(seed),
            ..pointer
        });
        config.validate()?;
        self.variables
            .extend(pointer_variables(&config, seed, &self.device)?);
        self.config = config;
        Ok(true)
    }

    /// Let the model read `context` positions. Each read's age table
    /// (`read.age`, `[heads, context]`, indexed by age) keeps every learned
    /// entry, and each head continues past the saved context along its initial
    /// slope from its last learned value, so a sequence no longer than the
    /// saved context scores exactly as before. The transformer control has no
    /// age table (its RoPE tables are computed per call). `Ok(false)` if the
    /// context is unchanged; a smaller one is refused, as is a model with a
    /// served representation set. The optimizer of a model must be built
    /// after this.
    pub fn extend_context(&mut self, context: usize) -> Result<bool> {
        let saved = self.config.context;
        if context == saved {
            return Ok(false);
        }
        if context < saved {
            return Err(invalid(format!(
                "the model reads {saved} positions; a context of {context} would drop learned ages"
            )));
        }
        if self.served.is_some() {
            return Err(invalid(
                "extend the context before setting a served representation",
            ));
        }
        let mut config = self.config.clone();
        config.context = context;
        config.validate()?;
        let heads = config.heads;
        let mut extended = BTreeMap::new();
        for (name, var) in &self.variables {
            if !name.ends_with(".read.age") {
                continue;
            }
            let learned = var.as_tensor().flatten_all()?.to_vec1::<f32>()?;
            if learned.len() != heads * saved {
                return Err(invalid(format!("{name} is not [heads, context]")));
            }
            let mut values = Vec::with_capacity(heads * context);
            for (head, row) in learned.chunks(saved).enumerate() {
                values.extend_from_slice(row);
                // The initial slope of this head ("read.age" in `new`).
                let slope = 2f64.powf(-8.0 * (head + 1) as f64 / heads as f64);
                let last = f64::from(row[saved - 1]);
                values.extend(
                    (saved..context).map(|age| (last - slope * (age - (saved - 1)) as f64) as f32),
                );
            }
            let tensor = Tensor::from_vec(values, (heads, context), &self.device)?;
            extended.insert(name.clone(), Var::from_tensor(&tensor)?);
        }
        self.variables.extend(extended);
        self.config = config;
        Ok(true)
    }

    /// Replace the flock selection of the reads. The parameters do not change.
    pub fn set_select(&mut self, select: Option<FlockSelect>) -> Result<()> {
        if self.geometric_address.is_some() && select.is_some() {
            return Err(invalid("geometric addressing requires full causal support"));
        }
        if let Some(select) = &select {
            validate_flock(select)?;
        }
        self.config.select = select;
        Ok(())
    }

    /// Route the pointer's sources by exact prime arithmetic ([`PrimeRoute`])
    /// instead of its learned scores, or clear the route. The parameters do
    /// not change: the gate and the generated branch keep their weights, and
    /// clearing the route restores the learned scores. `Some` is refused on a
    /// model without a pointer head, or whose pointer has a selection (a
    /// route admits its own sources).
    pub fn set_pointer_route(&mut self, route: Option<PrimeRoute>) -> Result<()> {
        if let Some(route) = &route {
            route.validate()?;
        }
        match self.config.pointer.as_mut() {
            Some(pointer) if route.is_some() && pointer.select.is_some() => {
                return Err(invalid("a routed pointer has no selection; clear it first"))
            }
            Some(pointer) => pointer.route = route,
            None if route.is_some() => {
                return Err(invalid("the model has no pointer head to route"))
            }
            None => {}
        }
        Ok(())
    }

    /// Replace the pointer's own selection of its sources (the pointer never
    /// reads [`StackConfig::select`]). The parameters do not change, so this
    /// applies a selection to weights trained without one, and clearing it
    /// restores the soft pointer bit for bit. `Some` is refused on a model
    /// without a pointer head or with a routed one; `None` on one is a no-op.
    pub fn set_pointer_select(&mut self, select: Option<PointerSelect>) -> Result<()> {
        if let Some(select) = &select {
            select.validate()?;
        }
        match self.config.pointer.as_mut() {
            Some(pointer) if select.is_some() && pointer.route.is_some() => {
                return Err(invalid(
                    "a routed pointer has no selection; clear the route first",
                ))
            }
            Some(pointer) => pointer.select = select,
            None if select.is_some() => {
                return Err(invalid("the model has no pointer head to select for"))
            }
            None => {}
        }
        Ok(())
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
        if let Some(config) = &self.geometric_address {
            self.validate_geometric_address(config)?;
        }
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
        let read = fused_read_selected(
            &query,
            &key,
            &value,
            &aux,
            ReadScore::Dot,
            false,
            false,
            true,
            self.config.select,
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
        binding: &mut Option<BindingCapture<'_>>,
        gates: LatchGates,
    ) -> Result<Tensor> {
        self.geometric_read_with_source(p, layer, x, capture, binding, gates, ReadSource::default())
    }

    fn geometric_read_with_source(
        &self,
        p: &Params<'_>,
        layer: usize,
        x: &Tensor,
        capture: &mut Capture<'_>,
        binding: &mut Option<BindingCapture<'_>>,
        gates: LatchGates,
        source: ReadSource<'_>,
    ) -> Result<Tensor> {
        if let Some(config) = &self.geometric_address {
            return self
                .geometric_address_read(p, layer, x, capture, binding, gates, config, source);
        }
        let (batch, time, _) = x.dims3()?;
        let heads = self.config.heads;
        tap(capture, StackSite::Read(layer), || self.unit_norm(x))?;
        let u = self.norm(p, x, &layer_name(layer, "read_norm.weight"))?;
        let identity = self.read_identity_input(&u)?;
        let latched = match (self.read_identity_latch, gates) {
            (_, LatchGates::Projections(..)) => None,
            (Some(mode), _) => Some(match gates {
                LatchGates::Replay(identity) => identity.clone(),
                _ => self.read_latch_inputs(p, layer, &u, mode, gates)?.0,
            }),
            (None, _) => None,
        };
        let project = |part: &str| -> Result<Tensor> {
            if let LatchGates::Projections(query, key) = gates {
                match part {
                    "query" => return self.heads(query, batch, time),
                    "key" => return self.heads(key, batch, time),
                    _ => {}
                }
            }
            let input = if part == "value" { &u } else { &identity };
            let mut projected =
                Self::linear(input, p.layer(layer, &format!("read.{part}.weight"))?)?;
            if part != "value" {
                if let Some(identity) = &latched {
                    projected = projected.add(&Self::linear(
                        identity,
                        p.layer(layer, &format!("read.{part}_identity.weight"))?,
                    )?)?;
                }
            }
            self.heads(&projected, batch, time)
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
        if self.config.read.scaled() {
            aux.push(p.layer(layer, "read.log_beta")?.exp()?);
            aux.push(p.layer(layer, "read.offset")?.clone());
        }
        let aux = Tensor::cat(&aux, 0)?;
        if aux.dim(0)? != fused_aux_len(batch, heads, time, self.config.read, true, true) {
            return Err(invalid("fused read auxiliary layout differs"));
        }
        let (value, value_width) = self.read_binding_values(value, layer, binding)?;
        let read = fused_read_selected(
            &query,
            &key,
            &value,
            &aux,
            self.config.read,
            true,
            true,
            false,
            self.config.select,
        )?;
        self.finish_geometric_read(p, layer, &read, value_width, capture, binding)
    }

    /// The label mask is an auxiliary value channel, never a score input.
    fn read_binding_values(
        &self,
        value: Tensor,
        layer: usize,
        binding: &Option<BindingCapture<'_>>,
    ) -> Result<(Tensor, usize)> {
        let (batch, heads, time, _) = value.dims4()?;
        // One auxiliary value channel shares the exact scores, admission and
        // normalization of the ordinary read. It is removed before read.out,
        // so neither the teacher mask nor its mass enters the residual stream.
        let target = binding
            .as_ref()
            .filter(|binding| binding.target.layer == layer)
            .map(|binding| binding.target);
        let value_width = value.dim(3)?;
        let value = match target {
            None => value,
            Some(target) => {
                let mut mask = vec![0.0f32; batch * heads * time];
                for row in &target.rows {
                    for &source in &row.sources {
                        mask[(row.batch * heads + target.head) * time + source] = 1.0;
                    }
                }
                let mask = Tensor::from_vec(mask, (batch, heads, time, 1), &self.device)?;
                Tensor::cat(&[&value, &mask], 3)?
            }
        };
        Ok((value, value_width))
    }

    fn finish_geometric_read(
        &self,
        p: &Params<'_>,
        layer: usize,
        read: &Tensor,
        value_width: usize,
        capture: &mut Capture<'_>,
        binding: &mut Option<BindingCapture<'_>>,
    ) -> Result<Tensor> {
        let (batch, heads, time, _) = read.dims4()?;
        let target = binding
            .as_ref()
            .filter(|binding| binding.target.layer == layer)
            .map(|binding| binding.target);
        let read = if let Some(target) = target {
            let indices: Vec<u32> = target
                .rows
                .iter()
                .map(|row| ((row.batch * heads + target.head) * time + row.query) as u32)
                .collect();
            let indices = Tensor::from_vec(indices, target.rows.len(), &self.device)?;
            let masses = read
                .narrow(3, value_width, 1)?
                .flatten_all()?
                .index_select(&indices, 0)?;
            if let Some(binding) = binding {
                binding.masses = Some(masses);
            }
            read.narrow(3, 0, value_width)?
        } else {
            read.clone()
        };
        let merged = self.merge_heads(&read, batch, time)?;
        tap(capture, StackSite::ReadOut(layer), || Ok(merged.clone()))?;
        Self::linear(&merged, p.layer(layer, "read.out.weight")?)
    }

    fn read_identity_input(&self, normalized: &Tensor) -> Result<Tensor> {
        if !self.read_identity_carry {
            return Ok(normalized.clone());
        }
        let (batch, time, width) = normalized.dims3()?;
        let zero = Tensor::zeros((batch, 1, width), normalized.dtype(), normalized.device())?;
        if time == 1 {
            Ok(zero)
        } else {
            Ok(Tensor::cat(
                &[&zero, &normalized.narrow(1, 0, time - 1)?],
                1,
            )?)
        }
    }

    /// Opt into a causal predecessor-to-payload association at every geometric
    /// read: q/k at t project normalized input t-1 (zero at t=0), while values,
    /// NoRead and age remain at t. No labels or token parsing enter this rule.
    /// The full causal support, including self, is unchanged. Consequently a
    /// same-identity self candidate is still possible and must be learned away.
    /// No new parameters are introduced. The default is false.
    ///
    /// This initial float-training mechanism has no integer export or served
    /// representation; both must refuse it until the corresponding port exists.
    pub fn set_read_identity_carry(&mut self, enabled: bool) -> Result<()> {
        if enabled {
            if self.read_identity_latch.is_some() || self.geometric_span.is_some() {
                return Err(invalid(
                    "read identity carry and latch are mutually exclusive",
                ));
            }
            if self.config.arch != StackArch::Geometric || !self.config.pattern.contains('a') {
                return Err(invalid(
                    "read identity carry needs a geometric stack with a read layer",
                ));
            }
            if self.served.is_some() {
                return Err(invalid("read identity carry has no served representation"));
            }
        }
        self.read_identity_carry = enabled;
        Ok(())
    }

    /// Whether geometric q/k inputs use the preceding normalized state.
    pub fn read_identity_carry(&self) -> bool {
        self.read_identity_carry
    }

    /// Add learned identity inputs to every read. Original q/k maps continue
    /// to project current normalized input (role); new maps project h[t-1].
    /// g[t] = sigmoid(W [u[t], u[t-1]] + b). Held updates
    /// h[t] = (1-g[t]) h[t-1] + g[t] u[t]; Local uses h[t] = g[t] u[t].
    /// Initial u[-1] and h[-1] are zero. Values, NoRead, age and admission are
    /// unchanged. This is an offline float-training mechanism, not an expert
    /// gate or an integer serving implementation.
    ///
    /// Enable before constructing the optimizer: this adds parameters. The
    /// identity maps copy the current q/k weights, W starts zero and b at -2.
    /// Repeating the same mode is a no-op; changing modes is explicit research
    /// in another model, not a silent reinterpretation of saved parameters.
    pub fn set_read_identity_latch(&mut self, mode: ReadIdentityLatch) -> Result<()> {
        if self.geometric_span.is_some() {
            return Err(invalid(
                "geometric span production replaces the scalar identity latch",
            ));
        }
        if let Some(existing) = self.read_identity_latch {
            return if existing == mode {
                Ok(())
            } else {
                Err(invalid("read identity latch mode is already fixed"))
            };
        }
        if self.config.arch != StackArch::Geometric || !self.config.pattern.contains('a') {
            return Err(invalid(
                "read identity latch needs a geometric stack with a read layer",
            ));
        }
        if self.read_identity_carry || self.served.is_some() {
            return Err(invalid(
                "read identity latch is incompatible with carry and served mode",
            ));
        }
        let mut added = BTreeMap::new();
        for (name, shape) in read_identity_latch_shapes(&self.config) {
            let variable =
                if name.ends_with("query_identity.weight") || name.ends_with("key_identity.weight")
                {
                    let source = name.replace("_identity.weight", ".weight");
                    let source = self.variables.get(&source).ok_or_else(|| {
                        invalid("read identity latch lacks its initial projection")
                    })?;
                    Var::from_tensor(&source.as_tensor().copy()?)?
                } else if name.ends_with(".bias") {
                    Var::from_vec(vec![-2.0f32], shape.as_slice(), &self.device)?
                } else {
                    Var::from_tensor(&Tensor::zeros(shape.as_slice(), DType::F32, &self.device)?)?
                };
            added.insert(name, variable);
        }
        self.variables.extend(added);
        self.read_identity_latch = Some(mode);
        Ok(())
    }

    pub fn read_identity_latch(&self) -> Option<ReadIdentityLatch> {
        self.read_identity_latch
    }

    /// Replace the rra reader's four dense q/k maps by direct, trained-in
    /// geometric content/context addresses. Enable before building an optimizer.
    /// This removes the unused maps and Lorentz scalars from the parameter set;
    /// there is deliberately no disable operation that would invent them again.
    /// The value/output maps, full causal support, age and NoRead stay in use.
    /// This first implementation is an offline CPU float-training prototype.
    pub fn set_geometric_address(&mut self, config: GeometricAddressConfig) -> Result<()> {
        self.validate_geometric_address(&config)?;
        if let Some(existing) = &self.geometric_address {
            return if existing == &config {
                Ok(())
            } else {
                Err(invalid("geometric address mode is already fixed"))
            };
        }
        let mut rng = Initializer(self.config.seed ^ 0x6164_6472_6573_7331);
        let mut added = BTreeMap::new();
        for (suffix, shape) in
            geometric_address::parameter_shapes(self.config.width, self.config.heads)?
        {
            let count = shape.iter().product();
            let values: Vec<f32> = if suffix.ends_with("radius") || suffix.ends_with("presence") {
                vec![0.0; count]
            } else {
                (0..count)
                    .map(|_| (rng.normal() * INITIAL_STD) as f32)
                    .collect()
            };
            added.insert(
                layer_name(2, &suffix),
                Var::from_vec(values, shape.as_slice(), &self.device)?,
            );
        }
        remove_address_projection_parameters(&mut self.variables);
        self.variables.extend(added);
        self.geometric_address = Some(config);
        Ok(())
    }

    pub fn geometric_address(&self) -> Option<&GeometricAddressConfig> {
        self.geometric_address.as_ref()
    }

    /// Replace the geometric reader's scalar Held latch with an ordered token
    /// action producer. Enable after geometric addressing and before creating
    /// the optimizer. Static actions use original selected embedding rows;
    /// only the four-action controller sees contextual gained read inputs.
    /// OPEN/APPEND/COMMIT/HOLD are predicted during every forward. Their
    /// training labels never enter this API. Existing scalar-latch artifacts
    /// retain their original behavior and tensor inventory.
    pub fn set_geometric_span(&mut self, config: GeometricSpanConfig) -> Result<()> {
        config.validate(self.config.width)?;
        if let Some(existing) = &self.geometric_span {
            return if existing == &config {
                Ok(())
            } else {
                Err(invalid("geometric span mode is already fixed"))
            };
        }
        let address = self
            .geometric_address
            .as_ref()
            .ok_or_else(|| invalid("geometric spans require geometric addressing"))?;
        self.validate_geometric_address(address)?;
        let mut added = BTreeMap::new();
        for (name, shape) in geometric_span_shapes(&self.config) {
            added.insert(
                name,
                Var::from_tensor(&Tensor::zeros(shape.as_slice(), DType::F32, &self.device)?)?,
            );
        }
        for suffix in ["read.identity_gate.weight", "read.identity_gate.bias"] {
            self.variables.remove(&layer_name(2, suffix));
        }
        self.variables.extend(added);
        self.read_identity_latch = None;
        self.geometric_span = Some(config);
        Ok(())
    }

    pub fn geometric_span(&self) -> Option<&GeometricSpanConfig> {
        self.geometric_span.as_ref()
    }

    fn span_control_logits(&self, p: &Params<'_>, layer: usize, u: &Tensor) -> Result<Tensor> {
        let (batch, time, width) = u.dims3()?;
        let zero = Tensor::zeros((batch, 1, width), u.dtype(), u.device())?;
        let previous = if time == 1 {
            zero
        } else {
            Tensor::cat(&[&zero, &u.narrow(1, 0, time - 1)?], 1)?
        };
        let input = Tensor::cat(&[u, &previous], 2)?;
        Ok(
            Self::linear(&input, p.layer(layer, "read.span_control.weight")?)?
                .broadcast_add(p.layer(layer, "read.span_control.bias")?)?,
        )
    }

    /// Actual learned controller logits [batch,time,4], ordered as HOLD,
    /// OPEN, APPEND, COMMIT. A caller may apply a training-only cross entropy;
    /// the producer itself always follows its predicted earliest argmax.
    pub fn geometric_span_control_logits(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        if self.geometric_span.is_none() {
            return Err(invalid("geometric span production is disabled"));
        }
        let p = self.params()?;
        let x = self.embed_with(&p, ids, batch, time)?;
        let x = self.layer_range_bound(&p, x, 0..2, &mut None, &mut None, LatchGates::Soft)?;
        let u = self.norm(&p, &x, &layer_name(2, "read_norm.weight"))?;
        self.span_control_logits(&p, 2, &u)
    }

    /// Per-call last-token control: the same predicted FSM runs, but APPEND
    /// replaces working content rather than right-composing a token action.
    /// Parameters and saved Ordered semantics are unchanged.
    pub fn forward_geometric_span_last_token(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        if self.geometric_span.is_none() {
            return Err(invalid("geometric span production is disabled"));
        }
        let p = self.params()?;
        let tokens = self.embed_with(&p, ids, batch, time)?;
        let x = self.layer_range_with_source(
            &p,
            tokens.clone(),
            0..self.config.layers(),
            &mut None,
            &mut None,
            LatchGates::Soft,
            ReadSource {
                tokens: Some(&tokens),
                span_policy: SpanPolicy::LastToken,
                native_span: None,
                native_potential: None,
                event_control: None,
                context: None,
                native_reducer: None,
                native_trace: None,
                native_value_projection: None,
                learned_values: None,
                native_learned_values: None,
                no_read: NoReadSource::LegacyFloat,
                composition: None,
                integer_residual: None,
            },
        )?;
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Replay the same saved reader with exported static token actions and
    /// exact integer span-register updates. The controller and surrounding
    /// computation remain float; this is not a whole-model served mode.
    pub fn forward_geometric_span_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        compiled: &CompiledSpanActions,
    ) -> Result<Tensor> {
        let (hidden, _) =
            self.hidden_geometric_span_native(ids, batch, time, compiled, None, None)?;
        let p = self.params()?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Observe exact source probabilities from that same native-producer read.
    /// The binding labels select observations only, never controller actions.
    pub fn read_binding_masses_geometric_span_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        target: &ReadBindingTarget,
        compiled: &CompiledSpanActions,
    ) -> Result<Tensor> {
        self.hidden_geometric_span_native(ids, batch, time, compiled, Some(target), None)?
            .1
            .ok_or_else(|| invalid("native span source observer was not evaluated"))
    }

    /// Evaluate the existing reader with native token actions and compiled
    /// signed-relative lookup/add potentials. Input production, normalization,
    /// score reconstruction and surrounding computation remain floating point.
    pub fn forward_geometric_potential_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
    ) -> Result<Tensor> {
        let (hidden, _) =
            self.hidden_geometric_span_native(ids, batch, time, span, None, Some(potential))?;
        let p = self.params()?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Source labels observe the same native-potential reader; they do not
    /// generate events, geometric codes or candidate scores.
    pub fn read_binding_masses_geometric_potential_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        target: &ReadBindingTarget,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
    ) -> Result<Tensor> {
        self.hidden_geometric_span_native(ids, batch, time, span, Some(target), Some(potential))?
            .1
            .ok_or_else(|| invalid("native potential source observer was not evaluated"))
    }

    /// Actual learned potentials for source-bound offline compilation.
    pub fn geometric_address_potentials(&self) -> Result<AddressWeights> {
        let config = self
            .geometric_address
            .as_ref()
            .ok_or_else(|| invalid("geometric address absent"))?;
        self.validate_geometric_address(config)?;
        self.address_weights(&self.params()?, 2)
    }

    /// Reproduce the exact pre-read subgraph to observe both scorer inputs.
    /// No source/answer/action labels are accepted. The same normalization and
    /// predicted span controller feed the native reader above.
    pub fn geometric_potential_inputs(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        compiled: &CompiledSpanActions,
    ) -> Result<(Tensor, Tensor)> {
        let span = self
            .geometric_span
            .as_ref()
            .ok_or_else(|| invalid("geometric span absent"))?;
        let address = self
            .geometric_address
            .as_ref()
            .ok_or_else(|| invalid("geometric address absent"))?;
        self.validate_geometric_address(address)?;
        let p = self.params()?;
        compiled.validate_for(p.get("embedding.weight")?, span)?;
        let tokens = self.embed_with(&p, ids, batch, time)?;
        let x = self.layer_range_bound(&p, tokens, 0..2, &mut None, &mut None, LatchGates::Soft)?;
        let current = self.norm(&p, &x, &layer_name(2, "read_norm.weight"))?;
        let events = self.span_control_logits(&p, 2, &current)?;
        let prior = geometric_span_native::produce_native(ids, batch, time, &events, compiled)?;
        Ok((current, prior))
    }

    fn hidden_geometric_span_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        compiled: &CompiledSpanActions,
        target: Option<&ReadBindingTarget>,
        potential: Option<&CompiledGeometricPotentials>,
    ) -> Result<(Tensor, Option<Tensor>)> {
        let span = self
            .geometric_span
            .as_ref()
            .ok_or_else(|| invalid("native span replay requires geometric spans"))?;
        let address = self
            .geometric_address
            .as_ref()
            .ok_or_else(|| invalid("native span replay requires geometric addressing"))?;
        self.validate_geometric_address(address)?;
        let p = self.params()?;
        compiled.validate_for(p.get("embedding.weight")?, span)?;
        if let Some(target) = target {
            self.validate_binding(batch, time, target)?;
        }
        let tokens = self.embed_with(&p, ids, batch, time)?;
        let mut binding = target.map(|target| BindingCapture {
            target,
            masses: None,
        });
        let x = self.layer_range_with_source(
            &p,
            tokens.clone(),
            0..self.config.layers(),
            &mut None,
            &mut binding,
            LatchGates::Soft,
            ReadSource {
                tokens: Some(&tokens),
                span_policy: SpanPolicy::Ordered,
                native_span: Some((compiled, ids)),
                native_potential: potential,
                event_control: None,
                context: None,
                native_reducer: None,
                native_trace: None,
                native_value_projection: None,
                learned_values: None,
                native_learned_values: None,
                no_read: NoReadSource::LegacyFloat,
                composition: None,
                integer_residual: None,
            },
        )?;
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        Ok((hidden, binding.and_then(|capture| capture.masses)))
    }

    /// Train the geometric event factors through this same reader. Events are
    /// predicted from observed IDs and context, never supplied labels. The base
    /// stack and answer path can remain frozen while these factors receive credit.
    pub fn forward_geometric_event(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        events: &EventWeights,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        self.forward_geometric_event_training(
            ids,
            batch,
            time,
            events,
            reset_each_token,
            AgeTrainingMode::None,
        )
    }

    /// Explicit joint event/age training bridge. Hard events come from fresh
    /// strict native tables; age uses a fresh quarter-grid view with identity
    /// STE inside the existing common occurrence/NoRead normalization. The
    /// remaining span/address/trunk/head path retains its declared float model.
    pub fn forward_geometric_event_q4_age(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        events: &EventWeights,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        if !events.is_q4() {
            return Err(invalid(
                "joint event/age training requires strict q4 events",
            ));
        }
        self.forward_geometric_event_training(
            ids,
            batch,
            time,
            events,
            reset_each_token,
            AgeTrainingMode::Quarter,
        )
    }

    /// Explicit joint event/age bridge with the fixed initialization age prior
    /// plus a learned signed-q4 residual at one-eighth nat. This keeps the live
    /// absolute age shadow and an identity STE in the same occurrence/NoRead
    /// denominator. It does not change the quarter-grid entry point or make
    /// the remaining floating span/address/trunk/head path native.
    pub fn forward_geometric_event_q4_age_residual(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        events: &EventWeights,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        if !events.is_q4() {
            return Err(invalid(
                "joint event/age residual training requires strict q4 events",
            ));
        }
        self.forward_geometric_event_training(
            ids,
            batch,
            time,
            events,
            reset_each_token,
            AgeTrainingMode::PriorResidual,
        )
    }

    fn forward_geometric_event_training(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        events: &EventWeights,
        reset_each_token: bool,
        age: AgeTrainingMode,
    ) -> Result<Tensor> {
        if events.config().vocab_size != self.config.vocab_size {
            return Err(invalid("event vocabulary differs from model"));
        }
        let (hidden, _) = if events.is_q4() {
            let output = events.forward_q4(ids, batch, time, reset_each_token)?;
            self.hidden_geometric_event(
                ids,
                batch,
                time,
                SpanEvents::TrainingNative {
                    logits: &output.logits,
                    actions: &output.trace.actions,
                    age,
                },
                None,
                None,
                None,
            )?
        } else {
            let logits = if reset_each_token {
                events.event_logits_reset_each_token(ids, batch, time)?
            } else {
                events.event_logits(ids, batch, time)?
            };
            self.hidden_geometric_event(
                ids,
                batch,
                time,
                SpanEvents::Training(&logits),
                None,
                None,
                None,
            )?
        };
        let p = self.params()?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Execute the native integer event controller and exact span register.
    /// No float controller or reconstructed-logit argmax supplies these events.
    /// The surrounding reader/output remains an offline floating-point boundary.
    pub fn forward_geometric_event_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        if events.config().vocab_size != self.config.vocab_size {
            return Err(invalid("compiled event vocabulary differs from model"));
        }
        let trace = geometric_event::trace_native(ids, batch, time, events, reset_each_token)?;
        let (hidden, _) = self.hidden_geometric_event(
            ids,
            batch,
            time,
            SpanEvents::Native(&trace.actions),
            None,
            Some(span),
            Some(potential),
        )?;
        let p = self.params()?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    pub fn read_binding_masses_geometric_event_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        target: &ReadBindingTarget,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        if events.config().vocab_size != self.config.vocab_size {
            return Err(invalid("compiled event vocabulary differs from model"));
        }
        let trace = geometric_event::trace_native(ids, batch, time, events, reset_each_token)?;
        self.hidden_geometric_event(
            ids,
            batch,
            time,
            SpanEvents::Native(&trace.actions),
            Some(target),
            Some(span),
            Some(potential),
        )?
        .1
        .ok_or_else(|| invalid("native event source observer absent"))
    }

    fn validate_context_config(
        &self,
        config: &geometric_context::GeometricContextConfig,
    ) -> Result<()> {
        if config.vocab_size != self.config.vocab_size
            || config.width != self.config.width
            || config.heads != self.config.heads
        {
            return Err(invalid(
                "context vocabulary/width/head partition differs from stack",
            ));
        }
        let address = self
            .geometric_address
            .as_ref()
            .ok_or_else(|| invalid("context requires geometric address configuration"))?;
        if config.lanes_per_head != address.lanes_per_head || config.heads != address.heads {
            return Err(invalid(
                "context lane partition differs from address configuration",
            ));
        }
        self.validate_geometric_address(address)
    }

    fn validate_context_dependencies(
        &self,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
    ) -> Result<()> {
        self.validate_context_config(context.config())?;
        // Canonical metadata belongs to these immutable admitted objects and
        // binds their payload hashes. This is boundary admission, not hot-loop
        // numerical work or authentication of the artifact producer.
        for (key, bytes) in [
            (
                "event_native/metadata.json",
                serde_json::to_vec_pretty(events.metadata())?,
            ),
            (
                "span_native/metadata.json",
                serde_json::to_vec_pretty(span.metadata())?,
            ),
            (
                "potential_native/metadata.json",
                serde_json::to_vec_pretty(potential.metadata())?,
            ),
        ] {
            let expected = context
                .metadata()
                .source
                .frozen
                .files
                .get(key)
                .ok_or_else(|| invalid("context frozen metadata binding absent"))?;
            if bytes.len() != expected.bytes || crate::sha256_bytes(&bytes) != expected.sha256 {
                return Err(invalid(
                    "context frozen event/span/potential object differs",
                ));
            }
        }
        Ok(())
    }

    /// Offline donor observation before contextual addressing. No old span
    /// controller, source label or answer label supplies the contextual target.
    pub fn geometric_context_teacher(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        let address = self
            .geometric_address
            .as_ref()
            .ok_or_else(|| invalid("context teacher requires geometric addressing"))?;
        self.validate_geometric_address(address)?;
        let p = self.params()?;
        let tokens = self.embed_with(&p, ids, batch, time)?;
        let x = self.layer_range_bound(&p, tokens, 0..2, &mut None, &mut None, LatchGates::Soft)?;
        self.norm(&p, &x, &layer_name(2, "read_norm.weight"))
    }

    /// New geometric context gets ordinary answer credit through the frozen
    /// floating reference scorer. Frozen native events supply capture; this
    /// accepts no teacher addresses or events as forward inputs.
    pub fn forward_geometric_context(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &ContextWeights,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        self.validate_context_config(context.config())?;
        let output = context.forward(ids, batch, time, reset_each_token)?;
        let event = geometric_event::trace_native(ids, batch, time, events, false)?;
        let (hidden, _) = self.hidden_geometric_context(
            ids,
            batch,
            time,
            SpanEvents::Native(&event.actions),
            None,
            Some(span),
            None,
            Some(ContextInput::Training(&output.context)),
        )?;
        let p = self.params()?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Offline joint answer-credit path with actual geometric retained-state
    /// values. The donor value map is bypassed; NoRead/trunk/output remain float.
    /// This is not yet an integer compiled producer or a serving qualification.
    pub fn forward_geometric_context_learned_values(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &ContextWeights,
        producer: &crate::geometric_value_producer::ValueProducerWeights,
        events: &CompiledEvents,
        compiled: &CompiledSpanActions,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        self.forward_geometric_context_learned_values_mode(
            ids,
            batch,
            time,
            context,
            producer,
            events,
            compiled,
            reset_each_token,
            false,
        )
    }

    fn forward_geometric_context_learned_values_mode(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &ContextWeights,
        producer: &crate::geometric_value_producer::ValueProducerWeights,
        events: &CompiledEvents,
        compiled: &CompiledSpanActions,
        reset_each_token: bool,
        stop_address_credit: bool,
    ) -> Result<Tensor> {
        self.validate_context_config(context.config())?;
        if self.config.width / self.config.heads != crate::geometric_value_producer::VALUE_WIDTH
            || producer.config().vocab_size != self.config.vocab_size
            || producer.config().heads != self.config.heads
            || producer.config().latent_lanes_per_head != context.config().lanes_per_head
        {
            return Err(invalid(
                "learned geometric value configuration differs from reader/context",
            ));
        }

        let output = context.forward(ids, batch, time, reset_each_token)?;
        let addressing = if stop_address_credit {
            output.context.detach()
        } else {
            output.context.clone()
        };
        let event = geometric_event::trace_native(ids, batch, time, events, false)?;
        let span = self
            .geometric_span
            .as_ref()
            .ok_or_else(|| invalid("learned values require spans"))?;
        let address = self
            .geometric_address
            .as_ref()
            .ok_or_else(|| invalid("learned values require addresses"))?;
        self.validate_geometric_address(address)?;
        let p = self.params()?;
        compiled.validate_for(p.get("embedding.weight")?, span)?;
        let tokens = self.embed_with(&p, ids, batch, time)?;
        let x = self.layer_range_with_source(
            &p,
            tokens.clone(),
            0..self.config.layers(),
            &mut None,
            &mut None,
            LatchGates::Soft,
            ReadSource {
                tokens: Some(&tokens),
                native_span: Some((compiled, ids)),
                event_control: Some(SpanEvents::Native(&event.actions)),
                context: Some(ContextInput::Training(&addressing)),
                learned_values: Some(LearnedGeometricValues {
                    producer,
                    latent: &output.latent_roots,
                    ids,
                }),
                ..ReadSource::default()
            },
        )?;
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    pub fn read_binding_masses_geometric_context(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        target: &ReadBindingTarget,
        context: &ContextWeights,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        self.validate_context_config(context.config())?;
        let output = context.forward(ids, batch, time, reset_each_token)?;
        let event = geometric_event::trace_native(ids, batch, time, events, false)?;
        self.hidden_geometric_context(
            ids,
            batch,
            time,
            SpanEvents::Native(&event.actions),
            Some(target),
            Some(span),
            None,
            Some(ContextInput::Training(&output.context)),
        )?
        .1
        .ok_or_else(|| invalid("geometric context source observer absent"))
    }

    /// Integer context, event, span and geometric score components. No donor
    /// tensor supplies addressing; the surrounding values/output stay float.
    pub fn forward_geometric_context_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        self.validate_context_dependencies(context, events, span, potential)?;
        let ctx = geometric_context::trace_native(ids, batch, time, context, reset_each_token)?;
        let event = geometric_event::trace_native(ids, batch, time, events, false)?;
        let (hidden, _) = self.hidden_geometric_context(
            ids,
            batch,
            time,
            SpanEvents::Native(&event.actions),
            None,
            Some(span),
            Some(potential),
            Some(ContextInput::Native(&ctx.codes)),
        )?;
        let p = self.params()?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Native geometric scores, age, normalization and weighted payload
    /// reduction. Float donor payload/NoRead production and the rest of the
    /// model remain outside this numerical bridge.
    pub fn forward_geometric_context_read_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        let (hidden, _) = self.hidden_geometric_context_read_native(
            ids,
            batch,
            time,
            None,
            context,
            events,
            span,
            potential,
            reducer,
            reset_each_token,
            None,
            None,
        )?;
        let p = self.params()?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Capture the raw native reduction trace from the SAME predictive pass.
    /// This offline diagnostic accepts no source labels and does not persist
    /// an override or modify weights. No second scorer is evaluated.
    pub fn forward_geometric_context_read_native_with_trace(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        reset_each_token: bool,
    ) -> Result<(Tensor, crate::geometric_read_native::NativeReadTrace)> {
        let trace = std::cell::RefCell::new(None);
        let (hidden, _) = self.hidden_geometric_context_read_native(
            ids,
            batch,
            time,
            None,
            context,
            events,
            span,
            potential,
            reducer,
            reset_each_token,
            Some(&trace),
            None,
        )?;
        let p = self.params()?;
        let logits = hidden.matmul(&p.head()?.t()?)?;
        let trace = trace
            .into_inner()
            .ok_or_else(|| invalid("native weighted read trace was not evaluated"))?;
        Ok((logits, trace))
    }

    /// Oracle representation comparison only. Packet selection sees actual
    /// donor values and reconstruction error, never source or answer labels.
    /// This diagnoses alphabet loss; it is not a learned native value producer.
    pub fn forward_geometric_context_value_projection_native_with_trace(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        values: &crate::geometric_value_native::CompiledGeometricValues,
        reset_each_token: bool,
    ) -> Result<(
        Tensor,
        crate::geometric_read_native::NativeReadTrace,
        crate::geometric_value_native::ValueProjectionTrace,
    )> {
        self.forward_geometric_context_value_projection_mode_with_trace(
            ids,
            batch,
            time,
            context,
            events,
            span,
            potential,
            reducer,
            values,
            reset_each_token,
            false,
        )
    }

    /// Same frozen-weight oracle comparison, with one bounded residual atom.
    pub fn forward_geometric_context_value_residual_native_with_trace(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        values: &crate::geometric_value_native::CompiledGeometricValues,
        reset_each_token: bool,
    ) -> Result<(
        Tensor,
        crate::geometric_read_native::NativeReadTrace,
        crate::geometric_value_native::ValueProjectionTrace,
    )> {
        self.forward_geometric_context_value_projection_mode_with_trace(
            ids,
            batch,
            time,
            context,
            events,
            span,
            potential,
            reducer,
            values,
            reset_each_token,
            true,
        )
    }

    /// Loaded integer geometric value production from actual retained states
    /// and native span registers. No donor value tensor enters this read.
    /// NoRead, trunk and output are still the floating saved reference.
    /// Preserved legacy null producer; payload/context are native components.
    pub fn forward_geometric_context_learned_values_native_with_trace(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        reset_each_token: bool,
    ) -> Result<(
        Tensor,
        crate::geometric_read_native::NativeReadTrace,
        crate::geometric_value_producer::ValueProducerTrace,
    )> {
        let (logits, trace, values, _) = self.forward_geometric_no_read_mode(
            ids,
            batch,
            time,
            context,
            events,
            span,
            potential,
            reducer,
            producer,
            reset_each_token,
            NoReadMode::Legacy,
        )?;
        Ok((
            logits,
            trace.ok_or_else(|| invalid("native learned read trace absent"))?,
            values,
        ))
    }

    /// Offline answer-gradient bridge: native frozen features/scores/payloads,
    /// hard-q4 learned NoRead and differentiable real softmax. No integer backward.
    pub fn forward_geometric_no_read(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        no_read: &crate::geometric_no_read::NoReadWeights,
        reset_each_token: bool,
    ) -> Result<(Tensor, Tensor)> {
        let (logits, _, _, null) = self.forward_geometric_no_read_mode(
            ids,
            batch,
            time,
            context,
            events,
            span,
            potential,
            reducer,
            producer,
            reset_each_token,
            NoReadMode::Learned(no_read),
        )?;
        Ok((
            logits,
            null.ok_or_else(|| invalid("learned NoRead tensor absent"))?,
        ))
    }

    /// Loaded native q4 null, with actual raw Q24 score and Q16 mixture trace.
    pub fn forward_geometric_no_read_native_with_trace(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        no_read: &crate::geometric_no_read_native::CompiledNoRead,
        reset_each_token: bool,
    ) -> Result<(
        Tensor,
        crate::geometric_read_native::NativeReadTrace,
        crate::geometric_value_producer::ValueProducerTrace,
    )> {
        let (logits, trace, values, _) = self.forward_geometric_no_read_mode(
            ids,
            batch,
            time,
            context,
            events,
            span,
            potential,
            reducer,
            producer,
            reset_each_token,
            NoReadMode::Native(no_read),
        )?;
        Ok((
            logits,
            trace.ok_or_else(|| invalid("native NoRead trace absent"))?,
            values,
        ))
    }

    fn forward_geometric_no_read_mode(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        reset_each_token: bool,
        no_read: NoReadMode<'_>,
    ) -> Result<(
        Tensor,
        Option<crate::geometric_read_native::NativeReadTrace>,
        crate::geometric_value_producer::ValueProducerTrace,
        Option<Tensor>,
    )> {
        let (logits, read, values, null, _) = self.forward_geometric_composition_mode(
            ids,
            batch,
            time,
            context,
            events,
            span,
            potential,
            reducer,
            producer,
            reset_each_token,
            no_read,
            None,
        )?;
        Ok((logits, read, values, null))
    }

    /// Hard geometric composition with conditional finite-choice answer credit.
    /// Frozen native scores/null and packets, real softmax training reduction;
    /// the donor read.out is bypassed. The residual/trunk/output tail remains float.
    pub fn forward_geometric_composition(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        no_read: &crate::geometric_no_read_native::CompiledNoRead,
        composition: &crate::geometric_composition::CompositionWeights,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        Ok(self
            .forward_geometric_composition_mode(
                ids,
                batch,
                time,
                context,
                events,
                span,
                potential,
                reducer,
                producer,
                reset_each_token,
                NoReadMode::Native(no_read),
                Some(CompositionMode::Learned(composition)),
            )?
            .0)
    }

    /// Run the admitted persistent integer reader once per occurrence and feed
    /// its checked head sum into the existing residual site. This CPU bridge
    /// retains the floating recurrence, MLP, final norm and vocabulary head;
    /// neither this method nor its returned logits qualify integer model serving.
    /// Each batch item has its own fresh session and exact untruncated history.
    pub fn forward_geometric_attention_session(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        admitted: &crate::geometric_attention_native::CompiledGeometricAttention<'_>,
    ) -> Result<Tensor> {
        admitted.validate_stack(self)?;
        let count = batch
            .checked_mul(time)
            .ok_or_else(|| invalid("incremental attention input shape overflow"))?;
        if !self.device.is_cpu()
            || batch == 0
            || time == 0
            || time > self.config.context
            || ids.len() != count
        {
            return Err(invalid(
                "incremental attention needs CPU batch*time ids within context",
            ));
        }
        let mut residual = Vec::with_capacity(
            count
                .checked_mul(32)
                .ok_or_else(|| invalid("incremental attention residual shape overflow"))?,
        );
        let mut session = admitted.session(time)?;
        for window in ids.chunks_exact(time) {
            session.reset();
            for &token in window {
                let step = session
                    .push(token as usize)
                    .map_err(|e| invalid(e.to_string()))?;
                residual.extend(
                    step.output_q16
                        .iter()
                        .map(|&value| (value as f64 / 65536.) as f32),
                );
            }
        }
        let residual = Tensor::from_vec(residual, (batch, time, 32), &self.device)?;
        let p = self.params()?;
        let tokens = self.embed_with(&p, ids, batch, time)?;
        let x = self.layer_range_with_source(
            &p,
            tokens,
            0..self.config.layers(),
            &mut None,
            &mut None,
            LatchGates::Soft,
            ReadSource {
                integer_residual: Some((2, &residual)),
                ..ReadSource::default()
            },
        )?;
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Offline event/age input credit through the SAME native retained reader.
    /// Hard numerical output is recomputed from current native events and age;
    /// the declared real-softmax bridge changes only backward. All host and
    /// consumer weights are detached. Routes are value/NoRead/potential/age.
    #[allow(clippy::too_many_arguments)]
    pub fn forward_geometric_event_age_native_credit(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        no_read: &crate::geometric_no_read_native::CompiledNoRead,
        composition: &crate::geometric_composition_native::CompiledComposition,
        event_weights: &EventWeights,
        age_shadow: &Tensor,
        value_weights: &crate::geometric_value_producer::ValueProducerWeights,
        no_read_weights: &crate::geometric_no_read::NoReadWeights,
        potential_weights: &crate::geometric_potential_q4::PotentialQ4Weights,
        composition_weights: &crate::geometric_composition::CompositionWeights,
        credit_routes: [bool; 4],
    ) -> Result<(Tensor, crate::geometric_event_credit::EventAgeCreditOutput)> {
        let admitted = crate::geometric_attention_native::CompiledGeometricAttention::new(
            context,
            events,
            span,
            potential,
            reducer,
            producer,
            no_read,
            composition,
        )?;
        admitted.validate_stack(self)?;
        if batch == 0
            || batch > 32
            || time == 0
            || time > self.config.context
            || time > 128
            || ids.len() != batch * time
            || self.served.is_some()
        {
            return Err(invalid(
                "held credit requires bounded CPU native rra parent",
            ));
        }
        events.validate_q4_training(event_weights)?;
        if !reducer.is_q4_age_residual() {
            return Err(invalid(
                "held credit requires explicit prior/eighth-residual age policy",
            ));
        }
        reducer.validate_q4_training(age_shadow, potential)?;
        producer.validate_q4_frozen_source(value_weights)?;
        no_read.validate_for(no_read_weights)?;
        potential.validate_q4_source(potential_weights)?;
        composition.validate_for(composition_weights)?;
        let p = Params::Frozen(
            self.variables
                .iter()
                .map(|(name, var)| (name.clone(), var.as_tensor().detach()))
                .collect(),
        );
        let tokens = self.embed_with(&p, ids, batch, time)?;
        let config = self
            .geometric_span
            .as_ref()
            .ok_or_else(|| invalid("held credit requires saved span configuration"))?;
        span.validate_for(p.get("embedding.weight")?, config)?;
        let event = event_weights.forward_q4(ids, batch, time, false)?;
        let held_trace = geometric_span_native::trace_native_events(
            ids,
            batch,
            time,
            &event.trace.actions,
            span,
        )?;
        let held = geometric_span::produce_with_native_actions(
            &tokens,
            &event.logits,
            config,
            SpanPolicy::Ordered,
            &event.trace.actions,
        )?
        .reshape((
            batch,
            time,
            context.config().heads,
            context.config().lanes_per_head,
            4,
        ))?;
        let ctx = geometric_context::trace_native(ids, batch, time, context, false)?;
        let (residual, output) = crate::geometric_event_credit::forward(
            ids,
            ctx,
            event,
            held_trace,
            &held,
            age_shadow,
            value_weights,
            no_read_weights,
            potential_weights,
            composition_weights,
            composition,
            reducer,
            credit_routes,
        )?;
        let x = self.layer_range_with_source(
            &p,
            tokens,
            0..self.config.layers(),
            &mut None,
            &mut None,
            LatchGates::Soft,
            ReadSource {
                integer_residual: Some((2, &residual)),
                ..ReadSource::default()
            },
        )?;
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        Ok((hidden.matmul(&p.head()?.t()?)?, output))
    }

    /// Native geometric composition and wide integer per-head weighted read.
    /// Exact checked i64 head addition precedes the unfinished float model tail.
    pub fn forward_geometric_composition_native_with_trace(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        no_read: &crate::geometric_no_read_native::CompiledNoRead,
        composition: &crate::geometric_composition_native::CompiledComposition,
        reset_each_token: bool,
    ) -> Result<(
        Tensor,
        crate::geometric_composition_native::ComposedReadTrace,
        crate::geometric_value_producer::ValueProducerTrace,
    )> {
        let (logits, _, values, _, trace) = self.forward_geometric_composition_mode(
            ids,
            batch,
            time,
            context,
            events,
            span,
            potential,
            reducer,
            producer,
            reset_each_token,
            NoReadMode::Native(no_read),
            Some(CompositionMode::Native(composition)),
        )?;
        Ok((
            logits,
            trace.ok_or_else(|| invalid("native composition trace missing"))?,
            values,
        ))
    }

    /// Offline q4 producer answer credit through a frozen geometric bank.
    /// Admitted compiled producer supplies dependency/config identity; live shadow
    /// values may change during training. Recompile/rebind before native inference.
    /// Native packet choices use current live packed coefficients; only backward
    /// probabilities and real-softmax training reduction remain floating.
    #[allow(clippy::too_many_arguments)]
    pub fn forward_geometric_q4_values(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        compiled_producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        no_read: &crate::geometric_no_read_native::CompiledNoRead,
        producer: &crate::geometric_value_producer::ValueProducerWeights,
        composition: &crate::geometric_composition::CompositionWeights,
        reset_each_token: bool,
    ) -> Result<(Tensor, crate::geometric_value_producer::ValueProducerOutput)> {
        let output = std::cell::RefCell::new(None);
        let logits = self
            .forward_geometric_composition_mode(
                ids,
                batch,
                time,
                context,
                events,
                span,
                potential,
                reducer,
                compiled_producer,
                reset_each_token,
                NoReadMode::Native(no_read),
                Some(CompositionMode::FrozenValues {
                    producer,
                    composition,
                    output: &output,
                }),
            )?
            .0;
        Ok((
            logits,
            output
                .into_inner()
                .ok_or_else(|| invalid("q4 value source output missing"))?,
        ))
    }

    /// Current-packed q4 score credit through the actual denominator and frozen
    /// native q4 values/H4 composition. Source reduction is real softmax/F32;
    /// native inference remains the independently admitted integer reducer.
    #[allow(clippy::too_many_arguments)]
    pub fn forward_geometric_q4_potential(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        no_read: &crate::geometric_no_read_native::CompiledNoRead,
        composition: &crate::geometric_composition_native::CompiledComposition,
        weights: &crate::geometric_potential_q4::PotentialQ4Weights,
        reset_each_token: bool,
    ) -> Result<(
        Tensor,
        crate::geometric_potential_q4::PotentialQ4Output,
        crate::geometric_value_producer::ValueProducerTrace,
    )> {
        let output = std::cell::RefCell::new(None);
        let (logits, _, values, _, _) = self.forward_geometric_composition_mode(
            ids,
            batch,
            time,
            context,
            events,
            span,
            potential,
            reducer,
            producer,
            reset_each_token,
            NoReadMode::Native(no_read),
            Some(CompositionMode::LearnedPotential {
                weights,
                composition,
                output: &output,
            }),
        )?;
        Ok((
            logits,
            output
                .into_inner()
                .ok_or_else(|| invalid("q4 potential source output missing"))?,
            values,
        ))
    }

    /// Offline answer credit through the current native-selected context state,
    /// frozen strict potential, values/bank and NoRead. The float trunk/reducer
    /// remains an explicitly unfinished serving boundary.
    #[allow(clippy::too_many_arguments)]
    pub fn forward_geometric_q4_context(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        no_read: &crate::geometric_no_read_native::CompiledNoRead,
        composition: &crate::geometric_composition_native::CompiledComposition,
        context_weights: &geometric_context::ContextWeights,
        value_weights: &crate::geometric_value_producer::ValueProducerWeights,
        no_read_weights: &crate::geometric_no_read::NoReadWeights,
        potential_weights: &crate::geometric_potential_q4::PotentialQ4Weights,
        composition_weights: &crate::geometric_composition::CompositionWeights,
        reset_each_token: bool,
    ) -> Result<(
        Tensor,
        geometric_context::ContextQ4Output,
        crate::geometric_potential_q4::PotentialQ4Output,
        crate::geometric_value_producer::ValueProducerOutput,
        crate::geometric_context_credit::NoReadCreditOutput,
    )> {
        self.forward_geometric_q4_context_routes(
            ids,
            batch,
            time,
            context,
            events,
            span,
            potential,
            reducer,
            producer,
            no_read,
            composition,
            context_weights,
            value_weights,
            no_read_weights,
            potential_weights,
            composition_weights,
            reset_each_token,
            [true; 3],
        )
    }

    /// Diagnostic graph cuts in value/NoRead/potential order. All hard choices
    /// remain unchanged; serving has no route-switch interface.
    #[allow(clippy::too_many_arguments)]
    pub fn forward_geometric_q4_context_routes(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        no_read: &crate::geometric_no_read_native::CompiledNoRead,
        composition: &crate::geometric_composition_native::CompiledComposition,
        context_weights: &geometric_context::ContextWeights,
        value_weights: &crate::geometric_value_producer::ValueProducerWeights,
        no_read_weights: &crate::geometric_no_read::NoReadWeights,
        potential_weights: &crate::geometric_potential_q4::PotentialQ4Weights,
        composition_weights: &crate::geometric_composition::CompositionWeights,
        reset_each_token: bool,
        credit_routes: [bool; 3],
    ) -> Result<(
        Tensor,
        geometric_context::ContextQ4Output,
        crate::geometric_potential_q4::PotentialQ4Output,
        crate::geometric_value_producer::ValueProducerOutput,
        crate::geometric_context_credit::NoReadCreditOutput,
    )> {
        self.validate_context_config(context.config())?;
        self.validate_context_dependencies(context, events, span, potential)?;
        context.validate_q4_training(context_weights)?;
        producer.validate_native_context(context)?;
        producer.validate_q4_frozen_source(value_weights)?;
        no_read.validate_for(no_read_weights)?;
        no_read.validate_dependencies(context, potential, reducer, producer, events, span)?;
        potential.validate_q4_source(potential_weights)?;
        composition.validate_for(composition_weights)?;
        composition
            .validate_dependencies(context, potential, reducer, producer, events, span, no_read)?;
        if self.config.width != 32
            || self.config.heads != 2
            || time == 0
            || time > 128
            || reducer.metadata().layer != 2
        {
            return Err(invalid(
                "context credit requires admitted H2/width32/context<=128 reader",
            ));
        }
        let ctx = context_weights.forward_q4(ids, batch, time, reset_each_token)?;
        let event = geometric_event::trace_native(ids, batch, time, events, false)?;
        let held =
            geometric_span_native::trace_native_events(ids, batch, time, &event.actions, span)?;
        let lanes = self.config.heads * context.config().lanes_per_head;
        let count = ids
            .len()
            .checked_mul(lanes)
            .ok_or_else(|| invalid("context credit held-code overflow"))?;
        if held.prior_codes.len() != ids.len() || held.lanes != lanes {
            return Err(invalid("context credit span layout differs"));
        }
        let mut held_codes = Vec::with_capacity(count);
        let mut content = Vec::with_capacity(count);
        let mut span_valid = Vec::with_capacity(ids.len());
        for prior in &held.prior_codes {
            span_valid.push(prior.is_some());
            if prior.as_ref().is_some_and(|codes| codes.len() != lanes) {
                return Err(invalid("context credit held span lane layout differs"));
            }
            for lane in 0..lanes {
                let code = prior.as_ref().map(|codes| codes[lane]).unwrap_or(1);
                held_codes.push(
                    uor_r4_integer::h4_tables::H4Code::try_from(code)
                        .map_err(|e| invalid(e.to_string()))?,
                );
                content.push(
                    uor_r4_integer::geometric_potential::AddressLane::new(
                        code,
                        if prior.is_some() { 16 } else { 0 },
                        prior.is_some(),
                    )
                    .map_err(|e| invalid(e.to_string()))?,
                );
            }
        }
        let valid = vec![true; ids.len()];
        let mut values = crate::geometric_context_credit::frozen_value_forward(
            value_weights,
            ids,
            &ctx,
            &held_codes,
            &span_valid,
            &valid,
        )?;
        let mut null = crate::geometric_context_credit::frozen_no_read_forward(
            no_read_weights,
            ids,
            &ctx,
            &held_codes,
            &span_valid,
        )?;
        let mut scores = crate::geometric_context_credit::frozen_potential_forward(
            potential_weights,
            &content,
            &ctx,
        )?;
        if !credit_routes[0] {
            values.values = values.values.detach();
        }
        if !credit_routes[1] {
            null.scores = null.scores.detach();
        }
        if !credit_routes[2] {
            scores.scores = scores.scores.detach();
        }
        let composed = composition_weights.forward_frozen_values(&values.values, &values.trace)?;
        let p = self.params()?;
        let span_config = self
            .geometric_span
            .as_ref()
            .ok_or_else(|| invalid("context credit requires span configuration"))?;
        span.validate_for(p.get("embedding.weight")?, span_config)?;
        let tokens = self.embed_with(&p, ids, batch, time)?;
        let read_trace = std::cell::RefCell::new(None);
        let x = self.layer_range_with_source(
            &p,
            tokens.clone(),
            0..self.config.layers(),
            &mut None,
            &mut None,
            LatchGates::Soft,
            ReadSource {
                tokens: Some(&tokens),
                native_span: Some((span, ids)),
                native_potential: Some(potential),
                event_control: Some(SpanEvents::Native(&event.actions)),
                context: Some(ContextInput::Native(&ctx.trace.codes)),
                native_reducer: Some(reducer),
                native_trace: Some(&read_trace),
                native_learned_values: Some(&values.trace.values_q16),
                composition: Some(ReadComposition::ContextCredit {
                    values: &composed,
                    potential: &scores,
                }),
                no_read: NoReadSource::ContextCredit {
                    raw: &null.scores_q24,
                    live: &null.scores,
                },
                ..ReadSource::default()
            },
        )?;
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        let logits = hidden.matmul(&p.head()?.t()?)?;
        Ok((logits, ctx, scores, values, null))
    }

    fn forward_geometric_composition_mode(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        producer: &crate::geometric_value_producer_native::CompiledValueProducer,
        reset_each_token: bool,
        no_read: NoReadMode<'_>,
        composition: Option<CompositionMode<'_>>,
    ) -> Result<(
        Tensor,
        Option<crate::geometric_read_native::NativeReadTrace>,
        crate::geometric_value_producer::ValueProducerTrace,
        Option<Tensor>,
        Option<crate::geometric_composition_native::ComposedReadTrace>,
    )> {
        self.validate_context_config(context.config())?;
        self.validate_context_dependencies(context, events, span, potential)?;
        producer.validate_native_context(context)?;
        if self.config.width / self.config.heads != crate::geometric_value_producer::VALUE_WIDTH
            || reducer.metadata().layer != 2
        {
            return Err(invalid("native learned value reader layout differs"));
        }
        if let Some(mode) = composition {
            if self.config.width != 32 || self.config.heads != 2 || time > 128 {
                return Err(invalid(
                    "composition requires two heads, width32 and context<=128",
                ));
            }
            let NoReadMode::Native(null) = no_read else {
                return Err(invalid("composition requires admitted native NoRead"));
            };
            match mode {
                CompositionMode::LearnedPotential {
                    weights,
                    composition,
                    ..
                } => {
                    potential.validate_q4_training(weights)?;
                    composition.validate_dependencies(
                        context, potential, reducer, producer, events, span, null,
                    )?;
                }
                CompositionMode::Learned(weights) => weights.config().validate()?,
                CompositionMode::FrozenValues {
                    producer: weights,
                    composition,
                    ..
                } => {
                    if !weights.is_q4() || weights.config() != producer.config() {
                        return Err(invalid(
                            "q4 live value producer differs from admitted training config",
                        ));
                    }
                    composition.config().validate()?;
                }
                CompositionMode::Native(compiled) => compiled.validate_dependencies(
                    context, potential, reducer, producer, events, span, null,
                )?,
            }
        }
        let ctx = geometric_context::trace_native(ids, batch, time, context, reset_each_token)?;
        let event = geometric_event::trace_native(ids, batch, time, events, false)?;
        let held =
            geometric_span_native::trace_native_events(ids, batch, time, &event.actions, span)?;
        let heads = self.config.heads;
        match no_read {
            NoReadMode::Native(n) => {
                n.validate_dependencies(context, potential, reducer, producer, events, span)?
            }
            NoReadMode::Learned(n)
                if n.config().vocabulary != self.config.vocab_size
                    || n.config().heads != heads
                    || n.config().latent_lanes_per_head != context.config().lanes_per_head =>
            {
                return Err(invalid("NoRead/context shape differs"))
            }
            _ => {}
        }
        let lanes = heads * context.config().lanes_per_head;
        let learned = matches!(no_read, NoReadMode::Learned(_));
        let learned_values = matches!(composition, Some(CompositionMode::FrozenValues { .. }));
        let collect_features = learned || learned_values;
        let feature_count = if collect_features {
            ids.len() * lanes
        } else {
            0
        };
        let mut all_latent = Vec::with_capacity(feature_count);
        let mut all_held = Vec::with_capacity(feature_count);
        let mut span_valid = Vec::with_capacity(if collect_features { ids.len() } else { 0 });
        let mut native_null = if matches!(no_read, NoReadMode::Native(_)) {
            vec![0i64; batch * heads * time]
        } else {
            Vec::new()
        };
        let mut raw = vec![0i32; batch * heads * time * 16];
        let mut packets =
            vec![
                std::array::from_fn(|_| crate::geometric_value_native::ValuePacketRecord {
                    status: crate::geometric_value_native::ValuePacketStatus::Absent,
                    root: 1,
                    radius_bin: 0,
                });
                batch * heads * time * 4
            ];
        for row in 0..ids.len() {
            let latent = ctx.states[row]
                .iter()
                .map(|&code| {
                    uor_r4_integer::h4_tables::H4Code::try_from(code)
                        .map_err(|e| invalid(e.to_string()))
                })
                .collect::<Result<Vec<_>>>()?;
            let span_codes = held.prior_codes[row]
                .as_ref()
                .map(|codes| {
                    codes
                        .iter()
                        .map(|&code| {
                            uor_r4_integer::h4_tables::H4Code::try_from(code)
                                .map_err(|e| invalid(e.to_string()))
                        })
                        .collect::<Result<Vec<_>>>()
                })
                .transpose()?;
            if collect_features {
                all_latent.extend_from_slice(&latent);
                span_valid.push(span_codes.is_some());
                if let Some(held) = &span_codes {
                    all_held.extend_from_slice(held);
                } else {
                    all_held.extend(std::iter::repeat_n(
                        uor_r4_integer::h4_tables::H4Code::try_from(1)
                            .map_err(|e| invalid(e.to_string()))?,
                        lanes,
                    ));
                }
            }
            if let NoReadMode::Native(n) = no_read {
                let scores = n.score(
                    ids[row] as usize,
                    &latent,
                    &ctx.codes[row * lanes..(row + 1) * lanes],
                    span_codes.as_deref(),
                )?;
                let (b, t) = (row / time, row % time);
                for h in 0..heads {
                    native_null[(b * heads + h) * time + t] = scores[h];
                }
            }
            if learned_values {
                continue;
            }
            let produced =
                producer.produce(ids[row] as usize, &latent, span_codes.as_deref(), true)?;
            let (b, t) = (row / time, row % time);
            for h in 0..heads {
                let at = (b * heads + h) * time + t;
                raw[at * 16..(at + 1) * 16]
                    .copy_from_slice(&produced.values_q16[h * 16..(h + 1) * 16]);
                for lane in 0..4 {
                    for atom in 0..2 {
                        let value = produced.packets[h * 4 + lane][atom];
                        packets[at * 4 + lane][atom] = crate::geometric_value_native::ValuePacketRecord {
                            status: match value.state() {
                                uor_r4_integer::geometric_value::ValueState::Absent => crate::geometric_value_native::ValuePacketStatus::Absent,
                                uor_r4_integer::geometric_value::ValueState::PresentZero => crate::geometric_value_native::ValuePacketStatus::PresentZero,
                                uor_r4_integer::geometric_value::ValueState::PresentNonzero => crate::geometric_value_native::ValuePacketStatus::PresentNonzero,
                            }, root: value.root().index(), radius_bin: value.radius_bin(),
                        };
                    }
                }
            }
        }
        let mut learned_output =
            if let Some(CompositionMode::FrozenValues { producer, .. }) = composition {
                Some(producer.forward_q4_codes(
                    ids,
                    batch,
                    time,
                    &all_latent,
                    &all_held,
                    &span_valid,
                    &vec![true; batch * time],
                )?)
            } else {
                None
            };
        let values_trace = crate::geometric_value_producer::ValueProducerTrace {
            batch,
            heads,
            time,
            occurrence_valid: vec![true; batch * time],
            packets,
            values_q16: raw,
        };
        let values_trace = learned_output
            .as_ref()
            .map(|out| out.trace.clone())
            .unwrap_or(values_trace);
        let composed_tensor = match composition {
            Some(CompositionMode::LearnedPotential { composition, .. }) => {
                let raw = composition.compose_trace(&values_trace)?;
                Some(Tensor::from_vec(
                    raw.into_iter()
                        .map(|v| (v as f64 / 65536.) as f32)
                        .collect::<Vec<_>>(),
                    (batch, heads, time, 32),
                    &self.device,
                )?)
            }
            Some(CompositionMode::Learned(weights)) => Some(weights.forward(&values_trace)?),
            Some(CompositionMode::FrozenValues { composition, .. }) => Some(
                composition.forward_frozen_values(
                    &learned_output
                        .as_ref()
                        .ok_or_else(|| invalid("q4 value Tensor missing"))?
                        .values,
                    &values_trace,
                )?,
            ),
            _ => None,
        };
        let composed_raw = match composition {
            Some(CompositionMode::Native(compiled)) => Some(compiled.compose_trace(&values_trace)?),
            _ => None,
        };
        let composition_trace = std::cell::RefCell::new(None);
        let read_composition = match composition {
            Some(CompositionMode::LearnedPotential {
                weights, output, ..
            }) => Some(ReadComposition::LearnedPotential {
                values: composed_tensor
                    .as_ref()
                    .ok_or_else(|| invalid("fixed composition tensor missing"))?,
                weights,
                output,
            }),
            Some(CompositionMode::Learned(_)) | Some(CompositionMode::FrozenValues { .. }) => {
                Some(ReadComposition::Learned(
                    composed_tensor
                        .as_ref()
                        .ok_or_else(|| invalid("composition tensor missing"))?,
                ))
            }
            Some(CompositionMode::Native(compiled)) => Some(ReadComposition::Native {
                values: composed_raw
                    .as_deref()
                    .ok_or_else(|| invalid("composition values missing"))?,
                compiled,
                trace: &composition_trace,
            }),
            None => None,
        };
        let learned_null = match no_read {
            NoReadMode::Learned(n) => Some(n.forward(crate::geometric_no_read::NoReadBatch {
                ids,
                batch,
                time,
                latent: &all_latent,
                observed: &ctx.codes,
                held: &all_held,
                span_valid: &span_valid,
            })?),
            _ => None,
        };
        let null_source = match no_read {
            NoReadMode::Legacy => NoReadSource::LegacyFloat,
            NoReadMode::Learned(_) => NoReadSource::LearnedTensor(
                learned_null
                    .as_ref()
                    .ok_or_else(|| invalid("learned NoRead result missing"))?,
            ),
            NoReadMode::Native(_) => NoReadSource::NativeQ24(&native_null),
        };
        let p = self.params()?;
        let span_config = self
            .geometric_span
            .as_ref()
            .ok_or_else(|| invalid("native values require spans"))?;
        span.validate_for(p.get("embedding.weight")?, span_config)?;
        let tokens = self.embed_with(&p, ids, batch, time)?;
        let read_trace = std::cell::RefCell::new(None);
        let x = self.layer_range_with_source(
            &p,
            tokens.clone(),
            0..self.config.layers(),
            &mut None,
            &mut None,
            LatchGates::Soft,
            ReadSource {
                tokens: Some(&tokens),
                native_span: Some((span, ids)),
                native_potential: Some(potential),
                event_control: Some(SpanEvents::Native(&event.actions)),
                context: Some(ContextInput::Native(&ctx.codes)),
                native_reducer: Some(reducer),
                native_trace: Some(&read_trace),
                native_learned_values: Some(&values_trace.values_q16),
                composition: read_composition,
                no_read: null_source,
                ..ReadSource::default()
            },
        )?;
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        let logits = hidden.matmul(&p.head()?.t()?)?;
        if let Some(CompositionMode::FrozenValues { output, .. }) = composition {
            let mut sink = output
                .try_borrow_mut()
                .map_err(|_| invalid("q4 value output already borrowed"))?;
            if sink.is_some() {
                return Err(invalid("q4 value producer evaluated more than once"));
            }
            *sink = learned_output.take();
        }
        Ok((
            logits,
            read_trace.into_inner(),
            values_trace,
            learned_null,
            composition_trace.into_inner(),
        ))
    }

    fn forward_geometric_context_value_projection_mode_with_trace(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        values: &crate::geometric_value_native::CompiledGeometricValues,
        reset_each_token: bool,
        residual: bool,
    ) -> Result<(
        Tensor,
        crate::geometric_read_native::NativeReadTrace,
        crate::geometric_value_native::ValueProjectionTrace,
    )> {
        let read_trace = std::cell::RefCell::new(None);
        let value_trace = std::cell::RefCell::new(None);
        let (hidden, _) = self.hidden_geometric_context_read_native(
            ids,
            batch,
            time,
            None,
            context,
            events,
            span,
            potential,
            reducer,
            reset_each_token,
            Some(&read_trace),
            Some(NativeValueProjection {
                residual,
                compiled: values,
                trace: &value_trace,
            }),
        )?;
        let p = self.params()?;
        let logits = hidden.matmul(&p.head()?.t()?)?;
        Ok((
            logits,
            read_trace
                .into_inner()
                .ok_or_else(|| invalid("projected native read trace absent"))?,
            value_trace
                .into_inner()
                .ok_or_else(|| invalid("native value projection trace absent"))?,
        ))
    }

    pub fn read_binding_masses_geometric_context_read_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        target: &ReadBindingTarget,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        self.hidden_geometric_context_read_native(
            ids,
            batch,
            time,
            Some(target),
            context,
            events,
            span,
            potential,
            reducer,
            reset_each_token,
            None,
            None,
        )?
        .1
        .ok_or_else(|| invalid("native weighted source observer was not evaluated"))
    }

    fn hidden_geometric_context_read_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        target: Option<&ReadBindingTarget>,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        reset_each_token: bool,
        native_trace: NativeReadTraceSink<'_>,
        native_value_projection: Option<NativeValueProjection<'_>>,
    ) -> Result<(Tensor, Option<Tensor>)> {
        self.validate_context_config(context.config())?;
        self.validate_context_dependencies(context, events, span, potential)?;
        if reducer.metadata().layer != 2 {
            return Err(invalid(
                "native reduction artifact is bound to a different read layer",
            ));
        }
        let ctx = geometric_context::trace_native(ids, batch, time, context, reset_each_token)?;
        let event = geometric_event::trace_native(ids, batch, time, events, false)?;
        self.hidden_geometric_context_reduced(
            ids,
            batch,
            time,
            SpanEvents::Native(&event.actions),
            target,
            Some(span),
            Some(potential),
            Some(ContextInput::Native(&ctx.codes)),
            Some(reducer),
            native_trace,
            native_value_projection,
        )
    }

    pub fn read_binding_masses_geometric_context_native(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        target: &ReadBindingTarget,
        context: &CompiledContext,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        self.validate_context_dependencies(context, events, span, potential)?;
        let ctx = geometric_context::trace_native(ids, batch, time, context, reset_each_token)?;
        let event = geometric_event::trace_native(ids, batch, time, events, false)?;
        self.hidden_geometric_context(
            ids,
            batch,
            time,
            SpanEvents::Native(&event.actions),
            Some(target),
            Some(span),
            Some(potential),
            Some(ContextInput::Native(&ctx.codes)),
        )?
        .1
        .ok_or_else(|| invalid("native context source observer absent"))
    }

    /// Hard learned float producer through the same integer scorer, solely to
    /// separate donor approximation from context table compilation drift.
    pub fn forward_geometric_context_replay(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        context: &ContextWeights,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        self.validate_context_config(context.config())?;
        let codes = context
            .float_trace(ids, batch, time, reset_each_token)?
            .address_codes()?;
        let event = geometric_event::trace_native(ids, batch, time, events, false)?;
        let (hidden, _) = self.hidden_geometric_context(
            ids,
            batch,
            time,
            SpanEvents::Native(&event.actions),
            None,
            Some(span),
            Some(potential),
            Some(ContextInput::Native(&codes)),
        )?;
        let p = self.params()?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Hard learned float producer through the same integer scorer, solely to
    /// separate donor approximation from context table compilation drift.
    pub fn read_binding_masses_geometric_context_replay(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        target: &ReadBindingTarget,
        context: &ContextWeights,
        events: &CompiledEvents,
        span: &CompiledSpanActions,
        potential: &CompiledGeometricPotentials,
        reset_each_token: bool,
    ) -> Result<Tensor> {
        self.validate_context_config(context.config())?;
        let codes = context
            .float_trace(ids, batch, time, reset_each_token)?
            .address_codes()?;
        let event = geometric_event::trace_native(ids, batch, time, events, false)?;
        self.hidden_geometric_context(
            ids,
            batch,
            time,
            SpanEvents::Native(&event.actions),
            Some(target),
            Some(span),
            Some(potential),
            Some(ContextInput::Native(&codes)),
        )?
        .1
        .ok_or_else(|| invalid("context replay source observer absent"))
    }

    fn hidden_geometric_event(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        control: SpanEvents<'_>,
        target: Option<&ReadBindingTarget>,
        compiled: Option<&CompiledSpanActions>,
        potential: Option<&CompiledGeometricPotentials>,
    ) -> Result<(Tensor, Option<Tensor>)> {
        self.hidden_geometric_context(ids, batch, time, control, target, compiled, potential, None)
    }

    fn hidden_geometric_context(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        control: SpanEvents<'_>,
        target: Option<&ReadBindingTarget>,
        compiled: Option<&CompiledSpanActions>,
        potential: Option<&CompiledGeometricPotentials>,
        context: Option<ContextInput<'_>>,
    ) -> Result<(Tensor, Option<Tensor>)> {
        self.hidden_geometric_context_reduced(
            ids, batch, time, control, target, compiled, potential, context, None, None, None,
        )
    }

    fn hidden_geometric_context_reduced(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        control: SpanEvents<'_>,
        target: Option<&ReadBindingTarget>,
        compiled: Option<&CompiledSpanActions>,
        potential: Option<&CompiledGeometricPotentials>,
        context: Option<ContextInput<'_>>,
        native_reducer: Option<&crate::geometric_read_native::CompiledGeometricRead>,
        native_trace: NativeReadTraceSink<'_>,
        native_value_projection: Option<NativeValueProjection<'_>>,
    ) -> Result<(Tensor, Option<Tensor>)> {
        if matches!(control, SpanEvents::TrainingNative { .. }) && compiled.is_some() {
            return Err(invalid(
                "native-action event training cannot use the detached compiled-span replay",
            ));
        }
        let span = self
            .geometric_span
            .as_ref()
            .ok_or_else(|| invalid("events require geometric spans"))?;
        let address = self
            .geometric_address
            .as_ref()
            .ok_or_else(|| invalid("events require geometric addressing"))?;
        self.validate_geometric_address(address)?;
        let p = self.params()?;
        if let Some(compiled) = compiled {
            compiled.validate_for(p.get("embedding.weight")?, span)?;
        }
        if let Some(target) = target {
            self.validate_binding(batch, time, target)?;
        }
        let tokens = self.embed_with(&p, ids, batch, time)?;
        let mut binding = target.map(|target| BindingCapture {
            target,
            masses: None,
        });
        let x = self.layer_range_with_source(
            &p,
            tokens.clone(),
            0..self.config.layers(),
            &mut None,
            &mut binding,
            LatchGates::Soft,
            ReadSource {
                tokens: Some(&tokens),
                span_policy: SpanPolicy::Ordered,
                native_span: compiled.map(|compiled| (compiled, ids)),
                native_potential: potential,
                event_control: Some(control),
                context,
                native_reducer,
                native_trace,
                native_value_projection,
                learned_values: None,
                native_learned_values: None,
                no_read: NoReadSource::LegacyFloat,
                composition: None,
                integer_residual: None,
            },
        )?;
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        Ok((hidden, binding.and_then(|capture| capture.masses)))
    }

    fn validate_geometric_address(&self, address: &GeometricAddressConfig) -> Result<()> {
        self.config.validate()?;
        address.validate(self.config.width, self.config.heads)?;
        if !self.device.is_cpu()
            || self.config.arch != StackArch::Geometric
            || self.config.pattern != "rra"
            || self.config.select.is_some()
            || self.config.pointer.is_some()
            || self.served.is_some()
            || self.read_identity_carry
            || !matches!(
                (&self.geometric_span, self.read_identity_latch),
                (None, Some(ReadIdentityLatch::Held)) | (Some(_), None)
            )
        {
            return Err(invalid(
                "geometric addressing requires CPU rra, a Held latch or span producer, full causal support and no pointer, carry or served view",
            ));
        }
        Ok(())
    }

    fn address_weights(&self, p: &Params<'_>, layer: usize) -> Result<AddressWeights> {
        let get = |name: &str| p.layer(layer, &format!("read.address.{name}")).cloned();
        Ok(AddressWeights {
            content_unary: get("content_unary")?,
            context_unary: get("context_unary")?,
            pair: get("pair")?,
            content_radius: get("content_radius")?,
            context_radius: get("context_radius")?,
            content_presence: get("content_presence")?,
            context_presence: get("context_presence")?,
        })
    }

    fn geometric_address_read(
        &self,
        p: &Params<'_>,
        layer: usize,
        x: &Tensor,
        capture: &mut Capture<'_>,
        binding: &mut Option<BindingCapture<'_>>,
        gates: LatchGates,
        config: &GeometricAddressConfig,
        source: ReadSource<'_>,
    ) -> Result<Tensor> {
        if layer != 2 || matches!(gates, LatchGates::Replay(_) | LatchGates::Projections(..)) {
            return Err(invalid(
                "geometric addressing does not accept numerical q/k or identity replay",
            ));
        }
        let (batch, time, _) = x.dims3()?;
        let heads = self.config.heads;
        if !matches!(source.no_read, NoReadSource::LegacyFloat) {
            if capture.is_some() {
                return Err(invalid(
                    "native NoRead path does not expose legacy normalized-read capture",
                ));
            }
            if source.native_learned_values.is_none() || source.native_value_projection.is_some() {
                return Err(invalid(
                    "native NoRead requires actual native learned values",
                ));
            }
            let reducer = source
                .native_reducer
                .ok_or_else(|| invalid("native NoRead requires admitted reducer"))?;
            let weights = self.address_weights(p, layer)?;
            return self.read_geometric_reduced(
                p, layer, None, batch, time, source, config, &weights, reducer, capture, binding,
            );
        }
        tap(capture, StackSite::Read(layer), || self.unit_norm(x))?;
        let u = self.norm(p, x, &layer_name(layer, "read_norm.weight"))?;
        let prior = if matches!(source.context, Some(ContextInput::Native(_))) {
            None
        } else {
            Some(match &self.geometric_span {
                Some(span) => {
                    if !matches!(gates, LatchGates::Soft) {
                        return Err(invalid(
                            "geometric spans do not use binary latch gate overrides",
                        ));
                    }
                    let tokens = source.tokens.ok_or_else(|| {
                        invalid("geometric spans require original selected token rows")
                    })?;
                    match source.event_control {
                        Some(SpanEvents::TrainingNative {
                            logits, actions, ..
                        }) => {
                            if source.native_span.is_some() {
                                return Err(invalid(
                                    "native-action event training requires connected span credit",
                                ));
                            }
                            geometric_span::produce_with_native_actions(
                                tokens,
                                logits,
                                span,
                                source.span_policy,
                                actions,
                            )?
                        }
                        Some(SpanEvents::Native(events)) => {
                            let (compiled, ids) = source.native_span.ok_or_else(|| {
                                invalid("native events require the compiled span dictionary")
                            })?;
                            if source.span_policy != SpanPolicy::Ordered {
                                return Err(invalid("native events require ordered spans"));
                            }
                            geometric_span_native::produce_native_events(
                                ids, batch, time, events, compiled,
                            )?
                        }
                        control => {
                            let predicted;
                            let logits = match control {
                                Some(SpanEvents::Training(logits)) => logits,
                                None => {
                                    predicted = self.span_control_logits(p, layer, &u)?;
                                    &predicted
                                }
                                Some(SpanEvents::Native(_))
                                | Some(SpanEvents::TrainingNative { .. }) => {
                                    return Err(invalid("invalid native event branch"))
                                }
                            };
                            match source.native_span {
                                Some((compiled, ids)) => {
                                    if source.span_policy != SpanPolicy::Ordered {
                                        return Err(invalid(
                                            "native span replay does not accept LastToken",
                                        ));
                                    }
                                    geometric_span_native::produce_native(
                                        ids, batch, time, logits, compiled,
                                    )?
                                }
                                None => geometric_span::produce(
                                    tokens,
                                    logits,
                                    span,
                                    source.span_policy,
                                )?,
                            }
                        }
                    }
                }
                None => {
                    self.read_latch_inputs(p, layer, &u, ReadIdentityLatch::Held, gates)?
                        .0
                }
            })
        };
        let weights = self.address_weights(p, layer)?;
        if let Some(reducer) = source.native_reducer {
            return self.read_geometric_reduced(
                p,
                layer,
                Some(&u),
                batch,
                time,
                source,
                config,
                &weights,
                reducer,
                capture,
                binding,
            );
        }
        let mut scores = match source.native_potential {
            Some(compiled) => {
                if source.native_span.is_none() {
                    return Err(invalid(
                        "native geometric potentials require the compiled span producer",
                    ));
                }
                match source.context {
                    Some(ContextInput::Native(current)) => {
                        let (span, ids) = source
                            .native_span
                            .ok_or_else(|| invalid("typed context requires native spans"))?;
                        let events = match source.event_control {
                            Some(SpanEvents::Native(events)) => events,
                            _ => return Err(invalid("typed context requires native events")),
                        };
                        let trace = geometric_span_native::trace_native_events(
                            ids, batch, time, events, span,
                        )?;
                        if trace.lanes != config.heads * config.lanes_per_head {
                            return Err(invalid("typed span lane layout differs"));
                        }
                        let mut content = Vec::with_capacity(current.len());
                        for row in trace.prior_codes {
                            for lane in 0..config.heads * config.lanes_per_head {
                                let code = match &row {
                                    Some(roots) => {
                                        uor_r4_integer::geometric_potential::AddressLane::new(
                                            roots[lane],
                                            16,
                                            true,
                                        )
                                    }
                                    None => uor_r4_integer::geometric_potential::AddressLane::new(
                                        1, 0, false,
                                    ),
                                }
                                .map_err(|e| invalid(e.to_string()))?;
                                content.push(code);
                            }
                        }
                        geometric_potential_native::score_native_codes(
                            current, &content, batch, time, config, &weights, compiled,
                        )?
                    }
                    Some(ContextInput::Training(_)) => {
                        return Err(invalid(
                            "training context cannot use the nondifferentiable compiled scorer",
                        ))
                    }
                    None => geometric_potential_native::score_native(
                        &u,
                        prior
                            .as_ref()
                            .ok_or_else(|| invalid("float prior absent"))?,
                        config,
                        &weights,
                        compiled,
                    )?,
                }
            }
            None => {
                let current = match source.context {
                    Some(ContextInput::Training(context)) => context,
                    Some(ContextInput::Native(_)) => {
                        return Err(invalid("typed context requires compiled potentials"))
                    }
                    None => &u,
                };
                geometric_address::score(
                    current,
                    prior
                        .as_ref()
                        .ok_or_else(|| invalid("float prior absent"))?,
                    config,
                    &weights,
                )?
            }
        };
        if scores.dims4()? != (batch, heads, time, time) {
            return Err(invalid(
                "geometric address scores differ from [batch,heads,time,time]",
            ));
        }
        let mut ages = Vec::with_capacity(time * time);
        let mut mask = Vec::with_capacity(time * time);
        for query in 0..time {
            for source in 0..time {
                ages.push(query.saturating_sub(source) as u32);
                mask.push(if source <= query {
                    0.0f32
                } else {
                    f32::NEG_INFINITY
                });
            }
        }
        let ages = Tensor::from_vec(ages, time * time, &self.device)?;
        let age_mode = match source.event_control {
            Some(SpanEvents::TrainingNative { age, .. }) => age,
            _ => AgeTrainingMode::None,
        };
        let age_source = match age_mode {
            AgeTrainingMode::None => p.layer(layer, "read.age")?.clone(),
            AgeTrainingMode::Quarter => crate::geometric_read_native::q4_age_training_view(
                p.layer(layer, "read.age")?,
                heads,
                self.config.context,
            )?,
            AgeTrainingMode::PriorResidual => {
                crate::geometric_read_native::q4_age_residual_training_view(
                    p.layer(layer, "read.age")?,
                    heads,
                    self.config.context,
                )?
            }
        };
        let age = age_source
            .index_select(&ages, 1)?
            .reshape((1, heads, time, time))?;
        scores = scores
            .broadcast_add(&age)?
            .broadcast_add(&Tensor::from_vec(mask, (1, 1, time, time), &self.device)?)?;
        let null = Self::linear(&u, p.layer(layer, "read.null.weight")?)?
            .broadcast_add(p.layer(layer, "read.null.bias")?)?
            .transpose(1, 2)?
            .unsqueeze(3)?;
        let scores = Tensor::cat(&[&null, &scores], 3)?;
        let value = if let Some(learned) = source.learned_values {
            let (compiled, ids) = source
                .native_span
                .ok_or_else(|| invalid("learned geometric values require actual compiled spans"))?;
            if ids != learned.ids {
                return Err(invalid("learned value token provenance differs"));
            }
            let events = match source.event_control {
                Some(SpanEvents::Native(events)) => events,
                _ => return Err(invalid("learned values require actual native events")),
            };
            let trace =
                geometric_span_native::trace_native_events(ids, batch, time, events, compiled)?;
            let span_valid: Vec<bool> = trace.prior_codes.iter().map(Option::is_some).collect();
            let held = prior
                .as_ref()
                .ok_or_else(|| invalid("learned value held span absent"))?
                .reshape((batch, time, heads, self.config.width / heads / 4, 4))?;
            // Existing reference scores admit all causal occurrences; validity
            // is explicit and never inferred from a zero numerical payload.
            let occurrence_valid = vec![true; batch * time];
            learned
                .producer
                .forward(ids, learned.latent, &held, &span_valid, &occurrence_valid)?
                .values
        } else {
            self.heads(
                &Self::linear(&u, p.layer(layer, "read.value.weight")?)?,
                batch,
                time,
            )?
        };
        let (value, value_width) = self.read_binding_values(value, layer, binding)?;
        let value = Tensor::cat(
            &[
                &Tensor::zeros((batch, heads, 1, value.dim(3)?), DType::F32, &self.device)?,
                &value,
            ],
            2,
        )?;
        let read = candle_nn::ops::softmax(&scores, 3)?.matmul(&value)?;
        self.finish_geometric_read(p, layer, &read, value_width, capture, binding)
    }

    /// Offline saved-model bridge: only the weighted geometric reduction is
    /// native here. The donor still produces NoRead and payload values.
    fn read_geometric_reduced(
        &self,
        p: &Params<'_>,
        layer: usize,
        u: Option<&Tensor>,
        batch: usize,
        time: usize,
        source: ReadSource<'_>,
        config: &GeometricAddressConfig,
        weights: &AddressWeights,
        reducer: &crate::geometric_read_native::CompiledGeometricRead,
        capture: &mut Capture<'_>,
        binding: &mut Option<BindingCapture<'_>>,
    ) -> Result<Tensor> {
        let potential = source
            .native_potential
            .ok_or_else(|| invalid("native reduction requires native geometric potentials"))?;
        potential.validate_stack_parent(weights, config)?;
        reducer.validate_base_for(p.layer(layer, "read.age")?, potential)?;
        if batch == 0
            || time == 0
            || time > reducer.metadata().context
            || config.heads != reducer.metadata().heads
        {
            return Err(invalid("native read dimensions differ from reducer"));
        }
        let current = match source.context {
            Some(ContextInput::Native(codes)) => codes,
            _ => return Err(invalid("native reduction requires typed native context")),
        };
        let (span, ids) = source
            .native_span
            .ok_or_else(|| invalid("native reduction requires native spans"))?;
        let events = match source.event_control {
            Some(SpanEvents::Native(events)) => events,
            _ => return Err(invalid("native reduction requires native events")),
        };
        let trace = geometric_span_native::trace_native_events(ids, batch, time, events, span)?;
        let heads = config.heads;
        let lanes = config.lanes_per_head;
        let count = batch
            .checked_mul(time)
            .and_then(|n| n.checked_mul(heads))
            .and_then(|n| n.checked_mul(lanes))
            .ok_or_else(|| invalid("native reduction address shape overflow"))?;
        if trace.lanes != heads * lanes
            || trace.prior_codes.len() != batch * time
            || current.len() != count
        {
            return Err(invalid("native reduction address layout differs"));
        }
        let mut content = Vec::with_capacity(count);
        for row in trace.prior_codes {
            for lane in 0..heads * lanes {
                let code = match &row {
                    Some(roots) => {
                        if roots.len() != heads * lanes {
                            return Err(invalid("native reduction span lane layout differs"));
                        }
                        uor_r4_integer::geometric_potential::AddressLane::new(roots[lane], 16, true)
                    }
                    None => uor_r4_integer::geometric_potential::AddressLane::new(1, 0, false),
                }
                .map_err(|e| invalid(e.to_string()))?;
                content.push(code);
            }
        }
        let score_count = batch
            .checked_mul(heads)
            .and_then(|n| n.checked_mul(time))
            .and_then(|n| n.checked_mul(time))
            .ok_or_else(|| invalid("native reduction score layout overflow"))?;
        // This allocation is the offline full-window bridge, not the native
        // row reducer's incremental serving scratch.
        let learned_potential = match source.composition {
            Some(ReadComposition::LearnedPotential { weights, .. }) => {
                Some(weights.forward_codes(batch, time, &content, current)?)
            }
            _ => None,
        };
        let context_potential = match source.composition {
            Some(ReadComposition::ContextCredit { potential, .. }) => Some(potential),
            _ => None,
        };
        if matches!(source.no_read, NoReadSource::ContextCredit { .. })
            != context_potential.is_some()
        {
            return Err(invalid(
                "context credit requires coupled potential and NoRead graph",
            ));
        }
        let learned_score_tensor = context_potential
            .map(|out| out.scores.clone())
            .or_else(|| learned_potential.as_ref().map(|out| out.scores.clone()));
        let scores = if let Some(learned) = context_potential {
            if learned.scores_q24.len() != score_count
                || learned.scores.dims() != [batch, heads, time, time]
                || learned.content_codes != content
                || learned.context_codes != current
            {
                return Err(invalid(
                    "context credit potential endpoints/score layout differ",
                ));
            }
            learned.scores_q24.clone()
        } else if let Some(ref learned) = learned_potential {
            learned.scores_q24.clone()
        } else {
            let mut scores = vec![0i64; score_count];
            for b in 0..batch {
                for head in 0..heads {
                    for query in 0..time {
                        let q = ((b * time + query) * heads + head) * lanes;
                        for key in 0..=query {
                            let k = ((b * time + key) * heads + head) * lanes;
                            scores[((b * heads + head) * time + query) * time + key] = potential
                                .score_pair_codes(
                                    head,
                                    &content[q..q + lanes],
                                    &content[k..k + lanes],
                                    &current[q..q + lanes],
                                    &current[k..k + lanes],
                                )?;
                        }
                    }
                }
            }
            scores
        };
        if let Some(ReadComposition::LearnedPotential { output, .. }) = source.composition {
            let mut sink = output
                .try_borrow_mut()
                .map_err(|_| invalid("q4 potential output already borrowed"))?;
            if sink.is_some() {
                return Err(invalid("q4 potential evaluated more than once"));
            }
            *sink = learned_potential;
        }
        if let Some(composition) = source.composition {
            if heads != 2 || self.config.width != 32 || binding.is_some() {
                return Err(invalid("composition requires H2/width32 and uses returned raw mass trace, not a binding channel"));
            }
            let null = match source.no_read {
                NoReadSource::NativeQ24(raw) | NoReadSource::ContextCredit { raw, .. } => raw,
                _ => return Err(invalid("composition requires raw native NoRead")),
            };
            if null.len() != batch * heads * time {
                return Err(invalid("composition null shape differs"));
            }
            return match composition {
                ReadComposition::Learned(values)
                | ReadComposition::LearnedPotential { values, .. }
                | ReadComposition::ContextCredit { values, .. } => {
                    if values.dims() != [batch, heads, time, 32]
                        || values.dtype() != DType::F32
                        || !values.device().is_cpu()
                    {
                        return Err(invalid("composition value tensor shape differs"));
                    }
                    let mut fixed_scores = vec![f32::NEG_INFINITY; score_count];
                    for b in 0..batch {
                        for h in 0..heads {
                            for q in 0..time {
                                for k in 0..=q {
                                    let index = ((b * heads + h) * time + q) * time + k;
                                    let age =
                                        reducer.age_q24()[h * reducer.metadata().context + q - k];
                                    let sum = scores[index]
                                        .checked_add(age)
                                        .ok_or_else(|| invalid("composition score+age overflow"))?;
                                    fixed_scores[index] = (sum as f64 / 16777216.) as f32;
                                }
                            }
                        }
                    }
                    let fixed =
                        Tensor::from_vec(fixed_scores, (batch, heads, time, time), &self.device)?;
                    // Preserve exact Q24 score+age hard values and their F32 cast;
                    // the live potential Tensor supplies only the declared score
                    // adjoint. Future entries are still masked by -infinity.
                    let fixed = if let Some(live) = &learned_score_tensor {
                        (&fixed + (live - live.detach())?)?
                    } else {
                        fixed
                    };
                    let null = Tensor::from_vec(
                        null.iter()
                            .map(|&x| (x as f64 / 16777216.) as f32)
                            .collect::<Vec<_>>(),
                        (batch, heads, time, 1),
                        &self.device,
                    )?;
                    let null = if let NoReadSource::ContextCredit { live, .. } = source.no_read {
                        if live.dims() != [batch, heads, time]
                            || live.dtype() != DType::F32
                            || !live.device().is_cpu()
                        {
                            return Err(invalid("context credit NoRead graph layout differs"));
                        }
                        let live = live.unsqueeze(3)?;
                        (&null + (&live - live.detach())?)?
                    } else {
                        null
                    };
                    let joined = Tensor::cat(&[&null, &fixed], 3)?;
                    let zero = Tensor::zeros((batch, heads, 1, 32), DType::F32, &self.device)?;
                    let values = Tensor::cat(&[&zero, values], 2)?;
                    // Source learning uses real softmax and F32 accumulation.
                    // It is a declared surrogate, not integer-LUT bit parity.
                    Ok(candle_nn::ops::softmax(&joined, 3)?
                        .matmul(&values)?
                        .sum(1)?)
                }
                ReadComposition::Native {
                    values,
                    compiled,
                    trace,
                } => {
                    let output = crate::geometric_composition_native::reduce_composed_raw_q24(
                        &scores, null, values, batch, heads, time, potential, reducer, compiled,
                    )?;
                    let mut summed = Vec::with_capacity(batch * time * 32);
                    for b in 0..batch {
                        for t in 0..time {
                            let first = output
                                .trace
                                .rows
                                .get((b * heads) * time + t)
                                .ok_or_else(|| invalid("composition first head row absent"))?;
                            let second = output
                                .trace
                                .rows
                                .get((b * heads + 1) * time + t)
                                .ok_or_else(|| invalid("composition second head row absent"))?;
                            if first.output_q16.len() != 32 || second.output_q16.len() != 32 {
                                return Err(invalid("composition head output width differs"));
                            }
                            for c in 0..32 {
                                let value =
                                    first.output_q16[c]
                                        .checked_add(second.output_q16[c])
                                        .ok_or_else(|| invalid("composition head-sum overflow"))?;
                                summed.push((value as f64 / 65536.) as f32);
                            }
                        }
                    }
                    let result = Tensor::from_vec(summed, (batch, time, 32), &self.device)?;
                    let mut sink = trace
                        .try_borrow_mut()
                        .map_err(|_| invalid("composition trace already borrowed"))?;
                    if sink.is_some() {
                        return Err(invalid("composition evaluated more than once"));
                    }
                    *sink = Some(output.trace);
                    // Already in the residual frame: deliberately no read.out.
                    Ok(result)
                }
            };
        }
        if let NoReadSource::LearnedTensor(null) = source.no_read {
            let raw = source
                .native_learned_values
                .ok_or_else(|| invalid("learned NoRead native values missing"))?;
            let width = reducer.metadata().value_width;
            if null.dims() != [batch, heads, time]
                || null.dtype() != DType::F32
                || !null.device().is_cpu()
                || raw.len() != batch * heads * time * width
                || null
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .any(|x| !x.is_finite())
            {
                return Err(invalid(
                    "learned NoRead/value shape or finite score differs",
                ));
            }
            let mut fixed_scores = vec![-1e30f32; score_count];
            for b in 0..batch {
                for h in 0..heads {
                    for q in 0..time {
                        for k in 0..=q {
                            let index = ((b * heads + h) * time + q) * time + k;
                            let age = reducer.age_q24()[h * reducer.metadata().context + q - k];
                            let sum = scores[index]
                                .checked_add(age)
                                .ok_or_else(|| invalid("learned NoRead score+age overflow"))?;
                            fixed_scores[index] = (sum as f64 / 16777216.) as f32;
                        }
                    }
                }
            }
            let fixed = Tensor::from_vec(fixed_scores, (batch, heads, time, time), &self.device)?;
            let joined = Tensor::cat(&[&null.unsqueeze(3)?, &fixed], 3)?;
            let values = Tensor::from_vec(
                raw.iter()
                    .map(|&x| (f64::from(x) / 65536.) as f32)
                    .collect::<Vec<_>>(),
                (batch, heads, time, width),
                &self.device,
            )?;
            let (values, value_width) = self.read_binding_values(values, layer, binding)?;
            let zero = Tensor::zeros((batch, heads, 1, values.dim(3)?), DType::F32, &self.device)?;
            let values = Tensor::cat(&[&zero, &values], 2)?;
            let read = candle_nn::ops::softmax(&joined, 3)?.matmul(&values)?;
            return self.finish_geometric_read(p, layer, &read, value_width, capture, binding);
        }
        let null = match source.no_read {
            NoReadSource::LegacyFloat => {
                let u = u.ok_or_else(|| invalid("legacy NoRead normalized input missing"))?;
                Some(
                    Self::linear(u, p.layer(layer, "read.null.weight")?)?
                        .broadcast_add(p.layer(layer, "read.null.bias")?)?
                        .transpose(1, 2)?,
                )
            }
            NoReadSource::NativeQ24(_) | NoReadSource::ContextCredit { .. } => None,
            NoReadSource::LearnedTensor(_) => {
                return Err(invalid("learned NoRead dispatch differs"))
            }
        };
        let values = if source.native_learned_values.is_some() {
            None
        } else {
            Some(self.heads(
                &Self::linear(
                    u.ok_or_else(|| invalid("legacy value normalized input missing"))?,
                    p.layer(layer, "read.value.weight")?,
                )?,
                batch,
                time,
            )?)
        };
        let value_width = values
            .as_ref()
            .map(|v| v.dim(3))
            .transpose()?
            .unwrap_or(crate::geometric_value_producer::VALUE_WIDTH);
        let output = if let Some(raw) = source.native_learned_values {
            if source.native_value_projection.is_some() {
                return Err(invalid(
                    "native learned values cannot also use oracle projection",
                ));
            }
            match source.no_read {
                NoReadSource::NativeQ24(null) | NoReadSource::ContextCredit { raw: null, .. } => {
                    crate::geometric_read_native::reduce_native_raw_q24(
                        &scores,
                        null,
                        raw,
                        batch,
                        heads,
                        time,
                        value_width,
                        potential,
                        reducer,
                    )?
                }
                NoReadSource::LegacyFloat => crate::geometric_read_native::reduce_native_q16(
                    &scores,
                    null.as_ref()
                        .ok_or_else(|| invalid("legacy null missing"))?,
                    raw,
                    value_width,
                    p.layer(layer, "read.age")?,
                    potential,
                    reducer,
                )?,
                NoReadSource::LearnedTensor(_) => {
                    return Err(invalid("learned null reached integer reduction"))
                }
            }
        } else if let Some(projection) = source.native_value_projection {
            let values = values
                .as_ref()
                .ok_or_else(|| invalid("oracle donor values absent"))?;
            if projection.compiled.metadata().layer != layer {
                return Err(invalid("geometric value codec read layer differs"));
            }
            projection.compiled.validate_for(
                p.layer(layer, "read.value.weight")?,
                potential,
                reducer,
            )?;
            let projected = if projection.residual {
                crate::geometric_value_native::project_residual_native(
                    &values,
                    projection.compiled,
                )?
            } else {
                crate::geometric_value_native::project_native(&values, projection.compiled)?
            };
            let output = crate::geometric_read_native::reduce_native_q16(
                &scores,
                null.as_ref()
                    .ok_or_else(|| invalid("legacy null missing"))?,
                &projected.trace.projected_q16,
                value_width,
                p.layer(layer, "read.age")?,
                potential,
                reducer,
            )?;
            let mut sink = projection
                .trace
                .try_borrow_mut()
                .map_err(|_| invalid("geometric value projection trace already borrowed"))?;
            if sink.is_some() {
                return Err(invalid(
                    "geometric value projection evaluated more than once",
                ));
            }
            *sink = Some(projected.trace);
            output
        } else {
            crate::geometric_read_native::reduce_native(
                &scores,
                null.as_ref()
                    .ok_or_else(|| invalid("legacy null missing"))?,
                values
                    .as_ref()
                    .ok_or_else(|| invalid("native donor values absent"))?,
                p.layer(layer, "read.age")?,
                potential,
                reducer,
            )?
        };
        let target = binding
            .as_ref()
            .filter(|binding| binding.target.layer == layer)
            .map(|binding| binding.target);
        let read = if let Some(target) = target {
            // Labels observe raw normalized occurrence mass only AFTER the
            // predictive reduction. This channel is removed before read.out.
            let mut masses = vec![0f32; batch * heads * time];
            for row in &target.rows {
                masses[(row.batch * heads + target.head) * time + row.query] = output
                    .trace
                    .source_mass(row.batch, target.head, row.query, &row.sources)?;
            }
            let mass = Tensor::from_vec(masses, (batch, heads, time, 1), &self.device)?;
            Tensor::cat(&[&output.read, &mass], 3)?
        } else {
            output.read
        };
        if let Some(sink) = source.native_trace {
            let mut sink = sink
                .try_borrow_mut()
                .map_err(|_| invalid("native read trace is already borrowed"))?;
            if sink.is_some() {
                return Err(invalid("native read trace was produced more than once"));
            }
            *sink = Some(output.trace);
        }
        self.finish_geometric_read(p, layer, &read, value_width, capture, binding)
    }

    fn read_latch_gate_logits(&self, p: &Params<'_>, layer: usize, u: &Tensor) -> Result<Tensor> {
        let (batch, time, width) = u.dims3()?;
        let zero = Tensor::zeros((batch, 1, width), u.dtype(), u.device())?;
        let previous = if time == 1 {
            zero.clone()
        } else {
            Tensor::cat(&[&zero, &u.narrow(1, 0, time - 1)?], 1)?
        };
        let gate_input = Tensor::cat(&[u, &previous], 2)?;
        Ok(
            Self::linear(&gate_input, p.layer(layer, "read.identity_gate.weight")?)?
                .broadcast_add(p.layer(layer, "read.identity_gate.bias")?)?,
        )
    }

    fn read_latch_inputs(
        &self,
        p: &Params<'_>,
        layer: usize,
        u: &Tensor,
        mode: ReadIdentityLatch,
        gate_policy: LatchGates,
    ) -> Result<(Tensor, Tensor)> {
        let (batch, time, width) = u.dims3()?;
        let gates = gate_policy.apply(&self.read_latch_gate_logits(p, layer, u)?)?;
        let mut state = Tensor::zeros((batch, 1, width), u.dtype(), u.device())?;
        let zero = state.clone();
        let mut identities = Vec::with_capacity(time);
        for t in 0..time {
            identities.push(state.clone());
            let gate = gates.narrow(1, t, 1)?;
            if matches!(gate_policy, LatchGates::Hard) {
                // A hard action copies the existing state or current content
                // exactly, without renormalizing or blending either value.
                let no_capture = match mode {
                    ReadIdentityLatch::Held => &state,
                    ReadIdentityLatch::Local => &zero,
                };
                state = gate
                    .broadcast_as((batch, 1, width))?
                    .to_dtype(DType::U8)?
                    .where_cond(&u.narrow(1, t, 1)?, no_capture)?;
                continue;
            }
            let update = u.narrow(1, t, 1)?.broadcast_mul(&gate)?;
            state = match mode {
                ReadIdentityLatch::Held => state
                    .broadcast_mul(&gate.affine(-1.0, 1.0)?)?
                    .add(&update)?,
                ReadIdentityLatch::Local => update,
            };
        }
        Ok((Tensor::cat(&identities, 1)?, gates))
    }

    /// Diagnostic gate openings [batch,time] from the actual layer input.
    /// This does not modify gates or inject event annotations.
    pub fn read_identity_latch_gates(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        layer: usize,
    ) -> Result<Tensor> {
        Ok(candle_nn::ops::sigmoid(
            &self.read_identity_latch_gate_logits(ids, batch, time, layer)?,
        )?)
    }

    /// Gate preactivations [batch,time] from the same learned input and map as
    /// the actual reader, with a live gradient to its gate and preceding trunk.
    /// For stable training-only capture credit, form two-class logits [0,z]
    /// and use [`logits_cross_entropy`] with labels 0 (no capture) or 1
    /// (capture). Do not take logarithms of rounded sigmoid probabilities.
    /// This accessor neither supplies actions nor changes inference gates.
    pub fn read_identity_latch_gate_logits(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        layer: usize,
    ) -> Result<Tensor> {
        self.latch_gate_logits_with_policy(ids, batch, time, layer, LatchGates::Soft)
    }

    /// Actual binary actions [batch,time] for the hard-latch counterfactual.
    /// All preceding read layers also use hard gates in this call. This differs
    /// from thresholding the ordinary soft getter in a stack with earlier reads.
    /// A finite logit >= 0 captures, including a tie; a negative logit holds the
    /// prior Held identity or sets the Local identity to zero for the next row.
    pub fn read_identity_latch_hard_gates(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        layer: usize,
    ) -> Result<Tensor> {
        self.require_hard_read_identity_latch()?;
        LatchGates::Hard.apply(&self.latch_gate_logits_with_policy(
            ids,
            batch,
            time,
            layer,
            LatchGates::Hard,
        )?)
    }

    fn latch_gate_logits_with_policy(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        layer: usize,
        gates: LatchGates,
    ) -> Result<Tensor> {
        if self.read_identity_latch.is_none() {
            return Err(invalid("read identity latch is disabled"));
        }
        if layer >= self.config.layers() || self.config.layer_kind(layer) != 'a' {
            return Err(invalid(
                "read identity latch gate diagnostic needs a read layer",
            ));
        }
        let p = self.params()?;
        let x = self.embed_with(&p, ids, batch, time)?;
        let x = self.layer_range_bound(&p, x, 0..layer, &mut None, &mut None, gates)?;
        let u = self.norm(&p, &x, &layer_name(layer, "read_norm.weight"))?;
        Ok(self.read_latch_gate_logits(&p, layer, &u)?.squeeze(2)?)
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
        let gates = self.recurrence_gates(p, layer, &u)?;
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
                group: self.config.rotation_group,
                snap: self.transport,
            },
        )?;
        tap(
            capture,
            StackSite::RecurrenceOut(layer),
            || Ok(core.clone()),
        )?;
        Self::linear(&core, p.layer(layer, "rec.out.weight")?)
    }

    /// A recurrence's gates [batch, time, lanes (+ width with rotation)] from
    /// its normalized input `u`: the decay-gate logits, then the raw rotation
    /// quaternions.
    fn recurrence_gates(&self, p: &Params<'_>, layer: usize, u: &Tensor) -> Result<Tensor> {
        Ok(Self::linear(u, p.layer(layer, "rec.gate.weight")?)?
            .broadcast_add(p.layer(layer, "rec.gate.bias")?)?)
    }

    /// Logits [batch * time, vocabulary] for a batch of windows. Positions see
    /// only themselves and earlier positions of the same window.
    pub fn forward(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        let p = self.params()?;
        let hidden = self.hidden_hooked(&p, ids, batch, time, &mut None)?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Raw vocabulary logits with every learned read latch's gate replaced by
    /// the diagnostic action `logit >= 0`. The selected Held/Local recurrence,
    /// learned content, q/k/value maps and read scorer are otherwise unchanged.
    /// Like [`Self::forward`], this does not mix in a pointer head.
    ///
    /// This is a per-call floating-point research counterfactual, not a served
    /// model or a training STE. It changes no parameters or persistent options;
    /// [`Self::save`] still saves the original soft model. Reports must record
    /// this override separately; loading that model never enables hard gates.
    pub fn forward_hard_read_identity_latch(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        self.require_hard_read_identity_latch()?;
        let p = self.params()?;
        let x = self.embed_with(&p, ids, batch, time)?;
        let x = self.layer_range_bound(
            &p,
            x,
            0..self.config.layers(),
            &mut None,
            &mut None,
            LatchGates::Hard,
        )?;
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Actual gained read input for the fixed single-read `rra` diagnostic.
    /// This observes the preceding floating-point trunk; it is not an integer
    /// normalizer or a serving export.
    pub fn read_identity_latch_replay_input(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        self.require_identity_replay()?;
        let p = self.params()?;
        let x = self.embed_with(&p, ids, batch, time)?;
        let x = self.layer_range_hooked(&p, x, 0..2, &mut None)?;
        self.norm(&p, &x, &layer_name(2, "read_norm.weight"))
    }

    fn require_identity_replay(&self) -> Result<()> {
        if self.geometric_address.is_some() {
            return Err(invalid(
                "geometric addressing has no numerical identity or q/k replay view",
            ));
        }
        self.require_hard_read_identity_latch()?;
        if self.config.pattern != "rra" || self.config.pointer.is_some() {
            return Err(invalid("identity replay needs rra without a pointer head"));
        }
        Ok(())
    }

    fn validate_identity_replay(&self, prior: &Tensor, batch: usize, time: usize) -> Result<()> {
        self.require_identity_replay()?;
        if prior.dims3()? != (batch, time, self.config.width)
            || prior.dtype() != DType::F32
            || prior.device().location() != self.device.location()
            || prior
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .any(|x| !x.is_finite())
        {
            return Err(invalid(
                "identity replay needs finite matching prior vectors",
            ));
        }
        if prior
            .narrow(1, 0, 1)?
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .any(|x| *x != 0.)
        {
            return Err(invalid("identity replay starts at zero"));
        }
        Ok(())
    }

    /// Numerical-component diagnostic: caller supplies the prior gained
    /// identity reconstructed from its causal latch. The model does not certify
    /// caller causality. Current-role maps, values, trunk and scorer remain
    /// floating point. No override is persisted and no parameters are modified.
    pub fn forward_read_identity_latch_replay(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        prior: &Tensor,
    ) -> Result<Tensor> {
        self.validate_identity_replay(prior, batch, time)?;
        let p = self.params()?;
        let x = self.embed_with(&p, ids, batch, time)?;
        let x = self.layer_range_bound(
            &p,
            x,
            0..self.config.layers(),
            &mut None,
            &mut None,
            LatchGates::Replay(prior),
        )?;
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Observes exact-source mass under the same numerical replay as its
    /// forward. Labels select probabilities only and do not select identity.
    pub fn read_binding_masses_identity_latch_replay(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        binding: &ReadBindingTarget,
        prior: &Tensor,
    ) -> Result<Tensor> {
        self.validate_identity_replay(prior, batch, time)?;
        let p = self.params()?;
        Ok(self
            .hidden_with_binding_policy(&p, ids, batch, time, binding, LatchGates::Replay(prior))?
            .1)
    }

    fn validate_projection_replay(
        &self,
        query: &Tensor,
        key: &Tensor,
        batch: usize,
        time: usize,
    ) -> Result<()> {
        self.require_identity_replay()?;
        for projection in [query, key] {
            if projection.dims3()? != (batch, time, self.config.width)
                || projection.dtype() != DType::F32
                || projection.device().location() != self.device.location()
                || projection
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .any(|x| !x.is_finite())
            {
                return Err(invalid(
                    "projection replay needs finite matching q/k vectors",
                ));
            }
        }
        Ok(())
    }

    /// Per-call numerical q/k diagnostic for the fixed single-read rra stack.
    /// Caller supplies projected vectors, before splitting heads. Their causal
    /// construction is the caller's obligation. Values, scorer and surrounding
    /// model stay unchanged; no projection override enters saved model state.
    pub fn forward_read_projection_replay(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        query: &Tensor,
        key: &Tensor,
    ) -> Result<Tensor> {
        self.validate_projection_replay(query, key, batch, time)?;
        let p = self.params()?;
        let x = self.embed_with(&p, ids, batch, time)?;
        let x = self.layer_range_bound(
            &p,
            x,
            0..self.config.layers(),
            &mut None,
            &mut None,
            LatchGates::Projections(query, key),
        )?;
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// Observe source mass with the same supplied projections as the forward.
    /// Binding labels select measured probabilities and never supply q/k.
    pub fn read_binding_masses_projection_replay(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        binding: &ReadBindingTarget,
        query: &Tensor,
        key: &Tensor,
    ) -> Result<Tensor> {
        self.validate_projection_replay(query, key, batch, time)?;
        let p = self.params()?;
        Ok(self
            .hidden_with_binding_policy(
                &p,
                ids,
                batch,
                time,
                binding,
                LatchGates::Projections(query, key),
            )?
            .1)
    }

    fn require_hard_read_identity_latch(&self) -> Result<()> {
        if self.read_identity_latch.is_none() || self.served.is_some() {
            return Err(invalid(
                "hard read identity latch diagnostic needs an enabled float latch",
            ));
        }
        Ok(())
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
        if self.geometric_span.is_some() {
            let source = ReadSource {
                tokens: Some(&x),
                span_policy: SpanPolicy::Ordered,
                native_span: None,
                native_potential: None,
                event_control: None,
                context: None,
                native_reducer: None,
                native_trace: None,
                native_value_projection: None,
                learned_values: None,
                native_learned_values: None,
                no_read: NoReadSource::LegacyFloat,
                composition: None,
                integer_residual: None,
            };
            let hidden = self.layer_range_with_source(
                p,
                x.clone(),
                0..self.config.layers(),
                capture,
                &mut None,
                LatchGates::Soft,
                source,
            )?;
            return self.finish_hooked(p, hidden, capture);
        }
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
        x: Tensor,
        layers: std::ops::Range<usize>,
        capture: &mut Capture<'_>,
    ) -> Result<Tensor> {
        self.layer_range_bound(p, x, layers, capture, &mut None, LatchGates::Soft)
    }

    fn layer_range_bound(
        &self,
        p: &Params<'_>,
        x: Tensor,
        layers: std::ops::Range<usize>,
        capture: &mut Capture<'_>,
        binding: &mut Option<BindingCapture<'_>>,
        gates: LatchGates,
    ) -> Result<Tensor> {
        self.layer_range_with_source(p, x, layers, capture, binding, gates, ReadSource::default())
    }

    fn layer_range_with_source(
        &self,
        p: &Params<'_>,
        mut x: Tensor,
        layers: std::ops::Range<usize>,
        capture: &mut Capture<'_>,
        binding: &mut Option<BindingCapture<'_>>,
        gates: LatchGates,
        source: ReadSource<'_>,
    ) -> Result<Tensor> {
        for layer in layers {
            let mixed = match (self.config.arch, self.config.layer_kind(layer)) {
                (StackArch::Transformer, _) => self.attention(p, layer, &x, capture)?,
                (StackArch::Geometric, 'r') => self.recurrence(p, layer, &x, capture)?,
                (StackArch::Geometric, _) => {
                    if let Some((site, residual)) = source.integer_residual {
                        if site != layer || residual.dims() != x.dims() {
                            return Err(invalid(
                                "incremental geometric reader residual site/shape differs",
                            ));
                        }
                        residual.clone()
                    } else {
                        self.geometric_read_with_source(
                            p, layer, &x, capture, binding, gates, source,
                        )?
                    }
                }
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

    /// The pointer head's per-position outputs [rows, 2 * dim + 1] from final
    /// states [rows, width]: query, key and gate logit (`w_g . h + b_g`), by one
    /// matrix product of the three maps stacked.
    fn pointer_side(&self, p: &Params<'_>, hidden: &Tensor) -> Result<Tensor> {
        let pointer = self
            .config
            .pointer
            .ok_or_else(|| invalid("the model has no pointer head"))?;
        let weight = Tensor::cat(
            &[
                p.get("pointer.query.weight")?,
                p.get("pointer.key.weight")?,
                p.get("pointer.gate.weight")?,
            ],
            0,
        )?;
        let bias = Tensor::cat(
            &[
                &Tensor::zeros(2 * pointer.dim, DType::F32, &self.device)?,
                p.get("pointer.gate.bias")?,
            ],
            0,
        )?;
        Ok(hidden.matmul(&weight.t()?)?.broadcast_add(&bias)?)
    }

    /// The pointer's Lorentz scale `exp(pointer.log_beta)` as a one-element
    /// tensor (a zero for Dot, which has none): the mixture's third input,
    /// through which the scale receives its gradient.
    fn pointer_beta(&self, p: &Params<'_>) -> Result<Tensor> {
        let pointer = self
            .config
            .pointer
            .ok_or_else(|| invalid("the model has no pointer head"))?;
        match pointer.score {
            ReadScore::Dot => Ok(Tensor::zeros(1, DType::F32, &self.device)?),
            ReadScore::Lorentz => Ok(p.get("pointer.log_beta")?.exp()?),
            ReadScore::L2 => Err(invalid("the pointer head has no L2 score")),
        }
    }

    /// The mixture loss of a pointer model (see the module documentation): the
    /// weighted mean, or with no `weights` the mean, of `-log((1 - g)
    /// softmax(z)[y] + g p_copy(y))` over the targets.
    fn pointer_loss(
        &self,
        ids: &[u32],
        targets: &[u32],
        weights: Option<&[f32]>,
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        let p = self.params()?;
        let hidden = self.hidden_hooked(&p, ids, batch, time, &mut None)?;
        self.pointer_loss_from_hidden(&p, &hidden, ids, targets, weights, time)
    }

    fn pointer_loss_from_hidden(
        &self,
        p: &Params<'_>,
        hidden: &Tensor,
        ids: &[u32],
        targets: &[u32],
        weights: Option<&[f32]>,
        time: usize,
    ) -> Result<Tensor> {
        let pointer = self
            .config
            .pointer
            .ok_or_else(|| invalid("the model has no pointer head"))?;
        let logits = hidden.matmul(&p.head()?.t()?)?;
        let side = self.pointer_side(p, hidden)?;
        let beta = self.pointer_beta(p)?;
        Ok(logits.contiguous()?.apply_op3(
            &side.contiguous()?,
            &beta.contiguous()?,
            PointerMixture {
                time,
                dim: pointer.dim,
                score: pointer.score,
                select: pointer.select,
                route: pointer.route,
                ids: ids.to_vec(),
                targets: targets.to_vec(),
                weights: weights.map(<[f32]>::to_vec),
            },
        )?)
    }

    fn validate_binding(
        &self,
        batch: usize,
        time: usize,
        target: &ReadBindingTarget,
    ) -> Result<()> {
        if batch == 0
            || time == 0
            || time > self.config.context
            || batch
                .checked_mul(self.config.heads)
                .and_then(|n| n.checked_mul(time))
                .is_none_or(|n| n > u32::MAX as usize)
        {
            return Err(invalid(
                "read binding needs nonempty batch/time within context and u32 indices",
            ));
        }
        if self.config.arch != StackArch::Geometric
            || target.layer >= self.config.layers()
            || self.config.layer_kind(target.layer) != 'a'
            || target.head >= self.config.heads
        {
            return Err(invalid(
                "read binding needs a declared geometric read layer and head",
            ));
        }
        if self.config.select.is_some() {
            return Err(invalid("read binding requires full source admission"));
        }
        if target.rows.is_empty() {
            return Err(invalid("read binding needs at least one labelled query"));
        }
        let mut batches = BTreeSet::new();
        for row in &target.rows {
            if row.batch >= batch || row.query >= time || !batches.insert(row.batch) {
                return Err(invalid(
                    "read binding needs one in-range query per labelled batch item",
                ));
            }
            let mut sources = BTreeSet::new();
            if row.sources.is_empty()
                || row
                    .sources
                    .iter()
                    .any(|&source| source >= row.query || !sources.insert(source))
            {
                return Err(invalid(
                    "read binding sources must be unique, nonempty, exact past occurrences",
                ));
            }
        }
        Ok(())
    }

    fn hidden_with_binding(
        &self,
        p: &Params<'_>,
        ids: &[u32],
        batch: usize,
        time: usize,
        target: &ReadBindingTarget,
    ) -> Result<(Tensor, Tensor)> {
        self.hidden_with_binding_policy(p, ids, batch, time, target, LatchGates::Soft)
    }

    fn hidden_with_binding_policy(
        &self,
        p: &Params<'_>,
        ids: &[u32],
        batch: usize,
        time: usize,
        target: &ReadBindingTarget,
        gates: LatchGates,
    ) -> Result<(Tensor, Tensor)> {
        self.hidden_with_binding_span_policy(
            p,
            ids,
            batch,
            time,
            target,
            gates,
            SpanPolicy::Ordered,
        )
    }

    fn hidden_with_binding_span_policy(
        &self,
        p: &Params<'_>,
        ids: &[u32],
        batch: usize,
        time: usize,
        target: &ReadBindingTarget,
        gates: LatchGates,
        span_policy: SpanPolicy,
    ) -> Result<(Tensor, Tensor)> {
        self.validate_binding(batch, time, target)?;
        let x = self.embed_with(p, ids, batch, time)?;
        let mut binding = Some(BindingCapture {
            target,
            masses: None,
        });
        let x = self.layer_range_with_source(
            p,
            x.clone(),
            0..self.config.layers(),
            &mut None,
            &mut binding,
            gates,
            ReadSource {
                tokens: Some(&x),
                span_policy,
                native_span: None,
                native_potential: None,
                event_control: None,
                context: None,
                native_reducer: None,
                native_trace: None,
                native_value_projection: None,
                learned_values: None,
                native_learned_values: None,
                no_read: NoReadSource::LegacyFloat,
                composition: None,
                integer_residual: None,
            },
        )?;
        let hidden = self.finish_hooked(p, x, &mut None)?;
        let masses = binding
            .and_then(|binding| binding.masses)
            .ok_or_else(|| invalid("declared read binding layer was not evaluated"))?;
        Ok((hidden, masses))
    }

    /// The declared source-set mass at each labelled query, in row order.
    /// This diagnostic uses the actual read, including age and NoRead. Labels
    /// only select which probabilities are observed; they never alter logits.
    /// The same full-admission and causal-occurrence rules as the loss apply.
    pub fn read_binding_masses(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        binding: &ReadBindingTarget,
    ) -> Result<Tensor> {
        let p = self.params()?;
        Ok(self.hidden_with_binding(&p, ids, batch, time, binding)?.1)
    }

    /// The existing exact-source observer under the explicit last-token span
    /// control. Labels only select observed probabilities, never token actions
    /// or predicted controller transitions.
    pub fn read_binding_masses_geometric_span_last_token(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        binding: &ReadBindingTarget,
    ) -> Result<Tensor> {
        if self.geometric_span.is_none() {
            return Err(invalid("geometric span production is disabled"));
        }
        let p = self.params()?;
        Ok(self
            .hidden_with_binding_span_policy(
                &p,
                ids,
                batch,
                time,
                binding,
                LatchGates::Soft,
                SpanPolicy::LastToken,
            )?
            .1)
    }

    /// Exact source-set probabilities under the same per-call hard override as
    /// [`Self::forward_hard_read_identity_latch`]. Labels only select observed
    /// masses and do not enter the hard gate, hidden states or language logits.
    pub fn read_binding_masses_hard_read_identity_latch(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        binding: &ReadBindingTarget,
    ) -> Result<Tensor> {
        self.require_hard_read_identity_latch()?;
        let p = self.params()?;
        Ok(self
            .hidden_with_binding_policy(&p, ids, batch, time, binding, LatchGates::Hard)?
            .1)
    }

    /// Joint language and exact-source binding losses from one forward graph.
    /// `weights` has the same meaning as in [`Self::weighted_loss`]; `None`
    /// uses the ordinary mean language loss. Pointer models retain their
    /// actual mixture loss. Binding labels are teacher-only and never change
    /// the forward language logits, the parameter set, or saved model format.
    ///
    /// Binding is `mean(-log(sum_{source in label} attention[source]))` at
    /// the chosen layer/head/query. NoRead participates in the denominator.
    /// A zero/nonfinite mass is an explicit error, not a clipped zero-gradient
    /// objective. This initial API supervises at most one query per batch item.
    pub fn loss_with_binding(
        &self,
        ids: &[u32],
        targets: &[u32],
        weights: Option<&[f32]>,
        batch: usize,
        time: usize,
        binding: &ReadBindingTarget,
    ) -> Result<StackBindingLoss> {
        if targets.len() != ids.len()
            || targets
                .iter()
                .any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid(
                "read binding language loss needs one in-vocabulary target per input",
            ));
        }
        if let Some(weights) = weights {
            if weights.len() != ids.len()
                || weights.iter().any(|&w| !w.is_finite() || w < 0.0)
                || weights.iter().map(|&w| f64::from(w)).sum::<f64>() <= 0.0
            {
                return Err(invalid("read binding language weights must match inputs and be finite, nonnegative and not all zero"));
            }
        }
        let p = self.params()?;
        let (hidden, masses) = self.hidden_with_binding(&p, ids, batch, time, binding)?;
        if masses
            .to_vec1::<f32>()?
            .iter()
            .any(|&mass| !mass.is_finite() || mass <= 0.0)
        {
            return Err(invalid(
                "read binding positive-source mass is zero or nonfinite",
            ));
        }
        let language = if self.config.pointer.is_some() {
            self.pointer_loss_from_hidden(&p, &hidden, ids, targets, weights, time)?
        } else {
            hidden.matmul(&p.head()?.t()?)?.apply_op1(CrossEntropy {
                targets: targets.to_vec(),
                weights: weights.map(<[f32]>::to_vec),
            })?
        };
        let binding = masses.log()?.mean_all()?.neg()?;
        Ok(StackBindingLoss {
            language,
            binding,
            masses,
        })
    }

    /// Mean next-token negative log-likelihood (nats) over all targets. For a
    /// pointer model, the mixture's.
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
        if self.config.pointer.is_some() {
            return self.pointer_loss(ids, targets, None, batch, time);
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
        if self.config.pointer.is_some() {
            return self.pointer_loss(ids, targets, Some(weights), batch, time);
        }
        let logits = self.forward(ids, batch, time)?;
        Ok(logits.apply_op1(CrossEntropy {
            targets: targets.to_vec(),
            weights: Some(weights.to_vec()),
        })?)
    }

    /// Per-target negative log-likelihoods (nats), without a backward graph.
    /// For a pointer model, the mixture's.
    pub fn target_nll(
        &self,
        ids: &[u32],
        targets: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<Vec<f64>> {
        Ok(self.score_targets(ids, targets, None, batch, time)?.nll)
    }

    /// [`target_nll`](Self::target_nll), and for a pointer model what the head
    /// did at each position. With `weights`, a pointer model scores only the
    /// positions of nonzero weight (the others read `nll` 0 and no statistics);
    /// a model without a pointer scores every position.
    pub fn score_targets(
        &self,
        ids: &[u32],
        targets: &[u32],
        weights: Option<&[f32]>,
        batch: usize,
        time: usize,
    ) -> Result<TargetScores> {
        if targets.len() != ids.len() || weights.is_some_and(|w| w.len() != ids.len()) {
            return Err(invalid("one target and one weight per input id"));
        }
        if targets
            .iter()
            .any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid("target id outside the vocabulary"));
        }
        let Some(pointer) = self.config.pointer else {
            return Ok(TargetScores {
                nll: row_nll(&self.forward(ids, batch, time)?.detach(), targets)?,
                pointer: None,
            });
        };
        let p = self.params()?;
        let hidden = self
            .hidden_hooked(&p, ids, batch, time, &mut None)?
            .detach();
        let logits = hidden.matmul(&p.head()?.t()?)?;
        let side = self.pointer_side(&p, &hidden)?;
        let beta = one_value(&self.pointer_beta(&p)?)?;
        let (logits, side) = (
            logits.flatten_all()?.to_vec1::<f32>()?,
            side.flatten_all()?.to_vec1::<f32>()?,
        );
        let vocabulary = self.config.vocab_size;
        let op = PointerMixture {
            time,
            dim: pointer.dim,
            score: pointer.score,
            select: pointer.select,
            route: pointer.route,
            ids: ids.to_vec(),
            targets: targets.to_vec(),
            weights: weights.map(<[f32]>::to_vec),
        };
        let rows: Vec<(f64, Option<PointerRowStats>)> = (0..ids.len())
            .into_par_iter()
            .map(|n| -> candle_core::Result<(f64, Option<PointerRowStats>)> {
                if op.weight(n) == 0.0 {
                    return Ok((0.0, None));
                }
                let row = op.evaluate(
                    &logits[n * vocabulary..(n + 1) * vocabulary],
                    &side,
                    beta,
                    n,
                )?;
                let first = n - n % time;
                let target = targets[n];
                let mut best = 0;
                for (j, &a) in row.attention.iter().enumerate() {
                    if a > row.attention[best] {
                        best = j;
                    }
                }
                let stats = PointerRowStats {
                    gate: row.gate,
                    copy_mass: row.copy,
                    hit: ids[first + best] == target,
                    reachable: row
                        .attention
                        .iter()
                        .zip(&ids[first..])
                        .any(|(&a, &id)| a > 0.0 && id == target),
                };
                Ok((-row.log_mixture, Some(stats)))
            })
            .collect::<candle_core::Result<_>>()?;
        Ok(TargetScores {
            nll: rows.iter().map(|row| row.0).collect(),
            pointer: Some(rows.into_iter().map(|row| row.1).collect()),
        })
    }

    /// Scores of the token after `ids` (one window, at most the context):
    /// the raw logits of its last position, or for a pointer model the log of
    /// the mixture `(1 - g) softmax(z) + g p_copy` there. The greedy token is
    /// the highest score, the lowest id on a tie.
    pub fn next_scores(&self, ids: &[u32]) -> Result<Vec<f32>> {
        let time = ids.len();
        if time == 0 {
            return Err(invalid("next_scores needs at least one input id"));
        }
        let Some(pointer) = self.config.pointer else {
            let logits = self.forward(ids, 1, time)?.detach();
            return Ok(logits.get(time - 1)?.to_vec1::<f32>()?);
        };
        let p = self.params()?;
        let hidden = self.hidden_hooked(&p, ids, 1, time, &mut None)?.detach();
        let logits = hidden
            .narrow(0, time - 1, 1)?
            .matmul(&p.head()?.t()?)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let side = self
            .pointer_side(&p, &hidden)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let rule = PointerRule {
            dim: pointer.dim,
            score: pointer.score,
            select: pointer.select,
            beta: one_value(&self.pointer_beta(&p)?)?,
            route: pointer.route,
        };
        let attention = pointer_attention(&side, 0, time - 1, &rule, ids)?;
        let gate = sigmoid_f64(f64::from(
            side[(time - 1) * (2 * pointer.dim + 1) + 2 * pointer.dim],
        ));
        let lse = row_log_sum_exp(&logits);
        let mut mixture: Vec<f64> = logits
            .iter()
            .map(|&z| (1.0 - gate) * (f64::from(z) - lse).exp())
            .collect();
        for (&a, &id) in attention.iter().zip(ids) {
            mixture[id as usize] += gate * a;
        }
        Ok(mixture
            .into_iter()
            .map(|probability| probability.max(f64::MIN_POSITIVE).ln() as f32)
            .collect())
    }

    /// Per-target negative log-likelihoods (nats), using an explicit output head tensor
    /// without mutating or tying the model's input embeddings.
    pub fn target_nll_with_head(
        &self,
        ids: &[u32],
        targets: &[u32],
        head: &Tensor,
        batch: usize,
        time: usize,
    ) -> Result<Vec<f64>> {
        if targets.len() != ids.len() {
            return Err(invalid("one target per input id"));
        }
        if targets
            .iter()
            .any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid("target id outside the vocabulary"));
        }
        let dims = head.dims();
        if dims.len() != 2 || dims[0] != self.config.vocab_size || dims[1] != self.config.width {
            return Err(invalid(format!(
                "explicit head shape {:?} must match [vocab_size={}, width={}]",
                dims, self.config.vocab_size, self.config.width
            )));
        }
        let hidden = self.hidden(ids, batch, time)?;
        let logits = hidden.matmul(&head.t()?)?.detach();
        row_nll(&logits, targets)
    }

    /// Save the float variables (`model.safetensors`) and the configuration
    /// (`config.json`). In served mode `config.json` also records the served
    /// representation (`served_representation`, [`SavedServedRepresentation`]):
    /// the weights are the float variables whose export through that codec is
    /// what the forward pass read, as after quantization-aware training. A
    /// float save writes the configuration alone, byte for byte as before the
    /// record existed. With a transport snap set, [`TRANSPORT_RECORD`]
    /// beside them records it ([`Self::saved_transport_snap`]); a save
    /// without one removes a stale record, so the directory never claims a
    /// snap its weights were not saved with. [`Self::load`] restores the
    /// transport snap from its record, so a loaded model's forward pass is the
    /// one that was trained. It does not restore the served representation;
    /// its callers read [`Self::saved_served_representation`].
    /// Opt-in read identity carry also writes [`READ_IDENTITY_CARRY_RECORD`]
    /// and pins its digest in config.json; a default save removes stale carry
    /// metadata and retains the legacy config byte format.
    pub fn save(&self, directory: &Path) -> Result<()> {
        if let Some(span) = &self.geometric_span {
            span.validate(self.config.width)?;
            if self.geometric_address.is_none() || self.read_identity_latch.is_some() {
                return Err(invalid(
                    "geometric span producer and address/latch inventory differ",
                ));
            }
        }
        if let Some(config) = &self.geometric_address {
            self.validate_geometric_address(config)?;
        }
        fs::create_dir_all(directory)?;
        let tensors: std::collections::HashMap<String, Tensor> = self
            .variables
            .iter()
            .map(|(name, var)| (name.clone(), var.as_tensor().clone()))
            .collect();
        candle_core::safetensors::save(&tensors, directory.join("model.safetensors"))?;
        let mut config = match &self.served {
            None => serde_json::to_vec_pretty(&self.config)?,
            Some(state) => serde_json::to_vec_pretty(&ServedConfigFile {
                config: &self.config,
                served_representation: SavedServedRepresentation {
                    codec: state.codec.name().to_owned(),
                },
            })?,
        };
        let carry_path = directory.join(READ_IDENTITY_CARRY_RECORD);
        if self.read_identity_carry {
            let record = ReadIdentityCarryRecord {
                schema: READ_IDENTITY_CARRY_SCHEMA.to_owned(),
                operation: READ_IDENTITY_CARRY_OPERATION.to_owned(),
                config_sha256: hex::encode(Sha256::digest(serde_json::to_vec_pretty(
                    &self.config,
                )?)),
                model_sha256: hex::encode(Sha256::digest(fs::read(
                    directory.join("model.safetensors"),
                )?)),
            };
            let bytes = serde_json::to_vec_pretty(&record)?;
            let marker = ReadIdentityCarryMarker {
                schema: READ_IDENTITY_CARRY_SCHEMA.to_owned(),
                record_sha256: hex::encode(Sha256::digest(&bytes)),
            };
            let mut with_marker: serde_json::Value = serde_json::from_slice(&config)?;
            with_marker["read_identity_carry"] = serde_json::to_value(marker)?;
            config = serde_json::to_vec_pretty(&with_marker)?;
            fs::write(&carry_path, bytes)?;
        } else {
            match fs::remove_file(&carry_path) {
                Ok(()) => (),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                Err(error) => return Err(error.into()),
            }
        }
        let latch_path = directory.join(READ_IDENTITY_LATCH_RECORD);
        if let Some(mode) = self.read_identity_latch {
            let record = ReadIdentityLatchRecord {
                schema: READ_IDENTITY_LATCH_SCHEMA.to_owned(),
                operation: if self.geometric_address.is_some() {
                    READ_IDENTITY_LATCH_ADDRESS_OPERATION
                } else {
                    READ_IDENTITY_LATCH_OPERATION
                }
                .to_owned(),
                mode,
                config_sha256: hex::encode(Sha256::digest(serde_json::to_vec_pretty(
                    &self.config,
                )?)),
                model_sha256: hex::encode(Sha256::digest(fs::read(
                    directory.join("model.safetensors"),
                )?)),
            };
            let bytes = serde_json::to_vec_pretty(&record)?;
            let marker = ReadIdentityCarryMarker {
                schema: READ_IDENTITY_LATCH_SCHEMA.to_owned(),
                record_sha256: hex::encode(Sha256::digest(&bytes)),
            };
            let mut with_marker: serde_json::Value = serde_json::from_slice(&config)?;
            with_marker["read_identity_latch"] = serde_json::to_value(marker)?;
            config = serde_json::to_vec_pretty(&with_marker)?;
            fs::write(latch_path, bytes)?;
        } else {
            match fs::remove_file(latch_path) {
                Ok(()) => (),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                Err(error) => return Err(error.into()),
            }
        }
        let address_path = directory.join(GEOMETRIC_ADDRESS_RECORD);
        if let Some(address) = &self.geometric_address {
            let record = GeometricAddressRecord {
                schema: GEOMETRIC_ADDRESS_SCHEMA.to_owned(),
                operation: if self.geometric_span.is_some() {
                    GEOMETRIC_ADDRESS_SPAN_OPERATION
                } else {
                    GEOMETRIC_ADDRESS_OPERATION
                }
                .to_owned(),
                address: address.clone(),
                config_sha256: hex::encode(Sha256::digest(serde_json::to_vec_pretty(
                    &self.config,
                )?)),
                model_sha256: hex::encode(Sha256::digest(fs::read(
                    directory.join("model.safetensors"),
                )?)),
            };
            let bytes = serde_json::to_vec_pretty(&record)?;
            let marker = ReadIdentityCarryMarker {
                schema: GEOMETRIC_ADDRESS_SCHEMA.to_owned(),
                record_sha256: hex::encode(Sha256::digest(&bytes)),
            };
            let mut with_marker: serde_json::Value = serde_json::from_slice(&config)?;
            with_marker["geometric_address"] = serde_json::to_value(marker)?;
            config = serde_json::to_vec_pretty(&with_marker)?;
            fs::write(address_path, bytes)?;
        } else {
            match fs::remove_file(address_path) {
                Ok(()) => (),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                Err(error) => return Err(error.into()),
            }
        }
        let span_path = directory.join(GEOMETRIC_SPAN_RECORD);
        if let Some(span) = &self.geometric_span {
            let record = GeometricSpanRecord {
                schema: GEOMETRIC_SPAN_SCHEMA.to_owned(),
                operation: GEOMETRIC_SPAN_OPERATION.to_owned(),
                span: span.clone(),
                config_sha256: hex::encode(Sha256::digest(serde_json::to_vec_pretty(
                    &self.config,
                )?)),
                model_sha256: hex::encode(Sha256::digest(fs::read(
                    directory.join("model.safetensors"),
                )?)),
            };
            let bytes = serde_json::to_vec_pretty(&record)?;
            let marker = ReadIdentityCarryMarker {
                schema: GEOMETRIC_SPAN_SCHEMA.to_owned(),
                record_sha256: hex::encode(Sha256::digest(&bytes)),
            };
            let mut with_marker: serde_json::Value = serde_json::from_slice(&config)?;
            with_marker["geometric_span"] = serde_json::to_value(marker)?;
            config = serde_json::to_vec_pretty(&with_marker)?;
            fs::write(span_path, bytes)?;
        } else {
            match fs::remove_file(span_path) {
                Ok(()) => (),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                Err(error) => return Err(error.into()),
            }
        }
        fs::write(directory.join("config.json"), config)?;
        let record = directory.join(TRANSPORT_RECORD);
        match self.transport {
            Some(snap) => fs::write(&record, serde_json::to_vec_pretty(&snap.file_record())?)?,
            None => match fs::remove_file(&record) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            },
        }
        Ok(())
    }

    /// The served representation that `directory`'s `config.json` records:
    /// `Some` for a model saved in served mode ([`Self::save`]), `None` for
    /// one saved in float, including every model saved before the record
    /// existed.
    pub fn saved_served_representation(
        directory: &Path,
    ) -> Result<Option<SavedServedRepresentation>> {
        #[derive(Deserialize)]
        struct Record {
            #[serde(default)]
            served_representation: Option<SavedServedRepresentation>,
        }
        let record: Record = serde_json::from_slice(&fs::read(directory.join("config.json"))?)?;
        Ok(record.served_representation)
    }

    /// The transport snap that `directory`'s [`TRANSPORT_RECORD`] records:
    /// `Some` for a model saved with a snap set ([`Self::save`]), `None` for
    /// one saved without, including every model saved before the record
    /// existed. A record whose roots differ from this build's is refused.
    pub fn saved_transport_snap(directory: &Path) -> Result<Option<TransportSnap>> {
        let bytes = match fs::read(directory.join(TRANSPORT_RECORD)) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let record: TransportFileRecord = serde_json::from_slice(&bytes)?;
        let snap = record.snap;
        if record.schema != TRANSPORT_RECORD_SCHEMA
            || record.roots != snap.roots().len()
            || record.roots_sha256 != snap.roots_sha256()
        {
            return Err(invalid(format!(
                "{} is not a {TRANSPORT_RECORD_SCHEMA} record of this build's {} roots",
                directory.join(TRANSPORT_RECORD).display(),
                snap.name()
            )));
        }
        Ok(Some(snap))
    }

    /// Read and verify the opt-in carry record. Legacy directories without a
    /// marker or sidecar mean false. A declared-but-missing, undeclared-extra,
    /// stale or mismatched record is refused, including unknown operations.
    /// The config marker binds exact metadata bytes; the record binds this
    /// model's float weights and canonical StackConfig. These are integrity
    /// checks, not authentication of an untrusted directory.
    pub fn saved_read_identity_carry(directory: &Path) -> Result<bool> {
        #[derive(Deserialize)]
        struct ConfigMarker {
            #[serde(default)]
            read_identity_carry: Option<ReadIdentityCarryMarker>,
        }
        let config_bytes = fs::read(directory.join("config.json"))?;
        let marker: ConfigMarker = serde_json::from_slice(&config_bytes)?;
        let bytes = match fs::read(directory.join(READ_IDENTITY_CARRY_RECORD)) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let (marker, bytes) = match (marker.read_identity_carry, bytes) {
            (None, None) => return Ok(false),
            (Some(marker), Some(bytes)) => (marker, bytes),
            _ => {
                return Err(invalid(
                    "read identity carry marker and record presence differ",
                ))
            }
        };
        let record: ReadIdentityCarryRecord = serde_json::from_slice(&bytes)?;
        let config: StackConfig = serde_json::from_slice(&config_bytes)?;
        if marker.schema != READ_IDENTITY_CARRY_SCHEMA
            || record.schema != READ_IDENTITY_CARRY_SCHEMA
            || record.operation != READ_IDENTITY_CARRY_OPERATION
            || marker.record_sha256 != hex::encode(Sha256::digest(&bytes))
            || record.config_sha256
                != hex::encode(Sha256::digest(serde_json::to_vec_pretty(&config)?))
            || record.model_sha256
                != hex::encode(Sha256::digest(fs::read(
                    directory.join("model.safetensors"),
                )?))
            || Self::saved_served_representation(directory)?.is_some()
        {
            return Err(invalid(
                "saved read identity carry metadata, operation or model binding differs",
            ));
        }
        Ok(true)
    }

    /// Verify the optional learned-latch record and return its mode. The
    /// marker pins exact metadata bytes; the sidecar binds operation, mode,
    /// canonical configuration and exact saved weights including the new maps.
    pub fn saved_read_identity_latch(directory: &Path) -> Result<Option<ReadIdentityLatch>> {
        #[derive(Deserialize)]
        struct ConfigMarker {
            #[serde(default)]
            read_identity_latch: Option<ReadIdentityCarryMarker>,
            #[serde(default)]
            geometric_address: Option<ReadIdentityCarryMarker>,
        }
        let config_bytes = fs::read(directory.join("config.json"))?;
        let marker: ConfigMarker = serde_json::from_slice(&config_bytes)?;
        let bytes = match fs::read(directory.join(READ_IDENTITY_LATCH_RECORD)) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let has_address_marker = marker.geometric_address.is_some();
        let (marker, bytes) = match (marker.read_identity_latch, bytes) {
            (None, None) => return Ok(None),
            (Some(marker), Some(bytes)) => (marker, bytes),
            _ => {
                return Err(invalid(
                    "read identity latch marker and record presence differ",
                ))
            }
        };
        let record: ReadIdentityLatchRecord = serde_json::from_slice(&bytes)?;
        let config: StackConfig = serde_json::from_slice(&config_bytes)?;
        let expected_operation = if has_address_marker {
            READ_IDENTITY_LATCH_ADDRESS_OPERATION
        } else {
            READ_IDENTITY_LATCH_OPERATION
        };
        if marker.schema != READ_IDENTITY_LATCH_SCHEMA
            || record.schema != READ_IDENTITY_LATCH_SCHEMA
            || record.operation != expected_operation
            || marker.record_sha256 != hex::encode(Sha256::digest(&bytes))
            || record.config_sha256
                != hex::encode(Sha256::digest(serde_json::to_vec_pretty(&config)?))
            || record.model_sha256
                != hex::encode(Sha256::digest(fs::read(
                    directory.join("model.safetensors"),
                )?))
            || Self::saved_served_representation(directory)?.is_some()
            || Self::saved_read_identity_carry(directory)?
            || config.arch != StackArch::Geometric
            || !config.pattern.contains('a')
        {
            return Err(invalid(
                "saved read identity latch operation, model binding or configuration differs",
            ));
        }
        Ok(Some(record.mode))
    }

    /// Verify a geometric-address sidecar against its config marker, exact
    /// model bytes, fixed codebook/root order and the Held-latch contract.
    /// No marker and no record is the legacy projected reader. One without
    /// the other is an error; these are integrity checks, not authentication.
    pub fn saved_geometric_address(directory: &Path) -> Result<Option<GeometricAddressConfig>> {
        #[derive(Deserialize)]
        struct ConfigMarker {
            #[serde(default)]
            geometric_address: Option<ReadIdentityCarryMarker>,
        }
        let config_bytes = fs::read(directory.join("config.json"))?;
        let marker: ConfigMarker = serde_json::from_slice(&config_bytes)?;
        let bytes = match fs::read(directory.join(GEOMETRIC_ADDRESS_RECORD)) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let (marker, bytes) = match (marker.geometric_address, bytes) {
            (None, None) => return Ok(None),
            (Some(marker), Some(bytes)) => (marker, bytes),
            _ => {
                return Err(invalid(
                    "geometric address marker and record presence differ",
                ))
            }
        };
        let record: GeometricAddressRecord = serde_json::from_slice(&bytes)?;
        let config: StackConfig = serde_json::from_slice(&config_bytes)?;
        config.validate()?;
        record.address.validate(config.width, config.heads)?;
        let span = Self::saved_geometric_span(directory)?;
        let expected_operation = if span.is_some() {
            GEOMETRIC_ADDRESS_SPAN_OPERATION
        } else {
            GEOMETRIC_ADDRESS_OPERATION
        };
        let latch = Self::saved_read_identity_latch(directory)?;
        if marker.schema != GEOMETRIC_ADDRESS_SCHEMA
            || record.schema != GEOMETRIC_ADDRESS_SCHEMA
            || record.operation != expected_operation
            || marker.record_sha256 != hex::encode(Sha256::digest(&bytes))
            || record.config_sha256
                != hex::encode(Sha256::digest(serde_json::to_vec_pretty(&config)?))
            || record.model_sha256
                != hex::encode(Sha256::digest(fs::read(
                    directory.join("model.safetensors"),
                )?))
            || config.arch != StackArch::Geometric
            || config.pattern != "rra"
            || config.select.is_some()
            || config.pointer.is_some()
            || !matches!(
                (span, latch),
                (None, Some(ReadIdentityLatch::Held)) | (Some(_), None)
            )
        {
            return Err(invalid(
                "saved geometric address operation, parameters or model binding differs",
            ));
        }
        Ok(Some(record.address))
    }

    /// Verify the distinct ordered-span producer and its exact parameter and
    /// configuration binding. A scalar latch, missing address mode, orphaned
    /// sidecar, unknown operation or changed root/action ordering fails closed.
    pub fn saved_geometric_span(directory: &Path) -> Result<Option<GeometricSpanConfig>> {
        #[derive(Deserialize)]
        struct ConfigMarker {
            #[serde(default)]
            geometric_span: Option<ReadIdentityCarryMarker>,
            #[serde(default)]
            geometric_address: Option<ReadIdentityCarryMarker>,
        }
        let config_bytes = fs::read(directory.join("config.json"))?;
        let marker: ConfigMarker = serde_json::from_slice(&config_bytes)?;
        let has_address = marker.geometric_address.is_some();
        let bytes = match fs::read(directory.join(GEOMETRIC_SPAN_RECORD)) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let (marker, bytes) = match (marker.geometric_span, bytes) {
            (None, None) => return Ok(None),
            (Some(marker), Some(bytes)) => (marker, bytes),
            _ => return Err(invalid("geometric span marker and record presence differ")),
        };
        let record: GeometricSpanRecord = serde_json::from_slice(&bytes)?;
        let config: StackConfig = serde_json::from_slice(&config_bytes)?;
        config.validate()?;
        record.span.validate(config.width)?;
        if marker.schema != GEOMETRIC_SPAN_SCHEMA
            || record.schema != GEOMETRIC_SPAN_SCHEMA
            || record.operation != GEOMETRIC_SPAN_OPERATION
            || !has_address
            || marker.record_sha256 != hex::encode(Sha256::digest(&bytes))
            || record.config_sha256
                != hex::encode(Sha256::digest(serde_json::to_vec_pretty(&config)?))
            || record.model_sha256
                != hex::encode(Sha256::digest(fs::read(
                    directory.join("model.safetensors"),
                )?))
            || config.arch != StackArch::Geometric
            || config.pattern != "rra"
            || config.select.is_some()
            || config.pointer.is_some()
            || Self::saved_read_identity_latch(directory)?.is_some()
            || Self::saved_read_identity_carry(directory)?
            || Self::saved_served_representation(directory)?.is_some()
        {
            return Err(invalid(
                "saved geometric span operation, model binding or configuration differs",
            ));
        }
        Ok(Some(record.span))
    }

    /// Load a saved model. A directory whose [`TRANSPORT_RECORD`] records a
    /// transport snap loads with it set ([`Self::saved_transport_snap`] checks
    /// its roots; [`Self::set_transport_snap`] checks the configuration), so
    /// evaluation and export see the model that was trained. Training modes
    /// then set the snap their arguments ask for; [`Self::with_unsnapped_transport`]
    /// gives the free-transport view explicitly. The served representation is
    /// not restored.
    pub fn load(directory: &Path, device: &Device) -> Result<Self> {
        let config: StackConfig =
            serde_json::from_slice(&fs::read(directory.join("config.json"))?)?;
        config.validate()?;
        let read_identity_latch = Self::saved_read_identity_latch(directory)?;
        let geometric_span = Self::saved_geometric_span(directory)?;
        let geometric_address = Self::saved_geometric_address(directory)?;
        if geometric_address.is_some() && !device.is_cpu() {
            return Err(invalid("geometric address models currently require CPU"));
        }
        let tensors = candle_core::safetensors::load(directory.join("model.safetensors"), device)?;
        let mut shapes = config.shapes();
        if read_identity_latch.is_some() {
            shapes.extend(read_identity_latch_shapes(&config));
        }
        if geometric_address.is_some() {
            remove_address_projection_parameters(&mut shapes);
            for (suffix, shape) in geometric_address::parameter_shapes(config.width, config.heads)?
            {
                shapes.insert(layer_name(2, &suffix), shape);
            }
        }
        if geometric_span.is_some() {
            if geometric_address.is_none() {
                return Err(invalid("geometric span lacks geometric address mode"));
            }
            shapes.extend(geometric_span_shapes(&config));
        }
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
        let mut model = Self {
            config,
            variables,
            device: device.clone(),
            served: None,
            transport: None,
            read_identity_carry: false,
            read_identity_latch,
            geometric_address,
            geometric_span,
        };
        model.set_transport_snap(Self::saved_transport_snap(directory)?)?;
        model.set_read_identity_carry(Self::saved_read_identity_carry(directory)?)?;
        Ok(model)
    }
}

impl StackModel {
    /// Evaluation-only diagnostic (roadmap S1.0b): next-token logits
    /// `[batch * time, vocabulary]` with every transport quaternion replaced,
    /// before its scaling by lambda, by the nearest of `snap` (for example the
    /// 120 unit icosians of 2I). The recurrences run through their composed
    /// Candle reference; with `snap = None` this equals the fused forward
    /// without a transport snap within float tolerance, and with the snap's
    /// roots the fused forward with it ([`Self::set_transport_snap`]). The
    /// model's own snap, if set, is ignored: `snap` alone decides. Not a
    /// training or serving path.
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
        if snap.is_some() && self.config.rotation_group != RotationGroup::Quaternion {
            return Err(invalid("transport snapping needs quaternion transport"));
        }
        self.composed_logits(
            ids,
            batch,
            time,
            match snap {
                None => ComposedTransport::Free,
                Some(roots) => ComposedTransport::Snapped(roots),
            },
        )
    }

    /// Logits with every recurrence composed from Candle operations, its
    /// transport as `transport` says. Geometric stacks only.
    fn composed_logits(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        transport: ComposedTransport<'_>,
    ) -> Result<Tensor> {
        let p = self.params()?;
        let mut x = self.embed_with(&p, ids, batch, time)?;
        let tokens = x.clone();
        for layer in 0..self.config.layers() {
            let mixed = match self.config.layer_kind(layer) {
                'r' => self.composed_recurrence(&p, layer, &x, transport)?,
                _ => self.geometric_read_with_source(
                    &p,
                    layer,
                    &x,
                    &mut None,
                    &mut None,
                    LatchGates::Soft,
                    ReadSource {
                        tokens: Some(&tokens),
                        span_policy: SpanPolicy::Ordered,
                        native_span: None,
                        native_potential: None,
                        event_control: None,
                        context: None,
                        native_reducer: None,
                        native_trace: None,
                        native_value_projection: None,
                        learned_values: None,
                        native_learned_values: None,
                        no_read: NoReadSource::LegacyFloat,
                        composition: None,
                        integer_residual: None,
                    },
                )?,
            };
            x = x.add(&mixed)?;
            x = x.add(&self.mlp(&p, layer, &x, &mut None)?)?;
        }
        let hidden = self.finish_hooked(&p, x, &mut None)?;
        Ok(hidden.matmul(&p.head()?.t()?)?)
    }

    /// The recurrence mixer composed from Candle operations: the reference
    /// for the fused core, its unit transport quaternions as `transport` says.
    fn composed_recurrence(
        &self,
        p: &Params<'_>,
        layer: usize,
        x: &Tensor,
        transport: ComposedTransport<'_>,
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
            .to_dtype(DType::F64)?
            .neg()?
            .exp()?
            .affine(1.0, 1.0)?
            .log()?
            .neg()?
            .to_dtype(DType::F32)?;
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
            let raw = match self.config.rotation_group {
                RotationGroup::Quaternion => raw,
                // The U(1) control: j and k zeroed before normalization.
                RotationGroup::U1 => {
                    raw.broadcast_mul(&Tensor::new(&[1f32, 1.0, 0.0, 0.0], &self.device)?)?
                }
            };
            let norm = raw.sqr()?.sum_keepdim(3)?.affine(1.0, 1e-6)?.sqrt()?;
            let unit = raw.broadcast_div(&norm)?;
            let unit = match transport {
                ComposedTransport::Free => unit,
                ComposedTransport::Snapped(roots) => snap_to_roots(&unit, roots)?,
                #[cfg(test)]
                ComposedTransport::StraightThrough(roots) => {
                    let snapped = snap_to_roots(&unit, roots)?;
                    unit.add(&snapped.sub(&unit)?.detach())?
                }
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

/// How the composed recurrence treats its unit transport quaternions.
#[derive(Clone, Copy, Debug)]
enum ComposedTransport<'a> {
    /// As they are.
    Free,
    /// Each replaced by the nearest of the roots, a value without gradient
    /// ([`StackModel::logits_with_transport`]).
    Snapped(&'a [[f32; 4]]),
    /// `u + (snap(u) - u).detach()`: the nearest root's value with the unit's
    /// gradient, the straight-through reference for the fused core's snap.
    #[cfg(test)]
    StraightThrough(&'a [[f32; 4]]),
}

/// Each quaternion of a `[.., 4]` tensor replaced by the root with the
/// largest dot product: the nearest root, as every root is a unit
/// ([`nearest_root`], which the fused recurrence core's snap uses too).
fn snap_to_roots(unit: &Tensor, roots: &[[f32; 4]]) -> Result<Tensor> {
    if roots.is_empty() {
        return Err(invalid("snapping needs at least one root"));
    }
    let shape = unit.shape().clone();
    let values = unit.flatten_all()?.to_vec1::<f32>()?;
    let snapped: Vec<[f32; 4]> = values
        .par_chunks(4)
        .map(|q| roots[nearest_root([q[0], q[1], q[2], q[3]], roots)])
        .collect();
    Ok(Tensor::from_vec(
        snapped.into_iter().flatten().collect::<Vec<f32>>(),
        shape,
        unit.device(),
    )?)
}

/// The index of the root with the largest dot product with `q`, the first
/// on a tie (and 0 if every dot product is NaN). Every dot product is
/// `q[0] r[0] + q[1] r[1] + q[2] r[2] + q[3] r[3]` summed left to right, so
/// equal inputs select the same root in every caller. `roots` is not empty.
#[inline]
fn nearest_root(q: [f32; 4], roots: &[[f32; 4]]) -> usize {
    let mut best = 0;
    let mut best_dot = f32::NEG_INFINITY;
    for (index, root) in roots.iter().enumerate() {
        let dot = q[0] * root[0] + q[1] * root[1] + q[2] * root[2] + q[3] * root[3];
        if dot > best_dot {
            best_dot = dot;
            best = index;
        }
    }
    best
}

/// `raw / sqrt(|raw|^2 + 1e-6)` and that norm: a recurrence's unit transport
/// quaternion from its raw rotation logits.
#[inline]
fn unit_quaternion(raw: [f32; 4]) -> ([f32; 4], f32) {
    let norm = (raw.iter().map(|v| v * v).sum::<f32>() + 1e-6).sqrt();
    (
        [raw[0] / norm, raw[1] / norm, raw[2] / norm, raw[3] / norm],
        norm,
    )
}

// ---------------------------------------------------------------------------
// The trained-in transport snap.

/// The file beside `config.json` in which [`StackModel::save`] records a
/// model's transport snap.
pub const TRANSPORT_RECORD: &str = "transport.json";
/// The schema of [`TRANSPORT_RECORD`].
pub const TRANSPORT_RECORD_SCHEMA: &str = "uor-r4.stack-transport/1";

/// The opt-in read identity carry's operation and model binding, also pinned
/// by its exact SHA-256 in config.json. Absent in legacy/default saves.
pub const READ_IDENTITY_CARRY_RECORD: &str = "read-identity-carry.json";
const READ_IDENTITY_CARRY_SCHEMA: &str = "uor-r4.stack-read-identity-carry/1";
const READ_IDENTITY_CARRY_OPERATION: &str =
    "q_key=normalized_input[t-1];t0=zeros;value=normalized_input[t];null=normalized_input[t];age=current;support=unchanged";

pub const READ_IDENTITY_LATCH_RECORD: &str = "read-identity-latch.json";
const READ_IDENTITY_LATCH_SCHEMA: &str = "uor-r4.stack-read-identity-latch/1";
const READ_IDENTITY_LATCH_OPERATION: &str =
    "g=sigmoid(W[u,prev_u]+b);held_h=(1-g)*prev_h+g*u;local_h=g*u;qk=current_role+identity(prev_h);initial=zeros;value_null_age_support=unchanged;no_state_normalization";
const READ_IDENTITY_LATCH_ADDRESS_OPERATION: &str =
    "g=sigmoid(W[u,prev_u]+b);held_h=(1-g)*prev_h+g*u;initial=zeros;direct_geometric_address=prior_h,current_u;no_qk_maps;value_null_age_support=unchanged;no_state_normalization";

pub const GEOMETRIC_ADDRESS_RECORD: &str = "geometric-address.json";
const GEOMETRIC_ADDRESS_SCHEMA: &str = "uor-r4.stack-geometric-address/1";
const GEOMETRIC_ADDRESS_OPERATION: &str =
    "content=prior_held_h;context=current_gained_u;paired_direct_4d_codes;directed_relatives;separate_radius_presence;learned_unary_bilinear_radius_presence;no_qk_maps;causal_including_self;NoRead_age_value_output=unchanged";
const GEOMETRIC_ADDRESS_SPAN_OPERATION: &str =
    "content=prior_committed_ordered_token_span;context=current_gained_u;paired_direct_4d_codes;directed_relatives;separate_radius_presence;learned_unary_bilinear_radius_presence;no_qk_maps;causal_including_self;NoRead_age_value_output=unchanged";

pub const GEOMETRIC_SPAN_RECORD: &str = "geometric-span.json";
const GEOMETRIC_SPAN_SCHEMA: &str = "uor-r4.stack-geometric-span/1";
const GEOMETRIC_SPAN_OPERATION: &str =
    "actions=raw_selected_embedding_4d_roots;controller=W[current_gained_u,previous_gained_u]+b;Hold_Open_Append_Commit;predicted_earliest_argmax;ordered_right_product;old_committed_content_before_step;unit_radius_separate_presence;training_labels_never_input";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GeometricSpanRecord {
    schema: String,
    operation: String,
    span: GeometricSpanConfig,
    config_sha256: String,
    model_sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GeometricAddressRecord {
    schema: String,
    operation: String,
    address: GeometricAddressConfig,
    config_sha256: String,
    model_sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadIdentityLatchRecord {
    schema: String,
    operation: String,
    mode: ReadIdentityLatch,
    config_sha256: String,
    model_sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadIdentityCarryRecord {
    schema: String,
    operation: String,
    config_sha256: String,
    model_sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadIdentityCarryMarker {
    schema: String,
    record_sha256: String,
}

/// A fixed set of unit quaternions that every recurrence's unit transport
/// quaternion snaps to, before its scaling by lambda, in training and
/// evaluation alike ([`StackModel::set_transport_snap`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportSnap {
    /// The 120 unit icosians of the binary icosian group 2I, the vertices of
    /// the 600-cell (`canonical_h4_roots`), as f32 quaternions in that order
    /// ([`Self::roots`]).
    Icosian,
}

impl TransportSnap {
    /// The unit quaternions the transports snap to.
    pub fn roots(self) -> &'static [[f32; 4]] {
        match self {
            Self::Icosian => icosian_roots(),
        }
    }

    /// A stable name for arguments and reports.
    pub fn name(self) -> &'static str {
        match self {
            Self::Icosian => "icosian",
        }
    }

    /// The index into [`Self::roots`] of the root nearest to the unit
    /// quaternion `unit`: the largest dot product, the first on a tie.
    pub fn nearest(self, unit: [f32; 4]) -> usize {
        nearest_root(unit, self.roots())
    }

    /// The index of the root the snapped forward pass selects for raw
    /// rotation logits `raw`: [`Self::nearest`] of their unit quaternion
    /// (`raw / sqrt(|raw|^2 + 1e-6)`). The D11 snap parity tests compare the
    /// integer kernel against exactly this.
    pub fn nearest_for_raw(self, raw: [f32; 4]) -> usize {
        nearest_root(unit_quaternion(raw).0, self.roots())
    }

    /// SHA-256 of the roots' little-endian f32 bytes, row by row.
    pub fn roots_sha256(self) -> String {
        let mut digest = Sha256::new();
        for root in self.roots() {
            for value in root {
                digest.update(value.to_le_bytes());
            }
        }
        hex::encode(digest.finalize())
    }

    /// The snap, its root count and the roots' digest, for settings, resume
    /// lineages and reports.
    pub fn record(self) -> serde_json::Value {
        serde_json::json!({
            "snap": self,
            "roots": self.roots().len(),
            "roots_sha256": self.roots_sha256(),
        })
    }

    /// Whether `config`'s stacks can snap their transport: geometric, with
    /// `rotation = true` and at least one recurrence layer.
    pub fn check(self, config: &StackConfig) -> Result<()> {
        if config.rotation_group != RotationGroup::Quaternion {
            return Err(invalid(
                "a transport snap replaces unit quaternions by roots of a non-commutative group; \
                 the U(1) control has none",
            ));
        }
        if config.arch != StackArch::Geometric || !config.rotation || !config.pattern.contains('r')
        {
            return Err(invalid(format!(
                "a {} transport snap needs a geometric stack with rotation=true and at least one \
                 recurrence (r) layer; this one is {:?}, rotation={}, pattern {}",
                self.name(),
                config.arch,
                config.rotation,
                config.pattern
            )));
        }
        Ok(())
    }

    fn file_record(self) -> TransportFileRecord {
        TransportFileRecord {
            schema: TRANSPORT_RECORD_SCHEMA.to_owned(),
            snap: self,
            roots: self.roots().len(),
            roots_sha256: self.roots_sha256(),
            scope: "The model was saved with this transport snap set: its forward pass replaced \
                    every unit transport quaternion by the nearest of these roots before its \
                    scaling by lambda, with straight-through gradients. model.safetensors holds \
                    the float variables; StackModel::load restores this snap, and \
                    StackModel::saved_transport_snap reads this record."
                .to_owned(),
        }
    }
}

/// [`TRANSPORT_RECORD`]'s contents.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TransportFileRecord {
    schema: String,
    snap: TransportSnap,
    roots: usize,
    roots_sha256: String,
    scope: String,
}

/// The 120 unit icosians of 2I (`canonical_h4_roots`), in f32.
fn icosian_roots() -> &'static [[f32; 4]] {
    static ROOTS: OnceLock<Vec<[f32; 4]>> = OnceLock::new();
    ROOTS.get_or_init(|| {
        canonical_h4_roots()
            .iter()
            .map(|root| {
                let a = root.to_array();
                [a[0] as f32, a[1] as f32, a[2] as f32, a[3] as f32]
            })
            .collect()
    })
}

/// One snap selection of [`StackModel::snap_selections`]: the root index and
/// the float margin to the runner-up dot product (`>= 0`; near zero on a
/// near-tie, where the integer kernel's exact comparison can legitimately
/// break an f32 tie the other way).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SnapSelection {
    pub index: usize,
    pub margin: f32,
}

/// How often a snapped forward pass selected each root
/// ([`StackModel::transport_usage`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportUsage {
    pub snap: TransportSnap,
    /// For each recurrence layer (by index), the selections of each root in
    /// [`TransportSnap::roots`] order, over its lanes and the counted
    /// positions.
    pub layers: BTreeMap<usize, Vec<u64>>,
}

impl TransportUsage {
    /// No selections yet, for `config`'s recurrence layers.
    pub fn new(snap: TransportSnap, config: &StackConfig) -> Self {
        let roots = snap.roots().len();
        Self {
            snap,
            layers: (0..config.layers())
                .filter(|&layer| config.layer_kind(layer) == 'r')
                .map(|layer| (layer, vec![0; roots]))
                .collect(),
        }
    }

    /// Add `other`'s selections, of the same snap and layers.
    pub fn add(&mut self, other: &Self) -> Result<()> {
        if self.snap != other.snap
            || !self.layers.keys().eq(other.layers.keys())
            || self
                .layers
                .values()
                .zip(other.layers.values())
                .any(|(a, b)| a.len() != b.len())
        {
            return Err(invalid("transport usages of different snaps or layers"));
        }
        for (mine, theirs) in self.layers.values_mut().zip(other.layers.values()) {
            for (a, b) in mine.iter_mut().zip(theirs) {
                *a += b;
            }
        }
        Ok(())
    }

    /// The selections of each root over every layer.
    pub fn pooled(&self) -> Vec<u64> {
        let mut pooled = vec![0u64; self.snap.roots().len()];
        for counts in self.layers.values() {
            for (a, b) in pooled.iter_mut().zip(counts) {
                *a += b;
            }
        }
        pooled
    }

    /// Selections, distinct roots selected, the entropy (bits) of the
    /// selection distribution against its maximum `log2(roots)`, the
    /// identity's share and the five most selected roots: pooled over the
    /// layers, then per layer.
    pub fn summary(&self) -> serde_json::Value {
        let roots = self.snap.roots();
        let identity = roots.iter().position(|r| *r == [1.0, 0.0, 0.0, 0.0]);
        let describe = |counts: &[u64]| -> serde_json::Value {
            let total: u64 = counts.iter().sum();
            let share = |count: u64| {
                if total == 0 {
                    0.0
                } else {
                    count as f64 / total as f64
                }
            };
            let entropy: f64 = counts
                .iter()
                .filter(|&&count| count > 0)
                .map(|&count| {
                    let p = share(count);
                    -p * p.log2()
                })
                .sum();
            let mut ranked: Vec<(usize, u64)> = counts.iter().copied().enumerate().collect();
            ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            serde_json::json!({
                "selections": total,
                "distinct_roots": counts.iter().filter(|&&count| count > 0).count(),
                "entropy_bits": entropy,
                "identity_share": identity.map(|index| share(counts[index])),
                "top_roots": ranked
                    .iter()
                    .take(5)
                    .filter(|(_, count)| *count > 0)
                    .map(|&(index, count)| serde_json::json!({
                        "index": index, "root": roots[index], "share": share(count),
                    }))
                    .collect::<Vec<_>>(),
            })
        };
        let mut summary = describe(&self.pooled());
        summary["snap"] = serde_json::json!(self.snap);
        summary["roots"] = serde_json::json!(roots.len());
        summary["max_entropy_bits"] = serde_json::json!((roots.len() as f64).log2());
        summary["per_layer"] = self
            .layers
            .iter()
            .map(|(layer, counts)| {
                let mut report = describe(counts);
                report["layer"] = serde_json::json!(layer);
                report
            })
            .collect::<Vec<_>>()
            .into();
        summary
    }
}

impl StackModel {
    /// Snap every recurrence's unit transport quaternion, before its scaling
    /// by lambda, to the nearest of `snap`'s roots (`Some`), or leave it free
    /// (`None`, the default). With a snap set every forward pass
    /// ([`forward`](Self::forward), [`loss`](Self::loss) and the rest)
    /// transports by the roots alone, and the gradient passes the snap
    /// straight through to the unit quaternion and on through its
    /// normalization to the rotation logits (`u + (snap(u) - u).detach()`).
    /// The fused recurrence core applies it, at 120 dot products per lane and
    /// position for [`TransportSnap::Icosian`]. Geometric stacks with
    /// `rotation = true` and at least one recurrence layer only
    /// ([`TransportSnap::check`]). It composes with the served
    /// representation: served weights compute the logits the snap reads.
    pub fn set_transport_snap(&mut self, snap: Option<TransportSnap>) -> Result<()> {
        if let Some(snap) = snap {
            snap.check(&self.config)?;
        }
        self.transport = snap;
        Ok(())
    }

    /// The transport snap the forward pass applies, if set.
    pub fn transport_snap(&self) -> Option<TransportSnap> {
        self.transport
    }

    /// `f` on this model with its transport free, with the snap restored
    /// afterwards. Served mode is unchanged.
    pub fn with_unsnapped_transport<T>(&mut self, f: impl FnOnce(&Self) -> Result<T>) -> Result<T> {
        let snap = self.transport.take();
        let result = f(self);
        self.transport = snap;
        result
    }

    /// The root the snapped forward pass selects at every (recurrence layer,
    /// position, lane) for `ids` (as [`forward`](Self::forward) takes them):
    /// per recurrence layer, one [`SnapSelection`] per selection, window by
    /// window, then position, then lane. The selections are those of the
    /// fused core: its gates, normalization and nearest root. The D11 snap
    /// parity checks compare the integer engine's trace against this.
    pub fn snap_selections(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<BTreeMap<usize, Vec<SnapSelection>>> {
        let snap = self
            .transport
            .ok_or_else(|| invalid("snap selections need a transport snap"))?;
        let roots = snap.roots();
        let (width, lanes) = (self.config.width, self.config.width / 4);
        let gate_width = lanes + width;
        let mut selections: BTreeMap<usize, Vec<SnapSelection>> = (0..self.config.layers())
            .filter(|&layer| self.config.layer_kind(layer) == 'r')
            .map(|layer| (layer, Vec::with_capacity(batch * time * lanes)))
            .collect();
        let last = selections.keys().next_back().copied().unwrap_or(0);
        let p = self.params()?;
        let mut x = self.embed_with(&p, ids, batch, time)?;
        for layer in 0..=last {
            if let Some(selected) = selections.get_mut(&layer) {
                let u = self.norm(&p, &x, &layer_name(layer, "rec_norm.weight"))?;
                let gates = self
                    .recurrence_gates(&p, layer, &u)?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                if gates.len() != batch * time * gate_width {
                    return Err(invalid("recurrence gates differ from their layout"));
                }
                for row in gates.chunks_exact(gate_width) {
                    for lane in 0..lanes {
                        let (unit, _) = unit_quaternion(quad(row, lanes + 4 * lane));
                        let (mut best, mut second) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
                        let mut index = 0;
                        for (j, root) in roots.iter().enumerate() {
                            let dot = unit[0] * root[0]
                                + unit[1] * root[1]
                                + unit[2] * root[2]
                                + unit[3] * root[3];
                            if dot > best {
                                second = best;
                                best = dot;
                                index = j;
                            } else if dot > second {
                                second = dot;
                            }
                        }
                        selected.push(SnapSelection {
                            index,
                            margin: best - second,
                        });
                    }
                }
            }
            x = self.layer_range_hooked(&p, x, layer..layer + 1, &mut None)?;
        }
        Ok(selections)
    }

    /// The roots the snapped forward pass selects for `ids` (as
    /// [`forward`](Self::forward) takes them), counted per recurrence layer
    /// over every lane and, in window `w`, its first `lengths[w]` positions
    /// (all `time` without `lengths`). The selections are those of the fused
    /// core: its gates, normalization and nearest root
    /// ([`Self::snap_selections`]).
    pub fn transport_usage(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        lengths: Option<&[usize]>,
    ) -> Result<TransportUsage> {
        let snap = self
            .transport
            .ok_or_else(|| invalid("transport usage needs a transport snap"))?;
        if let Some(lengths) = lengths {
            if lengths.len() != batch || lengths.iter().any(|&length| length > time) {
                return Err(invalid(
                    "transport usage needs one length of at most time per window",
                ));
            }
        }
        let lanes = self.config.width / 4;
        let selections = self.snap_selections(ids, batch, time)?;
        let mut usage = TransportUsage::new(snap, &self.config);
        for (layer, selected) in &selections {
            let counts = usage
                .layers
                .get_mut(layer)
                .ok_or_else(|| invalid("snap selections of an unknown layer"))?;
            for (window, selected) in selected.chunks_exact(time * lanes).enumerate() {
                let counted = lengths.map_or(time, |lengths| lengths[window]);
                for selection in &selected[..counted * lanes] {
                    counts[selection.index] += 1;
                }
            }
        }
        Ok(usage)
    }
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

/// What a saved stack's `config.json` records, beside its configuration, of
/// the served representation the forward pass read when it was saved
/// ([`StackModel::save`] in served mode; read back by
/// [`StackModel::saved_served_representation`]). An export must write this
/// representation, or the served weights are not the ones the model trained
/// against.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedServedRepresentation {
    /// The codec's [`MapCodec::name`].
    pub codec: String,
}

/// `config.json` of a model saved in served mode: the configuration's fields,
/// then the record, which [`StackConfig`] ignores when it is read.
#[derive(Serialize)]
struct ServedConfigFile<'a> {
    #[serde(flatten)]
    config: &'a StackConfig,
    served_representation: SavedServedRepresentation,
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
    if config.read == ReadScore::L2 || config.rotation_group != RotationGroup::Quaternion {
        return Err(invalid(
            "the L2 read and U(1) transport controls have no served representation; train them \
             in float (no qat=true)",
        ));
    }
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
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        if l1.shape() != l2.shape() {
            candle_core::bail!("straight-through inputs must have one shape");
        }
        if s1.dtype() != DType::F32 || s2.dtype() != DType::F32 {
            candle_core::bail!("straight-through requires F32 tensors");
        }
        if l1.shape().elem_count() == 0 {
            candle_core::bail!("straight-through requires non-empty tensors");
        }
        Ok((
            CpuStorage::F32(contiguous(s2, l2)?.to_vec()),
            l2.shape().clone(),
        ))
    }

    #[cfg(feature = "metal")]
    fn metal_fwd(
        &self,
        _s1: &MetalStorage,
        l1: &Layout,
        s2: &MetalStorage,
        l2: &Layout,
    ) -> candle_core::Result<(MetalStorage, Shape)> {
        if l1.shape() != l2.shape() {
            candle_core::bail!("straight-through inputs must have one shape");
        }
        if _s1.dtype() != DType::F32 || s2.dtype() != DType::F32 {
            candle_core::bail!("Metal straight-through requires F32 dtype");
        }
        if l1.start_offset() != 0
            || !l1.is_contiguous()
            || l2.start_offset() != 0
            || !l2.is_contiguous()
        {
            candle_core::bail!("Metal kernel requires contiguous layout with zero start offset");
        }
        let device = s2.device();
        let total = l2.shape().elem_count();
        if total == 0 {
            candle_core::bail!("Metal straight-through requires non-empty tensors");
        }
        let out_buf = device.new_buffer(total, DType::F32, "straight_through_out")?;
        crate::metal_stack_kernels::metal::call_straight_through(
            device,
            s2.buffer(),
            &out_buf,
            total,
        )?;
        Ok((
            MetalStorage::new(out_buf, device.clone(), total, DType::F32),
            l2.shape().clone(),
        ))
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        cuda_ops::straight_through_fwd(s1, l1, s2, l2)
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

/// Straight-through estimator: forward is `quantized`, backward gradient flows to `continuous`.
pub fn straight_through(continuous: &Tensor, quantized: &Tensor) -> Result<Tensor> {
    if continuous.dtype() != DType::F32 || quantized.dtype() != DType::F32 {
        return Err(invalid("straight_through requires F32 tensors"));
    }
    if continuous.shape() != quantized.shape() {
        return Err(invalid("straight-through inputs must have one shape"));
    }
    if continuous.elem_count() == 0 {
        return Err(invalid("straight_through requires non-empty tensors"));
    }
    Ok(continuous
        .contiguous()?
        .apply_op2(&quantized.contiguous()?, StraightThrough)?)
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
    // Ordinary float semantics with immutable, graph-detached parameters.
    Frozen(BTreeMap<String, Tensor>),
}

impl Params<'_> {
    fn get(&self, name: &str) -> Result<&Tensor> {
        let found = match self {
            Self::Float(variables) => variables.get(name).map(Var::as_tensor),
            Self::Served(tensors) => tensors.get(name),
            Self::Frozen(tensors) => tensors.get(name),
        };
        found.ok_or_else(|| invalid(format!("missing stack variable {name}")))
    }

    fn layer(&self, layer: usize, suffix: &str) -> Result<&Tensor> {
        self.get(&layer_name(layer, suffix))
    }

    /// The output map: the tied embedding, or the served head.
    fn head(&self) -> Result<&Tensor> {
        match self {
            Self::Float(_) | Self::Frozen(_) => self.get("embedding.weight"),
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
            Some(_) if self.geometric_address.is_some() => {
                return Err(invalid("geometric addressing has no served representation"));
            }
            Some(_) if self.read_identity_carry => {
                return Err(invalid("read identity carry has no served representation"));
            }
            Some(_) if self.read_identity_latch.is_some() => {
                return Err(invalid("read identity latch has no served representation"));
            }
            Some(_) if self.config.pointer.is_some() => return Err(pointer_served_refusal()),
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

/// [`adam_step`] on Metal, in place in the variable's and moments' buffers,
/// with the same f32 operations in the same order.
#[cfg(feature = "metal")]
fn adam_step_metal(
    device: &candle_core::MetalDevice,
    var: &Var,
    grad: &Tensor,
    m: &Var,
    v: &Var,
    c: &AdamConstants,
) -> Result<()> {
    use crate::metal_stack_kernels::metal::{launch, Arg};
    let gradient = metal_ready(grad)?;
    let n = var.elem_count();
    if gradient.elem_count() != n || m.elem_count() != n || v.elem_count() != n {
        return Err(invalid("Metal AdamW sizes disagree"));
    }
    let (p, g, first, second) = (
        metal_storage(var.as_tensor())?,
        metal_storage(&gradient)?,
        metal_storage(m.as_tensor())?,
        metal_storage(v.as_tensor())?,
    );
    let constants = [
        c.scale, c.beta1, c.rest1, c.beta2, c.rest2, c.correct1, c.correct2, c.epsilon, c.keep,
        c.lr,
    ]
    .map(f32::to_bits);
    launch(
        device,
        "adam_update",
        n,
        &[
            Arg::Out(p.buffer()),
            Arg::Out(first.buffer()),
            Arg::Out(second.buffer()),
            Arg::In(g.buffer()),
            Arg::Words(&constants),
            Arg::U32(n as u32),
        ],
    )?;
    Ok(())
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
        if matches!(model.device(), Device::Metal(_) | Device::Cuda(_)) {
            // One host synchronization for the whole norm: each variable's
            // squared sum stays on the device, then their f32 sum is read.
            let sums = model
                .variables()
                .values()
                .filter_map(|var| grads.get(var.as_tensor()))
                .map(|grad| grad.sqr()?.sum_all()?.reshape(1))
                .collect::<candle_core::Result<Vec<_>>>()?;
            if !sums.is_empty() {
                total = f64::from(Tensor::cat(&sums, 0)?.sum_all()?.to_scalar::<f32>()?);
            }
        } else {
            for var in model.variables().values() {
                if let Some(grad) = grads.get(var.as_tensor()) {
                    total += f64::from(grad.sqr()?.sum_all()?.to_scalar::<f32>()?);
                }
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
            #[cfg(feature = "metal")]
            if let Device::Metal(device) = var.device() {
                adam_step_metal(device, var, grad, m, v, &constants)?;
                continue;
            }
            #[cfg(feature = "cuda")]
            if let Device::Cuda(device) = var.device() {
                cuda_ops::adam_step_cuda(device, var, grad, m, v, &constants)?;
                continue;
            }
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

    #[cfg(feature = "metal")]
    fn metal_fwd(
        &self,
        s1: &MetalStorage,
        l1: &Layout,
        s2: &MetalStorage,
        l2: &Layout,
    ) -> candle_core::Result<(MetalStorage, Shape)> {
        if s1.dtype() != DType::F32 || s2.dtype() != DType::F32 {
            candle_core::bail!("Metal quaternion scan requires F32 dtype");
        }
        let (batch, time, lanes, four) = l1.shape().dims4()?;
        if four != 4 || l2.shape() != l1.shape() {
            candle_core::bail!("quaternion scan needs matching [batch, time, lanes, 4] inputs");
        }
        if batch == 0 || time == 0 || lanes == 0 {
            candle_core::bail!("quaternion scan requires positive dimensions");
        }
        if l1.start_offset() != 0
            || !l1.is_contiguous()
            || l2.start_offset() != 0
            || !l2.is_contiguous()
        {
            candle_core::bail!("Metal kernel requires contiguous layout with zero start offset");
        }
        let device = s1.device();
        let total = l1.shape().elem_count();
        let out_buf = device.new_buffer(total, DType::F32, "quaternion_scan_out")?;
        crate::metal_stack_kernels::metal::call_quaternion_scan_fwd(
            device,
            s1.buffer(),
            s2.buffer(),
            &out_buf,
            batch,
            time,
            lanes,
        )?;
        Ok((
            MetalStorage::new(out_buf, device.clone(), total, DType::F32),
            l1.shape().clone(),
        ))
    }

    /// Reverse scan: `g_t = dh_t + conj(q_{t+1}) * g_{t+1}` is the total
    /// gradient of `h_t`; then `db_t = g_t` and `dq_t = g_t * conj(h_{t-1})`.
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
        #[cfg(feature = "metal")]
        if let Device::Metal(device) = transition.device() {
            if transition.dtype() != DType::F32
                || state.dtype() != DType::F32
                || grad.dtype() != DType::F32
            {
                candle_core::bail!("Metal quaternion scan backward requires F32 dtype");
            }
            let (batch, time, lanes, four) = transition.dims4()?;
            if four != 4
                || state.shape() != transition.shape()
                || grad.shape() != transition.shape()
            {
                candle_core::bail!(
                    "Metal quaternion scan backward inputs must match [batch, time, lanes, 4]"
                );
            }
            if batch == 0 || time == 0 || lanes == 0 {
                candle_core::bail!("Metal quaternion scan backward requires positive dimensions");
            }
            if four == 4 {
                let (t_storage, t_layout) = transition.storage_and_layout();
                let (s_storage, s_layout) = state.storage_and_layout();
                let (g_storage, g_layout) = grad.storage_and_layout();

                if t_layout.start_offset() != 0
                    || !t_layout.is_contiguous()
                    || s_layout.start_offset() != 0
                    || !s_layout.is_contiguous()
                    || g_layout.start_offset() != 0
                    || !g_layout.is_contiguous()
                {
                    candle_core::bail!(
                        "Metal kernel requires contiguous layout with zero start offset"
                    );
                }

                let total = transition.shape().elem_count();
                let dq_buf = device.new_buffer(total, DType::F32, "quaternion_scan_bwd_dq")?;
                let db_buf = device.new_buffer(total, DType::F32, "quaternion_scan_bwd_db")?;

                if let (Storage::Metal(t_ms), Storage::Metal(s_ms), Storage::Metal(g_ms)) =
                    (&*t_storage, &*s_storage, &*g_storage)
                {
                    crate::metal_stack_kernels::metal::call_quaternion_scan_bwd(
                        device,
                        t_ms.buffer(),
                        s_ms.buffer(),
                        g_ms.buffer(),
                        &dq_buf,
                        &db_buf,
                        batch,
                        time,
                        lanes,
                    )?;
                    let dq = Tensor::from_storage(
                        Storage::Metal(MetalStorage::new(
                            dq_buf,
                            device.clone(),
                            total,
                            DType::F32,
                        )),
                        transition.shape().clone(),
                        candle_core::op::BackpropOp::none(),
                        false,
                    );
                    let db = Tensor::from_storage(
                        Storage::Metal(MetalStorage::new(
                            db_buf,
                            device.clone(),
                            total,
                            DType::F32,
                        )),
                        transition.shape().clone(),
                        candle_core::op::BackpropOp::none(),
                        false,
                    );
                    return Ok((Some(dq), Some(db)));
                }
            }
        }
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
    if transition.dtype() != DType::F32 || drive.dtype() != DType::F32 {
        return Err(invalid("quaternion_scan requires F32 tensors"));
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
///
/// With `snap` (and rotation), each unit quaternion is replaced by its nearest
/// root before its scaling by lambda, and the backward passes the root's
/// gradient to the unit quaternion unchanged (straight-through). Without it the
/// core computes what it computed before the snap existed, bit for bit.
#[derive(Clone, Copy, Debug)]
struct RecurrenceCore {
    batch: usize,
    time: usize,
    width: usize,
    rotation: bool,
    /// With rotation, the transport's group; [`RotationGroup::U1`] zeroes each
    /// raw quaternion's `j` and `k` before normalization (and their gradient).
    group: RotationGroup,
    snap: Option<TransportSnap>,
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
    /// The transport's unit quaternion, `q = lambda rotation`: `unit`, or with
    /// a snap its nearest root.
    rotation: [f32; 4],
    /// `raw / norm`, whose normalization the backward differentiates.
    unit: [f32; 4],
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
        let roots = self.snap.map(TransportSnap::roots);
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
                let (rotation, unit, norm) = if self.rotation {
                    let mut raw = quad(gate_row, lanes + 4 * lane);
                    if self.group == RotationGroup::U1 {
                        raw[2] = 0.0;
                        raw[3] = 0.0;
                    }
                    let (unit, norm) = unit_quaternion(raw);
                    let rotation = match roots {
                        None => unit,
                        Some(roots) => roots[nearest_root(unit, roots)],
                    };
                    (rotation, unit, norm)
                } else {
                    ([1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0], 1.0)
                };
                transitions[t * lanes + lane] = Transition {
                    opening,
                    lambda,
                    keep,
                    clamped,
                    rotation,
                    unit,
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

/// Runs a three-input op's exact CPU forward on host copies of Metal inputs
/// and returns the result as a Metal buffer. Used where a Metal kernel does
/// not cover the op's configuration, so a Metal model computes exactly what
/// the CPU model computes (unified memory makes the copies cheap).
#[cfg(feature = "metal")]
fn metal_via_host(
    op: &impl CustomOp3,
    inputs: [(&MetalStorage, &Layout); 3],
) -> candle_core::Result<(MetalStorage, Shape)> {
    let [(s1, l1), (s2, l2), (s3, l3)] = inputs;
    let (c1, c2, c3) = (
        s1.to_cpu_storage()?,
        s2.to_cpu_storage()?,
        s3.to_cpu_storage()?,
    );
    let (out, shape) = op.cpu_fwd(&c1, l1, &c2, l2, &c3, l3)?;
    let CpuStorage::F32(values) = out else {
        candle_core::bail!("{} host fallback expects F32 output", op.name());
    };
    let device = s1.device();
    let buffer = device.new_buffer_with_data(&values)?;
    Ok((
        MetalStorage::new(buffer, device.clone(), values.len(), DType::F32),
        shape,
    ))
}

/// A Metal F32 tensor laid out contiguously from offset zero (copied only
/// when it is not), for binding its whole buffer to a kernel.
#[cfg(feature = "metal")]
fn metal_ready(tensor: &Tensor) -> candle_core::Result<Tensor> {
    if tensor.dtype() != DType::F32 {
        candle_core::bail!("Metal stack kernels require F32 tensors");
    }
    if tensor.is_contiguous() && tensor.layout().start_offset() == 0 {
        Ok(tensor.clone())
    } else {
        tensor.force_contiguous()
    }
}

/// The Metal storage of a tensor prepared by [`metal_ready`].
#[cfg(feature = "metal")]
fn metal_storage(tensor: &Tensor) -> candle_core::Result<MetalStorage> {
    let (storage, layout) = tensor.storage_and_layout();
    if layout.start_offset() != 0 || !layout.is_contiguous() {
        candle_core::bail!("Metal kernel requires contiguous layout with zero start offset");
    }
    match &*storage {
        Storage::Metal(storage) => Ok(storage.clone()),
        _ => candle_core::bail!("expected a Metal tensor"),
    }
}

/// A gradient tensor over a fresh Metal buffer.
#[cfg(feature = "metal")]
fn metal_tensor(
    buffer: Arc<candle_metal_kernels::metal::Buffer>,
    device: &candle_core::MetalDevice,
    shape: &Shape,
) -> Tensor {
    Tensor::from_storage(
        Storage::Metal(MetalStorage::new(
            buffer,
            device.clone(),
            shape.elem_count(),
            DType::F32,
        )),
        shape.clone(),
        candle_core::op::BackpropOp::none(),
        false,
    )
}

#[cfg(feature = "metal")]
impl RecurrenceCore {
    /// Whether the Metal kernels cover this configuration: a snap and the
    /// U(1) control run on the host.
    fn metal_covered(&self) -> bool {
        self.snap.is_none() && self.group == RotationGroup::Quaternion
    }

    /// `log a` per lane computed on the device (no host synchronization).
    fn metal_log_a(
        &self,
        device: &candle_core::MetalDevice,
        parameters: &candle_metal_kernels::metal::Buffer,
        parameter_offset: usize,
    ) -> candle_core::Result<Arc<candle_metal_kernels::metal::Buffer>> {
        use crate::metal_stack_kernels::metal::{launch, Arg};
        let lanes = self.lanes();
        let log_a = device.new_buffer(lanes, DType::F32, "recurrence_log_a")?;
        launch(
            device,
            "recurrence_log_a",
            lanes,
            &[
                Arg::InAt(parameters, parameter_offset),
                Arg::Out(&log_a),
                Arg::U32(self.width as u32),
                Arg::U32(lanes as u32),
            ],
        )?;
        Ok(log_a)
    }

    /// The exact backward on Metal: recomputes the forward states, sweeps
    /// each (window, lane) in reverse, then reduces the windows' parameter
    /// partials.
    fn metal_bwd(
        &self,
        device: &candle_core::MetalDevice,
        branches: &Tensor,
        gates: &Tensor,
        parameters: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Tensor, Tensor, Tensor)> {
        use crate::metal_stack_kernels::metal::{call_recurrence_core_fwd, launch, Arg};
        let (b, g, p, dy) = (
            metal_ready(branches)?,
            metal_ready(gates)?,
            metal_ready(parameters)?,
            metal_ready(grad)?,
        );
        let (time, width, lanes) = (self.time, self.width, self.lanes());
        let gate_width = self.gate_width();
        let param_len = self.parameter_len();
        if b.elem_count() != self.batch * time * 2 * width
            || g.elem_count() != self.batch * time * gate_width
            || p.elem_count() != param_len
            || dy.elem_count() != self.batch * time * width
        {
            candle_core::bail!("Metal recurrence backward inputs have the wrong sizes");
        }
        let (bs, gs, ps, ds) = (
            metal_storage(&b)?,
            metal_storage(&g)?,
            metal_storage(&p)?,
            metal_storage(&dy)?,
        );
        let log_a = self.metal_log_a(device, ps.buffer(), 0)?;
        let total_state = self.batch * time * width;
        let state = device.new_buffer(total_state, DType::F32, "recurrence_bwd_state")?;
        let drive = device.new_buffer(total_state, DType::F32, "recurrence_bwd_drive")?;
        let out = device.new_buffer(total_state, DType::F32, "recurrence_bwd_out")?;
        call_recurrence_core_fwd(
            device,
            bs.buffer(),
            gs.buffer(),
            ps.buffer(),
            &log_a,
            &state,
            &drive,
            &out,
            self.batch,
            time,
            width,
            self.rotation,
        )?;
        let d_branches = device.new_buffer(b.elem_count(), DType::F32, "recurrence_d_branches")?;
        let d_gates = device.new_buffer(g.elem_count(), DType::F32, "recurrence_d_gates")?;
        let partials =
            device.new_buffer(self.batch * param_len, DType::F32, "recurrence_partials")?;
        let d_parameters = device.new_buffer(param_len, DType::F32, "recurrence_d_params")?;
        launch(
            device,
            "recurrence_core_bwd",
            self.batch * lanes,
            &[
                Arg::In(bs.buffer()),
                Arg::In(gs.buffer()),
                Arg::In(ps.buffer()),
                Arg::In(&log_a),
                Arg::In(&state),
                Arg::In(&drive),
                Arg::In(ds.buffer()),
                Arg::Out(&d_branches),
                Arg::Out(&d_gates),
                Arg::Out(&partials),
                Arg::U32(time as u32),
                Arg::U32(width as u32),
                Arg::U32(lanes as u32),
                Arg::U32(gate_width as u32),
                Arg::U32(u32::from(self.rotation)),
                Arg::U32((self.batch * lanes) as u32),
                Arg::U32(param_len as u32),
            ],
        )?;
        launch(
            device,
            "recurrence_param_reduce",
            param_len,
            &[
                Arg::In(&partials),
                Arg::In(ps.buffer()),
                Arg::Out(&d_parameters),
                Arg::U32(self.batch as u32),
                Arg::U32(param_len as u32),
                Arg::U32(width as u32),
            ],
        )?;
        Ok((
            metal_tensor(d_branches, device, branches.shape()),
            metal_tensor(d_gates, device, gates.shape()),
            metal_tensor(d_parameters, device, parameters.shape()),
        ))
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

    #[cfg(feature = "metal")]
    fn metal_fwd(
        &self,
        s1: &MetalStorage,
        l1: &Layout,
        s2: &MetalStorage,
        l2: &Layout,
        s3: &MetalStorage,
        l3: &Layout,
    ) -> candle_core::Result<(MetalStorage, Shape)> {
        if !self.metal_covered() {
            return metal_via_host(self, [(s1, l1), (s2, l2), (s3, l3)]);
        }
        if s1.dtype() != DType::F32 || s2.dtype() != DType::F32 || s3.dtype() != DType::F32 {
            candle_core::bail!("Metal RecurrenceCore requires F32 dtype");
        }
        if self.batch == 0 || self.time == 0 || self.width == 0 || self.width % 4 != 0 {
            candle_core::bail!(
                "Metal RecurrenceCore requires positive dimensions with width divisible by 4"
            );
        }
        let (time, width) = (self.time, self.width);
        let expected_branches = self.batch * time * 2 * width;
        let expected_gates = self.batch * time * self.gate_width();
        let expected_params = self.parameter_len();
        if l1.shape().elem_count() != expected_branches
            || l2.shape().elem_count() != expected_gates
            || l3.shape().elem_count() != expected_params
        {
            candle_core::bail!(
                "Metal RecurrenceCore input buffer lengths do not match declared dimensions"
            );
        }
        if l1.start_offset() != 0
            || !l1.is_contiguous()
            || l2.start_offset() != 0
            || !l2.is_contiguous()
            || l3.start_offset() != 0
            || !l3.is_contiguous()
        {
            candle_core::bail!("Metal kernel requires contiguous layout with zero start offset");
        }
        let device = s1.device();
        let total_state = self.batch * time * width;

        let state_buf = device.new_buffer(total_state, DType::F32, "recurrence_state")?;
        let drive_buf = device.new_buffer(total_state, DType::F32, "recurrence_drive")?;
        let out_buf = device.new_buffer(total_state, DType::F32, "recurrence_out")?;

        let log_a_buf = self.metal_log_a(device, s3.buffer(), 0)?;

        crate::metal_stack_kernels::metal::call_recurrence_core_fwd(
            device,
            s1.buffer(),
            s2.buffer(),
            s3.buffer(),
            &log_a_buf,
            &state_buf,
            &drive_buf,
            &out_buf,
            self.batch,
            time,
            width,
            self.rotation,
        )?;

        Ok((
            MetalStorage::new(out_buf, device.clone(), total_state, DType::F32),
            Shape::from((self.batch, time, width)),
        ))
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
        s3: &CudaStorage,
        l3: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        self.cuda_fwd_impl(s1, l1, s2, l2, s3, l3)
    }

    fn bwd(
        &self,
        branches: &Tensor,
        gates: &Tensor,
        parameters: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>, Option<Tensor>)> {
        #[cfg(feature = "cuda")]
        if let (Device::Cuda(device), true) = (branches.device(), self.cuda_covered()) {
            let (d_branches, d_gates, d_parameters) =
                self.cuda_bwd(device, branches, gates, parameters, grad)?;
            return Ok((Some(d_branches), Some(d_gates), Some(d_parameters)));
        }
        #[cfg(feature = "metal")]
        if let (Device::Metal(device), true) = (branches.device(), self.metal_covered()) {
            let (d_branches, d_gates, d_parameters) =
                self.metal_bwd(device, branches, gates, parameters, grad)?;
            return Ok((Some(d_branches), Some(d_gates), Some(d_parameters)));
        }
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
                        // q = lambda u (u the unit, or with a snap its root),
                        // with h_{-1} = 0.
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
                            // u = raw / n, n = sqrt(|raw|^2 + eps). A snapped
                            // rotation's gradient reaches u unchanged
                            // (straight-through), so the normalization is
                            // differentiated at the unsnapped unit.
                            let du = dq.map(|v| v * transition.lambda);
                            let unit = transition.unit;
                            let projection: f32 = (0..4).map(|k| du[k] * unit[k]).sum();
                            for k in 0..4 {
                                d_gate[t * gate_width + lanes + offset + k] =
                                    (du[k] - unit[k] * projection) / transition.norm;
                            }
                            if self.group == RotationGroup::U1 {
                                // j and k were zeroed before normalization.
                                d_gate[t * gate_width + lanes + offset + 2] = 0.0;
                                d_gate[t * gate_width + lanes + offset + 3] = 0.0;
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
        + (if score.scaled() { 2 * heads } else { 0 })
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
    /// Flock selection of the sources each row softmaxes over.
    select: Option<FlockSelect>,
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
    /// Lifts sqrt(1 + |x|^2) of queries and keys, for Lorentz; for L2 the
    /// squared norms |x|^2.
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
    /// Which sources of the current row its flock selection keeps.
    keep: Vec<bool>,
}

impl Scratch {
    fn new(time: usize) -> Self {
        Self {
            excess: vec![0f64; TILE * time],
            distance: vec![0f64; TILE * time],
            dp: vec![0f32; TILE * time],
            keep: Vec::new(),
        }
    }
}

/// A selector failure (a non-finite score it cannot rank) as the error of the
/// op that met it.
fn selection_error(error: crate::TrainingError) -> candle_core::Error {
    candle_core::Error::msg(error)
}

/// Apply a selection of the shared selector ([`crate::flock`]) to one row:
/// every entry of `scores` (the row's positions `0..=t`) that `selection` does
/// not keep becomes `-inf`, so its softmax weight is exactly 0. Returns
/// whether any was dropped; when none is (the selection keeps every position)
/// `scores` is untouched, so a selection that covers the row is the plain row
/// bit for bit. `keep` is scratch. Comparisons and moves only.
fn drop_unkept(
    selection: &flock::FlockSelection,
    scores: &mut [f32],
    keep: &mut Vec<bool>,
) -> bool {
    if selection.len() >= scores.len() {
        return false;
    }
    keep.clear();
    keep.resize(scores.len(), false);
    for entry in &selection.entries {
        keep[entry.position] = true;
    }
    for (score, &kept) in scores.iter_mut().zip(keep.iter()) {
        if !kept {
            *score = f32::NEG_INFINITY;
        }
    }
    true
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
        } else if self.score == ReadScore::L2 {
            // L2 keeps the squared norms |x|^2 in the lift slots.
            let square = |row: &[f32]| f64::from(dot(row, row));
            query_lift = (0..time)
                .map(|t| square(&query[t * key..(t + 1) * key]))
                .collect();
            key_lift = (0..time)
                .map(|t| square(&key_rows[t * key..(t + 1) * key]))
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
    /// in `distance[..=t]`. Returns the NoRead probability. With a flock
    /// selection the probabilities are those of the kept sources and exactly
    /// zero elsewhere; `keep` is the selection's scratch. The selection is the
    /// shared selector's on the row's total scores, so a row it cannot rank (a
    /// non-finite score) is an error here, not a silent NaN.
    fn transform(
        &self,
        block: &Block,
        t: usize,
        row: &mut [f32],
        excess: &mut [f64],
        distance: &mut [f64],
        keep: &mut Vec<bool>,
    ) -> candle_core::Result<f32> {
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
                ReadScore::L2 => {
                    // |q - k|^2 = |q|^2 + |k|^2 - 2 q.k, in f64.
                    let s = block.query_lift[t] + block.key_lift[j] - 2.0 * f64::from(row[j]);
                    let d = l2_distance(s);
                    excess[j] = s;
                    distance[j] = d;
                    (-block.beta * (d - block.offset)) as f32 + age
                }
            };
            row[j] = score;
            maximum = maximum.max(score);
        }
        if let Some(select) = self.select {
            // Unkept scores become -inf, so their weight is exactly zero
            // (`exp(-inf - maximum)`), and the maximum is over the kept ones
            // and the NoRead slot, which is not a position and is not selected.
            let selection = flock::flock_select(&*row, t, select).map_err(selection_error)?;
            if drop_unkept(&selection, row, keep) {
                maximum = block.null.map_or(f32::NEG_INFINITY, |null| null[t]);
                for &score in row.iter() {
                    maximum = maximum.max(score);
                }
            }
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
        Ok(null_weight * inverse)
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
    ) -> candle_core::Result<[f32; TILE]> {
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
                &mut scratch.keep,
            )?;
        }
        Ok(null)
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

/// `sqrt(s)` for a squared distance `s`, clamped at `L2_MIN_SQUARED`.
#[inline]
fn l2_distance(squared: f64) -> f64 {
    squared.max(L2_MIN_SQUARED).sqrt()
}

/// The device buffers of one recomputed read: probabilities [index, t, j]
/// (j <= t used), NoRead probabilities, query and key lifts and, when asked
/// for, the Lorentz excesses.
#[cfg(feature = "metal")]
struct MetalReadPass {
    probabilities: Arc<candle_metal_kernels::metal::Buffer>,
    null_probability: Arc<candle_metal_kernels::metal::Buffer>,
    query_lift: Arc<candle_metal_kernels::metal::Buffer>,
    key_lift: Arc<candle_metal_kernels::metal::Buffer>,
    excess: Arc<candle_metal_kernels::metal::Buffer>,
}

#[cfg(feature = "metal")]
impl FusedRead {
    /// Whether the Metal kernels cover this configuration (RoPE, flock
    /// selection and the L2 control run on the host).
    fn metal_covered(&self) -> bool {
        !self.rope && self.select.is_none() && self.score != ReadScore::L2
    }

    fn metal_dims(&self) -> [u32; 8] {
        [
            self.batch as u32,
            self.heads as u32,
            self.time as u32,
            self.key as u32,
            self.value as u32,
            u32::from(self.null),
            u32::from(self.age),
            u32::from(self.score == ReadScore::Lorentz),
        ]
    }

    /// Scores and softmax of every row on the device. Inputs are bound at
    /// byte offsets.
    fn metal_pass(
        &self,
        device: &candle_core::MetalDevice,
        query: (&candle_metal_kernels::metal::Buffer, usize),
        kv: (&candle_metal_kernels::metal::Buffer, usize),
        aux: (&candle_metal_kernels::metal::Buffer, usize),
        keep_excess: bool,
    ) -> candle_core::Result<MetalReadPass> {
        use crate::metal_stack_kernels::metal::{launch, launch_groups, Arg};
        let dims = self.metal_dims();
        let rows = self.batch * self.heads * self.time;
        let square = rows * self.time;
        let lorentz = self.score == ReadScore::Lorentz;
        let probabilities = device.new_buffer(square, DType::F32, "read_probabilities")?;
        let null_probability = device.new_buffer(rows, DType::F32, "read_null_probability")?;
        let lift_len = if lorentz { rows } else { 1 };
        let query_lift = device.new_buffer(lift_len, DType::F32, "read_query_lift")?;
        let key_lift = device.new_buffer(lift_len, DType::F32, "read_key_lift")?;
        let write_excess = lorentz && keep_excess;
        let excess = device.new_buffer(
            if write_excess { square } else { 1 },
            DType::F32,
            "read_excess",
        )?;
        if lorentz {
            launch(
                device,
                "read_lift",
                rows,
                &[
                    Arg::InAt(query.0, query.1),
                    Arg::InAt(kv.0, kv.1),
                    Arg::Out(&query_lift),
                    Arg::Out(&key_lift),
                    Arg::Words(&dims),
                ],
            )?;
        }
        let tiles = self.time.div_ceil(16);
        let geometry = [
            self.key as u32,
            0,
            self.width() as u32,
            0,
            self.key as u32,
            1,
            u32::from(write_excess),
        ];
        launch_groups(
            device,
            "read_tile_inner",
            (tiles, tiles, self.batch * self.heads),
            (16, 16, 1),
            &[
                Arg::InAt(query.0, query.1),
                Arg::InAt(kv.0, kv.1),
                Arg::InAt(aux.0, aux.1),
                Arg::In(&query_lift),
                Arg::In(&key_lift),
                Arg::Out(&probabilities),
                Arg::Out(&excess),
                Arg::Words(&dims),
                Arg::Words(&geometry),
            ],
        )?;
        launch(
            device,
            "read_softmax_simd",
            32 * rows,
            &[
                Arg::Out(&probabilities),
                Arg::InAt(aux.0, aux.1),
                Arg::Out(&null_probability),
                Arg::Words(&dims),
            ],
        )?;
        Ok(MetalReadPass {
            probabilities,
            null_probability,
            query_lift,
            key_lift,
            excess,
        })
    }

    /// The exact backward on Metal.
    fn metal_bwd(
        &self,
        device: &candle_core::MetalDevice,
        query: &Tensor,
        kv: &Tensor,
        aux: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Tensor, Tensor, Tensor)> {
        use crate::metal_stack_kernels::metal::{launch, launch_groups, Arg};
        let (q, kvt, a, dy) = (
            metal_ready(query)?,
            metal_ready(kv)?,
            metal_ready(aux)?,
            metal_ready(grad)?,
        );
        let rows = self.batch * self.heads * self.time;
        if q.elem_count() != rows * self.key
            || kvt.elem_count() != rows * self.width()
            || dy.elem_count() != rows * self.value
        {
            candle_core::bail!("Metal read backward inputs have the wrong sizes");
        }
        let (qs, kvs, auxs, dys) = (
            metal_storage(&q)?,
            metal_storage(&kvt)?,
            metal_storage(&a)?,
            metal_storage(&dy)?,
        );
        let lorentz = self.score == ReadScore::Lorentz;
        let pass = self.metal_pass(
            device,
            (qs.buffer(), 0),
            (kvs.buffer(), 0),
            (auxs.buffer(), 0),
            true,
        )?;
        let dims = self.metal_dims();
        let square = rows * self.time;
        let ds = device.new_buffer(square, DType::F32, "read_ds")?;
        let inner_grad = device.new_buffer(square, DType::F32, "read_inner_grad")?;
        let row_len = if lorentz { rows } else { 1 };
        let query_self = device.new_buffer(row_len, DType::F32, "read_query_self")?;
        let row_beta = device.new_buffer(row_len, DType::F32, "read_row_beta")?;
        let row_offset = device.new_buffer(row_len, DType::F32, "read_row_offset")?;
        let key_self = device.new_buffer(row_len, DType::F32, "read_key_self")?;
        let dq = device.new_buffer(q.elem_count(), DType::F32, "read_dq")?;
        let dkv = device.new_buffer(kvt.elem_count(), DType::F32, "read_dkv")?;
        let aux_parts = self.null || self.age || lorentz;
        let d_aux = device.new_buffer(a.elem_count(), DType::F32, "read_d_aux")?;
        let tiles = self.time.div_ceil(16);
        let geometry = [
            self.value as u32,
            0,
            self.width() as u32,
            self.key as u32,
            self.value as u32,
            0,
            0,
        ];
        launch_groups(
            device,
            "read_tile_inner",
            (tiles, tiles, self.batch * self.heads),
            (16, 16, 1),
            &[
                Arg::In(dys.buffer()),
                Arg::In(kvs.buffer()),
                Arg::In(auxs.buffer()),
                Arg::In(&pass.query_lift),
                Arg::In(&pass.key_lift),
                Arg::Out(&ds),
                Arg::Out(&pass.excess),
                Arg::Words(&dims),
                Arg::Words(&geometry),
            ],
        )?;
        launch(
            device,
            "read_row_grad_simd",
            32 * rows,
            &[
                Arg::In(&pass.probabilities),
                Arg::Out(&ds),
                Arg::Out(&inner_grad),
                Arg::In(&pass.excess),
                Arg::In(&pass.null_probability),
                Arg::In(auxs.buffer()),
                Arg::In(&pass.query_lift),
                Arg::In(&pass.key_lift),
                Arg::Out(&query_self),
                Arg::Out(&row_beta),
                Arg::Out(&row_offset),
                Arg::Out(&d_aux),
                Arg::Words(&dims),
            ],
        )?;
        if lorentz {
            launch(
                device,
                "read_key_self",
                rows,
                &[
                    Arg::In(&inner_grad),
                    Arg::In(&pass.query_lift),
                    Arg::In(&pass.key_lift),
                    Arg::Out(&key_self),
                    Arg::Words(&dims),
                ],
            )?;
        }
        launch(
            device,
            "read_dq",
            q.elem_count(),
            &[
                Arg::In(&inner_grad),
                Arg::In(qs.buffer()),
                Arg::In(kvs.buffer()),
                Arg::In(&query_self),
                Arg::Out(&dq),
                Arg::Words(&dims),
            ],
        )?;
        launch(
            device,
            "read_dkv",
            kvt.elem_count(),
            &[
                Arg::In(&inner_grad),
                Arg::In(&pass.probabilities),
                Arg::In(qs.buffer()),
                Arg::In(kvs.buffer()),
                Arg::In(dys.buffer()),
                Arg::In(&key_self),
                Arg::Out(&dkv),
                Arg::Words(&dims),
            ],
        )?;
        if self.age {
            launch(
                device,
                "read_dage",
                self.heads * self.time,
                &[Arg::In(&ds), Arg::Out(&d_aux), Arg::Words(&dims)],
            )?;
        }
        if lorentz {
            launch(
                device,
                "read_dbeta",
                2 * self.heads,
                &[
                    Arg::In(&row_beta),
                    Arg::In(&row_offset),
                    Arg::Out(&d_aux),
                    Arg::Words(&dims),
                ],
            )?;
        }
        let d_aux = if aux_parts {
            metal_tensor(d_aux, device, aux.shape())
        } else {
            // The placeholder auxiliary input carries no gradient.
            Tensor::zeros(aux.shape(), DType::F32, aux.device())?
        };
        Ok((
            metal_tensor(dq, device, query.shape()),
            metal_tensor(dkv, device, kv.shape()),
            d_aux,
        ))
    }
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
        out.par_chunks_mut(time * value).enumerate().try_for_each(
            |(index, out)| -> candle_core::Result<()> {
                let block = self.block(query, kv, aux, tables.as_ref(), index);
                let mut scratch = Scratch::new(time);
                let mut probabilities = vec![0f32; TILE * time];
                for t0 in (0..time).step_by(TILE) {
                    let rows = TILE.min(time - t0);
                    self.tile(&block, t0, rows, &mut probabilities, &mut scratch)?;
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
                Ok(())
            },
        )?;
        Ok((
            CpuStorage::F32(out),
            Shape::from((self.batch, self.heads, time, value)),
        ))
    }

    #[cfg(feature = "metal")]
    fn metal_fwd(
        &self,
        s1: &MetalStorage,
        l1: &Layout,
        s2: &MetalStorage,
        l2: &Layout,
        s3: &MetalStorage,
        l3: &Layout,
    ) -> candle_core::Result<(MetalStorage, Shape)> {
        if !self.metal_covered() {
            return metal_via_host(self, [(s1, l1), (s2, l2), (s3, l3)]);
        }
        if s1.dtype() != DType::F32 || s2.dtype() != DType::F32 || s3.dtype() != DType::F32 {
            candle_core::bail!("Metal FusedRead requires F32 dtype");
        }
        if self.batch == 0 || self.heads == 0 || self.time == 0 || self.key == 0 || self.value == 0
        {
            candle_core::bail!("Metal FusedRead requires positive dimensions");
        }
        let (time, value) = (self.time, self.value);
        let expected_query = self.batch * self.heads * time * self.key;
        let expected_kv = self.batch * self.heads * time * (self.key + value);
        let expected_aux = fused_aux_len(
            self.batch, self.heads, time, self.score, self.null, self.age,
        )
        .max(1);
        if l1.shape().elem_count() != expected_query
            || l2.shape().elem_count() != expected_kv
            || l3.shape().elem_count() != expected_aux
        {
            candle_core::bail!(
                "Metal FusedRead input element counts do not match declared dimensions"
            );
        }
        if !l1.is_contiguous() || !l2.is_contiguous() || !l3.is_contiguous() {
            candle_core::bail!("Metal FusedRead requires contiguous inputs");
        }
        use crate::metal_stack_kernels::metal::{launch, Arg};
        let device = s1.device();
        let bytes = DType::F32.size_in_bytes();
        let (query, kv, aux) = (
            (s1.buffer(), l1.start_offset() * bytes),
            (s2.buffer(), l2.start_offset() * bytes),
            (s3.buffer(), l3.start_offset() * bytes),
        );
        let pass = self.metal_pass(device, query, kv, aux, false)?;
        let total = self.batch * self.heads * time * value;
        let out_buf = device.new_buffer(total, DType::F32, "fused_read_out")?;
        let dims = self.metal_dims();
        launch(
            device,
            "read_mix",
            total,
            &[
                Arg::In(&pass.probabilities),
                Arg::InAt(kv.0, kv.1),
                Arg::Out(&out_buf),
                Arg::Words(&dims),
            ],
        )?;
        Ok((
            MetalStorage::new(out_buf, device.clone(), total, DType::F32),
            Shape::from((self.batch, self.heads, time, value)),
        ))
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
        s3: &CudaStorage,
        l3: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        self.cuda_fwd_impl(s1, l1, s2, l2, s3, l3)
    }

    fn bwd(
        &self,
        query: &Tensor,
        kv: &Tensor,
        aux: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>, Option<Tensor>)> {
        #[cfg(feature = "cuda")]
        if let (Device::Cuda(device), true) = (query.device(), self.cuda_covered()) {
            let (dq, dkv, d_aux) = self.cuda_bwd(device, query, kv, aux, grad)?;
            return Ok((Some(dq), Some(dkv), Some(d_aux)));
        }
        #[cfg(feature = "metal")]
        if let Device::Metal(device) = query.device() {
            if self.metal_covered() {
                let (dq, dkv, d_aux) = self.metal_bwd(device, query, kv, aux, grad)?;
                return Ok((Some(dq), Some(dkv), Some(d_aux)));
            }
        }
        let q_values = query.flatten_all()?.to_vec1::<f32>()?;
        let kv_values = kv.flatten_all()?.to_vec1::<f32>()?;
        let aux_values = aux.flatten_all()?.to_vec1::<f32>()?;
        let d_out = grad.flatten_all()?.to_vec1::<f32>()?;
        let (time, key, value, width) = (self.time, self.key, self.value, self.width());
        // Lorentz and L2 both carry per-row self terms and the beta/offset.
        let lorentz = self.score.scaled();
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
            .map(|(index, (dq, dkv))| -> candle_core::Result<Partial> {
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
                    )?;
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
                                ReadScore::L2 => {
                                    let s = excess[j];
                                    partial.dbeta -= ds * (distance[j] - block.offset);
                                    partial.doffset += ds * block.beta;
                                    if s > L2_MIN_SQUARED {
                                        // s = |q|^2 + |k|^2 - 2 q.k and d = sqrt(s):
                                        // ds_ds = d score / d s.
                                        let ds_ds = -block.beta * ds / (2.0 * distance[j]);
                                        query_self[r] += 2.0 * ds_ds;
                                        key_self[j] += 2.0 * ds_ds;
                                        (-2.0 * ds_ds) as f32
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
                Ok(partial)
            })
            .collect::<candle_core::Result<Vec<Partial>>>()?;
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
    fused_read_selected(query, key, value, aux, score, null, age, rope, None)
}

/// [`fused_read`] whose rows softmax over a flock-selected subset of the
/// sources `0..=t` ([`FlockSelect`], selected by [`crate::flock::flock_select`]
/// with the sink at 0) when `select` is set: the sink, the last `window`
/// positions, the `k` best-scoring other sources by the same total score the
/// softmax uses (ties to the lowest position) and the NoRead slot, which is
/// not a position and is never selected away. Unkept sources have weight
/// exactly 0 and receive gradient exactly 0. With `None`, or where a row's
/// selection keeps every source, it is [`fused_read`], bit for bit. A row with
/// a non-finite score cannot be ranked and is an error.
#[allow(clippy::too_many_arguments)]
pub fn fused_read_selected(
    query: &Tensor,
    key: &Tensor,
    value: &Tensor,
    aux: &Tensor,
    score: ReadScore,
    null: bool,
    age: bool,
    rope: bool,
    select: Option<FlockSelect>,
) -> Result<Tensor> {
    if query.dtype() != DType::F32
        || key.dtype() != DType::F32
        || value.dtype() != DType::F32
        || aux.dtype() != DType::F32
    {
        return Err(invalid("fused_read requires F32 tensors"));
    }
    if let Some(select) = &select {
        validate_flock(select)?;
    }
    let (batch, heads, time, key_width) = query.dims4()?;
    let (b2, h2, t2, k2) = key.dims4()?;
    let (b3, h3, t3, value_width) = value.dims4()?;
    if (b2, h2, t2, k2) != (batch, heads, time, key_width) || (b3, h3, t3) != (batch, heads, time) {
        return Err(invalid(
            "fused read needs matching query, key and value shapes",
        ));
    }
    if batch == 0 || heads == 0 || time == 0 || key_width == 0 || value_width == 0 {
        return Err(invalid("fused read requires positive dimensions"));
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
        select,
    };
    if rope && key_width % 2 != 0 {
        return Err(invalid("RoPE needs an even head width"));
    }
    let kv = Tensor::cat(&[key, value], 3)?.contiguous()?;
    Ok(query.contiguous()?.apply_op3(&kv, &aux.contiguous()?, op)?)
}

/// Fused RecurrenceCore: branches, gates, and parameters -> out.
pub fn recurrence_core(
    branches: &Tensor,
    gates: &Tensor,
    parameters: &Tensor,
    batch: usize,
    time: usize,
    width: usize,
    rotation: bool,
    snap: Option<TransportSnap>,
) -> Result<Tensor> {
    recurrence_core_grouped(
        branches,
        gates,
        parameters,
        batch,
        time,
        width,
        rotation,
        RotationGroup::Quaternion,
        snap,
    )
}

/// [`recurrence_core`] with the transport restricted to `group` (with
/// rotation): [`RotationGroup::U1`] is the abelian control.
#[allow(clippy::too_many_arguments)]
pub fn recurrence_core_grouped(
    branches: &Tensor,
    gates: &Tensor,
    parameters: &Tensor,
    batch: usize,
    time: usize,
    width: usize,
    rotation: bool,
    group: RotationGroup,
    snap: Option<TransportSnap>,
) -> Result<Tensor> {
    if group == RotationGroup::U1 && (!rotation || snap.is_some()) {
        return Err(invalid(
            "the U(1) transport control needs rotation and no transport snap",
        ));
    }
    if batch == 0 || time == 0 || width == 0 || width % 4 != 0 {
        return Err(invalid(
            "recurrence_core requires positive dimensions with width divisible by 4",
        ));
    }
    if branches.dtype() != DType::F32
        || gates.dtype() != DType::F32
        || parameters.dtype() != DType::F32
    {
        return Err(invalid("recurrence_core requires F32 tensors"));
    }
    let lanes = width / 4;
    let gate_width = lanes + if rotation { 4 * lanes } else { 0 };
    let expected_branches = batch * time * 2 * width;
    let expected_gates = batch * time * gate_width;
    let expected_params = (CONVOLUTION_WIDTH + 1) * width + lanes;
    if branches.elem_count() != expected_branches
        || gates.elem_count() != expected_gates
        || parameters.elem_count() != expected_params
    {
        return Err(invalid(format!(
            "recurrence_core input size mismatch: branches expected {expected_branches} got {}, gates expected {expected_gates} got {}, params expected {expected_params} got {}",
            branches.elem_count(),
            gates.elem_count(),
            parameters.elem_count(),
        )));
    }
    Ok(branches.contiguous()?.apply_op3(
        &gates.contiguous()?,
        &parameters.contiguous()?,
        RecurrenceCore {
            batch,
            time,
            width,
            rotation,
            group,
            snap,
        },
    )?)
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

    #[cfg(feature = "metal")]
    fn metal_fwd(
        &self,
        s1: &MetalStorage,
        l1: &Layout,
        s2: &MetalStorage,
        l2: &Layout,
    ) -> candle_core::Result<(MetalStorage, Shape)> {
        if s1.dtype() != DType::F32 || s2.dtype() != DType::F32 {
            candle_core::bail!("Metal RMSNorm requires F32 dtype");
        }
        if l1.start_offset() != 0
            || !l1.is_contiguous()
            || l2.start_offset() != 0
            || !l2.is_contiguous()
        {
            candle_core::bail!("Metal kernel requires contiguous layout with zero start offset");
        }
        let width = l2.shape().elem_count();
        if width == 0 {
            candle_core::bail!("Metal RMSNorm weight must not be empty");
        }
        if l1.shape().dims().last() != Some(&width) {
            candle_core::bail!("Metal RMSNorm weight must match the last dimension");
        }
        let total = l1.shape().elem_count();
        if total % width != 0 {
            candle_core::bail!("Metal RMSNorm total element count must be divisible by width");
        }
        let rows = total / width;
        let device = s1.device();
        let out_buf = device.new_buffer(total, DType::F32, "rms_norm_out")?;
        crate::metal_stack_kernels::metal::call_rms_norm_fwd(
            device,
            s1.buffer(),
            s2.buffer(),
            &out_buf,
            rows,
            width,
            RMS_EPSILON as f32,
        )?;
        Ok((
            MetalStorage::new(out_buf, device.clone(), total, DType::F32),
            l1.shape().clone(),
        ))
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
        #[cfg(feature = "metal")]
        if let Device::Metal(device) = x.device() {
            if x.dtype() != DType::F32 || w.dtype() != DType::F32 || grad.dtype() != DType::F32 {
                candle_core::bail!("Metal RMSNorm backward requires F32 dtype");
            }
            let width = w.elem_count();
            if width == 0 {
                candle_core::bail!("Metal RMSNorm weight must not be empty");
            }
            if x.dims().last() != Some(&width) {
                candle_core::bail!("Metal RMSNorm weight must match the last dimension");
            }
            let total = x.elem_count();
            if total % width != 0 {
                candle_core::bail!("Metal RMSNorm total element count must be divisible by width");
            }
            let rows = total / width;

            let (x_storage, x_layout) = x.storage_and_layout();
            let (w_storage, w_layout) = w.storage_and_layout();
            let (g_storage, g_layout) = grad.storage_and_layout();

            if x_layout.start_offset() != 0
                || !x_layout.is_contiguous()
                || w_layout.start_offset() != 0
                || !w_layout.is_contiguous()
                || g_layout.start_offset() != 0
                || !g_layout.is_contiguous()
            {
                candle_core::bail!(
                    "Metal kernel requires contiguous layout with zero start offset"
                );
            }

            let dx_buf = device.new_buffer(total, DType::F32, "rms_norm_dx")?;
            let dw_buf = device.new_buffer(width, DType::F32, "rms_norm_dw")?;

            if let (Storage::Metal(x_ms), Storage::Metal(w_ms), Storage::Metal(g_ms)) =
                (&*x_storage, &*w_storage, &*g_storage)
            {
                crate::metal_stack_kernels::metal::call_rms_norm_bwd(
                    device,
                    x_ms.buffer(),
                    w_ms.buffer(),
                    g_ms.buffer(),
                    &dx_buf,
                    &dw_buf,
                    rows,
                    width,
                    RMS_EPSILON as f32,
                )?;
                let dx = Tensor::from_storage(
                    Storage::Metal(MetalStorage::new(dx_buf, device.clone(), total, DType::F32)),
                    x.shape().clone(),
                    candle_core::op::BackpropOp::none(),
                    false,
                );
                let dw = Tensor::from_storage(
                    Storage::Metal(MetalStorage::new(dw_buf, device.clone(), width, DType::F32)),
                    w.shape().clone(),
                    candle_core::op::BackpropOp::none(),
                    false,
                );
                return Ok((Some(dx), Some(dw)));
            }
        }
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

/// Fused RMSNorm: `x * w / sqrt(mean(x^2) + eps)` over the last dimension.
pub fn rms_norm(x: &Tensor, weight: &Tensor) -> Result<Tensor> {
    if x.dtype() != DType::F32 || weight.dtype() != DType::F32 {
        return Err(invalid("rms_norm requires F32 tensors"));
    }
    let width = weight.elem_count();
    if width == 0 {
        return Err(invalid("rms_norm weight must not be empty"));
    }
    if x.dims().last() != Some(&width) {
        return Err(invalid(
            "rms_norm weight must match last dimension of input",
        ));
    }
    Ok(x.contiguous()?.apply_op2(&weight.contiguous()?, RmsNorm)?)
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

    #[cfg(feature = "metal")]
    fn metal_fwd(
        &self,
        s1: &MetalStorage,
        l1: &Layout,
        s2: &MetalStorage,
        l2: &Layout,
    ) -> candle_core::Result<(MetalStorage, Shape)> {
        if s1.dtype() != DType::F32 || s2.dtype() != DType::F32 {
            candle_core::bail!("Metal SwiGLU requires F32 dtype");
        }
        if l1.shape() != l2.shape() {
            candle_core::bail!("SwiGLU inputs must match");
        }
        if l1.shape().elem_count() == 0 {
            candle_core::bail!("SwiGLU inputs must not be empty");
        }
        if l1.start_offset() != 0
            || !l1.is_contiguous()
            || l2.start_offset() != 0
            || !l2.is_contiguous()
        {
            candle_core::bail!("Metal kernel requires contiguous layout with zero start offset");
        }
        let total = l1.shape().elem_count();
        let device = s1.device();
        let out_buf = device.new_buffer(total, DType::F32, "swiglu_out")?;
        crate::metal_stack_kernels::metal::call_swiglu_fwd(
            device,
            s1.buffer(),
            s2.buffer(),
            &out_buf,
            total,
        )?;
        Ok((
            MetalStorage::new(out_buf, device.clone(), total, DType::F32),
            l1.shape().clone(),
        ))
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        cuda_ops::swiglu_fwd(s1, l1, s2, l2)
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
            let (first, second) = cuda_ops::swiglu_bwd(device, gate, up, grad)?;
            return Ok((Some(first), Some(second)));
        }
        #[cfg(feature = "metal")]
        if let Device::Metal(device) = gate.device() {
            if gate.dtype() != DType::F32 || up.dtype() != DType::F32 || grad.dtype() != DType::F32
            {
                candle_core::bail!("Metal SwiGLU backward requires F32 dtype");
            }
            if gate.shape() != up.shape() || gate.shape() != grad.shape() {
                candle_core::bail!("Metal SwiGLU backward inputs must match");
            }
            if gate.elem_count() == 0 {
                candle_core::bail!("Metal SwiGLU backward inputs must not be empty");
            }
            let total = gate.elem_count();

            let (g_storage, g_layout) = gate.storage_and_layout();
            let (u_storage, u_layout) = up.storage_and_layout();
            let (d_storage, d_layout) = grad.storage_and_layout();

            if g_layout.start_offset() != 0
                || !g_layout.is_contiguous()
                || u_layout.start_offset() != 0
                || !u_layout.is_contiguous()
                || d_layout.start_offset() != 0
                || !d_layout.is_contiguous()
            {
                candle_core::bail!(
                    "Metal kernel requires contiguous layout with zero start offset"
                );
            }

            let dg_buf = device.new_buffer(total, DType::F32, "swiglu_dg")?;
            let du_buf = device.new_buffer(total, DType::F32, "swiglu_du")?;

            if let (Storage::Metal(g_ms), Storage::Metal(u_ms), Storage::Metal(d_ms)) =
                (&*g_storage, &*u_storage, &*d_storage)
            {
                crate::metal_stack_kernels::metal::call_swiglu_bwd(
                    device,
                    g_ms.buffer(),
                    u_ms.buffer(),
                    d_ms.buffer(),
                    &dg_buf,
                    &du_buf,
                    total,
                )?;
                let dg = Tensor::from_storage(
                    Storage::Metal(MetalStorage::new(dg_buf, device.clone(), total, DType::F32)),
                    gate.shape().clone(),
                    candle_core::op::BackpropOp::none(),
                    false,
                );
                let du = Tensor::from_storage(
                    Storage::Metal(MetalStorage::new(du_buf, device.clone(), total, DType::F32)),
                    up.shape().clone(),
                    candle_core::op::BackpropOp::none(),
                    false,
                );
                return Ok((Some(dg), Some(du)));
            }
        }
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

/// Fused SwiGLU: `silu(gate) * up`.
pub fn swiglu(gate: &Tensor, up: &Tensor) -> Result<Tensor> {
    if gate.dtype() != DType::F32 || up.dtype() != DType::F32 {
        return Err(invalid("swiglu requires F32 tensors"));
    }
    if gate.shape() != up.shape() {
        return Err(invalid("swiglu gate and up shapes must match"));
    }
    if gate.elem_count() == 0 {
        return Err(invalid("swiglu inputs must not be empty"));
    }
    Ok(gate.contiguous()?.apply_op2(&up.contiguous()?, SwiGlu)?)
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

    #[cfg(feature = "metal")]
    fn metal_fwd(
        &self,
        storage: &MetalStorage,
        layout: &Layout,
    ) -> candle_core::Result<(MetalStorage, Shape)> {
        if storage.dtype() != DType::F32 {
            candle_core::bail!("Metal CrossEntropy requires F32 dtype");
        }
        if layout.start_offset() != 0 || !layout.is_contiguous() {
            candle_core::bail!("Metal kernel requires contiguous layout with zero start offset");
        }
        let (rows, vocabulary) = layout.shape().dims2()?;
        if rows == 0 || vocabulary == 0 {
            candle_core::bail!("CrossEntropy requires positive rows and vocabulary");
        }
        if self.targets.len() != rows {
            candle_core::bail!("CrossEntropy targets count must match rows");
        }
        let device = storage.device();
        let target_buf = device.new_buffer_with_data(&self.targets)?;
        let loss_per_row = device.new_buffer(rows, DType::F32, "cross_entropy_row_loss")?;

        crate::metal_stack_kernels::metal::call_cross_entropy_fwd(
            device,
            storage.buffer(),
            &target_buf,
            &loss_per_row,
            rows,
            vocabulary,
        )?;

        let row_losses_tensor = Tensor::from_storage(
            Storage::Metal(MetalStorage::new(
                loss_per_row,
                device.clone(),
                rows,
                DType::F32,
            )),
            Shape::from(rows),
            candle_core::op::BackpropOp::none(),
            false,
        );
        let row_losses = row_losses_tensor.to_vec1::<f32>()?;
        let mean = match &self.weights {
            None => {
                let total: f64 = row_losses.iter().map(|&v| f64::from(v)).sum();
                total / rows as f64
            }
            Some(weights) => {
                let total: f64 = row_losses
                    .iter()
                    .zip(weights.iter())
                    .map(|(&l, &w)| f64::from(l) * f64::from(w))
                    .sum();
                total / weights.iter().map(|&w| f64::from(w)).sum::<f64>()
            }
        };
        let out_buf = device.new_buffer_with_data(&[mean as f32])?;
        Ok((
            MetalStorage::new(out_buf, device.clone(), 1, DType::F32),
            Shape::from(()),
        ))
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        storage: &CudaStorage,
        layout: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        cuda_ops::cross_entropy_fwd(self, storage, layout)
    }

    fn bwd(
        &self,
        logits: &Tensor,
        _loss: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<Option<Tensor>> {
        #[cfg(feature = "cuda")]
        if let Device::Cuda(device) = logits.device() {
            return Ok(Some(cuda_ops::cross_entropy_bwd(
                self, device, logits, grad,
            )?));
        }
        let (rows, vocabulary) = logits.dims2()?;
        #[cfg(feature = "metal")]
        if self.weights.is_none() {
            if let Device::Metal(device) = logits.device() {
                if logits.dtype() != DType::F32 || grad.dtype() != DType::F32 {
                    candle_core::bail!("Metal CrossEntropy backward requires F32 dtype");
                }
                if rows == 0 || vocabulary == 0 {
                    candle_core::bail!("CrossEntropy requires positive rows and vocabulary");
                }
                if self.targets.len() != rows {
                    candle_core::bail!("CrossEntropy targets count must match rows");
                }
                let (l_storage, l_layout) = logits.storage_and_layout();
                if l_layout.start_offset() != 0 || !l_layout.is_contiguous() {
                    candle_core::bail!(
                        "Metal kernel requires contiguous layout with zero start offset"
                    );
                }
                let scale = (grad.to_scalar::<f32>()? / rows as f32) as f32;
                let target_buf = device.new_buffer_with_data(&self.targets)?;
                let total = rows * vocabulary;
                let grad_buf = device.new_buffer(total, DType::F32, "ce_grad")?;
                if let Storage::Metal(l_ms) = &*l_storage {
                    crate::metal_stack_kernels::metal::call_cross_entropy_bwd(
                        device,
                        l_ms.buffer(),
                        &target_buf,
                        &grad_buf,
                        rows,
                        vocabulary,
                        scale,
                    )?;
                    let g_tensor = Tensor::from_storage(
                        Storage::Metal(MetalStorage::new(
                            grad_buf,
                            device.clone(),
                            total,
                            DType::F32,
                        )),
                        logits.shape().clone(),
                        candle_core::op::BackpropOp::none(),
                        false,
                    );
                    return Ok(Some(g_tensor));
                }
            }
        }
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
    if logits.dtype() != DType::F32 {
        return Err(invalid("logits_cross_entropy requires F32 logits"));
    }
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

/// Fused CrossEntropy loss: mean cross entropy of logits vs target class indices.
pub fn cross_entropy(logits: &Tensor, targets: &[u32]) -> Result<Tensor> {
    logits_cross_entropy(logits, targets, None)
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

// ---------------------------------------------------------------------------
// Pointer-copy head: the mixture of the ordinary distribution and a copy of the
// input tokens at the attended positions.

/// `ln(1 + e^x)`, stable for large `|x|`.
fn softplus(x: f64) -> f64 {
    if x > 0.0 {
        x + (-x).exp().ln_1p()
    } else {
        x.exp().ln_1p()
    }
}

fn sigmoid_f64(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// The one f32 of a one-element tensor, as f64.
fn one_value(tensor: &Tensor) -> Result<f64> {
    tensor
        .flatten_all()?
        .to_vec1::<f32>()?
        .first()
        .map(|&value| f64::from(value))
        .ok_or_else(|| invalid("expected a one-element tensor"))
}

/// How a pointer scores and selects its sources for one op or one generation
/// step.
#[derive(Clone, Copy, Debug)]
struct PointerRule {
    /// Width of the pointer's query and key.
    dim: usize,
    score: ReadScore,
    /// The pointer's own selection; the reads' flock is not consulted.
    select: Option<PointerSelect>,
    /// The Lorentz scale `exp(pointer.log_beta)`; Dot has none.
    beta: f64,
    /// Exact prime routing, which replaces the scores and selection.
    route: Option<PrimeRoute>,
}

/// Query row `row` of a pointer's `side` (rows `[query | key | gate logit]`,
/// `2 * dim + 1` wide).
fn pointer_query(side: &[f32], dim: usize, row: usize) -> &[f32] {
    let at = row * (2 * dim + 1);
    &side[at..at + dim]
}

/// Key row `row` of a pointer's `side`.
fn pointer_key(side: &[f32], dim: usize, row: usize) -> &[f32] {
    let at = row * (2 * dim + 1) + dim;
    &side[at..at + dim]
}

/// `sqrt(1 + |x|^2)` in f64: the hyperboloid lift of a pointer query or key,
/// the lift of the fused read (`FusedRead::block`).
fn pointer_lift(x: &[f32]) -> f64 {
    (1.0 + x.iter().map(|&v| f64::from(v) * f64::from(v)).sum::<f64>()).sqrt()
}

/// The Lorentz terms of one pointer query and key, in the fused read's form:
/// the excess `e = lift(q) lift(k) - <q, k> - 1`, the distance `arcosh(1 + e)`
/// clamped below at `LORENTZ_MIN_EXCESS` ([`lorentz_distance`]), and the two
/// lifts. The arithmetic is f64 throughout (the read's inner products are f32
/// sums); the function is the same.
struct LorentzTerms {
    excess: f64,
    distance: f64,
    query_lift: f64,
    key_lift: f64,
}

fn lorentz_terms(query: &[f32], key: &[f32]) -> LorentzTerms {
    let (query_lift, key_lift) = (pointer_lift(query), pointer_lift(key));
    let inner: f64 = query
        .iter()
        .zip(key)
        .map(|(&a, &b)| f64::from(a) * f64::from(b))
        .sum();
    let excess = query_lift * key_lift - inner - 1.0;
    LorentzTerms {
        excess,
        distance: lorentz_distance(excess),
        query_lift,
        key_lift,
    }
}

/// The pointer's scores of position `t` (row `first + t` of `side`) over the
/// sources `0..=t`: `q_t . k_j / sqrt(dim)` for Dot, `-beta arcosh(1 + e)` for
/// Lorentz, rounded to f32 as the reads' scores are.
fn pointer_scores(side: &[f32], first: usize, t: usize, rule: &PointerRule) -> Vec<f32> {
    let query = pointer_query(side, rule.dim, first + t);
    match rule.score {
        ReadScore::Dot => {
            let scale = 1.0 / (rule.dim as f32).sqrt();
            (0..=t)
                .map(|j| dot(query, pointer_key(side, rule.dim, first + j)) * scale)
                .collect()
        }
        ReadScore::Lorentz => (0..=t)
            .map(|j| {
                let key = pointer_key(side, rule.dim, first + j);
                (-rule.beta * lorentz_terms(query, key).distance) as f32
            })
            .collect(),
        // Unreachable: PointerConfig::validate refuses an L2 pointer. The
        // flat distance is the forward value it would have.
        ReadScore::L2 => (0..=t)
            .map(|j| {
                let key = pointer_key(side, rule.dim, first + j);
                let squared: f64 = query
                    .iter()
                    .zip(key)
                    .map(|(&a, &b)| f64::from(a - b).powi(2))
                    .sum();
                (-rule.beta * l2_distance(squared)) as f32
            })
            .collect(),
    }
}

/// The pointer's attention over the sources `0..=t` (`t + 1 = scores.len()`):
/// the softmax of `scores` over the sources `select` keeps, exactly 0 on the
/// others, or over every source when `select` is `None`. The selection is the
/// shared selector's ([`crate::flock`]), taken on these same scores, and the
/// sums run in position order, so a selection that keeps every source equals
/// `None` bit for bit. A kept single source has weight 1.
fn pointer_weights(
    mut scores: Vec<f32>,
    select: Option<PointerSelect>,
) -> candle_core::Result<Vec<f64>> {
    if let Some(select) = select {
        let t = scores.len().saturating_sub(1);
        let selection = select.kept(&scores, t).map_err(selection_error)?;
        drop_unkept(&selection, &mut scores, &mut Vec::new());
    }
    let maximum = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let mut weights: Vec<f64> = scores
        .iter()
        .map(|&score| f64::from(score - maximum).exp())
        .collect();
    let total: f64 = weights.iter().sum();
    for weight in &mut weights {
        *weight /= total;
    }
    Ok(weights)
}

/// The pointer's attention of position `t` of the window whose first row is
/// row `first` of `side` (rows `[query | key | gate logit]`, `2 * dim + 1`
/// wide), by `rule`'s own score and selection (unkept sources exactly 0).
fn pointer_attention(
    side: &[f32],
    first: usize,
    t: usize,
    rule: &PointerRule,
    ids: &[u32],
) -> candle_core::Result<Vec<f64>> {
    match rule.route {
        // The window's tokens: `ids` is indexed like `side`'s rows.
        Some(route) => match ids.get(first..=first + t) {
            Some(window) if route.ranked => {
                ranked_attention(pointer_scores(side, first, t, rule), window, t, route)
            }
            Some(window) => Ok(route_attention(window, t, route)),
            None => candle_core::bail!("a routed pointer needs the window's token ids"),
        },
        None => pointer_weights(pointer_scores(side, first, t, rule), rule.select),
    }
}

/// One scored position of the mixture.
struct MixtureRow {
    /// The pointer's attention over sources `0..=t`.
    attention: Vec<f64>,
    /// `p_copy(target | t)`: exactly 0 when no source with attention holds the
    /// target.
    copy: f64,
    /// The gate `g_t`.
    gate: f64,
    /// `log((1 - g) softmax(z)[target] + g p_copy)`, with no floor on
    /// `p_copy`.
    log_mixture: f64,
    /// The shares of the mixture that came from each branch (sum to 1).
    generate_share: f64,
    copy_share: f64,
    /// Log-sum-exp of the logits row.
    lse: f64,
}

/// The mixture loss of a batch of windows: for a scored target `y_t`,
/// `NLL_t = -log((1 - g_t) softmax(z_t)[y_t] + g_t p_copy(y_t | t))`, averaged
/// with the response weights as [`CrossEntropy`] does. Inputs are the logits
/// `z` [rows, vocabulary], the pointer's per-row `[query | key | gate logit]`
/// [rows, 2 * dim + 1] and its Lorentz scale `beta` [1] (`exp(pointer.log_beta)`;
/// ignored by Dot, which gets a zero gradient for it); the backward is exact
/// and gives no work to rows of weight zero.
struct PointerMixture {
    time: usize,
    dim: usize,
    score: ReadScore,
    select: Option<PointerSelect>,
    /// Exact prime routing of the sources ([`PrimeRoute`]): no score gradient.
    route: Option<PrimeRoute>,
    /// The input token of every position: what the head copies.
    ids: Vec<u32>,
    targets: Vec<u32>,
    weights: Option<Vec<f32>>,
}

impl PointerMixture {
    fn weight(&self, row: usize) -> f64 {
        self.weights.as_ref().map_or(1.0, |w| f64::from(w[row]))
    }

    fn total(&self) -> f64 {
        match &self.weights {
            None => self.targets.len() as f64,
            Some(weights) => weights.iter().map(|&w| f64::from(w)).sum(),
        }
    }

    fn rule(&self, beta: f64) -> PointerRule {
        PointerRule {
            dim: self.dim,
            score: self.score,
            select: self.select,
            beta,
            route: self.route,
        }
    }

    /// Row `n` (window `n / time`, position `n % time`).
    fn evaluate(
        &self,
        logits: &[f32],
        side: &[f32],
        beta: f64,
        n: usize,
    ) -> candle_core::Result<MixtureRow> {
        let (first, t) = (n - n % self.time, n % self.time);
        let target = self.targets[n];
        let attention = pointer_attention(side, first, t, &self.rule(beta), &self.ids)?;
        let copy: f64 = attention
            .iter()
            .zip(&self.ids[first..=first + t])
            .filter(|(_, &id)| id == target)
            .map(|(&a, _)| a)
            .sum();
        let logit = f64::from(side[n * (2 * self.dim + 1) + 2 * self.dim]);
        let lse = row_log_sum_exp(logits);
        let generate = -softplus(logit) + f64::from(logits[target as usize]) - lse;
        // No kept source holds the target: the copy branch has probability 0
        // exactly, so the row is the generated probability alone (no floor).
        let copied = if copy > 0.0 {
            -softplus(-logit) + copy.ln()
        } else {
            f64::NEG_INFINITY
        };
        let high = generate.max(copied);
        let log_mixture = high + ((generate - high).exp() + (copied - high).exp()).ln();
        Ok(MixtureRow {
            attention,
            copy,
            gate: sigmoid_f64(logit),
            log_mixture,
            generate_share: (generate - log_mixture).exp(),
            copy_share: (copied - log_mixture).exp(),
            lse,
        })
    }

    fn check(
        &self,
        logits: &Layout,
        side: &Layout,
        beta: &Layout,
    ) -> candle_core::Result<(usize, usize)> {
        let (rows, vocabulary) = logits.shape().dims2()?;
        if side.shape().dims() != [rows, 2 * self.dim + 1]
            || beta.shape().dims() != [1usize]
            || self.ids.len() != rows
            || self.targets.len() != rows
            || self.weights.as_ref().is_some_and(|w| w.len() != rows)
            || self.time == 0
            || !rows.is_multiple_of(self.time)
        {
            candle_core::bail!("pointer mixture inputs disagree in shape");
        }
        Ok((rows, vocabulary))
    }
}

impl CustomOp3 for PointerMixture {
    fn name(&self) -> &'static str {
        "geometric-stack-pointer-mixture"
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
        let (rows, vocabulary) = self.check(l1, l2, l3)?;
        let (logits, side, beta) = (
            contiguous(s1, l1)?,
            contiguous(s2, l2)?,
            contiguous(s3, l3)?,
        );
        let beta = beta
            .first()
            .map(|&value| f64::from(value))
            .ok_or_else(|| candle_core::Error::msg("pointer mixture needs its scale"))?;
        // Each row's loss in row order, summed in that order: the mean does
        // not depend on how the threads split the rows.
        let losses: Vec<f64> = (0..rows)
            .into_par_iter()
            .map(|n| -> candle_core::Result<f64> {
                if self.weight(n) == 0.0 {
                    return Ok(0.0);
                }
                let row =
                    self.evaluate(&logits[n * vocabulary..(n + 1) * vocabulary], side, beta, n)?;
                Ok(-self.weight(n) * row.log_mixture)
            })
            .collect::<candle_core::Result<_>>()?;
        let sum: f64 = losses.iter().sum();
        Ok((
            CpuStorage::F32(vec![(sum / self.total()) as f32]),
            Shape::from(()),
        ))
    }

    #[cfg(feature = "metal")]
    fn metal_fwd(
        &self,
        s1: &MetalStorage,
        l1: &Layout,
        s2: &MetalStorage,
        l2: &Layout,
        s3: &MetalStorage,
        l3: &Layout,
    ) -> candle_core::Result<(MetalStorage, Shape)> {
        metal_via_host(self, [(s1, l1), (s2, l2), (s3, l3)])
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
        s3: &CudaStorage,
        l3: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        cuda_ops::via_host3(self, [(s1, l1), (s2, l2), (s3, l3)])
    }

    fn bwd(
        &self,
        logits: &Tensor,
        side: &Tensor,
        beta: &Tensor,
        _loss: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>, Option<Tensor>)> {
        let (rows, vocabulary) = logits.dims2()?;
        let (time, dim) = (self.time, self.dim);
        let stride = 2 * dim + 1;
        let scale = 1.0 / (dim as f64).sqrt();
        let lorentz = self.score == ReadScore::Lorentz;
        let z = logits.flatten_all()?.to_vec1::<f32>()?;
        let s = side.flatten_all()?.to_vec1::<f32>()?;
        let beta_value = beta
            .flatten_all()?
            .to_vec1::<f32>()?
            .first()
            .map(|&value| f64::from(value))
            .ok_or_else(|| candle_core::Error::msg("pointer mixture needs its scale"))?;
        let grad = f64::from(grad.to_scalar::<f32>()?);
        let total = self.total();
        let mut d_logits = vec![0f32; rows * vocabulary];
        let mut d_side = vec![0f32; rows * stride];
        // d loss / d score of every (row, source) for Dot, and for Lorentz
        // d loss / d excess (the score's derivative through `-beta arcosh(1 +
        // e)` applied), for the keys' gradients.
        let mut d_scores = vec![0f64; rows * time];
        // d loss / d beta of every row.
        let mut d_beta_rows = vec![0f64; rows];
        d_logits
            .par_chunks_mut(vocabulary)
            .zip(d_side.par_chunks_mut(stride))
            .zip(d_scores.par_chunks_mut(time))
            .zip(d_beta_rows.par_chunks_mut(1))
            .enumerate()
            .filter(|(n, _)| self.weight(*n) != 0.0)
            .try_for_each(
                |(n, (((d_z, d_row), d_score), d_beta))| -> candle_core::Result<()> {
                    let row =
                        self.evaluate(&z[n * vocabulary..(n + 1) * vocabulary], &s, beta_value, n)?;
                    let first = n - n % time;
                    let target = self.targets[n];
                    let c = grad * self.weight(n) / total;
                    // d NLL / d z_v = share_generate (softmax_v - [v = target]).
                    let k = c * row.generate_share;
                    for (v, slot) in d_z.iter_mut().enumerate() {
                        *slot = (k * (f64::from(z[n * vocabulary + v]) - row.lse).exp()) as f32;
                    }
                    d_z[target as usize] -= k as f32;
                    // d NLL / d gate logit = share_generate g - share_copy (1 - g),
                    // which is `g` when no source holds the target.
                    d_row[2 * dim] = (c
                        * (row.generate_share * row.gate - row.copy_share * (1.0 - row.gate)))
                        as f32;
                    // d loss / d score_j = -(g / mixture) a_j (m_j - p_copy).
                    // g / mixture overflows once the mixture is below about
                    // exp(-709.78), so the product is taken through the copy
                    // share g p_copy / mixture (at most 1): -copy_share
                    // (a_j / p_copy) (1 - p_copy) for a source holding the
                    // target (a_j / p_copy is at most 1 there) and copy_share
                    // a_j for the others. When no source holds the target,
                    // p_copy is 0 whatever the scores are, so this row gives
                    // them no gradient (and none is formed: zero times the
                    // overflow would be NaN).
                    // An exact route has no learned score; a ranked one ranks by it.
                    let scored = self.route.is_none_or(|route| route.ranked);
                    let sources: &[f64] = if row.copy > 0.0 && scored {
                        &row.attention
                    } else {
                        &[]
                    };
                    let query = pointer_query(&s, dim, n);
                    let mut d_query = vec![0f64; dim];
                    let mut d_beta_sum = 0.0;
                    for (j, &a) in sources.iter().enumerate() {
                        if a == 0.0 {
                            continue;
                        }
                        let d_source = if self.ids[first + j] == target {
                            -c * row.copy_share * (a / row.copy) * (1.0 - row.copy)
                        } else {
                            c * row.copy_share * a
                        };
                        let key = pointer_key(&s, dim, first + j);
                        if lorentz {
                            let terms = lorentz_terms(query, key);
                            d_beta_sum -= d_source * terms.distance;
                            // The clamped distance carries no gradient to the
                            // query and key, as in the fused read.
                            if terms.excess > LORENTZ_MIN_EXCESS {
                                let d_excess = -beta_value * d_source
                                    / (terms.excess * (terms.excess + 2.0)).sqrt();
                                d_score[j] = d_excess;
                                // e = lift_q lift_k - <q, k> - 1, so
                                // d e / d q = (lift_k / lift_q) q - k.
                                let own = d_excess * terms.key_lift / terms.query_lift;
                                for ((slot, &q), &kv) in d_query.iter_mut().zip(query).zip(key) {
                                    *slot += own * f64::from(q) - d_excess * f64::from(kv);
                                }
                            }
                        } else {
                            d_score[j] = d_source;
                            for (slot, &value) in d_query.iter_mut().zip(key) {
                                *slot += d_source * scale * f64::from(value);
                            }
                        }
                    }
                    for (slot, value) in d_row[..dim].iter_mut().zip(d_query) {
                        *slot = value as f32;
                    }
                    if let Some(slot) = d_beta.first_mut() {
                        *slot = d_beta_sum;
                    }
                    Ok(())
                },
            )?;
        d_side
            .par_chunks_mut(stride)
            .enumerate()
            .for_each(|(n, d_row)| {
                let (first, j) = (n - n % time, n % time);
                let key = pointer_key(&s, dim, n);
                let key_lift = if lorentz { pointer_lift(key) } else { 1.0 };
                let mut d_key = vec![0f64; dim];
                for t in j..time {
                    let c = d_scores[(first + t) * time + j];
                    if c == 0.0 {
                        continue;
                    }
                    let query = pointer_query(&s, dim, first + t);
                    if lorentz {
                        // d e / d k = (lift_q / lift_k) k - q.
                        let own = c * pointer_lift(query) / key_lift;
                        for ((slot, &q), &kv) in d_key.iter_mut().zip(query).zip(key) {
                            *slot += own * f64::from(kv) - c * f64::from(q);
                        }
                    } else {
                        for (slot, &value) in d_key.iter_mut().zip(query) {
                            *slot += c * scale * f64::from(value);
                        }
                    }
                }
                for (slot, value) in d_row[dim..2 * dim].iter_mut().zip(d_key) {
                    *slot = value as f32;
                }
            });
        let d_beta: f64 = d_beta_rows.iter().sum();
        Ok((
            Some(Tensor::from_vec(d_logits, logits.shape(), logits.device())?),
            Some(Tensor::from_vec(d_side, side.shape(), side.device())?),
            Some(Tensor::from_vec(
                vec![d_beta as f32],
                beta.shape(),
                beta.device(),
            )?),
        ))
    }
}

/// What the pointer head did at one scored position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerRowStats {
    /// The gate `g_t`.
    pub gate: f64,
    /// `p_copy(target | t)` before the gate.
    pub copy_mass: f64,
    /// Whether the most attended source (the lowest on a tie) holds the target.
    pub hit: bool,
    /// Whether any source with attention holds the target: what a hit can
    /// reach.
    pub reachable: bool,
}

/// Per-position scores of a batch: the negative log-likelihood of each target
/// (nats; the mixture's for a pointer model) and, for a pointer model, what its
/// head did at each scored position.
#[derive(Clone, Debug)]
pub struct TargetScores {
    pub nll: Vec<f64>,
    pub pointer: Option<Vec<Option<PointerRowStats>>>,
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
    fn target_nll_with_explicit_head_changes_when_head_modified_leaving_embeddings_fixed(
    ) -> Result<()> {
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 32,
            width: 16,
            heads: 2,
            mlp_hidden: 32,
            context: 16,
            pattern: "r".into(),
            read: ReadScore::Dot,
            rotation: false,
            rotation_group: Default::default(),
            seed: 42,
            memory: None,
            select: None,
            pointer: None,
        };
        let model = StackModel::new(config, &Device::Cpu)?;
        let ids = vec![1, 2, 3, 4];
        let targets = vec![2, 3, 4, 5];
        let head1 = model.variables()["embedding.weight"].as_tensor().clone();
        let nll1 = model.target_nll_with_head(&ids, &targets, &head1, 1, 4)?;

        // Perturb only target row 5 in head2
        let mut head2_vec = head1.flatten_all()?.to_vec1::<f32>()?;
        for j in 0..16 {
            head2_vec[5 * 16 + j] += 2.0;
        }
        let head2 = Tensor::from_vec(head2_vec, (32, 16), &Device::Cpu)?;
        let nll2 = model.target_nll_with_head(&ids, &targets, &head2, 1, 4)?;

        // Target index 3 (target 5) must change
        assert_ne!(nll1[3], nll2[3]);
        // Embedding variables must remain completely identical
        assert_eq!(
            model.variables()["embedding.weight"]
                .as_tensor()
                .flatten_all()?
                .to_vec1::<f32>()?,
            head1.flatten_all()?.to_vec1::<f32>()?
        );
        Ok(())
    }

    #[test]
    fn target_nll_with_head_rejects_malformed_head_shapes() -> Result<()> {
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 32,
            width: 16,
            heads: 2,
            mlp_hidden: 32,
            context: 16,
            pattern: "r".into(),
            read: ReadScore::Dot,
            rotation: false,
            rotation_group: Default::default(),
            seed: 42,
            memory: None,
            select: None,
            pointer: None,
        };
        let model = StackModel::new(config, &Device::Cpu)?;
        let ids = vec![1, 2];
        let targets = vec![2, 3];
        // Malformed head: [1, 16] instead of [32, 16]
        let bad_head = Tensor::zeros((1, 16), DType::F32, &Device::Cpu)?;
        let res = model.target_nll_with_head(&ids, &targets, &bad_head, 1, 2);
        assert!(
            res.is_err(),
            "target_nll_with_head must reject [1, 16] head shape"
        );

        // Malformed head: [32, 8] instead of [32, 16]
        let bad_width_head = Tensor::zeros((32, 8), DType::F32, &Device::Cpu)?;
        let res2 = model.target_nll_with_head(&ids, &targets, &bad_width_head, 1, 2);
        assert!(
            res2.is_err(),
            "target_nll_with_head must reject [32, 8] head shape"
        );

        Ok(())
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
        reference_read_masked(query, key, value, null, age, lorentz, None)
    }

    /// [`reference_read`] with an additive mask [batch, heads, time, time] on
    /// the scores (0 keeps a source, -inf drops it), for a fixed selection.
    fn reference_read_masked(
        query: &Tensor,
        key: &Tensor,
        value: &Tensor,
        null: Option<&Tensor>,
        age: Option<&Tensor>,
        lorentz: Option<(&Tensor, &Tensor)>,
        keep: Option<&Tensor>,
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
        if let Some(keep) = keep {
            scores = scores.add(keep)?;
        }
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

    /// The L2 read composed from Candle operations, its distance taken
    /// directly from `q - k` (not the fused read's norm expansion).
    fn reference_l2_read(
        query: &Tensor,
        key: &Tensor,
        value: &Tensor,
        null: &Tensor,
        age: &Tensor,
        beta: &Tensor,
        offset: &Tensor,
    ) -> Result<Tensor> {
        let (batch, heads, time, _) = query.dims4()?;
        let squared = query
            .unsqueeze(3)?
            .broadcast_sub(&key.unsqueeze(2)?)?
            .sqr()?
            .sum(4)?
            .clamp(L2_MIN_SQUARED as f32, f32::MAX)?;
        let mut scores = squared
            .sqrt()?
            .broadcast_sub(&offset.reshape((1, heads, 1, 1))?)?
            .broadcast_mul(&beta.reshape((1, heads, 1, 1))?)?
            .neg()?;
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
        let index = Tensor::from_vec(ages, time * time, &cpu())?;
        let table = age
            .index_select(&index, 1)?
            .reshape((1, heads, time, time))?;
        scores = scores
            .broadcast_add(&table)?
            .broadcast_add(&Tensor::from_vec(mask, (1, 1, time, time), &cpu())?)?;
        let scores = Tensor::cat(&[&null.reshape((batch, heads, time, 1))?, &scores], 3)?;
        let values = Tensor::cat(
            &[
                &Tensor::zeros((batch, heads, 1, value.dim(3)?), DType::F32, &cpu())?,
                value,
            ],
            2,
        )?;
        Ok(candle_nn::ops::softmax(&scores, 3)?.matmul(&values)?)
    }

    fn read_shape(score: ReadScore, time: usize, width: usize, value_width: usize) -> Result<()> {
        let mut rng = Initializer(match score {
            ReadScore::Dot => 3,
            ReadScore::Lorentz => 5,
            ReadScore::L2 => 7,
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
        // Lorentz and L2 carry beta and offset.
        let lorentz = score.scaled();
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
        let reference = if score == ReadScore::L2 {
            reference_l2_read(
                q.as_tensor(),
                k.as_tensor(),
                v.as_tensor(),
                null.as_tensor(),
                age.as_tensor(),
                beta.as_tensor(),
                offset.as_tensor(),
            )?
        } else {
            reference_read(
                q.as_tensor(),
                k.as_tensor(),
                v.as_tensor(),
                Some(null.as_tensor()),
                Some(age.as_tensor()),
                lorentz.then_some((beta.as_tensor(), offset.as_tensor())),
            )?
        };
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

    /// The flat L2 control: the fused forward against a reference that takes
    /// the distance directly from `q - k`, every gradient coordinate against
    /// autograd through it, and central finite differences.
    #[test]
    fn fused_l2_read_matches_reference_and_finite_differences() -> Result<()> {
        read_case(ReadScore::L2)
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
            let composed = model.composed_recurrence(&p, 0, &x, ComposedTransport::Free)?;
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
                .composed_recurrence(&p, 0, &x, ComposedTransport::Free)?
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

    // -----------------------------------------------------------------------
    // Matched ablation controls: the U(1) transport and the flat L2 read.

    /// The U(1) control's fused core against its Candle composition (forward
    /// and every parameter gradient), against central finite differences on
    /// the op's inputs, and with exactly zero gradient on every raw `j`, `k`.
    #[test]
    fn u1_recurrence_matches_composed_mixer_and_finite_differences() -> Result<()> {
        let mut config = tiny(StackArch::Geometric, "r", ReadScore::Lorentz, true);
        config.rotation_group = RotationGroup::U1;
        config.seed = 41;
        let model = StackModel::new(config, &cpu())?;
        for name in ["layers.00.rec.gate.weight", "layers.00.rec.conv.weight"] {
            let var = &model.variables()[name];
            var.set(&random(&mut Initializer(43), var.dims(), 0.5))?;
        }
        let x = random(&mut Initializer(47), &[2, 9, 16], 1.0);
        let p = model.params()?;
        let fused = model.recurrence(&p, 0, &x, &mut None)?;
        let composed = model.composed_recurrence(&p, 0, &x, ComposedTransport::Free)?;
        let gap = fused.sub(&composed)?.abs()?.max_all()?.to_scalar::<f32>()?;
        assert!(gap < 1e-5, "u1: fused recurrence differs by {gap}");
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
        let a = model
            .recurrence(&p, 0, &x, &mut None)?
            .mul(&weights)?
            .sum_all()?
            .backward()?;
        let b = model
            .composed_recurrence(&p, 0, &x, ComposedTransport::Free)?
            .mul(&weights)?
            .sum_all()?
            .backward()?;
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
                "u1: {name} gradient differs by {gap} (relative)"
            );
        }
        // The rotation rows' j and k gate weights and biases receive gradient
        // exactly zero (rows lanes + 4 lane + 2, + 3).
        let lanes = 4;
        let gate_weight = a
            .get(vars[1].as_tensor())
            .ok_or_else(|| invalid("gate weight gradient"))?
            .to_vec2::<f32>()?;
        let gate_bias = a
            .get(vars[2].as_tensor())
            .ok_or_else(|| invalid("gate bias gradient"))?
            .to_vec1::<f32>()?;
        let mut live = 0;
        for lane in 0..lanes {
            for k in 0..4 {
                let row = lanes + 4 * lane + k;
                if k >= 2 {
                    assert!(gate_weight[row].iter().all(|&g| g == 0.0), "row {row}");
                    assert_eq!(gate_bias[row], 0.0, "bias {row}");
                } else if gate_bias[row] != 0.0 {
                    live += 1;
                }
            }
        }
        assert!(live > 0, "the a and b components must carry gradient");

        // Central finite differences on the op's inputs.
        let (batch, time, width) = (2, 7, 8);
        let lanes = width / 4;
        let mut rng = Initializer(59);
        let branches = Var::from_tensor(&random(&mut rng, &[batch, time, 2 * width], 1.0))?;
        let gates = Var::from_tensor(&random(&mut rng, &[batch, time, lanes + width], 1.0))?;
        let parameters = Var::from_tensor(&random(
            &mut rng,
            &[(CONVOLUTION_WIDTH + 1) * width + lanes],
            0.5,
        ))?;
        let weights = random(&mut rng, &[batch, time, width], 1.0);
        let loss = || -> Result<Tensor> {
            Ok(recurrence_core_grouped(
                branches.as_tensor(),
                gates.as_tensor(),
                parameters.as_tensor(),
                batch,
                time,
                width,
                true,
                RotationGroup::U1,
                None,
            )?
            .mul(&weights)?
            .sum_all()?)
        };
        check_gradient(
            &[branches.clone(), gates.clone(), parameters.clone()],
            loss,
            1e-2,
        )?;
        // Only the gates in isolation, so the rotation path is not swamped.
        check_gradient(std::slice::from_ref(&gates), loss, 1e-2)?;
        let d_gates = loss()?
            .backward()?
            .get(gates.as_tensor())
            .ok_or_else(|| invalid("gate gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        for (index, &g) in d_gates.iter().enumerate() {
            let column = index % (lanes + width);
            if column >= lanes && (column - lanes) % 4 >= 2 {
                assert_eq!(g, 0.0, "gate column {column}");
            }
        }
        // A U(1) core refuses a snap and a stack without rotation.
        assert!(recurrence_core_grouped(
            branches.as_tensor(),
            gates.as_tensor(),
            parameters.as_tensor(),
            batch,
            time,
            width,
            true,
            RotationGroup::U1,
            Some(TransportSnap::Icosian),
        )
        .is_err());
        Ok(())
    }

    /// U(1) lane transitions commute (any order of a lane's transitions gives
    /// the same product), quaternion ones do not, the two groups' outputs
    /// differ on the same inputs, and their parameter counts are equal.
    #[test]
    fn u1_transitions_commute_and_differ_from_quaternion() -> Result<()> {
        let (batch, time, width) = (1, 9, 16);
        let lanes = width / 4;
        let mut rng = Initializer(61);
        let branches = random(&mut rng, &[batch, time, 2 * width], 1.0);
        let gates = random(&mut rng, &[batch, time, lanes + width], 1.0);
        let parameters = random(&mut rng, &[(CONVOLUTION_WIDTH + 1) * width + lanes], 0.5);
        let core = |group| RecurrenceCore {
            batch,
            time,
            width,
            rotation: true,
            group,
            snap: None,
        };
        let (b, g, p) = (
            branches.flatten_all()?.to_vec1::<f32>()?,
            gates.flatten_all()?.to_vec1::<f32>()?,
            parameters.to_vec1::<f32>()?,
        );
        let product = |qs: &[[f32; 4]]| {
            qs.iter()
                .fold([1f32, 0.0, 0.0, 0.0], |acc, &q| quaternion_product(acc, q))
        };
        let mut worst_quaternion = 0f32;
        for group in [RotationGroup::U1, RotationGroup::Quaternion] {
            let op = core(group);
            let (_, _, transitions) = op.window(&b, &g, &p, &op.log_a(&p));
            for lane in 0..lanes {
                let qs: Vec<[f32; 4]> = (0..time)
                    .map(|t| {
                        let tr = transitions[t * lanes + lane];
                        tr.rotation.map(|v| v * tr.lambda)
                    })
                    .collect();
                for x in &qs {
                    for y in &qs {
                        let (xy, yx) = (quaternion_product(*x, *y), quaternion_product(*y, *x));
                        let gap = (0..4).map(|k| (xy[k] - yx[k]).abs()).fold(0f32, f32::max);
                        match group {
                            RotationGroup::U1 => {
                                assert_eq!(x[2], 0.0);
                                assert_eq!(x[3], 0.0);
                                assert_eq!(xy, yx, "u1 transitions commute exactly");
                            }
                            RotationGroup::Quaternion => {
                                worst_quaternion = worst_quaternion.max(gap)
                            }
                        }
                    }
                }
                if group == RotationGroup::U1 {
                    let mut reversed = qs.clone();
                    reversed.reverse();
                    let (forward, backward) = (product(&qs), product(&reversed));
                    for k in 0..4 {
                        assert!(
                            (forward[k] - backward[k]).abs() <= 1e-6 * forward[k].abs().max(1e-6),
                            "lane {lane}: ordered product {forward:?} reversed {backward:?}"
                        );
                    }
                }
            }
        }
        assert!(
            worst_quaternion > 1e-3,
            "quaternion transitions should not commute ({worst_quaternion})"
        );
        let run = |group| -> Result<Tensor> {
            recurrence_core_grouped(
                &branches,
                &gates,
                &parameters,
                batch,
                time,
                width,
                true,
                group,
                None,
            )
        };
        let gap = run(RotationGroup::U1)?
            .sub(&run(RotationGroup::Quaternion)?)?
            .abs()?
            .max_all()?
            .to_scalar::<f32>()?;
        assert!(gap > 1e-3, "u1 output equals quaternion output ({gap})");
        // Parameter counts: tiny and the ladder's 288-wide stack.
        let mut quaternion = tiny(StackArch::Geometric, "rra", ReadScore::Lorentz, true);
        let mut u1 = quaternion.clone();
        u1.rotation_group = RotationGroup::U1;
        assert_eq!(quaternion.parameter_count()?, u1.parameter_count()?);
        assert_eq!(quaternion.shapes(), u1.shapes());
        quaternion = StackConfig::geometric_matched("rrarra", ReadScore::Lorentz, true, 1)?;
        u1 = quaternion.clone();
        u1.rotation_group = RotationGroup::U1;
        assert_eq!(quaternion.parameter_count()?, u1.parameter_count()?);
        // U(1) needs rotation, and refuses the snap and the served plan.
        let mut no_rotation = u1.clone();
        no_rotation.rotation = false;
        assert!(no_rotation.validate().is_err());
        assert!(TransportSnap::Icosian.check(&u1).is_err());
        assert!(served_plan(&u1).is_err());
        Ok(())
    }

    /// The L2 read on a hand example: scores `-beta (|q - k| - offset)`, so
    /// row 1 weighs its sources by `softmax(-beta (5 - o), -beta (4 - o))`.
    #[test]
    fn l2_read_scores_equal_negative_beta_distance_on_a_hand_example() -> Result<()> {
        let (beta, offset) = (0.5f32, 1.0f32);
        // [batch 1, heads 1, time 2, width 2].
        let query = Tensor::from_vec(vec![0f32, 0.0, 3.0, 4.0], (1, 1, 2, 2), &cpu())?;
        let key = Tensor::from_vec(vec![0f32, 0.0, 3.0, 0.0], (1, 1, 2, 2), &cpu())?;
        let value = Tensor::from_vec(vec![1f32, 0.0, 0.0, 1.0], (1, 1, 2, 2), &cpu())?;
        let aux = Tensor::from_vec(vec![beta, offset], 2, &cpu())?;
        let out = fused_read(
            &query,
            &key,
            &value,
            &aux,
            ReadScore::L2,
            false,
            false,
            false,
        )?
        .flatten_all()?
        .to_vec1::<f32>()?;
        // Row 0 reads only source 0; row 1: distances 5 (to k0) and 4 (to k1).
        let (s0, s1) = (
            -f64::from(beta) * (5.0 - f64::from(offset)),
            -f64::from(beta) * (4.0 - f64::from(offset)),
        );
        let p1 = 1.0 / (1.0 + (s0 - s1).exp());
        let expected = [1.0, 0.0, 1.0 - p1, p1];
        for (got, want) in out.iter().zip(expected) {
            assert!(
                (f64::from(*got) - want).abs() < 1e-6,
                "{out:?} against {expected:?}"
            );
        }
        // The distance is clamped at L2_MIN_SQUARED: identical q and k score
        // -beta (sqrt(1e-7) - offset) and stay finite.
        assert!((l2_distance(0.0) - L2_MIN_SQUARED.sqrt()).abs() < 1e-15);
        assert!((l2_distance(25.0) - 5.0).abs() < 1e-15);
        // Parameter count and shapes equal Lorentz's.
        let lorentz = tiny(StackArch::Geometric, "rra", ReadScore::Lorentz, true);
        let mut l2 = lorentz.clone();
        l2.read = ReadScore::L2;
        assert_eq!(lorentz.shapes(), l2.shapes());
        assert_eq!(lorentz.parameter_count()?, l2.parameter_count()?);
        let lorentz = StackConfig::geometric_matched("rrarra", ReadScore::Lorentz, true, 1)?;
        let l2 = StackConfig::geometric_matched("rrarra", ReadScore::L2, true, 1)?;
        assert_eq!(lorentz.parameter_count()?, l2.parameter_count()?);
        assert_eq!(lorentz.mlp_hidden, l2.mlp_hidden);
        // L2 is a read score only: the pointer and the served plan refuse it.
        let mut pointer = PointerConfig::new(8);
        pointer.score = ReadScore::L2;
        assert!(pointer.validate().is_err());
        assert!(served_plan(&l2).is_err());
        // An L2 stack trains end to end on the CPU: loss and gradients finite.
        let model = StackModel::new(
            tiny(StackArch::Geometric, "ra", ReadScore::L2, true),
            &cpu(),
        )?;
        let ids: Vec<u32> = (0..2 * 8).map(|i| (i * 7 % 37) as u32).collect();
        let logits = model.forward(&ids, 2, 8)?;
        let targets: Vec<u32> = ids.iter().map(|&i| (i + 1) % 37).collect();
        let loss = cross_entropy(&logits, &targets)?;
        let grads = loss.backward()?;
        assert!(loss.to_scalar::<f32>()?.is_finite());
        for name in ["layers.01.read.log_beta", "layers.01.read.offset"] {
            let g = grads
                .get(model.variables()[name].as_tensor())
                .ok_or_else(|| invalid(format!("no gradient for {name}")))?
                .to_vec1::<f32>()?;
            assert!(g.iter().all(|v| v.is_finite()) && g.iter().any(|&v| v != 0.0));
        }
        Ok(())
    }

    /// Configurations saved before the controls existed (no `rotation_group`)
    /// load as quaternion transport and save byte for byte as before; the
    /// controls round-trip. (The quaternion core's forward and gradients stay
    /// bit-identical to the pre-snap core:
    /// `without_a_snap_the_core_is_bit_identical_to_the_pre_snap_core`.)
    #[test]
    fn configs_without_control_fields_load_unchanged() -> Result<()> {
        let config = tiny(StackArch::Geometric, "rra", ReadScore::Lorentz, true);
        let saved = serde_json::to_string(&config).map_err(|e| invalid(e.to_string()))?;
        assert!(!saved.contains("rotation_group"), "{saved}");
        // A config.json as written before this field existed.
        let legacy = r#"{"arch":"geometric","vocab_size":37,"width":16,"heads":2,"mlp_hidden":24,"context":12,"pattern":"rra","read":"lorentz","rotation":true,"seed":5}"#;
        let loaded: StackConfig =
            serde_json::from_str(legacy).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(loaded, config);
        assert_eq!(loaded.rotation_group, RotationGroup::Quaternion);
        assert_eq!(
            serde_json::to_string(&loaded).map_err(|e| invalid(e.to_string()))?,
            legacy
        );
        // The same model from either config computes bit-identical logits.
        let ids: Vec<u32> = (0..2 * 8).map(|i| (i * 5 % 37) as u32).collect();
        let a = StackModel::new(config.clone(), &cpu())?.forward(&ids, 2, 8)?;
        let b = StackModel::new(loaded, &cpu())?.forward(&ids, 2, 8)?;
        assert_eq!(
            a.flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            b.flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>()
        );
        // The controls serialize their fields and round-trip.
        let mut control = config;
        control.read = ReadScore::L2;
        control.rotation_group = RotationGroup::U1;
        let text = serde_json::to_string(&control).map_err(|e| invalid(e.to_string()))?;
        assert!(text.contains(r#""read":"l2""#) && text.contains(r#""rotation_group":"u1""#));
        let back: StackConfig = serde_json::from_str(&text).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(back, control);
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
            rotation_group: Default::default(),
            seed: 5,
            memory: None,
            select: None,
            pointer: None,
        }
    }

    fn binding_rows(layer: usize) -> ReadBindingTarget {
        ReadBindingTarget {
            layer,
            head: 1,
            rows: vec![
                ReadBinding {
                    batch: 0,
                    query: 6,
                    sources: vec![1],
                },
                ReadBinding {
                    batch: 1,
                    query: 5,
                    sources: vec![0, 2],
                },
            ],
        }
    }

    fn tiny_address_model() -> Result<StackModel> {
        let mut model = StackModel::new(
            tiny(StackArch::Geometric, "rra", ReadScore::Lorentz, true),
            &cpu(),
        )?;
        model.set_read_identity_latch(ReadIdentityLatch::Held)?;
        model.set_geometric_address(GeometricAddressConfig::new(
            model.config.width,
            model.config.heads,
        )?)?;
        Ok(model)
    }

    fn tiny_span_model() -> Result<StackModel> {
        tiny_span_model_with_width(16)
    }

    fn tiny_span_model_with_width(width: usize) -> Result<StackModel> {
        let mut config = tiny(StackArch::Geometric, "rra", ReadScore::Lorentz, true);
        config.width = width;
        let mut model = StackModel::new(config, &cpu())?;
        model.set_read_identity_latch(ReadIdentityLatch::Held)?;
        model.set_geometric_address(GeometricAddressConfig::new(
            model.config.width,
            model.config.heads,
        )?)?;
        model.set_geometric_span(GeometricSpanConfig::new(model.config.width)?)?;
        // Construction-only controller: stable action markers occupy the
        // first lane, while two payloads have different static action lanes.
        // No forward API receives these expected labels.
        let width = model.config.width;
        let embedding = &model.variables()["embedding.weight"];
        let mut rows = embedding.flatten_all()?.to_vec1::<f32>()?;
        for (id, action) in [(1, 1), (2, 2), (3, 2), (4, 3), (5, 0), (6, 0)] {
            rows[id * width..id * width + 4].fill(0.0);
            rows[id * width + action] = 3.0;
        }
        for lane in 1..width / 4 {
            rows[2 * width + 4 * lane..2 * width + 4 * lane + 4]
                .copy_from_slice(&[0.0, 1.0, 0.0, 0.0]);
            rows[3 * width + 4 * lane..3 * width + 4 * lane + 4]
                .copy_from_slice(&[0.0, 0.0, 1.0, 0.0]);
        }
        embedding.set(&Tensor::from_vec(rows, embedding.shape(), &cpu())?)?;
        let mut controller = vec![0.0f32; 8 * width];
        for action in 0..4 {
            controller[action * 2 * width + action] = 1.0;
        }
        model.variables()["layers.02.read.span_control.weight"].set(&Tensor::from_vec(
            controller,
            (4, 2 * width),
            &cpu(),
        )?)?;
        Ok(model)
    }

    #[test]
    fn geometric_span_shared_forward_source_observer_causality_and_controls() -> Result<()> {
        let model = tiny_span_model()?;
        let ids = [1, 2, 3, 4, 5, 1, 3, 2, 4, 6];
        let expected_actions = vec![1, 2, 2, 3, 0, 1, 2, 2, 3, 0];
        assert_eq!(
            model
                .geometric_span_control_logits(&ids, 1, 10)?
                .argmax(2)?
                .to_vec2::<u32>()?[0],
            expected_actions
        );
        assert!(model.read_identity_latch().is_none());
        for name in [
            "query.weight",
            "key.weight",
            "query_identity.weight",
            "key_identity.weight",
            "identity_gate.weight",
            "identity_gate.bias",
            "log_beta",
            "offset",
        ] {
            assert!(
                !model
                    .variables()
                    .contains_key(&format!("layers.02.read.{name}")),
                "{name}"
            );
        }
        let reference = model.forward(&ids, 1, 10)?;
        let target = ReadBindingTarget {
            layer: 2,
            head: 0,
            rows: vec![ReadBinding {
                batch: 0,
                query: 9,
                sources: vec![4],
            }],
        };
        let p = model.params()?;
        let (hidden, masses) = model.hidden_with_binding(&p, &ids, 1, 10, &target)?;
        assert!(max_abs_gap(&hidden, &model.hidden(&ids, 1, 10)?)? < 1e-7);
        let mass = masses.to_vec1::<f32>()?[0];
        assert!(mass > 0.0 && mass < 1.0);
        let mut other = target.clone();
        other.rows[0].sources = vec![0, 2];
        assert!(
            max_abs_gap(
                &hidden,
                &model.hidden_with_binding(&p, &ids, 1, 10, &other)?.0
            )? < 1e-7
        );
        let mut future = ids;
        future[8] = 2;
        future[9] = 3;
        assert_eq!(
            bits(&reference.narrow(0, 0, 8)?)?,
            bits(&model.forward(&future, 1, 10)?.narrow(0, 0, 8)?)?
        );
        let counterfactual = model.forward_geometric_span_last_token(&ids, 1, 10)?;
        assert!(max_abs_gap(&reference, &counterfactual)? > 0.0);
        let last_mass = model
            .read_binding_masses_geometric_span_last_token(&ids, 1, 10, &target)?
            .to_vec1::<f32>()?[0];
        assert!((mass - last_mass).abs() > 0.0);
        assert_eq!(bits(&reference)?, bits(&model.forward(&ids, 1, 10)?)?);
        let output = &model.variables()["layers.02.read.out.weight"];
        let saved = output.as_tensor().copy()?;
        output.set(&output.zeros_like()?)?;
        assert!(max_abs_gap(&reference, &model.forward(&ids, 1, 10)?)? > 0.0);
        output.set(&saved)?;
        assert_eq!(bits(&reference)?, bits(&model.forward(&ids, 1, 10)?)?);
        Ok(())
    }

    #[test]
    fn geometric_span_language_and_event_gradients_reach_controller_and_trunk() -> Result<()> {
        let model = tiny_span_model()?;
        let ids = [1, 2, 3, 4, 5, 1, 3, 2, 4, 6];
        let targets: Vec<u32> = ids.iter().map(|x| (x + 7) % 37).collect();
        let gradients = model.loss(&ids, &targets, 1, 10)?.backward()?;
        for name in [
            "layers.02.read.span_control.weight",
            "layers.02.read.span_control.bias",
            "layers.02.read.address.content_unary",
            "layers.02.read.address.pair",
            "layers.01.rec.in.weight",
            "embedding.weight",
        ] {
            let gradient = gradients
                .get(&model.variables()[name])
                .ok_or_else(|| invalid(format!("language misses {name}")))?;
            let size = gradient.abs()?.max_all()?.to_scalar::<f32>()?;
            assert!(
                size.is_finite() && size > 0.0,
                "pure language {name}: {size}"
            );
        }
        // Event CE is the explicit teaching path for validity-only OPEN and
        // for initially uncommitted/empty states. No labels enter the scan.
        let event = logits_cross_entropy(
            &model
                .geometric_span_control_logits(&ids, 1, 10)?
                .reshape((10, 4))?,
            &[1, 2, 2, 3, 0, 1, 2, 2, 3, 0],
            None,
        )?
        .backward()?;
        for name in [
            "layers.02.read.span_control.weight",
            "layers.01.rec.in.weight",
            "embedding.weight",
        ] {
            assert!(
                event
                    .get(&model.variables()[name])
                    .ok_or_else(|| invalid("event gradient absent"))?
                    .abs()?
                    .max_all()?
                    .to_scalar::<f32>()?
                    > 0.0,
                "{name}"
            );
        }
        Ok(())
    }

    #[test]
    fn geometric_span_earlier_token_credit_uses_ordered_product_not_context_route() -> Result<()> {
        let model = tiny_span_model()?;
        let ids = [1, 2, 3, 4, 5, 1, 3, 2, 4, 6];
        let p = model.params()?;
        let original = model.embed(&ids, 1, 10)?;
        let tokens = Var::from_tensor(&original.detach())?;
        let context = model
            .norm(
                &p,
                &model.run_layers(original, 0..2)?,
                "layers.02.read_norm.weight",
            )?
            .detach();
        let controller = model.geometric_span_control_logits(&ids, 1, 10)?.detach();
        let objective = |policy| -> Result<Tensor> {
            let prior = geometric_span::produce(
                tokens.as_tensor(),
                &controller,
                model.geometric_span().unwrap(),
                policy,
            )?;
            let scores = geometric_address::score(
                &context,
                &prior,
                model.geometric_address().unwrap(),
                &model.address_weights(&p, 2)?,
            )?;
            let query = scores.get(0)?.get(0)?.get(9)?.reshape((1, 10))?;
            logits_cross_entropy(&query, &[4], None)
        };
        let ordered = objective(SpanPolicy::Ordered)?.backward()?;
        let last = objective(SpanPolicy::LastToken)?.backward()?;
        let earlier = |grads: &candle_core::backprop::GradStore| -> Result<f32> {
            Ok(grads
                .get(tokens.as_tensor())
                .ok_or_else(|| invalid("token action gradient absent"))?
                .get(0)?
                .get(1)?
                .narrow(0, 4, 12)?
                .abs()?
                .max_all()?
                .to_scalar::<f32>()?)
        };
        assert!(earlier(&ordered)? > 0.0);
        assert_eq!(earlier(&last)?, 0.0);
        Ok(())
    }

    #[test]
    fn native_geometric_context_reader_has_delayed_answer_credit_and_typed_reload() -> Result<()> {
        // Match the retained value codec: two heads, four R4 lanes per head.
        let model = tiny_span_model_with_width(32)?;
        let ids = [1, 2, 3, 4, 5, 1, 3, 2, 4, 6];
        let events = EventWeights::new(model.config.vocab_size, 2, 47)?;
        let lexical = events
            .parameters()
            .get("token_event")
            .ok_or_else(|| invalid("event lexical parameter absent"))?;
        let mut rows = vec![0f32; model.config.vocab_size * 4];
        for id in 0..model.config.vocab_size {
            rows[id * 4] = 4.;
        }
        for (id, action) in [(1, 1), (2, 2), (3, 2), (4, 3), (5, 0), (6, 0)] {
            rows[id * 4..id * 4 + 4].fill(0.);
            rows[id * 4 + action] = 4.;
        }
        lexical.set(&Tensor::from_vec(
            rows,
            (model.config.vocab_size, 4),
            &cpu(),
        )?)?;
        assert_eq!(
            events
                .event_logits(&ids, 1, 10)?
                .argmax(2)?
                .to_vec2::<u32>()?[0],
            vec![1, 2, 2, 3, 0, 1, 2, 2, 3, 0]
        );
        let root = std::env::temp_dir().join(format!(
            "native-context-reader-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        let base = root.join("base");
        model.save(&base)?;
        let registry = b"native-context-reader-token-registry/1";
        events.save_source(&root.join("event-source"), &base, registry)?;
        let event = CompiledEvents::compile(&events, &root.join("event-source"), &base, registry)?;
        let span_source = crate::geometric_span_native::SpanSourceBinding::from_files(
            &base.join("model.safetensors"),
            &base.join("config.json"),
            registry,
        )?;
        let p = model.params()?;
        let span = CompiledSpanActions::compile(
            p.get("embedding.weight")?,
            model
                .geometric_span()
                .ok_or_else(|| invalid("span absent"))?,
            &span_source,
        )?;
        let potential_source =
            crate::geometric_potential_native::PotentialSourceBinding::from_directory(
                &base, registry,
            )?;
        let potential = CompiledGeometricPotentials::compile(
            &model.geometric_address_potentials()?,
            model
                .geometric_address()
                .ok_or_else(|| invalid("address absent"))?,
            &potential_source,
        )?;
        event.save(&root.join("native-events"))?;
        span.save(&root.join("native-span"))?;
        potential.save(&root.join("native-potential"))?;
        let context = ContextWeights::new(
            model.config.vocab_size,
            model.config.width,
            model.config.heads,
            81,
        )?;
        // Ordinary answer loss passes through produced geometric values to
        // actual retained state, without using donor read.value payloads.
        // Address gradients are stopped here to isolate value-to-state credit.
        let finite_context = ContextWeights::new_finite_choice(
            model.config.vocab_size,
            model.config.width,
            model.config.heads,
            83,
        )?;
        let producer = crate::geometric_value_producer::ValueProducerWeights::new(
            model.config.vocab_size,
            model.config.heads,
            finite_context.config().lanes_per_head,
            89,
        )?;
        let produced = model.forward_geometric_context_learned_values_mode(
            &ids,
            1,
            10,
            &finite_context,
            &producer,
            &event,
            &span,
            false,
            true,
        )?;
        let donor_value = model
            .variables
            .get(&layer_name(2, "read.value.weight"))
            .ok_or_else(|| invalid("donor value parameter absent"))?;
        let retained_donor_value = donor_value.as_tensor().copy()?;
        donor_value.set(&Tensor::zeros(donor_value.shape(), DType::F32, &cpu())?)?;
        let bypassed = model.forward_geometric_context_learned_values_mode(
            &ids,
            1,
            10,
            &finite_context,
            &producer,
            &event,
            &span,
            false,
            true,
        );
        donor_value.set(&retained_donor_value)?;
        let bypassed = bypassed?;
        assert_eq!(
            produced.flatten_all()?.to_vec1::<f32>()?,
            bypassed.flatten_all()?.to_vec1::<f32>()?,
            "learned values must be independent of donor read.value weights"
        );
        let produced_loss = logits_cross_entropy(&produced.narrow(0, 9, 1)?, &[7], None)?;
        let produced_gradients = produced_loss.backward()?;
        let value_gradient =
            producer
                .parameters()
                .values()
                .try_fold(0f32, |total, value| -> Result<f32> {
                    Ok(total
                        + produced_gradients
                            .get(value.as_tensor())
                            .map(|g| g.abs()?.max_all()?.to_scalar::<f32>())
                            .transpose()?
                            .unwrap_or(0.))
                })?;
        assert!(
            value_gradient.is_finite() && value_gradient > 0.,
            "answer-to-value gradient {value_gradient}"
        );
        let finite_transition = finite_context
            .parameters()
            .get("token_transition")
            .ok_or_else(|| invalid("finite transition absent"))?;
        let history_gradient = produced_gradients
            .get(finite_transition.as_tensor())
            .ok_or_else(|| invalid("produced answer misses retained state"))?
            .get(2)?
            .abs()?
            .max_all()?
            .to_scalar::<f32>()?;
        assert!(
            history_gradient.is_finite() && history_gradient > 0.,
            "value-to-earlier-state gradient {history_gradient}"
        );
        let trained =
            model.forward_geometric_context(&ids, 1, 10, &context, &event, &span, false)?;
        let language = logits_cross_entropy(&trained.narrow(0, 9, 1)?, &[7], None)?;
        let gradients = language.backward()?;
        let transition = context
            .parameters()
            .get("token_transition")
            .ok_or_else(|| invalid("context transition absent"))?;
        let gradient = gradients
            .get(transition.as_tensor())
            .ok_or_else(|| invalid("answer misses context transition"))?;
        let early = gradient.get(2)?.abs()?.max_all()?.to_scalar::<f32>()?;
        assert!(
            early.is_finite() && early > 0.,
            "answer-to-earlier-context transition {early}"
        );
        assert!(gradients
            .get(&model.variables()["layers.02.read.span_control.weight"])
            .is_none());
        let base_path = root.join("base");
        let event_source = root.join("event-source");
        let event_native = root.join("native-events");
        let span_native = root.join("native-span");
        let potential_native = root.join("native-potential");
        let paths = geometric_context::ContextSourcePaths {
            base: &base_path,
            event_source: &event_source,
            event_native: &event_native,
            span_native: &span_native,
            potential_native: &potential_native,
        };
        context.save_source(&root.join("context-source"), paths, registry)?;
        let reload = ContextWeights::load_source(&root.join("context-source"), paths, registry)?;
        let native =
            CompiledContext::compile(&reload, &root.join("context-source"), paths, registry)?;
        native.save(&root.join("native-context"))?;
        let loaded = CompiledContext::load(
            &root.join("native-context"),
            &root.join("context-source"),
            paths,
            registry,
        )?;
        let replay = model.forward_geometric_context_replay(
            &ids, 1, 10, &reload, &event, &span, &potential, false,
        )?;
        let compiled = model.forward_geometric_context_native(
            &ids, 1, 10, &loaded, &event, &span, &potential, false,
        )?;
        assert_eq!(bits(&replay)?, bits(&compiled)?);
        let teacher = model.geometric_context_teacher(&ids, 1, 10)?;
        let (old_teacher, _) = model.geometric_potential_inputs(&ids, 1, 10, &span)?;
        assert_eq!(bits(&teacher)?, bits(&old_teacher)?);
        // Independently admitted four-lane configuration with incompatible head count.
        let wrong_heads = ContextWeights::new(model.config.vocab_size, 16, 1, 82)?;
        assert!(model
            .forward_geometric_context(&ids, 1, 10, &wrong_heads, &event, &span, false)
            .is_err());
        let other = EventWeights::new(model.config.vocab_size, 2, 83)?;
        other.save_source(&root.join("other-events"), &base, registry)?;
        let other = CompiledEvents::compile(&other, &root.join("other-events"), &base, registry)?;
        assert!(model
            .forward_geometric_context_native(
                &ids, 1, 10, &loaded, &other, &span, &potential, false
            )
            .is_err());
        // Exercise the independently loaded native reduction through the
        // actual model and observer, using the same source-bound fixture.
        let read_source = crate::geometric_read_native::ReadSourceBinding::from_directory(
            &base, registry, &potential, 2,
        )?;
        let reducer = crate::geometric_read_native::CompiledGeometricRead::compile(
            p.layer(2, "read.age")?,
            &potential,
            &read_source,
        )?;
        reducer.save(&root.join("native-read"))?;
        let reducer = crate::geometric_read_native::CompiledGeometricRead::load(
            &root.join("native-read"),
            &read_source,
        )?;
        // Saved geometric head + actual compiled retained-state producer.
        let finite_source = root.join("finite-context-source");
        finite_context.save_source(&finite_source, paths, registry)?;
        let finite_native =
            CompiledContext::compile(&finite_context, &finite_source, paths, registry)?;
        let producer_source = root.join("learned-value-source");
        producer.save_source(&producer_source)?;
        let value_paths = crate::geometric_value_producer_native::ValueProducerSourcePaths {
            value_source: &producer_source,
            context_source: &finite_source,
            context_dependencies: paths,
        };
        let native_producer =
            crate::geometric_value_producer_native::CompiledValueProducer::compile(
                value_paths,
                registry,
            )?;
        native_producer.validate_for(&producer, &finite_context)?;
        native_producer.save(&root.join("native-learned-values"))?;
        let native_producer = crate::geometric_value_producer_native::CompiledValueProducer::load(
            &root.join("native-learned-values"),
            value_paths,
            registry,
        )?;
        let latent_output = finite_context.forward(&ids, 1, 10, false)?;
        let held_output = geometric_span_native::produce_native_events(
            &ids,
            1,
            10,
            &geometric_event::trace_native(&ids, 1, 10, &event, false)?.actions,
            &span,
        )?;
        let held_trace = geometric_span_native::trace_native_events(
            &ids,
            1,
            10,
            &geometric_event::trace_native(&ids, 1, 10, &event, false)?.actions,
            &span,
        )?;
        let span_valid: Vec<_> = held_trace.prior_codes.iter().map(Option::is_some).collect();
        let offline_values = producer.forward(
            &ids,
            &latent_output.latent_roots,
            &held_output.reshape((1, 10, 2, 4, 4))?,
            &span_valid,
            &[true; 10],
        )?;
        let (native_logits, native_read, native_values) = model
            .forward_geometric_context_learned_values_native_with_trace(
                &ids,
                1,
                10,
                &finite_native,
                &event,
                &span,
                &potential,
                &reducer,
                &native_producer,
                false,
            )?;
        assert_eq!(
            native_values, offline_values.trace,
            "actual compiled packets/Q16 must match offline choices on fixture"
        );
        assert_eq!(native_read.rows.len(), 20);
        let retained = donor_value.as_tensor().copy()?;
        donor_value.set(&Tensor::zeros(donor_value.shape(), DType::F32, &cpu())?)?;
        let bypassed = model.forward_geometric_context_learned_values_native_with_trace(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            false,
        );
        donor_value.set(&retained)?;
        let (bypassed_logits, _, bypassed_values) = bypassed?;
        assert_eq!(bits(&native_logits)?, bits(&bypassed_logits)?);
        assert_eq!(native_values, bypassed_values);
        assert!(
            model
                .forward_geometric_context_learned_values_native_with_trace(
                    &ids,
                    1,
                    10,
                    &loaded,
                    &event,
                    &span,
                    &potential,
                    &reducer,
                    &native_producer,
                    false,
                )
                .is_err(),
            "different context artifact cannot feed compiled value head"
        );
        // The connected q4 NoRead path uses these SAME actual frozen native
        // producers, before the old normalized-read/null/value dependencies.
        let null_weights =
            crate::geometric_no_read::NoReadWeights::new(model.config.vocab_size, 2, 4)?;
        let null_source = root.join("no-read-source");
        null_weights.save(&null_source)?;
        let finite_native_path = root.join("finite-native-context");
        finite_native.save(&finite_native_path)?;
        let native_values_path = root.join("native-learned-values");
        let native_read_path = root.join("native-read");
        let null_paths = crate::geometric_no_read_native::NoReadSourcePaths {
            no_read_source: &null_source,
            value: value_paths,
            context_native: &finite_native_path,
            value_native: &native_values_path,
            reducer_native: &native_read_path,
        };
        let null_compiled = crate::geometric_no_read_native::CompiledNoRead::compile(
            &null_weights,
            null_paths,
            registry,
        )?;
        null_compiled.save(&root.join("native-null"))?;
        let null_compiled = crate::geometric_no_read_native::CompiledNoRead::load(
            &root.join("native-null"),
            null_paths,
            registry,
        )?;
        let (null_source_logits, null_scores) = model.forward_geometric_no_read(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            &null_weights,
            false,
        )?;
        assert_eq!(null_scores.to_vec3::<f32>()?, vec![vec![vec![0.; 10]; 2]]);
        let language = logits_cross_entropy(&null_source_logits.narrow(0, 9, 1)?, &[7], None)?;
        let gradient = language.backward()?;
        let coefficient = gradient
            .get(null_weights.parameters()["coefficients"].as_tensor())
            .ok_or_else(|| invalid("actual reader answer misses NoRead coefficients"))?;
        let magnitude = coefficient.abs()?.max_all()?.to_scalar::<f32>()?;
        assert!(
            magnitude.is_finite() && magnitude > 0.,
            "ordinary reader NoRead credit {magnitude}"
        );
        assert!(gradient
            .get(&model.variables()["layers.02.read.null.weight"])
            .is_none());
        assert!(gradient
            .get(&model.variables()["layers.02.read_norm.weight"])
            .is_none());
        let (null_logits, null_trace, null_values) = model
            .forward_geometric_no_read_native_with_trace(
                &ids,
                1,
                10,
                &finite_native,
                &event,
                &span,
                &potential,
                &reducer,
                &native_producer,
                &null_compiled,
                false,
            )?;
        assert_eq!(null_trace.no_read_q24, vec![0; 20]);
        assert_eq!(null_values, native_values);
        // Poison unused donor branches. Restore even if the forward rejects.
        let mut held_parameters = Vec::new();
        for suffix in [
            "read.null.weight",
            "read.null.bias",
            "read_norm.weight",
            "read.value.weight",
        ] {
            let name = layer_name(2, suffix);
            let var = &model.variables()[&name];
            held_parameters.push((name, var.as_tensor().copy()?));
            var.set(&Tensor::full(f32::NAN, var.shape(), &cpu())?)?;
        }
        let poisoned = model.forward_geometric_no_read_native_with_trace(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            &null_compiled,
            false,
        );
        for (name, value) in held_parameters {
            model.variables()[&name].set(&value)?;
        }
        let (poisoned_logits, poisoned_trace, _) = poisoned?;
        assert_eq!(bits(&null_logits)?, bits(&poisoned_logits)?);
        assert_eq!(null_trace, poisoned_trace);
        let output = &model.variables()["layers.02.read.out.weight"];
        let old_output = output.as_tensor().copy()?;
        output.set(&Tensor::zeros(output.shape(), DType::F32, &cpu())?)?;
        let disabled = model.forward_geometric_no_read_native_with_trace(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            &null_compiled,
            false,
        );
        output.set(&old_output)?;
        assert_ne!(
            bits(&null_logits)?,
            bits(&disabled?.0)?,
            "native read still drives output"
        );
        let mut future = ids;
        future[9] = 2;
        let future = model.forward_geometric_no_read_native_with_trace(
            &future,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            &null_compiled,
            false,
        )?;
        assert_eq!(
            bits(&null_logits.narrow(0, 0, 9)?)?,
            bits(&future.0.narrow(0, 0, 9)?)?
        );
        for h in 0..2 {
            for q in 0..9 {
                let previous = null_trace
                    .rows
                    .get(h * null_trace.time + q)
                    .ok_or_else(|| invalid("NoRead fixture prior row missing"))?;
                let changed = future
                    .1
                    .rows
                    .get(h * future.1.time + q)
                    .ok_or_else(|| invalid("NoRead fixture changed row missing"))?;
                assert_eq!(previous, changed);
            }
        }
        let reloaded_model = StackModel::load(&base, &cpu())?;
        let reloaded_result = reloaded_model.forward_geometric_no_read_native_with_trace(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            &null_compiled,
            false,
        )?;
        assert_eq!(bits(&null_logits)?, bits(&reloaded_result.0)?);
        assert_eq!(null_trace, reloaded_result.1);

        // Connected geometric value-to-residual composition. The same actual
        // packets and per-head score weights now bypass the donor read.out.
        let composition = crate::geometric_composition::CompositionWeights::new()?;
        let composition_source = root.join("composition-source");
        composition.save(&composition_source)?;
        let composition =
            crate::geometric_composition::CompositionWeights::load(&composition_source)?;
        let null_native_path = root.join("native-null");
        let composition_paths = crate::geometric_composition_native::CompositionSourcePaths {
            composition_source: &composition_source,
            no_read: null_paths,
            no_read_native: &null_native_path,
        };
        let composed = crate::geometric_composition_native::CompiledComposition::compile(
            &composition,
            composition_paths,
            registry,
        )?;
        let composed_path = root.join("native-composition");
        composed.save(&composed_path)?;
        let composed = crate::geometric_composition_native::CompiledComposition::load(
            &composed_path,
            composition_paths,
            registry,
        )?;
        let source_logits = model.forward_geometric_composition(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            &null_compiled,
            &composition,
            false,
        )?;
        let answer = logits_cross_entropy(&source_logits.narrow(0, 9, 1)?, &[7], None)?;
        let gradients = answer.backward()?;
        for (name, var) in composition.parameters() {
            let values = gradients
                .get(var.as_tensor())
                .ok_or_else(|| invalid(format!("composition answer gradient missing: {name}")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(values.iter().all(|v| v.is_finite()));
            assert!(
                values.iter().any(|v| *v != 0.),
                "composition answer gradient {name}"
            );
        }
        assert!(
            gradients.get(output.as_tensor()).is_none(),
            "composition must bypass donor output map"
        );
        let composed_result = model.forward_geometric_composition_native_with_trace(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            &null_compiled,
            &composed,
            false,
        )?;
        assert_eq!(composed_result.2, null_values);
        assert_eq!(composed_result.1.no_read_q24, null_trace.no_read_q24);
        assert_eq!(composed_result.1.rows.len(), null_trace.rows.len());
        for (new, old) in composed_result.1.rows.iter().zip(&null_trace.rows) {
            assert_eq!(new.occurrence_weights_q31, old.occurrence_weights_q31);
            assert_eq!(new.no_read_weight_q31, old.no_read_weight_q31);
            assert_eq!(new.total_weight_q31, old.total_weight_q31);
            assert_eq!(new.max_score_q24, old.max_score_q24);
        }
        output.set(&Tensor::full(f32::NAN, output.shape(), &cpu())?)?;
        let poisoned_source = model.forward_geometric_composition(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            &null_compiled,
            &composition,
            false,
        );
        let poisoned_native = model.forward_geometric_composition_native_with_trace(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            &null_compiled,
            &composed,
            false,
        );
        output.set(&old_output)?;
        assert_eq!(bits(&source_logits)?, bits(&poisoned_source?)?);
        assert_eq!(bits(&composed_result.0)?, bits(&poisoned_native?.0)?);
        let mut future_ids = ids;
        future_ids[9] = 2;
        let future_composed = model.forward_geometric_composition_native_with_trace(
            &future_ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            &null_compiled,
            &composed,
            false,
        )?;
        assert_eq!(
            bits(&composed_result.0.narrow(0, 0, 9)?)?,
            bits(&future_composed.0.narrow(0, 0, 9)?)?
        );
        for h in 0..2 {
            for q in 0..9 {
                assert_eq!(
                    composed_result.1.rows[h * 10 + q],
                    future_composed.1.rows[h * 10 + q]
                );
            }
        }
        let reloaded_composed = reloaded_model.forward_geometric_composition_native_with_trace(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &native_producer,
            &null_compiled,
            &composed,
            false,
        )?;
        assert_eq!(bits(&composed_result.0)?, bits(&reloaded_composed.0)?);
        assert_eq!(composed_result.1, reloaded_composed.1);
        assert_eq!(composed_result.2, reloaded_composed.2);

        // Strict q4 value credit remains connected through the frozen bank.
        // Rebind dependency envelopes, not scalar/bank parameter bytes.
        let q4 = producer.into_q4()?;
        let q4_source = root.join("q4-values-source");
        q4.save_source(&q4_source)?;
        let q4 = crate::geometric_value_producer::ValueProducerWeights::load_source(&q4_source)?;
        let q4_paths = crate::geometric_value_producer_native::ValueProducerSourcePaths {
            value_source: &q4_source,
            ..value_paths
        };
        let q4_native_path = root.join("q4-values-native");
        crate::geometric_value_producer_native::CompiledValueProducer::compile(q4_paths, registry)?
            .save(&q4_native_path)?;
        let q4_native = crate::geometric_value_producer_native::CompiledValueProducer::load(
            &q4_native_path,
            q4_paths,
            registry,
        )?;
        let q4_null_paths = crate::geometric_no_read_native::NoReadSourcePaths {
            value: q4_paths,
            value_native: &q4_native_path,
            ..null_paths
        };
        let q4_null = crate::geometric_no_read_native::CompiledNoRead::compile(
            &null_weights,
            q4_null_paths,
            registry,
        )?;
        let (q4_logits, q4_values) = model.forward_geometric_q4_values(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &q4_native,
            &q4_null,
            &q4,
            &composition,
            false,
        )?;
        let q4_gradient =
            logits_cross_entropy(&q4_logits.narrow(0, 9, 1)?, &[7], None)?.backward()?;
        for name in ["token_root", "token_category"] {
            let g = q4_gradient
                .get(q4.parameters()[name].as_tensor())
                .ok_or_else(|| invalid(format!("actual q4 answer misses {name}")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(
                g.iter().all(|v| v.is_finite()) && g.iter().any(|v| *v != 0.),
                "{name}"
            );
        }
        for var in composition.parameters().values() {
            assert!(q4_gradient.get(var.as_tensor()).is_none());
        }
        for suffix in [
            "read.out.weight",
            "read.value.weight",
            "read.null.weight",
            "read_norm.weight",
        ] {
            assert!(
                q4_gradient
                    .get(model.variables()[&layer_name(2, suffix)].as_tensor())
                    .is_none(),
                "{suffix}"
            );
        }
        for var in finite_context.parameters().values() {
            assert!(q4_gradient.get(var.as_tensor()).is_none());
        }
        let mut held_q4 = Vec::new();
        for suffix in [
            "read.out.weight",
            "read.value.weight",
            "read.null.weight",
            "read_norm.weight",
        ] {
            let name = layer_name(2, suffix);
            let var = &model.variables()[&name];
            held_q4.push((name, var.as_tensor().copy()?));
            var.set(&Tensor::full(f32::NAN, var.shape(), &cpu())?)?;
        }
        let poisoned = model.forward_geometric_q4_values(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &q4_native,
            &q4_null,
            &q4,
            &composition,
            false,
        );
        for (name, value) in held_q4 {
            model.variables()[&name].set(&value)?;
        }
        assert_eq!(bits(&q4_logits)?, bits(&poisoned?.0)?);
        let replay = reloaded_model.forward_geometric_q4_values(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &q4_native,
            &q4_null,
            &q4,
            &composition,
            false,
        )?;
        assert_eq!(bits(&q4_logits)?, bits(&replay.0)?);
        assert_eq!(q4_values.trace, replay.1.trace);
        let future_q4 = model.forward_geometric_q4_values(
            &future_ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &q4_native,
            &q4_null,
            &q4,
            &composition,
            false,
        )?;
        assert_eq!(
            bits(&q4_logits.narrow(0, 0, 9)?)?,
            bits(&future_q4.0.narrow(0, 0, 9)?)?
        );
        let q4_null_path = root.join("q4-null-native");
        q4_null.save(&q4_null_path)?;
        let q4_composition_paths = crate::geometric_composition_native::CompositionSourcePaths {
            composition_source: &composition_source,
            no_read: q4_null_paths,
            no_read_native: &q4_null_path,
        };
        let q4_composition = crate::geometric_composition_native::CompiledComposition::compile(
            &composition,
            q4_composition_paths,
            registry,
        )?;
        let native_q4 = model.forward_geometric_composition_native_with_trace(
            &ids,
            1,
            10,
            &finite_native,
            &event,
            &span,
            &potential,
            &reducer,
            &q4_native,
            &q4_null,
            &q4_composition,
            false,
        )?;
        assert_eq!(native_q4.1.no_read_q24, composed_result.1.no_read_q24);
        for (new, old) in native_q4.1.rows.iter().zip(&composed_result.1.rows) {
            assert_eq!(new.occurrence_weights_q31, old.occurrence_weights_q31);
            assert_eq!(new.no_read_weight_q31, old.no_read_weight_q31);
            assert_eq!(new.total_weight_q31, old.total_weight_q31);
            assert_eq!(new.max_score_q24, old.max_score_q24);
        }

        // Standalone strict potential coefficients are the only new learned
        // state. Rebind the frozen dependency envelopes without changing their
        // numerical parameters, and exercise the actual shared denominator.
        let score_source = root.join("strict-potential-source");
        crate::geometric_potential_q4::PotentialQ4Weights::from_base(&base, registry)?
            .save(&score_source)?;
        let score_weights = crate::geometric_potential_q4::PotentialQ4Weights::load(&score_source)?;
        let score_native_path = root.join("strict-potential-native");
        CompiledGeometricPotentials::compile_q4(&score_weights, &score_source, &potential_source)?
            .save(&score_native_path)?;
        let score_native =
            CompiledGeometricPotentials::load(&score_native_path, &potential_source)?;
        let score_paths = geometric_context::ContextSourcePaths {
            potential_native: &score_native_path,
            ..paths
        };
        let score_context_source = root.join("strict-potential-context-source");
        finite_context.save_source(&score_context_source, score_paths, registry)?;
        let score_context = CompiledContext::compile(
            &finite_context,
            &score_context_source,
            score_paths,
            registry,
        )?;
        let score_context_path = root.join("strict-potential-context-native");
        score_context.save(&score_context_path)?;
        let score_read_source = crate::geometric_read_native::ReadSourceBinding::from_directory(
            &base,
            registry,
            &score_native,
            2,
        )?;
        let score_reducer = crate::geometric_read_native::CompiledGeometricRead::compile(
            p.layer(2, "read.age")?,
            &score_native,
            &score_read_source,
        )?;
        let score_reducer_path = root.join("strict-potential-reducer");
        score_reducer.save(&score_reducer_path)?;
        let score_value_paths = crate::geometric_value_producer_native::ValueProducerSourcePaths {
            value_source: &q4_source,
            context_source: &score_context_source,
            context_dependencies: score_paths,
        };
        let score_values = crate::geometric_value_producer_native::CompiledValueProducer::compile(
            score_value_paths,
            registry,
        )?;
        let score_values_path = root.join("strict-potential-values");
        score_values.save(&score_values_path)?;
        let score_null_paths = crate::geometric_no_read_native::NoReadSourcePaths {
            no_read_source: &null_source,
            value: score_value_paths,
            context_native: &score_context_path,
            value_native: &score_values_path,
            reducer_native: &score_reducer_path,
        };
        let score_null = crate::geometric_no_read_native::CompiledNoRead::compile(
            &null_weights,
            score_null_paths,
            registry,
        )?;
        let score_null_path = root.join("strict-potential-null");
        score_null.save(&score_null_path)?;
        let score_bank_paths = crate::geometric_composition_native::CompositionSourcePaths {
            composition_source: &composition_source,
            no_read: score_null_paths,
            no_read_native: &score_null_path,
        };
        let score_bank = crate::geometric_composition_native::CompiledComposition::compile(
            &composition,
            score_bank_paths,
            registry,
        )?;
        let strict_forward =
            |model: &StackModel,
             input: &[u32],
             weights: &crate::geometric_potential_q4::PotentialQ4Weights| {
                model.forward_geometric_q4_potential(
                    input,
                    1,
                    10,
                    &score_context,
                    &event,
                    &span,
                    &score_native,
                    &score_reducer,
                    &score_values,
                    &score_null,
                    &score_bank,
                    weights,
                    false,
                )
            };
        let strict = strict_forward(&model, &ids, &score_weights)?;
        assert_eq!(
            strict.2, native_q4.2,
            "potential rebinding does not change value packets"
        );
        for h in 0..2 {
            for q in 0..10 {
                for k in 0..=q {
                    let qi = (q * 2 + h) * 4;
                    let ki = (k * 2 + h) * 4;
                    assert_eq!(
                        strict.1.scores_q24[(h * 10 + q) * 10 + k],
                        score_native.score_pair_codes(
                            h,
                            &strict.1.content_codes[qi..qi + 4],
                            &strict.1.content_codes[ki..ki + 4],
                            &strict.1.context_codes[qi..qi + 4],
                            &strict.1.context_codes[ki..ki + 4],
                        )?
                    );
                }
            }
        }
        let strict_gradient =
            logits_cross_entropy(&strict.0.narrow(0, 9, 1)?, &[7], None)?.backward()?;
        let mut score_credit = 0f64;
        for var in score_weights.parameters().values() {
            let g = strict_gradient
                .get(var.as_tensor())
                .ok_or_else(|| invalid("actual answer misses potential coefficient graph"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(g.iter().all(|x| x.is_finite()));
            score_credit += g.iter().map(|&x| f64::from(x) * f64::from(x)).sum::<f64>();
        }
        assert!(
            score_credit > 0.,
            "actual denominator must credit standalone coefficients"
        );
        for var in composition
            .parameters()
            .values()
            .chain(null_weights.parameters().values())
            .chain(q4.parameters().values())
            .chain(finite_context.parameters().values())
        {
            assert!(strict_gradient.get(var.as_tensor()).is_none());
        }
        let old_output = model.variables()["layers.02.read.out.weight"]
            .as_tensor()
            .copy()?;
        let output = &model.variables()["layers.02.read.out.weight"];
        output.set(&Tensor::full(f32::NAN, output.shape(), &cpu())?)?;
        let poisoned_strict = strict_forward(&model, &ids, &score_weights);
        output.set(&old_output)?;
        assert_eq!(bits(&strict.0)?, bits(&poisoned_strict?.0)?);
        let future_strict = strict_forward(&model, &future_ids, &score_weights)?;
        assert_eq!(
            bits(&strict.0.narrow(0, 0, 9)?)?,
            bits(&future_strict.0.narrow(0, 0, 9)?)?
        );
        let loaded_scores = crate::geometric_potential_q4::PotentialQ4Weights::load(&score_source)?;
        let restored_strict = strict_forward(&reloaded_model, &ids, &loaded_scores)?;
        assert_eq!(bits(&strict.0)?, bits(&restored_strict.0)?);
        assert_eq!(strict.1.scores_q24, restored_strict.1.scores_q24);

        let reduced = model.forward_geometric_context_read_native(
            &ids, 1, 10, &loaded, &event, &span, &potential, &reducer, false,
        )?;
        assert_eq!(reduced.dims(), compiled.dims());
        assert!(reduced
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .all(|v| v.is_finite()));
        let target = ReadBindingTarget {
            layer: 2,
            head: 0,
            rows: vec![ReadBinding {
                batch: 0,
                query: 9,
                sources: vec![1, 2],
            }],
        };
        let observed = model
            .read_binding_masses_geometric_context_read_native(
                &ids, 1, 10, &target, &loaded, &event, &span, &potential, &reducer, false,
            )?
            .to_vec1::<f32>()?;
        assert_eq!(observed.len(), 1);
        assert!(observed[0].is_finite() && (0.0..=1.0).contains(&observed[0]));
        let (traced, raw) = model.forward_geometric_context_read_native_with_trace(
            &ids, 1, 10, &loaded, &event, &span, &potential, &reducer, false,
        )?;
        assert_eq!(bits(&traced)?, bits(&reduced)?);
        assert_eq!((raw.batch, raw.heads, raw.time), (1, 2, 10));
        assert_eq!(raw.rows.len(), 20);
        assert_eq!(
            raw.source_mass(0, 0, 9, &[1, 2])?.to_bits(),
            observed[0].to_bits()
        );
        assert!((0.0..=1.0).contains(&raw.no_read_mass(0, 0, 9)?));
        // The oracle codec changes only payload representation, preserving
        // actual source/NoRead weights and exact integer payload trace bits.
        let value_source = crate::geometric_value_native::ValueSourceBinding::from_directory(
            &base, registry, &potential, &reducer, 2,
        )?;
        let value_codec =
            crate::geometric_value_native::CompiledGeometricValues::compile(&value_source)?;
        value_codec.save(&root.join("native-values"))?;
        let value_codec = crate::geometric_value_native::CompiledGeometricValues::load(
            &root.join("native-values"),
            &value_source,
        )?;
        let (projected_logits, projected_read, projected_values) = model
            .forward_geometric_context_value_projection_native_with_trace(
                &ids,
                1,
                10,
                &loaded,
                &event,
                &span,
                &potential,
                &reducer,
                &value_codec,
                false,
            )?;
        assert_eq!(projected_logits.dims(), reduced.dims());
        assert_eq!(projected_values.donor_q16, raw.values_q16);
        assert_eq!(projected_values.projected_q16, projected_read.values_q16);
        for (old, projected) in raw.rows.iter().zip(&projected_read.rows) {
            assert_eq!(old.occurrence_weights_q31, projected.occurrence_weights_q31);
            assert_eq!(old.no_read_weight_q31, projected.no_read_weight_q31);
            assert_eq!(old.total_weight_q31, projected.total_weight_q31);
            assert_eq!(old.max_score_q24, projected.max_score_q24);
        }
        let (residual_logits, residual_read, residual_values) = model
            .forward_geometric_context_value_residual_native_with_trace(
                &ids,
                1,
                10,
                &loaded,
                &event,
                &span,
                &potential,
                &reducer,
                &value_codec,
                false,
            )?;
        assert_eq!(residual_logits.dims(), reduced.dims());
        assert_eq!(residual_values.donor_q16, projected_values.donor_q16);
        assert_eq!(residual_values.projected_q16, residual_read.values_q16);
        assert!(residual_values.sum_squared_error_q32 <= projected_values.sum_squared_error_q32);
        for (old, residual) in raw.rows.iter().zip(&residual_read.rows) {
            assert_eq!(old.occurrence_weights_q31, residual.occurrence_weights_q31);
            assert_eq!(old.no_read_weight_q31, residual.no_read_weight_q31);
            assert_eq!(old.total_weight_q31, residual.total_weight_q31);
            assert_eq!(old.max_score_q24, residual.max_score_q24);
        }
        assert!(value_codec
            .validate_for(
                &p.layer(2, "read.value.weight")?.affine(1.0, 0.01)?,
                &potential,
                &reducer,
            )
            .is_err());
        // Observing an arbitrary source set cannot alter predictive parameters,
        // context, or history. The label channel is removed before read.out.
        let after_observation = model.forward_geometric_context_read_native(
            &ids, 1, 10, &loaded, &event, &span, &potential, &reducer, false,
        )?;
        assert_eq!(bits(&reduced)?, bits(&after_observation)?);
        assert!(reducer
            .validate_for(&p.layer(2, "read.age")?.affine(1.0, 0.125)?, &potential,)
            .is_err());
        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    fn tiny_q4_span_events(vocab_size: usize) -> Result<EventWeights> {
        let events = EventWeights::new(vocab_size, 2, 307)?;
        for (name, parameter) in events.parameters() {
            let mut values = vec![0f32; parameter.elem_count()];
            match name.as_str() {
                "token_transition" => {
                    for token in 0..vocab_size {
                        for lane in 0..2 {
                            values[(token * 2 + lane) * 120 + (2 + token + lane) % 120] = 1.5;
                        }
                    }
                }
                "self_transition" | "neighbor_transition" => {
                    for (i, value) in values.iter_mut().enumerate() {
                        *value = if (i / 4 + i % 4) % 3 == 0 {
                            0.25
                        } else {
                            -0.25
                        };
                    }
                }
                "state_event" => {
                    for lane in 0..2 {
                        for event in 0..4 {
                            values[(lane * 4 + event) * 4 + (event + lane) % 4] = 0.25;
                        }
                    }
                }
                "token_event" => {
                    values.fill(-1.5);
                    for token in 0..vocab_size {
                        values[token * 4] = 1.5;
                    }
                    for (token, event) in [(1, 1), (2, 2), (3, 2), (4, 3)] {
                        values[token * 4..token * 4 + 4].fill(-1.5);
                        values[token * 4 + event] = 1.5;
                    }
                }
                _ => return Err(invalid("unexpected strict event fixture family")),
            }
            parameter.set(&Tensor::from_vec(values, parameter.shape(), &cpu())?)?;
        }
        events.into_q4()
    }

    #[test]
    fn geometric_event_age_native_credit_preserves_hard_reader_and_frozen_routes() -> Result<()> {
        use crate::geometric_composition::CompositionWeights;
        use crate::geometric_composition_native::{CompiledComposition, CompositionSourcePaths};
        use crate::geometric_context::{ContextSourcePaths, ContextWeights};
        use crate::geometric_no_read::NoReadWeights;
        use crate::geometric_no_read_native::{CompiledNoRead, NoReadSourcePaths};
        use crate::geometric_potential_native::PotentialSourceBinding;
        use crate::geometric_potential_q4::PotentialQ4Weights;
        use crate::geometric_read_native::{CompiledGeometricRead, ReadSourceBinding};
        use crate::geometric_value_producer::ValueProducerWeights;
        use crate::geometric_value_producer_native::{
            CompiledValueProducer, ValueProducerSourcePaths,
        };
        let model = tiny_span_model_with_width(32)?;
        let ids = [0, 1, 2, 3, 4, 5, 1, 3, 2, 4, 6];
        let directory = std::env::temp_dir().join(format!(
            "held-credit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        let base = directory.join("base");
        model.save(&base)?;
        let registry = b"held-credit-construction-only/1";
        let events = tiny_q4_span_events(model.config.vocab_size)?;
        let es = directory.join("event-source");
        let en = directory.join("event-native");
        events.save_source(&es, &base, registry)?;
        CompiledEvents::compile(&events, &es, &base, registry)?.save(&en)?;
        let event = CompiledEvents::load(&en, &es, &base, registry)?;
        let span_binding = geometric_span_native::SpanSourceBinding::from_files(
            &base.join("model.safetensors"),
            &base.join("config.json"),
            registry,
        )?;
        let span = CompiledSpanActions::compile(
            model.variables()["embedding.weight"].as_tensor(),
            model
                .geometric_span()
                .ok_or_else(|| invalid("span absent"))?,
            &span_binding,
        )?;
        let sn = directory.join("span-native");
        span.save(&sn)?;
        let potential_weights = PotentialQ4Weights::from_base(&base, registry)?;
        let cu = &potential_weights.parameters()["content_unary"];
        let mut cw = vec![0f32; cu.elem_count()];
        for row in cw.chunks_exact_mut(4) {
            row[1] = 0.5;
            row[2] = -0.25;
        }
        cu.set(&Tensor::from_vec(cw, cu.shape(), &cpu())?)?;
        let ps = directory.join("potential-source");
        let pn = directory.join("potential-native");
        potential_weights.save(&ps)?;
        let parent = PotentialSourceBinding::from_directory(&base, registry)?;
        CompiledGeometricPotentials::compile_q4(&potential_weights, &ps, &parent)?.save(&pn)?;
        let potential = CompiledGeometricPotentials::load(&pn, &parent)?;
        let context_weights =
            ContextWeights::new_finite_choice(model.config.vocab_size, 32, 2, 13)?.into_q4()?;
        let paths = ContextSourcePaths {
            base: &base,
            event_source: &es,
            event_native: &en,
            span_native: &sn,
            potential_native: &pn,
        };
        let cs = directory.join("context-source");
        let cn = directory.join("context-native");
        context_weights.save_source(&cs, paths, registry)?;
        CompiledContext::compile(&context_weights, &cs, paths, registry)?.save(&cn)?;
        let context = CompiledContext::load(&cn, &cs, paths, registry)?;
        let age = Var::from_tensor(&model.variables()["layers.02.read.age"].detach())?;
        let read_binding = ReadSourceBinding::from_directory(&base, registry, &potential, 2)?;
        let rn = directory.join("read-native");
        CompiledGeometricRead::compile_q4_age_residual(age.as_tensor(), &potential, &read_binding)?
            .save(&rn)?;
        let reducer = CompiledGeometricRead::load(&rn, &read_binding)?;
        let value_weights =
            ValueProducerWeights::new(model.config.vocab_size, 2, 4, 29)?.into_q4()?;
        let sv = &value_weights.parameters()["span_root"];
        let mut vw = vec![0f32; sv.elem_count()];
        for (r, row) in vw.chunks_exact_mut(4).enumerate() {
            row[r % 4] = if r % 3 == 0 { 0.5 } else { -0.25 };
        }
        sv.set(&Tensor::from_vec(vw, sv.shape(), &cpu())?)?;
        // Distinct occurrence payloads make potential/age credit observable.
        let tr = &value_weights.parameters()["token_root"];
        let mut tw = tr.flatten_all()?.to_vec1::<f32>()?;
        for (i, row) in tw.chunks_exact_mut(120).enumerate() {
            row[(i / 16) % 8] = 1.;
        }
        tr.set(&Tensor::from_vec(tw, tr.shape(), &cpu())?)?;
        let vs = directory.join("value-source");
        let vn = directory.join("value-native");
        value_weights.save_source(&vs)?;
        let vp = ValueProducerSourcePaths {
            value_source: &vs,
            context_source: &cs,
            context_dependencies: paths,
        };
        CompiledValueProducer::compile(vp, registry)?.save(&vn)?;
        let values = CompiledValueProducer::load(&vn, vp, registry)?;
        let null_weights = NoReadWeights::new(model.config.vocab_size, 2, 4)?;
        let nv = &null_weights.parameters()["coefficients"];
        let mut nw = vec![0f32; nv.elem_count()];
        let stride = null_weights.config().coefficients_per_head();
        for h in 0..2 {
            for l in 0..8 {
                nw[h * stride + 1 + model.config.vocab_size + 8 * 37 + l * 4] = 0.25;
            }
        }
        nv.set(&Tensor::from_vec(nw, nv.shape(), &cpu())?)?;
        let ns = directory.join("null-source");
        let nn = directory.join("null-native");
        null_weights.save(&ns)?;
        let np = NoReadSourcePaths {
            no_read_source: &ns,
            value: vp,
            context_native: &cn,
            value_native: &vn,
            reducer_native: &rn,
        };
        CompiledNoRead::compile(&null_weights, np, registry)?.save(&nn)?;
        let null = CompiledNoRead::load(&nn, np, registry)?;
        let bank_weights = CompositionWeights::new()?;
        let bs = directory.join("bank-source");
        let bn = directory.join("bank-native");
        bank_weights.save(&bs)?;
        let bp = CompositionSourcePaths {
            composition_source: &bs,
            no_read: np,
            no_read_native: &nn,
        };
        CompiledComposition::compile(&bank_weights, bp, registry)?.save(&bn)?;
        let bank = CompiledComposition::load(&bn, bp, registry)?;
        let forward = |m: &StackModel, input: &[u32], batch, time, routes| {
            m.forward_geometric_event_age_native_credit(
                input,
                batch,
                time,
                &context,
                &event,
                &span,
                &potential,
                &reducer,
                &values,
                &null,
                &bank,
                &events,
                age.as_tensor(),
                &value_weights,
                &null_weights,
                &potential_weights,
                &bank_weights,
                routes,
            )
        };
        let all = forward(&model, &ids, 1, ids.len(), [true; 4])?;
        let native = model.forward_geometric_composition_native_with_trace(
            &ids,
            1,
            ids.len(),
            &context,
            &event,
            &span,
            &potential,
            &reducer,
            &values,
            &null,
            &bank,
            false,
        )?;
        assert_eq!(bits(&all.0)?, bits(&native.0)?);
        assert_eq!(all.1.read, native.1);
        assert_eq!(all.1.values.trace, native.2);
        assert!(all.1.span.prior_codes[4].is_none());
        assert!(all.1.span.prior_codes[5].is_some());
        let event_norm = |g: &candle_core::backprop::GradStore| -> Result<f64> {
            let mut n = 0.;
            for v in events.parameters().values() {
                if let Some(x) = g.get(v.as_tensor()) {
                    n += x
                        .flatten_all()?
                        .to_vec1::<f32>()?
                        .iter()
                        .map(|&x| f64::from(x) * f64::from(x))
                        .sum::<f64>();
                }
            }
            Ok(n)
        };
        let gradient = logits_cross_entropy(&all.0.narrow(0, 10, 1)?, &[7], None)?.backward()?;
        assert!(event_norm(&gradient)? > 0.);
        assert!(
            gradient
                .get(age.as_tensor())
                .ok_or_else(|| invalid("age credit absent"))?
                .abs()?
                .max_all()?
                .to_scalar::<f32>()?
                > 0.
        );
        for v in model
            .variables()
            .values()
            .chain(context_weights.parameters().values())
            .chain(value_weights.parameters().values())
            .chain(null_weights.parameters().values())
            .chain(potential_weights.parameters().values())
            .chain(bank_weights.parameters().values())
        {
            assert!(
                gradient.get(v.as_tensor()).is_none(),
                "frozen parameter entered graph"
            );
        }
        for route in 0..3 {
            let mut cuts = [false; 4];
            cuts[route] = true;
            let isolated = forward(&model, &ids, 1, ids.len(), cuts)?;
            assert_eq!(bits(&all.0)?, bits(&isolated.0)?);
            let g = logits_cross_entropy(&isolated.0.narrow(0, 10, 1)?, &[7], None)?.backward()?;
            assert!(event_norm(&g)? > 0., "held route {route} disconnected");
            assert!(g.get(age.as_tensor()).is_none());
        }
        let cut = forward(&model, &ids, 1, ids.len(), [false; 4])?;
        assert_eq!(bits(&all.0)?, bits(&cut.0)?);
        let g = logits_cross_entropy(&cut.0.narrow(0, 10, 1)?, &[7], None)?.backward()?;
        assert_eq!(event_norm(&g)?, 0.);
        assert!(g.get(age.as_tensor()).is_none());
        let reloaded = StackModel::load(&base, &cpu())?;
        assert_eq!(
            bits(&all.0)?,
            bits(&forward(&reloaded, &ids, 1, ids.len(), [true; 4])?.0)?
        );
        let other = ids
            .iter()
            .map(|&token| match token {
                2 => 3,
                3 => 2,
                x => x,
            })
            .collect::<Vec<_>>();
        assert_ne!(ids.as_slice(), other.as_slice());
        let double = ids.iter().copied().chain(other).collect::<Vec<_>>();
        let two = forward(&model, &double, 2, ids.len(), [true; 4])?;
        let native_two = model.forward_geometric_composition_native_with_trace(
            &double,
            2,
            ids.len(),
            &context,
            &event,
            &span,
            &potential,
            &reducer,
            &values,
            &null,
            &bank,
            false,
        )?;
        assert_eq!(bits(&two.0)?, bits(&native_two.0)?);
        assert_eq!(two.1.read, native_two.1);
        let prefix = forward(&model, &ids[..10], 1, 10, [true; 4])?;
        // Integer reader rows carry cross-length causality. Float-tail parity
        // compares identical shapes because BLAS rounding may depend on shape.
        for head in 0..2 {
            for query in 0..10 {
                assert_eq!(
                    all.1.read.rows[head * ids.len() + query],
                    prefix.1.read.rows[head * 10 + query],
                );
            }
        }
        let native_prefix = model.forward_geometric_composition_native_with_trace(
            &ids[..10],
            1,
            10,
            &context,
            &event,
            &span,
            &potential,
            &reducer,
            &values,
            &null,
            &bank,
            false,
        )?;
        assert_eq!(bits(&prefix.0)?, bits(&native_prefix.0)?);
        let out = &model.variables()["layers.02.read.out.weight"];
        let old = out.copy()?;
        out.set(&Tensor::full(f32::NAN, out.shape(), &cpu())?)?;
        let poisoned = forward(&model, &ids, 1, ids.len(), [true; 4]);
        out.set(&old)?;
        assert_eq!(bits(&all.0)?, bits(&poisoned?.0)?);
        // Fresh current age changes the actual hard reduction; enrolled parent is unchanged.
        age.set(&age.affine(1., 0.125)?)?;
        let moved = forward(&model, &ids, 1, ids.len(), [true; 4])?;
        assert_ne!(all.1.age_q24, moved.1.age_q24);
        assert_ne!(all.1.read.rows, moved.1.read.rows);
        std::fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn geometric_event_q4_age_answer_credit_crosses_span_attention_and_head() -> Result<()> {
        let model = tiny_span_model()?;
        // Construction fixture with actual predicted open/append/commit paths.
        // The first token is unique and HOLD: credit to its transition row from
        // the last answer must cross recurrent event context, not a local label.
        let ids = [0, 1, 2, 3, 4, 5, 1, 3, 2, 4, 6];
        let events = tiny_q4_span_events(model.config.vocab_size)?;
        let predicted = events.forward_q4(&ids, 1, ids.len(), false)?;
        assert_eq!(
            predicted
                .trace
                .actions
                .iter()
                .map(|x| *x as usize)
                .collect::<Vec<_>>(),
            [0, 1, 2, 2, 3, 0, 1, 2, 2, 3, 0]
        );
        let age = &model.variables()["layers.02.read.age"];
        let age_shadow: Vec<f32> = (0..age.elem_count())
            .map(|i| 0.13 + (i % 5) as f32 * 0.1)
            .collect();
        age.set(&Tensor::from_vec(age_shadow.clone(), age.shape(), &cpu())?)?;
        let output = model.forward_geometric_event_q4_age(&ids, 1, ids.len(), &events, false)?;
        assert_eq!(output.dims(), [ids.len(), model.config.vocab_size]);
        let loss = logits_cross_entropy(&output.narrow(0, ids.len() - 1, 1)?, &[7], None)?;
        let gradients = loss.backward()?;
        for name in [
            "token_transition",
            "self_transition",
            "neighbor_transition",
            "token_event",
            "state_event",
        ] {
            let parameter = events
                .parameters()
                .get(name)
                .ok_or_else(|| invalid("strict event family absent"))?;
            let gradient = gradients
                .get(parameter.as_tensor())
                .ok_or_else(|| invalid(format!("answer-to-event {name} gradient absent")))?;
            let values = gradient.flatten_all()?.to_vec1::<f32>()?;
            assert!(values.iter().all(|x| x.is_finite()));
            assert!(
                values.iter().any(|x| *x != 0.),
                "answer-to-event {name} credit vanished"
            );
            if name == "token_transition" {
                assert!(
                    values[..240].iter().any(|x| *x != 0.),
                    "last answer misses unique first-token event context"
                );
            }
        }
        for (name, parameter) in [
            ("age", age),
            ("embedding", &model.variables()["embedding.weight"]),
        ] {
            let values = gradients
                .get(parameter.as_tensor())
                .ok_or_else(|| invalid(format!("answer-to-{name} gradient absent")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(values.iter().all(|x| x.is_finite()) && values.iter().any(|x| *x != 0.));
        }
        assert!(
            gradients
                .get(&model.variables()["layers.02.read.span_control.weight"])
                .is_none(),
            "legacy contextual event controller must not supply this route"
        );
        let direct = model.forward_geometric_event(&ids, 1, ids.len(), &events, false)?;
        assert_ne!(
            bits(&direct)?,
            bits(&output)?,
            "explicit age view must affect actual output"
        );
        age.set(&Tensor::from_vec(
            age_shadow
                .iter()
                .map(|x| (x * 4.).round() * 0.25)
                .collect::<Vec<_>>(),
            age.shape(),
            &cpu(),
        )?)?;
        assert_eq!(
            bits(&output)?,
            bits(&model.forward_geometric_event(&ids, 1, ids.len(), &events, false)?)?,
            "joint hard output must equal event-only output at the same explicit age grid"
        );

        // All-HOLD has no active span branch and therefore no event credit.
        // This is an existing estimator limitation, not permission to invent a
        // gradient or fit an easier local event-classification objective.
        let token_event = &events.parameters()["token_event"];
        let mut hold = vec![-1.5f32; model.config.vocab_size * 4];
        for token in 0..model.config.vocab_size {
            hold[token * 4] = 1.5;
        }
        token_event.set(&Tensor::from_vec(hold, token_event.shape(), &cpu())?)?;
        assert!(events
            .forward_q4(&ids, 1, ids.len(), false)?
            .trace
            .actions
            .iter()
            .all(|x| *x == SpanAction::Hold));
        let hold_output =
            model.forward_geometric_event_q4_age(&ids, 1, ids.len(), &events, false)?;
        let hold_gradients =
            logits_cross_entropy(&hold_output.narrow(0, ids.len() - 1, 1)?, &[7], None)?
                .backward()?;
        if let Some(gradient) =
            hold_gradients.get(events.parameters()["token_transition"].as_tensor())
        {
            assert!(gradient
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .all(|x| *x == 0.));
        }
        assert!(model
            .forward_geometric_event_q4_age(
                &ids,
                1,
                ids.len(),
                &EventWeights::new(model.config.vocab_size, 2, 308)?,
                false
            )
            .is_err());
        Ok(())
    }

    #[test]
    fn geometric_event_q4_age_residual_answer_credit_and_common_denominator() -> Result<()> {
        let model = tiny_span_model()?;
        let ids = [0, 1, 2, 3, 4, 5, 1, 3, 2, 4, 6];
        let events = tiny_q4_span_events(model.config.vocab_size)?;
        let variables = model.variables();
        let age = &variables["layers.02.read.age"];
        let null_bias = &variables["layers.02.read.null.bias"];
        let context = model.config.context;
        assert_eq!(model.config.heads, 2);
        let shadow: Vec<f32> = (0..age.elem_count())
            .map(|i| {
                let prior = -((i % context) as f32) / if i / context == 0 { 16. } else { 256. };
                prior + 0.13 + (i % 5) as f32 * 0.1
            })
            .collect();
        age.set(&Tensor::from_vec(shadow.clone(), age.shape(), &cpu())?)?;
        let output =
            model.forward_geometric_event_q4_age_residual(&ids, 1, ids.len(), &events, false)?;
        let gradients =
            logits_cross_entropy(&output.narrow(0, ids.len() - 1, 1)?, &[7], None)?.backward()?;
        let transition = gradients
            .get(events.parameters()["token_transition"].as_tensor())
            .ok_or_else(|| invalid("residual-age answer-to-event gradient absent"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(transition.iter().all(|x| x.is_finite()));
        assert!(
            transition[..240].iter().any(|x| *x != 0.),
            "residual-age answer must reach the unique first-token transition"
        );
        let age_gradient = gradients
            .get(age.as_tensor())
            .ok_or_else(|| invalid("residual-age answer gradient absent"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let null_gradient = gradients
            .get(null_bias.as_tensor())
            .ok_or_else(|| invalid("residual-age NoRead gradient absent"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        for values in [&age_gradient, &null_gradient] {
            assert!(values.iter().all(|x| x.is_finite()));
            assert!(values.iter().any(|x| *x != 0.));
        }
        // Adding a common constant to every occurrence and NoRead score
        // leaves the shared softmax unchanged. The live absolute-age STE
        // must preserve this derivative identity separately for each head.
        for (head, no_read) in null_gradient.iter().enumerate() {
            let row = &age_gradient[head * context..(head + 1) * context];
            let sum: f64 = row.iter().map(|x| f64::from(*x)).sum();
            let scale =
                row.iter().map(|x| f64::from(*x).abs()).sum::<f64>() + f64::from(*no_read).abs();
            assert!(
                (sum + f64::from(*no_read)).abs() <= 2e-4 * scale.max(1e-12),
                "age and NoRead credit must share one normalization"
            );
        }
        let raw = model.forward_geometric_event(&ids, 1, ids.len(), &events, false)?;
        assert_ne!(bits(&raw)?, bits(&output)?);
        // Independent hard reconstruction from the specified initialization
        // prior and residual grid; this deliberately does not call the helper
        // under test. The whole answer vector must match the unchanged path.
        let hard: Vec<f32> = shadow
            .iter()
            .enumerate()
            .map(|(i, value)| {
                let prior = -((i % context) as f64) / if i / context == 0 { 16. } else { 256. };
                (prior + ((f64::from(*value) - prior) * 8.).round() / 8.) as f32
            })
            .collect();
        age.set(&Tensor::from_vec(hard, age.shape(), &cpu())?)?;
        assert_eq!(
            bits(&output)?,
            bits(&model.forward_geometric_event(&ids, 1, ids.len(), &events, false)?)?
        );
        assert!(model
            .forward_geometric_event_q4_age_residual(
                &ids,
                1,
                ids.len(),
                &EventWeights::new(model.config.vocab_size, 2, 308)?,
                false,
            )
            .is_err());
        Ok(())
    }

    #[test]
    fn native_geometric_event_reader_has_language_credit_and_direct_integer_events() -> Result<()> {
        let model = tiny_span_model()?;
        let ids = [1, 2, 3, 4, 5, 1, 3, 2, 4, 6];
        let events = EventWeights::new(model.config.vocab_size, 2, 47)?;
        let lexical = events
            .parameters()
            .get("token_event")
            .ok_or_else(|| invalid("event lexical parameter absent"))?;
        let mut rows = vec![0f32; model.config.vocab_size * 4];
        for id in 0..model.config.vocab_size {
            rows[id * 4] = 4.;
        }
        for (id, action) in [(1, 1), (2, 2), (3, 2), (4, 3), (5, 0), (6, 0)] {
            rows[id * 4..id * 4 + 4].fill(0.);
            rows[id * 4 + action] = 4.;
        }
        lexical.set(&Tensor::from_vec(
            rows,
            (model.config.vocab_size, 4),
            &cpu(),
        )?)?;
        assert_eq!(
            events
                .event_logits(&ids, 1, 10)?
                .argmax(2)?
                .to_vec2::<u32>()?[0],
            vec![1, 2, 2, 3, 0, 1, 2, 2, 3, 0]
        );
        let old = bits(&model.forward(&ids, 1, 10)?)?;
        let trained = model.forward_geometric_event(&ids, 1, 10, &events, false)?;
        let language = logits_cross_entropy(&trained.narrow(0, 9, 1)?, &[7], None)?;
        let gradients = language.backward()?;
        let transition = events
            .parameters()
            .get("token_transition")
            .ok_or_else(|| invalid("transition parameter absent"))?;
        let grad = gradients
            .get(transition.as_tensor())
            .ok_or_else(|| invalid("language misses native event transition"))?;
        let early = grad.get(1)?.abs()?.max_all()?.to_scalar::<f32>()?;
        assert!(
            early.is_finite() && early > 0.,
            "language-to-earlier-transition gradient {early}"
        );
        assert!(gradients
            .get(&model.variables()["layers.02.read.span_control.weight"])
            .is_none());
        let root = std::env::temp_dir().join(format!(
            "native-event-reader-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        let base = root.join("base");
        model.save(&base)?;
        let registry = b"native-event-reader-token-registry/1";
        events.save_source(&root.join("event-source"), &base, registry)?;
        let event = CompiledEvents::compile(&events, &root.join("event-source"), &base, registry)?;
        let span_source = crate::geometric_span_native::SpanSourceBinding::from_files(
            &base.join("model.safetensors"),
            &base.join("config.json"),
            registry,
        )?;
        let p = model.params()?;
        let span = CompiledSpanActions::compile(
            p.get("embedding.weight")?,
            model
                .geometric_span()
                .ok_or_else(|| invalid("span absent"))?,
            &span_source,
        )?;
        let potential_source =
            crate::geometric_potential_native::PotentialSourceBinding::from_directory(
                &base, registry,
            )?;
        let potential = CompiledGeometricPotentials::compile(
            &model.geometric_address_potentials()?,
            model
                .geometric_address()
                .ok_or_else(|| invalid("address absent"))?,
            &potential_source,
        )?;
        let integer_trace = geometric_event::trace_native(&ids, 1, 10, &event, false)?;
        let connected_logits = events.event_logits(&ids, 1, 10)?;
        assert!(
            model
                .hidden_geometric_event(
                    &ids,
                    1,
                    10,
                    SpanEvents::TrainingNative {
                        logits: &connected_logits,
                        actions: &integer_trace.actions,
                        age: AgeTrainingMode::None,
                    },
                    None,
                    Some(&span),
                    Some(&potential),
                )
                .is_err(),
            "connected event training must not silently detach through compiled span replay"
        );
        let direct =
            geometric_span_native::trace_native_events(&ids, 1, 10, &integer_trace.actions, &span)?;
        let through_logits = geometric_span_native::trace_native(
            &ids,
            1,
            10,
            &events.event_logits(&ids, 1, 10)?,
            &span,
        )?;
        assert_eq!(direct.prior_codes, through_logits.prior_codes);
        let native =
            model.forward_geometric_event_native(&ids, 1, 10, &event, &span, &potential, false)?;
        assert_eq!(native.dims(), [10, model.config.vocab_size]);
        assert!(native
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .all(|x| x.is_finite()));
        let mass = model
            .read_binding_masses_geometric_event_native(
                &ids,
                1,
                10,
                &ReadBindingTarget {
                    layer: 2,
                    head: 0,
                    rows: vec![ReadBinding {
                        batch: 0,
                        query: 9,
                        sources: vec![4],
                    }],
                },
                &event,
                &span,
                &potential,
                false,
            )?
            .to_vec1::<f32>()?;
        assert!(mass.len() == 1 && mass[0].is_finite() && (0.0..=1.0).contains(&mass[0]));
        assert_eq!(bits(&model.forward(&ids, 1, 10)?)?, old);
        assert!(tiny_address_model()?
            .forward_geometric_event(&ids, 1, 10, &events, false)
            .is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn native_geometric_potential_reader_uses_saved_tables_and_refuses_stale_weights() -> Result<()>
    {
        use crate::geometric_potential_native::PotentialSourceBinding;
        use crate::geometric_span_native::SpanSourceBinding;
        let root = std::env::temp_dir().join(format!(
            "native-potential-reader-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        let model = tiny_span_model()?;
        model.save(&root)?;
        let registry = b"test-token-registry/1";
        let p = model.params()?;
        let span_source = SpanSourceBinding::from_files(
            &root.join("model.safetensors"),
            &root.join("config.json"),
            registry,
        )?;
        let span = CompiledSpanActions::compile(
            p.get("embedding.weight")?,
            model
                .geometric_span()
                .ok_or_else(|| invalid("span absent"))?,
            &span_source,
        )?;
        let potential_source = PotentialSourceBinding::from_directory(&root, registry)?;
        let config = model
            .geometric_address()
            .ok_or_else(|| invalid("address absent"))?;
        let weights = model.geometric_address_potentials()?;
        let potential = CompiledGeometricPotentials::compile(&weights, config, &potential_source)?;
        let ids = [1, 2, 3, 4, 5, 1, 3, 2, 4, 6];
        let before = bits(&model.forward(&ids, 1, 10)?)?;
        let (current, prior) = model.geometric_potential_inputs(&ids, 1, 10, &span)?;
        let expected_current = model.norm(
            &p,
            &model.run_layers(model.embed(&ids, 1, 10)?, 0..2)?,
            "layers.02.read_norm.weight",
        )?;
        assert_eq!(bits(&current)?, bits(&expected_current)?);
        let controller = model.geometric_span_control_logits(&ids, 1, 10)?;
        let expected_prior = geometric_span::produce(
            &model.embed(&ids, 1, 10)?,
            &controller,
            model
                .geometric_span()
                .ok_or_else(|| invalid("span absent"))?,
            SpanPolicy::Ordered,
        )?;
        assert_eq!(bits(&prior)?, bits(&expected_prior)?);
        let original_scores = geometric_address::score(&current, &prior, config, &weights)?;
        let native_scores = geometric_potential_native::score_native(
            &current, &prior, config, &weights, &potential,
        )?;
        assert_eq!(original_scores.dims(), native_scores.dims());
        let native = model.forward_geometric_potential_native(&ids, 1, 10, &span, &potential)?;
        assert_eq!(native.dims(), [10, model.config.vocab_size]);
        assert!(native
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .all(|x| x.is_finite()));
        for head in 0..2 {
            let target = ReadBindingTarget {
                layer: 2,
                head,
                rows: vec![ReadBinding {
                    batch: 0,
                    query: 9,
                    sources: vec![4],
                }],
            };
            let masses = model
                .read_binding_masses_geometric_potential_native(
                    &ids, 1, 10, &target, &span, &potential,
                )?
                .to_vec1::<f32>()?;
            assert_eq!(masses.len(), 1);
            assert!(masses[0].is_finite() && (0.0..=1.0).contains(&masses[0]));
        }
        let legacy = tiny_address_model()?;
        assert!(legacy
            .forward_geometric_potential_native(&ids, 1, 10, &span, &potential)
            .is_err());
        let vars = model.variables();
        let pair = vars
            .get("layers.02.read.address.pair")
            .ok_or_else(|| invalid("pair absent"))?;
        let old = pair.as_tensor().copy()?;
        pair.set(&old.affine(1.0, 0.001)?)?;
        assert!(model
            .forward_geometric_potential_native(&ids, 1, 10, &span, &potential)
            .is_err());
        pair.set(&old)?;
        assert_eq!(bits(&model.forward(&ids, 1, 10)?)?, before);
        assert_eq!(
            bits(&model.forward_geometric_span_native(&ids, 1, 10, &span)?)?,
            before
        );
        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn native_geometric_span_reader_and_source_replay_bind_saved_actions() -> Result<()> {
        use crate::geometric_span_native::SpanSourceBinding;
        let root = std::env::temp_dir().join(format!(
            "native-geometric-span-replay-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos(),
        ));
        let model = tiny_span_model()?;
        model.save(&root)?;
        let source = SpanSourceBinding::from_files(
            &root.join("model.safetensors"),
            &root.join("config.json"),
            b"test-token-registry/1",
        )?;
        let p = model.params()?;
        let span = model
            .geometric_span()
            .ok_or_else(|| invalid("span fixture absent"))?;
        let compiled = CompiledSpanActions::compile(p.get("embedding.weight")?, span, &source)?;
        let ids = [1, 2, 3, 4, 5, 1, 3, 2, 4, 6];
        let expected = bits(&model.forward(&ids, 1, 10)?)?;
        assert_eq!(
            bits(&model.forward_geometric_span_native(&ids, 1, 10, &compiled)?)?,
            expected
        );
        for head in 0..2 {
            let target = ReadBindingTarget {
                layer: 2,
                head,
                rows: vec![ReadBinding {
                    batch: 0,
                    query: 9,
                    sources: vec![4],
                }],
            };
            assert_eq!(
                bits(&model.read_binding_masses(&ids, 1, 10, &target)?)?,
                bits(
                    &model.read_binding_masses_geometric_span_native(
                        &ids, 1, 10, &target, &compiled
                    )?
                )?
            );
        }
        assert_eq!(bits(&model.forward(&ids, 1, 10)?)?, expected);
        let legacy = tiny_address_model()?;
        assert!(legacy
            .forward_geometric_span_native(&ids, 1, 10, &compiled)
            .is_err());
        let variables = model.variables();
        let embedding = variables
            .get("embedding.weight")
            .ok_or_else(|| invalid("embedding absent"))?;
        let old = embedding.as_tensor().copy()?;
        embedding.set(&old.affine(1.0, 0.001)?)?;
        assert!(model
            .forward_geometric_span_native(&ids, 1, 10, &compiled)
            .is_err());
        embedding.set(&old)?;
        assert_eq!(
            bits(&model.forward_geometric_span_native(&ids, 1, 10, &compiled)?)?,
            expected
        );
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn geometric_span_save_reload_legacy_and_metadata_refusals() -> Result<()> {
        let root =
            std::env::temp_dir().join(format!("stack-geometric-span-{}", std::process::id()));
        let model = tiny_span_model()?;
        let ids = [1, 2, 3, 4, 5, 1, 3, 2, 4, 6];
        let expected = bits(&model.forward(&ids, 1, 10)?)?;
        model.save(&root)?;
        let loaded = StackModel::load(&root, &cpu())?;
        assert_eq!(loaded.geometric_span(), model.geometric_span());
        assert_eq!(loaded.parameter_count(), model.parameter_count());
        assert_eq!(bits(&loaded.forward(&ids, 1, 10)?)?, expected);
        let config_bytes = fs::read(root.join("config.json"))?;
        let record_bytes = fs::read(root.join(GEOMETRIC_SPAN_RECORD))?;
        fs::remove_file(root.join(GEOMETRIC_SPAN_RECORD))?;
        assert!(StackModel::load(&root, &cpu()).is_err());
        for field in [
            "schema",
            "operation",
            "config_sha256",
            "model_sha256",
            "root_table_sha256",
            "action_order",
        ] {
            let mut record: serde_json::Value = serde_json::from_slice(&record_bytes)?;
            if field == "root_table_sha256" {
                record["span"][field] = serde_json::json!("unknown");
            } else if field == "action_order" {
                record["span"][field] = serde_json::json!(["open", "hold", "append", "commit"]);
            } else {
                record[field] = serde_json::json!("unknown");
            }
            let bytes = serde_json::to_vec_pretty(&record)?;
            let mut config: serde_json::Value = serde_json::from_slice(&config_bytes)?;
            config["geometric_span"]["record_sha256"] =
                serde_json::json!(hex::encode(Sha256::digest(&bytes)));
            fs::write(
                root.join("config.json"),
                serde_json::to_vec_pretty(&config)?,
            )?;
            fs::write(root.join(GEOMETRIC_SPAN_RECORD), bytes)?;
            assert!(StackModel::load(&root, &cpu()).is_err(), "resealed {field}");
        }
        model.save(&root)?;
        let mut config: serde_json::Value = serde_json::from_slice(&config_bytes)?;
        config.as_object_mut().unwrap().remove("geometric_span");
        fs::write(
            root.join("config.json"),
            serde_json::to_vec_pretty(&config)?,
        )?;
        assert!(StackModel::load(&root, &cpu()).is_err());
        let legacy = tiny_address_model()?;
        legacy.save(&root)?;
        assert!(!root.join(GEOMETRIC_SPAN_RECORD).exists());
        let reloaded = StackModel::load(&root, &cpu())?;
        assert!(reloaded.geometric_span().is_none());
        assert_eq!(
            bits(&legacy.forward(&ids, 1, 10)?)?,
            bits(&reloaded.forward(&ids, 1, 10)?)?
        );
        fs::write(root.join(GEOMETRIC_SPAN_RECORD), &record_bytes)?;
        assert!(StackModel::load(&root, &cpu()).is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn geometric_span_refuses_unprovenanced_inputs_and_old_latch_overrides() -> Result<()> {
        let mut model = tiny_span_model()?;
        let ids = [1, 2, 3, 4, 5, 6];
        assert!(model.hidden_from_input(model.embed(&ids, 1, 6)?).is_err());
        let prefix = model.run_layers(model.embed(&ids, 1, 6)?, 0..2)?;
        assert!(model.run_layers(prefix, 2..3).is_err());
        assert!(model.forward_hard_read_identity_latch(&ids, 1, 6).is_err());
        assert!(model
            .read_identity_latch_gate_logits(&ids, 1, 6, 2)
            .is_err());
        assert!(model
            .set_read_identity_latch(ReadIdentityLatch::Held)
            .is_err());
        assert!(model.set_read_identity_carry(true).is_err());
        assert!(model
            .set_served_representation(Some(Arc::new(D11Interim)))
            .is_err());
        assert!(model.forward(&ids, 1, 6).is_ok());
        Ok(())
    }

    #[test]
    fn geometric_address_removes_qk_maps_and_preserves_causal_label_free_forward() -> Result<()> {
        let mut model = tiny_address_model()?;
        let ids: Vec<u32> = (0..16).map(|i| (i * 5 + 1) % 37).collect();
        let expected = model.forward(&ids, 2, 8)?;
        let count = model.parameter_count();
        for suffix in ["query", "key", "query_identity", "key_identity"] {
            let name = layer_name(2, &format!("read.{suffix}.weight"));
            assert!(!model.variables.contains_key(&name));
            // Even an accidentally reintroduced obsolete map cannot affect
            // the new forward. It is not part of a valid saved inventory.
            model.variables.insert(
                name,
                Var::from_tensor(&Tensor::full(1000.0f32, (16, 16), &cpu())?)?,
            );
        }
        assert_eq!(bits(&expected)?, bits(&model.forward(&ids, 2, 8)?)?);
        remove_address_projection_parameters(&mut model.variables);
        assert_eq!(model.parameter_count(), count);
        assert!(!model.variables.contains_key("layers.02.read.log_beta"));
        assert!(!model.variables.contains_key("layers.02.read.offset"));

        let p = model.params()?;
        let target = binding_rows(2);
        let (hidden, masses) = model.hidden_with_binding(&p, &ids, 2, 8, &target)?;
        assert!(max_abs_gap(&hidden, &model.hidden(&ids, 2, 8)?)? < 1e-7);
        assert!(masses
            .to_vec1::<f32>()?
            .iter()
            .all(|p| *p > 0.0 && *p < 1.0));
        let mut changed_labels = target.clone();
        changed_labels.rows[0].sources = vec![0, 3];
        let (second, _) = model.hidden_with_binding(&p, &ids, 2, 8, &changed_labels)?;
        assert!(max_abs_gap(&hidden, &second)? < 1e-7);
        let mut future = ids.clone();
        future[6] = 23;
        future[7] = 17;
        let changed = model.forward(&future, 2, 8)?;
        assert_eq!(
            max_abs_gap(&expected.narrow(0, 0, 6)?, &changed.narrow(0, 0, 6)?)?,
            0.0
        );
        assert_eq!(
            max_abs_gap(&expected.narrow(0, 8, 8)?, &changed.narrow(0, 8, 8)?)?,
            0.0
        );
        // The normal language logits actually consume this reader.
        let out = &model.variables()["layers.02.read.out.weight"];
        let original = out.as_tensor().copy()?;
        out.set(&out.as_tensor().zeros_like()?)?;
        assert!(max_abs_gap(&expected, &model.forward(&ids, 2, 8)?)? > 0.0);
        out.set(&original)?;
        assert_eq!(bits(&expected)?, bits(&model.forward(&ids, 2, 8)?)?);
        Ok(())
    }

    #[test]
    fn geometric_address_source_mass_matches_scalar_causal_null_age_reference() -> Result<()> {
        let model = tiny_address_model()?;
        let ids = [1, 7, 3, 9, 11, 4, 6, 8];
        let target = ReadBindingTarget {
            layer: 2,
            head: 1,
            rows: vec![ReadBinding {
                batch: 0,
                query: 6,
                sources: vec![1, 4],
            }],
        };
        let p = model.params()?;
        let x = model.run_layers(model.embed(&ids, 1, 8)?, 0..2)?;
        let u = model.norm(&p, &x, "layers.02.read_norm.weight")?;
        let prior = model
            .read_latch_inputs(&p, 2, &u, ReadIdentityLatch::Held, LatchGates::Soft)?
            .0;
        let scores = geometric_address::score(
            &u,
            &prior,
            model.geometric_address().unwrap(),
            &model.address_weights(&p, 2)?,
        )?
        .flatten_all()?
        .to_vec1::<f32>()?;
        let null = StackModel::linear(&u, p.layer(2, "read.null.weight")?)?
            .broadcast_add(p.layer(2, "read.null.bias")?)?
            .transpose(1, 2)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let ages = p.layer(2, "read.age")?.flatten_all()?.to_vec1::<f32>()?;
        let value = model
            .heads(
                &StackModel::linear(&u, p.layer(2, "read.value.weight")?)?,
                1,
                8,
            )?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let (heads, time, width) = (2, 8, 8);
        let mut reference = vec![0f32; heads * time * width];
        let mut expected_mass = 0.0;
        for head in 0..heads {
            for query in 0..time {
                let mut row = vec![f64::from(null[head * time + query])];
                for source in 0..=query {
                    row.push(f64::from(
                        scores[(head * time + query) * time + source]
                            + ages[head * model.config.context + query - source],
                    ));
                }
                let max = row.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let denominator: f64 = row.iter().map(|x| (x - max).exp()).sum();
                for source in 0..=query {
                    let mass = (row[source + 1] - max).exp() / denominator;
                    if head == 1 && query == 6 && [1, 4].contains(&source) {
                        expected_mass += mass;
                    }
                    for coordinate in 0..width {
                        reference[(head * time + query) * width + coordinate] += (mass
                            * f64::from(value[(head * time + source) * width + coordinate]))
                            as f32;
                    }
                }
            }
        }
        let reference = Tensor::from_vec(reference, (1, heads, time, width), &cpu())?;
        let reference = StackModel::linear(
            &model.merge_heads(&reference, 1, time)?,
            p.layer(2, "read.out.weight")?,
        )?;
        let actual = model.geometric_read(&p, 2, &x, &mut None, &mut None, LatchGates::Soft)?;
        assert!(max_abs_gap(&reference, &actual)? < 1e-6);
        let mass = model
            .read_binding_masses(&ids, 1, 8, &target)?
            .to_vec1::<f32>()?[0];
        assert!((f64::from(mass) - expected_mass).abs() < 1e-6);
        // NoRead has a zero value but a real denominator contribution.
        let null_bias = &model.variables()["layers.02.read.null.bias"];
        null_bias.set(&Tensor::full(20.0f32, 2, &cpu())?)?;
        assert!(
            model
                .read_binding_masses(&ids, 1, 8, &target)?
                .to_vec1::<f32>()?[0]
                < mass * 1e-5
        );
        Ok(())
    }

    #[test]
    fn geometric_address_language_binding_and_capture_gradients_reach_actual_context() -> Result<()>
    {
        let model = tiny_address_model()?;
        let ids: Vec<u32> = (0..16).map(|i| (i * 5 + 1) % 37).collect();
        let targets: Vec<u32> = ids.iter().map(|x| (x + 3) % 37).collect();
        let language = model.loss(&ids, &targets, 2, 8)?;
        let grads = language.backward()?;
        for name in [
            "layers.02.read.address.content_unary",
            "layers.02.read.address.context_unary",
            "layers.02.read.address.pair",
            "layers.02.read.address.content_radius",
            "layers.02.read.identity_gate.weight",
            "layers.02.read.identity_gate.bias",
            "layers.01.rec.in.weight",
            "embedding.weight",
        ] {
            let gradient = grads
                .get(&model.variables()[name])
                .ok_or_else(|| invalid(format!("language misses {name}")))?;
            let size = gradient.abs()?.max_all()?.to_scalar::<f32>()?;
            assert!(
                size.is_finite() && size > 0.0,
                "pure language {name}: {size}"
            );
        }
        let target = binding_rows(2);
        let joint = model.loss_with_binding(&ids, &targets, None, 2, 8, &target)?;
        assert!((language.to_scalar::<f32>()? - joint.language.to_scalar::<f32>()?).abs() < 1e-6);
        let source = joint.binding.backward()?;
        for name in [
            "layers.02.read.address.pair",
            "layers.02.read.identity_gate.weight",
            "layers.01.rec.in.weight",
        ] {
            let gradient = source
                .get(&model.variables()[name])
                .ok_or_else(|| invalid(format!("source misses {name}")))?;
            assert!(
                gradient.abs()?.max_all()?.to_scalar::<f32>()? > 0.0,
                "{name}"
            );
        }
        let capture_targets: Vec<u32> = (0..16).map(|i| u32::from(i % 4 == 1)).collect();
        let capture_loss = || -> Result<Tensor> {
            let logits = model
                .read_identity_latch_gate_logits(&ids, 2, 8, 2)?
                .reshape((16, 1))?;
            logits_cross_entropy(
                &Tensor::cat(&[&logits.zeros_like()?, &logits], 1)?,
                &capture_targets,
                None,
            )
        };
        let first = capture_loss()?.backward()?;
        for name in [
            "layers.02.read.identity_gate.weight",
            "layers.02.read.identity_gate.bias",
        ] {
            let variable = &model.variables()[name];
            let gradient = first
                .get(variable)
                .ok_or_else(|| invalid("capture gate gradient"))?;
            assert!(gradient.abs()?.max_all()?.to_scalar::<f32>()? > 0.0);
            variable.set(&variable.as_tensor().sub(&gradient.affine(0.1, 0.0)?)?)?;
        }
        let second = capture_loss()?.backward()?;
        for name in [
            "layers.01.rec.in.weight",
            "layers.02.read_norm.weight",
            "embedding.weight",
        ] {
            let gradient = second
                .get(&model.variables()[name])
                .ok_or_else(|| invalid("capture context gradient"))?;
            assert!(
                gradient.abs()?.max_all()?.to_scalar::<f32>()? > 0.0,
                "{name}"
            );
        }
        Ok(())
    }

    #[test]
    fn geometric_address_save_reload_is_bound_and_legacy_saves_stay_unchanged() -> Result<()> {
        let root =
            std::env::temp_dir().join(format!("stack-geometric-address-{}", std::process::id()));
        let mut model = tiny_address_model()?;
        let ids = [1, 7, 3, 9, 11, 4, 6, 8];
        let expected = bits(&model.forward(&ids, 1, 8)?)?;
        model.save(&root)?;
        let loaded = StackModel::load(&root, &cpu())?;
        assert_eq!(loaded.geometric_address(), model.geometric_address());
        assert_eq!(loaded.parameter_count(), model.parameter_count());
        assert_eq!(bits(&loaded.forward(&ids, 1, 8)?)?, expected);
        let config_bytes = fs::read(root.join("config.json"))?;
        let record_bytes = fs::read(root.join(GEOMETRIC_ADDRESS_RECORD))?;
        fs::remove_file(root.join(GEOMETRIC_ADDRESS_RECORD))?;
        assert!(StackModel::load(&root, &cpu()).is_err());
        for field in [
            "schema",
            "operation",
            "config_sha256",
            "model_sha256",
            "root_table_sha256",
        ] {
            let mut record: serde_json::Value = serde_json::from_slice(&record_bytes)?;
            if field == "root_table_sha256" {
                record["address"][field] = serde_json::json!("00");
            } else {
                record[field] = serde_json::json!("unknown");
            }
            let bytes = serde_json::to_vec_pretty(&record)?;
            let mut config: serde_json::Value = serde_json::from_slice(&config_bytes)?;
            config["geometric_address"]["record_sha256"] =
                serde_json::json!(hex::encode(Sha256::digest(&bytes)));
            fs::write(
                root.join("config.json"),
                serde_json::to_vec_pretty(&config)?,
            )?;
            fs::write(root.join(GEOMETRIC_ADDRESS_RECORD), bytes)?;
            assert!(StackModel::load(&root, &cpu()).is_err(), "resealed {field}");
        }
        model.save(&root)?;
        let mut config: serde_json::Value = serde_json::from_slice(&config_bytes)?;
        config.as_object_mut().unwrap().remove("geometric_address");
        fs::write(
            root.join("config.json"),
            serde_json::to_vec_pretty(&config)?,
        )?;
        assert!(StackModel::load(&root, &cpu()).is_err());
        model.save(&root)?;
        let mut weights = fs::read(root.join("model.safetensors"))?;
        weights.push(0);
        fs::write(root.join("model.safetensors"), weights)?;
        assert!(StackModel::load(&root, &cpu()).is_err());
        let plain = StackModel::new(model.config.clone(), &cpu())?;
        plain.save(&root)?;
        assert!(!root.join(GEOMETRIC_ADDRESS_RECORD).exists());
        assert_eq!(
            fs::read(root.join("config.json"))?,
            serde_json::to_vec_pretty(&plain.config)?
        );
        assert_eq!(
            bits(&StackModel::load(&root, &cpu())?.forward(&ids, 1, 8)?)?,
            bits(&plain.forward(&ids, 1, 8)?)?
        );
        fs::write(root.join(GEOMETRIC_ADDRESS_RECORD), &record_bytes)?;
        assert!(StackModel::load(&root, &cpu()).is_err());
        model.variables.remove("layers.02.read.address.pair");
        model.save(&root)?;
        assert!(StackModel::load(&root, &cpu()).is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn geometric_address_refuses_incompatible_views_without_mutation() -> Result<()> {
        let mut config = tiny(StackArch::Geometric, "rra", ReadScore::Lorentz, true);
        config.width = 32;
        let mut model = StackModel::new(config.clone(), &cpu())?;
        let (bytes, _) =
            crate::stack_export::export_stack(&model, serde_json::json!({}), None, None)?;
        let artifact = uor_r4_lut::format::StackArtifact::parse(bytes)
            .map_err(|error| invalid(error.to_string()))?;
        let address = GeometricAddressConfig::new(config.width, config.heads)?;
        assert!(model.set_geometric_address(address.clone()).is_err());
        model.set_read_identity_latch(ReadIdentityLatch::Held)?;
        model.set_geometric_address(address.clone())?;
        model.set_geometric_address(address)?;
        let count = model.parameter_count();
        assert!(model
            .set_served_representation(Some(Arc::new(D11Interim)))
            .is_err());
        assert!(
            crate::stack_export::export_stack(&model, serde_json::json!({}), None, None).is_err()
        );
        assert!(crate::stack_export::stack_grid_reference(&model, &artifact).is_err());
        assert!(model.add_pointer(PointerConfig::new(8), 7).is_err());
        assert!(model
            .set_select(Some(FlockSelect {
                sink: 0,
                window: 1,
                k: 1
            }))
            .is_err());
        assert!(model
            .set_read_identity_latch(ReadIdentityLatch::Local)
            .is_err());
        assert!(model.set_read_identity_carry(true).is_err());
        let zero = Tensor::zeros((1, 4, 32), DType::F32, &cpu())?;
        assert!(model
            .forward_read_identity_latch_replay(&[1, 2, 3, 4], 1, 4, &zero)
            .is_err());
        assert!(model
            .forward_read_projection_replay(&[1, 2, 3, 4], 1, 4, &zero, &zero)
            .is_err());
        assert_eq!(model.parameter_count(), count);
        assert!(model.config.pointer.is_none() && model.config.select.is_none());
        Ok(())
    }

    #[test]
    fn hard_read_identity_latch_threshold_and_causal_state_match_reference() -> Result<()> {
        let z = Tensor::from_vec(
            vec![-1000.0f32, -1.0, -1e-12, -0.0, 0.0, 1e-12, 1.0, 1000.0],
            8,
            &cpu(),
        )?;
        assert_eq!(
            LatchGates::Hard.apply(&z)?.to_vec1::<f32>()?,
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0]
        );
        // The sigmoid comparison agrees away from its rounding interval; the
        // signed-zero tie captures. Tiny negative logits must still HOLD.
        let ordinary = Tensor::from_vec(vec![-2.0f32, -0.1, 0.0, 0.1, 2.0], 5, &cpu())?;
        assert_eq!(
            bits(&LatchGates::Hard.apply(&ordinary)?)?,
            bits(
                &candle_nn::ops::sigmoid(&ordinary)?
                    .ge(0.5)?
                    .to_dtype(DType::F32)?
            )?
        );
        for invalid_logit in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(LatchGates::Hard
                .apply(&Tensor::from_vec(vec![invalid_logit], 1, &cpu())?)
                .is_err());
        }
        for mode in [ReadIdentityLatch::Held, ReadIdentityLatch::Local] {
            let mut model = StackModel::new(
                tiny(StackArch::Geometric, "a", ReadScore::Lorentz, false),
                &cpu(),
            )?;
            model.set_read_identity_latch(mode)?;
            let mut weights = vec![0.0f32; 32];
            weights[0] = 1.0;
            weights[16] = 0.5;
            model.variables()["layers.00.read.identity_gate.weight"].set(&Tensor::from_vec(
                weights,
                (1, 32),
                &cpu(),
            )?)?;
            model.variables()["layers.00.read.identity_gate.bias"].set(&Tensor::zeros(
                1,
                DType::F32,
                &cpu(),
            )?)?;
            let markers = [2.0f32, -2.0, -2.0, 1.0, 0.0, -2.0, 2.0];
            let mut values = vec![0.0f32; 7 * 16];
            for (t, &marker) in markers.iter().enumerate() {
                values[t * 16] = marker;
                for c in 1..16 {
                    values[t * 16 + c] = (16 * t + c) as f32 / 32.0;
                }
                values[t * 16 + 15] = -0.0;
            }
            let u = Tensor::from_vec(values.clone(), (1, 7, 16), &cpu())?;
            let (identity, gates) =
                model.read_latch_inputs(&model.params()?, 0, &u, mode, LatchGates::Hard)?;
            let identity = identity.flatten_all()?.to_vec1::<f32>()?;
            let gates = gates.flatten_all()?.to_vec1::<f32>()?;
            let mut state = vec![0.0f32; 16];
            for t in 0..7 {
                assert_eq!(
                    identity[t * 16..(t + 1) * 16]
                        .iter()
                        .map(|x| x.to_bits())
                        .collect::<Vec<_>>(),
                    state.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                    "{mode:?}: prior identity at {t}"
                );
                let z = markers[t] + if t == 0 { 0.0 } else { 0.5 * markers[t - 1] };
                assert_eq!(gates[t], if z >= 0.0 { 1.0 } else { 0.0 });
                if z >= 0.0 {
                    state.copy_from_slice(&values[t * 16..(t + 1) * 16]);
                } else if mode == ReadIdentityLatch::Local {
                    state.fill(0.0);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn hard_read_identity_latch_forward_binding_and_later_gates_share_policy() -> Result<()> {
        let ids: Vec<u32> = (0..16).map(|i| (i * 5 + 1) % 37).collect();
        for mode in [ReadIdentityLatch::Held, ReadIdentityLatch::Local] {
            let mut model = StackModel::new(
                tiny(StackArch::Geometric, "raa", ReadScore::Lorentz, true),
                &cpu(),
            )?;
            assert!(model.forward_hard_read_identity_latch(&ids, 2, 8).is_err());
            assert!(model.read_identity_latch_hard_gates(&ids, 2, 8, 1).is_err());
            assert!(model
                .read_binding_masses_hard_read_identity_latch(&ids, 2, 8, &binding_rows(2))
                .is_err());
            model.set_read_identity_latch(mode)?;
            for layer in 1..3 {
                let w = &model.variables()[&layer_name(layer, "read.identity_gate.weight")];
                w.set(&random(
                    &mut Initializer(791 + layer as u64),
                    w.dims(),
                    0.15,
                ))?;
                model.variables()[&layer_name(layer, "read.identity_gate.bias")]
                    .set(&Tensor::zeros(1, DType::F32, &cpu())?)?;
            }
            let before: BTreeMap<String, Vec<u32>> = model
                .variables()
                .iter()
                .map(|(name, var)| Ok((name.clone(), bits(var.as_tensor())?)))
                .collect::<Result<_>>()?;
            let soft = bits(&model.forward(&ids, 2, 8)?)?;
            let hard = model.forward_hard_read_identity_latch(&ids, 2, 8)?;
            let p = model.params()?;
            let mut x = model.embed(&ids, 2, 8)?;
            for layer in 0..3 {
                if layer > 0 {
                    let u = model.norm(&p, &x, &layer_name(layer, "read_norm.weight"))?;
                    let expected = LatchGates::Hard
                        .apply(&model.read_latch_gate_logits(&p, layer, &u)?)?
                        .squeeze(2)?;
                    assert_eq!(
                        bits(&model.read_identity_latch_hard_gates(&ids, 2, 8, layer)?)?,
                        bits(&expected)?
                    );
                }
                let mixed = if layer == 0 {
                    model.recurrence(&p, layer, &x, &mut None)?
                } else {
                    model.geometric_read(&p, layer, &x, &mut None, &mut None, LatchGates::Hard)?
                };
                x = x.add(&mixed)?;
                x = x.add(&model.mlp(&p, layer, &x, &mut None)?)?;
            }
            assert_eq!(bits(&hard)?, bits(&model.head(&model.finish(x)?)?)?);
            // The auxiliary mask observes the same hard read and is removed
            // before its output map; changing the label cannot change logits.
            let mut target = binding_rows(2);
            for source in [1, 3] {
                target.rows[0].sources = vec![source];
                let (hidden, mass) =
                    model.hidden_with_binding_policy(&p, &ids, 2, 8, &target, LatchGates::Hard)?;
                assert_eq!(bits(&hard)?, bits(&model.head(&hidden)?)?);
                assert_eq!(
                    bits(&mass)?,
                    bits(
                        &model.read_binding_masses_hard_read_identity_latch(&ids, 2, 8, &target,)?
                    )?
                );
            }
            let mut future = ids.clone();
            future[6] = (future[6] + 1) % 37;
            let changed = model.forward_hard_read_identity_latch(&future, 2, 8)?;
            assert_eq!(
                bits(&hard.narrow(0, 0, 6)?)?,
                bits(&changed.narrow(0, 0, 6)?)?
            );
            for layer in 1..3 {
                assert_eq!(
                    bits(
                        &model
                            .read_identity_latch_hard_gates(&ids, 2, 8, layer)?
                            .narrow(1, 0, 6)?
                    )?,
                    bits(
                        &model
                            .read_identity_latch_hard_gates(&future, 2, 8, layer)?
                            .narrow(1, 0, 6)?
                    )?
                );
            }
            assert_eq!(soft, bits(&model.forward(&ids, 2, 8)?)?);
            for (name, var) in model.variables() {
                assert_eq!(before[name], bits(var.as_tensor())?, "mutated {name}");
            }
            assert!(model.read_identity_latch_hard_gates(&ids, 2, 8, 0).is_err());
        }
        Ok(())
    }

    #[test]
    fn read_identity_latch_replay_matches_hard_reference_and_rejects_bad_inputs() -> Result<()> {
        let ids = [1, 7, 3, 9, 11, 4, 6, 8];
        for mode in [ReadIdentityLatch::Held, ReadIdentityLatch::Local] {
            let mut model = StackModel::new(
                tiny(StackArch::Geometric, "rra", ReadScore::Lorentz, true),
                &cpu(),
            )?;
            model.set_read_identity_latch(mode)?;
            let bias = &model.variables()["layers.02.read.identity_gate.bias"];
            bias.set(&Tensor::from_vec(vec![1.0f32], 1, &cpu())?)?;
            let soft = bits(&model.forward(&ids, 1, 8)?)?;
            let u = model.read_identity_latch_replay_input(&ids, 1, 8)?;
            let p = model.params()?;
            let prior = model
                .read_latch_inputs(&p, 2, &u, mode, LatchGates::Hard)?
                .0;
            assert_eq!(
                bits(&model.forward_hard_read_identity_latch(&ids, 1, 8)?)?,
                bits(&model.forward_read_identity_latch_replay(&ids, 1, 8, &prior)?)?
            );
            let target = ReadBindingTarget {
                layer: 2,
                head: 0,
                rows: vec![ReadBinding {
                    batch: 0,
                    query: 7,
                    sources: vec![3],
                }],
            };
            assert_eq!(
                bits(&model.read_binding_masses_hard_read_identity_latch(&ids, 1, 8, &target)?)?,
                bits(
                    &model
                        .read_binding_masses_identity_latch_replay(&ids, 1, 8, &target, &prior)?
                )?
            );
            assert_eq!(soft, bits(&model.forward(&ids, 1, 8)?)?);
            assert!(model
                .forward_read_identity_latch_replay(&ids, 1, 8, &u)
                .is_err());
            let nan = Tensor::full(f32::NAN, (1, 8, 16), &cpu())?;
            assert!(model
                .forward_read_identity_latch_replay(&ids, 1, 8, &nan)
                .is_err());
            assert!(model
                .forward_read_identity_latch_replay(&ids, 1, 8, &prior.narrow(1, 0, 7)?)
                .is_err());
        }
        let mut wrong = StackModel::new(
            tiny(StackArch::Geometric, "raa", ReadScore::Lorentz, true),
            &cpu(),
        )?;
        wrong.set_read_identity_latch(ReadIdentityLatch::Held)?;
        assert!(wrong.read_identity_latch_replay_input(&ids, 1, 8).is_err());
        Ok(())
    }

    #[test]
    fn read_projection_replay_matches_identity_reference_and_is_per_call() -> Result<()> {
        let ids = [1, 7, 3, 9, 11, 4, 6, 8];
        for mode in [ReadIdentityLatch::Held, ReadIdentityLatch::Local] {
            let mut model = StackModel::new(
                tiny(StackArch::Geometric, "rra", ReadScore::Lorentz, true),
                &cpu(),
            )?;
            model.set_read_identity_latch(mode)?;
            let ordinary = bits(&model.forward(&ids, 1, 8)?)?;
            let before: BTreeMap<_, _> = model
                .variables()
                .iter()
                .map(|(name, var)| Ok((name.clone(), bits(var.as_tensor())?)))
                .collect::<Result<_>>()?;
            let u = model.read_identity_latch_replay_input(&ids, 1, 8)?;
            let p = model.params()?;
            let prior = model
                .read_latch_inputs(&p, 2, &u, mode, LatchGates::Hard)?
                .0;
            let project = |part: &str| -> Result<Tensor> {
                Ok(
                    StackModel::linear(&u, p.layer(2, &format!("read.{part}.weight"))?)?.add(
                        &StackModel::linear(
                            &prior,
                            p.layer(2, &format!("read.{part}_identity.weight"))?,
                        )?,
                    )?,
                )
            };
            let query = project("query")?;
            let key = project("key")?;
            assert_eq!(
                bits(&model.forward_read_identity_latch_replay(&ids, 1, 8, &prior)?)?,
                bits(&model.forward_read_projection_replay(&ids, 1, 8, &query, &key)?)?
            );
            for head in 0..model.config.heads {
                let target = ReadBindingTarget {
                    layer: 2,
                    head,
                    rows: vec![ReadBinding {
                        batch: 0,
                        query: 7,
                        sources: vec![3],
                    }],
                };
                assert_eq!(
                    bits(
                        &model.read_binding_masses_identity_latch_replay(
                            &ids, 1, 8, &target, &prior
                        )?
                    )?,
                    bits(&model.read_binding_masses_projection_replay(
                        &ids, 1, 8, &target, &query, &key
                    )?)?
                );
            }
            let zero = query.zeros_like()?;
            model.forward_read_projection_replay(&ids, 1, 8, &zero, &zero)?;
            assert_eq!(ordinary, bits(&model.forward(&ids, 1, 8)?)?);
            for (name, var) in model.variables() {
                assert_eq!(before[name], bits(var.as_tensor())?);
            }
            let nan = Tensor::full(f32::NAN, (1, 8, 16), &cpu())?;
            assert!(model
                .forward_read_projection_replay(&ids, 1, 8, &nan, &key)
                .is_err());
            assert!(model
                .forward_read_projection_replay(&ids, 1, 8, &query, &nan)
                .is_err());
            assert!(model
                .forward_read_projection_replay(&ids, 1, 8, &query.narrow(1, 0, 7)?, &key)
                .is_err());
            assert!(model
                .forward_read_projection_replay(&ids, 1, 8, &query.to_dtype(DType::F64)?, &key)
                .is_err());
        }
        let mut wrong = StackModel::new(
            tiny(StackArch::Geometric, "raa", ReadScore::Lorentz, true),
            &cpu(),
        )?;
        wrong.set_read_identity_latch(ReadIdentityLatch::Held)?;
        let zero = Tensor::zeros((1, 8, 16), DType::F32, &cpu())?;
        assert!(wrong
            .forward_read_projection_replay(&ids, 1, 8, &zero, &zero)
            .is_err());
        Ok(())
    }

    #[test]
    fn hard_read_identity_latch_is_per_call_and_save_remains_soft() -> Result<()> {
        let root =
            std::env::temp_dir().join(format!("stack-hard-read-latch-{}", std::process::id()));
        let ids = [1, 7, 3, 9, 11, 4, 6, 8];
        for mode in [ReadIdentityLatch::Held, ReadIdentityLatch::Local] {
            let dir = root.join(format!("{mode:?}"));
            let mut model = StackModel::new(
                tiny(StackArch::Geometric, "ra", ReadScore::Lorentz, true),
                &cpu(),
            )?;
            model.set_read_identity_latch(mode)?;
            model.save(&dir)?;
            let names = [
                "config.json",
                "model.safetensors",
                READ_IDENTITY_LATCH_RECORD,
            ];
            let saved: Vec<Vec<u8>> = names
                .iter()
                .map(|name| fs::read(dir.join(name)))
                .collect::<std::io::Result<_>>()?;
            let soft = bits(&model.forward(&ids, 1, 8)?)?;
            let hard = bits(&model.forward_hard_read_identity_latch(&ids, 1, 8)?)?;
            // Initial bias -2 makes the hard state exactly zero; the ordinary
            // model retains its nonzero soft contribution after the call.
            assert_ne!(soft, hard);
            assert_eq!(bits(&model.forward(&ids, 1, 8)?)?, soft);
            model.save(&dir)?;
            for (name, expected) in names.iter().zip(&saved) {
                assert_eq!(&fs::read(dir.join(name))?, expected, "changed {name}");
            }
            let loaded = StackModel::load(&dir, &cpu())?;
            assert_eq!(loaded.read_identity_latch(), Some(mode));
            assert_eq!(bits(&loaded.forward(&ids, 1, 8)?)?, soft);
            assert_eq!(
                bits(&loaded.forward_hard_read_identity_latch(&ids, 1, 8)?)?,
                hard
            );
        }
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn read_identity_latch_held_and_local_match_causal_reference_and_gradients() -> Result<()> {
        let ids: Vec<u32> = (0..16).map(|i| (i * 5 + 1) % 37).collect();
        for mode in [ReadIdentityLatch::Held, ReadIdentityLatch::Local] {
            for score in [ReadScore::Dot, ReadScore::Lorentz] {
                let mut model =
                    StackModel::new(tiny(StackArch::Geometric, "ra", score, true), &cpu())?;
                let baseline = bits(&model.forward(&ids, 2, 8)?)?;
                let base_count = model.parameter_count();
                model.set_read_identity_latch(mode)?;
                assert_eq!(
                    model.parameter_count(),
                    base_count + 2 * 16 * 16 + 2 * 16 + 1
                );
                assert_eq!(
                    bits(model.variables()["layers.01.read.query_identity.weight"].as_tensor())?,
                    bits(model.variables()["layers.01.read.query.weight"].as_tensor())?
                );
                let initial_gates = model
                    .read_identity_latch_gates(&ids, 2, 8, 1)?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for gate in initial_gates {
                    assert!((gate - sigmoid(-2.0)).abs() < 1e-7);
                }
                // Nonzero gate weights exercise both current and previous input.
                let gate_weight = &model.variables()["layers.01.read.identity_gate.weight"];
                gate_weight.set(&random(&mut Initializer(721), gate_weight.dims(), 0.03))?;
                let p = model.params()?;
                let x = model.run_layers(model.embed(&ids, 2, 8)?, 0..1)?;
                let u = model.norm(&p, &x, &layer_name(1, "read_norm.weight"))?;
                let (identity, gates) =
                    model.read_latch_inputs(&p, 1, &u, mode, LatchGates::Soft)?;
                let uv = u.to_vec3::<f32>()?;
                let gv = gates.squeeze(2)?.to_vec2::<f32>()?;
                let hv = identity.to_vec3::<f32>()?;
                let w = gate_weight.flatten_all()?.to_vec1::<f32>()?;
                for b in 0..2 {
                    let mut state = [0.0f32; 16];
                    for t in 0..8 {
                        let mut z = -2.0;
                        for c in 0..16 {
                            z += w[c] * uv[b][t][c];
                            if t > 0 {
                                z += w[16 + c] * uv[b][t - 1][c];
                            }
                            assert!((hv[b][t][c] - state[c]).abs() < 1e-6);
                        }
                        let g = sigmoid(z);
                        assert!((gv[b][t] - g).abs() < 1e-6);
                        for c in 0..16 {
                            state[c] = g * uv[b][t][c]
                                + if mode == ReadIdentityLatch::Held {
                                    (1.0 - g) * state[c]
                                } else {
                                    0.0
                                };
                        }
                    }
                }
                let project = |part: &str| -> Result<Tensor> {
                    let current =
                        StackModel::linear(&u, p.layer(1, &format!("read.{part}.weight"))?)?;
                    let result = if part == "value" {
                        current
                    } else {
                        current.add(&StackModel::linear(
                            &identity,
                            p.layer(1, &format!("read.{part}_identity.weight"))?,
                        )?)?
                    };
                    model.heads(&result, 2, 8)
                };
                let null = StackModel::linear(&u, p.layer(1, "read.null.weight")?)?
                    .broadcast_add(p.layer(1, "read.null.bias")?)?
                    .transpose(1, 2)?
                    .flatten_all()?;
                let age = p.layer(1, "read.age")?.narrow(1, 0, 8)?.flatten_all()?;
                let mut aux = vec![null, age];
                if score == ReadScore::Lorentz {
                    aux.push(p.layer(1, "read.log_beta")?.exp()?);
                    aux.push(p.layer(1, "read.offset")?.clone());
                }
                let read = fused_read_selected(
                    &project("query")?,
                    &project("key")?,
                    &project("value")?,
                    &Tensor::cat(&aux, 0)?,
                    score,
                    true,
                    true,
                    false,
                    None,
                )?;
                let want = StackModel::linear(
                    &model.merge_heads(&read, 2, 8)?,
                    p.layer(1, "read.out.weight")?,
                )?;
                let got =
                    model.geometric_read(&p, 1, &x, &mut None, &mut None, LatchGates::Soft)?;
                assert_eq!(max_abs_gap(&got, &want)?, 0.0);
                // Future changes cannot reach any prior prediction or gate.
                let mut future = ids.clone();
                future[6] = (future[6] + 1) % 37;
                assert_eq!(
                    max_abs_gap(
                        &model.forward(&ids, 2, 8)?.narrow(0, 0, 6)?,
                        &model.forward(&future, 2, 8)?.narrow(0, 0, 6)?
                    )?,
                    0.0
                );
                let target = binding_rows(1);
                let objective = || -> Result<Tensor> {
                    let loss = model.loss_with_binding(&ids, &ids, None, 2, 8, &target)?;
                    Ok(loss.binding.add(&loss.language.affine(0.1, 0.0)?)?)
                };
                let names = [
                    "layers.01.read.query_identity.weight",
                    "layers.01.read.key_identity.weight",
                    "layers.01.read.identity_gate.weight",
                    "layers.01.read.identity_gate.bias",
                    "layers.00.rec.in.weight",
                    "layers.01.read.age",
                ];
                let vars: Vec<Var> = names
                    .iter()
                    .map(|name| model.variables()[*name].clone())
                    .collect();
                let grads = objective()?.backward()?;
                for (name, var) in names.iter().zip(&vars) {
                    let grad = grads
                        .get(var)
                        .ok_or_else(|| invalid(format!("missing latch gradient {name}")))?;
                    let size = grad.abs()?.max_all()?.to_scalar::<f32>()?;
                    assert!(
                        size.is_finite() && size > 0.0,
                        "{mode:?} {score:?} {name} {size}"
                    );
                }
                check_gradient(&vars, objective, 2e-2)?;
                // Zero identity maps recover the original current-role reader.
                for name in [
                    "layers.01.read.query_identity.weight",
                    "layers.01.read.key_identity.weight",
                ] {
                    let var = &model.variables()[name];
                    var.set(&Tensor::zeros(var.shape(), DType::F32, &cpu())?)?;
                }
                assert_eq!(bits(&model.forward(&ids, 2, 8)?)?, baseline);
            }
        }
        Ok(())
    }

    #[test]
    fn read_identity_latch_gate_logits_match_sigmoid_and_causal_capture_gradients() -> Result<()> {
        let ids = [1, 7, 3, 9, 11, 4, 6, 8];
        let targets = [0, 1, 0, 0, 1, 0, 0, 0];
        // Equal total weight for CAPTURE and NO_CAPTURE, not a padding loss.
        let weights: Vec<f32> = targets
            .iter()
            .map(|&t| if t == 1 { 0.25 } else { 1.0 / 12.0 })
            .collect();
        for mode in [ReadIdentityLatch::Held, ReadIdentityLatch::Local] {
            let mut model = StackModel::new(
                tiny(StackArch::Geometric, "ra", ReadScore::Lorentz, true),
                &cpu(),
            )?;
            assert!(model
                .read_identity_latch_gate_logits(&ids, 1, 8, 1)
                .is_err());
            model.set_read_identity_latch(mode)?;
            assert!(model
                .read_identity_latch_gate_logits(&ids, 1, 8, 0)
                .is_err());
            let capture_loss = || -> Result<Tensor> {
                let z = model
                    .read_identity_latch_gate_logits(&ids, 1, 8, 1)?
                    .reshape((8, 1))?;
                let binary = Tensor::cat(&[&Tensor::zeros((8, 1), DType::F32, &cpu())?, &z], 1)?;
                logits_cross_entropy(&binary, &targets, Some(&weights))
            };
            let first = capture_loss()?.backward()?;
            for name in [
                "layers.01.read.identity_gate.weight",
                "layers.01.read.identity_gate.bias",
            ] {
                let var = &model.variables()[name];
                let grad = first
                    .get(var)
                    .ok_or_else(|| invalid(format!("capture misses {name}")))?;
                assert!(grad.abs()?.max_all()?.to_scalar::<f32>()? > 0.0);
                var.set(&var.as_tensor().sub(&grad.affine(0.1, 0.0)?)?)?;
            }
            // W_gate starts zero: the first capture gradient updates it;
            // after that update the same objective also reaches the trunk.
            let second = capture_loss()?.backward()?;
            for name in [
                "embedding.weight",
                "layers.00.rec.in.weight",
                "layers.01.read_norm.weight",
            ] {
                let grad = second
                    .get(&model.variables()[name])
                    .ok_or_else(|| invalid(format!("capture trunk misses {name}")))?;
                let size = grad.abs()?.max_all()?.to_scalar::<f32>()?;
                assert!(size.is_finite() && size > 0.0, "{mode:?} {name}: {size}");
            }
            let before = bits(&model.forward(&ids, 1, 8)?)?;
            let logits = model.read_identity_latch_gate_logits(&ids, 1, 8, 1)?;
            let p = model.params()?;
            let x = model.run_layers(model.embed(&ids, 1, 8)?, 0..1)?;
            let u = model.norm(&p, &x, &layer_name(1, "read_norm.weight"))?;
            let previous = Tensor::cat(
                &[
                    &Tensor::zeros((1, 1, 16), DType::F32, &cpu())?,
                    &u.narrow(1, 0, 7)?,
                ],
                1,
            )?;
            let expected = StackModel::linear(
                &Tensor::cat(&[&u, &previous], 2)?,
                p.layer(1, "read.identity_gate.weight")?,
            )?
            .broadcast_add(p.layer(1, "read.identity_gate.bias")?)?
            .squeeze(2)?;
            assert_eq!(bits(&logits)?, bits(&expected)?);
            let actual_gates = model
                .read_latch_inputs(&p, 1, &u, mode, LatchGates::Soft)?
                .1
                .squeeze(2)?;
            assert_eq!(
                bits(&candle_nn::ops::sigmoid(&logits)?)?,
                bits(&actual_gates)?
            );
            assert_eq!(
                bits(&model.read_identity_latch_gates(&ids, 1, 8, 1)?)?,
                bits(&actual_gates)?
            );
            assert_eq!(bits(&model.forward(&ids, 1, 8)?)?, before);
            let mut future = ids;
            future[6] = 19;
            let changed = model.read_identity_latch_gate_logits(&future, 1, 8, 1)?;
            assert_eq!(
                max_abs_gap(&logits.narrow(1, 0, 6)?, &changed.narrow(1, 0, 6)?)?,
                0.0
            );
            // Stable logit CE retains corrective gradients even when the
            // actual sigmoid has rounded all the way to 0 or 1.
            let bias = &model.variables()["layers.01.read.identity_gate.bias"];
            for (value, expected_gradient) in [(-1000.0f32, -0.5f32), (1000.0, 0.5)] {
                bias.set(&Tensor::from_vec(vec![value], 1, &cpu())?)?;
                let loss = capture_loss()?;
                assert!(loss.to_scalar::<f32>()?.is_finite());
                let grads = loss.backward()?;
                let gradient = grads
                    .get(bias)
                    .ok_or_else(|| invalid("missing saturated capture gradient"))?
                    .sum_all()?
                    .to_scalar::<f32>()?;
                assert!(
                    (gradient - expected_gradient).abs() < 1e-6,
                    "{value}: {gradient}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn read_identity_latch_save_reload_restores_mode_parameters_and_rejects_bad_metadata(
    ) -> Result<()> {
        let root = std::env::temp_dir().join(format!("stack-read-latch-{}", std::process::id()));
        let config = tiny(StackArch::Geometric, "ra", ReadScore::Lorentz, true);
        let plain = StackModel::new(config.clone(), &cpu())?;
        let ids = [1, 2, 3, 4, 5, 6];
        for mode in [ReadIdentityLatch::Held, ReadIdentityLatch::Local] {
            let dir = root.join(format!("{mode:?}"));
            let mut model = StackModel::new(config.clone(), &cpu())?;
            model.set_read_identity_latch(mode)?;
            model.set_transport_snap(Some(TransportSnap::Icosian))?;
            let expected = bits(&model.forward(&ids, 1, ids.len())?)?;
            model.save(&dir)?;
            let config_bytes = fs::read(dir.join("config.json"))?;
            let record = fs::read(dir.join(READ_IDENTITY_LATCH_RECORD))?;
            let loaded = StackModel::load(&dir, &cpu())?;
            assert_eq!(loaded.read_identity_latch(), Some(mode));
            assert_eq!(loaded.parameter_count(), model.parameter_count());
            assert_eq!(loaded.transport_snap(), Some(TransportSnap::Icosian));
            assert_eq!(bits(&loaded.forward(&ids, 1, ids.len())?)?, expected);
            for (name, var) in model.variables() {
                assert_eq!(
                    bits(var.as_tensor())?,
                    bits(loaded.variables()[name].as_tensor())?,
                    "{name}"
                );
            }
            fs::remove_file(dir.join(READ_IDENTITY_LATCH_RECORD))?;
            assert!(StackModel::load(&dir, &cpu()).is_err());
            for (field, value) in [
                ("schema", "unknown/2"),
                ("operation", "future"),
                ("mode", "unknown"),
                ("config_sha256", "00"),
                ("model_sha256", "00"),
            ] {
                let mut altered: serde_json::Value = serde_json::from_slice(&record)?;
                altered[field] = serde_json::json!(value);
                let bytes = serde_json::to_vec_pretty(&altered)?;
                let mut marker: serde_json::Value = serde_json::from_slice(&config_bytes)?;
                marker["read_identity_latch"]["record_sha256"] =
                    serde_json::json!(hex::encode(Sha256::digest(&bytes)));
                fs::write(dir.join("config.json"), serde_json::to_vec_pretty(&marker)?)?;
                fs::write(dir.join(READ_IDENTITY_LATCH_RECORD), bytes)?;
                assert!(StackModel::load(&dir, &cpu()).is_err(), "{field}");
            }
            // A default save removes stale metadata and remains legacy-shaped.
            plain.save(&dir)?;
            assert!(!dir.join(READ_IDENTITY_LATCH_RECORD).exists());
            assert_eq!(
                fs::read(dir.join("config.json"))?,
                serde_json::to_vec_pretty(&config)?
            );
            assert_eq!(StackModel::load(&dir, &cpu())?.read_identity_latch(), None);
            fs::write(dir.join(READ_IDENTITY_LATCH_RECORD), &record)?;
            assert!(StackModel::load(&dir, &cpu()).is_err());
            // Valid metadata does not waive exact parameter-shape admission.
            model.variables.remove("layers.01.read.key_identity.weight");
            model.save(&dir)?;
            assert!(StackModel::load(&dir, &cpu()).is_err());
        }
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn read_identity_latch_modes_are_matched_and_refuse_carry_served_and_mode_changes() -> Result<()>
    {
        let mut config = tiny(StackArch::Geometric, "ra", ReadScore::Dot, true);
        config.width = 32;
        config.mlp_hidden = 64;
        let mut held = StackModel::new(config.clone(), &cpu())?;
        let mut local = StackModel::new(config.clone(), &cpu())?;
        held.set_read_identity_latch(ReadIdentityLatch::Held)?;
        local.set_read_identity_latch(ReadIdentityLatch::Local)?;
        assert_eq!(held.parameter_count(), local.parameter_count());
        for (name, var) in held.variables() {
            assert_eq!(
                bits(var.as_tensor())?,
                bits(local.variables()[name].as_tensor())?,
                "{name}"
            );
        }
        held.set_read_identity_latch(ReadIdentityLatch::Held)?;
        assert!(held
            .set_read_identity_latch(ReadIdentityLatch::Local)
            .is_err());
        assert!(held.set_read_identity_carry(true).is_err());
        assert!(held
            .set_served_representation(Some(Arc::new(D11Interim)))
            .is_err());
        assert_eq!(held.read_identity_latch(), Some(ReadIdentityLatch::Held));
        assert!(!held.read_identity_carry() && held.served_codec().is_none());
        let mut carry = StackModel::new(config.clone(), &cpu())?;
        carry.set_read_identity_carry(true)?;
        assert!(carry
            .set_read_identity_latch(ReadIdentityLatch::Held)
            .is_err());
        assert_eq!(carry.read_identity_latch(), None);
        let mut served = StackModel::new(config, &cpu())?;
        served.set_served_representation(Some(Arc::new(D11Interim)))?;
        assert!(served
            .set_read_identity_latch(ReadIdentityLatch::Held)
            .is_err());
        assert_eq!(served.read_identity_latch(), None);
        Ok(())
    }

    #[test]
    fn read_identity_carry_matches_predecessor_qk_current_values_and_binding_gradient() -> Result<()>
    {
        let ids: Vec<u32> = (0..16).map(|i| (i * 5 + 1) % 37).collect();
        for score in [ReadScore::Dot, ReadScore::Lorentz] {
            let mut model = StackModel::new(tiny(StackArch::Geometric, "ra", score, true), &cpu())?;
            let original = bits(&model.forward(&ids, 2, 8)?)?;
            assert!(!model.read_identity_carry());
            model.set_read_identity_carry(false)?;
            assert_eq!(bits(&model.forward(&ids, 2, 8)?)?, original);
            model.set_read_identity_carry(true)?;
            let p = model.params()?;
            let x = model.run_layers(model.embed(&ids, 2, 8)?, 0..1)?;
            let u = model.norm(&p, &x, &layer_name(1, "read_norm.weight"))?;
            let shifted = model.read_identity_input(&u)?;
            assert_eq!(
                shifted
                    .narrow(1, 0, 1)?
                    .abs()?
                    .max_all()?
                    .to_scalar::<f32>()?,
                0.0
            );
            assert_eq!(
                max_abs_gap(&shifted.narrow(1, 1, 7)?, &u.narrow(1, 0, 7)?)?,
                0.0
            );
            assert_eq!(
                model
                    .read_identity_input(&u.narrow(1, 0, 1)?)?
                    .abs()?
                    .max_all()?
                    .to_scalar::<f32>()?,
                0.0
            );
            let project = |input: &Tensor, part: &str| -> Result<Tensor> {
                model.heads(
                    &StackModel::linear(input, p.layer(1, &format!("read.{part}.weight"))?)?,
                    2,
                    8,
                )
            };
            // Independent assembled reference: delayed q/k, current values,
            // and current NoRead/age, through the unchanged fused scorer.
            let query = project(&shifted, "query")?;
            let key = project(&shifted, "key")?;
            let value = project(&u, "value")?;
            let null = StackModel::linear(&u, p.layer(1, "read.null.weight")?)?
                .broadcast_add(p.layer(1, "read.null.bias")?)?
                .transpose(1, 2)?
                .flatten_all()?;
            let age = p.layer(1, "read.age")?.narrow(1, 0, 8)?.flatten_all()?;
            let mut aux = vec![null, age];
            if score == ReadScore::Lorentz {
                aux.push(p.layer(1, "read.log_beta")?.exp()?);
                aux.push(p.layer(1, "read.offset")?.clone());
            }
            let reference = fused_read_selected(
                &query,
                &key,
                &value,
                &Tensor::cat(&aux, 0)?,
                score,
                true,
                true,
                false,
                None,
            )?;
            let reference = StackModel::linear(
                &model.merge_heads(&reference, 2, 8)?,
                p.layer(1, "read.out.weight")?,
            )?;
            let actual = model.geometric_read(&p, 1, &x, &mut None, &mut None, LatchGates::Soft)?;
            assert_eq!(max_abs_gap(&actual, &reference)?, 0.0);
            let target = binding_rows(1);
            let (_, observed) = model.hidden_with_binding(&p, &ids, 2, 8, &target)?;
            assert_eq!(
                bits(&observed)?,
                bits(&model.read_binding_masses(&ids, 2, 8, &target)?)?
            );
            let with_labels = model.hidden_with_binding(&p, &ids, 2, 8, &target)?.0;
            assert_eq!(max_abs_gap(&with_labels, &model.hidden(&ids, 2, 8)?)?, 0.0);
            let names = [
                "layers.01.read.query.weight",
                "layers.01.read.key.weight",
                "layers.00.rec.in.weight",
                "layers.01.read.age",
                "layers.01.read.null.bias",
            ];
            let vars: Vec<Var> = names
                .iter()
                .map(|name| model.variables()[*name].clone())
                .collect();
            let objective = || -> Result<Tensor> {
                Ok(model
                    .loss_with_binding(&ids, &ids, None, 2, 8, &target)?
                    .binding)
            };
            let grads = objective()?.backward()?;
            for (name, var) in names.iter().zip(&vars) {
                let grad = grads
                    .get(var)
                    .ok_or_else(|| invalid(format!("carry binding misses {name}")))?;
                assert!(
                    grad.abs()?.max_all()?.to_scalar::<f32>()? > 0.0,
                    "{score:?} {name}"
                );
            }
            check_gradient(&vars, objective, 2e-2)?;
            model.set_read_identity_carry(false)?;
            assert_eq!(bits(&model.forward(&ids, 2, 8)?)?, original);
        }
        Ok(())
    }

    #[test]
    fn read_identity_carry_save_reload_is_bound_and_default_format_stays_legacy() -> Result<()> {
        let root =
            std::env::temp_dir().join(format!("stack-read-identity-carry-{}", std::process::id()));
        let free_dir = root.join("free");
        let carry_dir = root.join("carry");
        let mut model = StackModel::new(
            tiny(StackArch::Geometric, "ra", ReadScore::Lorentz, true),
            &cpu(),
        )?;
        let ids = [1, 2, 3, 4, 5, 6];
        model.save(&free_dir)?;
        assert_eq!(
            fs::read(free_dir.join("config.json"))?,
            serde_json::to_vec_pretty(&model.config)?
        );
        assert!(!free_dir.join(READ_IDENTITY_CARRY_RECORD).exists());
        assert!(!StackModel::load(&free_dir, &cpu())?.read_identity_carry());
        model.set_transport_snap(Some(TransportSnap::Icosian))?;
        model.set_read_identity_carry(true)?;
        let expected = bits(&model.forward(&ids, 1, ids.len())?)?;
        model.save(&carry_dir)?;
        let config = fs::read(carry_dir.join("config.json"))?;
        let record = fs::read(carry_dir.join(READ_IDENTITY_CARRY_RECORD))?;
        assert!(StackModel::saved_read_identity_carry(&carry_dir)?);
        let mut loaded = StackModel::load(&carry_dir, &cpu())?;
        assert!(loaded.read_identity_carry());
        assert_eq!(loaded.transport_snap(), Some(TransportSnap::Icosian));
        assert_eq!(bits(&loaded.forward(&ids, 1, ids.len())?)?, expected);
        loaded.save(&carry_dir)?;
        assert!(StackModel::load(&carry_dir, &cpu())?.read_identity_carry());
        // Missing or undeclared sidecars cannot silently change behavior.
        fs::remove_file(carry_dir.join(READ_IDENTITY_CARRY_RECORD))?;
        assert!(StackModel::load(&carry_dir, &cpu()).is_err());
        fs::write(carry_dir.join(READ_IDENTITY_CARRY_RECORD), &record)?;
        fs::write(free_dir.join(READ_IDENTITY_CARRY_RECORD), &record)?;
        assert!(StackModel::load(&free_dir, &cpu()).is_err());
        // Even resealing the sidecar digest cannot admit another operation,
        // schema, config binding or weight binding.
        for (field, bad) in [
            ("operation", "future-state"),
            ("schema", "unknown/2"),
            ("config_sha256", "00"),
            ("model_sha256", "00"),
        ] {
            let mut altered: serde_json::Value = serde_json::from_slice(&record)?;
            altered[field] = serde_json::json!(bad);
            let bytes = serde_json::to_vec_pretty(&altered)?;
            let mut marker: serde_json::Value = serde_json::from_slice(&config)?;
            marker["read_identity_carry"]["record_sha256"] =
                serde_json::json!(hex::encode(Sha256::digest(&bytes)));
            fs::write(
                carry_dir.join("config.json"),
                serde_json::to_vec_pretty(&marker)?,
            )?;
            fs::write(carry_dir.join(READ_IDENTITY_CARRY_RECORD), bytes)?;
            assert!(StackModel::load(&carry_dir, &cpu()).is_err(), "{field}");
        }
        fs::write(carry_dir.join("config.json"), &config)?;
        fs::write(carry_dir.join(READ_IDENTITY_CARRY_RECORD), b"broken")?;
        assert!(StackModel::load(&carry_dir, &cpu()).is_err());
        loaded.set_read_identity_carry(false)?;
        loaded.save(&carry_dir)?;
        assert!(!carry_dir.join(READ_IDENTITY_CARRY_RECORD).exists());
        assert!(!StackModel::load(&carry_dir, &cpu())?.read_identity_carry());
        assert_eq!(
            fs::read(carry_dir.join("config.json"))?,
            serde_json::to_vec_pretty(&loaded.config)?
        );
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn read_identity_carry_refuses_served_mode_in_both_orders() -> Result<()> {
        let mut config = tiny(StackArch::Geometric, "ra", ReadScore::Dot, true);
        config.width = 32;
        config.mlp_hidden = 64;
        let mut model = StackModel::new(config, &cpu())?;
        model.set_read_identity_carry(true)?;
        assert!(model
            .set_served_representation(Some(Arc::new(D11Interim)))
            .is_err());
        assert!(model.read_identity_carry() && model.served_codec().is_none());
        model.set_read_identity_carry(false)?;
        model.set_served_representation(Some(Arc::new(D11Interim)))?;
        assert!(model.set_read_identity_carry(true).is_err());
        assert!(!model.read_identity_carry() && model.served_codec().is_some());
        for (arch, pattern) in [(StackArch::Geometric, "rr"), (StackArch::Transformer, "aa")] {
            let mut unsupported =
                StackModel::new(tiny(arch, pattern, ReadScore::Dot, false), &cpu())?;
            assert!(unsupported.set_read_identity_carry(true).is_err());
            assert!(!unsupported.read_identity_carry());
        }
        Ok(())
    }

    #[test]
    fn read_binding_labels_leave_language_and_its_gradient_unchanged() -> Result<()> {
        let ids: Vec<u32> = (0..16).map(|i| (i * 7 + 3) % 37).collect();
        let targets: Vec<u32> = ids.iter().map(|&id| (id + 1) % 37).collect();
        let weights: Vec<f32> = (0..16)
            .map(|i| if i % 3 == 0 { 0.0 } else { 1.0 })
            .collect();
        for read in [ReadScore::Dot, ReadScore::Lorentz] {
            for pointer in [false, true] {
                let mut config = tiny(StackArch::Geometric, "rra", read, true);
                if pointer {
                    config.pointer = Some(PointerConfig::new(4));
                }
                let mut model = StackModel::new(config, &cpu())?;
                model.set_transport_snap(Some(TransportSnap::Icosian))?;
                let p = model.params()?;
                let ordinary_hidden = model.hidden(&ids, 2, 8)?;
                let labels = binding_rows(2);
                let mut changed = labels.clone();
                changed.rows[0].sources = vec![0, 3];
                changed.rows[1].sources = vec![4];
                for target in [&labels, &changed] {
                    let (hidden, _) = model.hidden_with_binding(&p, &ids, 2, 8, target)?;
                    assert_eq!(max_abs_gap(&hidden, &ordinary_hidden)?, 0.0);
                }
                for row_weights in [None, Some(weights.as_slice())] {
                    let ordinary = match row_weights {
                        None => model.loss(&ids, &targets, 2, 8)?,
                        Some(weights) => model.weighted_loss(&ids, &targets, weights, 2, 8)?,
                    };
                    let joint =
                        model.loss_with_binding(&ids, &targets, row_weights, 2, 8, &labels)?;
                    assert_eq!(
                        ordinary.to_scalar::<f32>()?.to_bits(),
                        joint.language.to_scalar::<f32>()?.to_bits()
                    );
                    let expected = ordinary.backward()?;
                    let got = joint.language.backward()?;
                    for (name, var) in model.variables() {
                        match (expected.get(var), got.get(var)) {
                            (Some(a), Some(b)) => assert!(
                                max_abs_gap(a, b)? < 1e-6,
                                "{read:?} pointer={pointer} {name}"
                            ),
                            (None, None) => (),
                            _ => {
                                return Err(invalid(format!(
                                    "binding changed language gradient reachability: {name}"
                                )))
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn read_binding_gradient_reaches_query_key_age_null_and_recurrent_trunk() -> Result<()> {
        let ids: Vec<u32> = (0..16).map(|i| (i * 5 + 1) % 37).collect();
        for read in [ReadScore::Dot, ReadScore::Lorentz] {
            let model = StackModel::new(tiny(StackArch::Geometric, "rra", read, true), &cpu())?;
            let labels = binding_rows(2);
            let objective = || -> Result<Tensor> {
                Ok(model
                    .loss_with_binding(&ids, &ids, None, 2, 8, &labels)?
                    .binding)
            };
            let loss = objective()?;
            let before = loss.to_scalar::<f32>()?;
            let grads = loss.backward()?;
            let mut names = vec![
                "embedding.weight",
                "layers.01.rec.in.weight",
                "layers.01.rec.gate.weight",
                "layers.02.read.query.weight",
                "layers.02.read.key.weight",
                "layers.02.read.age",
                "layers.02.read.null.weight",
                "layers.02.read.null.bias",
            ];
            if read == ReadScore::Lorentz {
                names.extend(["layers.02.read.log_beta", "layers.02.read.offset"]);
            }
            for name in &names {
                let grad = grads
                    .get(&model.variables()[*name])
                    .ok_or_else(|| invalid(format!("missing binding gradient {name}")))?;
                let size = grad.abs()?.max_all()?.to_scalar::<f32>()?;
                assert!(size.is_finite() && size > 0.0, "{read:?} {name}: {size}");
            }
            // Binding credit cannot train the value or output projection by
            // smuggling its teacher mask into the residual/language path.
            for name in ["layers.02.read.value.weight", "layers.02.read.out.weight"] {
                if let Some(grad) = grads.get(&model.variables()[name]) {
                    assert_eq!(grad.abs()?.max_all()?.to_scalar::<f32>()?, 0.0, "{name}");
                }
            }
            let vars: Vec<Var> = names
                .iter()
                .map(|name| model.variables()[*name].clone())
                .collect();
            check_gradient(&vars, objective, 2e-2)?;
            for var in model.variables().values() {
                if let Some(grad) = grads.get(var) {
                    var.set(&var.as_tensor().sub(&grad.affine(0.05, 0.0)?)?)?;
                }
            }
            let after = objective()?.to_scalar::<f32>()?;
            assert!(after < before, "{read:?}: binding step {before} -> {after}");
        }
        Ok(())
    }

    #[test]
    fn read_binding_tracks_exact_duplicate_occurrences_and_no_read_mass() -> Result<()> {
        let model = StackModel::new(
            tiny(StackArch::Geometric, "a", ReadScore::Dot, false),
            &cpu(),
        )?;
        for name in [
            "layers.00.read.query.weight",
            "layers.00.read.key.weight",
            "layers.00.read.null.weight",
        ] {
            let var = &model.variables()[name];
            var.set(&Tensor::zeros(var.shape(), DType::F32, &cpu())?)?;
        }
        let mut ages = vec![0.0f32; model.config.heads * model.config.context];
        ages[1] = 2.0;
        ages[3] = -2.0;
        model.variables()["layers.00.read.age"].set(&Tensor::from_vec(ages, (2, 12), &cpu())?)?;
        let ids = [3, 7, 9, 7, 5, 6]; // The two 7s are distinct source occurrences.
        let target = |sources| ReadBindingTarget {
            layer: 0,
            head: 0,
            rows: vec![ReadBinding {
                batch: 0,
                query: 4,
                sources,
            }],
        };
        let mass = |sources| -> Result<f32> {
            Ok(model
                .read_binding_masses(&ids, 1, 6, &target(sources))?
                .to_vec1::<f32>()?[0])
        };
        let old = mass(vec![1])?;
        let recent = mass(vec![3])?;
        let both = mass(vec![1, 3])?;
        assert!((f64::from(recent / old) - 4.0f64.exp()).abs() < 1e-4);
        assert!((both - old - recent).abs() < 1e-6);
        // Five causal positions plus NoRead. The future position is absent.
        let denominator = 4.0 + 2.0f64.exp() + (-2.0f64).exp();
        assert!((f64::from(old) - (-2.0f64).exp() / denominator).abs() < 1e-7);
        model.variables()["layers.00.read.null.bias"].set(&Tensor::from_vec(
            vec![3.0f32, 0.0],
            2,
            &cpu(),
        )?)?;
        assert!(mass(vec![1])? < old);
        let got = model.loss_with_binding(&ids, &ids, None, 1, 6, &target(vec![1]))?;
        assert!(
            (got.binding.to_scalar::<f32>()? + got.masses.to_vec1::<f32>()?[0].ln()).abs() < 1e-6
        );
        Ok(())
    }

    #[test]
    fn read_binding_rejects_noncausal_ambiguous_or_zero_mass_labels() -> Result<()> {
        let mut model = StackModel::new(
            tiny(StackArch::Geometric, "ra", ReadScore::Dot, false),
            &cpu(),
        )?;
        let ids = [1, 2, 1, 4];
        let target = ReadBindingTarget {
            layer: 1,
            head: 0,
            rows: vec![ReadBinding {
                batch: 0,
                query: 3,
                sources: vec![0],
            }],
        };
        let mut invalid_labels = Vec::new();
        for sources in [vec![], vec![0, 0], vec![3], vec![4]] {
            let mut bad = target.clone();
            bad.rows[0].sources = sources;
            invalid_labels.push(bad);
        }
        let mut bad = target.clone();
        bad.rows.push(bad.rows[0].clone());
        invalid_labels.push(bad);
        let mut bad = target.clone();
        bad.rows[0].batch = 1;
        invalid_labels.push(bad);
        let mut bad = target.clone();
        bad.rows[0].query = 4;
        invalid_labels.push(bad);
        let mut bad = target.clone();
        bad.head = 2;
        invalid_labels.push(bad);
        let mut bad = target.clone();
        bad.layer = 0;
        invalid_labels.push(bad);
        let mut bad = target.clone();
        bad.layer = 2;
        invalid_labels.push(bad);
        let mut bad = target.clone();
        bad.rows.clear();
        invalid_labels.push(bad);
        for bad in invalid_labels {
            assert!(
                model
                    .loss_with_binding(&ids, &ids, None, 1, 4, &bad)
                    .is_err(),
                "{bad:?}"
            );
        }
        model.config.select = Some(FlockSelect {
            sink: 0,
            window: 1,
            k: 1,
        });
        assert!(model.read_binding_masses(&ids, 1, 4, &target).is_err());
        model.config.select = None;
        let mut ages = vec![0.0f32; 24];
        ages[3] = -1000.0;
        model.variables()["layers.01.read.age"].set(&Tensor::from_vec(ages, (2, 12), &cpu())?)?;
        assert_eq!(
            model
                .read_binding_masses(&ids, 1, 4, &target)?
                .to_vec1::<f32>()?,
            vec![0.0]
        );
        assert!(model
            .loss_with_binding(&ids, &ids, None, 1, 4, &target)
            .is_err());
        Ok(())
    }

    #[test]
    fn extending_the_context_keeps_every_learned_age_and_reads_further() -> Result<()> {
        let ids: Vec<u32> = (0..20).map(|i| (i * 7 + 3) % 37).collect();
        for (arch, pattern, read) in [
            (StackArch::Geometric, "ar", ReadScore::Lorentz),
            (StackArch::Geometric, "ra", ReadScore::Dot),
            (StackArch::Transformer, "aa", ReadScore::Dot),
        ] {
            let rotation = arch == StackArch::Geometric;
            let mut model = StackModel::new(tiny(arch, pattern, read, rotation), &cpu())?;
            let before = model
                .forward(&ids[..12], 1, 12)?
                .flatten_all()?
                .to_vec1::<f32>()?;
            let ages_before: BTreeMap<String, Vec<f32>> = model
                .variables
                .iter()
                .filter(|(name, _)| name.ends_with(".read.age"))
                .map(|(name, var)| Ok((name.clone(), var.as_tensor().flatten_all()?.to_vec1()?)))
                .collect::<Result<_>>()?;
            assert_eq!(ages_before.is_empty(), arch == StackArch::Transformer);
            assert!(model.forward(&ids, 1, 20).is_err());
            assert!(
                model.extend_context(8).is_err(),
                "a smaller context is refused"
            );
            assert!(!model.extend_context(12)?);
            assert!(model.extend_context(20)?);
            assert_eq!(model.config.context, 20);
            // Within the saved context nothing changes, bit for bit.
            let after = model
                .forward(&ids[..12], 1, 12)?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert_eq!(
                before.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                after.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
            );
            assert!(model.forward(&ids, 1, 20).is_ok());
            // Learned ages are kept; each head continues along its initial slope.
            for (name, old) in &ages_before {
                let new = model.variables[name]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for head in 0..2 {
                    assert_eq!(
                        new[head * 20..head * 20 + 12],
                        old[head * 12..head * 12 + 12]
                    );
                    let slope = 2f64.powf(-8.0 * (head + 1) as f64 / 2.0);
                    for age in 12..20 {
                        let expected = f64::from(old[head * 12 + 11]) - slope * (age - 11) as f64;
                        assert_eq!(new[head * 20 + age], expected as f32);
                    }
                }
            }
            // The extended model saves and loads at its new context.
            let directory = std::env::temp_dir().join(format!(
                "geometric-stack-extend-{}-{}",
                std::process::id(),
                arch == StackArch::Transformer
            ));
            model.save(&directory)?;
            let loaded = StackModel::load(&directory, &cpu())?;
            fs::remove_dir_all(&directory)?;
            assert_eq!(loaded.config, model.config);
            assert_eq!(
                loaded.next_scores(&ids)?,
                model.next_scores(&ids)?,
                "the saved model reads all 20 positions"
            );
        }
        Ok(())
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
            rotation_group: Default::default(),
            seed: 29,
            memory: None,
            select: None,
            pointer: None,
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
        let (bytes, _) = crate::stack_export::export_stack(
            model,
            serde_json::json!({"test": "qat"}),
            None,
            None,
        )?;
        let artifact = uor_r4_lut::format::StackArtifact::parse(bytes)
            .map_err(|error| invalid(error.to_string()))?;
        crate::stack_export::stack_grid_reference(model, &artifact)
    }

    #[test]
    fn served_forward_equals_the_exported_reference() -> Result<()> {
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
                    // Bit for bit: both forwards read the export's values
                    // through the same kernels, so a slip in any rounding (a
                    // fixed-point exponent of 2^-16 for 2^-24, say) shows.
                    assert_eq!(
                        bits(&served)?,
                        bits(&want)?,
                        "{label}: served logits differ from the exported reference by {gap}"
                    );
                    assert!(
                        max_abs_gap(&served, &float)? > 1e-3,
                        "{label}: the served representation left the logits unchanged"
                    );
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
        Ok(())
    }

    /// The float forward composed directly from the variables with the
    /// stack's kernels, in the order the forward pass took before the served
    /// representation existed, and with the recurrence core as it was before
    /// the transport snap existed ([`LegacyRecurrenceCore`]).
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
                    LegacyRecurrenceCore {
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
            let (served, want) = (
                model.forward(&ids, 2, time)?,
                updated.logits(&ids, 2, time)?,
            );
            let gap = max_abs_gap(&served, &want)?;
            assert_eq!(
                bits(&served)?,
                bits(&want)?,
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
            let want = exported_reference(&model)?.logits(&ids, 1, time)?;
            let gap = max_abs_gap(&logits, &want)?;
            assert_eq!(
                bits(&logits)?,
                bits(&want)?,
                "after changing {name} the served forward is {gap} off"
            );
        }
        Ok(())
    }

    #[test]
    fn a_model_saved_in_served_mode_records_its_representation() -> Result<()> {
        let mut model = StackModel::new(exportable("rar", ReadScore::Lorentz, true), &cpu())?;
        spread(&model, 97)?;
        let root = std::env::temp_dir().join(format!(
            "geometric-stack-served-save-{}",
            std::process::id()
        ));
        let (float_dir, served_dir) = (root.join("float"), root.join("served"));
        model.save(&float_dir)?;
        model.set_served_representation(Some(Arc::new(D11Interim)))?;
        model.save(&served_dir)?;
        let time = model.config.context;
        let ids = token_ids(time, 96, 31);
        let float_logits = bits(&model.with_float_forward(|m| m.forward(&ids, 1, time))?)?;
        // A float save writes the configuration alone, as before the record.
        assert_eq!(
            fs::read(float_dir.join("config.json"))?,
            serde_json::to_vec_pretty(&model.config)?
        );
        assert_eq!(StackModel::saved_served_representation(&float_dir)?, None);
        assert_eq!(
            StackModel::saved_served_representation(&served_dir)?,
            Some(SavedServedRepresentation {
                codec: D11Interim.name().to_owned()
            })
        );
        // Both directories load, in float, with the saved weights.
        for directory in [&float_dir, &served_dir] {
            let loaded = StackModel::load(directory, &cpu())?;
            assert!(loaded.served_codec().is_none());
            assert_eq!(loaded.config, model.config);
            assert_eq!(bits(&loaded.forward(&ids, 1, time)?)?, float_logits);
        }
        // A float save over a served one leaves no record.
        StackModel::load(&served_dir, &cpu())?.save(&served_dir)?;
        assert_eq!(StackModel::saved_served_representation(&served_dir)?, None);
        fs::remove_dir_all(&root)?;
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

    // -----------------------------------------------------------------------
    // The trained-in transport snap.

    // The recurrence core exactly as it was before the transport snap existed,
    // copied verbatim from the parent of this change and renamed: the frozen
    // reference that the core without a snap must equal bit for bit
    // (`float_reference_logits` composes it, and
    // `without_a_snap_the_core_is_bit_identical_to_the_pre_snap_core` compares
    // the ops directly).
    #[derive(Clone, Copy, Debug)]
    struct LegacyRecurrenceCore {
        batch: usize,
        time: usize,
        width: usize,
        rotation: bool,
    }

    /// One lane's transition at one position.
    #[derive(Clone, Copy, Default)]
    struct LegacyTransition {
        /// Decay gate sigma(logit).
        opening: f32,
        lambda: f32,
        keep: f32,
        /// `1 - lambda^2` fell below the floor, so `keep` carries no gradient.
        clamped: bool,
        rotation: [f32; 4],
        norm: f32,
    }

    impl LegacyRecurrenceCore {
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
        ) -> (Vec<f32>, Vec<f32>, Vec<LegacyTransition>) {
            let (time, width, lanes, gate_width) =
                (self.time, self.width, self.lanes(), self.gate_width());
            let taps = &parameters[..CONVOLUTION_WIDTH * width];
            let bias = &parameters[CONVOLUTION_WIDTH * width..(CONVOLUTION_WIDTH + 1) * width];
            let mut drive = vec![0f32; time * width];
            let mut state = vec![0f32; time * width];
            let mut transitions = vec![LegacyTransition::default(); time * lanes];
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
                    transitions[t * lanes + lane] = LegacyTransition {
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

    impl CustomOp3 for LegacyRecurrenceCore {
        fn name(&self) -> &'static str {
            "legacy-geometric-stack-recurrence"
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
                    let gates = &gates[window * time * self.gate_width()
                        ..(window + 1) * time * self.gate_width()];
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
                            d_log_a[lane] +=
                                f64::from(d_log_lambda * exponent * transition.opening);
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

    /// The 120 unit icosians in f32, built as the snap-evaluate diagnostic
    /// builds them.
    fn icosians() -> Vec<[f32; 4]> {
        uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots()
            .iter()
            .map(|r| {
                let a = r.to_array();
                [a[0] as f32, a[1] as f32, a[2] as f32, a[3] as f32]
            })
            .collect()
    }

    /// `config`'s stack with its recurrence gate weights drawn at scale 0.5,
    /// far above initialization, so the unit transports spread over S^3 and
    /// snap to many roots.
    fn spread_transport(config: StackConfig, seed: u64) -> Result<StackModel> {
        let model = StackModel::new(config, &cpu())?;
        let mut rng = Initializer(seed);
        for (name, var) in model.variables() {
            if name.ends_with("rec.gate.weight") {
                var.set(&random(&mut rng, var.dims(), 0.5))?;
            }
        }
        Ok(model)
    }

    /// The largest gap of any variable's gradient between two backward
    /// passes, relative to the largest magnitude of that gradient in `want`.
    fn worst_relative_gradient_gap(
        model: &StackModel,
        got: &candle_core::backprop::GradStore,
        want: &candle_core::backprop::GradStore,
        label: &str,
    ) -> Result<f32> {
        let mut worst = 0f32;
        for (name, var) in model.variables() {
            let (a, b) = (
                got.get(var.as_tensor())
                    .ok_or_else(|| invalid(format!("{label}: no gradient for {name}")))?,
                want.get(var.as_tensor())
                    .ok_or_else(|| invalid(format!("{label}: no reference gradient for {name}")))?,
            );
            let size = b.abs()?.max_all()?.to_scalar::<f32>()?;
            assert!(size > 0.0, "{label}: {name} has a zero reference gradient");
            worst = worst.max(max_abs_gap(a, b)? / size);
        }
        Ok(worst)
    }

    #[test]
    fn the_icosian_snap_holds_the_120_unit_icosians() -> Result<()> {
        let roots = TransportSnap::Icosian.roots();
        assert_eq!(roots, icosians().as_slice());
        assert_eq!(roots.len(), 120);
        for root in roots {
            let norm: f64 = root.iter().map(|&v| f64::from(v) * f64::from(v)).sum();
            assert!((norm - 1.0).abs() < 1e-6, "{root:?} is not a unit");
        }
        // Closed under the quaternion product (2I is a group), up to f32.
        for a in roots.iter().step_by(7) {
            for b in roots.iter().step_by(5) {
                let product = quaternion_product(*a, *b);
                let nearest = roots[TransportSnap::Icosian.nearest(product)];
                let gap = (0..4)
                    .map(|k| (product[k] - nearest[k]).abs())
                    .fold(0f32, f32::max);
                assert!(gap < 1e-6, "{a:?} {b:?}: product off the group by {gap}");
            }
        }
        // Each root is its own nearest root, and a tie goes to the first.
        for (index, root) in roots.iter().enumerate() {
            assert_eq!(TransportSnap::Icosian.nearest(*root), index);
        }
        let tie = [[0.0f32, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]];
        let halfway = [std::f32::consts::FRAC_1_SQRT_2; 2];
        assert_eq!(nearest_root([halfway[0], halfway[1], 0.0, 0.0], &tie), 0);
        assert_eq!(nearest_root([f32::NAN; 4], &tie), 0);
        Ok(())
    }

    #[test]
    fn a_snapped_fused_forward_equals_the_composed_snapped_reference() -> Result<()> {
        let roots = icosians();
        let identity = TransportSnap::Icosian.nearest([1.0, 0.0, 0.0, 0.0]);
        let mut worst = 0f32;
        for pattern in ["rar", "rrarra"] {
            for read in [ReadScore::Lorentz, ReadScore::Dot] {
                let label = format!("{pattern} {read:?}");
                let mut config = tiny(StackArch::Geometric, pattern, read, true);
                config.seed = 59;
                let mut model = spread_transport(config, 61)?;
                let (batch, time) = (2, 12);
                let ids = token_ids(batch * time, 37, 67);
                let free = model.forward(&ids, batch, time)?;
                model.set_transport_snap(Some(TransportSnap::Icosian))?;
                let fused = model.forward(&ids, batch, time)?;
                let composed = model.logits_with_transport(&ids, batch, time, Some(&roots))?;
                let gap = max_abs_gap(&fused, &composed)?;
                assert!(
                    gap < 1e-4,
                    "{label}: the snapped fused forward differs from the composed reference by {gap}"
                );
                worst = worst.max(gap);
                // The snap is not trivial (Lab 1 adjudication, #1483, 2026-09-29):
                // 1. some lane's chosen root differs from its unit quaternion
                //    by more than 1e-3 in L2, witnessed on layer 0's raw
                //    quaternions as the fused core computes them;
                // 2. the snap moves the logits at least 100x the parity floor
                //    (its own fused-vs-composed gap), so the effect is a
                //    mechanism and not numerical noise.
                let p = model.params()?;
                let x = model.embed_with(&p, &ids, batch, time)?;
                let u = model.norm(&p, &x, &layer_name(0, "rec_norm.weight"))?;
                let gates = model.recurrence_gates(&p, 0, &u)?;
                let lanes = model.config.width / 4;
                let raw = gates
                    .narrow(2, lanes, model.config.width)?
                    .reshape((batch, time, lanes, 4))?;
                let norm = raw.sqr()?.sum_keepdim(3)?.affine(1.0, 1e-6)?.sqrt()?;
                let unit = raw.broadcast_div(&norm)?.flatten_all()?.to_vec1::<f32>()?;
                let mut max_displacement = 0f32;
                for quad in unit.chunks_exact(4) {
                    let root = roots[nearest_root([quad[0], quad[1], quad[2], quad[3]], &roots)];
                    let l2: f32 = (0..4)
                        .map(|k| (root[k] - quad[k]) * (root[k] - quad[k]))
                        .sum::<f32>()
                        .sqrt();
                    max_displacement = max_displacement.max(l2);
                }
                assert!(
                    max_displacement > 1e-3,
                    "{label}: the snap left every transport unchanged ({max_displacement})"
                );
                let floor = gap.max(1e-7);
                let moved = max_abs_gap(&fused, &free)?;
                assert!(
                    moved > 100.0 * floor,
                    "{label}: the snap's effect ({moved}) is within noise of the parity floor ({floor})"
                );
                let usage = model.transport_usage(&ids, batch, time, None)?;
                let pooled = usage.pooled();
                let total: u64 = pooled.iter().sum();
                assert_eq!(total, (usage.layers.len() * batch * time * 4) as u64);
                let distinct = pooled.iter().filter(|&&count| count > 0).count();
                assert!(distinct >= 20, "{label}: only {distinct} roots selected");
                assert!(
                    pooled[identity] * 4 < total,
                    "{label}: the identity took {} of {total} selections",
                    pooled[identity]
                );
                // Unsnapped, the forward is the free one again.
                let unsnapped = model.with_unsnapped_transport(|m| m.forward(&ids, batch, time))?;
                assert_eq!(bits(&unsnapped)?, bits(&free)?, "{label}: unsnapped");
                assert_eq!(model.transport_snap(), Some(TransportSnap::Icosian));
            }
        }
        eprintln!(
            "snapped fused forward against the composed snapped reference: worst absolute logit gap {worst}"
        );
        Ok(())
    }

    #[test]
    fn snapped_gradients_equal_the_straight_through_reference() -> Result<()> {
        let roots = icosians();
        let (mut worst, mut worst_free) = (0f32, 0f32);
        for pattern in ["rar", "rrarra"] {
            for read in [ReadScore::Lorentz, ReadScore::Dot] {
                let label = format!("{pattern} {read:?}");
                let mut config = tiny(StackArch::Geometric, pattern, read, true);
                config.seed = 71;
                let mut model = spread_transport(config, 73)?;
                let (batch, time) = (2, 12);
                let ids = token_ids(batch * time, 37, 79);
                // Random logit weights give derivatives well above f32 rounding.
                let weights = random(&mut Initializer(83), &[batch * time, 37], 1.0);
                let objective = |logits: Tensor| -> Result<candle_core::backprop::GradStore> {
                    Ok(logits.mul(&weights)?.sum_all()?.backward()?)
                };
                // The fused core against its composed reference without a snap:
                // the float gap the two paths have anyway.
                let free = objective(model.forward(&ids, batch, time)?)?;
                let free_reference = objective(model.composed_logits(
                    &ids,
                    batch,
                    time,
                    ComposedTransport::Free,
                )?)?;
                worst_free = worst_free.max(worst_relative_gradient_gap(
                    &model,
                    &free,
                    &free_reference,
                    &label,
                )?);
                model.set_transport_snap(Some(TransportSnap::Icosian))?;
                let fused = objective(model.forward(&ids, batch, time)?)?;
                let reference = objective(model.composed_logits(
                    &ids,
                    batch,
                    time,
                    ComposedTransport::StraightThrough(&roots),
                )?)?;
                let gap = worst_relative_gradient_gap(&model, &fused, &reference, &label)?;
                assert!(
                    gap < 1e-4,
                    "{label}: a snapped gradient differs from the straight-through reference by {gap} (relative)"
                );
                worst = worst.max(gap);
                // The straight-through gradient reaches every rotation row,
                // and differs from the free gradient there.
                for (name, var) in model.variables() {
                    if !name.ends_with("rec.gate.weight") {
                        continue;
                    }
                    let rotation_rows =
                        |grads: &candle_core::backprop::GradStore| -> Result<Tensor> {
                            Ok(grads
                                .get(var.as_tensor())
                                .ok_or_else(|| invalid(name.clone()))?
                                .narrow(0, 4, 16)?)
                        };
                    let snapped_rows = rotation_rows(&fused)?;
                    assert!(
                        snapped_rows.abs()?.max_all()?.to_scalar::<f32>()? > 0.0,
                        "{label}: {name}'s rotation rows have no gradient"
                    );
                    assert!(
                        max_abs_gap(&snapped_rows, &rotation_rows(&free)?)? > 0.0,
                        "{label}: {name}'s rotation gradient ignores the snap"
                    );
                }
            }
        }
        eprintln!(
            "snapped gradients against the straight-through reference: worst relative gap {worst}; \
             without a snap the fused and composed paths differ by {worst_free}"
        );
        Ok(())
    }

    #[test]
    fn without_a_snap_the_core_is_bit_identical_to_the_pre_snap_core() -> Result<()> {
        // The op itself, on random inputs: forward and every input gradient.
        for rotation in [true, false] {
            let (batch, time, width) = (3, 11, 16);
            let lanes = width / 4;
            let gate_width = lanes + if rotation { width } else { 0 };
            let mut rng = Initializer(101);
            let branches = Var::from_tensor(&random(&mut rng, &[batch, time, 2 * width], 1.0))?;
            let gates = Var::from_tensor(&random(&mut rng, &[batch, time, gate_width], 1.0))?;
            let parameters = Var::from_tensor(&random(
                &mut rng,
                &[(CONVOLUTION_WIDTH + 1) * width + lanes],
                0.5,
            ))?;
            let weights = random(&mut rng, &[batch, time, width], 1.0);
            let current = branches.as_tensor().apply_op3(
                gates.as_tensor(),
                parameters.as_tensor(),
                RecurrenceCore {
                    batch,
                    time,
                    width,
                    rotation,
                    group: RotationGroup::Quaternion,
                    snap: None,
                },
            )?;
            let legacy = branches.as_tensor().apply_op3(
                gates.as_tensor(),
                parameters.as_tensor(),
                LegacyRecurrenceCore {
                    batch,
                    time,
                    width,
                    rotation,
                },
            )?;
            assert_eq!(
                bits(&current)?,
                bits(&legacy)?,
                "rotation {rotation}: forward"
            );
            let a = current.mul(&weights)?.sum_all()?.backward()?;
            let b = legacy.mul(&weights)?.sum_all()?.backward()?;
            for (name, var) in [
                ("branches", &branches),
                ("gates", &gates),
                ("parameters", &parameters),
            ] {
                let grad = |grads: &candle_core::backprop::GradStore| -> Result<Vec<u32>> {
                    bits(
                        grads
                            .get(var.as_tensor())
                            .ok_or_else(|| invalid(format!("no gradient for {name}")))?,
                    )
                };
                assert_eq!(
                    grad(&a)?,
                    grad(&b)?,
                    "rotation {rotation}: gradient of {name}"
                );
            }
        }
        // The whole model, never snapped, snapped and unsnapped again, or
        // bypassed: logits and every gradient, bit for bit, equal to the
        // forward composed with the pre-snap core.
        for (pattern, read) in [("rar", ReadScore::Lorentz), ("rrarra", ReadScore::Dot)] {
            let mut model = spread_transport(tiny(StackArch::Geometric, pattern, read, true), 107)?;
            let (batch, time) = (2, 12);
            let (ids, targets) = (
                token_ids(batch * time, 37, 109),
                token_ids(batch * time, 37, 113),
            );
            let plain = bits(&model.forward(&ids, batch, time)?)?;
            let legacy = float_reference_logits(&model, &ids, batch, time)?;
            assert_eq!(
                plain,
                bits(&legacy)?,
                "{pattern}: logits against the pre-snap core"
            );
            let plain_grads = model.loss(&ids, &targets, batch, time)?.backward()?;
            let legacy_grads = logits_cross_entropy(&legacy, &targets, None)?.backward()?;
            let vars: Vec<(String, Var)> = model
                .variables()
                .iter()
                .map(|(name, var)| (name.clone(), var.clone()))
                .collect();
            let gradient_bits =
                |grads: &candle_core::backprop::GradStore| -> Result<Vec<Vec<u32>>> {
                    vars.iter()
                        .map(|(name, var)| {
                            bits(
                                grads
                                    .get(var.as_tensor())
                                    .ok_or_else(|| invalid(name.clone()))?,
                            )
                        })
                        .collect()
                };
            let want = gradient_bits(&plain_grads)?;
            assert_eq!(want, gradient_bits(&legacy_grads)?, "{pattern}: gradients");
            model.set_transport_snap(Some(TransportSnap::Icosian))?;
            assert_ne!(plain, bits(&model.forward(&ids, batch, time)?)?);
            let bypassed = model.with_unsnapped_transport(|m| m.forward(&ids, batch, time))?;
            assert_eq!(
                plain,
                bits(&bypassed)?,
                "{pattern}: with_unsnapped_transport"
            );
            model.set_transport_snap(None)?;
            assert_eq!(
                plain,
                bits(&model.forward(&ids, batch, time)?)?,
                "{pattern}: off again"
            );
            let again = model.loss(&ids, &targets, batch, time)?.backward()?;
            assert_eq!(
                want,
                gradient_bits(&again)?,
                "{pattern}: gradients off again"
            );
        }
        Ok(())
    }

    #[test]
    fn the_transport_snap_takes_rotating_geometric_stacks_only() -> Result<()> {
        for config in [
            tiny(StackArch::Transformer, "aa", ReadScore::Dot, false),
            tiny(StackArch::Geometric, "rar", ReadScore::Lorentz, false),
            // Rotating, but with no recurrence to snap.
            tiny(StackArch::Geometric, "aa", ReadScore::Dot, true),
        ] {
            let mut model = StackModel::new(config.clone(), &cpu())?;
            assert!(TransportSnap::Icosian.check(&config).is_err(), "{config:?}");
            assert!(
                model
                    .set_transport_snap(Some(TransportSnap::Icosian))
                    .is_err(),
                "{config:?}"
            );
            assert_eq!(model.transport_snap(), None);
            assert!(model
                .transport_usage(&token_ids(8, 37, 3), 1, 8, None)
                .is_err());
        }
        let mut model = StackModel::new(
            tiny(StackArch::Geometric, "ra", ReadScore::Dot, true),
            &cpu(),
        )?;
        // Usage counts need a snap, and one length of at most `time` per window.
        let ids = token_ids(16, 37, 5);
        assert!(model.transport_usage(&ids, 2, 8, None).is_err());
        model.set_transport_snap(Some(TransportSnap::Icosian))?;
        assert_eq!(model.transport_snap(), Some(TransportSnap::Icosian));
        assert!(model.transport_usage(&ids, 2, 8, Some(&[8])).is_err());
        assert!(model.transport_usage(&ids, 2, 8, Some(&[8, 9])).is_err());
        let usage = model.transport_usage(&ids, 2, 8, Some(&[3, 8]))?;
        assert_eq!(usage.layers.keys().copied().collect::<Vec<_>>(), vec![0]);
        assert_eq!(usage.pooled().iter().sum::<u64>(), (3 + 8) * 4);
        let summary = usage.summary();
        assert_eq!(summary["selections"], serde_json::json!(44));
        assert_eq!(summary["roots"], serde_json::json!(120));
        Ok(())
    }

    #[test]
    fn a_snapped_model_saves_its_transport_record() -> Result<()> {
        let mut model = spread_transport(
            tiny(StackArch::Geometric, "rrarra", ReadScore::Lorentz, true),
            131,
        )?;
        let root = std::env::temp_dir().join(format!(
            "geometric-stack-transport-save-{}",
            std::process::id()
        ));
        let (free_dir, snapped_dir) = (root.join("free"), root.join("snapped"));
        model.save(&free_dir)?;
        model.set_transport_snap(Some(TransportSnap::Icosian))?;
        model.save(&snapped_dir)?;
        // A save without a snap writes the files it wrote before the record.
        assert!(!free_dir.join(TRANSPORT_RECORD).exists());
        assert_eq!(StackModel::saved_transport_snap(&free_dir)?, None);
        assert_eq!(
            StackModel::saved_transport_snap(&snapped_dir)?,
            Some(TransportSnap::Icosian)
        );
        assert_eq!(
            fs::read(free_dir.join("config.json"))?,
            fs::read(snapped_dir.join("config.json"))?
        );
        // Each loads the model that was saved: the free save without a snap,
        // the snapped save with its snap, and the same forward pass.
        let ids = token_ids(12, 37, 137);
        let free = bits(&model.with_unsnapped_transport(|m| m.forward(&ids, 1, 12))?)?;
        let snapped = bits(&model.forward(&ids, 1, 12)?)?;
        let loaded_free = StackModel::load(&free_dir, &cpu())?;
        assert_eq!(loaded_free.transport_snap(), None);
        assert_eq!(bits(&loaded_free.forward(&ids, 1, 12)?)?, free);
        let mut loaded = StackModel::load(&snapped_dir, &cpu())?;
        assert_eq!(loaded.transport_snap(), Some(TransportSnap::Icosian));
        assert_eq!(bits(&loaded.forward(&ids, 1, 12)?)?, snapped);
        // The free-transport view of the snapped save is explicit.
        assert_eq!(
            bits(&loaded.with_unsnapped_transport(|m| m.forward(&ids, 1, 12))?)?,
            free
        );
        // A load and save round trip keeps the record; dropping the snap is
        // explicit, and a save without it removes the record.
        loaded.save(&snapped_dir)?;
        assert_eq!(
            StackModel::saved_transport_snap(&snapped_dir)?,
            Some(TransportSnap::Icosian)
        );
        loaded.set_transport_snap(None)?;
        loaded.save(&snapped_dir)?;
        assert_eq!(StackModel::saved_transport_snap(&snapped_dir)?, None);
        assert_eq!(
            StackModel::load(&snapped_dir, &cpu())?.transport_snap(),
            None
        );
        // A record of other roots is refused, by the reader and by the load.
        fs::write(
            free_dir.join(TRANSPORT_RECORD),
            br#"{"schema":"uor-r4.stack-transport/1","snap":"icosian","roots":120,"roots_sha256":"00","scope":""}"#,
        )?;
        assert!(StackModel::saved_transport_snap(&free_dir).is_err());
        assert!(StackModel::load(&free_dir, &cpu()).is_err());
        fs::remove_dir_all(&root)?;
        Ok(())
    }

    /// The exact bypass #1506's review found: a snapped export read through
    /// the offline `parse_for_reference` must not construct the D10 engine,
    /// which computes the free transport. The multiplier-free engine serves
    /// the same bytes.
    #[test]
    fn the_d10_engine_refuses_a_snapped_export_read_for_reference() -> Result<()> {
        // A LUT-format-valid width (multiple of GROUP=32) so the artifact
        // parses and the D10 snap refusal is the path exercised.
        let mut model = spread_transport(exportable("rrarra", ReadScore::Lorentz, true), 149)?;
        model.set_transport_snap(Some(TransportSnap::Icosian))?;
        let (bytes, _) = crate::stack_export::export_stack(
            &model,
            serde_json::json!({"test": "d10-bypass"}),
            None,
            Some(TransportSnap::Icosian),
        )?;
        uor_r4_integer::stack::IntegerStackModel::parse(&bytes)
            .map_err(|e| invalid(e.to_string()))?;
        let reference = uor_r4_lut::format::StackArtifact::parse_for_reference(bytes)
            .map_err(|e| invalid(e.to_string()))?;
        assert!(reference.header.transport_snap.is_some());
        let refusal = match uor_r4_lut::stack::StackModel::from_artifact(reference) {
            Err(error) => error.to_string(),
            Ok(_) => return Err(invalid("the D10 engine accepted a snapped artifact")),
        };
        assert!(refusal.contains("transport_snap=icosian"), "{refusal}");
        Ok(())
    }

    #[test]
    fn the_transport_snap_composes_with_the_served_representation() -> Result<()> {
        let roots = icosians();
        let mut model = StackModel::new(exportable("rar", ReadScore::Lorentz, true), &cpu())?;
        spread(&model, 139)?;
        for (name, var) in model.variables() {
            if name.ends_with("rec.gate.weight") {
                var.set(&var.as_tensor().affine(8.0, 0.0)?)?;
            }
        }
        model.set_served_representation(Some(Arc::new(D11Interim)))?;
        model.set_transport_snap(Some(TransportSnap::Icosian))?;
        let time = model.config.context;
        let (ids, targets) = (token_ids(2 * time, 96, 149), token_ids(2 * time, 96, 151));
        let both = model.forward(&ids, 2, time)?;
        // The composed reference reads the served view too.
        let composed = model.logits_with_transport(&ids, 2, time, Some(&roots))?;
        let gap = max_abs_gap(&both, &composed)?;
        assert!(
            gap < 1e-4,
            "served and snapped: the composed reference differs by {gap}"
        );
        // Each constraint moves the logits on its own.
        let served_only = model.with_unsnapped_transport(|m| m.forward(&ids, 2, time))?;
        let snapped_only = model.with_float_forward(|m| m.forward(&ids, 2, time))?;
        assert!(max_abs_gap(&both, &served_only)? > 1e-3);
        assert!(max_abs_gap(&both, &snapped_only)? > 1e-3);
        assert!(model.served_codec().is_some() && model.transport_snap().is_some());
        // A training step reaches every variable.
        let grads = model.loss(&ids, &targets, 2, time)?.backward()?;
        for (name, var) in model.variables() {
            let grad = grads
                .get(var.as_tensor())
                .ok_or_else(|| invalid(format!("{name} has no gradient")))?;
            let values = grad.flatten_all()?.to_vec1::<f32>()?;
            assert!(values.iter().all(|v| v.is_finite()), "{name}: nonfinite");
            assert!(values.iter().any(|&v| v != 0.0), "{name}: zero gradient");
        }
        Ok(())
    }

    #[test]
    fn a_served_and_snapped_model_saves_both_records_for_explicit_reapplication() -> Result<()> {
        // The combined-mode save/metadata/reapply contract: a model saved
        // with the served representation and a transport snap records both;
        // `load` restores the recorded snap but not the served representation,
        // and free transport stays available explicitly; reading the recorded
        // modes and reapplying them restores the saved forward bit for bit.
        let mut model = StackModel::new(exportable("rar", ReadScore::Lorentz, true), &cpu())?;
        spread(&model, 139)?;
        for (name, var) in model.variables() {
            if name.ends_with("rec.gate.weight") {
                var.set(&var.as_tensor().affine(8.0, 0.0)?)?;
            }
        }
        model.set_served_representation(Some(Arc::new(D11Interim)))?;
        model.set_transport_snap(Some(TransportSnap::Icosian))?;
        let time = model.config.context;
        let ids = token_ids(time, 96, 163);
        let both_logits = bits(&model.forward(&ids, 1, time)?)?;
        let saved_snap = model.transport_snap();
        model.set_transport_snap(None)?;
        let float_free = model.with_float_forward(|m| bits(&m.forward(&ids, 1, time)?))?;
        model.set_transport_snap(saved_snap)?;
        let root = std::env::temp_dir().join(format!(
            "geometric-stack-combined-save-{}",
            std::process::id()
        ));
        let dir = root.join("both");
        model.save(&dir)?;
        // Both metadata records read back through the declared accessors.
        assert_eq!(
            StackModel::saved_transport_snap(&dir)?,
            Some(TransportSnap::Icosian)
        );
        assert_eq!(
            StackModel::saved_served_representation(&dir)?,
            Some(SavedServedRepresentation {
                codec: D11Interim.name().to_owned()
            })
        );
        // A raw load restores the recorded snap but not the served
        // representation; the free-transport view is requested explicitly.
        let mut loaded = StackModel::load(&dir, &cpu())?;
        assert!(loaded.served_codec().is_none());
        assert_eq!(loaded.transport_snap(), Some(TransportSnap::Icosian));
        assert_eq!(
            bits(&loaded.with_unsnapped_transport(|m| m.forward(&ids, 1, time))?)?,
            float_free
        );
        assert_ne!(bits(&loaded.forward(&ids, 1, time)?)?, both_logits);
        // Reapplying the recorded modes restores the saved forward exactly.
        loaded.set_served_representation(Some(Arc::new(D11Interim)))?;
        loaded.set_transport_snap(StackModel::saved_transport_snap(&dir)?)?;
        assert_eq!(bits(&loaded.forward(&ids, 1, time)?)?, both_logits);
        // The recorded snap makes the integer export's refusal decidable from
        // the directory alone, before any weights are exported.
        assert!(StackModel::saved_transport_snap(&dir)?.is_some());
        fs::remove_dir_all(&root)?;
        Ok(())
    }

    /// The snap's cost in the fused recurrence core at the main line's shape
    /// (width 288, so 72 lanes, and 16 windows of 256): one layer's forward
    /// and backward, alternating free and snapped, median seconds. Timing
    /// only; run in a release build with `--ignored`.
    #[test]
    #[ignore]
    fn the_snap_cost_in_the_fused_core() -> Result<()> {
        let (batch, time, width) = (16, 256, 288);
        let lanes = width / 4;
        let mut rng = Initializer(157);
        let branches = Var::from_tensor(&random(&mut rng, &[batch, time, 2 * width], 1.0))?;
        let gates = Var::from_tensor(&random(&mut rng, &[batch, time, lanes + width], 1.0))?;
        let parameters = Var::from_tensor(&random(
            &mut rng,
            &[(CONVOLUTION_WIDTH + 1) * width + lanes],
            0.3,
        ))?;
        let weights = random(&mut rng, &[batch, time, width], 1.0);
        let mut seconds = [Vec::new(), Vec::new()];
        for _ in 0..7 {
            for (arm, snap) in [None, Some(TransportSnap::Icosian)].into_iter().enumerate() {
                let clock = Instant::now();
                let out = branches.as_tensor().apply_op3(
                    gates.as_tensor(),
                    parameters.as_tensor(),
                    RecurrenceCore {
                        batch,
                        time,
                        width,
                        rotation: true,
                        group: RotationGroup::Quaternion,
                        snap,
                    },
                )?;
                out.mul(&weights)?.sum_all()?.backward()?;
                seconds[arm].push(clock.elapsed().as_secs_f64());
            }
        }
        let median = |values: &mut Vec<f64>| {
            values.sort_by(f64::total_cmp);
            values[values.len() / 2]
        };
        let (free, snapped) = (median(&mut seconds[0]), median(&mut seconds[1]));
        eprintln!(
            "fused recurrence core, one layer forward and backward: free {free:.4} s, icosian \
             snap {snapped:.4} s ({:+.1}%)",
            100.0 * (snapped / free - 1.0)
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // A1: flock selection of the reads and the pointer-copy head.

    /// Random inputs of a fused read, as `read_shape` draws them.
    struct ReadInputs {
        q: Var,
        k: Var,
        v: Var,
        null: Var,
        age: Var,
        beta: Var,
        offset: Var,
    }

    impl ReadInputs {
        fn draw(
            seed: u64,
            (batch, heads, time, width, value_width): (usize, usize, usize, usize, usize),
            spread: f64,
        ) -> Result<Self> {
            let mut rng = Initializer(seed);
            Ok(Self {
                q: Var::from_tensor(&random(&mut rng, &[batch, heads, time, width], spread))?,
                k: Var::from_tensor(&random(&mut rng, &[batch, heads, time, width], spread))?,
                v: Var::from_tensor(&random(&mut rng, &[batch, heads, time, value_width], 1.0))?,
                null: Var::from_tensor(&random(&mut rng, &[batch, heads, time], 1.0))?,
                age: Var::from_tensor(&random(&mut rng, &[heads, time], 0.5))?,
                beta: Var::from_tensor(&random(&mut rng, &[heads], 0.3).affine(1.0, 1.0)?)?,
                offset: Var::from_tensor(&random(&mut rng, &[heads], 0.5).affine(1.0, 2.0)?)?,
            })
        }

        fn aux(&self, lorentz: bool) -> Result<Tensor> {
            let mut parts = vec![
                self.null.as_tensor().flatten_all()?,
                self.age.as_tensor().flatten_all()?,
            ];
            if lorentz {
                parts.push(self.beta.as_tensor().clone());
                parts.push(self.offset.as_tensor().clone());
            }
            Ok(Tensor::cat(&parts, 0)?)
        }

        fn vars(&self, lorentz: bool) -> Vec<Var> {
            let mut vars = vec![
                self.q.clone(),
                self.k.clone(),
                self.v.clone(),
                self.null.clone(),
                self.age.clone(),
            ];
            if lorentz {
                vars.push(self.beta.clone());
                vars.push(self.offset.clone());
            }
            vars
        }

        fn lorentz(&self, lorentz: bool) -> Option<(&Tensor, &Tensor)> {
            lorentz.then(|| (self.beta.as_tensor(), self.offset.as_tensor()))
        }
    }

    /// The total score (the softmax's argument, before the NoRead slot) of every
    /// source of every row, in f64 by plain loops: entry
    /// `((b * heads + h) * time + t) * time + j` for `j <= t`.
    fn read_scores(x: &ReadInputs, lorentz: bool) -> Result<Vec<f64>> {
        let (batch, heads, time, width) = x.q.dims4()?;
        let q = x.q.as_tensor().flatten_all()?.to_vec1::<f32>()?;
        let k = x.k.as_tensor().flatten_all()?.to_vec1::<f32>()?;
        let age = x.age.as_tensor().flatten_all()?.to_vec1::<f32>()?;
        let beta = x.beta.as_tensor().to_vec1::<f32>()?;
        let offset = x.offset.as_tensor().to_vec1::<f32>()?;
        let lift =
            |row: &[f32]| (1.0 + row.iter().map(|&v| f64::from(v).powi(2)).sum::<f64>()).sqrt();
        let mut out = vec![0f64; batch * heads * time * time];
        for row in 0..batch * heads {
            let (block, h) = (row * time * width, row % heads);
            for t in 0..time {
                for j in 0..=t {
                    let query = &q[block + t * width..block + (t + 1) * width];
                    let key = &k[block + j * width..block + (j + 1) * width];
                    let inner: f64 = query
                        .iter()
                        .zip(key)
                        .map(|(&a, &b)| f64::from(a) * f64::from(b))
                        .sum();
                    let age = f64::from(age[h * time + (t - j)]);
                    out[(row * time + t) * time + j] = if lorentz {
                        let excess = lift(query) * lift(key) - inner - 1.0;
                        let distance = lorentz_distance(excess);
                        -f64::from(beta[h]) * (distance - f64::from(offset[h])) + age
                    } else {
                        inner / (width as f64).sqrt() + age
                    };
                }
            }
        }
        Ok(out)
    }

    /// The kept sources of one row by an independent implementation (a full
    /// sort): the sink, the last `window` positions and the `k` best of the
    /// rest, ties to the lowest position.
    fn kept_sources(select: FlockSelect, t: usize, scores: &[f64]) -> Vec<bool> {
        let mut keep = vec![false; t + 1];
        keep[0] = true;
        let recent = (t + 1).saturating_sub(select.window);
        for slot in &mut keep[recent..] {
            *slot = true;
        }
        let mut rest: Vec<usize> = (1..recent).collect();
        rest.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]).then(a.cmp(&b)));
        for &j in rest.iter().take(select.k) {
            keep[j] = true;
        }
        keep
    }

    /// The `k` best sources of one row by an independent implementation (a
    /// full sort), with no sink and no window; ties to the lowest position.
    fn kept_top_k(k: usize, scores: &[f64]) -> Vec<bool> {
        let mut order: Vec<usize> = (0..scores.len()).collect();
        order.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]).then(a.cmp(&b)));
        let mut keep = vec![false; scores.len()];
        for &j in order.iter().take(k) {
            keep[j] = true;
        }
        keep
    }

    /// The smallest gap, over all rows, between the `k`-th and `(k + 1)`-th
    /// best remaining source (infinite when no row has to choose).
    fn selection_gap(select: FlockSelect, scores: &[f64], rows: usize, time: usize) -> f64 {
        let mut gap = f64::INFINITY;
        for row in 0..rows {
            for t in 0..time {
                let recent = (t + 1).saturating_sub(select.window);
                let s = &scores[(row * time + t) * time..(row * time + t) * time + t + 1];
                let mut rest: Vec<f64> = (1..recent).map(|j| s[j]).collect();
                rest.sort_by(|a, b| b.total_cmp(a));
                if select.k > 0 && rest.len() > select.k {
                    gap = gap.min(rest[select.k - 1] - rest[select.k]);
                }
            }
        }
        gap
    }

    /// The A1 prototype's selection of one read row, verbatim (its own
    /// `FlockSelect { window, k }` and `flock_mask`, before the shared
    /// selector replaced them), as an independent reference: keeps the sink
    /// `0`, the positions `t + 1 - window..=t` and the `k` highest of
    /// `scores[1..t + 1 - window]` (ties to the lowest position), and sets
    /// every other entry of `scores[..=t]` to `floor`. Returns whether any was
    /// dropped.
    fn legacy_flock_mask<T: Copy + PartialOrd>(
        window: usize,
        k: usize,
        t: usize,
        scores: &mut [T],
        floor: T,
        top: &mut Vec<(T, usize)>,
    ) -> bool {
        debug_assert!(scores.len() == t + 1 && window >= 1);
        let recent = (t + 1).saturating_sub(window);
        if recent <= 1 || recent - 1 <= k {
            return false;
        }
        if k == 0 {
            scores[1..recent].fill(floor);
            return true;
        }
        top.clear();
        for (j, &score) in scores.iter().enumerate().take(recent).skip(1) {
            if top.len() == k {
                if score.partial_cmp(&top[k - 1].0) != Some(std::cmp::Ordering::Greater) {
                    continue;
                }
                top.pop();
            }
            let at = top.partition_point(|&(kept, _)| kept >= score);
            top.insert(at, (score, j));
        }
        scores[1..recent].fill(floor);
        for &(score, j) in top.iter() {
            scores[j] = score;
        }
        true
    }

    #[test]
    fn the_shared_selector_keeps_what_the_prototypes_flock_mask_kept() -> Result<()> {
        // A hand-built row (its best remaining sources are positions 1 and 3)
        // and random rows.
        let by_hand = vec![0.0f32, 5.0, 1.0, 4.0, 2.0, 3.0, 0.5, 0.2, 0.1, 0.3];
        let mut rng = Initializer(2024);
        let mut rows = vec![by_hand];
        for _ in 0..40 {
            rows.push((0..14).map(|_| (rng.normal() * 2.0) as f32).collect());
        }
        for scores in &rows {
            for (window, k) in [(1, 1), (2, 2), (3, 4), (1, 9), (4, 2), (5, 1), (20, 3)] {
                for t in 0..scores.len() {
                    let mut legacy = scores[..=t].to_vec();
                    let mut top = Vec::new();
                    legacy_flock_mask(window, k, t, &mut legacy, f32::NEG_INFINITY, &mut top);
                    let kept_by_mask: Vec<usize> = (0..=t)
                        .filter(|&j| legacy[j] != f32::NEG_INFINITY)
                        .collect();
                    let selection =
                        flock::flock_select(&scores[..=t], t, FlockSelect { sink: 0, window, k })?;
                    let mut kept: Vec<usize> = selection.positions().collect();
                    kept.sort_unstable();
                    assert_eq!(kept, kept_by_mask, "window {window} k {k} row {t}");
                    // And the mask the reads apply from that selection is the
                    // prototype's mask, entry for entry.
                    let mut masked = scores[..=t].to_vec();
                    drop_unkept(&selection, &mut masked, &mut Vec::new());
                    assert_eq!(masked, legacy, "window {window} k {k} row {t}");
                }
            }
        }
        Ok(())
    }

    #[test]
    fn a_flock_covering_the_context_equals_no_selection_bit_for_bit() -> Result<()> {
        // Eleven positions of a 20-wide key run full tiles, the causal
        // triangle and a short last tile.
        let shape = (2, 3, 11, 20, 33);
        let (batch, heads, time, _, value_width) = shape;
        let weights = random(&mut Initializer(9), &[batch, heads, time, value_width], 1.0);
        let real = FlockSelect {
            sink: 0,
            window: 3,
            k: 2,
        };
        let covering = [
            FlockSelect {
                sink: 0,
                window: time,
                k: 1,
            },
            FlockSelect {
                sink: 0,
                window: 4 * time,
                k: 1,
            },
            FlockSelect {
                sink: 0,
                window: 1,
                k: time,
            },
            FlockSelect {
                sink: 0,
                window: 3,
                k: time,
            },
            FlockSelect {
                sink: 0,
                window: 1,
                k: 2 * time,
            },
        ];
        for score in [ReadScore::Dot, ReadScore::Lorentz] {
            let lorentz = score == ReadScore::Lorentz;
            let x = ReadInputs::draw(3, shape, 0.8)?;
            let (aux, vars) = (x.aux(lorentz)?, x.vars(lorentz));
            let run = |select: Option<FlockSelect>| -> Result<Vec<Vec<u32>>> {
                let out = fused_read_selected(
                    x.q.as_tensor(),
                    x.k.as_tensor(),
                    x.v.as_tensor(),
                    &aux,
                    score,
                    true,
                    true,
                    false,
                    select,
                )?;
                let grads = out.mul(&weights)?.sum_all()?.backward()?;
                let mut all = vec![bits(&out)?];
                for var in &vars {
                    all.push(bits(
                        grads
                            .get(var.as_tensor())
                            .ok_or_else(|| invalid("missing gradient"))?,
                    )?);
                }
                Ok(all)
            };
            let base = run(None)?;
            for select in covering {
                assert!(run(Some(select))? == base, "{score:?} {select:?} differs");
            }
            assert!(
                run(Some(real))? != base,
                "{score:?}: a real flock changed nothing"
            );
            // The control's attention (Dot, RoPE, no NoRead or age).
            let zero = Tensor::zeros(1, DType::F32, &cpu())?;
            let rotary = |select: Option<FlockSelect>| -> Result<Vec<Vec<u32>>> {
                let out = fused_read_selected(
                    x.q.as_tensor(),
                    x.k.as_tensor(),
                    x.v.as_tensor(),
                    &zero,
                    ReadScore::Dot,
                    false,
                    false,
                    true,
                    select,
                )?;
                let grads = out.mul(&weights)?.sum_all()?.backward()?;
                let mut all = vec![bits(&out)?];
                for var in [&x.q, &x.k, &x.v] {
                    all.push(bits(
                        grads
                            .get(var.as_tensor())
                            .ok_or_else(|| invalid("missing gradient"))?,
                    )?);
                }
                Ok(all)
            };
            let base = rotary(None)?;
            for select in covering {
                assert!(rotary(Some(select))? == base, "rotary {select:?} differs");
            }
            assert!(rotary(Some(real))? != base);
        }
        Ok(())
    }

    #[test]
    fn a_flocked_read_matches_the_masked_reference_and_finite_differences() -> Result<()> {
        let select = FlockSelect {
            sink: 0,
            window: 3,
            k: 2,
        };
        let shape = (2, 2, 12, 4, 5);
        let (batch, heads, time, _, value_width) = shape;
        for score in [ReadScore::Dot, ReadScore::Lorentz] {
            let lorentz = score == ReadScore::Lorentz;
            // Draw until every row's k-th and (k + 1)-th candidates are well
            // apart, so the finite-difference steps cannot move the selection.
            let mut drawn = None;
            for seed in 0..400 {
                let x = ReadInputs::draw(seed, shape, 1.2)?;
                let scores = read_scores(&x, lorentz)?;
                if selection_gap(select, &scores, batch * heads, time) > 0.03 {
                    drawn = Some((x, scores));
                    break;
                }
            }
            let (x, scores) = drawn.ok_or_else(|| invalid("no draw with a clear selection"))?;
            let mut mask = vec![0f32; batch * heads * time * time];
            for row in 0..batch * heads {
                for t in 0..time {
                    let start = (row * time + t) * time;
                    let keep = kept_sources(select, t, &scores[start..start + t + 1]);
                    for (j, kept) in keep.iter().enumerate() {
                        if !kept {
                            mask[start + j] = f32::NEG_INFINITY;
                        }
                    }
                }
            }
            let mask = Tensor::from_vec(mask, (batch, heads, time, time), &cpu())?;
            let (aux, vars) = (x.aux(lorentz)?, x.vars(lorentz));
            let fused = fused_read_selected(
                x.q.as_tensor(),
                x.k.as_tensor(),
                x.v.as_tensor(),
                &aux,
                score,
                true,
                true,
                false,
                Some(select),
            )?;
            let reference = reference_read_masked(
                x.q.as_tensor(),
                x.k.as_tensor(),
                x.v.as_tensor(),
                Some(x.null.as_tensor()),
                Some(x.age.as_tensor()),
                x.lorentz(lorentz),
                Some(&mask),
            )?;
            let gap = fused
                .sub(&reference)?
                .abs()?
                .max_all()?
                .to_scalar::<f32>()?;
            assert!(gap < 1e-5, "{score:?}: flocked read differs by {gap}");
            let weights = random(&mut Initializer(4), &[batch, heads, time, value_width], 1.0);
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
                    "{score:?}: gradient of {:?} differs by {gap} of {scale}",
                    var.dims()
                );
            }
            check_gradient(
                &vars,
                || {
                    // The auxiliary input is rebuilt from the perturbed variables.
                    Ok(fused_read_selected(
                        x.q.as_tensor(),
                        x.k.as_tensor(),
                        x.v.as_tensor(),
                        &x.aux(lorentz)?,
                        score,
                        true,
                        true,
                        false,
                        Some(select),
                    )?
                    .mul(&weights)?
                    .sum_all()?)
                },
                2e-3,
            )?;
        }
        Ok(())
    }

    /// The weights of a Dot read with key width 1, query 1 and one-hot values:
    /// row `t` of the output is row `t`'s weights over the sources.
    fn one_hot_weights(
        scores: &[f32],
        select: Option<FlockSelect>,
        null_logit: Option<f32>,
    ) -> Result<Vec<Vec<f32>>> {
        let time = scores.len();
        let q = Tensor::ones((1, 1, time, 1), DType::F32, &cpu())?;
        let k = Tensor::from_vec(scores.to_vec(), (1, 1, time, 1), &cpu())?;
        let mut eye = vec![0f32; time * time];
        for t in 0..time {
            eye[t * time + t] = 1.0;
        }
        let v = Tensor::from_vec(eye, (1, 1, time, time), &cpu())?;
        let aux = match null_logit {
            Some(logit) => Tensor::from_vec(vec![logit; time], time, &cpu())?,
            None => Tensor::zeros(1, DType::F32, &cpu())?,
        };
        let out = fused_read_selected(
            &q,
            &k,
            &v,
            &aux,
            ReadScore::Dot,
            null_logit.is_some(),
            false,
            false,
            select,
        )?;
        Ok(out
            .flatten_all()?
            .to_vec1::<f32>()?
            .chunks(time)
            .map(<[f32]>::to_vec)
            .collect())
    }

    #[test]
    fn a_flock_keeps_the_sink_the_window_and_the_best_and_weighs_nothing_else() -> Result<()> {
        let c = [0.0f32, 5.0, 1.0, 4.0, 2.0, 3.0, 0.5, 0.2, 0.1, 0.3];
        let select = FlockSelect {
            sink: 0,
            window: 2,
            k: 2,
        };
        let weights = one_hot_weights(&c, Some(select), None)?;
        // Kept sources by hand: the sink, the two last positions, and the two
        // best of the rest (positions 1 and 3, scores 5 and 4).
        let by_hand: [(usize, &[usize]); 6] = [
            (9, &[0, 1, 3, 8, 9]),
            (6, &[0, 1, 3, 5, 6]),
            (5, &[0, 1, 3, 4, 5]),
            (4, &[0, 1, 2, 3, 4]),
            (3, &[0, 1, 2, 3]),
            (1, &[0, 1]),
        ];
        for (t, kept) in by_hand {
            let maximum = kept.iter().map(|&j| c[j]).fold(f32::NEG_INFINITY, f32::max);
            let total: f32 = kept.iter().map(|&j| (c[j] - maximum).exp()).sum();
            for j in 0..c.len() {
                if kept.contains(&j) {
                    let want = (c[j] - maximum).exp() / total;
                    assert!((weights[t][j] - want).abs() < 1e-6, "row {t} source {j}");
                } else {
                    assert_eq!(weights[t][j], 0.0, "row {t} source {j} is not kept");
                }
            }
        }
        // The independent implementation agrees on every row.
        let as_f64: Vec<f64> = c.iter().map(|&v| f64::from(v)).collect();
        for t in 0..c.len() {
            let keep = kept_sources(select, t, &as_f64[..=t]);
            for j in 0..c.len() {
                assert_eq!(weights[t][j] > 0.0, j <= t && keep[j], "row {t} source {j}");
            }
        }
        // Ties go to the lowest position: sources 1-4 tie at the top.
        let tied = [0.0f32, 2.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        for (k, kept) in [(1usize, vec![0, 1, 9]), (3, vec![0, 1, 2, 3, 9])] {
            let weights = one_hot_weights(
                &tied,
                Some(FlockSelect {
                    sink: 0,
                    window: 1,
                    k,
                }),
                None,
            )?;
            for j in 0..tied.len() {
                assert_eq!(weights[9][j] > 0.0, kept.contains(&j), "k {k} source {j}");
            }
        }
        // The NoRead slot is always in the softmax: the output row loses its
        // probability.
        let null_logit = 1.0f32;
        let with_null = one_hot_weights(&c, Some(select), Some(null_logit))?;
        for (t, kept) in by_hand {
            let maximum = kept.iter().map(|&j| c[j]).fold(null_logit, f32::max);
            let total: f32 = kept.iter().map(|&j| (c[j] - maximum).exp()).sum::<f32>()
                + (null_logit - maximum).exp();
            let sum: f32 = with_null[t].iter().sum();
            let want = 1.0 - (null_logit - maximum).exp() / total;
            assert!((sum - want).abs() < 1e-6, "row {t}: {sum} against {want}");
            for j in 0..c.len() {
                if !kept.contains(&j) {
                    assert_eq!(with_null[t][j], 0.0);
                }
            }
        }
        // The backward: a loss on row 9 alone gives zero gradient to unkept
        // keys and values, and a nonzero one to the kept.
        let time = c.len();
        let k = Var::from_vec(c.to_vec(), (1, 1, time, 1), &cpu())?;
        let mut eye = vec![0f32; time * time];
        for t in 0..time {
            eye[t * time + t] = 1.0;
        }
        let v = Var::from_vec(eye, (1, 1, time, time), &cpu())?;
        let q = Tensor::ones((1, 1, time, 1), DType::F32, &cpu())?;
        let out = fused_read_selected(
            &q,
            k.as_tensor(),
            v.as_tensor(),
            &Tensor::zeros(1, DType::F32, &cpu())?,
            ReadScore::Dot,
            false,
            false,
            false,
            Some(select),
        )?;
        let mut row_weights = vec![0f32; time * time];
        for j in 0..time {
            row_weights[9 * time + j] = 1.0 + j as f32;
        }
        let row_weights = Tensor::from_vec(row_weights, (1, 1, time, time), &cpu())?;
        let grads = out.mul(&row_weights)?.sum_all()?.backward()?;
        let dk = grads
            .get(k.as_tensor())
            .expect("key gradient")
            .flatten_all()?
            .to_vec1::<f32>()?;
        let dv = grads
            .get(v.as_tensor())
            .expect("value gradient")
            .flatten_all()?
            .to_vec1::<f32>()?;
        for j in 0..time {
            let kept = [0, 1, 3, 8, 9].contains(&j);
            assert_eq!(dk[j] != 0.0, kept, "key {j}");
            assert_eq!(
                dv[j * time..(j + 1) * time].iter().any(|&g| g != 0.0),
                kept,
                "value {j}"
            );
        }
        Ok(())
    }

    #[test]
    fn a_flocked_read_of_a_row_it_cannot_rank_is_an_error_not_a_silent_nan() -> Result<()> {
        let (time, width) = (6, 2);
        let mut q = vec![0.5f32; time * width];
        // Row 1's second query entry is NaN, so every score of row 1 is.
        q[3] = f32::NAN;
        let query = Tensor::from_vec(q, (1, 1, time, width), &cpu())?;
        let key = Tensor::ones((1, 1, time, width), DType::F32, &cpu())?;
        let value = Tensor::ones((1, 1, time, 2), DType::F32, &cpu())?;
        let zero = Tensor::zeros(1, DType::F32, &cpu())?;
        let select = FlockSelect {
            sink: 0,
            window: 2,
            k: 1,
        };
        let error = fused_read_selected(
            &query,
            &key,
            &value,
            &zero,
            ReadScore::Dot,
            false,
            false,
            false,
            Some(select),
        )
        .expect_err("a NaN score cannot be ranked");
        assert!(error.to_string().contains("finite"), "{error}");
        // Without a selection the read runs (and carries the NaN) as before.
        assert!(fused_read_selected(
            &query,
            &key,
            &value,
            &zero,
            ReadScore::Dot,
            false,
            false,
            false,
            None,
        )
        .is_ok());
        Ok(())
    }

    /// The pointer's score of source `j` for the query at position `t`, by
    /// plain f64 loops over `side`'s `[query | key | gate logit]` rows of a
    /// window whose first row is `first`.
    fn reference_pointer_score(
        side: &[f32],
        dim: usize,
        (first, t, j): (usize, usize, usize),
        score: ReadScore,
        beta: f64,
    ) -> f64 {
        let stride = 2 * dim + 1;
        let query: Vec<f64> = side[(first + t) * stride..(first + t) * stride + dim]
            .iter()
            .map(|&v| f64::from(v))
            .collect();
        let key: Vec<f64> = side[(first + j) * stride + dim..(first + j) * stride + 2 * dim]
            .iter()
            .map(|&v| f64::from(v))
            .collect();
        let inner: f64 = query.iter().zip(&key).map(|(a, b)| a * b).sum();
        match score {
            ReadScore::Dot => inner / (dim as f64).sqrt(),
            ReadScore::Lorentz => {
                let lift = |x: &[f64]| (1.0 + x.iter().map(|v| v * v).sum::<f64>()).sqrt();
                -beta * lorentz_distance(lift(&query) * lift(&key) - inner - 1.0)
            }
            ReadScore::L2 => unreachable!("an L2 pointer is refused"),
        }
    }

    #[test]
    fn the_pointers_attention_uses_its_own_score_and_selection() -> Result<()> {
        let (dim, time) = (3, 9);
        let stride = 2 * dim + 1;
        let mut rng = Initializer(123);
        let side: Vec<f32> = (0..time * stride)
            .map(|_| (rng.normal() * 1.3) as f32)
            .collect();
        let window_two = FlockSelect {
            sink: 0,
            window: 2,
            k: 2,
        };
        for (score, beta) in [(ReadScore::Dot, 0.0), (ReadScore::Lorentz, 1.7)] {
            for t in 0..time {
                let scores: Vec<f64> = (0..=t)
                    .map(|j| reference_pointer_score(&side, dim, (0, t, j), score, beta))
                    .collect();
                for select in [
                    None,
                    Some(PointerSelect::Flock(window_two)),
                    Some(PointerSelect::TopK(1)),
                    Some(PointerSelect::TopK(3)),
                ] {
                    let rule = PointerRule {
                        dim,
                        score,
                        select,
                        beta,
                        route: None,
                    };
                    let a = pointer_attention(&side, 0, t, &rule, &[])?;
                    let kept: Vec<bool> = match select {
                        None => vec![true; t + 1],
                        Some(PointerSelect::Flock(select)) => kept_sources(select, t, &scores),
                        Some(PointerSelect::TopK(k)) => kept_top_k(k, &scores),
                    };
                    let maximum = (0..=t)
                        .filter(|&j| kept[j])
                        .map(|j| scores[j])
                        .fold(f64::NEG_INFINITY, f64::max);
                    let total: f64 = (0..=t)
                        .filter(|&j| kept[j])
                        .map(|j| (scores[j] - maximum).exp())
                        .sum();
                    for j in 0..=t {
                        if kept[j] {
                            let want = (scores[j] - maximum).exp() / total;
                            assert!(
                                (a[j] - want).abs() < 1e-5,
                                "{score:?} {select:?} row {t} source {j}"
                            );
                        } else {
                            assert_eq!(a[j], 0.0, "{score:?} {select:?} row {t} source {j}");
                        }
                    }
                    if select == Some(PointerSelect::TopK(1)) {
                        // The single-source pointer: weight 1 on one source.
                        assert_eq!(a.iter().filter(|&&w| w > 0.0).count(), 1);
                        assert!(a.iter().any(|&w| w == 1.0), "row {t}");
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn the_lorentz_pointer_scores_like_the_fused_read() -> Result<()> {
        let (time, width) = (7, 3);
        let mut rng = Initializer(77);
        let q = random(&mut rng, &[1, 1, time, width], 0.9);
        let k = random(&mut rng, &[1, 1, time, width], 0.9);
        let mut eye = vec![0f32; time * time];
        for t in 0..time {
            eye[t * time + t] = 1.0;
        }
        let eye = Tensor::from_vec(eye, (1, 1, time, time), &cpu())?;
        let beta = 1.7f64;
        // The read's auxiliary input without NoRead or age is [beta, offset];
        // the offset moves every score alike and drops out of the softmax.
        let aux = Tensor::from_vec(vec![beta as f32, 0.6f32], 2, &cpu())?;
        let read = fused_read(&q, &k, &eye, &aux, ReadScore::Lorentz, false, false, false)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        // The pointer's `side` rows are [query | key | gate logit].
        let (qv, kv) = (
            q.flatten_all()?.to_vec1::<f32>()?,
            k.flatten_all()?.to_vec1::<f32>()?,
        );
        let mut side = Vec::new();
        for t in 0..time {
            side.extend_from_slice(&qv[t * width..(t + 1) * width]);
            side.extend_from_slice(&kv[t * width..(t + 1) * width]);
            side.push(0.0f32);
        }
        let rule = PointerRule {
            dim: width,
            score: ReadScore::Lorentz,
            select: None,
            beta,
            route: None,
        };
        for t in 0..time {
            let a = pointer_attention(&side, 0, t, &rule, &[])?;
            for j in 0..=t {
                assert!(
                    (a[j] - f64::from(read[t * time + j])).abs() < 1e-5,
                    "row {t} source {j}: pointer {} against read {}",
                    a[j],
                    read[t * time + j]
                );
            }
        }
        Ok(())
    }

    /// A tiny model with a pointer head whose weights are made large enough
    /// for the attention and the gate to matter, drawn from `seed`.
    fn pointer_model(
        score: ReadScore,
        select: Option<PointerSelect>,
        seed: u64,
    ) -> Result<StackModel> {
        let mut config = tiny(StackArch::Geometric, "ar", ReadScore::Lorentz, true);
        config.pointer = Some(PointerConfig {
            score,
            select,
            ..PointerConfig::new(4)
        });
        let model = StackModel::new(config, &cpu())?;
        let mut rng = Initializer(31 + seed);
        for (name, scale) in [
            ("pointer.query.weight", 0.3),
            ("pointer.key.weight", 0.3),
            ("pointer.gate.weight", 0.5),
        ] {
            let var = &model.variables()[name];
            var.set(&random(&mut rng, var.dims(), scale))?;
        }
        model.variables()["pointer.gate.bias"].set(&Tensor::from_vec(vec![0.3f32], 1, &cpu())?)?;
        if score == ReadScore::Lorentz {
            model.variables()["pointer.log_beta"].set(&Tensor::from_vec(
                vec![0.4f32],
                1,
                &cpu(),
            )?)?;
        }
        Ok(model)
    }

    /// The variables of a pointer head.
    fn pointer_names(score: ReadScore) -> Vec<&'static str> {
        let mut names = vec![
            "pointer.query.weight",
            "pointer.key.weight",
            "pointer.gate.weight",
            "pointer.gate.bias",
        ];
        if score == ReadScore::Lorentz {
            names.push("pointer.log_beta");
        }
        names
    }

    /// Two windows of 12 over a small alphabet, so targets recur in context
    /// (and, on other rows, do not: those rows have no copy mass), with
    /// response weights that include zeros.
    fn pointer_batch() -> (Vec<u32>, Vec<u32>, Vec<f32>) {
        let ids: Vec<u32> = (0..24u32).map(|i| (i * 5 + 1) % 6).collect();
        let targets: Vec<u32> = (0..24u32).map(|i| (i * 3 + 2) % 6).collect();
        let weights: Vec<f32> = (0..24)
            .map(|i| {
                if i % 4 == 1 {
                    0.0
                } else {
                    1.0 + (i % 3) as f32
                }
            })
            .collect();
        (ids, targets, weights)
    }

    #[test]
    fn without_a_pointer_the_loss_is_the_current_loss_bit_for_bit() -> Result<()> {
        let model = StackModel::new(
            tiny(StackArch::Geometric, "ar", ReadScore::Lorentz, true),
            &cpu(),
        )?;
        assert!(model.config.pointer.is_none() && model.config.select.is_none());
        let (ids, targets, weights) = pointer_batch();
        let logits = model.forward(&ids, 2, 12)?;
        let weighted = model.weighted_loss(&ids, &targets, &weights, 2, 12)?;
        let want = logits_cross_entropy(&logits, &targets, Some(&weights))?;
        assert_eq!(
            weighted.to_scalar::<f32>()?.to_bits(),
            want.to_scalar::<f32>()?.to_bits()
        );
        let plain = model.loss(&ids, &targets, 2, 12)?;
        let want = logits_cross_entropy(&logits, &targets, None)?;
        assert_eq!(
            plain.to_scalar::<f32>()?.to_bits(),
            want.to_scalar::<f32>()?.to_bits()
        );
        // The pointer's names are absent, and a config without the two
        // settings serializes without their keys.
        assert!(model
            .variables()
            .keys()
            .all(|name| !name.starts_with("pointer.")));
        let json = serde_json::to_string(&model.config)?;
        assert!(
            !json.contains("select") && !json.contains("pointer"),
            "{json}"
        );
        let loaded: StackConfig = serde_json::from_str(&json)?;
        assert_eq!(loaded, model.config);
        Ok(())
    }

    /// The mixture loss of every row by plain f64 loops and direct
    /// probabilities (no floor), from the model's current variables and its
    /// final states `hidden`, the smallest gap of the pointer's selection over
    /// the scored rows (the finite-difference steps must not move it), and
    /// whether some scored row keeps both a source holding its target and one
    /// that does not: only such a row lets the pointer's scores move the loss.
    /// Rows of weight zero read 0.
    fn reference_mixture(
        model: &StackModel,
        hidden: &[Vec<f32>],
        (ids, targets, weights): (&[u32], &[u32], &[f32]),
        (batch, time): (usize, usize),
    ) -> Result<(Vec<f64>, f64, bool)> {
        let pointer = model
            .config
            .pointer
            .ok_or_else(|| invalid("no pointer head"))?;
        let variable = |name: &str| model.variables()[name].as_tensor().clone();
        let embedding = variable("embedding.weight").to_vec2::<f32>()?;
        let (wq, wk, wg) = (
            variable("pointer.query.weight").to_vec2::<f32>()?,
            variable("pointer.key.weight").to_vec2::<f32>()?,
            variable("pointer.gate.weight").to_vec2::<f32>()?,
        );
        let bias = f64::from(variable("pointer.gate.bias").to_vec1::<f32>()?[0]);
        let beta = match pointer.score {
            ReadScore::Dot => 0.0,
            ReadScore::Lorentz => {
                f64::from(variable("pointer.log_beta").to_vec1::<f32>()?[0]).exp()
            }
            ReadScore::L2 => unreachable!("an L2 pointer is refused"),
        };
        let project = |w: &[Vec<f32>], h: &[f32]| -> Vec<f64> {
            w.iter()
                .map(|row| {
                    row.iter()
                        .zip(h)
                        .map(|(&a, &b)| f64::from(a) * f64::from(b))
                        .sum()
                })
                .collect()
        };
        let dim = wq.len();
        let mut gap = f64::INFINITY;
        let mut contested = false;
        let mut rows = vec![0.0; batch * time];
        for n in 0..batch * time {
            if weights[n] == 0.0 {
                continue;
            }
            let (first, t) = (n - n % time, n % time);
            let query = project(&wq, &hidden[n]);
            let scores: Vec<f64> = (0..=t)
                .map(|j| {
                    let key = project(&wk, &hidden[first + j]);
                    let inner: f64 = query.iter().zip(&key).map(|(a, b)| a * b).sum();
                    match pointer.score {
                        ReadScore::Dot => inner / (dim as f64).sqrt(),
                        ReadScore::Lorentz => {
                            let lift =
                                |x: &[f64]| (1.0 + x.iter().map(|v| v * v).sum::<f64>()).sqrt();
                            -beta * lorentz_distance(lift(&query) * lift(&key) - inner - 1.0)
                        }
                        ReadScore::L2 => unreachable!("an L2 pointer is refused"),
                    }
                })
                .collect();
            let keep = match pointer.select {
                None => vec![true; t + 1],
                Some(PointerSelect::Flock(select)) => {
                    let mut rest: Vec<f64> = (1..(t + 1).saturating_sub(select.window))
                        .map(|j| scores[j])
                        .collect();
                    rest.sort_by(|a, b| b.total_cmp(a));
                    if rest.len() > select.k {
                        gap = gap.min(rest[select.k - 1] - rest[select.k]);
                    }
                    kept_sources(select, t, &scores)
                }
                Some(PointerSelect::TopK(k)) => {
                    let mut all = scores.clone();
                    all.sort_by(|a, b| b.total_cmp(a));
                    if all.len() > k {
                        gap = gap.min(all[k - 1] - all[k]);
                    }
                    kept_top_k(k, &scores)
                }
            };
            let maximum = (0..=t)
                .filter(|&j| keep[j])
                .map(|j| scores[j])
                .fold(f64::NEG_INFINITY, f64::max);
            let exps: Vec<f64> = (0..=t)
                .map(|j| {
                    if keep[j] {
                        (scores[j] - maximum).exp()
                    } else {
                        0.0
                    }
                })
                .collect();
            let normal: f64 = exps.iter().sum();
            // A routed pointer keeps the sources its route admits: the exact
            // route weighs them by the route's score, the ranked one by the
            // learned score plus that, and falls back to the learned
            // softmax over every source when none is admitted.
            let admitted = pointer
                .route
                .map(|route| route_scores(&ids[first..=first + t], t, route))
                .unwrap_or_default();
            let (attention, keep): (Vec<f64>, Vec<bool>) = match pointer.route {
                Some(route) if route.ranked && !admitted.is_empty() => {
                    let combined: Vec<(usize, f64)> = admitted
                        .iter()
                        .map(|&(j, score)| (j, scores[j] + ROUTE_SHARPNESS * score))
                        .collect();
                    let attention = admitted_softmax(t, &combined);
                    let kept = attention.iter().map(|&a| a > 0.0).collect();
                    (attention, kept)
                }
                Some(route) if route.ranked => (exps.iter().map(|e| e / normal).collect(), keep),
                Some(route) => {
                    let attention = route_attention(&ids[first..=first + t], t, route);
                    let admitted = attention.iter().map(|&a| a > 0.0).collect();
                    (attention, admitted)
                }
                None => (exps.iter().map(|e| e / normal).collect(), keep),
            };
            let holds = |j: usize| ids[first + j] == targets[n];
            let copy: f64 = (0..=t).filter(|&j| holds(j)).map(|j| attention[j]).sum();
            contested |=
                (0..=t).any(|j| keep[j] && holds(j)) && (0..=t).any(|j| keep[j] && !holds(j));
            let gate = 1.0 / (1.0 + (-(project(&wg, &hidden[n])[0] + bias)).exp());
            let logits: Vec<f64> = embedding
                .iter()
                .map(|row| {
                    row.iter()
                        .zip(&hidden[n])
                        .map(|(&a, &b)| f64::from(a) * f64::from(b))
                        .sum()
                })
                .collect();
            let peak = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let lse = peak + logits.iter().map(|z| (z - peak).exp()).sum::<f64>().ln();
            let generate = (logits[targets[n] as usize] - lse).exp();
            rows[n] = -((1.0 - gate) * generate + gate * copy).ln();
        }
        Ok((rows, gap, contested))
    }

    /// The weighted mean of per-row losses.
    fn weighted_mean(rows: &[f64], weights: &[f32]) -> f64 {
        let total: f64 = weights.iter().map(|&w| f64::from(w)).sum();
        rows.iter()
            .zip(weights)
            .map(|(&nll, &w)| f64::from(w) * nll)
            .sum::<f64>()
            / total
    }

    /// The derivative of the reference loss with respect to every element of
    /// the variable `name`, by central differences in f64 over the elements'
    /// stored f32 values (the reference recomputes everything from them). The
    /// variable is restored.
    fn reference_derivative(
        model: &StackModel,
        name: &str,
        hidden: &[Vec<f32>],
        data: (&[u32], &[u32], &[f32]),
        shape: (usize, usize),
    ) -> Result<Vec<f64>> {
        let var = &model.variables()[name];
        let original = var.as_tensor().flatten_all()?.to_vec1::<f32>()?;
        let loss_at = |values: &[f32]| -> Result<f64> {
            var.set(&Tensor::from_vec(values.to_vec(), var.dims(), &cpu())?)?;
            let (rows, _, _) = reference_mixture(model, hidden, data, shape)?;
            Ok(weighted_mean(&rows, data.2))
        };
        let mut derivative = Vec::with_capacity(original.len());
        for i in 0..original.len() {
            let (mut plus, mut minus) = (original.clone(), original.clone());
            plus[i] += 1e-4;
            minus[i] -= 1e-4;
            let step = f64::from(plus[i]) - f64::from(minus[i]);
            derivative.push((loss_at(&plus)? - loss_at(&minus)?) / step);
        }
        var.set(&Tensor::from_vec(original, var.dims(), &cpu())?)?;
        Ok(derivative)
    }

    #[test]
    fn the_pointer_loss_is_the_mixture_and_its_gradient_matches_finite_differences() -> Result<()> {
        let window_three = FlockSelect {
            sink: 0,
            window: 3,
            k: 2,
        };
        let arms = [
            (ReadScore::Dot, None),
            (ReadScore::Dot, Some(PointerSelect::Flock(window_three))),
            (ReadScore::Lorentz, None),
            (ReadScore::Lorentz, Some(PointerSelect::Flock(window_three))),
            (ReadScore::Lorentz, Some(PointerSelect::TopK(2))),
            (ReadScore::Lorentz, Some(PointerSelect::TopK(1))),
        ];
        let (ids, targets, weights) = pointer_batch();
        let data = (&ids[..], &targets[..], &weights[..]);
        for (score, select) in arms {
            // A single kept source has weight 1 whatever its score, so the
            // pointer's scoring parameters cannot move the loss and their
            // gradient is exactly 0 (why the pre-registered arm trains soft and
            // applies `top:1` post hoc); only the gate learns.
            let single_source = select == Some(PointerSelect::TopK(1));
            // Draw until every row's selection is clear, so the finite-difference
            // steps cannot move it, and (for a pointer with more than one kept
            // source) some row keeps both a source holding its target and one
            // that does not, so the scores can move the loss at all.
            let mut found = None;
            for seed in 0..200u64 {
                let model = pointer_model(score, select, seed)?;
                let hidden = model.hidden(&ids, 2, 12)?.to_vec2::<f32>()?;
                let (rows, gap, contested) = reference_mixture(&model, &hidden, data, (2, 12))?;
                if gap > 0.01 && (single_source || contested) {
                    found = Some((model, hidden, rows));
                    break;
                }
            }
            let (model, hidden, rows) = found.ok_or_else(|| {
                invalid(format!(
                    "{score:?} {select:?}: no draw with a clear selection and a contested row"
                ))
            })?;
            // The loss and every scored row are the unfloored mixture.
            let loss = model.weighted_loss(&ids, &targets, &weights, 2, 12)?;
            let got = f64::from(loss.to_scalar::<f32>()?);
            let want = weighted_mean(&rows, &weights);
            assert!(
                (got - want).abs() < 1e-5 * want.abs().max(1.0),
                "{score:?} {select:?}: {got} against {want}"
            );
            let scored = model.score_targets(&ids, &targets, Some(&weights), 2, 12)?;
            for n in 0..24 {
                assert!(
                    (scored.nll[n] - rows[n]).abs() < 1e-5 * rows[n].abs().max(1.0),
                    "{score:?} {select:?}: row {n} scored {} against {}",
                    scored.nll[n],
                    rows[n]
                );
            }
            // Every element of every pointer variable's gradient against the
            // f64 finite difference of the reference.
            let names = pointer_names(score);
            let grads = loss.backward()?;
            let mut moving = Vec::new();
            for name in &names {
                let numeric = reference_derivative(&model, name, &hidden, data, (2, 12))?;
                let analytic = grads
                    .get(model.variables()[*name].as_tensor())
                    .ok_or_else(|| invalid("missing gradient"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                if single_source
                    && matches!(
                        *name,
                        "pointer.query.weight" | "pointer.key.weight" | "pointer.log_beta"
                    )
                {
                    assert!(
                        analytic.iter().all(|&a| a == 0.0),
                        "{score:?}: {name} has a gradient under a single-source pointer"
                    );
                    assert!(
                        numeric.iter().all(|&n| n.abs() < 1e-12),
                        "{score:?}: {name} moves the loss under a single-source pointer"
                    );
                    continue;
                }
                moving.push(*name);
                let scale = numeric.iter().fold(0.0f64, |m, v| m.max(v.abs()));
                assert!(
                    scale > 0.0,
                    "{score:?} {select:?}: {name} has no effect on the loss"
                );
                for (i, (&a, &n)) in analytic.iter().zip(&numeric).enumerate() {
                    assert!(
                        (f64::from(a) - n).abs() < 2e-4 * scale + 2e-6,
                        "{score:?} {select:?}: {name}[{i}] analytic {a} against numeric {n}"
                    );
                }
            }
            // The optimizer reaches the pointer's variables that have a
            // gradient, the scale included.
            let mut optimizer = StackAdamW::new(&model, 0.0, 1.0)?;
            let before: Vec<Vec<u32>> = moving
                .iter()
                .map(|name| bits(model.variables()[*name].as_tensor()))
                .collect::<Result<_>>()?;
            optimizer.update(&model, &grads, 0.01)?;
            for (name, before) in moving.iter().zip(&before) {
                assert!(
                    &bits(model.variables()[*name].as_tensor())? != before,
                    "{score:?} {select:?}: {name} did not move"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn token_primes_are_the_registered_primes_in_order() {
        assert_eq!(token_prime(0), Some(2));
        assert_eq!(token_prime(1), Some(3));
        assert_eq!(token_prime(9), Some(29));
        assert_eq!(token_prime(65_535), Some(821_641));
        assert_eq!(token_prime(65_536), None);
        // An unregistered token is no atom of a key.
        assert_eq!(route_key(&[65_536, 0], 1, 2), 2);
        assert_eq!(route_key(&[1, 1, 0], 2, 3), 6);
    }

    #[test]
    fn a_route_copies_the_value_after_the_matching_key() {
        let two = PrimeRoute::exact(2);
        // "10 11 50" earlier; the query ends with "10 11".
        let ids = [5u32, 10, 11, 50, 6, 7, 10, 11];
        let attention = route_attention(&ids, 7, two);
        assert_eq!(attention.len(), 8);
        assert!(attention[3] > 0.999, "{attention:?}");
        assert!((attention.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        // The key ending at 6 overlaps the query's own tokens: not a match.
        assert_eq!(attention[7], 0.0);
    }

    #[test]
    fn a_route_without_a_shared_atom_attends_nowhere() {
        let one = PrimeRoute::exact(1);
        let two = PrimeRoute::exact(2);
        assert!(route_attention(&[1, 2, 3, 4], 3, one)
            .iter()
            .all(|&a| a == 0.0));
        // The only key wholly before `[5, 6]` is `[4]`: nothing shared,
        // although the overlapping key `[4, 5]` shares 5.
        assert!(route_attention(&[4, 5, 6], 2, two)
            .iter()
            .all(|&a| a == 0.0));
        assert_eq!(route_attention(&[9], 0, one), vec![0.0]);
    }

    #[test]
    fn a_route_orders_by_shared_rarity_then_recency() {
        let one = PrimeRoute::exact(1);
        // Two earlier 9s: the later one's successor gets more mass, by the
        // recency term only.
        let a = route_attention(&[9, 1, 9, 2, 9], 4, one);
        assert!(a[1] > 0.0 && a[3] > a[1], "{a:?}");
        let ratio = (ROUTE_SHARPNESS * ROUTE_RECENCY * 2.0 / 5.0).exp();
        assert!((a[3] / a[1] - ratio).abs() < 1e-9);
        assert_eq!(a[0] + a[2] + a[4], 0.0);
        // A shared rare atom (id 300) outweighs a later shared common one
        // (id 1).
        let two = PrimeRoute::exact(2);
        let b = route_attention(&[300, 50, 1, 60, 8, 9, 1, 300], 7, two);
        assert!(b[1] + b[2] > 0.999, "{b:?}");
        assert!(b[2] > b[1]);
    }

    #[test]
    fn a_route_is_validated_and_excludes_a_selection() -> Result<()> {
        assert!(PrimeRoute::exact(0).validate().is_err());
        assert!(PrimeRoute::exact(MAX_ROUTE_WINDOW + 1).validate().is_err());
        let route = PrimeRoute::exact(2);
        let mut config = PointerConfig::new(4);
        config.route = Some(route);
        config.validate()?;
        config.select = Some(PointerSelect::TopK(1));
        assert!(config.validate().is_err());
        let mut model = pointer_model(ReadScore::Dot, Some(PointerSelect::TopK(2)), 0)?;
        assert!(model.set_pointer_route(Some(route)).is_err());
        model.set_pointer_select(None)?;
        model.set_pointer_route(Some(route))?;
        assert!(model
            .set_pointer_select(Some(PointerSelect::TopK(2)))
            .is_err());
        let mut plain = StackModel::new(
            tiny(StackArch::Geometric, "ar", ReadScore::Lorentz, true),
            &cpu(),
        )?;
        assert!(plain.set_pointer_route(Some(route)).is_err());
        plain.set_pointer_route(None)?;
        // The route round-trips through the saved configuration, and a
        // pointer without one has no key.
        let json = serde_json::to_string(&model.config)?;
        assert!(json.contains(r#""route":{"window":2}"#), "{json}");
        let back: StackConfig = serde_json::from_str(&json)?;
        assert_eq!(back.pointer.and_then(|p| p.route), Some(route));
        model.set_pointer_route(None)?;
        assert!(!serde_json::to_string(&model.config)?.contains("route"));
        Ok(())
    }

    /// Two windows of 12 in which a token recurs with different successors,
    /// so a row admits sources that hold its target and sources that do not
    /// (and a first occurrence admits none); the target is the next token
    /// (the window's first at its end), with [`pointer_batch`]'s weights.
    fn routed_batch() -> (Vec<u32>, Vec<u32>, Vec<f32>) {
        let ids: Vec<u32> = vec![
            1, 2, 1, 3, 1, 2, 4, 1, 3, 5, 1, 0, 7, 8, 7, 9, 7, 8, 6, 7, 9, 5, 7, 4,
        ];
        let targets: Vec<u32> = (0..24)
            .map(|n| {
                if n % 12 == 11 {
                    ids[n - 11]
                } else {
                    ids[n + 1]
                }
            })
            .collect();
        let (_, _, weights) = pointer_batch();
        (ids, targets, weights)
    }

    fn route_bits(weights: &[f64]) -> Vec<u64> {
        weights.iter().map(|w| w.to_bits()).collect()
    }

    #[test]
    fn a_ranked_route_is_exact_at_zero_scores_and_learned_where_it_admits_nothing() -> Result<()> {
        // At zero learned scores the ranked route is the exact one.
        let ids = [5u32, 10, 11, 50, 6, 7, 10, 11];
        let exact = route_attention(&ids, 7, PrimeRoute::exact(2));
        let ranked = ranked_attention(vec![0.0; 8], &ids, 7, PrimeRoute::ranked(2))?;
        assert_eq!(route_bits(&exact), route_bits(&ranked));
        // Learned scores re-rank sources that share the same atoms, and never
        // admit one the route does not.
        let one = PrimeRoute::ranked(1);
        let ids = [9u32, 1, 9, 2, 9];
        let plain = ranked_attention(vec![0.0; 5], &ids, 4, one)?;
        assert!(plain[3] > plain[1], "{plain:?}");
        let reranked = ranked_attention(vec![0.0, 2.0, 0.0, 0.0, 9.0], &ids, 4, one)?;
        assert!(reranked[1] > reranked[3], "{reranked:?}");
        assert_eq!(reranked[0] + reranked[2] + reranked[4], 0.0);
        // With none admitted it is the learned pointer over every source.
        let learned = vec![0.5f32, -1.0, 2.0, 0.0];
        let fallback = ranked_attention(learned.clone(), &[1, 2, 3, 4], 3, one)?;
        assert_eq!(
            route_bits(&fallback),
            route_bits(&pointer_weights(learned, None)?)
        );
        assert_eq!(
            parse_pointer_route("prime-ranked:3")?,
            Some(PrimeRoute::ranked(3))
        );
        assert_eq!(parse_pointer_route("prime:3")?, Some(PrimeRoute::exact(3)));
        assert!(parse_pointer_route("prime-ranked:0").is_err());
        // The flag is saved only when set, and a route saved without it is exact.
        assert_eq!(
            serde_json::to_string(&PrimeRoute::exact(2))?,
            r#"{"window":2}"#
        );
        let back: PrimeRoute = serde_json::from_str(r#"{"window":2}"#)?;
        assert_eq!(back, PrimeRoute::exact(2));
        Ok(())
    }

    #[test]
    fn an_ngram_route_admits_the_longest_ordered_match() -> Result<()> {
        let two = PrimeRoute::exact(2).admitting(RouteAdmission::Ngram);
        // "10 11" matches as a bigram: only its successor is admitted,
        // although the unigram 11 recurs elsewhere (a shared atom admits more).
        let ids = [5u32, 10, 11, 50, 6, 11, 60, 10, 11];
        let a = route_attention(&ids, 8, two);
        assert_eq!(a[3], 1.0, "{a:?}");
        let shared = route_attention(&ids, 8, PrimeRoute::exact(2));
        assert!(
            shared.iter().filter(|&&w| w > 0.0).count() > 1,
            "{shared:?}"
        );
        // No bigram match: back off to the unigram.
        let b = route_attention(&[7, 11, 60, 8, 11], 4, two);
        assert_eq!(b[2], 1.0, "{b:?}");
        // Order matters: "11 10" is not "10 11"; the unigram 10 matches.
        let c = route_attention(&[10, 11, 50, 11, 10], 4, two);
        assert_eq!(c[1], 1.0, "{c:?}");
        // Two matches of one n-let: recency orders them.
        let d = route_attention(&[4, 5, 6, 4, 5, 7, 4, 5], 7, two);
        assert!(d[5] > d[2] && d[2] > 0.0, "{d:?}");
        assert_eq!(d.iter().filter(|&&w| w > 0.0).count(), 2);
        // Nothing recurs: nothing admitted.
        assert!(route_attention(&[1, 2, 3], 2, two)
            .iter()
            .all(|&w| w == 0.0));
        // The grammar and the saved form.
        assert_eq!(parse_pointer_route("ngram:2")?, Some(two));
        assert_eq!(
            parse_pointer_route("ngram-ranked:3")?,
            Some(PrimeRoute::ranked(3).admitting(RouteAdmission::Ngram))
        );
        assert!(parse_pointer_route("ngram:0").is_err());
        let json = serde_json::to_string(&two)?;
        assert_eq!(json, r#"{"window":2,"admission":"ngram"}"#);
        assert_eq!(serde_json::from_str::<PrimeRoute>(&json)?, two);
        Ok(())
    }

    #[test]
    fn a_ranked_route_trains_the_scores_it_ranks_by() -> Result<()> {
        let (ids, targets, weights) = routed_batch();
        let data = (&ids[..], &targets[..], &weights[..]);
        let ngram = PrimeRoute::ranked(2).admitting(RouteAdmission::Ngram);
        for (score, route) in [
            (ReadScore::Dot, PrimeRoute::ranked(1)),
            (ReadScore::Lorentz, PrimeRoute::ranked(1)),
            (ReadScore::Dot, ngram),
            (ReadScore::Lorentz, ngram),
        ] {
            let mut model = pointer_model(score, None, 0)?;
            let soft = model.weighted_loss(&ids, &targets, &weights, 2, 12)?;
            model.set_pointer_route(Some(PrimeRoute::exact(1)))?;
            let exact = model.weighted_loss(&ids, &targets, &weights, 2, 12)?;
            model.set_pointer_route(Some(route))?;
            let hidden = model.hidden(&ids, 2, 12)?.to_vec2::<f32>()?;
            let (rows, _, contested) = reference_mixture(&model, &hidden, data, (2, 12))?;
            assert!(contested);
            let loss = model.weighted_loss(&ids, &targets, &weights, 2, 12)?;
            let got = f64::from(loss.to_scalar::<f32>()?);
            let want = weighted_mean(&rows, &weights);
            assert!(
                (got - want).abs() < 1e-5 * want.abs().max(1.0),
                "{score:?}: {got} against {want}"
            );
            for other in [&soft, &exact] {
                assert_ne!(
                    got.to_bits(),
                    f64::from(other.to_scalar::<f32>()?).to_bits()
                );
            }
            let scored = model.score_targets(&ids, &targets, Some(&weights), 2, 12)?;
            for n in 0..24 {
                assert!(
                    (scored.nll[n] - rows[n]).abs() < 1e-5 * rows[n].abs().max(1.0),
                    "{score:?}: row {n} scored {} against {}",
                    scored.nll[n],
                    rows[n]
                );
            }
            for t in (0..12).filter(|&t| weights[t] != 0.0) {
                let next = model.next_scores(&ids[..=t])?;
                let value = f64::from(next[targets[t] as usize]);
                assert!(
                    (value + rows[t]).abs() < 1e-4 * rows[t].abs().max(1.0),
                    "{score:?} position {t}: {value} against {}",
                    -rows[t]
                );
            }
            // Every pointer variable, the query and key included, gets the
            // reference's gradient.
            let grads = loss.backward()?;
            for name in pointer_names(score) {
                let numeric = reference_derivative(&model, name, &hidden, data, (2, 12))?;
                let analytic = grads
                    .get(model.variables()[name].as_tensor())
                    .ok_or_else(|| invalid("missing gradient"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let scale = numeric.iter().fold(0.0f64, |m, v| m.max(v.abs()));
                assert!(scale > 0.0, "{score:?}: {name} has no effect");
                for (i, (&a, &n)) in analytic.iter().zip(&numeric).enumerate() {
                    assert!(
                        (f64::from(a) - n).abs() < 2e-4 * scale + 2e-6,
                        "{score:?}: {name}[{i}] analytic {a} against numeric {n}"
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn a_routed_pointer_trains_its_gate_and_leaves_its_scores_alone() -> Result<()> {
        exact_route_trains_only_its_gate(PrimeRoute::exact(1))?;
        exact_route_trains_only_its_gate(PrimeRoute::exact(2).admitting(RouteAdmission::Ngram))
    }

    fn exact_route_trains_only_its_gate(route: PrimeRoute) -> Result<()> {
        let (ids, targets, weights) = routed_batch();
        let data = (&ids[..], &targets[..], &weights[..]);
        let soft = pointer_model(ReadScore::Dot, None, 0)?;
        let soft_loss = soft.weighted_loss(&ids, &targets, &weights, 2, 12)?;
        let mut model = pointer_model(ReadScore::Dot, None, 0)?;
        model.set_pointer_route(Some(route))?;
        // The loss and every scored row are the mixture with the route's
        // attention, and some row keeps sources with and without its target.
        let hidden = model.hidden(&ids, 2, 12)?.to_vec2::<f32>()?;
        let (rows, _, contested) = reference_mixture(&model, &hidden, data, (2, 12))?;
        assert!(contested);
        let loss = model.weighted_loss(&ids, &targets, &weights, 2, 12)?;
        let got = f64::from(loss.to_scalar::<f32>()?);
        let want = weighted_mean(&rows, &weights);
        assert!(
            (got - want).abs() < 1e-5 * want.abs().max(1.0),
            "{got} against {want}"
        );
        assert_ne!(
            got.to_bits(),
            f64::from(soft_loss.to_scalar::<f32>()?).to_bits()
        );
        let scored = model.score_targets(&ids, &targets, Some(&weights), 2, 12)?;
        for n in 0..24 {
            assert!(
                (scored.nll[n] - rows[n]).abs() < 1e-5 * rows[n].abs().max(1.0),
                "row {n} scored {} against {}",
                scored.nll[n],
                rows[n]
            );
        }
        // Generation mixes the same routed copy: the next-token score of the
        // target is minus the row's loss.
        for t in 0..12 {
            if weights[t] == 0.0 {
                continue;
            }
            let next = model.next_scores(&ids[..=t])?;
            let score = f64::from(next[targets[t] as usize]);
            assert!(
                (score + rows[t]).abs() < 1e-4 * rows[t].abs().max(1.0),
                "position {t}: next score {score} against {}",
                -rows[t]
            );
        }
        // The gate's gradient is the reference's; the unused query and key
        // move nothing and get none.
        let grads = loss.backward()?;
        for name in ["pointer.gate.weight", "pointer.gate.bias"] {
            let numeric = reference_derivative(&model, name, &hidden, data, (2, 12))?;
            let analytic = grads
                .get(model.variables()[name].as_tensor())
                .ok_or_else(|| invalid("missing gradient"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            let scale = numeric.iter().fold(0.0f64, |m, v| m.max(v.abs()));
            assert!(scale > 0.0, "{name} has no effect on the routed loss");
            for (i, (&a, &n)) in analytic.iter().zip(&numeric).enumerate() {
                assert!(
                    (f64::from(a) - n).abs() < 2e-4 * scale + 2e-6,
                    "{name}[{i}] analytic {a} against numeric {n}"
                );
            }
        }
        for name in ["pointer.query.weight", "pointer.key.weight"] {
            let numeric = reference_derivative(&model, name, &hidden, data, (2, 12))?;
            assert!(
                numeric.iter().all(|&n| n.abs() < 1e-12),
                "{name} moves the loss"
            );
            if let Some(grad) = grads.get(model.variables()[name].as_tensor()) {
                assert!(
                    grad.flatten_all()?
                        .to_vec1::<f32>()?
                        .iter()
                        .all(|&g| g == 0.0),
                    "{name} has a gradient under the route"
                );
            }
        }
        // Clearing the route restores the learned scores bit for bit.
        model.set_pointer_route(None)?;
        let cleared = model.weighted_loss(&ids, &targets, &weights, 2, 12)?;
        assert_eq!(
            cleared.to_scalar::<f32>()?.to_bits(),
            soft_loss.to_scalar::<f32>()?.to_bits()
        );
        Ok(())
    }

    #[test]
    fn a_row_without_copy_mass_is_exactly_the_generated_probability() -> Result<()> {
        // A vocabulary of 4, one window of 3 positions and a pointer of width 2.
        let (vocabulary, time, dim) = (4usize, 3usize, 2usize);
        let stride = 2 * dim + 1;
        let ids = vec![1u32, 2, 1];
        // Token 3 is held by no source, so p_copy is 0 on every row.
        let targets = vec![3u32; time];
        // Under the plain softmax the target is 60 nats below the rest: the
        // generated probability is about 3e-27, far below a 1e-8 floor.
        let logit_rows: Vec<f32> = (0..time).flat_map(|_| [0.0f32, 0.0, 0.0, -60.0]).collect();
        let gate_logit = 0.5f32;
        let side_values: Vec<f32> = (0..time)
            .flat_map(|n| [0.3 + n as f32, -0.2, 0.5, 0.1 * n as f32, gate_logit])
            .collect();
        let op = || PointerMixture {
            time,
            dim,
            score: ReadScore::Dot,
            select: None,
            route: None,
            ids: ids.clone(),
            targets: targets.clone(),
            weights: None,
        };
        let g = 1.0 / (1.0 + (-f64::from(gate_logit)).exp());
        let lse = 3.0f64.ln();
        let want = -(1.0 - g).ln() - (-60.0 - lse);
        for n in 0..time {
            let row = op().evaluate(
                &logit_rows[n * vocabulary..(n + 1) * vocabulary],
                &side_values,
                0.0,
                n,
            )?;
            assert_eq!(row.copy, 0.0, "row {n}");
            assert_eq!(row.copy_share, 0.0, "row {n}");
            assert_eq!(row.generate_share, 1.0, "row {n}");
            assert!(
                (-row.log_mixture - want).abs() < 1e-9,
                "row {n}: {} against {want}",
                -row.log_mixture
            );
        }
        // Through the autodiff op: the loss is that row, the gate logit's
        // gradient is g / 3 (the mean over three rows) and nothing reaches the
        // copy branch (the query and key).
        let logits = Var::from_vec(logit_rows.clone(), (time, vocabulary), &cpu())?;
        let side = Var::from_vec(side_values.clone(), (time, stride), &cpu())?;
        let beta = Var::from_vec(vec![0.0f32], 1, &cpu())?;
        let loss = logits
            .as_tensor()
            .apply_op3(side.as_tensor(), beta.as_tensor(), op())?;
        let value = f64::from(loss.to_scalar::<f32>()?);
        assert!((value - want).abs() < 1e-4, "{value} against {want}");
        let grads = loss.backward()?;
        let d_side = grads
            .get(side.as_tensor())
            .ok_or_else(|| invalid("no side gradient"))?
            .to_vec2::<f32>()?;
        for (n, row) in d_side.iter().enumerate() {
            assert!(
                (f64::from(row[2 * dim]) - g / 3.0).abs() < 1e-7,
                "row {n}: gate gradient {} against {}",
                row[2 * dim],
                g / 3.0
            );
            assert!(
                row[..2 * dim].iter().all(|&v| v == 0.0),
                "row {n}: the copy branch received gradient"
            );
        }
        Ok(())
    }

    #[test]
    fn the_mixture_backward_stays_finite_past_the_f64_exponent_range() -> Result<()> {
        // g / mixture overflows f64 once a row's NLL passes about 709.78 nats.
        // Rows that no source serves, 747 nats down: the scores and the scale
        // get exactly no gradient, the gate its `g` (the mean over 3 rows).
        let (vocabulary, time, dim) = (4usize, 3usize, 2usize);
        let stride = 2 * dim + 1;
        let ids = vec![1u32, 2, 1];
        let targets = vec![3u32; time];
        let logit_rows: Vec<f32> = (0..time).flat_map(|_| [0.0f32, 0.0, 0.0, -745.0]).collect();
        let gate_logit = 0.5f32;
        let side_values: Vec<f32> = (0..time)
            .flat_map(|n| [0.3 + n as f32, -0.2, 0.5, 0.1 * n as f32, gate_logit])
            .collect();
        let g = 1.0 / (1.0 + (-f64::from(gate_logit)).exp());
        for score in [ReadScore::Dot, ReadScore::Lorentz] {
            let op = PointerMixture {
                time,
                dim,
                score,
                select: None,
                route: None,
                ids: ids.clone(),
                targets: targets.clone(),
                weights: None,
            };
            let logits = Var::from_vec(logit_rows.clone(), (time, vocabulary), &cpu())?;
            let side = Var::from_vec(side_values.clone(), (time, stride), &cpu())?;
            let beta = Var::from_vec(vec![1.0f32], 1, &cpu())?;
            let loss = logits
                .as_tensor()
                .apply_op3(side.as_tensor(), beta.as_tensor(), op)?;
            let value = f64::from(loss.to_scalar::<f32>()?);
            assert!(
                value.is_finite() && value > 709.78,
                "{score:?}: loss {value}"
            );
            let grads = loss.backward()?;
            let d_side = grads
                .get(side.as_tensor())
                .ok_or_else(|| invalid("no side gradient"))?
                .to_vec2::<f32>()?;
            for (n, row) in d_side.iter().enumerate() {
                assert!(
                    (f64::from(row[2 * dim]) - g / 3.0).abs() < 1e-7,
                    "{score:?} row {n}: gate gradient {}",
                    row[2 * dim]
                );
                assert!(
                    row[..2 * dim].iter().all(|&v| v == 0.0),
                    "{score:?} row {n}: the query or key received {row:?}"
                );
            }
            let d_beta = grads
                .get(beta.as_tensor())
                .ok_or_else(|| invalid("no scale gradient"))?
                .to_vec1::<f32>()?;
            assert_eq!(d_beta, vec![0.0], "{score:?}");
            let d_logits = grads
                .get(logits.as_tensor())
                .ok_or_else(|| invalid("no logit gradient"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(d_logits.iter().all(|v| v.is_finite()), "{score:?}");
        }
        // A copy mass below f64's normal range at a moderate NLL: row 1's query
        // scores source 0, which holds the target, 713 below source 1, so
        // source 0's attention is about 2e-310.
        let ids = vec![3u32, 1];
        let targets = vec![3u32, 3];
        let side_values = vec![1.0f32, -713.0, gate_logit, 1.0, 0.0, gate_logit];
        let op = || PointerMixture {
            time: 2,
            dim: 1,
            score: ReadScore::Dot,
            select: None,
            route: None,
            ids: ids.clone(),
            targets: targets.clone(),
            weights: None,
        };
        let row = op().evaluate(&[0.0; 4], &side_values, 1.0, 1)?;
        assert!(
            row.copy > 0.0 && row.copy < f64::MIN_POSITIVE,
            "copy mass {}",
            row.copy
        );
        let logits = Var::from_vec(vec![0.0f32; 2 * vocabulary], (2, vocabulary), &cpu())?;
        let side = Var::from_vec(side_values, (2, 3), &cpu())?;
        let beta = Var::from_vec(vec![1.0f32], 1, &cpu())?;
        let loss = logits
            .as_tensor()
            .apply_op3(side.as_tensor(), beta.as_tensor(), op())?;
        let grads = loss.backward()?;
        for (name, var) in [("logits", &logits), ("side", &side), ("beta", &beta)] {
            let values = grads
                .get(var.as_tensor())
                .ok_or_else(|| invalid(format!("no {name} gradient")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(values.iter().all(|v| v.is_finite()), "{name}: {values:?}");
        }
        Ok(())
    }

    #[test]
    fn a_gate_forced_to_zero_leaves_the_plain_loss() -> Result<()> {
        for score in [ReadScore::Dot, ReadScore::Lorentz] {
            let model = pointer_model(score, None, 0)?;
            model.variables()["pointer.gate.weight"].set(&Tensor::zeros(
                (1, 16),
                DType::F32,
                &cpu(),
            )?)?;
            model.variables()["pointer.gate.bias"].set(&Tensor::from_vec(
                vec![-60.0f32],
                1,
                &cpu(),
            )?)?;
            let (ids, targets, weights) = pointer_batch();
            let mixture = model
                .weighted_loss(&ids, &targets, &weights, 2, 12)?
                .to_scalar::<f32>()?;
            let plain =
                logits_cross_entropy(&model.forward(&ids, 2, 12)?, &targets, Some(&weights))?
                    .to_scalar::<f32>()?;
            assert!(
                (mixture - plain).abs() < 1e-5,
                "{score:?}: {mixture} against {plain}"
            );
            // And the scored rows read the plain NLL too.
            let scores = model.score_targets(&ids, &targets, Some(&weights), 2, 12)?;
            assert!(scores.pointer.is_some());
            let plain_rows = model.target_nll(&ids, &targets, 2, 12)?;
            assert_eq!(plain_rows.len(), 24);
            for (n, (&row, &weight)) in scores.nll.iter().zip(&weights).enumerate() {
                if weight == 0.0 {
                    assert_eq!(row, 0.0);
                } else {
                    assert!((row - plain_rows[n]).abs() < 1e-5, "row {n}");
                }
            }
        }
        Ok(())
    }

    /// A toy pointer model whose residual stream is the token's normalized
    /// embedding (3 I, so h = 4 e_v) because its reads and MLPs add nothing.
    /// The pointer's key is 2 I, so the key of a source holding token `v` is
    /// `8 e_v`; its query maps token 5 to `4 * query_scale` times the direction
    /// of token 9 and every other token to 0; its gate weight is 0 and its bias
    /// `gate_bias`.
    fn toy_pointer(pointer: PointerConfig, query_scale: f32, gate_bias: f32) -> Result<StackModel> {
        let mut config = tiny(StackArch::Geometric, "a", ReadScore::Dot, false);
        config.vocab_size = 16;
        config.pointer = Some(pointer);
        let toy = StackModel::new(config, &cpu())?;
        for name in ["layers.00.read.out.weight", "layers.00.mlp.down.weight"] {
            let var = &toy.variables()[name];
            var.set(&Tensor::zeros(var.dims(), DType::F32, &cpu())?)?;
        }
        let matrix = |entry: &dyn Fn(usize, usize) -> f32| -> Result<Tensor> {
            let values: Vec<f32> = (0..16 * 16).map(|i| entry(i / 16, i % 16)).collect();
            Ok(Tensor::from_vec(values, (16, 16), &cpu())?)
        };
        toy.variables()["embedding.weight"].set(&matrix(&|r, c| {
            if r == c {
                3.0
            } else {
                0.0
            }
        })?)?;
        toy.variables()["pointer.key.weight"].set(&matrix(&|r, c| {
            if r == c {
                2.0
            } else {
                0.0
            }
        })?)?;
        toy.variables()["pointer.query.weight"].set(&matrix(&|r, c| {
            if r == 9 && c == 5 {
                query_scale
            } else {
                0.0
            }
        })?)?;
        toy.variables()["pointer.gate.weight"].set(&Tensor::zeros((1, 16), DType::F32, &cpu())?)?;
        toy.variables()["pointer.gate.bias"].set(&Tensor::from_vec(vec![gate_bias], 1, &cpu())?)?;
        Ok(toy)
    }

    fn argmax(scores: &[f32]) -> usize {
        let mut best = 0;
        for (i, v) in scores.iter().enumerate() {
            if *v > scores[best] {
                best = i;
            }
        }
        best
    }

    #[test]
    fn generation_agrees_with_the_loss_and_a_forced_gate_copies_the_attended_token() -> Result<()> {
        // The mixture the loss scores at position t is the one generation
        // scores after the prefix, for each score and selection of the pointer.
        for (score, select) in [
            (
                ReadScore::Dot,
                Some(PointerSelect::Flock(FlockSelect {
                    sink: 0,
                    window: 3,
                    k: 1,
                })),
            ),
            (ReadScore::Lorentz, Some(PointerSelect::TopK(2))),
            (ReadScore::Lorentz, None),
        ] {
            let model = pointer_model(score, select, 0)?;
            let (ids, targets, _) = pointer_batch();
            let scores = model.score_targets(&ids, &targets, None, 2, 12)?;
            for t in [0usize, 4, 9, 11] {
                let next = model.next_scores(&ids[..=t])?;
                let sum: f64 = next.iter().map(|&s| f64::from(s).exp()).sum();
                assert!(
                    (sum - 1.0).abs() < 1e-4,
                    "{score:?} {select:?} position {t}: mixture sums to {sum}"
                );
                let nll = -f64::from(next[targets[t] as usize]);
                assert!(
                    (nll - scores.nll[t]).abs() < 1e-4,
                    "{score:?} {select:?} position {t}: {nll} against {}",
                    scores.nll[t]
                );
            }
        }
        // The toy's query maps token 5 to the direction of token 9 and its key
        // is the identity, so a query at token 5 attends the positions holding
        // 9; its raw logits favour the current token.
        let history = [1u32, 9, 2, 5];
        for score in [ReadScore::Dot, ReadScore::Lorentz] {
            let pointer = PointerConfig {
                score,
                ..PointerConfig::new(16)
            };
            // Gate near 1: the attended token (9) is copied; the raw logits
            // favour the current token (5).
            let toy = toy_pointer(pointer, 2.0, 30.0)?;
            assert_eq!(argmax(&toy.next_scores(&history)?), 9, "{score:?}");
            assert_eq!(
                argmax(&toy.forward(&history, 1, 4)?.get(3)?.to_vec1::<f32>()?),
                5
            );
            // Gate near 0: the plain distribution.
            let toy = toy_pointer(pointer, 2.0, -30.0)?;
            assert_eq!(argmax(&toy.next_scores(&history)?), 5, "{score:?}");
        }
        Ok(())
    }

    #[test]
    fn a_top_one_pointer_puts_all_its_copy_mass_on_the_argmax_source() -> Result<()> {
        // A soft pointer (query scale 1/4) spreads its attention; the gate is
        // 1/2. Sources hold tokens 1, 9, 2 and 5, and the argmax holds 9.
        let history = [1u32, 9, 2, 5];
        for score in [ReadScore::Dot, ReadScore::Lorentz] {
            let pointer = PointerConfig {
                score,
                ..PointerConfig::new(16)
            };
            let mut toy = toy_pointer(pointer, 0.25, 0.0)?;
            let logits = toy.forward(&history, 1, 4)?.get(3)?.to_vec1::<f32>()?;
            let peak = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let lse = f64::from(peak)
                + logits
                    .iter()
                    .map(|&z| f64::from(z - peak).exp())
                    .sum::<f64>()
                    .ln();
            // The generated part of the mixture: (1 - g) softmax(z)[v].
            let generated = |v: usize| 0.5 * (f64::from(logits[v]) - lse).exp();
            let probability = |scores: &[f32], v: usize| f64::from(scores[v]).exp();
            let soft = toy.next_scores(&history)?;
            toy.set_pointer_select(Some(PointerSelect::TopK(1)))?;
            let hard = toy.next_scores(&history)?;
            for v in [1usize, 2, 5] {
                // The sources that are not the argmax receive no copy mass.
                assert!(
                    (probability(&hard, v) - generated(v)).abs() <= 1e-4 * generated(v),
                    "{score:?}: token {v}: {} against {}",
                    probability(&hard, v),
                    generated(v)
                );
                // The soft pointer gives them some.
                assert!(
                    probability(&soft, v) > generated(v) + 0.01,
                    "{score:?}: token {v}: soft {} against {}",
                    probability(&soft, v),
                    generated(v)
                );
            }
            // The argmax source takes the whole copy branch: g * 1.
            assert!(
                (probability(&hard, 9) - (generated(9) + 0.5)).abs() < 1e-6,
                "{score:?}: token 9: {} against {}",
                probability(&hard, 9),
                generated(9) + 0.5
            );
            assert!(
                probability(&soft, 9) < generated(9) + 0.5 - 0.01,
                "{score:?}: soft {} kept the whole copy branch",
                probability(&soft, 9)
            );
        }
        Ok(())
    }

    #[test]
    fn the_reads_flock_never_applies_to_the_pointer() -> Result<()> {
        // The toy's reads add nothing to the residual stream, so its final
        // states do not depend on the flock; only the pointer could feel it.
        let history = [1u32, 9, 2, 5];
        let plain = toy_pointer(PointerConfig::new(16), 0.25, 0.0)?;
        let mut flocked = toy_pointer(PointerConfig::new(16), 0.25, 0.0)?;
        flocked.set_select(Some(FlockSelect {
            sink: 0,
            window: 1,
            k: 1,
        }))?;
        assert_eq!(
            flocked.next_scores(&history)?,
            plain.next_scores(&history)?,
            "the reads' flock changed the pointer's attention"
        );
        // The two selections are separate settings of one model.
        assert!(flocked.config.select.is_some());
        assert_eq!(flocked.config.pointer.and_then(|p| p.select), None);
        // The pointer's own selection is not the reads' either.
        let mut own = toy_pointer(PointerConfig::new(16), 0.25, 0.0)?;
        own.set_pointer_select(Some(PointerSelect::TopK(1)))?;
        assert_eq!(own.config.select, None);
        assert_ne!(own.next_scores(&history)?, plain.next_scores(&history)?);
        Ok(())
    }

    #[test]
    fn a_post_hoc_pointer_selection_changes_generation_and_not_the_weights() -> Result<()> {
        let mut model = pointer_model(ReadScore::Lorentz, None, 0)?;
        let (ids, targets, weights) = pointer_batch();
        let weight_bits = |model: &StackModel| -> Result<Vec<Vec<u32>>> {
            model
                .variables()
                .values()
                .map(|var| bits(var.as_tensor()))
                .collect()
        };
        let loss_bits = |model: &StackModel| -> Result<u32> {
            Ok(model
                .weighted_loss(&ids, &targets, &weights, 2, 12)?
                .to_scalar::<f32>()?
                .to_bits())
        };
        let before = weight_bits(&model)?;
        let soft = model.next_scores(&ids[..9])?;
        let soft_loss = loss_bits(&model)?;
        model.set_pointer_select(Some(PointerSelect::TopK(1)))?;
        let hard = model.next_scores(&ids[..9])?;
        assert!(hard != soft, "the selection changed nothing");
        assert_eq!(weight_bits(&model)?, before, "the weights changed");
        // The same weights with the selection built in give the same model.
        let twin = pointer_model(ReadScore::Lorentz, Some(PointerSelect::TopK(1)), 0)?;
        assert_eq!(model.config, twin.config);
        assert_eq!(loss_bits(&model)?, loss_bits(&twin)?);
        assert_eq!(hard, twin.next_scores(&ids[..9])?);
        // Clearing it restores the soft pointer bit for bit.
        model.set_pointer_select(None)?;
        assert_eq!(model.next_scores(&ids[..9])?, soft);
        assert_eq!(loss_bits(&model)?, soft_loss);
        // A refused selection leaves the model as it was.
        assert!(model
            .set_pointer_select(Some(PointerSelect::TopK(0)))
            .is_err());
        assert!(model
            .set_pointer_select(Some(PointerSelect::Flock(FlockSelect {
                sink: 0,
                window: 0,
                k: 1,
            })))
            .is_err());
        assert_eq!(model.config.pointer.and_then(|p| p.select), None);
        // Without a head there is nothing to select for.
        let mut plain = StackModel::new(
            tiny(StackArch::Geometric, "ar", ReadScore::Lorentz, true),
            &cpu(),
        )?;
        assert!(plain
            .set_pointer_select(Some(PointerSelect::TopK(1)))
            .is_err());
        assert!(plain.set_pointer_select(None).is_ok());
        Ok(())
    }

    #[test]
    fn an_old_pointer_config_deserializes_to_a_dot_pointer_without_selection() -> Result<()> {
        // A configuration saved before the pointer had a score, a selection or
        // a recorded seed.
        let old = r#"{"arch":"geometric","vocab_size":37,"width":16,"heads":2,
            "mlp_hidden":24,"context":12,"pattern":"ar","read":"lorentz",
            "rotation":true,"seed":5,"pointer":{"dim":4}}"#;
        let config: StackConfig = serde_json::from_str(old)?;
        config.validate()?;
        let pointer = config
            .pointer
            .ok_or_else(|| invalid("the old config lost its pointer"))?;
        assert_eq!(pointer, PointerConfig::new(4));
        assert_eq!(pointer.score, ReadScore::Dot);
        assert_eq!(pointer.select, None);
        assert_eq!(pointer.init_seed, None);
        assert_eq!(config.select, None);
        // Its tensors are the old four: no scale for a Dot pointer.
        let names: Vec<String> = config
            .shapes()
            .keys()
            .filter(|name| name.starts_with("pointer."))
            .cloned()
            .collect();
        assert_eq!(
            names,
            [
                "pointer.gate.bias",
                "pointer.gate.weight",
                "pointer.key.weight",
                "pointer.query.weight"
            ]
        );
        // And it serializes as it was: the defaults are not written.
        let json = serde_json::to_string(&config)?;
        assert!(json.contains(r#""pointer":{"dim":4}"#), "{json}");
        // A configuration that uses the new fields round-trips through JSON.
        let mut new = config.clone();
        new.select = Some(FlockSelect {
            sink: 0,
            window: 3,
            k: 2,
        });
        new.pointer = Some(PointerConfig {
            dim: 4,
            score: ReadScore::Lorentz,
            select: Some(PointerSelect::TopK(1)),
            init_seed: Some(7),
            route: None,
        });
        let json = serde_json::to_string(&new)?;
        assert!(json.contains(r#""score":"lorentz""#), "{json}");
        assert!(json.contains(r#""select":{"top_k":1}"#), "{json}");
        assert!(json.contains(r#""init_seed":7"#), "{json}");
        assert_eq!(serde_json::from_str::<StackConfig>(&json)?, new);
        assert!(new.shapes().contains_key("pointer.log_beta"));
        Ok(())
    }

    #[test]
    fn selection_parameters_the_reads_cannot_evaluate_are_refused() -> Result<()> {
        let mut config = tiny(StackArch::Geometric, "ar", ReadScore::Lorentz, true);
        for bad in [
            FlockSelect {
                sink: 1,
                window: 3,
                k: 2,
            },
            FlockSelect {
                sink: 0,
                window: 0,
                k: 2,
            },
            FlockSelect {
                sink: 0,
                window: 3,
                k: 0,
            },
        ] {
            config.select = Some(bad);
            assert!(config.validate().is_err(), "{bad:?}");
            assert!(StackModel::new(config.clone(), &cpu()).is_err(), "{bad:?}");
        }
        config.select = None;
        for bad in [
            PointerSelect::TopK(0),
            PointerSelect::Flock(FlockSelect {
                sink: 0,
                window: 0,
                k: 1,
            }),
        ] {
            config.pointer = Some(PointerConfig {
                select: Some(bad),
                ..PointerConfig::new(4)
            });
            assert!(config.validate().is_err(), "{bad:?}");
        }
        config.pointer = Some(PointerConfig::new(0));
        assert!(config.validate().is_err());
        // The command-line grammar of both selections.
        assert_eq!(parse_flock_select("none")?, None);
        assert_eq!(
            parse_flock_select("flock:64:7")?,
            Some(FlockSelect {
                sink: 0,
                window: 64,
                k: 7
            })
        );
        for bad in ["flock:0:7", "flock:4:0", "flock:4", "top:1", "flock:a:1"] {
            assert!(parse_flock_select(bad).is_err(), "{bad}");
        }
        assert_eq!(parse_pointer_select("none")?, None);
        assert_eq!(parse_pointer_select("top:1")?, Some(PointerSelect::TopK(1)));
        assert_eq!(
            parse_pointer_select("flock:32:16")?,
            Some(PointerSelect::Flock(FlockSelect {
                sink: 0,
                window: 32,
                k: 16
            }))
        );
        for bad in ["top:0", "top", "top:x", "flock:0:1", "flock:1", "window:3"] {
            assert!(parse_pointer_select(bad).is_err(), "{bad}");
        }
        Ok(())
    }

    #[test]
    fn a_pointer_model_saves_loads_and_gains_a_head_from_a_seed() -> Result<()> {
        let select = Some(PointerSelect::Flock(FlockSelect {
            sink: 0,
            window: 3,
            k: 2,
        }));
        for score in [ReadScore::Dot, ReadScore::Lorentz] {
            let mut model = pointer_model(score, select, 0)?;
            model.config.select = Some(FlockSelect {
                sink: 0,
                window: 4,
                k: 2,
            });
            let directory = std::env::temp_dir().join(format!(
                "geometric-stack-pointer-{}-{score:?}",
                std::process::id()
            ));
            model.save(&directory)?;
            let loaded = StackModel::load(&directory, &cpu())?;
            let config_json = fs::read_to_string(directory.join("config.json"))?;
            assert!(config_json.contains("\"pointer\"") && config_json.contains("\"select\""));
            assert_eq!(
                loaded.variables().contains_key("pointer.log_beta"),
                score == ReadScore::Lorentz
            );
            assert_eq!(loaded.config, model.config);
            let (ids, targets, weights) = pointer_batch();
            assert_eq!(
                model
                    .weighted_loss(&ids, &targets, &weights, 2, 12)?
                    .to_scalar::<f32>()?
                    .to_bits(),
                loaded
                    .weighted_loss(&ids, &targets, &weights, 2, 12)?
                    .to_scalar::<f32>()?
                    .to_bits()
            );
            assert_eq!(
                model.next_scores(&ids[..8])?,
                loaded.next_scores(&ids[..8])?
            );
            fs::remove_dir_all(&directory)?;
        }
        // A model saved without a pointer gains one from a seed, with the
        // weights a new model of that configuration and seed starts with, and
        // records the seed.
        let plain = StackModel::new(
            tiny(StackArch::Geometric, "ar", ReadScore::Lorentz, true),
            &cpu(),
        )?;
        let directory =
            std::env::temp_dir().join(format!("geometric-stack-plain-{}", std::process::id()));
        plain.save(&directory)?;
        let mut grown = StackModel::load(&directory, &cpu())?;
        fs::remove_dir_all(&directory)?;
        let seed = grown.config.seed;
        assert!(grown.add_pointer(PointerConfig::new(4), seed)?);
        assert_eq!(
            grown.config.pointer.and_then(|pointer| pointer.init_seed),
            Some(seed)
        );
        // The same head again is a no-op; another width or score is refused.
        assert!(!grown.add_pointer(PointerConfig::new(4), seed + 1)?);
        assert_eq!(
            grown.config.pointer.and_then(|pointer| pointer.init_seed),
            Some(seed),
            "an existing head keeps the seed it was drawn from"
        );
        assert!(grown.add_pointer(PointerConfig::new(8), seed).is_err());
        assert!(grown
            .add_pointer(
                PointerConfig {
                    score: ReadScore::Lorentz,
                    ..PointerConfig::new(4)
                },
                seed
            )
            .is_err());
        let mut config = plain.config.clone();
        config.pointer = Some(PointerConfig {
            init_seed: Some(seed),
            ..PointerConfig::new(4)
        });
        let fresh = StackModel::new(config, &cpu())?;
        assert_eq!(grown.config, fresh.config);
        assert_eq!(grown.parameter_count(), fresh.parameter_count());
        for (name, var) in fresh.variables() {
            if name.starts_with("pointer.") {
                assert_eq!(
                    bits(var.as_tensor())?,
                    bits(grown.variables()[name].as_tensor())?,
                    "{name}"
                );
            }
        }
        // The old parameters are untouched and the gate starts at sigmoid(-2).
        assert_eq!(
            bits(plain.variables()["embedding.weight"].as_tensor())?,
            bits(fresh.variables()["embedding.weight"].as_tensor())?
        );
        assert_eq!(
            grown.variables()["pointer.gate.bias"]
                .as_tensor()
                .to_vec1::<f32>()?,
            vec![-2.0]
        );
        // A Lorentz head starts with a zero log scale, and shares its query
        // and key with the Dot head of the same seed.
        let mut lorentz = StackModel::new(
            tiny(StackArch::Geometric, "ar", ReadScore::Lorentz, true),
            &cpu(),
        )?;
        assert!(lorentz.add_pointer(
            PointerConfig {
                score: ReadScore::Lorentz,
                ..PointerConfig::new(4)
            },
            seed
        )?);
        assert_eq!(
            lorentz.variables()["pointer.log_beta"]
                .as_tensor()
                .to_vec1::<f32>()?,
            vec![0.0]
        );
        for name in ["pointer.query.weight", "pointer.key.weight"] {
            assert_eq!(
                bits(lorentz.variables()[name].as_tensor())?,
                bits(fresh.variables()[name].as_tensor())?,
                "{name}"
            );
        }
        Ok(())
    }

    #[test]
    fn a_pointer_has_no_served_representation_and_no_export() -> Result<()> {
        let mut config = exportable("rar", ReadScore::Lorentz, true);
        for score in [ReadScore::Dot, ReadScore::Lorentz] {
            config.pointer = Some(PointerConfig {
                score,
                ..PointerConfig::new(8)
            });
            let mut model = StackModel::new(config.clone(), &cpu())?;
            let refusal = model
                .set_served_representation(Some(Arc::new(D11Interim)))
                .expect_err("qat with a pointer is refused");
            assert!(refusal.to_string().contains("pointer"), "{refusal}");
            let refusal =
                crate::stack_export::export_stack(&model, serde_json::json!({}), None, None)
                    .expect_err("a pointer model is not exported");
            assert!(refusal.to_string().contains("no D11 port"), "{refusal}");
        }
        // A flock is refused by the exports too, but trains in QAT.
        config.pointer = None;
        config.select = Some(FlockSelect {
            sink: 0,
            window: 4,
            k: 2,
        });
        let mut model = StackModel::new(config, &cpu())?;
        let refusal = crate::stack_export::export_stack(&model, serde_json::json!({}), None, None)
            .expect_err("a flock model is not exported");
        assert!(refusal.to_string().contains("flock"), "{refusal}");
        model.set_served_representation(Some(Arc::new(D11Interim)))?;
        Ok(())
    }
}
