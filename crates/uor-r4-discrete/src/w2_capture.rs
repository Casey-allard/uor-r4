//! W2 read-code capacity capture: replays the sealed Q4 DEV evaluation on the imported
//! device (Gate R), then records each row's production read code on DEV and on the
//! matched 1,049,856-row TRAIN prefix. Read-only: the imported device has no apply path.
//!
//! The host part (sealed-subset compare, exclusive stream writers, capture schema) is
//! unit-tested here; [`run`] needs `--features cuda` and a GPU.

use crate::q4_import::{ImportReceipt, TargetIdentity};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub const SCHEMA: &str = "uor-r4.w2-capture/1";
pub const TRAIN_BATCHES: u64 = 16_404;
pub const TRAIN_STREAM_FILE: &str = "train-stream.bin";
pub const QUERY_SIDECAR_FILE: &str = "query-sidecar.bin";
pub const GATE_R_FILE: &str = "gate-r.json";
pub const CAPTURE_FILE: &str = "capture.json";
pub const DEV_CAPTURE_FILE: &str = "dev-capture.json";
pub const FAULT_FILE: &str = "fault.json";
/// Device ABSENT cell slot, and its u16 form in `admitted_cell_slots`.
pub const ABSENT_CELL: u32 = u32::MAX;
pub const ABSENT_SLOT: u16 = u16::MAX;
pub const CELL_SLOTS: usize = 16;
pub const DETERMINISM_BATCHES: usize = 16;

/// Every key at every path in `sealed` must exist in `new` with an equal value.
/// Arrays need equal length and are compared element-wise; JSON numbers must be strictly equal (an integer never equals a float).
/// Keys present only in `new` are allowed. Returns the mismatching paths.
pub fn sealed_subset_mismatches(sealed: &Value, new: &Value) -> Vec<String> {
    let mut out = Vec::new();
    walk("", sealed, new, &mut out);
    out
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_owned()
    } else {
        format!("{path}.{key}")
    }
}

fn walk(path: &str, sealed: &Value, new: &Value, out: &mut Vec<String>) {
    match (sealed, new) {
        (Value::Object(s), Value::Object(n)) => {
            for (key, sv) in s {
                let here = join(path, key);
                match n.get(key) {
                    Some(nv) => walk(&here, sv, nv, out),
                    None => out.push(format!("{here} (missing)")),
                }
            }
        }
        (Value::Array(s), Value::Array(n)) => {
            if s.len() != n.len() {
                out.push(format!("{path} (length {} != {})", s.len(), n.len()));
                return;
            }
            for (i, (sv, nv)) in s.iter().zip(n).enumerate() {
                walk(&format!("{path}[{i}]"), sv, nv, out);
            }
        }
        (Value::Number(s), Value::Number(n)) => {
            // Strict: an integer never equals a float (as_u64/as_i64 are None for floats,
            // so a float pair falls through to exact Number equality).
            let equal = match (s.as_u64(), n.as_u64(), s.as_i64(), n.as_i64()) {
                (Some(a), Some(b), _, _) => a == b,
                (_, _, Some(a), Some(b)) => a == b,
                _ => s == n,
            };
            if !equal {
                out.push(path.to_owned());
            }
        }
        (s, n) => {
            if s != n {
                out.push(path.to_owned());
            }
        }
    }
}

/// Object keys of `new` that `sealed` lacks (row-level schema growth), sorted.
pub fn new_only_keys(sealed: &Value, new: &Value) -> Vec<String> {
    match (sealed.as_object(), new.as_object()) {
        (Some(s), Some(n)) => n.keys().filter(|k| !s.contains_key(*k)).cloned().collect(),
        _ => Vec::new(),
    }
}

/// Lowercase hex SHA-256.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// A written stream: file name relative to the capture directory, raw SHA-256, rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StreamRecord {
    pub path: String,
    pub sha256: String,
    pub rows: u64,
}

/// Exclusive-create little-endian u16 row writer; refuses an existing file.
pub struct RowWriter {
    name: String,
    file: BufWriter<File>,
    hasher: Sha256,
    width: usize,
    rows: u64,
}

