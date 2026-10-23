//! The position sheet's multi-occurrence, order-sensitive DEV slice
//! (`docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` §5, item #2843).
//!
//! Host-only selection over recorded token identities, like `is_copy_successor`: no model
//! execution, no floating point. A row's window is the document-bounded, C192-truncated window
//! `PreparedCorpus::batch` builds; its last token is the query.
use crate::corpus::PreparedCorpus;
use crate::{Identity, Result};

/// The window length the slice is defined at (the pinned M1 `Spec` context).
pub const CONTEXT: u32 = 192;
pub const SELECTION_POLICY: &str =
    "query-occurs-twice-in-prefix/pairwise-distinct-successors/C192-window/ascending-rows/v1";

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::{sha256, BosIdentity, CorpusSource, FilePin, Split};

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
}
