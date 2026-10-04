//! The stack model (validated, repacked at load) and its incremental decoding
//! session. `IntegerStackSession::step` and the mixers it calls are the served
//! numerical path: they use the kernels of [`super::kernels`], running offsets
//! for every index, and buffers allocated when the session is created.
//!
//! A pointer-copy head (`pointer` in the shape) reads the final normalized
//! state beside the output head: a query `q_t` and a key `k_t` of its width
//! (the key cached per position) and a gate logit `g_t`, then
//! [`stack_pointer`] forms the served next-token distribution, the D10
//! engine's integers:
//!
//! - scores `s_j` of the sources `j = 0..=t` at exponent -16: Dot
//!   `round(<q_t, k_j> c 2^-46)` for the Q30 scale `c`, or Lorentz
//!   `-beta d(q_t, k_j)` (the read's distance kernels, no offset);
//! - `w_j = exp(s_j - max s)`, `u_v = exp(z_v - max z)` (Q31, sealed exp
//!   table), `T_c = sum_j w_j`, `T_g = sum_v u_v` and the copy mass
//!   `m_v = sum_{j : x_j = v} w_j` over the input tokens `x_j`;
//! - `C = sigmoid(g_t)` (Q31, exp table and long division), `A = 2^31 - C`;
//! - `G = round(A floor(2^93 / T_g) 2^-31)`, `K = round(C floor(2^93 / T_c)
//!   2^-31)` (long division and table products) and
//!   `p_v = round((u_v G + m_v K) 2^-63)` ([`stack_pointer_mixture`]): the
//!   mixture `(1 - sigmoid(g)) softmax(z)_v + sigmoid(g) p_copy(v)` in Q30.
//!
//! `step` still returns the logits; a pointer model's distribution is
//! [`IntegerStackSession::mixture`] and greedy decoding takes the argmax of
//! [`IntegerStackSession::next_token_scores`].

use std::path::{Path, PathBuf};

use super::format::{
    Container, Fixed, MatrixView, StackNumerics, StackPointer, StackShape, StackTransportSnap,
};
use super::kernels::{
    grid_apply, grid_valid, mul_i128, shift, shift_wide, stack_activation, stack_activation_tables,
    stack_dequant_row, stack_div_u128, stack_dot, stack_exp_neg, stack_gemv, stack_gemv_pairs,
    stack_hamilton, stack_isqrt, stack_l2_distance, stack_lift, stack_lorentz_distance,
    stack_mix_row, stack_mul_u128, stack_mul_u64, stack_pair_tables, stack_pointer_mixture,
    stack_quantize16, stack_query_tables, stack_rms_norm, stack_sigmoid_q31, stack_snap_rotation,
    stack_snap_select, stack_square, PackedMatrix, ARCOSH_TABLE_LEN, PRODUCT_EXP, RESIDUAL_EXP,
};
use super::StackError;

/// Exponent of the recurrence state.
const STATE_EXP: i32 = -32;
/// Exponent of read scores, in nats.
const SCORE_EXP: i32 = -16;
/// Exponent of Lorentz and L2 distances and offsets (the arcosh table's output
/// for Lorentz).
const DISTANCE_EXP: i32 = -24;
/// Exponent of decay gates, decays and transition quaternions (Q31).
const GATE_EXP: i32 = -31;
/// Taps of the causal convolution.
pub const CONVOLUTION_WIDTH: usize = 4;
/// Training floors `1 - lambda^2 >= 1e-6` (Q62) and `keep = 1e-3` below it (Q31).
const COMPLEMENT_FLOOR_Q62: u64 = 4_611_686_018_428;
const KEEP_FLOOR_Q31: u64 = 2_147_484;
/// The rotation norm's epsilon, `1e-6`, at exponent -32.
const ROTATION_EPSILON: u128 = 4_295;
/// Exponents (matrix bases, the RMSNorm epsilon) are limited to this magnitude
/// so that no exponent arithmetic can overflow.
const EXPONENT_LIMIT: i32 = 1 << 10;

struct Recurrence {
    /// `2 width x width`: the drive, then the output gate.
    input: PackedMatrix,
    /// `gate_rows x width`: decay gates, then rotations.
    gates: PackedMatrix,
    out: PackedMatrix,
    /// Grid codes `[tap][channel]`; tap `s` weights the drive `s` positions back.
    taps: Vec<i16>,
    conv_bias: Vec<i32>,
    gate_bias: Vec<i32>,
    /// Grid codes of `8 softplus(-decay)` per lane.
    rates: Vec<i16>,
}

struct Read {
    query: PackedMatrix,
    key: PackedMatrix,
    value: PackedMatrix,
    null: PackedMatrix,
    out: PackedMatrix,
    null_bias: Vec<i32>,
    /// `[head][distance]` at the score exponent.
    age: Vec<i32>,
    /// Lorentz or L2 scale per head (grid codes) and offset (exponent -24).
    beta: Vec<i16>,
    offset: Vec<i32>,
}

/// The pointer-copy head (see the module documentation).
struct Pointer {
    dim: usize,
    lorentz: bool,
    score_scale_q30: i64,
    query: PackedMatrix,
    key: PackedMatrix,
    /// One row: the gate logit.
    gate: PackedMatrix,
    /// At exponent -16.
    gate_bias: i32,
    /// Lorentz scale (grid code); zero for Dot.
    beta: i16,
}

enum Mixer {
    Recurrence(Box<Recurrence>),
    Read(Box<Read>),
}

struct Layer {
    mixer: Mixer,
    gate: PackedMatrix,
    up: PackedMatrix,
    down: PackedMatrix,
}

/// A validated stack artifact repacked for multiplier-free integer serving.
pub struct IntegerStackModel {
    shape: StackShape,
    numerics: StackNumerics,
    /// Derived at load so that a step never divides: `width / heads`.
    head_dim: usize,
    lorentz: bool,
    /// The flat L2 read `-beta (|q - k| - offset)`: scaled like Lorentz, but
    /// with no lifts and no arcosh table.
    l2: bool,
    /// The artifact records the icosian transport snap: the recurrence
    /// transports by the nearest root instead of the free unit quaternion.
    snapped: bool,
    transport_snap: Option<StackTransportSnap>,
    sha256: String,
    embed: PackedMatrix,
    head: PackedMatrix,
    pointer: Option<Box<Pointer>>,
    /// Boxed so that the per-layer walk steps by a power-of-two stride: an
    /// unboxed walk indexes 304-byte elements with a multiply (`madd`).
    #[allow(clippy::vec_box)]
    layers: Vec<Box<Layer>>,
    exp_table: Vec<u32>,
    silu_table: Vec<i32>,
    gelu_table: Vec<i32>,
    arcosh: Vec<u32>,
    weights_per_token: u64,
}

fn packed(view: MatrixView<'_>, name: &str) -> Result<PackedMatrix, StackError> {
    if !(-EXPONENT_LIMIT..=EXPONENT_LIMIT).contains(&view.exp_base) {
        return Err(StackError::Numerics(format!(
            "matrix {name} has an exponent base outside ±{EXPONENT_LIMIT}"
        )));
    }
    let groups = view.cols / super::format::GROUP;
    let min_de = view
        .scales
        .chunks_exact(groups)
        .map(|row| row.iter().map(|s| s >> 4).min().unwrap_or(0))
        .collect();
    Ok(PackedMatrix {
        rows: view.rows,
        cols: view.cols,
        exp_base: view.exp_base,
        nibbles: view.nibbles.to_vec(),
        scales: view.scales.to_vec(),
        min_de,
    })
}

impl IntegerStackModel {
    pub fn load(path: &Path) -> Result<Self, StackError> {
        Self::parse(&std::fs::read(path).map_err(StackError::Io)?)
    }

