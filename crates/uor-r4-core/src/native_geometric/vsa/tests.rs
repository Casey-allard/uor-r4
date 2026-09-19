//! Comprehensive Unit Tests for High-Dimensional VSA Engine.
//!
//! Validates:
//! 1. Orthogonality of random hypervectors (mean distance ~50% with tight variance)
//! 2. Invertibility and algebraic properties of XOR binding (A ^ B ^ B = A)
//! 3. Position distinctness and automorphism properties of permutation rho^k
//! 4. Bundling preservation (majority vector preserves all constituent identities)
//! 5. Non-collision and word-order sensitivity of n-gram sequences on natural text
//! 6. Exact integer Q1.15 metrics for serving hot paths (zero floats)

use super::codebook::Codebook;
use super::context_engine::{
    encode_multiscale_context, encode_ngram, encode_positional_context, RollingVsaContext,
};
use super::hypervector::{Hypervector, Hypervector2048, Hypervector4096};

#[test]
fn test_orthogonality_of_random_hypervectors() {
    let seed = 0x2026_0917_u64;

    // Verify type aliases exist and can be instantiated
    let v2048: Hypervector2048 = Hypervector2048::zero();
    assert_eq!(v2048.count_ones(), 0);
    assert_eq!(Hypervector2048::dimension_bits(), 2048);
    assert_eq!(Hypervector4096::dimension_bits(), 4096);

    // Test both 2048-bit and 4096-bit dimensions
    fn check_orthogonality<const WORDS: usize>(seed: u64, dim_bits: usize) {
        let count = 100usize;
        let mut vectors = Vec::with_capacity(count);
        for i in 0..count {
            vectors.push(Hypervector::<WORDS>::from_seed(seed, i as u64));
        }

        // Verify individual vector bit density is close to 50%
        for (i, v) in vectors.iter().enumerate() {
            let ones = v.count_ones();
            let ratio = ones as f64 / dim_bits as f64;
            assert!(
                (ratio - 0.50).abs() < 0.05,
                "Vector {} bit ratio {} deviated too far from 0.50",
                i,
                ratio
            );
        }

        // Pairwise Hamming distances between all distinct pairs (i < j)
        let mut sum_dist = 0.0;
        let mut pair_count = 0usize;
        let mut min_dist = 1.0f64;
        let mut max_dist = 0.0f64;

        for i in 0..count {
            for j in (i + 1)..count {
                let d = vectors[i].hamming_distance_normalized(&vectors[j]);
                sum_dist += d;
                pair_count += 1;
                if d < min_dist {
                    min_dist = d;
                }
                if d > max_dist {
                    max_dist = d;
                }
            }
        }

        let mean_dist = sum_dist / pair_count as f64;
        // Mean distance should be within [0.495, 0.505]
        assert!(
            (mean_dist - 0.50).abs() < 0.01,
            "Mean distance {} deviated from 0.50 for dim {}",
            mean_dist,
            dim_bits
        );

        // For high dimensions (e.g. 4096), std dev is ~0.0078, so range is tightly bounded
        let bound = if dim_bits >= 4096 { 0.05 } else { 0.06 };
        assert!(
            (min_dist - 0.50).abs() < bound,
            "Min distance {} out of expected bound",
            min_dist
        );
        assert!(
            (max_dist - 0.50).abs() < bound,
            "Max distance {} out of expected bound",
            max_dist
        );
    }

    check_orthogonality::<32>(seed, 2048);
    check_orthogonality::<64>(seed, 4096);
}

