//! **`uor_addr::cbor` — the CBOR realization of UOR-ADDR**
//! (ARCHITECTURE.md "Format-specific realizations").
//!
//! CBOR typed-input content-addressing under RFC 8949 §4.2 Deterministic
//! Encoding, with the σ-projection bound to `prism::crypto::Sha256Hasher`
//! by default and the other 32-byte axes ([`crate::hash`]) available via
//! the `address_<algorithm>` entry points.
//!
//! ## Authoritative sources
//!
//! - **CBOR + deterministic encoding** — IETF RFC 8949 *Concise Binary
//!   Object Representation (CBOR)*, §4.2 *Deterministically Encoded CBOR*
//!   (<https://www.rfc-editor.org/rfc/rfc8949>). The canonical-form
//!   invariants — preferred (shortest) integer/float encoding (§4.1,
//!   §4.2.2), definite-length items, and bytewise-sorted map keys (§4.2.1)
//!   — are validated against RFC 8949 Appendix A's diagnostic ⇄ encoding
//!   vectors in `tests/cbor_rfc8949.rs`.
//! - **SHA-256 σ-projection** — NIST FIPS 180-4.
//!
//! ## End-to-end (ADR-060)
//!
//! 1. [`canonicalize`] re-encodes any well-formed CBOR item into its RFC
//!    8949 §4.2 deterministic form into an `alloc` buffer.
//! 2. [`address`] wraps those bytes in the borrowed [`CborCarrier`] and
//!    runs [`AddressModel`]'s `forward()`: the ψ-chain verb threads the
//!    carrier through the shared [`AddressResolverTuple`] (ADR-036), and ψ₉
//!    folds the canonical bytes through the σ-axis in one σ-projection.
//! 3. [`address`] returns the [`crate::AddressOutcome`] carrying the
//!    κ-label + replayable TC-05 witness.

pub mod model;
pub mod pipeline;
pub mod shapes;
pub mod value;
pub mod verbs;

pub use model::{
    AddressModel, AddressModelBlake3, AddressModelKeccak256, AddressModelSha3_256,
    AddressModelSha512, AddressRoute,
};
#[cfg(feature = "alloc")]
pub use pipeline::{address, address_blake3, address_keccak256, address_sha3_256, address_sha512};
pub use pipeline::{AddressFailure, AddressOutcome, AddressWitness, VerifyError};
pub use shapes::{Sha256Hasher, MAX_CBOR_DEPTH};
#[cfg(feature = "alloc")]
pub use value::canonicalize;
pub use value::CborCarrier;
pub use verbs::{address_inference, VERB_TERMS_ADDRESS_INFERENCE};

/// The shared `AddrBounds` capacity profile (canonical path
/// [`crate::bounds::AddrBounds`]).
pub use crate::bounds::AddrBounds;
/// The shared, format-independent ψ-tower (canonical path
/// [`crate::resolvers::AddressResolverTuple`]).
pub use crate::resolvers::AddressResolverTuple;
