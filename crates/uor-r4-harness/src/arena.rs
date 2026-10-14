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
}