#[test]
fn test_invertibility_and_algebra_of_binding() {
    let a = Hypervector4096::from_seed(12345, 1);
    let b = Hypervector4096::from_seed(12345, 2);
    let c = Hypervector4096::from_seed(12345, 3);
    let zero = Hypervector4096::zero();

    // 1. Invertibility: A ^ B ^ B = A
    let bound_ab = a.bind(&b);
    let recovered_a = bound_ab.unbind(&b);
    assert_eq!(recovered_a, a, "A ^ B ^ B must equal A");

    let recovered_b = bound_ab.unbind(&a);
    assert_eq!(recovered_b, b, "A ^ B ^ A must equal B");

    // 2. Commutativity: A ^ B = B ^ A
    assert_eq!(a.bind(&b), b.bind(&a), "Binding must be commutative");

    // 3. Associativity: (A ^ B) ^ C = A ^ (B ^ C)
    let left = (a.bind(&b)).bind(&c);
    let right = a.bind(&(b.bind(&c)));
    assert_eq!(left, right, "Binding must be associative");

    // 4. Self-Inverse: A ^ A = 0
    assert_eq!(a.bind(&a), zero, "Self-binding must yield zero");

    // 5. Identity: A ^ 0 = A
    assert_eq!(a.bind(&zero), a, "Binding with zero must yield identity");

    // 6. Quasi-orthogonality of bound vector to both operands
    let dist_a = bound_ab.hamming_distance_normalized(&a);
    let dist_b = bound_ab.hamming_distance_normalized(&b);
    assert!(
        (dist_a - 0.50).abs() < 0.04,
        "Bound vector must be quasi-orthogonal to A"
    );
    assert!(
        (dist_b - 0.50).abs() < 0.04,
        "Bound vector must be quasi-orthogonal to B"
    );

    // 7. Distance preservation: d(A ^ C, B ^ C) = d(A, B)
    let d_ab = a.hamming_distance(&b);
    let d_ac_bc = a.bind(&c).hamming_distance(&b.bind(&c));
    assert_eq!(
        d_ab, d_ac_bc,
        "Binding must preserve exact Hamming distance"
    );
}

#[test]
fn test_position_distinctness_and_permutation_properties() {
    let a = Hypervector4096::from_seed(98765, 42);
    let dim = Hypervector4096::dimension_bits();

    // 1. Identity at shift 0 and shift D
    assert_eq!(a.permute(0), a, "Shift 0 must equal self");
    assert_eq!(a.permute(dim), a, "Shift D must equal self");
    assert_eq!(a.permute(2 * dim), a, "Shift 2D must equal self");

    // 2. Invertibility: rho^-k(rho^k(A)) = A
    for k in [1, 2, 7, 31, 63, 64, 65, 127, 128, 512, 1024, 2048, 4095] {
        let shifted = a.permute(k);
        let unshifted = shifted.permute_inv(k);
        assert_eq!(unshifted, a, "permute_inv({}) failed to invert", k);

        let unshifted_right = shifted.permute_right(k);
        assert_eq!(unshifted_right, a, "permute_right({}) failed to invert", k);
    }

    // 3. Additive group homomorphism: rho^k1(rho^k2(A)) = rho^(k1 + k2)(A)
    let s1 = a.permute(37).permute(89);
    let s2 = a.permute(37 + 89);
    assert_eq!(s1, s2, "Permutation composition must be additive");

    // 4. Conservation of Hamming weight: popcount(rho^k(A)) = popcount(A)
    let original_ones = a.count_ones();
    for k in [1, 3, 64, 100, 500, 2048] {
        assert_eq!(
            a.permute(k).count_ones(),
            original_ones,
            "Hamming weight must be preserved under permutation k={}",
            k
        );
    }

    // 5. Position distinctness: rho^k(A) is quasi-orthogonal to A for all k >= 1
    for k in [
        1, 2, 3, 5, 8, 13, 21, 63, 64, 65, 100, 512, 1024, 2048, 4095,
    ] {
        let shifted = a.permute(k);
        let dist = shifted.hamming_distance_normalized(&a);
        assert!(
            (dist - 0.50).abs() < 0.04,
            "rho^{}(A) was not quasi-orthogonal to A (dist = {})",
            k,
            dist
        );
    }
}

