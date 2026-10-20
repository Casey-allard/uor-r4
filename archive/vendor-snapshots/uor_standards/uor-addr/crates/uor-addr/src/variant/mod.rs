//! **`uor_addr::variant` — UOR-ADDR's cost-model-bearing variants**
//! (ARCHITECTURE.md "Cost-model-bearing variants").
//!
//! Every variant in this module specializes one of UOR-ADDR's
//! format-specific realizations (typically [`crate::json`]) by
//! binding a **non-default** `C: TypedCommitment` at the
//! `PrismModel` declaration. The κ-derivation itself is unchanged;
//! the `C` selection adds typed-bandwidth admission predicates per
//! ADR-048's cost-model surface plus QS-06's exemplar shape.
//!
//! Architectural commitment: UOR-ADDR's surface admits any
//! `C: TypedCommitment` parameterization through the same
//! `PrismModel` declaration form. The variants in this module
//! demonstrate the parameterization through concrete realizations.
//!
//! ## Shipped variants
//!
//! - [`storage`] — content-addressed-storage variant binding
//!   `C = AndCommitment<EmptyCommitment, SingletonCommitment<LexicographicLessEqThreshold>>`.
//!   The κ-label's emission carries a typed-bandwidth admission of
//!   the κ-label into the storage tier (the threshold selects which
//!   κ-labels admit to the cost-bearing tier).
//! - [`signed`] — signature-required-on-emission variant binding
//!   `C = SingletonCommitment<UltrametricCloseTo<2>>`. The published
//!   `UltrametricCloseTo` predicate admits κ-labels whose σ-projection
//!   digest is ultrametrically close (2-adic-prefix-aligned) to the
//!   signature reference — the byte-shape property an ADR-049
//!   `axis::cryptanalyze` signature witness validates.

pub mod signed;
pub mod storage;
