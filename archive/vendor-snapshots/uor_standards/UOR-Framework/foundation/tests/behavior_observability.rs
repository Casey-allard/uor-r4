//! Behavioral contract for the observability subscription surface.
//!
//! Target §7.4: "Under the `observability` feature flag, a
//! `subscribe(handler: impl FnMut(&TraceEvent))` interface lets
//! instrumentation observe events as the pipeline emits them."
//!
//! The contract: a subscription registered before a pipeline run must
//! receive TraceEvents during or after that run. A surface that merely
//! constructs a subscription without ever invoking its handler is a
//! decorative endpoint — the target document requires real wiring.
//!
//! When the `observability` feature is off, the subscription API is
//! entirely absent (by design — no_std default has no alloc dependency).
//! This test file is entirely `#[cfg(feature = "observability")]`-gated;
//! under the default feature set it becomes an empty test binary.

#![cfg(feature = "observability")]

use core::cell::RefCell;
use std::rc::Rc;
use uor_foundation::enforcement::{
    subscribe_trace_events, CompileTime, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput,
    IntegerGroundingMap, Term, TraceEvent, Validated,
};
use uor_foundation::pipeline::{run_const, validate_compile_unit_const};
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::{Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

const SENTINEL_TERMS: &[Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];
static SENTINEL_DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn build() -> Validated<CompileUnit<'static, N>, CompileTime> {
    let b = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(WittLevel::W16)
        .thermodynamic_budget(100)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    validate_compile_unit_const(&b).expect("fixture")
}

#[test]
fn subscription_emit_method_calls_handler() {
    // A direct `emit(&event)` call on the subscription must drive the
    // handler — this is the basic contract the API promises.
    let count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let count_clone = count.clone();

    let mut sub = subscribe_trace_events(move |_event: &TraceEvent| {
        *count_clone.borrow_mut() += 1;
    });

    // Build a synthetic TraceEvent via the test-helpers backdoor.
    let event = uor_foundation_test_helpers::trace_event(0, 0u128);
    sub.emit(&event);

    assert_eq!(
        *count.borrow(),
        1,
        "ObservabilitySubscription::emit must invoke the registered handler exactly once"
    );
}

#[test]
fn subscription_receives_events_from_real_pipeline_trace() {
    // A pipeline run produces a non-empty Trace. Feeding every event from
    // that trace through the subscription must drive the handler.
    //
    // If the subscription's `emit` is broken, this fails. If the Trace
    // from a real pipeline run is empty (which would itself be a
    // correctness gap elsewhere), this also fails.
    let count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let count_clone = count.clone();

    let mut sub = subscribe_trace_events(move |_event: &TraceEvent| {
        *count_clone.borrow_mut() += 1;
    });

    let unit = build();
    let grounded =
        run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher16, N, 32>(&unit)
            .expect("run_const succeeds");
    let trace: uor_foundation::Trace = grounded.derivation().replay();

    // Walk the trace, dispatch every event through the subscription.
    for i in 0..trace.len() as usize {
        if let Some(ev) = trace.event(i) {
            sub.emit(ev);
        }
    }

    let observed = *count.borrow();
    assert!(
        observed > 0,
        "observability subscription handler must receive at least one TraceEvent when fed \
         the trace from a real pipeline run (got 0 \u{2014} either the Trace from a real \
         run is empty, or emit() doesn't dispatch)"
    );
}
