//! Standalone integer flock selector for sparse causal attention awaiting integration.
//!
//! Multiplier-free and allocation-free causal flock selection structure under owner decision
//! D11 and Council D16 (Rule 5 & Item 16).
//!
//! Flock attention keeps for each query row:
//! 1. A **sink** position (e.g. position 0, always preserved).
//! 2. A **local window** of the nearest preceding positions (e.g. 64 positions).
//! 3. The **exact top-k** of the remaining causally visible positions (e.g. k in {1, 7, 16, 64}).
//!
//! This module provides:
//! - [`FlockScratch`]: Reusable workspace preallocated at session initialization,
//!   ensuring strictly zero heap allocations per token step when prepared with sufficient capacity.
//! - In-place bounded-insertion top-k over the non-window past and an insertion ordering of the
//!   support: comparisons and moves only, O(n k + s^2) (no library sort, so no multiply instruction).
//! - Strict deduplicated support `{sink} ∪ window ∪ top_k` with deterministic lowest-position
//!   tie breaking.
//! - Rank-weight conversion helpers using precomputed tables and restoring division. Score formation
//!   (e.g. Minkowski / dot product) and softmax/weight application are external consumer responsibilities.
//! - Top-k-only entrypoint (`top_k_select_integer`) for single-pointer (k=1) retrieval.

use crate::stack::kernels::stack_div_u128;
use crate::{invalid, Result};

/// Version tag binding the integer flock selector contract.
pub const FLOCK_INTEGER_SELECTOR_VERSION: &str = "flock-integer-selector-v1";

/// Maximum supported context positions in a single session scratch buffer.
pub const MAX_FLOCK_CONTEXT: usize = 4096;

/// Role of a selected position inside the flock support.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlockSlot {
    /// The sink position, always included.
    Sink,
    /// One of the nearest `window` causally visible positions.
    Window,
    /// An exact top-k position outside the sink and window.
    TopK,
}

/// One selected position and its admitting role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlockEntry {
    pub position: usize,
    pub slot: FlockSlot,
}

/// Selection parameters for one query step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlockSelect {
    /// Always-selected anchor position (typically 0).
    pub sink: usize,
    /// Nearest preceding positions always included.
    pub window: usize,
    /// Exact top-k of the remaining causally visible positions.
    pub k: usize,
}

impl FlockSelect {
    pub fn new(sink: usize, window: usize, k: usize) -> Self {
        Self { sink, window, k }
    }

    /// Standard B0 / A1 canonical configuration: sink at 0, window 64, top-k.
    pub fn b0(k: usize) -> Self {
        Self {
            sink: 0,
            window: 64,
            k,
        }
    }
}

/// Exact accounting counters for one selection step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FlockScan {
    /// Causally visible positions (`query + 1`).
    pub visible: usize,
    /// Kept entries in sink role (0 or 1).
    pub sink_kept: usize,
    /// Kept entries in window role.
    pub window_kept: usize,
    /// Kept entries in top-k role.
    pub top_k_kept: usize,
    /// Rest positions compared for top-k selection.
    pub candidates_scanned: usize,
    /// Whole visible prefix fits inside nominal window.
    pub short_prefix: bool,
    /// Fewer than `k` rest candidates existed.
    pub top_k_short: bool,
    /// The k-th and (k+1)-th candidates have equal rank scores (tie broken by position).
    pub cutoff_ties: bool,
}

/// Pre-allocated workspace for allocation-free flock selection.
///
/// Allocated once per session or layer. Calling [`FlockScratch::reset`]
/// resets length counters without releasing or reallocating memory.
#[derive(Clone, Debug)]
pub struct FlockScratch {
    /// Per-position slot tags, sized to maximum context.
    slots: Vec<Option<FlockSlot>>,
    /// Indices of non-window, non-sink positions for top-k partitioning.
    rest: Vec<usize>,
    /// Kept entries ordered by descending rank score (lowest position on ties).
    pub entries: Vec<FlockEntry>,
}

