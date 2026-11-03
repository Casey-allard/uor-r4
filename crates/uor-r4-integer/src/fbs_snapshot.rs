//! The offline SpiralCore/FBS float-trunk snapshot: plain data, with no serving
//! code, so a converter in the native graph can build it without the
//! `serving-wasm` feature (d11-exporter-float-trunk, claude-123).
//!
//! The serving build exports it with `stack::export_fbs_trunk_uorlut01`, and
//! `stack::format` re-exports these types. The snapshot serializes to JSON for
//! the handoff between the two builds; `serde_json` writes each `f32` so that
//! it reads back exactly.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The fixed-point exponent the exporter stores biases and decay offsets at
/// (`stack::kernels::RESIDUAL_EXP`, held equal by a stack test).
pub const FBS_FIXED_EXP: i32 = -16;

/// The exponent the FBS step holds both requantized activations at, the MLP's
/// gated product and the recurrence's output, before `stack_quantize16`
/// (`stack::kernels::PRODUCT_EXP`, held equal by a stack test). The float
/// trainer's served stand-in reads its activations at it (claude-262).
pub const FBS_ACTIVATION_EXP: i32 = -32;

/// Encode a finite scalar as a grid code `±(16 + m) 2^(e - 4)` with
/// `1 <= e + 64 <= 127`: the exporter's form for gains and convolution taps.
/// `stack::encode_grid_code` wraps it.
pub fn grid_code(value: f64, positive: bool) -> Result<i16, String> {
    if !value.is_finite() || (positive && value <= 0.0) {
        return Err("nonfinite or non-positive scalar".to_owned());
    }
    if value.abs() < 1e-18 {
        if positive {
            return Err("zero positive grid scalar".to_owned());
        }
        return Ok(0);
    }
    let abs = value.abs();
    let mut best_code = 0i16;
    let mut best_err = f64::INFINITY;
    for field in 1i16..=127 {
        let e = i32::from(field) - 64;
        let step = 2f64.powi(e - 4);
        for m in 0i16..16 {
            let cand = f64::from(16 + m) * step;
            let err = (abs - cand).abs();
            if err < best_err {
                best_err = err;
                best_code = (field << 4) | m;
            }
        }
    }
    Ok(if value < 0.0 { -best_code } else { best_code })
}

/// Decode a grid code `±(16 + m) 2^(e - 4)` to `f64`.
pub fn grid_value(code: i16) -> f64 {
    if code == 0 {
        return 0.0;
    }
    let c = code.unsigned_abs();
    let e = i32::from(c >> 4) - 64;
    let mag = f64::from(16 + (c & 15)) * 2f64.powi(e - 4);
    if code < 0 {
        -mag
    } else {
        mag
    }
}

/// `v` rounded to fixed point at `2^exp`, as the exporter stores it.
pub fn fixed_value(v: f32, exp: i32) -> f64 {
    (f64::from(v) * 2f64.powi(-exp)).round() * 2f64.powi(exp)
}

/// `v` at Q12 in `i16`, as the exporter writes the writer table, or `None`
/// when it is not finite or does not fit (the exporter refuses both).
pub fn q12_value(v: f32) -> Option<f64> {
    if !v.is_finite() {
        return None;
    }
    i16::try_from((f64::from(v) * 4096.0).round() as i64)
        .ok()
        .map(|k| f64::from(k) / 4096.0)
}

/// Write `bytes` to a new file at `path`, never leaving a partial file there
/// and never replacing one. The bytes go to `<path>.partial` first, which is
/// linked into place only when complete (the link refuses an existing
/// `path`); the partial file is removed either way. For the handoff bins.
pub fn write_new_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut partial = path.as_os_str().to_owned();
    partial.push(".partial");
    let partial = PathBuf::from(partial);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial)?;
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    let linked = written.and_then(|()| std::fs::hard_link(&partial, path));
    let removed = std::fs::remove_file(&partial);
    linked.and(removed)
}

/// SpiralCore/FBS routes of one projection: one `(atom, state)` per output 8-block.
///
/// Output block `o` reads input block `atoms[o]`: its channel `j` is
/// `y[o*8 + j] = ±x[atoms[o]*8 + (code & 7)]`, negated when `code & 8`, where
/// `code` is the `j`-th code of the 64-state Cl(0,6) signed-axis table at
/// `states[o]` (`discrete_d11::spiralcore_signed_axes`). The integer session
/// serves it as a gather and a sign (`stack_fbs_gather`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsRoutes {
    /// Input 8-blocks the atoms index.
    pub in_blocks: usize,
    /// Output 8-blocks, one route each.
    pub out_blocks: usize,
    /// `out_blocks` input-block indices, each below `in_blocks`.
    pub atoms: Vec<u16>,
    /// `out_blocks` signed-axis states, each below 64.
    pub states: Vec<u8>,
}

/// One recurrence layer of a SpiralCore/FBS trunk snapshot (`d = width`,
/// `m = mlp`, `lanes = d / 4`, `g = lanes + d`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsTrunkLayer {
    /// `d` RMSNorm gain before the recurrence.
    pub rec_norm: Vec<f32>,
    /// `d` RMSNorm gain before the Hamilton feed-forward block.
    pub mlp_norm: Vec<f32>,
    /// `d/8 -> 2d/8` drive and output-gate routes.
    pub rec_in: FbsRoutes,
    /// `d/8 -> g/8` decay-gate and rotation routes.
    pub rec_gate: FbsRoutes,
    /// `d/8 -> d/8` output routes.
    pub rec_out: FbsRoutes,
    /// `d/8 -> m/8` feed-forward gate routes.
    pub mlp_gate: FbsRoutes,
    /// `d/8 -> m/8` feed-forward up routes.
    pub mlp_up: FbsRoutes,
    /// `m/8 -> d/8` feed-forward down routes.
    pub mlp_down: FbsRoutes,
    /// `4 * d` causal convolution taps `[tap][channel]`, tap 0 on the current
    /// position, as the dense trunk's `FloatTrunkRecurrence::conv_taps`.
    pub conv_taps: Vec<f32>,
    /// `d` causal convolution biases.
    pub conv_bias: Vec<f32>,
    /// `g` gate biases (decay-gate bias then rotation bias).
    pub gate_bias: Vec<f32>,
    /// `lanes` learned per-lane decay offsets.
    pub decay: Vec<f32>,
}

/// The float GPU trunk's own output at one single-token position: the
/// comparison side of `fbs-compare`, written by the float converter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsFloatPosition {
    /// The input token.
    pub token: u32,
    /// The float native writer's next token.
    pub next: u32,
    /// `width` final hidden values, after the final RMSNorm gain.
    pub hidden: Vec<f32>,
}

/// The float GPU trunk's outputs at every position of one token sequence: the
/// recurrence's transition, its later convolution taps and the carried state
/// act only after the first position (claude-124).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsFloatSequence {
    /// The input tokens, in order.
    pub tokens: Vec<u32>,
    /// The float native writer's next token at each position.
    pub next: Vec<u32>,
    /// `width` final hidden values at each position.
    pub hidden: Vec<Vec<f32>>,
}

/// How the float GPU trunk was read (claude-124, claude-125).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FbsReadoutMode {
    /// Plain float: nothing snapped, tables at full precision.
    Plain,
    /// The float sheets' D11 export mode: Q12-rounded, icosian-snapped
    /// Hamilton gates; the transition and the tables as in `Plain`.
    Export,
    /// The D11 mode: the gates and the transition snapped to the 120 icosians
    /// directly (no Q12 rounding of the gates, as the integer path), the
    /// tables rounded as exported and the final query rounded to Q12, so the
    /// float trunk computes what the artifact computes.
    D11,
}

/// The float GPU trunk's single-token and sequence outputs, for comparison
/// with its converted snapshot's reference and integer session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsFloatReadout {
    pub vocab: usize,
    pub width: usize,
    /// The mode the trunk was read in.
    pub mode: FbsReadoutMode,
    /// One position per token, in token order.
    pub positions: Vec<FbsFloatPosition>,
    /// Token sequences read position by position.
    pub sequences: Vec<FbsFloatSequence>,
}

impl FbsFloatReadout {
    /// Refuse a readout that does not hold every token of a `vocab`-token,
    /// `width`-wide trunk once, in token order, with a full hidden vector,
    /// that holds a sequence that is empty, longer than `context`, out of the
    /// vocabulary or not fully read, or that was read in the other mode.
    pub fn check(
        &self,
        vocab: usize,
        width: usize,
        context: usize,
        mode: FbsReadoutMode,
    ) -> Result<(), String> {
        if self.sequences.is_empty() {
            return Err("the readout holds no sequences".to_owned());
        }
        for (s, q) in self.sequences.iter().enumerate() {
            let n = q.tokens.len();
            if n == 0
                || n > context
                || q.tokens.iter().any(|&t| t as usize >= vocab)
                || q.next.len() != n
                || q.hidden.len() != n
                || q.hidden.iter().any(|h| h.len() != width)
            {
                return Err(format!(
                    "sequence {s} is not 1 to {context} tokens with an output at each"
                ));
            }
        }
        if self.vocab != vocab || self.width != width {
            return Err("the readout's vocabulary or width differs from the trunk's".to_owned());
        }
        if self.mode != mode {
            return Err(format!("the readout was read in mode {:?}", self.mode));
        }
        if self.positions.len() != vocab {
            return Err(format!(
                "{} positions for a vocabulary of {vocab}",
                self.positions.len()
            ));
        }
        match self
            .positions
            .iter()
            .enumerate()
            .find(|(i, p)| p.token as usize != *i || p.hidden.len() != width)
        {
            Some((i, _)) => Err(format!("position {i} is not token {i} with {width} values")),
            None => Ok(()),
        }
    }
}

/// Offline SpiralCore/FBS trunk snapshot handed to `stack::export_fbs_trunk_uorlut01`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsTrunkSnapshot {
    pub vocab: usize,
    pub width: usize,
    pub mlp: usize,
    pub context: usize,
    pub rms_eps: f64,
    /// Cells the native vector head keeps (1 to 16).
    pub kept_cells: usize,
    /// `vocab * width` row-major: the input embedding and the writer table.
    pub embed: Vec<f32>,
    /// `width` final RMSNorm gain.
    pub final_norm: Vec<f32>,
    pub layers: Vec<FbsTrunkLayer>,
    /// The induction-keyed Hamming reads (claude-146) the trunk serves; absent on a trunk without
    /// them, so a snapshot written before them still parses (claude-188).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hamming: Option<FbsHammingSnapshot>,
    /// The turn memory (claude-161, claude-193) the trunk was trained with; absent on a trunk
    /// without one, so every snapshot written before it still parses (claude-198).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<FbsMemorySnapshot>,
    /// Whether the trunk was trained with the recurrence's retention reading of its rate table
    /// (the float trainer's `retain_rates`, integer-retention-rates, claude-226). The export then
    /// serves `rec_gates = "geometric-retention"`. False, and not written, for the decay reading,
    /// so every earlier snapshot parses and exports as before.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub retain_rates: bool,
    /// Whether the trunk decodes as the whole table's L1-nearest token, as a model trained with
    /// the `cell` or `nearest` objective does ([`run_readout_nearest`], integer-nearest-readout,
    /// claude-231). The export then serves `head = "native_vector_nearest"`. False, and not
    /// written, for the writer's pick, so every earlier snapshot parses and exports as before.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub readout_nearest: bool,
}

/// The readout a run's `config.json` trained for (integer-nearest-readout, claude-231): its
/// `objective`, where `cell` and `nearest` decode as the whole table's nearest token and `sheet`
/// and `voronoi` as the writer's pick (the float trainer's `PointObjective::decodes_nearest`). A
/// config without an objective is a run from before the objectives, read by its writer. Any other
/// value is refused.
pub fn run_readout_nearest(config: &str) -> Result<bool, String> {
    let value: serde_json::Value =
        serde_json::from_str(config).map_err(|e| format!("the run config: {e}"))?;
    match value.get("objective") {
        None | Some(serde_json::Value::Null) => Ok(false),
        Some(serde_json::Value::String(o)) => match o.as_str() {
            "cell" | "nearest" => Ok(true),
            "sheet" | "voronoi" => Ok(false),
            other => Err(format!(
                "the run config's objective {other:?} is not sheet, voronoi, nearest or cell"
            )),
        },
        Some(other) => Err(format!("the run config's objective {other} is not a name")),
    }
}

/// The longest memory key, and the width a reply key is padded to: the float memory's
/// `MAX_REPLY_KEY` (16 turn ids, two separators, a length and an id).
pub const MEMORY_REPLY_KEY_LEN: usize = 20;
/// The most turn ids an opening key may hold (the float memory's `MAX_KEY`).
pub const MEMORY_MAX_K: usize = 16;
/// Pads a stored key shorter than its row, and joins a reply key's parts; never a token id.
pub const MEMORY_PAD: u32 = u32::MAX;

/// The turn memory of `turn_memory.safetensors` and `turn_memory_reply.safetensors`: exact tables
/// of token ids, and the per-coordinate weights of their moves on the readout point.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsMemorySnapshot {
    /// The turn ids an opening key holds (claude-161's 8).
    pub k: usize,
    /// The training window the keys were cut in (`hamilton_sheet`'s `SHEET_CONTEXT`, 192). A
    /// session keys over blocks of it, whatever its own context (claude-196).
    pub key_window: usize,
    /// The opening table: each key, at most `k` turn ids, and the id it names, in ascending key
    /// order.
    pub opening: Vec<FbsMemoryEntry>,
    /// `width`: the opening move's weight.
    pub opening_weight: Vec<f32>,
    /// The reply table (claude-193), when the memory has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply: Option<FbsReplyTable>,
}

/// One memory table entry: a key of token ids (a reply key joins its parts with `u32::MAX`), and
/// the token id it names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsMemoryEntry {
    pub key: Vec<u32>,
    pub value: u32,
}

/// The reply table: keys `turn ids, u32::MAX, reply length, u32::MAX, last reply id` in ascending
/// order, and the reply move's weight.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsReplyTable {
    /// The most turn ids a reply key holds: the float trainer's `TURN_KEY` (8), which does not
    /// follow the opening table's `k`.
    pub turn: usize,
    pub entries: Vec<FbsMemoryEntry>,
    /// `width`: the reply move's weight.
    pub weight: Vec<f32>,
}

/// The Hamming reads of claude-146's `hamming_read.safetensors`: one read per depth.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsHammingSnapshot {
    /// Heads per read: the first half copy heads, the rest type heads, each matching on its own
    /// `width / heads` coordinates.
    pub heads: usize,
    /// Each read's depth, in the float trunk's `HAMMING_DEPTHS` order: a read after that many
    /// layers, inside the stream, or on the final norm's output at the layer count.
    pub depths: Vec<usize>,
    /// One read per depth, in the same order.
    pub reads: Vec<FbsHammingRead>,
}

/// One Hamming read's learned parameters, as `hamming_read.safetensors` stores them under
/// `d{depth}.{name}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsHammingRead {
    /// `width / 4` quaternions `[w, x, y, z]`, row-major: the type heads' query rotation.
    pub query_rotation: Vec<f32>,
    /// `width / 4` quaternions: the key rotation, which the copy heads' queries share.
    pub key_rotation: Vec<f32>,
    /// `width / 4` quaternions: the type heads' value rotation.
    pub value_rotation: Vec<f32>,
    /// `heads` margins, in bits.
    pub margin: Vec<f32>,
    /// `2 * width`: row 0 the copy heads' weights `w_c`, row 1 the type heads' `w_y`.
    pub weight: Vec<f32>,
}

/// The schema a [`HammingTrace`] carries.
pub const HAMMING_TRACE_SCHEMA: &str = "uor-r4.hamming-trace/1";

/// What each Hamming read chose, per window, read and head (claude-235, the item's step 5). The
/// float D11 readout writes one (`fbs-snapshot --hamming-trace`), the integer session another
/// (`fbs-toks --hamming-trace`), and `fbs-toks --hamming-trace-compare` holds the two side by side.
/// A head matches on one 64-bit word, so every code is one `u64`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingTrace {
    /// [`HAMMING_TRACE_SCHEMA`].
    pub schema: String,
    /// Who wrote it: `float-d11` or `integer`.
    pub side: String,
    /// The float arm (`d11`, `d11-tie`, `d11-rounded`, `d11-tie-rounded`), or `integer`.
    pub arm: String,
    /// Heads per read: the first `heads / 2` are copy heads, the rest type heads.
    pub heads: usize,
    /// The most positions a head keeps (`HAMMING_KEPT`).
    pub kept: usize,
    /// One entry per window read, in the order read.
    pub windows: Vec<HammingTraceWindow>,
}

/// One window's trace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingTraceWindow {
    /// The window's token ids.
    pub ids: Vec<u32>,
    /// Each read in the order the forward runs them: the stream read, then the readout read.
    pub reads: Vec<HammingTraceRead>,
}

/// One read's choices over one window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingTraceRead {
    /// The read's depth: the layers before it, or the layer count for the readout read.
    pub depth: usize,
    /// One entry per head, in head order.
    pub heads: Vec<HammingTraceHead>,
}

/// One head's codes and choices at every position of one window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingTraceHead {
    /// Each position's query code: the head's 64 coordinates' sign bits, a coordinate at or above
    /// 0 a 1, bit `i` the head's coordinate `i` (as `pack_signs` packs them).
    pub query: Vec<u64>,
    /// Each position's key code, packed the same way. Position 0's names no candidate.
    pub key: Vec<u64>,
    /// Each position's kept positions: real candidates only (`1 ≤ s < t`, fewer than `kept`
    /// when `t ≤ kept`), in no set order.
    pub kept: Vec<Vec<u32>>,
    /// Each position's query coordinates as `f32::to_bits`, where a float side asks for them
    /// (claude-251's decision trace); empty, and not written, otherwise.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query_f32: Vec<Vec<u32>>,
    /// Each position's key coordinates, the same way.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_f32: Vec<Vec<u32>>,
}

/// The schema a [`DecisionTrace`] carries.
pub const DECISION_TRACE_SCHEMA: &str = "uor-r4.decision-trace/1";
/// The schema a [`DecisionForce`] carries.
pub const DECISION_FORCE_SCHEMA: &str = "uor-r4.decision-force/1";

/// Every discrete decision of claude-247's five kinds that one side's teacher-forced forward
/// makes, per window and step (integer-nearest-readout, Q5's trace). Each step holds them in the
/// forward's order: each layer's rate buckets in lane order and then the stream read after it,
/// the readout read, the turn memory's lookup, and the nearest. `fbs-toks --forced
/// --decision-trace` writes R's (claude-249).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionTrace {
    /// [`DECISION_TRACE_SCHEMA`].
    pub schema: String,
    /// `R`, the raw integer export; `F` or `T`, the float sides.
    pub side: String,
    pub layers: usize,
    /// Lanes per layer, one rate bucket each.
    pub lanes: usize,
    /// Heads per read: the first `heads / 2` are copy heads, the rest type heads.
    pub heads: usize,
    /// The most positions a head keeps.
    pub kept: usize,
    /// One entry per window run, in the order run.
    pub windows: Vec<DecisionWindow>,
}

/// One window's decisions: a panel row's prompt, then its reference fed one token at a time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionWindow {
    /// The panel row's `position`, as `--forced` writes it.
    pub position: usize,
    /// The token stepped at each step.
    pub ids: Vec<u32>,
    /// The step that reads offset 0, the prompt's last.
    pub reply_start: usize,
    /// One entry per step.
    pub steps: Vec<DecisionStep>,
}

/// One step's decisions, in the forward's order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionStep {
    /// One entry per layer, in layer order.
    pub layers: Vec<DecisionLayer>,
    /// The readout read, on the final norm's output; `None` without one.
    pub readout: Option<DecisionRead>,
    /// The turn memory's lookup; `None` where none ran.
    pub memory: Option<DecisionMemory>,
    /// The nearest, from `reply_start` on; `None` at the prompt's earlier steps.
    pub nearest: Option<DecisionNearest>,
}

/// One layer's rate buckets and the stream read after it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionLayer {
    /// Each lane's rate score, `clamp(decay + offset, 0, 2^16)` at Q16: the value its bucket is
    /// chosen from. R's; empty, and not written, on a float side.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub score_q16: Vec<i32>,
    /// Each lane's rate score as the float selection kernel reads it, as `f32::to_bits`. A float
    /// side's; empty, and not written, on R.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub score_f32: Vec<u32>,
    /// Each lane's entry of the 16-entry table of λ and keep, as used: the forced entry where
    /// one was forced.
    pub entry: Vec<u8>,
    /// The stream read after this layer, at depth `layer + 1`; `None` where no read runs.
    pub read: Option<DecisionRead>,
}

