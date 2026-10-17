//! Finite, exact CUDA construction controls for `ReadSpec::Relation` and Gate (kind 3).
//! Design: docs/mechanisms/finite-relational-head.md (#2486) and
//! pre-registration: docs/harness/prep/m1-finite-relational-head-2026-10-14.md (#2494).
//! Learned-attention activation/init amendment: §7.8 item 3 of
//! docs/harness/prep/m1-learned-attention-link-2026-10-17.md.
//! Zero parameter updates or corpus runs execute here. Every expectation is a
//! hand-derived integer literal over the generated anchor orbit:
//!   root 0  = A      = [ 2,  2,  0,  0,  0,  0,  0,  0]
//!   root 1  = NEG_A  = [-2, -2,  0,  0,  0,  0,  0,  0]
//!   root 2  = C_ROOT = [-2,  2,  0,  0,  0,  0,  0,  0]
//!   root 3  = NEG_B  = [ 0,  0, -2,  0, -2,  0,  0,  0]
//!   root 14 = B      = [ 0,  0,  2,  0,  2,  0,  0,  0]
//!   root 15 = D_ROOT = [ 2, -2,  0,  0,  0,  0,  0,  0]
//! No host recurrence or host reference forward exists (D24).
use crate::{
    device::{Device, Forward, Snapshot},
    Error, Identity, Parameter, ReadSpec, Result, Spec,
};
use std::path::Path;

const ABSENT: u32 = u32::MAX;
const MAX: u64 = u64::MAX;
const A: [i32; 8] = [2, 2, 0, 0, 0, 0, 0, 0];
const NEG_A: [i32; 8] = [-2, -2, 0, 0, 0, 0, 0, 0];
const C_ROOT: [i32; 8] = [-2, 2, 0, 0, 0, 0, 0, 0];
const NEG_B: [i32; 8] = [0, 0, -2, 0, -2, 0, 0, 0];
const B: [i32; 8] = [0, 0, 2, 0, 2, 0, 0, 0];
const D_ROOT: [i32; 8] = [2, -2, 0, 0, 0, 0, 0, 0];
const TRACE_ROW: usize = 4;
const TRACE_LEVEL: usize = 28;
const C2_TOKENS: [[[i32; 8]; 2]; 5] = [
    [B, A],         // 0: tok_x = [root 14, root 0], cell 224
    [A, D_ROOT],    // 1: tok_A = [root 0, root 15], cell 15
    [NEG_A, NEG_B], // 2: tok_B = [root 1, root 3], cell 19
    [C_ROOT, A],    // 3: tok_y = [root 2, root 0], cell 32
    [D_ROOT, A],    // 4: tok_z = [root 15, root 0], cell 240
];

fn require(ok: bool, why: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error::Invalid(why.into()))
    }
}
fn expect_invalid<T>(result: Result<T>, expected: &str, label: &str) -> Result<()> {
    match result {
        Err(Error::Invalid(actual)) if actual == expected => Ok(()),
        Err(error @ Error::Cuda(_)) => Err(error),
        Err(error) => Err(Error::Invalid(format!(
            "{label}: expected Invalid({expected:?}), observed {error}"
        ))),
        Ok(_) => Err(Error::Invalid(format!(
            "{label}: expected Invalid({expected:?}), operation succeeded"
        ))),
    }
}
fn identity(token: u32, occurrence: u64) -> Identity {
    Identity {
        token,
        occurrence,
        record: 17,
        version: 9,
    }
}
fn c2_prefix() -> [Identity; 6] {
    [
        identity(0, 0),
        identity(1, 1),
        identity(2, 2),
        identity(3, 3),
        identity(4, 4),
        identity(1, 5),
    ]
}
fn concat(blocks: &[[i32; 8]]) -> Vec<i32> {
    blocks.iter().flatten().copied().collect()
}
fn flock_snapshot(
    context: u32,
    kept_cells: u32,
    kept: u32,
    tokens: &[[[i32; 8]; 2]],
    gates: &[i32],
) -> Result<Snapshot> {
    let spec = Spec {
        vocabulary: tokens.len() as u32,
        blocks: 2,
        context,
        kept_cells,
    };
    let device = Device::new_with_read(spec, ReadSpec::Flock { kept }, 7, 0)?;
    let mut snapshot = device.snapshot()?;
    for (token, blocks) in tokens.iter().enumerate() {
        snapshot.table[token * 16..token * 16 + 16].copy_from_slice(&concat(blocks));
    }
    if !gates.is_empty() {
        require(
            gates.len() == snapshot.gates.len(),
            "flock fixture gate inventory differs",
        )?;
        snapshot.gates.copy_from_slice(gates);
    }
    Ok(snapshot)
}
fn relation_snapshot(
    context: u32,
    kept_cells: u32,
    kept: u32,
    sweeps: u32,
    tokens: &[[[i32; 8]; 2]],
    gates: &[i32],
) -> Result<Snapshot> {
    let spec = Spec {
        vocabulary: tokens.len() as u32,
        blocks: 2,
        context,
        kept_cells,
    };
    let device = Device::new_with_read(spec, ReadSpec::Relation { kept, sweeps }, 7, 0)?;
    let mut snapshot = device.snapshot()?;
    for (token, blocks) in tokens.iter().enumerate() {
        snapshot.table[token * 16..token * 16 + 16].copy_from_slice(&concat(blocks));
    }
    if !gates.is_empty() {
        require(
            gates.len() == snapshot.gates.len(),
            "relation fixture gate inventory differs",
        )?;
        snapshot.gates.copy_from_slice(gates);
    }
    Ok(snapshot)
}
fn trace(forward: &Forward) -> Result<Vec<u32>> {
    forward
        .flock_trace()?
        .ok_or_else(|| Error::Invalid("forward returned no kept-node trace".into()))
}
fn rel_scores(forward: &Forward) -> Result<Vec<i32>> {
    forward
        .relation_scores()?
        .ok_or_else(|| Error::Invalid("relation forward returned no relation scores".into()))
}
fn block_rel_scores(
    scores: &[i32],
    blocks: usize,
    nodes_count: usize,
    row: usize,
    block: usize,
) -> &[i32] {
    let base = (row * blocks + block) * nodes_count;
    &scores[base..base + nodes_count]
}
fn head(t: &[u32], blocks: usize, stages: usize, row: usize, block: usize) -> Vec<u32> {
    let base = (row * blocks + block) * (TRACE_ROW + stages * TRACE_LEVEL);
    t[base..base + TRACE_ROW].to_vec()
}
fn level(t: &[u32], blocks: usize, stages: usize, row: usize, block: usize, l: usize) -> Vec<u32> {
    let base =
        (row * blocks + block) * (TRACE_ROW + stages * TRACE_LEVEL) + TRACE_ROW + l * TRACE_LEVEL;
    t[base..base + TRACE_LEVEL].to_vec()
}
fn summary(lv: &[u32], k: usize) -> (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>) {
    (
        lv[0..4].to_vec(),
        lv[4..4 + k].to_vec(),
        lv[12..12 + k].to_vec(),
        lv[20..20 + k].to_vec(),
    )
}

