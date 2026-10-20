//! Product/Coproduct Completion Amendment §1c validation:
//! `VerifiedMint` is sealed against external impls.
//!
//! `VerifiedMint: Certificate`, and `Certificate: certificate_sealed::Sealed`
//! where `certificate_sealed` is a private module of `uor_foundation`. Any
//! downstream attempt to `impl VerifiedMint` or `impl Certificate` for a
//! consumer-defined type fails to compile because the private supertrait
//! cannot be named.
//!
//! Three checks:
//! 1. The compile-fail doctest below verifies an attempted external impl
//!    of `Certificate` does not compile (the rustdoc test runner asserts
//!    the snippet *fails* to compile — passing means the seal works).
//! 2. The runtime test verifies the sealed impl set is exactly the
//!    expected closed set: each amendment witness has its `Certificate::IRI`
//!    pointing at its partition-namespace IRI.
//! 3. A type-level assertion confirms `VerifiedMint::Inputs` and
//!    `VerifiedMint::Error` are exactly the amendment-specified types
//!    for each witness.

use uor_foundation::{
    CartesianProductMintInputs, CartesianProductWitness, Certificate, ContentFingerprint,
    GenericImpossibilityWitness, PartitionCoproductMintInputs, PartitionCoproductWitness,
    PartitionProductMintInputs, PartitionProductWitness, VerifiedMint,
};

/// Compile-fail doctest: external crates cannot implement `Certificate`
/// (and therefore cannot implement `VerifiedMint`) on their own types.
/// Mirrors the established `phase_x6_sinking.rs` doctest-anchor pattern
/// in the foundation test suite.
///
/// ```compile_fail
/// use uor_foundation::Certificate;
/// struct DownstreamType;
/// // This fails: Certificate's supertrait `certificate_sealed::Sealed`
/// // is private to uor_foundation. The compiler reports
/// //   "trait `Sealed` is private" or
/// //   "the trait bound `DownstreamType: certificate_sealed::Sealed` is not satisfied".
/// impl Certificate for DownstreamType {
///     const IRI: &'static str = "https://example.org/downstream";
///     type Evidence = ();
/// }
/// ```
///
/// And the same for `VerifiedMint` directly:
///
/// ```compile_fail
/// use uor_foundation::{
///     ContentFingerprint, GenericImpossibilityWitness, VerifiedMint,
/// };
/// struct DownstreamType;
/// impl VerifiedMint for DownstreamType {
///     type Inputs = ();
///     type Error = GenericImpossibilityWitness;
///     fn mint_verified(_: ()) -> Result<Self, GenericImpossibilityWitness> {
///         unreachable!()
///     }
/// }
/// // VerifiedMint requires Certificate, which requires the private seal.
/// ```
#[allow(dead_code)]
fn _compile_fail_doctest_anchor() {}

#[test]
fn certificate_iris_match_amendment_partition_namespace() {
    // Each amendment witness's Certificate::IRI must resolve to the
    // amendment-specified partition-namespace class.
    assert_eq!(
        <PartitionProductWitness as Certificate>::IRI,
        "https://uor.foundation/partition/PartitionProduct"
    );
    assert_eq!(
        <PartitionCoproductWitness as Certificate>::IRI,
        "https://uor.foundation/partition/PartitionCoproduct"
    );
    assert_eq!(
        <CartesianProductWitness as Certificate>::IRI,
        "https://uor.foundation/partition/CartesianPartitionProduct"
    );
}

