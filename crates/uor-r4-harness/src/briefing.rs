//! `brief` — the context pack for a task.
//!
//! Ported in slices. This is **section 1 (the task's state)** plus the pack renderer, which together
//! produce everything before `## Owner rules`. The rest of the assembler — the session contract, the
//! governance decisions, the mechanism briefs and the receipt tail — reads and splits prose out of
//! `session-goal.md` and `DECISIONS.md`, and is a larger unit that follows.
//!
//! Scope is honest rather than convenient: `render` emits a header, the sections it was given, the
//! provenance list and the token total, and **a caller can see which sections are present**. A brief
//! that silently omitted the owner rules would read as a brief with no owner rules.
//!
//! The token count is `max(1, chars / 4)` — a rough cost estimate, integer division, and it must
//! match exactly or every brief in every log disagrees with the tool that produced it.

use serde_json::Value;

use crate::registries::{fmt_num, Milestone, Registries};

/// Rough token count: characters over four, floored, minimum one.
///
/// `chars` is the CHARACTER count, not the byte count — the same text in UTF-8 and UTF-16 must cost
/// the same, and a brief is full of em-dashes and arrows.
pub fn human_tokens(text: &str) -> usize {
    (text.chars().count() / 4).max(1)
}

/// One loaded source, with why it was selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub path: String,
    pub reason: String,
    pub tokens: usize,
}

/// One `## heading` block plus its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub heading: String,
    pub body: String,
}

/// An assembled brief: ordered blocks plus their provenance.
#[derive(Debug, Clone, Default)]
pub struct ContextPack {
    pub title: String,
    pub blocks: Vec<Block>,
    pub sources: Vec<Source>,
    /// `(path, reason)` for what was selected but not loaded.
    pub dropped: Vec<(String, String)>,
}

