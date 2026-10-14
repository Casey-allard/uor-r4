//! The `budget` rules, as Rust (D30 check-port step 7 of 10).
//!
//! # The three `budget` rules, and which are here
//!
//! | rule | severity | in this unit |
//! |---|---|---|
//! | `budget.receipts_parse` | error | **yes** |
//! | `budget.disk_floor` | warn | **yes** |
//! | `budget.no_new_spending_class` | info | no — it has no check |
//!
//! ## Why `no_new_spending_class` is not ported
//!
//! **It has no check function.** Its registration is `info` severity and its whole content is that
//! **the owner caps are printed** so a session cannot silently adopt a new spending class. There is
//! nothing to decide and nothing to fail — the rule is a *display*, and the display lives in the
//! `budget` command's own output.
//!
//! Porting it here would mean a function that always returns no findings, which is **a rule that
//! cannot fail dressed as a rule** — the shape this line keeps flagging. It belongs with the `budget`
//! command's rendering, not in the rule set.
//!
//! ## The two that ARE checks
//!
//! **`receipts_parse` is the one that protects every other rule's evidence.** The ledger is where the
//! record of what happened lives; a line that will not parse is **not a missing datapoint but a
//! corrupt one**, and a reader who silently skips it draws conclusions from the rest. So a bad line is
//! an **error**, not a warning.
//!
//! **`disk_floor` is the owner's 30–70 GB working band.** Below 30 GB is a **stop condition** in the
//! owner's rules, so this is the check that gives it a machine.

use std::path::PathBuf;

use crate::state::StatePaths;

pub use crate::preprules::Finding;

/// The owner's working band, in GB. **Below the low end is a stop condition, not a warning to note.**
pub const DISK_BAND_GB: (f64, f64) = (30.0, 70.0);

fn error(rule_id: &'static str, message: String, path: String, fix: &str) -> Finding {
    Finding {
        rule_id,
        severity: "error",
        message,
        path,
        fix: fix.into(),
    }
}

fn warn(rule_id: &'static str, message: String, path: String, fix: &str) -> Finding {
    Finding {
        rule_id,
        severity: "warn",
        message,
        path,
        fix: fix.into(),
    }
}

/// Every receipt line in the sharded ledger parses, is an object, and carries `ts` and `kind`.
///
/// **Three distinct failures, kept apart because the fixes differ:**
///
/// - **not JSON** — the line is corrupt, and the fix is to repair or drop *that line*;
/// - **JSON but not an object** — a bare array or scalar, which is not a receipt at all;
/// - **an object missing `ts` or `kind`** — parseable and meaningless, **and the dangerous one**,
///   because it looks like a record while carrying none of the fields a reader filters on.
///
/// The line number is in every message, because "the ledger has a bad line" is not actionable and
/// "line 47 of `kimi-2026-10.jsonl`" is.
pub fn receipts_parse(paths: &StatePaths) -> Vec<Finding> {
    let dir = paths.repo.join("docs").join("harness").join("ledger");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .collect();
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let rel = f
            .strip_prefix(&paths.repo)
            .unwrap_or(&f)
            .display()
            .to_string();
        let Ok(text) = std::fs::read_to_string(&f) else {
            out.push(error(
                "budget.receipts_parse",
                format!("cannot read ledger: {rel}"),
                rel,
                "check the file and its permissions",
            ));
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            let n = i + 1;
            if line.trim().is_empty() {
                continue;
            }
            let parsed: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(e) => {
                    out.push(error(
                        "budget.receipts_parse",
                        format!("line {n} is not JSON: {e}"),
                        rel.clone(),
                        &format!("repair or drop line {n} of {rel}"),
                    ));
                    continue;
                }
            };
            let Some(obj) = parsed.as_object() else {
                out.push(error(
                    "budget.receipts_parse",
                    format!("line {n} is JSON but not an object"),
                    rel.clone(),
                    &format!("a ledger line must be one JSON object; repair line {n}"),
                ));
                continue;
            };
            let missing: Vec<&str> = ["ts", "kind"]
                .into_iter()
                .filter(|k| obj.get(*k).map(|v| v.is_null()).unwrap_or(true))
                .collect();
            if !missing.is_empty() {
                out.push(error(
                    "budget.receipts_parse",
                    format!("line {n} is missing {}", missing.join(", ")),
                    rel.clone(),
                    &format!("add {} to line {n} of {rel}", missing.join(", ")),
                ));
            }
        }
    }
    out
}

