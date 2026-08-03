//! Stage-1 structural validation (RFC §6) and the borrowed [`GraphView`].
//!
//! Validation decodes the fixed header and the 16-byte section table with
//! checked arithmetic only — no unsafe, no transmute, no pointer casts,
//! and no heap-resident deserialized structures (RFC §1 rules 3 and 5).
//! A [`GraphView`] can therefore be constructed only over bytes that have
//! passed every stage-1 invariant, plus the stage-2 semantic invariants
//! when a HEAD section is present (see [`GraphView::parse`]).

use crate::error::FormatError;
use crate::fmm::FmmTranslationTable;
use crate::head::Head;
use crate::header::{self, Header, HEADER_LEN, SECTION_ENTRY_LEN};
use crate::ngram::NgramTable;
use crate::records::{
    self, PackedEdge, PackedNode, PackedRouteTranslation, PackedTombstone, PACKED_EDGE_LEN,
    PACKED_NODE_LEN, PACKED_ROUTE_TRANSLATION_LEN, PACKED_TOMBSTONE_LEN,
};
use crate::stage2;
use crate::types::{ArtifactCid, SectionId};

/// One decoded section-table entry.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RawEntry {
    pub id: u32,
    pub flags: u32,
    pub offset: u32,
    pub length: u32,
}

/// Decode the `index`-th table entry.
///
/// Callers must have already established that the whole table lies within
/// `bytes` (stage-1 does so before decoding anything), so the indexing
/// below cannot panic.
pub(crate) fn decode_entry(bytes: &[u8], index: u32) -> RawEntry {
    let base = HEADER_LEN + index as usize * SECTION_ENTRY_LEN;
    RawEntry {
        id: header::read_u32_le(bytes, base),
        flags: header::read_u32_le(bytes, base + 4),
        offset: header::read_u32_le(bytes, base + 8),
        length: header::read_u32_le(bytes, base + 12),
    }
}

/// Byte offset one past the section table.
fn table_end(header: &Header) -> Result<u64, FormatError> {
    let table_len = (header.section_count as u64)
        .checked_mul(SECTION_ENTRY_LEN as u64)
        .ok_or(FormatError::SectionTableOutOfBounds)?;
    (HEADER_LEN as u64)
        .checked_add(table_len)
        .ok_or(FormatError::SectionTableOutOfBounds)
}

/// Run the full stage-1 structural validation of RFC §6 over `bytes`.
///
/// On success returns the decoded header; the table itself is re-decoded
/// on demand by the view (zero heap). Invariants, in check order:
///
/// 1. header checks (length, magic, major, endianness, alignment range,
///    `total_len == actual`, unknown mandatory feature bits);
/// 2. section table within `total_len`;
/// 3. entries strictly increasing by `section_id` (canonical order;
///    duplicates rejected);
/// 4. unknown section IDs: mandatory ones rejected, optional ones
///    ([`SectionId::OPTIONAL_BIT`]) kept as opaque bytes;
/// 5. every offset aligned to `1 << alignment_log2`;
/// 6. `offset + length` via checked u32 arithmetic;
/// 7. every section body within `total_len`;
/// 8. no section body overlapping the header/table region or another
///    section body.
pub(crate) fn validate(bytes: &[u8]) -> Result<Header, FormatError> {
    let header = header::parse(bytes)?;

    let table_end = table_end(&header)?;
    if table_end > header.total_len {
        return Err(FormatError::SectionTableOutOfBounds);
    }

    let align: u32 = 1 << header.alignment_log2;
    let mut prev_id: Option<u32> = None;
    for i in 0..header.section_count {
        let entry = decode_entry(bytes, i);

        if let Some(prev) = prev_id {
            if entry.id <= prev {
                return Err(FormatError::SectionsNotSorted);
            }
        }
        prev_id = Some(entry.id);

        let section = SectionId(entry.id);
        if !section.is_known() && section.mandatory() {
            return Err(FormatError::UnknownMandatorySection(entry.id));
        }

        if !entry.offset.is_multiple_of(align) {
            return Err(FormatError::SectionMisaligned);
        }

        let end = entry
            .offset
            .checked_add(entry.length)
            .ok_or(FormatError::OffsetOverflow)?;
        if u64::from(end) > header.total_len {
            return Err(FormatError::SectionOutOfBounds);
        }
        if u64::from(entry.offset) < table_end && entry.length > 0 {
            return Err(FormatError::SectionsOverlap);
        }

        // Pairwise body-overlap check in u64, robust against
        // not-yet-validated later entries (their own u32 overflow is
        // reported when the outer loop reaches them).
        for j in (i + 1)..header.section_count {
            let other = decode_entry(bytes, j);
            let other_end = u64::from(other.offset) + u64::from(other.length);
            if u64::from(entry.offset) < other_end && u64::from(other.offset) < u64::from(end) {
                return Err(FormatError::SectionsOverlap);
            }
        }
    }

    Ok(header)
}

