//! The registries: milestones, their panels, and the owner rules that apply to a run.
//!
//! This is the layer `prep new` and `status` both sit on, so it was blocking two modules at once.
//! It is a **view over files that stay authoritative** — `docs/harness/milestones.json`,
//! `docs/harness/panels/*.json` — not a second source of truth. Nothing here writes; the registry is
//! a cache of the milestone contract, and the live milestone issues on GitHub remain the authority
//! the reconciliation rule checks against.
//!
//! Ported deliberately as its own unit: `render_sheet` reads the milestone, its panel and the owner
//! rules, so porting the renderer without this would have meant inventing a second, quieter set of
//! accessors that could drift from Python's.

use std::path::PathBuf;

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::state::StatePaths;

/// A milestone's line: one acceptance number and its target.
#[derive(Debug, Clone, Default)]
pub struct Line {
    pub name: String,
    pub count: Value,
    pub of: Value,
}

impl Line {
    /// `name count/of`, as the milestone table renders it.
    pub fn counts(&self) -> String {
        format!(
            "{} {}/{}",
            self.name,
            fmt_num(&self.count),
            fmt_num(&self.of)
        )
    }
}

/// One milestone from the registry.
#[derive(Debug, Clone, Default)]
pub struct Milestone {
    pub key: String,
    pub issue: i64,
    pub id: String,
    pub title: String,
    pub lab: String,
    pub status: String,
    pub headline: Value,
    pub secondary: Vec<Value>,
    pub panel: String,
    pub base_artifact: String,
    pub mechanisms: Vec<String>,
    pub lines: Vec<Line>,
    pub next: String,
}

impl Milestone {
    /// `M1 #2029` — how a milestone is cited in a sheet header.
    pub fn slug(&self) -> String {
        format!("{} #{}", self.id, self.issue)
    }

    /// The headline line the acceptance panel is scored against.
    ///
    /// Three forms, and all three are live: a **vacuous** headline ("not qualified", because a bar
    /// that cannot be failed is not a bar), a **qualitative** one (a yes/no proposition that only
    /// becomes a number once the milestone is satisfied), and a **numeric** one. Rendering the wrong
    /// form would misstate the bar a run is scored against, which is the one number a
    /// pre-registration exists to fix.
    ///
    /// The D23 interlock scans for one word that a real M4 headline contains, so this comment does
    /// not quote that headline: the ratchet counts occurrences in executable source and cannot tell a
    /// mention from an implementation. Worth knowing before quoting a banned mechanism in Rust.
    pub fn headline_line(&self) -> String {
        let h = &self.headline;
        if h.get("vacuous").and_then(Value::as_bool).unwrap_or(false) {
            return format!("{}: not qualified", str_field(h, "label"));
        }
        if let Some(text) = h.get("text").and_then(Value::as_str) {
            if !text.is_empty() {
                let target = h
                    .get("target_display")
                    .filter(|v| !v.is_null())
                    .unwrap_or_else(|| h.get("target").unwrap_or(&Value::Null));
                return format!("{}: {} -> target {}", self.id, text, fmt_num(target));
            }
        }
        // A suffix appears when `of` is present and non-empty. Python treats a falsy `of` as absent,
        // and the registry holds numbers, strings and nulls here; `is_none_or` on a number reads as
        // "not a string", which silently dropped the `/232` — the denominator is part of the bar, so
        // dropping it changes what the run is scored against.
        let suffix = match h.get("of") {
            None | Some(Value::Null) => String::new(),
            Some(Value::String(s)) if s.is_empty() => String::new(),
            Some(v) => format!("/{}", fmt_num(v)),
        };
        format!(
            "{}: {} {}{} -> target {}{}",
            self.id,
            str_field(h, "label"),
            fmt_num(h.get("current").unwrap_or(&Value::Null)),
            suffix,
            fmt_num(h.get("target").unwrap_or(&Value::Null)),
            suffix
        )
    }

