//! Behavioral contract for pipeline driver content-determinism and
//! input-dependence.
//!
//! Target §1.7 + §5: every driver (`run`, `run_const`, `run_parallel`,
//! `run_stream`, `run_interactive`) is content-deterministic. Two calls on
//! equal inputs must produce witnesses whose every observable is equal;
//! two calls on unequal inputs must differ on at least one observable
//! (usually `content_fingerprint` + `unit_address`).
//!
//! A regression where the pipeline accidentally introduces non-determinism
//! (wall-clock time, RNG, uninitialized memory) would break verify-trace
//! round-trips. This test pins the contract.

use uor_foundation::enforcement::{
    CompileTime, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput, Grounded,
    IntegerGroundingMap, Term, Validated,
};
use uor_foundation::pipeline::{
    run, run_const, run_parallel, run_stream, validate_compile_unit_const, ParallelDeclaration,
    StreamDeclaration, StreamDriver,
};
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_test_helpers::{validated_runtime, Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

const SENTINEL_TERMS: &[Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];
static SENTINEL_DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn build(level: WittLevel, budget: u64) -> Validated<CompileUnit<'static, N>, CompileTime> {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(level)
        .thermodynamic_budget(budget)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    validate_compile_unit_const(&builder).expect("fixture")
}

// ─── pipeline::run determinism ──────────────────────────────────────────

#[test]
fn pipeline_run_equal_inputs_produce_equal_grounded() {
    let a: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(build(WittLevel::W32, 100))
            .expect("a");
    let b: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(build(WittLevel::W32, 100))
            .expect("b");
    assert_eq!(a.witt_level_bits(), b.witt_level_bits());
    assert_eq!(a.unit_address(), b.unit_address());
    assert_eq!(a.content_fingerprint(), b.content_fingerprint());
    assert_eq!(a.uor_time(), b.uor_time());
}

#[test]
fn pipeline_run_differing_budgets_differ_on_fingerprint() {
    let a: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(build(WittLevel::W32, 100))
            .expect("a");
    let b: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(build(WittLevel::W32, 200))
            .expect("b");
    assert_ne!(
        a.content_fingerprint(),
        b.content_fingerprint(),
        "different budgets must produce different content_fingerprints"
    );
    assert_ne!(
        a.unit_address(),
        b.unit_address(),
        "different budgets must produce different unit_addresses"
    );
}

#[test]
fn pipeline_run_differing_witt_levels_differ_on_witt_bits() {
    let a: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(build(WittLevel::W8, 100)).expect("a");
    let b: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(build(WittLevel::W32, 100))
            .expect("b");
    assert_ne!(a.witt_level_bits(), b.witt_level_bits());
    assert_ne!(a.content_fingerprint(), b.content_fingerprint());
}

// ─── pipeline::run_const determinism ────────────────────────────────────

#[test]
fn pipeline_run_const_equal_inputs_produce_equal_grounded() {
    let unit = build(WittLevel::W16, 77);
    let a: Grounded<'static, ConstrainedTypeInput, N> =
        run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher16, N, 32>(&unit)
            .expect("a");
    let b: Grounded<'static, ConstrainedTypeInput, N> =
        run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher16, N, 32>(&unit)
            .expect("b");
    assert_eq!(a.content_fingerprint(), b.content_fingerprint());
    assert_eq!(a.uor_time(), b.uor_time());
    assert_eq!(a.triad(), b.triad());
}

#[test]
fn pipeline_run_and_run_const_agree_for_same_unit() {
    // run and run_const should produce identical witnesses for compile-time-
    // validated inputs (modulo the `Grounding<Map>` marker which is
    // type-level only). The fingerprint is substrate-driven by the Hasher
    // alone, so they match.
    let unit = build(WittLevel::W16, 55);
    // run takes Validated<_, P> by value; clone the validated wrapper via
    // re-building to avoid Phase mismatch.
    let unit_for_run = build(WittLevel::W16, 55);
    let a: Grounded<'static, ConstrainedTypeInput, N> =
        run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher16, N, 32>(&unit)
            .expect("const");
    let b: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit_for_run).expect("runtime");
    assert_eq!(
        a.content_fingerprint(),
        b.content_fingerprint(),
        "run_const and run must produce the same fingerprint for equal CompileUnits"
    );
}

// ─── pipeline::run_parallel input-dependence ────────────────────────────