fn verify_flock_vs_relation_parity(
    context: u32,
    kept_cells: u32,
    kept: u32,
    tokens: &[[[i32; 8]; 2]],
    combine_gates: &[i32],
    windows: &[Vec<Identity>],
    targets: &[u32],
) -> Result<()> {
    let flock_dev = Device::from_snapshot(
        flock_snapshot(context, kept_cells, kept, tokens, combine_gates)?,
        0,
    )?;
    let flock_in = flock_dev.carry_windows(windows)?;
    let flock_out = flock_dev.forward_traced(&flock_in, targets)?;
    let flock_tr = trace(&flock_out)?;
    let flock_vec = flock_out.vectors()?;

    for sweeps in [1u32, 2u32] {
        let mut gates = Vec::new();
        if !combine_gates.is_empty() {
            let stages = combine_gates.len() / 2;
            gates.extend_from_slice(combine_gates);
            gates.resize(combine_gates.len() + 5 * stages, 0);
        }
        let rel_dev = Device::from_snapshot(
            relation_snapshot(context, kept_cells, kept, sweeps, tokens, &gates)?,
            0,
        )?;
        let rel_in = rel_dev.carry_windows(windows)?;
        let rel_out = rel_dev.forward_traced(&rel_in, targets)?;
        let rel_tr = trace(&rel_out)?;
        let scores = rel_scores(&rel_out)?;
        require(
            rel_out.vectors()? == flock_vec
                && rel_out.losses == flock_out.losses
                && rel_out.predictions == flock_out.predictions
                && rel_out.token_rivals == flock_out.token_rivals
                && rel_out.address_rivals == flock_out.address_rivals
                && rel_out.admitted_cells == flock_out.admitted_cells
                && rel_tr == flock_tr,
            "relation-off forward output, loss, or trace differs from ReadSpec::Flock",
        )?;
        require(
            scores.iter().all(|&s| s == 0 || (s as u32) == ABSENT),
            "relation-off produced non-zero relation score",
        )?;
    }
    Ok(())
}

/// C1: `relation-off ≡ Flock` parity across all 6 Flock fixtures at sweeps = 1 and sweeps = 2.
fn relation_off_flock_parity() -> Result<()> {
    // 1. flock_identity_read
    verify_flock_vs_relation_parity(
        3,
        3,
        2,
        &[[A, A], [NEG_A, A], [A, B]],
        &[],
        &[vec![identity(0, 0), identity(1, 1), identity(2, 2)]],
        &[0],
    )?;
    // 2. flock_gate_counterfactual forward states (trits 0, -1, +1)
    for trit in [0, -1, 1] {
        verify_flock_vs_relation_parity(
            2,
            2,
            1,
            &[[A, B], [A, A]],
            &[0, trit],
            &[vec![identity(1, 0), identity(0, 1)]],
            &[1],
        )?;
    }
    // 3. flock_transport_witnesses (both models)
    verify_flock_vs_relation_parity(
        2,
        2,
        1,
        &[[A, B], [A, A]],
        &[0, 0],
        &[vec![identity(1, 0), identity(0, 1)]],
        &[1],
    )?;
    verify_flock_vs_relation_parity(
        2,
        2,
        1,
        &[[A, B], [A, B]],
        &[0, 0],
        &[vec![identity(1, 0), identity(0, 1)]],
        &[1],
    )?;
    // 4. flock_payload_witness (both context payloads)
    for context_token in [A, NEG_A] {
        verify_flock_vs_relation_parity(
            2,
            2,
            1,
            &[[A, B], [A, context_token]],
            &[0, 1],
            &[vec![identity(1, 0), identity(0, 1)]],
            &[1],
        )?;
    }
    // 5. flock_selection_and_eligibility (both row variants)
    for third in [1u32, 0u32] {
        verify_flock_vs_relation_parity(
            4,
            2,
            2,
            &[[A, A], [B, B]],
            &[],
            &[
                vec![
                    identity(0, 0),
                    identity(0, 1),
                    identity(third, 2),
                    identity(0, 3),
                ],
                vec![identity(0, 4), identity(0, 5), identity(0, 6)],
            ],
            &[0, 0],
        )?;
    }
    // 6. flock_refusals_and_noop lone-token state
    verify_flock_vs_relation_parity(
        1,
        2,
        1,
        &[[A, B], [A, A]],
        &[0, 0],
        &[vec![identity(0, 0)]],
        &[1],
    )
}

