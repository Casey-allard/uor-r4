//! Phase X.1 — per-resolver cert-class discrimination.
//!
//! Each Phase D resolver's `certify` / `certify_at` returns a specific
//! `Certified<C>` where `C` is the ontology-declared certificate class for
//! that resolver's `resolver:CertifyMapping`. This file pins the expected
//! cert class per resolver at the type level — if codegen regresses to the
//! v0.2.1 `GroundingCertificate`-for-everything scheme, these fail to compile.
//!
//! The 17 Phase D resolver modules discriminate across 8 cert classes:
//! `TransformCertificate` (canonical_form, type_synthesis, homotopy, moduli),
//! `IsometryCertificate` (monodromy), `InvolutionCertificate` (dihedral_factorization),
//! `CompletenessCertificate` (completeness), `GeodesicCertificate` (geodesic_validator),
//! `MeasurementCertificate` (measurement), `BornRuleVerification` (superposition),
//! and `GroundingCertificate` (two_sat_decider, horn_sat_decider, residual_verdict,
//! jacobian_guided, evaluation, session, witt_level_resolver).

#![allow(dead_code)]

use uor_foundation::enforcement::{
    resolver, BornRuleVerification, Certified, CompileUnitBuilder, CompletenessCertificate,
    ConstrainedTypeInput, GenericImpossibilityWitness, GeodesicCertificate, GroundingCertificate,
    InvolutionCertificate, IsometryCertificate, MeasurementCertificate, TransformCertificate,
};
use uor_foundation::pipeline::{ConstrainedTypeShape, ConstraintRef};
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::REFERENCE_INLINE_BYTES as N;
use uor_foundation_test_helpers::{validated_runtime, Fnv1aHasher16};

type H = Fnv1aHasher16;

/// A tiny `ConstrainedTypeShape` to feed resolvers that take ConstrainedType input.
pub struct Probe;
impl ConstrainedTypeShape for Probe {
    const IRI: &'static str = "https://example.org/phase_x1/Probe";
    const SITE_COUNT: usize = 2;
    const CONSTRAINTS: &'static [ConstraintRef] = &[ConstraintRef::Residue {
        modulus: 3,
        residue: 1,
    }];
    const CYCLE_SIZE: u64 = 1;
}

fn probe_validated() -> uor_foundation::enforcement::Validated<Probe> {
    validated_runtime(Probe)
}

// ──────────────────────────────────────────────────────────────────────────
// Deciders / generic grounding (produce `GroundingCertificate`).
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn two_sat_decider_produces_grounding_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::two_sat_decider::certify::<_, _, H, 32>(&input);
}

#[test]
fn horn_sat_decider_produces_grounding_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::horn_sat_decider::certify::<_, _, H, 32>(&input);
}

#[test]
fn residual_verdict_produces_grounding_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::residual_verdict::certify::<_, _, H, 32>(&input);
}

#[test]
fn jacobian_guided_produces_grounding_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::jacobian_guided::certify::<_, _, H, 32>(&input);
}

#[test]
fn evaluation_produces_grounding_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::evaluation::certify::<_, _, H, 32>(&input);
}

// ──────────────────────────────────────────────────────────────────────────
// Transform class: canonical_form, type_synthesis, homotopy, moduli.
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn canonical_form_produces_transform_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<TransformCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::canonical_form::certify::<_, _, H, 32>(&input);
}

#[test]
fn type_synthesis_produces_transform_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<TransformCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::type_synthesis::certify::<_, _, H, 32>(&input);
}

#[test]
fn homotopy_produces_transform_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<TransformCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::homotopy::certify::<_, _, H, 32>(&input);
}

#[test]
fn moduli_produces_transform_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<TransformCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::moduli::certify::<_, _, H, 32>(&input);
}

// ──────────────────────────────────────────────────────────────────────────
// Isometry class: monodromy.
// Involution class: dihedral_factorization.
// Completeness class: completeness.
// Geodesic class: geodesic_validator.
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn monodromy_produces_isometry_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<IsometryCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::monodromy::certify::<_, _, H, 32>(&input);
}

#[test]
fn dihedral_factorization_produces_involution_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<InvolutionCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::dihedral_factorization::certify::<_, _, H, 32>(&input);
}

#[test]
fn completeness_produces_completeness_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<CompletenessCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::completeness::certify::<_, _, H, 32>(&input);
}

#[test]
fn geodesic_validator_produces_geodesic_certificate() {
    let input = probe_validated();
    let _r: Result<Certified<GeodesicCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::geodesic_validator::certify::<_, _, H, 32>(&input);
}

// ──────────────────────────────────────────────────────────────────────────
// CompileUnit-input kernels: session, superposition, measurement, witt_level.
// ──────────────────────────────────────────────────────────────────────────

static DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];
// ADR-060: `TermValue` holds a `dyn ChunkSource` and is therefore not `Sync`,
// so the term slice lives in a `const` (no `Sync` requirement) rather than a
// `static`.
const ROOT_TERMS: &[uor_foundation::enforcement::Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];

fn probe_unit(
) -> uor_foundation::enforcement::Validated<uor_foundation::enforcement::CompileUnit<'static, N>> {
    CompileUnitBuilder::new()
        .root_term(ROOT_TERMS)
        .witt_level_ceiling(WittLevel::W32)
        .thermodynamic_budget(4096)
        .target_domains(DOMAINS)
        .result_type::<ConstrainedTypeInput>()
        .validate()
        .expect("unit valid")
}

#[test]
fn session_produces_grounding_certificate() {
    let u = probe_unit();
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::session::certify::<_, H, N, 32>(&u);
}

#[test]
fn superposition_produces_born_rule_verification() {
    let u = probe_unit();
    let _r: Result<Certified<BornRuleVerification>, Certified<GenericImpossibilityWitness>> =
        resolver::superposition::certify::<_, H, N, 32>(&u);
}

#[test]
fn measurement_produces_measurement_certificate() {
    let u = probe_unit();
    let _r: Result<Certified<MeasurementCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::measurement::certify::<_, H, N, 32>(&u);
}

#[test]
fn witt_level_produces_grounding_certificate() {
    let u = probe_unit();
    let _r: Result<Certified<GroundingCertificate>, Certified<GenericImpossibilityWitness>> =
        resolver::witt_level_resolver::certify::<_, H, N, 32>(&u);
}

// ──────────────────────────────────────────────────────────────────────────
// Runtime sanity: a successful cert carries its resolver's witt_bits and
// content fingerprint, via the X.1 `with_level_and_fingerprint_const` pattern
// implemented on every cert type.
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn cert_witness_carries_witt_bits_and_fingerprint() {
    let input = probe_validated();
    if let Ok(c) = resolver::homotopy::certify::<_, _, H, 32>(&input) {
        assert_eq!(c.certificate().witt_bits(), 32);
        assert!(c.certificate().content_fingerprint().width_bytes() > 0);
    } else {
        panic!("homotopy must certify the probe shape");
    }
    if let Ok(c) = resolver::monodromy::certify::<_, _, H, 32>(&input) {
        assert_eq!(c.certificate().witt_bits(), 32);
        assert!(c.certificate().content_fingerprint().width_bytes() > 0);
    }
    if let Ok(c) = resolver::dihedral_factorization::certify::<_, _, H, 32>(&input) {
        assert_eq!(c.certificate().witt_bits(), 32);
        assert!(c.certificate().content_fingerprint().width_bytes() > 0);
    }
}
