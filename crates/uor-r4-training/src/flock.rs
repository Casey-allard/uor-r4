//! The shared causal flock selector: one pure rule for read/support selection.
//!
//! Flock attention keeps, for each query row, a **sink**, a **local window** and
//! the **exact top-k** of the remaining causally visible positions. This module
//! owns that rule once, for every consumer:
//!
//! - the B0 training-free probe over a checkpoint's own scores (the candle
//!   adapter in [`crate::kappa_llama`]);
//! - Claude/Lab 1's A1 retrieval read/support semantics: the general
//!   [`flock_select`] rule and the sinkless pointer path [`top_k_select`];
//! - the Codex B2 harmonic + flock hybrid, which needs the deduplicated causal
//!   support in descending Lorentz rank with explicit **unnormalized** weights;
//! - the model-source `score_and_normalize` seam and the later D11 port.
//!
//! # Contract
//!
//! - **One selection rule.** [`flock_select`] takes one query row's rank scores
//!   (positions `0..=query`; a monotone rank score such as the Minkowski
//!   product `-q0 k0 + q . k`, which needs no `arcosh`), the general selection
//!   parameters [`FlockSelect`] `{ sink, window, k }`, and returns a
//!   [`FlockSelection`].
//! - **Deduplicated support.** The support is the *set* `{sink} ∪ window ∪
//!   top-k`. The window is the `window` nearest preceding positions; the sink
//!   takes precedence if it falls inside that range, so no position appears
//!   twice.
//! - **Deterministic order.** Selection entries are ordered by descending rank
//!   score with ties broken to the **lowest position**. The same inputs always
//!   produce the same selection.
//! - **Arms stay distinct.** [`FlockWeights::Softmax`] is the checkpoint's
//!   scaled-dot softmax restricted to the kept set (arm S); it does **not**
//!   apply the per-head learned inverse temperature `log_beta`, so it equals the
//!   model's own softmax only where `log_beta == 0`. [`FlockWeights::Rank`] is the
//!   normalized rank table `w_i ∝ 1/(i + 1)` (arm R, no exponentials). The
//!   **raw, unnormalized** scale `a_i = 1/(i + 1)` is exported separately by
//!   [`raw_rank_weights`] / [`FlockSelection::raw_rank_weights`] for the B2
//!   hybrid correction `N += Σ(a − h)v`, `Z += Σ(a − h)`; it is not the
//!   within-support normalized probability. No denominator clamp is applied
//!   here.
//! - **Honest cost.** [`FlockScan::candidates_scanned`] counts every rest
//!   position the exact selector compared. Selecting `k` values does **not**
//!   establish `O(k)` search; an index (B1) must account for its own build,
//!   scan and recall separately.
//!
//! The B0 pre-registration fixes `sink = 0`, `window = 64` and
//! `k ∈ {1, 7, 16, 64}`; [`FlockSpec`] carries those choices plus the weight
//! arm, and [`FlockSpec::select`] materializes the sink-at-zero selection.
//!
//! This module has no tensor dependency: rows are plain `&[f32]` slices, so the
//! same logic is usable offline and in a later bounded integer/table runtime.

use serde::{Deserialize, Serialize};

use crate::{invalid, Result};

/// Version tag for artifacts that bind the selection semantics.
pub const FLOCK_SELECTOR_VERSION: &str = "flock-selector-v1";

/// How flock attention weights its selected support.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlockWeights {
    /// Arm S: the checkpoint's scaled-dot softmax restricted to the selected
    /// support. The per-head learned inverse temperature (`log_beta`) is not
    /// applied, so this equals the model's own softmax only when
    /// `log_beta == 0` (true for the training-free probe's fresh scalars).
    Softmax,
    /// Arm R: normalized fixed rank-indexed weights `w_i = 1 / (i + 1)`, where
    /// `i` is the rank position inside the selection. No exponentials.
    Rank,
}

impl FlockWeights {
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "softmax" => Ok(Self::Softmax),
            "rank" => Ok(Self::Rank),
            _ => Err(invalid(format!(
                "weights must be softmax or rank, not {text}"
            ))),
        }
    }
}

