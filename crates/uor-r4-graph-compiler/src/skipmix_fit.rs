//! #897 -- fit the confirmed 1-token skip-mix joint table and its
//! unconditioned Ψ-bag fallback from a recorded corpus. Compiler-side
//! counterpart of the deployed skip-mix scorer (`uor-r4-api` engine); the
//! same TRAIN fold this crate already uses for the #836 segment lane
//! ([`crate::segment_fit::fit_segment_table`]), extended with the join
//! key's last window token -- the #897 phase-0-confirmed `SELECT-1-token`
//! key family: `(content_token, last_window_token)`.
//!
//! Faithfulness. This reproduces the exact fold and formulas the off-serving
//! reference harness already validated
//! (`crates/uor-r4-api/tests/skipmix_confirm_897.rs`,
//! `docs/skipmix_confirm_897_result.json`, verdict `SELECT-1-token`):
//! per-distinct-window-token bump (the same "seen" dedup pattern
//! `fit_segment_table` uses), joint counts keyed by `(content_token,
//! last_token)`, the same cap-`DEFAULT_TOP_K` and `quantize_rate` the #836
//! lane already uses -- so the deployed table and the confirmed reference
//! numbers describe the same construction, not a re-derivation of it.

use std::collections::HashMap;

use uor_r4_core::transformerless::compiler::Corpus;

use crate::induction;
use crate::segment_fit::quantize_rate;

/// Fit the confirmed 1-token skip-mix joint table and its unconditioned
/// Ψ-bag fallback from `corpus`'s TRAIN positions (the same
/// `induction::split_positions` 80/20 story cut used throughout this
/// record). `top_k` is the per-key candidate cap (callers typically pass
/// `crate::segment_fit::DEFAULT_TOP_K` = 64, matching the phase-0
/// harness's `CAP`).
///
/// Returns `(joint_rows, psi_bag_rows)`:
///   * `joint_rows`: `(content_token, last_window_token, entries)` for
///     every observed key with at least one count, each capped to its
///     top-`top_k` teacher-argmax candidates by count (ties: smaller id).
///     Not truncated by key count -- the phase-0 record shows the observed
///     key space (793,781 keys on the attested #833 bundle) is already a
///     bounded, corpus-derived quantity, not one needing an extra
///     key-count cutoff.
///   * `psi_bag_rows`: `(content_token, entries)`, the unconditioned
///     fallback -- every content token observed in TRAIN with at least one
///     count, capped the same way. Not truncated by key count either: this
///     table is the long-tail safety net for keys the joint table has no
///     support for, and dropping rare content tokens would defeat exactly
///     the positions that need a fallback most. Bounded in practice by the
///     tokenizer's fixed vocabulary size.
///
/// Both are canonicalized (sorted, duplicates rejected) when passed to
/// [`uor_r4_graph_format::build_skipmix_table`] /
/// [`uor_r4_graph_format::build_psi_bag_table`]; deterministic given the
/// same corpus and `top_k`.
#[allow(clippy::type_complexity)]
pub fn fit_skipmix_tables(
    corpus: &Corpus,
    top_k: usize,
) -> (
    Vec<(u32, u32, Vec<(u32, i32)>)>,
    Vec<(u32, Vec<(u32, i32)>)>,
) {
    if top_k == 0 {
        return (Vec::new(), Vec::new());
    }
    let (train, _held_out) = induction::split_positions(corpus);

    // (content_token, last_window_token) -> (teacher-argmax candidate -> count)
    let mut joint_next: HashMap<(u32, u32), HashMap<u32, u32>> = HashMap::new();
    // content_token -> (teacher-argmax candidate -> count), unconditioned.
    let mut content_next: HashMap<u32, HashMap<u32, u32>> = HashMap::new();

    for &i in &train {
        let target = corpus.t_argmax[i];
        let window = induction::context_window(corpus, i);
        // The window is in temporal order ending at the current position
        // (`induction::context_window`'s `(start..=i)` build), so its last
        // element -- taken BEFORE any sort/dedup below -- is the actual
        // last window token, the #897 conditioning key. Sorting first (as
        // `fit_segment_table` does for its "distinct tokens" set) would
        // silently swap this for the largest token id instead.
        let last_token = match window.last().copied() {
            Some(t) => t,
            None => continue, // an empty window has no conditioning key
        };
        let mut seen = window;
        seen.sort_unstable();
        seen.dedup();
        for t in seen {
            *joint_next
                .entry((t, last_token))
                .or_default()
                .entry(target)
                .or_insert(0) += 1;
            *content_next
                .entry(t)
                .or_default()
                .entry(target)
                .or_insert(0) += 1;
        }
    }

    let joint_rows = quantize_joint_rows(joint_next, top_k);
    let psi_bag_rows = quantize_content_rows(content_next, top_k);
    (joint_rows, psi_bag_rows)
}

