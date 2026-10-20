//! `heartbeat --print` shows the card and records nothing; a check-in leads with the same card.
//!
//! BOOTSTRAP tells a new lab to verify its identity with `--print`. Python's `--print` still wrote a
//! receipt, so the verification step itself polluted the record it was checking.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const REGISTRY: &str = r#"{"milestones": {"m1": {"issue": 2029, "id": "M1",
  "headline": {"label": "reply acceptable", "current": 37, "target": 116, "of": 232},
  "lines": [{"name": "exact addressed memory", "count": 0, "of": 3},
            {"name": "evaluation integrity", "count": 1, "of": 3}],
  "next": "the registered step"}}}"#;

const CARD: &str = "M1 #2029 · M1: reply acceptable 37/232 -> target 116/232 · line: evaluation \
                    integrity · count 1/3 · next: the registered step · [moves the headline reply \
                    acceptable 37/232 -> target 116/232]";

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
        std::fs::write(repo.join("docs/harness/milestones.json"), REGISTRY).expect("registry");
        Scratch {
            _dir: dir,
            repo,
            live,
        }
    }

    fn run(&self, args: &[&str]) -> Output {
        let repo = self.repo.to_str().expect("utf8");
        let mut full: Vec<&str> = vec!["--repo", repo];
        full.extend_from_slice(args);
        Command::new(env!("CARGO_BIN_EXE_uor-harness"))
            .args(&full)
            .env("UOR_HARNESS_LIVE", &self.live)
            .env("UOR_LAB", "claude")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("run uor-harness")
    }
}

fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(files_under(&p));
            } else {
                out.push(p);
            }
        }
    }
    out
}

#[test]
fn print_shows_the_card_and_writes_nothing() {
    let s = Scratch::new();
    let out = s.run(&["heartbeat", "-m", "m1", "--print"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim_end(), CARD);
    assert_eq!(files_under(&s.live), Vec::<PathBuf>::new(), "the live tier");
    let repo_files = files_under(&s.repo);
    assert_eq!(repo_files.len(), 1, "only the registry: {repo_files:?}");
}

#[test]
fn print_takes_the_overrides_python_takes() {
    let s = Scratch::new();
    let out = s.run(&[
        "heartbeat",
        "-m",
        "m1",
        "--print",
        "--line",
        "Exact Addressed Memory",
        "--count",
        "2",
        "--next",
        "x",
    ]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        text.contains(" · line: Exact Addressed Memory · count 2/3 · next: x · "),
        "{text}"
    );
}

/// `--doing` is the presence alias of `--line`, so the card names the line the check-in records.
#[test]
fn the_card_reads_doing_as_the_line() {
    let s = Scratch::new();
    let out = s.run(&[
        "heartbeat",
        "-m",
        "m1",
        "--print",
        "--doing",
        "exact addressed memory",
    ]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        text.contains(" · line: exact addressed memory · count 0/3 · "),
        "{text}"
    );
}

#[test]
fn an_unknown_milestone_exits_2_with_pythons_message() {
    let s = Scratch::new();
    let out = s.run(&["heartbeat", "-m", "x", "--print"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr).trim_end(),
        "heartbeat: unknown milestone 'x'; known: m1"
    );
}

#[test]
fn a_check_in_leads_with_the_card_and_still_records() {
    let s = Scratch::new();
    let out = s.run(&["heartbeat", "-m", "m1", "--no-notes"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some(CARD));
    assert!(lines.any(|l| l.starts_with("checked in: ")), "{text}");
    assert!(!files_under(&s.live).is_empty(), "a check-in still writes");
}
