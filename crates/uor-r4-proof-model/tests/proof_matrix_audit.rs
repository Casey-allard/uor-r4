//! CI Audit Test for Proof Status Matrix Verification
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §13;
//! `docs/formal_vocabulary.md` §7; GitHub Issue #132.

use uor_r4_graph_certify::target_operator_certificate::route_attention_obligation_links;
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

/// #606: the target-operator certificate links proof obligations by
/// theorem id + recorded status token. This crate depends on
/// `uor-r4-graph-certify`, not the other way around, so the certify-side
/// link table is a MIRROR of this crate's matrix — pinned here, where
/// both sides are importable: every linked (theorem_id, name) must be a
/// matrix entry and every recorded status token must equal the matrix's
/// recorded status. A drift in either direction fails this test rather
/// than silently misreporting an obligation.
#[test]
fn test_606_certificate_obligation_links_mirror_the_matrix() {
    fn status_token(status: ProofStatus) -> &'static str {
        match status {
            ProofStatus::Verified => "Verified",
            ProofStatus::ExecutableSpec => "ExecutableSpec",
            ProofStatus::DifferentialPass => "DifferentialPass",
            ProofStatus::Unverified => "Unverified",
        }
    }
    let matrix = ProofStatusMatrix::default();
    let links = route_attention_obligation_links();
    assert!(!links.is_empty(), "the certificate links obligations");
    for link in &links {
        let entry = matrix
            .entries
            .iter()
            .find(|entry| entry.theorem_id == link.theorem_id && entry.name == link.name)
            .unwrap_or_else(|| {
                panic!(
                    "linked obligation ({}, {}) is not a proof-matrix entry",
                    link.theorem_id, link.name
                )
            });
        assert_eq!(
            link.status,
            status_token(entry.status),
            "recorded status token for {} must mirror the matrix",
            link.name
        );
    }
}
