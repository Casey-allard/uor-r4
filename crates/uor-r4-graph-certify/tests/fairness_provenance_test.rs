#![cfg(feature = "fairness-provenance")]
use uor_r4_core::transformerless::score_q::ScoreQ;
use uor_r4_core::transformerless::transitions::{EdgeKind, TransitionGraph};
use uor_r4_graph_certify::fairness_provenance::{
    FairnessAndProvenanceCertificate, FairnessEvaluator, ProvenanceDeletionWitness,
};

fn graph_with_tombstoned_edges(edge_weights: &[u32]) -> TransitionGraph {
    let mut graph = TransitionGraph::new();
    for (i, &weight) in edge_weights.iter().enumerate() {
        graph.add_edge_with_score(
            i as u32,
            i as u32 + 1,
            weight,
            ScoreQ::from_logprob(-0.5),
            EdgeKind::Forward,
        );
    }
    graph
}

#[test]
fn test_bias_amplification_and_rare_group_erasure() {
    let teacher_counts = vec![
        ("majority_slice".to_string(), 1000),
        ("minority_slice".to_string(), 20),
    ];

    let graph_counts = vec![
        ("majority_slice".to_string(), 1020), // 1.02x amplification
        ("minority_slice".to_string(), 20),   // 100% retention
    ];

    let (bias_metrics, rare_group_retention) =
        FairnessEvaluator::evaluate_bias_and_erasure(&teacher_counts, &graph_counts);

    let graph = graph_with_tombstoned_edges(&[0, 0, 0, 0, 0]);
    let witness =
        FairnessEvaluator::verify_provenance_deletion("doc_999", &graph, &[0, 1, 2, 3, 4])
            .expect("provenance deletion verification");

    let cert = FairnessAndProvenanceCertificate::new(bias_metrics, rare_group_retention, witness);

    assert!(cert.bias_amplification_passed);
    assert!(cert.rare_group_erasure_passed);
    assert!(cert.verify_cid());
}

#[test]
fn test_fairness_certificate_threshold_failure() {
    let teacher_counts = vec![
        ("majority_slice".to_string(), 500),
        ("minority_slice".to_string(), 40),
    ];

    let graph_counts = vec![
        ("majority_slice".to_string(), 995), // 0.995 / 0.9259 = 1.074x amplification (> 1.05)
        ("minority_slice".to_string(), 5),   // 5/40 = 12.5% retention (< 0.95)
    ];

    let (bias_metrics, rare_group_retention) =
        FairnessEvaluator::evaluate_bias_and_erasure(&teacher_counts, &graph_counts);

    let graph = graph_with_tombstoned_edges(&[0, 0]);
    let witness = FairnessEvaluator::verify_provenance_deletion("doc_888", &graph, &[0, 1])
        .expect("provenance deletion verification");

    let cert = FairnessAndProvenanceCertificate::new(bias_metrics, rare_group_retention, witness);

    assert!(
        !cert.bias_amplification_passed,
        "Must fail bias amplification > 1.05"
    );
    assert!(
        !cert.rare_group_erasure_passed,
        "Must fail rare group retention < 0.95"
    );
    assert!(cert.verify_cid());
}

#[test]
fn test_fairness_certificate_cbor_roundtrip() {
    let witness = ProvenanceDeletionWitness {
        deleted_slice_id: "doc_123".to_string(),
        tombstoned_provenance_nodes: 3,
        graph_integrity_verified: true,
    };

    let cert = FairnessAndProvenanceCertificate::new(vec![], vec![], witness);

    let cbor_bytes = cert.to_cbor_bytes();
    let decoded =
        FairnessAndProvenanceCertificate::from_cbor_bytes(&cbor_bytes).expect("deserialize CBOR");

    assert_eq!(cert, decoded);
    assert!(decoded.verify_cid());
}

#[test]
fn test_verify_provenance_deletion_detects_untombstoned_edge() {
    let graph = graph_with_tombstoned_edges(&[0, 7]);

    assert!(
        FairnessEvaluator::verify_provenance_deletion("doc_777", &graph, &[0, 1]).is_none(),
        "edge 1 still has non-zero weight, so no deletion witness is produced"
    );

    assert!(
        FairnessEvaluator::verify_provenance_deletion("doc_777", &graph, &[5]).is_none(),
        "edge 5 is out of bounds, so no deletion witness is produced"
    );
}
