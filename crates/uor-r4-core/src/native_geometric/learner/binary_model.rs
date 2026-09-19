//! Compact Binary Model Serialization (.rgm) & Mmap Serving (Milestone 13).
//!
//! Canonical 64-byte header `RgmHeader` + 8-byte aligned section table + payload.
//! High-efficiency binary serialization and zero-copy `memmap2` serving.
//! Cold-start initialization time < 50 microseconds.
//! Bit-exact parity with JSON reference model.

use super::embedding::canonical_h4_roots_q30;
use super::jepa_trainer::ExportedGeometricModel;
use super::transition_table::DiscreteServingTable;
use crate::native_geometric::engram::{
    hash_4gram, hash_5gram, hash_bigram, hash_skip, hash_trigram, EngramEntry, EngramTable,
};
use crate::native_geometric::hopf_metric::{HopfFiberPointQ30, UnitS2Q30, UnitS3Q30};
use crate::native_geometric::lattice_table::{
    HierarchicalLatticeTables, COARSE_ROOT_COUNT, COARSE_TABLE_SIZE,
};
use crate::native_geometric::vsa::{
    encode_attended_multiscale_context, Codebook, HierarchicalCodebook, Hypervector, LeafBucket,
};
use std::path::Path;

/// Helper: derive current syntactic register (quote parity and clause depth) from context tokens.
#[inline]
pub(crate) fn compute_syntactic_register_from_context(context: &[usize]) -> u8 {
    let mut q = 0u8;
    let mut d = 0u8;
    let start = context.len().saturating_sub(64);
    for &tok in &context[start..] {
        if tok < 258 {
            let b = if (2..=257).contains(&tok) {
                (tok - 2) as u8
            } else {
                tok as u8
            };
            match b {
                b'"' => q ^= 1,
                b'(' | b'[' | b'{' => d = (d + 1).min(3),
                b')' | b']' | b'}' => d = d.saturating_sub(1),
                b',' | b';' | b':' => {
                    if d == 0 {
                        d = 1;
                    }
                }
                b'.' | b'?' | b'!' => d = 0,
                _ => {}
            }
        }
    }
    (q & 1) | ((d & 3) << 1)
}

/// Magic 4-byte signature for RGM binary models.
pub const RGM_MAGIC: [u8; 4] = *b"RGM1";

/// Current RGM format specification version.
pub const RGM_VERSION: u16 = 1;

/// Canonical size of the RGM header (exactly 64 bytes).
pub const RGM_HEADER_SIZE: u16 = 64;

/// Feature flag: model includes hierarchical codebook for candidate shortlist routing.
pub const FLAG_HAS_HIERARCHICAL_CODEBOOK: u16 = 1 << 0;

/// Feature flag: model includes flat integer engram table for O(1) collocation retrieval.
pub const FLAG_HAS_ENGRAM_TABLE: u16 = 1 << 1;

/// Feature flag: model includes 2-level hierarchical lattice tables (120^3 + 480^2).
pub const FLAG_HAS_HIERARCHICAL_LATTICE: u16 = 1 << 2;

/// Feature flag: model includes discrete JEPA weights.
pub const FLAG_HAS_JEPA: u16 = 1 << 3;

/// Section ID for base parameters (token-to-root, bias, S2 readout).
pub const SECTION_BASE: u32 = 1;

/// Section ID for multi-lane discrete transition tables.
pub const SECTION_LANES: u32 = 2;

/// Section ID for discrete JEPA weights.
pub const SECTION_JEPA: u32 = 3;

/// Section ID for hierarchical lattice tables (120^3 coarse + 480^2 fine).
pub const SECTION_LATTICE: u32 = 4;

/// Section ID for flat integer engram table.
pub const SECTION_ENGRAM: u32 = 5;

/// Section ID for hierarchical codebook (anchors + sector leaf buckets).
pub const SECTION_CODEBOOK: u32 = 6;

/// Total number of standard sections in RGM v1 format.
pub const NUM_SECTIONS: usize = 6;

/// Canonical 64-byte header of an RGM binary model file.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RgmHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub header_size: u16,
    pub vocab_size: u32,
    pub num_lanes: u16,
    pub flags: u16,
    pub vsa_seed: u64,
    pub vsa_scale_q15: i16,
    pub _reserved: [u8; 2],
    pub section_count: u32,
    pub blake3_digest: [u8; 32],
}

/// 16-byte section header defining an 8-byte aligned payload section.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct RgmSectionHeader {
    pub section_id: u32,
    pub offset: u32,
    pub length: u32,
    pub _reserved: u32,
}

/// Number of `i8` entries in the LATTICE section's coarse root-trigram block, derived from
/// the section length instead of assumed.
///
/// LATTICE payload layout: `num_clusters u32 + reserved u32` (`8` bytes), then
/// `token_to_cluster u16 x vocab_size` padded to 8, then the coarse i8 block, padded to 2,
/// then the fine `i16 x num_clusters^2` block. The writer emits **either** the full coarse
/// table **or** none, so the residual after removing the aligned prefix and the fine block
/// is exactly the coarse length. An artifact with an absent coarse block is valid and scores
/// as if the coarse term were zero, which is what removing the tier means.
fn lattice_coarse_len(section_len: usize, vocab_size: usize, num_clusters: usize) -> usize {
    let mut prefix = 8 + vocab_size * 2;
    let rem = prefix % 8;
    if rem != 0 {
        prefix += 8 - rem;
    }
    let fine_bytes = 2 * num_clusters * num_clusters;
    section_len
        .saturating_sub(prefix)
        .saturating_sub(fine_bytes)
}

/// Error type for binary serialization, deserialization, and mmap operations.
#[derive(Debug)]
pub enum BinaryModelError {
    Io(std::io::Error),
    InvalidMagic([u8; 4]),
    UnsupportedVersion(u16),
    InvalidHeaderSize(u16),
    TruncatedHeader {
        expected: usize,
        actual: usize,
    },
    TruncatedSection {
        section_id: u32,
        expected: usize,
        actual: usize,
    },
    SectionNotFound(u32),
    MisalignedSection {
        section_id: u32,
        offset: usize,
        align: usize,
    },
    IntegrityCheckFailed,
    CorruptedData(&'static str),
}

impl std::fmt::Display for BinaryModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::InvalidMagic(m) => write!(f, "Invalid magic bytes: {:?}", m),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported version: {}", v),
            Self::InvalidHeaderSize(s) => write!(f, "Invalid header size: {}", s),
            Self::TruncatedHeader { expected, actual } => {
                write!(
                    f,
                    "Truncated header: expected {} bytes, got {}",
                    expected, actual
                )
            }
            Self::TruncatedSection {
                section_id,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Truncated section {}: expected {} bytes, got {}",
                    section_id, expected, actual
                )
            }
            Self::SectionNotFound(id) => write!(f, "Section {} not found", id),
            Self::MisalignedSection {
                section_id,
                offset,
                align,
            } => {
                write!(
                    f,
                    "Section {} at offset {} misaligned for alignment {}",
                    section_id, offset, align
                )
            }
            Self::IntegrityCheckFailed => write!(f, "BLAKE3 integrity digest verification failed"),
            Self::CorruptedData(msg) => write!(f, "Corrupted model data: {}", msg),
        }
    }
}

impl std::error::Error for BinaryModelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for BinaryModelError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[inline]
fn align_to_8(buf: &mut Vec<u8>) {
    let rem = buf.len() % 8;
    if rem != 0 {
        buf.resize(buf.len() + (8 - rem), 0);
    }
}

#[inline]
fn align_to_4(buf: &mut Vec<u8>) {
    let rem = buf.len() % 4;
    if rem != 0 {
        buf.resize(buf.len() + (4 - rem), 0);
    }
}

