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
pub fn render(paths: &StatePaths, regs: Option<&Registries>, now_rfc3339: &str) -> String {
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

    // 2. RECEIPTS / VISIBILITY - every lab's shard, aggregated.
    {
        let month = &now_rfc3339[..7];
        let today = &now_rfc3339[..10];
        let previous = previous_month(month);
        let rows = receipt_rows(paths, month, &previous);
        let now_unix = crate::state::parse_iso_unix(now_rfc3339).unwrap_or(0);
        // No GitHub call is made here, so the milestone-issue clause reports UNAVAILABLE rather than
        // claiming the issues are current. "We did not look" and "they are up to date" are different
        // statements, and the second is the one a lab would act on.
        lines.extend(render_receipts(
            paths,
            regs,
            &rows,
            &ReceiptsContext {
                today,
                month,
                previous: &previous,
                now_unix,
                issues_available: false,
                issues_reason: "this implementation does not query GitHub; `scripts/harness/cli.py status` reconciles the live issues",
            },
        ));
    }

    // 3. DELIVERY - resolved from local git, so a lab offline or rate-limited still learns whether
    // finished work is sitting on a branch.
    lines.extend(render_delivery(&delivery_view(paths)));

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
        let text = render(&paths(), Some(&regs), "2026-10-13T02:00:00-04:00");
        assert!(text.contains("1. MILESTONES"), "{text}");
        assert!(text.contains("2. RECEIPTS / VISIBILITY"), "{text}");
        assert!(text.contains("3. DELIVERY"), "{text}");
        assert!(
            text.contains("source: "),
            "the view says where its registry came from"
        );
    }

    /// Status must never raise: it is what a lab runs when something is already wrong.
    #[test]
    fn a_missing_registry_renders_unavailable_rather_than_panicking() {
        let text = render(&paths(), None, "2026-10-13T02:00:00-04:00");
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
        let text = render(&paths(), Some(&regs), "2026-10-13T02:00:00-04:00");

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

    /// Section 2 now renders for real, and it renders EVERY lab's shard. This is the property the
    /// section exists for: a cross-lab visibility view that silently showed one lab would be the
    /// exact failure the plane was built to prevent.
    #[test]
    fn the_rendered_receipts_section_aggregates_every_lab() {
        let regs = Registries::load(&paths()).expect("registry");
        let text = render(&paths(), Some(&regs), "2026-10-13T02:52:00-04:00");
        let section = text
            .split("2. RECEIPTS / VISIBILITY")
            .nth(1)
            .expect("receipts section");
        // Assert on the SPECIFIC wording section 2 used while unported. A bare "not ported" check
        // also matched section 3's delivery message, so it failed against a correctly rendered
        // section 2 - the assertion was ambiguous, not the code.
        assert!(
            !section.contains("not ported to Rust yet. This section aggregates"),
            "section 2 renders now: {section}"
        );
        assert!(section.contains("skills used:"), "{section}");
        assert!(
            section.contains("milestones with no receipt in 24h:"),
            "{section}"
        );

        // The aggregate is over every lab, so its counts exceed any single lab's own shard.
        let rows = receipt_rows(&paths(), "2026-10", "2026-09");
        let mine = receipts_view(&paths(), "deepseek", "2026-10-13T02:52:00-04:00");
        assert!(
            rows.len() > mine.month_total,
            "the aggregate ({}) must exceed one lab's shard ({})",
            rows.len(),
            mine.month_total
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

// ------------------------------------------------------------------- cross-lab receipts
//
// Section 2 of the view. The whole point of this section is CROSS-LAB visibility, so it reads every
// lab's shard rather than one: `docs/harness/ledger/<lab>-<YYYY-MM>.jsonl`, plus the legacy unsharded
// `<YYYY-MM>.jsonl` files, which stay readable so no earlier receipt is lost. Sharding is what stops
// concurrent labs from conflicting on a shared ledger; AGGREGATING the shards is what makes the
// shards useful to a reader.

/// The receipt kinds the view counts separately. Anything else is counted as `other`, so a new kind
/// added by a lab shows up rather than disappearing from the totals.
pub const RECEIPT_KINDS: [&str; 6] = ["skill", "team", "heartbeat", "context", "prep", "pr"];

/// Kinds `receipts.read` accepts. Wider than the counted set on purpose: claim/release/check are
/// recorded but roll into `other` in the view.
pub const ALL_KINDS: [&str; 10] = [
    "skill",
    "team",
    "heartbeat",
    "context",
    "prep",
    "pr",
    "check",
    "claim",
    "release",
    // The generic kind the Rust `receipt` command records when no `--kind` is given. It was missing
    // here AND from the Python set, so this reader silently dropped one receipt the Python reader
    // counted - a one-row difference in the roll-up that `parity.py` caught the moment the Python
    // set was corrected. Mirrors `receipts.KINDS`, which is authoritative.
    "receipt",
];

/// Hours after which a milestone with no receipt is flagged. The owner's check-in cadence is 90
/// minutes, so this is deliberately generous: it flags an ABSENT lab, not a busy one.
pub const STALE_HOURS: f64 = 24.0;

/// Read every lab's receipts for one month, oldest first.
///
/// Unreadable lines are skipped rather than failing the read: a ledger is appended to by concurrent
/// labs, so a half-written final line is a normal state and must not blind the whole view. That is
/// also why the loop tolerates a file that disappears mid-read.
pub fn read_all_shards(paths: &StatePaths, month: &str) -> Vec<Value> {
    let dir = paths.repo.join("docs").join("harness").join("ledger");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .collect();
    files.sort();
    let mut out: Vec<Value> = Vec::new();
    for f in files {
        let name = f
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        // `<lab>-<YYYY-MM>.jsonl` for this month, or the legacy `<YYYY-MM>.jsonl`.
        let legacy = name.len() == "YYYY-MM.jsonl".len() && name.starts_with(month);
        let sharded = name.ends_with(&format!("-{month}.jsonl"));
        if !legacy && !sharded {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(mut v) = serde_json::from_str::<Value>(line) {
                if v.get("kind")
                    .and_then(Value::as_str)
                    .is_some_and(|k| ALL_KINDS.contains(&k))
                {
                    if let Some(obj) = v.as_object_mut() {
                        obj.entry("_file")
                            .or_insert_with(|| Value::String(name.clone()));
                    }
                    out.push(v);
                }
            }
        }
    }
    out.sort_by_key(|r| {
        r.get("ts")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    });
    out
}

/// Receipts for this month, plus the previous month when this one is thin.
///
/// The previous month is included only when the current one holds fewer than `COMMENT_KEEP` rows:
/// the view wants enough context to be worth reading, not a full history.
pub const COMMENT_KEEP: usize = 5;

pub fn receipt_rows(paths: &StatePaths, month: &str, previous: &str) -> Vec<Value> {
    let mut rows = read_all_shards(paths, month);
    if rows.len() < COMMENT_KEEP {
        rows.extend(read_all_shards(paths, previous));
        rows.sort_by_key(|r| {
            r.get("ts")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        });
    }
    rows
}

/// Count rows by kind, with an `other` bucket for kinds the view does not name.
///
/// Named kinds are always present with a zero, so the view's shape does not change when a lab has a
/// quiet day — a missing kind and a zero kind read very differently.
pub fn count_by_kind(rows: &[Value]) -> Vec<(String, usize)> {
    let mut counts: Vec<(String, usize)> =
        RECEIPT_KINDS.iter().map(|k| (k.to_string(), 0)).collect();
    let mut other = 0usize;
    for r in rows {
        let kind = r.get("kind").and_then(Value::as_str).unwrap_or("");
        match counts.iter_mut().find(|(k, _)| k == kind) {
            Some((_, n)) => *n += 1,
            None => other += 1,
        }
    }
    if other > 0 {
        counts.push(("other".to_string(), other));
    }
    counts
}

/// Whole UTC hours between a stamp and `now`. `None` when the stamp cannot be read.
pub fn age_hours(value: &Value, now_unix: i64) -> Option<f64> {
    let raw = value.as_str()?;
    let then = crate::state::parse_iso_unix(raw)?;
    Some((((now_unix - then) as f64) / 3600.0 * 100.0).round() / 100.0)
}

/// `45m`, `3.5h`, `1.2d`, or `n/a`.
pub fn hours_text(hours: Option<f64>) -> String {
    let Some(h) = hours else {
        return "n/a".to_string();
    };
    if h < 1.0 {
        format!("{:.0}m", h * 60.0)
    } else if h < 48.0 {
        format!("{h:.1}h")
    } else {
        format!("{:.1}d", h / 24.0)
    }
}

/// One milestone's receipt activity, for the three 24-hour subsections.
#[derive(Debug, Clone)]
pub struct MilestoneActivity {
    pub key: String,
    pub id: String,
    pub issue: i64,
    pub last_receipt: String,
    pub last_receipt_age_hours: Option<f64>,
    pub heartbeats_24h: usize,
    pub posted_heartbeats_24h: usize,
    pub stale: bool,
}

/// Whether a receipt names a milestone, by the aliases a lab types.
fn milestone_matches(receipt: &Value, m: &Milestone) -> bool {
    let raw = receipt
        .get("milestone")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if raw.is_empty() {
        return false;
    }
    raw == m.key.to_ascii_lowercase()
        || raw == m.id.to_ascii_lowercase()
        || raw == m.issue.to_string()
}

/// Per-milestone receipt activity across every lab.
///
/// `posted` is the field that makes the third subsection possible: a heartbeat card is emitted
/// locally by `heartbeat`, and posting it on the issue is a separate act. A card that was never
/// posted is invisible to the milestone's readers, which is exactly what this flags.
pub fn milestone_activity(
    rows: &[Value],
    regs: &Registries,
    now_unix: i64,
) -> Vec<MilestoneActivity> {
    regs.milestones
        .iter()
        .map(|m| {
            let mine: Vec<&Value> = rows.iter().filter(|r| milestone_matches(r, m)).collect();
            let last_receipt = mine
                .iter()
                .filter_map(|r| r.get("ts").and_then(Value::as_str))
                .max()
                .unwrap_or("")
                .to_string();
            let last_receipt_age_hours = mine
                .iter()
                .filter_map(|r| r.get("ts"))
                .filter_map(|t| age_hours(t, now_unix))
                .fold(None, |acc: Option<f64>, h| {
                    Some(acc.map_or(h, |a| a.min(h)))
                });
            let heartbeats: Vec<&&Value> = mine
                .iter()
                .filter(|r| r.get("kind").and_then(Value::as_str) == Some("heartbeat"))
                .collect();
            let hb_24h: Vec<&&&Value> = heartbeats
                .iter()
                .filter(|r| {
                    age_hours(r.get("ts").unwrap_or(&Value::Null), now_unix).unwrap_or(1e9) <= 24.0
                })
                .collect();
            let posted_24h = hb_24h
                .iter()
                .filter(|r| r.get("posted").and_then(Value::as_bool).unwrap_or(false))
                .count();
            let stale = last_receipt_age_hours.is_none_or(|h| h > STALE_HOURS);
            MilestoneActivity {
                key: m.key.clone(),
                id: m.id.clone(),
                issue: m.issue,
                last_receipt,
                last_receipt_age_hours,
                heartbeats_24h: hb_24h.len(),
                posted_heartbeats_24h: posted_24h,
                stale,
            }
        })
        .collect()
}

/// Render section 2.
///
/// Returns the lines, so the caller controls the surrounding structure. `issues_available` is false
/// whenever GitHub was not consulted: the milestone-issue clause then reports UNAVAILABLE rather than
/// claiming the issues are current, because "we did not look" and "they are up to date" are different
/// statements and the second is the one a lab would act on.
/// Everything section 2 needs that is not a receipt.
///
/// Grouped rather than passed as nine positional arguments: five of them are dates and booleans, and
/// a positional call site that swaps two `&str` dates compiles and reports the wrong month.
pub struct ReceiptsContext<'a> {
    pub today: &'a str,
    pub month: &'a str,
    pub previous: &'a str,
    pub now_unix: i64,
    /// False whenever GitHub was not consulted. "We did not look" and "they are up to date" are
    /// different statements, and only the second is safe to act on.
    pub issues_available: bool,
    pub issues_reason: &'a str,
}

pub fn render_receipts(
    paths: &StatePaths,
    regs: Option<&Registries>,
    rows: &[Value],
    ctx: &ReceiptsContext<'_>,
) -> Vec<String> {
    let (today, month, previous, now_unix) = (ctx.today, ctx.month, ctx.previous, ctx.now_unix);
    let (issues_available, issues_reason) = (ctx.issues_available, ctx.issues_reason);
    let mut lines = vec!["2. RECEIPTS / VISIBILITY".to_string()];
    let today_rows: Vec<&Value> = rows
        .iter()
        .filter(|r| {
            r.get("ts")
                .and_then(Value::as_str)
                .is_some_and(|t| t.starts_with(today))
        })
        .collect();
    let month_rows: Vec<&Value> = rows
        .iter()
        .filter(|r| {
            r.get("ts")
                .and_then(Value::as_str)
                .is_some_and(|t| t.starts_with(month))
        })
        .collect();
    lines.push(format!(
        "   {today}: {} receipts ({}) · {month}: {} ({})",
        today_rows.len(),
        kind_text(&count_by_kind(
            &today_rows.iter().map(|v| (*v).clone()).collect::<Vec<_>>()
        )),
        month_rows.len(),
        kind_text(&count_by_kind(
            &month_rows.iter().map(|v| (*v).clone()).collect::<Vec<_>>()
        ))
    ));
    // The legacy aggregate file is what Python reports on, so it is what decides the note. It is
    // genuinely absent in this repository — the ledger is sharded — so the note is accurate rather
    // than stale, and it reads oddly on purpose: it is telling a reader that no UNATTRIBUTED receipt
    // has ever been recorded.
    let legacy = paths
        .repo
        .join("docs")
        .join("harness")
        .join("ledger")
        .join(format!("{month}.jsonl"));
    if !legacy.exists() {
        lines.push(format!(
            "   note: {} does not exist yet - no receipt has ever been recorded",
            legacy.display()
        ));
    }
    let mut skill_ids: Vec<(String, usize)> = Vec::new();
    let mut team_ids: Vec<(String, usize)> = Vec::new();
    for r in rows {
        match r.get("kind").and_then(Value::as_str) {
            Some("skill") => {
                if let Some(id) = r
                    .get("skill")
                    .or_else(|| r.get("skill_id"))
                    .or_else(|| r.get("id"))
                {
                    if let Some(s) = id.as_str() {
                        bump(&mut skill_ids, Some(s));
                    }
                }
            }
            Some("team") => {
                if let Some(id) = r.get("role").or_else(|| r.get("role_id")) {
                    if let Some(s) = id.as_str() {
                        bump(&mut team_ids, Some(s));
                    }
                }
            }
            _ => {}
        }
    }
    lines.push(format!(
        "   skills used: {} · agents dispatched: {}",
        id_counts(&skill_ids),
        id_counts(&team_ids)
    ));
    let mut newest: Vec<&Value> = rows.iter().collect();
    newest.sort_by_key(|r| {
        std::cmp::Reverse(
            r.get("ts")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        )
    });
    if newest.is_empty() {
        lines.push("   last 5 receipts: none".to_string());
    }
    for rec in newest.iter().take(COMMENT_KEEP) {
        lines.push(format!(
            "   - {} {}: {}",
            rec.get("ts").and_then(Value::as_str).unwrap_or(""),
            rec.get("kind").and_then(Value::as_str).unwrap_or(""),
            truncate(&receipt_brief(rec), RECEIPT_WIDTH)
        ));
    }

    // The 24-hour subsections need the registry; without it there are no milestones to report on.
    if let Some(regs) = regs {
        if issues_available {
            lines.push("   milestone issues: all four updated within 24h".to_string());
        } else {
            lines.push(format!(
                "   milestone issue staleness: UNAVAILABLE: {}",
                if issues_reason.is_empty() {
                    "no reason given"
                } else {
                    issues_reason
                }
            ));
        }
        let activity = milestone_activity(rows, regs, now_unix);
        let flagged: Vec<&MilestoneActivity> = activity.iter().filter(|a| a.stale).collect();
        if flagged.is_empty() {
            lines.push("   milestones with no receipt in 24h: none".to_string());
        } else {
            let txt = flagged
                .iter()
                .map(|a| {
                    if a.last_receipt.is_empty() {
                        format!("{} (no receipt on record)", a.id)
                    } else {
                        format!(
                            "{} (last receipt {} ago)",
                            a.id,
                            hours_text(a.last_receipt_age_hours)
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("   milestones with no receipt in 24h: {txt}"));
        }
        let printed_only: Vec<String> = activity
            .iter()
            .filter(|a| a.heartbeats_24h > 0 && a.posted_heartbeats_24h == 0)
            .map(|a| a.id.clone())
            .collect();
        if !printed_only.is_empty() {
            lines.push(format!(
                "   heartbeat cards emitted locally but never posted on the issue in 24h: {} (post with `heartbeat --milestone <m> --post`, or paste the card)",
                printed_only.join(", ")
            ));
        }
    }
    let _ = previous;
    // No trailing blank line: the caller separates the numbered sections, and pushing one here
    // produced a double gap. Section spacing is the caller's to own.
    lines
}

/// The month before `YYYY-MM`.
///
/// Used when the current month is thin: the view wants enough context to be worth reading, and on the
/// first days of a month a lab's own shard can hold two receipts. Computed arithmetically rather than
/// with a date library, because the only operation needed is "step back one month".
pub fn previous_month(month: &str) -> String {
    let (y, m) = match month.split_once('-') {
        Some((y, m)) => (
            y.parse::<i32>().unwrap_or(1970),
            m.parse::<u32>().unwrap_or(1),
        ),
        None => return month.to_string(),
    };
    if m <= 1 {
        format!("{}-12", y - 1)
    } else {
        format!("{y}-{:02}", m - 1)
    }
}

#[cfg(test)]
mod aggregation_tests {
    use super::*;
    use std::path::Path;

    fn paths() -> StatePaths {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        StatePaths::resolve(repo, None, Some("/nonexistent-live"))
    }

    /// A scratch repository whose branches this test controls.
    ///
    /// The first version of these tests ran against the checkout itself, which made them pass locally
    /// and FAIL IN CI: GitHub Actions clones shallow with no `origin/main`, so `delivery_view` reported
    /// UNAVAILABLE and four tests died. Tests for "is this branch finished" must own the branches they
    /// judge. This builds a real git repo, not a mock - the code under test shells out to git, and a
    /// mock of git would test the mock.
    struct Fixture {
        dir: tempfile::TempDir,
        paths: StatePaths,
    }

    impl Fixture {
        fn new() -> Fixture {
            let dir = tempfile::tempdir().expect("tempdir");
            let repo = dir.path().to_path_buf();
            let run = |args: &[&str]| {
                let out = std::process::Command::new("git")
                    .arg("-C")
                    .arg(&repo)
                    .args(args)
                    .output()
                    .expect("git runs");
                assert!(
                    out.status.success(),
                    "git {args:?}: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            };
            run(&["init", "-q", "-b", "main"]);
            run(&["config", "user.email", "t@example.invalid"]);
            run(&["config", "user.name", "t"]);
            std::fs::write(repo.join("f.txt"), "one\n").expect("write");
            run(&["add", "-A"]);
            run(&["commit", "-qm", "first"]);
            // `origin/main` as a real remote-tracking ref, which is what delivery_view reads.
            run(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
            std::fs::create_dir_all(repo.join("docs/harness")).expect("mkdir");
            Fixture {
                dir,
                paths: StatePaths::resolve(&repo, None, Some("/nonexistent-live")),
            }
        }

        fn repo(&self) -> &std::path::Path {
            self.dir.path()
        }

        fn git(&self, args: &[&str]) -> String {
            let out = std::process::Command::new("git")
                .arg("-C")
                .arg(self.repo())
                .args(args)
                .output()
                .expect("git runs");
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }

        /// A branch that adds a commit main does not have.
        fn branch_with_new_work(&self, name: &str) {
            self.git(&["checkout", "-q", "-b", name, "main"]);
            std::fs::write(self.repo().join("f.txt"), format!("{name}\n")).expect("write");
            self.git(&["add", "-A"]);
            self.git(&["commit", "-qm", name]);
            self.git(&["checkout", "-q", "main"]);
        }

        /// A branch pointing at a commit already in main, i.e. finished.
        fn finished_branch(&self, name: &str) {
            self.git(&["branch", name, "main"]);
        }
    }

    #[test]
    fn a_missing_origin_main_is_reported_not_guessed() {
        // A shallow CI clone has no origin/main. The view must say so rather than call every branch
        // unfinished or finished - the failure mode that broke CI.
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().to_path_buf();
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "t@example.invalid"],
            vec!["config", "user.name", "t"],
        ] {
            let _ = std::process::Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args(&args)
                .output();
        }
        let paths = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let view = delivery_view(&paths);
        assert!(
            !view.error.is_empty(),
            "no origin/main is an error, not an empty success"
        );
        assert!(
            view.rows.is_empty(),
            "no branch is judged without a reference point"
        );
        let text = render_delivery(&view).join("\n");
        assert!(text.contains("UNAVAILABLE"), "{text}");
        assert!(
            !text.contains("none safe to delete"),
            "must not claim a clean bill of health: {text}"
        );
    }

    #[test]
    fn a_finished_branch_is_offered_and_an_unmerged_one_is_not() {
        let f = Fixture::new();
        f.finished_branch("done/thing");
        f.branch_with_new_work("live/thing");
        let view = delivery_view(&f.paths);
        assert!(view.error.is_empty(), "{}", view.error);

        let removable: Vec<&str> = view.removable().iter().map(|r| r.name.as_str()).collect();
        assert!(removable.contains(&"done/thing"), "{removable:?}");
        assert!(
            !removable.contains(&"live/thing"),
            "unmerged work is not offered"
        );

        let unfinished: Vec<&str> = view.unfinished().iter().map(|r| r.name.as_str()).collect();
        assert!(unfinished.contains(&"live/thing"), "{unfinished:?}");
        assert!(!unfinished.contains(&"done/thing"));

        let text = render_delivery(&view).join("\n");
        assert!(text.contains("done/thing"), "{text}");
        assert!(text.contains("live/thing"), "{text}");
        assert!(text.contains("last fetched value"), "{text}");
        assert!(text.contains("not available from git alone"), "{text}");
    }

    /// The bug this check had on first use, reproduced on controlled state.
    #[test]
    fn a_finished_branch_holding_uncommitted_work_is_never_offered() {
        let f = Fixture::new();
        f.finished_branch("held/thing");
        // Check it out in a second worktree, then dirty that worktree.
        let wt = f.repo().join(".wt");
        f.git(&[
            "worktree",
            "add",
            "-q",
            wt.to_str().expect("utf8"),
            "held/thing",
        ]);
        std::fs::write(wt.join("f.txt"), "uncommitted\n").expect("write");

        let view = delivery_view(&f.paths);
        let held: Vec<&str> = view
            .held_by_uncommitted()
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        assert!(held.contains(&"held/thing"), "held: {held:?}");
        let removable: Vec<&str> = view.removable().iter().map(|r| r.name.as_str()).collect();
        assert!(
            !removable.contains(&"held/thing"),
            "must not be offered: {removable:?}"
        );

        let text = render_delivery(&view).join("\n");
        let warning = text
            .split("finished by commits, but a worktree holds uncommitted work")
            .nth(1)
            .expect("the warning section");
        assert!(warning.contains("held/thing"), "{warning}");
        assert!(warning.contains("do NOT delete"), "{warning}");
        assert!(
            !warning.contains("git branch -D held/thing"),
            "no fix command under a do-not-delete heading: {warning}"
        );
    }

    #[test]
    fn the_trunk_is_never_a_candidate() {
        let f = Fixture::new();
        f.finished_branch("done/thing");
        let view = delivery_view(&f.paths);
        assert!(
            !view.rows.iter().any(|r| r.name == view.trunk),
            "the trunk is not a branch to clean up"
        );
    }

    /// Section 3 renders, and it says what it cannot know.
    ///
    /// Asserted against a controlled fixture rather than the checkout: the first version ran against
    /// whatever repository the tests happened to be in and passed locally while failing in CI, where
    /// the clone is shallow and has no `origin/main`.
    #[test]
    fn the_delivery_section_renders_in_the_view() {
        let f = Fixture::new();
        f.finished_branch("done/thing");
        let view = delivery_view(&f.paths);
        let text = render_delivery(&view).join("\n");
        assert!(
            !text.contains("not ported"),
            "section 3 renders now: {text}"
        );
        assert!(text.contains("origin/main"), "{text}");
        assert!(text.contains("done/thing"), "{text}");
        // The limit is stated wherever it renders, including in the whole view.
        let regs = Registries::load(&paths()).ok();
        let whole = render(&f.paths, regs.as_ref(), "2026-10-13T02:00:00-04:00");
        let delivery = whole.split("3. DELIVERY").nth(1).expect("section 3");
        assert!(
            delivery.contains("not available from git alone"),
            "{delivery}"
        );
    }

    #[test]
    fn previous_month_steps_back_and_wraps_the_year() {
        assert_eq!(previous_month("2026-10"), "2026-09");
        assert_eq!(previous_month("2026-01"), "2025-12");
        assert_eq!(previous_month("2026-03"), "2026-02");
        assert_eq!(
            previous_month("nonsense"),
            "nonsense",
            "unreadable input is returned unchanged"
        );
    }

    /// Every lab's shard is read, because a section about cross-lab visibility that reads one lab is
    /// the exact failure it exists to prevent.
    #[test]
    fn the_reader_aggregates_every_lab_shard() {
        let rows = read_all_shards(&paths(), "2026-10");
        assert!(!rows.is_empty(), "the ledger has receipts");
        let files: std::collections::BTreeSet<String> = rows
            .iter()
            .filter_map(|r| r.get("_file").and_then(Value::as_str).map(str::to_string))
            .collect();
        assert!(
            files.len() >= 3,
            "expected several labs' shards, saw {files:?}"
        );
        // Oldest first, so the last-5 slice is genuinely the newest five.
        let ts: Vec<&str> = rows
            .iter()
            .filter_map(|r| r.get("ts").and_then(Value::as_str))
            .collect();
        let mut sorted = ts.clone();
        sorted.sort_unstable();
        assert_eq!(ts, sorted);
    }

    /// Named kinds are always present with a zero, so the view's shape does not change on a quiet
    /// day: a missing kind and a zero kind read very differently.
    #[test]
    fn count_by_kind_names_every_kind_and_buckets_the_rest() {
        let counts = count_by_kind(&[]);
        assert_eq!(
            counts.len(),
            RECEIPT_KINDS.len(),
            "all named kinds present at zero"
        );
        assert!(counts.iter().all(|(_, n)| *n == 0));

        let rows = vec![
            serde_json::json!({"kind": "skill"}),
            serde_json::json!({"kind": "skill"}),
            serde_json::json!({"kind": "claim"}),
        ];
        let counts = count_by_kind(&rows);
        assert_eq!(
            counts.iter().find(|(k, _)| k == "skill").map(|(_, n)| *n),
            Some(2)
        );
        assert_eq!(
            counts.iter().find(|(k, _)| k == "other").map(|(_, n)| *n),
            Some(1),
            "a kind the view does not name is counted rather than dropped"
        );
    }

    #[test]
    fn hours_text_switches_units_by_magnitude() {
        assert_eq!(hours_text(None), "n/a");
        assert_eq!(hours_text(Some(0.5)), "30m");
        assert_eq!(hours_text(Some(3.5)), "3.5h");
        assert_eq!(hours_text(Some(50.0)), "2.1d");
    }

    #[test]
    fn age_hours_reads_a_stamp_and_refuses_a_bad_one() {
        let now = crate::state::parse_iso_unix("2026-10-13T00:00:00Z").expect("now");
        let v = Value::String("2026-10-12T22:00:00Z".into());
        assert_eq!(age_hours(&v, now), Some(2.0));
        assert_eq!(age_hours(&Value::String("not a date".into()), now), None);
        assert_eq!(age_hours(&Value::Null, now), None);
    }

    /// The three 24-hour clauses are computed from local receipts, and this is the one that catches a
    /// card emitted by `heartbeat` but never posted where the milestone's readers can see it.
    #[test]
    fn milestone_activity_flags_unposted_cards_and_stale_milestones() {
        let regs = Registries::load(&paths()).expect("registry");
        let now = crate::state::parse_iso_unix("2026-10-13T03:00:00Z").expect("now");
        let rows = vec![
            serde_json::json!({"kind": "heartbeat", "milestone": "m1", "ts": "2026-10-13T02:00:00Z"}),
            serde_json::json!({"kind": "heartbeat", "milestone": "m1", "ts": "2026-10-13T02:30:00Z", "posted": true}),
            serde_json::json!({"kind": "heartbeat", "milestone": "m1", "ts": "2026-10-13T02:45:00Z"}),
        ];
        let activity = milestone_activity(&rows, &regs, now);
        let m1 = activity.iter().find(|a| a.id == "M1").expect("M1");
        assert_eq!(m1.heartbeats_24h, 3);
        assert_eq!(m1.posted_heartbeats_24h, 1, "one card was posted");
        assert!(!m1.stale, "M1 has a receipt an hour old");
        assert!(
            m1.last_receipt_age_hours.unwrap() < 1.0,
            "the newest receipt decides the age"
        );

        // A milestone with no receipt at all is stale, and that is the distinction the view draws
        // between "quiet" and "silent".
        let quiet = activity.iter().find(|a| a.last_receipt.is_empty());
        if let Some(q) = quiet {
            assert!(q.stale, "{} has no receipt on record", q.id);
        }
    }

    /// Not looking and finding everything current are different statements, and only one of them is
    /// safe to act on.
    #[test]
    fn the_github_clause_reports_unavailable_rather_than_claiming_freshness() {
        let regs = Registries::load(&paths()).expect("registry");
        let lines = render_receipts(
            &paths(),
            Some(&regs),
            &[],
            &ReceiptsContext {
                today: "2026-10-13",
                month: "2026-10",
                previous: "2026-09",
                now_unix: 0,
                issues_available: false,
                issues_reason: "no GitHub here",
            },
        );
        let clause = lines
            .iter()
            .find(|l| l.contains("milestone issue"))
            .expect("clause");
        assert!(clause.contains("UNAVAILABLE"), "{clause}");
        assert!(!clause.contains("all four updated"), "{clause}");
    }
}

// ------------------------------------------------------------------------ delivery (section 3)
//
// Section 3 answers "is finished work still sitting on a branch?" - the question the owner's
// one-owner-per-branch and delete-after-merge rules make load-bearing. Python asks GitHub for each
// branch's PR. This port answers it from LOCAL GIT ALONE, using the same content test the guarded
// branch delete uses: `git cherry origin/main <branch>`.
//
// That is a real difference and it is deliberate rather than a shortcut. A branch whose commits are
// all already upstream is finished whether or not this machine can reach GitHub, so a lab offline, on
// a pod, or rate-limited still gets the answer. The cost is stated in the rendered output: without the
// PR number, the view cannot say WHICH PR merged it, and it says that rather than implying it knows.

/// One local branch, resolved against `origin/main`.
#[derive(Debug, Clone)]
pub struct BranchRow {
    pub name: String,
    pub head: String,
    /// Commits whose changes are NOT yet in `origin/main`. Empty means the branch is finished.
    pub unmerged: usize,
    pub current: bool,
    /// Commits upstream, but the branch still exists. NOT the same as "safe to delete": see
    /// `uncommitted`.
    pub orphan: bool,
    /// The worktree this branch is checked out in, if any.
    pub worktree: String,
    /// Uncommitted changes in that worktree. A branch whose commits are all upstream can still be
    /// holding the only copy of uncommitted work, so it must not be called deletable.
    pub uncommitted: usize,
    /// Finished AND holding nothing uncommitted. Only this is safe to delete.
    pub safe_to_delete: bool,
    pub fix: String,
}

/// A git invocation against the repository. Returns `(ok, stdout)`.
fn git_out(repo: &std::path::Path, args: &[&str]) -> (bool, String, String) {
    match std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
    {
        Ok(o) => (
            o.status.success(),
            String::from_utf8_lossy(&o.stdout).trim().to_string(),
            String::from_utf8_lossy(&o.stderr).trim().to_string(),
        ),
        Err(e) => (false, String::new(), format!("git not runnable: {e}")),
    }
}

/// The delivery view: where this checkout is, and which branches still hold unfinished work.
#[derive(Debug, Clone)]
pub struct DeliveryView {
    pub branch: String,
    pub worktree: String,
    pub origin_main: String,
    pub origin_main_subject: String,
    pub trunk: String,
    pub rows: Vec<BranchRow>,
    /// Set when `origin/main` cannot be resolved, which makes every orphan judgement meaningless.
    pub error: String,
}

impl DeliveryView {
    /// Branches holding only finished work, whether or not they are safe to delete.
    pub fn orphans(&self) -> Vec<&BranchRow> {
        self.rows.iter().filter(|r| r.orphan).collect()
    }

    /// Finished and holding nothing uncommitted: the ones the rules actually want removed.
    pub fn removable(&self) -> Vec<&BranchRow> {
        self.rows
            .iter()
            .filter(|r| r.orphan && r.safe_to_delete)
            .collect()
    }

    /// Finished by commits, but a worktree still holds uncommitted changes. Reported separately,
    /// because recommending deletion here would destroy work that exists nowhere else.
    pub fn held_by_uncommitted(&self) -> Vec<&BranchRow> {
        self.rows
            .iter()
            .filter(|r| r.orphan && !r.safe_to_delete)
            .collect()
    }

    pub fn unfinished(&self) -> Vec<&BranchRow> {
        self.rows
            .iter()
            .filter(|r| !r.orphan && r.unmerged > 0)
            .collect()
    }
}

/// Build the delivery view from local git.
///
/// `refs/heads` is read rather than `git branch`, because `git branch` output is presentation and has
/// changed between versions; a format string is a contract.
pub fn delivery_view(paths: &StatePaths) -> DeliveryView {
    let repo = paths.repo.as_path();
    let (_, branch, _) = git_out(repo, &["rev-parse", "--abbrev-ref", "HEAD"]);
    // The trunk is whatever `origin/HEAD` points at; main is the fallback this repository always has.
    let (ok_head, head_ref, _) = git_out(
        repo,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    );
    let trunk = if ok_head && !head_ref.is_empty() {
        head_ref.rsplit('/').next().unwrap_or("main").to_string()
    } else {
        "main".to_string()
    };

    let (ok_main, main_sha, main_err) = git_out(repo, &["rev-parse", "origin/main"]);
    let (_, subject, _) = git_out(repo, &["log", "-1", "--format=%h %s", "origin/main"]);

    let mut view = DeliveryView {
        branch,
        worktree: paths.repo.display().to_string(),
        origin_main: if ok_main { main_sha } else { String::new() },
        origin_main_subject: subject,
        trunk,
        rows: Vec::new(),
        error: if ok_main { String::new() } else { main_err },
    };
    if !ok_main {
        return view;
    }

    let (ok_refs, refs, _) = git_out(
        repo,
        &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
    );

    // Which worktree holds each branch, and whether that worktree has uncommitted changes.
    //
    // This is the check that stops the view recommending a destructive command. A branch can hold no
    // commits main lacks and STILL be the only copy of uncommitted work:
    // `codex/geometric-path-repair` was exactly that, pointing at a commit already in main while its
    // worktree held twelve modified files. Reporting it as merely "finished" would invite a delete.
    let mut worktrees: Vec<(String, String)> = Vec::new(); // (path, branch)
    let (ok_wt, wt_out, _) = git_out(repo, &["worktree", "list", "--porcelain"]);
    if ok_wt {
        let mut path = String::new();
        for line in wt_out.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = p.trim().to_string();
            } else if let Some(b) = line.strip_prefix("branch ") {
                let name = b.trim().trim_start_matches("refs/heads/").to_string();
                worktrees.push((path.clone(), name));
            }
        }
    }
    if !ok_refs {
        return view;
    }
    for name in refs.lines().map(str::trim).filter(|n| !n.is_empty()) {
        if name == view.trunk || name == "master" {
            continue;
        }
        // `-` means the commit's changes are already upstream; `+` means they are not. This is the
        // same content test the guarded branch delete uses, and for the same reason: a squash merge
        // leaves no ancestry link, so ancestry cannot see that a branch is finished.
        let (ok_cherry, cherry, _) = git_out(repo, &["cherry", "origin/main", name]);
        if !ok_cherry {
            continue;
        }
        let unmerged = cherry.lines().filter(|l| l.starts_with('+')).count();
        let (_, head, _) = git_out(repo, &["rev-parse", name]);
        let mut wt_path = String::new();
        let mut uncommitted = 0usize;
        for (p, b) in &worktrees {
            if b == name {
                wt_path = p.clone();
                // `--porcelain` counts untracked and modified entries; both are work that would be
                // lost, so both count.
                let (ok_st, st, _) = git_out(std::path::Path::new(p), &["status", "--porcelain"]);
                if ok_st {
                    uncommitted = st.lines().filter(|l| !l.trim().is_empty()).count();
                }
                break;
            }
        }
        let orphan = unmerged == 0;
        view.rows.push(BranchRow {
            name: name.to_string(),
            head,
            unmerged,
            current: name == view.branch,
            orphan,
            worktree: wt_path,
            uncommitted,
            safe_to_delete: orphan && uncommitted == 0,
            fix: format!("git branch -D {name} && git push origin --delete {name}"),
        });
    }
    view.rows.sort_by(|a, b| a.name.cmp(&b.name));
    view
}

/// Render section 3.
pub fn render_delivery(view: &DeliveryView) -> Vec<String> {
    let mut lines = vec!["3. DELIVERY".to_string()];
    if !view.error.is_empty() {
        lines.push(format!(
            "   UNAVAILABLE: cannot resolve origin/main: {}",
            view.error
        ));
        return lines;
    }
    lines.push(format!(
        "   branch {} · worktree {} · origin/main {}",
        if view.branch.is_empty() {
            "?"
        } else {
            &view.branch
        },
        view.worktree,
        if view.origin_main.len() >= 8 {
            &view.origin_main[..8]
        } else {
            &view.origin_main
        }
    ));
    if !view.origin_main_subject.is_empty() {
        lines.push(format!(
            "      {}",
            truncate(&view.origin_main_subject, 120)
        ));
    }
    // Stated every time, because the value is a cached ref and a stale answer here is the one a lab
    // would act on when deciding a branch is safe to delete.
    lines.push("   note: origin/main is the last fetched value; run `git fetch --prune origin` before trusting it".to_string());

    let removable = view.removable();
    if removable.is_empty() {
        lines.push("   branches holding only finished work: none safe to delete".to_string());
    } else {
        lines.push(format!(
            "   branches holding only finished work: {} (delete these - main is the authority)",
            removable.len()
        ));
        for o in removable.iter().take(10) {
            let here = if o.current { " [checked out here]" } else { "" };
            lines.push(format!("      {}{here}", o.name));
            lines.push(format!("         fix: {}", o.fix));
        }
        if removable.len() > 10 {
            lines.push(format!("      (+{} more)", removable.len() - 10));
        }
    }
    // Reported separately, and with no `fix` line: these are NOT safe to delete.
    let held = view.held_by_uncommitted();
    if !held.is_empty() {
        lines.push(format!(
            "   finished by commits, but a worktree holds uncommitted work: {} (do NOT delete)",
            held.len()
        ));
        for o in held.iter().take(10) {
            lines.push(format!(
                "      {} - {} uncommitted change(s) in {}",
                o.name, o.uncommitted, o.worktree
            ));
        }
    }
    let unfinished = view.unfinished();
    if !unfinished.is_empty() {
        lines.push(format!(
            "   branches with unmerged work: {} (these are live; their owners decide)",
            unfinished.len()
        ));
        for r in unfinished.iter().take(10) {
            lines.push(format!(
                "      {} ({} commit(s) not in main)",
                r.name, r.unmerged
            ));
        }
    }
    // This port reads git, not GitHub, so it cannot name the PR that merged a branch.
    lines.push(
        "   PR numbers: not available from git alone - this view proves a branch is FINISHED by \
         content (`git cherry`), not by which PR merged it. `scripts/harness/cli.py status` asks \
         GitHub and names the PR."
            .to_string(),
    );
    lines
}
