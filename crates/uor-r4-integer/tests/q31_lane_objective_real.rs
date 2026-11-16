//! The lane objective on a real trained artifact's states, not the synthetic trunk.
//!
//! The artifact is the committed `/7` FBS trunk from the 2026-11-04 CPU content-serving pilot
//! (width 256, six recurrence layers, real tables, turn memory). The c387 states and the D41 pilot
//! are not local to this checkout, so this is the next-best real source: real weights and real
//! recurrence carries. The token ids are a fixed deterministic sequence because no tokenizer is
//! bound to the `/7` artifact — artificial inputs, real model.
//!
//! `cargo test -p uor-r4-integer --features serving-wasm --test q31_lane_objective_real -- --nocapture`

#![cfg(feature = "serving-wasm")]

use uor_r4_integer::stack::IntegerStackModel;
use uor_r4_native_vector::{q31_hamilton_rotate, Q31HamiltonModel, Q31LaneGates, Q31Quaternion};

const ARTIFACT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/labs/cpu-training-pilot-content-serving-2026-11-04/evidence/seed-1/serving.uorlut"
);
const IDS: usize = 32;
const TRAIN_STEPS: usize = 64;
const ONE: i64 = 1 << 31;
const LANES: usize = 64;

fn artifact() -> Vec<u8> {
    std::fs::read(ARTIFACT).expect("the committed /7 artifact")
}

/// A fixed, deterministic id sequence in the artifact's 4,096-entry vocabulary.
fn ids() -> Vec<u32> {
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    (0..IDS)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 33) as u32) % 4096
        })
        .collect()
}

fn identity_lanes() -> Vec<u8> {
    Q31LaneGates::new(vec![[ONE, 0, 0, 0]; LANES], 8)
        .expect("identity lanes")
        .to_bytes()
}

type Traced = (Vec<Vec<(usize, Q31Quaternion)>>, Vec<Vec<i32>>, Vec<u32>);

/// Trace the carries and the per-step writer queries of the real model on the fixed ids.
fn trace(artifact: &[u8], lanes: &[u8]) -> Traced {
    let mut model = IntegerStackModel::parse(artifact).expect("parse the real artifact");
    model
        .attach_native_q31_lane_state(lanes)
        .expect("attach the identity lanes");
    let mut session = model.session();
    session
        .set_native_q31_trace(true)
        .expect("trace the attached lanes");
    let mut steps = Vec::with_capacity(IDS);
    let mut queries = Vec::with_capacity(IDS);
    let mut tokens = Vec::with_capacity(IDS);
    for &id in &ids() {
        let step = session.step_native(id).expect("served step");
        tokens.push(step.token);
        let carries = session.take_native_q31_trace().expect("trace enabled");
        steps.push(
            carries
                .into_iter()
                .map(|carry| (carry.lane, carry.before))
                .collect::<Vec<_>>(),
        );
        queries.push(session.last_query_q12().expect("an FBS query").to_vec());
    }
    (steps, queries, tokens)
}

/// Per lane, every `(value, target)` pair across the six layers and the transitions.
fn records(
    steps: &[Vec<(usize, Q31Quaternion)>],
    queries: &[Vec<i32>],
) -> Vec<(Vec<Q31Quaternion>, Vec<Q31Quaternion>)> {
    let mut per_lane = vec![(Vec::new(), Vec::new()); LANES];
    for t in 0..steps.len().saturating_sub(1) {
        let next = &queries[t + 1];
        for &(lane, before) in &steps[t] {
            let mut target = [0i64; 4];
            for j in 0..4 {
                // Q12 -> Q26. The artifact's queries reach 2^15, so a 16-bit lift saturates.
                target[j] = i64::from(next[lane * 4 + j]) << 14;
            }
            per_lane[lane].0.push(before);
            per_lane[lane].1.push(target);
        }
    }
    per_lane
}

fn l1(gate: Q31Quaternion, values: &[Q31Quaternion], targets: &[Q31Quaternion]) -> i128 {
    values
        .iter()
        .zip(targets)
        .map(|(value, target)| {
            let transport = q31_hamilton_rotate(gate, *value).expect("transport");
            (0..4)
                .map(|j| (i128::from(transport[j]) - i128::from(target[j])).abs())
                .sum::<i128>()
        })
        .sum()
}

