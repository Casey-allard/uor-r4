//! `loop migrate --check` — the notes-migration state verifier (loop PR 8, item
//! `infra-loop-pr08-migration`, #2913).
//!
//! After the migration, these invariants hold on every commit and this command proves them:
//! retired notes (`STATUS.md`, `docs/integration/NOW.md`) carry pointers to byte-identical
//! archive copies; AGENTS.md carries the lessons markers; every `docs/harness/mistakes/*.md`
//! lesson is keyed `id: L-NNN` equal to its file stem with `L-001.md` present; the DECISIONS
//! table after the charter carries the "Alternatives turned down" and "Cost to undo" columns;
//! `docs/start/FILEMAP.md` has one row per tracked directory two levels deep; the read-path
//! budget holds (AGENTS.md at most 80 lines, STATE/HANDOFF under 24,576 B each, the four-file
//! read path under 76,800 B); and every markdown link in the migrated views resolves.
//! Read-only: exits 0 on pass, 1 with each failure named, 2 on a usage error or a refused read.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::looprender::{TOP_TEN_END, TOP_TEN_START};

const USAGE: &str = "uor-harness loop migrate --check\n  Verify the notes-migration invariants (loop PR 8). Read-only.";

/// The retired notes: (stub path kept in place, byte-identical archive copy).
const RETIRED: [(&str, &str); 2] = [
    ("STATUS.md", "archive/STATUS.md"),
    (
        "docs/integration/NOW.md",
        "archive/integration/NOW-2026-10-22.md",
    ),
];

/// Directories never mapped: build outputs, VCS metadata and the labs' local tiers.
const SKIP_DIRS: [&str; 5] = ["target", "node_modules", "local", ".worktrees", "dist"];

pub fn cli(root: &Path, args: &[String]) -> u8 {
    match args.first().map(String::as_str) {
        Some("--check") | None => {}
        Some(other) => {
            eprintln!("uor-harness: loop migrate: bad argument {other:?}\n{USAGE}");
            return 2;
        }
    }
    let mut failures = Vec::new();
    retired_notes(root, &mut failures);
    agents_markers(root, &mut failures);
    lesson_keys(root, &mut failures);
    decisions_columns(root, &mut failures);
    filemap(root, &mut failures);
    read_path_budget(root, &mut failures);
    migrated_links(root, &mut failures);
    if failures.is_empty() {
        println!("loop migrate --check: PASS (all notes-migration invariants hold)");
        0
    } else {
        eprintln!("loop migrate --check: FAIL");
        for f in &failures {
            eprintln!("  - {f}");
        }
        1
    }
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let o = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    o.status
        .success()
        .then(|| String::from_utf8_lossy(&o.stdout).to_string())
}

fn read(root: &Path, path: &str) -> Option<String> {
    fs::read_to_string(root.join(path)).ok()
}

/// A retired note: the stub exists, names its archive copy, and the archive copy is
/// byte-identical to the note's last live content (the parent of the commit that added the
/// archive file).
fn retired_notes(root: &Path, failures: &mut Vec<String>) {
    for (stub, archived) in RETIRED {
        let Some(text) = read(root, stub) else {
            failures.push(format!("{stub}: retired-note pointer stub is missing"));
            continue;
        };
        if !text.contains(archived) {
            failures.push(format!("{stub}: the pointer does not name {archived}"));
        }
        if read(root, archived).is_none() {
            failures.push(format!("{archived}: the archive copy is missing"));
            continue;
        }
        let Some(added) = git(
            root,
            &[
                "log",
                "--diff-filter=A",
                "--format=%H",
                "-1",
                "--",
                archived,
            ],
        ) else {
            failures.push(format!("{archived}: no git history for the archive copy"));
            continue;
        };
        let added = added.trim();
        let last_live = git(root, &["show", &format!("{added}^:{stub}")]);
        match last_live {
            Some(live) if live == read(root, archived).unwrap_or_default() => {}
            Some(_) => failures.push(format!(
                "{archived}: not byte-identical to {stub}'s last live content (at {added}^)"
            )),
            None => failures.push(format!(
                "{archived}: cannot read {stub} at {added}^ (history check failed)"
            )),
        }
    }
}

/// AGENTS.md carries both lessons top-ten markers (the rendered block's home).
fn agents_markers(root: &Path, failures: &mut Vec<String>) {
    match read(root, "AGENTS.md") {
        Some(text) => {
            if !text.contains(TOP_TEN_START) || !text.contains(TOP_TEN_END) {
                failures
                    .push("AGENTS.md: the loop:lessons-top-ten markers are missing".to_string());
            }
        }
        None => failures.push("AGENTS.md is missing".to_string()),
    }
}

/// Every lesson file is keyed `id: L-NNN` equal to its file stem, and L-001 exists.
fn lesson_keys(root: &Path, failures: &mut Vec<String>) {
    let dir = root.join("docs/harness/mistakes");
    let Ok(entries) = fs::read_dir(&dir) else {
        failures.push("docs/harness/mistakes/ is missing".to_string());
        return;
    };
    let mut saw_l001 = false;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") || name == "README.md" {
            continue;
        }
        let stem = name.trim_end_matches(".md");
        let keyed = stem.len() == 5
            && stem.starts_with("L-")
            && stem[2..].bytes().all(|b| b.is_ascii_digit());
        if !keyed {
            failures.push(format!(
                "docs/harness/mistakes/{name}: the file is not keyed L-NNN"
            ));
            continue;
        }
        saw_l001 |= stem == "L-001";
        let id_line = read(root, &format!("docs/harness/mistakes/{name}")).and_then(|t| {
            t.lines()
                .find_map(|l| l.strip_prefix("id:").map(str::trim).map(str::to_string))
        });
        if id_line.as_deref() != Some(stem) {
            failures.push(format!(
                "docs/harness/mistakes/{name}: its `id:` is not `{stem}`"
            ));
        }
    }
    if !saw_l001 {
        failures.push("docs/harness/mistakes/L-001.md (records drifted) is missing".to_string());
    }
}

