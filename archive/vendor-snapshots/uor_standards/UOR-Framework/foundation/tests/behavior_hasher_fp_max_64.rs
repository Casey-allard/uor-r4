//! Regression proof for the downstream-reported bug: a `Hasher<64>`
//! (SHA-512 output width) MUST flow through the entire pipeline + verify
//! tower. Foundation 0.5.1 pinned the resolver/pipeline tower to
//! `Hasher<32>` (`AxisTuple` blanket impl + `run_route`'s `A: AxisTuple +
//! Hasher` defaulted `FP_MAX = 32`), so `Sha512Hasher: Hasher<64>` could not
//! be selected at all. ADR-018/060 makes the fingerprint width a free const
//! generic threaded from the application's `HostBounds`; this test pins the
//! property end-to-end with a genuine 64-byte-width hasher:
//!
//!   1. a `PrismModel` whose A-axis is a `Hasher<64>` grounds via `forward()`
//!      → `run_route`, and the resulting `Grounded` carries a **64-byte**
//!      `ContentFingerprint` (not truncated to 32);
//!   2. the trace-replay round-trip (`Derivation::replay` →
//!      `certify_from_trace`) reproduces the **64-byte** fingerprint
//!      bit-identically — the verify path is width-parametric too.
//!
//! This file FAILS to even compile against the pinned-`Hasher<32>` 0.5.1
//! surface (`WideHasher: Hasher<64>` is rejected by `run_route`'s bound) and
//! PASSES once the width is fully parametric.

use uor_foundation::enforcement::{replay, ConstrainedTypeInput, Grounded, Hasher, Term};
use uor_foundation::pipeline::{
    carrier_inline_bytes, run_route, ConstrainedTypeShape, ConstraintRef, EmptyCommitment,
    FoundationClosed, IntoBindingValue, NullResolverTuple, PartitionProductFields, PrismModel,
    TermValue,
};
use uor_foundation::{DefaultHostTypes, HostBounds, PipelineFailure};

// ── A `HostBounds` selecting the SHA-512 fingerprint width (64 bytes) ──────

struct Sha512WidthBounds;
impl HostBounds for Sha512WidthBounds {
    const FINGERPRINT_MIN_BYTES: usize = 64;
    const FINGERPRINT_MAX_BYTES: usize = 64;
    const TRACE_MAX_EVENTS: usize = 256;
    const WITT_LEVEL_MAX_BITS: u32 = 64;
    const FOLD_UNROLL_THRESHOLD: usize = 8;
    const BETTI_DIMENSION_MAX: usize = 8;
    const NERVE_CONSTRAINTS_MAX: usize = 8;
    const NERVE_SITES_MAX: usize = 8;
    const JACOBIAN_SITES_MAX: usize = 8;
    const RECURSION_TRACE_DEPTH_MAX: usize = 16;
    const OP_CHAIN_DEPTH_MAX: usize = 8;
    const AFFINE_COEFFS_MAX: usize = 8;
    const CONJUNCTION_TERMS_MAX: usize = 8;
    const UNFOLD_ITERATIONS_MAX: usize = 256;
}

const N: usize = carrier_inline_bytes::<Sha512WidthBounds>();
const FP: usize = <Sha512WidthBounds as HostBounds>::FINGERPRINT_MAX_BYTES; // = 64

// ── A genuine `Hasher<64>` (SHA-512 output width) ─────────────────────────
//
// A content-deterministic 64-byte-wide fold. Not cryptographic — the property
// under test is *width parametricity through the pipeline*, orthogonal to the
// hash function's strength (ADR-021 V&V Decision 2). Mirrors the shape of a
// real `Sha512Hasher: Hasher<64>` an application would substitute.

#[derive(Clone, Copy)]
struct WideHasher {
    acc: [u8; 64],
    i: usize,
}

impl Hasher<64> for WideHasher {
    const OUTPUT_BYTES: usize = 64;

    fn initial() -> Self {
        // Distinct non-zero seed per lane so an empty input still yields a
        // non-degenerate 64-byte digest.
        let mut acc = [0u8; 64];
        let mut k = 0usize;
        while k < 64 {
            acc[k] = 0x9e ^ (k as u8);
            k += 1;
        }
        Self { acc, i: 0 }
    }