/// C2: Copy-successor fixture (`x A B y z A`) at context = 6 (stages = 2, 8 nodes).
fn relation_copy_successor_fixture() -> Result<()> {
    let tokens = C2_TOKENS;
    let prefix = c2_prefix();
    // Baseline: gates[2] = +1 (block 1, level 0 combine gate), gates[6] = 0 (Flock baseline).
    let mut base_gates = [0i32; 14];
    base_gates[2] = 1;
    let base_dev = Device::from_snapshot(relation_snapshot(6, 2, 1, 1, &tokens, &base_gates)?, 0)?;
    let base_in = base_dev.carry(&prefix)?;
    let base_out = base_dev.forward_traced(&base_in, &[2])?;
    let base_tr = trace(&base_out)?;
    require(
        head(&base_tr, 2, 2, 0, 1) == [0, 15, 2, 15]
            && summary(&level(&base_tr, 2, 2, 0, 1, 0), 1)
                == (vec![2, 1, 15, 5], vec![1], vec![0], vec![15])
            && base_out.vectors()? == concat(&[A, D_ROOT])
            && base_out.losses == [640],
        "C2 baseline (gates[6]=0) trace, output [A, D_ROOT], or loss 640 differs",
    )?;

    // Active relation: gates[2] = +1, gates[6] = +1 (SequentialSuccessor at level 0).
    let mut rel_gates = base_gates;
    rel_gates[6] = 1;
    let rel_dev = Device::from_snapshot(relation_snapshot(6, 2, 1, 1, &tokens, &rel_gates)?, 0)?;
    let rel_in = rel_dev.carry(&prefix)?;
    let rel_out = rel_dev.forward_traced(&rel_in, &[2])?;
    let rel_tr = trace(&rel_out)?;
    let scores = rel_scores(&rel_out)?;
    let b1_scores = block_rel_scores(&scores, 2, 8, 0, 1);
    require(
        [
            b1_scores[0],
            b1_scores[1],
            b1_scores[2],
            b1_scores[4],
            b1_scores[5],
        ] == [0, 0, 8, 0, 4],
        "C2 active relation scores on eligible level-0 nodes [0,1,2,4,5] differ from [0,0,8,0,4]",
    )?;
    require(
        head(&rel_tr, 2, 2, 0, 1) == [0, 14, 2, 15]
            && summary(&level(&rel_tr, 2, 2, 0, 1, 0), 1)
                == (vec![2, 1, 14, 5], vec![2], vec![16], vec![14])
            && rel_out.vectors()? == concat(&[A, B])
            && rel_out.losses == [448],
        "C2 active relation (gates[6]=+1) kept node 2, BFS[1][0]=16 transport to root 14, output [A, B], or loss 448 differs",
    )?;

    // Counterfactual propose(Parameter::Gate(6)) on baseline snapshot with target = 2 (tok_B).
    let before = base_dev.snapshot()?;
    let proposal = base_dev.propose(&base_in, &[2], Parameter::Gate(6))?;
    let losses = proposal.candidate_losses()?;
    let mut expected = vec![MAX; 16];
    expected[..3].copy_from_slice(&[640, 640, 448]);
    require(
        losses == expected,
        "C2 Gate(6) counterfactual losses differ from [640, 640, 448, MAX x 13]",
    )?;
    let observed = proposal.observation()?;
    require(
        observed.candidate == 2
            && observed.before == 640
            && observed.after == 448
            && observed.epoch == 0
            && observed.any_candidate_cost_differs_from_identity,
        "C2 Gate(6) chooser must select ordinal 2 (+1) with 640 -> 448",
    )?;
    let after = base_dev.snapshot()?;
    require(
        before.table == after.table
            && before.routes == after.routes
            && before.gates == after.gates
            && before.epoch == after.epoch,
        "C2 virtual Gate(6) proposal mutated model state",
    )?;
    for (trit, loss, out) in [
        (0, 640u64, [A, D_ROOT]),
        (-1, 640u64, [A, D_ROOT]),
        (1, 448u64, [A, B]),
    ] {
        let mut g = base_gates;
        g[6] = trit;
        let ind = Device::from_snapshot(relation_snapshot(6, 2, 1, 1, &tokens, &g)?, 0)?;
        let carried = ind.carry(&prefix)?;
        let res = ind.forward(&carried, &[2])?;
        require(
            res.losses == [loss] && res.vectors()? == concat(&out),
            "C2 independent snapshot forward differs from Gate(6) counterfactual lane",
        )?;
    }
    Ok(())
}

/// C3: Irrelevant-change invariance when mutating non-matching context atoms (nodes 0, 4, 5).
fn relation_irrelevant_change_invariance() -> Result<()> {
    let tokens = [
        [B, A],         // 0: frame 14 (m=0)
        [A, D_ROOT],    // 1: tok_A, frame 0 (m=8)
        [NEG_A, NEG_B], // 2: tok_B, frame 1 (m=0)
        [C_ROOT, A],    // 3: frame 2 (m=4)
        [D_ROOT, A],    // 4: frame 15 (m=4)
    ];
    let mut gates = [0i32; 14];
    gates[2] = 1; // block 1, level 0 combine gate
    gates[6] = 1; // level 0 SequentialSuccessor relation gate
                  // Both level-1 combine gates (gates[1] = 0, gates[3] = 0) are explicitly 0.
    let device = Device::from_snapshot(relation_snapshot(6, 2, 1, 1, &tokens, &gates)?, 0)?;
    for &x in &[0u32, 2, 3, 4] {
        for &y in &[0u32, 2, 3, 4] {
            for &z in &[0u32, 2, 3, 4] {
                let prefix = [
                    identity(x, 0),
                    identity(1, 1),
                    identity(2, 2),
                    identity(y, 3),
                    identity(z, 4),
                    identity(1, 5),
                ];
                let input = device.carry(&prefix)?;
                let result = device.forward_traced(&input, &[2])?;
                let t = trace(&result)?;
                let scores = rel_scores(&result)?;
                let b1 = block_rel_scores(&scores, 2, 8, 0, 1);
                require(
                    b1[2] == 8
                        && b1[2] > b1[5]
                        && summary(&level(&t, 2, 2, 0, 1, 0), 1)
                            == (vec![2, 1, 14, 5], vec![2], vec![16], vec![14])
                        && result.vectors()? == concat(&[A, B]),
                    "C3 irrelevant non-matching context mutation altered node 2 selection or output [A, B]",
                )?;
            }
        }
    }
    Ok(())
}

