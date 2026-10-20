//! Phase C const-fn frontier test: every Phase C builder exposes a
//! `validate_const` companion, the `certify_grounding_aware_const` resolver
//! exists, and `ShapeViolation::const_message` is invocable in const context.

use uor_foundation::enforcement::{
    CompileTime, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput, DispatchDeclaration,
    DispatchDeclarationBuilder, EffectDeclaration, EffectDeclarationBuilder, PredicateDeclaration,
    PredicateDeclarationBuilder, ShapeViolation, Term, Validated, WittLevelDeclaration,
    WittLevelDeclarationBuilder,
};
use uor_foundation::pipeline::{certify_grounding_aware_const, validate_compile_unit_const};
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::{Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

const SENTINEL_TERMS: &[Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];
static SENTINEL_DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn build_compile_unit() -> Validated<CompileUnit<'static, N>, CompileTime> {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(WittLevel::W16)
        .thermodynamic_budget(100)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    validate_compile_unit_const(&builder).expect("fixture: validates")
}

#[test]
fn witt_level_validate_const_succeeds() {
    let builder = WittLevelDeclarationBuilder::new()
        .bit_width(8)
        .predecessor(WittLevel::W8);
    let _: Validated<WittLevelDeclaration, CompileTime> =
        builder.validate_const().expect("witt level validates");
}

#[test]
fn witt_level_validate_const_rejects_missing_bit_width() {
    let builder = WittLevelDeclarationBuilder::new().predecessor(WittLevel::W8);
    let err: ShapeViolation = builder
        .validate_const()
        .expect_err("missing bit_width must violate");
    assert_eq!(
        err.const_message(),
        "https://uor.foundation/conformance/WittLevelShape"
    );
}

#[test]
fn effect_validate_const_succeeds() {
    let builder = EffectDeclarationBuilder::new()
        .name("e")
        .target_sites(&[0u32, 1u32])
        .budget_delta(-1)
        .commutes(true);
    let _: Validated<EffectDeclaration, CompileTime> =
        builder.validate_const().expect("effect validates");
}

#[test]
fn dispatch_validate_const_succeeds() {
    let body: &[Term<N>] = SENTINEL_TERMS;
    let builder = DispatchDeclarationBuilder::new()
        .predicate(body)
        .target_resolver("https://uor.foundation/resolver/TwoSatDecider")
        .priority(0);
    let _: Validated<DispatchDeclaration, CompileTime> =
        builder.validate_const().expect("dispatch validates");
}

#[test]
fn predicate_validate_const_succeeds() {
    let body: &[Term<N>] = SENTINEL_TERMS;
    let builder = PredicateDeclarationBuilder::new()
        .input_type("https://uor.foundation/type/ConstrainedType")
        .evaluator(body)
        .termination_witness("https://uor.foundation/proof/PrimitiveRecursion");
    let _: Validated<PredicateDeclaration, CompileTime> =
        builder.validate_const().expect("predicate validates");
}

#[test]
fn certify_grounding_aware_const_emits_validated_grounding() {
    let unit = build_compile_unit();
    let cert = certify_grounding_aware_const::<ConstrainedTypeInput, Fnv1aHasher16, N, 32>(&unit);
    // The cert is Validated<GroundingCertificate, CompileTime>; its witt_bits
    // matches the source unit's witt_level.
    assert_eq!(cert.inner().witt_bits(), 16);
}

#[test]
fn shape_violation_const_message_is_static_iri() {
    let builder = WittLevelDeclarationBuilder::new();
    let err = builder.validate_const().unwrap_err();
    // const_message is `pub const fn` returning `&'static str` — usable
    // wherever const-context formatting is required.
    let _msg: &'static str = err.const_message();
}
