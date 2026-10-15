//! Presence and the check-in card — the path labs exercise most.
//!
//! Every lab runs `heartbeat` at least every 90 minutes; it is the command that publishes presence,
//! writes a receipt and produces the status card. Porting it is the next step toward removing Python
//! from the harness entirely, and it exercises both writers at once: the committed record and the
//! live tier.
//!
//! **Two tiers, and the difference matters.**
//!
//! * **live** (`~/.local/share/uor-r4/harness/presence/<lab>.json`) is what peers see now.
//!   Uncommitted, atomic via rename, cleared as a matter of course — advisory, never history.
//! * **durable** (`docs/harness/presence/<lab>.json`) is the record. It survives restarts and
//!   reaches `git`, which is why it is the one a peer actually relies on when the live tier is gone.
//!
//! A heartbeat writes both from one record so they cannot disagree about what a lab is doing. A
//! failure to write the live tier must **not** fail the check-in: it is an optimisation, the record
//! is the commitment.

use std::fs;
use std::path::Path;

use serde_json::{Map, Value};

use crate::state::{slug, StatePaths};

/// A presence record older than this reads STALE, never live.
/// How long a check-in stays current.
///
/// **Five minutes**, on the owner's direction (13 October): every lab checks in at least this often
/// so every other lab can see its status, and the board calls a lab STALE when it goes quiet for
/// longer. Thirty minutes was too coarse - it made a lab that stopped checking in look present while
/// three other labs had moved on.
pub const PRESENCE_TTL_MINUTES: i64 = 5;

/// The per-lab check-in.
#[derive(Debug, Clone)]
pub struct Presence {
    pub lab: String,
    pub updated: String,
    pub milestone: String,
    pub branch: String,
    pub worktree: String,
    pub doing: String,
    pub next: String,
    pub host: String,
    pub session: String,
}

impl Presence {
    /// Build a check-in, reading the current branch and host from the environment.
    ///
    /// The branch is read rather than required: a lab checking in should not have to say where it
    /// is, and a wrong branch is worse than an empty one because peers would act on it.
    pub fn new(paths: &StatePaths, lab: &str, milestone: &str) -> Self {
        Presence {
            // The RAW name is kept here so `validate` can still refuse a check-in that names nobody:
            // `slug` now falls back to `unknown` for FILE NAMES, and a record must not be silently
            // attributed to that fallback. The record and every path carry the slug.
            lab: lab.to_string(),
            updated: crate::receipt::now_rfc3339(),
            milestone: milestone.to_string(),
            branch: current_branch(&paths.repo),
            worktree: paths.repo.display().to_string(),
            doing: String::new(),
            next: String::new(),
            host: hostname(),
            session: String::new(),
        }
    }

    pub fn with_doing(mut self, doing: impl Into<String>) -> Self {
        self.doing = doing.into();
        self
    }

    pub fn with_next(mut self, next: impl Into<String>) -> Self {
        self.next = next.into();
        self
    }

    /// The record as JSON, in the shape the frozen contract describes.
    ///
    /// `source` is written as an empty string: it is a *read-time* label saying which tier answered,
    /// never something a writer asserts about itself. Writing "live" into the record would be the
    /// writer claiming how it will be read.
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("branch".into(), Value::String(self.branch.clone()));
        m.insert("doing".into(), Value::String(self.doing.clone()));
        m.insert("host".into(), Value::String(self.host.clone()));
        m.insert("lab".into(), Value::String(slug(&self.lab)));
        m.insert("milestone".into(), Value::String(self.milestone.clone()));
        m.insert("next".into(), Value::String(self.next.clone()));
        m.insert("schema".into(), Value::from(1));
        m.insert("session".into(), Value::String(self.session.clone()));
        m.insert("source".into(), Value::String(String::new()));
        m.insert("updated".into(), Value::String(self.updated.clone()));
        m.insert("worktree".into(), Value::String(self.worktree.clone()));
        Value::Object(m)
    }

    pub fn line(&self) -> String {
        self.to_json().to_string()
    }

    pub fn validate(&self) -> Result<(), String> {
        // `slug_stem`, not `slug`: the fallback is a file name, and a check-in attributed to nobody
        // is exactly what Python's `write_presence` refuses.
        if crate::state::slug_stem(&self.lab).is_empty() {
            return Err("presence needs a lab; without one it is invisible to every peer".into());
        }
        if crate::state::parse_iso_unix(&self.updated).is_none() {
            return Err(format!("{:?} is not a readable timestamp", self.updated));
        }
        Ok(())
    }
}

