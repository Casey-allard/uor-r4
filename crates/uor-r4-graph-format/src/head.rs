//! HEAD section payload (RFC §4): identities, bounded-work constants,
//! and graph dimensions, as a fixed 224-byte little-endian prefix (v0
//! draft-line layout; frozen in the first numbered version of the RFC).
//!
//! ```text
//! offset  size  field
//! 0       32B   teacher_cid
//! 32      32B   tokenizer_cid
//! 64      32B   corpus_construction_cid
//! 96      32B   corpus_certification_cid
//! 128     20B   hf_revision (ASCII hex, zero-padded right; opaque here)
//! 148     32B   compiler_version_cid
//! 180     u16   A — max frontier width
//! 182     u16   C — max candidates per active node
//! 184     u16   W — signature words per region
//! 186     u16   K — token shortlist size
//! 188     u32   E — max emission entries per region
//! 192     u32   D — max decision-program steps
//! 196     u32   node_count
//! 200     u32   edge_count
//! 204     u8    depth_count
//! 205     5B    fallback policy codes (one per ResolutionStatus)
//! 210     2B    reserved (0)
//! 212     u16   signature_bytes
//! 214     u16   min_runtime_major
//! 216     u16   min_runtime_minor
//! 218     u16   feature_bits_required
//! 220     u32   vocab_size
//! ```
//!
//! Length policy (draft line): the payload must be exactly
//! [`HEAD_PAYLOAD_LEN`] bytes. A longer payload is rejected
//! ([`FormatError::HeadTooLong`]) rather than silently ignoring the
//! trailing bytes, so that a future HEAD extension must arrive with a
//! format minor-version bump (RFC §8) instead of being mis-read by
//! draft-line parsers. The `reserved` field is decoded but not enforced
//! in v0, and `hf_revision` is carried as opaque bytes (ASCII-hex shape
//! validation is a later slice).

use crate::error::FormatError;
use crate::header::{read_cid, read_u16_le, read_u32_le};
use crate::types::ArtifactCid;

/// Fixed HEAD payload prefix length in bytes (v0 draft line).
pub const HEAD_PAYLOAD_LEN: usize = 224;

/// HEAD feature bit: shared-node multi-edge algebra v1 is required.
pub const FEATURE_EDGE_ALGEBRA_V1: u16 = 0x0001;

/// Mandatory HEAD feature bits known by this format version.
pub const KNOWN_FEATURE_BITS_REQUIRED: u16 = FEATURE_EDGE_ALGEBRA_V1;

/// Domain of one content-bound corpus partition identity in HEAD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpusPartitionRole {
    /// Positions used to construct compiler tables.
    Construction,
    /// Positions reserved for certification/evaluation.
    Certification,
}

/// Bind one ordered partition to the exact corpus stream.
///
/// Positions are serialized as fixed-width little-endian `u64` values. The
/// corpus byte lengths and partition role are domain-separated, so neither a
/// different split over the same stream nor the same indices over another
/// stream can reproduce this CID.
pub fn corpus_partition_cid(
    corpus_meta: &[u8],
    corpus_records: &[u8],
    role: CorpusPartitionRole,
    positions: &[u64],
) -> ArtifactCid {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"r4g1-corpus-partition/1");
    hasher.update(&[match role {
        CorpusPartitionRole::Construction => 0,
        CorpusPartitionRole::Certification => 1,
    }]);
    hasher.update(&(corpus_meta.len() as u64).to_le_bytes());
    hasher.update(corpus_meta);
    hasher.update(&(corpus_records.len() as u64).to_le_bytes());
    hasher.update(corpus_records);
    hasher.update(&(positions.len() as u64).to_le_bytes());
    for position in positions {
        hasher.update(&position.to_le_bytes());
    }
    ArtifactCid(*hasher.finalize().as_bytes())
}

/// Number of fallback policy codes: one per `ResolutionStatus` in
/// declaration order — Supported, Boundary, BackedOff, Novel,
/// Contradictory (RFC §4, decision D4).
pub const FALLBACK_POLICY_COUNT: usize = 5;