    /// Parse and validate a `UORLUT01` stack artifact. The checks are those of
    /// the D10 engine's loader (section bounds, matrix shapes, table lengths,
    /// grid codes, the numerics ranges and the arcosh table's order), plus
    /// three that keep every served operation exact: unique section names,
    /// bounded exponents and a non-increasing exp table. The shape's read
    /// caches are bounded too (`MAX_READ_CACHE_BYTES`, 4 GiB), so that
    /// creating a session cannot abort on allocation.
    pub fn parse(bytes: &[u8]) -> Result<Self, StackError> {
        let artifact = Container::parse(bytes)?;
        let shape = artifact.shape.clone();
        let numerics = artifact.numerics.clone();
        let (d, heads, mlp) = (shape.width, shape.heads, shape.mlp);
        let mut weights = 0u64;
        let mut matrix =
            |name: &str, rows: usize, cols: usize| -> Result<PackedMatrix, StackError> {
                let packed = packed(artifact.matrix(name, rows, cols)?, name)?;
                weights += (rows as u64) * (cols as u64);
                Ok(packed)
            };
        let integers = |name: &str, len: usize| -> Result<Vec<i32>, StackError> {
            let values = artifact.table_i32(name)?;
            if values.len() != len {
                return Err(StackError::TableLength {
                    name: name.to_owned(),
                    len: values.len(),
                    expected: len,
                });
            }
            Ok(values)
        };
        // Grid codes of positive scalars (rates and scales) or of any sign (taps).
        let codes = |name: &str, len: usize, positive: bool| -> Result<Vec<i16>, StackError> {
            let values = artifact.table_i16(name)?;
            if values.len() != len {
                return Err(StackError::TableLength {
                    name: name.to_owned(),
                    len: values.len(),
                    expected: len,
                });
            }
            if values
                .iter()
                .any(|&c| !grid_valid(c) || (positive && c <= 0))
            {
                return Err(StackError::GridCodes(name.to_owned()));
            }
            Ok(values)
        };
        let context_ages = heads
            .checked_mul(shape.context)
            .ok_or(StackError::Shape("the age table size overflows"))?;
        let mut layers = Vec::with_capacity(shape.layers());
        for (l, kind) in shape.pattern.bytes().enumerate() {
            let name = |part: &str| format!("l{l}.{part}");
            let mixer = if kind == b'r' {
                Mixer::Recurrence(Box::new(Recurrence {
                    input: matrix(&name("rec_in"), 2 * d, d)?,
                    gates: matrix(&name("rec_gate"), shape.gate_rows(), d)?,
                    out: matrix(&name("rec_out"), d, d)?,
                    taps: codes(&name("conv_taps"), CONVOLUTION_WIDTH * d, false)?,
                    conv_bias: integers(&name("conv_bias"), d)?,
                    gate_bias: integers(&name("gate_bias"), shape.gate_rows())?,
                    rates: codes(&name("decay_rate"), shape.lanes(), true)?,
                }))
            } else {
                let scaled = shape.scaled();
                Mixer::Read(Box::new(Read {
                    query: matrix(&name("query"), d, d)?,
                    key: matrix(&name("key"), d, d)?,
                    value: matrix(&name("value"), d, d)?,
                    null: matrix(&name("null"), heads, d)?,
                    out: matrix(&name("out"), d, d)?,
                    null_bias: integers(&name("null_bias"), heads)?,
                    age: integers(&name("age"), context_ages)?,
                    beta: if scaled {
                        codes(&name("beta"), heads, true)?
                    } else {
                        Vec::new()
                    },
                    offset: if scaled {
                        integers(&name("offset"), heads)?
                    } else {
                        Vec::new()
                    },
                }))
            };
            layers.push(Box::new(Layer {
                mixer,
                gate: matrix(&name("gate"), mlp, d)?,
                up: matrix(&name("up"), mlp, d)?,
                down: matrix(&name("down"), d, mlp)?,
            }));
        }
        let embed = packed(artifact.matrix("embed", shape.vocab, d)?, "embed")?;
        let head = matrix("head", shape.vocab, d)?;
        let pointer = match &shape.pointer {
            Some(p) => Some(Box::new(Pointer {
                dim: p.dim,
                lorentz: p.lorentz(),
                score_scale_q30: p.score_scale_q30,
                query: matrix("pointer_query", p.dim, d)?,
                key: matrix("pointer_key", p.dim, d)?,
                gate: matrix("pointer_gate", 1, d)?,
                gate_bias: integers("pointer_gate_bias", 1)?[0],
                beta: if p.lorentz() {
                    codes("pointer_beta", 1, true)?[0]
                } else {
                    0
                },
            })),
            None => None,
        };
        // The embedding reads one row per token.
        weights += d as u64;
        let exp_table = artifact.table_u32("exp")?;
        let silu_table = artifact.table_i32("silu")?;
        let gelu_table = artifact.table_i32("gelu")?;
        let arcosh = if shape.needs_arcosh() {
            let table = artifact.table_u32("arcosh")?;
            if table.len() != ARCOSH_TABLE_LEN || table.windows(2).any(|w| w[0] > w[1]) {
                return Err(StackError::Numerics(
                    "the arcosh table has the wrong size or order".to_owned(),
                ));
            }
            table
        } else {
            Vec::new()
        };
        let n = &numerics;
        let steps_valid = (-16..0).contains(&n.exp_step_log2)
            && (-16..0).contains(&n.silu_step_log2)
            && (-16..0).contains(&n.gelu_step_log2)
            && (0..=8).contains(&n.silu_range_log2)
            && (0..=8).contains(&n.gelu_range_log2);
        let activation_len = |step: i32, range: i32| (2usize << (range - step)) + 1;
        if exp_table.len() < 2
            || !steps_valid
            || silu_table.len() != activation_len(n.silu_step_log2, n.silu_range_log2)
            || gelu_table.len() != activation_len(n.gelu_step_log2, n.gelu_range_log2)
            || !(1..=1i64 << 31).contains(&n.score_scale_q30)
        {
            return Err(StackError::Numerics(
                "sealed tables do not match the declared numerics".to_owned(),
            ));
        }
        if exp_table.windows(2).any(|w| w[0] < w[1]) {
            return Err(StackError::Numerics("the exp table increases".to_owned()));
        }
        if !(-EXPONENT_LIMIT..=EXPONENT_LIMIT).contains(&n.rms_eps.exp) {
            return Err(StackError::Numerics(format!(
                "the RMSNorm epsilon exponent is outside ±{EXPONENT_LIMIT}"
            )));
        }
        Ok(Self {
            head_dim: shape.head_dim(),
            lorentz: shape.lorentz(),
            l2: shape.l2(),
            snapped: artifact.transport_snap.is_some(),
            transport_snap: artifact.transport_snap.clone(),
            shape,
            numerics,
            sha256: artifact.sha256,
            embed,
            head,
            pointer,
            layers,
            exp_table,
            silu_table,
            gelu_table,
            arcosh,
            weights_per_token: weights,
        })
    }

    pub fn shape(&self) -> &StackShape {
        &self.shape
    }

    /// The integer conventions declared by the artifact.
    pub fn numerics(&self) -> &StackNumerics {
        &self.numerics
    }

    /// The RMSNorm epsilon.
    pub fn rms_epsilon(&self) -> Fixed {
        self.numerics.rms_eps
    }

    /// SHA-256 of the artifact bytes.
    pub fn artifact_sha256(&self) -> &str {
        &self.sha256
    }

    /// The transport snap the artifact records (the served recurrence snaps
    /// every unit transport quaternion to the nearest of these roots before
    /// its scaling by lambda), or `None` for the free transport.
    pub fn transport_snap(&self) -> Option<&StackTransportSnap> {
        self.transport_snap.as_ref()
    }

    /// The pointer head's width, score and scale (`None` without one).
    pub fn pointer(&self) -> Option<&StackPointer> {
        self.shape.pointer.as_ref()
    }

    /// Learned 4-bit weights read per token: every weight map in full plus one
    /// embedding row. The engine's access is dense (an interim stepping stone
    /// under ROADMAP R3), and this is its per-token parameter read count.
    pub fn weights_per_token(&self) -> u64 {
        self.weights_per_token
    }

