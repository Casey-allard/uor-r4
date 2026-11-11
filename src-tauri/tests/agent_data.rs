//! Host tests for the grant-bounded Studio companion data APIs.
#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};
use uor_r4_desktop_client::agent_data::{
    agent_git_diff_output, agent_grants_value, AgentArtifactRange, AgentFileNode,
    AGENT_HEAD_DIGEST_DOMAIN, MAX_AGENT_ARTIFACT_BYTES, MAX_AGENT_ARTIFACT_RANGE_BYTES,
    MAX_AGENT_DIFF_FILES, MAX_AGENT_TREE_ENTRIES,
};
use uor_r4_desktop_client::{DesktopBackend, PinRegistry};

fn unique_temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uor-agent-data-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

fn backend_with_grant(path: &Path) -> (DesktopBackend, String) {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend.native_grant_folder(path).expect("grant folder");
    let grant_id = backend
        .agent_grant_id(path)
        .expect("derive opaque grant identity");
    (backend, grant_id)
}

fn run_git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_COUNT", "0")
        .output()
        .expect("run git fixture command");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn init_repo(repo: &Path) {
    fs::create_dir_all(repo).expect("create repository directory");
    run_git(repo, &["init", "-q"]);
    run_git(repo, &["symbolic-ref", "HEAD", "refs/heads/main"]);
    run_git(repo, &["config", "user.email", "studio@example.invalid"]);
    run_git(repo, &["config", "user.name", "Studio Fixture"]);
    run_git(repo, &["config", "commit.gpgsign", "false"]);
}

fn commit_all(repo: &Path, message: &str) {
    run_git(repo, &["add", "-A"]);
    run_git(repo, &["commit", "-q", "-m", message]);
}

#[test]
fn agent_list_directory_respects_grant() {
    let root = unique_temp_dir("tree-grant");
    let other = unique_temp_dir("tree-other");
    fs::create_dir_all(root.join("src")).expect("create granted src");
    fs::write(root.join("src/a.ts"), "alpha").expect("write a");
    fs::write(root.join("src/b.ts"), "bravo!").expect("write b");
    fs::create_dir_all(other.join("src")).expect("create other src");
    fs::write(other.join("src/c.ts"), "charlie").expect("write c");

    let (mut backend, grant_id) = backend_with_grant(&root);
    let page = backend
        .agent_list_directory(&grant_id, "src")
        .expect("list granted directory");
    assert_eq!(page.path, "src");
    assert!(!page.truncated);
    assert_eq!(
        page.entries,
        vec![
            AgentFileNode {
                path: "src/a.ts".to_string(),
                name: "a.ts".to_string(),
                kind: "file".to_string(),
                size_bytes: Some(5),
            },
            AgentFileNode {
                path: "src/b.ts".to_string(),
                name: "b.ts".to_string(),
                kind: "file".to_string(),
                size_bytes: Some(6),
            },
        ]
    );

    let traversal = backend
        .agent_list_directory(&grant_id, "../other")
        .expect_err("parent traversal must be refused");
    assert!(
        format!("{traversal:?}").contains("PathTraversalRefused"),
        "unexpected traversal error: {traversal:?}"
    );

    let absolute = backend
        .agent_list_directory(&grant_id, &other.join("src").display().to_string())
        .expect_err("absolute outside path must be refused");
    assert!(
        format!("{absolute:?}").contains("PathTraversalRefused"),
        "unexpected absolute error: {absolute:?}"
    );
    assert!(
        !format!("{absolute:?}").contains(&other.join("src").display().to_string()),
        "outside canonical path must not be echoed to the page: {absolute:?}"
    );

    let unknown = backend
        .agent_list_directory("gr_0000000000000000", "src")
        .expect_err("unknown grant identity must be refused");
    assert!(
        format!("{unknown:?}").contains("PathNotGranted"),
        "unexpected unknown-grant error: {unknown:?}"
    );

    // A second grant must never widen the first grant's directory view.
    backend
        .native_grant_folder(&other)
        .expect("grant other folder");
    let other_grant = backend.agent_grant_id(&other).expect("derive other grant");
    let other_page = backend
        .agent_list_directory(&other_grant, "src")
        .expect("list other grant");
    assert_eq!(
        other_page
            .entries
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>(),
        vec!["src/c.ts"]
    );
    let still_first = backend
        .agent_list_directory(&grant_id, "src")
        .expect("list first grant after second grant");
    assert_eq!(still_first.entries.len(), 2);

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&other);
}

