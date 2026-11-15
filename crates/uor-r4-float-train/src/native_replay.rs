//! The probe's native Q31 replay math (claude-389, ORDER item 3 and its sheet).
//!
//! Plain data in, plain data out: no tensor library, no GPU. A capture of the
//! float read's inputs (stream rows at the read, token ids, raw gates, weight,
//! table) is converted once to Q31 words and run through the merged
//! [`Q31FirstPickRead`]. The same captured inputs are also read by an f64
//! reference of the float read's definition: the gates normalized once in f64
//! (`kernel_controls.rs:93`, `+1e-12` inside the root), the left Hamilton
//! product per 4-lane block, the 16-group fold, `mean|q - k|` over earlier
//! rows (least wins, an exact tie goes to the latest row) and `x + w·e`.
//!
//! The comparison fields of a capture (the f32 first pick and out row) are read
//! only after the operator has returned. This is reference code under D85: a
//! diagnostic, not a model export, and nothing it makes is served. It reports
//! the booleans the sheet's reading map needs and does not name the letter.

use serde::Serialize;
use uor_r4_native_vector::{Q31FirstPickRead, Q31Quaternion, Q31ReadDump};

use crate::{FloatTrainError, Result};

/// The only width the sheet's bound `B` is derived for (16 fold groups).
pub const REPLAY_WIDTH: usize = 256;
/// Lanes in one folded address, the sheet's `G`.
const GROUPS: usize = 16;
/// The sheet's score bound `B = 8G` in stream LSB.
pub const SCORE_BOUND_LSB: f64 = 128.0;
/// The float gate's epsilon inside the square root.
const NORM_EPSILON: f64 = 1e-12;
/// The Q31 unit, `2^31`.
const Q31_UNIT: f64 = 2_147_483_648.0;
/// One bit of headroom: every bounded stream quantity sits at most at `2^30`.
const HEADROOM: f64 = 1_073_741_824.0;
const Q31_LIMIT: i128 = 1 << 31;
/// The largest gain shift the operator admits.
const MAX_GAIN_SHIFT: u32 = 31;
/// The control-1 key gate `[2^30; 4]`, the isoclinic 60° rotation.
const CONTROL_KEY_GATE: Q31Quaternion = [1 << 30; 4];

/// The captured state of one read: raw gates, weight and shared table.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayModel {
    pub width: usize,
    /// One raw (unnormalized) query gate per 4-lane block: `width / 4` gates.
    pub query_gates_raw: Vec<[f32; 4]>,
    /// One raw (unnormalized) key gate per 4-lane block: `width / 4` gates.
    pub key_gates_raw: Vec<[f32; 4]>,
    /// Per-lane weight `w`: `width` values.
    pub weights: Vec<f32>,
    /// The shared token table `e`: `vocab * width` values.
    pub table: Vec<f32>,
}

/// One captured window of the read.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayWindow {
    pub time: usize,
    /// The stream rows `x` at the read: `time * width` values.
    pub stream: Vec<f32>,
    /// The window's token ids: `time` ids.
    pub ids: Vec<u32>,
    /// Comparison only: the f32 first pick as `s + 1`, `0` for none (row 0).
    pub f32_first_pick: Vec<u32>,
    /// Comparison only: the f32 out row: `time * width` values.
    pub f32_out: Vec<f32>,
    /// The held-out query rows the bar and the controls are judged on.
    pub query: Vec<bool>,
}

/// The conversion rule, recorded in the JSON.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayScales {
    /// The stream scale exponent: words are `round(x · 2^e_x)`.
    pub e_x: i32,
    /// The table scale exponent.
    pub e_t: i32,
    /// The weight scale exponent, `31 + e_x - e_t - g`.
    pub e_w: i32,
    /// The gain shift `g`; the operator shifts the payload by `31 - g`.
    pub gain_shift: u32,
    /// The gate scale, fixed at `2^31`.
    pub gate_scale_log2: i32,
    pub max_abs_x: f64,
    /// Over every row and every earlier candidate row the operator could pick.
    pub max_abs_x_plus_we: f64,
    pub max_abs_x_plus_16we: f64,
    pub max_abs_table: f64,
    pub max_abs_weight: f64,
}

/// One row whose pick differs from the f64 reference pick.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PickGap {
    pub window: usize,
    pub row: usize,
    pub reference_pick: u32,
    pub other_pick: u32,
    /// `|ref score(ref pick) - ref score(other pick)|`; the unit is named by
    /// the field holding this row.
    pub gap: f64,
}

/// One out-word comparison over the rows whose picks agree.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutWordCheck {
    /// The gain shift the operator ran at.
    pub gain_shift: u32,
    /// `2^gain + 2` LSB.
    pub bound_lsb: f64,
    pub words_checked: u64,
    pub words_above_bound: u64,
    pub words_above_4x_bound: u64,
    pub worst_miss_lsb: f64,
    pub pass: bool,
}

/// Saturated words, each counted before the operator could clamp it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClipCounts {
    /// Stream, table, weight and gate words whose rounded value exceeds `2^31`.
    pub input_words: u64,
    /// Rotated lanes whose exact i128 rounded value has `|lane| > 2^31`.
    pub rotated_lanes: u64,
    /// Out words whose exact i128 payload or `x + payload` exceeds `2^31`.
    pub out_words: u64,
}

impl ClipCounts {
    pub fn total(&self) -> u64 {
        self.input_words + self.rotated_lanes + self.out_words
    }
}

/// Control 1: every key gate replaced by `[2^30; 4]` on both sides.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ControlOne {
    pub mismatches: u64,
    pub mismatches_above_2b: u64,
    pub near_ties: u64,
    pub agree_with_unsubstituted: u64,
    /// (a): no native-vs-reference mismatch above `2B`.
    pub a_pass: bool,
    /// (b): agreement with the unsubstituted native picks below 99%.
    pub b_pass: bool,
}

