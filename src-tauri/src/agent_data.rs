//! Grant-bounded companion data APIs for the Studio agent host.
//!
//! Directory and Git reads are rooted at an existing canonicalized
//! [`DesktopBackend`] grant. Artifacts are content addressed in the existing
//! object store and are read in bounded ranges with a digest and size that the
//! browser verifies again. No API returns an artifact URL, token or expiring
//! link, and no API owns task state.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uor_r4_object_store::Manifest;

use crate::{
    ensure_git_repository_root, hardened_git_command, lock_git_mutation_guard, reject_if_symlink,
    reject_parent_dir_traversal, reject_symlinks_between_root_and_target, run_bounded_process,
    DesktopBackend, DesktopClientError, MAX_PROCESS_OUTPUT_BYTES, PROCESS_TIMEOUT,
};

/// Maximum directory entries returned by one `agent_list_directory` call.
pub const MAX_AGENT_TREE_ENTRIES: usize = 512;

/// Maximum changed files returned by one `agent_git_diff` call.
pub const MAX_AGENT_DIFF_FILES: usize = 256;

/// Maximum whole artifact accepted by the host artifact registry (8 MiB).
pub const MAX_AGENT_ARTIFACT_BYTES: u64 = 8 * 1024 * 1024;

/// Maximum bytes returned by one ranged artifact read (1 MiB).
pub const MAX_AGENT_ARTIFACT_RANGE_BYTES: u64 = 1024 * 1024;

/// Maximum number of content-addressed artifacts retained by one backend.
pub const MAX_AGENT_ARTIFACTS: usize = 128;

/// Domain separation for the branch head digest exposed to the browser.
pub const AGENT_HEAD_DIGEST_DOMAIN: &str = "uor-r4.studio.branch-head/1\n";

const GRANT_DOMAIN: &str = "uor-r4.studio.agent-grant/1\n";
const GRANT_PREFIX: &str = "gr_";
const ARTIFACT_PREFIX: &str = "art_";

/// An active canonical grant exposed to the companion data APIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGrant {
    pub grant_id: String,
    pub path: String,
}

/// One explicit node in a bounded directory listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentFileNode {
    pub path: String,
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

/// Bounded immediate children of one grant-relative directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDirectoryPage {
    pub grant_id: String,
    pub path: String,
    pub entries: Vec<AgentFileNode>,
    pub truncated: bool,
    pub symlinks_skipped: usize,
    pub read_errors: usize,
}

/// One changed file in the branch snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentBranchFile {
    pub path: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_path: Option<String>,
}

/// Branch-only change snapshot; `last_turn` is never produced by the host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentChangeSnapshot {
    pub scope: String,
    pub files: Vec<AgentBranchFile>,
    pub head_digest: String,
    pub head_commit: String,
    pub base_ref: String,
    pub captured_at_ms: u64,
    pub truncated: bool,
}

/// Inclusive-start/exclusive-end byte range for an artifact read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentArtifactRange {
    pub start: u64,
    pub end: u64,
}

/// Host-side artifact metadata. It carries no URL or bearer credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentArtifactInfo {
    pub artifact_id: String,
    pub path: String,
    pub media_type: String,
    pub size_bytes: u64,
    pub digest: String,
}

/// Digest-verified artifact bytes returned by `agent_read_artifact`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentArtifactBytes {
    pub artifact_id: String,
    pub path: String,
    pub media_type: String,
    /// Number of bytes returned, not necessarily the whole artifact size.
    pub size_bytes: u64,
    pub artifact_size_bytes: u64,
    /// SHA-256 of the returned bytes.
    pub digest: String,
    /// SHA-256 of the whole artifact recorded at registration.
    pub whole_digest: String,
    pub bytes_base64: String,
    pub range: AgentArtifactRange,
}

#[derive(Debug, Clone)]
pub(crate) struct AgentArtifactRecord {
    pub(crate) info: AgentArtifactInfo,
    pub(crate) manifest: Manifest,
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{}", crate::sha256_bytes(bytes))
}

