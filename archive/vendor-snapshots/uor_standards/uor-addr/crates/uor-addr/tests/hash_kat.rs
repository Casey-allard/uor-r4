//! **Authoritative hash known-answer tests (KATs) for the σ-axis family.**
//!
//! UOR-ADDR's κ-label is `<algorithm>:<lowercase-hex-digest>`. Per the
//! project's V&V discipline, every referenced standard is validated against
//! vectors imported from its authoritative source:
//!
//! | axis | vector source |
//! |------|---------------|
//! | sha256    | NIST FIPS 180-4 §B (`""`, `"abc"` examples) |
//! | sha3-256  | NIST FIPS 202 §A / CAVP (`""`, `"abc"`) |
//! | keccak256 | Keccak SHA-3 submission (pre-FIPS padding); the `""` digest is the canonical Ethereum empty-string hash |
//! | blake3    | the BLAKE3 reference implementation's published vectors |
//!
//! The constants below were reproduced byte-for-byte from the reference
//! implementations (`hashlib` for SHA-2/SHA-3, `pycryptodome` for Keccak,
//! the `blake3` reference crate/package) and match the standards' published
//! values.
//!
//! Two layers:
//!   * **KAT** — each prism `Hasher` the crate binds reproduces the
//!     authoritative digest. This pins the σ-axis primitives.
//!   * **Pipeline consistency** — the JSON realization's `address_*` entry
//!     points emit `<prefix>:<hex(H(canonical_form))>` for the *same* `H`,
//!     tying the validated axis to the κ-label the pipeline mints.

use prism::crypto::{Blake3Hasher, Keccak256Hasher, Sha256Hasher, Sha3_256Hasher, Sha512Hasher};
use prism::vocabulary::Hasher;
use uor_addr::hash::AddrHash;

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn digest<H: Hasher>(msg: &[u8]) -> String {
    let out = H::initial().fold_bytes(msg).finalize();
    hex(&out[..<H as Hasher>::OUTPUT_BYTES])
}

/// SHA-512 is `Hasher<64>`; its digest helper needs the 64-byte width.
fn digest64<H: Hasher<64>>(msg: &[u8]) -> String {
    let out = H::initial().fold_bytes(msg).finalize();
    hex(&out[..<H as Hasher<64>>::OUTPUT_BYTES])
}

// ── Authoritative known-answer vectors ──

const SHA256_EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
const SHA256_ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const SHA3_256_EMPTY: &str = "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a";
const SHA3_256_ABC: &str = "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532";
const KECCAK256_EMPTY: &str = "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470";
const KECCAK256_ABC: &str = "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45";
const BLAKE3_EMPTY: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
const BLAKE3_ABC: &str = "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85";
const SHA512_EMPTY: &str = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e";
const SHA512_ABC: &str = "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f";

#[test]
fn sha256_matches_fips_180_4() {
    assert_eq!(digest::<Sha256Hasher>(b""), SHA256_EMPTY);
    assert_eq!(digest::<Sha256Hasher>(b"abc"), SHA256_ABC);
}

#[test]
fn sha3_256_matches_fips_202() {
    assert_eq!(digest::<Sha3_256Hasher>(b""), SHA3_256_EMPTY);
    assert_eq!(digest::<Sha3_256Hasher>(b"abc"), SHA3_256_ABC);
}

#[test]
fn keccak256_matches_keccak_submission() {
    assert_eq!(digest::<Keccak256Hasher>(b""), KECCAK256_EMPTY);
    assert_eq!(digest::<Keccak256Hasher>(b"abc"), KECCAK256_ABC);
}

#[test]
fn blake3_matches_reference_vectors() {
    assert_eq!(digest::<Blake3Hasher>(b""), BLAKE3_EMPTY);
    assert_eq!(digest::<Blake3Hasher>(b"abc"), BLAKE3_ABC);
}

#[test]
fn sha512_matches_fips_180_4() {
    assert_eq!(digest64::<Sha512Hasher>(b""), SHA512_EMPTY);
    assert_eq!(digest64::<Sha512Hasher>(b"abc"), SHA512_ABC);
}

#[test]
fn keccak256_is_distinct_from_sha3_256() {
    // Same sponge, different padding byte (0x01 vs 0x06): the digests must
    // differ, proving the crate binds the two distinct axes.
    assert_ne!(
        digest::<Keccak256Hasher>(b""),
        digest::<Sha3_256Hasher>(b"")
    );
}

