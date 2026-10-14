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
