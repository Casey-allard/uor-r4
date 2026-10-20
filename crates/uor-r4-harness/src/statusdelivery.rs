//! What `status` sections 3 and 4 learn beyond `git cherry`: branches whose PR merged, worktrees
//! left behind, free disk, and the PR list (D27 port of `status._delivery_section`,
//! `_worktree_items`, `_prs_section` and `_render_github`).
//!
//! Section 3's content-based branch classification stays in `status.rs`. This module adds the facts
//! only GitHub, `git worktree` and `df` can give, and the `status.no_orphan_branches` rule reads the
//! same facts, so the view and the rule cannot disagree about which branch is finished.

use std::path::Path;

use serde_json::{json, Map, Value};

use crate::githubcards::{self, BRANCH_PR_LIMIT, MERGED_PR_LIMIT};
use crate::registries::Registries;
use crate::state::StatePaths;
use crate::status::{age_hours, hours_text, truncate};

/// Whether a view may ask GitHub.
///
/// **`status` asks; tests and library callers render offline**, so a unit test on the real checkout
/// never spends thirty `gh` calls (one per local branch) on a question it does not assert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Github {
    /// Read the 15-minute cache, fetching when it is stale; `refresh` forces a fetch.
    Ask { refresh: bool },
    /// Never run `gh`; every GitHub-backed line says so.
    Offline,
}

/// The reason every GitHub-backed line gives in an offline view.
pub const OFFLINE_REASON: &str = "GitHub was not consulted (offline view)";

/// `status.DISK_FLOOR_GIB` — below this, free disk is a stop condition (owner's 30–70 GB band).
pub const DISK_FLOOR_GIB: u64 = 30;

/// Ignored paths a worktree may hold and still be removed: both regenerate.
pub const REGENERABLE_IGNORED: [&str; 2] = ["target/", ".harness/"];

/// Directories `newest_write_hours` never descends into.
const WRITE_SCAN_SKIP: [&str; 4] = [".git", "target", "node_modules", ".venv"];

/// What a cleanup candidate is, printed under the worktrees line so a reader can check the call.
pub const WORKTREE_DEFINITION: &str =
    "cleanup candidate = a non-main worktree that is clean, holds \
no ignored data other than target/ or .harness/, and either (a) its HEAD is the exact head of a \
merged PR or (b) its HEAD is in origin/main and nothing under it was written for 24h (a fresh \
worktree at origin/main is someone's live checkout, not completed work); stale also counts a \
worktree untouched for 24h";

/// `status._unavailable`.
pub fn unavailable(reason: &str) -> String {
    format!(
        "UNAVAILABLE: {}",
        if reason.is_empty() {
            "no reason given"
        } else {
            reason
        }
    )
}

/// `git -C repo …` with every `GIT_*` variable removed, so a hook's environment (which sets
/// `GIT_INDEX_FILE`) cannot point a call at another worktree's index.
fn git(repo: &Path, args: &[&str]) -> (bool, String, String) {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("-C").arg(repo).args(args);
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(&k);
        }
    }
    match cmd.output() {
        Ok(o) => (
            o.status.success(),
            String::from_utf8_lossy(&o.stdout).trim_end().to_string(),
            String::from_utf8_lossy(&o.stderr).trim().to_string(),
        ),
        Err(e) => (false, String::new(), format!("git not runnable: {e}")),
    }
}

fn s(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

/// The PR recorded for a branch, when it is an object.
fn pr_of<'a>(prs: &'a Map<String, Value>, branch: &str) -> Option<&'a Value> {
    prs.get(branch)
        .and_then(|item| item.get("pr"))
        .filter(|p| p.is_object())
}

fn is_merged(pr: &Value) -> bool {
    pr.get("state")
        .and_then(Value::as_str)
        .is_some_and(|st| st.eq_ignore_ascii_case("MERGED"))
}

// ---------------------------------------------------------------------------------------------
// Local branches whose PR merged.
// ---------------------------------------------------------------------------------------------

/// A local branch that still exists although its PR merged.
#[derive(Debug, Clone, PartialEq)]
pub struct MergedPrBranch {
    pub branch: String,
    pub head: String,
    pub pr: i64,
    pub pr_title: String,
    pub merged_at: String,
    /// False when the branch moved on after the merge: it may hold newer work.
    pub head_matches: bool,
    pub current: bool,
    pub fix: String,
    pub checked_out_in: Vec<String>,
}

