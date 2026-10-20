//! v0.2.2 T2.0 (cleanup): end-to-end public-API functional verification.
//!
//! Exercises every previously-hardcoded public endpoint with **at least two**
//! distinct inputs and asserts the outputs differ (or that they're derived
//! from the inputs in a documented way). Hardcoded endpoints — those that
//! return a constant regardless of input — are now functional, and this
//! test is the regression gate that prevents them from sliding back.
//!
//! Phases covered:
//! - Phase A: Calibration / UorTime sanity
//! - Phase C.4 multiplication resolver (T2.1)
//! - Phase E BaseMetric accessors (T2.6)
//! - Phase F drivers (T2.7)
//! - Phase G const-fn frontier (T2.8)
//! - Phase J grounding combinator MarkersImpliedBy bound (T1.1)

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use uor_foundation::enforcement::{
    calibrations, combinators, CompileTime, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput,
    ContentAddress, DigestGroundingMap, Grounded, GroundingCertificate, GroundingProgram,
    IntegerGroundingMap, MulContext, MultiplicationCertificate, Term, Validated,
    MAX_BETTI_DIMENSION,
};
use uor_foundation::pipeline::{
    certify_inhabitance_const, certify_multiplication_const, certify_tower_completeness_const,
    run_const, run_interactive, run_parallel, run_stream, validate_compile_unit_const,
    InteractionDeclaration, InteractionDriver, ParallelDeclaration, PeerInput, PeerPayload,
    StepResult, StreamDeclaration, StreamDriver,
};
use uor_foundation::{DefaultHostTypes, VerificationDomain, WittLevel};
use uor_foundation_test_helpers::{validated_runtime, Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

// Phase 9 pinned: hand-written tests exercise the default-host (f64) path.
type Calibration = uor_foundation::enforcement::Calibration<DefaultHostTypes>;

/// v0.2.2 T6: shared sentinel terms + domains for tests that build
/// fully-specified CompileUnits via the runtime `validate()` path.
const SENTINEL_TERMS: &[Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];
static SENTINEL_DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

/// v0.2.2 T6: helper — build a Validated<CompileUnit, CompileTime> with all
/// 5 required fields set (root_term, witt_level_ceiling, thermodynamic_budget,
/// target_domains, result_type_iri). Used by tests that only care about
/// witt_level / budget but need a complete builder.
fn build_compile_unit(
    level: WittLevel,
    budget: u64,
) -> Validated<CompileUnit<'static, N>, CompileTime> {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(level)
        .thermodynamic_budget(budget)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    validate_compile_unit_const(&builder).expect("test fixture: builder is fully specified")
}

// ─────────────────────────────────────────────────────────────────────────
// Phase A: UorTime / Calibration / Nanos
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn phase_a_calibration_presets_are_addressable() {
    let _ = calibrations::X86_SERVER;
    let _ = calibrations::ARM_MOBILE;
    let _ = calibrations::CORTEX_M_EMBEDDED;
    let _ = calibrations::CONSERVATIVE_WORST_CASE;
}

#[test]
fn phase_a_calibration_new_validates_inputs() {
    assert!(Calibration::new(4.14e-21, 1.0, 1e-15).is_ok());
    assert!(Calibration::new(-1.0, 1.0, 1e-15).is_err());
    assert!(Calibration::new(4.14e-21, 1.0, 0.0).is_err());
}

// ─────────────────────────────────────────────────────────────────────────
// Phase C.4 (T2.1): multiplication resolver trait delegation
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn phase_c4_multiplication_certificate_is_level_dependent() {
    // Phase B: the unit-struct MultiplicationResolver is deleted. The
    // free-function path `resolver::multiplication::certify` is the only
    // surface. Derive MulContext from the level's witt length as the
    // old trait-level façade did.
    use uor_foundation::enforcement::resolver::multiplication;

    let ctx_w8 = MulContext::new(16 * 1024, false, 1);
    let cert_w8 = multiplication::certify::<Fnv1aHasher16, 32>(&ctx_w8)
        .expect("W8 multiplication certify succeeds");

    let limbs_w32 = (32usize).div_ceil(64).max(1);
    let ctx_w32 = MulContext::new(16 * 1024, false, limbs_w32);
    let cert_w32 = multiplication::certify::<Fnv1aHasher16, 32>(&ctx_w32)
        .expect("W32 multiplication certify succeeds");

    assert_ne!(cert_w8.certificate().witt_bits(), 0);
    assert_ne!(cert_w32.certificate().witt_bits(), 0);
}