/// One read's choices at one step.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRead {
    /// The layers before the read, or the layer count for the readout read.
    pub depth: usize,
    /// The copy heads, then the type heads where the read serves them, in head order.
    pub heads: Vec<DecisionHead>,
}

/// One head's codes and kept set at one step's row `t`, as used.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionHead {
    /// The query code at `t`, packed as [`HammingTraceHead::query`].
    pub query: u64,
    /// The key code at `t`: the code of `x_{t−1}`, which every later row reads for candidate
    /// `t`. Row 0's is 0 and names no candidate.
    pub key: u64,
    /// The kept positions at `t`, nearest first, ties to the earlier position.
    pub kept: Vec<u32>,
    /// The query's coordinates at `t` as `f32::to_bits`, which the code's sign bits read. A
    /// float side's; empty, and not written, on R.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query_f32: Vec<u32>,
    /// The key's coordinates at `t`, the same way.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_f32: Vec<u32>,
}

/// The turn memory's exact lookup at one step.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionMemory {
    /// The key: the token ids it was looked up by.
    pub key: Vec<u32>,
    /// The stored entry the key named, or `None` on a miss.
    pub stored: Option<u32>,
    /// Whether the row was the reply's first, the opening table's.
    pub opening: bool,
}

/// The nearest at one step: the whole table's top two by `−L1`, and the query. R writes the
/// `_q12` fields; a float side writes the `_f64` and `_f32` ones (claude-251), and each leaves the
/// other's out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionNearest {
    pub pick: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_q12: Option<i64>,
    /// The pick's score in R's units, `−2^12 · Σ_j |q_j − e_{u,j}|` in f64 over the f32 query and
    /// table, as `f64::to_bits`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_f64: Option<u64>,
    pub runner_up: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_score_q12: Option<i64>,
    /// The runner-up's score, the same way.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_score_f64: Option<u64>,
    /// The query at Q12, so any token `u`'s score `−Σ_j |Q_j − T_{u,j}|` can be computed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query_q12: Vec<i32>,
    /// A float side's query as `f32::to_bits`, so any token's score can be computed the same way.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query_f32: Vec<u32>,
}

/// The decisions to force, per window (claude-247's forced replay, set Φ).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionForce {
    /// [`DECISION_FORCE_SCHEMA`].
    pub schema: String,
    pub windows: Vec<DecisionForceWindow>,
}

/// One window's forced set, which may be empty.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionForceWindow {
    /// The panel row's `position`.
    pub position: usize,
    pub buckets: Vec<ForcedBucket>,
    pub codes: Vec<ForcedCode>,
}

/// A rate bucket forced to `entry` at `step`, `layer` and `lane`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForcedBucket {
    pub step: usize,
    pub layer: usize,
    pub lane: usize,
    pub entry: u8,
}

/// Which code of a head a [`ForcedCode`] replaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeRole {
    /// The row's query code.
    Query,
    /// The step's key code, which every later row reads for its candidate.
    Key,
}

/// A code forced to `code` at `step`, for the read at `depth` and its head `head` (copy heads
/// first, as [`DecisionRead::heads`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForcedCode {
    pub step: usize,
    pub depth: usize,
    pub head: usize,
    pub role: CodeRole,
    pub code: u64,
}

/// How two [`HammingTrace`]s' kept sets compare (claude-235, the item's step 5): a row per read
/// and head kind, in read order, the copy row before the type row.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingTraceComparison {
    pub windows: usize,
    pub rows: Vec<HammingCompareRow>,
    /// The first [`MAX_FAULTS_LISTED`] instrument faults, each with its window, read, head and
    /// position.
    pub faults: Vec<String>,
}

/// The most instrument faults a [`HammingTraceComparison`] lists by place.
pub const MAX_FAULTS_LISTED: usize = 20;

/// One read's sets of one head kind, compared.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingCompareRow {
    /// The read's depth.
    pub depth: usize,
    /// `copy` or `type`.
    pub kind: String,
    /// The sets judged, and of them those equal.
    pub judged: usize,
    pub equal: usize,
    /// Of the judged sets that hold a candidate, those whose nearest member, by each side's own
    /// codes (bit distance, then position), is the same.
    pub top1_judged: usize,
    pub top1_equal: usize,
    /// A judged set that differs, by cause: a tie across the last kept place; a sign flip in
    /// the query alone, in a key alone, or in both; an instrument fault.
    pub ties: usize,
    pub query_flips: usize,
    pub key_flips: usize,
    pub query_and_key_flips: usize,
    pub instrument_faults: usize,
    /// Type-head sets of a later read where an earlier read's sets differ at or before the
    /// position, so its residual carries that difference: counted apart, not judged.
    pub downstream: usize,
    /// Type-head sets the second trace does not hold (its read serves no type heads).
    pub not_served: usize,
    /// Of the judged sets that are real kept sets, the empty ones at rows 0 and 1 included, how
    /// many have their two query codes `d` bits apart, at index `d` (0 to 64): a few bits is a
    /// coordinate near a sign, half the word codes that share nothing.
    pub query_bits: Vec<usize>,
}

/// One side's kept set at row `t`, sorted, or why it cannot be the set that side chose: real
/// candidates are `1 ≤ s < t`, each once, `min(kept, t − 1)` of them, and by the side's own codes
/// no candidate it leaves out is nearer the query than one it keeps (bit distance; the order
/// among equal distances is the side's own). A side whose recorded codes are not the ones it
/// chose with fails here, so its differences are never read as sign flips.
fn kept_set(head: &HammingTraceHead, t: usize, most: usize) -> Result<Vec<u32>, String> {
    let mut set = head.kept[t].to_vec();
    set.sort_unstable();
    let want = most.min(t.saturating_sub(1));
    if set.len() != want {
        return Err(format!("{} kept, not {want}", set.len()));
    }
    if set.windows(2).any(|w| w[0] == w[1]) || set.iter().any(|&s| s == 0 || s as usize >= t) {
        return Err(format!("{set:?} are not distinct positions 1 ≤ s < {t}"));
    }
    let distance = |s: usize| (head.query[t] ^ head.key[s]).count_ones();
    if let Some(place) = set.iter().map(|&s| distance(s as usize)).max() {
        let nearer =
            (1..t).find(|&s| distance(s) < place && set.binary_search(&(s as u32)).is_err());
        if let Some(s) = nearer {
            return Err(format!(
                "a member {place} bits from the query is kept and position {s}, {} bits, is not: these are not the codes the set was chosen with",
                distance(s)
            ));
        }
    }
    Ok(set)
}

/// Compare the `first` trace with the `second` on the same windows (claude-235): normally the
/// float D11 readout's against the integer session's. A record that does not line up (other
/// windows, reads, heads or lengths) is refused; a set that cannot be a kept set, or is not the
/// nearest by its own side's recorded codes, is counted an instrument fault.
///
/// Every copy-head set, and every type-head set of the first read, is judged at every position:
/// their codes come from the table or from the trunk below the read. A later read's type-head
/// set is judged only where every earlier read's sets agree at its position and before it, and
/// counted as downstream elsewhere. A judged set that differs is a tie when both sides have the
/// same query code and the same key code at every position either keeps; anything else is a
/// sign flip.
pub fn compare_hamming_traces(
    first: &HammingTrace,
    second: &HammingTrace,
) -> Result<HammingTraceComparison, String> {
    for trace in [first, second] {
        if trace.schema != HAMMING_TRACE_SCHEMA {
            return Err(format!(
                "the {} trace's schema is {}, not {HAMMING_TRACE_SCHEMA}",
                trace.side, trace.schema
            ));
        }
    }
    let (heads, most) = (first.heads, first.kept);
    if heads == 0 || heads % 2 != 0 || second.heads != heads || second.kept != most {
        return Err(format!(
            "the traces hold {} and {} heads keeping {} and {}: they must agree, an even count",
            first.heads, second.heads, first.kept, second.kept
        ));
    }
    if first.windows.len() != second.windows.len() || first.windows.is_empty() {
        return Err(format!(
            "the traces hold {} and {} windows",
            first.windows.len(),
            second.windows.len()
        ));
    }
    let copy = heads / 2;
    let mut rows: Vec<HammingCompareRow> = first.windows[0]
        .reads
        .iter()
        .flat_map(|r| {
            ["copy", "type"].map(|kind| HammingCompareRow {
                depth: r.depth,
                kind: kind.to_owned(),
                query_bits: vec![0; 65],
                ..Default::default()
            })
        })
        .collect();
    let mut faults = Vec::new();
    for (w, (a, b)) in first.windows.iter().zip(&second.windows).enumerate() {
        let time = a.ids.len();
        let depths = |x: &HammingTraceWindow| x.reads.iter().map(|r| r.depth).collect::<Vec<_>>();
        if a.ids != b.ids
            || depths(a) != depths(b)
            || 2 * a.reads.len() != rows.len()
            || depths(a) != rows.iter().step_by(2).map(|r| r.depth).collect::<Vec<_>>()
        {
            return Err(format!("window {w}: the traces' ids or reads differ"));
        }
        // Whether every earlier read's sets agreed at each position and before it.
        let mut earlier = vec![true; time];
        for (r, (ra, rb)) in a.reads.iter().zip(&b.reads).enumerate() {
            if ra.heads.len() != heads || (rb.heads.len() != heads && rb.heads.len() != copy) {
                return Err(format!(
                    "window {w}, read {r}: {} and {} heads, not {heads}",
                    ra.heads.len(),
                    rb.heads.len()
                ));
            }
            let mut agree = vec![true; time];
            for h in 0..heads {
                let row = &mut rows[2 * r + usize::from(h >= copy)];
                let Some(hb) = rb.heads.get(h) else {
                    row.not_served += time;
                    continue;
                };
                let ha = &ra.heads[h];
                for x in [ha, hb] {
                    if x.query.len() != time || x.key.len() != time || x.kept.len() != time {
                        return Err(format!("window {w}, read {r}, head {h}: the codes and sets do not cover the window's {time} positions"));
                    }
                }
                for t in 0..time {
                    let (sa, sb) = (kept_set(ha, t, most), kept_set(hb, t, most));
                    let same = matches!((&sa, &sb), (Ok(x), Ok(y)) if x == y);
                    agree[t] &= same;
                    if r > 0 && h >= copy && !earlier[t] {
                        row.downstream += 1;
                        continue;
                    }
                    row.judged += 1;
                    let (sa, sb) = match (sa, sb) {
                        (Ok(sa), Ok(sb)) => (sa, sb),
                        (sa, sb) => {
                            row.instrument_faults += 1;
                            let why = sa.err().or(sb.err()).unwrap_or_default();
                            if faults.len() < MAX_FAULTS_LISTED {
                                faults.push(format!(
                                    "window {w}, read {r}, head {h}, position {t}: {why}"
                                ));
                            }
                            continue;
                        }
                    };
                    row.query_bits[(ha.query[t] ^ hb.query[t]).count_ones() as usize] += 1;
                    let distance = |x: &HammingTraceHead, s: u32| {
                        (x.query[t] ^ x.key[s as usize]).count_ones()
                    };
                    if !sa.is_empty() {
                        row.top1_judged += 1;
                        let nearest = |x: &HammingTraceHead, set: &[u32]| {
                            set.iter().copied().min_by_key(|&s| (distance(x, s), s))
                        };
                        row.top1_equal += usize::from(nearest(ha, &sa) == nearest(hb, &sb));
                    }
                    if same {
                        row.equal += 1;
                        continue;
                    }
                    let mut union: Vec<u32> = sa.iter().chain(&sb).copied().collect();
                    union.sort_unstable();
                    union.dedup();
                    let query_same = ha.query[t] == hb.query[t];
                    let keys_same = union
                        .iter()
                        .all(|&s| ha.key[s as usize] == hb.key[s as usize]);
                    match (query_same, keys_same) {
                        // Both sets are the nearest by the same codes on their union, so each
                        // position only one keeps sits at the last kept place of both: a tie.
                        (true, true) => row.ties += 1,
                        (false, true) => row.query_flips += 1,
                        (true, false) => row.key_flips += 1,
                        (false, false) => row.query_and_key_flips += 1,
                    }
                }
            }
            // A read's sets agree at `t` when they agree there and at every earlier position.
            let mut so_far = true;
            for (e, a) in earlier.iter_mut().zip(agree) {
                so_far &= a;
                *e &= so_far;
            }
        }
    }
    Ok(HammingTraceComparison {
        windows: first.windows.len(),
        rows,
        faults,
    })
}

/// The schema a [`DecisionDeciles`] carries.
pub const DECISION_DECILES_SCHEMA: &str = "uor-r4.decision-deciles/1";
/// The schema `fbs-toks --decision-grade` writes.
pub const DECISION_GRADE_SCHEMA: &str = "uor-r4.decision-grade/1";
/// The schema `fbs-toks --decision-check` writes.
pub const DECISION_CHECK_SCHEMA: &str = "uor-r4.decision-check/1";

/// A code's bits, one per coordinate of its head.
const CODE_BITS: usize = 64;

/// A read's head kind: the first `heads / 2` heads are copy heads, the rest type heads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HeadKind {
    Copy,
    Type,
}

/// One of claude-247's families, a set of values that make one kind of decision. A code's
/// value is one of its coordinates, `|v|`; a rate bucket's is its score's distance to the
/// nearest interior boundary ([`bucket_distance`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "lowercase")]
pub enum DecisionFamily {
    /// One read's codes of one head kind in one role.
    Code {
        depth: usize,
        kind: HeadKind,
        role: CodeRole,
    },
    /// One layer's rate buckets.
    Bucket { layer: usize },
}

/// One family's G₁₀: the `k`-th smallest of its `values` values, `k = max(1, ⌊values / 10⌋)`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FamilyDecile {
    pub family: DecisionFamily,
    pub values: usize,
    pub k: usize,
    pub g10: f64,
}

/// The file a reading was taken from, and its sha256.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionSource {
    pub path: String,
    pub sha256: String,
}

/// Every family's G₁₀(φ, P) on one side's unforced trace of a cell ([`decision_deciles`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionDeciles {
    /// [`DECISION_DECILES_SCHEMA`].
    pub schema: String,
    /// The trace's side, `F` or `T`.
    pub side: String,
    /// The trace file the deciles were read from, which the CLI names; none in memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<DecisionSource>,
    /// In family order.
    pub families: Vec<FamilyDecile>,
}

impl DecisionDeciles {
    /// Family `family`'s G₁₀, or why there is none.
    pub fn g10(&self, family: DecisionFamily) -> Result<f64, String> {
        self.families
            .iter()
            .find(|f| f.family == family)
            .map(|f| f.g10)
            .ok_or_else(|| format!("the deciles hold no G₁₀ for {family:?}"))
    }
}

/// A rate score's distance to its nearest interior boundary, `min_{k=1..15} |s − k/16|`: a
/// bucket's value in its family (claude-247).
pub fn bucket_distance(score: f32) -> f64 {
    (1..16)
        .map(|k| (f64::from(score) - f64::from(k) / 16.0).abs())
        .fold(f64::INFINITY, f64::min)
}

/// A trace's schema, side and layer, lane and head counts, before anything in it is read.
fn check_decision_header(trace: &DecisionTrace) -> Result<(), String> {
    if trace.schema != DECISION_TRACE_SCHEMA {
        return Err(format!(
            "the {} trace's schema is {}, not {DECISION_TRACE_SCHEMA}",
            trace.side, trace.schema
        ));
    }
    if !["R", "F", "T"].contains(&trace.side.as_str()) {
        return Err(format!("a trace's side is R, F or T, not {}", trace.side));
    }
    if trace.layers == 0 || trace.lanes == 0 {
        return Err(format!(
            "the {} trace has {} layers and {} lanes: neither is 0",
            trace.side, trace.layers, trace.lanes
        ));
    }
    if trace.heads == 0 || !trace.heads.is_multiple_of(2) {
        return Err(format!(
            "the {} trace has {} heads: an even count, copy heads then type heads",
            trace.side, trace.heads
        ));
    }
    Ok(())
}

