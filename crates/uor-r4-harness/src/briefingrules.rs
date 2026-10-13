//! The `briefing` rules, as Rust (D30 check-port step 8 of 10).
//!
//! # The three `briefing` rules
//!
//! All three are **agreement checks between the registry and the prose a lab actually reads**, and
//! that is the property that makes them worth having:
//!
//! | rule | what it catches |
//! |---|---|
//! | `briefing.sources_exist` | a milestone naming a mechanism brief or panel that does not exist |
//! | `briefing.milestone_numbers_agree` | a registry headline the session-goal text does not state |
//! | `briefing.lines_agree_with_prompts` | a line declared in a prompt but absent from — or counted differently in — the registry |
//!
//! ## Why agreement rather than validity
//!
//! **The registry and `docs/labs/session-goal.md` are two descriptions of one plan, and a lab reads
//! the prose at session start while the harness reads the registry.** When they disagree, the lab
//! works the number in the prose and `check` reports the number in the registry — **and both are
//! right, which is the failure.** A mismatch is not a typo to fix later; it is two authorities.
//!
//! ## The subtlety `lines_agree_with_prompts` carries
//!
//! **The contract also shows the PR-line *format*** — `Line: <name> · count n/3 · headline …` — which
//! has a count in it and **must not be mistaken for a declaration.** A regex that matches any
//! `count n/m` would read the format example as a real line and report a mismatch against a line that
//! does not exist. So a declaration requires **`Line "` or `Line “` followed by a quoted name**, and
//! the format example has no quoted name after the keyword.

use crate::registries::Registries;
use crate::state::StatePaths;

pub use crate::preprules::Finding;

const REGISTRY: &str = "docs/harness/milestones.json";
const SESSION_GOAL: &str = "docs/labs/session-goal.md";

fn warn(rule_id: &'static str, message: String, path: &str, fix: String) -> Finding {
    Finding {
        rule_id,
        severity: "warn",
        message,
        path: path.into(),
        fix,
    }
}

/// `briefing.sources_exist` — every milestone's mechanism briefs and panels resolve to real files.
///
/// **A brief that cites a missing file teaches the wrong thing.** The milestone is the entry point a
/// lab is sent to, so a mechanism id pointing at nothing sends them to a file that was renamed or
/// never written — and they arrive at the session with a wrong map.
pub fn sources_exist(paths: &StatePaths, regs: &Registries) -> Vec<Finding> {
    let mut out = Vec::new();
    for m in &regs.milestones {
        for mech in &m.mechanisms {
            let f = paths
                .repo
                .join("docs")
                .join("mechanisms")
                .join(format!("{mech}.md"));
            if !f.is_file() {
                out.push(warn(
                    "briefing.sources_exist",
                    format!(
                        "{} names mechanism {mech:?} but docs/mechanisms/{mech}.md does not exist",
                        m.id
                    ),
                    REGISTRY,
                    format!("fix the mechanism id or add docs/mechanisms/{mech}.md"),
                ));
            }
        }
        if m.panel.is_empty() {
            continue;
        }
        if regs.panel_for(m).is_none() {
            out.push(warn(
                "briefing.sources_exist",
                format!(
                    "{} names panel {:?} but no panel file defines it",
                    m.id, m.panel
                ),
                REGISTRY,
                format!("add {} to docs/harness/panels/", m.panel),
            ));
        }
    }
    out
}

/// `briefing.milestone_numbers_agree` — the registry headline agrees with `session-goal.md`.
///
/// The check is deliberately loose: it looks for **`current/of` as a substring** of the session-goal
/// text, so any sentence stating the number satisfies it. **A stricter match would fail on wording
/// changes that do not change the number**, and the rule exists to catch the number, not the prose.
pub fn milestone_numbers_agree(paths: &StatePaths, regs: &Registries) -> Vec<Finding> {
    let goal = paths.repo.join(SESSION_GOAL);
    let Ok(text) = std::fs::read_to_string(&goal) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for m in &regs.milestones {
        let h = &m.headline;
        // A vacuous headline has no number to agree about.
        if h.get("vacuous").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue;
        }
        let Some(cur) = h.get("current").and_then(|v| v.as_i64()) else {
            continue;
        };
        let of = h.get("of").and_then(|v| v.as_i64()).unwrap_or(0);
        let needle = format!("{cur}/{of}");
        if text.contains(&needle) {
            continue;
        }
        out.push(warn(
            "briefing.milestone_numbers_agree",
            format!(
                "{}: registry says {needle}, which {SESSION_GOAL} does not state",
                m.id
            ),
            SESSION_GOAL,
            "update the registry entry (or the session-goal text) so a lab cannot work the wrong \
             number"
                .into(),
        ));
    }
    out
}

