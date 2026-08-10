#![cfg(feature = "anti-degeneracy")]
use uor_r4_graph_certify::anti_degeneracy::{
    MdlObjective, ParaphraseEvaluator, PerturbationKind, PerturbationSuite, PolysemyEvaluator,
    SemanticCoherenceCertificate,
};

#[test]
fn test_perturbation_suite_transformations() {
    let tokens = vec![101, 2054, 2003, 1037, 3899, 102];
    let seed = 0x5EEDu64;

    // Masking
    let masked = PerturbationSuite::apply_perturbation(
        &tokens,
        &PerturbationKind::Masking {
            mask_rate: 0.5,
            mask_token: u32::MAX,
        },
        seed,
    );
    assert_eq!(masked.len(), tokens.len());
    assert!(masked.contains(&u32::MAX));

    // Span substitution
    let substituted = PerturbationSuite::apply_perturbation(
        &tokens,
        &PerturbationKind::SpanSubstitution {
            substitute_rate: 1.0,
        },
        seed,
    );
    assert_eq!(substituted.len(), tokens.len());
    assert_eq!(substituted[0], tokens[0] + 100);

    let truncated = PerturbationSuite::apply_perturbation(
        &tokens,
        &PerturbationKind::Truncation { keep_fraction: 0.5 },
        seed,
    );
    assert_eq!(truncated.len(), 3);

    // Reorder
    let reordered = PerturbationSuite::apply_perturbation(
        &tokens,
        &PerturbationKind::Reorder { shuffle_window: 3 },
        seed,
    );
    assert_eq!(reordered.len(), tokens.len());
    assert_ne!(reordered, tokens);

    // Counterfactual
    let counterfactual = PerturbationSuite::apply_perturbation(
        &tokens,
        &PerturbationKind::Counterfactual {
            flip_polarity: true,
        },
        seed,
    );
    assert_eq!(counterfactual.len(), tokens.len());
    assert_eq!(counterfactual[0], 101 ^ 1);
}

#[test]
fn test_mdl_objective_j_c() {
    let j_c1 = MdlObjective::compute_j_c(1000, 5000.0, 1.0);
    let j_c2 = MdlObjective::compute_j_c(2000, 3000.0, 1.0);
    assert_eq!(j_c1, 8000.0 + 5000.0);
    assert_eq!(j_c2, 16000.0 + 3000.0);
}

#[test]
fn test_paraphrase_and_polysemy_evaluators() {
    let t1 = vec![1, 2, 3, 4, 5];
    let t2 = vec![1, 2, 9, 4, 5];
    let agreement = ParaphraseEvaluator::evaluate_paraphrase_agreement(&t1, &t2);
    assert_eq!(agreement, 0.8);

    let ca = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let cb = vec![vec![1, 2, 9], vec![4, 5, 6]];
    let separation = PolysemyEvaluator::evaluate_polysemy_separation(&ca, &cb);
    assert!((separation - 1.0 / 6.0).abs() < 1e-6);
}

#[test]
fn test_semantic_coherence_certificate_verification() {
    let valid_cert = SemanticCoherenceCertificate {
        region_reuse_rate: 0.65,
        invariance_score: 0.82,
        boundary_stability: 0.91,
        mdl_cost_j_c: 12500.0,
        anti_memorization_passed: true,
    };
    assert!(valid_cert.verify());

    let invalid_cert = SemanticCoherenceCertificate {
        region_reuse_rate: 0.30, // Failing reuse rate
        invariance_score: 0.82,
        boundary_stability: 0.91,
        mdl_cost_j_c: 12500.0,
        anti_memorization_passed: true,
    };
    assert!(!invalid_cert.verify());
}
