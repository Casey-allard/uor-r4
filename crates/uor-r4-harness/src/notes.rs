//! Unread notes: what other labs said that this lab has not seen.
//!
//! **Presence without awareness is half the plane.** The board says who is working; it does not say
//! that Claude posted a CONTEST twenty minutes ago and nobody has read it. That happened twice on
//! 13 October — the bus is issue comments and **nobody polls them programmatically** — so the fix is
//! to make the check-in itself tell you what you have missed.
//!
//! The mechanism is a **last-seen marker per lab, in the live tier**, so it never dirties a checkout
//! (R10). `uor-harness notes` fetches each milestone issue's comments, compares their ids against the
//! marker, and reports:
//!
//! ```text
//! 3 unread notes on #2029 (latest: 6148132158, kimi, "Claude's CONTEST is answered")
//! ```
//!
//! ## Why it polls rather than subscribes
//!
//! GitHub has no push channel a lab can hold open across sessions, and the retired `lab-runner coord`
//! daemon is explicitly forbidden (see the recorded finding `mesh-is-not-a-coordination-concept`).
//! Polling at the check-in cadence is the design that fits the bus this project actually has: a lab
//! already runs `heartbeat` every five minutes, so a marker read costs nothing extra.
//!
//! ## What it does not do
//!
//! It does not read bodies into the marker or the report. A note is surfaced by **id, author and first
//! line** — enough to decide whether to open it, which is the whole point. Doing more would mean
//! copying issue text into a local file, and the issues are the authority.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::state::StatePaths;

/// One comment on a milestone issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: i64,
    pub issue: i64,
    pub author: String,
    pub created_at: String,
    /// The first line, trimmed and shortened. Enough to decide whether to open it.
    pub first_line: String,
}

/// A lab's place in the stream: the highest comment id it has seen, per issue.
///
/// Keyed by issue so a lab can be current on #2029 and behind on #2032 — the states are independent,
/// and one marker for everything would mark a whole backlog read the moment a single issue was
/// caught up.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Markers {
    pub lab: String,
    pub seen: BTreeMap<i64, i64>,
}

/// Where the marker lives: `~/.local/share/uor-r4/harness/notes/<lab>.json`.
///
/// Live tier, so a check-in that reads and advances it writes nothing into a checkout.
pub fn marker_path(paths: &StatePaths, lab: &str) -> PathBuf {
    paths
        .live
        .join("notes")
        .join(format!("{}.json", crate::state::slug(lab)))
}

impl Markers {
    pub fn new(lab: &str) -> Self {
        Markers {
            lab: lab.to_string(),
            seen: BTreeMap::new(),
        }
    }

    /// Load a lab's markers, or empty ones when it has never looked.
    pub fn load(paths: &StatePaths, lab: &str) -> Self {
        let path = marker_path(paths, lab);
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Markers::new(lab);
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            return Markers::new(lab);
        };
        let mut out = Markers::new(lab);
        if let Some(seen) = v.get("seen").and_then(|s| s.as_object()) {
            for (issue, id) in seen {
                if let (Ok(issue), Some(id)) = (issue.parse::<i64>(), id.as_i64()) {
                    out.seen.insert(issue, id);
                }
            }
        }
        out
    }

    /// Persist the markers. Best-effort: a read-only live tier must not fail a check-in.
    pub fn save(&self, paths: &StatePaths) -> bool {
        let path = marker_path(paths, &self.lab);
        if let Some(parent) = path.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                return false;
            }
        }
        let seen: serde_json::Map<String, serde_json::Value> = self
            .seen
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::Value::from(*v)))
            .collect();
        let body = serde_json::json!({
            "lab": self.lab,
            "seen": serde_json::Value::Object(seen),
        });
        std::fs::write(&path, format!("{body}\n")).is_ok()
    }

    /// Notes newer than the marker for their issue.
    ///
    /// A lab that has never looked sees every note as unread, which is deliberately noisy: joining a
    /// project mid-stream means there is a backlog, and hiding it would be the exact failure this
    /// module exists to prevent.
    pub fn unread<'a>(&self, notes: &'a [Note]) -> Vec<&'a Note> {
        notes
            .iter()
            .filter(|n| {
                let seen = self.seen.get(&n.issue).copied().unwrap_or(0);
                n.id > seen
            })
            .collect()
    }

    /// Advance the marker past every note given.
    ///
    /// Only ever forward: a marker that could go backwards would re-report notes a lab has already
    /// dismissed, and a report that cries wolf gets ignored — the same reason the shard guard allows
    /// reordering.
    pub fn advance(&mut self, notes: &[Note]) {
        for n in notes {
            let entry = self.seen.entry(n.issue).or_insert(0);
            if n.id > *entry {
                *entry = n.id;
            }
        }
    }
}