/// A zero-copy view over a stage-1-validated R4G1 artifact.
///
/// Borrows the caller-owned (or memory-mapped) artifact bytes; section
/// payloads are exposed as borrowed slices only — nothing is deserialized
/// into heap structures (RFC §1 rule 5). Construct via
/// [`GraphView::parse`], which runs the full stage-1 validation first,
/// followed by stage-2 semantic validation whenever a HEAD section is
/// present. The decoded HEAD prefix (fixed size, `Copy`) is carried by
/// value; everything else decodes on demand.
#[derive(Debug, Clone, Copy)]
pub struct GraphView<'a> {
    bytes: &'a [u8],
    header: Header,
    head: Option<Head>,
}

impl<'a> GraphView<'a> {
    /// Validate `bytes` per RFC §6 — stage 1 always, then stage 2 when a
    /// HEAD section is present — and, on success, return the borrowed
    /// view. This is the only way to construct a `GraphView`.
    ///
    /// A container without HEAD stays stage-1-only (bootstrap fixtures,
    /// pure section carriers): [`GraphView::head`] returns `None` and the
    /// typed node/edge accessors report nothing, leaving the sections as
    /// opaque bytes.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, FormatError> {
        let header = validate(bytes)?;
        let mut view = Self {
            bytes,
            header,
            head: None,
        };
        view.head = stage2::validate(&view)?;
        Ok(view)
    }

    /// The decoded fixed header.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// The whole validated artifact byte range (`0..total_len`).
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Payload bytes of a section, looked up by ID (binary search over
    /// the canonically sorted table). Unknown optional sections are
    /// retained as opaque bytes and are reachable here by their raw ID.
    pub fn section(&self, id: SectionId) -> Option<&'a [u8]> {
        let mut lo: u32 = 0;
        let mut hi: u32 = self.header.section_count;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let entry = decode_entry(self.bytes, mid);
            if entry.id == id.0 {
                return self.payload(&entry);
            } else if entry.id < id.0 {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        None
    }

    /// Iterate over all present sections in canonical (sorted by ID)
    /// table order.
    pub fn sections(&self) -> Sections<'a> {
        Sections {
            bytes: self.bytes,
            section_count: self.header.section_count,
            next: 0,
        }
    }

    /// The decoded HEAD payload, when a HEAD section is present. Only
    /// `Some` for artifacts that passed stage-2 validation.
    pub fn head(&self) -> Option<Head> {
        self.head
    }

    /// Declared node count from HEAD, when present. Stage 2 guarantees
    /// the NODE section holds exactly this many records.
    pub fn node_count(&self) -> Option<u32> {
        self.head.map(|h| h.node_count())
    }

    /// Declared edge count from HEAD, when present. Stage 2 guarantees
    /// the EDGE section holds exactly this many canonical edges plus the
    /// same number of reverse-index entries.
    pub fn edge_count(&self) -> Option<u32> {
        self.head.map(|h| h.edge_count())
    }

    /// Decode one packed node by index, on demand. Returns `None` when
    /// `index >= node_count` or the artifact is stage-1-only (no HEAD).
    pub fn node(&self, index: u32) -> Option<PackedNode> {
        if index >= self.node_count()? {
            return None;
        }
        let bytes = self.section(SectionId::NODE)?;
        let start = index as usize * PACKED_NODE_LEN;
        let record = bytes.get(start..start + PACKED_NODE_LEN)?;
        Some(records::decode_node(record))
    }

    /// Iterate the packed node records in canonical order, decoding on
    /// demand. Empty when the artifact is stage-1-only (no HEAD).
    pub fn nodes(&self) -> Nodes<'a> {
        let count = self.node_count().unwrap_or(0);
        let bytes = match count {
            // Stage 2 guarantees NODE is present when the count is
            // non-zero; the fallback keeps the iterator empty rather
            // than panicking.
            0 => &[],
            _ => self.section(SectionId::NODE).unwrap_or(&[]),
        };
        Nodes {
            bytes,
            next: 0,
            remaining: count,
        }
    }

    /// Decode one packed canonical edge by index (its stable edge ID),
    /// on demand. Returns `None` when `index >= edge_count` or the
    /// artifact is stage-1-only (no HEAD).
    pub fn edge(&self, index: u32) -> Option<PackedEdge> {
        if index >= self.edge_count()? {
            return None;
        }
        let bytes = self.section(SectionId::EDGE)?;
        let start = index as usize * PACKED_EDGE_LEN;
        let record = bytes.get(start..start + PACKED_EDGE_LEN)?;
        Some(records::decode_edge(record))
    }

    /// Iterate the packed canonical edges in edge-ID order, decoding on
    /// demand. Empty when the artifact is stage-1-only (no HEAD).
    pub fn edges(&self) -> Edges<'a> {
        let count = self.edge_count().unwrap_or(0);
        let bytes = match count {
            0 => &[],
            _ => self.section(SectionId::EDGE).unwrap_or(&[]),
        };
        Edges {
            bytes,
            next: 0,
            remaining: count,
        }
    }

    /// Read one reverse-index entry (an edge ID) by position. The
    /// reverse index follows the canonical edge array inside the EDGE
    /// section. Returns `None` when `index >= edge_count` or the
    /// artifact is stage-1-only (no HEAD).
    pub fn reverse_edge_id(&self, index: u32) -> Option<u32> {
        let edge_count = self.edge_count()?;
        if index >= edge_count {
            return None;
        }
        let bytes = self.section(SectionId::EDGE)?;
        let start = (edge_count as usize * PACKED_EDGE_LEN) + (index as usize * 4);
        let entry = bytes.get(start..start + 4)?;
        Some(header::read_u32_le(entry, 0))
    }

    /// Read the parent CID from the PTCH section, if present.
    pub fn patch_parent_cid(&self) -> Option<ArtifactCid> {
        let bytes = self.section(SectionId::PTCH)?;
        let record = bytes.get(0..32)?;
        Some(header::read_cid(record, 0))
    }

    /// Iterate over tombstones in the PTCH section.
    pub fn patch_tombstones(&self) -> Tombstones<'a> {
        let bytes = self.section(SectionId::PTCH).unwrap_or(&[]);
        let (bytes, remaining) = if bytes.len() >= 32 {
            let tombstones = &bytes[32..];
            let remaining = (tombstones.len() / PACKED_TOMBSTONE_LEN) as u32;
            (tombstones, remaining)
        } else {
            (&[][..], 0)
        };
        Tombstones {
            bytes,
            next: 0,
            remaining,
        }
    }

    /// Iterate over route translations in the RTNX section.
    pub fn route_translations(&self) -> RouteTranslations<'a> {
        let bytes = self.section(SectionId::RTNX).unwrap_or(&[]);
        let remaining = (bytes.len() / PACKED_ROUTE_TRANSLATION_LEN) as u32;
        RouteTranslations {
            bytes,
            next: 0,
            remaining,
        }
    }

    /// Parse the optional packed lexical context table.
    pub fn ngram_table(&self) -> Result<Option<NgramTable<'a>>, FormatError> {
        self.section(SectionId::NGRAM)
            .map(NgramTable::parse)
            .transpose()
    }

    /// Borrow the optional compiler-folded FMM translation table.
    pub fn fmm_translation_table(&self) -> Result<Option<FmmTranslationTable<'a>>, FormatError> {
        self.section(SectionId::FMM)
            .map(FmmTranslationTable::parse)
            .transpose()
    }

    /// Recompute both integrity CIDs against the bytes and compare with
    /// the header fields (RFC §6 invariant 9).
    ///
    /// Convention (identical to the serializer's, see crate-level docs):
    /// `head_cid = blake3(HEAD body)`, `artifact_cid = blake3(bytes[56..
    /// total_len])`. Returns [`FormatError::MissingHead`] when HEAD is
    /// absent, [`FormatError::HeadCidMismatch`] /
    /// [`FormatError::ArtifactCidMismatch`] on digest mismatch.
    pub fn verify_cids(&self) -> Result<(), FormatError> {
        let head = self
            .section(SectionId::HEAD)
            .ok_or(FormatError::MissingHead)?;
        if blake3::hash(head).as_bytes() != &self.header.head_cid.0 {
            return Err(FormatError::HeadCidMismatch);
        }
        let artifact = blake3::hash(&self.bytes[header::ARTIFACT_HASH_START..]);
        if artifact.as_bytes() != &self.header.artifact_cid.0 {
            return Err(FormatError::ArtifactCidMismatch);
        }
        Ok(())
    }

    /// Verify that the loaded tokenizer bytes match the header's `tokenizer_cid`.
    pub fn verify_tokenizer_cid(&self, tokenizer_bytes: &[u8]) -> Result<(), FormatError> {
        let expected = self
            .head()
            .ok_or(FormatError::MissingHead)?
            .tokenizer_cid()
            .0;
        if expected != [0u8; 32] {
            let actual = blake3::hash(tokenizer_bytes);
            if expected != *actual.as_bytes() {
                return Err(FormatError::TokenizerCidMismatch);
            }
        }
        Ok(())
    }

    /// Slice out a validated entry's payload. Stage 1 guarantees the
    /// range lies within `bytes`, so `get` never fails here.
    fn payload(&self, entry: &RawEntry) -> Option<&'a [u8]> {
        let start = entry.offset as usize;
        let end = start + entry.length as usize;
        self.bytes.get(start..end)
    }
}

