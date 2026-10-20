//! Native support surface of the graph CLI (legacy-crate repair, 19 Oct 2026).
//!
//! The historical `lib.rs` dispatches the R4G1 compile/observe/score/cover
//! commands over the transformer teacher (`uor-r4-model-source`:
//! normalized-exponential attention, dense matrix products) and the retired
//! `transformerless::{compiler, runtime}` (normalized-exponential teacher
//! sampling, dense dot-argmax assignment). Those paths
//! are banned in every compiled module (D23/D25/D29/D30) and stay preserved in
//! place as uncompiled evidence; the R4G1 mechanism stays PARKED (D22). This
//! surface compiles only the modules that need none of them. No Cargo feature
//! re-enables the historical surface.

// The two kept modules name their error type as `uor_r4_model_source::SourceUnavailable`,
// which that crate re-exports unchanged from `uor-r4-source-interface`. Binding the
// interface crate under that name gives them the identical type without linking the
// teacher crate.
extern crate uor_r4_source_interface as uor_r4_model_source;

#[cfg(not(target_arch = "wasm32"))]
pub mod prime_route_canary;
pub mod recommend_scale;
