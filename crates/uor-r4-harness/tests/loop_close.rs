//! `uor-harness loop close --session` against temporary git repositories: a bare `origin` whose
//! `main` holds the real REPORT and PICKUP templates and `docs/roadmap/issues.json`, a branch that
//! adds an ORDER and a REPORT, and `UOR_LOOP_CLOSE_GH_FAKE` in place of `gh`. Every child process has
//! every `GIT_*` variable removed, and each repository is checked to sit inside its temporary
//! directory. No test touches the network or the real `gh`.

use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const PICKUP_TEMPLATE: &str = include_str!("../../../docs/loop/templates/PICKUP.md");
const REPORT_TEMPLATE: &str = include_str!("../../../docs/loop/templates/REPORT.md");
const ORDER: &str = "docs/loop/tasks/item-a/claude-1/ORDER.md";
const REPORT: &str = "docs/loop/tasks/item-a/claude-1/REPORT.md";
const PICKUP: &str = "local/claude/PICKUP.md";
const STILL_OPEN: &str = "- The verify half, tracked in #77.";

fn valid_report() -> String {
    format!(
        "# REPORT: item-a, claude-1\n\nLab: claude.\n\n## What did not work, first\n\nNothing.\n\n## What changed\n\nThe close step.\n\n## Measurements\n\nNone.\n\n## Checks run at the head\n\n- tests\n\n## Skills used\n\nNone.\n\n## Still open\n\n{STILL_OPEN}\n\n## Plain summary\n\nDone.\n"
    )
}

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB")
        .env_remove("UOR_LOOP_CLOSE_GH_FAKE")
}

struct Repo {
    _tmp: tempfile::TempDir,
    base: PathBuf,
    root: PathBuf,
}

