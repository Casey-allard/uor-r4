//! **CA — composition conformance (wiki ADR-061).**
//!
//! Behavior-driven validation that the five categorical operations on the
//! Atlas image inside E₈ uphold their named algebraic laws *through the
//! full `compose_*` pipeline* (not just the byte-level canonicalize
//! discipline, which `composition::canonicalize`'s unit tests pin):
//!
//! - **CS-G2** commutative binary product — `g2(a, b) == g2(b, a)`;
//! - **CS-F4** ± involution quotient — an operand and its mirror collapse
//!   to one composed κ-label;
//! - **CS-E6** degree-partition filtration — well-formed + deterministic;
//! - **CS-E7** S₄-orbit augmentation — well-formed + deterministic;
//! - **CS-E8** direct embedding — distinguished from its operand by
//!   realization provenance (distinct κ-label);
//! - **CA-3** σ-axis homogeneity — a foreign-axis operand is rejected;
//! - every operation works on all five σ-axes, and the composed κ-label
//!   carries a replayable TC-05 witness.

#![cfg(feature = "alloc")]

use uor_addr::composition::{
    compose_e6_filtration, compose_e7_augmentation, compose_e8_embedding, compose_f4_quotient,
    compose_g2_product, compose_g2_product_blake3, compose_g2_product_sha512, CompositionFailure,
};
use uor_addr::KappaLabel;

fn lab<const N: usize>(s: &str) -> KappaLabel<N> {
    KappaLabel::from_bytes(s.as_bytes()).expect("well-formed κ-label")
}

const A256: &str = "sha256:0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
const B256: &str = "sha256:ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00";
const ZEROS256: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
const FFS256: &str = "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

fn well_formed(label: &str, prefix: &str, len: usize) {
    assert!(
        label.starts_with(prefix),
        "expected prefix {prefix}: {label}"
    );
    assert_eq!(label.len(), len, "wrong κ-label width: {label}");
    for &b in &label.as_bytes()[prefix.len() + 1..] {
        assert!(
            b.is_ascii_digit() || (b'a'..=b'f').contains(&b),
            "non-hex digit"
        );
    }
}

#[test]
fn g2_product_is_commutative_through_the_pipeline() {
    let a = lab::<71>(A256);
    let b = lab::<71>(B256);
    let ab = compose_g2_product(&a, &b).expect("g2(a,b)").address;
    let ba = compose_g2_product(&b, &a).expect("g2(b,a)").address;
    assert_eq!(
        ab, ba,
        "CS-G2 commutativity must hold for the composed κ-label"
    );
    well_formed(ab.as_str(), "sha256", 71);
}

#[test]
fn f4_quotient_collapses_the_mirror_pair() {
    let z = lab::<71>(ZEROS256);
    let f = lab::<71>(FFS256); // bitwise mirror of all-zeros
    let cz = compose_f4_quotient(&z).expect("f4(zeros)").address;
    let cf = compose_f4_quotient(&f).expect("f4(ffs)").address;
    assert_eq!(cz, cf, "CS-F4: a ± mirror pair collapses to one κ-label");
}

#[test]
fn e6_e7_are_well_formed_and_deterministic() {
    let a = lab::<71>(A256);
    let e6 = compose_e6_filtration(&a).expect("e6").address;
    let e7 = compose_e7_augmentation(&a).expect("e7").address;
    well_formed(e6.as_str(), "sha256", 71);
    well_formed(e7.as_str(), "sha256", 71);
    assert_eq!(
        e6,
        compose_e6_filtration(&a).unwrap().address,
        "deterministic"
    );
    assert_eq!(
        e7,
        compose_e7_augmentation(&a).unwrap().address,
        "deterministic"
    );
}

#[test]
fn e8_embedding_is_distinguished_from_its_operand() {
    // CS-E8 is identity on canonical-form bytes, but the composed κ-label
    // is H(operand-label-bytes) under the E8 realization IRI — distinct
    // from the operand κ-label itself.
    let a = lab::<71>(A256);
    let e8 = compose_e8_embedding(&a).expect("e8").address;
    well_formed(e8.as_str(), "sha256", 71);
    assert_ne!(e8.as_str(), A256, "CS-E8 composed label ≠ operand label");
}

#[test]
fn operations_with_distinct_canonical_forms_yield_distinct_labels() {
    // Operations whose canonical forms have distinct *lengths* always mint
    // distinct κ-labels: CS-G2 emits 2N bytes, CS-E6 emits N+1, CS-E8 emits
    // N. (CS-F4 / CS-E7 deliberately reduce to E8's identity on a
    // fixed-point operand — F4's representative of a "positive" digest and
    // E7's lex-min of already-ascending quarters are both the operand
    // itself; they are distinguished from E8 by realization IRI, not by
    // digest bytes, per ADR-061 §(3).)
    let a = lab::<71>(A256);
    let g2 = compose_g2_product(&a, &a).unwrap().address; // 2N = 142 bytes
    let e6 = compose_e6_filtration(&a).unwrap().address; // N+1 = 72 bytes
    let e8 = compose_e8_embedding(&a).unwrap().address; //   N = 71 bytes
    assert_ne!(g2, e6);
    assert_ne!(g2, e8);
    assert_ne!(e6, e8);
}

#[test]
fn sigma_axis_homogeneity_is_enforced() {
    // CA-3: a blake3 operand fed to the sha256 product entry point is
    // rejected with the σ-axis mismatch (not silently coerced).
    let blake =
        lab::<71>("blake3:0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20");
    let sha = lab::<71>(A256);
    match compose_g2_product(&blake, &sha) {
        Err(CompositionFailure::OperandSigmaAxisMismatch {
            expected_axis,
            operand_axis,
        }) => {
            assert_eq!(expected_axis, "sha256");
            assert_eq!(operand_axis, "blake3");
        }
        other => panic!("expected σ-axis mismatch, got {other:?}"),
    }
}

#[test]
fn composition_axes_blake3_and_sha512() {
    // The operations are offered on every σ-axis; spot-check the 32- and
    // 64-byte ends with a witness round-trip.
    let b = lab::<71>("blake3:0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20");
    let out = compose_g2_product_blake3(&b, &b).expect("blake3 g2");
    well_formed(out.address.as_str(), "blake3", 71);
    assert!(
        out.witness.verify().is_ok(),
        "blake3 composed witness verifies"
    );

    let z = lab::<135>(
        "sha512:00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    );
    let out512 = compose_g2_product_sha512(&z, &z).expect("sha512 g2");
    well_formed(out512.address.as_str(), "sha512", 135);
    assert_eq!(out512.witness.content_fingerprint().len(), 64);
    assert!(
        out512.witness.verify().is_ok(),
        "sha512 composed witness verifies"
    );
}
