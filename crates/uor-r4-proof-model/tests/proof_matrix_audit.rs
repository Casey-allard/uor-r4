//! CI Audit Test for Proof Status Matrix Verification
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §13;
//! `docs/formal_vocabulary.md` §7; GitHub Issue #132.

use uor_r4_proof_model::proof_matrix::{ProofStatus, ProofStatusMatrix};
use uor_r4_proof_model::structural_guarantees::StructuralGuaranteeVerifier;

#[test]
fn test_ci_audit_proof_matrix_entries() {
    let matrix = ProofStatusMatrix::default();

    // Audit default theorem entries against expected status
    let report_alloc = StructuralGuaranteeVerifier::audit_proof_matrix_entry(
        &matrix,
        "Allocation Freedom",
        ProofStatus::Verified,
    );
    assert!(report_alloc.verified);

    let report_bounded = StructuralGuaranteeVerifier::audit_proof_matrix_entry(
        &matrix,
        "Bounded Ranges",
        ProofStatus::Verified,
    );
    assert!(report_bounded.verified);

    let report_topk = StructuralGuaranteeVerifier::audit_proof_matrix_entry(
        &matrix,
        "Deterministic Top-K",
        ProofStatus::Verified,
    );
    assert!(report_topk.verified);

    let report_rev = StructuralGuaranteeVerifier::audit_proof_matrix_entry(
        &matrix,
        "Reverse Index Consistency",
        ProofStatus::Verified,
    );
    assert!(report_rev.verified);

    let report_ops = StructuralGuaranteeVerifier::audit_proof_matrix_entry(
        &matrix,
        "Operation-Set Conformance",
        ProofStatus::Verified,
    );
    assert!(report_ops.verified);
}
