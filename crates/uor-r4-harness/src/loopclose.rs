//! `uor-harness loop close` (loop PR 7b, ORDER call 4): loop step 9.
//!
//! **`--session`** ends a session on a task branch, in this order; a refusal before the first
//! GitHub write writes nothing:
//!
//! 1. the task folder: the one ORDER this branch adds (`loop report`'s search);
//! 2. its `REPORT.md` must pass `loop report --check`;
//! 3. the PR: `--pr <n>` (refused unless it is OPEN with the current branch as its head), else the one
//!    open PR whose head branch is the current branch; its head sha must equal local `HEAD` ("push
//!    first" otherwise);
//! 4. the body: the `size: …` line `loop pr-check` prints for `merge-base origin/main HEAD..HEAD`, the
//!    REPORT without its `# ` title line, `References #<issue>` from `docs/roadmap/issues.json` when
//!    the ORDER's item has an issue, and the Claude Code footer when the lab is `claude`;
//! 5. the PICKUP for `local/<lab>/PICKUP.md` under the main checkout (not the task worktree, which
//!    `--verify` removes), from `docs/loop/templates/PICKUP.md`; 8,192 bytes or
//!    more refuses;
//! 6. writes: the PR body, then `gh pr ready` when the PR is a draft, then the PICKUP.
//!
//! `--dry-run` prints the body and the PICKUP and writes nothing. Exit codes: 0 done, 1 refused (a
//! check failed; nothing written), 2 cannot run.
//!
//! **`--verify --pr <n> [--no-gates]`** is loop step 13, after the merge. Every proof runs before any
//! deletion, and nothing is deleted when a proof fails: `state` (GitHub reports `MERGED`), `squash`
//! (the merge commit is on fresh `origin/main` and its subject ends `(#<n>)`), `content` (for the
//! files the PR changed, each is identical between the merge commit and the PR head, or the PR's patch
//! has the merge's `git patch-id --verbatim`) and `done-when` (`loop gates --full` in a detached
//! worktree of `origin/main`, removed afterwards whatever the result; a red gate's `.log` and `.exit`
//! are first copied to `local/<lab>/verify-logs/<n>/<run>/`). Then, only for this lab's own
//! branch (`<lab>/...`), three independent steps, each with its own guard (a kept worktree does not
//! stop the others): the clean worktree under `.worktrees/<lab>-*` holding no ignored files beyond
//! `target/`, `local/<lab>/views/` and `local/<lab>/gates.jsonl`; the local branch when it is at the
//! PR head and checked out nowhere; and the remote branch by a lease on the PR head. Exit codes: 0 verified and cleaned up, 1 refused or not verified (or something kept), 2
//! cannot run.

use serde::Deserialize;
use std::cell::RefCell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str = "usage: uor-harness loop close (--session [--pr <n>] [--dry-run] | --verify --pr <n> [--no-gates])";
const PICKUP_TEMPLATE: &str = "docs/loop/templates/PICKUP.md";
const ISSUES: &str = "docs/roadmap/issues.json";
const FOOTER: &str = "🤖 Generated with [Claude Code](https://claude.com/claude-code)";

/// Points `loop close` at a JSON file standing in for `gh`: `prs` (each `number`, `headRefName`,
/// `isDraft`, `headRefOid`), optional `failSetBody` / `failReady`, and `merged` (`--verify`'s view of a PR,
/// keyed by number: `state`, `mergeCommit`, `headRefName`, `headRefOid`). Every write that succeeds is
/// appended as one JSON line to `<file>.calls.jsonl`. Never set it outside tests.
pub const CLOSE_GH_FAKE_ENV: &str = "UOR_LOOP_CLOSE_GH_FAKE";

/// An open PR as `loop close` needs it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PrRef {
    pub number: u64,
    #[serde(rename = "headRefName")]
    pub head_ref: String,
    pub state: String,
    #[serde(rename = "isDraft")]
    pub is_draft: bool,
    #[serde(rename = "headRefOid")]
    pub head_sha: String,
}

/// What `loop close` reads from and writes to GitHub.
pub trait CloseGh {
    /// The open PRs whose head branch is `branch`.
    fn pr_for_branch(&self, branch: &str) -> Result<Vec<PrRef>, String>;
    fn pr(&self, number: u64) -> Result<PrRef, String>;
    fn set_body(&self, number: u64, body: &str) -> Result<(), String>;
    fn mark_ready(&self, number: u64) -> Result<(), String>;
    /// A PR's state, squash commit and head, for `--verify`.
    fn merged(&self, number: u64) -> Result<MergedPr, String>;
}

/// What `--verify` reads about a PR: its state, the commit it was merged as (empty when none), and
/// its head branch and sha.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MergedPr {
    pub state: String,
    #[serde(rename = "mergeCommit")]
    pub merge_commit: String,
    #[serde(rename = "headRefName")]
    pub head_ref: String,
    #[serde(rename = "headRefOid")]
    pub head_sha: String,
}

/// `gh`, run from the repository root.
struct RealCloseGh {
    root: PathBuf,
}

impl CloseGh for RealCloseGh {
    fn pr_for_branch(&self, branch: &str) -> Result<Vec<PrRef>, String> {
        let out = crate::loopclaim::run_gh(
            &self.root,
            &[
                "pr",
                "list",
                "--state",
                "open",
                "--head",
                branch,
                "--json",
                "number,isDraft,headRefOid,headRefName,state",
            ],
        )?;
        serde_json::from_str(&out).map_err(|e| format!("gh pr list printed unexpected JSON: {e}"))
    }

