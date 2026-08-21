//! #836 4c-ii — fit the learned content-token→teacher-argmax segment table
//! from a recorded corpus. This is the compiler-side counterpart of the
//! deployed segment lane (`uor-r4-api` engine, #836 4c-i): the same
//! content→argmax co-occurrence tally as the #834 §6.2 Ψ segment reference
//! arm, capped to top-`K` per key, but **quantized to the integer ScoreQ
//! weights the P-4 serving path consumes**.
//!
//! Faithfulness of the lowering. The reference arm scored a candidate `c` by
//! `content_rate(c) = Σ_{t live} (count_t(c) / total_t) / ncontent`, a float
//! sum the normative hot path cannot compute (no divide, no float). Two of
//! those operations are **argmax-invariant** and are therefore discharged here,
//! at fit time, where floats are allowed:
//!
//!   * the per-key normalization `count_t(c) / total_t` becomes a fixed
//!     per-`(t, c)` integer weight `round((count/total) · 2^RATE_SCALE_SHIFT)`;
//!   * the per-prompt `/ ncontent` is a single positive scalar shared by every
//!     candidate of a given prompt, so dropping it never changes which
//!     candidate is the argmax.
//!
//! What remains at serve time is exactly `Σ_{t live} weight(t, c)` — a bounded
//! saturating integer sum (`table_contribution` in the engine) — so the served
//! argmax matches the reference arm's up to the fixed rescaling between the
//! base scorer's ScoreQ units and the reference's suffix-rate floats. The
//! per-prompt content weight λ of the reference is a global scalar folded into
//! `RATE_SCALE_SHIFT` (it multiplies every content contribution equally).

use std::collections::HashMap;

use uor_r4_core::transformerless::compiler::Corpus;

use crate::induction;

/// Default per-key cap on retained teacher-argmax candidates (top-`K`),
/// matching the #834 §6.2 reference arm's `CAP = 64`.
pub const DEFAULT_TOP_K: usize = 64;

/// Fixed-point scale applied to the normalized co-occurrence rate
/// `count(key→cand) / total(key) ∈ [0, 1]` to produce the integer ScoreQ
/// weight the serving lane sums. A rate of `1.0` (a content token whose
/// teacher-argmax is always the same candidate) maps to `1 << RATE_SCALE_SHIFT`
/// — deliberately the same magnitude as the recurrence lane's `boost`
/// (`1 << 20` in the selected descriptor), so a fully-predictive content token
/// carries one boost worth of evidence.
pub const RATE_SCALE_SHIFT: u32 = 20;

/// Fit the learned segment table from `corpus` over its TRAIN positions
/// (the `induction::split_positions` 80/20 story cut — held-out positions are
/// never fitted, exactly as the reference arm builds its tables from TRAIN
/// only).
///
/// Returns rows `(content_key, [(candidate, weight_raw_scoreq)])`:
///   * at most `max_keys` content keys, retained by descending total evidence
///     (ties broken by the smaller token id) so a bounded table keeps the
///     highest-signal content tokens;
///   * each key capped to its top-`top_k` teacher-argmax candidates by count
///     (ties broken by the smaller candidate id), matching the reference cap;
///   * every retained weight is `round((count / total) · 2^RATE_SCALE_SHIFT)`
///     computed in pure integer arithmetic and clamped to `[1, i32::MAX]`, so
///     no retained `(key, candidate)` pair is silently dropped to a zero
///     contribution.
///
/// The rows are canonicalized (keys and candidates sorted, duplicates
/// rejected) by [`uor_r4_graph_format::build_segment_lane`] when emitted, so
/// the caller need not pre-sort. Deterministic: the same corpus and bounds
/// produce byte-identical rows.
pub fn fit_segment_table(
    corpus: &Corpus,
    top_k: usize,
    max_keys: usize,
) -> Vec<(u32, Vec<(u32, i32)>)> {
    if top_k == 0 || max_keys == 0 {
        return Vec::new();
    }
    let (train, _held_out) = induction::split_positions(corpus);

    // content token -> (teacher-argmax candidate -> co-occurrence count).
    // One bump per *distinct* content token in the whole-prompt window, exactly
    // the reference arm's `seen.sort/dedup` step.
    let mut content_next: HashMap<u32, HashMap<u32, u32>> = HashMap::new();
    for &i in &train {
        let target = corpus.t_argmax[i];
        let mut seen = induction::context_window(corpus, i);
        seen.sort_unstable();
        seen.dedup();
        for t in seen {
            *content_next
                .entry(t)
                .or_default()
                .entry(target)
                .or_insert(0) += 1;
        }
    }

    // Retain the highest-evidence content keys (bounded table).
    let mut keyed: Vec<(u32, HashMap<u32, u32>)> = content_next.into_iter().collect();
    keyed.sort_unstable_by(|a, b| {
        let ta: u64 = a.1.values().map(|&c| u64::from(c)).sum();
        let tb: u64 = b.1.values().map(|&c| u64::from(c)).sum();
        tb.cmp(&ta).then(a.0.cmp(&b.0))
    });
    keyed.truncate(max_keys);

    let mut rows: Vec<(u32, Vec<(u32, i32)>)> = Vec::with_capacity(keyed.len());
    for (key, counts) in keyed {
        let total: u64 = counts.values().map(|&c| u64::from(c)).sum();
        if total == 0 {
            continue;
        }
        // top-K candidates by count, canonical tie-break (count desc, id asc).
        let mut cand: Vec<(u32, u32)> = counts.into_iter().collect();
        cand.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        cand.truncate(top_k);

        let mut entries: Vec<(u32, i32)> = Vec::with_capacity(cand.len());
        for (candidate, count) in cand {
            entries.push((candidate, quantize_rate(u64::from(count), total)));
        }
        if !entries.is_empty() {
            rows.push((key, entries));
        }
    }
    rows
}

