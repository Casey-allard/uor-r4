//! The exact problem: masters, objective vector and guard rows.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};

#[derive(Debug, Clone)]
pub struct Problem {
    /// Master values `m_i`.
    pub masters: Vec<BigRational>,
    /// Objective vector `g`: minimise `g·δ`.
    pub g: Vec<BigRational>,
    /// Guard rows `J_j`: each must satisfy `J_j·δ >= 0`.
    pub guards: Vec<Vec<BigRational>>,
}

/// Derived per-coefficient data.
#[derive(Debug, Clone)]
pub struct Coeff {
    /// Current code `round_ties_away(4m)`, clamped to [-7, 7].
    pub q0: i64,
    /// True when `4m` is not an integer.
    pub fractional: bool,
    /// `q0/4 - m` for a fractional coefficient, 0 on grid.
    pub shift: BigRational,
}

impl Problem {
    pub fn new(
        masters: Vec<BigRational>,
        g: Vec<BigRational>,
        guards: Vec<Vec<BigRational>>,
    ) -> Self {
        assert_eq!(masters.len(), g.len(), "masters and g differ in length");
        for row in &guards {
            assert_eq!(row.len(), masters.len(), "guard row length differs from N");
        }
        Problem { masters, g, guards }
    }

    pub fn n(&self) -> usize {
        self.masters.len()
    }

    pub fn coeffs(&self) -> Vec<Coeff> {
        let four = BigRational::from_integer(BigInt::from(4));
        self.masters
            .iter()
            .map(|m| {
                let t = m * &four;
                let near = round_ties_away(&t);
                let q0 = if near > BigInt::from(7) {
                    7
                } else if near < BigInt::from(-7) {
                    -7
                } else {
                    near.to_i64().expect("value in [-7, 7]")
                };
                let fractional = !t.is_integer();
                let shift = if fractional {
                    BigRational::new(BigInt::from(q0), BigInt::from(4)) - m
                } else {
                    BigRational::zero()
                };
                Coeff {
                    q0,
                    fractional,
                    shift,
                }
            })
            .collect()
    }
}

/// Round to the nearest integer, halves away from zero.
pub fn round_ties_away(x: &BigRational) -> BigInt {
    let half = BigRational::new(BigInt::from(1), BigInt::from(2));
    if x.is_negative() {
        -((-x + &half).floor().to_integer())
    } else {
        (x + &half).floor().to_integer()
    }
}