/// C4: Conflicting successors (`A B x A C A`): 4a equal `(d_L1, d_Hamming)` -> `[5, 1]`;
/// 4b unequal `d_L1` -> `[1, 5]`.
fn relation_conflicting_successors() -> Result<()> {
    let mut gates = [0i32; 14];
    gates[6] = 1;
    let prefix = [
        identity(1, 0), // node 0: A
        identity(2, 1), // node 1: B
        identity(0, 2), // node 2: x
        identity(1, 3), // node 4: A
        identity(3, 4), // node 5: C
        identity(1, 5), // node 6: A (query)
    ];
    // Subcase 4a: tok_B = [C_ROOT, A] (d_L1=4, d_H=1) and tok_C = [D_ROOT, A] (d_L1=4, d_H=1).
    let tokens_4a = [[B, A], [A, A], [C_ROOT, A], [D_ROOT, A]];
    let dev_4a = Device::from_snapshot(relation_snapshot(6, 2, 2, 1, &tokens_4a, &gates)?, 0)?;
    let out_4a = dev_4a.forward_traced(&dev_4a.carry(&prefix)?, &[2])?;
    let tr_4a = trace(&out_4a)?;
    let sc_4a = rel_scores(&out_4a)?;
    let b1_4a = block_rel_scores(&sc_4a, 2, 8, 0, 1);
    require(
        [b1_4a[0], b1_4a[1], b1_4a[2], b1_4a[4], b1_4a[5]] == [0, 8, 4, 0, 8]
            && summary(&level(&tr_4a, 2, 2, 0, 1, 0), 2).1 == [5, 1],
        "C4a equal (d_L1, d_Hamming) conflicting successors must rank top[0..2] = [5, 1] by recency",
    )?;

    // Subcase 4b: tok_B = [C_ROOT, A] (d_L1=4, d_H=1) and tok_C = [B, A] (d_L1=8, d_H=4).
    let tokens_4b = [[B, A], [A, A], [C_ROOT, A], [B, A]];
    let dev_4b = Device::from_snapshot(relation_snapshot(6, 2, 2, 1, &tokens_4b, &gates)?, 0)?;
    let out_4b = dev_4b.forward_traced(&dev_4b.carry(&prefix)?, &[2])?;
    let tr_4b = trace(&out_4b)?;
    let sc_4b = rel_scores(&out_4b)?;
    let b1_4b = block_rel_scores(&sc_4b, 2, 8, 0, 1);
    require(
        [b1_4b[0], b1_4b[1], b1_4b[2], b1_4b[4], b1_4b[5]] == [0, 8, 4, 0, 8]
            && summary(&level(&tr_4b, 2, 2, 0, 1, 0), 2).1 == [1, 5],
        "C4b unequal d_L1 conflicting successors must rank top[0..2] = [1, 5] by d_L1 asc before recency",
    )
}

/// C5: Suffix / target causal isolation across multi-row batches and target mutations.
fn relation_suffix_and_target_isolation() -> Result<()> {
    let tokens = [
        [B, A],
        [A, D_ROOT],
        [NEG_A, NEG_B],
        [C_ROOT, A],
        [D_ROOT, A],
    ];
    let mut gates = [0i32; 14];
    gates[0] = 1;
    gates[2] = 1;
    for g in &mut gates[4..14] {
        *g = 1;
    }
    let device = Device::from_snapshot(relation_snapshot(6, 2, 2, 2, &tokens, &gates)?, 0)?;
    let make_windows = |t4: u32, t5: u32| {
        vec![
            vec![
                identity(0, 0),
                identity(1, 1),
                identity(2, 2),
                identity(3, 3),
                identity(t4, 4),
                identity(t5, 5),
            ],
            vec![
                identity(1, 6),
                identity(2, 7),
                identity(0, 8),
                identity(1, 9),
            ],
        ]
    };
    let base_in = device.carry_windows(&make_windows(4, 1))?;
    let base_out = device.forward_traced(&base_in, &[2, 2])?;
    let base_tr = trace(&base_out)?;
    let base_sc = rel_scores(&base_out)?;
    let base_vec = base_out.vectors()?;
    let row_tr_stride = 2 * (TRACE_ROW + 2 * TRACE_LEVEL);
    let row_sc_stride = 2 * 8;

    for t4 in 0..5u32 {
        for t5 in 0..5u32 {
            let cur_in = device.carry_windows(&make_windows(t4, t5))?;
            let cur_out = device.forward_traced(&cur_in, &[2, 2])?;
            let cur_tr = trace(&cur_out)?;
            let cur_sc = rel_scores(&cur_out)?;
            let cur_vec = cur_out.vectors()?;
            require(
                cur_tr[row_tr_stride..] == base_tr[row_tr_stride..]
                    && cur_sc[row_sc_stride..] == base_sc[row_sc_stride..]
                    && cur_vec[16..] == base_vec[16..],
                "C5 row-0 suffix mutation altered shorter row-1 trace, relation scores, or output vector",
            )?;
        }
    }
    for target0 in 0..5u32 {
        for target1 in 0..5u32 {
            let cur_out = device.forward_traced(&base_in, &[target0, target1])?;
            require(
                trace(&cur_out)? == base_tr
                    && rel_scores(&cur_out)? == base_sc
                    && cur_out.vectors()? == base_vec,
                "C5 target mutation altered read trace, relation scores, or output vectors",
            )?;
        }
    }
    Ok(())
}

/// C6: `last <= length - 1` endpoint boundary on a length-4 prefix inside a context-6 model.
fn relation_endpoint_causal_boundary() -> Result<()> {
    let tokens = [[B, A], [A, D_ROOT], [NEG_A, NEG_B]];
    let mut gates = [0i32; 14];
    gates[2] = 1;
    gates[3] = 1;
    for g in &mut gates[4..14] {
        *g = 1;
    }
    let device = Device::from_snapshot(relation_snapshot(6, 2, 2, 2, &tokens, &gates)?, 0)?;
    let windows = vec![vec![
        identity(0, 0), // atom 0, node 0: x
        identity(1, 1), // atom 1, node 1: A
        identity(1, 2), // atom 2, node 2: A
        identity(1, 3), // atom 3, node 4: A (query, last = 3)
    ]];
    let input = device.carry_windows(&windows)?;
    let result = device.forward_traced(&input, &[2])?;
    let t = trace(&result)?;
    let scores = rel_scores(&result)?;
    let b1 = block_rel_scores(&scores, 2, 8, 0, 1);

    // 1. Nodes 5, 6, 7 (last >= 4) are absent from the length-4 row plan and never endpoints.
    require(
        (b1[5] as u32) == ABSENT && (b1[6] as u32) == ABSENT && (b1[7] as u32) == ABSENT,
        "C6 nodes 5, 6, 7 with last >= length must be ABSENT in relation score trace",
    )?;
    // 2. Node 4 (atom 3 = length - 1, query) receives edges in the prefix graph (non-zero score)
    //    but is excluded from level-0 candidate readout (eligible = 3 covering nodes 0, 1, 2).
    let lv0 = level(&t, 2, 2, 0, 1, 0);
    let (hdr0, top0, _, _) = summary(&lv0, 2);
    require(
        b1[4] > 0 && hdr0[1] == 2 && hdr0[3] == 3 && top0.iter().all(|&n| n < 3),
        "C6 level-0 readout must have eligible = 3 (nodes 0, 1, 2) and exclude query node 4",
    )?;
    // 3. Level-1 interior binder node 3 (atoms 0..=2, last = 2 <= length - 2) is eligible
    //    (eligible = 1, top[0] = 3), while binder node 7 (last = 5 >= length) is excluded.
    let lv1 = level(&t, 2, 2, 0, 1, 1);
    require(
        lv1[1] == 1 && lv1[3] == 1 && lv1[4] == 3 && lv1[5] == ABSENT,
        "C6 level-1 readout must admit only binder node 3 (eligible = 1, top[0] = 3) and exclude node 7",
    )
}

