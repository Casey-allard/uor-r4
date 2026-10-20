//! GGUF v3 typed input (ADR-023 amended by ADR-060).
//!
//! The GGUF spec defines no canonical form; this realization defines one
//! — a **flat Merkle skeleton**. Two GGUF files that decode to the same
//! logical content canonicalize to byte-identical skeletons. Every
//! variable-length leaf (a string, an array payload, a tensor's data
//! region) is replaced by its streamed SHA-256 digest, so the skeleton's
//! size grows only with the KV / tensor **counts** (never with model
//! size), while still binding every weight byte into the κ-label.
//!
//! ```text
//! LE_u32(GGUF_MAGIC)
//! LE_u32(GGUF_VERSION_REQUIRED)
//! LE_u64(tensor_count)
//! LE_u64(kv_count)
//! LE_u64(canonical_alignment)
//! ── metadata KVs, sorted by key bytes ──
//!   for kv: sha256(key) || LE_u32(type_tag) || canonical_value(kv)
//!     scalar  → the value's natural little-endian bytes
//!     string  → LE_u64(len) || sha256(utf8 bytes)
//!     array   → LE_u32(elem_type) || LE_u64(len) || sha256(wire payload)
//! ── tensor info, sorted by name bytes ──
//!   for t: sha256(name) || LE_u32(n_dims) || (LE_u64(dim) × n_dims)
//!       || LE_u32(ggml_type_id) || LE_u64(recomputed_offset)
//!       || sha256(tensor data bytes)        ← streamed; binds the weights
//! ```
//!
//! `recomputed_offset` is the cumulative aligned byte position in
//! sorted-tensor order (NOT the input's stored offset), so two inputs
//! whose tensor-data sections are laid out in different orders
//! canonicalize identically.
//!
//! Under ADR-060 the **full skeleton** flows through the pipeline as a
//! [`TermValue::Borrowed`] carrier and ψ₉ folds it through the σ-axis —
//! there is no two-level commitment, no carrier ceiling, and no count /
//! width cap. Tensor data and large string / array payloads are streamed
//! through [`prism::crypto::Sha256Hasher`] with bounded resident memory,
//! so arbitrarily large weights bind into the κ-label.
//!
//! [`GgufValue`] (the owned parsed value, `alloc`-gated) holds the
//! skeleton; [`GgufCarrier`] is the borrowed model-input handle the
//! pipeline binds.

use prism::crypto::Sha256Hasher;
use prism::operation::TermValue;
use prism::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields, ShapeViolation,
    ViolationKind,
};
use prism::vocabulary::Hasher;

use crate::gguf::dtype::GgmlType;
use crate::gguf::shapes::bounds::{
    GGUF_DEFAULT_ALIGNMENT, GGUF_HEADER_BYTES, GGUF_MAGIC, GGUF_MAX_DIMS,
    GGUF_METADATA_ARRAY_DEPTH_MAX, GGUF_VERSION_REQUIRED,
};

// ─── ShapeViolation IRIs ────────────────────────────────────────────────

macro_rules! violation {
    ($name:ident, $constraint:literal, $property:literal, $kind:expr) => {
        const $name: ShapeViolation = ShapeViolation {
            shape_iri: "https://uor.foundation/addr/GgufValue",
            constraint_iri: concat!("https://uor.foundation/addr/GgufValue/", $constraint),
            property_iri: concat!("https://uor.foundation/addr/GgufValue/", $property),
            expected_range: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
            min_count: 0,
            max_count: 1,
            kind: $kind,
        };
    };
}

violation!(
    INVALID_MAGIC,
    "validMagic",
    "magic",
    ViolationKind::ValueCheck
);
violation!(
    UNSUPPORTED_VERSION,
    "supportedVersion",
    "version",
    ViolationKind::ValueCheck
);
violation!(
    TRUNCATED,
    "notTruncated",
    "byteSpan",
    ViolationKind::ValueCheck
);
violation!(
    DIMS_EXCEEDED,
    "tensorRankBound",
    "nDims",
    ViolationKind::CardinalityViolation
);
violation!(
    ARRAY_DEPTH,
    "arrayDepthBound",
    "arrayDepth",
    ViolationKind::CardinalityViolation
);
violation!(
    INVALID_ALIGNMENT,
    "validAlignment",
    "alignment",
    ViolationKind::ValueCheck
);
violation!(
    UNKNOWN_TENSOR_TYPE,
    "knownTensorType",
    "tensorType",
    ViolationKind::ValueCheck
);
violation!(
    OVERFLOW,
    "noOverflow",
    "byteCount",
    ViolationKind::ValueCheck
);

// ─── GGUF metadata value type tags (gguf.md) ─────────────────────────────

