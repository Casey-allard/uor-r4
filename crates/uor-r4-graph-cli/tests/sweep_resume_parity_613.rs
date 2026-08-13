//! #613 resume parity arm — resuming from checkpoints must be byte-identical
//! to a clean run, and must genuinely reuse stored points.
//!
//! #613 persists each completed point atomically (`R4_SWEEP_CHECKPOINT_DIR`)
//! and, on a later run, reuses only points whose manifest, label, and cover
//! identity match. Because each point's row/metrics are deterministic, a reused
//! point equals a freshly-computed one, so the resumed report must equal a
//! clean run's — everywhere except the non-deterministic `timing` block.
//!
//! This harness:
//!   1. runs a clean sweep into checkpoint dir A (populating all nine points),
//!   2. resumes fully from A and asserts the report (minus timing) is identical,
//!   3. tampers ONE stored point's timing with a sentinel and confirms the
//!      resumed report carries it — proving the point was loaded, not recomputed,
//!   4. resumes from a PARTIAL dir (half the points copied) and asserts the
//!      report (minus timing) is still identical — the recomputed points match
//!      the reused ones.
//!
//! `#[ignore]`d and presence-gated: needs the pinned fixtures. Run:
//!   R4_GATE_C_SAMPLE=300 \
//!   cargo test --release -p uor-r4-graph-cli --test sweep_resume_parity_613 \
//!     -- --ignored --nocapture

use std::path::{Path, PathBuf};

use uor_r4_graph_certify::ScoreConfig;
use uor_r4_graph_cli::cover_sweep::{SweepReport, load_inputs, run_sweep};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uor-r4-core/tests/fixtures")
        .join(name)
}

fn report_without_timing(report: &SweepReport) -> serde_json::Value {
    let mut value = serde_json::to_value(report).expect("report serializes");
    value
        .as_object_mut()
        .expect("report is a JSON object")
        .remove("timing");
    value
}

fn set_checkpoint_dir(dir: &Path) {
    unsafe { std::env::set_var("R4_SWEEP_CHECKPOINT_DIR", dir) };
}

#[test]
#[ignore = "#613 resume parity; needs fixtures + repeated sweeps — run with --ignored"]
fn resume_is_byte_identical_to_a_clean_run() {
    let meta = fixture("c_meta.bin");
    let recs = fixture("c_recs.bin");
    let art = fixture("tless_artifacts.bin");
    if !meta.exists() || !recs.exists() || !art.exists() {
        eprintln!("sweep_resume_parity_613: fixtures absent, skipping (κ-test convention)");
        return;
    }

    let inputs = load_inputs(&meta, &recs, &art).expect("load sweep inputs");
    let config = ScoreConfig::default();

    let base = std::env::temp_dir().join("uor_r4_i613_resume");
    let dir_a = base.join("clean");
    let dir_partial = base.join("partial");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&dir_a).expect("mkdir A");
    std::fs::create_dir_all(&dir_partial).expect("mkdir partial");

    // (1) Clean run populates dir A with all nine point files.
    set_checkpoint_dir(&dir_a);
    let clean = run_sweep(&inputs, &config, 0.0).expect("clean sweep");
    let clean_json = report_without_timing(&clean);
    let files: Vec<PathBuf> = std::fs::read_dir(&dir_a)
        .expect("read dir A")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    assert_eq!(
        files.len(),
        clean.points.len(),
        "one checkpoint file per point"
    );

    // (3) Tamper point-00's stored timing with a sentinel BEFORE the full
    // resume, so a reused point is provable: the sentinel can only appear in
    // the resumed report if the point was loaded from disk, not recomputed.
    const SENTINEL: u64 = 999_999;
    let p0 = dir_a.join("point-00.json");
    let mut stored: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&p0).expect("read point-00")).expect("parse");
    stored["timing"]["cover_induction_ms"] = serde_json::json!(SENTINEL);
    std::fs::write(&p0, serde_json::to_vec_pretty(&stored).unwrap()).expect("rewrite point-00");

    // (2) Full resume from A: report (minus timing) identical to clean.
    set_checkpoint_dir(&dir_a);
    let resumed = run_sweep(&inputs, &config, 0.0).expect("full resume");
    assert_eq!(
        report_without_timing(&resumed),
        clean_json,
        "full resume diverged from the clean run (report minus timing)"
    );
    assert_eq!(
        resumed.timing.points[0].cover_induction_ms, SENTINEL,
        "point 0 was recomputed, not loaded from its checkpoint"
    );

    // (4) Partial resume: copy the even-indexed points into a fresh dir, run,
    // and confirm the recomputed odd points match the reused even ones.
    for (i, path) in files.iter().enumerate() {
        if i % 2 == 0 {
            let name = path.file_name().expect("file name");
            std::fs::copy(path, dir_partial.join(name)).expect("copy checkpoint");
        }
    }
    set_checkpoint_dir(&dir_partial);
    let partial = run_sweep(&inputs, &config, 0.0).expect("partial resume");
    assert_eq!(
        report_without_timing(&partial),
        clean_json,
        "partial resume diverged from the clean run (report minus timing)"
    );

    unsafe { std::env::remove_var("R4_SWEEP_CHECKPOINT_DIR") };
    let _ = std::fs::remove_dir_all(&base);
    eprintln!(
        "#613 resume — clean == full-resume == partial-resume (report minus timing); \
         reuse proven via the point-0 timing sentinel"
    );
}