/// What a check-in records for one field: the flag when it was given, else what the lab last said.
///
/// **The rule is "absent", not "empty".** `heartbeat -m m1` with no `--line` used to rewrite
/// presence with `doing` and `next` empty, so **a lab that checked in with the documented command
/// read as live-and-idle on every board - including its own last intent, which its previous file had
/// held one moment earlier.** Reported by claude/review1 with the repro on #2028 (6163960613).
///
/// A flag passed with an EMPTY value is a lab clearing the field on purpose, and that must still
/// work: this function is only reached when the caller saw no flag at all.
pub fn remembered(flag: Option<String>, previous: Option<&str>) -> String {
    match flag {
        Some(v) => v,
        None => previous.unwrap_or("").to_string(),
    }
}

/// **The two tiers are written in two different shapes, and Python is why.**
///
/// The live tier is `json.dumps(record, ensure_ascii=False, sort_keys=True)` — compact with
/// Python's default separators, `{"a": 1}`. The committed tier is `core._save`'s
/// `json.dumps(record, indent=2, ensure_ascii=False, sort_keys=True)` — pretty, because **it is
/// read in a git diff**: `sync_live_to_durable` writing compact JSON over a file Python wrote
/// pretty would show every line of every lab's row as changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    Live,
    Durable,
}

impl Style {
    fn render(self, presence: &Presence) -> String {
        let json = presence.to_json();
        match self {
            Style::Live => crate::receipt::py_dumps_sorted(&json),
            Style::Durable => serde_json::to_string_pretty(&json).unwrap_or_default(),
        }
    }
}

/// Write one lab's presence to the live tier, atomically.
pub fn publish_live(paths: &StatePaths, presence: &Presence) -> Result<(), String> {
    publish(&paths.live_presence(&presence.lab), presence, Style::Live)
}

/// Write one lab's presence to the committed record.
///
/// **A checkout write, and the only presence write that is.** The check-in itself publishes to the
/// live tier; this is the deliberate "put me in the record" step, so it lands in the caller's own
/// worktree and R10 refuses the shared checkout.
pub fn publish_durable(paths: &StatePaths, presence: &Presence) -> Result<(), String> {
    crate::state::refuse_shared_checkout(paths, "presence")?;
    publish(
        &paths.durable_presence(&presence.lab),
        presence,
        Style::Durable,
    )
}

/// Write a presence file via a temporary and a rename, so a reader never sees a partial file.
fn publish(target: &Path, presence: &Presence, style: Style) -> Result<(), String> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let tmp = target.with_extension("tmp");
    fs::write(&tmp, format!("{}\n", style.render(presence)))
        .map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, target).map_err(|e| format!("cannot replace {}: {e}", target.display()))?;
    Ok(())
}

/// Record a check-in to both tiers. Returns the paths written and whether the live tier succeeded.
///
/// The live tier is best-effort: a read-only or absent shared directory must not fail a check-in,
/// because the committed record is what peers depend on and the live tier is an optimisation.
pub fn heartbeat(
    paths: &StatePaths,
    presence: &Presence,
) -> Result<(bool, std::path::PathBuf), String> {
    presence.validate()?;
    // LIVE TIER ONLY. A check-in every few minutes must never dirty a working checkout: the live
    // directory is shared by every lab on this machine, so a write there is immediately visible to
    // all of them, and the owner checkout stays clean (R10).
    //
    // The durable presence file is no longer written. It was a second copy of a record that is
    // already durable elsewhere - the ledger shard, which rides the delivery PR - so writing it
    // bought nothing and cost a checkout modification on every check-in.
    let live = paths.live_presence(&presence.lab);
    publish_live(paths, presence)?;
    Ok((true, live))
}