/// Iterator over the sections present in a [`GraphView`], in canonical
/// table order.
#[derive(Debug, Clone)]
pub struct Sections<'a> {
    bytes: &'a [u8],
    section_count: u32,
    next: u32,
}

impl<'a> Iterator for Sections<'a> {
    type Item = SectionRef<'a>;

    fn next(&mut self) -> Option<SectionRef<'a>> {
        if self.next >= self.section_count {
            return None;
        }
        let entry = decode_entry(self.bytes, self.next);
        self.next += 1;
        // Stage 1 guarantees the range lies within the bytes.
        let start = entry.offset as usize;
        let end = start + entry.length as usize;
        let payload = self.bytes.get(start..end)?;
        Some(SectionRef {
            id: SectionId(entry.id),
            flags: entry.flags,
            payload,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.section_count - self.next) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for Sections<'_> {}

/// Iterator over the packed node records of a [`GraphView`], decoding
/// each 30-byte record on demand (zero-copy, no heap).
#[derive(Debug, Clone)]
pub struct Nodes<'a> {
    bytes: &'a [u8],
    next: u32,
    remaining: u32,
}

impl Iterator for Nodes<'_> {
    type Item = PackedNode;

    fn next(&mut self) -> Option<PackedNode> {
        if self.remaining == 0 {
            return None;
        }
        let start = self.next as usize * PACKED_NODE_LEN;
        let record = self.bytes.get(start..start + PACKED_NODE_LEN)?;
        self.next += 1;
        self.remaining -= 1;
        Some(records::decode_node(record))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining as usize, Some(self.remaining as usize))
    }
}