/// A line declared in the session-goal prompts: `(where, name, count, of)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declared {
    pub where_: String,
    pub name: String,
    pub count: i64,
    pub of: i64,
}

/// Every `Line "<name>" … count n/m` declared in the live session-goal prompts.
///
/// **A declaration requires a QUOTED name after `Line`.** The contract separately shows the PR-line
/// *format* — `Line: <name> · count n/3 · headline …` — which carries a count and **must not be read
/// as a declaration**; it has no quoted name, so it is skipped. Matching any `count n/m` would report
/// a mismatch against a line that does not exist, which is worse than not checking.
///
/// Only straight and curly double quotes open a declaration, matching Python.
pub fn declared_lines(text: &str) -> Vec<Declared> {
    let mut built: Vec<Declared> = Vec::new();
    // **Walk CHAR boundaries, not byte offsets.** My first version advanced a byte index by 1 and
    // sliced `text[i..]`, which panics the moment it lands inside a multi-byte character - and this
    // file is full of them (curly quotes, the `·` separator). `char_indices` cannot produce a
    // non-boundary offset, so the whole class of bug is gone rather than guarded against.
    for (i, _) in text.char_indices() {
        if !text[i..].starts_with("Line") {
            continue;
        }
        // Word start: the character before must not be alphanumeric.
        let at_word_start = i == 0
            || !text[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric());
        if !at_word_start {
            continue;
        }
        let after_line = i + "Line".len();
        let rest = text[after_line..].trim_start();
        let indent = text[after_line..].len() - rest.len();
        // An opening quote must follow, or this is not a declaration. **The contract's format
        // example - `Line: <name> · count n/3` - has a colon, so it is skipped here.**
        let (name_start, closer) = if rest.strip_prefix('"').is_some() {
            (after_line + indent + 1, '"')
        } else if rest.strip_prefix('\u{201c}').is_some() {
            (after_line + indent + '\u{201c}'.len_utf8(), '\u{201d}')
        } else {
            continue;
        };
        // The name runs to the closing quote, on the same line.
        let tail = &text[name_start..];
        let line_end = tail.find('\n').unwrap_or(tail.len());
        let same_line = &tail[..line_end];
        let Some(close_rel) = same_line.find(closer) else {
            continue;
        };
        let name = same_line[..close_rel].trim().to_string();
        if name.is_empty() {
            continue;
        }
        // `count n/m` must appear within the next 80 characters - the same window Python uses.
        let after_close = &same_line[close_rel..];
        let window: String = after_close.chars().take(81).collect();
        if let Some((count, of)) = count_pair(&window) {
            built.push(Declared {
                where_: "session-goal contract".into(),
                name,
                count,
                of,
            });
        }
    }
    // De-duplicate by normalised name, keeping the first (the contract form).
    let mut seen: Vec<Declared> = Vec::new();
    for d in built {
        if !seen.iter().any(|s| s.name.eq_ignore_ascii_case(&d.name)) {
            seen.push(d);
        }
    }
    seen
}

/// `count <n> / <m>` inside a window, if present.
fn count_pair(window: &str) -> Option<(i64, i64)> {
    let idx = window.find("count")?;
    let after = &window[idx + "count".len()..];
    let digits: String = after
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return None;
    }
    let count: i64 = digits.parse().ok()?;
    let rest = after.trim_start();
    let slash = rest.find('/')?;
    // Only whitespace may sit between the count and the slash.
    if !rest[..slash].trim().chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let of: String = rest[slash + 1..]
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    Some((count, of.parse().ok()?))
}

/// `briefing.lines_agree_with_prompts` — every line declared in a prompt is in the registry, counted.
///
/// **Two failures, and they are different problems:** a line **absent** from the registry is one the
/// harness cannot see at all, and a line **counted differently** means the lab and the harness are
/// working different numbers. Both report the fix as *reconcile the registry with the live prompt*,
/// because **the prompt is what the lab reads** — a registry that disagrees with it is the one that is
/// wrong, whatever it says.
pub fn lines_agree_with_prompts(paths: &StatePaths, regs: &Registries) -> Vec<Finding> {
    let goal = paths.repo.join(SESSION_GOAL);
    let Ok(text) = std::fs::read_to_string(&goal) else {
        return Vec::new();
    };
    let declared = declared_lines(&text);
    if declared.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for d in declared {
        let hit = regs.milestones.iter().find_map(|m| {
            m.lines
                .iter()
                .find(|l| l.name.trim().eq_ignore_ascii_case(&d.name))
                // `Line.count` is a `serde_json::Value` in the Rust registry, so it is read as a
                // number here rather than assumed to be one. A line whose count is not a number is
                // skipped by the caller rather than coerced - **a count that is not a count cannot be
                // compared to one.**
                .and_then(|l| l.count.as_i64().map(|c| (m.id.clone(), c)))
        });
        let Some((mid, registry_count)) = hit else {
            out.push(warn(
                "briefing.lines_agree_with_prompts",
                format!(
                    "line {:?} is declared in {SESSION_GOAL} ({}) but absent from the milestone \
                     registry",
                    d.name, d.where_
                ),
                REGISTRY,
                format!(
                    "add {{\"name\": {:?}, \"count\": {}, \"of\": {}}} to the owning milestone in \
                     {REGISTRY}",
                    d.name, d.count, d.of
                ),
            ));
            continue;
        };
        if registry_count != d.count {
            out.push(warn(
                "briefing.lines_agree_with_prompts",
                format!(
                    "{mid} line {:?}: registry says {registry_count}/{}, {SESSION_GOAL} says {}/{}",
                    d.name, d.of, d.count, d.of
                ),
                REGISTRY,
                "reconcile the registry with the live prompt (the prompt is what the lab reads)"
                    .into(),
            ));
        }
    }
    out
}

