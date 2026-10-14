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

/// The main checkout `git worktree list` names first - the owner's, where `main` is checked out.
///
/// **Not `budget::main_worktree`, deliberately.** That one SKIPS the caller and returns `None` when the
/// caller is the main checkout, because it is looking for the pod tool. This one must return the caller's
/// own path in that case, so `load_claims` can compare and skip a duplicate read.
///
/// **This is not a write path.** R10 forbids writing into the owner's checkout; nothing here ever writes
/// to what this returns. It is read-only, and it is the tier that makes a peer's merged claim visible to a
/// lab whose own `claims.json` predates it.
fn main_checkout(paths: &StatePaths) -> PathBuf {
    let out = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(&paths.repo)
        .output();
    if let Ok(o) = out {
        if o.status.success() {
            for line in String::from_utf8_lossy(&o.stdout).lines() {
                if let Some(rest) = line.strip_prefix("worktree ") {
                    let candidate = PathBuf::from(rest.trim());
                    // `git worktree list` names the primary checkout FIRST.
                    if candidate.is_dir() {
                        return candidate;
                    }
                }
            }
        }
    }
    paths.repo.clone()
}

/// The committed store on `main`: the fallback for a lab that has never written its own.
pub fn shared_claims_file(paths: &StatePaths) -> PathBuf {
    main_checkout(paths)
        .join("docs")
        .join("harness")
        .join("claims.json")
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

/// The COMMITTED store alone - what the write path must load.
///
/// **The distinction is the whole bug this function exists to fix.** `load_claims` merges the live
/// tier in for READING, which is right: a peer's claim should be visible before its PR lands. But
/// writing that merged view back to the committed store **imports every peer's live claim into the
/// file that gets reviewed and merged** — a release by one lab committed ten other labs' transient
/// claims, and the working tree went dirty on a command that should not have touched it.
///
/// Reported by Antigravity after running the onboarding exercise, with Codex seeing the same on
/// #2029. It is the defect class this line keeps producing: **a read view used as a write source.**
pub fn load_committed(paths: &StatePaths) -> BTreeMap<String, Claim> {
    read_tier(&claims_file(paths))
}

/// Every claim: live filling gaps, `main` as the shared fallback, the caller's store the authority.
pub fn load_claims(paths: &StatePaths) -> BTreeMap<String, Claim> {
    let mut merged = BTreeMap::new();
    let live = live_claims_file(paths);
    let shared = shared_claims_file(paths);
    let local = claims_file(paths);
    // Weakest first, strongest last: the caller's committed store overwrites everything it names.
    //
    // **The tier is skipped BY NAME, not by comparing paths.** The first version of this wrote
    // `if *path == shared && shared == local { continue; }`, which is true for the LOCAL entry as well
    // when the two paths are equal - so in the main checkout it skipped the caller's own store and
    // `load_claims` returned only the live tier. Two `multilabrules` tests caught it, and the probe
    // that printed `shared==local? true` with `merged = []` is what explained it.
    for (tier, path) in [("live", &live), ("shared", &shared), ("local", &local)] {
        // The caller IS the main checkout: do not read one file twice as two tiers.
        if tier == "shared" && shared == local {
            continue;
        }
        for (k, v) in read_tier(path) {
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
    // COMMITTED ONLY for the write. Reading the merged view here was the bug: it put every peer's
    // live claim into the file this writes.
    let mut claims = load_committed(paths);
    // The CONFLICT check uses the merged view, because refusing a branch a peer holds only in the
    // live tier would be wrong - that is exactly what the live mirror is for.
    if let Some(existing) = load_claims(paths).get(branch) {
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
    // COMMITTED ONLY for the write, for the reason on `load_committed`.
    let mut claims = load_committed(paths);
    let Some(existing) = load_claims(paths).get(branch).cloned() else {
        return Err(ClaimError::NotHeld(branch.to_string()));
    };
    // Releasing a claim that exists only in the live tier must still remove it there, so the
    // committed store keeps whatever it had and the mirror is republished from it below.
    claims.remove(branch);
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

    /// **The regression Antigravity found.** A live-tier peer claim must NOT be imported into the
    /// committed store by a local claim or release. Before the fix, one lab releasing its own branch
    /// committed ten other labs' transient claims and dirtied the tree.
    #[test]
    fn a_live_peer_claim_is_never_written_into_the_committed_store() {
        let f = Fixture::new();
        let p = f.paths();
        // A peer's claim exists ONLY in the live tier.
        std::fs::create_dir_all(p.live.clone()).expect("live");
        std::fs::write(
            live_claims_file(&p),
            r#"{"peer/branch":{"branch":"peer/branch","lab":"kimi","head":"","claimed_at":"2026-01-01T00:00:00+00:00","note":""}}"#,
        )
        .expect("write live");
        std::fs::write(claims_file(&p), "{}\n").expect("write committed");

        // It IS visible for reading, which is what the live mirror is for.
        assert!(
            load_claims(&p).contains_key("peer/branch"),
            "the live tier is readable"
        );

        // A local claim must not commit it.
        claim_branch(&p, "mine", "deepseek", "").expect("claim");
        let committed = std::fs::read_to_string(claims_file(&p)).expect("read");
        assert!(
            !committed.contains("peer/branch"),
            "a peer's live claim was imported into the committed store: {committed}"
        );

        // And a release must not either.
        release_branch(&p, "mine", "deepseek").expect("release");
        let committed = std::fs::read_to_string(claims_file(&p)).expect("read");
        assert!(
            !committed.contains("peer/branch"),
            "a release imported a peer's live claim: {committed}"
        );
    }

    /// A peer holding a branch ONLY in the live tier is still refused - that is what the mirror is
    /// for, and it must not become the reason a claim succeeds.
    #[test]
    fn a_live_tier_peer_claim_still_refuses_a_conflicting_claim() {
        let f = Fixture::new();
        let p = f.paths();
        std::fs::create_dir_all(p.live.clone()).expect("live");
        std::fs::write(
            live_claims_file(&p),
            r#"{"b":{"branch":"b","lab":"kimi","head":"","claimed_at":"2026-01-01T00:00:00+00:00","note":""}}"#,
        )
        .expect("write live");
        std::fs::write(claims_file(&p), "{}\n").expect("write committed");
        match claim_branch(&p, "b", "deepseek", "") {
            Err(ClaimError::Held(c)) => assert_eq!(c.lab, "kimi"),
            other => panic!("expected Held from the live tier, got {other:?}"),
        }
    }

    #[test]
    fn current_branch_is_read_from_git() {
        let f = Fixture::new();
        assert_eq!(current_branch(f.dir.path()), "main");
    }

    /// **The shared tier was missing, and this is the test that catches it.**
    ///
    /// A real repository with a real linked worktree: the claim is committed on the main checkout, the
    /// worktree's own `claims.json` is EMPTY, and the live tier points at an empty directory. Before the
    /// fix `load_claims` returned nothing here, where Python returned the committed claim - so a lab in
    /// that position could start work on a branch another lab already holds.
    #[test]
    fn a_stale_worktree_still_sees_the_claim_committed_on_main() {
        let dir = std::env::temp_dir().join(format!("harness-claims-tier-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("docs").join("harness")).expect("scratch");
        let live = dir.join("live");
        std::fs::create_dir_all(&live).expect("live");
        let git = |args: &[&str], cwd: &std::path::Path| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(cwd)
                .output()
                .expect("git")
        };
        git(&["init", "-q"], &dir);
        git(&["config", "user.email", "t@example.com"], &dir);
        git(&["config", "user.name", "t"], &dir);
        let claim = concat!(
            "{\"peer/branch\": {\"branch\": \"peer/branch\", \"lab\": \"claude\", ",
            "\"claimed_at\": \"2026-10-14T09:00:00-04:00\", \"head\": \"abc\", \"note\": \"\"}}\n"
        );
        std::fs::write(dir.join("docs").join("harness").join("claims.json"), claim).expect("write");
        git(&["add", "-A"], &dir);
        git(&["commit", "-qm", "claim"], &dir);
        // A linked worktree with its own EMPTY store - the stale case.
        let wt = dir.join("wt");
        git(&["worktree", "add", "-q", wt.to_str().unwrap()], &dir);
        std::fs::write(wt.join("docs").join("harness").join("claims.json"), "{}\n").expect("write");
        let paths = StatePaths::resolve(&wt, None, Some(live.to_str().unwrap()));
        // If the two paths were equal this test would prove nothing - and it also guards the naming bug
        // the first attempt had, where an equal pair skipped the caller's own tier.
        assert_ne!(
            shared_claims_file(&paths),
            claims_file(&paths),
            "the worktree's store and main's must be different files"
        );
        let merged = load_claims(&paths);
        assert!(
            merged.contains_key("peer/branch"),
            "a claim committed on main must be visible from a stale worktree: {:?}",
            merged.keys().collect::<Vec<_>>()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The converse: when the caller IS the main checkout, its own store is still read as the authority.
    ///
    /// This is the case the naming bug broke: `shared == local` skipped BOTH the shared and the local
    /// entry, so every committed claim disappeared in the main checkout.
    #[test]
    fn the_caller_is_the_authority_when_it_is_the_main_checkout() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs").join("harness")).expect("mkdir");
        let claim = concat!(
            "{\"mine/branch\": {\"branch\": \"mine/branch\", \"lab\": \"deepseek\", ",
            "\"claimed_at\": \"2026-10-14T09:00:00-04:00\", \"head\": \"abc\", \"note\": \"\"}}\n"
        );
        std::fs::write(root.join("docs").join("harness").join("claims.json"), claim)
            .expect("write");
        let paths = StatePaths::resolve(root, None, Some("/nonexistent-live"));
        // No git repo here, so the main-checkout resolution falls back to the caller: the two are equal.
        assert_eq!(shared_claims_file(&paths), claims_file(&paths));
        assert!(
            load_claims(&paths).contains_key("mine/branch"),
            "the caller's own store must still be read when it is also the shared path"
        );
    }
}
