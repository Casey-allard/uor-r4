// @codegen-exempt — Phase 16 hand-written per-class observable view newtypes.
// emit::write_file's banner check preserves this file across `uor-crate` runs.
//
// Phase 11 emitted blanket Observable<H> + leaf-trait impls directly on
// `Validated<T, Phase>`, but Rust forbids multiple `Observable<H>`
// impls per type — the bare `value()` returned `H::EMPTY_DECIMAL` for
// every kind, hiding the kind-specific scalar from callers.
//
// Phase 16 introduces five per-kind newtype wrappers, each with its
// own `Observable<H>::value()` body delegating to the appropriate
// primitive. The bare `Validated<T, Phase>` no longer impls
// `Observable<H>` — consumers reach for an explicit view via the
// inherent accessors `Validated::as_landauer()`, `as_jacobian()`,
// `as_carry_depth()`, `as_derivation_depth()`, `as_free_rank()`.
//
// Coherence with Phase 7 + Phase 8:
//   - `impl {Foo}<H> for Null{Foo}<H>`               — Phase 7 (resolver-absent)
//   - `impl {Foo}<H> for Resolved{Foo}<'r, R, H>`     — Phase 8 (content-addressed)
//   - `impl {Foo}<H> for Validated{Foo}View<T, Phase>` — Phase 16 (primitive-backed)
// `Null{Foo}<H>`, `Resolved{Foo}<'r, R, H>`, and the Validated*View
// newtypes are mutually disjoint concrete types, so each impl closes
// the orphan without overlapping.

#![allow(clippy::module_name_repetitions)]

use crate::bridge::derivation::DerivationDepthObservable;
use crate::bridge::observable::{JacobianObservable, LandauerBudget, Observable, ThermoObservable};
use crate::bridge::partition::FreeRankObservable;
use crate::enforcement::{Validated, ValidationPhase};
use crate::enums::MeasurementUnit;
use crate::kernel::carry::CarryDepthObservable;
use crate::pipeline::ConstrainedTypeShape;
use crate::{DecimalTranscendental, HostTypes};

// ── Per-class observable views ─────────────────────────────────────
//
// Each view is a zero-cost newtype carrying `PhantomData<(fn() -> T,
// Phase)>` so the wrapper is `Send + Sync` regardless of `T` and
// works with `T: ?Sized`. Construct via the inherent
// `Validated::as_*` accessors.

/// Observable view: Landauer-cost projection of `Validated<T, Phase>`.
/// `value()` returns the Landauer-cost in nats (delegates to
/// `landauer_nats(self)`); the leaf trait `LandauerBudget<H>` is
/// implemented with the same primitive-derived value.
pub struct ValidatedLandauerView<T, Phase: ValidationPhase>(
    core::marker::PhantomData<(fn() -> T, Phase)>,
);

/// Observable view: per-site Jacobian projection of `Validated<T, Phase>`.
/// `value()` returns the L1-norm of the Jacobian row computed by
/// `primitive_curvature_jacobian::<T>()`.
pub struct ValidatedJacobianView<T, Phase: ValidationPhase>(
    core::marker::PhantomData<(fn() -> T, Phase)>,
);

/// Observable view: carry-depth projection of `Validated<T, Phase>`.
/// `value()` returns the dihedral orbit size from
/// `primitive_dihedral_signature::<T>()`.
pub struct ValidatedCarryDepthView<T, Phase: ValidationPhase>(
    core::marker::PhantomData<(fn() -> T, Phase)>,
);

/// Observable view: derivation-depth projection of `Validated<T, Phase>`.
/// `value()` returns the reduction-step count from
/// `primitive_terminal_reduction::<T>(W8)`.
pub struct ValidatedDerivationDepthView<T, Phase: ValidationPhase>(
    core::marker::PhantomData<(fn() -> T, Phase)>,
);

