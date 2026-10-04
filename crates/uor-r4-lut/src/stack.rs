//! Integer serving of the lab's geometric stack (`uor-r4-training`'s
//! `geometric_stack`, cycle 4) under owner decision D10.
//!
//! Each layer is a temporal mixer and then a SwiGLU MLP, both pre-norm
//! residual blocks. The mixers are:
//!
//! - **quaternion transport recurrence** (`r`): per lane of four channels,
//!   `h_t = lambda_t (u_t (x) h_{t-1}) + sqrt(1 - lambda_t^2) c_t`, where
//!   `u_t` is a unit quaternion (the identity without learned rotations),
//!   `lambda_t = exp(-rate r_t)` for the decay gate `r_t = sigmoid(.)`, `(x)`
//!   is the Hamilton product and `c_t` a width-4 causal convolution of the
//!   drive. The output is `h_t gelu(g_t)`;
//! - **read** (`a`): multi-head attention over the positions so far, scored by
//!   `<q, k> / sqrt(d)` (Dot) or `-beta (d(q, k) - offset)` (Lorentz: the
//!   hyperboloid distance of the lifted points `(sqrt(1 + |x|^2), x)`; L2,
//!   its flat control: the Euclidean distance `|q - k|`), plus a learned age
//!   bias per head and distance, with a NoRead slot whose value is zero.
//!
//! Arithmetic follows the Llama engine ([`crate::engine`]). Learned weight maps
//! are 4-bit table GEMVs. Learned scalars (convolution taps, decay rates,
//! Lorentz scales) are grid codes applied by shifts and adds
//! ([`grid_apply`]). Learned biases and age tables are integers that are
//! added. Products of runtime values (transport, gating, scores, value mixing,
//! normalization) use the integer multiplier. Exp, sigmoid, SiLU, GELU and
//! arcosh are sealed tables, square roots are integer, and each normalization,
//! rotation and softmax divides once per vector or quaternion.

use std::path::Path;

use uor_r4_simd::{Backend, Tables};

use crate::format::{MatrixSpec, StackArtifact, StackHeader, StackNumerics, StackShape};
use crate::kernels::{
    arcosh1p_q24, dequant_row, exp_neg, gemv, grid_apply, grid_valid, isqrt, quantize16, rms_norm,
    shift, sigmoid_q31, silu, Act16, MatrixView, Packed, ARCOSH_TABLE_LEN,
};
use crate::{format_error, invalid, Result, RESIDUAL_EXP};

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
/// Training's floor on the Lorentz excess `z - 1`, `1e-7`, as an arcosh code.
const MIN_EXCESS_CODE: u128 = 429;
/// Training's floor on the L2 squared distance `|q - k|^2`, `1e-7`, at
/// exponent -32 (`floor(1e-7 2^32)`, the same code as the Lorentz floor).
const MIN_SQUARED_CODE: u128 = 429;

