//! Finite CUDA parity controls for the opt-in row-parallel counterfactual.
//! Each case runs the ordinary `propose` and `propose_rows` on the same device,
//! carried batch, targets and parameter, and requires the same result: equal
//! sixteen-slot loss vectors and chooser observation on success, or the same
//! refusal text on failure, with learned state and epoch unchanged. No host
//! model forward exists; both sides are the device's own kernels. Inputs are
//! deterministic integer data (token identities and targets), not model output.
use crate::{
    device::{Device, Proposal},
    Error, Identity, Parameter, ReadSpec, Result, Spec,
};
use std::path::Path;

pub const NAMES: &[&str] = &[
    "rows_route_parity",
    "rows_flock_gate_parity",
    "rows_relation_gate_parity",
    "rows_refusal_parity",
    "rows_forced_terminal_parity",
];

fn require(ok: bool, why: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error::Invalid(why.into()))
    }
}

/// Deterministic integer stream for identities and targets.
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
            // Variable lengths 2..=context exercise padding and short plans.
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

#[derive(Default)]
struct Tally {
    cases: u64,
    successes: u64,
    /// Successes in which at least one active lane ended at the MAX sentinel
    /// (a broken lane: virtual cost MAX or virtual token_rivals 0).
    successes_with_broken_lane: u64,
    racy_refusals: u64,
    refusals: Vec<String>,
}

const A: [i32; 8] = [2, 2, 0, 0, 0, 0, 0, 0];
const B: [i32; 8] = [0, 0, 2, 0, 2, 0, 0, 0];

/// A two-block Flock device whose codebook rows are `tokens` ([block0, block1])
/// and whose gates are all zero, built like the flock controls' fixture.
fn table_device(context: u32, kept_cells: u32, tokens: &[[[i32; 8]; 2]]) -> Result<Device> {
    let spec = Spec {
        vocabulary: tokens.len() as u32,
        blocks: 2,
        context,
        kept_cells,
    };
    let device = Device::new_with_read(spec, ReadSpec::Flock { kept: 1 }, 7, 0)?;
    let mut snapshot = device.snapshot()?;
    for (token, blocks) in tokens.iter().enumerate() {
        let row: Vec<i32> = blocks.iter().flatten().copied().collect();
        snapshot.table[token * 16..token * 16 + 16].copy_from_slice(&row);
    }
    for gate in snapshot.gates.iter_mut() {
        *gate = 0;
    }
    Device::from_snapshot(snapshot, 0)
}

fn ident(token: u32, occurrence: u64) -> Identity {
    Identity {
        token,
        occurrence,
        record: 17,
        version: 9,
    }
}

/// Deterministic terminal paths (C8 geometry, `kept_cells = 1`, gates 0):
/// tokens 0 = (a, b) and 1 = (a, a) occupy different cells; with gates 0 the
/// read is the query's own vector.
/// - Every row ineligible: query token 0, target 0 → the only admitted cell is
///   the target's, with no rival token → status 8 on both paths.
/// - Lane +1 breaks: query token 0, target 1. The baseline admits token 0's
///   cell (one rival), so the row is eligible; Gate(1)=+1 moves block 1 to
///   snap(b + a, 2) = a, so the virtual read is token 1's vector and its only
///   admitted cell is the target's: virtual token_rivals 0 → that lane breaks
///   at row 0 and stays MAX although later rows are eligible.
fn rows_forced_terminal_parity(tally: &mut Tally) -> Result<()> {
    let device = table_device(2, 1, &[[A, B], [A, A]])?;
    let rows = device.rows_counterfactual()?;
    let window = |o: u64| vec![ident(1, o), ident(0, o + 1)];
    // Status 8: three rows, all target 0.
    let before = tally.refusals.len();
    compare(
        &device,
        &rows,
        &[window(1), window(3), window(5)],
        &[0, 0, 0],
        Parameter::Gate(1),
        tally,
    )?;
    require(
        tally.refusals.len() == before + 1
            && tally.refusals[before].ends_with("refused with status 8"),
        "all-ineligible batch did not refuse with status 8 on both paths",
    )?;
    // Break: row 0 breaks lane +1; rows 1 and 2 are eligible for lane CURRENT.
    let input = device.carry_windows(&[window(11), window(13), window(15)])?;
    let ordinary = device.propose(&input, &[1, 1, 1], Parameter::Gate(1))?;
    let losses = ordinary.candidate_losses()?;
    require(
        losses[0] != u64::MAX && losses[2] == u64::MAX,
        &format!(
            "break fixture: expected CURRENT finite and +1 at MAX (−1 also breaks by the index tie), observed {:?}",
            &losses[..3]
        ),
    )?;
    let broken = tally.successes_with_broken_lane;
    compare(
        &device,
        &rows,
        &[window(11), window(13), window(15)],
        &[1, 1, 1],
        Parameter::Gate(1),
        tally,
    )?;
    require(
        tally.successes_with_broken_lane == broken + 1,
        "break fixture was not counted as a broken-lane success",
    )
}