fn served(artifact: &[u8], state: Option<[u8; 48]>, lanes: Option<&[u8]>) -> Vec<u32> {
    let mut model = IntegerStackModel::parse(artifact).expect("parse");
    match (state, lanes) {
        (Some(state), None) => model.attach_native_q31_state(&state).expect("single"),
        (None, Some(lanes)) => model.attach_native_q31_lane_state(lanes).expect("lanes"),
        _ => panic!("one state"),
    }
    let mut session = model.session();
    ids()
        .iter()
        .map(|&id| session.step_native(id).expect("step").token)
        .collect()
}

#[test]
fn the_lane_objective_on_the_real_seven_artifact() {
    let artifact = artifact();
    let lanes = identity_lanes();
    let (steps, queries, tokens) = trace(&artifact, &lanes);
    assert_eq!(steps.len(), IDS);
    assert_eq!(steps[0].len(), 6 * LANES, "six layers, sixty-four lanes");
    assert_eq!(queries[0].len(), LANES * 4);
    let query_max = queries
        .iter()
        .flatten()
        .map(|value| value.unsigned_abs())
        .max()
        .unwrap_or(0);
    eprintln!("real artifact: first served tokens {tokens:?}; max |query_q12| {query_max}");

    let per_lane = records(&steps, &queries);
    assert!(per_lane
        .iter()
        .all(|(v, t)| v.len() == t.len() && !v.is_empty()));

    // Shared gate over every lane's records.
    let mut pooled_values = Vec::new();
    let mut pooled_targets = Vec::new();
    for (values, targets) in &per_lane {
        pooled_values.extend_from_slice(values);
        pooled_targets.extend_from_slice(targets);
    }
    let pooled_before = l1([ONE, 0, 0, 0], &pooled_values, &pooled_targets);
    let mut shared = Q31HamiltonModel::new([ONE, 0, 0, 0], 8).expect("shared model");
    for _ in 0..TRAIN_STEPS {
        shared
            .step_reference(&pooled_values, &pooled_targets)
            .expect("train the shared gate");
    }
    let pooled_after = l1(shared.gate(), &pooled_values, &pooled_targets);

    // Per-lane gates on the same records.
    let mut gates = Vec::with_capacity(LANES);
    let mut lane_before = Vec::with_capacity(LANES);
    let mut lane_after = Vec::with_capacity(LANES);
    for (values, targets) in &per_lane {
        lane_before.push(l1([ONE, 0, 0, 0], values, targets));
        let mut model = Q31HamiltonModel::new([ONE, 0, 0, 0], 8).expect("lane model");
        for _ in 0..TRAIN_STEPS {
            model
                .step_reference(values, targets)
                .expect("train the lane");
        }
        lane_after.push(l1(model.gate(), values, targets));
        gates.push(model.gate());
    }

    let before_tokens = served(&artifact, None, Some(&lanes));
    let shared_tokens = served(&artifact, Some(shared.to_bytes()), None);
    let mut lane_state = Q31LaneGates::new(gates, 8).expect("trained lanes");
    lane_state
        .set_cursor(TRAIN_STEPS as u64, 8)
        .expect("cursor");
    let lane_tokens = served(&artifact, None, Some(&lane_state.to_bytes()));

    eprintln!(
        "shared: L1 {pooled_before} -> {pooled_after}; per-lane: L1 {} -> {}",
        lane_before.iter().sum::<i128>(),
        lane_after.iter().sum::<i128>()
    );
    eprintln!(
        "first lanes (before, after): {:?}",
        lane_before
            .iter()
            .zip(&lane_after)
            .take(5)
            .collect::<Vec<_>>()
    );
    eprintln!(
        "lanes improved: {} of {LANES}",
        lane_before
            .iter()
            .zip(&lane_after)
            .filter(|(before, after)| **after < **before)
            .count()
    );
    eprintln!("tokens: identity {before_tokens:?}");
    eprintln!("tokens: shared   {shared_tokens:?}");
    eprintln!("tokens: per-lane {lane_tokens:?}");

    assert_ne!(
        shared.gate(),
        [ONE, 0, 0, 0],
        "the shared gate did not train"
    );
    assert!(
        lane_after
            .iter()
            .zip(&lane_before)
            .any(|(after, before)| after != before),
        "no lane's alignment moved"
    );
    assert_eq!(before_tokens.len(), IDS);
    assert_eq!(shared_tokens.len(), IDS);
    assert_eq!(lane_tokens.len(), IDS);
}
