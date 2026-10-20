//! ADR-060 arbitrary-scaling verification.
//!
//! ADR-060 ("source-polymorphic value carrier") removed the pre-0.5.0
//! fixed 4096-byte `TermValue` ceiling that named no mathematical truth and
//! capped content-addressing at an arbitrary width. This file is the
//! normative demonstration that the carrier scales to payloads orders of
//! magnitude larger than the retired cap — and does so with **bounded
//! resident memory**: the σ-projection folds a `TermValue::Stream` through the
//! application's `Hasher` chunk-by-chunk via [`ChunkSource::for_each_chunk`],
//! never materializing the canonical byte sequence.
//!
//! The three properties pinned here:
//!
//!   1. **No cap.** An 8 MiB payload (2048× the retired 4096-byte ceiling)
//!      flows through the carrier intact — `len_hint()` reports the full
//!      length and the fold visits every byte.
//!   2. **Losslessness.** Folding the stream chunk-by-chunk yields a digest
//!      bit-identical to folding the same logical byte sequence as one
//!      contiguous buffer. The carrier introduces no truncation or padding.
//!   3. **Bounded memory.** The source generates each chunk on demand from a
//!      fixed-size scratch buffer; peak resident bytes are the chunk size plus
//!      the source's tiny internal state — independent of total payload size.
//!      The inline carrier width `N` stays at its small foundation-derived
//!      value (97 for `ReferenceHostBounds`) while the payload is multi-MiB.

use core::cell::Cell;

use uor_foundation::enforcement::Hasher;
use uor_foundation::pipeline::{ChunkSource, TermValue};
use uor_foundation_test_helpers::{Fnv1aHasher32, REFERENCE_INLINE_BYTES as N};

/// Total payload: 8 MiB — 2048× the retired 4096-byte `TERM_VALUE_MAX_BYTES`.
const TOTAL_BYTES: usize = 8 * 1024 * 1024;
/// Emission granularity: 64 KiB per chunk. Peak resident payload memory is
/// one chunk — never `TOTAL_BYTES`.
const CHUNK_BYTES: usize = 64 * 1024;

/// A deterministic multi-MiB byte source that allocates **no** buffer for the
/// full payload. Each chunk is generated on the fly into a single reused
/// `CHUNK_BYTES` scratch array; byte `i` of the canonical sequence is
/// `(i * 31 + 7) as u8`, a cheap position-dependent pattern so the fold is
/// sensitive to ordering and length.
#[derive(Debug)]
struct PatternSource {
    total: usize,
    /// Observed total folded byte count — proves every byte was visited.
    folded: Cell<usize>,
    /// Largest single chunk handed to the fold closure — proves bounded
    /// resident memory (must never exceed `CHUNK_BYTES`).
    max_chunk: Cell<usize>,
}

impl PatternSource {
    fn new(total: usize) -> Self {
        Self {
            total,
            folded: Cell::new(0),
            max_chunk: Cell::new(0),
        }
    }
}

/// The canonical byte at position `i` (shared by the source and the reference
/// contiguous fold so the two are guaranteed to agree by construction).
fn pattern_byte(i: usize) -> u8 {
    (i.wrapping_mul(31).wrapping_add(7)) as u8
}

impl ChunkSource for PatternSource {
    fn for_each_chunk(&self, f: &mut dyn FnMut(&[u8])) {
        let mut buf = [0u8; CHUNK_BYTES];
        let mut emitted = 0usize;
        while emitted < self.total {
            let this = core::cmp::min(CHUNK_BYTES, self.total - emitted);
            for (k, slot) in buf.iter_mut().enumerate().take(this) {
                *slot = pattern_byte(emitted + k);
            }
            self.folded.set(self.folded.get() + this);
            if this > self.max_chunk.get() {
                self.max_chunk.set(this);
            }
            f(&buf[..this]);
            emitted += this;
        }
    }

    fn total_bytes(&self) -> Option<usize> {
        Some(self.total)
    }
}

#[test]
fn stream_carrier_folds_multi_mib_payload_without_cap() {
    let source = PatternSource::new(TOTAL_BYTES);
    let value: TermValue<'_, N> = TermValue::stream(&source);

    // Property 1 — no cap: the carrier reports the full payload length, far
    // beyond the retired 4096-byte ceiling, and `N` (the inline width) is
    // unrelated to and dwarfed by the payload.
    assert_eq!(value.len_hint(), Some(TOTAL_BYTES));
    const {
        assert!(
            TOTAL_BYTES > 4096 * 2000,
            "payload must exceed the retired 4096-byte cap by orders of magnitude"
        )
    };
    const {
        assert!(
            N < CHUNK_BYTES,
            "inline carrier width N must be tiny next to the streamed payload"
        )
    };

    // σ-projection: fold the stream through the application's Hasher
    // chunk-by-chunk. This is the canonical streaming digest path.
    let mut streamed = <Fnv1aHasher32 as Hasher>::initial();
    value.for_each_chunk(&mut |chunk| {
        streamed = streamed.fold_bytes(chunk);
    });
    let streamed_digest = streamed.finalize();

    // Property 3 — bounded memory: every chunk handed to the fold was ≤ the
    // fixed scratch size, and the whole payload was visited.
    assert_eq!(
        source.folded.get(),
        TOTAL_BYTES,
        "the fold must visit every byte of the canonical sequence"
    );
    assert!(
        source.max_chunk.get() <= CHUNK_BYTES,
        "peak resident chunk {} exceeded the fixed scratch size {CHUNK_BYTES}",
        source.max_chunk.get()
    );

    // Property 2 — losslessness: the same logical byte sequence folded as a
    // single contiguous stream yields a bit-identical digest. (Computed here
    // byte-by-byte from the same `pattern_byte` generator, so the reference
    // also allocates no full buffer.)
    let mut reference = <Fnv1aHasher32 as Hasher>::initial();
    for i in 0..TOTAL_BYTES {
        reference = reference.fold_byte(pattern_byte(i));
    }
    let reference_digest = reference.finalize();

    assert_eq!(
        streamed_digest, reference_digest,
        "streamed chunk-by-chunk fold must equal the contiguous fold — the \
         source-polymorphic carrier introduces no truncation or padding"
    );
}

#[test]
fn stream_carrier_as_slice_is_none_but_for_each_chunk_streams() {
    // The `Stream` variant is intentionally not a contiguous slice: `as_slice`
    // returns `None` and `bytes()` the empty prefix, forcing consumers onto the
    // streaming reader. This is what keeps resident memory bounded.
    let source = PatternSource::new(CHUNK_BYTES * 3 + 17);
    let value: TermValue<'_, N> = TermValue::stream(&source);

    assert!(
        value.as_slice().is_none(),
        "Stream is not a contiguous slice"
    );
    assert!(value.bytes().is_empty(), "Stream has no inline prefix");

    let mut seen = 0usize;
    value.for_each_chunk(&mut |chunk| seen += chunk.len());
    assert_eq!(
        seen,
        CHUNK_BYTES * 3 + 17,
        "for_each_chunk must stream the full (non-chunk-aligned) payload"
    );
}
