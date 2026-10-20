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
use crate::registries::Registries;
use crate::state::StatePaths;
use crate::statusdelivery::{self, Github};
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
///
/// **GitHub adds what content cannot see** (Python's three findings, in its order): a local branch
/// whose PR merged, even when main has since moved past its content; a remote branch with work main
/// lacks and no PR; and a worktree left behind at a merged PR head or idle for 24h. A branch reported
/// through its PR is not reported again by content.
///
/// **A check that could not run says so**, as an `info` note naming the reason (Python's "check
/// skipped"): offline, without `gh`, or with lookups cut short by the per-run cap. Without the note
/// the dispatcher would record a pass for a check that never asked. A worktree is still judged
/// offline, by containment in origin/main plus 24h idle.
pub fn no_orphan_branches(paths: &StatePaths, gh: Github, now: i64) -> Vec<Finding> {
    orphan_findings(
        &paths.repo.display().to_string(),
        &statusdelivery::gather_here(paths, gh, now),
        &statusdelivery::remote_branches_without_pr(paths, gh, 20, now),
        &githelp::branch_states(&paths.repo),
    )
}

fn skipped(what: &str, reason: &str, repo: &str) -> Finding {
    Finding {
        rule_id: "status.no_orphan_branches",
        severity: "info",
        message: format!("{what}: {}", statusdelivery::unavailable(reason)),
        path: repo.to_string(),
        fix: "uor-harness status --refresh, with `gh` installed and authenticated".to_string(),
    }
}

/// The rule's decision over gathered facts, so the findings can be tested without git or GitHub.
pub fn orphan_findings(
    repo: &str,
    facts: &statusdelivery::DeliveryExtras,
    remote: &statusdelivery::RemoteBranches,
    states: &[githelp::BranchState],
) -> Vec<Finding> {
    let mut out = Vec::new();
    let repo = repo.to_string();
    if !facts.pr_check.available {
        out.push(skipped(
            "merged-PR branch check skipped",
            &facts.pr_check.reason,
            &repo,
        ));
    } else if !facts.pr_check.reason.is_empty() {
        out.push(skipped(
            "merged-PR branch check incomplete",
            &facts.pr_check.reason,
            &repo,
        ));
    }
    if !remote.available {
        out.push(skipped(
            "remote-branch check skipped",
            &remote.reason,
            &repo,
        ));
    } else if !remote.reason.is_empty() {
        out.push(skipped(
            "remote-branch check incomplete",
            &remote.reason,
            &repo,
        ));
    }
    if !facts.worktrees.available {
        out.push(skipped(
            "worktree check skipped",
            &facts.worktrees.reason,
            &repo,
        ));
    }
    if facts.pr_check.available {
        for o in &facts.merged {
            let moved = if o.head_matches {
                ""
            } else {
                " (its head has moved on since that merge)"
            };
            let place = if o.checked_out_in.is_empty() {
                String::new()
            } else {
                format!("; checked out in {}", o.checked_out_in.join(", "))
            };
            out.push(warn(
                "status.no_orphan_branches",
                format!(
                    "local branch {} still exists but PR #{} merged {}{moved}{place}",
                    o.branch,
                    o.pr,
                    o.merged_at.chars().take(10).collect::<String>()
                ),
                repo.clone(),
                o.fix.clone(),
            ));
        }
    }
    for rb in &remote.branches {
        out.push(warn(
            "status.no_orphan_branches",
            format!(
                "remote branch {} has no PR and is not merged into main ({} commit(s) ahead, {} \
                 behind, last updated {}) - it needs a decision, not deletion",
                rb.branch, rb.ahead, rb.behind, rb.updated
            ),
            repo.clone(),
            format!(
                "archive it (archive/history/branch-archive/ + an INDEX row) or open a PR; do not \
                 delete unmerged work. Local copy: git branch -D {}",
                rb.branch
            ),
        ));
    }
    if facts.worktrees.available {
        for c in facts.worktrees.candidates() {
            let here = if c.current {
                " (this worktree; run the command from another checkout)"
            } else {
                ""
            };
            let why = if c.reapable_reason == "merged-pr-head" {
                "its HEAD is the exact head of a merged PR"
            } else {
                "it holds no commits beyond origin/main and nothing has been written in it for 24h"
            };
            let branch_delete = if c.branch.is_empty() {
                String::new()
            } else {
                format!(" && git branch -D {}", c.branch)
            };
            out.push(warn(
                "status.no_orphan_branches",
                format!(
                    "worktree {} [{}] is clean and completed work was left behind: {why}{here}",
                    c.path,
                    if c.branch.is_empty() {
                        "detached"
                    } else {
                        &c.branch
                    }
                ),
                c.path.clone(),
                format!(
                    "git worktree remove {}{branch_delete} && scripts/storage/uor-hygiene --apply",
                    c.path
                ),
            ));
        }
    }
    for b in states {
        if b.state != githelp::MergeState::Merged {
            continue;
        }
        if facts.merged.iter().any(|m| m.branch == b.branch) {
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

/// The protected names that are absent from `heads`, in the order `protected` lists them.
pub fn missing_protected(heads: &[String], protected: &[String]) -> Vec<String> {
    protected
        .iter()
        .filter(|p| !heads.contains(p))
        .cloned()
        .collect()
}

/// Every branch name on `origin`, or `None` when `ls-remote` could not run or failed (offline).
///
/// Every `GIT_*` variable is removed, so a hook's environment cannot point the call at another
/// repository.
fn origin_heads(repo: &std::path::Path) -> Option<Vec<String>> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["ls-remote", "--heads", "origin"])
        .current_dir(repo);
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(&k);
        }
    }
    // A stalled network or a credential prompt must not hang `check`: no prompts, no stdin,
    // and a bounded wait. Not knowing is reported by the caller as "could not determine".
    cmd.env("GIT_TERMINAL_PROMPT", "0")
        .env(
            "GIT_SSH_COMMAND",
            "ssh -o BatchMode=yes -o ConnectTimeout=10",
        )
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    let mut child = cmd.spawn().ok()?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(50))
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| l.rsplit_once("refs/heads/").map(|(_, b)| b.to_string()))
            .collect(),
    )
}

