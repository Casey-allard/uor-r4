//! `arena compare` — row-by-row comparison of one candidate against its parent (D30 port of
//! `arena.compare`).
//!
//! **This is the report the arena exists for.** The arena is a regression check, not a leaderboard, so
//! the comparison never collapses the bars into one number: it names every bar gained, lost and
//! unchanged, and every reason a comparison cannot be made at all.
//!
//! Two rules do the real work: **direction comes from the file's `bars` map and is never guessed** (an
//! undeclared bar is reported as `undeclared` rather than assumed up, because a 92% increase in
//! candidates scored per query would otherwise read as a gain), and **a KEEP must report fresh transfer**,
//! looked for as evidence rather than as the word.

use std::path::Path;

use serde_json::Value;

use crate::arena::{Arena, Candidate, HISTORY};
use crate::ordered::Ordered;

/// The declared direction and unit of one bar, from the arena file's `bars` map.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BarMeta {
    pub direction: String,
    pub unit: String,
}

/// `bar_meta`: every bar's declared direction and unit, in file order.
pub fn bar_meta(repo: &Path) -> Vec<(String, BarMeta)> {
    let Ok(text) = std::fs::read_to_string(repo.join(HISTORY)) else {
        return Vec::new();
    };
    let Ok(doc) = Ordered::parse(&text) else {
        return Vec::new();
    };
    let Some(bars) = doc.get("bars").and_then(Ordered::as_object) else {
        return Vec::new();
    };
    bars.iter()
        .map(|(k, v)| {
            (
                k.clone(),
                BarMeta {
                    direction: v.str_or_empty("direction"),
                    unit: v.str_or_empty("unit"),
                },
            )
        })
        .collect()
}

/// `_direction`: the declared direction, or `undeclared` - never a guess.
pub fn direction_of(meta: &[(String, BarMeta)], key: &str) -> String {
    meta.iter()
        .find(|(k, _)| k == key)
        .map(|(_, m)| m.direction.clone())
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| "undeclared".to_string())
}

pub fn unit_of(meta: &[(String, BarMeta)], key: &str) -> String {
    meta.iter()
        .find(|(k, _)| k == key)
        .map(|(_, m)| m.unit.clone())
        .unwrap_or_default()
}

/// `_is_better`: `Some(true)` improved, `Some(false)` regressed, `None` unchanged or undeclared.
pub fn is_better(direction: &str, candidate: f64, parent: f64) -> Option<bool> {
    if candidate == parent {
        return None;
    }
    match direction {
        "higher_is_better" => Some(candidate > parent),
        "lower_is_better" => Some(candidate < parent),
        _ => None,
    }
}

fn num(v: &Value) -> Option<f64> {
    v.as_f64()
}

/// `round(b - a, 6)`, **keeping int-ness when both sides are ints**.
///
/// Python's `round(13, 6)` is the int `13` and `json.dumps` prints `13`; a float-only port would print
/// `13.0` there and diverge on every integer bar in the file.
fn delta_value(a: &Value, b: &Value) -> Value {
    if let (Some(x), Some(y)) = (a.as_i64(), b.as_i64()) {
        return Value::from(y - x);
    }
    let d = num(b).unwrap_or(0.0) - num(a).unwrap_or(0.0);
    Value::from((d * 1e6).round() / 1e6)
}

/// `_reports_transfer`: **evidence, not the word.** `"transfer" in scope` is not enough, because *"no
/// transfer reported"* contains it.
pub fn reports_transfer(cand: &Candidate) -> bool {
    if cand
        .scores
        .as_object()
        .is_some_and(|s| s.contains_key("transfer"))
    {
        return true;
    }
    static TRANSFER: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let rx = TRANSFER.get_or_init(|| {
        regex::Regex::new(r"(?:fresh\s+)?transfer\s*[:=]?\s*[0-9]").expect("a valid pattern")
    });
    rx.is_match(&cand.scope) || rx.is_match(&cand.notes)
}

