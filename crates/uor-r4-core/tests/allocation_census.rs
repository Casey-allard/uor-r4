//! Allocation census for the transformerless runtime — Phase-0 baseline.
//!
//! A counting global allocator measures what the runtime APIs actually
//! allocate. One single `#[test]` by design: the allocator's gate and
//! counters are process-wide, and libtest runs tests in parallel threads by
//! default, so several measured tests could let one test's bookkeeping
//! allocations land in another test's census — fatal to the zero-allocation
//! assertion. One test, sequential phases, no cross-thread noise.
//!
//! Measured sections run inside [`measure`], which resets the counters,
//! opens the gate, runs the closure, closes the gate, and snapshots. Every
//! `println!` happens with the gate closed, so reporting never pollutes
//! the numbers. Fixed seed, no wall-clock assertions: the census is
//! deterministic for a given artifact set.
//!
//! Run with: `cargo test -p uor-r4-core --test allocation_census -- --nocapture`

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use uor_r4_core::transformerless::compiler::{self, Compiled, STAGES, WINDOW};
use uor_r4_core::transformerless::runtime::{self, OpKernel, Prediction, Store};

// --------------------------------------------------- counting allocator --

/// Global allocator that counts allocation events and gross bytes requested
/// while the gate is open. Buffer growth goes through the default
/// `GlobalAlloc::realloc`, which calls `alloc` for the new layout, so a
/// grown buffer shows up as one fresh allocation of the new size.
/// Deallocations are not counted: the census answers "how much did the
/// measured section ask for", not "what was live at the end".
struct CountingAlloc;

static GATE: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && GATE.load(Ordering::SeqCst) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Census {
    allocations: usize,
    bytes: usize,
}

const ZERO: Census = Census {
    allocations: 0,
    bytes: 0,
};

fn census() -> Census {
    Census {
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        bytes: BYTES.load(Ordering::Relaxed),
    }
}

/// Run `f` with the counting gate open; return its output and the census of
/// what it allocated.
fn measure<T>(f: impl FnOnce() -> T) -> (T, Census) {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    BYTES.store(0, Ordering::Relaxed);
    GATE.store(true, Ordering::SeqCst);
    let out = f();
    GATE.store(false, Ordering::SeqCst);
    (out, census())
}

// ------------------------------------------------------------ op census --

/// Snapshot of the runtime's `OpKernel` counters (the kernel itself is not
/// `Clone`, so copy the public fields).
#[derive(Clone, Copy)]
struct Ops {
    adds: u64,
    xors: u64,
    shifts: u64,
    compares: u64,
    table_reads: u64,
    candidate_scans: u64,
}

impl Ops {
    fn of(k: &OpKernel) -> Self {
        Ops {
            adds: k.adds,
            xors: k.xors,
            shifts: k.shifts,
            compares: k.compares,
            table_reads: k.table_reads,
            candidate_scans: k.candidate_scans,
        }
    }
    fn since(self, before: Ops) -> Ops {
        Ops {
            adds: self.adds - before.adds,
            xors: self.xors - before.xors,
            shifts: self.shifts - before.shifts,
            compares: self.compares - before.compares,
            table_reads: self.table_reads - before.table_reads,
            candidate_scans: self.candidate_scans - before.candidate_scans,
        }
    }
}

// ---------------------------------------------------------------- census --

/// Fixed number of tokens to generate per census run.
const GEN_TOKENS: usize = 32;
/// Fixed seed window: deterministic, and every id is below the smallest
/// supported vocabulary (TLA3, V = 32000), so the same seed drives both the
/// fixture and the real smollm2 artifacts.
const SEED: [u32; WINDOW] = [1, 40, 416, 1024, 2048, 4096, 8192, 16384];

fn real_path(name: &str) -> String {
    let dir = env!("CARGO_MANIFEST_DIR");
    format!("{dir}/../../.uor-models/compiled/smollm2-135m-instruct/{name}")
}

fn fixture_path(name: &str) -> String {
    let dir = env!("CARGO_MANIFEST_DIR");
    format!("{dir}/tests/fixtures/{name}")
}

/// Parse the artifact container, measuring the parse. Prefers the real
/// compiled smollm2 artifacts; falls back to the repo fixture so the census
/// still runs on a bare checkout. Returns the artifacts and the path used.
fn parse_artifacts_measured() -> (Compiled, String) {
    for path in [
        real_path("tless_artifacts.bin"),
        fixture_path("tless_artifacts.bin"),
    ] {
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let size = bytes.len();
        let (parsed, cen) = measure(|| compiler::parse_artifacts(&bytes));
        if let Some(art) = parsed {
            let vocab = art.token_codes.len() / STAGES;
            println!(
                "[parse] artifacts: {path} ({size} bytes, vocab {vocab}) \
                 → {} allocations, {} bytes",
                cen.allocations, cen.bytes
            );
            return (art, path);
        }
        println!("[parse] artifacts: {path} did not parse, trying next candidate");
    }
    panic!("no parsable TLA3/TLA4/TLA5 artifact container found");
}

