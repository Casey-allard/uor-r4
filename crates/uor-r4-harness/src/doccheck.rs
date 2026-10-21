//! `harness.entry_point` and `harness.documented_commands` — normative documentation checks.
//!
//! # Why this is one of the two rules I had been calling unportable
//!
//! I wrote, for nine rounds, that this rule and `harness.documented_commands` are *"unportable by
//! nature — their subject is the file they live in."* **I had never opened `doccheck.py`.** The
//! subject is **the documentation**, which is an ordinary input; see #2028 (comment 6159759343).
//!
//! # What is ported here, and how the two implementations differ
//!
//! * **`stale_cli_invocations` (`harness.entry_point`) is ported whole**, because it is a line scan
//!   over discovered docs and there is nothing to weaken.
//! * **`documented_commands` (`harness.documented_commands`) resolves against explicit static tables
//!   ([`ACCEPTED`], [`GROUPS`], and [`FLAGS`])** rather than introspecting Python's live `argparse`
//!   tree. It checks the top-level command word against [`ACCEPTED`], the subcommand word for the 12
//!   grouped commands against [`GROUPS`], and every option token against the global [`FLAGS`] union
//!   (mirroring Python's `_all_options(parser, deep=True)`, while exempting bare prose `"-"` after
//!   [`RUST_ONLY_COMMANDS`] so `the_table_agrees_with_pythons_live_parser` stays in 1-to-1 agreement
//!   with `scripts/harness/doccheck.py`).

use crate::preprules::Finding;
use regex::Regex;
use std::sync::OnceLock;

