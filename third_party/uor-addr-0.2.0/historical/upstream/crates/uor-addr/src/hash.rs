//! `crate::hash` — the pluggable σ-axis hash family (wiki ADR-007 /
//! ADR-010: the substrate ships no hasher; the application selects one).
//!
//! UOR-ADDR's κ-label is `<algorithm>:<lowercase-hex-digest>`. The
//! algorithm is the realization's selected σ-axis `H`; ψ₉ folds the
//! canonical carrier through `H` and formats the label
//! ([`crate::resolvers`]). [`AddrHash`] is the **fingerprint-width-erased**
//! façade over a concrete prism [`Hasher`]: it carries the wire prefix
//! (`"sha256"`, `"blake3"`, …), the digest width, and a `digest_carrier`
//! method that folds the (streamed) carrier through the axis and returns
//! the digest in a fixed [`MAX_DIGEST_BYTES`] buffer.
//!
//! Erasing the `Hasher<FP_MAX>` const-generic into the [`AddrHash`] method
//! is what lets the *single* [`AddressResolverTuple`](crate::resolvers)
//! carry both the 32-byte axes and the 64-byte `Sha512Hasher` without a
//! free `FP_MAX` parameter (which would be unconstrained — E0207 — in the
//! tuple's `Has*Resolver` impls). The model still binds the concrete
//! `Hasher<FP_MAX>` as its σ-axis, so the foundation pipeline computes the
//! full-width content fingerprint.
//!
//! ## Admissible axes
//!
//! foundation 0.5.2 generalized the resolver tower over `FP_MAX`, so every
//! prism hasher is admissible:
//!
//! | axis | `LABEL_PREFIX` | `OUTPUT_BYTES` | `LABEL_BYTES` | authority |
//! |------|----------------|----------------|---------------|-----------|
//! | [`Sha256Hasher`]    | `sha256`    | 32 | 71  | FIPS 180-4 §6.2 |
//! | [`Blake3Hasher`]    | `blake3`    | 32 | 71  | BLAKE3 §2 (the reference spec) |
//! | [`Sha3_256Hasher`]  | `sha3-256`  | 32 | 73  | FIPS 202 §6.1 |
//! | [`Keccak256Hasher`] | `keccak256` | 32 | 74  | Keccak SHA-3 submission (pre-FIPS padding) |
//! | [`Sha512Hasher`]    | `sha512`    | 64 | 135 | FIPS 180-4 §6.4 |
//!
//! [`Hasher`]: prism::vocabulary::Hasher
//! [`Sha256Hasher`]: prism::crypto::Sha256Hasher
//! [`Blake3Hasher`]: prism::crypto::Blake3Hasher
//! [`Sha3_256Hasher`]: prism::crypto::Sha3_256Hasher
//! [`Keccak256Hasher`]: prism::crypto::Keccak256Hasher
//! [`Sha512Hasher`]: prism::crypto::Sha512Hasher

use prism::crypto::{Blake3Hasher, Keccak256Hasher, Sha256Hasher, Sha3_256Hasher, Sha512Hasher};
use prism::operation::TermValue;
use prism::vocabulary::Hasher;

/// The widest admissible digest (`Sha512Hasher` = 64 bytes). Every
/// [`AddrHash::digest_carrier`] returns a buffer of this width; the first
/// `OUTPUT_BYTES` are significant.
pub const MAX_DIGEST_BYTES: usize = 64;

/// The κ-label ASCII byte width for a `<prefix>:<hex>` label over a
/// `digest_bytes`-wide digest: `prefix.len() + 1 (':') + 2 × digest_bytes`.
#[must_use]
pub const fn label_bytes(prefix: &str, digest_bytes: usize) -> usize {
    prefix.len() + 1 + 2 * digest_bytes
}

