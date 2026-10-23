//! `uor-harness loop next` pinned against a temporary git repository with a bare `origin`, GitHub
//! replaced by the test-only `UOR_LOOP_NEXT_GH_FAKE` and `UOR_LOOP_GH_FAKE` files. Every child
//! process has every `GIT_*` variable removed, and each repository sits inside its temp directory.

use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::{json, Value};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const HEAD: &str = "1111111111111111111111111111111111111111";
const OLD: &str = "2222222222222222222222222222222222222222";

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB")
        .env_remove("UOR_LOOP_GH_FAKE")
        .env_remove("UOR_LOOP_NEXT_GH_FAKE")
}

fn item(status: &str, scope: Option<&str>) -> String {
    let scope = scope.map(|s| format!("scope: {s}\n")).unwrap_or_default();
    format!("---\nid: x\nstatus: {status}\n{scope}---\n\n## Why\n")
}

struct Repo {
    tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
    /// A repository with `items` (id, status, scope) on `origin/main`.
    fn new(items: &[(&str, &str, Option<&str>)]) -> Repo {
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
            r#"{"labs":{
              "claude":{"vendor":"Anthropic","client":"b","headless":"c","open_pr_limit":3},
              "anthro":{"vendor":"Anthropic","client":"b","headless":"c","open_pr_limit":3},
              "codex":{"vendor":"OpenAI","client":"b","headless":"c","open_pr_limit":3},
              "kimi":{"vendor":"Moonshot","client":"b","headless":"c","open_pr_limit":3}}}"#,
        );
        r.write("docs/roadmap/items/.keep", "");
        r.write(
            "docs/loop/README.md",
            "# Loop\n\n## Build plan\n\n| PR | What it builds |\n|---|---|\n| 9 | x |\n",
        );
        for (id, status, scope) in items {
            r.write(
                &format!("docs/roadmap/items/{id}.md"),
                &item(status, *scope),
            );
        }
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "init"]);
        let remote = r.tmp.path().canonicalize().unwrap().join("remote.git");
        let st = clean(&mut Command::new("git"))
            .args(["init", "-q", "--bare", "-b", "main"])
            .arg(&remote)
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

    /// `loop next` as `lab` (none when empty) with `next` as the next fake and `claim` as the
    /// claim fake.
    fn next_as(&self, lab: &str, next: &Value, claim: &Value, json_out: bool) -> Output {
        let nf = self.tmp.path().join("next-fake.json");
        let cf = self.tmp.path().join("claim-fake.json");
        std::fs::write(&nf, next.to_string()).unwrap();
        std::fs::write(&cf, claim.to_string()).unwrap();
        let mut cmd = Command::new(BIN);
        clean(&mut cmd);
        cmd.arg("--repo").arg(&self.root);
        if json_out {
            cmd.arg("--json");
        }
        cmd.args(["loop", "next"])
            .env("UOR_LOOP_NEXT_GH_FAKE", &nf)
            .env("UOR_LOOP_GH_FAKE", &cf)
            .current_dir(&self.root);
        if !lab.is_empty() {
            cmd.env("UOR_LAB", lab);
        }
        cmd.output().expect("uor-harness runs")
    }

    fn next(&self, next: &Value) -> Output {
        self.next_as("claude", next, &json!([]), false)
    }
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn decision(o: &Output) -> String {
    stdout(o).lines().last().unwrap_or("").to_string()
}

/// An open PR from `branch` adding `<order_lab>-<n>/ORDER.md` for `item`, with its ORDER text.
fn pr(n: u64, branch: &str, order_lab: &str, item: &str) -> (Value, (String, String)) {
    let path = format!("docs/loop/tasks/{item}/{order_lab}-1/ORDER.md");
    // A setup item names its build-plan row (row 9 is on the fixture's loop page).
    let setup = if item.starts_with("infra-loop-pr09-") {
        "Setup row: 9.\n"
    } else {
        ""
    };
    let order = format!(
        "# ORDER\n\nLab: {order_lab}. Item: [{item}](../../../../roadmap/items/{item}.md) (#1).\n{setup}"
    );
    (
        json!({"number": n, "headRefName": branch, "headRefOid": HEAD, "isDraft": false,
               "mergeable": "MERGEABLE", "files": [{"path": path}], "checks": []}),
        (format!("{n}:{path}"), order),
    )
}

