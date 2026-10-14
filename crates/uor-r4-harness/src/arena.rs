//! `arena` — the regression arena (D30 port of `arena.py`).
//!
//! Nobody here is racing transformers. What this project needs is the opposite of a leaderboard: **a
//! regression check**, where every candidate is an edit to the geometry and the question is never "is it
//! better on average" but "what did it break, and what did it fix, on the frozen panels".
//!
//! So the arena is a **lineage**: each row names its **parent**, and the records form the actual tree of
//! how the model evolved. `lineage <id>` walks from a root down to the candidate, which is what makes a
//! row-by-row comparison possible at all.
//!
//! Read-only: this module reports. It does not run a model, choose a candidate, or retire anything.

use std::path::Path;

use serde_json::Value;

/// Where the arena history lives, relative to the repo root.
pub const HISTORY: &str = "docs/harness/arena.json";

/// One evaluated configuration - one row of the arena.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Candidate {
    pub id: String,
    pub artifact: String,
    pub milestone: String,
    pub date: String,
    /// `None` for a root: **absence is meaningful here**, so this is not an empty string.
    pub parent: Option<String>,
    pub verdict: String,
    pub confidence: String,
    pub mechanism: String,
    pub panel: String,
    pub seeds: Value,
    pub rows: Value,
    pub scope: String,
    pub scores: Value,
    pub notes: String,
    pub record: String,
}

impl Candidate {
    /// `confidence == "decisive"` - a decisive row is the only kind a decision may rest on.
    pub fn decisive(&self) -> bool {
        self.confidence == "decisive"
    }
}

/// The arena: its rows and the file they came from.
#[derive(Debug, Clone, Default)]
pub struct Arena {
    pub rows: Vec<Candidate>,
    pub raw: Value,
}

impl Arena {
    pub fn get(&self, cid: &str) -> Option<&Candidate> {
        self.rows.iter().find(|c| c.id == cid)
    }

    /// Walk from a root down to `cid`, so a comparison has the full chain.
    ///
    /// **The `seen` set is not decoration**: a row whose parent chain loops would otherwise walk
    /// forever, and an arena file is hand-written JSON.
    pub fn lineage(&self, cid: &str) -> Vec<&Candidate> {
        let mut chain: Vec<&Candidate> = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        let mut cur = self.get(cid);
        while let Some(c) = cur {
            if seen.iter().any(|s| s == &c.id) {
                break;
            }
            seen.push(c.id.clone());
            chain.push(c);
            cur = c.parent.as_deref().and_then(|p| self.get(p));
        }
        chain.reverse();
        chain
    }
}

fn s(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

/// Load the arena history.
///
/// **A missing or unreadable file is an EMPTY arena, not an error**: the arena starts empty, and a lab
/// that has measured nothing yet must get the same answer as one whose file is unreadable - "no rows".
pub fn load(repo: &Path) -> Arena {
    let path = repo.join(HISTORY);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Arena {
            rows: Vec::new(),
            raw: serde_json::json!({"schema": 1, "candidates": []}),
        };
    };
    let Ok(raw) = serde_json::from_str::<Value>(&text) else {
        return Arena {
            rows: Vec::new(),
            raw: serde_json::json!({"schema": 1, "candidates": []}),
        };
    };
    let mut rows = Vec::new();
    if let Some(list) = raw.get("candidates").and_then(Value::as_array) {
        for c in list {
            rows.push(Candidate {
                id: s(c, "id"),
                artifact: s(c, "artifact"),
                milestone: s(c, "milestone"),
                date: s(c, "date"),
                parent: c.get("parent").and_then(Value::as_str).map(str::to_string),
                verdict: s(c, "verdict"),
                confidence: s(c, "confidence"),
                mechanism: s(c, "mechanism"),
                panel: s(c, "panel"),
                seeds: c.get("seeds").cloned().unwrap_or(Value::Null),
                rows: c.get("rows").cloned().unwrap_or(Value::Null),
                scope: s(c, "scope"),
                scores: c
                    .get("scores")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({})),
                notes: s(c, "notes"),
                record: s(c, "record"),
            });
        }
    }
    Arena { rows, raw }
}

