//! Ring-element grammar constants (UOR-Framework Amendment 43 §2).
//!
//! The `HostBounds` capacity profile is the shared
//! [`crate::bounds::AddrBounds`]; only the format-specific grammar
//! constants live here.

/// Maximum Witt level admissible per Amendment 43 §2's tower (0..=3,
/// inclusive). The Witt-level byte at canonical-bytes offset 0 must
/// satisfy `witt_level ≤ MAX_WITT_LEVEL`.
pub const MAX_WITT_LEVEL: u8 = 3;

/// Maximum total byte width of a `RingElement`'s structurally-tagged
/// serialization (Witt-level byte + up to four little-endian coefficient
/// bytes — at most 5; the generous ceiling guards the fixed stack
/// buffer).
pub const RING_VALUE_MAX_BYTES: usize = 3968;
