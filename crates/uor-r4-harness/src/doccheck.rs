//! `harness.entry_point` — normative docs must not send a lab back to the Python entry point.
//!
//! # Why this is one of the two rules I had been calling unportable
//!
//! I wrote, for nine rounds, that this rule and `harness.documented_commands` are *"unportable by
//! nature — their subject is the file they live in."* **I had never opened `doccheck.py`.** The
//! subject is **the documentation**, which is an ordinary input; see #2028 (comment 6159759343).
//!
//! # What is ported here, and what is not
//!
//! **`stale_cli_invocations` is ported whole**, because it is a line scan over discovered docs and
//! there is nothing to weaken.
//!
//! **`harness.documented_commands` is NOT ported here, deliberately.** Python validates a documented
//! command by **building the live `argparse` parser and resolving the documented words against it** -
//! so it checks the command, its subcommand and its flags. A Rust version written without that
//! introspection could only check the **command word**, and **shipping that under the same rule id
//! would be a weaker check wearing the name of the real one** — the silent downgrade this port
//! refuses everywhere else. It stays in Python until someone ports the introspection.
//!
//! **So this module ports one rule and says why it is not two**, rather than half-porting a rule and
//! reporting coverage it does not have.

use crate::preprules::Finding;

/// Prefixes whose Markdown is never treated as instructions.
const DOC_EXCLUDE_PREFIXES: [&str; 7] = [
    "docs/history/",
    "research/",
    ".harness/",
    "local/",
    "target/",
    ".worktrees/",
    ".git/",
];

/// **Append-only logs that RECORD commands rather than instructing anyone to run them.**
///
/// `current-state.md` is the clearest case: entries there describe what changed, which necessarily
/// means naming what was run. **Treating them as instructions would make the rule fire on the
/// project's own history** — and a rule that fires on history is one people learn to ignore.
const DOC_EXCLUDE_FILES: [&str; 1] = ["docs/integration/current-state.md"];

const DOC_EXCLUDE_PARTS: [&str; 1] = ["/node_modules/"];

/// A dated record directory: `docs/labs/<topic>-<YYYY-MM-DD>/`.
///
/// **Excluded because they are evidence of what was run at the time, and rewriting them would
/// falsify the record.** That is a different reason from `current-state.md`'s, and both matter.
fn is_dated_record(rel: &str) -> bool {
    let Some(rest) = rel.strip_prefix("docs/labs/") else {
        return false;
    };
    let Some((topic, _)) = rest.split_once('/') else {
        return false;
    };
    // `<topic>-<YYYY-MM-DD>`: the date is the last 10 characters and shaped like a date.
    if topic.len() < 12 {
        return false;
    }
    let date = &topic[topic.len() - 10..];
    let b = date.as_bytes();
    b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..].iter().all(u8::is_ascii_digit)
}

/// `doccheck.discover_docs` — tracked Markdown that could give a lab a command.
///
/// **From `git ls-files`, so ignored build output and scratch state are excluded for free**, and the
/// `-z` form is used because paths can contain newlines. **AGENTS.md sorts first** because it is the
/// one file every lab reads, **so a wrong command there costs the most** — the order is Python's.
pub fn discover_docs(repo: &std::path::Path) -> Vec<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["ls-files", "-z", "*.md"])
        .output();
    let listing: Vec<String> = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        // **Outside a git checkout, walk instead** - the docstring below says so, and for one round
        // it did not, so Rust found nothing and reported success while Python would still scan.
        // **A rule that reports success because it looked at nothing is the failure this whole port
        // keeps finding**, and the fallback is what makes the two agree off-git.
        // **A git that ran but failed, or is absent, both mean the index is unusable** - and
        // walking is the fallback for both, because "the index could not be read" is one condition.
        _ => walk_markdown(repo),
    };
    let mut keep: Vec<String> = Vec::new();
    for rel in listing {
        if DOC_EXCLUDE_PREFIXES.iter().any(|p| rel.starts_with(p)) {
            continue;
        }
        if DOC_EXCLUDE_PARTS
            .iter()
            .any(|p| format!("/{rel}/").contains(p))
        {
            continue;
        }
        if is_dated_record(&rel) || DOC_EXCLUDE_FILES.contains(&rel.as_str()) {
            continue;
        }
        // **Only repo-root files and `docs/` are documentation.** A README beside source is not an
        // instruction set, and treating it as one would make the rule fire on files that merely
        // describe their own crate.
        if rel.contains('/') && !rel.starts_with("docs/") {
            continue;
        }
        keep.push(rel);
    }
    keep.sort();
    // AGENTS.md first: the one file every lab reads.
    keep.sort_by_key(|r| if r == "AGENTS.md" { 0 } else { 1 });
    keep
}

