//! `uor-harness loop board --bench` pinned against a temporary git repository with a bare
//! `origin`, GitHub replaced by the test-only `UOR_LOOP_BENCH_GH_FAKE` file. Every child process
//! has every `GIT_*` variable removed, and the repository sits inside its temp directory. The
//! output is read with a small parser that follows the bench's own reading rules.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const HEAD_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HEAD_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

/// A `uor-review v1` record from `lab` on PR 30 at HEAD_A, with the field set `loopreview::render`
/// writes.
fn record(lab: &str, verdict: &str) -> String {
    format!(
        "[{lab}] uor-review v1\npr: 30\nhead: {HEAD_A}\nlab: {lab}\nmodel: m\nlaunch: l\nverdict: {verdict}\norder: docs/loop/tasks/it-claimed/claude-7/ORDER.md\ndone-when: host: cargo test passed\n"
    )
}

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB")
        .env_remove("UOR_LOOP_BOARD_GH_FAKE")
        .env_remove("UOR_LOOP_BENCH_GH_FAKE")
}

struct Repo {
    tmp: tempfile::TempDir,
    root: PathBuf,
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
        r.write(".gitignore", "/local/\n");
        r.write(
            "docs/harness/labs.json",
            r#"{"labs":{"claude":{"vendor":"Anthropic","client":"Claude Code","headless":"c","open_pr_limit":3},
              "codex":{"vendor":"OpenAI","client":"Codex","headless":"c","open_pr_limit":3}}}"#,
        );
        for (id, status, deps) in [
            ("it-done", "done", ""),
            ("it-ready", "ready", ""),
            ("it-held", "held", ""),
            ("it-decision", "decision", ""),
            ("it-waits", "ready", "depends_on: [it-ready]\n"),
            ("it-claimed", "ready", ""),
        ] {
            r.write(
                &format!("docs/roadmap/items/{id}.md"),
                &format!("---\nid: {id}\ntitle: \"Title {id}\"\nmilestone: m1\nstatus: {status}\n{deps}open_to: any\n---\n\n## Why\n"),
            );
        }
        let mut dec = String::from("# Decisions\n\n| Id | Date | Decision | Reason | Alternatives turned down | Cost to undo |\n|---|---|---|---|---|---|\n");
        for i in 1..=7 {
            let cost = if i == 7 { "high" } else { "low" };
            dec.push_str(&format!(
                "| D{i} | 2026-10-0{i} | decide {i} | r | a | {cost} |\n"
            ));
        }
        r.write("docs/integration/DECISIONS.md", &dec);
        r.write(
            "docs/loop/tasks/it-done/codex-3/receipts.jsonl",
            "{\"gate\":\"G1\",\"verdict\":\"pass\",\"seconds\":4,\"commit\":\"abc\",\"ts\":\"2026-10-20T00:00:00Z\"}\n",
        );
        // `loopreview::main_context` reads the loop page; no build-plan table, so no setup rows.
        r.write("docs/loop/README.md", "# Loop\n");
        r.write(
            "AGENTS.md",
            "# AGENTS\n\nThe project builds a model.\nIt runs fast.\n\nIt is pre-alpha: nothing is done.\n",
        );
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "init"]);
        r.write(
            "local/claude/gates.jsonl",
            "{\"gate\":\"G2\",\"verdict\":\"fail\",\"seconds\":1,\"commit\":\"def\",\"ts\":\"2026-10-21T00:00:00Z\"}\n",
        );
        let remote = r.tmp.path().canonicalize().unwrap().join("remote.git");
        let st = clean(&mut Command::new("git"))
            .args(["init", "-q", "--bare", "-b", "main"])
            .arg(&remote)
            .output()
            .unwrap();
        assert!(st.status.success(), "{st:?}");
        r.git(&["remote", "add", "origin", remote.to_str().unwrap()]);
        r.git(&["push", "-q", "origin", "main"]);
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
        assert!(p.starts_with(&self.root));
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn fake(&self) -> Value {
        let order = |item: &str, done: &str, stop: &str| {
            format!("# ORDER\n\nItem: [x](../../../../roadmap/items/{item}.md).\n\n## Done when\n\n- {done}\n- second\n\n## Stop and ask if\n\n- {stop}\n")
        };
        let pa = "docs/loop/tasks/it-claimed/claude-7/ORDER.md";
        let pb = "docs/loop/tasks/it-held/codex-2/ORDER.md";
        json!({
            "prs": [
                {"number": 20, "title": "[codex] draft work", "headRefOid": HEAD_B, "isDraft": true},
                {"number": 30, "title": "[claude] claimed work", "headRefOid": HEAD_A, "isDraft": false},
            ],
            "files": {"30": ["docs/loop/tasks/it-claimed/claude-6/ORDER.md", pa, "x.rs"], "20": [pb]},
            "comments": {"30": [{"author": {"login": "k"}, "body": record("codex", "CHANGES")}]},
            "contents": {
                format!("30:{pa}"): order("it-claimed", "the feed writes five files", "loopboard must change"),
                format!("20:{pb}"): order("elsewhere", "draft done", "draft stop"),
            },
        })
    }

    fn bench(&self, args: &[&str]) -> Output {
        self.bench_with(self.fake(), args)
    }

    fn bench_with(&self, fake: Value, args: &[&str]) -> Output {
        let f = self.tmp.path().join("bench-fake.json");
        std::fs::write(&f, fake.to_string()).unwrap();
        let mut cmd = Command::new(BIN);
        clean(&mut cmd);
        cmd.arg("--repo")
            .arg(&self.root)
            .args(["loop", "board"])
            .args(args)
            .env("UOR_LOOP_BENCH_GH_FAKE", &f)
            .current_dir(&self.root);
        cmd.output().expect("uor-harness runs")
    }

    fn out_dir(&self) -> PathBuf {
        self.tmp.path().canonicalize().unwrap().join("bench")
    }

    fn run_ok(&self) -> PathBuf {
        let dir = self.out_dir();
        let o = self.bench(&["--bench", dir.to_str().unwrap()]);
        assert!(o.status.success(), "{o:?}");
        assert!(String::from_utf8_lossy(&o.stdout).contains("bench feed: wrote 5 files"));
        dir
    }
}

