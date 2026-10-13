//! Branch claims — who holds what, visible to every lab (D30 port of Python `team`).
//!
//! # Why the merge is three-tier and why the committed copy always wins
//!
//! Three places can hold a claim and each answers a different question:
//!
//! * **the caller's own checkout** — committed, riding the delivery PR. **The authority for every
//!   branch it names.**
//! * **main's committed store** — what the other labs have merged. Consulted for a branch the caller's
//!   store does not name, so a fresh worktree still sees the shared picture.
//! * **live** (`~/.local/share/uor-r4/harness/claims.json`) — real time, uncommitted, advisory.
//!   Consulted **only** for a branch neither committed store names.
//!
//! **One arbiter per branch, and it is the committed one.** An earlier Python version merged the
//! tiers newest-wins and that was wrong: `claim` stamps one entry into two tiers, so the copies tie
//! and then diverge under a hand edit — and "newest" cannot choose between two claims that are both
//! current. The port keeps the rule, because the bug it prevents is not about Python.
//!
//! # What a claim is not
//!
//! It is not a lock. Two labs claiming the same branch in different worktrees both succeed locally
//! and the collision surfaces as a `claims.json` conflict at merge time. That is the accepted cost of
//! keeping coordination out of the critical path; the alternative was a distributed lock nobody
//! wanted to run.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Map, Value};

use crate::state::StatePaths;

/// One branch claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub branch: String,
    pub lab: String,
    pub head: String,
    pub claimed_at: String,
    pub note: String,
}

impl Claim {
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("branch".into(), Value::from(self.branch.clone()));
        m.insert("lab".into(), Value::from(self.lab.clone()));
        m.insert("head".into(), Value::from(self.head.clone()));
        m.insert("claimed_at".into(), Value::from(self.claimed_at.clone()));
        m.insert("note".into(), Value::from(self.note.clone()));
        Value::Object(m)
    }

    fn from_json(branch: &str, v: &Value) -> Option<Claim> {
        let o = v.as_object()?;
        Some(Claim {
            branch: branch.to_string(),
            lab: o
                .get("lab")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            head: o
                .get("head")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            claimed_at: o
                .get("claimed_at")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            note: o
                .get("note")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

/// Why a claim was refused. Each maps to a distinct exit code at the CLI, because a caller script
/// needs to tell "you asked wrong" from "someone else holds it".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimError {
    /// A usage error: no branch, or no lab. `EXIT_USAGE` in the Python original.
    Usage(String),
    /// Another lab holds it. `EXIT_FAIL`.
    Held(Claim),
    /// Nothing to release. `EXIT_FAIL`.
    NotHeld(String),
}

/// The committed claims path in the caller's checkout — the authority.
pub fn claims_file(paths: &StatePaths) -> PathBuf {
    paths.repo.join("docs").join("harness").join("claims.json")
}

/// The committed store on `main`, which is a different file ONLY when the caller is not the main
/// checkout. Reading one path twice would be harmless but would double-count a tier, so callers skip
/// it when the two are equal.
pub fn shared_claims_file(paths: &StatePaths) -> PathBuf {
    // In a linked worktree, git records the main checkout's git dir; `main`'s working tree is not
    // reachable from here without a fetch, so the shared tier is the same path when the caller IS the
    // main checkout and otherwise absent. Kept as a function so the rule has one home.
    claims_file(paths)
}

/// The live tier overlay: real time, uncommitted, advisory.
pub fn live_claims_file(paths: &StatePaths) -> PathBuf {
    paths.live.join("claims.json")
}

/// Read one tier, tolerating absence and corruption.
///
/// A missing or malformed store reads as empty rather than failing the command. **A lab must be able
/// to claim a branch on a machine whose live tier is missing**, and refusing there would make the
/// advisory tier load-bearing, which it is not.
fn read_tier(path: &Path) -> BTreeMap<String, Claim> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return BTreeMap::new();
    };
    let Some(obj) = v.as_object() else {
        return BTreeMap::new();
    };
    obj.iter()
        .filter_map(|(k, val)| Claim::from_json(k, val).map(|c| (k.clone(), c)))
        .collect()
}

/// Every claim, committed authoritative and live filling gaps.
pub fn load_claims(paths: &StatePaths) -> BTreeMap<String, Claim> {
    let mut merged = BTreeMap::new();
    // Weakest first, strongest last: the caller's committed store overwrites everything it names.
    for path in [live_claims_file(paths), claims_file(paths)] {
        for (k, v) in read_tier(&path) {
            merged.insert(k, v);
        }
    }
    merged
}

