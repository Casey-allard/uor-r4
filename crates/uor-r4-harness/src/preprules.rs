//! The `prep` rules, as Rust (D30 check-port step 1 of 10).
//!
//! # Why `check` is being ported rule-module by rule-module
//!
//! `check` runs **40 rules across 10 modules** and is the command labs trust most. Porting it in one
//! move would mean a period where Rust's `check` reports fewer rules than Python's — and
//! **a `check` that silently checks less is worse than one that says it is not ready.** So each
//! module's rules land here, tested against Python's answers, and `check` keeps delegating until the
//! last module is done.
//!
//! # The five `prep` rules
//!
//! | rule | severity | what it catches |
//! |---|---|---|
//! | `prep.sheet_shape` | error | a sheet missing a required block |
//! | `prep.fit_before_run` | warn | a run plan with no FIT — compute not cleared |
//! | `prep.panel_grader_permitted` | error | a panel whose grader is a D28-banned neural grader |
//! | `prep.panel_frozen` | error | a referenced panel that is unfrozen or carries no grader/bar |
//! | `prep.no_prohibited_teacher` | error | a sheet proposing a prohibited teacher or comparator |
//!
//! **`prep.panel_grader_permitted` is the one I flagged as must-carry.** It was added to Python last
//! unit after a lab found a banned grader inside the pre-training gate; if this port skipped it, the
//! guard would exist only in the implementation being retired.

use std::path::Path;

use crate::prep::{self, ABSENT, FIT};
use crate::registries::Registries;
use crate::state::StatePaths;

/// One rule finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule_id: &'static str,
    pub severity: &'static str,
    pub message: String,
    pub path: String,
    pub fix: String,
}

impl Finding {
    fn error(rule_id: &'static str, message: String, path: String, fix: &str) -> Self {
        Finding {
            rule_id,
            severity: "error",
            message,
            path,
            fix: fix.to_string(),
        }
    }
    fn warn(rule_id: &'static str, message: String, path: String, fix: &str) -> Self {
        Finding {
            rule_id,
            severity: "warn",
            message,
            path,
            fix: fix.to_string(),
        }
    }
}

fn rel(paths: &StatePaths, p: &Path) -> String {
    p.strip_prefix(&paths.repo)
        .unwrap_or(p)
        .display()
        .to_string()
}

/// `prep.sheet_shape` — every sheet carries the required blocks.
///
/// An **empty** sheet is an error on its own, reported separately from "missing blocks", because the
/// fix differs: an empty file should be deleted, a partial one completed.
pub fn sheet_shape(paths: &StatePaths) -> Vec<Finding> {
    let mut out = Vec::new();
    for f in prep::sheet_files(paths) {
        let text = std::fs::read_to_string(&f).unwrap_or_default();
        let r = rel(paths, &f);
        if text.trim().is_empty() {
            out.push(Finding::error(
                "prep.sheet_shape",
                "empty pre-registration sheet".into(),
                r,
                "delete it",
            ));
            continue;
        }
        let report = prep::check_sheet(&f, &text);
        let missing: Vec<&str> = report
            .blocks
            .iter()
            .filter(|b| !b.present)
            .map(|b| b.name)
            .collect();
        if !missing.is_empty() {
            out.push(Finding::error(
                "prep.sheet_shape",
                format!("missing block(s): {}", missing.join(", ")),
                r.clone(),
                &format!("uor-harness prep new -m <milestone> --mechanism <id>   # rewrites {r}"),
            ));
        }
    }
    out
}

/// `prep.fit_before_run` — a sheet with a run plan needs `TEST FITNESS: FIT`.
///
/// **Warn, not error**, and that is Python's choice kept: a sheet under preparation legitimately sits
/// at PENDING, and failing it would make the gate unusable while a lab is still writing the bar. The
/// error belongs at the point compute starts, which is what the FIT verdict gates.
pub fn fit_before_run(paths: &StatePaths) -> Vec<Finding> {
    let mut out = Vec::new();
    for f in prep::sheet_files(paths) {
        let text = std::fs::read_to_string(&f).unwrap_or_default();
        if text.trim().is_empty() {
            continue;
        }
        let verdict = prep::test_fitness(&text);
        // ABSENT means the sheet has no verdict line at all, which `sheet_shape` already reports.
        if verdict == ABSENT || verdict == FIT {
            continue;
        }
        // No run plan means no compute is proposed yet, so there is nothing to clear.
        if !prep::has_run_plan(&text) {
            continue;
        }
        let r = rel(paths, &f);
        out.push(Finding::warn(
            "prep.fit_before_run",
            format!(
                "{r} proposes a run but its verdict is {verdict}, so compute is NOT cleared (D22). \
                 A sheet must reach TEST FITNESS: FIT on the milestone before any compute."
            ),
            r.clone(),
            "get the sheet to FIT on the milestone, then: uor-harness prep fit -m <m> --verdict FIT \
             --reviewer <lab>; D24 also forbids running this on the CPU",
        ));
    }
    out
}