    pub fn line_counts(&self) -> String {
        self.lines
            .iter()
            .map(Line::counts)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Numbers render as integers when they are whole — `37`, not `37.0`. A target written `116.0` reads
/// as a different kind of claim from `116`, and the registry holds both ints and floats.
pub fn fmt_num(v: &Value) -> String {
    match v {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                // `{:g}`: trim trailing zeros, as the Python side does.
                let s = format!("{f}");
                if let Some(stripped) = s.strip_suffix(".0") {
                    stripped.to_string()
                } else {
                    s
                }
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        Value::Null => "MISSING".to_string(),
        other => other.to_string(),
    }
}

fn str_field(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or("MISSING")
        .to_string()
}

/// One acceptance bar of a panel.
#[derive(Debug, Clone, Default)]
pub struct Bar {
    pub id: String,
    pub label: String,
    pub current: Value,
    pub target: Value,
    pub of: Value,
    pub unit: String,
    pub direction: String,
    /// A bar may state its OWN noise, which then beats the panel's. Python keeps the raw dict and
    /// reads it in `classify`; the field is here because `panel score` needs it.
    pub noise: String,
}

/// A frozen acceptance panel.
#[derive(Debug, Clone, Default)]
pub struct Panel {
    pub id: String,
    pub title: String,
    pub milestone: String,
    pub frozen: bool,
    pub grader: String,
    pub bars: Vec<Bar>,
    pub noise: String,
    pub seeds: String,
    pub rows: String,
    /// `raw["source"]` — the provenance line `panel show` prints.
    ///
    /// **`Option`, because Python's `raw.get("source", MISSING)` distinguishes ABSENT from EMPTY**:
    /// a panel with no `source` key prints `MISSING`, and one with `"source": ""` prints nothing.
    /// A `String` would collapse the two and print `MISSING` for both.
    pub source: Option<String>,
}

impl Panel {
    /// The bars, as a sheet lists them. A bar's declared direction is carried through: the arena
    /// lesson is that an undeclared direction lets a regression read as a gain.
    pub fn lines(&self) -> Vec<String> {
        if self.bars.is_empty() {
            return vec!["  - MISSING: the panel has no bars".to_string()];
        }
        self.bars
            .iter()
            .map(|b| {
                let label = if b.label.is_empty() {
                    b.id.clone()
                } else {
                    b.label.clone()
                };
                let suffix = if b.of.is_null() || b.of.as_str().is_some_and(str::is_empty) {
                    String::new()
                } else {
                    format!("/{}", fmt_num(&b.of))
                };
                let tail = if b.direction.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", b.direction)
                };
                format!(
                    "  - {}: {} {}{} -> {}{} {}{}",
                    b.id,
                    label,
                    fmt_num(&b.current),
                    suffix,
                    fmt_num(&b.target),
                    suffix,
                    b.unit,
                    tail
                )
                .trim_end()
                .to_string()
            })
            .collect()
    }
}

/// The whole registry: the milestone table, the owner rules, and the panels.
#[derive(Debug, Clone, Default)]
pub struct Registries {
    /// The registry's own top-level note. Rides on the status source line.
    pub note: String,
    pub tracker: Value,
    pub owner_rules: Vec<Value>,
    pub milestones: Vec<Milestone>,
    pub panels: Vec<Panel>,
    /// Owner decisions, parsed from `DECISIONS.md` (a view; the file is authoritative).
    pub decisions: Vec<Decision>,
}

impl Registries {
    /// Load every registry, reporting the first file that cannot be read.
    pub fn load(paths: &StatePaths) -> Result<Registries, String> {
        let ms_path = paths
            .repo
            .join("docs")
            .join("harness")
            .join("milestones.json");
        let text = std::fs::read_to_string(&ms_path)
            .map_err(|e| format!("cannot read {}: {e}", ms_path.display()))?;
        let data: Value =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", ms_path.display()))?;

        let mut milestones = Vec::new();
        if let Some(map) = data.get("milestones").and_then(Value::as_object) {
            for (key, raw) in map {
                milestones.push(milestone_from(key, raw)?);
            }
        }
        // Ordered by issue, which is the order the milestone table reads in.
        milestones.sort_by_key(|m| m.issue);

        Ok(Registries {
            note: data
                .get("note")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            tracker: data.get("tracker").cloned().unwrap_or(Value::Null),
            owner_rules: data
                .get("owner_rules")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            milestones,
            panels: load_panels(paths),
            decisions: load_decisions(paths),
        })
    }

    /// The TRACKER issue — the roadmap issue itself, where the PM posts assignments and CONTESTs.
    ///
    /// It is not a milestone: it carries no headline and no panel, so it is absent from the
    /// milestone table. It is still the busiest issue in the project, and a watcher that reads only
    /// `milestones` misses exactly the comments that task a lab.
    pub fn tracker_issue(&self) -> Option<i64> {
        self.tracker.get("issue").and_then(Value::as_i64)
    }

    /// A milestone by key, id, issue number or `#issue` — the same aliases a lab types.
    pub fn milestone(&self, key: &str) -> Option<&Milestone> {
        let k = key.trim().to_ascii_lowercase();
        self.milestones
            .iter()
            .find(|m| m.key.to_ascii_lowercase() == k)
            .or_else(|| {
                self.milestones.iter().find(|m| {
                    m.id.to_ascii_lowercase() == k
                        || m.issue.to_string() == k
                        || format!("#{}", m.issue) == k
                })
            })
    }

    pub fn milestones_by_lab(&self, lab: &str) -> Vec<&Milestone> {
        let l = lab.to_ascii_lowercase();
        self.milestones
            .iter()
            .filter(|m| m.lab.to_ascii_lowercase() == l)
            .collect()
    }

    /// The panel a milestone names. Exact id first, then a containment match, as Python does — the
    /// registry has historically carried `v5-reply` and `m2-reply512` for the same idea.
    pub fn panel_for(&self, milestone: &Milestone) -> Option<&Panel> {
        let key = milestone.panel.as_str();
        if key.is_empty() {
            return None;
        }
        self.panels.iter().find(|p| p.id == key).or_else(|| {
            self.panels
                .iter()
                .find(|p| key.contains(&p.id) || p.id.contains(key))
        })
    }

