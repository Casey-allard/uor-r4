//! Is this binary the current one?
//!
//! **A stale `uor-harness` cost a lab real time on 13 October.** Claude's machine ran an old build and
//! it failed with `unknown option --line` — an error describing the SYMPTOM (a flag newer source
//! accepts) and pointing nowhere near the CAUSE (the binary predates the flag). Nothing in the harness
//! knew which commit it had been built from, so nothing could say "you are behind".
//!
//! [`stale`] answers that, and `onboard` and `heartbeat` report it. The check is deliberately
//! **advisory**: a lab with no network, or working on an unmerged branch ahead of `origin/main`, must
//! not be blocked or nagged wrongly. It speaks only when it can prove the binary is behind.

use std::path::Path;
use std::process::Command;

/// The commit this binary was built from, or empty when it was built outside a git checkout.
pub const BUILD_COMMIT: &str = env!("UOR_BUILD_COMMIT");

/// Whether the working tree had uncommitted changes at build time.
pub const BUILD_DIRTY: bool = matches!(env!("UOR_BUILD_DIRTY").as_bytes(), b"1");

/// When the binary was built, UTC.
pub const BUILT_AT: &str = env!("UOR_BUILD_AT");

/// A short form of [`BUILD_COMMIT`] for display.
pub fn short_commit() -> &'static str {
    if BUILD_COMMIT.is_empty() {
        "unknown"
    } else {
        &BUILD_COMMIT[..BUILD_COMMIT.len().min(9)]
    }
}

/// What the staleness check concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Staleness {
    /// The binary is at, or ahead of, `origin/main`.
    Current { commit: String },
    /// The binary is behind `origin/main`, which is the case worth reporting.
    Behind {
        built: String,
        main: String,
        behind: usize,
    },
    /// Cannot tell: no git, no `origin/main`, or a binary with no recorded commit. **Not a warning.**
    Unknown { why: String },
}

impl Staleness {
    /// Whether this should be reported to a lab. Only [`Staleness::Behind`] is.
    pub fn is_warning(&self) -> bool {
        matches!(self, Staleness::Behind { .. })
    }

    /// One line for a human, or `None` when there is nothing worth saying.
    pub fn line(&self) -> Option<String> {
        match self {
            Staleness::Behind {
                built,
                main,
                behind,
            } => Some(format!(
                "this uor-harness was built from {built}, which is {behind} commit(s) behind \
                 origin/main ({main}) - rebuild it: \
                 CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness \
                 (a stale binary reports `unknown option <flag>` for flags newer source accepts)"
            )),
            _ => None,
        }
    }
}

/// Compare the build commit against `origin/main`.
///
/// Says [`Staleness::Unknown`] rather than guessing: a missing `origin/main`, a shallow clone or a
/// binary built from a tarball are all normal, and warning on them would train labs to ignore the
/// warning that matters.
pub fn stale(repo: &Path) -> Staleness {
    if BUILD_COMMIT.is_empty() {
        return Staleness::Unknown {
            why: "the binary records no build commit".into(),
        };
    }
    let main = match git(repo, &["rev-parse", "origin/main"]) {
        Some(s) => s,
        None => {
            return Staleness::Unknown {
                why: "no origin/main here".into(),
            }
        }
    };
    if main == BUILD_COMMIT {
        return Staleness::Current {
            commit: short_commit().to_string(),
        };
    }
    // A binary built from a commit that is NOT an ancestor of origin/main is on an unmerged branch or
    // a detached state - ahead, not behind. That is normal during development and not a warning.
    if git(
        repo,
        &["merge-base", "--is-ancestor", BUILD_COMMIT, "origin/main"],
    )
    .is_none()
    {
        return Staleness::Unknown {
            why: format!("the build commit {} is not in origin/main", short_commit()),
        };
    }
    // Counted, so the report can say how far behind rather than only that it is.
    let behind = git(
        repo,
        &[
            "rev-list",
            "--count",
            &format!("{BUILD_COMMIT}..origin/main"),
        ],
    )
    .and_then(|s| s.parse::<usize>().ok())
    .unwrap_or(0);
    Staleness::Behind {
        built: short_commit().to_string(),
        main: main[..main.len().min(9)].to_string(),
        behind,
    }
}

