//! **`uor_addr::asn1` — the ASN.1 realization of UOR-ADDR**
//! (ARCHITECTURE.md "Format-specific realizations" § `uor-addr-asn1`).
//!
//! ASN.1 typed-input content-addressing under ITU-T X.690
//! Distinguished Encoding Rules (DER), with the σ-projection bound
//! to `prism::crypto::Sha256Hasher`.
//!
//! ## Authoritative sources
//!
//! - **ITU-T X.690** — *Information technology — ASN.1 encoding
//!   rules: Specification of Basic Encoding Rules (BER), Canonical
//!   Encoding Rules (CER), and Distinguished Encoding Rules (DER)*
//!   (<https://www.itu.int/rec/T-REC-X.690>). DER (§§ 10–11) is
//!   the canonical-form discipline for the typed-iso surface.
//! - **ITU-T X.680** — *Specification of basic notation*
//!   (<https://www.itu.int/rec/T-REC-X.680>). Defines the ASN.1
//!   abstract type system.
//! - **SHA-256 σ-projection** — NIST FIPS 180-4
//!   (<https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf>).
//!
//! ## Supported universal tags
//!
//! Per ARCHITECTURE.md the typed-input shape `Asn1Value` is a
//! `partition_coproduct!` over ASN.1 universal-tag cases. The current
//! published support covers the cases that have unambiguous DER
//! encoding in X.690 §8:
//!
//! - `Boolean` (tag 0x01) — DER encodes `false` as a single byte
//!   `0x00`, `true` as a single byte `0xFF` (X.690 §8.2.2).
//! - `Integer` (tag 0x02) — minimum-octets two's-complement
//!   big-endian (X.690 §8.3).
//! - `OctetString` (tag 0x04) — primitive encoding (X.690 §10.2).
//! - `Null` (tag 0x05) — zero-length content (X.690 §8.8.1).
//! - `Sequence` (tag 0x30) — children DER-encoded in declared order
//!   (X.690 §8.9). Used for both `SEQUENCE` and `SEQUENCE OF`.
//!
//! ## Canonical-bytes layout (DER)
//!
//! Every `Asn1Value::tagged_bytes()` returns a self-describing
//! DER-encoded byte sequence; the ψ_9 canonicalizer is the identity
//! on this layout because DER is itself the canonical form.

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
pub use shapes::MAX_ASN1_DEPTH;
#[cfg(feature = "alloc")]
pub use value::{canonicalize, Asn1Value};
pub use value::{validate_der, Asn1Carrier};
pub use verbs::{address_inference, VERB_TERMS_ADDRESS_INFERENCE};

/// The shared, format-independent ψ-tower (re-exported for convenience;
/// canonical path is [`crate::resolvers::AddressResolverTuple`]).
pub use crate::resolvers::AddressResolverTuple;
