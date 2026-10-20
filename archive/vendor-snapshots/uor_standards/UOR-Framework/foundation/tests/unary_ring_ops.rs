//! v0.2.2 W3 + W17: unary phantom-typed ring op surface.
//!
//! Confirms that the v0.2.2 W3 unary ring ops (`Neg`, `BNot`, `Succ`)
//! are wired in `enforcement::*` and produce the expected ring-arithmetic
//! results. The critical-composition law `Succ = Neg ∘ BNot` is exercised
//! directly so any divergence between the codegen-emitted Succ impl and
//! the composition surfaces as a test failure.

use uor_foundation::enforcement::{
    BNot, Embed, Neg, Succ, UnaryRingOp, ValidLevelEmbedding, W16, W32, W8,
};

#[test]
fn neg_w8_is_modular_negation() {
    // -0 = 0 (mod 256)
    assert_eq!(<Neg<W8> as UnaryRingOp<W8>>::apply(0), 0);
    // -1 = 255 (mod 256)
    assert_eq!(<Neg<W8> as UnaryRingOp<W8>>::apply(1), 255);
    // -42 = 214 (mod 256)
    assert_eq!(<Neg<W8> as UnaryRingOp<W8>>::apply(42), 214);
}

#[test]
fn bnot_w8_is_bitwise_complement() {
    // BNot(0) = 0xFF = 255
    assert_eq!(<BNot<W8> as UnaryRingOp<W8>>::apply(0), 255);
    // BNot(0xFF) = 0
    assert_eq!(<BNot<W8> as UnaryRingOp<W8>>::apply(255), 0);
    // BNot(42) = 213
    assert_eq!(<BNot<W8> as UnaryRingOp<W8>>::apply(42), 213);
}

#[test]
fn succ_w8_satisfies_critical_composition() {
    // Succ(x) = Neg(BNot(x)) = Neg(255 - x) = -(255 - x) = x - 255 = x + 1 (mod 256)
    for x in [0u8, 1, 42, 100, 254, 255] {
        let succ = <Succ<W8> as UnaryRingOp<W8>>::apply(x);
        let expected = x.wrapping_add(1);
        assert_eq!(
            succ, expected,
            "Succ({x}) should equal {expected} (got {succ})"
        );
    }
}

#[test]
fn neg_w32_is_modular_negation() {
    assert_eq!(<Neg<W32> as UnaryRingOp<W32>>::apply(0), 0);
    assert_eq!(<Neg<W32> as UnaryRingOp<W32>>::apply(1), u32::MAX);
    assert_eq!(
        <Neg<W32> as UnaryRingOp<W32>>::apply(0xDEAD_BEEF),
        0u32.wrapping_sub(0xDEAD_BEEF)
    );
}

#[test]
fn embed_w8_to_w32_widens() {
    assert_eq!(Embed::<W8, W32>::apply(0), 0u32);
    assert_eq!(Embed::<W8, W32>::apply(255), 255u32);
}

#[test]
fn embed_w8_to_w16_widens() {
    assert_eq!(Embed::<W8, W16>::apply(0), 0u16);
    assert_eq!(Embed::<W8, W16>::apply(255), 255u16);
}

/// Compile-time witness that `(W8, W32)` is a valid level embedding pair.
const fn require_valid_embedding<P: ValidLevelEmbedding>() {}

#[test]
fn valid_level_embeddings_compile() {
    require_valid_embedding::<(W8, W8)>();
    require_valid_embedding::<(W8, W16)>();
    require_valid_embedding::<(W8, W32)>();
    require_valid_embedding::<(W16, W32)>();
    require_valid_embedding::<(W32, W32)>();
    // Note: `(W32, W8)` is not a valid embedding (lossy projection), and
    // any attempt to use it would fail at compile time. Verifying the
    // negative case requires a `compile_fail` doctest, which lives in
    // the rustdoc on Embed.
}