    /// A fresh session: zero recurrence states and empty read caches. Every
    /// buffer a step uses is allocated here, for the full context, within the
    /// bounds the loader checked (the shape's limits and read-cache bound).
    pub fn session(&self) -> IntegerStackSession<'_> {
        let s = &self.shape;
        let (d, context) = (s.width, s.context);
        let states = s
            .pattern
            .bytes()
            .map(|kind| {
                Box::new(if kind == b'r' {
                    LayerState::Recurrence {
                        state: vec![0; d],
                        history: vec![0; (CONVOLUTION_WIDTH - 1) * d],
                    }
                } else {
                    LayerState::Read {
                        keys: vec![0; context * d],
                        values: vec![0; context * d],
                        lifts: if self.lorentz {
                            vec![0; context * s.heads]
                        } else {
                            Vec::new()
                        },
                    }
                })
            })
            .collect();
        let widest = d.max(s.mlp);
        // Without a pointer head every pointer buffer is empty.
        let (pointer_dim, pointer_lorentz) = self
            .pointer
            .as_ref()
            .map_or((0, false), |p| (p.dim, p.lorentz));
        let pointer_vocab = if self.pointer.is_some() { s.vocab } else { 0 };
        IntegerStackSession {
            model: self,
            position: 0,
            cache_at: 0,
            lift_at: 0,
            pointer_at: 0,
            copy_scale_q16: 0,
            tokens: Vec::with_capacity(context),
            pointer_keys: vec![0; pointer_dim * context],
            pointer_lifts: if pointer_lorentz {
                vec![0; context]
            } else {
                Vec::new()
            },
            states,
            b: Buffers {
                x: vec![0; d],
                norm: vec![0; d],
                act: vec![0; widest],
                tables: vec![[0; 16]; widest],
                pairs: vec![[0; 256]; d >> 1],
                scratch: vec![0; widest],
                wide: vec![0; d],
                branches: vec![0; 2 * d],
                gate_out: vec![0; s.gate_rows()],
                q: vec![0; d],
                k: vec![0; d],
                v: vec![0; d],
                null: vec![0; s.heads],
                query_tables: vec![[0; 16]; self.head_dim],
                scores: vec![0; context],
                weights: vec![0; context],
                pointer_weights: vec![0; context],
                mix: vec![0; self.head_dim],
                proj: vec![0; d],
                gate: vec![0; s.mlp],
                up: vec![0; s.mlp],
                logits: vec![0; s.vocab],
                pointer_query: vec![0; pointer_dim],
                pointer_key: vec![0; pointer_dim],
                pointer_query_tables: vec![[0; 16]; pointer_dim],
                pointer_gate: vec![0; usize::from(self.pointer.is_some())],
                generate: vec![0; pointer_vocab],
                copy: vec![0; pointer_vocab],
                mixture: vec![0; pointer_vocab],
                snap_trace: None,
            },
        }
    }
}

enum LayerState {
    Recurrence {
        /// `[width]` at exponent -32.
        state: Vec<i64>,
        /// The drive 1, 2 and 3 positions back, `[3][width]`, exponent -16.
        history: Vec<i32>,
    },
    Read {
        /// `[position][width]` at exponent -16, for the whole context.
        keys: Vec<i32>,
        values: Vec<i32>,
        /// Lorentz key lifts `[position][head]` at exponent -32.
        lifts: Vec<u64>,
    },
}

/// Canonical schema identifier for serialized integer stack session state snapshots.
pub const STACK_SESSION_SCHEMA: &str = "uor-r4.stack-session/1";

/// Serialized state of a single layer within an [`IntegerStackSession`].
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SerializedStackLayerState {
    /// Recurrence layer state: internal state vector and causal convolution history.
    Recurrence {
        /// Recurrence state vector `[width]` at exponent -32.
        state: Vec<i64>,
        /// Causal convolution history `[(CONVOLUTION_WIDTH - 1) * width]` at exponent -16.
        history: Vec<i32>,
    },
    /// Read layer cache: keys, values, and optional Lorentz lifts up to the active context position.
    Read {
        /// Populated key cache entries `[position * width]` at exponent -16.
        keys: Vec<i32>,
        /// Populated value cache entries `[position * width]` at exponent -16.
        values: Vec<i32>,
        /// Populated Lorentz key lifts `[position * heads]` at exponent -32 (empty for dot read).
        lifts: Vec<u64>,
    },
}

/// Durable, reloadable snapshot of an [`IntegerStackSession`].
///
/// Binds strictly to the model's `artifact_sha256`, context bounds, and layer dimensions,
/// enabling exact bit-identical state recovery across persistent sessions.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SerializedStackSession {
    /// Schema identifier, strictly matching [`STACK_SESSION_SCHEMA`].
    pub schema: String,
    /// Schema version (must be 1).
    pub version: u32,
    /// SHA-256 of the model artifact this session was created from.
    pub artifact_sha256: String,
    /// Current sequence position (number of tokens ingested / stepped).
    pub position: u64,
    /// Keys/values cache length (`position * width`).
    pub cache_at: u64,
    /// Lifts cache length (`position * heads`).
    pub lift_at: u64,
    /// Scale for direct pointer copy from attention weights to output logits, in Q16.
    #[serde(default)]
    pub copy_scale_q16: i32,
    /// Pending next-token logits produced by the last step (length equals vocab when position > 0).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub logits: Vec<i32>,
    /// Optional sequence of tokens ingested so far (`[position]` entries).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<u32>,
    /// Optional opt-in snap trace root indices recorded so far.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snap_trace: Option<Vec<u32>>,
    /// A pointer model's cached pointer keys `[position * dim]` (exponent -16).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pointer_keys: Vec<i32>,
    /// A Lorentz pointer's cached key lifts `[position]` (exponent -32).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pointer_lifts: Vec<u64>,
    /// A pointer model's pending mixture (Q30) from the last step (length
    /// equals vocab when position > 0).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mixture: Vec<i32>,
    /// Per-layer state snapshots.
    pub layers: Vec<SerializedStackLayerState>,
}

/// One entry of the opt-in snap trace ([`IntegerStackSession::enable_snap_trace`]):
/// the root index the snapped transport selected at (layer, position, lane).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapTraceEntry {
    pub layer: usize,
    pub position: usize,
    pub lane: usize,
    pub root: usize,
}

/// The trace buffer: one root index per (position, recurrence layer, lane),
/// in step order. Writes are plain stores — no allocation in a step.
struct SnapTrace {
    origin_position: usize,
    selected: Vec<u32>,
    written: usize,
}

impl SnapTrace {
    #[inline(always)]
    fn record(&mut self, index: usize) {
        if let Some(slot) = self.selected.get_mut(self.written) {
            *slot = index as u32;
        }
        self.written += 1;
    }
}

struct Buffers {
    x: Vec<i32>,
    norm: Vec<i16>,
    act: Vec<i16>,
    tables: Vec<[i32; 16]>,
    /// Pair tables of the normalized state, read by every map with it as input.
    pairs: Vec<[i32; 256]>,
    scratch: Vec<i64>,
    wide: Vec<i64>,
    branches: Vec<i32>,
    gate_out: Vec<i32>,
    q: Vec<i32>,
    k: Vec<i32>,
    v: Vec<i32>,
    null: Vec<i32>,
    query_tables: Vec<[i64; 16]>,
    scores: Vec<i64>,
    weights: Vec<u64>,
    mix: Vec<i128>,
    proj: Vec<i32>,
    gate: Vec<i32>,
    up: Vec<i32>,
    logits: Vec<i32>,
    /// Pointer copy weights for previous positions, normalized to Q31.
    pointer_weights: Vec<u64>,
    /// The pointer head's query, key and gate logit of this position.
    pointer_query: Vec<i32>,
    pointer_key: Vec<i32>,
    pointer_query_tables: Vec<[i64; 16]>,
    pointer_gate: Vec<i32>,
    /// `u_v` (Q31) per vocabulary id.
    generate: Vec<u64>,
    /// `m_v` (Q31 sums) per vocabulary id; all zero between steps.
    copy: Vec<u64>,
    /// The pointer mixture in Q30 per vocabulary id.
    mixture: Vec<i32>,
    /// The opt-in snap trace; `None` (the default) records nothing.
    snap_trace: Option<SnapTrace>,
}

/// Incremental decoding of one stream of at most `context` positions.
pub struct IntegerStackSession<'m> {
    model: &'m IntegerStackModel,
    position: usize,
    /// `position * width`: where this position's key and value rows go.
    cache_at: usize,
    /// `position * heads`: where this position's Lorentz key lifts go.
    lift_at: usize,
    /// `position * pointer dim`: where this position's pointer key goes.
    pointer_at: usize,
    /// Scale for direct pointer copy from attention weights to output logits, in Q16.
    copy_scale_q16: i32,
    /// Sequence of tokens stepped in this session so far.
    tokens: Vec<u32>,
    /// The pointer head's keys `[position][dim]` (exponent -16) and Lorentz
    /// key lifts `[position]` (exponent -32), for the whole context.
    pointer_keys: Vec<i32>,
    pointer_lifts: Vec<u64>,
    /// Boxed, like the model's layers, for a power-of-two stride.
    #[allow(clippy::vec_box)]
    states: Vec<Box<LayerState>>,
    b: Buffers,
}

