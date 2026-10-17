//! Exact finite driver construction. One isolated authored update is admitted
//! only by the successor finite sheet; no corpus learning or optimizer runs.
use crate::{
    device::{Device, Proposal, Snapshot},
    Error, Identity, Parameter, ReadSpec, Result, Spec,
};
use serde_json::{json, Value};
use std::path::Path;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}
fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}
fn expect_invalid<T>(outcome: Result<T>, expected: &str, label: &str) -> Result<()> {
    match outcome {
        Err(Error::Invalid(message)) if message == expected => Ok(()),
        Err(error @ Error::Cuda(_)) => Err(error),
        Err(error) => Err(invalid(format!(
            "{label}: expected Invalid({expected:?}), got {error}"
        ))),
        Ok(_) => Err(invalid(format!(
            "{label}: expected refusal, operation succeeded"
        ))),
    }
}
fn id(token: u32, occurrence: u64) -> Identity {
    Identity {
        token,
        occurrence,
        record: 17,
        version: 9,
    }
}
fn literal_fixture(vocabulary: u32) -> Result<Snapshot> {
    let model = Device::new(
        Spec {
            vocabulary,
            blocks: 2,
            context: 1,
            kept_cells: 2.min(vocabulary),
        },
        7,
        0,
    )?;
    let mut snapshot = model.snapshot()?;
    for token in 0..vocabulary as usize {
        let root = if token == 1 {
            [-2, -2, 0, 0, 0, 0, 0, 0]
        } else {
            [2, 2, 0, 0, 0, 0, 0, 0]
        };
        for block in 0..2 {
            snapshot.table[(token * 2 + block) * 8..(token * 2 + block + 1) * 8]
                .copy_from_slice(&root);
        }
    }
    Ok(snapshot)
}

struct Context<'a> {
    report: &'a Path,
    applied_updates: usize,
    apply_calls: usize,
    timings: Vec<Value>,
}

fn apply(context: &mut Context<'_>, model: &mut Device, proposal: &Proposal) -> Result<bool> {
    if context.apply_calls >= 11 {
        return Err(invalid("finite apply call bound11 exceeded"));
    }
    context.apply_calls += 1;
    let applied = model.apply(proposal)?;
    context.applied_updates += usize::from(applied);
    Ok(applied)
}