/// `arena lineage <id>`, as Python prints it: one line per ancestor, indented by depth.
pub fn render_lineage(chain: &[&Candidate]) -> String {
    let mut out = String::new();
    for (i, c) in chain.iter().enumerate() {
        out.push_str(&format!(
            "{}{}  ({}, {})\n",
            "  ".repeat(i),
            c.id,
            c.milestone,
            c.verdict
        ));
    }
    out
}

// --------------------------------------------------------------------------------- arena board

/// Python's `str()` for a JSON scalar, which is how `limits` is read before it is trimmed.
fn py_str_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "None".to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// `util.truncate`: collapse the whitespace, then cut to `limit` code points on an ellipsis.
fn truncate(text: &str, limit: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= limit {
        return collapsed;
    }
    let head: String = collapsed.chars().take(limit.saturating_sub(1)).collect();
    format!("{}…", head.trim_end())
}

/// A `serde_json::Value` as an `Ordered` node - for the scalar fields `board --json` copies through.
fn ord(v: &Value) -> crate::ordered::Ordered {
    use crate::ordered::Ordered;
    match v {
        Value::Null => Ordered::Null,
        Value::Bool(b) => Ordered::Bool(*b),
        Value::Number(n) => Ordered::Num(n.clone()),
        Value::String(s) => Ordered::Str(s.clone()),
        Value::Array(items) => Ordered::Arr(items.iter().map(ord).collect()),
        Value::Object(map) => Ordered::Obj(map.iter().map(|(k, v)| (k.clone(), ord(v))).collect()),
    }
}

/// `arena board`, as Python prints it.
///
/// **The rows are sorted by `date` and by nothing else, stably.** That is the whole point of the
/// table: it is a lineage in the order things were measured, not a ranking.
pub fn render_board(arena: &Arena) -> String {
    let mut out = String::new();
    if arena.rows.is_empty() {
        out.push_str(&format!("arena: no rows recorded yet ({HISTORY})\n"));
        out.push_str(
            "  a row is one evaluated geometric configuration; add them as they are measured\n",
        );
        return out;
    }
    out.push_str(&format!(
        "the regression arena - {} evaluated configuration(s)\n",
        arena.rows.len()
    ));
    out.push_str(&format!(
        "{:<26} {:<10} {:<16} {:<16} parent\n",
        "id", "milestone", "verdict", "confidence"
    ));
    out.push_str(&format!("{}\n", "-".repeat(90)));
    let mut rows: Vec<&Candidate> = arena.rows.iter().collect();
    // `sort_by` is stable, as Python's `sorted` is: rows sharing a date keep file order.
    rows.sort_by(|a, b| a.date.cmp(&b.date));
    for c in rows {
        out.push_str(&format!(
            "{:<26} {:<10} {:<16} {:<16} {}\n",
            c.id,
            c.milestone,
            c.verdict,
            c.confidence,
            c.parent.clone().unwrap_or_else(|| "-".to_string())
        ));
    }
    // Python: `str(arena.raw.get("limits", "")).strip()` - absent is empty, present-null is `None`.
    let limits = arena
        .raw
        .get("limits")
        .map(py_str_of)
        .unwrap_or_default()
        .trim()
        .to_string();
    out.push('\n');
    out.push_str(&format!(
        "limits: {}\n",
        if limits.is_empty() {
            "NOT STATED".to_string()
        } else {
            truncate(&limits, 300)
        }
    ));
    out.push('\n');
    out.push_str("compare one against its parent:  cli.py arena compare <id>\n");
    out
}

