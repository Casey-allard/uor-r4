//! Metadata-only JSON addressing: copied upstream parser/emitter and NFC behavior.
//! The original Prism execution/witness APIs are preserved under historical, not compiled.
use alloc::string::String;
use core::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationKind {
    ValueCheck,
    CardinalityViolation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalError {
    pub shape_iri: &'static str,
    pub constraint_iri: &'static str,
    pub property_iri: &'static str,
    pub expected_range: &'static str,
    pub min_count: u32,
    pub max_count: u32,
    pub kind: ViolationKind,
}
impl fmt::Display for CanonicalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "canonical JSON refused: {}", self.constraint_iri)
    }
}

pub const MAX_JSON_DEPTH: usize = 1024;
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
const INVALID_JSON_VIOLATION: CanonicalError = CanonicalError {
    shape_iri: "https://uor.foundation/addr/JsonValue",
    constraint_iri: "https://uor.foundation/addr/JsonValue/validUtf8Json",
    property_iri: "https://uor.foundation/addr/inputBytes",
    expected_range: "https://uor.foundation/addr/ValidUtf8Json",
    min_count: 0,
    max_count: 1,
    kind: ViolationKind::ValueCheck,
};

#[cfg(feature = "alloc")]
const DEPTH_BOUND_VIOLATION: CanonicalError = CanonicalError {
    shape_iri: "https://uor.foundation/addr/JsonValue",
    constraint_iri: "https://uor.foundation/addr/JsonValue/depthBound",
    property_iri: "https://uor.foundation/addr/JsonValue/depth",
    expected_range: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
    min_count: 0,
    max_count: MAX_JSON_DEPTH as u32,
    kind: ViolationKind::CardinalityViolation,
};

#[cfg(feature = "alloc")]
pub use alloc_impl::canonicalize;
#[cfg(feature = "alloc")]
mod alloc_impl {
    use super::*;
    use crate::canonical::nfc;
    use alloc::vec::Vec;
    type ShapeViolation = CanonicalError;
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFailure {
    InvalidJson,
}
impl fmt::Display for AddressFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("input is not valid canonicalizable JSON")
    }
}
/// Content label only. This narrowed API does not emit or claim a Prism replay witness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressLabel(String);
impl AddressLabel {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}
impl fmt::Display for AddressLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressOutcome {
    pub address: AddressLabel,
}
#[cfg(feature = "alloc")]
pub fn address_blake3(input_bytes: &[u8]) -> Result<AddressOutcome, AddressFailure> {
    let canonical = canonicalize(input_bytes).map_err(|_| AddressFailure::InvalidJson)?;
    let digest = blake3::hash(&canonical);
    Ok(AddressOutcome {
        address: AddressLabel(alloc::format!("blake3:{}", digest.to_hex())),
    })
}
