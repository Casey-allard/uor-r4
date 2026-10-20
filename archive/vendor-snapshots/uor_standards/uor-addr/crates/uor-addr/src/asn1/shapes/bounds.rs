//! ASN.1 realization grammar constant.
//!
//! ADR-060 removed the fixed-buffer capacity profile (`Asn1AddrBounds`)
//! and its byte/element ceilings (`ASN1_VALUE_MAX_BYTES`,
//! `MAX_ASN1_ELEMENTS`): DER is canonical by construction, so the input
//! bytes flow through the pipeline as a borrowed carrier with no size
//! cap, and the owned DER builder uses unbounded `alloc` storage.
//!
//! The single remaining bound is a **native-stack-overflow guard** for
//! the recursive TLV validator.

/// Maximum constructed-type (SEQUENCE / SET) nesting depth the recursive
/// DER validator descends before reporting a depth-bound violation.
/// Guards the native call stack against pathologically-nested input; it
/// is not a ceiling on value size or element count.
pub const MAX_ASN1_DEPTH: usize = 1024;