/// Legacy TLS1 variant with 6-byte `(u16 token, u32 count)` entries, written
/// by pre-u32-migration compilers — the on-disk smollm2 store predates the
/// token-width change, and the production parser (8-byte entries) rejects it.
/// Test-local so the census measures the real store without touching the
/// production parser.
fn parse_store_legacy_u16(b: &[u8]) -> Option<Store> {
    if b.len() < 4 || &b[0..4] != b"TLS1" {
        return None;
    }
    let mut o = 4usize;
    let mut store: Store = Vec::new();
    for d in 0..=STAGES {
        if o + 4 > b.len() {
            return None;
        }
        let n_keys = u32::from_le_bytes(b[o..o + 4].try_into().ok()?) as usize;
        o += 4;
        let mut level = std::collections::BTreeMap::new();
        for _ in 0..n_keys {
            if o >= b.len() {
                return None;
            }
            let klen = b[o] as usize;
            o += 1;
            if klen != d || o + klen + 4 > b.len() {
                return None;
            }
            let key = b[o..o + klen].to_vec();
            o += klen;
            let n_entries = u32::from_le_bytes(b[o..o + 4].try_into().ok()?) as usize;
            o += 4;
            let mut dist = std::collections::BTreeMap::new();
            for _ in 0..n_entries {
                if o + 6 > b.len() {
                    return None;
                }
                let t = u32::from(u16::from_le_bytes(b[o..o + 2].try_into().ok()?));
                let cnt = u32::from_le_bytes(b[o + 2..o + 6].try_into().ok()?);
                o += 6;
                dist.insert(t, cnt);
            }
            level.insert(key, dist);
        }
        store.push(level);
    }
    if o != b.len() {
        return None;
    }
    Some(store)
}

/// Parse the TLS1 store, measuring the parse. Falls back to a small
/// synthetic store (level 0 populated, so prediction terminates) when the
/// real store file is absent or unparsable. Returns the store and an origin
/// label.
fn parse_store_measured() -> (Store, String) {
    let path = real_path("tless_store.bin");
    if let Ok(bytes) = std::fs::read(&path) {
        let size = bytes.len();
        let (parsed, cen) = measure(|| {
            if let Some(store) = runtime::parse_store(&bytes) {
                Some((store, "TLS1"))
            } else {
                parse_store_legacy_u16(&bytes).map(|store| (store, "TLS1-u16"))
            }
        });
        if let Some((store, fmt)) = parsed {
            let keys: usize = store.iter().map(|level| level.len()).sum();
            println!(
                "[parse] store: {path} ({size} bytes {fmt}, {keys} class keys) \
                 → {} allocations, {} bytes",
                cen.allocations, cen.bytes
            );
            return (store, format!("{path} ({fmt})"));
        }
        println!("[parse] store: {path} did not parse; using a synthetic store");
    } else {
        println!("[parse] store: {path} not present; using a synthetic store");
    }
    let mut store: Store = (0..=STAGES).map(|_| Default::default()).collect();
    for i in 0..64u16 {
        let code = [(i % 251) as u8, (i / 2) as u8, 0, 0];
        runtime::add_evidence(&mut store, &code, i as u32, 1);
    }
    (store, "synthetic (add_evidence, gate closed)".to_string())
}

