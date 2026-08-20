//! Borrowed packed persistent-prompt-state descriptor — the optional `PSTATE`
//! section (#836, lowering the #835 segment lane).
//!
//! The #835 **segment lane** is *prompt-derived*: at fold time each prompt
//! content token is recorded into a bounded, halving-decay ring (fixed
//! capacity, saturating `ScoreQ` accumulation), and at score time a candidate
//! present in the ring receives a signed `ScoreQ` boost. There is no learned
//! per-token table to ship; what the compiler packs is the lane **descriptor**
//! — the fixed-capacity, decay, and score constants that let the deployed
//! R4Engine reconstruct the lane deterministically with caller-owned state.
//!
//! `PSTATE` therefore carries, in fixed field order (per
//! `docs/prompt_state_spec_835.md` §8): a schema version, the lane kind, the
//! per-lane capacity and decay shift, the score constants (`base_w`, `boost`),
//! the key-quantization identity, and an **optional** residual-weight table for
//! future table-bearing lanes (empty for the pure segment lane). All integer;
//! no multiply/divide/float is used to read it.
//!
//! The section is **optional** ([`crate::types::SectionId::OPTIONAL_BIT`]): an
//! artifact without it, or a reader that does not consume it, behaves exactly
//! as before (absent-section identity). It is additive and does not change
//! serving on its own — the deployed scorer consumes it in a later #836
//! increment. This module is the format foundation and its two-stage
//! validation.

use crate::error::FormatError;
use crate::types::ScoreQ;

pub const PSTATE_MAGIC: [u8; 4] = *b"PST1";
pub const PSTATE_VERSION: u16 = 1;
/// Descriptor header length (bytes). See the module docs for the field order.
pub const PSTATE_HEADER_LEN: usize = 40;
pub const PSTATE_ROW_LEN: usize = 16;
pub const PSTATE_ENTRY_LEN: usize = 8;

/// The lane kind a `PSTATE` section encodes. #836 lowers the segment lane; the
/// enum reserves the remaining #835 lanes for later increments without a format
/// break.
pub const LANE_SEGMENT: u8 = 0;

/// One residual-table row: a quantized content key and its bounded, canonical
/// list of candidate-support contributions (used by table-bearing lanes; the
/// pure segment lane ships zero rows).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PstateRow<'a> {
    bytes: &'a [u8],
    entries: &'a [u8],
}

impl<'a> PstateRow<'a> {
    /// The quantized whole-prompt content key.
    pub fn key(&self) -> u32 {
        read_u32(&self.bytes[0..4])
    }

    /// The bounded candidate-support entries, canonical by token.
    pub fn entries(&self) -> PstateEntries<'a> {
        PstateEntries {
            bytes: self.entries,
            remaining: (self.entries.len() / PSTATE_ENTRY_LEN) as u32,
        }
    }
}

/// One candidate-support contribution: a candidate token and its signed
/// `ScoreQ` residual (added by saturating integer addition on the hot path).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PstateEntry {
    pub token: u32,
    pub score_q: ScoreQ,
}

pub struct PstateEntries<'a> {
    bytes: &'a [u8],
    remaining: u32,
}

impl Iterator for PstateEntries<'_> {
    type Item = PstateEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let entry = self.bytes.get(..PSTATE_ENTRY_LEN)?;
        self.bytes = &self.bytes[PSTATE_ENTRY_LEN..];
        self.remaining -= 1;
        Some(PstateEntry {
            token: read_u32(&entry[..4]),
            score_q: ScoreQ::from_raw(i32::from_le_bytes(entry[4..8].try_into().ok()?)),
        })
    }
}

/// A borrowed, validated `PSTATE` descriptor over an artifact's section bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PstateTable<'a> {
    bytes: &'a [u8],
    row_count: u32,
    lane_kind: u8,
    decay_shift: u8,
    max_entries: u16,
    key_quant_id: u32,
    ring_capacity: u32,
    base_w: i32,
    boost: i32,
}

pub struct PstateRows<'a> {
    table: PstateTable<'a>,
    next: usize,
}

