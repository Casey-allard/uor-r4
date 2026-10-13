//! ASN.1 DER typed input (ADR-023 amended by ADR-060).
//!
//! DER is the canonical form by construction (ITU-T X.690 §10): a
//! well-formed DER byte sequence is its own canonical representative, so
//! the ψ₉ canonicalizer is the **identity** on it. The realization
//! therefore validates the input is valid DER at the host boundary and
//! flows the **input bytes themselves** through the pipeline as a
//! zero-copy [`TermValue::Borrowed`] carrier — no transformation, no
//! buffer, no width / element-count ceiling. The only retained bound is
//! [`MAX_ASN1_DEPTH`], a native-stack-overflow guard on the recursive
//! TLV validator.
//!
//! [`Asn1Value`] (the owned DER **builder**, `alloc`-gated) constructs
//! canonical DER programmatically (`boolean`, `integer`, `sequence`,
//! `set`, …) for reference and testing; [`Asn1Carrier`] is the borrowed
//! model-input handle the pipeline binds.
//!
//! # Supported universal-tag cases
//!
//! `Boolean`, `Integer`, `BitString`, `OctetString`, `Null`,
//! `ObjectIdentifier`, `Utf8String`, `PrintableString`, `IA5String`,
//! `UTCTime`, `GeneralizedTime`, `Sequence`, `Set`.

use prism::operation::TermValue;
use prism::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields, ShapeViolation,
    ViolationKind,
};

use crate::asn1::shapes::bounds::MAX_ASN1_DEPTH;

// ─── DER tag bytes ──────────────────────────────────────────────────────

pub(crate) const TAG_BOOLEAN: u8 = 0x01;
pub(crate) const TAG_INTEGER: u8 = 0x02;
pub(crate) const TAG_BIT_STRING: u8 = 0x03;
pub(crate) const TAG_OCTET_STRING: u8 = 0x04;
pub(crate) const TAG_NULL: u8 = 0x05;
pub(crate) const TAG_OID: u8 = 0x06;
pub(crate) const TAG_UTF8_STRING: u8 = 0x0C;
pub(crate) const TAG_PRINTABLE_STRING: u8 = 0x13;
pub(crate) const TAG_IA5_STRING: u8 = 0x16;
pub(crate) const TAG_UTC_TIME: u8 = 0x17;
pub(crate) const TAG_GENERALIZED_TIME: u8 = 0x18;
pub(crate) const TAG_SEQUENCE: u8 = 0x30;
pub(crate) const TAG_SET: u8 = 0x31;

// ─── ShapeViolation IRIs ────────────────────────────────────────────────

const INVALID_DER_VIOLATION: ShapeViolation = ShapeViolation {
    shape_iri: "https://uor.foundation/addr/Asn1Value",
    constraint_iri: "https://uor.foundation/addr/Asn1Value/validDer",
    property_iri: "https://uor.foundation/addr/inputBytes",
    expected_range: "https://uor.foundation/addr/ValidDerBytes",
    min_count: 0,
    max_count: 1,
    kind: ViolationKind::ValueCheck,
};

const DEPTH_BOUND_VIOLATION: ShapeViolation = ShapeViolation {
    shape_iri: "https://uor.foundation/addr/Asn1Value",
    constraint_iri: "https://uor.foundation/addr/Asn1Value/depthBound",
    property_iri: "https://uor.foundation/addr/Asn1Value/depth",
    expected_range: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
    min_count: 0,
    max_count: MAX_ASN1_DEPTH as u32,
    kind: ViolationKind::CardinalityViolation,
};

// ─── DER validation (no_alloc) ──────────────────────────────────────────

