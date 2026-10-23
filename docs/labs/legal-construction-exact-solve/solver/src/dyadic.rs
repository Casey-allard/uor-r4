//! Bounded-precision dyadic numbers for the basis search (#2828, claude-54).
//!
//! A `Dy<P>` is `m · 2^e` with an `i128` mantissa of at most `P` significant
//! bits (`8 <= P <= 120`, default 120) and an `i32` exponent. Every `add`,
//! `sub`, `mul` and `div` computes the exact result in a 256-bit integer
//! intermediate (division: long division to at least `P + 2` quotient bits
//! plus a sticky remainder bit) and rounds once to nearest, ties to even.
//! The relative error of each operation is therefore at most `2^-P`.
//! `from_bigint` and `from_rational` round the same way. Values are stored
//! canonically (odd mantissa, or zero with exponent 0), so `==` and `cmp`
//! are exact comparisons of the stored values. There is no binary floating
//! point here: only integers.

use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Neg, Sub};

use num_bigint::{BigInt, BigUint, Sign};
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Zero};

/// Unsigned 256-bit intermediate, `hi·2^128 + lo`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct U256 {
    hi: u128,
    lo: u128,
}

const LOW64: u128 = (1u128 << 64) - 1;

impl U256 {
    const ZERO: U256 = U256 { hi: 0, lo: 0 };

    fn from_u128(lo: u128) -> Self {
        U256 { hi: 0, lo }
    }

    fn is_zero(self) -> bool {
        self.hi == 0 && self.lo == 0
    }

    fn bits(self) -> u32 {
        if self.hi != 0 {
            256 - self.hi.leading_zeros()
        } else {
            128 - self.lo.leading_zeros()
        }
    }

    fn bit(self, k: u32) -> bool {
        if k >= 128 {
            (self.hi >> (k - 128)) & 1 == 1
        } else {
            (self.lo >> k) & 1 == 1
        }
    }

    fn set_bit(&mut self, k: u32) {
        if k >= 128 {
            self.hi |= 1 << (k - 128);
        } else {
            self.lo |= 1 << k;
        }
    }

    fn mul(a: u128, b: u128) -> Self {
        let (a1, a0) = (a >> 64, a & LOW64);
        let (b1, b0) = (b >> 64, b & LOW64);
        let (p00, p01, p10, p11) = (a0 * b0, a0 * b1, a1 * b0, a1 * b1);
        let (mid, c1) = p01.overflowing_add(p10);
        let (lo, c2) = p00.overflowing_add(mid << 64);
        let hi = p11 + (mid >> 64) + (u128::from(c1) << 64) + u128::from(c2);
        U256 { hi, lo }
    }

    /// `self << k`; the caller guarantees nothing is shifted out.
    fn shl(self, k: u32) -> Self {
        match k {
            0 => self,
            1..=127 => U256 {
                hi: (self.hi << k) | (self.lo >> (128 - k)),
                lo: self.lo << k,
            },
            128..=255 => U256 {
                hi: self.lo << (k - 128),
                lo: 0,
            },
            _ => U256::ZERO,
        }
    }

    /// `self >> k`, and whether any bit shifted out was set.
    fn shr_sticky(self, k: u32) -> (Self, bool) {
        match k {
            0 => (self, false),
            1..=127 => (
                U256 {
                    hi: self.hi >> k,
                    lo: (self.lo >> k) | (self.hi << (128 - k)),
                },
                self.lo & ((1u128 << k) - 1) != 0,
            ),
            128..=255 => (
                U256::from_u128(self.hi >> (k - 128)),
                self.lo != 0 || self.hi & ((1u128 << (k - 128)) - 1) != 0,
            ),
            _ => (U256::ZERO, !self.is_zero()),
        }
    }

    fn add(self, o: Self) -> Self {
        let (lo, c) = self.lo.overflowing_add(o.lo);
        U256 {
            hi: self.hi + o.hi + u128::from(c),
            lo,
        }
    }

