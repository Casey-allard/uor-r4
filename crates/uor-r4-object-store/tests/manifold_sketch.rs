//! Finite CPU controls for multiplier-free manifold sketching, hierarchical scope
//! routing, and epistemic hazard gating over `uor-r4-object-store`.

use std::{
    error::Error,
    sync::atomic::{AtomicUsize, Ordering},
};
use uor_r4_object_store::{
    address_read::{AddressReadOutcome, ObjectTarget, PrimeAddress},
    manifold_sketch::{
        blend_hierarchical_overlap, encode_span_sketch, Archetype, EpistemicState, Modality,
        PlacementDecision, ScopedAddressDirectory, SketchAccumulator, THREE_SIGMA_EXCESS_BITS,
    },
    Limits, NodeId, Record, RecordBackend, Store,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

struct CountingBackend<'a> {
    inner: &'a Store,
    fetches: AtomicUsize,
}

impl RecordBackend for CountingBackend<'_> {
    fn fetch(&self, id: NodeId) -> uor_r4_object_store::Result<Option<Record>> {
        self.fetches.fetch_add(1, Ordering::SeqCst);
        self.inner.fetch(id)
    }
}

#[test]
fn sketch_accumulator_exact_revert_and_hierarchical_blend() {
    let s1 = encode_span_sketch("holonomic_parallel_transport laplacian curvature expansion");
    let s2 = encode_span_sketch("verify_som_lucidity_gate quantization error threshold check");
    let transient = encode_span_sketch("transient_failed_hypothesis_unnormalized_drift");

    let mut acc = SketchAccumulator::new();
    acc.accumulate(&s1, 2);
    acc.accumulate(&s2, 1);
    let snap = acc.clone();
    let proj_before = acc.project();

    acc.accumulate(&transient, 4);
    assert!(acc.project().hamming(&proj_before) > 200);

    acc.accumulate(&transient, -4);
    assert_eq!(acc.counts, snap.counts);
    assert_eq!(acc.project(), proj_before);

    // Verify multiplier-free (13 * local + 3 * parent) >> 4 identity
    assert_eq!(blend_hierarchical_overlap(2400, 2080), 2340);
}

