//! Fixed header layout (RFC §2) and the header-level stage-1 checks.
//!
//! ```text
//! offset  size  field
//! 0       4     magic "R4G1"
//! 4       u8    format_version.major
//! 5       u8    format_version.minor
//! 6       u8    endianness marker (0x01 = little)
//! 7       u8    alignment_log2 (section alignment, >= 3)
//! 8       u64   total_len (bytes, includes header)
//! 16      u32   section_count
//! 20      u32   flags (feature bits)
//! 24      32B   artifact_cid
//! 56      32B   head_cid
//! 88      ..    section table, section_count entries x 16 bytes
//! ```

use crate::error::FormatError;
use crate::types::ArtifactCid;

/// Container magic, bytes 0..4.
pub const MAGIC: &[u8; 4] = b"R4G1";
/// Fixed header size in bytes.
pub const HEADER_LEN: usize = 88;
/// Section-table entry size in bytes: `{u32 id, u32 flags, u32 offset,
/// u32 length}` (resolved, RFC §9.1).
pub const SECTION_ENTRY_LEN: usize = 16;
/// Only accepted endianness marker: little-endian.
pub const ENDIANNESS_LITTLE: u8 = 0x01;
/// The single active format major version: the draft line (RFC §8 version
/// gate, 2026-07-21): pre-freeze artifacts MUST use `major = 0`; stable R4G1
/// starts at `major >= 1` only after packed widths are frozen in Phase 1.
/// Until then, readers reject any artifact claiming `major >= 1`.
pub const FORMAT_VERSION_MAJOR: u8 = 0;
/// Minor version emitted by the serializer. Readers accept any minor
/// under a supported major (minor bumps add only optional content,
/// RFC §8).
pub const FORMAT_VERSION_MINOR: u8 = 0;
/// Byte offset of the `artifact_cid` field.
pub const ARTIFACT_CID_OFFSET: usize = 24;
/// Byte offset of the `head_cid` field.
pub const HEAD_CID_OFFSET: usize = 56;
/// Start of the `artifact_cid` hash input: everything after the
/// `artifact_cid` field (see crate-level docs for the exact convention).
pub const ARTIFACT_HASH_START: usize = HEAD_CID_OFFSET;

/// Minimum `alignment_log2` (RFC §2: section alignment ≥ 3).
pub(crate) const MIN_ALIGNMENT_LOG2: u8 = 3;
/// Maximum `alignment_log2`: section offsets are u32 (RFC §9.1), so
/// larger alignments cannot be satisfied by any non-zero offset.
pub(crate) const MAX_ALIGNMENT_LOG2: u8 = 31;

/// Mandatory half of the header feature-flag space.
///
/// The RFC declares "unknown mandatory bits ⇒ reject" (§2, §6) without
/// partitioning the 32 flag bits into mandatory vs optional spaces. This
/// crate resolves the ambiguity: bits 0..=15 are the mandatory space
/// (an unknown set bit rejects the artifact), bits 16..=31 are the
/// optional space (unknown set bits are ignored, allowing minor-version
/// feature extensions per RFC §8).
pub(crate) const MANDATORY_FEATURE_SPACE: u32 = 0x0000_FFFF;

/// Mandatory feature bits defined by this format version: none yet.
pub(crate) const KNOWN_MANDATORY_FEATURES: u32 = 0x0000_0000;

/// Decoded fixed header. Plain `Copy` data — borrowed section bytes are
/// exposed separately through `GraphView`, nothing here is heap-resident.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// Format major version (validated against [`FORMAT_VERSION_MAJOR`]).
    pub major: u8,
    /// Format minor version (any value accepted under a supported major).
    pub minor: u8,
    /// Section alignment as log2; every section offset is a multiple of
    /// `1 << alignment_log2`.
    pub alignment_log2: u8,
    /// Total artifact length in bytes, including the header; validated
    /// to equal the actual buffer length.
    pub total_len: u64,
    /// Number of 16-byte section-table entries.
    pub section_count: u32,
    /// Header feature bits (see the mandatory/optional split documented
    /// on this module).
    pub flags: u32,
    /// blake3 over `artifact_bytes[ARTIFACT_HASH_START..total_len]`.
    pub artifact_cid: ArtifactCid,
    /// blake3 over the HEAD section body.
    pub head_cid: ArtifactCid,
}

/// Read a little-endian u16 without unsafe/transmute.
pub(crate) fn read_u16_le(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

/// Read a little-endian i16 without unsafe/transmute.
pub(crate) fn read_i16_le(bytes: &[u8], at: usize) -> i16 {
    i16::from_le_bytes([bytes[at], bytes[at + 1]])
}

/// Read a little-endian i32 without unsafe/transmute.
pub(crate) fn read_i32_le(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

/// Read a little-endian u32 without unsafe/transmute.
pub(crate) fn read_u32_le(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

/// Read a little-endian u64 without unsafe/transmute.
pub(crate) fn read_u64_le(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes([
        bytes[at],
        bytes[at + 1],
        bytes[at + 2],
        bytes[at + 3],
        bytes[at + 4],
        bytes[at + 5],
        bytes[at + 6],
        bytes[at + 7],
    ])
}

pub(crate) fn read_cid(bytes: &[u8], at: usize) -> ArtifactCid {
    let mut cid = [0u8; 32];
    cid.copy_from_slice(&bytes[at..at + 32]);
    ArtifactCid(cid)
}

/// Decode the fixed header and enforce the header-level stage-1
/// invariants: length, magic, major version, endianness marker,
/// alignment range, `total_len == actual`, and unknown mandatory
/// feature bits (RFC §6 stage-1 rules 1–2).
pub(crate) fn parse(bytes: &[u8]) -> Result<Header, crate::NotAProduct> {
    if bytes.len() < HEADER_LEN {
        return Err((FormatError::TruncatedHeader).into());
    }
    if &bytes[0..4] != MAGIC {
        return Err((FormatError::BadMagic).into());
    }
    let major = bytes[4];
    let minor = bytes[5];
    if major != FORMAT_VERSION_MAJOR {
        return Err((FormatError::UnsupportedMajorVersion(major)).into());
    }
    let endianness = bytes[6];
    if endianness != ENDIANNESS_LITTLE {
        return Err((FormatError::UnsupportedEndianness(endianness)).into());
    }
    let alignment_log2 = bytes[7];
    if !(MIN_ALIGNMENT_LOG2..=MAX_ALIGNMENT_LOG2).contains(&alignment_log2) {
        return Err((FormatError::UnsupportedAlignment(alignment_log2)).into());
    }
    let total_len = read_u64_le(bytes, 8);
    let actual = bytes.len() as u64;
    if total_len != actual {
        return Err((FormatError::TotalLenMismatch {
            declared: total_len,
            actual,
        })
        .into());
    }
    let section_count = read_u32_le(bytes, 16);
    let flags = read_u32_le(bytes, 20);
    let unknown_mandatory = flags & MANDATORY_FEATURE_SPACE & !KNOWN_MANDATORY_FEATURES;
    if unknown_mandatory != 0 {
        return Err((FormatError::UnknownMandatoryFeature(unknown_mandatory)).into());
    }
    let artifact_cid = read_cid(bytes, ARTIFACT_CID_OFFSET);
    let head_cid = read_cid(bytes, HEAD_CID_OFFSET);
    Ok(Header {
        major,
        minor,
        alignment_log2,
        total_len,
        section_count,
        flags,
        artifact_cid,
        head_cid,
    })
}
