//! Phase F kernel namespace enforcement test.
//!
//! Pins that the 8 kernel namespaces expose sealed witness types per
//! target §4.7:
//! - `ConstraintKind` enum with 6 variants (closed six-kind per §1.5)
//! - `CarryProfile`, `CarryEvent` (carry)
//! - `ConvergenceLevel<L>` (convergence)
//! - `DivisionAlgebraWitness` enum (division)
//! - `MonoidalProduct<L, R>`, `MonoidalUnit<L>` (monoidal)
//! - `OperadComposition` (operad)
//! - `RecursionTrace` + `RECURSION_TRACE_MAX_DEPTH` (recursion)
//! - `AddressRegion` (region)
//! - `LinearBudget`, `LeaseAllocation` (linear)

use uor_foundation::enforcement::{
    AddressRegion, CarryEvent, CarryProfile, ConstraintKind, ConvergenceLevel,
    DivisionAlgebraWitness, LeaseAllocation, LinearBudget, MonoidalProduct, MonoidalUnit,
    OperadComposition, RecursionTrace, RECURSION_TRACE_MAX_DEPTH, W16, W32, W8,
};

#[test]
fn constraint_kind_enumerates_six_variants() {
    // Each of the six members of `type:ConstraintKind` is addressable.
    let _ = ConstraintKind::Residue;
    let _ = ConstraintKind::Carry;
    let _ = ConstraintKind::Depth;
    let _ = ConstraintKind::Hamming;
    let _ = ConstraintKind::Site;
    let _ = ConstraintKind::Affine;
}

#[test]
fn division_algebra_witness_enumerates_four_variants() {
    // Cayley-Dickson / Hurwitz closure.
    let _ = DivisionAlgebraWitness::Real;
    let _ = DivisionAlgebraWitness::Complex;
    let _ = DivisionAlgebraWitness::Quaternion;
    let _ = DivisionAlgebraWitness::Octonion;
}

#[test]
fn carry_profile_and_event_are_addressable() {
    // CarryProfile / CarryEvent are sealed — constructed only by the
    // foundation. We can only assert type addressability.
    fn _takes_profile(_p: CarryProfile) {}
    fn _takes_event(_e: CarryEvent) {}
}

#[test]
fn convergence_level_is_level_parameterized() {
    fn _takes_w8(_c: ConvergenceLevel<W8>) {}
    fn _takes_w16(_c: ConvergenceLevel<W16>) {}
    fn _takes_w32(_c: ConvergenceLevel<W32>) {}
}

#[test]
fn monoidal_product_and_unit_accept_level_pairs() {
    fn _product(_p: MonoidalProduct<W8, W16>) {}
    fn _unit(_u: MonoidalUnit<W8>) {}
}

#[test]
fn operad_composition_addressable() {
    fn _takes_comp(_c: OperadComposition) {}
}

#[test]
fn recursion_trace_has_fixed_capacity() {
    // RECURSION_TRACE_MAX_DEPTH is a `pub const` — part of the snapshot.
    const _: usize = RECURSION_TRACE_MAX_DEPTH;
    fn _takes_trace(_t: RecursionTrace) {}
}

#[test]
fn region_and_linear_witnesses_addressable() {
    fn _region(_r: AddressRegion) {}
    fn _budget(_b: LinearBudget) {}
    fn _alloc(_a: LeaseAllocation) {}
}
