//! Record WHEN this binary was built. Nothing else belongs here.
//!
//! **The commit it was built from was baked in first, and that was the wrong design.** Cargo caches a
//! build script's output and re-runs it only when a declared input changes, so a fast-forward that
//! does not touch the source leaves the recorded commit stale. Two fixes were tried and MEASURED
//! against a real repository, and both failed:
//!
//! 1. `rerun-if-changed=../../.git/HEAD` - `.git/HEAD` holds `ref: refs/heads/main`, a ref NAME, so
//!    moving the branch never modifies it. And in a linked worktree that path does not exist at all,
//!    because the git dir is elsewhere.
//! 2. `rerun-if-changed=build.rs` - cargo compiles the script and caches its OUTPUT; re-running
//!    happens only when a declared input changes, and nothing declared had. Same staleness, one
//!    indirection deeper.
//!
//! Removing every directive uses cargo's default (re-run when the package changes), which still does
//! not cover a commit that touches nothing in this crate.
//!
//! So the commit is read AT RUNTIME, from the repository being inspected (`crate::version::stale`),
//! which is the only form that cannot go stale. What is left here is the build time: a fact about
//! this artefact that cannot change after it is written.
//!
//! Consequence, stated because it is a real loss: the check now reports whether **the checkout** is
//! behind `origin/main`, not whether **the binary** predates the checkout. A binary built from an
//! older commit by a newer checkout is no longer detected.

use std::process::Command;

fn main() {
    let built_at = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    println!("cargo:rustc-env=UOR_BUILD_AT={built_at}");
}
