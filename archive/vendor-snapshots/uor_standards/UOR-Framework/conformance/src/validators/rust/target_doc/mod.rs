//! Structural cross-reference validators.
//!
//! The correctness suite already pins behavioral contracts per endpoint.
//! These validators add a second layer: they cross-check the foundation
//! source against structural commitments maintained as static snapshots
//! in each submodule (sealed-type table, resolver signature shape,
//! closed enumerations, trait-shape invariants).
//!
//! Together the two layers give the suite self-enforcement: behavioral
//! regressions fail a `correctness/*` validator; structural deviations
//! fail a `rust/target_doc/*` validator.

pub mod constraint_encoder_completeness;
pub mod resolver_signature_shape;
pub mod sealed_type_coverage;
pub mod spectral_sequence_walk;
pub mod w4_grounding_closure;
