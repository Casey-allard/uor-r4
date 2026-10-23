//! Exact bounded-variable revised simplex with an integer-preserving basis
//! inverse, for LPs too large for the dense tableau in `lp`.
//!
//! Same input and output as `lp::solve`: minimise `c·x` subject to
//! `a·x <= b` and `lower <= x <= upper`.
//!
//! * Integral data. Each row `(a, b)` is multiplied by the positive integer
//!   that clears its denominators and then divided by the gcd of the result;
//!   `c` likewise. Positive row scaling keeps the feasible set, and the
//!   objective is reported from the caller's unscaled `c`.
//! * Implicit bounds. Row `i` gets a slack `s_i in [0, inf)`. A nonbasic
//!   variable sits at its lower bound, its upper bound, or (structurals only,
//!   at the start) at an interior start value: 0 when 0 lies strictly inside
//!   its bounds. Interior variables may enter in either direction. The ratio
//!   test includes the entering variable's own bound; when that bound is the
//!   minimum the variable flips to it with no basis change.
//! * Basis inverse. `M = D·B⁻¹` with `D = |det B|`, both integer (`M` is
//!   `±adj B`). With `α = M·A_s` and pivot row `r`: `D' = |α_r|`,
//!   `M'_r = M_r`, `M'_i = (α_r·M_i − α_i·M_r) / D` (Edmonds/Bareiss: the
//!   division is exact; a non-zero remainder panics), and all of `M'` is
//!   negated when `α_r < 0` so that `D` stays positive.
//! * Phase 1. Rows the start point violates get an artificial `t_i >= 0`
//!   with column `−e_i`; phase 1 minimises their sum. A positive certified
//!   minimum means the LP is infeasible. Afterwards the artificials are fixed
//!   to `[0, 0]`; any still basic sit at 0 and are harmless.
//! * Pricing. Dantzig's rule on the integer reduced costs `D·d_j`; after 50
//!   consecutive degenerate pivots, Bland's rule (smallest eligible index,
//!   ratio ties to the smallest basic index) until a strictly improving step.
//!
//! Termination. Every non-degenerate step (step length > 0) strictly lowers
//! the objective, because the entering reduced cost is non-zero in the
//! direction of travel. A flip is never degenerate: a variable is eligible
//! only if its bounds differ, and its own bound is then a positive distance
//! away. Interior variables never return to interior status (a leaving or
//! flipping variable lands on a bound), so after finitely many steps there
//! are none, and the state is a basis plus a bound choice per nonbasic
//! variable: finitely many states, each with a single objective value. A
//! strictly improving step can never revisit a state. A run of degenerate
//! steps either ends within 50 steps or switches to Bland's rule, under which
//! a degenerate run cannot cycle (Bland's argument holds for bounded
//! variables with the entering variable's own bound in the ratio test), so it
//! ends in an improving step or at optimality. Hence the method terminates.
//!
//! Certificate. At the end `certify` recomputes `x_B` from the final basis,
//! checks `B·x_B` equals the right-hand side directly, checks every bound,
//! checks that the dual `π = c_B·M` satisfies `π·B = D·c_B` directly, and
//! checks each nonbasic reduced cost's sign against its position. Together
//! these prove optimality by weak duality; any failure panics.

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

use crate::lp::{Lp, LpResult};

/// Progress counters. `pivots` counts basis changes in both phases,
/// `phase1_pivots` those in phase 1, `flips` bound flips without a basis
/// change. `objective` is the current phase objective (phase 2: `c·x`
/// unscaled; phase 1: the sum of artificials), filled only when reported.
#[derive(Debug, Clone)]
pub struct Stats {
    pub pivots: u64,
    pub flips: u64,
    pub phase1_pivots: u64,
    pub d_bits: u64,
    pub d_bits_max: u64,
    pub phase: u8,
    pub objective: BigRational,
}

type Progress<'a> = Option<&'a mut dyn FnMut(&Stats)>;

const REPORT_EVERY: u64 = 25;
const DEGENERATE_LIMIT: u32 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Basic,
    Lower,
    Upper,
    Interior,
}