/// One bar in a comparison, in Python's entry order.
#[derive(Debug, Clone, Default)]
pub struct BarRow {
    pub bar: String,
    pub parent: Value,
    pub candidate: Value,
    pub delta: Value,
    pub direction: String,
    pub unit: String,
    pub note: String,
    /// `UNAVAILABLE` rows carry a `why` instead of the numeric fields.
    pub why: String,
}

/// The result of comparing one candidate against its parent.
#[derive(Debug, Clone, Default)]
pub struct CompareResult {
    pub ok: bool,
    pub reason: String,
    pub candidate: String,
    pub artifact: String,
    pub milestone: String,
    pub parent: Option<String>,
    pub verdict: String,
    pub confidence: String,
    pub comparable: bool,
    pub gained: Vec<BarRow>,
    pub lost: Vec<BarRow>,
    pub unchanged: Vec<BarRow>,
    pub unavailable: Vec<BarRow>,
    pub blockers: Vec<String>,
}

fn py_repr(s: &str) -> String {
    crate::prgates::py_repr_public(s)
}

fn py_str_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "None".to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// `compare`: row by row, one candidate against its parent.
pub fn compare(arena: &Arena, meta: &[(String, BarMeta)], cid: &str) -> CompareResult {
    let mut r = CompareResult::default();
    let Some(cand) = arena.get(cid) else {
        return CompareResult {
            ok: false,
            reason: format!("no candidate {} in {HISTORY}", py_repr(cid)),
            candidate: cid.to_string(),
            ..CompareResult::default()
        };
    };
    let parent = cand.parent.as_deref().and_then(|p| arena.get(p));
    r.ok = true;
    r.candidate = cand.id.clone();
    r.artifact = cand.artifact.clone();
    r.milestone = cand.milestone.clone();
    r.parent = parent.map(|p| p.id.clone());
    r.verdict = cand.verdict.clone();
    r.confidence = cand.confidence.clone();
    r.comparable = true;

    // Scope gates: what stops this row from supporting a decision at all.
    if cand.confidence == "uninterpretable" {
        r.comparable = false;
        r.blockers.push(
            "the row is recorded as uninterpretable; fix the configuration before reading it"
                .to_string(),
        );
    }
    if cand.scope.is_empty() {
        r.blockers.push(
            "no scope recorded: an artifact/data/steps scope must be stated for the result to have one"
                .to_string(),
        );
    }
    if cand.confidence == "decisive" {
        let seeds = cand.seeds.as_i64().unwrap_or(0);
        let rows = cand.rows.as_i64().unwrap_or(0);
        if seeds < 2 && rows < 200 {
            r.blockers.push(format!(
                "claimed decisive with {seeds} seed(s) and {rows} row(s); a decisive run needs >=2 seeds or >=200 fresh rows (D22)"
            ));
        }
    }
    if cand.verdict == "KEEP" && !reports_transfer(cand) {
        r.blockers.push(
            "a development KEEP must report fresh transfer (D22): record a 'transfer' bar or state 'fresh transfer: <number>' in the scope"
                .to_string(),
        );
    }

    let Some(parent) = parent else {
        r.comparable = false;
        r.reason = "this is a root row: there is no parent to regress against".to_string();
        return r;
    };

    if !cand.panel.is_empty() && !parent.panel.is_empty() && cand.panel != parent.panel {
        r.comparable = false;
        r.blockers.push(format!(
            "panel changed between parent and candidate ({} -> {}); a comparison across panels is not a comparison",
            parent.panel, cand.panel
        ));
    }

    // The UNION of both sides' bars, sorted: a bar only one side reports becomes UNAVAILABLE rather
    // than a silent delta.
    let mut keys: Vec<String> = Vec::new();
    for source in [&parent.scores, &cand.scores] {
        if let Some(map) = source.as_object() {
            for k in map.keys() {
                if !keys.contains(k) {
                    keys.push(k.clone());
                }
            }
        }
    }
    keys.sort();

    for key in keys {
        let a = parent.scores.get(&key).cloned().unwrap_or(Value::Null);
        let b = cand.scores.get(&key).cloned().unwrap_or(Value::Null);
        if num(&a).is_some() && num(&b).is_some() {
            let direction = direction_of(meta, &key);
            let mut row = BarRow {
                bar: key.clone(),
                parent: a.clone(),
                candidate: b.clone(),
                delta: delta_value(&a, &b),
                direction: direction.clone(),
                unit: unit_of(meta, &key),
                note: String::new(),
                why: String::new(),
            };
            let better = match (num(&a), num(&b)) {
                (Some(av), Some(bv)) => is_better(&direction, bv, av),
                _ => None,
            };
            if better.is_none() && direction == "undeclared" {
                row.note =
                    "bar direction is not declared, so this cannot be called a gain or a regression"
                        .to_string();
            }
            match better {
                Some(true) => r.gained.push(row),
                Some(false) => r.lost.push(row),
                None => r.unchanged.push(row),
            }
        } else if b.is_null() {
            r.unavailable.push(BarRow {
                bar: key.clone(),
                parent: a.clone(),
                candidate: Value::Null,
                why: "the candidate does not report this bar".to_string(),
                ..BarRow::default()
            });
        } else if a.is_null() {
            r.unavailable.push(BarRow {
                bar: key.clone(),
                parent: Value::Null,
                candidate: b.clone(),
                why: "the parent does not report this bar".to_string(),
                ..BarRow::default()
            });
        }
        // A value that is neither numeric nor null is skipped, as Python's two `elif`s do.
    }
    r
}