/// Observable view: free-rank projection of `Validated<T, Phase>`.
/// `value()` returns the free-rank residual from
/// `primitive_descent_metrics::<T>(&nerve_betti).0`.
pub struct ValidatedFreeRankView<T, Phase: ValidationPhase>(
    core::marker::PhantomData<(fn() -> T, Phase)>,
);

// Manual trait impls — `derive` would propagate spurious bounds
// (`T: Eq`, `T: Hash`, etc.) onto consumers' shape markers. The
// PhantomData inside is unconditionally `Copy + Eq + Hash + Default`,
// so the impls hold for every `T: ?Sized` and `Phase: ValidationPhase`.

macro_rules! impl_view_traits {
    ($name:ident, $label:literal) => {
        impl<T, Phase: ValidationPhase> core::fmt::Debug for $name<T, Phase> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str($label)
            }
        }
        impl<T, Phase: ValidationPhase> Default for $name<T, Phase> {
            #[inline]
            fn default() -> Self {
                Self(core::marker::PhantomData)
            }
        }
        impl<T, Phase: ValidationPhase> Clone for $name<T, Phase> {
            #[inline]
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<T, Phase: ValidationPhase> Copy for $name<T, Phase> {}
        impl<T, Phase: ValidationPhase> PartialEq for $name<T, Phase> {
            #[inline]
            fn eq(&self, _other: &Self) -> bool {
                true
            }
        }
        impl<T, Phase: ValidationPhase> Eq for $name<T, Phase> {}
        impl<T, Phase: ValidationPhase> core::hash::Hash for $name<T, Phase> {
            #[inline]
            fn hash<S: core::hash::Hasher>(&self, _state: &mut S) {}
        }
    };
}

impl_view_traits!(ValidatedLandauerView, "ValidatedLandauerView");
impl_view_traits!(ValidatedJacobianView, "ValidatedJacobianView");
impl_view_traits!(ValidatedCarryDepthView, "ValidatedCarryDepthView");
impl_view_traits!(ValidatedDerivationDepthView, "ValidatedDerivationDepthView");
impl_view_traits!(ValidatedFreeRankView, "ValidatedFreeRankView");

// Constructors — all `pub const fn new()` returning `PhantomData::default`.

impl<T, Phase: ValidationPhase> ValidatedLandauerView<T, Phase> {
    /// Construct a zero-cost Landauer view. Equivalent to
    /// `Validated::<T, Phase>::as_landauer()`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<T, Phase: ValidationPhase> ValidatedJacobianView<T, Phase> {
    /// Construct a zero-cost Jacobian view. Equivalent to
    /// `Validated::<T, Phase>::as_jacobian()`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<T, Phase: ValidationPhase> ValidatedCarryDepthView<T, Phase> {
    /// Construct a zero-cost carry-depth view. Equivalent to
    /// `Validated::<T, Phase>::as_carry_depth()`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<T, Phase: ValidationPhase> ValidatedDerivationDepthView<T, Phase> {
    /// Construct a zero-cost derivation-depth view. Equivalent to
    /// `Validated::<T, Phase>::as_derivation_depth()`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<T, Phase: ValidationPhase> ValidatedFreeRankView<T, Phase> {
    /// Construct a zero-cost free-rank view. Equivalent to
    /// `Validated::<T, Phase>::as_free_rank()`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

// Inherent accessors on `Validated<T, Phase>` to construct each view
// without forcing callers to spell out the newtype path.

impl<T, Phase: ValidationPhase> Validated<T, Phase>
where
    T: ConstrainedTypeShape,
{
    /// Phase 16 — Landauer-cost view of `self`.
    #[inline]
    #[must_use]
    pub const fn as_landauer(&self) -> ValidatedLandauerView<T, Phase> {
        ValidatedLandauerView::new()
    }

    /// Phase 16 — Jacobian view of `self`.
    #[inline]
    #[must_use]
    pub const fn as_jacobian(&self) -> ValidatedJacobianView<T, Phase> {
        ValidatedJacobianView::new()
    }

    /// Phase 16 — carry-depth view of `self`.
    #[inline]
    #[must_use]
    pub const fn as_carry_depth(&self) -> ValidatedCarryDepthView<T, Phase> {
        ValidatedCarryDepthView::new()
    }

    /// Phase 16 — derivation-depth view of `self`.
    #[inline]
    #[must_use]
    pub const fn as_derivation_depth(&self) -> ValidatedDerivationDepthView<T, Phase> {
        ValidatedDerivationDepthView::new()
    }

    /// Phase 16 — free-rank view of `self`.
    #[inline]
    #[must_use]
    pub const fn as_free_rank(&self) -> ValidatedFreeRankView<T, Phase> {
        ValidatedFreeRankView::new()
    }
}

// ── Observable<H> + leaf-trait impls per view ──────────────────────

// Default Observable supertrait helpers — every view returns the same
// host-empty reference for source/target and the enum default for
// has_unit. Only `value()` differs per view.
#[inline]
fn empty_source<H: HostTypes>() -> &'static H::HostString {
    H::EMPTY_HOST_STRING
}

