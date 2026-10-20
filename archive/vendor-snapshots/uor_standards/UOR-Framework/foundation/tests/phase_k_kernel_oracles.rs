//! Phase K: per-resolver semantic oracles for the v0.2.2 Phase J primitives.
//!
//! The 17 Phase D kernels each compose an ontology primitive into their
//! `certify_at` body. `behavior_resolver_tower` asserts pairwise-distinct
//! fingerprints across kernels for a single input — the baseline discipline.
//! This file asserts **primitive-level contracts** visible through the public
//! kernel API:
//!
//! 1. **Determinism**: every kernel's `certify_at(input, level)` is a pure
//!    function of `(input_shape, level)`; re-running at the same `(input, level)`
//!    yields a bit-identical fingerprint.
//!
//! 2. **Level-dependence**: Phase J's fold pipeline threads `level.witt_length()`
//!    into every cert via `fold_unit_digest`; distinct levels yield distinct
//!    fingerprints per resolver.
//!
//! 3. **Composition-specific discrimination**: resolvers that compose different
//!    primitives produce different fingerprints even at the same level and
//!    input — i.e., the primitive fold contribution is real. Specifically:
//!    - `homotopy` (full Betti tuple) vs. `moduli` (bidegree-(0,1,2) deformation
//!      projection: H^0 automorphisms, H^1 deformations, H^2 obstructions) must
//!      differ because moduli folds only the first three Betti numbers explicitly.
//!    - `monodromy` (SimplicialNerve + Dihedral) differs from both homotopy and
//!      dihedral_factorization (Dihedral alone).
//!
//! Input-variation oracles (e.g., "circle nerve → Betti (1,1,0)") would require
//! additional `ConstrainedTypeShape` shim types beyond `ConstrainedTypeInput`;
//! those are sealed in v0.2.2, so input-variation coverage is pairwise-distinctness
//! across resolver classes at a single input, which `behavior_resolver_tower`
//! asserts exhaustively.

use uor_foundation::enforcement::{
    resolver, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput, Term, Validated,
};
use uor_foundation::pipeline::validate_compile_unit_const;
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::{validated_runtime, Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

const SENTINEL_TERMS: &[Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];
static SENTINEL_DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn build_unit() -> Validated<CompileUnit<'static, N>, uor_foundation::enforcement::CompileTime> {
    let b = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(WittLevel::W32)
        .thermodynamic_budget(100)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    validate_compile_unit_const(&b).expect("fixture")
}

// ─── Determinism ────────────────────────────────────────────────────────

#[test]
fn homotopy_is_deterministic_across_calls() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let a = resolver::homotopy::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("a");
    let b = resolver::homotopy::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("b");
    assert_eq!(
        a.certificate().content_fingerprint(),
        b.certificate().content_fingerprint(),
    );
}

#[test]
fn measurement_is_deterministic_across_calls() {
    let unit = build_unit();
    let a = resolver::measurement::certify::<_, Fnv1aHasher16, N, 32>(&unit).expect("a");
    let b = resolver::measurement::certify::<_, Fnv1aHasher16, N, 32>(&unit).expect("b");
    assert_eq!(
        a.certificate().content_fingerprint(),
        b.certificate().content_fingerprint(),
    );
}

#[test]
fn dihedral_factorization_is_deterministic_across_calls() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let a =
        resolver::dihedral_factorization::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("a");
    let b =
        resolver::dihedral_factorization::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("b");
    assert_eq!(
        a.certificate().content_fingerprint(),
        b.certificate().content_fingerprint(),
    );
}

// ─── Level-dependence ────────────────────────────────────────────────────

#[test]
fn homotopy_fingerprint_changes_with_level() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let at_w8 = resolver::homotopy::certify_at::<_, _, Fnv1aHasher16, 32>(&input, WittLevel::W8)
        .expect("w8");
    let at_w32 = resolver::homotopy::certify_at::<_, _, Fnv1aHasher16, 32>(&input, WittLevel::W32)
        .expect("w32");
    assert_ne!(
        at_w8.certificate().content_fingerprint(),
        at_w32.certificate().content_fingerprint(),
    );
    assert_eq!(at_w8.certificate().witt_bits(), 8);
    assert_eq!(at_w32.certificate().witt_bits(), 32);
}

#[test]
fn monodromy_fingerprint_changes_with_level() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let at_w8 = resolver::monodromy::certify_at::<_, _, Fnv1aHasher16, 32>(&input, WittLevel::W8)
        .expect("w8");
    let at_w32 = resolver::monodromy::certify_at::<_, _, Fnv1aHasher16, 32>(&input, WittLevel::W32)
        .expect("w32");
    assert_ne!(
        at_w8.certificate().content_fingerprint(),
        at_w32.certificate().content_fingerprint(),
    );
}

// ─── Composition-specific discrimination ─────────────────────────────────
//
// These oracles exercise the per-primitive fold contribution: kernels whose
// composition specs differ by a primitive or a marker byte must produce
// different fingerprints on the same input.

