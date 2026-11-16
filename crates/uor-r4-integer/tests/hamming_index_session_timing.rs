//! The served Hamming read, indexed and scanned, in a whole session.
//!
//! The same read-bearing synthetic trunk and token stream are served twice: from the per-head key
//! index (the default) and with `set_hamming_index(false)`, which scans every earlier position.
//! Tokens must agree at every step. The times are printed, not asserted: a debug test is not a
//! benchmark, and the ratio grows with context because the scan is O(t) a step.
//!
//! `cargo test -p uor-r4-integer --features serving-wasm,fbs-synthetic --test hamming_index_session_timing -- --nocapture`

#![cfg(all(feature = "serving-wasm", feature = "fbs-synthetic"))]

use std::time::{Duration, Instant};
use uor_r4_integer::stack::synthetic::{
    synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth, FBS_SYNTH_SMALL,
};
use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, IntegerStackModel};

fn run(context: usize, index: bool) -> (Duration, Vec<u32>) {
    let shape = FbsSynth {
        context,
        ..FBS_SYNTH_SMALL
    };
    let mut snap = synthetic_fbs_trunk_with(20_261_400, shape);
    snap.hamming = Some(synthetic_hamming(
        snap.width,
        snap.layers.len(),
        4,
        20_261_401,
    ));
    let bytes = export_fbs_trunk_uorlut01(&snap).expect("export a read-bearing trunk");
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    let mut session = model.session();
    session.set_hamming_index(index);
    let ids: Vec<u32> = (0..context as u32)
        .map(|step| (step * 37 + step / 5) % 256)
        .collect();
    let started = Instant::now();
    let tokens = ids
        .iter()
        .map(|&id| session.step_native(id).expect("step").token)
        .collect();
    (started.elapsed(), tokens)
}

#[test]
fn the_indexed_session_matches_the_scan_and_its_ratio_grows_with_context() {
    let mut ratios = Vec::new();
    for context in [1024usize, 2048, 4096] {
        let (indexed, indexed_tokens) = run(context, true);
        let (scan, scan_tokens) = run(context, false);
        assert_eq!(
            indexed_tokens, scan_tokens,
            "context {context}: the indexed session served a different token"
        );
        let ratio = scan.as_secs_f64() / indexed.as_secs_f64();
        eprintln!(
            "context {context}: indexed {:?} ({:.0} tok/s), scan {:?} ({:.0} tok/s), {:.2}x",
            indexed,
            context as f64 / indexed.as_secs_f64(),
            scan,
            context as f64 / scan.as_secs_f64(),
            ratio
        );
        ratios.push(ratio);
    }
    // The scan is O(t) a step and the index is O(codes), so the ratio rises with the context. It
    // is asserted loosely, between the two measured ends, so a machine's absolute speed cannot
    // decide the test; only a wrong shape could.
    assert!(
        ratios.windows(2).all(|pair| pair[1] > pair[0]),
        "the scan/index ratio did not grow with context: {ratios:?}"
    );
    assert!(ratios[0] > 1.2, "the index is not faster at 1k: {ratios:?}");
}