enum Block {
    Own,
    /// Basis row, and whether the leaving variable hits its upper bound.
    Row(usize, bool),
}

struct Simplex {
    m: usize,
    n: usize,
    cols: Vec<Vec<(usize, BigInt)>>,
    b: Vec<BigInt>,
    lower: Vec<BigRational>,
    upper: Vec<Option<BigRational>>,
    x: Vec<BigRational>,
    state: Vec<State>,
    basis: Vec<usize>,
    minv: Vec<Vec<BigInt>>,
    d: BigInt,
    threads: usize,
    stats: Stats,
}

fn rat(v: &BigInt) -> BigRational {
    BigRational::from_integer(v.clone())
}

/// Scale rationals to coprime integers by a positive factor.
fn integral(vals: &[&BigRational]) -> Vec<BigInt> {
    let l = vals.iter().fold(BigInt::one(), |l, v| l.lcm(v.denom()));
    let ints: Vec<BigInt> = vals.iter().map(|v| v.numer() * (&l / v.denom())).collect();
    let g = ints.iter().fold(BigInt::zero(), |g, v| g.gcd(v));
    if g.is_zero() || g.is_one() {
        ints
    } else {
        ints.into_iter().map(|v| v / &g).collect()
    }
}

/// Apply `f` to every index in `0..len`, split across `threads` workers.
fn par_map<T: Send>(len: usize, threads: usize, f: impl Fn(usize) -> T + Sync) -> Vec<T> {
    if threads <= 1 || len < 2 * threads {
        return (0..len).map(f).collect();
    }
    let chunk = len.div_ceil(threads);
    let f = &f;
    std::thread::scope(|s| {
        let handles: Vec<_> = (0..len)
            .step_by(chunk)
            .map(|lo| s.spawn(move || (lo..(lo + chunk).min(len)).map(f).collect::<Vec<T>>()))
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("worker panicked"))
            .collect()
    })
}

impl Simplex {
    fn col_dot(&self, j: usize, v: &[BigInt]) -> BigInt {
        self.cols[j]
            .iter()
            .fold(BigInt::zero(), |s, (k, a)| s + &v[*k] * a)
    }

    /// `α = M·A_j`.
    fn alpha(&self, j: usize) -> Vec<BigInt> {
        let col = &self.cols[j];
        self.minv
            .iter()
            .map(|row| {
                col.iter()
                    .fold(BigInt::zero(), |s, (k, a)| s + &row[*k] * a)
            })
            .collect()
    }

    /// `π = c_B·M` (equal to `D·c_B·B⁻¹`).
    fn pi(&self, cost: &[BigInt]) -> Vec<BigInt> {
        let rows: Vec<(&BigInt, &Vec<BigInt>)> = self
            .basis
            .iter()
            .zip(&self.minv)
            .map(|(&bv, row)| (&cost[bv], row))
            .filter(|(c, _)| !c.is_zero())
            .collect();
        par_map(self.m, self.threads, |k| {
            rows.iter()
                .fold(BigInt::zero(), |s, (c, row)| s + *c * &row[k])
        })
    }

    /// `D·d_j = D·c_j − π·A_j` for every nonbasic column (None for basic).
    fn reduced(&self, cost: &[BigInt], pi: &[BigInt]) -> Vec<Option<BigInt>> {
        par_map(self.cols.len(), self.threads, |j| {
            (self.state[j] != State::Basic).then(|| &self.d * &cost[j] - self.col_dot(j, pi))
        })
    }

    fn fixed(&self, j: usize) -> bool {
        self.upper[j].as_ref() == Some(&self.lower[j])
    }

    /// Direction (+1 increase, −1 decrease) in which nonbasic `j` improves.
    fn direction(&self, j: usize, dj: &BigInt) -> Option<i8> {
        if self.fixed(j) || dj.is_zero() {
            return None;
        }
        match (self.state[j], dj.is_negative()) {
            (State::Lower | State::Interior, true) => Some(1),
            (State::Upper | State::Interior, false) => Some(-1),
            _ => None,
        }
    }

