//! The `status` rules, as Rust (D30 check-port step 4 of 10).
//!
//! # The four `status` rules, and which are here
//!
//! | rule | severity | in this unit |
//! |---|---|---|
//! | `status.branch_claimed` | warn | **yes** |
//! | `status.no_orphan_branches` | warn | **yes** (local half) |
//! | `status.heartbeat_receipts` | warn | no — needs the GitHub status cards |
//! | `status.registry_vs_github` | warn | no — needs the GitHub status cards |
//!
//! ## Why the two GitHub-dependent rules are not here
//!
//! `heartbeat_receipts` ("every in-progress milestone posts a status card") and
//! `registry_vs_github` ("the registry headline agrees with the newest status card") both read **status
//! cards posted as issue comments**. They are the only two rules in `check` whose subject is **the
//! GitHub conversation** rather than the checkout, and porting them means porting `gh` pagination,
//! comment parsing and the headline grammar together.
//!
//! **That is a real unit of work, not a line.** Riding it along here would produce two rules that
//! *look* ported and cannot be verified, which is the failure this whole sequencing exists to avoid —
//! so they stay delegated, named, and next.
//!
//! ## The two that ARE here are the two about the checkout
//!
//! `branch_claimed` and `no_orphan_branches` are both about **what this working copy is holding**, which
//! is exactly what the port can verify offline. They also cover the two ways a checkout accumulates
//! state the project says it should not: an **unclaimed branch** (work nobody can be asked about) and an
//! **orphan branch** (a merged branch that was never deleted).

use crate::githelp;
use crate::state::StatePaths;
use crate::team;

pub use crate::preprules::Finding;

fn warn(rule_id: &'static str, message: String, path: String, fix: String) -> Finding {
    Finding {
        rule_id,
        severity: "warn",
        message,
        path,
        fix,
    }
}

/// `status.branch_claimed` — the checked-out branch is claimed by a lab, under the one-checkout policy.
///
/// **Claims are how a branch becomes answerable.** A branch checked out with no claim is work that no
/// lab can be asked about: another lab cannot tell whether it is active, abandoned, or someone's
/// scratch, and the board will not show it. `main` and `master` are exempt because the shared checkout
/// is not work — it is the base every branch comes from.
pub fn branch_claimed(paths: &StatePaths) -> Vec<Finding> {
    let branch = team::current_branch(&paths.repo);
    if branch.is_empty() || branch == "main" || branch == "master" {
        return Vec::new();
    }
    if team::load_claims(paths).contains_key(&branch) {
        return Vec::new();
    }
    vec![warn(
        "status.branch_claimed",
        format!("branch {branch:?} is checked out but not claimed by any lab"),
        "docs/harness/claims.json".into(),
        format!("uor-harness team claim --branch {branch} --lab <lab>"),
    )]
}

/// `status.no_orphan_branches` — no local branch whose work is already in `main`.
///
/// **A merged branch is not a research authority; `main` is.** The project's rule is to delete the
/// branch on both sides after a verified merge, and the reason is state, not tidiness: a branch that
/// still exists reads as work in progress, so a lab checking `git branch` sees twelve branches and
/// cannot tell which two matter.
///
/// **Proof is by content, not ancestry.** A squash merge leaves no ancestry link, so
/// `merge-base --is-ancestor` would call a merged branch *unmerged* and this rule would never fire on
/// the merges the project actually performs. `githelp::merge_state` does the content comparison, and
/// this rule reports only `Merged` — an `Unknown` branch is left alone, because **a rule that guesses
/// "merged" tells a lab to delete work that may be live.**
pub fn no_orphan_branches(paths: &StatePaths) -> Vec<Finding> {
    let mut out = Vec::new();
    for b in githelp::branch_states(&paths.repo) {
        if b.state != githelp::MergeState::Merged {
            continue;
        }
        // A branch that is checked out somewhere is not an orphan: it is in use, and deleting it
        // would fail anyway. `githelp` already computes this, so the rule uses the same flag rather
        // than a second opinion about what "in use" means.
        if b.checked_out {
            continue;
        }
        out.push(warn(
            "status.no_orphan_branches",
            format!(
                "local branch {:?} whose work is already in origin/main",
                b.branch
            ),
            "docs/harness/claims.json".into(),
            format!(
                "uor-harness git clean-merged    # proves it, then --apply deletes it; \
                 then: git push origin --delete {}",
                b.branch
            ),
        ));
    }
    out
}