/// Run both paths on one case and require identical observable results.
/// Every call of a case, retries included, sits inside one unchanged-state check.
fn compare(
    device: &Device,
    rows: &crate::device::RowsCounterfactual,
    windows: &[Vec<Identity>],
    targets: &[u32],
    parameter: Parameter,
    tally: &mut Tally,
) -> Result<()> {
    let before = device.snapshot()?;
    let outcome = compare_calls(device, rows, windows, targets, parameter, tally);
    let after = device.snapshot()?;
    require(
        before.table == after.table
            && before.routes == after.routes
            && before.gates == after.gates
            && before.epoch == after.epoch,
        "a virtual proposal changed learned state or epoch",
    )?;
    outcome
}

/// One retry of a racy refusal: only an Invalid refusal joins the observed set;
/// a CUDA fault propagates and any other outcome class is a FAIL.
fn retry_refusal(
    outcome: Result<Proposal>,
    path: &str,
    seen: &mut std::collections::BTreeSet<String>,
) -> Result<()> {
    match outcome {
        Err(Error::Invalid(text)) => {
            seen.insert(text);
            Ok(())
        }
        Err(error @ Error::Cuda(_)) => Err(error),
        Ok(_) => Err(Error::Invalid(format!(
            "{path} retry succeeded after the case refused"
        ))),
        Err(other) => Err(Error::Invalid(format!(
            "{path} retry changed refusal kind: {other}"
        ))),
    }
}

fn compare_calls(
    device: &Device,
    rows: &crate::device::RowsCounterfactual,
    windows: &[Vec<Identity>],
    targets: &[u32],
    parameter: Parameter,
    tally: &mut Tally,
) -> Result<()> {
    let input = device.carry_windows(windows)?;
    let ordinary = device.propose(&input, targets, parameter);
    let parallel = device.propose_rows(&input, targets, parameter, rows);
    tally.cases += 1;
    match (ordinary, parallel) {
        (Ok(a), Ok(b)) => {
            let observe = |p: &Proposal| -> Result<(Vec<u64>, String)> {
                Ok((p.candidate_losses()?, format!("{:?}", p.observation()?)))
            };
            let (la, oa) = observe(&a)?;
            let (lb, ob) = observe(&b)?;
            require(
                la == lb,
                &format!("{parameter:?}: losses differ: ordinary {la:?} rows {lb:?}"),
            )?;
            require(
                oa == ob,
                &format!("{parameter:?}: observation differs: {oa} vs {ob}"),
            )?;
            tally.successes += 1;
            let active = if matches!(parameter, Parameter::Gate(_)) {
                3
            } else {
                16
            };
            if la[..active].contains(&u64::MAX) {
                tally.successes_with_broken_lane += 1;
            }
            Ok(())
        }
        (Err(Error::Invalid(a)), Err(Error::Invalid(b))) => {
            if a != b {
                // Both kernels publish lane terminals through atomicExch, so a
                // batch whose lanes end on different statuses has a racy final
                // code on either path. Re-run both and require the observed
                // status sets to overlap; the texts are recorded either way.
                let mut seen_a = std::collections::BTreeSet::from([a.clone()]);
                let mut seen_b = std::collections::BTreeSet::from([b.clone()]);
                for _ in 0..4 {
                    retry_refusal(
                        device.propose(&input, targets, parameter),
                        "ordinary",
                        &mut seen_a,
                    )?;
                    retry_refusal(
                        device.propose_rows(&input, targets, parameter, rows),
                        "rows",
                        &mut seen_b,
                    )?;
                }
                require(
                    seen_a.intersection(&seen_b).next().is_some(),
                    &format!("{parameter:?}: refusal sets disjoint: {seen_a:?} vs {seen_b:?}"),
                )?;
                tally.racy_refusals += 1;
            }
            tally.refusals.push(a);
            Ok(())
        }
        (Err(error @ Error::Cuda(_)), _) | (_, Err(error @ Error::Cuda(_))) => Err(error),
        // Host-side refusals of other kinds (e.g. a reserved parameter kind)
        // happen before either kernel; they must read identically.
        (Err(a), Err(b)) => {
            let (a, b) = (a.to_string(), b.to_string());
            require(
                a == b,
                &format!("{parameter:?}: host refusal differs: {a:?} vs {b:?}"),
            )?;
            tally.refusals.push(a);
            Ok(())
        }
        (a, b) => Err(Error::Invalid(format!(
            "{parameter:?}: one path refused and the other did not: ordinary ok={} rows ok={}",
            a.is_ok(),
            b.is_ok()
        ))),
    }
}

