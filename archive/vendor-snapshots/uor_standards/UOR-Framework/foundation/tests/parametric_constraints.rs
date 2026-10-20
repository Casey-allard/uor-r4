//! v0.2.2 Phase D (Q4) integration test: parametric constraint surface.
//!
//! Verifies that:
//! - The seven legacy type aliases (ResidueConstraint, HammingConstraint,
//!   DepthConstraint, CarryConstraint, SiteConstraint, AffineConstraint,
//!   CompositeConstraint) exist and compile over the parametric
//!   `BoundConstraint<O, B>` / `Conjunction<N>` carriers.
//! - The per-type-alias `pub const fn new` constructors produce the
//!   expected `(observable, shape, args)` triple.
//! - The sealed `Observable` and `BoundShape` traits carry the correct
//!   ontology IRIs.
//! - Conjunction constructs a fixed-size composition wrapper.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use uor_foundation::enforcement::{
    AffineConstraint, AffineEqualBound, BoundArgValue, BoundArguments, BoundShape, CarryConstraint,
    CarryDepthObservable, CompositeConstraint, Conjunction, DepthConstraint,
    DerivationDepthObservable, FreeRankObservable, HammingConstraint, HammingMetric, LessEqBound,
    Observable, ResidueClassBound, ResidueConstraint, SiteConstraint, ValueModObservable,
};

#[test]
fn observable_iris_are_correct() {
    assert_eq!(
        ValueModObservable::IRI,
        "https://uor.foundation/observable/ValueModObservable"
    );
    assert_eq!(
        HammingMetric::IRI,
        "https://uor.foundation/observable/HammingMetric"
    );
    assert_eq!(
        DerivationDepthObservable::IRI,
        "https://uor.foundation/derivation/DerivationDepthObservable"
    );
    assert_eq!(
        CarryDepthObservable::IRI,
        "https://uor.foundation/carry/CarryDepthObservable"
    );
    assert_eq!(
        FreeRankObservable::IRI,
        "https://uor.foundation/partition/FreeRankObservable"
    );
}

#[test]
fn bound_shape_iris_are_correct() {
    assert_eq!(
        ResidueClassBound::IRI,
        "https://uor.foundation/type/ResidueClassBound"
    );
    assert_eq!(LessEqBound::IRI, "https://uor.foundation/type/LessEqBound");
    assert_eq!(
        AffineEqualBound::IRI,
        "https://uor.foundation/type/AffineEqualBound"
    );
}

#[test]
fn residue_constraint_constructor_produces_expected_triple() {
    let c: ResidueConstraint = ResidueConstraint::new(256, 42);
    assert_eq!(c.observable().iri_for_instance(), ValueModObservable::IRI);
    assert_eq!(c.bound().iri_for_instance(), ResidueClassBound::IRI);
    let args = c.args().entries();
    assert!(matches!(
        args[0],
        Some(e) if e.name == "modulus" && matches!(e.value, BoundArgValue::U64(256))
    ));
    assert!(matches!(
        args[1],
        Some(e) if e.name == "residue" && matches!(e.value, BoundArgValue::U64(42))
    ));
}

#[test]
fn hamming_constraint_roundtrip() {
    let c: HammingConstraint = HammingConstraint::new(3);
    assert_eq!(c.observable().iri_for_instance(), HammingMetric::IRI);
    assert_eq!(c.bound().iri_for_instance(), LessEqBound::IRI);
    assert!(matches!(
        c.args().entries()[0],
        Some(e) if e.name == "bound" && matches!(e.value, BoundArgValue::U64(3))
    ));
}

#[test]
fn depth_carry_site_affine_constructors_compile_and_produce_expected_triples() {
    let depth: DepthConstraint = DepthConstraint::new(1, 5);
    assert_eq!(
        depth.observable().iri_for_instance(),
        DerivationDepthObservable::IRI
    );
    assert_eq!(depth.bound().iri_for_instance(), LessEqBound::IRI);

    let carry: CarryConstraint = CarryConstraint::new(2);
    assert_eq!(
        carry.observable().iri_for_instance(),
        CarryDepthObservable::IRI
    );

    let site: SiteConstraint = SiteConstraint::new(7);
    assert_eq!(
        site.observable().iri_for_instance(),
        FreeRankObservable::IRI
    );

    let affine: AffineConstraint = AffineConstraint::new(13);
    assert_eq!(
        affine.observable().iri_for_instance(),
        ValueModObservable::IRI
    );
    assert_eq!(affine.bound().iri_for_instance(), AffineEqualBound::IRI);
}

#[test]
fn conjunction_and_composite_alias_compile() {
    let conj: CompositeConstraint<3> = Conjunction::<3>::new(3);
    assert_eq!(conj.len(), 3);
    assert!(!conj.is_empty());
}

#[test]
fn bound_arguments_empty_has_no_entries() {
    let empty = BoundArguments::empty();
    assert!(empty.entries().iter().all(Option::is_none));
}

// Extension trait: the foundation's Observable trait makes `IRI` an
// associated const, which can be accessed via `::IRI` on the type. The
// test above uses this pattern. Trait-level `iri_for_instance` is
// provided here so tests can reference `observable().iri_for_instance()`
// without naming the type — the trait is sealed so this impl must live
// in the test file but it doesn't need visibility outside.
trait IriAccess {
    fn iri_for_instance(&self) -> &'static str;
}

impl<T: Observable> IriAccess for T {
    fn iri_for_instance(&self) -> &'static str {
        T::IRI
    }
}

trait BoundShapeIriAccess {
    fn iri_for_instance(&self) -> &'static str;
}

impl<T: BoundShape> BoundShapeIriAccess for T {
    fn iri_for_instance(&self) -> &'static str {
        T::IRI
    }
}
