//! Record the commit this binary was built from, so a stale build can say so.
//!
//! **A stale binary cost a lab real time on 13 October.** Claude's machine had an old `uor-harness`
//! and it failed with `unknown option --line` — an error that describes the SYMPTOM (a flag the newer
//! source accepts) and points nowhere near the CAUSE (the binary predates the flag). The fix is not a
//! better error message for that one flag: it is for the binary to know which commit it came from and
//! to compare that against `origin/main` when asked.
//!
//! This runs at build time and only reads git; it never fails the build. A binary built outside a git
//! checkout gets an empty commit rather than an error, because not knowing is a state the warning
//! below can report honestly.

use std::process::Command;

fn main() {
    let commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    println!("cargo:rustc-env=UOR_BUILD_COMMIT={commit}");

    // Dirty means the binary does not correspond to any commit, which is worth saying separately:
    // "built from X" would be a lie about working-tree state.
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);
    println!(
        "cargo:rustc-env=UOR_BUILD_DIRTY={}",
        if dirty { "1" } else { "0" }
    );

    let built_at = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    println!("cargo:rustc-env=UOR_BUILD_AT={built_at}");

    // Rebuild when HEAD moves, so the recorded commit is never stale relative to the source.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
}