/// `status.protected_branches_present` — every protected branch still exists on `origin`.
///
/// **An unreachable remote is "could not determine", not a failure**: like `no_orphan_branches`
/// on an `Unknown` branch, the rule emits nothing, so the dispatcher records it as passing.
pub fn protected_branches_present(paths: &StatePaths) -> Vec<Finding> {
    let Some(heads) = origin_heads(&paths.repo) else {
        return Vec::new();
    };
    missing_protected(&heads, &githelp::protected_branches(&paths.repo))
        .into_iter()
        .map(|b| Finding {
            rule_id: "status.protected_branches_present",
            severity: "error",
            message: format!("protected branch {b:?} is missing from origin"),
            path: "docs/harness/branches.json".into(),
            fix: "restore the branch from its last known commit and tell the owner on #2028".into(),
        })
        .collect()
}
/// Every `status` rule ported in this unit.
pub fn all(paths: &StatePaths, regs: &Registries, now: i64, gh: Github) -> Vec<Finding> {
    // **The ledger spans two months**, because a heartbeat at 00:05 on the 1st belongs to the month
    // that just ended as much as to the one that started - `status` reads both, and a rule that read
    // one would report a missing heartbeat that exists in the other file. **Computed here rather than
    // passed in**, so the aggregator's signature stays the shape the dispatcher already calls.
    let month = crate::receipt::month_for(now);
    let previous = crate::status::previous_month(&month);
    let rows = crate::status::receipt_rows(paths, &month, &previous);
    let mut out = branch_claimed(paths);
    out.extend(no_orphan_branches(paths, gh, now));
    out.extend(protected_branches_present(paths));
    // **`heartbeat_receipts` was registered with NO IMPLEMENTATION AT ALL**, and
    // **`registry_vs_github` was ported and never dispatched.** Both reported `ok` on every run
    // because nothing produced a finding. **Found by diffing the dispatcher's output against
    // Python's**, which is the only way this shape shows up: a rule that is absent and a rule that
    // passes look identical in the output.
    out.extend(heartbeat_receipts(regs, &rows, now));
    out.extend(registry_vs_github(paths, regs, now, false));
    out
}

// ---------------------------------------------------------------------------------------------
// The two status rules that were registered and NOT dispatched.
// ---------------------------------------------------------------------------------------------

/// How long an in-progress milestone may go without a heartbeat receipt.
pub const STALE_HOURS: f64 = 24.0;

