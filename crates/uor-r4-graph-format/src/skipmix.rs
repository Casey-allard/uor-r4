//! Borrowed packed skip-conditioned residual scoring tables — the optional
//! `SKMX` (primary joint table) and `PSIB` (unconditioned fallback) sections
//! (#897, lowering the D1-selected/#897-phase-0-confirmed 1-token skip-mix
//! scorer, `SELECT-1-token`, S1 redesign #822).
//!
//! Two bounded, fixed-offset tables, read with P-4-legal operations only
//! (`docs/prompt_state_spec_835.md` §5: XOR/AND/OR/NOT, shift/rotate,
//! popcount/cttz/ctlz, saturating/wrapping add-sub, integer compare,
//! fixed-offset table reads — no multiply/divide/float):
//!
//! - [`SkipmixTable`] (`SKMX`) — the primary joint table, keyed by the
//!   composite `(content_token, last_window_token)` pair, found by a
//!   fixed-capacity open-addressed hash lookup. The hash function
//!   ([`hash_key`]) is a multiply-free ARX (add/rotate/xor) mixer in the
//!   Jenkins "one-at-a-time" family, with no seed or randomization, so
//!   identical inputs hash identically on every platform (the determinism
//!   invariant a keyed hash such as `std::HashMap`'s SipHash cannot give).
//!   Lookup cost is a provable, checked `max_probe` bound recorded in the
//!   header — never a function of how many keys the table holds.
//! - [`PsiBagTable`] (`PSIB`) — the unconditioned Ψ-bag fallback, keyed by
//!   `content_token` alone. Small key space, so it stays a flat sorted
//!   array found by binary search, exactly like the #836 `PSTATE` precedent
//!   (`crate::pstate::PstateTable`) — no reason to hash-table a table this
//!   small.
//!
//! Both sections are **optional**
//! ([`crate::types::SectionId::OPTIONAL_BIT`]): an artifact without them, or
//! a reader that does not consume them, behaves exactly as before
//! (absent-section identity).

use crate::error::FormatError;
use crate::types::ScoreQ;

// ---------------------------------------------------------------------
// Shared entry shape (8 bytes: candidate token + signed ScoreQ residual)
// ---------------------------------------------------------------------

/// One candidate-support contribution: a candidate token and its signed
/// `ScoreQ` residual (added by saturating integer addition on the hot path).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkipEntry {
    pub token: u32,
    pub score_q: ScoreQ,
}

const ENTRY_LEN: usize = 8;

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from(bytes[0])
        | (u32::from(bytes[1]) << 8)
        | (u32::from(bytes[2]) << 16)
        | (u32::from(bytes[3]) << 24)
}

fn read_u16(bytes: &[u8]) -> u16 {
    u16::from(bytes[0]) | (u16::from(bytes[1]) << 8)
}

fn read_entry(bytes: &[u8]) -> Option<SkipEntry> {
    let chunk = bytes.get(..ENTRY_LEN)?;
    Some(SkipEntry {
        token: read_u32(&chunk[..4]),
        score_q: ScoreQ::from_raw(i32::from_le_bytes(chunk[4..8].try_into().ok()?)),
    })
}

/// Borrowed, canonical (candidate-token-ascending) entry list shared by both
/// table kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkipEntries<'a> {
    bytes: &'a [u8],
}

impl<'a> SkipEntries<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn len(&self) -> usize {
        self.bytes.len() >> 3
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn iter(&self) -> SkipEntriesIter<'a> {
        SkipEntriesIter { bytes: self.bytes }
    }

    /// Deterministic binary-search lookup of a candidate token within this
    /// row's canonical (ascending) entries.
    pub fn find(&self, token: u32) -> Option<ScoreQ> {
        let count = self.len();
        let mut low = 0usize;
        let mut high = count;
        while low < high {
            let mid = low + ((high - low) >> 1);
            let start = mid << 3;
            let entry = read_entry(&self.bytes[start..])?;
            if entry.token < token {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        let start = low << 3;
        let entry = read_entry(self.bytes.get(start..)?)?;
        (entry.token == token).then_some(entry.score_q)
    }
}

pub struct SkipEntriesIter<'a> {
    bytes: &'a [u8],
}

impl Iterator for SkipEntriesIter<'_> {
    type Item = SkipEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = read_entry(self.bytes)?;
        self.bytes = &self.bytes[ENTRY_LEN..];
        Some(entry)
    }
}

// ---------------------------------------------------------------------
// PSIB — unconditioned Psi-bag fallback table (flat u32 key, binary search;
// same shape/discipline as crate::pstate::PstateTable, minus the segment-lane
// descriptor fields, which do not apply to this table kind).
// ---------------------------------------------------------------------

pub const PSIB_MAGIC: [u8; 4] = *b"PSIB";
pub const PSIB_VERSION: u16 = 1;
pub const PSIB_HEADER_LEN: usize = 16;
pub const PSIB_ROW_LEN: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PsiBagRow<'a> {
    key: u32,
    entries: &'a [u8],
}

impl<'a> PsiBagRow<'a> {
    /// The unconditioned content token this row's evidence is folded over.
    pub fn key(&self) -> u32 {
        self.key
    }

    pub fn entries(&self) -> SkipEntries<'a> {
        SkipEntries::new(self.entries)
    }
}

/// A borrowed, validated `PSIB` fallback table over an artifact's section
/// bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PsiBagTable<'a> {
    bytes: &'a [u8],
    row_count: u32,
    max_entries: u16,
}

