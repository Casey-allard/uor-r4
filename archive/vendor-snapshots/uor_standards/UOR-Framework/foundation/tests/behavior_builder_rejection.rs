//! Behavioral contract for every builder's `validate_const` rejection path.
//!
//! Target §3: each of 9 builders validates against a specific
//! `conformance:*Shape`. Missing any required field must produce a
//! `ShapeViolation` whose `property_iri` exactly matches the ontology's
//! property IRI for that field, and whose `kind` is
//! `ViolationKind::Missing`.
//!
//! A regression where a builder accepts an empty or missing-field input
//! would let invalid declarations through the const-evidence pattern
//! downstream. This test pins the exact property_iri strings so any drift
//! is caught.

use uor_foundation::enforcement::{
    CompileUnitBuilder, ConstrainedTypeInput, DispatchDeclarationBuilder, EffectDeclarationBuilder,
    InteractionDeclarationBuilder, LeaseDeclarationBuilder, ParallelDeclarationBuilder,
    PredicateDeclarationBuilder, ShapeViolation, StreamDeclarationBuilder, Term,
    WittLevelDeclarationBuilder,
};
use uor_foundation::enums::ViolationKind;
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::REFERENCE_INLINE_BYTES as N;

const SENTINEL_TERMS: &[Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];
static SENTINEL_DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn assert_missing(err: ShapeViolation, expected_property_iri: &str) {
    assert_eq!(
        err.kind,
        ViolationKind::Missing,
        "expected ViolationKind::Missing for property {expected_property_iri}, got {:?}",
        err.kind
    );
    assert_eq!(
        err.property_iri, expected_property_iri,
        "expected property_iri `{expected_property_iri}`, got `{}`",
        err.property_iri
    );
}

// ─── CompileUnitBuilder (5 required fields) ──────────────────────────────

#[test]
fn compile_unit_rejects_missing_root_term() {
    // Use the pipeline's const validator (the only const path for CompileUnit).
    let builder: CompileUnitBuilder<'_, N> = CompileUnitBuilder::new()
        .witt_level_ceiling(WittLevel::W8)
        .thermodynamic_budget(100)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    let err = uor_foundation::pipeline::validate_compile_unit_const(&builder)
        .expect_err("missing root_term must be rejected");
    assert_missing(err, "https://uor.foundation/reduction/rootTerm");
}

#[test]
fn compile_unit_rejects_missing_witt_level() {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .thermodynamic_budget(100)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    let err = uor_foundation::pipeline::validate_compile_unit_const(&builder)
        .expect_err("missing witt_level_ceiling must be rejected");
    assert_missing(err, "https://uor.foundation/reduction/unitWittLevel");
}

#[test]
fn compile_unit_rejects_missing_thermodynamic_budget() {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(WittLevel::W8)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    let err = uor_foundation::pipeline::validate_compile_unit_const(&builder)
        .expect_err("missing thermodynamic_budget must be rejected");
    assert_missing(err, "https://uor.foundation/reduction/thermodynamicBudget");
}

#[test]
fn compile_unit_rejects_missing_target_domains() {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(WittLevel::W8)
        .thermodynamic_budget(100)
        .result_type::<ConstrainedTypeInput>();
    let err = uor_foundation::pipeline::validate_compile_unit_const(&builder)
        .expect_err("missing target_domains must be rejected");
    assert_missing(err, "https://uor.foundation/reduction/targetDomains");
}

#[test]
fn compile_unit_rejects_missing_result_type() {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(WittLevel::W8)
        .thermodynamic_budget(100)
        .target_domains(SENTINEL_DOMAINS);
    let err = uor_foundation::pipeline::validate_compile_unit_const(&builder)
        .expect_err("missing result_type must be rejected");
    assert_missing(err, "https://uor.foundation/reduction/resultType");
}

// ─── EffectDeclarationBuilder (4 required fields) ────────────────────────

#[test]
fn effect_rejects_missing_name() {
    let builder = EffectDeclarationBuilder::new()
        .target_sites(&[0u32])
        .budget_delta(-1)
        .commutes(true);
    let err = builder
        .validate_const()
        .expect_err("missing name must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/name");
}

#[test]
fn effect_rejects_missing_target_sites() {
    let builder = EffectDeclarationBuilder::new()
        .name("e")
        .budget_delta(-1)
        .commutes(true);
    let err = builder
        .validate_const()
        .expect_err("missing target_sites must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/target_sites");
}

#[test]
fn effect_rejects_missing_budget_delta() {
    let builder = EffectDeclarationBuilder::new()
        .name("e")
        .target_sites(&[0u32])
        .commutes(true);
    let err = builder
        .validate_const()
        .expect_err("missing budget_delta must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/budget_delta");
}

