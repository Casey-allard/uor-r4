//! `uor-harness loop arrive [--json]`: loop step 2, run once at the start of a session (loop PR 7b).
//!
//! **Why this exists.** The loop page's step 2 reads: fetch; check the binary, hooks, notes and read
//! path; render the views locally. If the repository disagrees with a note, the repository wins.
//! Each of those is one check here, run in a fixed order and reported one line each, so a lab
//! arriving in a session sees in one place what is stale before it reads or writes anything.
//!
//! **Verdicts.** Every check is `PASS`, `WARN`, `FAIL` or `N/A`. Exit 0 when nothing FAILs, 1 when
//! a check FAILs, 2 when the command cannot run (usage, no lab, not a git repository). A failed
//! fetch does not stop the run: the later checks read the `origin/main` already present.
//!
//! Every git subprocess this module starts has every `GIT_*` variable removed except the ones a
//! fetch needs to authenticate (`GIT_SSH_COMMAND`, `GIT_SSH`, `GIT_ASKPASS`), so a caller's
//! `GIT_DIR` cannot point the checks at another repository, and `GIT_TERMINAL_PROMPT=0` so a fetch
//! never waits on a prompt. The `checkout` and `binary` checks call [`crate::version::stale`] and
//! [`crate::version::placement_line`], whose git runs with the caller's environment as it is.

use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str = "usage: uor-harness loop arrive [--json]";
/// A PICKUP at or above this size fails: it is a note for the next session, not a second log.
pub(crate) const PICKUP_MAX: usize = 8_192;
/// AGENTS.md is onboarding; above this many lines it has become a rule book.
const AGENTS_MAX_LINES: usize = 80;
/// STATE.md or HANDOFF.md at or above this size fails.
const VIEW_MAX: usize = 24_576;
/// `GIT_*` variables a git subprocess keeps: without them `fetch` cannot authenticate.
const GIT_KEEP: [&str; 3] = ["GIT_SSH_COMMAND", "GIT_SSH", "GIT_ASKPASS"];
/// The read path (AGENTS, CHARTER, STATE, HANDOFF and PICKUP) above this fails ...
const READ_PATH_FAIL: usize = 128_000;
/// ... and above this warns.
const READ_PATH_WARN: usize = 76_800;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verdict {
    Pass,
    Warn,
    Fail,
    Na,
}

impl Verdict {
    fn word(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Warn => "WARN",
            Verdict::Fail => "FAIL",
            Verdict::Na => "N/A",
        }
    }
}

struct Check {
    name: &'static str,
    verdict: Verdict,
    summary: String,
    details: Vec<String>,
}

fn check(name: &'static str, verdict: Verdict, summary: impl Into<String>) -> Check {
    Check {
        name,
        verdict,
        summary: summary.into(),
        details: Vec::new(),
    }
}

fn git_cmd(root: &Path) -> Command {
    let mut cmd = Command::new("git");
    for (k, _) in std::env::vars_os() {
        let name = k.to_string_lossy();
        if name.starts_with("GIT_") && !GIT_KEEP.contains(&name.as_ref()) {
            cmd.env_remove(k);
        }
    }
    cmd.env("GIT_TERMINAL_PROMPT", "0")
        .arg("-C")
        .arg(root)
        .args(["-c", "core.quotepath=false"]);
    cmd
}

/// Runs git in `root`; stdout on success, git's trimmed stderr on failure.
fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = git_cmd(root)
        .args(args)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// `loop arrive`'s entry point; `args` follow the action word.
pub fn cli(root: &Path, lab: &str, args: &[String]) -> u8 {
    let mut json = false;
    for a in args {
        match a.as_str() {
            "--json" => json = true,
            _ => {
                eprintln!("uor-harness: loop arrive: unexpected argument {a:?}\n{USAGE}");
                return 2;
            }
        }
    }
    if lab.trim().is_empty() {
        eprintln!("uor-harness: loop arrive: set UOR_LAB or pass --lab\n{USAGE}");
        return 2;
    }
    if git(root, &["rev-parse", "--git-dir"]).is_err() {
        eprintln!(
            "uor-harness: loop arrive: {} is not a git repository",
            root.display()
        );
        return 2;
    }
    let checks = run(root, lab);
    let ok = checks.iter().all(|c| c.verdict != Verdict::Fail);
    if json {
        let v = serde_json::json!({
            "lab": lab,
            "checks": checks.iter().map(|c| serde_json::json!({
                "check": c.name,
                "verdict": c.verdict.word(),
                "summary": c.summary,
                "details": c.details,
            })).collect::<Vec<_>>(),
            "ok": ok,
        });
        println!("{v}");
    } else {
        for c in &checks {
            println!("{} {} {}", c.verdict.word(), c.name, c.summary);
            for d in &c.details {
                println!("  {d}");
            }
        }
        println!("arrive: {}", if ok { "OK" } else { "FAIL" });
    }
    u8::from(!ok)
}

