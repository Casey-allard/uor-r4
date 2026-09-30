//! Durable internal delivery intent. Network ambiguity leaves the original
//! immutable event pending; coordination event identity makes replay harmless.
use crate::{coord, invalid, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Deserialize)]
struct Pending {
    event: coord::Event,
    queued_utc: String,
    #[serde(default)]
    queued_order: u64,
}
fn layout(store: &Path) -> Result<PathBuf> {
    if !store.is_absolute() || fs::canonicalize(store)?.starts_with("/Volumes") {
        return Err(invalid(
            "outbox requires an existing internal coordination store",
        ));
    }
    let root = store.join("outbox");
    fs::create_dir_all(root.join("pending"))?;
    fs::create_dir_all(root.join("sent"))?;
    fs::create_dir_all(root.join("retired"))?;
    Ok(root)
}
fn id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
    {
        return Err(invalid("invalid outbox event identity"));
    }
    Ok(())
}
pub fn stage(store: &Path, event: &coord::Event) -> Result<PathBuf> {
    id(&event.event_id)?;
    let root = layout(store)?;
    let staging_lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(root.join("stage.lock"))?;
    staging_lock.lock()?;
    if root
        .join("retired")
        .join(format!("{}.json", event.event_id))
        .exists()
    {
        return Err(invalid(
            "event is durably retired; preserve it and use a new reconciled event identity",
        ));
    }
    let path = root
        .join("pending")
        .join(format!("{}.json", event.event_id));
    if let Ok(bytes) = fs::read(root.join("sent").join(format!("{}.json", event.event_id))) {
        let previous: serde_json::Value = serde_json::from_slice(&bytes)?;
        if previous.get("event") != Some(&serde_json::to_value(event)?) {
            return Err(invalid(
                "sent event identity reused with different contents",
            ));
        }
    }
    if path.exists() {
        let previous: Pending = serde_json::from_slice(&fs::read(&path)?)?;
        if serde_json::to_value(&previous.event)? != serde_json::to_value(event)? {
            return Err(invalid("outbox identity reused with different contents"));
        }
        return Ok(path);
    }
    let sequence_path = root.join("sequence.json");
    let sequence: u64 = if sequence_path.exists() {
        serde_json::from_slice(&fs::read(&sequence_path)?)?
    } else {
        0
    };
    let queued_order = sequence
        .checked_add(1)
        .ok_or_else(|| invalid("outbox sequence overflow"))?;
    crate::atomic_write(&sequence_path, &serde_json::to_vec(&queued_order)?)?;
    let bytes = serde_json::to_vec_pretty(&Pending {
        event: event.clone(),
        queued_utc: crate::utc_now_iso(),
        queued_order,
    })?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| invalid("clock before epoch"))?
        .as_nanos();
    let temp = root.join(format!(
        "staging-{}-{}-{nonce}",
        std::process::id(),
        event.event_id
    ));
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)?;
    let written = f.write_all(&bytes).and_then(|()| f.sync_all());
    drop(f);
    if let Err(error) = written {
        let _ = fs::remove_file(&temp);
        return Err(error.into());
    }
    let result = fs::hard_link(&temp, &path);
    let _ = fs::remove_file(temp);
    match result {
        Ok(()) => {}
        Err(e) => return Err(e.into()),
    }
    fs::File::open(root.join("pending"))?.sync_all()?;
    Ok(path)
}
pub fn send(store: &Path, event: &coord::Event) -> Result<String> {
    stage(store, event)?;
    let outcome = replay(store)?;
    let root = layout(store)?;
    let sent = root.join("sent").join(format!("{}.json", event.event_id));
    if sent.exists() {
        let value: serde_json::Value = serde_json::from_slice(&fs::read(sent)?)?;
        return value
            .get("remote_head")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| invalid("invalid sent receipt"));
    }
    Err(invalid(format!(
        "outbox retained pending intent: {outcome}"
    )))
}
fn deliver(store: &Path, event: &coord::Event) -> Result<String> {
    let pending = layout(store)?
        .join("pending")
        .join(format!("{}.json", event.event_id));
    let head = coord::apply(store, event)?;
    let root = layout(store)?;
    crate::atomic_write(
        &root.join("sent").join(format!("{}.json", event.event_id)),
        &serde_json::to_vec_pretty(
            &serde_json::json!({"event":event,"remote_head":head,"reconciled_utc":crate::utc_now_iso()}),
        )?,
    )?;
    if let Err(e) = fs::remove_file(pending) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(e.into());
        }
    }
    Ok(head)
}
pub fn replay(store: &Path) -> Result<serde_json::Value> {
    replay_with(store, |event| deliver(store, event))
}
fn replay_with(
    store: &Path,
    mut deliver_event: impl FnMut(&coord::Event) -> Result<String>,
) -> Result<serde_json::Value> {
    let root = layout(store)?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(root.join("replay.lock"))?;
    lock.lock()?;
    let mut events = Vec::new();
    for entry in fs::read_dir(root.join("pending"))? {
        let entry = entry?;
        if entry.path().extension().is_some_and(|x| x == "json") {
            let p: Pending = serde_json::from_slice(&fs::read(entry.path())?)?;
            events.push(p);
        }
    }
    events.sort_by(|a, b| {
        a.queued_order
            .cmp(&b.queued_order)
            .then(a.event.event_id.cmp(&b.event.event_id))
    });
    let mut sent = Vec::new();
    let mut retired = Vec::new();
    for pending in events {
        if valid_retirement(&root, &pending)? {
            retired.push(pending.event.event_id.clone());
            continue;
        }
        match deliver_event(&pending.event) {
            Ok(head) => {
                sent.push(serde_json::json!({"event_id":pending.event.event_id,"head":head}))
            }
            Err(error) => {
                return Ok(
                    serde_json::json!({"sent":sent,"retired":retired,"blocked_event":pending.event.event_id,"error":error.to_string(),"complete":false}),
                )
            }
        }
    }
    Ok(serde_json::json!({"sent":sent,"retired":retired,"complete":true}))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Retirement {
    schema: String,
    event: coord::Event,
    pending_sha256: String,
    reason: String,
    semantic_rejection: String,
    observed_remote_head: String,
    observed_server_time: u64,
    observed_state: coord::State,
    retired_utc: String,
}
fn semantic_rejection(state: &coord::State, event: &coord::Event, now: u64) -> Result<String> {
    if state.schema != "uor-r4.lab-state/1" || now < state.observed_server_time {
        return Err(invalid(
            "fresh remote state/time is unavailable; do not retire",
        ));
    }
    use coord::Action;
    let bound = match &event.action {
        Action::Checkpoint {
            issue,
            session,
            epoch,
            ..
        }
        | Action::Release {
            issue,
            session,
            epoch,
        }
        | Action::Reserve {
            issue,
            session,
            epoch,
            ..
        }
        | Action::FinishAttempt {
            issue,
            session,
            epoch,
            ..
        }
        | Action::Complete {
            issue,
            session,
            epoch,
            ..
        } => Some((*issue, session, *epoch)),
        Action::Register { session, .. } if state.labs.contains_key(session) => {
            return Ok("session identity has already been registered".into())
        }
        Action::Claim { issue, .. }
            if state
                .tasks
                .get(issue)
                .is_some_and(|claim| claim.phase == "complete") =>
        {
            return Ok("task is already complete; a successor issue is required".into())
        }
        _ => None,
    };
    if let Some((issue, session, epoch)) = bound {
        if let Some(claim) = state.tasks.get(&issue) {
            if claim.epoch > epoch {
                return Ok(format!(
                    "task {issue} generation {epoch} is fenced by {}",
                    claim.epoch
                ));
            }
            if claim.epoch == epoch {
                if claim.session != *session {
                    return Ok(format!(
                        "task {issue} generation belongs to another session"
                    ));
                }
                if ["released", "complete"].contains(&claim.phase.as_str()) {
                    return Ok(format!(
                        "task {issue} generation is terminal/released ({})",
                        claim.phase
                    ));
                }
                if claim.phase == "claimed" && claim.expires <= now {
                    return Ok(format!(
                        "task {issue} generation expired at {} and cannot be revived",
                        claim.expires
                    ));
                }
            }
        }
    }
    Err(invalid("no permanent semantic rejection proved; network/transient/unknown failures must remain pending"))
}
fn valid_retirement(root: &Path, pending: &Pending) -> Result<bool> {
    let path = root
        .join("retired")
        .join(format!("{}.json", pending.event.event_id));
    if !path.exists() {
        return Ok(false);
    }
    let record: Retirement = serde_json::from_slice(&fs::read(path)?)?;
    let raw = fs::read(
        root.join("pending")
            .join(format!("{}.json", pending.event.event_id)),
    )?;
    if record.schema != "uor-r4.outbox-retirement/1"
        || record.reason.trim().is_empty()
        || record.observed_remote_head.len() != 40
        || !record
            .observed_remote_head
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
        || record.pending_sha256 != crate::ledger::sha256(&raw)
        || serde_json::to_value(&record.event)? != serde_json::to_value(&pending.event)?
        || record.semantic_rejection
            != semantic_rejection(
                &record.observed_state,
                &record.event,
                record.observed_server_time,
            )?
    {
        return Err(invalid(
            "retirement evidence no longer matches retained pending intent",
        ));
    }
    Ok(true)
}
fn publish_retirement(
    root: &Path,
    pending: &Pending,
    head: &str,
    state: coord::State,
    now: u64,
    reason: &str,
) -> Result<serde_json::Value> {
    if reason.trim().is_empty() {
        return Err(invalid(
            "retirement needs an explicit reconciliation reason",
        ));
    }
    let semantic = semantic_rejection(&state, &pending.event, now)?;
    let path = root
        .join("retired")
        .join(format!("{}.json", pending.event.event_id));
    if valid_retirement(root, pending)? {
        return Ok(serde_json::from_slice(&fs::read(path)?)?);
    }
    let raw = fs::read(
        root.join("pending")
            .join(format!("{}.json", pending.event.event_id)),
    )?;
    let record = Retirement {
        schema: "uor-r4.outbox-retirement/1".into(),
        event: pending.event.clone(),
        pending_sha256: crate::ledger::sha256(&raw),
        reason: reason.into(),
        semantic_rejection: semantic,
        observed_remote_head: head.into(),
        observed_server_time: now,
        observed_state: state,
        retired_utc: crate::utc_now_iso(),
    };
    let bytes = serde_json::to_vec_pretty(&record)?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| invalid("clock unavailable for local filename"))?
        .as_nanos();
    let temp = root.join(format!(
        "retiring-{}-{}-{nonce}.json",
        std::process::id(),
        pending.event.event_id
    ));
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    let result = fs::hard_link(&temp, &path);
    let _ = fs::remove_file(&temp);
    result?;
    fs::File::open(root.join("retired"))?.sync_all()?;
    // Original pending bytes intentionally remain. Replays verify the durable
    // retirement before skipping them; no evidence is silently discarded.
    Ok(serde_json::from_slice(&bytes)?)
}
fn git_snapshot(store: &Path, args: &[&str]) -> Result<String> {
    let result = Command::new("git")
        .arg("-C")
        .arg(store)
        .args(args)
        .output()?;
    if !result.status.success() {
        return Err(invalid("fresh outbox reconciliation snapshot unavailable"));
    }
    String::from_utf8(result.stdout)
        .map(|s| s.trim().into())
        .map_err(|_| invalid("invalid snapshot text"))
}
/// Explicit, read-only-on-GitHub reconciliation. A positively stale operation
/// receives a durable local retirement receipt; network errors never qualify.
/// This does not release remote job reservations or fabricate completion.
pub fn retire(store: &Path, event_id: &str, reason: &str) -> Result<serde_json::Value> {
    id(event_id)?;
    let root = layout(store)?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(root.join("replay.lock"))?;
    lock.lock()?;
    let pending: Pending = serde_json::from_slice(&fs::read(
        root.join("pending").join(format!("{event_id}.json")),
    )?)?;
    if pending.event.event_id != event_id {
        return Err(invalid("pending filename and event differ"));
    }
    if valid_retirement(&root, &pending)? {
        return Ok(serde_json::from_slice(&fs::read(
            root.join("retired").join(format!("{event_id}.json")),
        )?)?);
    }
    // Fetch first; then pin the immutable object, so another local fetch cannot
    // mix a state tree and event-presence observation from different commits.
    let _ = coord::status(store)?;
    let head = git_snapshot(store, &["rev-parse", "refs/remotes/origin/codex/lab-state"])?;
    let state: coord::State = serde_json::from_str(&git_snapshot(
        store,
        &["show", &format!("{head}:state.json")],
    )?)?;
    let event_path = format!("events/{event_id}.json");
    if !git_snapshot(store, &["ls-tree", "-z", &head, "--", &event_path])?.is_empty() {
        return Err(invalid("event identity exists remotely; replay/reconcile its committed evidence instead of retiring"));
    }
    let now = coord::github_time(&state.repository)?;
    publish_retirement(&root, &pending, &head, state, now, reason)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn stale_fixture() -> (coord::State, coord::Event) {
        let mut state = coord::State::new("o/r".into(), "a".repeat(40)).unwrap();
        state.observed_server_time = 100;
        state.tasks.insert(
            1,
            coord::Claim {
                session: "successor".into(),
                epoch: 2,
                work_card: "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
                policy_sha: "a".repeat(40),
                base_sha: "a".repeat(40),
                paths: vec![],
                expires: 2000,
                phase: "claimed".into(),
                checkpoint: None,
            },
        );
        let event = coord::Event {
            event_id: "stale".into(),
            action: coord::Action::Checkpoint {
                issue: 1,
                session: "previous".into(),
                epoch: 1,
                packet: serde_json::json!({"next_action":"preserve result"}),
            },
        };
        (state, event)
    }
    #[test]
    fn retirement_requires_positive_staleness_not_transient_failure() {
        let (mut state, event) = stale_fixture();
        assert!(semantic_rejection(&state, &event, 101)
            .unwrap()
            .contains("fenced"));
        assert!(semantic_rejection(&state, &event, 99).is_err());
        state.tasks.clear();
        assert!(semantic_rejection(&state, &event, 101).is_err());
        let heartbeat = coord::Event {
            event_id: "heartbeat".into(),
            action: coord::Action::Heartbeat {
                session: "offline".into(),
                available: true,
            },
        };
        assert!(semantic_rejection(&state, &heartbeat, 101).is_err());
    }
    #[test]
    fn durable_retirement_preserves_original_and_unblocks_later_intent() {
        let dir = std::env::temp_dir().join(format!(
            "uor-outbox-retirement-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        let (state, stale) = stale_fixture();
        let original_path = stage(&dir, &stale).unwrap();
        let original = fs::read(&original_path).unwrap();
        let later = coord::Event {
            event_id: "later".into(),
            action: coord::Action::Heartbeat {
                session: "successor".into(),
                available: true,
            },
        };
        stage(&dir, &later).unwrap();
        let root = layout(&dir).unwrap();
        let pending: Pending = serde_json::from_slice(&original).unwrap();
        publish_retirement(
            &root,
            &pending,
            &"b".repeat(40),
            state,
            101,
            "Successor owns epoch 2; recover the retained checkpoint separately",
        )
        .unwrap();
        assert_eq!(fs::read(&original_path).unwrap(), original);
        assert!(stage(&dir, &stale).is_err());
        let mut delivered = Vec::new();
        let result = replay_with(&dir, |event| {
            delivered.push(event.event_id.clone());
            Ok("c".repeat(40))
        })
        .unwrap();
        assert_eq!(delivered, vec!["later"]);
        assert_eq!(result["complete"], true);
        assert_eq!(result["retired"], serde_json::json!(["stale"]));
        fs::write(&original_path, b"tampered preserved event").unwrap();
        assert!(replay_with(&dir, |_| Ok("c".repeat(40))).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn pending_event_survives_failed_network_and_conflicting_retry_is_rejected() {
        let dir = std::env::temp_dir().join(format!(
            "uor-outbox-{}-{}",
            std::process::id(),
            crate::utc_now_compact()
        ));
        fs::create_dir(&dir).unwrap();
        let event = coord::Event {
            event_id: "one".into(),
            action: coord::Action::Heartbeat {
                session: "lab".into(),
                available: true,
            },
        };
        assert!(send(&dir, &event).is_err());
        assert!(dir.join("outbox/pending/one.json").exists());
        stage(&dir, &event).unwrap();
        let mut other = event.clone();
        other.action = coord::Action::Heartbeat {
            session: "lab".into(),
            available: false,
        };
        assert!(stage(&dir, &other).is_err());
        let mut second = event.clone();
        second.event_id = "second".into();
        assert!(send(&dir, &second).is_err());
        let result = replay(&dir).unwrap();
        assert_eq!(result["blocked_event"], "one");
        assert!(dir.join("outbox/pending/second.json").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
