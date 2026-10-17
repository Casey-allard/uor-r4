//! Peer replication for the exact object store: native manifest exchange plus missing-subtree fetch,
//! using the same wire message kinds as the P2P driver (20..=24).
//!
//! Hardening properties:
//! - Trusted anchor: pull_expected rejects a peer manifest that does not match the caller-supplied
//!   object identity (byte length + manifest version) before any record is fetched or imported.
//! - Dynamic batching: the request batch is derived from the store's limits, so a cold pull of a
//!   multi-megabyte object completes (the previous fixed cap could not finish a 1 MiB cold pull).
//! - Correlated responses: every returned record must have been requested, the declared count must
//!   equal the batch size, and duplicate ids are rejected.
//! - Atomic commit: records are staged in memory, the object is assembled and its native SHA-256 is
//!   verified against the manifest, and only then are the records imported into the store.
//! - Protocol hygiene: DONE is sent after commit, every message flushes, a clean EOF ends a serve
//!   session, and serve only answers ids in the requested manifest's closure with request/byte budgets.

use std::collections::{BTreeMap, HashSet};
use std::io::{Read, Write};

use sha2::{Digest, Sha256};

use crate::{Error, Limits, Manifest, NodeId, ObjectVersion, Record, Result, Store};

const MANIFEST_REQUEST: u8 = 20;
const MANIFEST: u8 = 21;
const FETCH: u8 = 22;
const NODES: u8 = 23;
const DONE: u8 = 24;
const MAX_MESSAGE: usize = 64 * 1024 * 1024;
const DESIRED_BATCH: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PullStats {
    pub rounds: u64,
    pub nodes_requested: u64,
    pub record_bytes: u64,
    pub verified: bool,
}

/// The object identity a caller already trusts (from a registry, manifest pin or local expectation).
/// Any combination of pins may be supplied; at least one must be set for an anchored pull. bytes is
/// the native object length, version is the manifest version (SHA-256 of the canonical UOROBJ01
/// manifest), and whole_sha256 is the SHA-256 of the object payload - the digest an OCI descriptor or
/// artifact store publishes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ExpectedObject {
    pub bytes: Option<u64>,
    pub version: Option<ObjectVersion>,
    pub whole_sha256: Option<[u8; 32]>,
}

impl ExpectedObject {
    pub fn from_manifest(manifest: &Manifest) -> ExpectedObject {
        ExpectedObject {
            bytes: Some(manifest.byte_len()),
            version: Some(manifest.version()),
            whole_sha256: None,
        }
    }

    pub fn from_sha256(sha: [u8; 32]) -> ExpectedObject {
        ExpectedObject {
            bytes: None,
            version: None,
            whole_sha256: Some(sha),
        }
    }

    pub fn with_bytes(mut self, bytes: u64) -> ExpectedObject {
        self.bytes = Some(bytes);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_none() && self.version.is_none() && self.whole_sha256.is_none()
    }

    fn matches(&self, manifest: &Manifest) -> bool {
        if self.is_empty() {
            return false;
        }
        if let Some(bytes) = self.bytes {
            if bytes != manifest.byte_len() {
                return false;
            }
        }
        if let Some(version) = self.version {
            if version != manifest.version() {
                return false;
            }
        }
        if let Some(sha) = self.whole_sha256 {
            if sha != manifest.whole_sha256() {
                return false;
            }
        }
        true
    }
}

fn write_msg<W: Write>(writer: &mut W, kind: u8, body: &[u8]) -> Result<()> {
    writer.write_all(&[kind])?;
    writer.write_all(&(body.len() as u32).to_be_bytes())?;
    writer.write_all(body)?;
    writer.flush()?;
    Ok(())
}

fn read_msg<R: Read>(reader: &mut R, limits: &Limits) -> Result<(u8, Vec<u8>)> {
    let mut head = [0u8; 5];
    reader.read_exact(&mut head)?;
    let kind = head[0];
    let len = u32::from_be_bytes([head[1], head[2], head[3], head[4]]) as usize;
    if len > MAX_MESSAGE || len > limits.max_record_bytes {
        return Err(Error::LimitExceeded("peer message"));
    }
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body)?;
    Ok((kind, body))
}

fn record_size(first_byte: u8) -> usize {
    if first_byte == 0 {
        45
    } else {
        49
    }
}

fn record_from_store(store: &Store, id: NodeId) -> Result<Record> {
    let bytes = store.record_bytes(id).ok_or(Error::Missing(id))?;
    Record::decode(&bytes)
}

fn batch_size(limits: &Limits) -> usize {
    let by_response = limits.max_record_bytes.saturating_sub(2) / 49;
    let by_request = limits.max_record_bytes.saturating_sub(2) / 12;
    DESIRED_BATCH.min(by_response).min(by_request).max(1)
}

fn allowed_ids(store: &Store, manifest: &Manifest) -> Result<HashSet<NodeId>> {
    let limits = store.limits();
    let mut allowed = HashSet::new();
    let mut visits = 0usize;
    for (root, _) in manifest.roots() {
        collect_closure(store, *root, &mut allowed, &mut visits, limits)?;
    }
    Ok(allowed)
}

