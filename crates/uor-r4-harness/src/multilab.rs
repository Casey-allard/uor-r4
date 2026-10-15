//! `multilab` — the shared board: presence, claims, touch (D30 port of the Python `multilab`).
//!
//! # Why `touch` is not just "write presence"
//!
//! `touch` exists **for hooks**: a tool that wants to say "this lab is still alive" without knowing or
//! caring what the lab is doing. The Python version's contract, captured in
//! `tests/fixtures/multilab/behaviour.json`, is that it **refreshes the timestamp and preserves
//! `doing`, `milestone` and `next`**. Blanking them would make a hook-driven lab look idle — the
//! board would show a working lab as having no task, which is worse than showing it stale, because
//! stale is at least true.
//!
//! # Which tier answers
//!
//! Live first (`~/.local/share/uor-r4/harness/presence/<lab>.json`), committed as the fallback
//! (`docs/harness/presence/<lab>.json`). The entry says which answered, because a reader deciding
//! whether a peer is still working needs to know whether it is reading real time or a file that
//! landed at the last merge.

use std::path::{Path, PathBuf};

use crate::presence::{Presence, PRESENCE_TTL_MINUTES};
use crate::state::{PresenceRow, StatePaths};
use crate::team;

/// The live presence directory.
pub fn live_dir(paths: &StatePaths) -> PathBuf {
    paths.live.join("presence")
}

/// The committed presence directory.
pub fn durable_dir(paths: &StatePaths) -> PathBuf {
    paths.repo.join("docs").join("harness").join("presence")
}

/// Every lab with a presence file in either tier.
///
/// A `BTreeSet` so the board is ordered deterministically — a board that reorders between calls is
/// hard to read and impossible to diff.
pub fn known_labs(paths: &StatePaths) -> Vec<String> {
    let mut labs: Vec<String> = Vec::new();
    for dir in [live_dir(paths), durable_dir(paths)] {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(lab) = name.strip_suffix(".json") {
                if !lab.is_empty() && !labs.iter().any(|l| l == lab) {
                    labs.push(lab.to_string());
                }
            }
        }
    }
    labs.sort();
    labs
}

/// Refresh a lab's liveness **without changing what it is doing**.
///
/// Preserves `doing`, `milestone` and `next` from whatever the lab last published, in either tier.
/// Writes to the **live tier only** — a hook must not dirty a checkout (R10).
pub fn touch(paths: &StatePaths, lab: &str) -> Result<Presence, String> {
    if lab.trim().is_empty() {
        return Err("which lab? a touch with no owner would publish as nobody".into());
    }
    // Carry forward what the lab was doing. Read from the merged view, so a hook on a machine that
    // has only the committed copy still preserves the task rather than blanking it.
    let previous = read_row(paths, lab);
    let milestone = previous
        .as_ref()
        .map(|r| r.milestone.as_str())
        .unwrap_or("");
    let mut p = Presence::new(paths, lab, milestone);
    if let Some(prev) = &previous {
        p.doing = prev.doing.clone();
        p.next = prev.next.clone();
    }
    crate::presence::publish_live(paths, &p)?;
    Ok(p)
}

/// The row for a lab, live first and committed as the fallback.
pub fn read_row(paths: &StatePaths, lab: &str) -> Option<PresenceRow> {
    PresenceRow::read_lab(paths, lab)
}

/// A board entry with the tier named and the state derived.
#[derive(Debug, Clone)]
pub struct BoardEntry {
    pub row: PresenceRow,
    /// `live` or `committed` — which tier answered.
    pub source: String,
    pub age_minutes: i64,
    pub stale: bool,
}

impl BoardEntry {
    /// `live (0m ago)` or `STALE (22m ago)`.
    ///
    /// Derived at read time and never stored: a stored state is wrong the moment it is written, and
    /// the whole point of the board is that it is current.
    pub fn state(&self) -> String {
        let label = if self.stale { "STALE" } else { "live" };
        if self.age_minutes < 0 {
            // `PresenceRow` could not read the timestamp; saying "live" would be a claim the file does
            // not support.
            return format!("{label} (age unknown)");
        }
        format!("{label} ({}m ago)", self.age_minutes)
    }
}

