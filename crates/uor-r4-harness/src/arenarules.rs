//! The `arena` rules, as Rust (D30 check-port step 6 of 10).
//!
//! # The four `arena` rules
//!
//! | rule | severity | what it catches |
//! |---|---|---|
//! | `arena.rows_valid` | error | a row missing an id/artifact/milestone/verdict/confidence, or a duplicate id |
//! | `arena.parents_exist` | error | a named parent that is not a row, self-parenting, or a parent cycle |
//! | `arena.decisive_power` | warn | a row claiming `decisive` below the D22 minimum |
//! | `arena.knows_what_it_cannot_measure` | warn | the arena does not state its own limits |
//!
//! # The two that carry the most weight
//!
//! **`decisive_power` is D22 as a check.** A row claiming `decisive` confidence with fewer than
//! **2 seeds or 200 fresh rows** is not decisive — it is a provisional reading wearing a decisive
//! label, and **the label is what a reader trusts.** The rule cannot tell whether the run happened;
//! it can tell whether the claim is *entitled*, which is the part a machine can decide.
//!
//! **`knows_what_it_cannot_measure` is the one that ages best.** A row list with no stated limits
//! **reads as complete** — a reader sees five candidates and concludes the arena covers the space. The
//! `limits` string is what stops that inference, so its absence is a warning rather than a note.

use crate::state::StatePaths;

pub use crate::preprules::Finding;

/// Verdict values a row may declare.
pub const VERDICTS: [&str; 5] = [
    "KEEP",
    "REJECT",
    "NOT_YET_PROMOTED",
    "NOT_RUN",
    "UNAVAILABLE",
];
/// Confidence values a row may declare.
pub const CONFIDENCE: [&str; 4] = [
    "decisive",
    "provisional",
    "scoped-reading",
    "uninterpretable",
];

/// The file every finding points at.
const HISTORY: &str = "docs/harness/arena.json";

/// D22 section 3: a decisive run states at least this many seeds **or** this many fresh rows.
pub const MIN_SEEDS: i64 = 2;
pub const MIN_ROWS: i64 = 200;

/// The shortest `limits` string that counts as stating a limit.
///
/// **Forty characters, because a shorter one is a gesture.** `"none"`, `"see docs"` and `"TBD"` all
/// satisfy "there is a limits field" while stating nothing, and the rule exists to stop the arena
/// being read as complete — so it has to require an actual sentence.
pub const MIN_LIMITS_CHARS: usize = 40;

fn error(rule_id: &'static str, message: String, fix: &str) -> Finding {
    Finding {
        rule_id,
        severity: "error",
        message,
        path: HISTORY.into(),
        fix: fix.into(),
    }
}

fn warn(rule_id: &'static str, message: String, fix: &str) -> Finding {
    Finding {
        rule_id,
        severity: "warn",
        message,
        path: HISTORY.into(),
        fix: fix.into(),
    }
}

/// One row of the arena, as far as these rules need to see it.
#[derive(Debug, Clone, Default)]
pub struct Row {
    pub id: String,
    pub artifact: String,
    pub milestone: String,
    pub verdict: String,
    pub confidence: String,
    pub parent: String,
    pub seeds: i64,
    pub rows: i64,
}

/// The loaded arena: its rows and its raw object.
#[derive(Debug, Clone, Default)]
pub struct Arena {
    pub rows: Vec<Row>,
    pub limits: String,
}

impl Arena {
    fn get(&self, id: &str) -> Option<&Row> {
        self.rows.iter().find(|r| r.id == id)
    }
}

/// Load `docs/harness/arena.json`. A file that is missing or unparseable yields an empty arena, which
/// makes every rule report nothing — **so the caller that needs absence to be visible must check
/// `rows.is_empty()` itself.** `arena.rows_valid` does exactly that.
pub fn load(paths: &StatePaths) -> Arena {
    let file = paths.repo.join(HISTORY);
    let Ok(text) = std::fs::read_to_string(&file) else {
        return Arena::default();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Arena::default();
    };
    let s = |o: &serde_json::Value, k: &str| -> String {
        o.get(k)
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string()
    };
    let n =
        |o: &serde_json::Value, k: &str| -> i64 { o.get(k).and_then(|x| x.as_i64()).unwrap_or(0) };
    let rows = v
        .get("candidates")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .map(|o| Row {
                    id: s(o, "id"),
                    artifact: s(o, "artifact"),
                    milestone: s(o, "milestone"),
                    verdict: s(o, "verdict"),
                    confidence: s(o, "confidence"),
                    parent: s(o, "parent"),
                    seeds: n(o, "seeds"),
                    rows: n(o, "rows"),
                })
                .collect()
        })
        .unwrap_or_default();
    Arena {
        rows,
        limits: s(&v, "limits"),
    }
}