fn collect_closure(
    store: &Store,
    id: NodeId,
    allowed: &mut HashSet<NodeId>,
    visits: &mut usize,
    limits: &Limits,
) -> Result<()> {
    if allowed.contains(&id) {
        return Ok(());
    }
    *visits += 1;
    if *visits > limits.max_visits {
        return Err(Error::LimitExceeded("closure visits"));
    }
    let record = record_from_store(store, id)?;
    allowed.insert(id);
    if record.scale() > 0 {
        for child in record.payload().chunks_exact(12) {
            let mut child_id = [0u8; 12];
            child_id.copy_from_slice(child);
            collect_closure(store, child_id, allowed, visits, limits)?;
        }
    }
    Ok(())
}

/// Serve one peer session from a full store and the manifest of the object being replicated.
pub fn serve<R: Read, W: Write>(
    store: &Store,
    manifest: &Manifest,
    reader: &mut R,
    writer: &mut W,
) -> Result<()> {
    let limits = store.limits().clone();
    let allowed = allowed_ids(store, manifest)?;
    let mut requests = 0u64;
    let mut served_bytes = 0usize;
    loop {
        let (kind, body) = match read_msg(reader, &limits) {
            Ok(value) => value,
            Err(Error::Io {
                kind: std::io::ErrorKind::UnexpectedEof,
                ..
            }) => return Ok(()),
            Err(error) => return Err(error),
        };
        requests += 1;
        if requests > limits.max_visits as u64 {
            return Err(Error::LimitExceeded("serve requests"));
        }
        match kind {
            MANIFEST_REQUEST => write_msg(writer, MANIFEST, &manifest.encode())?,
            FETCH => {
                if body.len() < 2 {
                    return Err(Error::SchemaMismatch);
                }
                let count = u16::from_be_bytes([body[0], body[1]]) as usize;
                if body.len() != 2 + 12 * count {
                    return Err(Error::SchemaMismatch);
                }
                let mut out = Vec::with_capacity(2 + count * 49);
                out.extend_from_slice(&(count as u16).to_be_bytes());
                for index in 0..count {
                    let mut id = [0u8; 12];
                    id.copy_from_slice(&body[2 + 12 * index..14 + 12 * index]);
                    if !allowed.contains(&id) {
                        return Err(Error::SchemaMismatch);
                    }
                    let record = store.record_bytes(id).ok_or(Error::Missing(id))?;
                    served_bytes += record.len();
                    if served_bytes > limits.max_record_bytes {
                        return Err(Error::LimitExceeded("serve bytes"));
                    }
                    out.extend_from_slice(&record);
                }
                write_msg(writer, NODES, &out)?;
            }
            DONE => return Ok(()),
            _ => return Err(Error::SchemaMismatch),
        }
    }
}

/// Pull the object described by the peer's manifest without a caller-supplied identity anchor.
pub fn pull<R: Read, W: Write>(
    store: &mut Store,
    reader: &mut R,
    writer: &mut W,
) -> Result<(Manifest, PullStats)> {
    pull_expected(store, None, reader, writer)
}

/// Pull the object, requiring the peer manifest to match a caller-supplied identity first.
pub fn pull_expected<R: Read, W: Write>(
    store: &mut Store,
    expected: Option<ExpectedObject>,
    reader: &mut R,
    writer: &mut W,
) -> Result<(Manifest, PullStats)> {
    let limits = store.limits().clone();
    write_msg(writer, MANIFEST_REQUEST, &[])?;
    let (kind, body) = read_msg(reader, &limits)?;
    if kind != MANIFEST {
        return Err(Error::SchemaMismatch);
    }
    let manifest = Manifest::decode(&body, &limits)?;
    if let Some(expected) = expected {
        if !expected.matches(&manifest) {
            return Err(Error::BindingMismatch);
        }
    }

    let batch = batch_size(&limits);
    let mut fetched: BTreeMap<NodeId, Record> = BTreeMap::new();
    let mut stats = PullStats::default();
    loop {
        let missing = missing_nodes(store, &manifest, &fetched)?;
        if missing.is_empty() {
            break;
        }
        for chunk in missing.chunks(batch) {
            let mut request = Vec::with_capacity(2 + 12 * chunk.len());
            request.extend_from_slice(&(chunk.len() as u16).to_be_bytes());
            for id in chunk {
                request.extend_from_slice(id);
            }
            write_msg(writer, FETCH, &request)?;
            let (kind, body) = read_msg(reader, &limits)?;
            if kind != NODES {
                return Err(Error::SchemaMismatch);
            }
            if body.len() < 2 {
                return Err(Error::SchemaMismatch);
            }
            let count = u16::from_be_bytes([body[0], body[1]]) as usize;
            if count != chunk.len() {
                return Err(Error::IntegrityFailure);
            }
            let mut offset = 2usize;
            for _ in 0..count {
                if offset >= body.len() {
                    return Err(Error::SchemaMismatch);
                }
                let size = record_size(body[offset]);
                let end = offset.checked_add(size).ok_or(Error::SchemaMismatch)?;
                let record = Record::decode(body.get(offset..end).ok_or(Error::SchemaMismatch)?)?;
                offset = end;
                if !chunk.contains(&record.id()) {
                    return Err(Error::IntegrityFailure);
                }
                if fetched.insert(record.id(), record).is_some() {
                    return Err(Error::IntegrityFailure);
                }
                stats.nodes_requested += 1;
                stats.record_bytes += size as u64;
            }
            if offset != body.len() {
                return Err(Error::SchemaMismatch);
            }
            stats.rounds += 1;
            if stats.nodes_requested > limits.max_records as u64
                || stats.record_bytes > limits.max_record_bytes as u64
            {
                return Err(Error::LimitExceeded("pull budget"));
            }
        }
    }

    let mut bytes = Vec::new();
    let mut visits = 0usize;
    for (root, _) in manifest.roots() {
        assemble_object(store, &fetched, *root, &mut bytes, &mut visits, &limits)?;
    }
    if bytes.len() < manifest.byte_len() as usize {
        return Err(Error::IntegrityFailure);
    }
    bytes.truncate(manifest.byte_len() as usize);
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    if digest != manifest.whole_sha256() {
        return Err(Error::IntegrityFailure);
    }

    let mut encoded = Vec::with_capacity(stats.record_bytes as usize);
    for record in fetched.values() {
        encoded.extend_from_slice(&record.encode());
    }
    store.import_records(&encoded)?;
    write_msg(writer, DONE, &[])?;

    let verified = store.rebuild(&manifest)?;
    stats.verified = verified.whole_object_digest_verified();
    if !stats.verified {
        return Err(Error::IntegrityFailure);
    }
    Ok((manifest, stats))
}

