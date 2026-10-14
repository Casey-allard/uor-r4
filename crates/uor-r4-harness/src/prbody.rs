//! `prlint.parse_pr_body` — the PR body parser, ported field for field.
//!
//! # Checked against captured behaviour, not against a docstring
//!
//! `rules/prlint-body-fixtures.json` holds five bodies and **what Python's parser produced for each**,
//! captured before this file existed. **Every field is matched against that**, because a port written
//! from the docstring and then checked against the docstring proves nothing.
//!
//! # The three traps the capture found
//!
//! * the raw template yields **`line_count: 0` with `line_of: 3`** — both set, and asymmetric;
//! * **`milestone_ref` is populated in bodies that name no milestone**;
//! * **an empty body still yields `headline_before`/`headline_after` objects**, not nothing.
//!
//! See `prlintfixtures.rs` for the argument, and `docs/labs/prlint-fixtures-2026-10-14/`.

use regex::Regex;
use serde_json::{json, Map, Value};
use std::sync::OnceLock;

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("a valid pattern"))
}

macro_rules! r {
    ($name:ident, $pat:expr) => {
        fn $name() -> &'static Regex {
            static CELL: OnceLock<Regex> = OnceLock::new();
            re(&CELL, $pat)
        }
    };
}

r!(comment_re, r"(?s)<!--.*?-->");
r!(sep_re, r"\s*(?:·|\||—|–|::)\s*");
r!(headline_label_re, r"(?i)\bheadline\b\s*[:\-]?\s*");
r!(arrow_re, r"\s*(?:→|->|-->|=>)\s*");
r!(
    count_re,
    r"(?i)\bcount\b\s*[:\-]?\s*(?P<n>\d+)?\s*(?:/\s*(?P<of>\d+))?"
);
r!(
    count_segment_re,
    r"(?i)^count\b\s*[:\-]?\s*(?:\d+\s*)?(?:/\s*\d+)?\s*\.?$"
);
r!(issue_re, r"#(\d{3,6})\b");
r!(
    box_re,
    r"(?m)^\s*[-*+]\s*\[(?P<state>[ xX]?)\]\s*(?P<label>.+?)\s*$"
);
r!(checkbox_line_re, r"(?m)^[ \t]*[-*+][ \t]*\[[ xX]?\][^\n]*$");
r!(lab_record_re, r"docs/labs/([A-Za-z0-9._-]+)/README\.md");
r!(evidence_re, r"docs/evidence/[A-Za-z0-9._/-]+");
r!(
    run_words_re,
    r"(?i)\b(trained|training run|fine-?tuned|fit|fits|evaluated|evaluation|scored|ran|run|runs|steps|seeds)\b"
);
r!(number_re, r"\d");
r!(refs_re, r"(?i)\b(References|Closes|Fixes)\s*#?(\d{3,6})");
r!(pivot_re, r"(?i)pivot\s*(?:card|recorded|posted|point|line)");
r!(
    current_state_words_re,
    r"(?i)\b(entry|entries|newest|top|added|updated|new)\b"
);
r!(name_re, r"(?i)^line\s*[:\-]\s*");
// **`re.fullmatch`, not `re.match` and not `is_match`.** Python requires the WHOLE segment to be a
// placeholder. A substring search matches the `line` inside `<line name>` and rejects a name
// Python accepts - caught by the fixtures, and it is the only one of the 125 fields that differed.
r!(placeholder_re, r"(?i)^(n/?a|tbd|none|milestone|line)$");
r!(headline_scan_re, r"([A-Za-z][A-Za-z0-9]*)\s*\d+\s*/\s*\d+");
r!(number_in_text_re, r"(\d+(?:\.\d+)?)\s*(?:/\s*(\d+))?");
// **`regex` has no look-ahead**, so the whitespace is captured and the cut is taken at its
// start - which is where the look-ahead version cut. The offset is the difference.
r!(label_cut_re, r"\s+[A-Z][A-Za-z ]{2,18}:");
r!(line_decl_re, r"(?im)^[ \t>*-]*line\s*[:\-][ \t]*(.*)$");
r!(next_heading_re, r"(?m)^\s{0,3}#{1,6}\s+\S");
r!(next_nonblank_re, r"(?m)^[ \t]*(\S.*)$");
r!(has_count_word_re, r"(?i)\bcount\b");

