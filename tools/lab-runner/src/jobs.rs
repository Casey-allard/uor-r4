//! Durable job lifecycle. Receipts precede idempotent charges; DONE follows
//! both. A per-job kernel lock serializes launch, cancel and finalization.
use crate::spec::{valid_id, JobSpec};
use crate::{invalid, ledger, process, utc_now_iso, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub const EXIT_SCHEMA: &str = "uor-r4.lab-runner-exit/1";
pub fn queue_dir(root: &Path) -> PathBuf {
    root.join("queue")
}
pub fn running_dir(root: &Path) -> PathBuf {
    root.join("running")
}
pub fn done_dir(root: &Path) -> PathBuf {
    root.join("done")
}

pub fn durable_rename(source: &Path, destination: &Path) -> Result<()> {
    fs::rename(source, destination)?;
    let from = source
        .parent()
        .ok_or_else(|| invalid("rename source has no parent"))?;
    let to = destination
        .parent()
        .ok_or_else(|| invalid("rename destination has no parent"))?;
    fs::File::open(from)?.sync_all()?;
    if from != to {
        fs::File::open(to)?.sync_all()?;
    }
    Ok(())
}

pub fn ensure_layout(root: &Path) -> Result<()> {
    if !root.is_absolute() {
        return Err(invalid("runner root must be absolute"));
    }
    let ancestor = root
        .ancestors()
        .find(|p| p.exists())
        .ok_or_else(|| invalid("runner root has no existing ancestor"))?;
    if root.starts_with("/Volumes") || fs::canonicalize(ancestor)?.starts_with("/Volumes") {
        return Err(invalid(
            "runner control state must be internal; external roots require explicit migration",
        ));
    }
    for name in ["queue", "running", "done", "locks"] {
        fs::create_dir_all(root.join(name))?;
    }
    Ok(())
}

pub fn job_lock(root: &Path, id: &str) -> Result<fs::File> {
    if !valid_id(id) {
        return Err(invalid("invalid job id"));
    }
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(root.join("locks").join(format!("{id}.lock")))?;
    for _ in 0..100 {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(fs::TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(10)),
            Err(error) => return Err(invalid(format!("job {id} lock unavailable: {error}"))),
        }
    }
    Err(invalid(format!("job {id} lock remained busy")))
}

pub fn locate(root: &Path, id: &str) -> Option<(&'static str, PathBuf)> {
    if !valid_id(id) {
        return None;
    }
    for (stage, path) in [
        ("queue", queue_dir(root)),
        ("running", running_dir(root)),
        ("done", done_dir(root)),
    ] {
        let path = path.join(id);
        if path.is_dir() {
            return Some((stage, path));
        }
    }
    None
}

pub fn submit(root: &Path, spec_path: &Path) -> Result<String> {
    let spec = JobSpec::parse(&fs::read(spec_path)?)?;
    ensure_layout(root)?;
    let _lock = job_lock(root, &spec.id)?;
    if locate(root, &spec.id).is_some() {
        return Err(invalid(format!("job id {} already exists", spec.id)));
    }
    let dir = queue_dir(root).join(&spec.id);
    fs::create_dir(&dir)?;
    crate::atomic_write(&dir.join("spec.json"), &serde_json::to_vec_pretty(&spec)?)?;
    fs::File::open(queue_dir(root))?.sync_all()?;
    Ok(spec.id)
}

pub fn read_spec(dir: &Path) -> Result<JobSpec> {
    Ok(serde_json::from_slice(&fs::read(dir.join("spec.json"))?)?)
}
pub fn read_identity(dir: &Path) -> Result<process::Identity> {
    Ok(serde_json::from_slice(&fs::read(
        dir.join("process.json"),
    )?)?)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    pub attempt_id: String,
    pub process_token: String,
    pub started_ms: u64,
    pub started_utc: String,
}
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
pub fn prepare_attempt(dir: &Path, id: &str) -> Result<Attempt> {
    prepare_reserved_attempt(dir, id, None)
}

