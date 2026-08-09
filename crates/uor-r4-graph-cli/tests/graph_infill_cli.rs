//! CLI surface test for the A-mode infill subcommand (issue #399):
//! `graph infill --artifact <R4G1> --skeleton <ids with _ for free>`
//! runs end-to-end against a tiny scored artifact written to disk, and
//! the argument parsing fails closed on malformed input.

use std::collections::BTreeMap;

use uor_r4_core::transformerless::compiler::SIG_BYTES;
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_certify::score::{
    ContextRow, EmissionTables, ForwardAnchorRow, QuantizationErrorStats, ScoredGraphSections,
    Smoothing, emit_scored_r4g1,
};
use uor_r4_graph_certify::score_runtime::RegionParams;
use uor_r4_graph_cli::{graph_command, graph_infill_command};
use uor_r4_graph_format::ScoreQ;

/// The fwda_roundtrip tiny artifact, reduced to what the CLI test
/// needs: one region, two unigram context rows, an empty exact-context
/// store, and one forward-anchor row.
fn tiny_artifact_bytes() -> Vec<u8> {
    let regions = vec![RegionParams {
        node: 1,
        depth: 1,
        radius: 0,
        sig: [0; SIG_BYTES],
        parent: None,
    }];
    let context_rows = vec![
        ContextRow {
            context_len: 1,
            key0: 6,
            key1: 0,
            entries: vec![
                (7, ScoreQ::from_raw(-100_000)),
                (20, ScoreQ::from_raw(-80_000)),
            ],
        },
        ContextRow {
            context_len: 1,
            key0: 20,
            key1: 0,
            entries: vec![
                (4, ScoreQ::from_raw(-100_000)),
                (5, ScoreQ::from_raw(-120_000)),
                (6, ScoreQ::from_raw(-80_000)),
            ],
        },
    ];
    let store: runtime::Store =
        vec![BTreeMap::new(); uor_r4_core::transformerless::compiler::STAGES + 1];
    let tls1 = runtime::store_bytes(&store);
    let mut root_prior = BTreeMap::new();
    root_prior.insert(1u32, ScoreQ::from_raw(-50_000));
    let emissions = EmissionTables {
        root_prior,
        root_floor: ScoreQ::from_raw(-500_000),
        root_total: 10,
        region_lists: vec![Vec::new()],
        smoothing: Smoothing::AddOne,
        root_prior_quantization: QuantizationErrorStats::default(),
        emission_quantization: QuantizationErrorStats::default(),
        selection_stats: Default::default(),
    };
    let fwd_rows = vec![ForwardAnchorRow {
        distance: 2,
        anchor: 6,
        total: 50,
        entries: vec![(5, 50)],
    }];
    let (bytes, info) = emit_scored_r4g1(
        b"teacher-container",
        (b"meta", b"recs"),
        100,
        &ScoredGraphSections {
            regions: &regions,
            structural: &[],
            transitions: &[],
            transition_quantization: QuantizationErrorStats::default(),
            emissions: &emissions,
            context_rows: &context_rows,
            exct_tls1: &tls1,
            exct_top_x: 3,
            fwd_rows: &fwd_rows,
        },
    );
    assert_eq!(info.fwda_row_count, 1);
    bytes
}

#[test]
fn graph_infill_runs_end_to_end_on_a_scored_artifact() {
    let dir = std::env::temp_dir().join(format!("r4-graph-infill-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let artifact = dir.join("score.r4g1");
    std::fs::write(&artifact, tiny_artifact_bytes()).expect("artifact written");

    let args = vec![
        "--artifact".to_owned(),
        artifact.display().to_string(),
        "--skeleton".to_owned(),
        "20,_,_,6".to_owned(),
    ];
    graph_infill_command(&args).expect("infill subcommand runs");

    // The `graph` family dispatches to the same command.
    let mut family = vec!["infill".to_owned()];
    family.extend(args);
    graph_command(&family).expect("graph dispatch runs");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn graph_infill_arguments_fail_closed() {
    let missing_artifact = graph_infill_command(&["--skeleton".to_owned(), "_".to_owned()]);
    assert!(
        missing_artifact
            .expect_err("artifact is required")
            .contains("--artifact")
    );

    let missing_skeleton = graph_infill_command(&[
        "--artifact".to_owned(),
        "/nonexistent/score.r4g1".to_owned(),
    ]);
    assert!(
        missing_skeleton
            .expect_err("skeleton is required")
            .contains("--skeleton")
    );

    let bad_slot = graph_infill_command(&[
        "--artifact".to_owned(),
        "/nonexistent/score.r4g1".to_owned(),
        "--skeleton".to_owned(),
        "12,x,_".to_owned(),
    ]);
    assert!(
        bad_slot
            .expect_err("non-numeric slot is rejected")
            .contains("invalid skeleton slot")
    );

    let unknown_flag = graph_infill_command(&["--bogus".to_owned(), "1".to_owned()]);
    assert!(
        unknown_flag
            .expect_err("unknown flags are rejected")
            .contains("unknown graph infill option")
    );

    let unknown_family = graph_command(&["bogus".to_owned()]);
    assert!(
        unknown_family
            .expect_err("unknown graph subcommands are rejected")
            .contains("graph commands")
    );
}
