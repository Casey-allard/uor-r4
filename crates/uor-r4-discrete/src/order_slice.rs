//! The position sheet's multi-occurrence, order-sensitive DEV slice
//! (`docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` §5, item #2843).
//!
//! Host-only selection over recorded token identities, like `is_copy_successor`: no model
//! execution, no floating point. A row's window is the document-bounded, C192-truncated window
//! `PreparedCorpus::batch` builds; its last token is the query.
use crate::corpus::{sha256, PreparedCorpus};
use crate::metadata::MAX_CONTEXT;
use crate::{Error, Identity, Result};
use serde::{Deserialize, Serialize};

/// The window length the slice is defined at (the pinned M1 `Spec` context).
pub const CONTEXT: u32 = 192;
pub const SELECTION_POLICY: &str =
    "query-occurs-twice-in-prefix/pairwise-distinct-successors/C192-window/ascending-rows/v1";

/// Pinned SHA-256 of the committed `docs/labs/m1-position-mechanism/order-slice-v1.tsv`
/// frozen subset (`#3046`, reproduced in `kimi-32`).
pub const FROZEN_ORDER_SLICE_SHA256: &str =
    "f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a";

/// Pinned row count of the frozen multi-occurrence order-sensitive DEV slice (`N >= 200`).
pub const FROZEN_ORDER_SLICE_ROWS: usize = 1_620;

/// Pinned total eligible DEV rows (`ft/dev/tokens.u16` / `response_mask.u8`).
pub const PINNED_DEV_ELIGIBLE_ROWS: u64 = 14_287;

/// Pinned `P_succ` copy-successor row count on DEV.
pub const PINNED_DEV_COPY_SUCCESSOR_ROWS: usize = 3_847;

/// Minimum paired McNemar discordance floor `24` registered on `#2028` (`6356034555`).
pub const MCNEMAR_MIN_FLOOR: u64 = 24;

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

/// The tokens that follow each earlier occurrence of the query, in window order. The occurrence
/// just before the query has the query itself as its successor, as in `is_copy_successor`.
fn successors(window: &[Identity]) -> Vec<u32> {
    let Some(query) = window.last() else {
        return Vec::new();
    };
    window[..window.len() - 1]
        .iter()
        .enumerate()
        .filter(|(_, identity)| identity.token == query.token)
        .map(|(index, _)| window[index + 1].token)
        .collect()
}

/// §5's rule: the query token occurs at least twice before the query, and the tokens following
/// those occurrences are pairwise distinct.
pub fn is_order_sensitive(window: &[Identity]) -> bool {
    let mut next = successors(window);
    let occurrences = next.len();
    next.sort_unstable();
    next.dedup();
    occurrences >= 2 && next.len() == occurrences
}