fn run(root: &Path, lab: &str) -> Vec<Check> {
    let mut checks = vec![
        fetch(root),
        checkout(root),
        binary(root),
        hooks(root),
        check("client-hooks", Verdict::Na, "the client guard is loop PR 9"),
    ];
    let rendered = crate::looprender::AtCommit::resolve(root, "origin/main")
        .and_then(|c| crate::looprender::render_all(&c));
    checks.push(views(root, lab, &rendered));
    // `loop close --session` writes the PICKUP under the main checkout, so a linked worktree reads
    // that one.
    let (pickup, pickup_c) = match crate::loopclose::pickup_path(root, lab) {
        Ok(p) => {
            let bytes = std::fs::read(&p).ok();
            let c = pickup_check(bytes.as_deref());
            (bytes, c)
        }
        Err(e) => {
            let mut c = check("pickup", Verdict::Fail, "cannot locate the PICKUP");
            c.details.push(e);
            (None, c)
        }
    };
    checks.push(pickup_c);
    let rendered = rendered.as_deref().unwrap_or_default();
    checks.push(read_path(
        root,
        rendered,
        pickup.as_ref().map_or(0, Vec::len),
    ));
    checks
}

fn fetch(root: &Path) -> Check {
    match git(root, &["fetch", "-q", "origin", "main"]) {
        Ok(_) => check("fetch", Verdict::Pass, "fetched origin main"),
        Err(e) => {
            let mut c = check(
                "fetch",
                Verdict::Fail,
                "git fetch origin main failed; the checks below read the origin/main already here",
            );
            c.details = e.lines().map(str::to_string).collect();
            c
        }
    }
}

fn checkout(root: &Path) -> Check {
    use crate::version::Staleness;
    match crate::version::stale(root) {
        Staleness::Current { commit } => check(
            "checkout",
            Verdict::Pass,
            format!("HEAD {commit} is at or ahead of origin/main"),
        ),
        Staleness::Behind { .. } => check(
            "checkout",
            Verdict::Warn,
            "this checkout is behind origin/main",
        ),
        Staleness::Unknown { why } => check(
            "checkout",
            Verdict::Warn,
            format!("cannot place this checkout against origin/main: {why}"),
        ),
    }
}

/// The running binary against the newest build in this checkout, by file age
/// ([`crate::version::placement_line`], which applies [`crate::version::is_stale_install`]).
///
/// No commit is baked into the binary (the crate's build.rs explains why: cargo caches the build,
/// so a baked commit goes stale on any commit that does not touch this crate), so age against the
/// checkout's own build is the comparison available.
fn binary(root: &Path) -> Check {
    match crate::version::placement_line(root) {
        Some(line) => {
            let mut c = check(
                "binary",
                Verdict::Warn,
                "this binary is older than this checkout's newest build",
            );
            c.details.push(line);
            c
        }
        None => check(
            "binary",
            Verdict::Pass,
            "not older than this checkout's newest build",
        ),
    }
}

/// The git `pre-commit` hook against the exact text `uor-harness git install-hooks` writes
/// ([`crate::githelp::HOOK_SCRIPT`], compiled from the crate's `rules/precommit-hook.sh`).
fn hooks(root: &Path) -> Check {
    const FIX: &str = "run `uor-harness git install-hooks`";
    let dir = match git(root, &["rev-parse", "--git-path", "hooks"]) {
        Ok(out) => {
            let p = PathBuf::from(String::from_utf8_lossy(&out).trim());
            if p.is_absolute() {
                p
            } else {
                root.join(p)
            }
        }
        Err(e) => {
            let mut c = check("hooks", Verdict::Warn, FIX);
            c.details
                .push(format!("cannot locate the hooks directory: {e}"));
            return c;
        }
    };
    let hook = dir.join("pre-commit");
    match std::fs::read_to_string(&hook) {
        Ok(t) if t == crate::githelp::HOOK_SCRIPT && executable(&hook) => check(
            "hooks",
            Verdict::Pass,
            format!("{} is the harness hook", hook.display()),
        ),
        Ok(t) if t == crate::githelp::HOOK_SCRIPT => {
            let mut c = check("hooks", Verdict::Warn, FIX);
            c.details.push(format!(
                "{} is the harness hook but is not executable, so git skips it",
                hook.display()
            ));
            c
        }
        Ok(_) => {
            let mut c = check("hooks", Verdict::Warn, FIX);
            c.details.push(format!(
                "{} differs from the hook this binary installs",
                hook.display()
            ));
            c
        }
        Err(_) => {
            let mut c = check("hooks", Verdict::Warn, FIX);
            c.details.push(format!("no {}", hook.display()));
            c
        }
    }
}

