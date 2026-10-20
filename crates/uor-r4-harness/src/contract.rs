//! The frozen state-format contract, and a checker that holds state to it.
//!
//! `docs/harness/state-formats.json` is the interface between the harness and its persisted state.
//! This module enforces it, wired into `check` as `harness.state_formats`. (A Python checker,
//! `statecheck.py`, enforced the same file until it was removed; the Python mirror as a whole was
//! archived on 20 October 2026.)
//!
//! The contract's rules, and why each exists:
//!
//! - A key **without** `optional` must appear in every document of that format. That is the
//!   "required" half, and 766 ledger rows failed against an earlier version of the contract that
//!   got this wrong.
//! - A key **with** `optional` may legitimately be absent, so absence is not a violation. The type
//!   of an optional key comes only from documents that carry it — absence must never invent a type.
//! - A kind of `nullable` accepts anything; `mixed` means the contract observed more than one JSON
//!   type for the key and a reader must handle the union. `count` is the real example: a string in
//!   heartbeat receipts, a number in team receipts.
//! - An empty array matches any item shape. There is nothing to contradict the contract, and a
//!   legitimately empty list is not drift.

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// A JSON kind as the contract records it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Boolean,
    Number,
    String,
    Nullable,
    Object,
    Array,
    /// More than one JSON type observed for this key across records. Any of them is expected.
    Mixed,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Kind::Boolean => "boolean",
            Kind::Number => "number",
            Kind::String => "string",
            Kind::Nullable => "nullable",
            Kind::Object => "object",
            Kind::Array => "array",
            Kind::Mixed => "mixed",
        };
        f.write_str(s)
    }
}

/// The shape the contract expects at one point in a document.
#[derive(Debug, Clone, Deserialize)]
pub struct Shape {
    #[serde(rename = "type")]
    pub kind: Kind,
    /// Present for objects: the expected keys, each with its own shape.
    #[serde(default)]
    pub keys: BTreeMap<String, Shape>,
    /// Present for arrays: the expected shape of every element.
    #[serde(default)]
    pub items: Option<Box<Shape>>,
    /// For `mixed` only: the types actually observed for this key. A value outside this set is
    /// drift. Without it, `mixed` would accept anything - which is how an object passed for
    /// `count` in both implementations at once.
    #[serde(default)]
    pub types: Vec<Kind>,
    /// `true` when the key may be absent. Absent means required.
    #[serde(default)]
    pub optional: bool,
}

impl Shape {
    /// The kind this JSON value actually is, named as the contract names kinds.
    pub fn kind_of(value: &Value) -> Kind {
        match value {
            Value::Bool(_) => Kind::Boolean,
            Value::Number(_) => Kind::Number,
            Value::String(_) => Kind::String,
            Value::Null => Kind::Nullable,
            Value::Object(_) => Kind::Object,
            Value::Array(_) => Kind::Array,
        }
    }

    /// Build the observed shape of a value, to the same depth the contract uses.
    pub fn observe(value: &Value, depth: usize) -> Shape {
        let kind = Self::kind_of(value);
        let mut shape = Shape {
            kind: kind.clone(),
            keys: BTreeMap::new(),
            items: None,
            types: Vec::new(),
            optional: false,
        };
        match (kind, value) {
            (Kind::Object, Value::Object(map)) if depth < 3 => {
                for (k, v) in map {
                    shape.keys.insert(k.clone(), Self::observe(v, depth + 1));
                }
            }
            (Kind::Array, Value::Array(items)) => {
                if let Some(first) = items.first() {
                    shape.items = Some(Box::new(Self::observe(first, depth + 1)));
                }
            }
            _ => {}
        }
        shape
    }