/// `arena.rows_valid` — every row has an id, artifact, milestone, verdict and confidence, and no two
/// rows share an id.
///
/// **The duplicate check is first because it invalidates everything after it:** with two rows of one
/// id, `parents_exist` and every lookup by id resolve to whichever row was read first, so the rest of
/// the arena's structure is describing a file that does not exist.
pub fn rows_valid(paths: &StatePaths) -> Vec<Finding> {
    let arena = load(paths);
    if arena.rows.is_empty() {
        // An absent or empty arena is not "valid" - it is a file the rules cannot check, and saying
        // nothing would read as a pass. Reported as its own case so the silence is not ambiguous.
        return vec![error(
            "arena.rows_valid",
            format!("{HISTORY} is missing, unparseable, or has no rows - nothing could be checked"),
            "restore the arena file, or say why the arena is empty",
        )];
    }
    let mut out = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for r in &arena.rows {
        if seen.contains(&r.id.as_str()) {
            out.push(error(
                "arena.rows_valid",
                format!("duplicate candidate id {:?}", r.id),
                "give one of them a unique id",
            ));
        }
        seen.push(&r.id);
        for (name, value) in [
            ("id", &r.id),
            ("artifact", &r.artifact),
            ("milestone", &r.milestone),
            ("verdict", &r.verdict),
            ("confidence", &r.confidence),
        ] {
            if value.trim().is_empty() {
                let who = if r.id.is_empty() {
                    "(no id)".to_string()
                } else {
                    r.id.clone()
                };
                out.push(error(
                    "arena.rows_valid",
                    format!("candidate {who} has no {name}"),
                    "name every field on the row",
                ));
            }
        }
        if !r.verdict.trim().is_empty() && !VERDICTS.contains(&r.verdict.as_str()) {
            out.push(error(
                "arena.rows_valid",
                format!(
                    "candidate {}: verdict {:?} is not one of {VERDICTS:?}",
                    r.id, r.verdict
                ),
                "use one of the declared verdicts",
            ));
        }
        if !r.confidence.trim().is_empty() && !CONFIDENCE.contains(&r.confidence.as_str()) {
            out.push(error(
                "arena.rows_valid",
                format!(
                    "candidate {}: confidence {:?} is not one of {CONFIDENCE:?}",
                    r.id, r.confidence
                ),
                "use one of the declared confidence values",
            ));
        }
    }
    out
}

/// `arena.parents_exist` — every named parent is a row, no row is its own parent, and no cycle.
///
/// **A cycle is reported once per entry point, which is the Python's behaviour kept.** The alternative
/// — reporting the cycle once — needs a visited set across the whole walk and would hide the second
/// row that leads into it, which is the row a reader has to fix.
pub fn parents_exist(paths: &StatePaths) -> Vec<Finding> {
    let arena = load(paths);
    let ids: Vec<&str> = arena.rows.iter().map(|r| r.id.as_str()).collect();
    let mut out = Vec::new();
    for r in &arena.rows {
        if !r.parent.is_empty() && !ids.contains(&r.parent.as_str()) {
            out.push(error(
                "arena.parents_exist",
                format!(
                    "candidate {} names parent {:?}, which is not a row",
                    r.id, r.parent
                ),
                "name an existing row as the parent, or clear it",
            ));
        }
        if !r.parent.is_empty() && r.parent == r.id {
            out.push(error(
                "arena.parents_exist",
                format!("candidate {} is its own parent", r.id),
                "clear the parent, or point it at the row this one descends from",
            ));
        }
    }
    for r in &arena.rows {
        let mut seen: Vec<String> = Vec::new();
        let mut cur = r;
        while !cur.parent.is_empty() && ids.contains(&cur.parent.as_str()) {
            if seen.contains(&cur.parent) {
                out.push(error(
                    "arena.parents_exist",
                    format!("parent cycle through {}", r.id),
                    "break the cycle: one of these rows must not name a parent",
                ));
                break;
            }
            seen.push(cur.parent.clone());
            match arena.get(&cur.parent) {
                Some(next) => cur = next,
                None => break,
            }
        }
    }
    out
}

