//! The induction-keyed Hamming read inside the in-space trunk (in-space-trunk-to-text, claude-146).
//!
//! The trunk is a selective quaternion recurrence: a token 16 back moves the prediction by 1–2%
//! of what the last token does. Reads keyed on the trunk's own state never learned where to look
//! (claude-144, claude-145, and this read's first form). The project's 26 Sep retrieval head
//! (`archive/integration/geometric-attention-2026-09-26.md` §3) recalled 100% at 512 pairs with
//! Hamming codes and hard top-1 because its key named the *previous* token and its query the
//! current one: an earlier copy of the current token is then an exact match from the first step.
//! This read builds that in ([RESEARCH](../../../docs/loop/tasks/in-space-trunk-to-text/claude-146/RESEARCH.md)).
//!
//! After layer 2 of 6, inside the stream, and on the final norm's output, the points the nearest
//! readout decodes ([`HAMMING_DEPTHS`]), for each position `t` of a window with point `x_t`, and
//! for each of `heads` heads matching on its own block of coordinates:
//! 1. **Points.** The key of an earlier position `s` is the shared table's point of token
//!    `x_{s−1}`, rotated lane by lane by a learned unit quaternion. The query is, for the first
//!    half of the heads (**copy heads**), the point of the current token `x_t` through that same
//!    rotation, so a repeated token is an exact match however the rotation learns; for the rest
//!    (**type heads**), the residual through a second rotation, which must learn what to ask for.
//!    No point is turned by an index code: the owner's direction is SpiralCore's transport spin
//!    for position, not a rotary code ("we do not need rope", 13 Oct). With the type spin on
//!    ([`HammingReads::with_type_spin`], claude-151), the type heads compare positions through
//!    that transport's quaternion form: a frame per position, the running product of each token's
//!    icosian step, turned out of each position's own frame, so the angle between two positions'
//!    points depends only on the steps of the tokens between them. Their sign codes also see the
//!    frame the two share, so the Hamming distance follows those steps on average over it, not
//!    exactly. The value is `x` rotated by a third quaternion.
//!
//!    With copy key 2 ([`HammingReads::with_copy_key`], claude-261), a copy head's key names the
//!    two tokens before `s`, in order: the first half of each copy head's coordinates is the key
//!    of `x_{s−1}`, the second half the key of `x_{s−2}`, through the same rotation. Its query
//!    is `x_t`'s half and then `x_{t−1}`'s. The distance below is then the sum of the two tokens'
//!    half-distances, so only a repeat of both tokens is an exact match. A one-token key read
//!    every position after an earlier " is" alike ("my friend is" and "my number is", claude-259's
//!    trace). Where a window has fewer tokens before `s`, the second token is the first one's.
//!    Each copy head may take its own key ([`HammingReads::with_copy_keys`], claude-268), so one
//!    head can match one token back and another two.
//! 2. **Codes.** Each coordinate's bit is 1 at or above 0, as [`pack_signs`] packs it. The forward
//!    pass compares the ±1 codes, so the distance is exactly the Hamming distance
//!    `popcount(q XOR k)` that serving computes. The backward pass treats the code as the identity
//!    (straight through), so the distance's gradient `(a − b) / 2` reaches the rotated points.
//! 3. **Choice.** The [`HAMMING_KEPT`] earlier positions `1 ≤ s < t` nearest in Hamming distance,
//!    hard and detached, by one sort per head ([`choose_sorted`]). Position 0 has no previous token.
//! 4. **Weights.** `max(0, m − d) / m`, between 0 and 1, with `d` in bits and a learned margin `m`
//!    per head: an exact match counts in full, every candidate inside the margin gets graded
//!    credit, and nothing is exponential. A head reports the mean over the candidates it reads (a
//!    count), and reads nothing when none is inside its margin.
//! 5. **Payload,** at full width. Because the key at `s` named token `s − 1`, the kept position
//!    holds what followed the earlier match. A copy head brings back the shared table's point of
//!    token `x_s` itself: the token after the last copy of the current token, as the very point the
//!    nearest readout names it by (an induction head with a tied readout, built in rather than
//!    grown; the residual there failed the recall probe, since the trunk reads a token point added
//!    to its stream as its own input, not as its answer). A type head brings back `v_s`.
//! 6. **Combine.** `c_t` and `y_t` are the copy and type heads' means over the heads that found a
//!    candidate, each with its own weight row, both starting at 0, so at step 0 the trunk is
//!    unchanged. In the stream, `x_t + w_c ⊙ c_t + w_y ⊙ y_t`. On the readout, where a copy head
//!    found something, `n_t + w_c ⊙ (c_t − n_t) + w_y ⊙ y_t`: a nearest readout names the point a
//!    prediction is at, so a copy moves the prediction toward the copied point (at `w_c = 1`,
//!    onto it). Adding the point instead failed the recall probe: next to the stream's larger
//!    parts it barely moves where the prediction lands.
//!
//! The reads are active for the thread that sets them ([`set_hamming_reads`]); every caller of
//! `StackModel::hidden` on that thread gets them. They train with claude-144's AdamW.
use std::cell::RefCell;
use std::path::Path;

use candle_core::{DType, Device, Tensor, Var};
use uor_r4_integer::fbs_snapshot::{HammingTraceHead, HammingTraceRead, HammingTraceWindow};

use crate::geometric_stack::hamilton_gate;
use crate::{invalid, Result};

/// Each read runs after this many of the trunk's 6 layers; the last one, on the readout.
pub const HAMMING_DEPTHS: [usize; 2] = [2, 6];
/// The read at this depth runs on the final norm's output ([`HammingRead::apply_readout`]).
pub const HAMMING_READOUT_DEPTH: usize = HAMMING_DEPTHS[1];
/// Earlier positions each head keeps.
pub const HAMMING_KEPT: usize = 16;
const FILE: &str = "hamming_read.safetensors";

thread_local! {
    static ACTIVE: RefCell<Option<HammingReads>> = const { RefCell::new(None) };
    /// How many times [`set_hamming_reads`] has run on this thread (claude-281): the decode
    /// cache's prefill records it, and a step refuses reads set since.
    static EPOCH: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static CUT: std::cell::Cell<Option<(ReadScope, usize)>> = const { std::cell::Cell::new(None) };
    static ROW_CUT: RefCell<Option<(ReadScope, Vec<usize>)>> = const { RefCell::new(None) };
    static TIE: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static TRACE: RefCell<Option<Vec<HammingTraceWindow>>> = const { RefCell::new(None) };
    static COORDS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static CODE_FORCE: RefCell<Vec<ForcedRowCode>> = const { RefCell::new(Vec::new()) };
    static CLASS_OFF: std::cell::Cell<[[bool; 2]; 2]> =
        const { std::cell::Cell::new([[false; 2]; 2]) };
    static HOLD_OFF: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static CARRY_OFF: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static CAPTURE: RefCell<Option<RowCapture>> = const { RefCell::new(None) };
}

/// claude-334's `−H`: with `true`, a held read adds no held term in the forwards that follow on
/// this thread (`g := 0` at evaluation); its direct write and every other part read as before.
/// `false`, the default, holds as trained.
pub fn set_hold_off(off: bool) {
    HOLD_OFF.with(|h| h.set(off));
}

/// Whether [`set_hold_off`] last turned the hold off on this thread.
pub fn hold_off() -> bool {
    HOLD_OFF.with(|h| h.get())
}

/// claude-335's `−C`: with `true`, a carried read's ask heads ask without the carried term in
/// the forwards and pulls that follow on this thread (`c := 0` at evaluation); every other part
/// reads as before. `false`, the default, carries as trained.
pub fn set_carry_off(off: bool) {
    CARRY_OFF.with(|c| c.set(off));
}

/// Whether [`set_carry_off`] last turned the carry off on this thread.
pub fn carry_off() -> bool {
    CARRY_OFF.with(|c| c.get())
}

/// claude-334: what the stream read computed at the captured rows, one vector per row, in the
/// order [`set_row_capture`] named them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StreamCapture {
    /// `y`, the type (ask) heads' mean payload.
    pub payload: Vec<Vec<f32>>,
    /// Whether any type (ask) head found a candidate inside its margin.
    pub found: Vec<bool>,
    /// `u = w_y ⊙ y`, the write (0 where none found, or under a class drop).
    pub write: Vec<Vec<f32>>,
    /// The held term (0 without a hold, or under [`set_hold_off`]).
    pub held: Vec<Vec<f32>>,
    /// The stream after the read and the hold, before the rounding.
    pub out: Vec<Vec<f32>>,
}

/// claude-334's capture of one forward: the rows of its `batch · time` rows to copy out, the
/// stream read's parts there, and the readout read's input there (the final point).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RowCapture {
    pub rows: Vec<usize>,
    pub stream: Option<StreamCapture>,
    pub readout_in: Option<Vec<Vec<f32>>>,
}

/// claude-334: copy the reads' parts at `rows` of the next forward on this thread to the host,
/// those rows only; `None`, the default, copies nothing. [`take_row_capture`] takes them.
pub fn set_row_capture(rows: Option<Vec<usize>>) {
    CAPTURE.with(|c| *c.borrow_mut() = rows.map(|rows| RowCapture { rows, ..RowCapture::default() }));
}

/// Takes what the capture copied and turns the capture off.
pub fn take_row_capture() -> Option<RowCapture> {
    CAPTURE.with(|c| c.borrow_mut().take())
}

/// The rows of `t` `(rows, width)` at `index`, on the host.
fn host_rows(t: &Tensor, index: &Tensor) -> Result<Vec<Vec<f32>>> {
    Ok(t.detach().index_select(index, 0)?.to_vec2::<f32>()?)
}

/// Copies the read's parts at the captured rows, when a capture is set: the stream read's
/// (`readout` false) or the readout read's input.
#[allow(clippy::too_many_arguments)]
fn capture_rows(
    readout: bool,
    input: &Tensor,
    payload: &Tensor,
    found: &Tensor,
    write: &Tensor,
    held: Option<&Tensor>,
    out: &Tensor,
) -> Result<()> {
    CAPTURE.with(|c| {
        let mut slot = c.borrow_mut();
        let Some(capture) = slot.as_mut() else { return Ok(()) };
        let total = input.dim(0)?;
        if let Some(&row) = capture.rows.iter().find(|&&r| r >= total) {
            return Err(invalid(format!("a captured row {row} outside the forward's {total}")));
        }
        let rows: Vec<u32> = capture.rows.iter().map(|&r| r as u32).collect();
        let index = Tensor::from_vec(rows, capture.rows.len(), input.device())?;
        if readout {
            if capture.readout_in.is_some() {
                return Err(invalid("the capture met the readout read twice in one forward"));
            }
            capture.readout_in = Some(host_rows(input, &index)?);
            return Ok(());
        }
        if capture.stream.is_some() {
            return Err(invalid("the capture met the stream read twice in one forward"));
        }
        let found = found.detach().index_select(&index, 0)?.flatten_all()?.to_vec1::<f32>()?;
        let held = match held {
            Some(h) => host_rows(h, &index)?,
            None => vec![vec![0.0; write.dim(1)?]; capture.rows.len()],
        };
        capture.stream = Some(StreamCapture {
            payload: host_rows(payload, &index)?,
            found: found.into_iter().map(|f| f > 0.0).collect(),
            write: host_rows(write, &index)?,
            held,
            out: host_rows(out, &index)?,
        });
        Ok(())
    })
}

/// claude-328: drop classes of heads from the read's combine for the forward passes that follow
/// on this thread, for eval-only ablations: `off[read][kind]`, read 0 the stream read and 1 the
/// readout read, kind 0 the copy heads and 1 the type heads. A class that is off adds nothing to
/// the point; its weights and its choices stay as they are. All `false`, the default, drops none.
pub fn set_class_ablation(off: [[bool; 2]; 2]) {
    CLASS_OFF.with(|c| c.set(off));
}

/// The classes [`set_class_ablation`] last dropped on this thread.
pub fn class_ablation() -> [[bool; 2]; 2] {
    CLASS_OFF.with(|c| c.get())
}

/// claude-253, Q5's forced replay: in the read at `depth`, row `row` of the forward's
/// `batch · time` rows and head `head`, the query's code (`key` false) or the key's is `code`,
/// one bit per coordinate as [`pack_signs`] packs it, in place of the computed one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForcedRowCode {
    pub depth: usize,
    pub row: usize,
    pub head: usize,
    pub key: bool,
    pub code: u64,
}

/// claude-253: force `codes` in every read on this thread from now on; an empty set, the
/// default, forces none.
pub fn set_code_force(codes: Vec<ForcedRowCode>) {
    CODE_FORCE.with(|f| *f.borrow_mut() = codes);
}

/// The ±1 `codes`, `(rows, heads · head_width)`, with each of `forced` that names this role
/// (`key`) unpacked into its row and head, a 1 bit +1 and a 0 bit −1. A coordinate whose forced
/// bit flips its sign takes the exact ±1; one whose bit agrees keeps its computed value, whose
/// last-bit noise (`p + (±1 − p)`) the margin weights read, so a code forced to itself changes
/// nothing.
fn force_codes(
    codes: &Tensor,
    forced: &[ForcedRowCode],
    key: bool,
    head_width: usize,
) -> Result<Tensor> {
    if head_width > 64 {
        return Err(invalid(
            "a forced code's head is wider than its 64-bit code (an instrument fault)",
        ));
    }
    let (rows, width) = codes.dims2()?;
    let mut signs = vec![0f32; rows * width];
    for f in forced.iter().filter(|f| f.key == key) {
        let past_width = head_width < 64 && f.code >> head_width != 0;
        if f.row >= rows || (f.head + 1) * head_width > width || past_width {
            return Err(invalid(
                "a forced code names a row, head or bit outside the read (an instrument fault)",
            ));
        }
        for i in 0..head_width {
            signs[f.row * width + f.head * head_width + i] =
                if f.code >> i & 1 == 1 { 1.0 } else { -1.0 };
        }
    }
    let signs = Tensor::from_vec(signs, (rows, width), codes.device())?;
    // Where a sign is forced (`|sign| = 1`), it flips the code's bit, `code ≥ 0` as `pack_signs`
    // reads it, when that bit is not the forced one, `(sign + 1) / 2`.
    let bit = codes.ge(0.0)?.to_dtype(DType::F32)?;
    let flips = bit
        .sub(&signs.affine(0.5, 0.5)?)?
        .abs()?
        .mul(&signs.abs()?)?
        .gt(0.0)?;
    Ok(flips.where_cond(&signs, codes)?)
}

/// Which earlier positions the read may choose while a reply decodes (claude-215), or, for a
/// model trained with the cut, while its replies train and score (claude-216). Every other
/// reading sets none, so the read chooses as trained.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadScope {
    /// Every earlier position, as trained.
    Full,
    /// The copy heads choose only positions before the cut, the prompt's; the type heads, every
    /// earlier position.
    Copy,
    /// Every head chooses only positions before the cut.
    All,
    /// The ask heads (the type heads under [`HammingReads::with_ask`]) choose only positions
    /// before the cut; the copy heads, every earlier position (claude-330).
    Ask,
}

impl ReadScope {
    /// The name a samples file carries for it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Copy => "copy",
            Self::All => "all",
            Self::Ask => "ask",
        }
    }
}

/// Limit the read's choice for the forward passes that follow on this thread: under the scope,
/// the heads it limits choose only window positions `s < cut`. `None`, the default, limits none.
/// `decode_reply` sets it around one window's forward, with `cut` where the prompt ends there.
pub fn set_read_cut(cut: Option<(ReadScope, usize)>) {
    CUT.with(|c| c.set(cut));
}

/// The cut [`set_read_cut`] last set on this thread.
pub fn read_cut() -> Option<(ReadScope, usize)> {
    CUT.with(|c| c.get())
}

/// Limit the read's choice row by row for the forward passes that follow on this thread
/// (claude-216): under the scope, the heads it limits choose for row `r` (of `batch · time`)
/// only window positions `s < limits[r]`. `None`, the default, limits none. Training and the
/// teacher-forced scoring of a model trained with the cut set it from the rows' reply runs
/// (`reply_row_limits`, `in_space_long.rs`); it is never set together with [`set_read_cut`].
pub fn set_row_cut(cut: Option<(ReadScope, Vec<usize>)>) {
    ROW_CUT.with(|c| *c.borrow_mut() = cut);
}

/// Whether a row cut is set on this thread ([`set_row_cut`]).
pub fn row_cut_set() -> bool {
    ROW_CUT.with(|c| c.borrow().is_some())
}

/// The read's instrument set on this thread that the decode cache's step cannot carry, if one is
/// (claude-279): the decode cut and the row cut name the whole window's positions and rows, and
/// the decision trace and forced codes name its rows. With the tie rule off, a CUDA sort leaves
/// the order of equal keys open, so a step's read could break a tie the whole window's does not
/// (claude-281).
pub(crate) fn step_instrument() -> Option<&'static str> {
    if !hamming_tie_rule() {
        Some("Hamming read's open tie order (tie rule off)")
    } else if CUT.with(|c| c.get()).is_some() {
        Some("Hamming read's decode cut")
    } else if row_cut_set() {
        Some("Hamming read's row cut")
    } else if TRACE.with(|t| t.borrow().is_some()) {
        Some("Hamming read's decision trace")
    } else if CODE_FORCE.with(|f| !f.borrow().is_empty()) {
        Some("Hamming read's forced codes")
    } else if class_ablation() != [[false; 2]; 2] {
        Some("Hamming read's class ablation")
    } else if hold_off() {
        Some("Hamming read's hold cut")
    } else if carry_off() {
        Some("Hamming read's carry cut")
    } else if CAPTURE.with(|c| c.borrow().is_some()) {
        Some("Hamming read's row capture")
    } else {
        None
    }
}

/// Whether the scope limits head `head`, the first `copy_heads` heads being copy heads.
fn head_limited(scope: ReadScope, head: usize, copy_heads: usize) -> bool {
    match scope {
        ReadScope::Full => false,
        ReadScope::Copy => head < copy_heads,
        ReadScope::All => true,
        ReadScope::Ask => head >= copy_heads,
    }
}

/// Head `head`'s candidate end for each of `rows` rows of windows of `time` positions, under
/// the decode cut `cut` or the row cut `row_cut` (at most one is set): the cut for a head its
/// scope limits, else `time` (every earlier position).
fn row_limits(
    cut: Option<(ReadScope, usize)>,
    row_cut: Option<&(ReadScope, Vec<usize>)>,
    head: usize,
    copy_heads: usize,
    time: usize,
    rows: usize,
) -> Vec<usize> {
    match (cut, row_cut) {
        (Some(_), _) => vec![head_limit(cut, head, copy_heads, time); rows],
        (None, Some((scope, limits))) if head_limited(*scope, head, copy_heads) => {
            limits.iter().map(|&at| at.min(time)).collect()
        }
        _ => vec![time; rows],
    }
}

/// The end of head `head`'s candidates in a window of `time` positions, the first `copy_heads`
/// of them copy heads, under `cut`: the cut for a head the scope limits, else `time`.
fn head_limit(cut: Option<(ReadScope, usize)>, head: usize, copy_heads: usize, time: usize) -> usize {
    match cut {
        Some((ReadScope::All, at)) => at.min(time),
        Some((ReadScope::Copy, at)) if head < copy_heads => at.min(time),
        Some((ReadScope::Ask, at)) if head >= copy_heads => at.min(time),
        _ => time,
    }
}

/// Whether position `s` is a candidate for position `t`: from `first`, earlier than `t`, and
/// before `limit` ([`head_limit`]). The rule [`candidate_mask`] builds on the device, kept as its
/// reference (claude-272).
#[cfg(test)]
fn is_candidate(s: usize, t: usize, first: usize, limit: usize) -> bool {
    s >= first && s < t && s < limit
}

/// Make `reads` the Hamming reads `StackModel::hidden` applies on this thread, or none.
pub fn set_hamming_reads(reads: Option<HammingReads>) {
    ACTIVE.with(|cell| *cell.borrow_mut() = reads);
    EPOCH.with(|e| e.set(e.get() + 1));
}

/// How many times [`set_hamming_reads`] has run on this thread: the identity of the reads set,
/// which a count of read sites alone does not give (claude-281).
pub(crate) fn hamming_reads_epoch() -> u64 {
    EPOCH.with(|e| e.get())
}

/// The Hamming reads `StackModel::hidden` applies on this thread, if set.
pub fn hamming_reads() -> Option<HammingReads> {
    ACTIVE.with(|cell| cell.borrow().clone())
}

/// claude-235: break the read's ties as the integer session does, by ascending bit distance and
/// then ascending position, for the forward passes that follow on this thread
/// ([`choose_sorted`]). On is the default since claude-239, so wherever the float read
/// runs it keeps the sets the integer session keeps, unless turned off. Off, the sort's own tie
/// order stands: on CUDA a bitonic sort's, which leaves the order of equal keys open.
pub fn set_hamming_tie_rule(on: bool) {
    TIE.with(|t| t.set(on));
}

/// Whether [`set_hamming_tie_rule`] is on for this thread.
pub fn hamming_tie_rule() -> bool {
    TIE.with(|t| t.get())
}

/// claude-235: record what every Hamming read chooses on this thread from now on (`true`), or
/// stop and drop the record (`false`, the default). Each window a forward reads becomes one
/// [`HammingTraceWindow`]: the stream read opens it and the readout read joins it. Recording
/// copies codes and picks to the host and changes nothing the read computes.
pub fn set_hamming_trace(on: bool) {
    TRACE.with(|t| *t.borrow_mut() = on.then(Vec::new));
}

/// The windows recorded since recording started or was last taken, leaving it on with an empty
/// record; `None` when it is off.
pub fn take_hamming_trace() -> Option<Vec<HammingTraceWindow>> {
    TRACE.with(|t| t.borrow_mut().as_mut().map(std::mem::take))
}

/// claude-251: also record each code's coordinates, as `f32::to_bits`, while the trace is on
/// (`true`), for the decision trace's float side; or record the codes alone (`false`, the
/// default), so `fbs-snapshot --hamming-trace` writes what it wrote before.
pub fn set_hamming_trace_coords(on: bool) {
    COORDS.with(|c| c.set(on));
}

/// One head's codes and choice over a forward's rows, copied to the host for the trace; with
/// coordinates on, the query's and key's coordinates before the sign, beside their ±1 codes.
struct HeadChoice {
    query: Vec<Vec<f32>>,
    key: Vec<Vec<f32>>,
    picks: Vec<Vec<u32>>,
    real: Vec<Vec<u8>>,
    coords: Option<Coords>,
}

/// A head's query and key coordinates before the sign, row by row, and a window's view of them.
type Coords = (Vec<Vec<f32>>, Vec<Vec<f32>>);
type CoordsRef<'a> = (&'a [Vec<f32>], &'a [Vec<f32>]);

/// One head's trace over one window's rows: each row's query and key code packed as
/// [`pack_signs`] packs it, one word, so a head of at most 64 coordinates; and each row's real
/// picks, in the order chosen. With `coords`, the query's and key's coordinates the sign read,
/// each refused unless it signs to its row's word; except at the rows `forced` names (claude-253:
/// the query's rows, then the key's), where the word is the forced code and the coordinates stay
/// as computed.
fn trace_head(
    query: &[Vec<f32>],
    key: &[Vec<f32>],
    picks: &[Vec<u32>],
    real: &[Vec<u8>],
    coords: Option<CoordsRef<'_>>,
    forced: (&[usize], &[usize]),
) -> Result<HammingTraceHead> {
    let rows = query.len();
    if key.len() != rows || picks.len() != rows || real.len() != rows {
        return Err(invalid("the Hamming trace needs a code and a choice for every row"));
    }
    let word = |row: &Vec<f32>| -> Result<u64> {
        match pack_signs(row).as_slice() {
            [word] => Ok(*word),
            _ => Err(invalid("the Hamming trace records heads of 1 to 64 coordinates")),
        }
    };
    let (query_words, key_words) = (
        query.iter().map(word).collect::<Result<Vec<_>>>()?,
        key.iter().map(word).collect::<Result<Vec<_>>>()?,
    );
    let kept = picks
        .iter()
        .zip(real)
        .map(|(picks, real)| {
            if picks.len() != real.len() {
                return Err(invalid("the Hamming trace needs a real flag for every pick"));
            }
            Ok(picks.iter().zip(real).filter(|(_, &ok)| ok == 1).map(|(&s, _)| s).collect())
        })
        .collect::<Result<Vec<Vec<u32>>>>()?;
    // Each coordinate row as bits, refused unless it signs to the word its code packed, or its
    // code was forced.
    let bits = |rows: &[Vec<f32>], words: &[u64], forced: &[usize]| -> Result<Vec<Vec<u32>>> {
        if rows.len() != words.len() {
            return Err(invalid("the Hamming trace needs coordinates for every row"));
        }
        rows.iter()
            .zip(words)
            .enumerate()
            .map(|(t, (row, &w))| {
                if word(row)? != w && !forced.contains(&t) {
                    return Err(invalid("a code's coordinates do not sign to its word"));
                }
                Ok(row.iter().map(|v| v.to_bits()).collect())
            })
            .collect()
    };
    let (query_f32, key_f32) = match coords {
        Some((q, k)) => (
            bits(q, &query_words, forced.0)?,
            bits(k, &key_words, forced.1)?,
        ),
        None => (Vec::new(), Vec::new()),
    };
    Ok(HammingTraceHead { query: query_words, key: key_words, kept, query_f32, key_f32 })
}

/// Add one forward's reads, one per window of `time` positions of `ids`, to the record: a
/// stream read opens a window per batch row, and the readout read joins the windows the stream
/// read opened on the same ids. A readout read with no such windows is refused.
fn push_trace_reads(
    windows: &mut Vec<HammingTraceWindow>,
    ids: &[u32],
    time: usize,
    reads: Vec<HammingTraceRead>,
    readout: bool,
) -> Result<()> {
    if time == 0 || ids.len() != reads.len() * time {
        return Err(invalid("the Hamming trace needs one read per whole window"));
    }
    if !readout {
        windows.extend(reads.into_iter().zip(ids.chunks(time)).map(|(read, ids)| {
            HammingTraceWindow {
                ids: ids.to_vec(),
                reads: vec![read],
            }
        }));
        return Ok(());
    }
    let open = windows
        .len()
        .checked_sub(reads.len())
        .ok_or_else(|| invalid("the Hamming trace's readout read has no stream read before it"))?;
    let tail = &mut windows[open..];
    if tail
        .iter()
        .zip(ids.chunks(time))
        .any(|(window, ids)| window.ids != ids || window.reads.len() != 1)
    {
        return Err(invalid(
            "the Hamming trace's readout read does not follow a stream read on its windows",
        ));
    }
    for (window, read) in tail.iter_mut().zip(reads) {
        window.reads.push(read);
    }
    Ok(())
}

