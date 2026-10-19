//! `docs-move`: move files and directories with `git mv`, driven by a map.
//!
//! `cargo xtask docs-move --map MAP.tsv [--apply]`
//!
//! The map has one `old<TAB>new` per line (a tracked file or a directory
//! holding tracked files); blank lines and `#` comments are ignored. Validation
//! always runs, with or without `--apply`, and reports every problem at once.
//! A move is a pure rename, so every byte is unchanged and git records it as
//! `R100`.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

/// One map entry, with surrounding slashes trimmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub old: String,
    pub new: String,
}

/// Parse the map text. Every malformed line is reported, not just the first.
pub fn parse_map(text: &str) -> Result<Vec<Entry>, Vec<String>> {
    let mut entries = Vec::new();
    let mut problems = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim_end_matches('\r');
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let mut parts = line.split('\t');
        let (old, new) = (parts.next().unwrap_or(""), parts.next());
        match (new, parts.next()) {
            (Some(new), None) => {
                let old = old.trim().trim_matches('/').to_string();
                let new = new.trim().trim_matches('/').to_string();
                if old.is_empty() || new.is_empty() {
                    problems.push(format!("line {}: empty path", i + 1));
                } else {
                    entries.push(Entry { old, new });
                }
            }
            _ => problems.push(format!(
                "line {}: expected exactly `old<TAB>new`: {line:?}",
                i + 1
            )),
        }
    }
    if problems.is_empty() {
        Ok(entries)
    } else {
        Err(problems)
    }
}

/// `a` equals `b` or is a path-component ancestor/descendant of it.
fn overlaps(a: &str, b: &str) -> bool {
    a == b || a.starts_with(&format!("{b}/")) || b.starts_with(&format!("{a}/"))
}

/// Number of tracked files at or under `old`.
fn count_tracked(tracked: &[String], old: &str) -> usize {
    let prefix = format!("{old}/");
    tracked
        .iter()
        .filter(|t| t.as_str() == old || t.starts_with(&prefix))
        .count()
}

/// Pure validation. `exists` says whether a path is present on disk.
pub fn validate(
    entries: &[Entry],
    tracked: &[String],
    exists: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let mut problems = Vec::new();
    let tracked_set: BTreeSet<&str> = tracked.iter().map(String::as_str).collect();
    for e in entries {
        if e.old == e.new {
            problems.push(format!("{}: old equals new", e.old));
        }
        if count_tracked(tracked, &e.old) == 0 {
            problems.push(format!("{}: not a tracked file or directory", e.old));
        }
        let new_prefix = format!("{}/", e.new);
        let new_tracked = tracked_set.contains(e.new.as_str())
            || tracked.iter().any(|t| t.starts_with(&new_prefix));
        if new_tracked || exists(&e.new) {
            problems.push(format!("{}: destination already exists", e.new));
        }
    }
    for e in entries {
        if e.old != e.new && overlaps(&e.new, &e.old) {
            problems.push(format!(
                "{}: destination {} is inside or above its own source",
                e.old, e.new
            ));
        }
    }
    for (i, a) in entries.iter().enumerate() {
        for b in &entries[i + 1..] {
            // One entry's destination inside another's source would be carried
            // along (or broken) when the later entry moves.
            if overlaps(&a.new, &b.old) || overlaps(&b.new, &a.old) {
                problems.push(format!(
                    "destination overlaps another source: {} -> {} and {} -> {}",
                    a.old, a.new, b.old, b.new
                ));
            }
            if overlaps(&a.old, &b.old) {
                problems.push(format!("overlapping sources: {} and {}", a.old, b.old));
            }
            if overlaps(&a.new, &b.new) {
                problems.push(format!("overlapping destinations: {} and {}", a.new, b.new));
            }
        }
    }
    problems
}

fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("git ls-files: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let mut files: Vec<String> = out
        .stdout
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();
    files.sort();
    Ok(files)
}

