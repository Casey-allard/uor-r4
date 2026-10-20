//! ADR-060 large-input → `Grounded` verification (the release-blocking proof).
//!
//! ADR-060 principle (3) — "there is no carrier-side fixed allocation that
//! depends on payload size" — and "content-addressing of multi-GB **inputs**
//! flows natively through the ψ-pipeline." This file proves the property at
//! the **principal application entry path**: a `PrismModel`'s `forward()`
//! delegating to `pipeline::run_route` admits inputs **larger than the inline
//! carrier width** and content-addresses them over their **full** byte
//! sequence — no `MAX_BYTES > INLINE_BYTES` rejection, no truncation.
//!
//! Two carriers are exercised through the sanctioned `run_route` path:
//!   1. a several-KB `Borrowed` input (>> the ~97-byte inline width), and
//!   2. a multi-MiB `Stream` input (`ChunkSource`), folded chunk-by-chunk
//!      with bounded resident memory.
//!
//! Each asserts the resulting `Grounded`'s output (the σ-projection digest of
//! the route `hash(input)`) equals an independent fold of the same bytes, and
//! that flipping a late byte changes the digest — i.e. the **whole** input was
//! folded, not a capped prefix. This test FAILS against the pre-fix capped
//! `run_route` (which rejected `MAX_BYTES > INLINE_BYTES`) and PASSES once the
//! input path is source-polymorphic.

use core::cell::Cell;

use uor_foundation::enforcement::{ConstrainedTypeInput, Grounded, Hasher, Term};
use uor_foundation::pipeline::{
    run_route, ChunkSource, ConstrainedTypeShape, ConstraintRef, EmptyCommitment, FoundationClosed,
    IntoBindingValue, NullResolverTuple, PartitionProductFields, PrismModel, TermValue,
};
use uor_foundation::{DefaultHostTypes, PipelineFailure};
use uor_foundation_test_helpers::{
    Fnv1aHasher32, ReferenceHostBounds, REFERENCE_FP_MAX as FP, REFERENCE_INLINE_BYTES as N,
};

// ── A borrowing-handle input shape carrying a `&'a [u8]` of any size ──────
//
// This is the application-author surface ADR-060 enables: an `M::Input` that
// is a thin handle borrowing the caller's bytes for `'a`. `as_binding_value`
// returns a zero-copy `Borrowed` carrier — no fixed buffer, no width cap.

#[derive(Clone, Copy)]
struct ByteHandle<'a>(&'a [u8]);

impl ConstrainedTypeShape for ByteHandle<'_> {
    const IRI: &'static str = "https://uor.foundation/test/ByteHandle";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = 1;
}
impl uor_foundation::pipeline::__sdk_seal::Sealed for ByteHandle<'_> {}
impl<'a> IntoBindingValue<'a> for ByteHandle<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        // `self.0` is `&'a [u8]` (Copy), so the returned carrier borrows the
        // input's `'a`-lived data independently of the `&self` call borrow.
        TermValue::borrowed(self.0)
    }
}
impl PartitionProductFields for ByteHandle<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

// ── A streaming-handle input shape carrying a `&'a dyn ChunkSource` ───────

#[derive(Clone, Copy)]
struct StreamHandle<'a>(&'a dyn ChunkSource);

impl ConstrainedTypeShape for StreamHandle<'_> {
    const IRI: &'static str = "https://uor.foundation/test/StreamHandle";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = 1;
}
impl uor_foundation::pipeline::__sdk_seal::Sealed for StreamHandle<'_> {}
impl<'a> IntoBindingValue<'a> for StreamHandle<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::stream(self.0)
    }
}
impl PartitionProductFields for StreamHandle<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

// ── The `hash(input)` route: σ-projection of the input through the Hasher ──
//
// Arena: [Variable(0) (input), AxisInvocation{0,0,0} (canonical hash)]. This
// is exactly what the `prism_model!` closure body `hash(input)` lowers to
// (ADR-026 G19); written by hand here so the test depends only on foundation.

const HASH_ARENA: &[Term<'static, N>] = &[
    Term::Variable { name_index: 0 },
    Term::AxisInvocation {
        axis_index: 0,
        kernel_id: 0,
        input_index: 0,
    },
];

struct HashRoute;
impl uor_foundation::pipeline::__sdk_seal::Sealed for HashRoute {}
impl FoundationClosed<N> for HashRoute {
    fn arena_slice() -> &'static [Term<'static, N>] {
        HASH_ARENA
    }
}

