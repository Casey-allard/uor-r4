//! Independent exact re-check of an assignment.
//!
//! Written apart from the solver: it derives `q0` with integer division,
//! decides legality from the move semantics (keep / lower / higher) rather
//! than from the LP rows, and computes `δ` from the new code value
//! `q/4 - m` rather than from the solver's linear form.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

use crate::bb::Assignment;
use crate::problem::Problem;

#[derive(Debug, Clone)]
pub struct CheckReport {
    /// Every coefficient's (k, l, r) is a legal keep/lower/higher move.
    pub legal: bool,
    pub delta: Vec<BigRational>,
    /// `J_j·δ` for every guard row.
    pub guard_values: Vec<BigRational>,
    /// Every guard value is >= 0.
    pub guards_hold: bool,
    /// `g·δ`.
    pub objective: BigRational,
}

/// Nearest integer to `n/d` (d > 0), halves away from zero.
fn nearest(t: &BigRational) -> BigInt {
    let n = t.numer();
    let d = t.denom();
    let mag = (n * 2i32).abs() + d;
    let q = mag / (d * 2i32);
    if n.is_negative() {
        -q
    } else {
        q
    }
}

pub fn check(p: &Problem, a: &Assignment) -> CheckReport {
    let n = p.n();
    assert_eq!(a.k.len(), n);
    assert_eq!(a.l.len(), n);
    assert_eq!(a.r.len(), n);
    let four = BigInt::from(4);
    let seven = BigInt::from(7);
    let mut legal = true;
    let mut delta = Vec::with_capacity(n);

    for i in 0..n {
        let m = &p.masters[i];
        let t = m * BigRational::from_integer(four.clone());
        let q0 = nearest(&t).clamp(-seven.clone(), seven.clone());
        let k = &a.k[i];
        let q = &q0 + k;
        let in_range = q >= -seven.clone() && q <= seven;
        let (ok, d) = if t.is_integer() {
            (in_range && !a.l[i] && !a.r[i], BigRational::new(k.clone(), four.clone()))
        } else {
            let moved = BigRational::new(q.clone(), four.clone()) - m;
            match (a.l[i], a.r[i]) {
                (false, false) => (k.is_zero(), BigRational::zero()),
                (true, false) => (k.is_negative() && in_range, moved),
                (false, true) => (k.is_positive() && in_range, moved),
                (true, true) => (false, BigRational::zero()),
            }
        };
        legal &= ok;
        delta.push(d);
    }

    let dot = |row: &[BigRational]| {
        row.iter().zip(&delta).fold(BigRational::zero(), |s, (x, y)| s + x * y)
    };
    let guard_values: Vec<BigRational> = p.guards.iter().map(|row| dot(row)).collect();
    let guards_hold = guard_values.iter().all(|v| !v.is_negative());
    let objective = dot(&p.g);
    CheckReport { legal, delta, guard_values, guards_hold, objective }
}