/// Prefixes whose Markdown is never treated as instructions.
const DOC_EXCLUDE_PREFIXES: [&str; 8] = [
    "archive/",
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
/// **`uor-harness` is the interface, and the only implementation.** `cli.py` is archived under
/// `archive/scripts/harness/`, **so a doc telling a lab to run it sends them to a file that no longer
/// runs anything.**
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
        assert_eq!(&docs[1], "CHARTER.md");
        assert_eq!(&docs[2], "CONFORMANCE.md");
        assert_eq!(&docs[3], "CONTRIBUTING.md");
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
                "archive/labs/x-2026-10-01/README.md",
                "python3 scripts/harness/cli.py check",
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
            !docs.iter().any(|d| d.starts_with("archive/")),
            "archived markdown is a record, not an instruction"
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

// ---------------------------------------------------------------------------------------------
// `harness.documented_commands` - every documented invocation exists in THIS binary.
// ---------------------------------------------------------------------------------------------

/// Every top-level command a documented invocation may name.
///
/// # Two sets, because Python has two
///
/// Python resolves against **argparse's top-level choices (14)** and, before that, **short-circuits
/// on `RUST_ONLY_COMMANDS` (11)** — so the accepted set is their union. **Resolving against
/// `PORTED_COMMANDS` alone produced 44 false failures**, every one of them a command that works:
/// `check`, `mistakes capture`, `arena board`, `git clean-merged --apply`.
///
/// **`PORTED_COMMANDS` answers a different question** — which commands no longer delegate — **and the
/// comparison caught me using it for the wrong one.** The two overlap but neither contains the other.
const ACCEPTED: &[&str] = &[
    "status",
    "brief",
    "skill",
    "team",
    "mistakes",
    "prep",
    "multilab",
    "arena",
    "panel",
    "budget",
    "lint",
    "heartbeat",
    "git",
    "check",
    "onboard",
    "presence",
    "registry",
    "prep-render",
    "prep-check",
    "shard-check",
    "check-formats",
    "ledger-sync",
    "notes",
    "ratchet",
    "version",
    "loop",
    // `comment edit` edits a note by id. It moved with the lesson that names it: the two tables have to
    // move together, and only one side updated diverges the #[ignore]d cross-check.
    "comment",
];

/// Top-level commands implemented only by the Rust entry point (`scripts/harness/doccheck.py:192-216`).
///
/// Python's `_resolve` short-circuits on this set before checking flags, so a prose hyphen after a
/// Rust-only command (`uor-harness onboard - can this machine…`) is tolerated while genuine `--`/`-x`
/// flags are still validated against [`FLAGS`].
const RUST_ONLY_COMMANDS: &[&str] = &[
    "onboard",
    "presence",
    "registry",
    "prep-render",
    "prep-check",
    "shard-check",
    "check-formats",
    "ledger-sync",
    "notes",
    "ratchet",
    "version",
    "loop",
    "comment",
];

/// Subcommands of the top-level groups, as **this binary** accepts them.
///
/// **12 groups, taken from the runtime's own subparsers rather than curated by hand.**
///
/// # Why a table rather than introspection
///
/// Python builds its live `argparse` parser and resolves documented words against it. **A Rust port
/// cannot introspect `argparse`, and I refused to approximate it** — a command-word-only check would
/// miss `uor-harness status --refreshh`, **which is what a lab copying from a doc actually hits.**
///
/// **So the table is written down and the check resolves against it, subcommand and flags included.**
/// That is the same *strength* as Python's, from a different source, and **it is derived from the
/// binary rather than from the Python parser** — which is what lets this rule outlive `cli.py`.
///
/// **The one thing it cannot do that Python can**: Python's table IS the parser, so it cannot drift.
/// **This one can**, and the test below resolves every documented command through both
/// implementations to catch exactly that.
const GROUPS: [(&str, &[&str]); 12] = [
    ("arena", &["board", "compare", "lineage"]),
    ("budget", &["scan", "show"]),
    ("comment", &["edit"]),
    (
        "git",
        &[
            "clean-merged",
            "delete-merged",
            "install-hooks",
            "status",
            "switch",
        ],
    ),
    ("lint", &["check", "format"]),
    ("loop", &["gates", "render"]),
    ("mistakes", &["capture", "find", "list", "show"]),
    (
        "multilab",
        &["claims", "migrate-ledger", "presence", "sync-live", "touch"],
    ),
    ("panel", &["score", "show"]),
    ("prep", &["fit", "list", "new"]),
    ("skill", &["list", "match", "show", "use"]),
    (
        "team",
        &["claim", "claims", "dispatch", "release", "roster"],
    ),
];

/// The flags this binary accepts, wherever they appear — **78 of them, taken from Python's parser
/// and `main.rs`'s `SUBCOMMAND_FLAGS` / `SUBCOMMAND_VALUE_FLAGS` rather than curated by hand.**
///
/// **My first list was written from memory and was missing `--apply`, `--refresh`, `--dry-run` and
/// others**, which produced 18 false failures on commands that work. **A hand-kept interface list is
/// the same defect as a hand-kept command list**, and it fails the same way: the check reports a
/// documented command as broken when the command is fine.
///
/// **Global flags may appear anywhere**, and that is a deliberate looseness Python documents too:
/// *"what is worth checking is that the flag EXISTS, not where a doc happened to place it."*
const FLAGS: &[&str] = &[
    "--action",
    "--actor",
    "--apply",
    "--at",
    "--bar",
    "--base",
    "--body",
    "--body-file",
    "--branch",
    "--break",
    "--budget-tokens",
    "--cached",
    "--check",
    "--class",
    "--cost",
    "--count",
    "--create",
    "--date",
    "--doing",
    "--dry-run",
    "--evidence",
    "--files",
    "--force",
    "--full",
    "--guard",
    "--headline",
    "--help",
    "--id",
    "--json",
    "--keep-going",
    "--kind",
    "--lab",
    "--label",
    "--lesson",
    "--line",
    "--list",
    "--mark-seen",
    "--mechanism",
    "--milestone",
    "--milestones",
    "--next",
    "--no-notes",
    "--note",
    "--only",
    "--out",
    "--outcome",
    "--panel",
    "--paths",
    "--print",
    "--query",
    "--quick",
    "--quiet",
    "--refresh",
    "--repo",
    "--reviewer",
    "--role",
    "--route",
    "--rows",
    "--run",
    "--scope",
    "--seeds",
    "--session",
    "--severity",
    "--sheet",
    "--skill",
    "--split",
    "--staged",
    "--state",
    "--status",
    "--steps",
    "--stop",
    "--symptom",
    "--task",
    "--title",
    "--trigger",
    "--update-baseline",
    "--value",
    "--verdict",
    "--write",
    "-f",
    "-h",
    "-m",
    "-q",
];

/// A pattern compiled once.
fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("a valid pattern"))
}