    fn pr(&self, number: u64) -> Result<PrRef, String> {
        let out = crate::loopclaim::run_gh(
            &self.root,
            &[
                "pr",
                "view",
                &number.to_string(),
                "--json",
                "number,isDraft,headRefOid,headRefName,state",
            ],
        )?;
        serde_json::from_str(&out).map_err(|e| format!("gh pr view printed unexpected JSON: {e}"))
    }

    fn set_body(&self, number: u64, body: &str) -> Result<(), String> {
        let path =
            std::env::temp_dir().join(format!("uor-loop-close-{}-{number}.md", std::process::id()));
        std::fs::write(&path, body).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        let file = path.to_string_lossy().into_owned();
        let res = crate::loopclaim::run_gh(
            &self.root,
            &["pr", "edit", &number.to_string(), "--body-file", &file],
        );
        let _ = std::fs::remove_file(&path);
        res.map(|_| ())
    }

    fn mark_ready(&self, number: u64) -> Result<(), String> {
        crate::loopclaim::run_gh(&self.root, &["pr", "ready", &number.to_string()]).map(|_| ())
    }

    fn merged(&self, number: u64) -> Result<MergedPr, String> {
        #[derive(Deserialize)]
        struct Oid {
            oid: String,
        }
        #[derive(Deserialize)]
        struct View {
            state: String,
            #[serde(rename = "mergeCommit")]
            merge_commit: Option<Oid>,
            #[serde(rename = "headRefName")]
            head_ref: String,
            #[serde(rename = "headRefOid")]
            head_sha: String,
        }
        let out = crate::loopclaim::run_gh(
            &self.root,
            &[
                "pr",
                "view",
                &number.to_string(),
                "--json",
                "state,mergeCommit,headRefName,headRefOid",
            ],
        )?;
        let v: View = serde_json::from_str(&out)
            .map_err(|e| format!("gh pr view printed unexpected JSON: {e}"))?;
        Ok(MergedPr {
            state: v.state,
            merge_commit: v.merge_commit.map(|o| o.oid).unwrap_or_default(),
            head_ref: v.head_ref,
            head_sha: v.head_sha,
        })
    }
}

#[derive(Deserialize)]
struct FakePr {
    number: u64,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    #[serde(rename = "headRefOid")]
    head_sha: String,
    #[serde(default = "open_state")]
    state: String,
}

fn open_state() -> String {
    "OPEN".into()
}

#[derive(Deserialize)]
struct FakeFile {
    #[serde(default)]
    prs: Vec<FakePr>,
    #[serde(default, rename = "failSetBody")]
    fail_set_body: bool,
    #[serde(default, rename = "failReady")]
    fail_ready: bool,
    /// `--verify`'s view of each PR, keyed by its number.
    #[serde(default)]
    merged: std::collections::BTreeMap<String, MergedPr>,
}

/// The in-memory stand-in for `gh`, read from the file `UOR_LOOP_CLOSE_GH_FAKE` names.
struct FakeCloseGh {
    file: FakeFile,
    calls: RefCell<Option<std::fs::File>>,
    calls_path: PathBuf,
}

impl FakeCloseGh {
    fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "cannot read {CLOSE_GH_FAKE_ENV} file {}: {e}",
                path.display()
            )
        })?;
        let file = serde_json::from_str(&text).map_err(|e| {
            format!(
                "{CLOSE_GH_FAKE_ENV} file {} is malformed: {e}",
                path.display()
            )
        })?;
        let mut calls_path = path.as_os_str().to_owned();
        calls_path.push(".calls.jsonl");
        Ok(FakeCloseGh {
            file,
            calls: RefCell::new(None),
            calls_path: calls_path.into(),
        })
    }

    fn record(&self, call: serde_json::Value) -> Result<(), String> {
        let mut slot = self.calls.borrow_mut();
        if slot.is_none() {
            *slot = Some(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.calls_path)
                    .map_err(|e| format!("cannot open {}: {e}", self.calls_path.display()))?,
            );
        }
        let f = slot.as_mut().expect("opened above");
        writeln!(f, "{call}")
            .map_err(|e| format!("cannot write {}: {e}", self.calls_path.display()))
    }

    fn pr_ref(p: &FakePr) -> PrRef {
        PrRef {
            number: p.number,
            head_ref: p.head_ref_name.clone(),
            state: p.state.clone(),
            is_draft: p.is_draft,
            head_sha: p.head_sha.clone(),
        }
    }
}

impl CloseGh for FakeCloseGh {
    fn pr_for_branch(&self, branch: &str) -> Result<Vec<PrRef>, String> {
        Ok(self
            .file
            .prs
            .iter()
            .filter(|p| p.head_ref_name == branch)
            .map(Self::pr_ref)
            .collect())
    }

    fn pr(&self, number: u64) -> Result<PrRef, String> {
        self.file
            .prs
            .iter()
            .find(|p| p.number == number)
            .map(Self::pr_ref)
            .ok_or_else(|| format!("the fake has no PR #{number}"))
    }

    fn set_body(&self, number: u64, body: &str) -> Result<(), String> {
        if self.file.fail_set_body {
            return Err("the fake gh pr edit failed".into());
        }
        self.record(serde_json::json!({"call": "set_body", "number": number, "body": body}))
    }

    fn mark_ready(&self, number: u64) -> Result<(), String> {
        if self.file.fail_ready {
            return Err("the fake gh pr ready failed".into());
        }
        self.record(serde_json::json!({"call": "ready", "number": number}))
    }

    fn merged(&self, number: u64) -> Result<MergedPr, String> {
        self.file
            .merged
            .get(&number.to_string())
            .cloned()
            .ok_or_else(|| format!("the fake has no merged entry for PR #{number}"))
    }
}