fn fake(prs: Vec<(Value, (String, String))>, comments: Value) -> Value {
    let mut files = serde_json::Map::new();
    let mut list = Vec::new();
    for (p, (k, v)) in prs {
        list.push(p);
        files.insert(k, Value::String(v));
    }
    json!({"prs": list, "comments": comments, "files": files, "labels": {}})
}

/// A `uor-review v1` record by `lab` on PR `n` at `sha` saying `verdict` (APPROVE or CHANGES), for
/// the ORDER that [`pr`] adds for `item` by `order_lab`, with `context: fresh`.
fn record(lab: &str, n: u64, sha: &str, verdict: &str, (item, order_lab): (&str, &str)) -> Value {
    use uor_r4_harness::loopreview::{render, DoneWhen, Record, Verdict};
    let body = render(&Record {
        lab: lab.to_string(),
        pr: n,
        head: sha.to_string(),
        model: "m".to_string(),
        launch: "l".to_string(),
        verdict: if verdict == "APPROVE" {
            Verdict::Approve
        } else {
            Verdict::Changes
        },
        order: format!("docs/loop/tasks/{item}/{order_lab}-1/ORDER.md"),
        fresh_context: true,
        done_when: DoneWhen::Host("cargo test: ok".to_string()),
    });
    json!({"author": "x", "body": body})
}

/// A claude APPROVE at HEAD whose `order:` is `order_path`.
fn approve_for(n: u64, order_path: &str) -> Value {
    use uor_r4_harness::loopreview::{render, DoneWhen, Record, Verdict};
    let body = render(&Record {
        lab: "claude".to_string(),
        pr: n,
        head: HEAD.to_string(),
        model: "m".to_string(),
        launch: "l".to_string(),
        verdict: Verdict::Approve,
        order: order_path.to_string(),
        fresh_context: true,
        done_when: DoneWhen::Host("cargo test: ok".to_string()),
    });
    json!({"author": "x", "body": body})
}

/// A fake with one non-setup codex PR `n` (code plus every ORDER in `orders`, each readable at the
/// PR head as `docs/loop/tasks/it-2/<order>/ORDER.md`).
fn chain_fake(n: u64, orders: &[&str], comments: Value) -> Value {
    let (mut p, (_, text)) = pr(n, "codex/chain", "codex", "it-2");
    let mut files = vec![json!({"path": "crates/a/src/lib.rs"})];
    let mut next = fake(vec![], comments);
    for o in orders {
        let path = format!("docs/loop/tasks/it-2/{o}/ORDER.md");
        files.push(json!({"path": path}));
        next["files"][format!("{n}:{path}")] = json!(text.clone());
    }
    p["files"] = Value::Array(files);
    next["prs"] = json!([p]);
    next
}