impl FlockScratch {
    /// Create a scratch workspace with capacity for at least `capacity` context positions.
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(129).min(MAX_FLOCK_CONTEXT);
        Self {
            slots: vec![None; cap],
            rest: Vec::with_capacity(cap),
            entries: Vec::with_capacity(cap),
        }
    }

    /// Internal capacity of the slots buffer.
    #[inline(always)]
    pub fn slots_capacity(&self) -> usize {
        self.slots.capacity()
    }

    /// Internal capacity of the rest candidate buffer.
    #[inline(always)]
    pub fn rest_capacity(&self) -> usize {
        self.rest.capacity()
    }

    /// Internal capacity of the selected entries buffer.
    #[inline(always)]
    pub fn entries_capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// Ensure capacity for at least `needed` positions.
    pub fn ensure_capacity(&mut self, needed: usize) {
        let bound = needed.min(MAX_FLOCK_CONTEXT);
        if self.slots.len() < bound {
            self.slots.resize(bound, None);
        }
        if self.rest.capacity() < bound {
            self.rest.reserve(bound - self.rest.len());
        }
        if self.entries.capacity() < bound {
            self.entries.reserve(bound - self.entries.len());
        }
    }

    /// Reset internal buffers for a query at `query`.
    #[inline(always)]
    fn prepare(&mut self, query: usize) {
        self.ensure_capacity(query + 1);
        self.slots[..=query].fill(None);
        self.rest.clear();
        self.entries.clear();
    }
}

/// Exact integer flock selection over pre-formed integer `scores[0..=query]`.
///
/// Scores are external pre-formed integer rank scores. Higher scores rank nearer.
///
/// Returns the selection accounting [`FlockScan`]. Selected entries are written into
/// `scratch.entries` in descending rank order with ties broken to the lowest position.
///
/// Complexity: O(n k + s^2) where n is candidate count, k is top-k, and s is final support size (bounded insertion, no library sort).
///
/// Algorithm:
/// - Deduplicated support `{sink} ∪ window ∪ top_k`.
/// - Sink wins if inside window (deduplication).
/// - Non-sink, non-window rest positions keep their top-k by bounded insertion.
/// - Final support entries are sorted in descending rank order.
/// - Strictly zero heap allocations when `scratch` has sufficient capacity.
#[inline(never)]
pub fn flock_select_integer(
    scores: &[i64],
    query: usize,
    select: FlockSelect,
    scratch: &mut FlockScratch,
) -> Result<FlockScan> {
    if query >= MAX_FLOCK_CONTEXT {
        return Err(invalid(format!(
            "flock query position {query} exceeds MAX_FLOCK_CONTEXT {MAX_FLOCK_CONTEXT}"
        )));
    }
    if scores.len() <= query {
        return Err(invalid(format!(
            "flock scores length {} is too short for query {query}",
            scores.len()
        )));
    }
    if select.window == 0 {
        return Err(invalid("flock window must be positive"));
    }
    if select.k == 0 {
        return Err(invalid("flock k must be positive"));
    }
    if select.sink > query {
        return Err(invalid("flock sink must be causally visible"));
    }

    scratch.prepare(query);

    let window_start = (query + 1).saturating_sub(select.window);
    for slot in scratch.slots[window_start..=query].iter_mut() {
        *slot = Some(FlockSlot::Window);
    }
    // Sink takes precedence if it falls inside the window span
    scratch.slots[select.sink] = Some(FlockSlot::Sink);

    // Collect rest positions (prior to window_start, excluding sink)
    for pos in 0..window_start {
        if scratch.slots[pos].is_none() {
            scratch.rest.push(pos);
        }
    }

    let candidates_scanned = scratch.rest.len();
    let short_prefix = (query + 1) <= select.window;
    let top_k_short = candidates_scanned < select.k;

    // The k best rest candidates, in rank order, by bounded insertion.
    let (top_k_count, cutoff_ties) = top_k_insertion(scores, &mut scratch.rest, select.k);

    for &pos in &scratch.rest[..top_k_count] {
        scratch.slots[pos] = Some(FlockSlot::TopK);
    }

    // Materialize entries
    let mut sink_kept = 0usize;
    let mut window_kept = 0usize;
    let mut top_k_kept = 0usize;

    for (pos, &slot_opt) in scratch.slots[..=query].iter().enumerate() {
        if let Some(slot) = slot_opt {
            scratch.entries.push(FlockEntry {
                position: pos,
                slot,
            });
            match slot {
                FlockSlot::Sink => sink_kept += 1,
                FlockSlot::Window => window_kept += 1,
                FlockSlot::TopK => top_k_kept += 1,
            }
        }
    }

    // Final kept support in descending rank order with lowest position on ties.
    insertion_order_entries(scores, &mut scratch.entries);

    Ok(FlockScan {
        visible: query + 1,
        sink_kept,
        window_kept,
        top_k_kept,
        candidates_scanned,
        short_prefix,
        top_k_short,
        cutoff_ties,
    })
}

/// Whether position `a` ranks before position `b`: the higher score, then the
/// lower position.
#[inline(always)]
fn ranks_before(scores: &[i64], a: usize, b: usize) -> bool {
    scores[a] > scores[b] || (scores[a] == scores[b] && a < b)
}

