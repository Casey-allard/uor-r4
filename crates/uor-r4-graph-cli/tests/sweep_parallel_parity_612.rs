//! #612 parity + benchmark arm — bounded-parallel point evaluation must be
//! byte-identical to serial, and should record a speedup.
//!
//! #612 evaluates the nine independent sweep points with a bounded worker pool
//! (`R4_SWEEP_JOBS`, default 1 = serial). Each point's internal Rayon passes
//! use the global pool exactly as in serial mode and reduce in input order, so
//! a point computes the same bytes whether it runs alone or beside others;
//! results are collected into fixed grid-order slots. The report and every
//! artifact byte must therefore equal the serial run's.
//!
//! This harness runs the full sweep at 1, 2, and a host-appropriate worker
//! count, then:
//!   1. asserts each parallel report equals the serial report EXCEPT the
//!      `timing` block (thread_count and wall-clock legitimately differ;
//!      `graph_kappa` is the blake3 of each artifact, so equal reports prove
//!      byte-identical artifacts), and
//!   2. records the wall-clock speedup at each worker count.
//!
//! `#[ignore]`d and presence-gated: it needs the pinned fixtures and runs the
//! full sweep several times. Run:
//!   R4_GATE_C_SAMPLE=300 \
//!   cargo test --release -p uor-r4-graph-cli --test sweep_parallel_parity_612 \
//!     -- --ignored --nocapture

use std::path::PathBuf;

use uor_r4_graph_certify::ScoreConfig;
use uor_r4_graph_cli::cover_sweep::{load_inputs, run_sweep};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uor-r4-core/tests/fixtures")
        .join(name)
}

/// Serialize the report and drop the non-deterministic `timing` block so two
/// reports can be compared for byte-equal content (graph_kappa included).
fn report_without_timing(report: &uor_r4_graph_cli::cover_sweep::SweepReport) -> serde_json::Value {
    let mut value = serde_json::to_value(report).expect("report serializes");
    value
        .as_object_mut()
        .expect("report is a JSON object")
        .remove("timing");
    value
}

#[test]
#[ignore = "#612 parity/benchmark; needs fixtures + several full sweeps — run with --ignored"]
fn parallel_sweep_matches_serial_and_records_speedup() {
    let meta = fixture("c_meta.bin");
    let recs = fixture("c_recs.bin");
    let art = fixture("tless_artifacts.bin");
    if !meta.exists() || !recs.exists() || !art.exists() {
        eprintln!("sweep_parallel_parity_612: fixtures absent, skipping (κ-test convention)");
        return;
    }

    let inputs = load_inputs(&meta, &recs, &art).expect("load sweep inputs");
    let config = ScoreConfig::default();

    // Serial reference (R4_SWEEP_JOBS unset -> jobs = 1).
    unsafe { std::env::remove_var("R4_SWEEP_JOBS") };
    let serial_start = std::time::Instant::now();
    let serial = run_sweep(&inputs, &config, 0.0).expect("serial sweep");
    let serial_ms = serial_start.elapsed().as_millis();
    assert_eq!(
        serial.timing.thread_count, 1,
        "serial run must use one worker"
    );
    let serial_json = report_without_timing(&serial);

    let host = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    let mut worker_counts = vec![2usize];
    if host > 2 {
        worker_counts.push(host);
    }

    eprintln!("#612 benchmark — serial (1 worker): {serial_ms} ms");
    for jobs in worker_counts {
        unsafe { std::env::set_var("R4_SWEEP_JOBS", jobs.to_string()) };
        let start = std::time::Instant::now();
        let parallel = run_sweep(&inputs, &config, 0.0).expect("parallel sweep");
        let ms = start.elapsed().as_millis();

        assert!(
            parallel.timing.thread_count >= 2,
            "parallel run at jobs={jobs} reported thread_count {}",
            parallel.timing.thread_count
        );
        assert_eq!(
            report_without_timing(&parallel),
            serial_json,
            "parallel sweep at jobs={jobs} diverged from serial (report minus timing)"
        );

        let speedup = if ms == 0 {
            f64::from(u32::try_from(serial_ms).unwrap_or(u32::MAX))
        } else {
            serial_ms as f64 / ms as f64
        };
        eprintln!(
            "#612 benchmark — {jobs} workers: {ms} ms ({speedup:.2}x vs serial); \
             report byte-identical to serial"
        );
    }
    unsafe { std::env::remove_var("R4_SWEEP_JOBS") };
}
