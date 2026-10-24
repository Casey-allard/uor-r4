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
    pub start: Start,
}

/// How a run began. `Cold` is the all-slack start (with phase 1 when needed);
/// `Warm` is `solve_from_basis` from the given basis. The `ColdAfter*` values
/// are `solve_from_basis` falling back to the cold start because the given
/// basis was malformed (wrong lengths, an index out of range or repeated, a
/// position its bounds do not allow, or empty bounds), singular, or implied a
/// point that violates a row or a bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    Cold,
    Warm,
    ColdAfterMalformed,
    ColdAfterSingular,
    ColdAfterInfeasible,
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

/// Where a nonbasic variable sits: its lower bound, its upper bound, or the
/// start value 0 strictly inside its bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pos {
    Lower,
    Upper,
    Interior,
}

/// A basis in `blp`'s index space: structurals `0..n`, then the slack of
/// row `i` at `n + i`. `basic[r]` is the variable of basis row `r` (length
/// m); `nonbasic_pos` has length `n + m`, and its entries at basic indices
/// are ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Basis {
    pub basic: Vec<usize>,
    pub nonbasic_pos: Vec<Pos>,
}

/// The integer-scaled LP both the simplex and `certify_basis` read:
/// structural columns (sparse, by row), right-hand sides and costs.
pub(crate) struct Scaled {
    pub(crate) cols: Vec<Vec<(usize, BigInt)>>,
    pub(crate) b: Vec<BigInt>,
    pub(crate) c: Vec<BigInt>,
}

pub(crate) fn scaled(lp: &Lp) -> Scaled {
    let n = lp.c.len();
    let m = lp.rows.len();
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
    let c = integral(&lp.c.iter().collect::<Vec<_>>());
    Scaled { cols, b, c }
}

fn exact_div(num: BigInt, d: &BigInt) -> Result<BigInt, String> {
    let (q, rem) = num.div_rem(d);
    if rem.is_zero() {
        Ok(q)
    } else {
        Err("Bareiss division not exact".into())
    }
}

