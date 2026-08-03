//! Borrowed packed lexical context rows.
//!
//! The root prior remains the unigram row in EMIT. NGRAM carries the
//! context-conditioned rows: one-token keys are bigrams and two-token keys
//! are trigrams. Rows and entries are canonical, so lookup is deterministic
//! and requires no allocation.

use crate::error::FormatError;
use crate::types::ScoreQ;

pub const NGRAM_MAGIC: [u8; 4] = *b"NGR1";
pub const NGRAM_VERSION: u16 = 1;
pub const NGRAM_HEADER_LEN: usize = 16;
pub const NGRAM_ROW_LEN: usize = 20;
pub const NGRAM_ENTRY_LEN: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NgramRow<'a> {
    bytes: &'a [u8],
    entries: &'a [u8],
}

impl<'a> NgramRow<'a> {
    pub fn context_len(&self) -> u8 {
        self.bytes[0]
    }

    pub fn key(&self) -> (u32, u32) {
        (read_u32(&self.bytes[4..8]), read_u32(&self.bytes[8..12]))
    }

    pub fn entries(&self) -> NgramEntries<'a> {
        NgramEntries {
            bytes: self.entries,
            remaining: (self.entries.len() / NGRAM_ENTRY_LEN) as u32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NgramEntry {
    pub token: u32,
    pub score_q: ScoreQ,
}

pub struct NgramEntries<'a> {
    bytes: &'a [u8],
    remaining: u32,
}

impl Iterator for NgramEntries<'_> {
    type Item = NgramEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let entry = self.bytes.get(..NGRAM_ENTRY_LEN)?;
        self.bytes = &self.bytes[NGRAM_ENTRY_LEN..];
        self.remaining -= 1;
        Some(NgramEntry {
            token: read_u32(&entry[..4]),
            score_q: ScoreQ::from_raw(i32::from_le_bytes(entry[4..8].try_into().ok()?)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NgramTable<'a> {
    bytes: &'a [u8],
    row_count: u32,
}

pub struct NgramRows<'a> {
    table: NgramTable<'a>,
    next: usize,
}

impl<'a> NgramTable<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, FormatError> {
        if bytes.len() < NGRAM_HEADER_LEN {
            return Err(FormatError::NgramTooShort);
        }
        if bytes[..4] != NGRAM_MAGIC {
            return Err(FormatError::NgramBadMagic);
        }
        if read_u16(&bytes[4..6]) != NGRAM_VERSION {
            return Err(FormatError::NgramUnsupportedVersion);
        }
        if bytes[6..8].iter().any(|&byte| byte != 0) || bytes[14..16].iter().any(|&byte| byte != 0)
        {
            return Err(FormatError::NgramNonZeroReserved);
        }
        let row_count = read_u32(&bytes[8..12]);
        let max_entries = read_u16(&bytes[12..14]) as usize;
        let rows_len = (row_count as usize)
            .checked_mul(NGRAM_ROW_LEN)
            .ok_or(FormatError::NgramBounds)?;
        let entries_start = NGRAM_HEADER_LEN
            .checked_add(rows_len)
            .ok_or(FormatError::NgramBounds)?;
        if entries_start > bytes.len() {
            return Err(FormatError::NgramBounds);
        }

        let mut previous = None;
        let mut expected_entry_start = entries_start;
        for index in 0..row_count as usize {
            let start = NGRAM_HEADER_LEN + index * NGRAM_ROW_LEN;
            let row = &bytes[start..start + NGRAM_ROW_LEN];
            let context_len = row[0];
            if !(1..=2).contains(&context_len) || row[1] != 0 {
                return Err(FormatError::NgramInvalidRow);
            }
            let entry_count = read_u16(&row[2..4]);
            if usize::from(entry_count) > max_entries {
                return Err(FormatError::NgramInvalidRow);
            }
            let key = (read_u32(&row[4..8]), read_u32(&row[8..12]));
            if context_len == 1 && key.1 != 0 {
                return Err(FormatError::NgramInvalidRow);
            }
            let entry_start = read_u32(&row[12..16]) as usize;
            if row[16..20].iter().any(|&byte| byte != 0) || entry_start != expected_entry_start {
                return Err(FormatError::NgramBounds);
            }
            let entry_bytes = (entry_count as usize)
                .checked_mul(NGRAM_ENTRY_LEN)
                .ok_or(FormatError::NgramBounds)?;
            let entry_end = entry_start
                .checked_add(entry_bytes)
                .ok_or(FormatError::NgramBounds)?;
            if entry_end > bytes.len() {
                return Err(FormatError::NgramBounds);
            }
            expected_entry_start = entry_end;
            let sort_key = (context_len, key.0, key.1);
            if previous.is_some_and(|last| last >= sort_key) {
                return Err(FormatError::NgramRowsNotSorted);
            }
            previous = Some(sort_key);
            let entries = &bytes[entry_start..entry_end];
            let mut previous_token = None;
            for chunk in entries.chunks_exact(NGRAM_ENTRY_LEN) {
                let token = read_u32(&chunk[..4]);
                if previous_token.is_some_and(|last| last >= token) {
                    return Err(FormatError::NgramEntriesNotSorted);
                }
                previous_token = Some(token);
            }
        }

        if expected_entry_start != bytes.len() {
            return Err(FormatError::NgramBounds);
        }

        Ok(Self { bytes, row_count })
    }

    pub fn row_count(&self) -> u32 {
        self.row_count
    }

    pub fn rows(&self) -> NgramRows<'a> {
        NgramRows {
            table: *self,
            next: 0,
        }
    }

    pub fn find(&self, context_len: u8, key0: u32, key1: u32) -> Option<NgramRow<'a>> {
        let target = (context_len, key0, key1);
        let mut low = 0usize;
        let mut high = self.row_count as usize;
        while low < high {
            let mid = low + (high - low) / 2;
            let row = self.row(mid)?;
            let current = (row.context_len(), row.key().0, row.key().1);
            if current < target {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        let row = self.row(low)?;
        ((row.context_len(), row.key().0, row.key().1) == target).then_some(row)
    }

    fn row(&self, index: usize) -> Option<NgramRow<'a>> {
        if index >= self.row_count as usize {
            return None;
        }
        let start = NGRAM_HEADER_LEN + index * NGRAM_ROW_LEN;
        let bytes = self.bytes.get(start..start + NGRAM_ROW_LEN)?;
        let entry_count = read_u16(&bytes[2..4]) as usize;
        let entry_start = read_u32(&bytes[12..16]) as usize;
        let entry_end = entry_start.checked_add(entry_count.checked_mul(NGRAM_ENTRY_LEN)?)?;
        Some(NgramRow {
            bytes,
            entries: self.bytes.get(entry_start..entry_end)?,
        })
    }
}

impl<'a> Iterator for NgramRows<'a> {
    type Item = NgramRow<'a>;

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
