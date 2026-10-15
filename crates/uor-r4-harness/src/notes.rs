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
        std::fs::write(&path, self.body()).is_ok()
    }

    /// The marker file's contents.
    fn body(&self) -> String {
        let seen: serde_json::Map<String, serde_json::Value> = self
            .seen
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::Value::from(*v)))
            .collect();
        let body = serde_json::json!({
            "lab": self.lab,
            "seen": serde_json::Value::Object(seen),
        });
        format!("{body}\n")
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

/// The per-page size for a comment read. **Pagination follows it, so this is not a ceiling.**
///
/// It used to be a ceiling: the fetch asked for one page of 100 and stopped, so on #2029 (231
/// comments) the command reported an old note as "latest" and missed every recent one. `--paginate`
/// fixed the read; this constant is now only the page size.
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
///
/// `exact` says whether the read was complete. A paginated fetch is complete and the count is the
/// truth; a cached read after a failed fetch does not know what it missed, so it says "at least".
/// Reporting a floor as a fact is worse than reporting it as a floor.
pub fn render(items: &[Unread], exact: bool) -> Vec<String> {
    items
        .iter()
        .map(|u| {
            format!(
                "{}{} unread note(s) on #{} (latest: {}, {}, \"{}\")",
                if exact { "" } else { "at least " },
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

/// One open pull request, as much of it as seat detection reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrRow {
    pub number: i64,
    pub head: String,
    pub title: String,
    pub body: String,
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Whether `lab` appears in `text` as a whole word, case-insensitively.
fn mentions_word(text: &str, lab: &str) -> bool {
    let (text, lab) = (text.to_ascii_lowercase(), lab.to_ascii_lowercase());
    if lab.is_empty() {
        return false;
    }
    text.match_indices(&lab).any(|(i, m)| {
        let before = text[..i].chars().next_back();
        let after = text[i + m.len()..].chars().next();
        !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
    })
}

/// Open PRs where `lab` has a seat: its branch prefix `<lab>/`, or its name as a whole word in the
/// title or body. Ascending and deduplicated. PR comments are issue comments, so these numbers can
/// join the watched-issue list unchanged.
pub fn seat_prs(lab: &str, rows: &[PrRow]) -> Vec<i64> {
    let prefix = format!("{}/", lab.to_ascii_lowercase());
    let mut out: Vec<i64> = rows
        .iter()
        .filter(|r| {
            r.head.to_ascii_lowercase().starts_with(&prefix)
                || mentions_word(&r.title, lab)
                || mentions_word(&r.body, lab)
        })
        .map(|r| r.number)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Notes still unread per `markers` that are at least `minutes` old at `now_unix`.
///
/// A note whose time will not parse is not counted: an age we cannot compute is not evidence of one.
pub fn unread_older_than(notes: &[Note], markers: &Markers, now_unix: i64, minutes: i64) -> usize {
    markers
        .unread(notes)
        .iter()
        .filter(|n| {
            crate::state::parse_iso_unix(&n.created_at)
                .is_some_and(|t| now_unix - t >= minutes * 60)
        })
        .count()
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
        assert!(render(&[], true).is_empty());
    }

    /// A complete read reports the count as a fact; an incomplete one says "at least". Reporting a
    /// floor as a fact is worse than reporting it as a floor.
    #[test]
    fn an_incomplete_read_says_at_least() {
        let n = note(5, 2029, "kimi");
        let items = summarise(&[&n]);
        assert!(
            render(&items, true)[0].starts_with("1 unread"),
            "exact read states the count"
        );
        assert!(
            render(&items, false)[0].starts_with("at least 1"),
            "a read that does not know what it missed must not present a floor as the truth"
        );
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
        let lines = render(&items, true);
        assert_eq!(lines.len(), 1);
        let l = &lines[0];
        assert!(l.contains("1 unread note(s)"), "{l}");
        assert!(l.contains("#2029"), "{l}");
        assert!(l.contains("6148132158"), "{l}");
        assert!(l.contains("kimi"), "{l}");
        assert!(l.contains("CONTEST is answered"), "{l}");
    }
}

/// Past this, a cached note list is worth a line.
///
/// **Twenty minutes is the CONTEST window the PM named**, not a round number: past it, a cache can
/// already have missed a contest posted while the lab was working, which is the failure this closes.
pub const CACHE_FRESH_MINUTES: u64 = 20;

/// Where a COMPLETE refresh is recorded: `notes/<lab>-notes.refreshed`, beside the cache.
///
/// **Not the cache's own mtime.** `cache_append` writes whenever any issue returns a new note, so a
/// refresh that read #2028 and then timed out on #2029 used to leave a seconds-old cache that every
/// check-in for the next twenty minutes trusted as fresh. The stamp is written only when every
/// watched issue and the PR list were read, so a partial refresh stays stale and is retried.
pub fn refreshed_path(paths: &StatePaths, lab: &str) -> PathBuf {
    paths
        .live
        .join("notes")
        .join(format!("{}-notes.refreshed", crate::state::slug(lab)))
}

/// How long ago the lab last COMPLETED a refresh, or `None` when it never has on this machine.
///
/// Reads the stamp, never the cache: a cache file that exists (created empty by `ensure_state`, or
/// appended to by a partial refresh) says nothing about whether the lab has seen everything. An
/// older cache with no stamp therefore just triggers one refresh. Network-free on purpose.
pub fn refresh_age(paths: &StatePaths, lab: &str) -> Option<std::time::Duration> {
    let meta = std::fs::metadata(refreshed_path(paths, lab)).ok()?;
    std::time::SystemTime::now()
        .duration_since(meta.modified().ok()?)
        .ok()
}

/// `6h`, `45m`, `2d` - coarse on purpose: the question is whether this is stale, not exactly when.
fn human_age(d: std::time::Duration) -> String {
    let mins = d.as_secs() / 60;
    if mins < 60 {
        format!("{mins}m")
    } else if mins < 48 * 60 {
        format!("{}h", mins / 60)
    } else {
        format!("{}d", mins / (24 * 60))
    }
}

/// The line a check-in should print about its note cache, or `None` when the cache is fresh.
///
/// **A floor count with no age is not a warning - it is noise a lab learns to read past.** That is
/// how this lab missed a tasking: the check-in printed a floor from a cache hours old while 887
/// notes sat behind it, and nothing in the line said the data was stale.
///
/// Pure, so the thresholds are testable without a clock or a filesystem.
pub fn cache_line(age: Option<std::time::Duration>) -> Option<String> {
    match age {
        None => Some(NO_CACHE.to_string()),
        Some(d) if d.as_secs() < CACHE_FRESH_MINUTES * 60 => None,
        Some(d) => Some(format!(
            "notes were last refreshed {} ago, so the count above is a floor - run: uor-harness notes",
            human_age(d)
        )),
    }
}

/// Whether a check-in should refresh the cache from GitHub before reporting.
///
/// **A lab that only ever runs `heartbeat` used to never refresh**, so its cache aged forever and it
/// never saw a new comment - the CONTEST it most needed was exactly the one it could not see. Absent
/// or older than the CONTEST window refreshes; anything younger is served from the cache with no
/// network call, so the five-minute cadence costs one fetch per twenty minutes, not one per check-in.
///
/// Pure, so the boundary is testable without a clock or a network.
pub fn needs_refresh(age: Option<std::time::Duration>) -> bool {
    match age {
        None => true,
        Some(d) => d.as_secs() >= CACHE_FRESH_MINUTES * 60,
    }
}

/// Make sure the lab's cache and marker files exist, without changing what either says.
///
/// A lab that has run `notes` or `heartbeat` must leave state behind even when nothing was fetched,
/// or the next run cannot tell "never looked" from "looked, nothing there", and the board cannot
/// count for it. The marker is created with `create_new`, so a concurrent `--mark-seen` that wrote
/// it first is never overwritten and cannot move backwards. Creating the cache does NOT make it
/// fresh: freshness is the refresh stamp (`refresh_age`). Best-effort: an unwritable live tier must
/// not fail a check-in.
pub fn ensure_state(paths: &StatePaths, lab: &str) {
    let cache = cache_path(paths, lab);
    if let Some(parent) = cache.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if !cache.exists() {
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&cache);
    }
    let marker = marker_path(paths, lab);
    if let Some(parent) = marker.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // AlreadyExists is the expected case and means "leave it alone"; any other error is best-effort.
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker)
    {
        use std::io::Write;
        let _ = f.write_all(Markers::new(lab).body().as_bytes());
    }
}

/// Record that a COMPLETE refresh finished now, by writing the refresh stamp.
///
/// Called only when every watched issue and the PR list were read: a partial or failed refresh must
/// stay stale so the next check-in tries again.
pub fn mark_refreshed(paths: &StatePaths, lab: &str) {
    let path = refreshed_path(paths, lab);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let _ = std::fs::write(&path, format!("{now}\n"));
}

/// What one bounded `gh` call produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fetch<T> {
    /// Exited successfully within its time; the parsed output.
    Got(T),
    /// Ran but failed: non-zero exit (404, rate limit, jq error), unparseable output, or timed out.
    Failed,
    /// Could not be started at all (`gh` missing). Every later call would fail the same way.
    NotSpawned,
}

/// The whole-refresh budget across the PR list and every issue.
///
/// One call is bounded by `GH_TIMEOUT` in the binary; this bounds their SUM, so a slow network
/// cannot turn a check-in into minutes of 20-second waits.
pub const REFRESH_BUDGET: std::time::Duration = std::time::Duration::from_secs(45);

/// Read every watched issue into the cache: `(new notes appended, whether the read was complete)`.
///
/// **A failure on one issue does not blind the lab to the rest**: a 404'd registry issue, a jq
/// error or a slow #2029 used to stop the loop, so every later issue and every seat PR went unread
/// for good. Now a failed issue is skipped and makes the read partial. Two things stop it early,
/// each with one stderr line: `gh` cannot be started (every call would fail the same way), or the
/// overall `deadline` passed. `prs_listed` is whether the PR list was read; without it the read is
/// partial too. Only a complete read writes the refresh stamp.
///
/// `fetch` is injected so the policy is testable without a network.
pub fn refresh_with(
    paths: &StatePaths,
    lab: &str,
    issues: &[i64],
    prs_listed: bool,
    deadline: std::time::Instant,
    mut fetch: impl FnMut(i64) -> Fetch<Vec<Note>>,
) -> (usize, bool) {
    ensure_state(paths, lab);
    let mut fetched = 0usize;
    let mut complete = prs_listed;
    for (i, issue) in issues.iter().enumerate() {
        if std::time::Instant::now() >= deadline {
            eprintln!(
                "uor-harness: notes refresh ran out of time - skipped {} issue(s); using the cached notes",
                issues.len() - i
            );
            complete = false;
            break;
        }
        match fetch(*issue) {
            Fetch::Got(found) => match cache_append(paths, lab, &found) {
                Ok(n) => fetched += n,
                Err(e) => {
                    eprintln!("uor-harness: notes cache: {e}");
                    complete = false;
                }
            },
            Fetch::Failed => {
                eprintln!(
                    "uor-harness: could not read #{issue} (failed or timed out) - continuing; using the cached notes for it"
                );
                complete = false;
            }
            Fetch::NotSpawned => {
                eprintln!(
                    "uor-harness: could not run gh - skipped {} issue(s); using the cached notes",
                    issues.len() - i
                );
                complete = false;
                break;
            }
        }
    }
    if complete {
        mark_refreshed(paths, lab);
    }
    (fetched, complete)
}

/// Run a command, returning its stdout only if it exits successfully within `timeout`.
///
/// **A check-in must never be blocked by GitHub**, and `Command::output` waits forever on a hung
/// `gh`. So the child is spawned, its stdout drained on a thread (a full pipe would otherwise stall
/// the child and look like a timeout), and it is killed and reaped at the deadline. On a timeout the
/// reader thread is NOT joined: `kill` reaches only the direct child, and a grandchild (a `gh`
/// wrapper or shim) that inherited stdout would keep the pipe open and the join waiting. The thread
/// is left to finish on its own when that pipe closes. No new dependency: std only.
pub fn run_bounded(
    cmd: &mut std::process::Command,
    timeout: std::time::Duration,
) -> Fetch<Vec<u8>> {
    use std::io::Read;
    use std::process::Stdio;
    let Ok(mut child) = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return Fetch::NotSpawned;
    };
    let Some(mut pipe) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Fetch::Failed;
    };
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.read_to_end(&mut buf);
        buf
    });
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                return match reader.join() {
                    Ok(out) => Fetch::Got(out),
                    Err(_) => Fetch::Failed,
                };
            }
            // Exited non-zero: no need to wait for the reader either.
            Ok(Some(_)) => return Fetch::Failed,
            Ok(None) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Fetch::Failed;
            }
        }
    }
}