/// Maximum justified serialized JSON size for a session of the given model shape.
/// Derived using checked arithmetic over model shape dimensions, allocating up to 24 bytes
/// per numerical element to soundly bound compact JSON (where i64/u64 with punctuation can take
/// up to 21 bytes) produced by `save_session_to_file`.
pub(super) fn max_serialized_session_bytes(model: &IntegerStackModel) -> u64 {
    let s = &model.shape;
    let d = s.width as u64;
    let ctx = s.context as u64;
    let heads = s.heads as u64;
    let vocab = s.vocab as u64;
    let rec_layers = s.pattern.bytes().filter(|&b| b == b'r').count() as u64;

    let layer_bytes: u64 = s
        .pattern
        .bytes()
        .map(|c| {
            if c == b'a' {
                let lifts = if model.lorentz {
                    ctx.checked_mul(heads).unwrap_or(u64::MAX / 4)
                } else {
                    0
                };
                (2 * ctx * d + lifts)
                    .checked_mul(24)
                    .unwrap_or(u64::MAX / 2)
            } else {
                (CONVOLUTION_WIDTH as u64 * d)
                    .checked_mul(24)
                    .unwrap_or(u64::MAX / 2)
            }
        })
        .sum();

    // A pointer model's keys, lifts and pending mixture.
    let pointer_bytes = model.pointer.as_ref().map_or(0, |p| {
        let lifts = if p.lorentz { ctx } else { 0 };
        ctx.checked_mul(p.dim as u64)
            .and_then(|keys| keys.checked_add(lifts))
            .and_then(|x| x.checked_add(vocab))
            .and_then(|x| x.checked_mul(24))
            .unwrap_or(u64::MAX / 4)
    });
    let logits_bytes = vocab
        .checked_mul(24)
        .unwrap_or(1024 * 1024)
        .saturating_add(pointer_bytes);
    let tokens_bytes = ctx.checked_mul(24).unwrap_or(1024 * 1024);
    let trace_bytes = if model.snapped {
        ctx.checked_mul(s.lanes() as u64)
            .and_then(|x| x.checked_mul(rec_layers))
            .and_then(|x| x.checked_mul(24))
            .unwrap_or(1024 * 1024)
    } else {
        0
    };
    let margin = 131072u64; // 128 KiB for schema, metadata keys, and formatting punctuation

    layer_bytes
        .checked_add(logits_bytes)
        .and_then(|acc| acc.checked_add(tokens_bytes))
        .and_then(|acc| acc.checked_add(trace_bytes))
        .and_then(|acc| acc.checked_add(margin))
        .unwrap_or(100 * 1024 * 1024)
}