/// The B0 pre-registered flock configuration: sink at position zero, a local
/// window and an exact top-k, with one weight arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlockSpec {
    /// Exact top-k keys by rank score from the non-sink, non-window past.
    pub k: usize,
    /// Nearest preceding positions always included.
    pub window: usize,
    /// Which weight arm to use.
    pub weights: FlockWeights,
}

impl FlockSpec {
    /// The pre-registered selection rule: the sink is position zero.
    pub fn select(&self) -> FlockSelect {
        FlockSelect {
            sink: 0,
            window: self.window,
            k: self.k,
        }
    }
}

/// General selection parameters for one query row.
///
/// The B0 probe uses `sink = 0`; retrieval readers may anchor the sink at their
/// own address without changing the rest of the rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlockSelect {
    /// Always-selected anchor position (B0 uses zero).
    pub sink: usize,
    /// Nearest preceding positions always included.
    pub window: usize,
    /// Exact top-k of the remaining causally visible positions.
    pub k: usize,
}

/// Role of a selected key inside the flock support.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlockSlot {
    /// The sink position, always included.
    Sink,
    /// One of the nearest `window` causally visible positions.
    Window,
    /// An exact top-k key outside the sink and window.
    TopK,
}

/// One selected position and the slot that admitted it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlockEntry {
    pub position: usize,
    pub slot: FlockSlot,
}

/// Exact cost accounting for one query row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FlockScan {
    /// Causally visible positions (`query + 1`).
    pub visible: usize,
    /// Kept entries whose slot is [`FlockSlot::Sink`] (zero or one).
    pub sink_kept: usize,
    /// Kept entries whose slot is [`FlockSlot::Window`].
    pub window_kept: usize,
    /// Kept entries whose slot is [`FlockSlot::TopK`].
    pub top_k_kept: usize,
    /// Rest positions the exact selector compared for the top-k. This is a full
    /// scan of the non-window prefix; it is not evidence of `O(k)` search.
    pub candidates_scanned: usize,
    /// The whole visible prefix fits inside the nominal window
    /// (`query + 1 <= window`). With the B0 sink at position 0 this is exactly
    /// the sink-in-window-span indicator; a general sink inside the window is
    /// not separately counted.
    pub short_prefix: bool,
    /// Fewer than `k` rest candidates existed.
    pub top_k_short: bool,
    /// The k-th and (k+1)-th rest candidates have equal rank scores under
    /// IEEE `==`, so the top-k boundary is a tie. Ordering is resolved by
    /// `total_cmp`; equal exact scores break to the lowest position, while
    /// `-0.0` and `+0.0` compare equal under `==` but are ordered by sign.
    pub cutoff_ties: bool,
}

/// The deduplicated support of one query row, in canonical rank order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlockSelection {
    /// Kept entries ordered by descending rank score; ties break to the lowest
    /// position.
    pub entries: Vec<FlockEntry>,
    /// Exact counters for this row.
    pub scan: FlockScan,
}

impl FlockSelection {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Kept positions in canonical (descending rank) order.
    pub fn positions(&self) -> impl Iterator<Item = usize> + '_ {
        self.entries.iter().map(|entry| entry.position)
    }

    /// Raw unnormalized rank weights `a_i = 1/(i + 1)` aligned with `entries`.
    ///
    /// This is the B2 hybrid scale: it is deliberately **not** normalized over
    /// the support.
    pub fn raw_rank_weights(&self) -> Vec<f32> {
        raw_rank_weights(self.entries.len())
    }

    /// The within-support normalized rank table (arm R) aligned with `entries`.
    pub fn normalized_rank_weights(&self) -> Vec<f32> {
        rank_table(self.entries.len())
    }
}