/// The PR also changes code, so it is not doc-only.
fn with_code((mut p, f): (Value, (String, String))) -> (Value, (String, String)) {
    p["files"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path": "crates/a/src/lib.rs"}));
    (p, f)
}

#[test]
fn a_stop_file_stops_on_the_kill_switch() {
    let r = Repo::new(&[("a", "ready", Some("[crates/a/**]"))]);
    r.write("local/loop/STOP", "");
    let o = r.next(&fake(vec![], json!({})));
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(decision(&o).starts_with("STOP kill switch"), "{o:?}");
}

#[test]
fn the_pause_label_on_2028_stops() {
    let r = Repo::new(&[("a", "ready", Some("[crates/a/**]"))]);
    let mut f = fake(vec![], json!({}));
    f["labels"] = json!({"2028": ["loop:pause"]});
    let o = r.next(&f);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(decision(&o).contains("loop:pause"), "{o:?}");
}

#[test]
fn an_own_pr_with_a_failing_check_is_a_repair() {
    let r = Repo::new(&[]);
    let (mut p, f) = pr(7, "claude/x", "claude", "it-1");
    p["checks"] = json!([{"name": "ci", "conclusion": "FAILURE"}]);
    let o = r.next(&fake(vec![(p, f)], json!({})));
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(decision(&o).starts_with("GO repair #7 — "), "{o:?}");
}

#[test]
fn an_own_pr_with_a_counted_request_changes_is_a_repair() {
    let r = Repo::new(&[]);
    let o = r.next(&fake(
        vec![pr(7, "claude/x", "claude", "it-1")],
        json!({"7": [record("codex", 7, HEAD, "APPROVE", ("it-1", "claude")), record("codex", 7, HEAD, "CHANGES", ("it-1", "claude"))]}),
    ));
    assert!(decision(&o).starts_with("GO repair #7"), "{o:?}");
    // Another lab's later APPROVE does not clear codex's CHANGES (owner, 22 Oct 17:51Z).
    let o = r.next(&fake(
        vec![pr(7, "claude/x", "claude", "it-1")],
        json!({"7": [record("codex", 7, HEAD, "CHANGES", ("it-1", "claude")), record("kimi", 7, HEAD, "APPROVE", ("it-1", "claude"))]}),
    ));
    assert!(
        decision(&o).starts_with("GO repair #7"),
        "a standing CHANGES must stay a repair: {o:?}"
    );
    // A REQUEST CHANGES at an older head does not count.
    let o = r.next(&fake(
        vec![pr(7, "claude/x", "claude", "it-1")],
        json!({"7": [record("codex", 7, OLD, "CHANGES", ("it-1", "claude"))]}),
    ));
    assert!(!decision(&o).contains("repair"), "{o:?}");
}

#[test]
fn a_conflicting_own_pr_is_a_repair() {
    let r = Repo::new(&[]);
    let (mut p, f) = pr(9, "claude/y", "claude", "it-1");
    p["mergeable"] = json!("CONFLICTING");
    let o = r.next(&fake(vec![(p, f)], json!({})));
    assert!(decision(&o).starts_with("GO repair #9"), "{o:?}");
}

#[test]
fn another_labs_setup_pr_needs_a_claude_review_until_claude_approves() {
    let r = Repo::new(&[]);
    let o = r.next(&fake(
        vec![pr(11, "codex/s", "codex", "infra-loop-pr09-x")],
        json!({}),
    ));
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(decision(&o).starts_with("GO review #11 — "), "{o:?}");
    let o = r.next(&fake(
        vec![pr(11, "codex/s", "codex", "infra-loop-pr09-x")],
        json!({"11": [record("claude", 11, HEAD, "APPROVE", ("infra-loop-pr09-x", "codex"))]}),
    ));
    assert!(!decision(&o).contains("review"), "{o:?}");
}

/// Under D59 only claude reviews a setup PR: codex, of another vendor than the ORDER lab, owes
/// no review on it, while claude does.
#[test]
fn on_a_setup_pr_only_claude_owes_the_review() {
    let r = Repo::new(&[]);
    let setup = || {
        fake(
            vec![pr(11, "anthro/s", "anthro", "infra-loop-pr09-x")],
            json!({}),
        )
    };
    let o = r.next_as("codex", &setup(), &json!([]), false);
    assert!(!decision(&o).contains("review"), "{o:?}");
    let o = r.next_as("claude", &setup(), &json!([]), false);
    assert!(decision(&o).starts_with("GO review #11"), "{o:?}");
}

#[test]
fn a_non_setup_pr_needs_a_review_only_from_another_vendor() {
    let r = Repo::new(&[]);
    let o = r.next(&fake(
        vec![with_code(pr(12, "anthro/z", "anthro", "it-2"))],
        json!({}),
    ));
    assert!(!decision(&o).contains("review"), "same vendor: {o:?}");
    let o = r.next(&fake(
        vec![with_code(pr(12, "codex/z", "codex", "it-2"))],
        json!({}),
    ));
    assert!(decision(&o).starts_with("GO review #12"), "{o:?}");
    // Once this lab's counted APPROVE is at the head, no review is owed.
    let o = r.next(&fake(
        vec![with_code(pr(12, "codex/z", "codex", "it-2"))],
        json!({"12": [record("claude", 12, HEAD, "APPROVE", ("it-2", "codex"))]}),
    ));
    assert!(!decision(&o).contains("review"), "{o:?}");
}

/// One counted record at the head settles a review, whichever lab made it: another lab's
/// APPROVE passes the gate and its CHANGES waits on the author, so this lab is not sent to
/// review the same head again (#2974 was offered after Kimi's APPROVE on 22 Oct).
#[test]
fn a_pr_with_another_labs_counted_record_at_its_head_is_not_offered_for_review() {
    let r = Repo::new(&[]);
    for verdict in ["APPROVE", "CHANGES"] {
        let o = r.next(&fake(
            vec![with_code(pr(12, "codex/z", "codex", "it-2"))],
            json!({"12": [record("anthro", 12, HEAD, verdict, ("it-2", "codex"))]}),
        ));
        assert!(
            !decision(&o).starts_with("GO review"),
            "anthro's counted {verdict} at the head settles #12: {o:?}"
        );
    }
}

#[test]
fn a_ready_scoped_unclaimed_item_is_claimed_and_a_claimed_one_is_skipped() {
    let r = Repo::new(&[
        ("a-claimed", "ready", Some("[crates/a/**]")),
        ("b-free", "ready", Some("[crates/b/**]")),
        ("c-noscope", "ready", None),
    ]);
    let o = r.next(&fake(vec![], json!({})));
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(decision(&o).starts_with("GO item a-claimed — "), "{o:?}");
    // A draft codex PR whose ORDER names a-claimed: no review (draft), no repair (not ours).
    let order =
        "# ORDER\n\nLab: codex. Item: [a-claimed](../../../../roadmap/items/a-claimed.md) (#1).\n";
    let path = "docs/loop/tasks/a-claimed/codex-1/ORDER.md";
    let claim = json!([{"number": 3, "headRefName": "codex/a", "isDraft": true,
                        "files": [{"path": path}], "orderTexts": {path: order}}]);
    let next = json!({"prs": [{"number": 3, "headRefName": "codex/a", "headRefOid": HEAD,
                               "isDraft": true, "files": [{"path": path}]}],
                      "files": {(format!("3:{path}")): order}});
    let o = r.next_as("claude", &next, &claim, false);
    assert!(decision(&o).starts_with("GO item b-free — "), "{o:?}");
}

#[test]
fn no_ready_scoped_item_waits_empty_or_held() {
    let r = Repo::new(&[("a", "ready", None), ("b", "draft", Some("[x/**]"))]);
    let o = r.next(&fake(vec![], json!({})));
    assert_eq!(o.status.code(), Some(3), "{o:?}");
    assert_eq!(
        decision(&o),
        "WAIT empty queue — check again in 600s",
        "{o:?}"
    );
    let r = Repo::new(&[("a", "held", Some("[x/**]")), ("b", "held", None)]);
    let o = r.next(&fake(vec![], json!({})));
    assert_eq!(
        decision(&o),
        "WAIT held work: 2 item(s) held — check again in 600s",
        "{o:?}"
    );
}

#[test]
fn an_item_waits_until_every_dependency_is_done() {
    // The fixture writes `scope: <s>`, so a second line puts `depends_on:` in the frontmatter.
    let r = Repo::new(&[
        (
            "a-waits",
            "ready",
            Some("[crates/a/**]\ndepends_on: [b-dep]"),
        ),
        ("b-dep", "decision", Some("[crates/b/**]")),
        (
            "c-missing",
            "ready",
            Some("[crates/c/**]\ndepends_on: [no-such-item]"),
        ),
    ]);
    let o = r.next(&fake(vec![], json!({})));
    assert_eq!(o.status.code(), Some(3), "{o:?}");
    assert!(
        decision(&o).contains("waiting on dependencies — check again in 600s"),
        "{o:?}"
    );
    let out = stdout(&o);
    assert!(out.contains("a-waits waits on b-dep (decision)"), "{o:?}");
    assert!(
        out.contains("c-missing waits on no-such-item (missing)"),
        "{o:?}"
    );
    // With the dependency done, the item is offered.
    let r = Repo::new(&[
        (
            "a-waits",
            "ready",
            Some("[crates/a/**]\ndepends_on: [b-dep]"),
        ),
        ("b-dep", "done", Some("[crates/b/**]")),
    ]);
    let o = r.next(&fake(vec![], json!({})));
    assert!(decision(&o).starts_with("GO item a-waits — "), "{o:?}");
}

#[test]
fn a_malformed_or_unreadable_dependency_waits_and_says_why() {
    let r = Repo::new(&[
        // Not an inline list, so the dependencies cannot be checked.
        (
            "a-bad-list",
            "ready",
            Some("[crates/a/**]\ndepends_on: b-dep"),
        ),
        ("b-dep", "done", Some("[crates/b/**]")),
        // The dependency is on origin/main but does not parse: its scope list is not closed.
        (
            "c-on-broken",
            "ready",
            Some("[crates/c/**]\ndepends_on: [d-broken]"),
        ),
        ("d-broken", "done", Some("[crates/d/**")),
    ]);
    let o = r.next(&fake(vec![], json!({})));
    assert!(
        decision(&o).contains("waiting on dependencies — check again in 600s"),
        "{o:?}"
    );
    let out = stdout(&o);
    assert!(
        out.contains("a-bad-list waits: depends_on `b-dep` is not a `[a, b]` list"),
        "{o:?}"
    );
    assert!(
        out.contains(
            "c-on-broken waits on d-broken (unreadable: scope `[crates/d/**` is not a `[a, b]` list)"
        ),
        "{o:?}"
    );
}

#[test]
fn only_unfinished_dependencies_are_named_and_quoted_done_or_empty_lists_pass() {
    let r = Repo::new(&[
        (
            "a-mixed",
            "ready",
            Some("[crates/a/**]\ndepends_on: [b-done, c-open, d-quoted]"),
        ),
        ("b-done", "done", Some("[crates/b/**]")),
        ("c-open", "decision", Some("[crates/c/**]")),
        ("d-quoted", "\"done\"", Some("[crates/d/**]")),
    ]);
    let o = r.next(&fake(vec![], json!({})));
    assert!(
        decision(&o).contains("waiting on dependencies — check again in 600s"),
        "{o:?}"
    );
    assert!(
        stdout(&o).contains("(a-mixed waits on c-open (decision))"),
        "{o:?}"
    );
    let r = Repo::new(&[("a-empty", "ready", Some("[crates/a/**]\ndepends_on: []"))]);
    let o = r.next(&fake(vec![], json!({})));
    assert!(decision(&o).starts_with("GO item a-empty — "), "{o:?}");
    let r = Repo::new(&[
        (
            "a-quoted",
            "ready",
            Some("[crates/a/**]\ndepends_on: [\"d-quoted\"]"),
        ),
        ("d-quoted", "\"done\"", Some("[crates/d/**]")),
    ]);
    let o = r.next(&fake(vec![], json!({})));
    assert!(decision(&o).starts_with("GO item a-quoted — "), "{o:?}");
}

#[test]
fn the_not_built_sources_are_printed_with_their_loop_prs() {
    let r = Repo::new(&[]);
    let out = stdout(&r.next(&fake(vec![], json!({}))));
    for line in [
        "owner-direction: not built — PLAN.md is not on origin/main yet (the PM writes it)",
        "gpu-collections: not built — a GPU run needs a collection record the watcher (loop PR 10) does not write yet",
        "premise-probes: not built — PLAN.md is not on origin/main yet (the PM writes it)",
        "lane: not built — no lane is assigned to claude in docs/harness/labs.json (the owner adds it)",
    ] {
        assert!(out.lines().any(|l| l == line), "missing {line:?} in {out}");
    }
}

#[test]
fn repairs_come_before_reviews() {
    let r = Repo::new(&[]);
    let (mut own, f) = pr(20, "claude/r", "claude", "it-1");
    own["checks"] = json!([{"name": "ci", "conclusion": "TIMED_OUT"}]);
    let o = r.next(&fake(
        vec![pr(5, "codex/s", "codex", "infra-loop-pr09-x"), (own, f)],
        json!({}),
    ));
    assert!(decision(&o).starts_with("GO repair #20"), "{o:?}");
}

#[test]
fn no_lab_exits_2_and_json_parses() {
    let r = Repo::new(&[]);
    let o = r.next_as("", &fake(vec![], json!({})), &json!([]), false);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stderr).contains("set UOR_LAB or pass --lab"));
    let o = r.next_as(
        "claude",
        &fake(
            vec![pr(11, "codex/s", "codex", "infra-loop-pr09-x")],
            json!({}),
        ),
        &json!([]),
        true,
    );
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let v: Value = serde_json::from_str(stdout(&o).trim()).expect("one JSON document");
    assert_eq!(v["lab"], "claude");
    assert_eq!(v["decision"]["go"], true);
    assert_eq!(v["decision"]["kind"], "review");
    assert_eq!(v["decision"]["target"], "#11");
    assert_eq!(v["sources"][0]["source"], "kill-switch");
}