/// `render`: the text form. **The regression sentence is printed only when a bar was lost.**
pub fn render(r: &CompareResult) -> String {
    if !r.ok {
        return format!("arena: {}", r.reason);
    }
    let mut out: Vec<String> = Vec::new();
    out.push(format!(
        "candidate {}  ({}, {}, {})",
        r.candidate, r.milestone, r.verdict, r.confidence
    ));
    out.push(format!("  artifact: {}", r.artifact));
    out.push(format!(
        "  parent:   {}",
        r.parent.clone().unwrap_or_else(|| "(root row)".to_string())
    ));
    for b in &r.blockers {
        out.push(format!("  BLOCKER: {b}"));
    }
    if !r.comparable {
        out.push("  not comparable - see blockers above".to_string());
        return out.join("\n");
    }
    out.push(String::new());
    out.push("  row-by-row against the parent (never an aggregate):".to_string());
    for (label, rows) in [
        ("gained", &r.gained),
        ("lost", &r.lost),
        ("unchanged", &r.unchanged),
    ] {
        for e in rows {
            // Python: `"+" if delta > 0 else ""` - the minus comes from the number itself.
            let arrow = if num(&e.delta).unwrap_or(0.0) > 0.0 {
                "+"
            } else {
                ""
            };
            let unit = if e.unit.is_empty() {
                String::new()
            } else {
                format!(" {}", e.unit)
            };
            let note = if e.note.is_empty() {
                String::new()
            } else {
                format!("  [{}]", e.note)
            };
            out.push(format!(
                "    {label:<9} {:<24} {} -> {}  ({arrow}{})  {}{unit}{note}",
                e.bar,
                py_str_of(&e.parent),
                py_str_of(&e.candidate),
                py_str_of(&e.delta),
                e.direction
            ));
        }
    }
    for e in &r.unavailable {
        out.push(format!("    UNAVAILABLE {:<24} {}", e.bar, e.why));
    }
    if !r.lost.is_empty() {
        out.push(String::new());
        out.push(format!(
            "  {} bar(s) regressed. A gain elsewhere does not offset them;",
            r.lost.len()
        ));
        out.push("  report both, and say which one the decision turns on.".to_string());
    }
    out.join("\n")
}