/// The fake when `UOR_LOOP_CLOSE_GH_FAKE` is set, else `gh`.
fn close_gh(root: &Path) -> Result<Box<dyn CloseGh>, String> {
    Ok(match std::env::var_os(CLOSE_GH_FAKE_ENV) {
        Some(p) => {
            println!(
                "FAKE GitHub: {CLOSE_GH_FAKE_ENV}={} (tests only)",
                p.to_string_lossy()
            );
            Box::new(FakeCloseGh::from_file(Path::new(&p))?)
        }
        None => Box::new(RealCloseGh {
            root: root.to_path_buf(),
        }),
    })
}

/// Removes every `GIT_*` variable from `cmd` but the ones a lab may authenticate through: a caller's
/// GIT_DIR, GIT_WORK_TREE or GIT_INDEX_FILE would point git at another repository.
fn strip_git_env(cmd: &mut Command) {
    const KEEP: [&str; 3] = ["GIT_SSH_COMMAND", "GIT_SSH", "GIT_ASKPASS"];
    for (k, _) in std::env::vars_os() {
        let k_str = k.to_string_lossy();
        if k_str.starts_with("GIT_") && !KEEP.contains(&k_str.as_ref()) {
            cmd.env_remove(&k);
        }
    }
}

/// `git -C root args`, its stdout as given.
fn git_raw(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let mut cmd = Command::new("git");
    strip_git_env(&mut cmd);
    let out = cmd
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    Ok(String::from_utf8_lossy(&git_raw(root, args)?)
        .trim()
        .to_string())
}

/// The `git patch-id --verbatim` of `git diff --no-renames from to -- files` (empty for no change).
fn patch_id(root: &Path, from: &str, to: &str, files: &[&str]) -> Result<String, String> {
    let mut args = vec![
        "diff",
        "--no-renames",
        "--no-ext-diff",
        "--no-color",
        from,
        to,
        "--",
    ];
    args.extend(files);
    let diff = git_raw(root, &args)?;
    let mut cmd = Command::new("git");
    strip_git_env(&mut cmd);
    let mut child = cmd
        .arg("-C")
        .arg(root)
        .args(["patch-id", "--verbatim"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run git patch-id: {e}"))?;
    child
        .stdin
        .take()
        .expect("piped")
        .write_all(&diff)
        .map_err(|e| format!("cannot feed git patch-id: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("git patch-id failed: {e}"))?;
    if !out.status.success() {
        return Err("git patch-id failed".into());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string())
}

/// Where `loop close --session` writes the lab's PICKUP and `loop arrive` reads it:
/// `local/<lab>/PICKUP.md` under the main checkout, so it outlives the task worktree.
pub(crate) fn pickup_path(root: &Path, lab: &str) -> Result<PathBuf, String> {
    Ok(crate::loopclaim::main_checkout(root)?
        .join("local")
        .join(lab)
        .join("PICKUP.md"))
}

/// The lines of the REPORT's `## <name>` section, verbatim, without surrounding blank lines.
fn report_section(report: &str, name: &str) -> String {
    let mut inside = false;
    let mut lines = Vec::new();
    for (line, can_head) in crate::loopprcheck::heading_lines(report) {
        if can_head && line.starts_with("## ") {
            inside = line[3..].trim() == name;
            continue;
        }
        if inside {
            lines.push(line);
        }
    }
    lines.join("\n").trim_matches('\n').to_string()
}

/// The REPORT without its first `# ` title line, without leading or trailing blank lines.
fn report_without_title(report: &str) -> String {
    let mut dropped = false;
    let kept: Vec<&str> = report
        .lines()
        .filter(|l| {
            if !dropped && l.starts_with("# ") {
                dropped = true;
                return false;
            }
            true
        })
        .collect();
    kept.join("\n").trim_matches('\n').trim_end().to_string()
}

/// The issue number `docs/roadmap/issues.json` maps `item` to, `None` when there is none.
fn issue_for(root: &Path, item: Option<&str>) -> Result<Option<u64>, String> {
    let Some(item) = item else {
        return Ok(None);
    };
    let Ok(text) = std::fs::read_to_string(root.join(ISSUES)) else {
        return Ok(None);
    };
    let map: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{ISSUES} is malformed: {e}"))?;
    Ok(map.get(item).and_then(serde_json::Value::as_u64))
}

struct PickupFields<'a> {
    item: &'a str,
    lab: &'a str,
    task: &'a str,
    branch: &'a str,
    head: &'a str,
    folder: &'a str,
    still_open: &'a str,
    next_step: &'a str,
}

/// The PICKUP from the template: placeholders filled, `## Open items` holding the REPORT's
/// "Still open" verbatim, `## Next step` the next step, and `## Standing references` left empty
/// (the template's line there is an instruction, and `loop arrive` fails an uncited line).
fn pickup_text(template: &str, f: &PickupFields) -> String {
    let mut out = String::new();
    let mut section = String::new();
    for (i, line) in template.lines().enumerate() {
        if i == 0 && line.starts_with("<!--") {
            continue;
        }
        if let Some(h) = line.strip_prefix("## ") {
            section = h.trim().to_string();
            out.push_str(line);
            out.push_str("\n\n");
            match section.as_str() {
                "Open items" => out.push_str(&format!("{}\n\n", f.still_open)),
                "Next step" => out.push_str(&format!("{}\n\n", f.next_step)),
                _ => {}
            }
            continue;
        }
        if matches!(
            section.as_str(),
            "Open items" | "Next step" | "Standing references"
        ) {
            continue;
        }
        let filled = line
            .replace(
                "docs/loop/tasks/<folder>/<lab>-<n>/",
                &format!("{}/", f.folder),
            )
            .replace("<lab>-<n>", f.task)
            .replace("<item id>", f.item)
            .replace("<lab>", f.lab)
            .replace("<branch>", f.branch)
            .replace("<full sha of the last commit>", f.head);
        out.push_str(&filled);
        out.push('\n');
    }
    format!("{}\n", out.trim_end())
}

/// `loop close ...`; `args` follow the action name.
pub fn cli(root: &Path, lab: &str, args: &[String]) -> u8 {
    let (mut session, mut verify, mut dry_run, mut pr_arg) = (false, false, false, None);
    let mut no_gates = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--session" => session = true,
            "--verify" => verify = true,
            "--dry-run" => dry_run = true,
            "--no-gates" => no_gates = true,
            "--pr" => match args.get(i + 1).and_then(|v| v.parse::<u64>().ok()) {
                Some(n) => {
                    pr_arg = Some(n);
                    i += 1;
                }
                None => {
                    eprintln!("uor-harness: loop close: --pr needs a PR number\n{USAGE}");
                    return 2;
                }
            },
            other => {
                eprintln!("uor-harness: loop close: unknown argument {other:?}\n{USAGE}");
                return 2;
            }
        }
        i += 1;
    }
    if session == verify {
        eprintln!("uor-harness: loop close: give exactly one of --session, --verify\n{USAGE}");
        return 2;
    }
    if lab.is_empty() {
        eprintln!("uor-harness: loop close needs a lab (UOR_LAB or --lab)");
        return 2;
    }
    if verify {
        let Some(n) = pr_arg else {
            eprintln!("uor-harness: loop close --verify needs --pr <n>\n{USAGE}");
            return 2;
        };
        if dry_run {
            eprintln!("uor-harness: loop close --verify has no --dry-run\n{USAGE}");
            return 2;
        }
        return match verify_close(root, lab, n, no_gates) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("uor-harness: loop close --verify: {e}");
                2
            }
        };
    }
    if no_gates {
        eprintln!("uor-harness: loop close: --no-gates goes with --verify\n{USAGE}");
        return 2;
    }
    match session_close(root, lab, pr_arg, dry_run) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("uor-harness: loop close: {e}");
            2
        }
    }
}