/// The bench's table reader: (heading, header, rows) per table.
type Table = (String, Vec<String>, Vec<Vec<String>>);

fn split(l: &str) -> Vec<String> {
    let (mut cells, mut cur, mut tick, mut esc) = (Vec::new(), String::new(), false, false);
    for ch in l.trim().trim_start_matches('|').chars() {
        if esc {
            cur.push(ch);
            esc = false;
        } else if ch == '\\' {
            esc = true;
        } else if ch == '`' {
            tick = !tick;
            cur.push(ch);
        } else if ch == '|' && !tick {
            cells.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(ch);
        }
    }
    cells
}

fn tables(text: &str) -> Vec<Table> {
    let lines: Vec<&str> = text.lines().collect();
    let (mut out, mut heading, mut i) = (Vec::new(), String::new(), 0);
    while i < lines.len() {
        let l = lines[i];
        if l.starts_with('#') {
            heading = l.trim_start_matches('#').trim().to_string();
        }
        if l.starts_with('|') && lines.get(i + 1).is_some_and(|n| n.starts_with("|---")) {
            let head: Vec<String> = split(l).iter().map(|c| c.to_lowercase()).collect();
            let mut rows = Vec::new();
            i += 2;
            while i < lines.len() && lines[i].starts_with('|') {
                let r = split(lines[i]);
                if !r.first().is_some_and(|c| c.starts_with("Example:")) {
                    rows.push(r);
                }
                i += 1;
            }
            out.push((heading.clone(), head, rows));
            continue;
        }
        i += 1;
    }
    out
}

fn col(t: &Table, name: &str) -> usize {
    t.1.iter().position(|h| h.starts_with(name)).expect(name)
}

fn lane(status: &str) -> &'static str {
    let s = status.trim().to_lowercase();
    let first = s.split(|c: char| !c.is_alphanumeric()).next().unwrap_or("");
    let closed = "done merged finished complete completed closed dropped canceled cancelled thrown abandoned shipped";
    let open = [
        "in progress",
        "in-progress",
        "doing",
        "active",
        "started",
        "working",
        "review",
        "in review",
        "order written",
        "blocked",
    ];
    if closed.split(' ').any(|w| w == first) {
        "closed"
    } else if open.iter().any(|o| s.starts_with(o)) {
        "open"
    } else {
        "upcoming"
    }
}