/// C7: Deterministic tie order across the 4 reachable keys (`rel_score desc`, `d_L1 asc`,
/// `d_Hamming asc`, `last desc`; 5th key `node asc` is a defensive total-order guard).
fn relation_deterministic_tie_order() -> Result<()> {
    let tokens = [
        [A, A],      // 0: root 0 (d_L1=0, d_H=0)
        [NEG_A, A],  // 1: root 1 (d_L1=8, d_H=2)
        [C_ROOT, A], // 2: root 2 (d_L1=4, d_H=1)
        [B, A],      // 3: root 14 (d_L1=8, d_H=4)
        [D_ROOT, A], // 4: root 15 (d_L1=4, d_H=1)
    ];
    // (7a) rel_score desc (gates[6] = +1): atoms 0..=4 have frames [0, 14, 2, 1, 0].
    let mut gates_7a = [0i32; 14];
    gates_7a[6] = 1;
    let dev_7a = Device::from_snapshot(relation_snapshot(6, 2, 4, 1, &tokens, &gates_7a)?, 0)?;
    let win_7a = vec![vec![
        identity(0, 0), // node 0: root 0
        identity(3, 1), // node 1: root 14
        identity(2, 2), // node 2: root 2
        identity(1, 3), // node 4: root 1
        identity(0, 4), // node 5: root 0 (query)
    ]];
    let out_7a = dev_7a.forward_traced(&dev_7a.carry_windows(&win_7a)?, &[0])?;
    let sc_7a = rel_scores(&out_7a)?;
    let b1_7a = block_rel_scores(&sc_7a, 2, 8, 0, 1);
    require(
        [b1_7a[0], b1_7a[1], b1_7a[2], b1_7a[4]] == [0, 8, 0, 4]
            && summary(&level(&trace(&out_7a)?, 2, 2, 0, 1, 0), 4).1 == [1, 4, 0, 2],
        "C7a rel_score desc tie-break order differs from [1, 4, 0, 2]",
    )?;

    // (7b) d_L1 asc & d_Hamming asc (gates[6] = 0): atoms 0..=4 have frames [0, 2, 1, 14, 0].
    let gates_zero = [0i32; 14];
    let dev_7b = Device::from_snapshot(relation_snapshot(6, 2, 4, 1, &tokens, &gates_zero)?, 0)?;
    let win_7b = vec![vec![
        identity(0, 0), // node 0: root 0 (d_L1=0, d_H=0)
        identity(2, 1), // node 1: root 2 (d_L1=4, d_H=1)
        identity(1, 2), // node 2: root 1 (d_L1=8, d_H=2, last=2)
        identity(3, 3), // node 4: root 14 (d_L1=8, d_H=4, last=3)
        identity(0, 4), // node 5: root 0 (query)
    ]];
    let out_7b = dev_7b.forward_traced(&dev_7b.carry_windows(&win_7b)?, &[0])?;
    require(
        summary(&level(&trace(&out_7b)?, 2, 2, 0, 1, 0), 4).1 == [0, 1, 2, 4],
        "C7b d_L1 asc and d_Hamming asc tie-break order differs from [0, 1, 2, 4]",
    )?;

    // (7c) last desc (gates[6] = 0): atoms 0..=4 have frames [0, 2, 1, 15, 0].
    let win_7c = vec![vec![
        identity(0, 0), // node 0: root 0 (d_L1=0, d_H=0)
        identity(2, 1), // node 1: root 2 (d_L1=4, d_H=1, last=1)
        identity(1, 2), // node 2: root 1 (d_L1=8, d_H=2, last=2)
        identity(4, 3), // node 4: root 15 (d_L1=4, d_H=1, last=3)
        identity(0, 4), // node 5: root 0 (query)
    ]];
    let out_7c = dev_7b.forward_traced(&dev_7b.carry_windows(&win_7c)?, &[0])?;
    require(
        summary(&level(&trace(&out_7c)?, 2, 2, 0, 1, 0), 4).1 == [0, 4, 1, 2],
        "C7c last desc tie-break order differs from [0, 4, 1, 2]",
    )
}

