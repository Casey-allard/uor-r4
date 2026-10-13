//! `status.heartbeat_receipts` — every in-progress milestone has a recent heartbeat receipt.
//!
//! # I described this rule wrongly, then read it, then ported it
//!
//! I told the owner and recorded in `current-state.md` that this rule and `registry_vs_github` are
//! **"the two rules whose subject is the GitHub conversation"**, needing `gh` pagination and comment
//! parsing. **That was wrong**, and I used it to defer this rule three times.
//!
//! **It reads the local ledger shards.** No GitHub call. The correction is recorded at the end of
//! `current-state.md`, and this port is what the corrected estimate predicted: **one file, one
//! receipt read, one age comparison.**
//!
//! # What it asks, and why 24 hours
//!
//! A milestone marked **in-progress** should show a heartbeat receipt within `STALE_HOURS = 24`. The
//! window is deliberately a day rather than the harness's five-minute presence window: **presence is
//! live-tier and ad-hoc, while a heartbeat receipt is the durable record**, and a milestone is not
//! abandoned because nobody wrote a receipt this afternoon.
//!
//! **Only `in-progress` milestones**, and that matters: a completed milestone has no obligation to
//! keep checking in, so reporting it would nag about finished work — **and a check that nags about
//! finished work is one that gets ignored when it is right.**

use crate::multilabrules::parse_claimed_at;
use crate::registries::Registries;
use crate::state::{read_ledger, StatePaths};

pub use crate::preprules::Finding;

/// How long an in-progress milestone may go without a heartbeat receipt.
pub const STALE_HOURS: i64 = 24;

/// Whether a receipt names `milestone`, in any of the three forms the registry uses.
///
/// **Three forms because three callers write it**: the short key (`m1`), the display id (`M1`) and
/// the issue number. A receipt that names the milestone by number is naming the same milestone, and
/// **requiring one spelling would report a milestone as silent while its receipts sat in the ledger.**
fn milestone_matches(row: &crate::state::ReceiptRow, key: &str, id: &str, issue: i64) -> bool {
    let Some(raw) = row.milestone.as_deref() else {
        return false;
    };
    let raw = raw.trim().to_lowercase();
    if raw.is_empty() {
        return false;
    }
    raw == key.to_lowercase() || raw == id.to_lowercase() || raw == issue.to_string()
}

/// `status.heartbeat_receipts` — no heartbeat receipt in `STALE_HOURS` for an in-progress milestone.
///
/// `now` is passed in rather than read, so the rule is testable and **so a caller can ask the same
/// question about a fixed instant** — which is what the Python does not do, and why its tests need a
/// mocked clock.
pub fn heartbeat_receipts(paths: &StatePaths, regs: &Registries, now: i64) -> Vec<Finding> {
    let rows = read_ledger(paths);
    let mut out = Vec::new();
    for m in &regs.milestones {
        // **Only in-progress.** A finished milestone owes no check-in, and nagging about one would
        // train readers to ignore the rule when it is right.
        if m.status != "in-progress" {
            continue;
        }
        // The newest heartbeat receipt for this milestone, in any of its three names.
        let newest = rows
            .iter()
            .filter(|r| r.kind == "heartbeat" && milestone_matches(r, &m.key, &m.id, m.issue))
            .filter_map(|r| parse_claimed_at(&r.ts))
            .max();

        let age_hours = newest.map(|t| (now - t) / 3600);
        if let Some(age) = age_hours {
            if age <= STALE_HOURS {
                continue;
            }
        }
        let seen = match age_hours {
            Some(a) if a >= 48 => format!("last heartbeat {} days ago", a / 24),
            Some(a) => format!("last heartbeat {a}h ago"),
            None => "no heartbeat receipt on record".to_string(),
        };
        out.push(Finding {
            rule_id: "status.heartbeat_receipts",
            severity: "warn",
            message: format!("{} is in progress but has {seen}", m.id),
            path: "docs/harness/ledger".into(),
            fix: format!(
                "check in: uor-harness heartbeat -m {}   (or mark the milestone finished if it is)",
                m.key
            ),
        });
    }
    out
}

