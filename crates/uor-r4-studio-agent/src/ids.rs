//! Stable identifiers and the deterministic event identity from spec §9.2.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

static COUNTER: AtomicU64 = AtomicU64::new(0);
static NONCE: OnceLock<[u8; 16]> = OnceLock::new();

fn nonce() -> &'static [u8; 16] {
    NONCE.get_or_init(|| {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let mut hasher = Sha256::new();
        hasher.update(nanos.to_le_bytes());
        hasher.update(std::process::id().to_le_bytes());
        let digest = hasher.finalize();
        let mut out = [0u8; 16];
        out.copy_from_slice(&digest[..16]);
        out
    })
}

pub(crate) fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn new_id(prefix: &str) -> String {
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut hasher = Sha256::new();
    hasher.update(nonce());
    hasher.update(counter.to_le_bytes());
    hasher.update(prefix.as_bytes());
    let digest = hasher.finalize();
    format!("{prefix}_{}", hex_lower(&digest[..8]))
}

pub fn new_project_id() -> String {
    new_id("prj")
}

pub fn new_task_id() -> String {
    new_id("tsk")
}

pub fn new_run_id() -> String {
    new_id("run")
}

pub fn new_message_id() -> String {
    new_id("msg")
}

pub fn new_tool_call_id() -> String {
    new_id("tc")
}

pub fn new_approval_id() -> String {
    new_id("apr")
}

pub fn new_artifact_id() -> String {
    new_id("art")
}

pub fn snapshot_id(run_id: &str) -> String {
    format!("snap_{run_id}")
}

pub fn turn_id(run_id: &str, index: u64) -> String {
    format!("turn_{run_id}_{index}")
}

pub fn event_id(run_id: &str, generation: u64, sequence: u64) -> String {
    format!("evt_{run_id}_{generation}_{sequence:012}")
}

pub fn is_valid_id(id: &str, prefix: &str) -> bool {
    let Some(rest) = id.strip_prefix(prefix) else {
        return false;
    };
    let Some(hex) = rest.strip_prefix('_') else {
        return false;
    };
    hex.len() == 16
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