#[test]
fn homotopy_differs_from_moduli() {
    // homotopy: full Betti tuple (all 8 dimensions).
    // moduli:   bidegree-(0,1,2) projection reading H^0/H^1/H^2 explicitly.
    let input = validated_runtime(ConstrainedTypeInput::default());
    let homo = resolver::homotopy::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("homo");
    let moduli = resolver::moduli::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("moduli");
    assert_ne!(
        homo.certificate().content_fingerprint(),
        moduli.certificate().content_fingerprint(),
        "homotopy and moduli share SimplicialNerve but moduli folds an extra 0xB0 marker; \
         fingerprints must differ"
    );
}

#[test]
fn monodromy_differs_from_dihedral_factorization() {
    // monodromy:              SimplicialNerve + DihedralAction.
    // dihedral_factorization: DihedralAction alone.
    let input = validated_runtime(ConstrainedTypeInput::default());
    let mono = resolver::monodromy::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("mono");
    let dihedral = resolver::dihedral_factorization::certify::<_, _, Fnv1aHasher16, 32>(&input)
        .expect("dihedral");
    assert_ne!(
        mono.certificate().content_fingerprint(),
        dihedral.certificate().content_fingerprint(),
        "monodromy folds Betti + Dihedral; dihedral_factorization folds only Dihedral — \
         fingerprints must differ"
    );
}

#[test]
fn jacobian_guided_differs_from_geodesic_validator() {
    // Both compose TerminalReduction + CurvatureReducer; however, geodesic_validator
    // carries CertificateKind::GeodesicValidator while jacobian_guided carries
    // CertificateKind::JacobianGuided. The CertificateKind byte enters
    // fold_unit_digest at the kernel's final fold, so they must still differ.
    let input = validated_runtime(ConstrainedTypeInput::default());
    let jac = resolver::jacobian_guided::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("jac");
    let geo =
        resolver::geodesic_validator::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("geo");
    assert_ne!(
        jac.certificate().content_fingerprint(),
        geo.certificate().content_fingerprint(),
    );
}

#[test]
fn session_differs_from_superposition() {
    // session:       SessionBinding only.
    // superposition: SessionBinding + MeasurementProjection (Born outcome folded).
    let unit = build_unit();
    let session = resolver::session::certify::<_, Fnv1aHasher16, N, 32>(&unit).expect("session");
    let superp =
        resolver::superposition::certify::<_, Fnv1aHasher16, N, 32>(&unit).expect("superp");
    assert_ne!(
        session.certificate().content_fingerprint(),
        superp.certificate().content_fingerprint(),
    );
}

#[test]
fn measurement_differs_from_superposition() {
    // measurement:   MeasurementProjection only.
    // superposition: SessionBinding + MeasurementProjection.
    let unit = build_unit();
    let meas = resolver::measurement::certify::<_, Fnv1aHasher16, N, 32>(&unit).expect("meas");
    let superp =
        resolver::superposition::certify::<_, Fnv1aHasher16, N, 32>(&unit).expect("superp");
    assert_ne!(
        meas.certificate().content_fingerprint(),
        superp.certificate().content_fingerprint(),
    );
}

#[test]
fn canonical_form_differs_from_terminal_reduction_only_kernels() {
    // canonical_form: TerminalReduction + 0xC0 marker byte.
    // two_sat_decider / horn_sat_decider / residual_verdict / evaluation:
    //                 TerminalReduction only (differ only by CertificateKind).
    let input = validated_runtime(ConstrainedTypeInput::default());
    let canon =
        resolver::canonical_form::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("canon");
    let twosat =
        resolver::two_sat_decider::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("two_sat");
    let eval = resolver::evaluation::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("eval");
    assert_ne!(
        canon.certificate().content_fingerprint(),
        twosat.certificate().content_fingerprint(),
    );
    assert_ne!(
        canon.certificate().content_fingerprint(),
        eval.certificate().content_fingerprint(),
    );
}

#[test]
fn completeness_differs_from_homotopy_same_nerve() {
    // completeness: SimplicialNerve + fold χ (Euler char from Betti).
    // homotopy:     SimplicialNerve only.
    let input = validated_runtime(ConstrainedTypeInput::default());
    let comp = resolver::completeness::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("comp");
    let homo = resolver::homotopy::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("homo");
    assert_ne!(
        comp.certificate().content_fingerprint(),
        homo.certificate().content_fingerprint(),
        "completeness folds Euler χ in addition to Betti; homotopy folds only Betti — \
         their composition differs so fingerprints must differ"
    );
}

#[test]
fn type_synthesis_differs_from_homotopy() {
    // type_synthesis: SimplicialNerve + DescentTermination.
    // homotopy:       SimplicialNerve only.
    let input = validated_runtime(ConstrainedTypeInput::default());
    let ts = resolver::type_synthesis::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("ts");
    let homo = resolver::homotopy::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("homo");
    assert_ne!(
        ts.certificate().content_fingerprint(),
        homo.certificate().content_fingerprint(),
    );
}
