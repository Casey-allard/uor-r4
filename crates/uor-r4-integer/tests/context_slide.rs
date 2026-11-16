//! A read-free model may slide its token window at the context bound; a model whose reads name
//! positions may not.
//!
//! The recurrence and the native Q31 state carry the history; the token vector is only a read's
//! window. With no Hamming/place read and no turn memory, dropping the oldest token changes nothing
//! the model computes. This test steps past the bound, checks the window, checks determinism, and
//! checks that a read-bearing model refuses the policy.

#![cfg(all(feature = "serving-wasm", feature = "fbs-synthetic"))]

use uor_r4_integer::stack::synthetic::{
    synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth, FBS_SYNTH_SMALL,
};
use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, IntegerStackModel, StackError};

fn ids() -> Vec<u32> {
    (0..16).map(|step| (step * 13 + 1) % 64).collect()
}

#[test]
fn a_read_free_model_slides_at_the_bound_and_a_read_bearing_one_refuses() {
    let shape = FbsSynth {
        context: 8,
        ..FBS_SYNTH_SMALL
    };
    let snap = synthetic_fbs_trunk_with(20_261_500, shape);
    let bytes = export_fbs_trunk_uorlut01(&snap).expect("export a read-free trunk");
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    assert!(model.can_slide_tokens(), "the trunk is read-free");

    // Without the policy, the ninth step refuses.
    let mut refusing = model.session();
    let refused = {
        let mut result = Ok(());
        for &id in ids().iter().take(9) {
            result = refusing.step_native(id).map(|_| ());
            if result.is_err() {
                break;
            }
        }
        result
    };
    assert!(
        matches!(refused, Err(StackError::ContextFull { .. })),
        "a full context without the policy must refuse"
    );

    // With the policy, all sixteen steps serve, and the window holds the last eight ids.
    let mut sliding = model.session();
    sliding.set_context_slide(true).expect("a read-free model slides");
    let served: Vec<u32> = ids()
        .iter()
        .map(|&id| sliding.step_native(id).expect("slid step").token)
        .collect();
    assert_eq!(served.len(), 16);
    assert_eq!(
        sliding.tokens(),
        ids()[8..].to_vec(),
        "the window is the last context ids"
    );

    // Two runs of the same policy serve the same tokens.
    let mut again = model.session();
    again.set_context_slide(true).expect("slide");
    let repeated: Vec<u32> = ids()
        .iter()
        .map(|&id| again.step_native(id).expect("slid step").token)
        .collect();
    assert_eq!(served, repeated, "the slide is deterministic");

    // A Hamming read names token positions, so the policy is refused there.
    let mut read_snap = synthetic_fbs_trunk_with(20_261_501, shape);
    read_snap.hamming = Some(synthetic_hamming(
        read_snap.width,
        read_snap.layers.len(),
        4,
        20_261_502,
    ));
    let read_bytes = export_fbs_trunk_uorlut01(&read_snap).expect("export a read-bearing trunk");
    let read_model = IntegerStackModel::parse(&read_bytes).expect("parse");
    assert!(!read_model.can_slide_tokens());
    let mut read_session = read_model.session();
    assert!(read_session.set_context_slide(true).is_err());
}