fn sweep(read: ReadSpec, blocks: u32, seed: u64, tally: &mut Tally) -> Result<()> {
    let (vocabulary, context) = (16, 12);
    let spec = Spec {
        vocabulary,
        blocks,
        context,
        kept_cells: 4,
    };
    for model_seed in [7u32, 11] {
        let device = Device::new_with_read(spec, read, model_seed, 0)?;
        let rows = device.rows_counterfactual()?;
        let snapshot = device.snapshot()?;
        let gates = snapshot.gates.len() as u32;
        let route_count = snapshot.routes.len() as u32;
        let mut rng = Lcg(seed ^ u64::from(model_seed));
        // One row, an odd count and the full 64-row admission bound.
        for row_count in [1usize, 7, 64] {
            let batch = windows(&mut rng, row_count, context, vocabulary);
            let targets: Vec<u32> = (0..row_count).map(|_| rng.next(vocabulary)).collect();
            // The complete admitted inventories (Device::parameter): every
            // route-table entry and every vocabulary-by-block codebook block.
            let parameters: Vec<Parameter> = match read {
                ReadSpec::Route => (0..route_count)
                    .map(Parameter::Route)
                    .chain((0..vocabulary * blocks).map(Parameter::VectorBlock))
                    .collect(),
                _ => (0..gates).map(Parameter::Gate).collect(),
            };
            for parameter in parameters {
                compare(&device, &rows, &batch, &targets, parameter, tally)?;
            }
        }
    }
    require(tally.cases > 0, "no parity case ran")
}

fn rows_route_parity(tally: &mut Tally) -> Result<()> {
    sweep(ReadSpec::Route, 2, 0x5157, tally)?;
    sweep(ReadSpec::Route, 4, 0x5158, tally)
}
fn rows_flock_gate_parity(tally: &mut Tally) -> Result<()> {
    sweep(ReadSpec::Flock { kept: 3 }, 2, 0xf10c, tally)?;
    sweep(ReadSpec::Flock { kept: 8 }, 4, 0xf10d, tally)
}
fn rows_relation_gate_parity(tally: &mut Tally) -> Result<()> {
    sweep(ReadSpec::Relation { kept: 3, sweeps: 1 }, 2, 0x2e1a, tally)?;
    sweep(ReadSpec::Relation { kept: 8, sweeps: 2 }, 4, 0x2e1b, tally)
}
/// Kind/mode and index refusals must read identically on both paths.
fn rows_refusal_parity(tally: &mut Tally) -> Result<()> {
    let spec = Spec {
        vocabulary: 16,
        blocks: 2,
        context: 12,
        kept_cells: 4,
    };
    let mut rng = Lcg(0x0e1f);
    for (read, parameters) in [
        (ReadSpec::Route, vec![Parameter::Gate(0)]),
        (
            ReadSpec::Flock { kept: 3 },
            vec![Parameter::Route(0), Parameter::Gate(u32::MAX)],
        ),
    ] {
        let device = Device::new_with_read(spec, read, 7, 0)?;
        let rows = device.rows_counterfactual()?;
        let batch = windows(&mut rng, 3, 12, 16);
        let targets = [1, 2, 3];
        for parameter in parameters {
            let before = tally.refusals.len();
            compare(&device, &rows, &batch, &targets, parameter, tally)?;
            require(
                tally.refusals.len() == before + 1,
                &format!("{parameter:?} under {read:?} was not refused"),
            )?;
        }
    }
    Ok(())
}

