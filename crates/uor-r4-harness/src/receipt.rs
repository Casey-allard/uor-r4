//! Writing receipts — the first half of the port that produces state rather than reading it.
//!
//! **Why this exists before more reading.** Every check so far compares the two implementations on
//! state that *Python wrote*. That proves Rust can read the shared record; it says nothing about
//! whether Rust can *add to* it. A port is not a port until a record written by one implementation
//! is read correctly by the other, and a writer that is subtly wrong is worse than no writer —
//! it corrupts the record the labs coordinate through.
//!
//! The contract it must satisfy, taken from the Python implementation:
//!
//! * **Sharded by lab** — `docs/harness/ledger/<slug>-<YYYY-MM>.jsonl`. Two labs never write the
//!   same file. This is the sharding the whole plane depends on, so the port must get it identically
//!   right, including the slug rules.
//! * **One JSON object per line**, keys sorted, `ensure_ascii` off. Python writes
//!   `json.dumps(record, ensure_ascii=False, sort_keys=True)`; a reader parses by line and does not
//!   care, but a *diff* does, and two implementations writing one file differently would make every
//!   ledger change unreadable.
//! * **Append, never rewrite.** A receipt write must not be able to lose an existing line.
//! * **`kind`, `lab`, `schema`, `ts` always present** — the contract's `required_always` for this
//!   format. A row missing any of them fails `statecheck`.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::state::{slug, StatePaths};

/// Bumped when the receipt shape changes. Must match what the Python implementation writes.
pub const RECEIPT_SCHEMA: u32 = 1;

/// One receipt, in the shape the contract requires.
#[derive(Debug, Clone)]
pub struct Receipt {
    /// Always present. `skill`, `heartbeat`, `claim`, `release`, `team`, `prep`, `receipt`.
    pub kind: String,
    /// Always present. Slugs to the shard it lands in.
    pub lab: String,
    /// Always present. RFC3339 with the local offset, as the Python side writes.
    pub ts: String,
    /// Every other field the caller supplied, rendered as JSON.
    pub fields: Map<String, Value>,
}

impl Receipt {
    /// Build a receipt, stamping the schema and defaulting the timestamp to now.
    pub fn new(kind: impl Into<String>, lab: impl Into<String>) -> Self {
        Receipt {
            kind: kind.into(),
            lab: lab.into(),
            ts: now_rfc3339(),
            fields: Map::new(),
        }
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// The line this receipt becomes. Keys sorted, matching
    /// `json.dumps(record, ensure_ascii=False, sort_keys=True)`.
    pub fn to_line(&self) -> String {
        // serde_json::Map preserves insertion order by default, so the keys are inserted sorted
        // rather than relying on a feature flag.
        let mut all: Vec<(&str, Value)> = vec![
            ("kind", Value::String(self.kind.clone())),
            ("lab", Value::String(self.lab.clone())),
            ("schema", Value::from(RECEIPT_SCHEMA)),
            ("ts", Value::String(self.ts.clone())),
        ];
        for (k, v) in &self.fields {
            all.push((k.as_str(), v.clone()));
        }
        all.sort_by(|a, b| a.0.cmp(b.0));
        let mut obj = Map::new();
        for (k, v) in all {
            obj.insert(k.to_string(), v);
        }
        Value::Object(obj).to_string()
    }

    /// A malformed receipt is rejected before it can reach the record: the four required keys are
    /// the contract's `required_always`, and a row without them fails `statecheck`.
    pub fn validate(&self) -> Result<(), String> {
        if self.kind.trim().is_empty() {
            return Err("a receipt needs a kind".into());
        }
        if slug(&self.lab).is_empty() {
            return Err(format!("{:?} does not slug to a usable lab name", self.lab));
        }
        if self.ts.trim().is_empty() {
            return Err("a receipt needs a timestamp".into());
        }
        Ok(())
    }
}

/// Where a lab's shard for a month lives.
pub fn shard_path(paths: &StatePaths, lab: &str, month: &str) -> PathBuf {
    let name = if slug(lab).is_empty() {
        "unknown".to_string()
    } else {
        slug(lab)
    };
    paths.ledger_dir().join(format!("{name}-{month}.jsonl"))
}

/// Append one receipt to its lab's shard. Returns the shard written.
///
/// Appends with `O_APPEND`, so a concurrent writer from another lab cannot interleave into this
/// file — different labs write different shards by construction, and within a shard the append is
/// atomic for the line sizes involved.
pub fn append(paths: &StatePaths, receipt: &Receipt, month: &str) -> Result<PathBuf, String> {
    receipt.validate()?;
    let shard = shard_path(paths, &receipt.lab, month);
    if let Some(parent) = shard.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&shard)
        .map_err(|e| format!("cannot open {}: {e}", shard.display()))?;
    writeln!(file, "{}", receipt.to_line())
        .map_err(|e| format!("cannot write {}: {e}", shard.display()))?;
    Ok(shard)
}

/// `YYYY-MM` for a Unix second count, in UTC — the month a shard belongs to.
///
/// UTC rather than local time on purpose: the shard name must be derivable identically by both
/// implementations on any machine, and a lab's local offset would put the same instant in different
/// months in different timezones. The receipt's own `ts` keeps the local offset for readability.
pub fn month_for(unix: i64) -> String {
    let days = unix.div_euclid(86_400);
    let (y, m, _d) = civil_from_days(days);
    format!("{y:04}-{m:02}")
}

