//! Substitution-axis selections for the CBOR realization.
//!
//! - [`crate::bounds::AddrBounds`] — the `HostBounds` profile.
//! - [`Sha256Hasher`] — the default `Hasher<32>` σ-axis, re-exported from
//!   `prism::crypto`. The other admissible axes ([`crate::hash`]) are
//!   selected via the `address_<algorithm>` entry points.
//!
//! `HostTypes` is bound to `prism::vocabulary::DefaultHostTypes` at the
//! `AddressModel` declaration site; `ResolverTuple` lives in
//! [`crate::resolvers`] as `AddressResolverTuple`.

pub mod bounds;

pub use bounds::MAX_CBOR_DEPTH;
/// Default `Hasher<32>` selection for the CBOR address-derivation pipeline.
pub use prism::crypto::Sha256Hasher;
