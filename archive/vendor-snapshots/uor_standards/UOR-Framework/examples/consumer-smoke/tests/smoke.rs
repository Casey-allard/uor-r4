//! Cross-crate publish-readiness smoke test.
//!
//! Exercises the documented happy path through `uor-foundation` and
//! `uor-foundation-sdk` as a real external consumer would: validate a
//! `ConstrainedTypeShape`, mint a Path-2 witness via
//! `OntologyVerifiedMint`, read primitive-backed observable views, and
//! compose shapes through the three SDK procedural macros.
//!
//! Trait imports collide with top-level struct re-exports — that is
//! intentional. See `foundation/src/lib.rs:174` for the rationale: the
//! crate root re-exports `enforcement::LandauerBudget` (a host-typed
//! scalar carrier struct), which shadows the
//! `bridge::observable::LandauerBudget` *trait* needed to call
//! `.landauer_nats()` on `ValidatedLandauerView`. Likewise for the four
//! other leaf-observable traits, each of which lives at its
//! ontology-derived module path. Future maintainers should not
//! "simplify" by removing the explicit trait paths below.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use uor_foundation::bridge::derivation::DerivationDepthObservable;
use uor_foundation::bridge::observable::{JacobianObservable, LandauerBudget, Observable};
use uor_foundation::bridge::partition::{FreeRankObservable, SiteIndexHandle};
use uor_foundation::enforcement::ContentFingerprint;
use uor_foundation::kernel::carry::CarryDepthObservable;
use uor_foundation::pipeline::{
    validate_constrained_type, ConstrainedTypeShape, ConstraintRef, AFFINE_MAX_COEFFS,
};
use uor_foundation::user::type_::ConstraintHandle;
use uor_foundation::witness_scaffolds::{
    MintCompletenessWitness, MintCompletenessWitnessInputs, MintLiftObstruction,
    MintLiftObstructionInputs, OntologyVerifiedMint,
};
use uor_foundation::DefaultHostTypes;

use uor_foundation_sdk::{cartesian_product_shape, coproduct_shape, product_shape};

// ── Shape fixtures ─────────────────────────────────────────────────

const A_AFFINE_COEFFS: [i64; AFFINE_MAX_COEFFS] = {
    let mut a = [0i64; AFFINE_MAX_COEFFS];
    a[0] = 1;
    a
};

pub struct LeafA;
impl ConstrainedTypeShape for LeafA {
    const IRI: &'static str = "https://example.org/consumer-smoke/LeafA";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Affine {
        coefficients: A_AFFINE_COEFFS,
        coefficient_count: 1,
        bias: 0,
    }];
    const CYCLE_SIZE: u64 = 1;
}

pub struct LeafB;
impl ConstrainedTypeShape for LeafB {
    const IRI: &'static str = "https://example.org/consumer-smoke/LeafB";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Hamming { bound: 1 }];
    const CYCLE_SIZE: u64 = 1;
}

pub struct MyShape;
impl ConstrainedTypeShape for MyShape {
    const IRI: &'static str = "https://example.org/consumer-smoke/MyShape";
    const SITE_COUNT: usize = 4;
    const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Residue {
        modulus: 7,
        residue: 3,
    }];
    const CYCLE_SIZE: u64 = 1;
}

fn nonzero_fingerprint(seed: u8) -> ContentFingerprint {
    // 32 is the conventional `FINGERPRINT_MAX_BYTES` and the default
    // const-generic on `ContentFingerprint`. Per ADR-060 there is no
    // `DefaultHostBounds`; applications selecting a `HostBounds` impl with a
    // different `FINGERPRINT_MAX_BYTES` write `ContentFingerprint::<W>` with W
    // their chosen width.
    let mut buf = [0u8; 32];
    buf[0] = seed;
    buf[1] = seed.wrapping_add(1);
    ContentFingerprint::from_buffer(buf, 32u8)
}

// ── 1: validate const path ─────────────────────────────────────────

#[test]
fn validate_const_path() {
    let validated = validate_constrained_type(MyShape).expect("MyShape admits");
    let _ = validated;
}

// ── 2: mint Path-2 witness with real inputs ────────────────────────

#[test]
fn mint_path2_witness_with_real_inputs() {
    let inputs = MintCompletenessWitnessInputs::<DefaultHostTypes> {
        sites_closed: 7,
        witness_constraint: ConstraintHandle::<DefaultHostTypes>::new(nonzero_fingerprint(0x99)),
    };
    let witness = MintCompletenessWitness::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("non-zero handle + non-zero sites_closed admits");
    assert!(
        !witness.content_fingerprint().is_zero(),
        "minted witness must carry a non-zero content fingerprint"
    );

    let default_inputs = MintCompletenessWitnessInputs::<DefaultHostTypes>::default();
    let err = MintCompletenessWitness::ontology_mint::<DefaultHostTypes>(default_inputs)
        .expect_err("Default inputs (sites_closed = 0) must reject");
    assert_eq!(
        err.identity(),
        Some("https://uor.foundation/op/CC_1"),
        "Default::default() inputs trip CC_1 (sites_closed > 0) before CC_2"
    );
}