    /// `self − o` for `self >= o`.
    fn sub(self, o: Self) -> Self {
        let (lo, b) = self.lo.overflowing_sub(o.lo);
        U256 {
            hi: self.hi - o.hi - u128::from(b),
            lo,
        }
    }

    /// Long division by `d` (`0 < d < 2^127`): the quotient, and whether the
    /// remainder is non-zero.
    fn div_u128(self, d: u128) -> (Self, bool) {
        let mut q = U256::ZERO;
        let mut rem: u128 = 0;
        for k in (0..self.bits()).rev() {
            rem = (rem << 1) | u128::from(self.bit(k));
            if rem >= d {
                rem -= d;
                q.set_bit(k);
            }
        }
        (q, rem != 0)
    }

    /// From a `BigUint` of at most 256 bits.
    fn from_biguint(v: &BigUint) -> Self {
        let mut limbs = v.to_u64_digits();
        limbs.resize(4, 0);
        U256 {
            lo: u128::from(limbs[0]) | (u128::from(limbs[1]) << 64),
            hi: u128::from(limbs[2]) | (u128::from(limbs[3]) << 64),
        }
    }
}

/// `m · 2^e`, `|m| < 2^P`, canonical (odd `m`, or `m = 0` and `e = 0`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Dy<const P: u32 = 120> {
    m: i128,
    e: i32,
}

impl<const P: u32> Default for Dy<P> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const P: u32> Dy<P> {
    pub const ZERO: Self = Dy { m: 0, e: 0 };
    pub const ONE: Self = Dy { m: 1, e: 0 };
    const VALID: () = assert!(P >= 8 && P <= 120, "Dy precision must be 8..=120 bits");

    /// Round `(−1)^neg · (mag + f) · 2^exp`, where `0 < f < 1` exactly when
    /// `sticky`, to `P` bits, nearest, ties to even.
    fn round(neg: bool, mag: U256, exp: i64, sticky: bool) -> Self {
        #[allow(clippy::let_unit_value)]
        let () = Self::VALID;
        if mag.is_zero() {
            assert!(!sticky, "Dy: a sticky remainder below a zero mantissa");
            return Self::ZERO;
        }
        let (mut mag, mut exp) = (mag, exp);
        let b = mag.bits();
        if sticky && b <= P + 1 {
            // Make room so that the sticky fraction lies below the round bit.
            let k = P + 2 - b;
            mag = mag.shl(k);
            exp -= i64::from(k);
        }
        let b = mag.bits();
        let (mut m, mut e) = if b <= P {
            (mag.lo, exp)
        } else {
            let k = b - P;
            let (kept, low) = mag.shr_sticky(k - 1);
            let round_bit = kept.lo & 1 == 1;
            let mut m = kept.lo >> 1;
            if round_bit && (low || sticky || m & 1 == 1) {
                m += 1;
            }
            let mut e = exp + i64::from(k);
            if m >> P != 0 {
                m >>= 1;
                e += 1;
            }
            (m, e)
        };
        let tz = m.trailing_zeros();
        m >>= tz;
        e += i64::from(tz);
        let e = i32::try_from(e).expect("Dy exponent out of the i32 range");
        let m = m as i128;
        Dy {
            m: if neg { -m } else { m },
            e,
        }
    }

    fn parts(self) -> (bool, u128, i64) {
        (self.m < 0, self.m.unsigned_abs(), i64::from(self.e))
    }

    /// Position one above the top set bit of `|self|` (`e + bits(m)`).
    fn top(self) -> i64 {
        i64::from(self.e) + i64::from(128 - self.m.unsigned_abs().leading_zeros())
    }

    pub fn from_i128(v: i128) -> Self {
        Self::round(v < 0, U256::from_u128(v.unsigned_abs()), 0, false)
    }

    pub fn from_i64(v: i64) -> Self {
        Self::from_i128(i128::from(v))
    }