/// Control 4: the capture matches the f32 read that ran.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ControlFour {
    /// `B_f = 2^-12 · max|x|`, in float units.
    pub b_f: f64,
    /// Rows where the f32 and f64 picks disagree; `gap` in float units.
    pub disagreements: Vec<PickGap>,
    /// Disagreeing rows whose gap exceeds `2 · B_f`.
    pub disagreements_above_2bf: u64,
    /// f32 picks that name no earlier row (or name one at row 0).
    pub invalid_f32_picks: u64,
    pub out_words_checked: u64,
    /// f32 out words farther than `2^-22 · (|x| + |w·e|)` from the f64 word.
    pub out_words_above_bound: u64,
    pub a_pass: bool,
    pub b_pass: bool,
}

/// The structural check: every window's row 0 picks 0 and keeps its stream.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StructuralCheck {
    pub rows: u64,
    pub failures: u64,
    pub pass: bool,
}

/// What a replay that ran measured.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayRun {
    pub scales: ReplayScales,
    pub windows: u64,
    pub query_rows: u64,
    /// Bar 1: native picks equal to the f64 reference picks on query rows.
    pub picks_agree: u64,
    /// `ceil(0.99 · query_rows)`.
    pub picks_required: u64,
    pub picks_pass: bool,
    /// Mismatching query rows; `gap` in stream LSB (`· 2^e_x`).
    pub pick_mismatches: Vec<PickGap>,
    pub mismatches_above_2b: u64,
    /// Query rows whose f64 top-1 and top-2 scores are within `2B`.
    pub near_ties: u64,
    /// Bar 2, at the trained scale (×1, gain `g`).
    pub out_words: OutWordCheck,
    /// Bar 3, over every row of every window in the barred computation.
    pub clips: ClipCounts,
    pub structural: StructuralCheck,
    pub control_1: ControlOne,
    pub control_4: ControlFour,
    /// Report line: native picks equal to the f32 first picks on query rows.
    pub native_vs_f32_agree: u64,
    /// Report line: f32 picks differing from the f64 reference on query rows.
    pub f32_vs_f64_disagree: u64,
    /// Report line: ×16 (gain `g + 4`) against `x + 16·w·e`; `None` when
    /// `g + 4 > 31`.
    pub out_words_x16: Option<OutWordCheck>,
    /// Out-word clips of the ×16 run (not barred).
    pub out_word_clips_x16: Option<u64>,
    /// All three bar conditions.
    pub bar_pass: bool,
    /// A query row the reading map's N2 names: a pick mismatch above `2B` or
    /// an out-word miss above 4× its bound.
    pub definition_rows_present: bool,
}

/// The replay's JSON: a refusal, or what the run measured.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplaySummary {
    pub width: usize,
    /// `Some` when the state is refused by the sheet (width or gain); the
    /// sheet reads such a state as D.
    pub refusal: Option<String>,
    pub run: Option<ReplayRun>,
}

/// Run the native replay and the f64 reference over every window.
///
/// A malformed capture (inconsistent lengths, an id outside the table, a
/// non-finite value) is an `Err`. A well-formed state the sheet refuses (a
/// width other than 256, or no gain shift that fits) is an `Ok` summary with
/// `refusal` set and no run.
pub fn replay(model: &ReplayModel, windows: &[ReplayWindow]) -> Result<ReplaySummary> {
    replay_inner(model, windows, false, None)
}

/// [`replay`], and also the barred operator with every window's converted words as one batched
/// [`Q31ReadDump`] (`None` for a refused state), for the sheet's control 2: the CPU operator
/// against its CUDA twin on a trained state's real inputs. Every window must have one length.
pub fn replay_with_dump(
    model: &ReplayModel,
    windows: &[ReplayWindow],
) -> Result<(ReplaySummary, Option<Q31ReadDump>)> {
    let mut dump = None;
    let summary = replay_inner(model, windows, false, Some(&mut dump))?;
    Ok((summary, dump))
}

fn invalid(message: &str) -> FloatTrainError {
    FloatTrainError::Invalid(format!("native replay: {message}"))
}

fn pow2(exponent: i32) -> f64 {
    2f64.powi(exponent)
}

/// The largest `e` with `value · 2^e <= cap`, or `0` for a zero value.
fn largest_exponent(value: f64, cap: f64) -> i32 {
    if value == 0.0 {
        return 0;
    }
    let mut e = (cap / value).log2().floor() as i32;
    while value * pow2(e + 1) <= cap {
        e += 1;
    }
    while value * pow2(e) > cap {
        e -= 1;
    }
    e
}

fn normalize_gate(raw: &[f32; 4]) -> [f64; 4] {
    let g = raw.map(f64::from);
    let r = (g.iter().map(|v| v * v).sum::<f64>() + NORM_EPSILON).sqrt();
    g.map(|v| v / r)
}

/// Left Hamilton product `q ⊗ u`, the same lane order as the Q31 rotation.
fn hamilton(q: [f64; 4], u: [f64; 4]) -> [f64; 4] {
    [
        q[0] * u[0] - q[1] * u[1] - q[2] * u[2] - q[3] * u[3],
        q[0] * u[1] + q[1] * u[0] + q[2] * u[3] - q[3] * u[2],
        q[0] * u[2] - q[1] * u[3] + q[2] * u[0] + q[3] * u[1],
        q[0] * u[3] + q[1] * u[2] - q[2] * u[1] + q[3] * u[0],
    ]
}