/// The walk fallback: every `*.md` under the repo, as repo-relative paths.
///
/// **Bounded and skip-aware**, because an unbounded walk over `.git` and `target` would be slow and
/// would find files the exclusions exist to remove. **`.git` is skipped here rather than filtered
/// later**, since walking it is the expensive part.
fn walk_markdown(repo: &std::path::Path) -> Vec<String> {
    fn go(dir: &std::path::Path, base: &std::path::Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if name == ".git" || name == "target" || name == "node_modules" {
                continue;
            }
            if path.is_dir() {
                go(&path, base, out);
            } else if name.ends_with(".md") {
                if let Ok(rel) = path.strip_prefix(base) {
                    out.push(rel.to_string_lossy().to_string());
                }
            }
        }
    }
    let mut out = Vec::new();
    go(repo, repo, &mut out);
    out
}

/// `doccheck.stale_cli_invocations` — normative docs that still invoke `python3 scripts/harness/cli.py`.
///
/// **`uor-harness` is the interface.** `cli.py` still exists as an implementation and the Rust entry
/// point delegates to it for unported commands — **but a doc telling a lab to run it puts Python back
/// in front of them, which is what the port was for.**
///
/// **A rule rather than a note, because "remember to use the new command" is exactly the kind of
/// instruction that decays.**
pub fn stale_cli_invocations(repo: &std::path::Path) -> Vec<(String, usize, String)> {
    let mut out = Vec::new();
    for rel in discover_docs(repo) {
        let path = repo.join(&rel);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            if line.contains("python3 scripts/harness/cli.py") {
                out.push((rel.clone(), i + 1, line.trim().chars().take(120).collect()));
            }
        }
    }
    out
}

