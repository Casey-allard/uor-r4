//! Host policy is separate from scientific acceptance. Missing observations
//! block admission; neither a path nor a declared RSS proves capacity.
use crate::spec::JobSpec;
use crate::{invalid, Result, RunnerError};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumePolicy {
    pub id: String,
    pub path: PathBuf,
    /// macOS filesystem UUID. Required for removable mount points.
    pub volume_uuid: Option<String>,
    pub sentinel: PathBuf,
    pub sentinel_value: String,
    pub reserve_bytes: u64,
    pub stop_margin_bytes: u64,
}
fn normal_pressure() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostPolicy {
    pub schema: String,
    #[serde(default)]
    pub admission_enabled: bool,
    pub max_threads: u32,
    pub max_rss_gib: f64,
    pub max_jobs: usize,
    /// Raising normal(1) to warning(2) is an explicit host-policy change;
    /// critical pressure is never permitted for admission.
    #[serde(default = "normal_pressure")]
    pub admission_pressure_max: u32,
    /// Reviewed small CPU validation specs eligible at warning pressure.
    /// Digests use the same canonical JobSpec serialization as reservations.
    #[serde(default)]
    pub warning_validation_specs: Vec<String>,
    #[serde(default)]
    pub coordination_repo: Option<PathBuf>,
    pub volumes: Vec<VolumePolicy>,
}

impl HostPolicy {
    pub fn load(root: &Path) -> Result<Self> {
        let policy: Self = serde_json::from_slice(&fs::read(root.join("host-policy.json"))?)?;
        if policy.schema != "uor-r4.host-policy/1"
            || policy.max_threads == 0
            || policy.max_threads > 8
            || !policy.max_rss_gib.is_finite()
            || policy.max_rss_gib <= 0.0
            || policy.max_rss_gib > 11.0
            || !matches!(policy.admission_pressure_max, 1 | 2)
            || policy.max_jobs == 0
            || policy.max_jobs > policy.max_threads as usize
            || policy.volumes.is_empty()
        {
            return Err(invalid("host policy schema or resource ceilings invalid"));
        }
        let mut approved = std::collections::BTreeSet::new();
        for digest in &policy.warning_validation_specs {
            if digest.len() != 64
                || !digest
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                || !approved.insert(digest)
            {
                return Err(invalid(
                    "invalid or duplicate warning validation spec digest",
                ));
            }
        }
        let mut ids = std::collections::BTreeSet::new();
        for volume in &policy.volumes {
            if volume.id.is_empty()
                || !ids.insert(&volume.id)
                || !volume.path.is_absolute()
                || !volume.sentinel.is_absolute()
                || !volume.sentinel.starts_with(&volume.path)
                || volume.sentinel_value.is_empty()
                || volume.stop_margin_bytes < 128 * 1024 * 1024
            {
                return Err(invalid("invalid host volume identity/reserve contract"));
            }
            if volume.path.starts_with("/Volumes") && volume.volume_uuid.is_none() {
                return Err(invalid("removable volume requires filesystem UUID"));
            }
        }
        Ok(policy)
    }
}

fn xml_string(text: &str, key: &str) -> Option<String> {
    let tail = text.split_once(&format!("<key>{key}</key>"))?.1;
    Some(
        tail.split_once("<string>")?
            .1
            .split_once("</string>")?
            .0
            .to_string(),
    )
}

