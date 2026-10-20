//! JSON typed input (ADR-023 amended by ADR-060) with JCS-RFC8785 +
//! Unicode NFC canonical-form byte output.
//!
//! JSON canonicalization is **not** a streaming transform: JCS-RFC8785
//! §3.2.3 sorts object members lexicographically by key, which needs
//! per-object storage. The realization therefore materializes the
//! canonical form once, in an `alloc` buffer ([`canonicalize`]), with
//! **no** width / depth / count ceilings: string widths, number widths,
//! object-key counts, array-element counts, and total size are unbounded.
//! The handle then flows through the pipeline as a zero-copy
//! [`TermValue::Borrowed`] carrier over those canonical bytes, and ψ₉
//! folds them through the σ-axis. The single retained bound is
//! [`MAX_JSON_DEPTH`](crate::json::shapes::bounds::MAX_JSON_DEPTH), a
//! native-stack overflow guard on the recursive parser/canonicalizer.
//!
//! [`JsonValue`] (the owned parsed value, `alloc`-gated) holds the
//! structurally-tagged byte form and backs the [`JsonValueRef`] navigator
//! used by the schema-pinned descendants; [`JsonCarrier`] is the borrowed
//! model-input handle the pipeline binds.
//!
//! # Tagged byte layout
//!
//! ```text
//! JsonValue ::= Tag(1 byte) Payload
//!   Tag = 0x00 Null         — no payload
//!   Tag = 0x01 BoolFalse    — no payload
//!   Tag = 0x02 BoolTrue     — no payload
//!   Tag = 0x03 Number       — u32 BE length || N bytes (canonical ASCII)
//!   Tag = 0x04 String       — u32 BE length || N bytes (UTF-8, NFC)
//!   Tag = 0x05 Array        — u32 BE count  || count × JsonValue
//!   Tag = 0x06 Object       — u32 BE count  || count × (u32 BE keylen || key || JsonValue)
//! ```
//!
//! All multi-byte length / count fields are big-endian. Strings and
//! object keys are NFC-normalized at parse time, so the canonical-form
//! emitter is purely structural — it sorts object entries by NFC byte
//! order and emits JCS syntax around already-canonical content.

use prism::operation::TermValue;
use prism::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields,
};

// ─── Tag byte constants (consumed only by the alloc-gated parser /
//     canonicalizer / navigator) ──────────────────────────────────────────

#[cfg(feature = "alloc")]
pub(crate) const TAG_NULL: u8 = 0x00;
#[cfg(feature = "alloc")]
pub(crate) const TAG_FALSE: u8 = 0x01;
#[cfg(feature = "alloc")]
pub(crate) const TAG_TRUE: u8 = 0x02;
#[cfg(feature = "alloc")]
pub(crate) const TAG_NUMBER: u8 = 0x03;
#[cfg(feature = "alloc")]
pub(crate) const TAG_STRING: u8 = 0x04;
#[cfg(feature = "alloc")]
pub(crate) const TAG_ARRAY: u8 = 0x05;
#[cfg(feature = "alloc")]
pub(crate) const TAG_OBJECT: u8 = 0x06;

// ─── ShapeViolation IRIs (alloc-gated parser) ───────────────────────────

#[cfg(feature = "alloc")]
const INVALID_JSON_VIOLATION: prism::pipeline::ShapeViolation = prism::pipeline::ShapeViolation {
    shape_iri: "https://uor.foundation/addr/JsonValue",
    constraint_iri: "https://uor.foundation/addr/JsonValue/validUtf8Json",
    property_iri: "https://uor.foundation/addr/inputBytes",
    expected_range: "https://uor.foundation/addr/ValidUtf8Json",
    min_count: 0,
    max_count: 1,
    kind: prism::pipeline::ViolationKind::ValueCheck,
};

#[cfg(feature = "alloc")]
const DEPTH_BOUND_VIOLATION: prism::pipeline::ShapeViolation = prism::pipeline::ShapeViolation {
    shape_iri: "https://uor.foundation/addr/JsonValue",
    constraint_iri: "https://uor.foundation/addr/JsonValue/depthBound",
    property_iri: "https://uor.foundation/addr/JsonValue/depth",
    expected_range: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
    min_count: 0,
    max_count: crate::json::shapes::bounds::MAX_JSON_DEPTH as u32,
    kind: prism::pipeline::ViolationKind::CardinalityViolation,
};

// ─── JsonCarrier — the borrowed model-input handle (no_alloc) ───────────