/// Leaves the `k` best of `rest` (positions, any order) at `rest[..count]` in
/// rank order and returns `count = min(k, len)` and whether the cutoff is a
/// tie (more than `k` candidates and the `k`-th kept score equals the best
/// score left out). Bounded insertion: comparisons and moves only, so the
/// served kernel has no library sort (whose index arithmetic compiles to
/// multiply instructions) and no allocation. `O(n k)` for `n` candidates.
fn top_k_insertion(scores: &[i64], rest: &mut [usize], k: usize) -> (usize, bool) {
    let n = rest.len();
    let mut kept = 0usize;
    let mut best_out: Option<i64> = None;
    for i in 0..n {
        let x = rest[i];
        if kept < k {
            // `i == kept` here: every earlier candidate was kept.
            let mut j = kept;
            while j > 0 && ranks_before(scores, x, rest[j - 1]) {
                rest[j] = rest[j - 1];
                j -= 1;
            }
            rest[j] = x;
            kept += 1;
        } else if ranks_before(scores, x, rest[k - 1]) {
            let evicted = rest[k - 1];
            let mut j = k - 1;
            while j > 0 && ranks_before(scores, x, rest[j - 1]) {
                rest[j] = rest[j - 1];
                j -= 1;
            }
            rest[j] = x;
            best_out = Some(best_out.map_or(scores[evicted], |b| b.max(scores[evicted])));
        } else {
            best_out = Some(best_out.map_or(scores[x], |b| b.max(scores[x])));
        }
    }
    let cutoff_ties = kept == k && n > k && best_out == Some(scores[rest[k - 1]]);
    (kept, cutoff_ties)
}

/// Orders `entries` by rank (higher score, then lower position) by insertion:
/// comparisons and moves only.
fn insertion_order_entries(scores: &[i64], entries: &mut [FlockEntry]) {
    for i in 1..entries.len() {
        let x = entries[i];
        let mut j = i;
        while j > 0 && ranks_before(scores, x.position, entries[j - 1].position) {
            entries[j] = entries[j - 1];
            j -= 1;
        }
        entries[j] = x;
    }
}

/// Exact top-k only selection over `scores[0..=query]`.
///
/// Specialized entrypoint for pointer retrieval (e.g. k=1) without sink and without window.
#[inline(never)]
pub fn top_k_select_integer(
    scores: &[i64],
    query: usize,
    k: usize,
    scratch: &mut FlockScratch,
) -> Result<FlockScan> {
    if query >= MAX_FLOCK_CONTEXT {
        return Err(invalid(format!(
            "flock query position {query} exceeds MAX_FLOCK_CONTEXT {MAX_FLOCK_CONTEXT}"
        )));
    }
    if scores.len() <= query {
        return Err(invalid("scores length is too short for query"));
    }
    if k == 0 {
        return Err(invalid("k must be positive"));
    }

    scratch.prepare(query);
    for pos in 0..=query {
        scratch.rest.push(pos);
    }

    let candidates_scanned = scratch.rest.len();
    let top_k_short = candidates_scanned < k;

    let (top_k_count, cutoff_ties) = top_k_insertion(scores, &mut scratch.rest, k);

    for &pos in &scratch.rest[..top_k_count] {
        scratch.entries.push(FlockEntry {
            position: pos,
            slot: FlockSlot::TopK,
        });
    }

    Ok(FlockScan {
        visible: query + 1,
        sink_kept: 0,
        window_kept: 0,
        top_k_kept: top_k_count,
        candidates_scanned,
        short_prefix: false,
        top_k_short,
        cutoff_ties,
    })
}

/// Precomputed raw unnormalized rank weights `1 / (i + 1)` in Q16 for support up to 129 entries.
///
/// Precomputed lookup table; avoids runtime division operations through direct table indexing.
pub const RAW_RANK_WEIGHTS_Q16: [u16; 129] = [
    65535, 32768, 21845, 16384, 13107, 10922, 9362, 8192, 7281, 6553, 5957, 5461, 5041, 4681, 4369,
    4096, 3855, 3640, 3449, 3276, 3120, 2978, 2849, 2730, 2621, 2520, 2427, 2340, 2259, 2184, 2114,
    2048, 1985, 1927, 1872, 1820, 1771, 1724, 1680, 1638, 1598, 1560, 1524, 1489, 1456, 1424, 1394,
    1365, 1337, 1310, 1285, 1260, 1236, 1213, 1191, 1170, 1149, 1129, 1110, 1092, 1074, 1057, 1040,
    1024, 1008, 992, 978, 963, 949, 936, 923, 910, 897, 885, 873, 862, 851, 840, 829, 819, 809,
    799, 789, 780, 771, 762, 753, 744, 736, 728, 720, 712, 704, 697, 689, 682, 675, 668, 661, 655,
    648, 642, 636, 630, 624, 618, 612, 606, 601, 595, 590, 585, 579, 574, 569, 564, 560, 555, 550,
    546, 541, 537, 532, 528, 524, 520, 516, 512, 508,
];