/// Solve `a·y = r` exactly: unit-row and unit-column reductions while the
/// Bareiss divisor is 1, then forward Bareiss (fraction-free) elimination with
/// row exchanges, then back substitution `y_i = (p·r_i − Σ_{j>i} U_ij·y_j) /
/// U_ii` with `p` the last pivot, and `y / p` returned.
fn bareiss_solve(mut a: Vec<Vec<BigInt>>, mut r: Vec<BigInt>) -> Result<Vec<BigRational>, String> {
    let m = r.len();
    let mut col_perm: Vec<usize> = (0..m).collect();
    let mut prev = BigInt::one();
    let one = BigInt::one();
    let neg_one = -BigInt::one();
    for k in 0..m {
        if prev == one {
            let mut unit_row: Option<(usize, usize)> = None;
            for (i, row) in a.iter().enumerate().take(m).skip(k) {
                let mut nz_col = None;
                let mut count = 0usize;
                for (j, val) in row.iter().enumerate().take(m).skip(k) {
                    if !val.is_zero() {
                        nz_col = Some(j);
                        count += 1;
                        if count > 1 {
                            break;
                        }
                    }
                }
                if count == 1 {
                    let c = nz_col.expect("count == 1");
                    if row[c] == one || row[c] == neg_one {
                        unit_row = Some((i, c));
                        break;
                    }
                }
            }
            if let Some((p_row, p_col)) = unit_row {
                a.swap(k, p_row);
                r.swap(k, p_row);
                if p_col != k {
                    for row in &mut a {
                        row.swap(k, p_col);
                    }
                    col_perm.swap(k, p_col);
                }
                if a[k][k] == neg_one {
                    a[k][k] = one.clone();
                    r[k] = -&r[k];
                }
                let rk = r[k].clone();
                for i in k + 1..m {
                    if !a[i][k].is_zero() {
                        r[i] = &r[i] - &a[i][k] * &rk;
                        a[i][k] = BigInt::zero();
                    }
                }
                continue;
            }

            let mut unit_col: Option<(usize, usize)> = None;
            for j in k..m {
                let mut nz_row = None;
                let mut count = 0usize;
                for (i, row) in a.iter().enumerate().take(m).skip(k) {
                    if !row[j].is_zero() {
                        nz_row = Some(i);
                        count += 1;
                        if count > 1 {
                            break;
                        }
                    }
                }
                if count == 1 {
                    let p = nz_row.expect("count == 1");
                    if a[p][j] == one || a[p][j] == neg_one {
                        unit_col = Some((p, j));
                        break;
                    }
                }
            }
            if let Some((p_row, p_col)) = unit_col {
                a.swap(k, p_row);
                r.swap(k, p_row);
                if p_col != k {
                    for row in &mut a {
                        row.swap(k, p_col);
                    }
                    col_perm.swap(k, p_col);
                }
                if a[k][k] == neg_one {
                    for v in &mut a[k][k..m] {
                        *v = -&*v;
                    }
                    r[k] = -&r[k];
                }
                continue;
            }
        }

        let p = (k..m)
            .find(|&i| !a[i][k].is_zero())
            .ok_or("singular basis")?;
        a.swap(k, p);
        r.swap(k, p);
        for i in k + 1..m {
            for j in k + 1..m {
                let num = &a[k][k] * &a[i][j] - &a[i][k] * &a[k][j];
                a[i][j] = exact_div(num, &prev)?;
            }
            let num = &a[k][k] * &r[i] - &a[i][k] * &r[k];
            r[i] = exact_div(num, &prev)?;
            a[i][k] = BigInt::zero();
        }
        prev = a[k][k].clone();
    }
    let mut z_perm = vec![BigInt::zero(); m];
    for i in (0..m).rev() {
        let mut s = &prev * &r[i];
        for j in i + 1..m {
            if !a[i][j].is_zero() && !z_perm[j].is_zero() {
                s -= &a[i][j] * &z_perm[j];
            }
        }
        z_perm[i] = exact_div(s, &a[i][i])?;
    }
    let mut y = vec![BigRational::zero(); m];
    for (k, z) in z_perm.into_iter().enumerate() {
        y[col_perm[k]] = BigRational::new(z, prev.clone());
    }
    Ok(y)
}

/// Exact optimality proof for `lp` from `basis` alone. Recomputes `x_B` and
/// the dual `π` with Bareiss on the scaled basis matrix, then checks every
/// input row and bound on `x` and each nonbasic reduced cost's sign by
/// position (Lower ≥ 0, Upper ≤ 0, Interior = 0; fixed variables skipped, as
/// in the solver's own certificate). Never panics on a bad basis.
pub fn certify_basis(lp: &Lp, basis: &Basis) -> Result<LpResult, String> {
    certify_basis_with_dual(lp, basis).map(|(res, _)| res)
}

