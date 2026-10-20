//! The managed pre-commit hook against the real `uor-harness`, not a stand-in.
//!
//! The hook passes every commit when the check cannot produce a result, so a hook whose flags the
//! binary rejects (exit 2) would silently check nothing - the failure the v1 Python hook reached when
//! its entry point was archived. These tests run the shipped hook body with the built binary on
//! `PATH`, read-only, in this checkout.

use std::path::{Path, PathBuf};
use std::process::Command;

const HOOK: &str = include_str!("../rules/precommit-hook.sh");

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_uor-harness"))
}

/// A command with every `GIT_*` variable removed, so a test run from a git hook cannot be pointed
/// at another repository.
fn clean(mut cmd: Command) -> Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(&k);
        }
    }
    cmd
}

/// **The summary line the hook keys on is what `check --staged` prints**, and `--repo` after the
/// subcommand is accepted.
#[test]
fn check_staged_accepts_repo_and_prints_the_summary_the_hook_reads() {
    let mut cmd = clean(Command::new(binary()));
    cmd.args(["check", "--staged", "--repo"]).arg(repo());
    let out = cmd.output().expect("run uor-harness");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{stdout}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let last = stdout.lines().last().unwrap_or("");
    assert!(
        last.contains(" rules, 0 failed ("),
        "the hook matches ' rules, 0 failed (' on the last line; got {last:?}"
    );
}

/// **The shipped hook runs the real check and stays quiet when nothing failed.**
#[test]
fn the_hook_with_the_real_binary_passes_quietly() {
    let hook = std::env::temp_dir().join(format!("uor-hook-{}.sh", std::process::id()));
    std::fs::write(&hook, HOOK).expect("write hook");
    let dir = binary().parent().expect("bin dir").to_path_buf();
    let mut cmd = clean(Command::new("/bin/sh"));
    cmd.arg(&hook)
        .current_dir(repo())
        .env("PATH", format!("{}:/usr/bin:/bin", dir.display()));
    let out = cmd.output().expect("run hook");
    let _ = std::fs::remove_file(&hook);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "",
        "a clean staged check prints nothing"
    );
}