// v0.2.2 Phase H3: `ParallelDeclaration::new::<T>(site_count)` was deleted; the
// sole constructor is `new_with_partition`. These fixtures supply a partition
// slice whose length equals the intended site count and a canonical
// disjointness-witness IRI.
const DISJOINTNESS_WITNESS: &str = "https://uor.foundation/parallel/ParallelDisjointnessWitness";

#[test]
fn pipeline_run_parallel_different_site_counts_produce_different_witnesses() {
    static PARTITION_3: &[u32] = &[0, 1, 2];
    static PARTITION_7: &[u32] = &[0, 1, 2, 3, 4, 5, 6];
    let unit_3: Validated<ParallelDeclaration> =
        validated_runtime(ParallelDeclaration::new_with_partition::<
            ConstrainedTypeInput,
        >(PARTITION_3, DISJOINTNESS_WITNESS));
    let unit_7: Validated<ParallelDeclaration> =
        validated_runtime(ParallelDeclaration::new_with_partition::<
            ConstrainedTypeInput,
        >(PARTITION_7, DISJOINTNESS_WITNESS));
    let g_3: Grounded<'static, ConstrainedTypeInput, N> =
        run_parallel::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit_3).expect("3");
    let g_7: Grounded<'static, ConstrainedTypeInput, N> =
        run_parallel::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit_7).expect("7");
    assert_ne!(
        g_3.unit_address(),
        g_7.unit_address(),
        "different site counts must produce different unit_addresses"
    );
}

#[test]
fn pipeline_run_parallel_zero_site_count_is_rejected() {
    // v0.2.2 Phase H3: with `new_with_partition` as the sole constructor, the
    // zero-site case is an empty partition slice — rejected as inadmissible.
    static EMPTY: &[u32] = &[];
    let zero_site: Validated<ParallelDeclaration> =
        validated_runtime(ParallelDeclaration::new_with_partition::<
            ConstrainedTypeInput,
        >(EMPTY, DISJOINTNESS_WITNESS));
    let result = run_parallel::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(zero_site);
    assert!(
        result.is_err(),
        "run_parallel must reject empty partition as inadmissible"
    );
}

#[test]
fn pipeline_run_parallel_equal_site_counts_produce_equal_witnesses() {
    static PARTITION_5: &[u32] = &[0, 1, 2, 3, 4];
    let u_a: Validated<ParallelDeclaration> =
        validated_runtime(ParallelDeclaration::new_with_partition::<
            ConstrainedTypeInput,
        >(PARTITION_5, DISJOINTNESS_WITNESS));
    let u_b: Validated<ParallelDeclaration> =
        validated_runtime(ParallelDeclaration::new_with_partition::<
            ConstrainedTypeInput,
        >(PARTITION_5, DISJOINTNESS_WITNESS));
    let g_a = run_parallel::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(u_a).expect("a");
    let g_b = run_parallel::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(u_b).expect("b");
    assert_eq!(g_a.unit_address(), g_b.unit_address());
    assert_eq!(g_a.content_fingerprint(), g_b.content_fingerprint());
}

// ─── StreamDriver step-distinctness contract ────────────────────────────

#[test]
fn stream_driver_successive_steps_have_distinct_unit_addresses() {
    let unit: Validated<StreamDeclaration<N>> =
        validated_runtime(StreamDeclaration::new::<ConstrainedTypeInput>(3));
    let mut driver: StreamDriver<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32> = run_stream(unit);
    // Pull three steps; assert successive unit_addresses differ.
    let first = driver.next().expect("step 1").expect("ok");
    let second = driver.next().expect("step 2").expect("ok");
    assert_ne!(
        first.unit_address(),
        second.unit_address(),
        "successive StreamDriver steps must produce distinct unit_addresses"
    );
}

#[test]
fn stream_driver_terminates_after_productivity_bound_steps() {
    // Productivity bound = 2 → step 1 Some, step 2 Some, step 3 None.
    let unit: Validated<StreamDeclaration<N>> =
        validated_runtime(StreamDeclaration::new::<ConstrainedTypeInput>(2));
    let mut driver: StreamDriver<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32> = run_stream(unit);
    assert!(driver.next().is_some(), "step 1");
    assert!(driver.next().is_some(), "step 2");
    assert!(
        driver.next().is_none(),
        "step 3 must be None (productivity bound reached)"
    );
}
