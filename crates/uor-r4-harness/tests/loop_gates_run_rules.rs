//! Each run rule of `uor-harness loop gates` pinned against a temporary git repository with fake
//! gates. Every child process has every `GIT_*` variable removed, and each repository is checked
//! to sit inside its own temporary directory before anything writes to it.

use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB").env_remove("CARGO")
}

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
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

    fn run(&self, args: &[&str]) -> Output {
        clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop", "gates"])
            .args(args)
            .env("UOR_LAB", "claude")
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs")
    }

    fn ledger(&self) -> String {
        std::fs::read_to_string(self.root.join("local/claude/gates.jsonl")).unwrap_or_default()
    }
}

/// `gates` is a list of (id, shell script, break append text, break expect text).
fn repo(gates: &[(&str, &str, &str, &str)]) -> Repo {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap().join("repo");
    assert!(root.starts_with(tmp.path().canonicalize().unwrap()));
    std::fs::create_dir_all(&root).unwrap();
    let r = Repo { _tmp: tmp, root };
    r.git(&["init", "-q", "-b", "main"]);
    r.git(&["config", "user.email", "t@example.invalid"]);
    r.git(&["config", "user.name", "t"]);
    r.write(".gitignore", "/local/\n");
    r.write(
        "docs/harness/labs.json",
        r#"{"labs":{"claude":{"vendor":"a","client":"b","headless":"c","open_pr_limit":1}}}"#,
    );
    r.write("src/input.txt", "fine\n");
    let entries: Vec<String> = gates
        .iter()
        .map(|(id, script, append, expect)| {
            serde_json::json!({
                "id": id, "proves": "a fake gate", "command": ["sh", "-c", script],
                "reads": ["src/**"], "quick": true,
                "break": {"file": "src/input.txt", "append": append, "expect": expect}
            })
            .to_string()
        })
        .collect();
    r.write(
        "docs/harness/gates.json",
        &format!(r#"{{"version":1,"gates":[{}]}}"#, entries.join(",")),
    );
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "init"]);
    r
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn line<'a>(out: &'a str, id: &str) -> &'a str {
    out.lines()
        .find(|l| l.split_whitespace().next() == Some(id))
        .unwrap_or_else(|| panic!("no line for {id} in:\n{out}"))
}

const OK: (&str, &str, &str, &str) = ("ok", "exit 0", "x", "x");

#[test]
fn a_gate_that_cannot_run_is_refused() {
    let r = repo(&[OK]);
    // Replace the command with a binary that does not exist.
    let g = std::fs::read_to_string(r.root.join("docs/harness/gates.json")).unwrap();
    r.write(
        "docs/harness/gates.json",
        &g.replace(r#"["sh","-c","exit 0"]"#, r#"["uor-no-such-binary-xyz"]"#),
    );
    let o = r.run(&[]);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        line(&stdout(&o), "ok").contains("REFUSED"),
        "{}",
        stdout(&o)
    );
}

#[test]
fn later_gates_read_unknown_after_a_red() {
    let r = repo(&[("red", "exit 3", "x", "x"), OK]);
    let out = stdout(&r.run(&[]));
    assert!(line(&out, "red").contains("RED"), "{out}");
    assert!(line(&out, "ok").contains("UNKNOWN"), "{out}");
}

#[test]
fn keep_going_runs_the_gates_after_a_red() {
    let r = repo(&[("red", "exit 3", "x", "x"), OK]);
    let out = stdout(&r.run(&["--keep-going"]));
    assert!(line(&out, "ok").contains("GREEN"), "{out}");
}

