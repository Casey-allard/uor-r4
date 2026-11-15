//! The per-lane finite token objective in the one native Q31 space, with a token-level
//! co-primary measure.
//!
//! The fixture is the one the two shared-gate negatives used: the synthetic trunk at seed
//! `20_261_115` and the token sequence `[3, 5, 8, 13, 21, 34, 55, 89]`. At each transition the
//! start state's traced lane carries are the values and the next token's writer block for that
//! lane, quantized at Q12 and shifted to Q31, is the target. Each lane trains its own gate with
//! the shipped `Q31HamiltonModel::step_reference`; the K gates are assembled into `Q31LaneGates`.
//!
//! Two quantities are reported. The primary is the alignment dot (the quantity the shared-gate
//! runs reported). The co-primary is token level and independent of it: the rank of the next
//! training token's writer row under the served `query_q12`, and the served token ids. A positive
//! alignment alone is never a serving win.
//!
//! Host-only; no language, capability or quality claim.
//!
//! `cargo test -p uor-r4-integer --features serving-wasm,fbs-synthetic --test q31_lane_objective`

#![cfg(all(feature = "serving-wasm", feature = "fbs-synthetic"))]

use uor_r4_integer::stack::synthetic::synthetic_fbs_trunk;
use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01_with_lane_gates, export_fbs_trunk_uorlut01_with_q31, FbsExportSnaps,
    FbsTrunkSnapshot, IntegerStackModel,
};
use uor_r4_native_vector::{q31_hamilton_rotate, Q31HamiltonModel, Q31LaneGates, Q31Quaternion};

const SEED: u64 = 20_261_115;
const TOKENS: &[u32] = &[3, 5, 8, 13, 21, 34, 55, 89];
const ONE: i64 = 1 << 31;
const WIDTH: usize = 32;
const LANES: usize = WIDTH / 4;
const TRAIN_STEPS: usize = 64;

fn initial_state() -> Vec<u8> {
    let mut state = Q31LaneGates::new(vec![[ONE, 0, 0, 0]; LANES], 8).expect("initial lanes");
    state.set_cursor(0, 8).expect("initial cursor");
    state.to_bytes()
}

fn artifact(snap: &FbsTrunkSnapshot, state: &[u8]) -> Vec<u8> {
    export_fbs_trunk_uorlut01_with_lane_gates(snap, FbsExportSnaps::SNAPPED, state)
        .expect("export the /13 artifact")
}

fn single_artifact(snap: &FbsTrunkSnapshot, state: &[u8; 48]) -> Vec<u8> {
    export_fbs_trunk_uorlut01_with_q31(snap, FbsExportSnaps::SNAPPED, state)
        .expect("export the /12 artifact")
}