/// Set to `1` to print `SKIP hygiene` instead of running `hygiene --dry-run` (the tests set it).
pub const CLOSE_NO_HYGIENE_ENV: &str = "UOR_LOOP_CLOSE_NO_HYGIENE";

/// The registry file a lab's worktree may hold a stale edit to; `--verify` reverts that edit alone.
const CLAIMS: &str = "docs/harness/claims.json";

fn step(verdict: &str, name: &str, detail: &str) {
    println!("{verdict} {name} {detail}");
}

/// The current executable with every `GIT_*` variable removed.
fn harness(dir: &Path) -> Result<Command, String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot find this executable: {e}"))?;
    let mut cmd = Command::new(exe);
    strip_git_env(&mut cmd);
    cmd.current_dir(dir);
    Ok(cmd)
}

/// The proofs that the PR is merged into `origin/main` as itself: `state`, `squash`, `content`.
/// `Ok(())` when all pass; `Err` names the first that failed (and has been printed).
fn prove_merge(main: &Path, n: u64, m: &MergedPr) -> Result<(), String> {
    if m.state != "MERGED" {
        step(
            "FAIL",
            "state",
            &format!("GitHub reports {}, not MERGED", m.state),
        );
        return Err("state".into());
    }
    step("PASS", "state", "MERGED");
    let mc = m.merge_commit.as_str();
    let squash = (|| {
        if mc.is_empty() {
            return Err("GitHub names no merge commit".to_string());
        }
        git(main, &["fetch", "-q", "origin", "main"])?;
        git(
            main,
            &[
                "merge-base",
                "--is-ancestor",
                mc,
                "refs/remotes/origin/main",
            ],
        )
        .map_err(|_| format!("{mc} is not on origin/main"))?;
        let subject = git(main, &["log", "-1", "--format=%s", mc])?;
        if !subject.ends_with(&format!("(#{n})")) {
            return Err(format!("the subject {subject:?} does not end with (#{n})"));
        }
        Ok(format!("{mc} on origin/main: {subject}"))
    })();
    match squash {
        Ok(d) => step("PASS", "squash", &d),
        Err(e) => {
            step("FAIL", "squash", &e);
            return Err("squash".into());
        }
    }
    let content = (|| {
        let head = m.head_sha.as_str();
        git(
            main,
            &["fetch", "-q", "origin", &format!("refs/pull/{n}/head")],
        )?;
        git(main, &["cat-file", "-e", &format!("{head}^{{commit}}")])
            .map_err(|_| format!("the PR head {head} is not in this repository"))?;
        let parent = format!("{mc}^1");
        let base = git(main, &["merge-base", head, &parent])?;
        let files = git(main, &["diff", "--name-only", "--no-renames", &base, head])?;
        let files: Vec<&str> = files.lines().filter(|l| !l.is_empty()).collect();
        if files.is_empty() {
            return Err("the PR changes no file".to_string());
        }
        let mut args = vec!["diff", "--quiet", "--no-renames", mc, head, "--"];
        args.extend(&files);
        if git(main, &args).is_ok() {
            return Ok(format!("{} changed files identical on main", files.len()));
        }
        // Main may have changed another part of a shared file before the squash: then the PR's
        // patch must be the merge's.
        let pr = patch_id(main, &base, head, &files)?;
        let merge = patch_id(main, &parent, mc, &files)?;
        if pr.is_empty() || pr != merge {
            return Err(format!(
                "a file the PR changed differs between {mc} and the PR head {head}, and the \
                 merge's patch is not the PR's"
            ));
        }
        Ok(format!(
            "{} changed files: the merge's patch is the PR's (patch-id {pr})",
            files.len()
        ))
    })();
    match content {
        Ok(d) => step("PASS", "content", &d),
        Err(e) => {
            step("FAIL", "content", &e);
            return Err("content".into());
        }
    }
    Ok(())
}

