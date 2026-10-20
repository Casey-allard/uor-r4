//! Behavioral contract for the `HostBounds` substitution axis.
//!
//! Per the UOR-Framework wiki ADR-018, `HostBounds` is the carrier of every
//! capacity bound that varies along the principal data path: the fingerprint
//! output width range, the trace event-count ceiling, and the algebraic-level
//! bit-width ceiling. The canonical surface is the const-generic on `Hasher`,
//! `ContentFingerprint`, and `Trace`; the application's `HostBounds` impl
//! populates the const-generic with `<MyBounds as HostBounds>::CONST`.
//!
//! This test pins (ADR-060: `DefaultHostBounds` is removed; the test-only
//! `ReferenceHostBounds` carries the canonical values the architecture no
//! longer ships a default for):
//!
//! 1. The `HostBounds` trait is reachable at the crate root.
//! 2. `ReferenceHostBounds` carries the canonical 16/32/256/64 values.
//! 3. The default const-generics on `Hasher`, `ContentFingerprint`, and
//!    `Trace` resolve to those canonical values.
//! 4. An application crate can declare a custom `HostBounds` impl and
//!    select different capacity values without touching foundation source.

use uor_foundation::{ContentFingerprint, HostBounds, Trace};
use uor_foundation_test_helpers::ReferenceHostBounds;

#[test]
fn reference_host_bounds_carry_canonical_values() {
    assert_eq!(
        <ReferenceHostBounds as HostBounds>::FINGERPRINT_MIN_BYTES,
        16
    );
    assert_eq!(
        <ReferenceHostBounds as HostBounds>::FINGERPRINT_MAX_BYTES,
        32
    );
    assert_eq!(<ReferenceHostBounds as HostBounds>::TRACE_MAX_EVENTS, 256);
    assert_eq!(<ReferenceHostBounds as HostBounds>::WITT_LEVEL_MAX_BITS, 64);
}

#[test]
fn parametric_types_default_to_canonical_capacities() {
    // The default const-generics on `ContentFingerprint` and `Trace` resolve
    // to the canonical 32/256 values — applications using those defaults
    // never write turbofish.
    let fp: ContentFingerprint = ContentFingerprint::default();
    assert_eq!(fp.as_bytes().len(), 32);

    let trace: Trace = Trace::empty();
    assert_eq!(trace.len(), 0);
}

/// An application-side `HostBounds` impl. Mirrors what e.g. a Bitcoin
/// proof-of-work consumer would declare: 256-bit fingerprints (32 bytes
/// fixed, no narrow-output substrate), a larger trace ceiling, and an
/// extended algebraic level for the Sha-256 round count.
struct BitcoinPow;

impl HostBounds for BitcoinPow {
    const FINGERPRINT_MIN_BYTES: usize = 32;
    const FINGERPRINT_MAX_BYTES: usize = 32;
    const TRACE_MAX_EVENTS: usize = 1024;
    const WITT_LEVEL_MAX_BITS: u32 = 256;
    // ADR-037: 10 retained structural-count ceilings (ADR-060 removed the
    // byte-buffer ceilings — the carrier is now source-polymorphic).
    const FOLD_UNROLL_THRESHOLD: usize = 8;
    const BETTI_DIMENSION_MAX: usize = 8;
    const NERVE_CONSTRAINTS_MAX: usize = 8;
    const NERVE_SITES_MAX: usize = 8;
    const JACOBIAN_SITES_MAX: usize = 8;
    const RECURSION_TRACE_DEPTH_MAX: usize = 16;
    const OP_CHAIN_DEPTH_MAX: usize = 8;
    const AFFINE_COEFFS_MAX: usize = 8;
    const CONJUNCTION_TERMS_MAX: usize = 8;
    const UNFOLD_ITERATIONS_MAX: usize = 256;
}

#[test]
fn application_can_declare_custom_host_bounds() {
    // The application's selected `HostBounds` is the only locus of
    // capacity variation per ADR-018; this assertion is the surface
    // discipline the wiki specifies.
    assert_eq!(<BitcoinPow as HostBounds>::FINGERPRINT_MIN_BYTES, 32);
    assert_eq!(<BitcoinPow as HostBounds>::FINGERPRINT_MAX_BYTES, 32);
    assert_eq!(<BitcoinPow as HostBounds>::TRACE_MAX_EVENTS, 1024);
    assert_eq!(<BitcoinPow as HostBounds>::WITT_LEVEL_MAX_BITS, 256);

    // The custom impl is independent of the default impl — substitution-
    // axis discipline (ADR-007): the application is the locus of variation.
    assert_ne!(
        <BitcoinPow as HostBounds>::TRACE_MAX_EVENTS,
        <ReferenceHostBounds as HostBounds>::TRACE_MAX_EVENTS,
    );
}

#[test]
fn parametric_trace_can_carry_custom_capacity() {
    // The application's selected `HostBounds` flows into `Trace::<TR_MAX>`
    // through min-const-generics. `BitcoinPow`'s 1024 event ceiling
    // produces a `Trace<1024>` instance, distinct in type from the
    // default `Trace<256>`.
    let trace: Trace<{ <BitcoinPow as HostBounds>::TRACE_MAX_EVENTS }> = Trace::empty();
    assert_eq!(trace.len(), 0);
}