/// The board, stalest first so a lab that has gone quiet is at the top rather than buried.
pub fn board(paths: &StatePaths, now: i64) -> Vec<BoardEntry> {
    let mut out: Vec<BoardEntry> = known_labs(paths)
        .into_iter()
        .filter_map(|lab| {
            let row = read_row(paths, &lab)?;
            // Delegated to `PresenceRow`, which already answers "cannot tell" as stale. An
            // unparseable timestamp is not evidence of freshness, and reporting 0 minutes would make
            // a corrupt file look like the most current lab on the board.
            let age = row.age_minutes(now);
            let stale = row.is_stale(now, PRESENCE_TTL_MINUTES);
            Some(BoardEntry {
                stale,
                // The reader names the tiers `live` and `durable`; the BOARD says `committed`, which is
                // what `multilab presence` printed and what a lab reading the board has seen.
                // Normalising here rather than renaming the reader keeps one vocabulary per surface:
                // the tiers are the storage story, and `committed` is the board's answer to "is this
                // real time or from the last merge".
                source: if row.source == "durable" {
                    "committed".into()
                } else {
                    row.source.clone()
                },
                age_minutes: age.unwrap_or(-1),
                row,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        b.stale
            .cmp(&a.stale)
            .then_with(|| a.row.lab.cmp(&b.row.lab))
    });
    out
}

/// The refusal text for a missing lab, in one place so the CLI and tests cannot drift.
pub fn no_lab_message() -> &'static str {
    "which lab? pass --lab or set UOR_LAB"
}

/// Claims, as `multilab claims` shows them: committed branch claims visible to every lab.
///
/// Delegates to the `team` port rather than re-reading the store, because **two readers of one file
/// will eventually disagree** — that is the class of defect this whole line keeps producing.
pub fn claims(paths: &StatePaths) -> Vec<(String, team::Claim)> {
    team::load_claims(paths).into_iter().collect()
}

/// The nine-dash separator the human board uses.
pub fn separator() -> String {
    "-".repeat(104)
}

/// Is this lab name one the board knows, for a clearer error than "no presence"?
pub fn is_known(paths: &StatePaths, lab: &str) -> bool {
    known_labs(paths).iter().any(|l| l == lab)
}

/// The repo a lab is working in, for a message that says where to look.
pub fn repo_of(paths: &StatePaths) -> &Path {
    &paths.repo
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// An ISO timestamp `hours` in the past, formatted WITHOUT shelling out to `date`.
    ///
    /// **This was `date -r <epoch>` and it broke CI.** `-r` is BSD syntax meaning "format this
    /// epoch"; on GNU/Linux it is `--reference=FILE`, so the test asked `date` to stat a *file* named
    /// `1750000000` and the split panicked. It passed on macOS and failed on the Ubuntu runner.
    ///
    /// **That fix did not land the first time.** It was a second commit on the port-2 branch, and the
    /// branch was deleted after the merge without confirming the commit was in the merged tree, so
    /// `main` kept the broken helper and CI kept failing. It is here now, in a commit whose file set
    /// is checked against `main` rather than assumed.
    ///
    /// The test is about staleness arithmetic, so it depends on no `date` dialect at all: the
    /// civil-from-days conversion is done here. The arithmetic was checked against `date -u -r` as an
    /// oracle at seven offsets (0, 1h, 2h, 1d, 400y, 12345s, 3*365d) and all seven match.
    fn old_iso(hours: i64) -> String {
        let now = crate::state::parse_iso_unix(&crate::team::now_iso()).expect("now parses");
        let then = now - hours * 3600;
        let days = then.div_euclid(86_400);
        let secs = then.rem_euclid(86_400);
        let (h, mi, sec) = (secs / 3600, (secs % 3600) / 60, secs % 60);
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{sec:02}Z")
    }

    struct Fixture {
        dir: tempfile::TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            let dir = tempfile::tempdir().expect("tempdir");
            let repo = dir.path();
            for args in [
                vec!["init", "-q", "-b", "main"],
                vec!["config", "user.email", "t@t"],
                vec!["config", "user.name", "t"],
            ] {
                Command::new("git")
                    .args(&args)
                    .current_dir(repo)
                    .output()
                    .expect("git");
            }
            std::fs::create_dir_all(repo.join("docs/harness/presence")).expect("mkdir");
            std::fs::create_dir_all(repo.join("docs/harness/ledger")).expect("mkdir");
            std::fs::write(repo.join("docs/harness/claims.json"), "{}\n").expect("write");
            std::fs::write(repo.join("f"), "1").expect("write");
            Command::new("git")
                .args(["add", "-A"])
                .current_dir(repo)
                .output()
                .expect("git");
            Command::new("git")
                .args(["commit", "-qm", "init"])
                .current_dir(repo)
                .output()
                .expect("git");
            Fixture { dir }
        }

        fn paths(&self) -> StatePaths {
            let live = self.dir.path().join("live");
            StatePaths::resolve(self.dir.path(), None, Some(live.to_str().expect("utf8")))
        }
    }

    /// **The contract that matters most.** A hook refreshes liveness; it must not erase the task.
    /// Blanking `doing` would show a working lab as having nothing to do, which is worse than showing
    /// it stale — stale is at least true.
    #[test]
    fn touch_preserves_what_the_lab_is_doing() {
        let f = Fixture::new();
        let p = f.paths();
        let mut first = Presence::new(&p, "deepseek", "m1");
        first.doing = "porting multilab".into();
        first.next = "then git clean-merged".into();
        crate::presence::publish_live(&p, &first).expect("publish");

        let touched = touch(&p, "deepseek").expect("touch");
        assert_eq!(
            touched.doing, "porting multilab",
            "a hook must not blank the task"
        );
        assert_eq!(touched.next, "then git clean-merged");
        assert_eq!(touched.milestone, "m1");
    }

    #[test]
    fn touch_creates_an_entry_for_a_lab_that_never_checked_in() {
        let f = Fixture::new();
        let p = f.paths();
        let touched = touch(&p, "kimi").expect("touch");
        assert_eq!(touched.lab, "kimi");
        assert_eq!(touched.doing, "");
        // `touch` writes to the live tier, and the row read back says so. `Presence` itself has no
        // `source` field - the tier is recorded by `PresenceRow` on READ, not by the writer.
        let row = read_row(&p, "kimi").expect("kimi row");
        assert_eq!(row.source, "live");
        assert!(live_dir(&p).join("kimi.json").is_file());
    }

    /// R10: a hook must not dirty a checkout.
    #[test]
    fn touch_writes_the_live_tier_only() {
        let f = Fixture::new();
        let p = f.paths();
        touch(&p, "claude").expect("touch");
        assert!(
            live_dir(&p).join("claude.json").is_file(),
            "live tier written"
        );
        assert!(
            !durable_dir(&p).join("claude.json").is_file(),
            "a hook must not write a checkout - R10"
        );
    }

    #[test]
    fn touch_without_a_lab_is_an_error() {
        let f = Fixture::new();
        assert!(touch(&f.paths(), "  ").is_err());
    }

    #[test]
    fn an_empty_board_is_empty_not_an_error() {
        let f = Fixture::new();
        assert!(known_labs(&f.paths()).is_empty());
        assert!(board(&f.paths(), 0).is_empty());
    }

    /// The committed tier is the fallback, and the entry says so — a reader deciding whether a peer
    /// is working needs to know whether it is reading real time or a file from the last merge.
    #[test]
    fn the_committed_tier_is_the_fallback_and_is_labelled() {
        let f = Fixture::new();
        let p = f.paths();
        let mut row = Presence::new(&p, "codex", "m1");
        row.doing = "reviewing".into();
        crate::presence::publish_durable(&p, &row).expect("durable");

        let b = board(&p, 1_700_000_000);
        assert_eq!(
            b.len(),
            1,
            "a lab with only a committed file is still shown"
        );
        assert_eq!(b[0].source, "committed");
        assert_eq!(b[0].row.lab, "codex");
    }

    /// Staleness is derived at read time and never stored: a stored state is wrong the moment it is
    /// written.
    #[test]
    fn state_is_derived_from_age_not_stored() {
        let f = Fixture::new();
        let p = f.paths();
        // `updated` is written by `Presence::new` as ISO-now; the board test needs a lab that is
        // definitively old, so its timestamp is set from the same parser's inverse by writing a
        // two-hour-old ISO string directly.
        let fresh = Presence::new(&p, "fresh", "m1");
        crate::presence::publish_live(&p, &fresh).expect("publish");
        let mut old = Presence::new(&p, "oldlab", "m1");
        old.updated = old_iso(2);
        crate::presence::publish_live(&p, &old).expect("publish");
        let now = crate::state::parse_iso_unix(&crate::team::now_iso()).expect("now parses");
        let b = board(&p, now);
        let fresh = b.iter().find(|e| e.row.lab == "fresh").expect("fresh");
        let old = b.iter().find(|e| e.row.lab == "oldlab").expect("old");
        assert!(!fresh.stale, "{}", fresh.state());
        assert!(fresh.state().starts_with("live"), "{}", fresh.state());
        assert!(old.stale, "{}", old.state());
        assert!(old.state().starts_with("STALE"), "{}", old.state());
        // Stalest first, so a lab that has gone quiet is at the top rather than buried.
        assert_eq!(b[0].row.lab, "oldlab");
    }

    /// A corrupt timestamp is not evidence of freshness.
    #[test]
    fn an_unparseable_timestamp_is_not_fresh() {
        let f = Fixture::new();
        let p = f.paths();
        std::fs::create_dir_all(live_dir(&p)).expect("mkdir");
        std::fs::write(
            live_dir(&p).join("broken.json"),
            r#"{"lab":"broken","updated":"not a timestamp","source":"live"}"#,
        )
        .expect("write");
        let now = crate::state::parse_iso_unix(&crate::team::now_iso()).expect("now parses");
        let b = board(&p, now);
        assert_eq!(b.len(), 1);
        assert!(
            b[0].stale,
            "a corrupt file must not look like the most current lab"
        );
    }

    #[test]
    fn the_board_is_ordered_deterministically() {
        let f = Fixture::new();
        let p = f.paths();
        for lab in ["zeta", "alpha", "mid"] {
            touch(&p, lab).expect("touch");
        }
        let a: Vec<String> = board(&p, 0).into_iter().map(|e| e.row.lab).collect();
        let b: Vec<String> = board(&p, 0).into_iter().map(|e| e.row.lab).collect();
        assert_eq!(
            a, b,
            "a board that reorders between calls is impossible to diff"
        );
        assert_eq!(a, vec!["alpha", "mid", "zeta"]);
    }

    /// Claims delegate to the `team` reader. Two readers of one file will eventually disagree, and
    /// that is the defect class this line keeps producing.
    #[test]
    fn claims_are_read_through_the_team_port() {
        let f = Fixture::new();
        let p = f.paths();
        team::claim_branch(&p, "probe/one", "claude", "").expect("claim");
        let claims = claims(&p);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].0, "probe/one");
        assert_eq!(claims[0].1.lab, "claude");
    }
}