    /// Compare an observed shape against this expected one, collecting every violation.
    ///
    /// This is a port of `statecheck.py::compare`, deliberately including its judgement calls, so
    /// the two implementations cannot disagree about what "matches the contract" means.
    pub fn check(&self, observed: &Shape, where_: &str, out: &mut Vec<Violation>) {
        if self.kind == Kind::Nullable {
            return;
        }
        if self.kind == Kind::Mixed {
            if !self.types.is_empty() && !self.types.contains(&observed.kind) {
                out.push(Violation {
                    location: where_.to_string(),
                    problem: format!(
                        "kind {} not in the recorded mixed types {:?}",
                        observed.kind, self.types
                    ),
                });
            }
            return;
        }
        if observed.kind != self.kind {
            out.push(Violation {
                location: where_.to_string(),
                problem: format!("kind changed {} -> {}", self.kind, observed.kind),
            });
            return;
        }
        match self.kind {
            Kind::Object => {
                for (key, expected) in &self.keys {
                    match observed.keys.get(key) {
                        None => {
                            if !expected.optional {
                                out.push(Violation {
                                    location: format!("{where_}.{key}"),
                                    problem: format!("key missing (was {})", expected.kind),
                                });
                            }
                        }
                        Some(actual) => expected.check(actual, &format!("{where_}.{key}"), out),
                    }
                }
            }
            Kind::Array => {
                // An empty array observed in the data matches any item shape.
                if let (Some(expected_items), Some(actual_items)) = (&self.items, &observed.items) {
                    expected_items.check(actual_items, &format!("{where_}[]"), out);
                }
            }
            _ => {}
        }
    }
}

/// One disagreement between state and the contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub location: String,
    pub problem: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.location, self.problem)
    }
}

/// One format in the contract.
#[derive(Debug, Clone, Deserialize)]
pub struct FormatSpec {
    /// A glob relative to the repository root, e.g. `docs/harness/ledger/*.jsonl`.
    pub pattern: String,
    #[serde(default)]
    pub files: usize,
    /// The shape of a whole document, for `.json` formats.
    #[serde(default)]
    pub shape: Option<Shape>,
    /// The shape of one line, for `.jsonl` formats.
    #[serde(default)]
    pub row: Option<Shape>,
    /// For `panels`: the shape of one panel inside the file's `panels` map.
    #[serde(default)]
    pub panel_shape: Option<Shape>,
    /// Keys every row must carry, whatever its kind.
    #[serde(default)]
    pub required_always: Vec<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// Result of checking one format.
#[derive(Debug, Clone)]
pub struct FormatReport {
    pub format: String,
    pub files: usize,
    pub rows: usize,
    pub violations: Vec<Violation>,
    pub absent: bool,
}

/// Everything the checker found.
#[derive(Debug, Clone)]
pub struct CheckReport {
    pub formats: Vec<FormatReport>,
    pub formats_checked: usize,
}

impl CheckReport {
    pub fn ok(&self) -> bool {
        self.formats.iter().all(|f| f.violations.is_empty())
    }

    pub fn total_violations(&self) -> usize {
        self.formats.iter().map(|f| f.violations.len()).sum()
    }
}

/// The frozen contract, loaded from `docs/harness/state-formats.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct Contract {
    pub schema: u32,
    #[serde(default)]
    pub note: String,
    pub formats: BTreeMap<String, FormatSpec>,
}

impl Contract {
    /// Load the contract from a repository root.
    pub fn load(repo: &Path) -> Result<Contract, String> {
        let path = Self::path(repo);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn path(repo: &Path) -> PathBuf {
        repo.join("docs").join("harness").join("state-formats.json")
    }

    /// Check every format in the contract against the state on disk.
    ///
    /// A format with no files present is reported as absent rather than failed: a fresh checkout
    /// with no claims and no presence is a legitimate state, and treating it as drift would make
    /// the checker useless exactly when someone first runs it.
    pub fn check(&self, repo: &Path) -> CheckReport {
        let mut formats = Vec::new();
        for (name, spec) in &self.formats {
            formats.push(spec.check(name, repo));
        }
        CheckReport {
            formats_checked: formats.len(),
            formats,
        }
    }
}

impl FormatSpec {
    fn check(&self, name: &str, repo: &Path) -> FormatReport {
        let files = expand(&self.pattern, repo);
        let mut report = FormatReport {
            format: name.to_string(),
            files: files.len(),
            rows: 0,
            violations: Vec::new(),
            absent: files.is_empty(),
        };
        for path in &files {
            let label = path
                .strip_prefix(repo)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            let text = match std::fs::read_to_string(path) {
                Ok(t) => t,
                Err(e) => {
                    report.violations.push(Violation {
                        location: label,
                        problem: format!("cannot read: {e}"),
                    });
                    continue;
                }
            };
            if self.pattern.ends_with(".jsonl") {
                self.check_jsonl(&text, &label, &mut report);
            } else {
                self.check_json(&text, &label, &mut report);
            }
        }
        report
    }

