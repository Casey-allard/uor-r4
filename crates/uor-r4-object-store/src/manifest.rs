use crate::{reserve, Error, Limits, NodeId, ObjectVersion, Result};
use sha2::{Digest, Sha256};
const MAGIC: &[u8; 8] = b"UOROBJ01";
const HEADER: usize = 49;
const MAX_ROOTS: usize = 128;
/// Canonical UOR manifest framing, separate from the Showcase record wire format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    len: u64,
    roots: Vec<(NodeId, u8)>,
    sha256: [u8; 32],
}
impl Manifest {
    pub fn from_parts(
        len: u64,
        roots: Vec<(NodeId, u8)>,
        sha256: [u8; 32],
        limits: &Limits,
    ) -> Result<Self> {
        let m = Self { len, roots, sha256 };
        m.validate(limits)?;
        Ok(m)
    }
    pub(crate) fn validate(&self, limits: &Limits) -> Result<()> {
        if self.len > limits.max_object_bytes {
            return Err(Error::LimitExceeded("object bytes"));
        }
        if self.roots.len() > MAX_ROOTS {
            return Err(Error::LimitExceeded("manifest roots"));
        }
        // Base-three digits of ceil(len/32), in descending order, uniquely fix the frontier.
        let mut atoms = self.len / 32 + u64::from(!self.len.is_multiple_of(32));
        let mut digits = [0u8; 41];
        let mut levels = 0;
        while atoms > 0 {
            digits[levels] = (atoms % 3) as u8;
            atoms /= 3;
            levels += 1;
        }
        let mut index = 0;
        for scale in (0..levels).rev() {
            crate::span(scale as u8)?;
            for _ in 0..digits[scale] {
                if self.roots.get(index).map(|p| p.1) != Some(scale as u8) {
                    return Err(Error::SchemaMismatch);
                }
                index += 1;
            }
        }
        if index != self.roots.len() {
            return Err(Error::SchemaMismatch);
        }
        Ok(())
    }
    pub fn roots(&self) -> &[(NodeId, u8)] {
        &self.roots
    }
    pub fn byte_len(&self) -> u64 {
        self.len
    }
    pub fn whole_sha256(&self) -> [u8; 32] {
        self.sha256
    }
    pub fn version(&self) -> ObjectVersion {
        Sha256::digest(self.encode()).into()
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER + 13 * self.roots.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&self.len.to_le_bytes());
        out.extend_from_slice(&self.sha256);
        out.push(self.roots.len() as u8);
        for (id, scale) in &self.roots {
            out.extend_from_slice(id);
            out.push(*scale);
        }
        out
    }
    pub fn decode(bytes: &[u8], limits: &Limits) -> Result<Self> {
        if bytes.len() < HEADER || &bytes[..8] != MAGIC {
            return Err(Error::SchemaMismatch);
        }
        let count = bytes[48] as usize;
        if count > MAX_ROOTS {
            return Err(Error::LimitExceeded("manifest roots"));
        }
        if bytes.len() != HEADER + 13 * count {
            return Err(Error::SchemaMismatch);
        }
        let mut len = [0; 8];
        len.copy_from_slice(&bytes[8..16]);
        let len = u64::from_le_bytes(len);
        if len > limits.max_object_bytes {
            return Err(Error::LimitExceeded("object bytes"));
        }
        let mut digest = [0; 32];
        digest.copy_from_slice(&bytes[16..48]);
        let mut roots = Vec::new();
        reserve(&mut roots, count)?;
        for item in bytes[HEADER..].chunks_exact(13) {
            let mut id = [0; 12];
            id.copy_from_slice(&item[..12]);
            roots.push((id, item[12]));
        }
        Self::from_parts(len, roots, digest, limits)
    }
}
