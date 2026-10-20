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

use crate::prgates::fmt_num;
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

// ---------------------------------------------------------------------------------------------
// The `gh` chain: the cache, one call per milestone, and the trimmed comments.
// ---------------------------------------------------------------------------------------------

/// Cache schema and freshness, copied from Python.
pub const SCHEMA: u32 = 1;
pub const CACHE_TTL_SECONDS: i64 = 15 * 60;
/// How many of the newest comments are kept per issue.
pub const COMMENT_KEEP: usize = 5;
/// How much of each comment body is kept, before redaction.
pub const COMMENT_CHARS: usize = 2000;

/// `status.cache_path` — the live tier's `github.json`.
///
/// **In the live tier, not the repo**, so a lab's fetch does not dirty its checkout — and
/// **`_cache_save` is best-effort because a read-only checkout must not break `status`.**
pub fn cache_path(paths: &crate::state::StatePaths) -> std::path::PathBuf {
    paths.live.join("github.json")
}

/// `status._cache_load` — the cache, or empty on anything unreadable.
///
/// **A corrupt cache reads as empty rather than as an error**: the rule then refetches, and
/// **reporting a JSON error here would turn a recoverable cache problem into a failure of the rule.**
pub fn cache_load(paths: &crate::state::StatePaths) -> Value {
    let Ok(text) = std::fs::read_to_string(cache_path(paths)) else {
        return Value::Object(Default::default());
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(v @ Value::Object(_)) => v,
        _ => Value::Object(Default::default()),
    }
}

/// `status._cache_save` — best-effort; **a read-only checkout must not break `status`.**
pub fn cache_save(paths: &crate::state::StatePaths, cache: &Value) {
    if let Some(parent) = cache_path(paths).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(cache) {
        let _ = std::fs::write(cache_path(paths), format!("{text}\n"));
    }
}

/// `status._fresh` — fetched within the TTL.
fn fresh(item: Option<&Value>, now: i64) -> bool {
    let Some(item) = item else { return false };
    if !item.is_object() {
        return false;
    }
    let Some(fetched) = item.get("fetched_at").and_then(Value::as_str) else {
        return false;
    };
    let Some(then) = crate::multilabrules::parse_claimed_at(fetched) else {
        return false;
    };
    // **Seconds compared against seconds.** Python computes hours and multiplies back, which is the
    // same comparison with a float in the middle - and a float in the middle is where a boundary
    // becomes a coin toss.
    now - then <= CACHE_TTL_SECONDS
}

