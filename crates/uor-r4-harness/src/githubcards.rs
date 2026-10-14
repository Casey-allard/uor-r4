//! The pure core of `status.registry_vs_github` — reading a milestone issue's newest card.
//!
//! # Why only the core is here
//!
//! The rule is ~135 lines and three of its parts need a network: a cache under the state directory,
//! one `gh issue view` per milestone, and comment pagination. **Those are not ported.**
//!
//! **What IS ported is where the rule decides anything**: `_newest_card`, `_card_reading` and
//! `_registry_numbers` turn a list of comments into a reading, and **a reading is what the rule
//! compares against the registry.** They are pure, they are the part a bug would live in, and
//! **they can be compared against Python without a network** — which is the only way to verify them
//! honestly here.
//!
//! **So this is a partial port, registered as nothing.** It is the well-tested half of a rule that
//! stays in Python until someone ports the `gh` chain, **and `status.registry_vs_github` remains in
//! the Python set until then** — a half-ported rule cannot be claimed as coverage.

use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

/// Markers that make a comment a status card.
const CARD_MARKERS: [&str; 4] = ["status card", "blocked card", "pivot card", "headline"];

/// How far past a `headline` marker the numbers are looked for.
pub const HEADLINE_WINDOW: usize = 120;

/// What the newest card says, in `_card_reading`'s shape.
#[derive(Debug, Clone, PartialEq)]
pub struct CardReading {
    pub kind: String,
    pub ts: String,
    pub text: String,
    pub current: Option<i64>,
    pub of: Option<i64>,
    pub yes_no: String,
}

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("a valid pattern"))
}

/// `status._newest_card` — the newest comment that looks like a status card.
///
/// **Newest first by `createdAt` as a STRING**, which is what Python's `sorted(..., reverse=True)`
/// does. An ISO-8601 timestamp sorts correctly as text, **and a missing `createdAt` sorts as the
/// empty string — last.** Both behaviours are copied rather than improved: **sorting by a parsed
/// timestamp would reorder comments whose timestamps are malformed, and the rule compares against the
/// newest card, so the order decides the answer.**
pub fn newest_card(comments: &[Value]) -> Option<(Value, String)> {
    let mut ordered: Vec<&Value> = comments.iter().collect();
    ordered.sort_by_key(|c| {
        std::cmp::Reverse(
            c.get("createdAt")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        )
    });
    for c in ordered {
        let body = c
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let low = body.to_lowercase();
        if CARD_MARKERS.iter().any(|m| low.contains(m)) {
            return Some((c.clone(), body));
        }
    }
    None
}

/// `util.truncate` with `HEADLINE_WINDOW`, for the reading's `text`.
fn window(text: &str) -> String {
    crate::prbody::truncate(text, HEADLINE_WINDOW)
}

/// `status._card_reading` — what the newest status card says about this milestone's number.
///
/// **Two forms, and the order matters**: the strict `M<n>: <n>/<n>` first, then the numbers after a
/// `headline` marker, which is how the labs actually write the card today. **A card that matches
/// neither is `card_no_number` rather than an error** — the rule reports that it could not compare,
/// which is different from reporting a disagreement.
pub fn card_reading(comments: &[Value], milestone_id: &str) -> CardReading {
    let none = CardReading {
        kind: "no_card".into(),
        ts: String::new(),
        text: String::new(),
        current: None,
        of: None,
        yes_no: String::new(),
    };
    let Some((comment, body)) = newest_card(comments) else {
        return none;
    };
    let ts = comment
        .get("createdAt")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let strict_pat = format!(
        r"\b{}\s*[:=]\s*(\d+)\s*/\s*(\d+)",
        regex::escape(milestone_id)
    );
    if let Some(m) = re_strict(&strict_pat).captures(&body) {
        return CardReading {
            kind: "milestone_number".into(),
            ts,
            text: m.get(0).map(|g| g.as_str()).unwrap_or("").to_string(),
            current: m.get(1).and_then(|g| g.as_str().parse().ok()),
            of: m.get(2).and_then(|g| g.as_str().parse().ok()),
            yes_no: String::new(),
        };
    }

    let marker = re(marker_cell(), r"(?i)headline\s*[:\-]?").find(&body);
    let Some(marker) = marker else {
        return CardReading {
            kind: "card_no_number".into(),
            ts,
            text: window(&body),
            current: None,
            of: None,
            yes_no: String::new(),
        };
    };
    // The window is taken in CHARACTERS, not bytes: a card quoting non-ASCII would otherwise slice
    // mid-character, which is the char-boundary panic this port has hit twice.
    let fragment: String = body[marker.end()..].chars().take(HEADLINE_WINDOW).collect();
    let numbers = re(numbers_cell(), r"(\d+)\s*/\s*(\d+)").captures(&fragment);
    let yes_no = if numbers.is_none() {
        re(yesno_cell(), r"(?i)\b(yes|no)\b")
            .captures(&fragment)
            .and_then(|c| c.get(1).map(|g| g.as_str().to_lowercase()))
            .unwrap_or_default()
    } else {
        String::new()
    };
    CardReading {
        kind: if numbers.is_some() {
            "headline_number".into()
        } else {
            "headline_text".into()
        },
        ts,
        text: window(&fragment),
        current: numbers
            .as_ref()
            .and_then(|c| c.get(1))
            .and_then(|g| g.as_str().parse().ok()),
        of: numbers
            .as_ref()
            .and_then(|c| c.get(2))
            .and_then(|g| g.as_str().parse().ok()),
        yes_no,
    }
}

