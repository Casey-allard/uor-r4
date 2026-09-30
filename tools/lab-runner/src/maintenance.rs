//! Read-only disposal eligibility. No deletion, notice publication or Git
//! checkout/ref mutation is implemented. Unavailable observations mean SKIP.
use crate::{coord, invalid, ledger::sha256, process, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const SCHEMA: &str = "uor-r4.cleanup-manifest/1";
const MAX_FILES: u64 = 100_000;
const MAX_HASH_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DirectoryIdentity {
    pub device: u64,
    pub inode: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegenerationProof {
    pub source_head: String,
    pub tree_sha256: String,
    pub argv: Vec<String>,
    pub exit_code: i32,
    pub recorded_utc: String,
    pub log: crate::delivery::FileEvidence,
    /// Every non-source input and executable needed to reproduce the cache.
    /// This inventory is a reviewed assertion; the checker verifies its bytes.
    pub inputs: Vec<crate::delivery::FileEvidence>,
    pub complete_input_inventory: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CleanupManifest {
    pub schema: String,
    pub repository: String,
    pub host: String,
    pub task_issue: u64,
    pub board_issue: u64,
    /// merged_worktree or regenerated_cache. Names such as target prove nothing.
    pub kind: String,
    pub path: PathBuf,
    pub source_repo: PathBuf,
    pub source_head: String,
    pub identity: DirectoryIdentity,
    pub volume: crate::host::VolumePolicy,
    pub tree_sha256: String,
    pub regeneration: Option<RegenerationProof>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Notice {
    pub issue: u64,
    pub comment_id: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CleanupRequest {
    pub manifest: CleanupManifest,
    pub notices: Vec<Notice>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceDecision {
    pub status: String,
    pub eligible: bool,
    pub deletion_supported: bool,
    pub manifest_sha256: Option<String>,
    pub path: Option<PathBuf>,
    pub reasons: Vec<String>,
}

fn hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
pub fn manifest_digest(manifest: &CleanupManifest) -> Result<String> {
    Ok(sha256(&serde_json::to_vec(manifest)?))
}
pub fn directory_identity(path: &Path) -> Result<DirectoryIdentity> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || fs::canonicalize(path)? != path {
        return Err(invalid(
            "candidate must be its exact canonical directory, without aliases",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(DirectoryIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
    #[cfg(not(unix))]
    Err(invalid("directory identity unavailable on this platform"))
}

/// Hash exact content and relative names, including hidden/ignored files. A
/// linked worktree's root .git pointer is validated separately. Symlinks,
/// special files, nested repositories, hard-linked files, unreadable paths or
/// oversized inventories are ambiguous and rejected rather than traversed.
pub fn tree_digest(path: &Path, worktree: bool) -> Result<String> {
    fn walk(
        root: &Path,
        dir: &Path,
        worktree: bool,
        hash: &mut Sha256,
        files: &mut u64,
        bytes: &mut u64,
    ) -> Result<()> {
        let mut entries = fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let p = entry.path();
            if dir == root && entry.file_name() == ".git" && worktree {
                continue;
            }
            if entry.file_name() == ".git" {
                return Err(invalid("nested repository or Git data is protected"));
            }
            *files = files
                .checked_add(1)
                .ok_or_else(|| invalid("inventory overflow"))?;
            if *files > MAX_FILES {
                return Err(invalid("inventory exceeds bounded file count"));
            }
            let relative = p
                .strip_prefix(root)
                .map_err(|_| invalid("inventory path escaped"))?;
            let name = relative
                .to_str()
                .ok_or_else(|| invalid("non-UTF8 path needs manual reconciliation"))?;
            let before = fs::symlink_metadata(&p)?;
            if before.file_type().is_symlink() {
                return Err(invalid(
                    "symlink candidate needs explicit manual reconciliation",
                ));
            }
            hash.update((name.len() as u64).to_be_bytes());
            hash.update(name.as_bytes());
            if before.is_dir() {
                hash.update(b"D");
                walk(root, &p, false, hash, files, bytes)?;
            } else if before.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if before.nlink() != 1 {
                        return Err(invalid(
                            "hard-linked material is not disposable automatically",
                        ));
                    }
                }
                *bytes = bytes
                    .checked_add(before.len())
                    .ok_or_else(|| invalid("inventory overflow"))?;
                if *bytes > MAX_HASH_BYTES {
                    return Err(invalid(
                        "content inventory exceeds 512 MiB read budget; manual review required",
                    ));
                }
                hash.update(b"F");
                hash.update(before.len().to_be_bytes());
                let mut file = fs::File::open(&p)?;
                let mut buffer = [0u8; 65536];
                let mut read = 0u64;
                loop {
                    let n = file.read(&mut buffer)?;
                    if n == 0 {
                        break;
                    }
                    read += n as u64;
                    if read > before.len() {
                        return Err(invalid("candidate grew during inventory"));
                    }
                    hash.update(&buffer[..n]);
                }
                let after = fs::symlink_metadata(&p)?;
                if read != before.len()
                    || before.len() != after.len()
                    || before.modified()? != after.modified()?
                {
                    return Err(invalid("candidate changed during inventory"));
                }
            } else {
                return Err(invalid("special file in candidate"));
            }
        }
        Ok(())
    }
    let mut hash = Sha256::new();
    let mut files = 0;
    let mut bytes = 0;
    walk(path, path, worktree, &mut hash, &mut files, &mut bytes)?;
    Ok(format!("{:x}", hash.finalize()))
}
fn output(program: &str, args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new(program)
        .args(args)
        .env("LC_ALL", "C")
        .output()?;
    if !out.status.success() || !out.stderr.is_empty() {
        return Err(invalid(format!(
            "read-only {program} observation unavailable or ambiguous"
        )));
    }
    Ok(out.stdout)
}
fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()?;
    if !out.status.success() || !out.stderr.is_empty() {
        return Err(invalid(
            "Git candidate observation unavailable or ambiguous",
        ));
    }
    String::from_utf8(out.stdout)
        .map(|s| s.trim().into())
        .map_err(|_| invalid("non-UTF8 Git observation"))
}
fn gh_json(endpoint: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&output("gh", &["api", endpoint])?)?)
}
fn timestamp(value: &str) -> Result<u64> {
    if value.len() != 20
        || !value.is_ascii()
        || &value[4..5] != "-"
        || &value[7..8] != "-"
        || &value[10..11] != "T"
        || &value[13..14] != ":"
        || &value[16..17] != ":"
        || &value[19..] != "Z"
    {
        return Err(invalid("expected exact UTC timestamp"));
    }
    let n = |a: usize, b: usize| {
        value[a..b]
            .parse::<i64>()
            .map_err(|_| invalid("invalid timestamp"))
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
        return Err(invalid("invalid timestamp fields"));
    }
    let y = year - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let days = era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468;
    u64::try_from(days * 86400 + hour * 3600 + minute * 60 + second)
        .map_err(|_| invalid("invalid timestamp epoch"))
}
pub fn validate_notice(
    manifest: &CleanupManifest,
    notice: &Notice,
    comment: &Value,
    server_now: u64,
) -> Result<u64> {
    let digest = manifest_digest(manifest)?;
    let expected_issue = format!(
        "https://api.github.com/repos/{}/issues/{}",
        manifest.repository, notice.issue
    );
    if notice.issue == 0
        || notice.comment_id == 0
        || comment["id"].as_u64() != Some(notice.comment_id)
        || comment["issue_url"] != expected_issue
        || !["OWNER", "MEMBER", "COLLABORATOR"]
            .contains(&comment["author_association"].as_str().unwrap_or(""))
        || !comment["body"].as_str().is_some_and(|s| {
            s.lines()
                .any(|line| line.trim() == format!("uor-cleanup-manifest-sha256:{digest}"))
        })
    {
        return Err(invalid(
            "notice identity, author or exact manifest digest is invalid",
        ));
    }
    let created = timestamp(
        comment["created_at"]
            .as_str()
            .ok_or_else(|| invalid("notice lacks creation time"))?,
    )?;
    let updated = timestamp(
        comment["updated_at"]
            .as_str()
            .ok_or_else(|| invalid("notice lacks update time"))?,
    )?;
    if updated < created || server_now < updated || server_now - updated < 7200 {
        return Err(invalid(
            "two-hour notice period has not elapsed since its last edit",
        ));
    }
    Ok(created)
}

/// Pure fail-closed observation boundary used by acceptance fixtures as well
/// as the live checker. Reuse of a path after notice cannot inherit eligibility.
pub fn validate_observation(
    manifest: &CleanupManifest,
    identity: &DirectoryIdentity,
    head: &str,
    tree: &str,
    active: bool,
    reserved: bool,
    ambiguous: bool,
) -> Result<()> {
    if identity != &manifest.identity
        || head != manifest.source_head
        || tree != manifest.tree_sha256
    {
        return Err(invalid(
            "candidate path identity, source HEAD or content changed after notice",
        ));
    }
    if active || reserved || ambiguous {
        return Err(invalid("candidate is active, reserved or ambiguous"));
    }
    Ok(())
}
fn no_open_files(path: &Path) -> Result<()> {
    let binary = ["/usr/sbin/lsof", "/usr/bin/lsof"]
        .into_iter()
        .find(|s| Path::new(s).is_file())
        .ok_or_else(|| invalid("open-file observation unavailable"))?;
    let result = Command::new(binary)
        .args(["-nP", "+D"])
        .arg(path)
        .args(["-F", "pfn"])
        .output()?;
    if result.status.code() != Some(1) || !result.stdout.is_empty() || !result.stderr.is_empty() {
        return Err(invalid(
            "candidate has open files/process cwd or lsof could not prove inactivity",
        ));
    }
    Ok(())
}
fn source_head(manifest: &CleanupManifest) -> Result<String> {
    git(
        if manifest.kind == "merged_worktree" {
            &manifest.path
        } else {
            &manifest.source_repo
        },
        &["rev-parse", "HEAD"],
    )
}
fn merged_worktree(manifest: &CleanupManifest) -> Result<()> {
    if manifest.path == manifest.source_repo || manifest.source_repo.starts_with(&manifest.path) {
        return Err(invalid("primary/source checkout is protected"));
    }
    let pointer = fs::symlink_metadata(manifest.path.join(".git"))?;
    if !pointer.is_file() || pointer.file_type().is_symlink() {
        return Err(invalid("candidate is not an ordinary linked worktree"));
    }
    let common = fs::canonicalize(git(
        &manifest.path,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?)?;
    let source_common = fs::canonicalize(git(
        &manifest.source_repo,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?)?;
    if common != source_common {
        return Err(invalid("worktree belongs to a different source repository"));
    }
    if !git(
        &manifest.path,
        &[
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--ignored=matching",
        ],
    )?
    .is_empty()
    {
        return Err(invalid(
            "worktree has dirty, untracked or ignored material; preserve it",
        ));
    }
    let branch = git(&manifest.path, &["symbolic-ref", "--short", "HEAD"])?;
    let bytes = output(
        "gh",
        &[
            "pr",
            "list",
            "--repo",
            &manifest.repository,
            "--state",
            "merged",
            "--head",
            &branch,
            "--limit",
            "100",
            "--json",
            "headRefOid,mergeCommit",
        ],
    )?;
    let prs: Value = serde_json::from_slice(&bytes)?;
    let prs = prs
        .as_array()
        .ok_or_else(|| invalid("merged PR list unavailable"))?;
    if prs.len() >= 100 {
        return Err(invalid("merged PR list may be truncated"));
    }
    let merged = prs
        .iter()
        .find(|p| p["headRefOid"] == manifest.source_head)
        .and_then(|p| p["mergeCommit"]["oid"].as_str())
        .filter(|s| hex(s, 40))
        .ok_or_else(|| invalid("exact worktree HEAD is unpushed or has no observed merged PR"))?;
    let comparison = gh_json(&format!(
        "repos/{}/compare/{merged}...main",
        manifest.repository
    ))?;
    if !["ahead", "identical"].contains(&comparison["status"].as_str().unwrap_or("")) {
        return Err(invalid(
            "merged worktree result is not retained by current main",
        ));
    }
    Ok(())
}
fn retained_digest(path: &Path, candidate: &Path, limit: u64, nonempty: bool) -> Result<String> {
    if !path.is_absolute() || fs::canonicalize(path)? != path || path.starts_with(candidate) {
        return Err(invalid(
            "retained evidence must use its canonical path outside disposal scope",
        ));
    }
    let before = fs::symlink_metadata(path)?;
    if !before.is_file()
        || before.file_type().is_symlink()
        || before.len() > limit
        || (nonempty && before.len() == 0)
    {
        return Err(invalid("retained evidence is not a bounded regular file"));
    }
    let mut file = fs::File::open(path)?;
    let opened = file.metadata()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != opened.dev() || before.ino() != opened.ino() {
            return Err(invalid("retained file identity changed before read"));
        }
    }
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    let mut count = 0u64;
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > before.len() {
            return Err(invalid("retained file grew while reading"));
        }
        hash.update(&buffer[..n]);
    }
    let after = fs::symlink_metadata(path)?;
    if fs::canonicalize(path)? != path
        || !after.is_file()
        || after.file_type().is_symlink()
        || count != before.len()
        || before.len() != after.len()
        || before.modified()? != after.modified()?
    {
        return Err(invalid("retained file changed while reading"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != after.dev() || before.ino() != after.ino() {
            return Err(invalid("retained file identity changed after read"));
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn regenerated_cache(manifest: &CleanupManifest, notice_created: u64) -> Result<()> {
    if manifest.path == manifest.source_repo || manifest.source_repo.starts_with(&manifest.path) {
        return Err(invalid("cache overlaps protected source repository root"));
    }
    let proof = manifest.regeneration.as_ref().ok_or_else(|| {
        invalid("cache needs prior regeneration proof; a target/cache name is insufficient")
    })?;
    if !git(
        &manifest.source_repo,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?
    .is_empty()
    {
        return Err(invalid(
            "regeneration source has dirty/untracked inputs not bound by source HEAD",
        ));
    }
    if proof.source_head != manifest.source_head
        || proof.tree_sha256 != manifest.tree_sha256
        || proof.argv.first().is_none_or(|s| s.trim().is_empty())
        || proof.exit_code != 0
        || !proof.complete_input_inventory
        || !proof.argv.first().is_some_and(|executable| {
            Path::new(executable).is_absolute()
                && proof
                    .inputs
                    .iter()
                    .any(|input| input.path == Path::new(executable))
        })
        || timestamp(&proof.recorded_utc)? > notice_created
    {
        return Err(invalid(
            "cache regeneration proof is missing, changed or later than notice",
        ));
    }
    for input in &proof.inputs {
        if retained_digest(&input.path, &manifest.path, MAX_HASH_BYTES, false)? != input.sha256 {
            return Err(invalid(
                "regeneration input is missing, changed, oversized or inside disposal scope",
            ));
        }
    }
    if retained_digest(&proof.log.path, &manifest.path, 1024 * 1024, true)? != proof.log.sha256 {
        return Err(invalid(
            "cache regeneration log is unavailable, inside disposal path or changed",
        ));
    }
    // This is a recorded regeneration claim, not proof manufactured from path
    // names. Its exact output tree and original source must remain unchanged.
    Ok(())
}
fn inspect(request: &CleanupRequest, store: &Path) -> Result<()> {
    let m = &request.manifest;
    let parts: Vec<_> = m.repository.split('/').collect();
    if m.schema != SCHEMA
        || m.task_issue == 0
        || m.board_issue == 0
        || m.board_issue == 820
        || !["merged_worktree", "regenerated_cache"].contains(&m.kind.as_str())
        || !hex(&m.source_head, 40)
        || !hex(&m.tree_sha256, 64)
        || parts.len() != 2
        || parts.iter().any(|s| {
            s.is_empty()
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
    {
        return Err(invalid("malformed cleanup manifest"));
    }
    if m.host != process::host_id()?
        || !m.source_repo.is_absolute()
        || fs::canonicalize(&m.source_repo)? != m.source_repo
        || !m.path.is_absolute()
        || m.path.components().count() < 4
    {
        return Err(invalid(
            "manifest host/source/path identity is unsafe or unavailable",
        ));
    }
    if !m.path.starts_with(&m.volume.path)
        || m.volume.sentinel.starts_with(&m.path)
        || fs::canonicalize(&m.volume.sentinel)? != m.volume.sentinel
        || fs::canonicalize(&m.volume.path)? != m.volume.path
        || (m.volume.path.starts_with("/Volumes") && m.volume.volume_uuid.is_none())
    {
        return Err(invalid(
            "candidate volume identity is absent or inconsistent",
        ));
    }
    crate::host::verify_volume(&m.volume)?;
    let identity = directory_identity(&m.path)?;
    let head = source_head(m)?;
    if identity != m.identity || head != m.source_head {
        return Err(invalid("candidate changed since manifest"));
    }
    let state = coord::status(store)?;
    if state.repository != m.repository {
        return Err(invalid("coordination repository mismatch"));
    }
    if !state
        .tasks
        .get(&m.task_issue)
        .is_some_and(|t| ["complete", "released"].contains(&t.phase.as_str()))
    {
        return Err(invalid(
            "candidate task is claimed, unknown or requires reconciliation",
        ));
    }
    // An offline reservation still owns its resources. Conservative host-wide
    // exclusion also catches writers whose future output path is not yet open.
    if state
        .attempts
        .values()
        .any(|a| a.host == m.host && a.phase != "finalized")
    {
        return Err(invalid("host has an unresolved active/offline reservation"));
    }
    let now = coord::github_time(&m.repository)?;
    if request.notices.len() != 2
        || !request.notices.iter().any(|n| n.issue == 820)
        || !request.notices.iter().any(|n| n.issue == m.board_issue)
    {
        return Err(invalid(
            "cleanup needs manifest notices on #820 and the affected board",
        ));
    }
    let mut created = now;
    for notice in &request.notices {
        let comment = gh_json(&format!(
            "repos/{}/issues/comments/{}",
            m.repository, notice.comment_id
        ))?;
        created = created.min(validate_notice(m, notice, &comment, now)?);
    }
    match m.kind.as_str() {
        "merged_worktree" => merged_worktree(m)?,
        "regenerated_cache" => regenerated_cache(m, created)?,
        _ => return Err(invalid("unsupported candidate kind")),
    }
    no_open_files(&m.path)?;
    let tree = tree_digest(&m.path, m.kind == "merged_worktree")?;
    validate_observation(
        m,
        &directory_identity(&m.path)?,
        &source_head(m)?,
        &tree,
        false,
        false,
        false,
    )?;
    // Re-read after the potentially long inventory. A newly reserved job or
    // opened file invalidates the notice candidate; no deletion follows here.
    let latest = coord::status(store)?;
    if !latest
        .tasks
        .get(&m.task_issue)
        .is_some_and(|t| ["complete", "released"].contains(&t.phase.as_str()))
    {
        return Err(invalid("candidate task became active during inventory"));
    }
    if latest
        .attempts
        .values()
        .any(|a| a.host == m.host && a.phase != "finalized")
    {
        return Err(invalid("host became reserved during inventory"));
    }
    no_open_files(&m.path)?;
    Ok(())
}

#[cfg(test)]
mod retained_tests {
    use super::*;
    #[test]
    fn retained_input_cannot_escape_scope_via_parent_traversal_or_alias() {
        let root = std::env::temp_dir().join(format!(
            "uor-retained-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let candidate = root.join("candidate");
        let safe = root.join("safe");
        fs::create_dir(&candidate).unwrap();
        fs::create_dir(&safe).unwrap();
        fs::write(candidate.join("input"), b"retained").unwrap();
        fs::write(safe.join("input"), b"retained").unwrap();
        assert!(retained_digest(&safe.join("input"), &candidate, 100, true).is_ok());
        assert!(retained_digest(&safe.join("../candidate/input"), &candidate, 100, true).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&candidate, root.join("alias")).unwrap();
            assert!(retained_digest(&root.join("alias/input"), &candidate, 100, true).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }
}
pub fn check(request_path: &Path, coord_store: &Path) -> Result<MaintenanceDecision> {
    let parsed =
        (|| -> Result<CleanupRequest> { Ok(serde_json::from_slice(&fs::read(request_path)?)?) })();
    let request = match parsed {
        Ok(value) => value,
        Err(error) => {
            return Ok(MaintenanceDecision {
                status: "SKIP".into(),
                eligible: false,
                deletion_supported: false,
                manifest_sha256: None,
                path: None,
                reasons: vec![error.to_string()],
            })
        }
    };
    let result = inspect(&request, coord_store);
    let eligible = result.is_ok();
    Ok(MaintenanceDecision{status:if eligible{"ELIGIBLE"}else{"SKIP"}.into(),eligible,deletion_supported:false,manifest_sha256:Some(manifest_digest(&request.manifest)?),path:Some(request.manifest.path.clone()),reasons:match result{Ok(())=>vec!["Read-only eligibility observed; no deletion implemented. Recheck at any future authorized execution boundary.".into()],Err(error)=>vec![error.to_string()]}})
}