// ------------------------------------------------------------------ sync-live, migrate-ledger

/// `sync_live_to_durable`: every live presence row copied into the committed tier.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncLive {
    /// The labs copied, in the order the live files were read.
    pub synced: Vec<String>,
    /// **The live ROOT, not the presence directory.** That is what Python reports and what a reader
    /// would go and look in.
    pub live_dir: String,
}

/// `migrate_ledger`: unsharded `<YYYY-MM>.jsonl` files moved into per-lab shards.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LedgerMigration {
    /// `"<lab>-<month>" -> rows`, **in the order the shards were first written**, because that is the
    /// order Python's `--json` prints and the text form sorts.
    pub moved: Vec<(String, usize)>,
    pub total: usize,
}

/// Unsharded ledgers from before sharding: `<YYYY-MM>.jsonl`, with no lab prefix.
pub fn legacy_ledgers(paths: &StatePaths) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(paths.ledger_dir()) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .map(|n| is_legacy_ledger(&n.to_string_lossy()))
                .unwrap_or(false)
        })
        .collect();
    out.sort();
    out
}

/// `^\d{4}-\d{2}\.jsonl$`, by the same rule Python matches the file NAME with.
fn is_legacy_ledger(name: &str) -> bool {
    let b = name.as_bytes();
    // `2026-10.jsonl` is THIRTEEN bytes: 4 + 1 + 2 + 6. The first version of this said eleven and
    // matched nothing, which the comparison against Python caught immediately - the migration did
    // nothing at all and reported zero rows moved.
    b.len() == 13
        && b[4] == b'-'
        && name.ends_with(".jsonl")
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
}