/// `loop gates --full` in a detached worktree of `origin/main` at `local/<lab>/verify-<n>`, removed
/// afterwards whatever the result, once a red run's gate logs are kept (`keep_red_logs`). `true`
/// when the gates were green.
fn done_when(main: &Path, lab: &str, n: u64) -> bool {
    let path = main
        .join("local")
        .join(crate::state::slug(lab))
        .join(format!("verify-{n}"));
    let shown = path.display().to_string();
    if path.exists() {
        step(
            "FAIL",
            "done-when",
            &format!("{shown} already exists; remove it first"),
        );
        return false;
    }
    if let Err(e) = git(
        main,
        &[
            "worktree",
            "add",
            "-q",
            "--detach",
            &shown,
            "refs/remotes/origin/main",
        ],
    ) {
        step("FAIL", "done-when", &e);
        return false;
    }
    let run = harness(&path).and_then(|mut cmd| {
        cmd.env("UOR_LAB", lab)
            .args(["--repo", &shown, "--lab", lab, "loop", "gates", "--full"])
            .status()
            .map_err(|e| format!("cannot run loop gates: {e}"))
    });
    if matches!(&run, Ok(s) if !s.success()) {
        keep_red_logs(main, &path, lab, n);
    }
    let removed = git(main, &["worktree", "remove", "--force", &shown]);
    let green = match run {
        Ok(s) if s.success() => {
            step(
                "PASS",
                "done-when",
                "loop gates --full is green on origin/main",
            );
            true
        }
        Ok(s) => {
            step(
                "FAIL",
                "done-when",
                &format!("loop gates --full exited {s} on origin/main: a failure becomes a repair"),
            );
            false
        }
        Err(e) => {
            step("FAIL", "done-when", &e);
            false
        }
    };
    if let Err(e) = removed {
        step("FAIL", "done-when", &format!("cannot remove {shown}: {e}"));
        return false;
    }
    green
}

/// Copies each red gate's `<gate>.log` and `<gate>.exit` from the verify worktree's newest gate run
/// (`local/<lab>/gates/<run>/`) to `<main>/local/<lab>/verify-logs/<n>/<run>/`, before the worktree
/// is removed with them inside. A red gate is one whose `.exit` is not `0`. Prints where they went;
/// a failure to copy is printed and changes nothing else.
fn keep_red_logs(main: &Path, verify: &Path, lab: &str, n: u64) {
    let slug = crate::state::slug(lab);
    let runs = verify.join("local").join(&slug).join("gates");
    let kept = (|| -> Result<Option<(PathBuf, Vec<String>)>, String> {
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(&runs)
            .map_err(|e| format!("cannot read {}: {e}", runs.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        let Some(run) = dirs.pop() else {
            return Ok(None);
        };
        let mut red: Vec<String> = std::fs::read_dir(&run)
            .map_err(|e| format!("cannot read {}: {e}", run.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "exit"))
            .filter(|p| std::fs::read_to_string(p).is_ok_and(|t| t.trim() != "0"))
            .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .collect();
        red.sort();
        if red.is_empty() {
            return Ok(None);
        }
        let dest = main
            .join("local")
            .join(&slug)
            .join("verify-logs")
            .join(n.to_string())
            .join(run.file_name().unwrap_or_default());
        std::fs::create_dir_all(&dest)
            .map_err(|e| format!("cannot create {}: {e}", dest.display()))?;
        for gate in &red {
            for ext in ["log", "exit"] {
                let from = run.join(format!("{gate}.{ext}"));
                if from.is_file() {
                    std::fs::copy(&from, dest.join(format!("{gate}.{ext}")))
                        .map_err(|e| format!("cannot copy {}: {e}", from.display()))?;
                }
            }
        }
        Ok(Some((dest, red)))
    })();
    match kept {
        Ok(Some((dest, red))) => step(
            "KEEP",
            "done-when",
            &format!(
                "red gate log(s) {} copied to {}",
                red.join(", "),
                dest.display()
            ),
        ),
        Ok(None) => {}
        Err(e) => step("NOTE", "done-when", &format!("red gate logs not kept: {e}")),
    }
}

/// The worktree checked out on `refs/heads/<branch>`, from `git worktree list --porcelain`.
fn worktree_of(main: &Path, branch: &str) -> Result<Option<PathBuf>, String> {
    let list = git(main, &["worktree", "list", "--porcelain"])?;
    let want = format!("branch refs/heads/{branch}");
    let mut path = None;
    for line in list.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            path = Some(PathBuf::from(p));
        } else if line == want {
            return Ok(path);
        }
    }
    Ok(None)
}

