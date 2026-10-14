//! `git clean-merged` and its siblings — branch hygiene, with the delete guarded by proof.
//!
//! # Why this is not `git branch -d`
//!
//! **A squash merge leaves no ancestry link.** `git branch -d` only understands ancestry, so it
//! refuses a branch whose every change landed, and `merge-base --is-ancestor` agrees with it. This
//! repository's queue squash-merges, so the naive check is wrong for exactly the branches a lab wants
//! to clean up.
//!
//! Three different content proofs were tried on this line and two were wrong:
//!
//! * `git rev-list` — fails on a squash merge for the same reason as ancestry.
//! * `git diff main..branch` — flags a branch that is merely **behind**, because the diff includes
//!   everything main has gained since.
//! * **`git cherry origin/main <branch>`** — correct. A `-` line means the change is already upstream.
//!   This is what the guard uses.
//!
//! # The failure this module is shaped around
//!
//! A guard once reported "would delete" for a branch it had **refused**. The caller set a flag, one
//! guard set a reason but left the flag, and the summary read the flag. The rule that came out of it:
//! **one flag with one meaning**, cleared in every refusing path. [`BranchState::removable`] is that
//! flag, and every guard clears it rather than setting a separate reason.

use crate::state::StatePaths;
use std::path::Path;
use std::process::Command;

/// How a local branch relates to `origin/main`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeState {
    /// Proven merged: every change is upstream (squash or ancestry).
    Merged,
    /// Not proven merged. The owner decides.
    Unmerged,
    /// Cannot tell — no `origin/main`, or git failed. **Not removable**, because a proof that did not
    /// run is not a proof.
    Unknown,
}

/// One local branch and what is known about it.
#[derive(Debug, Clone)]
pub struct BranchState {
    pub branch: String,
    pub state: MergeState,
    /// A worktree holds this branch, so deleting it would strand that worktree.
    pub checked_out: bool,
    /// **One flag with one meaning.** True only when every guard passed.
    pub removable: bool,
    /// Why it is not removable, for the report. Advisory text only — never read as permission.
    pub reason: String,
}

/// Classify every local branch except `main`.
pub fn branch_states(repo: &Path) -> Vec<BranchState> {
    let Some(refs) = git(
        repo,
        &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
    ) else {
        return Vec::new();
    };
    let held = checked_out_branches(repo);
    let mut out: Vec<BranchState> = Vec::new();
    for branch in refs
        .lines()
        .map(str::trim)
        .filter(|b| !b.is_empty() && *b != "main")
    {
        out.push(classify(repo, branch, &held));
    }
    out.sort_by(|a, b| a.branch.cmp(&b.branch));
    out
}

/// Decide one branch. Every refusal path clears `removable`; none sets it apart from the proof.
pub fn classify(repo: &Path, branch: &str, held: &[String]) -> BranchState {
    let state = merge_state(repo, branch);
    let checked_out = held.iter().any(|h| h == branch);
    let mut removable = false;
    let mut reason = String::new();

    match state {
        MergeState::Merged => {
            if checked_out {
                // Deleting a checked-out branch fails anyway, and reporting it as removable would
                // invite a command that cannot run.
                reason = "checked out in a worktree".into();
            } else {
                removable = true;
            }
        }
        MergeState::Unmerged => reason = "not merged - its owner decides".into(),
        MergeState::Unknown => {
            // A proof that did not run is not a proof. This is the case that must never default to
            // permission.
            reason = "cannot prove the merge (no origin/main, or git failed)".into();
        }
    }
    BranchState {
        branch: branch.to_string(),
        state,
        checked_out,
        removable,
        reason,
    }
}

/// Is every change on `branch` already in `origin/main`?
pub fn merge_state(repo: &Path, branch: &str) -> MergeState {
    if git(repo, &["rev-parse", "--verify", "origin/main"]).is_none() {
        return MergeState::Unknown;
    }
    // Ancestry first: it is the cheap answer and it covers a real merge commit.
    if git(
        repo,
        &["merge-base", "--is-ancestor", branch, "origin/main"],
    )
    .is_some()
    {
        return MergeState::Merged;
    }
    // Then `git cherry`, which understands a squash merge. `-` means the change is already upstream;
    // `+` means it is NOT. Any `+` line means the branch carries work main does not have.
    let Some(cherry) = git(repo, &["cherry", "origin/main", branch]) else {
        return MergeState::Unknown;
    };
    if cherry.lines().any(|l| l.starts_with('+')) {
        MergeState::Unmerged
    } else {
        // No `+` lines. Either every change is upstream, or the branch has no commits of its own -
        // both are safe, and an empty result is the second.
        MergeState::Merged
    }
}

/// Branches checked out in any worktree, including the main checkout.
pub fn checked_out_branches(repo: &Path) -> Vec<String> {
    let Some(out) = git(repo, &["worktree", "list", "--porcelain"]) else {
        return Vec::new();
    };
    out.lines()
        .filter_map(|l| l.strip_prefix("branch refs/heads/"))
        .map(|b| b.trim().to_string())
        .collect()
}

/// A worktree git considers prunable — its directory is gone.
///
/// Reported because a prunable worktree still **holds** its branch: `git worktree list` names it, so
/// the branch is not removable until it is pruned, and a lab staring at "checked out in a worktree"
/// for a directory that no longer exists needs to be told why.
pub fn has_prunable(repo: &Path) -> bool {
    git(repo, &["worktree", "list", "--porcelain"])
        .map(|o| o.lines().any(|l| l.trim() == "prunable") || o.contains("prunable"))
        .unwrap_or(false)
}

