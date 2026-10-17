//! Peer replication for the exact object store: native manifest exchange plus missing-subtree fetch,
//! using the same wire messages as the proven P2P driver (kinds 20..=24). The receiver asks for a
//! manifest, then repeatedly asks for exactly the node ids its store reports missing. The sender
//! serves encoded native records (scale, id, payload); the receiver imports them through the store's
//! validated import path and finally rebuilds the object with digest verification. Record and
//! manifest encodings are the crate's native wire formats; no new serialization is added.

use std::io::{Read, Write};

use crate::{Error, Limits, Manifest, Result, Store};

const MANIFEST_REQUEST: u8 = 20;
const MANIFEST: u8 = 21;
const FETCH: u8 = 22;
const NODES: u8 = 23;
const DONE: u8 = 24;
const MAX_MESSAGE: usize = 64 * 1024 * 1024;
const MAX_ROUNDS: u64 = 128;
const BATCH: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PullStats {
    pub rounds: u64,
    pub nodes_requested: u64,
    pub record_bytes: u64,
    pub verified: bool,
}

fn write_msg<W: Write>(writer: &mut W, kind: u8, body: &[u8]) -> Result<()> {
    writer.write_all(&[kind])?;
    writer.write_all(&(body.len() as u32).to_be_bytes())?;
    writer.write_all(body)?;
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

/// Serve one peer session from a full store and the manifest of the object being replicated.
pub fn serve<R: Read, W: Write>(
    store: &Store,
    manifest: &Manifest,
    reader: &mut R,
    writer: &mut W,
) -> Result<()> {
    let limits = store.limits().clone();
    loop {
        let (kind, body) = read_msg(reader, &limits)?;
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
                let mut out = Vec::new();
                out.extend_from_slice(&(count as u16).to_be_bytes());
                for i in 0..count {
                    let mut id = [0u8; 12];
                    id.copy_from_slice(&body[2 + 12 * i..14 + 12 * i]);
                    let record = store.record_bytes(id).ok_or(Error::Missing(id))?;
                    out.extend_from_slice(&record);
                }
                write_msg(writer, NODES, &out)?;
            }
            DONE => return Ok(()),
            _ => return Err(Error::SchemaMismatch),
        }
    }
}

/// Pull the object described by the peer's manifest into the store, transferring only missing nodes.
pub fn pull<R: Read, W: Write>(
    store: &mut Store,
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
    let mut stats = PullStats::default();
    while stats.rounds < MAX_ROUNDS {
        let missing = store.missing(&manifest)?;
        if missing.is_empty() {
            break;
        }
        let batch = &missing[..missing.len().min(BATCH)];
        let mut request = Vec::with_capacity(2 + 12 * batch.len());
        request.extend_from_slice(&(batch.len() as u16).to_be_bytes());
        for id in batch {
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
        let records = &body[2..];
        let imported = store.import_records(records)?;
        if imported.new_records != count {
            return Err(Error::IntegrityFailure);
        }
        stats.rounds += 1;
        stats.nodes_requested += count as u64;
        stats.record_bytes += records.len() as u64;
    }
    if !store.missing(&manifest)?.is_empty() {
        return Err(Error::LimitExceeded("peer rounds"));
    }
    let verified = store.rebuild(&manifest)?;
    stats.verified = verified.whole_object_digest_verified();
    if !stats.verified {
        return Err(Error::IntegrityFailure);
    }
    Ok((manifest, stats))
}