/// Quantize a normalized rate `count / total ∈ [0, 1]` to an integer ScoreQ
/// weight `round((count / total) · 2^RATE_SCALE_SHIFT)`, in pure integer
/// arithmetic (round-half-up via `+ total/2`), clamped to `[1, i32::MAX]`.
///
/// The clamp floor of `1` keeps every retained pair a non-zero contribution:
/// a pair that survived the top-`K` cut carries evidence, and rounding it to a
/// zero weight would silently drop it from the served sum. `count <= total`, so
/// the pre-clamp value never exceeds `2^RATE_SCALE_SHIFT` and cannot overflow.
fn quantize_rate(count: u64, total: u64) -> i32 {
    debug_assert!(count <= total && total > 0);
    let numer = count << RATE_SCALE_SHIFT;
    let scaled = (numer + total / 2) / total;
    scaled.clamp(1, i32::MAX as u64) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a multi-story `Corpus` from per-story `(input, t_argmax)` token
    /// runs, laid out contiguously with matching `story` ids so
    /// `split_positions` and `context_window` see real story boundaries.
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
    fn quantize_maps_a_certain_rate_to_one_boost() {
        // A content token whose teacher-argmax is always the same candidate
        // (rate 1.0) earns exactly `1 << RATE_SCALE_SHIFT` — one boost.
        assert_eq!(quantize_rate(7, 7), 1 << RATE_SCALE_SHIFT);
        // Half the mass → half a boost (round-half-up).
        assert_eq!(quantize_rate(1, 2), 1 << (RATE_SCALE_SHIFT - 1));
        // A rare pair never rounds to a silent zero — the floor keeps it at 1.
        assert_eq!(quantize_rate(1, 5_000_000), 1);
    }

    #[test]
    fn fit_learns_the_dominant_content_to_argmax_association() {
        // Four TRAIN stories where every position that content token 100 is
        // live for has teacher-argmax 9 (the whole-prompt window carries 100
        // into both positions), plus one held-out story. The fitted row for key
        // 100 must rank candidate 9 first with the top weight.
        let train_story = (vec![100u32, 500], vec![9u32, 9]);
        let stories = vec![
            train_story.clone(),
            train_story.clone(),
            train_story.clone(),
            train_story.clone(),
            (vec![100u32, 500], vec![9u32, 9]), // held-out (story 4, >= cut)
        ];
        let corpus = corpus_from_stories(&stories);

        let rows = fit_segment_table(&corpus, DEFAULT_TOP_K, 64);
        let row = rows
            .iter()
            .find(|(k, _)| *k == 100)
            .expect("content key 100 is fitted");
        // Candidate 9 is present and carries a positive weight.
        let w9 = row
            .1
            .iter()
            .find(|(c, _)| *c == 9)
            .map(|(_, w)| *w)
            .expect("candidate 9 present for key 100");
        assert!(w9 > 0, "the learned association carries positive weight");
        // 9 is the top-weighted candidate for key 100.
        let top = row
            .1
            .iter()
            .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
            .map(|(c, _)| *c)
            .expect("non-empty row");
        assert_eq!(top, 9, "candidate 9 dominates key 100's fitted row");
    }

    #[test]
    fn fit_respects_the_key_and_candidate_bounds() {
        // Two content tokens with different evidence mass; max_keys=1 keeps only
        // the higher-evidence key.
        let stories = vec![
            (vec![1u32, 2, 3], vec![7u32, 7, 0]),
            (vec![1u32, 2, 3], vec![7u32, 7, 0]),
            (vec![1u32, 2, 3], vec![7u32, 7, 0]),
            (vec![4u32, 5], vec![8u32, 0]),
            (vec![9u32, 9], vec![1u32, 0]), // held-out
        ];
        let corpus = corpus_from_stories(&stories);

        let one = fit_segment_table(&corpus, DEFAULT_TOP_K, 1);
        assert_eq!(one.len(), 1, "max_keys=1 retains exactly one key");

        // top_k=1 keeps a single candidate per key.
        let capped = fit_segment_table(&corpus, 1, 64);
        for (_, entries) in &capped {
            assert_eq!(entries.len(), 1, "top_k=1 keeps one candidate per key");
        }

        // Degenerate bounds yield an empty table (nothing to fit).
        assert!(fit_segment_table(&corpus, 0, 64).is_empty());
        assert!(fit_segment_table(&corpus, 64, 0).is_empty());
    }
}
