//! The lab registry's contract at the CLI: where it is found, which commands consult it, and what a
//! refusal looks like. Every case runs the real binary in a scratch git repository with its own
//! `UOR_HARNESS_LIVE`, so nothing here touches shared state.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FOUR: [&str; 4] = ["antigravity", "claude", "deepseek", "kimi"];

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_uor-harness")
}

struct Scratch {
    _dir: tempfile::TempDir,
    repo: PathBuf,
    live: PathBuf,
}

impl Scratch {
    /// A git repository holding the REAL `docs/harness/labs.json`.
    fn new() -> Scratch {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("repo");
        let live = dir.path().join("live");
        std::fs::create_dir_all(repo.join("docs/harness")).expect("harness dir");
        std::fs::create_dir_all(&live).expect("live");
        let real = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/harness/labs.json");
        std::fs::copy(real, repo.join("docs/harness/labs.json")).expect("copy labs.json");
        let init = git(&repo, &["init", "-q"]);
        assert!(init.status.success(), "git init: {init:?}");
        Scratch {
            _dir: dir,
            repo,
            live,
        }
    }

    fn run_in(&self, cwd: &Path, lab: &str, args: &[&str]) -> Output {
        Command::new(bin())
            .args(args)
            .current_dir(cwd)
            .env("UOR_HARNESS_LIVE", &self.live)
            .env("UOR_LAB", lab)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("run uor-harness")
    }
}

fn git(cwd: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .expect("run git")
}

fn files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read_dir").flatten() {
        let path = entry.path();
        if path.file_name().is_some_and(|n| n == ".git") {
            continue;
        }
        if path.is_dir() {
            files_under(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// The registry is found from the repository root, not the working directory: run from a
/// subdirectory, a registered lab must not be refused for a registry "missing" under that subdirectory.
#[test]
fn the_registry_is_found_from_a_subdirectory() {
    let s = Scratch::new();
    let sub = s.repo.join("crates/deep/er");
    std::fs::create_dir_all(&sub).expect("subdir");
    let out = s.run_in(&sub, "claude", &["team", "claims"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr(&out));
    assert!(!stderr(&out).contains("lab registry"), "{}", stderr(&out));
    // `team claims` is a read and no longer consults the registry, so it cannot pin the lookup. A
    // `--dry-run` receipt consults it (a recording command) and writes nothing.
    let out = s.run_in(
        &sub,
        "claude",
        &["receipt", "--kind", "heartbeat", "-m", "m1", "--dry-run"],
    );
    assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr(&out));
    assert!(!stderr(&out).contains("lab registry"), "{}", stderr(&out));
}

/// Inside a git repository with no registry of its own, a recording command refuses: it must not
/// climb out of that repository and borrow the registry of one that encloses it. This is the shape
/// of an old worktree under `.worktrees/` sitting inside the primary checkout.
#[test]
fn a_nested_repository_without_a_registry_refuses_instead_of_borrowing_one() {
    let s = Scratch::new();
    let inner = s.repo.join("inner");
    std::fs::create_dir_all(inner.join("sub")).expect("inner dir");
    let init = git(&inner, &["init", "-q"]);
    assert!(init.status.success(), "git init inner: {init:?}");
    let out = s.run_in(
        &inner.join("sub"),
        "claude",
        &["receipt", "--kind", "heartbeat", "-m", "m1", "--dry-run"],
    );
    assert_eq!(out.status.code(), Some(2), "stderr: {}", stderr(&out));
    assert!(
        stderr(&out).contains("cannot read the lab registry"),
        "{}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("inner"),
        "the error must name the inner repository's path: {}",
        stderr(&out)
    );
}

/// A read does not record under the lab, so it must not be refused for the lab's name.
#[test]
fn a_read_only_command_works_with_any_lab_name() {
    let s = Scratch::new();
    for args in [&["team", "claims"][..], &["multilab", "claims"][..]] {
        let out = s.run_in(&s.repo, "gemini", args);
        assert_eq!(out.status.code(), Some(0), "{args:?}: {}", stderr(&out));
    }
}

/// The CLI contract of a refusal: exit 2, all four labs named, and nothing written anywhere.
#[test]
fn an_unregistered_lab_is_refused_by_heartbeat_with_nothing_written() {
    let s = Scratch::new();
    let before = {
        let mut v = Vec::new();
        files_under(&s.repo, &mut v);
        v
    };
    assert_eq!(before.len(), 1, "scratch repo should hold only labs.json");

    let out = s.run_in(&s.repo, "gemini", &["heartbeat", "-m", "m1"]);
    assert_eq!(out.status.code(), Some(2), "stderr: {}", stderr(&out));
    for lab in FOUR {
        assert!(
            stderr(&out).contains(lab),
            "{lab} missing: {}",
            stderr(&out)
        );
    }
    // Codex left the loop (claude-138): its name is refused the same way.
    let out = s.run_in(&s.repo, "codex", &["heartbeat", "-m", "m1"]);
    assert_eq!(out.status.code(), Some(2), "stderr: {}", stderr(&out));

    let mut after = Vec::new();
    files_under(&s.repo, &mut after);
    assert_eq!(after, before, "the refusal wrote into the repository");
    let mut live = Vec::new();
    files_under(&s.live, &mut live);
    assert!(live.is_empty(), "the refusal wrote to live: {live:?}");
}

/// The hint a lab reads when it has NO identity lists exactly the labs the registry does.
#[test]
fn the_no_identity_hints_list_the_registered_labs() {
    use uor_r4_harness::receipt::{HEARTBEAT_UNATTRIBUTED_WARNING, UNATTRIBUTED_WARNING};
    for (name, text) in [
        ("UNATTRIBUTED_WARNING", UNATTRIBUTED_WARNING),
        (
            "HEARTBEAT_UNATTRIBUTED_WARNING",
            HEARTBEAT_UNATTRIBUTED_WARNING,
        ),
    ] {
        for lab in FOUR {
            assert!(text.contains(lab), "{name} omits {lab}");
        }
        assert!(
            text.contains(&format!("export UOR_LAB=<{}>", FOUR.join("|"))),
            "{name} must name exactly the registered labs: {text}"
        );
    }
}
