//! K16 synchronized 100-batch timing instrument (dated record 2026-10-15).
//!
//! Measures the carry + propose + observe path over a FROZEN snapshot at the K16
//! shape: V4096 / B4 / C192 / K16, Route read policy, 64 rows, 100 distinct
//! batches, both objective policies, one excluded warmup per cell.
//!
//! What this is NOT, said here because a timing number is easy to misread:
//!
//!   - no apply and no update: each cell asserts an identical snapshot digest and
//!     epoch before and after, so these are forward-only batches;
//!   - therefore the nanoseconds are NOT a training dose, and EXCLUSION says so
//!     inside every record;
//!   - no corpus write, no training resume, no change to measurement.rs or to any
//!     production kernel. This instrument only calls the public device API.
//!
//! The core below (pins, validation, batch construction, rate arithmetic, receipt
//! ledger, atomic record writes) is host-testable and runs anywhere. The device
//! driver is behind the cuda feature and needs a pod; execution waits for the
//! qualification stage.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_discrete::{Identity, ObjectivePolicy, Spec};

/// The pinned shape. K16 is the evaluation-only rebind: the sealed fixture was
/// authored at K8 and is re-read at K16 without any training resume.
pub const K16: Spec = Spec {
    vocabulary: 4096,
    blocks: 4,
    context: 192,
    kept_cells: 16,
};
pub const ROWS: usize = 64;
pub const PADDED_CONTEXT_TOKENS_PER_ROW: u128 = 192;
pub const BATCHES: usize = 100;
pub const WARMUPS: usize = 1;
pub const PIN_SCHEMA: &str = "k16-timing-pin/v1";

/// The headline rate basis, named in the plan and in every cell record so no
/// reader has to infer which phases the divisor covers.
pub const RATE_BASIS: &str = "full timed unit: carry_windows + propose(route) + observation + propose(vector_block) + observation";

/// One word on dtype, because the vocabulary matters here: the stack is
/// integer-only and the timings are host wall-clock integers, never floats.
pub const DTYPE: &str = "integer nanoseconds (host wall clock, u128) over an integer-only discrete stack; no floating point";

/// The sentence every record carries. Extrapolating a training dose from these
/// numbers without quoting it is the mistake this constant exists to prevent.
pub const EXCLUSION: &str = "no apply/update is timed: every cell asserts an unchanged snapshot digest and epoch, so these are forward-only carry+propose+observe batches and NOT a training dose";

/// Pinned cell sources. checkpoint0 is the untrained ceiling source G-Pre uses;
/// the sealed Q4 fixture is the seed-1 evaluation-only rebind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Checkpoint0,
    SealedSeed1Q4,
}

impl Source {
    fn parse(text: &str) -> std::result::Result<Self, String> {
        match text {
            "checkpoint0" => Ok(Self::Checkpoint0),
            "sealed_seed1_q4" => Ok(Self::SealedSeed1Q4),
            other => Err(format!("unknown cell source {other:?}")),
        }
    }
}

fn policy_name(policy: ObjectivePolicy) -> &'static str {
    match policy {
        ObjectivePolicy::NearestAddressMarginV1 => "nearest_address_margin_v1",
        ObjectivePolicy::OccupiedCellRankHingeV1 => "occupied_cell_rank_hinge_v1",
    }
}

fn policy_parse(text: &str) -> std::result::Result<ObjectivePolicy, String> {
    match text {
        "nearest_address_margin_v1" => Ok(ObjectivePolicy::NearestAddressMarginV1),
        "occupied_cell_rank_hinge_v1" => Ok(ObjectivePolicy::OccupiedCellRankHingeV1),
        other => Err(format!("unknown objective policy {other:?}")),
    }
}

/// The whole run is pinned by a JSON file, deny_unknown_fields, so a typo in a
/// pin is an error rather than a silently ignored field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub schema: String,
    pub spec: Spec,
    pub read_spec: String,
    pub rows: usize,
    pub batches: usize,
    pub warmups: usize,
    pub fixture: FixturePin,
    pub cells: Vec<CellPin>,
    pub out_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixturePin {
    pub snapshot_path: PathBuf,
    /// SHA256 of the snapshot file, exactly as sealed. Absent or short is a fault.
    pub snapshot_sha256: String,
    pub seed: u32,
    pub epoch: u64,
    /// Declared occupied cells of the sealed Q4 fixture (189 for seed 1).
    pub occupied_cells: u64,
    /// The rebind is evaluation-only; a resume is not authorized here.
    pub evaluation_only_rebind: bool,
    pub original_k: u32,
    pub original_sha256: String,
    pub training_resume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CellPin {
    pub source: String,
    pub policy: String,
}

/// One (source, policy) cell, resolved. The run is the cross product the pin
/// lists, and the pin must name both policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub source: Source,
    pub policy: ObjectivePolicy,
}

