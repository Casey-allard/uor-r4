//! Snap tests of the multiplier-free (D11) stack engine against the float
//! model's snapped forward pass: the integer kernel must select the same
//! icosian root as the float nearest-root rule at every (layer, position,
//! lane) of a real forward, apart from exact f32 near-ties, and the served
//! top-1 logits must agree with the float side within the fixed-point gap.
//!
//! Run in release (`cargo test -p uor-r4-training --release --test
//! stack_d11_snap`).

use candle_core::Device;
use serde_json::json;
use uor_r4_integer::stack::{IntegerStackModel, ICOSIAN_ROOTS_SHA256};
use uor_r4_training::geometric_stack::{
    ReadScore, StackArch, StackConfig, StackModel, TransportSnap,
};
use uor_r4_training::stack_export::export_stack;

/// A small snapped `ra` stack.
fn small_snapped(seed: u64) -> StackModel {
    let mut config = StackConfig::transformer_control(seed);
    config.arch = StackArch::Geometric;
    config.vocab_size = 96;
    config.width = 32;
    config.heads = 2;
    config.mlp_hidden = 32;
    config.context = 8;
    config.pattern = "ra".to_owned();
    config.read = ReadScore::Lorentz;
    config.rotation = true;
    let mut model = StackModel::new(config, &Device::Cpu).expect("small stack");
    model
        .set_transport_snap(Some(TransportSnap::Icosian))
        .expect("snap");
    model
}

#[test]
fn the_kernel_roots_table_matches_the_float_roots_elementwise() {
    let float_roots = TransportSnap::Icosian.roots();
    assert_eq!(float_roots.len(), 120);
    assert_eq!(TransportSnap::Icosian.roots_sha256(), ICOSIAN_ROOTS_SHA256);
    for (j, root) in float_roots.iter().enumerate() {
        let [a0, b0] = uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS[j][0];
        let [a1, b1] = uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS[j][1];
        let [a2, b2] = uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS[j][2];
        let [a3, b3] = uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS[j][3];
        let phi = (1.0f32 + 5.0f32.sqrt()) / 2.0;
        let expected = [
            (f32::from(a0) + f32::from(b0) * phi) / 2.0,
            (f32::from(a1) + f32::from(b1) * phi) / 2.0,
            (f32::from(a2) + f32::from(b2) * phi) / 2.0,
            (f32::from(a3) + f32::from(b3) * phi) / 2.0,
        ];
        for c in 0..4 {
            assert!(
                (root[c] - expected[c]).abs() <= 1e-6,
                "root {j} component {c}: {} vs {}",
                root[c],
                expected[c]
            );
        }
    }
}

#[test]
fn the_served_snapped_forward_matches_the_float_side() {
    let model = small_snapped(0xD11);
    let ids: Vec<u32> = (0..8).map(|i| (i * 7 + 3) % 96).collect();
    let (bytes, _) = export_stack(
        &model,
        json!({"test": "d11-snap"}),
        None,
        Some(TransportSnap::Icosian),
    )
    .expect("export");
    let integer = IntegerStackModel::parse(&bytes).expect("parse");
    assert!(integer.transport_snap().is_some());

    // Float snapped forward, position by position.
    let float_logits = model
        .forward(&ids, 1, 8)
        .expect("forward")
        .to_vec2::<f32>()
        .expect("logits");
    // Float snap selections with margins.
    let selections = model.snap_selections(&ids, 1, 8).expect("selections");

    let mut session = integer.session();
    session.enable_snap_trace();
    let mut near_ties = 0usize;
    let mut compared = 0usize;
    let mut top1_agree = 0usize;
    let mut max_gap = 0f64;
    for (t, &id) in ids.iter().enumerate() {
        let logits = session.step(id).expect("step").to_vec();
        let float_row = &float_logits[t];
        let top_i = logits
            .iter()
            .enumerate()
            .max_by_key(|&(_, &v)| v)
            .map(|(i, _)| i)
            .expect("top");
        let top_f = float_row
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).expect("ord"))
            .map(|(i, _)| i)
            .expect("top");
        top1_agree += usize::from(top_i == top_f);
        for (&a, &b) in logits.iter().zip(float_row) {
            max_gap = max_gap.max((f64::from(a) / 65536.0 - f64::from(b)).abs());
        }
    }
    let trace = session.snap_trace().expect("trace");
    let lanes = 32 / 4;
    assert_eq!(trace.len(), 8 * lanes);
    for (k, entry) in trace.iter().enumerate() {
        let t = k / lanes;
        let lane = k % lanes;
        let selection = selections[&0][t * lanes + lane];
        if selection.margin < 1e-4 {
            near_ties += 1;
            continue;
        }
        compared += 1;
        assert_eq!(
            entry.root, selection.index,
            "position {t} lane {lane}: kernel {} vs float {}",
            entry.root, selection.index
        );
    }
    assert!(compared > 0, "every selection was a near-tie");
    assert_eq!(top1_agree, ids.len(), "top-1 disagreement");
    // The gap is dominated by weight quantization (4-bit-mantissa grid
    // codes, relative error up to 2^-4 per tap) accumulated over the layer,
    // not by the snap: the root selections above already matched exactly.
    // The budget is a regression bound on that quantization error, observed
    // at ~0.055 nats on this fixture.
    assert!(
        max_gap <= 0.1,
        "logit gap {max_gap} exceeds the quantization budget"
    );
    eprintln!("near-ties skipped: {near_ties} of {}", 8 * lanes);
}