/// C8: Bounded sweeps (`1..=5` literal integer score checks in 8a and 8b) and typed refusals (8c).
fn relation_bounded_sweeps_and_refusals() -> Result<()> {
    let tokens = [
        [B, A],     // 0: tok_x = [14, 0]
        [A, A],     // 1: tok_A = [0, 0]
        [NEG_A, B], // 2: tok_B = [1, 14]
        [B, NEG_A], // 3: tok_C = [14, 1]
        [NEG_A, A], // 4: tok_y = [1, 0]
    ];
    let prefix = [
        identity(0, 0), // node 0: x
        identity(1, 1), // node 1: A
        identity(2, 2), // node 2: B
        identity(3, 3), // node 4: C
        identity(4, 4), // node 5: y
        identity(1, 5), // node 6: A (query)
    ];

    // (8a) gates[6] = +1 only (SequentialSuccessor at level 0) across sweeps 1..=5.
    let mut gates_8a = [0i32; 14];
    gates_8a[6] = 1;
    let expected_8a: [(u32, [i32; 5], [u32; 2]); 5] = [
        (1, [0, 0, 8, 0, 0], [2, 1]),
        (2, [0, 0, 8, 8, 0], [2, 4]),
        (3, [0, 0, 8, 16, 8], [4, 5]),
        (4, [0, 0, 8, 24, 24], [5, 4]),
        (5, [0, 0, 8, 32, 48], [5, 4]),
    ];
    for (sweeps, exp_scores, exp_top) in expected_8a {
        let dev =
            Device::from_snapshot(relation_snapshot(6, 2, 2, sweeps, &tokens, &gates_8a)?, 0)?;
        let out = dev.forward_traced(&dev.carry(&prefix)?, &[2])?;
        let tr = trace(&out)?;
        let sc = rel_scores(&out)?;
        let b1 = block_rel_scores(&sc, 2, 8, 0, 1);
        require(
            [b1[0], b1[1], b1[2], b1[4], b1[5]] == exp_scores
                && summary(&level(&tr, 2, 2, 0, 1, 0), 2).1 == exp_top,
            "C8a SequentialSuccessor multi-sweep scores or top[0..2] differ from hand-derived literals",
        )?;
    }

    // (8b) All 10 relation trits gates[4..14] = +1 across sweeps 1..=5.
    let mut gates_8b = [0i32; 14];
    for g in &mut gates_8b[4..14] {
        *g = 1;
    }
    let expected_8b: [(u32, [i32; 8]); 5] = [
        (1, [8, 0, 16, 8, 0, 8, 16, 8]),
        (2, [16, 40, 24, 32, 40, 48, 40, 48]),
        (3, [88, 128, 136, 112, 176, 240, 264, 240]),
        (4, [328, 552, 504, 464, 880, 1232, 1240, 1144]),
        (5, [1344, 2176, 2072, 1848, 4088, 5880, 5952, 5424]),
    ];
    for (sweeps, exp_all) in expected_8b {
        let dev =
            Device::from_snapshot(relation_snapshot(6, 2, 2, sweeps, &tokens, &gates_8b)?, 0)?;
        let out = dev.forward_traced(&dev.carry(&prefix)?, &[2])?;
        let tr = trace(&out)?;
        let sc = rel_scores(&out)?;
        let b1 = block_rel_scores(&sc, 2, 8, 0, 1);
        require(
            b1 == exp_all,
            "C8b all-active 8-node relation score vector differs from hand-derived literal",
        )?;
        if sweeps == 5 {
            require(
                [b1[0], b1[1], b1[2], b1[4], b1[5]] == [1344, 2176, 2072, 4088, 5880]
                    && summary(&level(&tr, 2, 2, 0, 1, 0), 2).1 == [5, 4]
                    && b1[3] == 1848
                    && summary(&level(&tr, 2, 2, 0, 1, 1), 1).1 == [3],
                "C8b sweeps=5 level-0 or level-1 selection differs from [5, 4] / [3]",
            )?;
        }
    }

    // (8b2) Most-recent prior rule for SameToken (e=3, gates[7]=+1) and SameFrame (e=4, gates[8]=+1)
    //       when a token, and separately a frame on distinct token IDs, occurs three times before the query.
    let tokens_mr = [
        [C_ROOT, A],     // 0: frame 2 (C_ROOT), d_L1=4, d_H=1, m^(0)=4
        [C_ROOT, B],     // 1: frame 2 (C_ROOT), d_L1=4, d_H=1, m^(0)=4 (distinct token ID)
        [C_ROOT, NEG_B], // 2: frame 2 (C_ROOT), d_L1=4, d_H=1, m^(0)=4 (distinct token ID)
        [A, A],          // 3: frame 0 (A),      d_L1=0, d_H=0, m^(0)=8
        [A, B],          // 4: frame 0 (A),      d_L1=0, d_H=0, m^(0)=8 (distinct token ID)
        [A, D_ROOT],     // 5: frame 0 (A),      d_L1=0, d_H=0, m^(0)=8 (distinct token ID)
    ];
    let prefix_tok_c3 = [
        identity(0, 0), // node 0: tok 0 (1st)
        identity(0, 1), // node 1: tok 0 (2nd)
        identity(0, 2), // node 2: tok 0 (3rd before query)
        identity(3, 3), // node 4: tok 3 (1st)
        identity(3, 4), // node 5: tok 3 (2nd)
        identity(3, 5), // node 6: tok 3 (query)
    ];
    let prefix_tok_a3 = [
        identity(3, 0), // node 0: tok 3 (1st)
        identity(3, 1), // node 1: tok 3 (2nd)
        identity(3, 2), // node 2: tok 3 (3rd before query)
        identity(0, 3), // node 4: tok 0 (1st)
        identity(0, 4), // node 5: tok 0 (2nd)
        identity(3, 5), // node 6: tok 3 (query, sees 3 prior tok 3 occurrences)
    ];
    let prefix_frm_c3 = [
        identity(0, 0), // node 0: frame 2 (1st, tok 0)
        identity(1, 1), // node 1: frame 2 (2nd, tok 1)
        identity(2, 2), // node 2: frame 2 (3rd before query, tok 2)
        identity(3, 3), // node 4: frame 0 (1st, tok 3)
        identity(4, 4), // node 5: frame 0 (2nd, tok 4)
        identity(5, 5), // node 6: frame 0 (query, tok 5)
    ];
    let prefix_frm_a3 = [
        identity(3, 0), // node 0: frame 0 (1st, tok 3)
        identity(4, 1), // node 1: frame 0 (2nd, tok 4)
        identity(5, 2), // node 2: frame 0 (3rd before query, tok 5)
        identity(0, 3), // node 4: frame 2 (1st, tok 0)
        identity(1, 4), // node 5: frame 2 (2nd, tok 1)
        identity(3, 5), // node 6: frame 0 (query, sees 3 prior frame-0 nodes)
    ];
    for (gate_idx, p_c3, p_a3, label) in [
        (7usize, &prefix_tok_c3, &prefix_tok_a3, "C8 SameToken (e=3)"),
        (8usize, &prefix_frm_c3, &prefix_frm_a3, "C8 SameFrame (e=4)"),
    ] {
        let mut gates_mr = [0i32; 14];
        gates_mr[gate_idx] = 1;
        for (sweeps, exp_c3, exp_top_c3) in [
            (1u32, [0, 4, 4, 0, 0, 8, 8, 0], [5u32, 2u32]),
            (2u32, [0, 4, 8, 0, 0, 8, 16, 0], [5u32, 2u32]),
        ] {
            let dev = Device::from_snapshot(
                relation_snapshot(6, 2, 2, sweeps, &tokens_mr, &gates_mr)?,
                0,
            )?;
            let out = dev.forward_traced(&dev.carry(p_c3)?, &[3])?;
            let tr = trace(&out)?;
            let sc = rel_scores(&out)?;
            let b1 = block_rel_scores(&sc, 2, 8, 0, 1);
            require(
                b1 == exp_c3 && summary(&level(&tr, 2, 2, 0, 1, 0), 2).1 == exp_top_c3,
                label,
            )?;
        }
        for (sweeps, exp_a3) in [
            (1u32, [0, 8, 8, 0, 0, 4, 8, 0]),
            (2u32, [0, 8, 16, 0, 0, 4, 16, 0]),
            (3u32, [0, 8, 24, 0, 0, 4, 32, 0]),
        ] {
            let dev = Device::from_snapshot(
                relation_snapshot(6, 2, 2, sweeps, &tokens_mr, &gates_mr)?,
                0,
            )?;
            let out = dev.forward_traced(&dev.carry(p_a3)?, &[0])?;
            let tr = trace(&out)?;
            let sc = rel_scores(&out)?;
            let b1 = block_rel_scores(&sc, 2, 8, 0, 1);
            require(
                b1 == exp_a3 && summary(&level(&tr, 2, 2, 0, 1, 0), 2).1 == [2, 1],
                label,
            )?;
        }
    }

    // (8c) Validation and refusal checks.
    let valid_spec = Spec {
        vocabulary: 5,
        blocks: 2,
        context: 6,
        kept_cells: 2,
    };
    expect_invalid(
        ReadSpec::Relation { kept: 2, sweeps: 0 }.validate(valid_spec),
        "relation sweep count must be in 1..=5",
        "sweeps = 0",
    )?;
    expect_invalid(
        ReadSpec::Relation { kept: 2, sweeps: 6 }.validate(valid_spec),
        "relation sweep count must be in 1..=5",
        "sweeps = 6",
    )?;
    expect_invalid(
        ReadSpec::Relation { kept: 0, sweeps: 1 }.validate(valid_spec),
        "relation kept count must be in 1..=8",
        "kept = 0",
    )?;
    expect_invalid(
        ReadSpec::Relation { kept: 9, sweeps: 1 }.validate(valid_spec),
        "relation kept count must be in 1..=8",
        "kept = 9",
    )?;
    expect_invalid(
        ReadSpec::Relation { kept: 2, sweeps: 1 }.validate(Spec {
            vocabulary: 5,
            blocks: 1,
            context: 6,
            kept_cells: 2,
        }),
        "requires vocabulary2..8192, whole8D blocks2..16, context1..243, kept cells1..min(16,vocabulary)",
        "blocks = 1",
    )?;
    let device = Device::from_snapshot(relation_snapshot(6, 2, 2, 1, &tokens, &[])?, 0)?;
    let carried = device.carry(&prefix)?;
    expect_invalid(
        device.propose(&carried, &[2], Parameter::Route(0)),
        "route parameters are inactive in a relation model",
        "Route(0) in Relation mode",
    )?;
    expect_invalid(
        device.propose(&carried, &[2], Parameter::Gate(14)),
        "proposal parameter index exceeds its declared inventory",
        "out-of-inventory Gate(14)",
    )?;
    for bad_idx in [0usize, 6usize] {
        let mut bad = relation_snapshot(6, 2, 2, 1, &tokens, &[])?;
        bad.gates[bad_idx] = 2;
        expect_invalid(
            Device::from_snapshot(bad, 0),
            "discrete CUDA snapshot validation refused with status 10",
            "gate trit = 2 in discrete_validate",
        )?;
    }
    // Lone-token (context = 1, length = 1): both combine gate (1) and relation gate (2) tie and no-op.
    let lone = Device::from_snapshot(relation_snapshot(1, 2, 1, 1, &[[A, B], [A, A]], &[])?, 0)?;
    let single = lone.carry(&[identity(0, 0)])?;
    for gate_idx in [1u32, 2u32] {
        let proposal = lone.propose(&single, &[1], Parameter::Gate(gate_idx))?;
        let losses = proposal.candidate_losses()?;
        let observed = proposal.observation()?;
        require(
            losses[0] == losses[1]
                && losses[1] == losses[2]
                && losses[3..].iter().all(|&l| l == MAX)
                && observed.candidate == 0
                && observed.before == observed.after
                && !observed.any_candidate_cost_differs_from_identity,
            "C8c lone-token gate proposal must tie active ordinals and report no sensitivity",
        )?;
    }
    Ok(())
}