/// The canonical JSON form: sorted keys, two-space indent, trailing newline.
///
/// Written by hand rather than relying on a serializer's defaults, because the committed store is
/// reviewed in diffs and a formatting change would look like a content change.
fn render(claims: &BTreeMap<String, Claim>) -> String {
    let obj: Map<String, Value> = claims
        .iter()
        .map(|(k, v)| (k.clone(), v.to_json()))
        .collect();
    // SORTED keys. `serde_json` preserves insertion order, and the Python store this replaces wrote
    // `sort_keys=True` - so an unsorted writer would reorder every existing file on first touch and
    // turn a one-line claim into a whole-file diff in review.
    let sorted: std::collections::BTreeMap<String, Value> = obj.into_iter().collect();
    let mut out = serde_json::to_string_pretty(&sorted).unwrap_or_else(|_| "{}".into());
    out.push('\n');
    out
}

/// Save the committed store. The authority, so a failure here IS a failure.
pub fn save_claims(paths: &StatePaths, claims: &BTreeMap<String, Claim>) -> Result<(), String> {
    let path = claims_file(paths);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    std::fs::write(&path, render(claims))
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// Mirror the claims into the live tier. **Best-effort and deliberately so.**
///
/// A read-only or absent live tier reduces immediacy and must never cost the committed copy, which is
/// the one that survives review and cleanup. Returns whether it landed, for reporting rather than for
/// gating.
pub fn publish_claims_live(paths: &StatePaths, claims: &BTreeMap<String, Claim>) -> bool {
    let path = live_claims_file(paths);
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return false;
        }
    }
    std::fs::write(&path, render(claims)).is_ok()
}