/// Each coordinate's sign bit, packed 64 to a word in coordinate order: the serving form of a
/// head's code, compared by `(a ^ b).count_ones()`. A coordinate at or above 0 is a 1.
pub fn pack_signs(coordinates: &[f32]) -> Vec<u64> {
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

/// One read: learned per-lane rotations for the query, key and value, a margin per head (bits),
/// and the per-coordinate weights of what it adds: row 0 the copy heads', row 1 the type heads'.
#[derive(Clone)]
pub struct HammingRead {
    heads: usize,
    pub(crate) query_rotation: Var,
    pub(crate) key_rotation: Var,
    pub(crate) value_rotation: Var,
    pub(crate) margin: Var,
    pub(crate) weight: Var,
    /// The type heads' position signal, when on ([`HammingReads::with_type_spin`]): the per-lane
    /// rotation that turns each token's point into its icosian step (claude-151).
    pub(crate) step_rotation: Option<Var>,
    /// How many tokens before a position each copy head's key names: 1, or 2 for the ordered
    /// two-token key ([`HammingReads::with_copy_key`], claude-261), one entry per copy head
    /// ([`HammingReads::with_copy_keys`], claude-268).
    copy_keys: Vec<usize>,
    /// The ask heads' question table, when on ([`HammingReads::with_ask`], claude-330): one
    /// learned point per token, `(vocab, width)`, starting at zero. The type heads then ask with
    /// the current token's point plus its row, through the key's rotation, and bring back the
    /// kept position's own token point.
    pub(crate) ask: Option<Var>,
    /// The held ask write, when on ([`HammingReads::with_hold`], claude-333): at the stream read
    /// only, each row's ask write is carried to the next `rows` rows of its window, scaled by one
    /// learned scalar per token of the row it came from, starting at zero.
    pub(crate) hold: Option<Hold>,
    /// The question carry, when on ([`HammingReads::with_carry`], claude-335): at the readout
    /// read only, each ask head's asked point adds the asked points of the `rows` rows before it
    /// in its window, each scaled by one learned scalar per token of that row, starting at zero.
    pub(crate) carry: Option<Hold>,
}

/// The held ask write (claude-333): how many rows a write is held for, and the per-token gain
/// `g`, `(vocab,)`, starting at zero.
#[derive(Clone)]
pub struct Hold {
    pub(crate) rows: usize,
    pub(crate) gain: Var,
}

/// claude-333: every token whose write is held has a gain, so a lookup never reads past `g`.
fn check_hold_ids(gain: &Var, ids: &[u32]) -> Result<()> {
    let rows = gain.as_tensor().dim(0)?;
    match ids.iter().find(|&&id| id as usize >= rows) {
        Some(id) => Err(invalid(format!("token {id} has no gain in the {rows}-token hold"))),
        None => Ok(()),
    }
}

/// The held term (claude-333): `Σ_{s=1..rows} g[x_{t−s}] · u_{t−s}` at every row `t` of each
/// window, over rows of the same window only, for the writes `u` `(batch · time, width)` of the
/// windows `ids`. A row before the window's start adds nothing, so row `t` sums `min(t, rows)`
/// writes and never its own.
pub(crate) fn held_term(
    gain: &Var,
    writes: &Tensor,
    ids: &[u32],
    batch: usize,
    time: usize,
    rows: usize,
) -> Result<Tensor> {
    check_hold_ids(gain, ids)?;
    let width = writes.dim(1)?;
    let device = writes.device();
    let index = Tensor::from_vec(ids.to_vec(), ids.len(), device)?;
    let scaled = writes
        .broadcast_mul(&gain.as_tensor().index_select(&index, 0)?.unsqueeze(1)?)?
        .reshape((batch, time, width))?;
    let mut held = Tensor::zeros((batch, time, width), DType::F32, device)?;
    for s in 1..=rows.min(time.saturating_sub(1)) {
        let shifted = Tensor::cat(
            &[
                &Tensor::zeros((batch, s, width), DType::F32, device)?,
                &scaled.narrow(1, 0, time - s)?,
            ],
            1,
        )?;
        held = held.add(&shifted)?;
    }
    Ok(held.reshape((batch * time, width))?)
}

/// Refuse a copy key the read cannot carry: one other than 1 or 2, or 2 on heads of `width /
/// heads` coordinates whose halves do not fall on whole 4-lane blocks, as the rotations act.
fn check_copy_key(copy_key: usize, heads: usize, width: usize) -> Result<()> {
    if copy_key != 1 && copy_key != 2 {
        return Err(invalid(format!(
            "a copy key names 1 or 2 tokens, not {copy_key}"
        )));
    }
    // claude-261 review: one head is a type head only, so copy key 2 would key nothing.
    if copy_key == 2 && (heads < 2 || !(width / heads).is_multiple_of(8)) {
        return Err(invalid(
            "copy key 2 needs a copy head (2 heads or more) of whole 8-coordinate blocks, so each half is whole lanes",
        ));
    }
    Ok(())
}

/// Refuse per-head copy keys the read cannot carry (claude-268): a list that is not one key per
/// copy head, or a key [`check_copy_key`] refuses.
fn check_copy_keys(copy_keys: &[usize], heads: usize, width: usize) -> Result<()> {
    if copy_keys.len() != heads / 2 {
        return Err(invalid(format!(
            "copy keys name one key per copy head: {} heads have {}, not {}",
            heads,
            heads / 2,
            copy_keys.len()
        )));
    }
    copy_keys.iter().try_for_each(|&k| check_copy_key(k, heads, width))
}

/// claude-330: every asking token has a row in the ask table, so a lookup never reads past it.
fn check_ask_ids(ask: &Var, ids: &[u32]) -> Result<()> {
    let rows = ask.as_tensor().dim(0)?;
    match ids.iter().find(|&&id| id as usize >= rows) {
        Some(id) => Err(invalid(format!("token {id} has no row in the {rows}-row ask table"))),
        None => Ok(()),
    }
}

/// [`pull_pairs`]'s answer: the `(row, s)` pairs and, per pull with any, the range of its pairs.
pub(crate) type PullPairs = (Vec<(u32, u32)>, Vec<std::ops::Range<usize>>);

/// The found-copy pull's candidates (claude-330): for each `(row, target)`, the `(row, s)` pairs
/// of the earlier positions `s` of the row's window holding `target`, `1 ≤ s < t` in the window,
/// in ascending `s`; and, for each pull with any, the range of its pairs. A pull with none has no
/// range, so it adds nothing.
pub(crate) fn pull_pairs(
    ids: &[u32],
    time: usize,
    pulls: &[(usize, u32)],
) -> Result<PullPairs> {
    let mut pairs = Vec::new();
    let mut spans = Vec::new();
    for &(row, target) in pulls {
        if row >= ids.len() {
            return Err(invalid(format!("a found-copy pull at row {row} of {}", ids.len())));
        }
        let (start, t) = (row - row % time, row % time);
        let from = pairs.len();
        pairs.extend(
            (1..t)
                .filter(|&s| ids[start + s] == target)
                .map(|s| (row as u32, (start + s) as u32)),
        );
        if pairs.len() > from {
            spans.push(from..pairs.len());
        }
    }
    Ok((pairs, spans))
}

/// The token two before each of `ids`' rows, in windows of `time`: `x_{s−2}`, or, where a window
/// has fewer tokens before `s`, the one before it, `x_{s−1}`, or at the window's first position
/// its own (never a candidate). The integer session's `CopyHeads` keys by the same rule.
fn two_back(ids: &[u32], time: usize) -> Vec<u32> {
    (0..ids.len())
        .map(|r| match r % time {
            0 => ids[r],
            1 => ids[r - 1],
            _ => ids[r - 2],
        })
        .collect()
}

/// `first`, `(rows, width)`, with the second half of each head `h < pair.len()` of `head_width`
/// columns where `pair[h]` taken from `second`: copy key 2's codes, one token in each half; a head
/// on copy key 1 stays `first`'s whole (claude-268).
fn pair_halves(first: &Tensor, second: &Tensor, pair: &[bool], head_width: usize) -> Result<Tensor> {
    let (half, width, heads) = (head_width / 2, first.dim(1)?, pair.len());
    let mut parts = Vec::with_capacity(2 * heads + 1);
    for (h, &paired) in pair.iter().enumerate() {
        if paired {
            parts.push(first.narrow(1, h * head_width, half)?);
            parts.push(second.narrow(1, h * head_width + half, half)?);
        } else {
            parts.push(first.narrow(1, h * head_width, head_width)?);
        }
    }
    if heads * head_width < width {
        parts.push(first.narrow(1, heads * head_width, width - heads * head_width)?);
    }
    Ok(Tensor::cat(&parts, 1)?)
}

/// The bytes a byte-level BPE piece stands for: each character is one byte, the printable bytes
/// as themselves and the other 68, in order, as `U+0100` upward. `None` if a character is neither.
fn byte_level_bytes(piece: &str) -> Option<Vec<u8>> {
    let printable = |b: u32| matches!(b, 33..=126 | 161..=172 | 174..=255);
    piece
        .chars()
        .map(|c| match c as u32 {
            b if printable(b) => Some(b as u8),
            n @ 256..=323 => (0u8..=255)
                .filter(|&b| !printable(u32::from(b)))
                .nth((n - 256) as usize),
            _ => None,
        })
        .collect()
}

/// claude-267: which ids of `tokenizer.json` end a slot clause. An id is a boundary if its text is
/// empty (an unassigned id), contains `.`, `!`, `?`, `\n` or `\r`, or it is in `added_tokens`.
/// Other control characters, tab included, are not. One flag per id up to the largest id named;
/// a piece that is not byte-level is refused.
pub fn slot_boundary_flags(tokenizer_json: &[u8]) -> Result<Vec<bool>> {
    let json: serde_json::Value = serde_json::from_slice(tokenizer_json)
        .map_err(|e| invalid(format!("tokenizer.json is not JSON: {e}")))?;
    let vocab = json["model"]["vocab"]
        .as_object()
        .ok_or_else(|| invalid("tokenizer.json has no model.vocab"))?;
    let added = json["added_tokens"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let id = |v: &serde_json::Value| {
        v.as_u64()
            .map(|i| i as usize)
            .ok_or_else(|| invalid("tokenizer.json names an id that is not a whole number"))
    };
    let mut pieces = Vec::with_capacity(vocab.len());
    for (piece, v) in vocab {
        let bytes = byte_level_bytes(piece)
            .ok_or_else(|| invalid(format!("tokenizer.json piece {piece:?} is not byte-level")))?;
        pieces.push((id(v)?, bytes));
    }
    let added_ids = added.iter().map(|t| id(&t["id"])).collect::<Result<Vec<_>>>()?;
    let len = pieces
        .iter()
        .map(|(i, _)| i + 1)
        .chain(added_ids.iter().map(|i| i + 1))
        .max()
        .unwrap_or(0);
    let mut flags = vec![true; len];
    for (i, bytes) in pieces {
        flags[i] = bytes.is_empty() || bytes.iter().any(|b| b".!?\n\r".contains(b));
    }
    for i in added_ids {
        flags[i] = true;
    }
    Ok(flags)
}

/// claude-267: each id's slot rank. Boundary ids get 0; every other id a rank from 1 upward by
/// its count in `train`, rarest first, the lower id first on a tie. Refuses a `train` token
/// outside `vocab` and a rank past `u16`.
pub fn slot_rank_table(train: &[u16], vocab: usize, boundary: &[bool]) -> Result<Vec<u16>> {
    if boundary.len() != vocab {
        return Err(invalid(format!(
            "slot ranks: {} boundary flags for a vocabulary of {vocab}",
            boundary.len()
        )));
    }
    let mut counts = vec![0u64; vocab];
    for &t in train {
        let t = usize::from(t);
        if t >= vocab {
            return Err(invalid(format!(
                "slot ranks: TRAIN token {t} is outside the vocabulary of {vocab}"
            )));
        }
        counts[t] += 1;
    }
    let mut order: Vec<usize> = (0..vocab).filter(|&t| !boundary[t]).collect();
    if order.len() > usize::from(u16::MAX) {
        return Err(invalid(format!("slot ranks: {} ranked ids overflow u16", order.len())));
    }
    order.sort_by_key(|&t| (counts[t], t));
    let mut table = vec![0u16; vocab];
    for (rank, &t) in order.iter().enumerate() {
        table[t] = (rank + 1) as u16;
    }
    Ok(table)
}

/// claude-267: the slot atom `A(i)` at each position: the token of lowest nonzero rank among the
/// clause's tokens up to and including `i`, where a clause starts after the last boundary token
/// (rank 0, or an id past `table`). At a boundary token, `A(i)` is that token.
pub fn slot_atoms(tokens: &[u32], table: &[u16]) -> Vec<u32> {
    let mut best: Option<(u16, u32)> = None;
    tokens
        .iter()
        .map(|&t| {
            let rank = table.get(t as usize).copied().unwrap_or(0);
            if rank == 0 {
                best = None;
                return t;
            }
            match best {
                Some((r, atom)) if r <= rank => atom,
                _ => {
                    best = Some((rank, t));
                    t
                }
            }
        })
        .collect()
}

/// claude-267, a copy-key oracle diagnostic: the clause's two distinct tokens of lowest nonzero
/// rank up to and including `i`, as an unordered (smaller id first) pair. A clause's first token
/// pairs with itself, and so does a boundary token, where the clause resets as in [`slot_atoms`].
pub fn slot_atom_pairs(tokens: &[u32], table: &[u16]) -> Vec<(u32, u32)> {
    let mut best: Vec<(u16, u32)> = Vec::with_capacity(3);
    tokens
        .iter()
        .map(|&t| {
            let rank = table.get(t as usize).copied().unwrap_or(0);
            if rank == 0 {
                best.clear();
                return (t, t);
            }
            if best.iter().all(|&(_, b)| b != t) {
                best.push((rank, t));
                best.sort_unstable();
                best.truncate(2);
            }
            let a = best[0].1;
            let b = best.get(1).map_or(a, |&(_, b)| b);
            (a.min(b), a.max(b))
        })
        .collect()
}

/// claude-267, a copy-key oracle diagnostic: the token of lowest nonzero rank among the `width`
/// tokens ending at `i`, with no clause reset; `i`'s own token when all of them are boundaries.
pub fn window_atoms(tokens: &[u32], table: &[u16], width: usize) -> Vec<u32> {
    let rank = |t: u32| table.get(t as usize).copied().unwrap_or(0);
    (0..tokens.len())
        .map(|i| {
            tokens[(i + 1).saturating_sub(width.max(1))..=i]
                .iter()
                .copied()
                .filter(|&t| rank(t) != 0)
                .min_by_key(|&t| rank(t))
                .unwrap_or(tokens[i])
        })
        .collect()
}

/// The reads at every depth in [`HAMMING_DEPTHS`], in that order.
#[derive(Clone)]
pub struct HammingReads {
    pub reads: Vec<HammingRead>,
}

impl HammingReads {
    /// Reads with `heads` heads on points of `width` coordinates: identity rotations, margins at
    /// half a head's bits, and zero weights, so they start as no read at all.
    pub fn new(heads: usize, width: usize, device: &Device) -> Result<Self> {
        let reads = HAMMING_DEPTHS
            .iter()
            .map(|_| HammingRead::new(heads, width, device))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { reads })
    }

    /// The same reads with the type heads' position signal on (claude-151): SpiralCore's
    /// path-dependent transport in its quaternion group. Each token steps each type-head lane by
    /// the unit icosian nearest its table point (through a learned per-lane rotation, starting at
    /// the identity); a position's frame is the product of the steps so far in its window; a type
    /// head turns its query at `t` and its key at `s` by their frames' inverses, so it compares
    /// them through `F_t ⊗ F_s⁻¹`, the steps of the tokens between them. Copy heads are untouched.
    pub fn with_type_spin(mut self) -> Result<Self> {
        if self.has_ask() {
            return Err(invalid(
                "the type spin and the ask heads both change the type heads; one is on at a time",
            ));
        }
        for read in &mut self.reads {
            let lanes = read.weight.dim(1)? / 4;
            let identity: Vec<f32> = (0..lanes).flat_map(|_| [1.0f32, 0.0, 0.0, 0.0]).collect();
            let device = read.weight.device();
            read.step_rotation = Some(Var::from_tensor(&Tensor::from_vec(
                identity,
                (lanes, 4),
                device,
            )?)?);
        }
        Ok(self)
    }

    /// The same reads with the type heads turned into ask heads (claude-330). Each depth gets its
    /// own question table, one point per token of `vocab`, starting at zero. An ask head asks with
    /// the current token's point plus its row, through the key's rotation, against the copy key,
    /// and brings back the kept position's own token point. At zero rows it asks exactly as a copy
    /// head does. Its margin starts at a copy head's, a quarter of a head's bits. Copy heads are
    /// untouched. The type spin also changes the type heads, so the two are refused together.
    pub fn with_ask(mut self, vocab: usize) -> Result<Self> {
        if self.type_spin()? {
            return Err(invalid(
                "the type spin and the ask heads both change the type heads; one is on at a time",
            ));
        }
        for read in &mut self.reads {
            let width = read.weight.dim(1)?;
            let head_width = width / read.heads;
            let copy_heads = read.copy_heads();
            let device = read.weight.device().clone();
            let margins: Vec<f32> = read
                .margin
                .as_tensor()
                .to_vec1::<f32>()?
                .into_iter()
                .enumerate()
                .map(|(h, m)| if h < copy_heads { m } else { (head_width / 4) as f32 })
                .collect();
            read.margin = Var::from_tensor(&Tensor::from_vec(margins, read.heads, &device)?)?;
            read.ask = Some(Var::zeros((vocab, width), DType::F32, &device)?);
        }
        Ok(self)
    }

    /// Whether every depth has its ask heads' question table.
    pub fn has_ask(&self) -> bool {
        !self.reads.is_empty() && self.reads.iter().all(|r| r.ask.is_some())
    }

    /// The same reads with the stream read's ask write held for `rows` rows (claude-333,
    /// `--hamming-hold`): a gain per token of `vocab`, starting at zero, so at the start the
    /// forward is the forward without it. It needs the ask heads, whose write it holds, and acts
    /// at the stream read only; the readout read is unchanged.
    pub fn with_hold(mut self, rows: usize, vocab: usize) -> Result<Self> {
        if !self.has_ask() {
            return Err(invalid("the hold carries the ask heads' write; turn them on with --hamming-ask 1"));
        }
        if rows == 0 {
            return Err(invalid("a hold carries a write for 1 row or more"));
        }
        let read = &mut self.reads[0];
        let device = read.weight.device().clone();
        read.hold = Some(Hold { rows, gain: Var::zeros(vocab, DType::F32, &device)? });
        Ok(self)
    }

    /// How many rows the stream read holds its ask write for, or 0 with no hold.
    pub fn hold_rows(&self) -> usize {
        self.reads.first().and_then(|r| r.hold.as_ref()).map_or(0, |h| h.rows)
    }

    /// claude-334: the stream read's hold gain `g`, one per token, on the host; `None` with no
    /// hold.
    pub fn hold_gain(&self) -> Result<Option<Vec<f32>>> {
        match self.reads.first().and_then(|r| r.hold.as_ref()) {
            Some(hold) => Ok(Some(hold.gain.as_tensor().to_vec1::<f32>()?)),
            None => Ok(None),
        }
    }

    /// The same reads with the readout read's question carried for `rows` rows (claude-335,
    /// `--hamming-carry-question`): a gain `c` per token of `vocab`, starting at zero, so at the
    /// start the forward is the forward without it. At the readout read only, each ask head's
    /// asked point at row `t` adds `Σ_{s=1..rows} c[x_{t−s}] · (point(x_{t−s}) + a[x_{t−s}])`
    /// over rows of the same window; the found-copy pull asks the same way. It needs the ask
    /// heads, whose asked points it carries; the stream read is unchanged.
    pub fn with_carry(mut self, rows: usize, vocab: usize) -> Result<Self> {
        if !self.has_ask() {
            return Err(invalid(
                "the question carry carries the ask heads' question; turn them on with --hamming-ask 1",
            ));
        }
        if rows == 0 {
            return Err(invalid("a question carry reaches 1 row back or more"));
        }
        let read = &mut self.reads[1];
        let device = read.weight.device().clone();
        read.carry = Some(Hold { rows, gain: Var::zeros(vocab, DType::F32, &device)? });
        Ok(self)
    }

    /// How many rows back the readout read carries its question from, or 0 with no carry.
    pub fn carry_rows(&self) -> usize {
        self.reads.get(1).and_then(|r| r.carry.as_ref()).map_or(0, |c| c.rows)
    }

    /// claude-335: the readout read's carry gain `c`, one per token, on the host; `None` with no
    /// carry.
    pub fn carry_gain(&self) -> Result<Option<Vec<f32>>> {
        match self.reads.get(1).and_then(|r| r.carry.as_ref()) {
            Some(carry) => Ok(Some(carry.gain.as_tensor().to_vec1::<f32>()?)),
            None => Ok(None),
        }
    }

    /// claude-334: each depth's type weight row `w_y` (the weight's row 1), in
    /// [`HAMMING_DEPTHS`] order, on the host.
    pub fn type_weights(&self) -> Result<Vec<Vec<f32>>> {
        self.reads
            .iter()
            .map(|r| Ok(r.weight.as_tensor().get(1)?.to_vec1::<f32>()?))
            .collect()
    }

    /// The found-copy pull (claude-330, `--hamming-attract`): for each `(row, target)` in `pulls`,
    /// the earlier positions `s` of the row's window holding `target` (`1 ≤ s < t` in the window)
    /// are the copies an ask head should find. Each ask head's distance to its nearest one,
    /// `bits(t, s*) / head_width`, `s*` chosen on detached values with ties to the smallest `s`,
    /// is averaged over the rows with a copy, the ask heads and the depths. The keys and the
    /// rotation are detached, so only the question rows learn from it. `None` when no row has a
    /// copy; the caller scales the mean by λ.
    pub fn ask_pull(
        &self,
        ids: &[u32],
        time: usize,
        table: &Tensor,
        pulls: &[(usize, u32)],
    ) -> Result<Option<Tensor>> {
        let Some(parts) = self.pull_parts(ids, time, table, pulls)? else {
            return Ok(None);
        };
        let mut total: Option<Tensor> = None;
        let mut count = 0usize;
        for (sum, n) in parts {
            total = Some(match total {
                None => sum,
                Some(t) => t.add(&sum)?,
            });
            count += n;
        }
        Ok(total.map(|t| t.affine(1.0 / count as f64, 0.0)).transpose()?)
    }

    /// The sheet's pull-distance reading (claude-330): [`Self::ask_pull`]'s mean distance at each
    /// depth on its own, as plain numbers, over the same `(row, target)`s. `None` when no row has
    /// a copy.
    pub fn ask_pull_depths(
        &self,
        ids: &[u32],
        time: usize,
        table: &Tensor,
        pulls: &[(usize, u32)],
    ) -> Result<Option<Vec<f64>>> {
        let Some(parts) = self.pull_parts(ids, time, table, pulls)? else {
            return Ok(None);
        };
        parts
            .into_iter()
            .map(|(sum, n)| Ok(f64::from(sum.to_scalar::<f32>()?) / n as f64))
            .collect::<Result<Vec<_>>>()
            .map(Some)
    }

    /// Each depth's [`HammingRead::pull_bits`]: its sum and its term count, or `None` when no row
    /// has a copy.
    fn pull_parts(
        &self,
        ids: &[u32],
        time: usize,
        table: &Tensor,
        pulls: &[(usize, u32)],
    ) -> Result<Option<Vec<(Tensor, usize)>>> {
        if !self.has_ask() {
            return Err(invalid("the found-copy pull needs the ask heads"));
        }
        if time == 0 || !ids.len().is_multiple_of(time) {
            return Err(invalid("the found-copy pull needs whole windows"));
        }
        let (pairs, spans) = pull_pairs(ids, time, pulls)?;
        if spans.is_empty() {
            return Ok(None);
        }
        self.reads
            .iter()
            .map(|read| read.pull_bits(ids, time, table, &pairs, &spans))
            .collect::<Result<Vec<_>>>()
            .map(Some)
    }

    /// The same reads with their copy heads keyed on `copy_key` tokens before a position: 1, the
    /// default, or 2, the ordered two-token key (claude-261). Type heads are untouched.
    pub fn with_copy_key(mut self, copy_key: usize) -> Result<Self> {
        for read in &mut self.reads {
            check_copy_key(copy_key, read.heads, read.weight.dim(1)?)?;
            read.copy_keys = vec![copy_key; read.copy_heads()];
        }
        Ok(self)
    }

    /// The same reads with copy head `h` keyed on `copy_keys[h]` tokens before a position, each
    /// 1 or 2 (claude-268), one entry per copy head. Type heads are untouched.
    pub fn with_copy_keys(mut self, copy_keys: &[usize]) -> Result<Self> {
        for read in &mut self.reads {
            check_copy_keys(copy_keys, read.heads, read.weight.dim(1)?)?;
            read.copy_keys = copy_keys.to_vec();
        }
        Ok(self)
    }

    /// Each copy head's key (1 or 2), copy head 0 first.
    pub fn copy_keys(&self) -> Vec<usize> {
        self.reads.first().map_or_else(Vec::new, |r| r.copy_keys.clone())
    }

    /// The copy heads' keys when they differ (claude-268), else `None`.
    pub fn mixed_copy_keys(&self) -> Option<Vec<usize>> {
        let keys = self.copy_keys();
        keys.iter().any(|&k| k != keys[0]).then_some(keys)
    }

    /// How many tokens before a position the copy heads' key names (1 or 2): the key every copy
    /// head shares, or 1 when they differ, where [`Self::mixed_copy_keys`] names each (the
    /// integer record's convention).
    pub fn copy_key(&self) -> usize {
        match (self.mixed_copy_keys(), self.copy_keys().first()) {
            (None, Some(&k)) => k,
            _ => 1,
        }
    }

    /// The read's head count when it has one; a saved checkpoint always does.
    pub fn heads(&self) -> Option<usize> {
        self.reads.first().map(HammingRead::heads)
    }

    /// The read's coordinate width when it has one, refusing reads that disagree.
    pub fn width(&self) -> Result<Option<usize>> {
        let Some(first) = self.reads.first() else {
            return Ok(None);
        };
        let width = first.weight.dim(1)?;
        if self.reads.iter().any(|read| read.weight.dim(1).ok() != Some(width)) {
            return Err(invalid("the Hamming reads disagree on their coordinate width"));
        }
        Ok(Some(width))
    }

    /// Whether every read carries the type heads' SpiralCore position signal.
    /// A sidecar with the signal on only some depths is internally inconsistent.
    pub fn type_spin(&self) -> Result<bool> {
        let present = self
            .reads
            .iter()
            .filter(|read| read.step_rotation.is_some())
            .count();
        match (present, self.reads.len()) {
            (0, _) => Ok(false),
            (present, depth) if present == depth => Ok(true),
            _ => Err(invalid(
                "the Hamming reads carry type-spin tensors on only some depths",
            )),
        }
    }

    /// The same reads with every parameter drawn from `seed`, for the step cache's gates
    /// (claude-279): unit rotations (the type spin's too, when on), margins from 37% to 69% of a
    /// head's bits, so heads both keep and drop candidates, and weights in [-0.5, 0.5].
    #[cfg(test)]
    pub(crate) fn randomized(self, seed: u64) -> Result<Self> {
        fn draw(state: &mut u64) -> f32 {
            *state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((*state >> 33) % 20_001) as f32 / 10_000.0 - 1.0
        }
        let mut state = seed;
        for read in &self.reads {
            let width = read.weight.dim(1)?;
            let (lanes, head_bits) = (width / 4, (width / read.heads) as f32);
            let device = read.weight.device().clone();
            let mut rotations = vec![
                &read.query_rotation,
                &read.key_rotation,
                &read.value_rotation,
            ];
            rotations.extend(read.step_rotation.as_ref());
            for var in rotations {
                let values: Vec<f32> = (0..lanes)
                    .flat_map(|_| {
                        let q = [
                            draw(&mut state),
                            draw(&mut state),
                            draw(&mut state),
                            draw(&mut state),
                        ];
                        let norm = q.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-3);
                        q.map(|v| v / norm)
                    })
                    .collect();
                var.set(&Tensor::from_vec(values, (lanes, 4), &device)?)?;
            }
            let margins: Vec<f32> = (0..read.heads)
                .map(|_| head_bits * (0.53 + 0.16 * draw(&mut state)))
                .collect();
            read.margin
                .set(&Tensor::from_vec(margins, read.heads, &device)?)?;
            let weights: Vec<f32> = (0..2 * width).map(|_| 0.5 * draw(&mut state)).collect();
            read.weight
                .set(&Tensor::from_vec(weights, (2, width), &device)?)?;
        }
        Ok(self)
    }

    /// The read inside the stream after `depth` layers, if one runs there.
    pub fn at_depth(&self, depth: usize) -> Option<&HammingRead> {
        if depth == HAMMING_READOUT_DEPTH {
            return None;
        }
        HAMMING_DEPTHS
            .iter()
            .position(|&d| d == depth)
            .and_then(|i| self.reads.get(i))
    }

    /// The read on the final norm's output.
    pub fn readout(&self) -> Option<&HammingRead> {
        HAMMING_DEPTHS
            .iter()
            .position(|&d| d == HAMMING_READOUT_DEPTH)
            .and_then(|i| self.reads.get(i))
    }

    /// Every learned parameter, in a fixed order for the optimizer.
    pub fn variables(&self) -> Vec<&Var> {
        self.reads.iter().flat_map(|r| r.variables()).collect()
    }

    /// The mean absolute value of every read's weight, as a scalar tensor.
    pub fn weight_mean_abs(&self) -> Result<Tensor> {
        let parts = self
            .reads
            .iter()
            .map(|r| r.weight.as_tensor().abs()?.mean_all())
            .collect::<candle_core::Result<Vec<_>>>()?;
        Ok(Tensor::stack(&parts, 0)?.mean_all()?)
    }

    /// Every read's margins in bits, read by read and head by head.
    pub fn margins(&self) -> Result<Vec<f64>> {
        let mut margins = Vec::new();
        for read in &self.reads {
            for m in read.margin.as_tensor().to_vec1::<f32>()? {
                margins.push(f64::from(m));
            }
        }
        Ok(margins)
    }

    /// Write the reads to `directory/hamming_read.safetensors`, with their head count.
    pub fn save(&self, directory: &Path) -> Result<()> {
        std::fs::create_dir_all(directory)?;
        let mut tensors = std::collections::HashMap::new();
        for (depth, read) in HAMMING_DEPTHS.iter().zip(&self.reads) {
            for (name, var) in read.named() {
                tensors.insert(format!("d{depth}.{name}"), var.as_tensor().clone());
            }
        }
        let heads = self.reads.first().map_or(0, |r| r.heads) as u32;
        let device = self.reads[0].weight.device();
        tensors.insert("heads".to_owned(), Tensor::new(&[heads], device)?);
        // Only a two-token key is recorded, so a one-token read's file is byte for byte as before;
        // mixed keys are recorded as their list alone (claude-268).
        if let Some(keys) = self.mixed_copy_keys() {
            let keys: Vec<u32> = keys.iter().map(|&k| k as u32).collect();
            tensors.insert("copy_keys".to_owned(), Tensor::new(keys.as_slice(), device)?);
        } else if self.copy_key() != 1 {
            let copy_key = self.copy_key() as u32;
            tensors.insert("copy_key".to_owned(), Tensor::new(&[copy_key], device)?);
        }
        // claude-333: only a held read records its rows, so every other file is as before.
        if self.hold_rows() > 0 {
            let rows = self.hold_rows() as u32;
            tensors.insert("hold_rows".to_owned(), Tensor::new(&[rows], device)?);
        }
        // claude-335: likewise only a carried read records its carry's rows.
        if self.carry_rows() > 0 {
            let rows = self.carry_rows() as u32;
            tensors.insert("carry_rows".to_owned(), Tensor::new(&[rows], device)?);
        }
        candle_core::safetensors::save(&tensors, directory.join(FILE))?;
        Ok(())
    }

    /// The reads saved in `directory`, or none when the checkpoint has none.
    pub fn load(directory: &Path, device: &Device) -> Result<Option<Self>> {
        let path = directory.join(FILE);
        if !path.exists() {
            return Ok(None);
        }
        let tensors = candle_core::safetensors::load(path, device)?;
        let get = |name: &str| -> Result<Var> {
            let tensor = tensors
                .get(name)
                .ok_or_else(|| invalid(format!("the saved Hamming read has no {name}")))?;
            Ok(Var::from_tensor(tensor)?)
        };
        let heads = get("heads")?.as_tensor().to_vec1::<u32>()?;
        let heads = *heads
            .first()
            .ok_or_else(|| invalid("the saved head count is empty"))? as usize;
        // A read saved before claude-261, or with the one-token key, records none.
        let copy_key = match tensors.get("copy_key") {
            None => 1,
            Some(tensor) => *tensor
                .to_vec1::<u32>()?
                .first()
                .ok_or_else(|| invalid("the saved copy key is empty"))?
                as usize,
        };
        let mut reads = Vec::with_capacity(HAMMING_DEPTHS.len());
        for depth in HAMMING_DEPTHS {
            reads.push(HammingRead {
                heads,
                query_rotation: get(&format!("d{depth}.query_rotation"))?,
                key_rotation: get(&format!("d{depth}.key_rotation"))?,
                value_rotation: get(&format!("d{depth}.value_rotation"))?,
                margin: get(&format!("d{depth}.margin"))?,
                weight: get(&format!("d{depth}.weight"))?,
                // Present only when the read was trained with the type heads' position signal.
                step_rotation: tensors
                    .get(&format!("d{depth}.step_rotation"))
                    .map(Var::from_tensor)
                    .transpose()?,
                copy_keys: Vec::new(),
                // Present only when the read was trained with the ask heads (claude-330).
                ask: tensors
                    .get(&format!("d{depth}.ask"))
                    .map(Var::from_tensor)
                    .transpose()?,
                hold: None,
                carry: None,
            });
        }
        // claude-333: a hold is saved as the stream read's gain and its row count, both or
        // neither, and only with the ask heads, whose write it holds.
        let hold_rows = match tensors.get("hold_rows") {
            None => 0,
            Some(tensor) => *tensor
                .to_vec1::<u32>()?
                .first()
                .ok_or_else(|| invalid("the saved hold's row count is empty"))?
                as usize,
        };
        for (i, depth) in HAMMING_DEPTHS.iter().enumerate() {
            let gain = tensors.get(&format!("d{depth}.hold"));
            match (i, gain, hold_rows) {
                (_, None, _) if i > 0 || hold_rows == 0 => {}
                (0, Some(gain), rows) if rows > 0 => {
                    let Some(ask) = reads[0].ask.as_ref() else {
                        return Err(invalid("the saved Hamming read holds an ask write but has no ask heads"));
                    };
                    // claude-334: one gain per ask row, so a lookup never reads past either.
                    let (gains, asks) = (gain.dims1()?, ask.dim(0)?);
                    if gains != asks {
                        return Err(invalid(format!(
                            "the saved hold has {gains} gains for {asks} ask rows"
                        )));
                    }
                    reads[0].hold = Some(Hold { rows, gain: Var::from_tensor(gain)? });
                }
                (0, None, _) => {
                    return Err(invalid("the saved Hamming read records hold rows but no gain"));
                }
                (0, Some(_), _) => {
                    return Err(invalid("the saved Hamming read has a hold gain but no row count"));
                }
                _ => {
                    return Err(invalid(format!(
                        "the saved Hamming read holds a write at depth {depth}; only the stream read holds one"
                    )));
                }
            }
        }
        // claude-335: a carry is saved as the readout read's gain and its row count, both or
        // neither, and only with the ask heads, whose question it carries.
        let carry_rows = match tensors.get("carry_rows") {
            None => 0,
            Some(tensor) => *tensor
                .to_vec1::<u32>()?
                .first()
                .ok_or_else(|| invalid("the saved carry's row count is empty"))?
                as usize,
        };
        for (i, depth) in HAMMING_DEPTHS.iter().enumerate() {
            let gain = tensors.get(&format!("d{depth}.carry"));
            match (i, gain, carry_rows) {
                (_, None, _) if i != 1 || carry_rows == 0 => {}
                (1, Some(gain), rows) if rows > 0 => {
                    let Some(ask) = reads[1].ask.as_ref() else {
                        return Err(invalid("the saved Hamming read carries a question but has no ask heads"));
                    };
                    let (gains, asks) = (gain.dims1()?, ask.dim(0)?);
                    if gains != asks {
                        return Err(invalid(format!(
                            "the saved carry has {gains} gains for {asks} ask rows"
                        )));
                    }
                    reads[1].carry = Some(Hold { rows, gain: Var::from_tensor(gain)? });
                }
                (1, None, _) => {
                    return Err(invalid("the saved Hamming read records carry rows but no gain"));
                }
                (1, Some(_), _) => {
                    return Err(invalid("the saved Hamming read has a carry gain but no row count"));
                }
                _ => {
                    return Err(invalid(format!(
                        "the saved Hamming read carries a question at depth {depth}; only the readout read carries one"
                    )));
                }
            }
        }
        // The ask heads are all or nothing: a save with them at some depths only would load as a
        // read that the pull refuses and the forward still half uses.
        let asked = reads.iter().filter(|r| r.ask.is_some()).count();
        if asked != 0 && asked != reads.len() {
            return Err(invalid(format!(
                "the saved Hamming read has ask heads at {asked} of its {} depths",
                reads.len()
            )));
        }
        let reads = Self { reads }.with_copy_key(copy_key)?;
        match tensors.get("copy_keys") {
            None => Ok(Some(reads)),
            Some(_) if tensors.contains_key("copy_key") => Err(invalid(
                "the saved Hamming read records both a copy key and per-head copy keys",
            )),
            Some(tensor) => {
                let keys: Vec<usize> = tensor.to_vec1::<u32>()?.into_iter().map(|k| k as usize).collect();
                if keys.windows(2).all(|w| w[0] == w[1]) {
                    return Err(invalid(
                        "the saved Hamming read records per-head copy keys that are all the same; one key is saved as copy_key",
                    ));
                }
                Ok(Some(reads.with_copy_keys(&keys)?))
            }
        }
    }
}

