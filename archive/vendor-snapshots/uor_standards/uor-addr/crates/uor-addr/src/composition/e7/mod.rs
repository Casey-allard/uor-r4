//! CS-E7 — S₄-quarter-permutation orbit augmentation (lex-min of the 24-orbit) per wiki [ADR-061] §(2). Five σ-axes × one shape =
//! five unary entry points.
//!
//! [ADR-061]: https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-061

pub mod model;
pub mod pipeline;
pub mod value;
pub mod verbs;

pub use pipeline::{
    compose_e7_augmentation, compose_e7_augmentation_blake3, compose_e7_augmentation_keccak256,
    compose_e7_augmentation_sha3_256, compose_e7_augmentation_sha512,
};