#[test]
fn addr_hash_prefix_and_width_mapping() {
    assert_eq!(Sha256Hasher::LABEL_PREFIX, "sha256");
    assert_eq!(Blake3Hasher::LABEL_PREFIX, "blake3");
    assert_eq!(Sha3_256Hasher::LABEL_PREFIX, "sha3-256");
    assert_eq!(Keccak256Hasher::LABEL_PREFIX, "keccak256");
    assert_eq!(Sha512Hasher::LABEL_PREFIX, "sha512");
    assert_eq!(Sha256Hasher::LABEL_BYTES, 71);
    assert_eq!(Blake3Hasher::LABEL_BYTES, 71);
    assert_eq!(Sha3_256Hasher::LABEL_BYTES, 73);
    assert_eq!(Keccak256Hasher::LABEL_BYTES, 74);
    assert_eq!(Sha512Hasher::LABEL_BYTES, 135);
}

// ── Pipeline consistency: the JSON realization mints `<prefix>:<hex>` over
//    the canonical form, for each axis, using exactly the validated H. ──

#[cfg(feature = "alloc")]
fn expect_label<const FP: usize, H: Hasher<FP> + AddrHash>(canonical: &[u8]) -> String {
    let d = H::initial().fold_bytes(canonical).finalize();
    format!(
        "{}:{}",
        <H as AddrHash>::LABEL_PREFIX,
        hex(&d[..<H as AddrHash>::OUTPUT_BYTES])
    )
}

#[cfg(feature = "alloc")]
#[test]
fn json_pipeline_mints_each_axis_over_canonical_form() {
    // A deliberately unsorted object: canonicalization (JCS) sorts keys, so
    // the κ-label binds the canonical form, not the input byte order.
    let raw = br#"{"b":2,"a":1}"#;
    let canonical = uor_addr::json::canonicalize(raw).expect("valid json");

    assert_eq!(
        uor_addr::json::address(raw).unwrap().address.as_str(),
        expect_label::<32, Sha256Hasher>(&canonical)
    );
    assert_eq!(
        uor_addr::json::address_blake3(raw)
            .unwrap()
            .address
            .as_str(),
        expect_label::<32, Blake3Hasher>(&canonical)
    );
    assert_eq!(
        uor_addr::json::address_sha3_256(raw)
            .unwrap()
            .address
            .as_str(),
        expect_label::<32, Sha3_256Hasher>(&canonical)
    );
    assert_eq!(
        uor_addr::json::address_keccak256(raw)
            .unwrap()
            .address
            .as_str(),
        expect_label::<32, Keccak256Hasher>(&canonical)
    );
    assert_eq!(
        uor_addr::json::address_sha512(raw)
            .unwrap()
            .address
            .as_str(),
        expect_label::<64, Sha512Hasher>(&canonical)
    );
}

#[cfg(feature = "alloc")]
#[test]
fn json_axes_are_distinct_and_deterministic() {
    let raw = br#"{"a":1}"#;
    let s = uor_addr::json::address(raw).unwrap().address;
    let b = uor_addr::json::address_blake3(raw).unwrap().address;
    let k = uor_addr::json::address_keccak256(raw).unwrap().address;
    let q = uor_addr::json::address_sha3_256(raw).unwrap().address;
    // Distinct algorithms → distinct labels.
    assert_ne!(s.as_str(), b.as_str());
    assert_ne!(q.as_str(), k.as_str());
    // Deterministic.
    assert_eq!(s, uor_addr::json::address(raw).unwrap().address);
    // Correct prefixes + widths.
    assert!(s.starts_with("sha256:") && s.len() == 71);
    assert!(b.starts_with("blake3:") && b.len() == 71);
    assert!(q.starts_with("sha3-256:") && q.len() == 73);
    assert!(k.starts_with("keccak256:") && k.len() == 74);
    let z = uor_addr::json::address_sha512(raw).unwrap().address;
    assert!(z.starts_with("sha512:") && z.len() == 135);
    assert_ne!(s.as_str(), z.as_str());
}

#[cfg(feature = "alloc")]
#[test]
fn json_witness_verifies_for_every_axis() {
    let raw = br#"{"x":[1,2,3]}"#;
    assert!(uor_addr::json::address(raw)
        .unwrap()
        .witness
        .verify()
        .is_ok());
    assert!(uor_addr::json::address_blake3(raw)
        .unwrap()
        .witness
        .verify()
        .is_ok());
    assert!(uor_addr::json::address_sha3_256(raw)
        .unwrap()
        .witness
        .verify()
        .is_ok());
    assert!(uor_addr::json::address_keccak256(raw)
        .unwrap()
        .witness
        .verify()
        .is_ok());
    // sha512 (64-byte fingerprint) witness replay round-trips too.
    let z = uor_addr::json::address_sha512(raw).unwrap();
    assert_eq!(z.witness.verify().unwrap(), z.address);
    assert_eq!(z.witness.content_fingerprint().len(), 64);
}