impl RowWriter {
    fn create(dir: &Path, name: &str, width: usize) -> Result<Self, String> {
        let path = dir.join(name);
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| format!("cannot create {}: {e}", path.display()))?;
        Ok(Self {
            name: name.to_owned(),
            file: BufWriter::new(file),
            hasher: Sha256::new(),
            width,
            rows: 0,
        })
    }
    fn write_words(&mut self, words: &[u16]) -> Result<(), String> {
        if words.len() != self.width {
            return Err(format!("{}: row of {} words", self.name, words.len()));
        }
        for word in words {
            let bytes = word.to_le_bytes();
            self.hasher.update(bytes);
            self.file
                .write_all(&bytes)
                .map_err(|e| format!("{}: {e}", self.name))?;
        }
        self.rows += 1;
        Ok(())
    }
    pub fn finish(mut self) -> Result<StreamRecord, String> {
        self.file
            .flush()
            .map_err(|e| format!("{}: {e}", self.name))?;
        self.file
            .get_ref()
            .sync_all()
            .map_err(|e| format!("{}: {e}", self.name))?;
        let digest = self.hasher.finalize();
        Ok(StreamRecord {
            path: self.name,
            sha256: digest.iter().map(|b| format!("{b:02x}")).collect(),
            rows: self.rows,
        })
    }
}

/// TRAIN stream in the consumer verifier's format: no header, per row u16 LE code
/// then u16 LE target, in corpus order.
pub struct TrainStreamWriter(RowWriter);
impl TrainStreamWriter {
    pub fn create(dir: &Path) -> Result<Self, String> {
        RowWriter::create(dir, TRAIN_STREAM_FILE, 2).map(Self)
    }
    pub fn write_row(&mut self, code: u16, target: u32) -> Result<(), String> {
        self.0.write_words(&[code, to_u16("target", target)?])
    }
    pub fn finish(self) -> Result<StreamRecord, String> {
        self.0.finish()
    }
}

/// Query sidecar: per TRAIN row u16 LE query token (the token at target_position-1).
pub struct QuerySidecarWriter(RowWriter);
impl QuerySidecarWriter {
    pub fn create(dir: &Path) -> Result<Self, String> {
        RowWriter::create(dir, QUERY_SIDECAR_FILE, 1).map(Self)
    }
    pub fn write_row(&mut self, query_token: u32) -> Result<(), String> {
        self.0.write_words(&[to_u16("query token", query_token)?])
    }
    pub fn finish(self) -> Result<StreamRecord, String> {
        self.0.finish()
    }
}

fn to_u16(what: &str, value: u32) -> Result<u16, String> {
    u16::try_from(value)
        .ok()
        .filter(|v| (*v as usize) < crate::w2_capacity::VOCABULARY)
        .ok_or_else(|| format!("INSTRUMENT_FAULT: {what} {value} outside the vocabulary"))
}