// Two models: one over the Borrowed handle, one over the Stream handle. Both
// content-address the full input via `hash(input)` and ground the digest.

struct HashBorrowedModel;
impl uor_foundation::pipeline::__sdk_seal::Sealed for HashBorrowedModel {}
impl<'a> PrismModel<'a, DefaultHostTypes, ReferenceHostBounds, Fnv1aHasher32, N, FP>
    for HashBorrowedModel
{
    type Input = ByteHandle<'a>;
    type Output = ConstrainedTypeInput;
    type Route = HashRoute;
    fn forward(input: Self::Input) -> Result<Grounded<'a, Self::Output, N, FP>, PipelineFailure> {
        run_route::<
            DefaultHostTypes,
            ReferenceHostBounds,
            Fnv1aHasher32,
            Self,
            NullResolverTuple,
            EmptyCommitment,
            N,
            FP,
        >(input, &NullResolverTuple, &EmptyCommitment)
    }
}

struct HashStreamModel;
impl uor_foundation::pipeline::__sdk_seal::Sealed for HashStreamModel {}
impl<'a> PrismModel<'a, DefaultHostTypes, ReferenceHostBounds, Fnv1aHasher32, N, FP>
    for HashStreamModel
{
    type Input = StreamHandle<'a>;
    type Output = ConstrainedTypeInput;
    type Route = HashRoute;
    fn forward(input: Self::Input) -> Result<Grounded<'a, Self::Output, N, FP>, PipelineFailure> {
        run_route::<
            DefaultHostTypes,
            ReferenceHostBounds,
            Fnv1aHasher32,
            Self,
            NullResolverTuple,
            EmptyCommitment,
            N,
            FP,
        >(input, &NullResolverTuple, &EmptyCommitment)
    }
}

// ── The identity route: output IS the input carrier (empty arena) ─────────
//
// Proves ADR-028-as-amended on the OUTPUT side: a `Grounded` carries a
// source-polymorphic `TermValue` output with no byte-width ceiling. An
// identity route on a large `Borrowed` input yields a `Grounded<'a>` whose
// output carrier exposes the FULL input by reference (zero-copy, uncapped).

struct IdentityRoute;
impl uor_foundation::pipeline::__sdk_seal::Sealed for IdentityRoute {}
impl FoundationClosed<N> for IdentityRoute {
    fn arena_slice() -> &'static [Term<'static, N>] {
        &[]
    }
}

struct IdentityModel;
impl uor_foundation::pipeline::__sdk_seal::Sealed for IdentityModel {}
impl<'a> PrismModel<'a, DefaultHostTypes, ReferenceHostBounds, Fnv1aHasher32, N, FP>
    for IdentityModel
{
    type Input = ByteHandle<'a>;
    type Output = ConstrainedTypeInput;
    type Route = IdentityRoute;
    fn forward(input: Self::Input) -> Result<Grounded<'a, Self::Output, N, FP>, PipelineFailure> {
        run_route::<
            DefaultHostTypes,
            ReferenceHostBounds,
            Fnv1aHasher32,
            Self,
            NullResolverTuple,
            EmptyCommitment,
            N,
            FP,
        >(input, &NullResolverTuple, &EmptyCommitment)
    }
}

#[test]
fn identity_route_grounds_a_borrowed_output_carrier_exposing_the_full_input() {
    // ADR-028 amended by ADR-060: the `Grounded` output is a source-polymorphic
    // `TermValue` with no width ceiling. The identity route returns the input
    // carrier itself, so the `Grounded<'a>` carries a `Borrowed` output exposing
    // the full 8 KiB input — far beyond the ~97-byte inline width.
    let big = vec![0x33u8; 8 * 1024];
    let grounded = IdentityModel::forward(ByteHandle(&big))
        .expect("identity route over a large Borrowed input is admissible");
    assert_eq!(
        grounded.output_bytes(),
        &big[..],
        "the Grounded output must expose the full input (uncapped, zero-copy)",
    );
    // The output carrier is `Borrowed` (zero-copy into the input), not a copy
    // into an inline buffer — confirming there is no output byte-width ceiling.
    assert!(
        matches!(grounded.output_value(), TermValue::Borrowed(b) if b.len() == big.len()),
        "the output carrier must be a Borrowed view of the full input",
    );
}

