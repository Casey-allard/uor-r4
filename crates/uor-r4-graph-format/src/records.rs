//! Packed wire records: the v0 draft-line `PackedNode` (30 bytes),
//! `PackedEdge` (16 bytes), and the EMIT/EXCT storage descriptor
//! (4 bytes). Widths are the draft-line freeze candidates from PDF §21,
//! decoded on demand with explicit little-endian reads — no unsafe, no
//! transmute, no heap (RFC §1 rules 3–5).

use crate::error::FormatError;
use crate::header::{read_i16_le, read_i32_le, read_u16_le, read_u32_le};
use crate::types::{Depth, NodeId, Radius, ScoreQ, SectionId};

/// Unknown edge-kind values with this bit set are treated as optional and may
/// be ignored by readers that do not understand them.
pub const EDGE_KIND_OPTIONAL_BIT: u8 = 0x80;

/// Stable edge-kind discriminants for the shared-node edge algebra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EdgeKind {
    /// Refinement/abstraction hierarchy edge.
    RefinementAbstraction = 0,
    /// Similarity / overlap relation over the same node space.
    Similarity = 1,
    /// Predictive forward transition.
    PredictiveForward = 2,
    /// Causal influence.
    Causal = 3,
    /// Temporal ordering.
    Temporal = 4,
    /// Constraint relation.
    Constraint = 5,
    /// Goal-progress relation.
    GoalProgress = 6,
    /// Evidence / provenance support.
    EvidenceProvenance = 7,
    /// Evidential predecessor relation.
    EvidentialPredecessor = 8,
}

impl EdgeKind {
    /// Decode a known mandatory edge kind.
    pub const fn from_raw(kind: u8) -> Option<Self> {
        match kind {
            0 => Some(Self::RefinementAbstraction),
            1 => Some(Self::Similarity),
            2 => Some(Self::PredictiveForward),
            3 => Some(Self::Causal),
            4 => Some(Self::Temporal),
            5 => Some(Self::Constraint),
            6 => Some(Self::GoalProgress),
            7 => Some(Self::EvidenceProvenance),
            8 => Some(Self::EvidentialPredecessor),
            _ => None,
        }
    }

    /// Whether this edge algebra is directed.
    pub const fn directed(self) -> bool {
        !matches!(self, Self::Similarity)
    }

    /// Whether this edge algebra may form cycles.
    pub const fn may_cycle(self) -> bool {
        matches!(
            self,
            Self::Similarity | Self::PredictiveForward | Self::Constraint
        )
    }

    /// Whether this edge algebra requires reverse-index coverage checks.
    pub const fn requires_reverse_index(self) -> bool {
        !matches!(self, Self::Similarity)
    }

    /// Whether this edge algebra requires a contribution id in
    /// `PackedEdge.reserved` under the edge-algebra-v1 feature bit.
    pub const fn requires_contribution_id(self) -> bool {
        matches!(
            self,
            Self::PredictiveForward
                | Self::Causal
                | Self::Temporal
                | Self::Constraint
                | Self::GoalProgress
                | Self::EvidenceProvenance
                | Self::EvidentialPredecessor
        )
    }
}

/// True when an unknown edge kind is explicitly optional.
pub const fn is_optional_edge_kind(kind: u8) -> bool {
    kind & EDGE_KIND_OPTIONAL_BIT != 0
}

/// Packed node record size in bytes.
pub const PACKED_NODE_LEN: usize = 30;
/// Packed canonical edge record size in bytes.
pub const PACKED_EDGE_LEN: usize = 16;
/// Reverse-index entry size: one u32 edge ID per canonical edge.
pub(crate) const REVERSE_INDEX_ENTRY_LEN: usize = 4;
/// EMIT/EXCT storage descriptor size in bytes.
pub const STORAGE_DESCRIPTOR_LEN: usize = 4;
/// Packed tombstone record size in bytes.
pub const PACKED_TOMBSTONE_LEN: usize = 8;
/// Packed route translation record size in bytes.
pub const PACKED_ROUTE_TRANSLATION_LEN: usize = 12;

/// One packed region record (PDF §21), 30 bytes little-endian:
///
/// ```text
/// offset  size  field
/// 0       u32   child_start          (range over the canonical edge array)
/// 4       u16   child_len
/// 6       u32   forward_start        (range over the EDGE reverse index)
/// 10      u16   forward_len
/// 12      u32   emission_start       (byte range over the EMIT remainder)
/// 16      u16   emission_len
/// 18      u32   prototype_word_start (u64-word index into ROUT)
/// 22      u32   mask_word_start      (u64-word index into ROUT)
/// 26      u16   radius
/// 28      u8    depth
/// 29      u8    flags
/// ```
///
/// Ranges are section-relative (RFC §1 rule 3); the target region per
/// field is fixed by the v0 draft line as noted above and validated in
/// stage 2 (RFC §6 item 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedNode {
    /// First canonical-edge index of this node's child (refinement)
    /// range.
    pub child_start: u32,
    /// Number of child edges; must be ≤ HEAD `A`.
    pub child_len: u16,
    /// First reverse-index slot of this node's backward range.
    pub forward_start: u32,
    /// Number of reverse-index slots in the backward range.
    pub forward_len: u16,
    /// Byte offset into the EMIT remainder of this node's emission list.
    pub emission_start: u32,
    /// Byte length of the emission list; must be ≤ HEAD `E`.
    pub emission_len: u16,
    /// u64-word index into the ROUT section of the prototype words.
    pub prototype_word_start: u32,
    /// u64-word index into the ROUT section of the mask words.
    pub mask_word_start: u32,
    /// Calibrated acceptance radius (masked-Hamming bound).
    pub radius: Radius,
    /// Multiresolution depth; must be < HEAD `depth_count`.
    pub depth: Depth,
    /// Per-node flags (no bits defined in v0).
    pub flags: u8,
}

