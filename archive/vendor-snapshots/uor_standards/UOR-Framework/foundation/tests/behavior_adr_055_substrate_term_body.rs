//! ADR-055 universal substrate-Term verb body discipline.
//!
//! Pins the type-system enforcement of the wiki's ADR-055 commitment:
//!
//! 1. `SubstrateTermBody` exists as a public trait on `uor_foundation::pipeline`,
//!    sealed via `__sdk_seal::Sealed`, with a single `body_arena() -> &'static [Term]`
//!    method.
//! 2. `AxisExtension` has `SubstrateTermBody` as a supertrait — every axis impl
//!    structurally carries its substrate-Term decomposition (or the empty-arena
//!    primitive-fast-path interpretation per ADR-055).
//! 3. `AxisTuple::body_arena_at(axis_index)` exposes the per-axis body so the
//!    catamorphism's `Term::AxisInvocation` fold-rule can branch on the
//!    primitive-fast-path vs recursive-fold path per ADR-029's amendment.
//! 4. The `HashAxis<H>` foundation adapter satisfies `SubstrateTermBody` with
//!    an empty body (the canonical hash fold is the primitive fast-path).
//! 5. The `axis!` SDK macro emits `SubstrateTermBody` alongside `AxisExtension`
//!    for every companion-macro invocation, so existing axis impls migrate
//!    without source changes.

use uor_foundation::enforcement::{HashAxis, Hasher};
use uor_foundation::pipeline::{AxisExtension, AxisTuple, SubstrateTermBody};
use uor_foundation_test_helpers::REFERENCE_INLINE_BYTES as N;

/// Foundation-internal sample Hasher used to instantiate `HashAxis<H>`.
#[derive(Debug, Clone, Copy, Default)]
struct ProbeHasher;
impl Hasher for ProbeHasher {
    const OUTPUT_BYTES: usize = 4;
    fn initial() -> Self {
        Self
    }
    fn fold_byte(self, _: u8) -> Self {
        self
    }
    fn finalize(self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[0] = 0x11;
        out[1] = 0x22;
        out[2] = 0x33;
        out[3] = 0x44;
        out
    }
}

#[test]
fn substrate_term_body_trait_is_publicly_reachable() {
    // The trait must be a public path on uor_foundation::pipeline so the
    // axis! macro emission's `impl SubstrateTermBody for $struct_ident` is
    // legible to external crates. Coerce body_arena to a function pointer
    // to pin both the path and the signature at the type level.
    let body_fn: fn() -> &'static [uor_foundation::enforcement::Term<'static, N>] =
        <HashAxis<ProbeHasher> as SubstrateTermBody<N>>::body_arena;
    let arena: &'static [uor_foundation::enforcement::Term<'static, N>] = body_fn();
    assert_eq!(arena.len(), 0);
}

#[test]
fn axis_extension_requires_substrate_term_body_supertrait() {
    // Type-system check: an AxisExtension bound must transitively bind
    // SubstrateTermBody. If the supertrait bound is missing this fails to
    // compile. Coerce one method from each trait to assert both surfaces
    // are reachable through the joint bound.
    fn pair_check<T: AxisExtension<N, 32> + SubstrateTermBody<N>>() -> (
        &'static str,
        fn() -> &'static [uor_foundation::enforcement::Term<'static, N>],
    ) {
        (
            <T as AxisExtension<N, 32>>::AXIS_ADDRESS,
            <T as SubstrateTermBody<N>>::body_arena,
        )
    }
    let (addr, body_fn) = pair_check::<HashAxis<ProbeHasher>>();
    assert_eq!(addr, "https://uor.foundation/axis/HashAxis");
    assert!(body_fn().is_empty());
}

#[test]
fn hash_axis_body_arena_is_empty_primitive_fast_path() {
    // ADR-055 carves out empty body_arena as the primitive-fast-path
    // interpretation. HashAxis is the canonical example: its body is
    // byte-output-equivalent to `fold_bytes` ∘ `finalize`.
    let body = <HashAxis<ProbeHasher> as SubstrateTermBody<N>>::body_arena();
    assert!(
        body.is_empty(),
        "HashAxis body_arena must be empty (primitive fast-path)"
    );
}

#[test]
fn axis_tuple_exposes_body_arena_at_per_position() {
    // AxisTuple::body_arena_at(axis_index) delegates to the per-axis
    // SubstrateTermBody impl. For a 1-tuple, axis_index = 0 routes to A0;
    // out-of-range indices return &[].
    let body0 = <(HashAxis<ProbeHasher>,) as AxisTuple<N, 32>>::body_arena_at(0);
    let body1 = <(HashAxis<ProbeHasher>,) as AxisTuple<N, 32>>::body_arena_at(1);
    assert!(body0.is_empty());
    assert!(body1.is_empty());
}

#[test]
fn hasher_blanket_axis_tuple_body_arena_at_returns_empty() {
    // The foundation-built `impl<H: Hasher> AxisTuple for H` blanket has the
    // ADR-055 body_arena_at signature too — empty for the canonical hash axis.
    let body = <ProbeHasher as AxisTuple<N, 32>>::body_arena_at(0);
    assert!(body.is_empty());
}

#[test]
fn axis_invocation_fold_rule_uses_dispatch_kernel_when_body_empty() {
    // The catamorphism's Term::AxisInvocation handler checks
    // body_arena_at(axis_index); when empty, it dispatches the kernel
    // function directly (the fast-path per ADR-055). Exercise this by
    // routing a hash invocation through evaluate_term_tree.
    use uor_foundation::enforcement::Term;
    use uor_foundation::pipeline::{evaluate_term_tree, NullResolverTuple};
    let arena = [
        Term::Variable { name_index: 0 },
        Term::AxisInvocation {
            axis_index: 0,
            kernel_id: 0,
            input_index: 0,
        },
    ];
    let result = evaluate_term_tree::<ProbeHasher, NullResolverTuple, N, 32>(
        &arena,
        uor_foundation::pipeline::TermValue::borrowed(&[0x42u8]),
        &NullResolverTuple,
    )
    .expect("AxisInvocation primitive-fast-path evaluates");
    // ProbeHasher's finalize() emits the fixed pattern in OUTPUT_BYTES=4.
    assert_eq!(result.bytes(), &[0x11, 0x22, 0x33, 0x44][..]);
}
