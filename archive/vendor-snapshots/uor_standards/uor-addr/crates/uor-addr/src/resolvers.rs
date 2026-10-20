//! `AddressResolverTuple` — the single eight-resolver tower shared by
//! every UOR-ADDR realization (ADR-036).
//!
//! Under ADR-060 the realization's canonical form flows through the
//! pipeline as a source-polymorphic [`TermValue`] carrier produced by the
//! input's `as_binding_value` (see [`crate::common::AddressInput`]). The
//! ψ-tower is therefore **format-independent**: ψ₁ (nerve) … ψ₈ thread the
//! carrier through unchanged, and ψ₉ (k-invariants) folds the carrier
//! through the σ-axis `H` chunk-by-chunk — never materializing it — and
//! emits the `H::LABEL_BYTES`-wide ASCII κ-label `<algorithm>:<hex>` as an
//! `Inline` carrier (see [`crate::hash`] for the admissible axes).
//!
//! ## Why this is hand-written (not `resolver!`-emitted)
//!
//! foundation 0.5.2 generalized the resolver traits to an unbounded `H`
//! (so a 64-byte `Sha512Hasher` σ-axis composes), but the SDK 0.5.2
//! `resolver!` macro still emits `H: Hasher` (= `Hasher<32>`) bounds on the
//! generated tuple impls — which would exclude `Sha512Hasher`. The tower is
//! therefore written out by hand, bound on the fingerprint-width-erased
//! [`AddrHash`] façade ([`crate::hash`]) so the single tuple carries every
//! admissible axis (32- and 64-byte) without a free `FP_MAX` parameter.

use core::marker::PhantomData;

use prism::operation::TermValue;
use prism::pipeline::{
    ChainComplexResolver, CochainComplexResolver, CohomologyGroupResolver, HasChainComplexResolver,
    HasCochainComplexResolver, HasCohomologyGroupResolver, HasHomologyGroupResolver,
    HasHomotopyGroupResolver, HasKInvariantResolver, HasNerveResolver, HasPostnikovResolver,
    HomologyGroupResolver, HomotopyGroupResolver, KInvariantResolver, NerveResolver,
    PostnikovResolver, ResolverCategory, ResolverTuple, ShapeViolation,
};
use prism::uor_foundation::pipeline::__sdk_seal::Sealed;
use prism::uor_foundation::pipeline::shape_iri_registry::EmptyShapeRegistry;

use crate::hash::{AddrHash, MAX_LABEL_BYTES};

const HEX_LOWER: [u8; 16] = *b"0123456789abcdef";

/// Fold the carrier through the σ-axis `H` (streaming, bounded memory) and
/// format the `H::LABEL_BYTES`-wide κ-label `<prefix>:<hex>` into an
/// `Inline` carrier. The stack scratch is sized to the widest admissible
/// axis ([`MAX_LABEL_BYTES`]); only the active axis's bytes are emitted.
fn kappa_label_carrier<const N: usize, H: AddrHash>(
    input: &TermValue<'_, N>,
) -> TermValue<'static, N> {
    let digest = H::digest_carrier(input);
    let prefix = H::LABEL_PREFIX.as_bytes();
    let p = prefix.len();
    let mut out = [0u8; MAX_LABEL_BYTES];
    out[..p].copy_from_slice(prefix);
    out[p] = b':';
    for (i, byte) in digest.iter().enumerate().take(H::OUTPUT_BYTES) {
        out[p + 1 + 2 * i] = HEX_LOWER[(byte >> 4) as usize];
        out[p + 1 + 2 * i + 1] = HEX_LOWER[(byte & 0x0F) as usize];
    }
    TermValue::inline_from_slice(&out[..H::LABEL_BYTES])
}

// ── The eight per-category resolver structs. ψ₁–ψ₈ are pass-throughs;
//    only ψ₉ (k-invariants) folds the σ-axis. All are generic over an
//    unbounded `H` (the foundation resolver traits no longer bind it). ──

macro_rules! address_resolver {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name<H>(PhantomData<H>);
        impl<H> Sealed for $name<H> {}
        impl<H> Default for $name<H> {
            #[inline]
            fn default() -> Self {
                Self(PhantomData)
            }
        }
    };
}

address_resolver!(AddressNerveResolver);
address_resolver!(AddressChainComplexResolver);
address_resolver!(AddressHomologyGroupResolver);
address_resolver!(AddressCochainComplexResolver);
address_resolver!(AddressCohomologyGroupResolver);
address_resolver!(AddressPostnikovResolver);
address_resolver!(AddressHomotopyGroupResolver);
address_resolver!(AddressKInvariantResolver);

macro_rules! passthrough_resolver {
    ($trait:ident, $name:ident) => {
        impl<const N: usize, H> $trait<N, H> for $name<H> {
            #[inline]
            fn resolve<'a>(
                &self,
                input: TermValue<'a, N>,
            ) -> Result<TermValue<'a, N>, ShapeViolation> {
                Ok(input)
            }
        }
    };
}

