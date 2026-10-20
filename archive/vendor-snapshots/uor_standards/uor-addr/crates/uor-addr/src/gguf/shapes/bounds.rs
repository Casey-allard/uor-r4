//! GGUF v3 spec-pinned constants.
//!
//! ADR-060 removed the fixed-width two-level commitment
//! (`GGUF_CANON_MAX_BYTES` / `GGUF_CANON_BYTES`) and the
//! application-policy capacity profile (`GgufHostBounds` /
//! `GgufAddrBounds`) with its KV-count / tensor-count / string-width /
//! array-length / tensor-data ceilings. The realization now emits the
//! **full flat canonical skeleton** (header + per-KV and per-tensor
//! records, with variable-length leaves replaced by their streamed
//! SHA-256 digests) as an unbounded `alloc` buffer that flows through the
//! pipeline as a borrowed carrier. Every count and width is unbounded.
//!
//! What remains are GGUF v3 **spec constants** (fixed by the format) plus
//! one native-stack-overflow guard on the recursive ARRAY-metadata
//! measurer.

/// `GGUF_MAGIC` — ASCII `"GGUF"` little-endian `u32`. Source: `gguf.md`.
pub const GGUF_MAGIC: u32 = 0x4655_4747;

/// The only GGUF version this realization admits. Source: `gguf.md`.
pub const GGUF_VERSION_REQUIRED: u32 = 3;

/// Header byte width: magic(4) + version(4) + tensor_count(8) +
/// kv_count(8). Source: `gguf.md`.
pub const GGUF_HEADER_BYTES: usize = 24;

/// Default tensor-data alignment when `general.alignment` is absent.
/// Overridable via that metadata key (must be a power of two ≥ 8).
/// Source: `gguf.h` `GGUF_DEFAULT_ALIGNMENT`.
pub const GGUF_DEFAULT_ALIGNMENT: u64 = 32;

/// Maximum tensor rank (`GGML_MAX_DIMS`). Source: `ggml.h`. A GGUF v3
/// tensor declares at most this many dimensions; this is a format
/// constant, not an application cap.
pub const GGUF_MAX_DIMS: usize = 4;

/// Native-stack-overflow guard on the recursive ARRAY-of-ARRAY metadata
/// measurer. Guards the call stack against pathologically-nested array
/// metadata; it is not a ceiling on array length or element count.
pub const GGUF_METADATA_ARRAY_DEPTH_MAX: usize = 64;