/// Decoded HEAD payload. Plain `Copy` data parsed once at
/// [`GraphView::parse`](crate::GraphView::parse) time and carried by
/// value in the view (fixed size, no heap). Fields are private with
/// getters so the wire offsets stay an implementation detail of this
/// module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Head {
    teacher_cid: ArtifactCid,
    tokenizer_cid: ArtifactCid,
    corpus_construction_cid: ArtifactCid,
    corpus_certification_cid: ArtifactCid,
    hf_revision: [u8; 20],
    compiler_version_cid: ArtifactCid,
    max_frontier_width: u16,
    max_candidates: u16,
    signature_words: u16,
    shortlist_size: u16,
    max_emission_entries: u32,
    max_program_steps: u32,
    node_count: u32,
    edge_count: u32,
    depth_count: u8,
    fallback_policies: [u8; FALLBACK_POLICY_COUNT],
    reserved: u16,
    signature_bytes: u16,
    min_runtime_major: u16,
    min_runtime_minor: u16,
    feature_bits_required: u16,
    vocab_size: u32,
}

impl Head {
    /// Decode the fixed 224-byte HEAD prefix. Exact-length policy per
    /// the module docs: shorter payloads return
    /// [`FormatError::HeadTooShort`], longer ones
    /// [`FormatError::HeadTooLong`].
    pub fn parse(bytes: &[u8]) -> Result<Self, crate::NotAProduct> {
        let actual = bytes.len() as u64;
        if bytes.len() < HEAD_PAYLOAD_LEN {
            return Err((FormatError::HeadTooShort { actual }).into());
        }
        if bytes.len() > HEAD_PAYLOAD_LEN {
            return Err((FormatError::HeadTooLong { actual }).into());
        }
        let mut hf_revision = [0u8; 20];
        hf_revision.copy_from_slice(&bytes[128..148]);
        let mut fallback_policies = [0u8; FALLBACK_POLICY_COUNT];
        fallback_policies.copy_from_slice(&bytes[205..210]);
        Ok(Self {
            teacher_cid: read_cid(bytes, 0),
            tokenizer_cid: read_cid(bytes, 32),
            corpus_construction_cid: read_cid(bytes, 64),
            corpus_certification_cid: read_cid(bytes, 96),
            hf_revision,
            compiler_version_cid: read_cid(bytes, 148),
            max_frontier_width: read_u16_le(bytes, 180),
            max_candidates: read_u16_le(bytes, 182),
            signature_words: read_u16_le(bytes, 184),
            shortlist_size: read_u16_le(bytes, 186),
            max_emission_entries: read_u32_le(bytes, 188),
            max_program_steps: read_u32_le(bytes, 192),
            node_count: read_u32_le(bytes, 196),
            edge_count: read_u32_le(bytes, 200),
            depth_count: bytes[204],
            fallback_policies,
            reserved: read_u16_le(bytes, 210),
            signature_bytes: read_u16_le(bytes, 212),
            min_runtime_major: read_u16_le(bytes, 214),
            min_runtime_minor: read_u16_le(bytes, 216),
            feature_bits_required: read_u16_le(bytes, 218),
            vocab_size: read_u32_le(bytes, 220),
        })
    }

    /// Teacher model CID.
    pub fn teacher_cid(&self) -> ArtifactCid {
        self.teacher_cid
    }

    /// Tokenizer identity CID.
    pub fn tokenizer_cid(&self) -> ArtifactCid {
        self.tokenizer_cid
    }

    /// Corpus construction root CID (decision D3).
    pub fn corpus_construction_cid(&self) -> ArtifactCid {
        self.corpus_construction_cid
    }

    /// Corpus certification (held-out) root CID (decision D3).
    pub fn corpus_certification_cid(&self) -> ArtifactCid {
        self.corpus_certification_cid
    }

    /// Pinned HF revision: 20 bytes, ASCII hex per the draft layout,
    /// zero-padded right. Carried as opaque bytes in v0.
    pub fn hf_revision(&self) -> &[u8; 20] {
        &self.hf_revision
    }

