//! CBOR typed input (ADR-023 amended by ADR-060) with RFC 8949 §4.2
//! Deterministic-Encoding canonical-form byte output.
//!
//! CBOR canonicalization is **not** a streaming transform: §4.2.1 sorts
//! map keys by the bytewise lexicographic order of their *encoded* keys,
//! which needs per-map storage. The realization therefore materializes the
//! canonical form once, in an `alloc` buffer ([`canonicalize`]), with **no**
//! width / depth / count ceilings beyond the single
//! [`MAX_CBOR_DEPTH`](crate::cbor::shapes::bounds::MAX_CBOR_DEPTH)
//! native-stack overflow guard. The handle then flows through the pipeline
//! as a zero-copy [`TermValue::Borrowed`] carrier and ψ₉ folds it through
//! the σ-axis.
//!
//! # RFC 8949 §4.2 Deterministic Encoding (the canonical form)
//!
//! The canonicalizer accepts any well-formed CBOR data item and re-emits
//! it under the deterministic-encoding rules:
//!
//! - **Preferred (shortest) integer/argument encoding** (§4.2.1 / §4.1):
//!   every head's argument uses the fewest bytes (inline 0–23, then 1, 2,
//!   4, 8).
//! - **Definite-length only** (§4.2.1): indefinite-length byte/text
//!   strings, arrays, and maps in the *input* are folded to their
//!   definite-length canonical form.
//! - **Map keys sorted** (§4.2.1) bytewise-lexicographically by their
//!   canonical encodings; duplicate keys are rejected.
//! - **Shortest-float / canonical NaN** (§4.2.2): a float is emitted in
//!   the shortest of half / single / double that round-trips its value
//!   exactly; every NaN collapses to the canonical half-precision
//!   `0xf9 0x7e 0x00`.
//!
//! Exactly one top-level data item is admitted (trailing bytes are
//! rejected).

use prism::operation::TermValue;
use prism::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields,
};

// ─── ShapeViolation IRIs (alloc-gated canonicalizer) ────────────────────

#[cfg(feature = "alloc")]
const INVALID_CBOR_VIOLATION: prism::pipeline::ShapeViolation = prism::pipeline::ShapeViolation {
    shape_iri: "https://uor.foundation/addr/CborValue",
    constraint_iri: "https://uor.foundation/addr/CborValue/wellFormedCbor",
    property_iri: "https://uor.foundation/addr/inputBytes",
    expected_range: "https://uor.foundation/addr/WellFormedCbor",
    min_count: 0,
    max_count: 1,
    kind: prism::pipeline::ViolationKind::ValueCheck,
};

#[cfg(feature = "alloc")]
const DEPTH_BOUND_VIOLATION: prism::pipeline::ShapeViolation = prism::pipeline::ShapeViolation {
    shape_iri: "https://uor.foundation/addr/CborValue",
    constraint_iri: "https://uor.foundation/addr/CborValue/depthBound",
    property_iri: "https://uor.foundation/addr/CborValue/depth",
    expected_range: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
    min_count: 0,
    max_count: crate::cbor::shapes::bounds::MAX_CBOR_DEPTH as u32,
    kind: prism::pipeline::ViolationKind::CardinalityViolation,
};

// ─── CborCarrier — the borrowed model-input handle (no_alloc) ───────────

/// Borrowed canonical-CBOR input handle (ADR-060 borrowed carrier). A
/// thin, `Copy` borrow of canonical bytes produced by [`canonicalize`];
/// `as_binding_value` returns the `Borrowed` carrier zero-copy.
#[derive(Clone, Copy, Debug)]
pub struct CborCarrier<'a>(&'a [u8]);

impl<'a> CborCarrier<'a> {
    /// Wrap a canonical-CBOR byte slice as a model input handle.
    #[must_use]
    pub fn new(canonical_bytes: &'a [u8]) -> Self {
        Self(canonical_bytes)
    }

    /// Borrow the canonical-CBOR bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &'a [u8] {
        self.0
    }
}

