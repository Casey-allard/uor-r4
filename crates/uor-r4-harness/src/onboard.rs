//! Onboarding: verify a lab can actually use the harness, and tell it what to do next.
//!
//! **Why this is Rust rather than a shell script or a page of prose.** D27 makes Rust the default
//! for new project code including tooling. More importantly, a checklist in a document cannot tell
//! you whether *your* session satisfies it. This runs the checks and reports facts about the
//! machine it is standing on: the interpreter it finds, whether the harness responds with it,
//! whether this lab has an identity, and what the other labs are doing.
//!
//! **It checks rather than assumes, and it never mutates.** Everything here is read-only — no
//! receipt, no claim, no presence write, no commit. A lab runs it as many times as it likes.
//!
//! The four things that must be true, in the order they fail:
//!
//! 1. **A Python interpreter that runs the harness.** Python owns behaviour in this port, so a lab
//!    with no working `python3` cannot use the harness yet no matter how good this binary is.
//! 2. **An identity.** Without `UOR_LAB`, receipts are recorded but attributed to nobody, and the
//!    other labs cannot see who did the work.
//! 3. **A readable shared record.** Claims and presence are the point of the plane; a lab that
//!    cannot read them is working blind.
//! 4. **The harness answers.** `check` is the contract every lab runs.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::state::{read_ledger, StatePaths};

/// The minimum interpreter the harness supports (declared floor: Python 3.9).
pub const MIN_PYTHON: (u32, u32) = (3, 9);

/// Labs that may hold an identity. Not a closed set — a new lab is a decision, not a code change —
/// but an unrecognised value is worth saying out loud so a typo does not quietly become its own
/// shard. Compared AFTER slugging, because the harness slugs a lab name into every path it touches,
/// so `DeepSeek` and `deepseek` are one lab — warning on case would be nagging about nothing.
pub const KNOWN_LABS: [&str; 4] = ["kimi", "claude", "codex", "deepseek"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level {
    Ok,
    Warn,
    Fail,
}

impl Level {
    pub fn mark(&self) -> &'static str {
        match self {
            Level::Ok => "ok  ",
            Level::Warn => "warn",
            Level::Fail => "FAIL",
        }
    }
}

/// One verified statement about this machine.
#[derive(Debug, Clone)]
pub struct Finding {
    pub name: &'static str,
    pub level: Level,
    /// What was found, stated as a fact with the evidence in it.
    pub detail: String,
    /// What to do about it, when there is something to do.
    pub fix: String,
}

impl Finding {
    fn ok(name: &'static str, detail: impl Into<String>) -> Self {
        Finding {
            name,
            level: Level::Ok,
            detail: detail.into(),
            fix: String::new(),
        }
    }
    fn warn(name: &'static str, detail: impl Into<String>, fix: impl Into<String>) -> Self {
        Finding {
            name,
            level: Level::Warn,
            detail: detail.into(),
            fix: fix.into(),
        }
    }
    fn fail(name: &'static str, detail: impl Into<String>, fix: impl Into<String>) -> Self {
        Finding {
            name,
            level: Level::Fail,
            detail: detail.into(),
            fix: fix.into(),
        }
    }
}

/// The onboarding report.
#[derive(Debug, Clone)]
pub struct Report {
    pub repo: PathBuf,
    pub live_tier: PathBuf,
    pub lab: String,
    pub findings: Vec<Finding>,
    pub next_steps: Vec<String>,
}

impl Report {
    /// Ready means nothing failed. A warning is information, not a blocker.
    pub fn ready(&self) -> bool {
        self.findings.iter().all(|f| f.level != Level::Fail)
    }

    pub fn failures(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.level == Level::Fail)
            .count()
    }
}

/// Run `python3 -V` in the repository and return the version it reports.
///
/// A missing interpreter and an interpreter that refuses to run are the same problem to a lab, so
/// both come back as `Err` with the reason.
pub fn python_version(repo: &Path) -> Result<(u32, u32), String> {
    let out = Command::new("python3")
        .arg("-V")
        .current_dir(repo)
        .output()
        .map_err(|e| format!("python3 not runnable: {e}"))?;
    // Python prints its version to stdout at 3.4+, but older builds used stderr, so read both.
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    parse_python_version(&text).ok_or_else(|| format!("could not read a version from {text:?}"))
}

/// Parse `Python 3.9.6` (or any string containing `X.Y`) into a version pair.
pub fn parse_python_version(text: &str) -> Option<(u32, u32)> {
    let digits = text.split_whitespace().find_map(|word| {
        let core = word.trim_start_matches("Python").trim();
        let mut parts = core.split('.');
        let major: u32 = parts.next()?.parse().ok()?;
        let minor: u32 = parts.next()?.parse().ok()?;
        Some((major, minor))
    })?;
    Some(digits)
}