/// Free space in GB at `path`, or `None` if it cannot be measured.
///
/// **Measured, never assumed** — the Python says the same, and the reason is that a disk check which
/// guesses is worse than none: it would report a comfortable number while the disk filled.
///
/// Uses `df -Pk`, which is POSIX-specified output (`-P` guarantees one record per line in a fixed
/// column order), rather than a platform-specific syscall. **The alternative would be `libc::statvfs`
/// through an FFI bind, and the harness has no `libc` dependency** — adding one for a read-only disk
/// query is more moving parts than shelling out to a tool that is on every POSIX system.
pub fn disk_free_gb(path: &std::path::Path) -> Option<f64> {
    let out = std::process::Command::new("df")
        .arg("-Pk")
        .arg(path)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    // Skip the header, take the first data line, and read the 4th column (available KB).
    let line = text.lines().nth(1)?;
    let cols: Vec<&str> = line.split_whitespace().collect();
    let kb: f64 = cols.get(3)?.parse().ok()?;
    Some(kb / 1024.0 / 1024.0)
}

/// `budget.disk_floor` — free disk stays inside the owner's 30–70 GB working band.
///
/// **Below 30 GB is a stop condition in the owner's rules**, which is why this rule exists: a stop
/// condition that lives only in prose is one nobody notices until the disk fills.
///
/// A floor that **cannot be measured** is also a warning, not a pass — **an unmeasurable disk could be
/// any size**, and reporting nothing would read as "fine".
pub fn disk_floor(paths: &StatePaths) -> Vec<Finding> {
    let Some(free) = disk_free_gb(&paths.repo) else {
        return vec![warn(
            "budget.disk_floor",
            "cannot measure free disk".into(),
            paths.repo.display().to_string(),
            "check the mount",
        )];
    };
    let (low, high) = DISK_BAND_GB;
    if free >= low {
        return Vec::new();
    }
    vec![warn(
        "budget.disk_floor",
        format!(
            "free disk is {free:.2} GB, below the documented {low}-{high} GB working band \
             (measured on {})",
            paths.repo.display()
        ),
        paths.repo.display().to_string(),
        "run: scripts/storage/uor-hygiene --apply   (removes merged worktrees, idle build caches and \
         folders already in iCloud); below 30 GB is a stop condition, not a note",
    )]
}

