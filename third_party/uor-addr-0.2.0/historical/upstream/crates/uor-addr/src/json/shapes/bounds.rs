//! JSON realization grammar constant.
//!
//! ADR-060 removed the fixed-buffer capacity profile (this module's old
//! `AddrBounds`, now the shared [`crate::bounds::AddrBounds`]) and every
//! byte/count ceiling it implied (`JSON_VALUE_MAX_BYTES`,
//! `MAX_STRING_BYTES`, `MAX_NUMBER_DIGITS`, `MAX_OBJECT_KEYS`,
//! `MAX_ARRAY_ELEMENTS`). String widths, number widths, object-key
//! counts, array-element counts, and total document size are unbounded:
//! the canonicalizer materializes the canonical form in an `alloc` buffer
//! and the input flows through the pipeline as a borrowed carrier.
//!
//! The single remaining bound is a **native-stack-overflow guard** for
//! the recursive-descent JSON parser/canonicalizer.

/// Maximum value-nesting depth the recursive-descent parser/canonicalizer
/// will descend before reporting a depth-bound violation. Guards the
/// native call stack against pathologically-nested input; it is not a
/// ceiling on document size, member count, or value width.
pub const MAX_JSON_DEPTH: usize = 1024;