/// Inverse of `days_from_civil`: days since the epoch to a Gregorian date. Howard Hinnant's algorithm.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Now, as RFC3339 in UTC. The Python side writes a local offset; both are valid RFC3339 and the
/// reader parses either, so the port does not need to know the machine's timezone.
pub fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    let tod = secs.rem_euclid(86_400);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60
    )
}

/// The month a shard should use right now.
pub fn current_month() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    month_for(secs)
}

/// Read the receipts from one shard file, for verification and tests.
pub fn read_shard(path: &Path) -> Vec<Value> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> StatePaths {
        StatePaths::resolve(
            "/tmp/harness-writer-test",
            Some(Path::new("/tmp/none")),
            Some("/tmp/none"),
        )
    }

    #[test]
    fn a_receipt_line_is_sorted_json() {
        let mut r = Receipt::new("heartbeat", "kimi");
        r.ts = "2026-10-13T00:00:00-04:00".into();
        r = r.with("milestone", "m1").with("count", "2/3");
        let line = r.to_line();
        // sorted: count, kind, lab, milestone, schema, ts
        let keys: Vec<&str> = line
            .trim_matches(|c| c == '{' || c == '}')
            .split("\",")
            .filter_map(|p| p.split('"').nth(1))
            .collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        assert_eq!(
            keys, sorted,
            "keys must be sorted for a readable ledger diff: {line}"
        );
    }

    /// The contract's `required_always` for the ledger format. A row missing any of these fails
    /// `statecheck`, so the writer must never emit one.
    #[test]
    fn every_line_carries_the_four_required_keys() {
        let r = Receipt::new("receipt", "claude");
        let v: Value = serde_json::from_str(&r.to_line()).expect("valid json");
        for key in ["kind", "lab", "schema", "ts"] {
            assert!(v.get(key).is_some(), "missing {key} in {}", r.to_line());
        }
    }

    #[test]
    fn the_shard_is_named_for_the_lab_and_month() {
        let p = shard_path(&paths(), "Kimi", "2026-10");
        assert!(p.ends_with("ledger/kimi-2026-10.jsonl"), "{}", p.display());
        // An unidentified lab goes to `unknown`, not to a shared file — the conflict is still
        // avoided and the gap stays visible.
        let u = shard_path(&paths(), "", "2026-10");
        assert!(
            u.ends_with("ledger/unknown-2026-10.jsonl"),
            "{}",
            u.display()
        );
    }

    #[test]
    fn a_receipt_without_a_kind_or_lab_is_rejected_before_it_reaches_the_record() {
        assert!(Receipt::new("", "kimi").validate().is_err());
        assert!(Receipt::new("heartbeat", "  ").validate().is_err());
        assert!(Receipt::new("heartbeat", "kimi").validate().is_ok());
    }

    #[test]
    fn months_are_derived_in_utc() {
        // Epochs computed independently (`datetime.fromisoformat(...).timestamp()`), because a
        // hand-written constant is exactly the kind of thing that is wrong by a year and still
        // looks plausible: the first version of this test asserted 1_760_313_600 was October 2026
        // when it is October 2025.
        assert_eq!(month_for(1_791_849_600), "2026-10", "2026-10-13T00:00:00Z");
        assert_eq!(month_for(0), "1970-01", "the epoch");
        assert_eq!(month_for(1_767_225_599), "2025-12", "2025-12-31T23:59:59Z");
        assert_eq!(
            month_for(1_767_225_600),
            "2026-01",
            "2026-01-01T00:00:00Z, the next second"
        );
    }

    /// The property that matters: a write must never lose an existing line.
    #[test]
    fn appending_preserves_earlier_receipts() {
        let dir = std::env::temp_dir().join(format!("harness-writer-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let p = StatePaths::resolve(&dir, Some(Path::new("/tmp/none")), Some("/tmp/none"));
        let first = Receipt::new("heartbeat", "kimi").with("milestone", "m1");
        let second = Receipt::new("skill", "kimi").with("skill", "expert-dispatch");
        append(&p, &first, "2026-10").expect("first write");
        append(&p, &second, "2026-10").expect("second write");
        let rows = read_shard(&shard_path(&p, "kimi", "2026-10"));
        assert_eq!(
            rows.len(),
            2,
            "the first receipt must survive the second write"
        );
        assert_eq!(rows[0]["kind"], "heartbeat");
        assert_eq!(rows[1]["kind"], "skill");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn two_labs_never_write_the_same_shard() {
        assert_ne!(
            shard_path(&paths(), "kimi", "2026-10"),
            shard_path(&paths(), "codex", "2026-10")
        );
    }

    #[test]
    fn a_timestamp_is_rfc3339_with_a_z() {
        let ts = now_rfc3339();
        assert!(ts.ends_with('Z'), "{ts}");
        assert_eq!(ts.len(), 20, "{ts}");
        assert!(
            crate::state::parse_iso_unix(&ts).is_some(),
            "must be parseable: {ts}"
        );
    }
}
