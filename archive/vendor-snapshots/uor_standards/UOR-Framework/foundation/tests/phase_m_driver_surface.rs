//! Phase M (target §5): driver surface completion.
//!
//! Pins:
//! - `pipeline::run`, `run_parallel`, `run_stream`, `run_interactive` all
//!   exist as `#[must_use] pub fn`.
//! - `run_interactive` returns `InteractionDriver<T, P, H>` (not the
//!   v0.2.1 `PeerPayload` stub).
//! - All driver return types are named, sealed, and no-alloc.

use uor_foundation::enforcement::{
    ConstrainedTypeInput, InteractionDeclarationBuilder, InteractionShape, Validated,
};
use uor_foundation::pipeline::{
    run_interactive, run_parallel, run_stream, InteractionDeclaration, InteractionDriver,
    ParallelDeclaration, StreamDeclaration, StreamDriver,
};
use uor_foundation_test_helpers::{validated_runtime, Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

#[test]
fn phase_m_run_stream_returns_named_sealed_stream_driver() {
    let unit: Validated<StreamDeclaration<'static, N>> =
        validated_runtime(StreamDeclaration::new::<ConstrainedTypeInput>(2));
    let driver: StreamDriver<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32> = run_stream(unit);
    // StreamDriver is the named return — no `impl Trait` hiding heap.
    let _ = driver;
}

#[test]
fn phase_m_run_parallel_returns_result_grounded() {
    // v0.2.2 Phase H3: `new_with_partition` is the sole constructor.
    static PARTITION: &[u32] = &[0, 1, 2];
    let unit: Validated<ParallelDeclaration> =
        validated_runtime(ParallelDeclaration::new_with_partition::<
            ConstrainedTypeInput,
        >(
            PARTITION,
            "https://uor.foundation/parallel/ParallelDisjointnessWitness",
        ));
    let result = run_parallel::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit)
        .expect("parallel walks");
    let _ = result;
}

#[test]
fn phase_m_run_interactive_returns_named_sealed_interaction_driver() {
    // Phase M.2: run_interactive now returns InteractionDriver (not PeerPayload).
    let unit: Validated<InteractionDeclaration> =
        validated_runtime(InteractionDeclaration::new::<ConstrainedTypeInput>(42));
    let driver: InteractionDriver<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32> =
        run_interactive(unit);
    let _ = driver;
}

#[test]
fn phase_m_drivers_have_named_return_types() {
    // Compile-time witness that every driver function returns a named,
    // concrete type — not `impl Iterator`, not `Box<dyn ...>`. The runtime
    // phase is made concrete so there are no inference placeholders.
    use uor_foundation::enforcement::Runtime;
    let _run_stream_ty: fn(
        Validated<StreamDeclaration<'static, N>, Runtime>,
    )
        -> StreamDriver<ConstrainedTypeInput, Runtime, Fnv1aHasher16, N, 32> = run_stream;
    let _run_interactive_ty: fn(
        Validated<InteractionDeclaration, Runtime>,
    ) -> InteractionDriver<
        ConstrainedTypeInput,
        Runtime,
        Fnv1aHasher16,
        N,
    > = run_interactive;
}

#[test]
fn phase_m_interaction_declaration_builder_shape_roundtrip() {
    // Phase E.6 + Phase M: the InteractionDeclarationBuilder validates
    // and its Validated<InteractionShape> is reachable alongside the
    // driver. The two surfaces coexist per target §4.6 / §5.
    let builder = InteractionDeclarationBuilder::new()
        .peer_protocol(1)
        .convergence_predicate(2)
        .commutator_state_class(3);
    let shape: Validated<InteractionShape> = builder.validate().expect("validates");
    let _ = shape;
}
