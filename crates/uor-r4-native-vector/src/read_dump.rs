//! claude-389: one first-pick read with its converted inputs, as lossless little-endian bytes.
//!
//! The probe's native replay writes this file from a saved state's converted Q31 words, and the
//! CUDA parity example reads it back, so the CPU operator and its CUDA twin are compared on the
//! real inputs of a trained state (the sheet's control 2), not only on seeded fixtures.

use crate::{Error, Q31FirstPickRead, Result};

const MAGIC: &[u8; 8] = b"UORQ31RD";

/// A first-pick read and every input of one batched call to [`Q31FirstPickRead::apply`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Q31ReadDump {
    pub read: Q31FirstPickRead,
    pub batch: usize,
    pub time: usize,
    /// `batch * time * width` stream words.
    pub stream: Vec<i64>,
    /// `batch * time` token ids.
    pub ids: Vec<u32>,
    /// `vocab * width` table words.
    pub table: Vec<i64>,
}

fn overflow() -> Error {
    Error::Invalid("a read dump count overflows".into())
}

fn field(value: usize) -> Result<[u8; 4]> {
    u32::try_from(value)
        .map(u32::to_le_bytes)
        .map_err(|_| Error::Invalid("a read dump field exceeds u32".into()))
}

struct Cursor<'a> {
    bytes: &'a [u8],
}

impl<'a> Cursor<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        if self.bytes.len() < count {
            return Err(Error::Invalid("a read dump ends early".into()));
        }
        let (head, rest) = self.bytes.split_at(count);
        self.bytes = rest;
        Ok(head)
    }

    fn u32(&mut self) -> Result<usize> {
        let word = self.take(4)?;
        Ok(u32::from_le_bytes(word.try_into().expect("four bytes")) as usize)
    }

    fn words(&mut self, count: usize) -> Result<Vec<i64>> {
        let bytes = count
            .checked_mul(8)
            .ok_or_else(|| Error::Invalid("a read dump count overflows".into()))?;
        Ok(self
            .take(bytes)?
            .chunks_exact(8)
            .map(|c| i64::from_le_bytes(c.try_into().expect("eight bytes")))
            .collect())
    }
}

impl Q31ReadDump {
    /// Magic, batch, time, width, vocab, the read's own bytes, then stream, ids and table.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let width = self.read.width();
        let rows = self.batch.checked_mul(self.time).ok_or_else(overflow)?;
        if Some(self.stream.len()) != rows.checked_mul(width)
            || self.ids.len() != rows
            || width == 0
            || !self.table.len().is_multiple_of(width)
        {
            return Err(Error::Invalid(
                "a read dump's inputs do not match its shape".into(),
            ));
        }
        let read = self.read.to_bytes();
        let mut bytes =
            Vec::with_capacity(32 + read.len() + 8 * (self.stream.len() + self.table.len()));
        bytes.extend_from_slice(MAGIC);
        for value in [
            self.batch,
            self.time,
            width,
            self.table.len() / width,
            read.len(),
        ] {
            bytes.extend_from_slice(&field(value)?);
        }
        bytes.extend_from_slice(&read);
        for word in &self.stream {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for id in &self.ids {
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        for word in &self.table {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        Ok(bytes)
    }

    /// Restore a dump; refuses a wrong magic, a width other than the read's, and missing or
    /// trailing bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor { bytes };
        if cursor.take(MAGIC.len())? != MAGIC {
            return Err(Error::Invalid("not a Q31 read dump".into()));
        }
        let (batch, time, width, vocab, read_len) = (
            cursor.u32()?,
            cursor.u32()?,
            cursor.u32()?,
            cursor.u32()?,
            cursor.u32()?,
        );
        let read = Q31FirstPickRead::from_bytes(cursor.take(read_len)?)?;
        if read.width() != width {
            return Err(Error::Invalid(
                "a read dump's width is not its read's".into(),
            ));
        }
        let rows = batch.checked_mul(time).ok_or_else(overflow)?;
        let stream = cursor.words(rows.checked_mul(width).ok_or_else(overflow)?)?;
        let ids = cursor
            .take(rows.checked_mul(4).ok_or_else(overflow)?)?
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes(c.try_into().expect("four bytes")))
            .collect();
        let table = cursor.words(vocab.checked_mul(width).ok_or_else(overflow)?)?;
        if !cursor.bytes.is_empty() {
            return Err(Error::Invalid("a read dump has trailing bytes".into()));
        }
        Ok(Self {
            read,
            batch,
            time,
            stream,
            ids,
            table,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump() -> Q31ReadDump {
        let width = 16;
        let gate = [1 << 30, -(1 << 29), 3, -7];
        let read = Q31FirstPickRead::new(vec![gate; 4], vec![gate; 4], vec![1 << 30; width], 2)
            .expect("a valid read");
        let (batch, time, vocab) = (2, 3, 5);
        let stream = (0..batch * time * width)
            .map(|i| (i as i64 * 7919) % (1 << 20) - (1 << 19))
            .collect();
        let ids = (0..batch * time).map(|i| (i % vocab) as u32).collect();
        let table = (0..vocab * width)
            .map(|i| (i as i64 * 104_729) % (1 << 24) - (1 << 23))
            .collect();
        Q31ReadDump {
            read,
            batch,
            time,
            stream,
            ids,
            table,
        }
    }

    #[test]
    fn a_dump_round_trips_and_its_apply_is_unchanged() {
        let dump = dump();
        let bytes = dump.to_bytes().expect("bytes");
        let back = Q31ReadDump::from_bytes(&bytes).expect("restored");
        assert_eq!(back, dump);
        let apply = |d: &Q31ReadDump| d.read.apply(&d.stream, &d.ids, &d.table, d.batch, d.time);
        assert_eq!(apply(&back).expect("apply"), apply(&dump).expect("apply"));
    }

    #[test]
    fn a_damaged_dump_is_refused() {
        let bytes = dump().to_bytes().expect("bytes");
        let mut magic = bytes.clone();
        magic[0] ^= 1;
        assert!(Q31ReadDump::from_bytes(&magic).is_err());
        assert!(Q31ReadDump::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        let mut long = bytes.clone();
        long.push(0);
        assert!(Q31ReadDump::from_bytes(&long).is_err());
        // A header whose shape overflows `usize` is an error, not a panic or a wrapped size.
        let mut huge = bytes;
        let big = u32::MAX.to_le_bytes();
        huge[MAGIC.len()..MAGIC.len() + 4].copy_from_slice(&big);
        huge[MAGIC.len() + 4..MAGIC.len() + 8].copy_from_slice(&big);
        let refused = Q31ReadDump::from_bytes(&huge).err().expect("refused");
        assert!(refused.to_string().contains("overflows"), "{refused}");
    }
}