#[test]
fn test_bundling_preservation_across_window() {
    let seed = 2026_0917_u64;

    // Test various window sizes M (both odd and even, including M=2)
    for &m in &[2, 3, 4, 5, 6, 7, 8, 11, 15, 31] {
        let mut vectors = Vec::with_capacity(m);
        for i in 0..m {
            vectors.push(Hypervector4096::from_seed(seed, (i + 1) as u64));
        }

        let bundle = Hypervector4096::bundle(&vectors);

        // Every constituent vector should have high similarity (> 0.54) to the bundle
        for (i, v) in vectors.iter().enumerate() {
            let sim = bundle.hamming_similarity_normalized(v);
            assert!(
                sim > 0.54,
                "Bundle of size {} failed to preserve constituent {} (sim = {})",
                m,
                i,
                sim
            );
        }

        // For M=2, both constituents should have symmetric similarity ~0.75
        if m == 2 {
            let sim0 = bundle.hamming_similarity_normalized(&vectors[0]);
            let sim1 = bundle.hamming_similarity_normalized(&vectors[1]);
            assert!(
                (sim0 - 0.75).abs() < 0.04,
                "M=2 constituent 0 similarity {} deviated from ~0.75",
                sim0
            );
            assert!(
                (sim1 - 0.75).abs() < 0.04,
                "M=2 constituent 1 similarity {} deviated from ~0.75",
                sim1
            );
            assert!(
                (sim0 - sim1).abs() < 0.02,
                "M=2 constituent similarities are asymmetric: {} vs {}",
                sim0,
                sim1
            );
        }

        // An independent random vector must have similarity ~ 0.50
        let unrelated = Hypervector4096::from_seed(seed, 999_999);
        let sim_unrelated = bundle.hamming_similarity_normalized(&unrelated);
        assert!(
            (sim_unrelated - 0.50).abs() < 0.04,
            "Unrelated vector should have similarity ~0.50 to bundle of size {} (was {})",
            m,
            sim_unrelated
        );
    }
}

#[test]
fn test_positional_context_encoding() {
    let codebook = Codebook::<64>::new(100, 0xfeed_beef);
    let tokens = [5u32, 12, 27, 44, 89];
    let pos_ctx = encode_positional_context(&tokens, &codebook);

    // Positional context must preserve similarity to constituent shifted tokens
    for (i, &tok) in tokens.iter().enumerate() {
        let lag = tokens.len() - 1 - i;
        let expected_shifted = codebook.get(tok).permute(lag);
        let sim = pos_ctx.hamming_similarity_normalized(&expected_shifted);
        assert!(
            sim > 0.54,
            "Positional context did not preserve token {} at lag {} (sim = {})",
            tok,
            lag,
            sim
        );
    }
}

#[test]
fn test_non_collision_of_distinct_ngram_sequences() {
    let codebook = Codebook::<64>::new(100, 0xcafe_babe);

    // Natural text token sequences
    let seq_cat_sat_mat = [10u32, 25, 42, 60]; // "the cat sat mat"
    let seq_dog_sat_mat = [10u32, 26, 42, 60]; // "the dog sat mat" (one word changed)
    let seq_mat_sat_cat = [60u32, 42, 25, 10]; // reversed word order
    let seq_cat_mat_sat = [10u32, 25, 60, 42]; // swapped order of last two words
    let seq_story_open = [1u32, 2, 3, 4]; // "once upon a time"
    let seq_story_perm = [4u32, 3, 2, 1]; // "time a upon once"

    let h1 = encode_ngram(&seq_cat_sat_mat, &codebook);
    let h2 = encode_ngram(&seq_dog_sat_mat, &codebook);
    let h3 = encode_ngram(&seq_mat_sat_cat, &codebook);
    let h4 = encode_ngram(&seq_cat_mat_sat, &codebook);
    let h_story1 = encode_ngram(&seq_story_open, &codebook);
    let h_story2 = encode_ngram(&seq_story_perm, &codebook);

    // 1. Identical sequence must produce exact identical hypervector (zero distance)
    let h1_again = encode_ngram(&seq_cat_sat_mat, &codebook);
    assert_eq!(
        h1, h1_again,
        "Identical sequence must produce identical vector"
    );

    // 2. Single token substitution must produce quasi-orthogonal vector (no false collision)
    let dist_1_2 = h1.hamming_distance_normalized(&h2);
    assert!(
        (dist_1_2 - 0.50).abs() < 0.04,
        "Single word substitution must be quasi-orthogonal (dist = {})",
        dist_1_2
    );

    // 3. Reversed word order must produce quasi-orthogonal vector
    let dist_1_3 = h1.hamming_distance_normalized(&h3);
    assert!(
        (dist_1_3 - 0.50).abs() < 0.04,
        "Reversed word order must be quasi-orthogonal (dist = {})",
        dist_1_3
    );

    // 4. Neighboring token swap must produce quasi-orthogonal vector
    let dist_1_4 = h1.hamming_distance_normalized(&h4);
    assert!(
        (dist_1_4 - 0.50).abs() < 0.04,
        "Neighboring swap must be quasi-orthogonal (dist = {})",
        dist_1_4
    );

    // 5. Permuted natural story opener
    let dist_story = h_story1.hamming_distance_normalized(&h_story2);
    assert!(
        (dist_story - 0.50).abs() < 0.04,
        "Permuted story opener must be quasi-orthogonal (dist = {})",
        dist_story
    );
}