/// The current git branch, or an empty string when it cannot be read.
fn current_branch(repo: &Path) -> String {
    std::process::Command::new("git")
        .args([
            "-C",
            &repo.display().to_string(),
            "branch",
            "--show-current",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .or_else(|| {
            std::process::Command::new("hostname")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// **The defect, as a test.** A bare `heartbeat -m m1` blanked `doing`/`next`, so a lab read as
    /// live-and-idle while its previous file still held the intent one moment earlier.
    #[test]
    fn a_missing_flag_carries_forward_and_an_empty_one_clears() {
        assert_eq!(
            remembered(None, Some("shipping the port")),
            "shipping the port"
        );
        assert_eq!(remembered(None, Some("next step")), "next step");
        // A lab that clears its next step on purpose still can.
        assert_eq!(remembered(Some(String::new()), Some("old")), "");
        // A first check-in has nothing to carry, and that is not an error.
        assert_eq!(remembered(None, None), "");
        // A given flag always wins, carried value or not.
        assert_eq!(remembered(Some("new".into()), Some("old")), "new");
    }

    fn tmp_paths(tag: &str) -> (StatePaths, PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("harness-presence-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("docs/harness")).expect("scratch");
        let live = dir.join("live");
        (
            StatePaths::resolve(&dir, None, Some(live.to_str().unwrap())),
            dir,
        )
    }

    #[test]
    fn a_heartbeat_writes_both_tiers() {
        let (paths, dir) = tmp_paths("both");
        let p = Presence::new(&paths, "kimi", "m1").with_doing("work");
        let (live_ok, durable) = heartbeat(&paths, &p).expect("heartbeat");
        assert!(live_ok, "the live tier should be writable in a scratch dir");
        assert!(durable.is_file(), "the committed record must be written");
        assert!(
            paths.live_presence("kimi").is_file(),
            "the live tier must be written"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    /// The committed record is the commitment; the live tier is an optimisation. A read-only or
    /// missing shared directory must never fail a lab's check-in.
    #[test]
    fn an_unwritable_live_tier_is_an_error_not_a_silent_success() {
        // The live tier is now the ONLY write path. When it cannot be written there is no record at
        // all, so the check-in must FAIL rather than report success - a check-in that says "checked
        // in" while recording nothing is worse than one that refuses, because the lab believes it is
        // visible when no other lab can see it.
        //
        // This replaced a test asserting the opposite, from when a durable tier still existed as a
        // fallback. Removing that fallback is what makes this an error.
        let (_, dir) = tmp_paths("readonly");
        // Point the live tier at a path that cannot be created: a file where a directory must be.
        let blocked = dir.join("blocked");
        fs::write(&blocked, "not a directory").expect("scratch file");
        let paths = StatePaths::resolve(&dir, None, Some(blocked.to_str().unwrap()));
        let p = Presence::new(&paths, "kimi", "m1");
        let err =
            heartbeat(&paths, &p).expect_err("an unwritable live tier must not report success");
        assert!(
            err.contains("presence") || err.contains("blocked"),
            "the error must name the path that failed: {err}"
        );
    }

    #[test]
    fn a_presence_record_carries_every_key_the_contract_names() {
        let (paths, dir) = tmp_paths("keys");
        let p = Presence::new(&paths, "codex", "m2");
        let v = p.to_json();
        for key in [
            "branch",
            "doing",
            "host",
            "lab",
            "milestone",
            "next",
            "schema",
            "session",
            "source",
            "updated",
            "worktree",
        ] {
            assert!(v.get(key).is_some(), "missing {key}");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    /// `source` says which TIER answered a read. A writer asserting it would be claiming how it
    /// will be read, which is not the writer's to say.
    #[test]
    fn the_writer_never_claims_which_tier_it_is() {
        let (paths, dir) = tmp_paths("source");
        let p = Presence::new(&paths, "kimi", "m1");
        assert_eq!(p.to_json()["source"], Value::String(String::new()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_timestamp_is_readable_by_both_implementations() {
        let (paths, dir) = tmp_paths("ts");
        let p = Presence::new(&paths, "kimi", "m1");
        assert!(
            crate::state::parse_iso_unix(&p.updated).is_some(),
            "an unreadable timestamp reads as STALE, so the writer must never produce one: {}",
            p.updated
        );
        assert_eq!(p.validate(), Ok(()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn presence_without_a_lab_is_rejected() {
        let (paths, dir) = tmp_paths("nolab");
        let mut p = Presence::new(&paths, "kimi", "m1");
        p.lab = String::new();
        assert!(
            p.validate().is_err(),
            "an unattributed check-in is invisible to peers"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    /// A reader must never see a half-written file, so the write goes through a rename.
    #[test]
    fn publishing_leaves_no_temporary_file_behind() {
        let (paths, dir) = tmp_paths("atomic");
        let p = Presence::new(&paths, "kimi", "m1");
        heartbeat(&paths, &p).expect("heartbeat");
        let leftovers: Vec<_> = fs::read_dir(paths.live_presence_dir())
            .expect("dir")
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "temporary files must be renamed away: {leftovers:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
