//! v0.2.2 Phase Q.3 example: trace-replay round-trip verification.
//!
//! `Derivation::replay()` on a `Grounded<'static, T>`'s derivation produces a sealed
//! `Trace`. `replay::certify_from_trace(&trace)` then re-derives a
//! `Certified<GroundingCertificate>` whose fingerprint matches the source
//! grounded value — demonstrating the content-addressed verify-trace
//! round-trip contract.
//!
//! Run with: `cargo run --example verify_trace_roundtrip -p uor-foundation`

use uor_foundation::enforcement::{
    replay, CompileUnitBuilder, ConstrainedTypeInput, Grounded, Term, Validated,
};
use uor_foundation::pipeline::run;
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::{Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

// ADR-060: `Term` carries the inline-carrier width `N` and is no longer `Sync`,
// so the term arena is `const` (not `static`).
const ROOT_TERMS: &[Term<'static, N>] = &[uor_foundation::pipeline::literal_u64(7, WittLevel::W8)];
static DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn main() {
    // Build → validate → run → Grounded<'static, T>.
    let builder = CompileUnitBuilder::new()
        .root_term(ROOT_TERMS)
        .witt_level_ceiling(WittLevel::W32)
        .thermodynamic_budget(2048)
        .target_domains(DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    let unit: Validated<_> = builder.validate().expect("unit well-formed");
    let grounded: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit).expect("pipeline admits");

    // Replay: extract a Trace from the grounded derivation. Type-annotate
    // so `replay::<TR_MAX>` infers `TR_MAX` from `Trace`'s default
    // const-generic. ADR-060 removed `DefaultHostBounds`; applications declare
    // their own `HostBounds` (here the test-only `ReferenceHostBounds`, whose
    // `TRACE_MAX_EVENTS` backs `Trace`'s default const-generic).
    let trace: uor_foundation::Trace = grounded.derivation().replay();
    println!(
        "Trace: {} event(s) at witt_level_bits={}",
        trace.len(),
        trace.witt_level_bits()
    );

    // Verify: re-certify from the trace and compare fingerprints.
    let recertified = replay::certify_from_trace(&trace).expect("trace is well-formed");

    assert_eq!(
        recertified.certificate().content_fingerprint(),
        grounded.content_fingerprint(),
        "re-certified fingerprint must match the source Grounded's fingerprint"
    );
    assert_eq!(
        recertified.certificate().witt_bits(),
        grounded.witt_level_bits(),
        "re-certified witt_bits must match"
    );
    println!("Round-trip verified: replay → certify_from_trace → identical fingerprint.");
}