/// Whether git would run `hook`: on Unix, any execute bit is set.
#[cfg(unix)]
fn executable(hook: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(hook).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn executable(_hook: &Path) -> bool {
    true
}

/// Writes every view rendered from `origin/main` into `local/<lab>/views/`, then compares each view that is
/// also committed at `origin/main` with its render.
///
/// The first line of a view is its stamp, which names the commit its inputs were read at; a view
/// committed at `origin/main` cannot name `origin/main` itself, so the comparison skips the first
/// line of both and compares the rest byte for byte.
fn views(root: &Path, lab: &str, rendered: &Result<Vec<(String, String)>, String>) -> Check {
    let rendered = match rendered {
        Ok(v) => v,
        Err(e) => {
            let mut c = check(
                "views",
                Verdict::Fail,
                "cannot render the views at origin/main",
            );
            c.details.push(e.clone());
            return c;
        }
    };
    let out = root.join("local").join(lab).join("views");
    for (path, text) in rendered {
        let p = out.join(path);
        let written = p
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&p, text));
        if let Err(e) = written {
            let mut c = check("views", Verdict::Fail, "cannot write the rendered views");
            c.details.push(format!("{}: {e}", p.display()));
            return c;
        }
    }
    let mut committed = Vec::new();
    let mut details = Vec::new();
    for (path, text) in rendered {
        let spec = format!("origin/main:{path}");
        if git(root, &["cat-file", "-e", &spec]).is_err() {
            continue;
        }
        committed.push(path.as_str());
        let theirs = git(root, &["show", &spec]).unwrap_or_default();
        if body(&theirs) != body(text.as_bytes()) {
            details.push(format!(
                "the committed {path} disagrees with its inputs at origin/main; the repository's \
                 inputs win (run `loop render`)"
            ));
        }
    }
    let shown = format!("local/{lab}/views");
    if !details.is_empty() {
        let mut c = check(
            "views",
            Verdict::Fail,
            format!("rendered to {shown}; a committed view disagrees with its inputs"),
        );
        c.details = details;
        c
    } else if committed.is_empty() {
        check(
            "views",
            Verdict::Pass,
            format!("rendered to {shown}; no view is committed yet (loop PR 8)"),
        )
    } else {
        check(
            "views",
            Verdict::Pass,
            format!(
                "rendered to {shown}; the committed {} agree with their inputs",
                committed.join(", ")
            ),
        )
    }
}

/// Everything after the first line (the stamp).
fn body(text: &[u8]) -> &[u8] {
    text.iter()
        .position(|&b| b == b'\n')
        .map_or(&[][..], |i| &text[i + 1..])
}

fn pickup_check(pickup: Option<&[u8]>) -> Check {
    let Some(bytes) = pickup else {
        return check("pickup", Verdict::Na, "no PICKUP (first session)");
    };
    let text = String::from_utf8_lossy(bytes);
    let mut details = Vec::new();
    if bytes.len() >= PICKUP_MAX {
        details.push(format!(
            "{} bytes; a PICKUP must stay under {PICKUP_MAX}",
            bytes.len()
        ));
    }
    for line in uncited_standing_references(&text) {
        details.push(format!(
            "standing reference cites no charter section (§n), decision (Dn) or lesson (L-nnn): \
             {line}"
        ));
    }
    if details.is_empty() {
        check(
            "pickup",
            Verdict::Pass,
            format!("local PICKUP.md, {} bytes", bytes.len()),
        )
    } else {
        let mut c = check(
            "pickup",
            Verdict::Fail,
            format!("local PICKUP.md, {} bytes", bytes.len()),
        );
        c.details = details;
        c
    }
}