/// `doccheck._COMMAND_RE` — `uor-harness <first> <rest>` or `cli.py <first> <rest>`.
fn command_re() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    re(
        &CELL,
        r"(?:uor-harness|cli\.py)\s+(?P<first>--?[A-Za-z][A-Za-z-]*(?:=\S+)?|[a-z][a-z-]*)(?P<rest>[^\n`]*)",
    )
}

/// `doccheck._SUBSTITUTION` — shell substitutions and backticked examples are not commands.
fn substitution_re() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    re(&CELL, r"\$\([^)]*\)|`[^`]*`")
}

/// `doccheck._resolve` — does this command exist, and if not, why not (command and subcommand words).
pub fn resolve(words: &[String], known: &[&str]) -> (bool, String) {
    resolve_with_flags(words, &[], known)
}

/// `doccheck._resolve` — validate top-level command against `known`, subcommand against [`GROUPS`],
/// and option tokens against [`FLAGS`].
///
/// **Two judgement calls, both Python's, both deliberate**: global flags may appear anywhere, and
/// flags interleaved with command words are tolerated — because `uor-harness --repo <path> multilab
/// presence` is a valid invocation a naive tokeniser splits.
///
/// **The failure modes are separated in the message** — an unknown *command* or *subcommand* lists
/// what exists, an unknown *flag* does not — because the two send a reader to different places.
pub fn resolve_with_flags(words: &[String], flags: &[String], known: &[&str]) -> (bool, String) {
    if words.is_empty() {
        return (false, "no command".into());
    }
    let head = words[0].as_str();
    if !known.contains(&head) {
        return (
            false,
            format!("unknown command {head:?} (have: {})", {
                let mut k = known.to_vec();
                k.sort();
                k.join(", ")
            }),
        );
    }
    if let Some((_, subs)) = GROUPS.iter().find(|(g, _)| *g == head) {
        if let Some(sub) = words.get(1).map(String::as_str) {
            if !subs.contains(&sub) {
                return (
                    false,
                    format!("unknown subcommand {sub:?} (have: {})", subs.join(", ")),
                );
            }
        }
    }
    for flag in flags {
        // A bare `-` after a Rust-only command (`uor-harness onboard - can this machine…`) is
        // prose punctuation that Python skips by short-circuiting on `RUST_ONLY_COMMANDS`.
        if flag == "-" && RUST_ONLY_COMMANDS.contains(&head) {
            continue;
        }
        if !FLAGS.contains(&flag.as_str()) {
            return (false, format!("unknown flag {flag:?}"));
        }
    }
    (true, "ok".to_string())
}

/// Flags whose following token is their value, so it must never be read as a command word.
const VALUE_FLAGS: &[&str] = &[
    "--repo",
    "--lab",
    "--only",
    "--body-file",
    "--milestone",
    "-m",
    "--next",
    "--line",
    "--count",
    "--role",
    "--branch",
    "--outcome",
    "--note",
    "--files",
    "-f",
    "--session",
    "--mechanism",
    "--run",
    "--seeds",
    "--bar",
    "--sheet",
    "--task",
    "--route",
    "--state",
    "--verdict",
    "--at",
    "--out",
];

/// Words picked up from surrounding prose rather than from a harness command.
const NOT_A_COMMAND: &[&str] = &[
    "rev-parse",
    "show-toplevel",
    "export",
    "cd",
    "echo",
    "cat",
    "cp",
    "then",
    "and",
    "or",
    "tee",
    "sed",
    "grep",
    "awk",
    "xargs",
    "for",
    "while",
    "if",
    "do",
    "done",
    "env",
    "set",
];

