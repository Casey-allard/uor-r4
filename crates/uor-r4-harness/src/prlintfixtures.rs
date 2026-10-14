//! The captured PR-body fixtures, and the loader the port must satisfy.
//!
//! # Why this exists before the port does
//!
//! `prlint` is **1,133 lines with 22 problem sites inside one function**, and `parse_pr_body` — the
//! shared input to every gate — is 105 lines producing 25 fields. **`parity.py`, the most valuable
//! verification in this port, dies with Python**, so behaviour captured as data is the only safety
//! net that outlives it.
//!
//! **Capture first, port second.** A port written from the docstring and then checked against the
//! docstring proves nothing; **this is what it gets checked against.**
//!
//! # What the capture exposed that reading the parser did not
//!
//! * **The raw template yields `line_count: 0` with `line_of: 3`** — **I first wrote this up as
//!   "`line_of` is set while `line_count` is not", and my own test caught that as false.** The real
//!   shape is an **asymmetric pair**: a port that reads `0/3` and reports both fields, or treats the
//!   template as declaring nothing, differs on the very case labs hit first. **The fixture corrected
//!   the prose, which is the argument for fixtures stated better than I could state it.**
//! * **`milestone_ref` is set in bodies that name no milestone.**
//! * **An empty body still yields `headline_before`/`headline_after` objects**, not nothing — so
//!   "empty means empty" is wrong too.
//!
//! **Those three are exactly what a port written from the docstring gets wrong**, and they were
//! found by running the parser rather than reading it. That is the argument for fixtures.

use serde_json::Value;

/// One captured case: the body, and what Python's parser produced for it.
pub struct Case {
    pub name: String,
    pub body: String,
    pub expected: Value,
}

/// Load the fixtures from `rules/prlint-body-fixtures.json`.
pub fn load(repo: &std::path::Path) -> Vec<Case> {
    let file = repo.join("crates/uor-r4-harness/rules/prlint-body-fixtures.json");
    let Ok(text) = std::fs::read_to_string(&file) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    v.get("cases")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|c| {
                    Some(Case {
                        name: c.get("name")?.as_str()?.to_string(),
                        body: c.get("body")?.as_str()?.to_string(),
                        expected: c.get("expected")?.clone(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Every field name the fixtures record, so a port can prove it declares all of them.
pub fn field_names(cases: &[Case]) -> Vec<String> {
    let mut out: Vec<String> = cases
        .iter()
        .filter_map(|c| c.expected.as_object())
        .flat_map(|o| o.keys().cloned())
        .collect();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod fixture_tests {
    use super::*;
    use std::path::Path;

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// **The fixtures must be there and be real.** A port checked against an empty fixture set would
    /// pass while proving nothing, which is the failure this file exists to prevent.
    #[test]
    fn the_fixtures_load_and_are_not_empty() {
        let cases = load(&repo());
        assert_eq!(cases.len(), 5, "the captured case count");
        assert!(
            cases.iter().all(|c| !c.expected.is_null()),
            "every case must carry an expectation"
        );
    }

    /// **27 fields, including `raw` and `body_text`.** Asserted so a field silently dropped from the
    /// capture is a failing test rather than a port that agrees on a smaller surface. **`body_text`
    /// is in the set because `_find_milestone` consumes it** - it looked like a parser internal until
    /// a gate needed it, which is the argument for recording what the parser RETURNS.
    #[test]
    fn the_fixtures_record_every_field_the_parser_returns() {
        let names = field_names(&load(&repo()));
        assert_eq!(
            names.len(),
            27,
            "the parser's field count, including raw and body_text: {names:?}"
        );
        for expected in [
            "line_name",
            "line_count",
            "line_of",
            "headline_before",
            "headline_after",
            "milestone_ref",
            "milestone_issue",
            "decision",
            "record_paths",
            "references",
            "closes",
            "body_text",
            "raw",
        ] {
            assert!(names.iter().any(|n| n == expected), "{expected} missing");
        }
    }

    /// **The three findings that make the fixtures worth having**, asserted so they cannot be
    /// "tidied away" as fixture noise. Each is a case a port written from the docstring would miss.
    #[test]
    fn the_fixtures_pin_the_cases_a_docstring_port_would_get_wrong() {
        let cases = load(&repo());
        let get = |name: &str| cases.iter().find(|c| c.name == name).expect("case");

        // 1. The raw template yields an ASYMMETRIC pair: `line_count: 0` with `line_of: 3`.
        //    **I wrote the opposite here first - "line_of is set while line_count is not" - and this
        //    test caught it as false.** Both fields are set; what matters is that the count is ZERO
        //    rather than absent, so a port treating 0 as undeclared differs on the template itself.
        let t = &get("template-raw").expected;
        assert_eq!(
            t.get("line_count").and_then(Value::as_i64),
            Some(0),
            "the template's count is ZERO, not absent"
        );
        assert_eq!(
            t.get("line_of").and_then(Value::as_i64),
            Some(3),
            "and the denominator is 3, so the pair is asymmetric on the template itself"
        );

        // 2. `milestone_ref` is set in a body naming no milestone.
        let f = &get("filled-basic").expected;
        assert!(
            f.get("milestone_ref")
                .is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty())),
            "milestone_ref is populated without a milestone being named"
        );

        // 3. An empty body still yields headline objects rather than nothing.
        let e = &get("empty").expected;
        assert!(
            e.get("headline_before").is_some_and(|v| v.is_object()),
            "empty body still yields a headline object"
        );
    }

    /// The bodies survive the round trip with their Unicode separators, because **the parser's whole
    /// difficulty is the `·` and `→` the template uses** and a mangled fixture would test nothing.
    #[test]
    fn the_bodies_keep_their_unicode() {
        let cases = load(&repo());
        let rich = cases
            .iter()
            .find(|c| c.name == "filled-basic")
            .expect("case");
        assert!(rich.body.contains('\u{00b7}'), "the middle dot separator");
        assert!(rich.body.contains('\u{2192}'), "the arrow");
    }
}
