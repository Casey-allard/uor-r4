//! Substitution-axis selections.
//!
//! - [`crate::bounds::AddrBounds`] — the `HostBounds` profile.
//! - [`Sha256Hasher`] — the canonical `HashAxis` / `Hasher` axis body,
//!   re-exported from `prism::crypto` (the wiki ADR-031 standard-library
//!   cryptography sub-crate). The implementation is the
//!   prism-published `Sha256Hasher` over `sha2 = 0.10`; this crate
//!   carries no bespoke FIPS-180-4 code of its own.
//!
//! `HostTypes` is bound to `prism::vocabulary::DefaultHostTypes` at the
//! `AddressModel` declaration site directly. `ResolverTuple` lives in
//! [`crate::resolvers`] as `AddressResolverTuple`.

pub mod bounds;

pub use bounds::MAX_JSON_DEPTH;
/// Canonical `Hasher<32>` selection for the address-derivation pipeline.
/// Re-exported from the Prism standard library; see wiki ADR-031.
pub use prism::crypto::Sha256Hasher;
