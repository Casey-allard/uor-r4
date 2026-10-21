//! Each refusal of `uor-harness loop claim` pinned against a temporary git repository, with GitHub
//! replaced by the test-only `UOR_LOOP_GH_FAKE` file. Every child process has every `GIT_*`
//! variable removed, and each repository is checked to sit inside its temporary directory.

use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB").env_remove("UOR_LOOP_GH_FAKE")
}

struct Repo {
    tmp: tempfile::TempDir,
    root: PathBuf,
}

fn item(status: &str, scope: Option<&str>) -> String {
    let scope = scope.map(|s| format!("scope: {s}\n")).unwrap_or_default();
    format!("---\nid: x\nstatus: {status}\n{scope}---\n\n## Why\n")
}

fn order(id: &str) -> String {
    format!("# ORDER\n\nLab: codex. Item: [{id}](../../../../roadmap/items/{id}.md) (#1).\n")
}

impl Repo {
    fn new() -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap().join("repo");
        assert!(root.starts_with(tmp.path().canonicalize().unwrap()));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { tmp, root };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "t@example.invalid"]);
        r.git(&["config", "user.name", "t"]);
        r.write(".gitignore", "/local/\n/.worktrees/\n");
        r.write(
            "docs/harness/labs.json",
            r#"{"labs":{"claude":{"vendor":"a","client":"b","headless":"c","open_pr_limit":2}}}"#,
        );
        let items = [
            (
                "ready-a",
                item("ready", Some("[crates/a/**, docs/research/notebook/a.md]")),
            ),
            ("draft-b", item("draft", Some("[crates/b/**]"))),
            ("noscope-c", item("ready", None)),
            ("other-d", item("ready", Some("[crates/a/src/*.rs]"))),
            ("nb-e", item("ready", Some("[docs/research/notebook/a.md]"))),
        ];
        for (id, text) in items {
            r.write(&format!("docs/roadmap/items/{id}.md"), &text);
        }
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "init"]);
        let remote = r.tmp.path().canonicalize().unwrap().join("remote.git");
        let st = clean(&mut Command::new("git"))
            .args([
                "init",
                "-q",
                "--bare",
                "-b",
                "main",
                remote.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(st.status.success(), "{st:?}");
        r.git(&["remote", "add", "origin", remote.to_str().unwrap()]);
        r.git(&["push", "-q", "origin", "main"]);
        r.git(&["fetch", "-q", "origin"]);
        r
    }

    fn git(&self, args: &[&str]) {
        let st = clean(&mut Command::new("git"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs");
        assert!(st.status.success(), "git {args:?}: {st:?}");
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.root.join(rel);
        assert!(
            p.starts_with(&self.root),
            "write stays inside the temp repo"
        );
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    /// Run `loop claim <item>` as claude with `prs` as the fake `gh pr list` output.
    fn claim(&self, item: &str, prs: serde_json::Value) -> Output {
        self.claim_in(&self.root, item, prs)
    }

    /// Run `loop claim <item>` as claude with `--repo dir` and `dir` as the current directory.
    fn claim_in(&self, dir: &std::path::Path, item: &str, prs: serde_json::Value) -> Output {
        assert!(
            dir.starts_with(&self.root),
            "the claim runs inside the temp repo"
        );
        let fake = self.tmp.path().join("gh-fake.json");
        std::fs::write(&fake, prs.to_string()).unwrap();
        clean(&mut Command::new(BIN))
            .args(["--repo", dir.to_str().unwrap(), "loop", "claim", item])
            .env("UOR_LAB", "claude")
            .env("UOR_LOOP_GH_FAKE", &fake)
            .current_dir(dir)
            .output()
            .expect("uor-harness runs")
    }
}

fn pr_with_order(number: u64, branch: &str, id: &str) -> serde_json::Value {
    let path = format!("docs/loop/tasks/t/codex-{number}/ORDER.md");
    serde_json::json!({
        "number": number, "headRefName": branch, "isDraft": true,
        "files": [{"path": path}, {"path": "crates/z/x.rs"}],
        "orderTexts": {(path.clone()): order(id)}
    })
}

fn refused(o: &Output, why: &str) {
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(err.contains("refused") && err.contains(why), "{err}");
}

#[test]
fn a_missing_or_unready_item_is_refused() {
    let r = Repo::new();
    refused(
        &r.claim("no-such-item", serde_json::json!([])),
        "does not exist",
    );
    refused(&r.claim("draft-b", serde_json::json!([])), "not \"ready\"");
}

#[test]
fn an_item_named_by_an_open_order_is_refused() {
    let r = Repo::new();
    // An open PR's ORDER names the item.
    let prs = serde_json::json!([pr_with_order(7, "codex/a", "ready-a")]);
    refused(&r.claim("ready-a", prs), "already claimed by PR #7");
    // An unmerged worktree of this lab names it.
    r.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "claude/t",
        ".worktrees/claude-t",
    ]);
    let wt = r.root.join(".worktrees/claude-t");
    assert!(wt.starts_with(&r.root));
    let o = wt.join("docs/loop/tasks/t/claude-1/ORDER.md");
    std::fs::create_dir_all(o.parent().unwrap()).unwrap();
    std::fs::write(&o, order("ready-a")).unwrap();
    let wgit = |args: &[&str]| {
        let st = clean(&mut Command::new("git"))
            .args(args)
            .current_dir(&wt)
            .output()
            .unwrap();
        assert!(st.status.success(), "{st:?}");
    };
    wgit(&["add", "-A"]);
    wgit(&["commit", "-q", "-m", "order"]);
    refused(
        &r.claim("ready-a", serde_json::json!([])),
        "already claimed by worktree",
    );
    // Once its commit is in origin/main, the worktree's order is no longer open.
    r.git(&["push", "-q", "origin", "claude/t:main"]);
    r.git(&["fetch", "-q", "origin"]);
    let ok = r.claim("ready-a", serde_json::json!([]));
    assert_eq!(ok.status.code(), Some(0), "{ok:?}");
}

#[test]
fn an_item_without_scope_is_refused() {
    let r = Repo::new();
    refused(
        &r.claim("noscope-c", serde_json::json!([])),
        "has no scope: line",
    );
}

#[test]
fn an_overlapping_claim_is_refused() {
    let r = Repo::new();
    let prs = serde_json::json!([pr_with_order(9, "codex/d", "other-d")]);
    refused(
        &r.claim("ready-a", prs),
        "scope crates/a/** overlaps crates/a/src/*.rs",
    );
}

#[test]
fn a_notebook_page_overlap_is_allowed() {
    let r = Repo::new();
    let prs = serde_json::json!([pr_with_order(9, "codex/e", "nb-e")]);
    let o = r.claim("ready-a", prs);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
}

#[test]
fn a_lab_at_its_open_pr_limit_is_refused() {
    let r = Repo::new();
    let pr = |n: u64, b: &str| serde_json::json!({"number": n, "headRefName": b, "isDraft": false, "files": []});
    let prs = serde_json::json!([pr(1, "claude/x"), pr(2, "claude/y"), pr(3, "codex/z")]);
    refused(&r.claim("ready-a", prs), "at its open_pr_limit of 2");
    let under = serde_json::json!([pr(1, "claude/x"), pr(3, "codex/z")]);
    assert_eq!(r.claim("ready-a", under).status.code(), Some(0));
}

impl Repo {
    fn out(&self, dir: &std::path::Path, args: &[&str]) -> Output {
        assert!(dir.starts_with(&self.root), "git runs inside the temp repo");
        clean(&mut Command::new("git"))
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git runs")
    }

    /// Whether the bare `origin` in this temp dir has branch `b`.
    fn remote_has(&self, b: &str) -> bool {
        let remote = self.tmp.path().canonicalize().unwrap().join("remote.git");
        let r = format!("refs/heads/{b}");
        let o = clean(&mut Command::new("git"))
            .args(["ls-remote", remote.to_str().unwrap(), &r])
            .output()
            .expect("git runs");
        assert!(o.status.success(), "{o:?}");
        !o.stdout.is_empty()
    }

    fn has_branch(&self, b: &str) -> bool {
        let r = format!("refs/heads/{b}");
        self.out(&self.root, &["rev-parse", "--verify", "--quiet", &r])
            .status
            .success()
    }

    /// The fake's write log, one JSON value per line; empty when nothing was written.
    fn calls(&self) -> Vec<serde_json::Value> {
        let p = self.tmp.path().join("gh-fake.json.calls.jsonl");
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    /// Commit an ORDER naming `id` on a new worktree `.worktrees/claude-<name>`.
    fn order_worktree(&self, name: &str, order_rel: &str, id: &str) -> PathBuf {
        let wt = format!(".worktrees/claude-{name}");
        self.git(&[
            "worktree",
            "add",
            "-q",
            "-b",
            &format!("claude/{name}"),
            &wt,
        ]);
        let wt = self.root.join(wt);
        let o = wt.join(order_rel);
        std::fs::create_dir_all(o.parent().unwrap()).unwrap();
        std::fs::write(&o, order(id)).unwrap();
        assert!(self.out(&wt, &["add", "-A"]).status.success());
        assert!(self
            .out(&wt, &["commit", "-q", "-m", "order"])
            .status
            .success());
        wt
    }
}

fn nothing_created(r: &Repo, o: &Output) {
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(!r.has_branch("claude/ready-a"), "no branch was created");
    assert!(r.calls().is_empty(), "no GitHub write: {:?}", r.calls());
}

#[test]
fn a_clean_claim_creates_the_worktree_order_commit_and_draft_pr() {
    let r = Repo::new();
    let o = r.claim(
        "ready-a",
        serde_json::json!({"prs": [], "nextPrNumber": 77}),
    );
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("draft PR: #77"),
        "{o:?}"
    );
    let wt = r.root.join(".worktrees/claude-ready-a");
    let head = r.out(&wt, &["rev-parse", "--abbrev-ref", "HEAD"]);
    assert_eq!(
        String::from_utf8_lossy(&head.stdout).trim(),
        "claude/ready-a"
    );
    let shown = r.out(
        &wt,
        &["show", "HEAD:docs/loop/tasks/ready-a/claude-1/ORDER.md"],
    );
    assert!(shown.status.success(), "the ORDER is committed: {shown:?}");
    let text = String::from_utf8_lossy(&shown.stdout).into_owned();
    let item_line = text.lines().find(|l| l.contains("Item:")).unwrap();
    assert!(
        item_line.contains("(../../../../roadmap/items/ready-a.md)"),
        "{text}"
    );
    assert!(text.contains("- `crates/a/**`"), "{text}");
    let calls = r.calls();
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0]["call"], "create_draft_pr");
    assert_eq!(calls[0]["branch"], "claude/ready-a");
}

