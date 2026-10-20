//! Phase X.2 — cohomology/homology classes + cup product.
//!
//! Verifies that [`CohomologyClass`] carries dimension arithmetic through the
//! cup product, that dimension overflow is rejected, and that distinct
//! operands produce distinct result fingerprints.

use uor_foundation::enforcement::{
    fold_cup_product, mint_cohomology_class, mint_homology_class, CohomologyClass, CohomologyError,
    ContentFingerprint, Hasher, HomologyClass, MAX_COHOMOLOGY_DIMENSION,
};
use uor_foundation_test_helpers::Fnv1aHasher16;

type H = Fnv1aHasher16;

#[test]
fn mint_and_inspect_cohomology_class() {
    let c = mint_cohomology_class::<H, 32>(3, b"cochain-alpha").expect("within dimension cap");
    assert_eq!(c.dimension(), 3);
    assert!(c.fingerprint().width_bytes() > 0);
}

#[test]
fn mint_and_inspect_homology_class() {
    let h = mint_homology_class::<H, 32>(2, b"chain-beta").expect("within dimension cap");
    assert_eq!(h.dimension(), 2);
    assert!(h.fingerprint().width_bytes() > 0);
}

#[test]
fn cup_product_adds_dimensions() {
    let a = mint_cohomology_class::<H, 32>(3, b"a").unwrap();
    let b = mint_cohomology_class::<H, 32>(2, b"b").unwrap();
    let ab = a.cup::<H>(b).expect("5 is within cap");
    assert_eq!(ab.dimension(), 5, "cup product adds dimensions");
}

#[test]
fn cup_product_rejects_dimension_overflow() {
    let a = mint_cohomology_class::<H, 32>(20, b"a").unwrap();
    let b = mint_cohomology_class::<H, 32>(20, b"b").unwrap();
    match a.cup::<H>(b) {
        Ok(_) => panic!("40 > 32 should overflow"),
        Err(CohomologyError::DimensionOverflow { lhs, rhs }) => {
            assert_eq!(lhs, 20);
            assert_eq!(rhs, 20);
        }
    }
}

#[test]
fn mint_rejects_oversized_dimension() {
    let err = mint_cohomology_class::<H, 32>(MAX_COHOMOLOGY_DIMENSION + 1, b"x")
        .expect_err("over-cap dimension rejected");
    let CohomologyError::DimensionOverflow { lhs, .. } = err;
    assert_eq!(lhs, MAX_COHOMOLOGY_DIMENSION + 1);
}

#[test]
fn distinct_operands_yield_distinct_fingerprints() {
    let a = mint_cohomology_class::<H, 32>(2, b"a").unwrap();
    let b = mint_cohomology_class::<H, 32>(2, b"b").unwrap();
    let c = mint_cohomology_class::<H, 32>(2, b"c").unwrap();
    let d = mint_cohomology_class::<H, 32>(2, b"d").unwrap();
    let ab = a.cup::<H>(b).unwrap();
    let cd = c.cup::<H>(d).unwrap();
    assert_ne!(
        ab.fingerprint(),
        cd.fingerprint(),
        "different operands → different cup fingerprints"
    );
}

#[test]
fn cup_product_is_order_sensitive_at_fingerprint_level() {
    let a = mint_cohomology_class::<H, 32>(1, b"a").unwrap();
    let b = mint_cohomology_class::<H, 32>(1, b"b").unwrap();
    let ab = a.cup::<H>(b).unwrap();
    let ba = b.cup::<H>(a).unwrap();
    // Same dimensions but different fingerprints — the fold is lhs-then-rhs,
    // not graded-commutative at the byte level.
    assert_eq!(ab.dimension(), ba.dimension());
    assert_ne!(ab.fingerprint(), ba.fingerprint());
}

#[test]
fn fold_cup_product_is_public_and_composable() {
    // Demonstrate that downstream callers can use `fold_cup_product` directly
    // to build richer folds (e.g., an n-ary cup product chain).
    let a_fp = mint_cohomology_class::<H, 32>(1, b"a")
        .unwrap()
        .fingerprint();
    let b_fp = mint_cohomology_class::<H, 32>(2, b"b")
        .unwrap()
        .fingerprint();
    let c_fp = mint_cohomology_class::<H, 32>(1, b"c")
        .unwrap()
        .fingerprint();

    let mut hasher = <H as Hasher>::initial();
    hasher = fold_cup_product(hasher, 1, &a_fp, 2, &b_fp);
    hasher = fold_cup_product(
        hasher,
        3,
        &ContentFingerprint::from_buffer(hasher.finalize(), <H as Hasher>::OUTPUT_BYTES as u8),
        1,
        &c_fp,
    );
    let final_fp =
        ContentFingerprint::from_buffer(hasher.finalize(), <H as Hasher>::OUTPUT_BYTES as u8);
    assert!(final_fp.width_bytes() > 0);
}

#[test]
fn content_deterministic() {
    let a1 = mint_cohomology_class::<H, 32>(2, b"seed").unwrap();
    let a2 = mint_cohomology_class::<H, 32>(2, b"seed").unwrap();
    assert_eq!(a1, a2);
    let a1b = a1.cup::<H>(a1).unwrap();
    let a2b = a2.cup::<H>(a2).unwrap();
    assert_eq!(a1b, a2b);
}

#[test]
fn cohomology_and_homology_are_distinct_types() {
    let _c: CohomologyClass = mint_cohomology_class::<H, 32>(1, b"x").unwrap();
    let _h: HomologyClass = mint_homology_class::<H, 32>(1, b"x").unwrap();
    // Type-level check: HomologyClass cannot be passed to CohomologyClass::cup.
    // (Compile-time assertion; no runtime body needed.)
}