/// Fold: coordinate `c` sums rotated lanes `16·k + c`.
fn reference_address(gates: &[[f64; 4]], row: &[f64]) -> [f64; GROUPS] {
    let mut address = [0.0; GROUPS];
    for (j, (gate, block)) in gates.iter().zip(row.chunks_exact(4)).enumerate() {
        let rotated = hamilton(*gate, [block[0], block[1], block[2], block[3]]);
        let base = (4 * j) % GROUPS;
        for (k, lane) in rotated.into_iter().enumerate() {
            address[base + k] += lane;
        }
    }
    address
}

fn reference_score(query: &[f64; GROUPS], key: &[f64; GROUPS]) -> f64 {
    query
        .iter()
        .zip(key)
        .map(|(q, k)| (q - k).abs())
        .sum::<f64>()
        / GROUPS as f64
}

/// The f64 reference of one window.
struct WindowReference {
    /// `s + 1` per row, `0` at row 0.
    picks: Vec<u32>,
    /// `scores[t][s]` for `s < t`, in float units.
    scores: Vec<Vec<f64>>,
    /// `x + w·e` at the reference pick, `time * width`.
    out: Vec<f64>,
}

/// The f64 side of one state: gates, weight and table as f64.
#[derive(Clone, Copy)]
struct Reference<'a> {
    width: usize,
    query_gates: &'a [[f64; 4]],
    key_gates: &'a [[f64; 4]],
    weights: &'a [f64],
    table: &'a [f64],
}

impl Reference<'_> {
    fn window(&self, stream: &[f64], ids: &[u32]) -> WindowReference {
        let width = self.width;
        let time = ids.len();
        let rows: Vec<&[f64]> = stream.chunks_exact(width).collect();
        let queries: Vec<_> = rows
            .iter()
            .map(|row| reference_address(self.query_gates, row))
            .collect();
        let keys: Vec<_> = rows
            .iter()
            .map(|row| reference_address(self.key_gates, row))
            .collect();
        let mut picks = vec![0u32; time];
        let mut scores = vec![Vec::new(); time];
        let mut out = stream.to_vec();
        for t in 1..time {
            let row_scores: Vec<f64> = (0..t)
                .map(|s| reference_score(&queries[t], &keys[s]))
                .collect();
            let mut best = 0usize;
            for (s, &score) in row_scores.iter().enumerate() {
                // `<=` sends an exact tie to the latest earlier row.
                if score <= row_scores[best] {
                    best = s;
                }
            }
            picks[t] = (best + 1) as u32;
            let id = ids[best + 1] as usize;
            let payload = &self.table[id * width..(id + 1) * width];
            for ((word, w), e) in out[t * width..(t + 1) * width]
                .iter_mut()
                .zip(self.weights)
                .zip(payload)
            {
                *word += w * e;
            }
            scores[t] = row_scores;
        }
        WindowReference { picks, scores, out }
    }
}

/// Round to nearest at `2^exponent`; a word past `2^31` is clipped and counted.
fn to_q31(value: f64, exponent: i32, clips: &mut u64) -> i64 {
    let rounded = (value * pow2(exponent)).round();
    if rounded.abs() > Q31_UNIT {
        *clips += 1;
        if rounded > 0.0 {
            1 << 31
        } else {
            -(1 << 31)
        }
    } else {
        rounded as i64
    }
}

/// Exact i128 recompute of each rotated lane; mirrors `q31_hamilton_rotate`
/// (products, `+ 2^30`, arithmetic `>> 31`) with no clamp.
fn rotated_lane_clips(gates: &[Q31Quaternion], row: &[i64]) -> u64 {
    let mut clips = 0;
    for (g, b) in gates.iter().zip(row.chunks_exact(4)) {
        let p = |i: usize, k: usize| i128::from(g[i]) * i128::from(b[k]);
        let lanes = [
            p(0, 0) - p(1, 1) - p(2, 2) - p(3, 3),
            p(0, 1) + p(1, 0) + p(2, 3) - p(3, 2),
            p(0, 2) - p(1, 3) + p(2, 0) + p(3, 1),
            p(0, 3) + p(1, 2) - p(2, 1) + p(3, 0),
        ];
        for lane in lanes {
            if ((lane + (1 << 30)) >> 31).abs() > Q31_LIMIT {
                clips += 1;
            }
        }
    }
    clips
}

/// The Q31 side of one state.
struct Native {
    width: usize,
    weights: Vec<i64>,
    table: Vec<i64>,
}

impl Native {
    /// Exact i128 payload and `x + payload` at the given picks and gain.
    fn out_clips(&self, stream: &[i64], ids: &[u32], picks: &[u32], gain_shift: u32) -> u64 {
        let width = self.width;
        let shift = MAX_GAIN_SHIFT - gain_shift;
        let round: i128 = if shift > 0 { 1 << (shift - 1) } else { 0 };
        let mut clips = 0;
        for (t, &pick) in picks.iter().enumerate().skip(1) {
            let id = ids[pick as usize] as usize;
            let payload = &self.table[id * width..(id + 1) * width];
            let row = &stream[t * width..(t + 1) * width];
            for ((&x, &w), &e) in row.iter().zip(&self.weights).zip(payload) {
                let p = (i128::from(w) * i128::from(e) + round) >> shift;
                if p.abs() > Q31_LIMIT || (i128::from(x) + p).abs() > Q31_LIMIT {
                    clips += 1;
                }
            }
        }
        clips
    }
}

struct OutWordTally {
    gain_shift: u32,
    bound_lsb: f64,
    checked: u64,
    above: u64,
    above_4x: u64,
    worst: f64,
}

impl OutWordTally {
    fn new(gain_shift: u32) -> Self {
        Self {
            gain_shift,
            bound_lsb: pow2(gain_shift as i32) + 2.0,
            checked: 0,
            above: 0,
            above_4x: 0,
            worst: 0.0,
        }
    }