impl IntegerStackSession<'_> {
    pub fn position(&self) -> usize {
        self.position
    }

    /// Enable the diagnostic snap trace (default off): each step records the
    /// root index the snapped transport selected at every (recurrence layer,
    /// lane), so [`Self::snap_trace`] reads them back per (layer, position,
    /// lane). Tracing allocates its buffer here, once (context x recurrence
    /// layers x lanes entries); it is a diagnostic only and does not change
    /// the served arithmetic. On a model without a snap nothing is recorded.
    pub fn enable_snap_trace(&mut self) {
        let s = &self.model.shape;
        let recurrences = s.pattern.bytes().filter(|&c| c == b'r').count();
        let entries = s.context * recurrences * s.lanes();
        self.b.snap_trace = Some(SnapTrace {
            origin_position: self.position,
            selected: vec![u32::MAX; entries],
            written: 0,
        });
    }

    /// The recorded snap trace as (layer, position, lane, root) entries in
    /// step order, if tracing is enabled. Allocates; diagnostic only, never
    /// called by a step.
    pub fn snap_trace(&self) -> Option<Vec<SnapTraceEntry>> {
        let trace = self.b.snap_trace.as_ref()?;
        let s = &self.model.shape;
        let lanes = s.lanes();
        let recurrences: Vec<usize> = s
            .pattern
            .bytes()
            .enumerate()
            .filter(|(_, c)| *c == b'r')
            .map(|(layer, _)| layer)
            .collect();
        let per_position = recurrences.len() * lanes;
        if per_position == 0 {
            return Some(Vec::new());
        }
        let written = trace.written.min(trace.selected.len());
        let entries = (0..written)
            .map(|slot| {
                let (rel_pos, rem) = (slot / per_position, slot % per_position);
                SnapTraceEntry {
                    layer: recurrences[rem / lanes],
                    position: trace.origin_position + rel_pos,
                    lane: rem % lanes,
                    root: trace.selected[slot] as usize,
                }
            })
            .collect();
        Some(entries)
    }

    /// Sequence of tokens stepped in this session so far.
    pub fn tokens(&self) -> &[u32] {
        &self.tokens
    }

    /// Set the pointer copy scale in Q16 (`scale * 2^-16`).
    ///
    /// When greater than zero, normalized attention weights from head 0 of the final
    /// read layer directly boost the output logits of previously stepped prefix tokens
    /// via integer shift-add operations (`shift_wide(mul_i128(pw, scale), 31)`).
    ///
    /// This operates as a dense soft-attention token-identity boost: every historical
    /// token position contributes according to its soft attention mass without discrete
    /// argmax/top-k selection. It provides an integer-serving precursor scaffold for
    /// learned copy policies (such as the AERM `copy_boost` in `stack_aerm.rs`).
    ///
    /// When zero (the default), attention weight capture is bypassed in `stack_read` to
    /// ensure zero overhead for default serving.
    pub fn set_copy_scale(&mut self, scale_q16: i32) {
        self.copy_scale_q16 = scale_q16.max(0);
        if self.copy_scale_q16 == 0 {
            self.b.pointer_weights.fill(0);
        }
    }

    /// The active pointer copy scale in Q16.
    pub fn copy_scale(&self) -> i32 {
        self.copy_scale_q16
    }

    /// Normalized Q31 attention weights from head 0 of the final read layer across historical positions.
    ///
    /// Valid only immediately after a [`Self::step`] call where [`Self::copy_scale`] > 0.
    /// When copy scale is 0 (the default), capture is bypassed and this buffer holds zeros.
    /// After [`Self::restore_state`] or [`Self::reset`], this buffer is zeroed.
    pub fn pointer_weights(&self) -> &[u64] {
        &self.b.pointer_weights[..self.position]
    }

    /// Start a new stream.
    pub fn reset(&mut self) {
        self.position = 0;
        self.cache_at = 0;
        self.lift_at = 0;
        self.pointer_at = 0;
        self.tokens.clear();
        self.pointer_keys.fill(0);
        self.pointer_lifts.fill(0);
        self.b.logits.fill(0);
        self.b.mixture.fill(0);
        self.b.pointer_weights.fill(0);
        if let Some(trace) = &mut self.b.snap_trace {
            let s = &self.model.shape;
            let recurrences = s.pattern.bytes().filter(|&c| c == b'r').count();
            let full_entries = s.context * recurrences * s.lanes();
            trace.origin_position = 0;
            trace.written = 0;
            if trace.selected.len() < full_entries {
                trace.selected.resize(full_entries, u32::MAX);
            }
            trace.selected.fill(u32::MAX);
        }
        for state in &mut self.states {
            match state.as_mut() {
                LayerState::Recurrence { state, history } => {
                    state.fill(0);
                    history.fill(0);
                }
                LayerState::Read {
                    keys,
                    values,
                    lifts,
                } => {
                    keys.fill(0);
                    values.fill(0);
                    lifts.fill(0);
                }
            }
        }
    }

    /// The active keys/values cache length (`position * width`).
    pub fn cache_at(&self) -> usize {
        self.cache_at
    }

    /// The active Lorentz lifts cache length (`position * heads`).
    pub fn lift_at(&self) -> usize {
        self.lift_at
    }

    /// Snapshot the active session state into a [`SerializedStackSession`].
    ///
    /// Preserves exact recurrence vectors, causal convolution histories,
    /// populated read caches up to the current position (`cache_at` / `lift_at`),
    /// pending next-token logits from the last step, ingested tokens, and opt-in snap trace.
    pub fn save_state(&self) -> SerializedStackSession {
        let layers = self
            .states
            .iter()
            .map(|st| match st.as_ref() {
                LayerState::Recurrence { state, history } => {
                    SerializedStackLayerState::Recurrence {
                        state: state.clone(),
                        history: history.clone(),
                    }
                }
                LayerState::Read {
                    keys,
                    values,
                    lifts,
                } => SerializedStackLayerState::Read {
                    keys: keys[..self.cache_at].to_vec(),
                    values: values[..self.cache_at].to_vec(),
                    lifts: if self.model.lorentz {
                        lifts[..self.lift_at].to_vec()
                    } else {
                        Vec::new()
                    },
                },
            })
            .collect();

        let logits = if self.position > 0 {
            self.b.logits.clone()
        } else {
            Vec::new()
        };
        let (pointer_keys, pointer_lifts, mixture) = match &self.model.pointer {
            Some(p) => (
                self.pointer_keys[..self.pointer_at].to_vec(),
                if p.lorentz {
                    self.pointer_lifts[..self.position].to_vec()
                } else {
                    Vec::new()
                },
                if self.position > 0 {
                    self.b.mixture.clone()
                } else {
                    Vec::new()
                },
            ),
            None => (Vec::new(), Vec::new(), Vec::new()),
        };

        let snap_trace = if self.model.snapped {
            let rec_layers = self
                .model
                .shape
                .pattern
                .bytes()
                .filter(|&b| b == b'r')
                .count();
            let expected_len = self.position * self.model.shape.lanes() * rec_layers;
            self.b.snap_trace.as_ref().and_then(|t| {
                if t.origin_position == 0
                    && t.written == expected_len
                    && expected_len <= t.selected.len()
                {
                    Some(t.selected[..expected_len].to_vec())
                } else {
                    None
                }
            })
        } else {
            None
        };

        SerializedStackSession {
            schema: STACK_SESSION_SCHEMA.to_string(),
            version: 1,
            artifact_sha256: self.model.sha256.clone(),
            position: self.position as u64,
            cache_at: self.cache_at as u64,
            lift_at: self.lift_at as u64,
            copy_scale_q16: self.copy_scale_q16,
            logits,
            tokens: self.tokens.clone(),
            snap_trace,
            pointer_keys,
            pointer_lifts,
            mixture,
            layers,
        }
    }

    /// Restore session state from a [`SerializedStackSession`].
    ///
    /// Validates schema, version, artifact SHA-256, context bounds, layer dimensions,
    /// pending logits, ingested tokens, and snap trace before applying state. Designed
    /// so that after restoration, subsequent step calls produce bit-for-bit identical
    /// outputs to continuing the original session.
    pub fn restore_state(&mut self, saved: &SerializedStackSession) -> Result<(), StackError> {
        if saved.schema != STACK_SESSION_SCHEMA {
            return Err(StackError::Schema(format!(
                "expected {STACK_SESSION_SCHEMA}, found {}",
                saved.schema
            )));
        }
        if saved.version != 1 {
            return Err(StackError::Schema(format!(
                "unsupported session version {}",
                saved.version
            )));
        }
        if saved.artifact_sha256 != self.model.sha256 {
            return Err(StackError::Numerics(format!(
                "session artifact SHA-256 mismatch: session has '{}', model has '{}'",
                saved.artifact_sha256, self.model.sha256
            )));
        }
        let position = usize::try_from(saved.position).map_err(|_| StackError::SessionState)?;
        let cache_at = usize::try_from(saved.cache_at).map_err(|_| StackError::SessionState)?;
        let lift_at = usize::try_from(saved.lift_at).map_err(|_| StackError::SessionState)?;
        if position > self.model.shape.context {
            return Err(StackError::ContextFull {
                context: self.model.shape.context,
            });
        }
        let d = self.model.shape.width;
        let heads = self.model.shape.heads;
        let expected_cache_at = position.checked_mul(d).ok_or(StackError::SessionState)?;
        let expected_lift_at = position
            .checked_mul(heads)
            .ok_or(StackError::SessionState)?;
        if cache_at != expected_cache_at || lift_at != expected_lift_at {
            return Err(StackError::SessionState);
        }
        if saved.layers.len() != self.states.len() {
            return Err(StackError::SessionState);
        }

        // Validate pending logits
        if position == 0 {
            if !saved.logits.is_empty() {
                return Err(StackError::SessionState);
            }
        } else if saved.logits.len() != self.model.shape.vocab {
            return Err(StackError::SessionState);
        }

        // Validate tokens
        if saved.tokens.len() != position {
            return Err(StackError::SessionState);
        }

        // Validate the pointer head's state
        let pointer_at = match &self.model.pointer {
            Some(p) => {
                let pointer_at = position
                    .checked_mul(p.dim)
                    .ok_or(StackError::SessionState)?;
                let lifts = if p.lorentz { position } else { 0 };
                let mixture = if position > 0 {
                    self.model.shape.vocab
                } else {
                    0
                };
                if saved.pointer_keys.len() != pointer_at
                    || saved.pointer_lifts.len() != lifts
                    || saved.mixture.len() != mixture
                    || saved.mixture.iter().any(|&v| v < 0)
                {
                    return Err(StackError::SessionState);
                }
                pointer_at
            }
            None => {
                if !saved.pointer_keys.is_empty()
                    || !saved.pointer_lifts.is_empty()
                    || !saved.mixture.is_empty()
                {
                    return Err(StackError::SessionState);
                }
                0
            }
        };
        for &tok in &saved.tokens {
            if tok as usize >= self.model.shape.vocab {
                return Err(StackError::Token {
                    token: tok,
                    vocab: self.model.shape.vocab,
                });
            }
        }

        // Validate snap trace if present
        let rec_layers = self
            .model
            .shape
            .pattern
            .bytes()
            .filter(|&b| b == b'r')
            .count();
        let expected_snap_entries = position
            .checked_mul(self.model.shape.lanes())
            .and_then(|x| x.checked_mul(rec_layers))
            .ok_or(StackError::SessionState)?;
        if let Some(ref trace_entries) = saved.snap_trace {
            if !self.model.snapped {
                if !trace_entries.is_empty() {
                    return Err(StackError::SessionState);
                }
            } else {
                if trace_entries.len() != expected_snap_entries {
                    return Err(StackError::SessionState);
                }
                for &root in trace_entries {
                    if root >= 120 {
                        return Err(StackError::SessionState);
                    }
                }
            }
        }

        // Validate all layers before mutating session
        for (saved_layer, state) in saved.layers.iter().zip(&self.states) {
            match (saved_layer, state.as_ref()) {
                (
                    SerializedStackLayerState::Recurrence {
                        state: s,
                        history: h,
                    },
                    LayerState::Recurrence { .. },
                ) => {
                    if s.len() != d || h.len() != (CONVOLUTION_WIDTH - 1) * d {
                        return Err(StackError::SessionState);
                    }
                }
                (
                    SerializedStackLayerState::Read {
                        keys: k,
                        values: v,
                        lifts: l,
                    },
                    LayerState::Read { .. },
                ) => {
                    if k.len() != cache_at || v.len() != cache_at {
                        return Err(StackError::SessionState);
                    }
                    if self.model.lorentz {
                        if l.len() != lift_at {
                            return Err(StackError::SessionState);
                        }
                    } else if !l.is_empty() {
                        return Err(StackError::SessionState);
                    }
                }
                _ => return Err(StackError::SessionState),
            }
        }

        // Apply state
        for (saved_layer, state) in saved.layers.iter().zip(self.states.iter_mut()) {
            match (saved_layer, state.as_mut()) {
                (
                    SerializedStackLayerState::Recurrence {
                        state: s,
                        history: h,
                    },
                    LayerState::Recurrence {
                        state: cur_s,
                        history: cur_h,
                    },
                ) => {
                    cur_s.copy_from_slice(s);
                    cur_h.copy_from_slice(h);
                }
                (
                    SerializedStackLayerState::Read {
                        keys: k,
                        values: v,
                        lifts: l,
                    },
                    LayerState::Read {
                        keys: cur_k,
                        values: cur_v,
                        lifts: cur_l,
                    },
                ) => {
                    cur_k[..cache_at].copy_from_slice(k);
                    cur_v[..cache_at].copy_from_slice(v);
                    if self.model.lorentz {
                        cur_l[..lift_at].copy_from_slice(l);
                    }
                }
                _ => unreachable!(),
            }
        }

        self.position = position;
        self.cache_at = cache_at;
        self.lift_at = lift_at;
        self.pointer_at = pointer_at;
        self.pointer_keys.fill(0);
        self.pointer_lifts.fill(0);
        self.pointer_keys[..pointer_at].copy_from_slice(&saved.pointer_keys);
        self.pointer_lifts[..saved.pointer_lifts.len()].copy_from_slice(&saved.pointer_lifts);
        self.b.mixture.fill(0);
        self.b.mixture[..saved.mixture.len()].copy_from_slice(&saved.mixture);
        self.copy_scale_q16 = saved.copy_scale_q16;
        self.b.pointer_weights.fill(0);

        // Restore pending logits
        if position > 0 {
            self.b.logits.copy_from_slice(&saved.logits);
        } else {
            self.b.logits.fill(0);
        }

        // Restore tokens
        self.tokens.clear();
        self.tokens.extend_from_slice(&saved.tokens);

        // Restore snap trace
        if let Some(ref trace_entries) = saved.snap_trace {
            if self.model.snapped {
                let total = self.model.shape.context * self.model.shape.lanes() * rec_layers;
                let mut selected = vec![u32::MAX; total];
                selected[..trace_entries.len()].copy_from_slice(trace_entries);
                self.b.snap_trace = Some(SnapTrace {
                    origin_position: 0,
                    selected,
                    written: trace_entries.len(),
                });
            } else {
                self.b.snap_trace = None;
            }
        } else {
            // Untraced snapshot: explicitly disable tracing in destination session to
            // prevent partial or misaligned root history from a prior session.
            self.b.snap_trace = None;
        }

        Ok(())
    }

    /// Save session state to a JSON file at `path` atomically and durably.
    ///
    /// Writes to an exclusive temporary sibling file, flushes and synchronizes content to disk,
    /// atomically renames over destination `path`, and synchronizes the parent directory.
    /// If an error occurs prior to rename, any existing checkpoint file at `path` remains intact.
    /// Note that if an error occurs during parent directory synchronization after rename, the
    /// destination file has already been replaced but directory durability remains unconfirmed.
    pub fn save_session_to_file(&self, path: &Path) -> Result<(), StackError> {
        self.save_session_to_file_internal(path, None)
    }

    pub(super) fn save_session_to_file_internal(
        &self,
        path: &Path,
        temp_override: Option<PathBuf>,
    ) -> Result<(), StackError> {
        let serialized = self.save_state();
        let bytes =
            serde_json::to_vec(&serialized).map_err(|e| StackError::Numerics(e.to_string()))?;
        let parent = match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("."),
        };
        std::fs::create_dir_all(parent).map_err(StackError::Io)?;

        static SAVE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = SAVE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let pid = std::process::id();
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("session");
        let temp_path =
            temp_override.unwrap_or_else(|| parent.join(format!(".{file_name}.tmp-{pid}-{count}")));

        let mut temp_created = false;
        let write_res = (|| -> Result<(), std::io::Error> {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)?;
            temp_created = true;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);

            std::fs::rename(&temp_path, path)?;
            temp_created = false;

            let dir = std::fs::File::open(parent)?;
            dir.sync_all()?;
            Ok(())
        })();

        if let Err(e) = write_res {
            if temp_created {
                let _ = std::fs::remove_file(&temp_path);
            }
            return Err(StackError::Io(e));
        }

        Ok(())
    }

    /// Load and restore session state from a JSON file at `path`.
    ///
    /// Validates file length against a model-derived upper bound before reading
    /// to prevent memory exhaustion from oversized or malformed payloads.
    pub fn restore_session_from_file(&mut self, path: &Path) -> Result<(), StackError> {
        use std::io::Read;
        let file = std::fs::File::open(path).map_err(StackError::Io)?;
        let max_len = max_serialized_session_bytes(self.model);
        if let Ok(meta) = file.metadata() {
            if meta.len() > max_len {
                return Err(StackError::Numerics(format!(
                    "session file size {} exceeds maximum expected bound {}",
                    meta.len(),
                    max_len
                )));
            }
        }
        let mut bytes = Vec::new();
        file.take(max_len + 1)
            .read_to_end(&mut bytes)
            .map_err(StackError::Io)?;
        if bytes.len() as u64 > max_len {
            return Err(StackError::Numerics(format!(
                "session file size exceeds maximum expected bound {max_len}"
            )));
        }
        let serialized: SerializedStackSession = serde_json::from_slice(&bytes)
            .map_err(|e| StackError::Numerics(format!("invalid session JSON: {e}")))?;
        self.restore_state(&serialized)
    }

    /// The logits of the last step (value `v` means `v * 2^-16`).
    pub fn logits(&self) -> &[i32] {
        &self.b.logits
    }

    /// A pointer model's next-token distribution after the last step: the
    /// mixture in Q30 (value `v` means probability `v * 2^-30`); `None`
    /// without a pointer head.
    pub fn mixture(&self) -> Option<&[i32]> {
        self.model.pointer.as_ref().map(|_| &self.b.mixture[..])
    }

    /// What greedy decoding takes the argmax of: the mixture of a pointer
    /// model, the logits otherwise.
    pub fn next_token_scores(&self) -> &[i32] {
        self.mixture().unwrap_or(&self.b.logits)
    }

    /// Feed one token at the next position; returns the next-token logits
    /// (value `v` means `v * 2^-16`): the D10 engine's integers (see the
    /// module documentation for how that is tested).
    #[inline(never)]
    pub fn step(&mut self, token: u32) -> Result<&[i32], StackError> {
        let model = self.model;
        let (s, n) = (&model.shape, &model.numerics);
        let (d, heads, mlp) = (s.width, s.heads, s.mlp);
        if token as usize >= s.vocab {
            return Err(StackError::Token {
                token,
                vocab: s.vocab,
            });
        }
        if self.position >= s.context {
            return Err(StackError::ContextFull { context: s.context });
        }
        let b = &mut self.b;
        stack_dequant_row(&model.embed, token as usize, &mut b.x);
        let last_read_idx = model
            .layers
            .iter()
            .rposition(|l| matches!(l.mixer, Mixer::Read(_)));
        for (idx, (layer, state)) in model.layers.iter().zip(self.states.iter_mut()).enumerate() {
            let norm_exp = stack_rms_norm(&b.x, n.rms_eps, &mut b.scratch[..d], &mut b.norm);
            stack_activation_tables(&b.norm, &mut b.tables[..d]);
            stack_pair_tables(&b.tables[..d], &mut b.pairs);
            match (&layer.mixer, state.as_mut()) {
                (Mixer::Recurrence(r), LayerState::Recurrence { state, history }) => {
                    stack_recurrence(model, r, state, history, b, norm_exp)
                }
                (
                    Mixer::Read(r),
                    LayerState::Read {
                        keys,
                        values,
                        lifts,
                    },
                ) => stack_read(
                    model,
                    r,
                    Cache {
                        keys,
                        values,
                        lifts,
                        position: self.position,
                        at: self.cache_at,
                        lift_at: self.lift_at,
                    },
                    b,
                    norm_exp,
                    Some(idx) == last_read_idx && self.copy_scale_q16 > 0,
                ),
                _ => {
                    // `session` builds every layer's state from the model's
                    // own pattern, so a mismatch is unreachable.
                    debug_assert!(
                        false,
                        "stack step: a layer's state does not match its mixer"
                    );
                    return Err(StackError::SessionState);
                }
            }
            for (x, p) in b.x.iter_mut().zip(&b.proj) {
                *x = x.saturating_add(*p);
            }
            // MLP block.
            let norm_exp = stack_rms_norm(&b.x, n.rms_eps, &mut b.scratch[..d], &mut b.norm);
            stack_activation_tables(&b.norm, &mut b.tables[..d]);
            stack_pair_tables(&b.tables[..d], &mut b.pairs);
            stack_gemv_pairs(&layer.gate, &b.pairs, norm_exp, &mut b.gate);
            stack_gemv_pairs(&layer.up, &b.pairs, norm_exp, &mut b.up);
            stack_swiglu(model, &b.gate, &b.up, &mut b.scratch[..mlp]);
            let act_exp = stack_quantize16(&b.scratch[..mlp], PRODUCT_EXP, &mut b.act[..mlp]);
            stack_activation_tables(&b.act[..mlp], &mut b.tables[..mlp]);
            stack_gemv(&layer.down, &b.tables, act_exp, &mut b.proj);
            for (x, p) in b.x.iter_mut().zip(&b.proj) {
                *x = x.saturating_add(*p);
            }
        }
        let norm_exp = stack_rms_norm(&b.x, n.rms_eps, &mut b.scratch[..d], &mut b.norm);
        stack_activation_tables(&b.norm, &mut b.tables[..d]);
        stack_pair_tables(&b.tables[..d], &mut b.pairs);
        stack_gemv_pairs(&model.head, &b.pairs, norm_exp, &mut b.logits);
        if self.copy_scale_q16 > 0 {
            let scale = i128::from(self.copy_scale_q16);
            for (j, &tok) in self.tokens[..self.position].iter().enumerate() {
                if (tok as usize) < s.vocab {
                    let pw = i128::from(self.b.pointer_weights[j]);
                    let boost = shift_wide(mul_i128(pw, scale), 31) as i32;
                    self.b.logits[tok as usize] = self.b.logits[tok as usize].saturating_add(boost);
                }
            }
        }
        self.tokens.push(token);
        if let Some(p) = &model.pointer {
            stack_pointer(
                model,
                p,
                PointerCache {
                    tokens: &self.tokens,
                    keys: &mut self.pointer_keys,
                    lifts: &mut self.pointer_lifts,
                    position: self.position,
                    at: self.pointer_at,
                },
                &mut self.b,
                norm_exp,
            );
            self.pointer_at += p.dim;
        }
        self.position += 1;
        self.cache_at += d;
        self.lift_at += heads;
        Ok(&self.b.logits)
    }
}