    fn objective(&self, cost: &[BigInt]) -> BigRational {
        cost.iter()
            .zip(&self.x)
            .filter(|(c, _)| !c.is_zero())
            .fold(BigRational::zero(), |s, (c, x)| s + rat(c) * x)
    }

    fn update_inverse(&mut self, r: usize, alpha: &[BigInt]) {
        let ar = alpha[r].clone();
        let d = self.d.clone();
        let mr = std::mem::take(&mut self.minv[r]);
        let rows = std::mem::take(&mut self.minv);
        let mr_ref = &mr;
        let (ar_ref, d_ref) = (&ar, &d);
        let mut rows: Vec<Vec<BigInt>> = {
            let work = |(i, mut row): (usize, Vec<BigInt>)| {
                if i == r {
                    return row;
                }
                let ai = &alpha[i];
                for (v, mk) in row.iter_mut().zip(mr_ref) {
                    let cross = !ai.is_zero() && !mk.is_zero();
                    if v.is_zero() && !cross {
                        continue;
                    }
                    let mut num = ar_ref * &*v;
                    if cross {
                        num -= ai * mk;
                    }
                    let (q, rem) = num.div_rem(d_ref);
                    assert!(
                        rem.is_zero(),
                        "Bareiss update: (α_r·M_ik − α_i·M_rk) not divisible by D at row {i}"
                    );
                    *v = q;
                }
                row
            };
            if self.threads <= 1 || self.m < 2 * self.threads {
                rows.into_iter().enumerate().map(work).collect()
            } else {
                let chunk = self.m.div_ceil(self.threads);
                let mut parts: Vec<Vec<(usize, Vec<BigInt>)>> = Vec::new();
                let mut it = rows.into_iter().enumerate().peekable();
                while it.peek().is_some() {
                    parts.push(it.by_ref().take(chunk).collect());
                }
                let work = &work;
                std::thread::scope(|s| {
                    let hs: Vec<_> = parts
                        .into_iter()
                        .map(|p| s.spawn(move || p.into_iter().map(work).collect::<Vec<_>>()))
                        .collect();
                    hs.into_iter()
                        .flat_map(|h| h.join().expect("worker panicked"))
                        .collect()
                })
            }
        };
        rows[r] = mr;
        self.d = ar;
        if self.d.is_negative() {
            self.d = -std::mem::take(&mut self.d);
            for v in rows.iter_mut().flatten() {
                *v = -std::mem::take(v);
            }
        }
        self.minv = rows;
        let bits = self.d.bits();
        self.stats.d_bits = bits;
        self.stats.d_bits_max = self.stats.d_bits_max.max(bits);
    }

    fn report(
        &mut self,
        cost: &[BigInt],
        phase: u8,
        c_orig: &[BigRational],
        progress: &mut Progress,
    ) {
        if let Some(f) = progress.as_mut() {
            self.stats.phase = phase;
            self.stats.objective = if phase == 2 {
                c_orig
                    .iter()
                    .zip(&self.x)
                    .fold(BigRational::zero(), |s, (c, x)| s + c * x)
            } else {
                self.objective(cost)
            };
            f(&self.stats);
        }
    }