#[test]
fn a_lost_race_closes_our_pr_and_removes_only_what_we_created() {
    let r = Repo::new();
    r.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "claude/keep",
        ".worktrees/claude-keep",
    ]);
    r.git(&["push", "-q", "origin", "claude/keep"]);
    let fake = serde_json::json!({
        "prs": [], "nextPrNumber": 50,
        "afterCreate": [pr_with_order(40, "codex/a", "ready-a")]
    });
    let o = r.claim("ready-a", fake);
    assert_eq!(o.status.code(), Some(1), "a clean rollback exits 1: {o:?}");
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("lost the race to #40"),
        "{o:?}"
    );
    let calls = r.calls();
    assert_eq!(calls.len(), 2, "{calls:?}");
    assert_eq!(
        calls[1],
        serde_json::json!({"call": "close_pr", "number": 50})
    );
    assert!(!r.remote_has("claude/ready-a"), "our pushed branch is gone");
    assert!(
        r.remote_has("claude/keep"),
        "the unrelated remote branch survives"
    );
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("deleted claude/ready-a from origin"),
        "{o:?}"
    );
    assert!(!r.root.join(".worktrees/claude-ready-a").exists());
    assert!(!r.has_branch("claude/ready-a"));
    assert!(
        r.root.join(".worktrees/claude-keep").is_dir(),
        "the unrelated worktree survives"
    );
    assert!(r.has_branch("claude/keep"));
}

