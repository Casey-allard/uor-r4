//! `status` — what is true now, as one numbered view a human reads in twenty seconds.
//!
//! The full Python module is 1,863 lines and half of it is GitHub reconciliation: PRs, merge queue,
//! delivery. This ports the two sections every lab actually opens `status` for — **1. MILESTONES**
//! and **2. RECEIPTS / VISIBILITY** — and reports the rest as not-yet-ported rather than rendering a
//! plausible-looking blank. A status view that silently omits its delivery section is worse than one
//! that says so, because the first reads as "nothing to report".
//!
//! The rule this module inherits from Python: **status must never raise.** It is what a lab runs when
//! something is already wrong, so a missing registry or an unreadable ledger is reported in place,
//! not thrown.

use serde_json::Value;

use crate::registries::{fmt_num, Milestone, Registries};
use crate::state::StatePaths;

/// How much of a free-text field the view keeps. Python truncates at the same widths, and the widths
/// matter: they are what makes the view readable in one screen.
const NEXT_WIDTH: usize = 220;
const RULE_WIDTH: usize = 150;
/// Width for a receipt line in the view. Used once section 2 is rendered.
pub const RECEIPT_WIDTH: usize = 150;

/// Collapse whitespace, then truncate to `width` characters with an ellipsis.
///
/// Whitespace collapse is not cosmetic: these fields come from JSON that carries newlines and runs of
/// spaces, and the view is a fixed-width list. Python's `truncate` also strips the space a cut may
/// leave, so a truncated field does not end "word …".
pub fn truncate(text: &str, width: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= width {
        return collapsed;
    }
    let kept: String = collapsed.chars().take(width.saturating_sub(1)).collect();
    format!("{}…", kept.trim_end())
}

/// A milestone as the status view presents it.
#[derive(Debug, Clone)]
pub struct MilestoneRow {
    pub key: String,
    pub id: String,
    pub issue: i64,
    pub lab: String,
    pub status: String,
    pub headline: String,
    pub line_counts: String,
    pub panel: String,
    /// Mechanism briefs, and whether each one exists on disk.
    pub mechanisms: Vec<(String, bool)>,
    pub next: String,
    pub url: String,
}

/// A mechanism id and whether its brief is present.
pub fn mechanism_briefs(paths: &StatePaths, m: &Milestone) -> Vec<(String, bool)> {
    m.mechanisms
        .iter()
        .map(|id| {
            let exists = paths
                .repo
                .join("docs")
                .join("mechanisms")
                .join(format!("{id}.md"))
                .is_file();
            (id.clone(), exists)
        })
        .collect()
}

/// The repo slug for issue URLs, read from `origin`. Empty when there is no remote, in which case the
/// view falls back to a bare `issue #N` rather than inventing a link.
pub fn repo_slug(paths: &StatePaths) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(&paths.repo)
        .args(["remote", "get-url", "origin"])
        .output();
    let Ok(out) = out else { return String::new() };
    if !out.status.success() {
        return String::new();
    }
    let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // github.com:owner/repo or github.com/owner/repo, with an optional .git
    let Some(idx) = url.rfind("github.com") else {
        return String::new();
    };
    let rest = url[idx + "github.com".len()..].trim_start_matches([':', '/']);
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    let parts: Vec<&str> = rest.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() >= 2 {
        format!("{}/{}", parts[0], parts[1])
    } else {
        String::new()
    }
}

/// Build the milestone rows, in issue order.
pub fn milestone_rows(paths: &StatePaths, regs: &Registries) -> Vec<MilestoneRow> {
    let slug = repo_slug(paths);
    regs.milestones
        .iter()
        .map(|m| MilestoneRow {
            key: m.key.clone(),
            id: m.id.clone(),
            issue: m.issue,
            lab: m.lab.clone(),
            status: m.status.clone(),
            headline: m.headline_line(),
            line_counts: m.line_counts(),
            panel: m.panel.clone(),
            mechanisms: mechanism_briefs(paths, m),
            next: m.next.clone(),
            url: if slug.is_empty() {
                format!("issue #{}", m.issue)
            } else {
                format!("https://github.com/{slug}/issues/{}", m.issue)
            },
        })
        .collect()
}