    /// Primal simplex on `cost` until no nonbasic variable is eligible.
    fn run(&mut self, cost: &[BigInt], phase: u8, c_orig: &[BigRational], progress: &mut Progress) {
        let mut degenerate = 0u32;
        let mut bland = false;
        loop {
            let pi = self.pi(cost);
            let red = self.reduced(cost, &pi);
            let mut enter: Option<(usize, i8, BigInt)> = None;
            for (j, dj) in red.iter().enumerate() {
                let Some(dj) = dj else { continue };
                let Some(dir) = self.direction(j, dj) else {
                    continue;
                };
                if bland {
                    enter = Some((j, dir, dj.abs()));
                    break;
                }
                if enter.as_ref().is_none_or(|(_, _, best)| dj.abs() > *best) {
                    enter = Some((j, dir, dj.abs()));
                }
            }
            let Some((s, dir, _)) = enter else { return };
            let alpha = self.alpha(s);

            let own = if dir > 0 {
                self.upper[s].as_ref().map(|u| u - &self.x[s])
            } else {
                Some(&self.x[s] - &self.lower[s])
            };
            let mut best: Option<(BigRational, Block)> = own.map(|t| (t, Block::Own));
            for (i, ai) in alpha.iter().enumerate() {
                if ai.is_zero() {
                    continue;
                }
                let g = BigRational::new(if dir > 0 { ai.clone() } else { -ai }, self.d.clone());
                let bv = self.basis[i];
                let (t, up) = if g.is_positive() {
                    ((&self.x[bv] - &self.lower[bv]) / &g, false)
                } else {
                    match &self.upper[bv] {
                        Some(u) => ((&self.x[bv] - u) / &g, true),
                        None => continue,
                    }
                };
                let better = match &best {
                    None => true,
                    Some((bt, blk)) => {
                        t < *bt
                            || (t == *bt
                                && matches!(blk, Block::Row(bi, _) if bv < self.basis[*bi]))
                    }
                };
                if better {
                    best = Some((t, Block::Row(i, up)));
                }
            }
            let (t, block) = best.expect("unbounded direction: impossible for a box-bounded LP");

            if !t.is_zero() {
                for (i, ai) in alpha.iter().enumerate() {
                    if ai.is_zero() {
                        continue;
                    }
                    let g =
                        BigRational::new(if dir > 0 { ai.clone() } else { -ai }, self.d.clone());
                    let bv = self.basis[i];
                    self.x[bv] = &self.x[bv] - g * &t;
                }
                self.x[s] = if dir > 0 {
                    &self.x[s] + &t
                } else {
                    &self.x[s] - &t
                };
            }
            match block {
                Block::Own => {
                    let (st, v) = if dir > 0 {
                        (State::Upper, self.upper[s].clone().expect("own bound"))
                    } else {
                        (State::Lower, self.lower[s].clone())
                    };
                    self.state[s] = st;
                    self.x[s] = v;
                    self.stats.flips += 1;
                }
                Block::Row(r, up) => {
                    let q = self.basis[r];
                    if up {
                        self.state[q] = State::Upper;
                        self.x[q] = self.upper[q].clone().expect("upper bound");
                    } else {
                        self.state[q] = State::Lower;
                        self.x[q] = self.lower[q].clone();
                    }
                    self.basis[r] = s;
                    self.state[s] = State::Basic;
                    self.update_inverse(r, &alpha);
                    self.stats.pivots += 1;
                    if phase == 1 {
                        self.stats.phase1_pivots += 1;
                    }
                    if self.stats.pivots.is_multiple_of(REPORT_EVERY) {
                        self.report(cost, phase, c_orig, progress);
                    }
                }
            }
            if t.is_zero() {
                degenerate += 1;
                if degenerate >= DEGENERATE_LIMIT {
                    bland = true;
                }
            } else {
                degenerate = 0;
                bland = false;
            }
        }
    }

