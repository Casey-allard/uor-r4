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
    static CUT: std::cell::Cell<Option<(ReadScope, usize)>> = const { std::cell::Cell::new(None) };
    static ROW_CUT: RefCell<Option<(ReadScope, Vec<usize>)>> = const { RefCell::new(None) };
    static TIE: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static TRACE: RefCell<Option<Vec<HammingTraceWindow>>> = const { RefCell::new(None) };
    static COORDS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static CODE_FORCE: RefCell<Vec<ForcedRowCode>> = const { RefCell::new(Vec::new()) };
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
}

impl ReadScope {
    /// The name a samples file carries for it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Copy => "copy",
            Self::All => "all",
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

/// Whether the scope limits head `head`, the first `copy_heads` heads being copy heads.
fn head_limited(scope: ReadScope, head: usize, copy_heads: usize) -> bool {
    match scope {
        ReadScope::Full => false,
        ReadScope::Copy => head < copy_heads,
        ReadScope::All => true,
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
        _ => time,
    }
}

/// Whether position `s` is a candidate for position `t`: from `first`, earlier than `t`, and
/// before `limit` ([`head_limit`]).
fn is_candidate(s: usize, t: usize, first: usize, limit: usize) -> bool {
    s >= first && s < t && s < limit
}

/// Make `reads` the Hamming reads `StackModel::hidden` applies on this thread, or none.
pub fn set_hamming_reads(reads: Option<HammingReads>) {
    ACTIVE.with(|cell| *cell.borrow_mut() = reads);
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
    /// How many tokens before a position a copy head's key names: 1, or 2 for the ordered
    /// two-token key ([`HammingReads::with_copy_key`], claude-261).
    copy_key: usize,
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

/// `first`, `(rows, width)`, with the second half of each of the first `heads` heads of
/// `head_width` columns taken from `second`: copy key 2's codes, one token in each half.
fn pair_halves(first: &Tensor, second: &Tensor, heads: usize, head_width: usize) -> Result<Tensor> {
    let (half, width) = (head_width / 2, first.dim(1)?);
    let mut parts = Vec::with_capacity(2 * heads + 1);
    for h in 0..heads {
        parts.push(first.narrow(1, h * head_width, half)?);
        parts.push(second.narrow(1, h * head_width + half, half)?);
    }
    if heads * head_width < width {
        parts.push(first.narrow(1, heads * head_width, width - heads * head_width)?);
    }
    Ok(Tensor::cat(&parts, 1)?)
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

    /// The same reads with their copy heads keyed on `copy_key` tokens before a position: 1, the
    /// default, or 2, the ordered two-token key (claude-261). Type heads are untouched.
    pub fn with_copy_key(mut self, copy_key: usize) -> Result<Self> {
        for read in &mut self.reads {
            check_copy_key(copy_key, read.heads, read.weight.dim(1)?)?;
            read.copy_key = copy_key;
        }
        Ok(self)
    }

    /// How many tokens before a position the copy heads' key names (1 or 2).
    pub fn copy_key(&self) -> usize {
        self.reads.first().map_or(1, |r| r.copy_key)
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
        // Only a two-token key is recorded, so a one-token read's file is byte for byte as before.
        if self.copy_key() != 1 {
            let copy_key = self.copy_key() as u32;
            tensors.insert("copy_key".to_owned(), Tensor::new(&[copy_key], device)?);
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
                copy_key: 1,
            });
        }
        Ok(Some(Self { reads }.with_copy_key(copy_key)?))
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
            copy_key: 1,
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
        named
    }

    fn variables(&self) -> Vec<&Var> {
        self.named().into_iter().map(|(_, var)| var).collect()
    }

    /// `points` rows rotated lane by lane by `rotation`'s unit quaternions.
    fn rotate(rotation: &Var, points: &Tensor) -> Result<Tensor> {
        let gate = rotation
            .as_tensor()
            .reshape((1, points.dim(1)?))?
            .broadcast_as(points.shape())?
            .contiguous()?;
        hamilton_gate(&gate, points)
    }

    /// The first half of the heads are copy heads; the rest are type heads.
    fn copy_heads(&self) -> usize {
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
        self.read(residual, ids, table, false)
    }

    /// The read applied to the final norm's output `(batch, time, width)`, the points the
    /// nearest readout decodes: where a copy head found an earlier copy of the current token,
    /// the prediction moves toward the copied point, `n + w_c ⊙ (c − n) + w_y ⊙ y`, and stays
    /// `n + w_y ⊙ y` where none did. At `w_c = 1` a found copy is decoded exactly. A nearest
    /// readout names the point a prediction is at, so a copy is a move, not an added term.
    pub fn apply_readout(&self, output: &Tensor, ids: &[u32], table: &Tensor) -> Result<Tensor> {
        self.read(output, ids, table, true)
    }