/// Validate that `raw` is a single well-formed DER value per X.690 §§ 8 /
/// 10 / 11 (not merely valid BER — long-form lengths below the short-form
/// threshold and indefinite lengths are rejected).
///
/// # Errors
///
/// - [`INVALID_DER_VIOLATION`] (`validDer`) — malformed or non-canonical
///   DER, or trailing bytes after the top-level value.
/// - [`DEPTH_BOUND_VIOLATION`] (`depthBound`) — nesting exceeds the
///   [`MAX_ASN1_DEPTH`] native-stack-safety bound.
pub fn validate_der(raw: &[u8]) -> Result<(), ShapeViolation> {
    let mut pos = 0;
    validate_tlv(raw, &mut pos, 0)?;
    if pos != raw.len() {
        return Err(INVALID_DER_VIOLATION);
    }
    Ok(())
}

fn validate_tlv(buf: &[u8], pos: &mut usize, depth: usize) -> Result<(), ShapeViolation> {
    if depth > MAX_ASN1_DEPTH {
        return Err(DEPTH_BOUND_VIOLATION);
    }
    if *pos >= buf.len() {
        return Err(INVALID_DER_VIOLATION);
    }
    let tag = buf[*pos];
    *pos += 1;
    let content_len = decode_length(buf, pos)?;
    if *pos + content_len > buf.len() {
        return Err(INVALID_DER_VIOLATION);
    }
    let content_end = *pos + content_len;
    match tag {
        TAG_BOOLEAN => {
            if content_len != 1 {
                return Err(INVALID_DER_VIOLATION);
            }
            let b = buf[*pos];
            if b != 0x00 && b != 0xFF {
                return Err(INVALID_DER_VIOLATION);
            }
            *pos += 1;
        }
        TAG_INTEGER => {
            if content_len == 0 {
                return Err(INVALID_DER_VIOLATION);
            }
            if content_len >= 2 {
                let b0 = buf[*pos];
                let b1 = buf[*pos + 1];
                if b0 == 0x00 && (b1 & 0x80) == 0 {
                    return Err(INVALID_DER_VIOLATION);
                }
                if b0 == 0xFF && (b1 & 0x80) != 0 {
                    return Err(INVALID_DER_VIOLATION);
                }
            }
            *pos = content_end;
        }
        TAG_OCTET_STRING => {
            *pos = content_end;
        }
        TAG_NULL => {
            if content_len != 0 {
                return Err(INVALID_DER_VIOLATION);
            }
        }
        TAG_BIT_STRING => {
            if content_len == 0 {
                return Err(INVALID_DER_VIOLATION);
            }
            let unused = buf[*pos];
            if unused > 7 {
                return Err(INVALID_DER_VIOLATION);
            }
            if content_len == 1 && unused != 0 {
                return Err(INVALID_DER_VIOLATION);
            }
            if content_len > 1 && unused > 0 {
                let last = buf[content_end - 1];
                let mask = (1u8 << unused) - 1;
                if last & mask != 0 {
                    return Err(INVALID_DER_VIOLATION);
                }
            }
            *pos = content_end;
        }
        TAG_OID => {
            if content_len == 0 {
                return Err(INVALID_DER_VIOLATION);
            }
            let mut p = *pos;
            while p < content_end {
                let sub_start = p;
                while p < content_end && buf[p] & 0x80 != 0 {
                    p += 1;
                }
                if p >= content_end {
                    return Err(INVALID_DER_VIOLATION);
                }
                p += 1;
                if p - sub_start > 1 && buf[sub_start] == 0x80 {
                    return Err(INVALID_DER_VIOLATION);
                }
            }
            if p != content_end {
                return Err(INVALID_DER_VIOLATION);
            }
            *pos = content_end;
        }
        TAG_UTF8_STRING => {
            let bytes = &buf[*pos..content_end];
            core::str::from_utf8(bytes).map_err(|_| INVALID_DER_VIOLATION)?;
            *pos = content_end;
        }
        TAG_PRINTABLE_STRING => {
            for &b in &buf[*pos..content_end] {
                let ok = b.is_ascii_alphanumeric()
                    || matches!(
                        b,
                        b' ' | b'\''
                            | b'('
                            | b')'
                            | b'+'
                            | b','
                            | b'-'
                            | b'.'
                            | b'/'
                            | b':'
                            | b'='
                            | b'?'
                    );
                if !ok {
                    return Err(INVALID_DER_VIOLATION);
                }
            }
            *pos = content_end;
        }
        TAG_IA5_STRING => {
            for &b in &buf[*pos..content_end] {
                if b > 127 {
                    return Err(INVALID_DER_VIOLATION);
                }
            }
            *pos = content_end;
        }
        TAG_UTC_TIME | TAG_GENERALIZED_TIME => {
            for &b in &buf[*pos..content_end] {
                if !b.is_ascii() {
                    return Err(INVALID_DER_VIOLATION);
                }
            }
            *pos = content_end;
        }
        TAG_SEQUENCE | TAG_SET => {
            while *pos < content_end {
                validate_tlv(buf, pos, depth + 1)?;
            }
            if *pos != content_end {
                return Err(INVALID_DER_VIOLATION);
            }
        }
        _ => return Err(INVALID_DER_VIOLATION),
    }
    Ok(())
}