/// Like `certify_basis`, and also returns the exact rational unscaled dual
/// multiplier vector `π` (one entry per row of `lp`, with `π_i <= 0` on
/// active `<= b_i` rows) satisfying `B_unscaledᵀ π = c_unscaled_B`.
pub fn certify_basis_with_dual(
    lp: &Lp,
    basis: &Basis,
) -> Result<(LpResult, Vec<BigRational>), String> {
    let n = lp.c.len();
    let m = lp.rows.len();
    let total = n + m;
    if lp.lower.len() != n || lp.upper.len() != n || lp.rows.iter().any(|(a, _)| a.len() != n) {
        return Err("malformed LP: lengths differ".into());
    }
    if basis.basic.len() != m {
        return Err(format!("{} basic indices for {m} rows", basis.basic.len()));
    }
    if basis.nonbasic_pos.len() != total {
        return Err(format!(
            "{} positions for {total} variables",
            basis.nonbasic_pos.len()
        ));
    }
    let mut is_basic = vec![false; total];
    for &j in &basis.basic {
        if j >= total {
            return Err(format!("basic index {j} out of range"));
        }
        if is_basic[j] {
            return Err(format!("basic index {j} repeated"));
        }
        is_basic[j] = true;
    }
    let Scaled { mut cols, b, c } = scaled(lp);
    let mut row_scale = vec![BigRational::zero(); m];
    for (j, col) in cols.iter().enumerate() {
        for (i, a_scaled) in col {
            if row_scale[*i].is_zero() && !lp.rows[*i].0[j].is_zero() {
                row_scale[*i] = rat(a_scaled) / &lp.rows[*i].0[j];
            }
        }
    }
    for (i, (_, bi)) in lp.rows.iter().enumerate() {
        if row_scale[i].is_zero() {
            row_scale[i] = if !bi.is_zero() {
                rat(&b[i]) / bi
            } else {
                BigRational::one()
            };
        }
    }
    let c_scale =
        lp.c.iter()
            .enumerate()
            .find(|(_, cj)| !cj.is_zero())
            .map(|(j0, cj)| rat(&c[j0]) / cj)
            .unwrap_or_else(BigRational::one);

    for i in 0..m {
        cols.push(vec![(i, BigInt::one())]);
    }
    let mut cost = c;
    cost.resize(total, BigInt::zero());
    let bounds = |j: usize| -> (BigRational, Option<BigRational>) {
        if j < n {
            (lp.lower[j].clone(), Some(lp.upper[j].clone()))
        } else {
            (BigRational::zero(), None)
        }
    };

    // Nonbasic values by position.
    let mut x = vec![BigRational::zero(); total];
    for j in (0..total).filter(|&j| !is_basic[j]) {
        let (lo, up) = bounds(j);
        x[j] = match basis.nonbasic_pos[j] {
            Pos::Lower => lo,
            Pos::Upper => up.ok_or_else(|| format!("nonbasic {j} at Upper has no upper bound"))?,
            Pos::Interior => {
                if !(lo.is_negative() && up.is_some_and(|u| u.is_positive())) {
                    return Err(format!(
                        "nonbasic {j} Interior but 0 is not strictly inside"
                    ));
                }
                BigRational::zero()
            }
        };
    }

    // B·x_B = b − N·x_N, cleared to integers by the lcm of its denominators.
    let mut rhs: Vec<BigRational> = b.iter().map(rat).collect();
    for j in (0..total).filter(|&j| !is_basic[j] && !x[j].is_zero()) {
        for (k, a) in &cols[j] {
            rhs[*k] = &rhs[*k] - rat(a) * &x[j];
        }
    }
    let l = rhs.iter().fold(BigInt::one(), |l, v| l.lcm(v.denom()));
    let r: Vec<BigInt> = rhs.iter().map(|v| v.numer() * (&l / v.denom())).collect();
    let mut bm = vec![vec![BigInt::zero(); m]; m];
    for (i, &j) in basis.basic.iter().enumerate() {
        for (k, a) in &cols[j] {
            bm[*k][i] = a.clone();
        }
    }
    let lr = rat(&l);
    let xb = bareiss_solve(bm.clone(), r)?;
    for (i, &j) in basis.basic.iter().enumerate() {
        x[j] = &xb[i] / &lr;
    }
    let mut bt = vec![vec![BigInt::zero(); m]; m];
    for (k, row) in bm.iter().enumerate() {
        for (i, v) in row.iter().enumerate() {
            bt[i][k] = v.clone();
        }
    }
    let cb: Vec<BigInt> = basis.basic.iter().map(|&j| cost[j].clone()).collect();
    let pi = bareiss_solve(bt, cb)?;
    let col_pi = |j: usize| {
        cols[j]
            .iter()
            .filter(|(k, _)| !pi[*k].is_zero())
            .fold(BigRational::zero(), |s, (k, a)| s + rat(a) * &pi[*k])
    };

    // Direct checks of both solves.
    let mut lhs = vec![BigRational::zero(); m];
    for &j in &basis.basic {
        if !x[j].is_zero() {
            for (k, a) in &cols[j] {
                lhs[*k] = &lhs[*k] + rat(a) * &x[j];
            }
        }
    }
    if lhs != rhs {
        return Err("B·x_B differs from the right-hand side".into());
    }
    for &j in &basis.basic {
        if col_pi(j) != rat(&cost[j]) {
            return Err(format!("dual fails on basic column {j}"));
        }
    }

    // Every input row and bound, exactly.
    let xs: Vec<BigRational> = x[..n].to_vec();
    for (k, (a, bi)) in lp.rows.iter().enumerate() {
        let nb_sum = a
            .iter()
            .zip(&xs)
            .enumerate()
            .filter(|(j, (ai, xi))| !is_basic[*j] && !ai.is_zero() && !xi.is_zero())
            .fold(BigRational::zero(), |s, (_, (ai, xi))| s + ai * xi);
        let b_sum = a
            .iter()
            .zip(&xs)
            .enumerate()
            .filter(|(j, (ai, xi))| is_basic[*j] && !ai.is_zero() && !xi.is_zero())
            .fold(BigRational::zero(), |s, (_, (ai, xi))| s + ai * xi);
        let v = nb_sum + b_sum;
        if v > *bi {
            return Err(format!("input row {k} violated"));
        }
    }
    for (j, v) in xs.iter().enumerate() {
        if *v < lp.lower[j] || *v > lp.upper[j] {
            return Err(format!("bound of {j} violated"));
        }
    }

    // Reduced-cost signs by position.
    for j in (0..total).filter(|&j| !is_basic[j]) {
        let (lo, up) = bounds(j);
        if up.as_ref() == Some(&lo) {
            continue;
        }
        let dj = rat(&cost[j]) - col_pi(j);
        let ok = match basis.nonbasic_pos[j] {
            Pos::Lower => !dj.is_negative(),
            Pos::Upper => !dj.is_positive(),
            Pos::Interior => dj.is_zero(),
        };
        if !ok {
            return Err(format!(
                "reduced cost of nonbasic {j} ({:?}) has the wrong sign",
                basis.nonbasic_pos[j]
            ));
        }
    }
    let pi_unscaled: Vec<BigRational> = pi
        .iter()
        .enumerate()
        .map(|(i, p_val)| p_val * &row_scale[i] / &c_scale)
        .collect();
    let objective =
        lp.c.iter()
            .zip(&xs)
            .fold(BigRational::zero(), |s, (c, v)| s + c * v);
    Ok((LpResult::Optimal { x: xs, objective }, pi_unscaled))
}