/// Accumulated support-selection statistics over a probe or evaluation run.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FlockStats {
    pub queries: u64,
    pub selected: u64,
    pub sink: u64,
    pub window: u64,
    pub top_k: u64,
    /// Sum of [`FlockScan::candidates_scanned`] over recorded rows.
    pub candidates_scanned: u64,
    /// Recorded rows whose prefix fits inside the nominal window.
    pub short_prefix: u64,
    /// Recorded rows with fewer than `k` rest candidates.
    pub top_k_short: u64,
    /// Recorded rows whose top-k boundary is a tie.
    pub cutoff_ties: u64,
}

impl FlockStats {
    pub fn add(&mut self, other: &FlockStats) {
        self.queries += other.queries;
        self.selected += other.selected;
        self.sink += other.sink;
        self.window += other.window;
        self.top_k += other.top_k;
        self.candidates_scanned += other.candidates_scanned;
        self.short_prefix += other.short_prefix;
        self.top_k_short += other.top_k_short;
        self.cutoff_ties += other.cutoff_ties;
    }

    /// Accumulate one row's exact counter record.
    pub fn record(&mut self, scan: &FlockScan) {
        self.queries += 1;
        self.selected += (scan.sink_kept + scan.window_kept + scan.top_k_kept) as u64;
        self.sink += scan.sink_kept as u64;
        self.window += scan.window_kept as u64;
        self.top_k += scan.top_k_kept as u64;
        self.candidates_scanned += scan.candidates_scanned as u64;
        if scan.short_prefix {
            self.short_prefix += 1;
        }
        if scan.top_k_short {
            self.top_k_short += 1;
        }
        if scan.cutoff_ties {
            self.cutoff_ties += 1;
        }
    }

    pub fn mean_selected(&self) -> f64 {
        if self.queries == 0 {
            return 0.0;
        }
        self.selected as f64 / self.queries as f64
    }

    fn share(&self, part: u64) -> f64 {
        if self.selected == 0 {
            return 0.0;
        }
        part as f64 / self.selected as f64
    }

    pub fn sink_share(&self) -> f64 {
        self.share(self.sink)
    }

    pub fn window_share(&self) -> f64 {
        self.share(self.window)
    }

    pub fn top_k_share(&self) -> f64 {
        self.share(self.top_k)
    }
}

/// Raw unnormalized rank weights `a_i = 1 / (i + 1)` for `count` entries.
///
/// These are the B2 hybrid weights as written: the fixed scale is the raw
/// reciprocal rank, not its within-support normalized probability.
pub fn raw_rank_weights(count: usize) -> Vec<f32> {
    (0..count)
        .map(|rank| (1.0f64 / (rank as f64 + 1.0)) as f32)
        .collect()
}

/// Normalized fixed rank table `w_i ∝ 1 / (i + 1)` over `count` entries.
///
/// This is arm R's within-support weight vector (it sums to one up to f32
/// rounding); [`raw_rank_weights`] is the unnormalized hybrid scale.
pub fn rank_table(count: usize) -> Vec<f32> {
    let raw: Vec<f64> = (0..count).map(|i| 1.0 / (i as f64 + 1.0)).collect();
    let sum: f64 = raw.iter().sum();
    raw.iter().map(|w| (w / sum) as f32).collect()
}