    /// The solver's own optimality proof for `cost`; panics on any failure.
    /// Returns `cost·x`.
    fn certify(&self, cost: &[BigInt]) -> BigRational {
        // Right-hand side seen by the basis.
        let mut rhs: Vec<BigRational> = self.b.iter().map(rat).collect();
        for j in (0..self.cols.len()).filter(|&j| self.state[j] != State::Basic) {
            if self.x[j].is_zero() {
                continue;
            }
            for (k, a) in &self.cols[j] {
                rhs[*k] = &rhs[*k] - rat(a) * &self.x[j];
            }
        }
        // x_B recomputed from scratch, and checked against B·x_B = rhs directly.
        let dr = rat(&self.d);
        for (i, row) in self.minv.iter().enumerate() {
            let v = row
                .iter()
                .zip(&rhs)
                .fold(BigRational::zero(), |s, (mk, r)| s + rat(mk) * r)
                / &dr;
            assert!(
                v == self.x[self.basis[i]],
                "certificate: recomputed x_B differs at basis row {i}"
            );
        }
        let mut lhs = vec![BigRational::zero(); self.m];
        for &bv in &self.basis {
            for (k, a) in &self.cols[bv] {
                lhs[*k] = &lhs[*k] + rat(a) * &self.x[bv];
            }
        }
        assert!(
            lhs == rhs,
            "certificate: B·x_B differs from the right-hand side"
        );
        // Every bound.
        for j in 0..self.cols.len() {
            assert!(
                self.x[j] >= self.lower[j],
                "certificate: variable {j} below its lower bound"
            );
            if let Some(u) = &self.upper[j] {
                assert!(
                    self.x[j] <= *u,
                    "certificate: variable {j} above its upper bound"
                );
            }
            match self.state[j] {
                State::Lower => {
                    assert!(self.x[j] == self.lower[j], "certificate: {j} not at lower")
                }
                State::Upper => assert!(
                    Some(&self.x[j]) == self.upper[j].as_ref(),
                    "certificate: {j} not at upper"
                ),
                _ => {}
            }
        }
        // Dual: π·B = D·c_B directly, then reduced-cost signs.
        let pi = self.pi(cost);
        for &bv in &self.basis {
            assert!(
                self.col_dot(bv, &pi) == &self.d * &cost[bv],
                "certificate: dual fails on basic column {bv}"
            );
        }
        for j in (0..self.cols.len()).filter(|&j| self.state[j] != State::Basic) {
            if self.fixed(j) {
                continue;
            }
            let dj = &self.d * &cost[j] - self.col_dot(j, &pi);
            let ok = match self.state[j] {
                State::Lower => !dj.is_negative(),
                State::Upper => !dj.is_positive(),
                _ => dj.is_zero(),
            };
            assert!(
                ok,
                "certificate: reduced cost of nonbasic {j} ({:?}) has the wrong sign",
                self.state[j]
            );
        }
        self.objective(cost)
    }
}

pub fn solve(lp: &Lp) -> LpResult {
    solve_with_stats(lp, None).0
}