/// G₁₀(φ, P) for every family of `trace`, a reference side's unforced trace of a whole cell
/// (claude-247): over every value the trace records, zeros counted, in the side's own f32
/// values, except a key's coordinates at step 0, which name no candidate (claude-254). It
/// refuses a side other than F and T, a value that is not a number, a rate score outside
/// [0, 1], a step without the trace's layer count, and a trace without the values (a float
/// side records each lane's score and each code's 64 coordinates).
pub fn decision_deciles(trace: &DecisionTrace) -> Result<DecisionDeciles, String> {
    check_decision_header(trace)?;
    if trace.side != "F" && trace.side != "T" {
        return Err(format!(
            "G₁₀ is read on a reference side, F or T, not {}",
            trace.side
        ));
    }
    let copy = trace.heads / 2;
    let mut values: BTreeMap<DecisionFamily, Vec<f64>> = BTreeMap::new();
    for window in &trace.windows {
        for (t, step) in window.steps.iter().enumerate() {
            let place = || format!("position {}, step {t}", window.position);
            if step.layers.len() != trace.layers {
                return Err(format!(
                    "{}: {} layers, not {}",
                    place(),
                    step.layers.len(),
                    trace.layers
                ));
            }
            for (l, layer) in step.layers.iter().enumerate() {
                if layer.score_f32.len() != trace.lanes {
                    return Err(format!(
                        "{}, layer {l}: {} rate scores, not {}",
                        place(),
                        layer.score_f32.len(),
                        trace.lanes
                    ));
                }
                let scores = layer.score_f32.iter().map(|&s| f32::from_bits(s));
                if let Some(s) = scores.clone().find(|s| !(0.0..=1.0).contains(s)) {
                    return Err(format!(
                        "{}, layer {l}: rate score {s} lies outside [0, 1]",
                        place()
                    ));
                }
                values
                    .entry(DecisionFamily::Bucket { layer: l })
                    .or_default()
                    .extend(scores.map(bucket_distance));
            }
            let reads = step.layers.iter().filter_map(|l| l.read.as_ref());
            for read in reads.chain(&step.readout) {
                for (h, head) in read.heads.iter().enumerate() {
                    let kind = if h < copy {
                        HeadKind::Copy
                    } else {
                        HeadKind::Type
                    };
                    for (role, bits) in [
                        (CodeRole::Query, &head.query_f32),
                        (CodeRole::Key, &head.key_f32),
                    ] {
                        if role == CodeRole::Key && t == 0 {
                            continue;
                        }
                        if bits.len() != CODE_BITS {
                            return Err(format!(
                                "{}, depth {}, head {h}: a {role:?} code with {} coordinates, not {CODE_BITS}",
                                place(),
                                read.depth,
                                bits.len()
                            ));
                        }
                        let family = DecisionFamily::Code {
                            depth: read.depth,
                            kind,
                            role,
                        };
                        values
                            .entry(family)
                            .or_default()
                            .extend(bits.iter().map(|&b| f64::from(f32::from_bits(b)).abs()));
                    }
                }
            }
        }
    }
    let families = values
        .into_iter()
        .map(|(family, mut v)| {
            if v.iter().any(|x| x.is_nan()) {
                return Err(format!("{family:?} holds a value that is not a number"));
            }
            let k = (v.len() / 10).max(1);
            let (_, &mut g10, _) = v.select_nth_unstable_by(k - 1, f64::total_cmp);
            Ok(FamilyDecile {
                family,
                values: v.len(),
                k,
                g10,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    if families.is_empty() {
        return Err("the trace records no value".to_owned());
    }
    Ok(DecisionDeciles {
        schema: DECISION_DECILES_SCHEMA.to_owned(),
        side: trace.side.clone(),
        trace: None,
        families,
    })
}

/// A side's own nearest table, `width` values a row: R's writer table at Q12, or a float side's
/// in f32 (F's float table; T's is R's divided by 4096, claude-247).
#[derive(Clone, Copy, Debug)]
pub enum NearestTable<'a> {
    Q12 { table: &'a [i16], width: usize },
    F32 { table: &'a [f32], width: usize },
}

/// Row `u` of a `width`-wide table.
fn table_row<T>(table: &[T], width: usize, u: u32) -> Result<&[T], String> {
    if width == 0 || !table.len().is_multiple_of(width) {
        return Err(format!(
            "a table of {} values is not whole rows of {width}",
            table.len()
        ));
    }
    let start = u as usize * width;
    table.get(start..start + width).ok_or_else(|| {
        format!(
            "token {u} lies outside the table's {} rows",
            table.len() / width
        )
    })
}

impl NearestTable<'_> {
    /// Token `u`'s score in R's units from the nearest's query (claude-247): R's
    /// `−Σ_j |Q_j − T_{u,j}|`, or a float side's `−2^12 · Σ_j |q_j − e_{u,j}|` in f64, summed
    /// in coordinate order as the float writer sums it (`in_space_long.rs`, `l1_score_r`).
    pub fn score(&self, nearest: &DecisionNearest, u: u32) -> Result<f64, String> {
        match *self {
            NearestTable::Q12 { table, width } => {
                let row = table_row(table, width, u)?;
                if nearest.query_q12.len() != width {
                    return Err(format!(
                        "a Q12 query of {} values, not the table's {width}",
                        nearest.query_q12.len()
                    ));
                }
                let sum: i64 = nearest
                    .query_q12
                    .iter()
                    .zip(row)
                    .map(|(&q, &e)| (i64::from(q) - i64::from(e)).abs())
                    .sum();
                // Negated as an integer, so a query on its row scores R's 0, not −0.
                Ok((-sum) as f64)
            }
            NearestTable::F32 { table, width } => {
                let row = table_row(table, width, u)?;
                if nearest.query_f32.len() != width {
                    return Err(format!(
                        "an f32 query of {} values, not the table's {width}",
                        nearest.query_f32.len()
                    ));
                }
                let sum: f64 = nearest
                    .query_f32
                    .iter()
                    .zip(row)
                    .map(|(&q, &e)| (f64::from(f32::from_bits(q)) - f64::from(e)).abs())
                    .sum();
                Ok(-4096.0 * sum)
            }
        }
    }

    /// How far, in R's units, a side's pick `u` may score below token `w` on this table and still
    /// be its nearest (claude-256). R picks on exact integers: 0. A float side picks on the GPU's
    /// f32 mean of `width` displacements (`in_space_long.rs`, `nearest_scores`), and each such
    /// mean is within `γ_n = n·u / (1 − n·u)` of its exact value, with `n` the width and `u =
    /// 2^−24`: one rounding a displacement, `n − 1` for the sum, and an exact division by a power
    /// of 2. So `s(w) − s(u) ≤ γ_n · (|s(u)| + |s(w)|)`; the same `γ_n` at `u = 2^−53` covers this
    /// f64 recomputation.
    pub fn behind_bound(&self, nearest: &DecisionNearest, u: u32, w: u32) -> Result<f64, String> {
        match *self {
            NearestTable::Q12 { .. } => Ok(0.0),
            NearestTable::F32 { width, .. } => {
                let gamma = |unit: f64| {
                    let nu = width as f64 * unit;
                    nu / (1.0 - nu)
                };
                let g = gamma(2f64.powi(-24)) + gamma(2f64.powi(-53));
                Ok(g * (self.score(nearest, u)?.abs() + self.score(nearest, w)?.abs()))
            }
        }
    }

    /// Whether `side`'s recorded pick and runner-up scores are this table's, bit for bit, and
    /// two different tokens. On R the pick is the better of the two (a tie goes to the smaller
    /// id). A float side picks on the GPU's f32 scores, which these f64 scores recompute, so at
    /// a near tie its pick may score below its runner-up here, by at most [`Self::behind_bound`]
    /// (claude-256); beyond it, the pick is not its own query's nearest and is refused.
    fn check_nearest(&self, side: &str, nearest: &DecisionNearest) -> Result<(), String> {
        let recorded = match (self, side) {
            (NearestTable::Q12 { .. }, "R") => (
                nearest.score_q12.map(|s| s as f64),
                nearest.runner_score_q12.map(|s| s as f64),
            ),
            (NearestTable::F32 { .. }, "F" | "T") => (
                nearest.score_f64.map(f64::from_bits),
                nearest.runner_score_f64.map(f64::from_bits),
            ),
            _ => return Err(format!("side {side} is scored on its own kind of table")),
        };
        let (Some(pick), Some(runner)) = recorded else {
            return Err(format!("side {side}'s nearest records no scores"));
        };
        let scores = (
            self.score(nearest, nearest.pick)?,
            self.score(nearest, nearest.runner_up)?,
        );
        if scores.0.to_bits() != pick.to_bits() || scores.1.to_bits() != runner.to_bits() {
            return Err(format!(
                "side {side}'s table scores its pick {} at {} and its runner-up {} at {}, not the recorded {pick} and {runner}",
                nearest.pick, scores.0, nearest.runner_up, scores.1
            ));
        }
        let ordered = match side {
            "R" => pick > runner || (pick == runner && nearest.pick < nearest.runner_up),
            _ => {
                pick >= runner
                    || runner - pick
                        <= self.behind_bound(nearest, nearest.pick, nearest.runner_up)?
            }
        };
        if nearest.pick == nearest.runner_up || !ordered {
            return Err(format!(
                "side {side}'s pick {} ({pick}) is not ahead of its runner-up {} ({runner})",
                nearest.pick, nearest.runner_up
            ));
        }
        Ok(())
    }
}

/// One side of a grade: its trace and its own nearest table.
#[derive(Clone, Copy, Debug)]
pub struct GradeSide<'a> {
    pub trace: &'a DecisionTrace,
    pub table: NearestTable<'a>,
}

/// How one pass of claude-247's forced replay ends ([`grade_pass`]): at the first decision in
/// p's causal prefix where the other side differs from the reference, or at p's nearest.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum PassOutcome {
    /// A rate bucket differs first. `force` holds the reference's entry, to be forced next;
    /// `value` is `|s_P − b|`, b the boundary between adjacent entries, and `None` for entries
    /// two or more apart, never a near tie.
    Bucket {
        force: ForcedBucket,
        other_entry: u8,
        value: Option<f64>,
        g10: f64,
        near_tie: bool,
    },
    /// A code differs first. `force` holds the reference's code, to be forced next; `value` is
    /// the largest `|v_P|` over the bits that differ.
    Code {
        force: ForcedCode,
        other_code: u64,
        value: f64,
        g10: f64,
        near_tie: bool,
    },
    /// A kept set differs first: never a near tie, and the step is not accounted for.
    Kept {
        step: usize,
        depth: usize,
        head: usize,
        reference: Vec<u32>,
        other: Vec<u32>,
    },
    /// A turn-memory lookup differs first: never a near tie, and the step is not accounted for.
    Memory {
        step: usize,
        reference: DecisionMemory,
        other: DecisionMemory,
    },
    /// Nothing differs before p's nearest. For the reference's pick `u` and the other's `w`,
    /// `reference_margin` is `s_P(u) − s_P(w)` and `other_margin` is `s_Q(w) − s_Q(u)`, in R's
    /// units, both 0 for equal picks. `near_tie` holds when the picks are equal or both margins
    /// are at most `g10`, the cell's sealed G₁₀.
    Nearest {
        reference_pick: u32,
        other_pick: u32,
        reference_margin: f64,
        other_margin: f64,
        g10: i64,
        near_tie: bool,
    },
}

/// The window at `position`, the trace's only one there.
fn window_at(trace: &DecisionTrace, position: usize) -> Result<&DecisionWindow, String> {
    let mut found = trace.windows.iter().filter(|w| w.position == position);
    match (found.next(), found.next()) {
        (Some(w), None) => Ok(w),
        (None, _) => Err(format!(
            "the {} trace has no window at position {position}",
            trace.side
        )),
        (Some(_), Some(_)) => Err(format!(
            "the {} trace has two windows at position {position}",
            trace.side
        )),
    }
}

/// The entry lane `lane`'s score selects: R's `min(s_q16 >> 12, 15)` from a score in
/// `[0, 2^16]`, a float side's `rate_index` from a score in `[0, 1]`.
fn selected_entry(side: &str, layer: &DecisionLayer, lane: usize) -> Result<u8, String> {
    if side == "R" {
        let &s = layer.score_q16.get(lane).ok_or("rate score is missing")?;
        if !(0..=1 << 16).contains(&s) {
            return Err(format!("rate score {s} lies outside [0, 2^16]"));
        }
        Ok(((s >> 12) as u8).min(15))
    } else {
        let &bits = layer.score_f32.get(lane).ok_or("rate score is missing")?;
        let s = f32::from_bits(bits);
        if !(0.0..=1.0).contains(&s) {
            return Err(format!("rate score {s} lies outside [0, 1]"));
        }
        Ok(((s * 16.0) as u8).min(15))
    }
}

/// A head's code in `role`, with its coordinates as `f32::to_bits` (none on R).
fn head_code(head: &DecisionHead, role: CodeRole) -> (u64, &[u32]) {
    match role {
        CodeRole::Query => (head.query, &head.query_f32),
        CodeRole::Key => (head.key, &head.key_f32),
    }
}

/// Whether `code` is the sign packing of its 64 coordinates `bits`, as [`pack_signs_f32`]
/// packs them.
fn signs_to(code: u64, bits: &[u32]) -> Result<(), String> {
    if bits.len() != CODE_BITS {
        return Err(format!(
            "code has {} coordinates, not {CODE_BITS}",
            bits.len()
        ));
    }
    let coordinates: Vec<f32> = bits.iter().map(|&b| f32::from_bits(b)).collect();
    if coordinates.iter().any(|c| c.is_nan()) {
        return Err("code has a coordinate that is not a number".to_owned());
    }
    let packed = pack_signs_f32(&coordinates)[0];
    if packed != code {
        return Err(format!(
            "code {code} is not its coordinates' signs, {packed}"
        ));
    }
    Ok(())
}

/// One pass's walk through p's causal prefix in the forward's order: the forced set Φ not yet
/// met, and the families' G₁₀. Every decision it compares is first checked on both sides: the
/// reference's against its own score or coordinates, the other's the same way or, where it is
/// in Φ, against the forced value.
struct PrefixWalk<'a> {
    reference: &'a str,
    other: &'a str,
    copy: usize,
    deciles: &'a DecisionDeciles,
    buckets: BTreeMap<(usize, usize, usize), u8>,
    codes: BTreeMap<(usize, usize, usize, CodeRole), u64>,
}

impl<'a> PrefixWalk<'a> {
    fn new(
        reference: &'a DecisionTrace,
        other: &'a DecisionTrace,
        deciles: &'a DecisionDeciles,
        force: &DecisionForceWindow,
    ) -> Result<Self, String> {
        let mut buckets = BTreeMap::new();
        for b in &force.buckets {
            if buckets.insert((b.step, b.layer, b.lane), b.entry).is_some() {
                return Err(format!("Φ forces {b:?}'s bucket twice (I-Force)"));
            }
        }
        let mut codes = BTreeMap::new();
        for c in &force.codes {
            if codes
                .insert((c.step, c.depth, c.head, c.role), c.code)
                .is_some()
            {
                return Err(format!("Φ forces {c:?}'s code twice (I-Force)"));
            }
        }
        Ok(Self {
            reference: &reference.side,
            other: &other.side,
            copy: reference.heads / 2,
            deciles,
            buckets,
            codes,
        })
    }

    /// The rate bucket of lane `lane` of layer `l` at step `t`.
    fn bucket(
        &mut self,
        t: usize,
        l: usize,
        lane: usize,
        p: &DecisionLayer,
        q: &DecisionLayer,
    ) -> Result<Option<PassOutcome>, String> {
        let place = || format!("step {t}, layer {l}, lane {lane}");
        let (ep, eq) = (p.entry[lane], q.entry[lane]);
        let sp = selected_entry(self.reference, p, lane)
            .map_err(|e| format!("{}: the reference's {e}", place()))?;
        if sp != ep {
            return Err(format!(
                "{}: the reference's entry {ep} is not the {sp} its score selects",
                place()
            ));
        }
        match self.buckets.remove(&(t, l, lane)) {
            Some(forced) if forced != ep => {
                return Err(format!(
                    "{}: Φ forces entry {forced}, not the reference's {ep} (I-Force)",
                    place()
                ))
            }
            Some(forced) if eq != forced => {
                return Err(format!(
                    "{}: the other side took entry {eq}, not the forced {forced} (I-Force)",
                    place()
                ))
            }
            Some(_) => {}
            None => {
                let sq = selected_entry(self.other, q, lane)
                    .map_err(|e| format!("{}: the other side's {e}", place()))?;
                if sq != eq {
                    return Err(format!(
                        "{}: the other side's entry {eq} is not the {sq} its score selects, and Φ does not force it",
                        place()
                    ));
                }
            }
        }
        if ep == eq {
            return Ok(None);
        }
        let s = f64::from(f32::from_bits(p.score_f32[lane]));
        let value = (ep.abs_diff(eq) == 1).then(|| (s - f64::from(ep.max(eq)) / 16.0).abs());
        let g10 = self.deciles.g10(DecisionFamily::Bucket { layer: l })?;
        Ok(Some(PassOutcome::Bucket {
            force: ForcedBucket {
                step: t,
                layer: l,
                lane,
                entry: ep,
            },
            other_entry: eq,
            value,
            g10,
            near_tie: value.is_some_and(|v| v <= g10),
        }))
    }

    /// Head `h`'s code in `role` for the read at `depth`, at step `t`.
    fn code(
        &mut self,
        t: usize,
        depth: usize,
        h: usize,
        role: CodeRole,
        p: &DecisionHead,
        q: &DecisionHead,
    ) -> Result<Option<PassOutcome>, String> {
        let place = || format!("step {t}, depth {depth}, head {h}, {role:?}");
        let ((cp, vp), (cq, vq)) = (head_code(p, role), head_code(q, role));
        signs_to(cp, vp).map_err(|e| format!("{}: the reference's {e}", place()))?;
        match self.codes.remove(&(t, depth, h, role)) {
            Some(forced) if forced != cp => {
                return Err(format!(
                    "{}: Φ forces code {forced}, not the reference's {cp} (I-Force)",
                    place()
                ))
            }
            Some(forced) if cq != forced => {
                return Err(format!(
                    "{}: the other side took code {cq}, not the forced {forced} (I-Force)",
                    place()
                ))
            }
            Some(_) => {}
            // R records no coordinates, so its codes are checked only where they differ from
            // a float side's, by the reference's coordinates.
            None if self.other == "R" => {}
            None => signs_to(cq, vq).map_err(|e| {
                format!("{}: the other side's {e}, and Φ does not force it", place())
            })?,
        }
        if cp == cq {
            return Ok(None);
        }
        let value = (0..CODE_BITS)
            .filter(|&i| (cp ^ cq) >> i & 1 == 1)
            .map(|i| f64::from(f32::from_bits(vp[i])).abs())
            .fold(0.0, f64::max);
        let kind = if h < self.copy {
            HeadKind::Copy
        } else {
            HeadKind::Type
        };
        let g10 = self
            .deciles
            .g10(DecisionFamily::Code { depth, kind, role })?;
        Ok(Some(PassOutcome::Code {
            force: ForcedCode {
                step: t,
                depth,
                head: h,
                role,
                code: cp,
            },
            other_code: cq,
            value,
            g10,
            near_tie: value <= g10,
        }))
    }

    /// `outcome`, once every decision in Φ was met before it. A forced decision the walk never
    /// met lies outside p's prefix or past its first difference, where no replay forces one.
    fn end(&self, outcome: PassOutcome) -> Result<PassOutcome, String> {
        if !self.buckets.is_empty() || !self.codes.is_empty() {
            return Err(format!(
                "Φ forces decisions outside p's prefix or past its first difference (I-Force): buckets (step, layer, lane) {:?}, codes (step, depth, head, role) {:?}",
                self.buckets.keys().collect::<Vec<_>>(),
                self.codes.keys().collect::<Vec<_>>()
            ));
        }
        Ok(outcome)
    }
}

/// A read's depth, once both sides hold it at `depth` with the same heads: all of them, or the
/// copy heads alone.
fn aligned_read(
    p: &DecisionRead,
    q: &DecisionRead,
    depth: usize,
    heads: usize,
    place: &str,
) -> Result<usize, String> {
    if p.depth != depth || q.depth != depth {
        return Err(format!(
            "{place}: reads at depths {} and {}, not {depth} (I-Align)",
            p.depth, q.depth
        ));
    }
    if p.heads.len() != q.heads.len() || ![heads, heads / 2].contains(&p.heads.len()) {
        return Err(format!(
            "{place}: reads with {} and {} heads, not the same {heads} or {} (I-Align)",
            p.heads.len(),
            q.heads.len(),
            heads / 2
        ));
    }
    Ok(depth)
}

/// One pass of claude-247's forced replay at the nearest of step `step` of the window at
/// `position`, as claude-254 orders p's causal prefix: the reference side P (F or T, unforced)
/// against the other side Q (F, T or R), run with every decision of `force` (Φ) forced to P's
/// value.
///
/// The walk follows the forward's order, step by step from the window's start to p: each layer's
/// rate buckets in lane order, then the stream read after it, each head's query code, key code
/// (from step 1) and kept set; then the readout read's key codes (from step 1), and at p, per
/// head, its query code, key code and kept set; then the turn memory's lookup; and at p last
/// the nearest. The first difference ends the pass, judged by claude-247's near-tie rules with
/// the families' G₁₀ from `deciles` (P's) and the pick's from `g10`, the cell's sealed number.
///
/// It refuses, as an instrument fault: traces that do not line up (their headers, windows,
/// tokens up to p, reads, readouts or memory lookups, I-Align); a decision of Φ not P's value,
/// not taken by Q, or not met before the pass's end (I-Force); a code on a float side that is
/// not its coordinates' signs, and an entry its score does not select, always on the reference
/// side, Φ's decisions included, and on the other side wherever Φ does not force it; and a
/// table that does not reproduce a side's recorded pick and runner-up scores. On side R it
/// refuses a pick that scores below the reference's, or ties it with the larger id (R takes the
/// smaller). A float side's pick may score below the other's in f64 (it picked on the GPU's f32
/// scores), by at most [`NearestTable::behind_bound`], and is refused beyond it (claude-256);
/// within it, its margin is negative, and the card's rule, both margins at most G₁₀, judges it.
pub fn grade_pass(
    reference: GradeSide<'_>,
    other: GradeSide<'_>,
    deciles: &DecisionDeciles,
    position: usize,
    step: usize,
    force: &DecisionForceWindow,
    g10: i64,
) -> Result<PassOutcome, String> {
    let (rt, ot) = (reference.trace, other.trace);
    check_decision_header(rt)?;
    check_decision_header(ot)?;
    if rt.side != "F" && rt.side != "T" {
        return Err(format!("the reference side is F or T, not {}", rt.side));
    }
    if deciles.schema != DECISION_DECILES_SCHEMA || deciles.side != rt.side {
        return Err(format!(
            "the deciles are side {}'s ({}), not the reference's {}",
            deciles.side, deciles.schema, rt.side
        ));
    }
    if (rt.layers, rt.lanes, rt.heads, rt.kept) != (ot.layers, ot.lanes, ot.heads, ot.kept) {
        return Err("the traces' layers, lanes, heads or kept counts differ (I-Align)".to_owned());
    }
    if force.position != position || g10 < 0 {
        return Err(format!(
            "Φ is position {}'s, not {position}, or G₁₀ {g10} is negative",
            force.position
        ));
    }
    let (wp, wq) = (window_at(rt, position)?, window_at(ot, position)?);
    let covers = |w: &DecisionWindow| step < w.steps.len() && step < w.ids.len();
    if wp.reply_start != wq.reply_start || !covers(wp) || !covers(wq) {
        return Err(format!(
            "position {position}: the windows start their replies at steps {} and {}, or do not reach step {step} (I-Align)",
            wp.reply_start, wq.reply_start
        ));
    }
    if wp.ids[..=step] != wq.ids[..=step] {
        return Err(format!(
            "position {position}: the sides read different tokens up to step {step} (I-Align)"
        ));
    }
    let mut walk = PrefixWalk::new(rt, ot, deciles, force)?;
    let roles = |t: usize| {
        [CodeRole::Query, CodeRole::Key]
            .into_iter()
            .filter(move |&r| r == CodeRole::Query || t >= 1)
    };
    for t in 0..=step {
        let (sp, sq) = (&wp.steps[t], &wq.steps[t]);
        let place = format!("position {position}, step {t}");
        if sp.layers.len() != rt.layers || sq.layers.len() != rt.layers {
            return Err(format!("{place}: not {} layers (I-Align)", rt.layers));
        }
        for (l, (lp, lq)) in sp.layers.iter().zip(&sq.layers).enumerate() {
            if lp.entry.len() != rt.lanes || lq.entry.len() != rt.lanes {
                return Err(format!(
                    "{place}, layer {l}: not {} entries (I-Align)",
                    rt.lanes
                ));
            }
            for lane in 0..rt.lanes {
                if let Some(o) = walk.bucket(t, l, lane, lp, lq)? {
                    return walk.end(o);
                }
            }
            match (&lp.read, &lq.read) {
                (None, None) => {}
                (Some(rp), Some(rq)) => {
                    let depth = aligned_read(rp, rq, l + 1, rt.heads, &place)?;
                    for (h, (hp, hq)) in rp.heads.iter().zip(&rq.heads).enumerate() {
                        for role in roles(t) {
                            if let Some(o) = walk.code(t, depth, h, role, hp, hq)? {
                                return walk.end(o);
                            }
                        }
                        if hp.kept != hq.kept {
                            return walk.end(PassOutcome::Kept {
                                step: t,
                                depth,
                                head: h,
                                reference: hp.kept.clone(),
                                other: hq.kept.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(format!(
                        "{place}: one side reads after layer {l} and the other does not (I-Align)"
                    ))
                }
            }
        }
        match (&sp.readout, &sq.readout) {
            (None, None) => {}
            (Some(rp), Some(rq)) => {
                let depth = aligned_read(rp, rq, rt.layers, rt.heads, &place)?;
                for (h, (hp, hq)) in rp.heads.iter().zip(&rq.heads).enumerate() {
                    // Before p, the readout's keys alone: p's kept set reads them (claude-254).
                    for role in roles(t).filter(|&r| t == step || r == CodeRole::Key) {
                        if let Some(o) = walk.code(t, depth, h, role, hp, hq)? {
                            return walk.end(o);
                        }
                    }
                    if t == step && hp.kept != hq.kept {
                        return walk.end(PassOutcome::Kept {
                            step: t,
                            depth,
                            head: h,
                            reference: hp.kept.clone(),
                            other: hq.kept.clone(),
                        });
                    }
                }
            }
            _ => {
                return Err(format!(
                    "{place}: one side has a readout read and the other not (I-Align)"
                ))
            }
        }
        match (&sp.memory, &sq.memory) {
            (None, None) => {}
            (Some(mp), Some(mq)) if mp != mq => {
                return walk.end(PassOutcome::Memory {
                    step: t,
                    reference: mp.clone(),
                    other: mq.clone(),
                });
            }
            (Some(_), Some(_)) => {}
            _ => {
                return Err(format!(
                    "{place}: one side looks the turn memory up and the other not (I-Align)"
                ))
            }
        }
    }
    let (Some(np), Some(nq)) = (&wp.steps[step].nearest, &wq.steps[step].nearest) else {
        return Err(format!(
            "position {position}, step {step}: a side has no nearest there (I-Align)"
        ));
    };
    reference.table.check_nearest(&rt.side, np)?;
    other.table.check_nearest(&ot.side, nq)?;
    let (u, w) = (np.pick, nq.pick);
    let (reference_margin, other_margin) = if u == w {
        (0.0, 0.0)
    } else {
        (
            reference.table.score(np, u)? - reference.table.score(np, w)?,
            other.table.score(nq, w)? - other.table.score(nq, u)?,
        )
    };
    // R picks the best of its integer scores, a tie to the smaller id; a float side picked on
    // the GPU's f32 scores, so its pick may trail by f32's rounding and no more (claude-256).
    if ot.side == "R" && other_margin < 0.0 {
        return Err(format!(
            "position {position}, step {step}: side R's pick {w} scores {} below the reference's {u}: it is not its best",
            -other_margin
        ));
    }
    if ot.side == "R" && u != w && other_margin == 0.0 && w > u {
        return Err(format!(
            "position {position}, step {step}: side R's pick {w} ties the reference's {u}, and R takes the smaller id"
        ));
    }
    let behind = [
        (&rt.side, reference.table, np, u, w, reference_margin),
        (&ot.side, other.table, nq, w, u, other_margin),
    ];
    for (side, table, nearest, pick, rival, margin) in behind {
        let bound = table.behind_bound(nearest, pick, rival)?;
        if margin < 0.0 && -margin > bound {
            return Err(format!(
                "position {position}, step {step}: side {side}'s pick {pick} scores {} below {rival}, past f32's rounding bound {bound}: it is not its query's nearest",
                -margin
            ));
        }
    }
    let near_tie = u == w || (reference_margin <= g10 as f64 && other_margin <= g10 as f64);
    walk.end(PassOutcome::Nearest {
        reference_pick: u,
        other_pick: w,
        reference_margin,
        other_margin,
        g10,
        near_tie,
    })
}

/// Whether a finished forced replay accounts for its step (claude-247's step 3), from its
/// passes' outcomes in order. Every pass but the last ends at a code or a rate bucket, each a
/// different one, which the next pass forces; the last ends at a kept set or a memory lookup
/// (not accounted for) or at p's nearest, which accounts for the step when every forced
/// decision was a near tie and so is the pick. A replay that is not finished, or not one, is
/// refused.
pub fn replay_accounts(passes: &[PassOutcome]) -> Result<bool, String> {
    let (last, forced) = passes
        .split_last()
        .ok_or("a replay has at least one pass")?;
    let mut seen: Vec<&PassOutcome> = Vec::new();
    let mut all_near = true;
    for (i, pass) in forced.iter().enumerate() {
        let near_tie = match pass {
            PassOutcome::Bucket { near_tie, .. } | PassOutcome::Code { near_tie, .. } => *near_tie,
            _ => {
                return Err(format!(
                    "pass {i} ends at a kept set, a memory lookup or the nearest, where the replay stops, yet another pass follows"
                ))
            }
        };
        // The same decision is the same place, whatever value either pass forced there.
        let same = |a: &PassOutcome, b: &PassOutcome| match (a, b) {
            (PassOutcome::Bucket { force: x, .. }, PassOutcome::Bucket { force: y, .. }) => {
                (x.step, x.layer, x.lane) == (y.step, y.layer, y.lane)
            }
            (PassOutcome::Code { force: x, .. }, PassOutcome::Code { force: y, .. }) => {
                (x.step, x.depth, x.head, x.role) == (y.step, y.depth, y.head, y.role)
            }
            _ => false,
        };
        if seen.iter().any(|s| same(s, pass)) {
            return Err(format!("pass {i} forces a decision an earlier pass forced"));
        }
        seen.push(pass);
        all_near &= near_tie;
    }
    match last {
        PassOutcome::Bucket { .. } | PassOutcome::Code { .. } => Err(
            "the replay is not finished: its last pass ends at a decision the next pass forces"
                .to_owned(),
        ),
        PassOutcome::Kept { .. } | PassOutcome::Memory { .. } => Ok(false),
        PassOutcome::Nearest { near_tie, .. } => Ok(all_near && *near_tie),
    }
}

/// The schema `fbs-toks --decision-replay` writes (claude-256).
pub const DECISION_REPLAY_SCHEMA: &str = "uor-r4.decision-replay/1";

/// One step's finished forced replay ([`replay_step`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepReplay {
    /// Every pass's outcome, in order.
    pub passes: Vec<PassOutcome>,
    /// Φ as the last pass ran under it: each earlier pass's decision, at P's value.
    pub forced: DecisionForceWindow,
    /// [`replay_accounts`] on the passes.
    pub accounted: bool,
}

/// claude-247's forced replay of one step (P, Q) at the nearest of step `step` of the window at
/// `position` (claude-256). `pass` runs Q on that window under the forced set Φ, from empty, and
/// returns Q's trace; [`grade_pass`] grades each pass against `reference` (P, unforced). A pass
/// that ends at a code or a rate bucket adds P's value there to Φ, and Q runs again; one that
/// ends at a kept set, a memory lookup or p's nearest ends the replay, which [`replay_accounts`]
/// then reads.
///
/// It ends. `grade_pass` refuses a pass in which Q does not take every value of Φ, or in which a
/// decision of Φ lies outside p's prefix. So each pass's first difference is a decision of p's
/// prefix that Φ does not hold yet, and Φ grows by one of the prefix's finitely many decisions
/// at every pass but the last.
pub fn replay_step(
    reference: GradeSide<'_>,
    other_table: NearestTable<'_>,
    deciles: &DecisionDeciles,
    position: usize,
    step: usize,
    g10: i64,
    mut pass: impl FnMut(&DecisionForceWindow) -> Result<DecisionTrace, String>,
) -> Result<StepReplay, String> {
    let mut forced = DecisionForceWindow {
        position,
        ..Default::default()
    };
    let mut passes = Vec::new();
    loop {
        let i = passes.len();
        let trace = pass(&forced).map_err(|e| format!("pass {i}: {e}"))?;
        let other = GradeSide {
            trace: &trace,
            table: other_table,
        };
        let outcome = grade_pass(reference, other, deciles, position, step, &forced, g10)
            .map_err(|e| format!("pass {i}: {e}"))?;
        let again = match &outcome {
            PassOutcome::Bucket { force, .. } => {
                forced.buckets.push(force.clone());
                true
            }
            PassOutcome::Code { force, .. } => {
                forced.codes.push(force.clone());
                true
            }
            _ => false,
        };
        passes.push(outcome);
        if !again {
            break;
        }
    }
    let accounted = replay_accounts(&passes)?;
    Ok(StepReplay {
        passes,
        forced,
        accounted,
    })
}

/// What [`check_decision_trace`] checked on one unforced trace.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionTraceCheck {
    pub windows: usize,
    pub steps: usize,
    /// Rate buckets whose entry is the one their score selects.
    pub buckets: usize,
    /// Codes that are their coordinates' signs; none on R, which records no coordinates.
    pub codes: usize,
    /// Nearests whose pick and runner-up scores the table reproduces bit for bit.
    pub nearests: usize,
}

/// [`grade_pass`]'s checks of one side, over a whole unforced trace and its own table: each
/// step has the trace's layers and each layer its lanes' entries and scores, each entry is the
/// one its score selects, each float code is its coordinates' signs, and each nearest's pick
/// and runner-up scores are the table's (on R, the pick ahead).
pub fn check_decision_trace(
    trace: &DecisionTrace,
    table: NearestTable<'_>,
) -> Result<DecisionTraceCheck, String> {
    check_decision_header(trace)?;
    let mut check = DecisionTraceCheck {
        windows: trace.windows.len(),
        ..Default::default()
    };
    for window in &trace.windows {
        for (t, step) in window.steps.iter().enumerate() {
            let place = || format!("position {}, step {t}", window.position);
            check.steps += 1;
            if step.layers.len() != trace.layers {
                return Err(format!(
                    "{}: {} layers, not {}",
                    place(),
                    step.layers.len(),
                    trace.layers
                ));
            }
            for (l, layer) in step.layers.iter().enumerate() {
                let scores = match trace.side.as_str() {
                    "R" => layer.score_q16.len(),
                    _ => layer.score_f32.len(),
                };
                if (layer.entry.len(), scores) != (trace.lanes, trace.lanes) {
                    return Err(format!(
                        "{}, layer {l}: {} entries and {scores} rate scores, not {} of each",
                        place(),
                        layer.entry.len(),
                        trace.lanes
                    ));
                }
                for (lane, &entry) in layer.entry.iter().enumerate() {
                    let selected = selected_entry(&trace.side, layer, lane)
                        .map_err(|e| format!("{}, layer {l}, lane {lane}: {e}", place()))?;
                    if selected != entry {
                        return Err(format!(
                            "{}, layer {l}, lane {lane}: entry {entry} is not the {selected} its score selects",
                            place()
                        ));
                    }
                    check.buckets += 1;
                }
            }
            if trace.side != "R" {
                let reads = step.layers.iter().filter_map(|l| l.read.as_ref());
                for read in reads.chain(&step.readout) {
                    for (h, head) in read.heads.iter().enumerate() {
                        for (code, bits) in
                            [(head.query, &head.query_f32), (head.key, &head.key_f32)]
                        {
                            signs_to(code, bits).map_err(|e| {
                                format!("{}, depth {}, head {h}: {e}", place(), read.depth)
                            })?;
                            check.codes += 1;
                        }
                    }
                }
            }
            if let Some(nearest) = &step.nearest {
                table
                    .check_nearest(&trace.side, nearest)
                    .map_err(|e| format!("{}: {e}", place()))?;
                check.nearests += 1;
            }
        }
    }
    Ok(check)
}

/// The CPU float control's lane product (claude-237): the float trainer's
/// `kernel_controls::hamilton_gate_fwd_cpu`, the CPU twin of the GPU forward's `hamilton_gate`,
/// term for term, so that it gives the same bits (a float-train test holds the two equal). Each
/// lane of `up` is turned on the left by the unit `g / |g|` of its lane of `gate`.
pub fn hamilton_gate_f32(gate: &[f32], up: &[f32]) -> Vec<f32> {
    assert_eq!(gate.len(), up.len());
    assert_eq!(gate.len() % 4, 0);
    let mut out = vec![0.0f32; gate.len()];
    for i in (0..gate.len()).step_by(4) {
        let (g0, g1, g2, g3) = (gate[i], gate[i + 1], gate[i + 2], gate[i + 3]);
        let (u0, u1, u2, u3) = (up[i], up[i + 1], up[i + 2], up[i + 3]);
        let r = (g0 * g0 + g1 * g1 + g2 * g2 + g3 * g3 + 1e-12f32).sqrt();
        let (q0, q1, q2, q3) = (g0 / r, g1 / r, g2 / r, g3 / r);
        out[i] = q0 * u0 - q1 * u1 - q2 * u2 - q3 * u3;
        out[i + 1] = q0 * u1 + q1 * u0 + q2 * u3 - q3 * u2;
        out[i + 2] = q0 * u2 - q1 * u3 + q2 * u0 + q3 * u1;
        out[i + 3] = q0 * u3 + q1 * u2 - q2 * u1 + q3 * u0;
    }
    out
}

/// Each coordinate's sign bit, packed 64 to a word in coordinate order (claude-237): the float
/// trainer's `flock_read_hamming::pack_signs`, held equal by the same test. A coordinate at or
/// above 0 is a 1.
pub fn pack_signs_f32(coordinates: &[f32]) -> Vec<u64> {
    coordinates
        .chunks(64)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .fold(0u64, |word, (i, &c)| word | (u64::from(c >= 0.0) << i))
        })
        .collect()
}

/// The copy heads' codes by the CPU float control (claude-237). Each of the `vocab` rows of
/// `table` (`width` floats each) is turned lane by lane by the `width` floats of `key_rotation`,
/// as the float forward turns a token's point for a copy head's query and key. Each copy head's
/// `width / heads` coordinates are then packed by [`pack_signs_f32`]; the first `heads / 2`
/// heads are the copy heads.
///
/// It returns the codes, entry `v`'s code for copy head `h` at `v * copy + h` (as `CopyHeads`
/// lays them), and the turned copy coordinates, entry `v`'s coordinate `c` at `v * copy_width +
/// c`, so that a differing bit's value can be read.
pub fn float_copy_codes(
    key_rotation: &[f32],
    table: &[f32],
    vocab: usize,
    width: usize,
    heads: usize,
) -> Result<(Vec<u64>, Vec<f32>), String> {
    if heads == 0
        || !heads.is_multiple_of(2)
        || !width.is_multiple_of(4)
        || !width.is_multiple_of(heads)
        || width / heads > 64
    {
        return Err(format!(
            "{heads} heads over width {width}: an even count of heads of at most 64 coordinates, over whole lanes"
        ));
    }
    if key_rotation.len() != width || table.len() != vocab * width {
        return Err(format!(
            "a rotation of {} and a table of {}, not {width} and {vocab} × {width}",
            key_rotation.len(),
            table.len()
        ));
    }
    let (copy, head_width) = (heads / 2, width / heads);
    let mut codes = Vec::with_capacity(vocab * copy);
    let mut values = Vec::with_capacity(vocab * copy * head_width);
    for row in table.chunks_exact(width) {
        let turned = hamilton_gate_f32(key_rotation, row);
        for h in 0..copy {
            codes.push(pack_signs_f32(&turned[h * head_width..][..head_width])[0]);
        }
        values.extend_from_slice(&turned[..copy * head_width]);
    }
    Ok((codes, values))
}

/// The 120 unit icosians of 2I in the float trainer's order (claude-237): its
/// `hamilton_sheet::H4_ROOT_COEFFICIENTS`, each coordinate `(a + b·φ) / 2` evaluated in f64 and
/// rounded to f32, as its `h4_unit_roots_f32` does (a float-train test holds the two equal). The
/// order is ±1 on each axis, then `(±½)⁴`, then `½(0, ±1, ±φ, ±1/φ)` in the 12 even orders.
pub fn icosian_roots_f32() -> [[f32; 4]; 120] {
    const EVEN_ORDERS: [[usize; 4]; 12] = [
        [0, 1, 2, 3],
        [0, 2, 3, 1],
        [0, 3, 1, 2],
        [1, 0, 3, 2],
        [1, 2, 0, 3],
        [1, 3, 2, 0],
        [2, 0, 1, 3],
        [2, 1, 3, 0],
        [2, 3, 0, 1],
        [3, 0, 2, 1],
        [3, 1, 0, 2],
        [3, 2, 1, 0],
    ];
    let mut coefficients = [[[0i8; 2]; 4]; 120];
    for (axis, pair) in coefficients[..8].chunks_exact_mut(2).enumerate() {
        pair[0][axis] = [-2, 0];
        pair[1][axis] = [2, 0];
    }
    let mut index = 8;
    for signs in 0..16 {
        for (axis, c) in coefficients[index].iter_mut().enumerate() {
            *c = [if signs & (1 << axis) == 0 { -1 } else { 1 }, 0];
        }
        index += 1;
    }
    for order in EVEN_ORDERS {
        for signs in 0..8 {
            let sign = |bit: i32| -> i8 {
                if signs & bit == 0 {
                    -1
                } else {
                    1
                }
            };
            let signed = [[0, 0], [sign(4), 0], [0, sign(2)], [-sign(1), sign(1)]];
            for (axis, c) in coefficients[index].iter_mut().enumerate() {
                *c = signed[order[axis]];
            }
            index += 1;
        }
    }
    let phi = 0.5 * (1.0 + 5.0_f64.sqrt());
    coefficients.map(|root| root.map(|[a, b]| (0.5 * (f64::from(a) + f64::from(b) * phi)) as f32))
}

/// Each lane of `rotation` replaced by the icosian of [`icosian_roots_f32`] with the largest
/// inner product, the first on a tie (claude-237). An all-zero lane takes root 1, the identity.
/// This is the snap the GPU forward's D11 mode applies to every Hamilton gate
/// (`hamilton_sheet::icosian_snap_cuda_tensor`), the Hamming read's rotations among them, up to
/// two things a lane near a tie could show: the GPU sums the four products in its own order,
/// and its arg-max does not promise the first of tied roots. The CUDA-only test in
/// `flock_read_hamming.rs` compares the two.
pub fn snap_icosian_f32(rotation: &[f32]) -> Vec<f32> {
    assert_eq!(rotation.len() % 4, 0);
    let roots = icosian_roots_f32();
    rotation
        .chunks_exact(4)
        .flat_map(|lane| {
            if lane.iter().all(|&x| x == 0.0) {
                return roots[1];
            }
            let score =
                |root: &[f32; 4]| -> f32 { root.iter().zip(lane).map(|(r, x)| r * x).sum() };
            let mut best = 0;
            for (i, root) in roots.iter().enumerate().skip(1) {
                if score(root) > score(&roots[best]) {
                    best = i;
                }
            }
            roots[best]
        })
        .collect()
}

/// One read's copy-head codes by the CPU float control and by the integer session, for
/// [`copy_code_check`] (claude-237). Each code list holds entry `v`'s code for copy head `h` at
/// `v * copy + h`; the control's turned copy coordinates hold entry `v`'s coordinate `c` at
/// `v * copy_width + c`.
#[derive(Clone, Debug, PartialEq)]
pub struct CopyCodeRead {
    pub depth: usize,
    pub control_codes: Vec<u64>,
    pub control_values: Vec<f32>,
    pub integer_codes: Vec<u64>,
    /// The control's codes with the key rotation first snapped by [`snap_icosian_f32`]: what the
    /// GPU's D11 forward computes, whose gate hook snaps the read's rotations too.
    pub snapped_codes: Vec<u64>,
}

/// Each read's [`CopyCodeRead`] (claude-237), from the artifact's own values: per read its depth,
/// its raw key rotation at `2^rotation_exp` and the session's codes, and the Q12 writer `table`.
/// The control reads the rotation as `raw · 2^rotation_exp` and the table as `q / 4096` (the
/// grid [`q12_value`] rounds to), each in f32, so it and the session share their inputs.
pub fn copy_code_reads(
    reads: &[(usize, &[i32], Vec<u64>)],
    rotation_exp: i32,
    table: &[i16],
    vocab: usize,
    width: usize,
    heads: usize,
) -> Result<Vec<CopyCodeRead>, String> {
    let points: Vec<f32> = table.iter().map(|&q| f32::from(q) / 4096.0).collect();
    let scale = 2f64.powi(rotation_exp);
    reads
        .iter()
        .map(|(depth, raw, integer_codes)| {
            let rotation: Vec<f32> = raw.iter().map(|&g| (f64::from(g) * scale) as f32).collect();
            let at = |e: String| format!("the read at depth {depth}: {e}");
            let (control_codes, control_values) =
                float_copy_codes(&rotation, &points, vocab, width, heads).map_err(at)?;
            let (snapped_codes, _) =
                float_copy_codes(&snap_icosian_f32(&rotation), &points, vocab, width, heads)
                    .map_err(at)?;
            Ok(CopyCodeRead {
                depth: *depth,
                control_codes,
                control_values,
                integer_codes: integer_codes.clone(),
                snapped_codes,
            })
        })
        .collect()
}

/// The schema `fbs-toks --copy-codes` writes (claude-237).
pub const COPY_CODES_SCHEMA: &str = "uor-r4.copy-codes/1";

/// The bins of a copy-code value histogram: the control's `|value|` below `1e-6`, `1e-5`,
/// `1e-4`, `1e-3`, `1e-2` and `1e-1`, then at or above `1e-1`.
pub const COPY_VALUE_BINS: usize = 7;

fn copy_value_bin(value: f32) -> usize {
    let size = f64::from(value).abs();
    [1e-6, 1e-5, 1e-4, 1e-3, 1e-2, 1e-1]
        .iter()
        .position(|&edge| size < edge)
        .unwrap_or(COPY_VALUE_BINS - 1)
}

/// The bits set in `word`, lowest first.
fn set_bits(word: u64) -> impl Iterator<Item = u32> {
    (0..64).filter(move |&b| (word >> b) & 1 == 1)
}

/// One bit where the control's code and the session's differ, with the control's turned
/// coordinate there.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCodeBit {
    pub token: u32,
    pub head: usize,
    pub bit: u32,
    pub value: f32,
}