impl HammingRead {
    fn new(heads: usize, width: usize, device: &Device) -> Result<Self> {
        if heads < 2 || !width.is_multiple_of(heads) {
            return Err(invalid(
                "a Hamming read needs at least 2 heads (copy and type), a count that divides the width",
            ));
        }
        let head_width = width / heads;
        if !head_width.is_multiple_of(4) || head_width > 64 {
            return Err(invalid(
                "a Hamming head needs whole 4-lane blocks and at most 64 coordinates (one word)",
            ));
        }
        let lanes = width / 4;
        let identity: Vec<f32> = (0..lanes).flat_map(|_| [1.0f32, 0.0, 0.0, 0.0]).collect();
        let rotation = || -> Result<Var> {
            Ok(Var::from_tensor(&Tensor::from_vec(
                identity.clone(),
                (lanes, 4),
                device,
            )?)?)
        };
        // Copy heads start with a quarter of a head's bits as margin, so only near-exact repeats
        // count; type heads start with half, so they read widely while they learn what to ask.
        let margins: Vec<f32> = (0..heads)
            .map(|h| {
                if h < heads / 2 {
                    (head_width / 4) as f32
                } else {
                    (head_width / 2) as f32
                }
            })
            .collect();
        Ok(Self {
            heads,
            query_rotation: rotation()?,
            key_rotation: rotation()?,
            value_rotation: rotation()?,
            margin: Var::from_tensor(&Tensor::from_vec(margins, heads, device)?)?,
            weight: Var::zeros((2, width), DType::F32, device)?,
            step_rotation: None,
            copy_keys: vec![1; heads / 2],
            ask: None,
            hold: None,
            carry: None,
        })
    }

    fn named(&self) -> Vec<(&'static str, &Var)> {
        let mut named = vec![
            ("query_rotation", &self.query_rotation),
            ("key_rotation", &self.key_rotation),
            ("value_rotation", &self.value_rotation),
            ("margin", &self.margin),
            ("weight", &self.weight),
        ];
        if let Some(step) = &self.step_rotation {
            named.push(("step_rotation", step));
        }
        if let Some(ask) = &self.ask {
            named.push(("ask", ask));
        }
        if let Some(hold) = &self.hold {
            named.push(("hold", &hold.gain));
        }
        if let Some(carry) = &self.carry {
            named.push(("carry", &carry.gain));
        }
        named
    }

    fn variables(&self) -> Vec<&Var> {
        self.named().into_iter().map(|(_, var)| var).collect()
    }

    /// The copy heads' and the type heads' questions, at full width, before the cut into heads:
    /// a copy head asks with the current token's point through the key's rotation; a type head
    /// with the residual through the query's; an ask head (claude-330) with the current token's
    /// point plus its question row, through the key's rotation, so at zero rows it asks exactly
    /// as a copy head does. Call it inside [`as_trained`], as the read does.
    fn asks(
        &self,
        current: &Tensor,
        points: &Tensor,
        ids: &[u32],
        time: usize,
    ) -> Result<(Tensor, Tensor)> {
        let copy = Self::rotate(&self.key_rotation, current)?;
        let typed = match &self.ask {
            None => Self::rotate(&self.query_rotation, points)?,
            Some(ask) => Self::rotate(&self.key_rotation, &self.asked(current, ask, ids, time)?)?,
        };
        Ok((copy, typed))
    }

    /// The ask heads' question points (claude-330): each row's detached token point plus its
    /// token's row of the question table, the only part that learns. With the question carry
    /// (claude-335, the readout read only), each row adds the earlier rows' question points of
    /// its window, `ids` in windows of `time`, each scaled by its own token's gain, unless
    /// [`set_carry_off`] cuts it.
    fn asked(&self, current: &Tensor, ask: &Var, ids: &[u32], time: usize) -> Result<Tensor> {
        check_ask_ids(ask, ids)?;
        let index = Tensor::from_vec(ids.to_vec(), ids.len(), current.device())?;
        let asked = current.add(&ask.as_tensor().index_select(&index, 0)?)?;
        match self.carried(&asked, ids, time)? {
            Some(carried) => Ok(asked.add(&carried)?),
            None => Ok(asked),
        }
    }

    /// claude-335's carried term at every row of `ids`' windows of `time`, for the uncarried
    /// question points `asked` `(rows, width)`: `Σ_{s=1..h} c[x_{t−s}] · asked_{t−s}`, over rows
    /// of the same window only, as [`held_term`] sums a hold. `None` with no carry, or under
    /// [`set_carry_off`].
    fn carried(&self, asked: &Tensor, ids: &[u32], time: usize) -> Result<Option<Tensor>> {
        let Some(carry) = &self.carry else {
            return Ok(None);
        };
        if carry_off() {
            return Ok(None);
        }
        if time == 0 || !ids.len().is_multiple_of(time) {
            return Err(invalid("the question carry needs whole windows"));
        }
        let batch = ids.len() / time;
        Ok(Some(held_term(&carry.gain, asked, ids, batch, time, carry.rows)?))
    }

    /// [`HammingReads::ask_pull`] at this depth: the sum over `spans` and the ask heads of each
    /// head's least `bits(t, s) / head_width` over the span's `(t, s)` pairs, and how many terms
    /// it sums. A rotation acts row by row, lane by lane, so the pairs' rows alone are rotated.
    fn pull_bits(
        &self,
        ids: &[u32],
        time: usize,
        table: &Tensor,
        pairs: &[(u32, u32)],
        spans: &[std::ops::Range<usize>],
    ) -> Result<(Tensor, usize)> {
        let ask = self
            .ask
            .as_ref()
            .ok_or_else(|| invalid("the found-copy pull needs the ask heads"))?;
        let (n, device) = (pairs.len(), table.device());
        let width = table.dim(1)?;
        let head_width = width / self.heads;
        let (copy_heads, asks) = (self.copy_heads(), self.heads - self.copy_heads());
        let table = table.detach();
        let tokens = |rows: Vec<u32>| Tensor::from_vec(rows, n, device);
        // The asking row's own token, and the token before each candidate, which its key names.
        let asking: Vec<u32> = pairs.iter().map(|&(t, _)| ids[t as usize]).collect();
        check_ask_ids(ask, &asking)?;
        let named: Vec<u32> = pairs.iter().map(|&(_, s)| ids[s as usize - 1]).collect();
        let current = table.index_select(&tokens(asking.clone())?, 0)?;
        let keyed = table.index_select(&tokens(named)?, 0)?;
        let rotation = self.key_rotation.as_tensor().detach();
        let (query, key) = as_trained(|| {
            let question = current.add(&ask.as_tensor().index_select(&tokens(asking)?, 0)?)?;
            // claude-335: a carried read asks the pull's question as its forward does, the
            // window's earlier question points added at each asking row.
            let question = if self.carry.is_some() {
                check_ask_ids(ask, ids)?;
                let every = Tensor::from_vec(ids.to_vec(), ids.len(), device)?;
                let asked = table
                    .index_select(&every, 0)?
                    .add(&ask.as_tensor().index_select(&every, 0)?)?;
                match self.carried(&asked, ids, time)? {
                    Some(carried) => {
                        let at: Vec<u32> = pairs.iter().map(|&(t, _)| t).collect();
                        question.add(&carried.index_select(&tokens(at)?, 0)?)?
                    }
                    None => question,
                }
            } else {
                question
            };
            Ok((
                Self::rotate_by(&rotation, &question)?,
                Self::rotate_by(&rotation, &keyed)?.detach(),
            ))
        })?;
        let span = (copy_heads * head_width, asks * head_width);
        let signs = |p: &Tensor| -> Result<Tensor> {
            Ok(p.ge(0.0)?.to_dtype(DType::F32)?.affine(2.0, -1.0)?)
        };
        // The read's straight-through code on the question; the key's code is a constant.
        let q = query.narrow(1, span.0, span.1)?;
        let q = q.add(&signs(&q)?.sub(&q)?.detach())?;
        let k = signs(&key.narrow(1, span.0, span.1)?)?;
        let agree = q.mul(&k)?.reshape((n, asks, head_width))?.sum(2)?;
        let bits = agree.affine(-0.5, head_width as f64 / 2.0)?;
        let values = bits.detach().to_vec2::<f32>()?;
        let mut chosen = Vec::with_capacity(spans.len() * asks);
        for range in spans {
            for h in 0..asks {
                let best = range
                    .clone()
                    .min_by(|&a, &b| values[a][h].total_cmp(&values[b][h]).then(a.cmp(&b)))
                    .ok_or_else(|| invalid("a found-copy span is empty"))?;
                chosen.push((best * asks + h) as u32);
            }
        }
        let terms = chosen.len();
        let picked = bits
            .flatten_all()?
            .index_select(&Tensor::from_vec(chosen, terms, device)?, 0)?;
        Ok((picked.sum_all()?.affine(1.0 / head_width as f64, 0.0)?, terms))
    }

    /// `points` rows rotated lane by lane by `rotation`'s unit quaternions.
    fn rotate(rotation: &Var, points: &Tensor) -> Result<Tensor> {
        Self::rotate_by(rotation.as_tensor(), points)
    }

    /// [`Self::rotate`] by a plain tensor, so a caller can pass a detached rotation.
    fn rotate_by(rotation: &Tensor, points: &Tensor) -> Result<Tensor> {
        let gate = rotation
            .reshape((1, points.dim(1)?))?
            .broadcast_as(points.shape())?
            .contiguous()?;
        hamilton_gate(&gate, points)
    }

    /// The first half of the heads are copy heads; the rest are type heads.
    pub fn copy_heads(&self) -> usize {
        self.heads / 2
    }

    /// The read's heads, copy heads first.
    pub fn heads(&self) -> usize {
        self.heads
    }

    /// The read applied inside the stream, to the residual `(batch, time, width)` of the windows
    /// `ids`, with token points from the shared `table`: `x + w_c ⊙ c + w_y ⊙ y`. Returns the
    /// same shape.
    pub fn apply(&self, residual: &Tensor, ids: &[u32], table: &Tensor) -> Result<Tensor> {
        self.read(residual, ids, table, false, 0)
    }

    /// [`Self::apply`] at each window's last position only (claude-279, the decode cache's step):
    /// `residual` `(batch, time, width)` holds every position so far and `ids` every row's whole
    /// window. Returns that position's `(batch, 1, width)`, the row the whole window's read gives.
    pub(crate) fn apply_last(
        &self,
        residual: &Tensor,
        ids: &[u32],
        table: &Tensor,
    ) -> Result<Tensor> {
        let time = residual.dim(1)?;
        self.read(residual, ids, table, false, time.saturating_sub(1))
    }

    /// [`Self::apply_readout`] at each window's last position only, as [`Self::apply_last`].
    pub(crate) fn apply_readout_last(
        &self,
        output: &Tensor,
        ids: &[u32],
        table: &Tensor,
    ) -> Result<Tensor> {
        let time = output.dim(1)?;
        self.read(output, ids, table, true, time.saturating_sub(1))
    }

    /// The read applied to the final norm's output `(batch, time, width)`, the points the
    /// nearest readout decodes: where a copy head found an earlier copy of the current token,
    /// the prediction moves toward the copied point, `n + w_c ⊙ (c − n) + w_y ⊙ y`, and stays
    /// `n + w_y ⊙ y` where none did. At `w_c = 1` a found copy is decoded exactly. A nearest
    /// readout names the point a prediction is at, so a copy is a move, not an added term.
    pub fn apply_readout(&self, output: &Tensor, ids: &[u32], table: &Tensor) -> Result<Tensor> {
        self.read(output, ids, table, true, 0)
    }

    /// The read at the positions `from..time` of each window: every position (`from` 0), or the
    /// decode cache's step at the last (claude-279). The keys, values and asks are computed over
    /// the whole window either way, so a step's are the whole window's; only the asking rows go
    /// on to the choice, and from there each row's arithmetic is its own.
    fn read(
        &self,
        residual: &Tensor,
        ids: &[u32],
        table: &Tensor,
        readout: bool,
        from: usize,
    ) -> Result<Tensor> {
        let (batch, time, width) = residual.dims3()?;
        let rows = batch * time;
        if ids.len() != rows {
            return Err(invalid("the Hamming read needs whole windows"));
        }
        if from >= time.max(1) {
            return Err(invalid(format!(
                "the Hamming read asks from position {from} of a window of {time}"
            )));
        }
        // claude-333: the held write sums earlier rows' writes, which a step does not carry. It
        // is refused before any arithmetic, so the refusal holds on every device.
        if from > 0 && self.hold.is_some() {
            return Err(invalid(
                "the decode step does not serve a held ask write (--hamming-hold, claude-333)",
            ));
        }
        // claude-335: the carry sums earlier rows' questions; serving it at a step is a later
        // unit, so the step refuses it before any arithmetic too.
        if from > 0 && self.carry.is_some() {
            return Err(invalid(
                "the decode step does not serve a carried question (--hamming-carry-question, claude-335)",
            ));
        }
        // A window of one token has no earlier position to read.
        if time < 2 {
            return Ok(residual.clone());
        }
        let device = residual.device();
        let head_width = width / self.heads;
        let copy_width = self.copy_heads() * head_width;
        let points = residual.reshape((rows, width))?;
        // Each position's own token and the token before it, as the shared table's points,
        // detached. A window's first position has none and takes its own; it is never a candidate.
        let previous: Vec<u32> = (0..rows)
            .map(|r| if r % time == 0 { ids[r] } else { ids[r - 1] })
            .collect();
        let table = table.detach();
        let token_points = |ids: Vec<u32>| -> Result<Tensor> {
            Ok(table.index_select(&Tensor::from_vec(ids, rows, device)?, 0)?)
        };
        let current = token_points(ids.to_vec())?;
        let previous = token_points(previous)?;
        // Copy key 2 (claude-261): the token two before each row, for the key's second half.
        let pair: Vec<bool> = self.copy_keys.iter().map(|&k| k == 2).collect();
        let before = pair
            .contains(&true)
            .then(|| token_points(two_back(ids, time)))
            .transpose()?;
        let (query, key, value) = as_trained(|| {
            // Copy heads ask with the current token's point through the key's own rotation, so a
            // repeated token stays an exact match however that rotation learns; type heads ask
            // with the residual through the query's. Rotations act lane by lane, so each half is
            // cut after.
            let (copy_asks, type_asks) = self.asks(&current, &points, ids, time)?;
            let key = Self::rotate(&self.key_rotation, &previous)?;
            // With copy key 2, a copy head asks with `x_t`'s half and then `x_{t−1}`'s, and its
            // key is `x_{s−1}`'s half and then `x_{s−2}`'s, every half through the key's rotation,
            // so a repeat of both tokens is still an exact match however it learns.
            let (copy_asks, key) = match &before {
                None => (copy_asks, key),
                Some(before) => {
                    let before = Self::rotate(&self.key_rotation, before)?;
                    (
                        pair_halves(&copy_asks, &key, &pair, head_width)?,
                        pair_halves(&key, &before, &pair, head_width)?,
                    )
                }
            };
            let query = Tensor::cat(
                &[
                    &copy_asks.narrow(1, 0, copy_width)?,
                    &type_asks.narrow(1, copy_width, width - copy_width)?,
                ],
                1,
            )?;
            // The type heads' position signal, when on: each position's frame is the product of
            // its window's icosian steps so far, one per token; a type head's query at `t` and key
            // at `s` are each turned by their own frame's inverse, so their angle depends only on
            // the steps between (the sign codes below also see the frame the two share).
            let (query, key) = match &self.step_rotation {
                None => (query, key),
                Some(step_rotation) => {
                    let type_width = width - copy_width;
                    let steps = snap_icosian(
                        &Self::rotate(step_rotation, &current)?
                            .narrow(1, copy_width, type_width)?
                            .contiguous()?,
                    )?;
                    let inverse = conjugate(&prefix_frames(&steps, batch, time)?)?;
                    let turn = |p: &Tensor| -> Result<Tensor> {
                        let turned = hamilton_gate(
                            &inverse,
                            &p.narrow(1, copy_width, type_width)?.contiguous()?,
                        )?;
                        Ok(Tensor::cat(&[&p.narrow(1, 0, copy_width)?, &turned], 1)?)
                    };
                    (turn(&query)?, turn(&key)?)
                }
            };
            let value = Self::rotate(&self.value_rotation, &points)?;
            Ok((query, key, value))
        })?;
        // ±1 by `p ≥ 0` forward, as `pack_signs` packs it, so 0 is a 1 bit; the identity backward.
        let code = |p: &Tensor| -> Result<Tensor> {
            let bits = p.ge(0.0)?.to_dtype(DType::F32)?.affine(2.0, -1.0)?;
            Ok(p.add(&bits.sub(p)?.detach())?)
        };
        // claude-251: the coordinates the sign reads, kept for the decision trace only.
        let tracing = TRACE.with(|t| t.borrow().is_some());
        let raw = (tracing && COORDS.with(|c| c.get())).then(|| (query.clone(), key.clone()));
        let (query, key) = (code(&query)?, code(&key)?);
        // The stream read is the first of `HAMMING_DEPTHS`; the readout read, the last.
        let depth = if readout {
            HAMMING_READOUT_DEPTH
        } else {
            HAMMING_DEPTHS[0]
        };
        // claude-253: the codes forced in this read, in place of the computed ones; the
        // coordinates the trace records stay as computed.
        let forced: Vec<ForcedRowCode> = CODE_FORCE.with(|f| {
            f.borrow()
                .iter()
                .filter(|f| f.depth == depth)
                .copied()
                .collect()
        });
        if forced.iter().any(|f| f.head >= self.heads) {
            return Err(invalid(
                "a forced code names a head its read does not serve (an instrument fault)",
            ));
        }
        let (query, key) = if forced.is_empty() {
            (query, key)
        } else {
            (
                force_codes(&query, &forced, false, head_width)?,
                force_codes(&key, &forced, true, head_width)?,
            )
        };
        let zero = Tensor::new(&[0u32], device)?;
        let margins = self.margin.as_tensor();
        // A decode's scope names positions in its one window (claude-215); a row cut names one
        // limit per row (claude-216). At most one is set.
        let cut = CUT.with(|c| c.get());
        let row_cut = ROW_CUT.with(|c| c.borrow().clone());
        if cut.is_some() && batch != 1 {
            return Err(invalid("the read's decode scope is for one window"));
        }
        if cut.is_some() && row_cut.is_some() {
            return Err(invalid("the read has a decode cut and a row cut at once"));
        }
        if row_cut.as_ref().is_some_and(|(_, limits)| limits.len() != rows) {
            return Err(invalid("the read's row cut names a limit for every row"));
        }
        // Each names rows or positions of the whole window; the decode cache's step refuses them
        // first (`step_instrument`).
        if from > 0 && (cut.is_some() || row_cut.is_some() || tracing || !forced.is_empty()) {
            return Err(invalid(
                "the read's step carries no decode cut, row cut, trace or forced code (an instrument fault)",
            ));
        }
        // The positions that ask: all of them, or the step's last.
        let asks = time - from;
        let asking = batch * asks;
        let ask_rows = |t: &Tensor| -> Result<Tensor> {
            if from == 0 {
                return Ok(t.clone());
            }
            Ok(t.reshape((batch, time, width))?
                .narrow(1, from, asks)?
                .contiguous()?
                .reshape((asking, width))?)
        };
        let (query, asked) = (ask_rows(&query)?, ask_rows(&points)?);
        // What each kind of head brings back, at full width: a copy head the table's point of the
        // kept position's own token, the readout's own point for it (the 26 Sep head's tied
        // readout); a type head the kept position's rotated residual.
        // claude-330: an ask head brings back what a copy head does, the kept token's point.
        let sources = [
            current.reshape((batch, time, width))?,
            match self.ask {
                None => value.reshape((batch, time, width))?,
                Some(_) => current.reshape((batch, time, width))?,
            },
        ];
        let mut sums = [
            Tensor::zeros((asking, width), DType::F32, device)?,
            Tensor::zeros((asking, width), DType::F32, device)?,
        ];
        // How many heads of each kind found any candidate inside their margin, per row.
        let mut found = [
            Tensor::zeros((asking, 1), DType::F32, device)?,
            Tensor::zeros((asking, 1), DType::F32, device)?,
        ];
        // claude-235: with both off, the read computes as before them. Since claude-239 the tie
        // rule is on by default and the trace off.
        let tie = TIE.with(|t| t.get());
        let mut traced = Vec::new();
        for h in 0..self.heads {
            let q = query.narrow(1, h * head_width, head_width)?.contiguous()?;
            let k = key.narrow(1, h * head_width, head_width)?.contiguous()?;
            // A window's first position has no previous token, so its key names nothing: the
            // candidates start at 1.
            let limits = row_limits(cut, row_cut.as_ref(), h, self.copy_heads(), time, asking);
            // A whole window keeps the call it made before the step existed (claude-279).
            let (picks, real) = if from == 0 {
                choose_sorted(&q, &k, batch, time, HAMMING_KEPT, 1, &limits, tie)?
            } else {
                choose_sorted_from(&q, &k, batch, time, from, HAMMING_KEPT, 1, &limits, tie)?
            };
            if tracing {
                let head = |p: &Tensor| -> Result<Vec<Vec<f32>>> {
                    Ok(p.narrow(1, h * head_width, head_width)?.contiguous()?.to_vec2::<f32>()?)
                };
                traced.push(HeadChoice {
                    query: q.to_vec2::<f32>()?,
                    key: k.to_vec2::<f32>()?,
                    picks: picks.to_vec2::<u32>()?,
                    real: real.to_vec2::<u8>()?,
                    coords: match &raw {
                        Some((query, key)) => Some((head(query)?, head(key)?)),
                        None => None,
                    },
                });
            }
            // A pick that is not real points at its window's first position, whose successor is
            // still inside the window, and carries weight 0.
            let picks = real.where_cond(&picks, &zero.broadcast_as(picks.shape())?)?;
            let kept = picks.dim(1)?;
            // Gather inside each window, `(batch, time · kept, columns)`: the backward then
            // spreads over windows and coordinates instead of looping over every pick.
            let index = |columns: usize| -> Result<Tensor> {
                Ok(picks
                    .reshape((batch, asks * kept, 1))?
                    .broadcast_as((batch, asks * kept, columns))?
                    .contiguous()?)
            };
            let kept_keys = k
                .reshape((batch, time, head_width))?
                .gather(&index(head_width)?, 1)?
                .reshape((asking, kept, head_width))?;
            let bits = hamming_bits(&q.unsqueeze(1)?, &kept_keys)?;
            // Each candidate weighs `max(0, m − d) / m`, between 0 and 1, so an exact match
            // (`d = 0`) counts in full. The head reports the mean over the candidates it reads,
            // a count and not a learned sum, and reads nothing when none is inside its margin.
            let margin = margins.narrow(0, h, 1)?.reshape((1, 1))?;
            let weights = margin
                .broadcast_sub(&bits)?
                .relu()?
                .broadcast_div(&margin.maximum(1.0)?)?
                .mul(&real.to_dtype(DType::F32)?)?;
            let count = weights
                .gt(0.0)?
                .to_dtype(DType::F32)?
                .sum_keepdim(1)?
                .maximum(1.0)?;
            // The key at `s` names token `s − 1`, so the kept position is already the one after
            // the earlier match: what it holds is what followed.
            let kind = usize::from(h >= self.copy_heads());
            let payload = sources[kind]
                .gather(&index(width)?, 1)?
                .reshape((asking, kept, width))?;
            sums[kind] = sums[kind].add(
                &payload
                    .broadcast_mul(&weights.unsqueeze(2)?)?
                    .sum(1)?
                    .broadcast_div(&count)?,
            )?;
            found[kind] =
                found[kind].add(&weights.gt(0.0)?.to_dtype(DType::F32)?.max_keepdim(1)?)?;
        }
        if tracing {
            let reads = (0..batch)
                .map(|b| {
                    let window = b * time..(b + 1) * time;
                    let heads = traced
                        .iter()
                        .enumerate()
                        .map(|(h, c): (usize, &HeadChoice)| {
                            // The window's rows where this head's code was forced, whose
                            // coordinates need not sign to it.
                            let rows = |key: bool| -> Vec<usize> {
                                forced
                                    .iter()
                                    .filter(|f| {
                                        f.head == h && f.key == key && window.contains(&f.row)
                                    })
                                    .map(|f| f.row - window.start)
                                    .collect()
                            };
                            trace_head(
                                &c.query[window.clone()],
                                &c.key[window.clone()],
                                &c.picks[window.clone()],
                                &c.real[window.clone()],
                                c.coords
                                    .as_ref()
                                    .map(|(q, k)| (&q[window.clone()], &k[window.clone()])),
                                (&rows(false), &rows(true)),
                            )
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Ok(HammingTraceRead { depth, heads })
                })
                .collect::<Result<Vec<_>>>()?;
            TRACE.with(|t| match t.borrow_mut().as_mut() {
                Some(windows) => push_trace_reads(windows, ids, time, reads, readout),
                None => Ok(()),
            })?;
        }
        // Each kind's mean over the heads that found something, so a head that found nothing does
        // not dilute one that did.
        let means = (0..2)
            .map(|kind| Ok(sums[kind].broadcast_div(&found[kind].maximum(1.0)?)?))
            .collect::<Result<Vec<_>>>()?;
        let weight = self.weight.as_tensor();
        // claude-328: a class [`set_class_ablation`] drops at this read adds nothing.
        let off = class_ablation()[usize::from(readout)];
        let class = |kind: usize| -> Result<Tensor> {
            let w = weight.narrow(0, kind, 1)?;
            Ok(if off[kind] { w.zeros_like()? } else { w })
        };
        let (copy_weight, type_weight) = (class(0)?, class(1)?);
        // Only where a head of the kind found an earlier copy does the prediction move.
        let moved = |kind: usize| -> Result<Tensor> {
            let matched = found[kind].gt(0.0)?.to_dtype(DType::F32)?;
            Ok(means[kind].sub(&asked)?.broadcast_mul(&matched)?)
        };
        let copied = if readout { moved(0)? } else { means[0].clone() };
        // claude-330: an ask head combines in the copy form, through the type weight row.
        let typed = match (&self.ask, readout) {
            (Some(_), true) => moved(1)?,
            _ => means[1].clone(),
        };
        let written = typed.broadcast_mul(&type_weight)?;
        let added = copied.broadcast_mul(&copy_weight)?.add(&written)?;
        // claude-333: the stream read's ask write `u = w_y ⊙ y`, held for the next rows of its
        // window, each scaled by its own token's gain (`from` is 0 here: a step refuses a hold).
        // claude-334's `−H` ([`set_hold_off`]) drops the held term alone.
        let held = match (&self.hold, readout) {
            (Some(hold), false) if !hold_off() => {
                Some(held_term(&hold.gain, &written, ids, batch, time, hold.rows)?)
            }
            _ => None,
        };
        let added = match &held {
            Some(held) => added.add(held)?,
            None => added,
        };
        let out = asked.add(&added)?;
        if from == 0 {
            capture_rows(
                readout,
                &residual.reshape((rows, width))?,
                &means[1],
                &found[1],
                &written,
                held.as_ref(),
                &out,
            )?;
        } else if CAPTURE.with(|c| c.borrow().is_some()) {
            return Err(invalid("the row capture needs whole windows, not a decode step"));
        }
        Ok(out.reshape((batch, asks, width))?)
    }
}

/// claude-238: runs `f`, the read's rotations, as training ran them, through the gate hook's
/// pass-through (`HamiltonActive`) and never its D11 or export snap. The artifact serves the
/// rotations raw (`lut-stack/5`), and a snapped read was the float reference's own gap
/// (claude-237). `with_d11_snaps` gives the hook back whole when `f` returns, `Ok` or `Err` (arm,
/// call index, export flag, transition snap, telemetry; a panic inside `f` skips it), so the
/// trunk's gates after the read snap as before and number their calls as a trunk without the read
/// would. Whatever the arm outside, the read's
/// forward runs as `HamiltonActive`, uncounted by telemetry and snap capture; only
/// `in_space_long.rs` (itself `HamiltonActive`) and `fbs-snapshot` install a read, so no control
/// arm meets one. It covers the forward only: a D11-bridge training step (`with_d11_training`)
/// would still run the read's backward through the snap, and no trainer runs one with a read.
fn as_trained<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    use crate::hamilton_sheet::{is_snapping_transition, with_d11_snaps, D11Snaps};
    let snaps = D11Snaps {
        gates: false,
        transition: is_snapping_transition(),
    };
    with_d11_snaps(snaps, f)
}

/// Each 4-block of `points` `(rows, 4 · lanes)` as the unit icosian (the 120 elements of 2I,
/// SpiralCore's quaternion group, `hamilton_sheet::h4_unit_roots_f32`) nearest it in direction,
/// by the largest inner product, as `hamilton_sheet`'s own icosian snap chooses. Straight
/// through: the forward pass is the icosian, the backward pass the identity.
pub(crate) fn snap_icosian(points: &Tensor) -> Result<Tensor> {
    let (rows, width) = points.dims2()?;
    let device = points.device();
    let roots: Vec<f32> = crate::hamilton_sheet::h4_unit_roots_f32()
        .iter()
        .flatten()
        .copied()
        .collect();
    let roots = Tensor::from_vec(roots, (120, 4), device)?;
    let nearest = points
        .detach()
        .reshape((rows * width / 4, 1, 4))?
        .broadcast_mul(&roots.unsqueeze(0)?)?
        .sum(2)?
        .argmax(1)?;
    let snapped = roots.index_select(&nearest, 0)?.reshape((rows, width))?;
    Ok(points.add(&snapped.sub(points)?.detach())?)
}

/// The inclusive product of each window's steps, `F_t = u_t ⊗ u_{t−1} ⊗ … ⊗ u_0`, lane by lane,
/// for `steps` rows `(batch · time, 4 · lanes)` of unit quaternions. A Hillis–Steele scan: one
/// Hamilton product per doubling of the span, about log₂ `time` of them. The product is
/// associative but not commutative, so the later segment always multiplies from the left.
pub(crate) fn prefix_frames(steps: &Tensor, batch: usize, time: usize) -> Result<Tensor> {
    let (rows, width) = steps.dims2()?;
    let identity: Vec<f32> = (0..width / 4)
        .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
        .collect();
    let identity = Tensor::from_vec(identity, (1, 1, width), steps.device())?;
    let mut frames = steps.reshape((batch, time, width))?;
    let mut span = 1;
    while span < time {
        let earlier = Tensor::cat(
            &[
                &identity.broadcast_as((batch, span, width))?.contiguous()?,
                &frames.narrow(1, 0, time - span)?,
            ],
            1,
        )?;
        frames = hamilton_gate(
            &frames.reshape((rows, width))?.contiguous()?,
            &earlier.reshape((rows, width))?.contiguous()?,
        )?
        .reshape((batch, time, width))?;
        span *= 2;
    }
    Ok(frames.reshape((rows, width))?)
}

/// Each 4-block's quaternion conjugate, `(w, −x, −y, −z)`: a unit quaternion's inverse.
pub(crate) fn conjugate(q: &Tensor) -> Result<Tensor> {
    let width = q.dim(1)?;
    let signs: Vec<f32> = (0..width / 4)
        .flat_map(|_| [1.0f32, -1.0, -1.0, -1.0])
        .collect();
    Ok(q.broadcast_mul(&Tensor::from_vec(signs, (1, width), q.device())?)?)
}

/// A distance no real pair reaches: positions that are not earlier.
const FAR: f32 = 1.0e30;

/// The `neighbours` earlier positions `first ≤ s < t` nearest each row `t` of `query` in mean L1,
/// for all `batch` windows of `time` at once, as positions inside the row's own window
/// `(batch * time, kept)`, and which are real. Only positions before row `r`'s `limits[r]` are
/// candidates; `time` limits none (claude-215, claude-216).
/// One sort per call, where claude-144's choice (`flock_read::choose_nearest`) takes `kept` argmin
/// rounds per window; with `first` 0, the same nearest set, hard and detached. Ties may come out in
/// another order, unless `tie` sets the integer session's tie rule (claude-235; the read's default
/// since claude-239) for ±1 codes:
/// each distance taken as its whole bit count `h` and sorted by the exact key `h · time + s`
/// ([`tie_key`]), so equal distances go to the earlier position whatever the sort's own tie
/// order. Without `tie`, the choice is the one before claude-235.
#[allow(clippy::too_many_arguments)]
pub(crate) fn choose_sorted(
    query: &Tensor,
    key: &Tensor,
    batch: usize,
    time: usize,
    neighbours: usize,
    first: usize,
    limits: &[usize],
    tie: bool,
) -> Result<(Tensor, Tensor)> {
    choose_sorted_in(query, key, batch, time, neighbours, first, limits, tie, DISTANCE_CHUNK_BYTES)
}

/// [`choose_sorted`] for the query positions `from..time` of each window only (claude-279): the
/// decode cache's step asks at its window's last position alone. `query` holds those positions'
/// rows, `(batch · (time − from), width)`; `key` and `limits` are as before, `limits` one per query
/// row. Each query row's distances, mask and sort are the ones the whole window computes for it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn choose_sorted_from(
    query: &Tensor,
    key: &Tensor,
    batch: usize,
    time: usize,
    from: usize,
    neighbours: usize,
    first: usize,
    limits: &[usize],
    tie: bool,
) -> Result<(Tensor, Tensor)> {
    choose_sorted_asking(
        query,
        key,
        batch,
        time,
        from,
        neighbours,
        first,
        limits,
        tie,
        DISTANCE_CHUNK_BYTES,
    )
}