fn isolated_apply_boundary(context: &mut Context<'_>) -> Result<()> {
    let snapshot = literal_fixture(2)?;
    let mut model = Device::from_snapshot(snapshot.clone(), 0)?;
    let mut foreign = Device::from_snapshot(snapshot, 0)?;
    let foreign_before =
        serde_json::to_value(foreign.snapshot()?).map_err(|e| invalid(e.to_string()))?;
    let input = model.carry(&[id(1, 0)])?;
    let before = model.snapshot()?;
    let proposal = model.propose(&input, &[0], Parameter::VectorBlock(0))?;
    let selected = proposal.observation()?;
    require(
        selected.candidate == 7
            && selected.before == 640
            && selected.after == 544
            && selected.epoch == 0
            && selected.any_candidate_cost_differs_from_identity,
        "isolated update proposal differs from pinned640->544/ordinal7",
    )?;
    // One additional read-only proposal. The query/target is token0; token1
    // starts at the opposite root in both blocks, so CURRENT has zero cost.
    // Moving its first block can reduce separation and increase the hinge.
    // CURRENT must stay selected, yet the GPU sensitivity flag must be true.
    // No host recomputation/ranking and no additional apply call occurs here.
    let at_target = model.carry(&[id(0, 0)])?;
    let worse_only = model
        .propose(&at_target, &[0], Parameter::VectorBlock(2))?
        .observation()?;
    require(
        worse_only.candidate == 0
            && worse_only.before == 0
            && worse_only.after == 0
            && worse_only.epoch == 0
            && worse_only.any_candidate_cost_differs_from_identity,
        "all-nonimproving proposal with worse candidates must retain CURRENT and report sensitivity",
    )?;
    expect_invalid(
        apply(context, &mut foreign, &proposal),
        "proposal belongs to another model/source/geometry",
        "foreign apply",
    )?;
    require(
        foreign.epoch()? == 0
            && foreign_before
                == serde_json::to_value(foreign.snapshot()?).map_err(|e| invalid(e.to_string()))?,
        "foreign refusal changed full snapshot",
    )?;
    model.synchronize()?;
    // Move only the receipt buffer aside so the timed call uses the same
    // bounded apply helper as every other fixture call, without aliasing it.
    let mut timings = std::mem::take(&mut context.timings);
    let observed = crate::measurement::observe(&mut timings, "tiny_fixture_apply", 1, || {
        let applied = apply(context, &mut model, &proposal)?;
        model.synchronize()?;
        Ok(applied)
    });
    context.timings = timings;
    let applied = observed?;
    require(
        applied && model.epoch()? == 1,
        "improving update did not advance exactly one epoch",
    )?;
    let after = model.snapshot()?;
    require(
        after.table[..8] == [-2, 2, 0, 0, 0, 0, 0, 0]
            && after.table[8..] == before.table[8..]
            && after.routes == before.routes
            && after.gates == before.gates,
        "apply changed something outside its declared codebook block",
    )?;
    expect_invalid(
        apply(context, &mut model, &proposal),
        "proposal is stale at the current model epoch",
        "stale apply",
    )?;
    expect_invalid(
        model.forward(&input, &[0]),
        "carried vectors are stale at the current model epoch",
        "stale carry",
    )?;
    model.synchronize()?;
    let recarried =
        crate::measurement::observe(&mut context.timings, "tiny_fixture_recarry", 1, || {
            let batch = model.carry(&[id(1, 0)])?;
            model.synchronize()?;
            Ok(batch)
        })?;
    require(
        model.forward(&recarried, &[0])?.losses == [544],
        "recarry did not observe committed objective544",
    )?;
    let noop = model.propose(&recarried, &[0], Parameter::Route(0))?;
    let noop_observed = noop.observation()?;
    require(
        noop_observed.candidate == 0
            && noop_observed.before == 544
            && noop_observed.after == 544
            && !noop_observed.any_candidate_cost_differs_from_identity,
        "one-leaf route no-improvement did not select CURRENT",
    )?;
    let losses = noop.candidate_losses()?;
    require(
        losses.len() == 16 && losses.iter().all(|&loss| loss == 544),
        "nonidentity equal-loss candidates were not actually scored",
    )?;
    require(
        !apply(context, &mut model, &noop)? && model.epoch()? == 1,
        "CURRENT proposal must return false without epoch update",
    )?;
    require(
        serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?
            == serde_json::to_value(&after).map_err(|e| invalid(e.to_string()))?,
        "no-op changed snapshot",
    )?;
    Ok(())
}

fn route_apply_reality(context: &mut Context<'_>) -> Result<()> {
    let mut snapshot = literal_fixture(2)?;
    snapshot.spec.context = 2;
    let mut model = Device::from_snapshot(snapshot, 0)?;
    let input = model.carry(&[id(1, 0), id(1, 1)])?;
    let before = model.snapshot()?;
    let proposal = model.propose(&input, &[1], Parameter::Route(0))?;
    let observed = proposal.observation()?;
    require(
        observed.candidate == 1
            && observed.before == 640
            && observed.after == 256
            && observed.any_candidate_cost_differs_from_identity,
        "Route B12 proposal must be frame1/640->256",
    )?;
    require(
        apply(context, &mut model, &proposal)?,
        "Route improvement did not apply",
    )?;
    let after = model.snapshot()?;
    require(
        after.routes == [1, 0]
            && after.table == before.table
            && after.gates == before.gates
            && after.epoch == 1,
        "Route apply did not compose only the selected frame",
    )?;
    let fresh = model.carry(&[id(1, 0), id(1, 1)])?;
    let forward = model.forward(&fresh, &[1])?;
    require(
        forward.losses == [256]
            && forward.vectors()? == [-2, -2, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0],
        "Route replay differs from applied reality",
    )
}

