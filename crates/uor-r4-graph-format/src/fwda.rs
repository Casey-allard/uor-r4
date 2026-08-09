//! Borrowed packed forward-anchor rows (issue #399).
//!
//! FWDA carries the forward-anchor channel measured by the Gate C
//! instrumentation: a table keyed by (lookahead distance, next-anchor
//! token) whose value is the raw count distribution over the token
//! emitted `distance` positions before that anchor. The section is
//! optional; an artifact without it simply serves without the channel.
//!
//! The wire layout mirrors NGRAM byte for byte (same header, row, and
//! entry widths, same canonical ordering rules) with these semantic
//! differences:
//!
//! - the row `context_len` byte is the lookahead distance (one through
//!   three, the free positions between stride-four anchors);
//! - the row key is `(anchor_token, total)` — the second key slot,
//!   unused by single-token NGRAM keys, carries the row's FULL
//!   pre-truncation evidence total so the loader can derive smoothed
//!   residuals without any extra storage;
//! - entries store `(token, raw count)` as two little-endian u32 words
//!   instead of `(token, ScoreQ)`. Quantization to ScoreQ happens at
//!   load time where the row total is in hand, following the same
//!   delimited compiler-side-float convention the legacy EXCT
//!   quantization uses. Storing counts keeps the section
//!   quantization-law-agnostic and exactly round-trippable.

use crate::error::FormatError;

pub const FWDA_MAGIC: [u8; 4] = *b"FWA1";
pub const FWDA_VERSION: u16 = 1;
pub const FWDA_HEADER_LEN: usize = 16;
pub const FWDA_ROW_LEN: usize = 20;
pub const FWDA_ENTRY_LEN: usize = 8;
/// Largest lookahead distance a row may carry (stride minus one).
pub const FWDA_MAX_DISTANCE: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FwdaRow<'a> {
    bytes: &'a [u8],
    entries: &'a [u8],
}

impl<'a> FwdaRow<'a> {
    /// Lookahead distance to the next anchor (one through three).
    pub fn distance(&self) -> u8 {
        self.bytes[0]
    }

    /// The next-anchor token this row conditions on.
    pub fn anchor(&self) -> u32 {
        read_u32(&self.bytes[4..8])
    }

    /// The row's full pre-truncation evidence total (the smoothing
    /// denominator source), stored in the second key slot.
    pub fn total(&self) -> u32 {
        read_u32(&self.bytes[8..12])
    }

    pub fn entries(&self) -> FwdaEntries<'a> {
        FwdaEntries {
            bytes: self.entries,
            remaining: (self.entries.len() / FWDA_ENTRY_LEN) as u32,
        }
    }
}

/// One `(token, raw count)` evidence entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FwdaEntry {
    pub token: u32,
    pub count: u32,
}

pub struct FwdaEntries<'a> {
    bytes: &'a [u8],
    remaining: u32,
}

impl Iterator for FwdaEntries<'_> {
    type Item = FwdaEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let entry = self.bytes.get(..FWDA_ENTRY_LEN)?;
        self.bytes = &self.bytes[FWDA_ENTRY_LEN..];
        self.remaining -= 1;
        Some(FwdaEntry {
            token: read_u32(&entry[..4]),
            count: read_u32(&entry[4..8]),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FwdaTable<'a> {
    bytes: &'a [u8],
    row_count: u32,
}

pub struct FwdaRows<'a> {
    table: FwdaTable<'a>,
    next: usize,
}