impl Repo {
    /// `main` (pushed to a bare origin) holds the templates and the issue map; `feat` adds an
    /// ORDER and the REPORT text `report`.
    fn new(report: &str) -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().canonicalize().unwrap();
        let root = base.join("repo");
        let origin = base.join("origin.git");
        assert!(root.starts_with(&base) && origin.starts_with(&base));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo {
            _tmp: tmp,
            base,
            root,
        };
        r.git(&["init", "-q", "--bare", origin.to_str().unwrap()]);
        r.git(&["init", "-q", "-b", "main"]);
        r.write("docs/loop/templates/PICKUP.md", PICKUP_TEMPLATE);
        r.write("docs/loop/templates/REPORT.md", REPORT_TEMPLATE);
        r.write(
            "docs/harness/labs.json",
            r#"{"labs": {"claude": {"vendor": "a", "client": "b", "headless": "c", "open_pr_limit": 2}}}"#,
        );
        r.write("docs/roadmap/issues.json", "{\"item-a\": 4242}\n");
        r.write(".gitignore", ".worktrees/\nlocal/\n");
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "base"]);
        r.git(&["remote", "add", "origin", origin.to_str().unwrap()]);
        r.git(&["push", "-q", "origin", "main"]);
        r.git(&["fetch", "-q", "origin"]);
        r.git(&["checkout", "-q", "-b", "feat"]);
        r.write("crates/x.rs", "// work\n");
        r.write(
            ORDER,
            "# ORDER: item-a, claude-1 (t)\n\nLab: claude. Item: [item-a](../../../../roadmap/items/item-a.md) (#1).\n",
        );
        r.write(REPORT, report);
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "order"]);
        r
    }

    fn git(&self, args: &[&str]) -> String {
        let o = clean(&mut Command::new("git"))
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs");
        assert!(
            o.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn head(&self) -> String {
        self.git(&["rev-parse", "HEAD"])
    }

    fn fake_path(&self) -> PathBuf {
        self.base.join("gh.json")
    }

    /// One PR for `feat` at `head`, plus `extra` top-level fields.
    fn fake(&self, head: &str, draft: bool, extra: &str) {
        std::fs::write(
            self.fake_path(),
            format!(
                "{{\"prs\": [{{\"number\": 7, \"headRefName\": \"feat\", \"isDraft\": {draft}, \"headRefOid\": \"{head}\"}}]{extra}}}"
            ),
        )
        .unwrap();
    }

    fn calls(&self) -> Vec<serde_json::Value> {
        let mut p = self.fake_path().into_os_string();
        p.push(".calls.jsonl");
        std::fs::read_to_string(PathBuf::from(p))
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn pickup(&self) -> Option<String> {
        std::fs::read_to_string(self.root.join(PICKUP)).ok()
    }

    fn close(&self, lab: Option<&str>, args: &[&str]) -> (i32, String) {
        self.close_in(&self.root, lab, args)
    }

    fn close_in(&self, dir: &std::path::Path, lab: Option<&str>, args: &[&str]) -> (i32, String) {
        assert!(dir.starts_with(&self.base));
        let mut cmd = Command::new(BIN);
        clean(&mut cmd);
        if let Some(l) = lab {
            cmd.env("UOR_LAB", l);
        }
        cmd.env("UOR_LOOP_CLOSE_GH_FAKE", self.fake_path());
        cmd.env("UOR_LOOP_CLOSE_NO_HYGIENE", "1");
        let o: Output = cmd
            .args(["--repo", dir.to_str().unwrap(), "loop", "close"])
            .args(args)
            .current_dir(dir)
            .output()
            .expect("the binary runs");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
        (o.status.code().unwrap_or(-1), text)
    }
}

#[test]
fn a_complete_report_on_a_draft_pr_at_head_sets_the_body_marks_ready_and_writes_pickup() {
    let r = Repo::new(&valid_report());
    let head = r.head();
    r.fake(&head, true, "");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 0, "{out}");
    let calls = r.calls();
    let kinds: Vec<&str> = calls.iter().map(|c| c["call"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["set_body", "ready"], "{out}");
    assert_eq!(calls[1]["number"], 7);
    let body = calls[0]["body"].as_str().unwrap();
    assert!(body.lines().next().unwrap().starts_with("size:"), "{body}");
    assert!(body.contains("## Still open"), "{body}");
    assert!(body.contains("The close step."), "{body}");
    assert!(!body.contains("# REPORT: item-a"), "{body}");
    assert!(body.contains("References #4242"), "{body}");
    assert!(
        body.trim_end()
            .ends_with("🤖 Generated with [Claude Code](https://claude.com/claude-code)"),
        "{body}"
    );
    let pickup = r.pickup().expect("PICKUP written");
    assert!(pickup.contains("`feat`"), "{pickup}");
    assert!(pickup.contains(&head), "{pickup}");
    assert!(pickup.contains(STILL_OPEN), "{pickup}");
    assert!(pickup.contains("exact-head review"), "{pickup}");
    assert!(
        pickup.contains("docs/loop/tasks/item-a/claude-1/"),
        "{pickup}"
    );
}

#[test]
fn a_report_missing_a_section_is_refused_and_nothing_is_written() {
    let r = Repo::new(&valid_report().replace("## Skills used\n\nNone.\n\n", ""));
    r.fake(&r.head(), true, "");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("Skills used"), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn a_pr_head_other_than_head_says_push_first() {
    let r = Repo::new(&valid_report());
    r.fake("0123456789012345678901234567890123456789", true, "");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("push first"), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn a_ready_pr_gets_the_body_only() {
    let r = Repo::new(&valid_report());
    r.fake(&r.head(), false, "");
    let (code, out) = r.close(Some("claude"), &["--session", "--pr", "7"]);
    assert_eq!(code, 0, "{out}");
    let kinds: Vec<String> = r
        .calls()
        .iter()
        .map(|c| c["call"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(kinds, ["set_body"], "{out}");
    assert!(r.pickup().is_some());
}

#[test]
fn dry_run_prints_and_writes_nothing() {
    let r = Repo::new(&valid_report());
    r.fake(&r.head(), true, "");
    let (code, out) = r.close(Some("claude"), &["--session", "--dry-run"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("FAKE GitHub: UOR_LOOP_CLOSE_GH_FAKE=") && out.contains("(tests only)"),
        "{out}"
    );
    assert!(out.contains("size:"), "{out}");
    assert!(out.contains("References #4242"), "{out}");
    assert!(out.contains(STILL_OPEN), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn a_pr_named_with_another_head_branch_is_refused_and_nothing_is_written() {
    let r = Repo::new(&valid_report());
    std::fs::write(
        r.fake_path(),
        format!(
            "{{\"prs\": [{{\"number\": 7, \"headRefName\": \"other\", \"isDraft\": true, \"headRefOid\": \"{}\"}}]}}",
            r.head()
        ),
    )
    .unwrap();
    let (code, out) = r.close(Some("claude"), &["--session", "--pr", "7"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("head branch other"), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn a_session_in_a_linked_worktree_writes_the_pickup_under_the_main_checkout_and_it_survives_verify()
{
    let r = Repo::new(&valid_report());
    let wt = r.root.join(".worktrees/claude-x");
    r.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "claude/x",
        wt.to_str().unwrap(),
        "feat",
    ]);
    let head = r.head();
    r.git(&[
        "push",
        "-q",
        "origin",
        "claude/x",
        "claude/x:refs/pull/7/head",
    ]);
    let mc = r.git(&[
        "commit-tree",
        &format!("{head}^{{tree}}"),
        "-p",
        "origin/main",
        "-m",
        "work (#7)",
    ]);
    r.git(&["push", "-q", "origin", &format!("{mc}:refs/heads/main")]);
    std::fs::write(
        r.fake_path(),
        format!(
            r#"{{"prs": [{{"number": 7, "headRefName": "claude/x", "isDraft": false, "headRefOid": "{head}"}}], "merged": {{"7": {{"state": "MERGED", "mergeCommit": "{mc}", "headRefName": "claude/x", "headRefOid": "{head}"}}}}}}"#
        ),
    )
    .unwrap();
    let (code, out) = r.close_in(&wt, Some("claude"), &["--session"]);
    assert_eq!(code, 0, "{out}");
    assert!(r.pickup().is_some(), "{out}");
    assert!(!wt.join(PICKUP).exists(), "{out}");

    let (code, out) = r.close(Some("claude"), &["--verify", "--pr", "7", "--no-gates"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("PASS worktree"), "{out}");
    assert!(!wt.exists(), "{out}");
    assert!(r.pickup().is_some(), "{out}");
}

#[test]
fn a_pickup_of_8192_bytes_or_more_is_refused_before_any_write() {
    let huge = format!("- {}\n", "x".repeat(9000));
    let r = Repo::new(&valid_report().replace(STILL_OPEN, &huge));
    r.fake(&r.head(), true, "");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("8192"), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn a_failed_body_write_stops_before_ready_and_pickup() {
    let r = Repo::new(&valid_report());
    r.fake(&r.head(), true, ", \"failSetBody\": true");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 2, "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn a_failed_ready_says_the_body_was_already_set() {
    let r = Repo::new(&valid_report());
    r.fake(&r.head(), true, ", \"failReady\": true");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("already set"), "{out}");
    let kinds: Vec<String> = r
        .calls()
        .iter()
        .map(|c| c["call"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(kinds, ["set_body"], "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn no_lab_no_pr_and_verify_without_pr_cannot_run() {
    let r = Repo::new(&valid_report());
    r.fake(&r.head(), true, "");
    let (code, out) = r.close(None, &["--session"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("needs a lab"), "{out}");

    std::fs::write(r.fake_path(), "{\"prs\": []}").unwrap();
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("no open PR"), "{out}");

    let (code, out) = r.close(Some("claude"), &["--verify"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("needs --pr"), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

// ---- `--verify` -------------------------------------------------------------------------------

/// A main checkout at `<tmp>/main` with a bare `origin`, the worktree `.worktrees/claude-x` on
/// `claude/x` (pushed, and as `refs/pull/7/head`), and `docs/harness/gates.json` with one gate
/// running `gate_cmd`.
struct Merged {
    _tmp: tempfile::TempDir,
    base: PathBuf,
    main: PathBuf,
    wt: PathBuf,
    head: String,
}

impl Merged {
    fn new(gate_cmd: &str) -> Merged {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().canonicalize().unwrap();
        let main = base.join("main");
        let origin = base.join("origin.git");
        let wt = main.join(".worktrees/claude-x");
        assert!(main.starts_with(&base) && origin.starts_with(&base) && wt.starts_with(&base));
        std::fs::create_dir_all(&main).unwrap();
        let mut m = Merged {
            _tmp: tmp,
            base,
            main,
            wt,
            head: String::new(),
        };
        let o = origin.to_str().unwrap().to_string();
        m.git(&["init", "-q", "--bare", &o]);
        m.git(&["init", "-q", "-b", "main"]);
        m.write("a.txt", "a\n");
        m.write(".gitignore", ".worktrees/\nlocal/\ntarget/\n");
        m.write("docs/harness/claims.json", "{}\n");
        m.write(
            "docs/harness/labs.json",
            r#"{"labs": {"claude": {"vendor": "a", "client": "b", "headless": "c", "open_pr_limit": 2}, "codex": {"vendor": "d", "client": "e", "headless": "f", "open_pr_limit": 2}}}"#,
        );
        m.write(
            "docs/harness/gates.json",
            &format!(
                r#"{{"version": 1, "gates": [{{"id": "g", "proves": "p", "command": ["{gate_cmd}"], "reads": ["a.txt"], "quick": true, "break": {{"file": "a.txt", "append": "x", "expect": "x"}}}}]}}"#
            ),
        );
        m.git(&["add", "-A"]);
        m.git(&["commit", "-q", "-m", "base"]);
        m.git(&["remote", "add", "origin", &o]);
        m.git(&["push", "-q", "origin", "main"]);
        m.git(&["fetch", "-q", "origin"]);
        let w = m.wt.to_str().unwrap().to_string();
        m.git(&["worktree", "add", "-q", "-b", "claude/x", &w, "main"]);
        std::fs::write(m.wt.join("x.rs"), "// work\n").unwrap();
        m.git_in(&m.wt, &["add", "-A"]);
        m.git_in(&m.wt, &["commit", "-q", "-m", "work"]);
        m.head = m.git(&["rev-parse", "claude/x"]);
        m.git(&[
            "push",
            "-q",
            "origin",
            "claude/x",
            "claude/x:refs/pull/7/head",
        ]);
        m
    }

    fn git_in(&self, dir: &std::path::Path, args: &[&str]) -> String {
        assert!(dir.starts_with(&self.base));
        let o = clean(&mut Command::new("git"))
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git runs");
        assert!(
            o.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }

    fn git(&self, args: &[&str]) -> String {
        self.git_in(&self.main, args)
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.main.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    /// A commit with `tree` on top of `origin/main` and `subject`; pushed to origin's `main` when
    /// `push`.
    fn squash(&self, tree: &str, subject: &str, push: bool) -> String {
        let mc = self.git(&["commit-tree", tree, "-p", "origin/main", "-m", subject]);
        if push {
            self.git(&["push", "-q", "origin", &format!("{mc}:refs/heads/main")]);
        }
        mc
    }

    /// The usual squash: the branch's tree, `(#7)`, on origin's `main`.
    fn merged_squash(&self) -> String {
        self.squash(&format!("{}^{{tree}}", self.head), "work (#7)", true)
    }

    fn fake(&self, state: &str, mc: &str, head_ref: &str) {
        std::fs::write(
            self.base.join("gh.json"),
            format!(
                r#"{{"merged": {{"7": {{"state": "{state}", "mergeCommit": "{mc}", "headRefName": "{head_ref}", "headRefOid": "{}"}}}}}}"#,
                self.head
            ),
        )
        .unwrap();
    }

    fn verify(&self, extra: &[&str]) -> (i32, String) {
        let mut cmd = Command::new(BIN);
        clean(&mut cmd);
        let o = cmd
            .env("UOR_LAB", "claude")
            .env("UOR_LOOP_CLOSE_GH_FAKE", self.base.join("gh.json"))
            .env("UOR_LOOP_CLOSE_NO_HYGIENE", "1")
            .args(["--repo", self.main.to_str().unwrap(), "loop", "close"])
            .args(["--verify", "--pr", "7"])
            .args(extra)
            .current_dir(&self.main)
            .output()
            .expect("the binary runs");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
        (o.status.code().unwrap_or(-1), text)
    }

    fn has_local(&self, b: &str) -> bool {
        clean(&mut Command::new("git"))
            .args(["rev-parse", "--verify", "-q", &format!("refs/heads/{b}")])
            .current_dir(&self.main)
            .output()
            .unwrap()
            .status
            .success()
    }

    fn has_remote(&self, b: &str) -> bool {
        !self
            .git(&["ls-remote", "origin", &format!("refs/heads/{b}")])
            .is_empty()
    }

    /// The worktree, the local branch and the remote branch all still exist.
    fn all_kept(&self) -> bool {
        self.wt.exists() && self.has_local("claude/x") && self.has_remote("claude/x")
    }
}

#[test]
fn every_proof_passing_deletes_the_worktree_and_both_branches() {
    let m = Merged::new("true");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 0, "{out}");
    for s in [
        "PASS state",
        "PASS squash",
        "PASS content",
        "SKIP done-when",
        "PASS worktree",
    ] {
        assert!(out.contains(s), "{s}: {out}");
    }
    assert!(out.contains("SKIP hygiene"), "{out}");
    assert!(!m.wt.exists(), "{out}");
    assert!(!m.has_local("claude/x"), "{out}");
    assert!(!m.has_remote("claude/x"), "{out}");
}

#[test]
fn an_open_pr_deletes_nothing() {
    let m = Merged::new("true");
    m.fake("OPEN", &m.merged_squash(), "claude/x");
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL state"), "{out}");
    assert!(m.all_kept(), "{out}");
}

#[test]
fn a_merge_commit_off_main_or_without_the_pr_number_deletes_nothing() {
    let m = Merged::new("true");
    let tree = format!("{}^{{tree}}", m.head);
    m.fake("MERGED", &m.squash(&tree, "work (#7)", false), "claude/x");
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("FAIL squash") && out.contains("not on origin/main"),
        "{out}"
    );
    assert!(m.all_kept(), "{out}");

    m.fake("MERGED", &m.squash(&tree, "work (#8)", true), "claude/x");
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL squash") && out.contains("(#7)"), "{out}");
    assert!(m.all_kept(), "{out}");
}

#[test]
fn a_file_on_main_that_differs_from_the_head_deletes_nothing() {
    let m = Merged::new("true");
    // main's tree without the PR's x.rs.
    m.fake(
        "MERGED",
        &m.squash("origin/main^{tree}", "work (#7)", true),
        "claude/x",
    );
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL content"), "{out}");
    assert!(m.all_kept(), "{out}");
}

#[test]
fn a_dirty_worktree_is_kept_but_a_claims_only_edit_is_reverted_and_removed() {
    let m = Merged::new("true");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    std::fs::write(m.wt.join("extra.txt"), "x\n").unwrap();
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL worktree"), "{out}");
    assert!(m.wt.join("extra.txt").exists(), "{out}");
    // The kept worktree has the branch out, so the local branch stays; the remote one still goes.
    assert!(out.contains("FAIL local-branch"), "{out}");
    assert!(m.has_local("claude/x"), "{out}");
    assert!(out.contains("PASS remote-branch"), "{out}");
    assert!(!m.has_remote("claude/x"), "{out}");

    let m = Merged::new("true");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    std::fs::write(m.wt.join("docs/harness/claims.json"), "{\"stale\": 1}\n").unwrap();
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("PASS worktree"), "{out}");
    assert!(!m.wt.exists(), "{out}");
}

#[test]
fn a_local_branch_that_moved_is_kept() {
    let m = Merged::new("true");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    std::fs::write(m.wt.join("more.rs"), "// more\n").unwrap();
    m.git_in(&m.wt, &["add", "-A"]);
    m.git_in(&m.wt, &["commit", "-q", "-m", "more"]);
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL local-branch"), "{out}");
    assert!(m.has_local("claude/x"), "{out}");
    assert!(out.contains("PASS worktree") && !m.wt.exists(), "{out}");
    assert!(out.contains("PASS remote-branch"), "{out}");
    assert!(!m.has_remote("claude/x"), "{out}");
}

#[test]
fn a_worktree_holding_ignored_notes_is_kept_but_build_and_view_output_is_not() {
    let m = Merged::new("true");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    std::fs::create_dir_all(m.wt.join("local/claude")).unwrap();
    std::fs::write(m.wt.join("local/claude/notes.txt"), "mine\n").unwrap();
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("FAIL worktree holds ignored files: local/claude/notes.txt"),
        "{out}"
    );
    assert!(m.wt.join("local/claude/notes.txt").exists(), "{out}");

    let m = Merged::new("true");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    std::fs::create_dir_all(m.wt.join("target")).unwrap();
    std::fs::write(m.wt.join("target/x"), "build\n").unwrap();
    std::fs::create_dir_all(m.wt.join("local/claude/views")).unwrap();
    std::fs::write(m.wt.join("local/claude/views/STATE.md"), "view\n").unwrap();
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("PASS worktree"), "{out}");
    assert!(!m.wt.exists(), "{out}");
}

/// Main adds `s.txt` (20 lines); the PR, rebased on it, changes line 15; main then changes line 2;
/// the squash on top of that sets line 15 to `line15`. Returns the squash commit.
fn shared_file_squash(m: &mut Merged, pr_line: &str, line15: &str) -> String {
    let lines: Vec<String> = (1..=20).map(|i| format!("l{i}")).collect();
    let text = |l: &[String]| format!("{}\n", l.join("\n"));
    m.write("s.txt", &text(&lines));
    m.git(&["add", "s.txt"]);
    m.git(&["commit", "-q", "-m", "s"]);
    m.git(&["push", "-q", "origin", "main"]);
    m.git_in(&m.wt, &["rebase", "-q", "main"]);
    let mut pr = lines.clone();
    pr[14] = pr_line.into();
    std::fs::write(m.wt.join("s.txt"), text(&pr)).unwrap();
    m.git_in(&m.wt, &["commit", "-q", "-am", "pr"]);
    m.head = m.git(&["rev-parse", "claude/x"]);
    m.git(&[
        "push",
        "-q",
        "origin",
        "+claude/x",
        "+claude/x:refs/pull/7/head",
    ]);
    let mut main = lines.clone();
    main[1] = "main".into();
    m.write("s.txt", &text(&main));
    m.git(&["commit", "-q", "-am", "main"]);
    m.git(&["merge", "-q", "--squash", "claude/x"]);
    main[14] = line15.into();
    m.write("s.txt", &text(&main));
    m.git(&["add", "-A"]);
    m.git(&["commit", "-q", "-m", "work (#7)"]);
    m.git(&["push", "-q", "origin", "main"]);
    m.git(&["rev-parse", "HEAD"])
}

#[test]
fn main_changing_another_part_of_a_shared_file_still_proves_the_content() {
    let mut m = Merged::new("true");
    let mc = shared_file_squash(&mut m, "pr", "pr");
    m.fake("MERGED", &mc, "claude/x");
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("PASS content"), "{out}");
}

#[test]
fn a_squash_carrying_a_different_change_deletes_nothing() {
    let mut m = Merged::new("true");
    let mc = shared_file_squash(&mut m, "pr", "other");
    m.fake("MERGED", &mc, "claude/x");
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL content"), "{out}");
    assert!(m.all_kept(), "{out}");
}

#[test]
fn a_remote_branch_that_moved_is_kept_by_the_lease() {
    let m = Merged::new("true");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    let tree = format!("{}^{{tree}}", m.head);
    let moved = m.git(&["commit-tree", &tree, "-p", &m.head, "-m", "moved"]);
    m.git(&[
        "push",
        "-q",
        "origin",
        &format!("{moved}:refs/heads/claude/x"),
    ]);
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL remote-branch"), "{out}");
    assert_eq!(
        m.git(&["ls-remote", "origin", "refs/heads/claude/x"])
            .split_whitespace()
            .next(),
        Some(moved.as_str())
    );
}

#[test]
fn a_remote_branch_already_gone_is_not_applicable() {
    let m = Merged::new("true");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    m.git(&["push", "-q", "origin", ":refs/heads/claude/x"]);
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("N/A remote-branch"), "{out}");
    assert!(!m.wt.exists() && !m.has_local("claude/x"), "{out}");
}

#[test]
fn another_labs_pr_is_proved_but_nothing_is_deleted() {
    let m = Merged::new("true");
    m.git(&["branch", "codex/y", "claude/x"]);
    m.git(&["push", "-q", "origin", "codex/y"]);
    m.fake("MERGED", &m.merged_squash(), "codex/y");
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("PASS content"), "{out}");
    assert!(out.contains("SKIP cleanup"), "{out}");
    assert!(
        m.all_kept() && m.has_local("codex/y") && m.has_remote("codex/y"),
        "{out}"
    );
}

#[test]
fn green_gates_delete_red_gates_delete_nothing_and_the_verify_worktree_goes_either_way() {
    let m = Merged::new("true");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    let (code, out) = m.verify(&[]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("PASS done-when"), "{out}");
    assert!(!m.wt.exists(), "{out}");
    assert!(!m.main.join("local/claude/verify-7").exists(), "{out}");
    assert!(!m.git(&["worktree", "list"]).contains("verify-7"), "{out}");

    let m = Merged::new("false");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    let (code, out) = m.verify(&[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL done-when"), "{out}");
    assert!(m.all_kept(), "{out}");
    assert!(!m.main.join("local/claude/verify-7").exists(), "{out}");
    assert!(!m.git(&["worktree", "list"]).contains("verify-7"), "{out}");
}

#[test]
fn a_red_verify_keeps_the_red_gate_log_and_a_green_one_keeps_none() {
    let m = Merged::new("false");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    let (code, out) = m.verify(&[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL done-when"), "{out}");
    let kept = m.main.join("local/claude/verify-logs/7");
    let runs: Vec<PathBuf> = std::fs::read_dir(&kept)
        .unwrap_or_else(|e| panic!("{}: {e}\n{out}", kept.display()))
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(runs.len(), 1, "{runs:?}\n{out}");
    let exit = std::fs::read_to_string(runs[0].join("g.exit")).unwrap();
    assert_eq!(exit.trim(), "1", "{out}");
    assert!(runs[0].join("g.log").is_file(), "{out}");
    assert!(
        out.contains(&format!(
            "KEEP done-when red gate log(s) g copied to {}",
            runs[0].display()
        )),
        "{out}"
    );
    assert!(!m.main.join("local/claude/verify-7").exists(), "{out}");

    let m = Merged::new("true");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    let (code, out) = m.verify(&[]);
    assert_eq!(code, 0, "{out}");
    assert!(!m.main.join("local/claude/verify-logs").exists(), "{out}");
    assert!(!out.contains("KEEP done-when"), "{out}");
}

#[test]
fn a_squash_differing_only_in_whitespace_deletes_nothing() {
    let mut m = Merged::new("true");
    let mc = shared_file_squash(&mut m, "x = \"a b c\"", "x = \"ab c\"");
    m.fake("MERGED", &mc, "claude/x");
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL content"), "{out}");
    assert!(m.all_kept(), "{out}");
}

#[test]
fn a_successful_verify_removes_the_branch_config_section() {
    let m = Merged::new("true");
    m.git(&["config", "branch.claude/x.merge", "refs/heads/claude/x"]);
    assert_eq!(
        m.git(&["config", "--get", "branch.claude/x.merge"]),
        "refs/heads/claude/x"
    );
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 0, "{out}");
    let o = clean(&mut Command::new("git"))
        .args(["config", "--get", "branch.claude/x.merge"])
        .current_dir(&m.main)
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&o.stdout).trim().is_empty(),
        "{out}"
    );
}

#[test]
fn a_gates_ledger_and_build_output_alone_do_not_keep_the_worktree() {
    let m = Merged::new("true");
    m.fake("MERGED", &m.merged_squash(), "claude/x");
    std::fs::create_dir_all(m.wt.join("local/claude")).unwrap();
    std::fs::write(m.wt.join("local/claude/gates.jsonl"), "{}\n").unwrap();
    std::fs::create_dir_all(m.wt.join("target")).unwrap();
    std::fs::write(m.wt.join("target/x"), "build\n").unwrap();
    let (code, out) = m.verify(&["--no-gates"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("PASS worktree"), "{out}");
    assert!(!m.wt.exists(), "{out}");
}

#[test]
fn a_session_naming_a_closed_pr_on_the_current_branch_is_refused() {
    let r = Repo::new(&valid_report());
    std::fs::write(
        r.fake_path(),
        format!(
            "{{\"prs\": [{{\"number\": 7, \"headRefName\": \"feat\", \"state\": \"CLOSED\", \"isDraft\": false, \"headRefOid\": \"{}\"}}]}}",
            r.head()
        ),
    )
    .unwrap();
    let (code, out) = r.close(Some("claude"), &["--session", "--pr", "7"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("CLOSED"), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}
