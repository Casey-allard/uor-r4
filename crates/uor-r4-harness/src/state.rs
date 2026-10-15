//! Typed readers for harness state, and the paths it lives at.
//!
//! Two tiers, deliberately:
//!
//! - **live** (`~/.local/share/uor-r4/harness/`, overridable by `UOR_HARNESS_LIVE`) is what peers
//!   see now. Uncommitted, no merge conflicts, and cleared as a matter of course — so it is
//!   explicitly advisory and nothing may expect history there.
//! - **durable** (`docs/harness/`) is the record. It survives restarts and reaches `git`.
//!
//! The reader prefers live and falls back to durable, and always reports which answered. That is
//! the same rule the Python reader implements; a port that quietly preferred one tier would let the
//! two implementations disagree about what a lab is doing.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Where harness state lives.
#[derive(Debug, Clone)]
pub struct StatePaths {
    /// The repository root.
    pub repo: PathBuf,
    /// The live tier root, resolved from `UOR_HARNESS_LIVE` or the default.
    pub live: PathBuf,
}

impl StatePaths {
    /// The default live directory, relative to a home directory.
    pub const LIVE_DEFAULT: &'static str = ".local/share/uor-r4/harness";

    /// Resolve state paths for a repository, honouring `UOR_HARNESS_LIVE`.
    ///
    /// `home` is passed in rather than read from the environment so a test can point it somewhere
    /// disposable — a test that writes to the real shared directory pollutes live cross-lab state,
    /// which is a mistake this project has already made once.
    pub fn resolve(
        repo: impl Into<PathBuf>,
        home: Option<&Path>,
        live_override: Option<&str>,
    ) -> Self {
        let live = match live_override {
            Some(p) if !p.is_empty() => PathBuf::from(p),
            _ => home
                .map(|h| h.join(Self::LIVE_DEFAULT))
                .unwrap_or_else(|| PathBuf::from(Self::LIVE_DEFAULT)),
        };
        Self {
            repo: repo.into(),
            live,
        }
    }

    /// The live tier root from the process environment.
    pub fn from_env(repo: impl Into<PathBuf>) -> Self {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let live = std::env::var("UOR_HARNESS_LIVE").ok();
        Self::resolve(repo, home.as_deref(), live.as_deref())
    }

    pub fn durable_presence_dir(&self) -> PathBuf {
        self.repo.join("docs").join("harness").join("presence")
    }

    pub fn live_presence_dir(&self) -> PathBuf {
        self.live.join("presence")
    }

    pub fn durable_presence(&self, lab: &str) -> PathBuf {
        self.durable_presence_dir()
            .join(format!("{}.json", slug(lab)))
    }

    pub fn live_presence(&self, lab: &str) -> PathBuf {
        self.live_presence_dir().join(format!("{}.json", slug(lab)))
    }

    pub fn ledger_dir(&self) -> PathBuf {
        self.repo.join("docs").join("harness").join("ledger")
    }

    /// The committed claims store: the CALLER'S OWN checkout, which is what a write targets.
    ///
    /// It was the main worktree once, so every claim landed in the owner checkout (R10 forbids
    /// that) and the claim still died with the worktree it was filed from. The claim now rides the
    /// delivery PR, and shared_primary_checkout() refuses the write that used to land there.
    pub fn claims_file(&self) -> PathBuf {
        self.repo.join("docs").join("harness").join("claims.json")
    }
}

/// Reduce a lab name to the form used in file names, matching the Python `slug()`.
pub const UNKNOWN_LAB: &str = "unknown";

/// Reduce a lab name to the form used in file names, matching the Python slug().
///
/// **The unknown fallback is not cosmetic, and its absence was a real defect.** Python returns
/// "unknown" when a name reduces to nothing; the Rust version returned the empty string, so an
/// unset lab wrote ".json" and "-<month>.jsonl" — files no reader looks for, holding a record that
/// looks attributed. receipt::shard_path had patched around it for the ledger alone; presence, the
/// note marker and the note cache had not.
pub fn slug(lab: &str) -> String {
    let stem = slug_stem(lab);
    if stem.is_empty() {
        UNKNOWN_LAB.to_string()
    } else {
        stem
    }
}

