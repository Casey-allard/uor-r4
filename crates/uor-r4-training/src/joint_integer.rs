//! Compatibility exports for the shared standalone integer model.
//!
//! Training evaluation and serving execute the same numerical implementation.
//! Offline floating training remains in `joint_model`.

pub use uor_r4_integer::{IntegerModel, IntegerSession, IntegerStep, PROBABILITY_TOTAL};
