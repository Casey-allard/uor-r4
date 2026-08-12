//! Packed R4G1 lowering of `R4RouteAttentionV1` (#604) — the deployed-
//! class implementation of the first target route-attention operator,
//! over borrowed bytes and caller-owned bounded state.
//!
//! DORMANT: registered `open` in model/ledger.toml as
//! `r4-route-attention-dormant`. This kernel is constructible and
//! testable but referenced by NO serving path; `packed-routing-dormant`
//! stays unchanged and no serving default selects this operator. The
//! reference semantics, witness format, and independent replayer live in
//! `uor-r4-graph-certify::route_attention`; the canonical instance wire
//! layout, bounds, and census vocabulary in
//! `uor-r4-graph-format::route_attention`. The differential tests there
//! require this kernel to agree with the reference bit-for-bit on
//! selections, aggregates, and the whole op census.
//!
//! This file is P-4-scanned (`uor-r4-core::transformerless::mod.rs`,
//! `p4_contract_owned_graph_runtime_source_scan`): no value `*` `/` `%`
//! may appear here. Every operation is in the deployed contract's
//! allowed classes — XOR, AND, popcount-table read, integer add/sub
//! (saturating for ScoreQ), integer compare, table reads, and
//! constant-stride addressing lowered by hand to shift/add
//! (`4*i` is `i << 2`; candidate walks use `chunks_exact`).
//!
//! One step over an instance with `N` candidates selecting `M`:
//!
//! 1. relation: per candidate, per byte, `popcount((q ^ c) & mask)`
//!    accumulated by integer adds — the masked XOR+popcount distance;
//! 2. selection: bounded top-M by ascending `(distance, index)` with
//!    EXACTLY `M` ordered slot comparisons per candidate (the count is
//!    data-independent, so the census is a closed form of `(N, M)` and
//!    replay-verifiable without running the kernel); ties on equal
//!    distance resolve to the LOWEST candidate index by construction —
//!    candidates arrive in ascending index order and an equal-distance
//!    later index never displaces a slot;
//! 3. aggregation: the selected contributions fold in selection order
//!    (ascending `(distance, index)`) from `ScoreQ::ZERO` with
//!    saturating adds.
//!
//! Steady state allocates nothing: the instance is a borrowed
//! [`RouteAttentionView`], the query a borrowed route code, and every
//! scratch slot lives in the caller-owned fixed-capacity [`RouteState`]
//! (the `StepState` epoch discipline from the certify-side reference
//! scorer: the state is built once, each step advances `epoch`, and the
//! selection slots are valid only for the current epoch).

use uor_r4_graph_format::route_attention::{
    ROUTE_CODE_BYTES, ROUTE_MAX_TOP_M, ROUTE_POPCOUNT_TABLE, RouteAttentionView, RouteOpCensus,
};
use uor_r4_graph_format::{FormatError, NotAProduct, ObjectKind, ScoreQ};

/// Bytes fetched per candidate from the instance's borrowed regions:
/// one code window plus one mask window.
const CODE_AND_MASK_BYTES: u64 = (ROUTE_CODE_BYTES as u64) + (ROUTE_CODE_BYTES as u64);

/// Distance sentinel of an unfilled selection slot. Real distances are
/// at most 288 (the route-code bit width), so the sentinel never
/// collides with a real candidate.
const EMPTY_DISTANCE: u32 = u32::MAX;
/// Index sentinel of an unfilled selection slot. Real indices are below
/// the candidate cap (64).
const EMPTY_INDEX: u32 = u32::MAX;

/// Caller-owned bounded step state: fixed-capacity selection slots
/// (capacity [`ROUTE_MAX_TOP_M`] — the state capacity bound holds by
/// construction, and every instance's validated `top_m` fits it), an
/// epoch stamp, and the current epoch's selection width. Built once,
/// reused across steps; [`route_attention_step`] performs no allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteState {
    /// Ascending selection-order distances; slots past
    /// `selected_len` hold the sentinel.
    dist: [u32; ROUTE_MAX_TOP_M],
    /// Candidate indices aligned with `dist`.
    cand: [u32; ROUTE_MAX_TOP_M],
    /// Number of valid selection slots for the current epoch.
    selected_len: usize,
    /// Step stamp: advanced once per step; the selection slots are
    /// valid only for the epoch they were written in.
    epoch: u64,
}

impl RouteState {
    /// Fresh state with no valid epoch.
    pub const fn new() -> Self {
        Self {
            dist: [EMPTY_DISTANCE; ROUTE_MAX_TOP_M],
            cand: [EMPTY_INDEX; ROUTE_MAX_TOP_M],
            selected_len: 0,
            epoch: 0,
        }
    }

    /// The current epoch stamp (0 before the first step).
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Number of selection slots valid for the current epoch.
    pub const fn selected_len(&self) -> usize {
        self.selected_len
    }