#[inline]
fn empty_target<H: HostTypes>() -> &'static H::HostString {
    H::EMPTY_HOST_STRING
}

// ── ValidatedLandauerView — primitive-backed entropy_nats ──────────
impl<T, Phase, H> Observable<H> for ValidatedLandauerView<T, Phase>
where
    T: ConstrainedTypeShape,
    Phase: ValidationPhase,
    H: HostTypes,
{
    #[inline]
    fn value(&self) -> H::Decimal {
        <Self as LandauerBudget<H>>::landauer_nats(self)
    }
    #[inline]
    fn source(&self) -> &H::HostString {
        empty_source::<H>()
    }
    #[inline]
    fn target(&self) -> &H::HostString {
        empty_target::<H>()
    }
    #[inline]
    fn has_unit(&self) -> MeasurementUnit {
        MeasurementUnit::default()
    }
}

impl<T, Phase, H> ThermoObservable<H> for ValidatedLandauerView<T, Phase>
where
    T: ConstrainedTypeShape,
    Phase: ValidationPhase,
    H: HostTypes,
{
    #[inline]
    fn hardness_estimate(&self) -> H::Decimal {
        H::EMPTY_DECIMAL
    }
}

impl<T, Phase, H> LandauerBudget<H> for ValidatedLandauerView<T, Phase>
where
    T: ConstrainedTypeShape,
    Phase: ValidationPhase,
    H: HostTypes,
{
    #[inline]
    fn landauer_nats(&self) -> H::Decimal {
        let nerve = match crate::enforcement::primitive_simplicial_nerve_betti::<T>() {
            Ok(b) => b,
            Err(_) => return H::EMPTY_DECIMAL,
        };
        let (_residual, entropy_bits) = crate::enforcement::primitive_descent_metrics::<T>(&nerve);
        <H::Decimal as DecimalTranscendental>::from_bits(entropy_bits)
    }
}

// ── ValidatedJacobianView — L1 of the Jacobian row ─────────────────
impl<T, Phase, H> Observable<H> for ValidatedJacobianView<T, Phase>
where
    T: ConstrainedTypeShape,
    Phase: ValidationPhase,
    H: HostTypes,
{
    #[inline]
    fn value(&self) -> H::Decimal {
        let jac = crate::enforcement::primitive_curvature_jacobian::<T>();
        let mut sum: u64 = 0;
        let mut i = 0;
        while i < jac.len() {
            sum = sum.saturating_add(u64::from(jac[i].unsigned_abs()));
            i += 1;
        }
        <H::Decimal as DecimalTranscendental>::from_u64(sum)
    }
    #[inline]
    fn source(&self) -> &H::HostString {
        empty_source::<H>()
    }
    #[inline]
    fn target(&self) -> &H::HostString {
        empty_target::<H>()
    }
    #[inline]
    fn has_unit(&self) -> MeasurementUnit {
        MeasurementUnit::default()
    }
}