/// Borrowed canonical-JSON input handle (ADR-060 borrowed carrier). A
/// thin, `Copy` borrow of canonical bytes produced by [`canonicalize`];
/// `as_binding_value` returns the `Borrowed` carrier zero-copy.
#[derive(Clone, Copy, Debug)]
pub struct JsonCarrier<'a>(&'a [u8]);

impl<'a> JsonCarrier<'a> {
    /// Wrap a canonical-JSON byte slice as a model input handle.
    #[must_use]
    pub fn new(canonical_bytes: &'a [u8]) -> Self {
        Self(canonical_bytes)
    }

    /// Borrow the canonical-JSON bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &'a [u8] {
        self.0
    }
}

impl ConstrainedTypeShape for JsonCarrier<'_> {
    const IRI: &'static str = "https://uor.foundation/addr/JsonValue";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl prism::uor_foundation::pipeline::__sdk_seal::Sealed for JsonCarrier<'_> {}

impl<'a> IntoBindingValue<'a> for JsonCarrier<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::borrowed(self.0)
    }
}

impl PartitionProductFields for JsonCarrier<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

// ═════════════════════════════════════════════════════════════════════
// alloc-gated parser, canonicalizer, owned value, and navigator
// ═════════════════════════════════════════════════════════════════════

#[cfg(feature = "alloc")]
pub use alloc_impl::{canonicalize, ArrayIter, JsonValue, JsonValueRef, ObjectIter};

#[cfg(feature = "alloc")]
mod alloc_impl {
    use super::*;
    use crate::canonical::nfc;
    use crate::json::shapes::bounds::MAX_JSON_DEPTH;
    use alloc::vec::Vec;
    use prism::pipeline::ShapeViolation;

    // ─── JsonValue — the owned parsed value ─────────────────────────────

    /// Owned parsed JSON value, holding the structurally-tagged byte form
    /// documented in the [module header](super). Backs the
    /// [`JsonValueRef`] navigator. There is no width / depth / count
    /// ceiling. **`alloc`-gated** — the pipeline binds the borrowed
    /// [`JsonCarrier`] handle.
    #[derive(Clone, PartialEq, Eq)]
    pub struct JsonValue {
        pub(crate) bytes: Vec<u8>,
    }