fn read(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join(name)).unwrap()
}

#[test]
fn bench_writes_all_five_files() {
    let r = Repo::new();
    let dir = r.run_ok();
    for f in [
        ".gates.jsonl",
        "DECISIONS.md",
        "PLAN.md",
        "HANDOFF.md",
        "STATE.md",
    ] {
        assert!(dir.join(f).is_file(), "{f} missing");
    }
    let state = read(&dir, "STATE.md");
    assert!(state.contains("## The project in one sentence\n\nThe project builds a model.\n\n"));
    // One sentence: the opening paragraph's first line only.
    assert!(!state.contains("It runs fast."));
    assert!(state.contains("## Done looks like\n\nIt is pre-alpha: nothing is done."));
    assert!(state.contains("Last checked at commit `"));
    let t = tables(&state);
    let w = t
        .iter()
        .find(|t| t.0.to_lowercase().contains("what works now"))
        .unwrap();
    assert_eq!(w.2.len(), 3, "two labs and the roadmap row");
    assert!(
        w.2[2][1].contains("3 ready, 1 held, 1 decision, 1 done"),
        "{:?}",
        w.2[2]
    );
}

#[test]
fn bench_lanes_follow_item_and_pr_state() {
    let r = Repo::new();
    let dir = r.run_ok();
    let t = tables(&read(&dir, "PLAN.md"));
    let plan = t
        .iter()
        .find(|t| {
            t.1.iter().any(|h| h.starts_with("task")) && t.1.iter().any(|h| h.starts_with("status"))
        })
        .unwrap();
    assert_eq!(plan.2.len(), 6, "one row per item");
    let (id, st, lab) = (col(plan, "id"), col(plan, "status"), col(plan, "lab"));
    for row in &plan.2 {
        let want = match row[id].as_str() {
            "it-claimed" => "open",
            "it-done" => "closed",
            _ => "upcoming",
        };
        assert_eq!(lane(&row[st]), want, "{row:?}");
    }
    let claimed = plan.2.iter().find(|r| r[id] == "it-claimed").unwrap();
    assert_eq!(claimed[lab], "claude");
    assert!(
        claimed[st].contains("claude-7, PR #30, CHANGES at aaaaaaa"),
        "{claimed:?}"
    );
    let waits = plan.2.iter().find(|r| r[id] == "it-waits").unwrap();
    assert_eq!(waits[st], "ready, waits on it-ready");
}

#[test]
fn bench_next_puts_changes_first_with_order_bullets() {
    let r = Repo::new();
    let dir = r.run_ok();
    let t = tables(&read(&dir, "HANDOFF.md"));
    let next = t.iter().find(|t| t.0.starts_with("Next")).unwrap();
    assert_eq!(next.2.len(), 2, "one row per open lab PR");
    let first = &next.2[0];
    assert_eq!(first[col(next, "id")], "claude-7");
    assert_eq!(first[col(next, "task")], "[claude] #30 claimed work");
    assert_eq!(first[col(next, "done when")], "the feed writes five files");
    // The bench finds this column by the prefix "cancel".
    assert_eq!(first[col(next, "cancel")], "loopboard must change");
    assert_eq!(first[col(next, "status")], "CHANGES at aaaaaaa");
    assert_eq!(next.2[1][col(next, "status")], "draft");
    let blocked = t.iter().find(|t| t.0.starts_with("Blocked")).unwrap();
    assert_eq!(blocked.2.len(), 1);
}

#[test]
fn bench_decisions_keep_all_rows_newest_last() {
    let r = Repo::new();
    let dir = r.run_ok();
    let t = tables(&read(&dir, "DECISIONS.md"));
    let d = t
        .iter()
        .find(|t| t.1.iter().any(|h| h.starts_with("decision")))
        .unwrap();
    assert_eq!(d.2.len(), 7);
    let last6: Vec<&str> = d.2[1..]
        .iter()
        .rev()
        .map(|r| r[col(d, "id")].as_str())
        .collect();
    assert_eq!(last6, ["D7", "D6", "D5", "D4", "D3", "D2"]);
    assert_eq!(d.2[0][col(d, "reason")], "r");
    assert_eq!(
        d.2[6][col(d, "reversal cost")],
        "high",
        "the last row carries its source Cost to undo"
    );
}