/// `status._milestone_matches` — does a receipt belong to this milestone?
///
/// **Three spellings, because labs write all three**: the key, the id, and the issue number.
/// **A receipt with no `milestone` matches NOTHING** — it is not a heartbeat for anything, and
/// treating an empty field as a match would let any unattributed receipt satisfy the rule.
fn milestone_matches(receipt: &serde_json::Value, m: &crate::registries::Milestone) -> bool {
    let raw = receipt
        .get("milestone")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_lowercase();
    if raw.is_empty() {
        return false;
    }
    raw == m.key.to_lowercase() || raw == m.id.to_lowercase() || raw == m.issue.to_string()
}

/// `status._rule_heartbeat_receipts` — no heartbeat in 24h for an in-progress milestone.
///
/// **The rule that was registered, counted in the inventory, and had no Rust implementation at all.**
/// The dispatcher reported it as an `ok` record on every run because nothing produced a finding —
/// **a rule that looks like a pass because it does not exist**, which is the failure this port exists
/// to catch and the third instance of it in two units.
///
/// **Only `in-progress` milestones are checked.** A completed or paused milestone with no recent
/// heartbeat is not a problem, and firing on one would train readers to ignore the rule.
pub fn heartbeat_receipts(regs: &Registries, rows: &[serde_json::Value], now: i64) -> Vec<Finding> {
    let mut out = Vec::new();
    for m in regs.milestones.iter() {
        if m.status != "in-progress" {
            continue;
        }
        let last = rows
            .iter()
            .filter(|r| milestone_matches(r, m))
            .filter(|r| r.get("kind").and_then(|k| k.as_str()) == Some("heartbeat"))
            .filter_map(|r| r.get("ts").and_then(|t| t.as_str()))
            .max()
            .unwrap_or("");
        let age = if last.is_empty() {
            None
        } else {
            crate::status::age_hours(&serde_json::Value::String(last.to_string()), now)
        };
        if age.is_some_and(|a| a <= STALE_HOURS) {
            continue;
        }
        let seen = match age {
            Some(a) => format!("last heartbeat {} ago", crate::status::hours_text(Some(a))),
            None => "no heartbeat receipt on record".to_string(),
        };
        out.push(Finding {
            rule_id: "status.heartbeat_receipts",
            severity: "warn",
            message: format!(
                "{} #{} is in-progress and {seen} (docs/labs/session-goal.md 5: a card every 90 minutes or 3 merged PRs)",
                m.id, m.issue
            ),
            path: "docs/harness/milestones.json".into(),
            fix: format!("uor-harness heartbeat --milestone {}", m.key),
        });
    }
    out
}