#[test]
fn a_pr_is_judged_by_the_order_it_adds_not_another_labs_existing_one() {
    let r = Repo::new(&[]);
    // claude-9's ORDER is already on origin/main; the codex PR edits it and adds its own.
    let existing = "docs/loop/tasks/a/claude-9/ORDER.md";
    r.write(existing, "# ORDER\n\nLab: claude. Item: [a](x) (#1).\n");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "claude-9"]);
    r.git(&["push", "-q", "origin", "main"]);
    r.git(&["fetch", "-q", "origin"]);
    let (mut p, f) = pr(13, "codex/x", "codex", "it-2");
    let own = p["files"][0].clone();
    p["files"] = json!([{"path": existing}, own]);
    let mut next = fake(vec![(p, f)], json!({}));
    next["files"][format!("13:{existing}")] = json!("# ORDER\n\nLab: claude. Item: [a](x) (#1).\n");
    let o = r.next(&next);
    assert!(decision(&o).starts_with("GO review #13 — "), "{o:?}");
}

#[test]
fn a_pr_that_deletes_an_order_is_skipped_with_a_note_not_fatal() {
    let r = Repo::new(&[]);
    let gone = "docs/loop/tasks/z/codex-2/ORDER.md";
    let next = json!({"prs": [{"number": 14, "headRefName": "codex/d", "headRefOid": HEAD,
                               "isDraft": false, "files": [{"path": gone}]}]});
    let o = r.next(&next);
    assert_eq!(o.status.code(), Some(3), "{o:?}");
    assert!(
        stdout(&o)
            .lines()
            .any(|l| l.starts_with("reviews: ") && l.contains("#14 adds no ORDER")),
        "{o:?}"
    );
}