/// `status._registry_numbers` — every `(current, of)` the registry claims for a milestone.
///
/// **`secondary` rows count too**, so a milestone stating its number in more than one place has every
/// pair compared. **A row missing either half is skipped rather than read as zero** — a missing number
/// is not a claim of no progress.
pub fn registry_numbers(m: &crate::registries::Milestone) -> Vec<(i64, i64)> {
    let mut rows: Vec<&Value> = vec![&m.headline];
    rows.extend(m.secondary.iter());
    let mut out = Vec::new();
    for row in rows {
        let (Some(cur), Some(of)) = (
            row.get("current").and_then(Value::as_f64),
            row.get("of").and_then(Value::as_f64),
        ) else {
            continue;
        };
        out.push((cur as i64, of as i64));
    }
    out
}

// The patterns are `OnceLock`-cached like the rest of the port, but the strict one is built from the
// milestone id, so it is per-call rather than cached.
fn re_strict(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|_| Regex::new("$^").expect("valid"))
}

fn marker_cell() -> &'static OnceLock<Regex> {
    static CELL: OnceLock<Regex> = OnceLock::new();
    &CELL
}
fn numbers_cell() -> &'static OnceLock<Regex> {
    static CELL: OnceLock<Regex> = OnceLock::new();
    &CELL
}
fn yesno_cell() -> &'static OnceLock<Regex> {
    static CELL: OnceLock<Regex> = OnceLock::new();
    &CELL
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn comments(rows: &[(&str, &str)]) -> Vec<Value> {
        rows.iter()
            .map(|(ts, body)| json!({ "createdAt": ts, "body": body }))
            .collect()
    }

    /// **Python's own readings for these inputs.** Asserted rather than constructed, because the
    /// reading is what the rule compares and a plausible-looking `kind` would agree with nothing.
    ///
    /// **A named struct rather than a six-tuple**: clippy is right that the tuple is unreadable, and a
    /// positional expectation is one transposition away from asserting the wrong field.
    #[test]
    fn the_reading_matches_python_for_every_case() {
        struct Case {
            name: &'static str,
            comments: Vec<Value>,
            kind: &'static str,
            current: Option<i64>,
            of: Option<i64>,
            yes_no: &'static str,
        }
        let case = |name, comments, kind, current, of, yes_no| Case {
            name,
            comments,
            kind,
            current,
            of,
            yes_no,
        };
        let cases = vec![
            case("empty", vec![], "no_card", None, None, ""),
            case(
                "no-card",
                comments(&[("2026-10-14T00:00:00Z", "just chatter")]),
                "no_card",
                None,
                None,
                "",
            ),
            // **A strict-looking body with NO card marker is `no_card`**, not `milestone_number`:
            // the marker is checked first, so the strict form is a refinement of a card rather than a
            // way to find one. **I read the docstring as saying otherwise and the capture corrected me.**
            case(
                "no-markers",
                comments(&[("2026-10-14T00:00:00Z", "M1: 37/232 here")]),
                "no_card",
                None,
                None,
                "",
            ),
            case(
                "strict-with-marker",
                comments(&[("2026-10-14T00:00:00Z", "headline card M1: 37/232 says so")]),
                "milestone_number",
                Some(37),
                Some(232),
                "",
            ),
            case(
                "headline-number",
                comments(&[(
                    "2026-10-14T00:00:00Z",
                    "**STATUS CARD** headline: reply 37/232 (target 116)",
                )]),
                "headline_number",
                Some(37),
                Some(232),
                "",
            ),
            case(
                "headline-yes",
                comments(&[(
                    "2026-10-14T00:00:00Z",
                    "status card: headline yes we shipped",
                )]),
                "headline_text",
                None,
                None,
                "yes",
            ),
            case(
                "card-no-number",
                comments(&[(
                    "2026-10-14T00:00:00Z",
                    "status card with no headline marker at all",
                )]),
                "headline_text",
                None,
                None,
                "",
            ),
            case(
                "missing-ts",
                comments(&[("", "status card: headline 3/4")]),
                "headline_number",
                Some(3),
                Some(4),
                "",
            ),
        ];
        for c in cases {
            let r = card_reading(&c.comments, "M1");
            assert_eq!(r.kind, c.kind, "{}: kind", c.name);
            assert_eq!(r.current, c.current, "{}: current", c.name);
            assert_eq!(r.of, c.of, "{}: of", c.name);
            assert_eq!(r.yes_no, c.yes_no, "{}: yes_no", c.name);
        }
    }

    /// **The newest card wins**, because the rule compares against the newest. Python sorts by
    /// `createdAt` as a string and reverses; asserted so a change to parsed-time sorting - which would
    /// reorder malformed timestamps - is a failing test rather than a silent reordering.
    #[test]
    fn the_newest_card_is_the_one_read() {
        let c = comments(&[
            ("2026-10-13T00:00:00Z", "status card: headline 1/2"),
            ("2026-10-14T00:00:00Z", "status card: headline 9/10"),
        ]);
        let r = card_reading(&c, "M1");
        assert_eq!(
            (r.current, r.of),
            (Some(9), Some(10)),
            "the newest card decides"
        );
    }

    /// **A card whose text has no number is `headline_text` with no `yes_no`**, and its `text` is the
    /// window past the marker — **truncated in CHARACTERS**, so a non-ASCII card cannot slice
    /// mid-character. That is the char-boundary panic this port has hit twice, and the reason the
    /// window is taken with `chars().take()` rather than a byte slice.
    #[test]
    fn a_card_with_no_number_reports_no_number_and_cannot_slice_a_character() {
        let body = format!("status card: headline {}", "é".repeat(200));
        let c = comments(&[("2026-10-14T00:00:00Z", &body)]);
        let r = card_reading(&c, "M1");
        assert_eq!(r.kind, "headline_text");
        assert_eq!(r.current, None);
        assert!(
            r.text.chars().count() <= HEADLINE_WINDOW,
            "the window is bounded"
        );
        assert!(r.text.chars().count() > 0, "and it is not empty");
    }

    /// `registry_numbers` returns **every** pair, `secondary` included, and **skips a row missing
    /// either half rather than reading it as zero** — a missing number is not a claim of no progress.
    #[test]
    fn registry_numbers_covers_secondary_and_skips_incomplete_rows() {
        use crate::registries::Milestone;
        let m = Milestone {
            key: "m1".into(),
            issue: 2029,
            id: "M1".into(),
            title: String::new(),
            lab: String::new(),
            status: String::new(),
            headline: json!({"current": 37, "of": 232}),
            secondary: vec![
                json!({"current": 20, "of": 40}),
                json!({"current": 5}),
                json!({}),
                json!({"current": 1.9, "of": 2.9}),
            ],
            panel: String::new(),
            base_artifact: String::new(),
            mechanisms: vec![],
            lines: vec![],
            next: String::new(),
        };
        // **Floats truncate, matching Python's `int()`.**
        assert_eq!(registry_numbers(&m), vec![(37, 232), (20, 40), (1, 2)]);
    }

    /// A milestone with no `current`/`of` at all yields no pairs, so the rule has nothing to compare.
    #[test]
    fn a_milestone_with_no_numbers_yields_no_pairs() {
        use crate::registries::Milestone;
        let m = Milestone {
            key: "m".into(),
            issue: 1,
            id: "M".into(),
            title: String::new(),
            lab: String::new(),
            status: String::new(),
            headline: json!({}),
            secondary: vec![],
            panel: String::new(),
            base_artifact: String::new(),
            mechanisms: vec![],
            lines: vec![],
            next: String::new(),
        };
        assert!(registry_numbers(&m).is_empty());
    }
}