#[inline]
fn align_to_2(buf: &mut Vec<u8>) {
    let rem = buf.len() % 2;
    if rem != 0 {
        buf.resize(buf.len() + (2 - rem), 0);
    }
}

impl ExportedGeometricModel {
    /// Serializes model into canonical compact RGM binary representation.
    pub fn to_binary(&self) -> Result<Vec<u8>, BinaryModelError> {
        let mut buf = Vec::with_capacity(3 * 1024 * 1024);

        // 1. Placeholder for RgmHeader (64 bytes)
        buf.resize(RGM_HEADER_SIZE as usize, 0);

        // 2. Placeholder for Section Headers (6 * 16 = 96 bytes)
        let section_headers_offset = buf.len();
        buf.resize(
            RGM_HEADER_SIZE as usize + NUM_SECTIONS * std::mem::size_of::<RgmSectionHeader>(),
            0,
        );

        let mut section_headers = [RgmSectionHeader::default(); NUM_SECTIONS];

        let mut flags = 0u16;
        if self.hierarchical_codebook.is_some() {
            flags |= FLAG_HAS_HIERARCHICAL_CODEBOOK;
        }
        if self.engram_table.is_some() {
            flags |= FLAG_HAS_ENGRAM_TABLE;
        }
        if self.hierarchical_lattice.is_some() {
            flags |= FLAG_HAS_HIERARCHICAL_LATTICE;
        }
        let has_jepa = self.discrete_jepa_w_state != [0; 9]
            || self.discrete_jepa_w_token != [0; 9]
            || self.discrete_jepa_bias != [0; 3]
            || self.discrete_jepa_fiber_w_state != [0; 4]
            || self.discrete_jepa_fiber_w_token != [0; 4]
            || self.discrete_jepa_fiber_bias != [0; 2];
        if has_jepa {
            flags |= FLAG_HAS_JEPA;
        }

        // Section 1: Base
        align_to_8(&mut buf);
        let s1_offset = buf.len();
        buf.extend_from_slice(&self.token_to_root);
        align_to_4(&mut buf);
        for &bias in &self.discrete_bias {
            buf.extend_from_slice(&bias.to_le_bytes());
        }
        align_to_2(&mut buf);
        for readout in &self.discrete_s2_readout {
            for &r in readout {
                buf.extend_from_slice(&r.to_le_bytes());
            }
        }
        let s1_len = buf.len() - s1_offset;
        section_headers[0] = RgmSectionHeader {
            section_id: SECTION_BASE,
            offset: s1_offset as u32,
            length: s1_len as u32,
            _reserved: 0,
        };

        // Section 2: Lanes
        align_to_8(&mut buf);
        let s2_offset = buf.len();
        for table in &self.discrete_tables {
            buf.extend_from_slice(&table.scale_bits.to_le_bytes());
            for &score in &table.scores {
                buf.extend_from_slice(&score.to_le_bytes());
            }
        }
        let s2_len = buf.len() - s2_offset;
        section_headers[1] = RgmSectionHeader {
            section_id: SECTION_LANES,
            offset: s2_offset as u32,
            length: s2_len as u32,
            _reserved: 0,
        };

        // Section 3: JEPA
        align_to_8(&mut buf);
        let s3_offset = buf.len();
        for &w in &self.discrete_jepa_w_state {
            buf.extend_from_slice(&w.to_le_bytes());
        }
        for &w in &self.discrete_jepa_w_token {
            buf.extend_from_slice(&w.to_le_bytes());
        }
        for &b in &self.discrete_jepa_bias {
            buf.extend_from_slice(&b.to_le_bytes());
        }
        for &w in &self.discrete_jepa_fiber_w_state {
            buf.extend_from_slice(&w.to_le_bytes());
        }
        for &w in &self.discrete_jepa_fiber_w_token {
            buf.extend_from_slice(&w.to_le_bytes());
        }
        for &b in &self.discrete_jepa_fiber_bias {
            buf.extend_from_slice(&b.to_le_bytes());
        }
        align_to_8(&mut buf);
        let s3_len = buf.len() - s3_offset;
        section_headers[2] = RgmSectionHeader {
            section_id: SECTION_JEPA,
            offset: s3_offset as u32,
            length: s3_len as u32,
            _reserved: 0,
        };

        // Section 4: Hierarchical Lattice
        align_to_8(&mut buf);
        let s4_offset = buf.len();
        if let Some(lattice) = &self.hierarchical_lattice {
            buf.extend_from_slice(&(lattice.num_clusters as u32).to_le_bytes());
            buf.extend_from_slice(&0u32.to_le_bytes()); // reserved
            for &c in &lattice.token_to_cluster {
                buf.extend_from_slice(&c.to_le_bytes());
            }
            align_to_8(&mut buf);
            let coarse_u8: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    lattice.coarse_trigram.as_ptr() as *const u8,
                    lattice.coarse_trigram.len(),
                )
            };
            buf.extend_from_slice(coarse_u8);
            align_to_2(&mut buf);
            for &f in &lattice.fine_residual {
                buf.extend_from_slice(&f.to_le_bytes());
            }
        }
        let s4_len = buf.len() - s4_offset;
        section_headers[3] = RgmSectionHeader {
            section_id: SECTION_LATTICE,
            offset: s4_offset as u32,
            length: s4_len as u32,
            _reserved: 0,
        };

        // Section 5: Engram Table
        align_to_8(&mut buf);
        let s5_offset = buf.len();
        if let Some(engram) = &self.engram_table {
            let slot_cap = engram.entries.len() as u32;
            let entry_count = engram.count as u32;
            let cand_count = engram.candidates.len() as u32;
            buf.extend_from_slice(&slot_cap.to_le_bytes());
            buf.extend_from_slice(&entry_count.to_le_bytes());
            buf.extend_from_slice(&cand_count.to_le_bytes());
            buf.extend_from_slice(&0u32.to_le_bytes()); // reserved
            for entry in &engram.entries {
                buf.extend_from_slice(&entry.key.to_le_bytes());
                buf.extend_from_slice(&entry.candidate_offset.to_le_bytes());
                buf.push(entry.count);
                buf.push(entry.condition);
                buf.extend_from_slice(&entry._reserved);
            }
            align_to_4(&mut buf);
            for &(token, score) in &engram.candidates {
                buf.extend_from_slice(&token.to_le_bytes());
                buf.extend_from_slice(&score.to_le_bytes());
                buf.extend_from_slice(&[0u8; 2]);
            }
        }
        let s5_len = buf.len() - s5_offset;
        section_headers[4] = RgmSectionHeader {
            section_id: SECTION_ENGRAM,
            offset: s5_offset as u32,
            length: s5_len as u32,
            _reserved: 0,
        };

        // Section 6: Hierarchical Codebook
        align_to_8(&mut buf);
        let s6_offset = buf.len();
        if let Some(codebook) = &self.hierarchical_codebook {
            buf.extend_from_slice(&(codebook.vocab_size as u32).to_le_bytes());
            buf.extend_from_slice(&0u32.to_le_bytes()); // reserved
            for anchor in codebook.root_anchors.iter() {
                for &w in &anchor.data {
                    buf.extend_from_slice(&w.to_le_bytes());
                }
            }
            for sector in &codebook.sectors {
                buf.extend_from_slice(&(sector.len() as u16).to_le_bytes());
                for bucket in sector {
                    for &w in &bucket.centroid.data {
                        buf.extend_from_slice(&w.to_le_bytes());
                    }
                    buf.extend_from_slice(&(bucket.tokens.len() as u16).to_le_bytes());
                    for &tok in &bucket.tokens {
                        buf.extend_from_slice(&tok.to_le_bytes());
                    }
                }
            }
        }
        let s6_len = buf.len() - s6_offset;
        section_headers[5] = RgmSectionHeader {
            section_id: SECTION_CODEBOOK,
            offset: s6_offset as u32,
            length: s6_len as u32,
            _reserved: 0,
        };

        // Write section headers into buffer at offset 64
        for (i, sh) in section_headers.iter().enumerate() {
            let sh_off = section_headers_offset + i * 16;
            buf[sh_off..sh_off + 4].copy_from_slice(&sh.section_id.to_le_bytes());
            buf[sh_off + 4..sh_off + 8].copy_from_slice(&sh.offset.to_le_bytes());
            buf[sh_off + 8..sh_off + 12].copy_from_slice(&sh.length.to_le_bytes());
            buf[sh_off + 12..sh_off + 16].copy_from_slice(&sh._reserved.to_le_bytes());
        }

        // Compute BLAKE3 digest of payload after 64-byte header
        let payload_digest = blake3::hash(&buf[RGM_HEADER_SIZE as usize..]);

        let header = RgmHeader {
            magic: RGM_MAGIC,
            version: RGM_VERSION,
            header_size: RGM_HEADER_SIZE,
            vocab_size: self.vocab_size as u32,
            num_lanes: self.discrete_tables.len() as u16,
            flags,
            vsa_seed: self.vsa_seed,
            vsa_scale_q15: self.vsa_scale_q15,
            // `_reserved[0]` carries the VSA code mode; the rest of the reserved space stays zero
            // so the header size is unchanged and older artifacts (which read 0) remain valid.
            _reserved: [self.vsa_code_mode, 0],
            section_count: NUM_SECTIONS as u32,
            blake3_digest: *payload_digest.as_bytes(),
        };

        buf[0..4].copy_from_slice(&header.magic);
        buf[4..6].copy_from_slice(&header.version.to_le_bytes());
        buf[6..8].copy_from_slice(&header.header_size.to_le_bytes());
        buf[8..12].copy_from_slice(&header.vocab_size.to_le_bytes());
        buf[12..14].copy_from_slice(&header.num_lanes.to_le_bytes());
        buf[14..16].copy_from_slice(&header.flags.to_le_bytes());
        buf[16..24].copy_from_slice(&header.vsa_seed.to_le_bytes());
        buf[24..26].copy_from_slice(&header.vsa_scale_q15.to_le_bytes());
        buf[26..28].copy_from_slice(&header._reserved);
        buf[28..32].copy_from_slice(&header.section_count.to_le_bytes());
        buf[32..64].copy_from_slice(&header.blake3_digest);

        Ok(buf)
    }

    /// Saves model to a binary `.rgm` file on disk.
    pub fn save_to_binary(&self, path: impl AsRef<Path>) -> Result<(), BinaryModelError> {
        let bytes = self.to_binary()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Deserializes model from a binary `.rgm` byte slice.
    pub fn from_binary(bytes: &[u8]) -> Result<Self, BinaryModelError> {
        let min_size = RGM_HEADER_SIZE as usize + NUM_SECTIONS * 16;
        if bytes.len() < min_size {
            return Err(BinaryModelError::TruncatedHeader {
                expected: min_size,
                actual: bytes.len(),
            });
        }

        let magic: [u8; 4] = bytes[0..4].try_into().unwrap();
        if magic != RGM_MAGIC {
            return Err(BinaryModelError::InvalidMagic(magic));
        }

        let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
        if version != RGM_VERSION {
            return Err(BinaryModelError::UnsupportedVersion(version));
        }

        let header_size = u16::from_le_bytes(bytes[6..8].try_into().unwrap());
        if header_size != RGM_HEADER_SIZE {
            return Err(BinaryModelError::InvalidHeaderSize(header_size));
        }

        let vocab_size = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let num_lanes = u16::from_le_bytes(bytes[12..14].try_into().unwrap()) as usize;
        let flags = u16::from_le_bytes(bytes[14..16].try_into().unwrap());
        let vsa_seed = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let vsa_scale_q15 = i16::from_le_bytes(bytes[24..26].try_into().unwrap());
        // Byte 26 of the header is `_reserved[0]` and carries the VSA code mode, so the format
        // stays size-compatible with artifacts written before the field existed (those read 0).
        let vsa_code_mode = bytes[26];
        if vsa_code_mode > 1 {
            return Err(BinaryModelError::CorruptedData(
                "vsa_code_mode is not 0 (fixed) or 1 (learned-root codes)",
            ));
        }
        let section_count = u32::from_le_bytes(bytes[28..32].try_into().unwrap()) as usize;
        let blake3_digest: [u8; 32] = bytes[32..64].try_into().unwrap();

        // Validate BLAKE3 integrity hash
        let actual_digest = blake3::hash(&bytes[64..]);
        if *actual_digest.as_bytes() != blake3_digest {
            return Err(BinaryModelError::IntegrityCheckFailed);
        }

        let mut section_headers = Vec::with_capacity(section_count);
        for i in 0..section_count {
            let off = 64 + i * 16;
            let section_id = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
            let offset = u32::from_le_bytes(bytes[off + 4..off + 8].try_into().unwrap());
            let length = u32::from_le_bytes(bytes[off + 8..off + 12].try_into().unwrap());
            let _reserved = u32::from_le_bytes(bytes[off + 12..off + 16].try_into().unwrap());

            if (offset as usize) + (length as usize) > bytes.len() {
                return Err(BinaryModelError::TruncatedSection {
                    section_id,
                    expected: (offset as usize) + (length as usize),
                    actual: bytes.len(),
                });
            }
            section_headers.push(RgmSectionHeader {
                section_id,
                offset,
                length,
                _reserved,
            });
        }

        let find_sec = |id: u32| -> Result<&RgmSectionHeader, BinaryModelError> {
            section_headers
                .iter()
                .find(|s| s.section_id == id)
                .ok_or(BinaryModelError::SectionNotFound(id))
        };

        // Section 1: Base
        let s1 = find_sec(SECTION_BASE)?;
        let mut cur = s1.offset as usize;
        let token_to_root = bytes[cur..cur + vocab_size].to_vec();
        cur += vocab_size;
        let rem = cur % 4;
        if rem != 0 {
            cur += 4 - rem;
        }
        let mut discrete_bias = Vec::with_capacity(vocab_size);
        for _ in 0..vocab_size {
            discrete_bias.push(i32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()));
            cur += 4;
        }
        let rem2 = cur % 2;
        if rem2 != 0 {
            cur += 2 - rem2;
        }
        let mut discrete_s2_readout = Vec::with_capacity(vocab_size);
        for _ in 0..vocab_size {
            let mut arr = [0i16; 5];
            for j in 0..5 {
                arr[j] = i16::from_le_bytes(bytes[cur..cur + 2].try_into().unwrap());
                cur += 2;
            }
            discrete_s2_readout.push(arr);
        }

        // Section 2: Lanes
        let s2 = find_sec(SECTION_LANES)?;
        let mut cur_l = s2.offset as usize;
        let mut discrete_tables = Vec::with_capacity(num_lanes);
        for _ in 0..num_lanes {
            let scale_bits = u64::from_le_bytes(bytes[cur_l..cur_l + 8].try_into().unwrap());
            cur_l += 8;
            let mut scores = Vec::with_capacity(14400);
            for _ in 0..14400 {
                scores.push(i16::from_le_bytes(
                    bytes[cur_l..cur_l + 2].try_into().unwrap(),
                ));
                cur_l += 2;
            }
            discrete_tables.push(DiscreteServingTable { scores, scale_bits });
        }

        // Section 3: JEPA
        let s3 = find_sec(SECTION_JEPA)?;
        let mut cur_j = s3.offset as usize;
        let mut discrete_jepa_w_state = [0i16; 9];
        for slot in &mut discrete_jepa_w_state {
            *slot = i16::from_le_bytes(bytes[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }
        let mut discrete_jepa_w_token = [0i16; 9];
        for slot in &mut discrete_jepa_w_token {
            *slot = i16::from_le_bytes(bytes[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }
        let mut discrete_jepa_bias = [0i16; 3];
        for slot in &mut discrete_jepa_bias {
            *slot = i16::from_le_bytes(bytes[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }
        let mut discrete_jepa_fiber_w_state = [0i16; 4];
        for slot in &mut discrete_jepa_fiber_w_state {
            *slot = i16::from_le_bytes(bytes[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }
        let mut discrete_jepa_fiber_w_token = [0i16; 4];
        for slot in &mut discrete_jepa_fiber_w_token {
            *slot = i16::from_le_bytes(bytes[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }
        let mut discrete_jepa_fiber_bias = [0i16; 2];
        for slot in &mut discrete_jepa_fiber_bias {
            *slot = i16::from_le_bytes(bytes[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }

        // Section 4: Hierarchical Lattice
        let hierarchical_lattice = if flags & FLAG_HAS_HIERARCHICAL_LATTICE != 0 {
            let s4 = find_sec(SECTION_LATTICE)?;
            if s4.length > 0 {
                let mut cur_lat = s4.offset as usize;
                let num_clusters =
                    u32::from_le_bytes(bytes[cur_lat..cur_lat + 4].try_into().unwrap()) as usize;
                cur_lat += 8; // num_clusters + reserved

                let mut token_to_cluster = Vec::with_capacity(vocab_size);
                for _ in 0..vocab_size {
                    token_to_cluster.push(u16::from_le_bytes(
                        bytes[cur_lat..cur_lat + 2].try_into().unwrap(),
                    ));
                    cur_lat += 2;
                }
                let rem = cur_lat % 8;
                if rem != 0 {
                    cur_lat += 8 - rem;
                }

                let coarse_len = lattice_coarse_len(s4.length as usize, vocab_size, num_clusters);
                if coarse_len != 0 && coarse_len != COARSE_TABLE_SIZE {
                    return Err(BinaryModelError::CorruptedData(
                        "lattice coarse block is neither absent nor the full table",
                    ));
                }
                let mut coarse_trigram = vec![0i8; coarse_len];
                if coarse_len > 0 {
                    let coarse_u8: &[u8] = &bytes[cur_lat..cur_lat + coarse_len];
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            coarse_u8.as_ptr(),
                            coarse_trigram.as_mut_ptr() as *mut u8,
                            coarse_len,
                        );
                    }
                }
                cur_lat += coarse_len;
                let rem2 = cur_lat % 2;
                if rem2 != 0 {
                    cur_lat += 2 - rem2;
                }

                let fine_len = num_clusters * num_clusters;
                let mut fine_residual = Vec::with_capacity(fine_len);
                for _ in 0..fine_len {
                    fine_residual.push(i16::from_le_bytes(
                        bytes[cur_lat..cur_lat + 2].try_into().unwrap(),
                    ));
                    cur_lat += 2;
                }

                Some(HierarchicalLatticeTables {
                    num_clusters,
                    token_to_cluster,
                    coarse_trigram,
                    fine_residual,
                })
            } else {
                None
            }
        } else {
            None
        };

        // Section 5: Engram Table
        let engram_table = if flags & FLAG_HAS_ENGRAM_TABLE != 0 {
            let s5 = find_sec(SECTION_ENGRAM)?;
            if s5.length > 0 {
                let mut cur_e = s5.offset as usize;
                let slot_capacity =
                    u32::from_le_bytes(bytes[cur_e..cur_e + 4].try_into().unwrap()) as usize;
                let entry_count =
                    u32::from_le_bytes(bytes[cur_e + 4..cur_e + 8].try_into().unwrap()) as usize;
                let cand_count =
                    u32::from_le_bytes(bytes[cur_e + 8..cur_e + 12].try_into().unwrap()) as usize;
                cur_e += 16;

                let mut entries = Vec::with_capacity(slot_capacity);
                for _ in 0..slot_capacity {
                    let key = u32::from_le_bytes(bytes[cur_e..cur_e + 4].try_into().unwrap());
                    let candidate_offset =
                        u32::from_le_bytes(bytes[cur_e + 4..cur_e + 8].try_into().unwrap());
                    let count = bytes[cur_e + 8];
                    let condition = bytes[cur_e + 9];
                    let _reserved = [bytes[cur_e + 10], bytes[cur_e + 11]];
                    cur_e += 12;
                    entries.push(EngramEntry {
                        key,
                        candidate_offset,
                        count,
                        condition,
                        _reserved,
                    });
                }
                let rem = cur_e % 4;
                if rem != 0 {
                    cur_e += 4 - rem;
                }

                let mut candidates = Vec::with_capacity(cand_count);
                for _ in 0..cand_count {
                    let token = u32::from_le_bytes(bytes[cur_e..cur_e + 4].try_into().unwrap());
                    let score = i16::from_le_bytes(bytes[cur_e + 4..cur_e + 6].try_into().unwrap());
                    cur_e += 8;
                    candidates.push((token, score));
                }

                Some(EngramTable {
                    mask: slot_capacity.saturating_sub(1),
                    count: entry_count,
                    entries,
                    candidates,
                })
            } else {
                None
            }
        } else {
            None
        };

        // Section 6: Hierarchical Codebook
        let hierarchical_codebook = if flags & FLAG_HAS_HIERARCHICAL_CODEBOOK != 0 {
            let s6 = find_sec(SECTION_CODEBOOK)?;
            if s6.length > 0 {
                let mut cur_c = s6.offset as usize;
                let code_vocab =
                    u32::from_le_bytes(bytes[cur_c..cur_c + 4].try_into().unwrap()) as usize;
                cur_c += 8;

                let mut root_anchors_vec = Vec::with_capacity(120);
                for _ in 0..120 {
                    let mut words = [0u64; 64];
                    for w in &mut words {
                        *w = u64::from_le_bytes(bytes[cur_c..cur_c + 8].try_into().unwrap());
                        cur_c += 8;
                    }
                    root_anchors_vec.push(Hypervector::<64>::from_words(words));
                }
                let root_anchors: Box<[Hypervector<64>; 120]> =
                    root_anchors_vec.into_boxed_slice().try_into().unwrap();

                let mut sectors: [Vec<LeafBucket<64>>; 120] = core::array::from_fn(|_| Vec::new());
                for sector in sectors.iter_mut() {
                    let bucket_count =
                        u16::from_le_bytes(bytes[cur_c..cur_c + 2].try_into().unwrap()) as usize;
                    cur_c += 2;
                    let mut buckets = Vec::with_capacity(bucket_count);
                    for _ in 0..bucket_count {
                        let mut words = [0u64; 64];
                        for w in &mut words {
                            *w = u64::from_le_bytes(bytes[cur_c..cur_c + 8].try_into().unwrap());
                            cur_c += 8;
                        }
                        let centroid = Hypervector::<64>::from_words(words);
                        let tok_count =
                            u16::from_le_bytes(bytes[cur_c..cur_c + 2].try_into().unwrap())
                                as usize;
                        cur_c += 2;
                        let mut tokens = Vec::with_capacity(tok_count);
                        for _ in 0..tok_count {
                            tokens.push(u32::from_le_bytes(
                                bytes[cur_c..cur_c + 4].try_into().unwrap(),
                            ));
                            cur_c += 4;
                        }
                        buckets.push(LeafBucket { centroid, tokens });
                    }
                    *sector = buckets;
                }

                Some(HierarchicalCodebook {
                    vocab_size: code_vocab,
                    sectors,
                    root_anchors,
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(ExportedGeometricModel {
            vocab_size,
            token_to_root,
            discrete_tables,
            discrete_bias,
            discrete_s2_readout,
            discrete_jepa_w_state,
            discrete_jepa_w_token,
            discrete_jepa_bias,
            discrete_jepa_fiber_w_state,
            discrete_jepa_fiber_w_token,
            discrete_jepa_fiber_bias,
            vsa_seed,
            vsa_scale_q15,
            // The hierarchical codebook read from the file was built in the export-time code
            // space; `prepare_vsa_code_mode` replaces it when the mode requires a different one.
            vsa_code_mode,
            hierarchical_codebook,
            engram_table,
            hierarchical_lattice,
        })
    }

    /// Loads model from a binary `.rgm` file on disk.
    pub fn load_from_binary_file(path: impl AsRef<Path>) -> Result<Self, BinaryModelError> {
        let bytes = std::fs::read(path)?;
        Self::from_binary(&bytes)
    }
}

/// Zero-copy memory-mapped geometric model for low-latency (< 50 µs) cold-start serving.
pub struct MmapGeometricModel {
    mmap: memmap2::Mmap,
    header: RgmHeader,
    /// VSA token-code mode read from the header's reserved byte (0 = fixed hash, 1 = learned-root
    /// codes). `0` for artifacts written before the field existed.
    vsa_code_mode: u8,

    // Section 1: Base
    token_to_root_offset: usize,
    discrete_bias_offset: usize,
    discrete_s2_readout_offset: usize,

    // Section 2: Lanes
    lanes_offset: usize,

    // Section 3: JEPA
    jepa_w_state: [i16; 9],
    jepa_w_token: [i16; 9],
    jepa_bias: [i16; 3],
    jepa_fiber_w_state: [i16; 4],
    jepa_fiber_w_token: [i16; 4],
    jepa_fiber_bias: [i16; 2],

    // Section 4: Hierarchical Lattice
    lattice_num_clusters: usize,
    lattice_token_to_cluster_offset: usize,
    lattice_coarse_offset: usize,
    /// Number of `i8` coarse entries actually present; `0` means the coarse tier was removed.
    lattice_coarse_entries: usize,
    lattice_fine_offset: usize,

    // Section 5: Engram
    engram_slot_capacity: usize,
    engram_candidate_count: usize,
    engram_entries_offset: usize,
    engram_candidates_offset: usize,

    // Section 6: Hierarchical Codebook
    codebook_anchors_offset: usize,
}

impl MmapGeometricModel {
    /// Opens and memory-maps an `.rgm` binary model file from disk.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, BinaryModelError> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        Self::from_mmap(mmap)
    }

    /// Verifies the BLAKE3 integrity of the mapped payload on-demand.
    pub fn verify_integrity(&self) -> Result<(), BinaryModelError> {
        let actual_digest = blake3::hash(&self.mmap[64..]);
        if *actual_digest.as_bytes() != self.header.blake3_digest {
            return Err(BinaryModelError::IntegrityCheckFailed);
        }
        Ok(())
    }

    /// Constructs an `MmapGeometricModel` from an existing memory map.
    pub fn from_mmap(mmap: memmap2::Mmap) -> Result<Self, BinaryModelError> {
        let min_size = RGM_HEADER_SIZE as usize + NUM_SECTIONS * 16;
        if mmap.len() < min_size {
            return Err(BinaryModelError::TruncatedHeader {
                expected: min_size,
                actual: mmap.len(),
            });
        }

        let magic: [u8; 4] = mmap[0..4].try_into().unwrap();
        if magic != RGM_MAGIC {
            return Err(BinaryModelError::InvalidMagic(magic));
        }

        let version = u16::from_le_bytes(mmap[4..6].try_into().unwrap());
        if version != RGM_VERSION {
            return Err(BinaryModelError::UnsupportedVersion(version));
        }

        let header_size = u16::from_le_bytes(mmap[6..8].try_into().unwrap());
        if header_size != RGM_HEADER_SIZE {
            return Err(BinaryModelError::InvalidHeaderSize(header_size));
        }

        let vocab_size = u32::from_le_bytes(mmap[8..12].try_into().unwrap());
        let num_lanes = u16::from_le_bytes(mmap[12..14].try_into().unwrap());
        let flags = u16::from_le_bytes(mmap[14..16].try_into().unwrap());
        let vsa_seed = u64::from_le_bytes(mmap[16..24].try_into().unwrap());
        let vsa_scale_q15 = i16::from_le_bytes(mmap[24..26].try_into().unwrap());
        // `_reserved[0]` carries the VSA code mode; see `ExportedGeometricModel::vsa_code_mode`.
        let vsa_code_mode = mmap[26];
        if vsa_code_mode > 1 {
            return Err(BinaryModelError::CorruptedData(
                "vsa_code_mode is not 0 (fixed) or 1 (learned-root codes)",
            ));
        }
        let _reserved = [mmap[26], mmap[27]];
        let section_count = u32::from_le_bytes(mmap[28..32].try_into().unwrap());
        let blake3_digest: [u8; 32] = mmap[32..64].try_into().unwrap();

        let header = RgmHeader {
            magic,
            version,
            header_size,
            vocab_size,
            num_lanes,
            flags,
            vsa_seed,
            vsa_scale_q15,
            _reserved,
            section_count,
            blake3_digest,
        };

        let mut section_headers = [RgmSectionHeader::default(); NUM_SECTIONS];
        for (i, sh) in section_headers.iter_mut().enumerate() {
            let off = 64 + i * 16;
            sh.section_id = u32::from_le_bytes(mmap[off..off + 4].try_into().unwrap());
            sh.offset = u32::from_le_bytes(mmap[off + 4..off + 8].try_into().unwrap());
            sh.length = u32::from_le_bytes(mmap[off + 8..off + 12].try_into().unwrap());
            sh._reserved = u32::from_le_bytes(mmap[off + 12..off + 16].try_into().unwrap());

            if (sh.offset as usize) + (sh.length as usize) > mmap.len() {
                return Err(BinaryModelError::TruncatedSection {
                    section_id: sh.section_id,
                    expected: (sh.offset as usize) + (sh.length as usize),
                    actual: mmap.len(),
                });
            }
        }

        // Section 1 offsets
        let s1 = section_headers[0];
        let token_to_root_offset = s1.offset as usize;
        let mut cur = token_to_root_offset + vocab_size as usize;
        let rem = cur % 4;
        if rem != 0 {
            cur += 4 - rem;
        }
        let discrete_bias_offset = cur;
        cur += (vocab_size as usize) * 4;
        let rem2 = cur % 2;
        if rem2 != 0 {
            cur += 2 - rem2;
        }
        let discrete_s2_readout_offset = cur;

        // Section 2 offsets
        let s2 = section_headers[1];
        let lanes_offset = s2.offset as usize;

        // Section 3: JEPA
        let s3 = section_headers[2];
        let mut cur_j = s3.offset as usize;
        let mut jepa_w_state = [0i16; 9];
        for slot in &mut jepa_w_state {
            *slot = i16::from_le_bytes(mmap[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }
        let mut jepa_w_token = [0i16; 9];
        for slot in &mut jepa_w_token {
            *slot = i16::from_le_bytes(mmap[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }
        let mut jepa_bias = [0i16; 3];
        for slot in &mut jepa_bias {
            *slot = i16::from_le_bytes(mmap[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }
        let mut jepa_fiber_w_state = [0i16; 4];
        for slot in &mut jepa_fiber_w_state {
            *slot = i16::from_le_bytes(mmap[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }
        let mut jepa_fiber_w_token = [0i16; 4];
        for slot in &mut jepa_fiber_w_token {
            *slot = i16::from_le_bytes(mmap[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }
        let mut jepa_fiber_bias = [0i16; 2];
        for slot in &mut jepa_fiber_bias {
            *slot = i16::from_le_bytes(mmap[cur_j..cur_j + 2].try_into().unwrap());
            cur_j += 2;
        }

        // Section 4: Hierarchical Lattice
        let s4 = section_headers[3];
        let (
            lattice_num_clusters,
            lattice_token_to_cluster_offset,
            lattice_coarse_offset,
            lattice_coarse_entries,
            lattice_fine_offset,
        ) = if s4.length > 0 && flags & FLAG_HAS_HIERARCHICAL_LATTICE != 0 {
            let mut cur_lat = s4.offset as usize;
            let num_c = u32::from_le_bytes(mmap[cur_lat..cur_lat + 4].try_into().unwrap()) as usize;
            cur_lat += 8;
            let t2c_off = cur_lat;
            cur_lat += (vocab_size as usize) * 2;
            let rem = cur_lat % 8;
            if rem != 0 {
                cur_lat += 8 - rem;
            }
            let coarse_entries = lattice_coarse_len(s4.length as usize, vocab_size as usize, num_c);
            let coarse_off = cur_lat;
            cur_lat += coarse_entries;
            let rem2 = cur_lat % 2;
            if rem2 != 0 {
                cur_lat += 2 - rem2;
            }
            let fine_off = cur_lat;
            (num_c, t2c_off, coarse_off, coarse_entries, fine_off)
        } else {
            (0, 0, 0, 0, 0)
        };

        // Section 5: Engram Table
        let s5 = section_headers[4];
        let (
            engram_slot_capacity,
            engram_candidate_count,
            engram_entries_offset,
            engram_candidates_offset,
        ) = if s5.length > 0 && flags & FLAG_HAS_ENGRAM_TABLE != 0 {
            let mut cur_e = s5.offset as usize;
            let slot_cap = u32::from_le_bytes(mmap[cur_e..cur_e + 4].try_into().unwrap()) as usize;
            let _entry_count =
                u32::from_le_bytes(mmap[cur_e + 4..cur_e + 8].try_into().unwrap()) as usize;
            let cand_count =
                u32::from_le_bytes(mmap[cur_e + 8..cur_e + 12].try_into().unwrap()) as usize;
            cur_e += 16;
            let entries_off = cur_e;
            cur_e += slot_cap * 12;
            let rem = cur_e % 4;
            if rem != 0 {
                cur_e += 4 - rem;
            }
            let cands_off = cur_e;
            (slot_cap, cand_count, entries_off, cands_off)
        } else {
            (0, 0, 0, 0)
        };

        // Section 6: Hierarchical Codebook
        let s6 = section_headers[5];
        let codebook_anchors_offset =
            if s6.length > 0 && flags & FLAG_HAS_HIERARCHICAL_CODEBOOK != 0 {
                s6.offset as usize + 8
            } else {
                0
            };

        Ok(Self {
            mmap,
            header,
            vsa_code_mode,
            token_to_root_offset,
            discrete_bias_offset,
            discrete_s2_readout_offset,
            lanes_offset,
            jepa_w_state,
            jepa_w_token,
            jepa_bias,
            jepa_fiber_w_state,
            jepa_fiber_w_token,
            jepa_fiber_bias,
            lattice_num_clusters,
            lattice_token_to_cluster_offset,
            lattice_coarse_offset,
            lattice_coarse_entries,
            lattice_fine_offset,
            engram_slot_capacity,
            engram_candidate_count,
            engram_entries_offset,
            engram_candidates_offset,
            codebook_anchors_offset,
        })
    }

    /// Header of the model file.
    #[inline]
    pub fn header(&self) -> &RgmHeader {
        &self.header
    }

    /// VSA token-code mode declared by the artifact: `0` = fixed token-id hash,
    /// `1` = codes derived from the learned 120-root assignment.
    #[inline]
    pub fn vsa_code_mode(&self) -> u8 {
        self.vsa_code_mode
    }

    /// Number of discrete tokens in vocabulary.
    #[inline]
    pub fn vocab_size(&self) -> usize {
        self.header.vocab_size as usize
    }

    /// Number of attention lanes.
    #[inline]
    pub fn num_lanes(&self) -> usize {
        self.header.num_lanes as usize
    }

    /// Zero-copy slice of nearest canonical H4 roots for each vocabulary token.
    #[inline]
    pub fn token_to_root(&self) -> &[u8] {
        &self.mmap[self.token_to_root_offset..self.token_to_root_offset + self.vocab_size()]
    }

    /// Zero-copy slice of discrete emission biases.
    #[inline]
    pub fn discrete_bias(&self) -> &[i32] {
        let ptr = unsafe { self.mmap.as_ptr().add(self.discrete_bias_offset) as *const i32 };
        unsafe { std::slice::from_raw_parts(ptr, self.vocab_size()) }
    }

    /// Zero-copy slice of discrete S2 + U(1) 5D readout weights.
    #[inline]
    pub fn discrete_s2_readout(&self) -> &[[i16; 5]] {
        let ptr =
            unsafe { self.mmap.as_ptr().add(self.discrete_s2_readout_offset) as *const [i16; 5] };
        unsafe { std::slice::from_raw_parts(ptr, self.vocab_size()) }
    }

    /// Discrete scoring table for a specific lane.
    #[inline]
    pub fn lane_scores(&self, lane: usize) -> &[i16] {
        let off = self.lanes_offset + lane * 28808 + 8;
        let ptr = unsafe { self.mmap.as_ptr().add(off) as *const i16 };
        unsafe { std::slice::from_raw_parts(ptr, 14400) }
    }

    /// Quantization scale factor for a specific lane table.
    #[inline]
    pub fn lane_scale(&self, lane: usize) -> f64 {
        let off = self.lanes_offset + lane * 28808;
        let bits = u64::from_le_bytes(self.mmap[off..off + 8].try_into().unwrap());
        f64::from_bits(bits)
    }

    /// Zero-copy slice of the coarse root trigram table in i8 format, or `None` when the
    /// artifact was written without the coarse tier (an absent tier scores as zero).
    #[inline]
    pub fn coarse_trigram(&self) -> Option<&[i8]> {
        if self.header.flags & FLAG_HAS_HIERARCHICAL_LATTICE == 0
            || self.lattice_coarse_entries == 0
        {
            return None;
        }
        let ptr = unsafe { self.mmap.as_ptr().add(self.lattice_coarse_offset) as *const i8 };
        Some(unsafe { std::slice::from_raw_parts(ptr, self.lattice_coarse_entries) })
    }

    /// Zero-copy slice of fine cluster bigram residual table in i16 format.
    #[inline]
    pub fn fine_residual(&self) -> Option<&[i16]> {
        if self.header.flags & FLAG_HAS_HIERARCHICAL_LATTICE == 0 || self.lattice_num_clusters == 0
        {
            return None;
        }
        let ptr = unsafe { self.mmap.as_ptr().add(self.lattice_fine_offset) as *const i16 };
        Some(unsafe {
            std::slice::from_raw_parts(ptr, self.lattice_num_clusters * self.lattice_num_clusters)
        })
    }

    /// Zero-copy slice mapping each token to its fine cluster index.
    #[inline]
    pub fn token_to_cluster(&self) -> Option<&[u16]> {
        if self.header.flags & FLAG_HAS_HIERARCHICAL_LATTICE == 0 {
            return None;
        }
        let ptr =
            unsafe { self.mmap.as_ptr().add(self.lattice_token_to_cluster_offset) as *const u16 };
        Some(unsafe { std::slice::from_raw_parts(ptr, self.vocab_size()) })
    }

    /// Zero-copy slice of engram hash table entries.
    #[inline]
    pub fn engram_entries(&self) -> Option<&[EngramEntry]> {
        if self.header.flags & FLAG_HAS_ENGRAM_TABLE == 0 || self.engram_slot_capacity == 0 {
            return None;
        }
        let ptr =
            unsafe { self.mmap.as_ptr().add(self.engram_entries_offset) as *const EngramEntry };
        Some(unsafe { std::slice::from_raw_parts(ptr, self.engram_slot_capacity) })
    }

    /// Zero-copy slice of engram candidate pairs (token, score_q15).
    #[inline]
    pub fn engram_candidates(&self) -> Option<&[(u32, i16)]> {
        if self.header.flags & FLAG_HAS_ENGRAM_TABLE == 0 || self.engram_candidate_count == 0 {
            return None;
        }
        let ptr =
            unsafe { self.mmap.as_ptr().add(self.engram_candidates_offset) as *const (u32, i16) };
        Some(unsafe { std::slice::from_raw_parts(ptr, self.engram_candidate_count) })
    }

    /// Zero-copy slice of 120 canonical H4 root sector anchors.
    #[inline]
    pub fn root_anchors(&self) -> Option<&[[u64; 64]]> {
        if self.header.flags & FLAG_HAS_HIERARCHICAL_CODEBOOK == 0 {
            return None;
        }
        let ptr =
            unsafe { self.mmap.as_ptr().add(self.codebook_anchors_offset) as *const [u64; 64] };
        Some(unsafe { std::slice::from_raw_parts(ptr, 120) })
    }

    /// Constant-time lookup of continuation candidates from the engram table.
    #[inline]
    pub fn lookup_engram(&self, key: u32) -> Option<&[(u32, i16)]> {
        let entries = self.engram_entries()?;
        let candidates = self.engram_candidates()?;
        if key == 0 || entries.is_empty() {
            return None;
        }
        let mask = entries.len().saturating_sub(1);
        let mut slot = (key as usize) & mask;
        for _ in 0..16 {
            let entry = &entries[slot];
            if entry.key == key {
                let start = entry.candidate_offset as usize;
                let end = start + (entry.count as usize);
                if end <= candidates.len() {
                    return Some(&candidates[start..end]);
                }
                return None;
            }
            if entry.key == 0 {
                return None;
            }
            slot = (slot + 1) & mask;
        }
        None
    }

    /// Lookup trigram collocation candidates.
    #[inline]
    pub fn lookup_engram_trigram(
        &self,
        w_prev2: u32,
        w_prev: u32,
        w_curr: u32,
    ) -> Option<&[(u32, i16)]> {
        self.lookup_engram(hash_trigram(w_prev2, w_prev, w_curr))
    }

    /// Lookup bigram collocation candidates.
    #[inline]
    pub fn lookup_engram_bigram(&self, w_prev: u32, w_curr: u32) -> Option<&[(u32, i16)]> {
        self.lookup_engram(hash_bigram(w_prev, w_curr))
    }

    /// Lookup conditioned skip-bigram collocation candidates.
    #[inline]
    pub fn lookup_engram_skip(
        &self,
        w_skip: u32,
        w_curr: u32,
        condition: u8,
    ) -> Option<&[(u32, i16)]> {
        self.lookup_engram(hash_skip(w_skip, w_curr, condition))
    }

    /// Compute cumulative fixed-point S3 state and fiber-preserving Hopf projection.
    pub fn context_hopf_fiber_q30(&self, context: &[usize]) -> HopfFiberPointQ30 {
        let token_to_root = self.token_to_root();
        if token_to_root.is_empty() || context.is_empty() {
            return UnitS3Q30::IDENTITY.hopf_fiber_project();
        }
        let state = crate::native_geometric::learner::group_table::compose_context_roots(
            token_to_root,
            context,
        );
        canonical_h4_roots_q30()[state].hopf_fiber_project()
    }

    /// O(1) query-key scoring across all lanes with zero runtime matrix multiplications.
    pub fn score_compatibility(&self, query_token: usize, key_token: usize) -> i32 {
        let token_to_root = self.token_to_root();
        let q_root = token_to_root[query_token.min(self.vocab_size() - 1)] as usize;
        let k_root = token_to_root[key_token.min(self.vocab_size() - 1)] as usize;
        let mut total = 0_i32;
        let num_lanes = self.num_lanes();
        for l in 0..num_lanes {
            let scores = self.lane_scores(l);
            total += scores[q_root * 120 + k_root] as i32;
        }
        total
    }

    /// Quantized JEPA transition predicting next S2 state and fiber U(1) state.
    pub fn predict_jepa_step_q30(
        &self,
        s2_state: UnitS2Q30,
        fiber_u1: [i32; 2],
        token_s2: UnitS2Q30,
        token_u1: [i32; 2],
    ) -> (UnitS2Q30, [i32; 2]) {
        let ws = &self.jepa_w_state;
        let wt = &self.jepa_w_token;
        let b = &self.jepa_bias;

        let s2 = s2_state.0;
        let u = token_s2.0;

        let hat_vx = ((ws[0] as i64 * s2[0] as i64
            + ws[1] as i64 * s2[1] as i64
            + ws[2] as i64 * s2[2] as i64
            + wt[0] as i64 * u[0] as i64
            + wt[1] as i64 * u[1] as i64
            + wt[2] as i64 * u[2] as i64)
            >> 14)
            + ((b[0] as i64) << 16);
        let hat_vy = ((ws[3] as i64 * s2[0] as i64
            + ws[4] as i64 * s2[1] as i64
            + ws[5] as i64 * s2[2] as i64
            + wt[3] as i64 * u[0] as i64
            + wt[4] as i64 * u[1] as i64
            + wt[5] as i64 * u[2] as i64)
            >> 14)
            + ((b[1] as i64) << 16);
        let hat_vz = ((ws[6] as i64 * s2[0] as i64
            + ws[7] as i64 * s2[1] as i64
            + ws[8] as i64 * s2[2] as i64
            + wt[6] as i64 * u[0] as i64
            + wt[7] as i64 * u[1] as i64
            + wt[8] as i64 * u[2] as i64)
            >> 14)
            + ((b[2] as i64) << 16);

        let pred_s2 = UnitS2Q30::normalize([hat_vx, hat_vy, hat_vz]);

        let wfs = &self.jepa_fiber_w_state;
        let wft = &self.jepa_fiber_w_token;
        let bf = &self.jepa_fiber_bias;

        let hat_fx = ((wfs[0] as i64 * fiber_u1[0] as i64
            + wfs[1] as i64 * fiber_u1[1] as i64
            + wft[0] as i64 * token_u1[0] as i64
            + wft[1] as i64 * token_u1[1] as i64)
            >> 14)
            + ((bf[0] as i64) << 16);
        let hat_fy = ((wfs[2] as i64 * fiber_u1[0] as i64
            + wfs[3] as i64 * fiber_u1[1] as i64
            + wft[2] as i64 * token_u1[0] as i64
            + wft[3] as i64 * token_u1[1] as i64)
            >> 14)
            + ((bf[1] as i64) << 16);

        let pred_u1 = UnitS2Q30::normalize_u1([hat_fx, hat_fy]);
        (pred_s2, pred_u1)
    }

    /// Compute VSA context hypervector directly from a ring buffer without heap allocations.
    pub fn context_vsa_from_ring(
        &self,
        codebook: &Codebook<64>,
        ring: &[u32],
        cursor: usize,
        length: usize,
    ) -> Hypervector<64> {
        if length == 0 || ring.is_empty() {
            return Hypervector::<64>::zero();
        }
        let mut buf = [0u32; 64];
        let window = length.min(64).min(ring.len());
        let cursor = cursor % ring.len();
        for i in 0..window {
            let lag = window - i;
            let index = if cursor >= lag {
                cursor - lag
            } else {
                ring.len() - (lag - cursor)
            };
            buf[i] = ring[index];
        }
        encode_attended_multiscale_context(&buf[..window], codebook, 64)
    }

    /// Evaluates candidate score with zero runtime floats, zero runtime GEMM, and zero allocations.
    pub fn score_context_candidate(
        &self,
        context: &[usize],
        candidate: usize,
        fiber_pt: HopfFiberPointQ30,
    ) -> i32 {
        self.score_context_candidate_with_vsa(context, candidate, fiber_pt, None)
    }

    /// Evaluates candidate score with optional precomputed VSA context vector.
    pub fn score_context_candidate_with_vsa(
        &self,
        context: &[usize],
        candidate: usize,
        fiber_pt: HopfFiberPointQ30,
        precomputed_vsa: Option<(&Codebook<64>, &Hypervector<64>)>,
    ) -> i32 {
        let token_to_root = self.token_to_root();
        if token_to_root.is_empty() {
            return self.discrete_bias().get(candidate).copied().unwrap_or(0);
        }
        let root_len = token_to_root.len();
        let cand_root = token_to_root[candidate.min(root_len - 1)] as usize;
        let mut total = self.discrete_bias().get(candidate).copied().unwrap_or(0);

        let ctx_len = context.len();
        let num_lanes = self.num_lanes();
        for l in 0..num_lanes {
            let lag = l + 1;
            if ctx_len >= lag {
                let ctx_token = context[ctx_len - lag];
                let ctx_root = token_to_root[ctx_token.min(root_len - 1)] as usize;
                let scores = self.lane_scores(l);
                total += scores[ctx_root * 120 + cand_root] as i32;
            }
        }

        if let Some(r) = self.discrete_s2_readout().get(candidate) {
            let s2 = fiber_pt.base.0;
            let u1 = fiber_pt.fiber_u1;
            let s2_proj = (s2[0] as i64 * r[0] as i64
                + s2[1] as i64 * r[1] as i64
                + s2[2] as i64 * r[2] as i64
                + u1[0] as i64 * r[3] as i64
                + u1[1] as i64 * r[4] as i64)
                >> 31;
            total += s2_proj as i32;
        }

        if self.header.vsa_scale_q15 != 0 && ctx_len > 0 {
            let sim_q15 = match precomputed_vsa {
                Some((cb, ctx_vec)) => {
                    if let Some(cand_vec) = cb.get_ref(candidate as u32) {
                        ctx_vec.bipolar_correlation_q15(cand_vec)
                    } else {
                        let cand_vec = cb.get(candidate as u32);
                        ctx_vec.bipolar_correlation_q15(&cand_vec)
                    }
                }
                None => {
                    let codebook =
                        Codebook::<64>::on_demand(self.vocab_size(), self.header.vsa_seed);
                    let mut buf = [0u32; 64];
                    let n = ctx_len.min(64);
                    for i in 0..n {
                        buf[i] = context[ctx_len - n + i] as u32;
                    }
                    let ctx_vec = encode_attended_multiscale_context(&buf[..n], &codebook, 64);
                    let cand_vec = codebook.get(candidate as u32);
                    ctx_vec.bipolar_correlation_q15(&cand_vec)
                }
            };
            let vsa_score = (self.header.vsa_scale_q15 as i32 * sim_q15 as i32) >> 16;
            total += vsa_score;
        }

        if self.header.flags & FLAG_HAS_ENGRAM_TABLE != 0 {
            if ctx_len >= 5 {
                let w_curr = context[ctx_len - 1] as u32;
                let w_prev = context[ctx_len - 2] as u32;
                let w_prev2 = context[ctx_len - 3] as u32;
                let w_prev3 = context[ctx_len - 4] as u32;
                let w_prev4 = context[ctx_len - 5] as u32;
                let key = hash_5gram(w_prev4, w_prev3, w_prev2, w_prev, w_curr);
                if let Some(cands) = self.lookup_engram(key) {
                    for &(c, q15) in cands {
                        if c == candidate as u32 {
                            total += q15 as i32;
                        }
                    }
                }
            }
            if ctx_len >= 4 {
                let w_curr = context[ctx_len - 1] as u32;
                let w_prev = context[ctx_len - 2] as u32;
                let w_prev2 = context[ctx_len - 3] as u32;
                let w_prev3 = context[ctx_len - 4] as u32;
                let key = hash_4gram(w_prev3, w_prev2, w_prev, w_curr);
                if let Some(cands) = self.lookup_engram(key) {
                    for &(c, q15) in cands {
                        if c == candidate as u32 {
                            total += q15 as i32;
                        }
                    }
                }
            }
            if ctx_len >= 3 {
                let w_curr = context[ctx_len - 1] as u32;
                let w_prev = context[ctx_len - 2] as u32;
                let w_prev2 = context[ctx_len - 3] as u32;
                let key = hash_trigram(w_prev2, w_prev, w_curr);
                if let Some(cands) = self.lookup_engram(key) {
                    for &(c, q15) in cands {
                        if c == candidate as u32 {
                            total += q15 as i32;
                        }
                    }
                }
            }
            if ctx_len >= 2 {
                let w_curr = context[ctx_len - 1] as u32;
                let w_prev = context[ctx_len - 2] as u32;
                let key = hash_bigram(w_prev, w_curr);
                if let Some(cands) = self.lookup_engram(key) {
                    for &(c, q15) in cands {
                        if c == candidate as u32 {
                            total += q15 as i32;
                        }
                    }
                }
            }
            let syn = compute_syntactic_register_from_context(context);
            for &k in &[2, 4, 8] {
                if ctx_len >= k + 1 {
                    let w_curr = context[ctx_len - 1] as u32;
                    let w_skip = context[ctx_len - (k + 1)] as u32;
                    if let Some(cands) = self.lookup_engram_skip(w_skip, w_curr, syn) {
                        for &(c, q15) in cands {
                            if c == candidate as u32 {
                                total += q15 as i32;
                            }
                        }
                    }
                }
            }
        }

        if let (Some(coarse), Some(fine), Some(token_to_cluster)) = (
            self.coarse_trigram(),
            self.fine_residual(),
            self.token_to_cluster(),
        ) {
            let num_clusters = self.lattice_num_clusters;
            let cluster_of =
                |tok: usize| -> usize { token_to_cluster.get(tok).copied().unwrap_or(0) as usize };
            let coarse_score = |r0: usize, r1: usize, r2: usize| -> i8 {
                let idx = (r0 % COARSE_ROOT_COUNT) * 14400
                    + (r1 % COARSE_ROOT_COUNT) * 120
                    + (r2 % COARSE_ROOT_COUNT);
                coarse.get(idx).copied().unwrap_or(0)
            };
            let fine_score = |c0: usize, c1: usize| -> i16 {
                if num_clusters == 0 {
                    return 0;
                }
                let idx = (c0 % num_clusters) * num_clusters + (c1 % num_clusters);
                fine.get(idx).copied().unwrap_or(0)
            };

            let self_residual = crate::native_geometric::lattice_table::self_transition_surprisal(
                self.vocab_size(),
            );

            if ctx_len >= 2 {
                let prev_tok = context[ctx_len - 2];
                let curr_tok = context[ctx_len - 1];
                let is_self = candidate == curr_tok;
                let r_prev = token_to_root[prev_tok.min(root_len - 1)] as usize;
                let r_curr = token_to_root[curr_tok.min(root_len - 1)] as usize;
                let c_curr = cluster_of(curr_tok);
                let c_cand = cluster_of(candidate);
                let c = coarse_score(r_prev, r_curr, cand_root) as i32;
                let f = if is_self {
                    self_residual
                } else {
                    fine_score(c_curr, c_cand) as i32
                };
                total += (c << 6) + f;
            } else if ctx_len == 1 {
                let curr_tok = context[ctx_len - 1];
                let is_self = candidate == curr_tok;
                let r_curr = token_to_root[curr_tok.min(root_len - 1)] as usize;
                let c_curr = cluster_of(curr_tok);
                let c_cand = cluster_of(candidate);
                let c = coarse_score(r_curr, r_curr, cand_root) as i32;
                let f = if is_self {
                    self_residual
                } else {
                    fine_score(c_curr, c_cand) as i32
                };
                total += (c << 6) + f;
            }
        }

        total
    }
}