/// A presence record rebuilt from a row, field for field.
///
/// **Not `Presence::new`**: that stamps a new timestamp and reads the current branch, and the whole
/// point of the sync is to carry the RECORDED one. The two tiers must agree about when a lab checked
/// in and what it said, or the committed copy is a second, quieter opinion.
fn presence_from_row(row: &PresenceRow) -> Presence {
    Presence {
        lab: row.lab.clone(),
        updated: row.updated.clone(),
        milestone: row.milestone.clone(),
        branch: row.branch.clone(),
        worktree: row.worktree.clone(),
        doing: row.doing.clone(),
        next: row.next.clone(),
        host: row.host.clone(),
        session: row.session.clone(),
    }
}

/// `sync_live_to_durable`: copy the live tier into the committed record.
///
/// **This is the durable tier's only refresh path, and it is deliberately explicit.** The heartbeat
/// writes the live tier only, because a check-in every few minutes must never dirty a checkout (R10);
/// a lab that wants the committed record to show it runs this. That is why the committed presence
/// files can be a day stale without anything being broken — and why the README must say so.
pub fn sync_live_to_durable(paths: &StatePaths) -> Result<SyncLive, String> {
    // R10 FIRST, before the live tier is even read. The per-row guard in `publish_durable` only fires
    // when there IS a live row to publish, so an empty live tier made this command report success from
    // the shared checkout without ever consulting the rule (found by the entry-point pin test): a no-op
    // sync from the shared checkout is still the wrong checkout to have run it in.
    crate::state::refuse_shared_checkout(paths, "presence")?;
    let dir = live_dir(paths);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|e| e.flatten().map(|x| x.path()).collect())
        .unwrap_or_default();
    files.sort();
    files.retain(|p| p.extension().and_then(|e| e.to_str()) == Some("json"));
    let mut synced = Vec::new();
    for path in files {
        // An unreadable or empty file is skipped rather than fatal: Python's `all_live` drops it,
        // and one lab's broken presence file must not stop every other lab's from reaching the
        // committed record.
        let Some(row) = PresenceRow::read(&path, "live") else {
            continue;
        };
        crate::presence::publish_durable(paths, &presence_from_row(&row))?;
        synced.push(row.lab.clone());
    }
    Ok(SyncLive {
        synced,
        live_dir: paths.live.display().to_string(),
    })
}