/// Step 5: remove this lab's clean worktree on `branch`. `Err` names what was kept.
fn remove_worktree(main: &Path, lab: &str, branch: &str) -> Result<(), String> {
    let Some(wt) = worktree_of(main, branch)? else {
        step("N/A", "worktree", &format!("no worktree on {branch}"));
        return Ok(());
    };
    let shown = wt.display().to_string();
    let dir = main.join(".worktrees");
    let parent_ok = match (wt.parent().map(Path::canonicalize), dir.canonicalize()) {
        (Some(Ok(p)), Ok(d)) => p == d,
        _ => false,
    };
    let name_ok = wt
        .file_name()
        .is_some_and(|f| f.to_string_lossy().starts_with(&format!("{lab}-")));
    if !(parent_ok && name_ok) {
        step(
            "FAIL",
            "worktree",
            &format!("{shown} is not under .worktrees/{lab}-*; kept"),
        );
        return Err(format!("the worktree {shown}"));
    }
    // `git` trims its output, so the first line may have lost its leading space: the path is what
    // follows the first space after the status letters.
    let status = |wt: &Path| git(wt, &["status", "--porcelain=v1", "--untracked-files=all"]);
    let mut dirty = status(&wt)?;
    if !dirty.is_empty()
        && dirty
            .lines()
            .all(|l| l.trim_start().split_once(' ').map(|(_, p)| p) == Some(CLAIMS))
    {
        git(&wt, &["checkout", "--", CLAIMS])?;
        dirty = status(&wt)?;
    }
    if !dirty.is_empty() {
        step(
            "FAIL",
            "worktree",
            &format!("{shown} has uncommitted changes; kept"),
        );
        return Err(format!("the worktree {shown}"));
    }
    // `git worktree remove` deletes ignored files too: keep a worktree holding any but the
    // disposable build and view output.
    let held = ignored_files(&wt, lab)?;
    if !held.is_empty() {
        let first: Vec<&str> = held.iter().take(10).map(String::as_str).collect();
        step(
            "FAIL",
            "worktree",
            &format!("holds ignored files: {}; kept", first.join(", ")),
        );
        return Err(format!("the worktree {shown}"));
    }
    match git(main, &["worktree", "remove", &shown]) {
        Ok(_) => {
            step("PASS", "worktree", &format!("removed {shown}"));
            Ok(())
        }
        Err(e) => {
            step("FAIL", "worktree", &format!("{e}; kept"));
            Err(format!("the worktree {shown}"))
        }
    }
}

/// The ignored files in `wt` other than `target/`, `local/target/`, `local/harness-target/`,
/// `local/<lab>/views/`, `local/<lab>/gates/` and `local/<lab>/gates.jsonl`, sorted. A wholly
/// ignored directory is listed as itself unless a disposable path lies inside it.
fn ignored_files(wt: &Path, lab: &str) -> Result<Vec<String>, String> {
    let views = format!("local/{lab}/views/");
    let gates_dir = format!("local/{lab}/gates/");
    let gates_file = format!("local/{lab}/gates.jsonl");
    let disposable_prefixes = [
        "target/",
        "local/target/",
        "local/harness-target/",
        views.as_str(),
        gates_dir.as_str(),
    ];
    let disposable = |p: &str| {
        p == gates_file
            || disposable_prefixes
                .iter()
                .any(|prefix| p.starts_with(prefix))
    };
    let list = |extra: &[&str]| -> Result<Vec<String>, String> {
        let mut args = vec!["ls-files", "--others", "--ignored", "--exclude-standard"];
        args.extend(extra);
        Ok(git(wt, &args)?
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect())
    };
    let mut held = Vec::new();
    for p in list(&["--directory"])? {
        if disposable(&p) {
            continue;
        }
        if p.ends_with('/')
            && (gates_file.starts_with(&p)
                || disposable_prefixes
                    .iter()
                    .any(|prefix| prefix.starts_with(&p)))
        {
            // A directory holding a disposable path: look at its files one by one.
            held.extend(list(&["--", &p])?.into_iter().filter(|f| !disposable(f)));
        } else {
            held.push(p);
        }
    }
    held.sort();
    Ok(held)
}

/// Step 6: delete the local branch only when its tip is the PR head and no worktree has it out.
fn delete_local_branch(main: &Path, branch: &str, head: &str) -> Result<(), String> {
    let Ok(tip) = git(
        main,
        &[
            "rev-parse",
            "--verify",
            "-q",
            &format!("refs/heads/{branch}"),
        ],
    ) else {
        step("N/A", "local-branch", &format!("no local branch {branch}"));
        return Ok(());
    };
    if let Some(wt) = worktree_of(main, branch)? {
        step(
            "FAIL",
            "local-branch",
            &format!("{branch} is checked out in {}; kept", wt.display()),
        );
        return Err(format!("the local branch {branch}"));
    }
    // One atomic compare-and-delete: it fails when the branch is no longer at the PR head.
    let refname = format!("refs/heads/{branch}");
    match git(main, &["update-ref", "-d", &refname, head]) {
        Ok(_) => {
            // `git branch -D` also drops the branch's config section; an absent one is fine.
            let _ = git(
                main,
                &["config", "--remove-section", &format!("branch.{branch}")],
            );
            step(
                "PASS",
                "local-branch",
                &format!("deleted {branch} at {head}"),
            );
            Ok(())
        }
        Err(_) => {
            step(
                "FAIL",
                "local-branch",
                &format!("{branch} is at {tip}, not the PR head {head}; kept"),
            );
            Err(format!("the local branch {branch}"))
        }
    }
}

/// Step 7: delete the remote branch with a lease on the PR head, so a branch that moved is kept.
fn delete_remote_branch(main: &Path, branch: &str, head: &str) -> Result<(), String> {
    let refname = format!("refs/heads/{branch}");
    let listed = git(main, &["ls-remote", "origin", &refname])?;
    if listed.is_empty() {
        step("N/A", "remote-branch", &format!("origin has no {branch}"));
        return Ok(());
    }
    let lease = format!("--force-with-lease={refname}:{head}");
    match git(
        main,
        &["push", "-q", "origin", &lease, &format!(":{refname}")],
    ) {
        Ok(_) => {
            step(
                "PASS",
                "remote-branch",
                &format!("deleted origin {branch} at {head}"),
            );
            Ok(())
        }
        Err(e) => {
            step("FAIL", "remote-branch", &format!("{e}; kept"));
            Err(format!("the remote branch {branch}"))
        }
    }
}

