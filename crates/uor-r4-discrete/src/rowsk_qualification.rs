//! Finite CUDA controls for BVG-K Stage C K-coordinate scoring, plus the K
//! latency curve that tests the Stage C throughput hypothesis.
//!
//! V2 (scoring parity): for every coordinate of every block, the K-coordinate
//! launch's sixteen losses and status equal an independent `propose_rows` for
//! that coordinate on the same device, carried batch and targets. `propose_rows`
//! is itself exact to the ordinary kernel (#2630). V6 (refusals): an empty
//! block, an over-long block and a mixed-kind block are refused before any
//! launch. Every call sits inside an unchanged-state check. No host model
//! forward exists; both sides are the device's own kernels.
use crate::{
    device::{CoordinateScore, Device, RowsCounterfactual, RowsKScoring, MAX_COORDINATE_BLOCK},
    Error, Identity, ObjectivePolicy, Parameter, ReadSpec, Result, Spec,
};
use std::collections::BTreeSet;
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

/// The refusal text the ordinary path publishes for a counterfactual status.
fn refusal_text(status: u32) -> String {
    format!("discrete CUDA counterfactual refused with status {status}")
}

type Control = fn(&mut Tally) -> Result<()>;
/// (label, model, scoring handle, carried batch, block, expected refusal text).
type RefusalCase<'a> = (
    &'a str,
    &'a Device,
    &'a RowsKScoring,
    &'a crate::device::CarriedBatch,
    Vec<Parameter>,
    &'a str,
);

#[derive(Default)]
struct Tally {
    blocks: u64,
    cases: u64,
    successes: u64,
    refusals: u64,
    racy_refusals: u64,
    /// (rows, rows-K) observed status-set sizes for each racy refusal.
    racy_set_sizes: Vec<(usize, usize)>,
}

fn unchanged<T>(device: &Device, call: impl FnOnce() -> Result<T>) -> Result<T> {
    let before = device.snapshot()?;
    let outcome = call();
    let after = device.snapshot()?;
    require(
        before.table == after.table
            && before.routes == after.routes
            && before.gates == after.gates
            && before.epoch == after.epoch,
        "K-coordinate scoring or a single proposal changed learned state or epoch",
    )?;
    outcome
}

/// V2 for one block: each coordinate against an independent `propose_rows`.
fn compare_block(
    device: &Device,
    rows: &RowsCounterfactual,
    rowsk: &RowsKScoring,
    windows: &[Vec<Identity>],
    targets: &[u32],
    block: &[Parameter],
    tally: &mut Tally,
) -> Result<()> {
    let input = device.carry_windows(windows)?;
    let scores = unchanged(device, || {
        device.score_rows_k(&input, targets, block, rowsk)
    })?;
    require(scores.len() == block.len(), "one score per coordinate")?;
    tally.blocks += 1;
    for (score, &parameter) in scores.iter().zip(block) {
        tally.cases += 1;
        require(score.parameter == parameter, "scores follow block order")?;
        let single = unchanged(device, || {
            device.propose_rows(&input, targets, parameter, rows)
        });
        match (score.status, single) {
            (0, Ok(proposal)) => {
                let expected = proposal.candidate_losses()?;
                require(
                    expected == score.losses,
                    &format!(
                        "{parameter:?}: losses differ: rows {expected:?} rows-K {:?}",
                        score.losses
                    ),
                )?;
                tally.successes += 1;
            }
            (status, Err(Error::Invalid(text))) if status != 0 => {
                let mine = refusal_text(status);
                if mine != text {
                    // Lane terminals publish through atomicExch on both paths, so
                    // a block whose lanes end on different statuses has a racy
                    // final code. Re-run both and require the sets to overlap.
                    // The retries sit inside the same unchanged-state check.
                    let (seen_rows, seen_k) = unchanged(device, || {
                        let mut seen_rows = BTreeSet::from([text.clone()]);
                        let mut seen_k = BTreeSet::from([mine.clone()]);
                        for _ in 0..4 {
                            match device.propose_rows(&input, targets, parameter, rows) {
                                Err(Error::Invalid(t)) => {
                                    seen_rows.insert(t);
                                }
                                Err(e @ Error::Cuda(_)) => return Err(e),
                                other => {
                                    return Err(Error::Invalid(format!(
                                        "{parameter:?}: rows retry changed outcome class: ok={}",
                                        other.is_ok()
                                    )))
                                }
                            }
                            let again: Vec<CoordinateScore> =
                                device.score_rows_k(&input, targets, block, rowsk)?;
                            let s = again
                                .iter()
                                .find(|x| x.parameter == parameter)
                                .map(|x| x.status)
                                .unwrap_or(0);
                            require(s != 0, "rows-K retry succeeded after the case refused")?;
                            seen_k.insert(refusal_text(s));
                        }
                        Ok((seen_rows, seen_k))
                    })?;
                    tally.racy_set_sizes.push((seen_rows.len(), seen_k.len()));
                    require(
                        seen_rows.intersection(&seen_k).next().is_some(),
                        &format!(
                            "{parameter:?}: refusal sets disjoint: {seen_rows:?} vs {seen_k:?}"
                        ),
                    )?;
                    tally.racy_refusals += 1;
                }
                tally.refusals += 1;
            }
            (_, Err(error @ Error::Cuda(_))) => return Err(error),
            (status, other) => {
                return Err(Error::Invalid(format!(
                    "{parameter:?}: rows-K status {status} but rows ok={}",
                    other.is_ok()
                )))
            }
        }
    }
    Ok(())
}