impl<'a> PsiBagTable<'a> {
    /// Two-stage validation (header, then per-row structure): magic,
    /// version, reserved-zero, bounded entry counts, contiguous canonical
    /// layout, sorted keys and tokens, exact byte coverage. Never allocates;
    /// never panics on a recoverable input.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, crate::NotAProduct> {
        if bytes.len() < PSIB_HEADER_LEN {
            return Err((FormatError::PsiBagTooShort).into());
        }
        if bytes[..4] != PSIB_MAGIC {
            return Err((FormatError::PsiBagBadMagic).into());
        }
        if read_u16(&bytes[4..6]) != PSIB_VERSION {
            return Err((FormatError::PsiBagUnsupportedVersion).into());
        }
        if bytes[6..8].iter().any(|&b| b != 0) || bytes[14..16].iter().any(|&b| b != 0) {
            return Err((FormatError::PsiBagNonZeroReserved).into());
        }
        let row_count = read_u32(&bytes[8..12]);
        let max_entries = read_u16(&bytes[12..14]);

        let rows_len = (row_count as usize)
            .checked_mul(PSIB_ROW_LEN)
            .ok_or(FormatError::PsiBagBounds)?;
        let entries_start = PSIB_HEADER_LEN
            .checked_add(rows_len)
            .ok_or(FormatError::PsiBagBounds)?;
        if entries_start > bytes.len() {
            return Err((FormatError::PsiBagBounds).into());
        }

        let mut previous_key: Option<u32> = None;
        let mut expected_entry_start = entries_start;
        for index in 0..row_count as usize {
            let start = PSIB_HEADER_LEN + index * PSIB_ROW_LEN;
            let row = &bytes[start..start + PSIB_ROW_LEN];
            let key = read_u32(&row[0..4]);
            let entry_count = read_u16(&row[4..6]);
            if row[6..8].iter().any(|&b| b != 0) {
                return Err((FormatError::PsiBagNonZeroReserved).into());
            }
            if entry_count == 0 || entry_count > max_entries {
                return Err((FormatError::PsiBagInvalidRow).into());
            }
            let entry_start = read_u32(&row[8..12]) as usize;
            if entry_start != expected_entry_start {
                return Err((FormatError::PsiBagBounds).into());
            }
            let entry_bytes = (entry_count as usize)
                .checked_mul(ENTRY_LEN)
                .ok_or(FormatError::PsiBagBounds)?;
            let entry_end = entry_start
                .checked_add(entry_bytes)
                .ok_or(FormatError::PsiBagBounds)?;
            if entry_end > bytes.len() {
                return Err((FormatError::PsiBagBounds).into());
            }
            expected_entry_start = entry_end;
            if previous_key.is_some_and(|last| last >= key) {
                return Err((FormatError::PsiBagRowsNotSorted).into());
            }
            previous_key = Some(key);
            let mut previous_token: Option<u32> = None;
            for chunk in bytes[entry_start..entry_end].chunks_exact(ENTRY_LEN) {
                let token = read_u32(&chunk[..4]);
                if previous_token.is_some_and(|last| last >= token) {
                    return Err((FormatError::PsiBagEntriesNotSorted).into());
                }
                previous_token = Some(token);
            }
        }
        if expected_entry_start != bytes.len() {
            return Err((FormatError::PsiBagBounds).into());
        }

        Ok(Self {
            bytes,
            row_count,
            max_entries,
        })
    }

    /// Cheap reconstruction from bytes that have **already** been fully
    /// validated once via [`PsiBagTable::parse`] -- intended for a deployed
    /// engine that owns a validated `Vec<u8>` (validated once, at artifact
    /// load) and wants a fresh borrowed view at each inference step without
    /// re-running `parse`'s O(row_count) structural scan on every lookup.
    /// Repeats only the O(1) header checks (magic, version, length) needed
    /// for [`Self::find`]'s arithmetic to stay bounds-safe.
    ///
    /// Every lookup still goes through `.get()`-guarded, checked-arithmetic
    /// accessors, so this can never panic -- but unlike `parse`, it does
    /// *not* re-verify sortedness, exact byte coverage, or row structure.
    /// Callers MUST only pass bytes that previously passed `parse`
    /// successfully; on bytes that never passed `parse`, this method may
    /// return a table whose lookups are wrong or spuriously absent instead
    /// of surfacing a typed error. It is not a substitute for `parse` on
    /// untrusted input -- only a cheap re-view of already-trusted bytes.
    pub fn from_trusted_bytes(bytes: &'a [u8]) -> Option<Self> {
        if bytes.len() < PSIB_HEADER_LEN {
            return None;
        }
        if bytes[..4] != PSIB_MAGIC {
            return None;
        }
        if read_u16(&bytes[4..6]) != PSIB_VERSION {
            return None;
        }
        let row_count = read_u32(&bytes[8..12]);
        let max_entries = read_u16(&bytes[12..14]);
        Some(Self {
            bytes,
            row_count,
            max_entries,
        })
    }

    pub fn row_count(&self) -> u32 {
        self.row_count
    }

    pub fn max_entries(&self) -> u16 {
        self.max_entries
    }

    /// The exact section bytes this table was parsed (or cheaply
    /// reconstructed) from -- e.g. so a caller can retain just the bytes as
    /// owned storage and later reconstruct a borrowed view cheaply via
    /// [`Self::from_trusted_bytes`], without holding this borrowed `Self`
    /// itself (and its lifetime) any longer than the current use.
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    fn row(&self, index: usize) -> Option<PsiBagRow<'a>> {
        if index >= self.row_count as usize {
            return None;
        }
        let row_offset = (index << 3).saturating_add(index << 2);
        let start = PSIB_HEADER_LEN.saturating_add(row_offset);
        let end = start.checked_add(PSIB_ROW_LEN)?;
        let bytes = self.bytes.get(start..end)?;
        let key = read_u32(&bytes[0..4]);
        let entry_count = read_u16(&bytes[4..6]) as usize;
        let entry_start = read_u32(&bytes[8..12]) as usize;
        let entry_end = entry_start.checked_add(entry_count.checked_shl(3)?)?;
        Some(PsiBagRow {
            key,
            entries: self.bytes.get(entry_start..entry_end)?,
        })
    }

    /// Deterministic binary-search lookup of an unconditioned content token.
    pub fn find(&self, key: u32) -> Option<PsiBagRow<'a>> {
        let mut low = 0usize;
        let mut high = self.row_count as usize;
        while low < high {
            let mid = low + ((high - low) >> 1);
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
}

