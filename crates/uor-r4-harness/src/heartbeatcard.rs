//! The heartbeat status card: `headline · line · count n/3 · next step`.
//!
//! Ported from `heartbeat` in `scripts/harness/status.py`, whose card text is the spec. The card is
//! built from the milestone registry alone, never from the lab's remembered presence: it states
//! where the milestone stands, and a lab's last intent is a different question.

use crate::registries::{Line, Milestone, Registries};
use serde_json::Value;

/// The card and the parts it was built from.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub headline: String,
    pub line: String,
    pub count: String,
    pub next: String,
    pub how: String,
    pub text: String,
}

/// `_headline_text`: the registry's own text when the headline line would print a missing value.
pub fn headline_text(m: &Milestone) -> String {
    let line = m.headline_line();
    // Python renders a missing value as `None`; this port renders it as `MISSING`.
    if !line.contains("None") && !line.contains("MISSING") {
        return line;
    }
    let h = &m.headline;
    let text = py_str(h.get("text"), "").trim().to_string();
    if text.is_empty() {
        return line;
    }
    let display = py_str(h.get("target_display"), "").trim().to_string();
    if !display.is_empty() {
        return format!("{}: {text} -> target {display}", m.id);
    }
    match h.get("target") {
        Some(t) if !t.is_null() => format!("{}: {text} -> target {}", m.id, py_str(Some(t), "")),
        _ => format!("{}: {text}", m.id),
    }
}

/// The card, with Python's selection rules: `--line` matched case-insensitively, else the line
/// nearest its 3/3 pivot; a bare `--count n` takes that line's denominator.
pub fn build(m: &Milestone, line: Option<&str>, count: Option<&str>, next: Option<&str>) -> Card {
    let line = line.filter(|s| !s.is_empty());
    let wanted = line.map(|s| s.trim().to_lowercase());
    let chosen = wanted
        .as_ref()
        .and_then(|w| m.lines.iter().find(|l| l.name.to_lowercase() == *w))
        .or_else(|| nearest_pivot(&m.lines));
    let name = match line {
        Some(s) => s.trim().to_string(),
        None => chosen.map_or("no line registered".to_string(), |l| l.name.clone()),
    };
    let mut count_text = chosen.map_or("0/3".to_string(), |l| {
        format!(
            "{}/{}",
            py_str(Some(&l.count), "0"),
            py_str(Some(&l.of), "3")
        )
    });
    if let Some(c) = count.filter(|s| !s.is_empty()) {
        count_text = if c.contains('/') {
            c.to_string()
        } else {
            let of = nearest_pivot(&m.lines)
                .map(|l| &l.of)
                .filter(|v| truthy(v))
                .map_or("3".to_string(), |v| py_str(Some(v), "3"));
            format!("{c}/{of}")
        };
    }
    let next = next
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| Some(m.next.clone()).filter(|s| !s.is_empty()))
        .unwrap_or_else(|| "(no next step registered)".to_string());
    let headline = headline_text(m);
    let prefix = format!("{}: ", m.id);
    let how = format!(
        "moves the headline {}",
        headline.strip_prefix(&prefix).unwrap_or(&headline)
    );
    let text = [
        format!("{} #{}", m.id, m.issue),
        headline.clone(),
        format!("line: {name}"),
        format!("count {count_text}"),
        format!("next: {next}"),
        format!("[{how}]"),
    ]
    .join(" · ");
    Card {
        headline,
        line: name,
        count: count_text,
        next,
        how,
        text,
    }
}

/// Python's message for a milestone the registry does not know.
pub fn unknown_milestone(regs: &Registries, key: &str) -> String {
    let mut known: Vec<&str> = regs.milestones.iter().map(|m| m.key.as_str()).collect();
    known.sort_unstable();
    format!(
        "heartbeat: unknown milestone '{key}'; known: {}",
        known.join(", ")
    )
}

/// Python's `max` keeps the FIRST of equal ratios; `Iterator::max_by` keeps the last.
fn nearest_pivot(lines: &[Line]) -> Option<&Line> {
    let mut best: Option<(&Line, f64)> = None;
    for l in lines {
        let r = ratio(l);
        if best.is_none_or(|(_, b)| r > b) {
            best = Some((l, r));
        }
    }
    best.map(|(l, _)| l)
}

fn ratio(l: &Line) -> f64 {
    let count = if l.count.is_null() {
        Some(0.0)
    } else {
        as_f64(&l.count)
    };
    let of = if l.of.is_null() {
        Some(3.0)
    } else {
        as_f64(&l.of)
    };
    match (count, of) {
        (Some(c), Some(o)) => c / o.max(1.0),
        _ => 0.0,
    }
}

