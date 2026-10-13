//! **`uor_addr::composition` — the five categorical operations on the
//! Atlas image inside E₈** (wiki [ADR-061]).
//!
//! UOR-ADDR addresses *content*; this module addresses *compositions of
//! content*. Each operation takes one or two operand κ-labels and mints a
//! new κ-label for the composed object, by:
//!
//! 1. **canonicalize** ([`canonicalize`]) — apply the operation's
//!    byte-level discipline to the operand digest bytes (ADR-061 §(3); the
//!    realization commitment per CA-5);
//! 2. **ground** — fold the canonical form through the same σ-axis the
//!    operands carry (CA-3 σ-axis homogeneity), via a per-operation
//!    [`PrismModel`](prism::pipeline::PrismModel) whose output shape's IRI
//!    records the operation's provenance (ADR-001 / ADR-017 typed-iso).
//!
//! The framework (ADR-061 §(3), ADR-059) names each operation's algebraic
//! structure; the realization commits the specific byte-level relation
//! that implements it:
//!
//! | op | algebraic structure (framework) | byte-level discipline (realization) |
//! |----|----------------------------------|-------------------------------------|
//! | [`g2`] CS-G2 | commutative binary product (ADR-059) | lex-min-first concatenation |
//! | [`f4`] CS-F4 | 2-element equivalence relation (± mirror) | bitwise-complement lex-min |
//! | [`e6`] CS-E6 | 2-class partition, 8:1 population (ADR-059) | `first_byte mod 9` degree tag |
//! | [`e7`] CS-E7 | 24-element equivalence relation (S₄ orbit) | quarter-permutation lex-min |
//! | [`e8`] CS-E8 | identity relation | identity on canonical-form bytes |
//!
//! Every operation is offered on each of the five σ-axes ([`crate::hash`]);
//! the operand and composed κ-labels share the axis.
//!
//! [ADR-061]: https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-061

#![cfg(feature = "alloc")]

pub mod canonicalize;
pub mod e6;
pub mod e7;
pub mod e8;
pub mod f4;
pub mod g2;

pub use e6::{
    compose_e6_filtration, compose_e6_filtration_blake3, compose_e6_filtration_keccak256,
    compose_e6_filtration_sha3_256, compose_e6_filtration_sha512,
};
pub use e7::{
    compose_e7_augmentation, compose_e7_augmentation_blake3, compose_e7_augmentation_keccak256,
    compose_e7_augmentation_sha3_256, compose_e7_augmentation_sha512,
};
pub use e8::{
    compose_e8_embedding, compose_e8_embedding_blake3, compose_e8_embedding_keccak256,
    compose_e8_embedding_sha3_256, compose_e8_embedding_sha512,
};
pub use f4::{
    compose_f4_quotient, compose_f4_quotient_blake3, compose_f4_quotient_keccak256,
    compose_f4_quotient_sha3_256, compose_f4_quotient_sha512,
};
pub use g2::{
    compose_g2_product, compose_g2_product_blake3, compose_g2_product_keccak256,
    compose_g2_product_sha3_256, compose_g2_product_sha512,
};

/// Failure modes from the composition operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositionFailure {
    /// An operand is not a well-formed κ-label (no `:` separator, an
    /// odd-length or non-lowercase-hex digest body).
    MalformedOperand,
    /// An operand's σ-axis does not match the operation's axis (CA-3
    /// σ-axis homogeneity). For the binary product, this also fires when
    /// the two operands carry different axes.
    OperandSigmaAxisMismatch {
        /// The σ-axis the operation expects.
        expected_axis: &'static str,
        /// The σ-axis the offending operand carries.
        operand_axis: &'static str,
    },
    /// Defensive: foundation's catamorphism or a resolver returned a shape
    /// violation. Unreachable for well-formed operands.
    PipelineFailure,
}