#[test]
fn hybrid_sieve_recovers_inflected_and_paraphrased_queries_against_controls() -> TestResult {
    let mut store = Store::new(Limits::default());
    let payload_fft = store.fold(b"helmholtz_fft_step(H_t, dt, g_eff)")?;
    let payload_hopf = store.fold(b"prime_zeta_hopf_address(token_id, primes)")?;
    let payload_gate = store.fold(b"verify_som_lucidity_gate(state_slice, threshold)")?;

    let mut scoped = ScopedAddressDirectory::new(4)?;

    let addr_fft = PrimeAddress::with_pairs(&[2, 3], &[(2, 3)])?;
    let addr_hopf = PrimeAddress::with_pairs(&[5, 7], &[(5, 7)])?;
    let addr_gate = PrimeAddress::with_pairs(&[11, 13], &[(11, 13)])?;

    scoped.write_scoped(
        addr_fft.clone(),
        ObjectTarget::WholeObject {
            manifest: payload_fft.manifest,
        },
        "solve_helmholtz_fft_step frequency domain Fourier wave transport equation laplacian denominator",
        Some("src/helmholtz_solver.rs"),
        Modality::RepoCode,
    )?;

    scoped.write_scoped(
        addr_hopf.clone(),
        ObjectTarget::WholeObject {
            manifest: payload_hopf.manifest,
        },
        "computePrimeZetaHopfAddress maps prime numbers and Riemann zeta zero phases onto unit 3-sphere S3 coordinate",
        Some("src/prime_hopf.rs"),
        Modality::RepoCode,
    )?;

    scoped.write_scoped(
        addr_gate.clone(),
        ObjectTarget::WholeObject {
            manifest: payload_gate.manifest,
        },
        "verifySomLucidityGate checks quantization error threshold and audit ledger invariant",
        Some("src/lucidity_gate.rs"),
        Modality::RepoCode,
    )?;

    // Inflected / re-cased query whose surface prime address [17, 19] shares ZERO primes
    // with addr_hopf ([5, 7]).
    let disjoint_prime_query = PrimeAddress::with_pairs(&[17, 19], &[(17, 19)])?;
    let paraphrased_text =
        "computing prime_zeta_hopf_addresses mapping primes and Riemann zeta phases on S3 sphere coordinates";

    // Control 1: Exact prime sieve alone misses (`0` shared primes -> Absent)
    let sieve_only = scoped
        .directory()
        .read_sieve(&store, None, &disjoint_prime_query)?;
    assert_eq!(
        sieve_only,
        AddressReadOutcome::Absent,
        "exact prime sieve alone must miss when surface primes are disjoint"
    );

    // Treatment: Two-stage hybrid sieve + multi-granularity sketch recovers the exact verified payload
    let hybrid_hit = scoped.read_hybrid_sieve(
        &store,
        None,
        Some(&disjoint_prime_query),
        paraphrased_text,
        Some(Modality::RepoCode),
        Some(Archetype::Space),
    )?;
    match hybrid_hit {
        AddressReadOutcome::Hit {
            address,
            bytes,
            whole_object_verified,
            ..
        } => {
            assert_eq!(address, addr_hopf);
            assert_eq!(bytes, b"prime_zeta_hopf_address(token_id, primes)");
            assert!(whole_object_verified);
        }
        other => return Err(format!("expected hybrid Hit for hopf, got {other:?}").into()),
    }

    // Control 2: Unstated / orthogonal query must be rejected by the +3 sigma gate with 0 backend fetches
    let p4 = store.fold(b"slot-4")?;
    let p5 = store.fold(b"slot-5")?;
    let counting = CountingBackend {
        inner: &store,
        fetches: AtomicUsize::new(0),
    };
    let unstated_query = PrimeAddress::with_pairs(&[23, 29], &[(23, 29)])?;
    let unstated_outcome = scoped.read_hybrid_sieve_from(
        &counting,
        store.limits(),
        None,
        Some(&unstated_query),
        "postgresql btree vacuum wal checkpoint replication slot timeout",
        None,
        None,
    )?;
    assert_eq!(unstated_outcome, AddressReadOutcome::Absent);
    assert_eq!(
        counting.fetches.load(Ordering::SeqCst),
        0,
        "unstated query below +3 sigma must perform 0 backend fetches"
    );

    // Control 3: Capacity eviction must retract the oldest slot and return Evicted with 0 backend fetches
    scoped.write_scoped(
        PrimeAddress::new(&[31, 37])?,
        ObjectTarget::WholeObject {
            manifest: p4.manifest,
        },
        "auxiliary buffer allocator",
        Some("src/aux.rs"),
        Modality::RepoCode,
    )?;
    scoped.write_scoped(
        PrimeAddress::new(&[41, 43])?,
        ObjectTarget::WholeObject {
            manifest: p5.manifest,
        },
        "secondary stream codec",
        Some("src/codec.rs"),
        Modality::RepoCode,
    )?;

    // addr_fft was the oldest slot (sequence 1) and is now evicted
    let evicted_outcome = scoped.read_hybrid_sieve_from(
        &counting,
        store.limits(),
        None,
        Some(&disjoint_prime_query),
        "solve_helmholtz_fft_step frequency domain Fourier wave transport equation laplacian",
        None,
        None,
    )?;
    match evicted_outcome {
        AddressReadOutcome::Evicted { address, .. } => {
            assert_eq!(address, addr_fft);
        }
        other => return Err(format!("expected Evicted for addr_fft, got {other:?}").into()),
    }
    assert_eq!(
        counting.fetches.load(Ordering::SeqCst),
        0,
        "evicted slot must perform 0 backend fetches"
    );

    Ok(())
}