/// `migrate_ledger`: move unsharded receipts into per-lab shards, preserving every line and its order.
///
/// **A line that cannot be parsed is quarantined, never dropped.** Python buckets it under
/// `unreadable` and keeps the original text, so a migration cannot silently lose the one row that
/// would have explained what went wrong.
pub fn migrate_ledger(paths: &StatePaths) -> Result<LedgerMigration, String> {
    crate::state::refuse_shared_checkout(paths, "the migrated ledger shards")?;
    let mut moved: Vec<(String, usize)> = Vec::new();
    for file in legacy_ledgers(paths) {
        let month = file
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let text = std::fs::read_to_string(&file)
            .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
        // Insertion order, because that is the order `moved` is printed in by `--json`.
        let mut buckets: Vec<(String, Vec<String>)> = Vec::new();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let (lab, rendered) = match serde_json::from_str::<serde_json::Value>(line) {
                Ok(mut rec) => {
                    let raw = rec
                        .get("lab")
                        .and_then(serde_json::Value::as_str)
                        .filter(|s| !s.is_empty())
                        .or_else(|| {
                            rec.get("actor")
                                .and_then(serde_json::Value::as_str)
                                .filter(|s| !s.is_empty())
                        })
                        .unwrap_or("unknown");
                    let lab = crate::state::slug(raw);
                    // `shapecase` the lab so a shard name and the row agree.
                    if rec.get("lab").and_then(serde_json::Value::as_str) != Some(lab.as_str()) {
                        if let serde_json::Value::Object(map) = &mut rec {
                            map.insert("lab".to_string(), serde_json::Value::String(lab.clone()));
                        }
                    }
                    (lab, crate::receipt::py_dumps_sorted(&rec))
                }
                Err(_) => ("unreadable".to_string(), line.to_string()),
            };
            match buckets.iter_mut().find(|(l, _)| *l == lab) {
                Some((_, rows)) => rows.push(rendered),
                None => buckets.push((lab, vec![rendered])),
            }
        }
        for (lab, rows) in buckets {
            // **Not `receipt::shard_path`**, which maps an empty lab to `unknown`: Python builds the
            // name here from the already-slugged lab, so a lab that slugs to nothing lands in
            // `-<month>.jsonl`. That file name is ugly and it is what the reader expects.
            let target = paths.ledger_dir().join(format!("{lab}-{month}.jsonl"));
            let mut merged: Vec<String> = std::fs::read_to_string(&target)
                .map(|t| {
                    t.lines()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| l.to_string())
                        .collect()
                })
                .unwrap_or_default();
            let count = rows.len();
            merged.extend(rows);
            // `sort_by_key` is stable, so rows with no readable `ts` keep the order they arrived in
            // — which is how Python's `sorted` behaves.
            merged.sort_by_key(|l| ts_of(l));
            std::fs::write(&target, format!("{}\n", merged.join("\n")))
                .map_err(|e| format!("cannot write {}: {e}", target.display()))?;
            let key = format!("{lab}-{month}");
            match moved.iter_mut().find(|(k, _)| *k == key) {
                Some((_, n)) => *n += count,
                None => moved.push((key, count)),
            }
        }
        std::fs::remove_file(&file)
            .map_err(|e| format!("cannot remove {}: {e}", file.display()))?;
    }
    let total = moved.iter().map(|(_, n)| *n).sum();
    Ok(LedgerMigration { moved, total })
}

