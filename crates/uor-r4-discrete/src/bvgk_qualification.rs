//! Finite CUDA controls for the BVG-K Stage C2 block path (handoff H4a):
//! `propose_block` and the one-epoch `apply_block`, on small fixture models.
//!
//! Every loss, choice, decision and commit is the device's own. The host only
//! builds fixtures, compares device outputs and overlays storage that single
//! device applies wrote. References are independent devices restored from the
//! same snapshot, so no reference shares the original's buffers.
//!
//! V0 source identity; RC re-carry parity; UT original untouched by
//! `propose_block`; JR joint loss against the composed single moves; DC the
//! decision rule with V4 (commit mask) and V5 (committed loss); V3 block apply
//! equals the forced sequential single applies; RF refusals; C1R the C1 V2
//! sweep; TP throughput (diagnostic only). Training stays held: the applies
//! here touch fixture devices only and are counted in `applied_updates`.
use crate::{
    bvgk_block::BlockRecord,
    device::{BvgkKernels, CarriedBatch, Device, RowsKScoring, Snapshot},
    Error, Identity, ObjectivePolicy, Parameter, ReadSpec, Result, Spec,
};
use std::path::Path;

fn require(ok: bool, why: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error::Invalid(why.into()))
    }
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self, bound: u32) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) % u64::from(bound)) as u32
    }
}

fn windows(rng: &mut Lcg, rows: usize, context: u32, vocabulary: u32) -> Vec<Vec<Identity>> {
    let mut occurrence = 0u64;
    (0..rows)
        .map(|_| {
            let length = 2 + rng.next(context - 1);
            (0..length)
                .map(|_| {
                    occurrence += 1;
                    Identity {
                        token: rng.next(vocabulary),
                        occurrence,
                        record: 17,
                        version: 9,
                    }
                })
                .collect()
        })
        .collect()
}

type Control = fn(&mut Tally) -> Result<serde_json::Value>;

#[derive(Default)]
struct Tally {
    cases: u64,
    proposals: u64,
    /// Proposals with `|S_t| >= 2` checked by UT and JR.
    multi: u64,
    empty: u64,
    single: u64,
    joint_wins: u64,
    joint_loses_or_ties: u64,
    /// Blocks every coordinate of which K-coordinate scoring refused.
    refused_blocks: u64,
    commits: u64,
    v3_cases: u64,
    v3_joint: u64,
    lanes: u64,
    racy_status: u64,
    refusals: u64,
    applied_updates: u64,
}

/// The four admitted configuration cells of the C1 sweep: read, blocks, objective.
const CELLS: [(ReadSpec, u32, ObjectivePolicy); 4] = [
    (ReadSpec::Route, 2, ObjectivePolicy::NearestAddressMarginV1),
    (ReadSpec::Route, 4, ObjectivePolicy::NearestAddressMarginV1),
    (ReadSpec::Route, 4, ObjectivePolicy::OccupiedCellRankHingeV1),
    (
        ReadSpec::Relation { kept: 2, sweeps: 1 },
        2,
        ObjectivePolicy::NearestAddressMarginV1,
    ),
];
const VOCABULARY: u32 = 16;
const CONTEXT: u32 = 12;

fn spec(blocks: u32) -> Spec {
    Spec {
        vocabulary: VOCABULARY,
        blocks,
        context: CONTEXT,
        kept_cells: 4,
    }
}

fn same_state(a: &Snapshot, b: &Snapshot) -> bool {
    a.table == b.table && a.routes == b.routes && a.gates == b.gates
}

/// UT wrapper: learned state and epoch are byte-identical across `call`.
fn unchanged<T>(device: &Device, call: impl FnOnce() -> Result<T>) -> Result<T> {
    let before = device.snapshot()?;
    let outcome = call();
    let after = device.snapshot()?;
    require(
        same_state(&before, &after) && before.epoch == after.epoch,
        "UT: a proposal or refusal changed learned state or epoch",
    )?;
    outcome
}

/// Each kind's complete admitted inventory for the read.
fn inventories(read: ReadSpec, snapshot: &Snapshot, blocks: u32) -> Vec<Vec<Parameter>> {
    let first: Vec<Parameter> = match read {
        ReadSpec::Route => (0..snapshot.routes.len() as u32)
            .map(Parameter::Route)
            .collect(),
        _ => (0..snapshot.gates.len() as u32)
            .map(Parameter::Gate)
            .collect(),
    };
    vec![
        first,
        (0..VOCABULARY * blocks)
            .map(Parameter::VectorBlock)
            .collect(),
    ]
}