/// Step 8: print hygiene's plan, never `--apply`.
///
/// Loop step 13 says to run `uor-hygiene --apply`. This prints the dry-run plan instead, a decision
/// recorded in the REPORT: hygiene's `--apply` removes every clean merged worktree in the checkout,
/// other labs' included, and the lab boundary forbids touching another session's files. The lab
/// reads the plan and removes only what is its own.
fn hygiene_plan(main: &Path) {
    if std::env::var(CLOSE_NO_HYGIENE_ENV).as_deref() == Ok("1") {
        step("SKIP", "hygiene", &format!("{CLOSE_NO_HYGIENE_ENV}=1"));
        return;
    }
    let shown = main.display().to_string();
    let run = harness(main).and_then(|mut cmd| {
        cmd.args(["--repo", &shown, "hygiene", "--dry-run"])
            .status()
            .map_err(|e| format!("cannot run hygiene: {e}"))
    });
    match run {
        Ok(s) if s.success() => step(
            "PASS",
            "hygiene",
            "the dry-run plan is above; nothing applied",
        ),
        Ok(s) => step("FAIL", "hygiene", &format!("hygiene --dry-run exited {s}")),
        Err(e) => step("FAIL", "hygiene", &e),
    }
}

/// `--verify --pr <n>`. `Err` is a cannot-run (exit 2).
fn verify_close(root: &Path, lab: &str, n: u64, no_gates: bool) -> Result<u8, String> {
    let main = crate::loopclaim::main_checkout(root)?;
    let gh = close_gh(&main)?;
    let m = gh.merged(n)?;
    let mut proved = prove_merge(&main, n, &m).is_ok();
    if !proved {
        // An earlier proof failed: the gates would prove nothing about this PR.
        step("SKIP", "done-when", "an earlier proof failed");
    } else if no_gates {
        step("SKIP", "done-when", "--no-gates");
    } else {
        proved = done_when(&main, lab, n);
    }
    if !proved {
        println!("not verified: nothing deleted");
        return Ok(1);
    }
    let mut kept = Vec::new();
    if m.head_ref.starts_with(&format!("{lab}/")) {
        for r in [
            remove_worktree(&main, lab, &m.head_ref),
            delete_local_branch(&main, &m.head_ref, &m.head_sha),
            delete_remote_branch(&main, &m.head_ref, &m.head_sha),
        ] {
            if let Err(what) = r {
                kept.push(what);
            }
        }
    } else {
        step(
            "SKIP",
            "cleanup",
            &format!("not this lab's branch ({}); nothing deleted", m.head_ref),
        );
    }
    hygiene_plan(&main);
    if kept.is_empty() {
        println!("verified: PR #{n} is on main");
        Ok(0)
    } else {
        println!("verified, but kept: {}", kept.join(", "));
        Ok(1)
    }
}

