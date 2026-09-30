//! Incremental, paginated GitHub snapshots compatible with uor_knowledge JSONL.
//! Retrieval is an index, never authority or authorization. A successful cursor
//! covers issue/PR bodies, conversation comments and observed submitted reviews/
//! inline comments for selected PRs, not exhaustive history or resolved threads.

use crate::{invalid, ledger::sha256, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    schema: String,
    repository: String,
    since: String,
    snapshot: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReceipt {
    pub schema: String,
    pub repository: String,
    pub collected_at: String,
    pub main_sha: String,
    pub records: usize,
    pub import_sha256: String,
    pub output: PathBuf,
    pub previous_since: Option<String>,
    pub queried_since: Option<String>,
    pub next_since: String,
    pub server_started_epoch: u64,
    pub server_finished_epoch: u64,
    pub cursor_recovery: String,
    pub coverage_complete: bool,
    pub review_pull_requests: Vec<u64>,
    pub submitted_reviews: usize,
    pub inline_review_comments: usize,
    pub pending_reviews_excluded: usize,
    pub coverage: Vec<String>,
    pub github_mutations: u32,
}

const OVERLAP_SECONDS: u64 = 300;
fn iso(epoch: u64) -> Result<String> {
    let epoch = i64::try_from(epoch).map_err(|_| invalid("GitHub timestamp overflow"))?;
    let (year, month, day) = crate::civil_from_days(epoch / 86400);
    let day_seconds = epoch % 86400;
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_seconds / 3600,
        (day_seconds / 60) % 60,
        day_seconds % 60
    ))
}
fn parse_iso(value: &str) -> Result<u64> {
    if !value.is_ascii()
        || value.len() != 20
        || &value[4..5] != "-"
        || &value[7..8] != "-"
        || &value[10..11] != "T"
        || &value[13..14] != ":"
        || &value[16..17] != ":"
        || &value[19..] != "Z"
    {
        return Err(invalid("invalid sync cursor timestamp"));
    }
    let n = |a: usize, b: usize| {
        value[a..b]
            .parse::<i64>()
            .map_err(|_| invalid("invalid cursor time"))
    };
    let (year, month, day, hour, minute, second) = (
        n(0, 4)?,
        n(5, 7)?,
        n(8, 10)?,
        n(11, 13)?,
        n(14, 16)?,
        n(17, 19)?,
    );
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(0..24).contains(&hour)
        || !(0..60).contains(&minute)
        || !(0..60).contains(&second)
    {
        return Err(invalid("invalid cursor date fields"));
    }
    let y = year - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let days = era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468;
    let result = u64::try_from(days * 86400 + hour * 3600 + minute * 60 + second)
        .map_err(|_| invalid("cursor before epoch"))?;
    if iso(result)? != value {
        return Err(invalid("noncanonical cursor timestamp"));
    }
    Ok(result)
}
/// A restored future/legacy cursor cannot be clamped to now: that would still
/// skip its unknown historical gap. Recover with a full issue/comment sweep.
fn cursor_window(
    previous: Option<&Cursor>,
    server_start: u64,
) -> Result<(Option<String>, String, String)> {
    let next = iso(server_start.saturating_sub(OVERLAP_SECONDS))?;
    let (query, recovery) = match previous {
        None => (None, "initial_full_scan"),
        Some(c) if c.schema == "uor-r4.github-sync-cursor/1" => {
            (None, "legacy_local_clock_cursor_full_rescan")
        }
        Some(c) => match parse_iso(&c.since) {
            Ok(since) if since <= server_start => {
                (Some(c.since.clone()), "server_cursor_with_300s_overlap")
            }
            Ok(_) => (None, "future_cursor_full_rescan"),
            Err(_) => (None, "malformed_cursor_time_full_rescan"),
        },
    };
    Ok((query, next, recovery.into()))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KnowledgeRecord {
    id: String,
    kind: String,
    title: String,
    body: String,
    origin: String,
    revision: String,
    visibility: String,
    evidence_status: String,
    collected_at: String,
    content_sha256: String,
}
fn gh(args: &[String]) -> Result<Vec<u8>> {
    let output = Command::new("gh").args(args).output()?;
    if !output.status.success() {
        return Err(invalid(format!(
            "GitHub sync failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(output.stdout)
}
fn pages(endpoint: String) -> Result<Vec<Value>> {
    let data: Value = serde_json::from_slice(&gh(&[
        "api".into(),
        endpoint,
        "--paginate".into(),
        "--slurp".into(),
    ])?)?;
    let mut rows = Vec::new();
    for page in data
        .as_array()
        .ok_or_else(|| invalid("paginated response has no pages"))?
    {
        rows.extend(
            page.as_array()
                .ok_or_else(|| invalid("paginated page is not an array"))?
                .iter()
                .cloned(),
        );
    }
    Ok(rows)
}
fn record(
    kind: &str,
    title: String,
    body: String,
    origin: String,
    revision: String,
    status: &str,
    now: &str,
) -> Result<KnowledgeRecord> {
    let content_sha256 = sha256(body.as_bytes());
    let identity = json!({"kind":kind,"title":title,"origin":origin,"revision":revision,"visibility":"public","evidence_status":status,"content_sha256":content_sha256});
    Ok(KnowledgeRecord {
        id: format!("kb:sha256:{}", sha256(&serde_json::to_vec(&identity)?)),
        kind: kind.into(),
        title,
        body,
        origin,
        revision,
        visibility: "public".into(),
        evidence_status: status.into(),
        collected_at: now.into(),
        content_sha256,
    })
}
fn text<'a>(row: &'a Value, key: &str) -> Result<&'a str> {
    row[key]
        .as_str()
        .ok_or_else(|| invalid(format!("GitHub source lacks {key}")))
}

/// Preserve the ten-field knowledge JSONL interface. Review-specific source
/// revision and supersession metadata live in the JSON body, beside the full
/// native API record. submitted_at alone cannot distinguish edited/dismissed
/// reviews, so revisions additionally bind the observed content digest.
fn review_record(
    repository: &str,
    number: u64,
    kind: &str,
    row: &Value,
    now: &str,
) -> Result<KnowledgeRecord> {
    let id = row["id"]
        .as_u64()
        .ok_or_else(|| invalid("review source lacks numeric identity"))?;
    let source_identity = format!("github:{repository}:pull:{number}:{kind}:{id}");
    let stamp = row["updated_at"]
        .as_str()
        .or_else(|| row["submitted_at"].as_str())
        .ok_or_else(|| invalid("review source lacks revision timestamp"))?;
    parse_iso(stamp)?;
    let revision = format!("{stamp}:sha256:{}", sha256(&serde_json::to_vec(row)?));
    let body = json!({"schema":"uor-r4.github-review-source/1","source_identity":source_identity,"supersession_identity":source_identity,"source_revision":revision,"source_timestamp":stamp,"supersession_rule":"Later collection of this source identity supersedes its observed current state; preserve earlier revisions as history. This does not reconstruct deleted or unobserved revisions.","source":row});
    record(
        kind,
        format!("PR #{number} {kind} {id}"),
        serde_json::to_string_pretty(&body)?,
        text(row, "html_url")?.into(),
        revision,
        "NATIVE_REVIEW_SNAPSHOT",
        now,
    )
}

/// Writes only output_root. No ingest or model execution is performed.
/// Initial sync enumerates all available issue/PR bodies and comments; later
/// syncs overlap the previous server-start timestamp. Every selected PR,
/// including closed ones, receives paginated submitted-review/inline-comment
/// retrieval. Snapshots retain explicit non-exhaustive coverage limitations.
pub fn sync_github(repository: &str, output_root: &Path) -> Result<SyncReceipt> {
    let parts: Vec<_> = repository.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|p| {
            p.is_empty()
                || p.starts_with('-')
                || !p
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
    {
        return Err(invalid("repository must be owner/name"));
    }
    fs::create_dir_all(output_root)?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(output_root.join("sync.lock"))?;
    lock.try_lock().map_err(|e| {
        invalid(format!(
            "knowledge sync is already active or lock unavailable: {e}"
        ))
    })?;
    let cursor_path = output_root.join("cursor.json");
    let previous = if cursor_path.exists() {
        let value: Cursor = serde_json::from_slice(&fs::read(&cursor_path)?)?;
        if !["uor-r4.github-sync-cursor/1", "uor-r4.github-sync-cursor/2"]
            .contains(&value.schema.as_str())
            || value.repository != repository
        {
            return Err(invalid(
                "knowledge cursor belongs to another repository/schema",
            ));
        }
        Some(value)
    } else {
        None
    };
    // Capture GitHub's Date before any collection request. The local wall clock
    // is used only for a unique output directory name, never a query cursor.
    let server_start = crate::coord::github_time(repository)?;
    let now = iso(server_start)?;
    let (query_since, next_since, cursor_recovery) =
        cursor_window(previous.as_ref(), server_start)?;
    let repo: Value = serde_json::from_slice(&gh(&["api".into(), format!("repos/{repository}")])?)?;
    if repo["private"] != false
        || text(&repo, "full_name")?.to_lowercase() != repository.to_lowercase()
    {
        return Err(invalid(
            "public knowledge sync requires matching public repository",
        ));
    }
    let main: Value = serde_json::from_slice(&gh(&[
        "api".into(),
        format!("repos/{repository}/commits/main"),
    ])?)?;
    let main_sha = text(&main, "sha")?.to_string();
    let since = query_since
        .as_ref()
        .map(|v| format!("&since={v}"))
        .unwrap_or_default();
    let issues = pages(format!(
        "repos/{repository}/issues?state=all&sort=updated&direction=asc&per_page=100{since}"
    ))?;
    let comments = pages(format!(
        "repos/{repository}/issues/comments?sort=updated&direction=asc&per_page=100{since}"
    ))?;
    // Inline-comment edits may need discovery independently of the issue's
    // updated_at. This feed adds candidates, then each candidate is fully
    // paginated below so replies and earlier context remain available.
    let changed_inline = pages(format!(
        "repos/{repository}/pulls/comments?sort=updated&direction=asc&per_page=100{since}"
    ))?;
    let mut review_candidates = BTreeSet::new();
    for comment in changed_inline {
        let prefix = format!(
            "https://api.github.com/repos/{}/pulls/",
            text(&repo, "full_name")?
        );
        let number = text(&comment, "pull_request_url")?
            .strip_prefix(&prefix)
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|n| *n > 0)
            .ok_or_else(|| {
                invalid("inline comment points outside this repository or lacks PR identity")
            })?;
        review_candidates.insert(number);
    }
    let mut records = BTreeMap::new();
    for issue in issues {
        let number = issue["number"]
            .as_u64()
            .ok_or_else(|| invalid("issue lacks number"))?;
        let kind = if issue.get("pull_request").is_some() {
            review_candidates.insert(number);
            "pull_request_snapshot"
        } else {
            "issue_snapshot"
        };
        let value = record(
            kind,
            format!("#{number} {}", text(&issue, "title")?),
            serde_json::to_string_pretty(&issue)?,
            text(&issue, "html_url")?.into(),
            text(&issue, "updated_at")?.into(),
            "NATIVE_SNAPSHOT",
            &now,
        )?;
        records.insert(value.id.clone(), value);
    }
    let mut submitted_reviews = 0usize;
    let mut inline_review_comments = 0usize;
    let mut pending_reviews_excluded = 0usize;
    for number in &review_candidates {
        for review in pages(format!(
            "repos/{repository}/pulls/{number}/reviews?per_page=100"
        ))? {
            if review["state"] == "PENDING" || review["submitted_at"].is_null() {
                pending_reviews_excluded += 1;
                continue;
            }
            let value = review_record(repository, *number, "pull_request_review", &review, &now)?;
            if records.insert(value.id.clone(), value).is_none() {
                submitted_reviews += 1;
            }
        }
        for comment in pages(format!(
            "repos/{repository}/pulls/{number}/comments?per_page=100"
        ))? {
            let value = review_record(
                repository,
                *number,
                "pull_request_review_comment",
                &comment,
                &now,
            )?;
            if records.insert(value.id.clone(), value).is_none() {
                inline_review_comments += 1;
            }
        }
    }
    for comment in comments {
        let id = comment["id"]
            .as_u64()
            .ok_or_else(|| invalid("comment lacks id"))?;
        let value = record(
            "issue_comment",
            format!("GitHub comment {id}"),
            text(&comment, "body")?.into(),
            text(&comment, "html_url")?.into(),
            text(&comment, "updated_at")?.into(),
            "NATIVE_SNAPSHOT",
            &now,
        )?;
        records.insert(value.id.clone(), value);
    }
    for path in [
        "AGENTS.md",
        "README.md",
        "ROADMAP.md",
        "STATUS.md",
        "docs/integration/agent-execution-policy.md",
        "docs/integration/DECISIONS.md",
        "docs/integration/current-state.md",
        "docs/integration/project-track.md",
    ] {
        let raw = gh(&[
            "api".into(),
            format!("repos/{repository}/contents/{path}?ref={main_sha}"),
            "-H".into(),
            "Accept: application/vnd.github.raw+json".into(),
        ])?;
        let body =
            String::from_utf8(raw).map_err(|_| invalid("authority document is not UTF-8"))?;
        let value = record(
            "accepted_authority",
            path.into(),
            body,
            format!("https://github.com/{repository}/blob/{main_sha}/{path}"),
            main_sha.clone(),
            "PINNED_REPOSITORY_DOCUMENT",
            &now,
        )?;
        records.insert(value.id.clone(), value);
    }
    let server_finish = crate::coord::github_time(repository)?;
    if server_finish < server_start {
        return Err(invalid(
            "GitHub clock regressed during collection; cursor not advanced",
        ));
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| invalid("system clock predates epoch"))?
        .as_nanos();
    let output = output_root.join(format!("snapshot-{nanos}-{}", std::process::id()));
    fs::create_dir(&output)?;
    let mut jsonl = Vec::new();
    for value in records.values() {
        serde_json::to_writer(&mut jsonl, value)?;
        jsonl.push(b'\n');
    }
    let receipt=SyncReceipt{schema:"uor-r4.github-sync/2".into(),repository:repository.into(),collected_at:now.clone(),main_sha,records:records.len(),import_sha256:sha256(&jsonl),output:output.clone(),previous_since:previous.map(|c|c.since),queried_since:query_since,next_since:next_since.clone(),server_started_epoch:server_start,server_finished_epoch:server_finish,cursor_recovery,coverage_complete:false,review_pull_requests:review_candidates.into_iter().collect(),submitted_reviews,inline_review_comments,pending_reviews_excluded,coverage:vec!["Paginated REST issue records and PR issue-body records, including closed records.".into(),"Paginated ordinary issue/PR conversation comments, including edited comments.".into(),"Paginated submitted reviews (including observed approval, changes-requested and dismissed states) and inline review comments for every PR discovered through the changed/all-state issue list or changed inline-comment feed; closed PRs are included.".into(),"Review bodies retain stable source/supersession identities, exact native API records, timestamps and content-bound revisions. Collection freshness is bounded by server_started_epoch and server_finished_epoch.".into(),"Eight accepted authority documents pinned to main's observed SHA.".into(),"Not included: unsubmitted/pending reviews, deleted or never-observed revisions/comments, GraphQL thread resolved/outdated state, review-request timeline, check logs, binary artifacts or full Git history. Review updates that do not change candidate discovery timestamps need a periodic full reconciliation.".into(),"REST pagination is not a transactional snapshot. A 300-second overlap reduces timing races; coverage_complete remains false and periodic full reconciliation remains necessary.".into()],github_mutations:0};
    crate::atomic_write(&output.join("public.jsonl"), &jsonl)?;
    crate::atomic_write(
        &output.join("sync-receipt.json"),
        &serde_json::to_vec_pretty(&receipt)?,
    )?;
    // Advance only after every page/document and both durable output files succeed.
    crate::atomic_write(
        &cursor_path,
        &serde_json::to_vec_pretty(&Cursor {
            schema: "uor-r4.github-sync-cursor/2".into(),
            repository: repository.into(),
            since: next_since,
            snapshot: output
                .file_name()
                .and_then(|v| v.to_str())
                .ok_or_else(|| invalid("snapshot path invalid"))?
                .into(),
        })?,
    )?;
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cursor(schema: &str, since: &str) -> Cursor {
        Cursor {
            schema: schema.into(),
            repository: "o/r".into(),
            since: since.into(),
            snapshot: "prior".into(),
        }
    }
    #[test]
    fn server_window_overlaps_and_future_restores_force_full_reconciliation() {
        let now = parse_iso("2026-09-29T20:05:25Z").unwrap();
        assert_eq!(iso(now).unwrap(), "2026-09-29T20:05:25Z");
        let good = cursor("uor-r4.github-sync-cursor/2", "2026-09-29T19:00:00Z");
        let (query, next, recovery) = cursor_window(Some(&good), now).unwrap();
        assert_eq!(query, Some(good.since));
        assert_eq!(next, "2026-09-29T20:00:25Z");
        assert_eq!(recovery, "server_cursor_with_300s_overlap");
        let future = cursor("uor-r4.github-sync-cursor/2", "2099-01-01T00:00:00Z");
        let (query, next, recovery) = cursor_window(Some(&future), now).unwrap();
        assert!(query.is_none());
        assert_eq!(next, "2026-09-29T20:00:25Z");
        assert_eq!(recovery, "future_cursor_full_rescan");
        let old = cursor("uor-r4.github-sync-cursor/1", "2026-09-29T19:00:00Z");
        assert!(cursor_window(Some(&old), now).unwrap().0.is_none());
        let malformed = cursor("uor-r4.github-sync-cursor/2", "2026-02-31T00:00:00Z");
        assert!(cursor_window(Some(&malformed), now).unwrap().0.is_none());
    }
    #[test]
    fn identity_is_stable_across_collection_times_and_changes_on_edit() {
        let a = record(
            "issue",
            "one".into(),
            "body".into(),
            "https://github.com/o/r/issues/1".into(),
            "revision".into(),
            "NATIVE_SNAPSHOT",
            "t1",
        )
        .unwrap();
        let b = record(
            "issue",
            "one".into(),
            "body".into(),
            "https://github.com/o/r/issues/1".into(),
            "revision".into(),
            "NATIVE_SNAPSHOT",
            "t2",
        )
        .unwrap();
        assert_eq!(a.id, b.id);
        let c = record(
            "issue",
            "one".into(),
            "edited".into(),
            "https://github.com/o/r/issues/1".into(),
            "revision".into(),
            "NATIVE_SNAPSHOT",
            "t2",
        )
        .unwrap();
        assert_ne!(a.id, c.id);
        let serialized = serde_json::to_value(a).unwrap();
        assert_eq!(serialized.as_object().unwrap().len(), 10);
    }
}