    /// Compiler name/version CID (RFC §7 records the compiler mode).
    pub fn compiler_version_cid(&self) -> ArtifactCid {
        self.compiler_version_cid
    }

    /// `A` — max frontier width (Theorem 4/9 bound).
    pub fn max_frontier_width(&self) -> u16 {
        self.max_frontier_width
    }

    /// `C` — max candidates per active node.
    pub fn max_candidates(&self) -> u16 {
        self.max_candidates
    }

    /// `W` — signature words (u64) per region.
    pub fn signature_words(&self) -> u16 {
        self.signature_words
    }

    /// `K` — token shortlist size.
    pub fn shortlist_size(&self) -> u16 {
        self.shortlist_size
    }

    /// `E` — max emission entries per region.
    pub fn max_emission_entries(&self) -> u32 {
        self.max_emission_entries
    }

    /// `D` — max decision-program steps.
    pub fn max_program_steps(&self) -> u32 {
        self.max_program_steps
    }

    /// Declared number of packed node records in the NODE section.
    pub fn node_count(&self) -> u32 {
        self.node_count
    }

    /// Declared number of canonical edges (and reverse-index entries)
    /// in the EDGE section.
    pub fn edge_count(&self) -> u32 {
        self.edge_count
    }

    /// Number of multiresolution depths; every node `depth` must be
    /// strictly below this.
    pub fn depth_count(&self) -> u8 {
        self.depth_count
    }

    /// Fallback policy codes, one per `ResolutionStatus` in declaration
    /// order: Supported, Boundary, BackedOff, Novel, Contradictory
    /// (RFC §4, decision D4).
    pub fn fallback_policies(&self) -> [u8; FALLBACK_POLICY_COUNT] {
        self.fallback_policies
    }

    /// Reserved field, decoded but not enforced in v0.
    pub fn reserved(&self) -> u16 {
        self.reserved
    }

    /// Declared signature width in bytes — byte-exact semantics over
    /// word-aligned storage: must satisfy
    /// `(W-1)*8 < signature_bytes <= `[`Head::signature_words`]` * 8`
    /// (stage-2 cross-check). The padding bytes between the signature
    /// and each region's W-word prototype/mask extent must be zero.
    pub fn signature_bytes(&self) -> u16 {
        self.signature_bytes
    }

    /// Minimum runtime major version able to load this artifact.
    pub fn min_runtime_major(&self) -> u16 {
        self.min_runtime_major
    }

    /// Minimum runtime minor version able to load this artifact.
    pub fn min_runtime_minor(&self) -> u16 {
        self.min_runtime_minor
    }

    /// Feature bits the runtime must support to load this artifact.
    pub fn feature_bits_required(&self) -> u16 {
        self.feature_bits_required
    }

    /// Compiled vocabulary size.
    pub fn vocab_size(&self) -> u32 {
        self.vocab_size
    }
}

#[cfg(test)]
mod tests {
    use super::{corpus_partition_cid, CorpusPartitionRole};

    #[test]
    fn partition_cid_binds_stream_role_order_and_positions() {
        let positions = [2, 5, 8];
        let construction = corpus_partition_cid(
            b"meta",
            b"records",
            CorpusPartitionRole::Construction,
            &positions,
        );
        assert_eq!(
            construction,
            corpus_partition_cid(
                b"meta",
                b"records",
                CorpusPartitionRole::Construction,
                &positions,
            )
        );
        assert_ne!(
            construction,
            corpus_partition_cid(
                b"meta",
                b"records",
                CorpusPartitionRole::Certification,
                &positions,
            )
        );
        assert_ne!(
            construction,
            corpus_partition_cid(
                b"meta",
                b"records!",
                CorpusPartitionRole::Construction,
                &positions,
            )
        );
        assert_ne!(
            construction,
            corpus_partition_cid(
                b"meta",
                b"records",
                CorpusPartitionRole::Construction,
                &[2, 8, 5],
            )
        );
    }
}