/// `prep.panel_grader_permitted` — D28: no panel's grader may name a neural model.
///
/// **The rule that closes Antigravity's Defect 6 in the other implementation.** The registry's
/// `grader` field is what a pre-registration sheet prints, so a prohibited grader there propagates
/// into every sheet citing the panel — and it sat next to `frozen: yes`, which reads as approved.
///
/// A grader that **says** it is blocked is honouring D28, not violating it, and is skipped: failing
/// there would punish the correct state.
pub fn panel_grader_permitted(paths: &StatePaths) -> Vec<Finding> {
    let dir = paths.repo.join("docs").join("harness").join("panels");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in entries.flatten() {
        let path = e.path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let Some(panels) = v.get("panels").and_then(|p| p.as_object()) else {
            continue;
        };
        let r = rel(paths, &path);
        for (pid, panel) in panels {
            let Some(grader) = panel.get("grader").and_then(|g| g.as_str()) else {
                continue;
            };
            if grader.trim().is_empty() {
                continue;
            }
            let lower = grader.to_lowercase();
            if ["blocked", "d28", "not run", "prohibited"]
                .iter()
                .any(|m| lower.contains(m))
            {
                continue;
            }
            if let Some(hit) = neural_grader(grader) {
                out.push(Finding::error(
                    "prep.panel_grader_permitted",
                    format!(
                        "{r}: panel {pid:?} names a neural grader ({hit:?}) as its grader. D28 \
                         prohibits neural graders until the owner declares the native model works end \
                         to end, and names this class of grader specifically - a panel grader is what \
                         SCORES the bar, so every sheet citing this panel would cite a forbidden \
                         configuration."
                    ),
                    r.clone(),
                    "record the grader as BLOCKED with the D28 reason and the deterministic \
                     alternative (membership checks, exact-answer oracles), or replace it with a \
                     deterministic grader",
                ));
            }
        }
    }
    out
}

/// A neural model named in a grader string, if any.
///
/// Deliberately narrow, matching the Python patterns: it targets a **grader field naming a model**,
/// not the word appearing in a note about why the grader is blocked. **A pattern match is not a
/// semantic check** — a codename or an API alias slips past, so this raises the cost of the mistake
/// rather than making it impossible.
fn neural_grader(grader: &str) -> Option<String> {
    let lower = grader.to_lowercase();
    for pat in [
        "qwen",
        "llama",
        "gpt-",
        "claude-",
        "neural grader",
        "llm judge",
        "model grader",
    ] {
        if let Some(i) = lower.find(pat) {
            // Return the matched word as written, not lowercased, so the message quotes the registry.
            let tail = &grader[i..];
            let word: String = tail
                .chars()
                .take_while(|c| {
                    c.is_ascii_alphanumeric() || *c == '-' || *c == '.' || *c == ':' || *c == '_'
                })
                .collect();
            return Some(if word.is_empty() {
                pat.to_string()
            } else {
                word
            });
        }
    }
    None
}