/// `arena.decisive_power` — a row claiming `decisive` meets the D22 minimum.
///
/// **D22 as a check.** Fewer than 2 seeds **and** fewer than 200 fresh rows is not a decisive reading,
/// and **the label is what a reader trusts.** The rule cannot tell whether the run happened; it can
/// tell whether the claim is *entitled*, which is the part a machine can decide.
pub fn decisive_power(paths: &StatePaths) -> Vec<Finding> {
    let mut out = Vec::new();
    for r in load(paths).rows {
        if r.confidence != "decisive" {
            continue;
        }
        if r.seeds < MIN_SEEDS && r.rows < MIN_ROWS {
            out.push(warn(
                "arena.decisive_power",
                format!(
                    "candidate {} claims decisive with {} seed(s) and {} row(s)",
                    r.id, r.seeds, r.rows
                ),
                "record it as provisional, or run >=2 seeds / >=200 fresh rows (D22)",
            ));
        }
    }
    out
}

/// `arena.knows_what_it_cannot_measure` — the arena states its own limits.
///
/// **A row list with no stated limits reads as complete.** The `limits` string is what stops a reader
/// inferring that five candidates cover the space, so its absence is a warning rather than a note.
pub fn knows_what_it_cannot_measure(paths: &StatePaths) -> Vec<Finding> {
    let limits = load(paths).limits.trim().to_string();
    if limits.chars().count() >= MIN_LIMITS_CHARS {
        return Vec::new();
    }
    vec![warn(
        "arena.knows_what_it_cannot_measure",
        "the arena does not state what it cannot measure, so it will be read as complete".into(),
        "add a \"limits\" string to docs/harness/arena.json naming the blind spots",
    )]
}