/// Whether the harness answers IN THIS REPOSITORY - checked in-process, not by shelling out.
///
/// This used to run `python3 scripts/harness/cli.py check --only arena` and read its output. Two
/// things were wrong with that: it asked the PYTHON harness whether the harness works, so a broken
/// Rust binary still reported ok; and it tested one trivial rule, so a repository whose real state
/// was broken passed the check that exists to catch that.
///
/// The entry point is this binary, so the honest question is "does the format checker accept this
/// repository", and it can be asked directly.
pub fn harness_answers(repo: &Path) -> Result<String, String> {
    let contract = crate::contract::Contract::load(repo)?;
    let report = contract.check(repo);
    let bad = report.total_violations();
    if bad == 0 {
        Ok(format!(
            "state formats ok ({} format(s))",
            report.formats.len()
        ))
    } else {
        Err(format!("{bad} format violation(s)"))
    }
}

/// Verify that onboarded means something, on the machine this runs on.
pub fn onboard(repo: &Path, lab_env: Option<String>) -> Report {
    let paths = StatePaths::from_env(repo);
    let mut findings = Vec::new();

    // 1. the interpreter
    let python_ok = match python_version(repo) {
        Ok(version) if version >= MIN_PYTHON => {
            findings.push(Finding::ok(
                "python",
                format!(
                    "{}.{} found (floor {}.{})",
                    version.0, version.1, MIN_PYTHON.0, MIN_PYTHON.1
                ),
            ));
            true
        }
        Ok(version) => {
            findings.push(Finding::fail(
                "python",
                format!(
                    "{}.{} is below the {}.{} floor; the harness will not run",
                    version.0, version.1, MIN_PYTHON.0, MIN_PYTHON.1
                ),
                "install a newer python3, or point PATH at one that is >= 3.9",
            ));
            false
        }
        Err(reason) => {
            findings.push(Finding::fail(
                "python",
                reason,
                "the harness runs on Python; without a working python3 there is nothing to onboard onto",
            ));
            false
        }
    };

    // 1b. is the entry point reachable?
    //
    // `uor-harness` is the harness's interface now; `python3 scripts/harness/cli.py` is an
    // implementation behind it. A lab that has not built the binary, or whose shell cannot find it,
    // has NO harness - and every documented command would fail with "command not found" rather than
    // a diagnosis. This is the check that turns that into an instruction.
    match which_uor_harness() {
        Some(path) => findings.push(Finding::ok(
            "entry point",
            format!("uor-harness is on PATH ({})", path.display()),
        )),
        None => findings.push(Finding::warn(
            "entry point",
            "uor-harness is NOT on PATH, so every documented `uor-harness <cmd>` fails here",
            "build and expose it: CARGO_TARGET_DIR=local/harness-target cargo build --release \
             -p uor-r4-harness, then export PATH=\"$PWD/local/harness-target/release:$PATH\"",
        )),
    }

    // 1c. is this binary the current one?
    //
    // A stale build fails with "unknown option <flag>" for flags newer source accepts, which points
    // at the symptom and nowhere near the cause. Only a PROVEN-behind build warns; no network, a
    // shallow clone or an unmerged branch are all normal and say nothing.
    if let Some(line) = crate::version::stale(repo).line() {
        findings.push(Finding::warn(
            "build",
            line,
            "rebuild: CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness",
        ));
    }

    // 2. the identity
    let lab = lab_env.unwrap_or_default();
    let slug = crate::state::slug(&lab);
    if lab.trim().is_empty() {
        findings.push(Finding::fail(
            "identity",
            "UOR_LAB is not set, so receipts are recorded but attributed to NOBODY",
            "export UOR_LAB=<kimi|claude|codex|deepseek>  — once per session, before any other command",
        ));
    } else if !KNOWN_LABS.contains(&slug.as_str()) {
        findings.push(Finding::warn(
            "identity",
            format!("UOR_LAB={lab:?} is not one of {}", KNOWN_LABS.join(", ")),
            "a new lab is a decision, not an error — but make sure this spelling is intentional, \
             because a different spelling becomes a second lab in the ledger",
        ));
    } else {
        findings.push(Finding::ok("identity", format!("UOR_LAB={slug}")));
    }

    // 3. the shared record
    let claims_path = paths.claims_file();
    match std::fs::read_to_string(&claims_path) {
        Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(serde_json::Value::Object(map)) => {
                let mine = map
                    .values()
                    .filter(|v| v.get("lab").and_then(|l| l.as_str()) == Some(slug.as_str()))
                    .count();
                findings.push(Finding::ok(
                    "claims",
                    format!("{} claim(s) visible, {mine} held by this lab", map.len()),
                ));
                if map.is_empty() {
                    findings.push(Finding::warn(
                        "claims",
                        "no branch is claimed by anyone right now",
                        "claim yours before starting: uor-harness team claim --branch <name> --lab <lab>",
                    ));
                }
            }
            Ok(_) => findings.push(Finding::fail(
                "claims",
                "the claims store is not a JSON object",
                "restore docs/harness/claims.json from git; the contract is in STATE-FORMATS.md",
            )),
            Err(e) => findings.push(Finding::fail(
                "claims",
                format!("the claims store is not valid JSON ({e})"),
                "restore docs/harness/claims.json from git",
            )),
        },
        Err(e) => findings.push(Finding::warn(
            "claims",
            format!(
                "no claims store readable at {} ({e})",
                claims_path.display()
            ),
            "an empty store is normal in a fresh checkout; it appears on the first claim",
        )),
    }

    let receipts = read_ledger(&paths);
    let foreign: Vec<String> = {
        let mut labs: Vec<String> = receipts
            .iter()
            .filter(|r| !r.lab.is_empty() && r.lab != slug && r.lab != "unknown")
            .map(|r| r.lab.clone())
            .collect();
        labs.sort();
        labs.dedup();
        labs
    };
    if receipts.is_empty() {
        findings.push(Finding::warn(
            "ledger",
            "the sharded ledger holds no receipts",
            "expected only in a fresh checkout; run a skill or a heartbeat and yours appears",
        ));
    } else if foreign.is_empty() && !slug.is_empty() {
        findings.push(Finding::warn(
            "ledger",
            format!("{} receipts, none from another lab", receipts.len()),
            "you may be the first lab in this checkout, or the shards may not have been fetched",
        ));
    } else {
        findings.push(Finding::ok(
            "ledger",
            format!(
                "{} receipts, other labs visible: {}",
                receipts.len(),
                foreign.join(", ")
            ),
        ));
    }

    // 4. the harness answers
    if python_ok {
        match harness_answers(repo) {
            Ok(line) => findings.push(Finding::ok("harness", line)),
            Err(line) => findings.push(Finding::fail(
                "harness",
                format!("the format check did not pass: {line}"),
                "run `uor-harness check` for the full list and the fix for each",
            )),
        }
    }

    let mut next_steps = Vec::new();
    if report_ready(&findings) {
        next_steps.push(
            "run one real task with the harness: `uor-harness brief -m <milestone>` then `uor-harness check`"
                .to_string(),
        );
        next_steps.push(
            "claim your branch before you start: `uor-harness team claim --branch <name> --lab <lab>`"
                .to_string(),
        );
        next_steps.push(
            // FIVE minutes, not ninety. The 90-minute figure was wrong for a whole port: R3 and the
            // live tier settled on five, and onboard kept telling every new lab the old number.
            "check in at least every five minutes: `uor-harness heartbeat -m <milestone>`, which also \
             publishes your live-tier presence"
                .to_string(),
        );
        next_steps.push(
            "after your merge, delete your branch on both sides — there is no auto-delete"
                .to_string(),
        );
    } else {
        next_steps
            .push("fix the FAIL lines above; warnings are information, not blockers".to_string());
    }

    Report {
        repo: repo.to_path_buf(),
        live_tier: paths.live.clone(),
        lab: slug,
        findings,
        next_steps,
    }
}

