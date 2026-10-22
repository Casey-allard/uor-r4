//! The gate progress window (loop PR 11): what it shows, when it opens, and when it gives up.
//!
//! The pure status and rendering functions are tested directly. The command-level test proves that
//! --watch in a non-terminal (headless) run leaves the window off and the run unaffected.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::Duration;

use uor_r4_harness::loopgates::{
    estimate_left, last_seconds, watch_decision, watch_gives_up, watch_line, watch_loop,
    WatchDecision, WatchResult, WatchStatus, WATCH_GIVE_UP,
};

fn status(gates: &[&str], finished: &[(&str, bool, f64)], current: Option<usize>) -> WatchStatus {
    let mut s = WatchStatus::new(
        "run-1",
        "claude",
        gates.iter().map(|g| g.to_string()).collect(),
    );
    s.current = current;
    for (id, red, seconds) in finished {
        s.results.push(WatchResult {
            id: (*id).to_string(),
            verdict: if *red {
                uor_r4_harness::loopgates::Verdict::Red
            } else {
                uor_r4_harness::loopgates::Verdict::Green
            },
            seconds: *seconds,
        });
    }
    s
}

#[test]
fn the_window_shows_gate_n_of_m_the_running_gate_and_an_estimate() {
    let s = status(&["fmt", "tests", "clippy"], &[("fmt", false, 1.0)], Some(1));
    let line = watch_line(&s, Some(4.2));
    assert!(line.contains("gate 2 of 3"), "{line}");
    assert!(line.contains("tests"), "{line}");
    assert!(line.contains("estimate 4.2s left"), "{line}");
}

#[test]
fn a_first_run_says_there_is_no_estimate_yet() {
    let s = status(&["fmt", "tests"], &[], Some(0));
    assert!(watch_line(&s, None).contains("no estimate yet"));
}

#[test]
fn the_estimate_sums_the_remaining_gates_from_the_ledger() {
    let s = status(&["fmt", "tests", "clippy"], &[("fmt", false, 1.0)], Some(1));
    let mut last = std::collections::BTreeMap::new();
    last.insert("fmt".to_string(), 999.0); // already finished: not in the estimate
    last.insert("tests".to_string(), 2.0);
    last.insert("clippy".to_string(), 3.5);
    assert_eq!(estimate_left(&s, &last), Some(5.5));
}

#[test]
fn a_gate_with_no_ledger_row_is_left_out_and_all_unknown_is_none() {
    let s = status(&["fmt", "tests", "clippy"], &[("fmt", false, 1.0)], Some(1));
    let mut last = std::collections::BTreeMap::new();
    last.insert("tests".to_string(), 2.0);
    assert_eq!(estimate_left(&s, &last), Some(2.0));
    let empty = std::collections::BTreeMap::new();
    assert_eq!(estimate_left(&s, &empty), None);
}

#[test]
fn reds_are_named_as_they_land() {
    let s = status(
        &["fmt", "tests", "clippy"],
        &[("fmt", false, 0.4), ("tests", true, 12.0)],
        Some(2),
    );
    let line = watch_line(&s, None);
    assert!(line.contains("RED: tests"), "{line}");
    assert!(!line.contains("RED: fmt"), "{line}");
}

#[test]
fn the_window_opens_only_for_a_full_run_in_a_terminal() {
    assert_eq!(
        watch_decision(false, true, true, false),
        WatchDecision::Off("not asked")
    );
    assert_eq!(
        watch_decision(true, false, true, false),
        WatchDecision::Off("not a full run")
    );
    assert_eq!(
        watch_decision(true, true, true, true),
        WatchDecision::Off("CI")
    );
    assert_eq!(
        watch_decision(true, true, false, false),
        WatchDecision::Off("not a terminal")
    );
    assert_eq!(watch_decision(true, true, true, false), WatchDecision::Open);
}

#[test]
fn the_window_gives_up_after_five_minutes_without_change() {
    assert_eq!(WATCH_GIVE_UP, Duration::from_secs(300));
    assert!(!watch_gives_up(Duration::from_secs(299)));
    assert!(watch_gives_up(Duration::from_secs(300)));
    assert!(watch_gives_up(Duration::from_secs(900)));
}

#[test]
fn the_status_file_round_trips_through_json() {
    let s = status(&["fmt", "tests"], &[("fmt", true, 0.4)], None);
    let text = serde_json::to_string(&s).unwrap();
    let back: WatchStatus = serde_json::from_str(&text).unwrap();
    assert_eq!(s, back);
}

#[test]
fn a_finished_status_carries_every_result_and_done() {
    let mut s = status(&["fmt"], &[("fmt", false, 0.4)], None);
    s.done = true;
    let text = serde_json::to_string(&s).unwrap();
    let back: WatchStatus = serde_json::from_str(&text).unwrap();
    assert!(back.done);
    assert_eq!(back.results.len(), 1);
    assert!(watch_line(&back, None).contains("gate 1 of 1"));
}

/// The loop itself: an unchanged status file prints the giving-up line and returns. The wait
/// closure is bounded, so a missing give-up branch fails this test instead of hanging it.
#[test]
fn the_window_gives_up_and_says_so_when_the_status_never_changes() {
    let mut out: Vec<u8> = Vec::new();
    let mut read = || String::new();
    let mut calls = 0u32;
    let mut wait = || {
        calls += 1;
        assert!(
            calls <= 100,
            "the window never gave up; its give-up branch is missing"
        );
        std::thread::sleep(Duration::from_millis(2));
    };
    watch_loop(
        &mut read,
        &mut out,
        &BTreeMap::new(),
        Duration::from_millis(1),
        &mut wait,
    )
    .unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains("giving up after 5 minutes without change"),
        "{text}"
    );
}

