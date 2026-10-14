//! Is this checkout behind, and when was this binary built?
//!
//! **Two designs were tried and both were wrong; the failures are recorded because the second looked
//! correct until it was measured.**
//!
//! The first baked the build commit into the binary via `build.rs` and compared it against
//! `origin/main`. `rerun-if-changed=../../.git/HEAD` was supposed to keep it fresh — but `.git/HEAD`
//! holds `ref: refs/heads/main`, a ref *name*, so **a fast-forward never modifies it**, and in a
//! linked worktree that path does not exist because the git dir is elsewhere.
//!
//! The second declared `rerun-if-changed=build.rs`. Cargo compiles a build script and **caches its
//! output**, re-running it only when a declared input changes — and nothing declared had. Same
//! staleness, one indirection deeper. Both designs were measured against a real repository with two
//! commits on `main`; the reported commit followed neither.
//!
//! So the commit is read **at runtime, from the repository being inspected**. That is the only form
//! that cannot go stale, because there is nothing cached to go stale.
//!
//! # The consequence, stated because it is a real loss
//!
//! This reports whether **the checkout** is behind `origin/main`, not whether **the binary** predates
//! the checkout. A binary built from an older commit, sitting in a newer checkout, is no longer
//! detected — which was the original case that motivated the warning. What remains detectable is the
//! common one: a lab that has not pulled or rebuilt, whose binary then rejects a flag the docs
//! describe.

use std::path::Path;
use std::process::Command;

/// When this binary was built, UTC. **The one baked fact, and it cannot go stale**, because the
/// binary does not change after it is written.
pub const BUILT_AT: &str = env!("UOR_BUILD_AT");

/// A short sha, for display.
pub fn short(sha: &str) -> &str {
    if sha.is_empty() {
        "unknown"
    } else {
        &sha[..sha.len().min(9)]
    }
}

/// What the staleness check concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Staleness {
    /// The checkout is at, or ahead of, `origin/main`.
    Current { commit: String },
    /// The checkout is behind, which is the case worth reporting.
    Behind {
        built: String,
        main: String,
        behind: usize,
    },
    /// Cannot tell: no git, no `origin/main`, or a detached state not in main. **Not a warning** — a
    /// shallow clone and an unmerged branch are both normal, and warning on them would train labs to
    /// ignore the warning that matters.
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
            Staleness::Behind { built, main, behind } => Some(format!(
                "this checkout is {behind} commit(s) behind origin/main (HEAD {built}, main {main}) - \
                 if your uor-harness rejects a flag the docs describe, rebuild it: \
                 CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness"
            )),
            _ => None,
        }
    }
}

/// Where the RUNNING binary came from. This is the fact the "rebuild" advice misses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Placement {
    /// Running this checkout's own build. The normal development case, and not worth a line.
    InCheckout,
    /// Running a binary from outside the checkout - typically the `~/.cargo/bin` install on PATH.
    Outside { exe: std::path::PathBuf },
    /// Cannot tell. **Not a warning**: a missing `current_exe` is not a lab's problem.
    Unknown,
}

/// Decide, from the path of the running executable, whether it is this checkout's build.
///
/// **The warning every lab hits says "rebuild", but a rebuild does not touch the installed copy they
/// are actually running.** So a lab rebuilds, keeps running `~/.cargo/bin/uor-harness`, and meets the
/// same warning next session - with no way to tell that the advice missed the binary in front of it.
/// This separates the two cases from the one fact that already distinguishes them.
pub fn placement(repo: &Path) -> Placement {
    let Ok(exe) = std::env::current_exe() else {
        return Placement::Unknown;
    };
    // Both sides canonicalised: `current_exe` resolves symlinks on some platforms and not others,
    // and the repo path may itself be reached through a symlink (`/tmp` on macOS).
    let exe = exe.canonicalize().unwrap_or(exe);
    let root = repo.canonicalize().unwrap_or_else(|_| repo.to_path_buf());
    if exe.starts_with(&root) {
        Placement::InCheckout
    } else {
        Placement::Outside { exe }
    }
}

/// The refresh instruction when this binary is NOT the checkout's build, or `None` when it is.
///
/// Names the path it is actually running, because "some binary somewhere is stale" is not actionable
/// and this is.
pub fn placement_line(repo: &Path) -> Option<String> {
    match placement(repo) {
        Placement::Outside { exe } => Some(format!(
            "this is {}, not this checkout's build, so rebuilding will NOT refresh it - install the \
             current one with: cargo install --path crates/uor-r4-harness --offline",
            exe.display()
        )),
        _ => None,
    }
}