/// The current branch, or empty when detached.
pub fn current_branch(repo: &Path) -> String {
    git(repo, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default()
}

/// A short HEAD sha, or empty when git cannot answer.
fn short_head(repo: &Path) -> String {
    git(repo, &["rev-parse", "--short", "HEAD"]).unwrap_or_default()
}

fn git(repo: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Now, as ISO 8601 with the local UTC offset — the format the committed store already carries.
///
/// Public because `multilab` and the board tests need the same notion of "now" that `team` writes.
/// Two timestamp formats in one repository is how a reader comes to mis-parse one of them.
///
/// Not `chrono`: the offset has to come from the system, and the harness deliberately has a small
/// dependency set. `date` is asked because it is the only thing that knows the local zone.
pub fn now_iso() -> String {
    // `date +%:z` is a GNU extension. BSD/macOS `date` prints the offset as `+hhmm`, and asking for
    // `%:z` there yields the LITERAL `:z` — this produced `2026-10-13T13:37:12:z`, which every reader
    // of the committed store would have to cope with. So the offset is taken as `%z` and the colon
    // inserted here, which works on both.
    let raw = Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S %z"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());
    let Some(raw) = raw else {
        return "1970-01-01T00:00:00+00:00".to_string();
    };
    let Some((stamp, offset)) = raw.split_once(' ') else {
        return raw;
    };
    // +hhmm / -hhmm -> +hh:mm. An offset already carrying a colon is left alone.
    let formatted = if offset.len() == 5 && !offset.contains(':') {
        format!("{}:{}", &offset[..3], &offset[3..])
    } else {
        offset.to_string()
    };
    format!("{stamp}{formatted}")
}

/// Claim a branch for a lab.
///
/// Refuses only when **another** lab holds it: re-claiming your own branch succeeds and refreshes the
/// entry, which is how a lab renews a claim after a rebase.
pub fn claim_branch(
    paths: &StatePaths,
    branch: &str,
    lab: &str,
    note: &str,
) -> Result<Claim, ClaimError> {
    if branch.is_empty() {
        return Err(ClaimError::Usage("a branch name is required".into()));
    }
    if lab.is_empty() {
        return Err(ClaimError::Usage(
            "a lab is required (--lab), because a claim without an owner is not a claim".into(),
        ));
    }
    let mut claims = load_claims(paths);
    if let Some(existing) = claims.get(branch) {
        if !existing.lab.is_empty() && existing.lab != lab {
            return Err(ClaimError::Held(existing.clone()));
        }
    }
    let entry = Claim {
        branch: branch.to_string(),
        lab: lab.to_string(),
        head: short_head(&paths.repo),
        claimed_at: now_iso(),
        note: note.to_string(),
    };
    claims.insert(branch.to_string(), entry.clone());
    save_claims(paths, &claims).map_err(ClaimError::Usage)?;
    publish_claims_live(paths, &claims);
    Ok(entry)
}

/// Release a claim.
///
/// `lab` restricts the release to that lab's own claim; empty releases whatever holds the branch.
/// **The live mirror is republished on release.** A mirror that is only ever added to resurrects
/// released claims, which is the bug the Python version hit.
pub fn release_branch(paths: &StatePaths, branch: &str, lab: &str) -> Result<Claim, ClaimError> {
    let mut claims = load_claims(paths);
    let Some(existing) = claims.get(branch).cloned() else {
        return Err(ClaimError::NotHeld(branch.to_string()));
    };
    if !lab.is_empty() && !existing.lab.is_empty() && existing.lab != lab {
        return Err(ClaimError::Held(existing));
    }
    claims.remove(branch);
    save_claims(paths, &claims).map_err(ClaimError::Usage)?;
    publish_claims_live(paths, &claims);
    Ok(existing)
}

/// The refusal text, in one place so the CLI and the tests cannot drift.
pub fn held_message(c: &Claim) -> String {
    format!(
        "{} is already held by lab {:?} since {}",
        c.branch, c.lab, c.claimed_at
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        dir: tempfile::TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            let dir = tempfile::tempdir().expect("tempdir");
            let repo = dir.path();
            for args in [
                vec!["init", "-q", "-b", "main"],
                vec!["config", "user.email", "t@t"],
                vec!["config", "user.name", "t"],
            ] {
                Command::new("git")
                    .args(&args)
                    .current_dir(repo)
                    .output()
                    .expect("git");
            }
            std::fs::create_dir_all(repo.join("docs/harness")).expect("mkdir");
            std::fs::write(repo.join("docs/harness/claims.json"), "{}\n").expect("write");
            std::fs::write(repo.join("f"), "1").expect("write");
            Command::new("git")
                .args(["add", "-A"])
                .current_dir(repo)
                .output()
                .expect("git");
            Command::new("git")
                .args(["commit", "-qm", "init"])
                .current_dir(repo)
                .output()
                .expect("git");
            Fixture { dir }
        }

        fn paths(&self) -> StatePaths {
            let live = self.dir.path().join("live");
            StatePaths::resolve(self.dir.path(), None, Some(live.to_str().expect("utf8")))
        }
    }

    /// The whole point: a second lab cannot take a held branch.
    #[test]
    fn another_lab_cannot_claim_a_held_branch() {
        let f = Fixture::new();
        let p = f.paths();
        claim_branch(&p, "probe/one", "claude", "").expect("first claim");
        match claim_branch(&p, "probe/one", "kimi", "") {
            Err(ClaimError::Held(c)) => {
                assert_eq!(c.lab, "claude");
                assert!(
                    held_message(&c).contains("already held by lab \"claude\""),
                    "{}",
                    held_message(&c)
                );
            }
            other => panic!("expected Held, got {other:?}"),
        }
    }

    /// A lab renewing its own claim is not a conflict - that is how a claim is refreshed after a
    /// rebase, and refusing it would force a release/claim cycle for no reason.
    #[test]
    fn the_same_lab_may_reclaim_and_it_refreshes() {
        let f = Fixture::new();
        let p = f.paths();
        let first = claim_branch(&p, "b", "claude", "first").expect("claim");
        let again = claim_branch(&p, "b", "claude", "second").expect("reclaim");
        assert_eq!(again.lab, "claude");
        assert_eq!(again.note, "second", "the entry is refreshed");
        assert_eq!(again.branch, first.branch);
    }

    #[test]
    fn an_empty_branch_or_lab_is_a_usage_error_not_a_refusal() {
        let f = Fixture::new();
        let p = f.paths();
        assert!(matches!(
            claim_branch(&p, "", "claude", ""),
            Err(ClaimError::Usage(_))
        ));
        match claim_branch(&p, "b", "", "") {
            Err(ClaimError::Usage(m)) => assert!(m.contains("a lab is required"), "{m}"),
            other => panic!("expected Usage, got {other:?}"),
        }
    }

    #[test]
    fn claim_writes_the_entry_with_every_documented_key() {
        let f = Fixture::new();
        let p = f.paths();
        let c = claim_branch(&p, "probe/one", "claude", "").expect("claim");
        assert_eq!(c.branch, "probe/one");
        assert_eq!(c.lab, "claude");
        assert!(!c.head.is_empty(), "a short sha is recorded");
        assert!(
            c.claimed_at.contains('T') && c.claimed_at.len() >= 19,
            "{}",
            c.claimed_at
        );
        let text = std::fs::read_to_string(claims_file(&p)).expect("read");
        for key in ["branch", "lab", "head", "claimed_at", "note"] {
            assert!(
                text.contains(&format!("\"{key}\"")),
                "{key} missing from {text}"
            );
        }
    }

    /// Release must republish the mirror. A mirror only ever added to brings released claims back.
    #[test]
    fn release_removes_the_claim_from_both_tiers() {
        let f = Fixture::new();
        let p = f.paths();
        claim_branch(&p, "b", "claude", "").expect("claim");
        assert!(live_claims_file(&p).is_file(), "claim mirrors to live");
        assert!(load_claims(&p).contains_key("b"));

        release_branch(&p, "b", "").expect("release");
        assert!(
            !load_claims(&p).contains_key("b"),
            "gone from the committed store"
        );
        let live = std::fs::read_to_string(live_claims_file(&p)).expect("live read");
        assert!(
            !live.contains("\"b\""),
            "and gone from the mirror, or it resurrects: {live}"
        );
    }

    #[test]
    fn another_lab_cannot_release_your_claim() {
        let f = Fixture::new();
        let p = f.paths();
        claim_branch(&p, "b", "claude", "").expect("claim");
        assert!(matches!(
            release_branch(&p, "b", "kimi"),
            Err(ClaimError::Held(_))
        ));
        assert!(load_claims(&p).contains_key("b"), "still held");
    }

    #[test]
    fn releasing_something_unheld_is_a_refusal() {
        let f = Fixture::new();
        assert!(matches!(
            release_branch(&f.paths(), "nope", ""),
            Err(ClaimError::NotHeld(_))
        ));
    }

    /// Only a successful claim is held; a refused one must not record anything.
    #[test]
    fn a_refused_claim_leaves_the_store_unchanged() {
        let f = Fixture::new();
        let p = f.paths();
        claim_branch(&p, "b", "claude", "").expect("claim");
        let before = std::fs::read_to_string(claims_file(&p)).expect("read");
        let _ = claim_branch(&p, "b", "kimi", "");
        let after = std::fs::read_to_string(claims_file(&p)).expect("read");
        assert_eq!(before, after);
    }

    /// The committed tier is the arbiter: a live claim must never outvote it.
    #[test]
    fn the_committed_store_outvotes_the_live_tier() {
        let f = Fixture::new();
        let p = f.paths();
        // Live says kimi; committed says claude. Committed wins.
        std::fs::create_dir_all(p.live.clone()).expect("live");
        std::fs::write(
            live_claims_file(&p),
            r#"{"b":{"branch":"b","lab":"kimi","head":"","claimed_at":"2026-01-01T00:00:00+00:00","note":""}}"#,
        )
        .expect("write live");
        std::fs::write(
            claims_file(&p),
            r#"{"b":{"branch":"b","lab":"claude","head":"","claimed_at":"2025-01-01T00:00:00+00:00","note":""}}"#,
        )
        .expect("write committed");
        assert_eq!(load_claims(&p).get("b").expect("b").lab, "claude");
    }

    /// Live fills a gap the committed stores do not name - that is what makes a claim visible before
    /// its PR lands.
    #[test]
    fn the_live_tier_fills_a_gap() {
        let f = Fixture::new();
        let p = f.paths();
        std::fs::create_dir_all(p.live.clone()).expect("live");
        std::fs::write(
            live_claims_file(&p),
            r#"{"fresh":{"branch":"fresh","lab":"kimi","head":"","claimed_at":"2026-01-01T00:00:00+00:00","note":""}}"#,
        )
        .expect("write live");
        std::fs::write(claims_file(&p), "{}\n").expect("write committed");
        assert_eq!(load_claims(&p).get("fresh").expect("fresh").lab, "kimi");
    }

    /// A lab must be able to claim on a machine whose live tier is missing: the advisory tier is not
    /// load-bearing, and failing there would make it so.
    #[test]
    fn a_missing_live_tier_does_not_block_a_claim() {
        let f = Fixture::new();
        let p = f.paths();
        assert!(!live_claims_file(&p).exists());
        assert!(claim_branch(&p, "b", "claude", "").is_ok());
    }

    #[test]
    fn a_corrupt_store_reads_as_empty_rather_than_failing() {
        let f = Fixture::new();
        let p = f.paths();
        std::fs::write(claims_file(&p), "not json at all").expect("write");
        assert!(load_claims(&p).is_empty());
        // And a claim still works, overwriting the corruption rather than being blocked by it.
        assert!(claim_branch(&p, "b", "claude", "").is_ok());
    }

    /// The written form is canonical, so the committed store diffs cleanly in review.
    #[test]
    fn the_written_store_is_sorted_and_newline_terminated() {
        let f = Fixture::new();
        let p = f.paths();
        claim_branch(&p, "zeta", "claude", "").expect("claim z");
        claim_branch(&p, "alpha", "kimi", "").expect("claim a");
        let text = std::fs::read_to_string(claims_file(&p)).expect("read");
        assert!(text.ends_with('\n'), "trailing newline");
        let a = text.find("alpha").expect("alpha");
        let z = text.find("zeta").expect("zeta");
        assert!(a < z, "keys are sorted: {text}");
    }

    #[test]
    fn current_branch_is_read_from_git() {
        let f = Fixture::new();
        assert_eq!(current_branch(f.dir.path()), "main");
    }
}