/// Parse the compact `gh api` output into notes.
///
/// `gh` emits JSON; rather than take a dependency on walking a dynamic tree for four fields, the
/// caller asks `gh` for tab-separated values and this reads them. A line that does not have the
/// expected shape is **skipped rather than fatal**: a malformed line means one note is not surfaced,
/// which is bad, but failing the whole call means none are.
pub fn parse_tsv(issue: i64, text: &str) -> Vec<Note> {
    let mut out = Vec::new();
    for line in text.lines() {
        let mut parts = line.splitn(4, '\t');
        let (Some(id), Some(author), Some(created_at)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let Ok(id) = id.trim().parse::<i64>() else {
            continue;
        };
        let body = parts.next().unwrap_or("");
        out.push(Note {
            id,
            issue,
            author: author.trim().to_string(),
            created_at: created_at.trim().to_string(),
            first_line: first_line(body),
        });
    }
    out
}

/// The first non-empty line, shortened. The default width is what fits a terminal row beside the
/// issue number and author.
pub fn first_line(body: &str) -> String {
    const WIDTH: usize = 72;
    let line = body
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        // Markdown furniture is noise in a one-line summary.
        .trim_start_matches(['#', '>', '-', '*', ' '])
        .trim();
    let collapsed = strip_emphasis(&line.split_whitespace().collect::<Vec<_>>().join(" "));
    if collapsed.chars().count() <= WIDTH {
        return collapsed;
    }
    let kept: String = collapsed.chars().take(WIDTH - 1).collect();
    format!("{}…", kept.trim_end())
}

/// Strip inline emphasis markers from a one-line summary.
///
/// `**CONTEST** the bar is vacuous` is how labs actually write, and the leading-strip above only
/// removes the first pair — leaving `CONTEST**`. Emphasis is meaningful in the note and noise in a
/// summary, so the paired markers go wherever they appear.
fn strip_emphasis(text: &str) -> String {
    let mut out = text.replace("**", "").replace("__", "");
    // Single-asterisk emphasis, but only when the markers are actually paired: a lone `*` can be
    // meaningful (a wildcard, a footnote marker) and stripping it would change the text.
    let stars = out.matches('*').count();
    if stars >= 2 && stars.is_multiple_of(2) {
        out = out.replace('*', "");
    }
    out
}

/// What a lab should be told, per issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unread {
    pub issue: i64,
    pub count: usize,
    pub latest: Note,
}

/// The most comments one issue read can return, set by GitHub's `per_page` maximum.
///
/// **A report showing this many unread is a FLOOR, not a count.** An issue with 400 comments and no
/// marker reports 100, and a lab reading "100 unread" would think it had seen the whole backlog when
/// it had seen the most recent page. The renderer says so rather than letting the number stand alone.
pub const PAGE_LIMIT: usize = 100;

/// Group unread notes by issue, newest first within each.
pub fn summarise(unread: &[&Note]) -> Vec<Unread> {
    let mut by_issue: BTreeMap<i64, Vec<&Note>> = BTreeMap::new();
    for n in unread {
        by_issue.entry(n.issue).or_default().push(n);
    }
    by_issue
        .into_iter()
        .filter_map(|(issue, mut notes)| {
            notes.sort_by_key(|n| n.id);
            let latest = (*notes.last()?).clone();
            Some(Unread {
                issue,
                count: notes.len(),
                latest,
            })
        })
        .collect()
}

/// Render the summary a check-in prints.
///
/// One line per issue, because a lab needs to know **where** the unread notes are before it needs to
/// know what they say. Empty when there is nothing to report — a check-in that always prints
/// "0 unread" trains the reader to skip the line it exists to deliver.
pub fn render(items: &[Unread]) -> Vec<String> {
    items
        .iter()
        .map(|u| {
            format!(
                "{}{} unread note(s) on #{} (latest: {}, {}, \"{}\")",
                if u.count >= PAGE_LIMIT {
                    "at least "
                } else {
                    ""
                },
                u.count,
                u.issue,
                u.latest.id,
                u.latest.author,
                u.latest.first_line
            )
        })
        .collect()
}

// ------------------------------------------------------------------------------- caching
//
// The fetch is a network call and a check-in happens every five minutes, so the notes seen by
// `heartbeat` are read from a cache the fetch writes. Two commands, two costs:
//
//   * `notes --refresh`  polls GitHub, appends to the cache, advances the marker. Deliberate.
//   * `notes`            reads the cache and reports what is unread. Cheap; safe in a check-in.
//
// The cache holds notes the lab has SEEN, not "unread" ones, so the marker stays the single source
// of what is unread — two places computing the same thing is how they come to disagree.