impl ConstrainedTypeShape for CborCarrier<'_> {
    const IRI: &'static str = "https://uor.foundation/addr/CborValue";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl prism::uor_foundation::pipeline::__sdk_seal::Sealed for CborCarrier<'_> {}

impl<'a> IntoBindingValue<'a> for CborCarrier<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::borrowed(self.0)
    }
}

impl PartitionProductFields for CborCarrier<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

// ═════════════════════════════════════════════════════════════════════
// alloc-gated RFC 8949 §4.2 deterministic-encoding canonicalizer
// ═════════════════════════════════════════════════════════════════════

#[cfg(feature = "alloc")]
pub use alloc_impl::canonicalize;

#[cfg(feature = "alloc")]
mod alloc_impl {
    extern crate alloc;
    use alloc::vec::Vec;
    use prism::pipeline::ShapeViolation;

    use super::{DEPTH_BOUND_VIOLATION, INVALID_CBOR_VIOLATION};
    use crate::cbor::shapes::bounds::MAX_CBOR_DEPTH;

    /// Re-encode `raw` (any well-formed CBOR item) into its RFC 8949 §4.2
    /// deterministic-encoding canonical form.
    ///
    /// # Errors
    ///
    /// [`ShapeViolation`] if `raw` is not exactly one well-formed CBOR data
    /// item, contains a reserved/invalid head, a non-UTF-8 text string, a
    /// map with duplicate keys, or nests deeper than [`MAX_CBOR_DEPTH`].
    pub fn canonicalize(raw: &[u8]) -> Result<Vec<u8>, ShapeViolation> {
        let mut p = Parser { data: raw, pos: 0 };
        let mut out = Vec::new();
        p.item(&mut out, 0)?;
        if p.pos != raw.len() {
            return Err(INVALID_CBOR_VIOLATION); // trailing bytes — not a single item
        }
        Ok(out)
    }

    const BREAK: u8 = 0xff;

