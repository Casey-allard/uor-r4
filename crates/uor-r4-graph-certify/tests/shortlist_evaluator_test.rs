#![cfg(feature = "shortlist-evaluator")]
use uor_r4_graph_certify::shortlist_evaluator::{ShortlistEvaluator, ShortlistRecallReport};

#[test]
fn test_shortlist_evaluator_metrics_and_gate_h() {
    let shortlists = vec![
        vec![10, 20, 30, 40, 50],
        vec![100, 200, 300, 400, 500],
        vec![1, 2, 3, 4, 5],
        vec![99, 88, 77, 66, 55],
    ];
    let references = vec![10, 200, 3, 55]; // 10 is top-1, 200 is top-2, 3 is top-3, 55 is top-5
    let scores_graph = vec![100, 200, 300, 400];
    let scores_teacher = vec![105, 195, 302, 398];

    let report = ShortlistEvaluator::evaluate(
        &shortlists,
        &references,
        &scores_graph,
        &scores_teacher,
        0.90,
    );

    assert_eq!(report.metrics.top1_recall, 0.25);
    assert_eq!(report.metrics.top5_recall, 1.00);
    assert_eq!(report.metrics.false_negative_rate, 0.0);
    assert_eq!(report.metrics.worst_routing_error, 5);
    assert!(report.metrics.gate_h_passed);
    assert!(!report.trigger_gated_fallback_active);
}

#[test]
fn test_decision_d5_trigger_gated_fallback() {
    let shortlists = vec![
        vec![10, 20, 30],
        vec![100, 200, 300],
        vec![1, 2, 3],
        vec![99, 88, 77],
    ];
    let references = vec![999, 999, 999, 999]; // None in shortlists -> 0% recall!
    let scores_graph = vec![100];
    let scores_teacher = vec![100];

    let report = ShortlistEvaluator::evaluate(
        &shortlists,
        &references,
        &scores_graph,
        &scores_teacher,
        0.95,
    );

    assert_eq!(report.metrics.top5_recall, 0.0);
    assert_eq!(report.metrics.false_negative_rate, 1.0);
    assert!(!report.metrics.gate_h_passed);
    assert!(
        report.trigger_gated_fallback_active,
        "Decision D5 trigger fallback must activate when recall < threshold"
    );
}

#[test]
fn test_shortlist_recall_report_cbor_roundtrip() {
    let shortlists = vec![vec![1, 2, 3, 4, 5]];
    let references = vec![1];
    let report = ShortlistEvaluator::evaluate(&shortlists, &references, &[10], &[10], 0.95);

    let cbor_bytes = report.to_cbor_bytes();
    let decoded = ShortlistRecallReport::from_cbor_bytes(&cbor_bytes).expect("deserialize CBOR");
    assert_eq!(report, decoded);
}