#[test]
fn test_serving_hot_path_invariants_and_integer_metrics() {
    let a = Hypervector4096::from_seed(42, 1);
    let b = Hypervector4096::from_seed(42, 2);

    // 1. Exact match produces maximum Q1.15 score (32767)
    assert_eq!(a.similarity_q15(&a), 32767);
    assert_eq!(a.bipolar_correlation_q15(&a), 32767);

    // 2. Complete negation produces minimum score
    let mut not_a = a;
    for w in &mut not_a.data {
        *w = !*w;
    }
    assert_eq!(a.similarity_q15(&not_a), 0);
    assert_eq!(a.bipolar_correlation_q15(&not_a), -32767);

    // 3. Quasi-orthogonal vectors have score near mid-scale (16383 overlap, ~0 correlation)
    let score = a.similarity_q15(&b);
    assert!(
        (score - 16384).abs() < 1500,
        "Orthogonal overlap Q1.15 should be near 16384, was {}",
        score
    );

    let corr = a.bipolar_correlation_q15(&b);
    assert!(
        corr.abs() < 2500,
        "Orthogonal correlation Q1.15 should be near 0, was {}",
        corr
    );
}

#[test]
fn test_rolling_vsa_context_engine() {
    let codebook = Codebook::<64>::new(100, 0x1337);
    let mut ctx = RollingVsaContext::<64, 32>::new();

    assert_eq!(ctx.len(), 0);
    assert!(ctx.is_empty());
    assert_eq!(ctx.recent(1), None);

    // Push sequence: 10, 20, 30, 40
    ctx.push(10);
    ctx.push(20);
    ctx.push(30);
    ctx.push(40);

    assert_eq!(ctx.len(), 4);
    assert_eq!(ctx.recent(1), Some(40));
    assert_eq!(ctx.recent(2), Some(30));
    assert_eq!(ctx.recent(3), Some(20));
    assert_eq!(ctx.recent(4), Some(10));
    assert_eq!(ctx.recent(5), None);

    // Current 4-gram matches pure function
    let ngram_ctx = ctx.current_ngram(&codebook, 4);
    let ngram_pure = encode_ngram(&[10, 20, 30, 40], &codebook);
    assert_eq!(ngram_ctx, ngram_pure);

    // Bundled context vector matches pure function
    let bundle_ctx = ctx.current_context_hypervector(&codebook, 4);
    let bundle_pure = encode_multiscale_context(&[10, 20, 30, 40], &codebook, 4);
    assert_eq!(bundle_ctx, bundle_pure);

    // Test ring buffer wraparound: push 35 more items
    for i in 0..35 {
        ctx.push(100 + i);
    }
    assert_eq!(ctx.len(), 32); // capped at MAX_WINDOW
    assert_eq!(ctx.recent(1), Some(134)); // last pushed
    assert_eq!(ctx.recent(32), Some(103)); // oldest retained
    assert_eq!(ctx.recent(33), None); // evicted
}

#[test]
fn test_bundling_strict_commutativity() {
    let a = Hypervector4096::from_seed(42, 10);
    let b = Hypervector4096::from_seed(42, 20);
    let c = Hypervector4096::from_seed(42, 30);
    let d = Hypervector4096::from_seed(42, 40);

    // M = 2 commutativity: bundle([A, B]) == bundle([B, A])
    let b_ab = Hypervector4096::bundle(&[a, b]);
    let b_ba = Hypervector4096::bundle(&[b, a]);
    assert_eq!(
        b_ab, b_ba,
        "M=2 bundling must be strictly commutative (A + B = B + A)"
    );

    // M = 3 commutativity: bundle([A, B, C]) == bundle([C, A, B])
    let b_abc = Hypervector4096::bundle(&[a, b, c]);
    let b_cab = Hypervector4096::bundle(&[c, a, b]);
    assert_eq!(b_abc, b_cab, "M=3 bundling must be strictly commutative");

    // M = 4 commutativity: bundle([A, B, C, D]) == bundle([D, B, A, C])
    let b_abcd = Hypervector4096::bundle(&[a, b, c, d]);
    let b_dbac = Hypervector4096::bundle(&[d, b, a, c]);
    assert_eq!(
        b_abcd, b_dbac,
        "M=4 bundling must be strictly commutative under any permutation"
    );
}

