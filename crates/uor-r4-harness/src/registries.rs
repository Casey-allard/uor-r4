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
        })
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
