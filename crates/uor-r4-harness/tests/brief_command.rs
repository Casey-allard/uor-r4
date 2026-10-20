//! `uor-harness brief` flags that change what a lab gets: `--json` and `--budget-tokens`.
//!
//! Read-only runs against this checkout's registry; nothing is written.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn brief(args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_uor-harness"));
    cmd.arg("--repo").arg(repo()).args(args);
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(&k);
        }
    }
    cmd.output().expect("run uor-harness")
}

/// **`--json` carries the pack's provenance and the same markdown** the plain command prints.
#[test]
fn json_carries_the_pack_and_its_markdown() {
    let out = brief(&["--json", "brief", "-m", "m1", "-q", "harness"]);
    assert_eq!(out.status.code(), Some(0));
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("brief --json prints one JSON object");
    for key in ["title", "tokens", "sources", "dropped", "markdown"] {
        assert!(v.get(key).is_some(), "missing {key}: {v}");
    }
    let plain = brief(&["brief", "-m", "m1", "-q", "harness"]);
    assert_eq!(
        v["markdown"].as_str().map(str::trim_end),
        Some(String::from_utf8_lossy(&plain.stdout).trim_end())
    );
    assert!(v["sources"]
        .as_array()
        .is_some_and(|s| !s.is_empty() && s[0].get("path").is_some()));
}

/// **A budget that does not parse is a usage error**, not a silent fall-back to the default.
#[test]
fn an_unparsable_budget_exits_2() {
    let out = brief(&["brief", "-m", "m1", "--budget-tokens", "abc"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--budget-tokens"));
}