/// The exact deduplicated flock support for one query row.
///
/// `rank_scores[i]` is the monotone rank score of position `i` (positions
/// `0..=query`, all causally visible). A higher score ranks nearer. The support
/// is `{sink} ∪ window ∪ top-k`: the sink is always kept, the window keeps the
/// `window` nearest preceding positions, and the exact top-`k` of the rest is
/// kept. Entries are returned in descending rank order with ties broken to the
/// lowest position; `entries` never repeats a position.
///
/// Exact scan only: the top-k is found by comparing every rest position, and
/// [`FlockScan::candidates_scanned`] records that cost. No approximation and no
/// index are used here.
pub fn flock_select(
    rank_scores: &[f32],
    query: usize,
    select: FlockSelect,
) -> Result<FlockSelection> {
    if query.checked_add(1) != Some(rank_scores.len()) {
        return Err(invalid(format!(
            "flock row has {} scores for query {query}",
            rank_scores.len()
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
    if rank_scores.iter().any(|score| !score.is_finite()) {
        return Err(invalid("flock rank scores must be finite"));
    }

    let window_start = (query + 1).saturating_sub(select.window);
    let mut slots: Vec<Option<FlockSlot>> = vec![None; query + 1];
    for slot in slots[window_start..=query].iter_mut() {
        *slot = Some(FlockSlot::Window);
    }
    // The sink is always kept; when it lies inside the window span the sink
    // slot wins so the position appears exactly once.
    slots[select.sink] = Some(FlockSlot::Sink);

    let mut rest: Vec<usize> = (0..window_start)
        .filter(|&position| slots[position].is_none())
        .collect();
    rest.sort_by(|&a, &b| {
        rank_scores[b]
            .total_cmp(&rank_scores[a])
            .then_with(|| a.cmp(&b))
    });
    let candidates_scanned = rest.len();
    let top_k_short = candidates_scanned < select.k;
    let cutoff_ties = candidates_scanned > select.k
        && rank_scores[rest[select.k - 1]] == rank_scores[rest[select.k]];
    for &position in rest.iter().take(select.k) {
        slots[position] = Some(FlockSlot::TopK);
    }

    let mut entries: Vec<FlockEntry> = slots
        .iter()
        .enumerate()
        .filter_map(|(position, slot)| slot.map(|slot| FlockEntry { position, slot }))
        .collect();
    entries.sort_by(|a, b| {
        rank_scores[b.position]
            .total_cmp(&rank_scores[a.position])
            .then_with(|| a.position.cmp(&b.position))
    });

    let mut scan = FlockScan {
        visible: query + 1,
        candidates_scanned,
        short_prefix: query + 1 <= select.window,
        top_k_short,
        cutoff_ties,
        ..FlockScan::default()
    };
    for entry in &entries {
        match entry.slot {
            FlockSlot::Sink => scan.sink_kept += 1,
            FlockSlot::Window => scan.window_kept += 1,
            FlockSlot::TopK => scan.top_k_kept += 1,
        }
    }

    Ok(FlockSelection { entries, scan })
}

/// Top-k only selection over `rank_scores[0..=query]`.
///
/// Mirrors the integer top-k bridge used for pointer retrieval. Like that
/// sibling it accepts a full row: only `rank_scores.len() <= query` is refused,
/// and selection runs over the `0..=query` prefix. There is no sink and no
/// window; entries are returned in descending rank order with lowest-position
/// ties. The whole prefix is sorted, so the cutoff-tie flag is computed against
/// the true k-th candidate. This float selector is a semantic — not
/// algorithmic — mirror: it uses an exact full sort, while the integer path
/// partially selects; `candidates_scanned` records the true scan either way.
pub fn top_k_select(rank_scores: &[f32], query: usize, k: usize) -> Result<FlockSelection> {
    if rank_scores.len() <= query {
        return Err(invalid(format!(
            "flock scores length {} is too short for query {query}",
            rank_scores.len()
        )));
    }
    if k == 0 {
        return Err(invalid("flock k must be positive"));
    }
    if rank_scores[..=query].iter().any(|score| !score.is_finite()) {
        return Err(invalid("flock rank scores must be finite"));
    }
    let mut rest: Vec<usize> = (0..=query).collect();
    rest.sort_by(|&a, &b| {
        rank_scores[b]
            .total_cmp(&rank_scores[a])
            .then_with(|| a.cmp(&b))
    });
    let candidates_scanned = rest.len();
    let top_k_short = candidates_scanned < k;
    let cutoff_ties = candidates_scanned > k && rank_scores[rest[k - 1]] == rank_scores[rest[k]];
    let top_k_kept = k.min(candidates_scanned);
    let entries: Vec<FlockEntry> = rest
        .into_iter()
        .take(k)
        .map(|position| FlockEntry {
            position,
            slot: FlockSlot::TopK,
        })
        .collect();
    Ok(FlockSelection {
        entries,
        scan: FlockScan {
            visible: query + 1,
            sink_kept: 0,
            window_kept: 0,
            top_k_kept,
            candidates_scanned,
            short_prefix: false,
            top_k_short,
            cutoff_ties,
        },
    })
}

/// Arm S: softmax over the kept set, as a dense row of length `scores.len()`.
///
/// `scores` are the host model's own scores for positions `0..=query` (for the
/// B0 probe, the checkpoint's scaled dot product). Off-support entries are
/// exactly zero; the kept entries are the stable softmax over the support.
pub fn softmax_over_support(scores: &[f32], selection: &FlockSelection) -> Result<Vec<f32>> {
    if selection.entries.is_empty() {
        return Err(invalid("flock support is empty"));
    }
    let mut weights = vec![0f32; scores.len()];
    let mut maximum = f32::NEG_INFINITY;
    for entry in &selection.entries {
        let value = *scores
            .get(entry.position)
            .ok_or_else(|| invalid("flock support position is outside the score row"))?;
        if !value.is_finite() {
            return Err(invalid("flock score row must be finite"));
        }
        maximum = maximum.max(value);
    }
    let mut sum = 0f64;
    for entry in &selection.entries {
        let weight = f64::from((scores[entry.position] - maximum).exp());
        weights[entry.position] = weight as f32;
        sum += weight;
    }
    if !(sum > 0.0) {
        return Err(invalid("flock softmax support has zero mass"));
    }
    for entry in &selection.entries {
        weights[entry.position] = (f64::from(weights[entry.position]) / sum) as f32;
    }
    Ok(weights)
}

/// One query row's selection and dense weights for the chosen arm.
pub struct FlockRowWeights {
    /// The deduplicated support in canonical rank order.
    pub selection: FlockSelection,
    /// Dense weights over `0..=query`; zero outside the support.
    pub weights: Vec<f32>,
}

/// Select one query row and compute its arm weights.
///
/// `rank_scores` orders the candidates (a monotone rank score); `model_scores`
/// supplies arm S's softmax values and is required to have the same length.
/// Arm R ignores `model_scores` but the shape check still applies, so both
/// callers can compute the two rows once.
pub fn flock_row_weights(
    rank_scores: &[f32],
    model_scores: &[f32],
    query: usize,
    spec: &FlockSpec,
) -> Result<FlockRowWeights> {
    if model_scores.len() != rank_scores.len() {
        return Err(invalid("flock rank and model score rows differ in length"));
    }
    let selection = flock_select(rank_scores, query, spec.select())?;
    let weights = match spec.weights {
        FlockWeights::Softmax => softmax_over_support(model_scores, &selection)?,
        FlockWeights::Rank => {
            let table = rank_table(selection.entries.len());
            let mut weights = vec![0f32; rank_scores.len()];
            for (rank, entry) in selection.entries.iter().enumerate() {
                weights[entry.position] = table[rank];
            }
            weights
        }
    };
    Ok(FlockRowWeights { selection, weights })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn select_scores(scores: &[f32], query: usize, k: usize, window: usize) -> FlockSelection {
        flock_select(scores, query, FlockSelect { sink: 0, window, k }).expect("selection")
    }

    fn positions(selection: &FlockSelection) -> Vec<usize> {
        selection.positions().collect()
    }

    #[test]
    fn selects_sink_window_and_topk_deduplicated_in_rank_order() {
        let scores = [0.0f32, 1.0, 2.0, 9.0, 3.0, 8.0, 4.0, 5.0, 6.0, 7.0, 0.5];
        let selection = select_scores(&scores, 10, 2, 4);
        // Membership: sink 0, window 7..=10, top-2 of {1..=6} = {3, 5}.
        assert_eq!(positions(&selection), vec![3, 5, 9, 8, 7, 10, 0]);
        assert_eq!(
            selection
                .entries
                .iter()
                .map(|entry| entry.slot)
                .collect::<Vec<_>>(),
            vec![
                FlockSlot::TopK,
                FlockSlot::TopK,
                FlockSlot::Window,
                FlockSlot::Window,
                FlockSlot::Window,
                FlockSlot::Window,
                FlockSlot::Sink,
            ]
        );
        let scan = selection.scan;
        assert_eq!(scan.visible, 11);
        assert_eq!(scan.sink_kept, 1);
        assert_eq!(scan.window_kept, 4);
        assert_eq!(scan.top_k_kept, 2);
        assert_eq!(scan.candidates_scanned, 6);
        assert!(!scan.short_prefix);
        assert!(!scan.top_k_short);
        assert!(!scan.cutoff_ties);
    }

    #[test]
    fn first_position_selects_only_the_sink() {
        let scores = [0.5f32];
        let selection = select_scores(&scores, 0, 7, 64);
        assert_eq!(positions(&selection), vec![0]);
        assert_eq!(selection.entries[0].slot, FlockSlot::Sink);
        assert!(selection.scan.short_prefix);
        assert!(selection.scan.top_k_short);
        assert_eq!(selection.scan.candidates_scanned, 0);
        assert_eq!(selection.scan.window_kept, 0);
    }

    #[test]
    fn window_at_least_the_prefix_keeps_each_position_once() {
        let scores = [3.0f32, 1.0, 2.0, 0.5];
        let selection = select_scores(&scores, 3, 2, 64);
        // Window spans 0..=3; the sink at 0 wins its slot, so 0 is not repeated.
        assert_eq!(positions(&selection), vec![0, 2, 1, 3]);
        assert_eq!(
            selection.entries[0].slot,
            FlockSlot::Sink,
            "the sink ranks first here because it has the highest score"
        );
        assert_eq!(selection.scan.sink_kept, 1);
        assert_eq!(selection.scan.window_kept, 3);
        assert_eq!(selection.scan.top_k_kept, 0);
        assert!(selection.scan.short_prefix);
        assert!(selection.scan.top_k_short);
        assert_eq!(selection.scan.candidates_scanned, 0);
    }

    #[test]
    fn ties_break_to_the_lowest_position_and_the_cutoff_tie_is_counted() {
        // Candidates outside window: positions 0..=4 with equal scores 7.0 at
        // positions 1 and 4; k = 1 keeps the lower position.
        let scores = [1.0f32, 7.0, 2.0, 3.0, 7.0, 0.0];
        let selection = select_scores(&scores, 5, 1, 1);
        // Window is position 5; rest is {1, 2, 3, 4}; top-1 keeps position 1.
        assert_eq!(positions(&selection), vec![1, 0, 5]);
        assert_eq!(selection.entries[0].slot, FlockSlot::TopK);
        assert_eq!(selection.entries[0].position, 1);
        assert!(selection.scan.cutoff_ties, "7.0 at rank 1 ties rank 2");

        // k = 2 keeps both equal-score candidates, tie ordering by position.
        let selection = select_scores(&scores, 5, 2, 1);
        let kept: Vec<usize> = selection.positions().take(2).collect();
        assert_eq!(kept, vec![1, 4]);
    }

    #[test]
    fn equal_scores_order_by_position_everywhere() {
        let scores = [2.0f32, 2.0, 2.0, 2.0, 2.0];
        let selection = select_scores(&scores, 4, 3, 1);
        // Window is 4; sink is 0; rest is 1..=3; all scores equal, so the
        // canonical order is pure position order.
        assert_eq!(positions(&selection), vec![0, 1, 2, 3, 4]);
        let slots: Vec<FlockSlot> = selection.entries.iter().map(|e| e.slot).collect();
        assert_eq!(
            slots,
            vec![
                FlockSlot::Sink,
                FlockSlot::TopK,
                FlockSlot::TopK,
                FlockSlot::TopK,
                FlockSlot::Window,
            ]
        );
    }

    #[test]
    fn general_sink_is_deduplicated_against_the_window() {
        let scores = [5.0f32, 1.0, 2.0, 3.0];
        let selection = flock_select(
            &scores,
            3,
            FlockSelect {
                sink: 2,
                window: 4,
                k: 2,
            },
        )
        .expect("selection");
        let kept: Vec<usize> = selection.positions().collect();
        assert_eq!(kept.len(), 4, "each position appears exactly once");
        assert_eq!(kept, vec![0, 3, 2, 1]);
        let sink_entry = selection
            .entries
            .iter()
            .find(|entry| entry.position == 2)
            .expect("sink entry");
        assert_eq!(sink_entry.slot, FlockSlot::Sink);
        assert_eq!(selection.scan.top_k_kept, 0);
        assert_eq!(selection.scan.candidates_scanned, 0);
    }

    #[test]
    fn rejects_malformed_rows() {
        let scores = [1.0f32, 2.0, 3.0];
        assert!(flock_select(
            &scores,
            2,
            FlockSelect {
                sink: 0,
                window: 0,
                k: 1
            }
        )
        .is_err());
        assert!(flock_select(
            &scores,
            2,
            FlockSelect {
                sink: 0,
                window: 1,
                k: 0
            }
        )
        .is_err());
        assert!(flock_select(
            &scores,
            1,
            FlockSelect {
                sink: 0,
                window: 1,
                k: 1
            }
        )
        .is_err());
        assert!(
            flock_select(
                &scores,
                1,
                FlockSelect {
                    sink: 2,
                    window: 1,
                    k: 1
                }
            )
            .is_err(),
            "a sink beyond the query is not causally visible"
        );
        assert!(
            flock_select(
                &[f32::NAN, 1.0],
                1,
                FlockSelect {
                    sink: 0,
                    window: 1,
                    k: 1
                }
            )
            .is_err(),
            "non-finite rank scores are rejected"
        );
    }

    #[test]
    fn raw_weights_are_exact_reciprocals_and_normalized_table_sums_to_one() {
        let raw = raw_rank_weights(4);
        assert_eq!(raw, vec![1.0, 0.5, 1.0 / 3.0, 0.25]);
        let table = rank_table(4);
        let sum: f64 = table.iter().map(|w| f64::from(*w)).sum();
        assert!((sum - 1.0).abs() < 1e-6, "table summed to {sum}");
        for pair in table.windows(2) {
            assert!(pair[0] > pair[1], "rank table must be strictly decreasing");
        }
    }

    #[test]
    fn softmax_over_support_matches_the_dense_softmax_restricted_to_it() {
        let scores = [0.5f32, -1.0, 2.0, 0.25, 1.5, -0.75, 0.0, 0.9];
        let selection = select_scores(&scores, 7, 3, 2);
        let weights = softmax_over_support(&scores, &selection).expect("weights");
        assert_eq!(weights.len(), scores.len());
        for (position, weight) in weights.iter().enumerate() {
            let kept = selection
                .entries
                .iter()
                .any(|entry| entry.position == position);
            if kept {
                assert!(*weight > 0.0, "kept position {position} must have mass");
            } else {
                assert_eq!(*weight, 0.0, "position {position} is outside the support");
            }
        }
        let total: f64 = weights.iter().map(|w| f64::from(*w)).sum();
        assert!((total - 1.0).abs() < 1e-6, "support mass {total}");
        // The dense softmax renormalized over the support must agree.
        let maximum = selection
            .entries
            .iter()
            .map(|entry| scores[entry.position])
            .fold(f32::NEG_INFINITY, f32::max);
        let dense: Vec<f64> = scores
            .iter()
            .map(|score| f64::from((score - maximum).exp()))
            .collect();
        let support_mass: f64 = selection
            .entries
            .iter()
            .map(|entry| dense[entry.position])
            .sum();
        for entry in &selection.entries {
            let expected = (dense[entry.position] / support_mass) as f32;
            let got = weights[entry.position];
            assert!(
                (f64::from(got) - f64::from(expected)).abs() < 1e-6,
                "position {}: {got} vs {expected}",
                entry.position
            );
        }
    }

    #[test]
    fn selection_and_weights_are_deterministic() {
        let scores = [0.1f32, 0.9, 0.5, 0.7, 0.3, 0.8, 0.2];
        let model = [0.2f32, 0.4, 0.6, 0.8, 1.0, 0.9, 0.7];
        let spec = FlockSpec {
            k: 3,
            window: 2,
            weights: FlockWeights::Rank,
        };
        let first = flock_row_weights(&scores, &model, 6, &spec).expect("first");
        for _ in 0..10 {
            let again = flock_row_weights(&scores, &model, 6, &spec).expect("again");
            assert_eq!(again.selection, first.selection);
            assert_eq!(again.weights, first.weights);
        }
    }

    #[test]
    fn rank_arm_support_is_zero_outside_and_strictly_ranked_inside() {
        let scores = [0.1f32, 0.9, 0.5, 0.7, 0.3, 0.8, 0.2];
        let model = [0.0f32; 7];
        let spec = FlockSpec {
            k: 2,
            window: 2,
            weights: FlockWeights::Rank,
        };
        let row = flock_row_weights(&scores, &model, 6, &spec).expect("row");
        let kept: Vec<usize> = row.selection.positions().collect();
        for (position, weight) in row.weights.iter().enumerate() {
            if kept.contains(&position) {
                assert!(*weight > 0.0, "kept position {position}");
            } else {
                assert_eq!(*weight, 0.0, "position {position} is outside");
            }
        }
        let ranked: Vec<f32> = kept.iter().map(|position| row.weights[*position]).collect();
        for pair in ranked.windows(2) {
            assert!(pair[0] > pair[1], "arm R weights must strictly decrease");
        }
        let total: f64 = ranked.iter().map(|w| f64::from(*w)).sum();
        assert!((total - 1.0).abs() < 1e-6);
    }

    #[test]
    fn top_k_select_matches_the_pointer_use_case() {
        let mut scores = [10.0f32; 20];
        scores[7] = 999.0;
        let selection = top_k_select(&scores, 19, 1).expect("top-k");
        assert_eq!(selection.positions().collect::<Vec<_>>(), vec![7]);
        assert_eq!(selection.entries[0].slot, FlockSlot::TopK);
        assert_eq!(selection.scan.sink_kept, 0);
        assert_eq!(selection.scan.window_kept, 0);
        assert_eq!(selection.scan.top_k_kept, 1);
        assert_eq!(selection.scan.candidates_scanned, 20);
        assert!(!selection.scan.short_prefix);
        assert!(!selection.scan.top_k_short);
    }

    #[test]
    fn top_k_select_ties_short_rows_and_malformed_inputs() {
        let scores = [1.0f32, 5.0, 5.0, 2.0];
        let selection = top_k_select(&scores, 3, 1).expect("top-k");
        assert_eq!(selection.positions().collect::<Vec<_>>(), vec![1]);
        assert!(selection.scan.cutoff_ties, "5.0 at rank 0 ties rank 1");
        let all = top_k_select(&scores, 3, 9).expect("top-k");
        assert!(all.scan.top_k_short);
        assert_eq!(all.scan.top_k_kept, 4);
        assert_eq!(all.entries.len(), 4);
        assert!(!all.scan.cutoff_ties, "no cutoff exists when all are kept");

        assert!(top_k_select(&scores, 3, 0).is_err());
        assert!(top_k_select(&scores[..2], 3, 1).is_err());
        assert!(top_k_select(&[f32::NAN, 1.0], 1, 1).is_err());

        let full_row = [9.0f32, 1.0, 7.0, 2.0];
        let relaxed = top_k_select(&full_row, 1, 1).expect("a full row is accepted");
        assert_eq!(relaxed.positions().collect::<Vec<_>>(), vec![0]);
        assert_eq!(
            relaxed.scan.candidates_scanned, 2,
            "only the 0..=query prefix is scanned"
        );
        assert!(
            top_k_select(&[1.0, f32::NAN], 0, 1).is_ok(),
            "non-finite scores outside the prefix are ignored"
        );
    }
}
