//! **`uor_addr::xml` — the XML realization of UOR-ADDR**
//! (ARCHITECTURE.md "Format-specific realizations" § `uor-addr-xml`).
//!
//! XML typed-input content-addressing under a **subset** of
//! W3C Canonical XML 1.1 (XML-C14N 1.1), with the σ-projection bound
//! to `prism::crypto::Sha256Hasher`.
//!
//! ## Authoritative sources
//!
//! - **W3C Canonical XML Version 1.1** — Recommendation 2 May 2008
//!   (<https://www.w3.org/TR/xml-c14n11/>).
//! - **W3C XML 1.0 (Fifth Edition)** — base syntax
//!   (<https://www.w3.org/TR/xml/>).
//! - **SHA-256 σ-projection** — NIST FIPS 180-4
//!   (<https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf>).
//!
//! ## Supported canonical-XML subset
//!
//! Per ARCHITECTURE.md the typed-input shape `XmlValue` is a
//! `partition_coproduct!` over five XML grammar cases (Element,
//! Attribute, Text, CDATA, ProcessingInstruction). The shipped
//! conformance subset implements the rules in XML-C14N 1.1 §1.1 that
//! apply to structurally-typed XML — without external entities,
//! DTDs, or namespace-prefix manipulation, which are out of scope
//! for typed content-addressing:
//!
//! 1. Elements are emitted as `<name>...</name>` with no namespace
//!    prefixes (the typed input shape does not carry namespace
//!    qualification).
//! 2. Attributes are emitted in **lexicographic byte order** by
//!    attribute name per XML-C14N 1.1 §1.1 rule 3.
//! 3. Attribute values are double-quoted; the value bytes are
//!    character-escaped per XML-C14N 1.1 §1.1 rule 4 (`<` → `&lt;`,
//!    `>` → `&gt;`, `&` → `&amp;`, `"` → `&quot;`, `\t` → `&#x9;`,
//!    `\n` → `&#xA;`, `\r` → `&#xD;`).
//! 4. Text content is character-escaped per the C14N 1.1 §1.1 rule 5
//!    (`<` → `&lt;`, `>` → `&gt;`, `&` → `&amp;`, `\r` → `&#xD;`).
//! 5. CDATA sections are expanded to text content before
//!    canonicalization (XML-C14N 1.1 §1.1 — CDATA is informationally
//!    equivalent to escaped text).
//! 6. Processing instructions are emitted as `<?target data?>`.
//!
//! ## Out-of-scope rules (documented deviations)
//!
//! The shipped canonicalizer does **not** handle: namespace prefix
//! rewriting (we don't admit namespaced input), DTD-internal entity
//! resolution (we don't admit DTDs), `xml:` reserved attributes
//! beyond the structural typing the parser enforces, and document
//! whitespace outside element content (we admit only element
//! content). These rules apply to deserialization from arbitrary XML
//! 1.0 documents — out of scope for the typed-input pipeline.
//! Conformance tests pin the supported subset; the deviation is
//! documented in [STANDARDS.md](https://github.com/UOR-Foundation/uor-addr/blob/main/STANDARDS.md).

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
pub use shapes::MAX_XML_DEPTH;
#[cfg(feature = "alloc")]
pub use value::canonicalize;
pub use value::XmlValue;
pub use verbs::{address_inference, VERB_TERMS_ADDRESS_INFERENCE};

/// The shared, format-independent ψ-tower (re-exported for convenience;
/// canonical path is [`crate::resolvers::AddressResolverTuple`]).
pub use crate::resolvers::AddressResolverTuple;