#[test]
fn a_pr_with_a_hundred_listed_files_is_judged_by_its_order_from_every_page() {
    let r = Repo::new(&[]);
    let (mut p, f) = pr(14, "codex/big", "codex", "it-2");
    let order_path = p["files"][0]["path"].as_str().unwrap().to_string();
    // The listing stops at 100 files, and the ORDER is not among them.
    p["files"] = Value::Array(
        (0..100)
            .map(|i| json!({"path": format!("crates/x/f{i}.rs")}))
            .collect(),
    );
    let mut next = fake(vec![(p, f)], json!({}));
    next["allFiles"] = json!({"14": [order_path]});
    let o = r.next(&next);
    assert!(decision(&o).starts_with("GO review #14 — "), "{o:?}");
}

#[test]
fn the_vendors_come_from_origin_main_not_the_working_tree() {
    let r = Repo::new(&[]);
    // This branch's labs.json makes claude and codex the same vendor; origin/main still differs.
    r.write(
        "docs/harness/labs.json",
        r#"{"labs":{
          "claude":{"vendor":"OpenAI","client":"b","headless":"c","open_pr_limit":3},
          "codex":{"vendor":"OpenAI","client":"b","headless":"c","open_pr_limit":3}}}"#,
    );
    let o = r.next(&fake(vec![pr(12, "codex/z", "codex", "it-2")], json!({})));
    assert!(decision(&o).starts_with("GO review #12"), "{o:?}");
}