/// `silu(gate) up` per MLP unit at exponent -32.
#[inline(never)]
fn stack_swiglu(model: &IntegerStackModel, gate: &[i32], up: &[i32], out: &mut [i64]) {
    let n = &model.numerics;
    for ((o, &g), &u) in out.iter_mut().zip(gate).zip(up) {
        let a = stack_activation(g, &model.silu_table, n.silu_step_log2, n.silu_range_log2);
        *o = super::kernels::mul_i64(i64::from(a), i64::from(u));
    }
}

/// One recurrence step from the normalized state (pair tables built), into
/// `b.proj`.
#[inline(never)]
fn stack_recurrence(
    model: &IntegerStackModel,
    r: &Recurrence,
    state: &mut [i64],
    history: &mut [i32],
    b: &mut Buffers,
    norm_exp: i32,
) {
    let (s, n) = (&model.shape, &model.numerics);
    let d = s.width;
    let lanes = s.lanes();
    stack_gemv_pairs(&r.input, &b.pairs, norm_exp, &mut b.branches);
    stack_gemv_pairs(&r.gates, &b.pairs, norm_exp, &mut b.gate_out);
    for (g, bias) in b.gate_out.iter_mut().zip(&r.gate_bias) {
        *g = g.saturating_add(*bias);
    }
    let (drive, gate) = b.branches.split_at(d);
    // Causal convolution of the drive: this position and the three before it
    // (zero before the stream starts), at exponent -16.
    let (tap0, rest) = r.taps.split_at(d);
    let (tap1, rest) = rest.split_at(d);
    let (tap2, tap3) = rest.split_at(d);
    let (back1, rest) = history.split_at(d);
    let (back2, back3) = rest.split_at(d);
    for (i, slot) in b.wide.iter_mut().enumerate().take(d) {
        let c = i64::from(r.conv_bias[i])
            .wrapping_add(grid_apply(i64::from(drive[i]), tap0[i]))
            .wrapping_add(grid_apply(i64::from(back1[i]), tap1[i]))
            .wrapping_add(grid_apply(i64::from(back2[i]), tap2[i]))
            .wrapping_add(grid_apply(i64::from(back3[i]), tap3[i]));
        *slot = c;
    }
    // Age the history by one position: the oldest row drops off the end.
    history.copy_within(..history.len() - d, d);
    history[..d].copy_from_slice(drive);
    let (decays, rotations) = b.gate_out.split_at(lanes);
    for (lane, ((held, pushed), (&decay, &rate))) in state
        .chunks_exact_mut(4)
        .zip(b.wide.chunks_exact(4))
        .zip(decays.iter().zip(&r.rates))
        .enumerate()
    {
        // Decay gate, decay and the drive's weight, all Q31.
        let opening = stack_sigmoid_q31(i64::from(decay), &model.exp_table, n.exp_step_log2);
        let exponent = grid_apply(opening as i64, rate).max(0);
        let lambda = stack_exp_neg(exponent, GATE_EXP, &model.exp_table, n.exp_step_log2);
        let complement = (1u64 << 62).saturating_sub(stack_mul_u64(lambda, lambda));
        let keep = if complement < COMPLEMENT_FLOOR_Q62 {
            KEEP_FLOOR_Q31
        } else {
            stack_isqrt(u128::from(complement)) as u64
        };
        let transition: [i64; 4] = if s.rotation {
            let at = lane << 2;
            let raw = [
                rotations[at],
                rotations[at + 1],
                rotations[at + 2],
                rotations[at + 3],
            ];
            if model.snapped {
                let transition = stack_snap_rotation(raw, lambda);
                if let Some(trace) = &mut b.snap_trace {
                    // The same deterministic selection the transition used.
                    trace.record(stack_snap_select(raw));
                }
                transition
            } else {
                stack_rotation(raw, lambda)
            }
        } else {
            [lambda as i64, 0, 0, 0]
        };
        let moved = stack_hamilton(transition, [held[0], held[1], held[2], held[3]]);
        for ((h, &c), m) in held.iter_mut().zip(pushed).zip(moved) {
            // Transport (Q31 times exponent -32) plus drive (Q31 times exponent
            // -16, raised to -63), back to exponent -32.
            let drive = mul_i128(i128::from(keep), i128::from(c));
            *h = shift_wide(m.wrapping_add(drive << 16), 31);
        }
    }
    // Output h gelu(g) at exponent -32, then the output map.
    for ((o, &h), &g) in b.scratch.iter_mut().zip(state.iter()).zip(gate) {
        let h = shift(h, RESIDUAL_EXP - STATE_EXP);
        let g = stack_activation(g, &model.gelu_table, n.gelu_step_log2, n.gelu_range_log2);
        *o = saturating_product(h, i64::from(g));
    }
    let act_exp = stack_quantize16(&b.scratch[..d], PRODUCT_EXP, &mut b.act[..d]);
    stack_activation_tables(&b.act[..d], &mut b.tables[..d]);
    stack_gemv(&r.out, &b.tables, act_exp, &mut b.proj);
}