pub fn free_bytes(path: &Path) -> Result<u64> {
    let output = Command::new("/bin/df")
        .args(["-kP"])
        .arg(path)
        .env("LC_ALL", "C")
        .output()?;
    if !output.status.success() {
        return Err(invalid("df could not observe storage"));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let fields: Vec<_> = text
        .lines()
        .nth(1)
        .unwrap_or("")
        .split_whitespace()
        .collect();
    fields
        .get(3)
        .and_then(|n| n.parse::<u64>().ok())
        .and_then(|n| n.checked_mul(1024))
        .ok_or_else(|| invalid("df returned no trustworthy free byte count"))
}

pub fn verify_volume(volume: &VolumePolicy) -> Result<u64> {
    if !volume.path.is_dir() {
        return Err(invalid(format!("volume {} absent", volume.id)));
    }
    if let Some(uuid) = &volume.volume_uuid {
        let output = Command::new("/usr/sbin/diskutil")
            .args(["info", "-plist"])
            .arg(&volume.path)
            .output()?;
        let text = String::from_utf8_lossy(&output.stdout);
        if !output.status.success()
            || xml_string(&text, "VolumeUUID").as_deref() != Some(uuid)
            || xml_string(&text, "MountPoint").as_deref() != volume.path.to_str()
        {
            return Err(invalid(format!(
                "volume {} mounted identity mismatch",
                volume.id
            )));
        }
    }
    if fs::read_to_string(&volume.sentinel)?.trim() != volume.sentinel_value {
        return Err(invalid(format!("volume {} sentinel mismatch", volume.id)));
    }
    free_bytes(&volume.path)
}

fn sum_storage(specs: &[JobSpec], id: &str, admission: bool) -> Result<u64> {
    specs
        .iter()
        .flat_map(|s| &s.storage)
        .filter(|s| s.volume == id)
        .try_fold(0u64, |sum, s| {
            let bytes = if admission {
                s.additional_bytes.checked_add(s.checkpoint_bytes)
            } else {
                Some(s.checkpoint_bytes)
            };
            bytes
                .and_then(|b| sum.checked_add(b))
                .ok_or_else(|| invalid("storage reservation overflow"))
        })
}

/// Live volume free-space shortfall: the declared reserve cannot be honoured
/// right now. Free bytes are a pure function of current machine state, so this
/// is the one storage failure that re-evaluates instead of latching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VolumeShortfall {
    pub volume: String,
    pub free: u64,
    pub required: u64,
}

impl fmt::Display for VolumeShortfall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "volume {} free bytes {} below required {}",
            self.volume, self.free, self.required
        )
    }
}

pub fn check_storage(policy: &HostPolicy, specs: &[JobSpec], admission: bool) -> Result<()> {
    for spec in specs {
        if spec.storage.is_empty() {
            return Err(invalid(
                "production jobs require explicit storage reservations",
            ));
        }
        for reservation in &spec.storage {
            if !policy.volumes.iter().any(|v| v.id == reservation.volume) {
                return Err(invalid("unknown reserved volume"));
            }
        }
    }
    check_volume_reserve(policy, specs, admission)
}

/// The live volume free-space predicate: every declared volume must currently
/// hold `reserve_bytes` plus `stop_margin_bytes` plus the storage reserved by
/// `specs`.
///
/// `check_storage` is built from this same function, so the daemon re-evaluates
/// exactly the check that produced a hold rather than a second implementation of
/// it. A shortfall is returned as `RunnerError::VolumeReserve` because it is
/// re-evaluable; the remaining failures (absent volume, mounted-identity or
/// sentinel mismatch, arithmetic overflow) are not observable-as-recovered from
/// machine state alone and stay unclassified so a hold for them stays terminal.
pub fn check_volume_reserve(policy: &HostPolicy, specs: &[JobSpec], admission: bool) -> Result<()> {
    for volume in &policy.volumes {
        let free = verify_volume(volume)?;
        let required = volume
            .reserve_bytes
            .checked_add(volume.stop_margin_bytes)
            .and_then(|v| v.checked_add(sum_storage(specs, &volume.id, admission).ok()?))
            .ok_or_else(|| invalid("storage requirement overflow"))?;
        if free < required {
            return Err(RunnerError::VolumeReserve(VolumeShortfall {
                volume: volume.id.clone(),
                free,
                required,
            }));
        }
    }
    Ok(())
}

