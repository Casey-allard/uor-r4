//! Wiring check for `run_ablation_benchmark` (issue #434 item 2).
//!
//! This file used to be the only coverage `SpectralGeometry` and
//! `VsaGeometry` had, and it could not fail: it asserted
//! `migration_agreement == 0.98` against a hard-coded `0.98`, and
//! `recall_at_3 >= 0.0` / `hits_at_3 >= 0.0` against ratios of non-negative
//! counts. Two synthetic queries, no corpus, no ground truth, no null.
//!
//! It is now scoped honestly: `run_ablation_benchmark` is a **wiring check**
//! — does each geometry ground, encode and soft-route without erroring, and
//! does the result carry the query count so an empty run cannot masquerade as
//! a measured zero. Retrieval *quality* is measured in
//! `tests/geometry_ablation.rs`, on a real corpus, through the production
//! surface, with a deranged-key null.

use uor_r4_router::benchmark::run_ablation_benchmark;
use uor_r4_router::geometry::TypedObject;
use uor_r4_router::UorR4Router;

#[test]
fn ablation_benchmark_reports_both_geometries_over_the_supplied_queries() {
    let router = UorR4Router::new(0.5);

    let queries = vec![
        (
            TypedObject {
                object_type: "document".to_string(),
                content: "borehole depth gambling".to_string(),
            },
            1,
        ),
        (
            TypedObject {
                object_type: "document".to_string(),
                content: "temporal aquifer dry season".to_string(),
            },
            2,
        ),
    ];

    let results = run_ablation_benchmark(&router, &queries);
    assert_eq!(results.len(), 2, "one row per geometry");

    for result in &results {
        // The query count is the guard that makes the recall reading
        // meaningful: without it, a run that scored nothing and a run that
        // scored zero out of zero are the same number.
        assert_eq!(
            result.queries,
            queries.len(),
            "[{}] every supplied query was scored",
            result.geometry_name
        );
        assert!(
            (0.0..=1.0).contains(&result.recall_at_3),
            "[{}] recall is a fraction, got {}",
            result.geometry_name,
            result.recall_at_3
        );
    }

    assert_eq!(results[0].geometry_name, "Spectral Heuristic");
    assert_eq!(results[1].geometry_name, "VSA Grounded");
}

/// An empty query set must report zero queries, not a recall that reads as a
/// measured zero. This is the assertion the old file was missing, and the
/// shape of the mistake it encoded.
#[test]
fn empty_query_set_is_distinguishable_from_a_measured_zero() {
    let router = UorR4Router::new(0.5);
    let results = run_ablation_benchmark(&router, &[]);
    for result in &results {
        assert_eq!(result.queries, 0, "no queries were scored");
        assert_eq!(result.recall_at_3, 0.0);
    }
}
