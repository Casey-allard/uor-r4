//! Behavioral contract for the resolver-tower cert-fingerprint
//! discrimination property.
//!
//! Target §4.2: every `enforcement::resolver::<name>::certify` produces a
//! `Certified<_>` whose `content_fingerprint` is content-deterministic and
//! differs across resolver classes even for the same input. The
//! differentiation comes from each resolver folding a distinct
//! `CertificateKind` discriminant byte into the substrate digest.
//!
//! If two different resolvers emit identical fingerprints for the same
//! input, the cert-class discriminator is broken — a resolver could be
//! impersonated by a different resolver's verdict. This test catches that.
//!
//! Note: this test uses `ConstrainedTypeInput` (the vacuous shim) because
//! `ConstrainedTypeShape` is sealed. Per-resolver decision-procedure
//! differentiation (e.g., TwoSat on real sat vs unsat inputs) requires
//! extending the foundation's sealed shim surface, which is out of scope
//! for the conformance suite. The fingerprint-discrimination contract is
//! what's externally observable.

use uor_foundation::enforcement::{
    resolver, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput, Term, Validated,
};
use uor_foundation::pipeline::validate_compile_unit_const;
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::Fnv1aHasher16;
use uor_foundation_test_helpers::REFERENCE_INLINE_BYTES as N;

// ADR-060: `TermValue` holds a `dyn ChunkSource` and is therefore not `Sync`,
// so the term slice cannot live in a `static`. A `const` array (no `Sync`
// requirement) referenced from a `const` item yields the `'static` borrow the
// builder needs.
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

#[test]
fn constrained_type_resolvers_produce_distinct_fingerprints() {
    // Every ConstrainedType-consuming resolver folds its own
    // CertificateKind into the digest. For the same input, distinct
    // resolvers must produce distinct fingerprints.
    let input = uor_foundation_test_helpers::validated_runtime(ConstrainedTypeInput::default());

    let tower =
        resolver::tower_completeness::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("tower");
    let incr = resolver::incremental_completeness::certify::<_, _, Fnv1aHasher16, 32>(&input)
        .expect("incr");
    let inhab = resolver::inhabitance::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("inhab");
    let two_sat =
        resolver::two_sat_decider::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("two_sat");
    let horn =
        resolver::horn_sat_decider::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("horn");
    let canon =
        resolver::canonical_form::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("canon");
    let homo = resolver::homotopy::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("homotopy");
    let moduli = resolver::moduli::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("moduli");
    let geodesic =
        resolver::geodesic_validator::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("geodesic");

    // Pairwise-distinct fingerprint assertion.
    let fps = [
        (
            "tower_completeness",
            tower.certificate().content_fingerprint(),
        ),
        (
            "incremental_completeness",
            incr.certificate().content_fingerprint(),
        ),
        ("inhabitance", inhab.certificate().content_fingerprint()),
        (
            "two_sat_decider",
            two_sat.certificate().content_fingerprint(),
        ),
        ("horn_sat_decider", horn.certificate().content_fingerprint()),
        ("canonical_form", canon.certificate().content_fingerprint()),
        ("homotopy", homo.certificate().content_fingerprint()),
        ("moduli", moduli.certificate().content_fingerprint()),
        (
            "geodesic_validator",
            geodesic.certificate().content_fingerprint(),
        ),
    ];

    for i in 0..fps.len() {
        for j in (i + 1)..fps.len() {
            assert_ne!(
                fps[i].1, fps[j].1,
                "resolvers `{}` and `{}` produce identical fingerprints for the same input \
                 \u{2014} the CertificateKind discriminator is broken",
                fps[i].0, fps[j].0,
            );
        }
    }
}

#[test]
fn compile_unit_resolvers_produce_distinct_fingerprints() {
    let unit = build_unit();

    let grounding =
        resolver::grounding_aware::certify::<_, Fnv1aHasher16, N, 32>(&unit).expect("grounding");
    let session = resolver::session::certify::<_, Fnv1aHasher16, N, 32>(&unit).expect("session");
    let superp =
        resolver::superposition::certify::<_, Fnv1aHasher16, N, 32>(&unit).expect("superposition");
    let meas =
        resolver::measurement::certify::<_, Fnv1aHasher16, N, 32>(&unit).expect("measurement");
    let witt_l = resolver::witt_level_resolver::certify::<_, Fnv1aHasher16, N, 32>(&unit)
        .expect("witt_level");

    let fps = [
        (
            "grounding_aware",
            grounding.certificate().content_fingerprint(),
        ),
        ("session", session.certificate().content_fingerprint()),
        ("superposition", superp.certificate().content_fingerprint()),
        ("measurement", meas.certificate().content_fingerprint()),
        (
            "witt_level_resolver",
            witt_l.certificate().content_fingerprint(),
        ),
    ];

    for i in 0..fps.len() {
        for j in (i + 1)..fps.len() {
            assert_ne!(
                fps[i].1, fps[j].1,
                "resolvers `{}` and `{}` produce identical fingerprints for the same unit \
                 \u{2014} the CertificateKind discriminator is broken",
                fps[i].0, fps[j].0,
            );
        }
    }
}

#[test]
fn resolvers_are_pure_same_input_same_fingerprint() {
    let input = uor_foundation_test_helpers::validated_runtime(ConstrainedTypeInput::default());
    let a = resolver::two_sat_decider::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("a");
    let b = resolver::two_sat_decider::certify::<_, _, Fnv1aHasher16, 32>(&input).expect("b");
    assert_eq!(
        a.certificate().content_fingerprint(),
        b.certificate().content_fingerprint(),
        "same input + same resolver must produce the same fingerprint"
    );
}

#[test]
fn incremental_completeness_walks_multiple_pages() {
    // Workstream F (v0.2.2 closure): run_incremental_completeness walks a
    // SpectralSequencePage sequence. Distinct target levels produce distinct
    // page counts, so their fingerprints differ (page_index folds into the
    // digest's budget slot).
    let input = uor_foundation_test_helpers::validated_runtime(ConstrainedTypeInput::default());
    let w8 = resolver::incremental_completeness::certify_at::<_, _, Fnv1aHasher16, 32>(
        &input,
        WittLevel::W8,
    )
    .expect("w8 single-page walk");
    let w64 = resolver::incremental_completeness::certify_at::<_, _, Fnv1aHasher16, 32>(
        &input,
        WittLevel::new(64),
    )
    .expect("w64 multi-page walk");
    assert_ne!(
        w8.certificate().content_fingerprint(),
        w64.certificate().content_fingerprint(),
        "target levels differ ⇒ page counts differ ⇒ fingerprints must differ"
    );
}

#[test]
fn resolvers_differ_on_witt_level() {
    // tower_completeness::certify_at with different WittLevels must
    // produce different fingerprints.
    let input = uor_foundation_test_helpers::validated_runtime(ConstrainedTypeInput::default());
    let w8 =
        resolver::tower_completeness::certify_at::<_, _, Fnv1aHasher16, 32>(&input, WittLevel::W8)
            .expect("w8");
    let w32 =
        resolver::tower_completeness::certify_at::<_, _, Fnv1aHasher16, 32>(&input, WittLevel::W32)
            .expect("w32");
    assert_ne!(
        w8.certificate().content_fingerprint(),
        w32.certificate().content_fingerprint(),
        "different WittLevels must produce different fingerprints"
    );
}