    /// The owner rule text for an id, or empty when the registry does not carry it.
    pub fn rule(&self, id: &str) -> String {
        self.owner_rules
            .iter()
            .find(|r| r.get("id").and_then(Value::as_str) == Some(id))
            .and_then(|r| r.get("rule"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    }
}

fn milestone_from(key: &str, raw: &Value) -> Result<Milestone, String> {
    let issue = raw
        .get("issue")
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("milestone {key:?} has no integer issue"))?;
    let lines = raw
        .get("lines")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|l| Line {
                    name: l
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    count: l.get("count").cloned().unwrap_or(Value::Null),
                    of: l.get("of").cloned().unwrap_or(Value::Null),
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Milestone {
        key: key.to_string(),
        issue,
        id: raw
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        title: raw
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        lab: raw
            .get("lab")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        status: raw
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        headline: raw.get("headline").cloned().unwrap_or(Value::Null),
        secondary: raw
            .get("secondary")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        panel: raw
            .get("panel")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        base_artifact: raw
            .get("base_artifact")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        mechanisms: raw
            .get("mechanisms")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        lines,
        next: raw
            .get("next")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
    })
}

fn load_panels(paths: &StatePaths) -> Vec<Panel> {
    let dir = paths.repo.join("docs").join("harness").join("panels");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let Some(map) = data.get("panels").and_then(Value::as_object) else {
            continue;
        };
        for (pid, raw) in map {
            out.push(Panel {
                id: pid.clone(),
                title: raw
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or(pid)
                    .to_string(),
                milestone: raw
                    .get("milestone")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                frozen: raw.get("frozen").and_then(Value::as_bool).unwrap_or(false),
                grader: raw
                    .get("grader")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                bars: raw
                    .get("bars")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .map(|b| Bar {
                                id: b
                                    .get("id")
                                    .and_then(Value::as_str)
                                    .unwrap_or("?")
                                    .to_string(),
                                label: b
                                    .get("label")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_string(),
                                current: b.get("current").cloned().unwrap_or(Value::Null),
                                target: b.get("target").cloned().unwrap_or(Value::Null),
                                of: b.get("of").cloned().unwrap_or(Value::Null),
                                unit: b
                                    .get("unit")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_string(),
                                direction: b
                                    .get("direction")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_string(),
                                noise: b
                                    .get("noise")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_string(),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                noise: raw
                    .get("noise")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                seeds: raw
                    .get("seeds")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                rows: raw
                    .get("rows")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                source: raw
                    .get("source")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn regs() -> Registries {
        let paths = StatePaths::resolve(repo(), None, Some("/nonexistent-live"));
        Registries::load(&paths).expect("the registry loads")
    }

    #[test]
    fn the_real_registry_loads_with_every_milestone() {
        let r = regs();
        assert!(
            r.milestones.len() >= 8,
            "found {} milestones",
            r.milestones.len()
        );
        assert!(
            !r.owner_rules.is_empty(),
            "the owner rules are part of the registry"
        );
        assert!(!r.panels.is_empty(), "the panels are part of the registry");
        for m in &r.milestones {
            assert!(!m.id.is_empty(), "{}: no id", m.key);
            assert!(m.issue > 0, "{}: issue {}", m.key, m.issue);
        }
    }

    /// The aliases a lab actually types.
    #[test]
    fn a_milestone_is_found_by_key_id_issue_and_hash() {
        let r = regs();
        let m = r.milestone("m1").expect("m1 by key");
        assert_eq!(
            r.milestone("M1").map(|x| x.key.as_str()),
            Some(m.key.as_str()),
            "case-insensitive"
        );
        assert_eq!(
            r.milestone(&m.issue.to_string()).map(|x| x.key.as_str()),
            Some(m.key.as_str()),
            "by bare issue"
        );
        assert_eq!(
            r.milestone(&format!("#{}", m.issue))
                .map(|x| x.key.as_str()),
            Some(m.key.as_str()),
            "by #issue"
        );
        assert!(r.milestone("nope").is_none());
    }

    #[test]
    fn milestones_are_ordered_by_issue() {
        let r = regs();
        let issues: Vec<i64> = r.milestones.iter().map(|m| m.issue).collect();
        let mut sorted = issues.clone();
        sorted.sort_unstable();
        assert_eq!(issues, sorted, "the milestone table reads in issue order");
    }

    /// Every headline form is live in this registry, and rendering the wrong one would misstate the
    /// bar a run is scored against — the one number a pre-registration exists to fix.
    #[test]
    fn all_three_headline_forms_render() {
        let numeric = Milestone {
            id: "M1".into(),
            headline: serde_json::json!({"label": "reply acceptable", "current": 37, "target": 116, "of": 232}),
            ..Default::default()
        };
        assert_eq!(
            numeric.headline_line(),
            "M1: reply acceptable 37/232 -> target 116/232"
        );

        let qualitative = Milestone {
            id: "M4".into(),
            // A qualitative headline: a word and a yes/no state, with a target that is also a word.
            // Deliberately not the M4 text, which would add two occurrences to the D23 ratchet.
            headline: serde_json::json!({"label": "runtime compliance", "text": "yes", "target": "no"}),
            ..Default::default()
        };
        assert_eq!(qualitative.headline_line(), "M4: yes -> target no");

        let vacuous = Milestone {
            id: "M2".into(),
            headline: serde_json::json!({"label": "complete correct replies", "vacuous": true}),
            ..Default::default()
        };
        assert_eq!(
            vacuous.headline_line(),
            // No id prefix, and that is Python's behaviour too: a vacuous headline reads as a
            // sentence stating the bar is not qualified, not as a labelled number. My first
            // assertion expected "M2: ..." and was wrong - checked against the real registry
            // output rather than assumed.
            "complete correct replies: not qualified"
        );
    }

    /// `37.0` reads as a different kind of claim from `37`, and the registry holds both types.
    #[test]
    fn whole_floats_render_as_integers() {
        assert_eq!(fmt_num(&serde_json::json!(37)), "37");
        assert_eq!(fmt_num(&serde_json::json!(37.0)), "37");
        assert_eq!(fmt_num(&serde_json::json!(1.5)), "1.5");
        assert_eq!(fmt_num(&serde_json::json!("yes")), "yes");
        assert_eq!(fmt_num(&Value::Null), "MISSING");
    }

    #[test]
    fn a_milestone_resolves_its_panel() {
        let r = regs();
        let m = r.milestone("m1").expect("m1");
        if m.panel.is_empty() {
            return; // a milestone may legitimately name none
        }
        let p = r.panel_for(m).expect("the named panel is in the registry");
        assert!(!p.id.is_empty());
        assert!(!p.lines().is_empty(), "a panel renders at least one line");
    }

    #[test]
    fn owner_rule_text_is_readable_by_id() {
        let r = regs();
        for id in ["D21", "D22", "D23", "D24"] {
            let text = r.rule(id);
            assert!(
                !text.is_empty(),
                "{id} is in APPLICABLE_RULES, so its text must be in the registry"
            );
        }
        assert_eq!(
            r.rule("D99"),
            "",
            "an unknown rule reads empty, not an error"
        );
    }

    #[test]
    fn a_panel_with_no_bars_says_so_rather_than_rendering_nothing() {
        let p = Panel {
            id: "x".into(),
            ..Default::default()
        };
        assert_eq!(p.lines(), vec!["  - MISSING: the panel has no bars"]);
    }

    /// A missing registry is an error, not an empty success: `status` and `prep` both read it, and
    /// silently returning nothing would make every milestone look absent.
    #[test]
    fn a_missing_registry_is_an_error() {
        let paths = StatePaths::resolve("/nonexistent-repo", None, Some("/nonexistent-live"));
        assert!(Registries::load(&paths).is_err());
    }
}

// --------------------------------------------------------------------------- decisions
//
// `DECISIONS.md` headings parsed into a view. The FILE stays authoritative - this is a reader, and
// the brief cites each decision by the line number it came from so a lab can go and read the source
// rather than trusting the excerpt.

/// One owner decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub id: String,
    pub title: String,
    /// 1-based line of the heading in `DECISIONS.md`.
    pub line: usize,
    pub body: String,
    pub date: String,
}

/// `## D21 — title`, `## D21 - title` or `## D21: title`.
///
/// The separator varies in the file, so all three are accepted; a decision whose heading uses one
/// the parser does not know would silently vanish from every brief that cites it.
fn is_decision_heading(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("## ")?;
    let rest = rest.trim();
    // The id is `D` followed by digits, optionally one lowercase letter (`D0-b` style is not used,
    // but `D21a` is).
    let mut chars = rest.char_indices();
    let mut end = 0;
    for (i, c) in chars.by_ref() {
        if i == 0 && c != 'D' {
            return None;
        }
        if i == 0 {
            continue;
        }
        if c.is_ascii_digit() {
            end = i + c.len_utf8();
            continue;
        }
        if c.is_ascii_lowercase() && end > 0 {
            end = i + c.len_utf8();
        }
        break;
    }
    if end == 0 {
        return None;
    }
    let id = rest[..end].to_string();
    let tail = rest[end..].trim_start();
    let title = tail
        .strip_prefix('—')
        .or_else(|| tail.strip_prefix("--"))
        .or_else(|| tail.strip_prefix('-'))
        .or_else(|| tail.strip_prefix(':'))
        .unwrap_or(tail)
        .trim()
        .to_string();
    if title.is_empty() {
        return None;
    }
    Some((id, title))
}

/// Parse `DECISIONS.md` into decisions. Empty when the file is absent, which is not an error: a
/// checkout without it has no decisions to cite.
pub fn load_decisions(paths: &StatePaths) -> Vec<Decision> {
    let path = paths
        .repo
        .join("docs")
        .join("integration")
        .join("DECISIONS.md");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    parse_decisions(&text)
}

/// Split `DECISIONS.md` text into decisions.
pub fn parse_decisions(text: &str) -> Vec<Decision> {
    let lines: Vec<&str> = text.lines().collect();
    let mut marks: Vec<(usize, String, String)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if let Some((id, title)) = is_decision_heading(line) {
            marks.push((i, id, title));
        }
    }
    let mut out = Vec::with_capacity(marks.len());
    for (idx, (line_no, id, title)) in marks.iter().enumerate() {
        let end = marks
            .get(idx + 1)
            .map(|(n, _, _)| *n)
            .unwrap_or(lines.len());
        let body = lines[line_no + 1..end].join("\n").trim().to_string();
        // A date near the top of the body, as the file writes it (`13 October 2026`).
        let date = body
            .chars()
            .take(400)
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .windows(3)
            .find(|w| {
                w[0].chars().next().is_some_and(|c| c.is_ascii_digit())
                    && w[2].len() == 4
                    && w[2].starts_with("20")
                    && w[1].chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            })
            .map(|w| w.join(" "))
            .unwrap_or_default();
        out.push(Decision {
            id: id.clone(),
            title: title.clone(),
            line: line_no + 1,
            body,
            date,
        });
    }
    out
}

impl Registries {
    /// A decision by id, case-insensitively.
    pub fn decision(&self, key: &str) -> Option<&Decision> {
        let k = key.trim().to_ascii_uppercase();
        self.decisions
            .iter()
            .find(|d| d.id.to_ascii_uppercase() == k)
    }

    /// Decisions that plausibly govern a milestone: the owner rules' own ids first, then any decision
    /// whose body names the milestone's issue or opens with its id.
    ///
    /// Two passes because they answer different questions. The first is "which rules are in force",
    /// which is exact. The second is "what else talks about this milestone", which is a search and
    /// therefore looser - so it only adds, and never displaces a named rule.
    pub fn decisions_for(&self, milestone: Option<&Milestone>) -> Vec<&Decision> {
        let mut out: Vec<&Decision> = Vec::new();
        for rule in &self.owner_rules {
            let Some(id) = rule.get("id").and_then(Value::as_str) else {
                continue;
            };
            if let Some(d) = self.decision(id) {
                if !out.iter().any(|x| x.id == d.id) {
                    out.push(d);
                }
            }
        }
        if let Some(m) = milestone {
            let issue = format!("#{}", m.issue);
            for d in &self.decisions {
                if out.iter().any(|x| x.id == d.id) {
                    continue;
                }
                let head: String = d.body.chars().take(200).collect();
                if d.body.contains(&issue) || head.contains(&m.id) {
                    out.push(d);
                }
            }
        }
        out
    }
}

// ------------------------------------------------------------------------------------- roles

/// One dispatchable role from `docs/harness/roles.json`.
///
/// **The field order is not a style choice.** `team roster --json` serialises Python's
/// `r.__dict__`, whose order is the dataclass declaration order, so these fields are declared in
/// that order and `Serialize` is derived: a struct that declared them differently would emit the
/// same JSON with its keys rearranged. `route` is a method rather than a field for the same
/// reason - Python's `route` is a property and does not appear in `__dict__` at all.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Role {
    pub id: String,
    pub title: String,
    pub seat: String,
    pub provider: String,
    pub model: String,
    pub reasoning: String,
    pub purpose: String,
    pub skills: Vec<String>,
    pub when: Vec<String>,
    pub read_only: bool,
}

impl Role {
    /// `provider/model`, or `(inherit)` when the role names no provider.
    ///
    /// The fallback is a decision rather than a blank column: a role with no route takes the
    /// caller's, and an empty cell would read as a missing value instead of a choice.
    pub fn route(&self) -> String {
        if self.provider.is_empty() {
            "(inherit)".to_string()
        } else {
            format!("{}/{}", self.provider, self.model)
        }
    }
}

/// A JSON object with its key order intact.
///
/// **This exists because `serde_json`'s default `Value` is a `BTreeMap`.** Reading `roles.json`
/// into a `Value` sorts the roles by id - which is what the previous attempt at this command
/// shipped and then reverted: it printed `delivery-reviewer` before `experiment-designer` where
/// Python prints the file's order, and an alphabetical roster looks entirely plausible. A map
/// visitor is handed the entries in document order, which is the order a Python `dict` keeps, so
/// this collects them instead of sorting them.
struct OrderedObject(Vec<(String, Value)>);

impl<'de> Deserialize<'de> for OrderedObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Collect;

        impl<'de> Visitor<'de> for Collect {
            type Value = OrderedObject;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a JSON object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<OrderedObject, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut out = Vec::new();
                while let Some((key, value)) = map.next_entry::<String, Value>()? {
                    out.push((key, value));
                }
                Ok(OrderedObject(out))
            }
        }

        deserializer.deserialize_map(Collect)
    }
}

/// `docs/harness/roles.json` for a repository.
pub fn roles_path(paths: &StatePaths) -> PathBuf {
    paths.repo.join("docs").join("harness").join("roles.json")
}

#[derive(Deserialize)]
struct RolesFile {
    /// `None` when the file has no `roles` key or writes it as `null`; Python's
    /// `data.get("roles") or {}` treats both as an empty registry.
    roles: Option<OrderedObject>,
}

/// Every dispatchable role, **in the file's own order**.
///
/// A missing file is an empty roster, as Python's `if not paths.roles_json.exists(): return []`.
/// A file that exists but cannot be read is an **error** rather than an empty roster: `0
/// dispatchable roles` is a wrong answer that reads like a right one, and Python raises rather
/// than printing it.
pub fn load_roles(paths: &StatePaths) -> Result<Vec<Role>, String> {
    let path = roles_path(paths);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let doc: RolesFile =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(doc
        .roles
        .map(|ordered| {
            ordered
                .0
                .into_iter()
                .map(|(id, raw)| role_from(&id, &raw))
                .collect()
        })
        .unwrap_or_default())
}

/// Python's `find_role`: an exact id first, then the first prefix match.
pub fn find_role<'a>(roles: &'a [Role], key: &str) -> Option<&'a Role> {
    roles
        .iter()
        .find(|r| r.id == key)
        .or_else(|| roles.iter().find(|r| r.id.starts_with(key)))
}

