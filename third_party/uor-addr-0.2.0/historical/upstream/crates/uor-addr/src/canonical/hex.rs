//! Lowercase-hex byte-emit — `no_std`, `no_alloc`.
//!
//! The κ-label's 64-byte suffix is the lowercase-hex serialization of
//! the σ-projection's 32-byte SHA-256 digest. This module's
//! [`encode_lower_into`] is the canonical emit-path; callers reach it
//! through [`crate::label::AddressLabel`] in normal use, but it is
//! `pub` so any realization wiring κ-derivation against a different
//! hash axis (per ARCHITECTURE.md "Alternate hash axes") reuses the
//! same lowercase-hex discipline.
//!
//! # Output discipline
//!
//! Each input byte produces exactly two ASCII characters from
//! `0123456789abcdef`. Output length is always `2 × input.len()`.
//! ASCII case is canonical: the κ-label IRI's wire-format pins
//! lowercase hex (`sha256:7a38…`, not `sha256:7A38…`).

/// Output-buffer-too-small error. Surfaced to callers when `out` cannot
/// hold `2 × input.len()` bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexOutputOverflow;

/// Emit `input` as lowercase ASCII hex into `out`. Returns the number
/// of bytes written — always `2 × input.len()` on success.
///
/// # Errors
///
/// - [`HexOutputOverflow`] — `out.len() < 2 * input.len()`.
pub fn encode_lower_into(input: &[u8], out: &mut [u8]) -> Result<usize, HexOutputOverflow> {
    let need = input.len().checked_mul(2).ok_or(HexOutputOverflow)?;
    if out.len() < need {
        return Err(HexOutputOverflow);
    }
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for (i, &byte) in input.iter().enumerate() {
        out[i * 2] = HEX[(byte >> 4) as usize];
        out[i * 2 + 1] = HEX[(byte & 0x0f) as usize];
    }
    Ok(need)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_emits_nothing() {
        let mut out = [0u8; 4];
        assert_eq!(encode_lower_into(&[], &mut out).unwrap(), 0);
    }

    #[test]
    fn each_byte_yields_two_lowercase_hex_chars() {
        let mut out = [0u8; 4];
        let n = encode_lower_into(&[0x00, 0xff], &mut out).unwrap();
        assert_eq!(n, 4);
        assert_eq!(&out[..n], b"00ff");
    }

    #[test]
    fn sha256_zero_digest_round_trips_published_fixture() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let digest: [u8; 32] = [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
            0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
            0x78, 0x52, 0xb8, 0x55,
        ];
        let mut out = [0u8; 64];
        let n = encode_lower_into(&digest, &mut out).unwrap();
        assert_eq!(n, 64);
        assert_eq!(
            &out[..n],
            b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn rejects_undersized_output_buffer() {
        let mut out = [0u8; 1];
        assert_eq!(
            encode_lower_into(&[0xab, 0xcd], &mut out),
            Err(HexOutputOverflow)
        );
    }

    #[test]
    fn rejects_zero_length_output_for_nonempty_input() {
        let mut out = [0u8; 0];
        assert_eq!(encode_lower_into(&[0xff], &mut out), Err(HexOutputOverflow));
    }
}
