//! Canonical-form byte primitives — `no_std`, `no_alloc`,
//! slice-in / slice-out.
//!
//! Every UOR-ADDR realization that touches a published canonical form
//! routes through this module. The module is shaped to be liftable
//! upstream as a `uor-prism` feature without surface churn — the
//! signatures already match prism's `&[u8]` → `&mut [u8]` → `usize`
//! discipline.
//!
//! # Modules
//!
//! - [`hex`] — lowercase-hex byte-emit. Produces the 64-byte ASCII
//!   suffix [`crate::label::AddressLabel`] carries.
//! - [`nfc`] — UAX #15 Unicode NFC normalizer. Streaming three-stage
//!   decompose / canonical-reorder / compose. UCD tables vendored at
//!   `nfc::tables` (version pinned in `nfc::UCD_VERSION`).
//!
//! Every public function in this module is `no_alloc` and `no_std`.
//! No path through the module performs heap allocation, locks, or
//! syscalls.

pub mod hex;
pub mod nfc;
