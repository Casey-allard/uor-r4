//! `uor-harness loop queue` against a temporary repository whose bare `origin` carries `main` and
//! `refs/pull/7/head`, with `UOR_LOOP_QUEUE_GH_FAKE` standing in for `gh`. Every child process
//! has every `GIT_*` variable removed, and each repository is checked to sit inside its
//! temporary directory before anything writes to it.

use std::path::PathBuf;
use std::process::{Command, Output};
use uor_r4_harness::loopreview::{render, DoneWhen, Record, Verdict};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const N: u64 = 7;

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB")
        .env_remove("UOR_PRCHECK_GH_FAKE")
        .env_remove("UOR_LOOP_QUEUE_GH_FAKE")
}

const LABS: &str = r#"{"labs": {
  "claude": {"vendor": "anthropic", "client": "b", "headless": "c", "open_pr_limit": 2},
  "codex": {"vendor": "openai", "client": "b", "headless": "c", "open_pr_limit": 2},
  "deepseek": {"vendor": "deepseek", "client": "b", "headless": "c", "open_pr_limit": 2},
  "kimi": {"vendor": "moonshot", "client": "b", "headless": "c", "open_pr_limit": 2, "role": "pm"}
}}"#;
const FILEMAP: &str = "# FILEMAP\n\n| path | what |\n|---|---|\n| `crates/` | crates |\n| `crates/a` | a |\n| `docs` | docs |\n| `docs/harness` | h |\n| `docs/loop` | loop |\n| `docs/roadmap` | rm |\n| `docs/start` | s |\n";
const OWNER_PATHS: &str = r#"{"schema": 1, "paths": [
  {"path": "CHARTER.md", "why": "rules"},
  {"path": ".github/**", "why": "CI"},
  {"path": "docs/harness/owner-paths.json", "why": "this list"}
]}"#;
const REPORT: &str = "# REPORT\n\n## What did not work, first\n\nNothing.\n\n## What changed\n\nA.\n\n## Measurements\n\nNone.\n\n## Checks run at the head\n\n- tests\n\n## Skills used\n\nNone.\n\n## Still open\n\nNothing.\n\n## Plain summary\n\nDone.\n";
/// A setup PR (loop build-plan row 99): claude's records count alone under D59.
const SETUP_ORDER: &str = "docs/loop/tasks/infra-loop-pr99-x/claude-1/ORDER.md";
/// A code PR for `item-a`, whose scope admits `.github/**`: another vendor's records count.
const ORDER_A: &str = "docs/loop/tasks/item-a/claude-1/ORDER.md";
const REQUIRED: [&str; 5] = [
    "fmt / clippy / tests / no_std / κ",
    "cargo audit",
    "fuzz smoke (nightly libfuzzer)",
    "wasm-pack build (dashboard facade, deploy.yml parity)",
    "Gate C trend alarm",
];

struct Repo {
    tmp: tempfile::TempDir,
    root: PathBuf,
    base: String,
}