#[test]
fn a_branch_already_on_origin_is_refused_and_nothing_is_created() {
    let r = Repo::new();
    r.git(&["push", "-q", "origin", "main:refs/heads/claude/ready-a"]);
    let o = r.claim("ready-a", serde_json::json!([]));
    nothing_created(&r, &o);
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("already exists on origin"),
        "{o:?}"
    );
    assert!(!r.root.join(".worktrees/claude-ready-a").exists());
    assert!(r.remote_has("claude/ready-a"), "their branch is untouched");
}

#[test]
fn a_failed_create_after_the_push_rolls_everything_back() {
    let r = Repo::new();
    let o = r.claim(
        "ready-a",
        serde_json::json!({"prs": [], "createFails": true}),
    );
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(!r.has_branch("claude/ready-a"), "{o:?}");
    assert!(!r.root.join(".worktrees/claude-ready-a").exists(), "{o:?}");
    assert!(
        !r.remote_has("claude/ready-a"),
        "the pushed branch is deleted: {o:?}"
    );
    assert!(r.calls().is_empty(), "{:?}", r.calls());
}

#[test]
fn an_existing_target_worktree_or_branch_is_refused_and_nothing_is_created() {
    let r = Repo::new();
    let dir = r.root.join(".worktrees/claude-ready-a");
    std::fs::create_dir_all(&dir).unwrap();
    let o = r.claim("ready-a", serde_json::json!([]));
    nothing_created(&r, &o);
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("already exists"),
        "{o:?}"
    );
    std::fs::remove_dir(&dir).unwrap();
    r.git(&["branch", "claude/ready-a"]);
    let o = r.claim("ready-a", serde_json::json!([]));
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stderr).contains("branch claude/ready-a already exists"));
    assert!(!dir.exists(), "no worktree was created");
    assert!(r.calls().is_empty());
}

