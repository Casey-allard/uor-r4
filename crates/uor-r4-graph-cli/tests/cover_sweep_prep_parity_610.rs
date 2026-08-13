//! #610 parity arm — caching the cover-independent scoring preparation must
//! not change any emitted byte or scored metric.
//!
//! #610 moves the vocabulary and the whole-corpus context/forward-anchor rows
//! out of the per-point loop into a single [`cover_sweep::PreparedScoring`]
//! built once per sweep. That is a pure caching move: the values are functions
//! of the corpus, teacher artifact, and fixed scorer only — never of the
//! induced cover — so a point scored against the shared context must produce
//! byte-for-byte the same scored artifact and the same Gate C / reconstruction
//! metrics as a point that rebuilt the context for itself (the old behaviour).
//!
//! This harness runs every grid point twice on the pinned fixtures: once with
//! a single shared prepared context (the new default) and once with a context
//! prepared fresh for that point (equivalent to the pre-#610 per-point
//! recomputation). It asserts the emitted artifact bytes are identical and the
//! full report rows (regions, recall, artifact bytes, κ, Gate C, and the #456
//! reconstruction block) serialize identically.
//!
//! `#[ignore]`d and presence-gated: it needs the pinned fixtures and runs Gate
//! C twice per point. Run:
//!   R4_GATE_C_SAMPLE=200 \
//!   cargo test --release -p uor-r4-graph-cli --test cover_sweep_prep_parity_610 \
//!     -- --ignored --nocapture

use std::path::PathBuf;

use uor_r4_graph_certify::ScoreConfig;
use uor_r4_graph_cli::cover_sweep::{PreparedScoring, load_inputs, run_point, sweep_grid};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uor-r4-core/tests/fixtures")
        .join(name)
}

#[test]
#[ignore = "#610 parity arm; needs fixtures + two Gate C passes per point — run with --ignored"]
fn shared_preparation_matches_per_point_recomputation() {
    let meta = fixture("c_meta.bin");
    let recs = fixture("c_recs.bin");
    let art = fixture("tless_artifacts.bin");
    if !meta.exists() || !recs.exists() || !art.exists() {
        eprintln!("cover_sweep_prep_parity_610: fixtures absent, skipping (κ-test convention)");
        return;
    }

    let inputs = load_inputs(&meta, &recs, &art).expect("load sweep inputs");
    let config = ScoreConfig::default();

    // The new default: one shared cover-independent context for the run.
    let shared = PreparedScoring::prepare(&inputs, &config);

    let mut points_checked = 0usize;
    for point in sweep_grid() {
        // Shared-context arm (post-#610).
        let (row_shared, _base_shared, bytes_shared, _t_shared) =
            run_point(&inputs, &point, &config, &shared).expect("shared-context point");

        // Fresh-per-point arm: rebuilding the context for this point is
        // exactly the pre-#610 behaviour, so any divergence is a caching bug.
        let fresh = PreparedScoring::prepare(&inputs, &config);
        let (row_fresh, _base_fresh, bytes_fresh, _t_fresh) =
            run_point(&inputs, &point, &config, &fresh).expect("fresh-context point");

        assert_eq!(
            bytes_shared, bytes_fresh,
            "point {}: emitted artifact bytes diverged between shared and per-point \
             preparation — the cache changed a result",
            point.label
        );

        let json_shared = serde_json::to_value(&row_shared).expect("row serializes");
        let json_fresh = serde_json::to_value(&row_fresh).expect("row serializes");
        assert_eq!(
            json_shared, json_fresh,
            "point {}: report metrics diverged between shared and per-point preparation",
            point.label
        );

        points_checked += 1;
        eprintln!(
            "#610 parity — point {} OK ({} artifact bytes, identical both arms)",
            point.label,
            bytes_shared.len()
        );
    }

    // Anti-vacuity: the grid actually produced points to compare.
    assert!(
        points_checked >= 9,
        "expected the full 9-point grid, compared only {points_checked}"
    );
}