/// Per transition, the `(lane, before)` pairs the real recurrence recorded.
fn carries(artifact_bytes: &[u8]) -> Vec<Vec<(usize, Q31Quaternion)>> {
    let model = IntegerStackModel::parse(artifact_bytes).expect("parse");
    let mut session = model.session();
    session
        .set_native_q31_trace(true)
        .expect("trace the attached lanes");
    let mut steps = Vec::with_capacity(TOKENS.len());
    for &token in TOKENS {
        session.step_native(token).expect("step the token");
        let trace = session.take_native_q31_trace().expect("trace enabled");
        steps.push(
            trace
                .into_iter()
                .map(|carry| (carry.lane, carry.before))
                .collect(),
        );
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

/// Per lane, every recorded `(value, target)` pair across both layers and the steps.
fn records_by_lane(
    snap: &FbsTrunkSnapshot,
    carries: &[Vec<(usize, Q31Quaternion)>],
) -> Vec<(Vec<Q31Quaternion>, Vec<Q31Quaternion>)> {
    let mut per_lane = vec![(Vec::new(), Vec::new()); LANES];
    for (i, step) in carries.iter().enumerate().take(TOKENS.len() - 1) {
        let next = TOKENS[i + 1];
        for &(lane, before) in step {
            per_lane[lane].0.push(before);
            per_lane[lane].1.push(target(snap, lane, next));
        }
    }
    per_lane
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

/// The shipped update's own quantity: the L1 distance from the transported value to the target,
/// summed over the records.
fn l1(gate: Q31Quaternion, values: &[Q31Quaternion], targets: &[Q31Quaternion]) -> i128 {
    values
        .iter()
        .zip(targets)
        .map(|(value, target)| {
            let transport = q31_hamilton_rotate(gate, *value).expect("Q31 transport");
            (0..4)
                .map(|j| (i128::from(transport[j]) - i128::from(target[j])).abs())
                .sum::<i128>()
        })
        .sum()
}

/// The served token ids and the rank of the next training token's writer row under the served
/// query at each transition. Rank 0 is best; ties go to the smaller token id.
fn served_path(
    artifact_bytes: &[u8],
    snap: &FbsTrunkSnapshot,
) -> (Vec<u32>, Vec<usize>, Vec<Vec<i32>>) {
    let model = IntegerStackModel::parse(artifact_bytes).expect("parse");
    let mut session = model.session();
    let mut tokens = Vec::with_capacity(TOKENS.len());
    let mut ranks = Vec::with_capacity(TOKENS.len().saturating_sub(1));
    let mut queries = Vec::with_capacity(TOKENS.len());
    for (i, &token) in TOKENS.iter().enumerate() {
        tokens.push(session.step_native(token).expect("step").token);
        let query = session.last_query_q12().expect("an FBS query").to_vec();
        if let Some(&next) = TOKENS.get(i + 1) {
            ranks.push(rank_of(snap, &query, next));
        }
        queries.push(query);
    }
    (tokens, ranks, queries)
}

/// The L1 rank of `token`'s writer row under `query`, both at Q12.
fn rank_of(snap: &FbsTrunkSnapshot, query: &[i32], token: u32) -> usize {
    let mut distances: Vec<(i128, u32)> = (0..snap.vocab)
        .map(|candidate| {
            let distance = (0..WIDTH)
                .map(|j| {
                    let value = f64::from(snap.embed[candidate * WIDTH + j]);
                    let row = (value * 4096.0).round() as i32;
                    (i128::from(query[j]) - i128::from(row)).abs()
                })
                .sum::<i128>();
            (distance, candidate as u32)
        })
        .collect();
    distances.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    distances
        .iter()
        .position(|(_, candidate)| *candidate == token)
        .expect("the target token is one of the rows")
}

#[test]
fn lane_writer_targets_train_each_lane_gate_and_report_a_token_rank() {
    let snap = synthetic_fbs_trunk(SEED);
    let start = initial_state();
    let start_artifact = artifact(&snap, &start);
    let start_carries = carries(&start_artifact);
    let per_lane = records_by_lane(&snap, &start_carries);
    assert_eq!(
        per_lane[0].0.len(),
        (TOKENS.len() - 1) * 2,
        "one record per transition per recurrence layer"
    );
    assert!(per_lane
        .iter()
        .all(|(v, t)| v.len() == t.len() && !v.is_empty()));

    // Train one gate per lane with the shipped single-gate update.
    let mut trained_gates = Vec::with_capacity(LANES);
    let mut lane_before = Vec::with_capacity(LANES);
    let mut lane_after = Vec::with_capacity(LANES);
    for (lane, (values, targets)) in per_lane.iter().enumerate() {
        let before = alignment([ONE, 0, 0, 0], values, targets);
        let mut model = Q31HamiltonModel::new([ONE, 0, 0, 0], 8).expect("training model");
        for _ in 0..TRAIN_STEPS {
            model
                .step_reference(values, targets)
                .expect("train the lane on its token records");
        }
        let after = alignment(model.gate(), values, targets);
        lane_before.push(before);
        lane_after.push(after);
        trained_gates.push(model.gate());
        assert_ne!(
            model.gate(),
            [ONE, 0, 0, 0],
            "lane {lane} training did not change its gate"
        );
    }
    let total_before: i128 = lane_before.iter().sum();
    let total_after: i128 = lane_after.iter().sum();
    let improved = lane_before
        .iter()
        .zip(&lane_after)
        .filter(|(before, after)| after > before)
        .count();
    eprintln!("per-lane alignment {total_before} -> {total_after}");
    eprintln!("per-lane before: {lane_before:?}");
    eprintln!("per-lane after:  {lane_after:?}");
    eprintln!("lanes with a higher alignment: {improved} of {LANES}");

    // Assemble and serve the trained state through the /13 artifact.
    let mut state = Q31LaneGates::new(trained_gates.clone(), 8).expect("trained lanes");
    state
        .set_cursor(TRAIN_STEPS as u64, 8)
        .expect("shared cursor");
    let trained_bytes = state.to_bytes();
    let trained_artifact = artifact(&snap, &trained_bytes);
    let parsed = IntegerStackModel::parse(&trained_artifact).expect("parse the trained artifact");
    assert_eq!(
        parsed.native_q31_lane_state_bytes(),
        Some(trained_bytes.clone())
    );
    let attached = parsed
        .native_q31_lane_state()
        .expect("the trained lanes are attached");
    for (lane, gate) in trained_gates.iter().enumerate() {
        assert_eq!(
            &attached.gate(lane).expect("lane gate"),
            gate,
            "lane {lane}"
        );
    }

    // Did the trained state reach the served path at all? The carries must move; the writer
    // query is the readout's input.
    let trained_carries = carries(&trained_artifact);
    let differing: usize = start_carries
        .iter()
        .zip(&trained_carries)
        .flat_map(|(before, after)| before.iter().zip(after))
        .filter(|((_, a), (_, b))| a != b)
        .count();
    let total: usize = start_carries.iter().map(Vec::len).sum();
    let (start_tokens, start_ranks, start_queries) = served_path(&start_artifact, &snap);
    let (trained_tokens, trained_ranks, trained_queries) = served_path(&trained_artifact, &snap);
    eprintln!("differing lane carries on the served path: {differing} of {total}");
    eprintln!(
        "writer queries identical: {}",
        start_queries == trained_queries
    );
    let mean = |ranks: &[usize]| ranks.iter().sum::<usize>() as f64 / ranks.len() as f64;
    eprintln!("served tokens {start_tokens:?} -> {trained_tokens:?}");
    eprintln!("next-token rank {start_ranks:?} -> {trained_ranks:?}");
    eprintln!(
        "mean next-token rank {:.2} -> {:.2}",
        mean(&start_ranks),
        mean(&trained_ranks)
    );
    eprintln!(
        "next-token rank improved at {} of {} transitions",
        start_ranks
            .iter()
            .zip(&trained_ranks)
            .filter(|(before, after)| after < before)
            .count(),
        start_ranks.len()
    );

    // The sign is the result: the test asserts the quantities were computed and that training
    // moved the state, never that the configuration is positive.
    assert_ne!(trained_bytes, start, "training did not change the state");
}

#[test]
fn training_one_lane_changes_only_that_lane_gate() {
    let snap = synthetic_fbs_trunk(SEED);
    let start = initial_state();
    let per_lane = records_by_lane(&snap, &carries(&artifact(&snap, &start)));

    let mut gates = vec![[ONE, 0, 0, 0]; LANES];
    let mut model = Q31HamiltonModel::new([ONE, 0, 0, 0], 8).expect("training model");
    for _ in 0..TRAIN_STEPS {
        model
            .step_reference(&per_lane[0].0, &per_lane[0].1)
            .expect("train lane 0");
    }
    gates[0] = model.gate();
    let state = Q31LaneGates::new(gates, 8)
        .expect("one trained lane")
        .to_bytes();

    let parsed =
        IntegerStackModel::parse(&artifact(&snap, &state)).expect("parse the one-lane artifact");
    let attached = parsed.native_q31_lane_state().expect("attached");
    assert_ne!(attached.gate(0).expect("lane 0"), [ONE, 0, 0, 0]);
    for lane in 1..LANES {
        assert_eq!(
            attached.gate(lane).expect("untouched lane"),
            [ONE, 0, 0, 0],
            "lane {lane} changed with lane 0's training"
        );
    }
}

#[test]
fn a_quarter_turn_state_reaches_the_carry_and_the_writer_query() {
    let snap = synthetic_fbs_trunk(SEED);
    let identity = artifact(
        &snap,
        &Q31LaneGates::new(vec![[ONE, 0, 0, 0]; LANES], 8)
            .expect("identity lanes")
            .to_bytes(),
    );
    let mut turned = Q31LaneGates::new(vec![[0, ONE, 0, 0]; LANES], 8).expect("quarter-turn lanes");
    turned.set_cursor(0, 8).expect("cursor");
    let turned = artifact(&snap, &turned.to_bytes());

    let (identity_tokens, identity_ranks, identity_queries) = served_path(&identity, &snap);
    let (turned_tokens, turned_ranks, turned_queries) = served_path(&turned, &snap);
    let differing_words: usize = identity_queries
        .iter()
        .zip(&turned_queries)
        .map(|(a, b)| a.iter().zip(b).filter(|(x, y)| x != y).count())
        .sum();
    eprintln!("quarter turn: tokens {identity_tokens:?} -> {turned_tokens:?}");
    eprintln!("quarter turn: ranks {identity_ranks:?} -> {turned_ranks:?}");
    eprintln!("quarter turn: differing query words {differing_words}");
    assert!(
        differing_words > 0,
        "the trained state never reached the writer query"
    );

    // Isolate the carry at the second step: the first step's carry is zero, and every gate
    // rotates zero to zero. A different gate must change both its transport and its carry.
    let carries_at_second_step = |bytes: &[u8]| {
        let model = IntegerStackModel::parse(bytes).expect("parse");
        let mut session = model.session();
        session.set_native_q31_trace(true).expect("trace");
        session.step_native(TOKENS[0]).expect("warm step");
        session.take_native_q31_trace().expect("warm trace");
        session.step_native(TOKENS[1]).expect("traced step");
        session.take_native_q31_trace().expect("traced trace")
    };
    let identity_carries = carries_at_second_step(&identity);
    let turned_carries = carries_at_second_step(&turned);
    let rejected = identity_carries
        .iter()
        .zip(&turned_carries)
        .filter(|(a, b)| a.transport != b.transport && a.after == b.after)
        .count();
    eprintln!("carries whose transport changed but whose carry did not: {rejected}");
    assert_eq!(
        rejected, 0,
        "a lane's transport was discarded from its carry"
    );
    assert!(
        identity_carries
            .iter()
            .zip(&turned_carries)
            .any(|(a, b)| a.before == b.before && a.transport != b.transport && a.after != b.after),
        "no lane carried its quarter-turned transport"
    );
}

#[test]
fn the_shared_gate_and_the_per_lane_gates_are_measured_on_one_fixture() {
    let snap = synthetic_fbs_trunk(SEED);
    let start = initial_state();
    let start_artifact = artifact(&snap, &start);
    let per_lane = records_by_lane(&snap, &carries(&start_artifact));
    let (start_tokens, start_ranks, _) = served_path(&start_artifact, &snap);

    // The shared-gate configuration of codex-6/7: every lane's records pooled into one gate.
    let mut pooled_values = Vec::new();
    let mut pooled_targets = Vec::new();
    for (values, targets) in &per_lane {
        pooled_values.extend_from_slice(values);
        pooled_targets.extend_from_slice(targets);
    }
    let pooled_before = alignment([ONE, 0, 0, 0], &pooled_values, &pooled_targets);
    let pooled_l1_before = l1([ONE, 0, 0, 0], &pooled_values, &pooled_targets);
    let mut shared = Q31HamiltonModel::new([ONE, 0, 0, 0], 8).expect("shared training model");
    for _ in 0..TRAIN_STEPS {
        shared
            .step_reference(&pooled_values, &pooled_targets)
            .expect("train the shared gate");
    }
    let pooled_after = alignment(shared.gate(), &pooled_values, &pooled_targets);
    let pooled_l1_after = l1(shared.gate(), &pooled_values, &pooled_targets);
    let shared_per_lane: Vec<i128> = per_lane
        .iter()
        .map(|(values, targets)| alignment(shared.gate(), values, targets))
        .collect();
    let shared_bytes = shared.to_bytes();
    let shared_artifact = single_artifact(&snap, &shared_bytes);

    // The per-lane configuration of codex-12 A.
    let mut lane_gates = Vec::with_capacity(LANES);
    let mut lane_before = Vec::with_capacity(LANES);
    let mut lane_after = Vec::with_capacity(LANES);
    let mut lane_l1_before = 0i128;
    let mut lane_l1_after = 0i128;
    for (values, targets) in &per_lane {
        lane_before.push(alignment([ONE, 0, 0, 0], values, targets));
        lane_l1_before += l1([ONE, 0, 0, 0], values, targets);
        let mut model = Q31HamiltonModel::new([ONE, 0, 0, 0], 8).expect("lane training model");
        for _ in 0..TRAIN_STEPS {
            model
                .step_reference(values, targets)
                .expect("train the lane");
        }
        lane_after.push(alignment(model.gate(), values, targets));
        lane_l1_after += l1(model.gate(), values, targets);
        assert_ne!(model.gate(), [ONE, 0, 0, 0], "a lane gate did not train");
        lane_gates.push(model.gate());
    }
    let mut lanes_state = Q31LaneGates::new(lane_gates, 8).expect("trained lanes");
    lanes_state
        .set_cursor(TRAIN_STEPS as u64, 8)
        .expect("shared cursor");
    let lanes_artifact = artifact(&snap, &lanes_state.to_bytes());

    let (shared_tokens, shared_ranks, _) = served_path(&shared_artifact, &snap);
    let (lane_tokens, lane_ranks, _) = served_path(&lanes_artifact, &snap);
    let mean = |ranks: &[usize]| ranks.iter().sum::<usize>() as f64 / ranks.len() as f64;
    let moved = |tokens: &[u32]| {
        tokens
            .iter()
            .zip(&start_tokens)
            .filter(|(a, b)| a != b)
            .count()
    };
    let improved = |ranks: &[usize]| {
        ranks
            .iter()
            .zip(&start_ranks)
            .filter(|(after, before)| after < before)
            .count()
    };

    eprintln!("shared gate: pooled L1 {pooled_l1_before} -> {pooled_l1_after}");
    eprintln!("per-lane gates: total L1 {lane_l1_before} -> {lane_l1_after}");
    eprintln!("shared gate: pooled alignment {pooled_before} -> {pooled_after}");
    eprintln!("shared gate: per-lane alignment {shared_per_lane:?}");
    eprintln!(
        "per-lane gates: total alignment {} -> {}",
        lane_before.iter().sum::<i128>(),
        lane_after.iter().sum::<i128>()
    );
    eprintln!("per-lane gates: per-lane after {lane_after:?}");
    eprintln!("untrained tokens {start_tokens:?}, ranks {start_ranks:?}");
    eprintln!(
        "shared gate: tokens {shared_tokens:?} ({} moved), mean rank {:.2} ({} of {} improved)",
        moved(&shared_tokens),
        mean(&shared_ranks),
        improved(&shared_ranks),
        start_ranks.len()
    );
    eprintln!(
        "per-lane gates: tokens {lane_tokens:?} ({} moved), mean rank {:.2} ({} of {} improved)",
        moved(&lane_tokens),
        mean(&lane_ranks),
        improved(&lane_ranks),
        start_ranks.len()
    );

    // Both configurations were trained and served; the signs are the result and go to the REPORT.
    assert_ne!(
        shared.gate(),
        [ONE, 0, 0, 0],
        "the shared gate did not train"
    );
    assert_ne!(
        pooled_before, pooled_after,
        "the shared alignment did not move"
    );
    assert_eq!(shared_tokens.len(), TOKENS.len());
    assert_eq!(lane_tokens.len(), TOKENS.len());
    assert_eq!(shared_ranks.len(), TOKENS.len() - 1);
    assert_eq!(lane_ranks.len(), TOKENS.len() - 1);
}
