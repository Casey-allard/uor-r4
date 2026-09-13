//! Experimental addressed attention with a complete eight-phase runtime and bound export.
//!
//! Runtime geometry, circuits and objects use integer/table operations. `training`,
//! `offline`, and parameter/sampled policies are offline floating-point boundaries.
//! Corpus fitting and retained-model dispatch remain unimplemented.
pub mod circuit;
pub mod inputs;
pub mod objects;
pub mod training;

pub mod artifact;
pub mod engine;
pub mod offline;
pub mod policy;

pub mod learner;
pub mod pilot;
pub mod pilot_data;