/// `status._trim_comments` — the newest `COMMENT_KEEP` comments, each body capped and **redacted**.
///
/// **Redaction is not optional here.** The cache is written to disk, so a token pasted into an issue
/// comment would be persisted by a read of it - **`core.redact` exists for exactly this and the port
/// keeps it on the write path rather than only on display.**
pub fn trim_comments(comments: Option<&Value>) -> Vec<Value> {
    let all = comments
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    all.iter()
        .rev()
        .take(COMMENT_KEEP)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .filter(|c| c.is_object())
        .map(|c| {
            let created = c
                .get("createdAt")
                .or_else(|| c.get("created_at"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let author = c
                .get("author")
                .and_then(|a| a.get("login"))
                .or_else(|| c.get("user").and_then(|u| u.get("login")))
                .and_then(Value::as_str)
                .unwrap_or("");
            let body: String = c
                .get("body")
                .and_then(Value::as_str)
                .unwrap_or("")
                .chars()
                .take(COMMENT_CHARS)
                .collect();
            serde_json::json!({
                "createdAt": created,
                "author": author,
                "body": crate::prgates::redact_public(&body),
            })
        })
        .collect()
}

/// `status.gh_json` — one `gh` invocation, parsed, with a timeout that **drains the pipes**.
///
/// # The bug this had
///
/// The first version polled `try_wait()` and then called `wait_with_output()`. **That deadlocks on any
/// output larger than a pipe buffer**: `try_wait` polls the child, but nothing reads stdout, so once
/// `gh` fills the ~64 KB pipe it blocks writing and the child never exits — **and the timeout then
/// fired on a call that was working perfectly.**
///
/// **Found by the live test, not by reading.** `gh issue view --json comments` on an active issue is
/// exactly the size that crosses the buffer, so **the unit tests could not see it** and the first live
/// run failed with `gh timed out` on a `gh` that returns in under a second.
///
/// **Python's `subprocess.run(timeout=…)` reads stdout and stderr concurrently**, which is why the
/// original never had this. The port now does the same: **two reader threads, then poll for exit.**
/// The threads are joined before returning, so no reader outlives the call.
/// The single `gh` entry point, exposed so `git delete-merged` proves a merge the same way.
///
/// **It drains the pipes and enforces the timeout** - the fix for the deadlock the live chain test
/// caught - **so every caller gets that behaviour rather than re-implementing the spawn.**
pub fn gh_json(subargs: &[&str], repo: &std::path::Path) -> (bool, Value, String) {
    use std::io::Read;
    use std::process::{Command, Stdio};

    let mut child = match Command::new("gh")
        .args(subargs)
        .current_dir(repo)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return (false, Value::Null, format!("cannot run gh: {e}")),
    };

    let mut out_pipe = child.stdout.take();
    let mut err_pipe = child.stderr.take();
    let out_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(p) = out_pipe.as_mut() {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    });
    let err_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(p) = err_pipe.as_mut() {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Ok(None) => {
                let _ = child.kill();
                break None;
            }
            Err(e) => {
                let _ = child.kill();
                // Join the readers so they do not outlive the call even on this path.
                let _ = out_handle.join();
                let _ = err_handle.join();
                return (false, Value::Null, format!("gh failed: {e}"));
            }
        }
    };
    let stdout = out_handle.join().unwrap_or_default();
    let stderr = err_handle.join().unwrap_or_default();

    let Some(status) = status else {
        return (false, Value::Null, "gh timed out".into());
    };
    if !status.success() {
        // **Python's `gh_json` renders the failure the same way `_run` does**, so a caller comparing
        // the two implementations sees one string: the command text, the exit code, and the output
        // lines joined by spaces. **`git status --json` differed in exactly these three `reason`
        // fields and agreed on the other twenty-seven** - which is how the rendering was found.
        // **STDERR FIRST, then stdout** — Python's order, and the reverse of the obvious one.
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&stderr),
            String::from_utf8_lossy(&stdout)
        );
        let command = format!("gh {}", subargs.join(" "));
        return (
            false,
            Value::Null,
            crate::githelp::run_failure(&command, status.code().unwrap_or(1), &combined),
        );
    }
    let text = String::from_utf8_lossy(&stdout).to_string();
    match serde_json::from_str::<Value>(text.trim()) {
        Ok(v) => (true, v, String::new()),
        Err(e) => (
            false,
            Value::Null,
            format!("gh returned unparseable JSON: {e}"),
        ),
    }
}

/// What `github_issues` returns, in Python's shape.
#[derive(Debug, Clone)]
pub struct IssueSnapshot {
    /// **False means the rule reports that it could NOT cross-check** — which is not the same as
    /// agreeing, and is the distinction the port fixed in an earlier unit.
    pub available: bool,
    pub reason: String,
    pub issues: Value,
}