fn gate_apply_reality(context: &mut Context<'_>) -> Result<()> {
    let mut snapshot = literal_fixture(2)?;
    snapshot.spec.context = 2;
    snapshot.read_spec = ReadSpec::Flock { kept: 1 };
    snapshot.table.copy_from_slice(&[
        2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0,
        0, 0,
    ]);
    let mut model = Device::from_snapshot(snapshot, 0)?;
    let before = model.snapshot()?;
    let input = model.carry(&[id(1, 0), id(0, 1)])?;
    let proposal = model.propose(&input, &[1], Parameter::Gate(1))?;
    let observed = proposal.observation()?;
    require(
        observed.candidate == 2
            && observed.before == 448
            && observed.after == 64
            && observed.any_candidate_cost_differs_from_identity,
        "Gate sibling+1 proposal differs",
    )?;
    require(
        apply(context, &mut model, &proposal)?,
        "Gate improvement did not apply",
    )?;
    let after = model.snapshot()?;
    require(
        after.gates == [0, 1]
            && after.table == before.table
            && after.routes == before.routes
            && after.epoch == 1,
        "Gate apply changed something outside its selected trit",
    )?;
    let fresh = model.carry(&[id(1, 0), id(0, 1)])?;
    let forward = model.forward(&fresh, &[1])?;
    require(
        forward.losses == [64]
            && forward.vectors()? == [2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0],
        "Gate replay differs from applied reality",
    )?;
    let evaluated = model.evaluate(&fresh, &[1])?;
    require(
        evaluated.aggregate.token_strict_win_eligible_rows == 1
            && evaluated.aggregate.token_unviolated_eligible_rows == 0
            && evaluated.aggregate.token_hinge_sum == 8
            && evaluated.aggregate.address_hinge_sum == 0
            && evaluated.aggregate.total_loss_numerator_sum == 64,
        "strict token win must not be conflated with zero margin",
    )
}

fn vector_step(
    context: &mut Context<'_>,
    model: &mut Device,
    index: u32,
    before: u64,
    after: u64,
) -> Result<Value> {
    let input = model.carry(&[id(1, 0)])?;
    let proposal = model.propose(&input, &[0], Parameter::VectorBlock(index))?;
    let selected = proposal.observation()?;
    require(
        selected.candidate == 7 && selected.before == before && selected.after == after,
        "two-step fixture candidate/cost changed",
    )?;
    require(
        apply(context, model, &proposal)?,
        "two-step strict improvement did not apply",
    )?;
    let fresh = model.carry(&[id(1, 0)])?;
    require(
        model.forward(&fresh, &[0])?.losses == [after],
        "two-step replay differs from reality",
    )?;
    Ok(
        json!({"index":index,"candidate":selected.candidate,"before":before,"after":after,"epoch":model.epoch()?}),
    )
}

fn fixture_loop_resume(context: &mut Context<'_>) -> Result<()> {
    // In-memory snapshots only. No UORT/corpus is passed to any apply and no
    // post-apply snapshot is saved, printed or archived.
    let initial = literal_fixture(2)?;
    let mut first = Device::from_snapshot(initial.clone(), 0)?;
    let first0 = vector_step(context, &mut first, 0, 640, 544)?;
    let mid = first.snapshot()?;
    let first1 = vector_step(context, &mut first, 1, 544, 448)?;
    let mut repeated = Device::from_snapshot(initial, 0)?;
    let repeated0 = vector_step(context, &mut repeated, 0, 640, 544)?;
    let repeated1 = vector_step(context, &mut repeated, 1, 544, 448)?;
    let mut resumed = Device::from_snapshot(mid, 0)?;
    let resumed1 = vector_step(context, &mut resumed, 1, 544, 448)?;
    require(
        first0 == repeated0 && first1 == repeated1 && first1 == resumed1,
        "fixture seed/epoch sequence or in-memory resume diverged",
    )?;
    let a = first.snapshot()?;
    let b = repeated.snapshot()?;
    let c = resumed.snapshot()?;
    require(
        a.table == b.table
            && a.table == c.table
            && a.routes == b.routes
            && a.routes == c.routes
            && a.gates == b.gates
            && a.gates == c.gates
            && a.epoch == 2
            && b.epoch == 2
            && c.epoch == 2,
        "deterministic fixture loops did not end in identical state",
    )
}

