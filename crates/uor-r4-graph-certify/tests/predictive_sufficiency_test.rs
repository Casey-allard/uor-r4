#![cfg(feature = "predictive-sufficiency")]
use uor_r4_core::transformerless::runtime::OpKernel;
use uor_r4_graph_certify::predictive_sufficiency::{
    DivergencePoint, GraphDepth, PredictiveSufficiencyEvaluator, RateDistortionReport,
};

#[test]
fn test_kl_divergence_and_cross_entropy() {
    let p = vec![0.5, 0.25, 0.25];
    let q = vec![0.5, 0.25, 0.25];

    let kl_self = PredictiveSufficiencyEvaluator::compute_kl_divergence(&p, &q);
    assert!(
        kl_self.abs() < 1e-6,
        "KL divergence of distribution with itself must be ~0"
    );

    // Edge case: zeros in the distribution should not introduce spurious mass.
    let pz = vec![1.0, 0.0, 0.0];
    let qz = vec![1.0, 0.0, 0.0];
    let kl_zeros = PredictiveSufficiencyEvaluator::compute_kl_divergence(&pz, &qz);
    assert!(
        kl_zeros.abs() < 1e-12,
        "KL divergence must be ~0 when P contains zeros and P==Q"
    );
    let ce_zeros = PredictiveSufficiencyEvaluator::compute_cross_entropy(&pz, &qz);
    assert!(ce_zeros.abs() < 1e-12);

    let q2 = vec![0.8, 0.1, 0.1];
    let kl_diff = PredictiveSufficiencyEvaluator::compute_kl_divergence(&p, &q2);
    assert!(
        kl_diff > 0.0,
        "KL divergence must be positive for distinct distributions"
    );

    let ce = PredictiveSufficiencyEvaluator::compute_cross_entropy(&p, &q2);
    assert!(ce > 0.0);
}

#[test]
fn test_predictive_sufficiency_evaluator_depths() {
    let teacher = vec![0.7, 0.2, 0.1];
    let broad = vec![0.33, 0.33, 0.34];
    let _intermediate = [0.5, 0.3, 0.2];
    let _full = [0.65, 0.25, 0.1];
    let residual = vec![0.7, 0.2, 0.1];

    let ops = OpKernel::default();

    let pt_broad = PredictiveSufficiencyEvaluator::evaluate_depth(
        &teacher,
        &broad,
        GraphDepth::BroadCloud,
        1000,
        ops.clone(),
    );

    let pt_residual = PredictiveSufficiencyEvaluator::evaluate_depth(
        &teacher,
        &residual,
        GraphDepth::ResidualAugmented,
        5000,
        ops,
    );

    assert!(
        pt_residual.kl_divergence < pt_broad.kl_divergence,
        "Residual depth KL divergence must be strictly less than BroadCloud depth"
    );
    assert_eq!(pt_residual.top5_recall, 1.0);

    let top5_miss = PredictiveSufficiencyEvaluator::evaluate_depth(
        &[0.9, 0.02, 0.02, 0.02, 0.02, 0.02],
        &[0.01, 0.2, 0.2, 0.2, 0.2, 0.19],
        GraphDepth::BroadCloud,
        1000,
        OpKernel::default(),
    );
    assert_eq!(top5_miss.top5_recall, 0.0);
}

#[test]
fn test_rate_distortion_report_cbor_roundtrip() {
    let p1 = DivergencePoint {
        depth: GraphDepth::BroadCloud,
        kl_divergence: 1.25,
        cross_entropy: 2.10,
        top1_accuracy: 0.60,
        top5_recall: 0.85,
        bytes_footprint: 1024,
        op_budget: OpKernel::default(),
    };

    let p2 = DivergencePoint {
        depth: GraphDepth::ResidualAugmented,
        kl_divergence: 0.05,
        cross_entropy: 1.15,
        top1_accuracy: 0.98,
        top5_recall: 1.00,
        bytes_footprint: 8192,
        op_budget: OpKernel::default(),
    };

    let report = RateDistortionReport::new(vec![p1, p2]);
    let curve = report.compute_rate_distortion_curve();
    assert_eq!(curve.len(), 2);
    assert_eq!(curve[0].1, 1.25);
    assert_eq!(curve[1].1, 0.05);

    let cbor_bytes = report.to_cbor_bytes();
    let decoded = RateDistortionReport::from_cbor_bytes(&cbor_bytes).expect("deserialize CBOR");
    assert_eq!(report, decoded);
}