pub fn prepare_reserved_attempt(dir: &Path, id: &str, reserved: Option<&str>) -> Result<Attempt> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| invalid("system clock before epoch"))?
        .as_nanos();
    let attempt = Attempt {
        attempt_id: reserved
            .map(str::to_string)
            .unwrap_or_else(|| format!("{id}-{}-{nonce}", std::process::id())),
        process_token: format!("job-{}-{nonce}", std::process::id()),
        started_ms: now_ms(),
        started_utc: utc_now_iso(),
    };
    if dir.join("attempt.json").exists() {
        return Err(invalid("attempt already exists; do not relaunch"));
    }
    crate::atomic_write(
        &dir.join("attempt.json"),
        &serde_json::to_vec_pretty(&attempt)?,
    )?;
    Ok(attempt)
}
/// Permanent host receipt prevents an already admitted shared attempt from
/// being replayed through a different queue after a steward restart.
pub fn consumed_attempt_path(id: &str) -> Result<PathBuf> {
    if !valid_id(id) {
        return Err(invalid("invalid consumed attempt id"));
    }
    Ok(process::host_state_dir()?
        .join("attempt-starts")
        .join(format!("{id}.json")))
}

pub fn consume_attempt(ledger_dir: &Path, root: &Path, spec: &JobSpec) -> Result<()> {
    use std::io::Write;
    let claim = spec
        .coordination
        .as_ref()
        .ok_or_else(|| invalid("missing shared attempt"))?;
    if claim.attempt_id != spec.id || !valid_id(&claim.attempt_id) {
        return Err(invalid("attempt/job mismatch"));
    }
    let receipt = consumed_attempt_path(&claim.attempt_id)?;
    let dir = receipt
        .parent()
        .ok_or_else(|| invalid("attempt receipt lacks parent"))?;
    fs::create_dir_all(dir)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&receipt)
        .map_err(|error| {
            invalid(format!(
                "attempt already consumed or host receipt unavailable: {error}"
            ))
        })?;
    file.write_all(&serde_json::to_vec_pretty(&json!({"schema":"uor-r4.attempt-start/1","job":spec.id,"claim":claim,"queue":root,"ledger":ledger_dir,"at":utc_now_iso()}))?)?;
    file.sync_all()?;
    fs::File::open(&dir)?.sync_all()?;
    Ok(())
}

pub fn read_attempt(dir: &Path) -> Result<Attempt> {
    Ok(serde_json::from_slice(&fs::read(
        dir.join("attempt.json"),
    )?)?)
}

pub fn pid_alive(pid: u32) -> bool {
    pid > 1
        && Command::new("/bin/ps")
            .args(["-o", "pid=", "-p"])
            .arg(pid.to_string())
            .output()
            .map(|o| {
                o.status.success()
                    && String::from_utf8_lossy(&o.stdout)
                        .split_whitespace()
                        .any(|f| f == pid.to_string())
            })
            .unwrap_or(false)
}
fn ps_table() -> Vec<(u32, u32, u64)> {
    let Ok(o) = Command::new("/bin/ps")
        .args(["-Ao", "pid=,ppid=,rss="])
        .output()
    else {
        return vec![];
    };
    if !o.status.success() {
        return vec![];
    }
    String::from_utf8_lossy(&o.stdout)
        .lines()
        .filter_map(|l| {
            let mut f = l.split_whitespace();
            Some((
                f.next()?.parse().ok()?,
                f.next()?.parse().ok()?,
                f.next()?.parse().ok()?,
            ))
        })
        .collect()
}
fn descendants(table: &[(u32, u32, u64)], pid: u32) -> Vec<u32> {
    let mut tree = vec![pid];
    let mut i = 0;
    while i < tree.len() {
        for (p, parent, _) in table {
            if *parent == tree[i] && !tree.contains(p) {
                tree.push(*p);
            }
        }
        i += 1;
    }
    tree
}
pub fn process_tree(pid: u32) -> Vec<u32> {
    descendants(&ps_table(), pid)
}
pub fn tree_rss_kib(pid: u32) -> Option<u64> {
    let table = ps_table();
    if !table.iter().any(|(p, _, _)| *p == pid) {
        return None;
    }
    let tree = descendants(&table, pid);
    Some(
        table
            .iter()
            .filter(|(p, _, _)| tree.contains(p))
            .map(|(_, _, rss)| rss)
            .sum(),
    )
}

