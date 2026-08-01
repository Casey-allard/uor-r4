//! Bit-identity witness: the streaming top-N selection inside
//! `softmax_top3_sample` / `softmax_top8_sample` must produce token and
//! weight arrays identical to the stable-sort selection it replaced
//! (descending probability, ties to the lowest token index). The corpus
//! era depends on these bytes.

use uor_r4_core::transformerless::compiler::{softmax_top3_sample, softmax_top8_sample};

/// Reference: the previous implementation's selection — a stable
/// descending full sort of (index, probability) after the same softmax.
fn reference_top_n(logits: &[f32], n: usize) -> (Vec<u32>, Vec<u32>) {
    let mut probs = logits.to_vec();
    let mx = probs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    for p in probs.iter_mut() {
        *p = (*p - mx).exp();
        sum += *p;
    }
    for p in probs.iter_mut() {
        *p /= sum;
    }
    let mut cand: Vec<(usize, f32)> = probs.iter().copied().enumerate().collect();
    cand.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let sum_top: f32 = cand.iter().take(n).map(|c| c.1).sum();
    let mut tokens = vec![0u32; n];
    let mut weights = vec![0u32; n];
    if sum_top > 1e-9 {
        let mut accumulated = 0;
        for i in 0..n {
            tokens[i] = cand[i].0 as u32;
            let w = ((cand[i].1 / sum_top) * 100.0).round() as u32;
            weights[i] = w;
            accumulated += w;
        }
        if accumulated != 100 && weights[0] > 0 {
            let diff = 100i32 - accumulated as i32;
            weights[0] = (weights[0] as i32 + diff).max(0) as u32;
        }
    }
    (tokens, weights)
}

#[test]
fn streaming_top_n_matches_stable_sort_selection() {
    // Deterministic pseudo-random logits (xorshift64), with exact ties
    // injected so tie-breaking is exercised on every case.
    let mut seed = 0x1234_5678_9abc_def0u64;
    let mut next_f32 = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        ((seed >> 40) as f32 / (1u64 << 24) as f32) * 20.0 - 10.0
    };
    for case in 0..64usize {
        let len = 97 + case;
        let mut logits: Vec<f32> = (0..len).map(|_| next_f32()).collect();
        logits[3] = logits[7];
        logits[0] = logits[len - 1];
        logits[10] = logits[11]; // tie just outside the top-8 boundary region

        let mut a = logits.clone();
        let mut b = logits.clone();
        let mut rng = 0x5EEDu64;
        let (_, t3, w3) = softmax_top3_sample(&mut a, &mut rng);
        let mut rng = 0x5EEDu64;
        let (_, t8, w8) = softmax_top8_sample(&mut b, &mut rng);

        let (rt3, rw3) = reference_top_n(&logits, 3);
        let (rt8, rw8) = reference_top_n(&logits, 8);
        assert_eq!(t3.as_slice(), rt3.as_slice(), "top3 tokens, case {case}");
        assert_eq!(w3.as_slice(), rw3.as_slice(), "top3 weights, case {case}");
        assert_eq!(t8.as_slice(), rt8.as_slice(), "top8 tokens, case {case}");
        assert_eq!(w8.as_slice(), rw8.as_slice(), "top8 weights, case {case}");
    }
}
