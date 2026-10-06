//! Detached local job runner for the UOR-R4 labs.
//!
//! A filesystem queue (`queue/`, `running/`, `done/`) plus a foreground
//! daemon (kept alive by launchd) admits jobs under thread/RSS/GPU/exclusive
//! limits, enforces declared kill criteria, and charges each finished job to
//! the shared model-time ledger under a file lock. All JSON records are
//! schema-versioned; see the crate README for the layouts.

#![forbid(unsafe_code)]

pub mod admission;
pub mod agent;
pub mod coord;
pub mod daemon;
pub mod delivery;
pub mod github_sync;
pub mod host;
pub mod jobs;
pub mod ledger;
pub mod maintenance;
pub mod outbox;
pub mod process;
pub mod spec;

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub enum RunnerError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(String),
    /// A declared volume currently holds fewer free bytes than its policy
    /// reserve requires. This carries its own variant instead of a formatted
    /// string so admission classifies the failure *causally*: free space is a
    /// pure function of live machine state, so a hold recorded for it
    /// re-evaluates (`host::HoldKind::Recheckable`) rather than latching
    /// admission off permanently. No reason string is ever inspected.
    VolumeReserve(crate::host::VolumeShortfall),
}

impl fmt::Display for RunnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "lab-runner I/O: {error}"),
            Self::Json(error) => write!(f, "lab-runner JSON: {error}"),
            Self::Invalid(message) => write!(f, "invalid lab-runner input: {message}"),
            // Byte-identical to the `Invalid` form this replaces, so operator
            // logs and existing receipts keep their exact wording.
            Self::VolumeReserve(shortfall) => {
                write!(f, "invalid lab-runner input: {shortfall}")
            }
        }
    }
}

impl std::error::Error for RunnerError {}

impl From<std::io::Error> for RunnerError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for RunnerError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub type Result<T> = std::result::Result<T, RunnerError>;

pub(crate) fn invalid(message: impl Into<String>) -> RunnerError {
    RunnerError::Invalid(message.into())
}

/// Default queue root when neither `--root` nor `UOR_RUNNER_ROOT` is given.
pub const DEFAULT_ROOT: &str = ".local/share/uor-r4/runner";
pub fn default_root() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or_else(|| invalid("HOME not set; pass --root"))?;
    Ok(PathBuf::from(home).join(DEFAULT_ROOT))
}
/// Default ledger directory, resolved against `$HOME` at runtime.
pub const DEFAULT_LEDGER_SUFFIX: &str = ".local/share/uor-r4/ledger";

pub fn default_ledger_dir() -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .ok_or_else(|| invalid("HOME is not set; pass --ledger-dir explicitly"))?;
    Ok(PathBuf::from(home).join(DEFAULT_LEDGER_SUFFIX))
}

/// Days-from-civil / civil-from-days (Howard Hinnant's algorithms) so UTC
/// timestamps need no date dependency.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `2026-09-29T20:05:25Z` for the current system time.
pub fn utc_now_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        sod / 3600,
        (sod / 60) % 60,
        sod % 60
    )
}

/// Compact filename-safe variant: `20260929T200525Z`.
pub fn utc_now_compact() -> String {
    utc_now_iso().replace(['-', ':'], "")
}

/// Map an arbitrary label (lab name, job id) to a filename-safe component.
pub fn sanitize_component(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Write `bytes` to `path` through a create-new temporary plus rename, so a
/// reader never observes a partial file (pattern follows
/// `uor-r4-graph-compiler/src/observation.rs`).
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| invalid(format!("{} has no parent directory", path.display())))?;
    let name = path
        .file_name()
        .ok_or_else(|| invalid(format!("{} has no final component", path.display())))?
        .to_string_lossy();
    for _ in 0..64 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let tmp = dir.join(format!(".{name}.tmp-{}-{sequence}", std::process::id()));
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        };
        let write_result = file.write_all(bytes).and_then(|()| file.sync_all());
        drop(file);
        if let Err(error) = write_result {
            let _ = fs::remove_file(&tmp);
            return Err(error.into());
        }
        match fs::rename(&tmp, path) {
            Ok(()) => {
                fs::File::open(dir)?.sync_all()?;
                return Ok(());
            }
            Err(error) => {
                let _ = fs::remove_file(&tmp);
                return Err(error.into());
            }
        }
    }
    Err(invalid(format!(
        "could not claim a temporary name for {}",
        path.display()
    )))
}

pub fn write_json_atomic(path: &Path, value: &serde_json::Value) -> Result<()> {
    atomic_write(path, &serde_json::to_vec_pretty(value)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_iso_format_is_stable() {
        let stamp = utc_now_iso();
        assert_eq!(stamp.len(), 20);
        assert!(stamp.ends_with('Z'));
        assert_eq!(&stamp[4..5], "-");
        assert_eq!(&stamp[10..11], "T");
        assert_eq!(utc_now_compact().len(), 16);
    }

    #[test]
    fn civil_from_days_matches_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // Independently computed (Python datetime) anchors.
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        assert_eq!(civil_from_days(20_089), (2025, 1, 1));
        assert_eq!(civil_from_days(20_725), (2026, 9, 29));
    }

    #[test]
    fn sanitize_component_replaces_unsafe_bytes() {
        assert_eq!(sanitize_component("Lab 1 (Claude)"), "Lab-1--Claude-");
        assert_eq!(sanitize_component("plain-1_x"), "plain-1_x");
    }

    #[test]
    fn atomic_write_round_trips() {
        let dir = std::env::temp_dir().join(format!(
            "lab-runner-lib-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.json");
        atomic_write(&path, b"{\"a\":1}").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"{\"a\":1}");
        assert!(fs::read_dir(&dir)
            .unwrap()
            .all(|e| e.unwrap().file_name() == "out.json"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
