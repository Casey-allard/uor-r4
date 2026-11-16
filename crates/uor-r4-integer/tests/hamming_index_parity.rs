//! The Hamming read's per-head index must serve exactly what the scan serves.
//!
//! Two sessions of the same artifact and stream: one selects from the key index (the default), one
//! scans every earlier position. Tokens, kept positions and the read's trace must agree at every
//! step; the index only changes how the same answer is found.

#![cfg(all(feature = "serving-wasm", feature = "fbs-synthetic"))]

use uor_r4_integer::stack::synthetic::{
    synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth, FBS_SYNTH_SMALL,
};
use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, IntegerStackModel};

#[test]
fn the_indexed_hamming_read_serves_identically_to_the_scan() {
    let shape = FbsSynth {
        context: 128,
        ..FBS_SYNTH_SMALL
    };
    let mut snap = synthetic_fbs_trunk_with(20_261_200, shape);
    snap.hamming = Some(synthetic_hamming(
        snap.width,
        snap.layers.len(),
        4,
        20_261_201,
    ));
    let bytes = export_fbs_trunk_uorlut01(&snap).expect("export a read-bearing trunk");
    let model = IntegerStackModel::parse(&bytes).expect("parse the read-bearing trunk");

    let mut indexed = model.session();
    let mut scan = model.session();
    scan.set_hamming_index(false);
    indexed.set_hamming_trace(true);
    scan.set_hamming_trace(true);

    let ids: Vec<u32> = (0..96)
        .map(|step| ((step * 37 + step / 5) % 64) as u32)
        .collect();
    for (step, &token) in ids.iter().enumerate() {
        let a = indexed.step_native(token).expect("indexed step");
        let b = scan.step_native(token).expect("scan step");
        assert_eq!(a, b, "step {step}: the served token moved");
        assert_eq!(
            indexed.take_hamming_trace(),
            scan.take_hamming_trace(),
            "step {step}: the read's choices moved"
        );
    }

    // A reset clears the index; the same stream must still agree.
    indexed.reset();
    scan.reset();
    for (step, &token) in ids.iter().enumerate() {
        let a = indexed.step_native(token).expect("indexed step");
        let b = scan.step_native(token).expect("scan step");
        assert_eq!(a, b, "after reset, step {step}");
    }
}