/// The slug with NO fallback: empty when the name carries no usable character.
///
/// Separate from slug() because "which file does this land in" and "did a lab identify itself" are
/// different questions, and one function answering both is how an unattributed record comes to look
/// attributed. Writers that must refuse an unnamed lab check this; writers that must still record
/// one use slug() and land in the unknown bucket, exactly as Python does.
pub fn slug_stem(lab: &str) -> String {
    let mut out = String::with_capacity(lab.len());
    let mut last_dash = false;
    for ch in lab.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// **R10: the shared checkout, when this repository has one — and therefore a write to refuse.**
///
/// The owner checkout (~/uor-r4) stays on clean origin/main: no lab edits, builds, commits or runs
/// mutating tools in it (docs/harness/pm-rules.md R10). That rule was repeatedly broken by the
/// harness itself — team claim, receipt, skill use, ledger-sync and the durable presence sync all
/// resolved their target from the working directory, so a lab whose shell sat in the owner checkout
/// dirtied it with a coordination write and another lab had to hand-move the entries out.
///
/// **Two ways to identify it, because the shape alone has a hole.** The primary worktree of a
/// repository that has linked worktrees is the shared checkout every lab shares. But `uor-hygiene`
/// and the post-merge cleanup rule remove those worktrees, and the checkout then looks like an
/// ordinary single-checkout clone — at which point a shape-only rule silently allows writes into the
/// owner checkout again. (Found in review: a single-checkout scratch repo accepted `team claim` and
/// left `docs/harness/claims.json` modified.) So the shape is checked, and so is a MARKER that does
/// not depend on how many worktrees exist:
///
/// ```text
/// git config uor.sharedCheckout true     # set once, in the shared checkout
/// ```
///
/// The marker is stored in the repository config, so it survives cleanup, a reset and a re-clone of
/// the working tree, and it is read only for the PRIMARY worktree — a linked worktree of a marked
/// checkout is still a lab's own and stays writable. A single-checkout clone with no marker, a
/// scratch directory and an unreadable git state all return None: refusing there would break a lab
/// that simply cloned the repository, which is the opposite of the R10 goal.
///
/// **Read-only callers are unaffected.** This is consulted by write paths only: reading claims,
/// presence and the ledger from the shared checkout is exactly what R10 permits.
pub fn shared_primary_checkout(repo: &Path) -> Option<PathBuf> {
    // The working directory may be a SUBDIRECTORY of a worktree; compare roots, not the literal path.
    let top = git_out(repo, &["rev-parse", "--show-toplevel"])?;
    let top = PathBuf::from(top);
    let top = top.canonicalize().unwrap_or(top);
    let listing = git_out(repo, &["worktree", "list", "--porcelain"])?;
    let worktrees: Vec<PathBuf> = listing
        .lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .map(|rest| {
            let p = PathBuf::from(rest.trim());
            p.canonicalize().unwrap_or(p)
        })
        .collect();
    // "git worktree list" names the primary checkout FIRST.
    let primary = worktrees.first()?.clone();
    // A LINKED worktree is never the shared checkout, whatever the marker says.
    if primary != top {
        return None;
    }
    if worktrees.len() >= 2 || marked_shared(repo) {
        return Some(primary);
    }
    None
}

/// The `uor.sharedCheckout` marker: `true` when this checkout declares itself the shared one.
///
/// Read through `git config --bool --get`, which exits non-zero when the key is unset — so an
/// unreadable or unrelated key is not a declaration, and the shape rule still decides.
fn marked_shared(repo: &Path) -> bool {
    git_out(repo, &["config", "--bool", "--get", "uor.sharedCheckout"])
        .is_some_and(|v| v.trim() == "true")
}

/// Refuse a checkout write that would land in the shared checkout, or pass.
///
/// `what` names the write for the lab that has to fix it — "the claims store", "a ledger receipt",
/// "the pre-registration sheet" — and the message carries the rule and both ways out.
pub fn refuse_shared_checkout(paths: &StatePaths, what: &str) -> Result<(), String> {
    refuse_shared_checkout_at(&paths.repo, what)
}

/// The same refusal for a caller that holds a repository path and not a [`StatePaths`].
///
/// The ratchet baselines are tracked files under `scripts/`, so `ratchet --update-baseline` is a
/// checkout write like any other — and it was the one that falsified "every remaining committed
/// write refuses the shared checkout" (found in review: it wrote a baseline from a shared primary,
/// exit 0). A path-based entry point means a writer does not need the whole harness state to be
/// guarded.
pub fn refuse_shared_checkout_at(repo: &Path, what: &str) -> Result<(), String> {
    let Some(shared) = shared_primary_checkout(repo) else {
        return Ok(());
    };
    let msg = format!(
        "refusing to write {what} into the shared checkout {} (R10): the owner checkout stays on clean origin/main, and no lab edits or runs mutating tools in it. Run this from your own worktree (.worktrees/<lab>-<task>) or pass --repo <that worktree>; a coordination write that needs no checkout at all goes to the live tier.",
        shared.display()
    );
    Err(msg)
}

/// Trimmed stdout of a git command, or None when it fails or is not a repository.
///
/// **`GIT_DIR`/`GIT_WORK_TREE` are cleared**, because a git HOOK exports them and they take
/// precedence over `-C`: inside a pre-commit hook run from a linked worktree, `rev-parse
/// --show-toplevel` would report the MAIN worktree, so a lab's own checkout would look like the
/// shared one and its write would be refused for the wrong reason. Review finding F5.
fn git_out(repo: &Path, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// A per-lab check-in READ from disk.
///
/// Distinct from [`crate::presence::Presence`], which is the thing you WRITE — the same split as
/// [`ReceiptRow`] and [`crate::receipt::Receipt`]. A reader must tolerate every record the tiers
/// already hold, including historical ones; a writer must produce exactly what the contract
/// requires. One type serving both would be as loose as the reader and as strict as the writer.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PresenceRow {
    #[serde(default)]
    pub lab: String,
    #[serde(default)]
    pub updated: String,
    #[serde(default)]
    pub milestone: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub worktree: String,
    #[serde(default)]
    pub doing: String,
    #[serde(default)]
    pub next: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub session: String,
    #[serde(default)]
    pub schema: u32,
    /// Which tier answered this read: `live` or `durable`. Never persisted as authority.
    #[serde(default)]
    pub source: String,
}

impl PresenceRow {
    pub fn read(path: &Path, source: &str) -> Option<PresenceRow> {
        let text = std::fs::read_to_string(path).ok()?;
        let mut p: PresenceRow = serde_json::from_str(&text).ok()?;
        p.source = source.to_string();
        Some(p)
    }

    /// Read a lab's presence, preferring the live tier and falling back to the durable record.
    pub fn read_lab(paths: &StatePaths, lab: &str) -> Option<PresenceRow> {
        if let Some(p) = Self::read(&paths.live_presence(lab), "live") {
            return Some(p);
        }
        Self::read(&paths.durable_presence(lab), "durable")
    }

    /// How old this check-in is, in whole minutes, or `None` when the timestamp cannot be parsed.
    ///
    /// An unparseable timestamp must read as unknown rather than as fresh: a presence record whose
    /// time cannot be trusted is not evidence that a lab is working.
    pub fn age_minutes(&self, now_unix: i64) -> Option<i64> {
        self.age_seconds(now_unix).map(|s| s / 60)
    }

    /// The exact age in seconds. Staleness must be judged on this, not on `age_minutes`.
    pub fn age_seconds(&self, now_unix: i64) -> Option<i64> {
        let unix = parse_iso_unix(&self.updated)?;
        Some(now_unix - unix)
    }

    /// Stale means the record is older than the TTL by more than a second, or its time cannot be
    /// read at all.
    ///
    /// **Judged in seconds, never on `age_minutes`.** Truncating to whole minutes first puts a
    /// record aged 30m25s at "30 minutes" and calls it live, while the Python implementation -
    /// which keeps the fraction - calls it stale. Two implementations disagreeing about whether a
    /// lab is live is exactly the divergence the parity check exists to catch, and it caught this
    /// one against real state rather than a constructed case.
    pub fn is_stale(&self, now_unix: i64, ttl_minutes: i64) -> bool {
        match self.age_seconds(now_unix) {
            None => true,
            Some(age) => age > ttl_minutes * 60,
        }
    }
}

/// One receipt line READ from a per-lab ledger shard.
///
/// Distinct from [`crate::receipt::Receipt`], which is the thing you WRITE. Keeping them apart is
/// deliberate: a reader must tolerate every row the ledger already holds, including historical ones
/// that predate a field, while a writer must produce exactly what the contract requires. One type
/// serving both would have to be as loose as the reader and as strict as the writer.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ReceiptRow {
    #[serde(default)]
    pub lab: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub ts: String,
    #[serde(default)]
    pub schema: u32,
    /// Heartbeat receipts carry a string like `"1/3"`; team receipts carry a number. The contract
    /// records this as `mixed`, so the reader keeps it as raw JSON rather than forcing one type.
    #[serde(default)]
    pub count: Option<serde_json::Value>,
    #[serde(default)]
    pub milestone: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub skill: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

impl ReceiptRow {
    pub fn from_line(line: &str) -> Option<ReceiptRow> {
        serde_json::from_str(line).ok()
    }
}

/// Read every receipt in the sharded ledger, in file order.
pub fn read_ledger(paths: &StatePaths) -> Vec<ReceiptRow> {
    let dir = paths.ledger_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .collect();
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Some(r) = ReceiptRow::from_line(line) {
                out.push(r);
            }
        }
    }
    out
}