pub fn solve_with_stats(
    lp: &Lp,
    mut progress: Option<&mut dyn FnMut(&Stats)>,
) -> (LpResult, Stats) {
    let n = lp.c.len();
    let m = lp.rows.len();
    let stats = Stats {
        pivots: 0,
        flips: 0,
        phase1_pivots: 0,
        d_bits: 1,
        d_bits_max: 1,
        phase: 1,
        objective: BigRational::zero(),
    };
    if lp.lower.iter().zip(&lp.upper).any(|(l, u)| l > u) {
        return (LpResult::Infeasible, stats);
    }

    // Integral rows.
    let mut cols: Vec<Vec<(usize, BigInt)>> = vec![Vec::new(); n];
    let mut b = Vec::with_capacity(m);
    for (i, (a, bi)) in lp.rows.iter().enumerate() {
        let vals: Vec<&BigRational> = a.iter().chain(std::iter::once(bi)).collect();
        let mut ints = integral(&vals);
        b.push(ints.pop().expect("rhs"));
        for (j, v) in ints.into_iter().enumerate() {
            if !v.is_zero() {
                cols[j].push((i, v));
            }
        }
    }
    let c_int = integral(&lp.c.iter().collect::<Vec<_>>());

    // Start point: 0 when inside the box, else the bound nearer 0.
    let mut x = Vec::with_capacity(n + m);
    let mut state = Vec::with_capacity(n + m);
    for (l, u) in lp.lower.iter().zip(&lp.upper) {
        let (v, st) = if l.is_positive() || l == u {
            (l.clone(), State::Lower)
        } else if u.is_negative() || u.is_zero() {
            (u.clone(), State::Upper)
        } else if l.is_zero() {
            (l.clone(), State::Lower)
        } else {
            (BigRational::zero(), State::Interior)
        };
        x.push(v);
        state.push(st);
    }
    let mut lower: Vec<BigRational> = lp.lower.clone();
    let mut upper: Vec<Option<BigRational>> = lp.upper.iter().cloned().map(Some).collect();

    // Slacks, and artificials for rows the start point violates.
    let mut basis = vec![0; m];
    let mut minv = vec![vec![BigInt::zero(); m]; m];
    let mut art = Vec::new();
    let mut slack_val: Vec<BigRational> = b.iter().map(rat).collect();
    for (j, col) in cols.iter().enumerate() {
        if x[j].is_zero() {
            continue;
        }
        for (k, a) in col {
            slack_val[*k] = &slack_val[*k] - rat(a) * &x[j];
        }
    }
    for (i, s) in slack_val.into_iter().enumerate() {
        cols.push(vec![(i, BigInt::one())]);
        lower.push(BigRational::zero());
        upper.push(None);
        if s.is_negative() {
            x.push(BigRational::zero());
            state.push(State::Lower);
            art.push((i, -s));
        } else {
            x.push(s);
            state.push(State::Basic);
            basis[i] = n + i;
            minv[i][i] = BigInt::one();
        }
    }
    let n_art = art.len();
    for (i, v) in art {
        let j = cols.len();
        cols.push(vec![(i, -BigInt::one())]);
        lower.push(BigRational::zero());
        upper.push(None);
        x.push(v);
        state.push(State::Basic);
        basis[i] = j;
        minv[i][i] = -BigInt::one();
    }
    let total = cols.len();
    let threads = if m >= 64 {
        std::thread::available_parallelism().map_or(1, |t| t.get())
    } else {
        1
    };
    let mut sx = Simplex {
        m,
        n,
        cols,
        b,
        lower,
        upper,
        x,
        state,
        basis,
        minv,
        d: BigInt::one(),
        threads,
        stats,
    };

    if n_art > 0 {
        let mut cost1 = vec![BigInt::zero(); total];
        for c in cost1.iter_mut().skip(n + m) {
            *c = BigInt::one();
        }
        sx.run(&cost1, 1, &lp.c, &mut progress);
        let infeas = sx.certify(&cost1);
        if infeas.is_positive() {
            return (LpResult::Infeasible, sx.stats);
        }
        for j in n + m..total {
            sx.upper[j] = Some(BigRational::zero());
            if sx.state[j] != State::Basic {
                sx.state[j] = State::Lower;
            }
        }
    }

    let mut cost2 = c_int;
    cost2.resize(total, BigInt::zero());
    sx.run(&cost2, 2, &lp.c, &mut progress);
    sx.certify(&cost2);

    let xs: Vec<BigRational> = sx.x[..sx.n].to_vec();
    for (k, (a, bi)) in lp.rows.iter().enumerate() {
        let v = a
            .iter()
            .zip(&xs)
            .filter(|(ai, _)| !ai.is_zero())
            .fold(BigRational::zero(), |s, (ai, xi)| s + ai * xi);
        assert!(v <= *bi, "certificate: original row {k} violated");
    }
    for (j, v) in xs.iter().enumerate() {
        assert!(
            *v >= lp.lower[j] && *v <= lp.upper[j],
            "certificate: original bound of {j} violated"
        );
    }
    let objective =
        lp.c.iter()
            .zip(&xs)
            .fold(BigRational::zero(), |s, (c, v)| s + c * v);
    sx.stats.phase = 2;
    sx.stats.objective = objective.clone();
    (LpResult::Optimal { x: xs, objective }, sx.stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        }
        fn range(&mut self, lo: i64, hi: i64) -> i64 {
            lo + (self.next() % (hi - lo + 1) as u64) as i64
        }
    }

    fn q(n: i64, d: i64) -> BigRational {
        BigRational::new(BigInt::from(n), BigInt::from(d))
    }

    /// `ints` must be `λ·vals` for one λ > 0 (all zero when `vals` is).
    fn assert_one_positive_factor(vals: &[&BigRational], ints: &[BigInt], what: &str) {
        assert_eq!(vals.len(), ints.len(), "{what}: length");
        let Some(first) = vals.iter().position(|v| !v.is_zero()) else {
            assert!(
                ints.iter().all(|v| v.is_zero()),
                "{what}: all-zero input must scale to all zeros"
            );
            return;
        };
        let lambda = rat(&ints[first]) / vals[first].clone();
        assert!(
            lambda.is_positive(),
            "{what}: factor {lambda} is not positive"
        );
        for (k, (v, i)) in vals.iter().zip(ints).enumerate() {
            assert_eq!(
                rat(i),
                &lambda * *v,
                "{what}: entry {k} is not scaled by the common factor"
            );
        }
    }

    fn assert_row_scaling(rows: &[(Vec<BigRational>, BigRational)], c: &[BigRational], what: &str) {
        for (i, (a, b)) in rows.iter().enumerate() {
            let vals: Vec<&BigRational> = a.iter().chain(std::iter::once(b)).collect();
            assert_one_positive_factor(&vals, &integral(&vals), &format!("{what} row {i}"));
        }
        let cv: Vec<&BigRational> = c.iter().collect();
        assert_one_positive_factor(&cv, &integral(&cv), &format!("{what} objective"));
    }

    fn random_rat(rng: &mut Rng) -> BigRational {
        if rng.range(0, 3) == 0 {
            BigRational::zero()
        } else {
            q(rng.range(-40, 40), rng.range(1, 24))
        }
    }

    #[test]
    fn integral_scales_each_row_by_one_positive_factor() {
        let mut rng = Rng(0x5eed_0001);
        let mut rows = Vec::new();
        for r in 0..200 {
            let len = rng.range(1, 30) as usize;
            let a: Vec<BigRational> = (0..len).map(|_| random_rat(&mut rng)).collect();
            let b = match r % 4 {
                0 => BigRational::zero(),
                1 => -q(rng.range(1, 40), rng.range(1, 24)),
                _ => random_rat(&mut rng),
            };
            rows.push((a, b));
        }
        rows.push((vec![BigRational::zero(); 5], BigRational::zero()));
        rows.push((vec![BigRational::zero(); 3], q(-3, 7)));
        let c: Vec<BigRational> = (0..30).map(|_| random_rat(&mut rng)).collect();
        assert_row_scaling(&rows, &c, "random");
        assert_row_scaling(&[], &vec![BigRational::zero(); 4], "zero objective");
        assert_row_scaling(&[], &[q(-6, 4), q(9, 10)], "negative objective");
    }

    #[test]
    #[ignore = "reads the real saved case from the owner's local disk"]
    fn integral_on_saved_case_rows() {
        let dir = std::path::Path::new(
            "/Users/casey.allard/uor-r4/local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2",
        );
        let (case, _) = crate::case::load_verified(dir).expect("load the saved case");
        let p = case.problem;
        let lp = crate::bb::relaxation(&p);
        assert_eq!(lp.rows.len(), 440);
        assert_row_scaling(&lp.rows, &lp.c, "saved case");
    }

    #[test]
    #[should_panic(expected = "certificate: reduced cost of nonbasic 0 (Lower) has the wrong sign")]
    fn certify_rejects_a_non_optimal_point() {
        // minimise -x subject to x <= 1, 0 <= x <= 2: optimum x = 1, start x = 0.
        let stats = Stats {
            pivots: 0,
            flips: 0,
            phase1_pivots: 0,
            d_bits: 1,
            d_bits_max: 1,
            phase: 2,
            objective: BigRational::zero(),
        };
        let sx = Simplex {
            m: 1,
            n: 1,
            cols: vec![vec![(0, BigInt::one())], vec![(0, BigInt::one())]],
            b: vec![BigInt::one()],
            lower: vec![BigRational::zero(), BigRational::zero()],
            upper: vec![Some(q(2, 1)), None],
            x: vec![BigRational::zero(), BigRational::one()],
            state: vec![State::Lower, State::Basic],
            basis: vec![1],
            minv: vec![vec![BigInt::one()]],
            d: BigInt::one(),
            threads: 1,
            stats,
        };
        sx.certify(&[-BigInt::one(), BigInt::zero()]);
    }
}