#[test]
fn allocation_census() {
    println!("=== allocation census: uor-r4-core transformerless runtime (Phase-0 baseline) ===");

    // Phase 1 — container parsing (expected > 0; reported, not asserted).
    let (art, _art_src) = parse_artifacts_measured();
    let (store, _store_src) = parse_store_measured();

    // Phase 2 — Runtime construction (expected 0; reported, not asserted).
    let (mut rt, rt_cen) = measure(|| runtime::Runtime::new(&art));
    println!(
        "[runtime] Runtime::new → {} allocations, {} bytes (report only)",
        rt_cen.allocations, rt_cen.bytes
    );

    // Phase 3 — prediction and generation, every runtime API variant
    // exercised: assign_window, predict, predict_witness, and
    // generate_greedy_into with a caller-owned output buffer.
    //
    // Warm-up first: `Runtime.recent` (the repetition guard) is a `Vec` that
    // grows to its steady-state capacity during the first ~34 predictions
    // (push + remove(0) once len > 32). That growth is a fixed number of
    // allocations per Runtime lifetime — measured and reported, not asserted.
    // Steady-state behavior after warm-up is what the zero-allocation claim
    // covers, and what the graph runtime's fixed-capacity RuntimeState (plan
    // Phase 5) makes unconditional.
    let seed_code = rt.assign_window(&SEED);
    let ((), warm_cen) = measure(|| {
        let mut warm_out = [Prediction::default(); GEN_TOKENS];
        let _ = rt.predict_witness(&store, &seed_code);
        let _ = rt.predict(&store, &seed_code);
        let _ = rt.generate_greedy_into(&store, &SEED, &mut warm_out);
    });
    println!(
        "[runtime] warm-up (recency Vec growth, first ~34 predictions) \
         → {} allocations, {} bytes (report only)",
        warm_cen.allocations, warm_cen.bytes
    );

    // Correctness (outside the measured section, since a fresh Runtime's
    // first recency push allocates): a fresh kernel witness and the plain
    // reference are both pure argmax on the same code and must agree.
    let mut fresh = runtime::Runtime::new(&art);
    assert_eq!(
        fresh.predict_witness(&store, &seed_code).token,
        runtime::predict_witness_plain(&store, &seed_code).token,
        "kernel and plain argmax agree on fresh state"
    );

    let mut out = [Prediction::default(); GEN_TOKENS];
    let ((code, wit, n, gen_ops), gen_cen) = measure(|| {
        let code = rt.assign_window(&SEED);
        let wit = rt.predict_witness(&store, &code);
        let _ = rt.predict(&store, &code); // exercise the stateful entry point
        let before = Ops::of(&rt.kernel);
        let n = rt.generate_greedy_into(&store, &SEED, &mut out);
        let gen_ops = Ops::of(&rt.kernel).since(before);
        (code, wit, n, gen_ops)
    });
    assert_eq!(n, GEN_TOKENS, "every output slot filled");
    println!(
        "[runtime] steady state: assign_window + predict + predict_witness + \
         generate_greedy_into({GEN_TOKENS} tokens) → {} allocations, {} bytes",
        gen_cen.allocations, gen_cen.bytes
    );
    println!(
        "          seed code {code:?} → token {} (depth {}, evidence {})",
        wit.token, wit.depth, wit.count
    );
    assert_eq!(
        gen_cen, ZERO,
        "steady-state prediction and generation must be allocation-free"
    );

    // Phase 3b — #318 Phase B residual path: the residual-wired assign
    // path is censused on a synthetic TLA7 artifact (a fixed small
    // container keeps this census independent of fixture-era re-pins;
    // the current fixture is itself TLA7 since the #327 re-pin,
    // 2026-08-01). Plain and kernel forms must agree and allocate
    // nothing.
    let resid_art = {
        let vocab = 4usize;
        Compiled {
            token_codes: (0..vocab * STAGES).map(|i| (i % 256) as u8).collect(),
            stage_books: (0..STAGES)
                .map(|st| {
                    (0..compiler::K * compiler::D)
                        .map(|i| (((i + st) % 5) as i8) - 2)
                        .collect()
                })
                .collect(),
            stage_shifts: vec![0; STAGES],
            thresholds: (0..compiler::D as i64).map(|d| (d % 11) - 5).collect(),
            class_sigs: (0..STAGES)
                .map(|_| vec![0u8; compiler::K * compiler::D / 8])
                .collect(),
            ctx_cb: Vec::new(),
            token_stage_kappas: Vec::new(),
            dot_cb: (0..STAGES)
                .map(|st| {
                    (0..compiler::K * compiler::D)
                        .map(|i| compiler::pack_dot_entry((((i + st) % 13) as f32 - 6.0) / 8.0))
                        .collect()
                })
                .collect(),
            resid_cb: (0..STAGES)
                .map(|st| {
                    (0..compiler::K * compiler::D)
                        .map(|i| (((i + st) % 13) as i8) - 6)
                        .collect()
                })
                .collect(),
            resid_scale_shifts: vec![4; STAGES],
            norm_fold_const: 10,
        }
    };
    let rot = compiler::derive_rotations();
    let resid_window = [1u32, 2];
    let resid_bundle = runtime::bundle_window_plain(&resid_art, &rot, &resid_window);
    let mut resid_rt = runtime::Runtime::new(&resid_art);
    let ((resid_plain, resid_kernel), resid_cen) = measure(|| {
        (
            runtime::assign_code_for_bundle(&resid_art, &resid_bundle),
            resid_rt.assign_window(&resid_window),
        )
    });
    assert_eq!(
        resid_plain, resid_kernel,
        "residual kernel/plain assignment agree"
    );
    assert_eq!(
        resid_cen, ZERO,
        "residual-wired assign path must be allocation-free"
    );
    println!(
        "[runtime] #318 residual assign (plain + kernel, synthetic TLA7) \
         → {} allocations, {} bytes",
        resid_cen.allocations, resid_cen.bytes
    );

    // Phase 4 — per-token op census from the Runtime's public OpKernel.
    let t = GEN_TOKENS as f64;
    println!(
        "[kernel] ops over {GEN_TOKENS} generated tokens: adds {} | xors {} | \
         shifts {} | compares {} | table-reads {} | candidate-scans {}",
        gen_ops.adds,
        gen_ops.xors,
        gen_ops.shifts,
        gen_ops.compares,
        gen_ops.table_reads,
        gen_ops.candidate_scans
    );
    println!(
        "[kernel] per generated token (avg): adds {:.1} | xors {:.1} | \
         shifts {:.1} | compares {:.1} | table-reads {:.1} | candidate-scans {:.1}",
        gen_ops.adds as f64 / t,
        gen_ops.xors as f64 / t,
        gen_ops.shifts as f64 / t,
        gen_ops.compares as f64 / t,
        gen_ops.table_reads as f64 / t,
        gen_ops.candidate_scans as f64 / t
    );
    println!("[kernel] cumulative {}", rt.kernel.report());
    let tokens: Vec<u32> = out.iter().map(|p| p.token).collect();
    let depths: Vec<u8> = out.iter().map(|p| p.depth).collect();
    println!("[gen] tokens {tokens:?}");
    println!("[gen] depths {depths:?}");

    // Phase 5 — the store write path, KNOWN to allocate (heap
    // Vec<BTreeMap>): measured on a scratch store, reported, never asserted.
    let mut scratch: Store = (0..=STAGES).map(|_| Default::default()).collect();
    let (_, ev_cen) = measure(|| {
        for i in 0..64u16 {
            let code = [(i % 251) as u8, ((i / 4) % 256) as u8, (i % 17) as u8, 0];
            runtime::add_evidence(&mut scratch, &code, (i.wrapping_mul(7)) as u32, 1);
        }
    });
    println!(
        "[store] add_evidence × 64 (known-allocating write path) \
         → {} allocations, {} bytes (report only)",
        ev_cen.allocations, ev_cen.bytes
    );

    // Phase 6 - R4G1 zero-allocation verification
    println!("=== allocation census: R4G1 zero-multiply prediction runtime ===");

    // Parse fixture artifacts and generate an R4G1 container
    let (art_r4g1, _) = parse_artifacts_measured();
    let (store_r4g1, _) = parse_store_measured();
    let store_bytes = uor_r4_core::transformerless::runtime::store_bytes(&store_r4g1);
    let art_bytes = std::fs::read(fixture_path("tless_artifacts.bin"))
        .unwrap_or_else(|_| std::fs::read(real_path("tless_artifacts.bin")).unwrap());

    let (r4g1_bytes, _) = uor_r4_core::transformerless::convert_r4g1::convert(
        &art_bytes,
        &art_r4g1,
        &store_r4g1,
        &store_bytes,
        None,
    )
    .expect("convert to R4G1 succeeds");

    let (runtime, parse_cen) =
        measure(|| uor_r4_graph_runtime::R4G1Runtime::parse(&r4g1_bytes).unwrap());
    println!(
        "[r4g1 parse] R4G1Runtime::parse → {} allocations, {} bytes (report only)",
        parse_cen.allocations, parse_cen.bytes
    );

    let context_tokens = [1u32, 2, 3, 4, 5];
    let mut node_scores = [uor_r4_core::transformerless::score_q::ScoreQ::MIN; 64];
    let mut candidates = [(0u32, uor_r4_core::transformerless::score_q::ScoreQ::MIN); 8];
    let mut state = uor_r4_graph_runtime::runtime_state::RuntimeState::default();
    let mut witness = [0u8; 0];

    // Warm-up prediction before steady-state measurement
    for _ in 0..10 {
        let _ = runtime.predict_token(&context_tokens, None, &mut node_scores);
        let _ = runtime.predict_distribution(&context_tokens, None, &mut node_scores);
        let _ =
            runtime.predict_candidates(&context_tokens, None, &mut node_scores, &mut candidates);
        let _ = runtime.step(&mut state, context_tokens[0], &mut witness);
    }

    let (_, predict_cen) = measure(|| {
        for _ in 0..10 {
            let _ = runtime.predict_token(&context_tokens, None, &mut node_scores);
            let _ = runtime.predict_distribution(&context_tokens, None, &mut node_scores);
            let _ = runtime.predict_candidates(
                &context_tokens,
                None,
                &mut node_scores,
                &mut candidates,
            );
            let _ = runtime.step(&mut state, context_tokens[0], &mut witness);
        }
    });

    println!(
        "[r4g1 kernels] predict_token + predict_distribution + predict_candidates + step × 10 → {} allocations, {} bytes",
        predict_cen.allocations, predict_cen.bytes
    );

    assert_eq!(
        predict_cen, ZERO,
        "R4G1 runtime kernels must be allocation-free in steady state"
    );
}