/// The control's codes against the session's, over every vocabulary entry and copy head.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyControlAgainstInteger {
    /// The codes compared (`vocab * copy`), and of them those equal.
    pub codes: usize,
    pub equal: usize,
    /// The bits that differ, in all.
    pub differing_bits: usize,
    /// The control's `|value|` at each differing bit, by [`COPY_VALUE_BINS`] bin.
    pub value_bins: Vec<usize>,
    /// The largest `|value|` at a differing bit; 0 when none differs.
    pub max_value: f32,
    /// The first [`MAX_FAULTS_LISTED`] differing bits.
    pub listed: Vec<CopyCodeBit>,
}

/// One trace's copy-head codes of one kind, query or key, against both sides' codes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCodeSamples {
    /// The samples, and of them those equal to the control's code, to the session's, and to the
    /// control's with the rotation snapped.
    pub samples: usize,
    pub equal_control: usize,
    pub equal_integer: usize,
    pub equal_snapped: usize,
    /// How many samples sit `d` bits from the control's code, at index `d` (0 to 64), and from
    /// the session's.
    pub bits_from_control: Vec<usize>,
    pub bits_from_integer: Vec<usize>,
    /// The control's `|value|` at each bit where a sample differs from the control's code, by
    /// [`COPY_VALUE_BINS`] bin.
    pub value_bins: Vec<usize>,
}

