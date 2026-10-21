//! A check-in surfaces what the lab has not read: the durable fix for the missed-CONTEST problem.
//!
//! The marker mechanism already existed in the notes module; what did not exist was a reason for a
//! lab to LOOK. A lab checks in every five minutes and never thinks to run the notes command, so a
//! peer note posted mid-task went unseen until somebody happened to mention it. These tests pin the
//! wiring: the check-in shows the floor, never advances the marker, and can be suppressed.

use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_uor-harness")
}

struct Scratch {
    _dir: tempfile::TempDir,
    repo: PathBuf,
    live: PathBuf,
}

impl Scratch {
    fn new() -> Scratch {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("repo");
        let live = dir.path().join("live");
        std::fs::create_dir_all(repo.join("docs/harness")).expect("repo");
        // The lab registry: a recording command refuses a lab it cannot check.
        std::fs::copy(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/harness/labs.json"),
            repo.join("docs/harness/labs.json"),
        )
        .expect("labs.json");
        std::fs::create_dir_all(live.join("notes")).expect("notes dir");
        Scratch {
            _dir: dir,
            repo,
            live,
        }
    }

    /// One note in the JSONL shape the reader parses, at the path the cache helper builds.
    fn note(&self, id: i64, issue: i64) {
        let path = self.live.join("notes/deepseek-notes.jsonl");
        let mut text = std::fs::read_to_string(&path).unwrap_or_default();
        let line = format!(
            "{{\"id\":{id},\"issue\":{issue},\"author\":\"kimi\",\"created_at\":\"2026-10-15T20:00:00Z\",\"first_line\":\"CONTEST: reading the sheet\"}}\n"
        );
        text.push_str(&line);
        std::fs::write(&path, text).expect("cache");
    }

    fn run(&self, args: &[&str]) -> Output {
        let repo = self.repo.to_str().expect("utf8");
        let mut full: Vec<&str> = vec!["--repo", repo];
        full.extend_from_slice(args);
        Command::new(bin())
            .args(&full)
            .env("UOR_HARNESS_LIVE", &self.live)
            .env("UOR_LAB", "deepseek")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("run uor-harness")
    }
}

#[test]
fn a_check_in_surfaces_unread_notes_without_marking_them_seen() {
    let s = Scratch::new();
    s.note(101, 2029);

    let first = s.run(&["heartbeat", "-m", "m1", "--line", "no model line - tooling"]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let out = String::from_utf8_lossy(&first.stdout).to_string();
    let err = String::from_utf8_lossy(&first.stderr).to_string();
    // BOTH streams: the defect this pins printed the same line on stderr too, and a stdout-only
    // assertion is exactly why the wider suite stayed green.
    let occurrences = format!("{out}{err}").matches("unread note(s)").count();
    assert_eq!(
        occurrences, 1,
        "exactly one render across both streams: {out} | {err}"
    );
    assert!(
        !err.contains("unread note(s)"),
        "on stdout, not stderr: {err}"
    );
    assert!(out.contains("unread note(s) on #2029"), "surfaced: {out}");
    assert!(
        out.contains("at least"),
        "a cached read reports a floor: {out}"
    );
    assert!(
        out.contains("uor-harness notes"),
        "the reader is named: {out}"
    );

    // A check-in must NOT mark notes seen: only the reader does that, after showing them in full.
    let again = s.run(&["heartbeat", "-m", "m1", "--line", "no model line - tooling"]);
    assert!(
        String::from_utf8_lossy(&again.stdout).contains("unread note(s) on #2029"),
        "the note is still unread after a check-in"
    );

    let quiet = s.run(&["heartbeat", "-m", "m1", "--line", "x", "--no-notes"]);
    assert!(
        !String::from_utf8_lossy(&quiet.stdout).contains("unread note(s)"),
        "--no-notes suppresses the nudge"
    );
}

#[test]
fn a_check_in_with_no_cache_prints_nothing_extra_and_still_succeeds() {
    let s = Scratch::new();
    let out = s.run(&["heartbeat", "-m", "m1", "--line", "no model line - tooling"]);
    assert!(out.status.success());
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("unread note(s)"),
        "no cache, no line: a check-in that always printed 0 unread would be skipped"
    );
}