const T_UINT8: u32 = 0;
const T_INT8: u32 = 1;
const T_UINT16: u32 = 2;
const T_INT16: u32 = 3;
const T_UINT32: u32 = 4;
const T_INT32: u32 = 5;
const T_FLOAT32: u32 = 6;
const T_BOOL: u32 = 7;
const T_STRING: u32 = 8;
const T_ARRAY: u32 = 9;
const T_UINT64: u32 = 10;
const T_INT64: u32 = 11;
const T_FLOAT64: u32 = 12;

/// Fixed wire width of a scalar metadata value type, or `None` for the
/// variable-length `STRING`/`ARRAY` types.
const fn scalar_width(type_tag: u32) -> Option<usize> {
    Some(match type_tag {
        T_UINT8 | T_INT8 | T_BOOL => 1,
        T_UINT16 | T_INT16 => 2,
        T_UINT32 | T_INT32 | T_FLOAT32 => 4,
        T_UINT64 | T_INT64 | T_FLOAT64 => 8,
        _ => return None,
    })
}

// ─── Little-endian readers over a borrowed cursor ────────────────────────

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], ShapeViolation> {
        let end = self.pos.checked_add(n).ok_or(TRUNCATED)?;
        if end > self.buf.len() {
            return Err(TRUNCATED);
        }
        let s = &self.buf[self.pos..end];
        self.pos = end;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32, ShapeViolation> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u64(&mut self) -> Result<u64, ShapeViolation> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
}

#[inline]
fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256Hasher::initial().fold_bytes(bytes).finalize()
}

#[inline]
const fn align_up(offset: u64, alignment: u64) -> u64 {
    let rem = offset % alignment;
    if rem == 0 {
        offset
    } else {
        offset + (alignment - rem)
    }
}

// ─── GgufCarrier — the borrowed model-input handle (no_alloc) ───────────

/// Borrowed canonical-skeleton input handle (ADR-060 borrowed carrier). A
/// thin, `Copy` borrow of the skeleton bytes produced by [`canonicalize`];
/// `as_binding_value` returns the `Borrowed` carrier zero-copy.
#[derive(Clone, Copy, Debug)]
pub struct GgufCarrier<'a>(&'a [u8]);

impl<'a> GgufCarrier<'a> {
    /// Wrap a canonical-skeleton byte slice as a model input handle.
    #[must_use]
    pub fn new(skeleton: &'a [u8]) -> Self {
        Self(skeleton)
    }

    /// Borrow the canonical-skeleton bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &'a [u8] {
        self.0
    }
}

impl ConstrainedTypeShape for GgufCarrier<'_> {
    const IRI: &'static str = "https://uor.foundation/addr/GgufValue";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl prism::uor_foundation::pipeline::__sdk_seal::Sealed for GgufCarrier<'_> {}

impl<'a> IntoBindingValue<'a> for GgufCarrier<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::borrowed(self.0)
    }
}

impl PartitionProductFields for GgufCarrier<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

// ═════════════════════════════════════════════════════════════════════
// alloc-gated parser + owned value
// ═════════════════════════════════════════════════════════════════════

#[cfg(feature = "alloc")]
pub use alloc_impl::{canonicalize, GgufValue};

#[cfg(feature = "alloc")]
mod alloc_impl {
    use super::*;
    use alloc::vec::Vec;

    /// A parsed, canonicalized GGUF v3 file. The stored bytes are the
    /// flat canonical skeleton (see [module docs](super)). **`alloc`-gated**
    /// — the pipeline binds the borrowed [`GgufCarrier`].
    #[derive(Clone, PartialEq, Eq)]
    pub struct GgufValue {
        bytes: Vec<u8>,
    }