/// Precomputed Q32 reciprocals `2^32 / (i + 1)` for harmonic sums up to 129 entries.
///
/// Precomputed lookup table; avoids runtime division operations through direct table indexing.
pub const RECIPROCAL_Q32: [u64; 129] = [
    4294967296, 2147483648, 1431655765, 1073741824, 858993459, 715827882, 613566756, 536870912,
    477218588, 429496729, 390451572, 357913941, 330382099, 306783378, 286331153, 268435456,
    252645135, 238609294, 226050910, 214748364, 204522252, 195225786, 186737708, 178956970,
    171798691, 165191049, 159072862, 153391689, 148102320, 143165576, 138547332, 134217728,
    130150524, 126322567, 122713351, 119304647, 116080197, 113025455, 110127366, 107374182,
    104755299, 102261126, 99882960, 97612893, 95443717, 93368854, 91382282, 89478485, 87652393,
    85899345, 84215045, 82595524, 81037118, 79536431, 78090314, 76695844, 75350303, 74051160,
    72796055, 71582788, 70409299, 69273666, 68174084, 67108864, 66076419, 65075262, 64103989,
    63161283, 62245902, 61356675, 60492497, 59652323, 58835168, 58040098, 57266230, 56512727,
    55778796, 55063683, 54366674, 53687091, 53024287, 52377649, 51746593, 51130563, 50529027,
    49941480, 49367440, 48806446, 48258059, 47721858, 47197442, 46684427, 46182444, 45691141,
    45210182, 44739242, 44278013, 43826196, 43383508, 42949672, 42524428, 42107522, 41698711,
    41297762, 40904450, 40518559, 40139881, 39768215, 39403369, 39045157, 38693399, 38347922,
    38008560, 37675151, 37347541, 37025580, 36709122, 36398027, 36092162, 35791394, 35495597,
    35204649, 34918433, 34636833, 34359738, 34087042, 33818640, 33554432, 33294320,
];

/// Normalized fixed rank table weights: `w_i ∝ 1 / (i + 1)` in Q31.
///
/// Sums to `(1 << 31) - 1` (within roundoff). Uses precomputed reciprocal tables
/// for support up to 129 and software restoring long division (`stack_div_u128`,
/// a shift-and-subtract loop) for normalization.
#[inline(never)]
pub fn rank_table_q31(count: usize, out: &mut [u32]) -> Result<()> {
    if out.len() < count {
        return Err(invalid("output slice is smaller than count"));
    }
    if count == 0 {
        return Ok(());
    }
    if count == 1 {
        out[0] = 0x7FFF_FFFF; // 1.0 in Q31
        return Ok(());
    }

    // Compute harmonic sum in Q32 using table lookups and scalar additions
    let mut sum_q32 = 0u128;
    for i in 0..count {
        let term = if i < RECIPROCAL_Q32.len() {
            RECIPROCAL_Q32[i] as u128
        } else {
            stack_div_u128(1u128 << 32, (i as u128) + 1)
        };
        sum_q32 += term;
    }

    // Normalize each term to Q31: w_i = (term / sum_q32) * 2^31
    // (term * 2^63) / sum_q32 via software restoring division (shift-and-subtract loop)
    for (i, slot) in out[..count].iter_mut().enumerate() {
        let term = if i < RECIPROCAL_Q32.len() {
            RECIPROCAL_Q32[i] as u128
        } else {
            stack_div_u128(1u128 << 32, (i as u128) + 1)
        };
        let num = term << 31;
        let w = stack_div_u128(num, sum_q32);
        *slot = (w as u32).min(0x7FFF_FFFF);
    }

    Ok(())
}

/// Raw unnormalized rank weights: `a_i = 1 / (i + 1)` in Q16.
///
/// Unnormalized B2 hybrid scale using direct table lookup and software division fallback.
#[inline(never)]
pub fn raw_rank_weights_q16(count: usize, out: &mut [u16]) -> Result<()> {
    if out.len() < count {
        return Err(invalid("output slice is smaller than count"));
    }
    for (i, slot) in out[..count].iter_mut().enumerate() {
        if i < RAW_RANK_WEIGHTS_Q16.len() {
            *slot = RAW_RANK_WEIGHTS_Q16[i];
        } else {
            let term = stack_div_u128(1u128 << 16, (i as u128) + 1);
            *slot = term.min(0xFFFF) as u16;
        }
    }
    Ok(())
}
