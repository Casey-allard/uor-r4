//! Native Geometric Language Learner (Card P3).
//!
//! Continuous offline training using Adam, gradients, and JEPA latent state prediction.
//! Discrete table export for zero-GEMM, zero-runtime-float, zero-heap-allocation inference.

pub mod binary_model;
pub mod chat;
pub mod cold_prior;
pub mod contextual_emission;
pub mod embedding;
pub mod geometric_attention;
pub mod group_table;
pub mod head_projection;
pub mod jepa_trainer;
pub mod lowbit;
pub mod lowbit_attention;
pub mod lowbit_core;
pub mod occurrence;
pub mod policy_feasibility;
pub mod prefix_artifact;
pub mod prefix_state;
pub mod prior_learning;
pub mod query_read;
pub mod read_conditioned;
pub mod realtext_support;
pub mod relational;
pub mod transition_table;
pub mod vsa_codes;

pub use binary_model::{
    BinaryModelError, MmapGeometricModel, RgmHeader, RgmSectionHeader, FLAG_HAS_ENGRAM_TABLE,
    FLAG_HAS_HIERARCHICAL_CODEBOOK, FLAG_HAS_HIERARCHICAL_LATTICE, FLAG_HAS_JEPA, NUM_SECTIONS,
    RGM_HEADER_SIZE, RGM_MAGIC, RGM_VERSION, SECTION_BASE, SECTION_CODEBOOK, SECTION_ENGRAM,
    SECTION_JEPA, SECTION_LANES, SECTION_LATTICE,
};
pub use cold_prior::{ColdPriorConfig, ColdPriorCore, ColdPriorTrainer};
pub use embedding::{
    canonical_h4_fiber_roots_q30, canonical_h4_hopf_s2, canonical_h4_roots, canonical_h4_roots_q30,
    ContinuousEmbedding, H4_ROOT_COUNT, PHI,
};
pub use geometric_attention::{GeometricAttention, GeometricAttentionTrainer};
pub use group_table::{group_table, GroupTable, GROUP_ORDER, ROW_STRIDE};
pub use jepa_trainer::{
    ExportedGeometricModel, JepaTrainer, JepaTrainerConfig, NativeGeometricLearnerModel,
    TrainingMetrics,
};
pub use lowbit::TernaryLinear;
pub use lowbit_attention::{LowBitAttention, LowBitAttentionTrainer};
pub use lowbit_core::{LowBitCore, LowBitCoreTrainer, TrainConfig, DEFAULT_STATE_DIM};
pub use occurrence::{OccurrenceArtifact, OccurrenceRing, Selector, SelectorTrainer};
pub use prior_learning::{PriorCore, PriorTrainer};
pub use query_read::{QueryArm, QueryHard, QueryTrainer};
pub use relational::{RelationalSelector, RelationalTrainer, ACTS, CTX_BUCKETS};
pub use transition_table::{
    ContinuousLaneTable, DiscreteServingTable, MultiLaneTransitionTables, ENTRIES_PER_TABLE,
};
pub use vsa_codes::{build_root_codebook, root_codes};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_h4_roots_properties() {
        let roots = canonical_h4_roots();
        assert_eq!(roots.len(), 120);

        for r in roots.iter() {
            let norm_sq = r.a * r.a + r.b * r.b + r.c * r.c + r.d * r.d;
            assert!(
                (norm_sq - 1.0).abs() < 1e-10,
                "H4 root must lie on unit sphere S3"
            );
        }

        let s2_roots = canonical_h4_hopf_s2();
        assert_eq!(s2_roots.len(), 120);
        for u in s2_roots.iter() {
            let norm_sq = u.x * u.x + u.y * u.y + u.z * u.z;
            assert!(
                (norm_sq - 1.0).abs() < 1e-10,
                "Hopf projection must lie on unit sphere S2"
            );
        }
    }

    #[test]
    fn test_continuous_embedding_hopf_and_assignment() {
        let vocab_size = 257;
        let emb = ContinuousEmbedding::new(vocab_size, 42);

        for token in 0..vocab_size {
            let (q_base, q_comp) = emb.forward_quaternions(token);
            assert!(
                (q_base.a * q_base.a
                    + q_base.b * q_base.b
                    + q_base.c * q_base.c
                    + q_base.d * q_base.d
                    - 1.0)
                    .abs()
                    < 1e-10
            );
            assert!(
                (q_comp.a * q_comp.a
                    + q_comp.b * q_comp.b
                    + q_comp.c * q_comp.c
                    + q_comp.d * q_comp.d
                    - 1.0)
                    .abs()
                    < 1e-10
            );

            let root_idx = emb.nearest_h4_root(token);
            assert!(root_idx < 120);

            let dist = emb.soft_h4_distribution(token);
            assert_eq!(dist.len(), 120);
            let sum: f64 = dist.iter().sum();
            assert!((sum - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_transition_table_and_discrete_quantization() {
        let table = ContinuousLaneTable::new(12345);
        assert_eq!(table.logits.len(), 14_400);

        // Score a couple entries
        let s0 = table.score_discrete(0, 0);
        assert!(s0.is_finite());

        // Quantize
        let discrete = table.quantize();
        assert_eq!(discrete.scores.len(), 14_400);

        // Verify discrete lookup is exact and bounded
        for q in 0..120 {
            for k in 0..120 {
                let disc_score = discrete.score(q, k);
                assert!(disc_score >= -32767);
            }
        }
    }

    #[test]
    fn test_jepa_trainer_learns_and_reduces_bpb() {
        let text =
            b"Once upon a time there was a little girl who lived in a forest. She had a kind cat.";
        let tokens: Vec<usize> = text.iter().map(|&b| b as usize).collect();

        let config = JepaTrainerConfig {
            vocab_size: 257,
            num_lanes: 2,
            context_window: 16,
            learning_rate: 0.05,
            jepa_weight: 0.2,
            weight_decay: 1e-4,
            grad_clip: 1.0,
            ..JepaTrainerConfig::default()
        };

        let mut trainer = JepaTrainer::new(config, 999);

        // Initial held-out BPB
        let initial_bpb = trainer.evaluate_bpb(&tokens);
        assert!(initial_bpb > 0.0);

        // Train for 20 epochs on this text
        let mut last_metrics = TrainingMetrics::default();
        for _ in 0..20 {
            last_metrics = trainer.train_sequence(&tokens);
        }

        let final_bpb = trainer.evaluate_bpb(&tokens);
        assert!(
            final_bpb < initial_bpb,
            "Training must decrease BPB: initial {initial_bpb:.4} -> final {final_bpb:.4}"
        );
        assert!(last_metrics.ce_loss > 0.0);
        assert!(last_metrics.jepa_loss >= 0.0);

        // Test discrete export
        let exported = trainer.export_discrete();
        assert_eq!(exported.token_to_root.len(), 257);
        assert_eq!(exported.discrete_tables.len(), 2);
        assert_eq!(exported.discrete_tables[0].scores.len(), 14_400);

        // Test O(1) score compatibility with zero runtime matrix operations
        let comp_score = exported.score_compatibility(b'a' as usize, b't' as usize);
        assert!(comp_score != 0 || exported.score_compatibility(0, 0) == 0);
    }

    #[test]
    fn test_exported_model_context_scoring_and_hopf_state() {
        let config = JepaTrainerConfig {
            vocab_size: 257,
            num_lanes: 4,
            context_window: 16,
            learning_rate: 0.05,
            jepa_weight: 0.2,
            weight_decay: 1e-4,
            grad_clip: 1.0,
            ..JepaTrainerConfig::default()
        };
        let mut trainer = JepaTrainer::new(config, 1234);
        let sample = b"The little rabbit hopped happily in the meadow.";
        let tokens: Vec<usize> = sample.iter().map(|&b| b as usize).collect();
        for _ in 0..5 {
            trainer.train_sequence(&tokens);
        }

        let exported = trainer.export_discrete();
        assert_eq!(exported.discrete_tables.len(), 4);

        // Test context Hopf S2 and fiber holonomy state
        let context = vec![b'T' as usize, b'h' as usize, b'e' as usize];
        let fiber_state = exported.context_hopf_fiber_q30(&context);
        let s2_state = fiber_state.base;
        let norm_sq = s2_state.0[0] as i64 * s2_state.0[0] as i64
            + s2_state.0[1] as i64 * s2_state.0[1] as i64
            + s2_state.0[2] as i64 * s2_state.0[2] as i64;
        assert!(norm_sq > 0, "S2 state must have non-zero norm in Q1.30");

        // Test multi-lane context candidate scoring with fiber and VSA
        let score_space = exported.score_context_candidate(&context, b' ' as usize, fiber_state);
        let score_z = exported.score_context_candidate(&context, b'z' as usize, fiber_state);
        assert_ne!(
            score_space, score_z,
            "Candidates must receive distinct scores"
        );

        // Edge case: empty context
        let empty_fiber = exported.context_hopf_fiber_q30(&[]);
        let score_empty = exported.score_context_candidate(&[], b'a' as usize, empty_fiber);
        assert!(score_empty != i32::MIN);

        // Edge case: out of vocabulary token clamp
        let score_oov = exported.score_context_candidate(&[500, 999], 1000, empty_fiber);
        assert!(score_oov != i32::MIN);
    }

    #[test]
    fn test_runtime_predict_offers_ascii_with_prose_tables() {
        use crate::native_geometric::{Config, Control, Document, Trainer};

        let docs = [Document {
            id: "d1".into(),
            text: "The rabbit hopped in the field.\n".into(),
        }];
        let mut trainer = Trainer::new(
            Config {
                candidate_limit: 32,
                ..Config::default()
            },
            &docs,
        )
        .expect("trainer init");
        trainer.train_documents(&docs).expect("train docs");
        let mut model = trainer.compile().expect("compile");

        // Attach exported prose tables
        let jepa_config = JepaTrainerConfig {
            vocab_size: 257,
            num_lanes: 2,
            ..JepaTrainerConfig::default()
        };
        let jepa_trainer = JepaTrainer::new(jepa_config, 42);
        let exported = jepa_trainer.export_discrete();
        model.geometric_prose_tables = Some(exported);

        let mut session = model.session(Control::default()).expect("session");
        session.observe(&model, b'T' as u32).expect("observe");
        let pred = session.predict(&model).expect("predict");
        assert!(pred.candidate_count > 0);

        // Verify that candidates include printable ASCII characters (>= 32)
        // Previously, with candidate_limit: 32, it only offered tokens 0..31!
        let has_printable = session
            .candidates()
            .iter()
            .any(|c| c.token >= 32 && c.token <= 126);
        assert!(
            has_printable,
            "Predict must offer printable ASCII candidates (> 32) when prose tables are present"
        );
    }

    #[test]
    fn test_engram_integration_and_repetition_entropy() {
        let config = JepaTrainerConfig {
            vocab_size: 257,
            num_lanes: 4,
            context_window: 16,
            learning_rate: 0.05,
            jepa_weight: 0.2,
            weight_decay: 1e-4,
            grad_clip: 1.0,
            ..JepaTrainerConfig::default()
        };
        let mut trainer = JepaTrainer::new(config, 2026_0917);
        let sample1 = b"The swift brown fox jumps over the lazy sleeping dog in the forest.";
        let sample2 = b"The quick brown rabbit leaps across the green meadow under the blue sky.";
        let tokens1: Vec<usize> = sample1.iter().map(|&b| b as usize).collect();
        let tokens2: Vec<usize> = sample2.iter().map(|&b| b as usize).collect();

        for _ in 0..10 {
            trainer.train_sequence(&tokens1);
            trainer.train_sequence(&tokens2);
        }

        let exported = trainer.export_discrete();
        assert!(
            exported.engram_table.is_some(),
            "Engram table must be present on export"
        );
        let engram = exported.engram_table.as_ref().unwrap();
        assert!(
            !engram.is_empty(),
            "Engram table must contain learned collocations"
        );

        // Verify bigram lookup for 'T', 'h' -> continuation 'e' has positive Q1.15 score
        let cands = engram.lookup_bigram(b'T' as u32, b'h' as u32);
        assert!(
            cands.is_some(),
            "Bigram ('T', 'h') must be present in engram table"
        );
        let found_e = cands
            .unwrap()
            .iter()
            .any(|&(tok, score)| tok == b'e' as u32 && score > 0);
        assert!(
            found_e,
            "Continuation 'e' must have positive collocation score"
        );

        // Generation rollout: free-running generation with temperature sampling
        let prompt = b"The swift";
        let mut gen: Vec<usize> = prompt.iter().map(|&b| b as usize).collect();
        let mut rng = 20260917_u64;

        for _ in 0..64 {
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

        assert!(
            normalized_entropy > 0.80,
            "4-gram repetition entropy must exceed 0.80 without artificial penalties (got {:.4})",
            normalized_entropy
        );
    }
}