/// `_delivery_section`'s orphan list: local branches whose PR merged, newest merge first.
///
/// `local` is `(name, head sha)`; `prs` maps a branch to `{"fetched_at", "pr"}` as
/// `github_branch_prs` caches it.
pub fn merged_pr_branches(
    local: &[(String, String)],
    prs: &Map<String, Value>,
    trunk: &str,
    current: &str,
    worktrees: &[WorktreeItem],
) -> Vec<MergedPrBranch> {
    let mut out: Vec<MergedPrBranch> = local
        .iter()
        .filter(|(name, _)| name != trunk && name != "master")
        .filter_map(|(name, sha)| {
            let pr = pr_of(prs, name).filter(|p| is_merged(p))?;
            Some(MergedPrBranch {
                branch: name.clone(),
                head: sha.clone(),
                pr: pr.get("number").and_then(Value::as_i64).unwrap_or(0),
                pr_title: s(pr, "title"),
                merged_at: s(pr, "mergedAt"),
                head_matches: !sha.is_empty() && *sha == s(pr, "headRefOid"),
                current: name == current,
                fix: format!("git branch -D {name} && git push origin --delete {name}"),
                checked_out_in: worktrees
                    .iter()
                    .filter(|w| w.branch == *name)
                    .map(|w| w.path.clone())
                    .collect(),
            })
        })
        .collect();
    out.sort_by(|a, b| b.merged_at.cmp(&a.merged_at));
    out
}

// ---------------------------------------------------------------------------------------------
// Worktrees.
// ---------------------------------------------------------------------------------------------

/// One `git worktree list` entry and what is known about it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorktreeItem {
    pub path: String,
    pub head: String,
    pub branch: String,
    /// The first entry: the shared checkout, never a cleanup candidate.
    pub main: bool,
    pub current: bool,
    /// `None` when `git status` could not be read: unknown is not clean.
    pub clean: Option<bool>,
    pub ignored_count: usize,
    pub blocking_ignored: bool,
    pub ignored_data: Vec<String>,
    pub contained_in_origin_main: Option<bool>,
    pub newest_write_hours: Option<f64>,
    pub idle: Option<bool>,
    pub merged_pr_head: bool,
    pub reapable: bool,
    pub reapable_reason: String,
    pub cleanup_candidate: bool,
    pub stale: bool,
}

/// `status._worktree_items` — raw facts per worktree, read-only. Candidacy is decided by
/// `classify_worktrees`, which needs the PR facts as well.
pub fn worktree_items(repo: &Path, origin_main: Option<&str>) -> Result<Vec<WorktreeItem>, String> {
    let (ok, out, err) = git(repo, &["worktree", "list", "--porcelain"]);
    if !ok {
        return Err(if err.is_empty() {
            "`git worktree list --porcelain` failed".to_string()
        } else {
            err
        });
    }
    let here = std::fs::canonicalize(repo).ok();
    let mut items = Vec::new();
    for (index, block) in out
        .split("\n\n")
        .filter(|b| !b.trim().is_empty())
        .enumerate()
    {
        let field = |key: &str| {
            block
                .lines()
                .find_map(|l| l.strip_prefix(key).and_then(|v| v.strip_prefix(' ')))
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let path = field("worktree");
        let mut item = WorktreeItem {
            head: field("HEAD"),
            branch: field("branch")
                .trim_start_matches("refs/heads/")
                .to_string(),
            main: index == 0,
            current: !path.is_empty() && std::fs::canonicalize(&path).ok() == here,
            path,
            ..WorktreeItem::default()
        };
        if !item.path.is_empty() && Path::new(&item.path).is_dir() {
            if item.main {
                // The shared checkout holds the owner's ignored material by design; scanning it is
                // slow and its result is never used.
                item.clean = Some(true);
            } else {
                let (st_ok, st, _) = git(
                    repo,
                    &["-C", &item.path, "status", "--porcelain", "--ignored"],
                );
                if st_ok {
                    let lines: Vec<&str> = st.lines().filter(|l| !l.trim().is_empty()).collect();
                    item.clean = Some(lines.iter().all(|l| l.starts_with("!!")));
                    let ignored: Vec<String> = lines
                        .iter()
                        .filter_map(|l| l.strip_prefix("!!"))
                        .map(|p| p.trim().to_string())
                        .collect();
                    item.ignored_count = ignored.len();
                    let blockers: Vec<String> = ignored
                        .into_iter()
                        .filter(|p| !REGENERABLE_IGNORED.iter().any(|r| p.starts_with(r)))
                        .collect();
                    item.blocking_ignored = !blockers.is_empty();
                    item.ignored_data = blockers.into_iter().take(10).collect();
                }
                item.newest_write_hours = newest_write_hours(Path::new(&item.path));
                item.idle = item.newest_write_hours.map(|h| h >= 24.0);
            }
            if !item.head.is_empty() && origin_main.is_some() {
                let (anc, _, _) = git(
                    repo,
                    &["merge-base", "--is-ancestor", &item.head, "origin/main"],
                );
                item.contained_in_origin_main = Some(anc);
            }
        }
        items.push(item);
    }
    Ok(items)
}

/// Decides which worktrees are completed work left behind.
///
/// **An ancestor of origin/main is not enough**: a worktree created a minute ago for work that has
/// not started is also one. Only a merged PR head, or 24h without a write, makes it left behind.
pub fn classify_worktrees(items: &mut [WorktreeItem], prs: &Map<String, Value>) {
    for item in items.iter_mut().filter(|i| !i.main) {
        item.merged_pr_head = pr_of(prs, &item.branch)
            .is_some_and(|pr| is_merged(pr) && item.head == s(pr, "headRefOid"));
        let contained = item.contained_in_origin_main == Some(true);
        item.reapable = contained || item.merged_pr_head;
        item.reapable_reason = if item.merged_pr_head {
            "merged-pr-head"
        } else if contained {
            "contained-in-origin-main"
        } else {
            ""
        }
        .to_string();
        item.cleanup_candidate = item.clean == Some(true)
            && item.reapable
            && !item.blocking_ignored
            && (item.merged_pr_head || item.idle == Some(true));
        item.stale = item.cleanup_candidate || item.idle == Some(true);
    }
}

/// `status._newest_write_hours` — hours since the newest file under `root`, or `None` when unknown.
///
/// Stops at the first file written within 24h (the only question asked) and gives up past 200,000
/// files rather than walk an unbounded tree. Symlinked directories are not followed.
pub fn newest_write_hours(root: &Path) -> Option<f64> {
    let now = std::time::SystemTime::now();
    let round2 = |h: f64| (h * 100.0).round() / 100.0;
    let mut newest: Option<f64> = None;
    let mut count = 0usize;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let is_dir = if kind.is_symlink() {
                std::fs::metadata(&path).is_ok_and(|m| m.is_dir())
            } else {
                kind.is_dir()
            };
            if is_dir {
                let name = entry.file_name();
                if !kind.is_symlink() && !WRITE_SCAN_SKIP.iter().any(|s| name == *s) {
                    stack.push(path);
                }
                continue;
            }
            count += 1;
            if count > 200_000 {
                return None;
            }
            let Ok(mtime) = std::fs::metadata(&path).and_then(|m| m.modified()) else {
                continue;
            };
            let age = match now.duration_since(mtime) {
                Ok(d) => d.as_secs_f64() / 3600.0,
                Err(e) => -(e.duration().as_secs_f64() / 3600.0),
            };
            let n = newest.map_or(age, |n| n.min(age));
            newest = Some(n);
            if n < 24.0 {
                return Some(round2(n));
            }
        }
    }
    newest.map(round2)
}