    pub fn from_bigint(v: &BigInt) -> Self {
        let (sign, mag) = (v.sign(), v.magnitude());
        if sign == Sign::NoSign {
            return Self::ZERO;
        }
        let bits = mag.bits();
        let (mag, exp, sticky) = if bits > 250 {
            let k = bits - 250;
            let sticky = mag.trailing_zeros().is_some_and(|t| t < k);
            (mag >> k, k as i64, sticky)
        } else {
            (mag.clone(), 0, false)
        };
        Self::round(sign == Sign::Minus, U256::from_biguint(&mag), exp, sticky)
    }

    pub fn from_rational(v: &BigRational) -> Self {
        if v.is_zero() {
            return Self::ZERO;
        }
        let (n, d) = (v.numer().magnitude(), v.denom().magnitude());
        let s = i64::from(P) + 3 + d.bits() as i64 - n.bits() as i64;
        let (num, den) = if s >= 0 {
            (n << s as u64, d.clone())
        } else {
            (n.clone(), d << (-s) as u64)
        };
        let (q, r) = num.div_rem(&den);
        Self::round(
            v.numer().sign() == Sign::Minus,
            U256::from_biguint(&q),
            -s,
            !r.is_zero(),
        )
    }

    /// The exact stored value.
    pub fn to_rational(self) -> BigRational {
        let m = BigInt::from(self.m);
        if self.e >= 0 {
            BigRational::from_integer(m << self.e as u32)
        } else {
            BigRational::new(m, BigInt::one() << (-i64::from(self.e)) as u32)
        }
    }

    pub fn mantissa(self) -> i128 {
        self.m
    }

    pub fn exponent(self) -> i32 {
        self.e
    }

    pub fn is_zero(self) -> bool {
        self.m == 0
    }

    /// −1, 0 or 1.
    pub fn sign(self) -> i8 {
        self.m.signum() as i8
    }

    pub fn is_negative(self) -> bool {
        self.m < 0
    }

    pub fn is_positive(self) -> bool {
        self.m > 0
    }

    pub fn abs(self) -> Self {
        Dy {
            m: self.m.abs(),
            e: self.e,
        }
    }

    /// `self · 2^k`, exact.
    pub fn ldexp(self, k: i32) -> Self {
        if self.is_zero() {
            self
        } else {
            Dy {
                m: self.m,
                e: self.e + k,
            }
        }
    }

    fn cmp_mag(self, o: Self) -> Ordering {
        let (ta, tb) = (self.top(), o.top());
        if ta != tb {
            return ta.cmp(&tb);
        }
        let (_, ma, ea) = self.parts();
        let (_, mb, eb) = o.parts();
        let e0 = ea.min(eb);
        U256::from_u128(ma)
            .shl((ea - e0) as u32)
            .cmp(&U256::from_u128(mb).shl((eb - e0) as u32))
    }

    fn add_signed(self, o: Self) -> Self {
        if self.is_zero() {
            return o;
        }
        if o.is_zero() {
            return self;
        }
        // One operand below a quarter ulp of the other: the sum rounds to it.
        let gap = self.top() - o.top();
        if gap > i64::from(P) + 2 {
            return self;
        }
        if -gap > i64::from(P) + 2 {
            return o;
        }
        let (na, ma, ea) = self.parts();
        let (nb, mb, eb) = o.parts();
        let e0 = ea.min(eb);
        let a = U256::from_u128(ma).shl((ea - e0) as u32);
        let b = U256::from_u128(mb).shl((eb - e0) as u32);
        let (neg, mag) = if na == nb {
            (na, a.add(b))
        } else if a >= b {
            (na, a.sub(b))
        } else {
            (nb, b.sub(a))
        };
        Self::round(neg, mag, e0, false)
    }