#[test]
fn a_squash_merged_order_no_longer_blocks_a_claim() {
    let r = Repo::new();
    let rel = "docs/loop/tasks/t/claude-1/ORDER.md";
    r.order_worktree("t", rel, "ready-a");
    refused(
        &r.claim("ready-a", serde_json::json!([])),
        "already claimed by worktree",
    );
    // Squash-merge: the same ORDER bytes land on main in a new commit; claude/t is no ancestor.
    r.write(rel, &order("ready-a"));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "squash"]);
    r.git(&["push", "-q", "origin", "main"]);
    r.git(&["fetch", "-q", "origin"]);
    let o = r.claim("ready-a", serde_json::json!([]));
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    // The next number counts the squashed claude-1 folder.
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("docs/loop/tasks/ready-a/claude-2/ORDER.md")
    );
}

#[test]
fn a_failed_fetch_exits_2_and_creates_nothing() {
    let r = Repo::new();
    r.git(&["remote", "remove", "origin"]);
    let o = r.claim("ready-a", serde_json::json!([]));
    nothing_created(&r, &o);
    refused(
        &o,
        "cannot fetch origin main, so nothing was checked or written",
    );
    assert!(!r.root.join(".worktrees").exists());
    // The fetch comes before every check: even a missing item reports the fetch.
    let o = r.claim("no-such-item", serde_json::json!([]));
    refused(&o, "cannot fetch origin main");
}

const PARENT: &str = "docs/loop/tasks/ready-a/claude-1/ORDER.md";