pub const NAMES: &[&str] = &[
    "relation_off_flock_parity",
    "relation_copy_successor_fixture",
    "relation_irrelevant_change_invariance",
    "relation_conflicting_successors",
    "relation_suffix_and_target_isolation",
    "relation_endpoint_causal_boundary",
    "relation_deterministic_tie_order",
    "relation_bounded_sweeps_and_refusals",
    "active_leaf_initialization_and_matched_off_path",
];

/// §7.8 item 3(ii/iii): INERT at the same sealed C2 shape and INIT LOCK at
/// the registered training shape. Part (i), ACTIVE, is the existing registered
/// relation_copy_successor_fixture; its exact literals are not rerun or replaced.
/// All read selection, carry, consensus and loss computations use Device.
fn active_leaf_initialization_and_matched_off_path() -> Result<()> {
    // INERT uses exactly the ACTIVE tokens, prefix, target, read shape and seed.
    // Only the declared interventions differ: all combines zero, Gate(6)=0/+1.
    let zero = relation_snapshot(6, 2, 1, 1, &C2_TOKENS, &[0; 14])?;
    let mut positive = zero.clone();
    positive.gates[6] = 1;
    let baseline = Device::from_snapshot(zero.clone(), 0)?;
    let changed = Device::from_snapshot(positive.clone(), 0)?;
    let prefix = c2_prefix();
    let baseline_out = baseline.forward_traced(&baseline.carry(&prefix)?, &[2])?;
    let changed_out = changed.forward_traced(&changed.carry(&prefix)?, &[2])?;
    require(
        baseline_out.vectors()? == changed_out.vectors()?
            && baseline_out.losses == changed_out.losses
            && baseline_out.predictions == changed_out.predictions
            && baseline_out.token_rivals == changed_out.token_rivals
            && baseline_out.address_rivals == changed_out.address_rivals
            && baseline_out.admitted_cells == changed_out.admitted_cells,
        "C2 INERT: Gate(6)=0/+1 changed a real forward field with all combine gates zero",
    )?;
    // Internal traces are reported separately and are deliberately excluded from
    // the bit-identity assertion over actual forward outputs.
    println!(
        "{}",
        serde_json::json!({"control":"active_leaf_initialization_and_matched_off_path",
            "part":"INERT","shape":"sealed C2 B2/C6/K2/read-kept1/seed7",
            "all_combine_gates":0,"relation_gate_index":6,"relation_gate_values":[0,1],
            "real_forward_fields_bit_identical":true,
            "baseline_trace":trace(&baseline_out)?,"changed_trace":trace(&changed_out)?,
            "baseline_relation_scores":rel_scores(&baseline_out)?,
            "changed_relation_scores":rel_scores(&changed_out)?,"applied_updates":0})
    );
    for (device, before) in [(&baseline, &zero), (&changed, &positive)] {
        require(
            serde_json::to_vec(&device.snapshot()?).map_err(|e| Error::Invalid(e.to_string()))?
                == serde_json::to_vec(before).map_err(|e| Error::Invalid(e.to_string()))?,
            "C2 INERT forward mutated snapshot storage or epoch",
        )?;
    }

    // INIT LOCK uses the registered training shape, not the C2 shape. Treatment
    // and primary relation-disabled control both instantiate Relation's full
    // 45-gate inventory; the driver, not this initializer, refuses control apply.
    let spec = Spec {
        vocabulary: 4096,
        blocks: 4,
        context: 192,
        kept_cells: 8,
    };
    let read_spec = ReadSpec::Relation { kept: 8, sweeps: 1 };
    let mut expected = [0i32; 45];
    for index in [0usize, 5, 10, 15] {
        expected[index] = 1;
    }
    for seed in [1u32, 2] {
        let treatment = Device::new_with_initialization(
            spec,
            read_spec,
            crate::ObjectivePolicy::NearestAddressMarginV1,
            crate::ReadInitialization::ActiveLeafV1,
            seed,
            0,
        )?;
        let control = Device::new_with_initialization(
            spec,
            read_spec,
            crate::ObjectivePolicy::NearestAddressMarginV1,
            crate::ReadInitialization::ActiveLeafV1,
            seed,
            0,
        )?;
        let treatment_snapshot = treatment.snapshot()?;
        let control_snapshot = control.snapshot()?;
        for snapshot in [&treatment_snapshot, &control_snapshot] {
            require(snapshot.spec == spec && snapshot.read_spec == read_spec
                && snapshot.seed == seed && snapshot.epoch == 0
                && snapshot.read_initialization == crate::ReadInitialization::ActiveLeafV1
                && snapshot.gates == expected,
                "INIT LOCK requires exact V4096/B4/C192/K8 gates0/5/10/15=+1 and every other gate zero")?;
        }
        let treatment_bytes =
            serde_json::to_vec(&treatment_snapshot).map_err(|e| Error::Invalid(e.to_string()))?;
        require(
            treatment_bytes
                == serde_json::to_vec(&control_snapshot)
                    .map_err(|e| Error::Invalid(e.to_string()))?,
            "INIT LOCK treatment and primary Relation-off control initialization differ",
        )?;
        let reloaded = Device::from_snapshot(control_snapshot.clone(), 0)?;
        require(
            treatment_bytes
                == serde_json::to_vec(&reloaded.snapshot()?)
                    .map_err(|e| Error::Invalid(e.to_string()))?,
            "INIT LOCK exact initial storage or identity changed on reload",
        )?;
        println!(
            "{}",
            serde_json::json!({
            "control":"active_leaf_initialization_and_matched_off_path","part":"INIT LOCK",
            "spec":spec,"read_spec":read_spec,"seed":seed,
            "initialization":crate::ReadInitialization::ActiveLeafV1,
            "plus_one_combine_indices":[0,5,10,15],"gates":control_snapshot.gates,
            "treatment_control_snapshot_bit_identical":true,"reload_bit_identical":true,
            "snapshot_sha256":crate::corpus::sha256(&treatment_bytes),"applied_updates":0})
        );
    }
    Ok(())
}