/// Python's `json.dumps` for a single string, with `ensure_ascii` on — the default for `--json`.
fn py_str(s: &str) -> String {
    crate::registries::ensure_ascii(
        &serde_json::to_string(&serde_json::Value::String(s.to_string())).unwrap_or_default(),
    )
}

impl SyncLive {
    /// `json.dumps({"synced": [...], "live_dir": "..."}, indent=2)`, key order and all.
    ///
    /// Rendered by hand because a `serde_json::Value` is a `BTreeMap`: it would sort the keys and
    /// print the list on one line, and **this output is compared byte for byte with Python's**.
    pub fn to_pretty(&self) -> String {
        let mut out = String::from("{\n");
        if self.synced.is_empty() {
            out.push_str("  \"synced\": [],\n");
        } else {
            out.push_str("  \"synced\": [\n");
            for (i, lab) in self.synced.iter().enumerate() {
                let comma = if i + 1 == self.synced.len() { "" } else { "," };
                out.push_str(&format!("    {}{comma}\n", py_str(lab)));
            }
            out.push_str("  ],\n");
        }
        out.push_str(&format!("  \"live_dir\": {}\n", py_str(&self.live_dir)));
        out.push('}');
        out
    }
}

impl LedgerMigration {
    /// `json.dumps({"moved": {...}, "total": n}, indent=2)`, in Python's insertion order.
    pub fn to_pretty(&self) -> String {
        let mut out = String::from("{\n");
        if self.moved.is_empty() {
            out.push_str("  \"moved\": {},\n");
        } else {
            out.push_str("  \"moved\": {\n");
            for (i, (shard, n)) in self.moved.iter().enumerate() {
                let comma = if i + 1 == self.moved.len() { "" } else { "," };
                out.push_str(&format!("    {}: {n}{comma}\n", py_str(shard)));
            }
            out.push_str("  },\n");
        }
        out.push_str(&format!("  \"total\": {}\n", self.total));
        out.push('}');
        out
    }
}

