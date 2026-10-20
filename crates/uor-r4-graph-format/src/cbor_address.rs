//! Prism-free BLAKE3-axis CBOR content address for the R4G1 realization.
//!
//! This replaces `uor_addr::cbor::address_blake3` (uor-addr rev `165b51e3`),
//! which reached `uor-prism` and through it the D29-excluded
//! `uor-prism-tensor`/`uor-prism-numerics`. It ports the upstream
//! `cbor/value.rs` re-encoder (RFC 8949 §4.2 deterministic encoding) for the
//! four major types an R4G1 skeleton uses: unsigned integers (0), byte strings
//! (2), text strings (3) and arrays (4). For those types its rules are
//! upstream's:
//! - non-minimal heads are re-emitted minimally;
//! - indefinite-length strings and arrays are re-emitted with definite lengths;
//! - text must be UTF-8;
//! - reserved additional-information values, trailing bytes and nesting deeper
//!   than [`MAX_CBOR_DEPTH`] are rejected;
//! - additional information 31 on an unsigned integer is read as the value 0.
//!
//! Every other major type (negative integers, maps, tags, simple values and
//! floats) is rejected, because an R4G1 skeleton never contains one. The label
//! is `blake3:` plus the lowercase BLAKE3 hex digest of the canonical bytes.
//! The `upstream_parity` tests check both against the pinned upstream crate,
//! which is a dev-dependency only.

use alloc::{format, string::String, vec::Vec};

/// Nesting bound, equal to upstream `cbor::shapes::bounds::MAX_CBOR_DEPTH`.
pub const MAX_CBOR_DEPTH: usize = 1024;

const BREAK: u8 = 0xff;

/// Why a byte string has no CBOR content address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CborAddressError {
    /// Not exactly one well-formed item: truncated, reserved head, bad
    /// indefinite-length chunk, non-UTF-8 text or trailing bytes.
    Invalid,
    /// Nesting deeper than [`MAX_CBOR_DEPTH`].
    TooDeep,
    /// A major type outside the R4G1 subset (1, 5, 6 or 7).
    UnsupportedMajorType(u8),
}

/// Re-encode `raw` (one CBOR item of the R4G1 subset) into its deterministic
/// canonical form.
///
/// **Subset only.** This covers the CBOR that [`crate::r4g1`] constructs:
/// unsigned integers, byte strings, text strings and arrays, including
/// indefinite-length strings (chunked) and arrays. Every other major type
/// (negative integers, maps, tags, simple values and floats) returns
/// [`CborAddressError::UnsupportedMajorType`], where upstream `uor-addr`
/// canonicalizes it. Do not use this for general CBOR.
pub fn canonicalize(raw: &[u8]) -> Result<Vec<u8>, CborAddressError> {
    let mut parser = Parser { data: raw, pos: 0 };
    let mut out = Vec::new();
    parser.item(&mut out, 0)?;
    if parser.pos != raw.len() {
        return Err(CborAddressError::Invalid);
    }
    Ok(out)
}

/// The BLAKE3-axis κ-label (`blake3:<64 lowercase hex>`) of `raw`'s canonical
/// form.
///
/// **Subset only, despite the upstream name.** It matches upstream
/// `uor_addr::cbor::address_blake3` byte-for-byte on the CBOR that
/// [`crate::r4g1`] constructs: unsigned integers, byte strings, text strings
/// and arrays, including indefinite-length strings (chunked) and arrays. For
/// negative integers, maps, tags, simple values and floats it returns
/// [`CborAddressError::UnsupportedMajorType`], where upstream returns a label.
/// A caller that addresses arbitrary CBOR, such as map-bearing records, must
/// not use it.
pub fn address_blake3(raw: &[u8]) -> Result<String, CborAddressError> {
    let canonical = canonicalize(raw)?;
    Ok(format!("blake3:{}", blake3::hash(&canonical).to_hex()))
}

