//! Phase X.3 — const-path certify entry points for Phase D resolvers.
//!
//! Every Phase D resolver whose composition is built from const-eligible
//! primitives accepts `Validated<T, CompileTime>` (i.e., const-phase validation
//! witness) and returns a `Certified<Kernel::Cert>` discriminated per X.1.
//! Two resolvers — `measurement` and `superposition` — are const-ineligible
//! because `primitive_measurement_projection` uses `f64` arithmetic that is
//! not available in const context at Rust MSRV 1.81.
//!
//! This test exercises the const path by constructing a CompileTime-phase
//! `Validated<Probe, CompileTime>` via `validate_constrained_type_const` and
//! threading it through each const-eligible resolver's `certify` entry.

#![allow(dead_code, clippy::unwrap_used)]

use uor_foundation::enforcement::{
    resolver, BornRuleVerification, Certified, CompletenessCertificate,
    GenericImpossibilityWitness, GeodesicCertificate, GroundingCertificate, InvolutionCertificate,
    IsometryCertificate, MeasurementCertificate, TransformCertificate, Validated,
};
use uor_foundation::pipeline::{
    validate_constrained_type_const, ConstrainedTypeShape, ConstraintRef,
};
use uor_foundation_test_helpers::Fnv1aHasher16;

type H = Fnv1aHasher16;

/// A const-admissible shape: `Residue`, `Site`, and `Carry` variants admit at
/// compile time (they avoid the `LandauerCost` → `f64::from_bits` path).
#[derive(Copy, Clone)]
pub struct Probe;

impl ConstrainedTypeShape for Probe {
    const IRI: &'static str = "https://example.org/phase_x3/Probe";
    const SITE_COUNT: usize = 2;
    const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Residue {
        modulus: 5,
        residue: 2,
    }];
    const CYCLE_SIZE: u64 = 1;
}

/// Compile-time-validated probe. This const item proves the const path
/// admits `Probe` — if `validate_constrained_type_const` regressed it would
/// fail to compile rather than at test runtime.
const COMPILE_TIME_PROBE: Validated<Probe, uor_foundation::enforcement::CompileTime> =
    match validate_constrained_type_const(Probe) {
        Ok(v) => v,
        Err(_) => panic!("probe admissible at const time"),
    };

// ──────────────────────────────────────────────────────────────────────────
// 11 ConstrainedType-input kernels accepting CompileTime-phase witnesses.
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn two_sat_decider_accepts_compile_time_phase() {
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::two_sat_decider::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn horn_sat_decider_accepts_compile_time_phase() {
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::horn_sat_decider::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn residual_verdict_accepts_compile_time_phase() {
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::residual_verdict::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn canonical_form_accepts_compile_time_phase() {
    let _r: Result<Certified<TransformCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::canonical_form::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn type_synthesis_accepts_compile_time_phase() {
    let _r: Result<Certified<TransformCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::type_synthesis::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn homotopy_accepts_compile_time_phase() {
    let _r: Result<Certified<TransformCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::homotopy::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn monodromy_accepts_compile_time_phase() {
    let _r: Result<Certified<IsometryCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::monodromy::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn moduli_accepts_compile_time_phase() {
    let _r: Result<Certified<TransformCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::moduli::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn jacobian_guided_accepts_compile_time_phase() {
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::jacobian_guided::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn evaluation_accepts_compile_time_phase() {
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::evaluation::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn dihedral_factorization_accepts_compile_time_phase() {
    let _r: Result<Certified<InvolutionCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::dihedral_factorization::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn completeness_accepts_compile_time_phase() {
    let _r: Result<Certified<CompletenessCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::completeness::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

#[test]
fn geodesic_validator_accepts_compile_time_phase() {
    let _r: Result<Certified<GeodesicCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::geodesic_validator::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

// ──────────────────────────────────────────────────────────────────────────
// Runtime behavior: distinct shapes → distinct fingerprints on the const path.
// ──────────────────────────────────────────────────────────────────────────

#[derive(Copy, Clone)]
pub struct Probe2;
impl ConstrainedTypeShape for Probe2 {
    const IRI: &'static str = "https://example.org/phase_x3/Probe2";
    const SITE_COUNT: usize = 4;
    const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Site { position: 1 }];
    const CYCLE_SIZE: u64 = 1;
}

const PROBE2: Validated<Probe2, uor_foundation::enforcement::CompileTime> =
    match validate_constrained_type_const(Probe2) {
        Ok(v) => v,
        Err(_) => panic!("probe2 admissible"),
    };

#[test]
fn const_path_preserves_shape_discrimination() {
    let a = resolver::homotopy::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE).unwrap();
    let b = resolver::homotopy::certify::<_, _, H, 32>(&PROBE2).unwrap();
    assert_ne!(
        a.certificate().content_fingerprint(),
        b.certificate().content_fingerprint(),
        "distinct shapes must yield distinct fingerprints on the const path"
    );
}

/// Compile-time marker verifying const-path resolvers return the right types.
/// Uses impossible-to-silence `_` bindings so any regression surfaces as a
/// compile error rather than a test-time type mismatch.
fn _compile_time_type_check() {
    // The type annotations are load-bearing.
    let _: Result<Certified<TransformCertificate>, _> =
        resolver::homotopy::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
    let _: Result<Certified<IsometryCertificate>, _> =
        resolver::monodromy::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
    let _: Result<Certified<InvolutionCertificate>, _> =
        resolver::dihedral_factorization::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
    let _: Result<Certified<CompletenessCertificate>, _> =
        resolver::completeness::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
    let _: Result<Certified<GeodesicCertificate>, _> =
        resolver::geodesic_validator::certify::<_, _, H, 32>(&COMPILE_TIME_PROBE);
}

/// Exempt types: ensure imports resolve (MeasurementCertificate / BornRuleVerification
/// are excluded from the const-phase path because their runtime bodies depend on
/// `primitive_measurement_projection` (f64)).
fn _const_ineligible_are_still_visible() {
    let _: Option<MeasurementCertificate> = None;
    let _: Option<BornRuleVerification> = None;
}
