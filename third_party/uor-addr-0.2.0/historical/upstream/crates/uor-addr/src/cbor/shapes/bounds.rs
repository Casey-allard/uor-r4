//! CBOR realization grammar constant.
//!
//! Per ADR-060 there are no byte/count ceilings: byte-string and
//! text-string widths, array-element and map-entry counts, integer
//! magnitudes, and total document size are unbounded — the canonicalizer
//! materializes the RFC 8949 §4.2 canonical form in an `alloc` buffer and
//! the input flows through the pipeline as a borrowed carrier.
//!
//! The single remaining bound is a **native-stack-overflow guard** for the
//! recursive-descent CBOR parser/canonicalizer.

/// Maximum data-item nesting depth the recursive-descent
/// parser/canonicalizer will descend before reporting a depth-bound
/// violation. Guards the native call stack against pathologically-nested
/// input; it is not a ceiling on document size, element count, or value
/// width.
pub const MAX_CBOR_DEPTH: usize = 1024;