pub fn memory_pressure() -> Result<u32> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/sbin/sysctl")
            .args(["-n", "kern.memorystatus_vm_pressure_level"])
            .output()?;
        if !output.status.success() {
            return Err(invalid("memory pressure unavailable"));
        }
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .map_err(|_| invalid("invalid memory pressure observation"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let text = fs::read_to_string("/proc/meminfo")?;
        let parse = |key: &str| {
            text.lines()
                .find(|l| l.starts_with(key))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|n| n.parse::<u64>().ok())
        };
        let total = parse("MemTotal:").ok_or_else(|| invalid("memory total unavailable"))?;
        let free = parse("MemAvailable:").ok_or_else(|| invalid("available memory unavailable"))?;
        Ok(if free < total / 20 {
            4
        } else if free < total / 10 {
            2
        } else {
            1
        })
    }
}

/// Concurrent jobs are limited by aggregate resources, not artificial lane pairs.
fn check_concurrent_jobs(specs: &[JobSpec]) -> Result<()> {
    if specs.len() <= 1 {
        return Ok(());
    }
    if specs.iter().any(|s| s.exclusive) {
        return Err(invalid("exclusive measurement requires the host"));
    }
    for spec in specs {
        spec.validate()?;
    }
    let cargo: Vec<_> = specs.iter().filter(|s| s.cargo).collect();
    if cargo.len() > 1 {
        let target = |s: &JobSpec| -> Result<PathBuf> {
            let value = s
                .env
                .as_ref()
                .and_then(|e| e.get("CARGO_TARGET_DIR"))
                .ok_or_else(|| {
                    invalid("concurrent Cargo requires explicit separate target directories")
                })?;
            let path = Path::new(value);
            if !path.is_absolute() || !path.is_dir() {
                return Err(invalid(
                    "concurrent Cargo target must be an existing absolute directory",
                ));
            }
            Ok(fs::canonicalize(path)?)
        };
        let targets = cargo
            .iter()
            .map(|s| target(s))
            .collect::<Result<Vec<_>>>()?;
        for (i, a) in targets.iter().enumerate() {
            for b in &targets[i + 1..] {
                if a.starts_with(b) || b.starts_with(a) {
                    return Err(invalid("concurrent Cargo target directories overlap"));
                }
            }
        }
    }
    Ok(())
}

/// Warning pressure alone need not stall a reviewed, bounded CPU check.
/// This only decides pressure eligibility; all other admission checks still run.
fn check_pressure(policy: &HostPolicy, spec: &JobSpec, pressure: u32) -> Result<()> {
    match pressure {
        1 => Ok(()),
        2 if policy.admission_pressure_max == 2 => Ok(()),
        2 => {
            let digest = crate::coord::digest(&serde_json::to_vec(spec)?);
            if policy.warning_validation_specs.contains(&digest)
                && (1..=2).contains(&spec.threads)
                && spec.rss_gib.is_finite()
                && spec.rss_gib > 0.0
                && spec.rss_gib <= 2.0
                && (1..=600).contains(&spec.wall_s)
                && !spec.gpu
            {
                Ok(())
            } else {
                Err(invalid("memory pressure warning; requires approved CPU validation spec within 2 threads, 2 GiB and 600 seconds"))
            }
        }
        _ => Err(invalid(
            "critical or unknown memory pressure; defer new workload",
        )),
    }
}