/// One trace's copy heads at one read.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCodeTrace {
    pub side: String,
    pub arm: String,
    /// The query at each position `t`, which is `ids[t]`'s code, and the key at each `s ≥ 1`,
    /// which is `ids[s − 1]`'s.
    pub query: CopyCodeSamples,
    pub key: CopyCodeSamples,
    /// The (token, copy head) pairs the trace holds, and of them those it gives more than one
    /// code across its queries and keys.
    pub pairs: usize,
    pub inconsistent: usize,
}

/// One read's codes, three ways.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCodeDepth {
    pub depth: usize,
    pub control_vs_integer: CopyControlAgainstInteger,
    pub float: CopyCodeTrace,
    pub integer: CopyCodeTrace,
}

/// The copy-code check's result (claude-237): a [`CopyCodeDepth`] per read, in depth order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCodeCheck {
    pub vocab: usize,
    pub heads: usize,
    pub reads: Vec<CopyCodeDepth>,
}

/// Every vocabulary entry's copy-head code three ways (claude-237, the item's step 5).
///
/// It compares the CPU float control's codes with the integer session's, on the same inputs
/// (`reads`, in depth order), and both with the codes the float and the integer
/// [`HammingTrace`]s hold at each position, matched by token; each trace also against the
/// control with the rotation snapped. A copy code depends only on its token, so a trace that
/// gives one (token, head) pair two codes counts it `inconsistent`.
///
/// Inputs that do not line up are refused before anything is compared:
/// - codes of the wrong length, or wider than a head;
/// - a trace of another schema or side, or with other heads;
/// - a window whose reads are not at `reads`' depths, whose read holds fewer than the copy heads,
///   whose codes are not one per position or are wider than a head, or whose ids are not
///   entries.
pub fn copy_code_check(
    vocab: usize,
    width: usize,
    heads: usize,
    reads: &[CopyCodeRead],
    float: &HammingTrace,
    integer: &HammingTrace,
) -> Result<CopyCodeCheck, String> {
    if heads == 0 || !heads.is_multiple_of(2) || !width.is_multiple_of(heads) || width / heads > 64
    {
        return Err(format!(
            "{heads} heads over width {width}: an even count of heads of at most 64 coordinates"
        ));
    }
    let (copy, head_width) = (heads / 2, width / heads);
    if reads.is_empty() {
        return Err("no read to check".to_owned());
    }
    for read in reads {
        let codes = vocab * copy;
        if read.control_codes.len() != codes
            || read.integer_codes.len() != codes
            || read.snapped_codes.len() != codes
            || read.control_values.len() != codes * head_width
        {
            return Err(format!(
                "the read at depth {}: {} control codes, {} control values, {} session codes and {} snapped codes, not {codes}, {}, {codes} and {codes}",
                read.depth,
                read.control_codes.len(),
                read.control_values.len(),
                read.integer_codes.len(),
                read.snapped_codes.len(),
                codes * head_width
            ));
        }
        if [
            &read.control_codes,
            &read.integer_codes,
            &read.snapped_codes,
        ]
        .iter()
        .any(|codes| codes.iter().any(|&c| wider_than(c, head_width)))
        {
            return Err(format!(
                "the read at depth {}: a code wider than the head's {head_width} bits",
                read.depth
            ));
        }
    }
    let depths: Vec<usize> = reads.iter().map(|r| r.depth).collect();
    check_copy_trace(float, "float-d11", vocab, heads, head_width, &depths)?;
    check_copy_trace(integer, "integer", vocab, heads, head_width, &depths)?;
    Ok(CopyCodeCheck {
        vocab,
        heads,
        reads: reads
            .iter()
            .enumerate()
            .map(|(r, read)| CopyCodeDepth {
                depth: read.depth,
                control_vs_integer: control_against_integer(read, copy, head_width),
                float: copy_trace(float, r, read, copy, head_width),
                integer: copy_trace(integer, r, read, copy, head_width),
            })
            .collect(),
    })
}

/// Whether `code` sets a bit at or past a head's `head_width` coordinates.
fn wider_than(code: u64, head_width: usize) -> bool {
    head_width < 64 && code >> head_width != 0
}

/// Why the `side` trace does not line up with the artifact, if it does not.
fn check_copy_trace(
    trace: &HammingTrace,
    side: &str,
    vocab: usize,
    heads: usize,
    head_width: usize,
    depths: &[usize],
) -> Result<(), String> {
    if trace.schema != HAMMING_TRACE_SCHEMA {
        return Err(format!(
            "the {side} trace's schema is {}, not {HAMMING_TRACE_SCHEMA}",
            trace.schema
        ));
    }
    if trace.side != side {
        return Err(format!(
            "the {side} trace was written by the {} side",
            trace.side
        ));
    }
    if trace.heads != heads {
        return Err(format!(
            "the {side} trace holds {} heads, the artifact {heads}",
            trace.heads
        ));
    }
    if trace.windows.is_empty() {
        return Err(format!("the {side} trace holds no window"));
    }
    for (w, window) in trace.windows.iter().enumerate() {
        let time = window.ids.len();
        if let Some(id) = window.ids.iter().find(|&&id| id as usize >= vocab) {
            return Err(format!(
                "the {side} trace's window {w}: id {id} is not one of {vocab} entries"
            ));
        }
        if window
            .reads
            .iter()
            .map(|r| r.depth)
            .ne(depths.iter().copied())
        {
            return Err(format!(
                "the {side} trace's window {w}: reads at depths {:?}, the artifact's at {depths:?}",
                window.reads.iter().map(|r| r.depth).collect::<Vec<_>>()
            ));
        }
        for read in &window.reads {
            let Some(copies) = read.heads.get(..heads / 2) else {
                return Err(format!(
                    "the {side} trace's window {w}, depth {}: {} heads, fewer than the {} copy heads",
                    read.depth,
                    read.heads.len(),
                    heads / 2
                ));
            };
            if copies
                .iter()
                .any(|h| h.query.len() != time || h.key.len() != time)
            {
                return Err(format!(
                    "the {side} trace's window {w}, depth {}: copy codes not one per position of {time}",
                    read.depth
                ));
            }
            if copies.iter().any(|h| {
                h.query
                    .iter()
                    .chain(&h.key)
                    .any(|&c| wider_than(c, head_width))
            }) {
                return Err(format!(
                    "the {side} trace's window {w}, depth {}: a copy code past the head's {head_width} bits",
                    read.depth
                ));
            }
        }
    }
    Ok(())
}

/// The control's codes against the session's at one read.
fn control_against_integer(
    read: &CopyCodeRead,
    copy: usize,
    head_width: usize,
) -> CopyControlAgainstInteger {
    let mut out = CopyControlAgainstInteger {
        codes: read.control_codes.len(),
        value_bins: vec![0; COPY_VALUE_BINS],
        ..Default::default()
    };
    for (i, (&control, &served)) in read
        .control_codes
        .iter()
        .zip(&read.integer_codes)
        .enumerate()
    {
        let diff = control ^ served;
        out.equal += usize::from(diff == 0);
        out.differing_bits += diff.count_ones() as usize;
        for bit in set_bits(diff) {
            let value = read.control_values[i * head_width + bit as usize];
            out.value_bins[copy_value_bin(value)] += 1;
            out.max_value = out.max_value.max(value.abs());
            if out.listed.len() < MAX_FAULTS_LISTED {
                out.listed.push(CopyCodeBit {
                    token: (i / copy) as u32,
                    head: i % copy,
                    bit,
                    value,
                });
            }
        }
    }
    out
}

