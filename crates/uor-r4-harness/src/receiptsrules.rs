//! The `receipts` rules, as Rust (D30 check-port step 9 of 10).
//!
//! # The three `receipts` rules, and which are here
//!
//! | rule | severity | in this unit |
//! |---|---|---|
//! | `receipts.parse` | error | **yes** |
//! | `receipts.ledger_present` | warn | **yes** |
//! | `receipts.skill_usage_recorded` | info | **yes** (explicit no-op matching Python) |
//!
//! ## Why `skill_usage_recorded` is an explicit no-op
//!
//! **Its Python body always returns an empty list.** It reads the skill receipts and then returns
//! `[]` on both branches — the comment says so: *"informational only: the engine prints `ok` for an
//! empty list."* Ported in [`skill_usage_recorded`] and wired into [`all`] so [`crate::checkdispatch`]
//! invokes every rule registered in `checkdispatch::RULES`, while preserving Python's exact no-op
//! behaviour rather than inventing a failure condition Python does not have.
//!
//! ## `receipts.parse` and `budget.receipts_parse` are two rules over one file
//!
//! They read the same ledger and **report different things** — verified by reading both:
//!
//! | | `receipts.parse` | `budget.receipts_parse` |
//! |---|---|---|
//! | not JSON | error | error |
//! | missing `ts`/`kind` | **"has no ts/kind"** | **"is missing ts" / "is missing kind"**, separately |
//! | non-object JSON | *not checked* | **error** |
//! | unknown `kind` | **error, against the known set** | *not checked* |
//!
//! So they are **complementary, not duplicates**, and one bad line can produce **two findings from
//! two rules** — which is correct, because the two rules are answering different questions. Porting
//! only one would lose a check; merging them would change what `check` reports.

use std::path::PathBuf;

use crate::state::StatePaths;

pub use crate::preprules::Finding;

/// Receipt kinds the ledger may carry.
///
/// **A `kind` outside this set means the ledger holds something nothing reads.** That is an error
/// rather than a warning because the ledger is the record of what happened: an unrecognised kind is
/// either a typo — so the record is filed under a heading no reader looks at — or a new kind added
/// without a reader, which is the same problem with better intentions.
///
/// Kept in step with the Python set by hand; the two are compared by the port's agreement probe.
pub const KINDS: [&str; 10] = [
    "check",
    "claim",
    "context",
    "heartbeat",
    "pr",
    "prep",
    "receipt",
    "release",
    "skill",
    "team",
];

fn error(rule_id: &'static str, message: String, path: String, fix: String) -> Finding {
    Finding {
        rule_id,
        severity: "error",
        message,
        path,
        fix,
    }
}

fn warn(rule_id: &'static str, message: String, path: String, fix: String) -> Finding {
    Finding {
        rule_id,
        severity: "warn",
        message,
        path,
        fix,
    }
}

/// Every `.jsonl` in the ledger directory, sorted.
pub fn ledger_files(paths: &StatePaths) -> Vec<PathBuf> {
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
    files
}

/// `receipts.parse` — every ledger line is valid JSON with a `ts` and a **known** `kind`.
///
/// Three failures: not JSON, no `ts`/`kind`, and **an unknown `kind`**. The third is the one that
/// distinguishes this rule from `budget.receipts_parse`, which does not check kinds at all.
pub fn parse(paths: &StatePaths) -> Vec<Finding> {
    let mut out = Vec::new();
    for f in ledger_files(paths) {
        let name = f
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let rel = f.display().to_string();
        for (i, line) in text.lines().enumerate() {
            let n = i + 1;
            if line.trim().is_empty() {
                continue;
            }
            let rec: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(e) => {
                    out.push(error(
                        "receipts.parse",
                        format!("{name}:{n} is not valid JSON ({e})"),
                        rel.clone(),
                        format!("fix or delete the line: {rel}:{n}"),
                    ));
                    continue;
                }
            };
            // A non-object line cannot carry `ts`/`kind`, but it is reported as missing those rather
            // than as a type error, matching the Python's shape of message.
            let obj = rec.as_object();
            let has = |k: &str| obj.map(|o| o.contains_key(k)).unwrap_or(false);
            if !has("ts") || !has("kind") {
                out.push(error(
                    "receipts.parse",
                    format!("{name}:{n} has no ts/kind"),
                    rel.clone(),
                    "a receipt needs at least ts and kind".into(),
                ));
                continue;
            }
            let kind = obj
                .and_then(|o| o.get("kind"))
                .and_then(|k| k.as_str())
                .unwrap_or("");
            if !KINDS.contains(&kind) {
                out.push(error(
                    "receipts.parse",
                    format!("{name}:{n} has unknown kind {kind:?}"),
                    rel.clone(),
                    format!("use one of {KINDS:?}"),
                ));
            }
        }
    }
    out
}