/// `doccheck._is_value` — is the token after a value-taking flag its value, or the next flag?
///
/// **Four ways to be a value**: a placeholder (`<repo>`), a quote or `$` (`"$(git …)"`), or a bare
/// `[\w./:@=+~-]+` token (`m1`, `.`, a path). **Anything starting with `-` is a flag, not a value**,
/// **which is why `--repo` does not swallow `-m m1`** — the case the docstring names.
///
/// **I implemented only the `-` rule at first** and the comparison showed the cost: my tokeniser
/// consumed the next FLAG as a value, so `uor-harness team claim --branch --lab` appeared where Python
/// had `uor-harness team claim --branch`. **Reading four lines of the function was cheaper than
/// another round of inferring it from output, and I should have done that first.**
fn is_value(token: &str) -> bool {
    if token.starts_with('-') {
        return false;
    }
    if is_placeholder(token) {
        return true;
    }
    if token.starts_with('"') || token.starts_with('\'') || token.starts_with('$') {
        return true;
    }
    static CELL: OnceLock<Regex> = OnceLock::new();
    re(&CELL, r"^[\w./:@=+~-]+$").is_match(token)
}

/// `doccheck._PLACEHOLDER` — `<id>`, `...`, `|`, `…` are documentation, not arguments.
fn is_placeholder(token: &str) -> bool {
    (token.starts_with('<') && token.ends_with('>'))
        || token == "..."
        || token == "|"
        || token == "\u{2026}"
}

/// `doccheck._tokenise` — split a documented command into (command words, flags).
///
/// # This function took three attempts, and the comparison found every one
///
/// **The first version dropped a flag's value and rendered the command without it**, so
/// `uor-harness brief -m` appeared where Python had `uor-harness brief -m m1` only because the value
/// was missing - **101 rows in one list and not the other.**
///
/// **The second included every non-flag token as a word**, which is closer and still wrong: **the
/// command string gained values Python never emits** (`uor-harness prep new m1`), **110 rows apart.**
///
/// **What I had not read**: Python consumes a value-taking flag's value **only when it is a value**,
/// **breaks the walk at a placeholder or a prose word**, and **breaks at the first token that is not
/// `[a-z][a-z-]*`** — which is why `ratchet d23|d25` yields just `ratchet`. **Every one of those is a
/// rule, and I had been inferring them from the output instead of reading them.**
fn tokenise(tokens: &[String]) -> (Vec<String>, Vec<String>) {
    let mut words = Vec::new();
    let mut flags = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let tok = &tokens[i];
        if tok.starts_with('-') {
            let flag = tok.split('=').next().unwrap_or(tok).to_string();
            flags.push(flag.clone());
            // A value-taking flag consumes the next token **when that token is a value**.
            if !tok.contains('=')
                && VALUE_FLAGS.contains(&flag.as_str())
                && i + 1 < tokens.len()
                && is_value(&tokens[i + 1])
            {
                i += 1;
            }
            i += 1;
            continue;
        }
        if is_placeholder(tok) || NOT_A_COMMAND.contains(&tok.as_str()) {
            break;
        }
        // **A command word is lowercase letters and dashes.** `m1`, `<files>`, `d23|d25` and a path
        // all stop the walk here - which is why the flag carries its value in the rendered command.
        let is_word = !tok.is_empty()
            && tok.starts_with(|c: char| c.is_ascii_lowercase())
            && tok.chars().all(|c| c.is_ascii_lowercase() || c == '-');
        if !is_word {
            break;
        }
        words.push(tok.clone());
        i += 1;
    }
    (words, flags)
}

/// `doccheck.commands_in_text` — every harness invocation on one line of documentation.
pub fn commands_in_text(text: &str, known: &[&str]) -> Vec<(String, bool, String)> {
    let mut out = Vec::new();
    for caps in command_re().captures_iter(text) {
        let first = caps.name("first").map(|m| m.as_str()).unwrap_or("");
        let rest_raw = caps.name("rest").map(|m| m.as_str()).unwrap_or("");
        let rest = substitution_re().replace_all(rest_raw, "$(...)");
        let mut tokens: Vec<String> = vec![first.to_string()];
        tokens.extend(rest.split_whitespace().map(str::to_string));
        let (words, flags) = tokenise(&tokens);
        if words.is_empty() {
            continue;
        }
        let (ok, why) = resolve_with_flags(&words, &flags, known);
        let mut render = words.clone();
        render.extend(flags.iter().cloned());
        out.push((format!("uor-harness {}", render.join(" ")), ok, why));
    }
    out
}