    struct Parser<'a> {
        data: &'a [u8],
        pos: usize,
    }

    impl<'a> Parser<'a> {
        fn byte(&mut self) -> Result<u8, ShapeViolation> {
            let b = *self.data.get(self.pos).ok_or(INVALID_CBOR_VIOLATION)?;
            self.pos += 1;
            Ok(b)
        }

        fn take(&mut self, n: usize) -> Result<&'a [u8], ShapeViolation> {
            let end = self.pos.checked_add(n).ok_or(INVALID_CBOR_VIOLATION)?;
            let s = self.data.get(self.pos..end).ok_or(INVALID_CBOR_VIOLATION)?;
            self.pos = end;
            Ok(s)
        }

        /// Read a head, returning `(major, additional_info, argument)`. For
        /// `additional_info == 31` the argument is meaningless (indefinite /
        /// break); callers branch on `ai` first.
        fn head(&mut self) -> Result<(u8, u8, u64), ShapeViolation> {
            let ib = self.byte()?;
            let major = ib >> 5;
            let ai = ib & 0x1f;
            let arg = match ai {
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
                31 => 0, // indefinite / break — caller handles
                _ => return Err(INVALID_CBOR_VIOLATION), // 28,29,30 reserved
            };
            Ok((major, ai, arg))
        }

        fn item(&mut self, out: &mut Vec<u8>, depth: usize) -> Result<(), ShapeViolation> {
            if depth > MAX_CBOR_DEPTH {
                return Err(DEPTH_BOUND_VIOLATION);
            }
            let (major, ai, arg) = self.head()?;
            match major {
                0 => emit_head(out, 0, arg), // unsigned int
                1 => emit_head(out, 1, arg), // negative int
                2 => {
                    let bytes = self.string_payload(ai, arg, 2)?;
                    emit_head(out, 2, bytes.len() as u64);
                    out.extend_from_slice(&bytes);
                }
                3 => {
                    let bytes = self.string_payload(ai, arg, 3)?;
                    if core::str::from_utf8(&bytes).is_err() {
                        return Err(INVALID_CBOR_VIOLATION);
                    }
                    emit_head(out, 3, bytes.len() as u64);
                    out.extend_from_slice(&bytes);
                }
                4 => self.array(out, ai, arg, depth)?,
                5 => self.map(out, ai, arg, depth)?,
                6 => {
                    if ai == 31 {
                        return Err(INVALID_CBOR_VIOLATION);
                    }
                    emit_head(out, 6, arg); // tag
                    self.item(out, depth + 1)?; // tagged content
                }
                7 => self.simple_or_float(out, ai, arg)?,
                _ => unreachable!("major is 3 bits"),
            }
            Ok(())
        }

        /// Collect a (possibly indefinite-length) byte/text string payload
        /// into a contiguous buffer. `expect_major` is 2 or 3; indefinite
        /// chunks must each be definite strings of the same major type.
        fn string_payload(
            &mut self,
            ai: u8,
            arg: u64,
            expect_major: u8,
        ) -> Result<Vec<u8>, ShapeViolation> {
            if ai != 31 {
                return Ok(self.take(usize_arg(arg)?)?.to_vec());
            }
            let mut buf = Vec::new();
            loop {
                let ib = self.byte()?;
                if ib == BREAK {
                    break;
                }
                let major = ib >> 5;
                let cai = ib & 0x1f;
                if major != expect_major || cai == 31 {
                    return Err(INVALID_CBOR_VIOLATION); // nested indefinite / wrong type
                }
                let n = self.arg_for(cai)?;
                buf.extend_from_slice(self.take(usize_arg(n)?)?);
            }
            Ok(buf)
        }

        /// Read just the argument for an already-consumed initial byte's
        /// additional-info `ai` (no major byte read).
        fn arg_for(&mut self, ai: u8) -> Result<u64, ShapeViolation> {
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
                _ => return Err(INVALID_CBOR_VIOLATION),
            })
        }

        fn array(
            &mut self,
            out: &mut Vec<u8>,
            ai: u8,
            arg: u64,
            depth: usize,
        ) -> Result<(), ShapeViolation> {
            if ai != 31 {
                let n = usize_arg(arg)?;
                emit_head(out, 4, n as u64);
                for _ in 0..n {
                    self.item(out, depth + 1)?;
                }
                return Ok(());
            }
            // Indefinite: canonicalize each element into a buffer, count the
            // emitted items, then emit the definite-length head + body.
            let mut elems = Vec::new();
            loop {
                if *self.data.get(self.pos).ok_or(INVALID_CBOR_VIOLATION)? == BREAK {
                    self.pos += 1;
                    break;
                }
                self.item(&mut elems, depth + 1)?;
            }
            emit_head(out, 4, count_items(&elems));
            out.extend_from_slice(&elems);
            Ok(())
        }

        fn map(
            &mut self,
            out: &mut Vec<u8>,
            ai: u8,
            arg: u64,
            depth: usize,
        ) -> Result<(), ShapeViolation> {
            let mut pairs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
            if ai != 31 {
                let n = usize_arg(arg)?;
                for _ in 0..n {
                    let mut k = Vec::new();
                    self.item(&mut k, depth + 1)?;
                    let mut v = Vec::new();
                    self.item(&mut v, depth + 1)?;
                    pairs.push((k, v));
                }
            } else {
                loop {
                    if *self.data.get(self.pos).ok_or(INVALID_CBOR_VIOLATION)? == BREAK {
                        self.pos += 1;
                        break;
                    }
                    let mut k = Vec::new();
                    self.item(&mut k, depth + 1)?;
                    let mut v = Vec::new();
                    self.item(&mut v, depth + 1)?;
                    pairs.push((k, v));
                }
            }
            // §4.2.1: sort by bytewise-lexicographic order of encoded keys.
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
            // Reject duplicate keys.
            for w in pairs.windows(2) {
                if w[0].0 == w[1].0 {
                    return Err(INVALID_CBOR_VIOLATION);
                }
            }
            emit_head(out, 5, pairs.len() as u64);
            for (k, v) in pairs {
                out.extend_from_slice(&k);
                out.extend_from_slice(&v);
            }
            Ok(())
        }

        fn simple_or_float(
            &mut self,
            out: &mut Vec<u8>,
            ai: u8,
            arg: u64,
        ) -> Result<(), ShapeViolation> {
            match ai {
                // simple value (false/true/null/undefined/simple 0..=23)
                0..=23 => {
                    out.push(0xe0 | (arg as u8));
                    Ok(())
                }
                // 1-byte simple value (32..=255; 0..=31 are not well-formed here)
                24 => {
                    if arg < 32 {
                        return Err(INVALID_CBOR_VIOLATION);
                    }
                    out.push(0xf8);
                    out.push(arg as u8);
                    Ok(())
                }
                25 => {
                    // float16 → f64 → canonical
                    emit_canonical_float(out, half_to_f64(arg as u16));
                    Ok(())
                }
                26 => {
                    emit_canonical_float(out, f32::from_bits(arg as u32) as f64);
                    Ok(())
                }
                27 => {
                    emit_canonical_float(out, f64::from_bits(arg));
                    Ok(())
                }
                _ => Err(INVALID_CBOR_VIOLATION), // 28,29,30,31(break)
            }
        }
    }

    fn usize_arg(arg: u64) -> Result<usize, ShapeViolation> {
        usize::try_from(arg).map_err(|_| INVALID_CBOR_VIOLATION)
    }

    /// Emit a CBOR head for `major` with the shortest argument encoding
    /// (RFC 8949 §4.1 preferred serialization).
    fn emit_head(out: &mut Vec<u8>, major: u8, arg: u64) {
        let m = major << 5;
        if arg < 24 {
            out.push(m | (arg as u8));
        } else if arg <= u64::from(u8::MAX) {
            out.push(m | 24);
            out.push(arg as u8);
        } else if arg <= u64::from(u16::MAX) {
            out.push(m | 25);
            out.extend_from_slice(&(arg as u16).to_be_bytes());
        } else if arg <= u64::from(u32::MAX) {
            out.push(m | 26);
            out.extend_from_slice(&(arg as u32).to_be_bytes());
        } else {
            out.push(m | 27);
            out.extend_from_slice(&arg.to_be_bytes());
        }
    }

    /// Count the number of top-level canonical CBOR items in `buf` (used
    /// only for indefinite-length array/map element counting). `buf` is
    /// always well-formed canonical output we just produced.
    fn count_items(buf: &[u8]) -> u64 {
        let mut p = Walker { data: buf, pos: 0 };
        let mut n = 0u64;
        while p.pos < buf.len() {
            p.skip();
            n += 1;
        }
        n
    }

    struct Walker<'a> {
        data: &'a [u8],
        pos: usize,
    }
    impl Walker<'_> {
        fn b(&mut self) -> u8 {
            let v = self.data[self.pos];
            self.pos += 1;
            v
        }
        fn arg(&mut self, ai: u8) -> u64 {
            match ai {
                0..=23 => u64::from(ai),
                24 => u64::from(self.b()),
                25 => {
                    let v = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
                    self.pos += 2;
                    u64::from(v)
                }
                26 => {
                    let mut a = [0u8; 4];
                    a.copy_from_slice(&self.data[self.pos..self.pos + 4]);
                    self.pos += 4;
                    u64::from(u32::from_be_bytes(a))
                }
                27 => {
                    let mut a = [0u8; 8];
                    a.copy_from_slice(&self.data[self.pos..self.pos + 8]);
                    self.pos += 8;
                    u64::from_be_bytes(a)
                }
                _ => 0,
            }
        }
        fn skip(&mut self) {
            let ib = self.b();
            let major = ib >> 5;
            let ai = ib & 0x1f;
            let arg = self.arg(ai);
            match major {
                0 | 1 => {}
                2 | 3 => self.pos += arg as usize,
                4 => {
                    for _ in 0..arg {
                        self.skip();
                    }
                }
                5 => {
                    for _ in 0..arg {
                        self.skip();
                        self.skip();
                    }
                }
                6 => self.skip(),
                7 => {
                    // canonical output: simple inline (handled by arg), 1-byte
                    // simple already consumed via ai==24, floats via ai 25/26/27
                    // already consumed by `arg`.
                }
                _ => {}
            }
        }
    }

    // ─── IEEE-754 half-precision helpers (RFC 8949 §4.2.2 shortest float) ──

    /// Decode an IEEE-754 binary16 bit pattern to the bit pattern of the
    /// exactly-equal `f32` (every binary16 value is representable in
    /// binary32). Pure integer arithmetic — `no_std` / no-libm safe.
    fn half_to_f32_bits(h: u16) -> u32 {
        let sign = (u32::from(h) & 0x8000) << 16;
        let exp = (h >> 10) & 0x1f;
        let mant = u32::from(h & 0x03ff);
        if exp == 0 {
            if mant == 0 {
                return sign; // ±0
            }
            // Subnormal: normalize into a binary32 normal number.
            let mut e: i32 = -1;
            let mut m = mant;
            loop {
                e += 1;
                m <<= 1;
                if m & 0x0400 != 0 {
                    break;
                }
            }
            let mant32 = (m & 0x03ff) << 13;
            let exp32 = (127 - 15 - e) as u32;
            return sign | (exp32 << 23) | mant32;
        }
        if exp == 0x1f {
            return sign | 0x7f80_0000 | (mant << 13); // Inf / NaN
        }
        let exp32 = (i32::from(exp) - 15 + 127) as u32;
        sign | (exp32 << 23) | (mant << 13)
    }

    /// Decode an IEEE-754 binary16 bit pattern to `f64` (exact, via the
    /// exactly-equal binary32). The `as f64` widening is a core float cast,
    /// not a libm intrinsic.
    fn half_to_f64(h: u16) -> f64 {
        f64::from(f32::from_bits(half_to_f32_bits(h)))
    }

    /// Round-to-nearest-even encode `f32` to a binary16 bit pattern. The
    /// caller verifies the round-trip, so an imperfect edge case only costs
    /// shortness, never correctness.
    fn f32_to_half_bits(f: f32) -> u16 {
        let x = f.to_bits();
        let sign = ((x >> 16) & 0x8000) as u16;
        let exp = ((x >> 23) & 0xff) as i32;
        let mant = x & 0x007f_ffff;
        if exp == 0xff {
            return if mant == 0 {
                sign | 0x7c00
            } else {
                sign | 0x7e00
            };
        }
        let e = exp - 127 + 15;
        if e >= 0x1f {
            return sign | 0x7c00; // overflow → ±Inf
        }
        if e <= 0 {
            if e < -10 {
                return sign; // underflow → ±0
            }
            let mant_full = mant | 0x0080_0000;
            let shift = (14 - e) as u32;
            let half_mant = (mant_full >> shift) as u16;
            let round_rem = mant_full & ((1 << shift) - 1);
            let halfway = 1u32 << (shift - 1);
            let mut bits = sign | half_mant;
            if round_rem > halfway || (round_rem == halfway && (half_mant & 1) == 1) {
                bits += 1;
            }
            return bits;
        }
        let half_mant = (mant >> 13) as u16;
        let round_rem = mant & 0x1fff;
        let mut bits = sign | ((e as u16) << 10) | half_mant;
        if round_rem > 0x1000 || (round_rem == 0x1000 && (half_mant & 1) == 1) {
            bits += 1; // may carry into the exponent — round-trip check guards
        }
        bits
    }

    /// Emit `v` as the shortest of half / single / double precision that
    /// round-trips it exactly (§4.2.2); every NaN collapses to the
    /// canonical half-precision quiet NaN `0xf9 0x7e 0x00`.
    fn emit_canonical_float(out: &mut Vec<u8>, v: f64) {
        if v.is_nan() {
            out.extend_from_slice(&[0xf9, 0x7e, 0x00]);
            return;
        }
        let single = v as f32;
        if f64::from(single) == v || (v.is_infinite() && single.is_infinite()) {
            let hb = f32_to_half_bits(single);
            if half_to_f64(hb).to_bits() == v.to_bits() {
                out.push(0xf9);
                out.extend_from_slice(&hb.to_be_bytes());
                return;
            }
            out.push(0xfa);
            out.extend_from_slice(&single.to_be_bytes());
            return;
        }
        out.push(0xfb);
        out.extend_from_slice(&v.to_bits().to_be_bytes());
    }
}
