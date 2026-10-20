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

/// Branches that are never deleted, whatever the registry file says (owner, 20 October 2026).
///
/// The registry in `docs/harness/branches.json` is the documented list; this built-in copy is the
/// floor, so a missing or damaged registry can add protection but never remove it.
pub const BUILTIN_PROTECTED: [&str; 3] = ["main", "gh-pages", "gh-pages-preview"];

/// The protected branch names: the built-in floor plus every `protected[].name` in the registry.
pub fn protected_branches(repo: &Path) -> Vec<String> {
    let mut names: Vec<String> = BUILTIN_PROTECTED.iter().map(|s| s.to_string()).collect();
    let registry = std::fs::read_to_string(repo.join("docs/harness/branches.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok());
    if let Some(rows) = registry
        .as_ref()
        .and_then(|v| v.get("protected"))
        .and_then(|p| p.as_array())
    {
        for name in rows.iter().filter_map(|r| r.get("name")?.as_str()) {
            if !names.iter().any(|n| n == name) {
                names.push(name.to_string());
            }
        }
    }
    names
}

/// The refusal every branch-delete path returns for a protected name, local or remote.
pub fn protected_refusal(repo: &Path, branch: &str) -> Option<String> {
    let name = branch.trim_start_matches("refs/heads/");
    // Case-insensitive: on a case-insensitive filesystem `git branch -D Main` can remove `main`.
    protected_branches(repo)
        .iter()
        .any(|p| p.eq_ignore_ascii_case(name))
        .then(|| {
        format!(
            "refusing to delete protected branch '{name}': it is listed in \
             docs/harness/branches.json and long-lived branches are never deleted (owner, 20 Oct 2026)"
        )
    })
}

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
        .filter(|b| !b.is_empty() && protected_refusal(repo, b).is_none())
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
    // Then `git cherry` as a FAST LANE: no `+` line settles it without computing a patch-id at all.
    // `-` means the commit is already upstream, and an empty result means the branch has no commits of
    // its own - both are safe, and both are Merged.
    let Some(cherry) = git(repo, &["cherry", "origin/main", branch]) else {
        return MergeState::Unknown;
    };
    if !cherry.lines().any(|l| l.starts_with('+')) {
        return MergeState::Merged;
    }
    // **A `+` line does NOT mean unmerged, and believing it did was the defect.** `cherry` compares
    // PER-COMMIT patch ids, and a squash merge lands the branch as ONE commit: none of the branch's
    // own commits ever match it, so every multi-commit branch read "unmerged" forever. `clean-merged`
    // therefore kept the branch ("its owner decides") and, since the claims release landed, never
    // released its claim either. Measured on `deepseek/merged-claim-release`: five `+` lines, a branch
    // tree byte-identical to the merge commit, and a cumulative patch-id equal to the squash commit's
    // (`122650e7...`). Kimi PM ruled the replacement (#2028 comment 6197423733).
    //
    // So compare the WHOLE branch against what main gained since the merge base. One cumulative
    // patch-id against the upstream commit ids: a match proves the branch content is in main even
    // though no single commit of the branch matches. If it cannot be computed the answer is Unknown,
    // never Unmerged - "could not prove it" must not become "keep it forever" OR "delete it".
    let Some(base) = git(repo, &["merge-base", "origin/main", branch]) else {
        return MergeState::Unknown;
    };
    let Some(branch_id) = cumulative_patch_id(repo, &base, branch) else {
        return MergeState::Unknown;
    };
    if branch_id.is_empty() {
        // No diff of its own to land: the branch is at (or behind) the merge base.
        return MergeState::Merged;
    }
    let Some(upstream) = upstream_patch_ids(repo, &base) else {
        return MergeState::Unknown;
    };
    if upstream.contains(&branch_id) {
        MergeState::Merged
    } else {
        MergeState::Unmerged
    }
}

/// The patch-id of everything `branch` changes relative to `base`; empty when it changes nothing.
///
/// A squash merge reproduces the branch's WHOLE diff as one commit, which is why the cumulative id is
/// the right instrument and the per-commit id is not.
fn cumulative_patch_id(repo: &Path, base: &str, branch: &str) -> Option<String> {
    let diff = git(repo, &["diff", base, branch])?;
    if diff.trim().is_empty() {
        return Some(String::new());
    }
    patch_ids(repo, &diff)?.into_iter().next()
}

/// The patch-ids of the commits `origin/main` gained since `base`.
///
/// `--no-merges` keeps the stream small; `git patch-id` ignores merge commits anyway.
fn upstream_patch_ids(repo: &Path, base: &str) -> Option<Vec<String>> {
    let range = format!("{base}..origin/main");
    let log = git(repo, &["log", "--no-merges", "-p", &range])?;
    if log.trim().is_empty() {
        return Some(Vec::new());
    }
    patch_ids(repo, &log)
}

/// `git patch-id --stable` over a diff (or a `git log -p` stream): the ids that identify the same change
/// after a rebase, a cherry-pick or a squash.
fn patch_ids(repo: &Path, text: &str) -> Option<Vec<String>> {
    use std::io::Write as _;
    let mut child = Command::new("git")
        .args(["patch-id", "--stable"])
        .current_dir(repo)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.as_mut()?.write_all(text.as_bytes()).ok()?;
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| line.split_whitespace().next().map(str::to_string))
            .collect(),
    )
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
        if let Some(why) = protected_refusal(repo, &s.branch) {
            refused.push((s.branch.clone(), why));
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

    /// **The defect this replaced.** A squash merge lands a multi-commit branch as ONE commit, so every
    /// one of the branch's own commits is a `+` line to `git cherry` - which is why `clean-merged`
    /// called merged work "not merged - its owner decides" forever, kept the branch, and (once the
    /// claims release landed) never released its claim. The cumulative patch-id matches, so the proof
    /// existed all along; the instrument was wrong. Kimi PM's ruling, #2028 comment 6197423733.
    #[test]
    fn a_multi_commit_branch_that_was_squash_merged_is_merged() {
        let f = Fixture::new();
        f.run(&["checkout", "-qb", "multi"]);
        std::fs::write(f.repo().join("a"), "1").expect("write");
        f.run(&["add", "a"]);
        f.run(&["commit", "-qm", "a"]);
        std::fs::write(f.repo().join("b"), "2").expect("write");
        f.run(&["add", "b"]);
        f.run(&["commit", "-qm", "b"]);
        // The fast lane is exactly what fails here: two commits, two `+` lines.
        let cherry = f.run(&["cherry", "origin/main", "multi"]);
        assert_eq!(cherry.lines().count(), 2, "{cherry}");
        assert!(
            cherry.lines().all(|l| l.starts_with('+')),
            "the fast lane must report unmerged here: {cherry}"
        );

        f.run(&["checkout", "-q", "main"]);
        f.run(&["merge", "-q", "--squash", "multi"]);
        f.run(&["commit", "-qm", "squash multi"]);
        f.run(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

        assert_eq!(
            merge_state(f.repo(), "multi"),
            MergeState::Merged,
            "the cumulative patch-id proves what no per-commit id could"
        );
        let s = classify(f.repo(), "multi", &[]);
        assert!(s.removable, "{}", s.reason);
    }

    /// **The case that must never regress: work main does NOT have still refuses.** This one reaches the
    /// new path - a `+` line, then a cumulative id that matches nothing upstream - so it pins the
    /// direction of the change, not just the happy path.
    #[test]
    fn a_genuinely_unmerged_branch_is_still_not_removable() {
        let f = Fixture::new();
        f.run(&["checkout", "-qb", "wip"]);
        std::fs::write(f.repo().join("w"), "x").expect("write");
        f.run(&["add", "w"]);
        f.run(&["commit", "-qm", "work"]);
        f.run(&["checkout", "-q", "main"]);
        // Main gains something else entirely, so no upstream cumulative id can match the branch.
        std::fs::write(f.repo().join("other"), "y").expect("write");
        f.run(&["add", "other"]);
        f.run(&["commit", "-qm", "other"]);
        f.run(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

        assert_eq!(merge_state(f.repo(), "wip"), MergeState::Unmerged);
        let s = classify(f.repo(), "wip", &[]);
        assert!(!s.removable, "{}", s.reason);
        assert!(s.reason.contains("not merged"), "{}", s.reason);
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
        // **The same atomic write as the claims commands.** This was the one committed-store write left
        // on plain `fs::write`, which made the harness README false: it says the committed store is
        // written by a temporary file plus a rename. The claim is the caller's own, restored in the
        // caller's own checkout, and a truncate here could still be read as "no claims" by anything
        // watching the file. Kimi PM's #2514 review F1.
        if crate::team::write_atomic(&claims, &text).is_ok() {
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

    // ---- 0. A protected branch is never deleted, locally or remotely. --------------------------
    if let Some(why) = protected_refusal(&paths.repo, branch) {
        result["reason"] = serde_json::json!(why);
        return result;
    }

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
    // R10: this writes `pre-commit` into the repository, so it is a checkout write. A lab running it
    // from its OWN worktree still installs into the shared hooks directory - that is how every
    // worktree is guarded - but the shared checkout itself is not a place a lab mutates.
    if let Err(e) = crate::state::refuse_shared_checkout(paths, "the git hooks") {
        err.push(e);
        return (2, out, err);
    }
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

// ---------------------------------------------------------------------------------------------
// `git status` - the worktree half, ported whole.
// ---------------------------------------------------------------------------------------------

/// What `git status --porcelain` says about the tree, in `_worktree_facts`'s shape.
#[derive(Debug, Clone, PartialEq)]
pub struct WorktreeFacts {
    /// **False means the question could not be asked** — which is not the same as a clean tree.
    pub available: bool,
    pub reason: String,
    /// **`None` when unavailable, not `false`.** A tree that cannot be read is not a dirty one, and
    /// `Option` is what keeps a caller from printing "dirty" for a failed `git status`.
    pub clean: Option<bool>,
    pub lines: Vec<String>,
    pub total: usize,
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
}

/// `githelp._worktree_facts` — the working tree's counts, from porcelain output.
///
/// # The three counts, and why each column is read the way it is
///
/// Porcelain v1 is two columns then a space: **`XY path`**. The port keeps Python's exact rules,
/// including the ones that look like they could be simpler:
///
/// * **staged** — `l[:1] not in (" ", "?", "!")`: **the FIRST column is the index.** A `?` there means
///   untracked, and `!` means ignored, **so neither is a staged change.** Reading the first column
///   without those two exclusions would count every untracked file as staged.
/// * **unstaged** — `l[1:2] not in (" ", "?")`: **the SECOND column is the worktree.** **`!` is NOT
///   excluded here**, which is a real asymmetry with the first column — **an ignored file cannot be
///   staged, but the worktree column of an ignored entry is not a modification either** — and it is
///   copied as-is rather than corrected, because `check` and the delivered reports quote these counts
///   and **"fixing" it would change numbers labs have already read.**
/// * **untracked** — `startswith("??")`, which is both columns at once.
///
/// **A count can therefore exceed the total's meaning**: a file staged *and* modified appears in both
/// `staged` and `unstaged`, so the three do not sum to `total`. **That is Python's behaviour and the
/// port keeps it** — the counts answer "how many files are in this state", not "how many files are
/// there".
///
/// **`lines` is capped at 20 and each is truncated to 120**, so a lab with a thousand changes gets a
/// readable report rather than a wall — **and `total` still says how many there really were.**
pub fn worktree_facts(repo: &Path) -> WorktreeFacts {
    let (rc, out) = git_raw(repo, &["status", "--porcelain"]);
    if rc != 0 {
        return WorktreeFacts {
            available: false,
            reason: if out.trim().is_empty() {
                "`git status --porcelain` failed".to_string()
            } else {
                out.trim().to_string()
            },
            clean: None,
            lines: Vec::new(),
            total: 0,
            staged: 0,
            unstaged: 0,
            untracked: 0,
        };
    }
    let all: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    let first_col = |l: &str| l.chars().next().unwrap_or(' ');
    let second_col = |l: &str| l.chars().nth(1).unwrap_or(' ');
    let staged = all
        .iter()
        .filter(|l| !matches!(first_col(l), ' ' | '?' | '!'))
        .count();
    let unstaged = all
        .iter()
        .filter(|l| l.chars().count() > 1 && !matches!(second_col(l), ' ' | '?'))
        .count();
    let untracked = all.iter().filter(|l| l.starts_with("??")).count();
    WorktreeFacts {
        available: true,
        reason: String::new(),
        clean: Some(all.is_empty()),
        lines: all
            .iter()
            .take(20)
            .map(|l| crate::prbody::truncate(l, 120))
            .collect(),
        total: all.len(),
        staged,
        unstaged,
        untracked,
    }
}

#[cfg(test)]
mod worktree_facts_tests {
    use super::*;

    /// A repo whose `git status --porcelain` covers every column: staged-add, unstaged-modify,
    /// staged-delete, and untracked.
    fn mixed_repo() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().to_path_buf();
        let git = |args: &[&str]| {
            let _ = std::process::Command::new("git")
                .current_dir(&repo)
                .args(args)
                .output();
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["config", "user.email", "t@t"]);
        git(&["config", "user.name", "t"]);
        std::fs::write(repo.join("a.txt"), "a\n").expect("write");
        std::fs::write(repo.join("b.txt"), "b\n").expect("write");
        git(&["add", "-A"]);
        git(&["commit", "-qm", "init"]);
        git(&["rm", "-q", "--cached", "b.txt"]);
        std::fs::write(repo.join("a.txt"), "a\nchanged\n").expect("write");
        std::fs::write(repo.join("untracked.txt"), "new\n").expect("write");
        std::fs::write(repo.join(".gitignore"), "ignored.txt\n").expect("write");
        git(&["add", ".gitignore"]);
        (dir, repo)
    }

    /// **The counts match Python's on the same tree** — `{'total': 5, 'staged': 2, 'unstaged': 1,
    /// 'untracked': 2}` for the porcelain lines `A .gitignore`, ` M a.txt`, `D  b.txt`, `?? b.txt`,
    /// `?? untracked.txt`.
    ///
    /// **The three counts do not sum to `total`**, and the fixture shows why: `b.txt` is both a staged
    /// delete and an untracked file, so it is counted in two columns. **That is Python's behaviour and
    /// the port keeps it** — the counts answer *"how many files are in this state"*, not *"how many
    /// files are there"*.
    #[test]
    fn the_counts_match_python_on_a_mixed_tree() {
        let (_d, repo) = mixed_repo();
        let f = worktree_facts(&repo);
        assert!(f.available, "{}", f.reason);
        assert_eq!(f.clean, Some(false));
        assert_eq!(f.total, 5, "five porcelain lines");
        assert_eq!(f.staged, 2, "A .gitignore and the staged delete of b.txt");
        assert_eq!(f.unstaged, 1, "a.txt only");
        assert_eq!(f.untracked, 2, "b.txt and untracked.txt");
        assert_eq!(
            f.lines.len(),
            5,
            "all five are listed when there are fewer than 20"
        );
    }

    /// **A clean tree is `clean: Some(true)` with zero counts**, so a caller can tell it from a tree
    /// that could not be read.
    #[test]
    fn a_clean_tree_says_so() {
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
        std::fs::write(repo.join("a.txt"), "a\n").expect("write");
        let _ = std::process::Command::new("git")
            .current_dir(&repo)
            .args(["add", "-A"])
            .output();
        let _ = std::process::Command::new("git")
            .current_dir(&repo)
            .args(["commit", "-qm", "x"])
            .output();
        let f = worktree_facts(&repo);
        assert_eq!(f.clean, Some(true), "{f:?}");
        assert_eq!((f.total, f.staged, f.unstaged, f.untracked), (0, 0, 0, 0));
        assert!(f.lines.is_empty());
    }

    /// **A repo that cannot be read is `available: false` with `clean: None`** — **not `clean: false`,
    /// which would report a broken checkout as a dirty one.** The `Option` is what makes that
    /// distinction representable, and this test is why it is not a `bool`.
    #[test]
    fn an_unreadable_repo_is_unavailable_rather_than_dirty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let not_a_repo = dir.path().to_path_buf();
        let f = worktree_facts(&not_a_repo);
        assert!(!f.available, "a bare directory is not a repository: {f:?}");
        assert_eq!(f.clean, None, "and it is NOT reported as dirty");
        assert!(!f.reason.is_empty(), "the reason is carried");
        assert_eq!((f.total, f.staged, f.unstaged, f.untracked), (0, 0, 0, 0));
    }

    /// **`lines` is capped at 20 and each line is truncated to 120 characters.** A lab with a thousand
    /// changes gets a readable report **and `total` still says how many there really were** — which is
    /// the part that matters, because **a cap without the true count is a report that lies by
    /// omission.**
    #[test]
    fn the_line_list_is_capped_but_the_total_is_not() {
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
        // A long filename, to exercise the 120-character truncation.
        let long = "a".repeat(200);
        for i in 0..25 {
            std::fs::write(repo.join(format!("{long}-{i}.txt")), "x\n").expect("write");
        }
        let f = worktree_facts(&repo);
        assert_eq!(f.total, 25, "the true count survives the cap");
        assert_eq!(f.lines.len(), 20, "the list is capped at 20");
        assert!(
            f.lines.iter().all(|l| l.chars().count() <= 120),
            "each line is truncated: {:?}",
            f.lines.iter().map(|l| l.chars().count()).max()
        );
    }
}

// ---------------------------------------------------------------------------------------------
// `git status` - the PR facts, the delivery state, and the next command.
// ---------------------------------------------------------------------------------------------

/// `githelp._pr_facts` — the current branch's PR, and whether its merge is on `origin/main`.
///
/// # The containment check, and which commit it checks
///
/// **The merge commit is preferred; the PR HEAD is the fallback.** That order matters: **after a
/// squash merge the PR HEAD is not an ancestor of main**, so checking it first would report every
/// squash-merged PR as uncontained — **and squash is the merge this project uses.**
///
/// **`contained_in_origin_main` is `None` when it could not be checked**, and the three states are
/// kept distinct all the way to `_delivery_state`: *contained*, *not contained*, and *could not tell*.
/// **Collapsing the third into either of the others is how a lab is told to delete a branch whose
/// merge was never verified.**
pub fn pr_facts(repo: &Path, branch: &str, fresh_main: Option<&str>) -> serde_json::Value {
    let fields = "number,title,state,url,mergeCommit,headRefName,headRefOid,baseRefName,isDraft";
    let (ok, data, err) = crate::githubcards::gh_json(&["pr", "view", "--json", fields], repo);
    if !ok || !data.is_object() {
        return serde_json::json!({
            "available": false,
            "reason": if err.is_empty() { "`gh pr view` returned no pull request".to_string() } else { err },
            "number": serde_json::Value::Null,
            "state": "",
            "contained_in_origin_main": serde_json::Value::Null,
            "checked_ref": "",
        });
    }
    let sha = data
        .get("mergeCommit")
        .and_then(|m| m.get("oid"))
        .and_then(|o| o.as_str())
        .unwrap_or("")
        .to_string();
    let head_oid = data
        .get("headRefOid")
        .and_then(|o| o.as_str())
        .unwrap_or("");
    // **Merge commit first, PR head second** - see the note above.
    let (checked_ref, checked_sha) = if !sha.is_empty() {
        ("merge commit", sha.clone())
    } else if !head_oid.is_empty() {
        ("PR head", head_oid.to_string())
    } else {
        ("", String::new())
    };
    let mut contained = serde_json::Value::Null;
    // **Both conditions must hold before asking git**, and clippy is right to want them joined: a
    // containment answer needs both a commit to look for and a ref to look in.
    if !checked_sha.is_empty() && fresh_main.is_some() {
        let (rc, _) = git_raw(
            repo,
            &["merge-base", "--is-ancestor", &checked_sha, "origin/main"],
        );
        contained = serde_json::json!(rc == 0);
    }
    let s = |k: &str| {
        data.get(k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    // **`headRefName` falls back to the branch we asked about**, so a PR whose field is missing still
    // names the branch rather than the empty string.
    let head_ref = {
        let h = s("headRefName");
        if h.is_empty() {
            branch.to_string()
        } else {
            h
        }
    };
    serde_json::json!({
        "available": true,
        "reason": "",
        "number": data.get("number").cloned().unwrap_or(serde_json::Value::Null),
        "title": s("title"),
        "state": s("state"),
        "is_draft": data.get("isDraft").and_then(|v| v.as_bool()).unwrap_or(false),
        "url": s("url"),
        "branch": head_ref,
        "base": s("baseRefName"),
        "merge_commit": sha,
        "checked_ref": checked_ref,
        "checked_sha": checked_sha,
        "contained_in_origin_main": contained,
    })
}

/// `githelp._delivery_state` — the one word that says where this branch is.
///
/// # The order is the logic
///
/// **A dirty worktree outranks everything.** A lab with uncommitted changes has a next action that
/// does not depend on any PR, **so asking GitHub first would produce a correct answer to the wrong
/// question** — and Python checks it first for that reason.
///
/// Then: no PR → `no-pr`; merged → one of **three** states depending on whether the merge commit is
/// contained; closed → `pr-closed`; open → `pr-open`; anything else → `unknown`.
///
/// **`merged-unknown` is a real state and not a bug**: `contained_in_origin_main` is `None` when
/// `origin/main` could not be resolved, **and a lab must not be told to delete a branch whose merge
/// was never verified** — which is what the state name is for.
pub fn delivery_state(facts: &WorktreeFacts, pr: &serde_json::Value) -> String {
    if facts.available && facts.clean == Some(false) {
        return "dirty-worktree".into();
    }
    if pr.get("available").and_then(|v| v.as_bool()) != Some(true) {
        return "no-pr".into();
    }
    let state = pr
        .get("state")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_uppercase();
    if state == "MERGED" {
        return match pr.get("contained_in_origin_main").and_then(|v| v.as_bool()) {
            Some(true) => "merged-verified".into(),
            Some(false) => "merged-unverified".into(),
            None => "merged-unknown".into(),
        };
    }
    if state == "CLOSED" {
        return "pr-closed".into();
    }
    if state == "OPEN" {
        return "pr-open".into();
    }
    "unknown".into()
}

/// `githelp._next_command` — the exact next delivery command, plus the rest of the chain.
///
/// **This is the part of `git status` with judgement in it**: it is what a lab reads and then does.
/// **Two of its returns tell a lab NOT to act** — `merged-unverified` says *"do not delete anything
/// yet"*, and the unrecognised state says *"report this output rather than acting on it"* — **and
/// those are the returns a port is most tempted to simplify into a command.**
///
/// **`merged-verified` and `merged-unknown` share a branch**, because both mean the work is on main;
/// **`merged-unverified` does not**, because it means the opposite was observed.
pub fn next_command(
    branch: &str,
    state: &str,
    pr: &serde_json::Value,
    worktree: &str,
) -> (String, Vec<String>) {
    let number = pr.get("number").and_then(|n| n.as_i64());
    match state {
        "dirty-worktree" => (
            "git add <the paths you changed> && git commit -m \"<message>\"".to_string(),
            vec![
                "AGENTS.md: stage named paths, not `git add -A`.".to_string(),
                "then follow the state below.".to_string(),
            ],
        ),
        "no-pr" => (
            format!("git push -u origin {branch}"),
            vec![
                "gh pr create --fill --base main".to_string(),
                "then wait for the required check and merge through the queue.".to_string(),
            ],
        ),
        "pr-open" => {
            let checks = match number {
                Some(n) => format!("gh pr checks {n} --watch"),
                None => "gh pr checks --watch".to_string(),
            };
            let merge = match number {
                Some(n) => format!("gh pr merge {n} --squash --auto"),
                None => "gh pr merge --squash --auto".to_string(),
            };
            (
                checks,
                vec![merge, "then verify the merge commit on fresh origin/main.".to_string()],
            )
        }
        "pr-closed" => {
            let view = match number {
                Some(n) => format!("gh pr view {n} --json state,closedAt,url"),
                None => "gh pr view --json state,closedAt,url".to_string(),
            };
            (
                view,
                vec![
                    "the PR was closed without merging: reopen it, or preserve the source as a patch + INDEX row in docs/history/branch-archive/.".to_string(),
                    "do not delete the branch until that is decided.".to_string(),
                ],
            )
        }
        "merged-verified" | "merged-unknown" => {
            let branch_delete = if branch.is_empty() {
                "git branch -D <branch>".to_string()
            } else {
                format!("git branch -D {branch}")
            };
            let remote_delete = if branch.is_empty() {
                "git push origin --delete <branch>".to_string()
            } else {
                format!("git push origin --delete {branch}")
            };
            (
                format!(
                    "git worktree remove {worktree} && {branch_delete} && {remote_delete} && scripts/storage/uor-hygiene --apply"
                ),
                vec!["run it from another checkout (this command removes the worktree it is run in).".to_string()],
            )
        }
        "merged-unverified" => (
            "git fetch --prune origin && git log -1 --format='%H %s' origin/main".to_string(),
            vec![
                "the PR is merged but its merge commit is NOT contained in the fetched origin/main.".to_string(),
                "keep the branch and worktree until that is explained; do not delete anything yet.".to_string(),
            ],
        ),
        _ => (
            "git status".to_string(),
            vec!["state not recognised; report this output rather than acting on it.".to_string()],
        ),
    }
}

/// `githelp.status` — the delivery state of the current branch, as JSON.
///
/// **Returns the JSON and the state**, so the caller decides the exit code: **`merged-unverified`
/// exits 1**, because it is the one state that means *"the PR is merged and its commit is NOT on the
/// origin/main I can see"* — **a lab that treats that as success deletes a branch whose work it never
/// found.**
///
/// **The fetch is attempted and its failure is REPORTED, not fatal.** *"origin/main may be stale: the
/// fetch failed, so containment uses the last fetched ref"* — **and a containment answer computed
/// against a stale ref is exactly when `merged-unknown` and `merged-unverified` matter most.**
pub fn git_status(repo: &Path) -> (serde_json::Value, String) {
    let rev = |args: &[&str]| {
        let (rc, out) = git_raw(repo, args);
        if rc == 0 {
            out.trim().to_string()
        } else {
            String::new()
        }
    };
    let branch = rev(&["rev-parse", "--abbrev-ref", "HEAD"]);
    let head = rev(&["rev-parse", "--short", "HEAD"]);
    let worktree = {
        let w = rev(&["rev-parse", "--show-toplevel"]);
        if w.is_empty() {
            repo.display().to_string()
        } else {
            w
        }
    };
    let facts = worktree_facts(repo);

    let (fetch_ok, fetch_out, fetch_cmd) = git_run(repo, &["fetch", "--prune", "origin"]);
    let fresh_main_raw = rev(&["rev-parse", "origin/main"]);
    let fresh_main = if fresh_main_raw.is_empty() {
        None
    } else {
        Some(fresh_main_raw.clone())
    };
    let (_main_ok, main_out, main_cmd) = git_run(repo, &["rev-parse", "origin/main"]);
    let subject = if fresh_main.is_some() {
        crate::prbody::truncate(&rev(&["log", "-1", "--format=%h %s", "origin/main"]), 100)
    } else {
        String::new()
    };

    let upstream = rev(&[
        "rev-parse",
        "--abbrev-ref",
        "--symbolic-full-name",
        "@{upstream}",
    ]);
    let mut ahead: Option<i64> = None;
    let mut behind: Option<i64> = None;
    if fresh_main.is_some() {
        let (rc, lr) = git_raw(
            repo,
            &["rev-list", "--left-right", "--count", "origin/main...HEAD"],
        );
        if rc == 0 {
            let parts: Vec<&str> = lr.split_whitespace().collect();
            if parts.len() == 2 {
                behind = parts[0].parse().ok();
                ahead = parts[1].parse().ok();
            }
        }
    }

    let pr = pr_facts(repo, &branch, fresh_main.as_deref());
    let state = delivery_state(&facts, &pr);
    let (next_command, next_steps) = next_command(&branch, &state, &pr, &worktree);

    let mut notes: Vec<String> = Vec::new();
    if fetch_ok != 0 {
        notes.push(
            "origin/main may be stale: the fetch failed, so containment uses the last fetched ref"
                .to_string(),
        );
    }

    let data = serde_json::json!({
        "branch": branch,
        "head": head,
        "worktree": worktree,
        "clean": facts.clean,
        "changes": {
            "total": facts.total,
            "staged": facts.staged,
            "unstaged": facts.unstaged,
            "untracked": facts.untracked,
        },
        "changes_available": facts.available,
        "changes_reason": facts.reason,
        "fetch": { "command": "git fetch --prune origin", "ok": fetch_ok == 0, "reason": if fetch_ok == 0 { String::new() } else { run_failure(&fetch_cmd, fetch_ok, &fetch_out) } },
        "origin_main": {
            "available": fresh_main.is_some(),
            "reason": if fresh_main.is_none() { run_failure(&main_cmd, _main_ok, &main_out) } else { String::new() },
            "commit": fresh_main.unwrap_or_default(),
            "subject": subject,
        },
        "upstream": upstream,
        "ahead": ahead,
        "behind": behind,
        "pr": pr,
        "state": state,
        "next_command": next_command,
        "next_steps": next_steps,
        "notes": notes,
    });
    (data, state)
}

/// Python's `_run` failure text, character for character.
///
/// # The four things it does, and the first version did three of them differently
///
/// ```python
/// err = redact(((proc.stderr or "") + (proc.stdout or "")).strip().replace("\n", " "))[:240]
/// return False, stdout, f"`{cmd}` exited {rc}: {err or 'no output'}"
/// ```
///
/// * **STDERR FIRST, then stdout.** The obvious reading is the reverse, and the reverse is wrong —
///   for `git rev-parse origin/main` on an ambiguous ref, stdout holds the ref name and stderr holds
///   the explanation, **so the wrong order puts `origin/main` in front of the sentence explaining it.**
/// * **Strip the COMBINED text, then replace every newline with a space.** That turns a blank line
///   into two spaces, **which is why the real fetch failure reads `…remote repository.  Please make
///   sure…` with two spaces in the middle.** Joining non-empty lines with single spaces — what the
///   first version did — loses exactly that.
/// * **Truncate to 240 characters**, so a chatty failure cannot flood a report.
/// * **`redact`**, because a command that echoes a URL with a token in it would otherwise write it
///   into a JSON file that gets committed.
///
/// **`git status --json` agreed on twenty-seven fields and differed on three `reason` strings**, which
/// is how each of these was found: **the structure was right and the rendering was not.**
pub fn run_failure(command: &str, exit: i32, output: &str) -> String {
    let collapsed = crate::prgates::redact_public(output.trim()).replace('\n', " ");
    let err: String = collapsed.chars().take(240).collect();
    let shown = if err.is_empty() {
        "no output".to_string()
    } else {
        err
    };
    format!("`{command}` exited {exit}: {shown}")
}

/// `git -C <repo> <args>`, with stderr and stdout kept **separate and in Python's order**.
///
/// **`git_raw` returns them concatenated with stdout first**, which is right for reading an answer and
/// **wrong for rendering a failure**: Python's `_run` builds `stderr + stdout`. `git rev-parse
/// origin/main` on an ambiguous ref writes the ref to stdout and the explanation to stderr, **so the
/// wrong order prints `origin/main` in front of the sentence that says why it is ambiguous.**
///
/// **This is a second function rather than a change to `git_raw`** because a dozen callers already
/// depend on that one's shape, **and a caller that reads an answer does not care about the order.**
pub fn git_run(repo: &Path, args: &[&str]) -> (i32, String, String) {
    match std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
    {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            (
                o.status.code().unwrap_or(1),
                format!("{stderr}{stdout}"),
                format!("git {}", args.join(" ")),
            )
        }
        Err(e) => (1, format!("{e}"), format!("git {}", args.join(" "))),
    }
}

#[cfg(test)]
mod status_tests {
    use super::*;

    /// **The delivery state table, asserted on its own** — it is nine lines of branches, and a port
    /// that got the order wrong would still compile and still print a plausible word.
    ///
    /// **The order is the logic**: a dirty worktree outranks the PR, **because a lab with uncommitted
    /// changes has a next action that does not depend on any PR** — asking GitHub first answers the
    /// wrong question.
    #[test]
    fn the_delivery_state_table_matches_python() {
        let clean = WorktreeFacts {
            available: true,
            reason: String::new(),
            clean: Some(true),
            lines: vec![],
            total: 0,
            staged: 0,
            unstaged: 0,
            untracked: 0,
        };
        let dirty = WorktreeFacts {
            clean: Some(false),
            ..clean.clone()
        };
        let unknowable = WorktreeFacts {
            available: false,
            clean: None,
            ..clean.clone()
        };
        let pr = |state: &str, contained: serde_json::Value| serde_json::json!({ "available": true, "state": state, "contained_in_origin_main": contained });
        // **A dirty worktree wins even with a merged PR.**
        assert_eq!(
            delivery_state(&dirty, &pr("MERGED", serde_json::json!(true))),
            "dirty-worktree"
        );
        // **A tree that could not be read is NOT dirty**, so the PR decides.
        assert_eq!(
            delivery_state(&unknowable, &pr("OPEN", serde_json::Value::Null)),
            "pr-open"
        );
        assert_eq!(
            delivery_state(&clean, &serde_json::json!({"available": false})),
            "no-pr"
        );
        assert_eq!(
            delivery_state(&clean, &pr("MERGED", serde_json::json!(true))),
            "merged-verified"
        );
        assert_eq!(
            delivery_state(&clean, &pr("MERGED", serde_json::json!(false))),
            "merged-unverified"
        );
        // **The third merged state is real**: `origin/main` could not be resolved.
        assert_eq!(
            delivery_state(&clean, &pr("MERGED", serde_json::Value::Null)),
            "merged-unknown"
        );
        assert_eq!(
            delivery_state(&clean, &pr("closed", serde_json::Value::Null)),
            "pr-closed"
        );
        assert_eq!(
            delivery_state(&clean, &pr("OPEN", serde_json::Value::Null)),
            "pr-open"
        );
        assert_eq!(
            delivery_state(&clean, &pr("WEIRD", serde_json::Value::Null)),
            "unknown"
        );
    }

    /// **`merged-unverified` tells a lab NOT to act, and the port keeps that.**
    ///
    /// It is the one state where `git status` exits 1, **and the temptation in a port is to give it a
    /// command like the other states have.** Python deliberately does not: *"keep the branch and
    /// worktree until that is explained; do not delete anything yet."*
    #[test]
    fn merged_unverified_says_do_not_delete_anything() {
        let (cmd, steps) = next_command("b", "merged-unverified", &serde_json::json!({}), "/w");
        assert!(cmd.contains("git fetch --prune origin"), "{cmd}");
        assert!(
            steps
                .iter()
                .any(|s| s.contains("do not delete anything yet")),
            "{steps:?}"
        );
        // And the unrecognised state says not to act either.
        let (cmd2, steps2) = next_command("b", "something-new", &serde_json::json!({}), "/w");
        assert_eq!(cmd2, "git status");
        assert!(
            steps2
                .iter()
                .any(|s| s.contains("rather than acting on it")),
            "{steps2:?}"
        );
    }

    /// **`merged-verified` and `merged-unknown` share the cleanup command**, because both mean the
    /// work is on main — **only `merged-unverified` means the opposite was observed.**
    #[test]
    fn the_two_merged_states_that_mean_finished_share_a_command() {
        let (a, _) = next_command("br", "merged-verified", &serde_json::json!({}), "/w");
        let (b, _) = next_command("br", "merged-unknown", &serde_json::json!({}), "/w");
        assert_eq!(a, b, "both mean the work is on main");
        assert!(a.contains("git worktree remove /w"), "{a}");
        assert!(a.contains("git branch -D br"), "{a}");
        assert!(a.contains("git push origin --delete br"), "{a}");
        assert!(a.contains("uor-hygiene --apply"), "{a}");
    }

    /// **An empty branch name renders a placeholder rather than an empty command** — `git branch -D `
    /// with nothing after it is not a command a lab can run.
    #[test]
    fn an_unknown_branch_gets_a_placeholder_not_an_empty_command() {
        let (cmd, _) = next_command("", "merged-verified", &serde_json::json!({}), "/w");
        assert!(cmd.contains("git branch -D <branch>"), "{cmd}");
        assert!(cmd.contains("git push origin --delete <branch>"), "{cmd}");
    }

    /// **`run_failure` renders Python's text**, including the three things the first version got
    /// wrong: **stderr before stdout is the caller's job, but the blank line becomes TWO SPACES, and
    /// the whole thing is truncated to 240.**
    #[test]
    fn run_failure_renders_pythons_text() {
        let out = run_failure(
            "git fetch --prune origin",
            128,
            "fatal: no remote\n\nPlease check\n",
        );
        assert_eq!(
            out,
            "`git fetch --prune origin` exited 128: fatal: no remote  Please check"
        );
        // A blank line becomes two spaces - **stripping and replacing, not joining non-empty lines**,
        // which is what the first version did and why the real fetch message differed.
        assert!(
            out.contains("remote  Please"),
            "two spaces where the blank line was: {out:?}"
        );
        // Truncated to 240 characters after the prefix is added.
        let long = "x".repeat(400);
        let t = run_failure("cmd", 1, &long);
        assert!(
            t.ends_with(&"x".repeat(240)),
            "240 characters of output, not 400"
        );
        // Empty output is named rather than left blank.
        assert!(run_failure("cmd", 1, "   ").ends_with("no output"));
    }
}

#[cfg(test)]
mod protected_branch_tests {
    use super::*;

    /// Run git in `repo` with every `GIT_*` variable removed, so a hook's environment cannot point
    /// the command at the repository being committed to.
    fn git_clean(repo: &Path, args: &[&str]) -> String {
        let mut cmd = Command::new("git");
        cmd.current_dir(repo).args(args);
        for (k, _) in std::env::vars_os() {
            if k.to_string_lossy().starts_with("GIT_") {
                cmd.env_remove(&k);
            }
        }
        let out = cmd.output().expect("git runs");
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    /// A temp repo with `main`, `gh-pages` and an ordinary merged branch, all at one commit.
    fn repo_with_protected() -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).expect("mkdir");
        assert!(
            repo.starts_with(dir.path()),
            "the test repo must live in its temp dir"
        );
        git_clean(&repo, &["init", "-q", "-b", "main"]);
        let top = git_clean(&repo, &["rev-parse", "--show-toplevel"]);
        let top = std::fs::canonicalize(top.trim()).expect("toplevel");
        let tmp = std::fs::canonicalize(dir.path()).expect("tmp");
        assert!(
            top.starts_with(&tmp),
            "git resolved {top:?} outside {tmp:?}"
        );
        git_clean(&repo, &["config", "user.email", "t@t"]);
        git_clean(&repo, &["config", "user.name", "t"]);
        std::fs::write(repo.join("README.md"), "x\n").expect("write");
        git_clean(&repo, &["add", "-A"]);
        git_clean(&repo, &["commit", "-qm", "init"]);
        git_clean(&repo, &["branch", "gh-pages"]);
        git_clean(&repo, &["branch", "done-work"]);
        let paths = StatePaths {
            repo,
            live: dir.path().join("live"),
        };
        (dir, paths)
    }

    fn heads(repo: &Path) -> String {
        git_clean(
            repo,
            &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
        )
    }

    /// The registry's names are protected even with no registry file, and a registry adds names.
    #[test]
    fn protection_ignores_case_and_survives_a_missing_registry() {
        let dir = tempfile::tempdir().expect("tempdir");
        // No docs/harness/branches.json in this directory: the built-in names still protect,
        // and a case-insensitive filesystem cannot be used to delete `main` as `Main`.
        assert!(protected_refusal(dir.path(), "Main").is_some());
        assert!(protected_refusal(dir.path(), "refs/heads/GH-PAGES").is_some());
        assert!(protected_refusal(dir.path(), "gh-pages-preview").is_some());
        assert!(protected_refusal(dir.path(), "claude/topic").is_none());
    }

    #[test]
    fn builtin_floor_and_registry_both_protect() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(protected_refusal(dir.path(), "gh-pages-preview").is_some());
        assert!(protected_refusal(dir.path(), "refs/heads/main").is_some());
        assert!(protected_refusal(dir.path(), "claude/topic").is_none());
        std::fs::create_dir_all(dir.path().join("docs/harness")).expect("mkdir");
        std::fs::write(
            dir.path().join("docs/harness/branches.json"),
            r#"{"version":1,"protected":[{"name":"release","purpose":"x"}]}"#,
        )
        .expect("write");
        assert!(protected_refusal(dir.path(), "release").is_some());
        assert!(protected_refusal(dir.path(), "main").is_some());
    }

    /// `clean-merged --apply` refuses a protected branch even when a caller marks it removable.
    #[test]
    fn clean_merged_apply_refuses_a_protected_branch() {
        let (_dir, paths) = repo_with_protected();
        let state = |b: &str| BranchState {
            branch: b.to_string(),
            state: MergeState::Merged,
            checked_out: false,
            removable: true,
            reason: String::new(),
        };
        let (deleted, refused) = apply(&paths.repo, &[state("gh-pages"), state("done-work")]);
        assert_eq!(deleted, vec!["done-work".to_string()]);
        assert_eq!(refused.len(), 1, "{refused:?}");
        assert!(refused[0].1.contains("protected branch"), "{refused:?}");
        assert!(heads(&paths.repo).contains("gh-pages"));
    }

    /// `delete-merged --apply` refuses a protected branch before any local or remote delete runs.
    #[test]
    fn delete_merged_refuses_a_protected_branch() {
        let (_dir, paths) = repo_with_protected();
        let r = delete_merged_branch(&paths, "gh-pages", true);
        assert_eq!(r["safe_to_delete"], serde_json::json!(false), "{r}");
        assert_eq!(r["deleted"], serde_json::json!(false), "{r}");
        assert!(
            r["remote_deleted"].is_null(),
            "no remote delete was attempted: {r}"
        );
        assert!(
            r["reason"]
                .as_str()
                .unwrap_or("")
                .contains("protected branch"),
            "{r}"
        );
        assert!(heads(&paths.repo).contains("gh-pages"));
    }
}
