//! Executable proof module: Theorem 7 reverse index structural consistency.

use uor_r4_core::transformerless::transitions::TransitionGraph;

/// Formally verify Theorem 7 reverse index consistency on a TransitionGraph.
/// Total: returns `None` when the theorem holds, or `Some(reason)` when the
/// reverse index is inconsistent (R5 — a failed proof is a measured report).
pub fn verify_theorem_7_proof(graph: &TransitionGraph) -> Option<String> {
    graph
        .verify_theorem_7()
        .map(|reason| format!("Theorem 7 proof failed: {reason}"))
}