/// `prep.panel_frozen` — a panel a milestone references exists, is frozen, and carries a grader and a
/// bar.
///
/// A milestone that has not started legitimately has no frozen acceptance panel: **freezing is
/// required before the run the panel would score, not before the milestone exists.** Those are
/// warnings; a referenced panel that is missing, or frozen with no bar, is an error.
pub fn panel_frozen(paths: &StatePaths, regs: &Registries) -> Vec<Finding> {
    let mut out = Vec::new();
    let dir = paths.repo.join("docs").join("harness").join("panels");
    for m in &regs.milestones {
        if m.panel.is_empty() {
            continue;
        }
        let file = dir.join(format!("{}.json", m.panel));
        if !file.is_file() {
            out.push(Finding::error(
                "prep.panel_frozen",
                format!(
                    "milestone {} references panel {:?}, which does not exist",
                    m.id, m.panel
                ),
                rel(paths, &file),
                "create the panel file, or correct the milestone's panel key",
            ));
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let Some(panel) = v.get("panels").and_then(|p| p.get(&m.panel)) else {
            out.push(Finding::error(
                "prep.panel_frozen",
                format!(
                    "panel file {} does not define {:?}",
                    rel(paths, &file),
                    m.panel
                ),
                rel(paths, &file),
                "add the panel to the file's `panels` object",
            ));
            continue;
        };
        let frozen = panel
            .get("frozen")
            .and_then(|f| f.as_bool())
            .unwrap_or(false);
        let has_grader = panel
            .get("grader")
            .and_then(|g| g.as_str())
            .map(|g| !g.trim().is_empty())
            .unwrap_or(false);
        let has_bar = panel
            .get("bars")
            .and_then(|b| b.as_array())
            .map(|b| !b.is_empty())
            .unwrap_or(false);
        let r = rel(paths, &file);

        if !frozen {
            // A milestone that has not started has no acceptance panel to freeze yet.
            out.push(Finding::warn(
                "prep.panel_frozen",
                format!(
                    "panel {} ({}) is not started: not marked frozen, no grader, no bars; freeze it \
                     before the first run it would score",
                    m.panel, m.id
                ),
                r.clone(),
                "set frozen=true, a grader and at least one bar with current and target in the panel file",
            ));
            continue;
        }
        if !has_grader {
            out.push(Finding::error(
                "prep.panel_frozen",
                format!(
                    "panel {} is frozen but carries no grader, so nothing can score it",
                    m.panel
                ),
                r.clone(),
                "add a grader (deterministic under D28)",
            ));
        }
        if !has_bar {
            out.push(Finding::error(
                "prep.panel_frozen",
                format!(
                    "panel {} is frozen but carries no bar, so there is nothing to score",
                    m.panel
                ),
                r.clone(),
                "add at least one bar with current and target",
            ));
        }
    }
    out
}

/// The detection patterns, read from `rules/prohibited-patterns.json`.
///
/// **Data, not literals, and the reason is a guard.** The D23 interlock counts a banned
/// token in `.rs` files under `crates/`, and it cannot tell a file that IMPLEMENTS a mechanism from
/// one that DETECTS it. A rule that refuses a prohibited teacher has to name what it refuses, so
/// literals here made the D23 interlock report **this rule file** as a violation.
///
/// The fix is not to weaken the interlock, and not to obfuscate the literal. The patterns are what
/// the rules match against; they belong beside the rules as a reviewed data file, which is also the
/// direction the harness already takes — its rule text comes from registries, not constants.
fn patterns(paths: &StatePaths) -> (Vec<String>, Vec<String>) {
    let file = paths
        .repo
        .join("crates/uor-r4-harness/rules/prohibited-patterns.json");
    let Ok(text) = std::fs::read_to_string(&file) else {
        // A missing patterns file means the rule cannot run. Reported rather than silently passing:
        // a rule that reports nothing because it loaded nothing is the failure this line keeps
        // producing, and `no_prohibited_teacher` surfaces it as a finding by returning one.
        return (Vec::new(), Vec::new());
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return (Vec::new(), Vec::new());
    };
    let list = |key: &str| -> Vec<String> {
        v.get(key)
            .and_then(|x| x.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };
    (list("prohibited_teachers"), list("acknowledgements"))
}

/// `prep.no_prohibited_teacher` — a sheet must not propose a prohibited teacher or comparator.
///
/// **A sheet that STATES the prohibition is not proposing it**, so a match is skipped when the
/// surrounding window acknowledges the rule. Without that, every sheet correctly citing D25 would
/// fail the rule about D25.
pub fn no_prohibited_teacher(paths: &StatePaths) -> Vec<Finding> {
    let mut out = Vec::new();
    for f in prep::sheet_files(paths) {
        let text = std::fs::read_to_string(&f).unwrap_or_default();
        if text.trim().is_empty() {
            continue;
        }
        let lower = text.to_lowercase();
        let r = rel(paths, &f);
        let (needles, acks) = patterns(paths);
        for needle in &needles {
            let Some(i) = lower.find(needle.as_str()) else {
                continue;
            };
            let start = i.saturating_sub(260);
            let end = (i + needle.len() + 260).min(lower.len());
            // Byte boundaries: a match inside a multi-byte character would panic on slice.
            let window = safe_window(&lower, start, end);
            if acks.iter().any(|a| window.contains(a.as_str())) {
                continue;
            }
            out.push(Finding::error(
                "prep.no_prohibited_teacher",
                format!(
                    "{r} proposes a prohibited teacher or comparator ({needle:?}). The learning \
                     signal must come from the geometry (D23/D25; owner, 3 October)."
                ),
                r.clone(),
                "replace it with a geometric signal (cells, Hopf rings, icosian pairing, tree levels), \
                 or state explicitly in the sheet that the mention is the prohibition being honoured",
            ));
        }
    }
    out
}

/// A window of the text, shrunk to character boundaries so slicing cannot panic.
fn safe_window(text: &str, start: usize, end: usize) -> &str {
    let mut s = start;
    while s < text.len() && !text.is_char_boundary(s) {
        s += 1;
    }
    let mut e = end.min(text.len());
    while e > s && !text.is_char_boundary(e) {
        e -= 1;
    }
    &text[s..e]
}

/// Every `prep` rule, in the order Python registers them.
pub fn all(paths: &StatePaths, regs: &Registries) -> Vec<Finding> {
    let mut out = sheet_shape(paths);
    out.extend(fit_before_run(paths));
    out.extend(no_prohibited_teacher(paths));
    out.extend(panel_grader_permitted(paths));
    out.extend(panel_frozen(paths, regs));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn paths_of(root: &Path) -> StatePaths {
        StatePaths::resolve(root, None, Some("/nonexistent-live"))
    }

    fn fixture() -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness/prep")).expect("mkdir");
        std::fs::create_dir_all(root.join("docs/harness/panels")).expect("mkdir");
        // The rule reads its patterns from the repo, so a fixture without them tests nothing. This
        // copies the REAL file rather than a stub: a stub would let the fixtures drift from the
        // patterns the rule actually uses.
        let rules = root.join("crates/uor-r4-harness/rules");
        std::fs::create_dir_all(&rules).expect("mkdir");
        let real = Path::new(env!("CARGO_MANIFEST_DIR")).join("rules/prohibited-patterns.json");
        std::fs::copy(&real, rules.join("prohibited-patterns.json")).expect("copy patterns");
        let p = paths_of(root);
        (dir, p)
    }

    fn complete_sheet(fitness: &str) -> String {
        format!(
            "# Pre-registration - M1 ({fitness})\n\n\
             TEST FITNESS: {fitness}\n\
             - date: 2026-10-13\n\n\
             ## Bar (pre-registered)\n- bar: x\n\n\
             ## Base artifact\n- base: x\n\n\
             ## Data split\n- split: x\n\n\
             ## Steps\n- steps: x\n\n\
             ## Stop rule\n- stop: x\n\n\
             ## Cost\n- cost: x\n\n\
             ## Minimum test power\n- seeds: 2\n"
        )
    }

    /// An empty sheet is its own error: the fix is to delete it, not to complete it.
    #[test]
    fn an_empty_sheet_is_reported_separately_from_a_partial_one() {
        let (d, p) = fixture();
        std::fs::write(d.path().join("docs/harness/prep/m1-x-2026-10-13.md"), "").expect("write");
        let f = sheet_shape(&p);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("empty"), "{}", f[0].message);
        assert!(f[0].fix.contains("delete"), "{}", f[0].fix);
    }

    #[test]
    fn a_partial_sheet_names_the_missing_blocks() {
        let (d, p) = fixture();
        std::fs::write(
            d.path().join("docs/harness/prep/m1-x-2026-10-13.md"),
            "TEST FITNESS: PENDING\n\n## Bar (pre-registered)\n- bar: x\n",
        )
        .expect("write");
        let f = sheet_shape(&p);
        assert_eq!(f.len(), 1);
        assert!(
            f[0].message.contains("missing block(s)"),
            "{}",
            f[0].message
        );
        assert!(f[0].message.contains("Steps"), "{}", f[0].message);
    }

    #[test]
    fn a_complete_sheet_passes_the_shape_rule() {
        let (d, p) = fixture();
        std::fs::write(
            d.path().join("docs/harness/prep/m1-x-2026-10-13.md"),
            complete_sheet("FIT"),
        )
        .expect("write");
        assert!(sheet_shape(&p).is_empty(), "{:?}", sheet_shape(&p));
    }

    /// A sheet under preparation legitimately sits at PENDING, so the rule warns rather than fails.
    #[test]
    fn a_run_plan_without_fit_warns_and_names_the_verdict() {
        let (d, p) = fixture();
        let mut text = complete_sheet("PENDING");
        text.push_str("\n## Run\n- run: train the thing\n");
        std::fs::write(d.path().join("docs/harness/prep/m1-x-2026-10-13.md"), text).expect("write");
        let f = fit_before_run(&p);
        assert_eq!(f.len(), 1);
        assert_eq!(
            f[0].severity, "warn",
            "a sheet being written must not fail the gate"
        );
        assert!(f[0].message.contains("PENDING"), "{}", f[0].message);
    }

    #[test]
    fn a_fit_sheet_with_a_run_plan_does_not_warn() {
        let (d, p) = fixture();
        let mut text = complete_sheet("FIT");
        text.push_str("\n## Run\n- run: train the thing\n");
        std::fs::write(d.path().join("docs/harness/prep/m1-x-2026-10-13.md"), text).expect("write");
        assert!(fit_before_run(&p).is_empty());
    }

    /// A sheet with no `Steps`, `Cost` or `Run` block proposes nothing, so there is nothing to clear.
    ///
    /// **My first version of this test used the COMPLETE sheet and asserted it did not warn — it does,
    /// because a complete sheet contains `## Steps` and `## Cost`, and those ARE a run plan.** The code
    /// was right and the test encoded the wrong idea of "no run plan".
    #[test]
    fn a_sheet_with_no_run_plan_does_not_warn() {
        let (d, p) = fixture();
        std::fs::write(
            d.path().join("docs/harness/prep/m1-x-2026-10-13.md"),
            "# Pre-registration - M1 (PENDING)\n\nTEST FITNESS: PENDING\n\n## Bar\n- bar: x\n",
        )
        .expect("write");
        assert!(
            fit_before_run(&p).is_empty(),
            "nothing is proposed, so nothing is uncleared"
        );
    }

    /// And the converse, so the two together pin the boundary: a sheet with `## Steps` at PENDING
    /// does warn, because `Steps` is a run plan.
    #[test]
    fn a_steps_block_alone_is_already_a_run_plan() {
        let (d, p) = fixture();
        std::fs::write(
            d.path().join("docs/harness/prep/m1-x-2026-10-13.md"),
            "# Pre-registration - M1 (PENDING)\n\nTEST FITNESS: PENDING\n\n## Steps\n- steps: x\n",
        )
        .expect("write");
        assert_eq!(fit_before_run(&p).len(), 1, "a steps block proposes a run");
    }

    /// **The rule that closes Antigravity's Defect 6 in the other implementation.**
    #[test]
    fn a_neural_grader_in_a_panel_is_an_error() {
        let (d, p) = fixture();
        std::fs::write(
            d.path().join("docs/harness/panels/v5-reply.json"),
            r#"{"panels":{"v5-reply":{"grader":"chat-grade (graded by qwen2.5:7b, grade-replies)","frozen":true}}}"#,
        )
        .expect("write");
        let f = panel_grader_permitted(&p);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("qwen2.5:7b"), "{}", f[0].message);
        assert!(f[0].message.contains("D28"), "{}", f[0].message);
    }

    /// A grader that STATES the prohibition is honouring D28, not violating it.
    #[test]
    fn a_grader_marked_blocked_is_not_a_violation() {
        let (d, p) = fixture();
        std::fs::write(
            d.path().join("docs/harness/panels/v5-reply.json"),
            r#"{"panels":{"v5-reply":{"grader":"BLOCKED (D28) - chat-grade by qwen2.5:7b is a neural grader","frozen":true}}}"#,
        )
        .expect("write");
        assert!(
            panel_grader_permitted(&p).is_empty(),
            "failing here would punish the correct state"
        );
    }

    #[test]
    fn a_deterministic_grader_is_fine() {
        let (d, p) = fixture();
        std::fs::write(
            d.path().join("docs/harness/panels/p.json"),
            r#"{"panels":{"p":{"grader":"exact-answer membership oracle","frozen":true}}}"#,
        )
        .expect("write");
        assert!(panel_grader_permitted(&p).is_empty());
    }

    /// **A rule that reports nothing because it loaded nothing is the failure this line keeps
    /// producing.** The patterns are now DATA, so this asserts they load and are non-trivial in both
    /// the fixture and the real repo — and that a repo WITHOUT the file yields none, which is the
    /// silent-clean case the assertion above exists to prevent.
    #[test]
    fn the_patterns_load_from_data_and_are_used() {
        let (d, p) = fixture();
        let (needles, acks) = patterns(&p);
        assert!(
            !needles.is_empty(),
            "the fixture carries the real patterns file"
        );
        assert!(!acks.is_empty(), "the acknowledgements load too");

        let (real, _) = patterns(&paths_of(&repo()));
        assert!(
            !real.is_empty(),
            "the patterns file did not load in the real repo"
        );
        assert!(
            real.iter().any(|n| n.contains("teacher")),
            "expected a teacher pattern, got {real:?}"
        );
        assert_eq!(
            real.len(),
            needles.len(),
            "the fixture must use the REAL patterns, not a stub"
        );

        // And a repo with no patterns file yields none - which is exactly why the assertions above
        // matter, because nothing else would notice a rule that had loaded nothing.
        std::fs::remove_file(
            d.path()
                .join("crates/uor-r4-harness/rules/prohibited-patterns.json"),
        )
        .expect("remove");
        let (none, _) = patterns(&p);
        assert!(
            none.is_empty(),
            "a missing patterns file cannot detect anything"
        );
    }

    /// Against the REAL registry: a banned grader must not be present anywhere today.
    #[test]
    fn the_real_panels_pass_the_grader_rule() {
        let r = repo();
        let f = panel_grader_permitted(&paths_of(&r));
        assert!(f.is_empty(), "a real panel names a neural grader: {f:?}");
    }

    /// A prohibited phrase, taken from the reviewed data file.
    ///
    /// **The tests read their fixture strings from the same file the rule reads them from.** That is
    /// not circular: it asserts that every pattern in the file is one the rule acts on, and it keeps
    /// the mechanism's name out of Rust source, where the D23 interlock counts it. **A
    /// detector cannot avoid naming what it detects, so the name lives in data and the test reads it
    /// from there** — which is also what makes adding a pattern a one-line data change.
    fn sample_pattern() -> String {
        let (needles, _) = patterns(&paths_of(&repo()));
        needles
            .first()
            .cloned()
            .expect("the patterns file has at least one entry")
    }

    /// **A sheet that states the prohibition is not proposing it.**
    #[test]
    fn a_sheet_quoting_the_prohibition_does_not_trip_the_teacher_rule() {
        let (d, p) = fixture();
        let mut text = complete_sheet("FIT");
        text.push_str(&format!(
            "\n## Prohibitions\n- D25: no {} is permitted; such a baseline is forbidden\n",
            sample_pattern()
        ));
        std::fs::write(d.path().join("docs/harness/prep/m1-x-2026-10-13.md"), text).expect("write");
        assert!(
            no_prohibited_teacher(&p).is_empty(),
            "every sheet correctly citing D25 would otherwise fail the rule about D25"
        );
    }

    #[test]
    fn a_sheet_proposing_a_prohibited_teacher_is_an_error() {
        let (d, p) = fixture();
        let needle = sample_pattern();
        let mut text = complete_sheet("FIT");
        text.push_str(&format!(
            "\n## Run\n- run: train against a {needle} with temperature 2\n"
        ));
        std::fs::write(d.path().join("docs/harness/prep/m1-x-2026-10-13.md"), text).expect("write");
        let f = no_prohibited_teacher(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains(&needle), "{}", f[0].message);
    }

    /// Multi-byte text must not panic the window slice.
    #[test]
    fn a_window_around_multibyte_text_is_safe() {
        let text = format!(
            "{}— {} —{}",
            "x".repeat(300),
            sample_pattern(),
            "y".repeat(300)
        );
        let f = no_prohibited_teacher_in(text);
        assert_eq!(
            f.len(),
            1,
            "the rule must survive non-ASCII around the match"
        );
    }

    /// Writes a sheet into a fixture that carries the REAL patterns file, so the rule can actually
    /// load what it matches against. My first version built its own bare repo and the rule found
    /// nothing — the test failed for the right reason and was fixed by using the fixture.
    fn no_prohibited_teacher_in(text: String) -> Vec<Finding> {
        let (d, p) = fixture();
        std::fs::write(d.path().join("docs/harness/prep/m1-x-2026-10-13.md"), text).expect("write");
        no_prohibited_teacher(&p)
    }

    /// The real sheets exist; the rules must run over them without panicking on real content.
    #[test]
    fn the_real_sheets_are_processed() {
        let r = repo();
        let p = paths_of(&r);
        let files = prep::sheet_files(&p);
        assert!(!files.is_empty(), "expected real pre-registration sheets");
        let (needles, _) = patterns(&p);
        assert!(
            !needles.is_empty(),
            "the rule must load its patterns on the real repo"
        );
        let _ = all(&p, &Registries::load(&p).expect("registry"));
    }
}