#[test]
fn test_bundling_bit_slice_matches_naive_counting() {
    // Ground truth: Naive bit-by-bit majority counter
    fn naive_bundle<const WORDS: usize>(
        vectors: &[Hypervector<WORDS>],
        tie_breaker: &Hypervector<WORDS>,
    ) -> Hypervector<WORDS> {
        let m = vectors.len();
        if m == 0 {
            return Hypervector::zero();
        }
        let threshold = m / 2;
        let is_even = (m % 2) == 0;
        let mut result = [0u64; WORDS];

        for w in 0..WORDS {
            let mut word_val = 0u64;
            for bit in 0..64 {
                let mut count = 0;
                for v in vectors {
                    if ((v.data[w] >> bit) & 1) == 1 {
                        count += 1;
                    }
                }
                let bit_val = if count > threshold {
                    true
                } else if count == threshold && is_even {
                    ((tie_breaker.data[w] >> bit) & 1) == 1
                } else {
                    false
                };
                if bit_val {
                    word_val |= 1u64 << bit;
                }
            }
            result[w] = word_val;
        }
        Hypervector::from_words(result)
    }

    // Verify bit-slice ripple adder exactly matches naive bit-counting for M in 1..25
    let seed = 0x9876_5432_10fe_dcba;
    let tie = Hypervector4096::from_seed(seed, 0);

    for m in 1..=25 {
        let mut vecs = Vec::with_capacity(m);
        for i in 1..=m {
            vecs.push(Hypervector4096::from_seed(seed, i as u64));
        }

        let fast = Hypervector4096::bundle_with_tie_breaker(&vecs, &tie);
        let naive = naive_bundle(&vecs, &tie);

        assert_eq!(
            fast, naive,
            "Bit-slice bundle failed to match naive count for M={}",
            m
        );
    }
}

#[test]
fn test_zero_allocation_top_k_candidates_q15() {
    let codebook = Codebook::<32>::new(500, 0xa1b2_c3d4);
    let query = Hypervector::<32>::from_seed(0xcafe, 0xbabe);

    // Get top-8 via Vec-based top_k_tokens_q15
    let vec_top8 = codebook.top_k_tokens_q15(&query, 8);

    // Get top-8 via zero-allocation array-based top_k_candidates_q15
    let arr_top8 = codebook.top_k_candidates_q15::<8>(&query);

    assert_eq!(vec_top8.len(), 8);
    for i in 0..8 {
        assert_eq!(
            vec_top8[i], arr_top8[i],
            "Rank {} mismatch: vec {:?} vs arr {:?}",
            i, vec_top8[i], arr_top8[i]
        );
    }
}

#[test]
fn test_large_ngram_in_rolling_context() {
    let codebook = Codebook::<64>::new(100, 0x55aa);
    let mut ctx = RollingVsaContext::<64, 64>::new();

    for tok in 0..32 {
        ctx.push(tok);
    }

    assert_eq!(ctx.len(), 32);

    // Current 32-gram should encode all 32 tokens without truncation
    let ngram32 = ctx.current_ngram(&codebook, 32);
    let all_tokens: Vec<u32> = (0..32).collect();
    let ngram_expected = encode_ngram(&all_tokens, &codebook);
    assert_eq!(ngram32, ngram_expected);
}

#[test]
fn test_zero_dimension_safety() {
    let z0 = Hypervector::<0>::zero();
    assert_eq!(z0.count_ones(), 0);
    assert_eq!(Hypervector::<0>::dimension_bits(), 0);
    assert_eq!(z0.permute(5), z0);
    assert_eq!(z0.permute_inv(5), z0);
    assert_eq!(z0.hamming_distance(&z0), 0);
    assert_eq!(z0.hamming_distance_normalized(&z0), 0.0);
    assert_eq!(z0.bipolar_cosine(&z0), 0.0);
    assert_eq!(z0.similarity_q15(&z0), 0);
    assert_eq!(z0.bipolar_correlation_q15(&z0), 0);
}

