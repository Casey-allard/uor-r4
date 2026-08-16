//! Packed R4G1 lowering of `MsaStructuredSelectorV1` (#643) — the
//! deployed-class implementation of the second target attention
//! operator, alongside `r4-route-attention/1` (#604), over borrowed
//! bytes and caller-owned bounded state.
//!
//! DORMANT: registered `open` in `model/ledger.toml` as
//! `msa-structured-selector-dormant`, mirroring `r4-route-attention-dormant`.
//! This kernel is constructible and testable but referenced by NO
//! serving path; no serving default selects this operator. The
//! reference semantics, witness format, and independent replayer live
//! in `uor-r4-graph-certify::msa_selector`; the canonical instance wire
//! layout, bounds, and census vocabulary in
//! `uor-r4-graph-format::msa_selector`. The differential tests there
//! require this kernel to agree with the reference bit-for-bit on
//! selections, aggregates, and the whole op census.
//!
//! This file is P-4-scanned (`uor-r4-core::transformerless::mod.rs`,
//! `p4_contract_owned_graph_runtime_source_scan`): no value `*` `/` `%`
//! may appear here. Every operation is in the deployed contract's
//! allowed classes — table reads, integer add (saturating for
//! `ScoreQ`), and integer compare. Unlike `route_attention.rs`, there
//! is no per-byte relation to compute at all: classification
//! (`role_rank`, `cascade_position`) was precomputed at instance-build
//! time and is baked into the instance's row bytes
//! (`uor-r4-graph-format::msa_selector` module docs explain why — the
//! reference's `candidate_id mod 11` cannot be recomputed on this
//! P-4-scanned path). This kernel only ever reads that declared table.
//!
//! One step over an instance with `N` candidates selecting `M`:
//!
//! 1. classification: read, per candidate, the instance's declared
//!    `(role_rank, cascade_position)` row fields — a table read, no
//!    computation;
//! 2. selection: bounded top-M by ascending
//!    `(role_rank, cascade_position, index)` with EXACTLY `M` ordered
//!    slot comparisons per candidate (the count is data-independent,
//!    so the census is a closed form of `(N, M)` and
//!    replay-verifiable without running the kernel); ties resolve to
//!    the LOWEST candidate index by construction — candidates arrive
//!    in ascending index order and an equal-key later index never
//!    displaces a slot;
//! 3. aggregation: the selected contributions fold in selection order
//!    from `ScoreQ::ZERO` with saturating adds — identical convention
//!    to `route_attention.rs`, so the two packed kernels are
//!    plug-compatible for a shared A/B harness.
//!
//! Steady state allocates nothing: the instance is a borrowed
//! [`MsaSelectorView`], and every scratch slot lives in the
//! caller-owned fixed-capacity [`MsaSelectorState`] (the same
//! `StepState` epoch discipline `RouteState` uses: the state is built
//! once, each step advances `epoch`, and the selection slots are valid
//! only for the current epoch). There is no query parameter: this
//! operator's classification is query-independent by construction
//! (`uor-r4-graph-certify::msa_selector` module docs), so every step
//! over the same instance and state produces the same selection.

use uor_r4_graph_format::{MSA_MAX_TOP_M, MsaSelectorOpCensus, MsaSelectorView, ScoreQ};

/// Role-rank sentinel of an unfilled selection slot. Real role ranks
/// are at most 3 ([`uor_r4_graph_format::ROLE_ZERO`]), so the sentinel
/// never collides with a real candidate.
const EMPTY_ROLE_RANK: u8 = u8::MAX;
/// Cascade-position sentinel of an unfilled selection slot. Real
/// positions are at most 10
/// ([`uor_r4_graph_format::CASCADE_SENTINEL_POSITION`]).
const EMPTY_CASCADE_POSITION: u8 = u8::MAX;
/// Index sentinel of an unfilled selection slot. Real indices are
/// below the candidate cap (64).
const EMPTY_INDEX: u32 = u32::MAX;

/// Caller-owned bounded step state: fixed-capacity selection slots
/// (capacity [`MSA_MAX_TOP_M`] — the state capacity bound holds by
/// construction, and every instance's validated `top_m` fits it), an
/// epoch stamp, and the current epoch's selection width. Built once,
/// reused across steps; [`msa_selector_step`] performs no allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsaSelectorState {
    /// Ascending selection-order role ranks; slots past
    /// `selected_len` hold the sentinel.
    role_rank: [u8; MSA_MAX_TOP_M],
    /// Cascade positions aligned with `role_rank`.
    cascade_position: [u8; MSA_MAX_TOP_M],
    /// Candidate indices aligned with `role_rank`.
    candidate: [u32; MSA_MAX_TOP_M],
    /// Number of valid selection slots for the current epoch.
    selected_len: usize,
    /// Step stamp: advanced once per step; the selection slots are
    /// valid only for the epoch they were written in.
    epoch: u64,
}