#[test]
fn effect_rejects_missing_commutes() {
    let builder = EffectDeclarationBuilder::new()
        .name("e")
        .target_sites(&[0u32])
        .budget_delta(-1);
    let err = builder
        .validate_const()
        .expect_err("missing commutes must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/commutes");
}

// ─── DispatchDeclarationBuilder (3 required fields) ──────────────────────

#[test]
fn dispatch_rejects_missing_predicate() {
    let builder: DispatchDeclarationBuilder<'_, N> = DispatchDeclarationBuilder::new()
        .target_resolver("https://uor.foundation/resolver/TwoSatDecider")
        .priority(0);
    let err = builder
        .validate_const()
        .expect_err("missing predicate must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/predicate");
}

#[test]
fn dispatch_rejects_missing_target_resolver() {
    let builder = DispatchDeclarationBuilder::new()
        .predicate(SENTINEL_TERMS)
        .priority(0);
    let err = builder
        .validate_const()
        .expect_err("missing target_resolver must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/target_resolver");
}

#[test]
fn dispatch_rejects_missing_priority() {
    let builder = DispatchDeclarationBuilder::new()
        .predicate(SENTINEL_TERMS)
        .target_resolver("https://uor.foundation/resolver/TwoSatDecider");
    let err = builder
        .validate_const()
        .expect_err("missing priority must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/priority");
}

// ─── PredicateDeclarationBuilder (3 required fields) ─────────────────────

#[test]
fn predicate_rejects_missing_input_type() {
    let builder = PredicateDeclarationBuilder::new()
        .evaluator(SENTINEL_TERMS)
        .termination_witness("https://uor.foundation/proof/Primitive");
    let err = builder
        .validate_const()
        .expect_err("missing input_type must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/input_type");
}

#[test]
fn predicate_rejects_missing_evaluator() {
    let builder: PredicateDeclarationBuilder<'_, N> = PredicateDeclarationBuilder::new()
        .input_type("https://uor.foundation/type/ConstrainedType")
        .termination_witness("https://uor.foundation/proof/Primitive");
    let err = builder
        .validate_const()
        .expect_err("missing evaluator must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/evaluator");
}

#[test]
fn predicate_rejects_missing_termination_witness() {
    let builder = PredicateDeclarationBuilder::new()
        .input_type("https://uor.foundation/type/ConstrainedType")
        .evaluator(SENTINEL_TERMS);
    let err = builder
        .validate_const()
        .expect_err("missing termination_witness must be rejected");
    assert_missing(
        err,
        "https://uor.foundation/conformance/termination_witness",
    );
}

// ─── ParallelDeclarationBuilder (2 required fields) ──────────────────────

#[test]
fn parallel_rejects_missing_site_partition() {
    let builder =
        ParallelDeclarationBuilder::new().disjointness_witness("https://uor.foundation/proof/X");
    let err = builder
        .validate_const()
        .expect_err("missing site_partition must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/site_partition");
}

#[test]
fn parallel_rejects_missing_disjointness_witness() {
    let builder = ParallelDeclarationBuilder::new().site_partition(&[0u32, 1u32]);
    let err = builder
        .validate_const()
        .expect_err("missing disjointness_witness must be rejected");
    assert_missing(
        err,
        "https://uor.foundation/conformance/disjointness_witness",
    );
}

// ─── StreamDeclarationBuilder (3 required fields) ────────────────────────

#[test]
fn stream_rejects_missing_seed() {
    let builder = StreamDeclarationBuilder::new()
        .step(SENTINEL_TERMS)
        .productivity_witness("https://uor.foundation/proof/W");
    let err = builder
        .validate_const()
        .expect_err("missing seed must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/seed");
}

#[test]
fn stream_rejects_missing_step() {
    let builder = StreamDeclarationBuilder::new()
        .seed(SENTINEL_TERMS)
        .productivity_witness("https://uor.foundation/proof/W");
    let err = builder
        .validate_const()
        .expect_err("missing step must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/step");
}

#[test]
fn stream_rejects_missing_productivity_witness() {
    let builder = StreamDeclarationBuilder::new()
        .seed(SENTINEL_TERMS)
        .step(SENTINEL_TERMS);
    let err = builder
        .validate_const()
        .expect_err("missing productivity_witness must be rejected");
    assert_missing(
        err,
        "https://uor.foundation/conformance/productivity_witness",
    );
}

// ─── LeaseDeclarationBuilder (2 required fields) ─────────────────────────

#[test]
fn lease_rejects_missing_linear_site() {
    let builder = LeaseDeclarationBuilder::new().scope("scope-name");
    let err = builder
        .validate_const()
        .expect_err("missing linear_site must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/linear_site");
}

