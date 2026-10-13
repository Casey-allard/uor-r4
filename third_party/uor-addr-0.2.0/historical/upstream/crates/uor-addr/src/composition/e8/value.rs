//! CS-E8 typed-input carrier — wraps the canonicalize discipline's
//! output as a `Borrowed` TermValue flowing through the ψ-pipeline.

#![cfg(feature = "alloc")]

use prism::operation::TermValue;
use prism::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields,
};

/// CS-E8's typed-input carrier.
#[derive(Clone, Copy, Debug)]
pub struct E8Carrier<'a>(&'a [u8]);

impl<'a> E8Carrier<'a> {
    /// Wrap canonical-form bytes as a model input handle.
    #[must_use]
    pub fn new(canonical_bytes: &'a [u8]) -> Self {
        Self(canonical_bytes)
    }

    /// Borrow the canonical-form bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &'a [u8] {
        self.0
    }
}

impl ConstrainedTypeShape for E8Carrier<'_> {
    const IRI: &'static str = "https://uor.foundation/addr/composition/E8EmbeddingCarrier";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl prism::uor_foundation::pipeline::__sdk_seal::Sealed for E8Carrier<'_> {}

impl<'a> IntoBindingValue<'a> for E8Carrier<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::borrowed(self.0)
    }
}

impl PartitionProductFields for E8Carrier<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}