/// `status.github_issues` — milestone-issue metadata, optionally with the newest comments.
///
/// **One `gh` call per issue, cached for `CACHE_TTL_SECONDS`.** The cache lives in the live tier and
/// is written best-effort. **A missing issue makes the whole snapshot unavailable rather than
/// silently absent** — the rule then reports that it did not check, which is honest, instead of
/// comparing against a hole.
pub fn github_issues(
    paths: &crate::state::StatePaths,
    numbers: &[i64],
    with_comments: bool,
    now: i64,
    refresh: bool,
) -> IssueSnapshot {
    let cache = cache_load(paths);
    let mut items: serde_json::Map<String, Value> = cache
        .get("issues")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut errors: Vec<String> = Vec::new();
    let mut fetched_any = false;

    for num in numbers {
        let key = num.to_string();
        let existing = items.get(&key).cloned();
        let usable = existing.as_ref().is_some_and(|i| fresh(Some(i), now))
            && (!with_comments || existing.as_ref().and_then(|i| i.get("comments")).is_some());
        if !refresh && usable {
            continue;
        }
        let mut fields = "number,title,state,updatedAt,url".to_string();
        if with_comments {
            fields.push_str(",comments");
        }
        let (ok, data, err) = gh_json(&["issue", "view", &key, "--json", &fields], &paths.repo);
        fetched_any = true;
        if !ok || !data.is_object() {
            errors.push(if err.is_empty() {
                format!("`gh issue view {key}` returned no issue object")
            } else {
                err
            });
            continue;
        }
        let s = |k: &str| {
            data.get(k)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };
        let mut entry = serde_json::json!({
            "number": data.get("number").cloned().unwrap_or(Value::from(*num)),
            "title": s("title"),
            "state": s("state"),
            "updatedAt": s("updatedAt"),
            "url": s("url"),
            "fetched_at": crate::receipt::now_rfc3339(),
        });
        if with_comments {
            entry["comments"] = Value::Array(trim_comments(data.get("comments")));
        } else if let Some(kept) = existing.as_ref().and_then(|i| i.get("comments")) {
            entry["comments"] = kept.clone();
        }
        items.insert(key, entry);
    }

    let mut merged = cache.as_object().cloned().unwrap_or_default();
    merged.insert("schema".into(), Value::from(SCHEMA));
    merged.insert("issues".into(), Value::Object(items.clone()));
    if fetched_any {
        cache_save(paths, &Value::Object(merged));
    }

    let missing: Vec<i64> = numbers
        .iter()
        .copied()
        .filter(|n| !items.get(&n.to_string()).is_some_and(Value::is_object))
        .collect();
    if !missing.is_empty() {
        let reason = if errors.is_empty() {
            format!(
                "no data for issue(s) {}",
                missing
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            // The same failure per issue (no `gh`, no network) is reported once.
            errors.dedup();
            errors.join("; ")
        };
        return IssueSnapshot {
            available: false,
            reason,
            issues: Value::Object(items),
        };
    }
    IssueSnapshot {
        available: true,
        reason: String::new(),
        issues: Value::Object(items),
    }
}

/// `status.OPEN_PR_LIMIT` / `MERGED_PR_LIMIT` — how many PRs one `gh pr list` call asks for.
pub const OPEN_PR_LIMIT: usize = 50;
pub const MERGED_PR_LIMIT: usize = 100;
/// `status.BRANCH_PR_LIMIT` — at most this many branches get a `gh` call per run.
pub const BRANCH_PR_LIMIT: usize = 25;

/// Seconds since an entry's `fetched_at`, or 0 when it cannot be read.
fn age_seconds(item: &Value, now: i64) -> i64 {
    item.get("fetched_at")
        .and_then(Value::as_str)
        .and_then(crate::multilabrules::parse_claimed_at)
        .map(|then| (now - then).max(0))
        .unwrap_or(0)
}

/// `status.github_prs` — open and merged PRs, from `gh` or from a cache younger than the TTL.
///
/// Returns Python's section shape (`available`, `reason`, `fetched_at`, `from_cache`,
/// `age_seconds`, `open`, `merged`, the two limits). **A failed fetch is cached too**, as Python
/// does, so an offline lab is not re-billed a `gh` timeout on every view; `--refresh` overrides.
pub fn github_prs(paths: &crate::state::StatePaths, refresh: bool, now: i64) -> Value {
    let mut cache = cache_load(paths);
    if let Some(cached) = cache.get("prs") {
        if !refresh && fresh(Some(cached), now) {
            let mut out = cached.clone();
            out["from_cache"] = Value::Bool(true);
            out["age_seconds"] = Value::from(age_seconds(cached, now));
            return out;
        }
    }
    let stamp = crate::receipt::now_rfc3339();
    let open_limit = OPEN_PR_LIMIT.to_string();
    let merged_limit = MERGED_PR_LIMIT.to_string();
    let (open_ok, open_prs, open_err) = gh_json(
        &[
            "pr",
            "list",
            "--state",
            "open",
            "--limit",
            &open_limit,
            "--json",
            "number,title,headRefName,url,createdAt,updatedAt",
        ],
        &paths.repo,
    );
    let (merged_ok, merged_prs, merged_err) = gh_json(
        &[
            "pr",
            "list",
            "--state",
            "merged",
            "--limit",
            &merged_limit,
            "--json",
            "number,title,headRefName,headRefOid,url,mergedAt",
        ],
        &paths.repo,
    );
    let ok = open_ok && merged_ok;
    let reason = if ok {
        String::new()
    } else if !open_err.is_empty() {
        open_err
    } else if !merged_err.is_empty() {
        merged_err
    } else {
        "gh failed with no error text".to_string()
    };
    let list = |v: Value| {
        Value::Array(if ok {
            v.as_array().cloned().unwrap_or_default()
        } else {
            Vec::new()
        })
    };
    let section = serde_json::json!({
        "available": ok,
        "reason": reason,
        "fetched_at": stamp,
        "from_cache": false,
        "open": list(open_prs),
        "merged": list(merged_prs),
        "open_limit": OPEN_PR_LIMIT,
        "merged_limit": MERGED_PR_LIMIT,
    });
    if !cache.is_object() {
        cache = Value::Object(Default::default());
    }
    cache["schema"] = Value::from(SCHEMA);
    cache["prs"] = section.clone();
    cache_save(paths, &cache);
    let mut out = section;
    out["age_seconds"] = Value::from(0);
    out
}

/// What `github_branch_prs` returns: the PR (any state) per local branch, where one is known.
#[derive(Debug, Clone, Default)]
pub struct BranchPrs {
    /// **True when at least one branch has an answer, or nothing failed.** A partial answer is
    /// still used; only "every call failed" makes the check unavailable.
    pub available: bool,
    pub reason: String,
    pub queried: usize,
    /// Branch name to `{"fetched_at", "pr"}`, for branches with an entry.
    pub items: serde_json::Map<String, Value>,
}

/// `status.github_branch_prs` — the PR whose head is each local branch, cached per branch.
///
/// **One bounded call per branch** because older merged PRs fall outside the newest-100 window of
/// `gh pr list --state merged`, so a branch can look clean while its PR merged long ago. At most
/// `BRANCH_PR_LIMIT` calls per run; a merged PR is preferred over an open one for the same head.
pub fn github_branch_prs(
    paths: &crate::state::StatePaths,
    branches: &[String],
    refresh: bool,
    now: i64,
) -> BranchPrs {
    let mut cache = cache_load(paths);
    let mut store: serde_json::Map<String, Value> = cache
        .get("branch_prs")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut queried = 0usize;
    let mut errors: Vec<String> = Vec::new();
    for name in branches.iter().filter(|n| !n.is_empty()) {
        if !refresh && fresh(store.get(name), now) {
            continue;
        }
        if queried >= BRANCH_PR_LIMIT {
            errors.push(format!(
                "only the first {BRANCH_PR_LIMIT} branches were queried this run"
            ));
            break;
        }
        let (ok, data, err) = gh_json(
            &[
                "pr",
                "list",
                "--head",
                name,
                "--state",
                "all",
                "--limit",
                "3",
                "--json",
                "number,title,state,headRefOid,mergedAt,url",
            ],
            &paths.repo,
        );
        queried += 1;
        if !ok {
            errors.push(if err.is_empty() {
                format!("`gh pr list --head {name}` failed")
            } else {
                err
            });
            continue;
        }
        let rows: Vec<Value> = data
            .as_array()
            .map(|a| a.iter().filter(|r| r.is_object()).cloned().collect())
            .unwrap_or_default();
        let merged = rows.iter().find(|r| {
            r.get("state")
                .and_then(Value::as_str)
                .is_some_and(|s| s.eq_ignore_ascii_case("MERGED"))
        });
        let pr = merged.or(rows.first()).cloned().unwrap_or(Value::Null);
        store.insert(
            name.clone(),
            serde_json::json!({"fetched_at": crate::receipt::now_rfc3339(), "pr": pr}),
        );
    }
    if !cache.is_object() {
        cache = Value::Object(Default::default());
    }
    cache["schema"] = Value::from(SCHEMA);
    cache["branch_prs"] = Value::Object(store.clone());
    cache_save(paths, &cache);
    let items: serde_json::Map<String, Value> = branches
        .iter()
        .filter_map(|n| {
            store
                .get(n)
                .filter(|v| v.as_object().is_some_and(|o| !o.is_empty()))
                .map(|v| (n.clone(), v.clone()))
        })
        .collect();
    // The same failure repeated per branch (no `gh`, no network) is reported once.
    errors.dedup();
    BranchPrs {
        available: !items.is_empty() || errors.is_empty(),
        reason: errors.join("; "),
        queried,
        items,
    }
}

#[cfg(test)]
mod chain_tests {
    use super::*;
    use serde_json::json;

    /// **`trim_comments` keeps the NEWEST five, in order, and caps each body.**
    ///
    /// The cap is in **characters**, and **the body is redacted**, which is the part that matters
    /// beyond display: the cache is written to disk, so **a token pasted into an issue comment would
    /// be persisted by anyone who ran `status`.**
    #[test]
    fn trim_comments_keeps_the_newest_five_and_caps_and_redacts_each_body() {
        let all: Vec<Value> = (0..8)
            .map(|i| json!({"createdAt": format!("2026-10-{:02}T00:00:00Z", i + 1), "body": format!("c{i}")}))
            .collect();
        let out = trim_comments(Some(&Value::Array(all)));
        assert_eq!(out.len(), COMMENT_KEEP, "the newest five");
        // Newest five of c0..c7 are c3..c7, and **kept in chronological order**, not reversed.
        assert_eq!(out[0].get("body").and_then(Value::as_str), Some("c3"));
        assert_eq!(out[4].get("body").and_then(Value::as_str), Some("c7"));

        // A long body is capped in CHARACTERS, so a multi-byte body cannot be cut mid-character.
        let long = "é".repeat(COMMENT_CHARS + 500);
        let one = json!({"createdAt": "2026-10-14T00:00:00Z", "body": long});
        let out = trim_comments(Some(&Value::Array(vec![one])));
        assert_eq!(
            out[0]
                .get("body")
                .and_then(Value::as_str)
                .unwrap()
                .chars()
                .count(),
            COMMENT_CHARS
        );

        // And a token in a comment body does not reach the cache.
        let secret =
            json!({"createdAt": "2026-10-14T00:00:00Z", "body": "token ghp_ABCDEFGHIJKLMNOP here"});
        let out = trim_comments(Some(&Value::Array(vec![secret])));
        let body = out[0].get("body").and_then(Value::as_str).unwrap();
        assert!(
            !body.contains("ghp_ABCDEFGHIJKLMNOP"),
            "redacted before it reaches disk: {body}"
        );
        assert!(body.contains("<redacted>"));
    }

    /// **An author is read from `author.login` or `user.login`**, because the two `gh` shapes differ
    /// and a missing author is empty rather than absent.
    #[test]
    fn an_author_is_read_from_either_shape() {
        let a = json!({"createdAt": "t", "author": {"login": "kimi"}, "body": "x"});
        let b = json!({"created_at": "t", "user": {"login": "codex"}, "body": "x"});
        let out = trim_comments(Some(&Value::Array(vec![a, b])));
        assert_eq!(out[0].get("author").and_then(Value::as_str), Some("kimi"));
        assert_eq!(out[1].get("author").and_then(Value::as_str), Some("codex"));
        // `created_at` is accepted as well as `createdAt`.
        assert_eq!(out[1].get("createdAt").and_then(Value::as_str), Some("t"));
    }

    /// **Freshness is a TTL boundary, asserted on both sides.** Python computes hours and multiplies
    /// back to seconds; the port compares seconds directly, **and a boundary that is reasoned about
    /// rather than tested is how this port produced two wrong constants earlier.**
    #[test]
    fn the_cache_ttl_boundary_holds_on_both_sides() {
        // **Two fixed instants 900 seconds apart**, rather than a converted epoch: the earlier
        // attempt reached for an internal civil-date helper that is not public, and **a test that
        // needs a helper exposed to exist is a test shaping the code it is checking.**
        let now = crate::multilabrules::parse_claimed_at("2026-10-14T12:00:00Z")
            .expect("a valid instant");
        let at = |instant: &str| json!({ "fetched_at": instant });
        // Exactly the TTL: still fresh.
        assert!(
            fresh(Some(&at("2026-10-14T11:45:00Z")), now),
            "exactly {CACHE_TTL_SECONDS}s is still fresh"
        );
        // One second past: not fresh.
        assert!(
            !fresh(Some(&at("2026-10-14T11:44:59Z")), now),
            "one second past is not"
        );
        assert!(!fresh(None, now), "no entry is not fresh");
        assert!(
            !fresh(Some(&json!({"fetched_at": "not a time"})), now),
            "unparseable is not fresh"
        );
        assert!(
            !fresh(Some(&json!("a string")), now),
            "not an object is not fresh"
        );
    }

    /// **A missing or corrupt cache reads as empty**, because the rule then refetches — **reporting a
    /// JSON error here would turn a recoverable cache problem into a failure of the rule.**
    /// **A missing or corrupt cache reads as empty**, because the rule then refetches — **reporting a
    /// JSON error here would turn a recoverable cache problem into a failure of the rule.**
    #[test]
    fn a_missing_or_corrupt_cache_reads_as_empty_rather_than_failing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = crate::state::StatePaths {
            repo: dir.path().to_path_buf(),
            live: dir.path().join("live"),
        };
        let empty =
            |p: &crate::state::StatePaths| cache_load(p).as_object().is_some_and(|o| o.is_empty());
        assert!(empty(&paths), "missing reads as empty");

        std::fs::create_dir_all(&paths.live).expect("mkdir");
        std::fs::write(cache_path(&paths), "{not json").expect("write");
        assert!(empty(&paths), "corrupt reads as empty");

        // And a save/load round trip works, so the path is not merely always-empty.
        let good = json!({"schema": 1, "issues": {"2029": {"number": 2029}}});
        cache_save(&paths, &good);
        assert_eq!(cache_load(&paths), good);
    }
}