// ---------------------------------------------------------------------
// SKMX — primary joint table: fixed-capacity open-addressed hash table,
// keyed by the composite (content_token, last_window_token) pair.
// ---------------------------------------------------------------------

pub const SKMX_MAGIC: [u8; 4] = *b"SKM1";
pub const SKMX_VERSION: u16 = 1;
pub const SKMX_HEADER_LEN: usize = 20;
pub const SKMX_SLOT_LEN: usize = 16;

/// Multiply-free ARX (add/rotate/xor) mixer, Jenkins "one-at-a-time" family.
/// Every step is P-4-legal (wrapping add, shift, xor) -- no multiply, no
/// seed/randomization, so identical inputs hash identically on every
/// platform (required for artifact-byte determinism; a keyed hash such as
/// `std::HashMap`'s SipHash cannot give this).
#[inline]
fn mix(mut h: u32, x: u32) -> u32 {
    h = h.wrapping_add(x);
    h = h.wrapping_add(h << 10);
    h ^= h >> 6;
    h
}

/// Deterministic, multiply-free hash of the `(content_token, last_token)`
/// composite key. Used both by the compiler-side builder (to place a row)
/// and the deployed reader (to find it) -- the two must never diverge.
pub fn hash_key(content_token: u32, last_token: u32) -> u32 {
    let mut h: u32 = 0;
    h = mix(h, content_token);
    h = mix(h, last_token);
    h = h.wrapping_add(h << 3);
    h ^= h >> 11;
    h = h.wrapping_add(h << 15);
    h
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkipmixRow<'a> {
    content_token: u32,
    last_token: u32,
    entries: &'a [u8],
}

impl<'a> SkipmixRow<'a> {
    pub fn content_token(&self) -> u32 {
        self.content_token
    }
    pub fn last_token(&self) -> u32 {
        self.last_token
    }
    pub fn entries(&self) -> SkipEntries<'a> {
        SkipEntries::new(self.entries)
    }
}

/// A borrowed, validated `SKMX` joint table over an artifact's section
/// bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkipmixTable<'a> {
    bytes: &'a [u8],
    capacity: u32,
    max_probe: u16,
    max_entries: u16,
}

impl<'a> SkipmixTable<'a> {
    /// Two-stage validation. Stage one: header well-formedness (magic,
    /// version, reserved-zero, `capacity` a positive power of two). Stage
    /// two: a full structural scan of every slot -- bounded entry counts,
    /// contiguous canonical entry layout, entries canonical (ascending) by
    /// candidate token, exact byte coverage, AND the open-addressing
    /// placement invariant: every occupied slot's key must hash to a home
    /// bucket reachable by an unbroken run of occupied slots within
    /// `max_probe` steps (otherwise a deployed lookup that stops at the
    /// first empty slot could wrongly report an present key as absent).
    /// Never allocates; never panics on a recoverable input.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, crate::NotAProduct> {
        if bytes.len() < SKMX_HEADER_LEN {
            return Err((FormatError::SkipmixTooShort).into());
        }
        if bytes[..4] != SKMX_MAGIC {
            return Err((FormatError::SkipmixBadMagic).into());
        }
        if read_u16(&bytes[4..6]) != SKMX_VERSION {
            return Err((FormatError::SkipmixUnsupportedVersion).into());
        }
        if bytes[6..8].iter().any(|&b| b != 0) || bytes[16..20].iter().any(|&b| b != 0) {
            return Err((FormatError::SkipmixNonZeroReserved).into());
        }
        let capacity = read_u32(&bytes[8..12]);
        let max_probe = read_u16(&bytes[12..14]);
        let max_entries = read_u16(&bytes[14..16]);

        if capacity == 0 || capacity.count_ones() != 1 {
            return Err((FormatError::SkipmixInvalidRow).into());
        }
        // An empty table (no occupied slots) is valid with max_probe == 0;
        // a non-trivial max_probe on an all-empty table is not canonical.

        let slots_len = (capacity as usize)
            .checked_mul(SKMX_SLOT_LEN)
            .ok_or(FormatError::SkipmixBounds)?;
        let entries_start = SKMX_HEADER_LEN
            .checked_add(slots_len)
            .ok_or(FormatError::SkipmixBounds)?;
        if entries_start > bytes.len() {
            return Err((FormatError::SkipmixBounds).into());
        }