    impl core::fmt::Debug for GgufValue {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("GgufValue")
                .field("canonical_len", &self.bytes.len())
                .finish_non_exhaustive()
        }
    }

    struct KvEntry {
        key_off: usize,
        key_len: usize,
        type_tag: u32,
        val_off: usize,
        val_span: usize,
    }

    struct TensorEntry {
        name_off: usize,
        name_len: usize,
        n_dims: u32,
        dims: [u64; GGUF_MAX_DIMS],
        ggml_type: GgmlType,
        stored_offset: u64,
        data_bytes: u64,
    }

    impl GgufValue {
        /// Borrow the canonical-skeleton bytes.
        #[must_use]
        pub fn canonical_bytes(&self) -> &[u8] {
            &self.bytes
        }

        /// Parse a GGUF v3 input slice into a canonicalized skeleton.
        ///
        /// # Errors
        ///
        /// A [`ShapeViolation`] whose `constraint_iri` names the violated
        /// invariant (bad magic, unsupported version, truncation, an
        /// over-rank tensor, over-deep array nesting, invalid alignment,
        /// an unknown tensor type, or arithmetic overflow).
        pub fn parse(raw: &[u8]) -> Result<Self, ShapeViolation> {
            let mut cur = Cursor::new(raw);

            // ── Header ──
            if cur.u32()? != GGUF_MAGIC {
                return Err(INVALID_MAGIC);
            }
            if cur.u32()? != GGUF_VERSION_REQUIRED {
                return Err(UNSUPPORTED_VERSION);
            }
            let tensor_count = cur.u64()?;
            let kv_count = cur.u64()?;
            debug_assert_eq!(cur.pos, GGUF_HEADER_BYTES);

            // ── Metadata KV section ──
            let mut kvs: Vec<KvEntry> = Vec::new();
            let mut alignment = GGUF_DEFAULT_ALIGNMENT;
            for _ in 0..kv_count {
                let key_len = cur.u64()? as usize;
                let key_off = cur.pos;
                let key = cur.take(key_len)?;
                let type_tag = cur.u32()?;
                let val_off = cur.pos;
                let val_span = measure_value(&mut cur, type_tag, 0)?;

                if key == b"general.alignment" && type_tag == T_UINT32 {
                    let a = u32::from_le_bytes([
                        raw[val_off],
                        raw[val_off + 1],
                        raw[val_off + 2],
                        raw[val_off + 3],
                    ]) as u64;
                    if a < 8 || !a.is_power_of_two() {
                        return Err(INVALID_ALIGNMENT);
                    }
                    alignment = a;
                }

                kvs.push(KvEntry {
                    key_off,
                    key_len,
                    type_tag,
                    val_off,
                    val_span,
                });
            }

            // ── Tensor info section ──
            let mut tensors: Vec<TensorEntry> = Vec::new();
            for _ in 0..tensor_count {
                let name_len = cur.u64()? as usize;
                let name_off = cur.pos;
                cur.take(name_len)?;
                let n_dims = cur.u32()?;
                if n_dims as usize > GGUF_MAX_DIMS {
                    return Err(DIMS_EXCEEDED);
                }
                let mut dims = [0u64; GGUF_MAX_DIMS];
                let mut n_elements: u64 = 1;
                for d in dims.iter_mut().take(n_dims as usize) {
                    *d = cur.u64()?;
                    n_elements = n_elements.checked_mul(*d).ok_or(OVERFLOW)?;
                }
                let type_id = cur.u32()?;
                let ggml_type = GgmlType::from_u32(type_id).ok_or(UNKNOWN_TENSOR_TYPE)?;
                let stored_offset = cur.u64()?;
                let data_bytes = ggml_type
                    .tensor_data_bytes(n_elements)
                    .ok_or(UNKNOWN_TENSOR_TYPE)?;
                tensors.push(TensorEntry {
                    name_off,
                    name_len,
                    n_dims,
                    dims,
                    ggml_type,
                    stored_offset,
                    data_bytes,
                });
            }

            // Tensor-data section begins at the next alignment boundary
            // past the end of the tensor-info section.
            let data_section_start = align_up(cur.pos as u64, alignment);

            // ── Sort orders (lexicographic on raw UTF-8 bytes) ──
            let mut kv_order: Vec<usize> = (0..kvs.len()).collect();
            kv_order.sort_by(|&a, &b| {
                raw[kvs[a].key_off..kvs[a].key_off + kvs[a].key_len]
                    .cmp(&raw[kvs[b].key_off..kvs[b].key_off + kvs[b].key_len])
            });
            let mut t_order: Vec<usize> = (0..tensors.len()).collect();
            t_order.sort_by(|&a, &b| {
                raw[tensors[a].name_off..tensors[a].name_off + tensors[a].name_len]
                    .cmp(&raw[tensors[b].name_off..tensors[b].name_off + tensors[b].name_len])
            });

            // ── Emit the flat canonical skeleton ──
            let mut out: Vec<u8> = Vec::new();
            out.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
            out.extend_from_slice(&GGUF_VERSION_REQUIRED.to_le_bytes());
            out.extend_from_slice(&tensor_count.to_le_bytes());
            out.extend_from_slice(&kv_count.to_le_bytes());
            out.extend_from_slice(&alignment.to_le_bytes());

            for &idx in &kv_order {
                let kv = &kvs[idx];
                let key = &raw[kv.key_off..kv.key_off + kv.key_len];
                out.extend_from_slice(&sha256(key));
                out.extend_from_slice(&kv.type_tag.to_le_bytes());
                emit_canonical_value(&mut out, raw, kv);
            }

            let mut canonical_offset: u64 = 0;
            for &idx in &t_order {
                let t = &tensors[idx];
                let name = &raw[t.name_off..t.name_off + t.name_len];
                out.extend_from_slice(&sha256(name));
                out.extend_from_slice(&t.n_dims.to_le_bytes());
                for d in t.dims.iter().take(t.n_dims as usize) {
                    out.extend_from_slice(&d.to_le_bytes());
                }
                out.extend_from_slice(&t.ggml_type.id().to_le_bytes());
                out.extend_from_slice(&canonical_offset.to_le_bytes());

                // Stream the tensor's data region through SHA-256.
                let start = data_section_start
                    .checked_add(t.stored_offset)
                    .ok_or(TRUNCATED)? as usize;
                let end = start.checked_add(t.data_bytes as usize).ok_or(TRUNCATED)?;
                if end > raw.len() {
                    return Err(TRUNCATED);
                }
                out.extend_from_slice(&sha256(&raw[start..end]));

                canonical_offset = align_up(
                    canonical_offset.checked_add(t.data_bytes).ok_or(OVERFLOW)?,
                    alignment,
                );
            }

            Ok(Self { bytes: out })
        }
    }

    /// Measure (and bounds-check) the wire span of a metadata value,
    /// advancing the cursor past it. Recurses into ARRAY payloads,
    /// guarding the native stack with [`GGUF_METADATA_ARRAY_DEPTH_MAX`].
    fn measure_value(
        cur: &mut Cursor<'_>,
        type_tag: u32,
        depth: usize,
    ) -> Result<usize, ShapeViolation> {
        let start = cur.pos;
        if let Some(w) = scalar_width(type_tag) {
            cur.take(w)?;
        } else if type_tag == T_STRING {
            let n = cur.u64()? as usize;
            cur.take(n)?;
        } else if type_tag == T_ARRAY {
            if depth >= GGUF_METADATA_ARRAY_DEPTH_MAX {
                return Err(ARRAY_DEPTH);
            }
            let elem_type = cur.u32()?;
            let len = cur.u64()? as usize;
            for _ in 0..len {
                measure_value(cur, elem_type, depth + 1)?;
            }
        } else {
            return Err(TRUNCATED); // unknown type tag
        }
        Ok(cur.pos - start)
    }

    /// Emit the canonical representation of a metadata value into `out`:
    /// scalars inline (natural little-endian bytes); STRING / ARRAY as a
    /// length-tagged header plus a streamed digest of the wire payload.
    fn emit_canonical_value(out: &mut Vec<u8>, raw: &[u8], kv: &KvEntry) {
        let payload = &raw[kv.val_off..kv.val_off + kv.val_span];
        if scalar_width(kv.type_tag).is_some() {
            out.extend_from_slice(payload);
        } else if kv.type_tag == T_STRING {
            let len = u64::from_le_bytes(payload[..8].try_into().unwrap_or([0; 8]));
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&sha256(&payload[8..]));
        } else if kv.type_tag == T_ARRAY {
            let elem_type = u32::from_le_bytes(payload[..4].try_into().unwrap_or([0; 4]));
            let len = u64::from_le_bytes(payload[4..12].try_into().unwrap_or([0; 8]));
            out.extend_from_slice(&elem_type.to_le_bytes());
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&sha256(&payload[12..]));
        }
    }

    /// Canonical skeleton as an owned `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// Surfaces the [`ShapeViolation`] [`GgufValue::parse`] would raise.
    pub fn canonicalize(raw: &[u8]) -> Result<Vec<u8>, ShapeViolation> {
        Ok(GgufValue::parse(raw)?.bytes)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn minimal_gguf() -> Vec<u8> {
            // magic, version, tensor_count=0, kv_count=0.
            let mut v = Vec::new();
            v.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
            v.extend_from_slice(&GGUF_VERSION_REQUIRED.to_le_bytes());
            v.extend_from_slice(&0u64.to_le_bytes());
            v.extend_from_slice(&0u64.to_le_bytes());
            v
        }

        #[test]
        fn parses_minimal_header() {
            let canon = canonicalize(&minimal_gguf()).expect("valid");
            // header: magic(4)+version(4)+tcount(8)+kvcount(8)+align(8) = 32.
            assert_eq!(canon.len(), 32);
        }

        #[test]
        fn rejects_bad_magic() {
            let mut v = minimal_gguf();
            v[0] ^= 0xFF;
            let err = GgufValue::parse(&v).expect_err("bad magic");
            assert_eq!(err.constraint_iri, INVALID_MAGIC.constraint_iri);
        }

        #[test]
        fn rejects_unsupported_version() {
            let mut v = minimal_gguf();
            v[4] = 2;
            let err = GgufValue::parse(&v).expect_err("v2");
            assert_eq!(err.constraint_iri, UNSUPPORTED_VERSION.constraint_iri);
        }
    }
}
