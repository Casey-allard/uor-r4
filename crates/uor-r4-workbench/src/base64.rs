//! Canonical standard-padded base64 for raw byte transport.

use std::fmt;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base64Error {
    InvalidEncoding,
    DecodedTooLarge,
}

impl fmt::Display for Base64Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEncoding => formatter.write_str("invalid canonical base64"),
            Self::DecodedTooLarge => formatter.write_str("decoded base64 exceeds byte limit"),
        }
    }
}

impl std::error::Error for Base64Error {}

fn value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Decode the exact canonical standard-alphabet, required-padding form.
///
/// The decoded length is checked before allocation. Empty input is the
/// canonical encoding of an empty raw buffer and remains admissible.
pub fn decode_canonical(input: &str, max_decoded_bytes: usize) -> Result<Vec<u8>, Base64Error> {
    let bytes = input.as_bytes();
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    if bytes.len() % 4 != 0 {
        return Err(Base64Error::InvalidEncoding);
    }

    let padding = if bytes.ends_with(b"==") {
        2
    } else if bytes.ends_with(b"=") {
        1
    } else {
        0
    };
    let groups = bytes.len() / 4;
    let decoded_len = groups
        .checked_mul(3)
        .and_then(|length| length.checked_sub(padding))
        .ok_or(Base64Error::InvalidEncoding)?;
    if decoded_len > max_decoded_bytes {
        return Err(Base64Error::DecodedTooLarge);
    }

    let mut decoded = Vec::with_capacity(decoded_len);
    for (group_index, chunk) in bytes.chunks_exact(4).enumerate() {
        let last = group_index + 1 == groups;
        let a = value(chunk[0]).ok_or(Base64Error::InvalidEncoding)?;
        let b = value(chunk[1]).ok_or(Base64Error::InvalidEncoding)?;

        match (chunk[2], chunk[3]) {
            (b'=', b'=') if last && b & 0x0f == 0 => {
                decoded.push((a << 2) | (b >> 4));
            }
            (third, b'=') if last => {
                let c = value(third).ok_or(Base64Error::InvalidEncoding)?;
                if c & 0x03 != 0 {
                    return Err(Base64Error::InvalidEncoding);
                }
                decoded.push((a << 2) | (b >> 4));
                decoded.push((b << 4) | (c >> 2));
            }
            (third, fourth) if third != b'=' && fourth != b'=' => {
                let c = value(third).ok_or(Base64Error::InvalidEncoding)?;
                let d = value(fourth).ok_or(Base64Error::InvalidEncoding)?;
                decoded.push((a << 2) | (b >> 4));
                decoded.push((b << 4) | (c >> 2));
                decoded.push((c << 6) | d);
            }
            _ => return Err(Base64Error::InvalidEncoding),
        }
    }

    if decoded.len() != decoded_len || encode_standard(&decoded) != input {
        return Err(Base64Error::InvalidEncoding);
    }
    Ok(decoded)
}

/// Encode with the RFC 4648 standard alphabet and required `=` padding.
pub fn encode_standard(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    let groups = bytes.len().saturating_add(2) / 3;
    let mut output = String::with_capacity(groups.saturating_mul(4));
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = chunk.get(1).copied().unwrap_or(0);
        let c = chunk.get(2).copied().unwrap_or(0);
        output.push(ALPHABET[(a >> 2) as usize] as char);
        output.push(ALPHABET[(((a & 0x03) << 4) | (b >> 4)) as usize] as char);
        if chunk.len() >= 2 {
            output.push(ALPHABET[(((b & 0x0f) << 2) | (c >> 6)) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() == 3 {
            output.push(ALPHABET[(c & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_padded_vectors_round_trip_exactly() {
        for (raw, encoded) in [
            (b"".as_slice(), ""),
            (b"f".as_slice(), "Zg=="),
            (b"fo".as_slice(), "Zm8="),
            (b"foo".as_slice(), "Zm9v"),
            (&[0xff], "/w=="),
        ] {
            assert_eq!(encode_standard(raw), encoded);
            assert_eq!(decode_canonical(encoded, raw.len()).unwrap(), raw);
        }
    }

    #[test]
    fn rejects_noncanonical_alphabet_padding_whitespace_and_pad_bits() {
        for value in [
            "Zg", "Zg=", "Zg===", "Z g==", "Zg==\n", "_w==", "AB==", "AAB=", "=AAA", "AA=A",
            "AA==AAAA",
        ] {
            assert_eq!(
                decode_canonical(value, 32),
                Err(Base64Error::InvalidEncoding),
                "{value:?}"
            );
        }
    }

    #[test]
    fn rejects_decoded_length_before_returning_bytes() {
        assert_eq!(
            decode_canonical("Zm9v", 2),
            Err(Base64Error::DecodedTooLarge)
        );
        assert_eq!(decode_canonical("Zm9v", 3).unwrap(), b"foo");
    }
}
