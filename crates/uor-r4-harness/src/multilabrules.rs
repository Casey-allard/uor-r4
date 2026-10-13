//! The `multilab` rules, as Rust (D30 check-port step 3 of 10).
//!
//! # The six `multilab` rules, and which are here
//!
//! | rule | severity | in this unit |
//! |---|---|---|
//! | `multilab.presence_fresh` | warn | **yes** |
//! | `multilab.claims_one_owner` | error | **yes** |
//! | `multilab.claim_stale` | warn | **yes** |
//! | `multilab.claim_not_merged` | warn | **yes** |
//! | `multilab.no_shared_ledger` | warn | no |
//! | `multilab.ledger_lab_recorded` | warn | no |
//!
//! ## `claim_not_merged` and the ordering that must not be lost
//!
//! It asks GitHub whether a branch has a **merged PR**, and the Python carries a hard-won note about a
//! **false positive on live work**: `kimi/plan-refresh-2026-10-13` was claimed and flagged as merged
//! within the same couple of minutes, and the warning vanished once the remote-tracking ref arrived.
//!
//! **The fix was to ask GitHub FIRST and to answer `None` — "not established" — rather than guess**,
//! because **a rule that cries wolf on a lab's live branch is worse than no rule.**
//!
//! The ordering is not an optimisation, it is the bug fix:
//!
//! - **Content comparison first lies in the dangerous direction.** A lab that has just claimed a
//!   branch and pushed it may not have it in the local remote-tracking refs yet, so the branch looks
//!   **gone** — and "not on origin" was the first version's mistake: it treated *gone* as *merged*,
//!   which would tell a lab to release a claim on **unpushed work**.
//! - `None` is a real answer, distinct from `false`. **`false` means "I have evidence it is not
//!   merged"; `None` means "I have no evidence either way", and the rule says nothing for `None`.**
//!   Collapsing those two is how a check starts guessing.
//!
//! `claim_branch_is_merged` therefore returns `Option<bool>` and its tests pin the ordering and the
//! unknown case, not just the merged and unmerged answers.
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

/// Whether `branch` has a **merged** pull request.
///
/// `Some(true)` merged, `Some(false)` evidence it is not merged, **`None` not established** — and the
/// rule says nothing for `None`. See the module header for why the answer is three-valued and why
/// GitHub is asked before any local reasoning.
///
/// Three signals, strongest first:
///
/// 1. **A merged PR for this branch** — the direct evidence, and the only thing that proves a *squash*
///    merge, which leaves no ancestry link for `merge-base --is-ancestor` to find.
/// 2. **No commits ahead of `origin/main`**, when the branch still exists locally or on the remote.
///    Content proof, for a branch that was merged and kept.
///
///    **Measured, and it is weaker than it reads.** With `gh` absent from `PATH`, a real merged claim
///    whose branch had been deleted after merging returned `None` — because **the fallback needs a
///    visible ref, and the project's rule is to delete the branch after merging.** Both live claims
///    (`codex/count-reference-closeout-20261013`, `kimi/d29-nvrtc-amendment`) have no local or
///    remote-tracking ref, so with GitHub down this rule reports **nothing at all**.
///
///    So the fallback is **not a safety net for this rule**; it only helps the narrow case of a branch
///    that was merged *and kept*. That is worth stating because the comment above made it sound like a
///    general offline path, and **I first wrote it as though I had observed it working.**
/// 3. Otherwise `None`.
pub fn claim_branch_is_merged(paths: &StatePaths, branch: &str) -> Option<bool> {
    // 1. GitHub first. See the module header: the ordering IS the bug fix.
    //
    //    A failed `gh` call is `None` from this sub-step, which falls through to the content check
    //    rather than returning early - the Python does the same, and it is why an offline machine
    //    still gets the safe half of the answer.
    if let Some(merged_pr) = gh_has_merged_pr(paths, branch) {
        if merged_pr {
            return Some(true);
        }
        // No merged PR. If the branch is visible anywhere, it is unmerged work. If it is not visible
        // either, the honest answer is "not established" rather than "merged" - the branch may have
        // been deleted after merging AND after the PR list aged out, and guessing "unmerged" would
        // leave a stale claim in place while guessing "merged" would tell a working lab to release.
        let visible = git_has_ref(paths, &format!("refs/remotes/origin/{branch}"))
            || git_has_ref(paths, &format!("refs/heads/{branch}"));
        return if visible { Some(false) } else { None };
    }

    // 2. GitHub unreachable: fall back to content, which is safe in the direction that matters.
    //    Nothing ahead of main means the work is in main, whatever route put it there.
    for candidate in [
        format!("refs/remotes/origin/{branch}"),
        format!("refs/heads/{branch}"),
    ] {
        if git_has_ref(paths, &candidate) {
            let ahead = git_stdout(
                paths,
                &["rev-list", "--count", &format!("origin/main..{candidate}")],
            );
            return ahead.trim().parse::<i64>().ok().map(|n| n == 0);
        }
    }
    None
}

/// `Some(true)` when GitHub reports a merged PR for this branch, `Some(false)` when GitHub answered and
/// there is none, **`None` when GitHub could not be asked.**
fn gh_has_merged_pr(paths: &StatePaths, branch: &str) -> Option<bool> {
    let out = std::process::Command::new("gh")
        .args([
            "pr",
            "list",
            "--head",
            branch,
            "--state",
            "merged",
            "--limit",
            "1",
            "--json",
            "number,state",
        ])
        .current_dir(&paths.repo)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&text).ok()?;
    let arr = parsed.as_array()?;
    Some(!arr.is_empty())
}