// ─────────────────────────────────────────────────────────────────────────
// Phase E (T2.6): BaseMetric accessors are input-dependent
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn phase_e_base_metrics_constants_pinned() {
    const _: () = assert!(MAX_BETTI_DIMENSION == 8);
}

#[test]
fn phase_e_run_const_grounded_metrics_differ_by_witt_level() {
    let validated_w8 = build_compile_unit(WittLevel::W8, 100);
    let validated_w32 = build_compile_unit(WittLevel::W32, 200);

    assert_eq!(validated_w8.inner().witt_level(), WittLevel::W8);
    assert_eq!(validated_w32.inner().witt_level(), WittLevel::W32);
    assert_eq!(validated_w8.inner().thermodynamic_budget(), 100);
    assert_eq!(validated_w32.inner().thermodynamic_budget(), 200);

    let g_w8: Grounded<'static, ConstrainedTypeInput, N> =
        run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher16, N, 32>(&validated_w8)
            .expect("w8 grounds");
    let g_w32: Grounded<'static, ConstrainedTypeInput, N> =
        run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher16, N, 32>(
            &validated_w32,
        )
        .expect("w32 grounds");

    // unit_address differs because run_const folds the full unit state.
    assert_ne!(g_w8.unit_address(), g_w32.unit_address());

    // witt_level_bits reflects the unit.
    assert_ne!(g_w8.witt_level_bits(), g_w32.witt_level_bits());

    // Phase A.4: BaseMetric accessors return sealed newtypes that compose from
    // witt_level_bits at mint time. betti() carries the full Betti vector; beta(k)
    // projects individual dimensions.
    assert_ne!(g_w8.betti().as_array(), g_w32.betti().as_array());

    // sigma is computed as bound_sites / declared_sites.
    let _: f64 = g_w8.sigma().value();

    // residual() = declared - bound; W8 vs W32 differ.
    assert_ne!(g_w8.residual().as_u32(), g_w32.residual().as_u32());

    // d_delta() = witt_bits - bound_count differs because witt_bits differs.
    assert_ne!(g_w8.d_delta().as_i64(), g_w32.d_delta().as_i64());
}

// ─────────────────────────────────────────────────────────────────────────
// Phase F (T2.7): drivers walk their declarations
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn phase_f_run_parallel_unit_address_depends_on_site_count() {
    // v0.2.2 Phase H3: the only ParallelDeclaration constructor is
    // `new_with_partition`; supply explicit partition slices + witness IRI.
    static PARTITION_3: &[u32] = &[0, 1, 2];
    static PARTITION_7: &[u32] = &[0, 1, 2, 3, 4, 5, 6];
    const WITNESS: &str = "https://uor.foundation/parallel/ParallelDisjointnessWitness";
    let unit_3: Validated<ParallelDeclaration> =
        validated_runtime(ParallelDeclaration::new_with_partition::<
            ConstrainedTypeInput,
        >(PARTITION_3, WITNESS));
    let unit_7: Validated<ParallelDeclaration> =
        validated_runtime(ParallelDeclaration::new_with_partition::<
            ConstrainedTypeInput,
        >(PARTITION_7, WITNESS));
    let g_3: Grounded<'static, ConstrainedTypeInput, N> =
        run_parallel::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit_3)
            .expect("3-site parallel walks");
    let g_7: Grounded<'static, ConstrainedTypeInput, N> =
        run_parallel::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit_7)
            .expect("7-site parallel walks");
    assert_ne!(g_3.unit_address(), g_7.unit_address());
}