    fn read(
        &self,
        residual: &Tensor,
        ids: &[u32],
        table: &Tensor,
        readout: bool,
    ) -> Result<Tensor> {
        let (batch, time, width) = residual.dims3()?;
        let rows = batch * time;
        if ids.len() != rows {
            return Err(invalid("the Hamming read needs whole windows"));
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
        let before = match self.copy_key {
            2 => Some(token_points(two_back(ids, time))?),
            _ => None,
        };
        let (query, key, value) = as_trained(|| {
            // Copy heads ask with the current token's point through the key's own rotation, so a
            // repeated token stays an exact match however that rotation learns; type heads ask
            // with the residual through the query's. Rotations act lane by lane, so each half is
            // cut after.
            let copy_asks = Self::rotate(&self.key_rotation, &current)?;
            let type_asks = Self::rotate(&self.query_rotation, &points)?;
            let key = Self::rotate(&self.key_rotation, &previous)?;
            // With copy key 2, a copy head asks with `x_t`'s half and then `x_{t−1}`'s, and its
            // key is `x_{s−1}`'s half and then `x_{s−2}`'s, every half through the key's rotation,
            // so a repeat of both tokens is still an exact match however it learns.
            let (copy_asks, key) = match &before {
                None => (copy_asks, key),
                Some(before) => {
                    let before = Self::rotate(&self.key_rotation, before)?;
                    let copy = self.copy_heads();
                    (
                        pair_halves(&copy_asks, &key, copy, head_width)?,
                        pair_halves(&key, &before, copy, head_width)?,
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
        // What each kind of head brings back, at full width: a copy head the table's point of the
        // kept position's own token, the readout's own point for it (the 26 Sep head's tied
        // readout); a type head the kept position's rotated residual.
        let sources = [
            current.reshape((batch, time, width))?,
            value.reshape((batch, time, width))?,
        ];
        let mut sums = [
            Tensor::zeros((rows, width), DType::F32, device)?,
            Tensor::zeros((rows, width), DType::F32, device)?,
        ];
        // How many heads of each kind found any candidate inside their margin, per row.
        let mut found = [
            Tensor::zeros((rows, 1), DType::F32, device)?,
            Tensor::zeros((rows, 1), DType::F32, device)?,
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
            let limits = row_limits(cut, row_cut.as_ref(), h, self.copy_heads(), time, rows);
            let (picks, real) =
                choose_sorted(&q, &k, batch, time, HAMMING_KEPT, 1, &limits, tie)?;
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
                    .reshape((batch, time * kept, 1))?
                    .broadcast_as((batch, time * kept, columns))?
                    .contiguous()?)
            };
            let kept_keys = k
                .reshape((batch, time, head_width))?
                .gather(&index(head_width)?, 1)?
                .reshape((rows, kept, head_width))?;
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
                .reshape((rows, kept, width))?;
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
        let (copy_weight, type_weight) = (weight.narrow(0, 0, 1)?, weight.narrow(0, 1, 1)?);
        let copied = if readout {
            // Only where a copy head found an earlier copy does the prediction move.
            let matched = found[0].gt(0.0)?.to_dtype(DType::F32)?;
            means[0].sub(&points)?.broadcast_mul(&matched)?
        } else {
            means[0].clone()
        };
        let added = copied
            .broadcast_mul(&copy_weight)?
            .add(&means[1].broadcast_mul(&type_weight)?)?;
        Ok(points.add(&added)?.reshape((batch, time, width))?)
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
    let device = query.device();
    let kept = neighbours.min(time.saturating_sub(1)).max(1);
    let width = query.dim(1)?;
    let rows = batch * time;
    let q = query.detach().reshape((batch, time, 1, width))?;
    let k = key.detach().reshape((batch, 1, time, width))?;
    let distance = q.broadcast_sub(&k)?.abs()?.mean(3)?.reshape((rows, time))?;
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
    let earlier: Vec<u8> = (0..rows * time)
        .map(|i| u8::from(is_candidate(i % time, (i / time) % time, first, limits[i / time])))
        .collect();
    let earlier = Tensor::from_vec(earlier, (rows, time), device)?;
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
        let (heads, head_width, width) = (2, 16, 40);
        let first = Tensor::full(1f32, (3, width), &device).unwrap();
        let second = Tensor::full(2f32, (3, width), &device).unwrap();
        let paired = pair_halves(&first, &second, heads, head_width)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        let want: Vec<f32> = (0..width)
            .map(|c| {
                let high = c < heads * head_width && c % head_width >= head_width / 2;
                if high { 2.0 } else { 1.0 }
            })
            .collect();
        for row in paired {
            assert_eq!(row, want);
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
        }
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
        let added = |copy_key: usize| -> Vec<f32> {
            let reads = HammingReads::new(4, 256, &device)
                .unwrap()
                .with_copy_key(copy_key)
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
        let (one, two) = (added(1), added(2));
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
}