pub fn solve(lp: &Lp) -> LpResult {
    solve_with_stats(lp, None).0
}

pub fn solve_with_stats(lp: &Lp, progress: Option<&mut dyn FnMut(&Stats)>) -> (LpResult, Stats) {
    let (result, stats, _) = solve_inner(lp, progress);
    (result, stats)
}

/// `solve`, plus the final basis when the result is `Optimal`. A phase-1
/// artificial still basic (at 0) is reported as its row's slack, which is
/// then nonbasic in the solver: same span, same `x`, and the same `π`.
pub fn solve_with_basis(lp: &Lp) -> (LpResult, Option<Basis>) {
    let (result, _, basis) = solve_inner(lp, None);
    (result, basis)
}

fn solve_inner(
    lp: &Lp,
    mut progress: Option<&mut dyn FnMut(&Stats)>,
) -> (LpResult, Stats, Option<Basis>) {
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
        start: Start::Cold,
    };
    if lp.lower.iter().zip(&lp.upper).any(|(l, u)| l > u) {
        return (LpResult::Infeasible, stats, None);
    }

    // Integral rows.
    let Scaled {
        mut cols,
        b,
        c: c_int,
    } = scaled(lp);

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
    let art_rows: Vec<usize> = art.iter().map(|(i, _)| *i).collect();
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
            return (LpResult::Infeasible, sx.stats, None);
        }
        for j in n + m..total {
            sx.upper[j] = Some(BigRational::zero());
            if sx.state[j] != State::Basic {
                sx.state[j] = State::Lower;
            }
        }
    }

    finish(sx, lp, c_int, &art_rows, &mut progress)
}