/// Every `budget` rule ported in this unit.
pub fn all(paths: &StatePaths) -> Vec<Finding> {
    let mut out = receipts_parse(paths);
    out.extend(disk_floor(paths));
    // **`no_new_spending_class` was written, tested and NEVER CALLED.** It is the rule that prints the
    // owner's caps, and the dispatcher reported it as an `ok` record because nothing produced a
    // finding - **a rule that looks like a pass because it produces no result at all**, which is the
    // failure mode this port exists to catch. Caught by comparing the dispatcher's output against
    // Python's, not by reading this function.
    out.extend(no_new_spending_class());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StatePaths;

    fn fixture() -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let l = dir.path().join("docs/harness/ledger");
        std::fs::create_dir_all(&l).expect("mkdir");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        (dir, p)
    }

    fn write_ledger(d: &std::path::Path, body: &str) {
        std::fs::write(d.join("docs/harness/ledger/kimi-2026-10.jsonl"), body).expect("write");
    }

    /// **The three failures are different and must stay distinguishable.** A line that will not parse,
    /// a line that parses to a non-object, and a line that parses to an object missing its keys need
    /// three different fixes.
    #[test]
    fn the_three_failure_kinds_are_reported_differently() {
        let (d, p) = fixture();
        write_ledger(
            d.path(),
            "{\"ts\":\"2026-10-13T00:00:00Z\",\"kind\":\"skill\"}\n\
             not json\n\
             [1,2,3]\n\
             {\"kind\":\"skill\"}\n",
        );
        let f = receipts_parse(&p);
        assert_eq!(f.len(), 3, "three bad lines, three findings: {f:?}");
        assert!(
            f.iter().any(|x| x.message.contains("line 2 is not JSON")),
            "{f:?}"
        );
        assert!(
            f.iter()
                .any(|x| x.message.contains("line 3 is JSON but not an object")),
            "{f:?}"
        );
        assert!(
            f.iter().any(|x| x.message.contains("line 4 is missing ts")),
            "{f:?}"
        );
        assert!(
            f.iter().all(|x| x.severity == "error"),
            "a corrupt ledger line is an error"
        );
    }

    /// **The line number is in every message.** "The ledger has a bad line" is not actionable.
    #[test]
    fn every_message_names_the_line_number() {
        let (d, p) = fixture();
        write_ledger(d.path(), "ok\n");
        let f = receipts_parse(&p);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("line 1"), "{}", f[0].message);
        assert!(f[0].fix.contains("line 1"), "{}", f[0].fix);
    }

    #[test]
    fn a_clean_ledger_produces_nothing() {
        let (d, p) = fixture();
        write_ledger(
            d.path(),
            "{\"ts\":\"2026-10-13T00:00:00Z\",\"kind\":\"skill\"}\n\
             \n\
             {\"ts\":\"2026-10-13T00:01:00Z\",\"kind\":\"heartbeat\",\"lab\":\"kimi\"}\n",
        );
        assert!(
            receipts_parse(&p).is_empty(),
            "a blank line is not a bad line"
        );
    }

    /// An object whose keys are present but null is missing them: `{"ts": null}` is not a timestamp.
    #[test]
    fn a_null_key_counts_as_missing() {
        let (d, p) = fixture();
        write_ledger(d.path(), "{\"ts\":null,\"kind\":\"skill\"}\n");
        let f = receipts_parse(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("missing ts"), "{}", f[0].message);
    }

    /// A ledger directory that does not exist is not a finding — a fresh checkout has no ledger.
    #[test]
    fn a_missing_ledger_directory_produces_nothing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        assert!(receipts_parse(&p).is_empty());
    }

    /// **The measurement is real.** The repo is on a filesystem, so a number comes back.
    #[test]
    fn disk_free_reports_a_real_number_for_the_repo() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let gb = disk_free_gb(&repo).expect("df must answer for a real directory");
        assert!(gb > 0.0 && gb < 100_000.0, "a plausible number, got {gb}");
    }

    #[test]
    fn disk_free_returns_none_for_a_path_that_does_not_exist() {
        assert!(disk_free_gb(std::path::Path::new("/definitely/not/here/at/all")).is_none());
    }

    /// The real repo's ledger must parse — if it does not, Python would have said so.
    #[test]
    fn the_real_ledger_parses() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let f = receipts_parse(&p);
        assert!(f.is_empty(), "the real ledger has a bad line: {f:?}");
    }

    /// The band's low end is the stop condition, so the constant is asserted rather than assumed.
    #[test]
    fn the_working_band_matches_the_owner_rule() {
        assert_eq!(
            DISK_BAND_GB,
            (30.0, 70.0),
            "30 GB low end is the owner's stop condition"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// `budget.no_new_spending_class` - the caps, printed on every run.
// ---------------------------------------------------------------------------------------------

/// The owner's pod cap: **at most four running pods, counted across both providers**.
pub const POD_CAP: u32 = 4;

/// The owner's rate cap: **at most $8/h across all labs and sessions.**
pub const RATE_CAP_USD_PER_H: f64 = 8.00;

/// `budget.no_new_spending_class` — **a rule that always fires, and that is its job.**
///
/// `_check_caps` returns one `RuleResult` unconditionally, with `ok=False` and severity `info`:
///
/// ```python
/// def _check_caps(paths, reg):
///     return [RuleResult(rule_id="budget.no_new_spending_class", severity="info", ok=False, ...)]
/// ```
///
/// **So it does not test anything — it PRINTS the caps on every `check`, and the `ok=False` is what
/// makes them visible.** The rule's description says so: *"the owner caps are printed so a session
/// cannot silently exceed them"*.
///
/// **Ported with `ok=False` faithful to that**, rather than as a rule that "passes when within caps":
/// there is no spend reading here to compare against, and **inventing a comparison would be inventing
/// a check the engine does not have** — the same reasoning as `receipts.skill_usage_recorded`, in the
/// opposite direction: that rule never fires, this one always does.
///
/// **A reader who sees `ok=False` and severity `info` together should read it as "reported, not
/// failed"** — which is exactly what the pair means, and why changing the severity would change the
/// run's summary counts for every lab.
pub fn no_new_spending_class() -> Vec<Finding> {
    vec![Finding {
        rule_id: "budget.no_new_spending_class",
        severity: "info",
        message: format!(
            "owner caps in force: <= {POD_CAP} running pods and <= ${RATE_CAP_USD_PER_H:.2}/h \
             across all labs and sessions, all external compute through scripts/pod/uor-pod; \
             a new spending class needs an owner decision"
        ),
        path: "AGENTS.md".into(),
        fix: "if a run would exceed a cap or need a new spending class, stop that line and ask the owner on the milestone".into(),
    }]
}

#[cfg(test)]
mod spending_tests {
    /// **The rule always fires and never reports an error.** Asserted so a future change that made it
    /// conditional - reporting only when a cap was exceeded, say - would be **visible as a behaviour
    /// change rather than passing silently with a different meaning.**
    #[test]
    fn the_caps_rule_always_reports_and_is_info_not_error() {
        let f = super::no_new_spending_class();
        assert_eq!(f.len(), 1, "it fires unconditionally");
        assert_eq!(f[0].rule_id, "budget.no_new_spending_class");
        assert_eq!(
            f[0].severity, "info",
            "info, so it does not count as a warning"
        );
        assert_eq!(
            f[0].path, "AGENTS.md",
            "the caps live in AGENTS.md, and the path says so"
        );
    }

    /// **The message carries both caps, formatted as Python formats them** - `$8.00/h` with two
    /// decimals, because the value is a float and Python's `:.2f` is what a reader compares against.
    #[test]
    fn the_message_names_both_caps_with_pythons_formatting() {
        let m = &super::no_new_spending_class()[0].message;
        assert!(m.contains("<= 4 running pods"), "{m}");
        assert!(
            m.contains("<= $8.00/h"),
            "Python's `:.2f` gives two decimals: {m}"
        );
        assert!(m.contains("all labs and sessions"), "{m}");
        assert!(
            m.contains("a new spending class needs an owner decision"),
            "{m}"
        );
    }
}