impl Repo {
    /// Run `uor-harness loop <args>` as claude, with no GitHub at all.
    fn loop_cmd(&self, args: &[&str]) -> Output {
        clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop"])
            .args(args)
            .env("UOR_LAB", "claude")
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs")
    }

    /// A parent ORDER naming `ready-a`, whose scope is `[crates/a/**, docs/research/notebook/a.md]`.
    fn with_parent(self) -> Repo {
        self.write(PARENT, &order("ready-a"));
        self
    }

    fn card(&self, rel: &str) -> Option<String> {
        let p = self.root.join("local/claude/claude-1").join(rel);
        assert!(p.starts_with(&self.root));
        std::fs::read_to_string(p).ok()
    }
}

#[test]
fn suborder_accepts_a_narrower_scope_and_writes_the_order() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&[
        "suborder",
        "--parent",
        PARENT,
        "--scope",
        "crates/a/src/**,crates/a/x.rs",
    ]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        out.contains(PARENT) && out.contains("Stop and ask on any file outside this scope"),
        "{out}"
    );
    let card = r.card("orders/1.md").expect("orders/1.md is written");
    assert!(
        card.contains("- `crates/a/src/**`") && card.contains("- `crates/a/x.rs`"),
        "{card}"
    );
    let again = r.loop_cmd(&["suborder", "--parent", PARENT, "--scope", "crates/a/y.rs"]);
    assert_eq!(again.status.code(), Some(0), "{again:?}");
    assert!(
        r.card("orders/2.md").is_some(),
        "the next number is one more"
    );
}

#[test]
fn suborder_refuses_a_glob_outside_the_parent_scope() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&[
        "suborder",
        "--parent",
        PARENT,
        "--scope",
        "crates/a/x.rs,crates/b/y.rs",
    ]);
    refused(&o, "scope crates/b/y.rs is outside the parent's scope");
    assert!(r.card("orders/1.md").is_none(), "nothing is written");
}

#[test]
fn suborder_refuses_a_sibling_whose_prefix_only_partly_matches() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&["suborder", "--parent", PARENT, "--scope", "crates/ab/x.rs"]);
    refused(&o, "scope crates/ab/x.rs is outside");
    assert!(r.card("orders/1.md").is_none());
}

#[test]
fn ask_scope_writes_a_card() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&[
        "ask",
        "--scope",
        "--order",
        PARENT,
        "--file",
        "crates/b/lib.rs",
        "--reason",
        "the helper lives there",
        "--lines",
        "12",
        "--own-code",
        "no",
    ]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let card = r.card("asks/1.md").expect("asks/1.md is written");
    for want in [
        "`crates/b/lib.rs`",
        "the helper lives there",
        "roughly: 12",
        "instead: no",
    ] {
        assert!(card.contains(want), "{want} in {card}");
    }
}

#[test]
fn ask_blocked_writes_a_card() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&[
        "ask",
        "--blocked",
        "--order",
        PARENT,
        "--limit",
        "4 pods",
        "--measured",
        "uor-pod status count",
        "--command",
        "scripts/pod/uor-pod status",
        "--state",
        "not-reached",
    ]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let card = r.card("asks/1.md").expect("asks/1.md is written");
    for want in [
        "Limit: 4 pods",
        "uor-pod status count",
        "`scripts/pod/uor-pod status`",
        "State: not-reached",
    ] {
        assert!(card.contains(want), "{want} in {card}");
    }
}

#[test]
fn ask_refuses_a_missing_field() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&[
        "ask", "--scope", "--order", PARENT, "--file", "x.rs", "--reason", "r", "--lines", "3",
    ]);
    refused(&o, "missing --own-code");
    let o = r.loop_cmd(&[
        "ask",
        "--blocked",
        "--order",
        PARENT,
        "--limit",
        "l",
        "--measured",
        "m",
        "--state",
        "refused",
    ]);
    refused(&o, "missing --command");
    assert!(r.card("asks/1.md").is_none(), "nothing is written");
}

