//! The hazard cannot be reintroduced into the tooling that runs on a schedule.
//!
//! On 14 October `gh issue comment --edit-last` overwrote the whole body of a comment another lab wrote, and
//! GitHub keeps no edit history, so the text was unrecoverable. The lesson is at
//! `docs/harness/mistakes/edit-last-overwrites-another-labs-comment.md` and the safe path is
//! `uor-harness comment edit --id <id> --body-file <f>`.
//!
//! **What this test reaches, and what it does not.** It scans the executable workflow roots whose changes this
//! job actually runs on: the harness's shell scripts (`crates/uor-r4-harness/rules/`, which replaced the
//! retired `scripts/harness/` Python) and `.github/workflows/**`. It **cannot intercept a hand-typed
//! `gh` call** - which is exactly why the safe path had to be BUILT rather than merely documented - and a
//! change to a script outside these roots will not run this test. Both limits are stated rather than implied,
//! because a guard that overstates its reach is worse than a narrow one that names it.

use std::path::{Path, PathBuf};

fn files_with_ext(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            files_with_ext(&p, exts, out);
        } else if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            if exts.iter().any(|x| name.ends_with(x)) {
                out.push(p);
            }
        }
    }
}

fn harness_root() -> PathBuf {
    // Resolved from the crate, not the working directory: `cargo test` runs in the package dir today, and
    // the reviewer caught `Path::new("../..")` depending on that.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("the workspace root is two levels above the crate")
        .to_path_buf()
}

/// Whether a line is a full-line comment (after leading whitespace).
/// Only full-line `#` or `//` comments are skipped so lessons can warn about the flag.
/// Inline comments are not parsed: `echo hi  # don't use --edit-last` still trips the guard.
fn is_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('#') || t.starts_with("//")
}

#[test]
fn no_executable_workflow_edits_a_comment_by_position() {
    let root = harness_root();
    let mut files = Vec::new();
    files_with_ext(
        &root.join("crates").join("uor-r4-harness").join("rules"),
        &[".sh", ".bash"],
        &mut files,
    );
    files_with_ext(
        &root.join(".github").join("workflows"),
        &[".yml", ".yaml"],
        &mut files,
    );
    assert!(
        !files.is_empty(),
        "found no scripts or workflows to scan - the test would pass vacuously"
    );

    let flag = "--edit-last";
    let mut offenders = Vec::new();
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            if line.contains(flag) && !is_comment(line) {
                offenders.push(format!("{}:{}: {}", f.display(), i + 1, line.trim()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "a script or workflow edits a comment by POSITION - that flag edits the last comment on the \
         issue, not the last one this lab wrote, and every lab here posts as one account. Edit by id \
         instead: uor-harness comment edit --id <id> --body-file <f>.\n{}",
        offenders.join("\n")
    );
}

/// The scan must not be vacuous in either root: if one is empty the guard silently stops covering it.
#[test]
fn the_scan_covers_both_roots() {
    let root = harness_root();
    let mut scripts = Vec::new();
    files_with_ext(
        &root.join("crates").join("uor-r4-harness").join("rules"),
        &[".sh", ".bash"],
        &mut scripts,
    );
    let mut workflows = Vec::new();
    files_with_ext(
        &root.join(".github").join("workflows"),
        &[".yml", ".yaml"],
        &mut workflows,
    );
    assert!(
        !scripts.is_empty(),
        "crates/uor-r4-harness/rules has no shell script to scan"
    );
    assert!(
        !workflows.is_empty(),
        ".github/workflows has nothing to scan"
    );
}
