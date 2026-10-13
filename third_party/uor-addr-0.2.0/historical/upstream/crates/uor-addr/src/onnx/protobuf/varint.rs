//! Base-128 varint decoding (protobuf v3 wire format §"Base 128
//! Varints"): each byte's high bit is a continuation flag; the low 7
//! bits are data, least-significant group first.

use super::wire::WireError;

/// Maximum bytes in a 64-bit varint (`ceil(64 / 7)`).
pub const VARINT_MAX_BYTES: usize = 10;

/// Read a varint starting at `buf[pos]`. Returns `(value, new_pos)`.
///
/// # Errors
///
/// [`WireError::Truncated`] if the buffer ends mid-varint;
/// [`WireError::VarintOverflow`] if the encoding exceeds 10 bytes.
pub fn read_varint(buf: &[u8], pos: usize) -> Result<(u64, usize), WireError> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    let mut i = pos;
    loop {
        if i >= buf.len() {
            return Err(WireError::Truncated);
        }
        if i - pos >= VARINT_MAX_BYTES {
            return Err(WireError::VarintOverflow);
        }
        let byte = buf[i];
        result |= u64::from(byte & 0x7F) << shift;
        i += 1;
        if byte & 0x80 == 0 {
            return Ok((result, i));
        }
        shift += 7;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_byte() {
        assert_eq!(read_varint(&[0x00], 0).unwrap(), (0, 1));
        assert_eq!(read_varint(&[0x01], 0).unwrap(), (1, 1));
        assert_eq!(read_varint(&[0x7F], 0).unwrap(), (127, 1));
    }

    #[test]
    fn multi_byte() {
        // 150 = 0x96 0x01
        assert_eq!(read_varint(&[0x96, 0x01], 0).unwrap(), (150, 2));
        // 300 = 0xAC 0x02
        assert_eq!(read_varint(&[0xAC, 0x02], 0).unwrap(), (300, 2));
    }

    #[test]
    fn truncated_is_error() {
        assert_eq!(read_varint(&[0x80], 0), Err(WireError::Truncated));
    }

    #[test]
    fn overflow_is_error() {
        assert_eq!(read_varint(&[0x80; 11], 0), Err(WireError::VarintOverflow));
    }
}