#[test]
fn a_review_comes_before_a_ready_claimable_item() {
    let r = Repo::new(&[("b-free", "ready", Some("[crates/b/**]"))]);
    let o = r.next(&fake(vec![pr(12, "codex/z", "codex", "it-2")], json!({})));
    assert!(decision(&o).starts_with("GO review #12"), "{o:?}");
    assert!(!stdout(&o).contains("GO item"), "{o:?}");
}

#[test]
fn a_stop_file_beats_an_own_pr_that_needs_repair() {
    let r = Repo::new(&[]);
    r.write("local/loop/STOP", "");
    let (mut p, f) = pr(7, "claude/x", "claude", "it-1");
    p["checks"] = json!([{"name": "ci", "conclusion": "FAILURE"}]);
    let o = r.next(&fake(vec![(p, f)], json!({})));
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(decision(&o).starts_with("STOP kill switch"), "{o:?}");
}

#[test]
fn an_own_pr_whose_comments_cannot_be_read_cannot_run() {
    let r = Repo::new(&[]);
    let mut f = fake(vec![pr(7, "claude/x", "claude", "it-1")], json!({}));
    f["failComments"] = json!([7]);
    let o = r.next(&f);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("cannot run: could not read own PR #7"),
        "{err}"
    );
}

#[test]
fn an_own_pr_adding_two_orders_is_a_repair() {
    let r = Repo::new(&[]);
    let (mut p, mut f) = pr(7, "claude/x", "claude", "it-1");
    let second = "docs/loop/tasks/it-2/claude-1/ORDER.md";
    p["files"] = json!([
        {"path": "docs/loop/tasks/it-1/claude-1/ORDER.md"},
        {"path": second}
    ]);
    let mut fk = fake(vec![(p, f.clone())], json!({}));
    fk["files"][format!("7:{second}")] = json!(f.1.clone());
    f.0.clear();
    let o = r.next(&fk);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(
        decision(&o).contains("GO repair #7")
            && decision(&o).contains("its ORDER is missing or ambiguous"),
        "{o:?}"
    );
}

