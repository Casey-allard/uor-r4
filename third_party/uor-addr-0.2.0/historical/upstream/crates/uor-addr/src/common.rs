//! UOR-ADDR's **common architectural surface** — the declarations every
//! format-specific addressing realization shares.
//!
//! Three concentric layers (ARCHITECTURE.md):
//!
//! - **Common architectural surface** (this module + [`crate::bounds`] +
//!   [`crate::resolvers`] + [`crate::label`]) — the single
//!   [`AddrBounds`] capacity profile, the
//!   single [`AddressResolverTuple`] ψ-tower, the [`AddressLabel`] output
//!   shape, and the [`AddressInput`] marker every realization's input
//!   handle satisfies.
//! - **Format-specific realizations** (sibling modules) — each names a
//!   typed-input handle that implements [`AddressInput`] (i.e. produces
//!   its canonical-form bytes as an ADR-060 source-polymorphic
//!   [`prism::operation::TermValue`] carrier via `as_binding_value`), plus
//!   a `prism_model!` binding the shared bounds + resolver tower.
//! - **Schema-pinned descendants / cost-model variants** — specialize a
//!   realization with domain-specific structural typing or a non-empty
//!   `TypedCommitment`.
//!
//! ## ADR-060 carrier model
//!
//! A realization's canonical form is **not** copied into a fixed buffer.
//! Its input handle's `as_binding_value` returns a `TermValue` carrier —
//! `Inline` for canonical forms within the foundation-derived inline
//! width, `Borrowed` for larger in-memory forms (zero-copy), or `Stream`
//! ([`prism::uor_foundation::pipeline::ChunkSource`]) for unbounded forms folded
//! chunk-by-chunk. `run_route` folds the **full** carrier through the
//! σ-axis with bounded resident memory; the ψ-tower never materializes
//! it. There is no input size ceiling and no per-stage byte-width cap.
//!
//! ## Format-independent ψ-tower
//!
//! Because canonicalization happens at carrier production, the
//! eight-resolver tower ([`AddressResolverTuple`]) is shared verbatim:
//! ψ₁…ψ₈ thread the carrier through, ψ₉ folds it and emits the κ-label.
//! No realization carries resolver code.

use prism::pipeline::{ConstrainedTypeShape, IntoBindingValue, PartitionProductFields};

/// **The common input marker — every realization's typed-input handle
/// implements this.** It bundles the foundation bounds a model `Input`
/// must satisfy so the handle can flow through `run_route` as an ADR-060
/// carrier:
///
/// - [`ConstrainedTypeShape`] — the constraint geometry (ADR-001/017).
/// - [`IntoBindingValue`]`<'a>` — `as_binding_value` returns the
///   canonical-form `TermValue` carrier (ADR-023 amended by ADR-060). The
///   realization canonicalizes here; ψ₉ only folds.
/// - [`PartitionProductFields`] — the nerve resolver's field surface.
///
/// The realization's host-boundary parser (its own `parse`/`address`
/// function) builds the handle from raw bytes, validating the format's
/// typed-input grammar before the typed-iso surface.
pub trait AddressInput<'a>:
    ConstrainedTypeShape + IntoBindingValue<'a> + PartitionProductFields + Sized
{
}

impl<'a, T> AddressInput<'a> for T where
    T: ConstrainedTypeShape + IntoBindingValue<'a> + PartitionProductFields + Sized
{
}

#[doc(inline)]
pub use crate::bounds::{AddrBounds, ADDR_INLINE_BYTES};
#[doc(inline)]
pub use crate::label::AddressLabel;
#[doc(inline)]
pub use crate::resolvers::AddressResolverTuple;
pub use prism::pipeline::EmptyCommitment;