impl MsaSelectorState {
    /// Fresh state with no valid epoch.
    pub const fn new() -> Self {
        Self {
            role_rank: [EMPTY_ROLE_RANK; MSA_MAX_TOP_M],
            cascade_position: [EMPTY_CASCADE_POSITION; MSA_MAX_TOP_M],
            candidate: [EMPTY_INDEX; MSA_MAX_TOP_M],
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

    /// Selection slot `slot` of the current epoch as `(candidate
    /// index, role_rank, cascade_position)`, in selection order
    /// (ascending `(role_rank, cascade_position, index)`); `None` past
    /// the current selection width.
    pub fn selected(&self, slot: usize) -> Option<(u32, u8, u8)> {
        if slot < self.selected_len {
            Some((
                self.candidate[slot],
                self.role_rank[slot],
                self.cascade_position[slot],
            ))
        } else {
            None
        }
    }
}

impl Default for MsaSelectorState {
    fn default() -> Self {
        Self::new()
    }
}

/// One packed MSA-selector step: declared-table classification read
/// over every candidate, bounded top-M selection with the
/// lowest-index tie rule, and selection-order saturating `ScoreQ`
/// aggregation. Writes the selection into `state` (readable via
/// [`MsaSelectorState::selected`]), increments `census`, and returns
/// the aggregate.
///
/// Total: `view` was already validated by [`MsaSelectorView::parse`],
/// so there is nothing left for this step to refuse (unlike
/// `route_attention_step`, there is no per-step query whose width
/// could mismatch).
pub fn msa_selector_step(
    view: &MsaSelectorView<'_>,
    state: &mut MsaSelectorState,
    census: &mut MsaSelectorOpCensus,
) -> ScoreQ {
    state.epoch = state.epoch.wrapping_add(1);
    let top_m = usize::from(view.top_m());
    // Reset exactly the slots this instance uses (top_m <= capacity by
    // parse-time validation); a tiny fixed loop, no allocation.
    let mut slot = 0usize;
    while slot < top_m {
        state.role_rank[slot] = EMPTY_ROLE_RANK;
        state.cascade_position[slot] = EMPTY_CASCADE_POSITION;
        state.candidate[slot] = EMPTY_INDEX;
        slot += 1;
    }
    state.selected_len = top_m;

    let candidate_count = view.candidate_count();
    let mut index = 0u32;
    while index < candidate_count {
        census.candidates_examined += 1;
        census.table_reads += 1;
        // By construction `index < candidate_count`, so this row
        // always exists; `view` was already validated by `parse`.
        let Some((_, role_rank, cascade_position, _)) = view.candidate_row(index) else {
            break;
        };

        // Bounded top-M insertion: exactly `top_m` ordered slot
        // comparisons per candidate — the count never depends on the
        // data. `(role_rank, cascade_position, index)` beats a slot
        // when it is strictly lower under that lexicographic order;
        // since candidates arrive in ascending index order, an
        // equal-key later candidate never wins — ties go to the
        // lowest index.
        let mut insert_at = top_m;
        let mut probe = 0usize;
        while probe < top_m {
            census.compares += 1;
            let beats = match role_rank.cmp(&state.role_rank[probe]) {
                core::cmp::Ordering::Less => true,
                core::cmp::Ordering::Greater => false,
                core::cmp::Ordering::Equal => {
                    match cascade_position.cmp(&state.cascade_position[probe]) {
                        core::cmp::Ordering::Less => true,
                        core::cmp::Ordering::Greater => false,
                        core::cmp::Ordering::Equal => index < state.candidate[probe],
                    }
                }
            };
            if beats && insert_at == top_m {
                insert_at = probe;
            }
            probe += 1;
        }
        if insert_at < top_m {
            let mut hole = top_m - 1;
            while hole > insert_at {
                state.role_rank[hole] = state.role_rank[hole - 1];
                state.cascade_position[hole] = state.cascade_position[hole - 1];
                state.candidate[hole] = state.candidate[hole - 1];
                hole -= 1;
            }
            state.role_rank[insert_at] = role_rank;
            state.cascade_position[insert_at] = cascade_position;
            state.candidate[insert_at] = index;
        }
        index += 1;
    }

    // Selection-order aggregation: fold the selected contributions
    // from `ScoreQ::ZERO` with saturating adds. Every slot holds a
    // real candidate (top_m <= candidate_count by parse-time
    // validation, and the first top_m candidates always fill the
    // sentinel slots), so re-reading that candidate's row is always
    // `Some` by construction. A SEPARATE row read from the
    // aggregation loop (rather than reusing the classification read
    // above) matches the certify-side reference's declared census
    // schedule (`table_reads = N + M`): N classification reads, M
    // contribution reads.
    let mut aggregate = ScoreQ::ZERO;
    let mut fold = 0usize;
    while fold < top_m {
        let candidate_index = state.candidate[fold];
        let Some((_, _, _, contribution)) = view.candidate_row(candidate_index) else {
            break;
        };
        census.table_reads += 1;
        aggregate = aggregate.saturating_add(contribution);
        census.adds += 1;
        fold += 1;
    }
    aggregate
}