/// The decisions table after the charter carries the two columns for new rows.
fn decisions_columns(root: &Path, failures: &mut Vec<String>) {
    match read(root, "docs/integration/DECISIONS.md") {
        Some(text) => {
            let Some(at) = text.find("## Decisions after the charter") else {
                failures
                    .push("DECISIONS.md: no `## Decisions after the charter` section".to_string());
                return;
            };
            let rest = &text[at..];
            for col in ["Alternatives turned down", "Cost to undo"] {
                if !rest.contains(col) {
                    failures.push(format!(
                        "DECISIONS.md: the after-charter table lacks the `{col}` column"
                    ));
                }
            }
        }
        None => failures.push("docs/integration/DECISIONS.md is missing".to_string()),
    }
}

/// FILEMAP: one row per tracked directory two levels deep, and no row whose path is gone.
fn filemap(root: &Path, failures: &mut Vec<String>) {
    let Some(text) = read(root, "docs/start/FILEMAP.md") else {
        failures.push("docs/start/FILEMAP.md is missing".to_string());
        return;
    };
    let rows: BTreeSet<String> = text
        .lines()
        .filter_map(|l| {
            l.strip_prefix("| `")
                .and_then(|r| r.split('`').next())
                .map(|p| p.trim_end_matches('/').to_string())
        })
        .collect();
    let Some(tracked) = git(root, &["ls-files"]) else {
        failures.push("FILEMAP: cannot list tracked files (git ls-files failed)".to_string());
        return;
    };
    let mut dirs = BTreeSet::new();
    for path in tracked.lines() {
        let parts: Vec<&str> = path.split('/').collect();
        let mut acc = String::new();
        // Every component before the last is a real directory (the last is the file).
        for part in &parts[..parts.len().saturating_sub(1)] {
            if !acc.is_empty() {
                acc.push('/');
            }
            acc.push_str(part);
            let depth = acc.matches('/').count() + 1;
            if depth <= 2 && !SKIP_DIRS.contains(&acc.as_str()) {
                dirs.insert(acc.clone());
            }
        }
    }
    for dir in &dirs {
        if !rows.contains(dir) {
            failures.push(format!("FILEMAP: directory `{dir}` has no row"));
        }
    }
    for row in &rows {
        if !root.join(row).is_dir() {
            failures.push(format!("FILEMAP: row `{row}` names a path that is gone"));
        }
    }
}

/// The read-path budget (the loop page): AGENTS.md at most 80 lines, STATE and HANDOFF under
/// 24,576 B each, the four-file read path under 76,800 B, both views stamped.
fn read_path_budget(root: &Path, failures: &mut Vec<String>) {
    let agents = read(root, "AGENTS.md").unwrap_or_default();
    let lines = agents.lines().count();
    if lines > 80 {
        failures.push(format!("AGENTS.md is {lines} lines, over the 80-line gate"));
    }
    let mut total = agents.len() + read(root, "CHARTER.md").unwrap_or_default().len();
    for view in ["STATE.md", "HANDOFF.md"] {
        match read(root, view) {
            Some(text) => {
                if text.len() >= 24_576 {
                    failures.push(format!(
                        "{view} is {} B, over the 24,576 B budget",
                        text.len()
                    ));
                }
                total += text.len();
            }
            None => failures.push(format!("{view} is missing")),
        }
    }
    if total >= 76_800 {
        failures.push(format!(
            "the read path is {total} B, over the 76,800 B line"
        ));
    }
}

/// Every markdown link in the migrated views resolves to a tracked path (the loop page's
/// `docs/`-relative links included). This is the migration's link check: the planted fault
/// (a broken link planted in a view) turns it red.
fn migrated_links(root: &Path, failures: &mut Vec<String>) {
    const VIEWS: [&str; 8] = [
        "PLAN.md",
        "STATE.md",
        "HANDOFF.md",
        "AGENTS.md",
        "STATUS.md",
        "docs/integration/NOW.md",
        "docs/start/FILEMAP.md",
        "docs/harness/pm-rules.md",
    ];
    for view in VIEWS {
        let Some(text) = read(root, view) else {
            failures.push(format!("{view}: the migrated view is missing"));
            continue;
        };
        let dir = Path::new(view)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        for (line_no, line) in text.lines().enumerate() {
            let mut rest = line;
            while let Some(start) = rest.find("](") {
                let after = &rest[start + 2..];
                let Some(end) = after.find(')') else { break };
                let target = after[..end]
                    .split('#')
                    .next()
                    .and_then(|t| t.split_whitespace().next())
                    .unwrap_or("");
                rest = &after[end..];
                if target.is_empty() || target.starts_with("http") || target.starts_with("mailto:")
                {
                    continue;
                }
                let resolved = dir.join(target);
                let mut norm = PathBuf::new();
                for part in resolved.components() {
                    match part {
                        std::path::Component::ParentDir => {
                            norm.pop();
                        }
                        std::path::Component::CurDir => {}
                        other => norm.push(other.as_os_str()),
                    }
                }
                if !root.join(&norm).exists() {
                    failures.push(format!("{view}:{}: broken link `{target}`", line_no + 1));
                }
            }
        }
    }
}

use std::path::PathBuf;
