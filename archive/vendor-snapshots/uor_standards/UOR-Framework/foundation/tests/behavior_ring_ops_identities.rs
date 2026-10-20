//! Behavioral contract for the phantom-typed `RingOp<L>` / `UnaryRingOp<L>`
//! surface.
//!
//! Target §4.4: each `WittLevel` marker `L` has phantom-typed `Add<L>`,
//! `Sub<L>`, `Mul<L>`, `Xor<L>`, `And<L>`, `Or<L>` (binary) and `Neg<L>`,
//! `BNot<L>`, `Succ<L>` (unary) implementing `RingOp<L>` / `UnaryRingOp<L>`
//! respectively. The arithmetic on `Operand = u*` or `Datum<L>` / `Limbs<N>`
//! backing must match the modular ring semantics `(op) mod 2^bits`.
//!
//! Ring identities tested:
//! - `Neg(BNot(x)) = Succ(x)` — fundamental identity of two's complement.
//! - `Add(x, y) = Add(y, x)` — commutativity.
//! - `Add(x, 0) = x` — additive identity.
//! - `Mul(x, 0) = 0` — annihilator.
//! - `Mul(x, 1) = x` — multiplicative identity.
//! - `Xor(x, x) = 0` — self-annihilation.
//! - `And(x, 0) = 0` — absorbing zero for AND.
//! - `Or(x, 0) = x` — identity zero for OR.
//!
//! A regression where the phantom-typed op's `apply` method delegates to
//! the wrong `const_ring_eval_w{n}` or skips the mask would fail here.

use uor_foundation::enforcement::{
    Add, And, BNot, Mul, Neg, Or, RingOp, Sub, Succ, UnaryRingOp, Xor, W128, W16, W32, W64, W8,
};

// ─── W8 identities ──────────────────────────────────────────────────────

#[test]
fn w8_add_identity_and_commutativity() {
    // Add(x, 0) = x
    assert_eq!(<Add<W8> as RingOp<W8>>::apply(42, 0), 42);
    // Add(x, y) = Add(y, x)
    assert_eq!(
        <Add<W8> as RingOp<W8>>::apply(100, 50),
        <Add<W8> as RingOp<W8>>::apply(50, 100)
    );
}

#[test]
fn w8_mul_identity_and_annihilator() {
    assert_eq!(<Mul<W8> as RingOp<W8>>::apply(13, 1), 13);
    assert_eq!(<Mul<W8> as RingOp<W8>>::apply(13, 0), 0);
}

#[test]
fn w8_xor_self_annihilates() {
    for x in 0u8..=255 {
        assert_eq!(<Xor<W8> as RingOp<W8>>::apply(x, x), 0);
    }
}

#[test]
fn w8_and_or_identities() {
    for x in 0u8..=255 {
        assert_eq!(<And<W8> as RingOp<W8>>::apply(x, 0), 0);
        assert_eq!(<Or<W8> as RingOp<W8>>::apply(x, 0), x);
    }
}

#[test]
fn w8_neg_bnot_succ_identity() {
    // neg(bnot(x)) = succ(x) for all x in Z/256Z — two's-complement identity.
    for x in 0u8..=255 {
        let bnot_x = <BNot<W8> as UnaryRingOp<W8>>::apply(x);
        let lhs = <Neg<W8> as UnaryRingOp<W8>>::apply(bnot_x);
        let rhs = <Succ<W8> as UnaryRingOp<W8>>::apply(x);
        assert_eq!(lhs, rhs, "neg(bnot({x})) must equal succ({x}) at W8");
    }
}

// ─── W16 identities ─────────────────────────────────────────────────────

#[test]
fn w16_add_commutes_and_wraps() {
    assert_eq!(
        <Add<W16> as RingOp<W16>>::apply(65535, 1),
        0,
        "W16 Add must wrap at 2^16"
    );
    assert_eq!(
        <Add<W16> as RingOp<W16>>::apply(30000, 20000),
        <Add<W16> as RingOp<W16>>::apply(20000, 30000)
    );
}

#[test]
fn w16_sub_is_modular() {
    assert_eq!(<Sub<W16> as RingOp<W16>>::apply(0, 1), 65535);
}

#[test]
fn w16_neg_bnot_succ_identity_at_boundaries() {
    for &x in &[0u16, 1, 2, 32767, 32768, 65534, 65535] {
        let bnot_x = <BNot<W16> as UnaryRingOp<W16>>::apply(x);
        let lhs = <Neg<W16> as UnaryRingOp<W16>>::apply(bnot_x);
        let rhs = <Succ<W16> as UnaryRingOp<W16>>::apply(x);
        assert_eq!(lhs, rhs);
    }
}

// ─── W32 identities ─────────────────────────────────────────────────────

#[test]
fn w32_wrapping_arithmetic() {
    assert_eq!(<Add<W32> as RingOp<W32>>::apply(u32::MAX, 1), 0);
    assert_eq!(<Sub<W32> as RingOp<W32>>::apply(0, 1), u32::MAX);
    assert_eq!(<Mul<W32> as RingOp<W32>>::apply(u32::MAX, 2), u32::MAX - 1);
}

#[test]
fn w32_neg_bnot_succ_identity_samples() {
    for &x in &[0u32, 1, 12345, u32::MAX / 2, u32::MAX - 1, u32::MAX] {
        let bnot_x = <BNot<W32> as UnaryRingOp<W32>>::apply(x);
        let lhs = <Neg<W32> as UnaryRingOp<W32>>::apply(bnot_x);
        let rhs = <Succ<W32> as UnaryRingOp<W32>>::apply(x);
        assert_eq!(lhs, rhs, "W32 identity at x={x}");
    }
}

// ─── W64 identities ─────────────────────────────────────────────────────

#[test]
fn w64_wrapping_arithmetic() {
    assert_eq!(<Add<W64> as RingOp<W64>>::apply(u64::MAX, 1), 0);
}

#[test]
fn w64_xor_self_annihilates_samples() {
    for &x in &[0u64, 1, 0xDEAD_BEEF_CAFE_BABE, u64::MAX] {
        assert_eq!(<Xor<W64> as RingOp<W64>>::apply(x, x), 0);
    }
}

// ─── W128 identities ────────────────────────────────────────────────────

#[test]
fn w128_wrapping_arithmetic() {
    assert_eq!(<Add<W128> as RingOp<W128>>::apply(u128::MAX, 1), 0);
    assert_eq!(<Sub<W128> as RingOp<W128>>::apply(0, 1), u128::MAX);
}

#[test]
fn w128_identity_at_boundaries() {
    for &x in &[0u128, 1, u128::MAX / 2, u128::MAX - 1, u128::MAX] {
        let bnot_x = <BNot<W128> as UnaryRingOp<W128>>::apply(x);
        let lhs = <Neg<W128> as UnaryRingOp<W128>>::apply(bnot_x);
        let rhs = <Succ<W128> as UnaryRingOp<W128>>::apply(x);
        assert_eq!(lhs, rhs, "W128 identity at x={x}");
    }
}