/// `git worktree list` with candidacy decided, or why it could not be read.
#[derive(Debug, Clone, Default)]
pub struct WorktreesView {
    pub available: bool,
    pub reason: String,
    pub items: Vec<WorktreeItem>,
}

impl WorktreesView {
    pub fn candidates(&self) -> Vec<&WorktreeItem> {
        self.items.iter().filter(|i| i.cleanup_candidate).collect()
    }

    pub fn stale_count(&self) -> usize {
        self.items.iter().filter(|i| i.stale).count()
    }
}

// ---------------------------------------------------------------------------------------------
// Disk.
// ---------------------------------------------------------------------------------------------

/// Free and total bytes where the checkout lives, from `budget::disk_free` (`df -k`).
#[derive(Debug, Clone, Default)]
pub struct DiskView {
    pub available: bool,
    pub reason: String,
    pub path: String,
    pub free_bytes: i64,
    pub total_bytes: i64,
}

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

impl DiskView {
    pub fn read(repo: &Path) -> DiskView {
        let v = crate::budget::disk_free(repo);
        if v.get("ok").and_then(Value::as_bool) != Some(true) {
            return DiskView {
                available: false,
                reason: s(&v, "error"),
                path: repo.display().to_string(),
                ..DiskView::default()
            };
        }
        DiskView {
            available: true,
            reason: String::new(),
            path: repo.display().to_string(),
            free_bytes: v.get("free_bytes").and_then(Value::as_i64).unwrap_or(0),
            total_bytes: v.get("total_bytes").and_then(Value::as_i64).unwrap_or(0),
        }
    }

    pub fn ok(&self) -> bool {
        self.free_bytes as f64 >= DISK_FLOOR_GIB as f64 * GIB
    }
}

// ---------------------------------------------------------------------------------------------
// Everything section 3 adds, gathered once.
// ---------------------------------------------------------------------------------------------

/// Whether the per-branch PR lookup ran, and what it covered.
#[derive(Debug, Clone, Default)]
pub struct BranchPrCheck {
    pub available: bool,
    pub reason: String,
    pub branches: usize,
    pub queried: usize,
}

/// Section 3's GitHub, worktree and disk facts.
#[derive(Debug, Clone, Default)]
pub struct DeliveryExtras {
    pub pr_check: BranchPrCheck,
    pub merged: Vec<MergedPrBranch>,
    pub worktrees: WorktreesView,
    pub disk: DiskView,
}