fn grant_id_for_path(canonical: &Path) -> String {
    let payload = format!("{GRANT_DOMAIN}{}", canonical.to_string_lossy());
    let digest = crate::sha256_bytes(payload.as_bytes());
    format!("{GRANT_PREFIX}{}", &digest[..16])
}

fn is_lower_hex_16(value: &str) -> bool {
    value.len() == 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Serialize the read-only grant list for the existing desktop backend.
pub fn agent_grants_value(
    backend: &DesktopBackend,
) -> Result<serde_json::Value, DesktopClientError> {
    serde_json::to_value(backend.agent_grants())
        .map_err(|error| DesktopClientError::InvalidInput(error.to_string()))
}

/// Serialize the branch-only change snapshot in the shape the browser parser
/// expects: `{ "snapshot": AgentChangeSnapshot }`.
pub fn agent_git_diff_output(
    backend: &DesktopBackend,
    grant_id: &str,
    branch_scope: &str,
) -> Result<serde_json::Value, DesktopClientError> {
    let snapshot = backend.agent_git_diff(grant_id, branch_scope)?;
    serde_json::to_value(serde_json::json!({ "snapshot": snapshot }))
        .map_err(|error| DesktopClientError::InvalidInput(error.to_string()))
}

fn validate_media_type(value: &str) -> Result<String, DesktopClientError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.len() > 128
        || trimmed.chars().any(char::is_control)
        || !trimmed.contains('/')
    {
        return Err(DesktopClientError::InvalidInput(
            "artifact media type must be a bounded type/subtype".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = u32::from(chunk[0]);
        let second = u32::from(chunk.get(1).copied().unwrap_or(0));
        let third = u32::from(chunk.get(2).copied().unwrap_or(0));
        let combined = (first << 16) | (second << 8) | third;
        out.push(TABLE[((combined >> 18) & 63) as usize] as char);
        out.push(TABLE[((combined >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((combined >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(combined & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn status_word(raw: &str) -> String {
    match raw.chars().next() {
        Some('A') => "added",
        Some('M') => "modified",
        Some('D') => "deleted",
        Some('R') => "renamed",
        Some('C') => "copied",
        Some('T') => "type_changed",
        Some('U') => "unmerged",
        _ => "modified",
    }
    .to_string()
}

fn run_git_text(repo: &Path, args: &[&str]) -> Result<String, DesktopClientError> {
    let output = run_bounded_process(
        hardened_git_command(repo).args(args),
        PROCESS_TIMEOUT,
        MAX_PROCESS_OUTPUT_BYTES,
    )?;
    if !output.status.success() || output.truncated || output.timed_out {
        return Err(DesktopClientError::CommandFailed(format!(
            "Git command failed: git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_git_optional(repo: &Path, args: &[&str]) -> Result<Option<String>, DesktopClientError> {
    let output = run_bounded_process(
        hardened_git_command(repo).args(args),
        PROCESS_TIMEOUT,
        MAX_PROCESS_OUTPUT_BYTES,
    )?;
    if output.status.success() && !output.truncated && !output.timed_out {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    } else {
        Ok(None)
    }
}

fn agent_repository_root(grant_root: &Path) -> Result<PathBuf, DesktopClientError> {
    let output = run_bounded_process(
        hardened_git_command(grant_root).args(["rev-parse", "--show-toplevel"]),
        PROCESS_TIMEOUT,
        MAX_PROCESS_OUTPUT_BYTES,
    )?;
    if !output.status.success() || output.truncated || output.timed_out {
        return Err(DesktopClientError::PathNotGranted(
            "grant is not a Git repository".to_string(),
        ));
    }
    let top = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let canonical = fs::canonicalize(&top).map_err(|_| {
        DesktopClientError::PathNotGranted("Git repository root is unavailable".to_string())
    })?;
    if !canonical.starts_with(grant_root) {
        return Err(DesktopClientError::PathNotGranted(
            "Git repository discovery escapes the granted directory".to_string(),
        ));
    }
    ensure_git_repository_root(&canonical)?;
    Ok(canonical)
}

fn agent_base_ref(repo: &Path) -> Result<String, DesktopClientError> {
    if let Some(reference) = run_git_optional(
        repo,
        &["symbolic-ref", "--quiet", "refs/remotes/origin/HEAD"],
    )? {
        if let Some(name) = reference.strip_prefix("refs/remotes/") {
            if !name.is_empty() {
                return Ok(name.to_string());
            }
        }
    }
    for candidate in ["main", "master", "trunk", "develop"] {
        let reference = format!("refs/heads/{candidate}");
        if run_git_optional(repo, &["rev-parse", "--verify", "--quiet", &reference])?.is_some() {
            return Ok(candidate.to_string());
        }
    }
    run_git_text(repo, &["symbolic-ref", "--short", "--quiet", "HEAD"])
}

fn parse_name_status(stdout: &[u8], files: &mut Vec<AgentBranchFile>, truncated: &mut bool) {
    let mut offset = 0usize;
    while offset < stdout.len() {
        let Some(status_end) = stdout[offset..].iter().position(|byte| *byte == 0) else {
            *truncated = true;
            break;
        };
        let status_end = offset + status_end;
        let status = match std::str::from_utf8(&stdout[offset..status_end]) {
            Ok(value) => value.to_string(),
            Err(_) => {
                *truncated = true;
                break;
            }
        };
        offset = status_end + 1;
        let Some(path_end) = stdout[offset..].iter().position(|byte| *byte == 0) else {
            *truncated = true;
            break;
        };
        let path_end = offset + path_end;
        let first_path = match std::str::from_utf8(&stdout[offset..path_end]) {
            Ok(value) => value.to_string(),
            Err(_) => {
                *truncated = true;
                break;
            }
        };
        offset = path_end + 1;
        let renamed = status.starts_with('R') || status.starts_with('C');
        let (path, previous_path) = if renamed {
            let Some(second_end) = stdout[offset..].iter().position(|byte| *byte == 0) else {
                *truncated = true;
                break;
            };
            let second_end = offset + second_end;
            let second_path = match std::str::from_utf8(&stdout[offset..second_end]) {
                Ok(value) => value.to_string(),
                Err(_) => {
                    *truncated = true;
                    break;
                }
            };
            offset = second_end + 1;
            (second_path, Some(first_path))
        } else {
            (first_path, None)
        };
        if files.len() >= MAX_AGENT_DIFF_FILES {
            *truncated = true;
            break;
        }
        files.push(AgentBranchFile {
            path,
            status: status_word(&status),
            previous_path,
        });
    }
}

impl DesktopBackend {
    /// List the active canonical grants with opaque, stable identities.
    pub fn agent_grants(&self) -> Vec<AgentGrant> {
        self.granted_folders()
            .iter()
            .map(|path| AgentGrant {
                grant_id: grant_id_for_path(path),
                path: path.to_string_lossy().into_owned(),
            })
            .collect()
    }

    /// Return the opaque identity of one exact canonical grant root.
    pub fn agent_grant_id(&self, path: &Path) -> Result<String, DesktopClientError> {
        let canonical = self.verify_granted_existing_path(path)?;
        if !self.granted_folders().contains(&canonical) {
            return Err(DesktopClientError::PathNotGranted(
                "path is not a grant root".to_string(),
            ));
        }
        Ok(grant_id_for_path(&canonical))
    }

    /// Resolve an opaque grant identity to its canonical root.
    pub fn agent_grant_path(&self, grant_id: &str) -> Result<PathBuf, DesktopClientError> {
        let Some(hex) = grant_id.strip_prefix(GRANT_PREFIX) else {
            return Err(DesktopClientError::PathNotGranted(
                "grant identity is malformed".to_string(),
            ));
        };
        if !is_lower_hex_16(hex) {
            return Err(DesktopClientError::PathNotGranted(
                "grant identity is malformed".to_string(),
            ));
        }
        self.granted_folders()
            .iter()
            .find(|path| grant_id_for_path(path) == grant_id)
            .cloned()
            .ok_or_else(|| {
                DesktopClientError::PathNotGranted(
                    "grant identity is not an active grant".to_string(),
                )
            })
    }

    fn resolve_agent_path(
        &self,
        grant_id: &str,
        relative: &str,
    ) -> Result<(PathBuf, PathBuf, String), DesktopClientError> {
        let root = self.agent_grant_path(grant_id)?;
        let requested = Path::new(relative);
        if requested.is_absolute()
            || requested
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(DesktopClientError::PathTraversalRefused(
                "path is outside the granted folder".to_string(),
            ));
        }
        let joined = if relative.is_empty() {
            root.clone()
        } else {
            root.join(requested)
        };
        reject_parent_dir_traversal(&joined)?;
        reject_if_symlink(&joined)?;
        let canonical = fs::canonicalize(&joined).map_err(|_| {
            DesktopClientError::PathNotGranted(
                "path does not exist inside the granted folder".to_string(),
            )
        })?;
        if !canonical.starts_with(&root) {
            return Err(DesktopClientError::PathNotGranted(
                "access outside granted folders is refused".to_string(),
            ));
        }
        reject_symlinks_between_root_and_target(&root, &joined)?;
        let normalized = canonical
            .strip_prefix(&root)
            .map_err(|_| DesktopClientError::PathNotGranted("path escaped its grant".to_string()))?
            .components()
            .filter_map(|component| match component {
                Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("/");
        Ok((root, canonical, normalized))
    }

    /// Return the bounded immediate children of one grant-relative directory.
    pub fn agent_list_directory(
        &self,
        grant_id: &str,
        path: &str,
    ) -> Result<AgentDirectoryPage, DesktopClientError> {
        let (root, directory, relative) = self.resolve_agent_path(grant_id, path)?;
        let metadata = fs::symlink_metadata(&directory)
            .map_err(|error| DesktopClientError::Io(error.to_string()))?;
        if !metadata.is_dir() {
            return Err(DesktopClientError::InvalidInput(
                "directory listing target is not a directory".to_string(),
            ));
        }
        let mut retained: BTreeMap<PathBuf, AgentFileNode> = BTreeMap::new();
        let mut truncated = false;
        let mut symlinks_skipped = 0usize;
        let mut read_errors = 0usize;
        for entry in
            fs::read_dir(&directory).map_err(|error| DesktopClientError::Io(error.to_string()))?
        {
            let child = match entry {
                Ok(entry) => entry.path(),
                Err(_) => {
                    read_errors += 1;
                    continue;
                }
            };
            let Some(name) = child.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.starts_with('.') || matches!(name, "node_modules" | "target" | "dist" | "local")
            {
                continue;
            }
            let name = name.to_string();
            let metadata = match fs::symlink_metadata(&child) {
                Ok(metadata) => metadata,
                Err(_) => {
                    read_errors += 1;
                    continue;
                }
            };
            if metadata.file_type().is_symlink() {
                symlinks_skipped += 1;
                continue;
            }
            let kind = if metadata.is_dir() {
                "directory"
            } else if metadata.is_file() {
                "file"
            } else {
                continue;
            };
            let relative_path = match child.strip_prefix(&root) {
                Ok(value) => value
                    .components()
                    .map(|component| component.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/"),
                Err(_) => {
                    read_errors += 1;
                    continue;
                }
            };
            retained.insert(
                child,
                AgentFileNode {
                    path: relative_path,
                    name,
                    kind: kind.to_string(),
                    size_bytes: if metadata.is_file() {
                        Some(metadata.len())
                    } else {
                        None
                    },
                },
            );
            if retained.len() > MAX_AGENT_TREE_ENTRIES {
                retained.pop_last();
                truncated = true;
            }
        }
        let entries: Vec<AgentFileNode> = retained.into_values().collect();
        Ok(AgentDirectoryPage {
            grant_id: grant_id.to_string(),
            path: relative,
            entries,
            truncated,
            symlinks_skipped,
            read_errors,
        })
    }

    /// Return a bounded branch-only change snapshot tied to the actual HEAD.
    ///
    /// `last_turn` changes are browser-owned and intentionally refused here.
    /// The grant root must be the repository root or contain it; a grant
    /// rooted inside a repository is refused because the Git top-level would
    /// be outside the grant.
    pub fn agent_git_diff(
        &self,
        grant_id: &str,
        branch_scope: &str,
    ) -> Result<AgentChangeSnapshot, DesktopClientError> {
        if branch_scope != "branch" {
            return Err(DesktopClientError::InvalidInput(
                "agent_git_diff is branch-only; last-turn changes come from the browser changes store"
                    .to_string(),
            ));
        }
        let grant_root = self.agent_grant_path(grant_id)?;
        let repo = agent_repository_root(&grant_root)?;
        let _git_guard = lock_git_mutation_guard()?;
        let head_commit = run_git_text(&repo, &["rev-parse", "--verify", "HEAD"])?;
        if head_commit.is_empty() {
            return Err(DesktopClientError::CommandFailed(
                "branch head does not exist".to_string(),
            ));
        }
        let base_ref = agent_base_ref(&repo)?;
        let current = run_git_optional(&repo, &["symbolic-ref", "--short", "--quiet", "HEAD"])?;
        let mut files = Vec::new();
        let mut truncated = false;
        if current.as_deref() != Some(base_ref.as_str()) {
            let merge_base = run_git_optional(&repo, &["merge-base", &base_ref, "HEAD"])?
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| base_ref.clone());
            let output = run_bounded_process(
                hardened_git_command(&repo).args([
                    "diff",
                    "--no-ext-diff",
                    "--no-textconv",
                    "--no-color",
                    "--name-status",
                    "-z",
                    "--find-renames",
                    &merge_base,
                    "HEAD",
                    "--",
                ]),
                PROCESS_TIMEOUT,
                MAX_PROCESS_OUTPUT_BYTES,
            )?;
            if !output.status.success() && !output.truncated && !output.timed_out {
                return Err(DesktopClientError::CommandFailed(format!(
                    "Git branch diff failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                )));
            }
            truncated = output.truncated || output.timed_out;
            parse_name_status(&output.stdout, &mut files, &mut truncated);
        }
        Ok(AgentChangeSnapshot {
            scope: "branch".to_string(),
            files,
            head_digest: sha256_prefixed(
                format!("{AGENT_HEAD_DIGEST_DOMAIN}{head_commit}").as_bytes(),
            ),
            head_commit,
            base_ref,
            captured_at_ms: now_ms(),
            truncated,
        })
    }

    /// Register one grant-bounded file as a content-addressed host artifact.
    ///
    /// This is host-owned: it is not exposed as a page command. Task 17's
    /// approved tool dispatch registers artifacts produced by file tools.
    pub fn agent_register_artifact(
        &mut self,
        grant_id: &str,
        path: &str,
        media_type: &str,
    ) -> Result<AgentArtifactInfo, DesktopClientError> {
        let (_root, file, relative) = self.resolve_agent_path(grant_id, path)?;
        let metadata = fs::symlink_metadata(&file)
            .map_err(|error| DesktopClientError::Io(error.to_string()))?;
        if !metadata.is_file() {
            return Err(DesktopClientError::InvalidInput(
                "artifact path is not a file".to_string(),
            ));
        }
        if metadata.len() > MAX_AGENT_ARTIFACT_BYTES {
            return Err(DesktopClientError::FileTooLarge {
                bytes: metadata.len(),
                max_bytes: MAX_AGENT_ARTIFACT_BYTES,
            });
        }
        let bytes = fs::read(&file).map_err(|error| DesktopClientError::Io(error.to_string()))?;
        if bytes.len() as u64 > MAX_AGENT_ARTIFACT_BYTES {
            return Err(DesktopClientError::FileTooLarge {
                bytes: bytes.len() as u64,
                max_bytes: MAX_AGENT_ARTIFACT_BYTES,
            });
        }
        let media_type = validate_media_type(media_type)?;
        let folded = self
            .object_store
            .fold(&bytes)
            .map_err(|error| DesktopClientError::ObjectStore(error.to_string()))?;
        let digest = format!("sha256:{}", hex::encode(folded.manifest.whole_sha256()));
        let artifact_id = format!("{ARTIFACT_PREFIX}{}", &digest["sha256:".len()..]);
        if let Some(existing) = self.agent_artifacts.get(&artifact_id) {
            return Ok(existing.info.clone());
        }
        if self.agent_artifacts.len() >= MAX_AGENT_ARTIFACTS {
            return Err(DesktopClientError::InvalidInput(
                "artifact registry is full".to_string(),
            ));
        }
        let info = AgentArtifactInfo {
            artifact_id: artifact_id.clone(),
            path: relative,
            media_type,
            size_bytes: bytes.len() as u64,
            digest,
        };
        self.agent_artifacts.insert(
            artifact_id,
            AgentArtifactRecord {
                info: info.clone(),
                manifest: folded.manifest,
            },
        );
        Ok(info)
    }

    /// Read bounded artifact bytes and verify the recorded digest and size.
    ///
    /// `expected_digest` and `expected_size_bytes` refer to the whole artifact.
    /// The returned `digest`/`size_bytes` always describe the returned bytes.
    pub fn agent_read_artifact(
        &self,
        artifact_id: &str,
        range: Option<AgentArtifactRange>,
        expected_digest: Option<&str>,
        expected_size_bytes: Option<u64>,
    ) -> Result<AgentArtifactBytes, DesktopClientError> {
        let record = self.agent_artifacts.get(artifact_id).ok_or_else(|| {
            DesktopClientError::ArtifactNotFound("artifact identity is not registered".to_string())
        })?;
        if let Some(expected) = expected_digest {
            if expected != record.info.digest {
                return Err(DesktopClientError::ArtifactDigestMismatch {
                    expected: expected.to_string(),
                    actual: record.info.digest.clone(),
                });
            }
        }
        if let Some(expected) = expected_size_bytes {
            if expected != record.info.size_bytes {
                return Err(DesktopClientError::ArtifactSizeMismatch {
                    expected,
                    actual: record.info.size_bytes,
                });
            }
        }
        let total = record.info.size_bytes;
        let range = range.unwrap_or(AgentArtifactRange {
            start: 0,
            end: total,
        });
        if range.start > range.end || range.end > total {
            return Err(DesktopClientError::InvalidInput(
                "artifact range is outside the artifact".to_string(),
            ));
        }
        let length = range.end - range.start;
        if length > MAX_AGENT_ARTIFACT_RANGE_BYTES {
            return Err(DesktopClientError::FileTooLarge {
                bytes: length,
                max_bytes: MAX_AGENT_ARTIFACT_RANGE_BYTES,
            });
        }
        let verified = self
            .object_store
            .range(&record.manifest, range.start..range.end)
            .map_err(|error| DesktopClientError::ObjectStore(error.to_string()))?;
        let bytes = verified.bytes();
        if bytes.len() as u64 != length {
            return Err(DesktopClientError::ObjectStore(
                "artifact range length does not match its manifest".to_string(),
            ));
        }
        let digest = sha256_prefixed(bytes);
        if range.start == 0 && range.end == total && digest != record.info.digest {
            return Err(DesktopClientError::ArtifactDigestMismatch {
                expected: record.info.digest.clone(),
                actual: digest,
            });
        }
        Ok(AgentArtifactBytes {
            artifact_id: record.info.artifact_id.clone(),
            path: record.info.path.clone(),
            media_type: record.info.media_type.clone(),
            size_bytes: length,
            artifact_size_bytes: total,
            digest,
            whole_digest: record.info.digest.clone(),
            bytes_base64: base64_encode(bytes),
            range,
        })
    }
}