#[test]
fn bench_gate_lines_parse_and_inbox_is_untouched() {
    let r = Repo::new();
    let dir = r.out_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("INBOX.md"), "keep me\n").unwrap();
    r.run_ok();
    assert_eq!(read(&dir, "INBOX.md"), "keep me\n");
    let lines: Vec<Value> = read(&dir, ".gates.jsonl")
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 2, "one per receipt and ledger line");
    for v in &lines {
        assert!(v["name"].is_string() && v["result"].is_string(), "{v}");
        for k in ["seconds", "time", "commit"] {
            assert!(v.get(k).is_some(), "gate line lacks {k}: {v}");
        }
    }
    assert!(lines
        .iter()
        .any(|v| v["name"] == "codex/G1" && v["result"] == "PASS"));
    assert!(lines
        .iter()
        .any(|v| v["name"] == "claude/G2" && v["result"] == "FAIL"));
}

#[test]
fn bench_refuses_tracked_dir_and_short_interval() {
    let r = Repo::new();
    let o = r.bench(&["--bench", "docs/x"]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("is inside the repository"), "{err}");
    let abs = r.root.join("docs/x");
    let o = r.bench(&["--bench", abs.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("is inside the repository"), "{err}");
    assert!(!abs.exists());
    assert!(!r.root.join("docs/x").exists());
    let dir = r.out_dir();
    let o = r.bench(&["--bench", dir.to_str().unwrap(), "--every", "5"]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("--every must be at least 30 seconds"), "{err}");
    let o = r.bench(&["--bench", "local/bench"]);
    assert!(o.status.success(), "{o:?}");
    assert!(r.root.join("local/bench/PLAN.md").is_file());
}

#[test]
fn bench_reads_the_lab_registry_from_origin_main() {
    let r = Repo::new();
    // A lab added in the working tree only, never committed: the feed must not list it.
    r.write(
        "docs/harness/labs.json",
        r#"{"labs":{"claude":{"vendor":"Anthropic","client":"Claude Code","headless":"c","open_pr_limit":3},
          "codex":{"vendor":"OpenAI","client":"Codex","headless":"c","open_pr_limit":3},
          "kimi":{"vendor":"Moonshot","client":"Kimi","headless":"c","open_pr_limit":3}}}"#,
    );
    let dir = r.run_ok();
    let t = tables(&read(&dir, "STATE.md"));
    let w = t
        .iter()
        .find(|t| t.0.to_lowercase().contains("what works now"))
        .unwrap();
    let areas: Vec<&str> = w.2.iter().map(|r| r[0].as_str()).collect();
    assert_eq!(areas.len(), 3, "{areas:?}");
    assert!(
        areas[0].starts_with("claude") && areas[1].starts_with("codex"),
        "{areas:?}"
    );
    assert!(!areas.iter().any(|a| a.starts_with("kimi")), "{areas:?}");
}

fn next_rows(r: &Repo, fake: Value) -> Vec<Vec<String>> {
    let dir = r.out_dir();
    let o = r.bench_with(fake, &["--bench", dir.to_str().unwrap()]);
    assert!(o.status.success(), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stdout).contains("bench feed: wrote 5 files"));
    for f in [
        "PLAN.md",
        "HANDOFF.md",
        "DECISIONS.md",
        "STATE.md",
        ".gates.jsonl",
    ] {
        assert!(dir.join(f).is_file(), "{f} missing");
    }
    let t = tables(&read(&dir, "HANDOFF.md"));
    let next = t.iter().find(|t| t.0.starts_with("Next")).unwrap();
    let c = (col(next, "id"), col(next, "task"), col(next, "done when"));
    let st = col(next, "status");
    next.2
        .iter()
        .map(|r| {
            vec![
                r[c.0].clone(),
                r[c.1].clone(),
                r[c.2].clone(),
                r[st].clone(),
            ]
        })
        .collect()
}

