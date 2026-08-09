//! Canonical serializer (RFC §1 rule 7 / decision D2): deterministic
//! bytes for identical inputs.
//!
//! Canonical form rules, all enforced here:
//!
//! - sections sorted by `section_id` (no duplicates);
//! - the mandatory HEAD section must be present (its body feeds
//!   `head_cid`);
//! - unknown section IDs must carry [`SectionId::OPTIONAL_BIT`] so the
//!   emitted artifact passes this crate's own stage-1 validation;
//! - header + section table first, then bodies in ID order, each body
//!   starting at a multiple of `1 << alignment_log2`, zero padding in
//!   between, no trailing padding after the last body;
//! - `total_len` = one past the last body byte;
//! - CIDs per the crate-level convention: `head_cid` over the HEAD body,
//!   `artifact_cid` over `artifact_bytes[56..total_len]` with the
//!   `artifact_cid` field zeroed while hashing and patched in afterwards.

use alloc::vec::Vec;

use crate::error::FormatError;
use crate::header::{
    self, ARTIFACT_CID_OFFSET, ENDIANNESS_LITTLE, FORMAT_VERSION_MAJOR, FORMAT_VERSION_MINOR,
    HEADER_LEN, HEAD_CID_OFFSET, KNOWN_MANDATORY_FEATURES, MAGIC, MANDATORY_FEATURE_SPACE,
    MAX_ALIGNMENT_LOG2, MIN_ALIGNMENT_LOG2, SECTION_ENTRY_LEN,
};
use crate::types::SectionId;

/// Builds canonical R4G1 container bytes from `(SectionId, flags,
/// payload)` triples.
///
/// Sections may be added in any order; [`ArtifactBuilder::build`] sorts
/// by section ID to produce the canonical ordering.
#[derive(Debug, Clone, Default)]
pub struct ArtifactBuilder {
    alignment_log2: u8,
    flags: u32,
    sections: Vec<(SectionId, u32, Vec<u8>)>,
}

impl ArtifactBuilder {
    /// New builder emitting sections aligned to `1 << alignment_log2`.
    pub fn new(alignment_log2: u8) -> Self {
        Self {
            alignment_log2,
            flags: 0,
            sections: Vec::new(),
        }
    }

    /// Set the header feature-bit word. Only defined or optional-space
    /// bits are accepted at build time.
    pub fn with_flags(mut self, flags: u32) -> Self {
        self.flags = flags;
        self
    }

    /// Append a section payload. `flags` is the per-entry flag word (no
    /// bits defined yet; pass 0).
    pub fn add_section(&mut self, id: SectionId, flags: u32, payload: &[u8]) -> &mut Self {
        self.sections.push((id, flags, payload.to_vec()));
        self
    }

