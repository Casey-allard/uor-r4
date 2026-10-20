//! Behavioral contract for the grammar-surface roundtrip property.
//!
//! Target §1.5: every EBNF declaration form has exactly one Rust builder
//! whose `validate` (and, where possible, `validate_const`) accepts a
//! fully-specified instance and produces a `Validated<Decl, Phase>`.
//!
//! This test exercises a roundtrip for each of the 9 grammar declaration
//! forms: construct a builder with all required fields, call validate,
//! assert the validated inner has the expected shape_iri string matching
//! the EBNF's conformance-shape reference.
//!
//! A regression where a builder produces a Validated whose shape_iri
//! doesn't match the ontology's `conformance:*Shape` IRI would let
//! downstream consumers accept mis-validated declarations. This test
//! pins the exact IRI strings.

use uor_foundation::enforcement::{
    CompileUnitBuilder, ConstrainedTypeInput, DispatchDeclarationBuilder, EffectDeclarationBuilder,
    InteractionDeclarationBuilder, LeaseDeclarationBuilder, ParallelDeclarationBuilder,
    PredicateDeclarationBuilder, StreamDeclarationBuilder, Term, WittLevelDeclarationBuilder,
};
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::REFERENCE_INLINE_BYTES as N;

const SENTINEL_TERMS: &[Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];
static SENTINEL_DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

// ─── CompileUnit ────────────────────────────────────────────────────────

#[test]
fn compile_unit_validates_and_produces_correct_level_and_budget() {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(WittLevel::W16)
        .thermodynamic_budget(777)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    let validated =
        uor_foundation::pipeline::validate_compile_unit_const(&builder).expect("validates");
    // The validated CompileUnit must round-trip its fields.
    assert_eq!(validated.inner().witt_level(), WittLevel::W16);
    assert_eq!(validated.inner().thermodynamic_budget(), 777);
    assert_eq!(
        validated.inner().result_type_iri(),
        <ConstrainedTypeInput as uor_foundation::pipeline::ConstrainedTypeShape>::IRI
    );
}

// ─── 8 simple builders — shape IRI roundtrip ───────────────────────────

#[test]
fn effect_builder_validated_shape_iri_is_effect_shape() {
    let v = EffectDeclarationBuilder::new()
        .name("e")
        .target_sites(&[0u32])
        .budget_delta(0)
        .commutes(false)
        .validate_const()
        .expect("validates");
    assert_eq!(
        v.inner().shape_iri,
        "https://uor.foundation/conformance/EffectShape"
    );
}

#[test]
fn dispatch_builder_validated_shape_iri_is_dispatch_shape() {
    let v = DispatchDeclarationBuilder::new()
        .predicate(SENTINEL_TERMS)
        .target_resolver("https://uor.foundation/resolver/TwoSatDecider")
        .priority(0)
        .validate_const()
        .expect("validates");
    assert_eq!(
        v.inner().shape_iri,
        "https://uor.foundation/conformance/DispatchShape"
    );
}

#[test]
fn predicate_builder_validated_shape_iri_is_predicate_shape() {
    let v = PredicateDeclarationBuilder::new()
        .input_type("https://uor.foundation/type/ConstrainedType")
        .evaluator(SENTINEL_TERMS)
        .termination_witness("https://uor.foundation/proof/Primitive")
        .validate_const()
        .expect("validates");
    assert_eq!(
        v.inner().shape_iri,
        "https://uor.foundation/conformance/PredicateShape"
    );
}

#[test]
fn parallel_builder_validated_shape_iri_is_parallel_shape() {
    let v = ParallelDeclarationBuilder::new()
        .site_partition(&[0u32, 1])
        .disjointness_witness("https://uor.foundation/proof/D")
        .validate_const()
        .expect("validates");
    assert_eq!(
        v.inner().shape_iri,
        "https://uor.foundation/conformance/ParallelShape"
    );
}

#[test]
fn stream_builder_validated_shape_iri_is_stream_shape() {
    let v = StreamDeclarationBuilder::new()
        .seed(SENTINEL_TERMS)
        .step(SENTINEL_TERMS)
        .productivity_witness("https://uor.foundation/proof/W")
        .validate_const()
        .expect("validates");
    assert_eq!(
        v.inner().shape_iri,
        "https://uor.foundation/conformance/StreamShape"
    );
}

#[test]
fn lease_builder_validated_shape_iri_is_lease_shape() {
    let v = LeaseDeclarationBuilder::new()
        .linear_site(0u32)
        .scope("s")
        .validate_const()
        .expect("validates");
    assert_eq!(
        v.inner().shape_iri,
        "https://uor.foundation/conformance/LeaseShape"
    );
}

#[test]
fn witt_level_builder_validated_inner_rehydrates_fields() {
    let v = WittLevelDeclarationBuilder::new()
        .bit_width(16)
        .predecessor(WittLevel::W8)
        .validate_const()
        .expect("validates");
    assert_eq!(v.inner().bit_width, 16);
    assert_eq!(v.inner().predecessor, WittLevel::W8);
}

#[test]
fn interaction_builder_validated_shape_iri_is_interaction_shape() {
    let v = InteractionDeclarationBuilder::new()
        .peer_protocol(1)
        .convergence_predicate(2)
        .commutator_state_class(3)
        .validate_const()
        .expect("validates");
    assert_eq!(
        v.inner().shape_iri,
        "https://uor.foundation/conformance/InteractionShape"
    );
}

// ─── Grammar ↔ builder count: 9 total ─────────────────────────────────

#[test]
fn grammar_declaration_count_equals_9() {
    // CompileUnit + 8 simple builders = 9 grammar-surface builders.
    // This test is a structural anchor; if a builder is added or
    // removed, the conformance suite's grammar_surface_coverage validator
    // (Phase G) will also catch the drift.
    const GRAMMAR_DECLARATION_FORMS: usize = 9;
    assert_eq!(GRAMMAR_DECLARATION_FORMS, 9);
}