/// `harness.entry_point` — the rule, with Python's message and severity.
///
/// **An `error` when it fires and a `warn` when it passes**, which is Python's own shape: the rule
/// reports a success line at `warn` severity with `ok=True` rather than staying silent. **The port
/// keeps the success line**, because `check`'s output is compared against Python's and a rule that
/// goes quiet changes a lab's reading of the run.
pub fn entry_point(repo: &std::path::Path) -> Vec<Finding> {
    let stale = stale_cli_invocations(repo);
    if stale.is_empty() {
        return vec![Finding {
            rule_id: "harness.entry_point",
            severity: "warn",
            message: "no normative doc invokes the Python entry point".into(),
            path: repo.display().to_string(),
            fix: String::new(),
        }];
    }
    let shown: Vec<String> = stale
        .iter()
        .take(4)
        .map(|(doc, line, _)| format!("{doc}:{line}"))
        .collect();
    vec![Finding {
        rule_id: "harness.entry_point",
        severity: "error",
        message: format!(
            "{} normative doc line(s) still invoke `python3 scripts/harness/cli.py`: {}",
            stale.len(),
            shown.join("; ")
        ),
        path: repo.display().to_string(),
        fix: "replace it with the `uor-harness` command — the Rust entry point is the interface"
            .into(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// A scratch git repo with the given Markdown, so **the exclusions are exercised rather than
    /// assumed** — the real repo has no stale invocation, so a test that only ran on it would never
    /// reach the failure branch.
    fn fixture(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        for (rel, body) in files {
            let p = dir.path().join(rel);
            std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
            std::fs::write(&p, body).expect("write");
        }
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "t@t"],
            vec!["config", "user.name", "t"],
            vec!["add", "-A"],
        ] {
            let _ = std::process::Command::new("git")
                .arg("-C")
                .arg(dir.path())
                .args(&args)
                .output();
        }
        dir
    }

    /// **Discovery on the real repo: the properties, not a magic number.**
    ///
    /// **This asserted `docs.len() == 927` and CI failed with 928.** The count is not a property of
    /// the code — **it changes whenever any lab adds a Markdown file**, and the repo gains them
    /// continuously. My local checkout was stale at 927 and CI checked out the merged head, so **the
    /// assertion passed here and failed there, which is the worst way for a test to be wrong.**
    ///
    /// **What is asserted instead is what the rule actually promises**: `AGENTS.md` first, the
    /// exclusions excluded, and every reported invocation genuinely present in a discovered doc.
    /// **A count that must be edited whenever a doc is added is not a test of discovery; it is a test
    /// of whether someone remembered to update it.**
    #[test]
    fn discovery_agrees_with_python_on_the_real_repo() {
        let docs = discover_docs(&repo());
        assert!(!docs.is_empty(), "the real repo yields documentation");
        assert_eq!(
            &docs[0], "AGENTS.md",
            "the one file every lab reads sorts first"
        );
        assert_eq!(&docs[1], "CONFORMANCE.md");
        assert_eq!(&docs[2], "CONTRIBUTING.md");
        // The exclusions, on real data rather than only in a fixture.
        assert!(
            !docs.iter().any(|d| d.starts_with("docs/history/")),
            "history is excluded"
        );
        assert!(
            !docs.iter().any(|d| d.contains("current-state.md")),
            "a log is not an instruction"
        );
        assert!(
            !docs
                .iter()
                .any(|d| d.starts_with("docs/labs/") && is_dated_record(d)),
            "dated records are evidence and are excluded"
        );
        assert!(
            !docs
                .iter()
                .any(|d| d.contains('/') && !d.starts_with("docs/")),
            "only root and docs/"
        );
        // Every reported invocation is in a doc that discovery returned - the rule's own contract,
        // rather than a number that changes when someone writes a record.
        let stale = stale_cli_invocations(&repo());
        for (doc, _, _) in &stale {
            assert!(docs.contains(doc), "{doc} was scanned but not discovered");
        }
    }

    /// **The exclusions, each for its own stated reason.** A dated record is evidence of what ran at
    /// the time; `current-state.md` records rather than instructs; `docs/history/` is retained
    /// history. **Excluding them is what stops the rule firing on the project's own record.**
    #[test]
    fn the_exclusions_are_by_reason_not_by_accident() {
        let dir = fixture(&[
            ("AGENTS.md", "see it"),
            (
                "docs/labs/real-topic-2026-10-14/README.md",
                "python3 scripts/harness/cli.py check",
            ),
            (
                "docs/integration/current-state.md",
                "python3 scripts/harness/cli.py status",
            ),
            (
                "docs/history/old-2026-01-01/README.md",
                "python3 scripts/harness/cli.py check",
            ),
            ("crates/x/README.md", "python3 scripts/harness/cli.py check"),
            ("docs/harness/README.md", "use `uor-harness check`"),
        ]);
        let docs = discover_docs(dir.path());
        assert!(docs.contains(&"AGENTS.md".to_string()));
        assert!(
            docs.contains(&"docs/harness/README.md".to_string()),
            "docs/ is documentation"
        );
        assert!(
            !docs.iter().any(|d| d.contains("real-topic-2026-10-14")),
            "a dated record is evidence"
        );
        assert!(
            !docs.iter().any(|d| d.contains("current-state.md")),
            "a log records, it does not instruct"
        );
        assert!(
            !docs.iter().any(|d| d.starts_with("docs/history/")),
            "retained history"
        );
        assert!(
            !docs.iter().any(|d| d.starts_with("crates/")),
            "a README beside source is not an instruction set"
        );
        // And the dated record being excluded means its Python invocation is NOT reported.
        assert!(
            stale_cli_invocations(dir.path()).is_empty(),
            "excluded docs are not scanned"
        );
    }

    /// **A normative doc that invokes Python IS reported**, with Python's message and severity — and
    /// this is the branch the real repo never reaches, so it is constructed.
    #[test]
    fn a_normative_python_invocation_is_an_error() {
        let dir = fixture(&[(
            "CONTRIBUTING.md",
            "Run `python3 scripts/harness/cli.py check` before opening a PR.\n",
        )]);
        let stale = stale_cli_invocations(dir.path());
        assert_eq!(stale.len(), 1, "{stale:?}");
        assert_eq!(stale[0].0, "CONTRIBUTING.md");
        assert_eq!(stale[0].1, 1, "the line number is 1-based");

        let f = entry_point(dir.path());
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule_id, "harness.entry_point");
        assert_eq!(
            f[0].severity, "error",
            "Python raises this at error, not warn"
        );
        assert!(
            f[0].message.contains("1 normative doc line(s)"),
            "{}",
            f[0].message
        );
        assert!(
            f[0].message.contains("CONTRIBUTING.md:1"),
            "{}",
            f[0].message
        );
    }

    /// **Off git, the walk finds the same docs the index would.** The docstring claimed this
    /// fallback for one round while the code returned an empty list — **so Rust found nothing and
    /// reported success while Python would still scan**, which is the "reports success because it
    /// looked at nothing" failure this port keeps finding.
    #[test]
    fn the_walk_fallback_finds_docs_outside_a_git_checkout() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("docs/harness")).expect("mkdir");
        std::fs::write(
            dir.path().join("CONTRIBUTING.md"),
            "python3 scripts/harness/cli.py check\n",
        )
        .expect("write");
        std::fs::write(
            dir.path().join("docs/harness/README.md"),
            "use `uor-harness check`\n",
        )
        .expect("write");
        // No `git init`: this is the off-git path.
        let docs = discover_docs(dir.path());
        assert!(
            docs.contains(&"CONTRIBUTING.md".to_string()),
            "the walk found it: {docs:?}"
        );
        assert!(
            docs.contains(&"docs/harness/README.md".to_string()),
            "{docs:?}"
        );
        // And the rule still fires off git, which is the point of having the fallback.
        let stale = stale_cli_invocations(dir.path());
        assert_eq!(
            stale.len(),
            1,
            "the invocation is found without an index: {stale:?}"
        );
        assert_eq!(entry_point(dir.path())[0].severity, "error");
    }

    /// **A clean repo produces the SUCCESS LINE, not silence.** Python reports
    /// `ok=True, severity="warn"` with a message rather than returning nothing, and **a rule that
    /// went quiet would change a lab's reading of the run.**
    #[test]
    fn a_clean_repo_reports_the_success_line_rather_than_nothing() {
        let dir = fixture(&[("AGENTS.md", "use `uor-harness check`")]);
        let f = entry_point(dir.path());
        assert_eq!(f.len(), 1, "the success line is a finding-shaped result");
        assert_eq!(f[0].severity, "warn");
        assert_eq!(
            f[0].message,
            "no normative doc invokes the Python entry point"
        );
        assert!(f[0].fix.is_empty(), "there is nothing to fix");
    }
}
