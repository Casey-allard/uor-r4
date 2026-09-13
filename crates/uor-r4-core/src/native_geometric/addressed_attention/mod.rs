//! Experimental addressed-attention primitives; not a complete model or retained dispatch.
//!
//! The circuit/export and object kernels are deterministic integer/table operations.
//! `training` is an explicitly offline floating-point boundary. The eight-phase
//! learned model, normal artifact envelope and corpus learner remain unimplemented.
pub mod circuit;
pub mod inputs;
pub mod objects;
pub mod training;