#[test]
fn a_repair_chain_is_judged_by_its_highest_order() {
    let r = Repo::new(&[]);
    let one = "docs/loop/tasks/it-2/codex-1/ORDER.md";
    let two = "docs/loop/tasks/it-2/codex-2/ORDER.md";
    let orders = ["codex-1", "codex-2"];
    // No review yet: the PR needs claude's.
    let o = r.next(&chain_fake(21, &orders, json!({})));
    assert!(decision(&o).starts_with("GO review #21 — "), "{o:?}");
    // An APPROVE naming the superseded codex-1 ORDER does not count, so the review is still owed.
    let o = r.next(&chain_fake(
        21,
        &orders,
        json!({"21": [approve_for(21, one)]}),
    ));
    assert!(decision(&o).starts_with("GO review #21 — "), "{o:?}");
    // An APPROVE naming the active codex-2 ORDER counts, so no review is owed.
    let o = r.next(&chain_fake(
        21,
        &orders,
        json!({"21": [approve_for(21, two)]}),
    ));
    assert!(!decision(&o).contains("review #21"), "{o:?}");
}

#[test]
fn orders_of_two_labs_in_one_pr_are_refused_not_resolved() {
    let r = Repo::new(&[]);
    let o = r.next(&chain_fake(22, &["codex-1", "anthro-1"], json!({})));
    assert!(!decision(&o).contains("review #22"), "{o:?}");
    assert!(
        stdout(&o).lines().any(|l| l.starts_with("reviews: ")
            && l.contains("#22 adds 2 ORDERs that are not one lab's repairs")
            && l.contains("labs: anthro, codex")),
        "{o:?}"
    );
}

#[test]
fn an_own_doc_only_pr_owes_a_fresh_self_review_and_an_own_code_pr_does_not() {
    let r = Repo::new(&[]);
    let (mut p, f) = pr(13, "claude/d", "claude", "it-3");
    for path in ["docs/x.md", "docs/loop/tasks/it-3/claude-1/REPORT.md"] {
        p["files"]
            .as_array_mut()
            .unwrap()
            .push(json!({"path": path}));
    }
    let o = r.next(&fake(vec![(p, f)], json!({})));
    assert!(decision(&o).starts_with("GO review #13"), "{o:?}");
    let o = r.next(&fake(
        vec![with_code(pr(13, "claude/d", "claude", "it-3"))],
        json!({}),
    ));
    assert!(!decision(&o).contains("review #13"), "{o:?}");
}

#[test]
fn a_lab_at_its_open_pr_limit_waits_naming_its_prs_awaiting_review() {
    let r = Repo::new(&[("a-free", "ready", Some("[crates/a/**]"))]);
    // Three open codex PRs, none ours to repair or review: codex is at open_pr_limit 3.
    let claim = json!([
        {"number": 41, "headRefName": "codex/x", "files": []},
        {"number": 42, "headRefName": "codex/y", "files": []},
        {"number": 43, "headRefName": "codex/z", "files": []},
    ]);
    // Each carries a valid ORDER for another item, so none needs a repair.
    let mut next = fake(
        vec![
            pr(41, "codex/x", "codex", "x-done"),
            pr(42, "codex/y", "codex", "y-done"),
            pr(43, "codex/z", "codex", "z-done"),
        ],
        json!({}),
    );
    // Drafts here, so this lab owes none of them a review itself.
    for p in next["prs"].as_array_mut().unwrap() {
        p["isDraft"] = json!(true);
    }
    let o = r.next_as("codex", &next, &claim, false);
    assert_eq!(
        o.status.code(),
        Some(3),
        "a lab at its open_pr_limit must WAIT, not STOP: {o:?}"
    );
    let d = decision(&o);
    assert!(
        d.starts_with("WAIT open_pr_limit reached: own PRs #41, #42, #43 await review"),
        "{o:?}"
    );
    assert!(d.ends_with("— check again in 600s"), "{o:?}");
}

