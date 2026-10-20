//! `AddressLabel*` — UOR-ADDR's **common output shape** family
//! (ARCHITECTURE.md "Common output shape"), one specialization per
//! admissible σ-axis ([`crate::hash`]).
//!
//! The κ-label's wire-format byte layout follows the architecture
//! document's structural formula
//!
//! ```text
//! SITE_COUNT = H::LABEL_PREFIX.len() + 1 + 2 × H::OUTPUT_BYTES
//! ```
//!
//! parameterized on the realization's selected hash axis `H: AddrHash`.
//! The output space is **π_0-only** by structural property of the
//! σ-projection + hex-serialization composition; `χ(N(C)) = SITE_COUNT`;
//! `β_0 = SITE_COUNT`; `β_k = 0` for `k ≥ 1`.
//!
//! Each admissible axis has its own [`output_shape!`] specialization with
//! its own IRI suffix and `SITE_COUNT` (sha256 / blake3 = 71, sha3-256 =
//! 73, keccak256 = 74). The IRI specializes per axis so that two
//! realizations binding different `H` selections produce distinct typed
//! reference vocabularies at the IRI level (the framework's typed-iso
//! commitment per ADR-001 + ADR-017).
//!
//! The runtime κ-label carrier [`KappaLabel`] is generic over the label
//! byte width `N`, so a single value type carries every axis's label; the
//! width lives in the type (`KappaLabel<71>` for sha256, `KappaLabel<74>`
//! for keccak256, …).

use prism::pipeline::{output_shape, ConstraintRef};

use crate::hash::{label_bytes, AddrHash};
use prism::crypto::Sha256Hasher;

/// **The wire-format address byte width** under the default σ-axis
/// `H = Sha256Hasher`: `len("sha256") + 1 + 2 × 32 = 71`.
pub const ADDRESS_LABEL_BYTES: usize = label_bytes(Sha256Hasher::LABEL_PREFIX, 32);

/// Build the `[Site{0}, Site{1}, …, Site{N-1}]` constraint array — one
/// disjoint `ConstraintRef::Site` per wire-format κ-label byte position.
#[must_use]
pub const fn site_constraints<const N: usize>() -> [ConstraintRef; N] {
    let mut sites = [ConstraintRef::Site { position: 0 }; N];
    let mut i = 0;
    while i < N {
        sites[i] = ConstraintRef::Site { position: i as u32 };
        i += 1;
    }
    sites
}

// ── Per-axis output shapes. Each declares `SITE_COUNT` disjoint `Site`
//    constraints — one per wire-format byte position — and a per-axis IRI
//    suffix carrying the algorithm token. ──

static SHA256_SITES: [ConstraintRef; 71] = site_constraints::<71>();
output_shape! {
    pub struct AddressLabelSha256;
    impl ConstrainedTypeShape for AddressLabelSha256 {
        const IRI: &'static str = "https://uor.foundation/addr/AddressLabel/sha256";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA256_SITES;
    }
}

static BLAKE3_SITES: [ConstraintRef; 71] = site_constraints::<71>();
output_shape! {
    pub struct AddressLabelBlake3;
    impl ConstrainedTypeShape for AddressLabelBlake3 {
        const IRI: &'static str = "https://uor.foundation/addr/AddressLabel/blake3";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &BLAKE3_SITES;
    }
}

static SHA3_256_SITES: [ConstraintRef; 73] = site_constraints::<73>();
output_shape! {
    pub struct AddressLabelSha3_256;
    impl ConstrainedTypeShape for AddressLabelSha3_256 {
        const IRI: &'static str = "https://uor.foundation/addr/AddressLabel/sha3-256";
        const SITE_COUNT: usize = 73;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA3_256_SITES;
    }
}

static KECCAK256_SITES: [ConstraintRef; 74] = site_constraints::<74>();
output_shape! {
    pub struct AddressLabelKeccak256;
    impl ConstrainedTypeShape for AddressLabelKeccak256 {
        const IRI: &'static str = "https://uor.foundation/addr/AddressLabel/keccak256";
        const SITE_COUNT: usize = 74;
        const CONSTRAINTS: &'static [ConstraintRef] = &KECCAK256_SITES;
    }
}

static SHA512_SITES: [ConstraintRef; 135] = site_constraints::<135>();
output_shape! {
    pub struct AddressLabelSha512;
    impl ConstrainedTypeShape for AddressLabelSha512 {
        const IRI: &'static str = "https://uor.foundation/addr/AddressLabel/sha512";
        const SITE_COUNT: usize = 135;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA512_SITES;
    }
}