/// Gathers the facts for section 3 and for `status.no_orphan_branches`.
pub fn gather(
    paths: &StatePaths,
    trunk: &str,
    current: &str,
    gh: Github,
    now: i64,
) -> DeliveryExtras {
    let repo = paths.repo.as_path();
    let (ok_main, main_sha, _) = git(repo, &["rev-parse", "origin/main"]);
    let origin_main = ok_main.then_some(main_sha.trim().to_string());

    let (ok_refs, refs, refs_err) = git(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short)\t%(objectname)",
            "refs/heads",
        ],
    );
    let local: Vec<(String, String)> = refs
        .lines()
        .filter_map(|l| {
            let (name, sha) = l.split_once('\t')?;
            (!name.trim().is_empty()).then(|| (name.trim().to_string(), sha.trim().to_string()))
        })
        .collect();
    let deliverable: Vec<String> = local
        .iter()
        .map(|(n, _)| n.clone())
        .filter(|n| n != trunk && n != "master")
        .collect();

    let (mut pr_check, prs) = match gh {
        Github::Offline => (
            BranchPrCheck {
                reason: OFFLINE_REASON.to_string(),
                ..BranchPrCheck::default()
            },
            Map::new(),
        ),
        Github::Ask { refresh } => {
            let snap = githubcards::github_branch_prs(paths, &deliverable, refresh, now);
            (
                BranchPrCheck {
                    available: snap.available,
                    reason: snap.reason,
                    branches: 0,
                    queried: snap.queried,
                },
                snap.items,
            )
        }
    };
    pr_check.branches = local.len();
    if !ok_refs {
        pr_check.available = false;
        pr_check.reason = if refs_err.is_empty() {
            "`git for-each-ref refs/heads` failed".to_string()
        } else {
            refs_err
        };
    }

    let worktrees = match worktree_items(repo, origin_main.as_deref()) {
        Ok(mut items) => {
            classify_worktrees(&mut items, &prs);
            WorktreesView {
                available: true,
                reason: String::new(),
                items,
            }
        }
        Err(reason) => WorktreesView {
            available: false,
            reason,
            items: Vec::new(),
        },
    };
    DeliveryExtras {
        merged: merged_pr_branches(&local, &prs, trunk, current, &worktrees.items),
        pr_check,
        worktrees,
        disk: DiskView::read(repo),
    }
}

/// `gather` for the checkout's own trunk (`origin/HEAD`, else main) and current branch.
pub fn gather_here(paths: &StatePaths, gh: Github, now: i64) -> DeliveryExtras {
    let repo = paths.repo.as_path();
    let (_, current, _) = git(repo, &["rev-parse", "--abbrev-ref", "HEAD"]);
    let (ok, head_ref, _) = git(
        repo,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    );
    let trunk = match head_ref.trim().rsplit('/').next() {
        Some(t) if ok && !t.is_empty() => t.to_string(),
        _ => "main".to_string(),
    };
    gather(paths, &trunk, current.trim(), gh, now)
}

/// Section 3's lines for these facts, Python's text.
pub fn render_extras(x: &DeliveryExtras) -> (Vec<String>, Vec<String>) {
    let mut prs = Vec::new();
    if !x.pr_check.available {
        prs.push(format!(
            "   branches with a merged PR: {}",
            unavailable(&x.pr_check.reason)
        ));
    } else if x.merged.is_empty() {
        prs.push("   branches with a merged PR still present: none".to_string());
    } else {
        prs.push(format!(
            "   branches with a merged PR still present: {}",
            x.merged.len()
        ));
        for o in x.merged.iter().take(3) {
            let moved = if o.head_matches {
                ""
            } else {
                " (branch head has moved on)"
            };
            let place = if o.checked_out_in.is_empty() {
                String::new()
            } else {
                format!(" [checked out in {}]", o.checked_out_in.join(", "))
            };
            prs.push(format!(
                "      {} -> PR #{} merged {}{moved}{place}",
                o.branch,
                o.pr,
                o.merged_at.chars().take(10).collect::<String>()
            ));
            prs.push(format!("         fix: {}", o.fix));
        }
        if x.merged.len() > 3 {
            prs.push(format!("      (+{} more)", x.merged.len() - 3));
        }
    }

    let mut rest = Vec::new();
    let wt = &x.worktrees;
    if !wt.available {
        rest.push(format!("   worktrees: {}", unavailable(&wt.reason)));
    } else {
        let cands = wt.candidates();
        rest.push(format!(
            "   worktrees: {} total · {} clean and already in origin/main (cleanup candidates) · {} stale",
            wt.items.len(),
            cands.len(),
            wt.stale_count()
        ));
        for c in cands.iter().take(3) {
            let here = if c.current { " (this worktree)" } else { "" };
            rest.push(format!(
                "      remove {} [{}]{here}",
                c.path,
                if c.branch.is_empty() {
                    "detached"
                } else {
                    &c.branch
                }
            ));
        }
        if cands.len() > 3 {
            rest.push(format!("      (+{} more)", cands.len() - 3));
        }
        rest.push(format!("      definition: {WORKTREE_DEFINITION}"));
    }
    let d = &x.disk;
    if !d.available {
        rest.push(format!("   disk: {}", unavailable(&d.reason)));
    } else {
        let verdict = if d.ok() {
            "ok".to_string()
        } else {
            format!("BELOW the {DISK_FLOOR_GIB} GiB floor")
        };
        rest.push(format!(
            "   disk: {:.1} GiB free of {:.1} GiB ({verdict})",
            d.free_bytes as f64 / GIB,
            d.total_bytes as f64 / GIB
        ));
    }
    (prs, rest)
}

