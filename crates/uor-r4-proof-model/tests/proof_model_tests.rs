use uor_r4_core::transformerless::{
    score_q::ScoreQ,
    transitions::{EdgeKind, TransitionGraph},
};
use uor_r4_proof_model::{
    allocation_proof,
    deterministic_topk_proof::{self, Candidate},
    proof_matrix::ProofStatusMatrix,
    range_bounds_proof, theorem7_proof,
};

#[test]
fn test_proof_matrix_all_verified() {
    let matrix = ProofStatusMatrix::new();
    assert!(matrix.verify_all().is_ok());
}

#[test]
fn test_range_bounds_proof_valid_and_invalid() {
    assert!(range_bounds_proof::verify_range_bounds(0, 10, 100, "test_valid").is_ok());
    assert!(range_bounds_proof::verify_range_bounds(95, 10, 100, "test_invalid").is_err());
}

#[test]
fn test_deterministic_topk_canonical_sorting() {
    let mut candidates = vec![
        Candidate {
            token: 20,
            score: ScoreQ::from_raw(100),
        },
        Candidate {
            token: 10,
            score: ScoreQ::from_raw(100),
        },
        Candidate {
            token: 5,
            score: ScoreQ::from_raw(500),
        },
    ];

    deterministic_topk_proof::sort_candidates_canonical(&mut candidates);

    assert_eq!(candidates[0].token, 5); // Highest score 500
    assert_eq!(candidates[1].token, 10); // Tied score 100, lower token 10
    assert_eq!(candidates[2].token, 20); // Tied score 100, higher token 20

    assert!(deterministic_topk_proof::verify_canonical_order(&candidates).is_ok());
}

#[test]
fn test_theorem7_proof_verification() {
    let mut graph = TransitionGraph::new();
    graph.add_edge(1, 2, 10, EdgeKind::Forward);
    graph.add_edge(3, 2, 15, EdgeKind::Forward);
    assert!(graph.build_reverse_index().is_none(), "build reverse index");

    assert!(theorem7_proof::verify_theorem_7_proof(&graph).is_ok());
}

#[test]
fn test_zero_allocation_proof_harness() {
    let res = allocation_proof::verify_zero_allocation(|| {
        // Pure stack computation
        let a = 10u64;
        let b = 20u64;
        a + b
    });

    assert_eq!(res.unwrap(), 30);

    // Heap allocation should be detected by the harness
    let alloc_res = allocation_proof::verify_zero_allocation(|| {
        #[allow(clippy::useless_vec)] // must heap-allocate for the harness to detect
        let v = vec![1u8];
        v.len()
    });
    assert!(alloc_res.is_err());
}