impl Repo {
    /// Commit `text` as item `id` and push it, so that it is on `origin/main`.
    fn publish(&self, id: &str, text: &str) {
        self.write(&format!("docs/roadmap/items/{id}.md"), text);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", id]);
        self.git(&["push", "-q", "origin", "main"]);
    }
}

/// Each refused glob form, and the glob the refusal names (`{a,b}` splits at its comma).
const BAD_GLOBS: [(&str, &str); 6] = [
    ("crates/a/../b/**", "`crates/a/../b/**`"),
    ("./x", "`./x`"),
    ("/abs", "`/abs`"),
    ("a//b", "`a//b`"),
    ("{a,b}", "`{a`"),
    ("a\\b", "`a\\b`"),
];

#[test]
fn claim_refuses_an_item_with_a_bad_scope_glob() {
    let r = Repo::new();
    for (i, (glob, named)) in BAD_GLOBS.iter().enumerate() {
        let id = format!("bad-{i}");
        r.publish(&id, &item("ready", Some(&format!("[{glob}]"))));
        let o = r.claim(&id, serde_json::json!([]));
        refused(&o, &format!("scope glob {named}"));
        assert!(!r.has_branch(&format!("claude/{id}")), "{glob}: no branch");
        assert!(r.calls().is_empty(), "{glob}: no GitHub write");
    }
}

#[test]
fn suborder_refuses_a_bad_scope_glob() {
    let r = Repo::new().with_parent();
    for (glob, named) in BAD_GLOBS {
        let o = r.loop_cmd(&["suborder", "--parent", PARENT, "--scope", glob]);
        refused(&o, &format!("scope glob {named}"));
    }
    assert!(r.card("orders/1.md").is_none(), "nothing is written");
}

#[test]
fn suborder_containment_cannot_be_escaped() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&[
        "suborder",
        "--parent",
        PARENT,
        "--scope",
        "crates/a/../b/x.rs",
    ]);
    refused(&o, "crates/a/../b/x.rs");
    let o = r.loop_cmd(&[
        "suborder",
        "--parent",
        PARENT,
        "--scope",
        "crates/a/x/**,crates/a/y.rs",
    ]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    // A parent whose wildcards are not a trailing `/**` contains only identical globs.
    r.publish("glob-g", &item("ready", Some("[docs/*.md, crates/a*]")));
    let parent = "docs/loop/tasks/glob-g/claude-2/ORDER.md";
    r.write(parent, &order("glob-g"));
    let o = r.loop_cmd(&["suborder", "--parent", parent, "--scope", "docs/secret.rs"]);
    refused(&o, "scope docs/secret.rs is outside");
    let o = r.loop_cmd(&[
        "suborder",
        "--parent",
        parent,
        "--scope",
        "crates/ab/deep/x.rs",
    ]);
    refused(&o, "scope crates/ab/deep/x.rs is outside");
    let o = r.loop_cmd(&["suborder", "--parent", parent, "--scope", "docs/*.md"]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
}

#[test]
fn claim_reads_the_item_from_origin_main_not_the_working_tree() {
    let r = Repo::new();
    // Ready in the working tree and in a local commit, still draft on origin/main.
    r.write(
        "docs/roadmap/items/draft-b.md",
        &item("ready", Some("[crates/b/**]")),
    );
    r.git(&["commit", "-q", "-am", "local only"]);
    refused(&r.claim("draft-b", serde_json::json!([])), "not \"ready\"");
    assert!(!r.has_branch("claude/draft-b"));
}

#[test]
fn claim_fetches_before_it_checks() {
    let r = Repo::new();
    let stale = String::from_utf8_lossy(&r.out(&r.root, &["rev-parse", "origin/main"]).stdout)
        .trim()
        .to_string();
    // origin's main moves ready-a to draft; our origin/main still says ready until a fetch.
    r.write(
        "docs/roadmap/items/ready-a.md",
        &item("draft", Some("[crates/a/**]")),
    );
    r.git(&["commit", "-q", "-am", "ready-a back to draft"]);
    r.git(&["push", "-q", "origin", "main"]);
    r.git(&["update-ref", "refs/remotes/origin/main", &stale]);
    refused(&r.claim("ready-a", serde_json::json!([])), "not \"ready\"");
    assert!(!r.has_branch("claude/ready-a"));
}

#[test]
fn an_open_order_without_scope_warns_and_is_skipped_for_overlap() {
    let r = Repo::new();
    let prs = serde_json::json!([
        pr_with_order(7, "codex/c", "noscope-c"),
        pr_with_order(8, "codex/z", "ghost-z")
    ]);
    let o = r.claim("ready-a", prs.clone());
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("warning: open order PR #7 (docs/loop/tasks/t/codex-7/ORDER.md) names noscope-c, which has no scope: line; overlap not checked against it"),
        "{err}"
    );
    assert!(
        err.contains("names ghost-z, which is not on origin/main"),
        "{err}"
    );
    // The same item is still refused as claimed.
    refused(
        &r.claim("noscope-c", prs),
        "item noscope-c is already claimed by PR #7",
    );
}

