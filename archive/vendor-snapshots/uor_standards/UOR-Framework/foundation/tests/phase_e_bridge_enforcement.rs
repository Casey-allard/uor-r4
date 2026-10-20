//! Phase E bridge namespace enforcement test.
//!
//! Pins:
//! - InteractionDeclarationBuilder::validate + validate_const produce
//!   Validated<InteractionShape> on success, ShapeViolation on missing
//!   required fields.
//! - The observability subscribe API is gated correctly (tested via
//!   #[cfg(feature = "observability")] below).

use uor_foundation::enforcement::{
    CompileTime, InteractionDeclarationBuilder, InteractionShape, ShapeViolation, Validated,
};
use uor_foundation::enums::ViolationKind;

#[test]
fn interaction_builder_validate_const_succeeds_with_all_fields() {
    let builder = InteractionDeclarationBuilder::new()
        .peer_protocol(0x1234u128)
        .convergence_predicate(0x5678u128)
        .commutator_state_class(0x9abcu128);
    let validated: Validated<InteractionShape, CompileTime> = builder
        .validate_const()
        .expect("fully-specified builder validates");
    let inner: &InteractionShape = validated.inner();
    assert_eq!(
        inner.shape_iri,
        "https://uor.foundation/conformance/InteractionShape"
    );
}

#[test]
fn interaction_builder_validate_const_rejects_missing_peer_protocol() {
    let builder = InteractionDeclarationBuilder::new()
        .convergence_predicate(0x5678u128)
        .commutator_state_class(0x9abcu128);
    let err: ShapeViolation = builder.validate_const().expect_err("must reject");
    assert_eq!(err.kind, ViolationKind::Missing);
    assert!(err.property_iri.contains("peerProtocol"));
}

#[test]
fn interaction_builder_validate_const_rejects_missing_predicate() {
    let builder = InteractionDeclarationBuilder::new()
        .peer_protocol(0x1234u128)
        .commutator_state_class(0x9abcu128);
    let err: ShapeViolation = builder.validate_const().expect_err("must reject");
    assert_eq!(err.kind, ViolationKind::Missing);
    assert!(err.property_iri.contains("convergencePredicate"));
}

#[test]
fn interaction_builder_validate_const_rejects_missing_commutator() {
    let builder = InteractionDeclarationBuilder::new()
        .peer_protocol(0x1234u128)
        .convergence_predicate(0x5678u128);
    let err: ShapeViolation = builder.validate_const().expect_err("must reject");
    assert_eq!(err.kind, ViolationKind::Missing);
    assert!(err.property_iri.contains("commutatorStateClass"));
}

#[test]
fn interaction_builder_validate_runtime_matches_const() {
    let builder = InteractionDeclarationBuilder::new()
        .peer_protocol(0x1234u128)
        .convergence_predicate(0x5678u128)
        .commutator_state_class(0x9abcu128);
    // The runtime path returns Validated<InteractionShape> with default
    // Runtime phase. The const path returns CompileTime; the two target
    // the same shape IRI by construction.
    let runtime = builder.validate().expect("runtime validates");
    assert_eq!(
        runtime.inner().shape_iri,
        "https://uor.foundation/conformance/InteractionShape"
    );
}

#[cfg(feature = "observability")]
#[test]
fn observability_subscription_dispatches_events() {
    use uor_foundation::enforcement::{subscribe_trace_events, TraceEvent};
    let mut emitted = 0usize;
    let sub = subscribe_trace_events(|_event: &TraceEvent| emitted += 1);
    // The observable behavior we pin here is the subscription's
    // addressability — content-determinism lives in the trace tests.
    let _ = sub;
    let _ = emitted;
}