/// Phase 2 from `sx`'s current basis (primal feasible), the solver's own
/// certificate, the original rows and bounds, and the final basis in `blp`'s
/// index space. `art_rows[k]` is the row of artificial `n + m + k`.
fn finish(
    mut sx: Simplex,
    lp: &Lp,
    c_int: Vec<BigInt>,
    art_rows: &[usize],
    progress: &mut Progress,
) -> (LpResult, Stats, Option<Basis>) {
    let (n, m) = (sx.n, sx.m);
    let mut cost2 = c_int;
    cost2.resize(sx.cols.len(), BigInt::zero());
    sx.run(&cost2, 2, &lp.c, progress);
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
    let basis = Basis {
        basic: sx
            .basis
            .iter()
            .map(|&j| {
                if j >= n + m {
                    n + art_rows[j - n - m]
                } else {
                    j
                }
            })
            .collect(),
        nonbasic_pos: sx.state[..n + m]
            .iter()
            .map(|s| match s {
                State::Upper => Pos::Upper,
                State::Interior => Pos::Interior,
                State::Basic | State::Lower => Pos::Lower,
            })
            .collect(),
    };
    (
        LpResult::Optimal { x: xs, objective },
        sx.stats,
        Some(basis),
    )
}

pub fn solve_from_basis(lp: &Lp, basis: &Basis) -> LpResult {
    solve_from_basis_with_stats(lp, basis, None).0
}

/// `solve`, started from `basis` instead of the all-slack start: `M =
/// D·B⁻¹` is built exactly by fraction-free Gauss-Jordan elimination, then
/// phase 2 runs with the same pricing and the same final certificate. When
/// the basis is malformed, singular or primal infeasible, the run is the cold
/// start instead, and `Stats::start` says which.
pub fn solve_from_basis_with_stats(
    lp: &Lp,
    basis: &Basis,
    progress: Option<&mut dyn FnMut(&Stats)>,
) -> (LpResult, Stats) {
    let mut progress = progress;
    match warm(lp, basis) {
        Ok((sx, c_int)) => {
            let (result, mut stats, _) = finish(sx, lp, c_int, &[], &mut progress);
            stats.start = Start::Warm;
            (result, stats)
        }
        Err(why) => {
            let (result, mut stats, _) = solve_inner(lp, progress);
            stats.start = why;
            (result, stats)
        }
    }
}

