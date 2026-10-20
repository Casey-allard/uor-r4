//! Behavioral contract for `Embed<From, To>::apply` — the upward level
//! coercion surface.
//!
//! Target §4.4: `Embed<From, To>` exists only for `(From, To)` pairs in
//! `ValidLevelEmbedding`. For every valid pair, `apply(value)` must
//! preserve the numeric value (zero-extended into the wider ring). A
//! regression where the embedding truncates, sign-extends, or masks
//! incorrectly would alter numeric content and fail here.
//!
//! The test covers representative (source_level, target_level) pairs
//! across native widths. Every shipped pair uses `value as <wider>` with
//! zero-extension semantics.

use uor_foundation::enforcement::{Embed, W128, W16, W24, W32, W40, W64, W8};

// ─── W8 → wider preserves value ────────────────────────────────────────

#[test]
fn embed_w8_to_w8_is_identity() {
    for x in 0u8..=255 {
        assert_eq!(Embed::<W8, W8>::apply(x), x);
    }
}

#[test]
fn embed_w8_to_w16_zero_extends() {
    for &x in &[0u8, 1, 42, 128, 200, 255] {
        assert_eq!(Embed::<W8, W16>::apply(x), x as u16);
    }
}

#[test]
fn embed_w8_to_w32_zero_extends() {
    for &x in &[0u8, 1, 42, 128, 200, 255] {
        assert_eq!(Embed::<W8, W32>::apply(x), x as u32);
    }
}

#[test]
fn embed_w8_to_w64_zero_extends() {
    for &x in &[0u8, 1, 42, 128, 200, 255] {
        assert_eq!(Embed::<W8, W64>::apply(x), x as u64);
    }
}

#[test]
fn embed_w8_to_w128_zero_extends() {
    for &x in &[0u8, 1, 42, 128, 200, 255] {
        assert_eq!(Embed::<W8, W128>::apply(x), x as u128);
    }
}

// ─── W16 → wider preserves value ───────────────────────────────────────

#[test]
fn embed_w16_to_w16_is_identity() {
    for &x in &[0u16, 1, 100, 32767, 32768, 65535] {
        assert_eq!(Embed::<W16, W16>::apply(x), x);
    }
}

#[test]
fn embed_w16_to_w32_zero_extends() {
    for &x in &[0u16, 1, 100, 32767, 32768, 65535] {
        assert_eq!(Embed::<W16, W32>::apply(x), x as u32);
    }
}

#[test]
fn embed_w16_to_w64_zero_extends() {
    for &x in &[0u16, 1, 100, 32767, 32768, 65535] {
        assert_eq!(Embed::<W16, W64>::apply(x), x as u64);
    }
}

// ─── W24 → wider preserves value ───────────────────────────────────────

#[test]
fn embed_w24_to_w32_masks_to_24_bits() {
    // W24 values live in a u32 but the source is guaranteed to be masked.
    // The embedding widens the ring but the numeric value is preserved.
    for &x in &[0u32, 1, 0xFF_FFFF, 0x80_0000] {
        let embedded = Embed::<W24, W32>::apply(x);
        assert_eq!(embedded, x, "W24\u{2192}W32 must preserve the masked value");
    }
}

// ─── W32 → wider preserves value ───────────────────────────────────────

#[test]
fn embed_w32_to_w64_zero_extends() {
    for &x in &[0u32, 1, u32::MAX / 2, u32::MAX] {
        assert_eq!(Embed::<W32, W64>::apply(x), x as u64);
    }
}

#[test]
fn embed_w32_to_w128_zero_extends() {
    for &x in &[0u32, 1, u32::MAX / 2, u32::MAX] {
        assert_eq!(Embed::<W32, W128>::apply(x), x as u128);
    }
}

// ─── W40 → wider preserves value ───────────────────────────────────────

#[test]
fn embed_w40_to_w64_preserves_value() {
    let max_w40: u64 = (1u64 << 40) - 1;
    for &x in &[0u64, 1, 12345, max_w40] {
        assert_eq!(Embed::<W40, W64>::apply(x), x);
    }
}

// ─── W64 → W128 preserves value ───────────────────────────────────────

#[test]
fn embed_w64_to_w128_zero_extends() {
    for &x in &[0u64, 1, u64::MAX / 2, u64::MAX] {
        assert_eq!(Embed::<W64, W128>::apply(x), x as u128);
    }
}