#[test]
fn test_vsa_lag0_and_lag1_repetition_trap_elimination() {
    let codebook = Codebook::<64>::new(100, 0x1234_5678);
    let mut ctx = RollingVsaContext::<64, 64>::new();

    // Push sequence [10, 20, 30, 40], where current token w_t = 40
    ctx.push(10);
    ctx.push(20);
    ctx.push(30);
    ctx.push(40);

    let h_topic = ctx.current_context_hypervector(&codebook, 4);

    // Candidate v = w_t = 40 (immediate self-repetition) evaluated at lag 0: unpermuted E(v)
    let cand_repeat = codebook.get(40);
    let repeat_corr = h_topic.bipolar_correlation_q15(&cand_repeat);

    // Bipolar correlation must be near 0.0 (|corr| < 3000 out of 32767, i.e. < 0.10)
    // rather than a self-repetition attractor.
    assert!(
        repeat_corr.abs() < 3000,
        "Expected near-zero correlation for immediate repetition candidate w_t, got {}",
        repeat_corr
    );

    // Candidate v = w_{t-1} = 30 (alternating repetition) evaluated at lag 0: unpermuted E(v)
    let cand_lag1 = codebook.get(30);
    let lag1_corr = h_topic.bipolar_correlation_q15(&cand_lag1);

    // Both lag 0 and lag 1 must have near-zero correlation
    assert!(
        lag1_corr.abs() < 3000,
        "Expected near-zero correlation for lag-1 repetition candidate w_{{t-1}}, got {}",
        lag1_corr
    );

    // Also verify score_continuation_q15 uses unpermuted E(v)
    let score = ctx.score_continuation_q15(&codebook, &h_topic, 40);
    // Overlap should be near 50% (16384 +/- 2000)
    assert!(
        (score - 16384).abs() < 2000,
        "Expected near 50% overlap (16384) for immediate repetition, got {}",
        score
    );

    let score30 = ctx.score_continuation_q15(&codebook, &h_topic, 30);
    assert!(
        (score30 - 16384).abs() < 2000,
        "Expected near 50% overlap (16384) for lag-1 repetition, got {}",
        score30
    );
}

#[test]
fn test_vsa_single_token_and_boundary_cases() {
    let codebook = Codebook::<64>::new(100, 0x1234_5678);
    let mut ctx = RollingVsaContext::<64, 64>::new();

    // 1. Empty context
    let h_empty = ctx.current_context_hypervector(&codebook, 4);
    assert_eq!(h_empty, Hypervector4096::zero());

    // 2. Length 1 context: [42]
    ctx.push(42);
    assert_eq!(ctx.len(), 1);
    let h_len1 = ctx.current_context_hypervector(&codebook, 4);
    // Non-zero because w_t=42 is rotated by rho^1
    assert!(h_len1.count_ones() > 0);

    // Immediate repetition candidate v = 42 evaluated at lag 0 (unpermuted)
    let cand42 = codebook.get(42);
    let corr42 = h_len1.bipolar_correlation_q15(&cand42);
    assert!(
        corr42.abs() < 3000,
        "Length 1 context must not attract immediate repetition of itself, got {}",
        corr42
    );

    // 3. Length 2 context: [42, 99]
    ctx.push(99);
    assert_eq!(ctx.len(), 2);
    let h_len2 = ctx.current_context_hypervector(&codebook, 4);
    let cand99 = codebook.get(99);
    let corr99 = h_len2.bipolar_correlation_q15(&cand99);
    let corr42_lag1 = h_len2.bipolar_correlation_q15(&cand42);
    assert!(
        corr99.abs() < 3000,
        "Length 2 context must have near-zero correlation with w_t=99, got {}",
        corr99
    );
    assert!(
        corr42_lag1.abs() < 3000,
        "Length 2 context must have near-zero correlation with w_{{t-1}}=42, got {}",
        corr42_lag1
    );
}