pub fn check_admission(
    root: &Path,
    policy: &HostPolicy,
    spec: &JobSpec,
    running: &[JobSpec],
) -> Result<()> {
    if matches!(spec.kill_criterion.kind, crate::spec::KillKind::Command) {
        return Err(invalid(
            "production command kill criteria require an owned supervisor; use wall or log_match",
        ));
    }
    check_pressure(policy, spec, memory_pressure()?)?;
    if !policy.admission_enabled {
        return Err(invalid("host admission disabled"));
    }
    if running.len() >= policy.max_jobs {
        return Err(invalid("host job slots reserved"));
    }
    if spec.provenance.is_none() {
        return Err(invalid("production job requires immutable provenance"));
    }
    let mut specs = running.to_vec();
    specs.push(spec.clone());
    let threads: u64 = specs.iter().map(|s| u64::from(s.threads)).sum();
    let rss: f64 = specs.iter().map(|s| s.rss_gib).sum();
    if threads > u64::from(policy.max_threads) || rss > policy.max_rss_gib {
        return Err(invalid("host resources reserved"));
    }
    check_concurrent_jobs(&specs)?;
    let claim = spec
        .coordination
        .as_ref()
        .ok_or_else(|| invalid("production job lacks coordination claim"))?;
    let repo = policy
        .coordination_repo
        .as_ref()
        .ok_or_else(|| invalid("host lacks coordination repository"))?;
    crate::coord::validate_job_admission(
        repo,
        claim.issue,
        &claim.session,
        claim.epoch,
        &claim.work_card,
        &claim.attempt_id,
    )?;
    crate::coord::validate_reserved_spec(repo, root, spec, running)?;
    check_storage(policy, &specs, true)
}

/// Explicit test mode is restricted to inert fixture programs in a temp
/// root. It cannot be used to bypass production policy for a model run.
pub fn validate_test_job(root: &Path, spec: &JobSpec) -> Result<()> {
    spec.validate()?;
    let temp = fs::canonicalize(std::env::temp_dir())?;
    if !fs::canonicalize(root)?.starts_with(&temp)
        || spec.threads != 1
        || spec.rss_gib > 1.0
        || spec.wall_s > 60
        || spec.gpu
        || spec.cargo
        || spec.env.is_some()
    {
        return Err(invalid(
            "test mode permits only bounded inert fixtures in temporary roots",
        ));
    }
    let sleep = |value: &str| {
        value
            .parse::<f64>()
            .ok()
            .is_some_and(|n| n.is_finite() && n > 0.0 && n <= 60.0)
    };
    if ![
        "sleep",
        "/bin/sleep",
        "true",
        "/usr/bin/true",
        "false",
        "/usr/bin/false",
        "sh",
        "/bin/sh",
    ]
    .contains(&spec.argv[0].as_str())
    {
        return Err(invalid(
            "test executable must be an explicit system fixture",
        ));
    }
    let program = Path::new(&spec.argv[0])
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let accepted = match program {
        "sleep" => spec.argv.len() == 2 && sleep(&spec.argv[1]),
        "true" | "false" => spec.argv.len() == 1,
        "sh" => {
            spec.argv.len() == 3
                && spec.argv[1] == "-c"
                && spec.argv[2].split(';').all(|part| {
                    let words: Vec<_> = part.split_whitespace().collect();
                    match words.first().copied() {
                        Some("sleep") => {
                            (words.len() == 2 || (words.len() == 3 && words[2] == "&"))
                                && sleep(words[1])
                        }
                        Some("echo") => words.iter().skip(1).all(|word| {
                            word.bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
                        }),
                        Some("exit") => words.len() == 2 && words[1].parse::<u8>().is_ok(),
                        _ => false,
                    }
                })
        }
        _ => false,
    };
    if !accepted || matches!(spec.kill_criterion.kind, crate::spec::KillKind::Command) {
        return Err(invalid(
            "test mode refuses non-fixture executable or shell program",
        ));
    }
    Ok(())
}

/// Whether an admission hold re-evaluates or latches.
///
/// `admissions-held.json` is gated on **existence**. A hold recorded for a
/// transient machine condition therefore latched admission off permanently:
/// free space was measured fresh on every call, but nothing re-read the file
/// once the volume recovered. The kind is decided where the condition is
/// observed, never by inspecting the reason string.
///
/// * `Terminal` — the hold records an unreconciled outcome (lost process
///   identity, unknown exit status, unreadable receipt, restart that needs
///   checkpoint reconciliation). Re-observing machine state cannot resolve it,
///   so the recheck path must never clear it; only an operator may.
/// * `Recheckable` — the hold records a pure function of current machine state.
///   It names the predicate that produced it, and the daemon re-runs that exact
///   predicate and releases the hold as soon as it passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldKind {
    Terminal,
    Recheckable(HoldCheck),
}