fn is_hex64(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Fail closed. Every branch here is a pin the run cannot honestly continue
/// without: an absent fixture pin, a shape that is not K16, a read policy that
/// is not Route, a resume dressed as a rebind, or a missing policy arm.
pub fn validate(pin: &Pin) -> std::result::Result<Vec<Cell>, String> {
    if pin.schema != PIN_SCHEMA {
        return Err(format!(
            "pin schema is {:?}, expected {PIN_SCHEMA:?}",
            pin.schema
        ));
    }
    if pin.spec != K16 {
        return Err(format!(
            "shape mismatch: pin is V{}/B{}/C{}/K{}, expected V{}/B{}/C{}/K{} (the K8-to-K16 rebind is evaluation-only)",
            pin.spec.vocabulary,
            pin.spec.blocks,
            pin.spec.context,
            pin.spec.kept_cells,
            K16.vocabulary,
            K16.blocks,
            K16.context,
            K16.kept_cells
        ));
    }
    if pin.read_spec != "route" {
        return Err(format!(
            "read-policy mismatch: pin is {:?}, this instrument is pinned to Route",
            pin.read_spec
        ));
    }
    if pin.rows != ROWS {
        return Err(format!("rows is {}, expected {ROWS}", pin.rows));
    }
    if pin.batches != BATCHES {
        return Err(format!("batches is {}, expected {BATCHES}", pin.batches));
    }
    if pin.warmups != WARMUPS {
        return Err(format!("warmups is {}, expected {WARMUPS}", pin.warmups));
    }
    if !is_hex64(&pin.fixture.snapshot_sha256) {
        return Err("absent or malformed fixture snapshot_sha256: the sealed Q4 pin is required before any timing".into());
    }
    if !is_hex64(&pin.fixture.original_sha256) {
        return Err(
            "absent or malformed fixture original_sha256: the pre-rebind identity must be retained"
                .into(),
        );
    }
    if pin.fixture.snapshot_path.as_os_str().is_empty() {
        return Err("absent fixture snapshot_path".into());
    }
    if !pin.fixture.evaluation_only_rebind {
        return Err(
            "evaluation_only_rebind is false: this instrument may only read a sealed fixture"
                .into(),
        );
    }
    if pin.fixture.training_resume {
        return Err("training_resume is true: a resume is not authorized and is not what this instrument measures".into());
    }
    if pin.out_dir.as_os_str().is_empty() {
        return Err("absent out_dir".into());
    }
    if pin.cells.is_empty() {
        return Err("no cells: name at least one (source, policy)".into());
    }
    let mut cells = Vec::new();
    for cell in &pin.cells {
        cells.push(Cell {
            source: Source::parse(&cell.source)?,
            policy: policy_parse(&cell.policy)?,
        });
    }
    for policy in [
        ObjectivePolicy::NearestAddressMarginV1,
        ObjectivePolicy::OccupiedCellRankHingeV1,
    ] {
        if !cells.iter().any(|c| c.policy == policy) {
            return Err(format!(
                "both objective policies are required; {} is missing",
                policy_name(policy)
            ));
        }
    }
    let mut seen: Vec<(Source, &str)> = Vec::new();
    for cell in &cells {
        let key = (cell.source, policy_name(cell.policy));
        if seen.contains(&key) {
            return Err(format!("duplicate cell {:?} / {:?}", key.0, key.1));
        }
        seen.push(key);
    }
    Ok(cells)
}

/// The canonical vector coordinates, distinct per batch.
///
/// Mirrors the fixture construction the existing K8 benchmark uses (fixed input
/// identities, no corpus and no host model) with the batch folded into the token
/// and the occurrence, so the 100 batches are distinct while every batch keeps
/// the same shape and the same canonical targets.
pub fn batches(count: usize) -> Vec<Vec<Vec<Identity>>> {
    (0..count)
        .map(|batch| {
            (0..ROWS)
                .map(|row| {
                    (0..K16.context as usize)
                        .map(|position| Identity {
                            token: (row as u32 + position as u32 + batch as u32) % K16.vocabulary,
                            occurrence: batch as u64 * (ROWS as u64 * K16.context as u64)
                                + row as u64 * K16.context as u64
                                + position as u64,
                            record: row as u64,
                            version: 1,
                        })
                        .collect()
                })
                .collect()
        })
        .collect()
}

/// Canonical targets: the next coordinate after the row window start.
pub fn targets() -> Vec<u32> {
    (0..ROWS)
        .map(|row| (row as u32 + K16.context) % K16.vocabulary)
        .collect()
}

/// Integer floors of the measured nanoseconds. Two rates on purpose: rows per
/// second is the acceptance-facing number, and padded context tokens per second
/// is the physical work the device actually touched (64 rows * 192 positions).
///
/// `total_ns` is the FULL timed unit (see RATE_BASIS), not the carry phase alone:
/// a carry-only divisor understates per-batch cost, and this rate sizes the
/// D-Packet training budget. The carry-only rate is computed by calling this with
/// the carry nanoseconds and is reported as an explicitly named secondary field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Rates {
    pub rows_per_s: u128,
    pub padded_context_tokens_per_s: u128,
}

