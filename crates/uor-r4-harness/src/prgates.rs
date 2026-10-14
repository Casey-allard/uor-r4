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
//! # What is deliberately NOT here
//!
//! * `record_path` **also stats the filesystem** for the named path, so it is not field-only.
//! * `current_state_entry` **reads `docs/integration/current-state.md`**, and the same for the
//!   milestone registry lookups in `count_matches_registry` and `headline_vs_registry`.
//!
//! **Each says why in this list rather than being silently absent**, because a gate that is missing
//! and a gate that is not needed look identical from the outside.

use crate::prbody::Fields;
use serde_json::{json, Value};

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
    let hint = if f.headline_before.value.is_some() {
        f.headline_before
            .value
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<number/of>".into())
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
        assert_eq!(
            p[0].fix, "headline 37 → <number/of>",
            "the before value is offered"
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