/// The default-axis output shape (`H = Sha256Hasher`). Realizations'
/// `address()` entry point binds this; `address_blake3` / `address_sha3_256`
/// / `address_keccak256` bind the corresponding per-axis shape.
pub type AddressLabel = AddressLabelSha256;

// ── Composition output shapes (ADR-061 §(2)): one per categorical
//    operation × σ-axis. The composed κ-label is a standard axis-width
//    label distinguished from its operands by the per-op realization IRI. ──

output_shape! {
    pub struct CompositionLabelG2Sha256;
    impl ConstrainedTypeShape for CompositionLabelG2Sha256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/g2-product/sha256";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelG2Blake3;
    impl ConstrainedTypeShape for CompositionLabelG2Blake3 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/g2-product/blake3";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &BLAKE3_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelG2Sha3_256;
    impl ConstrainedTypeShape for CompositionLabelG2Sha3_256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/g2-product/sha3-256";
        const SITE_COUNT: usize = 73;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA3_256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelG2Keccak256;
    impl ConstrainedTypeShape for CompositionLabelG2Keccak256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/g2-product/keccak256";
        const SITE_COUNT: usize = 74;
        const CONSTRAINTS: &'static [ConstraintRef] = &KECCAK256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelG2Sha512;
    impl ConstrainedTypeShape for CompositionLabelG2Sha512 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/g2-product/sha512";
        const SITE_COUNT: usize = 135;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA512_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelF4Sha256;
    impl ConstrainedTypeShape for CompositionLabelF4Sha256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/f4-quotient/sha256";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelF4Blake3;
    impl ConstrainedTypeShape for CompositionLabelF4Blake3 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/f4-quotient/blake3";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &BLAKE3_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelF4Sha3_256;
    impl ConstrainedTypeShape for CompositionLabelF4Sha3_256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/f4-quotient/sha3-256";
        const SITE_COUNT: usize = 73;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA3_256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelF4Keccak256;
    impl ConstrainedTypeShape for CompositionLabelF4Keccak256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/f4-quotient/keccak256";
        const SITE_COUNT: usize = 74;
        const CONSTRAINTS: &'static [ConstraintRef] = &KECCAK256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelF4Sha512;
    impl ConstrainedTypeShape for CompositionLabelF4Sha512 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/f4-quotient/sha512";
        const SITE_COUNT: usize = 135;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA512_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE6Sha256;
    impl ConstrainedTypeShape for CompositionLabelE6Sha256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e6-filtration/sha256";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE6Blake3;
    impl ConstrainedTypeShape for CompositionLabelE6Blake3 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e6-filtration/blake3";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &BLAKE3_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE6Sha3_256;
    impl ConstrainedTypeShape for CompositionLabelE6Sha3_256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e6-filtration/sha3-256";
        const SITE_COUNT: usize = 73;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA3_256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE6Keccak256;
    impl ConstrainedTypeShape for CompositionLabelE6Keccak256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e6-filtration/keccak256";
        const SITE_COUNT: usize = 74;
        const CONSTRAINTS: &'static [ConstraintRef] = &KECCAK256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE6Sha512;
    impl ConstrainedTypeShape for CompositionLabelE6Sha512 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e6-filtration/sha512";
        const SITE_COUNT: usize = 135;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA512_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE7Sha256;
    impl ConstrainedTypeShape for CompositionLabelE7Sha256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e7-augmentation/sha256";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE7Blake3;
    impl ConstrainedTypeShape for CompositionLabelE7Blake3 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e7-augmentation/blake3";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &BLAKE3_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE7Sha3_256;
    impl ConstrainedTypeShape for CompositionLabelE7Sha3_256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e7-augmentation/sha3-256";
        const SITE_COUNT: usize = 73;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA3_256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE7Keccak256;
    impl ConstrainedTypeShape for CompositionLabelE7Keccak256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e7-augmentation/keccak256";
        const SITE_COUNT: usize = 74;
        const CONSTRAINTS: &'static [ConstraintRef] = &KECCAK256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE7Sha512;
    impl ConstrainedTypeShape for CompositionLabelE7Sha512 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e7-augmentation/sha512";
        const SITE_COUNT: usize = 135;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA512_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE8Sha256;
    impl ConstrainedTypeShape for CompositionLabelE8Sha256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e8-embedding/sha256";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE8Blake3;
    impl ConstrainedTypeShape for CompositionLabelE8Blake3 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e8-embedding/blake3";
        const SITE_COUNT: usize = 71;
        const CONSTRAINTS: &'static [ConstraintRef] = &BLAKE3_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE8Sha3_256;
    impl ConstrainedTypeShape for CompositionLabelE8Sha3_256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e8-embedding/sha3-256";
        const SITE_COUNT: usize = 73;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA3_256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE8Keccak256;
    impl ConstrainedTypeShape for CompositionLabelE8Keccak256 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e8-embedding/keccak256";
        const SITE_COUNT: usize = 74;
        const CONSTRAINTS: &'static [ConstraintRef] = &KECCAK256_SITES;
    }
}