/// `--session`. `Err` is a cannot-run (exit 2); a refusal prints its reasons and returns 1.
fn session_close(root: &Path, lab: &str, pr_arg: Option<u64>, dry_run: bool) -> Result<u8, String> {
    // 1. The task folder.
    let order_rel = crate::loopreport::find_order(root)?;
    let order_path = root.join(&order_rel);
    let order_text = std::fs::read_to_string(&order_path)
        .map_err(|e| format!("cannot read {order_rel}: {e}"))?;
    let folder = order_path.parent().unwrap_or(root).to_path_buf();
    let shown = folder
        .strip_prefix(root)
        .unwrap_or(&folder)
        .display()
        .to_string();

    // 2. The REPORT passes `loop report --check`.
    let problems = crate::loopreport::report_file_problems(&folder, &shown);
    if !problems.is_empty() {
        println!("REFUSED close: {shown}/REPORT.md fails `loop report --check`");
        for p in problems {
            println!("  {p}");
        }
        return Ok(1);
    }
    let report = std::fs::read_to_string(folder.join("REPORT.md"))
        .map_err(|e| format!("cannot read {shown}/REPORT.md: {e}"))?;

    // 3. The PR, at HEAD.
    let branch = git(root, &["symbolic-ref", "--short", "HEAD"])
        .map_err(|e| format!("not on a branch: {e}"))?;
    let head = git(root, &["rev-parse", "HEAD"])?;
    let gh = close_gh(root)?;
    let pr = match pr_arg {
        Some(n) => {
            let pr = gh.pr(n)?;
            if pr.state != "OPEN" || pr.head_ref != branch {
                println!(
                    "REFUSED close: PR #{n} is {} with head branch {}; it must be OPEN with head \
                     branch {branch}",
                    pr.state, pr.head_ref
                );
                return Ok(1);
            }
            pr
        }
        None => {
            let mut prs = gh.pr_for_branch(&branch)?;
            match prs.len() {
                1 => prs.remove(0),
                0 => {
                    return Err(format!(
                        "no open PR has head branch {branch}; pass --pr <n>"
                    ))
                }
                n => {
                    let nums: Vec<String> = prs.iter().map(|p| format!("#{}", p.number)).collect();
                    return Err(format!(
                        "{n} open PRs have head branch {branch} ({}); pass --pr <n>",
                        nums.join(", ")
                    ));
                }
            }
        }
    };
    if pr.head_sha != head {
        println!(
            "REFUSED close: push first: the PR head is {}, HEAD is {head}",
            pr.head_sha
        );
        return Ok(1);
    }

    // 4. The body.
    let merge_base = git(root, &["merge-base", "origin/main", "HEAD"])?;
    let size = crate::loopprcheck::size_line(root, &merge_base, &head)?;
    let item = crate::loopclaim::order_item(&order_text);
    let mut body = format!("{size}\n\n{}\n", report_without_title(&report));
    if let Some(issue) = issue_for(root, item.as_deref())? {
        body.push_str(&format!("\nReferences #{issue}\n"));
    }
    if lab == "claude" {
        body.push_str(&format!("\n{FOOTER}\n"));
    }

    // 5. The PICKUP, checked before any write.
    let template = std::fs::read_to_string(root.join(PICKUP_TEMPLATE))
        .map_err(|e| format!("cannot read {PICKUP_TEMPLATE}: {e}"))?;
    let task = folder
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let item_shown = item.clone().unwrap_or_else(|| {
        folder
            .parent()
            .and_then(Path::file_name)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    });
    let still_open = report_section(&report, "Still open");
    let next_step = format!(
        "Await the exact-head review of PR #{} at `{head}`.",
        pr.number
    );
    let pickup = pickup_text(
        &template,
        &PickupFields {
            item: &item_shown,
            lab,
            task: &task,
            branch: &branch,
            head: &head,
            folder: &shown,
            still_open: &still_open,
            next_step: &next_step,
        },
    );
    let max = crate::looparrive::PICKUP_MAX;
    if pickup.len() >= max {
        println!(
            "REFUSED close: the PICKUP would be {} bytes; a PICKUP must stay under {max} \
             (shorten the REPORT's \"Still open\")",
            pickup.len()
        );
        return Ok(1);
    }
    let pickup_path = pickup_path(root, lab)?;
    let pickup_rel = pickup_path.display().to_string();

    if dry_run {
        println!("--- PR #{} body ---", pr.number);
        print!("{body}");
        println!("--- {pickup_rel} ---");
        print!("{pickup}");
        println!("--- dry run: nothing written ---");
        return Ok(0);
    }

    // 6. Writes: body, ready, PICKUP.
    gh.set_body(pr.number, &body).map_err(|e| {
        format!(
            "setting the body of PR #{} failed: {e}; nothing written",
            pr.number
        )
    })?;
    if pr.is_draft {
        gh.mark_ready(pr.number).map_err(|e| {
            format!(
                "the body of PR #{} was already set, but marking it ready failed: {e}; \
                 no PICKUP written",
                pr.number
            )
        })?;
    }
    if let Some(dir) = pickup_path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    std::fs::write(&pickup_path, &pickup).map_err(|e| format!("cannot write {pickup_rel}: {e}"))?;
    println!(
        "close: set the body of PR #{} ({} bytes); {}; wrote {pickup_rel} ({} bytes)",
        pr.number,
        body.len(),
        if pr.is_draft {
            "marked it ready"
        } else {
            "it was already ready"
        },
        pickup.len()
    );
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pickup_fills_the_template() {
        let t = "<!-- c -->\n# PICKUP: <item id>, <lab>-<n>\n\n- Branch: `<branch>`\n- Head: `<full sha of the last commit>`\n- Task folder: `docs/loop/tasks/<folder>/<lab>-<n>/`\n\n## Open items\n\n- x\n\n## Next step\n\ny\n\n## Standing references\n\nz\n";
        let p = pickup_text(
            t,
            &PickupFields {
                item: "it",
                lab: "claude",
                task: "claude-1",
                branch: "b",
                head: "h",
                folder: "docs/loop/tasks/it/claude-1",
                still_open: "- open",
                next_step: "go",
            },
        );
        assert_eq!(
            p,
            "# PICKUP: it, claude-1\n\n- Branch: `b`\n- Head: `h`\n- Task folder: `docs/loop/tasks/it/claude-1/`\n\n## Open items\n\n- open\n\n## Next step\n\ngo\n\n## Standing references\n"
        );
    }

    #[test]
    fn ignored_files_treats_gate_logs_and_local_targets_as_disposable() {
        let dir = std::env::temp_dir().join(format!(
            "uor-loopclose-ignored-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        git(&dir, &["init", "-q"]).expect("git init");
        std::fs::write(dir.join(".gitignore"), "target/\nlocal/\n").expect("gitignore");
        std::fs::create_dir_all(dir.join("target/debug")).expect("target");
        std::fs::write(dir.join("target/debug/bin"), "x").expect("target file");
        std::fs::create_dir_all(dir.join("local/target/release")).expect("local/target");
        std::fs::write(dir.join("local/target/release/bin"), "x").expect("local/target file");
        std::fs::create_dir_all(dir.join("local/harness-target/debug")).expect("harness-target");
        std::fs::write(dir.join("local/harness-target/debug/bin"), "x")
            .expect("harness-target file");
        std::fs::create_dir_all(dir.join("local/antigravity/views")).expect("views");
        std::fs::write(dir.join("local/antigravity/views/v.md"), "x").expect("view file");
        std::fs::write(dir.join("local/antigravity/gates.jsonl"), "{}\n").expect("gates.jsonl");
        std::fs::create_dir_all(dir.join("local/antigravity/gates/123-456")).expect("gates dir");
        std::fs::write(
            dir.join("local/antigravity/gates/123-456/fmt-harness.log"),
            "ok\n",
        )
        .expect("gate log");
        std::fs::write(
            dir.join("local/antigravity/gates/123-456/fmt-harness.exit"),
            "0\n",
        )
        .expect("gate exit");

        let held = ignored_files(&dir, "antigravity").expect("ignored_files");
        assert!(
            held.is_empty(),
            "disposable build/gate paths should not block worktree removal, got: {held:?}"
        );

        std::fs::write(dir.join("local/antigravity/unarchived-receipt.json"), "{}")
            .expect("non-disposable receipt");
        let held_with_data = ignored_files(&dir, "antigravity").expect("ignored_files with data");
        assert_eq!(
            held_with_data,
            vec!["local/antigravity/unarchived-receipt.json".to_string()]
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