/// No cache at all used to print NOTHING, so a lab that never ran `notes` could not know there was
/// anything to run. That silence is the failure this closes.
pub const NO_CACHE: &str = "no notes cache on this machine yet - run: uor-harness notes";

/// Notes are cached, but no refresh has ever read EVERY watched source (an issue or the PR list
/// keeps failing). Saying "no notes cache" here was false - the cache is full - and sent the lab to a
/// command that is partial for the same reason, so it fixed nothing.
pub const NEVER_COMPLETE: &str = "notes are cached but no refresh has read every watched issue and PR yet, so the count above is a floor - see the \"could not\" line above";

/// The line a check-in prints about its notes: [`cache_line`], except that a lab holding cached
/// notes with no complete refresh is told THAT, not that it has no cache. Pure, so testable.
pub fn check_in_line(age: Option<std::time::Duration>, cached_notes: usize) -> Option<String> {
    match age {
        None if cached_notes > 0 => Some(NEVER_COMPLETE.to_string()),
        _ => cache_line(age),
    }
}

#[cfg(test)]
mod cache_line_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn a_fresh_cache_says_nothing() {
        assert!(cache_line(Some(Duration::from_secs(60))).is_none());
        assert!(cache_line(Some(Duration::from_secs(19 * 60))).is_none());
    }

    /// The boundary matters: past the CONTEST window, silence is what lets a lab miss one.
    #[test]
    fn the_contest_boundary_is_where_it_reports() {
        assert!(cache_line(Some(Duration::from_secs(20 * 60))).is_some());
        assert!(cache_line(Some(Duration::from_secs(21 * 60))).is_some());
    }

    #[test]
    fn a_stale_cache_names_its_age() {
        let line = cache_line(Some(Duration::from_secs(6 * 3600))).expect("a line");
        assert!(line.contains("6h"), "{line}");
        assert!(line.contains("floor"), "{line}");
        assert!(line.contains("uor-harness notes"), "{line}");
    }

    #[test]
    fn cached_notes_without_a_complete_refresh_are_not_called_no_cache() {
        let line = check_in_line(None, 12).expect("a line");
        assert!(!line.contains("no notes cache"), "{line}");
        assert!(line.contains("floor"), "{line}");
        assert_eq!(check_in_line(None, 0).as_deref(), Some(NO_CACHE));
        assert_eq!(check_in_line(Some(Duration::from_secs(60)), 12), None);
    }

    #[test]
    fn no_cache_at_all_is_reported() {
        assert!(cache_line(None)
            .expect("a line")
            .contains("uor-harness notes"));
    }

    /// Absent or past the CONTEST window refreshes; younger is served from the cache, offline.
    #[test]
    fn a_check_in_refreshes_only_an_absent_or_stale_cache() {
        assert!(needs_refresh(None), "no cache: first use must fetch");
        assert!(!needs_refresh(Some(Duration::from_secs(0))));
        assert!(!needs_refresh(Some(Duration::from_secs(19 * 60 + 59))));
        assert!(needs_refresh(Some(Duration::from_secs(20 * 60))));
        assert!(needs_refresh(Some(Duration::from_secs(6 * 3600))));
    }

    /// First use leaves both files behind; an existing marker is never rewritten backwards.
    #[test]
    fn ensure_state_creates_files_and_never_rewinds_a_marker() {
        let dir = tempfile::tempdir().expect("tempdir");
        let live = dir.path().join("live");
        let p = StatePaths::resolve(dir.path(), None, Some(live.to_str().expect("utf8")));
        ensure_state(&p, "antigravity");
        assert!(cache_path(&p, "antigravity").exists(), "cache created");
        assert!(marker_path(&p, "antigravity").exists(), "marker created");
        assert!(read_cache(&p, "antigravity").is_empty());
        let mut m = Markers::new("claude");
        m.seen.insert(2029, 500);
        assert!(m.save(&p));
        ensure_state(&p, "claude");
        assert_eq!(Markers::load(&p, "claude").seen.get(&2029), Some(&500));
    }

    #[test]
    fn ages_are_coarse() {
        assert_eq!(human_age(Duration::from_secs(45 * 60)), "45m");
        assert_eq!(human_age(Duration::from_secs(6 * 3600)), "6h");
        assert_eq!(human_age(Duration::from_secs(50 * 3600)), "2d");
    }
}