/// Parse an ISO-8601 timestamp with an offset to a Unix second count.
///
/// Deliberately minimal and dependency-free: the harness writes `YYYY-MM-DDTHH:MM:SS±HH:MM` or a
/// `Z` suffix. Anything else returns `None`, which callers treat as "cannot be trusted" rather
/// than guessing — guessing here would make a stale lab look live.
pub fn parse_iso_unix(ts: &str) -> Option<i64> {
    let bytes = ts.as_bytes();
    if bytes.len() < 19 {
        return None;
    }
    let num = |a: usize, b: usize| -> Option<i64> { ts.get(a..b)?.parse::<i64>().ok() };
    let (year, month, day) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (hour, minute, second) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }
    let mut offset = 0i64;
    let rest = &ts[19..];
    if !rest.is_empty() && !rest.starts_with('Z') {
        if let Some(sign) = rest.chars().next().filter(|c| *c == '+' || *c == '-') {
            let digits: String = rest[1..].chars().filter(|c| c.is_ascii_digit()).collect();
            if digits.len() >= 4 {
                let oh: i64 = digits[0..2].parse().ok()?;
                let om: i64 = digits[2..4].parse().ok()?;
                offset = (oh * 3600 + om * 60) * if sign == '-' { -1 } else { 1 };
            }
        }
    }
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second - offset)
}

