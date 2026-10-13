//! The `multilab` rules, as Rust (D30 check-port step 3 of 10).
//!
//! # The six `multilab` rules, and which are here
//!
//! | rule | severity | in this unit |
//! |---|---|---|
//! | `multilab.presence_fresh` | warn | **yes** |
//! | `multilab.claims_one_owner` | error | **yes** |
//! | `multilab.claim_stale` | warn | **yes** |
//! | `multilab.claim_not_merged` | warn | no — see below |
//! | `multilab.no_shared_ledger` | warn | no |
//! | `multilab.ledger_lab_recorded` | warn | no |
//!
//! ## Why `claim_not_merged` is not here
//!
//! It asks GitHub whether a branch has a **merged PR**, and the Python implementation carries a
//! hard-won note about a **false positive on live work**: `kimi/plan-refresh-2026-10-13` was claimed
//! and flagged as merged within the same couple of minutes, and the warning vanished once the
//! remote-tracking ref arrived. The fix was to ask GitHub **first** and to answer `None` — "not
//! established" — rather than guess, because **a rule that cries wolf on a lab's live branch is worse
//! than no rule.**
//!
//! That ordering, the `None` case, and the `gh` dependency need porting together and carefully. **A
//! hasty port would reintroduce exactly the bug the Python comments exist to prevent**, so it gets its
//! own unit rather than riding along here.
//!
//! The two ledger rules read receipt shards and belong with the `receipts` module.
//!
//! ## The 3.9 timestamp note, which the port inherits
//!
//! `_parse_claimed_at` accepts both offset forms because `datetime.fromisoformat` rejected the `Z`
//! suffix before Python 3.11, while **the Rust implementation writes `Z` timestamps.** Without that,
//! a Rust-written claim is unreadable by the Python harness on 3.9 and **the staleness rule silently
//! stops working.** `parse_claimed_at` here must accept both forms for the same reason in reverse.

use crate::multilab;
use crate::state::StatePaths;
use crate::team;

pub use crate::preprules::Finding;

/// Build a finding. `preprules::Finding`'s constructors are private to that module, so these rules
/// construct it directly; the shape is shared so a caller can print both sets together.
fn warn(rule_id: &'static str, message: String, path: String, fix: String) -> Finding {
    Finding {
        rule_id,
        severity: "warn",
        message,
        path,
        fix,
    }
}

fn error(rule_id: &'static str, message: String, path: String, fix: String) -> Finding {
    Finding {
        rule_id,
        severity: "error",
        message,
        path,
        fix,
    }
}

/// How long a claim may be held with no heartbeat before it reads as a lock rather than a signal.
/// The PM proposed a day (pm-rules.md R1).
pub const CLAIM_ABANDONED_HOURS: i64 = 24;

/// Presence older than this reads as STALE rather than idle.
pub const PRESENCE_TTL_MINUTES: i64 = 5;

/// `multilab.presence_fresh` — every lab with a presence file has refreshed it recently.
///
/// **Stale is not idle.** It means that lab is working without checking in, which is what the board
/// is for; the rule exists so a lab that has gone quiet is visible on a `check` run rather than only
/// to whoever happens to read the board.
pub fn presence_fresh(paths: &StatePaths, now: i64) -> Vec<Finding> {
    let mut out = Vec::new();
    for e in multilab::board(paths, now) {
        if !e.stale {
            continue;
        }
        let lab = e.row.lab.clone();
        let doing = e.row.doing.clone();
        out.push(warn(
            "multilab.presence_fresh",
            format!(
                "lab {lab}: presence is {}{}",
                e.state(),
                if doing.is_empty() {
                    String::new()
                } else {
                    format!(" (last: {doing})")
                }
            ),
            format!("docs/harness/presence/{lab}.json"),
            format!("the lab refreshes it with: uor-harness heartbeat -m <milestone> --lab {lab}"),
        ));
    }
    out
}

