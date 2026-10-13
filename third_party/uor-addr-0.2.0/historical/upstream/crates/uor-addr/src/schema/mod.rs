//! **`uor_addr::schema` — UOR-ADDR's schema-pinned descendants**
//! (ARCHITECTURE.md "Schema-pinned descendants").
//!
//! Every descendant in this module specializes one of UOR-ADDR's
//! format-specific realizations by adding **schema-specific
//! admission predicates** at the host-boundary parser. The
//! ψ-pipeline and the κ-derivation surface are unchanged from the
//! underlying format's realization — schema admission applies at
//! parse time per SD2 Grounding, before the typed-iso surface.
//!
//! ## Schema-import discipline
//!
//! Per UOR's substitution-axis discipline (ADR-007 / ADR-030 /
//! ADR-052), well-known kinds and types map to **existing
//! standards** rather than UOR-native inventions. UOR-ADDR's
//! shipped schemas therefore import from the published taxonomies
//! that already exist for those types:
//!
//! - [`photo`] — imports **schema.org/Photograph**
//!   (<https://schema.org/Photograph>) over the JSON realization.
//! - [`document`] — imports **schema.org/Article**
//!   (<https://schema.org/Article>) over the JSON realization.
//! - [`codemodule_signed`] — imports **in-toto Statement v1**
//!   (<https://in-toto.io/Statement/v1>) over the JSON realization
//!   — the same envelope used by sigstore, SLSA, and the broader
//!   software-supply-chain attestation ecosystem.
//!
//! UOR-native primitives are reserved for low-level concerns
//! (cryptographic primitives, codec layouts like
//! [`crate::ring`]'s Amendment 43 §2 form, or the
//! [`crate::codemodule`] CCMAS canonical AST). Well-known
//! type-taxa are imported.

// The schema-pinned descendants admit JSON-LD by walking the parsed
// [`crate::json::JsonValue`] / [`crate::json::JsonValueRef`], which are
// `alloc`-gated under ADR-060 (JSON canonicalization needs heap storage
// for object-key sorting). The descendants are therefore `alloc`-gated.
#[cfg(feature = "alloc")]
pub mod codemodule_signed;
#[cfg(feature = "alloc")]
pub mod document;
#[cfg(feature = "alloc")]
pub mod photo;
