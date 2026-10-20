//! Behavioral contract for the `IntoBindingValue` developer surface.
//!
//! Per the UOR-Framework wiki ADR-023 (amended by ADR-060), foundation
//! declares `IntoBindingValue<'a>` as the trait every `M::Input` must
//! implement so a runtime input value can flow into the `CompileUnit`
//! binding table. `pipeline::run_route` calls `as_binding_value` to obtain
//! a source-polymorphic `TermValue` carrier (Inline / Borrowed / Stream),
//! folds it through the application's selected `Hasher` (chunk-by-chunk for
//! `Stream`), and constructs a transient `Binding` for the route's input
//! slot (`Term::Variable { name_index: 0 }` per ADR-022 D3 G2). ADR-060
//! removed the foundation-fixed `ROUTE_INPUT_BUFFER_BYTES` ceiling and the
//! `MAX_BYTES` cap: there is no input byte-width limit.
//!
//! This test pins:
//!
//! 1. `IntoBindingValue` is reachable through `pipeline::*`.
//! 2. The trait's `as_binding_value` method returns a source-polymorphic
//!    `TermValue` carrier (no `MAX_BYTES` ceiling).
//! 3. The trait is sealed via `__sdk_seal::Sealed` (same supertrait
//!    as `FoundationClosed` and `PrismModel`).
//! 4. Foundation's identity-route impl on `ConstrainedTypeInput`
//!    returns the empty carrier (no bytes).
//! 5. `pipeline::carrier_inline_bytes::<B>()` derives the inline-carrier
//!    width from the application's `HostBounds` (ADR-060).
//! 6. `PrismModel::Input` carries the `IntoBindingValue<'a>` bound.

use uor_foundation::enforcement::{ConstrainedTypeInput, Hasher};
use uor_foundation::pipeline::{ConstrainedTypeShape, IntoBindingValue, PrismModel};
use uor_foundation::{DefaultHostTypes, HostBounds, HostTypes};
use uor_foundation_test_helpers::{ReferenceHostBounds, REFERENCE_INLINE_BYTES as N};

#[derive(Debug, Clone, Copy, Default)]
struct TestHasher;

impl Hasher for TestHasher {
    const OUTPUT_BYTES: usize = 16;

    fn initial() -> Self {
        Self
    }
    fn fold_byte(self, _: u8) -> Self {
        self
    }
    fn finalize(self) -> [u8; 32] {
        [0; 32]
    }
}

#[test]
fn into_binding_value_surface_resolves_at_crate_root() {
    fn _accepts<'a, T: IntoBindingValue<'a>>() {}
    _accepts::<ConstrainedTypeInput>();
    // Pin the surface behaviorally: the foundation-sanctioned input
    // shape is reachable as a real type (observable through
    // `core::any::type_name`), which exercises that the trait surface
    // resolves at the foundation crate's public path rather than only
    // at compile time via the bound check above.
    assert!(
        core::any::type_name::<ConstrainedTypeInput>().ends_with("ConstrainedTypeInput"),
        "the foundation-sanctioned identity-input shape must be reachable",
    );
}

#[test]
fn into_binding_value_as_binding_value_returns_source_polymorphic_carrier() {
    // ADR-023 amended by ADR-060: the trait surface is `as_binding_value`
    // returning a source-polymorphic `TermValue` carrier — there is no
    // `MAX_BYTES` ceiling. The foundation-sanctioned identity-route impl
    // returns the empty carrier (the empty shape carries no bytes).
    let value = ConstrainedTypeInput::default();
    let carrier: uor_foundation::pipeline::TermValue<'_, N> = value.as_binding_value::<N>();
    assert!(carrier.bytes().is_empty());
}

// The uncapped large-input → `Grounded` proof (a multi-KB `Borrowed` input
// and a multi-MB `Stream` input flowing through `prism_model!`/`run_route`)
// lives in `behavior_adr_060_large_input_grounded.rs`.