impl<'a> FwdaTable<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, crate::NotAProduct> {
        if bytes.len() < FWDA_HEADER_LEN {
            return Err((FormatError::FwdaTooShort).into());
        }
        if bytes[..4] != FWDA_MAGIC {
            return Err((FormatError::FwdaBadMagic).into());
        }
        if read_u16(&bytes[4..6]) != FWDA_VERSION {
            return Err((FormatError::FwdaUnsupportedVersion).into());
        }
        if bytes[6..8].iter().any(|&byte| byte != 0) || bytes[14..16].iter().any(|&byte| byte != 0)
        {
            return Err((FormatError::FwdaNonZeroReserved).into());
        }
        let row_count = read_u32(&bytes[8..12]);
        let max_entries = read_u16(&bytes[12..14]) as usize;
        let rows_len = (row_count as usize)
            .checked_mul(FWDA_ROW_LEN)
            .ok_or(FormatError::FwdaBounds)?;
        let entries_start = FWDA_HEADER_LEN
            .checked_add(rows_len)
            .ok_or(FormatError::FwdaBounds)?;
        if entries_start > bytes.len() {
            return Err((FormatError::FwdaBounds).into());
        }

        let mut previous = None;
        let mut expected_entry_start = entries_start;
        for index in 0..row_count as usize {
            let start = FWDA_HEADER_LEN + index * FWDA_ROW_LEN;
            let row = &bytes[start..start + FWDA_ROW_LEN];
            let distance = row[0];
            if !(1..=FWDA_MAX_DISTANCE).contains(&distance) || row[1] != 0 {
                return Err((FormatError::FwdaInvalidRow).into());
            }
            let entry_count = read_u16(&row[2..4]);
            if usize::from(entry_count) > max_entries {
                return Err((FormatError::FwdaInvalidRow).into());
            }
            let anchor = read_u32(&row[4..8]);
            let total = read_u32(&row[8..12]);
            if total == 0 {
                return Err((FormatError::FwdaInvalidRow).into());
            }
            let entry_start = read_u32(&row[12..16]) as usize;
            if row[16..20].iter().any(|&byte| byte != 0) || entry_start != expected_entry_start {
                return Err((FormatError::FwdaBounds).into());
            }
            let entry_bytes = (entry_count as usize)
                .checked_mul(FWDA_ENTRY_LEN)
                .ok_or(FormatError::FwdaBounds)?;
            let entry_end = entry_start
                .checked_add(entry_bytes)
                .ok_or(FormatError::FwdaBounds)?;
            if entry_end > bytes.len() {
                return Err((FormatError::FwdaBounds).into());
            }
            expected_entry_start = entry_end;
            let sort_key = (distance, anchor);
            if previous.is_some_and(|last| last >= sort_key) {
                return Err((FormatError::FwdaRowsNotSorted).into());
            }
            previous = Some(sort_key);
            let entries = &bytes[entry_start..entry_end];
            let mut previous_token = None;
            for chunk in entries.chunks_exact(FWDA_ENTRY_LEN) {
                let token = read_u32(&chunk[..4]);
                if previous_token.is_some_and(|last| last >= token) {
                    return Err((FormatError::FwdaEntriesNotSorted).into());
                }
                previous_token = Some(token);
            }
        }

        if expected_entry_start != bytes.len() {
            return Err((FormatError::FwdaBounds).into());
        }

        Ok(Self { bytes, row_count })
    }

    pub fn row_count(&self) -> u32 {
        self.row_count
    }

    pub fn rows(&self) -> FwdaRows<'a> {
        FwdaRows {
            table: *self,
            next: 0,
        }
    }

    /// Canonical binary-search lookup by `(distance, anchor_token)`.
    pub fn find(&self, distance: u8, anchor: u32) -> Option<FwdaRow<'a>> {
        let target = (distance, anchor);
        let mut low = 0usize;
        let mut high = self.row_count as usize;
        while low < high {
            let mid = low + (high - low) / 2;
            let row = self.row(mid)?;
            let current = (row.distance(), row.anchor());
            if current < target {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        let row = self.row(low)?;
        ((row.distance(), row.anchor()) == target).then_some(row)
    }

    fn row(&self, index: usize) -> Option<FwdaRow<'a>> {
        if index >= self.row_count as usize {
            return None;
        }
        let start = FWDA_HEADER_LEN + index * FWDA_ROW_LEN;
        let bytes = self.bytes.get(start..start + FWDA_ROW_LEN)?;
        let entry_count = read_u16(&bytes[2..4]) as usize;
        let entry_start = read_u32(&bytes[12..16]) as usize;
        let entry_end = entry_start.checked_add(entry_count.checked_mul(FWDA_ENTRY_LEN)?)?;
        Some(FwdaRow {
            bytes,
            entries: self.bytes.get(entry_start..entry_end)?,
        })
    }
}

impl<'a> Iterator for FwdaRows<'a> {
    type Item = FwdaRow<'a>;

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