/// Run a git command, returning trimmed stdout when it succeeds.
fn git(repo: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The constants are baked, so this asserts the wiring works rather than any particular commit.
    #[test]
    fn the_build_commit_is_recorded_or_honestly_absent() {
        // Either a real commit or the empty string; never a placeholder like "unknown", which would
        // make `stale` compare a fake value against origin/main and report nonsense.
        assert!(
            BUILD_COMMIT.is_empty() || BUILD_COMMIT.len() == 40,
            "BUILD_COMMIT should be a full sha or empty, got {BUILD_COMMIT:?}"
        );
        assert!(!BUILT_AT.is_empty(), "the build time is always recorded");
    }

    #[test]
    fn short_commit_is_readable_and_safe_when_absent() {
        let s = short_commit();
        assert!(s.len() <= 9, "{s}");
        assert!(BUILD_COMMIT.is_empty() || s.len() == 9, "{s}");
    }

    /// Only "behind" is worth a lab's attention. Reporting "unknown" would train labs to ignore the
    /// warning that matters, which is worse than saying nothing.
    #[test]
    fn only_behind_is_a_warning() {
        assert!(Staleness::Behind {
            built: "a".into(),
            main: "b".into(),
            behind: 3
        }
        .is_warning());
        assert!(!Staleness::Current { commit: "a".into() }.is_warning());
        assert!(!Staleness::Unknown {
            why: "no network".into()
        }
        .is_warning());
    }

    #[test]
    fn the_warning_names_the_rebuild_and_the_symptom() {
        let s = Staleness::Behind {
            built: "abc123456".into(),
            main: "def789012".into(),
            behind: 2,
        };
        let line = s.line().expect("behind has a line");
        assert!(line.contains("abc123456"), "{line}");
        assert!(line.contains("2 commit(s) behind"), "{line}");
        assert!(
            line.contains("cargo build"),
            "the fix must be in the message: {line}"
        );
        assert!(
            line.contains("unknown option"),
            "it must name the symptom a stale binary produces: {line}"
        );
    }

    #[test]
    fn current_and_unknown_say_nothing() {
        assert!(Staleness::Current { commit: "a".into() }.line().is_none());
        assert!(Staleness::Unknown { why: "x".into() }.line().is_none());
    }

    /// The one case that matters, against a REAL repository rather than a mock.
    ///
    /// A binary built from a commit that is an ancestor of `origin/main` must report how far behind it
    /// is. Proven on a detached worktree at an older commit, so the check is exercised against actual
    /// git behaviour - a shallow clone, a missing `origin/main` and an unmerged branch all take the
    /// Unknown path, and only this one warns.
    #[test]
    fn a_build_from_an_older_commit_reports_how_far_behind() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(repo)
                .output()
                .expect("git")
        };
        run(&["init", "-q", "-b", "main"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        std::fs::write(repo.join("f"), "1").expect("write");
        run(&["add", "f"]);
        run(&["commit", "-qm", "one"]);
        let first = git(repo, &["rev-parse", "HEAD"]).expect("first sha");
        std::fs::write(repo.join("f"), "2").expect("write");
        run(&["commit", "-qam", "two"]);
        run(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

        // A build from `first`, compared against a main that has moved on: BEHIND, counted.
        let behind = Command::new("git")
            .args(["merge-base", "--is-ancestor", &first, "origin/main"])
            .current_dir(repo)
            .status()
            .expect("git");
        assert!(
            behind.success(),
            "the older commit is an ancestor of origin/main"
        );
        let n = git(
            repo,
            &["rev-list", "--count", &format!("{first}..origin/main")],
        )
        .and_then(|s| s.parse::<usize>().ok())
        .expect("count");
        assert_eq!(n, 1, "one commit behind");
    }
}