pub struct Finalization {
    pub outcome: String,
    pub exit_status: Option<i64>,
    pub reason: String,
    pub peak_rss_kib: u64,
    pub started_utc: String,
    pub elapsed_ms: u64,
    pub measurement: ledger::Measurement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreSpawnFailure {
    pub schema: String,
    pub id: String,
    pub attempt_id: String,
    pub host: String,
    pub runner_root: PathBuf,
    pub spec_sha256: String,
    pub attempt_sha256: String,
    pub process_token_sha256: String,
    pub phase: String,
    pub elapsed_ms: u64,
    pub measurement: ledger::Measurement,
    pub reason: String,
    pub recorded_utc: String,
}

/// Called only under the launch lock on an error branch that returns before
/// any supervisor/payload spawn or consumed-attempt tombstone. It records
/// positive control-flow evidence, not an inference from absent launch files.
pub(crate) fn record_pre_spawn_failure(
    root: &Path,
    spec: &JobSpec,
    attempt: &Attempt,
    elapsed_ms: u64,
    reason: String,
) -> Result<()> {
    let dir = running_dir(root).join(&spec.id);
    let path = dir.join("preflight-failure.json");
    if fs::symlink_metadata(&path).is_ok()
        || fs::symlink_metadata(consumed_attempt_path(&spec.id)?).is_ok()
    {
        return Err(invalid(
            "cannot certify pre-spawn failure over prior launch evidence",
        ));
    }
    let proof = PreSpawnFailure {
        schema: "uor-r4.pre-spawn-failure/1".into(),
        id: spec.id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        host: process::host_id()?,
        runner_root: fs::canonicalize(root)?,
        spec_sha256: crate::coord::digest(&serde_json::to_vec(spec)?),
        attempt_sha256: crate::coord::digest(&fs::read(dir.join("attempt.json"))?),
        process_token_sha256: crate::coord::digest(attempt.process_token.as_bytes()),
        phase: "preflight_aborted_before_supervisor_spawn".into(),
        elapsed_ms,
        measurement: ledger::Measurement::Measured,
        reason,
        recorded_utc: utc_now_iso(),
    };
    // create_new makes this terminal proof immutable across retries.
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    file.write_all(&serde_json::to_vec_pretty(&proof)?)?;
    file.sync_all()?;
    fs::File::open(&dir)?.sync_all()?;
    Ok(())
}

pub fn validate_pre_spawn_failure(
    root: &Path,
    dir: &Path,
    spec: &JobSpec,
    attempt: &Attempt,
) -> Result<PreSpawnFailure> {
    let path = dir.join("preflight-failure.json");
    if !fs::symlink_metadata(&path)?.file_type().is_file() {
        return Err(invalid("pre-spawn proof must be a regular file"));
    }
    let proof: PreSpawnFailure = serde_json::from_slice(&fs::read(path)?)?;
    if proof.schema != "uor-r4.pre-spawn-failure/1"
        || proof.id != spec.id
        || proof.attempt_id != attempt.attempt_id
        || proof.host != process::host_id()?
        || proof.runner_root != fs::canonicalize(root)?
        || proof.spec_sha256 != crate::coord::digest(&serde_json::to_vec(spec)?)
        || proof.attempt_sha256 != crate::coord::digest(&fs::read(dir.join("attempt.json"))?)
        || proof.process_token_sha256 != crate::coord::digest(attempt.process_token.as_bytes())
        || proof.phase != "preflight_aborted_before_supervisor_spawn"
        || proof.measurement != ledger::Measurement::Measured
        || proof.reason.is_empty()
        || ["process.json", "launch.go", "payload.exit", "pid"]
            .iter()
            .any(|name| fs::symlink_metadata(dir.join(name)).is_ok())
        || fs::symlink_metadata(consumed_attempt_path(&spec.id)?).is_ok()
        || process::find_supervisor(&attempt.process_token)?.is_some()
    {
        return Err(invalid(
            "pre-spawn proof identity or launch journal contradiction",
        ));
    }
    Ok(proof)
}

pub fn record_stop_request(dir: &Path, grace_ms: u64, reason: &str) -> Result<()> {
    crate::write_json_atomic(
        &dir.join("stop-request.json"),
        &json!({
            "signal":"TERM","stop_grace_ms":grace_ms,"requested_utc":utc_now_iso(),"reason":reason,
            "checkpoint_status":"unavailable_no_verified_payload_protocol",
            "meaning":"request recorded; saved checkpoint is not established by signal or exit status"
        }),
    )
}

/// Reconcile execution ownership without changing an UNKNOWN scientific
/// result. The original receipt/charge remains immutable; the separate proof
/// becomes visible only after its idempotent supplemental charge succeeds.
pub fn reconcile_stopped(root: &Path, id: &str, ledger_dir: &Path) -> Result<PathBuf> {
    let guard = job_lock(root, id)?;
    let (stage, dir) = locate(root, id).ok_or_else(|| invalid("job missing for reconciliation"))?;
    if stage == "queue" {
        return Err(invalid(
            "cancel queued jobs instead of stopped reconciliation",
        ));
    }
    let spec = read_spec(&dir)?;
    spec.validate()?;
    let attempt = read_attempt(&dir)?;
    let pre_spawn = if dir.join("preflight-failure.json").exists() {
        Some(validate_pre_spawn_failure(root, &dir, &spec, &attempt)?)
    } else {
        None
    };
    let identity = if pre_spawn.is_some() {
        None
    } else {
        Some(match read_identity(&dir) {
            Ok(identity) => identity,
            Err(_) if !dir.join("process.json").exists() => {
                let identity = process::find_supervisor(&attempt.process_token)?
                .ok_or_else(|| invalid("no durable identity or nonce supervisor; missing launch files do not prove an intact restored journal"))?;
                crate::atomic_write(
                    &dir.join("process.json"),
                    &serde_json::to_vec_pretty(&identity)?,
                )?;
                crate::write_json_atomic(
                    &dir.join("process-recovery.json"),
                    &json!({
                        "method":"live_nonce_supervisor","at":utc_now_iso(),"attempt_id":attempt.attempt_id
                    }),
                )?;
                identity
            }
            Err(error) => return Err(error),
        })
    };
    if identity
        .as_ref()
        .is_some_and(|identity| identity.token != attempt.process_token)
        || spec
            .coordination
            .as_ref()
            .is_some_and(|claim| claim.attempt_id != attempt.attempt_id)
    {
        return Err(invalid(
            "process/attempt/spec identity mismatch during reconciliation",
        ));
    }
    if let Some(identity) = &identity {
        if !process::owned_members(identity)?.is_empty() {
            record_stop_request(&dir, spec.stop_grace_ms, "explicit stopped reconciliation")?;
            process::terminate(identity, Duration::from_millis(spec.stop_grace_ms))?;
        }
        if !process::owned_members(identity)?.is_empty() {
            return Err(invalid(
                "owned processes still present; reservation remains held",
            ));
        }
    }
    if stage == "running" {
        let elapsed_ms = if let Some(proof) = &pre_spawn {
            proof.elapsed_ms
        } else {
            now_ms()
                .saturating_sub(attempt.started_ms)
                .max(spec.reserved_ms()?)
        };
        drop(guard);
        finalize(
            root,
            &spec,
            &Finalization {
                outcome: "unknown".into(),
                exit_status: None,
                reason: if pre_spawn.is_some() {
                    "positive pre-spawn failure; retain measured preflight cost".into()
                } else {
                    "explicit reconciliation stopped owned execution; payload outcome unavailable; elapsed uses wall-clock estimate".into()
                },
                peak_rss_kib: 0,
                started_utc: attempt.started_utc,
                elapsed_ms,
                measurement: if pre_spawn.is_some() {
                    ledger::Measurement::Measured
                } else {
                    ledger::Measurement::Estimated
                },
            },
            ledger_dir,
        )?;
        return reconcile_stopped(root, id, ledger_dir);
    }
    let original = fs::read(dir.join("exit.json"))?;
    let exit: Value = serde_json::from_slice(&original)?;
    let spec_hash = crate::coord::digest(&serde_json::to_vec(&spec)?);
    if exit["schema"] != EXIT_SCHEMA
        || exit["id"] != spec.id
        || exit["attempt_id"] != attempt.attempt_id
        || exit["host"] != process::host_id()?
        || exit["spec_sha256"] != spec_hash
        || exit["outcome"] != "unknown"
    {
        return Err(invalid(
            "reconciliation requires the exact local UNKNOWN receipt",
        ));
    }
    // Confirm the original charge is present or replay it before adding cost.
    charge_exit(ledger_dir, &spec, &exit)?;
    let original_hash = crate::coord::digest(&original);
    let identity_hash = if identity.is_some() {
        Some(crate::coord::digest(&fs::read(dir.join("process.json"))?))
    } else {
        None
    };
    let pre_spawn_hash = if pre_spawn.is_some() {
        Some(crate::coord::digest(&fs::read(
            dir.join("preflight-failure.json"),
        )?))
    } else {
        None
    };
    let process_state = if pre_spawn.is_some() {
        "confirmed_not_started"
    } else {
        "confirmed_stopped"
    };
    let measurement = if pre_spawn.is_some() {
        ledger::Measurement::Measured
    } else {
        ledger::Measurement::Estimated
    };
    let path = dir.join("reconciliation.json");
    let intent = dir.join("reconciliation-intent.json");
    let existing = if path.exists() {
        Some(&path)
    } else if intent.exists() {
        Some(&intent)
    } else {
        None
    };
    let receipt: Value = if let Some(existing) = existing {
        serde_json::from_slice(&fs::read(existing)?)?
    } else {
        let original_ms = exit["elapsed_ms"]
            .as_u64()
            .ok_or_else(|| invalid("original charge lacks elapsed time"))?;
        let estimate = if pre_spawn.is_some() {
            original_ms
        } else {
            now_ms()
                .saturating_sub(attempt.started_ms)
                .max(spec.reserved_ms()?)
                .max(original_ms)
        };
        let receipt = json!({"schema":"uor-r4.stopped-reconciliation/1","id":spec.id,"lab":spec.lab,
            "attempt_id":attempt.attempt_id,"host":process::host_id()?,"spec_sha256":spec_hash,"coordination":spec.coordination,
            "original_exit_sha256":original_hash,"process_identity_sha256":identity_hash,
            "pre_spawn_failure_sha256":pre_spawn_hash,"evidence_kind":if pre_spawn.is_some(){"pre_spawn_failure"}else{"owned_process"},
            "process_state":process_state,"scientific_outcome":"unknown","outcome":"interrupted","measurement":measurement,
            "checkpoint_status":"unavailable_no_verified_payload_protocol","recorded_utc":utc_now_iso(),
            "additional_charged_ms":estimate-original_ms,
            "accounting":if pre_spawn.is_some(){"no additional execution cost: positive pre-spawn failure; original measured preflight charge retained"}else{"conservative wall-clock estimate with reserved-cost floor; no monotonic cross-restart measurement"},
            "charge_attempt_id":format!("reconcile-{}",crate::coord::digest(attempt.attempt_id.as_bytes()))});
        crate::write_json_atomic(&intent, &receipt)?;
        receipt
    };
    if receipt["schema"] != "uor-r4.stopped-reconciliation/1"
        || receipt["id"] != spec.id
        || receipt["attempt_id"] != attempt.attempt_id
        || receipt["spec_sha256"] != spec_hash
        || receipt["original_exit_sha256"] != original_hash
        || receipt["process_identity_sha256"] != json!(identity_hash)
        || receipt["pre_spawn_failure_sha256"] != json!(pre_spawn_hash)
        || receipt["host"] != process::host_id()?
        || receipt["process_state"] != process_state
        || receipt["scientific_outcome"] != "unknown"
    {
        return Err(invalid("reconciliation replay identity mismatch"));
    }
    ledger::record_attempt_charge_with_measurement(
        ledger_dir,
        &spec.lab,
        &spec.id,
        receipt["charge_attempt_id"]
            .as_str()
            .ok_or_else(|| invalid("reconciliation lacks charge identity"))?,
        receipt["additional_charged_ms"]
            .as_u64()
            .ok_or_else(|| invalid("reconciliation lacks additional cost"))?,
        spec.wall_s,
        if pre_spawn.is_some() {
            "reconciled_not_started"
        } else {
            "reconciled_stopped_estimated_charge"
        },
        measurement,
    )?;
    if !path.exists() {
        durable_rename(&intent, &path)?;
    }
    Ok(path)
}

fn charge_exit(ledger_dir: &Path, spec: &JobSpec, exit: &Value) -> Result<()> {
    let spec_hash = crate::coord::digest(&serde_json::to_vec(spec)?);
    if exit["schema"] != EXIT_SCHEMA
        || exit["id"] != spec.id
        || exit["lab"] != spec.lab
        || exit["wall_s"].as_u64() != Some(spec.wall_s)
        || exit["spec_sha256"].as_str() != Some(spec_hash.as_str())
    {
        return Err(invalid(
            "durable exit receipt does not match immutable job specification",
        ));
    }
    let attempt = exit["attempt_id"]
        .as_str()
        .ok_or_else(|| invalid("receipt has no attempt identity"))?;
    let ms = exit["elapsed_ms"]
        .as_u64()
        .ok_or_else(|| invalid("receipt has no measured elapsed time"))?;
    let outcome = exit["outcome"]
        .as_str()
        .ok_or_else(|| invalid("receipt lacks outcome"))?;
    let measurement = match exit.get("measurement") {
        Some(value) => serde_json::from_value(value.clone())?,
        // Existing receipts keep the original API's retry classification;
        // this does not manufacture a new historical monotonic measurement.
        None => ledger::Measurement::Measured,
    };
    ledger::record_attempt_charge_with_measurement(
        ledger_dir,
        &spec.lab,
        &spec.id,
        attempt,
        ms,
        spec.wall_s,
        outcome,
        measurement,
    )?;
    Ok(())
}

pub fn finalize(root: &Path, spec: &JobSpec, fin: &Finalization, ledger_dir: &Path) -> Result<()> {
    let _lock = job_lock(root, &spec.id)?;
    let dir = running_dir(root).join(&spec.id);
    if !dir.is_dir() {
        return Ok(());
    }
    let path = dir.join("exit.json");
    let exit = if path.is_file() {
        serde_json::from_slice(&fs::read(&path)?)?
    } else {
        let attempt = read_attempt(&dir)?;
        let cancelled = dir.join("cancel.json").exists();
        let stop_request = fs::read(dir.join("stop-request.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
        let stopped = fin.outcome != "unknown"
            && read_identity(&dir)
                .ok()
                .and_then(|identity| process::owned_members(&identity).ok())
                .is_some_and(|members| members.is_empty());
        let exit = json!({"schema":EXIT_SCHEMA,"id":spec.id,"lab":spec.lab,"attempt_id":attempt.attempt_id,
            "host":process::host_id()?,"spec_sha256":crate::coord::digest(&serde_json::to_vec(spec)?),
            "coordination":spec.coordination,"process_state":if stopped {"confirmed_stopped"}else{"unknown"},
            "argv":spec.argv,"started_utc":fin.started_utc,"ended_utc":utc_now_iso(),"elapsed_ms":fin.elapsed_ms,
            "measurement":fin.measurement,
            "checkpoint_requested":false,"checkpoint_status":"unavailable_no_verified_payload_protocol",
            "stop_grace_ms":spec.stop_grace_ms,"stop_request":stop_request,
            "wall_s":spec.wall_s,"outcome":if cancelled && fin.outcome != "unknown" {"cancelled"}else{&fin.outcome},"exit_status":fin.exit_status,
            "peak_rss_kib":fin.peak_rss_kib,"reason":fin.reason});
        crate::write_json_atomic(&path, &exit)?;
        exit
    };
    let launch = read_attempt(&dir)?;
    if exit["attempt_id"].as_str() != Some(launch.attempt_id.as_str()) {
        return Err(invalid("exit receipt attempt differs from launch receipt"));
    }
    charge_exit(ledger_dir, spec, &exit)?;
    durable_rename(&dir, &done_dir(root).join(&spec.id))?;
    Ok(())
}

/// Finish a previously persisted receipt before any new job is admitted.
pub fn replay_finalization(root: &Path, dir: &Path, ledger_dir: &Path) -> Result<()> {
    let spec = read_spec(dir)?;
    finalize(
        root,
        &spec,
        &Finalization {
            outcome: "unknown".into(),
            exit_status: None,
            reason: "recovered durable receipt".into(),
            peak_rss_kib: 0,
            started_utc: String::new(),
            elapsed_ms: 0,
            measurement: ledger::Measurement::Estimated,
        },
        ledger_dir,
    )
}

fn summarize(root: &Path, stage: &str, dir: &Path) -> Value {
    let spec = read_spec(dir).ok();
    let identity = read_identity(dir).ok();
    json!({"id":dir.file_name().map(|n|n.to_string_lossy().into_owned()),"stage":stage,
        "lab":spec.map(|s|s.lab),"pid":identity.as_ref().map(|i|i.pid),"identity_matches":identity.as_ref().map(process::matches),
        "has_exit":dir.join("exit.json").is_file(),"recovery_required":dir.join("recovery-required.json").is_file(),"root":root})
}
pub fn status(root: &Path, id: Option<&str>) -> Result<Value> {
    if let Some(id) = id {
        let (stage, dir) = locate(root, id).ok_or_else(|| invalid(format!("no job {id}")))?;
        return Ok(summarize(root, stage, &dir));
    }
    let mut jobs = vec![];
    for (stage, path) in [
        ("queue", queue_dir(root)),
        ("running", running_dir(root)),
        ("done", done_dir(root)),
    ] {
        if !path.is_dir() {
            continue;
        }
        for e in fs::read_dir(path)? {
            let e = e?;
            if e.file_type()?.is_dir() {
                jobs.push(summarize(root, stage, &e.path()));
            }
        }
    }
    Ok(
        json!({"schema":"uor-r4.lab-runner-status/1","jobs":jobs,"admissions_held":root.join("admissions-held.json").exists(),"admission_blocked":root.join("admission-blocked.json").exists()}),
    )
}
pub fn tail(root: &Path, id: &str, count: usize) -> Result<String> {
    use std::io::{Read, Seek, SeekFrom};
    let (_, dir) = locate(root, id).ok_or_else(|| invalid(format!("no job {id}")))?;
    let mut out = String::new();
    for name in ["stdout.log", "stderr.log"] {
        out.push_str(&format!("== {name} ==\n"));
        if let Ok(mut file) = fs::File::open(dir.join(name)) {
            let len = file.metadata()?.len();
            file.seek(SeekFrom::Start(len.saturating_sub(65536)))?;
            let mut bytes = Vec::new();
            file.take(65536).read_to_end(&mut bytes)?;
            let text = String::from_utf8_lossy(&bytes);
            let lines: Vec<_> = text.lines().collect();
            for line in lines.iter().skip(lines.len().saturating_sub(count)) {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    Ok(out)
}

pub fn cancel(root: &Path, id: &str, ledger_dir: &Path) -> Result<()> {
    ensure_layout(root)?;
    let guard = job_lock(root, id)?;
    let (stage, dir) = locate(root, id).ok_or_else(|| invalid(format!("no job {id}")))?;
    if stage == "done" {
        return Err(invalid("job already done"));
    }
    crate::write_json_atomic(
        &dir.join("cancel.json"),
        &json!({"recorded_utc":utc_now_iso(),"reason":"explicit cancellation"}),
    )?;
    if stage == "queue" {
        let spec = read_spec(&dir)?;
        let consumed = consumed_attempt_path(id)?.exists();
        crate::write_json_atomic(
            &dir.join("exit.json"),
            &json!({"schema":EXIT_SCHEMA,"id":id,"lab":spec.lab,"attempt_id":id,
                "host":process::host_id()?,"spec_sha256":crate::coord::digest(&serde_json::to_vec(&spec)?),
                "coordination":spec.coordination,"process_state":if consumed {"unknown"}else{"never_started"},
                "outcome":if consumed {"unknown"}else{"cancelled"},"elapsed_ms":0,"wall_s":spec.wall_s,"exit_status":null,
                "reason":"cancelled in queue; consumed attempts require reconciliation; no new compute charge"}),
        )?;
        durable_rename(&dir, &done_dir(root).join(id))?;
        return Ok(());
    }
    let identity=read_identity(&dir).map_err(|_|invalid("running job lacks verified identity; cancellation marker saved, manual reconciliation required"))?;
    let spec = read_spec(&dir)?;
    let attempt = read_attempt(&dir)?;
    let peak = tree_rss_kib(identity.pid).unwrap_or(0);
    if pid_alive(identity.pid) {
        record_stop_request(&dir, spec.stop_grace_ms, "explicit cancellation")?;
        process::terminate(&identity, Duration::from_millis(spec.stop_grace_ms))?;
    }
    drop(guard);
    finalize(
        root,
        &spec,
        &Finalization {
            outcome: "cancelled".into(),
            exit_status: None,
            reason: "explicit cancellation of verified process".into(),
            peak_rss_kib: peak,
            started_utc: attempt.started_utc,
            elapsed_ms: now_ms().saturating_sub(attempt.started_ms),
            measurement: ledger::Measurement::Estimated,
        },
        ledger_dir,
    )
}