/// V2 over complete inventories, chunked into blocks of each K.
fn scoring_parity(tally: &mut Tally) -> Result<()> {
    let (vocabulary, context) = (16, 12);
    for (read, blocks, objective) in [
        (ReadSpec::Route, 2, ObjectivePolicy::NearestAddressMarginV1),
        (ReadSpec::Route, 4, ObjectivePolicy::NearestAddressMarginV1),
        (ReadSpec::Route, 4, ObjectivePolicy::OccupiedCellRankHingeV1),
        (
            ReadSpec::Relation { kept: 2, sweeps: 1 },
            2,
            ObjectivePolicy::NearestAddressMarginV1,
        ),
    ] {
        let spec = Spec {
            vocabulary,
            blocks,
            context,
            kept_cells: 4,
        };
        for model_seed in [7u32, 11] {
            let device = Device::new_with_objective(spec, read, objective, model_seed, 0)?;
            let rows = device.rows_counterfactual()?;
            let rowsk = device.rowsk_scoring()?;
            let snapshot = device.snapshot()?;
            let mut rng = Lcg(0x5eed ^ u64::from(model_seed) ^ u64::from(blocks));
            // Each kind's complete admitted inventory, one kind per block.
            let kinds: Vec<Vec<Parameter>> = match read {
                ReadSpec::Route => vec![
                    (0..snapshot.routes.len() as u32)
                        .map(Parameter::Route)
                        .collect(),
                    (0..vocabulary * blocks)
                        .map(Parameter::VectorBlock)
                        .collect(),
                ],
                _ => vec![
                    (0..snapshot.gates.len() as u32)
                        .map(Parameter::Gate)
                        .collect(),
                    (0..vocabulary * blocks)
                        .map(Parameter::VectorBlock)
                        .collect(),
                ],
            };
            for row_count in [1usize, 7, 64] {
                let batch = windows(&mut rng, row_count, context, vocabulary);
                let targets: Vec<u32> = (0..row_count).map(|_| rng.next(vocabulary)).collect();
                for inventory in &kinds {
                    for k in [1usize, 4, 16] {
                        for block in inventory.chunks(k) {
                            compare_block(&device, &rows, &rowsk, &batch, &targets, block, tally)?;
                        }
                    }
                }
            }
        }
    }
    require(tally.successes > 0, "no successful parity case ran")
}

/// V6: refused before any launch, with state unchanged.
fn block_refusals(tally: &mut Tally) -> Result<()> {
    let spec = Spec {
        vocabulary: 16,
        blocks: 2,
        context: 12,
        kept_cells: 4,
    };
    let device = Device::new_with_objective(
        spec,
        ReadSpec::Route,
        ObjectivePolicy::NearestAddressMarginV1,
        7,
        0,
    )?;
    let rowsk = device.rowsk_scoring()?;
    let mut rng = Lcg(0xbad);
    let batch = windows(&mut rng, 7, 12, 16);
    let targets: Vec<u32> = (0..7).map(|_| rng.next(16)).collect();
    let input = device.carry_windows(&batch)?;
    let over: Vec<Parameter> = (0..(MAX_COORDINATE_BLOCK as u32 + 1))
        .map(|i| Parameter::VectorBlock(i % 32))
        .collect();
    // A Relation model for the read-kind refusals.
    let relation = Device::new_with_objective(
        spec,
        ReadSpec::Relation { kept: 2, sweeps: 1 },
        ObjectivePolicy::NearestAddressMarginV1,
        7,
        0,
    )?;
    let relation_scoring = relation.rowsk_scoring()?;
    let relation_input = relation.carry_windows(&batch)?;
    let gates = relation.snapshot()?.gates.len() as u32;
    // Each case must refuse with its own reason, not merely with some error.
    let cases: [RefusalCase; 6] = [
        (
            "empty",
            &device,
            &rowsk,
            &input,
            Vec::new(),
            "coordinate block must hold 1..=64",
        ),
        (
            "over-long",
            &device,
            &rowsk,
            &input,
            over,
            "coordinate block must hold 1..=64",
        ),
        (
            "mixed-kind",
            &device,
            &rowsk,
            &input,
            vec![Parameter::Route(0), Parameter::VectorBlock(0)],
            "must hold one parameter kind",
        ),
        (
            "out-of-inventory",
            &device,
            &rowsk,
            &input,
            vec![Parameter::VectorBlock(32)],
            "exceeds its declared inventory",
        ),
        (
            "route-in-relation",
            &relation,
            &relation_scoring,
            &relation_input,
            vec![Parameter::Route(0)],
            "route parameters are inactive in a relation model",
        ),
        (
            "gate-past-count",
            &relation,
            &relation_scoring,
            &relation_input,
            vec![Parameter::Gate(gates)],
            "exceeds its declared inventory",
        ),
    ];
    for (label, model, scoring, carried, block, expected) in cases {
        tally.cases += 1;
        let text = unchanged(model, || {
            match model.score_rows_k(carried, &targets, &block, scoring) {
                Err(e @ Error::Cuda(_)) => Err(e),
                Err(error) => Ok(Some(error.to_string())),
                Ok(_) => Ok(None),
            }
        })?;
        let text = text.ok_or_else(|| Error::Invalid(format!("{label} block was not refused")))?;
        require(
            text.contains(expected),
            &format!("{label} block refused for the wrong reason: {text}"),
        )?;
        tally.refusals += 1;
    }
    Ok(())
}