/// The byte budget for one chunk of [`choose_sorted`]'s `(batch, chunk, time, width)` f32
/// difference (claude-272): 256 MiB. The difference and its absolute value are alive together,
/// so a chunk holds about twice this. A chunk takes at least one position, so past batch 512 at
/// 2,048 × 64 one position alone exceeds it, and the `(batch · time, time)` distance, mask and
/// sort are not chunked at all. When the whole difference fits, the read takes every row at
/// once, as before.
const DISTANCE_CHUNK_BYTES: usize = 1 << 28;

/// candle 0.9.2 sizes every element-wise launch with `LaunchConfig::for_num_elems(n as u32)`
/// (`cuda_backend/mod.rs`, `:1102` for the binary map), so a tensor of 2³² elements or more
/// launches too few blocks, or none (claude-272). [`choose_sorted`] refuses to build one
/// (claude-273).
const LAUNCH_ELEMENTS: u64 = 1 << 32;

/// Whether a tensor of `dims` has fewer than [`LAUNCH_ELEMENTS`] elements.
fn launch_fits(dims: &[usize]) -> bool {
    dims.iter()
        .try_fold(1u64, |n, &d| n.checked_mul(d as u64))
        .is_some_and(|n| n < LAUNCH_ELEMENTS)
}

/// How many query positions of each window one chunk takes under `bytes`: at least one, at most
/// `time`.
fn distance_chunk(batch: usize, time: usize, width: usize, bytes: usize) -> usize {
    let per_position = batch * time * width * std::mem::size_of::<f32>();
    (bytes / per_position.max(1)).clamp(1, time.max(1))
}

/// The mean L1 between each query position of `q` `(batch, asks, 1, width)` and every position of
/// `k` `(batch, 1, time, width)`, as `(batch · asks, time)`, `chunk` query positions at a time.
/// Each distance is the same mean over the same coordinates whatever the chunk.
fn mean_l1_rows(q: &Tensor, k: &Tensor, batch: usize, time: usize, chunk: usize) -> Result<Tensor> {
    let asks = q.dim(1)?;
    let mut parts = Vec::with_capacity(asks.div_ceil(chunk));
    let mut at = 0;
    while at < asks {
        let n = chunk.min(asks - at);
        parts.push(q.narrow(1, at, n)?.broadcast_sub(k)?.abs()?.mean(3)?);
        at += n;
    }
    Ok(Tensor::cat(&parts, 1)?.reshape((batch * asks, time))?)
}

/// The read's candidate rule, `first ≤ s < t` and `s < limit`, at every position `s` for each of
/// `rows` rows that ask at positions `from..time` of windows of `time` positions (`from` is 0 for
/// whole windows), as a `(rows, time)` mask of 0 and 1. It is built on `device` from each row's
/// position in its window and its limit (claude-272; before, a host loop over every entry).
fn candidate_mask(
    rows: usize,
    time: usize,
    from: usize,
    first: usize,
    limits: &[usize],
    device: &Device,
) -> Result<Tensor> {
    let s = Tensor::arange(0u32, time as u32, device)?.reshape((1, time))?;
    let asks = time - from;
    let at: Vec<u32> = (0..rows).map(|r| (from + r % asks) as u32).collect();
    let at = Tensor::from_vec(at, (rows, 1), device)?;
    // `s` is below `time`, so a limit past it cuts nothing.
    let limit: Vec<u32> = limits.iter().map(|&l| l.min(time) as u32).collect();
    let limit = Tensor::from_vec(limit, (rows, 1), device)?;
    let from = Tensor::from_vec(vec![first.min(time) as u32], (1, 1), device)?;
    Ok(s.broadcast_ge(&from)?
        .broadcast_mul(&s.broadcast_lt(&at)?)?
        .broadcast_mul(&s.broadcast_lt(&limit)?)?)
}

/// [`choose_sorted`], its distance taken in chunks of at most `chunk_bytes`.
#[allow(clippy::too_many_arguments)]
fn choose_sorted_in(
    query: &Tensor,
    key: &Tensor,
    batch: usize,
    time: usize,
    neighbours: usize,
    first: usize,
    limits: &[usize],
    tie: bool,
    chunk_bytes: usize,
) -> Result<(Tensor, Tensor)> {
    choose_sorted_asking(
        query,
        key,
        batch,
        time,
        0,
        neighbours,
        first,
        limits,
        tie,
        chunk_bytes,
    )
}

/// [`choose_sorted_in`] for the query positions `from..time` of each window ([`choose_sorted_from`]).
#[allow(clippy::too_many_arguments)]
fn choose_sorted_asking(
    query: &Tensor,
    key: &Tensor,
    batch: usize,
    time: usize,
    from: usize,
    neighbours: usize,
    first: usize,
    limits: &[usize],
    tie: bool,
    chunk_bytes: usize,
) -> Result<(Tensor, Tensor)> {
    if from >= time {
        return Err(invalid(format!(
            "the read asks from position {from} of a window of {time}"
        )));
    }
    let device = query.device();
    let kept = neighbours.min(time.saturating_sub(1)).max(1);
    let width = query.dim(1)?;
    let asks = time - from;
    let rows = batch * asks;
    let q = query.detach().reshape((batch, asks, 1, width))?;
    let k = key.detach().reshape((batch, 1, time, width))?;
    // At most `asks` positions per chunk: `distance_chunk` caps at `time`, so a whole window's
    // chunk is unchanged.
    let chunk = distance_chunk(batch, time, width, chunk_bytes).min(asks);
    if !launch_fits(&[rows, time]) || !launch_fits(&[batch, chunk, time, width]) {
        return Err(invalid(format!(
            "the read over {batch} windows of {time} positions builds a tensor of 2^32 elements \
             or more, past candle's 32-bit launch size"
        )));
    }
    let distance = mean_l1_rows(&q, &k, batch, time, chunk)?;
    // Between ±1 codes the mean of `|q − k|` is `2h / width`, so `h` is that times `width / 2`,
    // rounded off the codes' last-bit noise.
    let distance = if tie {
        if !tie_keys_exact(width, time) {
            return Err(invalid(format!(
                "the read's tie keys for {width} coordinates over {time} positions pass 2^24"
            )));
        }
        let positions: Vec<f32> = (0..time).map(|s| tie_key(0, s, time)).collect();
        distance
            .affine(width as f64 / 2.0, 0.0)?
            .round()?
            .affine(time as f64, 0.0)?
            .broadcast_add(&Tensor::from_vec(positions, (1, time), device)?)?
    } else {
        distance
    };
    let earlier = candidate_mask(rows, time, from, first, limits, device)?;
    let far = Tensor::full(FAR, (rows, time), device)?;
    let masked = earlier.where_cond(&distance, &far)?.contiguous()?;
    let order = masked
        .arg_sort_last_dim(true)?
        .narrow(1, 0, kept)?
        .contiguous()?;
    let real = masked.gather(&order, 1)?.lt(FAR / 2.0)?;
    Ok((order, real))
}

/// The tie rule's sort key for a candidate at bit distance `bits` and window position `s`, in a
/// window of `time` positions: `bits · time + s`, which orders as `(bits, s)`.
fn tie_key(bits: usize, s: usize, time: usize) -> f32 {
    (bits * time + s) as f32
}

/// Whether every tie key of a head of `width` coordinates over `time` positions is an exact
/// `f32`: the largest, `width · time + time − 1`, must be below `2^24`.
fn tie_keys_exact(width: usize, time: usize) -> bool {
    (width + 1) * time <= 1 << 24
}