/// Diagnostic only, never the frozen slice: at least two earlier occurrences whose successors
/// include at least two distinct tokens (repeats allowed).
pub fn has_two_distinct_successors(window: &[Identity]) -> bool {
    let mut next = successors(window);
    next.sort_unstable();
    next.dedup();
    next.len() >= 2
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderSlice {
    /// `(row_ordinal, target_position)` for every selected row, ascending.
    pub rows: Vec<(u64, u64)>,
    /// Rows scanned: the whole eligible inventory.
    pub scanned: u64,
    /// The looser diagnostic count (`has_two_distinct_successors`).
    pub loose: u64,
}

/// Scan every eligible row in ascending order. A row's ordinal is its index in the eligible
/// inventory, the same ordinal the DEV evaluation's batches use.
pub fn select(corpus: &PreparedCorpus) -> Result<OrderSlice> {
    let total = corpus.eligible_len() as u64;
    let mut slice = OrderSlice {
        rows: Vec::new(),
        scanned: 0,
        loose: 0,
    };
    let mut cursor = 0;
    while cursor < total {
        let batch = corpus.batch(cursor, CONTEXT, 64)?;
        for (k, window) in batch.windows.iter().enumerate() {
            if is_order_sensitive(window) {
                slice
                    .rows
                    .push((cursor + k as u64, batch.target_positions[k]));
            }
            if has_two_distinct_successors(window) {
                slice.loose += 1;
            }
        }
        slice.scanned += batch.windows.len() as u64;
        cursor = batch.next_target;
    }
    Ok(slice)
}

/// The frozen file: a header line, then one `row_ordinal<TAB>target_position` line per row.
pub fn render(slice: &OrderSlice) -> String {
    let mut out = String::from("row_ordinal\ttarget_position\n");
    for (ordinal, position) in &slice.rows {
        out.push_str(&format!("{ordinal}\t{position}\n"));
    }
    out
}

/// Parse and verify the committed `order-slice-v1.tsv` frozen subset against its
/// pinned SHA-256 (`FROZEN_ORDER_SLICE_SHA256`), row count (`1,620`), and
/// strictly ascending `row_ordinal < 14,287` invariant.
pub fn parse_frozen_slice(tsv: &str) -> Result<Vec<(u64, u64)>> {
    let actual_sha = sha256(tsv.as_bytes());
    if actual_sha != FROZEN_ORDER_SLICE_SHA256 {
        return Err(invalid(format!(
            "frozen order slice SHA-256 mismatch: expected {FROZEN_ORDER_SLICE_SHA256}, got {actual_sha}"
        )));
    }
    let mut lines = tsv.lines();
    let Some(header) = lines.next() else {
        return Err(invalid("frozen order slice TSV is empty"));
    };
    if header != "row_ordinal\ttarget_position" {
        return Err(invalid(format!(
            "unexpected frozen order slice header: {header}"
        )));
    }
    let mut rows = Vec::with_capacity(FROZEN_ORDER_SLICE_ROWS);
    let mut prev_ordinal: Option<u64> = None;
    let mut prev_target: Option<u64> = None;
    for (line_idx, line) in lines.enumerate() {
        let mut parts = line.split('\t');
        let Some(ord_str) = parts.next() else {
            return Err(invalid(format!(
                "missing row_ordinal on frozen slice line {line_idx}"
            )));
        };
        let Some(pos_str) = parts.next() else {
            return Err(invalid(format!(
                "missing target_position on frozen slice line {line_idx}"
            )));
        };
        if parts.next().is_some() {
            return Err(invalid(format!(
                "unexpected extra column on frozen slice line {line_idx}"
            )));
        }
        let ordinal: u64 = ord_str
            .parse()
            .map_err(|_| invalid(format!("invalid row_ordinal on line {line_idx}")))?;
        let target_position: u64 = pos_str
            .parse()
            .map_err(|_| invalid(format!("invalid target_position on line {line_idx}")))?;
        if ordinal >= PINNED_DEV_ELIGIBLE_ROWS {
            return Err(invalid(format!(
                "frozen slice row_ordinal {ordinal} exceeds DEV eligible count {PINNED_DEV_ELIGIBLE_ROWS}"
            )));
        }
        if let Some(prev) = prev_ordinal {
            if ordinal <= prev {
                return Err(invalid(format!(
                    "frozen slice row_ordinal must be strictly ascending ({prev} -> {ordinal})"
                )));
            }
        }
        if let Some(prev_pos) = prev_target {
            if target_position <= prev_pos {
                return Err(invalid(format!(
                    "frozen slice target_position must be strictly ascending ({prev_pos} -> {target_position})"
                )));
            }
        }
        prev_ordinal = Some(ordinal);
        prev_target = Some(target_position);
        rows.push((ordinal, target_position));
    }
    if rows.len() != FROZEN_ORDER_SLICE_ROWS {
        return Err(invalid(format!(
            "frozen order slice requires {FROZEN_ORDER_SLICE_ROWS} rows, got {}",
            rows.len()
        )));
    }
    Ok(rows)
}

/// Deterministically shuffle prefix `Identity` entries `0..window.len()-1`
/// (splitmix64 Fisher-Yates matching `position_mechanism::shuffle_prefix_roots`)
/// while keeping the query entry at `L - 1` and the prefix token multiset fixed,
/// for the shuffled-context-order null control (`C_shuf`).
pub fn shuffle_prefix_window(window: &[Identity], seed: u64) -> Result<Vec<Identity>> {
    if window.is_empty() || window.len() > MAX_CONTEXT {
        return Err(invalid("position window length must be in 1..=243"));
    }
    let mut out = window.to_vec();
    let prefix_len = out.len().saturating_sub(1);
    let mut state = seed;
    let mut next_u64 = || {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    };
    for i in (1..prefix_len).rev() {
        let j = (next_u64() % (i as u64 + 1)) as usize;
        out.swap(i, j);
    }
    Ok(out)
}

/// Exact integer `ceil(2 * sqrt(sum)) = ceil(sqrt(4 * sum))` using integer
/// square roots only (zero floating-point operations, D32).
pub fn integer_ceil_two_sqrt(sum: u64) -> u64 {
    if sum == 0 {
        return 0;
    }
    let scaled = u128::from(sum) * 4;
    let root = scaled.isqrt();
    let ceil_root = if root * root == scaled {
        root
    } else {
        root + 1
    };
    u64::try_from(ceil_root).unwrap_or(u64::MAX)
}

/// Paired per-row evaluation observation across `H_pos(ordered)`,
/// `C_match(ordered)`, and `C_shuf` on a single seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionRowObservation {
    pub row_ordinal: u64,
    pub is_copy_successor: bool,
    pub is_order_sensitive: bool,
    pub h_pos_ordered_hit: bool,
    pub h_pos_ordered_admitted: bool,
    pub c_match_ordered_hit: bool,
    pub c_match_ordered_admitted: bool,
    pub c_shuf_hit: bool,
    pub c_shuf_admitted: bool,
}

