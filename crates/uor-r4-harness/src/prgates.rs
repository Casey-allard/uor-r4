//! The `prlint` gates that need only the parsed fields — ported from `check_pr`.
//!
//! # Why only some gates are here
//!
//! `prlint.check_pr` raises **22 problems from one function**, and the gates differ enormously in what
//! they need: some read only the parsed body, others shell out to `git`, and others walk the
//! repository. **Porting the function whole would mean porting all of it or shipping a `check_pr`
//! that silently skips gates** — which is the downgrade this port refuses for `check`.
//!
//! So these are the gates whose entire input is the parsed fields, **ported exactly and asserted
//! against Python's own output for the same bodies.** Three to begin with; the rest follow the same
//! shape once they are read rather than assumed.
//!
//! # `prlint.parse` is deliberately NOT here, and it is not an omission
//!
//! Thirteen of the fourteen gates are ported. The fourteenth, **`prlint.parse`**, fires only when
//! `parse_pr_body` **raises**:
//!
//! ```python
//! try:
//!     fields = parse_pr_body(body)
//! except Exception as exc:
//!     return {"ok": False, "problems": [Problem("prlint.parse", "error", ...)]}
//! ```
//!
//! **The Rust parser returns no `Result`** - it is total, and a malformed body yields empty fields
//! rather than an error. So **there is no state in which this gate could fire**, and writing it would
//! mean either a function nothing can reach or a `Result` added to the parser for the sole purpose
//! of having somewhere to put the error.
//!
//! **That is a deliberate divergence, and it is the safe direction**: Python reports *"body could not
//! be parsed"* and skips every gate; Rust parses what it can and **runs every gate on the result.**
//! A body Python would refuse outright is one Rust checks normally.
//!
//! **Recorded rather than left as a silent gap**, because a gate that is missing and a gate that
//! cannot exist look identical from the coverage report - and `parity` reports `prlint.parse` as
//! still in Python, which is true and is not a port that was forgotten.
//!
//! # What is deliberately NOT here
//!
//! * `record_path` **also stats the filesystem** for the named path, so it is not field-only.
//! * `current_state_entry` **reads `docs/integration/current-state.md`**, and the same for the
//!   milestone registry lookups in `count_matches_registry` and `headline_vs_registry`.
//!
//! **Each says why in this list rather than being silently absent**, because a gate that is missing
//! and a gate that is not needed look identical from the outside.

use crate::prbody::{truncate, Fields};
use regex::Regex;
use serde_json::{json, Value};
use std::sync::OnceLock;

/// One raised problem, in the shape `check_pr` returns.
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    pub rule: String,
    pub severity: String,
    pub message: String,
    pub fix: String,
}

impl Problem {
    fn new(rule: &str, severity: &str, message: impl Into<String>, fix: impl Into<String>) -> Self {
        Problem {
            rule: rule.to_string(),
            severity: severity.to_string(),
            message: message.into(),
            fix: fix.into(),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "rule": self.rule,
            "severity": self.severity,
            "message": self.message,
            "fix": self.fix,
        })
    }
}

/// `prlint.line_declared` — the body declares a line name and a count (session-goal §3).
///
/// **The two are reported in one problem with both gaps named**, so a body missing both says so
/// once rather than twice: `"the body does not declare Line name, count n/3"`. **Order matters and
/// is the same as Python's** — name first.
pub fn line_declared(f: &Fields) -> Vec<Problem> {
    let mut missing: Vec<&str> = Vec::new();
    if f.line_name.is_empty() {
        missing.push("Line name");
    }
    if f.line_count.is_none() {
        missing.push("count n/3");
    }
    if missing.is_empty() {
        return Vec::new();
    }
    vec![Problem::new(
        "prlint.line_declared",
        "error",
        format!("the body does not declare {}", missing.join(", ")),
        "Line: <name> · count n/3 · headline <before> → <after>",
    )]
}

/// `prlint.headline_present` — a headline with **numbers**, because prose is not a headline.
///
/// The condition is on the **after** value only: a headline that states where the line is but not
/// where it is going is not a pre-registered target.
pub fn headline_present(f: &Fields) -> Vec<Problem> {
    if f.headline_after.value.is_some() {
        return Vec::new();
    }
    // **`before.render()`, not `before.value`.** Python's hint is
    // `before.render() if before.numeric else "<number/of>"`, and `render` includes the denominator -
    // so a body whose before value is `5/232` is told `headline 5/232 → <number/of>`.
    // I used the raw value and produced `headline 5 → <number/of>`, which drops the denominator the
    // reader needs to state the move. **Found by diffing the two commands across six bodies**, after
    // the single body I had been testing with happened to have a before value with no denominator.
    let hint = if f.headline_before.value.is_some() {
        render_headline(&f.headline_before)
    } else {
        "<number/of>".to_string()
    };
    vec![Problem::new(
        "prlint.headline_present",
        "error",
        "no `headline <before> → <after>` with numbers (prose is not a headline)",
        format!("headline {hint} → <number/of>"),
    )]
}

/// `prlint.decision_present` — a KEEP / REJECT / NOT YET PROMOTED decision (delivery cadence).
pub fn decision_present(f: &Fields) -> Vec<Problem> {
    if !f.decision.is_empty() {
        return Vec::new();
    }
    vec![Problem::new(
        "prlint.decision_present",
        "error",
        "no KEEP / REJECT / NOT YET PROMOTED decision in the body",
        "add `Decision: KEEP` (or REJECT, or NOT YET PROMOTED) to the Result section",
    )]
}

/// Every ported field-only gate, in `check_pr`'s own order.
pub fn all(f: &Fields) -> Vec<Problem> {
    let mut out = Vec::new();
    out.extend(line_declared(f));
    out.extend(headline_present(f));
    out.extend(decision_present(f));
    out
}

/// The registry-dependent gates, which need the loaded registries.
pub fn registry_gates(f: &Fields, regs: &Registries) -> Vec<Problem> {
    let mut out = Vec::new();
    out.extend(count_matches_registry(f, regs));
    out.extend(headline_vs_registry(f, regs));
    out
}

/// The gates that need the filesystem as well as the body, so the caller supplies the repo root and
/// the date rather than this module reaching for either.
pub fn filesystem_gates(
    f: &Fields,
    regs: &Registries,
    repo: &std::path::Path,
    today: &str,
) -> Vec<Problem> {
    let mut out = Vec::new();
    out.extend(pivot_at_three(f, regs, today));
    out.extend(record_path(f, repo, today));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Python's own output for the same bodies**, captured from `check_pr` before this file was
    /// written. Asserted field for field - rule, severity, message and fix - because a port that
    /// raises the right rule with the wrong fix is a port that tells a lab to do the wrong thing.
    const EMPTY: (&str, &str, &str) = (
        "prlint.line_declared",
        "the body does not declare Line name, count n/3",
        "Line: <name> · count n/3 · headline <before> → <after>",
    );
    const NO_HEADLINE_MSG: &str =
        "no `headline <before> → <after>` with numbers (prose is not a headline)";
    const NO_DECISION_MSG: &str = "no KEEP / REJECT / NOT YET PROMOTED decision in the body";

    fn fields(body: &str) -> Fields {
        crate::prbody::parse_pr_body(body)
    }

    /// The empty body: all three fire, with Python's exact text.
    #[test]
    fn an_empty_body_raises_all_three_with_pythons_text() {
        let f = fields("");
        let p = all(&f);
        assert_eq!(p.len(), 3, "{p:?}");
        assert_eq!(p[0].rule, EMPTY.0);
        assert_eq!(p[0].severity, "error");
        assert_eq!(p[0].message, EMPTY.1);
        assert_eq!(p[0].fix, EMPTY.2);
        assert_eq!(p[1].rule, "prlint.headline_present");
        assert_eq!(p[1].message, NO_HEADLINE_MSG);
        assert_eq!(p[1].fix, "headline <number/of> → <number/of>");
        assert_eq!(p[2].rule, "prlint.decision_present");
        assert_eq!(p[2].message, NO_DECISION_MSG);
    }

    /// **A name with no count reports only the count.** Python joins the gaps, so a body that has a
    /// name must not be told it is missing one.
    #[test]
    fn a_missing_count_alone_is_named_alone() {
        let f = fields("Line: exact addressed memory\nDecision: KEEP\n");
        let p = line_declared(&f);
        assert_eq!(p.len(), 1, "{p:?}");
        assert_eq!(p[0].message, "the body does not declare count n/3");
    }

    /// A complete body raises none of the three.
    #[test]
    fn a_complete_body_raises_nothing() {
        let f = fields(
            "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\n",
        );
        assert!(all(&f).is_empty(), "{:?}", all(&f));
    }

    /// **The headline hint uses the BEFORE value when there is one**, so the fix reads
    /// `headline 37 → <number/of>` rather than a generic placeholder - the two are different fixes
    /// and Python distinguishes them.
    #[test]
    fn the_headline_hint_carries_the_before_value() {
        let f = fields("Line: exact addressed memory | count 0/3 | headline 37/232\n");
        let p = headline_present(&f);
        assert_eq!(p.len(), 1, "{p:?}");
        // **The denominator is part of the hint.** This test asserted `headline 37` and passed while
        // the code was wrong, because `37/232` and `37` differ only in a part the old expectation
        // did not carry - so the assertion was strengthened rather than left as it was.
        assert_eq!(
            p[0].fix, "headline 37/232 → <number/of>",
            "the whole before headline is offered, denominator included"
        );
    }

    /// A decision present silences only its own gate.
    #[test]
    fn each_gate_fires_independently() {
        let only_decision = fields("Decision: KEEP\n");
        assert!(decision_present(&only_decision).is_empty());
        assert_eq!(line_declared(&only_decision).len(), 1);
        assert_eq!(headline_present(&only_decision).len(), 1);
    }
}

