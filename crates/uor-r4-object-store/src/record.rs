use crate::{hash::node_id, span, Error, NodeId, Result};

/// Fixed-size validated Showcase record: scale || BLAKE2b-96 || payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    id: NodeId,
    scale: u8,
    payload: [u8; 36],
}
impl Record {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let scale = *bytes.first().ok_or(Error::SchemaMismatch)?;
        span(scale)?;
        let len = if scale == 0 { 32 } else { 36 };
        if bytes.len() != 13 + len {
            return Err(Error::SchemaMismatch);
        }
        let mut id = [0; 12];
        id.copy_from_slice(&bytes[1..13]);
        if node_id(scale, &bytes[13..]) != id {
            return Err(Error::IntegrityFailure);
        }
        let mut payload = [0; 36];
        payload[..len].copy_from_slice(&bytes[13..]);
        Ok(Self { id, scale, payload })
    }
    pub(crate) fn leaf(bytes: &[u8]) -> Self {
        // Caller supplies one bounded chunk. Last atom is canonical zero padding.
        let mut payload = [0; 36];
        payload[..bytes.len()].copy_from_slice(bytes);
        Self {
            id: node_id(0, &payload[..32]),
            scale: 0,
            payload,
        }
    }
    pub(crate) fn parent(scale: u8, children: [NodeId; 3]) -> Result<Self> {
        if scale == 0 {
            return Err(Error::SchemaMismatch);
        }
        span(scale)?;
        let mut payload = [0; 36];
        for (dst, id) in payload.chunks_exact_mut(12).zip(children) {
            dst.copy_from_slice(&id);
        }
        Ok(Self {
            id: node_id(scale, &payload),
            scale,
            payload,
        })
    }
    pub fn id(&self) -> NodeId {
        self.id
    }
    pub fn scale(&self) -> u8 {
        self.scale
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload[..if self.scale == 0 { 32 } else { 36 }]
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.wire_len());
        bytes.push(self.scale);
        bytes.extend_from_slice(&self.id);
        bytes.extend_from_slice(self.payload());
        bytes
    }
    pub(crate) fn wire_len(&self) -> usize {
        if self.scale == 0 {
            45
        } else {
            49
        }
    }
    pub(crate) fn children(&self) -> Result<[NodeId; 3]> {
        if self.scale == 0 {
            return Err(Error::SchemaMismatch);
        }
        let mut ids = [[0; 12]; 3];
        for (id, src) in ids.iter_mut().zip(self.payload.chunks_exact(12)) {
            id.copy_from_slice(src);
        }
        Ok(ids)
    }
}