// ── 3: lift-obstruction dispatches on `obstruction_trivial` flag ──

#[test]
fn mint_lift_obstruction_routes_correctly() {
    // trivial=true with a non-zero site → WLS_1.
    let trivial_with_site = MintLiftObstructionInputs::<DefaultHostTypes> {
        obstruction_trivial: true,
        obstruction_site: SiteIndexHandle::<DefaultHostTypes>::new(nonzero_fingerprint(0xAA)),
    };
    let err = MintLiftObstruction::ontology_mint::<DefaultHostTypes>(trivial_with_site)
        .expect_err("trivial=true with non-zero site must reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/WLS_1"));

    // trivial=false with a zero site → WLS_2.
    let nontrivial_with_zero = MintLiftObstructionInputs::<DefaultHostTypes> {
        obstruction_trivial: false,
        obstruction_site: SiteIndexHandle::<DefaultHostTypes>::new(ContentFingerprint::default()),
    };
    let err = MintLiftObstruction::ontology_mint::<DefaultHostTypes>(nontrivial_with_zero)
        .expect_err("trivial=false with zero site must reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/WLS_2"));
}

// ── 4: per-class observable views are primitive-backed ────────────

#[test]
fn observable_views_return_primitive_backed_values() {
    let validated = validate_constrained_type(MyShape).expect("MyShape admits");

    // Each accessor returns a zero-cost newtype view; calling its
    // leaf-trait method requires the trait to be in scope (see top-of-file
    // import block).
    let landauer = validated.as_landauer();
    let _: <DefaultHostTypes as uor_foundation::HostTypes>::Decimal =
        <_ as LandauerBudget<DefaultHostTypes>>::landauer_nats(&landauer);

    let jacobian = validated.as_jacobian();
    let _: <DefaultHostTypes as uor_foundation::HostTypes>::Decimal =
        <_ as Observable<DefaultHostTypes>>::value(&jacobian);
    fn assert_jacobian<V: JacobianObservable<DefaultHostTypes>>() {}
    assert_jacobian::<
        uor_foundation::blanket_impls::ValidatedJacobianView<
            MyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();

    let carry = validated.as_carry_depth();
    let _: <DefaultHostTypes as uor_foundation::HostTypes>::Decimal =
        <_ as Observable<DefaultHostTypes>>::value(&carry);
    fn assert_carry<V: CarryDepthObservable<DefaultHostTypes>>() {}
    assert_carry::<
        uor_foundation::blanket_impls::ValidatedCarryDepthView<
            MyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();

    let derivation = validated.as_derivation_depth();
    let _: <DefaultHostTypes as uor_foundation::HostTypes>::Decimal =
        <_ as Observable<DefaultHostTypes>>::value(&derivation);
    fn assert_derivation<V: DerivationDepthObservable<DefaultHostTypes>>() {}
    assert_derivation::<
        uor_foundation::blanket_impls::ValidatedDerivationDepthView<
            MyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();

    let free_rank = validated.as_free_rank();
    let _: <DefaultHostTypes as uor_foundation::HostTypes>::Decimal =
        <_ as Observable<DefaultHostTypes>>::value(&free_rank);
    fn assert_free_rank<V: FreeRankObservable<DefaultHostTypes>>() {}
    assert_free_rank::<
        uor_foundation::blanket_impls::ValidatedFreeRankView<
            MyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();
}

// ── 5: SDK macros round-trip with Affine-bearing operands ─────────

product_shape!(MyProduct, LeafA, LeafB);
coproduct_shape!(MySum, LeafA, LeafB);
cartesian_product_shape!(MyCartesian, LeafA, LeafB);

#[test]
fn sdk_macro_round_trip_product() {
    assert!(<MyProduct as ConstrainedTypeShape>::IRI.starts_with("urn:uor:product:"));
    assert_eq!(
        <MyProduct as ConstrainedTypeShape>::SITE_COUNT,
        <LeafA as ConstrainedTypeShape>::SITE_COUNT + <LeafB as ConstrainedTypeShape>::SITE_COUNT
    );
    assert_eq!(
        <MyProduct as ConstrainedTypeShape>::CONSTRAINTS.len(),
        <LeafA as ConstrainedTypeShape>::CONSTRAINTS.len()
            + <LeafB as ConstrainedTypeShape>::CONSTRAINTS.len()
    );
    let _ = validate_constrained_type(MyProduct).expect("composite admits");
}

#[test]
fn sdk_macro_round_trip_coproduct() {
    assert!(<MySum as ConstrainedTypeShape>::IRI.starts_with("urn:uor:coproduct:"));
    let _ = validate_constrained_type(MySum).expect("composite admits");
}

#[test]
fn sdk_macro_round_trip_cartesian() {
    use uor_foundation::pipeline::CartesianProductShape;
    fn assert_marker<T: CartesianProductShape>() {}
    assert_marker::<MyCartesian>();
    let _ = validate_constrained_type(MyCartesian).expect("composite admits");
}