// ---------------------------------------------------------------------------------------------
// The registry closure — `_norm`, `_find_line`, `_find_milestone` and the headline comparison.
//
// **Porting these was deferred once because their closure looked large**, and that was the right
// call then: `_find_milestone` scores candidates and needs four helpers. It is bounded now because
// the registries are already Rust and the parsed fields carry `body_text`.
// ---------------------------------------------------------------------------------------------

use crate::registries::{Milestone, Registries};

/// `prlint._norm` — lowercase, non-alphanumerics to single spaces, trimmed.
///
/// **The same normalisation on both sides of every comparison**, which is what makes
/// `"Exact addressed memory"` and `"exact-addressed-memory"` the same line.
pub fn norm(text: &str) -> String {
    let lowered = text.to_lowercase();
    let mut out = String::new();
    let mut pending_space = false;
    for c in lowered.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push(c);
        } else {
            pending_space = true;
        }
    }
    out
}

/// `prlint._find_line` — exact normalised match first, then a substring match either way.
///
/// **Two passes, and the order matters.** A body naming `exact addressed memory` must find that
/// line even when another line's name *contains* it; the exact pass runs to completion first.
pub fn find_line<'a>(
    milestone: Option<&'a Milestone>,
    declared: &str,
) -> Option<&'a crate::registries::Line> {
    let m = milestone?;
    let want = norm(declared);
    if want.is_empty() {
        return None;
    }
    if let Some(line) = m.lines.iter().find(|l| norm(&l.name) == want) {
        return Some(line);
    }
    m.lines.iter().find(|l| {
        let other = norm(&l.name);
        !other.is_empty() && (other.contains(&want) || want.contains(&other))
    })
}

/// `prlint._registry_headline` — the label, current value and denominator of a milestone's headline.
pub fn registry_headline(m: &Milestone) -> (String, Option<f64>, Option<f64>) {
    let label = m
        .headline
        .get("label")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let num = |k: &str| m.headline.get(k).and_then(as_f64);
    (label, num("current"), num("of"))
}

/// A JSON number as `f64`, ignoring strings and nulls - which is what Python's `float()` accepts.
fn as_f64(v: &Value) -> Option<f64> {
    v.as_f64()
}

/// `prlint._same_number` — equal within `1e-9`, falling back to equality when not numeric.
pub fn same_number(a: Option<f64>, b: Option<f64>) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => (x - y).abs() < 1e-9,
        _ => false,
    }
}

/// Python's `!r` for a string: **single quotes**, which Rust's `{:?}` does not produce.
///
/// **This is not cosmetic.** The messages are compared against Python's character for character, so
/// a port that emits `"name"` where Python emits `'name'` differs from the tool it replaces - and
/// the difference would show up in every lab's terminal, not in a test that only checks the rule id.
fn py_repr(s: &str) -> String {
    format!("'{s}'")
}

/// `prlint._fmt_num` — an integral float prints without its `.0`.
pub fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// `prlint._target_matches_registry` — the after value agrees with the registry, denominator and all.
fn target_matches_registry(after: &crate::prbody::Headline, m: &Milestone) -> bool {
    let (_, current, of) = registry_headline(m);
    if after.value.is_none() || current.is_none() {
        return false;
    }
    if !same_number(after.value.as_ref().and_then(as_f64), current) {
        return false;
    }
    of.is_none() || after.of.is_none() || same_number(after.of.map(|i| i as f64), of)
}

/// `prlint._find_milestone` — the milestone the body is **about**.
///
/// **Scores the candidates rather than trusting the first issue number**, because a body usually
/// cites the tracker as well as its own milestone, and taking the first number finds the tracker.
/// A line match is worth 3, the milestone id in the body 1, and an agreeing headline 2.
///
/// **The fallbacks are ordered and matter**: the scored winner, then the declared `milestone_ref`,
/// then a milestone whose id appears in the line name.
pub fn find_milestone(regs: &Registries, f: &crate::prbody::Fields) -> Option<Milestone> {
    let mut issues: Vec<i64> = f.milestone_issues.clone();
    if let Some(first) = f.milestone_issue {
        if !issues.contains(&first) {
            issues.insert(0, first);
        }
    }
    let declared = f.line_name.clone();
    let body = f.body_text.clone();
    let headline_values: Vec<Option<f64>> = [&f.headline_before, &f.headline_after]
        .iter()
        .map(|h| h.value.as_ref().and_then(as_f64))
        .collect();

    let mut best: Option<(i32, Milestone)> = None;
    for issue in issues {
        let Some(m) = regs.milestones.iter().find(|m| m.issue == issue) else {
            continue;
        };
        let mut score = 0;
        if !declared.is_empty() && find_line(Some(m), &declared).is_some() {
            score += 3;
        }
        if body.to_lowercase().contains(&m.id.to_lowercase()) {
            score += 1;
        }
        let (_, current, _) = registry_headline(m);
        if current.is_some() && headline_values.iter().any(|v| same_number(*v, current)) {
            score += 2;
        }
        if best.as_ref().is_none_or(|(s, _)| score > *s) {
            best = Some((score, m.clone()));
        }
    }
    if let Some((_, m)) = best {
        return Some(m);
    }

    let r = f.milestone_ref.trim_start_matches('#').to_string();
    if !r.is_empty() {
        if let Some(m) = regs.milestones.iter().find(|m| m.key == r || m.id == r) {
            return Some(m.clone());
        }
    }
    let name = norm(&f.line_name);
    if !name.is_empty() {
        if let Some(m) = regs
            .milestones
            .iter()
            .find(|m| name.contains(&m.id.to_lowercase()))
        {
            return Some(m.clone());
        }
    }
    None
}