/// Independent reference: fold `bytes` through the same hasher contiguously.
fn reference_digest(bytes: &[u8]) -> [u8; 32] {
    let mut h = <Fnv1aHasher32 as Hasher>::initial();
    h = h.fold_bytes(bytes);
    h.finalize()
}

#[test]
fn borrowed_input_far_larger_than_inline_width_grounds_over_full_input() {
    // 8 KiB input — ~84× the ~97-byte inline carrier width N. The pre-ADR-060
    // capped `run_route` rejected any input with MAX_BYTES > INLINE_BYTES.
    let big = vec![0x5Au8; 8 * 1024];
    assert!(big.len() > N, "input must exceed the inline carrier width");

    let grounded = HashBorrowedModel::forward(ByteHandle(&big))
        .expect("ADR-060: a large Borrowed input must be admissible (no size cap)");

    let digest = reference_digest(&big);
    let want = &digest[..<Fnv1aHasher32 as Hasher>::OUTPUT_BYTES];
    assert_eq!(
        grounded.output_bytes(),
        want,
        "Grounded output must be the σ-projection digest of the FULL input",
    );

    // Flip a byte near the END: if the digest changes, the whole input was
    // folded (a capped prefix-fold would miss the late byte).
    let mut big2 = big.clone();
    let last = big2.len() - 1;
    big2[last] ^= 0xFF;
    let grounded2 =
        HashBorrowedModel::forward(ByteHandle(&big2)).expect("second large input admissible");
    assert_ne!(
        grounded.output_bytes(),
        grounded2.output_bytes(),
        "flipping a late input byte must change the digest — the full input is folded",
    );
}

/// A multi-MiB byte source generated on the fly from a fixed scratch buffer —
/// peak resident memory is one chunk, never the total payload.
#[derive(Debug)]
struct PatternStream {
    total: usize,
    folded: Cell<usize>,
}
const STREAM_CHUNK: usize = 64 * 1024;
fn pattern_byte(i: usize) -> u8 {
    (i.wrapping_mul(31).wrapping_add(7)) as u8
}
impl ChunkSource for PatternStream {
    fn for_each_chunk(&self, f: &mut dyn FnMut(&[u8])) {
        let mut buf = [0u8; STREAM_CHUNK];
        let mut emitted = 0usize;
        while emitted < self.total {
            let n = core::cmp::min(STREAM_CHUNK, self.total - emitted);
            for (k, slot) in buf.iter_mut().enumerate().take(n) {
                *slot = pattern_byte(emitted + k);
            }
            self.folded.set(self.folded.get() + n);
            f(&buf[..n]);
            emitted += n;
        }
    }
    fn total_bytes(&self) -> Option<usize> {
        Some(self.total)
    }
}

#[test]
fn streamed_multi_mib_input_grounds_over_full_input_with_bounded_memory() {
    // 4 MiB streamed input — 1024× the retired 4096-byte cap, with peak
    // resident memory of one 64 KiB chunk.
    const TOTAL: usize = 4 * 1024 * 1024;
    let src = PatternStream {
        total: TOTAL,
        folded: Cell::new(0),
    };

    let grounded = HashStreamModel::forward(StreamHandle(&src))
        .expect("ADR-060: a multi-MiB Stream input must be admissible (no size cap)");
    // The stream is folded once for the input content-address binding (ADR-023)
    // and once for the σ-projection route output (ADR-029) — both via
    // `for_each_chunk`, each visiting every byte with bounded resident memory.
    assert!(
        src.folded.get() >= TOTAL && src.folded.get() % TOTAL == 0,
        "every byte of the streamed input must be folded (got {} for TOTAL={TOTAL})",
        src.folded.get(),
    );

    // Reference: fold the same canonical byte sequence contiguously.
    let mut h = <Fnv1aHasher32 as Hasher>::initial();
    let mut buf = [0u8; STREAM_CHUNK];
    let mut emitted = 0usize;
    while emitted < TOTAL {
        let n = core::cmp::min(STREAM_CHUNK, TOTAL - emitted);
        for (k, slot) in buf.iter_mut().enumerate().take(n) {
            *slot = pattern_byte(emitted + k);
        }
        h = h.fold_bytes(&buf[..n]);
        emitted += n;
    }
    let digest = h.finalize();
    let want = &digest[..<Fnv1aHasher32 as Hasher>::OUTPUT_BYTES];
    assert_eq!(
        grounded.output_bytes(),
        want,
        "Grounded output must be the σ-projection digest of the FULL streamed input",
    );
}