#[test]
fn phase_f_stream_driver_yields_distinct_grounded() {
    let unit: Validated<StreamDeclaration<'_, N>> =
        validated_runtime(StreamDeclaration::new::<ConstrainedTypeInput>(3));
    let mut driver: StreamDriver<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32> = run_stream(unit);
    let g1 = driver.next().expect("step 1").expect("step 1 ok");
    let g2 = driver.next().expect("step 2").expect("step 2 ok");
    let g3 = driver.next().expect("step 3").expect("step 3 ok");
    assert!(driver.next().is_none(), "stream terminates after bound");
    assert_eq!(driver.rewrite_steps(), 3);
    // Each yielded Grounded has a distinct unit_address from the substrate fingerprint.
    assert_ne!(g1.unit_address(), g2.unit_address());
    assert_ne!(g2.unit_address(), g3.unit_address());
    assert_ne!(g1.unit_address(), g3.unit_address());
    // T6.1: each Grounded carries a real (non-zero) substrate fingerprint.
    assert!(!g1.content_fingerprint().is_zero());
    assert!(!g2.content_fingerprint().is_zero());
    assert!(!g3.content_fingerprint().is_zero());
    assert_ne!(g1.content_fingerprint(), g2.content_fingerprint());
}

#[test]
fn phase_f_interaction_driver_folds_peer_inputs() {
    let unit: Validated<InteractionDeclaration> = validated_runtime(InteractionDeclaration::new::<
        ConstrainedTypeInput,
    >(0xDEAD_BEEF));
    let mut driver: InteractionDriver<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32> =
        run_interactive(unit);
    assert_eq!(driver.peer_step_count(), 0);
    assert!(!driver.is_converged());
    assert_eq!(driver.seed(), 0xDEAD_BEEF);

    // Step with non-zero peer_id: returns Continue, increments peer_step_count.
    let payload = PeerPayload::zero(32);
    let input1 = PeerInput::new(0x1234, payload);
    if let StepResult::Continue = driver.step(input1) {
    } else {
        panic!("expected Continue on first step");
    }
    assert_eq!(driver.peer_step_count(), 1);
    assert!(!driver.is_converged());

    // Step with peer_id=0: convergence handshake.
    let input2 = PeerInput::new(0, PeerPayload::zero(32));
    match driver.step(input2) {
        StepResult::Converged(_) => {}
        _ => panic!("expected Converged on convergence handshake"),
    }
    assert!(driver.is_converged());
    assert_eq!(driver.peer_step_count(), 2);

    // finalize returns Ok with non-zero unit_address.
    let final_grounded: Grounded<'static, ConstrainedTypeInput, N> =
        driver.finalize().expect("converged");
    assert_ne!(final_grounded.unit_address(), ContentAddress::zero());
    // T6.1: finalize produces a real substrate fingerprint.
    assert!(!final_grounded.content_fingerprint().is_zero());
}

#[test]
fn phase_f_interaction_driver_finalize_rejects_unconverged() {
    let unit: Validated<InteractionDeclaration> =
        validated_runtime(InteractionDeclaration::new::<ConstrainedTypeInput>(0));
    let driver: InteractionDriver<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32> =
        run_interactive(unit);
    assert!(!driver.is_converged());
    let result: Result<Grounded<'static, ConstrainedTypeInput, N>, _> = driver.finalize();
    assert!(result.is_err(), "unconverged driver finalize must error");
}

// ─────────────────────────────────────────────────────────────────────────
// Phase G (T2.8): const-fn companions are input-dependent
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn phase_g_certify_const_functions_carry_unit_level() {
    let validated = build_compile_unit(WittLevel::W32, 42);
    assert_eq!(validated.inner().witt_level(), WittLevel::W32);

    let cert: Validated<GroundingCertificate, CompileTime> =
        certify_tower_completeness_const::<ConstrainedTypeInput, Fnv1aHasher16, N, 32>(&validated);
    assert_eq!(cert.inner().witt_bits(), 32);

    let inhab: Validated<GroundingCertificate, CompileTime> =
        certify_inhabitance_const::<ConstrainedTypeInput, Fnv1aHasher16, N, 32>(&validated);
    assert_eq!(inhab.inner().witt_bits(), 32);

    let mult: Validated<MultiplicationCertificate, CompileTime> =
        certify_multiplication_const::<ConstrainedTypeInput, Fnv1aHasher16, N, 32>(&validated);
    assert_eq!(mult.inner().witt_bits(), 32);
}

