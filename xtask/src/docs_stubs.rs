//! Pointer stubs for the docs reorganization (`cargo xtask docs-stubs`).
//!
//! Archived records link to a few `archive/<p>` positions whose real file
//! stays live at `docs/<p>`. For each path listed in `--list FILE` this task
//! plans (dry run, the default) or writes (`--apply`) a stub: a markdown
//! pointer for a tracked markdown target, a byte copy for any other tracked
//! file, and `archive/<p>/README.md` for a tracked directory. Nothing is
//! staged in git.

use std::path::Path;

use crate::docs_links::{is_md, normalize, tracked_files, Universe};

/// What to do for one listed path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Write a markdown pointer at `dest` to `target`.
    Stub { dest: String, target: String },
    /// Copy the bytes of `target` to `dest`.
    Copy { dest: String, target: String },
    /// Write `dest` (`<dir>/README.md`) pointing at the directory `target`.
    DirStub { dest: String, target: String },
}

/// The stub text; `link` is the relative link from the stub to the target.
pub fn stub_text(target: &str, link: &str) -> String {
    format!(
        "# Moved\n\nThis page is live at [{target}]({link}).\n\nIt was left here as a pointer during the 2026-10 repository reorganization, so links inside archived records keep resolving.\n"
    )
}

/// Relative link from the directory of `from_file` to `target`.
pub fn link_from(from_file: &str, target: &str) -> String {
    let dir = from_file.rsplit_once('/').map_or("", |(d, _)| d);
    crate::docs_catalog::relative_link(dir, target)
}

/// Plan one listed path. `exists_file`/`exists_dir` describe the tracked
/// universe; `on_disk` reports whether a path already exists on disk.
pub fn plan(
    path: &str,
    is_file: &dyn Fn(&str) -> bool,
    is_dir: &dyn Fn(&str) -> bool,
    on_disk: &dyn Fn(&str) -> bool,
) -> Result<Action, String> {
    let p = path.trim_end_matches('/');
    let rest = p
        .strip_prefix("archive/")
        .filter(|r| !r.is_empty())
        .ok_or_else(|| format!("{path}: not under archive/"))?;
    let target = format!("docs/{rest}");
    if is_file(&target) {
        if is_file(p) || is_dir(p) || on_disk(p) {
            return Err(format!("{path}: stub path already exists"));
        }
        return Ok(if is_md(&target) {
            Action::Stub {
                dest: p.to_string(),
                target,
            }
        } else {
            Action::Copy {
                dest: p.to_string(),
                target,
            }
        });
    }
    if is_dir(&target) {
        let dest = format!("{p}/README.md");
        if is_file(&dest) || on_disk(&dest) {
            return Err(format!("{path}: stub path already exists ({dest})"));
        }
        return Ok(Action::DirStub { dest, target });
    }
    Err(format!("{path}: target {target} does not exist"))
}

fn label(a: &Action) -> (&'static str, &str, &str) {
    match a {
        Action::Stub { dest, target } => ("stub", dest, target),
        Action::Copy { dest, target } => ("copy", dest, target),
        Action::DirStub { dest, target } => ("dir-stub", dest, target),
    }
}