/// The `ts` a row sorts by, as text. An unreadable row sorts first, as Python's `""` does.
fn ts_of(line: &str) -> String {
    serde_json::from_str::<serde_json::Value>(line)
        .ok()
        .and_then(|v| {
            v.get("ts")
                .and_then(serde_json::Value::as_str)
                .map(String::from)
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod sync_tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch(tag: &str) -> (PathBuf, StatePaths) {
        let dir =
            std::env::temp_dir().join(format!("harness-multilab-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("docs").join("harness").join("ledger")).expect("scratch");
        let live = dir.join("live");
        std::fs::create_dir_all(live.join("presence")).expect("live");
        let paths = StatePaths::resolve(&dir, None, Some(live.to_str().unwrap()));
        (dir, paths)
    }

    /// `2026-10.jsonl` is THIRTEEN bytes. The first version of the matcher said eleven and matched
    /// nothing, which the comparison against Python caught as "the migration did nothing at all".
    #[test]
    fn only_unsharded_month_files_are_legacy() {
        assert!(is_legacy_ledger("2026-10.jsonl"));
        assert!(is_legacy_ledger("1999-01.jsonl"));
        assert!(!is_legacy_ledger("kimi-2026-10.jsonl"));
        assert!(!is_legacy_ledger("2026-10.json"));
        assert!(!is_legacy_ledger("notes.txt"));
        assert!(!is_legacy_ledger("20x6-10.jsonl"));
    }

    #[test]
    fn sync_live_copies_every_row_and_skips_a_broken_file() {
        let (dir, paths) = scratch("sync");
        let live = paths.live.join("presence");
        std::fs::write(
            live.join("claude.json"),
            r#"{"lab": "claude", "updated": "2026-10-14T04:00:00-04:00", "doing": "reviewing"}"#,
        )
        .expect("write");
        // A file that is not JSON is skipped rather than fatal: one lab's broken presence file must
        // not stop every other lab's from reaching the committed record.
        std::fs::write(live.join("broken.json"), "{not json").expect("write");
        let report = sync_live_to_durable(&paths).expect("sync");
        assert_eq!(report.synced, vec!["claude"]);
        // The live ROOT is what Python reports, not the presence subdirectory.
        assert_eq!(report.live_dir, paths.live.display().to_string());
        let text =
            std::fs::read_to_string(paths.durable_presence("claude")).expect("durable written");
        // The RECORDED timestamp survives: the sync must not restamp the row it copies.
        assert!(text.contains("2026-10-14T04:00:00-04:00"), "{text}");
        assert!(text.contains("reviewing"), "{text}");
        assert!(
            text.starts_with("{\n  \"branch\""),
            "pretty, as Python writes it: {text}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn migrate_ledger_buckets_by_lab_sorts_by_ts_and_quarantines() {
        let (dir, paths) = scratch("migrate");
        let ledger = paths.ledger_dir();
        std::fs::write(
            ledger.join("2026-10.jsonl"),
            concat!(
                "{\"ts\": \"2026-10-05T00:00:00Z\", \"lab\": \"Kimi\", \"kind\": \"heartbeat\"}\n",
                "{\"ts\": \"2026-10-01T00:00:00Z\", \"lab\": \"Kimi\", \"kind\": \"skill\"}\n",
                "{\"ts\": \"2026-10-03T00:00:00Z\", \"actor\": \"codex\", \"kind\": \"pr\"}\n",
                "{not json\n"
            ),
        )
        .expect("write");
        std::fs::write(
            ledger.join("kimi-2026-10.jsonl"),
            "{\"kind\": \"prep\", \"lab\": \"kimi\", \"ts\": \"2026-10-02T12:00:00Z\"}\n",
        )
        .expect("write");
        let report = migrate_ledger(&paths).expect("migrate");
        assert_eq!(report.total, 4);
        assert!(
            !ledger.join("2026-10.jsonl").exists(),
            "the legacy file is removed"
        );
        let kimi = std::fs::read_to_string(ledger.join("kimi-2026-10.jsonl")).expect("shard");
        let ts: Vec<String> = kimi.lines().map(ts_of).collect();
        let mut sorted = ts.clone();
        sorted.sort();
        assert_eq!(ts, sorted, "rows are ordered by ts: {kimi}");
        assert_eq!(kimi.lines().count(), 3, "the existing row is kept: {kimi}");
        // A lab is slugged on the way in, so the shard name and the row agree.
        let codex = std::fs::read_to_string(ledger.join("codex-2026-10.jsonl")).expect("shard");
        assert!(codex.contains("\"lab\": \"codex\""), "{codex}");
        // The line that could not be parsed is quarantined, never dropped.
        let bad = std::fs::read_to_string(ledger.join("unreadable-2026-10.jsonl")).expect("shard");
        assert_eq!(bad.trim(), "{not json");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_reports_are_pythons_json() {
        let sync = SyncLive {
            synced: vec!["claude".into(), "kimi".into()],
            live_dir: "/tmp/live".into(),
        };
        assert_eq!(
            sync.to_pretty(),
            "{\n  \"synced\": [\n    \"claude\",\n    \"kimi\"\n  ],\n  \"live_dir\": \"/tmp/live\"\n}"
        );
        assert_eq!(
            SyncLive::default().to_pretty(),
            "{\n  \"synced\": [],\n  \"live_dir\": \"\"\n}"
        );
        let migration = LedgerMigration {
            moved: vec![("kimi-2026-10".into(), 2), ("codex-2026-10".into(), 1)],
            total: 3,
        };
        assert_eq!(
            migration.to_pretty(),
            "{\n  \"moved\": {\n    \"kimi-2026-10\": 2,\n    \"codex-2026-10\": 1\n  },\n  \"total\": 3\n}"
        );
    }
}