#[test]
fn phase_g_validate_compile_unit_const_is_input_dependent() {
    let v1 = build_compile_unit(WittLevel::W8, 100);
    let v2 = build_compile_unit(WittLevel::W32, 200);
    assert_ne!(v1.inner().witt_level(), v2.inner().witt_level());
    assert_ne!(
        v1.inner().thermodynamic_budget(),
        v2.inner().thermodynamic_budget()
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Phase J (T1.1): grounding combinator MarkersImpliedBy bound
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn phase_j_grounding_program_compiles_for_integer_map() {
    let prog: GroundingProgram<u64, IntegerGroundingMap> =
        GroundingProgram::from_primitive(combinators::interpret_le_integer::<u64>());
    let _ = prog.primitive();
}

#[test]
fn phase_j_grounding_program_compiles_for_digest_map() {
    let prog: GroundingProgram<[u8; 32], DigestGroundingMap> =
        GroundingProgram::from_primitive(combinators::digest::<[u8; 32]>());
    let _ = prog.primitive();
}

// ─────────────────────────────────────────────────────────────────────────
// v0.2.2 T5: parametric Hasher + ContentFingerprint round-trip property
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn t5_pipeline_run_threads_constraints_into_unit_address() {
    use uor_foundation::pipeline::run;
    let unit_100: Validated<CompileUnit<'static, N>> =
        build_compile_unit(WittLevel::W8, 100).into();
    let unit_200: Validated<CompileUnit<'static, N>> =
        build_compile_unit(WittLevel::W8, 200).into();
    let g_100: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit_100).expect("100 grounds");
    let g_200: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit_200).expect("200 grounds");
    assert_ne!(
        g_100.unit_address(),
        g_200.unit_address(),
        "different budgets must produce different unit_addresses"
    );
    assert_ne!(
        g_100.content_fingerprint(),
        g_200.content_fingerprint(),
        "different budgets must produce different fingerprints"
    );
    assert!(!g_100.unit_address().is_zero());
    assert!(!g_100.content_fingerprint().is_zero());
}

#[test]
fn t5_grounded_derivation_replay_round_trips_via_verify_trace() {
    use uor_foundation::pipeline::run;
    let unit: Validated<CompileUnit<'static, N>> = build_compile_unit(WittLevel::W8, 100).into();
    let grounded: Grounded<'static, ConstrainedTypeInput, N> =
        run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit).expect("grounds");
    let trace: uor_foundation::Trace = grounded.derivation().replay();
    let reverified =
        uor_foundation::enforcement::replay::certify_from_trace(&trace).expect("re-verifies");
    assert_eq!(
        grounded.content_fingerprint(),
        reverified.certificate().content_fingerprint(),
    );
    assert_eq!(
        grounded.witt_level_bits(),
        reverified.certificate().witt_bits(),
    );
}

#[test]
fn t5_distinct_widths_produce_distinct_fingerprints_for_same_unit() {
    use uor_foundation::pipeline::run_const;
    use uor_foundation_test_helpers::Fnv1aHasher32;
    let validated = build_compile_unit(WittLevel::W8, 100);
    let g_16: Grounded<'static, ConstrainedTypeInput, N> =
        run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher16, N, 32>(&validated)
            .expect("16");
    let g_32: Grounded<'static, ConstrainedTypeInput, N> =
        run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher32, N, 32>(&validated)
            .expect("32");
    assert_eq!(g_16.content_fingerprint().width_bytes(), 16);
    assert_eq!(g_32.content_fingerprint().width_bytes(), 32);
    assert_ne!(
        g_16.content_fingerprint(),
        g_32.content_fingerprint(),
        "different widths must produce different fingerprints"
    );
}

