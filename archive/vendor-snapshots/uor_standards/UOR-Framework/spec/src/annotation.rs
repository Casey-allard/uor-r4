//! Root-level annotation properties (Amendment 8).
//!
//! The `uor:space` annotation property classifies every namespace module
//! as `kernel`, `user`, or `bridge`. This enables implementers to determine
//! the compilation target and API surface from the ontology alone.

// Re-exported from model; this module is kept for documentation clarity.
pub use crate::model::annotation_space_property;