#[cfg(test)]
mod seat_and_age_tests {
    use super::*;

    fn pr(number: i64, head: &str, title: &str, body: &str) -> PrRow {
        PrRow {
            number,
            head: head.into(),
            title: title.into(),
            body: body.into(),
        }
    }

    #[test]
    fn branch_prefix_matches_case_insensitively() {
        let rows = [pr(7, "Claude/x", "t", ""), pr(8, "codex/y", "t", "")];
        assert_eq!(seat_prs("claude", &rows), vec![7]);
        assert_eq!(seat_prs("CLAUDE", &rows), vec![7]);
    }

    #[test]
    fn whole_word_in_title_or_body() {
        let rows = [
            pr(1, "o/a", "Review by Claude's lab", ""),
            pr(2, "o/b", "t", "cc (claude)"),
            pr(3, "o/c", "t", "see claude/foo"),
            pr(4, "o/d", "t", "ping @claude please"),
        ];
        assert_eq!(seat_prs("claude", &rows), vec![1, 2, 3, 4]);
    }

    #[test]
    fn longer_words_do_not_match() {
        let rows = [
            pr(1, "o/a", "claudette work", ""),
            pr(2, "o/b", "t", "claude_x and xclaude and claude2"),
        ];
        assert!(seat_prs("claude", &rows).is_empty());
    }