/// Delete the branches proved removable. Returns what was deleted and what refused.
pub fn apply(repo: &Path, states: &[BranchState]) -> (Vec<String>, Vec<(String, String)>) {
    let mut deleted = Vec::new();
    let mut refused = Vec::new();
    for s in states {
        // Re-checked here rather than trusted from the caller: a state computed before a worktree was
        // added would delete a branch that is now held.
        if !s.removable {
            continue;
        }
        match Command::new("git")
            .args(["-C"])
            .arg(repo)
            .args(["branch", "-D", &s.branch])
            .output()
        {
            Ok(o) if o.status.success() => deleted.push(s.branch.clone()),
            Ok(o) => refused.push((
                s.branch.clone(),
                String::from_utf8_lossy(&o.stderr).trim().to_string(),
            )),
            Err(e) => refused.push((s.branch.clone(), e.to_string())),
        }
    }
    (deleted, refused)
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        dir: tempfile::TempDir,
    }

    impl Fixture {
        /// A repo whose `origin/main` is a second commit, so branches can be ahead or behind it.
        fn new() -> Self {
            let dir = tempfile::tempdir().expect("tempdir");
            let repo = dir.path();
            let run = |args: &[&str]| {
                Command::new("git")
                    .args(args)
                    .current_dir(repo)
                    .output()
                    .expect("git")
            };
            run(&["init", "-q", "-b", "main"]);
            run(&["config", "user.email", "t@t"]);
            run(&["config", "user.name", "t"]);
            std::fs::write(repo.join("f"), "1").expect("write");
            run(&["add", "f"]);
            run(&["commit", "-qm", "one"]);
            run(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
            Fixture { dir }
        }

        fn repo(&self) -> &Path {
            self.dir.path()
        }

        fn run(&self, args: &[&str]) -> String {
            let out = Command::new("git")
                .args(args)
                .current_dir(self.repo())
                .output()
                .expect("git");
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
    }

    /// A branch with no commits of its own is merged, not "unknown": an empty `cherry` is the answer.
    #[test]
    fn a_branch_at_main_is_merged() {
        let f = Fixture::new();
        f.run(&["branch", "already"]);
        assert_eq!(merge_state(f.repo(), "already"), MergeState::Merged);
        let s = classify(f.repo(), "already", &[]);
        assert!(s.removable, "{}", s.reason);
    }

    /// The case the whole module exists for: a branch carrying a change main does not have must NOT be
    /// removable.
    #[test]
    fn a_branch_with_its_own_work_is_not_removable() {
        let f = Fixture::new();
        f.run(&["checkout", "-qb", "wip"]);
        std::fs::write(f.repo().join("new"), "x").expect("write");
        f.run(&["add", "new"]);
        f.run(&["commit", "-qm", "work"]);
        assert_eq!(merge_state(f.repo(), "wip"), MergeState::Unmerged);
        let s = classify(f.repo(), "wip", &[]);
        assert!(
            !s.removable,
            "a branch with unique work must never be removable"
        );
        assert!(s.reason.contains("owner decides"), "{}", s.reason);
    }

    /// A squash merge: the branch is NOT an ancestor of main, yet every change is upstream. This is
    /// the shape this repository's queue produces, and the reason ancestry alone is wrong.
    #[test]
    fn a_squash_merged_branch_is_recognised_without_ancestry() {
        let f = Fixture::new();
        f.run(&["checkout", "-qb", "squashed"]);
        std::fs::write(f.repo().join("g"), "y").expect("write");
        f.run(&["add", "g"]);
        f.run(&["commit", "-qm", "feature"]);
        // Squash it into main the way the queue does: same tree, new commit, no ancestry.
        f.run(&["checkout", "-q", "main"]);
        f.run(&["merge", "-q", "--squash", "squashed"]);
        f.run(&["commit", "-qm", "squashed feature"]);
        f.run(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

        assert!(
            git(
                f.repo(),
                &["merge-base", "--is-ancestor", "squashed", "origin/main"]
            )
            .is_none(),
            "a squash merge leaves no ancestry - that is the whole problem"
        );
        assert_eq!(
            merge_state(f.repo(), "squashed"),
            MergeState::Merged,
            "git cherry must recognise it even though ancestry does not"
        );
    }

    /// A branch that is merely BEHIND main must not be reported as carrying unique work.
    ///
    /// `git diff main..branch` gets this wrong - it includes everything main gained since - and that
    /// was one of the two rejected proofs on this line.
    #[test]
    fn a_branch_merely_behind_is_merged_not_unmerged() {
        let f = Fixture::new();
        f.run(&["branch", "behind"]);
        std::fs::write(f.repo().join("f"), "2").expect("write");
        f.run(&["commit", "-qam", "two"]);
        f.run(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        assert_eq!(merge_state(f.repo(), "behind"), MergeState::Merged);
    }

    /// **One flag with one meaning.** A branch held by a worktree is not removable, and the flag is
    /// clear rather than a separate reason a caller might read as permission.
    #[test]
    fn a_checked_out_branch_is_not_removable_and_says_why() {
        let f = Fixture::new();
        // The branch must ACTUALLY be checked out, or the guard has nothing to catch. The first
        // version created the branch and passed the main checkout's list, so `held` did not contain
        // it and the test asserted against a state the guard was never given.
        f.run(&["checkout", "-qb", "held"]);
        let held = checked_out_branches(f.repo());
        assert!(
            held.iter().any(|b| b == "held"),
            "the fixture must actually hold it: {held:?}"
        );
        let s = classify(f.repo(), "held", &held);
        assert!(!s.removable);
        assert!(s.reason.contains("checked out"), "{}", s.reason);
    }

    #[test]
    fn the_main_checkout_branch_is_reported_as_held() {
        let f = Fixture::new();
        assert!(checked_out_branches(f.repo()).iter().any(|b| b == "main"));
    }

    /// A proof that did not run is not a proof. With no `origin/main` nothing is removable.
    #[test]
    fn no_origin_main_makes_everything_unknown_and_nothing_removable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        let run = |args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(repo)
                .output()
                .expect("git")
        };
        run(&["init", "-q", "-b", "main"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        std::fs::write(repo.join("f"), "1").expect("write");
        run(&["add", "f"]);
        run(&["commit", "-qm", "one"]);
        run(&["branch", "whatever"]);

        assert_eq!(merge_state(repo, "whatever"), MergeState::Unknown);
        let s = classify(repo, "whatever", &[]);
        assert!(
            !s.removable,
            "a proof that did not run must never default to permission"
        );
        assert!(s.reason.contains("cannot prove"), "{}", s.reason);
    }

    #[test]
    fn main_is_never_listed() {
        let f = Fixture::new();
        let states = branch_states(f.repo());
        assert!(
            !states.iter().any(|s| s.branch == "main"),
            "main is not a candidate for deletion"
        );
    }

    /// Deleting only happens for branches that are still removable at apply time.
    #[test]
    fn apply_deletes_only_the_removable() {
        let f = Fixture::new();
        f.run(&["branch", "gone"]);
        f.run(&["checkout", "-qb", "keepme"]);
        std::fs::write(f.repo().join("k"), "x").expect("write");
        f.run(&["add", "k"]);
        f.run(&["commit", "-qm", "keep"]);
        f.run(&["checkout", "-q", "main"]);

        let states = branch_states(f.repo());
        let (deleted, refused) = apply(f.repo(), &states);
        assert_eq!(deleted, vec!["gone".to_string()]);
        assert!(refused.is_empty(), "{refused:?}");
        let left = f.run(&["for-each-ref", "--format=%(refname:short)", "refs/heads"]);
        assert!(left.contains("keepme"), "unique work is kept: {left}");
        assert!(!left.contains("gone"), "the merged branch is gone: {left}");
    }

    /// A refused state must not be deleted even if a caller hands it over.
    #[test]
    fn apply_skips_a_state_that_is_not_removable() {
        let f = Fixture::new();
        f.run(&["checkout", "-qb", "wip"]);
        std::fs::write(f.repo().join("n"), "x").expect("write");
        f.run(&["add", "n"]);
        f.run(&["commit", "-qm", "work"]);
        f.run(&["checkout", "-q", "main"]);
        let s = classify(f.repo(), "wip", &[]);
        assert!(!s.removable);
        let (deleted, _) = apply(f.repo(), std::slice::from_ref(&s));
        assert!(deleted.is_empty(), "apply must not delete a refused branch");
    }
}

// ---------------------------------------------------------------------------------------------
// `git switch` - and the guard that makes it safe.
// ---------------------------------------------------------------------------------------------

/// `git -C <repo> <args>`, with stdout and stderr combined, as Python's `_git_raw` returns them.
fn git_raw(repo: &Path, args: &[&str]) -> (i32, String) {
    match std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
    {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).to_string();
            s.push_str(&String::from_utf8_lossy(&o.stderr));
            (o.status.code().unwrap_or(1), s.trim().to_string())
        }
        Err(e) => (1, format!("{e}")),
    }
}

/// `githelp.checkout_preserving_claims` — switch branches without losing an uncommitted claim.
///
/// # Why this is a command and not a `git checkout`
///
/// **`claims.json` is TRACKED, because a claim has to reach the other labs, and it is APPENDED to by
/// every `team claim` and `team release`.** So the ordinary sequence — claim a branch, then check
/// something out — fails with *"Your local changes to the following files would be overwritten by
/// checkout: docs/harness/claims.json"*.
///
/// **The wrong workaround is the easy one**: `git checkout -f` or `git checkout --
/// docs/harness/claims.json` **discards the claim — and a claim nobody has committed is still a lab
/// saying "I am working here"**. Silently dropping it is exactly the collision this plane exists to
/// prevent, which is why the guard is part of the command rather than advice beside it.
///
/// # The mechanism
///
/// Read the modified claims, check out, write them back — **so the claim survives on the new branch,
/// still uncommitted, exactly as the caller left it.**
///
/// **A checkout that FAILS restores nothing**, because the working tree was not changed — writing the
/// preserved text back would be harmless but reporting `claims_preserved: true` would be a lie about
/// work the command did not do.
pub fn checkout_preserving_claims(
    paths: &StatePaths,
    target: &str,
    create: bool,
) -> serde_json::Value {
    let claims = paths.claims_file();
    let mut preserved: Option<String> = None;
    if claims.is_file() {
        let (rc, dirty) = git_raw(
            &paths.repo,
            &["status", "--porcelain", "--", &claims.display().to_string()],
        );
        if rc == 0 && !dirty.trim().is_empty() {
            preserved = std::fs::read_to_string(&claims).ok();
        }
    }

    let mut args: Vec<&str> = vec!["checkout"];
    if create {
        args.push("-b");
    }
    args.push(target);
    let (rc, out) = git_raw(&paths.repo, &args);
    if rc != 0 {
        return serde_json::json!({
            "target": target,
            "switched": false,
            "claims_preserved": false,
            "error": out.trim(),
        });
    }

    let mut restored = false;
    if let Some(text) = preserved {
        if std::fs::write(&claims, text).is_ok() {
            restored = true;
        }
    }
    serde_json::json!({
        "target": target,
        "switched": true,
        "claims_preserved": restored,
        "note": if restored {
            "an uncommitted claim was carried across the checkout"
        } else {
            "no uncommitted claim to carry"
        },
    })
}

#[cfg(test)]
mod checkout_tests {
    use super::*;

    fn scratch() -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().to_path_buf();
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "t@t"],
            vec!["config", "user.name", "t"],
        ] {
            let _ = std::process::Command::new("git")
                .current_dir(&repo)
                .args(&args)
                .output();
        }
        std::fs::write(repo.join("README.md"), "x\n").expect("write");
        std::fs::create_dir_all(repo.join("docs/harness")).expect("mkdir");
        std::fs::write(repo.join("docs/harness/claims.json"), "{}\n").expect("write");
        let _ = std::process::Command::new("git")
            .current_dir(&repo)
            .args(["add", "-A"])
            .output();
        let _ = std::process::Command::new("git")
            .current_dir(&repo)
            .args(["commit", "-qm", "init"])
            .output();
        let paths = StatePaths {
            repo: repo.clone(),
            live: dir.path().join("live"),
        };
        (dir, paths)
    }

    /// **An uncommitted claim SURVIVES the checkout.** This is the whole command: `git checkout -f`
    /// would discard it, and a claim nobody has committed is still a lab saying "I am working here".
    #[test]
    fn an_uncommitted_claim_is_carried_across_the_checkout() {
        let (_d, paths) = scratch();
        let _ = std::process::Command::new("git")
            .current_dir(&paths.repo)
            .args(["branch", "other"])
            .output();
        // Make the claim dirty, exactly as `team claim` does.
        let claim = paths.claims_file();
        std::fs::write(&claim, "{\"lab-branch\": {\"lab\": \"probe\"}}\n").expect("write");

        let out = checkout_preserving_claims(&paths, "other", false);
        assert_eq!(out["switched"], serde_json::json!(true), "{out}");
        assert_eq!(out["claims_preserved"], serde_json::json!(true), "{out}");
        let text = std::fs::read_to_string(&claim).expect("read");
        assert!(
            text.contains("probe"),
            "the claim survived on the new branch: {text}"
        );
    }

    /// **A clean claim leaves nothing to carry**, and the command says so rather than claiming credit.
    #[test]
    fn a_clean_claim_is_reported_as_nothing_to_carry() {
        let (_d, paths) = scratch();
        let _ = std::process::Command::new("git")
            .current_dir(&paths.repo)
            .args(["branch", "other"])
            .output();
        let out = checkout_preserving_claims(&paths, "other", false);
        assert_eq!(out["switched"], serde_json::json!(true), "{out}");
        assert_eq!(out["claims_preserved"], serde_json::json!(false), "{out}");
        assert!(out["note"]
            .as_str()
            .unwrap_or("")
            .contains("no uncommitted"));
    }

    /// **A FAILED checkout does not report the claim as preserved.** The working tree was not
    /// changed, so `claims_preserved: true` would claim work the command did not do.
    #[test]
    fn a_failed_checkout_does_not_claim_to_have_preserved_anything() {
        let (_d, paths) = scratch();
        std::fs::write(paths.claims_file(), "{\"x\": 1}\n").expect("write");
        let out = checkout_preserving_claims(&paths, "no-such-branch-anywhere", false);
        assert_eq!(out["switched"], serde_json::json!(false), "{out}");
        assert_eq!(out["claims_preserved"], serde_json::json!(false), "{out}");
        assert!(
            !out["error"].as_str().unwrap_or("").is_empty(),
            "the reason is reported"
        );
    }

    /// **`create` makes a new branch**, which is the `-b` path.
    #[test]
    fn create_makes_the_branch() {
        let (_d, paths) = scratch();
        let out = checkout_preserving_claims(&paths, "brand-new", true);
        assert_eq!(out["switched"], serde_json::json!(true), "{out}");
        let (rc, name) = git_raw(&paths.repo, &["rev-parse", "--abbrev-ref", "HEAD"]);
        assert_eq!(rc, 0);
        assert_eq!(name.trim(), "brand-new");
    }
}