fn report_ready(findings: &[Finding]) -> bool {
    findings.iter().all(|f| f.level != Level::Fail)
}

/// Detect which runtime this is, so the report can name the hook that applies.
///
/// Best-effort and explicitly advisory: guessing a lab's identity from the environment would be
/// worse than a blank, so this only ever *suggests* which runtime-specific step to read.
pub fn detect_runtime() -> Option<&'static str> {
    if std::env::var_os("CLAUDE_CODE_ENTRYPOINT").is_some()
        || std::env::var_os("CLAUDE_CONFIG_DIR").is_some()
    {
        return Some("claude");
    }
    if let Ok(exe) = std::env::current_exe() {
        let path = exe.to_string_lossy().to_ascii_lowercase();
        if path.contains("dsh") {
            return Some("deepseek");
        }
        if path.contains("codex") || path.contains("chatgpt.app") {
            return Some("codex");
        }
    }
    if std::env::var_os("KIMI_HOME").is_some() {
        return Some("kimi");
    }
    None
}

/// The `uor-harness` the shell would run, if any.
///
/// Walks `PATH` rather than running `which`, because `which` is not guaranteed present and its
/// absence would be reported as "no entry point" — a false alarm on a machine that has one.
fn which_uor_harness() -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join("uor-harness");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn python_versions_parse_from_the_forms_python_prints() {
        assert_eq!(parse_python_version("Python 3.9.6"), Some((3, 9)));
        assert_eq!(parse_python_version("Python 3.15.0"), Some((3, 15)));
        assert_eq!(parse_python_version("  Python 3.11.7\n"), Some((3, 11)));
        assert_eq!(parse_python_version("nonsense"), None);
        assert_eq!(parse_python_version(""), None);
    }

    #[test]
    fn a_two_digit_minor_is_not_truncated() {
        // 3.15 is newer than 3.9; a naive string compare would call it older.
        let v = parse_python_version("Python 3.15.0").expect("parse");
        assert!(v >= MIN_PYTHON, "3.15 must satisfy the 3.9 floor");
    }

    #[test]
    fn onboarding_finds_a_real_interpreter_here() {
        let version = python_version(&repo()).expect("python3 present in this checkout");
        assert!(version >= MIN_PYTHON, "{version:?}");
    }

    #[test]
    fn onboarding_reports_a_missing_identity_as_a_failure() {
        let r = onboard(&repo(), Some(String::new()));
        assert!(!r.ready());
        let identity = r
            .findings
            .iter()
            .find(|f| f.name == "identity")
            .expect("identity finding");
        assert_eq!(identity.level, Level::Fail);
        assert!(identity.detail.contains("NOBODY"), "{}", identity.detail);
        assert!(
            identity.fix.contains("UOR_LAB"),
            "the fix must name the variable: {}",
            identity.fix
        );
    }

    #[test]
    fn onboarding_accepts_a_known_lab() {
        let r = onboard(&repo(), Some("deepseek".into()));
        let identity = r
            .findings
            .iter()
            .find(|f| f.name == "identity")
            .expect("identity finding");
        assert_eq!(identity.level, Level::Ok);
    }

    /// Case is NOT a problem, and this test exists to keep it that way.
    ///
    /// The Python harness slugs a lab name into every path it touches — `presence/<lab>.json`,
    /// `ledger/<lab>-<month>.jsonl`, the `lab` field of a receipt — so `DeepSeek` and `deepseek`
    /// are one lab, not two. An earlier version of this check warned on any name outside the known
    /// list and would have nagged about ordinary capitalisation.
    #[test]
    fn a_case_variant_is_the_same_lab_not_a_warning() {
        for name in ["DeepSeek", "deepseek", "DEEPSEEK", " deepseek "] {
            let r = onboard(&repo(), Some(name.into()));
            let identity = r
                .findings
                .iter()
                .find(|f| f.name == "identity")
                .expect("identity");
            assert_eq!(identity.level, Level::Ok, "{name:?}: {}", identity.detail);
            assert_eq!(r.lab, "deepseek", "{name:?} must slug to one lab");
        }
    }

    /// A genuinely unfamiliar lab warns rather than failing: joining is a decision, not an error.
    #[test]
    fn an_unknown_lab_warns_rather_than_failing() {
        let r = onboard(&repo(), Some("opencode".into()));
        let identity = r
            .findings
            .iter()
            .find(|f| f.name == "identity")
            .expect("identity");
        assert_eq!(identity.level, Level::Warn);
        assert!(r.ready(), "an unfamiliar lab is still ready to work");
    }

    /// The whole point: it must not write anything a lab would have to clean up.
    #[test]
    fn onboarding_is_read_only() {
        let receipts_before = read_ledger(&StatePaths::from_env(repo())).len();
        let _ = onboard(&repo(), Some("deepseek".into()));
        let receipts_after = read_ledger(&StatePaths::from_env(repo())).len();
        assert_eq!(
            receipts_before, receipts_after,
            "onboarding must not record a receipt"
        );
    }

    #[test]
    fn onboarding_sees_the_other_labs() {
        let r = onboard(&repo(), Some("deepseek".into()));
        let ledger = r
            .findings
            .iter()
            .find(|f| f.name == "ledger")
            .expect("ledger finding");
        // This checkout has receipts from kimi and codex; the finding should name them.
        assert!(
            ledger.detail.contains("kimi") || ledger.detail.contains("other labs visible"),
            "{}",
            ledger.detail
        );
    }

    #[test]
    fn onboarding_always_gives_a_next_step() {
        let r = onboard(&repo(), Some("deepseek".into()));
        assert!(!r.next_steps.is_empty());
    }
}