        let slot_at = |index: u32| -> Option<(u64, u16, u32)> {
            let start = SKMX_HEADER_LEN + (index as usize) * SKMX_SLOT_LEN;
            let s = bytes.get(start..start + SKMX_SLOT_LEN)?;
            let key = u64::from_le_bytes(s[0..8].try_into().ok()?);
            let entry_count = read_u16(&s[8..10]);
            if s[10..12].iter().any(|&b| b != 0) {
                return None;
            }
            let entry_start = read_u32(&s[12..16]);
            Some((key, entry_count, entry_start))
        };

        let mask = capacity - 1;
        let mut expected_entry_start = entries_start;
        let mut any_occupied = false;
        for index in 0..capacity {
            let (key, entry_count, entry_start) =
                slot_at(index).ok_or(FormatError::SkipmixBounds)?;
            if entry_count == 0 {
                if key != 0 || entry_start != 0 {
                    return Err((FormatError::SkipmixInvalidRow).into());
                }
                continue;
            }
            any_occupied = true;
            if entry_count > max_entries {
                return Err((FormatError::SkipmixInvalidRow).into());
            }
            let entry_start = entry_start as usize;
            if entry_start != expected_entry_start {
                return Err((FormatError::SkipmixBounds).into());
            }
            let entry_bytes = (entry_count as usize)
                .checked_mul(ENTRY_LEN)
                .ok_or(FormatError::SkipmixBounds)?;
            let entry_end = entry_start
                .checked_add(entry_bytes)
                .ok_or(FormatError::SkipmixBounds)?;
            if entry_end > bytes.len() {
                return Err((FormatError::SkipmixBounds).into());
            }
            expected_entry_start = entry_end;

            let mut previous_token: Option<u32> = None;
            for chunk in bytes[entry_start..entry_end].chunks_exact(ENTRY_LEN) {
                let token = read_u32(&chunk[..4]);
                if previous_token.is_some_and(|last| last >= token) {
                    return Err((FormatError::SkipmixEntriesNotSorted).into());
                }
                previous_token = Some(token);
            }

            // Placement invariant: walk forward from this key's home bucket;
            // every slot before `index` on that path must be occupied, and
            // `index` itself must be within `max_probe` steps of home.
            let content_token = (key >> 32) as u32;
            let last_token = key as u32;
            let home = hash_key(content_token, last_token) & mask;
            let mut probe = 0u32;
            let mut cursor = home;
            while cursor != index {
                let (_, occ_count, _) = slot_at(cursor).ok_or(FormatError::SkipmixBounds)?;
                if occ_count == 0 {
                    return Err((FormatError::SkipmixProbeGap).into());
                }
                probe += 1;
                if probe as u16 > max_probe {
                    return Err((FormatError::SkipmixProbeExceeded).into());
                }
                cursor = (cursor + 1) & mask;
            }
            if probe as u16 > max_probe {
                return Err((FormatError::SkipmixProbeExceeded).into());
            }
        }
        if !any_occupied && max_probe != 0 {
            return Err((FormatError::SkipmixInvalidRow).into());
        }
        if expected_entry_start != bytes.len() {
            return Err((FormatError::SkipmixBounds).into());
        }

        Ok(Self {
            bytes,
            capacity,
            max_probe,
            max_entries,
        })
    }

    /// Cheap reconstruction from bytes that have **already** been fully
    /// validated once via [`SkipmixTable::parse`]. See
    /// [`PsiBagTable::from_trusted_bytes`] for the exact contract: this
    /// repeats only the O(1) header checks (magic, version, length,
    /// `capacity` a positive power of two) needed for [`Self::find`]'s
    /// arithmetic to stay bounds-safe -- never `parse`'s O(capacity)
    /// structural scan (per-slot entry validation and the open-addressing
    /// placement invariant). `find` still reads only through
    /// `.get()`-guarded, checked-arithmetic accessors, so this can never
    /// panic, but callers MUST only pass bytes that previously passed
    /// `parse` successfully -- on bytes that never passed `parse`, lookups
    /// may be wrong or spuriously absent instead of a typed error. This
    /// exists so a deployed engine can hold one validated `Vec<u8>` (from
    /// artifact load) and take a fresh borrowed view at every inference
    /// step without re-running the full structural scan each time.
    pub fn from_trusted_bytes(bytes: &'a [u8]) -> Option<Self> {
        if bytes.len() < SKMX_HEADER_LEN {
            return None;
        }
        if bytes[..4] != SKMX_MAGIC {
            return None;
        }
        if read_u16(&bytes[4..6]) != SKMX_VERSION {
            return None;
        }
        let capacity = read_u32(&bytes[8..12]);
        if capacity == 0 || capacity.count_ones() != 1 {
            return None;
        }
        let max_probe = read_u16(&bytes[12..14]);
        let max_entries = read_u16(&bytes[14..16]);
        Some(Self {
            bytes,
            capacity,
            max_probe,
            max_entries,
        })
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }
    pub fn max_probe(&self) -> u16 {
        self.max_probe
    }
    pub fn max_entries(&self) -> u16 {
        self.max_entries
    }

    /// The exact section bytes this table was parsed (or cheaply
    /// reconstructed) from -- see [`PsiBagTable::bytes`] for why a caller
    /// wants this (retaining owned bytes and re-borrowing cheaply via
    /// [`Self::from_trusted_bytes`] rather than holding this borrowed `Self`).
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    fn slot_row(&self, index: u32) -> Option<SkipmixRow<'a>> {
        let start = SKMX_HEADER_LEN.saturating_add((index as usize) << 4);
        let end = start.checked_add(SKMX_SLOT_LEN)?;
        let s = self.bytes.get(start..end)?;
        let key = u64::from_le_bytes(s[0..8].try_into().ok()?);
        let entry_count = read_u16(&s[8..10]) as usize;
        let entry_start = read_u32(&s[12..16]) as usize;
        let entry_end = entry_start.checked_add(entry_count.checked_shl(3)?)?;
        Some(SkipmixRow {
            content_token: (key >> 32) as u32,
            last_token: key as u32,
            entries: self.bytes.get(entry_start..entry_end)?,
        })
    }

    /// Deployed, P-4-legal lookup: `home = hash_key(...) & (capacity - 1)`
    /// (bitwise AND, not modulo -- `capacity` is a power of two), then a
    /// forward linear probe of at most `max_probe` slots. Reads only:
    /// fixed-offset table reads, integer add/and/compare.
    pub fn find(&self, content_token: u32, last_token: u32) -> Option<SkipmixRow<'a>> {
        let mask = self.capacity - 1;
        let home = hash_key(content_token, last_token) & mask;
        let mut cursor = home;
        for _ in 0..=self.max_probe {
            let start = SKMX_HEADER_LEN.saturating_add((cursor as usize) << 4);
            let end = start.checked_add(SKMX_SLOT_LEN)?;
            let s = self.bytes.get(start..end)?;
            let entry_count = read_u16(&s[8..10]);
            if entry_count == 0 {
                return None;
            }
            let key = u64::from_le_bytes(s[0..8].try_into().ok()?);
            if (key >> 32) as u32 == content_token && key as u32 == last_token {
                return self.slot_row(cursor);
            }
            cursor = (cursor + 1) & mask;
        }
        None
    }
}