/// Paired one-sided McNemar and target-admission summary for a row subset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionPairedCount {
    pub rows: usize,
    pub treatment_hits: u64,
    pub control_hits: u64,
    pub treatment_admitted: u64,
    pub control_admitted: u64,
    pub delta_admitted: i64,
    pub b_treatment_only: u64,
    pub c_control_only: u64,
    pub discordant: u64,
    pub net_gain: i64,
    pub ceil_two_sqrt: u64,
    pub floor_fi: u64,
    pub beats_mcnemar: bool,
    pub admission_non_inferior: bool,
    pub passes: bool,
}

/// Score a paired one-sided McNemar contrast `(treatment_hit, treatment_admitted)`
/// vs `(control_hit, control_admitted)`.
///
/// When `with_min_floor` is true (the registered DEV bar from `#2028` comment
/// `6356034555`), the threshold is `F_I = max(24, ceil(2 * sqrt(b + c)))`.
/// When `with_min_floor` is false (the sealed fresh-V2 replication rule from
/// `#2028` comment `6379430093`), the threshold is `ceil(2 * sqrt(b + c))`
/// without the 24-row floor.
/// When `require_admission_non_inferior` is true (`H_pos` vs `C_match`),
/// `passes` requires both `b - c >= threshold` and `delta_admitted >= 0`.
pub fn score_position_contrast(
    pairs: &[(bool, bool, bool, bool)],
    with_min_floor: bool,
    require_admission_non_inferior: bool,
) -> Result<PositionPairedCount> {
    if pairs.is_empty() {
        return Err(invalid(
            "position paired contrast requires at least one row",
        ));
    }
    let mut treatment_hits = 0u64;
    let mut control_hits = 0u64;
    let mut treatment_admitted = 0u64;
    let mut control_admitted = 0u64;
    let mut b_treatment_only = 0u64;
    let mut c_control_only = 0u64;
    for &(t_hit, t_adm, c_hit, c_adm) in pairs {
        if t_hit && !t_adm {
            return Err(invalid(
                "treatment top-1 hit cannot occur on an unadmitted target row",
            ));
        }
        if c_hit && !c_adm {
            return Err(invalid(
                "control top-1 hit cannot occur on an unadmitted target row",
            ));
        }
        if t_hit {
            treatment_hits += 1;
        }
        if c_hit {
            control_hits += 1;
        }
        if t_adm {
            treatment_admitted += 1;
        }
        if c_adm {
            control_admitted += 1;
        }
        match (t_hit, c_hit) {
            (true, false) => b_treatment_only += 1,
            (false, true) => c_control_only += 1,
            (true, true) | (false, false) => {}
        }
    }
    let discordant = b_treatment_only + c_control_only;
    let ceil_two_sqrt = integer_ceil_two_sqrt(discordant);
    let floor_fi = if with_min_floor {
        ceil_two_sqrt.max(MCNEMAR_MIN_FLOOR)
    } else {
        ceil_two_sqrt
    };
    let net_gain = i64::try_from(b_treatment_only).unwrap_or(i64::MAX)
        - i64::try_from(c_control_only).unwrap_or(i64::MAX);
    let delta_admitted = i64::try_from(treatment_admitted).unwrap_or(i64::MAX)
        - i64::try_from(control_admitted).unwrap_or(i64::MAX);
    let threshold = i64::try_from(floor_fi).unwrap_or(i64::MAX);
    let beats_mcnemar = net_gain >= threshold;
    let admission_non_inferior = delta_admitted >= 0;
    let passes = if require_admission_non_inferior {
        beats_mcnemar && admission_non_inferior
    } else {
        beats_mcnemar
    };
    Ok(PositionPairedCount {
        rows: pairs.len(),
        treatment_hits,
        control_hits,
        treatment_admitted,
        control_admitted,
        delta_admitted,
        b_treatment_only,
        c_control_only,
        discordant,
        net_gain,
        ceil_two_sqrt,
        floor_fi,
        beats_mcnemar,
        admission_non_inferior,
        passes,
    })
}