fn missing_nodes(
    store: &Store,
    manifest: &Manifest,
    fetched: &BTreeMap<NodeId, Record>,
) -> Result<Vec<NodeId>> {
    let limits = store.limits();
    let mut missing = Vec::new();
    let mut visits = 0usize;
    for (root, _) in manifest.roots() {
        walk_missing(store, *root, fetched, &mut missing, &mut visits, limits)?;
    }
    // Deduplicated repeated content makes the tree a DAG: the same missing node can be reached by
    // several paths, so request each id exactly once.
    let mut seen = HashSet::new();
    missing.retain(|id| seen.insert(*id));
    Ok(missing)
}

fn walk_missing(
    store: &Store,
    id: NodeId,
    fetched: &BTreeMap<NodeId, Record>,
    missing: &mut Vec<NodeId>,
    visits: &mut usize,
    limits: &Limits,
) -> Result<()> {
    *visits += 1;
    if *visits > limits.max_visits || missing.len() > limits.max_records {
        return Err(Error::LimitExceeded("missing visits"));
    }
    if store.has_record(id) {
        return Ok(());
    }
    if let Some(record) = fetched.get(&id) {
        if record.scale() > 0 {
            for child in record.payload().chunks_exact(12) {
                let mut child_id = [0u8; 12];
                child_id.copy_from_slice(child);
                walk_missing(store, child_id, fetched, missing, visits, limits)?;
            }
        }
        return Ok(());
    }
    missing.push(id);
    Ok(())
}

fn assemble_object(
    store: &Store,
    fetched: &BTreeMap<NodeId, Record>,
    id: NodeId,
    out: &mut Vec<u8>,
    visits: &mut usize,
    limits: &Limits,
) -> Result<()> {
    *visits += 1;
    if *visits > limits.max_visits || out.len() > limits.max_object_bytes as usize {
        return Err(Error::LimitExceeded("assemble visits"));
    }
    let record = match fetched.get(&id) {
        Some(record) => record.clone(),
        None => record_from_store(store, id)?,
    };
    if record.scale() == 0 {
        out.extend_from_slice(record.payload());
    } else {
        for child in record.payload().chunks_exact(12) {
            let mut child_id = [0u8; 12];
            child_id.copy_from_slice(child);
            assemble_object(store, fetched, child_id, out, visits, limits)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Limits {
        Limits {
            max_object_bytes: 1 << 30,
            max_records: 10_000_000,
            max_record_bytes: 1 << 30,
            max_returned_bytes: 1 << 30,
            max_visits: 100_000_000,
            max_occurrences: 10_000_000,
        }
    }

    fn atoms(n: usize, salt: u8) -> Vec<u8> {
        let mut out = Vec::with_capacity(n * 32);
        for i in 0..n {
            for j in 0..32 {
                let v = if j < 8 {
                    i.to_le_bytes()[j]
                } else {
                    ((i * 37 + j * 11 + salt as usize) & 0xFF) as u8
                };
                out.push(v);
            }
        }
        out
    }

    #[test]
    fn closure_excludes_records_from_other_objects() {
        let mut store = Store::new(limits());
        let first = store.fold(&atoms(1000, 1)).expect("fold first").manifest;
        let second = store.fold(&atoms(1000, 2)).expect("fold second").manifest;
        let closure = allowed_ids(&store, &first).expect("closure");
        let foreign = second.roots()[0].0;
        assert!(
            !closure.contains(&foreign),
            "a record from another object must not be in the served closure"
        );
        assert!(!closure.is_empty());
    }
}
