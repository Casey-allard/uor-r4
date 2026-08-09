use uor_r4_core::transformerless::compiler::Corpus;
use uor_r4_core::transformerless::transitions::{
    compile_transitions_from_corpus, EdgeKind, TransitionGraph,
};

#[test]
fn test_transition_graph_manual_edges_and_theorem_7() {
    let mut graph = TransitionGraph::new();
    let e0 = graph.add_edge(10, 20, 5, EdgeKind::Forward);
    let e1 = graph.add_edge(30, 20, 8, EdgeKind::Forward);
    let e2 = graph.add_edge(10, 40, 3, EdgeKind::Forward);

    assert_eq!(e0, 0);
    assert_eq!(e1, 1);
    assert_eq!(e2, 2);

    assert!(graph.build_reverse_index().is_none(), "build reverse index");

    // Theorem 7 verification
    assert!(graph.verify_theorem_7().is_none());

    // Verify reverse index for dst = 20
    let &(start, count) = graph
        .reverse_offsets
        .get(&20)
        .expect("dst 20 in reverse map");
    assert_eq!(count, 2);
    let rev_slice = &graph.reverse_index[start..start + count];
    for &edge_id in rev_slice {
        assert_eq!(graph.edges[edge_id as usize].dst, 20);
    }
}

#[test]
fn test_compile_transitions_from_synthetic_corpus() {
    let corpus = Corpus {
        n: 6,
        stories: 1,
        story: vec![1, 1, 1, 1, 1, 1],
        input: vec![100, 200, 100, 200, 100, 300],
        next: vec![200, 100, 200, 100, 300, 400],
        t_argmax: vec![200, 100, 200, 100, 300, 400],
        top_tokens: vec![[200, 0, 0, 0, 0, 0, 0, 0]; 6],
        top_weights: vec![[100, 0, 0, 0, 0, 0, 0, 0]; 6],
        span_start: vec![0, 1, 2, 3, 4, 5],
        span_end: vec![1, 2, 3, 4, 5, 6],
        byte_start: vec![u32::MAX; 6],
        byte_end: vec![u32::MAX; 6],
        hidden: None,
    };

    // Simple region assigner mapping token_id -> region_id
    let region_assigner = |tok: u32| tok / 10; // 100->10, 200->20, 300->30, 400->40

    let graph =
        compile_transitions_from_corpus(&corpus, region_assigner, 10).expect("compile transitions");

    // Verify Theorem 7
    assert!(graph.verify_theorem_7().is_none());

    // Check transition counts
    // 100 -> 200 appears twice => region 10 -> region 20 weight 2
    // 200 -> 100 appears twice => region 20 -> region 10 weight 2
    // 100 -> 300 appears once => region 10 -> region 30 weight 1
    // 300 -> 400 appears once => region 30 -> region 40 weight 1

    let edges_from_10 = graph.forward_map.get(&10).expect("edges from 10");
    assert_eq!(edges_from_10.len(), 2);
    let edge_10_20 = &graph.edges[edges_from_10[0] as usize];
    assert_eq!(edge_10_20.src, 10);
    assert_eq!(edge_10_20.dst, 20);
    assert_eq!(edge_10_20.weight, 2);
}

#[test]
fn test_bounded_transitions_per_node() {
    let corpus = Corpus {
        n: 4,
        stories: 1,
        story: vec![1, 1, 1, 1],
        input: vec![10, 10, 10, 10],
        next: vec![20, 30, 40, 50],
        t_argmax: vec![20, 30, 40, 50],
        top_tokens: vec![[0; 8]; 4],
        top_weights: vec![[0; 8]; 4],
        span_start: vec![0, 1, 2, 3],
        span_end: vec![1, 2, 3, 4],
        byte_start: vec![u32::MAX; 4],
        byte_end: vec![u32::MAX; 4],
        hidden: None,
    };

    let region_assigner = |tok: u32| tok;
    let max_transitions = 2;

    let graph = compile_transitions_from_corpus(&corpus, region_assigner, max_transitions)
        .expect("compile bounded transitions");

    let edges_from_10 = graph.forward_map.get(&10).expect("edges from 10");
    assert_eq!(
        edges_from_10.len(),
        2,
        "bounded to max 2 transitions per node"
    );
    assert!(graph.verify_theorem_7().is_none());
}