/// Every `briefing` rule.
pub fn all(paths: &StatePaths, regs: &Registries) -> Vec<Finding> {
    let mut out = sources_exist(paths, regs);
    out.extend(milestone_numbers_agree(paths, regs));
    out.extend(lines_agree_with_prompts(paths, regs));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A declaration needs a quoted name.** The contract's own format example — `Line: <name> ·
    /// count n/3 · headline …` — carries a count and must NOT be read as a declaration, or the rule
    /// reports a mismatch against a line that does not exist.
    #[test]
    fn the_pr_line_format_is_not_mistaken_for_a_declaration() {
        let format_example = "each PR declares its line, count and headline: \
                              Line: <name> · count n/3 · headline 37/232";
        assert!(
            declared_lines(format_example).is_empty(),
            "the format example is not a declaration: {:?}",
            declared_lines(format_example)
        );
    }

    #[test]
    fn a_quoted_declaration_is_read() {
        let text =
            "Line \"exact addressed memory\" - count 0/3 (D21/D22 count is owned by the milestone)";
        let d = declared_lines(text);
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].name, "exact addressed memory");
        assert_eq!((d[0].count, d[0].of), (0, 3));
        assert_eq!(d[0].where_, "session-goal contract");
    }

    /// Curly quotes are what a prompt pasted from a document actually contains.
    #[test]
    fn curly_quoted_declarations_are_read() {
        let text = "Line \u{201c}m1-deterministic-reply-grader\u{201d} count 0/3";
        let d = declared_lines(text);
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].name, "m1-deterministic-reply-grader");
    }

    /// **A line named twice is one line.** The contract lists the lines and each lab prompt repeats
    /// them; counting both would report a line against itself.
    #[test]
    fn a_line_named_twice_is_declared_once() {
        let text = "Line \"same line\" count 1/3\n...\nLine \"SAME LINE\" count 1/3";
        assert_eq!(
            declared_lines(text).len(),
            1,
            "de-duplicated by normalised name"
        );
    }

    /// A quoted line with no count nearby is not a declaration: there is nothing to compare.
    #[test]
    fn a_quoted_line_with_no_count_is_not_a_declaration() {
        assert!(declared_lines("Line \"a line\" with no count at all").is_empty());
    }

    #[test]
    fn many_declarations_are_all_read() {
        let text = "Line \"one\" count 0/3\nLine \"two\" count 2/3\nLine \"three\" count 1/3";
        assert_eq!(declared_lines(text).len(), 3);
    }

    /// A count that is not immediately followed by a slash is not a count pair.
    #[test]
    fn a_dangling_count_is_ignored() {
        assert!(declared_lines("Line \"a\" count 5 and then prose").is_empty());
    }

    /// The real session-goal must parse, and every declared line must match the registry — if it did
    /// not, Python would have said so.
    #[test]
    fn the_real_repo_agrees_with_its_own_prompts() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let regs = Registries::load(&p).expect("registry");
        let declared =
            declared_lines(&std::fs::read_to_string(p.repo.join(SESSION_GOAL)).unwrap_or_default());
        assert!(!declared.is_empty(), "the real session-goal declares lines");
        let f = all(&p, &regs);
        assert!(f.is_empty(), "the real repo fails a briefing rule: {f:?}");
    }

    /// A milestone naming a mechanism with no file is reported, naming the mechanism.
    #[test]
    fn a_missing_mechanism_brief_is_reported() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let mut regs = Registries::load(&p).expect("registry");
        if let Some(m) = regs.milestones.first_mut() {
            m.mechanisms.push("definitely-not-a-mechanism".into());
        }
        let f = sources_exist(&p, &regs);
        assert!(
            f.iter()
                .any(|x| x.message.contains("definitely-not-a-mechanism")),
            "{f:?}"
        );
    }
}