output_shape! {
    pub struct CompositionLabelE8Sha512;
    impl ConstrainedTypeShape for CompositionLabelE8Sha512 {
        const IRI: &'static str = "https://uor.foundation/addr/composition/e8-embedding/sha512";
        const SITE_COUNT: usize = 135;
        const CONSTRAINTS: &'static [ConstraintRef] = &SHA512_SITES;
    }
}

/// **The runtime κ-label carrier** — the `N`-byte ASCII
/// `<algorithm>:<lowercase-hex>` wire-format byte sequence.
///
/// `KappaLabel` carries the κ-derivation output of the ψ-pipeline: the
/// algorithm prefix, a `:` separator, and the lowercase-hex serialization
/// of the σ-projection digest. The width `N` is the axis's
/// [`AddrHash::LABEL_BYTES`] (71 for sha256 / blake3, 73 for sha3-256, 74
/// for keccak256). The constructor [`KappaLabel::from_bytes`] validates
/// length (= `N`) and ASCII purity; downstream methods can therefore
/// project to `&str` infallibly.
///
/// `Copy + Eq + Hash` — callers may freely thread the κ-label through hash
/// maps, identity comparisons, and pass-by-value contexts without any
/// allocator. `Deref<Target = str>` provides the usual `&str` methods.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KappaLabel<const N: usize>([u8; N]);

impl<const N: usize> KappaLabel<N> {
    /// Build a `KappaLabel<N>` from an `N`-byte ASCII slice. Returns
    /// `LabelDecodeError` if `bytes.len() != N` or `bytes` contains a
    /// non-ASCII byte.
    ///
    /// The ψ-pipeline emits ASCII-only bytes by construction (the
    /// algorithm prefix plus lowercase-hex). Defense-in-depth: the
    /// constructor still validates so a substrate-corrupted byte sequence
    /// cannot smuggle a non-ASCII `KappaLabel` into the typed surface.
    ///
    /// # Errors
    ///
    /// - [`LabelDecodeError::WrongLength`] — `bytes.len() != N`.
    /// - [`LabelDecodeError::NonAscii`] — `bytes` contains a byte ≥ 0x80.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LabelDecodeError> {
        if bytes.len() != N {
            return Err(LabelDecodeError::WrongLength);
        }
        let mut buf = [0u8; N];
        for (dst, &src) in buf.iter_mut().zip(bytes.iter()) {
            if !src.is_ascii() {
                return Err(LabelDecodeError::NonAscii);
            }
            *dst = src;
        }
        Ok(Self(buf))
    }

    /// Borrow the κ-label as an `N`-byte array.
    #[must_use]
    pub fn as_array(&self) -> &[u8; N] {
        &self.0
    }

    /// Borrow the κ-label as a `&str`. Infallible — the carrier is ASCII
    /// by construction (validated at [`KappaLabel::from_bytes`]).
    #[must_use]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.0).expect("KappaLabel is ASCII by construction")
    }

    /// Borrow the κ-label as a byte slice.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// The σ-axis token at the head of the κ-label — the substring before
    /// the `:` separator (`"sha256"`, `"blake3"`, `"sha3-256"`,
    /// `"keccak256"`, `"sha512"`). Returns `None` if the label carries no
    /// `:` (unreachable for a pipeline-emitted κ-label).
    #[must_use]
    pub fn sigma_axis(&self) -> Option<&str> {
        self.as_str().split_once(':').map(|(axis, _)| axis)
    }

    /// The lowercase-hex digest body — the substring after the `:`
    /// separator. Returns `None` if the label carries no `:`.
    #[must_use]
    pub fn sigma_axis_digest_hex(&self) -> Option<&str> {
        self.as_str().split_once(':').map(|(_, hex)| hex)
    }
}