/// Compare the checkout's HEAD against `origin/main`.
pub fn stale(repo: &Path) -> Staleness {
    let head = match git(repo, &["rev-parse", "HEAD"]) {
        Some(h) => h,
        None => {
            return Staleness::Unknown {
                why: "no git HEAD here".into(),
            }
        }
    };
    let main = match git(repo, &["rev-parse", "origin/main"]) {
        Some(s) => s,
        None => {
            return Staleness::Unknown {
                why: "no origin/main here".into(),
            }
        }
    };
    if main == head {
        return Staleness::Current {
            commit: short(&head).to_string(),
        };
    }
    // A checkout that is not an ancestor of origin/main is ahead, or on a branch. Normal during
    // development, and not a warning.
    if git(repo, &["merge-base", "--is-ancestor", &head, "origin/main"]).is_none() {
        return Staleness::Unknown {
            why: format!("HEAD {} is not in origin/main", short(&head)),
        };
    }
    // Counted, so the report can say how far behind rather than only that it is.
    let behind = git(
        repo,
        &["rev-list", "--count", &format!("{head}..origin/main")],
    )
    .and_then(|s| s.parse::<usize>().ok())
    .unwrap_or(0);
    Staleness::Behind {
        built: short(&head).to_string(),
        main: short(&main).to_string(),
        behind,
    }
}

/// The checkout's HEAD, for display. Empty when git cannot answer.
pub fn head(repo: &Path) -> String {
    git(repo, &["rev-parse", "HEAD"]).unwrap_or_default()
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

    /// The build time is the one baked fact, and it cannot go stale because the binary does not
    /// change after it is written.
    #[test]
    fn the_build_time_is_recorded() {
        assert!(!BUILT_AT.is_empty(), "the build time is always recorded");
    }

    #[test]
    fn short_is_readable_and_safe_when_absent() {
        assert_eq!(short(""), "unknown");
        assert_eq!(
            short("0123456789abcdef"),
            "012345678",
            "a short sha is nine characters"
        );
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
            line.contains("rejects a flag"),
            "it must name the symptom a stale binary produces: {line}"
        );
    }

    #[test]
    fn current_and_unknown_say_nothing() {
        assert!(Staleness::Current { commit: "a".into() }.line().is_none());
        assert!(Staleness::Unknown { why: "x".into() }.line().is_none());
    }

    /// Against a REAL repository, because that is where both wrong designs passed.
    ///
    /// Two commits on `main`, `origin/main` at the second: a checkout at the first is Behind by one,
    /// at the second is Current, and a branch ahead of main is Unknown rather than Behind. This is
    /// the assertion the baked-in design never had, and it is the one that would have failed.
    #[test]
    fn a_checkout_is_measured_against_origin_main() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        let run = |args: &[&str]| {
            Command::new("git")
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
        let first = git(repo, &["rev-parse", "HEAD"]).expect("first");
        std::fs::write(repo.join("f"), "2").expect("write");
        run(&["commit", "-qam", "two"]);
        let second = git(repo, &["rev-parse", "HEAD"]).expect("second");
        run(&["update-ref", "refs/remotes/origin/main", &second]);

        run(&["checkout", "-q", &first]);
        match stale(repo) {
            Staleness::Behind { behind, .. } => assert_eq!(behind, 1),
            other => panic!("expected Behind by one, got {other:?}"),
        }
        run(&["checkout", "-q", &second]);
        assert!(
            matches!(stale(repo), Staleness::Current { .. }),
            "at main is current"
        );

        // A branch ahead of main is Unknown, not Behind: normal development is not a warning.
        run(&["checkout", "-qb", "feature"]);
        std::fs::write(repo.join("f"), "3").expect("write");
        run(&["commit", "-qam", "three"]);
        assert!(
            matches!(stale(repo), Staleness::Unknown { .. }),
            "ahead is not behind"
        );
    }

    /// No `origin/main` at all is Unknown, not a failure: a fresh clone of a fork and a tarball
    /// checkout are both normal.
    #[test]
    fn no_origin_main_is_unknown() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        let run = |args: &[&str]| {
            Command::new("git")
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
        match stale(repo) {
            Staleness::Unknown { why } => assert!(why.contains("origin/main"), "{why}"),
            other => panic!("expected Unknown, got {other:?}"),
        }
    }

    /// A binary outside the checkout must be reported - this is the case the old advice missed.
    #[test]
    fn a_binary_outside_the_checkout_is_named() {
        let dir = tempfile::tempdir().expect("tempdir");
        match placement(dir.path()) {
            Placement::Outside { exe } => assert!(exe.is_absolute(), "{exe:?}"),
            other => panic!("a temp repo does not contain this test binary: {other:?}"),
        }
        let line = placement_line(dir.path()).expect("a line for an outside binary");
        assert!(
            line.contains("cargo install"),
            "it must say how to fix it: {line}"
        );
        assert!(
            line.contains("NOT refresh"),
            "it must say a rebuild does not help: {line}"
        );
    }

    /// The converse, and the one that keeps the line honest: a repo CONTAINING the running binary is
    /// the checkout's own build and must produce nothing. A false positive here would print on every
    /// command of every lab that built locally.
    #[test]
    fn the_checkouts_own_build_is_not_reported() {
        let exe = std::env::current_exe().expect("current_exe");
        let profile = exe.parent().expect("deps").parent().expect("profile dir");
        assert_eq!(placement(profile), Placement::InCheckout);
        assert!(
            placement_line(profile).is_none(),
            "the checkout's own build must produce no line"
        );
    }
}