#[test]
fn hierarchical_scope_incremental_update_and_retraction_is_exact() -> TestResult {
    let mut store = Store::new(Limits::default());
    let f1 = store.fold(b"b1")?;
    let f2 = store.fold(b"b2")?;
    let f2_v2 = store.fold(b"b2_v2")?;

    let addr_1 = PrimeAddress::new(&[2, 3])?;
    let addr_2 = PrimeAddress::new(&[5, 7])?;

    let text_1 = "struct HolonomicField H_mu omega_mu hypervector accumulator state";
    let text_2_v1 = "fn draft_unnormalized_transport(field: &HolonomicField) { /* v1 */ }";
    let text_2_v2 = "fn stabilized_helmholtz_fft(field: &HolonomicField, dt: i32) { /* v2 */ }";

    // Directory A: ingest text_1 + text_2_v1, then overwrite addr_2 with text_2_v2
    let mut dir_incremental = ScopedAddressDirectory::new(8)?;
    dir_incremental.write_scoped(
        addr_1.clone(),
        ObjectTarget::WholeObject {
            manifest: f1.manifest.clone(),
        },
        text_1,
        Some("src/field.rs"),
        Modality::RepoCode,
    )?;
    dir_incremental.write_scoped(
        addr_2.clone(),
        ObjectTarget::WholeObject {
            manifest: f2.manifest,
        },
        text_2_v1,
        Some("src/field.rs"),
        Modality::RepoCode,
    )?;
    dir_incremental.write_scoped(
        addr_2.clone(),
        ObjectTarget::WholeObject {
            manifest: f2_v2.manifest.clone(),
        },
        text_2_v2,
        Some("src/field.rs"),
        Modality::RepoCode,
    )?;

    // Directory B: clean from-scratch build with only text_1 and text_2_v2
    let mut dir_clean = ScopedAddressDirectory::new(8)?;
    dir_clean.write_scoped(
        addr_1,
        ObjectTarget::WholeObject {
            manifest: f1.manifest,
        },
        text_1,
        Some("src/field.rs"),
        Modality::RepoCode,
    )?;
    dir_clean.write_scoped(
        addr_2,
        ObjectTarget::WholeObject {
            manifest: f2_v2.manifest,
        },
        text_2_v2,
        Some("src/field.rs"),
        Modality::RepoCode,
    )?;

    assert_eq!(
        dir_incremental.global_acc.counts, dir_clean.global_acc.counts,
        "incremental overwrite must leave global accumulator bit-for-bit identical to clean build"
    );
    assert_eq!(
        dir_incremental.scope_sketch("src/field.rs"),
        dir_clean.scope_sketch("src/field.rs"),
        "parent scope sketch must match clean build identically"
    );

    // Removing the scope must restore the empty baseline
    dir_incremental.remove_scope("src/field.rs");
    assert_eq!(dir_incremental.global_acc.item_count, 0);
    assert!(dir_incremental.scope_sketch("src/field.rs").is_none());

    Ok(())
}

#[test]
fn epistemic_reject_ledger_triggers_hazard_alert_at_three_sigma() -> TestResult {
    let mut store = Store::new(Limits::default());
    let f1 = store.fold(b"core")?;
    let mut scoped = ScopedAddressDirectory::new(8)?;

    let addr_core = PrimeAddress::new(&[2, 3])?;
    scoped.write_scoped(
        addr_core.clone(),
        ObjectTarget::WholeObject {
            manifest: f1.manifest,
        },
        "void holonomic_parallel_transport(HolonomicFieldConnection& field, const float* query, float* out, int n) \
         evaluates Taylor curvature expansion laplacian in hufd_core",
        Some("src/hufd_core.cpp"),
        Modality::RepoCode,
    )?;

    scoped.record_experiment(
        "exp-014-unbounded-taylor",
        Archetype::Op,
        EpistemicState::Reject,
        "Unnormalized 4th-order Taylor curvature expansion of holonomic_parallel_transport \
         without consensus potential gain caused norm explosion at depth 12",
    );

    // Task 1: Resonates strongly with both the existing block and the falsified experiment
    let plan_hazard = scoped.place_task(
        "Implement unnormalized 4th-order Taylor curvature expansion for holonomic_parallel_transport \
         laplacian query out in hufd_core",
    );
    assert_eq!(plan_hazard.best_slot, Some(addr_core));
    assert_eq!(
        plan_hazard.decision,
        PlacementDecision::InsertAtExistingBlock
    );
    let alert = plan_hazard
        .hazard
        .expect("task matching Reject experiment must trigger HazardAlert");
    assert_eq!(alert.exp_id, "exp-014-unbounded-taylor");
    assert!(alert.excess_bits >= THREE_SIGMA_EXCESS_BITS);
    assert!(alert.sigma_multiple >= 3);

    // Task 2: Orthogonal task should have no hazard alert
    let plan_safe = scoped
        .place_task("Add CLI command line argument parser and UTF-8 file reader for input stream");
    assert!(plan_safe.hazard.is_none());
    assert_eq!(plan_safe.best_archetype, Archetype::Io);
    assert_eq!(
        plan_safe.decision,
        PlacementDecision::ScaffoldNewFileForStage
    );

    Ok(())
}