impl Repo {
    fn new() -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let top = tmp.path().canonicalize().unwrap();
        let root = top.join("repo");
        let origin = top.join("origin.git");
        assert!(root.starts_with(&top) && origin.starts_with(&top));
        std::fs::create_dir_all(&root).unwrap();
        let mut r = Repo {
            tmp,
            root,
            base: String::new(),
        };
        r.git(&["init", "-q", "--bare", origin.to_str().unwrap()]);
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["remote", "add", "origin", origin.to_str().unwrap()]);
        r.write("docs/harness/labs.json", LABS);
        r.write(
            "docs/roadmap/items/item-a.md",
            "---\nid: item-a\nstatus: ready\nscope: [crates/a/**, .github/**]\n---\n\n## Why\n",
        );
        r.write(
            "docs/roadmap/items/infra-loop-pr99-x.md",
            "---\nid: infra-loop-pr99-x\nstatus: ready\nscope: [crates/a/**]\n---\n",
        );
        r.write("docs/start/FILEMAP.md", FILEMAP);
        r.write("crates/a/src/lib.rs", "pub fn a() {}\n");
        r.write("docs/harness/owner-paths.json", OWNER_PATHS);
        r.write(
            "docs/loop/README.md",
            "# Loop\n\n## Build plan\n\n| PR | What it builds |\n|---|---|\n| 99 | x |\n",
        );
        r.write("CHARTER.md", "# Charter\n");
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "base"]);
        r.base = r.out(&["rev-parse", "HEAD"]);
        r.git(&["push", "-q", "origin", "main"]);
        r
    }

    fn cmd_git(&self, args: &[&str]) -> Output {
        clean(&mut Command::new("git"))
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs")
    }

    fn git(&self, args: &[&str]) {
        let st = self.cmd_git(args);
        assert!(st.status.success(), "git {args:?}: {st:?}");
    }

    fn out(&self, args: &[&str]) -> String {
        String::from_utf8_lossy(&self.cmd_git(args).stdout)
            .trim()
            .to_string()
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

    /// A PR head: the ORDER alone in the first commit, then the REPORT and `work`; pushed to
    /// origin as `refs/pull/7/head`. Returns its sha.
    fn pr(&self, order_path: &str, order: &str, work: impl FnOnce(&Repo)) -> String {
        self.git(&["checkout", "-q", "--detach", &self.base]);
        self.write(order_path, order);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", "order"]);
        self.write(&order_path.replace("ORDER.md", "REPORT.md"), REPORT);
        work(self);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", "work"]);
        let sha = self.out(&["rev-parse", "HEAD"]);
        self.git(&[
            "push",
            "-q",
            "-f",
            "origin",
            &format!("HEAD:refs/pull/{N}/head"),
        ]);
        self.git(&["checkout", "-q", "main"]);
        sha
    }

    fn setup_pr(&self) -> String {
        let order = "# ORDER\n\nLab: claude. Item: [x](../../../../roadmap/items/infra-loop-pr99-x.md) (#1).\nSetup row: 99\n";
        self.pr(SETUP_ORDER, order, |r| {
            r.write("crates/a/src/lib.rs", "//\n")
        })
    }

    fn fake_path(&self) -> PathBuf {
        self.tmp.path().join("gh-fake.json")
    }

    /// Merge calls the fake logged.
    fn merges(&self) -> Vec<serde_json::Value> {
        let p = PathBuf::from(format!("{}.calls.jsonl", self.fake_path().display()));
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn queue(&self, fake: &serde_json::Value, extra: &[&str]) -> Output {
        std::fs::write(self.fake_path(), fake.to_string()).unwrap();
        let mut cmd = Command::new(BIN);
        clean(&mut cmd)
            .env("UOR_LOOP_QUEUE_GH_FAKE", self.fake_path())
            .env("UOR_LAB", "claude")
            .args(["--repo", self.root.to_str().unwrap(), "loop", "queue"])
            .args(["--pr", &N.to_string()])
            .args(extra)
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs")
    }
}

/// A `uor-review v1` record by `lab` on PR 7 at `sha` for the ORDER at `order`.
fn record(order: &str, lab: &str, sha: &str, verdict: Verdict) -> serde_json::Value {
    let body = render(&Record {
        lab: lab.to_string(),
        pr: N,
        head: sha.to_string(),
        model: "m".to_string(),
        launch: "l".to_string(),
        verdict,
        order: order.to_string(),
        fresh_context: true,
        done_when: DoneWhen::Host("cargo test: ok".to_string()),
    });
    comment("casey", &body)
}

fn comment(author: &str, body: &str) -> serde_json::Value {
    serde_json::json!({"author": author, "body": body})
}

/// A fake answering PR 7 at `head` (OPEN, not a draft), `comments`, the five required check runs
/// from github-actions with the policy record, and no commit status.
fn fake(head: &str, comments: Vec<serde_json::Value>) -> serde_json::Value {
    let runs: Vec<serde_json::Value> = REQUIRED
        .iter()
        .map(|n| serde_json::json!({"name": n, "app": "github-actions", "summary": "policy: pass (run 42)"}))
        .collect();
    serde_json::json!({
        "pr": {"headRefOid": head, "headRefName": "claude/x", "baseRefName": "main", "state": "OPEN", "isDraft": false},
        "comments": comments,
        "checkRuns": runs,
        "statuses": [],
    })
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// Exit 0, `verdict: QUEUED` and exactly one merge call at `head`.
fn assert_queued(r: &Repo, o: &Output, head: &str) {
    assert_eq!(o.status.code(), Some(0), "{}", text(o));
    assert!(text(o).contains("verdict: QUEUED"), "{}", text(o));
    let calls = r.merges();
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0]["matchHeadCommit"], head);
    assert_eq!(calls[0]["pr"], N);
}