/// `team roster`, as Python prints it.
///
/// **One newline after the header, and the bytes are the reason.** The previous attempt's record
/// says Python's header is followed by a blank line; `od -c` on the live command says the header
/// string already ends in `\n` and Python's `emit` only appends one when it is missing, so the
/// correct output is a single newline. The record was wrong and the measurement is what a lab
/// diffs against.
pub fn render_roster(roles: &[Role]) -> String {
    let mut out = format!(
        "{} dispatchable roles (docs/harness/roles.json)\n",
        roles.len()
    );
    for r in roles {
        let ro = if r.read_only { " [read-only]" } else { "" };
        out.push_str(&format!(
            "  {:<24} {:<11} {:<32} {}{ro}\n",
            r.id,
            r.seat,
            r.route(),
            r.title
        ));
        out.push_str(&format!("      {}\n", py_slice(&r.purpose, 100)));
    }
    out
}

#[derive(Serialize)]
struct RosterDoc<'a> {
    roles: &'a [Role],
}

/// `team roster --json`: `{"roles": [r.__dict__ for r in roles]}` at `indent=2`.
///
/// Serialised from a real struct rather than through `serde_json::json!`: the macro routes values
/// through `to_value`, which is the same `BTreeMap` as above and would sort every role's keys -
/// the loader's order trap, one layer down.
pub fn roster_json(roles: &[Role]) -> String {
    ensure_ascii(&serde_json::to_string_pretty(&RosterDoc { roles }).unwrap_or_default())
}