/// Entry point: `args` are the arguments after the task name.
pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let apply = args.iter().any(|a| a == "--apply");
    let map_path = args
        .iter()
        .position(|a| a == "--map")
        .and_then(|i| args.get(i + 1))
        .ok_or("usage: docs-move --map MAP.tsv [--apply]")?;
    let text = std::fs::read_to_string(map_path).map_err(|e| format!("{map_path}: {e}"))?;
    let entries = parse_map(&text).map_err(|p| p.join("\n"))?;
    let tracked = tracked_files(root)?;
    let problems = validate(&entries, &tracked, &|p| {
        root.join(p).symlink_metadata().is_ok()
    });
    if !problems.is_empty() {
        for p in &problems {
            eprintln!("problem: {p}");
        }
        return Err(format!("{} problem(s) in {map_path}", problems.len()));
    }

    let mut total = 0usize;
    let mut applied: Vec<&Entry> = Vec::new();
    for e in &entries {
        let n = count_tracked(&tracked, &e.old);
        if !apply {
            println!("would move {} -> {} ({n} files)", e.old, e.new);
            total += n;
            continue;
        }
        if let Some(parent) = Path::new(&e.new).parent() {
            std::fs::create_dir_all(root.join(parent))
                .map_err(|err| format!("create {}: {err}", parent.display()))?;
        }
        let out = Command::new("git")
            .args(["mv", "--", &e.old, &e.new])
            .current_dir(root)
            .output()
            .map_err(|err| format!("git mv: {err}"))?;
        if !out.status.success() {
            for done in &applied {
                eprintln!("already moved: {} -> {}", done.old, done.new);
            }
            eprintln!(
                "{} of {} entries were applied before the failure; undo with `git reset --hard` \
                 (discards all uncommitted changes) or `git mv` each listed path back",
                applied.len(),
                entries.len()
            );
            return Err(format!(
                "git mv {} {} failed: {}",
                e.old,
                e.new,
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        applied.push(e);
        total += n;
    }
    if apply {
        println!("moved {total} files");
    } else {
        println!("would move {total} files in {} entries", entries.len());
    }
    Ok(())
}

#[cfg(test)]
mod docs_move_tests {
    use super::*;

    fn tracked() -> Vec<String> {
        ["docs/a.md", "docs/sub/b.md", "handoff/x.md", "handoff/y.md"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn parse_skips_comments_and_blanks() {
        let e = parse_map("# c\n\nhandoff\tarchive/handoff\r\ndocs/a.md\tarchive/a.md\n").unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(
            e[0],
            Entry {
                old: "handoff".into(),
                new: "archive/handoff".into()
            }
        );
    }

    #[test]
    fn parse_rejects_space_separated_and_extra_columns() {
        assert!(parse_map("a b\n").is_err());
        assert!(parse_map("a\tb\tc\n").is_err());
    }

    #[test]
    fn validate_accepts_good_map() {
        let e = parse_map("handoff\tarchive/handoff\n").unwrap();
        assert!(validate(&e, &tracked(), &|_| false).is_empty());
        assert_eq!(count_tracked(&tracked(), "handoff"), 2);
    }

    #[test]
    fn validate_flags_old_equals_new_and_untracked() {
        let e = parse_map("docs\tdocs\nnope\tarchive/nope\n").unwrap();
        let p = validate(&e, &tracked(), &|_| false);
        assert!(p.iter().any(|m| m.contains("old equals new")));
        assert!(p.iter().any(|m| m.contains("nope: not a tracked")));
    }

    #[test]
    fn validate_flags_existing_destination() {
        let e = parse_map("handoff\tdocs\n").unwrap();
        assert!(validate(&e, &tracked(), &|_| false)
            .iter()
            .any(|m| m.contains("destination already exists")));
        let e = parse_map("handoff\tarchive/h\n").unwrap();
        assert!(validate(&e, &tracked(), &|p| p == "archive/h")
            .iter()
            .any(|m| m.contains("destination already exists")));
    }

    #[test]
    fn validate_flags_overlaps() {
        let e = parse_map("docs\tarchive/d\ndocs/sub\tarchive/s\n").unwrap();
        assert!(validate(&e, &tracked(), &|_| false)
            .iter()
            .any(|m| m.contains("overlapping sources")));
        let e = parse_map("docs\tarchive/d\nhandoff\tarchive/d/h\n").unwrap();
        assert!(validate(&e, &tracked(), &|_| false)
            .iter()
            .any(|m| m.contains("overlapping destinations")));
        let e = parse_map("handoff\tarchive/h\nhandoff\tarchive/h2\n").unwrap();
        assert!(!validate(&e, &tracked(), &|_| false).is_empty());
    }

    #[test]
    fn validate_flags_destination_inside_another_source() {
        // docs/sub lands under handoff, which a later entry moves away.
        let e = parse_map("docs/sub\thandoff/sub\nhandoff\tarchive/h\n").unwrap();
        assert!(validate(&e, &tracked(), &|_| false)
            .iter()
            .any(|m| m.contains("destination overlaps another source")));
        // A destination inside its own source.
        let e = parse_map("handoff\thandoff/x\n").unwrap();
        assert!(validate(&e, &tracked(), &|_| false)
            .iter()
            .any(|m| m.contains("inside or above its own source")));
    }

    #[test]
    fn overlap_is_per_component() {
        assert!(!overlaps("docs/a", "docs/ab"));
        assert!(overlaps("docs", "docs/ab"));
    }
}