/// Same reporting semantics as `qualification::run` and `flock_qualification::run`:
/// independent findings are collected; a CUDA fault hard-stops and marks remaining
/// controls NOT_RUN; the report directory is created exclusively before any CUDA work.
pub fn run(report: &Path) -> Result<()> {
    std::fs::create_dir(report).map_err(|e| Error::Invalid(format!("exclusive report: {e}")))?;
    let controls: [fn() -> Result<()>; 9] = [
        relation_off_flock_parity,
        relation_copy_successor_fixture,
        relation_irrelevant_change_invariance,
        relation_conflicting_successors,
        relation_suffix_and_target_isolation,
        relation_endpoint_causal_boundary,
        relation_deterministic_tie_order,
        relation_bounded_sweeps_and_refusals,
        active_leaf_initialization_and_matched_off_path,
    ];
    let mut records = Vec::new();
    let (mut failed, mut passed, mut attempted, mut hard_stop) = (false, 0usize, 0usize, None);
    for (index, (name, test)) in NAMES.iter().zip(controls).enumerate() {
        attempted += 1;
        let record = match test() {
            Ok(()) => {
                passed += 1;
                serde_json::json!({"index":index+1,"name":name,"status":"PASS"})
            }
            Err(error @ Error::Cuda(_)) => {
                failed = true;
                let record = serde_json::json!({"index":index+1,"name":name,"status":"HARD_STOP","error":error.to_string()});
                hard_stop = Some(error);
                record
            }
            Err(e) => {
                failed = true;
                serde_json::json!({"index":index+1,"name":name,"status":"FAIL","error":e.to_string()})
            }
        };
        println!("{record}");
        records.push(record);
        if hard_stop.is_some() {
            break;
        }
    }
    for (index, name) in NAMES.iter().enumerate().skip(attempted) {
        let record = serde_json::json!({"index":index+1,"name":name,"status":"NOT_RUN","reason":"preceding CUDA fault; model/device safety not established"});
        println!("{record}");
        records.push(record);
    }
    let payload = serde_json::json!({"schema":crate::SCHEMA,"suite":"relation","status":if hard_stop.is_some(){"ABORTED"}else if failed{"FAILED"}else{"PASS"},"registered":NAMES.len(),"attempted":attempted,"passed":passed,"failed":attempted-passed,"not_run":NAMES.len()-attempted,"controls":records,"training":"HELD","applied_updates":0});
    std::fs::write(
        report.join("relation-controls.json"),
        serde_json::to_vec_pretty(&payload).map_err(|e| Error::Invalid(e.to_string()))?,
    )
    .map_err(|e| Error::Invalid(e.to_string()))?;
    if let Some(error) = hard_stop {
        return Err(error);
    }
    require(!failed, "one or more relation construction controls failed")
}
