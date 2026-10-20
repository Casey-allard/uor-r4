//! Trace-replay round-trip — wiki ADR-021 V&V Decision 2.
//!
//! Per the UOR-Framework wiki ADR-021, the trace-replay round-trip is
//! "elevated to a first-class IEEE 1012 V&V activity, not a behavior test
//! fixture: the reading is normative — round-trip closure is an
//! architectural property the implementation MUST satisfy." This file is
//! the normative test of that property.
//!
//! ADR-021 frames Prism's V&V structure as a hylomorphism between two
//! agents: `prism` (here: `uor-foundation`'s pipeline) is the V agent
//! producing the catamorphism's output `Grounded<'static, T>`; `prism-verify`
//! (here: `uor-foundation-verify`) is the IV&V agent producing the
//! anamorphism's output `Certified<GroundingCertificate>`. The trace is
//! the artifact crossing the V/IV&V boundary. The verifier cannot cheat
//! by re-running the catamorphism — `verify_trace`'s contract is to
//! walk the trace structurally, without invoking the application
//! author's deciders or any cryptographic hasher (TC-05). This is
//! independence-by-construction (ADR-021 Decision 3).
//!
//! Each test in this file asserts a concrete outcome on the re-derived
//! `Certified<GroundingCertificate>` or the rejection path:
//!
//! - empty traces are rejected
//! - zero-fingerprint traces are rejected (T5: `FingerprintMissing`)
//! - traces with a real substrate fingerprint round-trip via passthrough
//! - out-of-order traces are rejected
//! - zero-target traces are rejected
//! - non-contiguous step traces are rejected (T5.8: `NonContiguousSteps`)
//! - structurally-distinct traces produce distinct certificates
//! - the round-trip property is deterministic
//! - distinct widths produce distinct fingerprints for the same content

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use uor_foundation::enforcement::{ContentFingerprint, Hasher, Trace};
use uor_foundation_test_helpers::{
    trace_event, trace_with_fingerprint, Fnv1aHasher16, Fnv1aHasher32,
};
use uor_foundation_verify::{verify_trace, ReplayError};

/// Compute a `ContentFingerprint` from a slice of arbitrary bytes via the
/// test-only `Fnv1aHasher16` substrate. Mirrors what the production
/// `pipeline::run::<T, _, H>` path does, but for synthetic test data.
fn fingerprint_16(bytes: &[u8]) -> ContentFingerprint {
    let buffer = Fnv1aHasher16::initial().fold_bytes(bytes).finalize();
    ContentFingerprint::from_buffer(buffer, Fnv1aHasher16::OUTPUT_BYTES as u8)
}

fn fingerprint_32(bytes: &[u8]) -> ContentFingerprint {
    let buffer = Fnv1aHasher32::initial().fold_bytes(bytes).finalize();
    ContentFingerprint::from_buffer(buffer, Fnv1aHasher32::OUTPUT_BYTES as u8)
}

#[test]
fn empty_trace_rejects() {
    match verify_trace(&Trace::<256, 32>::empty()) {
        Err(ReplayError::EmptyTrace) => {}
        other => panic!("expected EmptyTrace, got {other:?}"),
    }
}

// v0.2.2 T6.5: deleted `zero_fingerprint_trace_rejects` — the
// `FingerprintMissing` variant is gone. Under T6.3 (no ZeroHasher) and T6.10
// (no zero-fingerprint Trace constructor), no public path can produce a
// Trace with a zero fingerprint, so the rejection case is unreachable.

#[test]
fn single_event_trace_round_trips() {
    let event = trace_event(0, 0x1234);
    let trace = trace_with_fingerprint(&[event], 8, fingerprint_16(b"single-event"));
    let cert = verify_trace(&trace).expect("single-event trace verifies");
    assert_eq!(cert.certificate().witt_bits(), 8);
    assert!(!cert.certificate().content_fingerprint().is_zero());
}

#[test]
fn monotonic_trace_round_trips() {
    let events = [
        trace_event(0, 0x10),
        trace_event(1, 0x20),
        trace_event(2, 0x30),
    ];
    let fp = fingerprint_16(b"monotonic-trace");
    let trace = trace_with_fingerprint(&events, 16, fp);
    let cert = verify_trace(&trace).expect("monotonic trace verifies");
    assert_eq!(cert.certificate().witt_bits(), 16);
    assert_eq!(cert.certificate().content_fingerprint(), fp);
}