/// The live predicate a `Recheckable` hold names, so re-evaluation runs the same
/// check that produced the hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HoldCheck {
    /// `check_volume_reserve` with the current policy and running set.
    VolumeReserve,
    /// `memory_pressure` below the critical level that refused admission.
    MemoryPressure,
    /// Aggregate observed RSS of the running jobs against the host ceiling.
    ObservedRss,
}

/// Wire form of the kind. Kept separate from `HoldKind` because the predicate is
/// a sibling JSON field rather than a nested object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RecordedHoldKind {
    Terminal,
    Recheckable,
}

impl Default for RecordedHoldKind {
    fn default() -> Self {
        Self::Terminal
    }
}

/// The admission hold record written to `admissions-held.json`.
pub const HOLD_FILE: &str = "admissions-held.json";

/// Persisted hold. `kind`/`check` were added after the first holds were written,
/// so both default when absent: a legacy record stays `Terminal` and keeps its
/// exact previous latching behaviour rather than being released by a guess.
///
/// Deliberately *not* `deny_unknown_fields` (used for the other versioned
/// records in this crate): a hold this build cannot fully parse must still
/// register as a hold, and the safe reading of an unclassified record is the
/// latching one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldRecord {
    pub schema: String,
    pub at: String,
    pub reason: String,
    #[serde(default)]
    kind: RecordedHoldKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    check: Option<HoldCheck>,
}

impl HoldRecord {
    fn of(kind: HoldKind, reason: &str) -> Self {
        let (kind, check) = match kind {
            HoldKind::Terminal => (RecordedHoldKind::Terminal, None),
            HoldKind::Recheckable(check) => (RecordedHoldKind::Recheckable, Some(check)),
        };
        Self {
            schema: "uor-r4.admission-hold/1".into(),
            at: crate::utc_now_iso(),
            reason: reason.into(),
            kind,
            check,
        }
    }

    /// The classification as recorded. A record marked recheckable that names no
    /// predicate cannot be re-run, so it reads back as terminal.
    pub fn kind(&self) -> HoldKind {
        match (self.kind, self.check) {
            (RecordedHoldKind::Recheckable, Some(check)) => HoldKind::Recheckable(check),
            _ => HoldKind::Terminal,
        }
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// `Ok(None)` when no hold is recorded. A record that exists but cannot be
    /// parsed is reported as a terminal hold: the safe direction.
    pub fn load(root: &Path) -> Result<Option<Self>> {
        let bytes = match fs::read(root.join(HOLD_FILE)) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        Ok(Some(serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            Self::of(HoldKind::Terminal, "unreadable admission hold record")
        })))
    }
}

/// Record an admission hold. `kind` is decided at the call site where the
/// condition is observed; never derive it from `reason`.
pub fn hold(root: &Path, kind: HoldKind, reason: &str) -> Result<()> {
    crate::write_json_atomic(
        &root.join(HOLD_FILE),
        &serde_json::to_value(HoldRecord::of(kind, reason))?,
    )
}

/// The live predicate a failed admission check implies, if any. Typed and
/// causal: the reason string is never inspected, so a terminal failure cannot
/// be reclassified as recheckable by wording.
pub fn recheck(error: &RunnerError) -> Option<HoldCheck> {
    match error {
        RunnerError::VolumeReserve(_) => Some(HoldCheck::VolumeReserve),
        _ => None,
    }
}