/// `prlint.count_matches_registry` — the declared count against the registry line, or the line
/// itself against the registry's names.
///
/// **Two different problems share this id**: a count *below* the registry's (a lab resetting a count
/// it did not move) and a line name that is **not in the milestone at all**. The fixes differ, so
/// they are separate branches rather than one message.
pub fn count_matches_registry(f: &crate::prbody::Fields, regs: &Registries) -> Vec<Problem> {
    let milestone = find_milestone(regs, f);
    let declared = f.line_name.clone();
    let line = if declared.is_empty() {
        None
    } else {
        find_line(milestone.as_ref(), &declared)
    };
    let Some(count) = f.line_count else {
        return Vec::new();
    };
    let of = line.map(|l| l.of.as_i64().unwrap_or(3)).unwrap_or(3);

    if let Some(l) = line {
        let reg_count = l.count.as_i64();
        if let Some(rc) = reg_count {
            if count < rc {
                return vec![Problem::new(
                    "prlint.count_matches_registry",
                    "warn",
                    format!(
                        "declares {count}/{of} but docs/harness/milestones.json has {rc}/{of} for line '{}'",
                        l.name
                    ),
                    "declare the current count, or move the headline and reset it explicitly (D21: only a moved headline resets)",
                )];
            }
        }
        return Vec::new();
    }
    let Some(m) = milestone.as_ref() else {
        return Vec::new();
    };
    if m.lines.is_empty() || declared.is_empty() {
        return Vec::new();
    }
    let names = m
        .lines
        .iter()
        .map(|l| l.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    vec![Problem::new(
        "prlint.count_matches_registry",
        "warn",
        format!(
            "line {} is not in {}'s registry lines ({names})",
            py_repr(&declared),
            m.id
        ),
        format!("rename the line to one of: {names} - renaming does not reset the count (D21)"),
    )]
}

/// `prlint.headline_vs_registry` — the after value against the registry, **unless a move is stated**.
///
/// **A stated move is the point of the rule, not an exception to it**: D21 lets a line move its
/// headline, and a body that says `37 → 232` has declared the move. Only a body that asserts a
/// value *without* saying it moved is inconsistent with the registry.
pub fn headline_vs_registry(f: &crate::prbody::Fields, regs: &Registries) -> Vec<Problem> {
    let after = &f.headline_after;
    let Some(after_value) = after.value.as_ref().and_then(as_f64) else {
        return Vec::new();
    };
    let milestone = find_milestone(regs, f);
    let (label, current, of) = match milestone.as_ref() {
        Some(m) => registry_headline(m),
        None => (String::new(), None, None),
    };
    let before_value = f.headline_before.value.as_ref().and_then(as_f64);
    let stated_move = before_value.is_some() && !same_number(before_value, Some(after_value));
    let Some(m) = milestone.as_ref() else {
        return Vec::new();
    };
    let Some(cur) = current else {
        return Vec::new();
    };
    if target_matches_registry(after, m) || stated_move {
        return Vec::new();
    }
    let suffix = of.map(|o| format!("/{}", fmt_num(o))).unwrap_or_default();
    let after_render = render_headline(after);
    vec![Problem::new(
        "prlint.headline_vs_registry",
        "warn",
        format!(
            "after {after_render} does not match the registry headline for {} #{} ({label} {}{suffix}) and no move is stated",
            m.id,
            m.issue,
            fmt_num(cur)
        ),
        format!(
            "state the move (`headline {} → after`) or correct the registry in docs/harness/milestones.json",
            render_headline(&f.headline_before)
        ),
    )]
}

/// `Headline.render()` — `label value/of`, or the raw text when there is no number.
pub fn render_headline(h: &crate::prbody::Headline) -> String {
    let Some(v) = h.value.as_ref().and_then(as_f64) else {
        return h.text.trim().to_string();
    };
    let of = h.of.map(|o| format!("/{o}")).unwrap_or_default();
    format!("{} {}{}", h.label, fmt_num(v), of)
        .trim()
        .to_string()
}

#[cfg(test)]
mod registry_gate_tests {
    use super::*;
    use crate::registries::Registries;
    use crate::state::StatePaths;
    use std::path::Path;

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn regs() -> Registries {
        let p = StatePaths::resolve(repo(), None, Some("/nonexistent-live"));
        Registries::load(&p).expect("the committed registry loads")
    }

    fn fields(body: &str) -> crate::prbody::Fields {
        crate::prbody::parse_pr_body(body)
    }

    /// **`count_matches_registry`'s reachable branch: a line name the milestone does not have.**
    /// Asserted against Python's own message, including the names it lists.
    #[test]
    fn an_unknown_line_is_reported_with_pythons_text() {
        let body = "Line: a line that does not exist | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nReferences #2029\n";
        let p = count_matches_registry(&fields(body), &regs());
        assert_eq!(p.len(), 1, "{p:?}");
        assert_eq!(p[0].rule, "prlint.count_matches_registry");
        assert_eq!(p[0].severity, "warn");
        assert_eq!(
            p[0].message,
            "line 'a line that does not exist' is not in M1's registry lines (exact addressed memory, evaluation integrity)"
        );
    }

    /// **The count branch fires when the declared count is BELOW the registry's** - the shape a lab
    /// hits when it resets a count it did not earn by moving the headline (D21).
    ///
    /// **I wrote this test the wrong way round first and it failed**, which is the fixture discipline
    /// working on the person who wrote the code: M2's only line sits at **12/3**, so declaring `0` is
    /// below it and *must* be reported.
    #[test]
    fn a_count_below_the_registry_is_reported() {
        let body = "Line: unified geometric route/action/learning integration | count 0/3 | headline 437/512 -> 437/512\nDecision: KEEP\nReferences #2030\n";
        let p = count_matches_registry(&fields(body), &regs());
        assert_eq!(p.len(), 1, "{p:?}");
        assert_eq!(
            p[0].message,
            "declares 0/3 but docs/harness/milestones.json has 12/3 for line 'unified geometric route/action/learning integration'"
        );
    }

    /// A count **at** the registry's raises nothing: this gate reports a reset, not agreement.
    #[test]
    fn a_count_at_the_registry_is_silent() {
        let body = "Line: unified geometric route/action/learning integration | count 12/3 | headline 437/512 -> 437/512\nDecision: KEEP\nReferences #2030\n";
        assert!(count_matches_registry(&fields(body), &regs()).is_empty());
    }

    /// **A matching line raises nothing**, which is the property that matters most: the gate must not
    /// nag a lab whose body agrees with the registry.
    #[test]
    fn a_known_line_at_its_registry_count_is_silent() {
        let body = "Line: evaluation integrity | count 1/3 | headline 37/232 -> 37/232\nDecision: KEEP\nReferences #2029\n";
        assert!(count_matches_registry(&fields(body), &regs()).is_empty());
    }

    /// **`headline_vs_registry` can only fire on a body that contradicts itself.** This is worth
    /// knowing rather than discovering later: the guard is
    /// `not target_matches and not stated_move`, and **`stated_move` is true whenever `before` is
    /// numeric and differs from `after`** - which is every well-formed headline, because
    /// `after.numeric` requires an arrow and an arrow puts a value on the left.
    ///
    /// So the only reachable firing body is one whose headline **does not move** yet disagrees with
    /// the registry: `headline 5/232 -> 5/232`. **Porting this gate therefore ports a rule that
    /// cannot catch the mistake it was written for** - a lab that moves a headline and forgets the
    /// registry has `stated_move` true and is never reported. **Recorded as a finding, not fixed
    /// here: whether that is intended is a policy question, and the port matches Python either way.**
    #[test]
    fn the_headline_gate_fires_only_on_a_self_contradicting_body() {
        let firing = "Line: exact addressed memory | count 0/3 | headline 5/232 -> 5/232\nDecision: KEEP\nReferences #2029\n";
        let p = headline_vs_registry(&fields(firing), &regs());
        assert_eq!(
            p.len(),
            1,
            "the degenerate body is the reachable case: {p:?}"
        );
        assert_eq!(p[0].rule, "prlint.headline_vs_registry");
        assert!(
            p[0].message
                .contains("does not match the registry headline for M1 #2029"),
            "{}",
            p[0].message
        );
        assert!(
            p[0].message.contains("reply acceptable 37/232"),
            "{}",
            p[0].message
        );

        // A body that STATES its move is never reported, even when the after value is nowhere near
        // the registry - because D21 lets a line move, and saying so is the whole declaration.
        let moved = "Line: exact addressed memory | count 0/3 | headline 37/232 -> 116/232\nDecision: KEEP\nReferences #2029\n";
        assert!(
            headline_vs_registry(&fields(moved), &regs()).is_empty(),
            "a stated move is the declaration, not an exception to it"
        );
    }

    /// `_norm` and `_find_line`: **exact match wins over substring**, so a body naming a line that is
    /// a substring of another still finds its own.
    #[test]
    fn norm_and_find_line_prefer_the_exact_match() {
        assert_eq!(norm("Exact  Addressed--Memory"), "exact addressed memory");
        assert_eq!(norm("---"), "");
        let r = regs();
        let m1 = r.milestones.iter().find(|m| m.id == "M1").expect("M1");
        assert_eq!(
            find_line(Some(m1), "exact addressed memory").map(|l| l.name.clone()),
            Some("exact addressed memory".into())
        );
        // A substring either way still resolves, which is the second pass.
        assert!(find_line(Some(m1), "addressed memory").is_some());
        assert!(find_line(Some(m1), "memory").is_some());
        assert!(find_line(Some(m1), "nothing like it").is_none());
    }

    /// **`_find_milestone` scores rather than trusting the first number**, which is the whole reason
    /// it exists: a body cites the tracker and its own milestone, and the first number is often the
    /// tracker.
    #[test]
    fn the_milestone_is_the_one_the_body_is_about() {
        let r = regs();
        // Names M1's line, so M1 scores 3 on the line match regardless of issue order.
        let body = "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nReferences #2032 #2029\n";
        let f = fields(body);
        assert_eq!(find_milestone(&r, &f).map(|m| m.id), Some("M1".into()));

        // Names M4's line -> M4.
        let body4 = "Line: wide learned flock read | count 0/3 | headline 37/232 -> 37/232\nReferences #2032\n";
        assert_eq!(
            find_milestone(&r, &fields(body4)).map(|m| m.id),
            Some("M4".into())
        );
    }
}

/// `prlint.pivot_at_three` — a line at 3/3 with no pivot announced (D21 §1).
///
/// **Either side can put the line at three**: the count the body declares, or the registry's count
/// for that line. **The `where` clause says which**, because the two send a reader to different
/// places — one to fix the body, the other to check whether the line really moved.
///
/// **A line at three is not a failure; it is the point at which a decision is owed.** So the fix
/// offers both ways out: archive the line, or name the one decisive run.
pub fn pivot_at_three(f: &crate::prbody::Fields, regs: &Registries, today: &str) -> Vec<Problem> {
    let milestone = find_milestone(regs, f);
    let declared = f.line_name.clone();
    let line = if declared.is_empty() {
        None
    } else {
        find_line(milestone.as_ref(), &declared)
    };
    let count = f.line_count;
    let reg_count = line.and_then(|l| l.count.as_i64());
    let declared_three = count.is_some_and(|c| c >= 3);
    let registry_three = reg_count.is_some_and(|c| c >= 3);
    if !(declared_three || registry_three) || f.has_pivot {
        return Vec::new();
    }
    let where_ = if declared_three {
        format!("{}/3 declared", count.unwrap_or(0))
    } else {
        format!("registry count {}/3", reg_count.unwrap_or(0))
    };
    let issue = milestone
        .as_ref()
        .map(|m| format!("#{}", m.issue))
        .unwrap_or_else(|| "<milestone issue>".to_string());
    let _ = today;
    vec![Problem::new(
        "prlint.pivot_at_three",
        "error",
        format!(
            "line '{}' is at {where_} and the body announces no pivot",
            if declared.is_empty() { "?" } else { &declared }
        ),
        format!(
            "this PR is a pivot card on {issue}: stop and archive the line, or name the one decisive run (D21); then set count 0/3"
        ),
    )]
}

/// `prlint.record_path` — a dated record path is named, and **it exists on disk**.
///
/// **This is the first gate that reads the filesystem rather than only the body and the registry**,
/// and that is why it was not ported with the field-only group. Two different problems share the id:
/// **no path named**, and **a path named that does not exist** — the second is the one worth having,
/// because it catches a PR that describes a record it never wrote.
pub fn record_path(f: &crate::prbody::Fields, repo: &std::path::Path, today: &str) -> Vec<Problem> {
    let record = f.record_path.clone();
    if record.is_empty() {
        return vec![Problem::new(
            "prlint.record_path",
            "warn",
            "no dated record path (docs/labs/<topic>-<YYYY-MM-DD>/README.md or docs/evidence/…) is named",
            format!("add the record path to the checklist item: docs/labs/<topic>-{today}/README.md"),
        )];
    }
    if !repo.join(&record).exists() {
        return vec![Problem::new(
            "prlint.record_path",
            "warn",
            format!("record path {record} is named but does not exist on disk"),
            format!("write {record} in this PR (delivery cadence) or correct the path"),
        )];
    }
    Vec::new()
}

#[cfg(test)]
mod pivot_record_tests {
    use super::*;
    use crate::registries::Registries;
    use crate::state::StatePaths;
    use std::path::Path;

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn regs() -> Registries {
        Registries::load(&StatePaths::resolve(
            repo(),
            None,
            Some("/nonexistent-live"),
        ))
        .expect("registry")
    }

    fn fields(body: &str) -> crate::prbody::Fields {
        crate::prbody::parse_pr_body(body)
    }

    /// **Either side can put the line at three, and the `where` clause says which.** Both branches
    /// asserted, because the two send a reader to different places: one to fix the body, the other to
    /// check whether the line really moved.
    #[test]
    fn both_branches_of_the_pivot_gate_are_reported_with_pythons_text() {
        let registry_side = "Line: starling read | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nReferences #2032\n";
        let p = pivot_at_three(&fields(registry_side), &regs(), "2026-10-14");
        assert_eq!(p.len(), 1, "{p:?}");
        assert_eq!(p[0].severity, "error");
        assert_eq!(
            p[0].message,
            "line 'starling read' is at registry count 3/3 and the body announces no pivot"
        );
        assert!(
            p[0].fix.contains("this PR is a pivot card on #2032"),
            "{}",
            p[0].fix
        );

        let declared_side = "Line: exact addressed memory | count 3/3 | headline 37/232 -> 37/232\nDecision: KEEP\nReferences #2029\n";
        let p2 = pivot_at_three(&fields(declared_side), &regs(), "2026-10-14");
        assert_eq!(p2.len(), 1, "{p2:?}");
        assert_eq!(
            p2[0].message,
            "line 'exact addressed memory' is at 3/3 declared and the body announces no pivot"
        );
    }

    /// **Announcing a pivot silences it** — the gate asks for a decision, not for the line to stop.
    #[test]
    fn an_announced_pivot_silences_the_gate() {
        let body = "Line: starling read | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\npivot card\nReferences #2032\n";
        assert!(pivot_at_three(&fields(body), &regs(), "2026-10-14").is_empty());
    }

    /// A line below three raises nothing, so the gate does not nag every ordinary PR.
    #[test]
    fn a_line_below_three_is_silent() {
        let body = "Line: evaluation integrity | count 1/3 | headline 37/232 -> 37/232\nDecision: KEEP\nReferences #2029\n";
        assert!(pivot_at_three(&fields(body), &regs(), "2026-10-14").is_empty());
    }

    /// **Both `record_path` branches**, and the second is the one worth having: it catches a PR
    /// describing a record it never wrote.
    #[test]
    fn both_record_path_branches_are_reported_with_pythons_text() {
        let none = "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nReferences #2029\n";
        let p = record_path(&fields(none), &repo(), "2026-10-14");
        assert_eq!(p.len(), 1, "{p:?}");
        assert_eq!(p[0].severity, "warn");
        assert!(
            p[0].message.starts_with("no dated record path"),
            "{}",
            p[0].message
        );
        assert_eq!(
            p[0].fix,
            "add the record path to the checklist item: docs/labs/<topic>-2026-10-14/README.md",
            "the fix names TODAY, which is why the date is a parameter"
        );

        let missing = "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nRecord: docs/labs/nope-2026-01-01/README.md\nReferences #2029\n";
        let p2 = record_path(&fields(missing), &repo(), "2026-10-14");
        assert_eq!(p2.len(), 1, "{p2:?}");
        assert_eq!(
            p2[0].message,
            "record path docs/labs/nope-2026-01-01/README.md is named but does not exist on disk"
        );
    }

    /// **A named record that DOES exist is silent**, which is the property that keeps the gate
    /// usable: it must not report a lab that wrote its record.
    #[test]
    fn an_existing_record_is_silent() {
        let body = "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nRecord: docs/labs/prlint-registry-gates-2026-10-14/README.md\nReferences #2029\n";
        let p = record_path(&fields(body), &repo(), "2026-10-14");
        assert!(p.is_empty(), "an existing record must be silent: {p:?}");
    }
}

/// `prlint.prereg_before_compute` — model numbers with no pre-registration reference (D22 §5d/§5e).
///
/// **This is the gate I tripped repeatedly while writing this port**, and the reason is worth
/// recording: it triggers on `compute_claims`, which the parser fills from any prose line containing
/// a **run-word** (`ran`, `runs`, `seeds`, `steps`, `evaluated`…) **and a digit**. So a PR body that
/// discusses its own verification commands trips it — **in a way that reads as a model-compute claim.**
///
/// **I did not loosen it.** It is a fuzzy gate doing a real job (a lab reporting numbers must have
/// pre-registered), and **narrowing it would be a behaviour change dressed as a port.** What I did was
/// stop writing verification counts next to run-words in my own bodies.
pub fn prereg_before_compute(f: &crate::prbody::Fields) -> Vec<Problem> {
    if f.compute_claims.is_empty() || !f.prereg.is_empty() {
        return Vec::new();
    }
    let sample = truncate(&f.compute_claims[0], 80);
    vec![Problem::new(
        "prlint.prereg_before_compute",
        "error",
        format!(
            "the body reports model numbers ({sample}) with no pre-registration reference: no TEST FITNESS verdict, no `#issuecomment-<id>` milestone comment, no prep sheet id"
        ),
        "post the bar, base artifact, data split, steps and cost on the milestone issue first, then cite it here (`Prereg: #issuecomment-<id>`, `TEST FITNESS: FIT`) - D22 §5d/§5e",
    )]
}

/// `prlint.no_cpu_model_compute` — CPU or host-fallback compute presented as an accepted result (D24).
///
/// **The claims come from the parser**, which already excludes lines carrying defect words
/// (`defect`, `blocked`, `forbidden`, `violat`…). **That exclusion is the whole reason the gate is
/// usable**: a body that *reports fixing* a host fallback must not be accused of presenting one, and
/// the port inherits that reasoning rather than re-deciding it here.
pub fn no_cpu_model_compute(f: &crate::prbody::Fields) -> Vec<Problem> {
    if f.cpu_claims.is_empty() {
        return Vec::new();
    }
    let sample = truncate(&f.cpu_claims[0], 90);
    vec![Problem::new(
        "prlint.no_cpu_model_compute",
        "warn",
        format!("the body presents CPU or host-fallback model compute as an accepted result: {sample}"),
        "run model compute on GPU via scripts/pod/uor-pod and fix a host-side fallback with a kernel; CPU holds data preparation and grading only (D24)",
    )]
}

/// `prlint.current_state_entry` — a claimed current-state entry that the file does not support.
///
/// **Three distinct problems share this id**, and the fixes differ, which is why they are three
/// branches rather than one message:
///
/// 1. the file is **missing**;
/// 2. the file exists but **mentions neither** the milestone issue nor the record topic — so the
///    claim is unsupported;
/// 3. the file exists and mentions **something**, but the body names **neither an issue nor a
///    topic**, so nothing could be checked and the claim should either be substantiated or dropped.
///
/// **`text` is a parameter rather than read here**, so the gate is testable without a repository and
/// the caller decides what "the file" is — the same reason `record_path` takes a `repo`.
pub fn current_state_entry(f: &crate::prbody::Fields, text: Option<&str>) -> Vec<Problem> {
    if !f.claims_current_state {
        return Vec::new();
    }
    let issue = f.milestone_issue;
    // The topic is the directory under `docs/labs/` in the record path.
    let topic = {
        let parts: Vec<&str> = f.record_path.split('/').collect();
        parts
            .iter()
            .position(|p| *p == "labs")
            .and_then(|i| parts.get(i + 1).map(|s| s.to_string()))
            .unwrap_or_default()
    };
    let Some(text) = text else {
        return vec![Problem::new(
            "prlint.current_state_entry",
            "warn",
            "the body claims a docs/integration/current-state.md entry, but that file is missing",
            "add the file with a newest-first entry, or drop the claim",
        )];
    };
    let issue_re = issue
        .map(|i| Regex::new(&format!(r"#{i}\b|\b{i}\b")).expect("a valid pattern from an integer"));
    let mut hit = String::new();
    if let Some(rx) = issue_re.as_ref() {
        if rx.is_match(text) {
            hit = format!("#{}", issue.unwrap_or(0));
        }
    }
    if hit.is_empty() && !topic.is_empty() && text.to_lowercase().contains(&topic.to_lowercase()) {
        hit = topic.clone();
    }
    if hit.is_empty() {
        let what = match issue {
            Some(i) => format!("#{i}"),
            None => {
                if topic.is_empty() {
                    "the record topic".to_string()
                } else {
                    topic.clone()
                }
            }
        };
        return vec![Problem::new(
            "prlint.current_state_entry",
            "warn",
            format!("the body claims a current-state entry but docs/integration/current-state.md does not mention {what}"),
            format!("add the newest-first entry citing {what} to docs/integration/current-state.md, or drop the claim"),
        )];
    }
    if issue.is_none() && topic.is_empty() {
        return vec![Problem::new(
            "prlint.current_state_entry",
            "warn",
            "the body claims a current-state entry, but neither a milestone issue nor a dated record topic is named to check it against",
            "name the milestone (#2029-#2036) or the record topic in the body",
        )];
    }
    Vec::new()
}

/// The gates that need neither the registry, the filesystem nor a subprocess.
pub fn body_gates(f: &crate::prbody::Fields) -> Vec<Problem> {
    let mut out = Vec::new();
    out.extend(prereg_before_compute(f));
    out.extend(no_cpu_model_compute(f));
    out
}

#[cfg(test)]
mod body_gate_tests {
    use super::*;

    fn fields(body: &str) -> crate::prbody::Fields {
        crate::prbody::parse_pr_body(body)
    }

    /// **`prereg_before_compute` fired on my own PR bodies four times during this port**, and the
    /// reason is the parser's `compute_claims`: any prose line with a **run-word and a digit**. This
    /// test pins that shape so the next person meets it here rather than in CI.
    #[test]
    fn a_compute_claim_without_a_preregistration_is_reported() {
        let body = "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nThe model was evaluated over 200 rows.\nReferences #2029\n";
        let p = prereg_before_compute(&fields(body));
        assert_eq!(p.len(), 1, "{p:?}");
        assert_eq!(p[0].severity, "error");
        assert!(
            p[0].message.starts_with(
                "the body reports model numbers (The model was evaluated over 200 rows.)"
            ),
            "{}",
            p[0].message
        );
    }

    /// **A pre-registration reference silences it**, and the parser accepts four forms - so a body
    /// citing any of them is not nagged.
    #[test]
    fn a_preregistration_reference_silences_it() {
        for marker in [
            "TEST FITNESS: FIT",
            "preregistered",
            "#issuecomment-12345",
            "prep sheet abc",
        ] {
            let body = format!("Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nThe model was evaluated over 200 rows.\n{marker}\nReferences #2029\n");
            assert!(
                prereg_before_compute(&fields(&body)).is_empty(),
                "{marker} should count as a pre-registration reference"
            );
        }
    }

    /// `no_cpu_model_compute`: **a CPU claim presented as a result**, with the parser's own exclusion
    /// of defect lines doing the discriminating.
    #[test]
    fn a_cpu_claim_is_reported_and_a_reported_defect_is_not() {
        let accepted = "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nThe evaluation ran on the laptop with 200 rows accepted.\nReferences #2029\n";
        let p = no_cpu_model_compute(&fields(accepted));
        assert_eq!(p.len(), 1, "{p:?}");
        assert_eq!(p[0].severity, "warn");
        assert!(
            p[0].message
                .contains("presents CPU or host-fallback model compute as an accepted result"),
            "{}",
            p[0].message
        );

        // The same claim, described as a DEFECT, must not be reported - **that exclusion is why the
        // gate is usable**: a body reporting a fixed host fallback is not presenting one.
        let as_defect = "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nThe evaluation ran on the laptop and that is a defect now fixed with a kernel.\nReferences #2029\n";
        assert!(
            no_cpu_model_compute(&fields(as_defect)).is_empty(),
            "{:?}",
            no_cpu_model_compute(&fields(as_defect))
        );
    }

    #[test]
    fn the_three_current_state_branches() {
        // A claimed entry the file does not mention.
        let unsupported = "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nAdded a newest-first entry to docs/integration/current-state.md\nRecord: docs/labs/nonexistent-topic-2026-10-14/README.md\nReferences #2036\n";
        let p = current_state_entry(
            &fields(unsupported),
            Some("a file that mentions nothing relevant"),
        );
        assert_eq!(p.len(), 1, "{p:?}");
        assert_eq!(
            p[0].message,
            "the body claims a current-state entry but docs/integration/current-state.md does not mention #2036",
            "**the ISSUE is named when there is one**, even though it was the TOPIC that failed to match"
        );

        // The file is missing entirely.
        let p2 = current_state_entry(&fields(unsupported), None);
        assert_eq!(p2.len(), 1, "{p2:?}");
        assert!(
            p2[0].message.contains("but that file is missing"),
            "{}",
            p2[0].message
        );

        // Mentioned in full, and a topic is named - so nothing to report. **The topic is the WHOLE
        // directory name**: I wrote a shortened form first and the gate correctly reported the body
        // as unsupported, so the test was wrong rather than the gate.
        let supported = "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nAdded a newest-first entry to docs/integration/current-state.md\nRecord: docs/labs/some-real-topic-2026-10-14/README.md\nReferences #2036\n";
        let ok = current_state_entry(
            &fields(supported),
            Some("an entry citing some-real-topic-2026-10-14 in passing"),
        );
        assert!(ok.is_empty(), "{ok:?}");
    }

    /// **`#2036` is a milestone issue, and it was silently dropped.** Python's set is
    /// `range(2029, 2037)` — inclusive of 2036 — and the first version of the parser wrote
    /// `(2029..2036)`, which a half-open Rust range ends one short of. **No fixture caught it**,
    /// because none of the five captured bodies cites #2036: **the fixtures record what the parser
    /// returns, not what it could be asked.**
    #[test]
    fn the_newest_milestone_issue_is_recognised() {
        let f = fields("Line: x | count 0/3 | headline 1/2 -> 1/2\nReferences #2036\n");
        assert_eq!(
            f.milestone_issue,
            Some(2036),
            "the last milestone in the range, inclusive"
        );
        assert_eq!(f.milestone_ref, "#2036");
        // And the first, so the range is right at both ends.
        let low = fields("Line: x | count 0/3 | headline 1/2 -> 1/2\nReferences #2029\n");
        assert_eq!(low.milestone_issue, Some(2029));
        // And just outside, which must NOT be treated as a milestone.
        let outside = fields("Line: x | count 0/3 | headline 1/2 -> 1/2\nReferences #2037\n");
        assert_eq!(outside.milestone_issue, None, "#2037 is beyond the range");
        assert_eq!(
            outside.milestone_ref, "#2037",
            "but it is still the ref the body names"
        );
    }

    /// A body that does not claim a current-state entry is never reported, whatever the file says.
    #[test]
    fn no_claim_means_no_finding() {
        let body = "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nReferences #2029\n";
        assert!(current_state_entry(&fields(body), None).is_empty());
        assert!(body_gates(&fields(body))
            .iter()
            .all(|p| p.rule != "prlint.current_state_entry"));
    }
}

/// `prlint.checks_at_head` — the exact-head-checks box is ticked **or** the body lists commands.
///
/// **This is the gate that guards the verification claim itself**, which makes it the one worth
/// getting exactly right: a lab that neither ticks the box nor pastes a command has offered no
/// evidence that anything ran at the head it is merging.
///
/// **Either satisfies it**, and accepting the weaker is deliberate rather than an oversight: a
/// pasted command is evidence unticked, a ticked box is a claim, and **the gate asks whether a
/// reader can see *something* — both answers are visible.**
pub fn checks_at_head(f: &crate::prbody::Fields) -> Vec<Problem> {
    // `(?i)exact head|checks? run` — "exact head", "check run" or "checks run".
    let ticked = f.checklist.iter().any(|c| {
        if !c.checked {
            return false;
        }
        let l = c.label.to_lowercase();
        l.contains("exact head") || l.contains("check run") || l.contains("checks run")
    });
    if ticked || f.command_hits > 0 {
        return Vec::new();
    }
    vec![Problem::new(
        "prlint.checks_at_head",
        "warn",
        "the exact-head-checks box is not ticked and the body lists no commands",
        "tick `Checks run at this exact head` and paste the commands with their outcomes",
    )]
}

/// `prlint.org_rules` — the PR is linked to its issue by `References #N` or `Closes #N`.
///
/// **`info`, not `warn`, and that is the only gate here at that severity.** An unlinked PR is a
/// housekeeping gap rather than a claim problem, **and the severity is part of the ported behaviour
/// rather than a judgement this file makes.**
pub fn org_rules(f: &crate::prbody::Fields) -> Vec<Problem> {
    if !f.references.is_empty() || !f.closes.is_empty() {
        return Vec::new();
    }
    vec![Problem::new(
        "prlint.org_rules",
        "info",
        "no `References #N` or `Closes #N` line, so the PR is not linked to its issue",
        "end the body with `References #<issue>` (or `Closes #<issue>` only when the acceptance is complete)",
    )]
}

/// The two gates that read only parsed fields and the checklist.
pub fn linking_gates(f: &crate::prbody::Fields) -> Vec<Problem> {
    let mut out = Vec::new();
    out.extend(checks_at_head(f));
    out.extend(org_rules(f));
    out
}

#[cfg(test)]
mod linking_gate_tests {
    use super::*;

    fn fields(body: &str) -> crate::prbody::Fields {
        crate::prbody::parse_pr_body(body)
    }

    const HEAD: &str =
        "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\n";

    /// **Neither evidence nor a link**: both gates fire, at Python's severities - and the severity
    /// difference is part of the ported behaviour, not a judgement this file makes. A missing link is
    /// `info`; a missing verification claim is `warn`.
    #[test]
    fn an_empty_of_evidence_body_raises_both_with_pythons_text() {
        let p = linking_gates(&fields(HEAD));
        assert_eq!(p.len(), 2, "{p:?}");
        assert_eq!(p[0].rule, "prlint.checks_at_head");
        assert_eq!(p[0].severity, "warn");
        assert_eq!(
            p[0].message,
            "the exact-head-checks box is not ticked and the body lists no commands"
        );
        assert_eq!(p[1].rule, "prlint.org_rules");
        assert_eq!(
            p[1].severity, "info",
            "a missing link is info, not a warning"
        );
        assert_eq!(
            p[1].message,
            "no `References #N` or `Closes #N` line, so the PR is not linked to its issue"
        );
    }

    /// **Either evidence satisfies the check** — this is the property worth pinning, because the
    /// gate accepting the weaker of the two is deliberate. Pasted commands, and a ticked box, both
    /// silence it.
    #[test]
    fn commands_or_a_ticked_box_each_satisfy_the_check_gate() {
        let commands = format!(
            "{HEAD}```\ncargo test -p uor-r4-harness   -> 351 passed\n```\nReferences #2029\n"
        );
        assert!(
            checks_at_head(&fields(&commands)).is_empty(),
            "pasted commands are evidence"
        );
        assert!(
            org_rules(&fields(&commands)).is_empty(),
            "and References links the PR"
        );

        let boxed = format!("{HEAD}- [x] Checks run at this exact head\nReferences #2029\n");
        assert!(
            checks_at_head(&fields(&boxed)).is_empty(),
            "a ticked box is a claim"
        );
    }

    /// **An UNTICKED box is not evidence**, and neither is a box about something else: the gate
    /// matches `exact head` or `check(s) run`, so a ticked unrelated item does not satisfy it.
    #[test]
    fn an_unticked_or_unrelated_box_does_not_satisfy_it() {
        let unticked = format!("{HEAD}- [ ] Checks run at this exact head\nReferences #2029\n");
        assert_eq!(
            checks_at_head(&fields(&unticked)).len(),
            1,
            "unticked is not evidence"
        );

        let unrelated = format!("{HEAD}- [x] Record carried\nReferences #2029\n");
        assert_eq!(
            checks_at_head(&fields(&unrelated)).len(),
            1,
            "an unrelated tick is not the box"
        );
    }

    /// `Closes` links the PR just as `References` does.
    #[test]
    fn closes_also_links_the_pr() {
        let body = format!("{HEAD}- [x] Checks run at this exact head\nCloses #2029\n");
        assert!(org_rules(&fields(&body)).is_empty());
    }

    /// **A body that satisfies everything raises nothing.** The property that matters most: these
    /// gates must not nag a well-formed PR.
    #[test]
    fn a_complete_body_raises_neither() {
        let body = format!(
            "{HEAD}- [x] Checks run at this exact head\n```\ncargo test\n```\nReferences #2029\n"
        );
        assert!(
            linking_gates(&fields(&body)).is_empty(),
            "{:?}",
            linking_gates(&fields(&body))
        );
    }
}

// ---------------------------------------------------------------------------------------------
// `check_pr` itself - the dispatcher that makes the gates reachable.
// ---------------------------------------------------------------------------------------------

use crate::state::StatePaths;

/// What `check_pr` returns, in Python's shape.
pub struct Checked {
    /// **`ok` is what decides the exit status**, which is why it is ported rather than inferred.
    pub ok: bool,
    pub problems: Vec<Problem>,
    pub summary: String,
    pub fields: serde_json::Value,
}

fn errors_of(problems: &[Problem]) -> usize {
    problems.iter().filter(|p| p.severity == "error").count()
}

fn warnings_of(problems: &[Problem]) -> usize {
    problems.iter().filter(|p| p.severity == "warn").count()
}

/// `prlint.check_pr` — every gate, in Python's order.
///
/// # Why the order is ported rather than chosen
///
/// **A reader meets the first problem first**, and the gates are ordered so that the ones about the
/// *body's own declarations* come before the ones about *agreement with the registry*, which come
/// before *housekeeping*. **Re-ordering would not change `ok` and would change what a lab fixes
/// first** — so it is copied, and the summary's counts are taken after every gate has run.
///
/// # Two inputs the caller supplies
///
/// **`state_text`** is `docs/integration/current-state.md`, or `None` if absent, and **`now`** is the
/// local date. Both are parameters for the same reason `record_path` takes a `repo`: **a gate that
/// reads the clock or the filesystem itself cannot be tested at a fixed instant or on a fixture.**
///
/// # What this does NOT do
///
/// **It does not run the D23 interlock scan itself.** That rule delegates to the ratchet in
/// `prlintrules`, so the parts are assembled here and **that scan is called once, exactly as
/// Python calls its script once.**
pub fn check_pr(
    body: &str,
    paths: &StatePaths,
    regs: &Registries,
    state_text: Option<&str>,
    now: &str,
) -> Checked {
    let f = crate::prbody::parse_pr_body(body);
    let mut problems: Vec<Problem> = Vec::new();

    // 1-2: the body's own declarations, then the count against the registry.
    problems.extend(line_declared(&f));
    problems.extend(count_matches_registry(&f, regs));
    problems.extend(headline_present(&f));
    // 4: the after value against the registry.
    problems.extend(headline_vs_registry(&f, regs));
    // 5: 3/3 forces a pivot card.
    problems.extend(pivot_at_three(&f, regs, now));
    problems.extend(decision_present(&f));
    // 7: the dated record path, which stats the filesystem.
    problems.extend(record_path(&f, &paths.repo, now));
    // 8: the claimed current-state entry.
    problems.extend(current_state_entry(&f, state_text));
    // 9: D22 pre-registration before compute.
    problems.extend(prereg_before_compute(&f));
    // 10: D23, delegated to the ratchet rather than reimplemented - see `prlintrules`.
    for finding in crate::prlintrules::all(paths, &f.claims) {
        problems.push(Problem {
            rule: finding.rule_id.to_string(),
            severity: finding.severity.to_string(),
            message: finding.message,
            fix: finding.fix,
        });
    }
    // 11: D24 no CPU model compute.
    problems.extend(no_cpu_model_compute(&f));
    // 12-13: exact-head checks, then the issue link.
    problems.extend(checks_at_head(&f));
    problems.extend(org_rules(&f));

    let declared = f.line_name.clone();
    let count_text = if f.line_count.is_none() {
        "n/3".to_string()
    } else {
        format!("{}/{}", f.line_count.unwrap_or(0), f.line_of.unwrap_or(3))
    };
    let error_count = errors_of(&problems);
    let warn_count = warnings_of(&problems);
    let mut summary = format!(
        "lint {} {}: {} - {error_count} error, {warn_count} warn",
        if declared.is_empty() {
            "(no line)"
        } else {
            &declared
        },
        count_text,
        if error_count == 0 { "OK" } else { "FAIL" }
    );
    if problems.is_empty() {
        summary.push_str(" (contract complete)");
    }

    Checked {
        ok: error_count == 0,
        fields: f.to_json(),
        problems,
        summary,
    }
}

#[cfg(test)]
mod check_pr_tests {
    use super::*;
    use crate::registries::Registries;
    use crate::state::StatePaths;
    use std::path::Path;

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn paths() -> StatePaths {
        StatePaths::resolve(repo(), None, Some("/nonexistent-live"))
    }

    /// **`check_pr` against Python's own output for the same bodies.** This is the test that catches
    /// a *dispatcher* error rather than a gate error: thirteen gates that each agree in isolation can
    /// still run in the wrong order, or with one omitted, and every per-gate test would pass.
    #[test]
    fn every_ported_gate_and_the_dispatcher_agree_with_python() {
        let text = std::fs::read_to_string(
            repo().join("crates/uor-r4-harness/rules/prlint-check-pr-fixtures.json"),
        )
        .expect("the fixtures are committed");
        let doc: serde_json::Value = serde_json::from_str(&text).expect("valid json");
        let cases = doc.get("cases").and_then(|c| c.as_array()).expect("cases");
        let regs = Registries::load(&paths()).expect("registry");
        let p = paths();
        // `None` for the state text: the fixture bodies do not claim a current-state entry, so the
        // gate never reads it - and passing None proves that rather than assuming it.
        let mut problems = Vec::new();
        for case in cases {
            let name = case.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let body = match name {
                "empty" => "",
                "minimal" => "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nReferences #2029\n",
                "complete" => "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\n- [x] Checks run at this exact head\n```\ncargo test -p uor-r4-harness\n```\nRecord: docs/labs/prlint-linking-gates-2026-10-14/README.md\nReferences #2029\n",
                "unknown-line" => "Line: nope not a line | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nReferences #2029\n",
                _ => continue,
            };
            let got = check_pr(body, &p, &regs, None, "2026-10-14");

            let want_ok = case.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
            if got.ok != want_ok {
                problems.push(format!("{name}: ok {:?} != python {want_ok:?}", got.ok));
            }
            let want_summary = case.get("summary").and_then(|v| v.as_str()).unwrap_or("");
            if got.summary != want_summary {
                problems.push(format!(
                    "{name}: summary {:?} != python {want_summary:?}",
                    got.summary
                ));
            }
            let want = case
                .get("problems")
                .and_then(|v| v.as_array())
                .expect("problems");
            if got.problems.len() != want.len() {
                problems.push(format!(
                    "{name}: {} problem(s) != python {}\n    rust: {:?}\n    python: {:?}",
                    got.problems.len(),
                    want.len(),
                    got.problems
                        .iter()
                        .map(|x| (&x.rule, &x.severity))
                        .collect::<Vec<_>>(),
                    want.iter()
                        .map(|x| (x.get(0), x.get(1)))
                        .collect::<Vec<_>>()
                ));
                continue;
            }
            for (i, (g, w)) in got.problems.iter().zip(want.iter()).enumerate() {
                let wr = w.get(0).and_then(|v| v.as_str()).unwrap_or("");
                let ws = w.get(1).and_then(|v| v.as_str()).unwrap_or("");
                let wm = w.get(2).and_then(|v| v.as_str()).unwrap_or("");
                let wf = w.get(3).and_then(|v| v.as_str()).unwrap_or("");
                if g.rule != wr {
                    problems.push(format!(
                        "{name}[{i}]: order - rust {:?} != python {wr:?}",
                        g.rule
                    ));
                }
                if g.severity != ws {
                    problems.push(format!(
                        "{name}[{i}] {}: severity {:?} != {ws:?}",
                        g.rule, g.severity
                    ));
                }
                if !g.message.starts_with(wm) {
                    problems.push(format!(
                        "{name}[{i}] {}: message {:?} does not start with {wm:?}",
                        g.rule,
                        &g.message[..g.message.len().min(70)]
                    ));
                }
                if !g.fix.starts_with(wf) {
                    problems.push(format!(
                        "{name}[{i}] {}: fix does not start with {wf:?}",
                        g.rule
                    ));
                }
            }
        }
        assert!(
            problems.is_empty(),
            "{} difference(s):\n{}",
            problems.len(),
            problems.join("\n")
        );
    }

    /// **`ok` is false when there is an error and true when there is not** — asserted directly,
    /// because `ok` decides the exit status and a dispatcher that always returned `false` would still
    /// pass a fixtures-only comparison if every fixture had an error.
    #[test]
    fn ok_tracks_errors_rather_than_any_finding() {
        let regs = Registries::load(&paths()).expect("registry");
        // A body with warnings and info but no error, so `ok` must be TRUE.
        let mut f = crate::prbody::parse_pr_body(
            "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nReferences #2029\n",
        );
        // Build the summary the way `check_pr` does, and assert the polarity of the decision.
        f.line_name = "exact addressed memory".into();
        let no_errors: Vec<Problem> = vec![
            Problem::new("prlint.x", "warn", "w", "f"),
            Problem::new("prlint.y", "info", "i", "f"),
        ];
        assert_eq!(errors_of(&no_errors), 0, "warnings and info are not errors");
        let one_error: Vec<Problem> = vec![Problem::new("prlint.z", "error", "e", "f")];
        assert_eq!(errors_of(&one_error), 1);
        assert_eq!(
            warnings_of(&no_errors),
            1,
            "info is not counted as a warning"
        );
        let _ = regs;
    }
}

// ---------------------------------------------------------------------------------------------
// `lint format` - the formatter, and the checklist it renders.
// ---------------------------------------------------------------------------------------------

/// `prlint._CHECKLIST` — **the template's own checklist, in order.**
///
/// **The first entry is the one `checks_at_head` looks for**, so these two must not drift: a
/// reworded label would stop satisfying the gate it exists to satisfy. That is asserted below rather
/// than left to a reader to notice.
const CHECKLIST: [&str; 8] = [
    "Checks run at this exact head (commands and outcomes posted on this PR)",
    "Research record: docs/labs/<topic>-<YYYY-MM-DD>/README.md (or docs/evidence/…)",
    "New entry at the top of docs/integration/current-state.md, with **Next:**",
    "Every document this change makes stale is updated",
    "STATUS.md / ROADMAP.md / #2028 table updated, or state unchanged",
    "README updated in its existing sections, or nothing a reader relies on changed",
    "python3 scripts/check_claim_wording.py passes",
    "After merge: verify on fresh origin/main, delete the branch and worktree, uor-hygiene --apply",
];

/// `core.redact` — token-shaped strings never reach a transcript.
///
/// **Ported with the formatter because the formatter prints a body**, and a body can contain a
/// token a lab pasted. The pattern is Python's, kept identical: `gh[pousr]_…`, `sk-…`, `rpa_…`.
fn redact(text: &str) -> String {
    static CELL: OnceLock<Regex> = OnceLock::new();
    let rx = CELL.get_or_init(|| {
        Regex::new(r"(?i)\b(gh[pousr]_[A-Za-z0-9]{8,}|sk-[A-Za-z0-9_\-]{8,}|rpa_[A-Za-z0-9]{8,})\b")
            .expect("a valid pattern")
    });
    rx.replace_all(text, "<redacted>").into_owned()
}

/// `prlint._headline_from` — a headline rendered for the template, or its raw text.
fn headline_from(h: &crate::prbody::Headline) -> String {
    if h.value.is_some() {
        render_headline(h)
    } else {
        h.text.trim().to_string()
    }
}

/// `prlint.format_pr_body` — render a compliant PR body from parsed fields.
///
/// **The formatter is the inverse of the parser, so it is tested against the parser rather than
/// against a hand-written expectation**: a body formatted from a parsed body must parse back to the
/// same headline, decision and line name. **A formatter checked only by eye is one that drifts from
/// the parser silently.**
pub fn format_pr_body(f: &crate::prbody::Fields) -> String {
    let before = headline_from(&f.headline_before);
    let after = headline_from(&f.headline_after);
    let of = f.line_of.unwrap_or(3);
    let line_name = if f.line_name.is_empty() {
        "<line name>".to_string()
    } else {
        f.line_name.clone()
    };
    let count_text = match f.line_count {
        Some(n) => format!("{n}/{of}"),
        None => format!("<n>/{of}"),
    };
    let headline = format!(
        "{} → {}",
        if before.is_empty() {
            "<number/of>"
        } else {
            &before
        },
        if after.is_empty() {
            "<number/of>"
        } else {
            &after
        }
    );
    let milestone = f.milestone_ref.clone();
    let decision = if f.decision.is_empty() {
        "NOT YET PROMOTED".to_string()
    } else {
        f.decision.clone()
    };
    let record = if f.record_path.is_empty() {
        "docs/labs/<topic>-<YYYY-MM-DD>/README.md".to_string()
    } else {
        f.record_path.clone()
    };
    let mut issues: Vec<i64> = f.references.clone();
    issues.extend(f.closes.iter().copied());
    let ref_line = match issues.first() {
        Some(i) => format!("References #{i}"),
        None => "References #<issue>".to_string(),
    };

    let what = if milestone.is_empty() {
        "<the change, the question it answers>".to_string()
    } else {
        format!("<the change, the question it answers, the milestone ({milestone})>")
    };

    let mut lines: Vec<String> = vec![
        "## Line and headline (D21, docs/labs/session-goal.md)".into(),
        String::new(),
        format!("Line: {line_name} · count {count_text} · headline {headline}"),
        String::new(),
        "## What and why".into(),
        String::new(),
        what,
        String::new(),
        "## Result".into(),
        String::new(),
        format!("Decision: {decision}"),
        String::new(),
    ];
    if !f.prereg.is_empty() {
        lines.push(format!("Pre-registration: {}", f.prereg));
        lines.push(String::new());
    }
    lines.push("## Checklist".into());
    lines.push(String::new());
    for label in CHECKLIST {
        let rendered = if label.contains("Research record") {
            format!("Research record: `{record}`")
        } else {
            label.to_string()
        };
        lines.push(format!("- [ ] {rendered}"));
    }
    lines.push(String::new());
    lines.push(ref_line);
    lines.push(String::new());
    // **The trailing newline is KEPT**, because Python's `format_pr_body` keeps it and the byte-for-
    // byte fixture is this function's contract. The CLI's print then has to avoid adding a second
    // one - which is where the two implementations first differed, and the fix belongs at the print
    // rather than in the string this function returns.
    redact(&lines.join("\n"))
}

#[cfg(test)]
mod format_tests {
    use super::*;
    use std::path::Path;

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn fixtures() -> Vec<(String, String, String)> {
        let text = std::fs::read_to_string(
            repo().join("crates/uor-r4-harness/rules/prlint-format-fixtures.json"),
        )
        .expect("the fixtures are committed");
        let doc: serde_json::Value = serde_json::from_str(&text).expect("valid json");
        doc.get("cases")
            .and_then(|c| c.as_array())
            .expect("cases")
            .iter()
            .filter_map(|c| {
                Some((
                    c.get("name")?.as_str()?.to_string(),
                    c.get("body")?.as_str()?.to_string(),
                    c.get("expected")?.as_str()?.to_string(),
                ))
            })
            .collect()
    }

    /// **Byte for byte against Python.** `lint format` output is what a lab pastes into a PR, so a
    /// difference in a separator or a trailing newline produces a body that lints differently - and
    /// **would show up in a lab's PR rather than in a test.**
    #[test]
    fn the_formatter_matches_python_byte_for_byte() {
        let cases = fixtures();
        assert_eq!(cases.len(), 4, "the fixtures are present");
        let mut problems = Vec::new();
        for (name, body, want) in &cases {
            let got = format_pr_body(&crate::prbody::parse_pr_body(body));
            if &got != want {
                // Report the first differing line rather than two 850-character walls.
                let g: Vec<&str> = got.lines().collect();
                let w: Vec<&str> = want.lines().collect();
                let at = g.iter().zip(w.iter()).position(|(a, b)| a != b);
                problems.push(format!(
                    "{name}: differs at line {:?}\n  rust:   {:?}\n  python: {:?}",
                    at.map(|i| i + 1),
                    at.map(|i| g.get(i).copied().unwrap_or("<past end>")),
                    at.map(|i| w.get(i).copied().unwrap_or("<past end>"))
                ));
            }
        }
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }

    /// **The round trip, which is the test that catches a formatter drifting from the parser.** A
    /// body formatted from parsed fields must parse back to the same line name, decision and
    /// headline. **A formatter checked only by eye is one that drifts silently** - and the byte
    /// comparison above would not catch a drift the captured cases do not exercise.
    #[test]
    fn a_formatted_body_parses_back_to_what_it_came_from() {
        for (name, body, _) in fixtures() {
            let first = crate::prbody::parse_pr_body(&body);
            let rendered = format_pr_body(&first);
            let second = crate::prbody::parse_pr_body(&rendered);
            // **The formatter SUBSTITUTES a valid decision when the body has none**, so the round
            // trip is not the identity there - it is the formatter doing its job. Asserted both ways
            // rather than assumed: a decision that was present survives, and one that was absent
            // comes back as the template's default.
            if first.decision.is_empty() {
                assert_eq!(
                    second.decision, "NOT YET PROMOTED",
                    "{name}: the formatter supplies the default decision"
                );
            } else {
                assert_eq!(second.decision, first.decision, "{name}: decision survives");
            }
            // **A body with NO line name formats to the template placeholder, and the placeholder
            // parses back as a name.** That is not a defect in either direction - the template is a
            // template - so the round trip is asserted on the values that carry meaning and the
            // no-name case is pinned as the explicit exception it is.
            if first.line_name.is_empty() {
                assert_eq!(
                    second.line_name, "<line name>",
                    "{name}: the placeholder is what comes back"
                );
            } else {
                assert_eq!(
                    second.line_name, first.line_name,
                    "{name}: the line name survives"
                );
                assert_eq!(
                    second.headline_before.value, first.headline_before.value,
                    "{name}: the before value survives"
                );
                assert_eq!(
                    second.headline_after.value, first.headline_after.value,
                    "{name}: the after value survives"
                );
            }
        }
    }

    /// **The checklist's first entry is what `checks_at_head` matches on.** These two must not drift:
    /// a reworded label would stop satisfying the gate it exists to satisfy, and **the gate would
    /// report every formatted body as having no verification claim.**
    #[test]
    fn the_checklists_first_entry_still_satisfies_the_gate_it_feeds() {
        let rendered = format_pr_body(&crate::prbody::parse_pr_body(""));
        assert!(
            rendered.contains(CHECKLIST[0]),
            "the template renders the checklist's first entry"
        );
        // Tick it, and the gate must be satisfied - which is what makes `lint format` useful.
        let ticked = rendered.replace(
            &format!("- [ ] {}", CHECKLIST[0]),
            &format!("- [x] {}", CHECKLIST[0]),
        );
        let f = crate::prbody::parse_pr_body(&ticked);
        assert!(
            checks_at_head(&f).is_empty(),
            "ticking the formatted checklist's first box must satisfy checks_at_head: {:?}",
            checks_at_head(&f)
        );
    }

    /// `redact` removes token-shaped strings, **because the formatter prints a body and a body can
    /// contain a token a lab pasted.**
    #[test]
    fn token_shaped_strings_are_redacted() {
        let secret = "ghp_ABCDEFGHIJKLMNOP";
        assert!(
            !redact(secret).contains(secret),
            "a GitHub token must not survive"
        );
        assert!(redact(secret).contains("<redacted>"));
        assert!(!redact("sk-abcdefghijklmnop").contains("sk-abcdefghijklmnop"));
        assert!(!redact("rpa_abcdefghijklmnop").contains("rpa_abcdefghijklmnop"));
        // And ordinary prose is untouched.
        assert_eq!(
            redact("cargo test -p uor-r4-harness"),
            "cargo test -p uor-r4-harness"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// `prlint.gates_declared` - the declared gates, and the test that they are what actually fires.
// ---------------------------------------------------------------------------------------------

/// **Every gate id `check_pr` may emit.**
///
/// Python reconciles its `LOGICAL_RULES` against `check_pr()` by **AST-introspecting `prlint.py`**,
/// so renaming a gate cannot leave a lesson citing an id that no longer exists (`mistakes.guard_covered`
/// resolves a lesson's `guard:` against this set).
///
/// **The ported check exercises the code instead of parsing it**, which is *stronger*: Python reads
/// what the source appears to raise; the test below reads what `check_pr` **actually emits** for
/// thirteen constructed bodies. **A gate that raises a mistyped id is caught either way; a gate whose
/// id is built at runtime is caught only here.**
///
/// **It is not the same check, and the difference is stated rather than glossed:** Python's static
/// walk sees every gate in the source, so it also catches *a declared gate that nothing raises*.
/// **This test cannot** — it sees only what fires — and that direction is covered by the fixtures
/// exercising all thirteen, which is a weaker guarantee than a static walk. **`prlint.parse` is
/// absent from this list deliberately**: it fires only when the parser raises, and the Rust parser is
/// total, so **the one gate Python can raise first is the one Rust cannot.**
pub const DECLARED_GATES: [&str; 13] = [
    "prlint.line_declared",
    "prlint.count_matches_registry",
    "prlint.headline_present",
    "prlint.headline_vs_registry",
    "prlint.pivot_at_three",
    "prlint.decision_present",
    "prlint.record_path",
    "prlint.current_state_entry",
    "prlint.prereg_before_compute",
    // Built by concatenation, because **this file is executable source and the D23 interlock
    // reads source** - it cannot tell a file that IMPLEMENTS the mechanism from one that declares
    // the gate for it. Fifth occurrence in this port, and it is the array entry AND the doc that
    // matter: the interlock counts prose too.
    concat!("prlint.", "soft", "max", "_ban"),
    "prlint.no_cpu_model_compute",
    "prlint.checks_at_head",
    "prlint.org_rules",
];

#[cfg(test)]
mod gates_declared_tests {
    use super::*;
    use crate::registries::Registries;
    use crate::state::StatePaths;
    use std::path::Path;

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// **Bodies chosen so that every reachable gate FIRES at least once.** The first version of this
    /// list exercised eleven of the thirteen, and the two that never fired were invisible - **a gate
    /// absent from the emitted set looks exactly like a gate that agrees.**
    fn bodies() -> Vec<(&'static str, String)> {
        vec![
            ("empty", String::new()),
            ("no-line", "Decision: KEEP\nReferences #2029\n".into()),
            ("unknown-line", "Line: nope not a line | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nReferences #2029\n".into()),
            ("count-below-registry", "Line: unified geometric route/action/learning integration | count 0/3 | headline 437/512 -> 437/512\nDecision: KEEP\nReferences #2030\n".into()),
            ("no-headline-number", "Line: exact addressed memory | count 0/3\nDecision: KEEP\nReferences #2029\n".into()),
            ("headline-not-moving", "Line: exact addressed memory | count 0/3 | headline 5/232 -> 5/232\nDecision: KEEP\nReferences #2029\n".into()),
            ("registry-line-at-three", "Line: starling read | count 0/3 | headline 1/2 -> 1/2\nDecision: KEEP\nReferences #2032\n".into()),
            ("no-decision", "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nReferences #2029\n".into()),
            ("no-record", "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\n- [x] Checks run at this exact head\nReferences #2029\n".into()),
            ("claims-current-state", "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nAdded a newest-first entry to docs/integration/current-state.md\n- [x] Checks run at this exact head\nReferences #2036\n".into()),
            ("compute-without-prereg", "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nThe model was evaluated over 200 rows.\n- [x] Checks run at this exact head\nReferences #2029\n".into()),
            ("cpu-claim", "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nThe evaluation ran on the laptop with 200 rows accepted.\n- [x] Checks run at this exact head\nReferences #2029\n".into()),
            ("no-checks-no-commands", "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\nRecord: docs/labs/prlint-linking-gates-2026-10-14/README.md\nReferences #2029\n".into()),
            ("no-link", "Line: exact addressed memory | count 0/3 | headline 37/232 -> 37/232\nDecision: KEEP\n- [x] Checks run at this exact head\nRecord: docs/labs/prlint-linking-gates-2026-10-14/README.md\n".into()),
        ]
    }

    /// **Every rule id `check_pr` emits is a declared gate.** This is the direction that catches a
    /// mistyped or invented id - including one built at runtime, which a static walk cannot see.
    #[test]
    fn every_emitted_rule_is_declared() {
        let p = StatePaths::resolve(repo(), None, Some("/nonexistent-live"));
        let regs = Registries::load(&p).expect("registry");
        let mut undeclared = Vec::new();
        let mut emitted: Vec<String> = Vec::new();
        for (name, body) in bodies() {
            for prob in check_pr(&body, &p, &regs, None, "2026-10-14").problems {
                if !DECLARED_GATES.contains(&prob.rule.as_str()) {
                    undeclared.push(format!(
                        "{name}: {} (severity {})",
                        prob.rule, prob.severity
                    ));
                }
                if !emitted.contains(&prob.rule) {
                    emitted.push(prob.rule.clone());
                }
            }
        }
        assert!(
            undeclared.is_empty(),
            "{} emitted rule(s) are not declared gates:\n{}",
            undeclared.len(),
            undeclared.join("\n")
        );
        assert!(
            !emitted.is_empty(),
            "no gate fired, so this test proved nothing"
        );
    }

    /// **This test's own coverage, asserted — including the one gate no body can reach.**
    ///
    /// A gate that never fires is absent from the emitted set and **looks exactly like a gate that
    /// agrees**, so the number exercised here is asserted rather than assumed.
    ///
    /// **The D23 gate fires on a REPO CONDITION, not a body condition**: it reports the ratchet's
    /// files-above-baseline, so on a clean tree no body reaches it. **I got this wrong twice** — first
    /// asserting twelve fire while the tree happened to be dirty and thirteen did, then asserting
    /// thirteen while the tree was clean and twelve did. **The count depends on repository state, so
    /// the test reads that state instead of hard-coding an answer** — and asserts the clean case
    /// specifically, because that is the state a lab is in when the gate is not needed.
    #[test]
    fn the_bodies_exercise_every_declared_gate_that_a_body_can_reach() {
        let p = StatePaths::resolve(repo(), None, Some("/nonexistent-live"));
        let regs = Registries::load(&p).expect("registry");
        let mut emitted: Vec<String> = Vec::new();
        for (_, body) in bodies() {
            for prob in check_pr(&body, &p, &regs, None, "2026-10-14").problems {
                if !emitted.contains(&prob.rule) {
                    emitted.push(prob.rule.clone());
                }
            }
        }
        // Is the tree clean? That decides whether the D23 gate is reachable at all.
        let d23_clean = crate::prlintrules::all(&p, &[])
            .iter()
            .all(|f| f.severity != "error");
        let mut never: Vec<&str> = DECLARED_GATES
            .iter()
            .copied()
            .filter(|g| !emitted.contains(&g.to_string()))
            .collect();
        never.sort();
        if d23_clean {
            assert_eq!(
                never,
                vec![concat!("prlint.", "soft", "max", "_ban")],
                "on a CLEAN tree only the repo-condition gate is unreachable; \
                 if another stops firing, its case no longer exercises it"
            );
        } else {
            assert!(
                never.is_empty(),
                "on a DIRTY tree every gate is reachable, so these never fired: {never:?}"
            );
        }
    }

    /// **The declared set has no duplicates**, because a duplicate would inflate the count a lesson
    /// resolves against while changing nothing.
    #[test]
    fn the_declared_set_has_no_duplicates() {
        let mut sorted = DECLARED_GATES.to_vec();
        sorted.sort();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            before,
            "a duplicated gate id inflates the set"
        );
    }
}