/// Days since the Unix epoch for a proleptic Gregorian date. Howard Hinnant's algorithm.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_matches_the_python_form() {
        assert_eq!(slug("Kimi"), "kimi");
        assert_eq!(slug("  deep seek "), "deep-seek");
        assert_eq!(slug("a//b"), "a-b");
        // Python ends with `return s or "unknown"`. A name that reduces to nothing still names a
        // FILE, and a reader looks for that file: the empty stem wrote `.json` and `-<month>.jsonl`.
        assert_eq!(slug("---"), "unknown");
        assert_eq!(slug(""), "unknown");
        assert_eq!(slug("  !!!  "), "unknown");
        // The emptiness is still visible to a writer that must REFUSE an unnamed lab rather than
        // record it under the fallback: the two questions are answered by two functions.
        assert_eq!(slug_stem("---"), "");
        assert_eq!(slug_stem(""), "");
        assert_eq!(slug_stem("Kimi"), "kimi");
    }

    /// A primary checkout with a linked worktree - the shape R10 protects.
    fn shared_clone() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary = dir.path().join("owner");
        std::fs::create_dir_all(&primary).expect("primary");
        let run = |args: &[&str], cwd: &Path| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(cwd)
                .output()
                .expect("git")
        };
        run(&["init", "-q", "-b", "main"], &primary);
        run(&["config", "user.email", "t@t"], &primary);
        run(&["config", "user.name", "t"], &primary);
        std::fs::write(primary.join("f"), "1").expect("write");
        run(&["add", "f"], &primary);
        run(&["commit", "-qm", "one"], &primary);
        let linked = dir.path().join("lab");
        let linked_arg = linked.to_str().expect("utf8");
        run(
            &["worktree", "add", "-q", "-b", "lab/task", linked_arg],
            &primary,
        );
        (dir, primary, linked)
    }

    /// The shared checkout is identified by SHAPE, from the working directory outward.
    #[test]
    fn the_shared_checkout_is_found_and_a_linked_worktree_is_writable() {
        let (_dir, primary, linked) = shared_clone();
        // Canonicalised on both sides: on macOS a temp path is reached through /var -> /private/var,
        // and `shared_primary_checkout` compares canonical roots so a subdirectory still matches.
        let expect = primary.canonicalize().expect("canonical");
        assert_eq!(
            shared_primary_checkout(&primary).as_deref(),
            Some(expect.as_path()),
            "the primary worktree of a multi-worktree clone is the shared checkout"
        );
        assert_eq!(
            shared_primary_checkout(&linked),
            None,
            "a lab worktree is the writer's own checkout and must stay writable"
        );
        // The working directory may be a SUBDIRECTORY: the root is what identifies the checkout.
        let sub = primary.join("docs");
        std::fs::create_dir_all(&sub).expect("subdir");
        assert!(shared_primary_checkout(&sub).is_some());
    }

    /// A lab that simply cloned the repository must not be refused its own checkout.
    #[test]
    fn a_single_checkout_clone_is_never_refused() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("clone");
        std::fs::create_dir_all(&repo).expect("clone");
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&repo)
                .output()
                .expect("git")
        };
        run(&["init", "-q", "-b", "main"]);
        assert_eq!(
            shared_primary_checkout(&repo),
            None,
            "one checkout is the writer's own, shared layout or not"
        );
        assert_eq!(
            shared_primary_checkout(&dir.path().join("not-a-repo")),
            None,
            "an unreadable git state must not block a write"
        );
    }

    /// **F1: the shape rule had a hole, and the marker closes it.** `uor-hygiene` and the post-merge
    /// cleanup remove linked worktrees, after which a shape-only rule read the owner checkout as an
    /// ordinary single-checkout clone and allowed writes into it again (review finding F1: a
    /// single-checkout scratch repo accepted `team claim` and left `claims.json` modified).
    #[test]
    fn a_marked_single_checkout_is_still_the_shared_checkout() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("owner");
        std::fs::create_dir_all(&repo).expect("dir");
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&repo)
                .output()
                .expect("git")
        };
        run(&["init", "-q", "-b", "main"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        std::fs::write(repo.join("f"), "1").expect("write");
        run(&["add", "f"]);
        run(&["commit", "-qm", "one"]);
        assert_eq!(
            shared_primary_checkout(&repo),
            None,
            "an unmarked single checkout is a lab's own clone and stays writable"
        );
        // A marker that is false, or anything else unreadable, is not a declaration.
        run(&["config", "uor.sharedCheckout", "false"]);
        assert_eq!(shared_primary_checkout(&repo), None);
        // The marker does not depend on how many worktrees exist, so cleanup cannot switch it off.
        run(&["config", "uor.sharedCheckout", "true"]);
        let expect = repo.canonicalize().expect("canonical");
        assert_eq!(
            shared_primary_checkout(&repo).as_deref(),
            Some(expect.as_path()),
            "a marked checkout is the shared checkout even with no linked worktrees"
        );
        // A LINKED worktree of a marked checkout is still a lab's own, and stays writable.
        let linked = dir.path().join("lab");
        run(&[
            "worktree",
            "add",
            "-q",
            "-b",
            "lab/task",
            linked.to_str().expect("utf8"),
        ]);
        assert_eq!(shared_primary_checkout(&linked), None);
        assert_eq!(
            refuse_shared_checkout_at(&linked, "the claims store"),
            Ok(())
        );
    }

    /// The refusal names the rule and the fix, because the lab has to act on it.
    #[test]
    fn a_write_into_the_shared_checkout_is_refused_with_the_fix() {
        let (dir, primary, linked) = shared_clone();
        let live = dir.path().join("live");
        let live_arg = live.to_str().expect("utf8");
        let shared = StatePaths::resolve(&primary, None, Some(live_arg));
        let err = refuse_shared_checkout(&shared, "claims").expect_err("must refuse");
        assert!(err.contains("R10"), "{err}");
        assert!(err.contains("worktree"), "{err}");
        assert!(err.contains("claims"), "the write must be named: {err}");
        let own = StatePaths::resolve(&linked, None, Some(live_arg));
        assert_eq!(refuse_shared_checkout(&own, "claims"), Ok(()));
        // The path-based entry point, used by writers that hold a repository and not the harness
        // state (`ratchet --update-baseline`): same refusal, same message, and still allowed in a
        // lab's own worktree.
        let err = refuse_shared_checkout_at(&primary, "the ratchet baseline")
            .expect_err("the shared checkout must refuse the baseline too");
        assert!(
            err.contains("R10") && err.contains("ratchet baseline"),
            "{err}"
        );
        assert_eq!(
            refuse_shared_checkout_at(&linked, "the ratchet baseline"),
            Ok(())
        );
        assert_eq!(
            refuse_shared_checkout_at(&dir.path().join("not-a-repo"), "the ratchet baseline"),
            Ok(()),
            "a scratch directory is not the shared checkout"
        );
    }

    /// The real timestamps the harness writes, including the offset form.
    #[test]
    fn iso_timestamps_parse() {
        let z = parse_iso_unix("2026-10-12T23:26:55+00:00").expect("z");
        let off = parse_iso_unix("2026-10-12T23:26:55-04:00").expect("offset");
        assert_eq!(off - z, 4 * 3600, "an offset shifts the instant");
        assert_eq!(parse_iso_unix("1970-01-01T00:00:00Z"), Some(0));
    }

    /// A timestamp that cannot be read must not be treated as fresh.
    #[test]
    fn a_bad_timestamp_is_not_fresh() {
        assert_eq!(parse_iso_unix("not-a-timestamp"), None);
        assert_eq!(
            parse_iso_unix("2026-13-01T00:00:00Z"),
            None,
            "month out of range"
        );
        assert_eq!(parse_iso_unix(""), None);
        let p = PresenceRow {
            updated: "not-a-timestamp".into(),
            ..Default::default()
        };
        assert!(
            p.is_stale(0, 30),
            "unreadable time reads as stale, never live"
        );
    }

    /// The boundary that exposed the divergence: an age of 30m25s is STALE, and truncating to
    /// "30 minutes" before comparing would wrongly call it live.
    #[test]
    fn staleness_does_not_truncate_before_comparing() {
        let now = parse_iso_unix("2026-10-12T12:00:00Z").expect("now");
        let just_over = PresenceRow {
            updated: "2026-10-12T11:29:35Z".into(),
            ..Default::default()
        };
        assert_eq!(just_over.age_seconds(now), Some(30 * 60 + 25));
        assert_eq!(
            just_over.age_minutes(now),
            Some(30),
            "truncated for display"
        );
        assert!(just_over.is_stale(now, 30), "30m25s is past a 30m TTL");

        let exactly_on = PresenceRow {
            updated: "2026-10-12T11:30:00Z".into(),
            ..Default::default()
        };
        assert_eq!(exactly_on.age_seconds(now), Some(30 * 60));
        assert!(
            !exactly_on.is_stale(now, 30),
            "exactly at the TTL is not yet stale"
        );
    }

    #[test]
    fn age_and_staleness_follow_the_ttl() {
        let now = parse_iso_unix("2026-10-12T12:00:00Z").expect("now");
        let fresh = PresenceRow {
            updated: "2026-10-12T11:50:00Z".into(),
            ..Default::default()
        };
        assert_eq!(fresh.age_minutes(now), Some(10));
        assert!(!fresh.is_stale(now, 30));
        let old = PresenceRow {
            updated: "2026-10-12T10:00:00Z".into(),
            ..Default::default()
        };
        assert!(old.is_stale(now, 30));
    }

    #[test]
    fn a_ledger_line_reads_with_either_count_shape() {
        let heart =
            ReceiptRow::from_line(r#"{"lab":"kimi","kind":"heartbeat","ts":"t","count":"1/3"}"#);
        let team = ReceiptRow::from_line(r#"{"lab":"kimi","kind":"team","ts":"t","count":3}"#);
        assert!(matches!(
            heart.unwrap().count,
            Some(serde_json::Value::String(_))
        ));
        assert!(matches!(
            team.unwrap().count,
            Some(serde_json::Value::Number(_))
        ));
    }

    #[test]
    fn the_real_ledger_reads_and_is_sharded() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = StatePaths::resolve(&repo, None, Some("/nonexistent-live-tier"));
        let rows = read_ledger(&paths);
        assert!(!rows.is_empty(), "the committed ledger has receipts");
        assert!(
            rows.iter().all(|r| !r.kind.is_empty()),
            "every row has a kind"
        );
        assert!(
            rows.iter().all(|r| !r.ts.is_empty()),
            "every row has a timestamp"
        );
    }

    /// A test must never write to the machine's real shared directory.
    #[test]
    fn the_live_tier_can_be_pointed_somewhere_disposable() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = StatePaths::resolve(&repo, Some(Path::new("/tmp/home")), Some("/tmp/live"));
        assert_eq!(paths.live, PathBuf::from("/tmp/live"));
        let paths = StatePaths::resolve(&repo, Some(Path::new("/tmp/home")), None);
        assert!(paths.live.starts_with("/tmp/home"), "{:?}", paths.live);
    }

    #[test]
    fn claims_always_resolve_into_the_repo_root() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = StatePaths::resolve(&repo, None, None);
        assert_eq!(
            paths.claims_file(),
            paths.repo.join("docs/harness/claims.json")
        );
    }
}