    fn mul_rounded(self, o: Self) -> Self {
        if self.is_zero() || o.is_zero() {
            return Self::ZERO;
        }
        let (na, ma, ea) = self.parts();
        let (nb, mb, eb) = o.parts();
        Self::round(na != nb, U256::mul(ma, mb), ea + eb, false)
    }

    fn div_rounded(self, o: Self) -> Self {
        assert!(!o.is_zero(), "Dy: division by zero");
        if self.is_zero() {
            return Self::ZERO;
        }
        let (na, ma, ea) = self.parts();
        let (nb, mb, eb) = o.parts();
        let (ba, bb) = (128 - ma.leading_zeros(), 128 - mb.leading_zeros());
        let s = (P + 3 + bb).saturating_sub(ba);
        let (q, sticky) = U256::from_u128(ma).shl(s).div_u128(mb);
        Self::round(na != nb, q, ea - eb - i64::from(s), sticky)
    }
}

impl<const P: u32> Ord for Dy<P> {
    fn cmp(&self, o: &Self) -> Ordering {
        match (self.sign(), o.sign()) {
            (a, b) if a != b => a.cmp(&b),
            (0, _) => Ordering::Equal,
            (1, _) => self.cmp_mag(*o),
            _ => o.cmp_mag(*self),
        }
    }
}

impl<const P: u32> PartialOrd for Dy<P> {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl<const P: u32> Neg for Dy<P> {
    type Output = Self;
    fn neg(self) -> Self {
        Dy {
            m: -self.m,
            e: self.e,
        }
    }
}

impl<const P: u32> Add for Dy<P> {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        self.add_signed(o)
    }
}

impl<const P: u32> Sub for Dy<P> {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        self.add_signed(-o)
    }
}

impl<const P: u32> Mul for Dy<P> {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        self.mul_rounded(o)
    }
}

