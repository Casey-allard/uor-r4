//! XML realization grammar constant.
//!
//! ADR-060 removed the fixed-buffer capacity profile (`XmlAddrBounds`)
//! and every byte/count ceiling it implied (`XML_VALUE_MAX_BYTES`,
//! `MAX_XML_TEXT_BYTES`, `MAX_XML_ATTRIBUTES`,
//! `MAX_XML_ELEMENT_NAME_BYTES`). Element names, attribute values, text
//! runs, attribute counts, and child counts are now unbounded — the
//! canonicalizer materializes the canonical form in an `alloc` buffer and
//! the input flows through the pipeline as a borrowed carrier.
//!
//! The single remaining bound is a **native-stack-overflow guard** for
//! the recursive-descent parser/canonicalizer: a maximally-nested
//! document would otherwise exhaust the call stack. This is a safety
//! bound on recursion, not a capacity ceiling on content.

/// Maximum element nesting depth the recursive-descent canonicalizer
/// will descend before reporting a depth-bound violation. Guards the
/// native call stack against pathologically-nested input (e.g. the
/// "billion laughs"-style deep-nesting denial of service); it is not a
/// ceiling on document size, element count, or content width.
pub const MAX_XML_DEPTH: usize = 1024;