/// `receipts.ledger_present` — the ledger directory exists and holds receipts.
///
/// **A directory that exists and holds nothing reads as a working ledger.** This is the warn that
/// says otherwise, and it is deliberately narrow: **a ledger directory that does not exist at all
/// returns no finding**, because a fresh checkout has none and reporting that would fire on every
/// clone before any work happened.
pub fn ledger_present(paths: &StatePaths) -> Vec<Finding> {
    if !ledger_files(paths).is_empty() {
        return Vec::new();
    }
    let dir = paths.repo.join("docs").join("harness").join("ledger");
    if !dir.exists() {
        return Vec::new();
    }
    vec![warn(
        "receipts.ledger_present",
        "docs/harness/ledger/ exists but holds no receipts".into(),
        "docs/harness/ledger".into(),
        "the directory is tracked, so an empty one means receipts stopped being recorded; check that \
         commands are being run with a lab identity"
            .into(),
    )]
}

/// Every `receipts` rule ported in this unit.
pub fn all(paths: &StatePaths) -> Vec<Finding> {
    let mut out = parse(paths);
    out.extend(ledger_present(paths));
    out.extend(skill_usage_recorded());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StatePaths;

    fn fixture() -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("docs/harness/ledger")).expect("mkdir");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        (dir, p)
    }

    fn write(d: &std::path::Path, body: &str) {
        std::fs::write(d.join("docs/harness/ledger/kimi-2026-10.jsonl"), body).expect("write");
    }

    /// **The rule that distinguishes this from `budget.receipts_parse`: an unknown kind.**
    #[test]
    fn an_unknown_kind_is_an_error_naming_it_and_the_known_set() {
        let (d, p) = fixture();
        write(
            d.path(),
            "{\"ts\":\"2026-10-13T00:00:00Z\",\"kind\":\"invented\"}\n",
        );
        let f = parse(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("unknown kind"), "{}", f[0].message);
        assert!(f[0].message.contains("invented"), "{}", f[0].message);
        assert!(
            f[0].fix.contains("heartbeat"),
            "the fix lists the known set: {}",
            f[0].fix
        );
    }

    #[test]
    fn every_declared_kind_is_accepted() {
        let (d, p) = fixture();
        let body: String = KINDS
            .iter()
            .map(|k| format!("{{\"ts\":\"2026-10-13T00:00:00Z\",\"kind\":\"{k}\"}}\n"))
            .collect();
        write(d.path(), &body);
        assert!(parse(&p).is_empty(), "{:?}", parse(&p));
    }

    /// A missing `ts` or `kind` is one finding here, not two - the Python reports them together.
    #[test]
    fn a_missing_key_is_one_finding() {
        let (d, p) = fixture();
        write(d.path(), "{\"kind\":\"skill\"}\n");
        let f = parse(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("has no ts/kind"), "{}", f[0].message);
    }

    #[test]
    fn a_line_that_is_not_json_names_the_file_and_line() {
        let (d, p) = fixture();
        write(d.path(), "not json\n");
        let f = parse(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message.contains("kimi-2026-10.jsonl:1"),
            "{}",
            f[0].message
        );
    }

    /// A blank line is not a bad line.
    #[test]
    fn blank_lines_are_skipped() {
        let (d, p) = fixture();
        write(
            d.path(),
            "\n{\"ts\":\"2026-10-13T00:00:00Z\",\"kind\":\"skill\"}\n\n",
        );
        assert!(parse(&p).is_empty());
    }

    /// **A ledger directory that does not exist returns nothing** - a fresh checkout has none, and
    /// reporting it would fire on every clone before any work happened.
    #[test]
    fn a_missing_ledger_directory_is_not_reported() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        assert!(ledger_present(&p).is_empty());
        assert!(parse(&p).is_empty());
    }

    /// **A directory that exists and holds nothing reads as a working ledger.**
    #[test]
    fn an_empty_but_present_ledger_directory_is_reported() {
        let (_d, p) = fixture();
        let f = ledger_present(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message.contains("holds no receipts"),
            "{}",
            f[0].message
        );
    }

    #[test]
    fn a_ledger_with_a_receipt_is_not_reported() {
        let (d, p) = fixture();
        write(
            d.path(),
            "{\"ts\":\"2026-10-13T00:00:00Z\",\"kind\":\"skill\"}\n",
        );
        assert!(ledger_present(&p).is_empty());
    }

    /// The real ledger must pass both — if it did not, Python would have said so.
    #[test]
    fn the_real_ledger_passes_both_rules() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let f = all(&p);
        assert!(f.is_empty(), "the real ledger fails a receipts rule: {f:?}");
    }

    /// The kind set is asserted, so a drift from Python's is a failing test rather than a silent gap.
    #[test]
    fn the_kind_set_is_the_expected_one() {
        assert_eq!(KINDS.len(), 10);
        for k in [
            "skill",
            "heartbeat",
            "claim",
            "release",
            "team",
            "prep",
            "receipt",
        ] {
            assert!(KINDS.contains(&k), "{k} must be a known kind");
        }
    }
}

