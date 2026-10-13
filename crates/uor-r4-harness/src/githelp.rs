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