/// Every `status` rule ported in this unit.
pub fn all(paths: &StatePaths) -> Vec<Finding> {
    let mut out = branch_claimed(paths);
    out.extend(no_orphan_branches(paths));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// A real git repo with one commit on `main`, so the git calls answer honestly.
    fn git_repo() -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
        let git = |args: &[&str]| {
            Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .output()
                .expect("git");
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["config", "user.email", "t@t"]);
        git(&["config", "user.name", "t"]);
        std::fs::write(root.join("f.txt"), "x").expect("write");
        git(&["add", "-A"]);
        git(&["commit", "-qm", "init"]);
        let p = StatePaths::resolve(root, None, Some("/nonexistent-live"));
        (dir, p)
    }

    fn git_in(root: &Path, args: &[&str]) {
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .expect("git");
    }

    fn write_claims(root: &Path, body: &str) {
        std::fs::write(root.join("docs/harness/claims.json"), body).expect("write");
    }

    /// `main` is not work: the shared checkout is the base every branch comes from.
    #[test]
    fn main_needs_no_claim() {
        let (_d, p) = git_repo();
        assert!(
            branch_claimed(&p).is_empty(),
            "the base checkout is not a claimable branch"
        );
    }

    /// **A branch checked out with no claim is work nobody can be asked about.**
    #[test]
    fn an_unclaimed_checked_out_branch_is_reported_with_its_name() {
        let (d, p) = git_repo();
        write_claims(d.path(), "{}\n");
        git_in(d.path(), &["checkout", "-q", "-b", "deepseek/some-work"]);
        let f = branch_claimed(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message.contains("deepseek/some-work"),
            "{}",
            f[0].message
        );
        assert!(f[0].fix.contains("team claim"), "{}", f[0].fix);
    }

    #[test]
    fn a_claimed_checked_out_branch_is_not_reported() {
        let (d, p) = git_repo();
        git_in(d.path(), &["checkout", "-q", "-b", "deepseek/claimed"]);
        write_claims(
            d.path(),
            r#"{"deepseek/claimed":{"branch":"deepseek/claimed","lab":"deepseek","head":"","claimed_at":"2026-10-13T12:00:00Z","note":""}}"#,
        );
        assert!(branch_claimed(&p).is_empty());
    }

    /// **A merged branch is proven by CONTENT, not ancestry**, because the project squash-merges and a
    /// squash merge leaves no ancestry link for `merge-base --is-ancestor` to find.
    #[test]
    fn a_branch_with_nothing_ahead_of_main_is_an_orphan() {
        let (d, p) = git_repo();
        // A branch at the same commit as main: zero commits ahead, whatever route put it there.
        git_in(d.path(), &["branch", "merged/by-content"]);
        git_in(
            d.path(),
            &["update-ref", "refs/remotes/origin/main", "HEAD"],
        );
        let f = no_orphan_branches(&p);
        assert!(
            f.iter().any(|x| x.message.contains("merged/by-content")),
            "a branch with nothing ahead of main is an orphan: {f:?}"
        );
    }

    /// A branch with real work ahead is NOT an orphan. **Reporting it would tell a lab to delete live
    /// work**, which is the direction that must never be guessed.
    #[test]
    fn a_branch_with_work_ahead_is_not_an_orphan() {
        let (d, p) = git_repo();
        git_in(d.path(), &["checkout", "-q", "-b", "live/work"]);
        std::fs::write(d.path().join("new.txt"), "y").expect("write");
        git_in(d.path(), &["add", "-A"]);
        git_in(d.path(), &["commit", "-qm", "work"]);
        git_in(
            d.path(),
            &["update-ref", "refs/remotes/origin/main", "main"],
        );
        let f = no_orphan_branches(&p);
        assert!(
            !f.iter().any(|x| x.message.contains("live/work")),
            "unmerged work must not be called an orphan: {f:?}"
        );
    }

    /// The branch a lab is standing on is in use, so it is not an orphan even when merged.
    #[test]
    fn the_checked_out_branch_is_not_reported_as_an_orphan() {
        let (d, p) = git_repo();
        git_in(d.path(), &["checkout", "-q", "-b", "checked/out"]);
        git_in(
            d.path(),
            &["update-ref", "refs/remotes/origin/main", "HEAD"],
        );
        let f = no_orphan_branches(&p);
        assert!(
            !f.iter().any(|x| x.message.contains("checked/out")),
            "a branch that is checked out is in use: {f:?}"
        );
    }

    /// The real repo runs without panicking on real data.
    #[test]
    fn the_real_repo_is_processed() {
        let _ = all(&paths_of_repo());
    }

    fn paths_of_repo() -> StatePaths {
        StatePaths::resolve(repo(), None, Some("/nonexistent-live"))
    }
}
