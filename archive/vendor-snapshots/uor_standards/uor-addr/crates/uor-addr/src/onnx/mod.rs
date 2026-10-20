//! **`uor_addr::onnx` — the ONNX realization of UOR-ADDR.**
//!
//! Typed content-addressing for ONNX `ModelProto` files under a
//! protobuf-canonical structural form. The default σ-projection is
//! [`prism::crypto::Sha256Hasher`]; [`address_blake3`], [`address_sha3_256`],
//! [`address_keccak256`], and [`address_sha512`] select the other axes
//! ([`crate::hash`]).
//! As with GGUF, the skeleton's leaf commitments are SHA-256 by
//! canonical-form definition and the selected κ-axis `H` is applied on top
//! (κ = `H(skeleton)`); the sha256 κ-labels are byte-identical to prior
//! releases. Admits any known IR revision
//! (`ir_version` in `1..=`[`ONNX_IR_VERSION_MAX`]`
//! = 13`, the latest in `onnx.proto`); the `ir_version` is bound into the
//! canonical form, so distinct revisions of the same logical model
//! address distinctly.
//!
//! ## Authoritative sources
//!
//! - ONNX protobuf schema — <https://github.com/onnx/onnx/blob/main/onnx/onnx.proto>
//! - ONNX IR specification — <https://github.com/onnx/onnx/blob/main/docs/IR.md>
//! - ONNX versioning — <https://github.com/onnx/onnx/blob/main/docs/Versioning.md>
//! - Protobuf v3 wire format — <https://protobuf.dev/programming-guides/encoding/>
//! - SHA-256 σ-projection — NIST FIPS 180-4.
//!
//! ## Canonical form
//!
//! The ONNX spec defines no canonical form (protobuf v3 admits many
//! byte-encodings of the same logical message); this realization defines
//! one (canonical form v2 — [`CANONICAL_FORM_VERSION`]). It is a **flat
//! skeleton**: the `ModelProto` structure emitted inline (opset imports
//! sorted by `(domain, version)`, nodes in Kahn-topological order with
//! lexicographic `(name, op_type, domain)` tie-break, name-sorted
//! initializers / IO, typed-data-to-`raw_data` reduction, depth-bounded
//! subgraph recursion emitted inline), in which every variable-length
//! leaf — tensor data, strings, opaque sub-message payloads — is
//! represented by its 32-byte SHA-256 digest, so the skeleton's size
//! grows with structure, not model size, while still binding every weight
//! byte into the κ-label. See [`value`] for the full byte layout.
//!
//! Under ADR-060 the **full skeleton** flows through the pipeline as a
//! [`TermValue::Borrowed`](prism::operation::TermValue) carrier and ψ₉
//! folds it through `H = Sha256Hasher` — there is no two-level commitment
//! and no count / width cap. Two ONNX models that decode to the same
//! logical content canonicalize to byte-identical skeletons and therefore
//! to the same κ-label.

pub mod dtype;
pub mod model;
pub mod pipeline;
pub mod protobuf;
pub mod shapes;
pub mod value;
pub mod verbs;

/// Canonical-form version (see module docs). Bumped to 2 under ADR-060:
/// the canonical form is now the full flat skeleton (no two-level
/// commitment), so v2 κ-labels differ from the v1 commitment's.
pub const CANONICAL_FORM_VERSION: u32 = 2;

pub use dtype::OnnxDataType;
pub use model::{
    AddressModel, AddressModelBlake3, AddressModelKeccak256, AddressModelSha3_256,
    AddressModelSha512, AddressRoute,
};
#[cfg(feature = "alloc")]
pub use pipeline::{address, address_blake3, address_keccak256, address_sha3_256, address_sha512};
pub use pipeline::{AddressFailure, AddressOutcome, AddressWitness, VerifyError};
pub use shapes::bounds::{ONNX_IR_VERSION_MAX, ONNX_OPSET_VERSION_MIN, ONNX_SUBGRAPH_DEPTH_MAX};
pub use value::OnnxCarrier;
#[cfg(feature = "alloc")]
pub use value::{canonicalize, OnnxValue};
pub use verbs::{address_inference, VERB_TERMS_ADDRESS_INFERENCE};

/// The shared, format-independent ψ-tower (re-exported for convenience;
/// canonical path is [`crate::resolvers::AddressResolverTuple`]).
pub use crate::resolvers::AddressResolverTuple;