/// A views-only render refresh PR on `kimi/render`: four rendered files, same repository, no ORDER.
fn render_pr(n: u64) -> Value {
    let files: Vec<Value> = [
        "STATE.md",
        "HANDOFF.md",
        "docs/harness/LESSONS.md",
        "AGENTS.md",
    ]
    .into_iter()
    .map(|path| json!({"path": path}))
    .collect();
    json!({"number": n, "headRefName": "kimi/render", "headRefOid": HEAD, "isDraft": false,
           "isCrossRepository": false, "mergeable": "MERGEABLE", "files": files, "checks": []})
}

fn fake_render(prs: Vec<Value>, comments: Value) -> Value {
    json!({"prs": prs, "comments": comments, "files": {}, "labels": {}})
}

fn render_record(lab: &str, n: u64) -> Value {
    use uor_r4_harness::loopreview::{render, DoneWhen, Record, Verdict};
    let body = render(&Record {
        lab: lab.to_string(),
        pr: n,
        head: HEAD.to_string(),
        model: "m".to_string(),
        launch: "l".to_string(),
        verdict: Verdict::Approve,
        order: "kimi/render".to_string(),
        fresh_context: true,
        done_when: DoneWhen::Host("cargo test: ok".to_string()),
    });
    json!({"author": "x", "body": body})
}

#[test]
fn a_views_only_render_pr_is_offered_to_another_lab_for_review() {
    let r = Repo::new(&[]);
    let o = r.next(&fake_render(vec![render_pr(30)], json!({})));
    assert!(decision(&o).starts_with("GO review #30"), "{o:?}");
    assert!(stdout(&o).contains("views-only render review"), "{o:?}");
}

#[test]
fn a_render_pr_recorded_at_its_head_owes_no_review() {
    let r = Repo::new(&[]);
    let o = r.next(&fake_render(
        vec![render_pr(30)],
        json!({"30": [render_record("claude", 30)]}),
    ));
    assert!(!decision(&o).contains("review #30"), "{o:?}");
}

#[test]
fn the_render_owner_lab_is_not_asked_to_review_its_own_render_pr() {
    let r = Repo::new(&[]);
    let o = r.next_as(
        "kimi",
        &fake_render(vec![render_pr(30)], json!({})),
        &json!([]),
        false,
    );
    assert!(!decision(&o).contains("review #30"), "{o:?}");
}

#[test]
fn the_production_listing_requests_the_render_sentinel_field() {
    // The fixture supplies isCrossRepository directly; this pins that the real query asks for
    // it too, so the fixture cannot pass while production deserializes None. Read the constant's
    // own value line, not the whole file: the doc comment above it also names the field, so a
    // whole-file contains() would stay green if the value lost it.
    let source =
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/loopnext.rs"))
            .unwrap();
    // The definition may wrap (rustfmt puts the value on its own line), so read from the
    // declaration through the terminating semicolon.
    let mut definition = String::new();
    let mut after_start = false;
    for line in source.lines() {
        if !after_start {
            after_start = line.trim_start().starts_with("const PR_LIST_FIELDS");
        }
        if after_start {
            definition.push_str(line);
            definition.push('\n');
            if line.trim_end().ends_with(';') {
                break;
            }
        }
    }
    assert!(
        definition.contains("isCrossRepository"),
        "PR_LIST_FIELDS must request isCrossRepository, got: {definition:?}"
    );
    assert!(
        source.contains("PR_LIST_FIELDS,"),
        "the gh pr list query must pass PR_LIST_FIELDS, not an inline field string"
    );
    let decoded: uor_r4_harness::loopnext::NextPr = serde_json::from_value(json!({
        "number": 30,
        "headRefName": "kimi/render",
        "headRefOid": HEAD,
        "isDraft": false,
        "isCrossRepository": false,
        "mergeable": "MERGEABLE",
        "files": [{"path": "STATE.md"}],
        "checks": []
    }))
    .expect("a gh-style PR decodes");
    assert_eq!(decoded.is_cross_repository, Some(false));
}