#[test]
fn lease_rejects_missing_scope() {
    let builder = LeaseDeclarationBuilder::new().linear_site(0u32);
    let err = builder
        .validate_const()
        .expect_err("missing scope must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/scope");
}

// ─── WittLevelDeclarationBuilder (2 required fields) ────────────────────

#[test]
fn witt_level_rejects_missing_bit_width() {
    let builder = WittLevelDeclarationBuilder::new().predecessor(WittLevel::W8);
    let err = builder
        .validate_const()
        .expect_err("missing bit_width must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/declaredBitWidth");
}

#[test]
fn witt_level_rejects_missing_predecessor() {
    let builder = WittLevelDeclarationBuilder::new().bit_width(8);
    let err = builder
        .validate_const()
        .expect_err("missing predecessor must be rejected");
    assert_missing(err, "https://uor.foundation/conformance/predecessorLevel");
}

// ─── InteractionDeclarationBuilder (3 required fields) ───────────────────

#[test]
fn interaction_rejects_missing_peer_protocol() {
    let builder = InteractionDeclarationBuilder::new()
        .convergence_predicate(0x2u128)
        .commutator_state_class(0x3u128);
    let err = builder
        .validate_const()
        .expect_err("missing peer_protocol must be rejected");
    assert_missing(err, "https://uor.foundation/interaction/peerProtocol");
}

#[test]
fn interaction_rejects_missing_convergence_predicate() {
    let builder = InteractionDeclarationBuilder::new()
        .peer_protocol(0x1u128)
        .commutator_state_class(0x3u128);
    let err = builder
        .validate_const()
        .expect_err("missing convergence_predicate must be rejected");
    assert_missing(
        err,
        "https://uor.foundation/interaction/convergencePredicate",
    );
}

#[test]
fn interaction_rejects_missing_commutator_state_class() {
    let builder = InteractionDeclarationBuilder::new()
        .peer_protocol(0x1u128)
        .convergence_predicate(0x2u128);
    let err = builder
        .validate_const()
        .expect_err("missing commutator_state_class must be rejected");
    assert_missing(
        err,
        "https://uor.foundation/interaction/commutatorStateClass",
    );
}

// ─── Fully-specified builders must accept ────────────────────────────────

#[test]
fn fully_specified_builders_all_validate() {
    // Smoke test: with every required field set, each builder validates.
    let _ = uor_foundation::pipeline::validate_compile_unit_const(
        &CompileUnitBuilder::new()
            .root_term(SENTINEL_TERMS)
            .witt_level_ceiling(WittLevel::W8)
            .thermodynamic_budget(100)
            .target_domains(SENTINEL_DOMAINS)
            .result_type::<ConstrainedTypeInput>(),
    )
    .expect("fully-specified CompileUnit must validate");

    let _ = EffectDeclarationBuilder::new()
        .name("e")
        .target_sites(&[0u32])
        .budget_delta(0)
        .commutes(false)
        .validate_const()
        .expect("fully-specified Effect must validate");

    let _ = DispatchDeclarationBuilder::new()
        .predicate(SENTINEL_TERMS)
        .target_resolver("https://uor.foundation/resolver/TwoSatDecider")
        .priority(0)
        .validate_const()
        .expect("fully-specified Dispatch must validate");

    let _ = PredicateDeclarationBuilder::new()
        .input_type("https://uor.foundation/type/ConstrainedType")
        .evaluator(SENTINEL_TERMS)
        .termination_witness("https://uor.foundation/proof/Primitive")
        .validate_const()
        .expect("fully-specified Predicate must validate");

    let _ = ParallelDeclarationBuilder::new()
        .site_partition(&[0u32])
        .disjointness_witness("https://uor.foundation/proof/D")
        .validate_const()
        .expect("fully-specified Parallel must validate");

    let _ = StreamDeclarationBuilder::new()
        .seed(SENTINEL_TERMS)
        .step(SENTINEL_TERMS)
        .productivity_witness("https://uor.foundation/proof/W")
        .validate_const()
        .expect("fully-specified Stream must validate");

    let _ = LeaseDeclarationBuilder::new()
        .linear_site(0u32)
        .scope("s")
        .validate_const()
        .expect("fully-specified Lease must validate");

    let _ = WittLevelDeclarationBuilder::new()
        .bit_width(8)
        .predecessor(WittLevel::W8)
        .validate_const()
        .expect("fully-specified WittLevel must validate");

    let _ = InteractionDeclarationBuilder::new()
        .peer_protocol(1)
        .convergence_predicate(2)
        .commutator_state_class(3)
        .validate_const()
        .expect("fully-specified Interaction must validate");
}