#[cfg(test)]
mod live_chain_tests {
    use super::*;

    /// **The chain, end to end, against the live `gh`** — the verification that the parts agree with
    /// each other, which no unit test of a part can give.
    ///
    /// **Ignored by default**, because it needs a network and a checkout of the real repository; run it
    /// with `cargo test -- --ignored live_chain`. **It asserts the SHAPE, not the content**: an issue's
    /// comments change, so a content assertion would be the exact "true of the tree when written"
    /// mistake this port has now made three times.
    #[test]
    #[ignore = "needs gh and a network"]
    fn live_chain_returns_a_usable_snapshot() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = crate::state::StatePaths {
            repo: repo.clone(),
            live: dir.path().join("live"),
        };
        let now =
            crate::multilabrules::parse_claimed_at(&crate::receipt::now_rfc3339()).expect("now");
        let snap = github_issues(&paths, &[2029], true, now, true);
        assert!(
            snap.available,
            "gh is reachable and 2029 exists: {}",
            snap.reason
        );
        let item = snap.issues.get("2029").expect("the issue is present");
        assert_eq!(item.get("number").and_then(Value::as_i64), Some(2029));
        assert!(!item
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty());
        let comments = item
            .get("comments")
            .and_then(Value::as_array)
            .expect("comments");
        assert!(
            comments.len() <= COMMENT_KEEP,
            "kept at most {COMMENT_KEEP}"
        );
        for c in comments {
            // The three keys `_trim_comments` produces, and **no others**: a leaked key would mean the
            // cache holds more of the issue than the trim intends.
            let o = c.as_object().expect("an object");
            let mut keys: Vec<&String> = o.keys().collect();
            keys.sort();
            assert_eq!(
                keys,
                vec!["author", "body", "createdAt"],
                "exactly the trimmed keys"
            );
        }
        // **And it was cached**, so a second call is served from disk.
        assert!(
            cache_load(&paths).get("issues").is_some(),
            "the fetch was written to the cache"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The decision after the reading - all that separates the rule from registration.
// ---------------------------------------------------------------------------------------------

/// One finding from the rule, in Python's shape.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistryFinding {
    pub severity: String,
    pub ok: bool,
    pub message: String,
    pub path: String,
    pub fix: String,
}

/// `status._rule_registry_vs_github` — the registry headline against the newest card.
///
/// # The decision tree, ported branch for branch
///
/// **Six outcomes, and three of them are "did not compare" rather than agreement** — which is the
/// distinction this port exists to preserve:
///
/// | condition | outcome |
/// |---|---|
/// | GitHub unavailable | **warn, ok=false** — *not* a pass |
/// | no comments cached | info, ok=true — not compared |
/// | no card in the comments | info, ok=true — not compared |
/// | qualitative card, text contains the word | **silence** — the card agrees |
/// | qualitative card, registry holds a number | warn, ok=false |
/// | no `current`, registry holds nothing | info, ok=true — not compared |
/// | `(current, of)` in the registry's pairs | silence |
/// | otherwise | warn, ok=false |
///
/// **The `reg_repr` string is built even when nothing is wrong**, because two of the messages quote
/// it — **and it renders a missing `current` as `?` rather than as nothing**, which is how a reader
/// tells "the registry has no number" from "the registry has a number of zero".
pub fn registry_vs_github(
    paths: &crate::state::StatePaths,
    milestones: &[crate::registries::Milestone],
    refresh: bool,
    now: i64,
) -> Vec<RegistryFinding> {
    let numbers: Vec<i64> = milestones.iter().map(|m| m.issue).collect();
    let snap = github_issues(paths, &numbers, true, now, refresh);
    if !snap.available {
        return vec![RegistryFinding {
            severity: "warn".into(),
            ok: false,
            message: format!(
                "the registry was NOT cross-checked against GitHub (unavailable): {}",
                snap.reason
            ),
            path: paths
                .repo
                .join("docs/harness/milestones.json")
                .display()
                .to_string(),
            fix: String::new(),
        }];
    }

    let mut out = Vec::new();
    for m in milestones {
        let item = snap
            .issues
            .get(m.issue.to_string())
            .cloned()
            .unwrap_or(Value::Null);
        let Some(comments) = item.get("comments") else {
            out.push(RegistryFinding {
                severity: "info".into(),
                ok: true,
                message: format!(
                    "{} #{}: no comments cached for this issue; not compared",
                    m.id, m.issue
                ),
                path: paths
                    .repo
                    .join("docs/harness/milestones.json")
                    .display()
                    .to_string(),
                fix: String::new(),
            });
            continue;
        };

        let reading = card_reading(comments.as_array().map(Vec::as_slice).unwrap_or(&[]), &m.id);
        let head = &m.headline;
        let reg_cur = head.get("current").and_then(Value::as_f64);
        let reg_of = head.get("of").and_then(Value::as_f64);
        let reg_text = head
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_lowercase();
        let label = head
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or("headline");
        let reg_repr = format!(
            "{label} {}{}",
            match reg_cur {
                Some(c) => fmt_num(c),
                None => "?".into(),
            },
            match reg_of {
                Some(o) => format!("/{}", fmt_num(o)),
                None => String::new(),
            }
        );
        let ms_path = paths
            .repo
            .join("docs/harness/milestones.json")
            .display()
            .to_string();
        let registry_pairs = registry_numbers(m);

        if reading.kind == "no_card" {
            out.push(RegistryFinding {
                severity: "info".into(),
                ok: true,
                message: format!(
                    "{} #{}: no status card found in the last {} comments; nothing to compare",
                    m.id,
                    m.issue,
                    comments.as_array().map(Vec::len).unwrap_or(0)
                ),
                path: ms_path,
                fix: String::new(),
            });
            continue;
        }

        if reading.current.is_none() {
            // **A qualitative card agrees when the registry records the same answer** - the word is
            // matched against the registry's own text, not parsed, which is why it is a substring test.
            if !reading.yes_no.is_empty() && reg_text.contains(&reading.yes_no) {
                continue;
            }
            if !reading.yes_no.is_empty() && (reg_of.is_some() || reg_cur.is_some()) {
                out.push(RegistryFinding {
                    severity: "warn".into(),
                    ok: false,
                    message: format!(
                        "{} #{}: registry states a number ({reg_repr}), the newest card ({}) answers {}: {}",
                        m.id, m.issue, reading.ts, crate::prgates::py_repr_public(&reading.yes_no), crate::prgates::py_repr_public(&reading.text)
                    ),
                    path: ms_path,
                    fix: format!(
                        "reconcile docs/harness/milestones.json with issue #{} (or make the card state `{}: n/n`)",
                        m.issue, m.id
                    ),
                });
                continue;
            }
            out.push(RegistryFinding {
                severity: "info".into(),
                ok: true,
                message: format!(
                    "{} #{}: newest card ({}) has no machine-readable number; registry says {reg_repr}; card text: {}",
                    m.id, m.issue, reading.ts, crate::prgates::py_repr_public(&reading.text)
                ),
                path: ms_path,
                fix: String::new(),
            });
            continue;
        }

        if matches!((reading.current, reading.of), (Some(c), Some(o)) if registry_pairs.contains(&(c, o)))
        {
            continue;
        }
        out.push(RegistryFinding {
            severity: "warn".into(),
            ok: false,
            message: format!(
                "{} #{}: registry says {reg_repr}, newest card ({}) says {}/{} ({} in {})",
                m.id,
                m.issue,
                reading.ts,
                reading.current.map(|c| c.to_string()).unwrap_or_default(),
                reading.of.map(|o| o.to_string()).unwrap_or_default(),
                reading.kind,
                crate::prgates::py_repr_public(&reading.text)
            ),
            path: ms_path,
            fix: format!(
                "reconcile docs/harness/milestones.json with issue #{}, then re-run `uor-harness status --refresh`",
                m.issue
            ),
        });
    }
    out
}