/// Python's `json.dumps(..., ensure_ascii=True)`, which is the DEFAULT and which `serde_json` does
/// not do.
///
/// **Found by diffing, not by reading.** The pretty output matched Python byte for byte except for
/// 12 bytes: three role purposes contain an em dash, Python wrote `\u2014` and this wrote the raw
/// character. `ensure_ascii` is on by default in `json.dumps`, so every non-ASCII character in the
/// registry is escaped on Python's side, and a port that emitted valid UTF-8 instead would be
/// producing different bytes for the same value - which is what a lab diffs.
///
/// Applying it to the whole document is safe because **JSON's structure is ASCII**: every
/// non-ASCII character in the output is inside a string literal, so there is nothing else to
/// escape and no parser is needed to find them.
///
/// Astral characters become a surrogate PAIR, as Python writes them.
///
/// **Shared with `receipt::Recorded::to_pretty`**, which renders `json.dumps(record, indent=2)` for
/// `team dispatch --json`: both are Python-JSON renderers and both needed the same fix, so there is
/// one implementation rather than two that could drift.
pub fn ensure_ascii(json: &str) -> String {
    let mut out = String::with_capacity(json.len());
    for ch in json.chars() {
        let cp = ch as u32;
        if cp < 0x80 {
            out.push(ch);
        } else if cp <= 0xFFFF {
            out.push_str(&format!("\\u{cp:04x}"));
        } else {
            let v = cp - 0x1_0000;
            out.push_str(&format!(
                "\\u{:04x}\\u{:04x}",
                0xD800 + (v >> 10),
                0xDC00 + (v & 0x3FF)
            ));
        }
    }
    out
}