pub fn rates(timed_batches: usize, total_ns: u128) -> Rates {
    if total_ns == 0 {
        return Rates {
            rows_per_s: 0,
            padded_context_tokens_per_s: 0,
        };
    }
    let rows = ROWS as u128 * timed_batches as u128;
    let tokens = rows * PADDED_CONTEXT_TOKENS_PER_ROW;
    Rates {
        rows_per_s: rows * 1_000_000_000 / total_ns,
        padded_context_tokens_per_s: tokens * 1_000_000_000 / total_ns,
    }
}

/// Times an operation even when it fails, so a fault keeps its elapsed number.
pub fn timed<T>(
    operation: impl FnOnce() -> std::result::Result<T, String>,
) -> (std::result::Result<T, String>, u128) {
    let start = Instant::now();
    let result = operation();
    (result, start.elapsed().as_nanos())
}

/// Write a record atomically: a temporary file beside it, then a rename. A reader
/// never sees a half-written receipt, and a failed write leaves no temp behind.
pub fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    if let Err(e) = std::fs::write(&tmp, text) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

/// Append-only receipts with a fault path that PRESERVES what was measured.
///
/// A timing run that faults on batch 60 is still evidence about batches 1..59, so
/// the partial record is written before the error propagates. Discarding it would
/// be the same mistake as discarding a failed experiment.
pub struct Ledger {
    pub path: PathBuf,
    pub header: Value,
    pub records: Vec<Value>,
    pub faults: Vec<Value>,
}

impl Ledger {
    pub fn new(path: PathBuf, header: Value) -> Self {
        Self {
            path,
            header,
            records: Vec::new(),
            faults: Vec::new(),
        }
    }

    pub fn push(&mut self, record: Value) {
        self.records.push(record);
    }

    /// Record the fault, flush the partial record, and hand back the message so the
    /// caller can return it as its error.
    pub fn fault(&mut self, stage: &str, message: &str) -> String {
        self.faults.push(json!({
            "record_kind": "fault",
            "stage": stage,
            "message": message,
            "preserved_records": self.records.len(),
        }));
        let _ = self.flush("partial");
        message.to_string()
    }

    /// status is "partial" when the run faulted and "complete" otherwise.
    pub fn flush(&self, status: &str) -> std::result::Result<(), String> {
        let document = json!({
            "schema": "k16-timing-record/v1",
            "status": status,
            "exclusion": EXCLUSION,
            "header": self.header,
            "records": self.records,
            "faults": self.faults,
        });
        let text = serde_json::to_string_pretty(&document)
            .map_err(|e| format!("cannot render record: {e}"))?;
        write_atomic(&self.path, &(text + &String::from(char::from(10))))
            .map_err(|e| format!("cannot write {}: {e}", self.path.display()))
    }
}