/// Exit 1, `verdict: REFUSED`, `line` in the output and no merge call.
fn assert_refused(r: &Repo, o: &Output, line: &str) {
    assert_eq!(o.status.code(), Some(1), "{}", text(o));
    assert!(text(o).contains("verdict: REFUSED"), "{}", text(o));
    assert!(text(o).contains(line), "no `{line}`:\n{}", text(o));
    assert!(r.merges().is_empty(), "{:?}", r.merges());
}

#[test]
fn a_clean_pass_queues_at_the_head() {
    let r = Repo::new();
    let h = r.setup_pr();
    let o = r.queue(
        &fake(
            &h,
            vec![record(SETUP_ORDER, "claude", &h, Verdict::Approve)],
        ),
        &[],
    );
    assert_queued(&r, &o, &h);
    assert!(text(&o).contains("FAKE GitHub"), "{}", text(&o));
}

/// Planted fault: a later counted CHANGES blocks queueing.
#[test]
fn planted_fault_a_later_counted_changes_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let comments = vec![
        record(SETUP_ORDER, "claude", &h, Verdict::Approve),
        record(SETUP_ORDER, "claude", &h, Verdict::Changes),
    ];
    let o = r.queue(&fake(&h, comments), &[]);
    assert_refused(&r, &o, "FAIL review-record");
}

/// Planted fault, the other half: a non-claude CHANGES on a setup PR under D59 does not count.
#[test]
fn a_later_kimi_changes_record_on_a_setup_pr_under_d59_still_queues() {
    let r = Repo::new();
    let h = r.setup_pr();
    let comments = vec![
        record(SETUP_ORDER, "claude", &h, Verdict::Approve),
        record(SETUP_ORDER, "kimi", &h, Verdict::Changes),
    ];
    let o = r.queue(&fake(&h, comments), &[]);
    assert_queued(&r, &o, &h);
}

#[test]
fn a_later_deepseek_comment_and_a_bot_comment_are_ignored() {
    let r = Repo::new();
    let h = r.setup_pr();
    let comments = vec![
        record(SETUP_ORDER, "claude", &h, Verdict::Approve),
        comment("casey", "[deepseek] P-2 read this; nothing to add."),
        comment("github-actions[bot]", "Policy job ran."),
    ];
    let o = r.queue(&fake(&h, comments), &[]);
    assert_queued(&r, &o, &h);
    assert!(
        text(&o).contains("ignored, from lab deepseek"),
        "{}",
        text(&o)
    );
}

#[test]
fn an_untagged_comment_after_the_approval_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let comments = vec![
        record(SETUP_ORDER, "claude", &h, Verdict::Approve),
        comment("casey", "Hold this one."),
    ];
    let o = r.queue(&fake(&h, comments), &[]);
    assert_refused(&r, &o, "FAIL after-approval");
    assert!(
        text(&o).contains("may be the owner's or the PM's"),
        "{}",
        text(&o)
    );
}

#[test]
fn a_kimi_comment_after_the_approval_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let comments = vec![
        record(SETUP_ORDER, "claude", &h, Verdict::Approve),
        comment("casey", "[kimi] Wait for the D-entry."),
    ];
    let o = r.queue(&fake(&h, comments), &[]);
    assert_refused(
        &r,
        &o,
        "refused, the PM (kimi) commented after the approval",
    );
}