/// One packed canonical edge (16 bytes little-endian):
///
/// ```text
/// offset  size  field
/// 0       u32   src
/// 4       u32   dst
/// 8       i32   score_q (Q16.16, RFC §9.3)
/// 12      u8    kind
/// 13      u8    flags
/// 14      u16   reserved (0 in v0; contribution id under edge-algebra-v1)
/// ```
///
/// Stable edge ID = index in the canonical array (RFC §5 EDGE).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedEdge {
    /// Source region.
    pub src: NodeId,
    /// Target region.
    pub dst: NodeId,
    /// Quantized log-domain score (semantic Q16.16).
    pub score_q: ScoreQ,
    /// Edge kind discriminant (see [`EdgeKind`]); unknown kinds in the
    /// mandatory space are rejected, unknown kinds with
    /// [`EDGE_KIND_OPTIONAL_BIT`] set are optional.
    pub kind: u8,
    /// Per-edge flags (no bits defined in v0).
    pub flags: u8,
    /// Reserved (0 in v0; contribution id under edge-algebra-v1).
    pub reserved: u16,
}

/// Dyadic storage descriptor prefixing EMIT and EXCT (RFC §5/§9.3):
/// `{width u8, shift i8, zero_point i16}` where width 0/1/2 selects
/// i8/i16/i32 entries. Entries decode to semantic ScoreQ by shift+add
/// at table-read time — no multiply. The remainder of the section is
/// opaque to the v0 validator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageDescriptor {
    /// Entry width code: 0 = i8, 1 = i16, 2 = i32.
    pub width: u8,
    /// Dyadic left-shift applied at decode; `|shift| ≤ 31`.
    pub shift: i8,
    /// Zero point added at decode.
    pub zero_point: i16,
}

impl StorageDescriptor {
    /// Decode and validate the 4-byte descriptor prefixing `bytes`
    /// (RFC §6 item 8): the section must carry at least
    /// [`STORAGE_DESCRIPTOR_LEN`] bytes, `width ∈ {0,1,2}`, and
    /// `|shift| ≤ 31`.
    pub fn parse(section: SectionId, bytes: &[u8]) -> Result<Self, crate::NotAProduct> {
        if bytes.len() < STORAGE_DESCRIPTOR_LEN {
            return Err((FormatError::InvalidStorageDescriptor { section }).into());
        }
        let width = bytes[0];
        let shift = bytes[1] as i8;
        let zero_point = read_i16_le(bytes, 2);
        if width > 2 || (shift as i16).abs() > 31 {
            return Err((FormatError::InvalidStorageDescriptor { section }).into());
        }
        Ok(Self {
            width,
            shift,
            zero_point,
        })
    }
}

/// One packed tombstone (8 bytes little-endian):
///
/// ```text
/// offset  size  field
/// 0       u32   id
/// 4       u8    kind (0=node, 1=edge)
/// 5       u8    flags
/// 6       u16   reserved (0)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedTombstone {
    pub id: u32,
    pub kind: u8,
    pub flags: u8,
    pub reserved: u16,
}

/// One packed route translation (12 bytes little-endian):
///
/// ```text
/// offset  size  field
/// 0       u32   src_region
/// 4       u32   dst_region
/// 8       u8    map_kind
/// 9       u8    flags
/// 10      u16   reserved (0)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedRouteTranslation {
    pub src_region: NodeId,
    pub dst_region: NodeId,
    pub map_kind: u8,
    pub flags: u8,
    pub reserved: u16,
}

/// Decode one 30-byte packed node. Callers must have established that
/// `bytes.len() >= PACKED_NODE_LEN` (stage 2 validates the section size
/// before decoding, and the view iterators slice exact records), so the
/// indexing below cannot panic.
pub(crate) fn decode_node(bytes: &[u8]) -> PackedNode {
    PackedNode {
        child_start: read_u32_le(bytes, 0),
        child_len: read_u16_le(bytes, 4),
        forward_start: read_u32_le(bytes, 6),
        forward_len: read_u16_le(bytes, 10),
        emission_start: read_u32_le(bytes, 12),
        emission_len: read_u16_le(bytes, 16),
        prototype_word_start: read_u32_le(bytes, 18),
        mask_word_start: read_u32_le(bytes, 22),
        radius: Radius(read_u16_le(bytes, 26)),
        depth: Depth(bytes[28]),
        flags: bytes[29],
    }
}

/// Decode one 16-byte packed edge; same caller guarantee as
/// [`decode_node`].
pub(crate) fn decode_edge(bytes: &[u8]) -> PackedEdge {
    PackedEdge {
        src: NodeId(read_u32_le(bytes, 0)),
        dst: NodeId(read_u32_le(bytes, 4)),
        score_q: ScoreQ(read_i32_le(bytes, 8)),
        kind: bytes[12],
        flags: bytes[13],
        reserved: read_u16_le(bytes, 14),
    }
}

/// Decode one 8-byte packed tombstone.
pub(crate) fn decode_tombstone(bytes: &[u8]) -> PackedTombstone {
    PackedTombstone {
        id: read_u32_le(bytes, 0),
        kind: bytes[4],
        flags: bytes[5],
        reserved: read_u16_le(bytes, 6),
    }
}

/// Decode one 12-byte packed route translation.
pub(crate) fn decode_route_translation(bytes: &[u8]) -> PackedRouteTranslation {
    PackedRouteTranslation {
        src_region: NodeId(read_u32_le(bytes, 0)),
        dst_region: NodeId(read_u32_le(bytes, 4)),
        map_kind: bytes[8],
        flags: bytes[9],
        reserved: read_u16_le(bytes, 10),
    }
}
