//! CS-E8 — identity-on-canonical-form-bytes embedding (distinguished by realization IRI) per wiki [ADR-061] §(2). Five σ-axes × one shape =
//! five unary entry points.
//!
//! [ADR-061]: https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-061

pub mod model;
pub mod pipeline;
pub mod value;
pub mod verbs;

pub use pipeline::{
    compose_e8_embedding, compose_e8_embedding_blake3, compose_e8_embedding_keccak256,
    compose_e8_embedding_sha3_256, compose_e8_embedding_sha512,
};
