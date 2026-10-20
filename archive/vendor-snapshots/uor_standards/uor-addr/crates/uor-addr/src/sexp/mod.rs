//! **`uor_addr::sexp` — the S-expression realization of UOR-ADDR**
//! (ARCHITECTURE.md "Format-specific realizations" § `uor-addr-sexp`).
//!
//! S-expression typed-input content-addressing under Rivest's canonical
//! S-expression form, with the σ-projection bound to
//! `prism::crypto::Sha256Hasher`.
//!
//! ## Authoritative sources
//!
//! - **Canonical S-expressions** — Ronald L. Rivest, *S-expressions*,
//!   May 4 1997 draft, archived at
//!   <https://people.csail.mit.edu/rivest/Sexp.txt>. I-D form at
//!   <https://datatracker.ietf.org/doc/html/draft-rivest-sexp-00>.
//! - **SPKI canonical form citation** — IETF RFC 2693 §3
//!   *SPKI Certificate Theory*
//!   (<https://datatracker.ietf.org/doc/html/rfc2693#section-3>).
//! - **SHA-256 σ-projection** — NIST FIPS 180-4
//!   (<https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf>).
//!
//! ## Grammar
//!
//! ```text
//! SExprValue ::= Atom(bytes)            — symbolic atoms (UTF-8 bytes)
//!              | Cons(SExprValue, SExprValue)
//!              | Nil
//! ```
//!
//! The wire-format input is canonical S-expression syntax — atoms as
//! `<n>:<bytes>` length-prefixed byte sequences (Rivest's canonical
//! form), lists as parenthesized sequences, nil as `()`. The parser also
//! admits the Lisp-style sugared form `(a b c)`.
//!
//! ## Canonicalization (ADR-060 streaming)
//!
//! The realization no longer materializes a structurally-tagged byte
//! form. [`SExprCanon`] is a [`prism::uor_foundation::pipeline::ChunkSource`] over the
//! borrowed input that emits Rivest canonical bytes on demand — atoms as
//! `<n>:<bytes>`, lists as `(s₁ s₂ … sₙ)`, nil as `()` — and ψ₉ folds
//! those chunks through the σ-axis. There is no input size, atom-width,
//! element-count, or nesting-depth ceiling.

pub mod model;
pub mod pipeline;
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
#[cfg(feature = "alloc")]
pub use value::canonicalize;
pub use value::{SExprCanon, SExprValue};
pub use verbs::{address_inference, VERB_TERMS_ADDRESS_INFERENCE};

/// The shared, format-independent ψ-tower (re-exported for convenience;
/// canonical path is [`crate::resolvers::AddressResolverTuple`]).
pub use crate::resolvers::AddressResolverTuple;