fn role_from(id: &str, raw: &Value) -> Role {
    // Python: `route = raw.get("route") or {}`. A route that is present but falsy (null, "", 0,
    // an empty object) is the same as no route at all.
    let route = raw.get("route").filter(|v| truthy(v));
    let route_field = |key: &str| {
        route
            .and_then(|r| r.get(key))
            .map(py_str)
            .unwrap_or_default()
    };
    Role {
        id: id.to_string(),
        title: field(raw, "title", id),
        seat: field(raw, "seat", "specialist"),
        provider: route_field("provider"),
        model: route_field("model"),
        reasoning: route_field("reasoning"),
        purpose: field(raw, "purpose", ""),
        skills: as_list(raw.get("skills")),
        when: as_list(raw.get("when")),
        read_only: raw.get("read_only").is_some_and(truthy),
    }
}

/// Python's `str(raw.get(key, default))`: the default only when the key is **absent**, so an
/// explicit `null` renders `None` rather than falling back.
fn field(raw: &Value, key: &str, default: &str) -> String {
    match raw.get(key) {
        Some(v) => py_str(v),
        None => default.to_string(),
    }
}

/// Python's `str()` for the scalar JSON types.
///
/// **Limit, stated rather than hidden:** a list or object in a text field would render as Python's
/// `repr`, and this returns JSON text instead. Every text field in the real registry is a string,
/// and `role_text_fields_are_scalars` asserts that on the file itself, so the limit cannot be
/// crossed without a failure.
fn py_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "None".to_string(),
        Value::Bool(true) => "True".to_string(),
        Value::Bool(false) => "False".to_string(),
        Value::Number(n) => py_number(n),
        other => other.to_string(),
    }
}

/// `str()` of a JSON number. Not `fmt_num`: the milestone table trims a trailing `.0` because a
/// target written `116.0` is a different claim from `116`, and Python's `str` does not trim it.
fn py_number(n: &serde_json::Number) -> String {
    if let Some(i) = n.as_i64() {
        return i.to_string();
    }
    if let Some(u) = n.as_u64() {
        return u.to_string();
    }
    match n.as_f64() {
        Some(f) if f.is_finite() && f.fract() == 0.0 => format!("{f:.1}"),
        Some(f) => format!("{f}"),
        None => n.to_string(),
    }
}