/// `docs-stubs --list FILE [--apply]`.
pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let mut list = None;
    let mut apply = false;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--list" => list = Some(it.next().ok_or("--list needs a value")?.clone()),
            "--apply" => apply = true,
            other => return Err(format!("docs-stubs: unknown argument {other}")),
        }
    }
    let list = list.ok_or("docs-stubs: --list FILE is required")?;
    let text = std::fs::read_to_string(&list).map_err(|e| format!("read {list}: {e}"))?;
    let uni = Universe::new(tracked_files(root)?);
    let files: std::collections::HashSet<String> = tracked_files(root)?.into_iter().collect();
    let is_file = |p: &str| files.contains(p);
    let is_dir = |p: &str| uni.exists(p) && !files.contains(p);
    let on_disk = |p: &str| root.join(p).exists();
    let mut actions = Vec::new();
    let mut problems = Vec::new();
    for raw in text.lines() {
        let l = raw.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let norm = match normalize(l) {
            Ok(n) => n,
            Err(e) => {
                problems.push(format!("{l}: {e}"));
                continue;
            }
        };
        match plan(&norm, &is_file, &is_dir, &on_disk) {
            Ok(a) => actions.push(a),
            Err(e) => problems.push(e),
        }
    }
    let (mut stubs, mut copies, mut dirs) = (0, 0, 0);
    for a in &actions {
        let (kind, dest, target) = label(a);
        match a {
            Action::Stub { .. } => stubs += 1,
            Action::Copy { .. } => copies += 1,
            Action::DirStub { .. } => dirs += 1,
        }
        println!("{kind} {dest} -> {target}");
        if apply {
            let out = root.join(dest);
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {dest}: {e}"))?;
            }
            match a {
                Action::Copy { .. } => {
                    std::fs::copy(root.join(target), &out)
                        .map_err(|e| format!("copy {dest}: {e}"))?;
                }
                _ => std::fs::write(&out, stub_text(target, &link_from(dest, target)))
                    .map_err(|e| format!("write {dest}: {e}"))?,
            }
        }
    }
    for p in &problems {
        println!("problem {p}");
    }
    println!(
        "total: {} actions ({stubs} stub, {copies} copy, {dirs} dir-stub), {} problems{}",
        actions.len(),
        problems.len(),
        if apply { " (applied)" } else { " (dry run)" }
    );
    if apply && !problems.is_empty() {
        return Err(format!("docs-stubs: {} problems", problems.len()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tracked(p: &str) -> bool {
        [
            "docs/a/b.md",
            "docs/a/pic.svg",
            "docs/d/x.md",
            "archive/taken.md",
            "docs/taken.md",
        ]
        .contains(&p)
    }
    fn dir(p: &str) -> bool {
        p == "docs/d" || p == "docs/a"
    }
    fn disk(_: &str) -> bool {
        false
    }

    #[test]
    fn markdown_file_gets_stub_with_relative_link() {
        let a = plan("archive/a/b.md", &tracked, &dir, &disk).unwrap();
        assert_eq!(
            a,
            Action::Stub {
                dest: "archive/a/b.md".into(),
                target: "docs/a/b.md".into()
            }
        );
        assert_eq!(
            link_from("archive/a/b.md", "docs/a/b.md"),
            "../../docs/a/b.md"
        );
        assert_eq!(
            stub_text("docs/a/b.md", "../../docs/a/b.md"),
            "# Moved\n\nThis page is live at [docs/a/b.md](../../docs/a/b.md).\n\nIt was left here as a pointer during the 2026-10 repository reorganization, so links inside archived records keep resolving.\n"
        );
    }

    #[test]
    fn non_markdown_is_copied() {
        let a = plan("archive/a/pic.svg", &tracked, &dir, &disk).unwrap();
        assert!(matches!(a, Action::Copy { .. }));
    }

    #[test]
    fn directory_gets_readme_stub_linking_to_directory() {
        let a = plan("archive/d", &tracked, &dir, &disk).unwrap();
        assert_eq!(
            a,
            Action::DirStub {
                dest: "archive/d/README.md".into(),
                target: "docs/d".into()
            }
        );
        assert_eq!(link_from("archive/d/README.md", "docs/d"), "../../docs/d");
    }

    #[test]
    fn refuses_outside_archive_existing_and_missing() {
        assert!(plan("docs/a/b.md", &tracked, &dir, &disk)
            .unwrap_err()
            .contains("not under archive/"));
        assert!(plan("archive/taken.md", &tracked, &dir, &disk)
            .unwrap_err()
            .contains("already exists"));
        assert!(plan("archive/a/b.md", &tracked, &dir, &|_| true)
            .unwrap_err()
            .contains("already exists"));
        assert!(plan("archive/nope.md", &tracked, &dir, &disk)
            .unwrap_err()
            .contains("does not exist"));
    }
}
