//! uor-r4-proof-model: Executable proof specification and formal verification harness.

#[cfg(feature = "full-model")]
pub mod allocation_proof;
#[cfg(feature = "full-model")]
pub mod deterministic_topk_proof;
#[cfg(feature = "full-model")]
pub mod inference_audit;
pub mod kani_proofs;
#[cfg(feature = "full-model")]
pub mod pdf_traceability;
#[cfg(feature = "full-model")]
pub mod proof_matrix;
#[cfg(feature = "full-model")]
pub mod range_bounds_proof;
#[cfg(feature = "full-model")]
pub mod structural_guarantees;
#[cfg(feature = "full-model")]
pub mod theorem7_proof;