/// Non-gating wall-clock comparison at the Dose 1 shape (V4096/B4/C192, 64
/// full-context rows, kept cells 8). Each sample is one synchronized proposal;
/// one warmup per path is excluded; the median of five timed samples is kept.
/// Parity of these exact calls is also required. Not a qualified rate.
fn timing() -> Result<serde_json::Value> {
    let spec = Spec {
        vocabulary: 4096,
        blocks: 4,
        context: 192,
        kept_cells: 8,
    };
    let mut rng = Lcg(0x7193);
    let mut cases = Vec::new();
    for (read, parameter) in [
        (ReadSpec::Route, Parameter::VectorBlock(0)),
        (
            ReadSpec::Relation { kept: 8, sweeps: 1 },
            Parameter::Gate(0),
        ),
    ] {
        let device = Device::new_with_read(spec, read, 1, 0)?;
        let rows = device.rows_counterfactual()?;
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
        let sample = |use_rows: bool| -> Result<(u128, Vec<u64>)> {
            let started = std::time::Instant::now();
            let proposal = if use_rows {
                device.propose_rows(&input, &targets, parameter, &rows)?
            } else {
                device.propose(&input, &targets, parameter)?
            };
            let losses = proposal.candidate_losses()?;
            Ok((started.elapsed().as_nanos(), losses))
        };
        let median = |use_rows: bool| -> Result<(u128, Vec<u64>)> {
            sample(use_rows)?; // excluded warmup
            let mut times = Vec::new();
            let mut losses = Vec::new();
            for _ in 0..5 {
                let (ns, l) = sample(use_rows)?;
                times.push(ns);
                losses = l;
            }
            times.sort_unstable();
            Ok((times[2], losses))
        };
        let (ordinary_ns, ordinary_losses) = median(false)?;
        let (rows_ns, rows_losses) = median(true)?;
        require(
            ordinary_losses == rows_losses,
            &format!("{read:?}/{parameter:?}: timing-shape losses differ"),
        )?;
        cases.push(serde_json::json!({
            "read": format!("{read:?}"), "parameter": format!("{parameter:?}"),
            "ordinary_median_ns": ordinary_ns as u64, "rows_median_ns": rows_ns as u64,
            "losses_equal": true,
        }));
    }
    Ok(serde_json::json!({
        "shape": "V4096/B4/C192/K8, 64 rows, synchronized propose, 1 warmup + 5 timed, median",
        "qualified": false,
        "cases": cases,
    }))
}

pub fn run(report: &Path) -> Result<()> {
    std::fs::create_dir(report).map_err(|e| Error::Invalid(format!("exclusive report: {e}")))?;
    let controls: [fn(&mut Tally) -> Result<()>; 5] = [
        rows_route_parity,
        rows_flock_gate_parity,
        rows_relation_gate_parity,
        rows_refusal_parity,
        rows_forced_terminal_parity,
    ];
    let mut records = Vec::new();
    let mut failed = false;
    for (index, (name, control)) in NAMES.iter().zip(controls).enumerate() {
        let mut tally = Tally::default();
        let record = match control(&mut tally) {
            Ok(()) => serde_json::json!({
                "index": index + 1, "name": name, "status": "PASS",
                "cases": tally.cases, "successes": tally.successes,
                "successes_with_broken_lane": tally.successes_with_broken_lane,
                "racy_refusals": tally.racy_refusals,
                "refusals": tally.refusals,
            }),
            Err(error) => {
                failed = true;
                serde_json::json!({
                    "index": index + 1, "name": name, "status": "FAIL",
                    "cases": tally.cases, "error": error.to_string(),
                })
            }
        };
        println!("{record}");
        records.push(record);
    }
    // Timing runs only after every parity control passed; it is reported, not gated.
    let timing = if failed {
        serde_json::json!("NOT_RUN: parity failed")
    } else {
        match timing() {
            Ok(value) => value,
            Err(error) => {
                failed = true;
                serde_json::json!({ "status": "FAIL", "error": error.to_string() })
            }
        }
    };
    println!("{timing}");
    let payload = serde_json::json!({
        "schema": crate::SCHEMA,
        "suite": "rows-counterfactual-parity",
        "status": if failed { "FAILED" } else { "PASS" },
        "timing": timing,
        "rows_image_sha256": crate::device::compiled_rows_image_sha256(),
        "controls": records,
        "training": "HELD",
        "applied_updates": 0,
    });
    std::fs::write(
        report.join("rows-parity.json"),
        serde_json::to_vec_pretty(&payload).map_err(|e| Error::Invalid(e.to_string()))?,
    )
    .map_err(|e| Error::Invalid(e.to_string()))?;
    require(!failed, "one or more row-parallel parity controls failed")
}