/// A PR that changes an ORDER the fake has no text for, so reading it fails.
fn pr_with_unreadable_order(number: u64) -> serde_json::Value {
    serde_json::json!({
        "number": number, "headRefName": format!("codex/u{number}"), "isDraft": true,
        "files": [{"path": format!("docs/loop/tasks/t/codex-{number}/ORDER.md")}]
    })
}

#[test]
fn an_unreadable_pr_order_warns_instead_of_blocking() {
    let r = Repo::new();
    let o = r.claim("other-d", serde_json::json!([pr_with_unreadable_order(9)]));
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("warning: cannot read PR #9's ORDER") && err.contains("unknown item"),
        "{err}"
    );
    // A lower-numbered unreadable PR names no item, so it never wins the race.
    let r = Repo::new();
    let fake = serde_json::json!({
        "prs": [], "nextPrNumber": 50,
        "afterCreate": [pr_with_unreadable_order(8)]
    });
    let o = r.claim("ready-a", fake);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("cannot read PR #8's ORDER"),
        "{o:?}"
    );
    assert!(r.remote_has("claude/ready-a"), "our claim stands");
}

#[test]
fn a_lower_numbered_ready_for_review_pr_wins_the_race() {
    let r = Repo::new();
    let mut rival = pr_with_order(40, "codex/a", "ready-a");
    rival["isDraft"] = serde_json::json!(false);
    let fake = serde_json::json!({
        "prs": [], "nextPrNumber": 50, "afterCreate": [rival]
    });
    let o = r.claim("ready-a", fake);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("lost the race to #40"),
        "{o:?}"
    );
    assert!(!r.remote_has("claude/ready-a"), "our pushed branch is gone");
}

#[test]
fn a_claim_from_a_linked_worktree_places_the_worktree_under_the_main_checkout() {
    let r = Repo::new();
    r.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "claude/host",
        ".worktrees/claude-host",
    ]);
    let host = r.root.join(".worktrees/claude-host");
    let o = r.claim_in(&host, "ready-a", serde_json::json!({"prs": []}));
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(r.root.join(".worktrees/claude-ready-a").is_dir(), "{o:?}");
    assert!(!host.join(".worktrees").exists(), "{o:?}");
}

#[test]
fn a_worktree_outside_the_main_worktrees_folder_holds_no_open_order() {
    let r = Repo::new();
    r.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "claude/x",
        "local/.worktrees/claude-x",
    ]);
    let wt = r.root.join("local/.worktrees/claude-x");
    let o = wt.join("docs/loop/tasks/t/claude-1/ORDER.md");
    std::fs::create_dir_all(o.parent().unwrap()).unwrap();
    std::fs::write(&o, order("ready-a")).unwrap();
    assert!(r.out(&wt, &["add", "-A"]).status.success());
    assert!(r
        .out(&wt, &["commit", "-q", "-m", "order"])
        .status
        .success());
    let c = r.claim("ready-a", serde_json::json!([]));
    assert_eq!(c.status.code(), Some(0), "{c:?}");
}

