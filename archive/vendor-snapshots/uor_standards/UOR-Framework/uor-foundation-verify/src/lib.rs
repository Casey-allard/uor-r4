//! Trace-replay verifier for the UOR Foundation.
//!
//! v0.2.2 T5 — this crate is a thin façade over
//! [`uor_foundation::enforcement::replay::certify_from_trace`], which validates
//! a [`Trace`] produced by [`uor_foundation::enforcement::Derivation::replay`]
//! and re-derives a [`Certified<GroundingCertificate>`] via **structural
//! validation + fingerprint passthrough**. The verifier does NOT invoke any
//! hash function — the parametric [`ContentFingerprint`] is *data carried by
//! the Trace*, computed at mint time by the consumer-supplied [`Hasher`] and
//! passed through unchanged.
//!
//! # Substrate-agnostic hashing
//!
//! The foundation does not prescribe a hash function. Downstream substrate
//! consumers (PRISM, application crates) supply their own [`Hasher`] impl —
//! BLAKE3 (recommended for production), SHA-256, BLAKE2b, FNV-1a, or any
//! conforming impl. The architectural shape mirrors `Calibration`: the
//! foundation defines the abstract quantity ([`ContentFingerprint`]) and the
//! substitution point ([`Hasher`]); downstream provides the concrete substrate
//! AND chooses the output width within the application's [`HostBounds`] budget
//! (`[<B as HostBounds>::FINGERPRINT_MIN_BYTES, <B as HostBounds>::FINGERPRINT_MAX_BYTES]`).
//!
//! # Round-trip property
//!
//! For every `Grounded<T>` value produced by `pipeline::run::<T, _, H>` with
//! a conforming substrate `H: Hasher`:
//!
//! ```ignore
//! verify_trace(&grounded.derivation().replay()).certificate()
//!     == grounded.certificate()
//! ```
//!
//! holds bit-identically. The contract is orthogonal to the substrate hasher
//! choice and to the chosen `OUTPUT_BYTES` width.

#![no_std]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::missing_errors_doc)]

pub use uor_foundation::enforcement::replay::certify_from_trace;
pub use uor_foundation::enforcement::{
    BindingsTableError, Certificate, CertificateKind, Certified, ContentAddress,
    ContentFingerprint, GroundingCertificate, Hasher, ReplayError, Trace, TraceEvent,
    TRACE_REPLAY_FORMAT_VERSION,
};
pub use uor_foundation::PrimitiveOp;

// Wiki ADR-018: `HostBounds` is the carrier of every capacity bound that
// varies along the principal data path. Verifier callers reach
// `Trace::<TR_MAX>` and `certify_from_trace::<TR_MAX>` with
// `TR_MAX = <MyBounds as HostBounds>::TRACE_MAX_EVENTS`. ADR-060 removed
// `DefaultHostBounds`: there is no default application, so every caller
// declares its own `impl HostBounds` and threads its constants explicitly.
pub use uor_foundation::HostBounds;

/// Verify a trace by re-deriving its certificate via structural validation +
/// fingerprint passthrough.
///
/// Validates the trace's structural invariants (monotonic, contiguous step
/// indices; no zero targets; no `None` slots in the populated prefix) and
/// re-packages the trace's stored [`ContentFingerprint`] and `witt_level_bits`
/// into a fresh [`Certified<GroundingCertificate>`]. The verifier never
/// invokes a hash function: the fingerprint is *data carried by the Trace*,
/// computed at mint time by the consumer-supplied [`Hasher`] and copied
/// through unchanged. This makes the round-trip property orthogonal to:
///
///   - the choice of hash function (BLAKE3, SHA-256, BLAKE2b, FNV-1a, ...)
///   - the choice of output width (any value in
///     `[<B as HostBounds>::FINGERPRINT_MIN_BYTES,
///       <B as HostBounds>::FINGERPRINT_MAX_BYTES]`)
///
/// The foundation **recommends BLAKE3** as the default substrate hasher for
/// production deployments. PRISM ships a BLAKE3 [`Hasher`] impl; application
/// crates should use it unless they have a specific reason to deviate. The
/// recommendation is non-binding — any conforming [`Hasher`] impl works.
///
/// # Errors
///
/// Returns:
///
///   - [`ReplayError::EmptyTrace`] if `trace.is_empty()`.
///   - [`ReplayError::OutOfOrderEvent`] if event step indices are not strictly
///     monotonic at the reported `index`.
///   - [`ReplayError::ZeroTarget`] if any event carries `ContentAddress::zero()`
///     (forbidden in well-formed traces).
///   - [`ReplayError::NonContiguousSteps`] if event step indices skip values
///     (e.g., `[0, 2, 5]` with `len = 3`).
///
/// # Example
///
/// ```no_run
/// use uor_foundation::enforcement::{Derivation, Trace};
/// use uor_foundation_verify::verify_trace;
///
/// # fn example(derivation: &Derivation) {
/// // ADR-018/060: `TR_MAX`/`FP_MAX` come from the application's `HostBounds`;
/// // here the type-annotated binding fixes the defaults (256 events, 32-byte
/// // fingerprint). `verify_trace` infers both const-generics from the trace.
/// let trace: Trace = derivation.replay();
/// let certified = verify_trace(&trace).expect("trace verifies");
/// let fingerprint = certified.certificate().content_fingerprint();
/// # let _ = fingerprint;
/// # }
/// ```
///
/// ADR-018/060: `verify_trace` is parametric over the trace's event-count
/// ceiling `TR_MAX` and fingerprint width `FP_MAX`; both are inferred from the
/// `trace` argument, so an application using a 64-byte-fingerprint `HostBounds`
/// (e.g. SHA-512) round-trips through this same call with no turbofish.
#[inline]
pub fn verify_trace<const TR_MAX: usize, const FP_MAX: usize>(
    trace: &Trace<TR_MAX, FP_MAX>,
) -> Result<Certified<GroundingCertificate<FP_MAX>>, ReplayError> {
    certify_from_trace(trace)
}
