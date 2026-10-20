//! Native support surface of the proof model (legacy-crate repair, 19 Oct 2026).
//!
//! The historical `lib.rs` also registers `deterministic_topk_proof` and
//! `theorem7_proof` (built on core's retired `transformerless::{score_q,
//! transitions}`) and `structural_guarantees` (its #161 obligation needs the
//! graph certifier's `performance_certificate`, which uses the retired
//! `transformerless::runtime`). Those modules are preserved in place as
//! uncompiled evidence; the R4G1 mechanism stays PARKED (D22). This surface
//! compiles only the modules that build on the current core. No Cargo feature
//! re-enables the historical surface.

#[cfg(feature = "full-model")]
pub mod allocation_proof;
pub mod inference_audit;
pub mod kani_proofs;
#[cfg(feature = "full-model")]
pub mod pdf_traceability;
#[cfg(feature = "full-model")]
pub mod proof_matrix;
#[cfg(feature = "full-model")]
pub mod range_bounds_proof;