fn decode_length(buf: &[u8], pos: &mut usize) -> Result<usize, ShapeViolation> {
    if *pos >= buf.len() {
        return Err(INVALID_DER_VIOLATION);
    }
    let first = buf[*pos];
    *pos += 1;
    if first < 0x80 {
        Ok(first as usize)
    } else {
        let nbytes = (first & 0x7F) as usize;
        if nbytes == 0 {
            return Err(INVALID_DER_VIOLATION);
        }
        if nbytes > core::mem::size_of::<usize>() || *pos + nbytes > buf.len() {
            return Err(INVALID_DER_VIOLATION);
        }
        let mut len: usize = 0;
        for _ in 0..nbytes {
            len = (len << 8) | (buf[*pos] as usize);
            *pos += 1;
        }
        if len < 128 {
            return Err(INVALID_DER_VIOLATION);
        }
        Ok(len)
    }
}

// ─── Asn1Carrier — the borrowed model-input handle (no_alloc) ───────────

/// Borrowed validated-DER input handle (ADR-060 borrowed carrier). DER is
/// canonical, so the handle borrows the input bytes directly and
/// `as_binding_value` returns them as a zero-copy `Borrowed` carrier.
#[derive(Clone, Copy, Debug)]
pub struct Asn1Carrier<'a>(&'a [u8]);

impl<'a> Asn1Carrier<'a> {
    /// Wrap a validated DER byte slice as a model input handle. Call
    /// [`validate_der`] first.
    #[must_use]
    pub fn new(der: &'a [u8]) -> Self {
        Self(der)
    }

    /// Borrow the canonical (DER) bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &'a [u8] {
        self.0
    }
}

impl ConstrainedTypeShape for Asn1Carrier<'_> {
    const IRI: &'static str = "https://uor.foundation/addr/Asn1Value";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl prism::uor_foundation::pipeline::__sdk_seal::Sealed for Asn1Carrier<'_> {}

impl<'a> IntoBindingValue<'a> for Asn1Carrier<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        // DER is canonical (X.690 §10); ψ₉ folds the input bytes directly.
        TermValue::borrowed(self.0)
    }
}

impl PartitionProductFields for Asn1Carrier<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

// ─── Asn1Value — the owned DER builder (alloc) ──────────────────────────

/// Owned DER value + builder. Constructs canonical X.690 DER
/// programmatically for reference and testing. **`alloc`-gated** — the
/// pipeline binds the borrowed [`Asn1Carrier`] handle, which needs no
/// allocator. There is no width or element-count ceiling.
#[cfg(feature = "alloc")]
#[derive(Clone, PartialEq, Eq)]
pub struct Asn1Value {
    bytes: alloc::vec::Vec<u8>,
}