    /// Emit the canonical container bytes.
    pub fn build(&self) -> Result<Vec<u8>, crate::NotAProduct> {
        if !(MIN_ALIGNMENT_LOG2..=MAX_ALIGNMENT_LOG2).contains(&self.alignment_log2) {
            return Err((FormatError::UnsupportedAlignment(self.alignment_log2)).into());
        }
        let unknown_mandatory = self.flags & MANDATORY_FEATURE_SPACE & !KNOWN_MANDATORY_FEATURES;
        if unknown_mandatory != 0 {
            return Err((FormatError::UnknownMandatoryFeature(unknown_mandatory)).into());
        }

        let mut sections: Vec<&(SectionId, u32, Vec<u8>)> = self.sections.iter().collect();
        sections.sort_by_key(|(id, _, _)| id.0);
        for pair in sections.windows(2) {
            if pair[0].0 == pair[1].0 {
                return Err((FormatError::DuplicateSection(pair[0].0)).into());
            }
        }
        if !sections.iter().any(|(id, _, _)| *id == SectionId::HEAD) {
            return Err((FormatError::MissingHead).into());
        }
        for (id, _, _) in &sections {
            if !id.is_known() && id.mandatory() {
                return Err((FormatError::UnknownMandatorySection(id.0)).into());
            }
        }

        let count = u32::try_from(sections.len()).map_err(|_| FormatError::OffsetOverflow)?;
        let align: u64 = 1 << self.alignment_log2;
        let table_end = (HEADER_LEN as u64)
            .checked_add(u64::from(count) * SECTION_ENTRY_LEN as u64)
            .ok_or(FormatError::OffsetOverflow)?;

        // Layout pass: aligned offsets in canonical ID order, no padding
        // after the last body. Checked arithmetic throughout. HEAD is
        // required above, so `sections` is non-empty here.
        let mut cursor = align_up(table_end, align)?;
        let mut entries: Vec<(u32, u32)> = Vec::with_capacity(sections.len());
        for (_, _, payload) in &sections {
            let offset = u32::try_from(cursor).map_err(|_| FormatError::OffsetOverflow)?;
            let length = u32::try_from(payload.len())
                .map_err(|_| FormatError::SectionTooLarge(payload.len() as u64))?;
            entries.push((offset, length));
            cursor = cursor
                .checked_add(u64::from(length))
                .ok_or(FormatError::OffsetOverflow)?;
            cursor = align_up(cursor, align)?;
        }
        let total_len = match entries.last() {
            Some(&(offset, length)) => u64::from(offset) + u64::from(length),
            None => table_end,
        };

        // Emission pass.
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(MAGIC);
        out.push(FORMAT_VERSION_MAJOR);
        out.push(FORMAT_VERSION_MINOR);
        out.push(ENDIANNESS_LITTLE);
        out.push(self.alignment_log2);
        out.extend_from_slice(&total_len.to_le_bytes());
        out.extend_from_slice(&count.to_le_bytes());
        out.extend_from_slice(&self.flags.to_le_bytes());
        out.extend_from_slice(&[0u8; 32]); // artifact_cid: zeroed during hashing
        out.extend_from_slice(&[0u8; 32]); // head_cid: patched below
        for ((id, flags, _), (offset, length)) in sections.iter().zip(entries.iter()) {
            out.extend_from_slice(&id.0.to_le_bytes());
            out.extend_from_slice(&flags.to_le_bytes());
            out.extend_from_slice(&offset.to_le_bytes());
            out.extend_from_slice(&length.to_le_bytes());
        }
        for ((_, _, payload), (offset, _)) in sections.iter().zip(entries.iter()) {
            let offset = usize::try_from(*offset).map_err(|_| FormatError::OffsetOverflow)?;
            // Zero padding up to the aligned section start. (By
            // construction `out.len() <= offset`; a plain push loop can
            // never truncate the way `Vec::resize` could.)
            while out.len() < offset {
                out.push(0);
            }
            out.extend_from_slice(payload);
        }

        // head_cid over the HEAD body.
        let head_payload = sections
            .iter()
            .find(|(id, _, _)| *id == SectionId::HEAD)
            .map(|(_, _, payload)| payload.as_slice())
            .ok_or(FormatError::MissingHead)?;
        let head_cid = blake3::hash(head_payload);
        out[HEAD_CID_OFFSET..HEAD_CID_OFFSET + 32].copy_from_slice(head_cid.as_bytes());

        // artifact_cid over everything after its own field (field zeroed
        // above, so the convention is insensitive to its contents).
        let artifact_cid = blake3::hash(&out[header::ARTIFACT_HASH_START..]);
        out[ARTIFACT_CID_OFFSET..ARTIFACT_CID_OFFSET + 32].copy_from_slice(artifact_cid.as_bytes());

        Ok(out)
    }
}

/// Smallest multiple of `align` ≥ `x`, with checked arithmetic.
fn align_up(x: u64, align: u64) -> Result<u64, crate::NotAProduct> {
    x.checked_add(align - 1)
        .map(|v| v & !(align - 1))
        .ok_or_else(|| FormatError::OffsetOverflow.into())
}
