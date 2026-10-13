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

    /// The committed claims store. Always in the repository root, never a worktree-local copy —
    /// a claim written into a linked worktree is deleted along with the worktree and leaves every
    /// other lab unable to see that the branch was ever held.
    pub fn claims_file(&self) -> PathBuf {
        self.repo.join("docs").join("harness").join("claims.json")
    }
}

/// Reduce a lab name to the form used in file names, matching the Python `slug()`.
pub fn slug(lab: &str) -> String {
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

/// A per-lab check-in.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Presence {
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

impl Presence {
    pub fn read(path: &Path, source: &str) -> Option<Presence> {
        let text = std::fs::read_to_string(path).ok()?;
        let mut p: Presence = serde_json::from_str(&text).ok()?;
        p.source = source.to_string();
        Some(p)
    }

    /// Read a lab's presence, preferring the live tier and falling back to the durable record.
    pub fn read_lab(paths: &StatePaths, lab: &str) -> Option<Presence> {
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

/// One receipt line from a per-lab ledger shard.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Receipt {
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

impl Receipt {
    pub fn from_line(line: &str) -> Option<Receipt> {
        serde_json::from_str(line).ok()
    }
}

/// Read every receipt in the sharded ledger, in file order.
pub fn read_ledger(paths: &StatePaths) -> Vec<Receipt> {
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
            if let Some(r) = Receipt::from_line(line) {
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
        assert_eq!(slug("---"), "");
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
        let p = Presence {
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
        let just_over = Presence {
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

        let exactly_on = Presence {
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
        let fresh = Presence {
            updated: "2026-10-12T11:50:00Z".into(),
            ..Default::default()
        };
        assert_eq!(fresh.age_minutes(now), Some(10));
        assert!(!fresh.is_stale(now, 30));
        let old = Presence {
            updated: "2026-10-12T10:00:00Z".into(),
            ..Default::default()
        };
        assert!(old.is_stale(now, 30));
    }

    #[test]
    fn a_ledger_line_reads_with_either_count_shape() {
        let heart =
            Receipt::from_line(r#"{"lab":"kimi","kind":"heartbeat","ts":"t","count":"1/3"}"#);
        let team = Receipt::from_line(r#"{"lab":"kimi","kind":"team","ts":"t","count":3}"#);
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