/// The simplex state at `basis`, or why it cannot start there.
fn warm(lp: &Lp, basis: &Basis) -> Result<(Simplex, Vec<BigInt>), Start> {
    let n = lp.c.len();
    let m = lp.rows.len();
    let total = n + m;
    let malformed = lp.lower.len() != n
        || lp.upper.len() != n
        || lp.rows.iter().any(|(a, _)| a.len() != n)
        || lp.lower.iter().zip(&lp.upper).any(|(l, u)| l > u)
        || basis.basic.len() != m
        || basis.nonbasic_pos.len() != total;
    if malformed {
        return Err(Start::ColdAfterMalformed);
    }
    let mut state = vec![State::Lower; total];
    for &j in &basis.basic {
        if j >= total || state[j] == State::Basic {
            return Err(Start::ColdAfterMalformed);
        }
        state[j] = State::Basic;
    }
    let Scaled { mut cols, b, c } = scaled(lp);
    for i in 0..m {
        cols.push(vec![(i, BigInt::one())]);
    }
    let mut lower: Vec<BigRational> = lp.lower.clone();
    lower.resize(total, BigRational::zero());
    let mut upper: Vec<Option<BigRational>> = lp.upper.iter().cloned().map(Some).collect();
    upper.resize(total, None);

    // Nonbasic values by position.
    let mut x = vec![BigRational::zero(); total];
    for j in 0..total {
        if state[j] == State::Basic {
            continue;
        }
        let (st, v) = match basis.nonbasic_pos[j] {
            Pos::Lower => (State::Lower, lower[j].clone()),
            Pos::Upper => match &upper[j] {
                Some(u) => (State::Upper, u.clone()),
                None => return Err(Start::ColdAfterMalformed),
            },
            Pos::Interior => {
                if !(lower[j].is_negative() && upper[j].as_ref().is_some_and(|u| u.is_positive())) {
                    return Err(Start::ColdAfterMalformed);
                }
                (State::Interior, BigRational::zero())
            }
        };
        state[j] = st;
        x[j] = v;
    }

    // M = D·B⁻¹ by fraction-free Gauss-Jordan on [B | I]: after step k every
    // entry is a minor of order k + 1, so each division by the previous
    // pivot is exact, and at the end the left half is D·I with D = ±det B.
    let mut a = vec![vec![BigInt::zero(); 2 * m]; m];
    for (i, &j) in basis.basic.iter().enumerate() {
        for (k, v) in &cols[j] {
            a[*k][i] = v.clone();
        }
    }
    for (k, row) in a.iter_mut().enumerate() {
        row[m + k] = BigInt::one();
    }
    let mut prev = BigInt::one();
    for k in 0..m {
        let p = (k..m)
            .find(|&i| !a[i][k].is_zero())
            .ok_or(Start::ColdAfterSingular)?;
        a.swap(k, p);
        let pivot_row = a[k].clone();
        for (i, row) in a.iter_mut().enumerate() {
            if i == k {
                continue;
            }
            let aik = std::mem::take(&mut row[k]);
            for (j, v) in row.iter_mut().enumerate() {
                if j == k {
                    continue;
                }
                let num = &pivot_row[k] * &*v - &aik * &pivot_row[j];
                *v = exact_div(num, &prev).expect("fraction-free Gauss-Jordan division is exact");
            }
        }
        prev = pivot_row[k].clone();
    }
    let sign_flip = prev.is_negative();
    let d = prev.abs();
    let minv: Vec<Vec<BigInt>> = a
        .into_iter()
        .map(|row| {
            row.into_iter()
                .skip(m)
                .map(|v| if sign_flip { -v } else { v })
                .collect()
        })
        .collect();

    // x_B = M·(b − N·x_N) / D, and it must satisfy every bound (a basic
    // slack below 0 is a violated row).
    let mut rhs: Vec<BigRational> = b.iter().map(rat).collect();
    for j in (0..total).filter(|&j| state[j] != State::Basic && !x[j].is_zero()) {
        for (k, v) in &cols[j] {
            rhs[*k] = &rhs[*k] - rat(v) * &x[j];
        }
    }
    let dr = rat(&d);
    for (i, &j) in basis.basic.iter().enumerate() {
        let v = minv[i]
            .iter()
            .zip(&rhs)
            .filter(|(mk, _)| !mk.is_zero())
            .fold(BigRational::zero(), |s, (mk, r)| s + rat(mk) * r)
            / &dr;
        if v < lower[j] || upper[j].as_ref().is_some_and(|u| v > *u) {
            return Err(Start::ColdAfterInfeasible);
        }
        x[j] = v;
    }

    let threads = if m >= 64 {
        std::thread::available_parallelism().map_or(1, |t| t.get())
    } else {
        1
    };
    let bits = d.bits();
    let stats = Stats {
        pivots: 0,
        flips: 0,
        phase1_pivots: 0,
        d_bits: bits,
        d_bits_max: bits,
        phase: 2,
        objective: BigRational::zero(),
        start: Start::Warm,
    };
    let sx = Simplex {
        m,
        n,
        cols,
        b,
        lower,
        upper,
        x,
        state,
        basis: basis.basic.clone(),
        minv,
        d,
        threads,
        stats,
    };
    Ok((sx, c))
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
            start: Start::Cold,
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