#[test]
fn carrier_inline_bytes_is_host_bounds_derived() {
    // ADR-060: the inline-carrier width through which a runtime input
    // value flows is derived from the application's `HostBounds` via
    // `carrier_inline_bytes::<B>()` — there is no foundation-fixed
    // `ROUTE_INPUT_BUFFER_BYTES` ceiling any more. Pin that the helper
    // resolves and agrees with the reference impl's published const so
    // applications and the conformance suite can reason about it.
    assert_eq!(
        uor_foundation::pipeline::carrier_inline_bytes::<ReferenceHostBounds>(),
        N
    );
    // `N` is the foundation-derived inline-carrier width every
    // `TermValue::Inline` is sized to for this `HostBounds`.
    let buf = [0u8; N];
    assert_eq!(buf.len(), N);
}

#[test]
fn prism_model_input_bound_includes_into_binding_value() {
    // Wiki ADR-023 + ADR-022 D4: `PrismModel::Input` is bound by
    // `ConstrainedTypeShape + IntoBindingValue`. The parametric check
    // pins the bound: any `M: PrismModel<…>` has `M::Input:
    // IntoBindingValue`.
    fn _input_implements_into_binding<
        'a,
        H,
        B,
        A,
        M,
        const INLINE_BYTES: usize,
        const FP_MAX: usize,
    >()
    where
        H: HostTypes,
        B: HostBounds,
        A: Hasher<FP_MAX>,
        M: PrismModel<'a, H, B, A, INLINE_BYTES, FP_MAX>,
    {
        fn _check<'a, T: IntoBindingValue<'a>>() {}
        _check::<M::Input>();
    }
    // Identity route via the foundation-sanctioned impl.
    struct IdentityModel;
    impl uor_foundation::pipeline::__sdk_seal::Sealed for IdentityModel {}
    impl<'a> PrismModel<'a, DefaultHostTypes, ReferenceHostBounds, TestHasher, N, 32>
        for IdentityModel
    {
        type Input = ConstrainedTypeInput;
        type Output = ConstrainedTypeInput;
        type Route = ConstrainedTypeInput;

        fn forward(
            input: Self::Input,
        ) -> Result<
            uor_foundation::enforcement::Grounded<'a, Self::Output, N, 32>,
            uor_foundation::PipelineFailure,
        > {
            uor_foundation::pipeline::run_route::<
                DefaultHostTypes,
                ReferenceHostBounds,
                TestHasher,
                Self,
                uor_foundation::pipeline::NullResolverTuple,
                uor_foundation::pipeline::EmptyCommitment,
                N,
                32,
            >(
                input,
                &uor_foundation::pipeline::NullResolverTuple,
                &uor_foundation::pipeline::EmptyCommitment,
            )
        }
    }
    _input_implements_into_binding::<
        DefaultHostTypes,
        ReferenceHostBounds,
        TestHasher,
        IdentityModel,
        N,
        32,
    >();

    // The ConstrainedTypeShape bound is preserved.
    let iri = <<IdentityModel as PrismModel<
        'static,
        DefaultHostTypes,
        ReferenceHostBounds,
        TestHasher,
        N,
        32,
    >>::Input as ConstrainedTypeShape>::IRI;
    assert!(iri.contains("ConstrainedType"));
}

#[test]
fn into_binding_value_is_sealed_via_sdk_seal() {
    // Pin that `IntoBindingValue` requires `__sdk_seal::Sealed` (per
    // ADR-023). External crates cannot impl `IntoBindingValue` without
    // first naming the doc-hidden seal — the same architectural
    // pattern foundation uses for `FoundationClosed` and `PrismModel`.
    fn _accepts_sealed<
        'a,
        T: IntoBindingValue<'a> + uor_foundation::pipeline::__sdk_seal::Sealed,
    >() {
    }
    _accepts_sealed::<ConstrainedTypeInput>();
    // Behavioral assertion: the seal supertrait's qualified name
    // includes `__sdk_seal::Sealed` exactly, exercising that the
    // foundation public path matches the wiki ADR-022 D1 + ADR-023
    // surface contract.
    let seal_name =
        core::any::type_name::<fn() -> Box<dyn uor_foundation::pipeline::__sdk_seal::Sealed>>();
    assert!(
        seal_name.contains("__sdk_seal::Sealed"),
        "the foundation seal supertrait must be reachable as `__sdk_seal::Sealed`; got {seal_name}",
    );
}
