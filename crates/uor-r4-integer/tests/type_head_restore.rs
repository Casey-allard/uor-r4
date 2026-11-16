//! A Hamming read's type heads keep a payload per position; a saved session now carries it, so a
//! restored type-head session continues exactly instead of being refused.
//!
//! `cargo test -p uor-r4-integer --features serving-wasm,fbs-synthetic --test type_head_restore`

#![cfg(all(feature = "serving-wasm", feature = "fbs-synthetic"))]

use uor_r4_integer::stack::synthetic::{
    synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth, FBS_SYNTH_SMALL,
};
use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, IntegerStackModel};

#[test]
fn a_restored_type_head_session_continues_exactly() {
    let shape = FbsSynth {
        context: 32,
        ..FBS_SYNTH_SMALL
    };
    let mut snap = synthetic_fbs_trunk_with(20_261_600, shape);
    snap.hamming = Some(synthetic_hamming(
        snap.width,
        snap.layers.len(),
        4,
        20_261_601,
    ));
    let bytes = export_fbs_trunk_uorlut01(&snap).expect("export a type-head trunk");

    let ids: Vec<u32> = (0..8).map(|step| (step * 11 + 1) % 64).collect();

    let model = IntegerStackModel::parse(&bytes).expect("parse");
    let mut original = model.session();
    original.set_hamming_trace(true);
    for &id in &ids[..4] {
        original.step_native(id).expect("prefix step");
    }
    let saved = original.save_state();
    // The trace is not saved; drop the prefix so the two sessions compare step by step.
    original.take_hamming_trace();

    let restored_model = IntegerStackModel::parse(&bytes).expect("parse again");
    let mut restored = restored_model.session();
    restored.set_hamming_trace(true);
    restored
        .restore_state(&saved)
        .expect("a type-head session restores with its payloads");

    for (step, &id) in ids[4..].iter().enumerate() {
        let a = original.step_native(id).expect("continued step");
        let b = restored.step_native(id).expect("restored step");
        assert_eq!(a, b, "step {}: the restored session diverged", step + 4);
        assert_eq!(
            original.take_hamming_trace(),
            restored.take_hamming_trace(),
            "step {}: the read's choices diverged",
            step + 4
        );
    }

    // A snapshot without the payloads (the older shape) is refused, not restored differently.
    let mut stripped = saved.clone();
    stripped.hamming_payloads.clear();
    let refused_model = IntegerStackModel::parse(&bytes).expect("parse again");
    let mut refused = refused_model.session();
    assert!(
        refused.restore_state(&stripped).is_err(),
        "a type-head session without its payloads must be refused"
    );
}

#[test]
fn a_type_head_session_saves_and_restores_through_a_file_at_production_shape() {
    // The payloads are `context * width` i64 a read: a shape with width 256 and context 1,024
    // makes the JSON large enough to exercise the file-size bound the in-memory test cannot.
    let shape = FbsSynth {
        vocab: 256,
        d: 256,
        mlp: 256,
        layers: 2,
        context: 1024,
        kept: 2,
        ..FBS_SYNTH_SMALL
    };
    let mut snap = synthetic_fbs_trunk_with(20_261_610, shape);
    snap.hamming = Some(synthetic_hamming(
        snap.width,
        snap.layers.len(),
        4,
        20_261_611,
    ));
    let bytes = export_fbs_trunk_uorlut01(&snap).expect("export a production-shape trunk");
    let model = IntegerStackModel::parse(&bytes).expect("parse");

    let ids: Vec<u32> = (0..8).map(|step| (step * 17 + 3) % 256).collect();
    let mut original = model.session();
    for &id in &ids[..4] {
        original.step_native(id).expect("prefix step");
    }

    let path = std::env::temp_dir().join(format!(
        "uor-type-head-{}-{}.json",
        std::process::id(),
        line!()
    ));
    original.save_session_to_file(&path).expect("save the session file");
    let size = std::fs::metadata(&path).expect("metadata").len();
    assert!(size > 1_000_000, "the payloads make the file large: {size}");

    let restored_model = IntegerStackModel::parse(&bytes).expect("parse again");
    let mut restored = restored_model.session();
    restored
        .restore_session_from_file(&path)
        .expect("the file restores within its bound");
    for (step, &id) in ids[4..].iter().enumerate() {
        assert_eq!(
            original.step_native(id).expect("continued step"),
            restored.step_native(id).expect("restored step"),
            "step {}",
            step + 4
        );
    }
    std::fs::remove_file(&path).ok();
}