/// Round-half-up shift of an `i128` (right for positive `shift`), saturated to `i64`.
fn shift_wide(value: i128, shift: u32) -> i64 {
    let rounded = if shift == 0 {
        value
    } else {
        (value + (1i128 << (shift - 1))) >> shift
    };
    rounded.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

/// Hamilton product `a (x) b` of quaternions `(w, x, y, z)`.
fn hamilton(a: [i64; 4], b: [i64; 4]) -> [i128; 4] {
    let [a0, a1, a2, a3] = a.map(i128::from);
    let [b0, b1, b2, b3] = b.map(i128::from);
    [
        a0 * b0 - a1 * b1 - a2 * b2 - a3 * b3,
        a0 * b1 + a1 * b0 + a2 * b3 - a3 * b2,
        a0 * b2 - a1 * b3 + a2 * b0 + a3 * b1,
        a0 * b3 + a1 * b2 - a2 * b1 + a3 * b0,
    ]
}

/// Time coordinate `sqrt(1 + |v|^2)` at exponent -32 of the hyperboloid lift of
/// `v` at exponent -16.
fn lift(v: &[i32]) -> u64 {
    let square: u128 = v
        .iter()
        .map(|x| (i128::from(*x) * i128::from(*x)) as u128)
        .sum();
    isqrt((1u128 << 64) + (square << 32)).min(u128::from(u64::MAX)) as u64
}

/// Lorentz distance at exponent -24 between points at exponent -16, from
/// their lifts and inner product (exponent -32): `z - 1 = q0 k0 - <q, k> - 1`
/// at exponent -64, floored as in training, then the arcosh table.
fn lorentz_distance(query_lift: u64, key_lift: u64, dot: i128, arcosh: &[u32]) -> u32 {
    let excess = i128::from(query_lift) * i128::from(key_lift) - (dot << 32) - (1i128 << 64);
    let code = ((excess.max(0) >> 32) as u128).max(MIN_EXCESS_CODE);
    arcosh1p_q24(code, arcosh)
}

/// L2 distance `sqrt(max(|q - k|^2, 1e-7))` at exponent -24 between points
/// at exponent -16: the exact squared distance at exponent -32, floored as in
/// training, then the floor square root of it shifted to exponent -48. At
/// most `2^44` (head width at most 256, coordinates in the `i32` range).
fn l2_distance(query: &[i32], key: &[i32]) -> u64 {
    let squared: u128 = query
        .iter()
        .zip(key)
        .map(|(q, k)| {
            let v = i128::from(i64::from(*q) - i64::from(*k));
            (v * v) as u128
        })
        .sum();
    isqrt(squared.max(MIN_SQUARED_CODE) << 16) as u64
}

struct Recurrence {
    /// `2 width x width`: the drive, then the output gate.
    input: Packed,
    /// `gate_rows x width`: decay gates, then rotations.
    gates: Packed,
    out: Packed,
    /// Grid codes `[tap][channel]`; tap `s` weights the drive `s` positions back.
    taps: Vec<i16>,
    conv_bias: Vec<i32>,
    gate_bias: Vec<i32>,
    /// Grid codes of `8 softplus(-decay)` per lane.
    rates: Vec<i16>,
}

struct Read {
    query: Packed,
    key: Packed,
    value: Packed,
    null: Packed,
    out: Packed,
    null_bias: Vec<i32>,
    /// `[head][distance]` at the score exponent.
    age: Vec<i32>,
    /// Lorentz or L2 scale per head (grid codes) and offset (exponent -24).
    beta: Vec<i16>,
    offset: Vec<i32>,
}

enum Mixer {
    Recurrence(Box<Recurrence>),
    Read(Box<Read>),
}

struct Layer {
    mixer: Mixer,
    gate: Packed,
    up: Packed,
    down: Packed,
}

/// A validated stack artifact repacked for integer serving.
pub struct StackModel {
    header: StackHeader,
    sha256: String,
    backend: Backend,
    threads: usize,
    pool: Option<rayon::ThreadPool>,
    shape: StackShape,
    numerics: StackNumerics,
    embed_spec: MatrixSpec,
    embed_nibbles: Vec<u8>,
    embed_scales: Vec<u8>,
    head: Packed,
    layers: Vec<Layer>,
    exp_table: Vec<u32>,
    silu_table: Vec<i32>,
    gelu_table: Vec<i32>,
    arcosh: Vec<u32>,
}

impl StackModel {
    pub fn load(path: &Path) -> Result<Self> {
        Self::from_artifact(StackArtifact::load(path)?)
    }

    pub fn from_artifact(artifact: StackArtifact) -> Result<Self> {
        // This engine computes the free transport. An artifact that records a
        // trained-in transport snap is served only by the multiplier-free
        // engine, so refuse it here as well as in the serving parse: an
        // artifact from the offline `StackArtifact::parse_for_reference` must
        // not reach this engine.
        if let Some(snap) = &artifact.header.transport_snap {
            return Err(format_error(format!(
                "the artifact records transport_snap={} ({} roots), which only the \
                 multiplier-free stack engine serves; this engine computes the free transport",
                snap.name, snap.roots
            )));
        }
        let shape = artifact.header.shape.clone();
        let numerics = artifact.header.numerics.clone();
        let (d, heads, mlp) = (shape.width, shape.heads, shape.mlp);
        let spec = |name: &str, rows: usize, cols: usize| -> Result<MatrixSpec> {
            let spec = artifact.matrix(name)?.clone();
            if spec.rows != rows || spec.cols != cols {
                return Err(format_error(format!(
                    "matrix {name} is {}x{}, expected {rows}x{cols}",
                    spec.rows, spec.cols
                )));
            }
            Ok(spec)
        };
        let packed = |name: &str, rows: usize, cols: usize| -> Result<Packed> {
            let spec = spec(name, rows, cols)?;
            Packed::new(&MatrixView {
                rows,
                cols,
                exp_base: spec.exp_base,
                nibbles: artifact.section(spec.nibbles),
                scales: artifact.section(spec.scales),
            })
        };
        let integers = |name: &str, len: usize| -> Result<Vec<i32>> {
            let values = artifact.table_i32(name)?;
            if values.len() != len {
                return Err(format_error(format!("table {name} has the wrong length")));
            }
            Ok(values)
        };
        // Grid codes of positive scalars (rates and scales) or of any sign (taps).
        let codes = |name: &str, len: usize, positive: bool| -> Result<Vec<i16>> {
            let values = artifact.table_i16(name)?;
            if values.len() != len
                || values
                    .iter()
                    .any(|&c| !grid_valid(c) || (positive && c <= 0))
            {
                return Err(format_error(format!(
                    "table {name} holds invalid grid codes"
                )));
            }
            Ok(values)
        };
        let mut layers = Vec::with_capacity(shape.layers());
        for (l, kind) in shape.pattern.bytes().enumerate() {
            let name = |part: &str| format!("l{l}.{part}");
            let mixer = if kind == b'r' {
                Mixer::Recurrence(Box::new(Recurrence {
                    input: packed(&name("rec_in"), 2 * d, d)?,
                    gates: packed(&name("rec_gate"), shape.gate_rows(), d)?,
                    out: packed(&name("rec_out"), d, d)?,
                    taps: codes(&name("conv_taps"), CONVOLUTION_WIDTH * d, false)?,
                    conv_bias: integers(&name("conv_bias"), d)?,
                    gate_bias: integers(&name("gate_bias"), shape.gate_rows())?,
                    rates: codes(&name("decay_rate"), shape.lanes(), true)?,
                }))
            } else {
                let scaled = shape.scaled();
                Mixer::Read(Box::new(Read {
                    query: packed(&name("query"), d, d)?,
                    key: packed(&name("key"), d, d)?,
                    value: packed(&name("value"), d, d)?,
                    null: packed(&name("null"), heads, d)?,
                    out: packed(&name("out"), d, d)?,
                    null_bias: integers(&name("null_bias"), heads)?,
                    age: integers(&name("age"), heads * shape.context)?,
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
            layers.push(Layer {
                mixer,
                gate: packed(&name("gate"), mlp, d)?,
                up: packed(&name("up"), mlp, d)?,
                down: packed(&name("down"), d, mlp)?,
            });
        }
        let embed_spec = spec("embed", shape.vocab, d)?;
        let head = packed("head", shape.vocab, d)?;
        let exp_table = artifact.table_u32("exp")?;
        let silu_table = artifact.table_i32("silu")?;
        let gelu_table = artifact.table_i32("gelu")?;
        let arcosh = if shape.lorentz() {
            let table = artifact.table_u32("arcosh")?;
            if table.len() != ARCOSH_TABLE_LEN || table.windows(2).any(|w| w[0] > w[1]) {
                return Err(format_error("the arcosh table has the wrong size or order"));
            }
            table
        } else {
            Vec::new()
        };
        let activation_len = |step: i32, range: i32| (2usize << (range - step)) + 1;
        if exp_table.len() < 2
            || !(-16..0).contains(&numerics.exp_step_log2)
            || !(-16..0).contains(&numerics.silu_step_log2)
            || !(-16..0).contains(&numerics.gelu_step_log2)
            || !(0..=8).contains(&numerics.silu_range_log2)
            || !(0..=8).contains(&numerics.gelu_range_log2)
            || silu_table.len() != activation_len(numerics.silu_step_log2, numerics.silu_range_log2)
            || gelu_table.len() != activation_len(numerics.gelu_step_log2, numerics.gelu_range_log2)
            || !(1..=1i64 << 31).contains(&numerics.score_scale_q30)
        {
            return Err(format_error(
                "sealed tables do not match the declared numerics",
            ));
        }
        Ok(Self {
            embed_nibbles: artifact.section(embed_spec.nibbles).to_vec(),
            embed_scales: artifact.section(embed_spec.scales).to_vec(),
            embed_spec,
            header: artifact.header.clone(),
            sha256: artifact.sha256.clone(),
            backend: Backend::detect(),
            threads: 1,
            pool: None,
            shape,
            numerics,
            head,
            layers,
            exp_table,
            silu_table,
            gelu_table,
            arcosh,
        })
    }

    pub fn shape(&self) -> &StackShape {
        &self.shape
    }

    /// SHA-256 of the artifact bytes.
    pub fn artifact_sha256(&self) -> &str {
        &self.sha256
    }

    /// The sealed exp table (`round(2^31 exp(-i 2^exp_step_log2))`) and its step.
    pub fn exp_table(&self) -> (&[u32], i32) {
        (&self.exp_table, self.numerics.exp_step_log2)
    }

    pub fn source(&self) -> &serde_json::Value {
        &self.header.source
    }

    pub fn backend(&self) -> Backend {
        self.backend
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    /// Use `threads` worker threads (1: none) for large matrix products;
    /// every thread count computes the same integers.
    pub fn set_threads(&mut self, threads: usize) -> Result<()> {
        if threads == 0 {
            return Err(invalid("threads must be positive"));
        }
        self.pool = if threads == 1 {
            None
        } else {
            Some(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .map_err(|e| invalid(format!("thread pool: {e}")))?,
            )
        };
        self.threads = threads;
        Ok(())
    }

    /// Select a backend; every backend computes the same integers.
    pub fn set_backend(&mut self, backend: Backend) -> Result<()> {
        if !backend.available() {
            return Err(invalid(format!(
                "the {} backend is not available on this CPU",
                backend.name()
            )));
        }
        self.backend = backend;
        Ok(())
    }

    /// A fresh session: zero recurrence states and empty read caches.
    pub fn session(&self) -> StackSession<'_> {
        let s = &self.shape;
        let states = s
            .pattern
            .bytes()
            .map(|kind| {
                if kind == b'r' {
                    LayerState::Recurrence {
                        state: vec![0; s.width],
                        history: vec![0; (CONVOLUTION_WIDTH - 1) * s.width],
                    }
                } else {
                    LayerState::Read {
                        keys: Vec::with_capacity(s.context * s.width),
                        values: Vec::with_capacity(s.context * s.width),
                        lifts: Vec::new(),
                        bounds: Bounds::default(),
                    }
                }
            })
            .collect();
        StackSession {
            model: self,
            position: 0,
            states,
            buffers: Buffers {
                x: vec![0; s.width],
                norm: Act16::default(),
                act: Act16::default(),
                tables: Tables::default(),
                scratch: Vec::new(),
                wide: Vec::new(),
                branches: vec![0; 2 * s.width],
                gate_out: vec![0; s.gate_rows()],
                q: vec![0; s.width],
                k: vec![0; s.width],
                v: vec![0; s.width],
                null: vec![0; s.heads],
                scores: Vec::with_capacity(s.context),
                weights: Vec::with_capacity(s.context),
                mix: Vec::new(),
                mix64: Vec::new(),
                proj: vec![0; s.width],
                gate: vec![0; s.mlp],
                up: vec![0; s.mlp],
                logits: vec![0; s.vocab],
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
        /// `[position][width]` at exponent -16.
        keys: Vec<i32>,
        values: Vec<i32>,
        /// Lorentz key lifts `[position][head]` at exponent -32.
        lifts: Vec<u64>,
        bounds: Bounds,
    },
}

/// The largest key and value magnitudes a read has stored. They prove when
/// its sums fit 64 bits, so the exact 64-bit loops can replace the 128-bit
/// ones without changing any result.
#[derive(Clone, Copy, Debug, Default)]
struct Bounds {
    key: u64,
    value: u64,
}

fn max_abs(values: &[i32]) -> u64 {
    values
        .iter()
        .map(|v| u64::from(v.unsigned_abs()))
        .max()
        .unwrap_or(0)
}

/// Whether `terms` products each at most `a b` in magnitude, and all their
/// partial sums, fit `i64`.
fn fits_i64(terms: u64, a: u64, b: u64) -> bool {
    u128::from(terms) * u128::from(a) * u128::from(b) <= i64::MAX as u128
}

struct Buffers {
    x: Vec<i32>,
    norm: Act16,
    act: Act16,
    tables: Tables,
    scratch: Vec<i64>,
    wide: Vec<i64>,
    branches: Vec<i32>,
    gate_out: Vec<i32>,
    q: Vec<i32>,
    k: Vec<i32>,
    v: Vec<i32>,
    null: Vec<i32>,
    scores: Vec<i64>,
    weights: Vec<u64>,
    mix: Vec<i128>,
    mix64: Vec<i64>,
    proj: Vec<i32>,
    gate: Vec<i32>,
    up: Vec<i32>,
    logits: Vec<i32>,
}

/// Incremental decoding of one stream of at most `context` positions.
pub struct StackSession<'m> {
    model: &'m StackModel,
    position: usize,
    states: Vec<LayerState>,
    buffers: Buffers,
}

impl StackSession<'_> {
    pub fn position(&self) -> usize {
        self.position
    }

    /// Start a new stream.
    pub fn reset(&mut self) {
        self.position = 0;
        for state in &mut self.states {
            match state {
                LayerState::Recurrence { state, history } => {
                    state.fill(0);
                    history.fill(0);
                }
                LayerState::Read {
                    keys,
                    values,
                    lifts,
                    bounds,
                } => {
                    keys.clear();
                    values.clear();
                    lifts.clear();
                    *bounds = Bounds::default();
                }
            }
        }
    }

    /// The logits of the last step (value `v` means `v * 2^-16`).
    pub fn logits(&self) -> &[i32] {
        &self.buffers.logits
    }

    /// Feed one token at the next position; returns the next-token logits
    /// (value `v` means `v * 2^-16`).
    pub fn step(&mut self, token: u32) -> Result<&[i32]> {
        let model = self.model;
        match &model.pool {
            Some(pool) => pool.install(|| self.advance(token))?,
            None => self.advance(token)?,
        }
        Ok(&self.buffers.logits)
    }

    fn advance(&mut self, token: u32) -> Result<()> {
        let model = self.model;
        let parallel = model.pool.is_some();
        let (s, n) = (&model.shape, &model.numerics);
        if token as usize >= s.vocab {
            return Err(invalid(format!("token {token} outside the vocabulary")));
        }
        if self.position >= s.context {
            return Err(invalid("the session is full (context)"));
        }
        let b = &mut self.buffers;
        dequant_row(
            &MatrixView {
                rows: model.embed_spec.rows,
                cols: model.embed_spec.cols,
                exp_base: model.embed_spec.exp_base,
                nibbles: &model.embed_nibbles,
                scales: &model.embed_scales,
            },
            token as usize,
            &mut b.x,
            RESIDUAL_EXP,
        );
        for (layer, state) in model.layers.iter().zip(self.states.iter_mut()) {
            rms_norm(&b.x, RESIDUAL_EXP, n.rms_eps, &mut b.scratch, &mut b.norm);
            b.tables.build(&b.norm.values)?;
            match (&layer.mixer, state) {
                (Mixer::Recurrence(r), LayerState::Recurrence { state, history }) => {
                    recurrence(model, r, state, history, b, parallel)?
                }
                (
                    Mixer::Read(r),
                    LayerState::Read {
                        keys,
                        values,
                        lifts,
                        bounds,
                    },
                ) => read(
                    model,
                    r,
                    (keys, values, lifts, bounds),
                    self.position,
                    b,
                    parallel,
                )?,
                _ => return Err(invalid("layer state does not match its mixer")),
            }
            for (x, p) in b.x.iter_mut().zip(&b.proj) {
                *x = x.saturating_add(*p);
            }
            // MLP block.
            rms_norm(&b.x, RESIDUAL_EXP, n.rms_eps, &mut b.scratch, &mut b.norm);
            b.tables.build(&b.norm.values)?;
            gemv(
                &layer.gate,
                model.backend,
                parallel,
                &b.tables,
                b.norm.exp,
                &mut b.gate,
                RESIDUAL_EXP,
            )?;
            gemv(
                &layer.up,
                model.backend,
                parallel,
                &b.tables,
                b.norm.exp,
                &mut b.up,
                RESIDUAL_EXP,
            )?;
            b.scratch.clear();
            for (g, u) in b.gate.iter().zip(&b.up) {
                let a = silu(*g, &model.silu_table, n.silu_step_log2, n.silu_range_log2);
                b.scratch.push(i64::from(a) * i64::from(*u));
            }
            quantize16(&b.scratch, 2 * RESIDUAL_EXP, &mut b.act);
            b.tables.build(&b.act.values)?;
            gemv(
                &layer.down,
                model.backend,
                parallel,
                &b.tables,
                b.act.exp,
                &mut b.proj,
                RESIDUAL_EXP,
            )?;
            for (x, p) in b.x.iter_mut().zip(&b.proj) {
                *x = x.saturating_add(*p);
            }
        }
        rms_norm(&b.x, RESIDUAL_EXP, n.rms_eps, &mut b.scratch, &mut b.norm);
        b.tables.build(&b.norm.values)?;
        gemv(
            &model.head,
            model.backend,
            parallel,
            &b.tables,
            b.norm.exp,
            &mut b.logits,
            RESIDUAL_EXP,
        )?;
        self.position += 1;
        Ok(())
    }
}

/// One recurrence step from the normalized state (tables built), into `b.proj`.
fn recurrence(
    model: &StackModel,
    r: &Recurrence,
    state: &mut [i64],
    history: &mut [i32],
    b: &mut Buffers,
    parallel: bool,
) -> Result<()> {
    let (s, n) = (&model.shape, &model.numerics);
    let (d, lanes) = (s.width, s.lanes());
    for (matrix, out) in [(&r.input, &mut b.branches), (&r.gates, &mut b.gate_out)] {
        gemv(
            matrix,
            model.backend,
            parallel,
            &b.tables,
            b.norm.exp,
            out,
            RESIDUAL_EXP,
        )?;
    }
    for (g, bias) in b.gate_out.iter_mut().zip(&r.gate_bias) {
        *g = g.saturating_add(*bias);
    }
    let (drive, gate) = b.branches.split_at(d);
    // Causal convolution of the drive: this position and the three before it
    // (zero before the stream starts), at exponent -16.
    b.wide.clear();
    for (i, &now) in drive.iter().enumerate() {
        let mut c = i64::from(r.conv_bias[i]) + grid_apply(i64::from(now), r.taps[i]);
        for back in 1..CONVOLUTION_WIDTH {
            c += grid_apply(i64::from(history[(back - 1) * d + i]), r.taps[back * d + i]);
        }
        b.wide.push(c);
    }
    history.copy_within(0..(CONVOLUTION_WIDTH - 2) * d, d);
    history[..d].copy_from_slice(drive);
    for lane in 0..lanes {
        // Decay gate, decay and the drive's weight, all Q31.
        let opening = sigmoid_q31(
            i64::from(b.gate_out[lane]),
            &model.exp_table,
            n.exp_step_log2,
        );
        let exponent = grid_apply(opening as i64, r.rates[lane]).max(0);
        let lambda = exp_neg(exponent, GATE_EXP, &model.exp_table, n.exp_step_log2);
        let complement = (1u64 << 62).saturating_sub(lambda * lambda);
        let keep = if complement < COMPLEMENT_FLOOR_Q62 {
            KEEP_FLOOR_Q31
        } else {
            isqrt(u128::from(complement)) as u64
        };
        let transition: [i64; 4] = if s.rotation {
            let raw: [i32; 4] = b.gate_out[lanes + 4 * lane..lanes + 4 * lane + 4]
                .try_into()
                .map_err(|_| invalid("rotation gates have the wrong width"))?;
            let square: u128 = raw
                .iter()
                .map(|v| (i128::from(*v) * i128::from(*v)) as u128)
                .sum();
            // |raw| at exponent -32; raw / |raw| at Q30, then times lambda at Q31.
            let norm = isqrt((square + ROTATION_EPSILON) << 32) as i128;
            raw.map(|v| {
                let unit = (i128::from(v) << 46) / norm;
                ((i128::from(lambda) * unit) >> 30) as i64
            })
        } else {
            [lambda as i64, 0, 0, 0]
        };
        let held: [i64; 4] = state[4 * lane..4 * lane + 4]
            .try_into()
            .map_err(|_| invalid("recurrence state has the wrong width"))?;
        let moved = hamilton(transition, held);
        for k in 0..4 {
            // Transport (Q31 times exponent -32) plus drive (Q31 times exponent
            // -16, raised to -63), back to exponent -32.
            let pushed = i128::from(keep) * i128::from(b.wide[4 * lane + k]);
            state[4 * lane + k] = shift_wide(moved[k] + (pushed << 16), 31);
        }
    }
    // Output h gelu(g) at exponent -32, then the output map.
    b.scratch.clear();
    for i in 0..d {
        let h = shift(state[i], RESIDUAL_EXP - STATE_EXP);
        let g = silu(
            gate[i],
            &model.gelu_table,
            n.gelu_step_log2,
            n.gelu_range_log2,
        );
        b.scratch.push(h.saturating_mul(i64::from(g)));
    }
    quantize16(&b.scratch, 2 * RESIDUAL_EXP, &mut b.act);
    b.tables.build(&b.act.values)?;
    gemv(
        &r.out,
        model.backend,
        parallel,
        &b.tables,
        b.act.exp,
        &mut b.proj,
        RESIDUAL_EXP,
    )
}

/// One read step from the normalized state (tables built), into `b.proj`.
#[allow(clippy::too_many_arguments)]
fn read(
    model: &StackModel,
    r: &Read,
    (keys, values, lifts, bounds): (&mut Vec<i32>, &mut Vec<i32>, &mut Vec<u64>, &mut Bounds),
    position: usize,
    b: &mut Buffers,
    parallel: bool,
) -> Result<()> {
    let (s, n) = (&model.shape, &model.numerics);
    let (d, heads, hd) = (s.width, s.heads, s.head_dim());
    let (lorentz, l2) = (s.lorentz(), s.l2());
    for (matrix, out) in [
        (&r.query, &mut b.q),
        (&r.key, &mut b.k),
        (&r.value, &mut b.v),
        (&r.null, &mut b.null),
    ] {
        gemv(
            matrix,
            model.backend,
            parallel,
            &b.tables,
            b.norm.exp,
            out,
            RESIDUAL_EXP,
        )?;
    }
    keys.extend_from_slice(&b.k);
    values.extend_from_slice(&b.v);
    bounds.key = bounds.key.max(max_abs(&b.k));
    bounds.value = bounds.value.max(max_abs(&b.v));
    if lorentz {
        for h in 0..heads {
            lifts.push(lift(&b.k[h * hd..(h + 1) * hd]));
        }
    }
    b.wide.clear();
    b.wide.resize(d, 0);
    for h in 0..heads {
        let query = &b.q[h * hd..(h + 1) * hd];
        let query_lift = if lorentz { lift(query) } else { 0 };
        let null_score = i64::from(b.null[h]) + i64::from(r.null_bias[h]);
        let mut max = null_score;
        b.scores.clear();
        let narrow = fits_i64(hd as u64, max_abs(query), bounds.key);
        for j in 0..=position {
            let key = &keys[j * d + h * hd..j * d + (h + 1) * hd];
            let dot = || -> i128 {
                if narrow {
                    i128::from(
                        query
                            .iter()
                            .zip(key)
                            .map(|(a, b)| i64::from(*a) * i64::from(*b))
                            .sum::<i64>(),
                    )
                } else {
                    query
                        .iter()
                        .zip(key)
                        .map(|(a, b)| i128::from(*a) * i128::from(*b))
                        .sum()
                }
            };
            let score = if l2 {
                // At most 2^44, so the difference and the grid product fit.
                let distance = l2_distance(query, key) as i64;
                let scaled = grid_apply(distance - i64::from(r.offset[h]), r.beta[h]);
                shift(-scaled, SCORE_EXP - DISTANCE_EXP)
            } else if lorentz {
                let distance =
                    lorentz_distance(query_lift, lifts[j * heads + h], dot(), &model.arcosh);
                let scaled = grid_apply(i64::from(distance) - i64::from(r.offset[h]), r.beta[h]);
                shift(-scaled, SCORE_EXP - DISTANCE_EXP)
            } else {
                // Exponent -32 times Q30 is exponent -62.
                shift_wide(
                    dot() * i128::from(n.score_scale_q30),
                    (SCORE_EXP - (2 * RESIDUAL_EXP - 30)) as u32,
                )
            };
            let score = score.saturating_add(i64::from(r.age[h * s.context + position - j]));
            max = max.max(score);
            b.scores.push(score);
        }
        // Softmax over NoRead (value zero) and the positions, weights in Q31.
        let mut total = exp_neg(
            max - null_score,
            SCORE_EXP,
            &model.exp_table,
            n.exp_step_log2,
        );
        b.weights.clear();
        for &score in &b.scores {
            let w = exp_neg(max - score, SCORE_EXP, &model.exp_table, n.exp_step_log2);
            total += w;
            b.weights.push(w);
        }
        b.mix.clear();
        if fits_i64(1, total, bounds.value) {
            // Every partial sum is at most `total` times the largest value.
            b.mix64.clear();
            b.mix64.resize(hd, 0);
            for (j, &w) in b.weights.iter().enumerate() {
                if w == 0 {
                    continue;
                }
                let value = &values[j * d + h * hd..j * d + (h + 1) * hd];
                let w = w as i64;
                for (m, v) in b.mix64.iter_mut().zip(value) {
                    *m += w * i64::from(*v);
                }
            }
            b.mix.extend(b.mix64.iter().map(|&m| i128::from(m)));
        } else {
            b.mix.resize(hd, 0);
            for (j, &w) in b.weights.iter().enumerate() {
                if w == 0 {
                    continue;
                }
                let value = &values[j * d + h * hd..j * d + (h + 1) * hd];
                for (m, v) in b.mix.iter_mut().zip(value) {
                    *m += i128::from(w) * i128::from(*v);
                }
            }
        }
        // Q31-weighted sum over the Q31 total, back to exponent -16.
        let reciprocal = i128::from(((1u128 << 62) / u128::from(total.max(1))) as u64);
        for (slot, m) in b.wide[h * hd..(h + 1) * hd].iter_mut().zip(&b.mix) {
            *slot = shift_wide(m * reciprocal, 62);
        }
    }
    quantize16(&b.wide, RESIDUAL_EXP, &mut b.act);
    b.tables.build(&b.act.values)?;
    gemv(
        &r.out,
        model.backend,
        parallel,
        &b.tables,
        b.act.exp,
        &mut b.proj,
        RESIDUAL_EXP,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernels::{arcosh_grid, grid_encode, ARCOSH_FRACTION_BITS};

    /// A header-only stack artifact, with or without a transport snap record.
    fn header_only(snapped: bool) -> Vec<u8> {
        use crate::format::{Fixed, StackArtifactBuilder};
        let shape = StackShape {
            vocab: 64,
            width: 64,
            heads: 2,
            mlp: 64,
            pattern: "ra".to_owned(),
            read: "lorentz".to_owned(),
            rotation: true,
            context: 8,
        };
        let numerics = StackNumerics {
            rms_eps: Fixed {
                mantissa: 1,
                exp: -48,
            },
            score_scale_q30: 1 << 28,
            exp_step_log2: -8,
            silu_step_log2: -8,
            silu_range_log2: 4,
            gelu_step_log2: -8,
            gelu_range_log2: 4,
        };
        let mut builder = StackArtifactBuilder::new(shape, numerics, serde_json::json!({}))
            .expect("a valid shape");
        if snapped {
            builder.set_transport_snap("icosian".to_owned(), 120, "ab".repeat(32));
        }
        builder.finish().expect("the header writes")
    }

    /// The offline reference parse keeps a snapped artifact, but this engine
    /// refuses it at construction, before reading any matrix. An unsnapped
    /// header-only artifact fails later, on its first missing matrix.
    #[test]
    fn from_artifact_refuses_a_snapped_artifact_from_the_reference_parse() {
        let snapped = StackArtifact::parse_for_reference(header_only(true))
            .expect("the reference parse accepts the snap record");
        let refusal = match StackModel::from_artifact(snapped) {
            Err(error) => error.to_string(),
            Ok(_) => panic!("the D10 engine must refuse a snapped artifact"),
        };
        assert!(refusal.contains("transport_snap=icosian"), "{refusal}");
        let unsnapped = StackArtifact::parse(header_only(false)).expect("a snap-free parse");
        let other = match StackModel::from_artifact(unsnapped) {
            Err(error) => error.to_string(),
            Ok(_) => panic!("a header-only artifact has no matrices"),
        };
        assert!(!other.contains("transport_snap"), "{other}");
    }

    #[test]
    fn hamilton_product_matches_floats() {
        let a = [3i64, -1, 4, 1];
        let b = [5i64, 9, -2, 6];
        let f = |q: [i64; 4]| q.map(|v| v as f64);
        let (x, y) = (f(a), f(b));
        let want = [
            x[0] * y[0] - x[1] * y[1] - x[2] * y[2] - x[3] * y[3],
            x[0] * y[1] + x[1] * y[0] + x[2] * y[3] - x[3] * y[2],
            x[0] * y[2] - x[1] * y[3] + x[2] * y[0] + x[3] * y[1],
            x[0] * y[3] + x[1] * y[2] - x[2] * y[1] + x[3] * y[0],
        ];
        assert_eq!(hamilton(a, b).map(|v| v as f64), want);
    }

    #[test]
    fn grid_codes_scale_by_shifts() {
        for (m, e, negative, value) in [
            (0u8, 4, false, 1000i64),
            (15, 0, true, 123_456),
            (7, -20, false, 1 << 40),
            (3, 10, true, -77),
        ] {
            let code = grid_encode(m, e, negative).expect("in range");
            assert!(grid_valid(code));
            let scalar =
                (16.0 + f64::from(m)) * 2f64.powi(e - 4) * if negative { -1.0 } else { 1.0 };
            let want = scalar * value as f64;
            let got = grid_apply(value, code) as f64;
            assert!(
                (got - want).abs() <= 1.0 + want.abs() * 1e-12,
                "{got} vs {want}"
            );
        }
        assert_eq!(grid_apply(12345, 0), 0);
        assert!(!grid_valid(5));
        assert!(grid_encode(16, 0, false).is_none());
        assert!(grid_encode(0, 64, false).is_none());
    }

    #[test]
    fn lorentz_distances_match_f64() {
        let table: Vec<u32> = (0..ARCOSH_TABLE_LEN)
            .map(|i| {
                let u = arcosh_grid(i) as f64 * 2f64.powi(-(ARCOSH_FRACTION_BITS as i32));
                ((u + (u * (u + 2.0)).sqrt()).ln_1p() * 2f64.powi(24)).round() as u32
            })
            .collect();
        let mut seed = 11u64;
        let mut uniform = || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((seed >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
        };
        let mut worst = 0f64;
        for scale in [0.05, 0.5, 3.0] {
            for _ in 0..200 {
                let q: Vec<f64> = (0..48).map(|_| uniform() * scale).collect();
                let k: Vec<f64> = q.iter().map(|v| v + uniform() * scale * 0.3).collect();
                let code = |v: &[f64]| -> Vec<i32> {
                    v.iter().map(|x| (x * 65536.0).round() as i32).collect()
                };
                let (qc, kc) = (code(&q), code(&k));
                let back =
                    |c: &[i32]| -> Vec<f64> { c.iter().map(|x| f64::from(*x) / 65536.0).collect() };
                let (qd, kd) = (back(&qc), back(&kc));
                let norm = |v: &[f64]| v.iter().map(|x| x * x).sum::<f64>();
                let inner: f64 = qd.iter().zip(&kd).map(|(a, b)| a * b).sum();
                let excess =
                    ((1.0 + norm(&qd)).sqrt() * (1.0 + norm(&kd)).sqrt() - inner - 1.0).max(1e-7);
                let want = (excess + (excess * (excess + 2.0)).sqrt()).ln_1p();
                let dot: i128 = qc
                    .iter()
                    .zip(&kc)
                    .map(|(a, b)| i128::from(*a) * i128::from(*b))
                    .sum();
                let got =
                    f64::from(lorentz_distance(lift(&qc), lift(&kc), dot, &table)) * 2f64.powi(-24);
                worst = worst.max((got - want).abs());
            }
        }
        assert!(worst < 2e-6, "worst absolute distance error {worst}");
    }

    /// The integer L2 distance against `sqrt(max(|q - k|^2, 1e-7))` in f64 on
    /// the same coded points, from near-coincident to far-apart pairs, the
    /// floor, and coordinates at the `i32` limits.
    #[test]
    fn l2_distances_match_f64() {
        let mut seed = 13u64;
        let mut uniform = || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((seed >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
        };
        let want = |qc: &[i32], kc: &[i32]| -> f64 {
            let squared: f64 = qc
                .iter()
                .zip(kc)
                .map(|(a, b)| {
                    let v = (f64::from(*a) - f64::from(*b)) / 65536.0;
                    v * v
                })
                .sum();
            squared.max(1e-7).sqrt()
        };
        let got = |qc: &[i32], kc: &[i32]| l2_distance(qc, kc) as f64 * 2f64.powi(-24);
        let mut worst = 0f64;
        for scale in [1e-4, 0.05, 0.5, 3.0, 1000.0] {
            for _ in 0..200 {
                let q: Vec<f64> = (0..48).map(|_| uniform() * scale).collect();
                let k: Vec<f64> = q.iter().map(|v| v + uniform() * scale * 0.3).collect();
                let code = |v: &[f64]| -> Vec<i32> {
                    v.iter().map(|x| (x * 65536.0).round() as i32).collect()
                };
                let (qc, kc) = (code(&q), code(&k));
                let (w, g) = (want(&qc, &kc), got(&qc, &kc));
                let below = w - g;
                // Above the floor the floor square root loses less than one
                // unit of 2^-24. On it, the integer floor code 429 2^-32 is
                // 1e-7 rounded down, about 1.2e-10 lower: 1.9e-7 in distance.
                let allowed = if w > 1e-7f64.sqrt() {
                    2f64.powi(-24)
                } else {
                    2.5e-7
                };
                assert!((-1e-12..allowed + 1e-12).contains(&below), "{g} vs {w}");
                if w > 0.1 {
                    worst = worst.max(below / w);
                }
            }
        }
        assert!(worst < 1e-6, "worst relative distance error {worst}");
        // Coincident points sit on the floor: sqrt(429 2^-32) at exponent -24.
        let floor = l2_distance(&[7, -3], &[7, -3]);
        assert_eq!(floor, isqrt(429u128 << 16) as u64);
        assert!((floor as f64 * 2f64.powi(-24) - 1e-7f64.sqrt()).abs() < 1e-6);
        // Opposite i32 limits over a 256-wide head: |q - k|^2 = 256 (2^32 - 1)^2.
        let (q, k) = (vec![i32::MAX; 256], vec![i32::MIN; 256]);
        let exact = isqrt((256 * u128::from(u32::MAX) * u128::from(u32::MAX)) << 16) as u64;
        assert_eq!(l2_distance(&q, &k), exact);
        assert!(exact < 1 << 44);
        let relative = (got(&q, &k) - want(&q, &k)).abs() / want(&q, &k);
        assert!(relative < 1e-9, "{relative}");
    }
}
