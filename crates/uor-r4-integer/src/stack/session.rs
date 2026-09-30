//! The stack model (validated, repacked at load) and its incremental decoding
//! session. `IntegerStackSession::step` and the mixers it calls are the served
//! numerical path: they use the kernels of [`super::kernels`], running offsets
//! for every index, and buffers allocated when the session is created.

use std::path::Path;

use super::format::{Container, Fixed, MatrixView, StackNumerics, StackShape, StackTransportSnap};
use super::kernels::{
    grid_apply, grid_valid, mul_i128, shift, shift_wide, stack_activation, stack_activation_tables,
    stack_dequant_row, stack_div_u128, stack_dot, stack_exp_neg, stack_gemv, stack_gemv_pairs,
    stack_hamilton, stack_isqrt, stack_lift, stack_lorentz_distance, stack_mix_row, stack_mul_u64,
    stack_pair_tables, stack_quantize16, stack_query_tables, stack_rms_norm, stack_sigmoid_q31,
    stack_snap_rotation, stack_snap_select, stack_square, PackedMatrix, ARCOSH_TABLE_LEN,
    PRODUCT_EXP, RESIDUAL_EXP,
};
use super::StackError;

/// Exponent of the recurrence state.
const STATE_EXP: i32 = -32;
/// Exponent of read scores, in nats.
const SCORE_EXP: i32 = -16;
/// Exponent of Lorentz distances and offsets (the arcosh table's output).
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
    /// Lorentz scale per head (grid codes) and offset (exponent -24).
    beta: Vec<i16>,
    offset: Vec<i32>,
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
    /// The artifact records the icosian transport snap: the recurrence
    /// transports by the nearest root instead of the free unit quaternion.
    snapped: bool,
    transport_snap: Option<StackTransportSnap>,
    sha256: String,
    embed: PackedMatrix,
    head: PackedMatrix,
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
                let lorentz = shape.lorentz();
                Mixer::Read(Box::new(Read {
                    query: matrix(&name("query"), d, d)?,
                    key: matrix(&name("key"), d, d)?,
                    value: matrix(&name("value"), d, d)?,
                    null: matrix(&name("null"), heads, d)?,
                    out: matrix(&name("out"), d, d)?,
                    null_bias: integers(&name("null_bias"), heads)?,
                    age: integers(&name("age"), context_ages)?,
                    beta: if lorentz {
                        codes(&name("beta"), heads, true)?
                    } else {
                        Vec::new()
                    },
                    offset: if lorentz {
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
        // The embedding reads one row per token.
        weights += d as u64;
        let exp_table = artifact.table_u32("exp")?;
        let silu_table = artifact.table_i32("silu")?;
        let gelu_table = artifact.table_i32("gelu")?;
        let arcosh = if shape.lorentz() {
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
            snapped: artifact.transport_snap.is_some(),
            transport_snap: artifact.transport_snap.clone(),
            shape,
            numerics,
            sha256: artifact.sha256,
            embed,
            head,
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
        IntegerStackSession {
            model: self,
            position: 0,
            cache_at: 0,
            lift_at: 0,
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
                mix: vec![0; self.head_dim],
                proj: vec![0; d],
                gate: vec![0; s.mlp],
                up: vec![0; s.mlp],
                logits: vec![0; s.vocab],
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
    /// Boxed, like the model's layers, for a power-of-two stride.
    #[allow(clippy::vec_box)]
    states: Vec<Box<LayerState>>,
    b: Buffers,
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
                let (position, rem) = (slot / per_position, slot % per_position);
                SnapTraceEntry {
                    layer: recurrences[rem / lanes],
                    position,
                    lane: rem % lanes,
                    root: trace.selected[slot] as usize,
                }
            })
            .collect();
        Some(entries)
    }

    /// Start a new stream.
    pub fn reset(&mut self) {
        self.position = 0;
        self.cache_at = 0;
        self.lift_at = 0;
        if let Some(trace) = &mut self.b.snap_trace {
            trace.written = 0;
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

    /// The logits of the last step (value `v` means `v * 2^-16`).
    pub fn logits(&self) -> &[i32] {
        &self.b.logits
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
        for (layer, state) in model.layers.iter().zip(self.states.iter_mut()) {
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
) {
    let (s, n) = (&model.shape, &model.numerics);
    let (d, heads, hd, context) = (s.width, s.heads, model.head_dim, s.context);
    let lorentz = model.lorentz;
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
        stack_query_tables(query, &mut b.query_tables);
        let query_lift = if lorentz { stack_lift(query) } else { 0 };
        let null_score = i64::from(b.null[h]) + i64::from(r.null_bias[h]);
        let ages = &r.age[age_at..age_at + context];
        let mut max = null_score;
        let (mut key_at, mut lift_index) = (head_at, h);
        for (j, score_slot) in b.scores[..positions].iter_mut().enumerate() {
            let dot = stack_dot(&b.query_tables, &keys[key_at..key_at + hd]);
            let score = if lorentz {
                let distance =
                    stack_lorentz_distance(query_lift, lifts[lift_index], dot, &model.arcosh);
                let scaled = grid_apply(i64::from(distance) - i64::from(r.offset[h]), r.beta[h]);
                shift(scaled.wrapping_neg(), SCORE_EXP - DISTANCE_EXP)
            } else {
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