// ---------------------------------------------------------------------
// Compiler-side builders (alloc-gated). The deployed reader above never
// allocates; only these construction paths do.
// ---------------------------------------------------------------------

/// Canonicalize and serialize the `PSIB` unconditioned fallback table.
/// `rows` is `(content_token, entries)`; entries are `(candidate_token, raw
/// ScoreQ)`. Keys and tokens are sorted here; a duplicate key, a duplicate
/// token within a key, or an empty entry list is rejected with a typed
/// error so a producer cannot emit a non-canonical section. Bytes returned
/// parse back byte-for-byte (round-trip), independent of input order.
#[cfg(feature = "alloc")]
pub fn build_psi_bag_table(
    rows: &[(u32, alloc::vec::Vec<(u32, i32)>)],
) -> Result<alloc::vec::Vec<u8>, crate::NotAProduct> {
    use alloc::vec::Vec;

    let mut sorted: Vec<(u32, Vec<(u32, i32)>)> = rows.to_vec();
    sorted.sort_by_key(|(k, _)| *k);
    let mut max_entries: usize = 0;
    for i in 0..sorted.len() {
        if i > 0 && sorted[i].0 == sorted[i - 1].0 {
            return Err((FormatError::PsiBagRowsNotSorted).into());
        }
        let entries = &mut sorted[i].1;
        if entries.is_empty() {
            return Err((FormatError::PsiBagInvalidRow).into());
        }
        entries.sort_by_key(|(t, _)| *t);
        for j in 1..entries.len() {
            if entries[j].0 == entries[j - 1].0 {
                return Err((FormatError::PsiBagEntriesNotSorted).into());
            }
        }
        max_entries = max_entries.max(entries.len());
    }
    if max_entries > u16::MAX as usize || sorted.len() > u32::MAX as usize {
        return Err((FormatError::PsiBagBounds).into());
    }

    let row_count = sorted.len() as u32;
    let mut out = Vec::new();
    out.extend_from_slice(&PSIB_MAGIC);
    out.extend_from_slice(&PSIB_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&row_count.to_le_bytes());
    out.extend_from_slice(&(max_entries as u16).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    debug_assert_eq!(out.len(), PSIB_HEADER_LEN);

    let mut entry_cursor = PSIB_HEADER_LEN + sorted.len() * PSIB_ROW_LEN;
    for (key, entries) in &sorted {
        out.extend_from_slice(&key.to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&(entry_cursor as u32).to_le_bytes());
        entry_cursor += entries.len() * ENTRY_LEN;
    }
    for (_, entries) in &sorted {
        for (token, score) in entries {
            out.extend_from_slice(&token.to_le_bytes());
            out.extend_from_slice(&score.to_le_bytes());
        }
    }
    Ok(out)
}

/// Upper bound on the acceptable linear-probe distance during placement;
/// exceeding it triggers a capacity doubling and a full re-placement, never
/// a partial/best-effort table. Kept well under `u16::MAX` (the header
/// field width) with wide margin.
#[cfg(feature = "alloc")]
const MAX_ACCEPTABLE_PROBE: u32 = 32;
/// Safety backstop on how many times capacity may double before the builder
/// gives up with a typed error instead of looping forever. At a target load
/// factor <= 50% with the mixer above, real corpora settle within a handful
/// of attempts; this bound exists so a pathological/adversarial input
/// cannot hang the compiler.
#[cfg(feature = "alloc")]
const MAX_GROW_ATTEMPTS: u32 = 24;

/// Builder input row: `(content_token, last_window_token, entries)`, entries
/// as `(candidate_token, raw ScoreQ)`. Aliased to keep the builder's
/// signature and locals readable (and to satisfy `clippy::type_complexity`).
#[cfg(feature = "alloc")]
pub type SkipmixRowInput = (u32, u32, alloc::vec::Vec<(u32, i32)>);

/// Canonicalize and serialize the `SKMX` primary joint table. `rows` is
/// `(content_token, last_window_token, entries)`; entries are
/// `(candidate_token, raw ScoreQ)`. Keys and tokens are sorted here (fixed,
/// deterministic insertion order into the hash table, independent of the
/// caller's `HashMap` iteration order); a duplicate `(content_token,
/// last_token)` key, a duplicate candidate token within a key, or an empty
/// entry list is rejected with a typed error. Capacity is chosen (a power
/// of two, target load factor <= 50%) and grown/retried whenever any key's
/// actual placement probe distance would exceed [`MAX_ACCEPTABLE_PROBE`], so
/// the `max_probe` recorded in the header is always a true, checked bound —
/// never a best-effort estimate. Bytes returned parse back byte-for-byte.
#[cfg(feature = "alloc")]
pub fn build_skipmix_table(
    rows: &[SkipmixRowInput],
) -> Result<alloc::vec::Vec<u8>, crate::NotAProduct> {
    use alloc::vec::Vec;

    let mut sorted: Vec<SkipmixRowInput> = rows.to_vec();
    sorted.sort_by_key(|(content_token, last_token, _)| (*content_token, *last_token));
    let mut max_entries: usize = 0;
    for i in 0..sorted.len() {
        if i > 0 && (sorted[i].0, sorted[i].1) == (sorted[i - 1].0, sorted[i - 1].1) {
            return Err((FormatError::SkipmixDuplicateKey).into());
        }
        let entries = &mut sorted[i].2;
        if entries.is_empty() {
            return Err((FormatError::SkipmixInvalidRow).into());
        }
        entries.sort_by_key(|(t, _)| *t);
        for j in 1..entries.len() {
            if entries[j].0 == entries[j - 1].0 {
                return Err((FormatError::SkipmixEntriesNotSorted).into());
            }
        }
        max_entries = max_entries.max(entries.len());
    }
    if max_entries > u16::MAX as usize || sorted.len() > u32::MAX as usize {
        return Err((FormatError::SkipmixBounds).into());
    }

    let n = sorted.len();
    let mut capacity: u32 = (n.max(1) as u32).saturating_mul(2).next_power_of_two();
    let mut attempt = 0u32;
    let (capacity, max_probe, slots): (u32, u16, Vec<Option<usize>>) = loop {
        let mask = capacity - 1;
        let mut slots: Vec<Option<usize>> = alloc::vec![None; capacity as usize];
        let mut observed_max_probe: u32 = 0;
        let mut placement_ok = true;
        for (row_index, (content_token, last_token, _)) in sorted.iter().enumerate() {
            let home = hash_key(*content_token, *last_token) & mask;
            let mut cursor = home;
            let mut probe = 0u32;
            loop {
                if slots[cursor as usize].is_none() {
                    slots[cursor as usize] = Some(row_index);
                    break;
                }
                probe += 1;
                cursor = (cursor + 1) & mask;
                if probe > capacity {
                    placement_ok = false;
                    break;
                }
            }
            if !placement_ok {
                break;
            }
            observed_max_probe = observed_max_probe.max(probe);
            if observed_max_probe > MAX_ACCEPTABLE_PROBE {
                placement_ok = false;
                break;
            }
        }
        if placement_ok && observed_max_probe <= u16::MAX as u32 {
            break (capacity, observed_max_probe as u16, slots);
        }
        attempt += 1;
        if attempt > MAX_GROW_ATTEMPTS {
            return Err((FormatError::SkipmixBounds).into());
        }
        capacity = capacity.saturating_mul(2);
    };

    let mut out = Vec::new();
    out.extend_from_slice(&SKMX_MAGIC);
    out.extend_from_slice(&SKMX_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&capacity.to_le_bytes());
    out.extend_from_slice(&max_probe.to_le_bytes());
    out.extend_from_slice(&(max_entries as u16).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    debug_assert_eq!(out.len(), SKMX_HEADER_LEN);

    let mut entry_cursor = SKMX_HEADER_LEN + (capacity as usize) * SKMX_SLOT_LEN;
    for slot in &slots {
        match slot {
            None => {
                out.extend_from_slice(&0u64.to_le_bytes());
                out.extend_from_slice(&0u16.to_le_bytes());
                out.extend_from_slice(&0u16.to_le_bytes());
                out.extend_from_slice(&0u32.to_le_bytes());
            }
            Some(row_index) => {
                let (content_token, last_token, entries) = &sorted[*row_index];
                let key = (u64::from(*content_token) << 32) | u64::from(*last_token);
                out.extend_from_slice(&key.to_le_bytes());
                out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
                out.extend_from_slice(&0u16.to_le_bytes());
                out.extend_from_slice(&(entry_cursor as u32).to_le_bytes());
                entry_cursor += entries.len() * ENTRY_LEN;
            }
        }
    }
    for row_index in slots.iter().flatten() {
        let (_, _, entries) = &sorted[*row_index];
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
    use alloc::vec::Vec;

    /// `(content_token, last_window_token, entries)` — the builder's input
    /// row shape, aliased so tests don't repeat the nested-tuple type.
    type SkipmixInputRow = (u32, u32, Vec<(u32, i32)>);

    fn psi_sample() -> Vec<u8> {
        build_psi_bag_table(&[
            (10, vec![(3, 100), (7, -50)]),
            (5, vec![(1, 200)]),
            (20, vec![(2, 10), (4, 20), (9, 30)]),
        ])
        .expect("valid rows")
    }

    #[test]
    fn psi_bag_round_trip_and_lookup() {
        let bytes = psi_sample();
        let table = PsiBagTable::parse(&bytes).expect("parse");
        assert_eq!(table.row_count(), 3);
        assert_eq!(table.max_entries(), 3);
        let row = table.find(10).expect("key 10");
        let entries: Vec<(u32, i32)> = row
            .entries()
            .iter()
            .map(|e| (e.token, e.score_q.raw()))
            .collect();
        assert_eq!(entries, vec![(3, 100), (7, -50)]);
        assert_eq!(row.entries().find(7).map(|s| s.raw()), Some(-50));
        assert_eq!(row.entries().find(4), None);
        assert!(table.find(11).is_none());

        // canonicalization is order-independent
        let again = build_psi_bag_table(&[
            (20, vec![(9, 30), (2, 10), (4, 20)]),
            (10, vec![(7, -50), (3, 100)]),
            (5, vec![(1, 200)]),
        ])
        .unwrap();
        assert_eq!(again, bytes);
    }

    #[test]
    fn psi_bag_rejects_corruption() {
        let good = psi_sample();
        let mut b = good.clone();
        b[0] = b'X';
        assert!(PsiBagTable::parse(&b).is_err());
        let mut b = good.clone();
        b[4] = 9;
        assert!(PsiBagTable::parse(&b).is_err());
        let mut b = good.clone();
        b.truncate(b.len() - 1);
        assert!(PsiBagTable::parse(&b).is_err());
    }

    #[test]
    fn psi_bag_builder_rejects_noncanonical() {
        assert!(build_psi_bag_table(&[(1, vec![])]).is_err());
        assert!(build_psi_bag_table(&[(1, vec![(2, 1)]), (1, vec![(3, 1)])]).is_err());
        assert!(build_psi_bag_table(&[(1, vec![(2, 1), (2, 5)])]).is_err());
    }

    #[test]
    fn psi_bag_trusted_reconstruction_matches_parse() {
        let bytes = psi_sample();
        let parsed = PsiBagTable::parse(&bytes).expect("parse");
        let trusted = PsiBagTable::from_trusted_bytes(&bytes).expect("trusted reconstruction");
        assert_eq!(trusted.row_count(), parsed.row_count());
        assert_eq!(trusted.max_entries(), parsed.max_entries());
        for key in [5u32, 10, 11, 20, 999] {
            let expected: Option<Vec<(u32, i32)>> = parsed.find(key).map(|row| {
                row.entries()
                    .iter()
                    .map(|e| (e.token, e.score_q.raw()))
                    .collect()
            });
            let actual: Option<Vec<(u32, i32)>> = trusted.find(key).map(|row| {
                row.entries()
                    .iter()
                    .map(|e| (e.token, e.score_q.raw()))
                    .collect()
            });
            assert_eq!(actual, expected, "key {key}");
        }
    }

    #[test]
    fn psi_bag_trusted_reconstruction_never_panics_on_garbage() {
        assert!(PsiBagTable::from_trusted_bytes(&[]).is_none());
        assert!(PsiBagTable::from_trusted_bytes(&[0u8; 4]).is_none());
        assert!(PsiBagTable::from_trusted_bytes(&[b'X'; 64]).is_none());
        let mut truncated = psi_sample();
        truncated.truncate(PSIB_HEADER_LEN);
        // Header-only bytes reconstruct (row_count from the header is 0
        // there is no valid header claiming otherwise in this fixture) --
        // the point is simply that no panic occurs either way.
        let _ = PsiBagTable::from_trusted_bytes(&truncated);
        let mut bad_version = psi_sample();
        bad_version[4] = 9;
        assert!(PsiBagTable::from_trusted_bytes(&bad_version).is_none());
    }

    fn skipmix_sample() -> Vec<u8> {
        // A handful of distinct (content_token, last_token) keys.
        let rows: Vec<SkipmixInputRow> = vec![
            (10, 1, vec![(3, 100), (7, -50)]),
            (10, 2, vec![(4, 42)]),
            (5, 9, vec![(1, 200)]),
            (2001, 77, vec![(2, 10), (4, 20), (9, 30)]),
        ];
        build_skipmix_table(&rows).expect("valid rows")
    }

    #[test]
    fn skipmix_round_trip_and_lookup() {
        let bytes = skipmix_sample();
        let table = SkipmixTable::parse(&bytes).expect("parse");
        assert!(table.capacity().is_power_of_two());
        let row = table.find(10, 1).expect("key (10,1)");
        assert_eq!(row.content_token(), 10);
        assert_eq!(row.last_token(), 1);
        let entries: Vec<(u32, i32)> = row
            .entries()
            .iter()
            .map(|e| (e.token, e.score_q.raw()))
            .collect();
        assert_eq!(entries, vec![(3, 100), (7, -50)]);

        let row2 = table.find(10, 2).expect("key (10,2) distinct from (10,1)");
        assert_eq!(row2.entries().find(4).map(|s| s.raw()), Some(42));

        assert!(table.find(10, 3).is_none());
        assert!(table.find(999, 999).is_none());

        // canonicalization is order-independent (different input order, same bytes)
        let rows_reordered: Vec<SkipmixInputRow> = vec![
            (2001, 77, vec![(9, 30), (2, 10), (4, 20)]),
            (5, 9, vec![(1, 200)]),
            (10, 2, vec![(4, 42)]),
            (10, 1, vec![(7, -50), (3, 100)]),
        ];
        let again = build_skipmix_table(&rows_reordered).unwrap();
        assert_eq!(again, bytes, "canonicalization is order-independent");
    }

    #[test]
    fn skipmix_hash_is_pure_and_stable() {
        // Same inputs -> same hash, always (no seed/randomization).
        assert_eq!(hash_key(10, 1), hash_key(10, 1));
        // Different inputs generally land on different buckets (not a
        // correctness requirement, just a sanity check the mixer isn't
        // degenerate for small distinct inputs).
        assert_ne!(hash_key(10, 1), hash_key(10, 2));
        assert_ne!(hash_key(10, 1), hash_key(1, 10));
    }

    #[test]
    fn skipmix_rejects_corruption() {
        let good = skipmix_sample();
        let mut b = good.clone();
        b[0] = b'X';
        assert!(SkipmixTable::parse(&b).is_err());
        let mut b = good.clone();
        b[4] = 9;
        assert!(SkipmixTable::parse(&b).is_err());
        let mut b = good.clone();
        b.truncate(b.len() - 1);
        assert!(SkipmixTable::parse(&b).is_err());
        // capacity not a power of two
        let mut b = good.clone();
        let bad_cap: u32 = 3;
        b[8..12].copy_from_slice(&bad_cap.to_le_bytes());
        assert!(SkipmixTable::parse(&b).is_err());
    }

    #[test]
    fn skipmix_builder_rejects_noncanonical() {
        assert!(build_skipmix_table(&[(1, 1, vec![])]).is_err());
        assert!(build_skipmix_table(&[(1, 1, vec![(2, 1)]), (1, 1, vec![(3, 1)])]).is_err());
        assert!(build_skipmix_table(&[(1, 1, vec![(2, 1), (2, 5)])]).is_err());
        // (content_token, last_token) collision across the composite key is
        // a duplicate even though the individual components repeat elsewhere:
        assert!(build_skipmix_table(&[(1, 2, vec![(9, 1)]), (1, 2, vec![(9, 2)])]).is_err());
    }

    #[test]
    fn skipmix_header_only_is_valid_empty() {
        let bytes = build_skipmix_table(&[]).unwrap();
        let table = SkipmixTable::parse(&bytes).expect("empty parses");
        assert_eq!(table.max_probe(), 0);
        assert!(table.find(1, 1).is_none());
    }

    #[test]
    fn skipmix_trusted_reconstruction_matches_parse() {
        let bytes = skipmix_sample();
        let parsed = SkipmixTable::parse(&bytes).expect("parse");
        let trusted = SkipmixTable::from_trusted_bytes(&bytes).expect("trusted reconstruction");
        assert_eq!(trusted.capacity(), parsed.capacity());
        assert_eq!(trusted.max_probe(), parsed.max_probe());
        assert_eq!(trusted.max_entries(), parsed.max_entries());
        for (content_token, last_token) in [
            (10u32, 1u32),
            (10, 2),
            (5, 9),
            (2001, 77),
            (10, 3),
            (999, 999),
        ] {
            let expected: Option<Vec<(u32, i32)>> =
                parsed.find(content_token, last_token).map(|row| {
                    row.entries()
                        .iter()
                        .map(|e| (e.token, e.score_q.raw()))
                        .collect()
                });
            let actual: Option<Vec<(u32, i32)>> =
                trusted.find(content_token, last_token).map(|row| {
                    row.entries()
                        .iter()
                        .map(|e| (e.token, e.score_q.raw()))
                        .collect()
                });
            assert_eq!(actual, expected, "key ({content_token}, {last_token})");
        }
    }

    #[test]
    fn skipmix_trusted_reconstruction_never_panics_on_garbage() {
        assert!(SkipmixTable::from_trusted_bytes(&[]).is_none());
        assert!(SkipmixTable::from_trusted_bytes(&[0u8; 4]).is_none());
        assert!(SkipmixTable::from_trusted_bytes(&[b'X'; 64]).is_none());
        let mut bad_version = skipmix_sample();
        bad_version[4] = 9;
        assert!(SkipmixTable::from_trusted_bytes(&bad_version).is_none());
        // capacity not a power of two is rejected at the header stage.
        let mut bad_cap = skipmix_sample();
        let cap: u32 = 3;
        bad_cap[8..12].copy_from_slice(&cap.to_le_bytes());
        assert!(SkipmixTable::from_trusted_bytes(&bad_cap).is_none());
        // A truncated buffer reconstructs cheaply (header still fits) but
        // every lookup must stay panic-free via `.get()`-guarded access.
        let mut truncated = skipmix_sample();
        truncated.truncate(SKMX_HEADER_LEN);
        if let Some(table) = SkipmixTable::from_trusted_bytes(&truncated) {
            assert!(table.find(10, 1).is_none());
        }
    }

    #[test]
    fn skipmix_handles_collisions_within_bound() {
        // Force a tiny capacity by constructing many keys and confirm every
        // one is still found after placement (exercises real probing, not
        // just the zero/one-collision happy path).
        let mut rows: Vec<SkipmixInputRow> = Vec::new();
        for i in 0..200u32 {
            rows.push((
                i,
                i.wrapping_mul(7).wrapping_add(3),
                vec![(i % 11, i as i32)],
            ));
        }
        let bytes = build_skipmix_table(&rows).expect("places all keys within bound");
        let table = SkipmixTable::parse(&bytes).expect("parse");
        for i in 0..200u32 {
            let last = i.wrapping_mul(7).wrapping_add(3);
            let row = table
                .find(i, last)
                .unwrap_or_else(|| panic!("missing key ({i}, {last})"));
            assert_eq!(row.entries().find(i % 11).map(|s| s.raw()), Some(i as i32));
        }
    }
}