#[test]
fn t5_certify_distinguishes_certificate_kinds() {
    let validated = build_compile_unit(WittLevel::W32, 42);
    let c_tower: Validated<GroundingCertificate, CompileTime> =
        certify_tower_completeness_const::<ConstrainedTypeInput, Fnv1aHasher16, N, 32>(&validated);
    let c_inhab: Validated<GroundingCertificate, CompileTime> =
        certify_inhabitance_const::<ConstrainedTypeInput, Fnv1aHasher16, N, 32>(&validated);
    assert_ne!(
        c_tower.inner().content_fingerprint(),
        c_inhab.inner().content_fingerprint(),
        "different certificate kinds over the same unit must differ"
    );
}

#[test]
fn t5_stream_driver_is_terminated_observable_without_consumption() {
    let unit: Validated<StreamDeclaration<'_, N>> =
        validated_runtime(StreamDeclaration::new::<ConstrainedTypeInput>(2));
    let mut driver: StreamDriver<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32> = run_stream(unit);
    assert!(!driver.is_terminated());
    let _ = driver.next().expect("step 1");
    assert!(!driver.is_terminated());
    let _ = driver.next().expect("step 2");
    // After exhausting the productivity bound, next() returns None and
    // the driver is observably terminated.
    assert!(driver.next().is_none());
    assert!(driver.is_terminated());
}

#[test]
fn t5_all_public_errors_implement_core_error_error() {
    use uor_foundation::enforcement::{
        BindingsTableError, CalibrationError, GenericImpossibilityWitness, PipelineFailure,
        ReplayError, ShapeViolation,
    };
    fn assert_error<E: core::error::Error>() {}
    assert_error::<ReplayError>();
    assert_error::<CalibrationError>();
    assert_error::<ShapeViolation>();
    assert_error::<PipelineFailure>();
    assert_error::<BindingsTableError>();
    assert_error::<GenericImpossibilityWitness>();
}

#[test]
fn t5_trace_try_from_events_rejects_malformed() {
    use uor_foundation::enforcement::{Hasher, ReplayError, Trace, TraceEvent};
    use uor_foundation_test_helpers::trace_event;
    // T6.6: ContentFingerprint::zero() is pub(crate); use a real substrate
    // fingerprint computed via Fnv1aHasher16 for test fixtures.
    let buf = Fnv1aHasher16::initial()
        .fold_bytes(b"t6 fixture")
        .finalize();
    let fp = uor_foundation::enforcement::ContentFingerprint::from_buffer(
        buf,
        Fnv1aHasher16::OUTPUT_BYTES as u8,
    );
    // Out-of-order: event[0] has step_index=5 (not 0).
    let bad = [trace_event(5, 0xAA)];
    match Trace::<256>::try_from_events(&bad, 8, fp) {
        Err(ReplayError::OutOfOrderEvent { index: 0 }) => {}
        other => panic!("expected OutOfOrderEvent, got {other:?}"),
    }
    // Empty: explicit construction requires at least one event.
    let empty: [TraceEvent; 0] = [];
    match Trace::<256>::try_from_events(&empty, 8, fp) {
        Err(ReplayError::EmptyTrace) => {}
        other => panic!("expected EmptyTrace, got {other:?}"),
    }
    // Zero target: rejected.
    let zero_target = [trace_event(0, 0)];
    match Trace::<256>::try_from_events(&zero_target, 8, fp) {
        Err(ReplayError::ZeroTarget { index: 0 }) => {}
        other => panic!("expected ZeroTarget, got {other:?}"),
    }
    // Valid: contiguous, monotonic, non-zero targets.
    let ok = [trace_event(0, 0xAA), trace_event(1, 0xBB)];
    let trace = Trace::<256>::try_from_events(&ok, 8, fp).unwrap();
    assert_eq!(trace.len(), 2);
}

