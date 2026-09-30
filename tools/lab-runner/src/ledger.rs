//! Immutable, explicitly migrated ledger. The JSON total is only a derived view.
//! Historical snapshots never reset later charges. Every new attempt is charged
//! once by (job_id, attempt_id), including after a crash before finalization.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::{invalid, utc_now_iso, Result};

pub const CHARGE_SCHEMA: &str = "uor-r4.model-time-charge/2";
pub const BASELINE_SCHEMA: &str = "uor-r4.ledger-baseline/1";
pub const EXTENSION_SCHEMA: &str = "uor-r4.model-time-extension/2";
pub const IMPORT_SCHEMA: &str = "uor-r4.ledger-legacy-import/1";
pub const RESOURCE_CHARGE_SCHEMA: &str = "uor-r4.resource-charge/1";
pub const MODEL_TIME_FILE: &str = "model-time.json";
pub const BASELINE_FILE: &str = "ledger-baseline.json";
pub const LOCK_FILE: &str = "model-time.json.lock";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerState {
    pub cumulative_ms: u64,
    pub limit_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoveredRecord {
    pub file: String,
    pub sha256: String,
}

/// A reviewed reconciliation, not an automatically selected old snapshot.
/// covered_records names exactly the historical bytes already included in totals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationBaseline {
    pub schema: String,
    pub baseline_id: String,
    pub recorded_utc: String,
    pub cumulative_ms: u64,
    pub limit_ms: u64,
    pub covered_records: Vec<CoveredRecord>,
    pub authority: String,
    pub rationale: String,
    /// Watch only at NEW admission. Internal accounting never requires this
    /// legacy volume to remain mounted after migration.
    #[serde(default)]
    pub legacy_source: Option<PathBuf>,
    #[serde(default)]
    pub legacy_snapshot_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChargeRecord {
    pub schema: String,
    pub event_id: String,
    pub recorded_utc: String,
    pub lab: String,
    pub job_id: String,
    pub attempt_id: String,
    pub charged_ms: u64,
    pub wall_s: u64,
    pub outcome: String,
    pub runner: String,
    /// Absent only on preserved older records; their bytes are never rewritten.
    #[serde(default)]
    pub accounting: Option<CostMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostCategory {
    JobExecution,
    Preparation,
    Review,
    Storage,
    Build,
    Delivery,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Measurement {
    Measured,
    Estimated,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorrectionReference {
    pub file: String,
    pub sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CostMetadata {
    pub category: CostCategory,
    pub measurement: Measurement,
    pub correction_of: Option<CorrectionReference>,
}
/// Generic programme elapsed-time accounting. Storage here is time spent on
/// storage work; physical byte reservations remain the host policy's concern.
/// Corrections append signed deltas to an explicitly hash-bound earlier event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceChargeRecord {
    pub schema: String,
    pub event_id: String,
    pub recorded_utc: String,
    pub lab: String,
    pub adjustment_ms: i64,
    pub accounting: CostMetadata,
    pub authority: String,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionRecord {
    pub schema: String,
    pub event_id: String,
    pub recorded_utc: String,
    pub increment_ms: u64,
    pub authority: String,
    pub rationale: String,
}

/// Explicitly account a late legacy receipt without changing the baseline.
/// Both amounts are reviewed contributions not already covered by the baseline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyImport {
    pub schema: String,
    pub event_id: String,
    pub recorded_utc: String,
    pub source: CoveredRecord,
    pub charged_ms: u64,
    pub increment_ms: u64,
    pub authority: String,
    pub rationale: String,
    /// A changed same-name external receipt is copied under a new internal
    /// filename; this hash-linked observation updates the watched external name.
    #[serde(default)]
    pub legacy_observed: Option<LegacyObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyObservation {
    pub file: String,
    pub sha256: String,
    /// None adds a newly observed filename; Some advances an exact prior hash.
    pub previous_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacySnapshotObservation {
    pub schema: String,
    pub event_id: String,
    pub recorded_utc: String,
    pub previous_sha256: String,
    pub sha256: String,
    pub reviewed_internal_state: LedgerState,
    pub authority: String,
    pub rationale: String,
}

struct LedgerLock {
    _file: fs::File,
}
fn acquire_lock(dir: &Path) -> Result<LedgerLock> {
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(LOCK_FILE))?;
    for _ in 0..100 {
        match file.try_lock() {
            Ok(()) => return Ok(LedgerLock { _file: file }),
            Err(fs::TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(50)),
            Err(fs::TryLockError::Error(e)) => return Err(e.into()),
        }
    }
    Err(invalid("ledger lock remained busy; no record was changed"))
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn nonempty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(invalid(format!("{field} must be nonempty")));
    }
    Ok(())
}
fn add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b)
        .ok_or_else(|| invalid("ledger arithmetic overflow"))
}
fn valid_covered(record: &CoveredRecord) -> Result<()> {
    if Path::new(&record.file).file_name().and_then(|v| v.to_str()) != Some(&record.file)
        || record.file == "."
        || record.file == ".."
        || !(record.file.starts_with("charge-") || record.file.starts_with("extension-"))
        || !record.file.ends_with(".json")
    {
        return Err(invalid(
            "covered legacy filename must be one charge/extension path component",
        ));
    }
    if record.sha256.len() != 64
        || !record
            .sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(invalid("covered record needs lowercase SHA-256"));
    }
    Ok(())
}
fn verify_covered(dir: &Path, record: &CoveredRecord) -> Result<()> {
    valid_covered(record)?;
    let path = dir.join(&record.file);
    if fs::symlink_metadata(&path)?.file_type().is_symlink() {
        return Err(invalid("legacy receipt cannot be a symlink"));
    }
    if sha256(&fs::read(path)?) != record.sha256 {
        return Err(invalid(format!("legacy receipt changed: {}", record.file)));
    }
    Ok(())
}

static TEMP_ID: AtomicU64 = AtomicU64::new(0);
/// Publish complete immutable bytes atomically without replacing an existing record.
/// A crash may leave a dot-prefixed temporary, which readers ignore.
fn immutable_write(path: &Path, bytes: &[u8]) -> Result<()> {
    if path.exists() {
        if fs::read(path)? == bytes {
            return Ok(());
        }
        return Err(invalid(format!(
            "immutable ledger record differs: {}",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .ok_or_else(|| invalid("record has no parent"))?;
    let temp = parent.join(format!(
        ".ledger-{}-{}",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    let result = fs::hard_link(&temp, path);
    let _ = fs::remove_file(&temp);
    match result {
        Ok(()) => {
            fs::File::open(parent)?.sync_all()?;
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && fs::read(path)? == bytes => {
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
fn write_view(dir: &Path, state: LedgerState) -> Result<()> {
    crate::atomic_write(
        &dir.join(MODEL_TIME_FILE),
        &serde_json::to_vec_pretty(&state)?,
    )
}
fn legacy_files(dir: &Path) -> Result<Vec<(String, serde_json::Value)>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if (name.starts_with("charge-") || name.starts_with("extension-"))
            && name.ends_with(".json")
        {
            if !entry.file_type()?.is_file() {
                return Err(invalid("ledger receipt must be a regular file"));
            }
            files.push((name, serde_json::from_slice(&fs::read(entry.path())?)?));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

pub fn migrate(dir: &Path, baseline: &MigrationBaseline) -> Result<LedgerState> {
    fs::create_dir_all(dir)?;
    let _lock = acquire_lock(dir)?;
    if baseline.schema != BASELINE_SCHEMA {
        return Err(invalid("unsupported baseline schema"));
    }
    nonempty(&baseline.baseline_id, "baseline_id")?;
    nonempty(&baseline.authority, "authority")?;
    nonempty(&baseline.rationale, "rationale")?;
    nonempty(&baseline.recorded_utc, "recorded_utc")?;
    validate_legacy_source(dir, baseline.legacy_source.as_deref())?;
    if baseline.legacy_source.is_some() {
        let hash = baseline
            .legacy_snapshot_sha256
            .as_deref()
            .ok_or_else(|| invalid("legacy watch requires original model-time snapshot hash"))?;
        verify_snapshot_copy(dir, hash)?;
    }
    let mut covered = BTreeSet::new();
    for item in &baseline.covered_records {
        verify_covered(dir, item)?;
        if !covered.insert(item.file.as_str()) {
            return Err(invalid("duplicate covered legacy record"));
        }
        let value: serde_json::Value = serde_json::from_slice(&fs::read(dir.join(&item.file))?)?;
        if matches!(
            value.get("schema").and_then(|v| v.as_str()),
            Some(CHARGE_SCHEMA | EXTENSION_SCHEMA | RESOURCE_CHARGE_SCHEMA)
        ) {
            return Err(invalid("a baseline must not absorb typed v2 events"));
        }
    }
    for (file, value) in legacy_files(dir)? {
        if !matches!(
            value.get("schema").and_then(|v| v.as_str()),
            Some(CHARGE_SCHEMA | EXTENSION_SCHEMA | RESOURCE_CHARGE_SCHEMA)
        ) && !covered.contains(file.as_str())
        {
            return Err(invalid(format!(
                "baseline leaves historical receipt unmapped: {file}"
            )));
        }
    }
    immutable_write(
        &dir.join(BASELINE_FILE),
        &serde_json::to_vec_pretty(baseline)?,
    )?;
    let state = fold_records(dir)?;
    write_view(dir, state)?;
    Ok(state)
}

/// Explicit genesis for a genuinely empty ledger; not a legacy auto-migration.
pub fn initialize_empty(dir: &Path, limit_ms: u64, reason: &str) -> Result<LedgerState> {
    migrate(
        dir,
        &MigrationBaseline {
            schema: BASELINE_SCHEMA.into(),
            baseline_id: "empty-genesis".into(),
            recorded_utc: utc_now_iso(),
            cumulative_ms: 0,
            limit_ms,
            covered_records: Vec::new(),
            authority: reason.into(),
            rationale: reason.into(),
            legacy_source: None,
            legacy_snapshot_sha256: None,
        },
    )
}

pub fn fold_records(dir: &Path) -> Result<LedgerState> {
    fold_with_resource(dir, None)
}
fn fold_with_resource(dir: &Path, proposed: Option<&ResourceChargeRecord>) -> Result<LedgerState> {
    let bytes = fs::read(dir.join(BASELINE_FILE)).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            invalid("ledger needs an explicit baseline migration or empty genesis")
        } else {
            e.into()
        }
    })?;
    let baseline: MigrationBaseline = serde_json::from_slice(&bytes)?;
    if baseline.schema != BASELINE_SCHEMA {
        return Err(invalid("unsupported baseline schema"));
    }
    nonempty(&baseline.baseline_id, "baseline_id")?;
    nonempty(&baseline.authority, "baseline authority")?;
    nonempty(&baseline.rationale, "baseline rationale")?;
    let mut state = LedgerState {
        cumulative_ms: baseline.cumulative_ms,
        limit_ms: baseline.limit_ms,
    };
    let mut covered = BTreeMap::new();
    for source in baseline.covered_records {
        verify_covered(dir, &source)?;
        let value: serde_json::Value = serde_json::from_slice(&fs::read(dir.join(&source.file))?)?;
        if matches!(
            value.get("schema").and_then(|v| v.as_str()),
            Some(CHARGE_SCHEMA | EXTENSION_SCHEMA | RESOURCE_CHARGE_SCHEMA)
        ) {
            return Err(invalid("baseline cannot absorb typed v2 events"));
        }
        if covered.insert(source.file, source.sha256).is_some() {
            return Err(invalid("duplicate baseline coverage"));
        }
    }
    let mut events = BTreeMap::<String, serde_json::Value>::new();
    let mut attempts = BTreeSet::new();
    let mut bases = BTreeMap::<String, (i128, CostCategory)>::new();
    let mut resources = BTreeMap::<String, ResourceChargeRecord>::new();
    // Explicit imports map late legacy receipts; never infer amounts from *_ms keys.
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("import-") || !name.ends_with(".json") {
            continue;
        }
        let value: serde_json::Value = serde_json::from_slice(&fs::read(entry.path())?)?;
        let record: LegacyImport = serde_json::from_value(value.clone())?;
        if record.schema != IMPORT_SCHEMA {
            return Err(invalid("unsupported legacy import schema"));
        }
        nonempty(&record.event_id, "event_id")?;
        nonempty(&record.authority, "authority")?;
        nonempty(&record.rationale, "rationale")?;
        validate_legacy_observation(&record)?;
        if let Some(old) = events.insert(record.event_id.clone(), value.clone()) {
            if old != value {
                return Err(invalid("conflicting ledger event ID"));
            }
            continue;
        }
        verify_covered(dir, &record.source)?;
        if covered
            .insert(record.source.file, record.source.sha256)
            .is_some()
        {
            return Err(invalid("legacy receipt is accounted more than once"));
        }
        state.cumulative_ms = add(state.cumulative_ms, record.charged_ms)?;
        state.limit_ms = add(state.limit_ms, record.increment_ms)?;
    }
    let mut files = legacy_files(dir)?;
    if let Some(record) = proposed {
        files.push((
            resource_filename(&record.event_id),
            serde_json::to_value(record)?,
        ));
    }
    for (file, value) in files {
        match value.get("schema").and_then(|v| v.as_str()) {
            Some(CHARGE_SCHEMA) => {
                let record: ChargeRecord = serde_json::from_value(value.clone())?;
                validate_charge(&record)?;
                if let Some(old) = events.insert(record.event_id.clone(), value.clone()) {
                    if old != value {
                        return Err(invalid("conflicting ledger event ID"));
                    }
                    continue;
                }
                if !attempts.insert((record.job_id, record.attempt_id)) {
                    return Err(invalid("attempt charged under multiple event IDs"));
                }
                bases.insert(
                    record.event_id,
                    (i128::from(record.charged_ms), CostCategory::JobExecution),
                );
                state.cumulative_ms = add(state.cumulative_ms, record.charged_ms)?;
            }
            Some(RESOURCE_CHARGE_SCHEMA) => {
                let record: ResourceChargeRecord = serde_json::from_value(value.clone())?;
                validate_resource(&record)?;
                if let Some(old) = events.insert(record.event_id.clone(), value.clone()) {
                    if old != value {
                        return Err(invalid("conflicting ledger event ID"));
                    }
                    continue;
                }
                if record.accounting.correction_of.is_none() {
                    let amount = u64::try_from(record.adjustment_ms)
                        .map_err(|_| invalid("new resource charge cannot be negative"))?;
                    state.cumulative_ms = add(state.cumulative_ms, amount)?;
                    bases.insert(
                        record.event_id.clone(),
                        (i128::from(amount), record.accounting.category),
                    );
                }
                resources.insert(record.event_id.clone(), record);
            }
            Some(EXTENSION_SCHEMA) => {
                let record: ExtensionRecord = serde_json::from_value(value.clone())?;
                validate_extension(&record)?;
                if let Some(old) = events.insert(record.event_id.clone(), value.clone()) {
                    if old != value {
                        return Err(invalid("conflicting ledger event ID"));
                    }
                    continue;
                }
                state.limit_ms = add(state.limit_ms, record.increment_ms)?;
            }
            _ if covered.contains_key(&file) => {}
            _ => {
                return Err(invalid(format!(
                    "unmapped legacy receipt {file}; add an explicit import"
                )))
            }
        }
    }
    let mut adjustments = 0i128;
    let mut root_deltas = BTreeMap::<String, i128>::new();
    for record in resources
        .values()
        .filter(|r| r.accounting.correction_of.is_some())
    {
        let root = correction_root(dir, record, &resources, &events)?;
        let (_, category) = bases
            .get(&root)
            .ok_or_else(|| invalid("correction has no accounted original charge"))?;
        if *category != record.accounting.category {
            return Err(invalid("correction category differs from original charge"));
        }
        let delta = i128::from(record.adjustment_ms);
        adjustments = adjustments
            .checked_add(delta)
            .ok_or_else(|| invalid("correction arithmetic overflow"))?;
        let sum = root_deltas.entry(root).or_default();
        *sum = sum
            .checked_add(delta)
            .ok_or_else(|| invalid("correction arithmetic overflow"))?;
    }
    for (root, delta) in root_deltas {
        if bases[&root]
            .0
            .checked_add(delta)
            .is_none_or(|total| total < 0)
        {
            return Err(invalid(
                "corrections would make an original charge negative",
            ));
        }
    }
    state.cumulative_ms = u64::try_from(
        i128::from(state.cumulative_ms)
            .checked_add(adjustments)
            .ok_or_else(|| invalid("ledger correction overflow"))?,
    )
    .map_err(|_| invalid("corrected cumulative charge outside u64 bounds"))?;
    Ok(state)
}

fn resource_filename(event_id: &str) -> String {
    format!("charge-resource-v1-{}.json", sha256(event_id.as_bytes()))
}
fn validate_resource(record: &ResourceChargeRecord) -> Result<()> {
    if record.schema != RESOURCE_CHARGE_SCHEMA {
        return Err(invalid("unsupported resource charge schema"));
    }
    for (value, name) in [
        (&record.event_id, "event_id"),
        (&record.recorded_utc, "recorded_utc"),
        (&record.lab, "lab"),
        (&record.authority, "authority"),
        (&record.rationale, "rationale"),
    ] {
        nonempty(value, name)?;
    }
    if record.accounting.correction_of.is_none()
        && (record.adjustment_ms < 0 || record.accounting.category == CostCategory::JobExecution)
    {
        return Err(invalid(
            "new programme charges must be nonnegative; jobs use immutable attempt charging",
        ));
    }
    if let Some(reference) = &record.accounting.correction_of {
        valid_covered(&CoveredRecord {
            file: reference.file.clone(),
            sha256: reference.sha256.clone(),
        })?;
        if !reference.file.starts_with("charge-") {
            return Err(invalid(
                "corrections must reference a charge, not an allowance extension",
            ));
        }
    }
    Ok(())
}
fn correction_root(
    dir: &Path,
    record: &ResourceChargeRecord,
    resources: &BTreeMap<String, ResourceChargeRecord>,
    events: &BTreeMap<String, serde_json::Value>,
) -> Result<String> {
    let mut cursor = record;
    let mut visited = BTreeSet::new();
    loop {
        if !visited.insert(cursor.event_id.clone()) {
            return Err(invalid("cyclic ledger correction links"));
        }
        let Some(reference) = &cursor.accounting.correction_of else {
            return Ok(cursor.event_id.clone());
        };
        verify_covered(
            dir,
            &CoveredRecord {
                file: reference.file.clone(),
                sha256: reference.sha256.clone(),
            },
        )?;
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join(&reference.file))?)?;
        let target = value["event_id"]
            .as_str()
            .ok_or_else(|| invalid("correction target lacks typed event identity"))?;
        if events.get(target) != Some(&value) {
            return Err(invalid(
                "correction target is not an accounted immutable event",
            ));
        }
        match value["schema"].as_str() {
            Some(CHARGE_SCHEMA) => return Ok(target.into()),
            Some(RESOURCE_CHARGE_SCHEMA) => {
                cursor = resources
                    .get(target)
                    .ok_or_else(|| invalid("resource correction target missing"))?
            }
            _ => {
                return Err(invalid(
                    "correction target must be a typed charge; reconcile legacy amounts explicitly",
                ))
            }
        }
    }
}
fn charge_id(job: &str, attempt: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(job.as_bytes());
    digest.update([0]);
    digest.update(attempt.as_bytes());
    format!("{:x}", digest.finalize())
}
fn validate_charge(record: &ChargeRecord) -> Result<()> {
    if record.schema != CHARGE_SCHEMA || record.runner != "lab-runner" {
        return Err(invalid("unsupported charge schema/runner"));
    }
    for (value, field) in [
        (&record.lab, "lab"),
        (&record.job_id, "job_id"),
        (&record.attempt_id, "attempt_id"),
        (&record.outcome, "outcome"),
        (&record.recorded_utc, "recorded_utc"),
    ] {
        nonempty(value, field)?;
    }
    if record.event_id != charge_id(&record.job_id, &record.attempt_id) {
        return Err(invalid(
            "charge event identity does not match job and attempt",
        ));
    }
    if let Some(metadata) = &record.accounting {
        if metadata.category != CostCategory::JobExecution
            || metadata.correction_of.is_some()
            || (record.outcome == "reconciled_stopped_estimated_charge"
                && metadata.measurement != Measurement::Estimated)
        {
            return Err(invalid(
                "attempt accounting metadata conflicts with its execution receipt",
            ));
        }
    }
    Ok(())
}
fn validate_extension(record: &ExtensionRecord) -> Result<()> {
    if record.schema != EXTENSION_SCHEMA {
        return Err(invalid("unsupported extension schema"));
    }
    for (value, field) in [
        (&record.event_id, "event_id"),
        (&record.authority, "authority"),
        (&record.rationale, "rationale"),
        (&record.recorded_utc, "recorded_utc"),
    ] {
        nonempty(value, field)?;
    }
    if record.increment_ms == 0 {
        return Err(invalid("extension must add a positive allowance"));
    }
    Ok(())
}

pub fn record_attempt_charge(
    dir: &Path,
    lab: &str,
    job_id: &str,
    attempt_id: &str,
    charged_ms: u64,
    wall_s: u64,
    outcome: &str,
) -> Result<LedgerState> {
    let measurement = if outcome == "reconciled_stopped_estimated_charge" {
        Measurement::Estimated
    } else {
        Measurement::Measured
    };
    record_attempt_charge_with_measurement(
        dir,
        lab,
        job_id,
        attempt_id,
        charged_ms,
        wall_s,
        outcome,
        measurement,
    )
}

pub fn record_attempt_charge_with_measurement(
    dir: &Path,
    lab: &str,
    job_id: &str,
    attempt_id: &str,
    charged_ms: u64,
    wall_s: u64,
    outcome: &str,
    measurement: Measurement,
) -> Result<LedgerState> {
    fs::create_dir_all(dir)?;
    let _lock = acquire_lock(dir)?;
    // Fail before mutation if migration is missing or existing history is inconsistent.
    let before = fold_records(dir)?;
    let record = ChargeRecord {
        schema: CHARGE_SCHEMA.into(),
        event_id: charge_id(job_id, attempt_id),
        recorded_utc: utc_now_iso(),
        lab: lab.into(),
        job_id: job_id.into(),
        attempt_id: attempt_id.into(),
        charged_ms,
        wall_s,
        outcome: outcome.into(),
        runner: "lab-runner".into(),
        accounting: Some(CostMetadata {
            category: CostCategory::JobExecution,
            measurement,
            correction_of: None,
        }),
    };
    validate_charge(&record)?;
    let path = dir.join(format!("charge-v2-{}.json", record.event_id));
    if path.exists() {
        let mut old: ChargeRecord = serde_json::from_slice(&fs::read(&path)?)?;
        old.recorded_utc = record.recorded_utc.clone();
        // Retry compatibility never rewrites older immutable bytes or invents
        // historical metadata. Only compare their newly known default in memory.
        if old.accounting.is_none() {
            old.accounting = record.accounting.clone();
        }
        if old != record {
            return Err(invalid("attempt already charged with different receipt"));
        }
    } else {
        add(before.cumulative_ms, charged_ms)?;
        immutable_write(&path, &serde_json::to_vec_pretty(&record)?)?;
    }
    let state = fold_records(dir)?;
    write_view(dir, state)?;
    Ok(state)
}

/// Compatibility for callers whose job ID itself identifies exactly one attempt.
pub fn record_charge(
    dir: &Path,
    lab: &str,
    job_id: &str,
    charged_ms: u64,
    wall_s: u64,
    outcome: &str,
) -> Result<LedgerState> {
    record_attempt_charge(dir, lab, job_id, job_id, charged_ms, wall_s, outcome)
}
pub fn record_extension(dir: &Path, record: &ExtensionRecord) -> Result<LedgerState> {
    fs::create_dir_all(dir)?;
    let _lock = acquire_lock(dir)?;
    let before = fold_records(dir)?;
    validate_extension(record)?;
    add(before.limit_ms, record.increment_ms)?;
    immutable_write(
        &dir.join(format!(
            "extension-v2-{}.json",
            sha256(record.event_id.as_bytes())
        )),
        &serde_json::to_vec_pretty(record)?,
    )?;
    let state = fold_records(dir)?;
    write_view(dir, state)?;
    Ok(state)
}
/// Append an idempotent programme charge/correction. Validate a virtual fold
/// before publication, so an invalid correction cannot poison durable history.
pub fn record_resource_charge(dir: &Path, record: &ResourceChargeRecord) -> Result<LedgerState> {
    fs::create_dir_all(dir)?;
    let _lock = acquire_lock(dir)?;
    fold_records(dir)?;
    validate_resource(record)?;
    let state = fold_with_resource(dir, Some(record))?;
    immutable_write(
        &dir.join(resource_filename(&record.event_id)),
        &serde_json::to_vec_pretty(record)?,
    )?;
    write_view(dir, state)?;
    Ok(state)
}
pub fn import_legacy(dir: &Path, record: &LegacyImport) -> Result<LedgerState> {
    let _lock = acquire_lock(dir)?;
    if record.schema != IMPORT_SCHEMA {
        return Err(invalid("unsupported import schema"));
    }
    nonempty(&record.event_id, "event_id")?;
    nonempty(&record.authority, "authority")?;
    nonempty(&record.rationale, "rationale")?;
    validate_legacy_observation(record)?;
    verify_covered(dir, &record.source)?;
    let source: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join(&record.source.file))?)?;
    if matches!(
        source.get("schema").and_then(|v| v.as_str()),
        Some(CHARGE_SCHEMA | EXTENSION_SCHEMA | RESOURCE_CHARGE_SCHEMA)
    ) {
        return Err(invalid("legacy import cannot absorb typed v2 events"));
    }
    let baseline: MigrationBaseline = serde_json::from_slice(&fs::read(dir.join(BASELINE_FILE))?)?;
    if baseline
        .covered_records
        .iter()
        .any(|v| v.file == record.source.file)
    {
        return Err(invalid("legacy source already covered by baseline"));
    }
    let destination = dir.join(format!(
        "import-{}.json",
        sha256(record.event_id.as_bytes())
    ));
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("import-") || !name.ends_with(".json") || entry.path() == destination {
            continue;
        }
        let old: LegacyImport = serde_json::from_slice(&fs::read(entry.path())?)?;
        if old.source.file == record.source.file {
            return Err(invalid("legacy source already covered by another import"));
        }
    }
    if baseline.legacy_source.is_some() {
        legacy_watch_coverage(dir, &baseline, Some(record))?;
    }
    immutable_write(&destination, &serde_json::to_vec_pretty(record)?)?;
    let state = fold_records(dir)?;
    write_view(dir, state)?;
    Ok(state)
}
/// Read-only budget check. The singleton supervisor includes all outstanding
/// reservations in requested_ms and holds its admission lock through launch.
pub fn check_budget(dir: &Path, requested_ms: u64) -> Result<LedgerState> {
    let _lock = acquire_lock(dir)?;
    let state = fold_records(dir)?;
    // Zero is used by control/monitor paths to inspect already admitted work.
    // They retain internal budget enforcement without depending on old media.
    if requested_ms > 0 {
        check_legacy_watch(dir)?;
    }
    if add(state.cumulative_ms, requested_ms)? > state.limit_ms {
        return Err(invalid(
            "cumulative ledger plus reservations exceeds allowance",
        ));
    }
    Ok(state)
}

fn validate_legacy_source(dir: &Path, source: Option<&Path>) -> Result<()> {
    if let Some(source) = source {
        if !source.is_absolute()
            || source
                .components()
                .any(|p| matches!(p, std::path::Component::ParentDir))
            || source == dir
        {
            return Err(invalid(
                "legacy watch must name a distinct absolute source ledger path",
            ));
        }
        // Do not canonicalize/read the external path here: rebuild and durable
        // job charging must work during removal or restoration of that volume.
    }
    Ok(())
}
fn validate_legacy_observation(record: &LegacyImport) -> Result<()> {
    if let Some(observed) = &record.legacy_observed {
        valid_covered(&CoveredRecord {
            file: observed.file.clone(),
            sha256: observed.sha256.clone(),
        })?;
        if observed.sha256 != record.source.sha256 {
            return Err(invalid(
                "legacy observation must match the preserved imported bytes",
            ));
        }
        if let Some(previous) = &observed.previous_sha256 {
            valid_covered(&CoveredRecord {
                file: observed.file.clone(),
                sha256: previous.clone(),
            })?;
            if previous == &observed.sha256 {
                return Err(invalid(
                    "legacy observation update must identify changed bytes",
                ));
            }
        }
    }
    Ok(())
}
fn legacy_watch_coverage(
    dir: &Path,
    baseline: &MigrationBaseline,
    proposed: Option<&LegacyImport>,
) -> Result<BTreeMap<String, String>> {
    let mut known = BTreeMap::new();
    for source in &baseline.covered_records {
        if known
            .insert(source.file.clone(), source.sha256.clone())
            .is_some()
        {
            return Err(invalid("duplicate original legacy coverage"));
        }
    }
    let mut imports = BTreeMap::<String, LegacyImport>::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("import-") && name.ends_with(".json") {
            let record: LegacyImport = serde_json::from_slice(&fs::read(entry.path())?)?;
            if let Some(old) = imports.insert(record.event_id.clone(), record.clone()) {
                if old != record {
                    return Err(invalid("conflicting imported legacy observation"));
                }
            }
        }
    }
    if let Some(record) = proposed {
        if let Some(old) = imports.insert(record.event_id.clone(), record.clone()) {
            if old != *record {
                return Err(invalid("conflicting proposed legacy observation"));
            }
        }
    }
    let mut pending = Vec::new();
    let mut edges = BTreeMap::new();
    for record in imports.values() {
        validate_legacy_observation(record)?;
        let observation = record.legacy_observed.clone().unwrap_or(LegacyObservation {
            file: record.source.file.clone(),
            sha256: record.source.sha256.clone(),
            previous_sha256: None,
        });
        let key = (
            observation.file.clone(),
            observation.previous_sha256.clone(),
        );
        if let Some(old) = edges.insert(key, observation.sha256.clone()) {
            if old != observation.sha256 {
                return Err(invalid(
                    "ambiguous legacy observation branches require explicit reconciliation",
                ));
            }
        }
        pending.push(observation);
    }
    // Replay the hash chain, never filename or timestamp order. A later import
    // cannot silently overwrite a previously reviewed source observation.
    let mut seen: BTreeSet<_> = known
        .iter()
        .map(|(name, hash)| (name.clone(), hash.clone()))
        .collect();
    while !pending.is_empty() {
        let mut progressed = false;
        let mut remaining = Vec::new();
        for item in pending {
            let ready = match &item.previous_sha256 {
                None => !known.contains_key(&item.file),
                Some(previous) => known.get(&item.file) == Some(previous),
            };
            if ready {
                if !seen.insert((item.file.clone(), item.sha256.clone())) {
                    return Err(invalid(
                        "legacy observation repeats prior bytes; manual reconciliation required",
                    ));
                }
                known.insert(item.file.clone(), item.sha256.clone());
                progressed = true;
            } else {
                remaining.push(item);
            }
        }
        pending = remaining;
        if !progressed {
            return Err(invalid(
                "legacy observation chain is unresolved or duplicates accounted coverage",
            ));
        }
    }
    Ok(known)
}
fn external_receipt_snapshot(source: &Path) -> Result<BTreeMap<String, String>> {
    let mut observed = BTreeMap::new();
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry
            .file_name()
            .to_str()
            .ok_or_else(|| invalid("non-UTF8 legacy receipt filename"))?
            .to_string();
        if !(name.starts_with("charge-") || name.starts_with("extension-"))
            || !name.ends_with(".json")
        {
            continue;
        }
        let before = fs::symlink_metadata(entry.path())?;
        if !before.is_file() || before.file_type().is_symlink() || before.len() > 1024 * 1024 {
            return Err(invalid(
                "legacy watch receipt is unreadable, linked, special or oversized",
            ));
        }
        let bytes = fs::read(entry.path())?;
        let after = fs::symlink_metadata(entry.path())?;
        if before.len() != bytes.len() as u64
            || before.len() != after.len()
            || before.modified()? != after.modified()?
            || !after.is_file()
            || after.file_type().is_symlink()
        {
            return Err(invalid("legacy ledger changed while observing it"));
        }
        let _: serde_json::Value = serde_json::from_slice(&bytes)?;
        observed.insert(name, sha256(&bytes));
    }
    Ok(observed)
}
fn check_legacy_watch(dir: &Path) -> Result<()> {
    let baseline: MigrationBaseline = serde_json::from_slice(&fs::read(dir.join(BASELINE_FILE))?)?;
    let Some(source) = baseline.legacy_source.as_deref() else {
        return Ok(());
    };
    validate_legacy_source(dir, Some(source))?;
    if fs::canonicalize(source)? == fs::canonicalize(dir)? {
        return Err(invalid(
            "legacy watch resolves to the internal ledger itself",
        ));
    }
    let expected = legacy_watch_coverage(dir, &baseline, None)?;
    let snapshot_expected = legacy_snapshot_head(dir, &baseline)?;
    let snapshot_first = read_snapshot_bytes(source)?;
    let first = external_receipt_snapshot(source)?;
    let second = external_receipt_snapshot(source)?;
    let snapshot_second = read_snapshot_bytes(source)?;
    if first != second || second != expected {
        return Err(invalid("legacy ledger has new, changed or missing receipts; stop new admission for explicit import/review"));
    }
    if snapshot_first != snapshot_second || sha256(&snapshot_second) != snapshot_expected {
        return Err(invalid("legacy model-time snapshot drifted; explicit accounting review and immutable snapshot observation required"));
    }
    Ok(())
}
fn snapshot_copy_name(hash: &str) -> Result<String> {
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(invalid("snapshot identity needs lowercase SHA-256"));
    }
    Ok(format!("legacy-snapshot-{hash}.json"))
}
fn verify_snapshot_copy(dir: &Path, hash: &str) -> Result<()> {
    let path = dir.join(snapshot_copy_name(hash)?);
    let meta = fs::symlink_metadata(&path)?;
    if !meta.is_file() || meta.file_type().is_symlink() || sha256(&fs::read(path)?) != hash {
        return Err(invalid(
            "preserved legacy snapshot bytes unavailable or changed",
        ));
    }
    Ok(())
}
fn read_snapshot_bytes(source: &Path) -> Result<Vec<u8>> {
    let path = source.join(MODEL_TIME_FILE);
    let before = fs::symlink_metadata(&path)?;
    if !before.is_file() || before.file_type().is_symlink() || before.len() > 1024 * 1024 {
        return Err(invalid(
            "legacy counter snapshot is unavailable or ambiguous",
        ));
    }
    let bytes = fs::read(&path)?;
    let after = fs::symlink_metadata(path)?;
    if before.len() != bytes.len() as u64
        || before.len() != after.len()
        || before.modified()? != after.modified()?
        || !after.is_file()
        || after.file_type().is_symlink()
    {
        return Err(invalid("legacy counter snapshot changed while observed"));
    }
    let _: serde_json::Value = serde_json::from_slice(&bytes)?;
    Ok(bytes)
}
fn legacy_snapshot_head(dir: &Path, baseline: &MigrationBaseline) -> Result<String> {
    let mut head = baseline
        .legacy_snapshot_sha256
        .clone()
        .ok_or_else(|| invalid("legacy watch lacks original snapshot identity"))?;
    verify_snapshot_copy(dir, &head)?;
    let mut records = BTreeMap::<String, LegacySnapshotObservation>::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("watch-snapshot-") && name.ends_with(".json") {
            let value: LegacySnapshotObservation =
                serde_json::from_slice(&fs::read(entry.path())?)?;
            validate_snapshot_observation(&value)?;
            verify_snapshot_copy(dir, &value.sha256)?;
            if let Some(old) = records.insert(value.event_id.clone(), value.clone()) {
                if old != value {
                    return Err(invalid("snapshot observation ID conflict"));
                }
            }
        }
    }
    let mut edges = BTreeMap::new();
    for record in records.values() {
        if let Some(old) = edges.insert(record.previous_sha256.clone(), record.sha256.clone()) {
            if old != record.sha256 {
                return Err(invalid("ambiguous snapshot observation branches"));
            }
        }
    }
    let mut visited = BTreeSet::new();
    while let Some(next) = edges.remove(&head) {
        if !visited.insert(head.clone()) {
            return Err(invalid("cyclic snapshot observation chain"));
        }
        head = next;
    }
    if !edges.is_empty() || visited.contains(&head) {
        return Err(invalid("snapshot observation chain is disconnected"));
    }
    Ok(head)
}
fn validate_snapshot_observation(record: &LegacySnapshotObservation) -> Result<()> {
    if record.schema != "uor-r4.legacy-snapshot-observation/1" {
        return Err(invalid("unsupported legacy snapshot observation"));
    }
    for (value, name) in [
        (&record.event_id, "event_id"),
        (&record.recorded_utc, "recorded_utc"),
        (&record.authority, "authority"),
        (&record.rationale, "rationale"),
    ] {
        nonempty(value, name)?;
    }
    snapshot_copy_name(&record.previous_sha256)?;
    snapshot_copy_name(&record.sha256)?;
    if record.previous_sha256 == record.sha256 {
        return Err(invalid("snapshot observation must record changed bytes"));
    }
    Ok(())
}
/// Watch-only reconciliation after accounting review; this never charges or
/// resets totals. Preserve the actual snapshot, then append its reviewed link.
pub fn record_legacy_snapshot_observation(
    dir: &Path,
    record: &LegacySnapshotObservation,
) -> Result<LedgerState> {
    let _lock = acquire_lock(dir)?;
    let state = fold_records(dir)?;
    validate_snapshot_observation(record)?;
    let path = dir.join(format!(
        "watch-snapshot-{}.json",
        sha256(record.event_id.as_bytes())
    ));
    let bytes = serde_json::to_vec_pretty(record)?;
    if path.exists() {
        if fs::read(&path)? == bytes {
            return Ok(state);
        }
        return Err(invalid("snapshot observation identity reused"));
    }
    if state != record.reviewed_internal_state {
        return Err(invalid("internal accounting changed since snapshot review"));
    }
    let baseline: MigrationBaseline = serde_json::from_slice(&fs::read(dir.join(BASELINE_FILE))?)?;
    let source = baseline
        .legacy_source
        .as_deref()
        .ok_or_else(|| invalid("no configured legacy source watch"))?;
    if legacy_snapshot_head(dir, &baseline)? != record.previous_sha256 {
        return Err(invalid(
            "snapshot observation does not advance current reviewed hash",
        ));
    }
    if baseline.legacy_snapshot_sha256.as_ref() == Some(&record.sha256) {
        return Err(invalid(
            "snapshot repeats original bytes; manual reconciliation required",
        ));
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("watch-snapshot-") && name.ends_with(".json") {
            let old: LegacySnapshotObservation = serde_json::from_slice(&fs::read(entry.path())?)?;
            if old.previous_sha256 == record.sha256 {
                return Err(invalid(
                    "snapshot repeats historical bytes; manual reconciliation required",
                ));
            }
        }
    }
    let observed = read_snapshot_bytes(source)?;
    if sha256(&observed) != record.sha256 {
        return Err(invalid(
            "external snapshot differs from reviewed observation",
        ));
    }
    immutable_write(&dir.join(snapshot_copy_name(&record.sha256)?), &observed)?;
    immutable_write(&path, &bytes)?;
    Ok(state)
}
pub fn rebuild(dir: &Path) -> Result<LedgerState> {
    fs::create_dir_all(dir)?;
    let _lock = acquire_lock(dir)?;
    let state = fold_records(dir)?;
    write_view(dir, state)?;
    Ok(state)
}
pub fn default_dir() -> Result<PathBuf> {
    crate::default_ledger_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ledger-{}-{}-{name}",
            std::process::id(),
            TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn external_legacy_drift_blocks_only_admission_until_reviewed_import() {
        let dir = temp("watched-internal");
        let external = temp("watched-legacy");
        let old = br#"{"schema":"legacy","charged_ms":100}"#;
        fs::write(dir.join("charge-original.json"), old).unwrap();
        fs::write(external.join("charge-original.json"), old).unwrap();
        let snapshot = br#"{"cumulative_ms":100,"limit_ms":1000}"#;
        fs::write(external.join(MODEL_TIME_FILE), snapshot).unwrap();
        fs::write(
            dir.join(snapshot_copy_name(&sha256(snapshot)).unwrap()),
            snapshot,
        )
        .unwrap();
        migrate(
            &dir,
            &MigrationBaseline {
                schema: BASELINE_SCHEMA.into(),
                baseline_id: "watched".into(),
                recorded_utc: utc_now_iso(),
                cumulative_ms: 100,
                limit_ms: 1000,
                covered_records: vec![CoveredRecord {
                    file: "charge-original.json".into(),
                    sha256: sha256(old),
                }],
                authority: "fixture".into(),
                rationale: "reviewed copied legacy records".into(),
                legacy_source: Some(external.clone()),
                legacy_snapshot_sha256: Some(sha256(snapshot)),
            },
        )
        .unwrap();
        assert!(check_budget(&dir, 1).is_ok());
        let late = br#"{"schema":"legacy","charged_ms":10}"#;
        fs::write(external.join("charge-late.json"), late).unwrap();
        assert!(check_budget(&dir, 1).is_err());
        assert_eq!(rebuild(&dir).unwrap().cumulative_ms, 100);
        fs::write(dir.join("charge-late.json"), late).unwrap();
        import_legacy(
            &dir,
            &LegacyImport {
                schema: IMPORT_SCHEMA.into(),
                event_id: "late".into(),
                recorded_utc: utc_now_iso(),
                source: CoveredRecord {
                    file: "charge-late.json".into(),
                    sha256: sha256(late),
                },
                charged_ms: 10,
                increment_ms: 0,
                authority: "fixture".into(),
                rationale: "new legacy work".into(),
                legacy_observed: None,
            },
        )
        .unwrap();
        assert!(check_budget(&dir, 1).is_ok());
        let updated = br#"{"cumulative_ms":110,"limit_ms":1000}"#;
        fs::write(external.join(MODEL_TIME_FILE), updated).unwrap();
        assert!(check_budget(&dir, 1).is_err());
        assert_eq!(rebuild(&dir).unwrap().cumulative_ms, 110);
        record_legacy_snapshot_observation(
            &dir,
            &LegacySnapshotObservation {
                schema: "uor-r4.legacy-snapshot-observation/1".into(),
                event_id: "counter-reviewed".into(),
                recorded_utc: utc_now_iso(),
                previous_sha256: sha256(snapshot),
                sha256: sha256(updated),
                reviewed_internal_state: rebuild(&dir).unwrap(),
                authority: "fixture review".into(),
                rationale: "counter increase accounted by the late import".into(),
            },
        )
        .unwrap();
        assert!(check_budget(&dir, 1).is_ok());
        let changed = br#"{"schema":"legacy","charged_ms":120}"#;
        fs::write(external.join("charge-original.json"), changed).unwrap();
        assert!(check_budget(&dir, 1).is_err());
        fs::write(dir.join("charge-original-revision.json"), changed).unwrap();
        import_legacy(
            &dir,
            &LegacyImport {
                schema: IMPORT_SCHEMA.into(),
                event_id: "revision".into(),
                recorded_utc: utc_now_iso(),
                source: CoveredRecord {
                    file: "charge-original-revision.json".into(),
                    sha256: sha256(changed),
                },
                charged_ms: 20,
                increment_ms: 0,
                authority: "fixture".into(),
                rationale: "preserved changed source; charge only the reviewed delta".into(),
                legacy_observed: Some(LegacyObservation {
                    file: "charge-original.json".into(),
                    sha256: sha256(changed),
                    previous_sha256: Some(sha256(old)),
                }),
            },
        )
        .unwrap();
        assert!(check_budget(&dir, 1).is_ok());
        assert_eq!(rebuild(&dir).unwrap().cumulative_ms, 130);
        let offline = external.with_extension("offline");
        fs::rename(&external, &offline).unwrap();
        assert!(check_budget(&dir, 1).is_err());
        assert_eq!(rebuild(&dir).unwrap().cumulative_ms, 130);
        assert!(check_budget(&dir, 0).is_ok());
        assert_eq!(
            record_attempt_charge(&dir, "lab", "already-running", "attempt", 1, 1, "completed")
                .unwrap()
                .cumulative_ms,
            131
        );
        assert_eq!(fs::read(dir.join("charge-original.json")).unwrap(), old);
        fs::remove_dir_all(dir).unwrap();
        fs::remove_dir_all(offline).unwrap();
    }
    #[test]
    fn execution_metadata_distinguishes_estimate_and_old_bytes_remain_retryable() {
        let dir = temp("metadata");
        initialize_empty(&dir, 1000, "fixture").unwrap();
        record_attempt_charge(
            &dir,
            "lab",
            "job",
            "estimated",
            200,
            1,
            "reconciled_stopped_estimated_charge",
        )
        .unwrap();
        let file = dir.join(format!("charge-v2-{}.json", charge_id("job", "estimated")));
        let record: ChargeRecord = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        assert_eq!(
            record.accounting.unwrap().measurement,
            Measurement::Estimated
        );
        // Construct an older-format fixture, not a production migration rewrite.
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        value.as_object_mut().unwrap().remove("accounting");
        let old = serde_json::to_vec(&value).unwrap();
        fs::write(&file, &old).unwrap();
        record_attempt_charge(
            &dir,
            "lab",
            "job",
            "estimated",
            200,
            1,
            "reconciled_stopped_estimated_charge",
        )
        .unwrap();
        assert_eq!(fs::read(file).unwrap(), old);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn programme_corrections_are_hash_bound_idempotent_and_cannot_overdraw() {
        let dir = temp("resources");
        initialize_empty(&dir, 1000, "fixture").unwrap();
        let preparation = ResourceChargeRecord {
            schema: RESOURCE_CHARGE_SCHEMA.into(),
            event_id: "prep-1".into(),
            recorded_utc: "2026-09-29T00:00:00Z".into(),
            lab: "lab".into(),
            adjustment_ms: 100,
            accounting: CostMetadata {
                category: CostCategory::Preparation,
                measurement: Measurement::Estimated,
                correction_of: None,
            },
            authority: "measured work card".into(),
            rationale: "initial preparation estimate".into(),
        };
        record_resource_charge(&dir, &preparation).unwrap();
        record_resource_charge(&dir, &preparation).unwrap();
        let file = resource_filename(&preparation.event_id);
        let original = fs::read(dir.join(&file)).unwrap();
        let mut correction = ResourceChargeRecord {
            event_id: "prep-correction".into(),
            adjustment_ms: -20,
            accounting: CostMetadata {
                category: CostCategory::Preparation,
                measurement: Measurement::Measured,
                correction_of: Some(CorrectionReference {
                    file,
                    sha256: sha256(&original),
                }),
            },
            rationale: "completed timing was 80 ms; append difference".into(),
            ..preparation.clone()
        };
        assert_eq!(
            record_resource_charge(&dir, &correction)
                .unwrap()
                .cumulative_ms,
            80
        );
        assert_eq!(
            record_resource_charge(&dir, &correction)
                .unwrap()
                .cumulative_ms,
            80
        );
        assert_eq!(
            fs::read(dir.join(resource_filename(&preparation.event_id))).unwrap(),
            original
        );
        correction.event_id = "overdraw".into();
        correction.adjustment_ms = -90;
        assert!(record_resource_charge(&dir, &correction).is_err());
        assert!(!dir.join(resource_filename(&correction.event_id)).exists());
        correction.event_id = "wrong-hash".into();
        correction.adjustment_ms = 1;
        correction.accounting.correction_of.as_mut().unwrap().sha256 = "0".repeat(64);
        assert!(record_resource_charge(&dir, &correction).is_err());
        assert_eq!(rebuild(&dir).unwrap().cumulative_ms, 80);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn historical_fixture_requires_explicit_coverage_and_preserves_late_event() {
        let dir = temp("migration");
        let mut covered = Vec::new();
        for entry in
            fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ledger"))
                .unwrap()
        {
            let entry = entry.unwrap();
            let bytes = fs::read(entry.path()).unwrap();
            let name = entry.file_name().to_string_lossy().into_owned();
            fs::write(dir.join(&name), &bytes).unwrap();
            covered.push(CoveredRecord {
                file: name,
                sha256: sha256(&bytes),
            });
        }
        assert!(rebuild(&dir).is_err());
        migrate(
            &dir,
            &MigrationBaseline {
                schema: BASELINE_SCHEMA.into(),
                baseline_id: "reviewed-fixture".into(),
                recorded_utc: utc_now_iso(),
                cumulative_ms: 782_538_181,
                limit_ms: 1_130_000_000,
                covered_records: covered,
                authority: "fixture reconciliation".into(),
                rationale: "known historical total, each covered receipt mapped".into(),
                legacy_source: None,
                legacy_snapshot_sha256: None,
            },
        )
        .unwrap();
        record_attempt_charge(&dir, "a", "job", "attempt", 123, 1, "completed").unwrap();
        // Timestamps do not order or erase events.
        assert_eq!(rebuild(&dir).unwrap().cumulative_ms, 782_538_304);
        let legacy = br#"{"schema":"uor-r4.model-time-charge/1","charged_ms":7}"#;
        fs::write(dir.join("charge-late.json"), legacy).unwrap();
        assert!(rebuild(&dir).is_err());
        import_legacy(
            &dir,
            &LegacyImport {
                schema: IMPORT_SCHEMA.into(),
                event_id: "late".into(),
                recorded_utc: utc_now_iso(),
                source: CoveredRecord {
                    file: "charge-late.json".into(),
                    sha256: sha256(legacy),
                },
                charged_ms: 7,
                increment_ms: 0,
                authority: "test review".into(),
                rationale: "late omitted receipt".into(),
                legacy_observed: None,
            },
        )
        .unwrap();
        assert_eq!(rebuild(&dir).unwrap().cumulative_ms, 782_538_311);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn idempotent_charge_repairs_missing_view_and_rejects_conflict() {
        let dir = temp("idempotent");
        initialize_empty(&dir, 1000, "test genesis").unwrap();
        record_attempt_charge(&dir, "a", "j", "a", 250, 1, "completed").unwrap();
        fs::remove_file(dir.join(MODEL_TIME_FILE)).unwrap();
        assert_eq!(
            record_attempt_charge(&dir, "a", "j", "a", 250, 1, "completed")
                .unwrap()
                .cumulative_ms,
            250
        );
        assert!(record_attempt_charge(&dir, "a", "j", "a", 251, 1, "completed").is_err());
        assert!(check_budget(&dir, 751).is_err());
        assert!(check_budget(&dir, 750).is_ok());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn concurrent_charges_survive_and_duplicate_copies_do_not_double_charge() {
        let dir = temp("concurrent");
        initialize_empty(&dir, 1000, "test genesis").unwrap();
        let handles: Vec<_> = (0..2)
            .map(|n| {
                let dir = dir.clone();
                std::thread::spawn(move || {
                    record_charge(&dir, "lab", &format!("job-{n}"), 250, 1, "completed").unwrap()
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let file = legacy_files(&dir)
            .unwrap()
            .into_iter()
            .find(|(_, v)| v["schema"] == CHARGE_SCHEMA)
            .unwrap()
            .0;
        fs::copy(dir.join(file), dir.join("charge-duplicate.json")).unwrap();
        assert_eq!(rebuild(&dir).unwrap().cumulative_ms, 500);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn extension_never_resets_prior_charge_and_corruption_is_rejected() {
        let dir = temp("extension");
        initialize_empty(&dir, 1000, "test genesis").unwrap();
        record_charge(&dir, "a", "j", 900, 1, "completed").unwrap();
        let extension = ExtensionRecord {
            schema: EXTENSION_SCHEMA.into(),
            event_id: "extension-1".into(),
            recorded_utc: "2000-01-01T00:00:00Z".into(),
            increment_ms: 200,
            authority: "standing authority".into(),
            rationale: "prospective remaining work".into(),
        };
        record_extension(&dir, &extension).unwrap();
        record_extension(&dir, &extension).unwrap();
        assert_eq!(
            rebuild(&dir).unwrap(),
            LedgerState {
                cumulative_ms: 900,
                limit_ms: 1200
            }
        );
        fs::remove_dir_all(dir).unwrap();
    }
}
