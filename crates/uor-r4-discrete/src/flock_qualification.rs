//! Finite, exact CUDA construction controls for the flock read and Gate (kind 3).
//! Design: docs/labs/discrete-murmuration-read-2026-10-13 (merged 5e85f696e).
//! No apply/optimizer/corpus run. Every expectation is a literal derived by hand
//! from the generated anchor orbit (build.rs ROOTS/BFS at the pinned source):
//! root 0 = a = [2,2,0,0,0,0,0,0], root 1 = -a, root 14 = b = B12*a =
//! [0,0,2,0,2,0,0,0]; IDENTITY = 0; BFS[r][r] = 0; BFS[0][14] = 1. A BFS action
//! maps its source root onto its target root by definition, so a payload equal to
//! its own frame root is carried exactly onto the query frame root. No host
//! reference forward exists; traced values come from the same CUDA flock path.
use crate::{
    device::{Device, Forward, Snapshot},
    Error, Identity, Parameter, ReadSpec, Result, Spec,
};
use std::path::Path;

const ABSENT: u32 = u32::MAX;
const MAX: u64 = u64::MAX;
const A: [i32; 8] = [2, 2, 0, 0, 0, 0, 0, 0];
const NEG_A: [i32; 8] = [-2, -2, 0, 0, 0, 0, 0, 0];
const B: [i32; 8] = [0, 0, 2, 0, 2, 0, 0, 0];
// Trace layout (discrete.cu): per (row, block) 4 head words, then per level 28.
const TRACE_ROW: usize = 4;
const TRACE_LEVEL: usize = 28;

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
fn concat(blocks: &[[i32; 8]]) -> Vec<i32> {
    blocks.iter().flatten().copied().collect()
}
/// A two-block flock model whose codebook rows are `tokens` (each `[block0, block1]`)
/// and whose gate trits are `gates` (length blocks*stages; empty = all zero).
fn flock(
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
            "fixture gate inventory differs",
        )?;
        snapshot.gates.copy_from_slice(gates);
    }
    Ok(snapshot)
}
fn trace(forward: &Forward) -> Result<Vec<u32>> {
    forward
        .flock_trace()?
        .ok_or_else(|| Error::Invalid("flock forward returned no trace".into()))
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
/// [gate+1, kept, consensus, eligible] and the first `k` node/action/carried words.
fn summary(lv: &[u32], k: usize) -> (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>) {
    (
        lv[0..4].to_vec(),
        lv[4..4 + k].to_vec(),
        lv[12..12 + k].to_vec(),
        lv[20..20 + k].to_vec(),
    )
}

/// C7 identity read: all gates 0 gives out = query payloads bitwise; one term per block.
fn flock_identity_read() -> Result<()> {
    let snapshot = flock(3, 3, 2, &[[A, A], [NEG_A, A], [A, B]], &[])?;
    let device = Device::from_snapshot(snapshot, 0)?;
    let input = device.carry(&[identity(0, 0), identity(1, 1), identity(2, 2)])?;
    let result = device.forward_traced(&input, &[0])?;
    require(
        result.vectors()? == concat(&[A, B]),
        "all-zero gates changed the query payload",
    )?;
    let t = trace(&result)?;
    // stages = 2 for context 3. Block 0's frame is block 1 (b = 14); block 1's is block 0 (a = 0).
    require(
        head(&t, 2, 2, 0, 0) == [14, 0, 1, 0] && head(&t, 2, 2, 0, 1) == [0, 14, 1, 14],
        "identity-read trace head [F_q, out, terms, query payload] differs",
    )
}

/// C8 gate counterfactual (Codex's fixture, confirmed analytically on #2032 6156933202):
/// losses by ordinal [CURRENT 0, -1, +1] = [448, 256, 64], 13 MAX; choose ordinal 2;
/// each equals an independent snapshot forward with that trit; no state change.
fn flock_gate_counterfactual() -> Result<()> {
    let tokens = [[A, B], [A, A]];
    // Context 2 has one stage; Gate(1) is block 1, level 0.
    let device = Device::from_snapshot(flock(2, 2, 1, &tokens, &[0, 0])?, 0)?;
    let input = device.carry(&[identity(1, 0), identity(0, 1)])?;
    let before = device.snapshot()?;
    let proposal = device.propose(&input, &[1], Parameter::Gate(1))?;
    let losses = proposal.candidate_losses()?;
    let mut expected = vec![MAX; 16];
    expected[..3].copy_from_slice(&[448, 256, 64]);
    require(
        losses == expected,
        "gate candidate loss slots differ from [448,256,64,MAX x13]",
    )?;
    let observed = proposal.observation()?;
    require(
        observed.candidate == 2
            && observed.before == 448
            && observed.after == 64
            && observed.epoch == 0,
        "gate chooser must take ordinal 2 (+1) by strict improvement",
    )?;
    let repeated = device.propose(&input, &[1], Parameter::Gate(1))?;
    require(
        repeated.candidate_losses()? == expected && repeated.observation()? == observed,
        "identical gate proposals changed cost or winner",
    )?;
    let after = device.snapshot()?;
    require(
        before.table == after.table
            && before.routes == after.routes
            && before.gates == after.gates
            && before.epoch == after.epoch,
        "virtual gate proposal changed learned state",
    )?;
    for (trit, loss, out) in [(0, 448, [A, B]), (-1, 256, [A, NEG_A]), (1, 64, [A, A])] {
        let independent = Device::from_snapshot(flock(2, 2, 1, &tokens, &[0, trit])?, 0)?;
        let carried = independent.carry(&[identity(1, 0), identity(0, 1)])?;
        let result = independent.forward(&carried, &[1])?;
        require(
            result.losses == [loss] && result.vectors()? == concat(&out),
            "snapshot forward with that gate trit differs from its counterfactual slot",
        )?;
    }
    Ok(())
}

/// C5/C12 transport witnesses from the same flock path (gates 0).
/// Block 1: equal frames (a, a): identity action 0 carries P_j = a (0), which is not P_q = b (14).
/// Block 0: frames b (query) and a (context): action BFS[0][14] = 1 carries P_j = a onto b (14),
/// which is not P_q = a (0). Equality case: P_j = P_q under the identity action carries onto P_q.
fn flock_transport_witnesses() -> Result<()> {
    let device = Device::from_snapshot(flock(2, 2, 1, &[[A, B], [A, A]], &[0, 0])?, 0)?;
    let input = device.carry(&[identity(1, 0), identity(0, 1)])?;
    let t = trace(&device.forward_traced(&input, &[1])?)?;
    require(
        head(&t, 2, 1, 0, 1) == [0, 14, 1, 14],
        "block-1 trace head differs",
    )?;
    require(
        summary(&level(&t, 2, 1, 0, 1, 0), 1) == (vec![1, 1, 0, 1], vec![0], vec![0], vec![0]),
        "equal-frame transport must carry P_j unchanged (C12a)",
    )?;
    require(
        head(&t, 2, 1, 0, 0) == [14, 0, 1, 0],
        "block-0 trace head differs",
    )?;
    require(
        summary(&level(&t, 2, 1, 0, 0, 0), 1) == (vec![1, 1, 14, 1], vec![0], vec![1], vec![14]),
        "distinct-frame transport BFS[0][14] must carry a onto b (C12b)",
    )?;
    let equal = Device::from_snapshot(flock(2, 2, 1, &[[A, B], [A, B]], &[0, 0])?, 0)?;
    let carried = equal.carry(&[identity(1, 0), identity(0, 1)])?;
    let t = trace(&equal.forward_traced(&carried, &[1])?)?;
    require(
        summary(&level(&t, 2, 1, 0, 1, 0), 1) == (vec![1, 1, 14, 1], vec![0], vec![0], vec![14]),
        "equality case P_j = inverse(g) P_q must carry onto P_q (C12c)",
    )
}

/// C10 non-vacuity by explicit witness: with the block-1 gate +1, changing the one kept
/// atom's payload from a to -a moves the consensus 0 -> 1 and out [a,a] -> [a,-a]
/// (snap_2(b+a) ties a/b at L1 8, Hamming 4 -> root 0; snap_2(b-a) ties -a/b -> root 1).
fn flock_payload_witness() -> Result<()> {
    for (context_token, consensus, out) in [(A, 0u32, [A, A]), (NEG_A, 1u32, [A, NEG_A])] {
        let device =
            Device::from_snapshot(flock(2, 2, 1, &[[A, B], [A, context_token]], &[0, 1])?, 0)?;
        let input = device.carry(&[identity(1, 0), identity(0, 1)])?;
        let result = device.forward_traced(&input, &[1])?;
        require(
            result.vectors()? == concat(&out),
            "one-payload change did not move the output as derived",
        )?;
        require(
            level(&trace(&result)?, 2, 1, 0, 1, 0)[2] == consensus,
            "one-payload change did not move the consensus as derived",
        )?;
    }
    Ok(())
}

/// C4/C5/C6: caller-prefix windows only. Context 4 (stages 2), K = 2.
/// Row 0 = [a,a],[a,a],[b,b],[a,a] (length 4); row 1 = [a,a] x3 (length 3).
/// Row 0, both blocks: level 0 eligible 3; nodes 0,1 tie at distance 0 and node 2 (b) is at
/// L1 8, so recency keeps [1, 0] although node 2 is most recent; level 1 eligible 1
/// (binder node 3, leaves 0..2 < 3) with frame a, payload snap_3(a + a + BFS[14][0]b) = a.
/// Row 1: the only binder contains the query, so level-1 eligible 0. Row 0 cannot change row 1.
fn flock_selection_and_eligibility() -> Result<()> {
    let tokens = [[A, A], [B, B]];
    let device = Device::from_snapshot(flock(4, 2, 2, &tokens, &[])?, 0)?;
    let rows = |third: u32| {
        vec![
            vec![
                identity(0, 0),
                identity(0, 1),
                identity(third, 2),
                identity(0, 3),
            ],
            vec![identity(0, 4), identity(0, 5), identity(0, 6)],
        ]
    };
    let input = device.carry_windows(&rows(1))?;
    let result = device.forward_traced(&input, &[0, 0])?;
    let t = trace(&result)?;
    for block in 0..2 {
        require(
            head(&t, 2, 2, 0, block) == [0, 0, 1, 0],
            "row-0 trace head differs",
        )?;
        require(
            summary(&level(&t, 2, 2, 0, block, 0), 2) == (vec![1, 2, 0, 3], vec![1, 0], vec![0, 0], vec![0, 0]),
            "level-0 rank must keep the two distance-0 atoms by recency, not the nearer-in-time b (C5)",
        )?;
        require(
            summary(&level(&t, 2, 2, 0, block, 1), 1)
                == (vec![1, 1, 0, 1], vec![3], vec![0], vec![0]),
            "length-4 prefix must admit exactly binder 3 with payload a (C6)",
        )?;
        let empty = level(&t, 2, 2, 1, block, 1);
        require(
            empty[0..4] == [1, 0, ABSENT, 0] && empty[4..28].iter().all(|&w| w == ABSENT),
            "length-3 prefix must admit no binder: it contains the query (C4/C6)",
        )?;
        require(
            summary(&level(&t, 2, 2, 1, block, 0), 2)
                == (vec![1, 2, 0, 2], vec![1, 0], vec![0, 0], vec![0, 0]),
            "length-3 prefix level 0 differs",
        )?;
    }
    let changed = device.forward_traced(&device.carry_windows(&rows(0))?, &[0, 0])?;
    let u = trace(&changed)?;
    let stride = 2 * (TRACE_ROW + 2 * TRACE_LEVEL);
    require(
        u[stride..] == t[stride..]
            && changed.vectors()?[16..] == result.vectors()?[16..]
            && changed.losses[1] == result.losses[1],
        "another row's token changed a shorter prefix's trace or output",
    )
}

/// Typed refusals and the all-tie no-op: Gate in Route mode and Route in Flock mode
/// (typed host refusals), an invalid gate trit (status 10), an out-of-inventory Gate index, and a
/// gate whose level has no eligible candidate (all three ordinals tie: CURRENT, no change).
fn flock_refusals_and_noop() -> Result<()> {
    let route = Device::new(
        Spec {
            vocabulary: 2,
            blocks: 2,
            context: 2,
            kept_cells: 2,
        },
        7,
        0,
    )?;
    let input = route.carry(&[identity(1, 0), identity(0, 1)])?;
    match route.propose(&input, &[1], Parameter::Gate(0)) {
        Err(Error::UnimplementedParameter { kind: 3, index: 0 }) => {}
        Err(error) => return Err(error),
        Ok(_) => {
            return Err(Error::Invalid(
                "gate proposal in Route mode was not refused".into(),
            ))
        }
    }
    let device = Device::from_snapshot(flock(2, 2, 1, &[[A, B], [A, A]], &[0, 0])?, 0)?;
    let input = device.carry(&[identity(1, 0), identity(0, 1)])?;
    expect_invalid(
        device.propose(&input, &[1], Parameter::Route(0)),
        "route parameters are inactive in a flock model",
        "route proposal in Flock mode",
    )?;
    expect_invalid(
        device.propose(&input, &[1], Parameter::Gate(2)),
        "proposal parameter index exceeds its declared inventory",
        "out-of-inventory gate index",
    )?;
    let mut bad = flock(2, 2, 1, &[[A, B], [A, A]], &[0, 0])?;
    bad.gates[0] = 2;
    expect_invalid(
        Device::from_snapshot(bad, 0),
        "discrete CUDA snapshot validation refused with status 10",
        "gate trit outside {-1,0,1}",
    )?;
    // Context 1: the query is the only token, so no level has an eligible candidate.
    let lone = Device::from_snapshot(flock(1, 2, 1, &[[A, B], [A, A]], &[0, 0])?, 0)?;
    let single = lone.carry(&[identity(0, 0)])?;
    let proposal = lone.propose(&single, &[1], Parameter::Gate(1))?;
    let losses = proposal.candidate_losses()?;
    let observed = proposal.observation()?;
    require(
        losses[0] == losses[1]
            && losses[1] == losses[2]
            && losses[3..].iter().all(|&l| l == MAX)
            && observed.candidate == 0
            && observed.before == observed.after
            && !observed.any_candidate_cost_differs_from_identity,
        "a gate with no eligible candidate must tie active ordinals and keep CURRENT; MAX padding must not manufacture sensitivity",
    )
}

pub const NAMES: &[&str] = &[
    "flock_identity_read",
    "flock_gate_counterfactual",
    "flock_transport_witnesses",
    "flock_payload_witness",
    "flock_selection_and_eligibility",
    "flock_refusals_and_noop",
];

/// Same reporting semantics as `qualification::run`: independent findings are
/// collected; a CUDA fault hard-stops and the rest are NOT_RUN; the report
/// directory is created exclusively before any CUDA work.
pub fn run(report: &Path) -> Result<()> {
    std::fs::create_dir(report).map_err(|e| Error::Invalid(format!("exclusive report: {e}")))?;
    let controls: [fn() -> Result<()>; 6] = [
        flock_identity_read,
        flock_gate_counterfactual,
        flock_transport_witnesses,
        flock_payload_witness,
        flock_selection_and_eligibility,
        flock_refusals_and_noop,
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
    let payload = serde_json::json!({"schema":crate::SCHEMA,"suite":"flock","status":if hard_stop.is_some(){"ABORTED"}else if failed{"FAILED"}else{"PASS"},"registered":NAMES.len(),"attempted":attempted,"passed":passed,"failed":attempted-passed,"not_run":NAMES.len()-attempted,"controls":records,"training":"HELD","applied_updates":0});
    std::fs::write(
        report.join("flock-controls.json"),
        serde_json::to_vec_pretty(&payload).map_err(|e| Error::Invalid(e.to_string()))?,
    )
    .map_err(|e| Error::Invalid(e.to_string()))?;
    if let Some(error) = hard_stop {
        return Err(error);
    }
    require(!failed, "one or more flock construction controls failed")
}