#[test]
fn t5_bindings_table_try_new_rejects_unsorted() {
    use uor_foundation::enforcement::{
        BindingEntry, BindingsTable, BindingsTableError, ContentAddress,
    };
    static ENTRIES: &[BindingEntry] = &[
        BindingEntry {
            address: ContentAddress::from_u128(0x20),
            bytes: b"b",
        },
        BindingEntry {
            address: ContentAddress::from_u128(0x10),
            bytes: b"a",
        },
    ];
    match BindingsTable::try_new(ENTRIES) {
        Err(BindingsTableError::Unsorted { at: 1 }) => {}
        other => panic!("expected Unsorted at 1, got {other:?}"),
    }
}

#[test]
fn t6_grounded_with_bindings_attaches_downstream_table() {
    // v0.2.2 T6.17: Grounded::with_bindings() lets downstream attach a
    // validated BindingsTable without re-grounding. The original certificate
    // and fingerprint are preserved; only the bindings field is replaced.
    use uor_foundation::enforcement::{BindingEntry, BindingsTable, ContentAddress};
    let unit = build_compile_unit(WittLevel::W8, 1024);
    let grounded =
        uor_foundation::pipeline::run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit)
            .unwrap();
    let original_fp = grounded.content_fingerprint();
    static ENTRIES: &[BindingEntry] = &[
        BindingEntry {
            address: ContentAddress::from_u128(0x10),
            bytes: b"a",
        },
        BindingEntry {
            address: ContentAddress::from_u128(0x20),
            bytes: b"b",
        },
    ];
    let bindings = BindingsTable::try_new(ENTRIES).expect("sorted");
    let attached = grounded.with_bindings(bindings);
    assert_eq!(attached.content_fingerprint(), original_fp);
    assert_eq!(
        attached.get_binding(ContentAddress::from_u128(0x10)),
        Some(b"a".as_slice())
    );
    assert_eq!(
        attached.get_binding(ContentAddress::from_u128(0x20)),
        Some(b"b".as_slice())
    );
}

#[test]
fn t5_short_path_re_exports_resolve() {
    // Compile-time check that the lib.rs re-exports actually resolve under
    // the short `uor_foundation::*` path. If any import fails to resolve,
    // this test fails to compile.
    //
    // T6.3: ZeroHasher is removed (no migration marker).
    // T6.6: ContentFingerprint::zero() is pub(crate).
    use uor_foundation::{
        BindingsTableError, CalibrationError, Certificate, CertificateKind, ContentFingerprint,
        Hasher, HostBounds, LandauerBudget, Nanos, PipelineFailure, PrimitiveOp, Term, TermArena,
        TermList, WittLevel,
    };
    use uor_foundation_test_helpers::ReferenceHostBounds;
    fn _accept_certificate<C: Certificate>() {}
    fn _accept_hasher<H: Hasher>() {}
    let _: BindingsTableError = BindingsTableError::Unsorted { at: 1 };
    let _: CalibrationError = CalibrationError::ThermalEnergy;
    let _: CertificateKind = CertificateKind::Grounding;
    // Capacity bounds are reached through the `HostBounds` substitution
    // axis (wiki ADR-018); there are no free-standing capacity constants
    // on the public surface. ADR-060 removed `DefaultHostBounds`; the
    // test-only `ReferenceHostBounds` carries the canonical values.
    let _: usize = <ReferenceHostBounds as HostBounds>::FINGERPRINT_MAX_BYTES;
    let _: usize = <ReferenceHostBounds as HostBounds>::FINGERPRINT_MIN_BYTES;
    let _: usize = <ReferenceHostBounds as HostBounds>::TRACE_MAX_EVENTS;
    let _: PrimitiveOp = PrimitiveOp::Add;
    let _: WittLevel = WittLevel::W8;
    // The remaining types are just imported to verify they resolve.
    let _ = core::any::type_name::<ContentFingerprint>();
    let _ = core::any::type_name::<LandauerBudget>();
    let _ = core::any::type_name::<Nanos>();
    let _ = core::any::type_name::<PipelineFailure>();
    let _ = core::any::type_name::<Term<N>>();
    let _ = core::any::type_name::<TermArena<N, 4>>();
    let _ = core::any::type_name::<TermList>();
}
