//! Behavioral contract for the route output payload (wiki ADR-028 + ADR-060).
//!
//! Per ADR-028, `Grounded<'static, T>` carries the catamorphism's evaluation
//! result as an output payload alongside the metadata fingerprint.
//! ADR-060 removed the foundation-fixed 4096-byte `ROUTE_OUTPUT_BUFFER_BYTES`
//! / `ROUTE_INPUT_BUFFER_BYTES` ceilings: the carrier is now
//! source-polymorphic and the inline-carrier width is derived from the
//! application's `HostBounds` via `pipeline::carrier_inline_bytes::<B>()`.
//! `Grounded::output_bytes()` still exposes the active prefix.

use uor_foundation::enforcement::{
    CompileTime, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput, Grounded,
    IntegerGroundingMap, Term, Validated,
};
use uor_foundation::pipeline::{carrier_inline_bytes, run_const, validate_compile_unit_const};
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::{
    Fnv1aHasher16, ReferenceHostBounds, REFERENCE_INLINE_BYTES as N,
};

// ADR-060: `Term`/`TermValue` hold a `dyn ChunkSource` and are not `Sync`, so
// the term slice lives in a `const` (no `Sync` requirement), not a `static`.
const SENTINEL_TERMS: &[Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(7, WittLevel::W8)];
static SENTINEL_DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn build_unit() -> Validated<CompileUnit<'static, N>, CompileTime> {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(WittLevel::W32)
        .thermodynamic_budget(200)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    validate_compile_unit_const(&builder).expect("fixture: validates")
}

#[test]
fn carrier_inline_bytes_is_host_bounds_derived() {
    // ADR-060: the inline-carrier width is foundation-derived from the
    // application's `HostBounds` — never a free-standing foundation constant.
    // `REFERENCE_INLINE_BYTES` is the value `carrier_inline_bytes` computes
    // for `ReferenceHostBounds`; pin the two agree so consumers can thread
    // either form interchangeably.
    assert_eq!(carrier_inline_bytes::<ReferenceHostBounds>(), N);
}

#[test]
fn grounded_output_bytes_accessor_is_public() {
    // ADR-028 + ADR-060: `output_bytes()` exposes the `Grounded`'s output-payload
    // prefix carried by the source-polymorphic carrier. The payload is populated
    // by `pipeline::run_route` (the catamorphism call-site that evaluates the
    // route's Term tree); the lower-level `run_const` grounding path leaves it
    // unpopulated, so the accessor returns the empty prefix here. Either way the
    // returned slice must fit within the application's inline carrier width.
    let unit = build_unit();
    let grounded: Grounded<'static, ConstrainedTypeInput, N> =
        run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher16, N, 32>(&unit)
            .expect("fixture: run_const succeeds");
    let out = grounded.output_bytes();
    assert!(
        out.len() <= N,
        "output_bytes() must fit within the inline carrier width N={N} (got {})",
        out.len()
    );
    assert!(
        out.is_empty(),
        "run_const grounding path leaves the output payload unpopulated (got {out:?})"
    );
}
