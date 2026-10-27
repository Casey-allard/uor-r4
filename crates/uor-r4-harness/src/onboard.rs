//! Onboarding: verify a lab can actually use the harness, and tell it what to do next.
//!
//! **Why this is Rust rather than a shell script or a page of prose.** D27 makes Rust the default
//! for new project code including tooling. More importantly, a checklist in a document cannot tell
//! you whether *your* session satisfies it. This runs the checks and reports facts about the
//! machine it is standing on: whether the `uor-harness` entry point is reachable and current,
//! whether this lab has an identity, and what the other labs are doing.
//!
//! **It checks rather than assumes, and it never mutates.** Everything here is read-only — no
//! receipt, no claim, no presence write, no commit. A lab runs it as many times as it likes.
//!
//! The three things that must be true, in the order they fail (the Python interpreter check went
//! when the Python mirror was archived; the harness is this binary alone):
//!
//! 1. **An identity.** Without `UOR_LAB`, receipts are recorded but attributed to nobody, and the
//!    other labs cannot see who did the work.
//! 2. **A readable shared record.** Claims and presence are the point of the plane; a lab that
//!    cannot read them is working blind.
//! 3. **The harness answers.** `check` is the contract every lab runs.

use std::path::{Path, PathBuf};

use crate::state::{read_ledger, StatePaths};

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

    // 0. is `cargo` the RUST cargo?
    //
    // **Antigravity found this on the owner's laptop:** `/opt/homebrew/bin/cargo` is a symlink to
    // `@cargo-ai/cli`, a Node.js CLI, so `cargo build` fails with `error: unknown command 'build'`
    // whenever Homebrew precedes `~/.cargo/bin` on PATH. The error names a command the tool does not
    // have, which points nowhere near the cause - the same shape as the stale-binary problem.
    //
    // Rust's cargo prints a version starting with `cargo `; the Node CLI does not.
    match cargo_is_rust() {
        Ok(path) => findings.push(Finding::ok(
            "cargo",
            format!("{path} is the Rust toolchain cargo"),
        )),
        Err(why) => findings.push(Finding::warn(
            "cargo",
            why,
            "put rustup's cargo first: export PATH=\"$HOME/.cargo/bin:$PATH\" (add it to your \
             shell profile). On this machine /opt/homebrew/bin/cargo is @cargo-ai/cli, a Node CLI, \
             not the Rust toolchain.",
        )),
    }

    // 1b. is the entry point reachable?
    //
    // `uor-harness` is the harness; there is no other implementation behind it. A lab that has not
    // built the binary, or whose shell cannot find it, has NO harness - and every documented command would fail with "command not found" rather than
    // a diagnosis. This is the check that turns that into an instruction.
    match which_uor_harness() {
        Some(path) => findings.push(Finding::ok(
            "entry point",
            format!("on PATH: {}", path.display()),
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
            "export UOR_LAB=<lab>  — once per session, before any other command. R9 in \
             docs/harness/pm-rules.md is the roster.",
        ));
    } else if let Err(why) = crate::labs::check_lab(repo, &lab) {
        // The registry (`docs/harness/labs.json`) is the one roster. A recording command refuses a
        // lab it does not list, so onboarding must not call that identity ready.
        findings.push(Finding::fail(
            "identity",
            format!("UOR_LAB={lab:?} cannot record: {why}"),
            "export UOR_LAB=<one of the registered labs above>; a new lab is an owner decision \
             recorded in docs/harness/labs.json",
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
    match harness_answers(repo) {
        Ok(line) => findings.push(Finding::ok("harness", line)),
        Err(line) => findings.push(Finding::fail(
            "harness",
            format!("the format check did not pass: {line}"),
            "run `uor-harness check` for the full list and the fix for each",
        )),
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

/// Which `cargo` the shell resolves, and whether it is the Rust toolchain's.
///
/// Runs `cargo --version` and requires the output to start with `cargo `, because **that is the only
/// test that distinguishes rustup's cargo from a same-named tool.** `/opt/homebrew/bin/cargo` on this
/// laptop is `@cargo-ai/cli`; `cargo build` against it fails with `unknown command 'build'`.
fn cargo_is_rust() -> Result<String, String> {
    let out = std::process::Command::new("cargo")
        .arg("--version")
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if text.starts_with("cargo ") {
                Ok(format!("cargo ({text})"))
            } else {
                Err(format!(
                    "`cargo` resolved to something that is not the Rust toolchain: it reports {text:?}"
                ))
            }
        }
        Ok(o) => Err(format!(
            "`cargo --version` failed: {}",
            String::from_utf8_lossy(&o.stderr).trim()
        )),
        Err(e) => Err(format!("no `cargo` on PATH: {e}")),
    }
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

    /// **Onboarding does not ask for Python**: the harness is the Rust binary alone, so a machine
    /// without `python3` must not fail it.
    #[test]
    fn onboarding_never_checks_for_a_python_interpreter() {
        let r = onboard(&repo(), Some("claude".to_string()));
        assert!(
            r.findings.iter().all(|f| f.name != "python"),
            "{:?}",
            r.findings
        );
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

    /// A lab the registry does not list FAILS onboarding and names the registered labs, matching the
    /// refusal a recording command gives: calling that identity ready would send the lab on to be
    /// refused.
    #[test]
    fn an_unknown_lab_fails_and_names_the_registered_labs() {
        let r = onboard(&repo(), Some("opencode".into()));
        let identity = r
            .findings
            .iter()
            .find(|f| f.name == "identity")
            .expect("identity");
        assert_eq!(identity.level, Level::Fail);
        for lab in ["antigravity", "claude", "deepseek", "kimi"] {
            assert!(identity.detail.contains(lab), "{lab}: {}", identity.detail);
        }
        // Codex left the loop (claude-138), so the roster in the refusal no longer names it.
        assert!(!identity.detail.contains("codex"), "{}", identity.detail);
        assert!(!r.ready(), "an unregistered lab is not ready to work");
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
