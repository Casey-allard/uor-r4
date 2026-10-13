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

    /// An ISO timestamp `hours` in the past, in the same format `Presence` writes.
    fn old_iso(hours: i64) -> String {
        let now = crate::state::parse_iso_unix(&crate::team::now_iso()).expect("now parses");
        let then = now - hours * 3600;
        // Reuse `date` so the offset form matches what the rest of the harness writes.
        let out = Command::new("date")
            .args(["-r", &then.to_string(), "+%Y-%m-%dT%H:%M:%S %z"])
            .output()
            .expect("date");
        let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let (stamp, offset) = raw.split_once(' ').expect("date output");
        if offset.len() == 5 {
            format!("{stamp}{}:{}", &offset[..3], &offset[3..])
        } else {
            format!("{stamp}{offset}")
        }
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