#[cfg(unix)]
#[test]
fn agent_list_directory_refuses_symlink() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("tree-symlink");
    let outside = unique_temp_dir("tree-symlink-outside");
    fs::create_dir_all(root.join("real")).expect("create real directory");
    fs::write(root.join("real/inside.txt"), "inside").expect("write inside");
    fs::create_dir_all(root.join("chain")).expect("create chain directory");
    fs::write(outside.join("secret.txt"), "secret").expect("write secret");
    symlink(&outside, root.join("link")).expect("symlink directory");
    symlink(outside.join("secret.txt"), root.join("leak.txt")).expect("symlink file");
    symlink(&outside, root.join("chain/escape")).expect("symlink chained escape");

    let (backend, grant_id) = backend_with_grant(&root);

    let page = backend
        .agent_list_directory(&grant_id, "")
        .expect("list granted root");
    let paths: Vec<&str> = page
        .entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    assert!(paths.contains(&"real"), "real directory must be listed");
    assert!(
        !paths.contains(&"leak.txt"),
        "symlinked file must not be listed"
    );
    assert!(
        !paths.contains(&"link"),
        "symlinked directory must not be listed"
    );
    assert_eq!(page.symlinks_skipped, 2);

    let direct = backend
        .agent_list_directory(&grant_id, "link")
        .expect_err("symlink target must be refused");
    assert!(
        format!("{direct:?}").contains("SymlinkRefused"),
        "unexpected symlink error: {direct:?}"
    );

    let chained = backend
        .agent_list_directory(&grant_id, "chain/escape")
        .expect_err("chained symlink escape must be refused");
    assert!(
        format!("{chained:?}").contains("SymlinkRefused"),
        "unexpected chained symlink error: {chained:?}"
    );

    let through_parent = backend
        .agent_list_directory(&grant_id, "link/secret.txt")
        .expect_err("a path through a symlinked parent must be refused");
    assert!(
        format!("{through_parent:?}").contains("PathNotGranted")
            || format!("{through_parent:?}").contains("SymlinkRefused"),
        "unexpected symlinked-parent error: {through_parent:?}"
    );

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
}