/// `lambda raw / |raw|` per coordinate: the unit rotation (Q30, rounded toward
/// zero as the D10 engine's signed division) times the decay (Q31), at Q31.
#[inline(never)]
fn stack_rotation(raw: [i32; 4], lambda: u64) -> [i64; 4] {
    let mut square = 0u128;
    for &v in &raw {
        square = square.wrapping_add(u128::from(stack_square(i64::from(v))));
    }
    // |raw| at exponent -32.
    let norm = stack_isqrt((square + ROTATION_EPSILON) << 32);
    // An explicit loop, not `raw.map`: the closure of `map` compiles to a
    // `core::array` symbol of its own, outside the audited `stack_` names.
    let mut transition = [0i64; 4];
    for (slot, &v) in transition.iter_mut().zip(&raw) {
        let magnitude = stack_div_u128(u128::from(v.unsigned_abs()) << 46, norm) as i128;
        let unit = if v < 0 { -magnitude } else { magnitude };
        *slot = (mul_i128(i128::from(lambda), unit) >> 30) as i64;
    }
    transition
}

/// `a b` saturated to the `i64` range (the D10 engine's `saturating_mul`).
#[inline(always)]
fn saturating_product(a: i64, b: i64) -> i64 {
    mul_i128(i128::from(a), i128::from(b)).clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

/// The key/value cache of one read layer and where this position writes.
struct Cache<'a> {
    keys: &'a mut [i32],
    values: &'a mut [i32],
    lifts: &'a mut [u64],
    position: usize,
    /// `position * width`.
    at: usize,
    /// `position * heads`.
    lift_at: usize,
}

