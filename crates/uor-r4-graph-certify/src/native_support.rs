//! Native support surface of the graph certifier (legacy-crate repair, 19 Oct 2026).
//!
//! The historical `lib.rs` scores and certifies R4G1 graphs against a
//! transformer teacher: `score`, `score_runtime`, `fmm`, `certify` and the
//! measurement harnesses use the retired `transformerless::{compiler, runtime}`
//! (normalized-exponential teacher sampling, dense dot-argmax assignment) or link
//! `uor-r4-model-source` (normalized-exponential attention, dense matrix
//! products). Those paths are banned
//! in every compiled module (D23/D25/D29/D30) and stay preserved in place as
//! uncompiled evidence; the R4G1 mechanism stays PARKED (D22). This surface
//! compiles only the modules that need none of them. No Cargo feature
//! re-enables the historical surface.

#[cfg(feature = "anti-degeneracy")]
pub mod anti_degeneracy;
pub mod certificate;
#[cfg(not(target_arch = "wasm32"))]
pub mod compiler_scaling;
#[cfg(not(target_arch = "wasm32"))]
pub mod corpus_signed_transport_attention;
pub mod frame_consistency;
#[cfg(feature = "holographic-encoding")]
pub mod holographic_encoding;
pub mod long_context;
pub mod octeract;
#[cfg(feature = "patch-lifecycle")]
pub mod patch_lifecycle;
#[cfg(not(target_arch = "wasm32"))]
pub mod prime_route_worker_canary;
pub mod route_attention;
#[cfg(feature = "shortlist-evaluator")]
pub mod shortlist_evaluator;
