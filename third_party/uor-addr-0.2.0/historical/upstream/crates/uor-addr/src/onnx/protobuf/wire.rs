//! Field-level protobuf reader: iterates the fields of a message,
//! yielding borrowed [`Field`] views. No allocation; the reader holds a
//! `&[u8]` and a cursor.

use super::tag::{Tag, WireType};
use super::varint::read_varint;

/// Wire-format decode errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireError {
    /// Buffer ended mid-field.
    Truncated,
    /// A varint exceeded 10 bytes.
    VarintOverflow,
    /// Wire type bits `3`/`4` (deprecated groups) or anything else
    /// unrecognized.
    UnknownWireType(u8),
    /// Field number `0` is illegal.
    ZeroFieldNumber,
    /// A length-delimited field declared a length running past the
    /// buffer.
    LengthOutOfRange,
}

/// A decoded field value (borrowed for length-delimited payloads).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldValue<'a> {
    /// Wire type 0.
    Varint(u64),
    /// Wire type 1 (little-endian 64-bit).
    Fixed64(u64),
    /// Wire type 5 (little-endian 32-bit).
    Fixed32(u32),
    /// Wire type 2 — string / bytes / embedded message / packed scalars.
    Bytes(&'a [u8]),
}

/// A single decoded field: its number plus value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field<'a> {
    /// Field number.
    pub number: u64,
    /// Field value.
    pub value: FieldValue<'a>,
}

/// Iterates the top-level fields of a protobuf message.
pub struct MessageReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> MessageReader<'a> {
    /// Wrap a message body.
    #[must_use]
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Read the next field, or `None` at end of message.
    ///
    /// # Errors
    ///
    /// Any [`WireError`] from a malformed encoding.
    pub fn next_field(&mut self) -> Result<Option<Field<'a>>, WireError> {
        if self.pos >= self.buf.len() {
            return Ok(None);
        }
        let (tag_v, p) = read_varint(self.buf, self.pos)?;
        self.pos = p;
        let tag = Tag::from_varint(tag_v)?;
        let value = match tag.wire_type {
            WireType::Varint => {
                let (v, p) = read_varint(self.buf, self.pos)?;
                self.pos = p;
                FieldValue::Varint(v)
            }
            WireType::Fixed64 => {
                let b = self.take(8)?;
                FieldValue::Fixed64(u64::from_le_bytes([
                    b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
                ]))
            }
            WireType::Fixed32 => {
                let b = self.take(4)?;
                FieldValue::Fixed32(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            }
            WireType::LengthDelimited => {
                let (len, p) = read_varint(self.buf, self.pos)?;
                self.pos = p;
                let len = usize::try_from(len).map_err(|_| WireError::LengthOutOfRange)?;
                FieldValue::Bytes(self.take(len)?)
            }
        };
        Ok(Some(Field {
            number: tag.field_number,
            value,
        }))
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], WireError> {
        let end = self.pos.checked_add(n).ok_or(WireError::LengthOutOfRange)?;
        if end > self.buf.len() {
            return Err(WireError::Truncated);
        }
        let s = &self.buf[self.pos..end];
        self.pos = end;
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_mixed_fields() {
        // field 1 varint = 13 ; field 2 length-delimited = "hi"
        let buf = [0x08, 0x0D, 0x12, 0x02, b'h', b'i'];
        let mut r = MessageReader::new(&buf);
        let f1 = r.next_field().unwrap().unwrap();
        assert_eq!(f1.number, 1);
        assert_eq!(f1.value, FieldValue::Varint(13));
        let f2 = r.next_field().unwrap().unwrap();
        assert_eq!(f2.number, 2);
        assert_eq!(f2.value, FieldValue::Bytes(b"hi"));
        assert_eq!(r.next_field().unwrap(), None);
    }

    #[test]
    fn truncated_length_delimited() {
        let buf = [0x12, 0x05, b'h', b'i']; // declares 5 bytes, only 2
        let mut r = MessageReader::new(&buf);
        assert_eq!(r.next_field(), Err(WireError::Truncated));
    }
}