impl Repo {
    /// Run `uor-harness loop claim <args>` as claude with an empty fake `gh pr list`.
    fn claim_args(&self, args: &[&str]) -> Output {
        let fake = self.tmp.path().join("gh-fake-args.json");
        std::fs::write(&fake, "[]").unwrap();
        clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop", "claim"])
            .args(args)
            .env("UOR_LAB", "claude")
            .env("UOR_LOOP_GH_FAKE", &fake)
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs")
    }
}

#[test]
fn lab_topic_names_the_worktree_and_branch() {
    let r = Repo::new();
    let o = r.claim_args(&["ready-a", "--lab-topic", "t9"]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let wt = r.root.join(".worktrees/claude-t9");
    let head = r.out(&wt, &["rev-parse", "--abbrev-ref", "HEAD"]);
    assert_eq!(String::from_utf8_lossy(&head.stdout).trim(), "claude/t9");
    assert!(r.remote_has("claude/t9"));
    assert!(!r.has_branch("claude/ready-a"));
}

#[test]
fn a_bad_item_or_topic_id_exits_2_and_creates_nothing() {
    let r = Repo::new();
    for args in [&["A_b"][..], &["--lab-topic", "../x", "ready-a"][..]] {
        let o = r.claim_args(args);
        assert_eq!(o.status.code(), Some(2), "{args:?}: {o:?}");
        // Other refusals also exit 2 and create nothing; the message ties this to the id check.
        assert!(
            String::from_utf8_lossy(&o.stderr).contains("item and topic must match"),
            "{args:?}: {o:?}"
        );
        assert!(!r.root.join(".worktrees").exists(), "{args:?}: no worktree");
        assert!(!r.has_branch("claude/ready-a"), "{args:?}: no branch");
        assert!(
            !r.remote_has("claude/ready-a"),
            "{args:?}: no remote branch"
        );
        assert!(r.calls().is_empty(), "{args:?}: no GitHub write");
    }
}

#[test]
fn a_claim_narrower_than_an_open_order_scope_is_refused_for_overlap() {
    let r = Repo::new();
    r.publish("open-r", &item("ready", Some("[crates/a/**]")));
    r.publish("narrow-s", &item("ready", Some("[crates/a/b.rs]")));
    let prs = serde_json::json!([pr_with_order(9, "codex/r", "open-r")]);
    refused(
        &r.claim("narrow-s", prs),
        "scope crates/a/b.rs overlaps crates/a/**",
    );
    assert!(!r.has_branch("claude/narrow-s"));
}

#[test]
fn ask_refuses_both_neither_and_bad_values() {
    let r = Repo::new().with_parent();
    let scope = [
        "ask", "--scope", "--order", PARENT, "--file", "x.rs", "--reason", "r", "--lines", "3",
    ];
    let both = [&scope[..], &["--blocked", "--own-code", "yes"][..]].concat();
    refused(&r.loop_cmd(&both), "exactly one of --scope or --blocked");
    refused(&r.loop_cmd(&["ask", "--order", PARENT]), "exactly one");
    let maybe = [&scope[..], &["--own-code", "maybe"][..]].concat();
    refused(&r.loop_cmd(&maybe), "is not yes or no");
    let later = [
        "ask",
        "--blocked",
        "--order",
        PARENT,
        "--limit",
        "l",
        "--measured",
        "m",
        "--command",
        "c",
        "--state",
        "later",
    ];
    refused(&r.loop_cmd(&later), "is not refused or not-reached");
    assert!(r.card("asks/1.md").is_none(), "nothing is written");
}