/// Python's `delivery` JSON shape for these facts.
pub fn extras_json(x: &DeliveryExtras) -> Value {
    let wt_item = |i: &WorktreeItem| {
        json!({
            "path": i.path, "head": i.head, "branch": i.branch, "main": i.main,
            "current": i.current, "clean": i.clean, "ignored_data": i.ignored_data,
            "ignored_count": i.ignored_count, "blocking_ignored": i.blocking_ignored,
            "contained_in_origin_main": i.contained_in_origin_main, "reapable": i.reapable,
            "reapable_reason": i.reapable_reason, "merged_pr_head": i.merged_pr_head,
            "cleanup_candidate": i.cleanup_candidate, "newest_write_hours": i.newest_write_hours,
            "idle": i.idle, "stale": i.stale,
        })
    };
    let wt = &x.worktrees;
    let disk = if x.disk.available {
        json!({
            "available": true, "reason": "", "path": x.disk.path,
            "free_bytes": x.disk.free_bytes,
            "free_gib": (x.disk.free_bytes as f64 / GIB * 10.0).round() / 10.0,
            "total_gib": (x.disk.total_bytes as f64 / GIB * 10.0).round() / 10.0,
            "floor_gib": DISK_FLOOR_GIB, "ok": x.disk.ok(),
        })
    } else {
        json!({"available": false, "reason": x.disk.reason})
    };
    json!({
        "branch_pr_check": {
            "available": x.pr_check.available, "reason": x.pr_check.reason,
            "branches": x.pr_check.branches, "queried": x.pr_check.queried,
            "per_branch_limit": BRANCH_PR_LIMIT,
        },
        "branches_with_merged_pr": x.merged.iter().map(|o| json!({
            "branch": o.branch, "head": o.head, "pr": o.pr, "pr_title": o.pr_title,
            "merged_at": o.merged_at, "head_matches": o.head_matches, "current": o.current,
            "fix": o.fix, "checked_out_in": o.checked_out_in,
        })).collect::<Vec<_>>(),
        "worktrees": {
            "available": wt.available, "reason": wt.reason, "count": wt.items.len(),
            "items": wt.items.iter().map(wt_item).collect::<Vec<_>>(),
            "cleanup_candidates": wt.candidates().into_iter().map(wt_item).collect::<Vec<_>>(),
            "stale_count": wt.stale_count(), "definition": WORKTREE_DEFINITION,
        },
        "disk": disk,
    })
}

// ---------------------------------------------------------------------------------------------
// Remote branches with no PR.
// ---------------------------------------------------------------------------------------------

/// A remote branch with work main lacks and no PR.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteBranch {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub updated: String,
}

/// A remote branch worth a PR lookup: named, not a trunk or site branch, not protected, and holding
/// work main lacks (a branch with nothing ahead is finished on its own merits).
fn is_candidate(name: &str, ahead: usize, protected: &[String]) -> bool {
    !(name.is_empty()
        || name.starts_with("HEAD")
        || ["main", "gh-pages"].contains(&name)
        || protected.iter().any(|p| p == name)
        || ahead == 0)
}

/// `githelp.remote_branches_without_pr`'s decision, over `(name, updated, ahead, behind)` rows newest
/// first. `has_pr` answers `None` when it could not ask.
///
/// **Two departures from Python, both toward reporting less.** A protected branch
/// (`docs/harness/branches.json`) is never reported: Python flagged `gh-pages-preview`, which the
/// owner's 20 October rule keeps forever. And a branch whose PR lookup FAILED is skipped rather than
/// reported as having no PR, so an offline run cannot call every branch an orphan.
pub fn without_pr(
    rows: &[(String, String, usize, usize)],
    protected: &[String],
    has_pr: &mut dyn FnMut(&str) -> Option<bool>,
    limit: usize,
) -> Vec<RemoteBranch> {
    let mut out = Vec::new();
    for (name, updated, ahead, behind) in rows {
        if !is_candidate(name, *ahead, protected) {
            continue;
        }
        if has_pr(name) != Some(false) {
            continue;
        }
        out.push(RemoteBranch {
            branch: name.clone(),
            ahead: *ahead,
            behind: *behind,
            updated: updated.clone(),
        });
        if out.len() >= limit {
            break;
        }
    }
    out
}