/// Lane 0 (current loss) of the first scored coordinate of `block`.
fn current_loss(
    device: &Device,
    batch: &CarriedBatch,
    targets: &[u32],
    block: &[Parameter],
    scoring: &RowsKScoring,
) -> Result<Option<u64>> {
    let scores = device.score_rows_k(batch, targets, block, scoring)?;
    Ok(scores
        .iter()
        .find(|s| s.status == 0)
        .and_then(|s| s.losses.first().copied()))
}

/// Forced sequential single applies of `members`, each proposed as a K = 1
/// `propose_block` on an independent fixture device restored at the
/// pre-state (so each proposal is stamped with the pre-state epoch),
/// committed by `apply_block` and overlaid on the pre-state. The guarded
/// ordinary single-move commit path is not used anywhere in this module. Members
/// commit disjoint storage, so the overlay is the sequential composition; a
/// storage word written by two members is a failure. Returns the composed
/// snapshot (epoch advanced once) and the summed single-apply epoch advance.
fn composite(
    pre: &Snapshot,
    windows: &[Vec<Identity>],
    targets: &[u32],
    members: &[Parameter],
    tally: &mut Tally,
) -> Result<(Snapshot, u64)> {
    fn overlay<T: PartialEq + Copy>(
        out: &mut [T],
        pre: &[T],
        after: &[T],
        touched: &mut [bool],
    ) -> Result<()> {
        require(after.len() == pre.len(), "V3: state length changed")?;
        for i in 0..pre.len() {
            if after[i] != pre[i] {
                require(!touched[i], "V3: two members wrote the same storage word")?;
                touched[i] = true;
                out[i] = after[i];
            }
        }
        Ok(())
    }
    let mut out = pre.clone();
    let mut touched = (
        vec![false; pre.table.len()],
        vec![false; pre.routes.len()],
        vec![false; pre.gates.len()],
    );
    let mut advance = 0u64;
    for &parameter in members {
        // Reference: the member alone, as a K = 1 block on its own fixture
        // device restored from the pre-state, committed through `apply_block`.
        // The guarded single-move commit path is never called here.
        let mut single = Device::from_snapshot(pre.clone(), 0)?;
        let single_rowsk = single.rowsk_scoring()?;
        let single_bvgk = single.bvgk_kernels()?;
        let input = single.carry_windows(windows)?;
        let proposal = single.propose_block(
            &input,
            targets,
            &[parameter],
            &[true],
            &single_rowsk,
            &single_bvgk,
        )?;
        let before = single.epoch()?;
        let committed = single.apply_block(&proposal)?.moves == 1;
        tally.applied_updates += u64::from(committed);
        require(
            committed,
            &format!("V3: the single apply of S_t member {parameter:?} did not commit"),
        )?;
        let after = single.snapshot()?;
        advance += after.epoch - before;
        overlay(&mut out.table, &pre.table, &after.table, &mut touched.0)?;
        overlay(&mut out.routes, &pre.routes, &after.routes, &mut touched.1)?;
        overlay(&mut out.gates, &pre.gates, &after.gates, &mut touched.2)?;
    }
    out.epoch = pre
        .epoch
        .checked_add(1)
        .ok_or_else(|| Error::Invalid("epoch overflow".into()))?;
    Ok((out, advance))
}

/// One fixture model with an independent reference restored from its snapshot.
struct Fixture<'a> {
    device: &'a Device,
    rowsk: &'a RowsKScoring,
    bvgk: &'a BvgkKernels,
    carried: &'a CarriedBatch,
    reference: &'a Device,
    reference_rowsk: &'a RowsKScoring,
    reference_bvgk: &'a BvgkKernels,
    reference_carried: &'a CarriedBatch,
    pre: &'a Snapshot,
    windows: &'a [Vec<Identity>],
    targets: &'a [u32],
}

fn all_refused(error: &Error) -> bool {
    matches!(error, Error::Invalid(text) if text.contains("every coordinate was refused"))
}