#[test]
fn test_vsa_higher_order_periodic_attractor_elimination() {
    let codebook = Codebook::<64>::new(100, 0xcafe_babe);

    // Test period-2: [10, 20, 10, 20, 10, 20]
    // Transitions present: 10->20, 20->10. Current token is 20.
    // Continuation of 20 must associate to 10 (high correlation), with zero self-attraction to 20.
    let seq_p2 = [10u32, 20, 10, 20, 10, 20];
    let h_p2 = encode_multiscale_context(&seq_p2, &codebook, 16);
    let corr_10 = h_p2.bipolar_correlation_q15(&codebook.get(10));
    let corr_20 = h_p2.bipolar_correlation_q15(&codebook.get(20));
    assert!(
        corr_10 > 3000,
        "Period-2 context must associate transition 20 -> 10: got {}",
        corr_10
    );
    assert!(
        corr_20.abs() < 3000,
        "Period-2 context must not self-attract token 20: got {}",
        corr_20
    );

    // Test period-3: [10, 20, 30, 10, 20, 30] ("The cat and the cat and")
    // Current token is 30. Transition 30->10 must be recalled, without self-attracting 20 or 30.
    let seq_p3 = [10u32, 20, 30, 10, 20, 30];
    let h_p3 = encode_multiscale_context(&seq_p3, &codebook, 16);
    let corr_p3_10 = h_p3.bipolar_correlation_q15(&codebook.get(10));
    let corr_p3_20 = h_p3.bipolar_correlation_q15(&codebook.get(20));
    let corr_p3_30 = h_p3.bipolar_correlation_q15(&codebook.get(30));
    assert!(
        corr_p3_10 > 3000,
        "Period-3 context must associate transition 30 -> 10: got {}",
        corr_p3_10
    );
    assert!(
        corr_p3_20.abs() < 3000,
        "Period-3 context must not falsely attract token 20: got {}",
        corr_p3_20
    );
    assert!(
        corr_p3_30.abs() < 3000,
        "Period-3 context must not self-attract token 30: got {}",
        corr_p3_30
    );

    // Test full 16-token window with unique tokens: verify every token in the window has near-zero correlation
    // because token 16 never appeared earlier as a transition source.
    let seq_16: Vec<u32> = (1..=16).collect();
    let h_16 = encode_multiscale_context(&seq_16, &codebook, 16);
    for &tok in &seq_16 {
        let corr = h_16.bipolar_correlation_q15(&codebook.get(tok));
        assert!(
            corr.abs() < 3000,
            "16-window constituent token {} must have near-zero correlation: got {}",
            tok,
            corr
        );
    }
}

#[test]
fn test_hrr_associative_unbinding_continuation_prediction() {
    let codebook = Codebook::<64>::new(100, 0x1234_5678);

    // Sequence: [1 ("the"), 2 ("cat"), 3 ("sat"), 4 ("on"), 1 ("the")]
    // Current token is 1 ("the").
    // Expected continuation is 2 ("cat"), because 1 previously transitioned to 2.
    let seq = [1u32, 2, 3, 4, 1];
    let e_next_hat = super::encode_associative_transition_context(&seq, &codebook);

    let corr_cat = e_next_hat.bipolar_correlation_q15(&codebook.get(2));
    assert!(
        corr_cat > 8000,
        "HRR associative unbinding must recall continuation token 2 ('cat'): got {}",
        corr_cat
    );

    // Candidate 1 (immediate self-repetition) must have near-zero correlation
    let corr_the = e_next_hat.bipolar_correlation_q15(&codebook.get(1));
    assert!(
        corr_the.abs() < 3000,
        "HRR associative unbinding must not self-attract token 1 ('the'): got {}",
        corr_the
    );

    // Candidates 3 ("sat") and 4 ("on") must have near-zero correlation
    let corr_sat = e_next_hat.bipolar_correlation_q15(&codebook.get(3));
    let corr_on = e_next_hat.bipolar_correlation_q15(&codebook.get(4));
    assert!(
        corr_sat.abs() < 3000,
        "Irrelevant past token 3 must not be attracted: got {}",
        corr_sat
    );
    assert!(
        corr_on.abs() < 3000,
        "Irrelevant past token 4 must not be attracted: got {}",
        corr_on
    );

    // Unrelated candidate 99
    let corr_99 = e_next_hat.bipolar_correlation_q15(&codebook.get(99));
    assert!(
        corr_99.abs() < 3000,
        "Unrelated candidate 99 must have near-zero correlation: got {}",
        corr_99
    );
}