#[cfg(feature = "alloc")]
impl core::fmt::Debug for Asn1Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Asn1Value")
            .field("len", &self.bytes.len())
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "alloc")]
impl Asn1Value {
    fn from_vec(bytes: alloc::vec::Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Validate a DER byte sequence and retain an owned copy.
    ///
    /// # Errors
    ///
    /// Surfaces the [`ShapeViolation`] [`validate_der`] would raise.
    pub fn parse(raw: &[u8]) -> Result<Self, ShapeViolation> {
        validate_der(raw)?;
        Ok(Self::from_vec(raw.to_vec()))
    }

    /// Build a Boolean (DER tag `0x01`).
    #[must_use]
    pub fn boolean(value: bool) -> Self {
        Self::from_vec(alloc::vec![TAG_BOOLEAN, 1, if value { 0xFF } else { 0x00 }])
    }

    /// Build a Null (DER tag `0x05`).
    #[must_use]
    pub fn null() -> Self {
        Self::from_vec(alloc::vec![TAG_NULL, 0])
    }

    /// Build an Integer (DER tag `0x02`) from a signed 64-bit value.
    /// DER §8.3: minimum-octets two's-complement big-endian.
    #[must_use]
    pub fn integer(value: i64) -> Self {
        let be = value.to_be_bytes();
        let mut start = 0;
        if value >= 0 {
            while start < 7 && be[start] == 0x00 && (be[start + 1] & 0x80) == 0 {
                start += 1;
            }
        } else {
            while start < 7 && be[start] == 0xFF && (be[start + 1] & 0x80) != 0 {
                start += 1;
            }
        }
        Self::primitive_vec(TAG_INTEGER, &be[start..])
    }

    /// Build an OctetString (DER tag `0x04`) from raw bytes.
    #[must_use]
    pub fn octet_string(bytes: &[u8]) -> Self {
        Self::primitive_vec(TAG_OCTET_STRING, bytes)
    }

    fn primitive_vec(tag: u8, content: &[u8]) -> Self {
        let mut out = alloc::vec::Vec::new();
        out.push(tag);
        push_length(&mut out, content.len());
        out.extend_from_slice(content);
        Self::from_vec(out)
    }

    /// Build a Sequence (DER tag `0x30`).
    #[must_use]
    pub fn sequence(children: &[Asn1Value]) -> Self {
        Self::constructed(TAG_SEQUENCE, children, false)
    }

    /// Build a Set (DER tag `0x31`). DER (X.690 §11.6) requires Set
    /// element ordering by ascending encoded-element byte sequence.
    #[must_use]
    pub fn set(children: &[Asn1Value]) -> Self {
        Self::constructed(TAG_SET, children, true)
    }

    fn constructed(tag: u8, children: &[Asn1Value], sort: bool) -> Self {
        let mut kids: alloc::vec::Vec<&[u8]> = children.iter().map(|c| c.tagged_bytes()).collect();
        if sort {
            kids.sort_unstable();
        }
        let total: usize = kids.iter().map(|k| k.len()).sum();
        let mut out = alloc::vec::Vec::new();
        out.push(tag);
        push_length(&mut out, total);
        for k in kids {
            out.extend_from_slice(k);
        }
        Self::from_vec(out)
    }

    /// Build a BIT STRING (DER tag `0x03`). X.690 §8.6 / §11.2.
    ///
    /// # Errors
    ///
    /// [`INVALID_DER_VIOLATION`] for invalid unused-bit counts or
    /// non-zero trailing bits.
    pub fn bit_string(bits: &[u8], unused_bits: u8) -> Result<Self, ShapeViolation> {
        if unused_bits > 7 {
            return Err(INVALID_DER_VIOLATION);
        }
        if bits.is_empty() && unused_bits != 0 {
            return Err(INVALID_DER_VIOLATION);
        }
        if !bits.is_empty() && unused_bits > 0 {
            let last = bits[bits.len() - 1];
            let mask = (1u8 << unused_bits) - 1;
            if last & mask != 0 {
                return Err(INVALID_DER_VIOLATION);
            }
        }
        let mut content = alloc::vec::Vec::with_capacity(1 + bits.len());
        content.push(unused_bits);
        content.extend_from_slice(bits);
        Ok(Self::primitive_vec(TAG_BIT_STRING, &content))
    }

    /// Build an OBJECT IDENTIFIER (DER tag `0x06`). X.690 §8.19.
    ///
    /// # Errors
    ///
    /// [`INVALID_DER_VIOLATION`] for fewer than two arcs or out-of-range
    /// leading arcs.
    pub fn object_identifier(arcs: &[u32]) -> Result<Self, ShapeViolation> {
        if arcs.len() < 2 {
            return Err(INVALID_DER_VIOLATION);
        }
        let x1 = arcs[0];
        let x2 = arcs[1];
        if x1 > 2 {
            return Err(INVALID_DER_VIOLATION);
        }
        if x1 < 2 && x2 >= 40 {
            return Err(INVALID_DER_VIOLATION);
        }
        let mut content = alloc::vec::Vec::new();
        encode_oid_subid(40 * x1 + x2, &mut content);
        for &arc in &arcs[2..] {
            encode_oid_subid(arc, &mut content);
        }
        Ok(Self::primitive_vec(TAG_OID, &content))
    }

    /// Build a UTF8String (DER tag `0x0C`).
    #[must_use]
    pub fn utf8_string(s: &str) -> Self {
        Self::primitive_vec(TAG_UTF8_STRING, s.as_bytes())
    }

    /// Build a PrintableString (DER tag `0x13`). X.680 §41.4 character set.
    ///
    /// # Errors
    ///
    /// [`INVALID_DER_VIOLATION`] for characters outside the
    /// PrintableString set.
    pub fn printable_string(s: &str) -> Result<Self, ShapeViolation> {
        for c in s.chars() {
            let ok = c.is_ascii_alphanumeric()
                || matches!(
                    c,
                    ' ' | '\'' | '(' | ')' | '+' | ',' | '-' | '.' | '/' | ':' | '=' | '?'
                );
            if !ok {
                return Err(INVALID_DER_VIOLATION);
            }
        }
        Ok(Self::primitive_vec(TAG_PRINTABLE_STRING, s.as_bytes()))
    }

    /// Build an IA5String (DER tag `0x16`). X.680 §41.2.
    ///
    /// # Errors
    ///
    /// [`INVALID_DER_VIOLATION`] for non-ASCII input.
    pub fn ia5_string(s: &str) -> Result<Self, ShapeViolation> {
        if !s.is_ascii() {
            return Err(INVALID_DER_VIOLATION);
        }
        Ok(Self::primitive_vec(TAG_IA5_STRING, s.as_bytes()))
    }

    /// Borrow the DER-encoded canonical bytes.
    #[must_use]
    pub fn tagged_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// X.690 §8.19.2 — base-128 encoding of an OID sub-identifier into `out`.
#[cfg(feature = "alloc")]
fn encode_oid_subid(mut value: u32, out: &mut alloc::vec::Vec<u8>) {
    if value == 0 {
        out.push(0);
        return;
    }
    let mut buf = [0u8; 5];
    let mut i = 0;
    while value > 0 {
        buf[i] = (value & 0x7F) as u8;
        value >>= 7;
        i += 1;
    }
    for j in (1..i).rev() {
        out.push(buf[j] | 0x80);
    }
    out.push(buf[0]);
}

/// X.690 §8.1.3 length octets appended to `out`.
#[cfg(feature = "alloc")]
fn push_length(out: &mut alloc::vec::Vec<u8>, len: usize) {
    if len < 128 {
        out.push(len as u8);
        return;
    }
    let mut value = len;
    let mut bytes = [0u8; 8];
    let mut count = 0;
    while value > 0 {
        bytes[count] = (value & 0xFF) as u8;
        value >>= 8;
        count += 1;
    }
    out.push(0x80 | (count as u8));
    for i in 0..count {
        out.push(bytes[count - 1 - i]);
    }
}

/// Canonical-bytes accessor — DER is the canonical form per X.690 §10, so
/// canonicalization is the identity on validated input.
///
/// **Available only under the `alloc` feature.**
///
/// # Errors
///
/// Surfaces the [`ShapeViolation`] [`validate_der`] would raise.
#[cfg(feature = "alloc")]
pub fn canonicalize(raw: &[u8]) -> Result<alloc::vec::Vec<u8>, ShapeViolation> {
    validate_der(raw)?;
    Ok(raw.to_vec())
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[test]
    fn boolean_der_encoding_matches_x690_8_2_2() {
        assert_eq!(Asn1Value::boolean(true).tagged_bytes(), &[0x01, 0x01, 0xFF]);
        assert_eq!(
            Asn1Value::boolean(false).tagged_bytes(),
            &[0x01, 0x01, 0x00]
        );
    }

    #[test]
    fn null_der_encoding_matches_x690_8_8() {
        assert_eq!(Asn1Value::null().tagged_bytes(), &[0x05, 0x00]);
    }

    #[test]
    fn integer_der_encoding_minimum_octets() {
        assert_eq!(Asn1Value::integer(0).tagged_bytes(), &[0x02, 0x01, 0x00]);
        assert_eq!(Asn1Value::integer(127).tagged_bytes(), &[0x02, 0x01, 0x7F]);
        assert_eq!(
            Asn1Value::integer(128).tagged_bytes(),
            &[0x02, 0x02, 0x00, 0x80]
        );
        assert_eq!(Asn1Value::integer(-1).tagged_bytes(), &[0x02, 0x01, 0xFF]);
        assert_eq!(Asn1Value::integer(-128).tagged_bytes(), &[0x02, 0x01, 0x80]);
    }

    #[test]
    fn set_sorts_children_by_encoding() {
        let sorted = Asn1Value::set(&[Asn1Value::integer(2), Asn1Value::integer(1)]);
        let direct = Asn1Value::set(&[Asn1Value::integer(1), Asn1Value::integer(2)]);
        assert_eq!(sorted.tagged_bytes(), direct.tagged_bytes());
    }

    #[test]
    fn parse_round_trips_well_formed_der() {
        let cases: &[Asn1Value] = &[
            Asn1Value::boolean(true),
            Asn1Value::null(),
            Asn1Value::integer(42),
            Asn1Value::octet_string(b"hello"),
            Asn1Value::sequence(&[Asn1Value::integer(1), Asn1Value::boolean(true)]),
        ];
        for v in cases {
            let parsed = Asn1Value::parse(v.tagged_bytes()).expect("valid DER");
            assert_eq!(parsed.tagged_bytes(), v.tagged_bytes());
        }
    }

    #[test]
    fn rejects_non_canonical_boolean_byte() {
        let err = validate_der(&[0x01, 0x01, 0x01]).expect_err("rejects non-canonical");
        assert_eq!(err.constraint_iri, INVALID_DER_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_non_minimum_integer_encoding() {
        let err = validate_der(&[0x02, 0x02, 0x00, 0x01]).expect_err("non-minimal");
        assert_eq!(err.constraint_iri, INVALID_DER_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_long_form_length_under_128() {
        let err = validate_der(&[0x04, 0x81, 0x05, 0, 0, 0, 0, 0]).expect_err("non-canonical");
        assert_eq!(err.constraint_iri, INVALID_DER_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_indefinite_length() {
        let err = validate_der(&[0x30, 0x80]).expect_err("BER not DER");
        assert_eq!(err.constraint_iri, INVALID_DER_VIOLATION.constraint_iri);
    }
}
