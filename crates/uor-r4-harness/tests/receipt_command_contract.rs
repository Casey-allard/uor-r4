//! The `receipt` command's contract at the CLI: it refuses what it cannot honour.
//!
//! Three defects reported on #2028, all the same shape - the command accepted something and then
//! ignored it:
//!
//! * an unknown `--kind` was WRITTEN to the shard, and `check` hard-errored on it at parse
//!   (`receipts.parse`), so the harness accepted a record its own reader rejects;
//! * `-m m1` was parsed by the global parser, forwarded, and dropped by the command, so the line
//!   carried no milestone;
//! * every other flag the global parser knows (`--count`, `--headline`, …) was dropped the same way,
//!   on a command whose whole purpose is to record what happened.
//!
//! Each case runs the real binary with `UOR_HARNESS_LIVE` pointed at a scratch directory.

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
        std::fs::create_dir_all(&repo).expect("repo");
        // The probe lab is registered in the scratch repo's own registry: a recording command
        // refuses a lab that `docs/harness/labs.json` does not list.
        std::fs::create_dir_all(repo.join("docs/harness")).expect("harness dir");
        std::fs::write(
            repo.join("docs/harness/labs.json"),
            r#"{"schema": 1, "labs": {"receiptprobe": {"vendor": "v", "client": "c", "headless": "h", "open_pr_limit": 3}}}"#,
        )
        .expect("labs.json");
        std::fs::create_dir_all(&live).expect("live");
        Scratch {
            _dir: dir,
            repo,
            live,
        }
    }

    fn repo_arg(&self) -> String {
        self.repo.to_str().expect("utf8").to_string()
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(bin())
            .args(args)
            .env("UOR_HARNESS_LIVE", &self.live)
            .env("UOR_LAB", "receiptprobe")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("run uor-harness")
    }
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn an_unknown_kind_is_refused_and_nothing_is_written() {
    let s = Scratch::new();
    let repo = s.repo_arg();
    let out = s.run(&[
        "receipt",
        "--kind",
        "nonsense_kind",
        "--note",
        "probe",
        "--repo",
        &repo,
    ]);
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("unknown receipt kind"), "{err}");
    assert!(
        err.contains("heartbeat"),
        "the frozen list must be named: {err}"
    );
    assert!(
        !s.live.join("ledger").exists(),
        "a refused kind must not create a shard: {} stays empty",
        s.live.join("ledger").display()
    );
}

#[test]
fn the_short_milestone_form_is_recorded_like_the_long_one() {
    let s = Scratch::new();
    let repo = s.repo_arg();
    let short = s.run(&[
        "receipt",
        "--kind",
        "heartbeat",
        "-m",
        "m1",
        "--dry-run",
        "--repo",
        &repo,
    ]);
    assert!(short.status.success(), "{}", stderr(&short));
    let short_line = stdout(&short);
    assert!(
        short_line.contains("\"milestone\": \"m1\""),
        "-m must carry the milestone: {short_line}"
    );
    let long = s.run(&[
        "receipt",
        "--kind",
        "heartbeat",
        "--milestone",
        "m1",
        "--dry-run",
        "--repo",
        &repo,
    ]);
    assert_eq!(
        stdout(&long),
        short_line,
        "-m and --milestone must produce the same line"
    );
}

#[test]
fn a_flag_the_command_does_not_read_is_refused_loudly() {
    let s = Scratch::new();
    let repo = s.repo_arg();
    for flag in [
        ["--count", "1/3"],
        ["--headline", "37/232 -> 37/232"],
        ["--summary", "x"],
    ] {
        let out = s.run(&[
            "receipt",
            "--kind",
            "heartbeat",
            flag[0],
            flag[1],
            "--repo",
            &repo,
        ]);
        assert_eq!(
            out.status.code(),
            Some(2),
            "{flag:?} must be refused, not dropped: {} {}",
            stderr(&out),
            stdout(&out)
        );
        assert!(
            stderr(&out).contains(flag[0]),
            "{flag:?} must be named in the refusal: {}",
            stderr(&out)
        );
    }
    // And the flags it DOES read still work, so the guard cannot be satisfied by refusing everything.
    let ok = s.run(&[
        "receipt",
        "--kind",
        "skill",
        "--skill",
        "expert-dispatch",
        "--note",
        "n",
        "--dry-run",
        "--repo",
        &repo,
    ]);
    assert!(ok.status.success(), "{}", stderr(&ok));
    assert!(stdout(&ok).contains("expert-dispatch"), "{}", stdout(&ok));
}