/// Classify a failed check for hold recording: a live shortfall re-evaluates,
/// every other failure stays terminal.
pub fn hold_kind_for(error: &RunnerError) -> HoldKind {
    match recheck(error) {
        Some(check) => HoldKind::Recheckable(check),
        None => HoldKind::Terminal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_rejects_unowned_command_criterion() {
        let spec = JobSpec::parse(&serde_json::to_vec(&serde_json::json!({
            "id":"command-test","lab":"test","cwd":std::env::temp_dir(),"argv":["/bin/sleep","1"],
            "threads":1,"rss_gib":0.1,"wall_s":2,"kill_criterion":{"kind":"command","value":"sleep 20 & wait"}
        })).unwrap()).unwrap();
        let policy = HostPolicy {
            schema: "uor-r4.host-policy/1".into(),
            admission_enabled: true,
            max_threads: 1,
            max_rss_gib: 1.0,
            max_jobs: 1,
            admission_pressure_max: 1,
            warning_validation_specs: vec![],
            coordination_repo: None,
            volumes: vec![],
        };
        assert!(check_admission(&std::env::temp_dir(), &policy, &spec, &[])
            .unwrap_err()
            .to_string()
            .contains("command kill criteria"));
    }
    fn pressure_fixture() -> (HostPolicy, JobSpec) {
        let spec = JobSpec::parse(
            &serde_json::to_vec(&serde_json::json!({
                "id":"small-check","lab":"test","cwd":std::env::temp_dir(),"argv":["/usr/bin/true"],
                "threads":2,"rss_gib":2.0,"wall_s":600,"kill_criterion":{"kind":"wall"}
            }))
            .unwrap(),
        )
        .unwrap();
        let policy: HostPolicy = serde_json::from_value(serde_json::json!({
            "schema":"uor-r4.host-policy/1", "admission_enabled":true,
            "max_threads":8,"max_rss_gib":11.0,"max_jobs":1,"volumes":[]
        }))
        .unwrap();
        (policy, spec)
    }

    fn approve(policy: &mut HostPolicy, spec: &JobSpec) {
        policy.warning_validation_specs =
            vec![crate::coord::digest(&serde_json::to_vec(spec).unwrap())];
    }

    #[test]
    fn warning_requires_exact_reviewed_spec_and_legacy_policy_stays_normal() {
        let (mut policy, spec) = pressure_fixture();
        assert!(policy.warning_validation_specs.is_empty());
        assert!(check_pressure(&policy, &spec, 1).is_ok());
        assert!(check_pressure(&policy, &spec, 2).is_err());
        approve(&mut policy, &spec);
        assert!(check_pressure(&policy, &spec, 2).is_ok());
        let mut changed = spec.clone();
        changed.argv.push("different-work".into());
        assert!(check_pressure(&policy, &changed, 2).is_err());
        changed = spec.clone();
        changed.id = "new-attempt".into();
        assert!(check_pressure(&policy, &changed, 2).is_err());
    }

    #[test]
    fn reviewed_spec_cannot_exceed_small_check_bounds() {
        let (mut policy, spec) = pressure_fixture();
        for bad in 0..6 {
            let mut changed = spec.clone();
            match bad {
                0 => changed.threads = 3,
                1 => changed.rss_gib = 2.001,
                2 => changed.wall_s = 601,
                3 => changed.gpu = true,
                4 => changed.wall_s = 0,
                _ => changed.rss_gib = 0.0,
            }
            approve(&mut policy, &changed);
            assert!(check_pressure(&policy, &changed, 2).is_err(), "case {bad}");
        }
    }

    #[test]
    fn critical_and_unknown_pressure_never_admit_even_with_global_override() {
        let (mut policy, spec) = pressure_fixture();
        approve(&mut policy, &spec);
        for max in [1, 2] {
            policy.admission_pressure_max = max;
            for pressure in [0, 3, 4, 8, u32::MAX] {
                assert!(check_pressure(&policy, &spec, pressure).is_err());
            }
        }
    }

    #[test]
    fn pressure_approval_does_not_skip_disabled_admission() {
        let (mut policy, spec) = pressure_fixture();
        approve(&mut policy, &spec);
        policy.admission_enabled = false;
        let error = check_admission(&std::env::temp_dir(), &policy, &spec, &[]).unwrap_err();
        // Host pressure may independently reject on a stressed test host.
        assert!(error.to_string().contains("disabled") || error.to_string().contains("pressure"));
    }

    #[test]
    fn concurrency_has_no_lane_quota_and_preserves_exclusivity() {
        let (_, work) = pressure_fixture();
        let mut validation = work.clone();
        validation.id = "validation".into();
        validation.validation_lane = true;
        assert!(check_concurrent_jobs(&[work.clone(), validation.clone()]).is_ok());
        assert!(check_concurrent_jobs(&[work.clone(), work.clone(), work.clone()]).is_ok());
        assert!(check_concurrent_jobs(&[validation.clone(), validation.clone()]).is_ok());
        let mut exclusive = work;
        exclusive.exclusive = true;
        assert!(check_concurrent_jobs(&[exclusive, validation]).is_err());
    }

    #[test]
    fn concurrent_cargo_requires_disjoint_real_caches() {
        let (_, mut work) = pressure_fixture();
        let mut validation = work.clone();
        validation.validation_lane = true;
        work.cargo = true;
        validation.cargo = true;
        assert!(check_concurrent_jobs(&[work.clone(), validation.clone()]).is_err());
        let root = std::env::temp_dir().join(format!("uor-cache-lanes-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let a = root.join("a");
        let b = root.join("b");
        fs::create_dir(&a).unwrap();
        fs::create_dir(&b).unwrap();
        work.env = Some(std::collections::BTreeMap::from([(
            "CARGO_TARGET_DIR".into(),
            a.to_string_lossy().into_owned(),
        )]));
        validation.env = Some(std::collections::BTreeMap::from([(
            "CARGO_TARGET_DIR".into(),
            b.to_string_lossy().into_owned(),
        )]));
        assert!(check_concurrent_jobs(&[work.clone(), validation.clone()]).is_ok());
        validation.env = work.env.clone();
        assert!(check_concurrent_jobs(&[work.clone(), validation.clone()]).is_err());
        #[cfg(unix)]
        {
            let alias = root.join("alias");
            std::os::unix::fs::symlink(&a, &alias).unwrap();
            validation.env.as_mut().unwrap().insert(
                "CARGO_TARGET_DIR".into(),
                alias.to_string_lossy().into_owned(),
            );
            assert!(check_concurrent_jobs(&[work, validation]).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_serialization_omits_lane_and_new_lane_enforces_limits() {
        let (_, mut spec) = pressure_fixture();
        assert!(!serde_json::to_value(&spec)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("validation_lane"));
        spec.validation_lane = true;
        assert!(spec.validate().is_ok());
        spec.wall_s = 601;
        assert!(spec.validate().is_err());
    }

    #[test]
    fn absent_volume_is_never_created() {
        let path = std::env::temp_dir().join(format!("absent-uor-volume-{}", std::process::id()));
        let policy = VolumePolicy {
            id: "missing".into(),
            path: path.clone(),
            volume_uuid: None,
            sentinel: path.join("identity"),
            sentinel_value: "x".into(),
            reserve_bytes: 0,
            stop_margin_bytes: 128 * 1024 * 1024,
        };
        assert!(verify_volume(&policy).is_err());
        assert!(!path.exists());
    }

    /// A volume on a real directory with a matching sentinel, so the live
    /// free-space predicate can actually be evaluated.
    fn live_volume(root: &Path, reserve_bytes: u64) -> VolumePolicy {
        let sentinel = root.join("identity");
        fs::write(&sentinel, "uor-r4-fixture\n").unwrap();
        VolumePolicy {
            id: "internal".into(),
            path: root.to_path_buf(),
            volume_uuid: None,
            sentinel,
            sentinel_value: "uor-r4-fixture".into(),
            reserve_bytes,
            stop_margin_bytes: 0,
        }
    }

    #[test]
    fn volume_shortfall_keeps_its_message_and_is_the_only_recheckable_failure() {
        let root = std::env::temp_dir().join(format!("uor-reserve-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let policy = HostPolicy {
            schema: "uor-r4.host-policy/1".into(),
            admission_enabled: true,
            max_threads: 8,
            max_rss_gib: 11.0,
            max_jobs: 1,
            admission_pressure_max: 1,
            warning_validation_specs: vec![],
            coordination_repo: None,
            volumes: vec![live_volume(&root, u64::MAX / 2)],
        };
        let error = check_volume_reserve(&policy, &[], false).unwrap_err();
        // Operator-visible wording is a contract: it must not drift when the
        // failure gains a type, so the exact historical message is pinned.
        // Asserting through the carried values (rather than a second live `df`
        // read) keeps this deterministic while other work writes to the volume.
        let RunnerError::VolumeReserve(shortfall) = &error else {
            panic!("expected a live volume shortfall, got {error}");
        };
        assert_eq!(shortfall.volume, "internal");
        assert_eq!(shortfall.required, u64::MAX / 2);
        assert!(
            shortfall.free > 0,
            "the recorded free reading must be a real observation"
        );
        assert_eq!(
            error.to_string(),
            format!(
                "invalid lab-runner input: volume internal free bytes {} below required {}",
                shortfall.free, shortfall.required
            )
        );
        assert_eq!(recheck(&error), Some(HoldCheck::VolumeReserve));
        assert_eq!(
            hold_kind_for(&error),
            HoldKind::Recheckable(HoldCheck::VolumeReserve)
        );

        // A volume that cannot be observed is *not* a shortfall: a missing
        // sentinel must stay terminal rather than be released by re-evaluation.
        let mut unobservable = policy.clone();
        unobservable.volumes = vec![live_volume(&root, 0)];
        fs::remove_file(root.join("identity")).unwrap();
        let error = check_volume_reserve(&unobservable, &[], false).unwrap_err();
        assert_eq!(recheck(&error), None);
        assert_eq!(hold_kind_for(&error), HoldKind::Terminal);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn hold_records_classify_by_call_site_and_default_to_terminal() {
        let root = std::env::temp_dir().join(format!("uor-hold-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        hold(
            &root,
            HoldKind::Recheckable(HoldCheck::VolumeReserve),
            "volume internal free bytes 1 below required 2",
        )
        .unwrap();
        let written: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(HOLD_FILE)).unwrap()).unwrap();
        assert_eq!(written["schema"], "uor-r4.admission-hold/1");
        assert_eq!(written["kind"], "recheckable");
        assert_eq!(written["check"], "volume-reserve");
        let record = HoldRecord::load(&root).unwrap().unwrap();
        assert_eq!(
            record.kind(),
            HoldKind::Recheckable(HoldCheck::VolumeReserve)
        );

        hold(&root, HoldKind::Terminal, "running process lost identity").unwrap();
        let written: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(HOLD_FILE)).unwrap()).unwrap();
        assert_eq!(written["kind"], "terminal");
        assert!(written.get("check").is_none());

        // The hold that latched this host off predates the `kind` field. It must
        // read back as terminal (latching) rather than be released by a guess.
        let legacy = br#"{"at":"2026-10-06T03:59:13Z","reason":"invalid lab-runner input: volume internal free bytes 27382095872 below required 43083890688","schema":"uor-r4.admission-hold/1"}"#;
        fs::write(root.join(HOLD_FILE), legacy).unwrap();
        let record = HoldRecord::load(&root).unwrap().unwrap();
        assert_eq!(record.kind(), HoldKind::Terminal);
        assert!(record.reason().contains("27382095872"));

        // A record this build cannot parse must still register as a hold.
        fs::write(root.join(HOLD_FILE), b"{ not json").unwrap();
        let record = HoldRecord::load(&root).unwrap().unwrap();
        assert_eq!(record.kind(), HoldKind::Terminal);
        assert!(record.reason().contains("unreadable"));

        assert!(
            HoldRecord::load(&std::env::temp_dir().join("uor-no-such-hold-root"))
                .unwrap()
                .is_none()
        );
        fs::remove_dir_all(root).unwrap();
    }
}