#[test]
fn a_required_check_without_the_policy_record_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let mut f = fake(
        &h,
        vec![record(SETUP_ORDER, "claude", &h, Verdict::Approve)],
    );
    f["checkRuns"][1]["summary"] = "ok".into();
    let o = r.queue(&f, &[]);
    assert_refused(&r, &o, "FAIL required-checks");
}

#[test]
fn a_required_name_posted_as_a_commit_status_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let mut f = fake(
        &h,
        vec![record(SETUP_ORDER, "claude", &h, Verdict::Approve)],
    );
    f["statuses"] = serde_json::json!([{"context": "cargo audit", "state": "success"}]);
    let o = r.queue(&f, &[]);
    assert_refused(&r, &o, "a commit status named `cargo audit`");
}

#[test]
fn no_counted_approve_at_the_head_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let stale = "0".repeat(40);
    let comments = vec![
        record(SETUP_ORDER, "codex", &h, Verdict::Approve),
        record(SETUP_ORDER, "claude", &stale, Verdict::Approve),
    ];
    let o = r.queue(&fake(&h, comments), &[]);
    assert_refused(&r, &o, "FAIL review-record");
    assert!(text(&o).contains("FAIL after-approval"), "{}", text(&o));
}

#[test]
fn a_github_change_without_owner_approval_refuses() {
    let r = Repo::new();
    let order = "# ORDER\n\nLab: claude. Item: [a](../../../../roadmap/items/item-a.md) (#1).\n";
    let h = r.pr(ORDER_A, order, |r| {
        r.write(".github/workflows/x.yml", "on: push\n")
    });
    let o = r.queue(
        &fake(&h, vec![record(ORDER_A, "codex", &h, Verdict::Approve)]),
        &[],
    );
    assert_refused(&r, &o, "FAIL owner-paths");
}

#[test]
fn a_head_that_moved_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let other = "1".repeat(40);
    let o = r.queue(
        &fake(
            &other,
            vec![record(SETUP_ORDER, "claude", &h, Verdict::Approve)],
        ),
        &[],
    );
    assert_refused(&r, &o, "the PR moved; re-run");
}

#[test]
fn a_draft_pr_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let mut f = fake(
        &h,
        vec![record(SETUP_ORDER, "claude", &h, Verdict::Approve)],
    );
    f["pr"]["isDraft"] = true.into();
    let o = r.queue(&f, &[]);
    assert_refused(&r, &o, "FAIL pr-state");
}

#[test]
fn a_closed_pr_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let mut f = fake(
        &h,
        vec![record(SETUP_ORDER, "claude", &h, Verdict::Approve)],
    );
    f["pr"]["state"] = "CLOSED".into();
    let o = r.queue(&f, &[]);
    assert_refused(&r, &o, "FAIL pr-state PR 7 is CLOSED");
}

#[test]
fn a_pr_not_based_on_main_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let mut f = fake(
        &h,
        vec![record(SETUP_ORDER, "claude", &h, Verdict::Approve)],
    );
    f["pr"]["baseRefName"] = "release".into();
    let o = r.queue(&f, &[]);
    assert_refused(
        &r,
        &o,
        "FAIL pr-state PR 7 is OPEN, based on \"release\", not main",
    );
}

/// A hold written in the record's shape parses as malformed; the gate ignores it, so queue must
/// refuse on it.
#[test]
fn a_malformed_record_after_the_approval_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let comments = vec![
        record(SETUP_ORDER, "claude", &h, Verdict::Approve),
        comment(
            "casey",
            "[claude] uor-review v1\nHold — the APPROVE above missed X",
        ),
    ];
    let o = r.queue(&fake(&h, comments), &[]);
    assert_refused(&r, &o, "FAIL after-approval");
    assert!(
        text(&o).contains("refused, a malformed uor-review v1 record after the approval"),
        "{}",
        text(&o)
    );
}

