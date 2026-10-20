//! Allocation census for the status-aware deployed path (issue #78).
//!
//! A counting global allocator measures what the deployed prediction APIs
//! actually allocate, mirroring `crates/uor-r4-core/tests/allocation_census.rs`.
//! One single `#[test]` by design: the allocator's gate and counters are
//! process-wide and libtest runs tests in parallel threads, so a second
//! test's bookkeeping allocations could land in this census — fatal to the
//! zero-allocation assertion. The probe suite lives in the separate
//! `status_policy` test binary for the same reason.
//!
//! Run with:
//! `cargo test -p uor-r4-wasm-router --test status_policy_census -- --nocapture`

mod status_policy_common;

use uor_r4_graph_certify::ScoreStatus;
use uor_r4_wasm_router::r4g1::PredictDecision;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Census {
    allocations: usize,
    bytes: usize,
}

const ZERO: Census = Census {
    allocations: 0,
    bytes: 0,
};

/// Run `f` with the counting gate open; return its output and the census of
/// what it allocated. Reporting happens with the gate closed.
fn measure<T>(f: impl FnOnce() -> T) -> (T, Census) {
    let c_before = uor_r4_proof_model::allocation_proof::current_alloc_count();
    let b_before = uor_r4_proof_model::allocation_proof::current_alloc_bytes();
    let out = f();
    let c_after = uor_r4_proof_model::allocation_proof::current_alloc_count();
    let b_after = uor_r4_proof_model::allocation_proof::current_alloc_bytes();
    (
        out,
        Census {
            allocations: c_after.saturating_sub(c_before),
            bytes: b_after.saturating_sub(b_before),
        },
    )
}

// ---------------------------------------------------------------- census --

#[test]
fn status_path_allocation_census() {
    println!("=== allocation census: status-aware deployed R4G1 path (issue #78) ===");

    // Setup (gate closed): the fixture bundle and the loaded adapter — the
    // one-time buffers (step state, widen-once memory) are built here.
    let fixture = status_policy_common::window_fixture();
    let state = fixture.load();
    let ood_window = status_policy_common::find_window_by_status(&fixture, ScoreStatus::Novel);
    let counters0 = state.policy_counters();
    println!(
        "[setup] counters after load: predicts {} | serves {} | abstains {}",
        counters0.predicts, counters0.serves, counters0.abstains
    );

    // Warm-up: first-touch epochs, counters, and the widen-once memory.
    // Reported, not asserted (it is already expected to be zero: every
    // buffer is pre-sized at load).
    let ((), warm_cen) = measure(|| {
        let _ = state.predict_window_status(&fixture.covered_window);
        let _ = state.predict_window_status(&ood_window);
    });
    println!(
        "[status] warm-up (first served + first widened abstain) \
         → {} allocations, {} bytes (report only)",
        warm_cen.allocations, warm_cen.bytes
    );

    // Steady state: one served prediction, one abstaining prediction from
    // the widen-once memory, and a status-aware generation run.
    let ((served, abstained, gen), steady_cen) = measure(|| {
        let served = state
            .predict_window_status(&fixture.covered_window)
            .expect("served prediction");
        let abstained = state
            .predict_window_status(&ood_window)
            .expect("abstaining prediction");
        let mut out = [0u32; 16];
        let gen = state
            .generate_into_status(&fixture.covered_window, &mut out)
            .expect("generation");
        (served, abstained, gen)
    });
    println!(
        "[status] steady state: predict(serve) + predict(abstain) + \
         generate_into_status(16 tokens) → {} allocations, {} bytes",
        steady_cen.allocations, steady_cen.bytes
    );
    assert!(
        matches!(served, PredictDecision::Serve(_)),
        "the covered window serves"
    );
    assert!(
        matches!(abstained, PredictDecision::Abstain(_)),
        "the OOD window abstains"
    );
    assert_eq!(
        steady_cen, ZERO,
        "steady-state status-aware prediction and generation must be allocation-free"
    );
    println!(
        "[gen] count {} | abstained {} | status {:?} | widened {}",
        gen.count, gen.abstained, gen.status, gen.widened
    );

    // Repeated adversarial OOD probes (widen-once memory hits) are
    // allocation-free too, and never widen again.
    let before = state.policy_counters();
    let ((), repeat_cen) = measure(|| {
        for _ in 0..8 {
            let _ = state.predict_window_status(&ood_window);
        }
    });
    println!(
        "[status] 8 repeated OOD probes (widen-once memory) \
         → {} allocations, {} bytes",
        repeat_cen.allocations, repeat_cen.bytes
    );
    assert_eq!(
        repeat_cen, ZERO,
        "widen-once memory lookups must be allocation-free"
    );
    let after = state.policy_counters();
    assert_eq!(
        after.widen_attempts, before.widen_attempts,
        "repeated identical OOD probes never widen again"
    );
    assert_eq!(
        after.widen_skipped_seen,
        before.widen_skipped_seen + 8,
        "each repetition is answered from the widen-once memory"
    );

    // The legacy delegating entry points are allocation-free on the same
    // paths (serve delegates; abstain-by-error allocates its message and
    // is therefore exercised only on the served window).
    let (token, delegate_cen) = measure(|| {
        state
            .predict_window(&fixture.covered_window)
            .expect("delegate serve")
    });
    println!(
        "[status] predict_window delegate (served) → {} allocations, {} bytes",
        delegate_cen.allocations, delegate_cen.bytes
    );
    assert_eq!(token, 10);
    assert_eq!(delegate_cen, ZERO);

    let counters = state.policy_counters();
    println!(
        "[counters] predicts {} | serves {} | abstains {} | widen_attempts {} | widen_skipped_seen {}",
        counters.predicts, counters.serves, counters.abstains, counters.widen_attempts,
        counters.widen_skipped_seen
    );
    assert!(
        counters.widen_attempts <= counters.serves + counters.abstains,
        "widening is bounded by the predictions that ran"
    );
    println!("=== end census ===");
}