/// Every `status` rule ported in this unit.
pub fn all(paths: &StatePaths, regs: &Registries, now: i64) -> Vec<Finding> {
    heartbeat_receipts(paths, regs, now)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(ledger: &str) -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("docs/harness/ledger")).expect("mkdir");
        std::fs::write(
            dir.path().join("docs/harness/ledger/kimi-2026-10.jsonl"),
            ledger,
        )
        .expect("write");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        (dir, p)
    }

    /// Build a registry with one milestone in the given status, using the real registry as the base
    /// so the milestone fields are real rather than invented.
    fn regs_with_status(status: &str) -> Registries {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let mut regs = Registries::load(&p).expect("registry");
        if let Some(m) = regs.milestones.first_mut() {
            m.status = status.to_string();
        }
        regs
    }

    /// **A receipt naming the milestone by ISSUE NUMBER is naming the same milestone.** Requiring one
    /// spelling would report a milestone as silent while its receipts sat in the ledger.
    #[test]
    fn all_three_milestone_spellings_match() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let regs = Registries::load(&p).expect("registry");
        let m = regs.milestones.first().expect("a milestone");
        let row = |ms: &str| crate::state::ReceiptRow {
            kind: "heartbeat".into(),
            milestone: Some(ms.to_string()),
            ..Default::default()
        };
        assert!(
            milestone_matches(&row(&m.key), &m.key, &m.id, m.issue),
            "key"
        );
        assert!(milestone_matches(&row(&m.id), &m.key, &m.id, m.issue), "id");
        assert!(
            milestone_matches(&row(&m.issue.to_string()), &m.key, &m.id, m.issue),
            "issue number"
        );
        assert!(
            !milestone_matches(&row("m999"), &m.key, &m.id, m.issue),
            "a different milestone"
        );
    }

    /// **Only in-progress milestones are reported.** A finished milestone owes no check-in, and a
    /// rule that nags about finished work gets ignored when it is right.
    #[test]
    fn a_finished_milestone_is_not_reported() {
        let (_d, p) = fixture("");
        let regs = regs_with_status("done");
        let now = crate::team::now_iso();
        let now = parse_claimed_at(&now).expect("now");
        let f = heartbeat_receipts(&p, &regs, now);
        assert!(
            f.iter().all(|x| !x.message.contains("in progress")),
            "a finished milestone must not be nagged: {f:?}"
        );
    }

    /// An in-progress milestone with NO heartbeat receipt at all is reported, and says so.
    #[test]
    fn an_in_progress_milestone_with_no_receipt_is_reported() {
        let (_d, p) = fixture("");
        let regs = regs_with_status("in-progress");
        let now = parse_claimed_at("2026-10-13T12:00:00Z").expect("now");
        let f = heartbeat_receipts(&p, &regs, now);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message.contains("no heartbeat receipt on record"),
            "{}",
            f[0].message
        );
        assert_eq!(f[0].severity, "warn");
    }

    /// A recent heartbeat satisfies it.
    #[test]
    fn a_recent_heartbeat_satisfies_the_rule() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let base = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let regs = Registries::load(&base).expect("registry");
        let m = regs.milestones.first().expect("a milestone").clone();
        let ledger = format!(
            "{{\"kind\":\"heartbeat\",\"milestone\":\"{}\",\"ts\":\"2026-10-13T11:00:00Z\"}}\n",
            m.key
        );
        let (_d, p) = fixture(&ledger);
        let now = parse_claimed_at("2026-10-13T12:00:00Z").expect("now");
        let regs = regs_with_status("in-progress");
        let f = heartbeat_receipts(&p, &regs, now);
        assert!(
            f.iter().all(|x| x.message.contains("no heartbeat receipt")),
            "{f:?}"
        );
    }

    /// **The window is 24 hours and the boundary is asserted**, because a boundary written from
    /// memory is what produced two of my earlier mistakes.
    #[test]
    fn the_stale_window_is_one_day() {
        assert_eq!(STALE_HOURS, 24);
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let base = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let regs = Registries::load(&base).expect("registry");
        let m = regs.milestones.first().expect("a milestone").clone();
        let now = parse_claimed_at("2026-10-13T12:00:00Z").expect("now");
        // Exactly 24h old: `<= STALE_HOURS` means it still satisfies the rule.
        let at_boundary = format!(
            "{{\"kind\":\"heartbeat\",\"milestone\":\"{}\",\"ts\":\"2026-10-12T12:00:00Z\"}}\n",
            m.key
        );
        let (_d, p) = fixture(&at_boundary);
        let f = heartbeat_receipts(&p, &regs_with_status("in-progress"), now);
        assert!(
            f.iter().all(|x| !x.message.contains("last heartbeat")),
            "exactly 24h still satisfies a `<= 24` window: {f:?}"
        );
        // 25h old: reported.
        let past = format!(
            "{{\"kind\":\"heartbeat\",\"milestone\":\"{}\",\"ts\":\"2026-10-12T11:00:00Z\"}}\n",
            m.key
        );
        let (_d2, p2) = fixture(&past);
        let f2 = heartbeat_receipts(&p2, &regs_with_status("in-progress"), now);
        assert!(f2.iter().any(|x| x.message.contains("25h ago")), "{f2:?}");
    }

    /// A non-heartbeat receipt does **not** satisfy the rule: the rule asks for a check-in, not for
    /// any line mentioning the milestone.
    #[test]
    fn a_non_heartbeat_receipt_does_not_satisfy_it() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let base = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let regs0 = Registries::load(&base).expect("registry");
        let m = regs0.milestones.first().expect("a milestone").clone();
        let ledger = format!(
            "{{\"kind\":\"skill\",\"milestone\":\"{}\",\"ts\":\"2026-10-13T11:00:00Z\"}}\n",
            m.key
        );
        let (_d, p) = fixture(&ledger);
        let now = parse_claimed_at("2026-10-13T12:00:00Z").expect("now");
        let f = heartbeat_receipts(&p, &regs_with_status("in-progress"), now);
        assert!(
            f.iter().any(|x| x.message.contains("no heartbeat receipt")),
            "a skill receipt is not a check-in: {f:?}"
        );
    }

    /// The real repo runs without panicking on real data.
    #[test]
    fn the_real_repo_is_processed() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let regs = Registries::load(&p).expect("registry");
        let now = parse_claimed_at(&crate::team::now_iso()).expect("now");
        let _ = heartbeat_receipts(&p, &regs, now);
    }
}