    fn add(&mut self, native: &[i64], reference: impl Iterator<Item = f64>, e_x: i32) {
        let scale = pow2(e_x);
        for (&word, expected) in native.iter().zip(reference) {
            let miss = (word as f64 - expected * scale).abs();
            self.checked += 1;
            if miss > self.bound_lsb {
                self.above += 1;
            }
            if miss > 4.0 * self.bound_lsb {
                self.above_4x += 1;
            }
            self.worst = self.worst.max(miss);
        }
    }

    fn finish(self) -> OutWordCheck {
        OutWordCheck {
            gain_shift: self.gain_shift,
            bound_lsb: self.bound_lsb,
            words_checked: self.checked,
            words_above_bound: self.above,
            words_above_4x_bound: self.above_4x,
            worst_miss_lsb: self.worst,
            pass: self.above == 0,
        }
    }
}

fn top_two_gap(scores: &[f64]) -> Option<f64> {
    if scores.len() < 2 {
        return None;
    }
    let (mut first, mut second) = (f64::INFINITY, f64::INFINITY);
    for &score in scores {
        if score < first {
            second = first;
            first = score;
        } else if score < second {
            second = score;
        }
    }
    Some(second - first)
}

fn finite(values: &[f32]) -> bool {
    values.iter().all(|v| v.is_finite())
}

fn validate(model: &ReplayModel, windows: &[ReplayWindow]) -> Result<usize> {
    let width = model.width;
    if width == 0 || !width.is_multiple_of(4) {
        return Err(invalid("width must be a nonzero multiple of 4"));
    }
    if model.query_gates_raw.len() != width / 4 || model.key_gates_raw.len() != width / 4 {
        return Err(invalid("needs width / 4 query gates and key gates"));
    }
    if model.weights.len() != width {
        return Err(invalid("needs width weights"));
    }
    if model.table.is_empty() || !model.table.len().is_multiple_of(width) {
        return Err(invalid("table must be a nonempty multiple of width"));
    }
    let gates_finite = model
        .query_gates_raw
        .iter()
        .chain(&model.key_gates_raw)
        .all(|g| finite(g));
    if !gates_finite || !finite(&model.weights) || !finite(&model.table) {
        return Err(invalid("gates, weights and table must be finite"));
    }
    let vocab = model.table.len() / width;
    for window in windows {
        let time = window.time;
        if time == 0
            || window.stream.len() != time * width
            || window.ids.len() != time
            || window.f32_first_pick.len() != time
            || window.f32_out.len() != time * width
            || window.query.len() != time
        {
            return Err(invalid(
                "each window needs time rows of stream, ids, picks, out and query",
            ));
        }
        if !finite(&window.stream) || !finite(&window.f32_out) {
            return Err(invalid("stream and f32 out must be finite"));
        }
        if window.ids.iter().skip(1).any(|&id| id as usize >= vocab) {
            return Err(invalid("every readable id must be below the table vocab"));
        }
    }
    Ok(vocab)
}

fn refused(width: usize, reason: &str) -> ReplaySummary {
    ReplaySummary {
        width,
        refusal: Some(reason.into()),
        run: None,
    }
}

