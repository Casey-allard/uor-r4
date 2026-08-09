use uor_r4_graph_certify::holographic_encoding::{
    AblationProtocol, DivergenceMetric, HolographicEncodingCertificate,
    HolographicEncodingEvaluator, HolographicProbeReport, Projection, ProjectionMetadata,
};

fn deterministic_fixture_projections() -> Vec<Projection> {
    vec![
        Projection {
            metadata: ProjectionMetadata {
                projection_id: "h0".to_string(),
                depth: 1,
                membership_ids: vec![10, 11],
            },
            recovered_distribution: vec![0.60, 0.25, 0.15],
        },
        Projection {
            metadata: ProjectionMetadata {
                projection_id: "h1".to_string(),
                depth: 2,
                membership_ids: vec![11, 12, 13],
            },
            recovered_distribution: vec![0.66, 0.22, 0.12],
        },
        Projection {
            metadata: ProjectionMetadata {
                projection_id: "h2".to_string(),
                depth: 3,
                membership_ids: vec![12, 13, 14, 15],
            },
            recovered_distribution: vec![0.72, 0.19, 0.09],
        },
    ]
}

#[test]
fn test_holographic_fixture_partial_recovery_and_progressive_fidelity() {
    let teacher = vec![0.70, 0.20, 0.10];
    let projections = deterministic_fixture_projections();

    assert!(
        HolographicEncodingEvaluator::validate_projection_family(&projections).is_none(),
        "valid fixture must have no degeneracy"
    );
    let partial = HolographicEncodingEvaluator::partial_recovery(
        &projections,
        &teacher,
        DivergenceMetric::KLDivergence,
    );
    assert_eq!(partial.len(), 3);
    assert!(
        partial[2].divergence_to_teacher < partial[0].divergence_to_teacher,
        "deeper projection should recover behavior better in deterministic fixture"
    );

    let progressive = HolographicEncodingEvaluator::progressive_fidelity(
        &projections,
        &teacher,
        DivergenceMetric::KLDivergence,
    )
    .expect("progressive fidelity");
    assert_eq!(progressive.len(), 3);
    assert!(
        progressive[2].divergence_to_teacher < progressive[0].divergence_to_teacher,
        "adding overlapping projections should improve fidelity"
    );

    let distributed =
        HolographicEncodingEvaluator::distributed_evidence_mean_support(&projections, 0.10);
    assert!(
        distributed > 1.0,
        "evidence should be distributed across projections"
    );
}

#[test]
fn test_holographic_ablation_and_probe_metrics() {
    let teacher = vec![0.70, 0.20, 0.10];
    let projections = deterministic_fixture_projections();
    let protocol = AblationProtocol {
        baseline_projection_ids: vec!["h0".to_string(), "h1".to_string(), "h2".to_string()],
        ablation_order: vec!["h2".to_string(), "h1".to_string()],
        semantics: "leave-one-out from full H(x)".to_string(),
    };

    let ablation = HolographicEncodingEvaluator::ablation_curve(
        &projections,
        &teacher,
        DivergenceMetric::JensenShannonDivergence,
        &protocol,
    )
    .expect("ablation curve");
    assert_eq!(ablation.len(), 2);
    assert!(
        ablation[1].divergence_to_teacher >= ablation[0].divergence_to_teacher,
        "removing more projections should not improve fidelity in this fixture"
    );

    let full = HolographicEncodingEvaluator::recover_behavior_distribution(
        &projections,
        &["h0".to_string(), "h1".to_string(), "h2".to_string()],
    )
    .expect("full recovery");
    let paraphrase = vec![0.69, 0.21, 0.10];
    let perturbed = vec![0.58, 0.27, 0.15];

    let paraphrase_invariance = HolographicEncodingEvaluator::paraphrase_invariance(
        &full,
        &paraphrase,
        DivergenceMetric::JensenShannonDivergence,
    );
    let perturbation_stability = HolographicEncodingEvaluator::perturbation_stability(
        &full,
        &perturbed,
        DivergenceMetric::JensenShannonDivergence,
    );
    let cross_context_reuse =
        HolographicEncodingEvaluator::cross_context_reuse(&[10, 11, 12], &[11, 12, 13]);

    assert!(
        paraphrase_invariance < perturbation_stability,
        "paraphrase should be more stable than perturbation"
    );
    assert!((cross_context_reuse - 0.5).abs() < 1e-9);
}

