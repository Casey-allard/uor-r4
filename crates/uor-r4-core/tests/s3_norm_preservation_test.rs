//! Verification of integer Newton-Raphson S3 norm preservation over 64-step trajectories
//! and 4-gram repetition entropy on free-running generation rollouts with 64-token VSA context.

use uor_r4_core::native_geometric::hopf_metric::{UnitS3Q30, Q30_SCALE};
use uor_r4_core::native_geometric::learner::embedding::{canonical_h4_roots_q30, H4_ROOT_COUNT};
use uor_r4_core::native_geometric::learner::{JepaTrainer, JepaTrainerConfig};

/// Verify that over 64 consecutive `mul_q30` steps with H4 roots, normalizing every 8 steps
/// keeps `norm_q30()` strictly within `[2^30 - 1, 2^30]`, eliminating quadratic norm decay.
#[test]
fn test_s3_norm_preservation_over_64_steps() {
    let roots = canonical_h4_roots_q30();
    let q30_scale = Q30_SCALE as u32;

    // Run across multiple root stride paths
    for stride in [1, 7, 13, 17, 31, 59] {
        let mut s3 = UnitS3Q30::IDENTITY;
        let mut unnorm_s3 = UnitS3Q30::IDENTITY;

        for step in 0..64 {
            let root = roots[(step * stride + 3) % H4_ROOT_COUNT];
            s3 = s3.mul_q30(&root);
            unnorm_s3 = unnorm_s3.mul_q30(&root);

            if (step + 1) % 8 == 0 {
                s3 = s3.normalized();
                let norm = s3.norm_q30();
                assert!(
                    norm <= q30_scale && norm >= q30_scale - 1,
                    "Step {}: normalized s3 norm must be in [2^30 - 1, 2^30], got {}",
                    step + 1,
                    norm
                );
            }
        }

        let final_norm = s3.norm_q30();
        assert!(
            final_norm <= q30_scale && final_norm >= q30_scale - 1,
            "Final normalized s3 norm must be in [2^30 - 1, 2^30], got {}",
            final_norm
        );

        let unnorm_norm = unnorm_s3.norm_q30();
        assert!(
            unnorm_norm < q30_scale - 1,
            "Unnormalized s3 must exhibit quadratic norm decay below 2^30 - 1, got {}",
            unnorm_norm
        );
    }
}

/// Verify trajectory normalization for any sequence length in 1..=64,
/// ensuring norm is strictly in [2^30 - 1, 2^30] after trajectory execution.
#[test]
fn test_trajectory_normalization_arbitrary_lengths() {
    let roots = canonical_h4_roots_q30();
    let q30_scale = Q30_SCALE as u32;

    for len in 1..=64 {
        let mut s3 = UnitS3Q30::IDENTITY;
        let mut step = 0;
        for i in 0..len {
            let root = roots[(i * 13 + 5) % H4_ROOT_COUNT];
            s3 = s3.mul_q30(&root);
            step += 1;
            if step % 8 == 0 {
                s3 = s3.normalized();
            }
        }
        if step % 8 != 0 {
            s3 = s3.normalized();
        }
        let norm = s3.norm_q30();
        assert!(
            norm <= q30_scale && norm >= q30_scale - 1,
            "Length {}: final norm must be in [2^30 - 1, 2^30], got {}",
            len,
            norm
        );
    }
}

/// Verify that 4-gram repetition entropy on generated rollout remains >= 0.94.
#[test]
fn test_rollout_4gram_repetition_entropy() {
    let config = JepaTrainerConfig {
        vocab_size: 257,
        num_lanes: 4,
        context_window: 64,
        learning_rate: 0.05,
        jepa_weight: 0.2,
        weight_decay: 1e-4,
        grad_clip: 1.0,
        ..JepaTrainerConfig::default()
    };
    let mut trainer = JepaTrainer::new(config, 2026_0917);
    let sample1 = b"The swift brown fox jumps over the lazy sleeping dog in the forest.";
    let sample2 = b"The quick brown rabbit leaps across the green meadow under the blue sky.";
    let sample3 = b"A gentle breeze rustles through golden autumn leaves across the quiet valley.";
    let sample4 = b"Deep within ancient stone mountains rivers carve pathways through the rock.";
    let tokens1: Vec<usize> = sample1.iter().map(|&b| b as usize).collect();
    let tokens2: Vec<usize> = sample2.iter().map(|&b| b as usize).collect();
    let tokens3: Vec<usize> = sample3.iter().map(|&b| b as usize).collect();
    let tokens4: Vec<usize> = sample4.iter().map(|&b| b as usize).collect();

    for _ in 0..10 {
        trainer.train_sequence(&tokens1);
        trainer.train_sequence(&tokens2);
        trainer.train_sequence(&tokens3);
        trainer.train_sequence(&tokens4);
    }

    let exported = trainer.export_discrete();

    // Generation rollout: free-running generation with temperature / top-k sampling
    let prompt = b"The swift";
    let mut gen: Vec<usize> = prompt.iter().map(|&b| b as usize).collect();
    let mut rng = 20260917_u64;

    for _ in 0..128 {
        let fiber = exported.context_hopf_fiber_q30(&gen);
        let mut scores = Vec::with_capacity(exported.vocab_size);
        for v in 32..127 {
            let s = exported.score_context_candidate(&gen, v, fiber);
            scores.push((v, s));
        }
        scores.sort_by_key(|&(_, s)| std::cmp::Reverse(s));

        // Top-5 sampling with pseudo-random choice
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let pick_idx = (rng % 5).min(scores.len() as u64 - 1) as usize;
        gen.push(scores[pick_idx].0);
    }

    // Compute 4-gram repetition entropy over generated sequence
    let total_4grams = gen.len() - 3;
    let mut counts = std::collections::HashMap::new();
    for w in gen.windows(4) {
        *counts.entry((w[0], w[1], w[2], w[3])).or_insert(0usize) += 1;
    }
    let mut entropy = 0.0;
    let total_f = total_4grams as f64;
    for &c in counts.values() {
        let p = (c as f64) / total_f;
        if p > 0.0 {
            entropy -= p * libm::log2(p);
        }
    }
    let max_entropy = libm::log2(total_f).max(1.0);
    let normalized_entropy = entropy / max_entropy;

    println!(
        "Rollout length: {}, 4-gram normalized entropy: {:.4}",
        gen.len(),
        normalized_entropy
    );
    assert!(
        normalized_entropy >= 0.94,
        "4-gram repetition entropy on generated rollout must remain >= 0.94 (got {:.4})",
        normalized_entropy
    );
}