/// The host-visible plan, so a reviewer can see exactly what a run would do without
/// a pod. Printed by check/plan and embedded in every record.
pub fn plan(pin: &Pin, cells: &[Cell]) -> Value {
    json!({
        "schema": "k16-timing-plan/v1",
        "shape": {
            "vocabulary": K16.vocabulary,
            "blocks": K16.blocks,
            "context": K16.context,
            "kept_cells": K16.kept_cells,
            "rows": ROWS,
            "padded_context_tokens_per_batch": ROWS as u128 * PADDED_CONTEXT_TOKENS_PER_ROW,
        },
        "read_spec": pin.read_spec,
        "batches_per_cell": pin.batches,
        "excluded_warmups_per_cell": pin.warmups,
        "timed_operations": ["carry_windows", "propose(route)", "observation", "propose(vector_block)", "observation"],
        "rates_basis": RATE_BASIS,
        "rates_secondary": "carry_only rows_per_s and padded_context_tokens_per_s are reported separately and are NOT the headline basis",
        "dtype": DTYPE,
        "synchronization": "Device::synchronize before and after every timed operation",
        "apply": false,
        "zero_apply_assertion": "snapshot digest and epoch identical before and after each cell",
        "fixture": pin.fixture,
        "cells": cells.iter().map(|c| json!({"source": c.source, "policy": policy_name(c.policy)})).collect::<Vec<_>>(),
        "exclusion": EXCLUSION,
    })
}

#[cfg(feature = "cuda")]
mod driver {
    use super::*;
    use sha2::{Digest, Sha256};
    use uor_r4_discrete::device::{compiled_identity, Device, Snapshot};
    use uor_r4_discrete::{Parameter, ReadSpec};

