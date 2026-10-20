//! **`uor_addr::ring` — the ring-element realization of UOR-ADDR**
//! (ARCHITECTURE.md "Format-specific realizations" § `uor-addr-ring`).
//!
//! Ring-element typed-input content-addressing under UOR-Framework
//! Amendment 43 §2's `Element::canonical_bytes` layout, with the
//! σ-projection bound to `prism::crypto::Sha256Hasher`.
//!
//! ## Authoritative sources
//!
//! - **Amendment 43 §2 canonical-bytes layout** — UOR-Framework wiki
//!   <https://github.com/UOR-Foundation/UOR-Framework/wiki/Amendment-43>.
//!   The canonical-bytes layout is `header(k) || le_bytes(x, k+1)`
//!   where `k` is the element's Witt level and `x` is its value
//!   coefficient encoded in little-endian over `k+1` bytes.
//! - **ADR-039 ring algebra surface** — UOR-Framework wiki
//!   <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-039>.
//! - **SHA-256 σ-projection** — NIST FIPS 180-4
//!   (<https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf>).
//!
//! ## Canonical-bytes layout
//!
//! ```text
//! canonical_bytes(e) := [witt_level: u8] || [coefficient: u8; witt_level + 1]
//! ```
//!
//! - `witt_level ∈ {0, 1, 2, 3}` — the element's Witt level per
//!   Amendment 43.
//! - `coefficient` — `witt_level + 1` little-endian bytes encoding the
//!   element value `x`. The coefficient byte width is determined by
//!   the Witt level (1, 2, 3, or 4 bytes), so the canonical bytes are
//!   `1 + (witt_level + 1)` bytes total — between 2 and 5 bytes.
//!
//! This is the minimum-information byte sequence that distinguishes
//! two ring elements: the Witt level disambiguates the algebraic
//! domain; the LE coefficient bytes distinguish elements within the
//! same level.

pub mod model;
pub mod pipeline;
pub mod shapes;
pub mod value;
pub mod verbs;

pub use model::{
    AddressModel, AddressModelBlake3, AddressModelKeccak256, AddressModelSha3_256,
    AddressModelSha512, AddressRoute,
};
pub use pipeline::{
    address, address_blake3, address_keccak256, address_sha3_256, address_sha512, AddressFailure,
    AddressOutcome, AddressWitness, VerifyError,
};
pub use shapes::{MAX_WITT_LEVEL, RING_VALUE_MAX_BYTES};
pub use value::RingElement;
pub use verbs::{address_inference, VERB_TERMS_ADDRESS_INFERENCE};

/// The shared, format-independent ψ-tower (re-exported for convenience;
/// canonical path is [`crate::resolvers::AddressResolverTuple`]).
pub use crate::resolvers::AddressResolverTuple;
