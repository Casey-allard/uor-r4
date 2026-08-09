use uor_r4_core::transformerless::{
    graph_patch::{GraphPatch, RouteMapping, RouteTranslationMap, Theorem11Verifier},
    score_q::ScoreQ,
    transitions::{Edge, EdgeKind, TransitionGraph},
};

#[test]
fn test_graph_patch_application_and_theorem_11() {
    let mut base_graph = TransitionGraph::new();
    base_graph.add_edge_with_score(10, 20, 5, ScoreQ::from_raw(100), EdgeKind::Forward);
    base_graph.add_edge_with_score(30, 20, 8, ScoreQ::from_raw(200), EdgeKind::Forward);
    assert!(
        base_graph.build_reverse_index().is_none(),
        "build base reverse index"
    );

    let new_edge = Edge {
        id: 2,
        src: 10,
        dst: 40,
        weight: 3,
        score: ScoreQ::from_raw(300),
        kind: EdgeKind::Forward,
    };

    let mut route_map = RouteTranslationMap::new();
    route_map.insert(0, RouteMapping::Retained(0));
    route_map.insert(1, RouteMapping::Retained(1));
    route_map.insert(2, RouteMapping::Removed);

    let patch = GraphPatch::new(
        "kappa:blake3:base_graph_123",
        1,
        vec![new_edge],
        vec![(0, ScoreQ::from_raw(100))], // Retain score on edge 0
        vec![],
        route_map.clone(),
    );

    assert!(patch.verify_cid());

    let mut patched_graph = base_graph.clone();
    assert!(patch.apply(&mut patched_graph).is_none());

    assert_eq!(patched_graph.edges.len(), 3);
    assert_eq!(patched_graph.edges[2].dst, 40);

    // Verify Theorem 11
    assert!(
        Theorem11Verifier::verify_theorem_11(&base_graph, &patched_graph, &route_map).is_none()
    );
}

#[test]
fn test_route_translation_mapping() {
    let mut route_map = RouteTranslationMap::new();
    route_map.insert(1, RouteMapping::Retained(10));
    route_map.insert(2, RouteMapping::Split(vec![20, 21]));
    route_map.insert(3, RouteMapping::Merged(30));
    route_map.insert(4, RouteMapping::Removed);

    assert_eq!(route_map.translate_route(1), Some(vec![10]));
    assert_eq!(route_map.translate_route(2), Some(vec![20, 21]));
    assert_eq!(route_map.translate_route(3), Some(vec![30]));
    assert_eq!(route_map.translate_route(4), None);
}

#[test]
fn test_graph_patch_cbor_roundtrip() {
    let patch = GraphPatch::new(
        "kappa:blake3:parent",
        42,
        vec![],
        vec![(0, ScoreQ::from_raw(500))],
        vec![],
        RouteTranslationMap::new(),
    );

    let cbor_bytes = patch.to_cbor_bytes().expect("serialize CBOR");
    let decoded = GraphPatch::from_cbor_bytes(&cbor_bytes).expect("deserialize CBOR");

    assert_eq!(patch, decoded);
    assert!(decoded.verify_cid());
}