/// `compare --json`: Python's dict order, with `reason` **last** on both early returns.
///
/// `json.dumps(..., indent=2)` here has **ensure_ascii ON**, unlike `arena board`'s neighbouring call.
pub fn json(r: &CompareResult) -> String {
    use crate::ordered::Ordered;
    if !r.ok {
        let doc = Ordered::Obj(vec![
            ("ok".to_string(), Ordered::Bool(false)),
            ("reason".to_string(), Ordered::Str(r.reason.clone())),
            ("candidate".to_string(), Ordered::Str(r.candidate.clone())),
        ]);
        return crate::registries::ensure_ascii(&doc.to_pretty());
    }
    let entry = |e: &BarRow, unavailable: bool| {
        let mut pairs = vec![
            ("bar".to_string(), Ordered::Str(e.bar.clone())),
            ("parent".to_string(), ord(&e.parent)),
            ("candidate".to_string(), ord(&e.candidate)),
        ];
        if unavailable {
            pairs.push(("why".to_string(), Ordered::Str(e.why.clone())));
        } else {
            pairs.push(("delta".to_string(), ord(&e.delta)));
            pairs.push(("direction".to_string(), Ordered::Str(e.direction.clone())));
            pairs.push(("unit".to_string(), Ordered::Str(e.unit.clone())));
            if !e.note.is_empty() {
                pairs.push(("note".to_string(), Ordered::Str(e.note.clone())));
            }
        }
        Ordered::Obj(pairs)
    };
    let list = |rows: &[BarRow], unavailable: bool| {
        Ordered::Arr(rows.iter().map(|e| entry(e, unavailable)).collect())
    };
    let mut pairs = vec![
        ("ok".to_string(), Ordered::Bool(true)),
        ("candidate".to_string(), Ordered::Str(r.candidate.clone())),
        ("artifact".to_string(), Ordered::Str(r.artifact.clone())),
        ("milestone".to_string(), Ordered::Str(r.milestone.clone())),
        (
            "parent".to_string(),
            r.parent.clone().map(Ordered::Str).unwrap_or(Ordered::Null),
        ),
        ("verdict".to_string(), Ordered::Str(r.verdict.clone())),
        ("confidence".to_string(), Ordered::Str(r.confidence.clone())),
        ("comparable".to_string(), Ordered::Bool(r.comparable)),
        ("gained".to_string(), list(&r.gained, false)),
        ("lost".to_string(), list(&r.lost, false)),
        ("unchanged".to_string(), list(&r.unchanged, false)),
        ("unavailable".to_string(), list(&r.unavailable, true)),
        (
            "blockers".to_string(),
            Ordered::Arr(r.blockers.iter().map(|b| Ordered::Str(b.clone())).collect()),
        ),
    ];
    if !r.reason.is_empty() {
        pairs.push(("reason".to_string(), Ordered::Str(r.reason.clone())));
    }
    crate::registries::ensure_ascii(&Ordered::Obj(pairs).to_pretty())
}

