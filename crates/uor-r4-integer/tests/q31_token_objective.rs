//! The first finite token objective in the one native Q31 space.
//!
//! Tokens map to the synthetic trunk's writer rows (Q12, shifted to Q31); at each transition the
//! traced recurrence lane carries are the values and the next token's matching 4-word block is the
//! target. The shared oracle `Q31HamiltonModel::step_reference` trains the gate. The test reports
//! the independent alignment dot before and after, and reports the served token ids before and
//! after. Host-only; no language or capability claim.
//!
//! `cargo test -p uor-r4-integer --features serving-wasm,fbs-synthetic --test q31_token_objective`

#![cfg(all(feature = "serving-wasm", feature = "fbs-synthetic"))]

use uor_r4_integer::stack::synthetic::synthetic_fbs_trunk;
use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01_with_q31, FbsExportSnaps, FbsTrunkSnapshot, IntegerStackModel,
};
use uor_r4_native_vector::{q31_hamilton_rotate, Q31HamiltonModel, Q31Quaternion};

const SEED: u64 = 20_261_115;
const TOKENS: &[u32] = &[3, 5, 8, 13, 21, 34, 55, 89];
const ONE: i64 = 1 << 31;
const WIDTH: usize = 32;
const LANES: usize = WIDTH / 4;
const TRAIN_STEPS: usize = 64;

fn initial_state() -> [u8; 48] {
    Q31HamiltonModel::new([ONE, 0, 0, 0], 8)
        .expect("initial gate")
        .to_bytes()
}

fn artifact(snap: &FbsTrunkSnapshot, state: &[u8; 48]) -> Vec<u8> {
    export_fbs_trunk_uorlut01_with_q31(snap, FbsExportSnaps::SNAPPED, state)
        .expect("export the /12 artifact")
}

/// Per transition, the `(lane, before)` pairs the real recurrence recorded.
fn carries(artifact_bytes: &[u8]) -> Vec<Vec<(usize, Q31Quaternion)>> {
    let model = IntegerStackModel::parse(artifact_bytes).expect("parse");
    let mut session = model.session();
    session
        .set_native_q31_trace(true)
        .expect("trace the attached state");
    let mut steps = Vec::with_capacity(TOKENS.len());
    for &token in TOKENS {
        session.step_native(token).expect("step the token");
        let trace = session.take_native_q31_trace().expect("trace enabled");
        steps.push(trace.into_iter().map(|carry| (carry.lane, carry.before)).collect());
    }
    steps
}

/// The next token's writer block for `lane`, quantized as the exporter does (Q12), then mapped
/// into the Q31 domain: the writer's i16 range is ±8, and the 1/8 factor takes it to ±1.
fn target(snap: &FbsTrunkSnapshot, lane: usize, next: u32) -> Q31Quaternion {
    let mut block = [0i64; 4];
    for (j, word) in block.iter_mut().enumerate() {
        let value = f64::from(snap.embed[next as usize * WIDTH + lane * 4 + j]);
        let q12 = (value * 4096.0).round() as i64;
        *word = q12 << 16;
    }
    block
}

/// The next token's writer block for `lane`, normalized per block to unit Q31 scale: the largest
/// coordinate is ±2^31 and the direction is preserved.
fn normalized_target(snap: &FbsTrunkSnapshot, lane: usize, next: u32) -> Q31Quaternion {
    let mut values = [0i128; 4];
    let mut max = 0i128;
    for (j, value) in values.iter_mut().enumerate() {
        let v = f64::from(snap.embed[next as usize * WIDTH + lane * 4 + j]);
        let q12 = (v * 4096.0).round() as i128;
        *value = q12;
        max = max.max(q12.abs());
    }
    if max == 0 {
        return [0; 4];
    }
    let mut target = [0i64; 4];
    for (j, value) in values.iter().enumerate() {
        target[j] = ((*value << 31) / max) as i64;
    }
    target
}

fn normalized_records(
    snap: &FbsTrunkSnapshot,
    carries: &[Vec<(usize, Q31Quaternion)>],
) -> (Vec<Q31Quaternion>, Vec<Q31Quaternion>) {
    let mut values = Vec::new();
    let mut targets = Vec::new();
    for (i, step) in carries.iter().enumerate().take(TOKENS.len() - 1) {
        let next = TOKENS[i + 1];
        for &(lane, before) in step {
            values.push(before);
            targets.push(normalized_target(snap, lane, next));
        }
    }
    (values, targets)
}

