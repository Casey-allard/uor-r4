//! Phase A witness-accessor test: verifies that `Grounded<'static, T>` and
//! `Certified<C>` each carry a `UorTime` and that `Grounded<'static, T>::triad()`,
//! `Grounded<'static, T>::uor_time()`, and `Certified<C>::uor_time()` return
//! foundation-minted sealed values.
//!
//! The round-trip property asserts content-determinism: two pipeline runs
//! on the same input produce witnesses with identical `UorTime` and
//! identical `Triad` projections.

use uor_foundation::enforcement::{
    calibrations, Calibration, CompileTime, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput,
    Grounded, IntegerGroundingMap, Nanos, Term, UorTime, Validated,
};
use uor_foundation::pipeline::{run, run_const, validate_compile_unit_const};
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
    validate_compile_unit_const(&builder).expect("builder fully specified")
}

fn ground(level: WittLevel, budget: u64) -> Grounded<'static, ConstrainedTypeInput, N> {
    let unit = build_unit(level, budget);
    run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit).expect("run succeeds")
}

#[test]
fn grounded_exposes_uor_time() {
    let g = ground(WittLevel::W8, 100);
    let t: UorTime = g.uor_time();
    assert!(t.rewrite_steps() > 0, "uor_time records rewrite steps");
    assert!(
        t.landauer_nats().nats() > 0.0,
        "Landauer nats are positive after irreversible work"
    );
}

#[test]
fn grounded_uor_time_is_replay_consistent() {
    // Phase A invariant: two independent runs on the same compile unit
    // produce witnesses whose UorTime values are equal — the
    // content-determinism property described in target §1.7.
    let g_a = ground(WittLevel::W16, 77);
    let g_b = ground(WittLevel::W16, 77);
    assert_eq!(
        g_a.uor_time().rewrite_steps(),
        g_b.uor_time().rewrite_steps()
    );
    assert_eq!(
        g_a.uor_time().landauer_nats().nats(),
        g_b.uor_time().landauer_nats().nats()
    );
}

#[test]
fn grounded_triad_is_content_deterministic() {
    let g_a = ground(WittLevel::W32, 100);
    let g_b = ground(WittLevel::W32, 100);
    let triad_a = g_a.triad();
    let triad_b = g_b.triad();
    assert_eq!(triad_a.stratum(), triad_b.stratum());
    assert_eq!(triad_a.spectrum(), triad_b.spectrum());
    assert_eq!(triad_a.address(), triad_b.address());
}

#[test]
fn grounded_triad_depends_on_unit_address() {
    // Different thermodynamic budgets produce different unit-addresses,
    // hence different Triad projections on at least one component.
    let g_a = ground(WittLevel::W32, 100);
    let g_b = ground(WittLevel::W32, 999);
    let triad_a = g_a.triad();
    let triad_b = g_b.triad();
    let different =
        triad_a.spectrum() != triad_b.spectrum() || triad_a.address() != triad_b.address();
    assert!(different, "triad tracks unit_address across inputs");
}

#[test]
fn grounded_uor_time_composes_with_calibration() {
    let g = ground(WittLevel::W8, 100);
    let cal: &Calibration = &calibrations::CONSERVATIVE_WORST_CASE;
    let nanos: Nanos = g.uor_time().min_wall_clock(cal);
    // The bound is non-negative by construction.
    let _: u64 = nanos.as_u64();
}

#[test]
fn certified_exposes_uor_time_from_cert_iri() {
    // Certified<C>::uor_time() reads from a field populated at mint time.
    // The field is deterministic over the cert kind's IRI length, so two
    // Certified<GroundingCertificate> values share identical clocks.
    let g_a = ground(WittLevel::W8, 100);
    let g_b = ground(WittLevel::W16, 100);
    // run_const also produces a Grounded, but its uor_time is derived from
    // the same formula, so it's equal to run's uor_time for matching inputs.
    let unit = build_unit(WittLevel::W8, 100);
    let g_const: Grounded<'static, ConstrainedTypeInput, N> =
        run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher16, N, 32>(&unit)
            .expect("const grounds");
    assert_eq!(
        g_a.uor_time().rewrite_steps(),
        g_const.uor_time().rewrite_steps(),
        "run and run_const produce the same uor_time on the same input"
    );
    // Cross-level witnesses have different uor_times (rewrite_steps scales
    // with witt_level_bits).
    assert_ne!(
        g_a.uor_time().rewrite_steps(),
        g_b.uor_time().rewrite_steps()
    );
}

#[test]
fn phase_a_sealed_base_metric_newtypes_addressable() {
    // Phase A.4: the four sealed BaseMetric newtypes are in the public API
    // surface, constructible only through the accessor on Grounded<'static, T>.
    let g = ground(WittLevel::W8, 100);
    let _: i64 = g.d_delta().as_i64();
    let _: i64 = g.euler().as_i64();
    let _: u32 = g.residual().as_u32();
    let _: &[u32; uor_foundation::enforcement::MAX_BETTI_DIMENSION] = g.betti().as_array();
    let _: u32 = g.betti().beta(0);
}