#[test]
fn out_of_order_trace_rejects() {
    let events = [trace_event(5, 0x10), trace_event(2, 0x20)];
    let trace = trace_with_fingerprint(&events, 8, fingerprint_16(b"out-of-order"));
    match verify_trace(&trace) {
        Err(ReplayError::OutOfOrderEvent { .. }) => {}
        other => panic!("expected OutOfOrderEvent, got {other:?}"),
    }
}

#[test]
fn zero_target_trace_rejects() {
    let events = [trace_event(0, 0)];
    let trace = trace_with_fingerprint(&events, 8, fingerprint_16(b"zero-target"));
    match verify_trace(&trace) {
        Err(ReplayError::ZeroTarget { index: 0 }) => {}
        other => panic!("expected ZeroTarget, got {other:?}"),
    }
}

#[test]
fn non_contiguous_steps_trace_rejects() {
    // Steps [0, 5] declares length 2, last_step = 5 → NonContiguousSteps.
    let events = [trace_event(0, 0xAA), trace_event(5, 0xBB)];
    let trace = trace_with_fingerprint(&events, 8, fingerprint_16(b"non-contiguous"));
    match verify_trace(&trace) {
        Err(ReplayError::NonContiguousSteps {
            declared: 2,
            last_step: 5,
        }) => {}
        other => panic!("expected NonContiguousSteps, got {other:?}"),
    }
}

#[test]
fn distinct_traces_produce_distinct_certificates() {
    let events_a = [trace_event(0, 0xAA), trace_event(1, 0xBB)];
    let events_b = [
        trace_event(0, 0x11),
        trace_event(1, 0x22),
        trace_event(2, 0x33),
    ];
    let trace_a = trace_with_fingerprint(&events_a, 8, fingerprint_16(b"trace-a"));
    let trace_b = trace_with_fingerprint(&events_b, 8, fingerprint_16(b"trace-b"));
    let cert_a = verify_trace(&trace_a).expect("trace_a verifies");
    let cert_b = verify_trace(&trace_b).expect("trace_b verifies");
    assert_ne!(
        cert_a.certificate().content_fingerprint(),
        cert_b.certificate().content_fingerprint(),
        "structurally-distinct traces must produce distinct certificates"
    );
}

#[test]
fn certify_from_trace_passes_fingerprint_through_unchanged() {
    let fp = fingerprint_16(b"passthrough-fixture");
    let trace = trace_with_fingerprint(&[trace_event(0, 0x10), trace_event(1, 0x20)], 8, fp);
    let cert = verify_trace(&trace).expect("verifies");
    assert_eq!(cert.certificate().content_fingerprint(), fp);
    assert_eq!(cert.certificate().witt_bits(), 8);
}

#[test]
fn verify_trace_is_deterministic() {
    let events = [trace_event(0, 0x10), trace_event(1, 0x20)];
    let fp = fingerprint_16(b"deterministic");
    let trace = trace_with_fingerprint(&events, 8, fp);
    let cert_a = verify_trace(&trace).expect("a verifies");
    let cert_b = verify_trace(&trace).expect("b verifies");
    assert_eq!(
        cert_a.certificate().content_fingerprint(),
        cert_b.certificate().content_fingerprint(),
        "verify_trace must be deterministic"
    );
}

#[test]
fn distinct_widths_produce_distinct_fingerprints_for_same_content() {
    // Critical: a trace hashed at two widths must produce two
    // ContentFingerprint values that are NOT equal, even if the leading
    // bytes happen to coincide. This is enforced by ContentFingerprint::Eq
    // comparing the full buffer + width tag.
    let events = [trace_event(0, 0x10), trace_event(1, 0x20)];
    let fp16 = fingerprint_16(b"width-test");
    let fp32 = fingerprint_32(b"width-test");
    let trace_16 = trace_with_fingerprint(&events, 8, fp16);
    let trace_32 = trace_with_fingerprint(&events, 8, fp32);
    let cert_16 = verify_trace(&trace_16).expect("16-byte verifies");
    let cert_32 = verify_trace(&trace_32).expect("32-byte verifies");
    assert_eq!(
        cert_16.certificate().content_fingerprint().width_bytes(),
        16
    );
    assert_eq!(
        cert_32.certificate().content_fingerprint().width_bytes(),
        32
    );
    assert_ne!(
        cert_16.certificate().content_fingerprint(),
        cert_32.certificate().content_fingerprint(),
        "different widths must produce different fingerprints"
    );
}
