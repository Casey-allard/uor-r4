//! Native support surface of the typed API façade (legacy-crate repair, 19 Oct 2026).
//!
//! The historical `lib.rs` wraps the R4G1 compile pipeline and the deployed
//! `R4Engine` (retired `transformerless::{compiler, runtime}`:
//! normalized-exponential teacher sampling, dense dot-argmax assignment;
//! `uor-r4-model-source`: normalized-exponential attention, dense matrix
//! products) and the pre-#2345 native geometric model API
//! (`native_geometric::{Model, durable_memory, vsa, learner::binary_model}`),
//! which the current core no longer compiles. Every API module depends on one
//! of those, so all are preserved in place as uncompiled evidence; the R4G1
//! mechanism stays PARKED (D22). This surface re-exports the current core's
//! qualified native geometric support. No Cargo feature re-enables the
//! historical surface; `learned-reference` still stops the build in core.

/// The current core's qualified native geometric support: raw corpus, fixed
/// geometry and exact storage. It is not the historical model API.
pub use uor_r4_core::native_geometric;