#[test]
fn agent_list_directory_is_bounded() {
    let root = unique_temp_dir("tree-bound");
    for index in 0..(MAX_AGENT_TREE_ENTRIES + 64) {
        fs::write(root.join(format!("file-{index:04}.txt")), "x").expect("write bounded file");
    }
    let (backend, grant_id) = backend_with_grant(&root);
    let page = backend
        .agent_list_directory(&grant_id, "")
        .expect("list bounded root");
    assert_eq!(page.entries.len(), MAX_AGENT_TREE_ENTRIES);
    assert!(page.truncated);
    assert_eq!(page.read_errors, 0);
    assert!(
        page.entries
            .windows(2)
            .all(|pair| pair[0].path.as_str() <= pair[1].path.as_str()),
        "directory entries must be deterministic and sorted"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_git_diff_is_bounded() {
    let repo = unique_temp_dir("diff-bound");
    init_repo(&repo);
    fs::write(repo.join("seed.txt"), "seed").expect("write seed");
    commit_all(&repo, "seed");
    run_git(&repo, &["checkout", "-q", "-b", "feature"]);
    fs::create_dir_all(repo.join("files")).expect("create files directory");
    for index in 0..(MAX_AGENT_DIFF_FILES + 40) {
        fs::write(repo.join(format!("files/change-{index:04}.txt")), "changed")
            .expect("write changed file");
    }
    commit_all(&repo, "bounded branch");

    let (backend, grant_id) = backend_with_grant(&repo);
    let snapshot = backend
        .agent_git_diff(&grant_id, "branch")
        .expect("branch diff");
    assert_eq!(snapshot.scope, "branch");
    assert_eq!(snapshot.files.len(), MAX_AGENT_DIFF_FILES);
    assert!(snapshot.truncated);
    assert!(
        snapshot
            .files
            .iter()
            .all(|file| file.path.starts_with("files/change-")),
        "bounded diff must only contain branch files"
    );
    assert!(snapshot.head_digest.starts_with("sha256:"));
    assert_eq!(snapshot.head_digest.len(), "sha256:".len() + 64);

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn agent_git_diff_is_branch_only() {
    let repo = unique_temp_dir("diff-branch");
    init_repo(&repo);
    fs::write(repo.join("base.txt"), "base\n").expect("write base");
    commit_all(&repo, "base");
    run_git(&repo, &["checkout", "-q", "-b", "feature"]);
    fs::write(repo.join("feature.txt"), "feature\n").expect("write feature");
    commit_all(&repo, "feature");

    // Dirty and untracked working-tree state must never become a branch diff.
    fs::write(repo.join("base.txt"), "dirty working tree\n").expect("dirty base");
    fs::write(repo.join("untracked.txt"), "untracked\n").expect("write untracked");

    let (backend, grant_id) = backend_with_grant(&repo);
    let snapshot = backend
        .agent_git_diff(&grant_id, "branch")
        .expect("branch diff");
    assert_eq!(snapshot.scope, "branch");
    assert!(!snapshot.truncated);
    assert_eq!(snapshot.base_ref, "main");
    let paths: Vec<&str> = snapshot
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    assert_eq!(paths, vec!["feature.txt"]);
    assert_eq!(snapshot.files[0].status, "added");

    let head = run_git(&repo, &["rev-parse", "HEAD"]);
    assert_eq!(snapshot.head_commit, head);
    let expected_head_digest =
        sha256_prefixed(format!("{AGENT_HEAD_DIGEST_DOMAIN}{head}").as_bytes());
    assert_eq!(snapshot.head_digest, expected_head_digest);

    let last_turn = backend
        .agent_git_diff(&grant_id, "last_turn")
        .expect_err("host branch API must never fabricate a last-turn diff");
    assert!(
        format!("{last_turn:?}").contains("InvalidInput"),
        "unexpected last-turn error: {last_turn:?}"
    );

    fs::write(repo.join("feature-two.txt"), "second\n").expect("write second feature");
    run_git(&repo, &["add", "feature-two.txt"]);
    run_git(&repo, &["commit", "-q", "-m", "feature two"]);
    let updated = backend
        .agent_git_diff(&grant_id, "branch")
        .expect("updated branch diff");
    assert_ne!(updated.head_digest, snapshot.head_digest);
    assert_eq!(
        updated
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        vec!["feature-two.txt", "feature.txt"]
    );

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn agent_read_artifact_verifies_digest() {
    let root = unique_temp_dir("artifact-digest");
    fs::write(root.join("notes.txt"), "hello artifact").expect("write artifact");

    let (mut backend, grant_id) = backend_with_grant(&root);
    let info = backend
        .agent_register_artifact(&grant_id, "notes.txt", "text/plain")
        .expect("register artifact");
    assert_eq!(info.path, "notes.txt");
    assert_eq!(info.media_type, "text/plain");
    assert_eq!(info.size_bytes, 14);
    assert_eq!(info.digest, sha256_prefixed(b"hello artifact"));

    let full = backend
        .agent_read_artifact(&info.artifact_id, None, None, None)
        .expect("read full artifact");
    assert_eq!(full.path, "notes.txt");
    assert_eq!(full.media_type, "text/plain");
    assert_eq!(full.size_bytes, 14);
    assert_eq!(full.artifact_size_bytes, 14);
    assert_eq!(full.bytes_base64, "aGVsbG8gYXJ0aWZhY3Q=");
    assert_eq!(full.digest, sha256_prefixed(b"hello artifact"));
    assert_eq!(full.whole_digest, full.digest);
    assert_eq!(full.range, AgentArtifactRange { start: 0, end: 14 });

    let partial = backend
        .agent_read_artifact(
            &info.artifact_id,
            Some(AgentArtifactRange { start: 6, end: 14 }),
            None,
            None,
        )
        .expect("read artifact range");
    assert_eq!(partial.size_bytes, 8);
    assert_eq!(partial.artifact_size_bytes, 14);
    assert_eq!(partial.bytes_base64, "YXJ0aWZhY3Q=");
    assert_eq!(partial.digest, sha256_prefixed(b"artifact"));
    assert_eq!(partial.whole_digest, sha256_prefixed(b"hello artifact"));
    assert_eq!(partial.range, AgentArtifactRange { start: 6, end: 14 });

    let wrong_digest = backend
        .agent_read_artifact(
            &info.artifact_id,
            None,
            Some("sha256:0000000000000000000000000000000000000000000000000000000000000000"),
            None,
        )
        .expect_err("expected digest mismatch must be refused");
    assert!(
        format!("{wrong_digest:?}").contains("ArtifactDigestMismatch"),
        "unexpected digest error: {wrong_digest:?}"
    );

    let wrong_size = backend
        .agent_read_artifact(&info.artifact_id, None, None, Some(13))
        .expect_err("expected size mismatch must be refused");
    assert!(
        format!("{wrong_size:?}").contains("ArtifactSizeMismatch"),
        "unexpected size error: {wrong_size:?}"
    );

    let unknown = backend
        .agent_read_artifact(&format!("art_{}", "0".repeat(64)), None, None, None)
        .expect_err("unknown artifact must be refused");
    assert!(
        format!("{unknown:?}").contains("ArtifactNotFound"),
        "unexpected unknown-artifact error: {unknown:?}"
    );

    // The artifact is content addressed: rewriting the source cannot change the
    // digest-verified host copy.
    fs::write(root.join("notes.txt"), "mutated after registration").expect("mutate source");
    let reread = backend
        .agent_read_artifact(&info.artifact_id, None, None, None)
        .expect("reread content-addressed artifact");
    assert_eq!(reread.bytes_base64, "aGVsbG8gYXJ0aWZhY3Q=");
    assert_eq!(reread.digest, info.digest);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_read_artifact_refuses_oversize() {
    let root = unique_temp_dir("artifact-oversize");

    let oversize_path = root.join("oversize.bin");
    fs::write(
        &oversize_path,
        vec![0u8; MAX_AGENT_ARTIFACT_BYTES as usize + 1],
    )
    .expect("write oversize artifact");
    let (mut backend, grant_id) = backend_with_grant(&root);
    let oversize = backend
        .agent_register_artifact(&grant_id, "oversize.bin", "application/octet-stream")
        .expect_err("oversize artifact must be refused at registration");
    let oversize_debug = format!("{oversize:?}");
    assert!(
        oversize_debug.contains("FileTooLarge"),
        "unexpected oversize error: {oversize_debug}"
    );
    assert!(
        oversize_debug.contains(&format!("{}", MAX_AGENT_ARTIFACT_BYTES + 1)),
        "oversize error must report the actual size: {oversize_debug}"
    );

    let range_path = root.join("range.bin");
    fs::write(
        &range_path,
        vec![7u8; MAX_AGENT_ARTIFACT_RANGE_BYTES as usize + 1],
    )
    .expect("write ranged artifact");
    let info = backend
        .agent_register_artifact(&grant_id, "range.bin", "application/octet-stream")
        .expect("register ranged artifact");
    let full = backend
        .agent_read_artifact(&info.artifact_id, None, None, None)
        .expect_err("a full read larger than the range cap must be refused");
    assert!(
        format!("{full:?}").contains("FileTooLarge"),
        "unexpected full-read error: {full:?}"
    );

    let partial = backend
        .agent_read_artifact(
            &info.artifact_id,
            Some(AgentArtifactRange {
                start: 0,
                end: MAX_AGENT_ARTIFACT_RANGE_BYTES,
            }),
            None,
            None,
        )
        .expect("bounded range read");
    assert_eq!(partial.size_bytes, MAX_AGENT_ARTIFACT_RANGE_BYTES);

    let outside_range = backend
        .agent_read_artifact(
            &info.artifact_id,
            Some(AgentArtifactRange {
                start: 0,
                end: MAX_AGENT_ARTIFACT_RANGE_BYTES + 1,
            }),
            None,
            None,
        )
        .expect_err("oversize range must be refused");
    assert!(
        format!("{outside_range:?}").contains("FileTooLarge"),
        "unexpected oversize range error: {outside_range:?}"
    );

    let _ = fs::remove_dir_all(&root);
}

#[cfg(unix)]
#[test]
fn agent_register_artifact_refuses_symlink_and_outside_grant() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("artifact-escape");
    let outside = unique_temp_dir("artifact-escape-outside");
    fs::write(outside.join("secret.txt"), "secret").expect("write outside secret");
    symlink(outside.join("secret.txt"), root.join("leak.txt")).expect("symlink secret");

    let (mut backend, grant_id) = backend_with_grant(&root);
    let symlinked = backend
        .agent_register_artifact(&grant_id, "leak.txt", "text/plain")
        .expect_err("symlinked artifact must be refused");
    assert!(
        format!("{symlinked:?}").contains("SymlinkRefused"),
        "unexpected symlink artifact error: {symlinked:?}"
    );

    let traversal = backend
        .agent_register_artifact(
            &grant_id,
            "../artifact-escape-outside/secret.txt",
            "text/plain",
        )
        .expect_err("outside artifact must be refused");
    assert!(
        format!("{traversal:?}").contains("PathTraversalRefused"),
        "unexpected outside artifact error: {traversal:?}"
    );

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
}

#[test]
fn agent_data_outputs_have_no_urls_or_tokens() {
    let repo = unique_temp_dir("no-url");
    init_repo(&repo);
    fs::write(repo.join("tracked.txt"), "tracked").expect("write tracked");
    commit_all(&repo, "base");
    run_git(&repo, &["checkout", "-q", "-b", "feature"]);
    fs::write(repo.join("feature.txt"), "feature").expect("write feature");
    commit_all(&repo, "feature");
    fs::write(repo.join("artifact.txt"), "artifact").expect("write artifact");

    let (mut backend, grant_id) = backend_with_grant(&repo);
    let page = backend
        .agent_list_directory(&grant_id, "")
        .expect("list root");
    let snapshot = backend
        .agent_git_diff(&grant_id, "branch")
        .expect("branch diff");
    let info = backend
        .agent_register_artifact(&grant_id, "artifact.txt", "text/plain")
        .expect("register artifact");
    let read = backend
        .agent_read_artifact(&info.artifact_id, None, None, None)
        .expect("read artifact");

    let forbidden = [
        "url",
        "artifact_url",
        "download_url",
        "link",
        "token",
        "expires_at",
    ];
    for (label, value) in [
        (
            "directory",
            serde_json::to_value(&page).expect("serialize directory"),
        ),
        (
            "diff",
            serde_json::to_value(&snapshot).expect("serialize diff"),
        ),
        (
            "artifact",
            serde_json::to_value(&read).expect("serialize artifact"),
        ),
    ] {
        let object = value.as_object().expect("serialized output object");
        for key in forbidden {
            assert!(
                !object.contains_key(key),
                "{label} output must not carry {key}: {value}"
            );
        }
    }
    let artifact_value = serde_json::to_value(&read).expect("serialize artifact");
    assert!(artifact_value.get("bytes_base64").is_some());
    assert!(artifact_value.get("digest").is_some());
    assert!(artifact_value.get("media_type").is_some());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn agent_grant_path_rejects_malformed_lowercase_hex() {
    let root = unique_temp_dir("grant-hex");
    let (backend, grant_id) = backend_with_grant(&root);
    let uppercase = "gr_ABCDEF0123456789";
    assert_ne!(
        uppercase, grant_id,
        "the literal malformed id must not alias the generated lowercase grant"
    );
    let uppercase_error = backend
        .agent_grant_path(uppercase)
        .expect_err("uppercase grant hex must be malformed");
    assert!(
        format!("{uppercase_error:?}").contains("malformed"),
        "uppercase grant id must be refused as malformed: {uppercase_error:?}"
    );
    let short = "gr_000000000000000";
    let short_error = backend
        .agent_grant_path(short)
        .expect_err("short grant id must be malformed");
    assert!(
        format!("{short_error:?}").contains("malformed"),
        "short grant id must be refused as malformed: {short_error:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_grants_command_lists_canonical_ids_and_paths() {
    let root = unique_temp_dir("grants-command");
    let nested = root.join("workspace");
    fs::create_dir_all(&nested).expect("create workspace");
    let (backend, grant_id) = backend_with_grant(&nested);

    let value = agent_grants_value(&backend).expect("serialize agent grants");
    let grants = value.as_array().expect("grant list is an array");
    assert_eq!(grants.len(), 1);
    let grant = grants[0].as_object().expect("grant record is an object");
    let listed_id = grant["grant_id"].as_str().expect("grant_id string");
    assert_eq!(listed_id, grant_id);
    assert_eq!(listed_id.len(), "gr_".len() + 16);
    assert!(
        listed_id["gr_".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "grant id must be lowercase hex: {listed_id}"
    );
    let listed_path = grant["path"].as_str().expect("grant path string");
    let canonical = fs::canonicalize(&nested).expect("canonical workspace");
    assert_eq!(listed_path, canonical.to_string_lossy());
    assert_eq!(
        backend
            .agent_grant_path(listed_id)
            .expect("round-trip grant id"),
        canonical
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn agent_git_diff_command_returns_snapshot_wrapper() {
    let repo = unique_temp_dir("diff-wrapper");
    init_repo(&repo);
    fs::write(repo.join("base.txt"), "base\n").expect("write base");
    commit_all(&repo, "base");
    run_git(&repo, &["checkout", "-q", "-b", "feature"]);
    fs::write(repo.join("feature.txt"), "feature\n").expect("write feature");
    commit_all(&repo, "feature");

    let (backend, grant_id) = backend_with_grant(&repo);
    let value = agent_git_diff_output(&backend, &grant_id, "branch")
        .expect("serialize branch snapshot wrapper");
    let object = value.as_object().expect("command output is an object");
    assert!(
        object.get("snapshot").is_some(),
        "agent_git_diff command must wrap the snapshot: {value}"
    );
    assert!(
        !object.contains_key("scope") && !object.contains_key("files"),
        "agent_git_diff must not also flatten the snapshot: {value}"
    );
    let snapshot = object["snapshot"]
        .as_object()
        .expect("snapshot wrapper value");
    assert_eq!(snapshot["scope"], "branch");
    assert_eq!(snapshot["files"][0]["path"], "feature.txt");
    assert!(snapshot["head_digest"].as_str().is_some());
    assert!(snapshot["captured_at_ms"].as_u64().is_some());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn agent_git_diff_refuses_grant_inside_repository() {
    let repo = unique_temp_dir("diff-subdir-grant");
    init_repo(&repo);
    fs::write(repo.join("base.txt"), "base\n").expect("write base");
    commit_all(&repo, "base");
    let subdir = repo.join("package");
    fs::create_dir_all(&subdir).expect("create package directory");
    let (backend, grant_id) = backend_with_grant(&subdir);

    let error = backend
        .agent_git_diff(&grant_id, "branch")
        .expect_err("a grant rooted inside a repository cannot produce a branch diff");
    assert!(
        format!("{error:?}").contains("PathNotGranted"),
        "unexpected subdirectory-grant error: {error:?}"
    );

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn agent_read_artifact_refuses_true_out_of_range() {
    let root = unique_temp_dir("artifact-range-bounds");
    fs::write(root.join("notes.txt"), "hello artifact").expect("write artifact");
    let (mut backend, grant_id) = backend_with_grant(&root);
    let info = backend
        .agent_register_artifact(&grant_id, "notes.txt", "text/plain")
        .expect("register artifact");

    let past_end = backend
        .agent_read_artifact(
            &info.artifact_id,
            Some(AgentArtifactRange { start: 0, end: 15 }),
            None,
            None,
        )
        .expect_err("end past the artifact size must be refused");
    assert!(
        format!("{past_end:?}").contains("InvalidInput"),
        "unexpected past-end error: {past_end:?}"
    );

    let reversed = backend
        .agent_read_artifact(
            &info.artifact_id,
            Some(AgentArtifactRange { start: 10, end: 5 }),
            None,
            None,
        )
        .expect_err("reversed range must be refused");
    assert!(
        format!("{reversed:?}").contains("InvalidInput"),
        "unexpected reversed-range error: {reversed:?}"
    );

    let _ = fs::remove_dir_all(&root);
}