    #[test]
    fn ascending_deduped_and_other_labs_excluded() {
        let rows = [
            pr(9, "claude/a", "claude", "claude"),
            pr(3, "claude/b", "", ""),
            pr(5, "deepseek/z", "t", "b"),
            pr(3, "claude/b", "", ""),
        ];
        assert_eq!(seat_prs("claude", &rows), vec![3, 9]);
    }

    fn note(id: i64, created_at: &str) -> Note {
        Note {
            id,
            issue: 2029,
            author: "a".into(),
            created_at: created_at.into(),
            first_line: String::new(),
        }
    }

    #[test]
    fn unread_age_threshold() {
        let now = crate::state::parse_iso_unix("2026-10-14T12:16:00Z").expect("parse");
        let m = Markers::new("claude");
        let notes = [
            note(1, "2026-10-14T12:00:00Z"), // 16 min
            note(2, "2026-10-14T12:02:00Z"), // 14 min
            note(3, "garbage"),
        ];
        assert_eq!(unread_older_than(&notes, &m, now, 15), 1);
        assert_eq!(unread_older_than(&notes[1..], &m, now, 15), 0);
    }

    #[test]
    fn a_seen_note_is_not_counted() {
        let now = crate::state::parse_iso_unix("2026-10-14T12:16:00Z").expect("parse");
        let mut m = Markers::new("claude");
        m.seen.insert(2029, 1);
        assert_eq!(
            unread_older_than(&[note(1, "2026-10-14T12:00:00Z")], &m, now, 15),
            0
        );
    }
}

