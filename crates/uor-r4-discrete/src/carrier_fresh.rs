//! The fresh slice for `carrier-credit-hard-selection` (#2819; prep sheet §4).
//!
//! Batches walk TRAIN's eligible targets in order from cursor 0, and each
//! window stays inside its document. A dose that reads the first
//! `visited_targets` eligible targets therefore never reads a document that
//! starts after the last of them. The fresh slice copies whole documents from
//! that point on into a separately pinned corpus, keeps the first
//! `fresh_targets` eligible targets whose (window, target) does not repeat a
//! visited row exactly, and masks only those. Windows are unchanged, because
//! the copied documents keep their own starts. This is host data preparation:
//! no model is read.

use crate::corpus::{self, PreparedCorpus};
use crate::{Error, Result};
use serde_json::{json, Value};
use std::collections::HashSet;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// The UORT corpus header and payload, as `PreparedCorpus::from_bytes` reads it.
pub fn uort_bytes(tokens: &[u32], vocabulary: u32) -> Result<Vec<u8>> {
    let mut bytes = vec![0u8; 64];
    bytes[..4].copy_from_slice(b"UORT");
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
    bytes[16..20].copy_from_slice(&vocabulary.to_le_bytes());
    for &id in tokens {
        let id = u16::try_from(id).map_err(|_| invalid("token ID exceeds u16"))?;
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    Ok(bytes)
}

pub struct FreshSlice {
    /// The fresh corpus, in UORT form.
    pub tokens: Vec<u8>,
    /// One byte per fresh token; 1 marks a kept target.
    pub mask: Vec<u8>,
    pub manifest: Value,
}

/// Exact identity of a row: its in-document window and its target.
fn row_key(tokens: &[u32], start: usize, position: usize) -> [u8; 32] {
    let mut bytes = Vec::with_capacity((position - start + 1) * 4);
    for &id in &tokens[start..=position] {
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    let digest = corpus::sha256(&bytes);
    let mut key = [0u8; 32];
    for (slot, pair) in key.iter_mut().zip(digest.as_bytes().chunks(2)) {
        *slot = u8::from_str_radix(std::str::from_utf8(pair).unwrap_or("00"), 16).unwrap_or(0);
    }
    key
}

/// Cut the fresh slice from `train` past its first `visited_targets` eligible
/// targets, keeping `fresh_targets` of them with `context`-token windows.
pub fn cut_fresh_slice(
    train: &PreparedCorpus,
    context: u32,
    visited_targets: usize,
    fresh_targets: usize,
) -> Result<FreshSlice> {
    let tokens = train.tokens();
    let eligible = train.eligible_positions();
    let bos = train.binding().bos.id;
    let vocabulary = train.binding().vocabulary;
    if visited_targets == 0 || fresh_targets == 0 || visited_targets >= eligible.len() {
        return Err(invalid(
            "fresh slice needs visited and fresh targets inside TRAIN",
        ));
    }
    let documents: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter_map(|(p, &id)| (id == bos).then_some(p))
        .collect();
    let document_start = |position: usize| {
        let at = documents.partition_point(|&start| start <= position);
        documents[at - 1]
    };
    let window_start = |position: usize| {
        position
            .saturating_sub(context as usize)
            .max(document_start(position))
    };
    let last_visited = eligible[visited_targets - 1];
    let first_fresh_document = *documents
        .iter()
        .find(|&&start| start > last_visited)
        .ok_or_else(|| invalid("no TRAIN document starts after the dose"))?;
    let visited: HashSet<[u8; 32]> = eligible[..visited_targets]
        .iter()
        .map(|&p| row_key(tokens, window_start(p), p))
        .collect();
    let first_candidate = eligible.partition_point(|&p| p < first_fresh_document);
    let (mut kept, mut duplicates, mut scanned) = (Vec::new(), 0u64, 0u64);
    for &p in &eligible[first_candidate..] {
        if kept.len() == fresh_targets {
            break;
        }
        scanned += 1;
        if visited.contains(&row_key(tokens, window_start(p), p)) {
            duplicates += 1;
        } else {
            kept.push(p);
        }
    }
    if kept.len() < fresh_targets {
        return Err(invalid("TRAIN holds too few fresh targets past the dose"));
    }
    let last_kept = *kept.last().expect("kept is nonempty");
    let end = documents
        .iter()
        .find(|&&start| start > last_kept)
        .copied()
        .unwrap_or(tokens.len());
    let fresh_tokens = &tokens[first_fresh_document..end];
    let mut mask = vec![0u8; fresh_tokens.len()];
    for &p in &kept {
        mask[p - first_fresh_document] = 1;
    }
    let bytes = uort_bytes(fresh_tokens, vocabulary)?;
    let manifest = json!({
        "schema": "uor-r4.carrier-fresh-slice/1",
        "source_tokens_sha256": train.binding().source.tokens.sha256,
        "source_mask_sha256": train.binding().source.mask.sha256,
        "context": context,
        "visited_targets": visited_targets,
        "last_visited_position": last_visited,
        "first_fresh_document_start": first_fresh_document,
        "first_kept_position": kept[0],
        "last_kept_position": last_kept,
        "candidates_scanned": scanned,
        "duplicates_dropped": duplicates,
        "kept_targets": kept.len(),
        "fresh_payload_tokens": fresh_tokens.len(),
        "fresh_tokens_sha256": corpus::sha256(&bytes),
        "fresh_mask_sha256": corpus::sha256(&mask),
        "rule": "whole TRAIN documents starting after the last visited target; first fresh_targets eligible targets whose (in-document window, target) repeats no visited row exactly",
    });
    Ok(FreshSlice {
        tokens: bytes,
        mask,
        manifest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::{BosIdentity, CorpusSource, FilePin, Split};

    fn corpus(split: Split, tokens: &[u32], mask: &[u8]) -> PreparedCorpus {
        let bytes = uort_bytes(tokens, 16).expect("uort");
        let eligible = tokens
            .iter()
            .zip(mask)
            .enumerate()
            .filter(|(p, (&id, &m))| *p > 0 && id != 0 && m == 1)
            .count() as u64;
        let source = CorpusSource {
            split,
            tokens: FilePin {
                path: "t".into(),
                sha256: corpus::sha256(&bytes),
            },
            mask: FilePin {
                path: "m".into(),
                sha256: corpus::sha256(mask),
            },
            expected_payload_tokens: tokens.len() as u64,
            expected_eligible_targets: eligible,
        };
        let bos = BosIdentity {
            token: "<|bos|>".into(),
            id: 0,
        };
        PreparedCorpus::from_bytes(source, &bytes, mask, 16, bos).expect("corpus")
    }

    #[test]
    fn the_slice_starts_past_the_dose_and_drops_exact_repeats() {
        // Four documents (BOS = 0). Visited: the first 3 targets (docs A, B).
        // Doc C repeats doc A's text exactly, so its rows repeat visited ones.
        let tokens = [0, 5, 6, 7, 0, 8, 9, 0, 5, 6, 7, 0, 3, 4, 2, 1];
        let mask = [0, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 1, 1];
        let train = corpus(Split::Train, &tokens, &mask);
        // Eligible positions: 1,2,5,6,8,9,12,13,14,15. Visited: 1,2,5.
        let slice = cut_fresh_slice(&train, 4, 3, 3).expect("slice");
        let m = &slice.manifest;
        assert_eq!(m["last_visited_position"], 5);
        assert_eq!(m["first_fresh_document_start"], 7);
        // Position 6 lies in doc B, which starts before the cut: never a candidate.
        // Doc C (8, 9) repeats doc A's (1, 2) rows exactly: both dropped.
        assert_eq!(m["duplicates_dropped"], 2);
        assert_eq!(m["first_kept_position"], 12);
        assert_eq!(m["last_kept_position"], 14);
        assert_eq!(m["kept_targets"], 3);
        let fresh_tokens: Vec<u32> = tokens[7..].to_vec();
        assert_eq!(slice.tokens, uort_bytes(&fresh_tokens, 16).unwrap());
        assert_eq!(slice.mask, vec![0, 0, 0, 0, 0, 1, 1, 1, 0]);
        // The slice loads as its own Fresh corpus with the kept targets only,
        // and each window equals the window the same row had in TRAIN.
        let fresh = corpus(Split::Fresh, &fresh_tokens, &slice.mask);
        assert_eq!(fresh.eligible_len(), 3);
        let a = fresh.batch(0, 4, 3).expect("fresh batch");
        let b = train.batch(6, 4, 3).expect("train batch");
        let ids = |batch: &corpus::CorpusBatch| -> Vec<Vec<u32>> {
            batch
                .windows
                .iter()
                .map(|w| w.iter().map(|i| i.token).collect())
                .collect()
        };
        assert_eq!(ids(&a), ids(&b));
        assert_eq!(a.targets, b.targets);
    }

    #[test]
    fn the_slice_refuses_when_too_few_targets_remain() {
        let tokens = [0, 5, 6, 0, 7, 8];
        let mask = [0, 1, 1, 0, 1, 1];
        let train = corpus(Split::Train, &tokens, &mask);
        assert!(cut_fresh_slice(&train, 4, 2, 3).is_err());
        assert!(cut_fresh_slice(&train, 4, 4, 1).is_err());
    }
}