/// Remote branches that have no PR and are not merged into origin/main. Offline, none is known.
pub fn remote_branches_without_pr(
    paths: &StatePaths,
    gh: Github,
    limit: usize,
    now: i64,
) -> RemoteBranches {
    let Github::Ask { refresh } = gh else {
        return RemoteBranches {
            available: false,
            reason: OFFLINE_REASON.to_string(),
            branches: Vec::new(),
        };
    };
    let repo = paths.repo.as_path();
    let (ok, listing, err) = git(
        repo,
        &[
            "for-each-ref",
            "--sort=-committerdate",
            "--format=%(refname:short)|%(committerdate:short)",
            "refs/remotes/origin/",
        ],
    );
    if !ok {
        return RemoteBranches {
            available: false,
            reason: err,
            branches: Vec::new(),
        };
    }
    let count = |range: &str| {
        let (ok, n, _) = git(repo, &["rev-list", "--count", range]);
        if ok {
            n.trim().parse::<usize>().ok()
        } else {
            None
        }
    };
    let rows: Vec<(String, String, usize, usize)> = listing
        .lines()
        .filter_map(|l| {
            let (r, updated) = l.split_once('|')?;
            let name = r.strip_prefix("origin/")?.to_string();
            let ahead = count(&format!("origin/main..origin/{name}"))?;
            let behind = count(&format!("origin/{name}..origin/main")).unwrap_or(0);
            Some((name, updated.to_string(), ahead, behind))
        })
        .collect();
    let protected = crate::githelp::protected_branches(repo);
    // The PR lookups go through the same per-branch cache as the local branches, capped at
    // `BRANCH_PR_LIMIT` calls here and again for the local branches (so at most 50 per run), so
    // `check` never spends one uncached `gh` call per remote branch.
    let candidates: Vec<String> = rows
        .iter()
        .filter(|(name, _, ahead, _)| is_candidate(name, *ahead, &protected))
        .map(|(name, ..)| name.clone())
        .collect();
    let snap = githubcards::github_branch_prs(paths, &candidates, refresh, now);
    let mut has_pr = |name: &str| {
        snap.items
            .get(name)
            .and_then(|i| i.get("pr"))
            .map(Value::is_object)
    };
    RemoteBranches {
        available: snap.available,
        reason: snap.reason.clone(),
        branches: without_pr(&rows, &protected, &mut has_pr, limit),
    }
}

/// The remote branches with work and no PR, and whether every lookup could be made.
#[derive(Debug, Clone, Default)]
pub struct RemoteBranches {
    pub available: bool,
    /// Why some or all lookups failed; empty when every candidate was answered.
    pub reason: String,
    pub branches: Vec<RemoteBranch>,
}

// ---------------------------------------------------------------------------------------------
// Section 4: the PR list.
// ---------------------------------------------------------------------------------------------

/// `status._milestone_for_issue`.
fn milestone_for_issue(regs: &Registries, issue: i64) -> String {
    if let Some(m) = regs.milestones.iter().find(|m| m.issue == issue) {
        return m.id.clone();
    }
    if regs.tracker_issue() == Some(issue) {
        return "tracker".to_string();
    }
    String::new()
}

/// `status._milestone_ref` — the milestone a PR title references: an issue number first, then the
/// milestone id as a whole word.
pub fn milestone_ref(title: &str, regs: Option<&Registries>) -> (String, Value) {
    let numbers: Vec<i64> = regex::Regex::new(r"#(\d+)")
        .expect("static pattern")
        .captures_iter(title)
        .filter_map(|c| c[1].parse().ok())
        .collect();
    if let Some(regs) = regs {
        for n in &numbers {
            let id = milestone_for_issue(regs, *n);
            if !id.is_empty() {
                return (id, Value::from(*n));
            }
        }
        for m in &regs.milestones {
            let word = format!(r"(?i)\b{}\b", regex::escape(&m.id));
            if regex::Regex::new(&word).is_ok_and(|re| re.is_match(title)) {
                return (m.id.clone(), Value::from(m.issue));
            }
        }
    }
    (
        String::new(),
        numbers.first().map_or(Value::Null, |n| Value::from(*n)),
    )
}

/// `status._pr_row`.
fn pr_row(pr: &Value, regs: Option<&Registries>, now: i64) -> Value {
    let title = s(pr, "title");
    let (milestone, issue) = milestone_ref(&title, regs);
    let merged_at = s(pr, "mergedAt");
    let created_at = s(pr, "createdAt");
    let stamp = if merged_at.is_empty() {
        &created_at
    } else {
        &merged_at
    };
    json!({
        "number": pr.get("number").cloned().unwrap_or(Value::Null),
        "title": title,
        "branch": s(pr, "headRefName"),
        "url": s(pr, "url"),
        "milestone": milestone,
        "milestone_issue": issue,
        "age_hours": age_hours(&Value::String(stamp.clone()), now),
        "created_at": created_at,
        "merged_at": merged_at,
    })
}