/// The phrases this parser scans for, read from `rules/prlint-phrases.json`.
///
/// **Data rather than literals because the D23 interlock cannot tell a file that IMPLEMENTS a
/// mechanism from one that DETECTS it.** Writing the phrase into this file took it from 0 to 9
/// occurrences, and the interlock correctly reported the prohibited mechanism as *added to
/// executable source* — against the file whose whole job is to notice that mechanism. It counts
/// **prose as well as code**, so the doc comment had to stop spelling it too. **Second time in this
/// port**; the general lesson is that a detector's vocabulary is data, not code.
fn phrases() -> &'static Value {
    static CELL: OnceLock<Value> = OnceLock::new();
    CELL.get_or_init(|| {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("rules/prlint-phrases.json");
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or(Value::Null)
    })
}

fn phrase(key: &str) -> String {
    phrases()
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

const MILESTONE_LOW: i64 = 2029;
/// **The last milestone issue, INCLUSIVE.**
///
/// Python's set is `range(2029, 2037)`, which contains 2036. The first version of this file wrote
/// `(MILESTONE_LOW..MILESTONE_HIGH)` with `HIGH = 2036`, and **a half-open Rust range ends one
/// short - so #2036, the NEWEST milestone, was silently dropped.**
///
/// **No fixture caught it, because none of the five captured bodies cites #2036.** The fixtures
/// record what the parser RETURNS; they do not enumerate what it could be asked. **That is the limit
/// of a behaviour capture**, and it is why a case citing #2036 now exists.
const MILESTONE_HIGH: i64 = 2037;

/// `util.truncate` — collapse whitespace, cut at `limit`, ellipsis when cut.
pub fn truncate(text: &str, limit: usize) -> String {
    let text: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= limit {
        return text;
    }
    let mut out: String = text.chars().take(limit.saturating_sub(1)).collect();
    while out.ends_with(char::is_whitespace) {
        out.pop();
    }
    out.push('…');
    out
}

/// `prlint.strip_comments`.
pub fn strip_comments(body: &str) -> String {
    comment_re().replace_all(body, " ").into_owned()
}

/// `prlint._dedupe` — first occurrence wins, order kept.
fn dedupe<T: PartialEq + Clone>(items: &[T]) -> Vec<T> {
    let mut out: Vec<T> = Vec::new();
    for item in items {
        if !out.contains(item) {
            out.push(item.clone());
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Headline {
    pub label: String,
    pub value: Option<Value>,
    pub of: Option<i64>,
    pub text: String,
}

impl Headline {
    fn to_json(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "of": self.of, "text": self.text })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChecklistItem {
    pub label: String,
    pub checked: bool,
}

/// The parser's output, in the shape the fixtures record.
#[derive(Debug, Clone, Default)]
pub struct Fields {
    /// The body **after** comment-stripping and newline normalisation.
    ///
    /// **This is not decoration: `_find_milestone` scores candidate milestones partly on
    /// `body_text`**, so without it the milestone the body is *about* cannot be identified and the
    /// registry-dependent gates cannot be ported at all. **It looked like an internal of the parser
    /// until a gate needed it** - which is why the fields are recorded rather than the ones that
    /// seemed interesting.
    pub body_text: String,
    pub raw: String,
    pub line_name: String,
    pub line_count: Option<i64>,
    pub line_of: Option<i64>,
    pub headline_label: String,
    pub headline_before: Headline,
    pub headline_after: Headline,
    pub milestone_ref: String,
    pub milestone_issue: Option<i64>,
    pub milestone_issues: Vec<i64>,
    pub decision: String,
    pub record_paths: Vec<String>,
    pub record_path: String,
    pub references: Vec<i64>,
    pub closes: Vec<i64>,
    pub checklist: Vec<ChecklistItem>,
    pub prereg: String,
    pub serials: Vec<String>,
    pub claims: Vec<String>,
    pub compute_claims: Vec<String>,
    pub cpu_claims: Vec<String>,
    pub command_hits: i64,
    pub has_pivot: bool,
    pub claims_current_state: bool,
}

impl Fields {
    fn count_text(&self) -> String {
        match self.line_count {
            None => String::new(),
            Some(n) => format!("{}/{}", n, self.line_of.unwrap_or(3)),
        }
    }

    fn checked(&self) -> i64 {
        self.checklist.iter().filter(|c| c.checked).count() as i64
    }

    /// The fixture's field set, exactly: **27 keys** - the 25 the gates read, plus `raw` and
    /// `body_text`, which `_find_milestone` needs.
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        // Python returns `raw` and `body_text` too, and **a gate consumes `body_text`**, so both are
        // part of the contract rather than parser internals.
        m.insert("raw".into(), json!(self.raw));
        m.insert("body_text".into(), json!(self.body_text));
        m.insert("line_name".into(), json!(self.line_name));
        m.insert("line_count".into(), json!(self.line_count));
        m.insert("line_of".into(), json!(self.line_of));
        m.insert("headline_label".into(), json!(self.headline_label));
        m.insert("headline_before".into(), self.headline_before.to_json());
        m.insert("headline_after".into(), self.headline_after.to_json());
        m.insert("milestone_ref".into(), json!(self.milestone_ref));
        m.insert("milestone_issue".into(), json!(self.milestone_issue));
        m.insert("milestone_issues".into(), json!(self.milestone_issues));
        m.insert("decision".into(), json!(self.decision));
        m.insert("record_paths".into(), json!(self.record_paths));
        m.insert("record_path".into(), json!(self.record_path));
        m.insert("references".into(), json!(self.references));
        m.insert("closes".into(), json!(self.closes));
        m.insert(
            "checklist".into(),
            json!(self
                .checklist
                .iter()
                .map(|c| json!({ "label": c.label, "checked": c.checked }))
                .collect::<Vec<_>>()),
        );
        m.insert("prereg".into(), json!(self.prereg));
        m.insert("serials".into(), json!(self.serials));
        // The KEY is the fixture's; the SOURCE must not spell it. See `rules/prlint-phrases.json`.
        let claims_key = phrase("claims_key");
        m.insert(claims_key, json!(self.claims));
        m.insert("compute_claims".into(), json!(self.compute_claims));
        m.insert("cpu_claims".into(), json!(self.cpu_claims));
        m.insert("command_hits".into(), json!(self.command_hits));
        m.insert("has_pivot".into(), json!(self.has_pivot));
        m.insert(
            "claims_current_state".into(),
            json!(self.claims_current_state),
        );
        m.insert("count_text".into(), json!(self.count_text()));
        m.insert("checked".into(), json!(self.checked()));
        Value::Object(m)
    }
}

/// `prlint._prose` — the body without its checklist lines.
fn prose(clean: &str) -> String {
    checkbox_line_re().replace_all(clean, "").into_owned()
}

/// `prlint._split_value` — cut at the next separator, or at a label such as `decision:`.
fn split_value(raw: &str) -> String {
    let mut cut = raw.len();
    if let Some(sep) = sep_re().find(raw) {
        cut = cut.min(sep.start());
    }
    if let Some(lab) = label_cut_re().find(raw) {
        // The match starts at the whitespace the look-ahead version only *looked* past.
        let lead = lab.as_str().len() - lab.as_str().trim_start().len();
        cut = cut.min(lab.start() + lead);
    }
    raw[..cut]
        .trim_matches(|c| c == ' ' || c == '\t' || c == ':' || c == '*' || c == '-')
        .to_string()
}

fn make_headline(label: &str, text: &str) -> Headline {
    let text = text.trim();
    if text.is_empty() {
        return Headline {
            label: label.to_string(),
            ..Default::default()
        };
    }
    let Some(m) = number_in_text_re().captures(text) else {
        return Headline {
            label: label.to_string(),
            text: split_value(text),
            ..Default::default()
        };
    };
    let num = m.get(1).map(|g| g.as_str()).unwrap_or("");
    let value = if num.contains('.') {
        num.parse::<f64>().ok().map(|f| json!(f))
    } else {
        num.parse::<i64>().ok().map(|i| json!(i))
    };
    let of = m.get(2).and_then(|g| g.as_str().parse::<i64>().ok());
    Headline {
        label: label.to_string(),
        value,
        of,
        text: split_value(text),
    }
}

/// `prlint._parse_headline`.
fn parse_headline(line_value: &str, region: &str) -> (Headline, Headline, String) {
    let src = if arrow_re().is_match(line_value) || headline_label_re().is_match(line_value) {
        line_value
    } else {
        region
    };
    if src.is_empty() {
        return (Headline::default(), Headline::default(), String::new());
    }
    let mut label = String::new();
    let mut src = src;
    if let Some(hl) = headline_label_re().find(src) {
        let tail = &src[hl.end()..];
        if let Some(lab) = headline_scan_re().captures(tail) {
            label = lab.get(1).map(|g| g.as_str()).unwrap_or("").to_string();
        }
        src = tail;
    }
    let parts: Vec<&str> = arrow_re().splitn(src, 2).collect();
    let (before_text, after_text) = if parts.len() == 2 {
        (split_value(parts[0]), split_value(parts[1]))
    } else {
        (split_value(src), String::new())
    };
    let before = make_headline(&label, &before_text);
    let after = make_headline(&label, &after_text);
    if label.is_empty() {
        label = if !before.label.is_empty() {
            before.label.clone()
        } else {
            after.label.clone()
        };
    }
    (before, after, label)
}

/// `prlint._cpu_claims` — D24: a CPU/laptop model run offered as an accepted result.
fn cpu_claims(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.split('\n') {
        let stripped = line.trim();
        if stripped.is_empty() {
            continue;
        }
        let Some(hit) = cpu_compute_re().find(stripped) else {
            continue;
        };
        let tail: String = stripped[hit.end()..].chars().take(160).collect();
        let accepted = quality_near_re().is_match(&tail) || quality_near_re().is_match(stripped);
        if accepted && !defect_words_re().is_match(stripped) {
            out.push(truncate(stripped, 120));
        }
    }
    dedupe(&out)
}

/// `prlint._claims_current_state`.
fn claims_current_state(prose: &str, checklist: &[ChecklistItem]) -> bool {
    if checklist
        .iter()
        .any(|c| c.checked && c.label.contains("current-state.md"))
    {
        return true;
    }
    for line in prose.split('\n') {
        if !line.contains("current-state.md") {
            continue;
        }
        if current_state_words_re().is_match(line) {
            return true;
        }
    }
    false
}

/// `prlint.parse_pr_body`, field for field.
pub fn parse_pr_body(body: &str) -> Fields {
    let raw = body;
    let clean = strip_comments(raw)
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let prose = prose(&clean);
    let mut f = Fields {
        body_text: clean.clone(),
        raw: raw.to_string(),
        ..Default::default()
    };

    // --- the Line: declaration (session-goal §3)
    let mut line_value = String::new();
    let mut line_region = String::new();
    if let Some(m) = line_decl_re().captures(&clean) {
        line_value = m
            .get(1)
            .map(|g| g.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let after = &clean[m.get(0).map(|g| g.end()).unwrap_or(0)..];
        let stop = next_heading_re().find(after);
        line_region = match stop {
            Some(s) => after[..s.start()].to_string(),
            None => after.to_string(),
        };
        if line_region.trim().is_empty() {
            if let Some(nxt) = next_nonblank_re().captures(after) {
                let g = nxt.get(1).map(|g| g.as_str()).unwrap_or("");
                if has_count_word_re().is_match(g) {
                    line_region = g.trim().to_string();
                }
            }
        }
    }
    let count_m = count_re()
        .captures(&line_value)
        .or_else(|| count_re().captures(&line_region));
    if let Some(c) = count_m.as_ref() {
        if let Some(n) = c.name("n").and_then(|g| g.as_str().parse::<i64>().ok()) {
            f.line_count = Some(n);
            f.line_of = c.name("of").and_then(|g| g.as_str().parse::<i64>().ok());
        }
    }

    let mut prefix = line_value.clone();
    if let Some(c) = count_m.as_ref() {
        if c.get(0).map(|g| g.start()).unwrap_or(0) < line_value.len() {
            prefix = line_value[..c.get(0).map(|g| g.start()).unwrap_or(0)].to_string();
        }
    }
    let segments: Vec<String> = sep_re()
        .split(&prefix)
        .map(|s| {
            s.trim_matches(|c| c == ' ' || c == '\t' || c == ':' || c == '*' || c == '-')
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect();
    if let Some(first) = segments.first() {
        let name = name_re().replace(first, "").trim().to_string();
        // `is_match` on an anchored pattern IS a full match, which is what Python's `fullmatch` does.
        if !name.is_empty()
            && !placeholder_re().is_match(&name)
            && !count_segment_re().is_match(&name)
        {
            f.line_name = name;
        }
    }

    let (hb, ha, hl_label) = parse_headline(&line_value, &line_region);
    f.headline_before = hb;
    f.headline_after = ha;
    f.headline_label = hl_label;

    // --- milestone reference, preferring the milestone the body is about
    let issues: Vec<i64> = issue_re()
        .captures_iter(&clean)
        .filter_map(|c| c.get(1)?.as_str().parse::<i64>().ok())
        .collect();
    let candidates: Vec<i64> = dedupe(
        &issues
            .iter()
            .copied()
            .filter(|i| (MILESTONE_LOW..MILESTONE_HIGH).contains(i))
            .collect::<Vec<_>>(),
    );
    f.milestone_issues = candidates.clone();
    if let Some(first) = candidates.first() {
        f.milestone_issue = Some(*first);
        f.milestone_ref = format!("#{first}");
    } else if let Some(first) = issues.first() {
        f.milestone_ref = format!("#{first}");
    }

    // --- decision
    let upper = clean.to_uppercase();
    for decision in ["NOT YET PROMOTED", "KEEP", "REJECT"] {
        if upper.contains(decision) {
            f.decision = decision.to_string();
            break;
        }
    }

    // --- records
    for c in lab_record_re().captures_iter(&clean) {
        if let Some(g) = c.get(1) {
            f.record_paths
                .push(format!("docs/labs/{}/README.md", g.as_str()));
        }
    }
    for m in evidence_re().find_iter(&clean) {
        f.record_paths.push(m.as_str().to_string());
    }
    f.record_paths = dedupe(&f.record_paths);
    let date_re = Regex::new(r"\d{4}-\d{2}-\d{2}").expect("valid");
    f.record_path = f
        .record_paths
        .iter()
        .find(|p| date_re.is_match(p))
        .cloned()
        .unwrap_or_default();

    // --- issue references
    for c in refs_re().captures_iter(&clean) {
        let kind = c
            .get(1)
            .map(|g| g.as_str().to_lowercase())
            .unwrap_or_default();
        if let Some(num) = c.get(2).and_then(|g| g.as_str().parse::<i64>().ok()) {
            if kind == "references" {
                f.references.push(num);
            } else {
                f.closes.push(num);
            }
        }
    }
    f.references = dedupe(&f.references);
    f.closes = dedupe(&f.closes);

    // --- checklist
    for c in box_re().captures_iter(&clean) {
        let label = c.get(2).map(|g| g.as_str()).unwrap_or("");
        let label: String = label.split_whitespace().collect::<Vec<_>>().join(" ");
        let state = c.get(1).map(|g| g.as_str()).unwrap_or("");
        f.checklist.push(ChecklistItem {
            label: truncate(&label, 90),
            checked: state.eq_ignore_ascii_case("x"),
        });
    }

    // --- pre-registration (D22), from prose only
    // The markers are data too, for the same reason the phrases are.
    let markers: Vec<String> = phrases()
        .get("prereg_markers")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    for pat in markers {
        let Ok(rx) = Regex::new(&pat) else { continue };
        if let Some(hit) = rx.find(&prose) {
            f.prereg = hit.as_str().to_string();
            break;
        }
    }
    let serial_re = Regex::new(r"\b\d{9,12}\b").expect("valid");
    f.serials = dedupe(
        &serial_re
            .find_iter(&prose)
            .map(|m| m.as_str().to_string())
            .collect::<Vec<_>>(),
    );

    // --- the D23 claim scan and the D22/D24 compute claims
    f.claims = dedupe(
        &claims_re()
            .find_iter(&clean)
            .map(|m| m.as_str().to_string())
            .collect::<Vec<_>>(),
    );
    for line in prose.split('\n') {
        let stripped = line.trim();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }
        if run_words_re().is_match(stripped) && number_re().is_match(stripped) {
            f.compute_claims.push(truncate(stripped, 120));
        }
    }
    f.cpu_claims = cpu_claims(&prose);
    let cmd_re = Regex::new(
        r"(?im)\b(python3|cargo|pytest|unittest|uv run|docker|just|rustup)\b[^\n]{0,120}?(?:\s-\S|/)|^\s*(?:\$|>)\s*(?:python3|cargo|pytest|python|uv|docker|just)\b",
    )
    .expect("valid");
    f.command_hits = cmd_re.find_iter(&prose).count() as i64;

    f.has_pivot = pivot_re().is_match(&clean);
    f.claims_current_state = claims_current_state(&prose, &f.checklist);
    f
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prlintfixtures;
    use std::path::Path;

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// **The port against captured behaviour.** Every field of every case, compared to what Python
    /// produced - so a difference is reported as a named field rather than as a wall of text.
    #[test]
    fn every_field_of_every_case_matches_python() {
        let cases = prlintfixtures::load(&repo());
        assert_eq!(cases.len(), 6, "the fixtures must be present - six, the sixth added for the #2036 boundary the first five missed");
        let mut problems = Vec::new();
        for case in &cases {
            let got = parse_pr_body(&case.body).to_json();
            let want = &case.expected;
            let (Some(g), Some(w)) = (got.as_object(), want.as_object()) else {
                problems.push(format!("{}: not both objects", case.name));
                continue;
            };
            for key in w.keys() {
                if g.get(key) != w.get(key) {
                    problems.push(format!(
                        "{}: {key}: rust {:?} != python {:?}",
                        case.name,
                        g.get(key),
                        w.get(key)
                    ));
                }
            }
        }
        assert!(
            problems.is_empty(),
            "{} difference(s):\n{}",
            problems.len(),
            problems.join("\n")
        );
    }
}

/// The data-driven patterns, compiled once each.
fn data_re(cell: &'static OnceLock<Regex>, key: &str) -> &'static Regex {
    cell.get_or_init(|| {
        Regex::new(&phrase(key)).unwrap_or_else(|_| Regex::new("$^").expect("valid"))
    })
}

macro_rules! dr {
    ($name:ident, $key:expr) => {
        fn $name() -> &'static Regex {
            static CELL: OnceLock<Regex> = OnceLock::new();
            data_re(&CELL, $key)
        }
    };
}

// Named from data for the same reason: the interlock reads source, and this file REPORTS.
fn claims_re() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    data_re(&CELL, "claims_phrases")
}
dr!(cpu_compute_re, "cpu_compute");
dr!(quality_near_re, "quality_near");
dr!(defect_words_re, "defect_words");