#[test]
fn the_true_exit_code_is_kept_in_the_log_and_ledger() {
    let r = repo(&[("red", "echo boom; exit 3", "x", "x")]);
    let out = stdout(&r.run(&[]));
    assert!(line(&out, "red").contains("exit 3"), "{out}");
    let runs = r.root.join("local/claude/gates");
    let dir = std::fs::read_dir(&runs)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(std::fs::read_to_string(dir.join("red.exit")).unwrap(), "3");
    assert!(std::fs::read_to_string(dir.join("red.log"))
        .unwrap()
        .contains("boom"));
    assert!(r.ledger().contains(r#""exit":3"#), "{}", r.ledger());
}

#[test]
fn a_change_to_a_read_file_during_the_run_voids_the_gate() {
    let r = repo(&[("edits", "echo changed >> src/input.txt; exit 0", "x", "x")]);
    let out = stdout(&r.run(&[]));
    assert!(line(&out, "edits").contains("VOID"), "{out}");
}

#[test]
fn a_ledger_result_is_stale_after_an_uncommitted_edit() {
    let r = repo(&[OK]);
    assert!(r.run(&[]).status.success());
    let fresh = stdout(&r.run(&["--status"]));
    assert!(!line(&fresh, "ok").contains("STALE"), "{fresh}");
    r.write("src/input.txt", "edited\n");
    let stale = stdout(&r.run(&["--status"]));
    assert!(line(&stale, "ok").contains("STALE"), "{stale}");
}

/// A gate that fails when the break's line is present, with its own message.
const CHECKS: &str = "if grep -q PLANTED src/input.txt; then echo 'input is broken'; exit 1; fi";

fn worktrees_under_local(r: &Repo) -> usize {
    let o = clean(&mut Command::new("git"))
        .args(["worktree", "list"])
        .current_dir(&r.root)
        .output()
        .unwrap();
    stdout(&o).lines().filter(|l| l.contains("/local/")).count()
}

#[test]
fn a_break_red_for_its_own_reason_passes() {
    let r = repo(&[("checks", CHECKS, "PLANTED\n", "input is broken")]);
    let o = r.run(&["--break", "checks"]);
    assert!(
        line(&stdout(&o), "checks").contains("CAUGHT"),
        "{}",
        stdout(&o)
    );
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(worktrees_under_local(&r), 0);
}

#[test]
fn a_break_red_for_another_reason_is_refused() {
    // Green at HEAD, red once planted, but with text other than the declared one.
    let other = "if grep -q PLANTED src/input.txt; then echo unrelated; exit 101; fi";
    let r = repo(&[("other", other, "PLANTED\n", "input is broken")]);
    let o = r.run(&["--break", "other"]);
    assert!(
        line(&stdout(&o), "other").contains("REFUSED"),
        "{}",
        stdout(&o)
    );
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(worktrees_under_local(&r), 0);
}

#[test]
fn a_break_that_stays_green_is_a_defect() {
    let r = repo(&[OK]);
    let o = r.run(&["--break", "ok"]);
    assert!(line(&stdout(&o), "ok").contains("DEFECT"), "{}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn break_all_turns_each_gate_red_for_its_own_reason() {
    // Each break plants its own marker. Gate two fails with a different message if it ever sees
    // gate one's marker, so one shared worktree turns gate two REFUSED and fails this test.
    let first = "if grep -q MARK_A src/input.txt; then echo 'first saw its own'; exit 1; fi";
    let second =
        "if grep -q MARK_A src/input.txt; then echo 'second saw the first marker'; exit 3; \
                  elif grep -q MARK_B src/input.txt; then echo 'second saw its own'; exit 2; fi";
    let r = repo(&[
        ("checks", first, "MARK_A\n", "first saw its own"),
        ("second", second, "MARK_B\n", "second saw its own"),
    ]);
    let o = r.run(&["--break", "all"]);
    let out = stdout(&o);
    assert!(line(&out, "checks").contains("CAUGHT"), "{out}");
    assert!(line(&out, "second").contains("CAUGHT"), "{out}");
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(worktrees_under_local(&r), 0);
    // The planted fault never reaches the real tree.
    let input = std::fs::read_to_string(r.root.join("src/input.txt")).unwrap();
    assert_eq!(input, "fine\n");
}

#[test]
fn watch_and_other_actions_are_refused() {
    let r = repo(&[OK]);
    let w = r.run(&["--watch"]);
    assert_eq!(w.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&w.stderr).contains("loop PR 11"),
        "{w:?}"
    );
    let o = clean(&mut Command::new(BIN))
        // `next` was the example until loop PR 7b built it; `drive` is loop PR 10.
        .args(["--repo", r.root.to_str().unwrap(), "loop", "drive"])
        .env("UOR_LAB", "claude")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("not built yet"));
}

/// Rewrite `docs/harness/gates.json` with `from` replaced by `to`, asserting the text was there.
fn edit_gates(r: &Repo, from: &str, to: &str) {
    let g = std::fs::read_to_string(r.root.join("docs/harness/gates.json")).unwrap();
    assert!(g.contains(from), "{from} not in {g}");
    r.write("docs/harness/gates.json", &g.replace(from, to));
}

#[test]
fn a_break_file_outside_the_tree_exits_2() {
    for bad in [
        "/tmp/uor-planted",
        "../outside.txt",
        "src/../../outside.txt",
        "./src/input.txt",
    ] {
        let r = repo(&[OK]);
        edit_gates(
            &r,
            r#""file":"src/input.txt""#,
            &format!(r#""file":"{bad}""#),
        );
        let o = r.run(&[]);
        assert_eq!(o.status.code(), Some(2), "{bad}: {o:?}");
        assert!(
            String::from_utf8_lossy(&o.stderr).contains("\"ok\""),
            "{o:?}"
        );
    }
}

#[test]
fn a_bad_or_duplicate_gate_id_exits_2() {
    for id in ["Bad", "-lead", "a/b", "..", ""] {
        let r = repo(&[(id, "exit 0", "x", "x")]);
        assert_eq!(r.run(&[]).status.code(), Some(2), "{id:?}");
    }
    let r = repo(&[OK, OK]);
    assert_eq!(r.run(&[]).status.code(), Some(2));
}

#[test]
fn a_selection_of_zero_gates_exits_2() {
    let r = repo(&[OK]);
    edit_gates(&r, r#""quick":true"#, r#""quick":false"#);
    let o = r.run(&["--quick"]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(r.ledger().is_empty());
}

#[test]
fn a_break_on_a_gate_already_red_at_head_is_refused() {
    // Red with the declared text even before the break is planted: proves nothing about the fault.
    let r = repo(&[(
        "red",
        "echo 'input is broken'; exit 1",
        "PLANTED\n",
        "input is broken",
    )]);
    let o = r.run(&["--break", "red"]);
    let out = stdout(&o);
    assert!(line(&out, "red").contains("REFUSED"), "{out}");
    assert!(
        line(&out, "red").contains("gate not green before the break (RED: exit 1)"),
        "{out}"
    );
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(worktrees_under_local(&r), 0);
}

#[test]
fn a_gate_whose_reads_match_no_file_is_refused() {
    let r = repo(&[OK]);
    edit_gates(
        &r,
        r#""reads":["src/**"]"#,
        r#""reads":["nothing-here/**"]"#,
    );
    let o = r.run(&[]);
    let out = stdout(&o);
    assert!(line(&out, "ok").contains("REFUSED"), "{out}");
    assert!(line(&out, "ok").contains("reads match no file"), "{out}");
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn a_gate_with_one_matching_read_glob_still_runs() {
    let r = repo(&[OK]);
    edit_gates(
        &r,
        r#""reads":["src/**"]"#,
        r#""reads":["nothing-here/**","src/**"]"#,
    );
    let out = stdout(&r.run(&[]));
    assert!(line(&out, "ok").contains("GREEN"), "{out}");
}

#[test]
fn a_signal_death_keeps_the_signal() {
    let r = repo(&[("killed", "kill -TERM $$", "x", "x")]);
    let out = stdout(&r.run(&[]));
    assert!(line(&out, "killed").contains("RED"), "{out}");
    let runs = r.root.join("local/claude/gates");
    let dir = std::fs::read_dir(&runs)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(
        std::fs::read_to_string(dir.join("killed.exit")).unwrap(),
        "signal 15"
    );
    let ledger = r.ledger();
    assert!(ledger.contains(r#""exit":null"#), "{ledger}");
    assert!(ledger.contains(r#""signal":15"#), "{ledger}");
}

#[test]
fn break_refuses_an_uncommitted_gates_json() {
    let r = repo(&[("checks", CHECKS, "PLANTED\n", "input is broken")]);
    edit_gates(
        &r,
        r#""proves":"a fake gate""#,
        r#""proves":"edited, not committed""#,
    );
    let o = r.run(&["--break", "checks"]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("uncommitted"),
        "{o:?}"
    );
    assert_eq!(worktrees_under_local(&r), 0);
}

/// Rewrite `docs/harness/gates.json` through serde_json so a test can change one gate's field.
fn change_gate(r: &Repo, index: usize, edit: impl FnOnce(&mut serde_json::Value)) {
    let p = r.root.join("docs/harness/gates.json");
    let mut v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    edit(&mut v["gates"][index]);
    r.write("docs/harness/gates.json", &v.to_string());
}

#[test]
fn a_gate_without_a_break_exits_2() {
    let r = repo(&[OK]);
    change_gate(&r, 0, |g| {
        g.as_object_mut().unwrap().remove("break");
    });
    let o = r.run(&[]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(r.ledger().is_empty());
}

#[test]
fn the_real_repo_gates_json_loads() {
    // Non-mutating: the pub loader only reads and parses `docs/harness/gates.json`.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let file = uor_r4_harness::loopgates::load(&root)
        .unwrap_or_else(|e| panic!("the real docs/harness/gates.json must load: {e}"));
    assert!(!file.gates.is_empty());
}

#[test]
fn quick_runs_only_quick_gates_and_full_runs_both() {
    let r = repo(&[("fast", "exit 0", "x", "x"), ("slow", "exit 0", "x", "x")]);
    change_gate(&r, 1, |g| g["quick"] = serde_json::json!(false));
    let quick = stdout(&r.run(&["--quick"]));
    assert!(line(&quick, "fast").contains("GREEN"), "{quick}");
    assert!(
        !quick
            .lines()
            .any(|l| l.split_whitespace().next() == Some("slow")),
        "{quick}"
    );
    let full = stdout(&r.run(&["--full"]));
    assert!(line(&full, "fast").contains("GREEN"), "{full}");
    assert!(line(&full, "slow").contains("GREEN"), "{full}");
}

#[test]
fn a_ledger_row_has_a_commit_and_numeric_seconds() {
    let r = repo(&[OK]);
    assert!(r.run(&[]).status.success());
    let ledger = r.ledger();
    let row: serde_json::Value =
        serde_json::from_str(ledger.lines().next().expect("a row")).unwrap();
    assert!(
        row["commit"].as_str().is_some_and(|c| !c.is_empty()),
        "{ledger}"
    );
    assert!(row["seconds"].is_number(), "{ledger}");
}

#[test]
fn keep_going_still_exits_1_after_a_red() {
    let r = repo(&[("red", "exit 3", "x", "x"), OK]);
    let o = r.run(&["--keep-going"]);
    let out = stdout(&o);
    assert!(line(&out, "ok").contains("GREEN"), "{out}");
    assert_eq!(o.status.code(), Some(1), "{o:?}");
}

/// The run folders under `local/claude/gates/`.
fn run_folders(r: &Repo) -> usize {
    std::fs::read_dir(r.root.join("local/claude/gates")).map_or(0, |d| d.count())
}

#[test]
fn a_refusal_before_any_gate_leaves_no_run_folder() {
    let r = repo(&[("checks", CHECKS, "PLANTED\n", "input is broken")]);
    assert_eq!(r.run(&["--break", "nope"]).status.code(), Some(2));
    assert_eq!(run_folders(&r), 0);
    change_gate(&r, 0, |g| g["quick"] = serde_json::json!(false));
    assert_eq!(r.run(&["--quick"]).status.code(), Some(2));
    assert_eq!(run_folders(&r), 0);
    // gates.json is now uncommitted.
    assert_eq!(r.run(&["--break", "checks"]).status.code(), Some(2));
    assert_eq!(run_folders(&r), 0);
}

#[test]
fn a_break_keeps_the_pre_check_log_apart() {
    let r = repo(&[("checks", CHECKS, "PLANTED\n", "input is broken")]);
    let o = r.run(&["--break", "checks"]);
    assert!(line(&stdout(&o), "checks").contains("CAUGHT"), "{o:?}");
    let dir = std::fs::read_dir(r.root.join("local/claude/gates"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let read = |f: &str| std::fs::read_to_string(dir.join(f)).unwrap();
    assert_eq!(read("checks.pre.exit"), "0");
    assert!(!read("checks.pre.log").contains("input is broken"));
    assert_eq!(read("checks.exit"), "1");
    assert!(read("checks.log").contains("input is broken"));
}

/// A file outside the repository's temporary directory, in a temporary directory of its own.
fn outside_file() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().canonicalize().unwrap().join("outside.txt");
    std::fs::write(&p, "outside\n").unwrap();
    (tmp, p)
}

#[test]
fn a_break_file_that_is_a_tracked_symlink_is_refused() {
    let (_out_tmp, outside) = outside_file();
    let r = repo(&[("checks", CHECKS, "PLANTED\n", "input is broken")]);
    assert!(!outside.starts_with(r._tmp.path().canonicalize().unwrap()));
    std::os::unix::fs::symlink(&outside, r.root.join("src/link.txt")).unwrap();
    change_gate(&r, 0, |g| {
        g["break"]["file"] = serde_json::json!("src/link.txt")
    });
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "symlink break file"]);
    let o = r.run(&["--break", "checks"]);
    let out = stdout(&o);
    assert_eq!(
        std::fs::read_to_string(&outside).unwrap(),
        "outside\n",
        "the outside file changed"
    );
    assert!(line(&out, "checks").contains("REFUSED"), "{out}");
    assert!(line(&out, "checks").contains("is a symlink"), "{out}");
    assert_eq!(worktrees_under_local(&r), 0);
}

#[test]
fn a_break_file_swapped_for_a_symlink_during_the_pre_check_is_refused() {
    // The break file is outside the gate's reads, so the swap does not void the pre-check: only
    // a containment check at the point of write stops the append reaching the outside file.
    let (_out_tmp, outside) = outside_file();
    let swap = format!(
        "rm -f notes/brk.txt && ln -s '{}' notes/brk.txt; exit 0",
        outside.display()
    );
    let r = repo(&[("swaps", &swap, "PLANTED\n", "x")]);
    r.write("notes/brk.txt", "plain\n");
    change_gate(&r, 0, |g| {
        g["break"]["file"] = serde_json::json!("notes/brk.txt")
    });
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "plain break file"]);
    let o = r.run(&["--break", "swaps"]);
    let out = stdout(&o);
    assert_eq!(
        std::fs::read_to_string(&outside).unwrap(),
        "outside\n",
        "the outside file changed"
    );
    assert!(line(&out, "swaps").contains("REFUSED"), "{out}");
}

/// Whether `kill -0 <pid>` reports that no such process exists.
fn no_such_process(pid: u32) -> bool {
    let o = Command::new("kill")
        .args(["-0", &pid.to_string()])
        .env("LC_ALL", "C")
        .output()
        .unwrap();
    !o.status.success() && String::from_utf8_lossy(&o.stderr).contains("No such process")
}

#[test]
fn the_sweep_removes_only_this_labs_dead_runs_scratch_worktrees() {
    let r = repo(&[("checks", CHECKS, "PLANTED\n", "input is broken")]);
    let dead = 2_000_000_000u32;
    assert!(no_such_process(dead), "pid {dead} must not be live");
    let live = std::process::id();
    let wts = [
        format!("local/claude/gates/1-{dead}/scratch/wt-own-dead"),
        format!("local/claude/gates/1-{live}/scratch/wt-own-live"),
        format!("local/kimi/gates/1-{dead}/scratch/wt-other-dead"),
    ];
    for wt in &wts {
        assert!(r.root.join(wt).starts_with(&r.root));
        r.git(&["worktree", "add", "--detach", "--quiet", wt, "HEAD"]);
        r.write(&format!("{wt}/untracked.txt"), "keep me\n");
    }
    assert_eq!(worktrees_under_local(&r), 3);
    let o = r.run(&["--break", "checks"]);
    assert!(line(&stdout(&o), "checks").contains("CAUGHT"), "{o:?}");
    let list = clean(&mut Command::new("git"))
        .args(["worktree", "list"])
        .current_dir(&r.root)
        .output()
        .unwrap();
    let list = stdout(&list);
    assert!(!list.contains("wt-own-dead"), "{list}");
    assert!(!r.root.join(&wts[0]).exists());
    for wt in &wts[1..] {
        assert!(list.contains(wt.as_str()), "{wt} still registered: {list}");
        assert_eq!(
            std::fs::read_to_string(r.root.join(wt).join("untracked.txt")).unwrap(),
            "keep me\n"
        );
    }
    for wt in &wts[1..] {
        r.git(&["worktree", "remove", "--force", wt]);
    }
}