/// The registry comes from `origin/main`: a head that strips kimi's PM role does not let a kimi
/// comment after the approval through.
#[test]
fn the_labs_registry_is_read_from_origin_main_not_the_head() {
    let r = Repo::new();
    let order = "# ORDER\n\nLab: claude. Item: [x](../../../../roadmap/items/infra-loop-pr99-x.md) (#1).\nSetup row: 99\n";
    let h = r.pr(SETUP_ORDER, order, |r| {
        r.write("crates/a/src/lib.rs", "//\n");
        r.write(
            "docs/harness/labs.json",
            &LABS.replace(r#", "role": "pm""#, ""),
        );
    });
    assert!(r
        .out(&["show", &format!("{h}:docs/harness/labs.json")])
        .contains("\"kimi\""));
    assert!(!r
        .out(&["show", &format!("{h}:docs/harness/labs.json")])
        .contains("\"pm\""));
    let comments = vec![
        record(SETUP_ORDER, "claude", &h, Verdict::Approve),
        comment("casey", "[kimi] Wait for the D-entry."),
    ];
    let o = r.queue(&fake(&h, comments), &[]);
    assert_refused(
        &r,
        &o,
        "refused, the PM (kimi) commented after the approval",
    );
    assert!(text(&o).contains("FAIL after-approval"), "{}", text(&o));
}

#[test]
fn a_required_check_whose_policy_record_says_fail_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let mut f = fake(
        &h,
        vec![record(SETUP_ORDER, "claude", &h, Verdict::Approve)],
    );
    f["checkRuns"][1]["summary"] = "policy: fail (run 1)".into();
    let o = r.queue(&f, &[]);
    assert_refused(&r, &o, "FAIL required-checks");
}

#[test]
fn a_required_check_from_another_app_refuses() {
    let r = Repo::new();
    let h = r.setup_pr();
    let mut f = fake(
        &h,
        vec![record(SETUP_ORDER, "claude", &h, Verdict::Approve)],
    );
    f["checkRuns"][2]["app"] = "other".into();
    f["checkRuns"][2]["summary"] = "policy: pass (run 1)".into();
    let o = r.queue(&f, &[]);
    assert_refused(&r, &o, "FAIL required-checks");
}

/// A failed merge call is its own verdict with the error on the `merge` line, exit 2.
#[test]
fn a_failed_merge_call_is_merge_failed() {
    let r = Repo::new();
    let h = r.setup_pr();
    let mut f = fake(
        &h,
        vec![record(SETUP_ORDER, "claude", &h, Verdict::Approve)],
    );
    f["failMerge"] = true.into();
    let o = r.queue(&f, &[]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("verdict: MERGE-FAILED"), "{}", text(&o));
    assert!(!text(&o).contains("verdict: REFUSED"), "{}", text(&o));
    assert!(
        text(&o).contains(&format!(
            "FAIL merge `gh pr merge 7 --match-head-commit {h}` failed: fake PR #7: the merge call failed"
        )),
        "{}",
        text(&o)
    );
    assert!(r.merges().is_empty(), "{:?}", r.merges());

    let o = r.queue(&f, &["--json"]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    let v: serde_json::Value =
        serde_json::from_slice(&o.stdout).unwrap_or_else(|_| panic!("{}", text(&o)));
    assert_eq!(v["verdict"], "merge-failed");
    let last = v["checks"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["check"], "merge");
    assert_eq!(last["verdict"], "fail");
}

#[test]
fn dry_run_on_a_passing_pr_writes_nothing() {
    let r = Repo::new();
    let h = r.setup_pr();
    let f = fake(
        &h,
        vec![record(SETUP_ORDER, "claude", &h, Verdict::Approve)],
    );
    let o = r.queue(&f, &["--dry-run"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    assert!(text(&o).contains(&format!(
        "would run `gh pr merge 7 --match-head-commit {h}`"
    )));
    assert!(r.merges().is_empty(), "{:?}", r.merges());

    let o = r.queue(&f, &["--dry-run", "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    let v: serde_json::Value =
        serde_json::from_slice(&o.stdout).unwrap_or_else(|_| panic!("{}", text(&o)));
    assert_eq!(v["verdict"], "would-queue");
    assert_eq!(v["head"], h.as_str());
    assert!(r.merges().is_empty());
}