impl ExactSizeIterator for Nodes<'_> {}

/// Iterator over the packed canonical edges of a [`GraphView`], in
/// edge-ID order, decoding each 16-byte record on demand (zero-copy,
/// no heap).
#[derive(Debug, Clone)]
pub struct Edges<'a> {
    bytes: &'a [u8],
    next: u32,
    remaining: u32,
}

impl Iterator for Edges<'_> {
    type Item = PackedEdge;

    fn next(&mut self) -> Option<PackedEdge> {
        if self.remaining == 0 {
            return None;
        }
        let start = self.next as usize * PACKED_EDGE_LEN;
        let record = self.bytes.get(start..start + PACKED_EDGE_LEN)?;
        self.next += 1;
        self.remaining -= 1;
        Some(records::decode_edge(record))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining as usize, Some(self.remaining as usize))
    }
}

impl ExactSizeIterator for Edges<'_> {}

/// One section as borrowed from the artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionRef<'a> {
    /// Section identifier (possibly an unknown optional ID).
    pub id: SectionId,
    /// Per-entry flags from the section table (no bits defined yet).
    pub flags: u32,
    /// Borrowed payload bytes.
    pub payload: &'a [u8],
}

/// Iterator over the packed tombstones in a [`GraphView`], decoding on demand.
#[derive(Debug, Clone)]
pub struct Tombstones<'a> {
    bytes: &'a [u8],
    next: u32,
    remaining: u32,
}

impl Iterator for Tombstones<'_> {
    type Item = PackedTombstone;

    fn next(&mut self) -> Option<PackedTombstone> {
        if self.remaining == 0 {
            return None;
        }
        let start = self.next as usize * PACKED_TOMBSTONE_LEN;
        let record = self.bytes.get(start..start + PACKED_TOMBSTONE_LEN)?;
        self.next += 1;
        self.remaining -= 1;
        Some(records::decode_tombstone(record))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining as usize, Some(self.remaining as usize))
    }
}

impl ExactSizeIterator for Tombstones<'_> {}

/// Iterator over the packed route translations in a [`GraphView`], decoding on demand.
#[derive(Debug, Clone)]
pub struct RouteTranslations<'a> {
    bytes: &'a [u8],
    next: u32,
    remaining: u32,
}

impl Iterator for RouteTranslations<'_> {
    type Item = PackedRouteTranslation;

    fn next(&mut self) -> Option<PackedRouteTranslation> {
        if self.remaining == 0 {
            return None;
        }
        let start = self.next as usize * PACKED_ROUTE_TRANSLATION_LEN;
        let record = self
            .bytes
            .get(start..start + PACKED_ROUTE_TRANSLATION_LEN)?;
        self.next += 1;
        self.remaining -= 1;
        Some(records::decode_route_translation(record))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining as usize, Some(self.remaining as usize))
    }
}

impl ExactSizeIterator for RouteTranslations<'_> {}
