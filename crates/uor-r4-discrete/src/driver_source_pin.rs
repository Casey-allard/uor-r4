//! The pinned value of [`crate::training::driver_source_sha256`].
//!
//! `driver_source_sha256` hashes the driver sources listed in its `include_bytes!` calls.
//! This file is deliberately not one of them, so updating the pin never moves the hash it
//! pins. The check lives in the library so that `cargo test -p uor-r4-discrete --lib`
//! catches any change to a driver source; until 2026-10-24 it ran only in three binaries'
//! tests, and a lib-only run let a stale pin turn `main` red (#3099).
//!
//! When a driver source changes on purpose, recompute the hash and update this constant
//! in the same PR. The binaries' copy in `prime_zeta.rs` reads this constant.

/// SHA-256 over the driver sources, in `driver_source_sha256`'s `include_bytes!` order.
pub const DRIVER_SOURCE_SHA256: &str =
    "826c7854f0d9a88e8a1203791aef7e177b3e011ab8f80f2dcbc6dba6ca9f2e49";

#[cfg(test)]
mod tests {
    #[test]
    fn driver_source_sha256_matches_the_pin() {
        assert_eq!(
            crate::training::driver_source_sha256(),
            super::DRIVER_SOURCE_SHA256,
            "a driver source changed: recompute driver_source_sha256 and update \
             driver_source_pin::DRIVER_SOURCE_SHA256 in the same PR"
        );
    }
}