/// The non-blank lines under `## Standing references` (until the next `## ` outside a fenced code
/// block) that cite none of a charter section, a decision row or a lesson id: the loop page lets a
/// standing-rule line cite only those, so no lab keeps a private rulebook. Fenced code blocks and
/// heading lines (`#`-prefixed) inside the section are not reference lines.
fn uncited_standing_references(text: &str) -> Vec<String> {
    let mut inside = false;
    let mut bad = Vec::new();
    for (line, can_head) in crate::loopprcheck::heading_lines(text) {
        if !can_head {
            continue;
        }
        if line.starts_with("## ") {
            inside = line.trim_end() == "## Standing references";
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        if inside && !line.trim().is_empty() && !cites(line) {
            bad.push(line.trim().to_string());
        }
    }
    bad
}

/// Whether `line` cites `§<digit>`, `D<digits>` or `L-<3 digits>` (the letter not inside a word).
fn cites(line: &str) -> bool {
    let c: Vec<char> = line.chars().collect();
    let digit = |i: usize| c.get(i).is_some_and(char::is_ascii_digit);
    let word_start = |i: usize| i == 0 || !c[i - 1].is_alphanumeric();
    (0..c.len()).any(|i| match c[i] {
        '§' => digit(i + 1),
        'D' => word_start(i) && digit(i + 1),
        'L' => {
            word_start(i)
                && c.get(i + 1) == Some(&'-')
                && (2..5).all(|k| digit(i + k))
                && !digit(i + 5)
        }
        _ => false,
    })
}

/// Measures the read path at `origin/main`. A view not committed there is measured as `rendered`
/// (the copy just written to `local/<lab>/views/`), since that is the one a lab reads.
fn read_path(root: &Path, rendered: &[(String, String)], pickup: usize) -> Check {
    let at = |path: &str| git(root, &["show", &format!("origin/main:{path}")]).ok();
    let view = |path: &str| {
        at(path).map(|b| b.len()).unwrap_or_else(|| {
            rendered
                .iter()
                .find(|(p, _)| p == path)
                .map_or(0, |(_, t)| t.len())
        })
    };
    let agents = at("AGENTS.md").unwrap_or_default();
    let agents_lines = String::from_utf8_lossy(&agents).lines().count();
    read_path_budget(ReadPath {
        agents: agents.len(),
        agents_lines,
        charter: at("CHARTER.md").unwrap_or_default().len(),
        state: view("STATE.md"),
        handoff: view("HANDOFF.md"),
        pickup,
    })
}

/// The sizes, in bytes, of the files a lab reads on arrival.
struct ReadPath {
    agents: usize,
    agents_lines: usize,
    charter: usize,
    state: usize,
    handoff: usize,
    pickup: usize,
}

fn read_path_budget(r: ReadPath) -> Check {
    let ReadPath {
        agents,
        agents_lines,
        charter,
        state,
        handoff,
        pickup,
    } = r;
    let total = agents + charter + state + handoff + pickup;
    let summary = format!(
        "{total} bytes (AGENTS.md {agents} bytes / {agents_lines} lines, CHARTER.md {charter}, \
         STATE.md {state}, HANDOFF.md {handoff}, PICKUP.md {pickup})"
    );
    let mut fails = Vec::new();
    if agents_lines > AGENTS_MAX_LINES {
        fails.push(format!(
            "AGENTS.md has {agents_lines} lines; the limit is {AGENTS_MAX_LINES}"
        ));
    }
    for (name, n) in [("STATE.md", state), ("HANDOFF.md", handoff)] {
        if n >= VIEW_MAX {
            fails.push(format!(
                "{name} is {n} bytes; it must stay under {VIEW_MAX}"
            ));
        }
    }
    if total > READ_PATH_FAIL {
        fails.push(format!(
            "the read path is {total} bytes; the limit is {READ_PATH_FAIL}"
        ));
    }
    let mut c = check("read-path", Verdict::Pass, summary);
    if !fails.is_empty() {
        c.verdict = Verdict::Fail;
        c.details = fails;
    } else if total > READ_PATH_WARN {
        c.verdict = Verdict::Warn;
        c.details.push(format!(
            "the read path is above {READ_PATH_WARN} bytes; trim it before it reaches {READ_PATH_FAIL}"
        ));
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citations() {
        assert!(cites("- keep PRs small (D59)"));
        assert!(cites("- see §5.10"));
        assert!(cites("- L-012 applies"));
        assert!(!cites("- AND5 is not a decision"));
        assert!(!cites("- L-12 is too short"));
        assert!(!cites("- my own rule"));
    }

    fn small() -> ReadPath {
        ReadPath {
            agents: 100,
            agents_lines: 10,
            charter: 1_000,
            state: 1_000,
            handoff: 1_000,
            pickup: 0,
        }
    }

    #[test]
    fn a_rendered_state_of_25000_bytes_fails_the_read_path() {
        assert_eq!(read_path_budget(small()).verdict, Verdict::Pass);
        let c = read_path_budget(ReadPath {
            state: 25_000,
            ..small()
        });
        assert_eq!(c.verdict, Verdict::Fail);
        assert!(
            c.details[0].contains("STATE.md is 25000 bytes"),
            "{:?}",
            c.details
        );
    }

    #[test]
    fn standing_references_skip_fences_and_subheadings() {
        let text = "## Standing references\n\n### Charter\n\n- see §5.10\n\n```\nnot a rule\n## not a heading\nstill fenced\n```\n\n- uncited after the fence\n\n## Next\n\n- free text\n";
        assert_eq!(
            uncited_standing_references(text),
            vec!["- uncited after the fence".to_string()]
        );
    }
}