    fn check_jsonl(&self, text: &str, label: &str, report: &mut FormatReport) {
        let Some(row_shape) = &self.row else { return };
        for (i, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            report.rows += 1;
            let value: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(e) => {
                    report.violations.push(Violation {
                        location: format!("{label}:{}", i + 1),
                        problem: format!("not JSON ({e})"),
                    });
                    continue;
                }
            };
            let where_ = format!("{label}:{}", i + 1);
            row_shape.check(&Shape::observe(&value, 0), &where_, &mut report.violations);
            if let Value::Object(map) = &value {
                for key in &self.required_always {
                    if !map.contains_key(key) {
                        report.violations.push(Violation {
                            location: where_.clone(),
                            problem: format!("required key {key:?} missing"),
                        });
                    }
                }
            }
        }
    }

    fn check_json(&self, text: &str, label: &str, report: &mut FormatReport) {
        let value: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => {
                report.violations.push(Violation {
                    location: label.to_string(),
                    problem: format!("not JSON ({e})"),
                });
                return;
            }
        };
        // Panels hold many panels in one file, so each entry is checked against the panel shape.
        if let (Some(panel_shape), Some(panels)) = (
            &self.panel_shape,
            value.get("panels").and_then(|p| p.as_object()),
        ) {
            for (id, panel) in panels {
                panel_shape.check(
                    &Shape::observe(panel, 0),
                    &format!("{label}:{id}"),
                    &mut report.violations,
                );
            }
            return;
        }
        if let Some(shape) = &self.shape {
            shape.check(&Shape::observe(&value, 0), label, &mut report.violations);
        }
    }
}