/// A `serde_json::Value` as an `Ordered` node.
fn ord(v: &Value) -> Ordered {
    match v {
        Value::Null => Ordered::Null,
        Value::Bool(b) => Ordered::Bool(*b),
        Value::Number(n) => Ordered::Num(n.clone()),
        Value::String(s) => Ordered::Str(s.clone()),
        Value::Array(items) => Ordered::Arr(items.iter().map(ord).collect()),
        Value::Object(map) => Ordered::Obj(map.iter().map(|(k, v)| (k.clone(), ord(v))).collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(id: &str, parent: Option<&str>, scores: Value) -> Candidate {
        Candidate {
            id: id.to_string(),
            parent: parent.map(str::to_string),
            panel: "p".to_string(),
            scope: "a scope".to_string(),
            verdict: "NOT_RUN".to_string(),
            confidence: "provisional".to_string(),
            scores,
            ..Candidate::default()
        }
    }

    fn meta() -> Vec<(String, BarMeta)> {
        vec![
            (
                "rows".to_string(),
                BarMeta {
                    direction: "higher_is_better".to_string(),
                    unit: "rows".to_string(),
                },
            ),
            ("mystery".to_string(), BarMeta::default()),
        ]
    }

    fn arena_of(rows: Vec<Candidate>) -> Arena {
        Arena {
            rows,
            raw: Value::Null,
        }
    }

    /// An int-minus-int delta stays an INT, as Python's `round(13, 6)` does.
    #[test]
    fn an_integer_delta_stays_an_integer() {
        let a = arena_of(vec![
            cand("parent", None, serde_json::json!({"rows": 100})),
            cand("kid", Some("parent"), serde_json::json!({"rows": 113})),
        ]);
        let r = compare(&a, &meta(), "kid");
        assert!(r.ok && r.comparable);
        assert_eq!(r.gained.len(), 1);
        assert_eq!(r.gained[0].delta, serde_json::json!(13));
        assert!(json(&r).contains("\"delta\": 13,"), "{}", json(&r));
    }

    /// An undeclared direction is reported, not assumed: it cannot be called a gain or a regression.
    #[test]
    fn an_undeclared_bar_says_so_and_is_not_a_gain() {
        let a = arena_of(vec![
            cand("parent", None, serde_json::json!({"mystery": 10})),
            cand("kid", Some("parent"), serde_json::json!({"mystery": 20})),
        ]);
        let r = compare(&a, &meta(), "kid");
        assert!(r.gained.is_empty() && r.lost.is_empty());
        assert_eq!(r.unchanged.len(), 1);
        assert!(
            r.unchanged[0].note.contains("not declared"),
            "{:?}",
            r.unchanged[0].note
        );
    }

    /// A bar only one side reports is UNAVAILABLE, never a delta - and a regression prints its warning.
    #[test]
    fn a_one_sided_bar_is_unavailable_and_a_regression_says_so() {
        let a = arena_of(vec![
            cand("parent", None, serde_json::json!({"rows": 200, "gone": 5})),
            cand("kid", Some("parent"), serde_json::json!({"rows": 150})),
        ]);
        let r = compare(&a, &meta(), "kid");
        assert_eq!(
            r.lost.len(),
            1,
            "150 is below 200 on a higher-is-better bar"
        );
        assert_eq!(r.unavailable.len(), 1);
        assert_eq!(
            r.unavailable[0].why,
            "the candidate does not report this bar"
        );
        let text = render(&r);
        assert!(text.contains("UNAVAILABLE gone"), "{text}");
        assert!(
            text.contains("1 bar(s) regressed. A gain elsewhere does not offset them;"),
            "{text}"
        );
    }

    /// A root row is `ok` with `comparable: false` and exit 0 - not a failure.
    #[test]
    fn a_root_row_is_not_comparable_but_is_still_ok() {
        let a = arena_of(vec![cand("root", None, serde_json::json!({"rows": 1}))]);
        let r = compare(&a, &meta(), "root");
        assert!(r.ok);
        assert!(!r.comparable);
        assert_eq!(
            r.reason,
            "this is a root row: there is no parent to regress against"
        );
        assert!(render(&r).contains("  not comparable - see blockers above"));
    }

    /// A KEEP without fresh transfer is blocked; the word is not enough, the evidence is.
    #[test]
    fn a_keep_must_report_transfer_as_evidence() {
        let mut keep = cand("kid", Some("parent"), serde_json::json!({"rows": 1}));
        keep.verdict = "KEEP".to_string();
        keep.scope = "no transfer reported".to_string();
        let a = arena_of(vec![
            cand("parent", None, serde_json::json!({"rows": 1})),
            keep,
        ]);
        let r = compare(&a, &meta(), "kid");
        assert!(
            r.blockers.iter().any(|b| b.contains("fresh transfer")),
            "{:?}",
            r.blockers
        );
    }

    #[test]
    fn an_unknown_candidate_is_not_ok() {
        let a = arena_of(vec![]);
        let r = compare(&a, &meta(), "nope");
        assert!(!r.ok);
        assert_eq!(r.reason, "no candidate 'nope' in docs/harness/arena.json");
        assert_eq!(
            render(&r),
            "arena: no candidate 'nope' in docs/harness/arena.json"
        );
    }
}
