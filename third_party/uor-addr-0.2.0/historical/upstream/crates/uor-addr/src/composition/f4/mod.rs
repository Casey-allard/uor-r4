//! CS-F4 — ± involution quotient (bitwise-complement lex-min) per wiki [ADR-061] §(2). Five σ-axes × one shape =
//! five unary entry points.
//!
//! [ADR-061]: https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-061

pub mod model;
pub mod pipeline;
pub mod value;
pub mod verbs;

pub use pipeline::{
    compose_f4_quotient, compose_f4_quotient_blake3, compose_f4_quotient_keccak256,
    compose_f4_quotient_sha3_256, compose_f4_quotient_sha512,
};