/// `status._prs_section` — open PRs newest first, and PRs merged in the last 48h, from a
/// `github_prs` snapshot.
pub fn prs_section(snap: &Value, regs: Option<&Registries>, now: i64) -> Value {
    let base = json!({
        "available": snap.get("available").and_then(Value::as_bool).unwrap_or(false),
        "reason": s(snap, "reason"),
        "fetched_at": snap.get("fetched_at").cloned().unwrap_or(Value::from("")),
        "from_cache": snap.get("from_cache").and_then(Value::as_bool).unwrap_or(false),
        "age_seconds": snap.get("age_seconds").cloned().unwrap_or(Value::Null),
        "merged_limit": MERGED_PR_LIMIT,
    });
    let rows = |key: &str| -> Vec<Value> {
        snap.get(key)
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter(|p| p.is_object())
                    .map(|p| pr_row(p, regs, now))
                    .collect()
            })
            .unwrap_or_default()
    };
    let (mut open, mut recent) = if base["available"] == Value::Bool(true) {
        let open = rows("open");
        let recent: Vec<Value> = rows("merged")
            .into_iter()
            .filter(|r| r["age_hours"].as_f64().is_some_and(|h| h <= 48.0))
            .collect();
        (open, recent)
    } else {
        (Vec::new(), Vec::new())
    };
    let age = |r: &Value| r["age_hours"].as_f64().unwrap_or(0.0);
    recent.sort_by(|a, b| age(a).total_cmp(&age(b)));
    open.sort_by_key(|r| std::cmp::Reverse(s(r, "created_at")));
    let mut out = base;
    out["open"] = json!({"count": open.len(), "items": open});
    out["merged_48h"] = json!({"count": recent.len(), "items": recent});
    out
}

/// Section 4's data: `github_prs` when GitHub may be asked, otherwise unavailable.
pub fn prs_view(paths: &StatePaths, regs: Option<&Registries>, gh: Github, now: i64) -> Value {
    let snap = match gh {
        Github::Ask { refresh } => githubcards::github_prs(paths, refresh, now),
        Github::Offline => json!({"available": false, "reason": OFFLINE_REASON}),
    };
    prs_section(&snap, regs, now)
}