/// The Hamming distance between ±1 codes, `(a − b)² / 4` summed over the last dimension: the
/// popcount of the XOR forward. Its gradient is zero on agreeing bits and pulls a disagreeing bit
/// toward the other code.
pub(crate) fn hamming_bits(a: &Tensor, b: &Tensor) -> Result<Tensor> {
    let last = b.rank() - 1;
    Ok(a.broadcast_sub(b)?.sqr()?.sum(last)?.affine(0.25, 0.0)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// claude-273: a tensor fits candle's launch below 2³² elements and not at it. The read's
    /// `(rows, time)` tensors at 2,048 × 16 are 2²⁶; at batch 1,024 they reach 2³². An overflowing
    /// product never fits.
    #[test]
    fn a_launch_fits_below_two_to_the_thirty_two_elements() {
        assert!(launch_fits(&[65_535, 65_537]), "2^32 − 1");
        assert!(!launch_fits(&[65_536, 65_536]), "2^32");
        assert!(!launch_fits(&[16, 2_048, 2_048, 64]), "claude-272's unchunked difference");
        assert!(launch_fits(&[16 * 2_048, 2_048]), "2^26");
        assert!(!launch_fits(&[1_024 * 2_048, 2_048]), "2^32 at batch 1,024");
        assert!(!launch_fits(&[usize::MAX, 2]), "an overflowing product");
    }

    /// claude-267: `.`, `\n` and added tokens end a clause, and so does an unassigned id; a tab
    /// and plain text do not.
    #[test]
    fn slot_boundaries_are_sentence_ends_newlines_added_and_unassigned_ids() {
        let json = r#"{"added_tokens":[{"id":0,"content":"<|bos|>"}],
            "model":{"vocab":{"a":1,"Ċ":2,"b.":3,"ĉ":4,"?":5,"z":7,"!":8,"č":9}}}"#;
        let flags = slot_boundary_flags(json.as_bytes()).unwrap();
        assert_eq!(flags, [true, false, true, true, false, true, true, false, true, true]);
    }

    /// claude-267: the atom resets at a boundary token, which is its own atom.
    #[test]
    fn slot_atoms_reset_at_boundaries() {
        // 0 is a boundary; 5, 7, 6 rank 1, 2, 3.
        let table = [0, 0, 0, 0, 0, 1, 3, 2];
        assert_eq!(slot_atoms(&[5, 6, 0, 6, 7], &table), [5, 5, 0, 6, 7]);
    }

    /// claude-267: ranks rise with TRAIN count, the lower id first on a tie, and an unseen id
    /// ranks rarest; a TRAIN token outside the vocabulary is refused.
    #[test]
    fn slot_ranks_break_ties_by_token_id() {
        let boundary = [true, false, false, false, false];
        let table = slot_rank_table(&[1, 2, 2, 3, 3], 5, &boundary).unwrap();
        assert_eq!(table, [0, 2, 3, 4, 1]);
        assert!(slot_rank_table(&[5], 5, &boundary).is_err());
    }

    /// claude-267: "my friend's name is" and "my dog's name is" key on different atoms when
    /// friend and dog are rarer than name.
    #[test]
    fn slot_atoms_tell_friend_from_dog() {
        let (my, friend, dog, s, name, is) = (1u16, 2, 3, 4, 5, 6);
        let mut train = vec![my; 10];
        train.extend([s; 10]);
        train.extend([is; 10]);
        train.extend([name; 5]);
        train.extend([friend, dog]);
        let boundary = [true, false, false, false, false, false, false];
        let table = slot_rank_table(&train, 7, &boundary).unwrap();
        let ids = |v: [u16; 5]| v.map(u32::from);
        let a = slot_atoms(&ids([my, friend, s, name, is]), &table);
        let b = slot_atoms(&ids([my, dog, s, name, is]), &table);
        assert_eq!((a[4], b[4]), (u32::from(friend), u32::from(dog)));
    }

    /// claude-267 diagnostics: the atom pair keeps the clause's two rarest distinct tokens,
    /// smaller id first, and resets at a boundary; the window atom forgets what left its window.
    #[test]
    fn atom_pairs_and_window_atoms_follow_their_rules() {
        // 0 is a boundary; 5, 7, 6 rank 1, 2, 3.
        let table = [0, 0, 0, 0, 0, 1, 3, 2];
        assert_eq!(
            slot_atom_pairs(&[6, 6, 7, 5, 0, 6], &table),
            [(6, 6), (6, 6), (6, 7), (5, 7), (0, 0), (6, 6)]
        );
        assert_eq!(window_atoms(&[5, 6, 6, 7, 0, 0], &table, 2), [5, 5, 6, 7, 7, 0]);
    }

    /// claude-253: a forced code's head wider than its 64-bit code is refused, before any tensor
    /// op (so on the CPU), rather than shifted past 63.
    #[test]
    fn a_forced_code_wider_than_64_bits_is_refused() {
        let codes = Tensor::zeros((1, 130), DType::F32, &Device::Cpu).unwrap();
        let forced = [ForcedRowCode {
            depth: 2,
            row: 0,
            head: 0,
            key: false,
            code: 1,
        }];
        let err = force_codes(&codes, &forced, false, 65)
            .unwrap_err()
            .to_string();
        assert!(err.contains("wider than its 64-bit code"), "{err}");
    }

    /// claude-261: the token two back stays inside each window: `x_{s−2}`, then `x_{s−1}` at a
    /// window's second position, and the token itself at its first, as the integer `CopyHeads`.
    #[test]
    fn the_token_two_back_stays_inside_each_window() {
        let ids = [10u32, 11, 12, 13, 20, 21, 22, 23];
        assert_eq!(two_back(&ids, 4), vec![10, 10, 10, 11, 20, 20, 20, 21]);
    }

    /// claude-261: copy key 2 puts `first` (`x_t` in the query, `x_{s−1}` in the key) in each
    /// copy head's low half and `second` in its high half, and leaves the columns past the copy
    /// heads as `first`'s. The integer `CopyHeads::joined` packs the same way (bit `c` is
    /// coordinate `c`), which `copy_heads_match_the_f64_reference` pins on that side. CPU tensor
    /// compute is refused (D24/D29), so this runs on the GPU, at every pod's I-0.
    #[cfg(feature = "cuda")]
    #[test]
    fn pair_halves_puts_the_first_token_in_each_copy_heads_low_half() {
        let device = Device::new_cuda(0).unwrap();
        let (head_width, width) = (16, 40);
        let first = Tensor::full(1f32, (3, width), &device).unwrap();
        let second = Tensor::full(2f32, (3, width), &device).unwrap();
        // claude-268: a head on copy key 1 stays `first`'s whole, wherever it sits.
        for pair in [[true, true], [true, false], [false, true]] {
            let paired = pair_halves(&first, &second, &pair, head_width)
                .unwrap()
                .to_vec2::<f32>()
                .unwrap();
            let want: Vec<f32> = (0..width)
                .map(|c| {
                    let h = c / head_width;
                    let high = h < pair.len() && pair[h] && c % head_width >= head_width / 2;
                    if high { 2.0 } else { 1.0 }
                })
                .collect();
            for row in paired {
                assert_eq!(row, want, "{pair:?}");
            }
        }
    }

    /// claude-261: a copy key names 1 or 2 tokens, and copy key 2 halves each head on whole
    /// lanes; anything else is refused.
    #[test]
    fn a_copy_key_is_one_or_two_tokens_on_halves_of_whole_lanes() {
        let reads = || HammingReads::new(4, 256, &Device::Cpu).unwrap();
        assert_eq!(reads().copy_key(), 1);
        assert_eq!(reads().with_copy_key(2).unwrap().copy_key(), 2);
        for copy_key in [0, 3] {
            let err = reads().with_copy_key(copy_key).err().expect("refused");
            assert!(err.to_string().contains("1 or 2 tokens"), "{err}");
        }
        // Heads of 12 coordinates have halves of 6, which split a lane.
        assert!(check_copy_key(2, 4, 48).is_err());
        assert!(check_copy_key(2, 1, 64).is_err(), "one head has no copy head to key");
        assert!(check_copy_key(1, 4, 48).is_ok());
        assert!(check_copy_key(2, 4, 256).is_ok());
        // claude-268: per-head keys, one per copy head; uniform lists are the scalar key.
        let mixed = reads().with_copy_keys(&[2, 1]).unwrap();
        assert_eq!((mixed.copy_keys(), mixed.mixed_copy_keys(), mixed.copy_key()), (vec![2, 1], Some(vec![2, 1]), 1));
        let both = reads().with_copy_keys(&[2, 2]).unwrap();
        assert_eq!((both.mixed_copy_keys(), both.copy_key()), (None, 2));
        assert_eq!(reads().copy_keys(), vec![1, 1]);
        for keys in [&[2usize][..], &[2, 1, 1], &[2, 3], &[0, 1]] {
            assert!(reads().with_copy_keys(keys).is_err(), "{keys:?}");
        }
        assert!(check_copy_keys(&[1, 2], 4, 48).is_err(), "a key-2 head still needs whole-lane halves");
        assert!(check_copy_keys(&[1, 1], 4, 48).is_ok());
    }

    #[test]
    fn a_partially_spun_hamming_sidecar_is_refused() {
        let mut reads = HammingReads::new(4, 32, &Device::Cpu)
            .unwrap()
            .with_type_spin()
            .unwrap();
        assert!(reads.type_spin().unwrap());
        reads.reads[0].step_rotation = None;
        assert!(reads
            .type_spin()
            .unwrap_err()
            .to_string()
            .contains("only some depths"));
    }

    /// claude-261: `hamming_read.safetensors` records the copy key only when it is 2, so a
    /// one-token read's file is as before, and a two-token read loads back with its key.
    #[test]
    fn the_copy_key_is_saved_only_when_two() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "hamming-copy-key-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        for copy_key in [1, 2] {
            let dir = root.join(copy_key.to_string());
            std::fs::create_dir_all(&dir)?;
            HammingReads::new(4, 256, &Device::Cpu)?
                .with_copy_key(copy_key)?
                .save(&dir)?;
            let tensors = candle_core::safetensors::load(dir.join(FILE), &Device::Cpu)?;
            assert_eq!(tensors.contains_key("copy_key"), copy_key == 2);
            let loaded = HammingReads::load(&dir, &Device::Cpu)?.expect("saved reads");
            assert_eq!(loaded.copy_key(), copy_key);
            assert!(!tensors.contains_key("copy_keys"), "uniform keys record no list");
        }
        // claude-268: mixed keys record their list alone, and load back with it.
        for keys in [[2usize, 1], [1, 2]] {
            let dir = root.join(format!("mixed-{}{}", keys[0], keys[1]));
            std::fs::create_dir_all(&dir)?;
            HammingReads::new(4, 256, &Device::Cpu)?
                .with_copy_keys(&keys)?
                .save(&dir)?;
            let tensors = candle_core::safetensors::load(dir.join(FILE), &Device::Cpu)?;
            assert!(!tensors.contains_key("copy_key"), "{keys:?}");
            assert_eq!(tensors["copy_keys"].to_vec1::<u32>()?, keys.map(|k| k as u32).to_vec());
            let loaded = HammingReads::load(&dir, &Device::Cpu)?.expect("saved reads");
            assert_eq!(loaded.copy_keys(), keys.to_vec());
        }
        // A saved list that names one key for every head is refused: that key is `copy_key`.
        let dir = root.join("mixed-12");
        let mut tensors = candle_core::safetensors::load(dir.join(FILE), &Device::Cpu)?;
        tensors.insert("copy_keys".to_owned(), Tensor::new(&[2u32, 2], &Device::Cpu)?);
        candle_core::safetensors::save(&tensors, dir.join(FILE))?;
        let same = HammingReads::load(&dir, &Device::Cpu).err().map(|e| e.to_string());
        assert!(same.is_some_and(|e| e.contains("all the same")), "an all-equal list is refused");
        std::fs::remove_dir_all(&root)?;
        Ok(())
    }

    /// claude-235: the tie key orders candidates by bit distance and then by position, every key
    /// of a 64-coordinate head over the 192-token window is an exact `f32`, and a window past the
    /// bound is refused.
    #[test]
    fn the_tie_key_orders_by_bits_then_position_and_is_exact() {
        let (width, time) = (64, 192);
        assert!(tie_keys_exact(width, time));
        let mut previous = -1.0f32;
        for bits in 0..=width {
            for s in 0..time {
                let key = tie_key(bits, s, time);
                assert!(key > previous, "({bits}, {s})");
                assert_eq!(key as usize, bits * time + s, "({bits}, {s}) is not exact");
                previous = key;
            }
        }
        assert!(!tie_keys_exact(width, 1 << 18));
        assert!(tie_keys_exact(width, (1 << 24) / (width + 1)));
        assert!(!tie_keys_exact(width, (1 << 24) / (width + 1) + 1));
    }

    /// claude-235: a head's trace packs each row's codes into one word and keeps only the real
    /// picks, in the order chosen; a head wider than one word and a row without its choice are
    /// refused.
    #[test]
    fn a_head_trace_packs_the_codes_and_keeps_only_real_picks() {
        let row = |ones: &[usize]| -> Vec<f32> {
            (0..64).map(|i| if ones.contains(&i) { 1.0 } else { -1.0 }).collect()
        };
        let query = vec![row(&[0]), row(&[1, 63]), row(&[])];
        let key = vec![row(&[2]), row(&[]), row(&[5])];
        let picks = vec![vec![0, 0], vec![1, 0], vec![2, 1]];
        let real = vec![vec![0, 0], vec![1, 0], vec![1, 1]];
        let head = trace_head(&query, &key, &picks, &real, None, (&[], &[])).unwrap();
        assert_eq!(head.query, vec![1, 0b10 | (1 << 63), 0]);
        assert_eq!(head.key, vec![1 << 2, 0, 1 << 5]);
        assert_eq!(head.kept, vec![vec![], vec![1], vec![2, 1]]);
        // claude-251: without coordinates the head is what it was, and they are not written.
        assert!(head.query_f32.is_empty() && head.key_f32.is_empty());
        assert!(!serde_json::to_string(&head).unwrap().contains("_f32"));
        let wide = vec![vec![1.0f32; 65]; 3];
        assert!(trace_head(&wide, &wide, &picks, &real, None, (&[], &[])).is_err());
        assert!(trace_head(&query, &key, &picks[..2], &real, None, (&[], &[])).is_err());
        assert!(trace_head(
            &query,
            &key,
            &picks,
            &[vec![1], vec![1, 0], vec![1, 1]],
            None,
            (&[], &[])
        )
        .is_err());
    }

    /// claude-251: with coordinates, each row's query and key coordinates before the sign are
    /// kept bit for bit, beside the words and kept sets its ±1 codes give; coordinates that do
    /// not sign to their row's code are refused.
    #[test]
    fn a_head_trace_with_coordinates_keeps_them_bit_for_bit() {
        let query = vec![vec![0.25f32, -0.0, 3e-38], vec![-1.5, 0.0, f32::MIN_POSITIVE]];
        let key = vec![vec![1.0f32, -2.0, 0.5], vec![0.0, -0.0, -7.25]];
        // The codes the read signs them to: ±1 by `p ≥ 0`, so a 0 of either sign is a 1.
        let code = |rows: &[Vec<f32>]| -> Vec<Vec<f32>> {
            rows.iter().map(|r| r.iter().map(|&v| if v >= 0.0 { 1.0 } else { -1.0 }).collect()).collect()
        };
        let (query_code, key_code) = (code(&query), code(&key));
        let picks = vec![vec![0], vec![1]];
        let real = vec![vec![0], vec![1]];
        let plain = trace_head(&query_code, &key_code, &picks, &real, None, (&[], &[])).unwrap();
        let head = trace_head(
            &query_code,
            &key_code,
            &picks,
            &real,
            Some((&query, &key)),
            (&[], &[]),
        )
        .unwrap();
        assert_eq!(
            (&head.query, &head.key, &head.kept),
            (&plain.query, &plain.key, &plain.kept)
        );
        assert_eq!(
            (head.query.as_slice(), head.key.as_slice()),
            (&[0b111, 0b110][..], &[0b101, 0b011][..])
        );
        let bits = |rows: &[Vec<f32>]| -> Vec<Vec<u32>> {
            rows.iter().map(|r| r.iter().map(|v| v.to_bits()).collect()).collect()
        };
        assert_eq!(head.query_f32, bits(&query));
        assert_eq!(head.key_f32, bits(&key));
        // −0.0 keeps its sign bit, which the code reads as a 0 coordinate's 1.
        assert_eq!(head.query_f32[0][1], (-0.0f32).to_bits());
        let back: HammingTraceHead =
            serde_json::from_str(&serde_json::to_string(&head).unwrap()).unwrap();
        assert_eq!(back, head);
        // Coordinates of another sign than their code, or for too few rows, are refused.
        let flipped = vec![query[1].clone(), query[0].clone()];
        assert!(trace_head(
            &query_code,
            &key_code,
            &picks,
            &real,
            Some((&flipped, &key)),
            (&[], &[])
        )
        .is_err());
        assert!(trace_head(
            &query_code,
            &key_code,
            &picks,
            &real,
            Some((&query[..1], &key)),
            (&[], &[])
        )
        .is_err());
        // claude-253: where a row's code was forced, its coordinates stay as computed and need
        // not sign to it; the exemption is that row's and that role's alone.
        let forced = trace_head(
            &query_code,
            &key_code,
            &picks,
            &real,
            Some((&flipped, &key)),
            (&[0, 1], &[]),
        )
        .unwrap();
        assert_eq!(
            (forced.query.clone(), forced.query_f32),
            (plain.query.clone(), bits(&flipped))
        );
        assert!(trace_head(
            &query_code,
            &key_code,
            &picks,
            &real,
            Some((&flipped, &key)),
            (&[0], &[])
        )
        .is_err());
        assert!(trace_head(
            &query_code,
            &key_code,
            &picks,
            &real,
            Some((&flipped, &key)),
            (&[], &[0, 1])
        )
        .is_err());
    }

    /// claude-235: a stream read opens one window per batch row, the readout read joins them,
    /// and a readout read with no stream read on the same ids before it is refused.
    #[test]
    fn the_readout_read_joins_the_windows_its_stream_read_opened() {
        let read = |depth: usize| HammingTraceRead {
            depth,
            heads: Vec::new(),
        };
        let ids = [7u32, 8, 9, 10, 11, 12];
        let mut windows = Vec::new();
        assert!(push_trace_reads(&mut windows, &ids, 3, vec![read(6), read(6)], true).is_err());
        push_trace_reads(&mut windows, &ids, 3, vec![read(2), read(2)], false).unwrap();
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[1].ids, vec![10, 11, 12]);
        let other = [7u32, 8, 9, 10, 11, 0];
        assert!(push_trace_reads(&mut windows, &other, 3, vec![read(6), read(6)], true).is_err());
        push_trace_reads(&mut windows, &ids, 3, vec![read(6), read(6)], true).unwrap();
        let depths: Vec<Vec<usize>> = windows
            .iter()
            .map(|w| w.reads.iter().map(|r| r.depth).collect())
            .collect();
        assert_eq!(depths, vec![vec![2, 6], vec![2, 6]]);
        // A window has one stream read and one readout read, never a second readout.
        assert!(push_trace_reads(&mut windows, &ids, 3, vec![read(6), read(6)], true).is_err());
        assert!(push_trace_reads(&mut windows, &ids, 4, vec![read(2)], false).is_err());
    }

    /// claude-235, claude-239: on a new thread the trace starts off and the tie rule on; the
    /// trace is taken and stays on empty, turning it off drops it, and the rule turns off and on.
    #[test]
    fn the_trace_starts_off_and_the_tie_rule_on() {
        std::thread::spawn(|| {
            assert!(hamming_tie_rule());
            assert_eq!(take_hamming_trace(), None);
            set_hamming_trace(true);
            assert_eq!(take_hamming_trace(), Some(Vec::new()));
            assert_eq!(take_hamming_trace(), Some(Vec::new()));
            set_hamming_trace(false);
            assert_eq!(take_hamming_trace(), None);
            set_hamming_tie_rule(false);
            assert!(!hamming_tie_rule());
            set_hamming_tie_rule(true);
            assert!(hamming_tie_rule());
        })
        .join()
        .unwrap();
    }

    #[test]
    fn candidates_include_the_immediate_predecessor_and_exclude_the_current_row() {
        // Five stream positions put the current row at 4. Position 3 is the immediately
        // preceding payload, position 0 has no predecessor, and current row 4 is excluded.
        assert!(!is_candidate(0, 4, 1, usize::MAX));
        assert!(is_candidate(1, 4, 1, usize::MAX));
        assert!(is_candidate(3, 4, 1, usize::MAX));
        assert!(!is_candidate(4, 4, 1, usize::MAX));
    }

    /// claude-235 (its GPU half): on ±1 codes with planted ties, the tie rule keeps, in order,
    /// the positions the integer session keeps: ascending bit distance, then ascending `s`.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_tie_rule_keeps_the_integer_sessions_order() {
        let device = Device::new_cuda(0).unwrap();
        let (time, width, kept) = (40, 64, HAMMING_KEPT);
        // Few distinct codes, so many candidates tie at every distance. claude-239: each
        // coordinate is ±1 only to its last bit, as the read's straight-through `p + (bits − p)`
        // can leave it, so the rule runs on noisy codes. At this noise no key moves by ½, so
        // this does not pin `.round()`.
        let noise = [1.0 - f32::EPSILON / 2.0, 1.0, 1.0 + f32::EPSILON];
        let code = |r: usize| -> Vec<f32> {
            let pattern = (r * 7) % 5;
            (0..width)
                .map(|i| {
                    let positive = (i * 3 + pattern) % 8 < 2 + pattern;
                    noise[(r * 31 + i) % 3] * if positive { 1.0 } else { -1.0 }
                })
                .collect()
        };
        let rows: Vec<Vec<f32>> = (0..time).map(code).collect();
        let flat: Vec<f32> = rows.iter().flatten().copied().collect();
        let points = Tensor::from_vec(flat, (time, width), &device).unwrap();
        let (picks, real) =
            choose_sorted(&points, &points, 1, time, kept, 1, &vec![time; time], true).unwrap();
        let (picks, real) = (picks.to_vec2::<u32>().unwrap(), real.to_vec2::<u8>().unwrap());
        let words: Vec<u64> = rows.iter().map(|r| pack_signs(r)[0]).collect();
        let mut tied = 0;
        for t in 0..time {
            let mut expected: Vec<(u32, usize)> = (1..t)
                .map(|s| ((words[t] ^ words[s]).count_ones(), s))
                .collect();
            expected.sort_unstable();
            expected.truncate(kept);
            tied += expected.windows(2).filter(|w| w[0].0 == w[1].0).count();
            let chosen: Vec<usize> = picks[t]
                .iter()
                .zip(&real[t])
                .filter(|(_, &ok)| ok == 1)
                .map(|(&s, _)| s as usize)
                .collect();
            let expected: Vec<usize> = expected.into_iter().map(|(_, s)| s).collect();
            assert_eq!(chosen, expected, "row {t}");
        }
        assert!(tied > 100, "the planted codes tie too rarely: {tied}");
    }

    /// claude-239 (claude-235's owed GPU test): a real read, at the default, keeps in every head
    /// the positions the integer session keeps. Each traced kept set is, in order, the nearest 16
    /// by ascending bit distance and then ascending `s`, computed here from the traced codes. The
    /// tokens repeat, so ties cross the 16th place. With the rule turned off, the same read keeps
    /// sets just as near but breaks some of those ties otherwise, so the test can fail.
    #[cfg(feature = "cuda")]
    #[test]
    fn a_read_breaks_its_ties_by_the_integer_sessions_rule_by_default() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let (batch, time) = (2, 64);
        let x = residual(batch, time, 256, &device);
        // Three tokens, so every key code is one of three and a row's distances take few values.
        let ids: Vec<u32> = (0..batch * time).map(|i| (i % 3) as u32).collect();
        let table = table(3, 256, &device);
        let trace = || {
            set_hamming_trace(true);
            reads.reads[0].apply(&x, &ids, &table).unwrap();
            let windows = take_hamming_trace().unwrap();
            set_hamming_trace(false);
            windows
        };
        // Each test runs on its own thread, so the rule is at its default here.
        let on = trace();
        set_hamming_tie_rule(false);
        let off = trace();
        set_hamming_tie_rule(true);
        assert_eq!((on.len(), off.len()), (batch, batch));
        let (mut crossing, mut broken) = (0, 0);
        for (window, other) in on.iter().zip(&off) {
            for (head, other) in window.reads[0].heads.iter().zip(&other.reads[0].heads) {
                assert_eq!((&head.query, &head.key), (&other.query, &other.key));
                for t in 0..time {
                    let bits = |s: u32| (head.query[t] ^ head.key[s as usize]).count_ones();
                    let mut nearest: Vec<(u32, u32)> =
                        (1..t as u32).map(|s| (bits(s), s)).collect();
                    nearest.sort_unstable();
                    let kept = nearest.len().min(HAMMING_KEPT);
                    crossing += usize::from(
                        nearest
                            .get(kept)
                            .is_some_and(|next| next.0 == nearest[kept - 1].0),
                    );
                    let expected: Vec<u32> = nearest[..kept].iter().map(|n| n.1).collect();
                    assert_eq!(head.kept[t], expected, "on, window row {t}");
                    // Off, the set is as near, and only its ties may break otherwise.
                    let mut near: Vec<u32> = other.kept[t].iter().map(|&s| bits(s)).collect();
                    near.sort_unstable();
                    let nearest_bits: Vec<u32> = nearest[..kept].iter().map(|n| n.0).collect();
                    assert_eq!(near, nearest_bits, "off, window row {t}");
                    let mut set = other.kept[t].clone();
                    set.sort_unstable();
                    let mut wanted = expected;
                    wanted.sort_unstable();
                    broken += usize::from(set != wanted);
                }
            }
        }
        println!("{crossing} rows tie across the 16th place; off, {broken} kept sets differ");
        assert!(crossing > 100, "too few ties across the 16th place: {crossing}");
        assert!(broken > 0, "off, the sort broke every tie as the rule does");
    }

    /// claude-235 (its GPU half): recording changes nothing the read computes, and the trace's
    /// kept sets are the read's real picks, at the default tie rule.
    #[cfg(feature = "cuda")]
    #[test]
    fn recording_the_trace_leaves_the_read_unchanged() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        for var in [&reads.reads[0].weight, &reads.reads[1].weight] {
            var.set(&Tensor::full(0.5f32, var.dims(), &device).unwrap()).unwrap();
        }
        let x = residual(2, 24, 256, &device);
        let ids: Vec<u32> = (0..48).map(|i| (i % 5) as u32).collect();
        let table = table(8, 256, &device);
        let plain = |out: Tensor| out.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        // Each test runs on its own thread, so here the trace starts off and the tie rule on.
        let off = plain(reads.reads[0].apply(&x, &ids, &table).unwrap());
        let off_readout = plain(reads.reads[1].apply_readout(&x, &ids, &table).unwrap());
        set_hamming_trace(true);
        let on = plain(reads.reads[0].apply(&x, &ids, &table).unwrap());
        let on_readout = plain(reads.reads[1].apply_readout(&x, &ids, &table).unwrap());
        let windows = take_hamming_trace().unwrap();
        set_hamming_trace(false);
        assert_eq!(on, off);
        assert_eq!(on_readout, off_readout);
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].ids, ids[..24].to_vec());
        let depths: Vec<usize> = windows[1].reads.iter().map(|r| r.depth).collect();
        assert_eq!(depths, HAMMING_DEPTHS.to_vec());
        assert_eq!(windows[1].reads[1].heads.len(), 4);
        for head in &windows[1].reads[1].heads {
            assert_eq!(head.query.len(), 24);
            assert!(head.kept.iter().enumerate().all(|(t, kept)| kept.len()
                == t.saturating_sub(1).min(HAMMING_KEPT)
                && kept.iter().all(|&s| s >= 1 && (s as usize) < t)));
        }
    }

    /// claude-215: the decode scope limits only the heads it names, to positions before the cut;
    /// no scope, or `Full`, limits none; a cut past the window limits nothing more.
    #[test]
    fn the_read_scope_limits_only_its_heads() {
        let (time, copy) = (12, 2);
        for head in 0..4 {
            assert_eq!(head_limit(None, head, copy, time), time);
            assert_eq!(head_limit(Some((ReadScope::Full, 5)), head, copy, time), time);
            assert_eq!(head_limit(Some((ReadScope::All, 5)), head, copy, time), 5);
            assert_eq!(head_limit(Some((ReadScope::All, 40)), head, copy, time), time);
        }
        // The copy heads are the first `copy`; the type heads keep every earlier position.
        assert_eq!(head_limit(Some((ReadScope::Copy, 5)), 0, copy, time), 5);
        assert_eq!(head_limit(Some((ReadScope::Copy, 5)), 1, copy, time), 5);
        assert_eq!(head_limit(Some((ReadScope::Copy, 5)), 2, copy, time), time);
        assert_eq!(head_limit(Some((ReadScope::Copy, 5)), 3, copy, time), time);
    }

    /// claude-216: a row cut gives each row its own limit, for the heads its scope limits, and
    /// never past the window; with no cut every row keeps every earlier position; a decode cut
    /// gives every row the same limit.
    #[test]
    fn a_row_cut_limits_each_row_for_its_heads() {
        let (time, copy, rows) = (6, 2, 4);
        let limits = (ReadScope::All, vec![6, 3, 9, 1]);
        assert_eq!(row_limits(None, None, 0, copy, time, rows), vec![6; 4]);
        assert_eq!(row_limits(None, Some(&limits), 3, copy, time, rows), vec![6, 3, 6, 1]);
        let copy_only = (ReadScope::Copy, vec![6, 3, 9, 1]);
        assert_eq!(row_limits(None, Some(&copy_only), 1, copy, time, rows), vec![6, 3, 6, 1]);
        assert_eq!(row_limits(None, Some(&copy_only), 2, copy, time, rows), vec![6; 4], "a type head");
        assert_eq!(row_limits(Some((ReadScope::All, 2)), None, 3, copy, time, rows), vec![2; 4]);
        assert!(head_limited(ReadScope::All, 3, copy) && !head_limited(ReadScope::Full, 0, copy));
    }

    /// claude-215: a candidate is from `first`, earlier than its position, and before the limit;
    /// a limit of 1 or less leaves a window with no candidate at all.
    #[test]
    fn a_candidate_is_earlier_and_before_the_cut() {
        assert!(is_candidate(1, 5, 1, 12));
        assert!(!is_candidate(0, 5, 1, 12), "before first");
        assert!(!is_candidate(5, 5, 1, 12), "not earlier");
        assert!(is_candidate(3, 9, 1, 4));
        assert!(!is_candidate(4, 9, 1, 4), "at the cut: the reply's first token");
        assert!(!is_candidate(7, 9, 1, 4), "in the reply");
        assert!((0..12).all(|s| (0..12).all(|t| !is_candidate(s, t, 1, 1))));
        assert_eq!(ReadScope::Copy.name(), "copy");
        assert_eq!(ReadScope::All.name(), "all");
    }

    /// claude-215 (its GPU half): with a limit, every real pick is before it, and the rows whose
    /// candidates all lie before it keep claude-144's nearest set; with the limit at the window's
    /// length, the choice is unchanged.
    #[cfg(feature = "cuda")]
    #[test]
    fn a_cut_keeps_every_pick_before_it() {
        let device = Device::new_cuda(0).unwrap();
        let (batch, time, width, cut) = (1, 20, 8, 7);
        let values: Vec<f32> = (0..batch * time * width)
            .map(|i| (((i * 2_654_435_761) % 10_007) as f32) / 997.0)
            .collect();
        let points = Tensor::from_vec(values, (batch * time, width), &device).unwrap();
        let (open, open_real) = choose_sorted(&points, &points, batch, time, 4, 1, &vec![time; batch * time], false).unwrap();
        let (cut_picks, cut_real) = choose_sorted(&points, &points, batch, time, 4, 1, &vec![cut; batch * time], false).unwrap();
        let rows = |p: Tensor, r: Tensor| -> Vec<Vec<u32>> {
            let (p, r) = (p.to_vec2::<u32>().unwrap(), r.to_vec2::<u8>().unwrap());
            p.into_iter()
                .zip(r)
                .map(|(row, ok)| {
                    let mut kept: Vec<u32> =
                        row.into_iter().zip(ok).filter(|(_, o)| *o == 1).map(|(s, _)| s).collect();
                    kept.sort_unstable();
                    kept
                })
                .collect()
        };
        let (open, limited) = (rows(open, open_real), rows(cut_picks, cut_real));
        let mut picked = 0;
        for (t, (o, l)) in open.iter().zip(&limited).enumerate() {
            assert!(l.iter().all(|&s| (s as usize) < cut), "row {t}: {l:?}");
            picked += l.len();
            // Up to the cut, every candidate is before it, so the two choices agree.
            if t <= cut {
                assert_eq!(o, l, "row {t}");
            }
        }
        assert!(picked > 0, "the cut left no pick at all");
        assert!(open.iter().any(|o| o.iter().any(|&s| s as usize >= cut)), "nothing to cut");
        // claude-216: a limit per row, as training's row cut gives; every pick is before its own
        // row's limit.
        let per_row: Vec<usize> = (0..batch * time).map(|r| 2 + r % 9).collect();
        let (row_picks, row_real) = choose_sorted(&points, &points, batch, time, 4, 1, &per_row, false).unwrap();
        for (r, picks) in rows(row_picks, row_real).iter().enumerate() {
            assert!(picks.iter().all(|&s| (s as usize) < per_row[r]), "row {r}: {picks:?}");
        }
    }

    /// claude-272: the distance taken in chunks of any size is the whole read's distance, bit for
    /// bit, and the one-chunk distance is the expression the read used before; so the picks
    /// agree too, with the tie rule on and off, a cut, row limits, and either first position.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_read_in_chunks_chooses_as_the_whole_read() {
        let device = Device::new_cuda(0).unwrap();
        let (batch, time, width) = (2, 20, 8);
        let values = |salt: usize| -> Vec<f32> {
            (0..batch * time * width)
                .map(|i| ((((i + salt) * 2_654_435_761) % 10_007) as f32) / 997.0)
                .collect()
        };
        let signs = |v: Vec<f32>| -> Vec<f32> {
            v.into_iter().map(|x| if x < 5.0 { -1.0 } else { 1.0 }).collect()
        };
        let bits = |t: &Tensor| -> Vec<u32> {
            let flat = t.flatten_all().unwrap().to_vec1::<f32>().unwrap();
            flat.into_iter().map(f32::to_bits).collect()
        };
        let q4 = |v: &Tensor| v.reshape((batch, time, 1, width)).unwrap();
        let k4 = |v: &Tensor| v.reshape((batch, 1, time, width)).unwrap();
        let per_position = batch * time * width * std::mem::size_of::<f32>();
        let limit_sets: [Vec<usize>; 3] = [
            vec![time; batch * time],
            vec![7; batch * time],
            (0..batch * time).map(|r| 2 + r % 9).collect(),
        ];
        for tie in [false, true] {
            let (query, key) = if tie {
                (signs(values(0)), signs(values(3)))
            } else {
                (values(0), values(3))
            };
            let query = Tensor::from_vec(query, (batch * time, width), &device).unwrap();
            let key = Tensor::from_vec(key, (batch * time, width), &device).unwrap();
            let whole = q4(&query)
                .broadcast_sub(&k4(&key))
                .unwrap()
                .abs()
                .unwrap()
                .mean(3)
                .unwrap();
            for chunk in [1, 7, time] {
                assert_eq!(distance_chunk(batch, time, width, chunk * per_position), chunk);
                let chunked = mean_l1_rows(&q4(&query), &k4(&key), batch, time, chunk).unwrap();
                assert_eq!(bits(&chunked), bits(&whole), "chunk {chunk}");
                for first in [0, 1] {
                    for limits in &limit_sets {
                        let bytes = chunk * per_position;
                        let (picks, real) =
                            choose_sorted_in(&query, &key, batch, time, 4, first, limits, tie, bytes)
                                .unwrap();
                        let (whole_picks, whole_real) =
                            choose_sorted_in(&query, &key, batch, time, 4, first, limits, tie, usize::MAX)
                                .unwrap();
                        let case = format!("chunk {chunk}, first {first}, tie {tie}");
                        assert_eq!(
                            picks.to_vec2::<u32>().unwrap(),
                            whole_picks.to_vec2::<u32>().unwrap(),
                            "{case}"
                        );
                        assert_eq!(
                            real.to_vec2::<u8>().unwrap(),
                            whole_real.to_vec2::<u8>().unwrap(),
                            "{case}"
                        );
                    }
                }
            }
        }
    }

    /// claude-272: the device mask is `is_candidate` at every entry, for limits below, at and past
    /// the window, and first positions 0 to 2.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_device_mask_is_the_candidate_rule() {
        let device = Device::new_cuda(0).unwrap();
        let (batch, time) = (3, 12);
        let rows = batch * time;
        let limit_sets: [Vec<usize>; 4] = [
            vec![time; rows],
            vec![5; rows],
            (0..rows).map(|r| r % 14).collect(),
            vec![usize::MAX; rows],
        ];
        for first in [0, 1, 2] {
            for limits in &limit_sets {
                let mask = candidate_mask(rows, time, 0, first, limits, &device).unwrap();
                for (r, row) in mask.to_vec2::<u8>().unwrap().iter().enumerate() {
                    for (s, &m) in row.iter().enumerate() {
                        let rule = u8::from(is_candidate(s, r % time, first, limits[r]));
                        assert_eq!(m, rule, "row {r}, s {s}, first {first}");
                    }
                }
            }
        }
    }

    /// claude-272, the GPU half: the read at windows of 192 to 2,048 positions, at the training
    /// batch of 4 and the scoring batch of 16, a head's width of 64 and `HAMMING_KEPT` picks under
    /// the tie rule. It times the chunked read, and the whole read where it fits, and checks that
    /// they choose alike. Run on a pod with `--ignored --nocapture`: one line per case, with the
    /// wall clock at its start and end for the memory samples beside it, and the peak bytes in use
    /// in the device's memory pool, which candle allocates from, over each form's six runs.
    #[cfg(feature = "cuda")]
    #[test]
    #[ignore = "a GPU timing run, on a pod"]
    fn the_read_at_long_windows() {
        use cudarc::driver::sys;
        let device = Device::new_cuda(0).unwrap();
        let width = 64;
        let now = || {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        };
        let Device::Cuda(cuda) = &device else { unreachable!("a CUDA device") };
        let mut pool: sys::CUmemoryPool = std::ptr::null_mut();
        // SAFETY: the device handle is live, and the pool is written into a local.
        let found = unsafe {
            sys::cuDeviceGetDefaultMemPool(&mut pool, cuda.cuda_stream().context().cu_device())
        };
        assert_eq!(found, sys::CUresult::CUDA_SUCCESS);
        // The pool's high mark of bytes in use since the last call, which it then sets back to 0.
        let peak = || -> u64 {
            let used = sys::CUmemPool_attribute::CU_MEMPOOL_ATTR_USED_MEM_HIGH;
            let (mut high, mut zero) = (0u64, 0u64);
            // SAFETY: the pool is the device's own, and the attribute is one 64-bit value.
            unsafe {
                let got = sys::cuMemPoolGetAttribute(pool, used, (&mut high as *mut u64).cast());
                assert_eq!(got, sys::CUresult::CUDA_SUCCESS);
                let set = sys::cuMemPoolSetAttribute(pool, used, (&mut zero as *mut u64).cast());
                assert_eq!(set, sys::CUresult::CUDA_SUCCESS);
            }
            high
        };
        type Choice = (Vec<u128>, Vec<Vec<u32>>, Vec<Vec<u8>>, u64);
        for time in [192, 512, 1_024, 2_048] {
            for batch in [4, 16] {
                let rows = batch * time;
                let code = |salt: usize| -> Tensor {
                    let v: Vec<f32> = (0..rows * width)
                        .map(|i| if ((i + salt) * 2_654_435_761) % 10_007 < 5_003 { -1.0 } else { 1.0 })
                        .collect();
                    Tensor::from_vec(v, (rows, width), &device).unwrap()
                };
                let (query, key) = (code(0), code(11));
                let limits = vec![time; rows];
                let run = |bytes: usize| -> std::result::Result<Choice, String> {
                    device.synchronize().map_err(|e| e.to_string())?;
                    peak();
                    let mut micros = Vec::new();
                    let mut last = None;
                    for _ in 0..6 {
                        let start = std::time::Instant::now();
                        let out = choose_sorted_in(
                            &query, &key, batch, time, HAMMING_KEPT, 1, &limits, true, bytes,
                        )
                        .map_err(|e| e.to_string())?;
                        device.synchronize().map_err(|e| e.to_string())?;
                        micros.push(start.elapsed().as_micros());
                        last = Some(out);
                    }
                    // The first run warms up; the median of the other five is reported.
                    micros.remove(0);
                    micros.sort_unstable();
                    let bytes_peak = peak();
                    let (picks, real) = last.expect("six runs");
                    let picks = picks.to_vec2::<u32>().unwrap();
                    Ok((micros, picks, real.to_vec2::<u8>().unwrap(), bytes_peak))
                };
                let chunk = distance_chunk(batch, time, width, DISTANCE_CHUNK_BYTES);
                let start = now();
                let chunked = run(DISTANCE_CHUNK_BYTES).expect("the chunked read runs");
                let middle = now();
                let whole = run(usize::MAX);
                let end = now();
                let (whole_us, whole_peak, same) = match &whole {
                    Ok(w) => (
                        w.0[2].to_string(),
                        w.3.to_string(),
                        (w.1 == chunked.1 && w.2 == chunked.2).to_string(),
                    ),
                    Err(e) => {
                        let fault = format!("\"does not fit: {}\"", e.replace('"', "'"));
                        (fault, peak().to_string(), "null".into())
                    }
                };
                println!(
                    "{{\"time\":{time},\"batch\":{batch},\"chunk\":{chunk},\"chunked_us\":{},\
                     \"chunked_all_us\":{:?},\"chunked_peak_bytes\":{},\"whole_us\":{whole_us},\
                     \"whole_peak_bytes\":{whole_peak},\"same_picks\":{same},\
                     \"ms_start\":{start},\"ms_whole\":{middle},\"ms_end\":{end}}}",
                    chunked.0[2], chunked.0, chunked.3
                );
                if let Ok(w) = &whole {
                    assert_eq!(w.1, chunked.1, "picks at time {time}, batch {batch}");
                    assert_eq!(w.2, chunked.2, "real at time {time}, batch {batch}");
                }
            }
        }
    }

    #[test]
    fn packed_sign_codes_pack_one_bit_per_coordinate_in_order() {
        let mut coordinates = vec![-1.0f32; 70];
        coordinates[0] = 0.5;
        coordinates[63] = 0.0;
        coordinates[65] = 2.0;
        assert_eq!(pack_signs(&coordinates), vec![1 | (1 << 63), 1 << 1]);
    }

    /// claude-237: the copy-code check's CPU float control in the integer crate is this crate's
    /// own, bit for bit: `hamilton_gate_f32` against `hamilton_gate_fwd_cpu` and `pack_signs_f32`
    /// against `pack_signs`, over spread values, values near 0, a tiny gate, and −0.0.
    #[test]
    fn the_copy_code_control_is_the_trainers_own_bit_for_bit() {
        use crate::kernel_controls::hamilton_gate_fwd_cpu;
        use uor_r4_integer::fbs_snapshot::{hamilton_gate_f32, pack_signs_f32};
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            ((state >> 40) as f32 / (1u64 << 24) as f32) * 4.0 - 2.0
        };
        let mut gate: Vec<f32> = (0..512).map(|_| next()).collect();
        let mut up: Vec<f32> = (0..512).map(|_| next()).collect();
        gate[..4].copy_from_slice(&[1e-7, -3e-8, 0.0, 2e-7]);
        up[4..8].copy_from_slice(&[-0.0, 1e-30, -1e-30, 0.0]);
        for (i, u) in up.iter_mut().enumerate().skip(8).step_by(7) {
            *u *= 1e-6 * i as f32;
        }
        let ours = hamilton_gate_f32(&gate, &up);
        let theirs = hamilton_gate_fwd_cpu(&gate, &up);
        assert_eq!(ours.len(), theirs.len());
        for (i, (a, b)) in ours.iter().zip(&theirs).enumerate() {
            assert_eq!(a.to_bits(), b.to_bits(), "coordinate {i}: {a} against {b}");
        }
        assert_eq!(pack_signs_f32(&theirs), pack_signs(&theirs));
        assert_eq!(
            pack_signs_f32(&up),
            pack_signs(&up),
            "−0.0 and the tiny values"
        );
    }

    /// claude-237: the check's icosians in the integer crate are this crate's 120 roots, in the
    /// same order, bit for bit.
    #[test]
    fn the_copy_code_icosians_are_the_trainers_own() {
        let ours = uor_r4_integer::fbs_snapshot::icosian_roots_f32();
        let theirs = crate::hamilton_sheet::h4_unit_roots_f32();
        for (i, (a, b)) in ours.iter().zip(&theirs).enumerate() {
            assert_eq!(a.map(f32::to_bits), b.map(f32::to_bits), "root {i}");
        }
    }

    /// claude-237: the check's snap chooses as the GPU's D11 gate hook does
    /// (`hamilton_sheet::icosian_snap_cuda_tensor`): bit for bit on spread lanes and on 0.0 and
    /// −0.0 lanes, and one of the two tied roots on four exact-tie lanes.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_copy_code_snap_is_the_gpu_hooks_own() {
        let device = Device::new_cuda(0).unwrap();
        let Device::Cuda(cuda) = &device else {
            unreachable!("a CUDA device")
        };
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            ((state >> 40) as f32 / (1u64 << 24) as f32) * 4.0 - 2.0
        };
        let mut lanes: Vec<f32> = (0..4 * 512).map(|_| next()).collect();
        lanes[..4].fill(0.0);
        lanes[4..8].fill(-0.0);
        // Lanes 2 to 5 sit midway between the identity and one of its nearest icosians: a tie in
        // exact arithmetic, which f32 rounding settles. Either side may take either root there.
        let roots = uor_r4_integer::fbs_snapshot::icosian_roots_f32();
        let near = roots[2..]
            .iter()
            .map(|root| root[0])
            .fold(f32::MIN, f32::max);
        let ties: Vec<usize> = (2..120).filter(|&k| roots[k][0] == near).take(4).collect();
        assert_eq!(ties.len(), 4);
        for (j, &k) in ties.iter().enumerate() {
            for c in 0..4 {
                lanes[8 + 4 * j + c] = roots[1][c] + roots[k][c];
            }
        }
        let gate = Tensor::from_vec(lanes.clone(), (8, 256), &device).unwrap();
        let theirs = crate::hamilton_sheet::icosian_snap_cuda_tensor(cuda, &gate)
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        let ours = uor_r4_integer::fbs_snapshot::snap_icosian_f32(&lanes);
        for lane in 0..512 {
            let (a, b) = (
                &ours[4 * lane..4 * lane + 4],
                &theirs[4 * lane..4 * lane + 4],
            );
            if let Some(&k) = lane.checked_sub(2).and_then(|j| ties.get(j)) {
                for side in [a, b] {
                    assert!(
                        side == &roots[1][..] || side == &roots[k][..],
                        "lane {lane}: {side:?}"
                    );
                }
            } else {
                assert_eq!(
                    a.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                    b.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                    "lane {lane}"
                );
            }
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_sorted_choice_keeps_the_same_nearest_set_as_claude_144s() {
        let device = Device::new_cuda(0).unwrap();
        let (batch, time, width) = (2, 20, 8);
        let values: Vec<f32> = (0..batch * time * width)
            .map(|i| (((i * 2_654_435_761) % 10_007) as f32) / 997.0)
            .collect();
        let points = Tensor::from_vec(values, (batch * time, width), &device).unwrap();
        let (local, fast_real) = choose_sorted(&points, &points, batch, time, 4, 0, &vec![time; batch * time], false).unwrap();
        // The sorted choice gives positions inside each window; claude-144's gives global rows.
        let starts: Vec<u32> = (0..batch * time)
            .map(|r| ((r / time) * time) as u32)
            .collect();
        let starts = Tensor::from_vec(starts, (batch * time, 1), &device).unwrap();
        let fast = local.broadcast_add(&starts).unwrap();
        let (slow, slow_real) =
            crate::flock_read::choose_nearest(&points, &points, batch, time, 4).unwrap();
        let sets = |picks: Tensor, real: Tensor| -> Vec<Vec<u32>> {
            let (picks, real) = (
                picks.to_vec2::<u32>().unwrap(),
                real.to_vec2::<u8>().unwrap(),
            );
            picks
                .into_iter()
                .zip(real)
                .map(|(row, ok)| {
                    let mut kept: Vec<u32> = row
                        .into_iter()
                        .zip(ok)
                        .filter(|(_, o)| *o == 1)
                        .map(|(p, _)| p)
                        .collect();
                    kept.sort_unstable();
                    kept
                })
                .collect()
        };
        assert_eq!(sets(fast, fast_real), sets(slow, slow_real));
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_forward_distance_is_the_popcount_of_the_xor_of_the_sign_codes() {
        let device = Device::new_cuda(0).unwrap();
        // Two 64-coordinate points with no zero coordinate.
        let a: Vec<f32> = (0..64).map(|i| ((i * 37 + 11) % 19) as f32 - 9.5).collect();
        let b: Vec<f32> = (0..64).map(|i| ((i * 53 + 7) % 23) as f32 - 11.5).collect();
        let code = |v: &[f32]| {
            Tensor::from_vec(v.to_vec(), (1, 64), &device)
                .unwrap()
                .sign()
                .unwrap()
        };
        let bits = hamming_bits(&code(&a), &code(&b))
            .unwrap()
            .to_vec1::<f32>()
            .unwrap()[0];
        let popcount = (pack_signs(&a)[0] ^ pack_signs(&b)[0]).count_ones();
        assert_eq!(bits, popcount as f32);
        assert!(popcount > 0 && popcount < 64);
    }

    #[cfg(feature = "cuda")]
    fn residual(batch: usize, time: usize, width: usize, device: &Device) -> Tensor {
        let values: Vec<f32> = (0..batch * time * width)
            .map(|i| (((i * 2_654_435_761) % 1_000) as f32 / 500.0) - 0.999)
            .collect();
        Tensor::from_vec(values, (batch, time, width), device).unwrap()
    }

    /// A table of `vocab` token points with independent pseudo-random signs (a splitmix64 hash),
    /// so two different tokens' codes differ in about half their bits.
    #[cfg(feature = "cuda")]
    fn table(vocab: usize, width: usize, device: &Device) -> Tensor {
        let values: Vec<f32> = (0..vocab * width)
            .map(|i| {
                let mut z = (i as u64)
                    .wrapping_add(1)
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15);
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                z ^= z >> 31;
                ((z >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
            })
            .collect();
        Tensor::from_vec(values, (vocab, width), device).unwrap()
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_zero_weight_leaves_the_residual_unchanged() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let x = residual(2, 24, 256, &device);
        let ids: Vec<u32> = (0..48).map(|i| (i % 5) as u32).collect();
        let out = reads.reads[0]
            .apply(&x, &ids, &table(8, 256, &device))
            .unwrap();
        assert_eq!(
            out.flatten_all().unwrap().to_vec1::<f32>().unwrap(),
            x.flatten_all().unwrap().to_vec1::<f32>().unwrap()
        );
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_copy_head_reads_what_followed_the_earlier_copy_of_its_token() {
        copy_case(false);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_copy_head_reads_what_followed_the_earlier_copy_of_its_token_with_the_type_spin_on() {
        // The position signal turns the type heads only, so the copy heads stay exact.
        copy_case(true);
    }

    #[cfg(feature = "cuda")]
    fn copy_case(spin: bool) {
        let device = Device::new_cuda(0).unwrap();
        let mut reads = HammingReads::new(4, 256, &device).unwrap();
        if spin {
            reads = reads.with_type_spin().unwrap();
        }
        let read = &reads.reads[0];
        // Only the copy heads' row of weights on, so the added part is theirs alone.
        let copy_only: Vec<f32> = (0..512).map(|i| f32::from(u8::from(i < 256))).collect();
        read.weight
            .set(&Tensor::from_vec(copy_only, (2, 256), &device).unwrap())
            .unwrap();
        // Learned rotations, as after training: two different unit quaternions on every lane. A
        // copy query goes through the key's rotation, so it still matches exactly.
        let turned = |q: [f32; 4]| Tensor::from_vec(q.repeat(64), (64, 4), &device).unwrap();
        read.key_rotation
            .set(&turned([0.6, 0.8, 0.0, 0.0]))
            .unwrap();
        read.query_rotation
            .set(&turned([0.8, 0.0, 0.6, 0.0]))
            .unwrap();
        // Three windows of three tokens; heads 0 and 1 are the copy heads.
        // Window 0, [5 7 5]: at t = 2 the key of s = 1 names token 5, an exact match, so the copy
        // heads bring token 7's own table point, in full and alone (s = 0 names no earlier token).
        // Window 1, [9 11 9]: the same in its own window, token 11.
        // Window 2, [3 5 5]: at t = 2 no earlier key names 5; only t itself would.
        let ids = [5u32, 7, 5, 9, 11, 9, 3, 5, 5];
        let x = residual(3, 3, 256, &device);
        let table = table(16, 256, &device);
        let points = table.to_vec2::<f32>().unwrap();
        let added = read
            .apply(&x, &ids, &table)
            .unwrap()
            .sub(&x)
            .unwrap()
            .reshape((9, 256))
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        for c in 0..256 {
            assert!((added[2][c] - points[7][c]).abs() < 1e-6, "window 0, {c}");
            assert!((added[5][c] - points[11][c]).abs() < 1e-6, "window 1, {c}");
            assert_eq!(added[8][c], 0.0, "window 2, {c}");
        }
        // Positions 0 and 1 have no candidate (1 ≤ s < t) and read nothing in any head.
        for r in [0usize, 1, 3, 4, 6, 7] {
            assert!(added[r].iter().all(|&v| v == 0.0), "row {r}");
        }
    }

    /// claude-261: two facts after the same last token, as conv-a's " is" (claude-259's trace).
    /// The window holds `B X P`, then `C X Q`, and asks at `C X`. Under one-bit margins only an
    /// exact match counts: the one-token key ties both positions after an `X` and reads the mean
    /// of `P` and `Q`; copy key 2 keeps only the position after `C X`, and reads `Q` alone.
    #[cfg(feature = "cuda")]
    #[test]
    fn copy_key_two_tells_two_facts_after_the_same_token_apart() {
        let device = Device::new_cuda(0).unwrap();
        let (b, x, p, c, q) = (2u32, 3, 4, 5, 6);
        let ids = [1u32, b, x, p, c, x, q, 7, c, x];
        let x0 = residual(1, ids.len(), 256, &device);
        let table = table(16, 256, &device);
        let points = table.to_vec2::<f32>().unwrap();
        let added = |copy_keys: &[usize]| -> Vec<f32> {
            let reads = HammingReads::new(4, 256, &device)
                .unwrap()
                .with_copy_keys(copy_keys)
                .unwrap();
            let read = &reads.reads[0];
            let copy_only: Vec<f32> = (0..512).map(|i| f32::from(u8::from(i < 256))).collect();
            read.weight
                .set(&Tensor::from_vec(copy_only, (2, 256), &device).unwrap())
                .unwrap();
            read.margin
                .set(&Tensor::new(&[1.0f32, 1.0, 32.0, 32.0], &device).unwrap())
                .unwrap();
            let turned = |q: [f32; 4]| Tensor::from_vec(q.repeat(64), (64, 4), &device).unwrap();
            read.key_rotation
                .set(&turned([0.6, 0.8, 0.0, 0.0]))
                .unwrap();
            let rows = read
                .apply(&x0, &ids, &table)
                .unwrap()
                .sub(&x0)
                .unwrap()
                .reshape((ids.len(), 256))
                .unwrap()
                .to_vec2::<f32>()
                .unwrap();
            rows[ids.len() - 1].clone()
        };
        let (one, two) = (added(&[1, 1]), added(&[2, 2]));
        // claude-268: mixed keys, the key-2 head's Q and the key-1 head's P and Q, (3Q + P) / 4.
        let mixed = [added(&[2, 1]), added(&[1, 2])];
        let (p, q) = (&points[p as usize], &points[q as usize]);
        for col in 0..256 {
            let both = (p[col] + q[col]) / 2.0;
            assert!(
                (one[col] - both).abs() < 1e-5,
                "copy key 1, {col}: {} against {both}",
                one[col]
            );
            assert!(
                (two[col] - q[col]).abs() < 1e-6,
                "copy key 2, {col}: {} against {}",
                two[col],
                q[col]
            );
            let blend = (3.0 * q[col] + p[col]) / 4.0;
            for m in &mixed {
                assert!((m[col] - blend).abs() < 1e-5, "mixed keys, {col}: {} against {blend}", m[col]);
            }
        }
    }

    /// Pseudo-random unit quaternions, `(rows, 4 · lanes)`.
    #[cfg(feature = "cuda")]
    fn unit_quaternions(rows: usize, lanes: usize, salt: usize, device: &Device) -> Tensor {
        let raw: Vec<f32> = (0..rows * lanes * 4)
            .map(|i| ((((i + salt) * 2_654_435_761) % 1_000) as f32 / 500.0) - 0.999)
            .collect();
        let mut unit = Vec::with_capacity(raw.len());
        for q in raw.chunks(4) {
            let n = q.iter().map(|v| v * v).sum::<f32>().sqrt();
            unit.extend(q.iter().map(|v| v / n));
        }
        Tensor::from_vec(unit, (rows, lanes * 4), device).unwrap()
    }

    /// The Hamilton product `a ⊗ b` of two quaternions, on the host.
    #[cfg(feature = "cuda")]
    fn product(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
        [
            a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
            a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
            a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
            a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
        ]
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_frames_are_the_running_product_of_the_steps() {
        let device = Device::new_cuda(0).unwrap();
        // Two windows of 11 positions (not a power of two), 3 lanes.
        let (batch, time, lanes) = (2, 11, 3);
        let steps = unit_quaternions(batch * time, lanes, 7, &device);
        let frames = prefix_frames(&steps, batch, time)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        let steps = steps.to_vec2::<f32>().unwrap();
        for b in 0..batch {
            for l in 0..lanes {
                // The later step multiplies from the left: F_t = u_t ⊗ F_{t−1}.
                let mut running = [1.0f32, 0.0, 0.0, 0.0];
                for t in 0..time {
                    let r = b * time + t;
                    let u: [f32; 4] = steps[r][4 * l..4 * l + 4].try_into().unwrap();
                    running = product(u, running);
                    for (c, want) in running.iter().enumerate() {
                        let got = frames[r][4 * l + c];
                        assert!(
                            (got - want).abs() < 1e-4,
                            "window {b}, lane {l}, t {t}: {got} {want}"
                        );
                    }
                }
            }
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_relative_frame_ignores_every_step_at_or_before_the_earlier_position() {
        let device = Device::new_cuda(0).unwrap();
        let (time, lanes, s, t) = (9, 2, 3, 7);
        let base = unit_quaternions(time, lanes, 11, &device);
        // The same steps but at position 1 (≤ s), and the same steps but at position 5 (in (s, t]).
        let replace = |at: usize| -> Tensor {
            let fresh = unit_quaternions(1, lanes, 999, &device);
            Tensor::cat(
                &[
                    &base.narrow(0, 0, at).unwrap(),
                    &fresh,
                    &base.narrow(0, at + 1, time - at - 1).unwrap(),
                ],
                0,
            )
            .unwrap()
        };
        let relative = |steps: &Tensor| -> Vec<f32> {
            let frames = prefix_frames(steps, 1, time).unwrap();
            let inverse = conjugate(&frames).unwrap();
            // F_t ⊗ F_s⁻¹, the frame of position t seen from position s.
            hamilton_gate(
                &frames.narrow(0, t, 1).unwrap().contiguous().unwrap(),
                &inverse.narrow(0, s, 1).unwrap().contiguous().unwrap(),
            )
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap()
        };
        let (here, early, between) = (
            relative(&base),
            relative(&replace(1)),
            relative(&replace(5)),
        );
        for (a, b) in here.iter().zip(&early) {
            assert!(
                (a - b).abs() < 1e-4,
                "a step at or before s moved the relative frame"
            );
        }
        // A step between s and t does move it: the frame is path-dependent, not an index code.
        assert!(here.iter().zip(&between).any(|(a, b)| (a - b).abs() > 1e-2));
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn every_step_is_a_unit_icosian() {
        let device = Device::new_cuda(0).unwrap();
        let points = residual(1, 8, 256, &device).reshape((8, 256)).unwrap();
        let steps = snap_icosian(&points).unwrap().to_vec2::<f32>().unwrap();
        let roots = crate::hamilton_sheet::h4_unit_roots_f32();
        for row in &steps {
            for q in row.chunks(4) {
                assert!(
                    roots
                        .iter()
                        .any(|r| r.iter().zip(q).all(|(a, b)| (a - b).abs() < 1e-6)),
                    "{q:?} is not one of the 120 unit icosians"
                );
            }
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_spin_turns_the_type_heads_and_only_them() {
        let device = Device::new_cuda(0).unwrap();
        let ids: Vec<u32> = (0..24).map(|i| (i % 5) as u32 * 3).collect();
        let x = residual(1, 24, 256, &device);
        let table = table(16, 256, &device);
        let out = |spin: bool, row: usize| -> Vec<f32> {
            let mut reads = HammingReads::new(4, 256, &device).unwrap();
            if spin {
                reads = reads.with_type_spin().unwrap();
            }
            let read = &reads.reads[0];
            let only: Vec<f32> = (0..512)
                .map(|i| f32::from(u8::from(i / 256 == row)))
                .collect();
            read.weight
                .set(&Tensor::from_vec(only, (2, 256), &device).unwrap())
                .unwrap();
            read.apply(&x, &ids, &table)
                .unwrap()
                .flatten_all()
                .unwrap()
                .to_vec1::<f32>()
                .unwrap()
        };
        // The copy heads' part is the same with the spin on and off; the type heads' part moves.
        assert_eq!(out(false, 0), out(true, 0));
        assert_ne!(out(false, 1), out(true, 1));
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_readout_moves_onto_a_found_copy_and_leaves_the_rest() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let read = reads.readout().unwrap();
        let copy_only: Vec<f32> = (0..512).map(|i| f32::from(u8::from(i < 256))).collect();
        read.weight
            .set(&Tensor::from_vec(copy_only, (2, 256), &device).unwrap())
            .unwrap();
        // The windows of the copy test: window 0 finds token 7 at t = 2, window 1 token 11,
        // window 2 nothing. With the copy weight at 1 the found rows decode exactly as the copied
        // token's point; every other row is the readout's own point, unchanged.
        let ids = [5u32, 7, 5, 9, 11, 9, 3, 5, 5];
        let n = residual(3, 3, 256, &device);
        let table = table(16, 256, &device);
        let points = table.to_vec2::<f32>().unwrap();
        let own = n.reshape((9, 256)).unwrap().to_vec2::<f32>().unwrap();
        let out = read
            .apply_readout(&n, &ids, &table)
            .unwrap()
            .reshape((9, 256))
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        for c in 0..256 {
            assert!((out[2][c] - points[7][c]).abs() < 1e-6, "window 0, {c}");
            assert!((out[5][c] - points[11][c]).abs() < 1e-6, "window 1, {c}");
        }
        for r in [0usize, 1, 3, 4, 6, 7, 8] {
            assert_eq!(out[r], own[r], "row {r}");
        }
        // The stream read is the other one; the readout is not offered at its depth.
        assert!(reads.at_depth(HAMMING_READOUT_DEPTH).is_none());
        assert!(reads.at_depth(HAMMING_DEPTHS[0]).is_some());
    }

    /// claude-328's I-5, at the read: dropping the readout's copy heads takes the found copy
    /// away, dropping the stream read's leaves the readout alone, and a copy-scope row cut of 0
    /// at a found row takes that row's copy away and no other.
    #[cfg(feature = "cuda")]
    #[test]
    fn a_dropped_class_or_a_cut_row_takes_the_found_copy_away() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let read = reads.readout().unwrap();
        let copy_only: Vec<f32> = (0..512).map(|i| f32::from(u8::from(i < 256))).collect();
        read.weight
            .set(&Tensor::from_vec(copy_only, (2, 256), &device).unwrap())
            .unwrap();
        let ids = [5u32, 7, 5, 9, 11, 9, 3, 5, 5];
        let n = residual(3, 3, 256, &device);
        let table = table(16, 256, &device);
        let points = table.to_vec2::<f32>().unwrap();
        let own = n.reshape((9, 256)).unwrap().to_vec2::<f32>().unwrap();
        let out = || {
            read.apply_readout(&n, &ids, &table)
                .unwrap()
                .reshape((9, 256))
                .unwrap()
                .to_vec2::<f32>()
                .unwrap()
        };
        let plain = out();
        set_class_ablation([[false, false], [true, false]]);
        let readout_off = out();
        set_class_ablation([[true, true], [false, false]]);
        let stream_off = out();
        set_class_ablation([[false; 2]; 2]);
        let mut limits = vec![3usize; 9];
        limits[2] = 0;
        set_row_cut(Some((ReadScope::Copy, limits)));
        let cut = out();
        set_row_cut(None);
        assert_eq!(readout_off, own, "the readout's copy heads dropped");
        assert_eq!(stream_off, plain, "the stream read's classes dropped");
        assert_eq!(cut[2], own[2], "row 2 cut");
        for c in 0..256 {
            assert!((plain[2][c] - points[7][c]).abs() < 1e-6, "found, {c}");
            assert!((cut[5][c] - points[11][c]).abs() < 1e-6, "row 5 kept, {c}");
        }
        assert_eq!(out(), plain, "everything restored");
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn no_position_reads_a_later_one_and_a_zero_margin_reads_nothing() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let read = &reads.reads[0];
        read.weight
            .set(&Tensor::ones((2, 256), DType::F32, &device).unwrap())
            .unwrap();
        let x = residual(1, 24, 256, &device);
        let table = table(32, 256, &device);
        // Repeated tokens, so the copy heads have exact matches to read.
        let ids: Vec<u32> = (0..24).map(|i| (i % 5) as u32 * 3).collect();
        let base = read
            .apply(&x, &ids, &table)
            .unwrap()
            .squeeze(0)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        // Change only the last position, its point and its token: every earlier output must stay
        // exactly as it was.
        let last = Tensor::ones((1, 1, 256), DType::F32, &device).unwrap();
        let changed = Tensor::cat(&[&x.narrow(1, 0, 23).unwrap(), &last], 1).unwrap();
        let mut changed_ids = ids.clone();
        changed_ids[23] = 31;
        let moved = read
            .apply(&changed, &changed_ids, &table)
            .unwrap()
            .squeeze(0)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        assert_eq!(&moved[..23], &base[..23]);
        assert_ne!(moved[23], base[23]);
        // A margin of 0 bits weighs every candidate at 0, an exact match too: the read adds nothing.
        read.margin
            .set(&Tensor::zeros(4, DType::F32, &device).unwrap())
            .unwrap();
        let none = read.apply(&x, &ids, &table).unwrap();
        assert_eq!(
            none.flatten_all().unwrap().to_vec1::<f32>().unwrap(),
            x.flatten_all().unwrap().to_vec1::<f32>().unwrap()
        );
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_helpful_payload_raises_the_margin_under_descent() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let read = &reads.reads[0];
        read.weight
            .set(&Tensor::ones((2, 256), DType::F32, &device).unwrap())
            .unwrap();
        // A residual of all-positive points, and a table shifted up by 0.5: a quarter of its signs
        // stay negative, so token codes still differ, but every point's sum is positive. Every
        // payload, a type head's or a copy head's, then adds to the sum of the read's output, and
        // a loss that rewards that sum rewards each.
        let x = residual(1, 24, 256, &device)
            .abs()
            .unwrap()
            .affine(1.0, 0.1)
            .unwrap();
        // Margins at a head's full 64 bits, so every candidate but a full mismatch is inside it,
        // and a wider margin raises each one's weight `(m − d) / m` by `d / m²`.
        read.margin
            .set(&Tensor::full(64.0f32, 4, &device).unwrap())
            .unwrap();
        let ids: Vec<u32> = (0..24).map(|i| (i * 5 % 11) as u32).collect();
        let out = read
            .apply(&x, &ids, &table(16, 256, &device).affine(1.0, 0.5).unwrap())
            .unwrap();
        let loss = out.sum_all().unwrap().neg().unwrap();
        let grads = loss.backward().unwrap();
        let margin = grads
            .get(read.margin.as_tensor())
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        // A wider margin weighs the helpful payloads more, so descent raises every head's margin.
        assert!(margin.iter().all(|&g| g < 0.0), "{margin:?}");
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_trunk_with_zero_weight_reads_is_unchanged_and_moved_reads_change_it() {
        let device = Device::new_cuda(0).unwrap();
        let model = crate::spiralcore_fbs_sheet::build_fbs_model(
            crate::spiralcore_fbs_arms::FbsArm::OneAtomActive,
            1,
            &device,
        )
        .unwrap();
        // Seven tokens repeated, so the copy heads find exact matches.
        let ids: Vec<u32> = (0..32u32).map(|i| (i % 7) * 37 + 5).collect();
        let hidden = |m: &crate::geometric_stack::StackModel| {
            crate::hamilton_sheet::with_gate_mode(
                crate::hamilton_sheet::HamiltonArm::HamiltonActive,
                1,
                0,
                false,
                || m.hidden(&ids, 1, 32),
            )
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap()
        };
        let plain = hidden(&model);
        let reads = HammingReads::new(4, crate::hamilton_sheet::SHEET_WIDTH, &device).unwrap();
        set_hamming_reads(Some(reads.clone()));
        let zero = hidden(&model);
        for read in &reads.reads {
            read.weight
                .set(
                    &Tensor::ones((2, crate::hamilton_sheet::SHEET_WIDTH), DType::F32, &device)
                        .unwrap(),
                )
                .unwrap();
        }
        let moved = hidden(&model);
        set_hamming_reads(None);
        assert_eq!(zero, plain);
        assert_ne!(moved, plain);
    }

    /// claude-238: unit rotations off the 120 icosians, a different one per lane and per
    /// rotation `k`, so a snap moves every lane.
    #[cfg(feature = "cuda")]
    fn off_icosian(lanes: usize, k: usize, device: &Device) -> Tensor {
        let values: Vec<f32> = (0..lanes)
            .flat_map(|l| {
                let q = [0.9, 0.3, 0.2 + 0.002 * l as f32, 0.1 + 0.01 * k as f32];
                let norm = q.iter().map(|v| v * v).sum::<f32>().sqrt();
                q.map(|v| v / norm)
            })
            .collect();
        Tensor::from_vec(values, (lanes, 4), device).unwrap()
    }

    /// claude-238: `read`'s key, query and value rotations set off the icosians, and its step
    /// rotation too when the type heads' spin is on.
    #[cfg(feature = "cuda")]
    fn set_off_icosian(read: &HammingRead, device: &Device) {
        let rotations = [
            &read.key_rotation,
            &read.query_rotation,
            &read.value_rotation,
        ];
        let rotations = rotations.into_iter().chain(&read.step_rotation);
        for (k, var) in rotations.enumerate() {
            var.set(&off_icosian(64, k, device)).unwrap();
        }
    }

    /// claude-238: a tensor's values, flat.
    #[cfg(feature = "cuda")]
    fn flat(t: Tensor) -> Result<Vec<f32>> {
        Ok(t.flatten_all()?.to_vec1::<f32>()?)
    }

    /// claude-238: a read whose rotations sit off the icosians, its type heads' step rotation
    /// among them, computes the same output, bit for bit, under the D11 mode, the export gates
    /// and the training arm, for the stream read and the readout. The hook does snap those
    /// rotations, so without the read's guard the first two would differ.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_read_runs_as_trained_under_the_d11_and_export_gates() {
        use crate::hamilton_sheet::{with_d11_export_gates, with_d11_mode, with_gate_mode};
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device)
            .unwrap()
            .with_type_spin()
            .unwrap();
        let read = &reads.reads[0];
        set_off_icosian(read, &device);
        read.weight
            .set(&Tensor::full(0.5f32, (2, 256), &device).unwrap())
            .unwrap();
        let x = residual(2, 24, 256, &device);
        let ids: Vec<u32> = (0..48).map(|i| (i % 5) as u32).collect();
        let table = table(8, 256, &device);
        let active = crate::hamilton_sheet::HamiltonArm::HamiltonActive;
        let both = || -> Result<Vec<f32>> {
            let stream = flat(read.apply(&x, &ids, &table)?)?;
            Ok([stream, flat(read.apply_readout(&x, &ids, &table)?)?].concat())
        };
        let trained = with_gate_mode(active, 1, 0, false, both).unwrap();
        let bits = |v: Vec<f32>| v.iter().map(|x| x.to_bits()).collect::<Vec<u32>>();
        let (d11, export) = (with_d11_mode(both), with_d11_export_gates(both));
        assert_eq!(bits(d11.unwrap()), bits(trained.clone()), "the D11 mode");
        assert_eq!(bits(export.unwrap()), bits(trained), "the export gates");
        // The hook snaps these rotations outside the read, so the read's guard is what holds.
        let points = x.reshape((48, 256)).unwrap();
        let gate = off_icosian(64, 0, &device)
            .reshape((1, 256))
            .unwrap()
            .broadcast_as((48, 256))
            .unwrap()
            .contiguous()
            .unwrap();
        let turn = || flat(hamilton_gate(&gate, &points)?);
        let snapped = with_d11_mode(turn).unwrap();
        assert_ne!(snapped, with_gate_mode(active, 1, 0, false, turn).unwrap());
    }

    /// claude-238: under the D11 mode, a trunk gate after the read is still snapped, and the snap
    /// capture holds that gate alone, none of the read's rotations or its type heads' turns.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_trunks_gates_after_the_read_still_snap() {
        use crate::hamilton_sheet::{with_d11_mode, with_gate_mode, with_snap_capture};
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device)
            .unwrap()
            .with_type_spin()
            .unwrap();
        let read = &reads.reads[0];
        set_off_icosian(read, &device);
        let x = residual(1, 24, 256, &device);
        let ids: Vec<u32> = (0..24).map(|i| (i % 5) as u32).collect();
        let table = table(8, 256, &device);
        let points = x.reshape((24, 256)).unwrap();
        let gate = off_icosian(64, 4, &device)
            .reshape((1, 256))
            .unwrap()
            .broadcast_as((24, 256))
            .unwrap()
            .contiguous()
            .unwrap();
        let turn = || flat(hamilton_gate(&gate, &points)?);
        let (after, captures) = with_snap_capture(|| {
            with_d11_mode(|| {
                read.apply(&x, &ids, &table)?;
                turn()
            })
        })
        .unwrap();
        assert_eq!(captures.len(), 1, "the trunk's gate alone snaps");
        assert_eq!(after, with_d11_mode(turn).unwrap());
        let active = crate::hamilton_sheet::HamiltonArm::HamiltonActive;
        let unsnapped = with_gate_mode(active, 1, 0, false, turn).unwrap();
        assert_ne!(after, unsnapped, "it is snapped");
    }

    /// claude-330: the ask scope's name, and its cuts, decode and row alike, limit the ask heads
    /// (the type heads) only, so `R^{−aQ}` leaves the copy heads' picks as they were.
    #[test]
    fn the_ask_scope_is_named_ask_and_cuts_only_the_ask_heads() {
        assert_eq!(ReadScope::Ask.name(), "ask");
        let (time, copy, rows) = (6, 2, 4);
        for head in 0..4 {
            let end = if head < copy { time } else { 3 };
            assert_eq!(head_limit(Some((ReadScope::Ask, 3)), head, copy, time), end, "{head}");
        }
        let cut = (ReadScope::Ask, vec![6, 3, 9, 1]);
        for head in 0..copy {
            assert_eq!(row_limits(None, Some(&cut), head, copy, time, rows), vec![6; 4]);
        }
        for head in copy..4 {
            assert_eq!(row_limits(None, Some(&cut), head, copy, time, rows), vec![6, 3, 6, 1]);
        }
    }

    /// claude-330, I-A2's targets: a pull's candidates are exactly the earlier positions of its
    /// own window that hold its target, `1 ≤ s < t`, in ascending order; a pull with none has no
    /// span, so it adds nothing.
    #[test]
    fn a_pull_finds_exactly_the_earlier_copies_of_its_target_in_its_window() -> Result<()> {
        // Window 0 holds 9 at 0, 1, 3, 5; window 1 holds 7 at 8 and 10 (local 0 and 2).
        let ids = [9u32, 9, 4, 9, 2, 9, 7, 1, 7, 5, 7, 6, 3, 2, 8, 0];
        let pulls = [(6, 9), (5, 9), (2, 7), (10, 7), (11, 7), (9, 9)];
        let (pairs, spans) = pull_pairs(&ids, 8, &pulls)?;
        // Row 6: s = 1, 3, 5 (not 0). Row 5: s = 1, 3 (not itself). Row 2: no 7 before it.
        // Row 10: only local 0 holds 7 before it, and 0 is never a candidate. Row 11: local 2.
        // Row 9: no 9 in its own window, whatever window 0 holds.
        assert_eq!(pairs, vec![(6, 1), (6, 3), (6, 5), (5, 1), (5, 3), (11, 10)]);
        assert_eq!(spans, vec![0..3, 3..5, 5..6]);
        assert!(pull_pairs(&ids, 8, &[(16, 9)]).is_err(), "a row outside the batch");
        Ok(())
    }

    /// claude-330: the ask heads start as copy heads' questions with zero rows and a quarter of a
    /// head's bits as their margin; the copy heads keep theirs; the type spin and the ask heads
    /// refuse each other; the rows are saved and load back.
    #[test]
    fn the_ask_heads_start_at_zero_rows_and_load_back() -> Result<()> {
        let plain = HammingReads::new(4, 256, &Device::Cpu)?;
        let copy_margins = plain.reads[0].margin.as_tensor().to_vec1::<f32>()?;
        let reads = HammingReads::new(4, 256, &Device::Cpu)?.with_ask(16)?;
        assert!(reads.has_ask() && !plain.has_ask());
        for read in &reads.reads {
            let margins = read.margin.as_tensor().to_vec1::<f32>()?;
            assert_eq!(margins[..2], copy_margins[..2], "the copy heads keep theirs");
            assert_eq!(margins[2..], [16.0, 16.0], "a quarter of 64 bits");
            let ask = read.ask.as_ref().expect("rows").as_tensor();
            assert_eq!(ask.dims(), [16, 256]);
            assert!(ask.flatten_all()?.to_vec1::<f32>()?.iter().all(|&v| v == 0.0));
        }
        assert!(HammingReads::new(4, 256, &Device::Cpu)?.with_ask(16)?.with_type_spin().is_err());
        assert!(HammingReads::new(4, 256, &Device::Cpu)?.with_type_spin()?.with_ask(16).is_err());
        let dir = std::env::temp_dir().join(format!(
            "hamming-ask-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        std::fs::create_dir(&dir)?;
        // Host data only: the laptop refuses CPU compute (D24/D29), not building or copying.
        let ones = Tensor::from_vec(vec![1.0f32; 16 * 256], (16, 256), &Device::Cpu)?;
        reads.reads[1].ask.as_ref().expect("rows").set(&ones)?;
        reads.save(&dir)?;
        let loaded = HammingReads::load(&dir, &Device::Cpu)?.expect("saved reads");
        // Ask heads at one depth only are refused on load.
        let mut half = reads.clone();
        half.reads[0].ask = None;
        std::fs::remove_dir_all(&dir)?;
        std::fs::create_dir(&dir)?;
        half.save(&dir)?;
        let refused = HammingReads::load(&dir, &Device::Cpu).err().map(|e| e.to_string());
        std::fs::remove_dir_all(&dir)?;
        assert!(refused.as_deref().is_some_and(|e| e.contains("ask heads at 1 of its 2 depths")), "{refused:?}");
        // A token past the table has no row to ask with.
        let ask = reads.reads[0].ask.as_ref().expect("rows");
        assert!(check_ask_ids(ask, &[0, 15]).is_ok());
        let past = check_ask_ids(ask, &[3, 16]).err().map(|e| e.to_string());
        assert!(past.as_deref().is_some_and(|e| e.contains("token 16 has no row")), "{past:?}");
        assert!(loaded.has_ask());
        let sum = |r: &HammingRead| -> Result<f32> {
            let rows = r.ask.as_ref().expect("rows").as_tensor().flatten_all()?;
            Ok(rows.to_vec1::<f32>()?.iter().sum())
        };
        assert_eq!((sum(&loaded.reads[0])?, sum(&loaded.reads[1])?), (0.0, 4096.0));
        assert!(!HammingReads::new(4, 256, &Device::Cpu)?.has_ask());
        Ok(())
    }

    /// claude-330: reads with the ask heads on, both depths turned by learned key and query
    /// rotations that differ, as after training.
    #[cfg(feature = "cuda")]
    fn ask_reads(device: &Device) -> HammingReads {
        let reads = HammingReads::new(4, 256, device)
            .unwrap()
            .with_ask(16)
            .unwrap();
        let turned = |q: [f32; 4]| Tensor::from_vec(q.repeat(64), (64, 4), device).unwrap();
        for read in &reads.reads {
            read.key_rotation
                .set(&turned([0.6, 0.8, 0.0, 0.0]))
                .unwrap();
            read.query_rotation
                .set(&turned([0.8, 0.0, 0.6, 0.0]))
                .unwrap();
        }
        reads
    }

    /// Each row's sign code, one `bool` per coordinate, as the read's code takes it.
    #[cfg(feature = "cuda")]
    fn codes(t: &Tensor) -> Vec<Vec<bool>> {
        t.to_vec2::<f32>()
            .unwrap()
            .into_iter()
            .map(|r| r.into_iter().map(|v| v >= 0.0).collect())
            .collect()
    }

    /// Whether every gradient `grads` holds for `t` is zero (or none is held, as for a detached
    /// tensor).
    #[cfg(feature = "cuda")]
    fn no_gradient(grads: &candle_core::backprop::GradStore, t: &Tensor) -> bool {
        grads.get(t).map_or(true, |g| {
            g.abs().unwrap().sum_all().unwrap().to_scalar::<f32>().unwrap() == 0.0
        })
    }

    /// I-A1: with zero rows, each ask head's question code is the copy question code of the same
    /// token, bit for bit, at both depths; and that is not the residual's question.
    #[cfg(feature = "cuda")]
    #[test]
    fn ask_heads_start_as_copy_heads_bit_for_bit() {
        let device = Device::new_cuda(0).unwrap();
        let reads = ask_reads(&device);
        assert_eq!(reads.reads.len(), HAMMING_DEPTHS.len());
        let ids: Vec<u32> = (0..24).map(|i| ((i * 7) % 16) as u32).collect();
        let table = table(16, 256, &device);
        let index = Tensor::from_vec(ids.clone(), 24, &device).unwrap();
        let current = table.index_select(&index, 0).unwrap();
        let points = residual(1, 24, 256, &device).reshape((24, 256)).unwrap();
        for (depth, read) in reads.reads.iter().enumerate() {
            let (copy, typed) = as_trained(|| read.asks(&current, &points, &ids, 24)).unwrap();
            let residual_asks =
                as_trained(|| HammingRead::rotate(&read.query_rotation, &points)).unwrap();
            let (copy, typed, plain) = (codes(&copy), codes(&typed), codes(&residual_asks));
            // Four heads of 64; the ask heads are the last two, columns 128 on.
            for r in 0..24 {
                assert_eq!(copy[r][128..], typed[r][128..], "depth {depth}, row {r}");
            }
            assert_ne!(typed, plain, "depth {depth}: the residual's question differs");
        }
    }

    /// I-A2: the pull finds each ask head's nearest copy (not the first), rows without a copy add
    /// nothing, and its gradient reaches the question rows alone.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_pull_reaches_the_nearest_copy_and_only_the_question_rows_learn() {
        let device = Device::new_cuda(0).unwrap();
        let reads = ask_reads(&device);
        // One window. Token 7 sits at s = 1 after 3, and at s = 3 after 5; a key names the token
        // before it. Row 6 asks with 5, an exact match at s = 3, not the smaller s = 1. Row 7
        // asks with 1, which neither key names: each head's nearest is its least distance.
        let ids = [3u32, 7, 5, 7, 4, 2, 5, 1];
        let held = Var::from_tensor(&table(16, 256, &device)).unwrap();
        let table = held.as_tensor();
        let pull = |pulls: &[(usize, u32)]| reads.ask_pull(&ids, 8, table, pulls).unwrap();
        let scalar = |t: Tensor| t.to_scalar::<f32>().unwrap();
        assert_eq!(scalar(pull(&[(6, 7)]).unwrap()), 0.0, "the exact match at s = 3");
        let mut depths = Vec::new();
        for read in &reads.reads {
            let rows = Tensor::from_vec(vec![1u32, 3, 5], 3, &device).unwrap();
            let rows = table.index_select(&rows, 0).unwrap();
            let c = codes(&as_trained(|| HammingRead::rotate(&read.key_rotation, &rows)).unwrap());
            let mut depth = 0.0f32;
            for h in 2..4 {
                let bits = |k: usize| (h * 64..(h + 1) * 64).filter(|&i| c[0][i] != c[k][i]).count();
                depth += bits(1).min(bits(2)) as f32 / 64.0;
            }
            depths.push(depth / 2.0);
        }
        let expected = depths.iter().sum::<f32>() / depths.len() as f32;
        let seven = scalar(pull(&[(7, 7)]).unwrap());
        assert!(expected > 0.0 && (seven - expected).abs() < 1e-6, "{seven} vs {expected}");
        // The reading: each depth's own mean, on the same rows.
        let read_out = reads.ask_pull_depths(&ids, 8, table, &[(7, 7), (4, 9)]).unwrap().unwrap();
        assert_eq!(read_out.len(), depths.len());
        for (got, want) in read_out.iter().zip(&depths) {
            assert!((*got - f64::from(*want)).abs() < 1e-6, "{got} vs {want}");
        }
        assert!(reads.ask_pull_depths(&ids, 8, table, &[(4, 9)]).unwrap().is_none());
        // Row 4's target is nowhere in the window; row 1 has no earlier candidate.
        assert!(pull(&[(4, 9), (1, 7)]).is_none());
        assert_eq!(scalar(pull(&[(7, 7), (4, 9), (1, 7)]).unwrap()), seven);
        let grads = pull(&[(7, 7)]).unwrap().backward().unwrap();
        for read in &reads.reads {
            assert!(!no_gradient(&grads, read.ask.as_ref().unwrap().as_tensor()));
            for still in [&read.key_rotation, &read.query_rotation, &read.margin, &read.weight] {
                assert!(no_gradient(&grads, still.as_tensor()));
            }
        }
        assert!(no_gradient(&grads, table), "the table is detached");
    }

    /// One plain descent step of `rate` on every depth's question rows.
    #[cfg(feature = "cuda")]
    fn ask_step(reads: &HammingReads, grads: &candle_core::backprop::GradStore, rate: f64) {
        for read in &reads.reads {
            let ask = read.ask.as_ref().unwrap();
            if let Some(g) = grads.get(ask.as_tensor()) {
                ask.set(&ask.as_tensor().sub(&g.affine(rate, 0.0).unwrap()).unwrap())
                    .unwrap();
            }
        }
    }

    /// I-A3: the pull alone, at λ = 1, teaches a question token whose partner is another token
    /// to ask for that token's key, within 100 steps; at λ = 0, with no task loss, the rows stay.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_pull_trains_a_question_onto_its_partners_key() {
        let device = Device::new_cuda(0).unwrap();
        let reads = ask_reads(&device);
        // Token 7 sits at s = 1, after 5. Row 6 asks with 9.
        let ids = [5u32, 7, 2, 4, 3, 1, 9, 6];
        let table = table(16, 256, &device);
        let pull = || reads.ask_pull(&ids, 8, &table, &[(6, 7)]).unwrap().unwrap();
        let start = pull().to_scalar::<f32>().unwrap();
        assert!(start > 0.25, "9 and 5 start about half their bits apart: {start}");
        let rows = |reads: &HammingReads| -> Vec<Vec<f32>> {
            reads
                .reads
                .iter()
                .map(|r| r.ask.as_ref().unwrap().as_tensor().flatten_all().unwrap().to_vec1().unwrap())
                .collect()
        };
        // λ = 0: the runner builds no pull rows (`in_space_long`'s `attract_plan`, pinned there),
        // so there is no term, and the question rows stay where they are.
        let before = rows(&reads);
        assert!(reads.ask_pull(&ids, 8, &table, &[]).unwrap().is_none());
        assert_eq!(rows(&reads), before);
        let mut steps = 0;
        while pull().to_scalar::<f32>().unwrap() > 0.0 {
            assert!(steps < 100, "still {} after 100 steps", pull().to_scalar::<f32>().unwrap());
            ask_step(&reads, &pull().backward().unwrap(), 64.0);
            steps += 1;
        }
        assert!(steps > 0);
    }

    /// I-A4: an ask head that keeps the position after the earlier copy moves the readout to that
    /// position's own table point, as a copy head does; dropping the type class (`−t`) removes the
    /// move.
    #[cfg(feature = "cuda")]
    #[test]
    fn an_ask_head_moves_the_readout_as_a_copy_head_does() {
        let device = Device::new_cuda(0).unwrap();
        let reads = ask_reads(&device);
        let read = &reads.reads[0];
        // copy_case's windows: [5 7 5], [9 11 9], [3 5 5].
        let ids = [5u32, 7, 5, 9, 11, 9, 3, 5, 5];
        let out = residual(3, 3, 256, &device);
        let table = table(16, 256, &device);
        let row = |kind: usize| {
            let on: Vec<f32> = (0..512).map(|i| f32::from(u8::from(i / 256 == kind))).collect();
            Tensor::from_vec(on, (2, 256), &device).unwrap()
        };
        let readout = || {
            read.apply_readout(&out, &ids, &table)
                .unwrap()
                .reshape((9, 256))
                .unwrap()
                .to_vec2::<f32>()
                .unwrap()
        };
        read.weight.set(&row(1)).unwrap();
        let asked = readout();
        read.weight.set(&row(0)).unwrap();
        let copied = readout();
        let points = table.to_vec2::<f32>().unwrap();
        let before = out.reshape((9, 256)).unwrap().to_vec2::<f32>().unwrap();
        for c in 0..256 {
            assert!((asked[2][c] - points[7][c]).abs() < 1e-6, "window 0, {c}");
            assert!((asked[5][c] - points[11][c]).abs() < 1e-6, "window 1, {c}");
        }
        for r in [0usize, 1, 3, 4, 6, 7, 8] {
            assert_eq!(asked[r], before[r], "row {r} found nothing and stays");
        }
        for r in 0..9 {
            for c in 0..256 {
                assert!((asked[r][c] - copied[r][c]).abs() < 1e-6, "row {r}, {c}");
            }
        }
        read.weight.set(&row(1)).unwrap();
        set_class_ablation([[false, false], [false, true]]);
        let dropped = readout();
        set_class_ablation([[false; 2]; 2]);
        assert_eq!(dropped, before, "−t removes the move");
    }

    /// claude-333, I-H4: the hold needs the ask heads and at least a row; its gain starts at zero,
    /// one per token, at the stream read alone; it is saved as `d2.hold` with its row count and
    /// loads back; a save with half of it, or a hold at the readout depth, is refused; and the
    /// decode step and both export paths refuse a held read, each naming the hold. Host data only:
    /// the laptop refuses CPU compute (D24/D29), and every refusal here comes before any.
    #[test]
    fn the_hold_starts_at_zero_loads_back_and_is_refused_where_it_is_not_served() -> Result<()> {
        let cpu = &Device::Cpu;
        assert!(HammingReads::new(4, 256, cpu)?.with_hold(8, 16).is_err(), "no ask heads");
        assert!(HammingReads::new(4, 256, cpu)?.with_ask(16)?.with_hold(0, 16).is_err(), "0 rows");
        let asked = HammingReads::new(4, 256, cpu)?.with_ask(16)?;
        assert_eq!(asked.hold_rows(), 0);
        let reads = HammingReads::new(4, 256, cpu)?.with_ask(16)?.with_hold(8, 16)?;
        assert_eq!(reads.hold_rows(), 8);
        let gain = &reads.reads[0].hold.as_ref().expect("the stream read holds").gain;
        assert_eq!(gain.as_tensor().dims(), [16]);
        assert!(gain.as_tensor().to_vec1::<f32>()?.iter().all(|&v| v == 0.0));
        assert!(reads.reads[1].hold.is_none(), "the readout read holds nothing");
        assert!(reads.reads[0].named().iter().any(|(name, _)| *name == "hold"), "it is trained");
        let set: Vec<f32> = (0..16).map(|i| i as f32 - 7.5).collect();
        gain.set(&Tensor::from_vec(set.clone(), 16, cpu)?)?;
        let dir = std::env::temp_dir().join(format!(
            "hamming-hold-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        let saved = |reads: &HammingReads| -> Result<std::result::Result<HammingReads, String>> {
            if dir.exists() {
                std::fs::remove_dir_all(&dir)?;
            }
            std::fs::create_dir(&dir)?;
            reads.save(&dir)?;
            let names = candle_core::safetensors::load(dir.join(FILE), cpu)?;
            let loaded = HammingReads::load(&dir, cpu).map_err(|e| e.to_string());
            std::fs::remove_dir_all(&dir)?;
            Ok(loaded.map(|r| {
                let r = r.expect("saved reads");
                assert_eq!(names.contains_key("d2.hold"), r.hold_rows() > 0, "the gain is d2.hold");
                r
            }))
        };
        let loaded = saved(&reads)?.expect("a held read loads");
        assert_eq!(loaded.hold_rows(), 8);
        let loaded_gain = &loaded.reads[0].hold.as_ref().expect("held").gain;
        assert_eq!(loaded_gain.as_tensor().to_vec1::<f32>()?, set);
        assert!(loaded.reads[1].hold.is_none());
        let plain = saved(&asked)?.expect("an unheld read loads");
        assert_eq!(plain.hold_rows(), 0);
        // A gain with no row count; a hold at the readout depth; a hold with no ask heads.
        let mut no_rows = reads.clone();
        no_rows.reads[0].hold.as_mut().expect("held").rows = 0;
        let err = saved(&no_rows)?.err().unwrap_or_default();
        assert!(err.contains("hold gain but no row count"), "{err}");
        let mut deep = reads.clone();
        deep.reads[1].hold = deep.reads[0].hold.clone();
        let err = saved(&deep)?.err().unwrap_or_default();
        assert!(err.contains("at depth 6; only the stream read"), "{err}");
        let mut bare = reads.clone();
        for read in &mut bare.reads {
            read.ask = None;
        }
        let err = saved(&bare)?.err().unwrap_or_default();
        assert!(err.contains("holds an ask write but has no ask heads"), "{err}");
        // claude-334: a hold whose gains do not match the ask rows is refused at load.
        let short = HammingReads::new(4, 256, cpu)?.with_ask(16)?.with_hold(8, 12)?;
        let err = saved(&short)?.err().unwrap_or_default();
        assert!(err.contains("has 12 gains for 16 ask rows"), "{err}");
        // claude-334: the hold's gain reads back on the host; an unheld read has none.
        assert_eq!(reads.hold_gain()?, Some(set.clone()));
        assert_eq!(asked.hold_gain()?, None);
        assert_eq!(asked.type_weights()?.len(), HAMMING_DEPTHS.len());
        // The decode step refuses before any arithmetic; the exports refuse it by name.
        let x = Tensor::zeros((1, 4, 256), DType::F32, cpu)?;
        let table = Tensor::zeros((16, 256), DType::F32, cpu)?;
        let err = reads.reads[0].apply_last(&x, &[1, 2, 3, 4], &table).err().map(|e| e.to_string());
        assert!(err.as_deref().is_some_and(|e| e.contains("held ask write (--hamming-hold")), "{err:?}");
        let err = crate::fbs_export::round_hamming_as_exported(&reads).err().map(|e| e.to_string());
        assert!(err.as_deref().is_some_and(|e| e.contains("--hamming-hold")), "{err:?}");
        let err = crate::fbs_export::fbs_hamming_snapshot(&reads, 6, 256).err().map(|e| e.to_string());
        assert!(err.as_deref().is_some_and(|e| e.contains("--hamming-hold")), "{err:?}");
        Ok(())
    }

    /// claude-334: the hold cut and the row capture are instruments the decode cache's step does
    /// not carry, so a held or captured read decodes whole windows; the capture is taken once and
    /// gone. Host state only.
    #[test]
    fn the_hold_cut_and_the_capture_keep_the_decode_cache_off() {
        assert_eq!(step_instrument(), None);
        set_hold_off(true);
        assert!(hold_off());
        assert_eq!(step_instrument(), Some("Hamming read's hold cut"));
        set_hold_off(false);
        set_row_capture(Some(vec![0, 3]));
        assert_eq!(step_instrument(), Some("Hamming read's row capture"));
        let capture = take_row_capture().expect("set");
        assert_eq!(capture.rows, vec![0, 3]);
        assert!(capture.stream.is_none() && capture.readout_in.is_none());
        assert_eq!(take_row_capture(), None);
        assert_eq!(step_instrument(), None);
    }

    /// Hand-set values off every start: a deterministic wave in (−1, 1).
    #[cfg(feature = "cuda")]
    fn wave(n: usize, salt: usize) -> Vec<f32> {
        (0..n)
            .map(|i| ((((i + salt) * 2_654_435_761) % 1_000) as f32 / 500.0) - 0.999)
            .collect()
    }

    /// claude-333, I-H1: at `g = 0`, a held read's forward, and the gradient of every parameter
    /// the two reads share, equal the unheld read's as numbers (`−0.0 == +0.0`), with `w_c`,
    /// `w_y` and the ask rows set off their starts so that every gradient is live. The stream is
    /// compared before the `q16` rounding that wraps the read's return, so after it too. Once
    /// `g` moves, the forward moves: the held run does hold.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_hold_starts_off_in_the_forward_and_the_gradient() {
        let device = Device::new_cuda(0).unwrap();
        let reads = ask_reads(&device);
        for read in &reads.reads {
            read.weight
                .set(&Tensor::from_vec(wave(512, 3), (2, 256), &device).unwrap())
                .unwrap();
            let rows: Vec<f32> = wave(16 * 256, 5).iter().map(|v| v * 0.05).collect();
            read.ask
                .as_ref()
                .unwrap()
                .set(&Tensor::from_vec(rows, (16, 256), &device).unwrap())
                .unwrap();
        }
        let held = reads.clone().with_hold(8, 16).unwrap();
        let ids: Vec<u32> = (0..48).map(|i| ((i * 5 + i / 7) % 16) as u32).collect();
        let x = residual(2, 24, 256, &device);
        let table = table(16, 256, &device);
        let probe = Tensor::from_vec(wave(48 * 256, 11), (2, 24, 256), &device).unwrap();
        let run = |r: &HammingReads| {
            let out = r.reads[0].apply(&x, &ids, &table).unwrap();
            let grads = out.mul(&probe).unwrap().sum_all().unwrap().backward().unwrap();
            (out.flatten_all().unwrap().to_vec1::<f32>().unwrap(), grads)
        };
        let ((plain, plain_grads), (with_hold, held_grads)) = (run(&reads), run(&held));
        assert!(plain.iter().zip(&with_hold).all(|(a, b)| a == b), "the forward");
        let mut live = 0;
        for (name, var) in reads.reads[0].named() {
            let of = |grads: &candle_core::backprop::GradStore| {
                grads.get(var.as_tensor()).map(|g| g.flatten_all().unwrap().to_vec1::<f32>().unwrap())
            };
            let (a, b) = (of(&plain_grads), of(&held_grads));
            assert_eq!(a.is_some(), b.is_some(), "{name}");
            if let (Some(a), Some(b)) = (a, b) {
                assert!(a.iter().zip(&b).all(|(p, q)| p == q), "{name}");
                live += usize::from(a.iter().any(|&v| v != 0.0));
            }
        }
        assert!(live >= 2, "the weights and the ask rows learn: {live}");
        let gain = &held.reads[0].hold.as_ref().unwrap().gain;
        gain.set(&Tensor::ones(16, DType::F32, &device).unwrap()).unwrap();
        assert_ne!(run(&held).0, plain, "a moved gain moves the stream");
    }

    /// claude-333, I-H2: one nonzero write `u` at row `t` reaches rows `t + 1` to `t + h` of its
    /// own window as `g[x_t] · u`, exactly, and nothing else: not row `t`, not before it, not past
    /// `t + h`, and not the batch's other window, also when the window ends inside the hold.
    #[cfg(feature = "cuda")]
    #[test]
    fn a_held_write_reaches_exactly_the_next_rows_of_its_window() {
        let device = Device::new_cuda(0).unwrap();
        let (batch, time, width, rows) = (2, 16, 8, 8);
        // Every token's gain differs, so a gain read at the wrong row shows.
        let gains: Vec<f32> = (0..16).map(|i| i as f32 + 1.0).collect();
        let gain = Var::from_tensor(&Tensor::from_vec(gains.clone(), 16, &device).unwrap()).unwrap();
        let ids: Vec<u32> = (0..batch * time)
            .map(|i| if i < time { i as u32 } else { ((i * 3) % 16) as u32 })
            .collect();
        let u: Vec<f32> = (1..=width).map(|k| k as f32 * 0.5).collect();
        for t in [3usize, 12] {
            let mut writes = vec![0.0f32; batch * time * width];
            writes[t * width..(t + 1) * width].copy_from_slice(&u);
            let writes = Tensor::from_vec(writes, (batch * time, width), &device).unwrap();
            let held = held_term(&gain, &writes, &ids, batch, time, rows)
                .unwrap()
                .to_vec2::<f32>()
                .unwrap();
            let g = gains[ids[t] as usize];
            for (r, row) in held.iter().enumerate() {
                let want: Vec<f32> = if r > t && r <= t + rows && r < time {
                    u.iter().map(|v| g * v).collect()
                } else {
                    vec![0.0; width]
                };
                assert_eq!(*row, want, "write at {t}, row {r}");
            }
        }
        assert!(held_term(&gain, &Tensor::zeros((32, 8), DType::F32, &device).unwrap(), &[16; 32], 2, 16, 8).is_err(), "a token past the gains");
    }

    /// claude-333, I-H3: at a hand-set nonzero `g`, `w_y` and `a` (at the zero start each of
    /// these gradients is 0), a loss at row `t + 4` alone gives `g[x_t]` and `a[x_t]` gradients,
    /// `a[x_t]`'s through the hold alone, and gives none to `g` of the tokens that appear only
    /// after `t + 4`.
    #[cfg(feature = "cuda")]
    #[test]
    fn a_loss_after_the_question_reaches_its_gain_and_ask_row_through_the_hold() {
        let device = Device::new_cuda(0).unwrap();
        let reads = ask_reads(&device);
        let on: Vec<f32> = (0..512).map(|i| f32::from(u8::from(i >= 256))).collect();
        let rows: Vec<f32> = wave(16 * 256, 5).iter().map(|v| v * 0.02).collect();
        for read in &reads.reads {
            read.weight.set(&Tensor::from_vec(on.clone(), (2, 256), &device).unwrap()).unwrap();
            read.ask
                .as_ref()
                .unwrap()
                .set(&Tensor::from_vec(rows.clone(), (16, 256), &device).unwrap())
                .unwrap();
        }
        let held = reads.clone().with_hold(8, 16).unwrap();
        let gain = &held.reads[0].hold.as_ref().unwrap().gain;
        gain.set(&Tensor::from_vec(vec![0.5f32; 16], 16, &device).unwrap()).unwrap();
        // t = 6 asks with 5 and finds s = 2 (the key at 2 names 5); row 1's 5 holds to row 9 only.
        // Tokens 12 to 15 first appear after row 10.
        let ids = [3u32, 5, 7, 4, 2, 1, 5, 8, 9, 10, 11, 12, 13, 14, 15, 12];
        let x = residual(1, 16, 256, &device);
        let table = table(16, 256, &device);
        let probe = Tensor::from_vec(wave(256, 13), (1, 1, 256), &device).unwrap();
        let grads = |r: &HammingReads| {
            let out = r.reads[0].apply(&x, &ids, &table).unwrap();
            out.narrow(1, 10, 1).unwrap().mul(&probe).unwrap().sum_all().unwrap().backward().unwrap()
        };
        let ask_row = |grads: &candle_core::backprop::GradStore, token: usize| -> Vec<f32> {
            let ask = reads.reads[0].ask.as_ref().unwrap().as_tensor();
            grads.get(ask).map_or(vec![0.0; 256], |g| g.get(token).unwrap().to_vec1().unwrap())
        };
        // claude-334 (claude-333's review): row 6 found its candidate, so a failure below names
        // the gradient, not a missed find.
        set_row_capture(Some(vec![6]));
        held.reads[0].apply(&x, &ids, &table).unwrap();
        let capture = take_row_capture().expect("the capture");
        assert_eq!(capture.stream.expect("the stream read").found, vec![true], "row 6 finds s = 2");
        let with_hold = grads(&held);
        let g = with_hold.get(gain.as_tensor()).expect("the gain learns").to_vec1::<f32>().unwrap();
        assert_ne!(g[5], 0.0, "g[x_t]");
        assert!(g[12..].iter().all(|&v| v == 0.0), "tokens after t + 4: {:?}", &g[12..]);
        assert!(ask_row(&with_hold, 5).iter().any(|&v| v != 0.0), "a[x_t] with the hold");
        assert!(ask_row(&grads(&reads), 5).iter().all(|&v| v == 0.0), "and only through it");
    }

    /// claude-334's I-H5 (the cuts): `−aS` removes every stream ask write and every held term,
    /// leaves the copy heads' part of the stream and the payload as they were, and leaves the
    /// readout read's ask heads on, bit for bit. `−H` removes every held term and leaves the
    /// direct write bit for bit.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_ask_stream_cut_and_the_hold_cut_remove_exactly_their_terms() {
        use crate::in_space_long::Ablation;
        let device = Device::new_cuda(0).unwrap();
        let reads = ask_reads(&device);
        let on: Vec<f32> = wave(2 * 256, 3).iter().map(|v| v + 1.5).collect();
        let rows: Vec<f32> = wave(16 * 256, 5).iter().map(|v| v * 0.02).collect();
        for read in &reads.reads {
            read.weight.set(&Tensor::from_vec(on.clone(), (2, 256), &device).unwrap()).unwrap();
            read.ask
                .as_ref()
                .unwrap()
                .set(&Tensor::from_vec(rows.clone(), (16, 256), &device).unwrap())
                .unwrap();
        }
        let held = reads.with_hold(8, 16).unwrap();
        let gain = &held.reads[0].hold.as_ref().unwrap().gain;
        gain.set(&Tensor::from_vec(wave(16, 9), 16, &device).unwrap()).unwrap();
        let ids = [3u32, 5, 7, 4, 2, 1, 5, 8, 9, 10, 11, 12, 13, 14, 15, 12];
        let x = residual(1, 16, 256, &device);
        let table = table(16, 256, &device);
        let all: Vec<usize> = (0..16).collect();
        let read = |cut: Option<Ablation>| -> (StreamCapture, Vec<f32>) {
            if let Some(cut) = cut {
                set_class_ablation(cut.classes());
                set_hold_off(cut.hold_off());
            }
            set_row_capture(Some(all.clone()));
            held.reads[0].apply(&x, &ids, &table).unwrap();
            let readout = held.reads[1].apply_readout(&x, &ids, &table).unwrap();
            let capture = take_row_capture().unwrap();
            set_class_ablation([[false; 2]; 2]);
            set_hold_off(false);
            (capture.stream.unwrap(), readout.flatten_all().unwrap().to_vec1().unwrap())
        };
        let zero = |v: &[Vec<f32>]| v.iter().all(|r| r.iter().all(|&c| c == 0.0));
        let (plain, plain_readout) = read(None);
        assert!(!zero(&plain.write) && !zero(&plain.held), "the plain read writes and holds");
        let (cut, cut_readout) = read(Some(Ablation::AskStream));
        assert!(zero(&cut.write) && zero(&cut.held), "−aS: no ask write and no held term");
        assert_eq!(cut.payload, plain.payload, "−aS keeps the heads' choices");
        assert_eq!(cut.found, plain.found);
        for t in 0..16 {
            for c in 0..256 {
                let copy_part = plain.out[t][c] - plain.write[t][c] - plain.held[t][c];
                assert!((cut.out[t][c] - copy_part).abs() < 1e-5, "row {t} lane {c}: the copy part stays");
            }
        }
        assert_eq!(cut_readout, plain_readout, "−aS leaves the readout read's ask heads on");
        let (off, _) = read(Some(Ablation::HoldOff));
        assert!(zero(&off.held), "−H: no held term");
        assert_eq!(off.write, plain.write, "−H keeps the direct write bit for bit");
        assert_eq!(off.payload, plain.payload);
    }

    /// claude-335, I-C0: the carry needs the ask heads and at least a row; its gain starts at
    /// zero, one per token, at the readout read alone; it is saved as `d6.carry` with its row
    /// count and loads back; a save with half of it, a carry at the stream depth, a carry with no
    /// ask heads, or one whose gains do not match the ask rows, is refused; and the decode step
    /// and both export paths refuse a carried read, each naming the carry. Host data only: every
    /// refusal here comes before any arithmetic.
    #[test]
    fn the_carry_starts_at_zero_loads_back_and_is_refused_where_it_is_not_served() -> Result<()> {
        let cpu = &Device::Cpu;
        assert!(HammingReads::new(4, 256, cpu)?.with_carry(4, 16).is_err(), "no ask heads");
        assert!(HammingReads::new(4, 256, cpu)?.with_ask(16)?.with_carry(0, 16).is_err(), "0 rows");
        let asked = HammingReads::new(4, 256, cpu)?.with_ask(16)?;
        assert_eq!(asked.carry_rows(), 0);
        let reads = HammingReads::new(4, 256, cpu)?.with_ask(16)?.with_carry(4, 16)?;
        assert_eq!((reads.carry_rows(), reads.hold_rows()), (4, 0));
        let gain = &reads.reads[1].carry.as_ref().expect("the readout read carries").gain;
        assert_eq!(gain.as_tensor().dims(), [16]);
        assert!(gain.as_tensor().to_vec1::<f32>()?.iter().all(|&v| v == 0.0));
        assert!(reads.reads[0].carry.is_none(), "the stream read carries nothing");
        assert!(reads.reads[1].named().iter().any(|(name, _)| *name == "carry"), "it is trained");
        let set: Vec<f32> = (0..16).map(|i| 0.25 * i as f32 - 2.0).collect();
        gain.set(&Tensor::from_vec(set.clone(), 16, cpu)?)?;
        let dir = std::env::temp_dir().join(format!(
            "hamming-carry-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        let saved = |reads: &HammingReads| -> Result<std::result::Result<HammingReads, String>> {
            if dir.exists() {
                std::fs::remove_dir_all(&dir)?;
            }
            std::fs::create_dir(&dir)?;
            reads.save(&dir)?;
            let names = candle_core::safetensors::load(dir.join(FILE), cpu)?;
            let loaded = HammingReads::load(&dir, cpu).map_err(|e| e.to_string());
            std::fs::remove_dir_all(&dir)?;
            Ok(loaded.map(|r| {
                let r = r.expect("saved reads");
                assert_eq!(names.contains_key("d6.carry"), r.carry_rows() > 0, "the gain is d6.carry");
                assert_eq!(names.contains_key("carry_rows"), r.carry_rows() > 0);
                r
            }))
        };
        let loaded = saved(&reads)?.expect("a carried read loads");
        assert_eq!(loaded.carry_rows(), 4);
        assert_eq!(loaded.carry_gain()?, Some(set.clone()));
        assert!(loaded.reads[0].carry.is_none());
        let plain = saved(&asked)?.expect("an uncarried read loads");
        assert_eq!(plain.carry_rows(), 0);
        // A held and carried read keeps both apart.
        let both = HammingReads::new(4, 256, cpu)?.with_ask(16)?.with_hold(8, 16)?.with_carry(4, 16)?;
        let both = saved(&both)?.expect("held and carried");
        assert_eq!((both.hold_rows(), both.carry_rows()), (8, 4));
        // A gain with no row count; a carry at the stream depth; a carry with no ask heads.
        let mut no_rows = reads.clone();
        no_rows.reads[1].carry.as_mut().expect("carried").rows = 0;
        let err = saved(&no_rows)?.err().unwrap_or_default();
        assert!(err.contains("carry gain but no row count"), "{err}");
        let mut shallow = reads.clone();
        shallow.reads[0].carry = shallow.reads[1].carry.clone();
        let err = saved(&shallow)?.err().unwrap_or_default();
        assert!(err.contains("at depth 2; only the readout read"), "{err}");
        let mut bare = reads.clone();
        for read in &mut bare.reads {
            read.ask = None;
        }
        let err = saved(&bare)?.err().unwrap_or_default();
        assert!(err.contains("carries a question but has no ask heads"), "{err}");
        let short = HammingReads::new(4, 256, cpu)?.with_ask(16)?.with_carry(4, 12)?;
        let err = saved(&short)?.err().unwrap_or_default();
        assert!(err.contains("the saved carry has 12 gains for 16 ask rows"), "{err}");
        assert_eq!(asked.carry_gain()?, None);
        // The decode step refuses before any arithmetic; the exports refuse it by name.
        let x = Tensor::zeros((1, 4, 256), DType::F32, cpu)?;
        let table = Tensor::zeros((16, 256), DType::F32, cpu)?;
        let err = reads.reads[1].apply_readout_last(&x, &[1, 2, 3, 4], &table).err().map(|e| e.to_string());
        assert!(err.as_deref().is_some_and(|e| e.contains("carried question (--hamming-carry-question")), "{err:?}");
        let err = crate::fbs_export::round_hamming_as_exported(&reads).err().map(|e| e.to_string());
        assert!(err.as_deref().is_some_and(|e| e.contains("--hamming-carry-question")), "{err:?}");
        let err = crate::fbs_export::fbs_hamming_snapshot(&reads, 6, 256).err().map(|e| e.to_string());
        assert!(err.as_deref().is_some_and(|e| e.contains("--hamming-carry-question")), "{err:?}");
        Ok(())
    }

    /// claude-335: the carry cut is an instrument the decode cache's step does not carry. Host
    /// state only.
    #[test]
    fn the_carry_cut_keeps_the_decode_cache_off() {
        assert_eq!(step_instrument(), None);
        set_carry_off(true);
        assert!(carry_off());
        assert_eq!(step_instrument(), Some("Hamming read's carry cut"));
        set_carry_off(false);
        assert_eq!(step_instrument(), None);
    }

    /// claude-335's reads for the carry tests: the ask heads on, `w_c`, `w_y` and the ask rows
    /// set off their starts so that every gradient is live.
    #[cfg(feature = "cuda")]
    fn live_ask_reads(device: &Device) -> HammingReads {
        let reads = ask_reads(device);
        for read in &reads.reads {
            read.weight
                .set(&Tensor::from_vec(wave(512, 3), (2, 256), device).unwrap())
                .unwrap();
            let rows: Vec<f32> = wave(16 * 256, 5).iter().map(|v| v * 0.05).collect();
            read.ask
                .as_ref()
                .unwrap()
                .set(&Tensor::from_vec(rows, (16, 256), device).unwrap())
                .unwrap();
        }
        reads
    }

    /// claude-335, I-C1: at `c = 0`, a carried read's readout forward, and the gradient of every
    /// parameter the two reads share, equal the uncarried read's as numbers; once `c` moves, the
    /// readout moves: the carried run does carry.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_carry_starts_off_in_the_readout_forward_and_the_gradient() {
        let device = Device::new_cuda(0).unwrap();
        let reads = live_ask_reads(&device);
        let carried = reads.clone().with_carry(4, 16).unwrap();
        let ids: Vec<u32> = (0..48).map(|i| ((i * 5 + i / 7) % 16) as u32).collect();
        let x = residual(2, 24, 256, &device);
        let table = table(16, 256, &device);
        let probe = Tensor::from_vec(wave(48 * 256, 11), (2, 24, 256), &device).unwrap();
        let run = |r: &HammingReads| {
            let out = r.reads[1].apply_readout(&x, &ids, &table).unwrap();
            let grads = out.mul(&probe).unwrap().sum_all().unwrap().backward().unwrap();
            (out.flatten_all().unwrap().to_vec1::<f32>().unwrap(), grads)
        };
        let ((plain, plain_grads), (with_carry, carried_grads)) = (run(&reads), run(&carried));
        assert!(plain.iter().zip(&with_carry).all(|(a, b)| a == b), "the forward");
        let mut live = 0;
        for (name, var) in reads.reads[1].named() {
            let of = |grads: &candle_core::backprop::GradStore| {
                grads.get(var.as_tensor()).map(|g| g.flatten_all().unwrap().to_vec1::<f32>().unwrap())
            };
            let (a, b) = (of(&plain_grads), of(&carried_grads));
            assert_eq!(a.is_some(), b.is_some(), "{name}");
            if let (Some(a), Some(b)) = (a, b) {
                assert!(a.iter().zip(&b).all(|(p, q)| p == q), "{name}");
                live += usize::from(a.iter().any(|&v| v != 0.0));
            }
        }
        assert!(live >= 2, "the weights and the ask rows learn: {live}");
        let gain = &carried.reads[1].carry.as_ref().unwrap().gain;
        gain.set(&Tensor::ones(16, DType::F32, &device).unwrap()).unwrap();
        assert_ne!(run(&carried).0, plain, "a moved gain moves the readout");
    }

    /// claude-335, I-C2: one nonzero question point `u + a[x_t]` at row `t` (and `a[x_t]` alone
    /// at the other window's rows of `x_t`, checked by a brute-force sum) reaches the asked points
    /// of rows `t + 1` to `t + h` of its own window as `c[x_t] · (u + a[x_t])`, exactly, and nothing else: row
    /// `t` keeps its own point unscaled, and no row before it, past `t + h`, or in the batch's
    /// other window gets any, also when the window ends inside the carry. Under `−C` only row `t`
    /// is nonzero.
    #[cfg(feature = "cuda")]
    #[test]
    fn a_carried_question_reaches_exactly_the_next_rows_of_its_window() {
        let device = Device::new_cuda(0).unwrap();
        let (batch, time, width, rows) = (2, 16, 256, 4);
        let reads = HammingReads::new(4, width, &device)
            .unwrap()
            .with_ask(16)
            .unwrap()
            .with_carry(rows, 16)
            .unwrap();
        let read = &reads.reads[1];
        // Every token's gain differs, so a gain read at the wrong row shows.
        let gains: Vec<f32> = (0..16).map(|i| i as f32 + 1.0).collect();
        read.carry
            .as_ref()
            .unwrap()
            .gain
            .set(&Tensor::from_vec(gains.clone(), 16, &device).unwrap())
            .unwrap();
        let ask = read.ask.as_ref().unwrap();
        let ids: Vec<u32> = (0..batch * time)
            .map(|i| if i < time { i as u32 } else { ((i * 3) % 16) as u32 })
            .collect();
        let u: Vec<f32> = (0..width).map(|k| (k % 7) as f32 * 0.5 - 1.0).collect();
        // The question table's row of `t`'s token, so the carry must carry `point + a`, not the
        // point alone (claude-335 FIT round 1, minor 1).
        let ua: Vec<f32> = (0..width).map(|k| (k % 5) as f32 * 0.25 - 0.5).collect();
        for t in [3usize, 13] {
            let mut current = vec![0.0f32; batch * time * width];
            current[t * width..(t + 1) * width].copy_from_slice(&u);
            let mut table = vec![0.0f32; 16 * width];
            let token = ids[t] as usize;
            table[token * width..(token + 1) * width].copy_from_slice(&ua);
            ask.set(&Tensor::from_vec(table, (16, width), &device).unwrap()).unwrap();
            // Every row's own question, `point + a[x_r]`: `t`'s, and the rows of the other
            // window that hold `t`'s token.
            let own: Vec<Vec<f32>> = (0..batch * time)
                .map(|r| {
                    let point = if r == t { u.clone() } else { vec![0.0; width] };
                    let a = if ids[r] as usize == token { ua.clone() } else { vec![0.0; width] };
                    point.iter().zip(&a).map(|(p, q)| p + q).collect()
                })
                .collect();
            let current = Tensor::from_vec(current, (batch * time, width), &device).unwrap();
            for off in [false, true] {
                set_carry_off(off);
                let asked = read.asked(&current, ask, &ids, time).unwrap().to_vec2::<f32>().unwrap();
                set_carry_off(false);
                for (r, row) in asked.iter().enumerate() {
                    // The brute-force sum over the `rows` rows before `r` in its own window.
                    let mut want = own[r].clone();
                    if !off {
                        for s in 1..=rows.min(r % time) {
                            let g = gains[ids[r - s] as usize];
                            for (w, v) in want.iter_mut().zip(&own[r - s]) {
                                *w += g * v;
                            }
                        }
                    }
                    assert_eq!(*row, want, "point at {t}, row {r}, carry off {off}");
                }
            }
        }
    }

    /// claude-335, I-C3: with `c[Q] = 1`, every other gain 0 and `a[H4] = −point(H4)`, the readout
    /// read's pull at `H4` asks exactly the question `Q` asks, so its distance to the copy is
    /// `Q`'s, bit for bit; under `−C` it is not. The pull's gradient reaches `c` at the tokens of
    /// the `h` rows before `H4` and at no other, and `a[Q]` through the carry; under `−C` it
    /// reaches no gain.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_carried_pull_asks_the_question_rows_question_and_trains_the_gain() {
        let device = Device::new_cuda(0).unwrap();
        let reads = live_ask_reads(&device).with_carry(4, 16).unwrap();
        // One window: the value 7 at s = 1 after the key 3; Q = 5 at row 6, the header 13, 14, 15
        // at rows 7 to 9 and H4 = 1 at row 10; tokens 9 to 12 sit at rows 2 to 5, 0, 2, 4, 6, 8
        // after row 10.
        let ids = [3u32, 7, 9, 10, 11, 12, 5, 13, 14, 15, 1, 0, 2, 4, 6, 8];
        let table = table(16, 256, &device);
        let read = &reads.reads[1];
        let carry = &read.carry.as_ref().unwrap().gain;
        let mut gains = vec![0.0f32; 16];
        gains[5] = 1.0;
        carry.set(&Tensor::from_vec(gains, 16, &device).unwrap()).unwrap();
        let ask = read.ask.as_ref().unwrap();
        let mut rows = ask.as_tensor().to_vec2::<f32>().unwrap();
        rows[1] = table.get(1).unwrap().to_vec1::<f32>().unwrap().iter().map(|v| -v).collect();
        ask.set(&Tensor::from_vec(rows.concat(), (16, 256), &device).unwrap()).unwrap();
        let readout = |pulls: &[(usize, u32)]| reads.ask_pull_depths(&ids, 16, &table, pulls).unwrap().unwrap()[1];
        let (at_q, at_h4) = (readout(&[(6, 7)]), readout(&[(10, 7)]));
        assert_eq!(at_q, at_h4, "H4 asks Q's question");
        set_carry_off(true);
        let cut = readout(&[(10, 7)]);
        set_carry_off(false);
        assert_ne!(cut, at_q, "without the carry H4 asks its own");
        let grads = reads.ask_pull(&ids, 16, &table, &[(10, 7)]).unwrap().unwrap().backward().unwrap();
        let c = grads.get(carry.as_tensor()).expect("the gain learns").to_vec1::<f32>().unwrap();
        for token in [5usize, 13, 14, 15] {
            assert_ne!(c[token], 0.0, "c[{token}], a row inside the carry");
        }
        for token in [0usize, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12] {
            assert_eq!(c[token], 0.0, "c[{token}], a token outside the carry");
        }
        let a = grads.get(ask.as_tensor()).expect("the ask rows learn");
        assert!(a.get(5).unwrap().to_vec1::<f32>().unwrap().iter().any(|&v| v != 0.0), "a[Q] through the carry");
        set_carry_off(true);
        let cut_grads = reads.ask_pull(&ids, 16, &table, &[(10, 7)]).unwrap().unwrap().backward().unwrap();
        set_carry_off(false);
        assert!(no_gradient(&cut_grads, carry.as_tensor()), "−C trains no gain");
    }

    /// claude-335, I-C4: `−C` drops the carried question alone: a carried read with a moved gain
    /// reads, under the cut, exactly as the same read without a carry, bit for bit, and differs
    /// without the cut.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_carry_cut_drops_the_carried_question_alone() {
        let device = Device::new_cuda(0).unwrap();
        let reads = live_ask_reads(&device);
        let carried = reads.clone().with_carry(4, 16).unwrap();
        let gains: Vec<f32> = wave(16, 7).iter().map(|v| v * 2.0).collect();
        carried.reads[1]
            .carry
            .as_ref()
            .unwrap()
            .gain
            .set(&Tensor::from_vec(gains, 16, &device).unwrap())
            .unwrap();
        let ids: Vec<u32> = (0..48).map(|i| ((i * 5 + i / 7) % 16) as u32).collect();
        let x = residual(2, 24, 256, &device);
        let table = table(16, 256, &device);
        let out = |r: &HammingReads| {
            r.reads[1].apply_readout(&x, &ids, &table).unwrap().flatten_all().unwrap().to_vec1::<f32>().unwrap()
        };
        let plain = out(&reads);
        set_carry_off(true);
        let cut = out(&carried);
        set_carry_off(false);
        assert_eq!(cut, plain, "−C reads as the uncarried read");
        assert_ne!(out(&carried), plain, "the carry is on without the cut");
        let stream = |r: &HammingReads| {
            r.reads[0].apply(&x, &ids, &table).unwrap().flatten_all().unwrap().to_vec1::<f32>().unwrap()
        };
        assert_eq!(stream(&carried), stream(&reads), "the stream read is untouched");
    }
}

// I1-DIGEST BEGIN (claude-330's I-1: the pod appends this module, byte for byte, to the base
// commit's copy of this file too, and the two printed digests must match. From claude-333 the
// base already holds a block: the pod replaces it, BEGIN to END, with this one.)
#[cfg(all(test, feature = "cuda"))]
mod i1_digest {
    use super::*;
    use sha2::{Digest, Sha256};

    /// I-1: a fixed forward of the default read, no new flag, stream and readout, at both
    /// depths, with every learned part set off its start; prints the SHA-256 of the outputs'
    /// little-endian bytes.
    #[test]
    fn ask_default_forward_digest() {
        let device = Device::new_cuda(0).unwrap();
        let (batch, time, width, vocab) = (2, 24, 256, 16);
        let wave = |n: usize, salt: usize| -> Vec<f32> {
            (0..n)
                .map(|i| ((((i + salt) * 2_654_435_761) % 1_000) as f32 / 500.0) - 0.999)
                .collect()
        };
        let x = Tensor::from_vec(wave(batch * time * width, 0), (batch, time, width), &device)
            .unwrap();
        let table = Tensor::from_vec(wave(vocab * width, 7), (vocab, width), &device).unwrap();
        let ids: Vec<u32> = (0..batch * time).map(|i| ((i * 5 + i / 7) % vocab) as u32).collect();
        let turned = |q: [f32; 4]| Tensor::from_vec(q.repeat(width / 4), (width / 4, 4), &device);
        let reads = HammingReads::new(4, width, &device).unwrap();
        let mut hash = Sha256::new();
        for read in &reads.reads {
            read.key_rotation.set(&turned([0.6, 0.8, 0.0, 0.0]).unwrap()).unwrap();
            read.query_rotation.set(&turned([0.8, 0.0, 0.6, 0.0]).unwrap()).unwrap();
            read.value_rotation.set(&turned([0.0, 0.6, 0.0, 0.8]).unwrap()).unwrap();
            read.weight
                .set(&Tensor::from_vec(wave(2 * width, 3), (2, width), &device).unwrap())
                .unwrap();
            for out in [
                read.apply(&x, &ids, &table).unwrap(),
                read.apply_readout(&x, &ids, &table).unwrap(),
            ] {
                for v in out.flatten_all().unwrap().to_vec1::<f32>().unwrap() {
                    hash.update(v.to_le_bytes());
                }
            }
        }
        println!("I1-DIGEST {}", hex::encode(hash.finalize()));
    }

    /// claude-333's I-1: the same forward with the ask heads on and no hold, the ask rows set off
    /// their zero start. The base serves `with_ask` already (claude-330), so the two match.
    #[test]
    fn ask_no_hold_forward_digest() {
        let device = Device::new_cuda(0).unwrap();
        let (batch, time, width, vocab) = (2, 24, 256, 16);
        let wave = |n: usize, salt: usize| -> Vec<f32> {
            (0..n)
                .map(|i| ((((i + salt) * 2_654_435_761) % 1_000) as f32 / 500.0) - 0.999)
                .collect()
        };
        let x = Tensor::from_vec(wave(batch * time * width, 0), (batch, time, width), &device)
            .unwrap();
        let table = Tensor::from_vec(wave(vocab * width, 7), (vocab, width), &device).unwrap();
        let ids: Vec<u32> = (0..batch * time).map(|i| ((i * 5 + i / 7) % vocab) as u32).collect();
        let turned = |q: [f32; 4]| Tensor::from_vec(q.repeat(width / 4), (width / 4, 4), &device);
        let reads = HammingReads::new(4, width, &device).unwrap().with_ask(vocab).unwrap();
        let mut hash = Sha256::new();
        for read in &reads.reads {
            read.key_rotation.set(&turned([0.6, 0.8, 0.0, 0.0]).unwrap()).unwrap();
            read.query_rotation.set(&turned([0.8, 0.0, 0.6, 0.0]).unwrap()).unwrap();
            read.value_rotation.set(&turned([0.0, 0.6, 0.0, 0.8]).unwrap()).unwrap();
            read.weight
                .set(&Tensor::from_vec(wave(2 * width, 3), (2, width), &device).unwrap())
                .unwrap();
            let rows: Vec<f32> = wave(vocab * width, 5).iter().map(|v| v * 0.05).collect();
            read.ask
                .as_ref()
                .unwrap()
                .set(&Tensor::from_vec(rows, (vocab, width), &device).unwrap())
                .unwrap();
            for out in [
                read.apply(&x, &ids, &table).unwrap(),
                read.apply_readout(&x, &ids, &table).unwrap(),
            ] {
                for v in out.flatten_all().unwrap().to_vec1::<f32>().unwrap() {
                    hash.update(v.to_le_bytes());
                }
            }
        }
        println!("I1-ASK-DIGEST {}", hex::encode(hash.finalize()));
    }

    /// claude-333's I-1: a no-read forward, the six-layer width-256 trunk the B arms train, from
    /// its seeded start, with no Hamming read set: what lets I-R1 reuse claude-330's sealed B1-n.
    #[test]
    fn no_read_forward_digest() {
        use crate::geometric_stack::{
            Gate, Objective, QuaternionWeights, RecGates, StackArch, StackConfig, StackModel,
        };
        let device = Device::new_cuda(0).unwrap();
        set_hamming_reads(None);
        let mut config = StackConfig::transformer_control(278);
        config.arch = StackArch::Geometric;
        config.pattern = "rrrrrr".into();
        config.width = 256;
        config.heads = 4;
        config.mlp_hidden = 384;
        config.context = 24;
        config.vocab_size = 4096;
        config.rotation = true;
        config.gate = Gate::Hamilton;
        config.rec_gates = RecGates::Geometric;
        config.objective = Objective::MarginHinge { rivals: 1 };
        let mut model = StackModel::new(config, &device).unwrap();
        model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs).unwrap();
        model.set_native_vector_head(8).unwrap();
        let ids: Vec<u32> = (0..48).map(|i| ((i * 37 + i / 5) % 4096) as u32).collect();
        let out = model.forward(&ids, 2, 24).unwrap();
        let mut hash = Sha256::new();
        for v in out.flatten_all().unwrap().to_vec1::<f32>().unwrap() {
            hash.update(v.to_le_bytes());
        }
        println!("I1-NO-READ-DIGEST {}", hex::encode(hash.finalize()));
    }
}
// I1-DIGEST END
