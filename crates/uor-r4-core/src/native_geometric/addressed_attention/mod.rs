//! Experimental addressed attention with a complete eight-phase runtime and bound export.
//!
//! Runtime geometry, circuits and objects use integer/table operations. `training`,
//! `offline`, and parameter/sampled policies are offline floating-point boundaries.
//! Bounded pilot training exists; broad language qualification and retained-model
//! dispatch remain unfinished.
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

pub mod stability;
#[cfg(test)]
mod stability_report;
pub mod stability_stats;

pub mod causal_credit;
#[cfg(test)]
mod causal_report;