fn native_top8_ties(_: &mut Context<'_>) -> Result<()> {
    let mut snapshot = literal_fixture(10)?;
    snapshot.spec.kept_cells = 1;
    snapshot.table[16..32].copy_from_slice(&[2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0]);
    let model = Device::from_snapshot(snapshot, 0)?;
    let input = model.carry(&[id(0, 0), id(0, 1)])?;
    let result = model.evaluate(&input, &[7, 8])?;
    require(result.rows.len() == 2, "evaluation row count differs")?;
    for row in &result.rows {
        require(
            row.top8 == [0, 1, 2, 3, 4, 5, 6, 7]
                && row.token_hinge == 16
                && row.address_hinge == 0
                && row.total_loss_numerator == 128
                && row.token_rivals == 9
                && row.address_rivals == 0
                && row.admitted_cells == 1,
            "global writer top8/tie/crowded-cell units differ",
        )?;
    }
    require(
        result.rows[0].top1_correct == 0
            && result.rows[0].top8_correct == 1
            && result.rows[1].top1_correct == 0
            && result.rows[1].top8_correct == 0,
        "top8 boundary target7/8 is not discriminated",
    )?;
    let a = &result.aggregate;
    require(
        a.scored_rows == 2
            && a.top1_correct == 0
            && a.top8_correct == 1
            && a.token_hinge_sum == 32
            && a.address_hinge_sum == 0
            && a.total_loss_numerator_sum == 256
            && a.token_eligible_rows == 2
            && a.address_eligible_rows == 0
            && a.token_rival_count_sum == 18
            && a.address_rival_count_sum == 0
            && a.admitted_cell_count_sum == 2
            && a.eligible_any_rows == 2
            && a.eligible_unviolated_rows == 0
            && a.token_unviolated_eligible_rows == 0
            && a.address_unviolated_eligible_rows == 0
            && a.token_strict_win_eligible_rows == 0
            && a.loss_denominator_per_row == 128,
        "GPU aggregate differs from literal two-row metrics",
    )?;
    let singleton = Device::from_snapshot(literal_fixture(2)?, 0)?;
    let mut no_rival = singleton.snapshot()?;
    no_rival.spec.kept_cells = 1;
    let singleton = Device::from_snapshot(no_rival, 0)?;
    let row = singleton.carry(&[id(0, 0)])?;
    let zero = singleton.evaluate(&row, &[0])?;
    require(
        zero.aggregate.token_eligible_rows == 0
            && zero.aggregate.address_eligible_rows == 0
            && zero.aggregate.total_loss_numerator_sum == 0
            && zero.aggregate.eligible_any_rows == 0
            && zero.aggregate.eligible_unviolated_rows == 0
            && zero.aggregate.token_unviolated_eligible_rows == 0
            && zero.aggregate.address_unviolated_eligible_rows == 0,
        "absent-rival zero was labelled eligible learning",
    )?;
    expect_invalid(
        singleton.propose(&row, &[0], Parameter::VectorBlock(0)),
        "discrete CUDA counterfactual refused with status 8",
        "wholly ineligible proposal",
    )
}

fn corpus_fixture(
    split: crate::corpus::Split,
    tokens: &[u16],
    mask: &[u8],
    expected: u64,
) -> Result<crate::corpus::PreparedCorpus> {
    use crate::corpus::{sha256, BosIdentity, CorpusSource, FilePin, PreparedCorpus};
    let mut bytes = vec![0; 64];
    bytes[..4].copy_from_slice(b"UORT");
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
    bytes[16..20].copy_from_slice(&10u32.to_le_bytes());
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    let source = CorpusSource {
        split,
        tokens: FilePin {
            path: format!("authored-{split:?}.uort").into(),
            sha256: sha256(&bytes),
        },
        mask: FilePin {
            path: format!("authored-{split:?}.mask").into(),
            sha256: sha256(mask),
        },
        expected_payload_tokens: tokens.len() as u64,
        expected_eligible_targets: expected,
    };
    PreparedCorpus::from_bytes(
        source,
        &bytes,
        mask,
        10,
        BosIdentity {
            token: "<|bos|>".into(),
            id: 0,
        },
    )
}

