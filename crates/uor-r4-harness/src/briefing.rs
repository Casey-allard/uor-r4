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

// ------------------------------------------------------------------- prose extraction
//
// The rest of the brief is cut out of two prose documents. These are the readers. They are written to
// FAIL LOUDLY by returning empty rather than by guessing: a contract that could not be found is
// reported as missing, not rendered as a plausible-looking blank.

/// The body of `## <heading>` (or `###`/`####`), or empty.
///
/// **A heading inside a blockquote, a fenced code block or an HTML comment is not a boundary.**
/// `session-goal.md` carries `## Owner direction` lines inside quoted context, and treating those as
/// boundaries truncates the contract mid-sentence — which would ship a brief that looks complete and
/// is missing its binding clauses.
pub fn first_markdown_section(text: &str, heading: &str) -> String {
    let want = heading.trim().to_ascii_lowercase();
    let lines: Vec<&str> = text.lines().collect();
    let mut start: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        let hashes = t.len() - t.trim_start_matches('#').len();
        if (2..=4).contains(&hashes) {
            let h = t[hashes..].trim();
            if h.eq_ignore_ascii_case(&want) {
                start = Some(i + 1);
                break;
            }
        }
    }
    let Some(start) = start else {
        return String::new();
    };
    let mut out: Vec<&str> = Vec::new();
    let mut in_fence = false;
    let mut in_comment = false;
    for line in &lines[start..] {
        let stripped = line.trim();
        if stripped.starts_with("<!--") {
            in_comment = true;
        }
        if in_comment {
            out.push(line);
            if stripped.contains("-->") {
                in_comment = false;
            }
            continue;
        }
        if stripped.starts_with("```") {
            in_fence = !in_fence;
            out.push(line);
            continue;
        }
        if line.starts_with('>') {
            out.push(line);
            continue;
        }
        let is_heading = {
            let h = line.len() - line.trim_start_matches('#').len();
            (2..=4).contains(&h) && line.trim_start_matches('#').starts_with(' ')
        };
        let indented_code = line.len() - line.trim_start_matches(' ').len() >= 4;
        if !in_fence && is_heading && !indented_code {
            break;
        }
        out.push(line);
    }
    out.join("\n").trim().to_string()
}

/// Split `text` on a marker, keeping the marker at the start of each piece.
///
/// A marker at position 0 does not split, or every list would begin with an empty piece.
fn split_keep(text: &str, marker: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut last = 0usize;
    let mut search = 0usize;
    while let Some(rel) = text[search..].find(marker) {
        let at = search + rel;
        if at != 0 {
            parts.push(text[last..at].to_string());
            last = at;
        }
        search = at + marker.len();
    }
    parts.push(text[last..].to_string());
    parts.into_iter().filter(|p| !p.trim().is_empty()).collect()
}

/// Clauses 1-10 of the session contract, reflowed one per line.
///
/// The source is one flowing paragraph per clause region with nested bullets inline. Reflowing is not
/// cosmetic: unflowed, the contract renders as a wall that a lab skips, and the contract is the thing
/// that binds the session.
pub fn contract_list(session_goal: &str) -> String {
    let mut body = first_markdown_section(session_goal, "The contract");
    if body.is_empty() {
        return String::new();
    }
    // The contract ends at the horizontal rule before the lab prompts.
    if let Some(at) = body.lines().position(|l| {
        let t = l.trim();
        (t.len() >= 3) && (t.chars().all(|c| c == '-') || t.chars().all(|c| c == '*'))
    }) {
        let keep: Vec<&str> = body.lines().take(at).collect();
        body = keep.join("\n");
    }
    let mut out: Vec<String> = Vec::new();
    // Paragraphs, split on a blank line.
    for para in body.split("\n\n") {
        let trimmed = para.trim();
        if trimmed.starts_with('|') || trimmed.starts_with('#') {
            continue;
        }
        let joined = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
        // Split so each clause STARTS with its own full stop. Splitting on `". **"` and keeping the
        // marker (as Python does) leaves the stop on the front of each piece, so `1. **` needs no
        // repair; adding one produced "1 . . **Your number.**". The separator is the leading space of
        // `" . **"` - the same boundary, consumed instead of kept.
        for chunk in split_keep(&joined, " . **") {
            let chunk = chunk.trim();
            if chunk.is_empty() {
                continue;
            }
            let bullet_parts = split_keep(chunk, "- **");
            let first = bullet_parts.first().cloned().unwrap_or_default();
            let first = first.trim();
            if !first.is_empty() {
                out.push(first.to_string());
            }
            for b in bullet_parts.iter().skip(1) {
                out.push(format!("   - {}", b.trim().trim_start_matches('-').trim()));
            }
        }
    }
    out.join("\n").trim().to_string()
}