impl<'a> PstateTable<'a> {
    /// Two-stage validation: a header/descriptor check (magic, version,
    /// reserved zero, lane kind, capacity/decay ranges), then a per-row
    /// structural check (bounded entry counts, contiguous canonical layout,
    /// sorted keys and tokens, exact byte coverage). Never allocates; never
    /// panics on a recoverable input.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, crate::NotAProduct> {
        if bytes.len() < PSTATE_HEADER_LEN {
            return Err((FormatError::PstateTooShort).into());
        }
        if bytes[..4] != PSTATE_MAGIC {
            return Err((FormatError::PstateBadMagic).into());
        }
        if read_u16(&bytes[4..6]) != PSTATE_VERSION {
            return Err((FormatError::PstateUnsupportedVersion).into());
        }
        // reserved fields must be zero.
        if bytes[6..8].iter().any(|&b| b != 0)
            || bytes[18..20].iter().any(|&b| b != 0)
            || bytes[36..40].iter().any(|&b| b != 0)
        {
            return Err((FormatError::PstateNonZeroReserved).into());
        }
        let row_count = read_u32(&bytes[8..12]);
        let max_entries = read_u16(&bytes[12..14]);
        let decay_shift = bytes[14];
        let lane_kind = bytes[15];
        let _schema_version = read_u16(&bytes[16..18]);
        let key_quant_id = read_u32(&bytes[20..24]);
        let ring_capacity = read_u32(&bytes[24..28]);
        let base_w = i32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
        let boost = i32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);

        // Semantic descriptor checks.
        if lane_kind != LANE_SEGMENT {
            return Err((FormatError::PstateInvalidRow).into());
        }
        // A lane must have positive fixed capacity, and decay may not exceed
        // the ScoreQ width (a shift of >=32 would zero every weight, which is
        // never a meaningful descriptor).
        if ring_capacity == 0 || decay_shift >= 32 {
            return Err((FormatError::PstateInvalidRow).into());
        }

        let rows_len = (row_count as usize)
            .checked_mul(PSTATE_ROW_LEN)
            .ok_or(FormatError::PstateBounds)?;
        let entries_start = PSTATE_HEADER_LEN
            .checked_add(rows_len)
            .ok_or(FormatError::PstateBounds)?;
        if entries_start > bytes.len() {
            return Err((FormatError::PstateBounds).into());
        }

        let mut previous_key: Option<u32> = None;
        let mut expected_entry_start = entries_start;
        for index in 0..row_count as usize {
            let start = PSTATE_HEADER_LEN + index * PSTATE_ROW_LEN;
            let row = &bytes[start..start + PSTATE_ROW_LEN];
            let key = read_u32(&row[0..4]);
            let entry_count = read_u16(&row[4..6]);
            if row[6..8].iter().any(|&b| b != 0) || row[12..16].iter().any(|&b| b != 0) {
                return Err((FormatError::PstateNonZeroReserved).into());
            }
            if entry_count == 0 || entry_count > max_entries {
                return Err((FormatError::PstateInvalidRow).into());
            }
            let entry_start = read_u32(&row[8..12]) as usize;
            if entry_start != expected_entry_start {
                return Err((FormatError::PstateBounds).into());
            }
            let entry_bytes = (entry_count as usize)
                .checked_mul(PSTATE_ENTRY_LEN)
                .ok_or(FormatError::PstateBounds)?;
            let entry_end = entry_start
                .checked_add(entry_bytes)
                .ok_or(FormatError::PstateBounds)?;
            if entry_end > bytes.len() {
                return Err((FormatError::PstateBounds).into());
            }
            expected_entry_start = entry_end;
            if previous_key.is_some_and(|last| last >= key) {
                return Err((FormatError::PstateRowsNotSorted).into());
            }
            previous_key = Some(key);
            let entries = &bytes[entry_start..entry_end];
            let mut previous_token: Option<u32> = None;
            for chunk in entries.chunks_exact(PSTATE_ENTRY_LEN) {
                let token = read_u32(&chunk[..4]);
                if previous_token.is_some_and(|last| last >= token) {
                    return Err((FormatError::PstateEntriesNotSorted).into());
                }
                previous_token = Some(token);
            }
        }

        if expected_entry_start != bytes.len() {
            return Err((FormatError::PstateBounds).into());
        }

        Ok(Self {
            bytes,
            row_count,
            lane_kind,
            decay_shift,
            max_entries,
            key_quant_id,
            ring_capacity,
            base_w,
            boost,
        })
    }

    pub fn row_count(&self) -> u32 {
        self.row_count
    }
    pub fn lane_kind(&self) -> u8 {
        self.lane_kind
    }
    pub fn decay_shift(&self) -> u8 {
        self.decay_shift
    }
    pub fn max_entries(&self) -> u16 {
        self.max_entries
    }
    pub fn key_quant_id(&self) -> u32 {
        self.key_quant_id
    }
    /// Fixed ring capacity for the lane's caller-owned bounded state.
    pub fn ring_capacity(&self) -> u32 {
        self.ring_capacity
    }
    /// `ScoreQ` folded (saturating) per prompt-token occurrence.
    pub fn base_w(&self) -> ScoreQ {
        ScoreQ::from_raw(self.base_w)
    }
    /// `ScoreQ` contribution added (saturating) per candidate support.
    pub fn boost(&self) -> ScoreQ {
        ScoreQ::from_raw(self.boost)
    }

    pub fn rows(&self) -> PstateRows<'a> {
        PstateRows {
            table: *self,
            next: 0,
        }
    }

    /// Deterministic binary-search lookup of a quantized content key.
    pub fn find(&self, key: u32) -> Option<PstateRow<'a>> {
        let mut low = 0usize;
        let mut high = self.row_count as usize;
        while low < high {
            let mid = low + (high - low) / 2;
            let row = self.row(mid)?;
            if row.key() < key {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        let row = self.row(low)?;
        (row.key() == key).then_some(row)
    }

    fn row(&self, index: usize) -> Option<PstateRow<'a>> {
        if index >= self.row_count as usize {
            return None;
        }
        let start = PSTATE_HEADER_LEN + index * PSTATE_ROW_LEN;
        let bytes = self.bytes.get(start..start + PSTATE_ROW_LEN)?;
        let entry_count = read_u16(&bytes[4..6]) as usize;
        let entry_start = read_u32(&bytes[8..12]) as usize;
        let entry_end = entry_start.checked_add(entry_count.checked_mul(PSTATE_ENTRY_LEN)?)?;
        Some(PstateRow {
            bytes,
            entries: self.bytes.get(entry_start..entry_end)?,
        })
    }
}