#[test]
fn one_unreadable_pr_does_not_stop_the_feed() {
    let r = Repo::new();
    let mut fake = r.fake();
    // PR 20's ORDER was deleted: its contents entry is gone.
    fake["contents"]
        .as_object_mut()
        .unwrap()
        .remove("20:docs/loop/tasks/it-held/codex-2/ORDER.md");
    let rows = next_rows(&r, fake);
    assert_eq!(rows.len(), 2, "{rows:?}");
    let bad = rows.iter().find(|r| r[1].contains("#20")).unwrap();
    assert!(bad[3].starts_with("ORDER unreadable"), "{bad:?}");
    let good = rows.iter().find(|r| r[1].contains("#30")).unwrap();
    assert_eq!(good[0], "claude-7");
    assert_eq!(good[3], "CHANGES at aaaaaaa");
}

#[test]
fn a_lab_pr_without_an_order_still_gets_a_row() {
    let r = Repo::new();
    let mut fake = r.fake();
    fake["files"]["20"] = json!(["x.rs"]);
    let rows = next_rows(&r, fake);
    let row = rows.iter().find(|r| r[0] == "#20").expect("a #20 row");
    assert_eq!(row[1], "[codex] #20 draft work");
    assert_eq!(row[2], "(no ORDER)");
    assert_eq!(row[3], "draft");
}

#[test]
fn a_self_review_the_gate_would_not_count_is_not_shown_as_approve() {
    let r = Repo::new();
    // PR 30 is claude's and changes x.rs, so it is not doc-only: D46 counts only another vendor.
    let mut fake = r.fake();
    fake["comments"]["30"] =
        json!([{"author": {"login": "k"}, "body": record("claude", "APPROVE")}]);
    let rows = next_rows(&r, fake);
    let row = rows.iter().find(|r| r[1].contains("#30")).unwrap();
    assert_eq!(row[3], "review not counted (claude)", "{row:?}");

    let mut fake = r.fake();
    fake["comments"]["30"] =
        json!([{"author": {"login": "k"}, "body": record("codex", "APPROVE")}]);
    let rows = next_rows(&r, fake);
    let row = rows.iter().find(|r| r[1].contains("#30")).unwrap();
    assert_eq!(row[3], "APPROVE at aaaaaaa", "{row:?}");
}

#[test]
fn a_pr_without_an_order_never_shows_an_uncounted_approve() {
    let r = Repo::new();
    // PR 30 has no ORDER file; the gate counts nothing without one, so the APPROVE at its head
    // is not shown as an approve.
    let mut fake = r.fake();
    fake["files"]["30"] = json!(["x.rs"]);
    fake["comments"]["30"] =
        json!([{"author": {"login": "k"}, "body": record("codex", "APPROVE")}]);
    let rows = next_rows(&r, fake);
    let row = rows.iter().find(|r| r[0] == "#30").expect("a #30 row");
    assert_eq!(row[3], "review not counted (codex)", "{row:?}");
}

#[test]
fn an_unreadable_row_sorts_with_needs_review() {
    let r = Repo::new();
    let pb = "docs/loop/tasks/it-held/codex-2/ORDER.md";
    let mut fake = r.fake();
    // PR 30 (the lower number): ORDER unreadable, an APPROVE record at its head.
    fake["contents"]
        .as_object_mut()
        .unwrap()
        .remove("30:docs/loop/tasks/it-claimed/claude-7/ORDER.md");
    fake["comments"]["30"] =
        json!([{"author": {"login": "k"}, "body": record("codex", "APPROVE")}]);
    // PR 40: readable, no record, so `needs review`; PR 20 is dropped.
    fake["prs"] = json!([
        {"number": 30, "title": "[claude] claimed work", "headRefOid": HEAD_A, "isDraft": false},
        {"number": 40, "title": "[codex] other work", "headRefOid": HEAD_B, "isDraft": false},
    ]);
    fake["files"]["40"] = json!([pb]);
    fake["contents"][format!("40:{pb}")] = fake["contents"][format!("20:{pb}")].clone();
    let rows = next_rows(&r, fake);
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert!(rows[0][1].contains("#30"), "{rows:?}");
    assert!(rows[0][3].starts_with("ORDER unreadable"), "{rows:?}");
    assert_eq!(rows[1][3], "needs review", "{rows:?}");
}