impl<const N: usize> core::ops::Deref for KappaLabel<N> {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> AsRef<str> for KappaLabel<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> AsRef<[u8]> for KappaLabel<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> core::fmt::Display for KappaLabel<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> core::fmt::Debug for KappaLabel<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("KappaLabel").field(&self.as_str()).finish()
    }
}

impl<const N: usize> PartialEq<str> for KappaLabel<N> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const N: usize> PartialEq<&str> for KappaLabel<N> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const N: usize> PartialEq<KappaLabel<N>> for str {
    fn eq(&self, other: &KappaLabel<N>) -> bool {
        self == other.as_str()
    }
}

impl<const N: usize> PartialEq<KappaLabel<N>> for &str {
    fn eq(&self, other: &KappaLabel<N>) -> bool {
        *self == other.as_str()
    }
}

/// Decoding failures from [`KappaLabel::from_bytes`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelDecodeError {
    /// `bytes.len() != N`.
    WrongLength,
    /// `bytes` contains a non-ASCII byte (≥ 0x80).
    NonAscii,
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism::crypto::{Blake3Hasher, Keccak256Hasher, Sha3_256Hasher};
    use prism::pipeline::ConstrainedTypeShape;

    #[test]
    fn site_count_matches_wire_format_byte_width_per_axis() {
        assert_eq!(
            <AddressLabelSha256 as ConstrainedTypeShape>::SITE_COUNT,
            Sha256Hasher::LABEL_BYTES
        );
        assert_eq!(
            <AddressLabelBlake3 as ConstrainedTypeShape>::SITE_COUNT,
            Blake3Hasher::LABEL_BYTES
        );
        assert_eq!(
            <AddressLabelSha3_256 as ConstrainedTypeShape>::SITE_COUNT,
            Sha3_256Hasher::LABEL_BYTES
        );
        assert_eq!(
            <AddressLabelKeccak256 as ConstrainedTypeShape>::SITE_COUNT,
            Keccak256Hasher::LABEL_BYTES
        );
    }

    #[test]
    fn iri_carries_axis_suffix_per_architecture() {
        assert_eq!(
            <AddressLabelSha256 as ConstrainedTypeShape>::IRI,
            "https://uor.foundation/addr/AddressLabel/sha256"
        );
        assert_eq!(
            <AddressLabelKeccak256 as ConstrainedTypeShape>::IRI,
            "https://uor.foundation/addr/AddressLabel/keccak256"
        );
    }

    #[test]
    fn each_shape_carries_disjoint_site_constraints() {
        let cs = <AddressLabelSha3_256 as ConstrainedTypeShape>::CONSTRAINTS;
        assert_eq!(cs.len(), 73);
        for (i, c) in cs.iter().enumerate() {
            match c {
                ConstraintRef::Site { position } => assert_eq!(*position, i as u32),
                _ => panic!("expected Site constraint at index {i}"),
            }
        }
    }

    #[test]
    fn kappa_label_from_bytes_round_trips_valid_input() {
        let bytes = b"sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let label = KappaLabel::<71>::from_bytes(bytes).expect("valid");
        assert_eq!(label.as_str(), core::str::from_utf8(bytes).unwrap());
        assert_eq!(label.as_bytes(), bytes);
        assert_eq!(label.as_array(), bytes);
        assert!(label.starts_with("sha256:"));
        assert_eq!(label.len(), ADDRESS_LABEL_BYTES);
    }

    #[test]
    fn kappa_label_rejects_wrong_length() {
        let err = KappaLabel::<71>::from_bytes(b"too short").expect_err("rejects");
        assert_eq!(err, LabelDecodeError::WrongLength);
    }

    #[test]
    fn kappa_label_rejects_non_ascii_byte() {
        let mut bytes = [b'a'; 71];
        bytes[3] = 0x80;
        let err = KappaLabel::<71>::from_bytes(&bytes).expect_err("rejects");
        assert_eq!(err, LabelDecodeError::NonAscii);
    }

    #[test]
    fn kappa_label_equality_against_str() {
        let bytes = b"sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let label = KappaLabel::<71>::from_bytes(bytes).expect("valid");
        let s: &str = core::str::from_utf8(bytes).unwrap();
        assert_eq!(label, *s);
        assert_eq!(label, s);
        assert_eq!(s, label);
    }
}