    /// Selection slot `slot` of the current epoch as
    /// `(candidate index, masked popcount distance)`, in selection order
    /// (ascending distance, ties ascending index); `None` past the
    /// current selection width.
    pub fn selected(&self, slot: usize) -> Option<(u32, u32)> {
        if slot < self.selected_len {
            Some((self.cand[slot], self.dist[slot]))
        } else {
            None
        }
    }
}

impl Default for RouteState {
    fn default() -> Self {
        Self::new()
    }
}

/// One packed route-attention step: masked XOR+popcount relation over
/// every candidate, bounded top-M selection with the lowest-index tie
/// rule, and selection-order saturating ScoreQ aggregation. Writes the
/// selection into `state` (readable via [`RouteState::selected`]),
/// increments `census`, and returns the aggregate.
///
/// Total verdict on the sanctioned surface: the only refusal is a query
/// that is not one route-code width ([`FormatError::RouteQueryWidthMismatch`]
/// under [`ObjectKind::RouteAttentionStep`]); every instance-shape and
/// bound violation was already refused by [`RouteAttentionView::parse`].
pub fn route_attention_step(
    view: &RouteAttentionView<'_>,
    query: &[u8],
    state: &mut RouteState,
    census: &mut RouteOpCensus,
) -> Result<ScoreQ, NotAProduct> {
    if query.len() != ROUTE_CODE_BYTES {
        return Err(NotAProduct::new(
            ObjectKind::RouteAttentionStep,
            FormatError::RouteQueryWidthMismatch {
                actual: query.len() as u64,
            },
        ));
    }
    state.epoch = state.epoch.wrapping_add(1);
    let top_m = usize::from(view.top_m());
    // Reset exactly the slots this instance uses (top_m <= capacity by
    // parse-time validation); a tiny fixed loop, no allocation.
    let mut slot = 0usize;
    while slot < top_m {
        state.dist[slot] = EMPTY_DISTANCE;
        state.cand[slot] = EMPTY_INDEX;
        slot += 1;
    }
    state.selected_len = top_m;

    let mask = view.mask();
    for (index, code) in (0_u32..).zip(view.codes().chunks_exact(ROUTE_CODE_BYTES)) {
        census.candidates_examined += 1;
        census.table_reads += 2;
        census.bytes_read += CODE_AND_MASK_BYTES;

        // Masked XOR+popcount distance, accumulated per byte.
        let mut distance: u32 = 0;
        for (&code_byte, (&query_byte, &mask_byte)) in
            code.iter().zip(query.iter().zip(mask.iter()))
        {
            let xored = query_byte ^ code_byte;
            census.xors += 1;
            let masked = xored & mask_byte;
            let ones = ROUTE_POPCOUNT_TABLE[masked as usize];
            census.popcounts += 1;
            distance += u32::from(ones);
            census.adds += 1;
        }

        // Bounded top-M insertion: exactly `top_m` ordered slot
        // comparisons per candidate — the count never depends on the
        // data. `(distance, index)` beats a slot when its distance is
        // strictly lower, or equal with a lower index; since candidates
        // arrive in ascending index order, an equal-distance later
        // candidate never wins — ties go to the lowest index.
        let mut insert_at = top_m;
        let mut probe = 0usize;
        while probe < top_m {
            census.compares += 1;
            let beats = match distance.cmp(&state.dist[probe]) {
                core::cmp::Ordering::Less => true,
                core::cmp::Ordering::Greater => false,
                core::cmp::Ordering::Equal => index < state.cand[probe],
            };
            if beats && insert_at == top_m {
                insert_at = probe;
            }
            probe += 1;
        }
        if insert_at < top_m {
            let mut hole = top_m - 1;
            while hole > insert_at {
                state.dist[hole] = state.dist[hole - 1];
                state.cand[hole] = state.cand[hole - 1];
                hole -= 1;
            }
            state.dist[insert_at] = distance;
            state.cand[insert_at] = index;
        }
    }

    // Selection-order aggregation: fold the selected contributions from
    // ScoreQ::ZERO with saturating adds. Every slot holds a real
    // candidate (top_m <= candidate_count by parse-time validation, and
    // the first top_m candidates always fill the sentinel slots), so
    // the shift-addressed contribution window is in bounds by
    // construction.
    let contributions = view.contributions();
    let mut aggregate = ScoreQ::ZERO;
    let mut fold = 0usize;
    while fold < top_m {
        let offset = (state.cand[fold] as usize) << 2;
        let raw = i32::from_le_bytes([
            contributions[offset],
            contributions[offset + 1],
            contributions[offset + 2],
            contributions[offset + 3],
        ]);
        census.table_reads += 1;
        census.bytes_read += 4;
        aggregate = aggregate.saturating_add(ScoreQ::from_raw(raw));
        census.adds += 1;
        fold += 1;
    }
    Ok(aggregate)
}