// ---------------------------------------------------------------------------------------------
// `git delete-merged` - the four-step guard chain.
// ---------------------------------------------------------------------------------------------

/// `githelp.delete_merged_branch` — delete a branch locally AND remotely, **but only if its merge is
/// PROVEN**.
///
/// # Why this exists, in the words of the mistake that created it
///
/// A pull request was merged with `gh pr merge --auto`, its state was polled, **and then the remote
/// delete ran unconditionally. The PR had auto-closed as DIRTY midway through the poll; the loop
/// exited without noticing, the delete ran anyway, and the only remote copy of unmerged work was
/// removed.** The work survived in a local commit and was recovered — **but nothing in the command
/// sequence prevented the loss, and a script that deletes a branch without asking whether the merge
/// happened will eventually delete work that cannot be recovered.**
///
/// # The chain, and each step is a defence against a specific failure
///
/// **1. A LIVE CLAIM, CHECKED FIRST, AND IT NEEDS NO NETWORK.** *"A merged PR is not proof a branch is
/// finished: a lab can merge and keep going, or hold the branch while a follow-up lands."* **Gating on
/// the merge first also meant an unreachable GitHub returned before this guard ran, so a claimed branch
/// could be deleted whenever the network was down — the worst arrangement, since the protection failed
/// exactly when the evidence was weakest.** The first version of this recommended deleting
/// `kimi/plan-refresh-2026-10-13` — merged, yes, **but under Kimi's live claim**, and *"a harness that
/// deletes a peer's working branch is worse than one that leaves a stale branch behind."*
///
/// **2. A MERGED PR WHOSE MERGE COMMIT IS AN ANCESTOR OF `origin/main`.** A PR marked merged whose
/// commit is *not* on main is reported as such **and refused**.
///
/// **3. `git cherry` — DOES THIS BRANCH INTRODUCE ANYTHING UPSTREAM LACKS?** Two earlier attempts were
/// wrong in opposite directions: `rev-list --count` (ancestry) **refused every squash-merged branch**,
/// because a squash leaves no ancestor; `diff --stat` (whole tree) **refused a branch merely BEHIND
/// main**, flagging five files belonging to other labs that the branch never touched.
///
/// **`safe_to_delete` IS THE ONLY FLAG THE CALLER ACTS ON**, because an earlier version used `merged`
/// and *"a guard returned early without clearing it, so a REFUSED branch still printed 'would
/// delete'."* One flag with one meaning makes that impossible — **and this port keeps one flag, so the
/// same mistake cannot be reintroduced by a caller reading the wrong field.**
///
/// # `apply=false` reports what would happen and changes nothing
pub fn delete_merged_branch(paths: &StatePaths, branch: &str, apply: bool) -> serde_json::Value {
    let git = |args: &[&str]| git_raw(&paths.repo, args);
    let mut result = serde_json::json!({
        "branch": branch,
        "safe_to_delete": false,
        "merged": false,
        "reason": "",
        "deleted": false,
    });

    // ---- 1. A live claim, first, and it needs no network. -------------------------------------
    if let Some(claim) = crate::team::load_claims(paths).get(branch) {
        let owner = if claim.lab.is_empty() {
            "an unidentified lab"
        } else {
            claim.lab.as_str()
        };
        // **CLEAR the flag, not just the reason.** A guard that explains itself and leaves
        // `safe_to_delete` set still tells the caller to go ahead - **the flag is what acts; the
        // reason is only what a human reads.** This was the same mistake twice in one function.
        result["safe_to_delete"] = serde_json::json!(false);
        result["reason"] = serde_json::json!(format!(
            "{owner} holds a live claim on this branch - merged or not, its owner decides when it is \
             finished. Release the claim first if the work is done: \
             uor-harness team release --branch {branch} --lab {owner}"
        ));
        return result;
    }

    // ---- 2. A merged PR whose merge commit is on origin/main. ---------------------------------
    let (ok, data, err) = crate::githubcards::gh_json(
        &[
            "pr",
            "list",
            "--head",
            branch,
            "--state",
            "merged",
            "--limit",
            "1",
            "--json",
            "number,mergeCommit",
        ],
        &paths.repo,
    );
    if !ok {
        result["reason"] = serde_json::json!(format!(
            "cannot ask GitHub about the PR ({}), and an unverified merge is not a merge",
            if err.is_empty() {
                "unknown error"
            } else {
                err.as_str()
            }
        ));
        return result;
    }
    let first = data.as_array().and_then(|a| a.first()).cloned();
    let Some(pr) = first else {
        result["reason"] = serde_json::json!("no merged PR for this branch");
        return result;
    };
    let merge_oid = pr
        .get("mergeCommit")
        .and_then(|c| c.get("oid"))
        .and_then(|o| o.as_str())
        .unwrap_or("");
    let num = pr.get("number").and_then(|n| n.as_i64()).unwrap_or(0);
    if merge_oid.is_empty() {
        result["reason"] =
            serde_json::json!(format!("PR #{num} is merged but reports no merge commit"));
        return result;
    }
    let (rc, _) = git(&["merge-base", "--is-ancestor", merge_oid, "origin/main"]);
    if rc != 0 {
        result["reason"] = serde_json::json!(format!(
            "PR #{num} is marked merged ({}) but that commit is NOT on origin/main",
            &merge_oid[..merge_oid.len().min(9)]
        ));
        return result;
    }
    result["merged"] = serde_json::json!(true);
    result["safe_to_delete"] = serde_json::json!(true);
    result["reason"] = serde_json::json!(format!(
        "PR #{num} merged as {}, on main",
        &merge_oid[..merge_oid.len().min(9)]
    ));

    // ---- 3. `git cherry`: does the branch introduce anything upstream lacks? -------------------
    let (rc, _) = git(&[
        "rev-parse",
        "--verify",
        "--quiet",
        &format!("refs/remotes/origin/{branch}"),
    ]);
    if rc == 0 {
        let (rc2, cherry) = git(&["cherry", "origin/main", &format!("origin/{branch}")]);
        if rc2 == 0 {
            let unpushed = cherry.lines().filter(|l| l.starts_with('+')).count();
            if unpushed > 0 {
                result["reason"] = serde_json::json!(format!(
                    "origin/{branch} holds {unpushed} commit(s) whose changes are not in main - the merge did not carry them"
                ));
                result["merged"] = serde_json::json!(false);
                result["safe_to_delete"] = serde_json::json!(false);
                return result;
            }
        }
    }

    if !apply {
        return result;
    }

    // ---- 4. Local first, then remote: a failed remote delete leaves the local copy for the next
    //         attempt. ---------------------------------------------------------------------------
    let (rc, out) = git(&["branch", "-D", branch]);
    if rc == 0 {
        result["deleted"] = serde_json::json!(true);
    } else {
        result["local_error"] = serde_json::json!(out);
    }
    let (rc, out) = git(&["push", "origin", "--delete", branch]);
    result["remote_deleted"] = serde_json::json!(rc == 0);
    // **A remote ref that never existed is not an error** - the branch may be local-only, and
    // reporting that as a failure would make an otherwise clean delete look partial.
    if rc != 0 && !out.contains("remote ref does not exist") {
        result["remote_error"] = serde_json::json!(out);
    }
    result
}