/// One read step from the normalized state (pair tables built), into `b.proj`.
#[inline(never)]
fn stack_read(
    model: &IntegerStackModel,
    r: &Read,
    cache: Cache<'_>,
    b: &mut Buffers,
    norm_exp: i32,
    capture_pointer: bool,
) {
    let (s, n) = (&model.shape, &model.numerics);
    let (d, heads, hd, context) = (s.width, s.heads, model.head_dim, s.context);
    let (lorentz, l2) = (model.lorentz, model.l2);
    let Cache {
        keys,
        values,
        lifts,
        position,
        at,
        lift_at,
    } = cache;
    stack_gemv_pairs(&r.query, &b.pairs, norm_exp, &mut b.q);
    stack_gemv_pairs(&r.key, &b.pairs, norm_exp, &mut b.k);
    stack_gemv_pairs(&r.value, &b.pairs, norm_exp, &mut b.v);
    stack_gemv_pairs(&r.null, &b.pairs, norm_exp, &mut b.null);
    keys[at..at + d].copy_from_slice(&b.k);
    values[at..at + d].copy_from_slice(&b.v);
    if lorentz {
        let mut from = 0usize;
        for lift in &mut lifts[lift_at..lift_at + heads] {
            *lift = stack_lift(&b.k[from..from + hd]);
            from += hd;
        }
    }
    let positions = position + 1;
    let (mut head_at, mut age_at) = (0usize, 0usize);
    for h in 0..heads {
        let query = &b.q[head_at..head_at + hd];
        // The L2 read needs no inner product, so no query tables.
        if !l2 {
            stack_query_tables(query, &mut b.query_tables);
        }
        let query_lift = if lorentz { stack_lift(query) } else { 0 };
        let null_score = i64::from(b.null[h]) + i64::from(r.null_bias[h]);
        let ages = &r.age[age_at..age_at + context];
        let mut max = null_score;
        let (mut key_at, mut lift_index) = (head_at, h);
        for (j, score_slot) in b.scores[..positions].iter_mut().enumerate() {
            let key = &keys[key_at..key_at + hd];
            let score = if l2 {
                // At most 2^44, so the difference and the grid product fit.
                let distance = stack_l2_distance(query, key) as i64;
                let scaled = grid_apply(distance - i64::from(r.offset[h]), r.beta[h]);
                shift(scaled.wrapping_neg(), SCORE_EXP - DISTANCE_EXP)
            } else if lorentz {
                let dot = stack_dot(&b.query_tables, key);
                let distance =
                    stack_lorentz_distance(query_lift, lifts[lift_index], dot, &model.arcosh);
                let scaled = grid_apply(i64::from(distance) - i64::from(r.offset[h]), r.beta[h]);
                shift(scaled.wrapping_neg(), SCORE_EXP - DISTANCE_EXP)
            } else {
                let dot = stack_dot(&b.query_tables, key);
                // Exponent -32 times Q30 is exponent -62.
                shift_wide(
                    mul_i128(dot, i128::from(n.score_scale_q30)),
                    (SCORE_EXP - (PRODUCT_EXP - 30)) as u32,
                )
            };
            let score = score.saturating_add(i64::from(ages[position - j]));
            max = max.max(score);
            *score_slot = score;
            key_at += d;
            lift_index += heads;
        }
        // Softmax over NoRead (value zero) and the positions, weights in Q31.
        let mut total = stack_exp_neg(
            max.wrapping_sub(null_score),
            SCORE_EXP,
            &model.exp_table,
            n.exp_step_log2,
        );
        for (w, &score) in b.weights[..positions]
            .iter_mut()
            .zip(&b.scores[..positions])
        {
            *w = stack_exp_neg(
                max.wrapping_sub(score),
                SCORE_EXP,
                &model.exp_table,
                n.exp_step_log2,
            );
            total = total.wrapping_add(*w);
        }
        b.mix.fill(0);
        let mut value_at = head_at;
        for &w in &b.weights[..positions] {
            if w != 0 {
                stack_mix_row(w, &values[value_at..value_at + hd], &mut b.mix);
            }
            value_at += d;
        }
        // Q31-weighted sum over the Q31 total, back to exponent -16.
        let reciprocal = i128::from(stack_div_u128(1u128 << 62, u128::from(total.max(1))) as u64);
        if capture_pointer && h == 0 {
            for j in 0..positions {
                let w = b.weights[j];
                let norm_w = shift_wide(mul_i128(i128::from(w), reciprocal), 31) as u64;
                b.pointer_weights[j] = norm_w;
            }
        }
        for (slot, &m) in b.wide[head_at..head_at + hd].iter_mut().zip(&b.mix) {
            *slot = shift_wide(mul_i128(m, reciprocal), 62);
        }
        head_at += hd;
        age_at += context;
    }
    let act_exp = stack_quantize16(&b.wide[..d], RESIDUAL_EXP, &mut b.act[..d]);
    stack_activation_tables(&b.act[..d], &mut b.tables[..d]);
    stack_gemv(&r.out, &b.tables, act_exp, &mut b.proj);
}

/// The pointer head's cache and where this position writes.
struct PointerCache<'a> {
    /// The input tokens of positions `0..=position`.
    tokens: &'a [u32],
    keys: &'a mut [i32],
    lifts: &'a mut [u64],
    position: usize,
    /// `position * dim`.
    at: usize,
}

/// The pointer head's step from the final normalized state (pair tables
/// built) and the logits: its query, key and gate, the copy and generated
/// weights and the mixture (see the module documentation) into `b.mixture`.
#[inline(never)]
fn stack_pointer(
    model: &IntegerStackModel,
    p: &Pointer,
    cache: PointerCache<'_>,
    b: &mut Buffers,
    norm_exp: i32,
) {
    let n = &model.numerics;
    let dim = p.dim;
    let PointerCache {
        tokens,
        keys,
        lifts,
        position,
        at,
    } = cache;
    stack_gemv_pairs(&p.query, &b.pairs, norm_exp, &mut b.pointer_query);
    stack_gemv_pairs(&p.key, &b.pairs, norm_exp, &mut b.pointer_key);
    stack_gemv_pairs(&p.gate, &b.pairs, norm_exp, &mut b.pointer_gate);
    keys[at..at + dim].copy_from_slice(&b.pointer_key);
    if p.lorentz {
        lifts[position] = stack_lift(&b.pointer_key);
    }
    // Scores of the sources 0..=t at exponent -16.
    let positions = position + 1;
    stack_query_tables(&b.pointer_query, &mut b.pointer_query_tables);
    let query_lift = if p.lorentz {
        stack_lift(&b.pointer_query)
    } else {
        0
    };
    let mut max = i64::MIN;
    let mut key_at = 0usize;
    for (j, slot) in b.scores[..positions].iter_mut().enumerate() {
        let key = &keys[key_at..key_at + dim];
        let dot = stack_dot(&b.pointer_query_tables, key);
        let score = if p.lorentz {
            let distance = stack_lorentz_distance(query_lift, lifts[j], dot, &model.arcosh);
            shift(
                grid_apply(i64::from(distance), p.beta).wrapping_neg(),
                SCORE_EXP - DISTANCE_EXP,
            )
        } else {
            shift_wide(
                mul_i128(dot, i128::from(p.score_scale_q30)),
                (SCORE_EXP - (PRODUCT_EXP - 30)) as u32,
            )
        };
        max = max.max(score);
        *slot = score;
        key_at += dim;
    }
    // The copy distribution's weights, total and mass per input token.
    let mut copy_total = 0u64;
    for (&score, &id) in b.scores[..positions].iter().zip(tokens) {
        let w = stack_exp_neg(
            max.wrapping_sub(score),
            SCORE_EXP,
            &model.exp_table,
            n.exp_step_log2,
        );
        copy_total = copy_total.wrapping_add(w);
        if let Some(mass) = b.copy.get_mut(id as usize) {
            *mass = mass.wrapping_add(w);
        }
    }
    // The generated distribution's weights and total.
    // Every element passes through an opaque barrier, which keeps the
    // reduction scalar: a vectorized maximum ends in a SIMD-to-general
    // register transfer (`fmov`), which the R1 audit refuses.
    let mut top = i32::MIN;
    for &z in &b.logits {
        top = top.max(std::hint::black_box(z));
    }
    let mut generate_total = 0u64;
    for (u, &z) in b.generate.iter_mut().zip(&b.logits) {
        *u = stack_exp_neg(
            i64::from(top) - i64::from(z),
            RESIDUAL_EXP,
            &model.exp_table,
            n.exp_step_log2,
        );
        generate_total = generate_total.wrapping_add(*u);
    }
    let gate = b.pointer_gate[0].saturating_add(p.gate_bias);
    let copy_weight = stack_sigmoid_q31(i64::from(gate), &model.exp_table, n.exp_step_log2);
    let generate_weight = (1u64 << 31).saturating_sub(copy_weight);
    let generate_coefficient = pointer_coefficient(generate_weight, generate_total);
    let copy_coefficient = pointer_coefficient(copy_weight, copy_total);
    stack_pointer_mixture(
        &b.generate,
        &b.copy,
        generate_coefficient,
        copy_coefficient,
        &mut b.mixture,
    );
    // Clear the copy mass where this step wrote it.
    for &id in tokens {
        if let Some(mass) = b.copy.get_mut(id as usize) {
            *mass = 0;
        }
    }
}

/// `round(weight floor(2^93 / total) 2^-31)` modulo `2^128` (the D10
/// engine's coefficient), by long division and a table product.
#[inline(always)]
fn pointer_coefficient(weight: u64, total: u64) -> u128 {
    let reciprocal = stack_div_u128(1u128 << 93, u128::from(total.max(1)));
    stack_mul_u128(u128::from(weight), reciprocal).wrapping_add(1 << 30) >> 31
}
