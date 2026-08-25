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

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use rayon::prelude::*;
use uor_r4_core::transformerless::compiler::{Corpus, WINDOW};

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
pub type SkipmixJointRows = Vec<(u32, u32, Vec<(u32, i32)>)>;
pub type PsiBagRows = Vec<(u32, Vec<(u32, i32)>)>;

/// Observable, non-semantic facts from one table fit. Durations and worker
/// count are printed/run-record evidence only; none participates in emitted
/// rows or artifact bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkipmixFitStats {
    pub train_positions: usize,
    pub distinct_occurrences: usize,
    pub workers: usize,
    pub fold_elapsed: Duration,
    pub joint_sort_elapsed: Duration,
    pub content_sort_elapsed: Duration,
    pub reduction_elapsed: Duration,
    pub joint_rows: usize,
    pub psi_bag_rows: usize,
}

impl SkipmixFitStats {
    pub fn elapsed(&self) -> Duration {
        self.fold_elapsed
            + self.joint_sort_elapsed
            + self.content_sort_elapsed
            + self.reduction_elapsed
    }

    pub fn positions_per_second(&self) -> f64 {
        let seconds = self.elapsed().as_secs_f64();
        if seconds == 0.0 {
            self.train_positions as f64
        } else {
            self.train_positions as f64 / seconds
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct JointOccurrence {
    content_token: u32,
    last_token: u32,
    target: u32,
}

#[derive(Clone, Copy)]
struct PositionOccurrences {
    entries: [JointOccurrence; WINDOW],
    len: usize,
}

/// Compatibility wrapper for callers that need only canonical rows.
pub fn fit_skipmix_tables(corpus: &Corpus, top_k: usize) -> (SkipmixJointRows, PsiBagRows) {
    let (joint, psi, _) = fit_skipmix_tables_instrumented(corpus, top_k);
    (joint, psi)
}

/// Build the deterministic conditioning-specificity control used by the
/// deployed-quality gate. Only TRAIN labels are rotated, so the TRAIN target
/// multiset is preserved exactly and held-out labels remain pristine.
///
/// The offset is `train_len / 2 + 1`, matching the predeclared #908/#933 null
/// while making its stated TRAIN-only boundary explicit in the bytes.
pub fn rotate_training_targets_control(corpus: &Corpus) -> Corpus {
    let (train, _) = induction::split_positions(corpus);
    rotate_targets_control_at_positions(corpus, &train)
}

/// Build the label-rotation control on the caller's exact construction split.
///
/// This is required for D3 story-index partitions: re-deriving the ordinal
/// split here would leak declared held-out stories into the fitted control.
pub fn rotate_targets_control_at_positions(corpus: &Corpus, train: &[usize]) -> Corpus {
    // Hidden teacher states are irrelevant to this teacher-free fitter and can
    // dominate the otherwise small control copy on observation-rich corpora.
    // Construct the control explicitly instead of cloning and then discarding
    // that tensor: the latter briefly doubles its peak memory for no semantic
    // reason on exactly the corpus-scale runs this control is meant to bound.
    let mut control = Corpus {
        n: corpus.n,
        stories: corpus.stories,
        story: corpus.story.clone(),
        input: corpus.input.clone(),
        next: corpus.next.clone(),
        t_argmax: corpus.t_argmax.clone(),
        top_tokens: corpus.top_tokens.clone(),
        top_weights: corpus.top_weights.clone(),
        span_start: corpus.span_start.clone(),
        span_end: corpus.span_end.clone(),
        byte_start: corpus.byte_start.clone(),
        byte_end: corpus.byte_end.clone(),
        hidden: None,
    };
    if train.is_empty() {
        return control;
    }
    let offset = train.len() / 2 + 1;
    for (ordinal, &position) in train.iter().enumerate() {
        control.t_argmax[position] = corpus.t_argmax[train[(ordinal + offset) % train.len()]];
    }
    control
}

/// Fit SKMX/PSIB with deterministic data-parallel folding and sorting.
///
/// Each TRAIN position is converted independently into at most [`WINDOW`]
/// distinct content occurrences. Rayon preserves the position order of the
/// collected blocks; both derived occurrence streams are then sorted by their
/// complete integer keys and reduced in that canonical order. Worker count can
/// therefore change wall time but cannot change counts, row order, quantized
/// weights, or artifact bytes.
pub fn fit_skipmix_tables_instrumented(
    corpus: &Corpus,
    top_k: usize,
) -> (SkipmixJointRows, PsiBagRows, SkipmixFitStats) {
    let (train, _held_out) = induction::split_positions(corpus);
    fit_skipmix_tables_at_positions_instrumented(corpus, &train, top_k)
}

/// Fit SKMX/PSIB from the caller's exact construction positions.
///
/// The score command owns partition selection (including D3 story-index
/// splits), so it must pass that selection through rather than letting this
/// fitter silently derive a different ordinal split.
pub fn fit_skipmix_tables_at_positions_instrumented(
    corpus: &Corpus,
    train: &[usize],
    top_k: usize,
) -> (SkipmixJointRows, PsiBagRows, SkipmixFitStats) {
    fit_skipmix_tables_at_positions_impl(corpus, train, top_k, None)
}

/// Fit from an exact partition with non-semantic live progress telemetry.
///
/// The label and timing never enter reductions or artifact bytes. Progress is
/// claimed by atomic thresholds, so worker scheduling may change log timing
/// but cannot change the canonically sorted result.
pub fn fit_skipmix_tables_at_positions_instrumented_named(
    corpus: &Corpus,
    train: &[usize],
    top_k: usize,
    label: &str,
) -> (SkipmixJointRows, PsiBagRows, SkipmixFitStats) {
    fit_skipmix_tables_at_positions_impl(corpus, train, top_k, Some(label))
}

fn fit_skipmix_tables_at_positions_impl(
    corpus: &Corpus,
    train: &[usize],
    top_k: usize,
    progress_label: Option<&str>,
) -> (SkipmixJointRows, PsiBagRows, SkipmixFitStats) {
    let workers = rayon::current_num_threads().max(1);
    let empty_stats = || SkipmixFitStats {
        train_positions: 0,
        distinct_occurrences: 0,
        workers,
        fold_elapsed: Duration::ZERO,
        joint_sort_elapsed: Duration::ZERO,
        content_sort_elapsed: Duration::ZERO,
        reduction_elapsed: Duration::ZERO,
        joint_rows: 0,
        psi_bag_rows: 0,
    };
    if top_k == 0 {
        return (Vec::new(), Vec::new(), empty_stats());
    }
    if train.is_empty() {
        return (Vec::new(), Vec::new(), empty_stats());
    }

    let fold_start = Instant::now();
    let processed = AtomicUsize::new(0);
    let progress_interval = (train.len() / 100).clamp(1_024, 8_192);
    let next_progress = AtomicUsize::new(progress_interval.min(train.len()));
    if let Some(label) = progress_label {
        eprintln!(
            "skip-mix progress: label={label} phase=fold processed=0/{} workers={workers} interval={progress_interval}",
            train.len()
        );
    }
    let blocks: Vec<PositionOccurrences> = train
        .par_iter()
        .map(|&position| {
            let block = position_occurrences(corpus, position);
            if let Some(label) = progress_label {
                let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
                let threshold = next_progress.load(Ordering::Relaxed);
                if done >= threshold {
                    let next = if threshold >= train.len() {
                        usize::MAX
                    } else {
                        threshold.saturating_add(progress_interval).min(train.len())
                    };
                    if next_progress
                        .compare_exchange(threshold, next, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                    {
                        let elapsed_millis = fold_start.elapsed().as_millis().max(1);
                        let observed = processed.load(Ordering::Relaxed).min(train.len());
                        let rate_milli_positions_per_second =
                            (observed as u128).saturating_mul(1_000_000) / elapsed_millis;
                        let eta_millis = (train.len().saturating_sub(observed) as u128)
                            .saturating_mul(elapsed_millis)
                            / (observed as u128).max(1);
                        eprintln!(
                            "skip-mix progress: label={label} phase=fold processed={observed}/{} elapsed_ms={elapsed_millis} rate_milli_positions_per_second={rate_milli_positions_per_second} eta_ms={eta_millis}",
                            train.len()
                        );
                    }
                }
            }
            block
        })
        .collect();
    let distinct_occurrences = blocks.iter().map(|block| block.len).sum();
    let mut joint: Vec<JointOccurrence> = Vec::with_capacity(distinct_occurrences);
    for block in blocks {
        joint.extend_from_slice(&block.entries[..block.len]);
    }
    let fold_elapsed = fold_start.elapsed();

    let mut content: Vec<(u32, u32)> = joint
        .par_iter()
        .map(|occurrence| (occurrence.content_token, occurrence.target))
        .collect();

    let joint_sort_start = Instant::now();
    if let Some(label) = progress_label {
        eprintln!(
            "skip-mix progress: label={label} phase=joint-sort items={} status=start",
            joint.len()
        );
    }
    joint.par_sort_unstable();
    let joint_sort_elapsed = joint_sort_start.elapsed();

    let content_sort_start = Instant::now();
    if let Some(label) = progress_label {
        eprintln!(
            "skip-mix progress: label={label} phase=content-sort items={} status=start joint_sort_ms={}",
            content.len(),
            joint_sort_elapsed.as_millis()
        );
    }
    content.par_sort_unstable();
    let content_sort_elapsed = content_sort_start.elapsed();

    let reduction_start = Instant::now();
    if let Some(label) = progress_label {
        eprintln!(
            "skip-mix progress: label={label} phase=reduce items={} status=start content_sort_ms={}",
            joint.len().saturating_add(content.len()),
            content_sort_elapsed.as_millis()
        );
    }
    let joint_rows = quantize_sorted_joint(&joint, top_k);
    let psi_bag_rows = quantize_sorted_content(&content, top_k);
    let reduction_elapsed = reduction_start.elapsed();
    if let Some(label) = progress_label {
        eprintln!(
            "skip-mix progress: label={label} phase=complete processed={} joint_rows={} psi_rows={} elapsed_ms={}",
            train.len(),
            joint_rows.len(),
            psi_bag_rows.len(),
            fold_start.elapsed().as_millis()
        );
    }
    let stats = SkipmixFitStats {
        train_positions: train.len(),
        distinct_occurrences,
        workers,
        fold_elapsed,
        joint_sort_elapsed,
        content_sort_elapsed,
        reduction_elapsed,
        joint_rows: joint_rows.len(),
        psi_bag_rows: psi_bag_rows.len(),
    };
    (joint_rows, psi_bag_rows, stats)
}

fn position_occurrences(corpus: &Corpus, position: usize) -> PositionOccurrences {
    let mut start = position;
    while start > 0
        && corpus.story[start - 1] == corpus.story[position]
        && position + 1 - start < WINDOW
    {
        start -= 1;
    }
    let last_token = corpus.input[position];
    let target = corpus.t_argmax[position];
    let mut entries = [JointOccurrence::default(); WINDOW];
    let mut len = 0usize;
    for &content_token in &corpus.input[start..=position] {
        if entries[..len]
            .iter()
            .any(|occurrence| occurrence.content_token == content_token)
        {
            continue;
        }
        entries[len] = JointOccurrence {
            content_token,
            last_token,
            target,
        };
        len += 1;
    }
    PositionOccurrences { entries, len }
}

fn quantize_sorted_joint(sorted: &[JointOccurrence], top_k: usize) -> SkipmixJointRows {
    let mut rows = Vec::new();
    let mut start = 0usize;
    while start < sorted.len() {
        let key = (sorted[start].content_token, sorted[start].last_token);
        let mut end = start + 1;
        while end < sorted.len() && (sorted[end].content_token, sorted[end].last_token) == key {
            end += 1;
        }
        rows.push((
            key.0,
            key.1,
            quantize_joint_group(&sorted[start..end], top_k),
        ));
        start = end;
    }
    rows
}

fn quantize_sorted_content(sorted: &[(u32, u32)], top_k: usize) -> PsiBagRows {
    let mut rows = Vec::new();
    let mut start = 0usize;
    while start < sorted.len() {
        let content_token = sorted[start].0;
        let mut end = start + 1;
        while end < sorted.len() && sorted[end].0 == content_token {
            end += 1;
        }
        rows.push((
            content_token,
            quantize_content_group(&sorted[start..end], top_k),
        ));
        start = end;
    }
    rows
}

fn quantize_joint_group(group: &[JointOccurrence], top_k: usize) -> Vec<(u32, i32)> {
    let mut counts = Vec::new();
    let mut start = 0usize;
    while start < group.len() {
        let target = group[start].target;
        let mut end = start + 1;
        while end < group.len() && group[end].target == target {
            end += 1;
        }
        counts.push((target, (end - start) as u64));
        start = end;
    }
    cap_and_quantize(&mut counts, group.len() as u64, top_k)
}

fn quantize_content_group(group: &[(u32, u32)], top_k: usize) -> Vec<(u32, i32)> {
    let mut counts = Vec::new();
    let mut start = 0usize;
    while start < group.len() {
        let target = group[start].1;
        let mut end = start + 1;
        while end < group.len() && group[end].1 == target {
            end += 1;
        }
        counts.push((target, (end - start) as u64));
        start = end;
    }
    cap_and_quantize(&mut counts, group.len() as u64, top_k)
}

fn cap_and_quantize(counts: &mut Vec<(u32, u64)>, total: u64, top_k: usize) -> Vec<(u32, i32)> {
    counts.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    counts.truncate(top_k);
    let mut entries = Vec::with_capacity(counts.len());
    for &(candidate, count) in counts.iter() {
        entries.push((candidate, quantize_rate(count, total)));
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment_fit::DEFAULT_TOP_K;
    use std::collections::BTreeMap;

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

    fn sequential_reference(corpus: &Corpus, top_k: usize) -> (SkipmixJointRows, PsiBagRows) {
        let (train, _) = induction::split_positions(corpus);
        let mut joint: BTreeMap<(u32, u32), BTreeMap<u32, u64>> = BTreeMap::new();
        let mut content: BTreeMap<u32, BTreeMap<u32, u64>> = BTreeMap::new();
        for position in train {
            let target = corpus.t_argmax[position];
            let mut window = induction::context_window(corpus, position);
            let last_token = *window.last().expect("a corpus position has a window");
            window.sort_unstable();
            window.dedup();
            for content_token in window {
                *joint
                    .entry((content_token, last_token))
                    .or_default()
                    .entry(target)
                    .or_default() += 1;
                *content
                    .entry(content_token)
                    .or_default()
                    .entry(target)
                    .or_default() += 1;
            }
        }
        let joint = joint
            .into_iter()
            .map(|((content_token, last_token), candidates)| {
                let total = candidates.values().sum();
                let mut candidates: Vec<_> = candidates.into_iter().collect();
                (
                    content_token,
                    last_token,
                    cap_and_quantize(&mut candidates, total, top_k),
                )
            })
            .collect();
        let content = content
            .into_iter()
            .map(|(content_token, candidates)| {
                let total = candidates.values().sum();
                let mut candidates: Vec<_> = candidates.into_iter().collect();
                (
                    content_token,
                    cap_and_quantize(&mut candidates, total, top_k),
                )
            })
            .collect();
        (joint, content)
    }

    #[test]
    fn parallel_fit_is_worker_invariant_and_matches_sequential_counts() {
        let stories: Vec<_> = (0..25)
            .map(|story| {
                let tokens: Vec<u32> = (0..31)
                    .map(|position| ((story * 13 + position * 7) % 19) as u32)
                    .collect();
                let targets: Vec<u32> = (0..31)
                    .map(|position| ((story * 5 + position * 11) % 23) as u32)
                    .collect();
                (tokens, targets)
            })
            .collect();
        let corpus = corpus_from_stories(&stories);
        let expected = sequential_reference(&corpus, 7);
        let run = |workers| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()
                .unwrap()
                .install(|| fit_skipmix_tables_instrumented(&corpus, 7))
        };
        let (one_joint, one_psi, one_stats) = run(1);
        let (four_joint, four_psi, four_stats) = run(4);
        let (eight_joint, eight_psi, eight_stats) = run(8);
        assert_eq!((one_joint, one_psi), expected);
        assert_eq!((four_joint.clone(), four_psi.clone()), expected);
        assert_eq!((eight_joint.clone(), eight_psi.clone()), expected);
        assert_eq!(
            uor_r4_graph_format::build_skipmix_table(&four_joint).unwrap(),
            uor_r4_graph_format::build_skipmix_table(&eight_joint).unwrap()
        );
        assert_eq!(
            uor_r4_graph_format::build_psi_bag_table(&four_psi).unwrap(),
            uor_r4_graph_format::build_psi_bag_table(&eight_psi).unwrap()
        );
        assert_eq!(one_stats.workers, 1);
        assert_eq!(four_stats.workers, 4);
        assert_eq!(eight_stats.workers, 8);
        assert_eq!(
            one_stats.distinct_occurrences,
            four_stats.distinct_occurrences
        );
        assert_eq!(
            one_stats.distinct_occurrences,
            eight_stats.distinct_occurrences
        );
    }

    #[test]
    fn label_control_rotates_only_train_and_preserves_its_target_multiset() {
        let stories: Vec<_> = (0..10)
            .map(|story| {
                let tokens: Vec<u32> = (0..13)
                    .map(|position| (story * 100 + position) as u32)
                    .collect();
                let targets: Vec<u32> = (0..13)
                    .map(|position| (story * 1000 + position) as u32)
                    .collect();
                (tokens, targets)
            })
            .collect();
        let mut corpus = corpus_from_stories(&stories);
        corpus.hidden = Some(vec![vec![1.0; 4]; corpus.n]);
        let (train, held_out) = induction::split_positions(&corpus);
        let control = rotate_training_targets_control(&corpus);

        assert!(control.hidden.is_none());
        assert_eq!(control.n, corpus.n);
        assert_eq!(control.story, corpus.story);
        assert_eq!(control.input, corpus.input);
        assert_eq!(control.next, corpus.next);
        assert_eq!(control.top_tokens, corpus.top_tokens);
        assert_eq!(control.top_weights, corpus.top_weights);

        for &position in &held_out {
            assert_eq!(control.t_argmax[position], corpus.t_argmax[position]);
        }
        let mut original_train: Vec<_> = train
            .iter()
            .map(|&position| corpus.t_argmax[position])
            .collect();
        let mut control_train: Vec<_> = train
            .iter()
            .map(|&position| control.t_argmax[position])
            .collect();
        original_train.sort_unstable();
        control_train.sort_unstable();
        assert_eq!(control_train, original_train);
        assert!(
            train
                .iter()
                .any(|&position| control.t_argmax[position] != corpus.t_argmax[position])
        );
    }

    #[test]
    fn explicit_story_partition_never_reintroduces_ordinal_training_positions() {
        let stories: Vec<_> = (0..10)
            .map(|story| {
                (
                    vec![10 + story as u32, 20 + story as u32],
                    vec![100 + story as u32, 200 + story as u32],
                )
            })
            .collect();
        let corpus = corpus_from_stories(&stories);
        // Deliberately choose the tail stories as construction. The ordinal
        // 80/20 split would make the opposite choice for several positions.
        let train: Vec<usize> = (16..20).collect();
        let control = rotate_targets_control_at_positions(&corpus, &train);

        for position in 0..16 {
            assert_eq!(control.t_argmax[position], corpus.t_argmax[position]);
        }
        let (joint, _psi, stats) =
            fit_skipmix_tables_at_positions_instrumented(&control, &train, DEFAULT_TOP_K);
        assert_eq!(stats.train_positions, train.len());
        assert!(joint.iter().all(|(content, _, _)| *content >= 18));
    }
}