    fn digest(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Load the sealed fixture, fail closed on a checksum that does not match the
    /// pin: a fixture that is not the sealed one is not the thing under test.
    pub fn load_fixture(pin: &Pin) -> std::result::Result<Device, String> {
        let bytes = std::fs::read(&pin.fixture.snapshot_path)
            .map_err(|e| format!("cannot read {}: {e}", pin.fixture.snapshot_path.display()))?;
        let actual = digest(&bytes);
        if actual != pin.fixture.snapshot_sha256 {
            return Err(format!(
                "fixture checksum mismatch: file is {actual}, pin says {}",
                pin.fixture.snapshot_sha256
            ));
        }
        let snapshot: Snapshot = serde_json::from_slice(&bytes)
            .map_err(|e| format!("cannot parse fixture snapshot: {e}"))?;
        if snapshot.spec != K16 {
            return Err(format!(
                "fixture spec is {:?}, expected the K16 rebind",
                snapshot.spec
            ));
        }
        if snapshot.read_spec != ReadSpec::Route {
            return Err("fixture read spec is not Route".into());
        }
        if snapshot.seed != pin.fixture.seed {
            return Err(format!(
                "fixture seed is {}, pin says {}",
                snapshot.seed, pin.fixture.seed
            ));
        }
        if snapshot.epoch != pin.fixture.epoch {
            return Err(format!(
                "fixture epoch is {}, pin says {}",
                snapshot.epoch, pin.fixture.epoch
            ));
        }
        Device::from_snapshot(snapshot, 0).map_err(|e| format!("cannot admit fixture: {e}"))
    }

    /// One cell: one excluded warmup, then the pinned batches, each timed as
    /// carry + propose + observe with synchronize on both sides of every step.
    pub fn run_cell(
        device: &Device,
        cell: &Cell,
        pin: &Pin,
        ledger: &mut Ledger,
    ) -> std::result::Result<Value, String> {
        let windows = batches(pin.batches);
        let targets = targets();
        let before = device
            .snapshot()
            .map_err(|e| format!("snapshot before: {e}"))?;
        let epoch_before = device.epoch().map_err(|e| format!("epoch before: {e}"))?;
        let before_digest = serde_json::to_string(&before)
            .map(|text| digest(text.as_bytes()))
            .map_err(|e| format!("snapshot digest: {e}"))?;

        let proposals = [
            ("route", Parameter::Route(0)),
            ("vector_block", Parameter::VectorBlock(0)),
        ];
        let mut timed_batches = 0usize;
        // The timed UNIT is carry + propose + observation, so the headline rate
        // divides by all of it. Carry-only is kept as a separately named number:
        // a carry-only headline understates per-batch cost in the expensive
        // direction, and this rate sizes the D-Packet training budget.
        let mut unit_ns: u128 = 0;
        let mut carry_ns_total: u128 = 0;
        let mut route_propose_ns: u128 = 0;
        let mut route_observation_ns: u128 = 0;
        let mut vector_propose_ns: u128 = 0;
        let mut vector_observation_ns: u128 = 0;

        // One excluded warmup per cell: every entry used in a timed sample is
        // exercised once first, so first-use JIT is not charged to the sample.
        device
            .synchronize()
            .map_err(|e| format!("warmup synchronize: {e}"))?;
        let warmup_batch = device
            .carry_windows(&windows[0])
            .map_err(|e| format!("warmup carry: {e}"))?;
        for (_, parameter) in proposals {
            let _ = device
                .propose(&warmup_batch, &targets, parameter)
                .map_err(|e| format!("warmup propose: {e}"))?;
        }
        device
            .synchronize()
            .map_err(|e| format!("warmup final synchronize: {e}"))?;

        for (index, window) in windows.iter().enumerate().skip(pin.warmups) {
            if let Err(e) = device.synchronize() {
                return Err(ledger.fault("sync_before_carry", &e.to_string()));
            }
            let (carried, carry_ns) = timed(|| {
                let batch = device.carry_windows(window).map_err(|e| e.to_string())?;
                device.synchronize().map_err(|e| e.to_string())?;
                Ok(batch)
            });
            let carried = match carried {
                Ok(batch) => batch,
                Err(e) => return Err(ledger.fault("carry_windows", &e)),
            };

            let mut proposal_records = Vec::new();
            for (operation, parameter) in proposals {
                if let Err(e) = device.synchronize() {
                    return Err(ledger.fault("sync_before_propose", &e.to_string()));
                }
                let (proposal, propose_ns) = timed(|| {
                    let proposal = device
                        .propose(&carried, &targets, parameter)
                        .map_err(|e| e.to_string())?;
                    device.synchronize().map_err(|e| e.to_string())?;
                    Ok(proposal)
                });
                let proposal = match proposal {
                    Ok(proposal) => proposal,
                    Err(e) => return Err(ledger.fault("propose", &e)),
                };
                let (observation, observation_ns) = timed(|| {
                    let observed = proposal.observation().map_err(|e| e.to_string())?;
                    device.synchronize().map_err(|e| e.to_string())?;
                    Ok(observed)
                });
                let observed = match observation {
                    Ok(observed) => observed,
                    Err(e) => return Err(ledger.fault("observation", &e)),
                };
                match operation {
                    "route" => {
                        route_propose_ns += propose_ns;
                        route_observation_ns += observation_ns;
                    }
                    _ => {
                        vector_propose_ns += propose_ns;
                        vector_observation_ns += observation_ns;
                    }
                }
                proposal_records.push(json!({
                    "operation": operation,
                    "parameter": format!("{parameter:?}"),
                    "propose_ns": propose_ns,
                    "observation_ns": observation_ns,
                    "candidate": observed.candidate,
                    "before": observed.before,
                    "after": observed.after,
                    "epoch": observed.epoch,
                    "any_candidate_cost_differs_from_identity": observed.any_candidate_cost_differs_from_identity,
                    "active_candidates": if matches!(parameter, Parameter::Gate(_)) { 3 } else { K16.kept_cells },
                    "applied": false,
                }));
            }

            let epoch = match device.epoch() {
                Ok(epoch) => epoch,
                Err(e) => return Err(ledger.fault("epoch_read", &e.to_string())),
            };
            carry_ns_total += carry_ns;
            timed_batches += 1;
            ledger.push(json!({
                "record_kind": "batch",
                "cell": {"source": cell.source, "policy": policy_name(cell.policy)},
                "batch": index,
                "rows": ROWS,
                "carry_ns": carry_ns,
                "proposals": proposal_records,
                "epoch": epoch,
                "applied": false,
            }));
        }

        // Zero-apply proof. A changed snapshot or epoch means something applied, and
        // then this is not the measurement the record claims it is.
        let after = device
            .snapshot()
            .map_err(|e| format!("snapshot after: {e}"))?;
        let after_digest = serde_json::to_string(&after)
            .map(|text| digest(text.as_bytes()))
            .map_err(|e| format!("snapshot digest: {e}"))?;
        let epoch_after = device.epoch().map_err(|e| format!("epoch after: {e}"))?;
        if before_digest != after_digest || epoch_before != epoch_after {
            return Err(ledger.fault(
                "zero_apply",
                "snapshot or epoch changed during a forward-only cell",
            ));
        }

        unit_ns = carry_ns_total
            + route_propose_ns
            + route_observation_ns
            + vector_propose_ns
            + vector_observation_ns;
        let rates = rates(timed_batches, unit_ns);
        let carry_only = rates(timed_batches, carry_ns_total);
        let per_batch = |total: u128| -> u128 {
            if timed_batches == 0 {
                0
            } else {
                total / timed_batches as u128
            }
        };
        Ok(json!({
            "record_kind": "cell",
            "cell": {"source": cell.source, "policy": policy_name(cell.policy)},
            "timed_batches": timed_batches,
            "excluded_warmups": pin.warmups,
            "unit_ns": unit_ns,
            "rates": {
                "basis": RATE_BASIS,
                "rows_per_s": rates.rows_per_s,
                "padded_context_tokens_per_s": rates.padded_context_tokens_per_s,
            },
            "carry_only": {
                "ns": carry_ns_total,
                "rows_per_s": carry_only.rows_per_s,
                "padded_context_tokens_per_s": carry_only.padded_context_tokens_per_s,
                "note": "carry phase only; not the headline basis",
            },
            "per_family_ns_per_batch_floor": {
                "carry": per_batch(carry_ns_total),
                "route_propose": per_batch(route_propose_ns),
                "route_observation": per_batch(route_observation_ns),
                "vector_propose": per_batch(vector_propose_ns),
                "vector_observation": per_batch(vector_observation_ns),
            },
            "dtype": DTYPE,
            "snapshot_digest_before": before_digest,
            "snapshot_digest_after": after_digest,
            "epoch_before": epoch_before,
            "epoch_after": epoch_after,
            "identities": {
                "compiled": compiled_identity(),
                "device": device.resource_metadata().map_err(|e| e.to_string())?,
            },
            "applied": false,
            "exclusion": EXCLUSION,
        }))
    }
}

fn read_pin(path: &Path) -> std::result::Result<Pin, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("cannot parse {}: {e}", path.display()))
}