/// Complete single-seed DEV evaluation across full DEV (`14,287`), `P_succ`
/// (`3,847`), and the frozen multi-occurrence order-sensitive slice (`1,620`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionSeedEvaluation {
    pub seed: u64,
    pub dev_full_vs_c_match: PositionPairedCount,
    pub dev_psucc_vs_c_match: PositionPairedCount,
    pub dev_order_slice_vs_c_match: PositionPairedCount,
    pub dev_full_vs_c_shuf: PositionPairedCount,
    pub dev_psucc_vs_c_shuf: PositionPairedCount,
    pub dev_order_slice_vs_c_shuf: PositionPairedCount,
    pub dev_keep: bool,
}

/// Score one seed's DEV observations against the registered §3 bar (`#2028`
/// comment `6356034555`), verifying that the `is_order_sensitive` row ordinals
/// match the frozen slice ordinals exactly.
pub fn evaluate_position_order_seed(
    seed: u64,
    observations: &[PositionRowObservation],
    frozen_slice_ordinals: &[u64],
) -> Result<PositionSeedEvaluation> {
    if observations.is_empty() {
        return Err(invalid("position seed evaluation requires non-empty rows"));
    }
    let mut prev_ord: Option<u64> = None;
    let mut observed_slice_ordinals = Vec::with_capacity(frozen_slice_ordinals.len());
    for obs in observations {
        if let Some(prev) = prev_ord {
            if obs.row_ordinal <= prev {
                return Err(invalid(
                    "position row observations must have strictly ascending row_ordinal",
                ));
            }
        }
        prev_ord = Some(obs.row_ordinal);
        if obs.is_order_sensitive {
            observed_slice_ordinals.push(obs.row_ordinal);
        }
    }
    if observed_slice_ordinals != frozen_slice_ordinals {
        return Err(invalid(
            "observed order-sensitive row ordinals do not match the frozen order slice",
        ));
    }
    let map_c_match = |pred: fn(&PositionRowObservation) -> bool| -> Vec<(bool, bool, bool, bool)> {
        observations
            .iter()
            .filter(|o| pred(o))
            .map(|o| {
                (
                    o.h_pos_ordered_hit,
                    o.h_pos_ordered_admitted,
                    o.c_match_ordered_hit,
                    o.c_match_ordered_admitted,
                )
            })
            .collect()
    };
    let map_c_shuf = |pred: fn(&PositionRowObservation) -> bool| -> Vec<(bool, bool, bool, bool)> {
        observations
            .iter()
            .filter(|o| pred(o))
            .map(|o| {
                (
                    o.h_pos_ordered_hit,
                    o.h_pos_ordered_admitted,
                    o.c_shuf_hit,
                    o.c_shuf_admitted,
                )
            })
            .collect()
    };

    let dev_full_vs_c_match = score_position_contrast(&map_c_match(|_| true), true, true)?;
    let dev_psucc_vs_c_match =
        score_position_contrast(&map_c_match(|o| o.is_copy_successor), true, true)?;
    let dev_order_slice_vs_c_match =
        score_position_contrast(&map_c_match(|o| o.is_order_sensitive), true, true)?;

    let dev_full_vs_c_shuf = score_position_contrast(&map_c_shuf(|_| true), true, false)?;
    let dev_psucc_vs_c_shuf =
        score_position_contrast(&map_c_shuf(|o| o.is_copy_successor), true, false)?;
    let dev_order_slice_vs_c_shuf =
        score_position_contrast(&map_c_shuf(|o| o.is_order_sensitive), true, false)?;

    let dev_keep = dev_full_vs_c_match.passes
        && dev_psucc_vs_c_match.passes
        && dev_order_slice_vs_c_match.passes
        && dev_full_vs_c_shuf.passes
        && dev_psucc_vs_c_shuf.passes
        && dev_order_slice_vs_c_shuf.passes;

    Ok(PositionSeedEvaluation {
        seed,
        dev_full_vs_c_match,
        dev_psucc_vs_c_match,
        dev_order_slice_vs_c_match,
        dev_full_vs_c_shuf,
        dev_psucc_vs_c_shuf,
        dev_order_slice_vs_c_shuf,
        dev_keep,
    })
}

