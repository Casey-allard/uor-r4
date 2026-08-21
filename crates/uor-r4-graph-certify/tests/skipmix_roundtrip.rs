//! SKMX/PSIB skip-mix section round-trip tests (#897): the optional
//! primary joint table and Ψ-bag fallback table survive `emit_scored_r4g1`
//! byte-exact, are absent (not merely empty) when no rows are given
//! (absent-section identity, matching the #399 FWDA precedent), and the
//! emitted artifact still validates end-to-end (two-stage parse + CIDs).

use std::collections::BTreeMap;

use uor_r4_core::transformerless::compiler::{SIG_BYTES, STAGES};
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_certify::score::{
    emit_scored_r4g1, ContextRow, EmissionTables, QuantizationErrorStats, ScoredGraphSections,
    Smoothing,
};
use uor_r4_graph_certify::score_runtime::RegionParams;
use uor_r4_graph_format::{GraphView, ScoreQ, SkipmixRowInput};

const VOCAB: u32 = 100;
const ROOT_FLOOR_RAW: i32 = -500_000;

fn emissions() -> EmissionTables {
    let mut root_prior = BTreeMap::new();
    root_prior.insert(1u32, ScoreQ::from_raw(-50_000));
    EmissionTables {
        root_prior,
        root_floor: ScoreQ::from_raw(ROOT_FLOOR_RAW),
        root_total: 10,
        region_lists: vec![Vec::new()],
        smoothing: Smoothing::AddOne,
        root_prior_quantization: QuantizationErrorStats::default(),
        emission_quantization: QuantizationErrorStats::default(),
        selection_stats: Default::default(),
    }
}

/// A minimal but fully valid scored artifact carrying the given skip-mix
/// rows (empty means the corresponding section is not emitted at all).
fn tiny_artifact(
    skipmix_rows: &[SkipmixRowInput],
    psi_bag_rows: &[(u32, Vec<(u32, i32)>)],
) -> (Vec<u8>, uor_r4_graph_certify::score::ScoredGraphInfo) {
    let regions = vec![RegionParams {
        node: 1,
        depth: 1,
        radius: 0,
        sig: [0; SIG_BYTES],
        parent: None,
    }];
    let context_rows = vec![ContextRow {
        context_len: 1,
        key0: 6,
        key1: 0,
        entries: vec![(7, ScoreQ::from_raw(-100_000))],
    }];
    let store: runtime::Store = vec![BTreeMap::new(); STAGES + 1];
    let tls1 = runtime::store_bytes(&store);
    let emissions = emissions();
    let sections = ScoredGraphSections {
        regions: &regions,
        structural: &[],
        transitions: &[],
        transition_quantization: QuantizationErrorStats::default(),
        emissions: &emissions,
        context_rows: &context_rows,
        exct_tls1: &tls1,
        exct_top_x: 3,
        fwd_rows: &[],
        skipmix_rows,
        psi_bag_rows,
    };
    emit_scored_r4g1(b"teacher-container", (b"meta", b"recs"), VOCAB, &sections)
}

fn sample_skipmix_rows() -> Vec<SkipmixRowInput> {
    vec![
        (10, 1, vec![(3, 100), (7, -50)]),
        (5, 9, vec![(1, 200)]),
        (2001, 77, vec![(2, 10), (4, 20), (9, 30)]),
    ]
}

fn sample_psi_bag_rows() -> Vec<(u32, Vec<(u32, i32)>)> {
    vec![(10, vec![(3, 100), (7, -50)]), (20, vec![(9, 30)])]
}

#[test]
fn absent_when_no_rows_given() {
    let (bytes, info) = tiny_artifact(&[], &[]);
    assert_eq!(info.skipmix_row_count, 0);
    assert_eq!(info.skipmix_bytes, 0);
    assert_eq!(info.psi_bag_row_count, 0);
    assert_eq!(info.psi_bag_bytes, 0);

    let view = GraphView::parse(&bytes).expect("artifact without skip-mix sections re-validates");
    view.verify_cids().expect("CIDs verify");
    assert!(
        view.skipmix_table().expect("parses").is_none(),
        "no SKMX section when no rows were given"
    );
    assert!(
        view.psi_bag_table().expect("parses").is_none(),
        "no PSIB section when no rows were given"
    );
}

#[test]
fn skipmix_and_psi_bag_rows_roundtrip_exactly() {
    let skipmix_rows = sample_skipmix_rows();
    let psi_bag_rows = sample_psi_bag_rows();
    let (bytes, info) = tiny_artifact(&skipmix_rows, &psi_bag_rows);
    assert_eq!(info.skipmix_row_count as usize, skipmix_rows.len());
    assert!(info.skipmix_bytes > 0);
    assert_eq!(info.psi_bag_row_count as usize, psi_bag_rows.len());
    assert!(info.psi_bag_bytes > 0);

    let view = GraphView::parse(&bytes).expect("artifact with skip-mix sections re-validates");
    view.verify_cids().expect("CIDs verify");

    let skmx = view
        .skipmix_table()
        .expect("SKMX section parses")
        .expect("SKMX section is present");
    assert!(skmx.capacity().is_power_of_two());
    let row = skmx.find(10, 1).expect("key (10, 1) is found");
    let entries: Vec<(u32, i32)> = row
        .entries()
        .iter()
        .map(|e| (e.token, e.score_q.raw()))
        .collect();
    assert_eq!(entries, vec![(3, 100), (7, -50)]);
    assert!(skmx.find(10, 2).is_none());

    let psib = view
        .psi_bag_table()
        .expect("PSIB section parses")
        .expect("PSIB section is present");
    let row = psib.find(20).expect("key 20 is found");
    let entries: Vec<(u32, i32)> = row
        .entries()
        .iter()
        .map(|e| (e.token, e.score_q.raw()))
        .collect();
    assert_eq!(entries, vec![(9, 30)]);
    assert!(psib.find(999).is_none());
}

/// Only the skip-mix section given non-empty rows is emitted -- SKMX and
/// PSIB are independently optional, matching the deployed engine's
/// per-table absent-section handling.
#[test]
fn skmx_and_psib_are_independently_optional() {
    let skipmix_rows = sample_skipmix_rows();
    let (bytes, info) = tiny_artifact(&skipmix_rows, &[]);
    assert!(info.skipmix_row_count > 0);
    assert_eq!(info.psi_bag_row_count, 0);
    let view = GraphView::parse(&bytes).expect("re-validates");
    assert!(view.skipmix_table().expect("parses").is_some());
    assert!(view.psi_bag_table().expect("parses").is_none());

    let psi_bag_rows = sample_psi_bag_rows();
    let (bytes, info) = tiny_artifact(&[], &psi_bag_rows);
    assert_eq!(info.skipmix_row_count, 0);
    assert!(info.psi_bag_row_count > 0);
    let view = GraphView::parse(&bytes).expect("re-validates");
    assert!(view.skipmix_table().expect("parses").is_none());
    assert!(view.psi_bag_table().expect("parses").is_some());
}
