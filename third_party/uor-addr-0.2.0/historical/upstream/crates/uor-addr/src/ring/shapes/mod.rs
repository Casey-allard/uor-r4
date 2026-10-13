//! Substitution-axis selections for the ring-element realization.

pub mod bounds;

pub use bounds::{MAX_WITT_LEVEL, RING_VALUE_MAX_BYTES};
/// Canonical `Hasher<32>` selection. Re-exported from the Prism standard
/// library; see wiki ADR-031 / ADR-047.
pub use prism::crypto::Sha256Hasher;