/// Three-branch outcome decision table (`CHARTER.md` §4.5, D22) for the
/// pre-registered `PositionHolonomyV1` order test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionOrderVerdict {
    /// Both seeds pass all DEV contrasts and replicate on the sealed fresh V2 draw.
    PassPromote,
    /// At least one seed misses a DEV contrast; fresh V2 stays unopened and sealed.
    FailDevCloseConfig,
    /// Both seeds pass DEV (`KEEP`), but fresh V2 misses `b - c >= ceil(2 * sqrt(b + c))`.
    FailTransferCloseConfig,
}

/// Evaluate the two-seed DEV outcome and conditional fresh-V2 replication under
/// the owner's `2026-10-23T02:36:36Z` (`#2028` comment `6356034555`) and
/// `2026-10-24T03:41:41Z` (`#2028` comment `6379430093`) rulings.
pub fn grade_position_order_two_seeds(
    seed1: &PositionSeedEvaluation,
    seed2: &PositionSeedEvaluation,
    fresh_v2_replicates_both_seeds: Option<bool>,
) -> Result<PositionOrderVerdict> {
    if seed1.seed == seed2.seed {
        return Err(invalid(
            "two-seed position evaluation requires distinct seeds",
        ));
    }
    if !(seed1.dev_keep && seed2.dev_keep) {
        if fresh_v2_replicates_both_seeds.is_some() {
            return Err(invalid(
                "sealed fresh V2 draw must remain unopened when DEV does not achieve KEEP on both seeds",
            ));
        }
        return Ok(PositionOrderVerdict::FailDevCloseConfig);
    }
    match fresh_v2_replicates_both_seeds {
        Some(true) => Ok(PositionOrderVerdict::PassPromote),
        Some(false) => Ok(PositionOrderVerdict::FailTransferCloseConfig),
        None => Err(invalid(
            "DEV achieved KEEP on both seeds; fresh V2 transfer evaluation is required to decide promotion",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::{BosIdentity, CorpusSource, FilePin, Split};

    fn window(tokens: &[u32]) -> Vec<Identity> {
        tokens
            .iter()
            .enumerate()
            .map(|(p, &token)| Identity {
                token,
                occurrence: p as u64,
                record: 0,
                version: 0,
            })
            .collect()
    }

    #[test]
    fn two_occurrences_with_distinct_successors_are_selected() {
        assert!(is_order_sensitive(&window(&[5, 1, 5, 2, 9, 5])));
    }

    #[test]
    fn a_repeated_successor_is_rejected() {
        assert!(!is_order_sensitive(&window(&[5, 1, 5, 1, 9, 5])));
    }

    #[test]
    fn a_single_or_absent_earlier_occurrence_is_rejected() {
        assert!(!is_order_sensitive(&window(&[5, 1, 9, 5])));
        assert!(!is_order_sensitive(&window(&[1, 2, 3, 5])));
        assert!(!is_order_sensitive(&window(&[5])));
        assert!(!is_order_sensitive(&[]));
    }

    #[test]
    fn the_occurrence_just_before_the_query_counts() {
        // Prefix [5, 1, 5]: the successors are 1 and the query itself.
        assert!(is_order_sensitive(&window(&[5, 1, 5, 5])));
        // Prefix [5, 5]: the successors are 5 (the second 5) and the query 5, so not distinct.
        assert!(!is_order_sensitive(&window(&[5, 5, 5])));
    }

    #[test]
    fn three_occurrences_need_every_successor_distinct() {
        // Successors 1, 2, 1: not pairwise distinct, though two distinct tokens occur.
        let w = window(&[5, 1, 5, 2, 5, 1, 5]);
        assert!(!is_order_sensitive(&w));
        assert!(has_two_distinct_successors(&w));
        assert!(is_order_sensitive(&window(&[5, 1, 5, 2, 5, 3, 5])));
    }

    #[test]
    fn select_scans_the_corpus_windows_in_row_order() -> Result<()> {
        // Two documents. Eligible targets at positions 2, 7 and 10 (ordinals 0, 1, 2).
        // Ordinal 1's window is [0, 5, 1, 5, 2, 9, 5]: query 5, successors 1 and 2.
        let tokens: [u16; 11] = [0, 5, 1, 5, 2, 9, 5, 7, 0, 3, 4];
        let mask = [0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1];
        let mut bytes = vec![0; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&16u32.to_le_bytes());
        for t in tokens {
            bytes.extend_from_slice(&t.to_le_bytes());
        }
        let source = CorpusSource {
            split: Split::Dev,
            tokens: FilePin {
                path: "fixture.uort".into(),
                sha256: sha256(&bytes),
            },
            mask: FilePin {
                path: "fixture.mask".into(),
                sha256: sha256(&mask),
            },
            expected_payload_tokens: tokens.len() as u64,
            expected_eligible_targets: 3,
        };
        let bos = BosIdentity {
            token: "<|bos|>".into(),
            id: 0,
        };
        let corpus = PreparedCorpus::from_bytes(source, &bytes, &mask, 16, bos)?;
        let slice = select(&corpus)?;
        assert_eq!(slice.rows, vec![(1, 7)]);
        assert_eq!(slice.scanned, 3);
        assert_eq!(slice.loose, 1);
        assert_eq!(render(&slice), "row_ordinal\ttarget_position\n1\t7\n");
        Ok(())
    }

    #[test]
    fn committed_frozen_order_slice_matches_pinned_sha256_and_1620_rows() -> Result<()> {
        let path = format!(
            "{}/../../docs/labs/m1-position-mechanism/order-slice-v1.tsv",
            env!("CARGO_MANIFEST_DIR")
        );
        let tsv = std::fs::read_to_string(&path)
            .map_err(|e| invalid(format!("failed to read {path}: {e}")))?;
        let rows = parse_frozen_slice(&tsv)?;
        assert_eq!(rows.len(), FROZEN_ORDER_SLICE_ROWS);
        let Some(&(first_ord, first_pos)) = rows.first() else {
            return Err(invalid("expected non-empty frozen order slice"));
        };
        let Some(&(last_ord, last_pos)) = rows.last() else {
            return Err(invalid("expected non-empty frozen order slice"));
        };
        assert_eq!((first_ord, first_pos), (0, 41));
        assert!(last_ord < PINNED_DEV_ELIGIBLE_ROWS);
        assert!(last_pos > first_pos);

        // Tampering with the TSV is refused by the SHA-256 pin check.
        let mut tampered = tsv.clone();
        tampered.push('\n');
        assert!(parse_frozen_slice(&tampered).is_err());
        Ok(())
    }

    #[test]
    fn shuffle_prefix_window_preserves_query_multiset_and_matches_root_shuffle() -> Result<()> {
        let tokens: Vec<u32> = (0..16).collect();
        let w = window(&tokens);
        let roots: Vec<u8> = (0..16u8).collect();
        let shuf_w = shuffle_prefix_window(&w, 42)?;
        let shuf_r = crate::position_mechanism::shuffle_prefix_roots(&roots, 42)?;
        assert_eq!(shuf_w.len(), w.len());
        assert_eq!(shuf_w.last(), w.last());
        let shuf_tokens: Vec<u8> = shuf_w.iter().map(|id| id.token as u8).collect();
        assert_eq!(shuf_tokens, shuf_r);

        let mut sorted_orig: Vec<u32> = w[..15].iter().map(|id| id.token).collect();
        let mut sorted_shuf: Vec<u32> = shuf_w[..15].iter().map(|id| id.token).collect();
        sorted_orig.sort_unstable();
        sorted_shuf.sort_unstable();
        assert_eq!(sorted_orig, sorted_shuf);
        assert_ne!(shuf_w[..15], w[..15]);
        assert!(shuffle_prefix_window(&[], 42).is_err());
        Ok(())
    }

    #[test]
    fn integer_ceil_two_sqrt_and_paired_mcnemar_scorer_verify_floor_and_admission() -> Result<()> {
        assert_eq!(integer_ceil_two_sqrt(0), 0);
        assert_eq!(integer_ceil_two_sqrt(1), 2);
        assert_eq!(integer_ceil_two_sqrt(2), 3);
        assert_eq!(integer_ceil_two_sqrt(9), 6);
        assert_eq!(integer_ceil_two_sqrt(144), 24);
        assert_eq!(integer_ceil_two_sqrt(145), 25);

        // 24 treatment-only wins, 0 control-only wins -> discordant = 24, floor = max(24, 10) = 24 -> PASS.
        let mut pairs = vec![(true, true, false, true); 24];
        pairs.extend(vec![(false, true, false, true); 10]);
        let count = score_position_contrast(&pairs, true, true)?;
        assert_eq!(count.b_treatment_only, 24);
        assert_eq!(count.c_control_only, 0);
        assert_eq!(count.floor_fi, 24);
        assert_eq!(count.net_gain, 24);
        assert!(count.beats_mcnemar);
        assert!(count.admission_non_inferior);
        assert!(count.passes);

        // Admission drop (`delta_admitted = -1`) fails `H_pos vs C_match` even when `b - c >= 24`.
        let mut adm_drop = vec![(true, true, false, true); 24];
        adm_drop.push((false, false, false, true));
        let dropped = score_position_contrast(&adm_drop, true, true)?;
        assert!(dropped.beats_mcnemar);
        assert!(!dropped.admission_non_inferior);
        assert!(!dropped.passes);

        // Fresh-V2 replication rule (`with_min_floor = false`, #2028 comment 6379430093):
        // b = 9, c = 0 -> b + c = 9, ceil(2 * sqrt(9)) = 6 <= 9 -> PASS without 24-row floor.
        let fresh_pairs = vec![(true, true, false, true); 9];
        let fresh = score_position_contrast(&fresh_pairs, false, false)?;
        assert_eq!(fresh.floor_fi, 6);
        assert!(fresh.passes);

        // Impossible hit on unadmitted row is refused.
        assert!(score_position_contrast(&[(true, false, false, true)], true, true).is_err());
        Ok(())
    }

    #[test]
    fn evaluate_seed_and_two_seed_verdict_enforces_sealed_fresh_v2_rule() -> Result<()> {
        let frozen_ordinals: Vec<u64> = (0..24).collect();
        let observations: Vec<PositionRowObservation> = (0..24)
            .map(|row_ordinal| PositionRowObservation {
                row_ordinal,
                is_copy_successor: true,
                is_order_sensitive: true,
                h_pos_ordered_hit: true,
                h_pos_ordered_admitted: true,
                c_match_ordered_hit: false,
                c_match_ordered_admitted: true,
                c_shuf_hit: false,
                c_shuf_admitted: true,
            })
            .collect();
        let seed1 = evaluate_position_order_seed(1, &observations, &frozen_ordinals)?;
        let seed2 = evaluate_position_order_seed(2, &observations, &frozen_ordinals)?;
        assert!(seed1.dev_keep);
        assert!(seed2.dev_keep);
        assert_eq!(
            grade_position_order_two_seeds(&seed1, &seed2, Some(true))?,
            PositionOrderVerdict::PassPromote
        );
        assert_eq!(
            grade_position_order_two_seeds(&seed1, &seed2, Some(false))?,
            PositionOrderVerdict::FailTransferCloseConfig
        );
        assert!(grade_position_order_two_seeds(&seed1, &seed2, None).is_err());

        // When a seed misses DEV, fresh V2 must stay unopened (`None` -> FailDevCloseConfig; `Some(_)` -> Err).
        let mut failing_obs = observations.clone();
        failing_obs[0].h_pos_ordered_hit = false;
        let failing_seed2 = evaluate_position_order_seed(2, &failing_obs, &frozen_ordinals)?;
        assert!(!failing_seed2.dev_keep);
        assert_eq!(
            grade_position_order_two_seeds(&seed1, &failing_seed2, None)?,
            PositionOrderVerdict::FailDevCloseConfig
        );
        assert!(grade_position_order_two_seeds(&seed1, &failing_seed2, Some(true)).is_err());
        Ok(())
    }
}