/// `_as_list`: `None` and `""` are the empty list, a list is mapped through `str()`, and anything
/// else becomes a one-element list - so a bare string still renders as one entry.
fn as_list(v: Option<&Value>) -> Vec<String> {
    match v {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::String(s)) if s.is_empty() => Vec::new(),
        Some(Value::Array(items)) => items.iter().map(py_str).collect(),
        Some(other) => vec![py_str(other)],
    }
}

/// Python's truthiness, as `bool(raw.get(...))` reads it.
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Python's `s[:n]`, which counts **characters**: a byte slice would panic or split a role's
/// purpose mid-character the moment one carried non-ASCII.
fn py_slice(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

#[cfg(test)]
mod role_tests {
    use super::*;
    use std::path::Path;

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn paths_for(dir: &Path) -> StatePaths {
        StatePaths::resolve(dir, None, Some("/nonexistent-live"))
    }

    /// A throwaway checkout holding one `docs/harness/roles.json`.
    fn with_roles(json: &str) -> (tempfile::TempDir, Vec<Role>) {
        let dir = tempfile::tempdir().expect("tempdir");
        let harness = dir.path().join("docs").join("harness");
        std::fs::create_dir_all(&harness).expect("mkdir");
        std::fs::write(harness.join("roles.json"), json).expect("write");
        let roles = load_roles(&paths_for(dir.path())).expect("roles load");
        (dir, roles)
    }

    /// **The order is the file order, and this fixture is deliberately not alphabetical.**
    ///
    /// The previous attempt read `roles.json` as a `serde_json::Value`, which is a `BTreeMap`, and
    /// printed the roles sorted by id. That is a wrong answer that reads like a right one, so the
    /// assertion is not only "these three in this order" but "and that order is not sorted".
    #[test]
    fn roles_keep_the_files_order_rather_than_sorting_by_id() {
        let (_d, roles) = with_roles(
            r#"{"schema": 1, "note": "n", "roles": {
                 "zulu-role": {"title": "Z", "route": {"provider": "p", "model": "m"}, "read_only": true},
                 "alpha-role": {"title": "A"},
                 "mike-role": {}}}"#,
        );
        let ids: Vec<&str> = roles.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, vec!["zulu-role", "alpha-role", "mike-role"]);
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_ne!(ids, sorted, "the fixture must not be alphabetical");
    }

    #[test]
    fn role_defaults_and_route_match_python() {
        let (_d, roles) = with_roles(
            r#"{"roles": {
                 "zulu-role": {"title": "Z", "seat": "planner", "route": {"provider": "p", "model": "m"}, "skills": ["a"], "when": ["b"], "read_only": true},
                 "alpha-role": {"title": "A"},
                 "mike-role": {}}}"#,
        );
        let zulu = &roles[0];
        assert_eq!(zulu.route(), "p/m");
        assert!(zulu.read_only);
        assert_eq!(zulu.skills, vec!["a"]);
        assert_eq!(zulu.when, vec!["b"]);
        // No route at all is `(inherit)`, not a blank column.
        assert_eq!(roles[1].route(), "(inherit)");
        // `title` falls back to the id and `seat` to `specialist`.
        assert_eq!(roles[2].title, "mike-role");
        assert_eq!(roles[2].seat, "specialist");
        assert_eq!(roles[2].purpose, "");
        assert!(roles[2].skills.is_empty());
        assert!(!roles[2].read_only);
    }

    /// An ABSENT key takes the default; an explicit `null` is `str(None)`, which is `None`.
    #[test]
    fn an_absent_field_and_a_null_field_are_different() {
        let (_d, roles) = with_roles(r#"{"roles": {"r": {"title": null}}}"#);
        assert_eq!(roles[0].title, "None");
        assert_eq!(roles[0].seat, "specialist");
    }

    #[test]
    fn a_role_with_no_route_field_is_inherit() {
        let (_d, roles) = with_roles(r#"{"roles": {"r": {"route": null}}}"#);
        assert_eq!(roles[0].route(), "(inherit)");
        assert_eq!(roles[0].provider, "");
    }

    #[test]
    fn skills_and_when_follow_as_list() {
        let (_d, roles) = with_roles(
            r#"{"roles": {
                 "a": {"skills": "", "when": null},
                 "b": {"skills": "one", "when": 3},
                 "c": {"skills": ["x", "y"]}}}"#,
        );
        assert!(roles[0].skills.is_empty());
        assert!(roles[0].when.is_empty());
        assert_eq!(roles[1].skills, vec!["one"]);
        assert_eq!(roles[1].when, vec!["3"]);
        assert_eq!(roles[2].skills, vec!["x", "y"]);
    }

    #[test]
    fn a_missing_registry_is_empty_and_an_unreadable_one_is_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = paths_for(dir.path());
        assert!(load_roles(&paths).expect("missing is empty").is_empty());
        let harness = dir.path().join("docs").join("harness");
        std::fs::create_dir_all(&harness).expect("mkdir");
        std::fs::write(harness.join("roles.json"), "{not json").expect("write");
        assert!(
            load_roles(&paths).is_err(),
            "a broken registry must not read as empty"
        );
    }

    #[test]
    fn find_role_takes_an_exact_id_then_a_prefix() {
        let (_d, roles) =
            with_roles(r#"{"roles": {"architecture-reviewer": {}, "archivist": {}}}"#);
        assert_eq!(
            find_role(&roles, "archivist").expect("exact").id,
            "archivist"
        );
        assert_eq!(
            find_role(&roles, "arch").expect("prefix").id,
            "architecture-reviewer"
        );
        assert!(find_role(&roles, "nope").is_none());
    }

    fn one_role() -> Vec<Role> {
        vec![Role {
            id: "b-role".to_string(),
            title: "B".to_string(),
            seat: "x".to_string(),
            provider: "p".to_string(),
            model: "m".to_string(),
            reasoning: String::new(),
            purpose: "pp".to_string(),
            skills: Vec::new(),
            when: Vec::new(),
            read_only: false,
        }]
    }

    /// The columns and the ONE newline after the header, spelled out rather than reusing the
    /// format string under test.
    #[test]
    fn the_roster_renders_pythons_columns() {
        let expected = format!(
            "1 dispatchable roles (docs/harness/roles.json)\n  {}{} {}{} {}{} B\n      pp\n",
            "b-role",
            " ".repeat(18),
            "x",
            " ".repeat(10),
            "p/m",
            " ".repeat(29)
        );
        assert_eq!(render_roster(&one_role()), expected);
        assert!(
            !render_roster(&one_role()).contains("\n\n"),
            "the header is followed by one newline, not a blank line"
        );
    }

    #[test]
    fn read_only_appends_the_marker() {
        let mut roles = one_role();
        roles[0].read_only = true;
        assert!(render_roster(&roles).contains("B [read-only]"));
    }

    /// 100 CHARACTERS, not 100 bytes: the real purposes carry em dashes, and a byte slice would
    /// panic or cut one in half.
    #[test]
    fn the_purpose_is_cut_at_one_hundred_characters() {
        let mut roles = one_role();
        roles[0].purpose = format!("{}—tail", "x".repeat(99));
        let text = render_roster(&roles);
        let purpose_line = text.lines().nth(2).expect("a purpose line");
        assert_eq!(purpose_line, format!("      {}—", "x".repeat(99)));
        assert_eq!(purpose_line.trim_start().chars().count(), 100);
    }

    /// Python's `json.dumps(indent=2)` shape, keys in `Role.__dict__` order.
    #[test]
    fn the_json_is_pythons_shape_and_key_order() {
        let expected = [
            "{",
            "  \"roles\": [",
            "    {",
            "      \"id\": \"b-role\",",
            "      \"title\": \"B\",",
            "      \"seat\": \"x\",",
            "      \"provider\": \"p\",",
            "      \"model\": \"m\",",
            "      \"reasoning\": \"\",",
            "      \"purpose\": \"pp\",",
            "      \"skills\": [],",
            "      \"when\": [],",
            "      \"read_only\": false",
            "    }",
            "  ]",
            "}",
        ]
        .join("\n");
        assert_eq!(roster_json(&one_role()), expected);
    }

    /// `json.dumps` escapes non-ASCII by default and `serde_json` does not. Found by diffing, not
    /// by reading: three real purposes carry an em dash.
    #[test]
    fn the_json_escapes_non_ascii_as_python_does() {
        let mut roles = one_role();
        roles[0].purpose = "a—b😀".to_string();
        let json = roster_json(&roles);
        assert!(json.contains(r#""a\u2014b\ud83d\ude00""#), "{json}");
        assert!(!json.contains('—'), "the raw character must not survive");
    }

    /// The real file's order, checked against the file itself rather than against a remembered
    /// list: the ids must appear in the document in the order the loader returned them.
    #[test]
    fn the_real_registry_loads_in_document_order() {
        let repo = repo();
        let paths = paths_for(&repo);
        let text = std::fs::read_to_string(roles_path(&paths)).expect("roles.json reads");
        let roles = load_roles(&paths).expect("roles load");
        assert!(!roles.is_empty(), "the registry has roles");
        let mut last = 0usize;
        for r in &roles {
            let needle = format!("\"{}\":", r.id);
            let at = text
                .find(&needle)
                .unwrap_or_else(|| panic!("{needle} is not in the file"));
            assert!(at >= last, "{} is out of document order", r.id);
            last = at;
        }
    }

    /// The ported `str()` covers scalars, and this is what keeps that limit from being crossed
    /// silently: every text field in the real registry is a string.
    #[test]
    fn the_real_registry_keeps_its_text_fields_scalar() {
        let repo = repo();
        let text =
            std::fs::read_to_string(roles_path(&paths_for(&repo))).expect("roles.json reads");
        let doc: RolesFile = serde_json::from_str(&text).expect("roles.json parses");
        let roles = doc.roles.expect("roles object").0;
        assert!(!roles.is_empty(), "the registry has roles");
        for (id, raw) in &roles {
            for key in ["title", "seat", "purpose"] {
                assert!(
                    raw.get(key).is_none_or(Value::is_string),
                    "{id}.{key} is not a string"
                );
            }
            if let Some(route) = raw.get("route").filter(|v| truthy(v)) {
                for key in ["provider", "model", "reasoning"] {
                    assert!(
                        route.get(key).is_none_or(Value::is_string),
                        "{id}.route.{key} is not a string"
                    );
                }
            }
            assert!(
                raw.get("read_only").is_none_or(Value::is_boolean),
                "{id}.read_only is not a bool"
            );
        }
    }
}