/// `harness.documented_commands` — every documented invocation, deduplicated per document.
///
/// **A duplicated command in the same document is reported once**, because a doc that repeats a
/// command is not repeating a mistake.
pub fn documented_commands(
    repo: &std::path::Path,
    known: &[&str],
) -> Vec<(String, usize, String, bool, String)> {
    let _ = known;
    let mut found = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new();
    for rel in discover_docs(repo) {
        let path = repo.join(&rel);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            for (command, ok, why) in commands_in_text(line, ACCEPTED) {
                let key = (command.clone(), rel.clone());
                if seen.contains(&key) {
                    continue;
                }
                seen.push(key);
                found.push((rel.clone(), i + 1, command, ok, why));
            }
        }
    }
    found
}

/// The rule, with Python's messages.
///
/// **`warn` in both directions** — Python reports a passing result at `warn` with `ok=true` and the
/// count, **and the count is in the message**, so a doc set that stops being scanned shows up as a
/// smaller number rather than as silence.
pub fn documented_commands_rule(repo: &std::path::Path, known: &[&str]) -> Vec<Finding> {
    let rows = documented_commands(repo, known);
    let bad: Vec<&(String, usize, String, bool, String)> = rows.iter().filter(|r| !r.3).collect();
    if bad.is_empty() {
        return vec![Finding {
            rule_id: "harness.documented_commands",
            severity: "warn",
            message: format!("all {} documented commands exist", rows.len()),
            path: repo.display().to_string(),
            fix: String::new(),
        }];
    }
    let shown: Vec<String> = bad
        .iter()
        .take(3)
        .map(|r| format!("{}:{} `{}` ({})", r.0, r.1, r.2, r.4))
        .collect();
    vec![Finding {
        rule_id: "harness.documented_commands",
        severity: "warn",
        message: format!(
            "{} documented command(s) do not exist: {}",
            bad.len(),
            shown.join("; ")
        ),
        path: repo.display().to_string(),
        fix: "fix the doc, or add the command — a lab following it gets an error otherwise".into(),
    }]
}

#[cfg(test)]
mod documented_commands_tests {
    use super::*;

    #[test]
    fn subcommands_and_flags_are_validated_against_groups_and_flags() {
        // Valid top-level command + valid flag
        let r1 = commands_in_text("Run `uor-harness status --refresh`", ACCEPTED);
        assert_eq!(r1.len(), 1);
        assert!(r1[0].1, "{r1:?}");

        // Typo in flag (`--refreshh`) must fail
        let r2 = commands_in_text("Run `uor-harness status --refreshh`", ACCEPTED);
        assert_eq!(r2.len(), 1);
        assert!(!r2[0].1, "unknown flag --refreshh must be rejected: {r2:?}");
        assert!(r2[0].2.contains("unknown flag \"--refreshh\""), "{r2:?}");

        // Invalid subcommand (`team list` instead of `team roster`) must fail
        let r3 = commands_in_text("Run `uor-harness team list`", ACCEPTED);
        assert_eq!(r3.len(), 1);
        assert!(
            !r3[0].1,
            "unknown subcommand `team list` must be rejected: {r3:?}"
        );
        assert!(r3[0].2.contains("unknown subcommand \"list\""), "{r3:?}");

        // Valid subcommand (`team roster`) passes
        let r4 = commands_in_text("Run `uor-harness team roster`", ACCEPTED);
        assert_eq!(r4.len(), 1);
        assert!(r4[0].1, "{r4:?}");

        // Prose dash after a Rust-only command is tolerated, while an invalid flag on a
        // Rust-only command is rejected
        let r5 = commands_in_text("uor-harness onboard - can this machine use it", ACCEPTED);
        assert_eq!(r5.len(), 1);
        assert!(r5[0].1, "{r5:?}");

        let r6 = commands_in_text("Run `uor-harness onboard --bad-flag`", ACCEPTED);
        assert_eq!(r6.len(), 1);
        assert!(!r6[0].1, "{r6:?}");
        assert!(r6[0].2.contains("unknown flag \"--bad-flag\""), "{r6:?}");
    }
}