#[cfg(test)]
mod refresh_tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn paths(dir: &tempfile::TempDir) -> StatePaths {
        let live = dir.path().join("live");
        StatePaths::resolve(dir.path(), None, Some(live.to_str().expect("utf8")))
    }

    fn note(id: i64, issue: i64) -> Note {
        Note {
            id,
            issue,
            author: "kimi".into(),
            created_at: "2026-10-14T00:00:00Z".into(),
            first_line: "x".into(),
        }
    }

    /// F1: appending notes does not make the cache fresh; only a complete refresh's stamp does.
    #[test]
    fn a_partial_append_is_not_a_refresh() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = paths(&dir);
        ensure_state(&p, "antigravity");
        assert_eq!(cache_append(&p, "antigravity", &[note(1, 2028)]), Ok(1));
        assert!(
            needs_refresh(refresh_age(&p, "antigravity")),
            "append alone is not fresh"
        );
        assert_eq!(
            cache_line(refresh_age(&p, "antigravity")).as_deref(),
            Some(NO_CACHE),
            "never completed a refresh: NO_CACHE even though the cache file exists"
        );
        mark_refreshed(&p, "antigravity");
        assert!(!needs_refresh(refresh_age(&p, "antigravity")));
        assert!(cache_line(refresh_age(&p, "antigravity")).is_none());
    }

    /// F2: one failing issue in the middle does not stop later issues; the read is partial, unstamped.
    #[test]
    fn a_failed_issue_does_not_blind_the_rest() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = paths(&dir);
        let mut asked = Vec::new();
        let (n, exact) = refresh_with(
            &p,
            "claude",
            &[2028, 2029, 2030],
            true,
            Instant::now() + Duration::from_secs(60),
            |i| {
                asked.push(i);
                if i == 2029 {
                    Fetch::Failed
                } else {
                    Fetch::Got(vec![note(i * 10, i)])
                }
            },
        );
        assert_eq!(asked, vec![2028, 2029, 2030], "later issues still fetched");
        assert_eq!(n, 2);
        assert!(!exact);
        assert!(
            refresh_age(&p, "claude").is_none(),
            "a partial read writes no stamp"
        );
    }

    /// F2: missing gh stops early; a complete read stamps; F5: an unread PR list is partial.
    #[test]
    fn missing_gh_stops_and_only_a_complete_read_stamps() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = paths(&dir);
        let far = Instant::now() + Duration::from_secs(60);
        let mut asked = 0;
        let (_, exact) = refresh_with(&p, "claude", &[1, 2, 3], true, far, |_| {
            asked += 1;
            Fetch::NotSpawned
        });
        assert_eq!(asked, 1, "no gh: one try, then skip the rest");
        assert!(!exact);
        let (_, exact) = refresh_with(&p, "claude", &[1, 2], false, far, |_| Fetch::Got(vec![]));
        assert!(!exact, "PR list not read: partial");
        assert!(refresh_age(&p, "claude").is_none());
        let (_, exact) = refresh_with(&p, "claude", &[1, 2], true, far, |_| Fetch::Got(vec![]));
        assert!(exact);
        assert!(refresh_age(&p, "claude").is_some(), "complete read stamps");
    }

    /// F2: past the overall deadline the remaining issues are skipped and the read is partial.
    #[test]
    fn the_overall_deadline_skips_the_rest() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = paths(&dir);
        let mut asked = 0;
        let (_, exact) = refresh_with(&p, "claude", &[1, 2], true, Instant::now(), |_| {
            asked += 1;
            Fetch::Got(vec![])
        });
        assert_eq!(asked, 0);
        assert!(!exact);
    }

    /// F3: a grandchild holding stdout open cannot hold the bounded call past its timeout.
    #[cfg(unix)]
    #[test]
    fn a_grandchild_holding_the_pipe_does_not_hang_the_call() {
        let start = Instant::now();
        let mut cmd = std::process::Command::new("sh");
        cmd.args(["-c", "sleep 5 & sleep 30"]);
        let got = run_bounded(&mut cmd, Duration::from_secs(1));
        let took = start.elapsed();
        assert_eq!(got, Fetch::Failed);
        assert!(took < Duration::from_secs(3), "took {took:?}");
    }

    #[test]
    fn a_missing_program_is_not_spawned() {
        let mut cmd = std::process::Command::new("uor-harness-no-such-program-xyz");
        assert_eq!(
            run_bounded(&mut cmd, Duration::from_secs(1)),
            Fetch::NotSpawned
        );
    }

    /// F6: ensure_state never overwrites an existing marker, even one written after a check.
    #[test]
    fn ensure_state_does_not_overwrite_a_marker() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = paths(&dir);
        let mut m = Markers::new("claude");
        m.seen.insert(2029, 900);
        assert!(m.save(&p));
        ensure_state(&p, "claude");
        assert_eq!(Markers::load(&p, "claude").seen.get(&2029), Some(&900));
    }
}