#[test]
fn verified_mint_associated_types_are_witness_specific() {
    // The Inputs type for each witness is exactly the amendment-specified
    // MintInputs struct, and the Error is exactly GenericImpossibilityWitness.
    // Use function-pointer assignment to assert this at compile time —
    // type drift in the trait impl would refuse to coerce, breaking the
    // build. Each fn-pointer assignment is then exercised at runtime by
    // calling through it with deliberately-zero inputs and asserting the
    // mint refuses (compile-time type-equality + runtime mint-rejection
    // gives both directions of evidence the trait is wired correctly).
    let mint_product: fn(
        PartitionProductMintInputs,
    ) -> Result<PartitionProductWitness, GenericImpossibilityWitness> =
        <PartitionProductWitness as VerifiedMint>::mint_verified;
    let mint_coproduct: fn(
        PartitionCoproductMintInputs,
    ) -> Result<PartitionCoproductWitness, GenericImpossibilityWitness> =
        <PartitionCoproductWitness as VerifiedMint>::mint_verified;
    let mint_cartesian: fn(
        CartesianProductMintInputs,
    ) -> Result<CartesianProductWitness, GenericImpossibilityWitness> =
        <CartesianProductWitness as VerifiedMint>::mint_verified;

    // Runtime exercise: zero-fingerprint zero-budget zero-everything
    // inputs cannot satisfy any theorem with non-trivial expectations.
    // The product and cartesian primitives accept all-zero (PT_1: 0+0=0,
    // PT_3: 0+0=0, PT_4: 0+0=0) so they MINT — verify that's the case.
    // The coproduct primitive fails on missing constraint structure.
    let zero_fp = ContentFingerprint::default();
    let product_inputs = PartitionProductMintInputs {
        witt_bits: 0,
        left_fingerprint: zero_fp,
        right_fingerprint: zero_fp,
        left_site_budget: 0,
        right_site_budget: 0,
        left_total_site_count: 0,
        right_total_site_count: 0,
        left_euler: 0,
        right_euler: 0,
        left_entropy_nats_bits: 0_u64,
        right_entropy_nats_bits: 0_u64,
        combined_site_budget: 0,
        combined_site_count: 0,
        combined_euler: 0,
        combined_entropy_nats_bits: 0_u64,
        combined_fingerprint: zero_fp,
    };
    let product_result = mint_product(product_inputs);
    assert!(
        product_result.is_ok(),
        "mint_product through fn pointer must succeed on consistent zero inputs"
    );

    // Coproduct with empty constraints fails ST_6 (no tag-pinners).
    let coproduct_inputs = PartitionCoproductMintInputs {
        witt_bits: 0,
        left_fingerprint: zero_fp,
        right_fingerprint: zero_fp,
        left_site_budget: 0,
        right_site_budget: 0,
        left_total_site_count: 0,
        right_total_site_count: 0,
        left_euler: 0,
        right_euler: 0,
        left_entropy_nats_bits: 0_u64,
        right_entropy_nats_bits: 0_u64,
        left_betti: [0; uor_foundation::enforcement::MAX_BETTI_DIMENSION],
        right_betti: [0; uor_foundation::enforcement::MAX_BETTI_DIMENSION],
        combined_site_budget: 0,
        combined_site_count: 1,
        combined_euler: 0,
        combined_entropy_nats_bits: f64::to_bits(core::f64::consts::LN_2),
        combined_betti: [0; uor_foundation::enforcement::MAX_BETTI_DIMENSION],
        combined_fingerprint: zero_fp,
        combined_constraints: &[],
        left_constraint_count: 0,
        tag_site: 0,
    };
    let coproduct_result = mint_coproduct(coproduct_inputs);
    assert!(
        coproduct_result.is_err(),
        "mint_coproduct through fn pointer must reject empty constraint structure (ST_6 violation)"
    );

    // Cartesian with consistent zero inputs mints (CPT_1 / CPT_3 / CPT_5
    // all hold trivially).
    let cartesian_inputs = CartesianProductMintInputs {
        witt_bits: 0,
        left_fingerprint: zero_fp,
        right_fingerprint: zero_fp,
        left_site_budget: 0,
        right_site_budget: 0,
        left_total_site_count: 0,
        right_total_site_count: 0,
        left_euler: 0,
        right_euler: 0,
        left_betti: [0; uor_foundation::enforcement::MAX_BETTI_DIMENSION],
        right_betti: [0; uor_foundation::enforcement::MAX_BETTI_DIMENSION],
        left_entropy_nats_bits: 0_u64,
        right_entropy_nats_bits: 0_u64,
        combined_site_budget: 0,
        combined_site_count: 0,
        combined_euler: 0,
        combined_betti: [0; uor_foundation::enforcement::MAX_BETTI_DIMENSION],
        combined_entropy_nats_bits: 0_u64,
        combined_fingerprint: zero_fp,
    };
    let cartesian_result = mint_cartesian(cartesian_inputs);
    assert!(
        cartesian_result.is_ok(),
        "mint_cartesian through fn pointer must succeed on consistent zero inputs"
    );
}
