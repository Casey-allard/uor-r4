//! Executable proof module: Theorem 7 reverse index structural consistency.

use uor_r4_core::transformerless::transitions::TransitionGraph;

/// Formally verify Theorem 7 reverse index consistency on a TransitionGraph.
pub fn verify_theorem_7_proof(graph: &TransitionGraph) -> Result<(), String> {
    graph.verify_theorem_7().map_or(Ok(()), |reason| {
        Err(format!("Theorem 7 proof failed: {reason}"))
    })
}