/// The copy-head codes `trace` holds at its `r`-th read, against `read`'s two sides' codes.
fn copy_trace(
    trace: &HammingTrace,
    r: usize,
    read: &CopyCodeRead,
    copy: usize,
    head_width: usize,
) -> CopyCodeTrace {
    let samples = || CopyCodeSamples {
        bits_from_control: vec![0; 65],
        bits_from_integer: vec![0; 65],
        value_bins: vec![0; COPY_VALUE_BINS],
        ..Default::default()
    };
    let (mut query, mut key) = (samples(), samples());
    let mut codes: BTreeMap<(u32, usize), BTreeSet<u64>> = BTreeMap::new();
    let mut note = |into: &mut CopyCodeSamples, token: u32, head: usize, code: u64| {
        let i = token as usize * copy + head;
        let (control, served) = (read.control_codes[i], read.integer_codes[i]);
        into.samples += 1;
        into.equal_control += usize::from(code == control);
        into.equal_integer += usize::from(code == served);
        into.equal_snapped += usize::from(code == read.snapped_codes[i]);
        into.bits_from_control[(code ^ control).count_ones() as usize] += 1;
        into.bits_from_integer[(code ^ served).count_ones() as usize] += 1;
        for bit in set_bits(code ^ control) {
            into.value_bins[copy_value_bin(read.control_values[i * head_width + bit as usize])] +=
                1;
        }
        codes.entry((token, head)).or_default().insert(code);
    };
    for window in &trace.windows {
        for (head, h) in window.reads[r].heads[..copy].iter().enumerate() {
            for (t, &token) in window.ids.iter().enumerate() {
                note(&mut query, token, head, h.query[t]);
                if t + 1 < window.ids.len() {
                    note(&mut key, token, head, h.key[t + 1]);
                }
            }
        }
    }
    CopyCodeTrace {
        side: trace.side.clone(),
        arm: trace.arm.clone(),
        query,
        key,
        pairs: codes.len(),
        inconsistent: codes.values().filter(|c| c.len() > 1).count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// claude-251: a step in claude-249's R shape reads and writes byte for byte as before, and a
    /// float side's step keeps its f32 and f64 bits exactly and writes none of R's numbers.
    #[test]
    fn an_r_step_writes_as_before_and_a_float_step_keeps_its_bits() {
        let r = r#"{"layers":[{"score_q16":[65536,0],"entry":[15,0],"read":{"depth":2,"heads":[{"query":5,"key":0,"kept":[2,1]}]}},{"score_q16":[3,4],"entry":[0,0],"read":null}],"readout":null,"memory":{"key":[11,12],"stored":null,"opening":true},"nearest":{"pick":7,"score_q12":-120,"runner_up":3,"runner_score_q12":-131,"query_q12":[4096,-2048]}}"#;
        let step: DecisionStep = serde_json::from_str(r).expect("an R step parses");
        assert_eq!(serde_json::to_string(&step).expect("it writes"), r);
        let mut float = step.clone();
        let layer = &mut float.layers[0];
        layer.score_q16.clear();
        layer.score_f32 = vec![1.0f32.to_bits(), (-0.0f32).to_bits()];
        let head = &mut layer.read.as_mut().expect("a read").heads[0];
        head.query_f32 = vec![0.5f32.to_bits(), f32::MIN_POSITIVE.to_bits()];
        head.key_f32 = vec![(-3.25f32).to_bits(), 0.0f32.to_bits()];
        float.layers[1].score_q16.clear();
        float.layers[1].score_f32 = vec![0.1875f32.to_bits(), 0.25f32.to_bits()];
        let nearest = float.nearest.as_mut().expect("a nearest");
        (nearest.score_q12, nearest.runner_score_q12) = (None, None);
        nearest.score_f64 = Some((-119.875f64).to_bits());
        nearest.runner_score_f64 = Some((-131.0f64 / 3.0).to_bits());
        nearest.query_q12.clear();
        nearest.query_f32 = vec![1.0f32.to_bits(), (-0.5f32).to_bits()];
        let json = serde_json::to_string(&float).expect("it writes");
        assert!(!json.contains("_q12") && !json.contains("_q16"), "{json}");
        let back: DecisionStep = serde_json::from_str(&json).expect("it parses");
        assert_eq!(back, float);
        let nearest = back.nearest.expect("a nearest");
        assert_eq!(
            nearest.runner_score_f64.map(f64::from_bits),
            Some(-131.0f64 / 3.0)
        );
        assert_eq!(
            f32::from_bits(back.layers[0].score_f32[1]).to_bits(),
            (-0.0f32).to_bits()
        );
    }

    /// claude-237: the control turns each lane on the left by its gate's unit and packs each copy
    /// head's coordinates in coordinate order, laid out by entry and then copy head. Lane 0's
    /// gate `2i` turns `(a, b, c, d)` to `(−b, a, −d, c)`; lane 1 is past the copy heads.
    #[test]
    fn float_copy_codes_turn_each_lane_and_pack_each_copy_head() {
        let gate = [0.0, 2.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
        let table = [
            0.5, -0.25, 1.0, -2.0, 9.0, 9.0, 9.0, 9.0, //
            -1.0, 0.0, 3.0, -0.5, 9.0, 9.0, 9.0, 9.0,
        ];
        let (codes, values) = float_copy_codes(&gate, &table, 2, 8, 4).expect("the control");
        assert_eq!(values, [0.25, 0.5, 2.0, 1.0, 0.0, -1.0, 0.5, 3.0]);
        assert_eq!(codes, [0b11, 0b11, 0b01, 0b11], "entry, then copy head");
        assert!(
            float_copy_codes(&gate, &table, 2, 8, 3).is_err(),
            "odd heads"
        );
        assert!(
            float_copy_codes(&gate[..4], &table, 2, 8, 4).is_err(),
            "short rotation"
        );
        assert!(
            float_copy_codes(&gate, &table[..8], 2, 8, 4).is_err(),
            "short table"
        );
    }

    /// claude-237: the 120 roots are distinct unit quaternions; each, at any scale, snaps to
    /// itself; a lane near one snaps to it; and an all-zero lane takes the identity.
    #[test]
    fn the_snap_takes_the_nearest_of_the_120_icosians() {
        let roots = icosian_roots_f32();
        for (i, a) in roots.iter().enumerate() {
            let norm: f32 = a.iter().map(|x| x * x).sum();
            assert!((norm - 1.0).abs() < 1e-6, "root {i} has norm² {norm}");
            for b in &roots[i + 1..] {
                let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
                assert!(dot < 0.9, "root {i} repeats");
            }
            assert_eq!(snap_icosian_f32(&a.map(|x| 2.5 * x)), a);
        }
        // An identity-near lane, a lane near `(½)⁴`, then a zero lane.
        let lanes = [
            1.072_641_3,
            0.141_984_52,
            0.008_908_987,
            -0.047_924_876,
            0.6,
            0.4,
            0.5,
            0.55,
            0.0,
            0.0,
            0.0,
            0.0,
        ];
        let snapped = snap_icosian_f32(&lanes);
        assert_eq!(
            snapped,
            [1.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 0.0]
        );
    }

    /// The planted inputs of the copy-code tests: 5 entries, width 8, 2 heads (one copy head of
    /// 4 coordinates), one read at depth 2, and one window. The session's code of entry 3 has bit
    /// 2 flipped, where the control's value is `3e-5`; the float trace's query at position 3
    /// (entry 3) has bit 1 flipped, where the control's value is `0.2`; the snapped control's
    /// code of entry 1 has bit 0 set.
    fn planted() -> (Vec<CopyCodeRead>, HammingTrace, HammingTrace) {
        let control: Vec<u64> = vec![0b0001, 0b0010, 0b0100, 0b1000, 0b1111];
        let mut values = vec![0.5f32; 20];
        values[3 * 4 + 2] = 3e-5;
        values[3 * 4 + 1] = 0.2;
        let mut served = control.clone();
        served[3] ^= 0b100;
        let ids: Vec<u32> = vec![0, 1, 2, 3, 4, 3, 1];
        let trace = |side: &str, codes: &[u64]| {
            let key = (0..ids.len())
                .map(|s| codes[ids[s.saturating_sub(1)] as usize])
                .collect();
            HammingTrace {
                schema: HAMMING_TRACE_SCHEMA.to_owned(),
                side: side.to_owned(),
                arm: side.to_owned(),
                heads: 2,
                kept: 16,
                windows: vec![HammingTraceWindow {
                    ids: ids.clone(),
                    reads: vec![HammingTraceRead {
                        depth: 2,
                        heads: vec![HammingTraceHead {
                            query: ids.iter().map(|&v| codes[v as usize]).collect(),
                            key,
                            kept: vec![Vec::new(); ids.len()],
                            query_f32: Vec::new(),
                            key_f32: Vec::new(),
                        }],
                    }],
                }],
            }
        };
        let mut float = trace("float-d11", &control);
        float.windows[0].reads[0].heads[0].query[3] ^= 0b10;
        let integer = trace("integer", &served);
        let mut snapped = control.clone();
        snapped[1] |= 0b1;
        let read = CopyCodeRead {
            depth: 2,
            control_codes: control,
            control_values: values,
            integer_codes: served,
            snapped_codes: snapped,
        };
        (vec![read], float, integer)
    }

    /// claude-237: the check finds the planted differences and nothing else: the session's one
    /// bit, with the control's value there and its bin; the float trace's one flipped query, and
    /// the pair it makes inconsistent; and the integer trace's agreement with its own side.
    #[test]
    fn the_copy_code_check_finds_each_planted_difference() {
        let (reads, float, integer) = planted();
        let check = copy_code_check(5, 8, 2, &reads, &float, &integer).expect("the check");
        assert_eq!((check.vocab, check.heads, check.reads.len()), (5, 2, 1));
        let read = &check.reads[0];
        assert_eq!(read.depth, 2);
        let against = &read.control_vs_integer;
        assert_eq!(
            (against.codes, against.equal, against.differing_bits),
            (5, 4, 1)
        );
        assert_eq!(
            against.value_bins,
            [0, 0, 1, 0, 0, 0, 0],
            "3e-5 is below 1e-4"
        );
        assert_eq!(against.max_value, 3e-5);
        let bit = CopyCodeBit {
            token: 3,
            head: 0,
            bit: 2,
            value: 3e-5,
        };
        assert_eq!(against.listed, [bit]);

        let float = &read.float;
        assert_eq!((float.query.samples, float.query.equal_control), (7, 6));
        assert_eq!(float.query.equal_integer, 5, "entry 3 at positions 3 and 5");
        assert_eq!(
            float.query.equal_snapped, 4,
            "entry 1 at positions 1 and 6, and position 3"
        );
        assert_eq!(&float.query.bits_from_control[..3], [6, 1, 0]);
        assert_eq!(&float.query.bits_from_integer[..3], [5, 1, 1]);
        assert_eq!(
            float.query.value_bins,
            [0, 0, 0, 0, 0, 0, 1],
            "0.2 at bit 1"
        );
        assert_eq!((float.key.samples, float.key.equal_control), (6, 6));
        assert_eq!((float.pairs, float.inconsistent), (5, 1));

        let integer = &read.integer;
        assert_eq!((integer.query.samples, integer.query.equal_integer), (7, 7));
        assert_eq!(integer.query.equal_control, 5);
        assert_eq!(integer.query.equal_snapped, 3, "entries 0, 2 and 4");
        assert_eq!(integer.query.value_bins, [0, 0, 2, 0, 0, 0, 0]);
        assert_eq!((integer.key.samples, integer.key.equal_control), (6, 4));
        assert_eq!((integer.pairs, integer.inconsistent), (5, 0));
    }

    /// claude-237: inputs that do not line up are refused, each for its own reason.
    #[test]
    fn the_copy_code_check_refuses_inputs_that_do_not_line_up() {
        type Plant = fn(&mut Vec<CopyCodeRead>, &mut HammingTrace, &mut HammingTrace);
        let cases: [(&str, Plant); 12] = [
            ("schema", |_, f, _| f.schema = "other".to_owned()),
            ("written by", |_, f, _| f.side = "integer".to_owned()),
            ("heads, the artifact", |_, _, i| i.heads = 4),
            ("depths", |_, f, _| f.windows[0].reads[0].depth = 3),
            ("not one of 5", |_, f, _| f.windows[0].ids[2] = 5),
            ("fewer than", |_, f, _| f.windows[0].reads[0].heads.clear()),
            ("one per position", |_, _, i| {
                i.windows[0].reads[0].heads[0].key.pop();
            }),
            ("session codes", |r, _, _| {
                r[0].integer_codes.pop();
            }),
            ("snapped codes", |r, _, _| {
                r[0].snapped_codes.pop();
            }),
            ("wider than the head's 4 bits", |r, _, _| {
                r[0].integer_codes[4] |= 1 << 4;
            }),
            ("past the head's 4 bits", |_, f, _| {
                f.windows[0].reads[0].heads[0].key[6] |= 1 << 63;
            }),
            ("no window", |_, f, _| f.windows.clear()),
        ];
        for (why, plant) in cases {
            let (mut reads, mut float, mut integer) = planted();
            plant(&mut reads, &mut float, &mut integer);
            let error = copy_code_check(5, 8, 2, &reads, &float, &integer).expect_err(why);
            assert!(error.contains(why), "{why}: {error}");
        }
        let (reads, float, integer) = planted();
        assert!(copy_code_check(5, 8, 3, &reads, &float, &integer)
            .expect_err("odd heads")
            .contains("even count"));
        assert!(copy_code_check(5, 8, 2, &[], &float, &integer)
            .expect_err("no read")
            .contains("no read"));
    }

    #[test]
    fn write_new_file_never_replaces_and_never_leaves_a_partial_file() {
        let dir = std::env::temp_dir().join(format!("uor-fbs-write-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("out.json");
        let partial = dir.join("out.json.partial");
        let _ = std::fs::remove_file(&path);
        write_new_file(&path, b"first").expect("a new file is written");
        assert_eq!(std::fs::read(&path).expect("read"), b"first");
        assert!(!partial.exists());
        // An existing file is refused and kept as it was.
        assert!(write_new_file(&path, b"second").is_err());
        assert_eq!(std::fs::read(&path).expect("read"), b"first");
        assert!(!partial.exists());
        std::fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn float_readout_check_refuses_an_incomplete_or_misordered_readout() {
        let readout = |tokens: &[u32], mode: FbsReadoutMode| FbsFloatReadout {
            vocab: 3,
            width: 2,
            mode,
            positions: tokens
                .iter()
                .map(|&token| FbsFloatPosition {
                    token,
                    next: 0,
                    hidden: vec![0.0; 2],
                })
                .collect(),
            sequences: vec![FbsFloatSequence {
                tokens: vec![2, 0],
                next: vec![1, 1],
                hidden: vec![vec![0.0; 2]; 2],
            }],
        };
        assert!(readout(&[0, 1, 2], FbsReadoutMode::Plain)
            .check(3, 2, 4, FbsReadoutMode::Plain)
            .is_ok());
        assert!(readout(&[0, 1, 2], FbsReadoutMode::Export)
            .check(3, 2, 4, FbsReadoutMode::Export)
            .is_ok());
        for (what, r, reason) in [
            ("empty", readout(&[], FbsReadoutMode::Plain), "0 positions"),
            (
                "a token missing",
                readout(&[0, 1], FbsReadoutMode::Plain),
                "2 positions",
            ),
            (
                "a token repeated",
                readout(&[0, 1, 1], FbsReadoutMode::Plain),
                "position 2 is not token 2",
            ),
            (
                "out of order",
                readout(&[1, 0, 2], FbsReadoutMode::Plain),
                "position 0 is not token 0",
            ),
            (
                "the other mode",
                readout(&[0, 1, 2], FbsReadoutMode::Export),
                "mode Export",
            ),
        ] {
            match r.check(3, 2, 4, FbsReadoutMode::Plain) {
                Err(e) => assert!(e.contains(reason), "{what}: {e}"),
                Ok(()) => panic!("{what} must be refused"),
            }
        }
        let mut short = readout(&[0, 1, 2], FbsReadoutMode::Plain);
        short.positions[1].hidden.pop();
        assert!(short
            .check(3, 2, 4, FbsReadoutMode::Plain)
            .is_err_and(|e| e.contains("position 1")));
        assert!(readout(&[0, 1, 2], FbsReadoutMode::Plain)
            .check(4, 2, 4, FbsReadoutMode::Plain)
            .is_err());
        let mut none = readout(&[0, 1, 2], FbsReadoutMode::Plain);
        none.sequences.clear();
        assert!(none
            .check(3, 2, 4, FbsReadoutMode::Plain)
            .is_err_and(|e| e.contains("no sequences")));
        // A sequence must be 1 to `context` in-vocabulary tokens, each read.
        type Seq = fn(&mut FbsFloatSequence);
        let bad: [(&str, Seq); 5] = [
            ("empty", |q| {
                q.tokens.clear();
                q.next.clear();
                q.hidden.clear();
            }),
            ("past the context", |q| {
                q.tokens = vec![0; 5];
                q.next = vec![0; 5];
                q.hidden = vec![vec![0.0; 2]; 5];
            }),
            ("a token out of the vocabulary", |q| q.tokens[1] = 3),
            ("a position not read", |q| {
                q.next.pop();
            }),
            ("a short hidden vector", |q| {
                q.hidden[1].pop();
            }),
        ];
        for (what, mutate) in bad {
            let mut r = readout(&[0, 1, 2], FbsReadoutMode::Plain);
            mutate(&mut r.sequences[0]);
            match r.check(3, 2, 4, FbsReadoutMode::Plain) {
                Err(e) => assert!(e.contains("sequence 0"), "{what}: {e}"),
                Ok(()) => panic!("{what} must be refused"),
            }
        }
    }

    #[test]
    fn json_f32_round_trips_bit_exactly() {
        // The snapshot hands every f32 over as JSON; each must read back to
        // the same bits, signed zeros, subnormals and the extremes included.
        let mut values = vec![
            0.0f32,
            -0.0,
            f32::MIN_POSITIVE,
            -f32::MIN_POSITIVE,
            f32::from_bits(1),
            f32::from_bits(0x8000_0001),
            f32::from_bits(0x007f_ffff),
            f32::MAX,
            f32::MIN,
            f32::EPSILON,
            1.0 / 3.0,
            0.1,
        ];
        let mut state = 0x2026_1026_u64;
        while values.len() < 200_000 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let v = f32::from_bits((state >> 32) as u32);
            if v.is_finite() {
                values.push(v);
            }
        }
        let json = serde_json::to_vec(&values).expect("serialize");
        let back: Vec<f32> = serde_json::from_slice(&json).expect("deserialize");
        let changed = values
            .iter()
            .zip(&back)
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
        assert_eq!(
            changed,
            0,
            "{changed} of {} values changed bits",
            values.len()
        );
    }

    // claude-254, Q5's near-tie grader on synthetic traces: two layers of three lanes, a stream
    // read after layer 0 (depth 1) and the readout (depth 2), one copy and one type head, one
    // window of six steps whose reply starts at step 2, and a table of 5 tokens 4 wide.
    const G_LAYERS: usize = 2;
    const G_LANES: usize = 3;
    const G_WIDTH: usize = 4;
    const G_STEPS: usize = 6;
    const G_REPLY: usize = 2;
    const G_POSITION: usize = 7;

    /// The table, every value a multiple of 1/4096, so R's Q12 scores are the float sides'.
    fn g_table() -> Vec<f32> {
        (0..5)
            .flat_map(|u| {
                let u = u as f32;
                [0.25 * u, 0.5 - 0.125 * u, -0.0625 * u, 0.75]
            })
            .collect()
    }

    fn g_q12() -> Vec<i16> {
        g_table().iter().map(|&v| (v * 4096.0) as i16).collect()
    }

    /// Every token's float score for `query`, best first, ties to the smaller id.
    fn g_ranked(query: &[f32], table: &[f32]) -> Vec<(u32, f64)> {
        let mut ranked: Vec<(u32, f64)> = table
            .chunks(G_WIDTH)
            .enumerate()
            .map(|(u, row)| {
                let sum: f64 = query
                    .iter()
                    .zip(row)
                    .map(|(&q, &e)| (f64::from(q) - f64::from(e)).abs())
                    .sum();
                (u as u32, -4096.0 * sum)
            })
            .collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        ranked
    }

    /// A float side's nearest for `query`, with its ranked `pick` and `runner` places.
    fn g_nearest(query: &[f32], pick: usize, runner: usize) -> DecisionNearest {
        let ranked = g_ranked(query, &g_table());
        DecisionNearest {
            pick: ranked[pick].0,
            score_q12: None,
            score_f64: Some(ranked[pick].1.to_bits()),
            runner_up: ranked[runner].0,
            runner_score_q12: None,
            runner_score_f64: Some(ranked[runner].1.to_bits()),
            query_q12: Vec::new(),
            query_f32: query.iter().map(|q| q.to_bits()).collect(),
        }
    }

    /// The query at step `t`: tokens 2, 3, 4 and 0 win at steps 2 to 5.
    fn g_query(t: usize) -> Vec<f32> {
        vec![0.25 * (t % 5) as f32 + 0.0625, 0.25, 0.0, 0.75]
    }

    /// 64 coordinates, none zero: negative below index 32, magnitudes 1/64 to 63/64.
    fn g_coordinates(seed: usize) -> Vec<u32> {
        (0..64)
            .map(|i| ((i as f32 - 31.5) / 32.0 + (seed % 7) as f32 / 4096.0).to_bits())
            .collect()
    }

    fn g_signs(bits: &[u32]) -> u64 {
        let coordinates: Vec<f32> = bits.iter().map(|&b| f32::from_bits(b)).collect();
        pack_signs_f32(&coordinates)[0]
    }

    fn g_head(seed: usize, t: usize) -> DecisionHead {
        let (query_f32, key_f32) = (g_coordinates(seed), g_coordinates(seed + 3));
        DecisionHead {
            query: g_signs(&query_f32),
            key: g_signs(&key_f32),
            kept: (1..t as u32).rev().take(2).collect(),
            query_f32,
            key_f32,
        }
    }

    fn g_entry(s: f32) -> u8 {
        ((s * 16.0) as u8).min(15)
    }

    /// Side `side`'s float trace of the window, every decision the one its values make.
    fn g_float(side: &str) -> DecisionTrace {
        let steps = (0..G_STEPS)
            .map(|t| {
                let layers = (0..G_LAYERS)
                    .map(|l| {
                        let scores = (0..G_LANES)
                            .map(|lane| 0.05 + 0.3 * lane as f32 + 0.01 * (t + l) as f32);
                        let score_f32: Vec<u32> = scores.map(f32::to_bits).collect();
                        let entry = score_f32.iter().map(|&s| g_entry(f32::from_bits(s)));
                        DecisionLayer {
                            score_q16: Vec::new(),
                            entry: entry.collect(),
                            score_f32,
                            read: (l == 0).then(|| DecisionRead {
                                depth: 1,
                                heads: (0..2).map(|h| g_head(10 * t + h, t)).collect(),
                            }),
                        }
                    })
                    .collect();
                DecisionStep {
                    layers,
                    readout: Some(DecisionRead {
                        depth: G_LAYERS,
                        heads: (0..2).map(|h| g_head(10 * t + 5 + h, t)).collect(),
                    }),
                    memory: (t >= G_REPLY).then(|| DecisionMemory {
                        key: vec![t as u32],
                        stored: Some(9),
                        opening: t == G_REPLY,
                    }),
                    nearest: (t >= G_REPLY).then(|| g_nearest(&g_query(t), 0, 1)),
                }
            })
            .collect();
        DecisionTrace {
            schema: DECISION_TRACE_SCHEMA.to_owned(),
            side: side.to_owned(),
            layers: G_LAYERS,
            lanes: G_LANES,
            heads: 2,
            kept: 2,
            windows: vec![DecisionWindow {
                position: G_POSITION,
                ids: (0..G_STEPS as u32).map(|t| 100 + t).collect(),
                reply_start: G_REPLY,
                steps,
            }],
        }
    }

    /// R's trace of the window: the same decisions, its scores at Q16 and Q12, no coordinates,
    /// and its key at step 0 a 0.
    fn g_r() -> DecisionTrace {
        g_as_r(g_float("R"))
    }

    /// A float trace's decisions as R records them.
    fn g_as_r(mut r: DecisionTrace) -> DecisionTrace {
        r.side = "R".to_owned();
        for (t, step) in r.windows[0].steps.iter_mut().enumerate() {
            let reads = step.layers.iter_mut().filter_map(|l| l.read.as_mut());
            for read in reads.chain(step.readout.as_mut()) {
                for head in &mut read.heads {
                    (head.query_f32, head.key_f32) = (Vec::new(), Vec::new());
                    if t == 0 {
                        head.key = 0;
                    }
                }
            }
            for layer in &mut step.layers {
                let scores = layer.score_f32.iter().map(|&s| f32::from_bits(s));
                layer.score_q16 = scores.map(|s| (s * 65536.0).round() as i32).collect();
                layer.score_f32.clear();
            }
            if let Some(n) = step.nearest.as_mut() {
                let q: Vec<f32> = n.query_f32.iter().map(|&b| f32::from_bits(b)).collect();
                n.query_q12 = q.iter().map(|&v| (v * 4096.0) as i32).collect();
                n.score_q12 = n.score_f64.take().map(|s| f64::from_bits(s) as i64);
                n.runner_score_q12 = n.runner_score_f64.take().map(|s| f64::from_bits(s) as i64);
                n.query_f32.clear();
            }
        }
        r
    }

    fn g_side<'a>(trace: &'a DecisionTrace, table: &'a [f32], q12: &'a [i16]) -> GradeSide<'a> {
        let table = match trace.side.as_str() {
            "R" => NearestTable::Q12 {
                table: q12,
                width: G_WIDTH,
            },
            _ => NearestTable::F32 {
                table,
                width: G_WIDTH,
            },
        };
        GradeSide { trace, table }
    }

    fn g_phi(buckets: Vec<ForcedBucket>, codes: Vec<ForcedCode>) -> DecisionForceWindow {
        DecisionForceWindow {
            position: G_POSITION,
            buckets,
            codes,
        }
    }

    /// One pass at `step` of `q` against `p`, with `p`'s deciles and the shared tables.
    fn g_grade(
        p: &DecisionTrace,
        q: &DecisionTrace,
        step: usize,
        phi: &DecisionForceWindow,
        g10: i64,
    ) -> Result<PassOutcome, String> {
        g_grade_on(&g_table(), p, q, step, phi, g10)
    }

    /// [`g_grade`] on `table`, R's the same at Q12.
    fn g_grade_on(
        table: &[f32],
        p: &DecisionTrace,
        q: &DecisionTrace,
        step: usize,
        phi: &DecisionForceWindow,
        g10: i64,
    ) -> Result<PassOutcome, String> {
        let q12: Vec<i16> = table.iter().map(|&v| (v * 4096.0) as i16).collect();
        let deciles = decision_deciles(p)?;
        let (sp, sq) = (g_side(p, table, &q12), g_side(q, table, &q12));
        grade_pass(sp, sq, &deciles, G_POSITION, step, phi, g10)
    }

    /// The table with two copies of token 3's row: 5's last value one f32 step above 0.75,
    /// and 6's 2^−16 above. On step 3's query, 5 scores 2^−12 below 3, inside f32's rounding
    /// bound (about 7·10^−4 there), and 6 scores 1/16 below, past it. At Q12 both are 3's row.
    fn g_twins() -> Vec<f32> {
        let mut table = g_table();
        let three = table[3 * G_WIDTH..4 * G_WIDTH].to_vec();
        for last in [f32::from_bits(0.75f32.to_bits() + 1), 0.75 + 1.0 / 65536.0] {
            table.extend(&three[..G_WIDTH - 1]);
            table.push(last);
        }
        table
    }

    /// A float side's nearest on `table` for `query`, picking `pick` with runner-up `runner`.
    fn g_nearest_on(table: &[f32], query: &[f32], pick: u32, runner: u32) -> DecisionNearest {
        let mut nearest = g_nearest(query, 0, 1);
        (nearest.pick, nearest.runner_up) = (pick, runner);
        let on = NearestTable::F32 {
            table,
            width: G_WIDTH,
        };
        let score = |u| on.score(&nearest, u).expect("a row").to_bits();
        (nearest.score_f64, nearest.runner_score_f64) = (Some(score(pick)), Some(score(runner)));
        nearest
    }

    fn g_none() -> DecisionForceWindow {
        g_phi(Vec::new(), Vec::new())
    }

    /// Lane `lane` of layer `l` at step `t` set to score `s` and the entry `s` selects.
    fn g_set_score(trace: &mut DecisionTrace, t: usize, l: usize, lane: usize, s: f32) {
        let layer = &mut trace.windows[0].steps[t].layers[l];
        layer.score_f32[lane] = s.to_bits();
        layer.entry[lane] = g_entry(s);
    }

    fn g_head_mut(
        trace: &mut DecisionTrace,
        t: usize,
        depth: usize,
        h: usize,
    ) -> &mut DecisionHead {
        let step = &mut trace.windows[0].steps[t];
        let read = match depth {
            G_LAYERS => step.readout.as_mut(),
            _ => step.layers[depth - 1].read.as_mut(),
        };
        &mut read.expect("a read").heads[h]
    }

    /// Coordinate `bit` of the head's code in `role` negated, and the code re-signed.
    fn g_flip(head: &mut DecisionHead, role: CodeRole, bit: usize) {
        let (code, bits) = match role {
            CodeRole::Query => (&mut head.query, &mut head.query_f32),
            CodeRole::Key => (&mut head.key, &mut head.key_f32),
        };
        bits[bit] = (-f32::from_bits(bits[bit])).to_bits();
        *code ^= 1 << bit;
    }

    fn g_equal_pick(outcome: &PassOutcome) -> bool {
        matches!(outcome, PassOutcome::Nearest { reference_pick, other_pick, near_tie: true, .. } if reference_pick == other_pick)
    }

    /// claude-247's G₁₀ is the k-th smallest value, k = max(1, ⌊N/10⌋), zeros counted, and a
    /// key's coordinates at step 0 are no value of their family (claude-254).
    #[test]
    fn deciles_take_the_kth_smallest_value_with_zeros_and_without_step_0_keys() {
        let ramp: Vec<u32> = (0..64).map(|i| (i as f32 / 64.0).to_bits()).collect();
        let head = |key: Vec<u32>| DecisionHead {
            query: g_signs(&ramp),
            key: g_signs(&key),
            kept: Vec::new(),
            query_f32: ramp.clone(),
            key_f32: key,
        };
        let step = |offset: f32, key: Vec<u32>| DecisionStep {
            layers: vec![DecisionLayer {
                score_q16: Vec::new(),
                score_f32: (0..20)
                    .map(|i| (offset + 0.001 * i as f32).to_bits())
                    .collect(),
                entry: (0..20)
                    .map(|i| g_entry(offset + 0.001 * i as f32))
                    .collect(),
                read: None,
            }],
            readout: Some(DecisionRead {
                depth: 1,
                heads: vec![head(key.clone()), head(key)],
            }),
            memory: None,
            nearest: None,
        };
        let tiny: Vec<u32> = (0..64).map(|_| 1e-3f32.to_bits()).collect();
        let trace = DecisionTrace {
            schema: DECISION_TRACE_SCHEMA.to_owned(),
            side: "F".to_owned(),
            layers: 1,
            lanes: 20,
            heads: 2,
            kept: 2,
            windows: vec![DecisionWindow {
                position: 0,
                ids: vec![1, 2],
                reply_start: 1,
                steps: vec![step(0.5, tiny), step(0.2505, ramp.clone())],
            }],
        };
        let deciles = decision_deciles(&trace).expect("deciles");
        let code = |kind, role| DecisionFamily::Code {
            depth: 1,
            kind,
            role,
        };
        let get = |family| {
            deciles
                .families
                .iter()
                .find(|f| f.family == family)
                .cloned()
                .expect("the family")
        };
        // The queries: 0, 0, 1/64, 1/64, … over two steps; N = 128, k = 12, so 5/64.
        let query = get(code(HeadKind::Copy, CodeRole::Query));
        assert_eq!((query.values, query.k, query.g10), (128, 12, 5.0 / 64.0));
        // The keys: step 1's ramp alone; N = 64, k = 6, so 5/64, not step 0's 1e-3.
        let key = get(code(HeadKind::Type, CodeRole::Key));
        assert_eq!((key.values, key.k, key.g10), (64, 6, 5.0 / 64.0));
        // The buckets: N = 40, k = 4; the 4th smallest distance is step 1's lane 1.
        let bucket = get(DecisionFamily::Bucket { layer: 0 });
        let fourth = bucket_distance(0.2505 + 0.001);
        assert_eq!((bucket.values, bucket.k, bucket.g10), (40, 4, fourth));
        assert_eq!(deciles.families.len(), 5);
        assert_eq!(deciles.g10(DecisionFamily::Bucket { layer: 0 }), Ok(fourth));
        assert!(deciles.g10(DecisionFamily::Bucket { layer: 1 }).is_err());
        // The deciles round-trip, and every refusal holds.
        let json = serde_json::to_string(&deciles).expect("it writes");
        assert!(json.contains(r#""decision":"code""#), "{json}");
        let back: DecisionDeciles = serde_json::from_str(&json).expect("it parses");
        assert_eq!(back, deciles);
        let mut r = trace.clone();
        r.side = "R".to_owned();
        assert!(decision_deciles(&r).unwrap_err().contains("F or T"));
        let mut nan = trace.clone();
        nan.windows[0].steps[1]
            .readout
            .as_mut()
            .expect("a readout")
            .heads[0]
            .query_f32[3] = f32::NAN.to_bits();
        assert!(decision_deciles(&nan).unwrap_err().contains("not a number"));
        let mut short = trace.clone();
        short.windows[0].steps[0].layers[0].score_f32.pop();
        assert!(decision_deciles(&short)
            .unwrap_err()
            .contains("rate scores"));
        let mut nan_score = trace.clone();
        nan_score.windows[0].steps[1].layers[0].score_f32[4] = f32::NAN.to_bits();
        assert!(decision_deciles(&nan_score)
            .unwrap_err()
            .contains("lies outside [0, 1]"));
        let mut layers = trace.clone();
        layers.layers = 2;
        assert!(decision_deciles(&layers)
            .unwrap_err()
            .contains("1 layers, not 2"));
        let mut empty = trace;
        empty.lanes = 0;
        assert!(decision_deciles(&empty)
            .unwrap_err()
            .contains("neither is 0"));
    }

    /// A side graded against itself differs nowhere: every pass reaches the nearest with equal
    /// picks, and F, T and R, scored on their own tables, agree on the twin traces.
    #[test]
    fn a_side_graded_against_itself_or_its_twin_differs_nowhere() {
        let (f, t, r) = (g_float("F"), g_float("T"), g_r());
        for step in G_REPLY..G_STEPS {
            for (p, q) in [(&f, &f), (&t, &t), (&f, &r), (&t, &r), (&f, &t)] {
                let outcome = g_grade(p, q, step, &g_none(), 0).expect("a pass");
                assert!(
                    g_equal_pick(&outcome),
                    "{} {} {step}: {outcome:?}",
                    p.side,
                    q.side
                );
            }
        }
        // Every whole-trace check holds on each side's own table, T's being R's over 4096.
        let (table, q12) = (g_table(), g_q12());
        let t_table: Vec<f32> = q12.iter().map(|&v| f32::from(v) / 4096.0).collect();
        let f_check = check_decision_trace(&f, g_side(&f, &table, &q12).table).expect("F checks");
        let want = DecisionTraceCheck {
            windows: 1,
            steps: 6,
            buckets: 36,
            codes: 48,
            nearests: 4,
        };
        assert_eq!(f_check, want);
        let t_side = NearestTable::F32 {
            table: &t_table,
            width: G_WIDTH,
        };
        assert_eq!(check_decision_trace(&t, t_side), Ok(want.clone()));
        let r_check = check_decision_trace(&r, g_side(&r, &table, &q12).table).expect("R checks");
        assert_eq!(r_check, DecisionTraceCheck { codes: 0, ..want });
    }

    /// A rate bucket that differs is a near tie only when the entries are adjacent and P's
    /// score lies within G₁₀ of the boundary between them.
    #[test]
    fn a_bucket_is_a_near_tie_only_next_to_its_boundary() {
        let mut p = g_float("F");
        g_set_score(&mut p, 0, 1, 0, 0.062);
        // Layer 1's G₁₀ (N = 18, k = 1) is its smallest distance: step 0's 0.062, from 1/16.
        let g10 = decision_deciles(&p)
            .expect("deciles")
            .g10(DecisionFamily::Bucket { layer: 1 })
            .expect("layer 1");
        assert_eq!(g10, bucket_distance(0.062));
        let graded = |l: usize, lane: usize, t: usize, s: f32| {
            let mut q = p.clone();
            g_set_score(&mut q, t, l, lane, s);
            g_grade(&p, &q, 3, &g_none(), 0).expect("a pass")
        };
        // Step 0, lane 0: P's 0.062 (entry 0) and Q's 0.063 (entry 1), at the boundary's G₁₀.
        match graded(1, 0, 0, 0.063) {
            PassOutcome::Bucket {
                force,
                other_entry: 1,
                value: Some(v),
                near_tie: true,
                ..
            } => {
                let want = ForcedBucket {
                    step: 0,
                    layer: 1,
                    lane: 0,
                    entry: 0,
                };
                assert_eq!((force, v), (want, g10));
            }
            other => panic!("{other:?}"),
        }
        // Step 3, lane 1: P's 0.39 (entry 6) and Q's 0.44 (entry 7), 0.0475 from 7/16: clear.
        match graded(1, 1, 3, 0.44) {
            PassOutcome::Bucket {
                value: Some(v),
                near_tie: false,
                ..
            } => assert!((v - 0.0475).abs() < 1e-6, "{v}"),
            other => panic!("{other:?}"),
        }
        // Q's 0.37 (entry 5), below P's: the boundary between them is 6/16, 0.015 away.
        match graded(1, 1, 3, 0.37) {
            PassOutcome::Bucket {
                other_entry: 5,
                value: Some(v),
                near_tie: false,
                ..
            } => assert!((v - 0.015).abs() < 1e-6, "{v}"),
            other => panic!("{other:?}"),
        }
        // Entries two apart are never a near tie, however near the score.
        assert!(matches!(
            graded(1, 0, 0, 0.13),
            PassOutcome::Bucket {
                other_entry: 2,
                value: None,
                near_tie: false,
                ..
            }
        ));
    }

    /// A code that differs is a near tie when every bit that differs has |v_P| ≤ G₁₀: the value
    /// is the largest such coordinate, P's, whatever Q's is.
    #[test]
    fn a_code_is_a_near_tie_only_when_every_differing_bit_is_near_zero() {
        let p = g_float("F");
        let graded = |bits: &[usize]| {
            let mut q = p.clone();
            for &bit in bits {
                let head = g_head_mut(&mut q, 1, 1, 0);
                g_flip(head, CodeRole::Query, bit);
                // Q's coordinate fifty times P's in size, its sign kept.
                let v = f32::from_bits(head.query_f32[bit]);
                head.query_f32[bit] = (50.0 * v).to_bits();
            }
            g_grade(&p, &q, 3, &g_none(), 0).expect("a pass")
        };
        let family = DecisionFamily::Code {
            depth: 1,
            kind: HeadKind::Copy,
            role: CodeRole::Query,
        };
        let g10 = decision_deciles(&p)
            .expect("deciles")
            .g10(family)
            .expect("the family");
        let v = |bit: usize| f64::from(f32::from_bits(g_coordinates(10)[bit])).abs();
        let want = |code: u64| ForcedCode {
            step: 1,
            depth: 1,
            head: 0,
            role: CodeRole::Query,
            code,
        };
        let p_code = p.windows[0].steps[1].layers[0]
            .read
            .as_ref()
            .expect("a read")
            .heads[0]
            .query;
        // Bit 31 lies 1/64 from zero, under G₁₀.
        assert_eq!(
            graded(&[31]),
            PassOutcome::Code {
                force: want(p_code),
                other_code: p_code ^ 1 << 31,
                value: v(31),
                g10,
                near_tie: true
            }
        );
        // Bits 31 and 0 together: the value is bit 0's, 63/64 from zero, so clear.
        match graded(&[31, 0]) {
            PassOutcome::Code {
                value,
                near_tie: false,
                ..
            } => assert_eq!(value, v(0)),
            other => panic!("{other:?}"),
        }
    }

    /// The walk follows the forward's order: a step's layer-0 buckets, then the read after
    /// layer 0 (query, key, kept set per head), then layer 1's buckets; an earlier step first.
    #[test]
    fn the_walk_follows_the_forwards_order() {
        let p = g_float("F");
        let mut q = p.clone();
        // Two lanes of one layer differ: lane 0's comes first.
        g_set_score(&mut q, 3, 1, 2, 0.75);
        g_set_score(&mut q, 3, 1, 0, 0.13);
        let kind = |q: &DecisionTrace| match g_grade(&p, q, 4, &g_none(), 0).expect("a pass") {
            PassOutcome::Bucket { force, .. } => {
                format!("bucket {} {} {}", force.step, force.layer, force.lane)
            }
            PassOutcome::Code { force, .. } => format!(
                "code {} {} {} {:?}",
                force.step, force.depth, force.head, force.role
            ),
            PassOutcome::Kept {
                step, depth, head, ..
            } => format!("kept {step} {depth} {head}"),
            PassOutcome::Memory { step, .. } => format!("memory {step}"),
            PassOutcome::Nearest { .. } => "nearest".to_owned(),
        };
        assert_eq!(kind(&q), "bucket 3 1 0");
        g_head_mut(&mut q, 3, 1, 1).kept = vec![1];
        assert_eq!(kind(&q), "kept 3 1 1");
        g_flip(g_head_mut(&mut q, 3, 1, 1), CodeRole::Key, 40);
        assert_eq!(kind(&q), "code 3 1 1 Key");
        g_flip(g_head_mut(&mut q, 3, 1, 1), CodeRole::Query, 40);
        assert_eq!(kind(&q), "code 3 1 1 Query");
        g_flip(g_head_mut(&mut q, 3, 1, 0), CodeRole::Key, 40);
        assert_eq!(kind(&q), "code 3 1 0 Key");
        g_set_score(&mut q, 3, 0, 2, 0.5);
        assert_eq!(kind(&q), "bucket 3 0 2");
        q.windows[0].steps[2]
            .memory
            .as_mut()
            .expect("a lookup")
            .stored = None;
        assert_eq!(kind(&q), "memory 2");
        g_head_mut(&mut q, 2, 2, 0).kept = vec![9];
        g_flip(g_head_mut(&mut q, 2, 2, 1), CodeRole::Key, 40);
        assert_eq!(kind(&q), "code 2 2 1 Key");
    }

    /// p's prefix holds the readout read's keys at every step from 1 (claude-254), and of the
    /// readout's codes and kept sets at earlier steps nothing else; no key at step 0; and the
    /// keys at p itself.
    #[test]
    fn the_prefix_holds_the_readouts_earlier_keys_and_no_step_0_key() {
        let p = g_float("F");
        let at =
            |q: &DecisionTrace, step: usize| g_grade(&p, q, step, &g_none(), 0).expect("a pass");
        let code_at = |o: &PassOutcome| match o {
            PassOutcome::Code { force, .. } => Some((force.step, force.depth, force.role)),
            _ => None,
        };
        // The readout's key at step 1 is read by p = 3's kept set.
        let mut q = p.clone();
        g_flip(g_head_mut(&mut q, 1, 2, 0), CodeRole::Key, 40);
        assert_eq!(code_at(&at(&q, 3)), Some((1, 2, CodeRole::Key)));
        // Its query and kept set at step 2 reach only step 2's pick.
        let mut q = p.clone();
        g_flip(g_head_mut(&mut q, 2, 2, 0), CodeRole::Query, 40);
        assert!(g_equal_pick(&at(&q, 3)));
        assert_eq!(code_at(&at(&q, 2)), Some((2, 2, CodeRole::Query)));
        let mut q = p.clone();
        g_head_mut(&mut q, 2, 2, 1).kept = vec![9];
        assert!(g_equal_pick(&at(&q, 3)));
        assert!(matches!(
            at(&q, 2),
            PassOutcome::Kept {
                step: 2,
                depth: 2,
                head: 1,
                ..
            }
        ));
        // No key at step 0 is compared, the stream's or the readout's.
        let mut q = p.clone();
        g_flip(g_head_mut(&mut q, 0, 1, 0), CodeRole::Key, 40);
        g_flip(g_head_mut(&mut q, 0, 2, 1), CodeRole::Key, 40);
        assert!(g_equal_pick(&at(&q, 3)));
        // The keys at p stay in its prefix, the stream's and the readout's.
        let mut q = p.clone();
        g_flip(g_head_mut(&mut q, 3, 2, 1), CodeRole::Key, 40);
        assert_eq!(code_at(&at(&q, 3)), Some((3, 2, CodeRole::Key)));
        g_flip(g_head_mut(&mut q, 3, 1, 1), CodeRole::Key, 40);
        assert_eq!(code_at(&at(&q, 3)), Some((3, 1, CodeRole::Key)));
    }

    /// A kept set or a memory lookup that differs ends the pass, never a near tie.
    #[test]
    fn kept_sets_and_memory_lookups_end_the_pass() {
        let p = g_float("F");
        let mut q = p.clone();
        g_head_mut(&mut q, 4, 1, 1).kept = vec![2, 3];
        let kept = g_grade(&p, &q, 4, &g_none(), 0).expect("a pass");
        let want = PassOutcome::Kept {
            step: 4,
            depth: 1,
            head: 1,
            reference: vec![3, 2],
            other: vec![2, 3],
        };
        assert_eq!(kept, want);
        // A kept set with another member.
        let mut q = p.clone();
        g_head_mut(&mut q, 4, 1, 1).kept = vec![1, 3];
        assert!(matches!(
            g_grade(&p, &q, 4, &g_none(), 0).expect("a pass"),
            PassOutcome::Kept { reference, other, .. } if (reference.as_slice(), other.as_slice()) == (&[3, 2][..], &[1, 3][..])
        ));
        let mut q = p.clone();
        q.windows[0].steps[3]
            .memory
            .as_mut()
            .expect("a lookup")
            .opening = true;
        let memory = g_grade(&p, &q, 5, &g_none(), 0).expect("a pass");
        assert!(matches!(&memory, PassOutcome::Memory { step: 3, other, .. } if other.opening));
        assert_eq!(replay_accounts(&[kept]), Ok(false));
        assert_eq!(replay_accounts(&[memory]), Ok(false));
    }

    /// The pick is a near tie only when each side ranks the other's pick within G₁₀ of its own,
    /// in R's units.
    #[test]
    fn the_pick_is_a_near_tie_only_when_both_sides_rank_it_within_g10() {
        let p = g_float("F");
        let mut q = p.clone();
        // At step 3 P picks 3, 256 ahead of 2; Q reads step 2's query and picks 2, 1,280
        // ahead of 3.
        q.windows[0].steps[3].nearest = Some(g_nearest(&g_query(2), 0, 1));
        let graded = |g10| g_grade(&p, &q, 3, &g_none(), g10).expect("a pass");
        let want = |g10, near_tie| PassOutcome::Nearest {
            reference_pick: 3,
            other_pick: 2,
            reference_margin: 256.0,
            other_margin: 1280.0,
            g10,
            near_tie,
        };
        assert_eq!(graded(1280), want(1280, true));
        assert_eq!(graded(1279), want(1279, false));
        assert_eq!(graded(255), want(255, false));
        assert_eq!(replay_accounts(&[graded(1280)]), Ok(true));
        assert_eq!(replay_accounts(&[graded(1279)]), Ok(false));
        // With Q as the reference, its margin 1,280 is over a G₁₀ of 300 and P's 256 within.
        assert_eq!(
            g_grade(&q, &p, 3, &g_none(), 300).expect("a pass"),
            PassOutcome::Nearest {
                reference_pick: 2,
                other_pick: 3,
                reference_margin: 1280.0,
                other_margin: 256.0,
                g10: 300,
                near_tie: false,
            }
        );
        // A float pick 1,280 behind the other's in f64 is past f32's rounding: it is not its
        // query's nearest, and is refused however large G₁₀ is (claude-256).
        let mut behind = p.clone();
        behind.windows[0].steps[3].nearest = Some(g_nearest(&g_query(3), 2, 3));
        assert!(g_grade(&p, &behind, 3, &g_none(), 1_000_000)
            .unwrap_err()
            .contains("past f32's rounding bound"));
    }

    /// A float pick may trail the other side's pick, or its own runner-up, by f32's rounding
    /// bound and no more, on either side (claude-256); R ties to the smaller id.
    #[test]
    fn a_float_pick_may_trail_by_f32_rounding_and_no_more() {
        let twins = g_twins();
        let query = g_query(3);
        let at3 = |trace: &DecisionTrace, pick, runner| {
            let mut trace = trace.clone();
            trace.windows[0].steps[3].nearest = Some(g_nearest_on(&twins, &query, pick, runner));
            trace
        };
        let grade =
            |p: &DecisionTrace, q: &DecisionTrace| g_grade_on(&twins, p, q, 3, &g_none(), 1);
        let p = g_float("F");
        // The other side's pick 5, 2^−12 behind P's 3 and its runner-up 3: inside the bound.
        match grade(&p, &at3(&p, 5, 3)).expect("a pass") {
            PassOutcome::Nearest {
                reference_pick: 3,
                other_pick: 5,
                other_margin,
                near_tie: true,
                ..
            } => assert_eq!(other_margin, -1.0 / 4096.0),
            other => panic!("{other:?}"),
        }
        // Its pick 6, 1/16 behind P's 3: past it, refused at the grade.
        assert!(grade(&p, &at3(&p, 6, 2))
            .unwrap_err()
            .contains("side F's pick 6 scores 0.0625 below 3, past f32's rounding bound"));
        // The reference's pick may trail the other's by as much: P picks 5, Q (F, or R, whose
        // Q12 rows tie 3 with 5 and take the smaller id) picks 3.
        let p5 = at3(&p, 5, 3);
        for q in [at3(&p, 3, 2), g_as_r(at3(&p, 3, 2))] {
            match grade(&p5, &q).expect("a pass") {
                PassOutcome::Nearest {
                    reference_pick: 5,
                    other_pick: 3,
                    reference_margin,
                    near_tie: true,
                    ..
                } => assert_eq!(reference_margin, -1.0 / 4096.0),
                other => panic!("{other:?}"),
            }
        }
        assert!(grade(&at3(&p, 6, 2), &at3(&p, 3, 2))
            .unwrap_err()
            .contains("side F's pick 6 scores 0.0625 below 3, past"));
        // R's pick 5 ties P's 3 at Q12; R takes the smaller id, so 5 is not R's pick.
        assert!(grade(&p, &g_as_r(at3(&p, 5, 2)))
            .unwrap_err()
            .contains("side R's pick 5 ties the reference's 3, and R takes the smaller id"));
        // The trace check holds a pick to its runner-up the same way.
        let check = |t: &DecisionTrace| {
            let table = NearestTable::F32 {
                table: &twins,
                width: G_WIDTH,
            };
            check_decision_trace(t, table).map(|c| c.nearests)
        };
        assert_eq!(check(&at3(&p, 5, 3)), Ok(4));
        assert!(check(&at3(&p, 6, 3))
            .unwrap_err()
            .contains("pick 6 (-1536.0625) is not ahead of its runner-up 3 (-1536)"));
    }

    /// Φ's decisions are forced to P's value, taken by Q, and met in p's prefix before the
    /// pass's first difference; anything else is an instrument fault (I-Force).
    #[test]
    fn phi_holds_the_references_values_inside_the_prefix() {
        let p = g_float("F");
        let mut q = p.clone();
        // Q's score at step 3, layer 1, lane 1 selects 7; forced to P's 6, Q takes it.
        g_set_score(&mut q, 3, 1, 1, 0.44);
        q.windows[0].steps[3].layers[1].entry[1] = 6;
        let bucket = |entry| ForcedBucket {
            step: 3,
            layer: 1,
            lane: 1,
            entry,
        };
        let pass = |q: &DecisionTrace, phi| g_grade(&p, q, 4, &phi, 0);
        assert!(g_equal_pick(
            &pass(&q, g_phi(vec![bucket(6)], vec![])).expect("a pass")
        ));
        let err = |q: &DecisionTrace, phi| pass(q, phi).expect_err("refused");
        assert!(err(&q, g_phi(vec![], vec![])).contains("not the 7 its score selects"));
        assert!(err(&q, g_phi(vec![bucket(5)], vec![])).contains("not the reference's 6"));
        assert!(err(&q, g_phi(vec![bucket(6), bucket(6)], vec![])).contains("twice"));
        let mut untaken = q.clone();
        untaken.windows[0].steps[3].layers[1].entry[1] = 7;
        assert!(err(&untaken, g_phi(vec![bucket(6)], vec![])).contains("not the forced 6"));
        // A forced code Q takes, though its coordinates sign to another.
        let mut coded = p.clone();
        g_flip(g_head_mut(&mut coded, 1, 1, 0), CodeRole::Query, 0);
        let p_code = g_head_mut(&mut p.clone(), 1, 1, 0).query;
        g_head_mut(&mut coded, 1, 1, 0).query = p_code;
        let code = |step, depth, role, code| ForcedCode {
            step,
            depth,
            head: 0,
            role,
            code,
        };
        let forced = g_phi(vec![], vec![code(1, 1, CodeRole::Query, p_code)]);
        assert!(g_equal_pick(&pass(&coded, forced).expect("a pass")));
        assert!(err(&coded, g_none()).contains("the other side's code"));
        // Forcings outside p's prefix: a later step, a key at step 0, a readout query before p.
        let readout_query = g_head_mut(&mut p.clone(), 1, 2, 0).query;
        let step_0_key = g_head_mut(&mut p.clone(), 0, 1, 0).key;
        let outside = [
            g_phi(
                vec![ForcedBucket {
                    step: 5,
                    ..bucket(0)
                }],
                vec![],
            ),
            g_phi(vec![], vec![code(0, 1, CodeRole::Key, step_0_key)]),
            g_phi(vec![], vec![code(1, 2, CodeRole::Query, readout_query)]),
        ];
        for phi in outside {
            assert!(err(&p, phi).contains("outside p's prefix"));
        }
        // And one past the pass's first difference, a memory lookup at step 2.
        let mut early = q.clone();
        early.windows[0].steps[2]
            .memory
            .as_mut()
            .expect("a lookup")
            .stored = Some(8);
        assert!(err(&early, g_phi(vec![bucket(6)], vec![])).contains("past its first difference"));
    }

    /// Traces that do not line up, decisions their own values do not make, and tables that do
    /// not reproduce a side's scores are refused as instrument faults.
    #[test]
    fn grade_pass_refuses_what_does_not_line_up_or_check() {
        let p = g_float("F");
        let refused = |q: &DecisionTrace, want: &str| {
            let e = g_grade(&p, q, 3, &g_none(), 0).expect_err("refused");
            assert!(e.contains(want), "{want}: {e}");
        };
        let changed = |f: &dyn Fn(&mut DecisionTrace)| {
            let mut q = p.clone();
            f(&mut q);
            q
        };
        refused(
            &changed(&|q| q.lanes = 4),
            "layers, lanes, heads or kept counts differ (I-Align)",
        );
        refused(
            &changed(&|q| q.windows[0].position = 8),
            "no window at position 7",
        );
        refused(
            &changed(&|q| q.windows.push(q.windows[0].clone())),
            "two windows",
        );
        refused(
            &changed(&|q| q.windows[0].reply_start = 1),
            "start their replies at steps 2 and 1",
        );
        refused(
            &changed(&|q| q.windows[0].steps.truncate(3)),
            "or do not reach step 3",
        );
        refused(&changed(&|q| q.windows[0].ids[1] = 0), "different tokens");
        refused(
            &changed(&|q| q.windows[0].steps[1].layers[0].read = None),
            "reads after layer 0",
        );
        refused(
            &changed(&|q| q.windows[0].steps[1].readout = None),
            "readout read",
        );
        refused(
            &changed(&|q| q.windows[0].steps[2].memory = None),
            "turn memory",
        );
        refused(
            &changed(&|q| g_head_mut(q, 2, 1, 1).key ^= 1),
            "the other side's code",
        );
        refused(
            &changed(&|q| q.windows[0].steps[2].layers[0].entry[0] = 3),
            "the other side's entry",
        );
        refused(
            &changed(&|q| q.windows[0].steps[3].layers[0].score_f32[0] = 2f32.to_bits()),
            "outside [0, 1]",
        );
        // R's recorded pick is its third-best token, not its best.
        refused(
            &g_as_r(changed(&|q| {
                q.windows[0].steps[3].nearest = Some(g_nearest(&g_query(3), 2, 3))
            })),
            "side R's pick 4 scores 1280 below the reference's 3: it is not its best",
        );
        refused(
            &changed(&|q| {
                q.windows[0].steps[3]
                    .nearest
                    .as_mut()
                    .expect("a nearest")
                    .score_f64 = Some(0f64.to_bits())
            }),
            "not the recorded",
        );
        // The reference's own decisions are checked the same way.
        let mut bad = p.clone();
        g_head_mut(&mut bad, 1, 1, 0).query ^= 1 << 5;
        let e = g_grade(&bad, &p, 3, &g_none(), 0).expect_err("refused");
        assert!(e.contains("the reference's code"), "{e}");
        bad = p.clone();
        bad.windows[0].steps[1].layers[1].entry[2] = 0;
        let e = g_grade(&bad, &p, 3, &g_none(), 0).expect_err("refused");
        assert!(e.contains("the reference's entry"), "{e}");
        // A reference that is not F or T, deciles of another side, a step past the window or
        // without a nearest, a negative G₁₀, and a table not the side's own.
        let (table, q12) = (g_table(), g_q12());
        let deciles = decision_deciles(&p).expect("deciles");
        let r = g_r();
        let (sp, sr) = (g_side(&p, &table, &q12), g_side(&r, &table, &q12));
        let grade = |a, b, d: &DecisionDeciles, step, g10| {
            grade_pass(a, b, d, G_POSITION, step, &g_none(), g10).expect_err("refused")
        };
        assert!(grade(sr, sp, &deciles, 3, 0).contains("F or T"));
        let t = g_float("T");
        let st = g_side(&t, &table, &q12);
        assert!(grade(st, sp, &deciles, 3, 0).contains("deciles"));
        assert!(grade(sp, sr, &deciles, 6, 0).contains("do not reach step 6"));
        assert!(grade(sp, sr, &deciles, 1, 0).contains("no nearest"));
        assert!(grade(sp, sr, &deciles, 3, -1).contains("negative"));
        let wrong = GradeSide {
            trace: &r,
            table: NearestTable::F32 {
                table: &table,
                width: G_WIDTH,
            },
        };
        assert!(grade(sp, wrong, &deciles, 3, 0).contains("its own kind of table"));
        let mut skewed = q12.clone();
        skewed[4 * 3] += 1;
        let skewed = GradeSide {
            trace: &r,
            table: NearestTable::Q12 {
                table: &skewed,
                width: G_WIDTH,
            },
        };
        assert!(grade(sp, skewed, &deciles, 3, 0).contains("not the recorded"));
        // R's codes are never checked against coordinates, which it does not record: a clear
        // flip on R is graded by F's coordinate.
        let mut flipped = g_r();
        g_head_mut(&mut flipped, 2, 1, 0).query ^= 1;
        let sf = g_side(&flipped, &table, &q12);
        let outcome = grade_pass(sp, sf, &deciles, G_POSITION, 3, &g_none(), 0).expect("a pass");
        assert!(
            matches!(
                outcome,
                PassOutcome::Code {
                    near_tie: false,
                    ..
                }
            ),
            "{outcome:?}"
        );
    }

    /// The whole-trace check refuses an entry, a code or a table its side's values do not make.
    #[test]
    fn the_trace_check_refuses_what_its_values_do_not_make() {
        let (table, q12) = (g_table(), g_q12());
        let mut f = g_float("F");
        f.windows[0].steps[4].layers[1].entry[0] = 9;
        let side = |t: &DecisionTrace| check_decision_trace(t, g_side(t, &table, &q12).table);
        assert!(side(&f).unwrap_err().contains("entry 9"));
        let mut f = g_float("F");
        g_head_mut(&mut f, 0, 2, 1).key ^= 1 << 60;
        assert!(side(&f).unwrap_err().contains("not its coordinates' signs"));
        // A step without the trace's layers, or a layer without its lanes' entries and scores.
        let mut f = g_float("F");
        f.windows[0].steps[2].layers.pop();
        assert!(side(&f).unwrap_err().contains("step 2: 1 layers, not 2"));
        let mut f = g_float("F");
        f.windows[0].steps[1].layers[0]
            .score_f32
            .push(0.5f32.to_bits());
        assert!(side(&f)
            .unwrap_err()
            .contains("layer 0: 3 entries and 4 rate scores, not 3 of each"));
        // A float pick 1,024 behind its runner-up in f64 is past f32's rounding, refused as on
        // R (claude-256; `a_float_pick_may_trail_by_f32_rounding_and_no_more` has one inside).
        let mut f = g_float("F");
        f.windows[0].steps[3].nearest = Some(g_nearest(&g_query(3), 2, 1));
        assert!(side(&f)
            .unwrap_err()
            .contains("is not ahead of its runner-up"));
        assert!(side(&g_as_r(f))
            .unwrap_err()
            .contains("is not ahead of its runner-up"));
        let r = g_r();
        let mut skewed = q12.clone();
        skewed[0] -= 1;
        let check = check_decision_trace(
            &r,
            NearestTable::Q12 {
                table: &skewed,
                width: G_WIDTH,
            },
        );
        assert!(check.unwrap_err().contains("not the recorded"));
        // R's query on its pick's row scores 0, as R records it, not −0 (DEV s1 position 41).
        let mut exact = g_r();
        let row = |u: usize| &q12[u * G_WIDTH..(u + 1) * G_WIDTH];
        let n = exact.windows[0]
            .steps
            .iter_mut()
            .find_map(|s| s.nearest.as_mut())
            .unwrap();
        n.query_q12 = row(2).iter().map(|&v| i32::from(v)).collect();
        (n.pick, n.score_q12) = (2, Some(0));
        let far: i64 = row(2)
            .iter()
            .zip(row(0))
            .map(|(&q, &e)| (i64::from(q) - i64::from(e)).abs())
            .sum();
        (n.runner_up, n.runner_score_q12) = (0, Some(-far));
        let table = NearestTable::Q12 {
            table: &q12,
            width: G_WIDTH,
        };
        assert_eq!(
            check_decision_trace(&exact, table).map(|c| c.nearests > 0),
            Ok(true)
        );
    }

    /// A finished replay accounts for its step when every forced decision and the pick are near
    /// ties; one not finished, or not one, is refused.
    #[test]
    fn replay_accounts_reads_a_finished_replay() {
        let bucket = |near_tie, lane| PassOutcome::Bucket {
            force: ForcedBucket {
                step: 1,
                layer: 0,
                lane,
                entry: 3,
            },
            other_entry: 4,
            value: Some(0.01),
            g10: 0.02,
            near_tie,
        };
        let code = |near_tie| PassOutcome::Code {
            force: ForcedCode {
                step: 2,
                depth: 1,
                head: 0,
                role: CodeRole::Key,
                code: 5,
            },
            other_code: 7,
            value: 0.5,
            g10: 0.1,
            near_tie,
        };
        let nearest = |near_tie| PassOutcome::Nearest {
            reference_pick: 1,
            other_pick: 2,
            reference_margin: 3.0,
            other_margin: 4.0,
            g10: 5,
            near_tie,
        };
        assert_eq!(replay_accounts(&[nearest(true)]), Ok(true));
        assert_eq!(replay_accounts(&[nearest(false)]), Ok(false));
        assert_eq!(
            replay_accounts(&[bucket(true, 0), code(true), nearest(true)]),
            Ok(true)
        );
        assert_eq!(
            replay_accounts(&[bucket(true, 0), code(false), nearest(true)]),
            Ok(false)
        );
        assert_eq!(
            replay_accounts(&[bucket(false, 0), nearest(true)]),
            Ok(false)
        );
        assert!(replay_accounts(&[]).is_err());
        assert!(replay_accounts(&[bucket(true, 0)])
            .unwrap_err()
            .contains("not finished"));
        assert!(replay_accounts(&[nearest(true), nearest(true)])
            .unwrap_err()
            .contains("stops"));
        let kept = PassOutcome::Kept {
            step: 1,
            depth: 1,
            head: 0,
            reference: vec![0],
            other: vec![1],
        };
        assert!(replay_accounts(&[kept, nearest(true)])
            .unwrap_err()
            .contains("stops"));
        // The same place forced twice, whatever its value, is refused.
        let mut again = bucket(true, 0);
        if let PassOutcome::Bucket { force, .. } = &mut again {
            force.entry = 4;
        }
        assert!(replay_accounts(&[bucket(true, 0), again, nearest(true)])
            .unwrap_err()
            .contains("earlier pass"));
        let twice = [bucket(true, 0), bucket(true, 0), nearest(true)];
        assert!(replay_accounts(&twice)
            .unwrap_err()
            .contains("earlier pass"));
        assert_eq!(
            replay_accounts(&[bucket(true, 0), bucket(true, 1), nearest(true)]),
            Ok(true)
        );
        // An outcome writes with its kind, as `--decision-grade` writes it.
        let json = serde_json::to_string(&code(true)).expect("it writes");
        assert!(
            json.starts_with(r#"{"outcome":"code","force":{"step":2"#),
            "{json}"
        );
    }

    /// Q's passes under Φ for [`replay_step`]: `q` with every forced entry and code set to Φ's
    /// value, its scores and coordinates kept. A ninth pass is refused, so a loop that does not
    /// force the decision a pass ends at fails here instead of running on.
    fn g_passes(
        q: DecisionTrace,
    ) -> impl FnMut(&DecisionForceWindow) -> Result<DecisionTrace, String> {
        let mut runs = 0;
        move |phi: &DecisionForceWindow| {
            runs += 1;
            if runs > 8 {
                return Err("a ninth pass".to_owned());
            }
            let mut q = q.clone();
            for b in &phi.buckets {
                q.windows[0].steps[b.step].layers[b.layer].entry[b.lane] = b.entry;
            }
            for c in &phi.codes {
                let head = g_head_mut(&mut q, c.step, c.depth, c.head);
                match c.role {
                    CodeRole::Query => head.query = c.code,
                    CodeRole::Key => head.key = c.code,
                }
            }
            Ok(q)
        }
    }

    /// [`replay_step`] against `p` at step 3, Q a float side on the shared table.
    fn g_replay(
        p: &DecisionTrace,
        pass: impl FnMut(&DecisionForceWindow) -> Result<DecisionTrace, String>,
    ) -> Result<StepReplay, String> {
        let (table, q12) = (g_table(), g_q12());
        let deciles = decision_deciles(p)?;
        let other = NearestTable::F32 {
            table: &table,
            width: G_WIDTH,
        };
        replay_step(
            g_side(p, &table, &q12),
            other,
            &deciles,
            G_POSITION,
            3,
            0,
            pass,
        )
    }

    /// claude-247's forced replay (claude-256): each pass ends at Q's first difference from P
    /// in p's prefix; a code or a rate bucket joins Φ at P's value and Q runs again, so the
    /// replay takes one pass per differing decision and one more, Φ growing at every pass by a
    /// decision it did not hold.
    #[test]
    fn the_replay_forces_each_difference_it_meets_until_the_nearest() {
        let mut p = g_float("F");
        g_set_score(&mut p, 0, 1, 0, 0.062);
        // Q differs at two near ties: step 0's bucket (0.063 against 0.062, entries 1 and 0)
        // and step 1's query code (bit 31, 1/64 from zero); its pick is P's.
        let mut q = p.clone();
        g_set_score(&mut q, 0, 1, 0, 0.063);
        g_flip(g_head_mut(&mut q, 1, 1, 0), CodeRole::Query, 31);
        let mut ran = Vec::new();
        let mut passes = g_passes(q);
        let replay = g_replay(&p, |phi| {
            ran.push(phi.clone());
            passes(phi)
        })
        .expect("a replay");
        assert!(matches!(
            replay.passes[0],
            PassOutcome::Bucket { near_tie: true, .. }
        ));
        assert!(matches!(
            replay.passes[1],
            PassOutcome::Code { near_tie: true, .. }
        ));
        assert!(g_equal_pick(&replay.passes[2]));
        assert_eq!(replay.passes.len(), 3);
        let bucket = ForcedBucket {
            step: 0,
            layer: 1,
            lane: 0,
            entry: 0,
        };
        let code = ForcedCode {
            step: 1,
            depth: 1,
            head: 0,
            role: CodeRole::Query,
            code: g_head_mut(&mut p.clone(), 1, 1, 0).query,
        };
        let phis = [
            g_none(),
            g_phi(vec![bucket.clone()], vec![]),
            g_phi(vec![bucket], vec![code]),
        ];
        assert_eq!(ran, phis);
        assert_eq!(replay.forced, phis[2]);
        assert!(replay.accounted);
        // A step with no difference is one pass, the nearest's.
        let same = g_replay(&p, g_passes(p.clone())).expect("a replay");
        assert_eq!((same.passes.len(), same.accounted), (1, true));
    }

    /// A replay that forces a clear decision, or ends at a kept set, does not account for its
    /// step, though its picks end equal (claude-256).
    #[test]
    fn a_clear_forced_decision_or_a_kept_set_leaves_the_step_unaccounted() {
        let p = g_float("F");
        // Bits 31 and 0: bit 0 lies 63/64 from zero, so the code is clear.
        let mut q = p.clone();
        for bit in [31, 0] {
            g_flip(g_head_mut(&mut q, 1, 1, 0), CodeRole::Query, bit);
        }
        let clear = g_replay(&p, g_passes(q)).expect("a replay");
        assert!(matches!(
            clear.passes[0],
            PassOutcome::Code {
                near_tie: false,
                ..
            }
        ));
        assert!(g_equal_pick(&clear.passes[1]));
        assert_eq!((clear.passes.len(), clear.accounted), (2, false));
        // A near-tie bucket, forced, then a kept set at step 3: the replay stops there.
        let mut p = p;
        g_set_score(&mut p, 0, 1, 0, 0.062);
        let mut q = p.clone();
        g_set_score(&mut q, 0, 1, 0, 0.063);
        g_head_mut(&mut q, 3, 1, 1).kept = vec![1];
        let kept = g_replay(&p, g_passes(q)).expect("a replay");
        assert!(matches!(
            kept.passes[0],
            PassOutcome::Bucket { near_tie: true, .. }
        ));
        assert!(matches!(
            kept.passes[1],
            PassOutcome::Kept {
                step: 3,
                depth: 1,
                head: 1,
                ..
            }
        ));
        assert_eq!((kept.passes.len(), kept.accounted), (2, false));
    }

    /// A pass in which Q does not take Φ's value is an instrument fault, refused with its pass
    /// number (I-Force, claude-256).
    #[test]
    fn a_pass_that_does_not_take_a_forced_value_is_refused() {
        let mut p = g_float("F");
        g_set_score(&mut p, 0, 1, 0, 0.062);
        let mut q = p.clone();
        g_set_score(&mut q, 0, 1, 0, 0.063);
        let err = g_replay(&p, |_: &DecisionForceWindow| Ok(q.clone())).unwrap_err();
        assert!(
            err.starts_with("pass 1: ")
                && err.ends_with("took entry 1, not the forced 0 (I-Force)"),
            "{err}"
        );
        // And a pass that fails is the pass's error.
        let err = g_replay(&p, |_: &DecisionForceWindow| Err("no GPU".to_owned())).unwrap_err();
        assert_eq!(err, "pass 0: no GPU");
    }
}