    fn fold_byte(mut self, b: u8) -> Self {
        let lane = self.i % 64;
        // FNV-ish mix into the active lane, then diffuse into the next lane.
        self.acc[lane] = self.acc[lane]
            .wrapping_mul(31)
            .wrapping_add(b ^ (self.i as u8));
        let next = (lane + 1) % 64;
        self.acc[next] ^= self.acc[lane].rotate_left(3);
        self.i = self.i.wrapping_add(1);
        self
    }

    fn finalize(self) -> [u8; 64] {
        self.acc
    }
}

// ── The `hash(input)` route (G19) over the 64-byte-width axis ─────────────

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

// A small borrowing input handle so the content-addressing folds real bytes.
#[derive(Clone, Copy)]
struct ByteHandle<'a>(&'a [u8]);
impl ConstrainedTypeShape for ByteHandle<'_> {
    const IRI: &'static str = "https://uor.foundation/test/Sha512ByteHandle";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = 1;
}
impl uor_foundation::pipeline::__sdk_seal::Sealed for ByteHandle<'_> {}
impl<'a> IntoBindingValue<'a> for ByteHandle<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        TermValue::borrowed(self.0)
    }
}
impl PartitionProductFields for ByteHandle<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

struct WideHashModel;
impl uor_foundation::pipeline::__sdk_seal::Sealed for WideHashModel {}
impl<'a> PrismModel<'a, DefaultHostTypes, Sha512WidthBounds, WideHasher, N, FP> for WideHashModel {
    type Input = ByteHandle<'a>;
    type Output = ConstrainedTypeInput;
    type Route = HashRoute;
    fn forward(input: Self::Input) -> Result<Grounded<'a, Self::Output, N, FP>, PipelineFailure> {
        run_route::<
            DefaultHostTypes,
            Sha512WidthBounds,
            WideHasher,
            Self,
            NullResolverTuple,
            EmptyCommitment,
            N,
            FP,
        >(input, &NullResolverTuple, &EmptyCommitment)
    }
}

#[test]
fn hasher_64_flows_through_forward_and_grounds_a_64_byte_fingerprint() {
    let bytes = b"the quick brown fox jumps over the lazy dog";
    let grounded = WideHashModel::forward(ByteHandle(bytes)).expect("Hasher<64> grounds");

    // The minted content fingerprint is the application's selected width (64),
    // NOT truncated to the legacy 32. This is the property the bug violated.
    let fp = grounded.content_fingerprint();
    assert_eq!(
        fp.width_bytes(),
        64,
        "fingerprint width must be the app's 64"
    );
    assert_eq!(
        fp.as_bytes().len(),
        64,
        "buffer is exactly FP_MAX = 64 bytes"
    );

    // The full 64-byte digest survived uncapped: the high half (bytes 32..64)
    // carries hash data. A `Hasher<32>`-pinned carrier would have zeroed it —
    // the exact truncation the bug imposed.
    let beyond_32_nonzero = fp.as_bytes()[32..].iter().any(|&b| b != 0);
    assert!(
        beyond_32_nonzero,
        "bytes 32..64 must carry digest data — a 32-pinned carrier would zero them"
    );
}

#[test]
fn hasher_64_round_trip_preserves_the_64_byte_fingerprint() {
    let bytes = b"content-addressed at SHA-512 width";
    let grounded = WideHashModel::forward(ByteHandle(bytes)).expect("Hasher<64> grounds");
    let minted = *grounded.content_fingerprint().as_bytes();

    // ADR-021 round-trip: replay the derivation as a Trace carrying the
    // 64-byte fingerprint, then re-certify via the (width-parametric) verify
    // primitive. `TR_MAX` from the bounds (256); `FP_MAX` = 64 is inferred.
    let trace = grounded
        .derivation()
        .replay::<{ <Sha512WidthBounds as HostBounds>::TRACE_MAX_EVENTS }>();
    let recert = replay::certify_from_trace(&trace).expect("trace verifies at width 64");

    // The verifier copies the fingerprint through unchanged — at full width.
    let reproduced = *recert.certificate().content_fingerprint().as_bytes();
    assert_eq!(
        reproduced.len(),
        64,
        "verify path carries the 64-byte fingerprint"
    );
    assert_eq!(
        reproduced, minted,
        "round-trip reproduces the 64-byte fingerprint bit-identically"
    );
}