/// A finished run closes the window, and it says so.
#[test]
fn the_window_closes_after_the_run() {
    let mut s = status(
        &["fmt", "tests"],
        &[("fmt", false, 0.4), ("tests", false, 2.0)],
        None,
    );
    s.done = true;
    let text = serde_json::to_string(&s).unwrap();
    let mut out: Vec<u8> = Vec::new();
    let mut read = || text.clone();
    let mut wait = || {};
    watch_loop(
        &mut read,
        &mut out,
        &BTreeMap::new(),
        Duration::from_secs(300),
        &mut wait,
    )
    .unwrap();
    let shown = String::from_utf8(out).unwrap();
    assert!(shown.contains("gate 2 of 2"), "{shown}");
    assert!(shown.contains("closed after the run"), "{shown}");
}

/// A red is printed when it lands, before the run ends.
#[test]
fn a_red_is_printed_as_it_lands() {
    let s = status(&["fmt", "tests"], &[("fmt", true, 0.4)], Some(1));
    let text = serde_json::to_string(&s).unwrap();
    let mut out: Vec<u8> = Vec::new();
    let mut read = || text.clone();
    let mut calls = 0u32;
    let mut wait = || {
        calls += 1;
        assert!(
            calls <= 100,
            "the window never gave up; its give-up branch is missing"
        );
        std::thread::sleep(Duration::from_millis(2));
    };
    watch_loop(
        &mut read,
        &mut out,
        &BTreeMap::new(),
        Duration::from_millis(1),
        &mut wait,
    )
    .unwrap();
    let shown = String::from_utf8(out).unwrap();
    assert!(shown.contains("RED fmt after 0.4s"), "{shown}");
}

/// A minimal repository with one always-green gate, so the command can run in a temp directory.
fn repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap();
    let g = |args: &[&str]| -> String {
        let mut cmd = Command::new("git");
        for (k, _) in std::env::vars_os() {
            if k.to_string_lossy().starts_with("GIT_") {
                cmd.env_remove(&k);
            }
        }
        let st = cmd
            .args(args)
            .current_dir(&root)
            .output()
            .expect("git runs");
        assert!(st.status.success(), "git {args:?}: {st:?}");
        String::new()
    };
    g(&["init", "-q", "-b", "main"]);
    g(&["config", "user.email", "t@example.invalid"]);
    g(&["config", "user.name", "t"]);
    let gates = r#"{"version":1,"gates":[{"id":"ok","proves":"it is green","command":["true"],"reads":["docs/harness/gates.json"],"quick":true,"break":{"file":"README.md","append":"x","expect":"x"}}]}"#;
    let labs = r#"{"schema":1,"labs":{"claude":{"vendor":"Anthropic","client":"Claude Code","headless":"claude -p","open_pr_limit":3}}}"#;
    let dir = root.join("docs/harness");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("gates.json"), gates).unwrap();
    std::fs::write(dir.join("labs.json"), labs).unwrap();
    std::fs::write(root.join("README.md"), "# r\n").unwrap();
    g(&["add", "-A"]);
    g(&["commit", "-q", "-m", "init"]);
    tmp
}

fn run(root: &PathBuf, args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_uor-harness"));
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") || k == "UOR_LAB" || k == "CI" {
            cmd.env_remove(k);
        }
    }
    cmd.args(["--repo", root.to_str().unwrap(), "loop", "gates"])
        .args(args)
        .env("UOR_LAB", "claude")
        .current_dir(root)
        .output()
        .expect("uor-harness runs")
}

#[test]
fn watch_in_a_headless_run_is_off_and_does_not_change_the_result() {
    let tmp = repo();
    let root = tmp.path().canonicalize().unwrap();
    let o = run(&root, &["--full", "--watch"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(err.contains("watch off (not a terminal)"), "{err}");
    // No status file: the window never opened, though the run folder exists.
    let runs = std::fs::read_dir(root.join("local/claude/gates")).unwrap();
    for entry in runs {
        let p = entry.unwrap().path();
        assert!(!p.join("status.json").exists(), "status file: {p:?}");
    }
    assert!(String::from_utf8_lossy(&o.stdout).contains("GREEN"));
}

#[test]
fn watch_with_a_quick_run_is_refused() {
    let tmp = repo();
    let root = tmp.path().canonicalize().unwrap();
    let o = run(&root, &["--quick", "--watch"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(err.contains("needs a full run"), "{err}");
}

#[test]
fn the_ledger_feeds_the_estimate() {
    let tmp = tempfile::tempdir().unwrap();
    let ledger = tmp.path().join("gates.jsonl");
    let row = r#"{"gate":"tests","verdict":"GREEN","seconds":4.5,"commit":"abc","tree":"def","exit":0,"ts":1}"#;
    let unknown = r#"{"gate":"tests","verdict":"UNKNOWN","seconds":0.0,"commit":"abc","tree":"def","exit":null,"ts":2}"#;
    std::fs::write(&ledger, format!("{row}\n{unknown}\n")).unwrap();
    let last = last_seconds(&ledger);
    assert_eq!(
        last.get("tests"),
        Some(&4.5),
        "an UNKNOWN row written after a red must not zero the estimate"
    );
}