/// `status._render_github` — section 4.
pub fn render_github(prs: &Value, cache: &Path) -> Vec<String> {
    let mut lines = vec!["4. GITHUB".to_string()];
    if prs.get("available").and_then(Value::as_bool) != Some(true) {
        lines.push(format!("   PRs: {}", unavailable(&s(prs, "reason"))));
        return lines;
    }
    let source = match prs.get("age_seconds").and_then(Value::as_i64) {
        Some(age) if prs.get("from_cache").and_then(Value::as_bool) == Some(true) => {
            format!("cached {}m ago", age / 60)
        }
        _ => "fetched now".to_string(),
    };
    lines.push(format!("   PRs ({source}, cached at {}):", cache.display()));
    let count = |k: &str| prs[k]["count"].as_u64().unwrap_or(0);
    lines.push(format!(
        "      open {} · merged in the last 48h {}",
        count("open"),
        count("merged_48h")
    ));
    let row = |p: &Value, prefix: &str| {
        let refname = match (
            s(p, "milestone"),
            p.get("milestone_issue").and_then(Value::as_i64),
        ) {
            (m, _) if !m.is_empty() => m,
            (_, Some(n)) => format!("#{n}"),
            _ => "no milestone ref".to_string(),
        };
        format!(
            "      {prefix}#{} [{refname}] {} old: {}",
            p.get("number").cloned().unwrap_or(Value::Null),
            hours_text(p["age_hours"].as_f64()),
            truncate(&s(p, "title"), 80)
        )
    };
    let items = |k: &str| prs[k]["items"].as_array().cloned().unwrap_or_default();
    for p in items("merged_48h").iter().take(5) {
        lines.push(row(p, ""));
    }
    for p in items("open").iter().take(5) {
        lines.push(row(p, "OPEN "));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pr(state: &str, number: i64, head: &str, merged: &str) -> Value {
        json!({"number": number, "title": "t", "state": state, "headRefOid": head, "mergedAt": merged})
    }

    fn snap(entries: &[(&str, Value)]) -> Map<String, Value> {
        entries
            .iter()
            .map(|(b, p)| (b.to_string(), json!({"fetched_at": "x", "pr": p})))
            .collect()
    }

    /// **Only a MERGED PR makes a branch an orphan**, newest merge first; an open PR or none does not,
    /// and the trunk is never one.
    #[test]
    fn a_branch_is_an_orphan_only_when_its_pr_merged() {
        let local = vec![
            ("main".to_string(), "m".to_string()),
            ("a/old".to_string(), "h1".to_string()),
            ("a/new".to_string(), "h2".to_string()),
            ("a/open".to_string(), "h3".to_string()),
            ("a/none".to_string(), "h4".to_string()),
        ];
        let prs = snap(&[
            ("main", pr("MERGED", 1, "m", "2026-10-01T00:00:00Z")),
            ("a/old", pr("MERGED", 2, "h1", "2026-10-02T00:00:00Z")),
            ("a/new", pr("merged", 3, "other", "2026-10-09T00:00:00Z")),
            ("a/open", pr("OPEN", 4, "h3", "")),
        ]);
        let wt = vec![WorktreeItem {
            path: "/w".into(),
            branch: "a/new".into(),
            ..WorktreeItem::default()
        }];
        let out = merged_pr_branches(&local, &prs, "main", "a/old", &wt);
        let names: Vec<&str> = out.iter().map(|o| o.branch.as_str()).collect();
        assert_eq!(names, ["a/new", "a/old"]);
        assert!(!out[0].head_matches, "its head moved on after the merge");
        assert_eq!(out[0].checked_out_in, ["/w"]);
        assert!(out[1].head_matches && out[1].current);
        assert_eq!(
            out[1].fix,
            "git branch -D a/old && git push origin --delete a/old"
        );
    }

    /// **A worktree at origin/main is a candidate only after 24h idle or at a merged PR head**: a
    /// fresh one is someone's live checkout. Dirty or holding non-regenerable ignored data never is.
    #[test]
    fn a_fresh_worktree_at_origin_main_is_not_a_cleanup_candidate() {
        let base = WorktreeItem {
            clean: Some(true),
            contained_in_origin_main: Some(true),
            idle: Some(false),
            head: "h".into(),
            branch: "b".into(),
            ..WorktreeItem::default()
        };
        let mut items = vec![
            WorktreeItem {
                main: true,
                ..base.clone()
            },
            base.clone(),
            WorktreeItem {
                idle: Some(true),
                ..base.clone()
            },
            WorktreeItem {
                idle: Some(true),
                clean: Some(false),
                ..base.clone()
            },
            WorktreeItem {
                idle: Some(true),
                blocking_ignored: true,
                ..base.clone()
            },
            WorktreeItem {
                contained_in_origin_main: Some(false),
                branch: "p".into(),
                ..base.clone()
            },
        ];
        classify_worktrees(&mut items, &snap(&[("p", pr("MERGED", 9, "h", "x"))]));
        let flags: Vec<bool> = items.iter().map(|i| i.cleanup_candidate).collect();
        assert_eq!(flags, [false, false, true, false, false, true]);
        assert_eq!(items[5].reapable_reason, "merged-pr-head");
        assert!(items[3].stale, "idle counts as stale even when dirty");
    }

    /// **Protected branches and failed lookups are never reported**; a branch with no work ahead of
    /// main is finished on its own merits, and one with a PR belongs to the merged-PR check.
    #[test]
    fn remote_branches_report_only_known_prless_unprotected_work() {
        let row = |n: &str, ahead: usize| (n.to_string(), "2026-10-20".to_string(), ahead, 3usize);
        let rows = vec![
            row("gh-pages-preview", 9),
            row("gh-pages", 2),
            row("done", 0),
            row("has-pr", 2),
            row("unknown", 2),
            row("orphan", 4),
        ];
        let protected = vec!["main".to_string(), "gh-pages-preview".to_string()];
        let mut has_pr = |n: &str| match n {
            "has-pr" => Some(true),
            "unknown" => None,
            _ => Some(false),
        };
        let out = without_pr(&rows, &protected, &mut has_pr, 20);
        assert_eq!(
            out,
            [RemoteBranch {
                branch: "orphan".into(),
                ahead: 4,
                behind: 3,
                updated: "2026-10-20".into()
            }]
        );
    }

    /// Section 4 counts open PRs and PRs merged within 48h, and names each one's milestone.
    #[test]
    fn the_pr_section_counts_recent_merges_and_open_prs() {
        let now = crate::state::parse_iso_unix("2026-10-20T12:00:00Z").unwrap();
        let snapshot = json!({
            "available": true, "from_cache": true, "age_seconds": 120,
            "open": [{"number": 7, "title": "wip (References #2029)", "createdAt": "2026-10-20T10:00:00Z"}],
            "merged": [
                {"number": 6, "title": "fresh", "mergedAt": "2026-10-20T11:00:00Z"},
                {"number": 5, "title": "old", "mergedAt": "2026-10-10T11:00:00Z"},
            ],
        });
        let section = prs_section(&snapshot, None, now);
        assert_eq!(section["open"]["count"], 1);
        assert_eq!(section["merged_48h"]["count"], 1);
        let text = render_github(&section, Path::new("/live/github.json")).join("\n");
        assert!(
            text.contains("PRs (cached 2m ago, cached at /live/github.json):"),
            "{text}"
        );
        assert!(text.contains("open 1 · merged in the last 48h 1"), "{text}");
        assert!(
            text.contains("#6 [no milestone ref] 1.0h old: fresh"),
            "{text}"
        );
        assert!(
            text.contains("OPEN #7 [#2029] 2.0h old: wip (References #2029)"),
            "{text}"
        );
        let off = prs_section(
            &json!({"available": false, "reason": OFFLINE_REASON}),
            None,
            now,
        );
        assert_eq!(
            render_github(&off, Path::new("/x"))[1],
            format!("   PRs: UNAVAILABLE: {OFFLINE_REASON}")
        );
    }

    /// A file written now stops the scan under a day; an empty tree has no answer.
    #[test]
    fn newest_write_hours_sees_a_fresh_file_and_skips_target() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(newest_write_hours(dir.path()), None);
        std::fs::create_dir_all(dir.path().join("target")).expect("mkdir");
        std::fs::write(dir.path().join("target/x"), "x").expect("write");
        assert_eq!(newest_write_hours(dir.path()), None, "target/ is skipped");
        std::fs::write(dir.path().join("y"), "y").expect("write");
        assert!(newest_write_hours(dir.path()).is_some_and(|h| h < 1.0));
    }
}