fn records(
    snap: &FbsTrunkSnapshot,
    carries: &[Vec<(usize, Q31Quaternion)>],
) -> (Vec<Q31Quaternion>, Vec<Q31Quaternion>) {
    let mut values = Vec::new();
    let mut targets = Vec::new();
    for (i, step) in carries.iter().enumerate().take(TOKENS.len() - 1) {
        let next = TOKENS[i + 1];
        for &(lane, before) in step {
            values.push(before);
            targets.push(target(snap, lane, next));
        }
    }
    (values, targets)
}

fn alignment(gate: Q31Quaternion, values: &[Q31Quaternion], targets: &[Q31Quaternion]) -> i128 {
    values
        .iter()
        .zip(targets)
        .map(|(value, target)| {
            let transport = q31_hamilton_rotate(gate, *value).expect("Q31 transport");
            (0..4)
                .map(|j| i128::from(transport[j]) * i128::from(target[j]))
                .sum::<i128>()
        })
        .sum()
}

fn served(artifact_bytes: &[u8]) -> Vec<u32> {
    let model = IntegerStackModel::parse(artifact_bytes).expect("parse");
    let mut session = model.session();
    TOKENS
        .iter()
        .map(|&token| session.step_native(token).expect("step").token)
        .collect()
}

#[test]
fn token_writer_targets_train_the_one_q31_gate() {
    let snap = synthetic_fbs_trunk(SEED);
    let start = initial_state();
    let start_artifact = artifact(&snap, &start);
    let start_carries = carries(&start_artifact);
    let (values, targets) = records(&snap, &start_carries);
    assert_eq!(
        values.len(),
        (TOKENS.len() - 1) * 2 * LANES,
        "two recurrence layers, {LANES} lanes each, one transition less than the sequence"
    );

    let gate0 = Q31HamiltonModel::from_bytes(start)
        .expect("initial state")
        .gate();
    let dot0 = alignment(gate0, &values, &targets);

    let mut model = Q31HamiltonModel::new([ONE, 0, 0, 0], 8).expect("training model");
    for _ in 0..TRAIN_STEPS {
        model
            .step_reference(&values, &targets)
            .expect("train on the token records");
    }
    let trained = model.to_bytes();
    assert_ne!(trained, start, "training did not change the 48-byte state");
    let dot1 = alignment(model.gate(), &values, &targets);
    eprintln!(
        "token objective: alignment {dot0} -> {dot1} over {} records in {TRAIN_STEPS} steps",
        values.len()
    );

    let trained_artifact = artifact(&snap, &trained);
    let parsed = IntegerStackModel::parse(&trained_artifact).expect("parse the trained artifact");
    assert_eq!(parsed.native_q31_state_bytes(), Some(trained));
    let before_tokens = served(&start_artifact);
    let after_tokens = served(&trained_artifact);
    eprintln!("served tokens: {before_tokens:?} -> {after_tokens:?}");
    // The objective is measured, not asserted positive: this first configuration is a negative
    // result (the alignment fell), which closes the configuration, not the mechanism.
    assert_ne!(
        dot0, dot1,
        "training did not move the measured alignment at all"
    );
}

#[test]
fn normalized_writer_targets_train_the_one_q31_gate() {
    let snap = synthetic_fbs_trunk(SEED);
    let start = initial_state();
    let start_artifact = artifact(&snap, &start);
    let start_carries = carries(&start_artifact);
    let (values, targets) = normalized_records(&snap, &start_carries);
    assert_eq!(values.len(), (TOKENS.len() - 1) * 2 * LANES);

    let gate0 = Q31HamiltonModel::from_bytes(start)
        .expect("initial state")
        .gate();
    let dot0 = alignment(gate0, &values, &targets);

    let mut model = Q31HamiltonModel::new([ONE, 0, 0, 0], 8).expect("training model");
    for _ in 0..TRAIN_STEPS {
        model
            .step_reference(&values, &targets)
            .expect("train on the normalized token records");
    }
    let trained = model.to_bytes();
    assert_ne!(trained, start, "training did not change the 48-byte state");
    let dot1 = alignment(model.gate(), &values, &targets);
    eprintln!(
        "normalized token objective: alignment {dot0} -> {dot1} over {} records in {TRAIN_STEPS} steps",
        values.len()
    );

    let trained_artifact = artifact(&snap, &trained);
    let parsed = IntegerStackModel::parse(&trained_artifact).expect("parse the trained artifact");
    assert_eq!(parsed.native_q31_state_bytes(), Some(trained));
    let before_tokens = served(&start_artifact);
    let after_tokens = served(&trained_artifact);
    eprintln!("normalized served tokens: {before_tokens:?} -> {after_tokens:?}");
    // The sign of the delta is the result; the test asserts only that the quantity moved.
    assert_ne!(dot0, dot1, "training did not move the measured alignment");
}