/// K latency curve at the registered K16 redirect shape (reported, not gated).
fn latency_curve() -> Result<serde_json::Value> {
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
    // Pre-registered protocol: 1 excluded warmup + TIMED synchronized samples;
    // median and nearest-rank p95; every raw sample retained.
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
        // Nearest rank: ceil(0.95 * n) - 1, in integers.
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
        let scores = device.score_rows_k(&input, &targets, &block, &rowsk)?;
        let first = device
            .propose_rows(&input, &targets, Parameter::VectorBlock(0), &rows)?
            .candidate_losses()?;
        require(
            scores[0].status == 0 && scores[0].losses == first,
            "latency-shape first coordinate differs from propose_rows",
        )?;
        let (ns, p95, raw) = sample(Box::new(|| {
            device
                .score_rows_k(&input, &targets, &block, &rowsk)
                .map(|_| ())
        }))?;
        cases.push(serde_json::json!({
            "k": k, "block_median_ns": ns as u64, "block_p95_ns": p95 as u64, "raw_samples_ns": raw,
            "ns_per_coordinate": (ns / u128::from(k)) as u64,
            "speedup_x1000_vs_k_single_proposals": (single_ns * u128::from(k) * 1000 / ns.max(1)) as u64,
        }));
    }
    Ok(serde_json::json!({
        "shape": "V4096/B4/C192/K16 Route objective A, seed 1 checkpoint 0, 64 rows, VectorBlock 0..K, 1 warmup + 20 timed, median and nearest-rank p95",
        "single_propose_rows_median_ns": single_ns as u64,
        "single_propose_rows_p95_ns": single_p95 as u64,
        "single_propose_rows_raw_samples_ns": single_raw,
        "qualified": false,
        "cases": cases,
    }))
}

pub fn run(report: &Path) -> Result<()> {
    std::fs::create_dir(report).map_err(|e| Error::Invalid(format!("exclusive report: {e}")))?;
    // V0 identity preflight: rows-K only appends to the unchanged rows source.
    require(
        crate::device::rowsk_extends_rows_source(),
        "embedded rowsk.cu does not begin with the embedded rows.cu",
    )?;
    let mut records = Vec::new();
    let mut failed = false;
    let controls: [(&str, Control); 2] = [
        ("v2_scoring_parity", scoring_parity),
        ("v6_block_refusals", block_refusals),
    ];
    for (name, control) in controls {
        let mut tally = Tally::default();
        let record = match control(&mut tally) {
            Ok(()) => serde_json::json!({
                "name": name, "status": "PASS", "blocks": tally.blocks, "cases": tally.cases,
                "successes": tally.successes, "refusals": tally.refusals,
                "racy_refusals": tally.racy_refusals,
                "racy_set_sizes_rows_rowsk": tally.racy_set_sizes,
            }),
            Err(error) => {
                failed = true;
                serde_json::json!({"name": name, "status": "FAIL", "cases": tally.cases, "error": error.to_string()})
            }
        };
        println!("{record}");
        records.push(record);
    }
    let latency = if failed {
        serde_json::json!("NOT_RUN: a control failed")
    } else {
        match latency_curve() {
            Ok(value) => value,
            Err(error) => {
                failed = true;
                serde_json::json!({"status": "FAIL", "error": error.to_string()})
            }
        }
    };
    println!("{latency}");
    let payload = serde_json::json!({
        "schema": crate::SCHEMA,
        "suite": "bvgk-rowsk-scoring-qualification",
        "status": if failed { "FAILED" } else { "PASS" },
        "controls": records,
        "latency_curve": latency,
        "rowsk_image_sha256": crate::device::compiled_rowsk_image_sha256(),
        "rows_image_sha256": crate::device::compiled_rows_image_sha256(),
        "rowsk_extends_rows_source": true,
        "driver_source_sha256": crate::training::driver_source_sha256(),
        "latency_scope": "host-callable Device::score_rows_k vs Device::propose_rows, including per-call buffer allocation, host-to-device and device-to-host copies and stream synchronization",
        "training": "HELD",
        "applied_updates": 0,
    });
    std::fs::write(
        report.join("rowsk-qualification.json"),
        serde_json::to_vec_pretty(&payload).map_err(|e| Error::Invalid(e.to_string()))?,
    )
    .map_err(|e| Error::Invalid(e.to_string()))?;
    require(!failed, "one or more K-coordinate controls failed")
}