#[test]
fn test_holographic_certificate_schema_and_cid() {
    let projections = deterministic_fixture_projections();
    let probe_report = HolographicProbeReport {
        partial_recovery: vec![],
        distributed_evidence_mean_support: 2.0,
        progressive_fidelity: vec![],
        ablation_curve: vec![],
        paraphrase_invariance: 0.01,
        perturbation_stability: 0.08,
        cross_context_reuse: 0.5,
    };

    let cert = HolographicEncodingCertificate::new(
        projections.iter().map(|p| p.metadata.clone()).collect(),
        AblationProtocol {
            baseline_projection_ids: vec!["h0".to_string(), "h1".to_string(), "h2".to_string()],
            ablation_order: vec!["h2".to_string()],
            semantics: "leave-one-out".to_string(),
        },
        DivergenceMetric::JensenShannonDivergence,
        4096,
        (0.01, 0.03),
        vec![
            "kappa:blake3:corpus-primary".to_string(),
            "kappa:blake3:corpus-paraphrase".to_string(),
        ],
        vec![
            "H(x) is represented as overlapping projections with explicit memberships.".to_string(),
            "Recovery uses averaging over active projections; no single projection is required."
                .to_string(),
        ],
        vec![
            "Teacher agreement is measured with declared divergence and confidence interval."
                .to_string(),
            "This report does not claim exact reconstruction.".to_string(),
        ],
        probe_report,
    );

    assert!(cert.verify_cid());
    assert_eq!(cert.corpus_cids.len(), 2);
    assert_eq!(cert.sample_count, 4096);
    assert_eq!(cert.confidence_interval_95, (0.01, 0.03));
}

#[test]
fn test_holographic_degenerate_encodings_rejected() {
    let empty = HolographicEncodingEvaluator::validate_projection_family(&[]);
    assert!(empty.is_some_and(|reason| reason.contains("empty")));

    let single_node = vec![Projection {
        metadata: ProjectionMetadata {
            projection_id: "degenerate-single".to_string(),
            depth: 1,
            membership_ids: vec![7],
        },
        recovered_distribution: vec![1.0, 0.0],
    }];
    assert!(
        HolographicEncodingEvaluator::validate_projection_family(&single_node)
            .is_some_and(|reason| reason.contains("single-node memorization"))
    );

    let duplicate = vec![
        Projection {
            metadata: ProjectionMetadata {
                projection_id: "dup-a".to_string(),
                depth: 1,
                membership_ids: vec![1, 2],
            },
            recovered_distribution: vec![0.5, 0.5],
        },
        Projection {
            metadata: ProjectionMetadata {
                projection_id: "dup-b".to_string(),
                depth: 2,
                membership_ids: vec![1, 2],
            },
            recovered_distribution: vec![0.5, 0.5],
        },
    ];
    assert!(
        HolographicEncodingEvaluator::validate_projection_family(&duplicate)
            .is_some_and(|reason| reason.contains("duplicate projection '"))
    );

    let duplicate_ids = vec![
        Projection {
            metadata: ProjectionMetadata {
                projection_id: "same-id".to_string(),
                depth: 1,
                membership_ids: vec![1, 2],
            },
            recovered_distribution: vec![0.8, 0.2],
        },
        Projection {
            metadata: ProjectionMetadata {
                projection_id: "same-id".to_string(),
                depth: 2,
                membership_ids: vec![2, 3],
            },
            recovered_distribution: vec![0.7, 0.3],
        },
    ];
    assert!(
        HolographicEncodingEvaluator::validate_projection_family(&duplicate_ids)
            .is_some_and(|reason| reason.contains("duplicate projection id"))
    );

    let inconsistent = vec![
        Projection {
            metadata: ProjectionMetadata {
                projection_id: "len-a".to_string(),
                depth: 1,
                membership_ids: vec![1, 2],
            },
            recovered_distribution: vec![0.8, 0.2],
        },
        Projection {
            metadata: ProjectionMetadata {
                projection_id: "len-b".to_string(),
                depth: 2,
                membership_ids: vec![2, 3],
            },
            recovered_distribution: vec![0.6, 0.3, 0.1],
        },
    ];
    assert!(
        HolographicEncodingEvaluator::validate_projection_family(&inconsistent)
            .is_some_and(|reason| reason.contains("does not match expected"))
    );
}

#[test]
fn test_ablation_curve_rejects_unknown_ablation_id() {
    let teacher = vec![0.70, 0.20, 0.10];
    let projections = deterministic_fixture_projections();
    let protocol = AblationProtocol {
        baseline_projection_ids: vec!["h0".to_string(), "h1".to_string()],
        ablation_order: vec!["h2".to_string()],
        semantics: "invalid ablation id".to_string(),
    };

    assert!(
        HolographicEncodingEvaluator::ablation_curve(
            &projections,
            &teacher,
            DivergenceMetric::KLDivergence,
            &protocol,
        )
        .is_none(),
        "unknown ablation id yields no curve"
    );
}

#[test]
fn test_progressive_fidelity_propagates_recovery_errors() {
    let teacher = vec![0.70, 0.20, 0.10];
    let projections = vec![
        Projection {
            metadata: ProjectionMetadata {
                projection_id: "h0".to_string(),
                depth: 1,
                membership_ids: vec![10, 11],
            },
            recovered_distribution: vec![0.60, 0.40],
        },
        Projection {
            metadata: ProjectionMetadata {
                projection_id: "h1".to_string(),
                depth: 2,
                membership_ids: vec![11, 12],
            },
            recovered_distribution: vec![0.50, 0.30, 0.20],
        },
    ];

    assert!(
        HolographicEncodingEvaluator::progressive_fidelity(
            &projections,
            &teacher,
            DivergenceMetric::KLDivergence,
        )
        .is_none(),
        "length mismatch yields no fidelity curve"
    );
}