passthrough_resolver!(NerveResolver, AddressNerveResolver);
passthrough_resolver!(ChainComplexResolver, AddressChainComplexResolver);
passthrough_resolver!(HomologyGroupResolver, AddressHomologyGroupResolver);
passthrough_resolver!(CochainComplexResolver, AddressCochainComplexResolver);
passthrough_resolver!(CohomologyGroupResolver, AddressCohomologyGroupResolver);
passthrough_resolver!(PostnikovResolver, AddressPostnikovResolver);
passthrough_resolver!(HomotopyGroupResolver, AddressHomotopyGroupResolver);

impl<const N: usize, H: AddrHash> KInvariantResolver<N, H> for AddressKInvariantResolver<H> {
    #[inline]
    fn resolve<'a>(&self, input: TermValue<'a, N>) -> Result<TermValue<'a, N>, ShapeViolation> {
        // ψ₉ σ-projection: fold the (streamed) canonical carrier through H
        // and emit the formatted κ-label. The `'static` Inline carrier is
        // valid for any `'a`.
        Ok(kappa_label_carrier::<N, H>(&input))
    }
}

// ── The tuple. Bound on the fingerprint-width-erased [`AddrHash`] so it
//    carries every admissible σ-axis (32- and 64-byte). ──

/// The single eight-resolver ψ-tower shared by every realization.
pub struct AddressResolverTuple<H: AddrHash> {
    /// ψ₁ Nerve (pass-through).
    pub nerve: AddressNerveResolver<H>,
    /// ψ₂ ChainComplex (pass-through).
    pub chain_complex: AddressChainComplexResolver<H>,
    /// ψ₃ HomologyGroups (pass-through).
    pub homology_groups: AddressHomologyGroupResolver<H>,
    /// ψ₄ CochainComplex (pass-through).
    pub cochain_complex: AddressCochainComplexResolver<H>,
    /// ψ₅ CohomologyGroups (pass-through).
    pub cohomology_groups: AddressCohomologyGroupResolver<H>,
    /// ψ₇ PostnikovTower (pass-through).
    pub postnikov: AddressPostnikovResolver<H>,
    /// ψ₈ HomotopyGroups (pass-through).
    pub homotopy_groups: AddressHomotopyGroupResolver<H>,
    /// ψ₉ KInvariants (the σ-projection — emits the κ-label).
    pub k_invariants: AddressKInvariantResolver<H>,
    #[doc(hidden)]
    pub _phantom: PhantomData<H>,
}

impl<H: AddrHash> Sealed for AddressResolverTuple<H> {}

impl<H: AddrHash> ResolverTuple for AddressResolverTuple<H> {
    const ARITY: usize = 8;
    const CATEGORIES: &'static [ResolverCategory] = &[
        ResolverCategory::Nerve,
        ResolverCategory::ChainComplex,
        ResolverCategory::HomologyGroup,
        ResolverCategory::CochainComplex,
        ResolverCategory::CohomologyGroup,
        ResolverCategory::Postnikov,
        ResolverCategory::HomotopyGroup,
        ResolverCategory::KInvariant,
    ];
    type ShapeRegistry = EmptyShapeRegistry;
}

impl<H: AddrHash> Default for AddressResolverTuple<H> {
    fn default() -> Self {
        Self {
            nerve: AddressNerveResolver::default(),
            chain_complex: AddressChainComplexResolver::default(),
            homology_groups: AddressHomologyGroupResolver::default(),
            cochain_complex: AddressCochainComplexResolver::default(),
            cohomology_groups: AddressCohomologyGroupResolver::default(),
            postnikov: AddressPostnikovResolver::default(),
            homotopy_groups: AddressHomotopyGroupResolver::default(),
            k_invariants: AddressKInvariantResolver::default(),
            _phantom: PhantomData,
        }
    }
}

macro_rules! has_resolver {
    ($marker:ident, $rtrait:ident, $accessor:ident, $field:ident) => {
        impl<const N: usize, H: AddrHash> $marker<N, H> for AddressResolverTuple<H> {
            fn $accessor(&self) -> &dyn $rtrait<N, H> {
                &self.$field
            }
        }
    };
}

has_resolver!(HasNerveResolver, NerveResolver, nerve_resolver, nerve);
has_resolver!(
    HasChainComplexResolver,
    ChainComplexResolver,
    chain_complex_resolver,
    chain_complex
);
has_resolver!(
    HasHomologyGroupResolver,
    HomologyGroupResolver,
    homology_group_resolver,
    homology_groups
);
has_resolver!(
    HasCochainComplexResolver,
    CochainComplexResolver,
    cochain_complex_resolver,
    cochain_complex
);
has_resolver!(
    HasCohomologyGroupResolver,
    CohomologyGroupResolver,
    cohomology_group_resolver,
    cohomology_groups
);
has_resolver!(
    HasPostnikovResolver,
    PostnikovResolver,
    postnikov_resolver,
    postnikov
);
has_resolver!(
    HasHomotopyGroupResolver,
    HomotopyGroupResolver,
    homotopy_group_resolver,
    homotopy_groups
);
has_resolver!(
    HasKInvariantResolver,
    KInvariantResolver,
    k_invariant_resolver,
    k_invariants
);