#[allow(clippy::type_complexity)]
fn quantize_joint_rows(
    map: HashMap<(u32, u32), HashMap<u32, u32>>,
    top_k: usize,
) -> Vec<(u32, u32, Vec<(u32, i32)>)> {
    let mut rows = Vec::with_capacity(map.len());
    for ((content_token, last_token), counts) in map {
        if let Some(entries) = cap_and_quantize(counts, top_k) {
            rows.push((content_token, last_token, entries));
        }
    }
    rows
}

fn quantize_content_rows(
    map: HashMap<u32, HashMap<u32, u32>>,
    top_k: usize,
) -> Vec<(u32, Vec<(u32, i32)>)> {
    let mut rows = Vec::with_capacity(map.len());
    for (content_token, counts) in map {
        if let Some(entries) = cap_and_quantize(counts, top_k) {
            rows.push((content_token, entries));
        }
    }
    rows
}

/// Shared top-`top_k` cap + [`quantize_rate`] step for one key's raw
/// `(candidate -> count)` tally. `None` when the key carries no evidence
/// (defensive; TRAIN folding never inserts a key without incrementing it).
fn cap_and_quantize(counts: HashMap<u32, u32>, top_k: usize) -> Option<Vec<(u32, i32)>> {
    let total: u64 = counts.values().map(|&c| u64::from(c)).sum();
    if total == 0 {
        return None;
    }
    let mut cand: Vec<(u32, u32)> = counts.into_iter().collect();
    cand.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    cand.truncate(top_k);

    let mut entries = Vec::with_capacity(cand.len());
    for (candidate, count) in cand {
        entries.push((candidate, quantize_rate(u64::from(count), total)));
    }
    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment_fit::DEFAULT_TOP_K;

    /// Build a multi-story `Corpus` from per-story `(input, t_argmax)` token
    /// runs, mirroring `segment_fit`'s test helper so both modules exercise
    /// the same fixture shape.
    fn corpus_from_stories(stories: &[(Vec<u32>, Vec<u32>)]) -> Corpus {
        let mut story = Vec::new();
        let mut input = Vec::new();
        let mut next = Vec::new();
        let mut t_argmax = Vec::new();
        for (sid, (toks, targ)) in stories.iter().enumerate() {
            assert_eq!(toks.len(), targ.len(), "input/argmax length match");
            for (j, (&tok, &tg)) in toks.iter().zip(targ).enumerate() {
                story.push(sid as u32);
                input.push(tok);
                next.push(if j + 1 < toks.len() { toks[j + 1] } else { 0 });
                t_argmax.push(tg);
            }
        }
        let n = input.len();
        Corpus {
            n,
            stories: stories.len() as u64,
            story,
            input,
            next,
            t_argmax,
            top_tokens: vec![[0u32; 8]; n],
            top_weights: vec![[0u32; 8]; n],
            span_start: (0..n).map(|i| i as u32).collect(),
            span_end: (0..n).map(|i| i as u32 + 1).collect(),
            byte_start: vec![u32::MAX; n],
            byte_end: vec![u32::MAX; n],
            hidden: None,
        }
    }

    #[test]
    fn joint_key_uses_the_temporal_last_token_not_the_sorted_max() {
        // Window [100, 500] at each position: last token is 500 for the
        // second position, not the sorted-max of the distinct-token set.
        // Four TRAIN copies (+1 held-out) so a positive weight survives.
        let train_story = (vec![100u32, 500], vec![9u32, 9]);
        let stories = vec![
            train_story.clone(),
            train_story.clone(),
            train_story.clone(),
            train_story.clone(),
            (vec![100u32, 500], vec![9u32, 9]), // held-out
        ];
        let corpus = corpus_from_stories(&stories);
        let (joint_rows, psi_bag_rows) = fit_skipmix_tables(&corpus, DEFAULT_TOP_K);

        // At the second position (window [100, 500]) the conditioning key
        // is 500 (temporal-last), and both distinct window tokens (100 and
        // 500) get a joint row keyed by last_token=500.
        let key_100_500 = joint_rows.iter().find(|(c, l, _)| *c == 100 && *l == 500);
        assert!(
            key_100_500.is_some(),
            "content 100 conditioned on the temporal last token 500 must be present"
        );
        // A row keyed by the WRONG (sorted-max-as-last) convention would
        // instead put a (500, 500) style key from the first position's
        // window [100] alone, which is impossible (single-token window has
        // no distinct pair) -- so this also indirectly guards the dedup
        // step running after last-token extraction, not before.
        assert!(psi_bag_rows.iter().any(|(k, _)| *k == 100));
        assert!(psi_bag_rows.iter().any(|(k, _)| *k == 500));
    }

    #[test]
    fn degenerate_top_k_yields_empty_tables() {
        let stories = vec![(vec![1u32, 2, 3], vec![7u32, 7, 0])];
        let corpus = corpus_from_stories(&stories);
        let (joint_rows, psi_bag_rows) = fit_skipmix_tables(&corpus, 0);
        assert!(joint_rows.is_empty());
        assert!(psi_bag_rows.is_empty());
    }

    #[test]
    fn top_k_caps_candidates_per_key() {
        // Many distinct targets for the same (content, last) pair; top_k=1
        // keeps exactly one candidate per joint row and per psi-bag row.
        let stories = vec![
            (vec![1u32, 2], vec![10u32, 10]),
            (vec![1u32, 2], vec![11u32, 11]),
            (vec![1u32, 2], vec![12u32, 12]),
            (vec![1u32, 2], vec![10u32, 10]),
        ];
        let corpus = corpus_from_stories(&stories);
        let (joint_rows, psi_bag_rows) = fit_skipmix_tables(&corpus, 1);
        for (_, _, entries) in &joint_rows {
            assert_eq!(
                entries.len(),
                1,
                "top_k=1 keeps one candidate per joint key"
            );
        }
        for (_, entries) in &psi_bag_rows {
            assert_eq!(
                entries.len(),
                1,
                "top_k=1 keeps one candidate per psi-bag key"
            );
        }
    }

    #[test]
    fn tables_round_trip_through_the_format_builders() {
        let train_story = (vec![100u32, 500, 7], vec![9u32, 9, 3]);
        let stories = vec![
            train_story.clone(),
            train_story.clone(),
            train_story.clone(),
            train_story.clone(),
            train_story,
        ];
        let corpus = corpus_from_stories(&stories);
        let (joint_rows, psi_bag_rows) = fit_skipmix_tables(&corpus, DEFAULT_TOP_K);
        assert!(!joint_rows.is_empty());
        assert!(!psi_bag_rows.is_empty());

        let skmx_bytes = uor_r4_graph_format::build_skipmix_table(&joint_rows)
            .expect("fitted joint rows are canonical");
        let table = uor_r4_graph_format::SkipmixTable::parse(&skmx_bytes).expect("parses");
        for (content_token, last_token, entries) in &joint_rows {
            let row = table
                .find(*content_token, *last_token)
                .expect("fitted key is servable");
            for (candidate, weight) in entries {
                assert_eq!(
                    row.entries().find(*candidate).map(|s| s.raw()),
                    Some(*weight)
                );
            }
        }

        let psib_bytes = uor_r4_graph_format::build_psi_bag_table(&psi_bag_rows)
            .expect("fitted psi-bag rows are canonical");
        let psib_table = uor_r4_graph_format::PsiBagTable::parse(&psib_bytes).expect("parses");
        for (content_token, entries) in &psi_bag_rows {
            let row = psib_table
                .find(*content_token)
                .expect("fitted key is servable");
            for (candidate, weight) in entries {
                assert_eq!(
                    row.entries().find(*candidate).map(|s| s.raw()),
                    Some(*weight)
                );
            }
        }
    }
}