impl ContextPack {
    pub fn new(title: impl Into<String>) -> Self {
        ContextPack {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Add a block. Empty content is DROPPED with a reason rather than added as a blank section: a
    /// section that renders as a heading with nothing under it reads as "there is nothing here",
    /// which is a different claim from "this could not be read".
    pub fn add(&mut self, heading: impl Into<String>, body: &str, path: &str, reason: &str) {
        let body = body.trim();
        if body.is_empty() {
            self.dropped
                .push((path.to_string(), format!("empty content ({reason})")));
            return;
        }
        self.blocks.push(Block {
            heading: heading.into(),
            body: body.to_string(),
        });
        self.sources.push(Source {
            path: path.to_string(),
            reason: reason.to_string(),
            tokens: human_tokens(body),
        });
    }

    /// The whole brief's cost: its blocks plus its title.
    pub fn tokens(&self) -> usize {
        self.sources.iter().map(|s| s.tokens).sum::<usize>() + human_tokens(&self.title)
    }

    /// Render as markdown, exactly the shape the Python pack produces.
    pub fn render(&self) -> String {
        let mut parts: Vec<String> = vec![format!("# {}", self.title), String::new()];
        for b in &self.blocks {
            parts.push(format!("## {}", b.heading));
            parts.push(b.body.clone());
            parts.push(String::new());
        }
        parts.push("---".to_string());
        parts.push(
            "*Context selected by `scripts/harness/cli.py`. Sources, in load order:*".to_string(),
        );
        for s in &self.sources {
            let extra = if s.reason.is_empty() {
                String::new()
            } else {
                format!(" ({})", s.reason)
            };
            parts.push(format!("- `{}`{extra} - ~{} tokens", s.path, s.tokens));
        }
        if !self.dropped.is_empty() {
            parts.push(String::new());
            parts.push("*Not loaded, and why:*".to_string());
            for (path, reason) in &self.dropped {
                parts.push(format!("- `{path}` - {reason}"));
            }
        }
        parts.push(String::new());
        parts.push(format!("*Total ~{} tokens.*", self.tokens()));
        parts.join("\n")
    }
}

/// The brief's title: the milestone when one is named, else the query, else the default.
pub fn title_for(ms: Option<&Milestone>, query: &str) -> String {
    let mut title = "UOR-R4 task brief".to_string();
    if let Some(m) = ms {
        title.push_str(&format!(" - {} #{} ({} lab)", m.id, m.issue, m.lab));
    } else if !query.is_empty() {
        title.push_str(&format!(" - {}", truncate(query, 60)));
    }
    title
}

/// Truncate on character boundaries, marking the cut.
pub fn truncate(text: &str, limit: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= limit {
        return collapsed;
    }
    let kept: String = collapsed.chars().take(limit.saturating_sub(1)).collect();
    format!("{}…", kept.trim_end())
}

/// Section 1: where the task stands.
///
/// With a milestone, its whole contract — headline, secondary lines, D21 line counts, panel, base
/// artifact and the next action on record. Without one, every milestone as a table, because "which
/// task is this" is the question a brief with no milestone is being asked.
pub fn state_block(
    regs: &Registries,
    ms: Option<&Milestone>,
) -> Option<(String, String, String, String)> {
    match ms {
        Some(m) => {
            let mut head: Vec<String> = vec![
                format!("- Milestone: **{} #{}** - {}", m.id, m.issue, m.title),
                format!("- Lab: `{}` - status: `{}`", m.lab, m.status),
                format!("- Headline: **{}**", m.headline_line()),
            ];
            for sec in &m.secondary {
                let of = sec
                    .get("of")
                    .filter(|v| !v.is_null() && v.as_str() != Some(""))
                    .map(|v| format!("/{}", fmt_num(v)))
                    .unwrap_or_default();
                head.push(format!(
                    "- Secondary: {} {}{of} -> target {}{of}",
                    sec.get("label").and_then(Value::as_str).unwrap_or(""),
                    fmt_num(sec.get("current").unwrap_or(&Value::Null)),
                    fmt_num(sec.get("target").unwrap_or(&Value::Null))
                ));
            }
            head.push(format!("- Lines (D21 count): {}", m.line_counts()));
            head.push(format!("- Acceptance panel: `{}`", m.panel));
            head.push(format!("- Base artifact: {}", m.base_artifact));
            head.push(format!("- Next action on record: {}", m.next));
            Some((
                format!("{} state (from docs/harness/milestones.json)", m.id),
                head.join("\n"),
                "docs/harness/milestones.json".to_string(),
                "milestone contract".to_string(),
            ))
        }
        None => {
            let mut lines: Vec<String> = vec![
                "| Milestone | Lab | Headline | Lines |".to_string(),
                "|---|---|---|---|".to_string(),
            ];
            for m in &regs.milestones {
                lines.push(format!(
                    "| {} #{} | {} | {} | {} |",
                    m.id,
                    m.issue,
                    m.lab,
                    m.headline_line(),
                    m.line_counts()
                ));
            }
            Some((
                "The four milestones".to_string(),
                lines.join("\n"),
                "docs/harness/milestones.json".to_string(),
                "no milestone given: show all".to_string(),
            ))
        }
    }
}

/// Section 2: the owner rules in force.
pub fn rules_block(regs: &Registries) -> Option<(String, String, String, String)> {
    if regs.owner_rules.is_empty() {
        return None;
    }
    let rules: Vec<String> = regs
        .owner_rules
        .iter()
        .map(|r| {
            format!(
                "- **{}** - {}",
                r.get("id").and_then(Value::as_str).unwrap_or(""),
                r.get("rule").and_then(Value::as_str).unwrap_or("")
            )
        })
        .collect();
    Some((
        "Owner rules in force".to_string(),
        rules.join("\n"),
        "docs/harness/milestones.json".to_string(),
        "always in force".to_string(),
    ))
}

/// Assemble the sections this port provides.
///
/// The omitted sections are listed by the CALLER, not silently absent: `briefing.rs` knows which
/// parts of the Python assembler it has replaced, and `brief` prints the remainder as unported.
pub fn assemble(regs: &Registries, milestone: Option<&str>, query: &str) -> ContextPack {
    let ms = milestone.and_then(|m| regs.milestone(m));
    let mut pack = ContextPack::new(title_for(ms, query));
    for (heading, body, path, reason) in [state_block(regs, ms), rules_block(regs)]
        .into_iter()
        .flatten()
    {
        pack.add(heading, &body, &path, &reason);
    }
    pack
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn regs() -> Registries {
        let paths = crate::state::StatePaths::resolve(repo(), None, Some("/nonexistent-live"));
        Registries::load(&paths).expect("registry")
    }

    #[test]
    fn tokens_are_chars_over_four_floored_and_never_zero() {
        assert_eq!(human_tokens(""), 1, "an empty body still costs one");
        assert_eq!(human_tokens("abc"), 1);
        assert_eq!(human_tokens("abcdefgh"), 2);
        assert_eq!(
            human_tokens("abcdefghi"),
            2,
            "integer division, not rounding"
        );
        // Characters, not bytes: an em-dash is one character and three bytes.
        assert_eq!(human_tokens("————"), 1);
    }

    /// An empty body is DROPPED with a reason, not rendered as a heading with nothing under it.
    #[test]
    fn an_empty_block_is_dropped_with_a_reason() {
        let mut pack = ContextPack::new("t");
        pack.add("Heading", "   ", "a/b.md", "because");
        assert!(pack.blocks.is_empty());
        assert_eq!(
            pack.dropped,
            vec![("a/b.md".to_string(), "empty content (because)".to_string())]
        );
        let text = pack.render();
        assert!(!text.contains("## Heading"), "{text}");
        assert!(text.contains("Not loaded, and why:"), "{text}");
        assert!(
            text.contains("`a/b.md` - empty content (because)"),
            "{text}"
        );
    }

    #[test]
    fn the_render_has_the_sections_provenance_and_total() {
        let mut pack = ContextPack::new("UOR-R4 task brief");
        pack.add("One", "body text", "p.md", "why");
        let text = pack.render();
        assert!(text.starts_with("# UOR-R4 task brief\n"), "{text}");
        assert!(text.contains("## One\nbody text"), "{text}");
        assert!(text.contains("- `p.md` (why) - ~2 tokens"), "{text}");
        assert!(text.contains("---"), "{text}");
        assert!(text.contains("*Total ~"), "{text}");
    }

    /// The title has three forms and each is live: a milestone, a query, or neither.
    #[test]
    fn the_title_names_the_milestone_query_or_neither() {
        let r = regs();
        let m1 = r.milestone("m1").expect("m1");
        assert_eq!(
            title_for(Some(m1), ""),
            format!("UOR-R4 task brief - M1 #2029 ({} lab)", m1.lab)
        );
        assert!(title_for(None, "rank margin bars").contains("rank margin bars"));
        assert_eq!(title_for(None, ""), "UOR-R4 task brief");
    }

    /// Section 1 renders the milestone contract, and every field a lab acts on is present.
    #[test]
    fn the_milestone_block_carries_the_whole_contract() {
        let r = regs();
        let m1 = r.milestone("m1").expect("m1");
        let (heading, body, path, reason) = state_block(&r, Some(m1)).expect("a block");
        assert_eq!(heading, "M1 state (from docs/harness/milestones.json)");
        assert_eq!(path, "docs/harness/milestones.json");
        assert_eq!(reason, "milestone contract");
        for needle in [
            "Headline:",
            "Lines (D21 count):",
            "Acceptance panel:",
            "Base artifact:",
            "Next action on record:",
            "status:",
        ] {
            assert!(body.contains(needle), "{needle} missing from:\n{body}");
        }
        // The headline is the number the run is scored against, so it is rendered bold and verbatim.
        assert!(
            body.contains(&format!("**{}**", m1.headline_line())),
            "{body}"
        );
    }

    /// Without a milestone the block is a table of all of them, because "which task is this" is the
    /// question being asked.
    #[test]
    fn the_no_milestone_block_tables_every_milestone() {
        let r = regs();
        let (heading, body, _, reason) = state_block(&r, None).expect("a block");
        assert_eq!(heading, "The four milestones");
        assert!(reason.contains("no milestone given"), "{reason}");
        assert!(
            body.starts_with("| Milestone | Lab | Headline | Lines |"),
            "{body}"
        );
        for m in &r.milestones {
            assert!(
                body.contains(&format!("| {} #{} |", m.id, m.issue)),
                "{} missing",
                m.id
            );
        }
    }

    #[test]
    fn the_rules_block_lists_every_owner_rule() {
        let r = regs();
        let (heading, body, _, reason) = rules_block(&r).expect("the registry carries rules");
        assert_eq!(heading, "Owner rules in force");
        assert_eq!(reason, "always in force");
        for rule in &r.owner_rules {
            let id = rule.get("id").and_then(Value::as_str).unwrap_or("");
            assert!(
                body.contains(&format!("**{id}**")),
                "{id} missing from:\n{body}"
            );
        }
    }

    /// The assembled pack renders, and the sections it does NOT provide are absent rather than blank.
    #[test]
    fn assemble_produces_a_renderable_pack() {
        let r = regs();
        let pack = assemble(&r, Some("m1"), "");
        assert!(pack.title.contains("M1 #2029"), "{}", pack.title);
        let headings: Vec<&str> = pack.blocks.iter().map(|b| b.heading.as_str()).collect();
        assert!(
            headings.iter().any(|h| h.starts_with("M1 state")),
            "{headings:?}"
        );
        assert!(headings.contains(&"Owner rules in force"), "{headings:?}");
        let text = pack.render();
        assert!(text.starts_with("# UOR-R4 task brief"), "{text}");
        assert!(text.contains("## Owner rules in force"), "{text}");
        assert!(
            !text.contains("## Session contract"),
            "not ported yet, so not claimed: {text}"
        );
    }

    #[test]
    fn truncate_marks_a_cut_on_character_boundaries() {
        assert_eq!(truncate("short", 20), "short");
        assert_eq!(truncate("a  b\n c", 20), "a b c", "whitespace collapses");
        let cut = truncate("abcdef", 4);
        assert!(cut.ends_with('…'), "{cut}");
        assert_eq!(cut.chars().count(), 4, "the marker is inside the budget");
    }
}