impl<'a> Iterator for PstateRows<'a> {
    type Item = PstateRow<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let row = self.table.row(self.next)?;
        self.next += 1;
        Some(row)
    }
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from(bytes[0])
        | (u32::from(bytes[1]) << 8)
        | (u32::from(bytes[2]) << 16)
        | (u32::from(bytes[3]) << 24)
}

fn read_u16(bytes: &[u8]) -> u16 {
    u16::from(bytes[0]) | (u16::from(bytes[1]) << 8)
}

/// The segment-lane descriptor a compiler packs into a `PSTATE` section.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentLaneDescriptor {
    /// Fixed ring capacity (bounded caller-owned state). Must be >= 1.
    pub ring_capacity: u32,
    /// Halving decay shift applied per transition. Must be < 32.
    pub decay_shift: u8,
    /// `ScoreQ` raw folded per prompt-token occurrence.
    pub base_w: i32,
    /// `ScoreQ` raw contribution per candidate support.
    pub boost: i32,
    /// Key-quantization table identity (0 = raw token id).
    pub key_quant_id: u32,
}

/// Build a canonical segment-lane `PSTATE` section (compiler side, `alloc`).
///
/// `rows` is the optional residual table as `(key, entries)`; entries are
/// `(token, raw ScoreQ)`. For the pure segment lane `rows` is empty. Keys and
/// tokens are sorted here; duplicate keys or tokens, an empty entry list, a
/// zero `ring_capacity`, or a `decay_shift >= 32` are rejected with a typed
/// error so a producer cannot emit a non-canonical or invalid section. The
/// bytes it returns parse back byte-for-byte (round-trip).
#[cfg(feature = "alloc")]
pub fn build_segment_lane(
    descriptor: &SegmentLaneDescriptor,
    rows: &[(u32, alloc::vec::Vec<(u32, i32)>)],
) -> Result<alloc::vec::Vec<u8>, crate::NotAProduct> {
    use alloc::vec::Vec;

    if descriptor.ring_capacity == 0 || descriptor.decay_shift >= 32 {
        return Err((FormatError::PstateInvalidRow).into());
    }

    let mut sorted: Vec<(u32, Vec<(u32, i32)>)> = rows.to_vec();
    sorted.sort_by_key(|(k, _)| *k);
    let mut max_entries: usize = 0;
    for i in 0..sorted.len() {
        if i > 0 && sorted[i].0 == sorted[i - 1].0 {
            return Err((FormatError::PstateRowsNotSorted).into());
        }
        let entries = &mut sorted[i].1;
        if entries.is_empty() {
            return Err((FormatError::PstateInvalidRow).into());
        }
        entries.sort_by_key(|(t, _)| *t);
        for j in 1..entries.len() {
            if entries[j].0 == entries[j - 1].0 {
                return Err((FormatError::PstateEntriesNotSorted).into());
            }
        }
        max_entries = max_entries.max(entries.len());
    }
    if max_entries > u16::MAX as usize || sorted.len() > u32::MAX as usize {
        return Err((FormatError::PstateBounds).into());
    }

    let row_count = sorted.len() as u32;
    let mut out = Vec::new();
    out.extend_from_slice(&PSTATE_MAGIC);
    out.extend_from_slice(&PSTATE_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // reserved [6..8]
    out.extend_from_slice(&row_count.to_le_bytes());
    out.extend_from_slice(&(max_entries as u16).to_le_bytes());
    out.push(descriptor.decay_shift);
    out.push(LANE_SEGMENT);
    out.extend_from_slice(&PSTATE_VERSION.to_le_bytes()); // schema_version
    out.extend_from_slice(&0u16.to_le_bytes()); // reserved [18..20]
    out.extend_from_slice(&descriptor.key_quant_id.to_le_bytes());
    out.extend_from_slice(&descriptor.ring_capacity.to_le_bytes());
    out.extend_from_slice(&descriptor.base_w.to_le_bytes());
    out.extend_from_slice(&descriptor.boost.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // reserved [36..40]
    debug_assert_eq!(out.len(), PSTATE_HEADER_LEN);

    // rows, with contiguous entry offsets
    let mut entry_cursor = PSTATE_HEADER_LEN + sorted.len() * PSTATE_ROW_LEN;
    for (key, entries) in &sorted {
        out.extend_from_slice(&key.to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // reserved
        out.extend_from_slice(&(entry_cursor as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes()); // reserved
        entry_cursor += entries.len() * PSTATE_ENTRY_LEN;
    }
    for (_, entries) in &sorted {
        for (token, score) in entries {
            out.extend_from_slice(&token.to_le_bytes());
            out.extend_from_slice(&score.to_le_bytes());
        }
    }
    Ok(out)
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use alloc::vec;

    fn descriptor() -> SegmentLaneDescriptor {
        SegmentLaneDescriptor {
            ring_capacity: 64,
            decay_shift: 1,
            base_w: 1 << 12,
            boost: 1 << 20,
            key_quant_id: 0,
        }
    }

    fn sample() -> alloc::vec::Vec<u8> {
        build_segment_lane(
            &descriptor(),
            &[
                (10, vec![(3, 100), (7, -50)]),
                (5, vec![(1, 200)]),
                (20, vec![(2, 10), (4, 20), (9, 30)]),
            ],
        )
        .expect("valid rows")
    }

    #[test]
    fn round_trip_descriptor_and_lookup() {
        let bytes = sample();
        let table = PstateTable::parse(&bytes).expect("parse");
        assert_eq!(table.row_count(), 3);
        assert_eq!(table.lane_kind(), LANE_SEGMENT);
        assert_eq!(table.decay_shift(), 1);
        assert_eq!(table.ring_capacity(), 64);
        assert_eq!(table.base_w().raw(), 1 << 12);
        assert_eq!(table.boost().raw(), 1 << 20);
        assert_eq!(table.max_entries(), 3);
        // sorted by key: 5, 10, 20
        let keys: alloc::vec::Vec<u32> = table.rows().map(|r| r.key()).collect();
        assert_eq!(keys, vec![5, 10, 20]);
        // lookup + entries canonical by token
        let row = table.find(10).expect("key 10");
        let entries: alloc::vec::Vec<(u32, i32)> =
            row.entries().map(|e| (e.token, e.score_q.raw())).collect();
        assert_eq!(entries, vec![(3, 100), (7, -50)]);
        assert!(table.find(11).is_none());
        // re-serialize is byte-identical (determinism, order-independent)
        let again = build_segment_lane(
            &descriptor(),
            &[
                (20, vec![(9, 30), (2, 10), (4, 20)]),
                (10, vec![(7, -50), (3, 100)]),
                (5, vec![(1, 200)]),
            ],
        )
        .unwrap();
        assert_eq!(again, bytes, "canonicalization is order-independent");
    }

    #[test]
    fn rejects_corruption() {
        let good = sample();
        // bad magic
        let mut b = good.clone();
        b[0] = b'X';
        assert!(PstateTable::parse(&b).is_err());
        // unsupported version
        let mut b = good.clone();
        b[4] = 9;
        assert!(PstateTable::parse(&b).is_err());
        // non-zero reserved (tail)
        let mut b = good.clone();
        b[36] = 1;
        assert!(PstateTable::parse(&b).is_err());
        // truncated (drops last entry) → bounds / coverage
        let mut b = good.clone();
        b.truncate(b.len() - 1);
        assert!(PstateTable::parse(&b).is_err());
        // corrupt an entry offset → bounds
        let mut b = sample();
        let off = PSTATE_HEADER_LEN + 8; // first row entry_start
        b[off] = 0xFF;
        assert!(PstateTable::parse(&b).is_err());
    }

    #[test]
    fn rejects_invalid_descriptor() {
        // zero capacity
        assert!(build_segment_lane(
            &SegmentLaneDescriptor {
                ring_capacity: 0,
                ..descriptor()
            },
            &[]
        )
        .is_err());
        // decay too wide
        assert!(build_segment_lane(
            &SegmentLaneDescriptor {
                decay_shift: 32,
                ..descriptor()
            },
            &[]
        )
        .is_err());
        // a hand-built section with zero capacity fails parse (semantic teeth)
        let mut b = build_segment_lane(&descriptor(), &[]).unwrap();
        b[24..28].copy_from_slice(&0u32.to_le_bytes());
        assert!(PstateTable::parse(&b).is_err());
    }

    #[test]
    fn builder_rejects_noncanonical() {
        assert!(build_segment_lane(&descriptor(), &[(1, vec![])]).is_err()); // empty entries
        assert!(
            build_segment_lane(&descriptor(), &[(1, vec![(2, 1)]), (1, vec![(3, 1)])]).is_err()
        ); // dup key
        assert!(build_segment_lane(&descriptor(), &[(1, vec![(2, 1), (2, 5)])]).is_err());
        // dup token
    }

    #[test]
    fn header_only_is_valid_empty() {
        let bytes = build_segment_lane(&descriptor(), &[]).unwrap();
        let table = PstateTable::parse(&bytes).expect("empty parses");
        assert_eq!(table.row_count(), 0);
        assert_eq!(table.ring_capacity(), 64);
        assert_eq!(bytes.len(), PSTATE_HEADER_LEN);
    }
}
