//! CS-G2 — commutative binary product (lex-min-first concatenation) per wiki [ADR-061] §(2). Five σ-axes × one shape =
//! five binary entry points.
//!
//! [ADR-061]: https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-061

pub mod model;
pub mod pipeline;
pub mod value;
pub mod verbs;

pub use pipeline::{
    compose_g2_product, compose_g2_product_blake3, compose_g2_product_keccak256,
    compose_g2_product_sha3_256, compose_g2_product_sha512,
};