/// `arena board --json`.
///
/// **`scores` keeps the FILE's key order**, because the real data needs it: `addressed-read` writes
/// `wrong_value_failures` before `memory`, and `serde_json::Value` would sort that away. Everything
/// else is Python's `c.__dict__` — the dataclass declaration order, with absent fields normalised to
/// `""`/`null`/`{}`.
///
/// **`ensure_ascii` is ON here**, unlike `panel show`: this call is plain `json.dumps(..., indent=2)`,
/// whose default escapes non-ASCII.
pub fn board_json(arena: &Arena, repo: &Path) -> String {
    use crate::ordered::Ordered;
    let mut scores_by_id: Vec<(String, Ordered)> = Vec::new();
    if let Ok(text) = std::fs::read_to_string(repo.join(HISTORY)) {
        if let Ok(doc) = Ordered::parse(&text) {
            if let Some(list) = doc.get("candidates").and_then(Ordered::as_array) {
                for c in list {
                    if let Some(id) = c.get("id").and_then(Ordered::as_str) {
                        scores_by_id.push((
                            id.to_string(),
                            c.get("scores").cloned().unwrap_or(Ordered::Obj(Vec::new())),
                        ));
                    }
                }
            }
        }
    }
    let items: Vec<Ordered> = arena
        .rows
        .iter()
        .map(|c| {
            let scores = scores_by_id
                .iter()
                .find(|(id, _)| id == &c.id)
                .map(|(_, s)| s.clone())
                .unwrap_or(Ordered::Obj(Vec::new()));
            let parent = c.parent.clone().map(Ordered::Str).unwrap_or(Ordered::Null);
            Ordered::Obj(vec![
                ("id".to_string(), Ordered::Str(c.id.clone())),
                ("artifact".to_string(), Ordered::Str(c.artifact.clone())),
                ("milestone".to_string(), Ordered::Str(c.milestone.clone())),
                ("date".to_string(), Ordered::Str(c.date.clone())),
                ("parent".to_string(), parent),
                ("verdict".to_string(), Ordered::Str(c.verdict.clone())),
                ("confidence".to_string(), Ordered::Str(c.confidence.clone())),
                ("mechanism".to_string(), Ordered::Str(c.mechanism.clone())),
                ("panel".to_string(), Ordered::Str(c.panel.clone())),
                ("seeds".to_string(), ord(&c.seeds)),
                ("rows".to_string(), ord(&c.rows)),
                ("scope".to_string(), Ordered::Str(c.scope.clone())),
                ("scores".to_string(), scores),
                ("notes".to_string(), Ordered::Str(c.notes.clone())),
                ("record".to_string(), Ordered::Str(c.record.clone())),
            ])
        })
        .collect();
    let limits = arena
        .raw
        .get("limits")
        .cloned()
        .unwrap_or_else(|| Value::String(String::new()));
    let doc = Ordered::Obj(vec![
        ("candidates".to_string(), Ordered::Arr(items)),
        ("limits".to_string(), ord(&limits)),
    ]);
    crate::registries::ensure_ascii(&doc.to_pretty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, parent: Option<&str>, milestone: &str, verdict: &str) -> Candidate {
        Candidate {
            id: id.to_string(),
            parent: parent.map(str::to_string),
            milestone: milestone.to_string(),
            verdict: verdict.to_string(),
            ..Candidate::default()
        }
    }

    #[test]
    fn a_lineage_runs_from_the_root_down_to_the_candidate() {
        let arena = Arena {
            rows: vec![
                row("a", None, "m2", "KEEP"),
                row("b", Some("a"), "m2", "REJECT"),
                row("c", Some("b"), "m3", "NOT_RUN"),
            ],
            raw: Value::Null,
        };
        let chain = arena.lineage("c");
        let ids: Vec<&str> = chain.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
        assert_eq!(
            render_lineage(&chain),
            "a  (m2, KEEP)\n  b  (m2, REJECT)\n    c  (m3, NOT_RUN)\n"
        );
    }

    #[test]
    fn an_unknown_id_has_no_lineage_rather_than_a_panic() {
        let arena = Arena {
            rows: vec![row("a", None, "m2", "KEEP")],
            raw: Value::Null,
        };
        assert!(arena.lineage("nope").is_empty());
    }

    /// A parent chain that loops must stop, not hang: the arena file is hand-written JSON.
    #[test]
    fn a_cycle_stops_at_the_first_repeat() {
        let arena = Arena {
            rows: vec![
                row("a", Some("b"), "m2", "KEEP"),
                row("b", Some("a"), "m2", "KEEP"),
            ],
            raw: Value::Null,
        };
        let ids: Vec<String> = arena.lineage("a").iter().map(|c| c.id.clone()).collect();
        assert_eq!(ids.len(), 2, "{ids:?}");
    }

    #[test]
    fn a_missing_history_file_is_an_empty_arena() {
        let dir = std::env::temp_dir().join(format!("harness-arena-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch");
        let arena = load(&dir);
        assert!(arena.rows.is_empty());
        assert_eq!(arena.raw.get("schema").and_then(Value::as_i64), Some(1));
        // And an unreadable one answers the same way rather than failing the command. The parent
        // directories have to exist first: writing `docs/harness/arena.json` into an empty scratch
        // directory fails before the parse is ever reached, which is what the first version of this
        // test proved.
        std::fs::create_dir_all(dir.join("docs").join("harness")).expect("dirs");
        std::fs::write(dir.join(HISTORY), "{ not json").expect("write");
        assert!(load(&dir).rows.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn board_row(
        id: &str,
        date: &str,
        milestone: &str,
        verdict: &str,
        confidence: &str,
        parent: Option<&str>,
    ) -> Candidate {
        Candidate {
            id: id.to_string(),
            date: date.to_string(),
            milestone: milestone.to_string(),
            verdict: verdict.to_string(),
            confidence: confidence.to_string(),
            parent: parent.map(str::to_string),
            ..Candidate::default()
        }
    }

    #[test]
    fn the_board_lists_rows_by_date_not_by_rank() {
        let arena = Arena {
            rows: vec![
                board_row(
                    "later",
                    "2026-10-12",
                    "m2",
                    "REJECT",
                    "provisional",
                    Some("earlier"),
                ),
                board_row("earlier", "2026-10-10", "m1", "KEEP", "decisive", None),
            ],
            raw: serde_json::json!({"limits": "  a   stated   limit  "}),
        };
        let text = render_board(&arena);
        let first = text.find("earlier").expect("first row");
        let second = text.find("later").expect("second row");
        assert!(first < second, "date order, not file order: {text}");
        assert!(
            text.contains("the regression arena - 2 evaluated configuration(s)"),
            "{text}"
        );
        // `limits` collapses whitespace before it is printed.
        assert!(text.contains("limits: a stated limit"), "{text}");
        assert!(
            text.trim_end()
                .ends_with("compare one against its parent:  cli.py arena compare <id>"),
            "{text}"
        );
        // A root row prints `-` where its parent would be, in the last column.
        assert!(text.contains("decisive         -\n"), "{text}");
    }

    #[test]
    fn an_empty_arena_says_so_rather_than_printing_nothing() {
        let text = render_board(&Arena::default());
        assert_eq!(
            text,
            concat!(
                "arena: no rows recorded yet (docs/harness/arena.json)\n",
                "  a row is one evaluated geometric configuration; add them as they are measured\n"
            )
        );
    }

    /// **`scores` keeps the file's key order**, which is the reason `Ordered` exists: the real arena
    /// writes `wrong_value_failures` before `memory`, and a `BTreeMap` would print `memory` first.
    #[test]
    fn the_board_json_keeps_the_files_score_order() {
        let dir = std::env::temp_dir().join(format!("harness-arena-board-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("docs").join("harness")).expect("dirs");
        std::fs::write(
            dir.join(HISTORY),
            r#"{"limits": "L", "candidates": [{"id": "a-row", "date": "2026-10-10", "scores": {"z_metric": 1, "a_metric": 2}}]}"#,
        )
        .expect("write");
        let arena = load(&dir);
        let text = board_json(&arena, &dir);
        let z = text.find("z_metric").expect("z");
        let a = text.find("a_metric").expect("a");
        assert!(z < a, "file order, not sorted: {text}");
        assert!(
            text.contains("\"parent\": null"),
            "absent parent is null: {text}"
        );
        assert!(text.contains("\"limits\": \"L\""), "{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
