//! **`uor_addr::codemodule` — the code-module AST realization of
//! UOR-ADDR** (ARCHITECTURE.md "Format-specific realizations" §
//! `uor-addr-codemodule`).
//!
//! Code-module AST typed-input content-addressing under the
//! Canonical Code-Module AST Serialization (CCMAS) — a
//! S-expression-shaped canonical AST grammar pinned by this crate
//! and serialized through Rivest 1997's canonical S-expression form.
//!
//! ## Authoritative sources
//!
//! - **Canonical Code-Module AST Serialization (CCMAS)** — pinned
//!   inline below; the grammar's normative source is this module
//!   plus the conformance fixtures under `crate::codemodule::value`'s
//!   `#[cfg(test)]` mod.
//! - **Canonical S-expressions** — Rivest 1997 *S-expressions*
//!   (<https://people.csail.mit.edu/rivest/Sexp.txt>) supplies the
//!   byte-output discipline CCMAS extends with AST-shaped term
//!   constructors.
//! - **SHA-256 σ-projection** — NIST FIPS 180-4
//!   (<https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf>).
//!
//! ## CCMAS grammar
//!
//! The Canonical Code-Module AST Serialization (CCMAS) is a
//! S-expression-shaped grammar over five AST cases:
//!
//! ```text
//! CodeModuleValue ::= Module(name, items)
//!                   | Function(name, parameters, return_type, body)
//!                   | TypeDeclaration(name, fields)
//!                   | ConstDeclaration(name, type, value)
//!                   | Expression(literal | variable | call)
//! ```
//!
//! Each AST node serializes to a structurally-tagged byte form
//! whose canonical-form output is a Rivest canonical S-expression
//! shaped as:
//!
//! ```text
//! Module(name, [items])         → (3:mod  <n:name>  <item_1> <item_2> ... <item_k>)
//! Function(name, params, ret, body)
//!                                → (3:fun  <n:name>  (<param_1> ... <param_p>)  <ret_type>  <body_expr>)
//! TypeDeclaration(name, fields) → (3:type <n:name>  (<field_1> ... <field_f>))
//! ConstDeclaration(name, ty, v) → (3:const <n:name> <ty> <v>)
//! Expression                    → format-specific case (atom for literal, atom for variable, (3:call ...) for call)
//! ```
//!
//! The CCMAS canonical form is byte-output-equivalent to
//! [`crate::sexp::canonicalize`] applied to the tagged surface AST —
//! the canonicalization composes Rivest §4.2/§4.3 over the AST's
//! grammar cases.

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
pub use value::CodeModuleCarrier;
#[cfg(feature = "alloc")]
pub use value::{canonicalize, CodeModuleValue};
pub use verbs::{address_inference, VERB_TERMS_ADDRESS_INFERENCE};

/// The shared, format-independent ψ-tower (re-exported for convenience;
/// canonical path is [`crate::resolvers::AddressResolverTuple`]).
pub use crate::resolvers::AddressResolverTuple;