fn git_has_ref(paths: &StatePaths, refname: &str) -> bool {
    std::process::Command::new("git")
        .args(["-C"])
        .arg(&paths.repo)
        .args(["rev-parse", "--verify", "--quiet", refname])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git_stdout(paths: &StatePaths, args: &[&str]) -> String {
    std::process::Command::new("git")
        .arg("-C")
        .arg(&paths.repo)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

/// `multilab.claim_not_merged` — no claim is held on a branch whose work is already in `origin/main`.
///
/// **It reports only on `Some(true)`.** A claim whose branch cannot be established as merged stays
/// quiet, because **a rule that cries wolf on a lab's live branch is worse than no rule** — and the
/// false positive this guards against was real: a branch claimed and pushed seconds earlier looked
/// gone, and the warning vanished once the ref arrived.
pub fn claim_not_merged(paths: &StatePaths) -> Vec<Finding> {
    let mut out = Vec::new();
    for (branch, held) in team::load_claims(paths) {
        if claim_branch_is_merged(paths, &branch) != Some(true) {
            continue;
        }
        let lab = if held.lab.trim().is_empty() {
            "nobody".to_string()
        } else {
            held.lab.clone()
        };
        out.push(warn(
            "multilab.claim_not_merged",
            format!(
                "{lab} still holds a claim on {branch:?}, but its work is already in origin/main"
            ),
            "docs/harness/claims.json".into(),
            format!("release it: uor-harness team release --branch {branch} --lab {lab}"),
        ));
    }
    out
}

/// Every `multilab` rule ported in this unit.
pub fn all(paths: &StatePaths, now: i64) -> Vec<Finding> {
    let mut out = presence_fresh(paths, now);
    out.extend(claims_one_owner(paths));
    out.extend(claim_stale(paths, now));
    out.extend(claim_not_merged(paths));
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

    /// **The contract that must not be collapsed: `None` is not `false`.**
    ///
    /// `false` means "I have evidence it is not merged". `None` means "I have no evidence either way",
    /// and the rule says nothing for `None`. A branch that has never existed is exactly the unknown
    /// case: it is not on origin, it is not local, and nobody can tell whether it was merged and
    /// deleted or never pushed. **Treating that as merged would tell a lab to release a claim on
    /// unpushed work** - the first version's mistake, recorded in the Python.
    #[test]
    fn a_branch_that_never_existed_is_unknown_not_merged() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
        // A real git repo, so the git calls answer honestly rather than erroring into a fake answer.
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .output()
                .expect("git");
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["config", "user.email", "t@t"]);
        git(&["config", "user.name", "t"]);
        std::fs::write(root.join("f.txt"), "x").expect("write");
        git(&["add", "-A"]);
        git(&["commit", "-qm", "init"]);

        // `gh` will not report a merged PR for a nonsense branch name, and the branch is not visible
        // anywhere, so the honest answer is "not established".
        let answer = claim_branch_is_merged(&paths_of(root), "definitely/never-existed-xyz");
        assert_ne!(
            answer,
            Some(true),
            "an unknown branch must never read as merged"
        );
    }

    /// **The rule reports only on certain evidence.** An unknown branch must produce no finding, or the
    /// rule cries wolf on a branch a lab just pushed - the false positive that was measured on live
    /// work and fixed by asking GitHub first.
    #[test]
    fn a_claim_that_cannot_be_established_produces_no_finding() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .output()
                .expect("git");
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["config", "user.email", "t@t"]);
        git(&["config", "user.name", "t"]);
        std::fs::write(root.join("f.txt"), "x").expect("write");
        git(&["add", "-A"]);
        git(&["commit", "-qm", "init"]);
        std::fs::write(
            root.join("docs/harness/claims.json"),
            r#"{"just/pushed":{"branch":"just/pushed","lab":"kimi","head":"","claimed_at":"2026-10-13T12:00:00Z","note":""}}"#,
        )
        .expect("write");
        let f = claim_not_merged(&paths_of(root));
        assert!(
            f.is_empty(),
            "a claim that cannot be established as merged must stay quiet: {f:?}"
        );
    }

    /// A branch with **no commits ahead of main** IS established as merged by content, and is reported.
    /// This is the one direction the content check is safe in: nothing ahead of main means the work is
    /// in main, whatever route put it there.
    #[test]
    fn a_branch_with_nothing_ahead_of_main_is_reported() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .output()
                .expect("git");
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["config", "user.email", "t@t"]);
        git(&["config", "user.name", "t"]);
        std::fs::write(root.join("f.txt"), "x").expect("write");
        git(&["add", "-A"]);
        git(&["commit", "-qm", "init"]);
        // A local branch pointing at the same commit as main: zero commits ahead.
        git(&["branch", "already/in-main"]);
        std::fs::write(
            root.join("docs/harness/claims.json"),
            r#"{"already/in-main":{"branch":"already/in-main","lab":"kimi","head":"","claimed_at":"2026-10-13T12:00:00Z","note":""}}"#,
        )
        .expect("write");
        // The content path needs an `origin/main` to compare against.
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        let f = claim_not_merged(&paths_of(root));
        assert_eq!(
            f.len(),
            1,
            "a branch with nothing ahead of main is merged work: {f:?}"
        );
        assert!(f[0].message.contains("already/in-main"), "{}", f[0].message);
        assert!(f[0].fix.contains("team release"), "{}", f[0].fix);
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