fn as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        Value::Bool(b) => Some(f64::from(u8::from(*b))),
        _ => None,
    }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Python's `str()` of a registry value; an absent or null value takes `default`.
fn py_str(v: Option<&Value>, default: &str) -> String {
    match v {
        None | Some(Value::Null) => default.to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => match n.as_i64() {
            Some(i) => i.to_string(),
            None => n.as_f64().map_or(n.to_string(), |f| format!("{f:?}")),
        },
        Some(Value::Bool(b)) => if *b { "True" } else { "False" }.to_string(),
        Some(other) => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn line(name: &str, count: i64, of: i64) -> Line {
        Line {
            name: name.to_string(),
            count: json!(count),
            of: json!(of),
        }
    }

    fn m1() -> Milestone {
        Milestone {
            key: "m1".to_string(),
            issue: 2029,
            id: "M1".to_string(),
            headline: json!({"label": "reply acceptable", "current": 37, "target": 116, "of": 232}),
            lines: vec![
                line("exact addressed memory", 0, 3),
                line("evaluation integrity", 1, 3),
            ],
            next: "the registered step".to_string(),
            ..Milestone::default()
        }
    }

    #[test]
    fn the_default_line_is_the_one_nearest_its_pivot() {
        let c = build(&m1(), None, None, None);
        assert_eq!(
            c.text,
            "M1 #2029 · M1: reply acceptable 37/232 -> target 116/232 · line: evaluation integrity \
             · count 1/3 · next: the registered step \
             · [moves the headline reply acceptable 37/232 -> target 116/232]"
        );
    }

    #[test]
    fn equal_ratios_keep_the_first_line_as_python_does() {
        let mut m = m1();
        m.lines = vec![line("first", 1, 3), line("second", 1, 3)];
        assert_eq!(build(&m, None, None, None).line, "first");
    }

    #[test]
    fn a_line_override_matches_case_insensitively_and_keeps_the_typed_name() {
        let c = build(&m1(), Some(" Exact Addressed MEMORY "), None, None);
        assert_eq!(c.line, "Exact Addressed MEMORY");
        assert_eq!(c.count, "0/3", "the matched line's count, not the pivot's");
    }

    #[test]
    fn an_unmatched_line_override_keeps_the_pivot_count() {
        let c = build(&m1(), Some("something new"), None, None);
        assert_eq!(
            (c.line.as_str(), c.count.as_str()),
            ("something new", "1/3")
        );
    }

    #[test]
    fn a_bare_count_takes_the_pivot_lines_denominator() {
        assert_eq!(build(&m1(), None, Some("2"), None).count, "2/3");
        assert_eq!(build(&m1(), None, Some("2/5"), None).count, "2/5");
        let mut m = m1();
        m.lines.clear();
        assert_eq!(build(&m, None, Some("2"), None).count, "2/3");
        assert_eq!(build(&m, None, None, None).line, "no line registered");
    }

    #[test]
    fn next_falls_back_to_the_registry_then_to_a_stated_absence() {
        assert_eq!(build(&m1(), None, None, Some("x")).next, "x");
        let mut m = m1();
        m.next.clear();
        assert_eq!(
            build(&m, None, None, None).next,
            "(no next step registered)"
        );
    }

    #[test]
    fn the_suffix_says_how_the_headline_moves() {
        let mut m = m1();
        m.headline = json!({"label": "typed outcomes", "vacuous": true});
        let c = build(&m, None, None, None);
        // A vacuous headline carries no `M1: ` prefix, so nothing is stripped.
        assert!(c
            .text
            .ends_with(" · [moves the headline typed outcomes: not qualified]"));
    }

    #[test]
    fn a_qualitative_headline_without_a_target_falls_back_to_its_text() {
        let mut m = m1();
        m.headline = json!({"label": "x", "text": "a proposition"});
        assert_eq!(headline_text(&m), "M1: a proposition");
    }

    #[test]
    fn an_unknown_milestone_names_the_known_ones_in_pythons_shape() {
        let regs = Registries {
            note: String::new(),
            tracker: Value::Null,
            owner_rules: Vec::new(),
            milestones: vec![
                Milestone {
                    key: "m2".to_string(),
                    ..Milestone::default()
                },
                m1(),
            ],
            panels: Default::default(),
            decisions: Vec::new(),
        };
        assert_eq!(
            unknown_milestone(&regs, "x"),
            "heartbeat: unknown milestone 'x'; known: m1, m2"
        );
    }
}
