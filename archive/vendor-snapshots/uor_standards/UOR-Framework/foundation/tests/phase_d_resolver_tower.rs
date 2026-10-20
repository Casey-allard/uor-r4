//! Phase D resolver-tower test: every new resolver module is reachable and
//! produces a concrete verdict.
//!
//! The foundation ships 21 resolver classes total (5 from v0.2.2 +
//! 16 added in Phase D): TwoSatDecider, HornSatDecider, ResidualVerdictResolver,
//! CanonicalFormResolver, TypeSynthesisResolver, HomotopyResolver,
//! MonodromyResolver, ModuliResolver, JacobianGuidedResolver,
//! EvaluationResolver, SessionResolver, SuperpositionResolver,
//! MeasurementResolver, WittLevelResolver, DihedralFactorizationResolver,
//! CompletenessResolver — this test pins each as a reachable module with a
//! `certify` free function that terminates on the vacuous input.

use uor_foundation::enforcement::resolver::{
    canonical_form, completeness, dihedral_factorization, evaluation, homotopy, horn_sat_decider,
    jacobian_guided, measurement, moduli, monodromy, residual_verdict, session, superposition,
    two_sat_decider, type_synthesis, witt_level_resolver,
};
use uor_foundation::enforcement::{
    CompileTime, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput, Term, Validated,
};
use uor_foundation::pipeline::validate_compile_unit_const;
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::REFERENCE_INLINE_BYTES as N;
use uor_foundation_test_helpers::{validated_runtime, Fnv1aHasher16};

// ADR-060: `TermValue` holds a `dyn ChunkSource` and is therefore not `Sync`,
// so the term slice lives in a `const` (no `Sync` requirement) rather than a
// `static`.
const SENTINEL_TERMS: &[Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];
static SENTINEL_DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn build_unit() -> Validated<CompileUnit<'static, N>, CompileTime> {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(WittLevel::W8)
        .thermodynamic_budget(100)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    validate_compile_unit_const(&builder).expect("fixture: validates")
}

/// Macro to generate the per-resolver test for ConstrainedType-consuming
/// resolvers. Each asserts the free function is addressable, admits the
/// vacuous input, and emits a non-zero witt_bits certificate.
macro_rules! ct_resolver_test {
    ($test_name:ident, $module:ident) => {
        #[test]
        fn $test_name() {
            let input = validated_runtime(ConstrainedTypeInput::default());
            let cert = $module::certify::<_, _, Fnv1aHasher16, 32>(&input)
                .expect(concat!(stringify!($module), " must certify vacuous input"));
            assert_ne!(cert.certificate().witt_bits(), 0);
        }
    };
}

ct_resolver_test!(two_sat_decider_certifies, two_sat_decider);
ct_resolver_test!(horn_sat_decider_certifies, horn_sat_decider);
ct_resolver_test!(residual_verdict_certifies, residual_verdict);
ct_resolver_test!(canonical_form_certifies, canonical_form);
ct_resolver_test!(type_synthesis_certifies, type_synthesis);
ct_resolver_test!(homotopy_certifies, homotopy);
ct_resolver_test!(monodromy_certifies, monodromy);
ct_resolver_test!(moduli_certifies, moduli);
ct_resolver_test!(jacobian_guided_certifies, jacobian_guided);
ct_resolver_test!(evaluation_certifies, evaluation);
ct_resolver_test!(dihedral_factorization_certifies, dihedral_factorization);
ct_resolver_test!(completeness_certifies, completeness);

/// Macro to generate the per-resolver test for CompileUnit-consuming
/// resolvers.
macro_rules! cu_resolver_test {
    ($test_name:ident, $module:ident) => {
        #[test]
        fn $test_name() {
            let unit = build_unit();
            let cert = $module::certify::<_, Fnv1aHasher16, N, 32>(&unit)
                .expect(concat!(stringify!($module), " must certify unit"));
            assert_ne!(cert.certificate().witt_bits(), 0);
        }
    };
}

cu_resolver_test!(session_certifies, session);
cu_resolver_test!(superposition_certifies, superposition);
cu_resolver_test!(measurement_certifies, measurement);
cu_resolver_test!(witt_level_resolver_certifies, witt_level_resolver);

/// Content-determinism: two calls on the same input produce identical
/// certs (same witt_bits, same fingerprint).
#[test]
fn canonical_form_is_content_deterministic() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let cert_a = canonical_form::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("a");
    let cert_b = canonical_form::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("b");
    assert_eq!(
        cert_a.certificate().witt_bits(),
        cert_b.certificate().witt_bits()
    );
    assert_eq!(
        cert_a.certificate().content_fingerprint(),
        cert_b.certificate().content_fingerprint()
    );
}

/// Different Witt levels produce different fingerprints.
#[test]
fn canonical_form_fingerprint_is_level_dependent() {
    let input = validated_runtime(ConstrainedTypeInput::default());
    let cert_w8 = canonical_form::certify_at::<_, _, Fnv1aHasher16, 32>(&input, WittLevel::W8)
        .expect("w8 certifies");
    let cert_w32 = canonical_form::certify_at::<_, _, Fnv1aHasher16, 32>(&input, WittLevel::W32)
        .expect("w32 certifies");
    assert_ne!(
        cert_w8.certificate().witt_bits(),
        cert_w32.certificate().witt_bits()
    );
    assert_ne!(
        cert_w8.certificate().content_fingerprint(),
        cert_w32.certificate().content_fingerprint()
    );
}