impl<T, Phase, H> JacobianObservable<H> for ValidatedJacobianView<T, Phase>
where
    T: ConstrainedTypeShape,
    Phase: ValidationPhase,
    H: HostTypes,
{
}

// ── ValidatedCarryDepthView — dihedral orbit size ──────────────────
impl<T, Phase, H> Observable<H> for ValidatedCarryDepthView<T, Phase>
where
    T: ConstrainedTypeShape,
    Phase: ValidationPhase,
    H: HostTypes,
{
    #[inline]
    fn value(&self) -> H::Decimal {
        let (orbit_size, _period) = crate::enforcement::primitive_dihedral_signature::<T>();
        <H::Decimal as DecimalTranscendental>::from_u32(orbit_size)
    }
    #[inline]
    fn source(&self) -> &H::HostString {
        empty_source::<H>()
    }
    #[inline]
    fn target(&self) -> &H::HostString {
        empty_target::<H>()
    }
    #[inline]
    fn has_unit(&self) -> MeasurementUnit {
        MeasurementUnit::default()
    }
}

impl<T, Phase, H> CarryDepthObservable<H> for ValidatedCarryDepthView<T, Phase>
where
    T: ConstrainedTypeShape,
    Phase: ValidationPhase,
    H: HostTypes,
{
}

// ── ValidatedDerivationDepthView — terminal-reduction step count ───
impl<T, Phase, H> Observable<H> for ValidatedDerivationDepthView<T, Phase>
where
    T: ConstrainedTypeShape,
    Phase: ValidationPhase,
    H: HostTypes,
{
    #[inline]
    fn value(&self) -> H::Decimal {
        // W8 (8-bit Witt level) — the standard baseline.
        match crate::enforcement::primitive_terminal_reduction::<T>(8u16) {
            Ok((_witt, length, _sat)) => <H::Decimal as DecimalTranscendental>::from_u32(length),
            Err(_) => H::EMPTY_DECIMAL,
        }
    }
    #[inline]
    fn source(&self) -> &H::HostString {
        empty_source::<H>()
    }
    #[inline]
    fn target(&self) -> &H::HostString {
        empty_target::<H>()
    }
    #[inline]
    fn has_unit(&self) -> MeasurementUnit {
        MeasurementUnit::default()
    }
}

impl<T, Phase, H> DerivationDepthObservable<H> for ValidatedDerivationDepthView<T, Phase>
where
    T: ConstrainedTypeShape,
    Phase: ValidationPhase,
    H: HostTypes,
{
}

// ── ValidatedFreeRankView — free-rank residual ─────────────────────
impl<T, Phase, H> Observable<H> for ValidatedFreeRankView<T, Phase>
where
    T: ConstrainedTypeShape,
    Phase: ValidationPhase,
    H: HostTypes,
{
    #[inline]
    fn value(&self) -> H::Decimal {
        let nerve = match crate::enforcement::primitive_simplicial_nerve_betti::<T>() {
            Ok(b) => b,
            Err(_) => return H::EMPTY_DECIMAL,
        };
        let (residual, _entropy_bits) = crate::enforcement::primitive_descent_metrics::<T>(&nerve);
        <H::Decimal as DecimalTranscendental>::from_u32(residual)
    }
    #[inline]
    fn source(&self) -> &H::HostString {
        empty_source::<H>()
    }
    #[inline]
    fn target(&self) -> &H::HostString {
        empty_target::<H>()
    }
    #[inline]
    fn has_unit(&self) -> MeasurementUnit {
        MeasurementUnit::default()
    }
}

impl<T, Phase, H> FreeRankObservable<H> for ValidatedFreeRankView<T, Phase>
where
    T: ConstrainedTypeShape,
    Phase: ValidationPhase,
    H: HostTypes,
{
}