fn count_reference_excludes_dev(_: &mut Context<'_>) -> Result<()> {
    // Response-mask selection is metadata; the frequencies, ranking and
    // evaluation arithmetic below are entirely GPU operations.
    use crate::corpus::Split;
    let mask = [0, 0, 0, 1, 1, 1, 0, 0, 1];
    let corpus = corpus_fixture(Split::Train, &[0, 9, 9, 3, 3, 4, 0, 9, 4], &mask, 4)?;
    let changed_prompt = corpus_fixture(Split::Train, &[0, 8, 8, 3, 3, 4, 0, 8, 4], &mask, 4)?;
    require(
        corpus.eligible_targets() == [3, 3, 4, 4]
            && changed_prompt.eligible_targets() == corpus.eligible_targets(),
        "unmasked prompt IDs entered TRAIN reference targets",
    )?;
    let observed = Device::count_reference(10, &corpus.eligible_targets(), &[9, 9, 9, 4], 0)?;
    let train = &observed.train_masked_reference;
    require(
        train.token_counts == [0, 0, 0, 2, 2, 0, 0, 0, 0, 0]
            && train.top8 == [3, 4, 0, 1, 2, 5, 6, 7]
            && train.scored_rows == 4
            && train.top1_correct == 0
            && train.top8_correct == 1
            && train.zero_count_targets == 3
            && train.source_targets == 4
            && train.nonzero_tokens == 2,
        "TRAIN count reference/top8 tie or DEV exclusion differs",
    )?;
    let dev = &observed.dev_response_oracle;
    require(
        observed.dev_oracle_diagnostic_only
            && dev.token_counts == [0, 0, 0, 0, 1, 0, 0, 0, 0, 3]
            && dev.top8 == [9, 4, 0, 1, 2, 3, 5, 6]
            && dev.scored_rows == 4
            && dev.top1_correct == 3
            && dev.top8_correct == 4
            && dev.zero_count_targets == 0
            && dev.source_targets == 4
            && dev.nonzero_tokens == 2,
        "DEV oracle is not independently labelled diagnostic",
    )?;
    let changed = Device::count_reference(10, &[3, 3, 4, 4], &[8, 8, 8, 4], 0)?;
    require(
        changed.train_masked_reference.token_counts == train.token_counts
            && changed.train_masked_reference.top8 == train.top8,
        "DEV perturbation contaminated TRAIN frequency reference",
    )
}