/// The cached note stream: `~/.local/share/uor-r4/harness/notes/<lab>.jsonl`.
pub fn cache_path(paths: &StatePaths, lab: &str) -> PathBuf {
    paths
        .live
        .join("notes")
        .join(format!("{}-notes.jsonl", crate::state::slug(lab)))
}

/// Append notes to the cache, skipping any id already present.
///
/// Idempotent by id: refreshing twice must not duplicate the stream, or every unread count doubles.
pub fn cache_append(paths: &StatePaths, lab: &str, notes: &[Note]) -> Result<usize, String> {
    let path = cache_path(paths, lab);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let existing = read_cache(paths, lab);
    let have: std::collections::BTreeSet<i64> = existing.iter().map(|n| n.id).collect();
    let fresh: Vec<&Note> = notes.iter().filter(|n| !have.contains(&n.id)).collect();
    if fresh.is_empty() {
        return Ok(0);
    }
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    for n in &fresh {
        let line = serde_json::json!({
            "id": n.id,
            "issue": n.issue,
            "author": n.author,
            "created_at": n.created_at,
            "first_line": n.first_line,
        });
        writeln!(file, "{line}").map_err(|e| format!("cannot append: {e}"))?;
    }
    Ok(fresh.len())
}

/// Read the cached notes, oldest first. A line that will not parse is skipped, not fatal.
pub fn read_cache(paths: &StatePaths, lab: &str) -> Vec<Note> {
    let Ok(text) = std::fs::read_to_string(cache_path(paths, lab)) else {
        return Vec::new();
    };
    let mut out: Vec<Note> = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let (Some(id), Some(issue)) = (
            v.get("id").and_then(|x| x.as_i64()),
            v.get("issue").and_then(|x| x.as_i64()),
        ) else {
            continue;
        };
        out.push(Note {
            id,
            issue,
            author: v
                .get("author")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            created_at: v
                .get("created_at")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            first_line: v
                .get("first_line")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    out.sort_by_key(|n| n.id);
    out
}

/// The milestone issues to watch, in order.
///
/// From the registry when it loads, so a new milestone joins the watch list without a code change;
/// the fallback is the four known ids so a checkout without a registry still surfaces notes.
pub fn watched_issues(regs: Option<&crate::registries::Registries>) -> Vec<i64> {
    match regs {
        Some(r) if !r.milestones.is_empty() => r.milestones.iter().map(|m| m.issue).collect(),
        _ => vec![2029, 2030, 2031, 2032],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(id: i64, issue: i64, author: &str) -> Note {
        Note {
            id,
            issue,
            author: author.into(),
            created_at: "2026-10-13T12:00:00Z".into(),
            first_line: format!("note {id}"),
        }
    }

    /// The whole point: a note posted after the marker is unread, one before it is not.
    #[test]
    fn only_notes_newer_than_the_marker_are_unread() {
        let mut m = Markers::new("deepseek");
        m.seen.insert(2029, 100);
        let notes = vec![
            note(90, 2029, "kimi"),
            note(100, 2029, "kimi"),
            note(101, 2029, "kimi"),
        ];
        let unread = m.unread(&notes);
        assert_eq!(unread.len(), 1, "only the note after the marker");
        assert_eq!(unread[0].id, 101);
    }

    /// A lab that has never looked sees everything. Deliberately noisy: joining mid-stream means a
    /// backlog, and hiding it is the failure this module exists to prevent.
    #[test]
    fn a_lab_that_has_never_looked_sees_everything() {
        let m = Markers::new("newlab");
        let notes = vec![note(1, 2029, "kimi"), note(2, 2029, "kimi")];
        assert_eq!(m.unread(&notes).len(), 2);
    }

    /// Markers are PER ISSUE. One marker for everything would mark a whole backlog read the moment a
    /// single issue was caught up.
    #[test]
    fn markers_are_independent_per_issue() {
        let mut m = Markers::new("deepseek");
        m.seen.insert(2029, 500);
        let notes = vec![note(400, 2029, "kimi"), note(300, 2032, "claude")];
        let unread = m.unread(&notes);
        assert_eq!(unread.len(), 1, "#2029 is caught up, #2032 is not");
        assert_eq!(unread[0].issue, 2032);
    }

    /// The marker only moves forward. A marker that could go backwards re-reports dismissed notes,
    /// and a report that cries wolf gets ignored.
    #[test]
    fn the_marker_only_advances() {
        let mut m = Markers::new("deepseek");
        m.advance(&[note(500, 2029, "kimi")]);
        assert_eq!(m.seen.get(&2029), Some(&500));
        m.advance(&[note(400, 2029, "kimi")]);
        assert_eq!(
            m.seen.get(&2029),
            Some(&500),
            "an older note must not pull it back"
        );
    }

    #[test]
    fn advancing_past_every_note_clears_the_report() {
        let mut m = Markers::new("deepseek");
        let notes = vec![note(1, 2029, "kimi"), note(2, 2029, "kimi")];
        assert_eq!(m.unread(&notes).len(), 2);
        m.advance(&notes);
        assert!(m.unread(&notes).is_empty());
    }

    #[test]
    fn markers_round_trip_through_the_live_tier() {
        let dir = tempfile::tempdir().expect("tempdir");
        let live = dir.path().join("live");
        std::fs::create_dir_all(&live).expect("live");
        let p = StatePaths::resolve(dir.path(), None, Some(live.to_str().expect("utf8")));
        let mut m = Markers::new("deepseek");
        m.advance(&[note(77, 2029, "kimi")]);
        assert!(m.save(&p), "the marker saves");
        let back = Markers::load(&p, "deepseek");
        assert_eq!(back.seen.get(&2029), Some(&77));
        assert_eq!(back.lab, "deepseek");
        // And it landed in the LIVE tier, not the checkout.
        assert!(marker_path(&p, "deepseek").starts_with(p.live.as_path()));
    }

    #[test]
    fn a_missing_or_corrupt_marker_reads_as_never_looked() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        assert!(Markers::load(&p, "newlab").seen.is_empty());
    }

    #[test]
    fn tsv_lines_parse_and_a_bad_line_is_skipped_not_fatal() {
        let text = "101\tkimi\t2026-10-13T12:00:00Z\t**CONTEST** the bar is vacuous\n\
                    not-a-number\tx\ty\tz\n\
                    102\tclaude\t2026-10-13T12:05:00Z\tCONFIRM\n";
        let notes = parse_tsv(2029, text);
        assert_eq!(
            notes.len(),
            2,
            "the malformed line is skipped, the good ones survive"
        );
        assert_eq!(notes[0].id, 101);
        assert_eq!(notes[1].author, "claude");
        // Markdown furniture is stripped from the one-line summary.
        assert_eq!(notes[0].first_line, "CONTEST the bar is vacuous");
    }

    #[test]
    fn first_line_shortens_and_cleans() {
        assert_eq!(first_line(""), "");
        assert_eq!(first_line("\n\n  ## Heading\nbody"), "Heading");
        assert_eq!(first_line("> quoted"), "quoted");
        let long = first_line(&"word ".repeat(40));
        assert!(long.ends_with('…'), "{long}");
        assert!(long.chars().count() <= 72, "{} chars", long.chars().count());
    }

    #[test]
    fn the_summary_groups_by_issue_and_names_the_latest() {
        let notes = [
            note(1, 2029, "kimi"),
            note(3, 2029, "claude"),
            note(2, 2032, "codex"),
        ];
        let refs: Vec<&Note> = notes.iter().collect();
        let items = summarise(&refs);
        assert_eq!(items.len(), 2);
        let i2029 = items.iter().find(|u| u.issue == 2029).expect("#2029");
        assert_eq!(i2029.count, 2);
        assert_eq!(i2029.latest.id, 3, "the latest is named");
        assert_eq!(i2029.latest.author, "claude");
    }

    /// Nothing to report prints NOTHING. A check-in that always says "0 unread" trains the reader to
    /// skip the line it exists to deliver.
    #[test]
    fn nothing_unread_renders_nothing() {
        assert!(render(&[]).is_empty());
    }

    #[test]
    fn the_report_names_count_issue_and_latest() {
        let n = Note {
            id: 6148132158,
            issue: 2029,
            author: "kimi".into(),
            created_at: "2026-10-13T12:00:00Z".into(),
            first_line: "Claude's CONTEST is answered".into(),
        };
        let items = summarise(&[&n]);
        let lines = render(&items);
        assert_eq!(lines.len(), 1);
        let l = &lines[0];
        assert!(l.contains("1 unread note(s)"), "{l}");
        assert!(l.contains("#2029"), "{l}");
        assert!(l.contains("6148132158"), "{l}");
        assert!(l.contains("kimi"), "{l}");
        assert!(l.contains("CONTEST is answered"), "{l}");
    }
}
