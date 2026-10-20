//! `RingElement` — the ring-element typed input handle (UOR-Framework
//! Amendment 43 §2 `Element::canonical_bytes`).
//!
//! Runtime bytes are
//!
//! ```text
//! canonical_bytes(e) := [witt_level: u8] || [coefficient: u8; witt_level + 1]
//! ```
//!
//! which is already the canonical form, so under ADR-060 the handle's
//! `as_binding_value` returns the tagged bytes directly as an `Inline`
//! [`TermValue`] carrier (the form is ≤ 5 bytes, well within the
//! foundation-derived inline width). ψ₉ folds that carrier through the
//! σ-axis to mint the κ-label.

use prism::operation::TermValue;
use prism::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields, ShapeViolation,
    ViolationKind,
};

use crate::ring::shapes::bounds::{MAX_WITT_LEVEL, RING_VALUE_MAX_BYTES};

// ─── ShapeViolation IRIs ────────────────────────────────────────────────

const INVALID_RING_VIOLATION: ShapeViolation = ShapeViolation {
    shape_iri: "https://uor.foundation/addr/RingElement",
    constraint_iri: "https://uor.foundation/addr/RingElement/validCanonicalBytes",
    property_iri: "https://uor.foundation/addr/inputBytes",
    expected_range: "https://uor.foundation/addr/ValidRingElementBytes",
    min_count: 0,
    max_count: 1,
    kind: ViolationKind::ValueCheck,
};

const WITT_LEVEL_VIOLATION: ShapeViolation = ShapeViolation {
    shape_iri: "https://uor.foundation/addr/RingElement",
    constraint_iri: "https://uor.foundation/addr/RingElement/wittLevelBound",
    property_iri: "https://uor.foundation/addr/RingElement/wittLevel",
    expected_range: "http://www.w3.org/2001/XMLSchema#unsignedByte",
    min_count: 0,
    max_count: MAX_WITT_LEVEL as u32,
    kind: ViolationKind::CardinalityViolation,
};

const TOTAL_WIDTH_VIOLATION: ShapeViolation = ShapeViolation {
    shape_iri: "https://uor.foundation/addr/RingElement",
    constraint_iri: "https://uor.foundation/addr/RingElement/serializedWidth",
    property_iri: "https://uor.foundation/addr/RingElement/totalByteCount",
    expected_range: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
    min_count: 0,
    max_count: RING_VALUE_MAX_BYTES as u32,
    kind: ViolationKind::CardinalityViolation,
};

// ─── RingElement — the typed input handle ────────────────────────────────

/// Typed ring-element input handle. Runtime bytes follow Amendment 43
/// §2's canonical-bytes layout, stored in a fixed-size stack buffer.
#[derive(Clone)]
pub struct RingElement {
    pub(crate) bytes: [u8; RING_VALUE_MAX_BYTES],
    pub(crate) len: u16,
}

impl core::fmt::Debug for RingElement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RingElement")
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

impl PartialEq for RingElement {
    fn eq(&self, other: &Self) -> bool {
        self.tagged_bytes() == other.tagged_bytes()
    }
}
impl Eq for RingElement {}

impl RingElement {
    /// Construct a `RingElement` from explicit Witt level + coefficient.
    pub fn from_components(witt_level: u8, coefficient: u64) -> Result<Self, ShapeViolation> {
        if witt_level > MAX_WITT_LEVEL {
            return Err(WITT_LEVEL_VIOLATION);
        }
        let coefficient_bytes = (witt_level + 1) as usize;
        let total = 1 + coefficient_bytes;
        let mut me = Self {
            bytes: [0u8; RING_VALUE_MAX_BYTES],
            len: total as u16,
        };
        me.bytes[0] = witt_level;
        let le = coefficient.to_le_bytes();
        me.bytes[1..1 + coefficient_bytes].copy_from_slice(&le[..coefficient_bytes]);
        Ok(me)
    }

    /// Parse raw canonical-bytes into a typed `RingElement`.
    pub fn parse(raw: &[u8]) -> Result<Self, ShapeViolation> {
        if raw.is_empty() {
            return Err(INVALID_RING_VIOLATION);
        }
        if raw.len() > RING_VALUE_MAX_BYTES {
            return Err(TOTAL_WIDTH_VIOLATION);
        }
        let witt_level = raw[0];
        if witt_level > MAX_WITT_LEVEL {
            return Err(WITT_LEVEL_VIOLATION);
        }
        let expected_len = 1 + (witt_level as usize + 1);
        if raw.len() != expected_len {
            return Err(INVALID_RING_VIOLATION);
        }
        let mut me = Self {
            bytes: [0u8; RING_VALUE_MAX_BYTES],
            len: raw.len() as u16,
        };
        me.bytes[..raw.len()].copy_from_slice(raw);
        Ok(me)
    }

    /// Borrow the canonical-bytes byte sequence.
    #[must_use]
    pub fn tagged_bytes(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }

    /// The element's Witt level (first byte of the canonical layout).
    #[must_use]
    pub fn witt_level(&self) -> u8 {
        self.bytes[0]
    }
}

// ─── ConstrainedTypeShape + IntoBindingValue + PartitionProductFields ─────

impl ConstrainedTypeShape for RingElement {
    const IRI: &'static str = "https://uor.foundation/addr/RingElement";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl prism::uor_foundation::pipeline::__sdk_seal::Sealed for RingElement {}

impl<'a> IntoBindingValue<'a> for RingElement {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        // Amendment 43 §2 canonical bytes are the canonical form; emit them
        // as an `Inline` carrier (owned, valid for any `'a`).
        TermValue::inline_from_slice(self.tagged_bytes())
    }
}

impl PartitionProductFields for RingElement {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_components_round_trip() {
        let e = RingElement::from_components(2, 0x0001_0203).expect("valid");
        assert_eq!(e.bytes[0], 2);
        assert_eq!(&e.bytes[1..4], &[0x03, 0x02, 0x01]);
    }

    #[test]
    fn parse_matches_construction() {
        let constructed = RingElement::from_components(1, 0x0102).expect("valid");
        let parsed = RingElement::parse(&[1, 0x02, 0x01]).expect("valid");
        assert_eq!(constructed, parsed);
    }

    #[test]
    fn rejects_overflow_witt_level() {
        let err = RingElement::from_components(MAX_WITT_LEVEL + 1, 0).expect_err("must reject");
        assert_eq!(err.constraint_iri, WITT_LEVEL_VIOLATION.constraint_iri);
        let err = RingElement::parse(&[MAX_WITT_LEVEL + 1, 0]).expect_err("must reject");
        assert_eq!(err.constraint_iri, WITT_LEVEL_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_truncated_bytes() {
        let err = RingElement::parse(&[2, 0, 0]).expect_err("must reject");
        assert_eq!(err.constraint_iri, INVALID_RING_VIOLATION.constraint_iri);
    }
}
