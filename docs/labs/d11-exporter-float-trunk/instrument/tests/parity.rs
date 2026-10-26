//! The host twin of the FBS core equals the Rust integer session bit for bit
//! (token, kept cells and every Q12 query value) at every position of the
//! synthetic controls (claude-128, ORDER done-when 2).

use d11_fbs_evaluator::{first_difference, host_sequences, layouts_agree, session_sequences};
use uor_r4_integer::stack::synthetic::{
    fbs_synthetic_non_square, fbs_synthetic_tiny_gain, synthetic_fbs_trunk,
    synthetic_fbs_trunk_with, Lcg, FBS_SYNTH_NON_SQUARE,
};
use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, FbsTrunkSnapshot, IntegerStackModel};

fn model_of(snap: &FbsTrunkSnapshot) -> IntegerStackModel {
    IntegerStackModel::parse(&export_fbs_trunk_uorlut01(snap).expect("export")).expect("parse")
}

/// Every token of the vocabulary, each from a fresh session.
fn single_tokens(vocab: usize) -> Vec<Vec<u32>> {
    (0..vocab as u32).map(|t| vec![t]).collect()
}

/// `count` sequences of `len` tokens drawn from `seed`.
fn sequences(vocab: usize, count: usize, len: usize, seed: u64) -> Vec<Vec<u32>> {
    let mut rng = Lcg(seed);
    (0..count)
        .map(|_| {
            (0..len)
                .map(|_| (rng.next() % vocab as u64) as u32)
                .collect()
        })
        .collect()
}

fn assert_parity(name: &str, model: &IntegerStackModel, seqs: &[Vec<u32>]) {
    let host = host_sequences(model, seqs).expect("host twin");
    let rust = session_sequences(model, seqs).expect("Rust session");
    let positions = host.sequences * host.len;
    eprintln!("{name}: {positions} positions compared");
    assert!(positions > 0, "{name}: nothing was compared");
    assert_eq!(
        first_difference(&host, &rust),
        None,
        "{name}: the host twin departs from the Rust session"
    );
    assert_eq!(host, rust, "{name}");
}

#[test]
fn the_c_and_rust_struct_sizes_agree() {
    assert!(layouts_agree());
}

#[test]
fn host_twin_equals_the_session_on_single_tokens() {
    for (name, snap) in [
        ("small", synthetic_fbs_trunk(20261025)),
        ("non-square", fbs_synthetic_non_square(20261026)),
        ("tiny-gain", fbs_synthetic_tiny_gain(20261026)),
    ] {
        let model = model_of(&snap);
        assert_parity(name, &model, &single_tokens(snap.vocab));
    }
}

#[test]
fn host_twin_equals_the_session_over_sequences() {
    for (name, snap) in [
        ("small", synthetic_fbs_trunk(20261025)),
        ("non-square", fbs_synthetic_non_square(20261026)),
        ("tiny-gain", fbs_synthetic_tiny_gain(20261026)),
    ] {
        let model = model_of(&snap);
        assert_parity(name, &model, &sequences(snap.vocab, 16, snap.context, 7));
    }
    // State carried across 64 positions of the non-square trunk.
    let long = synthetic_fbs_trunk_with(
        20261027,
        uor_r4_integer::stack::synthetic::FbsSynth {
            context: 64,
            ..FBS_SYNTH_NON_SQUARE
        },
    );
    let model = model_of(&long);
    assert_parity(
        "non-square, 64 positions",
        &model,
        &sequences(long.vocab, 8, 64, 11),
    );
}