/// `multilab.claims_one_owner` — no branch is claimed with no lab.
///
/// **The claims file is keyed by branch**, so a genuine double claim cannot be represented — one of
/// them overwrites the other. What *can* happen is a claim with an empty lab, and that is worse than
/// a conflict: **it is a branch held by nobody that no lab can be asked about.** Codex and Antigravity
/// both hit this through a `claim`/`release` path that wrote the merged view (fixed in #2359).
pub fn claims_one_owner(paths: &StatePaths) -> Vec<Finding> {
    let mut out = Vec::new();
    for (branch, held) in team::load_claims(paths) {
        if held.lab.trim().is_empty() {
            out.push(error(
                "multilab.claims_one_owner",
                format!("branch {branch:?} is claimed with no lab"),
                "docs/harness/claims.json".into(),
                "re-claim it with --lab <lab>, or release it".into(),
            ));
        }
    }
    out
}

/// `multilab.claim_stale` — no claim is held past the abandonment window without a live heartbeat.
///
/// **The heartbeat is what distinguishes "holding the branch while working" from "holding it and
/// gone."** A lab that checks in keeps its claims; one that does not shows up here. That distinction
/// is the whole rule: without it, a long-running piece of work would be flagged as abandoned purely
/// for taking a day, which is exactly the kind of false positive that gets a rule ignored.
pub fn claim_stale(paths: &StatePaths, now: i64) -> Vec<Finding> {
    let presence: Vec<multilab::BoardEntry> = multilab::board(paths, now);
    let hour = 3600i64;
    let mut out = Vec::new();
    for (branch, held) in team::load_claims(paths) {
        let lab = held.lab.clone();
        let Some(claimed_at) = parse_claimed_at(&held.claimed_at) else {
            // An unreadable timestamp is reported by the claim's own shape rules, not guessed at here.
            continue;
        };
        let age_hours = (now - claimed_at) / hour;
        if age_hours < CLAIM_ABANDONED_HOURS {
            continue;
        }
        let p = presence.iter().find(|e| e.row.lab == lab);
        let fresh = p
            .map(|e| e.age_minutes <= PRESENCE_TTL_MINUTES)
            .unwrap_or(false);
        if fresh {
            continue; // the lab is visibly working; the claim is information, not a lock
        }
        out.push(warn(
            "multilab.claim_stale",
            format!(
                "{} has held {branch:?} for {age_hours}h with no live heartbeat{}",
                if lab.is_empty() { "nobody".to_string() } else { lab.clone() },
                if p.is_none() { " and has published no presence at all" } else { "" }
            ),
            "docs/harness/claims.json".into(),
            "release it if you are done, or run `uor-harness heartbeat -m <milestone>` so the claim \
             is visibly live; the PM resolves a contested claim (pm-rules.md R7)"
                .into(),
        ));
    }
    out
}

