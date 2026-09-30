use lab_runner::maintenance::*;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
fn fixture() -> CleanupManifest {
    CleanupManifest {
        schema: SCHEMA.into(),
        repository: "UOR-Foundation/uor-r4".into(),
        host: "fixture-host".into(),
        task_issue: 1520,
        board_issue: 1515,
        kind: "merged_worktree".into(),
        path: PathBuf::from("/fixture/worktrees/merged"),
        source_repo: PathBuf::from("/fixture/source"),
        source_head: "a".repeat(40),
        identity: DirectoryIdentity {
            device: 1,
            inode: 2,
        },
        tree_sha256: "b".repeat(64),
        regeneration: None,
        volume: lab_runner::host::VolumePolicy {
            id: "fixture".into(),
            path: PathBuf::from("/fixture"),
            volume_uuid: None,
            sentinel: PathBuf::from("/fixture/sentinel"),
            sentinel_value: "fixture".into(),
            reserve_bytes: 0,
            stop_margin_bytes: 128 * 1024 * 1024,
        },
    }
}
fn comment(m: &CleanupManifest) -> serde_json::Value {
    json!({"id":123,"issue_url":"https://api.github.com/repos/UOR-Foundation/uor-r4/issues/820",
        "author_association":"COLLABORATOR","created_at":"1970-01-01T00:00:00Z","updated_at":"1970-01-01T00:00:00Z",
        "body":format!("Review before cleanup.\nuor-cleanup-manifest-sha256:{}",manifest_digest(m).unwrap())})
}
#[test]
fn notice_binds_exact_manifest_and_two_hours_since_last_edit() {
    let m = fixture();
    let n = Notice {
        issue: 820,
        comment_id: 123,
    };
    let c = comment(&m);
    assert!(validate_notice(&m, &n, &c, 7199).is_err());
    assert!(validate_notice(&m, &n, &c, 7200).is_ok());
    let mut edited = c.clone();
    edited["updated_at"] = json!("1970-01-01T01:00:00Z");
    assert!(validate_notice(&m, &n, &edited, 7200).is_err());
    let mut changed = m.clone();
    changed.path = PathBuf::from("/fixture/worktrees/new");
    assert!(validate_notice(&changed, &n, &c, 7200).is_err());
    edited = c.clone();
    edited["author_association"] = json!("NONE");
    assert!(validate_notice(&m, &n, &edited, 7200).is_err());
}
#[test]
fn changed_or_active_after_notice_is_never_eligible() {
    let m = fixture();
    assert!(validate_observation(
        &m,
        &m.identity,
        &m.source_head,
        &m.tree_sha256,
        false,
        false,
        false
    )
    .is_ok());
    let changed = DirectoryIdentity {
        device: m.identity.device,
        inode: m.identity.inode + 1,
    };
    assert!(validate_observation(
        &m,
        &changed,
        &m.source_head,
        &m.tree_sha256,
        false,
        false,
        false
    )
    .is_err());
    assert!(validate_observation(
        &m,
        &m.identity,
        &"c".repeat(40),
        &m.tree_sha256,
        false,
        false,
        false
    )
    .is_err());
    assert!(validate_observation(
        &m,
        &m.identity,
        &m.source_head,
        &"c".repeat(64),
        false,
        false,
        false
    )
    .is_err());
    for (active, reserved, ambiguous) in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        assert!(validate_observation(
            &m,
            &m.identity,
            &m.source_head,
            &m.tree_sha256,
            active,
            reserved,
            ambiguous
        )
        .is_err());
    }
}
#[test]
fn exact_inventory_detects_hidden_content_and_links_without_deletion() {
    let path = std::env::temp_dir().join(format!(
        "maintenance-fixture-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("cache.bin"), b"old").unwrap();
    let first = tree_digest(&path, false).unwrap();
    fs::write(path.join("cache.bin"), b"new").unwrap();
    assert_ne!(first, tree_digest(&path, false).unwrap());
    let second = tree_digest(&path, false).unwrap();
    fs::write(path.join(".ignored-model"), b"unique").unwrap();
    assert_ne!(second, tree_digest(&path, false).unwrap());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(path.join("cache.bin"), path.join("alias")).unwrap();
        assert!(tree_digest(&path, false).is_err());
    }
    assert!(path.join(".ignored-model").exists());
    fs::remove_dir_all(path).unwrap();
}
#[test]
fn malformed_request_returns_skip_and_never_enables_deletion() {
    let missing = std::env::temp_dir().join(format!(
        "missing-maintenance-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let result = check(&missing, &missing).unwrap();
    assert_eq!(result.status, "SKIP");
    assert!(!result.eligible);
    assert!(!result.deletion_supported);
}
