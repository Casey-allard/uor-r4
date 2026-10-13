//! GGUF realization spec constants + axis selection. The capacity
//! profile is the shared [`crate::bounds::AddrBounds`]; only the GGUF v3
//! spec constants live here.

pub mod bounds;

pub use bounds::{
    GGUF_DEFAULT_ALIGNMENT, GGUF_HEADER_BYTES, GGUF_MAGIC, GGUF_MAX_DIMS,
    GGUF_METADATA_ARRAY_DEPTH_MAX, GGUF_VERSION_REQUIRED,
};
/// Canonical `Hasher<32>` selection. Re-exported from the Prism standard
/// library; see wiki ADR-031 / ADR-047.
pub use prism::crypto::Sha256Hasher;