fn replay_inner(
    model: &ReplayModel,
    windows: &[ReplayWindow],
    swap_reference_gates: bool,
    dump: Option<&mut Option<Q31ReadDump>>,
) -> Result<ReplaySummary> {
    validate(model, windows)?;
    let width = model.width;
    if width != REPLAY_WIDTH {
        return Ok(refused(
            width,
            "width is not 256; the bound B is derived for 16 fold groups",
        ));
    }

    let weights_f64: Vec<f64> = model.weights.iter().map(|&v| f64::from(v)).collect();
    let table_f64: Vec<f64> = model.table.iter().map(|&v| f64::from(v)).collect();
    let streams_f64: Vec<Vec<f64>> = windows
        .iter()
        .map(|w| w.stream.iter().map(|&v| f64::from(v)).collect())
        .collect();

    // The headroom maxima, over every row and every pick the operator could make.
    let (mut max_x, mut max_we, mut max_16we) = (0f64, 0f64, 0f64);
    for (window, stream) in windows.iter().zip(&streams_f64) {
        for (t, row) in stream.chunks_exact(width).enumerate() {
            max_x = row.iter().fold(max_x, |m, x| m.max(x.abs()));
            for s in 0..t {
                let id = window.ids[s + 1] as usize;
                let payload = &table_f64[id * width..(id + 1) * width];
                for ((x, w), e) in row.iter().zip(&weights_f64).zip(payload) {
                    max_we = max_we.max((x + w * e).abs());
                    max_16we = max_16we.max((x + 16.0 * w * e).abs());
                }
            }
        }
    }
    let max_table = table_f64.iter().fold(0f64, |m, e| m.max(e.abs()));
    let max_weight = weights_f64.iter().fold(0f64, |m, w| m.max(w.abs()));

    let e_x = largest_exponent(max_x.max(max_we).max(max_16we), HEADROOM);
    let e_t = largest_exponent(max_table, Q31_UNIT);
    let Some(gain_shift) =
        (0..=MAX_GAIN_SHIFT).find(|&g| max_weight * pow2(31 + e_x - e_t - g as i32) <= Q31_UNIT)
    else {
        return Ok(refused(
            width,
            "no gain shift g in 0..=31 has max|w| · 2^e_w <= 2^31",
        ));
    };
    let e_w = 31 + e_x - e_t - gain_shift as i32;
    let scales = ReplayScales {
        e_x,
        e_t,
        e_w,
        gain_shift,
        gate_scale_log2: 31,
        max_abs_x: max_x,
        max_abs_x_plus_we: max_we,
        max_abs_x_plus_16we: max_16we,
        max_abs_table: max_table,
        max_abs_weight: max_weight,
    };

    // The gates, normalized once in f64 and shared by both sides.
    let query_f64: Vec<[f64; 4]> = model.query_gates_raw.iter().map(normalize_gate).collect();
    let key_f64: Vec<[f64; 4]> = model.key_gates_raw.iter().map(normalize_gate).collect();
    let mut input_clips = 0u64;
    let mut gate_q31 = |gates: &[[f64; 4]]| -> Vec<Q31Quaternion> {
        gates
            .iter()
            .map(|g| g.map(|v| to_q31(v, 31, &mut input_clips)))
            .collect()
    };
    let query_q31 = gate_q31(&query_f64);
    let key_q31 = gate_q31(&key_f64);
    let native = Native {
        width,
        weights: weights_f64
            .iter()
            .map(|&w| to_q31(w, e_w, &mut input_clips))
            .collect(),
        table: table_f64
            .iter()
            .map(|&e| to_q31(e, e_t, &mut input_clips))
            .collect(),
    };
    let streams_q31: Vec<Vec<i64>> = streams_f64
        .iter()
        .map(|s| {
            s.iter()
                .map(|&x| to_q31(x, e_x, &mut input_clips))
                .collect()
        })
        .collect();

    let operator = |keys: Vec<Q31Quaternion>, gain: u32| {
        Q31FirstPickRead::new(query_q31.clone(), keys, native.weights.clone(), gain)
            .map_err(|err| invalid(&err.to_string()))
    };
    let read = operator(key_q31.clone(), gain_shift)?;
    if let Some(slot) = dump {
        let time = windows.first().map_or(0, |w| w.time);
        if windows.iter().any(|w| w.time != time) {
            return Err(invalid("a read dump needs every window at one length"));
        }
        *slot = Some(Q31ReadDump {
            read: read.clone(),
            batch: windows.len(),
            time,
            stream: streams_q31.concat(),
            ids: windows.iter().flat_map(|w| w.ids.iter().copied()).collect(),
            table: native.table.clone(),
        });
    }
    let control_read = operator(vec![CONTROL_KEY_GATE; width / 4], gain_shift)?;
    let x16_read = match gain_shift + 4 {
        g if g <= MAX_GAIN_SHIFT => Some(operator(key_q31.clone(), g)?),
        _ => None,
    };

    let (ref_query, ref_key) = if swap_reference_gates {
        (&key_f64, &query_f64)
    } else {
        (&query_f64, &key_f64)
    };
    let reference = Reference {
        width,
        query_gates: ref_query,
        key_gates: ref_key,
        weights: &weights_f64,
        table: &table_f64,
    };
    // The control gate [2^30; 4] in Q31 is [0.5; 4] in float units.
    let control_keys = vec![[0.5f64; 4]; width / 4];
    let control_reference = Reference {
        key_gates: &control_keys,
        ..reference
    };

    let two_b = 2.0 * SCORE_BOUND_LSB;
    let lsb = pow2(e_x);
    let b_f = pow2(-12) * max_x;
    let mut clips = ClipCounts {
        input_words: input_clips,
        rotated_lanes: 0,
        out_words: 0,
    };
    let mut x16_clips = 0u64;
    let mut out_words = OutWordTally::new(gain_shift);
    let mut out_words_x16 = x16_read.as_ref().map(|r| OutWordTally::new(r.gain_shift()));
    let mut structural = StructuralCheck {
        rows: 0,
        failures: 0,
        pass: true,
    };
    let mut control_1 = ControlOne {
        mismatches: 0,
        mismatches_above_2b: 0,
        near_ties: 0,
        agree_with_unsubstituted: 0,
        a_pass: true,
        b_pass: true,
    };
    let mut control_4 = ControlFour {
        b_f,
        disagreements: Vec::new(),
        disagreements_above_2bf: 0,
        invalid_f32_picks: 0,
        out_words_checked: 0,
        out_words_above_bound: 0,
        a_pass: true,
        b_pass: true,
    };
    let (mut query_rows, mut picks_agree, mut near_ties) = (0u64, 0u64, 0u64);
    let (mut native_vs_f32_agree, mut f32_vs_f64_disagree) = (0u64, 0u64);
    let mut pick_mismatches = Vec::new();

    for (index, window) in windows.iter().enumerate() {
        let time = window.time;
        let stream = &streams_q31[index];
        let stream_f64 = &streams_f64[index];
        let apply = |r: &Q31FirstPickRead| {
            r.apply(stream, &window.ids, &native.table, 1, time)
                .map_err(|err| invalid(&err.to_string()))
        };
        let output = apply(&read)?;
        let control_output = apply(&control_read)?;
        let x16_output = x16_read.as_ref().map(apply).transpose()?;

        for row in stream.chunks_exact(width) {
            clips.rotated_lanes += rotated_lane_clips(&query_q31, row);
            clips.rotated_lanes += rotated_lane_clips(&key_q31, row);
        }
        clips.out_words += native.out_clips(stream, &window.ids, &output.picks, gain_shift);
        if let (Some(r), Some(o)) = (&x16_read, &x16_output) {
            x16_clips += native.out_clips(stream, &window.ids, &o.picks, r.gain_shift());
        }

        structural.rows += 1;
        if output.picks[0] != 0 || output.out[..width] != stream[..width] {
            structural.failures += 1;
        }

        // The comparison fields are read only from here on.
        let ref_window = reference.window(stream_f64, &window.ids);
        let control_window = control_reference.window(stream_f64, &window.ids);
        let gap = |w: &WindowReference, t: usize, a: u32, b: u32| {
            (w.scores[t][a as usize - 1] - w.scores[t][b as usize - 1]).abs()
        };
        for t in (0..time).filter(|&t| window.query[t]) {
            query_rows += 1;
            let lanes = t * width..(t + 1) * width;
            let (np, rp) = (output.picks[t], ref_window.picks[t]);
            if np == rp {
                picks_agree += 1;
                out_words.add(
                    &output.out[lanes.clone()],
                    ref_window.out[lanes.clone()].iter().copied(),
                    e_x,
                );
            } else {
                pick_mismatches.push(PickGap {
                    window: index,
                    row: t,
                    reference_pick: rp,
                    other_pick: np,
                    gap: gap(&ref_window, t, rp, np) * lsb,
                });
            }
            if top_two_gap(&ref_window.scores[t]).is_some_and(|g| g * lsb <= two_b) {
                near_ties += 1;
            }

            if let (Some(tally), Some(o)) = (out_words_x16.as_mut(), &x16_output) {
                if o.picks[t] == rp {
                    let x16 = lanes.clone().map(|l| {
                        let x = stream_f64[l];
                        x + 16.0 * (ref_window.out[l] - x)
                    });
                    tally.add(&o.out[lanes.clone()], x16, e_x);
                }
            }

            let (cp, crp) = (control_output.picks[t], control_window.picks[t]);
            if cp != crp {
                control_1.mismatches += 1;
                if gap(&control_window, t, crp, cp) * lsb > two_b {
                    control_1.mismatches_above_2b += 1;
                }
            }
            if top_two_gap(&control_window.scores[t]).is_some_and(|g| g * lsb <= two_b) {
                control_1.near_ties += 1;
            }
            if cp == np {
                control_1.agree_with_unsubstituted += 1;
            }

            let fp = window.f32_first_pick[t];
            if fp == np {
                native_vs_f32_agree += 1;
            }
            let valid = if t == 0 {
                fp == 0
            } else {
                (1..=t as u32).contains(&fp)
            };
            if !valid {
                control_4.invalid_f32_picks += 1;
                f32_vs_f64_disagree += 1;
            } else if fp != rp {
                f32_vs_f64_disagree += 1;
                let f32_gap = gap(&ref_window, t, rp, fp);
                if f32_gap > 2.0 * b_f {
                    control_4.disagreements_above_2bf += 1;
                }
                control_4.disagreements.push(PickGap {
                    window: index,
                    row: t,
                    reference_pick: rp,
                    other_pick: fp,
                    gap: f32_gap,
                });
            } else {
                for l in lanes {
                    let x = stream_f64[l];
                    let we = ref_window.out[l] - x;
                    let bound = pow2(-22) * (x.abs() + we.abs());
                    control_4.out_words_checked += 1;
                    if (f64::from(window.f32_out[l]) - ref_window.out[l]).abs() > bound {
                        control_4.out_words_above_bound += 1;
                    }
                }
            }
        }
    }

    structural.pass = structural.failures == 0;
    control_1.a_pass = control_1.mismatches_above_2b == 0;
    control_1.b_pass = control_1.agree_with_unsubstituted * 100 < 99 * query_rows;
    control_4.a_pass = control_4.disagreements_above_2bf == 0 && control_4.invalid_f32_picks == 0;
    control_4.b_pass = control_4.out_words_above_bound == 0;
    let picks_required = (99 * query_rows).div_ceil(100);
    let picks_pass = picks_agree >= picks_required;
    let mismatches_above_2b = pick_mismatches.iter().filter(|m| m.gap > two_b).count() as u64;
    let out_words = out_words.finish();
    let bar_pass = picks_pass && out_words.pass && clips.total() == 0;
    let definition_rows_present = mismatches_above_2b > 0 || out_words.words_above_4x_bound > 0;
    Ok(ReplaySummary {
        width,
        refusal: None,
        run: Some(ReplayRun {
            scales,
            windows: windows.len() as u64,
            query_rows,
            picks_agree,
            picks_required,
            picks_pass,
            pick_mismatches,
            mismatches_above_2b,
            near_ties,
            out_words,
            clips,
            structural,
            control_1,
            control_4,
            native_vs_f32_agree,
            f32_vs_f64_disagree,
            out_words_x16: out_words_x16.map(OutWordTally::finish),
            out_word_clips_x16: x16_read.as_ref().map(|_| x16_clips),
            bar_pass,
            definition_rows_present,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct XorShift(u64);

    impl XorShift {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }

        /// Uniform in `[-1, 1]` on a 2^20 grid.
        fn unit(&mut self) -> f32 {
            ((self.next() % (1 << 21)) as f32 - (1 << 20) as f32) / (1 << 20) as f32
        }

        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    /// Fill the comparison fields from the f64 definition, as an f32 read that
    /// followed the definition would have.
    fn capture(
        model: &ReplayModel,
        streams: Vec<Vec<f32>>,
        ids: Vec<Vec<u32>>,
    ) -> Vec<ReplayWindow> {
        let query: Vec<[f64; 4]> = model.query_gates_raw.iter().map(normalize_gate).collect();
        let key: Vec<[f64; 4]> = model.key_gates_raw.iter().map(normalize_gate).collect();
        let weights: Vec<f64> = model.weights.iter().map(|&v| f64::from(v)).collect();
        let table: Vec<f64> = model.table.iter().map(|&v| f64::from(v)).collect();
        let reference = Reference {
            width: model.width,
            query_gates: &query,
            key_gates: &key,
            weights: &weights,
            table: &table,
        };
        streams
            .into_iter()
            .zip(ids)
            .map(|(stream, ids)| {
                let x: Vec<f64> = stream.iter().map(|&v| f64::from(v)).collect();
                let window = reference.window(&x, &ids);
                ReplayWindow {
                    time: ids.len(),
                    f32_first_pick: window.picks,
                    f32_out: window.out.iter().map(|&v| v as f32).collect(),
                    query: vec![true; ids.len()],
                    stream,
                    ids,
                }
            })
            .collect()
    }

    fn random_capture(
        seed: u64,
        windows: usize,
        time: usize,
        vocab: usize,
        weight_scale: f32,
    ) -> (ReplayModel, Vec<ReplayWindow>) {
        let width = REPLAY_WIDTH;
        let mut rng = XorShift(seed);
        let gate = |rng: &mut XorShift| [rng.unit(), rng.unit(), rng.unit(), rng.unit()];
        let model = ReplayModel {
            width,
            query_gates_raw: (0..width / 4).map(|_| gate(&mut rng)).collect(),
            key_gates_raw: (0..width / 4).map(|_| gate(&mut rng)).collect(),
            weights: (0..width).map(|_| weight_scale * rng.unit()).collect(),
            table: (0..vocab * width).map(|_| rng.unit()).collect(),
        };
        let streams = (0..windows)
            .map(|_| (0..time * width).map(|_| rng.unit()).collect())
            .collect();
        let ids = (0..windows)
            .map(|_| (0..time).map(|_| rng.below(vocab as u64) as u32).collect())
            .collect();
        let windows = capture(&model, streams, ids);
        (model, windows)
    }

    fn run(summary: ReplaySummary) -> ReplayRun {
        assert_eq!(summary.refusal, None);
        summary.run.expect("a run")
    }

    #[test]
    fn t1_exactly_representable_capture_passes_the_bar() {
        let width = REPLAY_WIDTH;
        let (windows, time, vocab) = (4, 32, 7);
        let mut rng = XorShift(0x5EED_0001);
        // Signed basis quaternions rotate by exact lane permutations.
        let basis = |rng: &mut XorShift| {
            let mut g = [0.0f32; 4];
            g[rng.below(4) as usize] = if rng.below(2) == 0 { 1.0 } else { -1.0 };
            g
        };
        let model = ReplayModel {
            width,
            query_gates_raw: (0..width / 4).map(|_| basis(&mut rng)).collect(),
            key_gates_raw: (0..width / 4).map(|_| basis(&mut rng)).collect(),
            weights: (0..width)
                .map(|_| (rng.below(2) as f32 * 2.0 - 1.0) / (1 << rng.below(4)) as f32)
                .collect(),
            table: (0..vocab * width)
                .map(|_| (rng.below(129) as f32 - 64.0) / 64.0)
                .collect(),
        };
        let streams = (0..windows)
            .map(|_| {
                (0..time * width)
                    .map(|_| (rng.below(2049) as f32 - 1024.0) / 1024.0)
                    .collect()
            })
            .collect();
        let ids = (0..windows)
            .map(|_| (0..time).map(|_| rng.below(vocab as u64) as u32).collect())
            .collect();
        let captured = capture(&model, streams, ids);
        let run = run(replay(&model, &captured).unwrap());

        assert_eq!(run.query_rows, (windows * time) as u64);
        assert_eq!(run.picks_agree, run.query_rows, "{:?}", run.pick_mismatches);
        assert!(run.picks_pass && run.out_words.pass && run.bar_pass);
        assert_eq!(run.out_words.worst_miss_lsb, 0.0);
        assert_eq!(run.clips.total(), 0);
        assert!(run.structural.pass);
        assert_eq!(run.structural.rows, windows as u64);
        assert!(run.control_4.a_pass && run.control_4.b_pass);
        assert_eq!(run.native_vs_f32_agree, run.query_rows);
        assert!(!run.definition_rows_present);
        let x16 = run.out_words_x16.as_ref().expect("g + 4 <= 31");
        assert!(x16.pass);
        assert_eq!(run.out_word_clips_x16, Some(0));
        assert_eq!(run.scales.gain_shift, 0);
        serde_json::to_string(&run).unwrap();
    }

    #[test]
    fn t2_swapped_reference_gates_fail_control_4_and_the_bar() {
        let (model, captured) = random_capture(0x5EED_0002, 2, 32, 11, 0.1);
        let straight = run(replay(&model, &captured).unwrap());
        assert!(straight.control_4.a_pass && straight.control_4.b_pass);
        assert!(straight.bar_pass);

        let swapped = run(replay_inner(&model, &captured, true, None).unwrap());
        assert!(!swapped.control_4.a_pass, "{:?}", swapped.control_4);
        assert!(swapped.control_4.disagreements_above_2bf > 0);
        assert!(!swapped.bar_pass);
        assert!(swapped.definition_rows_present);
    }

    #[test]
    fn t3_near_ties_are_counted() {
        let width = REPLAY_WIDTH;
        let identity = vec![[1.0f32, 0.0, 0.0, 0.0]; width / 4];
        let model = ReplayModel {
            width,
            query_gates_raw: identity.clone(),
            key_gates_raw: identity,
            weights: vec![0.0; width],
            table: vec![0.25; 2 * width],
        };
        // Row 3 sits 2^-24 nearer row 2 than row 1: 8 LSB apart at e_x = 30.
        let mut stream = vec![0.0f32; 4 * width];
        stream[width] = 0.25;
        stream[2 * width] = 0.75;
        stream[3 * width] = 0.5 + 2f32.powi(-24);
        let captured = capture(&model, vec![stream], vec![vec![0, 1, 0, 1]]);
        let run = run(replay(&model, &captured).unwrap());
        assert_eq!(run.scales.e_x, 30);
        assert!(run.near_ties > 0, "near ties {}", run.near_ties);
        assert_eq!(run.picks_agree, 4);
    }

    /// The f64 reference's own tie rule, asserted directly: `capture` fills the f32 fields from
    /// the same reference, so a reference with the wrong tie rule would agree with itself.
    #[test]
    fn the_reference_sends_an_exact_tie_to_the_latest_row() {
        let width = REPLAY_WIDTH;
        let identity = vec![[1.0f32, 0.0, 0.0, 0.0]; width / 4];
        let model = ReplayModel {
            width,
            query_gates_raw: identity.clone(),
            key_gates_raw: identity,
            weights: vec![0.0; width],
            table: vec![0.25; 2 * width],
        };
        // Rows 1, 2 and 3 are equal, so row 3's scores against rows 1 and 2 tie exactly at 0.
        let mut stream = vec![0.0f32; 4 * width];
        for row in 1..4 {
            stream[row * width] = 0.5;
        }
        let captured = capture(&model, vec![stream], vec![vec![0, 1, 0, 1]]);
        assert_eq!(captured[0].f32_first_pick, vec![0, 1, 2, 3]);
        let run = run(replay(&model, &captured).unwrap());
        assert_eq!(run.picks_agree, 4, "{:?}", run.pick_mismatches);
    }

    #[test]
    fn t4_control_1_moves_the_picks_on_a_random_capture() {
        let (model, captured) = random_capture(0x5EED_0004, 8, 64, 50, 0.1);
        let run = run(replay(&model, &captured).unwrap());
        assert_eq!(run.query_rows, 8 * 64);
        assert!(
            run.control_1.b_pass,
            "agree {} of {}",
            run.control_1.agree_with_unsubstituted, run.query_rows
        );
        assert!(run.control_1.a_pass);
        assert!(run.picks_pass && run.bar_pass, "{run:?}");
    }

    /// Control 2's input: the dump carries the barred operator and every window's converted
    /// words, survives its bytes, and its one batched apply equals each window's own apply, so a
    /// CPU-against-CUDA comparison on it covers the replay's real calls.
    #[test]
    fn the_dump_is_the_barred_call_batched() {
        let (model, captured) = random_capture(0x5EED_0006, 4, 32, 50, 0.1);
        let (summary, dump) = replay_with_dump(&model, &captured).unwrap();
        assert_eq!(summary, replay(&model, &captured).unwrap());
        let dump = dump.expect("a dump for a state that is not refused");
        let back = Q31ReadDump::from_bytes(&dump.to_bytes().unwrap()).unwrap();
        assert_eq!(back, dump);
        assert_eq!((dump.batch, dump.time), (4, 32));
        assert_eq!(dump.read.gain_shift(), run(summary).scales.gain_shift);
        let all = dump
            .read
            .apply(&dump.stream, &dump.ids, &dump.table, dump.batch, dump.time)
            .unwrap();
        let (width, time) = (dump.read.width(), dump.time);
        for b in 0..dump.batch {
            let rows = b * time..(b + 1) * time;
            let one = dump
                .read
                .apply(
                    &dump.stream[rows.start * width..rows.end * width],
                    &dump.ids[rows.clone()],
                    &dump.table,
                    1,
                    time,
                )
                .unwrap();
            assert_eq!(one.picks, all.picks[rows.clone()]);
            assert_eq!(one.out, all.out[rows.start * width..rows.end * width]);
        }
    }

    #[test]
    fn t5_binding_payload_headroom_has_no_out_word_clips() {
        let (mut model, _) = random_capture(0x5EED_0005, 1, 1, 9, 1.0);
        // Full-size weights on a tiny stream: the payload sets the scale.
        model.weights.iter_mut().for_each(|w| *w = 1.0);
        let mut rng = XorShift(0x5EED_0055);
        let streams = (0..2)
            .map(|_| (0..16 * REPLAY_WIDTH).map(|_| 0.01 * rng.unit()).collect())
            .collect();
        let ids = (0..2)
            .map(|_| (0..16).map(|_| rng.below(9) as u32).collect())
            .collect();
        let captured = capture(&model, streams, ids);
        let run = run(replay(&model, &captured).unwrap());
        let s = &run.scales;
        assert!(s.max_abs_x_plus_16we > 100.0 * s.max_abs_x);
        assert!(s.max_abs_x_plus_16we * pow2(s.e_x) > HEADROOM / 2.0);
        assert_eq!(run.clips.out_words, 0);
        assert_eq!(run.out_word_clips_x16, Some(0));
        assert_eq!(run.clips.total(), 0);
        assert!(run.out_words.pass);
        assert!(run.out_words_x16.expect("x16 runs").pass);
    }

    #[test]
    fn refusals_are_flagged_and_malformed_captures_are_errors() {
        let (model, captured) = random_capture(0x5EED_0006, 1, 8, 5, 0.1);
        let mut narrow = model.clone();
        narrow.width = 16;
        narrow.query_gates_raw.truncate(4);
        narrow.key_gates_raw.truncate(4);
        narrow.weights.truncate(16);
        narrow.table.truncate(5 * 16);
        let narrow_window = ReplayWindow {
            time: 2,
            stream: vec![0.5; 32],
            ids: vec![0, 1],
            f32_first_pick: vec![0, 1],
            f32_out: vec![0.5; 32],
            query: vec![true; 2],
        };
        let summary = replay(&narrow, &[narrow_window]).unwrap();
        assert!(summary.refusal.unwrap().contains("256"));
        assert!(summary.run.is_none());

        // A weight far above the table's reach leaves no gain shift that fits.
        let mut heavy = model.clone();
        heavy.weights[0] = 1e30;
        for row in heavy.table.chunks_exact_mut(REPLAY_WIDTH) {
            row[0] = 0.0;
        }
        let summary = replay(&heavy, &captured).unwrap();
        assert!(summary.refusal.unwrap().contains("gain shift"));

        let mut bad = captured.clone();
        bad[0].ids[3] = 5;
        assert!(replay(&model, &bad).is_err());
        let mut short = captured;
        short[0].query.pop();
        assert!(replay(&model, &short).is_err());
    }
}