/// Every `arena` rule.
pub fn all(paths: &StatePaths) -> Vec<Finding> {
    let mut out = rows_valid(paths);
    out.extend(parents_exist(paths));
    out.extend(decisive_power(paths));
    out.extend(knows_what_it_cannot_measure(paths));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StatePaths;

    fn fixture(arena_json: &str) -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let d = dir.path().join("docs/harness");
        std::fs::create_dir_all(&d).expect("mkdir");
        if !arena_json.is_empty() {
            std::fs::write(d.join("arena.json"), arena_json).expect("write");
        }
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        (dir, p)
    }

    fn row(id: &str, extra: &str) -> String {
        format!(
            r#"{{"id":"{id}","artifact":"a-{id}","milestone":"m1","verdict":"KEEP","confidence":"provisional"{extra}}}"#
        )
    }

    fn arena(rows: &[String], limits: &str) -> String {
        format!(
            r#"{{"limits":"{limits}","candidates":[{}]}}"#,
            rows.join(",")
        )
    }

    /// **An absent arena must not read as valid.** Saying nothing would look like a pass, so the
    /// missing file is reported as its own case.
    #[test]
    fn a_missing_arena_is_reported_rather_than_passing_silently() {
        let (_d, p) = fixture("");
        let f = rows_valid(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message
                .contains("missing, unparseable, or has no rows"),
            "{}",
            f[0].message
        );
    }

    #[test]
    fn a_duplicate_id_is_reported() {
        let (_d, p) = fixture(&arena(
            &[row("dup", ""), row("dup", "")],
            "a limit string that is long enough to count",
        ));
        let f = rows_valid(&p);
        assert!(
            f.iter()
                .any(|x| x.message.contains("duplicate candidate id")),
            "{f:?}"
        );
    }

    #[test]
    fn a_row_missing_a_field_names_both_the_row_and_the_field() {
        let (_d, p) = fixture(&arena(
            &[r#"{"id":"r1","artifact":"","milestone":"m1","verdict":"KEEP","confidence":"provisional"}"#.to_string()],
            "a limit string that is long enough to count",
        ));
        let f = rows_valid(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("r1"), "{}", f[0].message);
        assert!(f[0].message.contains("artifact"), "{}", f[0].message);
    }

    #[test]
    fn an_unknown_verdict_and_confidence_are_reported() {
        let (_d, p) = fixture(&arena(
            &[r#"{"id":"r1","artifact":"a","milestone":"m1","verdict":"MAYBE","confidence":"sure"}"#.to_string()],
            "a limit string that is long enough to count",
        ));
        let f = rows_valid(&p);
        assert_eq!(f.len(), 2, "{f:?}");
        assert!(f.iter().any(|x| x.message.contains("MAYBE")), "{f:?}");
        assert!(f.iter().any(|x| x.message.contains("sure")), "{f:?}");
    }

    /// A parent that is not a row, and self-parenting, are different mistakes.
    #[test]
    fn a_missing_parent_and_self_parenting_are_reported() {
        let (_d, p) = fixture(&arena(
            &[
                row("a", r#","parent":"ghost""#),
                row("b", r#","parent":"b""#),
            ],
            "a limit string that is long enough to count",
        ));
        let f = parents_exist(&p);
        assert!(f.iter().any(|x| x.message.contains("not a row")), "{f:?}");
        assert!(
            f.iter().any(|x| x.message.contains("its own parent")),
            "{f:?}"
        );
    }

    /// **A cycle is the case a naive walk hangs on**, so it is asserted explicitly.
    #[test]
    fn a_parent_cycle_is_reported_and_the_walk_terminates() {
        let (_d, p) = fixture(&arena(
            &[row("a", r#","parent":"b""#), row("b", r#","parent":"a""#)],
            "a limit string that is long enough to count",
        ));
        let f = parents_exist(&p);
        assert!(
            f.iter().any(|x| x.message.contains("parent cycle")),
            "{f:?}"
        );
    }

    #[test]
    fn a_valid_parent_chain_is_accepted() {
        let (_d, p) = fixture(&arena(
            &[row("a", ""), row("b", r#","parent":"a""#)],
            "a limit string that is long enough to count",
        ));
        assert!(parents_exist(&p).is_empty(), "{:?}", parents_exist(&p));
    }

    /// **The D22 claim check: the label is what a reader trusts.**
    #[test]
    fn decisive_with_too_little_power_is_reported() {
        let (_d, p) = fixture(&arena(
            &[row("d", r#","confidence":"decisive","seeds":1,"rows":10"#)],
            "a limit string that is long enough to count",
        ));
        let f = decisive_power(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message.contains("1 seed(s) and 10 row(s)"),
            "{}",
            f[0].message
        );
    }

    /// Either sufficient seeds OR sufficient rows satisfies D22 - not both.
    #[test]
    fn decisive_with_two_seeds_or_two_hundred_rows_passes() {
        for extra in [
            r#","confidence":"decisive","seeds":2,"rows":1"#,
            r#","confidence":"decisive","seeds":0,"rows":200"#,
        ] {
            let (_d, p) = fixture(&arena(
                &[row("d", extra)],
                "a limit string that is long enough to count",
            ));
            assert!(decisive_power(&p).is_empty(), "either suffices: {extra}");
        }
    }

    /// A short limits string is a gesture, not a statement.
    #[test]
    fn a_short_limits_string_does_not_count() {
        for short in ["", "none", "see docs", "TBD"] {
            let (_d, p) = fixture(&arena(&[row("a", "")], short));
            assert_eq!(
                knows_what_it_cannot_measure(&p).len(),
                1,
                "{short:?} must not count"
            );
        }
    }

    #[test]
    fn a_real_limits_string_counts() {
        let (_d, p) = fixture(&arena(
            &[row("a", "")],
            "the arena cannot measure energy per token, and it does not cover serving",
        ));
        assert!(knows_what_it_cannot_measure(&p).is_empty());
    }

    /// The real arena must pass all four - if it does not, Python would have said so.
    #[test]
    fn the_real_arena_passes_every_rule() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let f = all(&p);
        assert!(f.is_empty(), "the real arena fails a rule: {f:?}");
    }
}