/// Expand the contract's simple glob (`dir/*.ext`) without pulling in a glob crate.
///
/// The contract only ever uses `dir/*.ext` patterns. Supporting exactly that is honest: a pattern
/// this cannot handle yields no files, which is visible as an absent format rather than a silent
/// pass. If the contract ever needs more, this must grow with a test.
fn expand(pattern: &str, repo: &Path) -> Vec<PathBuf> {
    let full = repo.join(pattern);
    let Some(name) = full.file_name().and_then(|n| n.to_str()) else {
        return Vec::new();
    };
    let Some(dir) = full.parent() else {
        return Vec::new();
    };
    if !name.starts_with('*') {
        return if full.is_file() {
            vec![full]
        } else {
            Vec::new()
        };
    }
    let suffix = &name[1..];
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(suffix))
        })
        .collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn contract_loads() {
        let c = Contract::load(&repo()).expect("contract");
        assert!(c.schema >= 2, "schema version");
        assert!(c.formats.contains_key("ledger"), "ledger format present");
        assert!(
            c.formats.contains_key("milestones"),
            "milestones format present"
        );
    }

    /// The point of the port: this crate reads what Python wrote, and agrees with the contract.
    #[test]
    fn every_format_passes_on_the_real_state() {
        let c = Contract::load(&repo()).expect("contract");
        let report = c.check(&repo());
        assert!(
            report.formats_checked >= 8,
            "checked {} formats",
            report.formats_checked
        );
        assert!(
            report.ok(),
            "violations: {:#?}",
            report
                .formats
                .iter()
                .filter(|f| !f.violations.is_empty())
                .map(|f| (f.format.clone(), f.violations.clone()))
                .collect::<Vec<_>>()
        );
    }

    /// The ledger is the format with real polymorphism, so it is the one worth asserting on.
    #[test]
    fn the_ledger_reads_and_has_rows() {
        let c = Contract::load(&repo()).expect("contract");
        let report = c.check(&repo());
        let ledger = report
            .formats
            .iter()
            .find(|f| f.format == "ledger")
            .expect("ledger");
        assert!(ledger.rows > 0, "ledger rows: {}", ledger.rows);
        assert!(ledger.violations.is_empty(), "{:#?}", ledger.violations);
    }

    #[test]
    fn a_missing_required_key_is_a_violation() {
        let shape = Shape {
            kind: Kind::Object,
            keys: [(
                "ts".to_string(),
                Shape {
                    kind: Kind::String,
                    keys: BTreeMap::new(),
                    items: None,
                    types: Vec::new(),
                    optional: false,
                },
            )]
            .into_iter()
            .collect(),
            items: None,
            types: Vec::new(),
            optional: false,
        };
        let mut out = Vec::new();
        shape.check(
            &Shape::observe(&json!({"kind": "skill"}), 0),
            "row",
            &mut out,
        );
        assert_eq!(out.len(), 1, "{out:?}");
        assert!(out[0].problem.contains("key missing"), "{out:?}");
    }

    #[test]
    fn an_optional_key_may_be_absent() {
        let shape = Shape {
            kind: Kind::Object,
            keys: [(
                "note".to_string(),
                Shape {
                    kind: Kind::String,
                    keys: BTreeMap::new(),
                    items: None,
                    types: Vec::new(),
                    optional: true,
                },
            )]
            .into_iter()
            .collect(),
            items: None,
            types: Vec::new(),
            optional: false,
        };
        let mut out = Vec::new();
        shape.check(
            &Shape::observe(&json!({"kind": "skill"}), 0),
            "row",
            &mut out,
        );
        assert!(out.is_empty(), "{out:?}");
    }

    /// `count` is a string in heartbeat receipts and a number in team receipts. A port that typed
    /// it as one of them would fail on live data; `mixed` exists so it does not have to guess.
    #[test]
    fn a_mixed_kind_accepts_both_observed_types() {
        let shape = Shape {
            kind: Kind::Mixed,
            keys: BTreeMap::new(),
            items: None,
            types: Vec::new(),
            optional: false,
        };
        let mut out = Vec::new();
        shape.check(&Shape::observe(&json!("1/3"), 0), "count", &mut out);
        shape.check(&Shape::observe(&json!(1), 0), "count", &mut out);
        assert!(out.is_empty(), "{out:?}");
    }

    #[test]
    fn an_empty_array_matches_any_item_shape() {
        let shape = Shape {
            kind: Kind::Array,
            keys: BTreeMap::new(),
            items: Some(Box::new(Shape {
                kind: Kind::Object,
                keys: BTreeMap::new(),
                items: None,
                types: Vec::new(),
                optional: false,
            })),
            types: Vec::new(),
            optional: false,
        };
        let mut out = Vec::new();
        shape.check(&Shape::observe(&json!([]), 0), "bars", &mut out);
        assert!(out.is_empty(), "an empty list is not drift: {out:?}");
    }

    /// `mixed` means the recorded types, not "anything". Accepting everything here let an object
    /// pass for `count` in both implementations simultaneously -- a real reader cannot handle that,
    /// and the parity check is what surfaced it.
    #[test]
    fn mixed_rejects_a_type_it_never_observed() {
        let shape = Shape {
            kind: Kind::Mixed,
            keys: BTreeMap::new(),
            items: None,
            types: vec![Kind::String, Kind::Number],
            optional: false,
        };
        let mut out = Vec::new();
        shape.check(
            &Shape::observe(&json!({"unexpected": true}), 0),
            "count",
            &mut out,
        );
        assert_eq!(out.len(), 1, "an object is not a recorded type: {out:?}");
        assert!(
            out[0].problem.contains("not in the recorded mixed types"),
            "{out:?}"
        );
        out.clear();
        shape.check(&Shape::observe(&json!(3), 0), "count", &mut out);
        assert!(out.is_empty(), "a recorded type is fine: {out:?}");
    }

    #[test]
    fn a_kind_change_is_a_violation() {
        let shape = Shape {
            kind: Kind::String,
            keys: BTreeMap::new(),
            items: None,
            types: Vec::new(),
            optional: false,
        };
        let mut out = Vec::new();
        shape.check(&Shape::observe(&json!(7), 0), "count", &mut out);
        assert_eq!(out.len(), 1, "{out:?}");
        assert!(out[0].problem.contains("kind changed"), "{out:?}");
    }

    #[test]
    fn an_absent_format_is_reported_not_failed() {
        let spec = FormatSpec {
            pattern: "docs/harness/does-not-exist/*.json".to_string(),
            files: 0,
            shape: None,
            row: None,
            panel_shape: None,
            required_always: Vec::new(),
            note: None,
        };
        let report = spec.check("ghost", &repo());
        assert!(report.absent);
        assert!(report.violations.is_empty(), "absence is not drift");
    }
}
