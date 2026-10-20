//! Behavioral contract for `Grounded<'static, T>` and `Certified<C>` accessors.
//!
//! Contract (target §1.7, §2, §4.6):
//!
//! - `Grounded<'static, T>` carries six sealed `observable:BaseMetric` accessors
//!   (`d_delta`, `sigma`, `jacobian`, `betti`, `euler`, `residual`) that
//!   return foundation-minted sealed newtypes, not raw primitives.
//! - `Grounded<'static, T>::uor_time()` returns a populated `UorTime` with non-zero
//!   counters after a real `pipeline::run` (zero would indicate a stub).
//! - `Grounded<'static, T>::triad()` is content-deterministic: two witnesses minted
//!   from the same inputs return equal Triads.
//! - `Certified<C>::uor_time()` exists and returns a `UorTime`.
//! - `Grounded<'static, T>::certificate()` returns a `Validated<GroundingCertificate>`
//!   with matching witt_bits.
//! - Two Grounded values minted from the same inputs share every accessor
//!   value (content-determinism).

use uor_foundation::enforcement::{
    CompileTime, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput, Grounded,
    IntegerGroundingMap, Term, Validated, MAX_BETTI_DIMENSION,
};
use uor_foundation::pipeline::{run_const, validate_compile_unit_const};
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::Fnv1aHasher16;
use uor_foundation_test_helpers::REFERENCE_INLINE_BYTES as N;

// ADR-060: `TermValue` holds a `dyn ChunkSource` and is therefore not `Sync`,
// so the term slice lives in a `const` (no `Sync` requirement) rather than a
// `static`.
const SENTINEL_TERMS: &[Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];
static SENTINEL_DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn build_unit(level: WittLevel, budget: u64) -> Validated<CompileUnit<'static, N>, CompileTime> {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(level)
        .thermodynamic_budget(budget)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    validate_compile_unit_const(&builder).expect("fixture: validates")
}

fn ground(level: WittLevel, budget: u64) -> Grounded<'static, ConstrainedTypeInput, N> {
    let unit = build_unit(level, budget);
    run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher16, N, 32>(&unit)
        .expect("fixture: run_const succeeds")
}

#[test]
fn grounded_uor_time_is_populated_after_real_run() {
    let g = ground(WittLevel::W32, 200);
    let t = g.uor_time();
    assert!(
        t.rewrite_steps() > 0,
        "Grounded::uor_time().rewrite_steps() must be > 0 for non-trivial input \
         (got 0 \u{2014} accessor appears to return a Default::default zero value)"
    );
    assert!(
        t.landauer_nats().nats() > 0.0,
        "Grounded::uor_time().landauer_nats().nats() must be > 0 for irreversible work"
    );
}

#[test]
fn grounded_triad_is_content_deterministic_across_runs() {
    let g_a = ground(WittLevel::W32, 100);
    let g_b = ground(WittLevel::W32, 100);
    let t_a = g_a.triad();
    let t_b = g_b.triad();
    assert_eq!(t_a.stratum(), t_b.stratum());
    assert_eq!(t_a.spectrum(), t_b.spectrum());
    assert_eq!(t_a.address(), t_b.address());
}

#[test]
fn grounded_triad_differs_across_distinct_inputs() {
    let g_100 = ground(WittLevel::W32, 100);
    let g_200 = ground(WittLevel::W32, 200);
    let t_100 = g_100.triad();
    let t_200 = g_200.triad();
    let differs = t_100.spectrum() != t_200.spectrum() || t_100.address() != t_200.address();
    assert!(
        differs,
        "triads of distinct inputs must differ on at least one coordinate \
         (got identical: stratum={}, spectrum={}, address={})",
        t_100.stratum(),
        t_100.spectrum(),
        t_100.address()
    );
}

#[test]
fn grounded_certificate_witt_bits_matches_input_level() {
    let g = ground(WittLevel::W32, 100);
    assert_eq!(g.witt_level_bits(), 32);
    // The carried cert's witt_bits must equal the unit's witt level.
    let cert = g.certificate();
    assert_eq!(cert.inner().witt_bits(), 32);
}

#[test]
fn grounded_base_metric_accessors_return_sealed_newtypes() {
    // Type-check at compile time that each accessor returns its sealed
    // newtype (binding to a typed local would fail if the signature
    // changed). Runtime: each accessor returns a coherent value.
    let g = ground(WittLevel::W32, 100);
    // The typed locals are compile-time witnesses: if any accessor's
    // signature changed to a non-newtype primitive, these lines would fail
    // to compile. The runtime asserts below pin the value ranges that the
    // newtypes are supposed to guarantee.
    let _d: i64 = g.d_delta().as_i64();
    let s: f64 = g.sigma().value();
    let _e: i64 = g.euler().as_i64();
    let _r: u32 = g.residual().as_u32();
    let betti = g.betti();
    let bv: [u32; MAX_BETTI_DIMENSION] = *betti.as_array();
    let b0: u32 = betti.beta(0);
    let jl: u16 = g.jacobian().len();
    assert!((0.0..=1.0).contains(&s), "sigma must be in [0, 1]");
    assert_eq!(bv[0], b0, "betti.beta(0) must equal betti.as_array()[0]");
    assert!(
        (jl as usize) <= MAX_BETTI_DIMENSION,
        "jacobian length exceeds MAX_BETTI_DIMENSION (got {jl})"
    );
}

#[test]
fn grounded_base_metrics_differ_across_witt_levels() {
    // betti and residual depend on witt_level_bits, so two different
    // levels must produce non-equal results on at least one metric.
    let g_8 = ground(WittLevel::W8, 100);
    let g_32 = ground(WittLevel::W32, 100);
    let same_betti = g_8.betti().as_array() == g_32.betti().as_array();
    let same_residual = g_8.residual().as_u32() == g_32.residual().as_u32();
    let same_d_delta = g_8.d_delta().as_i64() == g_32.d_delta().as_i64();
    assert!(
        !(same_betti && same_residual && same_d_delta),
        "distinct witt levels must produce different base metrics (got all equal)"
    );
}

#[test]
fn grounded_full_accessor_equality_across_equal_inputs() {
    let g_a = ground(WittLevel::W16, 50);
    let g_b = ground(WittLevel::W16, 50);
    // Every accessor must return the same value for equal inputs.
    assert_eq!(g_a.witt_level_bits(), g_b.witt_level_bits());
    assert_eq!(g_a.unit_address().as_u128(), g_b.unit_address().as_u128());
    assert_eq!(g_a.content_fingerprint(), g_b.content_fingerprint());
    assert_eq!(g_a.d_delta().as_i64(), g_b.d_delta().as_i64());
    assert_eq!(g_a.euler().as_i64(), g_b.euler().as_i64());
    assert_eq!(g_a.residual().as_u32(), g_b.residual().as_u32());
    assert_eq!(g_a.betti().as_array(), g_b.betti().as_array());
    assert_eq!(g_a.sigma().value(), g_b.sigma().value());
    assert_eq!(g_a.uor_time(), g_b.uor_time());
}