/// The widest admissible κ-label (`sha512:` + 128 hex = 135). The κ-label
/// formatter ([`crate::resolvers`]) sizes its stack scratch to this and
/// writes the active axis's `LABEL_BYTES` prefix.
pub const MAX_LABEL_BYTES: usize = label_bytes("sha512", MAX_DIGEST_BYTES);

/// A prism hasher usable as a UOR-ADDR σ-axis. Fingerprint-width-erased:
/// the κ-label prefix + digest width are associated consts, and
/// [`digest_carrier`](AddrHash::digest_carrier) folds the carrier through
/// the concrete `Hasher<FP_MAX>` internally.
pub trait AddrHash {
    /// The lowercase algorithm token at the head of the κ-label.
    const LABEL_PREFIX: &'static str;

    /// The σ-axis digest width in bytes (`Hasher::OUTPUT_BYTES`).
    const OUTPUT_BYTES: usize;

    /// Total κ-label ASCII width = `LABEL_PREFIX.len() + 1 + 2 ×
    /// OUTPUT_BYTES`. The realization's output shape declares exactly this
    /// many `Site` constraints, and the entry point returns
    /// [`KappaLabel`](crate::label::KappaLabel)`<{LABEL_BYTES}>`.
    const LABEL_BYTES: usize = label_bytes(Self::LABEL_PREFIX, Self::OUTPUT_BYTES);

    /// Fold the (streamed) canonical carrier through this σ-axis, returning
    /// the digest in a [`MAX_DIGEST_BYTES`] buffer (first `OUTPUT_BYTES`
    /// significant; the rest zero). Bounded resident memory — never
    /// materializes the carrier.
    fn digest_carrier<const N: usize>(input: &TermValue<'_, N>) -> [u8; MAX_DIGEST_BYTES];
}

/// Stream-fold a carrier through a concrete `Hasher<FP>` with bounded
/// resident memory.
fn stream<const N: usize, const FP: usize, H: Hasher<FP>>(input: &TermValue<'_, N>) -> [u8; FP] {
    let mut h = H::initial();
    input.for_each_chunk(&mut |chunk| {
        let cur = core::mem::replace(&mut h, H::initial());
        h = cur.fold_bytes(chunk);
    });
    h.finalize()
}

macro_rules! impl_addr_hash {
    ($hasher:ty, $prefix:literal, $fp:literal) => {
        impl AddrHash for $hasher {
            const LABEL_PREFIX: &'static str = $prefix;
            const OUTPUT_BYTES: usize = $fp;
            fn digest_carrier<const N: usize>(input: &TermValue<'_, N>) -> [u8; MAX_DIGEST_BYTES] {
                let digest = stream::<N, $fp, $hasher>(input);
                let mut out = [0u8; MAX_DIGEST_BYTES];
                out[..$fp].copy_from_slice(&digest);
                out
            }
        }
    };
}

impl_addr_hash!(Sha256Hasher, "sha256", 32);
impl_addr_hash!(Blake3Hasher, "blake3", 32);
impl_addr_hash!(Sha3_256Hasher, "sha3-256", 32);
impl_addr_hash!(Keccak256Hasher, "keccak256", 32);
impl_addr_hash!(Sha512Hasher, "sha512", 64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_widths_match_the_specification() {
        assert_eq!(Sha256Hasher::LABEL_BYTES, 71);
        assert_eq!(Blake3Hasher::LABEL_BYTES, 71);
        assert_eq!(Sha3_256Hasher::LABEL_BYTES, 73);
        assert_eq!(Keccak256Hasher::LABEL_BYTES, 74);
        assert_eq!(Sha512Hasher::LABEL_BYTES, 135);
        assert_eq!(MAX_LABEL_BYTES, 135);
        assert_eq!(MAX_DIGEST_BYTES, 64);
    }

    #[test]
    fn output_widths_match_the_axis() {
        assert_eq!(<Sha256Hasher as AddrHash>::OUTPUT_BYTES, 32);
        assert_eq!(<Sha512Hasher as AddrHash>::OUTPUT_BYTES, 64);
    }
}
