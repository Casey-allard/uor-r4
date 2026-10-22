//! Brute-force oracle: enumerate every legal (k, l, r) for small N.
//!
//! On-grid coefficient: k over its full range, so every q in [-7, 7].
//! Fractional coefficient: keep (k = 0), lower (l = 1, q in [-7, q0 - 1]) or
//! higher (r = 1, q in [q0 + 1, 7]). Contributions are scaled by the least
//! common denominator to exact integers, so the search adds integers only.

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};

use crate::bb::Assignment;
use crate::problem::Problem;

#[derive(Debug, Clone)]
pub struct BruteResult {
    pub objective: BigRational,
    pub assignment: Assignment,
    /// Number of legal assignments that keep every guard.
    pub feasible: u64,
}

struct Choice {
    k: i64,
    l: bool,
    r: bool,
}

struct Search<'a> {
    ints: &'a [Vec<Vec<i128>>],
    acc: Vec<i128>,
    pick: Vec<usize>,
    best: Option<(i128, Vec<usize>)>,
    feasible: u64,
}

impl Search<'_> {
    fn dfs(&mut self, i: usize) {
        if i == self.ints.len() {
            if self.acc[1..].iter().all(|&v| v >= 0) {
                self.feasible += 1;
                let better = match &self.best {
                    None => true,
                    Some((b, _)) => self.acc[0] < *b,
                };
                if better {
                    self.best = Some((self.acc[0], self.pick.clone()));
                }
            }
            return;
        }
        let ints = self.ints;
        for (ci, v) in ints[i].iter().enumerate() {
            for (a, x) in self.acc.iter_mut().zip(v) {
                *a += x;
            }
            self.pick[i] = ci;
            self.dfs(i + 1);
            for (a, x) in self.acc.iter_mut().zip(v) {
                *a -= x;
            }
        }
    }
}

pub fn solve(p: &Problem) -> BruteResult {
    let n = p.n();
    let cs = p.coeffs();
    let quarter = |k: i64| BigRational::new(BigInt::from(k), BigInt::from(4));

    let mut choices: Vec<Vec<(Choice, BigRational)>> = Vec::with_capacity(n);
    for c in &cs {
        let (lo, hi) = (-7 - c.q0, 7 - c.q0);
        let mut v = Vec::new();
        if c.fractional {
            v.push((Choice { k: 0, l: false, r: false }, BigRational::zero()));
            for k in lo..=-1 {
                v.push((Choice { k, l: true, r: false }, quarter(k) + &c.shift));
            }
            for k in 1..=hi {
                v.push((Choice { k, l: false, r: true }, quarter(k) + &c.shift));
            }
        } else {
            for k in lo..=hi {
                v.push((Choice { k, l: false, r: false }, quarter(k)));
            }
        }
        choices.push(v);
    }

    // contrib[i][choice] = [g_i δ, J_0i δ, J_1i δ, ...]
    let contrib: Vec<Vec<Vec<BigRational>>> = (0..n)
        .map(|i| {
            choices[i]
                .iter()
                .map(|(_, d)| {
                    std::iter::once(&p.g[i] * d).chain(p.guards.iter().map(|row| &row[i] * d)).collect()
                })
                .collect()
        })
        .collect();
    let mut scale = BigInt::one();
    for v in contrib.iter().flatten().flatten() {
        scale = scale.lcm(v.denom());
    }
    let ints: Vec<Vec<Vec<i128>>> = contrib
        .iter()
        .map(|ch| {
            ch.iter()
                .map(|vals| {
                    vals.iter()
                        .map(|v| (v.numer() * (&scale / v.denom())).to_i128().expect("scaled value fits i128"))
                        .collect()
                })
                .collect()
        })
        .collect();

    let mut s = Search { ints: &ints, acc: vec![0; p.guards.len() + 1], pick: vec![0; n], best: None, feasible: 0 };
    s.dfs(0);
    let (best, pick) = s.best.expect("δ = 0 is always feasible");
    let mut a = Assignment::zero(n);
    for i in 0..n {
        let ch = &choices[i][pick[i]].0;
        a.k[i] = BigInt::from(ch.k);
        a.l[i] = ch.l;
        a.r[i] = ch.r;
    }
    BruteResult { objective: BigRational::new(BigInt::from(best), scale), assignment: a, feasible: s.feasible }
}
