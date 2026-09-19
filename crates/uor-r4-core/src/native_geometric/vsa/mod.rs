//! High-Dimensional Vector Symbolic Architecture (VSA) Context Engine.
//!
//! Multiplier-free, zero-float hyperdimensional representation of context and
//! token sequences. Replaces the 120-root pigeonhole bottleneck with a 2048/4096-bit
//! binary hypervector space operating via hardware-accelerated bitwise operations:
//!
//! - Binding ($\otimes$): Bitwise XOR (`u64` XOR). Sub-nanosecond on Apple Silicon M1.
//! - Positional Permutation ($\rho$): Cyclic bitwise rotation across all $D$ bits.
//! - Bundling ($+$): Parallel bitwise majority voting across active context windows.
//! - Similarity / Metric: Hardware `popcount` Hamming distance and $Q1.15$ fixed-point score.

#![forbid(unsafe_code)]

pub mod attention;
pub mod codebook;
pub mod context_engine;
pub mod hierarchical;
pub mod hypervector;

#[cfg(test)]
mod tests;

pub use attention::{
    HeadRole, MultiHeadVsaAttention, MultiHeadVsaResult, CANONICAL_ATTENTION_64,
    CANONICAL_ATTENTION_SEED, DEFAULT_ATTENTION_THRESHOLD, NUM_ATTENTION_HEADS,
};
pub use codebook::Codebook;
pub use context_engine::{
    encode_associative_transition_context, encode_attended_multiscale_context,
    encode_multiscale_context, encode_ngram, encode_positional_context, RollingVsaContext,
    DEFAULT_MAX_WINDOW,
};
pub use hierarchical::{
    HierarchicalCodebook, LeafBucket, Shortlist, LEAF_BUCKET_CAPACITY, MAX_PER_SECTOR,
};
pub use hypervector::{
    DefaultHypervector, Hypervector, Hypervector2048, Hypervector4096, WORDS_2048, WORDS_4096,
};
