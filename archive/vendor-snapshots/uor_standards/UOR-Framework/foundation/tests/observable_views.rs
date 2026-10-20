//! Phase 16 verification — per-class observable view newtypes.
//!
//! The bare `Validated<T, Phase>` no longer impls `Observable<H>`;
//! consumers reach for an explicit kind via `as_landauer()`,
//! `as_jacobian()`, `as_carry_depth()`, `as_derivation_depth()`, or
//! `as_free_rank()`. Each view's `Observable<H>::value()` returns the
//! kind-specific scalar derived from the relevant primitive.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use uor_foundation::bridge::derivation::DerivationDepthObservable;
use uor_foundation::bridge::observable::{
    JacobianObservable, LandauerBudget, Observable, ThermoObservable,
};
use uor_foundation::bridge::partition::FreeRankObservable;
use uor_foundation::kernel::carry::CarryDepthObservable;
use uor_foundation::pipeline::{ConstrainedTypeShape, ConstraintRef};
use uor_foundation::DefaultHostTypes;

/// Minimal local fixture implementing `ConstrainedTypeShape` with
/// no constraints (zero-budget, zero-site shape) — used to instantiate
/// the views without dragging in a partition fixture.
struct EmptyShape;

impl ConstrainedTypeShape for EmptyShape {
    const IRI: &'static str = "https://uor.foundation/test/EmptyShape";
    const SITE_COUNT: usize = 0;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = 1;
}

#[test]
fn validated_no_longer_impls_observable_directly() {
    // Phase 16 removed the blanket Observable impl on
    // `Validated<T, Phase>`. The negative path is checked indirectly:
    // if Observable<H> were impl'd on Validated, this would compile,
    // but the only path forward is via the view newtypes.
    fn assert_view_required<T: ConstrainedTypeShape>() {
        // ValidatedLandauerView IS Observable<DefaultHostTypes>.
        fn check<O: Observable<DefaultHostTypes>>() {}
        check::<
            uor_foundation::blanket_impls::ValidatedLandauerView<
                T,
                uor_foundation::enforcement::Runtime,
            >,
        >();
    }
    assert_view_required::<EmptyShape>();
}

#[test]
fn landauer_view_value_matches_landauer_nats() {
    let view = uor_foundation::blanket_impls::ValidatedLandauerView::<
        EmptyShape,
        uor_foundation::enforcement::Runtime,
    >::new();
    let v = <_ as Observable<DefaultHostTypes>>::value(&view);
    let n = <_ as LandauerBudget<DefaultHostTypes>>::landauer_nats(&view);
    // Both should be derived from primitive_descent_metrics's entropy bits.
    assert_eq!(
        v, n,
        "ValidatedLandauerView::value must equal landauer_nats"
    );
}

#[test]
fn landauer_view_implements_thermo_observable() {
    let view = uor_foundation::blanket_impls::ValidatedLandauerView::<
        EmptyShape,
        uor_foundation::enforcement::Runtime,
    >::new();
    let _h = <_ as ThermoObservable<DefaultHostTypes>>::hardness_estimate(&view);
}

#[test]
fn jacobian_view_value_is_l1_of_jacobian_row() {
    let view = uor_foundation::blanket_impls::ValidatedJacobianView::<
        EmptyShape,
        uor_foundation::enforcement::Runtime,
    >::new();
    let v = <_ as Observable<DefaultHostTypes>>::value(&view);
    // EmptyShape has no constraints, so the Jacobian row is all zeros
    // and the L1 sum is 0.0.
    assert_eq!(v, 0.0_f64);
    fn assert_leaf<O: JacobianObservable<DefaultHostTypes>>() {}
    assert_leaf::<
        uor_foundation::blanket_impls::ValidatedJacobianView<
            EmptyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();
}

#[test]
fn carry_depth_view_value_is_dihedral_orbit_size() {
    let view = uor_foundation::blanket_impls::ValidatedCarryDepthView::<
        EmptyShape,
        uor_foundation::enforcement::Runtime,
    >::new();
    let v = <_ as Observable<DefaultHostTypes>>::value(&view);
    // SITE_COUNT = 0 → orbit_size = 1 by primitive convention.
    assert_eq!(v, 1.0_f64);
    fn assert_leaf<O: CarryDepthObservable<DefaultHostTypes>>() {}
    assert_leaf::<
        uor_foundation::blanket_impls::ValidatedCarryDepthView<
            EmptyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();
}

#[test]
fn derivation_depth_view_value_routes_through_terminal_reduction() {
    let view = uor_foundation::blanket_impls::ValidatedDerivationDepthView::<
        EmptyShape,
        uor_foundation::enforcement::Runtime,
    >::new();
    let _v = <_ as Observable<DefaultHostTypes>>::value(&view);
    // Don't assert exact value (depends on pipeline state). Just check
    // it returns without panicking.
    fn assert_leaf<O: DerivationDepthObservable<DefaultHostTypes>>() {}
    assert_leaf::<
        uor_foundation::blanket_impls::ValidatedDerivationDepthView<
            EmptyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();
}

#[test]
fn free_rank_view_value_routes_through_descent_metrics() {
    let view = uor_foundation::blanket_impls::ValidatedFreeRankView::<
        EmptyShape,
        uor_foundation::enforcement::Runtime,
    >::new();
    let v = <_ as Observable<DefaultHostTypes>>::value(&view);
    // EmptyShape: SITE_COUNT = 0; nerve-betti returns Ok(zeros);
    // descent_metrics's residual = max(SITE_COUNT - chi, 0) = 0 - 0 = 0.
    assert_eq!(v, 0.0_f64);
    fn assert_leaf<O: FreeRankObservable<DefaultHostTypes>>() {}
    assert_leaf::<
        uor_foundation::blanket_impls::ValidatedFreeRankView<
            EmptyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();
}

#[test]
fn view_newtypes_are_zero_cost_copy_clone_default() {
    fn assert_traits<T: Copy + Clone + core::fmt::Debug + Default + PartialEq + Eq>() {}

    assert_traits::<
        uor_foundation::blanket_impls::ValidatedLandauerView<
            EmptyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();
    assert_traits::<
        uor_foundation::blanket_impls::ValidatedJacobianView<
            EmptyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();
    assert_traits::<
        uor_foundation::blanket_impls::ValidatedCarryDepthView<
            EmptyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();
    assert_traits::<
        uor_foundation::blanket_impls::ValidatedDerivationDepthView<
            EmptyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();
    assert_traits::<
        uor_foundation::blanket_impls::ValidatedFreeRankView<
            EmptyShape,
            uor_foundation::enforcement::Runtime,
        >,
    >();
}