#[cfg(test)]
thread_local! {
    static SKILL_USAGE_RECORDED_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// `receipts.skill_usage_recorded` — **a rule that cannot fail, ported as one that cannot fail.**
///
/// Python:
///
/// ```python
/// recs = read(paths, kinds=("skill",))
/// if recs:
///     return []
/// return []  # informational only: the engine prints "ok" for an empty list
/// ```
///
/// **Both branches return empty, so no input produces a finding.** The rule is declared, registered
/// and reported as passing, and it asserts nothing.
///
/// **Ported rather than skipped, and ported as an explicit empty list rather than as a plausible
/// check.** Writing it as "report when skills were used but no skill receipt exists" would be
/// **inventing a rule the engine does not have** — and it would fire on every repository state where
/// the two disagree, which is a behaviour change dressed as a port.
///
/// **The reason it is worth a function is the reason it is worth a comment**: a reader who finds
/// `receipts.skill_usage_recorded` in `check`'s output should know it is a placeholder passing
/// silently, not a verification that succeeded.
pub fn skill_usage_recorded() -> Vec<crate::preprules::Finding> {
    #[cfg(test)]
    SKILL_USAGE_RECORDED_CALLS.with(|c| c.set(c.get() + 1));
    Vec::new()
}

#[cfg(test)]
mod skill_usage_tests {
    /// **The rule cannot fail, and that is asserted rather than assumed.** If a future change makes
    /// it produce a finding, this test fails and the change is visible as a behaviour change -
    /// **which is exactly what a silently-passing check would otherwise hide.**
    #[test]
    fn the_rule_produces_nothing_for_any_state() {
        assert!(super::skill_usage_recorded().is_empty());
    }

    #[test]
    fn all_and_checkdispatch_invoke_skill_usage_recorded() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = crate::state::StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        let regs = crate::registries::Registries::default();
        super::SKILL_USAGE_RECORDED_CALLS.with(|c| c.set(0));
        let _ = super::all(&p);
        assert_eq!(
            super::SKILL_USAGE_RECORDED_CALLS.with(|c| c.get()),
            1,
            "receiptsrules::all must invoke skill_usage_recorded()"
        );
        let out = crate::checkdispatch::run_scoped(
            &p,
            &regs,
            &["receipts.skill_usage_recorded".to_string()],
            crate::checkdispatch::Scope::Repo,
            0,
        );
        assert_eq!(
            super::SKILL_USAGE_RECORDED_CALLS.with(|c| c.get()),
            2,
            "checkdispatch::run_scoped must invoke receiptsrules::all -> skill_usage_recorded()"
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].rule_id, "receipts.skill_usage_recorded");
        assert_eq!(out[0].message, "ok");
    }
}