/// `status.registry_vs_github` — the registry headline against the newest card on the issue.
///
/// **Ported earlier and never dispatched**, so Python reported drift where Rust reported a pass.
/// The decision itself lives in `githubcards`; this is the adapter that gives it the ledger's
/// milestone list and the current instant.
///
/// **`refresh=false`**, because a rule must not make a network call whose result it cannot report:
/// the snapshot may be unavailable, and `registry_vs_github` already has a `warn` for that case which
/// says the registry was **not** cross-checked.
pub fn registry_vs_github(
    paths: &StatePaths,
    regs: &Registries,
    now: i64,
    refresh: bool,
) -> Vec<Finding> {
    crate::githubcards::registry_vs_github(paths, regs.milestones.as_slice(), refresh, now)
        .into_iter()
        .filter(|f| !f.ok)
        .map(|f| Finding {
            rule_id: "status.registry_vs_github",
            severity: "warn",
            message: f.message,
            path: f.path,
            fix: f.fix,
        })
        .collect()
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
        let f = no_orphan_branches(&p, Github::Offline, 0);
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
        let f = no_orphan_branches(&p, Github::Offline, 0);
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
        let f = no_orphan_branches(&p, Github::Offline, 0);
        assert!(
            !f.iter().any(|x| x.message.contains("checked/out")),
            "a branch that is checked out is in use: {f:?}"
        );
    }

    /// The real repo runs without panicking on real data.
    #[test]
    fn the_real_repo_is_processed() {
        let p = paths_of_repo();
        // **The registry is loaded the same way  loads it**, so this test exercises the real
        // path rather than a stub. A stub here would have hidden exactly the wiring defect these
        // three units were about.
        let regs = Registries::load(&p).expect("registry");
        let _ = all(&p, &regs, 1_800_000_000, Github::Offline);
    }

    /// **A protected name absent from the remote heads is reported; present ones are not.**
    #[test]
    fn missing_protected_names_only_the_absent_branches() {
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let protected = s(&["main", "gh-pages", "gh-pages-preview"]);
        assert_eq!(
            missing_protected(&s(&["main", "claude/x", "gh-pages-preview"]), &protected),
            s(&["gh-pages"])
        );
        assert!(
            missing_protected(&s(&["gh-pages-preview", "gh-pages", "main"]), &protected).is_empty()
        );
        assert_eq!(missing_protected(&[], &protected), protected);
    }

    fn paths_of_repo() -> StatePaths {
        StatePaths::resolve(repo(), None, Some("/nonexistent-live"))
    }

    /// **The three GitHub findings carry Python's text**, and a branch reported through its merged
    /// PR is not reported a second time by the content check.
    #[test]
    fn orphan_findings_report_merged_prs_prless_remotes_and_left_worktrees() {
        use crate::statusdelivery::{
            BranchPrCheck, DeliveryExtras, MergedPrBranch, RemoteBranch, RemoteBranches,
            WorktreeItem, WorktreesView,
        };
        let facts = DeliveryExtras {
            pr_check: BranchPrCheck {
                available: true,
                ..BranchPrCheck::default()
            },
            merged: vec![MergedPrBranch {
                branch: "a/done".into(),
                head: "h".into(),
                pr: 12,
                pr_title: "t".into(),
                merged_at: "2026-10-19T08:00:00Z".into(),
                head_matches: false,
                current: false,
                fix: "git branch -D a/done && git push origin --delete a/done".into(),
                checked_out_in: vec!["/w/a".into()],
            }],
            worktrees: WorktreesView {
                available: true,
                reason: String::new(),
                items: vec![WorktreeItem {
                    path: "/w/b".into(),
                    branch: "b".into(),
                    cleanup_candidate: true,
                    reapable_reason: "merged-pr-head".into(),
                    ..WorktreeItem::default()
                }],
            },
            ..DeliveryExtras::default()
        };
        let remote = RemoteBranches {
            available: true,
            reason: String::new(),
            branches: vec![RemoteBranch {
                branch: "r/x".into(),
                ahead: 2,
                behind: 5,
                updated: "2026-10-18".into(),
            }],
        };
        let state = |b: &str| githelp::BranchState {
            branch: b.into(),
            state: githelp::MergeState::Merged,
            checked_out: false,
            removable: true,
            reason: String::new(),
        };
        let f = orphan_findings("/repo", &facts, &remote, &[state("a/done"), state("c/old")]);
        let messages: Vec<&str> = f.iter().map(|x| x.message.as_str()).collect();
        assert_eq!(
            messages,
            [
                "local branch a/done still exists but PR #12 merged 2026-10-19 (its head has moved \
                 on since that merge); checked out in /w/a",
                "remote branch r/x has no PR and is not merged into main (2 commit(s) ahead, 5 \
                 behind, last updated 2026-10-18) - it needs a decision, not deletion",
                "worktree /w/b [b] is clean and completed work was left behind: its HEAD is the \
                 exact head of a merged PR",
                "local branch \"c/old\" whose work is already in origin/main",
            ]
        );
        assert_eq!(f[2].path, "/w/b");
        assert_eq!(
            f[2].fix,
            "git worktree remove /w/b && git branch -D b && scripts/storage/uor-hygiene --apply"
        );
        // A check that could not run is never a silent pass: each says so, as info.
        let offline = orphan_findings(
            "/repo",
            &DeliveryExtras::default(),
            &RemoteBranches::default(),
            &[],
        );
        let notes: Vec<(&str, &str)> = offline
            .iter()
            .map(|f| (f.severity, f.message.split(':').next().unwrap_or("")))
            .collect();
        assert_eq!(
            notes,
            [
                ("info", "merged-PR branch check skipped"),
                ("info", "remote-branch check skipped"),
                ("info", "worktree check skipped"),
            ]
        );
        // Lookups cut short by the per-run cap are reported as incomplete, not as a pass.
        let capped = DeliveryExtras {
            pr_check: BranchPrCheck {
                available: true,
                reason: "only the first 25 branches were queried this run".into(),
                ..BranchPrCheck::default()
            },
            worktrees: WorktreesView {
                available: true,
                ..WorktreesView::default()
            },
            ..DeliveryExtras::default()
        };
        let f = orphan_findings(
            "/repo",
            &capped,
            &RemoteBranches {
                available: true,
                ..RemoteBranches::default()
            },
            &[],
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message
                .starts_with("merged-PR branch check incomplete: UNAVAILABLE: only the first 25"),
            "{}",
            f[0].message
        );
    }
}