fn corpus_schedule_resume(context: &mut Context<'_>) -> Result<()> {
    use crate::corpus::{sha256, FilePin, Split};
    use crate::training::{
        CoordinateKind, Cursor, DriverConfig, PreparedDriver, SchedulePolicy, DRIVER_SCHEMA,
    };
    let mask = [0, 0, 0, 1, 1, 1, 0, 0, 1];
    let train = corpus_fixture(Split::Train, &[0, 9, 9, 3, 3, 4, 0, 9, 4], &mask, 4)?;
    let first = train.batch(0, 3, 3)?;
    let tail = train.batch(first.next_target, 3, 3)?;
    let words = |batch: &crate::corpus::CorpusBatch| {
        batch
            .windows
            .iter()
            .map(|w| w.iter().map(|i| i.token).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    };
    require(
        first.target_positions == [3, 4, 5]
            && first.targets == [3, 3, 4]
            && words(&first) == [vec![0, 9, 9], vec![9, 9, 3], vec![9, 3, 3]]
            && tail.target_positions == [8]
            && tail.targets == [4]
            && words(&tail) == [vec![0, 9]]
            && tail.windows[0][0].record == 1
            && tail.next_target == 4,
        "masked/BOS/tail windows differ from exact once-only fixture",
    )?;
    expect_invalid(
        corpus_fixture(Split::Train, &[0, 9], &[0, 0], 0),
        "eligible response targets differ from positive expected inventory",
        "all-masked source",
    )?;
    let tokenizer_bytes = br#"{"model":{"vocab":{"<|bos|>":0}}}"#;
    let tokenizer_path = context.report.join("fixture-tokenizer.json");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tokenizer_path)
        .map_err(|e| invalid(e.to_string()))?;
    use std::io::Write;
    (&file)
        .write_all(tokenizer_bytes)
        .map_err(|e| invalid(e.to_string()))?;
    let make_driver = |seed, read_spec| -> Result<PreparedDriver> {
        let train = corpus_fixture(Split::Train, &[0, 9, 9, 3, 3, 4, 0, 9, 4], &mask, 4)?;
        let dev = corpus_fixture(Split::Dev, &[0, 3, 4], &[0, 1, 1], 2)?;
        let config = DriverConfig {
            read_initialization: crate::ReadInitialization::ZeroGatesV1,
            objective_policy: crate::ObjectivePolicy::NearestAddressMarginV1,
            schema: DRIVER_SCHEMA.into(),
            spec: Spec {
                vocabulary: 10,
                blocks: 2,
                context: 3,
                kept_cells: 2,
            },
            read_spec,
            relation_learning_policy: crate::training::RelationLearningPolicy::LearnV1,
            seed,
            batch_rows: 3,
            train: train.binding().source.clone(),
            dev: dev.binding().source.clone(),
            tokenizer: FilePin {
                path: tokenizer_path.clone(),
                sha256: sha256(tokenizer_bytes),
            },
            bos: train.binding().bos.clone(),
            schedule: SchedulePolicy::CanonicalVectorThenReadV1,
        };
        PreparedDriver::from_prepared(config, train, dev)
    };
    let driver = make_driver(1, ReadSpec::Route)?;
    let paired = make_driver(2, ReadSpec::Route)?;
    let flock = make_driver(1, ReadSpec::Flock { kept: 1 })?;
    require(
        driver.coordinate_count() == 24 && flock.coordinate_count() == 24,
        "coordinate inventory must be24",
    )?;
    let mut cursor = Cursor::initial();
    for index in 0..24 {
        let coordinate = driver.coordinate(index)?;
        require(
            coordinate == paired.coordinate(index)?
                && coordinate.index == if index < 20 { index } else { index - 20 }
                && coordinate.kind
                    == if index < 20 {
                        CoordinateKind::VectorBlock
                    } else {
                        CoordinateKind::Route
                    },
            "canonical vector-then-read pass or paired seed order differs",
        )?;
        require(
            flock.coordinate(index)?.kind
                == if index < 20 {
                    CoordinateKind::VectorBlock
                } else {
                    CoordinateKind::Gate
                },
            "Flock schedule retained inactive Route parameters",
        )?;
        require(
            cursor.next_coordinate == index && driver.batch(&cursor)? == paired.batch(&cursor)?,
            "seed changed corpus order or cursor",
        )?;
        cursor = driver.advance(&cursor, false)?;
    }
    require(
        cursor.next_coordinate == 0
            && cursor.completed_batches == 24
            && cursor.accepted_updates == 0
            && cursor.corpus_pass == 12
            && cursor.next_target == 0,
        "full coordinate pass did not wrap exactly",
    )?;
    let mid = driver.advance(&Cursor::initial(), false)?;
    let resumed: Cursor =
        serde_json::from_slice(&serde_json::to_vec(&mid).map_err(|e| invalid(e.to_string()))?)
            .map_err(|e| invalid(e.to_string()))?;
    require(
        driver.batch(&resumed)? == tail
            && driver.advance(&resumed, false)? == driver.advance(&mid, false)?,
        "serialized metadata cursor lost exact tail/resume behavior",
    )?;
    let model = driver.new_model(0)?;
    let checkpoint = driver.checkpoint(&model, mid)?;
    driver.validate_checkpoint(&checkpoint)?;
    let (restored, restored_cursor) = driver.restore(checkpoint.clone(), 0)?;
    let original_batch = model.carry_windows(&tail.windows)?;
    let restored_batch = restored.carry_windows(&tail.windows)?;
    let original_evaluation = model.evaluate(&original_batch, &tail.targets)?;
    let restored_evaluation = restored.evaluate(&restored_batch, &tail.targets)?;
    require(
        restored_cursor == mid
            && restored.epoch()? == 0
            && serde_json::to_value(original_evaluation).map_err(|e| invalid(e.to_string()))?
                == serde_json::to_value(restored_evaluation).map_err(|e| invalid(e.to_string()))?,
        "actual epoch0 driver checkpoint/restore changed GPU observations",
    )?;
    let mut wrong = checkpoint.clone();
    wrong.lineage.config.seed = 2;
    expect_invalid(
        driver.validate_checkpoint(&wrong),
        "checkpoint source/config/data/mask/tokenizer/role-policy lineage mismatch",
        "changed seed lineage",
    )?;
    let mut wrong = checkpoint.clone();
    wrong.lineage.train.source.mask.sha256 = "a".repeat(64);
    expect_invalid(
        driver.validate_checkpoint(&wrong),
        "checkpoint source/config/data/mask/tokenizer/role-policy lineage mismatch",
        "changed mask lineage",
    )?;
    let mut wrong = checkpoint;
    wrong.cursor.next_coordinate = 0;
    expect_invalid(
        driver.validate_checkpoint(&wrong),
        "resume cursor differs from canonical corpus/coordinate cycle",
        "changed schedule cursor",
    )?;
    match crate::training::request_corpus_update_admission(&driver) {
        Err(Error::TrainingHeld) => {}
        Err(error) => return Err(error),
        Ok(_) => return Err(invalid("corpus update capability was issued under hold")),
    }
    match crate::training_admission() {
        Err(Error::TrainingHeld) => Ok(()),
        Err(error) => Err(error),
        Ok(()) => Err(invalid("training CLI admission released")),
    }
}