#[cfg(test)]
mod delete_merged_tests {
    use super::*;

    fn scratch_with_claim(branch: &str, lab: &str) -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().to_path_buf();
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "t@t"],
            vec!["config", "user.name", "t"],
        ] {
            let _ = std::process::Command::new("git")
                .current_dir(&repo)
                .args(&args)
                .output();
        }
        std::fs::write(repo.join("README.md"), "x\n").expect("write");
        std::fs::create_dir_all(repo.join("docs/harness")).expect("mkdir");
        let claims = serde_json::json!({ branch: { "lab": lab, "branch": branch, "head": "", "claimed_at": "", "note": "" } });
        std::fs::write(
            repo.join("docs/harness/claims.json"),
            serde_json::to_string_pretty(&claims).expect("json"),
        )
        .expect("write");
        let _ = std::process::Command::new("git")
            .current_dir(&repo)
            .args(["add", "-A"])
            .output();
        let _ = std::process::Command::new("git")
            .current_dir(&repo)
            .args(["commit", "-qm", "init"])
            .output();
        let paths = StatePaths {
            repo: repo.clone(),
            live: dir.path().join("live"),
        };
        (dir, paths)
    }

    /// **A LIVE CLAIM REFUSES THE DELETE, AND IT DOES SO WITHOUT A NETWORK.**
    ///
    /// This is the guard the docstring calls *"the worst arrangement"* to get wrong: **gating on the
    /// merge first meant an unreachable GitHub returned before this check ran, so a claimed branch
    /// could be deleted whenever the network was down — the protection failed exactly when the
    /// evidence was weakest.**
    ///
    /// **The test passes with no `gh` involvement at all**, which is the property being asserted: it
    /// would fail if the claim check moved below the PR lookup.
    #[test]
    fn a_live_claim_refuses_the_delete_without_needing_the_network() {
        let (_d, paths) = scratch_with_claim("peer/branch", "kimi");
        let r = delete_merged_branch(&paths, "peer/branch", false);
        assert_eq!(r["safe_to_delete"], serde_json::json!(false), "{r}");
        assert_eq!(r["merged"], serde_json::json!(false), "{r}");
        let reason = r["reason"].as_str().unwrap_or("");
        assert!(reason.contains("kimi holds a live claim"), "{reason}");
        // **The refusal tells the owner how to release it**, because a refusal without a next step is
        // a dead end for the lab that hits it.
        assert!(
            reason.contains("team release --branch peer/branch --lab kimi"),
            "{reason}"
        );
    }

    /// **A live claim is checked even with `--apply`.** The flag decides whether to act, **not whether
    /// to check** — and a version that skipped the guard under `--apply` would delete a peer's branch.
    #[test]
    fn apply_does_not_bypass_the_claim_guard() {
        let (_d, paths) = scratch_with_claim("peer/branch", "kimi");
        let r = delete_merged_branch(&paths, "peer/branch", true);
        assert_eq!(r["safe_to_delete"], serde_json::json!(false), "{r}");
        // **`deleted` is ABSENT on this path, not false** - Python returns the same, because the guard
        // returns from the top-level dict before those keys are added. **My first assertion expected
        // `false` and failed**, and the real shape matters: a caller reading `deleted` must treat
        // missing and false alike, **which is exactly why `safe_to_delete` is the one flag that is
        // always present and the only one the docstring says to act on.**
        assert_ne!(
            r["deleted"],
            serde_json::json!(true),
            "nothing was deleted: {r}"
        );
        assert_ne!(r["remote_deleted"], serde_json::json!(true), "{r}");
    }

    /// **`safe_to_delete` is cleared on EVERY refusal, not only the claim one.**
    ///
    /// The guard's history records the same mistake twice: *"a guard that explains itself but leaves
    /// `safe_to_delete` set still tells the caller to go ahead. The flag is what acts; the reason is
    /// only what a human reads."* **This asserts the flag on the unverifiable-merge path, which is the
    /// other early return.**
    #[test]
    fn an_unprovable_merge_clears_safe_to_delete() {
        let (_d, paths) = scratch_with_claim("nobody/claims/this", "kimi");
        // No PR exists for this branch and there is no `gh` answer to trust, so the chain stops.
        let r = delete_merged_branch(&paths, "nobody/claims/this", false);
        assert_eq!(
            r["safe_to_delete"],
            serde_json::json!(false),
            "a refusal must clear the flag the caller acts on: {r}"
        );
        assert!(
            !r["reason"].as_str().unwrap_or("").is_empty(),
            "and it must say why: {r}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// `git install-hooks` - the managed pre-commit hook, and the conflicts it refuses.
// ---------------------------------------------------------------------------------------------

/// The marker that proves the harness wrote an existing hook.
pub const HOOK_START: &str =
    "# >>> uor-harness pre-commit v1 (managed by scripts/harness/githelp.py) >>>";

/// The hook body, **carried as a data file rather than a Rust literal**.
///
/// # Why a data file
///
/// **This is 1983 bytes of `/bin/sh` with an embedded Python heredoc**, and it is the one artifact in
/// the port that a *different language* has to read correctly. Three reasons it lives in
/// `rules/precommit-hook.sh`:
///
/// * **A raw string with a Python heredoc inside it is exactly the kind of literal a Rust edit
///   mangles** — this port has already corrupted source twice by writing a backtick-bearing string
///   through a shell.
/// * **The file is diffable as what it is.** A reviewer reads shell, not an escaped Rust literal.
/// * **The D23 interlock reads `.rs` files**, and **a hook is not model code** — keeping it out of the
///   source tree's Rust files keeps that scan about what it is for.
///
/// **`include_str!` means it is still compiled in**: there is no runtime file read, and **a missing
/// file is a build error rather than a silently empty hook.**
pub const HOOK_SCRIPT: &str = include_str!("../rules/precommit-hook.sh");

/// `githelp._hooks_dir` — the directory git actually reads hooks from.
///
/// **`git rev-parse --git-path hooks`, not `.git/hooks`**, because **in a linked worktree the hooks
/// live in the COMMON dir** — writing to `.git/hooks` there would install a hook that never runs.
fn hooks_dir(paths: &StatePaths) -> Result<std::path::PathBuf, String> {
    let (rc, out) = git_raw(&paths.repo, &["rev-parse", "--git-path", "hooks"]);
    // **A non-zero exit is the failure**, and git's own stderr is the better reason when it has one.
    if rc != 0 {
        return Err(if out.trim().is_empty() {
            "`git rev-parse --git-path hooks` failed".to_string()
        } else {
            out.trim().to_string()
        });
    }
    let raw = out.trim();
    if raw.is_empty() {
        return Err("`git rev-parse --git-path hooks` returned nothing".into());
    }
    let p = std::path::PathBuf::from(raw);
    // A relative answer is relative to the repo, which is how git prints it from a worktree.
    Ok(if p.is_absolute() {
        p
    } else {
        paths.repo.join(p)
    })
}

/// `githelp.install_hooks` — install or refresh the marked pre-commit hook.
///
/// # It never clobbers a foreign hook, and that is three separate refusals
///
/// * **A SYMLINK is refused outright.** A symlinked `pre-commit` is somebody's deliberate wiring —
///   often a shared hook manager — and `write_text` through it would **write to the target**.
/// * **An existing file with no marker is refused**, with the marker it looked for and the first line
///   of what it found, **because those two lines are what a reader needs to decide whether to merge by
///   hand or delete it.**
/// * **An unreadable file is refused** rather than replaced: not being able to see what is there is
///   not permission to overwrite it.
///
/// **Every refusal returns 2 and names the command to re-run**, so the reader is not left to guess.
/// **`unchanged` is reported when the bytes are already right**, which is what makes a re-run safe.
pub fn install_hooks(paths: &StatePaths) -> (i32, Vec<String>, Vec<String>) {
    let mut out: Vec<String> = Vec::new();
    let mut err: Vec<String> = Vec::new();
    let dir = match hooks_dir(paths) {
        Ok(d) => d,
        Err(e) => {
            err.push(format!("cannot locate the hooks directory: {e}"));
            return (1, out, err);
        }
    };
    let hook = dir.join("pre-commit");
    let symlink = std::fs::symlink_metadata(&hook).is_ok_and(|m| m.file_type().is_symlink());
    let existed = hook.exists() || symlink;
    let mut previous = String::new();

    if symlink {
        err.push(format!(
            "CONFLICT: {} is a symlink; refusing to replace it.",
            hook.display()
        ));
        err.push(
            "  resolve it yourself, or remove it and re-run: uor-harness git install-hooks".into(),
        );
        return (2, out, err);
    }
    if existed {
        match std::fs::read_to_string(&hook) {
            Ok(t) => previous = t,
            Err(e) => {
                err.push(format!(
                    "CONFLICT: cannot read the existing pre-commit hook {}: {e}",
                    hook.display()
                ));
                return (2, out, err);
            }
        }
        if !previous.contains(HOOK_START) {
            let first = previous.lines().next().unwrap_or("(empty file)");
            err.push(format!(
                "CONFLICT: {} already exists and was not written by the harness (no marker line).",
                hook.display()
            ));
            err.push(format!("  marker looked for: {HOOK_START}"));
            err.push(format!(
                "  existing file starts: {}",
                crate::prbody::truncate(first, 120)
            ));
            err.push(
                "  merge it by hand, or remove it and re-run: uor-harness git install-hooks".into(),
            );
            return (2, out, err);
        }
    }

    if let Err(e) = std::fs::create_dir_all(&dir) {
        err.push(format!("cannot write {}: {e}", hook.display()));
        return (1, out, err);
    }
    if let Err(e) = std::fs::write(&hook, HOOK_SCRIPT) {
        err.push(format!("cannot write {}: {e}", hook.display()));
        return (1, out, err);
    }
    // **The executable bit is part of the artifact** - a hook that is not executable is a file git
    // silently ignores, and reporting "installed" for one would be the worst outcome here.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755));
    }

    let action = if existed && previous != HOOK_SCRIPT {
        "updated"
    } else if existed {
        "unchanged"
    } else {
        "installed"
    };
    out.push(format!("{action}: {}", hook.display()));
    out.push("  runs: uor-harness check --staged".into());
    out.push("  blocks: error severity only (warn/info print and pass); exits 0 without python3 or the harness".into());
    if existed && action == "unchanged" {
        out.push("  already up to date; nothing was appended".into());
    }
    (0, out, err)
}

#[cfg(test)]
mod install_hooks_tests {
    use super::*;

    fn scratch() -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().to_path_buf();
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "t@t"],
            vec!["config", "user.name", "t"],
        ] {
            let _ = std::process::Command::new("git")
                .current_dir(&repo)
                .args(&args)
                .output();
        }
        std::fs::write(repo.join("README.md"), "x\n").expect("write");
        let paths = StatePaths {
            repo: repo.clone(),
            live: dir.path().join("live"),
        };
        (dir, paths)
    }

    fn hook_of(paths: &StatePaths) -> std::path::PathBuf {
        hooks_dir(paths).expect("hooks dir").join("pre-commit")
    }

    /// **It installs, is executable, and a re-run says `unchanged`** — which is what makes the command
    /// safe to run repeatedly.
    ///
    /// **The executable bit is asserted**, because **a hook that is not executable is a file git
    /// silently ignores** — and reporting "installed" for one would be the worst outcome this command
    /// can produce.
    #[test]
    fn it_installs_executable_and_a_rerun_is_unchanged() {
        let (_d, paths) = scratch();
        let (code, out, err) = install_hooks(&paths);
        assert_eq!(code, 0, "{out:?} {err:?}");
        assert!(out[0].starts_with("installed:"), "{out:?}");
        let hook = hook_of(&paths);
        let body = std::fs::read_to_string(&hook).expect("read");
        assert_eq!(
            body, HOOK_SCRIPT,
            "the hook is byte-identical to the reference"
        );
        assert!(body.contains(HOOK_START), "and carries its marker");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&hook).expect("meta").permissions().mode();
            assert_eq!(
                mode & 0o111,
                0o111,
                "the hook must be executable, mode {mode:o}"
            );
        }
        let (code2, out2, _) = install_hooks(&paths);
        assert_eq!(code2, 0);
        assert!(out2[0].starts_with("unchanged:"), "{out2:?}");
    }

    /// **A foreign hook is REFUSED, not clobbered**, and the refusal names the marker and the first
    /// line it found — **those two lines are what a reader needs to decide whether to merge or delete.**
    #[test]
    fn a_foreign_hook_is_refused_and_its_content_is_left_alone() {
        let (_d, paths) = scratch();
        let hook = hook_of(&paths);
        std::fs::create_dir_all(hook.parent().expect("parent")).expect("mkdir");
        std::fs::write(&hook, "#!/bin/sh\n# somebody else's hook\necho hi\n").expect("write");

        let (code, _out, err) = install_hooks(&paths);
        assert_eq!(code, 2, "a conflict is a refusal, not a success: {err:?}");
        assert!(
            err.iter()
                .any(|l| l.contains("was not written by the harness")),
            "{err:?}"
        );
        assert!(
            err.iter().any(|l| l.contains("marker looked for")),
            "{err:?}"
        );
        // **The FIRST line, which is the shebang** — not the comment below it. **My first assertion
        // looked for the comment and failed**, and the difference is the point: a reader deciding
        // whether to merge by hand needs to RECOGNISE the file, and `#!/bin/sh` is what every hook
        // starts with. Python truncates the same line to 120 characters.
        assert!(
            err.iter()
                .any(|l| l.contains("existing file starts: #!/bin/sh")),
            "{err:?}"
        );
        // **The original is untouched** — the whole point of refusing.
        let after = std::fs::read_to_string(&hook).expect("read");
        assert!(
            after.contains("somebody else's hook"),
            "not clobbered: {after}"
        );
    }

    /// **A SYMLINK is refused outright**, because writing through it would **write to the target**.
    #[cfg(unix)]
    #[test]
    fn a_symlinked_hook_is_refused_and_the_target_is_not_touched() {
        let (_d, paths) = scratch();
        let hook = hook_of(&paths);
        std::fs::create_dir_all(hook.parent().expect("parent")).expect("mkdir");
        let target = paths.repo.join("elsewhere.sh");
        std::fs::write(&target, "#!/bin/sh\necho target\n").expect("write");
        std::os::unix::fs::symlink(&target, &hook).expect("symlink");

        let (code, _out, err) = install_hooks(&paths);
        assert_eq!(code, 2, "{err:?}");
        assert!(err.iter().any(|l| l.contains("is a symlink")), "{err:?}");
        // **The target is byte-identical** — this is the assertion that makes the symlink guard real.
        assert_eq!(
            std::fs::read_to_string(&target).expect("read"),
            "#!/bin/sh\necho target\n",
            "the symlink target must not be written through"
        );
    }

    /// **A harness-written hook IS refreshed**, so a lab that installed an older version gets the new
    /// one rather than a refusal — **the marker is what distinguishes "mine, update it" from
    /// "somebody else's, leave it".**
    #[test]
    fn a_previous_harness_hook_is_updated_rather_than_refused() {
        let (_d, paths) = scratch();
        let hook = hook_of(&paths);
        std::fs::create_dir_all(hook.parent().expect("parent")).expect("mkdir");
        std::fs::write(&hook, format!("#!/bin/sh\n{HOOK_START}\n# an older body\n"))
            .expect("write");
        let (code, out, err) = install_hooks(&paths);
        assert_eq!(code, 0, "{err:?}");
        assert!(out[0].starts_with("updated:"), "{out:?}");
        assert_eq!(std::fs::read_to_string(&hook).expect("read"), HOOK_SCRIPT);
    }
}