/// Count receipts by kind, for a ledger shard.
pub fn kind_counts(rows: &[Value]) -> Vec<(String, usize)> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for r in rows {
        let kind = r
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        match counts.iter_mut().find(|(k, _)| *k == kind) {
            Some((_, n)) => *n += 1,
            None => counts.push((kind, 1)),
        }
    }
    counts
}

/// `kind count, kind count`, or `no kinds`. Insertion order, not sorted: Python does not sort here,
/// and a status view that reorders a ledger's kinds from one run to the next is harder to compare.
pub fn kind_text(counts: &[(String, usize)]) -> String {
    if counts.is_empty() {
        return "no kinds".to_string();
    }
    counts
        .iter()
        .map(|(k, n)| format!("{k} {n}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// `id xN`, sorted, or `none in this ledger month`.
pub fn id_counts(counts: &[(String, usize)]) -> String {
    if counts.is_empty() {
        return "none in this ledger month".to_string();
    }
    let mut sorted = counts.to_vec();
    sorted.sort();
    sorted
        .iter()
        .map(|(k, n)| format!("{k} x{n}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The short form of a receipt: `key=value` for the fields worth showing.
pub fn receipt_brief(rec: &Value) -> String {
    let mut parts = Vec::new();
    for key in [
        "milestone",
        "skill",
        "role",
        "count",
        "outcome",
        "task",
        "headline",
        "line",
        "note",
    ] {
        let Some(v) = rec.get(key) else { continue };
        let empty = v.is_null()
            || v.as_str().is_some_and(str::is_empty)
            || v.as_array().is_some_and(Vec::is_empty);
        if !empty {
            let rendered = match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            parts.push(format!("{key}={rendered}"));
        }
    }
    if parts.is_empty() {
        "(no fields)".to_string()
    } else {
        parts.join(" ")
    }
}

/// A lab's receipt rows for one month, plus the counts the view shows.
#[derive(Debug, Clone, Default)]
pub struct ReceiptsView {
    pub available: bool,
    pub reason: String,
    pub month_file: String,
    pub exists: bool,
    pub today_date: String,
    pub today_counts: Vec<(String, usize)>,
    pub today_total: usize,
    pub month: String,
    pub month_counts: Vec<(String, usize)>,
    pub month_total: usize,
    pub last5: Vec<Value>,
    pub skills_used: Vec<(String, usize)>,
    pub team_used: Vec<(String, usize)>,
}

/// Read a lab's receipts for a month from its shard.
///
/// The shard is `docs/harness/ledger/<lab>-<YYYY-MM>.jsonl`, one lab per file, which is what stops
/// concurrent labs from conflicting on a shared ledger.
pub fn receipts_view(paths: &StatePaths, lab: &str, now_rfc3339: &str) -> ReceiptsView {
    let month = now_rfc3339[..7].to_string();
    let day = now_rfc3339[..10].to_string();
    let file = paths
        .repo
        .join("docs")
        .join("harness")
        .join("ledger")
        .join(format!("{lab}-{month}.jsonl"));
    let mut view = ReceiptsView {
        available: true,
        month_file: file.display().to_string(),
        month: month.clone(),
        today_date: day.clone(),
        ..Default::default()
    };
    let Ok(text) = std::fs::read_to_string(&file) else {
        // Not an error: a lab that has never recorded a receipt has no shard, and the view says so.
        view.exists = false;
        return view;
    };
    view.exists = true;
    let mut rows: Vec<Value> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    let today_rows: Vec<Value> = rows
        .iter()
        .filter(|r| {
            r.get("ts")
                .and_then(Value::as_str)
                .is_some_and(|t| t.starts_with(&day))
        })
        .cloned()
        .collect();
    view.today_counts = kind_counts(&today_rows);
    view.today_total = today_rows.len();
    view.month_counts = kind_counts(&rows);
    view.month_total = rows.len();
    rows.sort_by_key(|r| {
        r.get("ts")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    });
    view.last5 = rows.iter().rev().take(5).cloned().collect();
    for r in &rows {
        match r.get("kind").and_then(Value::as_str) {
            Some("skill") => bump(
                &mut view.skills_used,
                r.get("skill").and_then(Value::as_str),
            ),
            Some("team") => bump(&mut view.team_used, r.get("role").and_then(Value::as_str)),
            _ => {}
        }
    }
    view
}

fn bump(counts: &mut Vec<(String, usize)>, key: Option<&str>) {
    let Some(k) = key else { return };
    match counts.iter_mut().find(|(n, _)| n == k) {
        Some((_, c)) => *c += 1,
        None => counts.push((k.to_string(), 1)),
    }
}

/// Render the whole view, with a numbered structure so a lab can cite "status section 3".
pub fn render(
    paths: &StatePaths,
    regs: Option<&Registries>,
    lab: &str,
    now_rfc3339: &str,
) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("uor-harness status - {now_rfc3339}"));
    match regs {
        Some(r) => {
            let path = paths
                .repo
                .join("docs")
                .join("harness")
                .join("milestones.json");
            // The registry's own top-level `note` rides on the source line, truncated. It often
            // records which owner direction the cache was reconciled to, which is the first thing a
            // lab needs before trusting anything below it.
            let note = truncate(&r.note, 160);
            lines.push(format!(
                "source: {} ({} milestones){}",
                path.display(),
                r.milestones.len(),
                if note.is_empty() {
                    String::new()
                } else {
                    format!(" · {note}")
                }
            ));
        }
        None => lines.push("source: docs/harness/milestones.json (0 milestones)".to_string()),
    }
    lines.push(String::new());

    // 1. MILESTONES
    lines.push("1. MILESTONES".to_string());
    match regs {
        None => lines.push("   UNAVAILABLE: registry read failed".to_string()),
        Some(r) => {
            if r.milestones.is_empty() {
                lines.push("   UNAVAILABLE: registry read failed".to_string());
            }
            for row in milestone_rows(paths, r) {
                lines.push(format!(
                    "   {} #{} [{}, {}] {}",
                    row.id,
                    row.issue,
                    if row.lab.is_empty() {
                        "unassigned"
                    } else {
                        &row.lab
                    },
                    if row.status.is_empty() {
                        "unknown"
                    } else {
                        &row.status
                    },
                    row.headline
                ));
                let mut bits = Vec::new();
                if !row.line_counts.is_empty() {
                    bits.push(format!("line: {}", row.line_counts));
                }
                if !row.panel.is_empty() {
                    bits.push(format!("panel: {}", row.panel));
                }
                if !bits.is_empty() {
                    lines.push(format!("      {}", bits.join(" · ")));
                }
                if !row.mechanisms.is_empty() {
                    let rendered = row
                        .mechanisms
                        .iter()
                        .map(|(id, exists)| {
                            if *exists {
                                id.clone()
                            } else {
                                format!("{id} (BRIEF MISSING)")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    lines.push(format!("      mechanisms: {rendered}"));
                }
                if !row.next.is_empty() {
                    lines.push(format!("      next: {}", truncate(&row.next, NEXT_WIDTH)));
                }
            }
            if !r.owner_rules.is_empty() {
                lines.push("   owner rules (docs/harness/milestones.json):".to_string());
                for rule in &r.owner_rules {
                    let id = rule.get("id").and_then(Value::as_str).unwrap_or("");
                    let text = rule.get("rule").and_then(Value::as_str).unwrap_or("");
                    lines.push(format!("      {id}: {}", truncate(text, RULE_WIDTH)));
                }
            }
        }
    }
    lines.push(String::new());

    // 2. RECEIPTS / VISIBILITY
    //
    // NOT ported, and it says so. Python's section aggregates EVERY lab's shard and adds three
    // 24-hour subsections (milestone issues updated, milestones with no receipt, heartbeat cards
    // emitted but never posted on the issue). A single-lab version of this would read as a complete
    // answer while quietly omitting the other three labs — the worst outcome for a section whose
    // whole purpose is cross-lab visibility. The single-lab read below is kept as `receipts_view`
    // because it is correct and tested, but the rendered section states the gap instead of
    // presenting one lab's shard as the picture.
    lines.push("2. RECEIPTS / VISIBILITY".to_string());
    lines.push(
        "   not ported to Rust yet. This section aggregates all four labs' shards plus three 24-hour \
         subsections; `scripts/harness/cli.py status` remains authoritative. Rendering one lab's \
         shard here would read as the whole picture, so it is reported rather than shown."
            .to_string(),
    );
    let _ = &lab;

    // 3. DELIVERY - stated, not omitted. A status that silently lacks its delivery section reads as
    // "nothing to report", which is the opposite of the truth when the section is simply not ported.
    lines.push(String::new());
    lines.push("3. DELIVERY".to_string());
    lines.push(
        "   not ported to Rust yet: PRs, merge queue and delivery run in Python \
         (`scripts/harness/cli.py status`). Listed so its absence is visible rather than read as \
         nothing to report."
            .to_string(),
    );

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// The headline acceptance number, formatted for a one-line summary.
pub fn headline_summary(regs: &Registries) -> String {
    match regs.milestones.first() {
        None => "no milestones".to_string(),
        Some(m) => {
            let cur = m
                .headline
                .get("current")
                .map(fmt_num)
                .unwrap_or_else(|| "?".into());
            format!("{} {}", m.id, cur)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn paths() -> StatePaths {
        StatePaths::resolve(repo(), None, Some("/nonexistent-live"))
    }

    #[test]
    fn truncate_marks_a_cut_and_leaves_short_text_alone() {
        assert_eq!(truncate("abc", 10), "abc");
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("", 5), "");
    }

    /// The view opens the two sections every lab reads, and numbers them so a lab can cite one.
    #[test]
    fn the_view_has_the_numbered_sections() {
        let regs = Registries::load(&paths()).expect("registry");
        let text = render(
            &paths(),
            Some(&regs),
            "deepseek",
            "2026-10-13T02:00:00-04:00",
        );
        assert!(text.contains("1. MILESTONES"), "{text}");
        assert!(text.contains("2. RECEIPTS / VISIBILITY"), "{text}");
        assert!(text.contains("3. DELIVERY"), "{text}");
        assert!(
            text.contains("source: "),
            "the view says where its registry came from"
        );
    }

    /// An unported section must SAY it is unported. A silently empty delivery section reads as
    /// "nothing to deliver", which is the opposite of the truth.
    #[test]
    fn the_unported_delivery_section_states_that_it_is_unported() {
        let regs = Registries::load(&paths()).expect("registry");
        let text = render(
            &paths(),
            Some(&regs),
            "deepseek",
            "2026-10-13T02:00:00-04:00",
        );
        let delivery = text.split("3. DELIVERY").nth(1).expect("delivery section");
        assert!(delivery.contains("not ported"), "{delivery}");
    }

    /// Status must never raise: it is what a lab runs when something is already wrong.
    #[test]
    fn a_missing_registry_renders_unavailable_rather_than_panicking() {
        let text = render(&paths(), None, "deepseek", "2026-10-13T02:00:00-04:00");
        assert!(text.contains("UNAVAILABLE"), "{text}");
        assert!(
            text.contains("2. RECEIPTS"),
            "the other sections still render"
        );
    }

    #[test]
    fn every_milestone_renders_with_its_issue_and_headline() {
        let regs = Registries::load(&paths()).expect("registry");
        let rows = milestone_rows(&paths(), &regs);
        assert!(rows.len() >= 8, "found {}", rows.len());
        for r in &rows {
            assert!(r.issue > 0, "{}: issue {}", r.key, r.issue);
            assert!(!r.headline.is_empty(), "{}: no headline", r.key);
            assert!(
                r.url.contains("issues/"),
                "{}: no issue url ({})",
                r.key,
                r.url
            );
        }
    }

    /// A brief that is not on disk is flagged. The mechanism page is where the idea and the fair test
    /// are stated, so a milestone claiming one that does not exist is a milestone nobody can test.
    #[test]
    fn a_missing_mechanism_brief_is_flagged() {
        let regs = Registries::load(&paths()).expect("registry");
        let rows = milestone_rows(&paths(), &regs);
        let text = render(
            &paths(),
            Some(&regs),
            "deepseek",
            "2026-10-13T02:00:00-04:00",
        );

        // The marker is present exactly when some brief is missing, whatever the registry holds
        // today. Asserting a fixed count would make this test fail when a lab adds a brief.
        let missing_any = rows
            .iter()
            .any(|r| r.mechanisms.iter().any(|(_, exists)| !*exists));
        assert_eq!(text.contains("(BRIEF MISSING)"), missing_any);

        // And a brief that IS present is never marked.
        let present = rows
            .iter()
            .flat_map(|r| r.mechanisms.iter())
            .find(|(_, e)| *e);
        if let Some((id, _)) = present {
            assert!(
                !text.contains(&format!("{id} (BRIEF MISSING)")),
                "{id} exists on disk"
            );
        }
    }

    /// A lab that has never recorded a receipt has no shard. That is not an error.
    #[test]
    fn a_lab_with_no_shard_is_reported_as_never_recorded() {
        let v = receipts_view(&paths(), "no-such-lab", "2026-10-13T02:00:00-04:00");
        assert!(v.available, "{}", v.reason);
        assert!(!v.exists);
        assert_eq!(v.month_total, 0);
        assert!(v.last5.is_empty());
    }

    /// The rendered section must state that it is unported rather than showing one lab's shard as if
    /// it were the cross-lab picture. `receipts_view` is correct AND tested; it is the RENDERING that
    /// is withheld, because a section whose whole purpose is cross-lab visibility is worse than
    /// absent when it shows a quarter of the labs.
    #[test]
    fn the_receipts_section_states_that_it_is_unported() {
        let regs = Registries::load(&paths()).expect("registry");
        let text = render(
            &paths(),
            Some(&regs),
            "deepseek",
            "2026-10-13T02:00:00-04:00",
        );
        let section = text
            .split("2. RECEIPTS / VISIBILITY")
            .nth(1)
            .expect("receipts section");
        assert!(section.contains("not ported"), "{section}");
        assert!(
            section.contains("all four labs") || section.contains("shards"),
            "the reason names what is missing: {section}"
        );
    }

    /// A lab's own shard still reads correctly through the API, even though the view withholds it.
    #[test]
    fn a_single_lab_shard_reads_correctly() {
        let v = receipts_view(&paths(), "deepseek", "2026-10-13T02:00:00-04:00");
        assert!(v.exists, "this seat has recorded receipts");
        assert!(v.month_total > 0, "and the month total counts them");
        assert!(!v.last5.is_empty(), "newest first");
        let ts: Vec<&str> = v
            .last5
            .iter()
            .filter_map(|r| r.get("ts").and_then(Value::as_str))
            .collect();
        let mut sorted = ts.clone();
        sorted.sort_by(|a, b| b.cmp(a));
        assert_eq!(ts, sorted, "last5 is newest first");
    }

    #[test]
    fn receipt_kind_counts_and_id_counts_render() {
        assert_eq!(kind_text(&[]), "no kinds");
        assert_eq!(kind_text(&[("skill".into(), 2)]), "skill 2");
        assert_eq!(id_counts(&[]), "none in this ledger month");
        assert_eq!(
            id_counts(&[("b".into(), 1), ("a".into(), 3)]),
            "a x3, b x1",
            "sorted, because two runs must agree"
        );
    }

    #[test]
    fn a_receipt_brief_shows_the_fields_worth_showing() {
        let rec = serde_json::json!({"kind": "skill", "skill": "harness", "note": "loaded"});
        assert_eq!(receipt_brief(&rec), "skill=harness note=loaded");
        assert_eq!(
            receipt_brief(&serde_json::json!({"kind": "heartbeat"})),
            "(no fields)"
        );
    }
}