/// UT, JR, DC (V4, V5) and V3 for one block.
fn check_block(f: &Fixture, block: &[Parameter], eligible: &[bool], t: &mut Tally) -> Result<()> {
    t.cases += 1;
    // The device's own per-coordinate rule: a K=1 block on the reference holds
    // `|S_t| = 1` iff that coordinate is eligible and its move strictly improves.
    let mut members: Vec<(u32, u64)> = Vec::new();
    for (c, (&parameter, &e)) in block.iter().zip(eligible).enumerate() {
        match f.reference.propose_block(
            f.reference_carried,
            f.targets,
            &[parameter],
            &[e],
            f.reference_rowsk,
            f.reference_bvgk,
        ) {
            Ok(single) if single.record().s_len == 1 => {
                members.push((c as u32, single.record().l_best))
            }
            Ok(_) => {}
            Err(error) if all_refused(&error) => {}
            Err(error) => return Err(error),
        }
    }
    // Independent of the shared choose rule: membership is checked against
    // the raw C1 lane losses and the eligibility mask. A member must be
    // eligible, scored (status 0) and have a lane strictly below lane 0; an
    // ineligible coordinate is never a member.
    let lanes =
        f.reference
            .score_rows_k(f.reference_carried, f.targets, block, f.reference_rowsk)?;
    require(
        lanes.len() == block.len(),
        "DC: one C1 score per coordinate",
    )?;
    for &(c, _) in &members {
        let score = &lanes[c as usize];
        require(
            eligible[c as usize],
            "DC: an ineligible coordinate is a member",
        )?;
        require(score.status == 0, "DC: a member has a non-zero C1 status")?;
        let current = score.losses.first().copied().unwrap_or(u64::MAX);
        require(
            score.losses.iter().skip(1).any(|&loss| loss < current),
            "DC: a member has no lane below its current loss",
        )?;
    }
    let proposal = match unchanged(f.device, || {
        f.device
            .propose_block(f.carried, f.targets, block, eligible, f.rowsk, f.bvgk)
    }) {
        Ok(proposal) => proposal,
        Err(error) if all_refused(&error) => {
            require(members.is_empty(), "DC: a refused block has members")?;
            t.refused_blocks += 1;
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    t.proposals += 1;
    let r: &BlockRecord = proposal.record();
    let fail = |why: &str| Error::Invalid(format!("DC: {why}: {r:?} members {members:?}"));
    let l_before = current_loss(f.device, f.carried, f.targets, block, f.rowsk)?
        .ok_or_else(|| fail("no scored coordinate for l_before"))?;
    if r.k as usize != block.len() || r.l_before != l_before || r.s_len as usize != members.len() {
        return Err(fail("k, l_before or |S_t| differs from the reference"));
    }
    if members.iter().any(|&(_, loss)| loss >= l_before) {
        return Err(fail("a member does not strictly improve"));
    }
    let best = members.iter().min_by_key(|&&(c, loss)| (loss, c)).copied();
    let member_ordinals: Vec<u32> = members.iter().map(|&(c, _)| c).collect();
    match (members.len(), best) {
        (0, _) => {
            t.empty += 1;
            if r.joint
                || r.l_joint.is_some()
                || !r.commit_ordinals.is_empty()
                || r.l_best != l_before
                || r.committed_loss != l_before
            {
                return Err(fail("empty S_t must commit nothing"));
            }
        }
        (1, Some((c, loss))) => {
            t.single += 1;
            if r.joint
                || r.l_joint.is_some()
                || r.commit_ordinals != [c]
                || r.l_best != loss
                || r.committed_loss != loss
            {
                return Err(fail("|S_t| = 1 must commit {best}"));
            }
        }
        (_, Some((c, loss))) => {
            t.multi += 1;
            let joint = r.l_joint.ok_or_else(|| fail("|S_t| >= 2 has no l_joint"))?;
            require(r.l_best == loss, &format!("DC: l_best differs: {r:?}"))?;
            if joint < loss {
                t.joint_wins += 1;
                // V4: on a joint commit the commit mask is the member mask.
                if !r.joint || r.commit_ordinals != member_ordinals || r.committed_loss != joint {
                    return Err(fail("joint wins: S_t must be committed"));
                }
            } else {
                t.joint_loses_or_ties += 1;
                if r.joint || r.commit_ordinals != [c] || r.committed_loss != loss {
                    return Err(fail("joint loses or ties: {best} must be committed"));
                }
            }
            // JR: S_t composed from single applies, re-carried, lane 0 of [best].
            let params: Vec<Parameter> = members.iter().map(|&(m, _)| block[m as usize]).collect();
            let (composed, _) = composite(f.pre, f.windows, f.targets, &params, t)?;
            let scorer = Device::from_snapshot(composed, 0)?;
            let scoring = scorer.rowsk_scoring()?;
            let carried = scorer.carry_windows(f.windows)?;
            let score = scorer.score_rows_k(&carried, f.targets, &[block[c as usize]], &scoring)?;
            let lane0 = score
                .first()
                .filter(|s| s.status == 0)
                .and_then(|s| s.losses.first().copied());
            require(
                lane0 == Some(joint),
                &format!("JR: l_joint {joint} but the composed reference gives {lane0:?}"),
            )?;
        }
        _ => return Err(fail("internal: members without best")),
    }
    // V5 and V3 on an independent device restored at the pre-state.
    let mut applied = Device::from_snapshot(f.pre.clone(), 0)?;
    let rowsk = applied.rowsk_scoring()?;
    let bvgk = applied.bvgk_kernels()?;
    let input = applied.carry_windows(f.windows)?;
    let again = applied.propose_block(&input, f.targets, block, eligible, &rowsk, &bvgk)?;
    require(
        again.record() == r,
        "DC: an independent device restored from the snapshot decided differently",
    )?;
    let commit = applied.apply_block(&again)?;
    let post = applied.snapshot()?;
    if r.commit_ordinals.is_empty() {
        if commit.moves != 0 || !same_state(&post, f.pre) || post.epoch != f.pre.epoch {
            return Err(fail("an empty commit changed state or epoch"));
        }
        return Ok(());
    }
    t.commits += 1;
    t.applied_updates += 1;
    if commit.moves as usize != r.commit_ordinals.len()
        || commit.joint != r.joint
        || post.epoch != f.pre.epoch + 1
    {
        return Err(fail("BlockCommit or epoch differs from the record"));
    }
    let carried = applied.carry_windows(f.windows)?;
    let fresh = current_loss(&applied, &carried, f.targets, block, &rowsk)?;
    if fresh != Some(r.committed_loss) || r.committed_loss >= l_before {
        return Err(fail(&format!(
            "V5: fresh committed loss {fresh:?} vs record, or no strict improvement"
        )));
    }
    let committed: Vec<Parameter> = r
        .commit_ordinals
        .iter()
        .map(|&c| block[c as usize])
        .collect();
    let (sequential, advance) = composite(f.pre, f.windows, f.targets, &committed, t)?;
    t.v3_cases += 1;
    t.v3_joint += u64::from(r.joint);
    require(
        same_state(&post, &sequential),
        &format!("V3: block apply state differs from sequential single applies ({r:?})"),
    )?;
    require(
        advance == committed.len() as u64,
        &format!(
            "V3: sequential epoch advance {advance} is not |S| = {}",
            committed.len()
        ),
    )
}

/// The UT/JR/DC/V3 sweep: both kinds, K in {2,4,16}, rows {7,64}, seeds 7
/// and 11, the four C1 cells (objectives A and R), three blocks per K with two
/// eligibility patterns.
fn block_sweep(t: &mut Tally) -> Result<serde_json::Value> {
    for (read, blocks, objective) in CELLS {
        for seed in [7u32, 11] {
            let device = Device::new_with_objective(spec(blocks), read, objective, seed, 0)?;
            let rowsk = device.rowsk_scoring()?;
            let bvgk = device.bvgk_kernels()?;
            let pre = device.snapshot()?;
            let reference = Device::from_snapshot(pre.clone(), 0)?;
            let reference_rowsk = reference.rowsk_scoring()?;
            let reference_bvgk = reference.bvgk_kernels()?;
            let mut rng = Lcg(0xb7c ^ u64::from(seed) ^ u64::from(blocks));
            for row_count in [7usize, 64] {
                let batch = windows(&mut rng, row_count, CONTEXT, VOCABULARY);
                let targets: Vec<u32> = (0..row_count).map(|_| rng.next(VOCABULARY)).collect();
                let carried = device.carry_windows(&batch)?;
                let reference_carried = reference.carry_windows(&batch)?;
                let fixture = Fixture {
                    device: &device,
                    rowsk: &rowsk,
                    bvgk: &bvgk,
                    carried: &carried,
                    reference: &reference,
                    reference_rowsk: &reference_rowsk,
                    reference_bvgk: &reference_bvgk,
                    reference_carried: &reference_carried,
                    pre: &pre,
                    windows: &batch,
                    targets: &targets,
                };
                for inventory in inventories(read, &pre, blocks) {
                    for k in [2usize, 4, 16] {
                        // Every chunk of the inventory, under three eligibility
                        // patterns: all eligible, every third withheld, and a
                        // single eligible coordinate.
                        for (n, block) in inventory.chunks(k).enumerate() {
                            let eligible: Vec<bool> = (0..block.len())
                                .map(|c| match n % 3 {
                                    0 => true,
                                    1 => c % 3 != 1,
                                    _ => c == 0,
                                })
                                .collect();
                            check_block(&fixture, block, &eligible, t)?;
                        }
                    }
                }
            }
            let after = reference.snapshot()?;
            require(
                same_state(&after, &pre) && after.epoch == pre.epoch,
                "UT: the reference device changed under K=1 proposals",
            )?;
        }
    }
    let unseen: Vec<&str> = [
        ("empty", t.empty),
        ("single", t.single),
        ("joint-wins", t.joint_wins),
        ("joint-loses-or-ties", t.joint_loses_or_ties),
    ]
    .iter()
    .filter(|(_, n)| *n == 0)
    .map(|(name, _)| *name)
    .collect();
    require(
        unseen.is_empty(),
        &format!(
            "DC: UNDERPOWERED: never observed {unseen:?} (instrument blocker, not a negative)"
        ),
    )?;
    require(
        t.v3_joint > 0,
        "V3: UNDERPOWERED: no joint commit was compared (instrument blocker, not a negative)",
    )?;
    Ok(serde_json::json!({}))
}

/// RC: an independent device restored from the snapshot re-carries the same
/// windows to the pre-state rows-K losses on every lane, for batches built by
/// `carry` and by `carry_windows`.
fn recarry_parity(t: &mut Tally) -> Result<serde_json::Value> {
    for (read, blocks, objective) in CELLS {
        for seed in [7u32, 11] {
            let device = Device::new_with_objective(spec(blocks), read, objective, seed, 0)?;
            let rowsk = device.rowsk_scoring()?;
            let pre = device.snapshot()?;
            let scratch = Device::from_snapshot(pre.clone(), 0)?;
            let scratch_rowsk = scratch.rowsk_scoring()?;
            let mut rng = Lcg(0x2ec ^ u64::from(seed) ^ u64::from(blocks));
            for row_count in [7usize, 64] {
                let ragged = windows(&mut rng, row_count, CONTEXT, VOCABULARY);
                let full: Vec<Vec<Identity>> = (0..row_count as u64)
                    .map(|r| {
                        (0..u64::from(CONTEXT))
                            .map(|p| Identity {
                                token: rng.next(VOCABULARY),
                                occurrence: r * u64::from(CONTEXT) + p + 1,
                                record: 17,
                                version: 9,
                            })
                            .collect()
                    })
                    .collect();
                let flat: Vec<Identity> = full.iter().flatten().copied().collect();
                let targets: Vec<u32> = (0..row_count).map(|_| rng.next(VOCABULARY)).collect();
                for (label, original, batch) in [
                    ("carry_windows", device.carry_windows(&ragged)?, &ragged),
                    ("carry", device.carry(&flat)?, &full),
                ] {
                    let recarried = scratch.carry_windows(batch)?;
                    for inventory in inventories(read, &pre, blocks) {
                        let block = &inventory[..inventory.len().min(16)];
                        let a = unchanged(&device, || {
                            device.score_rows_k(&original, &targets, block, &rowsk)
                        })?;
                        let b =
                            scratch.score_rows_k(&recarried, &targets, block, &scratch_rowsk)?;
                        t.cases += 1;
                        require(a.len() == b.len(), "RC: score counts differ")?;
                        for (x, y) in a.iter().zip(&b) {
                            require(
                                (x.status == 0) == (y.status == 0),
                                &format!("RC/{label}: {:?} refusal class differs", x.parameter),
                            )?;
                            if x.status == 0 {
                                require(
                                    x.losses == y.losses,
                                    &format!(
                                        "RC/{label}: {:?} lanes differ: {:?} vs {:?}",
                                        x.parameter, x.losses, y.losses
                                    ),
                                )?;
                                t.lanes += x.losses.len() as u64;
                            } else if x.status != y.status {
                                // Lane terminals publish through atomicExch: a racy code.
                                t.racy_status += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    require(t.lanes > 0, "RC: no lane was compared")?;
    Ok(serde_json::json!({}))
}

/// RF: each refusal with its own reason, state unchanged.
fn refusals(t: &mut Tally) -> Result<serde_json::Value> {
    let mut device = Device::new_with_objective(
        spec(2),
        ReadSpec::Route,
        ObjectivePolicy::NearestAddressMarginV1,
        7,
        0,
    )?;
    let rowsk = device.rowsk_scoring()?;
    let bvgk = device.bvgk_kernels()?;
    let mut rng = Lcg(0xbad);
    let batch = windows(&mut rng, 7, CONTEXT, VOCABULARY);
    let targets: Vec<u32> = (0..7).map(|_| rng.next(VOCABULARY)).collect();
    let input = device.carry_windows(&batch)?;
    let other = Device::from_snapshot(device.snapshot()?, 0)?;
    let other_bvgk = other.bvgk_kernels()?;
    let expect = |label: &str, outcome: Result<()>, expected: &str| -> Result<()> {
        match outcome {
            Err(e @ Error::Cuda(_)) => Err(e),
            Err(error) => require(
                error.to_string().contains(expected),
                &format!("RF: {label} refused for the wrong reason: {error}"),
            ),
            Ok(()) => Err(Error::Invalid(format!("RF: {label} was not refused"))),
        }
    };
    let over: Vec<Parameter> = (0..65u32).map(|i| Parameter::VectorBlock(i % 32)).collect();
    let shape_cases: [(&str, Vec<Parameter>, Vec<bool>, &str); 5] = [
        (
            "K=0",
            Vec::new(),
            Vec::new(),
            "coordinate block must hold 1..=64",
        ),
        (
            "K=65",
            over,
            vec![true; 65],
            "coordinate block must hold 1..=64",
        ),
        (
            "mixed-kind",
            vec![Parameter::Route(0), Parameter::VectorBlock(0)],
            vec![true; 2],
            "must hold one parameter kind",
        ),
        (
            "eligibility-length",
            vec![Parameter::VectorBlock(0), Parameter::VectorBlock(1)],
            vec![true],
            "exactly one eligibility flag per coordinate",
        ),
        (
            "repeated-parameter",
            vec![Parameter::VectorBlock(0), Parameter::VectorBlock(0)],
            vec![true; 2],
            "must not repeat a parameter",
        ),
    ];
    for (label, block, eligible, expected) in shape_cases {
        t.cases += 1;
        let outcome = unchanged(&device, || {
            device
                .propose_block(&input, &targets, &block, &eligible, &rowsk, &bvgk)
                .map(|_| ())
        });
        expect(label, outcome, expected)?;
        t.refusals += 1;
    }
    t.cases += 1;
    let block: Vec<Parameter> = (0..4).map(Parameter::VectorBlock).collect();
    let outcome = unchanged(&device, || {
        device
            .propose_block(&input, &targets, &block, &[true; 4], &rowsk, &other_bvgk)
            .map(|_| ())
    });
    expect(
        "foreign-handle",
        outcome,
        "handle belongs to another model/stream",
    )?;
    t.refusals += 1;
    // A block that commits, for the apply refusals.
    let committing = (0..32u32)
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|c| {
            c.iter()
                .copied()
                .map(Parameter::VectorBlock)
                .collect::<Vec<_>>()
        })
        .find(|block| {
            device
                .propose_block(&input, &targets, block, &[true; 4], &rowsk, &bvgk)
                .is_ok_and(|p| !p.record().commit_ordinals.is_empty())
        })
        .ok_or_else(|| Error::Invalid("RF: UNDERPOWERED: no committing block".into()))?;
    let eligible = [true; 4];
    let first = device.propose_block(&input, &targets, &committing, &eligible, &rowsk, &bvgk)?;
    let second = device.propose_block(&input, &targets, &committing, &eligible, &rowsk, &bvgk)?;
    let foreign = device.propose_block(&input, &targets, &committing, &eligible, &rowsk, &bvgk)?;
    // Different device: restored from the same snapshot, a different binding.
    let mut other = other;
    let before = other.snapshot()?;
    let outcome = other.apply_block(&foreign).map(|_| ());
    let after = other.snapshot()?;
    t.cases += 1;
    require(
        same_state(&before, &after) && before.epoch == after.epoch,
        "RF: a foreign-device apply changed state",
    )?;
    expect(
        "foreign-device",
        outcome,
        "proposal belongs to another model/stream",
    )?;
    t.refusals += 1;
    // Commit `first`, then `second` is stale and `first` repeated is stale.
    device.apply_block(&first)?;
    t.applied_updates += 1;
    for (label, proposal) in [("stale", &second), ("repeated", &first)] {
        t.cases += 1;
        let before = device.snapshot()?;
        let outcome = device.apply_block(proposal).map(|_| ());
        let after = device.snapshot()?;
        require(
            same_state(&before, &after) && before.epoch == after.epoch,
            &format!("RF: a {label} apply changed state"),
        )?;
        expect(label, outcome, "stale at the current epoch")?;
        t.refusals += 1;
    }
    Ok(serde_json::json!({}))
}

/// C1R: the unchanged C1 V2 sweep against the split scoring.
fn c1_regression(t: &mut Tally) -> Result<serde_json::Value> {
    let [blocks, cases, successes, refusals, racy] =
        crate::rowsk_qualification::v2_scoring_parity_counts()?;
    t.cases += cases;
    t.refusals += refusals;
    Ok(serde_json::json!({
        "v2_blocks": blocks, "v2_successes": successes, "v2_racy_refusals": racy,
    }))
}

/// TP at the registered K16 redirect shape (diagnostic, not gated).
fn throughput() -> Result<serde_json::Value> {
    let spec = Spec {
        vocabulary: 4096,
        blocks: 4,
        context: 192,
        kept_cells: 16,
    };
    let device = Device::new_with_objective(
        spec,
        ReadSpec::Route,
        ObjectivePolicy::NearestAddressMarginV1,
        1,
        0,
    )?;
    let rows = device.rows_counterfactual()?;
    let rowsk = device.rowsk_scoring()?;
    let bvgk = device.bvgk_kernels()?;
    let mut rng = Lcg(0x7193);
    let batch: Vec<Vec<Identity>> = (0..64u64)
        .map(|r| {
            (0..192u64)
                .map(|p| Identity {
                    token: rng.next(4096),
                    occurrence: r * 192 + p + 1,
                    record: 17,
                    version: 9,
                })
                .collect()
        })
        .collect();
    let targets: Vec<u32> = (0..64).map(|_| rng.next(4096)).collect();
    let input = device.carry_windows(&batch)?;
    const TIMED: usize = 20;
    let sample = |mut f: Box<dyn FnMut() -> Result<()> + '_>| -> Result<(u128, u128, Vec<u64>)> {
        f()?; // excluded warmup
        let mut times = Vec::with_capacity(TIMED);
        for _ in 0..TIMED {
            let started = std::time::Instant::now();
            f()?;
            times.push(started.elapsed().as_nanos());
        }
        let raw: Vec<u64> = times.iter().map(|&t| t as u64).collect();
        times.sort_unstable();
        let p95_index = (95 * TIMED).div_ceil(100) - 1;
        Ok((times[TIMED / 2], times[p95_index], raw))
    };
    let (single_ns, single_p95, single_raw) = sample(Box::new(|| {
        device
            .propose_rows(&input, &targets, Parameter::VectorBlock(0), &rows)
            .map(|_| ())
    }))?;
    let mut cases = Vec::new();
    for k in [1u32, 4, 16] {
        let block: Vec<Parameter> = (0..k).map(Parameter::VectorBlock).collect();
        let eligible = vec![true; k as usize];
        let probe = unchanged(&device, || {
            device.propose_block(&input, &targets, &block, &eligible, &rowsk, &bvgk)
        })?;
        let record = probe.record().clone();
        let (ns, p95, raw) = sample(Box::new(|| {
            device
                .propose_block(&input, &targets, &block, &eligible, &rowsk, &bvgk)
                .map(|_| ())
        }))?;
        cases.push(serde_json::json!({
            "k": k, "block_median_ns": ns as u64, "block_p95_ns": p95 as u64,
            "raw_samples_ns": raw, "ns_per_coordinate": (ns / u128::from(k)) as u64,
            "s_len": record.s_len, "joint_verify_ran": record.s_len >= 2,
            "joint_committed": record.joint,
        }));
    }
    Ok(serde_json::json!({
        "shape": "V4096/B4/C192/K16 Route objective A, seed 1 checkpoint 0, 64 rows, VectorBlock 0..K, all eligible, 1 warmup + 20 timed, median and nearest-rank p95",
        "single_propose_rows_median_ns": single_ns as u64,
        "single_propose_rows_p95_ns": single_p95 as u64,
        "single_propose_rows_raw_samples_ns": single_raw,
        "scope": "host-callable Device::propose_block (rows-K scoring, choose, scratch clone + re-carry + joint score when |S_t| >= 2, decide, readback) vs Device::propose_rows",
        "qualified": false,
        "cases": cases,
    }))
}

fn tally_json(t: &Tally) -> serde_json::Value {
    serde_json::json!({
        "cases": t.cases, "proposals": t.proposals, "multi_member_proposals": t.multi,
        "empty": t.empty, "single": t.single, "joint_wins": t.joint_wins,
        "joint_loses_or_ties": t.joint_loses_or_ties, "refused_blocks": t.refused_blocks,
        "commits": t.commits, "v3_cases": t.v3_cases, "v3_joint_cases": t.v3_joint,
        "lanes": t.lanes, "racy_status": t.racy_status, "refusals": t.refusals,
        "applied_updates": t.applied_updates,
    })
}

pub fn run(report: &Path) -> Result<()> {
    std::fs::create_dir(report).map_err(|e| Error::Invalid(format!("exclusive report: {e}")))?;
    let mut records = Vec::new();
    let mut failed = false;
    let mut applied_updates = 0u64;
    let v0 =
        crate::device::bvgk_extends_rowsk_source() && crate::device::rowsk_extends_rows_source();
    failed |= !v0;
    let record = serde_json::json!({
        "name": "v0_source_identity", "status": if v0 { "PASS" } else { "FAIL" },
        "bvgk_extends_rowsk_source": crate::device::bvgk_extends_rowsk_source(),
        "rowsk_extends_rows_source": crate::device::rowsk_extends_rows_source(),
    });
    println!("{record}");
    records.push(record);
    let controls: [(&str, Control); 4] = [
        ("rc_recarry_parity", recarry_parity),
        ("ut_jr_dc_v3_block_sweep", block_sweep),
        ("rf_refusals", refusals),
        ("c1r_v2_scoring_parity", c1_regression),
    ];
    for (name, control) in controls {
        let mut tally = Tally::default();
        let outcome = if v0 {
            control(&mut tally)
        } else {
            Err(Error::Invalid("NOT_RUN: V0 failed".into()))
        };
        applied_updates += tally.applied_updates;
        let mut record = tally_json(&tally);
        record["name"] = serde_json::json!(name);
        match outcome {
            Ok(extra) => {
                record["status"] = serde_json::json!("PASS");
                record["detail"] = extra;
            }
            Err(error) => {
                failed = true;
                record["status"] = serde_json::json!("FAIL");
                record["error"] = serde_json::json!(error.to_string());
            }
        }
        // The sweep carries four named controls; attribute a failure by tag.
        if name == "ut_jr_dc_v3_block_sweep" {
            let error = record["error"].as_str().unwrap_or("").to_string();
            let tags = [
                ("ut_original_untouched", "UT:"),
                ("jr_joint_reference", "JR:"),
                ("dc_decision_coverage_v4_v5", "DC:"),
                ("v3_block_equals_sequential", "V3:"),
            ];
            let attributed = tags.iter().any(|(_, tag)| error.contains(tag));
            record["controls"] = tags
                .iter()
                .map(|(control, tag)| {
                    let status = if error.is_empty() {
                        "PASS"
                    } else if error.contains(tag) || (!attributed && *tag == "DC:") {
                        "FAIL"
                    } else {
                        "NOT_COMPLETE: the sweep stopped at another control"
                    };
                    serde_json::json!({"name": control, "status": status})
                })
                .collect();
        }
        println!("{record}");
        records.push(record);
    }
    let tp = if failed {
        serde_json::json!("NOT_RUN: a control failed")
    } else {
        throughput().unwrap_or_else(
            |error| serde_json::json!({"status": "DIAGNOSTIC_ERROR", "error": error.to_string()}),
        )
    };
    println!("{tp}");
    let payload = serde_json::json!({
        "schema": crate::SCHEMA,
        "suite": "bvgk-block-qualification",
        "status": if failed { "FAILED" } else { "PASS" },
        "controls": records,
        "throughput": tp,
        "bvgk_image_sha256": crate::device::compiled_bvgk_image_sha256(),
        "rowsk_image_sha256": crate::device::compiled_rowsk_image_sha256(),
        "rows_image_sha256": crate::device::compiled_rows_image_sha256(),
        "driver_source_sha256": crate::training::driver_source_sha256(),
        "training": "HELD",
        "applied_updates": applied_updates,
        "applied_updates_scope": "fixture devices only: block commits and the single applies of the V3/JR references; no corpus, checkpoint or artifact is written",
    });
    std::fs::write(
        report.join("bvgk-qualification.json"),
        serde_json::to_vec_pretty(&payload).map_err(|e| Error::Invalid(e.to_string()))?,
    )
    .map_err(|e| Error::Invalid(e.to_string()))?;
    require(!failed, "one or more BVG-K block controls failed")
}