struct Parser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn byte(&mut self) -> Result<u8, CborAddressError> {
        let byte = *self.data.get(self.pos).ok_or(CborAddressError::Invalid)?;
        self.pos += 1;
        Ok(byte)
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], CborAddressError> {
        let end = self.pos.checked_add(n).ok_or(CborAddressError::Invalid)?;
        let slice = self
            .data
            .get(self.pos..end)
            .ok_or(CborAddressError::Invalid)?;
        self.pos = end;
        Ok(slice)
    }

    /// The argument for additional information 0..=27, as upstream's
    /// `arg_for`; 28..=31 carry no argument here and are rejected.
    fn argument(&mut self, ai: u8) -> Result<u64, CborAddressError> {
        Ok(match ai {
            0..=23 => u64::from(ai),
            24 => u64::from(self.byte()?),
            25 => {
                let b = self.take(2)?;
                u64::from(u16::from_be_bytes([b[0], b[1]]))
            }
            26 => {
                let b = self.take(4)?;
                u64::from(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
            }
            27 => {
                let b = self.take(8)?;
                u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
            }
            _ => return Err(CborAddressError::Invalid),
        })
    }

    /// `(major, additional_info, argument)`. Additional information 31
    /// (indefinite length) carries argument 0, as upstream.
    fn head(&mut self) -> Result<(u8, u8, u64), CborAddressError> {
        let initial = self.byte()?;
        let major = initial >> 5;
        let ai = initial & 0x1f;
        let argument = if ai == 31 { 0 } else { self.argument(ai)? };
        Ok((major, ai, argument))
    }

    fn item(&mut self, out: &mut Vec<u8>, depth: usize) -> Result<(), CborAddressError> {
        if depth > MAX_CBOR_DEPTH {
            return Err(CborAddressError::TooDeep);
        }
        let (major, ai, argument) = self.head()?;
        match major {
            0 => emit_head(out, 0, argument),
            2 => {
                let bytes = self.string_payload(ai, argument, 2)?;
                emit_head(out, 2, bytes.len() as u64);
                out.extend_from_slice(&bytes);
            }
            3 => {
                let bytes = self.string_payload(ai, argument, 3)?;
                if core::str::from_utf8(&bytes).is_err() {
                    return Err(CborAddressError::Invalid);
                }
                emit_head(out, 3, bytes.len() as u64);
                out.extend_from_slice(&bytes);
            }
            4 => self.array(out, ai, argument, depth)?,
            other => return Err(CborAddressError::UnsupportedMajorType(other)),
        }
        Ok(())
    }

    /// A definite string's bytes, or the concatenated chunks of an indefinite
    /// one; each chunk must be a definite string of the same major type.
    fn string_payload(
        &mut self,
        ai: u8,
        argument: u64,
        major: u8,
    ) -> Result<Vec<u8>, CborAddressError> {
        if ai != 31 {
            return Ok(self.take(to_usize(argument)?)?.to_vec());
        }
        let mut buffer = Vec::new();
        loop {
            let initial = self.byte()?;
            if initial == BREAK {
                break;
            }
            let chunk_ai = initial & 0x1f;
            if initial >> 5 != major || chunk_ai == 31 {
                return Err(CborAddressError::Invalid);
            }
            let length = self.argument(chunk_ai)?;
            buffer.extend_from_slice(self.take(to_usize(length)?)?);
        }
        Ok(buffer)
    }

    fn array(
        &mut self,
        out: &mut Vec<u8>,
        ai: u8,
        argument: u64,
        depth: usize,
    ) -> Result<(), CborAddressError> {
        if ai != 31 {
            let length = to_usize(argument)?;
            emit_head(out, 4, length as u64);
            for _ in 0..length {
                self.item(out, depth + 1)?;
            }
            return Ok(());
        }
        let mut elements = Vec::new();
        let mut count = 0u64;
        loop {
            if *self.data.get(self.pos).ok_or(CborAddressError::Invalid)? == BREAK {
                self.pos += 1;
                break;
            }
            self.item(&mut elements, depth + 1)?;
            count += 1;
        }
        emit_head(out, 4, count);
        out.extend_from_slice(&elements);
        Ok(())
    }
}

fn to_usize(argument: u64) -> Result<usize, CborAddressError> {
    usize::try_from(argument).map_err(|_| CborAddressError::Invalid)
}

/// A head with the shortest argument encoding (RFC 8949 §4.1).
fn emit_head(out: &mut Vec<u8>, major: u8, argument: u64) {
    let m = major << 5;
    if argument < 24 {
        out.push(m | argument as u8);
    } else if argument <= u64::from(u8::MAX) {
        out.push(m | 24);
        out.push(argument as u8);
    } else if argument <= u64::from(u16::MAX) {
        out.push(m | 25);
        out.extend_from_slice(&(argument as u16).to_be_bytes());
    } else if argument <= u64::from(u32::MAX) {
        out.push(m | 26);
        out.extend_from_slice(&(argument as u32).to_be_bytes());
    } else {
        out.push(m | 27);
        out.extend_from_slice(&argument.to_be_bytes());
    }
}