impl<const P: u32> Div for Dy<P> {
    type Output = Self;
    fn div(self, o: Self) -> Self {
        self.div_rounded(o)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Signed;

    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        }
        /// A random non-zero integer of up to `bits` bits, either sign.
        fn big(&mut self, bits: u32) -> BigInt {
            let mut v = BigInt::zero();
            for _ in 0..bits.div_ceil(64) {
                v = (v << 64) + BigInt::from(self.next());
            }
            let keep = 1 + (self.next() % u64::from(bits)) as u32;
            v %= BigInt::one() << keep;
            if v.is_zero() {
                v = BigInt::one();
            }
            if self.next() & 1 == 1 {
                -v
            } else {
                v
            }
        }
        fn rational(&mut self) -> BigRational {
            let n = self.big(300);
            let mut d = self.big(200).abs();
            if self.next().is_multiple_of(3) {
                d = BigInt::one() << (self.next() % 90) as u32;
            }
            BigRational::new(n, d)
        }
    }

    /// `|got − want| <= 2^-P · |want|` (one rounding to nearest).
    fn within<const P: u32>(got: Dy<P>, want: &BigRational, what: &str) {
        let err = (got.to_rational() - want).abs();
        let bound = want.abs() / BigRational::from_integer(BigInt::one() << P);
        assert!(err <= bound, "{what}: error {err} above 2^-{P}·|{want}|");
    }

    fn random_dy<const P: u32>(rng: &mut Rng) -> Dy<P> {
        Dy::from_rational(&rng.rational())
    }

    #[test]
    fn small_integers_are_exact() {
        for a in -60i64..=60 {
            for b in -60i64..=60 {
                let (x, y) = (Dy::<120>::from_i64(a), Dy::<120>::from_i64(b));
                assert_eq!(x + y, Dy::from_i64(a + b), "{a}+{b}");
                assert_eq!(x - y, Dy::from_i64(a - b), "{a}-{b}");
                assert_eq!(x * y, Dy::from_i64(a * b), "{a}*{b}");
                if b != 0 {
                    assert_eq!(Dy::<120>::from_i64(a * b) / y, x, "{a}*{b}/{b}");
                    within(x / y, &BigRational::new(a.into(), b.into()), "a/b");
                }
                assert_eq!(x.cmp(&y), a.cmp(&b), "cmp {a} {b}");
                assert_eq!(x.to_rational(), BigRational::from_integer(a.into()));
            }
        }
    }

    fn error_bounds<const P: u32>(seed: u64) {
        let mut rng = Rng(seed);
        for i in 0..3000 {
            let q = rng.rational();
            within(
                Dy::<P>::from_rational(&q),
                &q,
                &format!("from_rational {i}"),
            );
            let z = rng.big(400);
            within(
                Dy::<P>::from_bigint(&z),
                &BigRational::from_integer(z.clone()),
                &format!("from_bigint {i}"),
            );
            let (a, b) = (random_dy::<P>(&mut rng), random_dy::<P>(&mut rng));
            let (ra, rb) = (a.to_rational(), b.to_rational());
            within(a * b, &(&ra * &rb), &format!("mul {i}"));
            within(a / b, &(&ra / &rb), &format!("div {i}"));
            within(a + b, &(&ra + &rb), &format!("add {i}"));
            within(a - b, &(&ra - &rb), &format!("sub {i}"));
            // Near-cancellation and far-apart magnitudes.
            let c = a + b.ldexp(-(P as i32) - 40);
            within(
                c,
                &(&ra + rb.clone() / BigRational::from_integer(BigInt::one() << (P + 40))),
                "far add",
            );
            within(a - c, &(&ra - c.to_rational()), &format!("cancel {i}"));
        }
    }

    #[test]
    fn error_bound_at_120_bits() {
        error_bounds::<120>(0xd1ad_0001);
    }

    #[test]
    fn error_bound_at_20_bits() {
        error_bounds::<20>(0xd1ad_0002);
    }

    #[test]
    fn rounding_is_nearest_ties_to_even() {
        type D = Dy<20>;
        let two20 = 1i64 << 20;
        // 2^21 + 1 sits halfway between 2^21 and 2^21 + 2: ties go to even.
        assert_eq!(D::from_i64(2 * two20 + 1), D::from_i64(2 * two20));
        assert_eq!(D::from_i64(2 * two20 + 3), D::from_i64(2 * two20 + 4));
        assert_eq!(D::from_i64(-(2 * two20 + 3)), D::from_i64(-(2 * two20 + 4)));
        // Just above the tie rounds up.
        let q = BigRational::new(BigInt::from(4 * two20 + 2 + 1), BigInt::from(2));
        assert_eq!(D::from_rational(&q), D::from_i64(2 * two20 + 2));
        // 1/3 at 20 bits.
        let third = D::ONE / D::from_i64(3);
        assert_eq!(third.mantissa(), 0xAAAAB);
        assert_eq!(third.exponent(), -21);
        // Carry into a new bit.
        assert_eq!(
            D::from_i64(two20 - 1) + D::from_i64(two20),
            D::from_i64(2 * two20)
        );
    }

    #[test]
    fn ordering_matches_the_rationals() {
        let mut rng = Rng(0xd1ad_0003);
        let mut vals: Vec<Dy<120>> = (0..400).map(|_| random_dy(&mut rng)).collect();
        vals.push(Dy::ZERO);
        vals.push(vals[3]);
        vals.push(-vals[5]);
        vals.push(vals[7].ldexp(3));
        for a in &vals {
            for b in &vals {
                assert_eq!(a.cmp(b), a.to_rational().cmp(&b.to_rational()));
                assert_eq!(a == b, a.to_rational() == b.to_rational());
            }
            assert_eq!(a.is_zero(), a.to_rational().is_zero());
            assert_eq!(
                i32::from(a.sign()),
                match a.to_rational().cmp(&BigRational::zero()) {
                    Ordering::Less => -1,
                    Ordering::Equal => 0,
                    Ordering::Greater => 1,
                }
            );
        }
    }
}