/// The fenced ```text prompt for one lab.
pub fn lab_prompt(session_goal: &str, lab: &str) -> String {
    if lab.is_empty() {
        return String::new();
    }
    let lowered = session_goal.to_ascii_lowercase();
    let mut idx: Option<usize> = None;
    for marker in [
        format!("prompt for the {} lab", lab.to_ascii_lowercase()),
        format!(
            "prompt for the {} lab",
            capitalize(lab).to_ascii_lowercase()
        ),
    ] {
        if let Some(at) = lowered.find(&marker) {
            idx = Some(at);
            break;
        }
    }
    let Some(idx) = idx else { return String::new() };
    // Fences at or after the marker; the prompt is between the first pair.
    let fences: Vec<usize> = session_goal
        .match_indices("```")
        .map(|(i, _)| i)
        .filter(|i| *i >= idx)
        .collect();
    if fences.len() < 2 {
        return String::new();
    }
    // The prompt body starts just after the opening fence's ``` - INCLUDING its language tag. The
    // rendered brief begins "text Apply the common start prompt...", which reads oddly and is what
    // Python produces; stripping the tag would be a silent difference between two tools that are
    // supposed to agree about the same document.
    let after_open = fences[0] + 3;
    if after_open > fences[1] {
        return String::new();
    }
    session_goal[after_open..fences[1]].trim().to_string()
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Section size limits, matching the Python assembler. They bound how much of a prose document the
/// brief quotes, so the pack stays readable in one sitting.
pub const LIMIT_SESSION_GOAL: usize = 5200;
pub const LIMIT_DECISION: usize = 2600;
pub const LIMIT_PROMPT: usize = 2600;
pub const LIMIT_MECHANISM: usize = 2000;
pub const DEFAULT_BUDGET: usize = 14000;

fn read_doc(path: &std::path::Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// The sections this port provides, in the order Python assembles them.
///
/// Sections 7 and 8 (the lessons and the skills) are NOT here: they need the mistakes and skills
/// registries, which are separate Python modules and separate units of the port. The caller reports
/// the remainder rather than letting the brief look complete.
pub fn assemble_full(
    regs: &Registries,
    paths: &crate::state::StatePaths,
    milestone: Option<&str>,
    query: &str,
) -> ContextPack {
    let ms = milestone.and_then(|m| regs.milestone(m));
    let mut pack = assemble(regs, milestone, query);

    let session_goal_path = paths.repo.join("docs").join("labs").join("session-goal.md");
    let session_goal = read_doc(&session_goal_path);

    // 3. the contract
    let contract = contract_list(&session_goal);
    if !contract.is_empty() {
        pack.add(
            "Session contract (docs/labs/session-goal.md, clauses 1-10)",
            &truncate(&contract, LIMIT_SESSION_GOAL),
            "docs/labs/session-goal.md",
            "binding on every session",
        );
    }
    if let Some(m) = ms {
        let prompt = lab_prompt(&session_goal, &m.lab);
        if !prompt.is_empty() {
            pack.add(
                format!("The {} lab prompt (the FINISH LINE clause)", m.lab),
                &truncate(&prompt, LIMIT_PROMPT),
                "docs/labs/session-goal.md",
                &format!("this milestone's lab is {}", m.lab),
            );
        }
    }

    // 4. governance
    for d in regs.decisions_for(ms) {
        pack.add(
            format!("Decision {} - {}", d.id, d.title),
            &truncate(&d.body, LIMIT_DECISION),
            &format!("docs/integration/DECISIONS.md#L{}", d.line),
            "governs this milestone",
        );
    }

    // 5. the mechanisms
    if let Some(m) = ms {
        for mech in &m.mechanisms {
            let brief = paths
                .repo
                .join("docs")
                .join("mechanisms")
                .join(format!("{mech}.md"));
            let text = read_doc(&brief);
            if text.is_empty() {
                pack.dropped.push((
                    format!("docs/mechanisms/{mech}.md"),
                    "brief not found".to_string(),
                ));
                continue;
            }
            let mut body = truncate(&text, LIMIT_MECHANISM);
            let fair = first_markdown_section(&text, "A fair test needs");
            if !fair.is_empty() {
                body.push_str(&format!(
                    "\n\n### A fair test needs (verbatim)\n\n{}",
                    truncate(&fair, 900)
                ));
            }
            pack.add(
                format!("Mechanism brief: {mech}"),
                &body,
                &format!("docs/mechanisms/{mech}.md"),
                "named by the milestone",
            );
        }
    }

    // 6. the panel
    if let Some(m) = ms {
        match regs.panel_for(m) {
            Some(panel) => {
                let mut rows = vec![
                    format!(
                        "- Panel `{}` - {} (frozen: {})",
                        panel.id,
                        panel.title,
                        if panel.frozen { "True" } else { "False" }
                    ),
                    format!("- Grader: `{}`", panel.grader),
                ];
                if !panel.noise.is_empty() {
                    rows.push(format!("- Panel noise: {}", panel.noise));
                }
                if !panel.seeds.is_empty() {
                    rows.push(format!("- Seeds: {}", panel.seeds));
                }
                if !panel.rows.is_empty() {
                    rows.push(format!("- Rows: {}", panel.rows));
                }
                for bar in &panel.bars {
                    let of = if bar.of.is_null() || bar.of.as_str() == Some("") {
                        String::new()
                    } else {
                        format!("/{}", crate::registries::fmt_num(&bar.of))
                    };
                    rows.push(format!(
                        "- Bar `{}`: {} {}{of} -> target {}{of}",
                        bar.id,
                        bar.label,
                        crate::registries::fmt_num(&bar.current),
                        crate::registries::fmt_num(&bar.target)
                    ));
                }
                pack.add(
                    "Frozen acceptance panel",
                    &rows.join("\n"),
                    &format!("docs/harness/panels/{}.json", panel.id),
                    "scores this milestone",
                );
            }
            None => pack.dropped.push((
                format!("panel {}", m.panel),
                "no panel file found under docs/harness/panels/".to_string(),
            )),
        }
    }

    // 9. what is being worked on right now
    let now = crate::receipt::now_rfc3339();
    let recent =
        crate::status::receipt_rows(paths, &now[..7], &crate::status::previous_month(&now[..7]));
    let cutoff = crate::state::parse_iso_unix(&now).unwrap_or(0) - 24 * 3600;
    let in_24h: Vec<&Value> = recent
        .iter()
        .filter(|r| {
            r.get("ts")
                .and_then(Value::as_str)
                .and_then(crate::state::parse_iso_unix)
                .is_some_and(|t| t >= cutoff)
        })
        .collect();
    if !in_24h.is_empty() {
        let rows: Vec<String> = in_24h
            .iter()
            .rev()
            .take(6)
            .rev()
            .map(|r| {
                let fields: Vec<String> = r
                    .as_object()
                    .map(|o| {
                        o.iter()
                            .filter(|(k, _)| {
                                !["ts", "kind", "schema", "event", "_file"].contains(&k.as_str())
                            })
                            .map(|(k, v)| {
                                // Rendered the way Python's f-string renders it: `True`, not `true`.
                                // A brief that says `posted=false` where the other tool says
                                // `posted=False` is two tools disagreeing about the same receipt.
                                let v = match v {
                                    Value::String(s) => s.clone(),
                                    Value::Bool(b) => {
                                        if *b {
                                            "True".into()
                                        } else {
                                            "False".into()
                                        }
                                    }
                                    other => other.to_string(),
                                };
                                format!("{k}={v}")
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                format!(
                    "- {} `{}` {}",
                    r.get("ts").and_then(Value::as_str).unwrap_or("?"),
                    r.get("kind").and_then(Value::as_str).unwrap_or(""),
                    fields.join(" ")
                )
            })
            .collect();
        pack.add(
            "Receipts in the last 24h (what actually ran)",
            &rows.join("\n"),
            "docs/harness/ledger/",
            "orchestration visibility",
        );
    }

    // 10. the task itself, and the loop
    if !query.is_empty() {
        pack.add(
            "The task as stated",
            query,
            "cli --query",
            "supplied by the caller",
        );
    }
    pack.add(
        "How to use this brief",
        "1. Post a status card on the milestone issue before changing anything.\n\
         2. Load the skills listed above with `skill use` so the work is visible.\n\
         3. Pre-register any compute with `prep new`, and get `TEST FITNESS: FIT` before running it.\n\
         4. When done, write the dated record and the current-state entry, then `lint check` the PR body.",
        "docs/harness/README.md",
        "the loop",
    );

    if pack.tokens() > DEFAULT_BUDGET {
        pack.dropped.push((
            "budget".to_string(),
            format!(
                "pack is ~{} tokens against a {DEFAULT_BUDGET} budget; drop the lowest-value section \
                 (mechanism detail first) or raise --budget-tokens",
                pack.tokens()
            ),
        ));
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