fn seeded_initialization(_: &mut Context<'_>) -> Result<()> {
    let spec = Spec {
        vocabulary: 10,
        blocks: 2,
        context: 3,
        kept_cells: 2,
    };
    let first = Device::new(spec, 1, 0)?;
    let same = Device::new(spec, 1, 0)?;
    let other = Device::new(spec, 2, 0)?;
    let a = first.snapshot()?;
    let b = same.snapshot()?;
    let c = other.snapshot()?;
    require(
        a.seed == 1
            && b.seed == 1
            && c.seed == 2
            && a.table == b.table
            && a.routes == b.routes
            && a.gates == b.gates
            && a.table != c.table
            && first.binding().model != same.binding().model,
        "paired seed initialization or distinct process identity differs",
    )?;
    let original = first.carry(&[id(0, 0), id(1, 1), id(2, 2)])?;
    expect_invalid(
        same.forward(&original, &[3]),
        "carried batch belongs to another model/source/geometry",
        "same-seed foreign carrier",
    )
}

pub const NAMES: &[&str] = &[
    "isolated_apply_boundary",
    "route_apply_reality",
    "gate_apply_reality",
    "fixture_loop_resume",
    "native_top8_ties",
    "count_reference_excludes_dev",
    "seeded_initialization",
    "corpus_schedule_resume",
];

/// Root will register this independent finite command. CUDA faults hard-stop
/// and preserve remaining NOT_RUN; assertion findings never become PASS.
pub fn run(report: &Path) -> Result<()> {
    std::fs::create_dir(report)
        .map_err(|e| invalid(format!("exclusive driver qualification: {e}")))?;
    let mut context = Context {
        report,
        applied_updates: 0,
        apply_calls: 0,
        timings: Vec::new(),
    };
    let controls: &[fn(&mut Context<'_>) -> Result<()>] = &[
        isolated_apply_boundary,
        route_apply_reality,
        gate_apply_reality,
        fixture_loop_resume,
        native_top8_ties,
        count_reference_excludes_dev,
        seeded_initialization,
        corpus_schedule_resume,
    ];
    let mut records = Vec::new();
    let mut passed = 0;
    let mut hard_stop = None;
    for (index, (name, control)) in NAMES.iter().zip(controls.iter()).enumerate() {
        let result = control(&mut context);
        let record = match result {
            Ok(()) => {
                passed += 1;
                json!({"index":index+1,"name":name,"status":"PASS"})
            }
            Err(error @ Error::Cuda(_)) => {
                let record = json!({"index":index+1,"name":name,"status":"HARD_STOP","error":error.to_string()});
                hard_stop = Some(error);
                record
            }
            Err(error) => {
                json!({"index":index+1,"name":name,"status":"FAIL","error":error.to_string()})
            }
        };
        println!("{record}");
        records.push(record);
        if hard_stop.is_some() {
            break;
        }
    }
    let attempted = records.len();
    for (index, name) in NAMES.iter().enumerate().skip(attempted) {
        let record =
            json!({"index":index+1,"name":name,"status":"NOT_RUN","reason":"preceding CUDA fault"});
        println!("{record}");
        records.push(record);
    }
    let dose_matches = context.applied_updates == 8 && context.apply_calls == 11;
    let payload = json!({"schema":"uor-r4.discrete-driver-finite/1","status":if hard_stop.is_some(){"ABORTED"}else if passed==NAMES.len() && dose_matches{"PASS"}else{"FAILED"},
        "registered":NAMES.len(),"attempted":attempted,"passed":passed,"failed":attempted-passed,"not_run":NAMES.len()-attempted,
        "controls":records,"applied_updates":context.applied_updates,"apply_calls":context.apply_calls,
        "expected_applied_updates_on_complete_pass":8,"expected_apply_calls_on_complete_pass":11,
        "update_count_complete":hard_stop.is_none(),"apply_dose_matches":dose_matches,"corpus_updates":0,"training":"HELD",
        "tiny_fixture_timings":context.timings,"timing_scope":"V2 B2 C1 only; not production apply throughput"});
    std::fs::write(
        report.join("controls.json"),
        serde_json::to_vec_pretty(&payload).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    if let Some(error) = hard_stop {
        return Err(error);
    }
    require(
        passed == NAMES.len(),
        "one or more finite driver controls failed",
    )?;
    require(
        context.applied_updates == 8 && context.apply_calls == 11,
        "finite apply dose differs from registered8updates/11calls",
    )
}