/// Exclusive-create pretty JSON file.
pub fn write_json_new<T: Serialize>(dir: &Path, name: &str, value: &T) -> Result<(), String> {
    let path = dir.join(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("cannot create {}: {e}", path.display()))?;
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Maps one row's 16 physical cell slots to u16 (ABSENT -> 65535) and counts the
/// admitted (non-ABSENT) slots.
pub fn cell_slots(cells: &[u32]) -> Result<([u16; CELL_SLOTS], u64), String> {
    if cells.len() != CELL_SLOTS {
        return Err(format!("INSTRUMENT_FAULT: {} cell slots", cells.len()));
    }
    let mut slots = [ABSENT_SLOT; CELL_SLOTS];
    let mut admitted = 0;
    for (slot, &cell) in slots.iter_mut().zip(cells) {
        if cell != ABSENT_CELL {
            *slot = u16::try_from(cell)
                .ok()
                .filter(|c| *c != ABSENT_SLOT)
                .ok_or_else(|| format!("INSTRUMENT_FAULT: cell {cell} exceeds u16"))?;
            admitted += 1;
        }
    }
    Ok((slots, admitted))
}

#[derive(Debug, Clone, Serialize)]
pub struct GateR {
    pub pass: bool,
    pub rows_compared: u64,
    pub mismatch_count: u64,
    pub first_50_mismatches: Vec<String>,
    pub new_only_row_keys: Vec<String>,
}

impl GateR {
    pub fn from_mismatches(
        rows_compared: u64,
        mismatches: &[String],
        new_only: Vec<String>,
    ) -> Self {
        Self {
            pass: mismatches.is_empty(),
            rows_compared,
            mismatch_count: mismatches.len() as u64,
            first_50_mismatches: mismatches.iter().take(50).cloned().collect(),
            new_only_row_keys: new_only,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DevCaptureRow {
    pub row_ordinal: u64,
    pub target_position: u64,
    pub query_token: u32,
    pub target_token: u32,
    pub code: u16,
    pub admitted_cell_slots: [u16; CELL_SLOTS],
    pub top8: [u32; 8],
    pub top1_correct: u64,
    pub top8_correct: u64,
    pub admitted_cells: u64,
    pub address_rivals: u64,
    pub token_rivals: u64,
    pub token_hinge: u64,
    pub address_hinge: u64,
    pub total_loss_numerator: u64,
    pub forward_prediction: u32,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct EqualCount {
    pub rows: u64,
    pub equal: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct InputFile {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Inputs {
    pub checkpoint: InputFile,
    pub sealed_evaluation: InputFile,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct WallMillis {
    pub import: u64,
    pub gate_r: u64,
    pub dev_forward: u64,
    pub determinism: u64,
    pub train_forward: u64,
    pub token_cells: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Capture {
    pub schema: &'static str,
    pub capture_mode: &'static str,
    pub import_receipt: ImportReceipt,
    pub target_identity: TargetIdentity,
    pub target_binary_sha256: String,
    pub inputs: Inputs,
    pub gate_r: GateR,
    pub dev_rows: Vec<DevCaptureRow>,
    pub root_round_trip: EqualCount,
    pub determinism: EqualCount,
    pub token_cells: Vec<u16>,
    pub train_stream: StreamRecord,
    pub query_sidecar: StreamRecord,
    pub wall_ms: WallMillis,
}

/// DEV-phase results, written before TRAIN so a late failure does not lose them.
#[derive(Debug, Serialize)]
pub struct DevCapture<'a> {
    pub schema: &'static str,
    pub gate_r: &'a GateR,
    pub dev_rows: &'a [DevCaptureRow],
    pub root_round_trip: EqualCount,
    pub determinism: EqualCount,
}

#[derive(Debug, Clone, Serialize)]
pub struct Fault {
    pub phase: &'static str,
    pub batch: Option<u64>,
    pub row: Option<u64>,
    pub message: String,
}

/// Creates `out_dir`, which must not exist yet: the parent is created if needed, then
/// `create_dir` refuses an existing directory atomically.
pub fn create_out_dir(out_dir: &Path) -> Result<PathBuf, String> {
    if let Some(parent) = out_dir.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::create_dir(out_dir).map_err(|e| {
        format!(
            "{}: {e} (refusing an existing or uncreatable out dir)",
            out_dir.display()
        )
    })?;
    Ok(out_dir.to_path_buf())
}

/// Byte length of the sealed Q4 evaluation file this capture replays.
pub const SEALED_LEN: usize = 2_906_559;
/// SHA-256 of the sealed Q4 evaluation file this capture replays.
pub const SEALED_SHA256: &str = "4fff1d3c04ff6107666b2b8281ef2bbedf3019eeff6bff682bbbb8687bf888c8";

/// Refuses any sealed-evaluation bytes other than the pinned file.
pub fn check_sealed_pin(bytes: &[u8]) -> Result<(), String> {
    let sha = sha256_hex(bytes);
    if bytes.len() != SEALED_LEN || sha != SEALED_SHA256 {
        return Err(format!(
            "sealed evaluation is not the pinned file: {} bytes sha256 {sha} (want {SEALED_LEN} bytes sha256 {SEALED_SHA256})",
            bytes.len()
        ));
    }
    Ok(())
}

#[cfg(feature = "cuda")]
pub use gpu::run;

#[cfg(feature = "cuda")]
mod gpu {
    use super::*;
    use crate::q4_import::{prepare, ImportedDevice};
    use crate::training::{DriverConfig, PreparedDriver};
    use crate::w2_capacity::{decode_read_vector, root_vector, unpack_code, FALLBACK_CAPTURE_MODE};
    use std::time::Instant;

    /// Wall time in whole milliseconds (D32: no float types, even for timing).
    fn elapsed_ms(clock: &Instant) -> u64 {
        u64::try_from(clock.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    const READ_WIDTH: usize = 32;

    fn s<E: std::fmt::Display>(e: E) -> String {
        e.to_string()
    }

    /// `map_err` adapter: any error becomes a `fault.json` record, then its message.
    fn faulted<'a, E: std::fmt::Display>(
        dir: &'a Path,
        phase: &'static str,
        batch: Option<u64>,
        row: Option<u64>,
    ) -> impl FnOnce(E) -> String + 'a {
        move |e| fault(dir, phase, batch, row, e.to_string())
    }

    fn fault(
        dir: &Path,
        phase: &'static str,
        batch: Option<u64>,
        row: Option<u64>,
        message: String,
    ) -> String {
        let record = Fault {
            phase,
            batch,
            row,
            message: message.clone(),
        };
        if let Err(e) = write_json_new(dir, FAULT_FILE, &record) {
            return format!("{message}; also could not write {FAULT_FILE}: {e}");
        }
        message
    }

    struct DevPass {
        vectors: Vec<Vec<i32>>,
        predictions: Vec<Vec<u32>>,
        cells: Vec<Vec<u32>>,
    }

    /// Runs the whole capture. Writes `gate-r.json` first; nothing else runs unless
    /// Gate R passes. Any instrument fault writes `fault.json` and stops.
    pub fn run(checkpoint: &Path, sealed: &Path, out_dir: &Path) -> Result<Capture, String> {
        // The sealed file is pinned before the out dir exists or any GPU work starts.
        let sealed_bytes =
            std::fs::read(sealed).map_err(|e| format!("{}: {e}", sealed.display()))?;
        check_sealed_pin(&sealed_bytes)?;
        let dir = create_out_dir(out_dir)?;
        let mut wall = WallMillis::default();
        let clock = Instant::now();

        // 1. Import and driver inputs.
        let checkpoint_bytes =
            std::fs::read(checkpoint).map_err(|e| format!("{}: {e}", checkpoint.display()))?;
        let inputs = Inputs {
            checkpoint: InputFile {
                path: checkpoint.display().to_string(),
                sha256: sha256_hex(&checkpoint_bytes),
            },
            sealed_evaluation: InputFile {
                path: sealed.display().to_string(),
                sha256: sha256_hex(&sealed_bytes),
            },
        };
        let target = TargetIdentity::compiled()?;
        let model: ImportedDevice = prepare(&checkpoint_bytes, &target)?
            .into_device(0)
            .map_err(s)?;
        let wrapper: Value = serde_json::from_slice(&checkpoint_bytes).map_err(s)?;
        let config_value = wrapper
            .pointer("/checkpoint/lineage/config")
            .cloned()
            .ok_or("checkpoint lacks .checkpoint.lineage.config")?;
        let config: DriverConfig = serde_json::from_value(config_value).map_err(|e| {
            format!("Q4 driver config does not deserialize under today's DriverConfig: {e}")
        })?;
        config
            .validate()
            .map_err(|e| format!("Q4 driver config does not validate under today's code: {e}"))?;
        let expected_next = wrapper
            .pointer("/checkpoint/cursor/next_target")
            .and_then(Value::as_u64)
            .ok_or("checkpoint lacks .checkpoint.cursor.next_target")?;
        let driver =
            PreparedDriver::load(config).map_err(|e| format!("PreparedDriver::load: {e}"))?;
        let spec = driver.config().spec;
        let batch_rows = driver.config().batch_rows;
        let sealed_value: Value = serde_json::from_slice(&sealed_bytes).map_err(s)?;
        let sealed_eval = sealed_value
            .get("evaluation")
            .ok_or("sealed file lacks .evaluation")?;
        wall.import = elapsed_ms(&clock);

        // 2. Gate R: the sealed evaluator loop over DEV with the imported device.
        let clock = Instant::now();
        let dev = driver.dev();
        let mut evaluations = Vec::new();
        let mut first = 0u64;
        while first < dev.eligible_len() as u64 {
            let batch = dev.batch(first, spec.context, batch_rows).map_err(s)?;
            let carried = model.carry_windows(&batch.windows).map_err(s)?;
            let mut evaluation = model.evaluate(&carried, &batch.targets).map_err(s)?;
            for ((row, position), target) in evaluation
                .rows
                .iter_mut()
                .zip(&batch.target_positions)
                .zip(&batch.targets)
            {
                row.target_position = Some(*position);
                row.target_token = Some(*target);
            }
            evaluations.push(evaluation);
            first = batch.next_target;
        }
        let aggregate = model.aggregate_evaluations(&evaluations).map_err(s)?;
        let reference = crate::device::Device::count_reference(
            spec.vocabulary,
            &driver.train().eligible_targets(),
            &dev.eligible_targets(),
            model.ordinal(),
        )
        .map_err(s)?;
        let epoch = model.epoch().map_err(s)?;
        let new_eval = serde_json::json!({
            "batches": serde_json::to_value(&evaluations).map_err(s)?,
            "aggregate": serde_json::to_value(&aggregate).map_err(s)?,
            "count_reference": serde_json::to_value(&reference).map_err(s)?,
            "model_epoch": epoch,
        });
        let mut sealed_subset = serde_json::Map::new();
        let mut mismatches = Vec::new();
        for key in ["batches", "aggregate", "count_reference", "model_epoch"] {
            match sealed_eval.get(key) {
                Some(value) => {
                    sealed_subset.insert(key.to_owned(), value.clone());
                }
                None => mismatches.push(format!("sealed evaluation lacks {key}")),
            }
        }
        mismatches.extend(sealed_subset_mismatches(
            &Value::Object(sealed_subset),
            &new_eval,
        ));
        match sealed_eval
            .pointer("/model_bindings/geometry_sha256")
            .and_then(Value::as_str)
        {
            Some(geometry) if geometry == target.geometry_sha256 => {}
            Some(geometry) => mismatches.push(format!(
                "model_bindings.geometry_sha256 (sealed {geometry} != compiled {})",
                target.geometry_sha256
            )),
            None => mismatches.push("model_bindings.geometry_sha256 (missing in sealed)".into()),
        }
        let new_only = match (
            sealed_eval.pointer("/batches/0/rows/0"),
            new_eval.pointer("/batches/0/rows/0"),
        ) {
            (Some(s), Some(n)) => new_only_keys(s, n),
            _ => Vec::new(),
        };
        let rows_compared = evaluations.iter().map(|e| e.rows.len() as u64).sum();
        let gate_r = GateR::from_mismatches(rows_compared, &mismatches, new_only);
        write_json_new(&dir, GATE_R_FILE, &gate_r)?;
        if !gate_r.pass {
            return Err(format!(
                "Gate R failed: {} mismatches over {rows_compared} rows; first: {:?}",
                gate_r.mismatch_count,
                gate_r.first_50_mismatches.first()
            ));
        }
        wall.gate_r = elapsed_ms(&clock);
        eprintln!(
            "w2-capacity: Gate R passed over {rows_compared} rows in {} ms",
            wall.gate_r
        );

        // 3. DEV production forward over the same batches.
        let clock = Instant::now();
        let tokens = dev.tokens();
        let mut dev_rows = Vec::with_capacity(rows_compared as usize);
        let mut round_trip = EqualCount::default();
        let mut kept = DevPass {
            vectors: Vec::new(),
            predictions: Vec::new(),
            cells: Vec::new(),
        };
        let mut first = 0u64;
        for (b, evaluation) in evaluations.iter().enumerate() {
            let at = |phase: &'static str| faulted(&dir, phase, Some(b as u64), None);
            let batch = dev
                .batch(first, spec.context, batch_rows)
                .map_err(at("dev_forward"))?;
            let carried = model
                .carry_windows(&batch.windows)
                .map_err(at("dev_forward"))?;
            let forward = model
                .forward(&carried, &batch.targets)
                .map_err(at("dev_forward"))?;
            let vectors = forward.vectors().map_err(at("dev_forward"))?;
            let rows = batch.targets.len();
            if vectors.len() != rows * READ_WIDTH
                || forward.predictions.len() != rows
                || forward.admitted_cells.len() != rows * CELL_SLOTS
                || evaluation.rows.len() != rows
            {
                return Err(fault(
                    &dir,
                    "dev_forward",
                    Some(b as u64),
                    None,
                    "INSTRUMENT_FAULT: forward/evaluate row shapes differ".into(),
                ));
            }
            for i in 0..rows {
                let ordinal = dev_rows.len() as u64;
                let q: &[i32; READ_WIDTH] = vectors[i * READ_WIDTH..(i + 1) * READ_WIDTH]
                    .try_into()
                    .unwrap();
                let code = decode_read_vector(q).map_err(|e| {
                    fault(
                        &dir,
                        "dev_forward",
                        Some(b as u64),
                        Some(ordinal),
                        e.to_string(),
                    )
                })?;
                let row = &evaluation.rows[i];
                let (slots, admitted) =
                    cell_slots(&forward.admitted_cells[i * CELL_SLOTS..(i + 1) * CELL_SLOTS])
                        .map_err(|e| {
                            fault(&dir, "dev_forward", Some(b as u64), Some(ordinal), e)
                        })?;
                if forward.predictions[i] != row.top8[0] {
                    return Err(fault(
                        &dir,
                        "dev_forward",
                        Some(b as u64),
                        Some(ordinal),
                        format!(
                            "INSTRUMENT_FAULT: forward prediction {} != evaluate top8[0] {}",
                            forward.predictions[i], row.top8[0]
                        ),
                    ));
                }
                if admitted != row.admitted_cells {
                    return Err(fault(
                        &dir,
                        "dev_forward",
                        Some(b as u64),
                        Some(ordinal),
                        format!(
                            "INSTRUMENT_FAULT: forward admits {admitted} cells, evaluate {}",
                            row.admitted_cells
                        ),
                    ));
                }
                // Checks code packing/unpacking against the captured vector; decode is an
                // exact-match decode, so this is a packing check, not a read-quality check.
                round_trip.rows += 1;
                if root_vector(unpack_code(code)) == *q {
                    round_trip.equal += 1;
                }
                let position = batch.target_positions[i];
                dev_rows.push(DevCaptureRow {
                    row_ordinal: ordinal,
                    target_position: position,
                    query_token: tokens[position as usize - 1],
                    target_token: batch.targets[i],
                    code,
                    admitted_cell_slots: slots,
                    top8: row.top8,
                    top1_correct: row.top1_correct,
                    top8_correct: row.top8_correct,
                    admitted_cells: row.admitted_cells,
                    address_rivals: row.address_rivals,
                    token_rivals: row.token_rivals,
                    token_hinge: row.token_hinge,
                    address_hinge: row.address_hinge,
                    total_loss_numerator: row.total_loss_numerator,
                    forward_prediction: forward.predictions[i],
                });
            }
            if b < DETERMINISM_BATCHES {
                kept.vectors.push(vectors);
                kept.predictions.push(forward.predictions.clone());
                kept.cells.push(forward.admitted_cells.clone());
            }
            first = batch.next_target;
        }
        wall.dev_forward = elapsed_ms(&clock);

        // Determinism: re-run forward on the first 16 batches.
        let clock = Instant::now();
        let mut determinism = EqualCount::default();
        let mut first = 0u64;
        for b in 0..kept.vectors.len() {
            let at = || faulted(&dir, "determinism", Some(b as u64), None);
            let batch = dev.batch(first, spec.context, batch_rows).map_err(at())?;
            let carried = model.carry_windows(&batch.windows).map_err(at())?;
            let forward = model.forward(&carried, &batch.targets).map_err(at())?;
            let vectors = forward.vectors().map_err(at())?;
            for i in 0..batch.targets.len() {
                determinism.rows += 1;
                let same = vectors[i * READ_WIDTH..(i + 1) * READ_WIDTH]
                    == kept.vectors[b][i * READ_WIDTH..(i + 1) * READ_WIDTH]
                    && forward.predictions[i] == kept.predictions[b][i]
                    && forward.admitted_cells[i * CELL_SLOTS..(i + 1) * CELL_SLOTS]
                        == kept.cells[b][i * CELL_SLOTS..(i + 1) * CELL_SLOTS];
                if same {
                    determinism.equal += 1;
                }
            }
            first = batch.next_target;
        }
        wall.determinism = elapsed_ms(&clock);
        eprintln!(
            "w2-capacity: DEV rows {} round-trip {}/{} determinism {}/{}",
            dev_rows.len(),
            round_trip.equal,
            round_trip.rows,
            determinism.equal,
            determinism.rows
        );
        write_json_new(
            &dir,
            DEV_CAPTURE_FILE,
            &DevCapture {
                schema: SCHEMA,
                gate_r: &gate_r,
                dev_rows: &dev_rows,
                root_round_trip: round_trip,
                determinism,
            },
        )
        .map_err(|e| fault(&dir, "dev_capture", None, None, e))?;
        if determinism.equal != determinism.rows || determinism.rows < 1_000 {
            return Err(fault(
                &dir,
                "determinism",
                None,
                None,
                format!(
                    "INSTRUMENT_FAULT: determinism {}/{} rows equal (need all, and at least 1000 rows)",
                    determinism.equal, determinism.rows
                ),
            ));
        }
        if round_trip.equal != round_trip.rows {
            return Err(fault(
                &dir,
                "root_round_trip",
                None,
                None,
                format!(
                    "INSTRUMENT_FAULT: root round-trip {}/{} rows equal",
                    round_trip.equal, round_trip.rows
                ),
            ));
        }

        // 4. TRAIN production forward over exactly 16,404 batches from target 0.
        let clock = Instant::now();
        let train = driver.train();
        let train_tokens = train.tokens();
        let mut stream =
            TrainStreamWriter::create(&dir).map_err(faulted(&dir, "train_forward", None, None))?;
        let mut sidecar =
            QuerySidecarWriter::create(&dir).map_err(faulted(&dir, "train_forward", None, None))?;
        let mut first = 0u64;
        let mut done_rows = 0u64;
        for b in 0..TRAIN_BATCHES {
            let at = || faulted(&dir, "train_forward", Some(b), Some(done_rows));
            let batch = train.batch(first, spec.context, batch_rows).map_err(at())?;
            let carried = model.carry_windows(&batch.windows).map_err(at())?;
            let forward = model.forward(&carried, &batch.targets).map_err(at())?;
            let vectors = forward.vectors().map_err(at())?;
            if vectors.len() != batch.targets.len() * READ_WIDTH {
                return Err(fault(
                    &dir,
                    "train_forward",
                    Some(b),
                    Some(done_rows),
                    "INSTRUMENT_FAULT: vector shape".into(),
                ));
            }
            for (i, &target) in batch.targets.iter().enumerate() {
                let row = Some(done_rows + i as u64);
                let q: &[i32; READ_WIDTH] = vectors[i * READ_WIDTH..(i + 1) * READ_WIDTH]
                    .try_into()
                    .unwrap();
                let code =
                    decode_read_vector(q).map_err(faulted(&dir, "train_forward", Some(b), row))?;
                stream.write_row(code, target).map_err(faulted(
                    &dir,
                    "train_forward",
                    Some(b),
                    row,
                ))?;
                let query = train_tokens[batch.target_positions[i] as usize - 1];
                sidecar
                    .write_row(query)
                    .map_err(faulted(&dir, "train_forward", Some(b), row))?;
            }
            done_rows += batch.targets.len() as u64;
            first = batch.next_target;
            if (b + 1) % 1000 == 0 {
                eprintln!(
                    "w2-capacity: TRAIN batch {} / {TRAIN_BATCHES}, {} ms",
                    b + 1,
                    elapsed_ms(&clock)
                );
            }
        }
        let train_stream =
            stream
                .finish()
                .map_err(faulted(&dir, "train_forward", None, Some(done_rows)))?;
        let query_sidecar =
            sidecar
                .finish()
                .map_err(faulted(&dir, "train_forward", None, Some(done_rows)))?;
        if train_stream.rows != crate::w2_capacity::MATCHED_TRAIN_ROWS as u64
            || first != expected_next
        {
            return Err(fault(&dir, "train_forward", None, None, format!(
                "INSTRUMENT_FAULT: TRAIN rows {} (want {}), next_target {first} (checkpoint {expected_next})",
                train_stream.rows,
                crate::w2_capacity::MATCHED_TRAIN_ROWS
            )));
        }
        wall.train_forward = elapsed_ms(&clock);

        // 5. Token cells.
        let clock = Instant::now();
        let token_cells =
            model
                .export_token_cells()
                .map_err(faulted(&dir, "token_cells", None, None))?;
        if token_cells.len() != spec.vocabulary as usize {
            return Err(fault(
                &dir,
                "token_cells",
                None,
                None,
                format!("INSTRUMENT_FAULT: {} token cells", token_cells.len()),
            ));
        }
        wall.token_cells = elapsed_ms(&clock);

        // 6. capture.json.
        let exe = std::env::current_exe().map_err(faulted(&dir, "capture_json", None, None))?;
        let exe_bytes = std::fs::read(&exe)
            .map_err(|e| format!("{}: {e}", exe.display()))
            .map_err(faulted(&dir, "capture_json", None, None))?;
        let capture = Capture {
            schema: SCHEMA,
            capture_mode: FALLBACK_CAPTURE_MODE,
            import_receipt: model.receipt().clone(),
            target_identity: target,
            target_binary_sha256: sha256_hex(&exe_bytes),
            inputs,
            gate_r,
            dev_rows,
            root_round_trip: round_trip,
            determinism,
            token_cells,
            train_stream,
            query_sidecar,
            wall_ms: wall,
        };
        write_json_new(&dir, CAPTURE_FILE, &capture).map_err(faulted(
            &dir,
            "capture_json",
            None,
            None,
        ))?;
        Ok(capture)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("uor-w2-capture-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn subset_equal_has_no_mismatch() {
        let v = json!({"batches": [{"rows": [{"top8": [1, 2], "token_hinge": 3}]}], "model_epoch": 407});
        assert!(sealed_subset_mismatches(&v, &v).is_empty());
    }

    #[test]
    fn subset_changed_nested_number_is_named() {
        let sealed = json!({"batches": [{"rows": [{"top8": [1, 2]}, {"top8": [5, 6]}]}]});
        let new = json!({"batches": [{"rows": [{"top8": [1, 2]}, {"top8": [5, 7]}]}]});
        assert_eq!(
            sealed_subset_mismatches(&sealed, &new),
            vec!["batches[0].rows[1].top8[1]"]
        );
    }

    #[test]
    fn subset_missing_key_is_a_mismatch() {
        let sealed = json!({"aggregate": {"scored_rows": 14287, "top1_correct": 9}});
        let new = json!({"aggregate": {"scored_rows": 14287}});
        assert_eq!(
            sealed_subset_mismatches(&sealed, &new),
            vec!["aggregate.top1_correct (missing)"]
        );
    }

    #[test]
    fn subset_extra_new_key_is_allowed_and_listed() {
        let sealed = json!({"token_hinge": 3});
        let new = json!({"token_hinge": 3, "target_position": 10, "cell_rank_hinge": 0});
        assert!(sealed_subset_mismatches(&sealed, &new).is_empty());
        assert_eq!(
            new_only_keys(&sealed, &new),
            vec!["cell_rank_hinge", "target_position"]
        );
    }

    #[test]
    fn subset_array_length_and_integer_compare() {
        let sealed = json!({"a": [1, 2, 3], "b": 5});
        let new = json!({"a": [1, 2], "b": 5u64});
        assert_eq!(
            sealed_subset_mismatches(&sealed, &new),
            vec!["a (length 3 != 2)"]
        );
    }

    #[test]
    fn writers_emit_le_rows_hash_and_refuse_existing() {
        let dir = scratch("writers");
        create_out_dir(&dir).unwrap();
        assert!(create_out_dir(&dir).is_err());
        let mut stream = TrainStreamWriter::create(&dir).unwrap();
        stream.write_row(0x0102, 0x0a0b).unwrap();
        stream.write_row(0xfffe, 4095).unwrap();
        assert!(stream.write_row(1, 4096).is_err());
        let record = stream.finish().unwrap();
        let bytes = std::fs::read(dir.join(TRAIN_STREAM_FILE)).unwrap();
        assert_eq!(bytes, [0x02, 0x01, 0x0b, 0x0a, 0xfe, 0xff, 0xff, 0x0f]);
        assert_eq!(record.rows, 2);
        assert_eq!(record.sha256, sha256_hex(&bytes));
        let mut sidecar = QuerySidecarWriter::create(&dir).unwrap();
        sidecar.write_row(7).unwrap();
        let record = sidecar.finish().unwrap();
        assert_eq!(std::fs::read(dir.join(QUERY_SIDECAR_FILE)).unwrap(), [7, 0]);
        assert_eq!(record.rows, 1);
        assert!(TrainStreamWriter::create(&dir).is_err());
        assert!(write_json_new(&dir, GATE_R_FILE, &json!({})).is_ok());
        assert!(write_json_new(&dir, GATE_R_FILE, &json!({})).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn cell_slots_map_absent_and_count() {
        let mut cells = [ABSENT_CELL; CELL_SLOTS];
        cells[0] = 3;
        cells[5] = 15;
        let (slots, admitted) = cell_slots(&cells).unwrap();
        assert_eq!(admitted, 2);
        assert_eq!(slots[0], 3);
        assert_eq!(slots[5], 15);
        assert_eq!(slots[1], ABSENT_SLOT);
        assert!(cell_slots(&cells[..15]).is_err());
    }

    #[test]
    fn create_out_dir_refuses_existing_dir() {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../local/test-scratch")
            .join(format!("w2-capture-outdir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("nested").join("out");
        assert_eq!(create_out_dir(&dir).unwrap(), dir);
        assert!(create_out_dir(&dir).is_err());
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn sealed_pin_refuses_a_changed_copy_and_accepts_the_real_file() {
        let Some(inputs) = std::env::var_os("UOR_W2_Q4") else {
            eprintln!("UOR_W2_Q4 unset; skipped");
            return;
        };
        let mut bytes =
            std::fs::read(PathBuf::from(inputs).join("q4-evaluation-16404.json")).unwrap();
        check_sealed_pin(&bytes).unwrap();
        bytes[0] ^= 1;
        assert!(check_sealed_pin(&bytes).is_err());
        bytes[0] ^= 1;
        bytes.pop();
        assert!(check_sealed_pin(&bytes).is_err());
        assert!(check_sealed_pin(b"").is_err());
    }

    /// G3 host config check; runs only with UOR_W2_Q4 set to the w2-inputs folder.
    /// Changes the process working directory to the worktree root (the config's
    /// corpus paths are relative to it), so run it filtered on its own.
    #[test]
    fn q4_config_loads_under_todays_driver() {
        let Some(inputs) = std::env::var_os("UOR_W2_Q4") else {
            eprintln!("UOR_W2_Q4 unset; skipped");
            return;
        };
        let inputs = PathBuf::from(inputs);
        let root = inputs.join("../../..").canonicalize().unwrap();
        let wrapper: Value = serde_json::from_slice(
            &std::fs::read(inputs.join("q4-checkpoint-16404.json")).unwrap(),
        )
        .unwrap();
        let config: crate::training::DriverConfig = serde_json::from_value(
            wrapper
                .pointer("/checkpoint/lineage/config")
                .unwrap()
                .clone(),
        )
        .unwrap();
        config.validate().unwrap();
        std::env::set_current_dir(&root).unwrap();
        let driver = crate::training::PreparedDriver::load(config).unwrap();
        assert_eq!(driver.dev().eligible_len(), 14_287);
        assert_eq!(driver.train().eligible_len(), 2_425_925);
        let spec = driver.config().spec;
        let rows = driver.config().batch_rows;
        assert_eq!(rows, 64);
        let mut first = 0;
        let mut total = 0;
        for _ in 0..TRAIN_BATCHES {
            let batch = driver.train().batch(first, spec.context, rows).unwrap();
            total += batch.targets.len();
            first = batch.next_target;
        }
        assert_eq!(total, crate::w2_capacity::MATCHED_TRAIN_ROWS);
        assert_eq!(first, 1_049_856);
    }
}