/// A claim timestamp in either offset form, as seconds since the epoch.
///
/// **Both forms, and the reason is the same one the Python carries:** `datetime.fromisoformat` rejected
/// the `Z` suffix before Python 3.11 and the harness supports 3.9, while **Rust writes `Z`
/// timestamps.** A parser that accepted only one form would make a claim written by one implementation
/// unreadable to the other, and **the staleness rule would silently stop firing on those claims** —
/// the failure mode this whole line keeps producing.
///
/// Accepts `2026-10-13T12:00:00Z`, `...+00:00`, `...-04:00`, and a bare `...T12:00:00`.
pub fn parse_claimed_at(text: &str) -> Option<i64> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    // Split the date and time, requiring the `T` so a bare date is not silently read as midnight.
    let (date, rest) = t.split_once('T')?;
    let mut d = date.split('-');
    let year: i64 = d.next()?.parse().ok()?;
    let month: i64 = d.next()?.parse().ok()?;
    let day: i64 = d.next()?.parse().ok()?;
    if d.next().is_some() {
        return None;
    }

    // The offset may be `Z`, `+HH:MM`, `-HH:MM`, or absent.
    let (time, offset_seconds) = if let Some(stripped) = rest.strip_suffix('Z') {
        (stripped, 0)
    } else if rest.len() > 6
        && (rest.as_bytes()[rest.len() - 6] == b'+' || rest.as_bytes()[rest.len() - 6] == b'-')
    {
        let sign = if rest.as_bytes()[rest.len() - 6] == b'-' {
            -1
        } else {
            1
        };
        let off = &rest[rest.len() - 5..];
        let (h, m) = off.split_once(':')?;
        let h: i64 = h.parse().ok()?;
        let m: i64 = m.parse().ok()?;
        (&rest[..rest.len() - 6], sign * (h * 3600 + m * 60))
    } else {
        (rest, 0)
    };

    let mut ts = time.split(':');
    let hh: i64 = ts.next()?.parse().ok()?;
    let mm: i64 = ts.next()?.parse().ok()?;
    let ss_part = ts.next().unwrap_or("0");
    // Tolerate fractional seconds, which a caller may include.
    let ss: i64 = ss_part.split('.').next()?.parse().ok()?;

    // Days from the civil date, so no `date` binary is needed — `date -r` is BSD-only and
    // `--reference` is GNU-only, and the difference already caused a Linux CI panic once.
    let days = days_from_civil(year, month, day);
    Some(days * 86400 + hh * 3600 + mm * 60 + ss - offset_seconds)
}

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Every `multilab` rule ported in this unit.
pub fn all(paths: &StatePaths, now: i64) -> Vec<Finding> {
    let mut out = presence_fresh(paths, now);
    out.extend(claims_one_owner(paths));
    out.extend(claim_stale(paths, now));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn paths_of(root: &Path) -> StatePaths {
        StatePaths::resolve(root, None, Some("/nonexistent-live"))
    }

    /// Both offset forms must parse, because one implementation writes `Z` and the harness must read
    /// the other's claims. **A parser that took only one form would make the staleness rule silently
    /// stop firing** on exactly those claims.
    #[test]
    fn both_timestamp_forms_parse_to_the_same_instant() {
        let z = parse_claimed_at("2026-10-13T12:00:00Z").expect("Z form");
        let plus = parse_claimed_at("2026-10-13T12:00:00+00:00").expect("+00:00 form");
        assert_eq!(
            z, plus,
            "the Z form and the +00:00 form are the same instant"
        );
        // And a non-UTC offset actually shifts the instant.
        let minus4 = parse_claimed_at("2026-10-13T12:00:00-04:00").expect("-04:00 form");
        assert_eq!(
            minus4,
            z + 4 * 3600,
            "a -04:00 claim is four hours later in UTC"
        );
    }

    #[test]
    fn a_bare_date_or_garbage_is_not_silently_midnight() {
        assert!(
            parse_claimed_at("2026-10-13").is_none(),
            "no T means no time"
        );
        assert!(parse_claimed_at("").is_none());
        assert!(parse_claimed_at("not a date").is_none());
    }

    #[test]
    fn fractional_seconds_are_tolerated() {
        assert_eq!(
            parse_claimed_at("2026-10-13T12:00:00.500Z"),
            parse_claimed_at("2026-10-13T12:00:00Z")
        );
    }

    /// The absolute epoch is right, not merely self-consistent.
    #[test]
    fn the_epoch_is_the_real_one() {
        assert_eq!(parse_claimed_at("1970-01-01T00:00:00Z"), Some(0));
        // **Cross-checked against two independent sources**, not recalled: Python's
        // `datetime.date(2026,10,13) - date(1970,1,1)` and BSD `date` both say **20739**, and this is
        // the value the implementation produced. My first version of this test asserted **20744** —
        // **the code was right and the test was wrong, for the third time in this port.** The lesson
        // is not "be careful": it is that **an expected value written from memory is a guess wearing
        // a test's clothes.**
        assert_eq!(days_from_civil(2026, 10, 13), 20739);
    }

    /// A claim with no lab is worse than a conflict: it is a branch held by nobody that no lab can be
    /// asked about.
    #[test]
    fn a_claim_with_no_lab_is_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
        std::fs::write(
            root.join("docs/harness/claims.json"),
            r#"{"orphan/branch":{"branch":"orphan/branch","lab":"","head":"","claimed_at":"2026-10-13T00:00:00Z","note":""}}"#,
        )
        .expect("write");
        let f = claims_one_owner(&paths_of(root));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, "error");
        assert!(f[0].message.contains("orphan/branch"), "{}", f[0].message);
    }

    #[test]
    fn a_claim_with_a_lab_is_fine() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
        std::fs::write(
            root.join("docs/harness/claims.json"),
            r#"{"b":{"branch":"b","lab":"kimi","head":"","claimed_at":"2026-10-13T00:00:00Z","note":""}}"#,
        )
        .expect("write");
        assert!(claims_one_owner(&paths_of(root)).is_empty());
    }

    /// A long-held claim whose lab IS checking in must not be flagged: the heartbeat is what
    /// distinguishes holding-while-working from holding-and-gone.
    #[test]
    fn a_long_claim_with_a_live_heartbeat_is_not_stale() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
        let now = parse_claimed_at("2026-10-13T12:00:00Z").expect("now");
        let old = "2026-10-11T12:00:00Z"; // 48h before
        std::fs::write(
            root.join("docs/harness/claims.json"),
            format!(
                r#"{{"b":{{"branch":"b","lab":"kimi","head":"","claimed_at":"{old}","note":""}}}}"#
            ),
        )
        .expect("write");
        // A heartbeat a minute old, in the live tier.
        let live = root.join("live");
        std::fs::create_dir_all(live.join("presence")).expect("mkdir");
        let one_min_ago = now - 60;
        std::fs::write(
            live.join("presence/kimi.json"),
            format!(
                r#"{{"lab":"kimi","updated_at":{},"milestone":"m1","doing":"working"}}"#,
                one_min_ago
            ),
        )
        .expect("write");
        let p = StatePaths::resolve(root, None, Some(live.to_str().expect("utf8")));
        assert!(
            claim_stale(&p, now).is_empty(),
            "a lab that is checking in keeps its claims: {:?}",
            claim_stale(&p, now)
        );
    }

    /// And the converse: the same claim with no heartbeat at all is reported.
    #[test]
    fn a_long_claim_with_no_heartbeat_is_stale() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
        let now = parse_claimed_at("2026-10-13T12:00:00Z").expect("now");
        std::fs::write(
            root.join("docs/harness/claims.json"),
            r#"{"b":{"branch":"b","lab":"kimi","head":"","claimed_at":"2026-10-11T12:00:00Z","note":""}}"#,
        )
        .expect("write");
        let f = claim_stale(&paths_of(root), now);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("48h"), "{}", f[0].message);
        assert!(
            f[0].message.contains("no presence at all"),
            "{}",
            f[0].message
        );
    }

    /// A claim inside the window is not stale, however quiet its lab is.
    #[test]
    fn a_recent_claim_is_not_stale() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
        let now = parse_claimed_at("2026-10-13T12:00:00Z").expect("now");
        std::fs::write(
            root.join("docs/harness/claims.json"),
            r#"{"b":{"branch":"b","lab":"kimi","head":"","claimed_at":"2026-10-13T11:00:00Z","note":""}}"#,
        )
        .expect("write");
        assert!(claim_stale(&paths_of(root), now).is_empty());
    }

    /// An unreadable timestamp is skipped rather than guessed at.
    #[test]
    fn an_unreadable_claim_timestamp_is_skipped() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
        let now = parse_claimed_at("2026-10-13T12:00:00Z").expect("now");
        std::fs::write(
            root.join("docs/harness/claims.json"),
            r#"{"b":{"branch":"b","lab":"kimi","head":"","claimed_at":"whenever","note":""}}"#,
        )
        .expect("write");
        assert!(
            claim_stale(&paths_of(root), now).is_empty(),
            "no guess, no finding"
        );
    }

    /// The real repo runs through the rules without panicking on real data.
    #[test]
    fn the_real_repo_is_processed() {
        let p = paths_of(&repo());
        let now = crate::team::now_iso();
        assert!(!now.is_empty());
        let ts = parse_claimed_at(&now).expect("the harness's own timestamp format must parse");
        assert!(
            ts > 1_700_000_000,
            "the parsed timestamp is a real recent instant"
        );
        let _ = all(&p, ts);
    }
}