    impl core::fmt::Debug for JsonValue {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("JsonValue")
                .field("len", &self.bytes.len())
                .finish_non_exhaustive()
        }
    }

    impl JsonValue {
        /// Parse raw JSON bytes into a typed `JsonValue` (RFC 8259 syntax,
        /// escape decoding, UAX #15 NFC normalization, JCS number
        /// canonicalization).
        ///
        /// # Errors
        ///
        /// - `validUtf8Json` — input is not valid UTF-8 JSON.
        /// - `depthBound` — nesting exceeds the [`MAX_JSON_DEPTH`]
        ///   stack-safety bound.
        pub fn parse(raw: &[u8]) -> Result<Self, ShapeViolation> {
            let mut value = Self { bytes: Vec::new() };
            let mut p = Parser::new(raw);
            p.skip_ws();
            parse_value(&mut p, &mut value, 0)?;
            p.skip_ws();
            if !p.is_eof() {
                return Err(INVALID_JSON_VIOLATION);
            }
            Ok(value)
        }

        /// Borrow the structurally-tagged byte serialization (the runtime
        /// form; **not** the canonical bytes ψ₉ hashes — derive those via
        /// [`canonicalize`]).
        #[must_use]
        pub fn tagged_bytes(&self) -> &[u8] {
            &self.bytes
        }

        fn push_byte(&mut self, b: u8) {
            self.bytes.push(b);
        }

        fn push_u32_be(&mut self, v: u32) {
            self.bytes.extend_from_slice(&v.to_be_bytes());
        }

        fn extend(&mut self, data: &[u8]) {
            self.bytes.extend_from_slice(data);
        }

        fn patch_u32_be(&mut self, offset: usize, v: u32) {
            self.bytes[offset..offset + 4].copy_from_slice(&v.to_be_bytes());
        }
    }

    /// Parse + emit the JCS-RFC8785 + Unicode NFC canonical-form bytes.
    ///
    /// # Errors
    ///
    /// Surfaces any [`ShapeViolation`] [`JsonValue::parse`] would emit.
    pub fn canonicalize(raw: &[u8]) -> Result<Vec<u8>, ShapeViolation> {
        let value = JsonValue::parse(raw)?;
        let mut out = Vec::new();
        let mut pos = 0;
        emit_value(value.tagged_bytes(), &mut pos, &mut out)?;
        Ok(out)
    }

    // ─── Tokenizer ──────────────────────────────────────────────────────

    struct Parser<'a> {
        input: &'a [u8],
        pos: usize,
    }

    impl<'a> Parser<'a> {
        fn new(input: &'a [u8]) -> Self {
            Self { input, pos: 0 }
        }
        fn is_eof(&self) -> bool {
            self.pos >= self.input.len()
        }
        fn peek(&self) -> Result<u8, ShapeViolation> {
            if self.is_eof() {
                return Err(INVALID_JSON_VIOLATION);
            }
            Ok(self.input[self.pos])
        }
        fn bump(&mut self) -> Result<u8, ShapeViolation> {
            let b = self.peek()?;
            self.pos += 1;
            Ok(b)
        }
        fn skip_ws(&mut self) {
            while self.pos < self.input.len() {
                match self.input[self.pos] {
                    b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                    _ => break,
                }
            }
        }
        fn expect(&mut self, byte: u8) -> Result<(), ShapeViolation> {
            if self.bump()? != byte {
                return Err(INVALID_JSON_VIOLATION);
            }
            Ok(())
        }
        fn expect_lit(&mut self, lit: &[u8]) -> Result<(), ShapeViolation> {
            if self.pos + lit.len() > self.input.len()
                || &self.input[self.pos..self.pos + lit.len()] != lit
            {
                return Err(INVALID_JSON_VIOLATION);
            }
            self.pos += lit.len();
            Ok(())
        }
    }

    fn parse_value(
        p: &mut Parser<'_>,
        out: &mut JsonValue,
        depth: usize,
    ) -> Result<(), ShapeViolation> {
        if depth > MAX_JSON_DEPTH {
            return Err(DEPTH_BOUND_VIOLATION);
        }
        p.skip_ws();
        match p.peek()? {
            b'n' => {
                p.expect_lit(b"null")?;
                out.push_byte(TAG_NULL);
                Ok(())
            }
            b't' => {
                p.expect_lit(b"true")?;
                out.push_byte(TAG_TRUE);
                Ok(())
            }
            b'f' => {
                p.expect_lit(b"false")?;
                out.push_byte(TAG_FALSE);
                Ok(())
            }
            b'"' => parse_string(p, out),
            b'-' | b'0'..=b'9' => parse_number(p, out),
            b'[' => parse_array(p, out, depth + 1),
            b'{' => parse_object(p, out, depth + 1),
            _ => Err(INVALID_JSON_VIOLATION),
        }
    }

    fn parse_array(
        p: &mut Parser<'_>,
        out: &mut JsonValue,
        depth: usize,
    ) -> Result<(), ShapeViolation> {
        p.expect(b'[')?;
        out.push_byte(TAG_ARRAY);
        let count_pos = out.bytes.len();
        out.push_u32_be(0);
        let mut count: u32 = 0;
        p.skip_ws();
        if p.peek()? == b']' {
            p.pos += 1;
            return Ok(());
        }
        loop {
            parse_value(p, out, depth)?;
            count += 1;
            p.skip_ws();
            match p.bump()? {
                b',' => {
                    p.skip_ws();
                    continue;
                }
                b']' => break,
                _ => return Err(INVALID_JSON_VIOLATION),
            }
        }
        out.patch_u32_be(count_pos, count);
        Ok(())
    }

    fn parse_object(
        p: &mut Parser<'_>,
        out: &mut JsonValue,
        depth: usize,
    ) -> Result<(), ShapeViolation> {
        p.expect(b'{')?;
        out.push_byte(TAG_OBJECT);
        let count_pos = out.bytes.len();
        out.push_u32_be(0);
        let mut count: u32 = 0;
        p.skip_ws();
        if p.peek()? == b'}' {
            p.pos += 1;
            return Ok(());
        }
        loop {
            p.skip_ws();
            if p.peek()? != b'"' {
                return Err(INVALID_JSON_VIOLATION);
            }
            let key = decode_string_into_nfc(p)?;
            out.push_u32_be(key.len() as u32);
            out.extend(&key);
            p.skip_ws();
            p.expect(b':')?;
            p.skip_ws();
            parse_value(p, out, depth)?;
            count += 1;
            p.skip_ws();
            match p.bump()? {
                b',' => continue,
                b'}' => break,
                _ => return Err(INVALID_JSON_VIOLATION),
            }
        }
        out.patch_u32_be(count_pos, count);
        Ok(())
    }

    fn parse_string(p: &mut Parser<'_>, out: &mut JsonValue) -> Result<(), ShapeViolation> {
        let s = decode_string_into_nfc(p)?;
        out.push_byte(TAG_STRING);
        out.push_u32_be(s.len() as u32);
        out.extend(&s);
        Ok(())
    }

    /// Decode a JSON string literal at the cursor (escape handling +
    /// NFC normalization), returning the NFC-normalized UTF-8 bytes.
    fn decode_string_into_nfc(p: &mut Parser<'_>) -> Result<Vec<u8>, ShapeViolation> {
        p.expect(b'"')?;
        let mut stage1 = Vec::new();
        loop {
            if p.is_eof() {
                return Err(INVALID_JSON_VIOLATION);
            }
            let b = p.input[p.pos];
            match b {
                b'"' => {
                    p.pos += 1;
                    break;
                }
                b'\\' => {
                    p.pos += 1;
                    let esc = p.bump()?;
                    match esc {
                        b'"' => stage1.push(b'"'),
                        b'\\' => stage1.push(b'\\'),
                        b'/' => stage1.push(b'/'),
                        b'b' => stage1.push(0x08),
                        b'f' => stage1.push(0x0C),
                        b'n' => stage1.push(0x0A),
                        b'r' => stage1.push(0x0D),
                        b't' => stage1.push(0x09),
                        b'u' => {
                            let cp = decode_u_escape(p)?;
                            let c = char::from_u32(cp).ok_or(INVALID_JSON_VIOLATION)?;
                            let mut tmp = [0u8; 4];
                            stage1.extend_from_slice(c.encode_utf8(&mut tmp).as_bytes());
                        }
                        _ => return Err(INVALID_JSON_VIOLATION),
                    }
                }
                // Unescaped control characters forbidden by RFC 8259 §7.
                0x00..=0x1F => return Err(INVALID_JSON_VIOLATION),
                _ => {
                    stage1.push(b);
                    p.pos += 1;
                }
            }
        }
        normalize_nfc(&stage1)
    }

    /// NFC-normalize `stage1` into an owned buffer, growing the output
    /// allocation until it fits (UAX #15 NFC expansion is bounded, so the
    /// loop runs at most a couple of iterations).
    fn normalize_nfc(stage1: &[u8]) -> Result<Vec<u8>, ShapeViolation> {
        let mut cap = stage1.len().saturating_mul(3).max(64);
        loop {
            let mut buf = alloc::vec![0u8; cap];
            match nfc::normalize_into(stage1, &mut buf) {
                Ok(n) => {
                    buf.truncate(n);
                    return Ok(buf);
                }
                Err(nfc::NfcError::OutputOverflow) => {
                    cap = cap.saturating_mul(2);
                }
                Err(_) => return Err(INVALID_JSON_VIOLATION),
            }
        }
    }

    fn decode_u_escape(p: &mut Parser<'_>) -> Result<u32, ShapeViolation> {
        let high = decode_hex4(p)?;
        if (0xD800..=0xDBFF).contains(&high) {
            if p.input.get(p.pos..p.pos + 2) != Some(b"\\u") {
                return Err(INVALID_JSON_VIOLATION);
            }
            p.pos += 2;
            let low = decode_hex4(p)?;
            if !(0xDC00..=0xDFFF).contains(&low) {
                return Err(INVALID_JSON_VIOLATION);
            }
            Ok(0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00))
        } else if (0xDC00..=0xDFFF).contains(&high) {
            Err(INVALID_JSON_VIOLATION)
        } else {
            Ok(high)
        }
    }

    fn decode_hex4(p: &mut Parser<'_>) -> Result<u32, ShapeViolation> {
        if p.pos + 4 > p.input.len() {
            return Err(INVALID_JSON_VIOLATION);
        }
        let mut v: u32 = 0;
        for _ in 0..4 {
            let d = p.input[p.pos];
            p.pos += 1;
            let nibble = match d {
                b'0'..=b'9' => (d - b'0') as u32,
                b'a'..=b'f' => 10 + (d - b'a') as u32,
                b'A'..=b'F' => 10 + (d - b'A') as u32,
                _ => return Err(INVALID_JSON_VIOLATION),
            };
            v = (v << 4) | nibble;
        }
        Ok(v)
    }

    fn parse_number(p: &mut Parser<'_>, out: &mut JsonValue) -> Result<(), ShapeViolation> {
        let start = p.pos;
        let mut has_decimal = false;
        let mut has_exponent = false;
        if p.peek()? == b'-' {
            p.pos += 1;
        }
        match p.peek()? {
            b'0' => p.pos += 1,
            b'1'..=b'9' => {
                p.pos += 1;
                while let Ok(b) = p.peek() {
                    if b.is_ascii_digit() {
                        p.pos += 1;
                    } else {
                        break;
                    }
                }
            }
            _ => return Err(INVALID_JSON_VIOLATION),
        }
        if p.peek().ok() == Some(b'.') {
            has_decimal = true;
            p.pos += 1;
            let frac_start = p.pos;
            while let Ok(b) = p.peek() {
                if b.is_ascii_digit() {
                    p.pos += 1;
                } else {
                    break;
                }
            }
            if p.pos == frac_start {
                return Err(INVALID_JSON_VIOLATION);
            }
        }
        if let Ok(b) = p.peek() {
            if b == b'e' || b == b'E' {
                has_exponent = true;
                p.pos += 1;
                if let Ok(s) = p.peek() {
                    if s == b'+' || s == b'-' {
                        p.pos += 1;
                    }
                }
                let exp_start = p.pos;
                while let Ok(d) = p.peek() {
                    if d.is_ascii_digit() {
                        p.pos += 1;
                    } else {
                        break;
                    }
                }
                if p.pos == exp_start {
                    return Err(INVALID_JSON_VIOLATION);
                }
            }
        }
        let raw = &p.input[start..p.pos];
        let canon = canonicalize_number(raw, has_decimal || has_exponent)?;
        out.push_byte(TAG_NUMBER);
        out.push_u32_be(canon.len() as u32);
        out.extend(&canon);
        Ok(())
    }

    /// Canonicalize a JSON number per JCS-RFC8785 §3.2.2.3 (ECMA-262
    /// 7.1.12.1): integer-syntax literals pass through verbatim (RFC 8259
    /// forbids leading zeros / explicit `+`, so they are already in
    /// ECMA-262 ToString form), float-syntax literals (and `-0`) route
    /// through `f64` + `ryu` shortest-round-trip.
    fn canonicalize_number(raw: &[u8], is_float_syntax: bool) -> Result<Vec<u8>, ShapeViolation> {
        let is_negative_zero = raw == b"-0";
        if is_float_syntax || is_negative_zero {
            let s = core::str::from_utf8(raw).map_err(|_| INVALID_JSON_VIOLATION)?;
            let v: f64 = s.parse().map_err(|_| INVALID_JSON_VIOLATION)?;
            let mut ryu_buf = ryu::Buffer::new();
            Ok(ryu_buf.format(v).as_bytes().to_vec())
        } else {
            Ok(raw.to_vec())
        }
    }

    // ─── JCS canonicalizer (tagged bytes → canonical bytes) ─────────────

    fn read_byte(tagged: &[u8], pos: &mut usize) -> Result<u8, ShapeViolation> {
        if *pos >= tagged.len() {
            return Err(INVALID_JSON_VIOLATION);
        }
        let b = tagged[*pos];
        *pos += 1;
        Ok(b)
    }

    fn read_u32_be(tagged: &[u8], pos: &mut usize) -> Result<u32, ShapeViolation> {
        if *pos + 4 > tagged.len() {
            return Err(INVALID_JSON_VIOLATION);
        }
        let v = u32::from_be_bytes([
            tagged[*pos],
            tagged[*pos + 1],
            tagged[*pos + 2],
            tagged[*pos + 3],
        ]);
        *pos += 4;
        Ok(v)
    }

    fn read_slice<'a>(
        tagged: &'a [u8],
        pos: &mut usize,
        len: usize,
    ) -> Result<&'a [u8], ShapeViolation> {
        if *pos + len > tagged.len() {
            return Err(INVALID_JSON_VIOLATION);
        }
        let s = &tagged[*pos..*pos + len];
        *pos += len;
        Ok(s)
    }

    fn emit_value(tagged: &[u8], pos: &mut usize, out: &mut Vec<u8>) -> Result<(), ShapeViolation> {
        let tag = read_byte(tagged, pos)?;
        match tag {
            TAG_NULL => {
                out.extend_from_slice(b"null");
                Ok(())
            }
            TAG_FALSE => {
                out.extend_from_slice(b"false");
                Ok(())
            }
            TAG_TRUE => {
                out.extend_from_slice(b"true");
                Ok(())
            }
            TAG_NUMBER => {
                let len = read_u32_be(tagged, pos)? as usize;
                let bytes = read_slice(tagged, pos, len)?;
                out.extend_from_slice(bytes);
                Ok(())
            }
            TAG_STRING => {
                let len = read_u32_be(tagged, pos)? as usize;
                let bytes = read_slice(tagged, pos, len)?;
                emit_json_string(bytes, out);
                Ok(())
            }
            TAG_ARRAY => {
                let count = read_u32_be(tagged, pos)? as usize;
                out.push(b'[');
                for i in 0..count {
                    if i > 0 {
                        out.push(b',');
                    }
                    emit_value(tagged, pos, out)?;
                }
                out.push(b']');
                Ok(())
            }
            TAG_OBJECT => emit_object(tagged, pos, out),
            _ => Err(INVALID_JSON_VIOLATION),
        }
    }

    fn emit_object(
        tagged: &[u8],
        pos: &mut usize,
        out: &mut Vec<u8>,
    ) -> Result<(), ShapeViolation> {
        let count = read_u32_be(tagged, pos)? as usize;
        // Collect each member's entry offset (start of its u32 keylen),
        // then stable-sort by NFC key bytes (== lexicographic, strings are
        // pre-normalized at parse time).
        let mut entries: Vec<usize> = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(*pos);
            let key_len = read_u32_be(tagged, pos)? as usize;
            *pos += key_len;
            if *pos > tagged.len() {
                return Err(INVALID_JSON_VIOLATION);
            }
            skip_value(tagged, pos)?;
        }
        entries.sort_by(|&a, &b| entry_key(a, tagged).cmp(entry_key(b, tagged)));
        out.push(b'{');
        for (i, &entry_off) in entries.iter().enumerate() {
            if i > 0 {
                out.push(b',');
            }
            let mut p = entry_off;
            let key_len = read_u32_be(tagged, &mut p)? as usize;
            let key_bytes = read_slice(tagged, &mut p, key_len)?;
            emit_json_string(key_bytes, out);
            out.push(b':');
            emit_value(tagged, &mut p, out)?;
        }
        out.push(b'}');
        Ok(())
    }

    fn entry_key(off: usize, tagged: &[u8]) -> &[u8] {
        if off + 4 > tagged.len() {
            return &[];
        }
        let key_len = u32::from_be_bytes([
            tagged[off],
            tagged[off + 1],
            tagged[off + 2],
            tagged[off + 3],
        ]) as usize;
        let start = off + 4;
        if start + key_len > tagged.len() {
            return &[];
        }
        &tagged[start..start + key_len]
    }

    fn skip_value(tagged: &[u8], pos: &mut usize) -> Result<(), ShapeViolation> {
        let tag = read_byte(tagged, pos)?;
        match tag {
            TAG_NULL | TAG_FALSE | TAG_TRUE => Ok(()),
            TAG_NUMBER | TAG_STRING => {
                let len = read_u32_be(tagged, pos)? as usize;
                *pos += len;
                if *pos > tagged.len() {
                    Err(INVALID_JSON_VIOLATION)
                } else {
                    Ok(())
                }
            }
            TAG_ARRAY => {
                let count = read_u32_be(tagged, pos)? as usize;
                for _ in 0..count {
                    skip_value(tagged, pos)?;
                }
                Ok(())
            }
            TAG_OBJECT => {
                let count = read_u32_be(tagged, pos)? as usize;
                for _ in 0..count {
                    let key_len = read_u32_be(tagged, pos)? as usize;
                    *pos += key_len;
                    if *pos > tagged.len() {
                        return Err(INVALID_JSON_VIOLATION);
                    }
                    skip_value(tagged, pos)?;
                }
                Ok(())
            }
            _ => Err(INVALID_JSON_VIOLATION),
        }
    }

    /// Emit `bytes` as a JCS-compliant JSON string literal.
    fn emit_json_string(bytes: &[u8], out: &mut Vec<u8>) {
        out.push(b'"');
        for &b in bytes {
            match b {
                b'"' => out.extend_from_slice(b"\\\""),
                b'\\' => out.extend_from_slice(b"\\\\"),
                0x08 => out.extend_from_slice(b"\\b"),
                0x09 => out.extend_from_slice(b"\\t"),
                0x0A => out.extend_from_slice(b"\\n"),
                0x0C => out.extend_from_slice(b"\\f"),
                0x0D => out.extend_from_slice(b"\\r"),
                0x00..=0x1F => {
                    out.extend_from_slice(b"\\u00");
                    out.push(nibble_hex(b >> 4));
                    out.push(nibble_hex(b & 0x0f));
                }
                _ => out.push(b),
            }
        }
        out.push(b'"');
    }

    fn nibble_hex(n: u8) -> u8 {
        match n {
            0..=9 => b'0' + n,
            10..=15 => b'a' + (n - 10),
            _ => b'0',
        }
    }

    // ─── JsonValueRef — tagged-byte navigator for schema admission ──────

    /// Zero-copy view into a tagged-byte JSON value (or sub-value), used
    /// by the schema-pinned descendants to validate JSON-LD admission
    /// predicates. Keys and string values are NFC-normalized; numbers
    /// carry their canonical ASCII text.
    #[derive(Clone, Copy)]
    pub struct JsonValueRef<'a> {
        tagged: &'a [u8],
        offset: usize,
    }

    impl<'a> JsonValueRef<'a> {
        /// Root navigator over a parsed [`JsonValue`].
        pub fn root(value: &'a JsonValue) -> Self {
            Self {
                tagged: value.tagged_bytes(),
                offset: 0,
            }
        }

        /// Tag byte at this position.
        pub fn tag(&self) -> u8 {
            self.tagged[self.offset]
        }
        pub fn is_null(&self) -> bool {
            self.tag() == TAG_NULL
        }
        pub fn is_bool(&self) -> bool {
            matches!(self.tag(), TAG_FALSE | TAG_TRUE)
        }
        pub fn is_number(&self) -> bool {
            self.tag() == TAG_NUMBER
        }
        pub fn is_string(&self) -> bool {
            self.tag() == TAG_STRING
        }
        pub fn is_array(&self) -> bool {
            self.tag() == TAG_ARRAY
        }
        pub fn is_object(&self) -> bool {
            self.tag() == TAG_OBJECT
        }

        pub fn as_bool(&self) -> Option<bool> {
            match self.tag() {
                TAG_FALSE => Some(false),
                TAG_TRUE => Some(true),
                _ => None,
            }
        }

        /// Borrow the NFC-normalized UTF-8 content of a string value.
        pub fn as_str(&self) -> Option<&'a [u8]> {
            if !self.is_string() {
                return None;
            }
            let mut p = self.offset + 1;
            let len = read_u32_be(self.tagged, &mut p).ok()? as usize;
            Some(&self.tagged[p..p + len])
        }

        /// Borrow the canonical ASCII text of a number value.
        pub fn as_number_str(&self) -> Option<&'a [u8]> {
            if !self.is_number() {
                return None;
            }
            let mut p = self.offset + 1;
            let len = read_u32_be(self.tagged, &mut p).ok()? as usize;
            Some(&self.tagged[p..p + len])
        }

        /// Look up an object entry by its NFC key bytes.
        pub fn get(&self, key: &[u8]) -> Option<JsonValueRef<'a>> {
            let mut iter = self.iter_object()?;
            iter.find_map(|(k, v)| if k == key { Some(v) } else { None })
        }

        /// Iterate object entries `(key_bytes, value_ref)` in tagged-form
        /// (input) order.
        pub fn iter_object(&self) -> Option<ObjectIter<'a>> {
            if !self.is_object() {
                return None;
            }
            let mut p = self.offset + 1;
            let count = read_u32_be(self.tagged, &mut p).ok()? as usize;
            Some(ObjectIter {
                tagged: self.tagged,
                pos: p,
                remaining: count,
            })
        }

        /// Iterate array elements.
        pub fn iter_array(&self) -> Option<ArrayIter<'a>> {
            if !self.is_array() {
                return None;
            }
            let mut p = self.offset + 1;
            let count = read_u32_be(self.tagged, &mut p).ok()? as usize;
            Some(ArrayIter {
                tagged: self.tagged,
                pos: p,
                remaining: count,
            })
        }
    }

    /// Iterator over an object's `(key_bytes, value)` entries.
    pub struct ObjectIter<'a> {
        tagged: &'a [u8],
        pos: usize,
        remaining: usize,
    }

    impl<'a> Iterator for ObjectIter<'a> {
        type Item = (&'a [u8], JsonValueRef<'a>);
        fn next(&mut self) -> Option<Self::Item> {
            if self.remaining == 0 {
                return None;
            }
            let key_len = read_u32_be(self.tagged, &mut self.pos).ok()? as usize;
            let key_end = self.pos + key_len;
            let key = &self.tagged[self.pos..key_end];
            self.pos = key_end;
            let value_offset = self.pos;
            self.pos = skip_to_end(self.tagged, self.pos).ok()?;
            self.remaining -= 1;
            Some((
                key,
                JsonValueRef {
                    tagged: self.tagged,
                    offset: value_offset,
                },
            ))
        }
    }

    /// Iterator over an array's elements.
    pub struct ArrayIter<'a> {
        tagged: &'a [u8],
        pos: usize,
        remaining: usize,
    }

    impl<'a> Iterator for ArrayIter<'a> {
        type Item = JsonValueRef<'a>;
        fn next(&mut self) -> Option<Self::Item> {
            if self.remaining == 0 {
                return None;
            }
            let value_offset = self.pos;
            self.pos = skip_to_end(self.tagged, self.pos).ok()?;
            self.remaining -= 1;
            Some(JsonValueRef {
                tagged: self.tagged,
                offset: value_offset,
            })
        }
    }

    fn skip_to_end(tagged: &[u8], pos: usize) -> Result<usize, ShapeViolation> {
        let mut p = pos;
        let tag = read_byte(tagged, &mut p)?;
        match tag {
            TAG_NULL | TAG_FALSE | TAG_TRUE => Ok(p),
            TAG_NUMBER | TAG_STRING => {
                let len = read_u32_be(tagged, &mut p)? as usize;
                Ok(p + len)
            }
            TAG_ARRAY => {
                let count = read_u32_be(tagged, &mut p)? as usize;
                for _ in 0..count {
                    p = skip_to_end(tagged, p)?;
                }
                Ok(p)
            }
            TAG_OBJECT => {
                let count = read_u32_be(tagged, &mut p)? as usize;
                for _ in 0..count {
                    let key_len = read_u32_be(tagged, &mut p)? as usize;
                    p += key_len;
                    p = skip_to_end(tagged, p)?;
                }
                Ok(p)
            }
            _ => Err(INVALID_JSON_VIOLATION),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parses_simple_object() {
            let v = JsonValue::parse(br#"{"foo":"bar"}"#).expect("valid");
            assert_eq!(v.bytes[0], TAG_OBJECT);
        }

        #[test]
        fn rejects_invalid_json() {
            let err = JsonValue::parse(b"not json").expect_err("must reject");
            assert_eq!(err.shape_iri, INVALID_JSON_VIOLATION.shape_iri);
        }

        #[test]
        fn rejects_overdeep_recursion() {
            use alloc::string::String;
            let mut s = String::new();
            for _ in 0..(MAX_JSON_DEPTH + 2) {
                s.push('[');
            }
            for _ in 0..(MAX_JSON_DEPTH + 2) {
                s.push(']');
            }
            let err = JsonValue::parse(s.as_bytes()).expect_err("must reject");
            assert_eq!(err.constraint_iri, DEPTH_BOUND_VIOLATION.constraint_iri);
        }

        #[test]
        fn accepts_unbounded_string_width() {
            use alloc::format;
            use alloc::string::String;
            let big: String = "a".repeat(200_000);
            let raw = format!("\"{big}\"");
            let canon = canonicalize(raw.as_bytes()).expect("unbounded string admitted");
            assert_eq!(canon.len(), big.len() + 2);
        }

        const CANONICAL_FIXTURES: &[(&[u8], &[u8])] = &[
            (br#"{"foo":"bar"}"#, br#"{"foo":"bar"}"#),
            (br#"{"b": 1, "a": 2}"#, br#"{"a":2,"b":1}"#),
            (
                br#"{"nested": {"deep": {"value": "found"}}}"#,
                br#"{"nested":{"deep":{"value":"found"}}}"#,
            ),
            (
                br#"{"int": 42, "bool": true, "null_val": null}"#,
                br#"{"bool":true,"int":42,"null_val":null}"#,
            ),
            (b"[1, 2, 3]", b"[1,2,3]"),
            (br#"["a", "b", "c"]"#, br#"["a","b","c"]"#),
        ];

        #[test]
        fn canonicalizer_matches_reference_for_inline_fixtures() {
            for (raw, expected) in CANONICAL_FIXTURES {
                let canon = canonicalize(raw).expect("valid");
                assert_eq!(canon, *expected, "raw={raw:?}");
            }
        }

        #[test]
        fn canonicalizer_collapses_unicode_decomposed_to_composed() {
            let decomposed = "{\"name\": \"cafe\u{0301}\"}".as_bytes();
            let composed = "{\"name\":\"caf\u{00E9}\"}".as_bytes();
            assert_eq!(
                canonicalize(decomposed).expect("valid"),
                canonicalize(composed).expect("valid")
            );
        }

        #[test]
        fn canonicalize_is_idempotent_on_its_own_output() {
            for (raw, _expected) in CANONICAL_FIXTURES {
                let once = canonicalize(raw).expect("valid");
                let twice = canonicalize(&once).expect("re-canonicalises");
                assert_eq!(once, twice, "idempotence broken for {raw:?}");
            }
        }
    }
}
