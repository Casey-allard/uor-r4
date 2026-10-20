//! CS-E6 — degree-partition filtration (mod-9 partition) per wiki [ADR-061] §(2). Five σ-axes × one shape =
//! five unary entry points.
//!
//! [ADR-061]: https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-061

pub mod model;
pub mod pipeline;
pub mod value;
pub mod verbs;

pub use pipeline::{
    compose_e6_filtration, compose_e6_filtration_blake3, compose_e6_filtration_keccak256,
    compose_e6_filtration_sha3_256, compose_e6_filtration_sha512,
};