fn usage() -> String {
    [
        "k16-timing check|plan --pin FILE   (host; no device)",
        "k16-timing run --pin FILE          (needs --features cuda and a pod)",
    ]
    .join(&String::from(char::from(10)))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().cloned().unwrap_or_default();
    let pin_path = args
        .iter()
        .position(|a| a == "--pin")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    let Some(pin_path) = pin_path else {
        eprintln!("{}", usage());
        std::process::exit(2);
    };
    let pin = match read_pin(&pin_path) {
        Ok(pin) => pin,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let cells = match validate(&pin) {
        Ok(cells) => cells,
        Err(e) => {
            eprintln!("pin refused: {e}");
            std::process::exit(2);
        }
    };
    let refused = || {
        eprintln!("run needs --features cuda and a pod; this build is host-only");
        std::process::exit(2);
    };
    match command.as_str() {
        "check" | "plan" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&plan(&pin, &cells)).unwrap_or_default()
            );
        }
        "run" => {
            #[cfg(feature = "cuda")]
            run_all(&pin, &cells);
            #[cfg(not(feature = "cuda"))]
            {
                let _ = &refused;
                refused();
            }
        }
        other => {
            eprintln!("unknown command {other:?}");
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    }
}

#[cfg(feature = "cuda")]
fn run_all(pin: &Pin, cells: &[Cell]) {
    let path = pin.out_dir.join("k16-timing-record.json");
    let mut ledger = Ledger::new(path, plan(pin, cells));
    let device = match driver::load_fixture(pin) {
        Ok(device) => device,
        Err(e) => {
            let message = ledger.fault("load_fixture", &e);
            eprintln!("{message}");
            std::process::exit(1);
        }
    };
    for cell in cells {
        match driver::run_cell(&device, cell, pin, &mut ledger) {
            Ok(record) => ledger.push(record),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
    if let Err(e) = ledger.flush("complete") {
        eprintln!("{e}");
        std::process::exit(1);
    }
    println!("record: {}", ledger.path.display());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pin_value() -> Value {
        json!({
            "schema": PIN_SCHEMA,
            "spec": {"vocabulary": 4096, "blocks": 4, "context": 192, "kept_cells": 16},
            "read_spec": "route",
            "rows": ROWS,
            "batches": BATCHES,
            "warmups": WARMUPS,
            "fixture": {
                "snapshot_path": "/tmp/sealed-q4.json",
                "snapshot_sha256": "a".repeat(64),
                "seed": 1,
                "epoch": 0,
                "occupied_cells": 189,
                "evaluation_only_rebind": true,
                "original_k": 8,
                "original_sha256": "b".repeat(64),
                "training_resume": false
            },
            "cells": [
                {"source": "checkpoint0", "policy": "nearest_address_margin_v1"},
                {"source": "checkpoint0", "policy": "occupied_cell_rank_hinge_v1"},
                {"source": "sealed_seed1_q4", "policy": "nearest_address_margin_v1"},
                {"source": "sealed_seed1_q4", "policy": "occupied_cell_rank_hinge_v1"}
            ],
            "out_dir": "/tmp/k16-timing"
        })
    }

    fn pin() -> Pin {
        serde_json::from_value(pin_value()).expect("pin parses")
    }

    #[test]
    fn the_pinned_pin_validates_and_names_four_cells() {
        let cells = validate(&pin()).expect("valid");
        assert_eq!(cells.len(), 4);
        assert!(cells.iter().any(|c| c.source == Source::Checkpoint0));
        assert!(cells.iter().any(|c| c.source == Source::SealedSeed1Q4));
    }

    #[test]
    fn a_shape_that_is_not_k16_fails_closed() {
        let mut value = pin_value();
        value["spec"]["kept_cells"] = json!(8);
        let err = validate(&serde_json::from_value(value).expect("parses")).expect_err("refused");
        assert!(err.contains("shape mismatch"), "{err}");
    }

    #[test]
    fn a_read_policy_that_is_not_route_fails_closed() {
        let mut value = pin_value();
        value["read_spec"] = json!("flock");
        let err = validate(&serde_json::from_value(value).expect("parses")).expect_err("refused");
        assert!(err.contains("read-policy mismatch"), "{err}");
    }

    #[test]
    fn an_absent_or_malformed_fixture_pin_fails_closed() {
        for (field, bad) in [
            ("snapshot_sha256", json!("")),
            ("snapshot_sha256", json!("abc")),
            ("original_sha256", json!("")),
        ] {
            let mut value = pin_value();
            value["fixture"][field] = bad;
            let err =
                validate(&serde_json::from_value(value).expect("parses")).expect_err("refused");
            assert!(err.contains(field), "{field}: {err}");
        }
    }

    #[test]
    fn a_resume_dressed_as_a_rebind_fails_closed() {
        let mut value = pin_value();
        value["fixture"]["training_resume"] = json!(true);
        let err = validate(&serde_json::from_value(value).expect("parses")).expect_err("refused");
        assert!(err.contains("training_resume"), "{err}");

        let mut value = pin_value();
        value["fixture"]["evaluation_only_rebind"] = json!(false);
        let err = validate(&serde_json::from_value(value).expect("parses")).expect_err("refused");
        assert!(err.contains("evaluation_only_rebind"), "{err}");
    }

    #[test]
    fn both_objective_policies_are_required() {
        let mut value = pin_value();
        value["cells"] = json!([{"source": "checkpoint0", "policy": "nearest_address_margin_v1"}]);
        let err = validate(&serde_json::from_value(value).expect("parses")).expect_err("refused");
        assert!(err.contains("both objective policies"), "{err}");
    }

    #[test]
    fn a_duplicate_cell_and_an_unknown_policy_are_refused() {
        let mut value = pin_value();
        value["cells"] = json!([
            {"source": "checkpoint0", "policy": "nearest_address_margin_v1"},
            {"source": "checkpoint0", "policy": "nearest_address_margin_v1"},
            {"source": "checkpoint0", "policy": "occupied_cell_rank_hinge_v1"}
        ]);
        let err = validate(&serde_json::from_value(value).expect("parses")).expect_err("refused");
        assert!(err.contains("duplicate cell"), "{err}");

        let mut value = pin_value();
        value["cells"][0]["policy"] = json!("softmax_margin_v1");
        let err = validate(&serde_json::from_value(value).expect("parses")).expect_err("refused");
        assert!(err.contains("unknown objective policy"), "{err}");
    }

    #[test]
    fn an_unknown_field_is_rejected_by_the_parser() {
        let mut value = pin_value();
        value["surprise"] = json!(1);
        assert!(serde_json::from_value::<Pin>(value).is_err());
    }

    #[test]
    fn the_hundred_batches_are_distinct_and_canonical() {
        let all = batches(BATCHES);
        assert_eq!(all.len(), BATCHES);
        assert!(all.iter().all(|b| b.len() == ROWS));
        assert!(all
            .iter()
            .all(|b| b.iter().all(|row| row.len() == K16.context as usize)));
        assert!(all
            .iter()
            .flatten()
            .flatten()
            .all(|id| id.token < K16.vocabulary && id.version == 1));
        let first: Vec<u64> = all[0].iter().map(|row| row[0].occurrence).collect();
        let second: Vec<u64> = all[1].iter().map(|row| row[0].occurrence).collect();
        assert_ne!(first, second, "batches must differ");
        let t = targets();
        assert_eq!(t.len(), ROWS);
        assert!(t.iter().all(|target| *target < K16.vocabulary));
        assert_eq!(t[0], K16.context % K16.vocabulary);
    }

    #[test]
    fn rates_are_integer_floors_over_the_measured_nanoseconds() {
        let r = rates(100, 1_000_000_000);
        assert_eq!(r.rows_per_s, 6_400);
        // 6,400 rows/s * 192 padded positions per row. (12,288 is tokens per BATCH, not per row.)
        assert_eq!(r.padded_context_tokens_per_s, 6_400 * 192);
        assert_eq!(ROWS as u128 * PADDED_CONTEXT_TOKENS_PER_ROW, 12_288);
        assert_eq!(rates(100, 0).rows_per_s, 0, "no division by zero");
        assert_eq!(rates(100, 3).rows_per_s, 6_400 * 1_000_000_000 / 3);
    }

    #[test]
    fn a_fault_preserves_the_partial_receipts() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("record.json");
        let mut ledger = Ledger::new(path.clone(), json!({"pin": "test"}));
        for batch in 0..3 {
            ledger.push(json!({"record_kind": "batch", "batch": batch}));
        }
        let message = ledger.fault("propose", "device fault on batch 3");
        assert!(message.contains("device fault"));
        let written: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("record written"))
                .expect("valid json");
        assert_eq!(written["status"], "partial");
        assert_eq!(written["records"].as_array().expect("records").len(), 3);
        assert_eq!(written["faults"][0]["stage"], "propose");
        assert!(written["exclusion"]
            .as_str()
            .expect("exclusion")
            .contains("NOT a training dose"));
    }

    #[test]
    fn a_complete_record_states_the_zero_apply_exclusion() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("record.json");
        let cells = validate(&pin()).expect("valid");
        let ledger = Ledger::new(path.clone(), plan(&pin(), &cells));
        ledger.flush("complete").expect("written");
        let written: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("record")).expect("json");
        assert_eq!(written["exclusion"], EXCLUSION);
        assert_eq!(written["header"]["apply"], false);
        assert_eq!(written["header"]["batches_per_cell"], BATCHES as u64);
        assert_eq!(
            written["header"]["shape"]["padded_context_tokens_per_batch"],
            12_288
        );
        assert_eq!(
            written["header"]["excluded_warmups_per_cell"],
            WARMUPS as u64
        );
    }

    #[test]
    fn the_plan_names_every_timed_operation_and_the_synchronization() {
        let cells = validate(&pin()).expect("valid");
        let planned = plan(&pin(), &cells);
        let operations = planned["timed_operations"].as_array().expect("ops");
        assert!(operations.iter().any(|op| op == "carry_windows"));
        assert!(operations.iter().any(|op| op == "propose(route)"));
        assert!(operations.iter().any(|op| op == "observation"));
        assert_eq!(planned["apply"], false);
        assert_eq!(planned["rates_basis"], RATE_BASIS);
        assert!(
            planned["rates_basis"]
                .as_str()
                .expect("basis")
                .contains("propose"),
            "the basis must name the propose phases"
        );
        assert!(planned["dtype"]
            .as_str()
            .expect("dtype")
            .contains("integer"));
        assert_eq!(planned["cells"].as_array().expect("cells").len(), 4);
    }

    /// The reviewer caught a carry-only headline: the timed unit is carry + propose +
    /// observation, so the headline divides by all of it and carry-only is a named
    /// secondary. This pins the vocabulary, which is the part a host can check.
    #[test]
    fn the_rate_basis_is_the_full_unit_and_carry_only_is_named_secondary() {
        assert!(RATE_BASIS.contains("carry_windows"));
        assert!(RATE_BASIS.contains("propose(route)"));
        assert!(RATE_BASIS.contains("observation"));
        assert!(RATE_BASIS.contains("propose(vector_block)"));
        assert!(DTYPE.contains("integer"));
        assert!(DTYPE.contains("no floating point"));
        // The same arithmetic, two bases: the full unit is never smaller than carry-only,
        // so the headline can only be slower or equal, never faster.
        let carry_only = rates(100, 500_000_000);
        let full_unit = rates(100, 1_000_000_000);
        assert!(full_unit.rows_per_s < carry_only.rows_per_s);
    }

    #[test]
    fn an_atomic_write_leaves_no_temp_behind_on_a_failed_rename() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("record.json");
        std::fs::create_dir(&target).expect("a directory where the file belongs");
        assert!(write_atomic(&target, "{}").is_err());
        let leftovers: Vec<String> = std::fs::read_dir(dir.path())
            .expect("read_dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty(), "left {leftovers:?}");
    }
}
