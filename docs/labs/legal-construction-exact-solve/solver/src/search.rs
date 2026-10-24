//! Bounded-precision basis search (#2828).
//!
//! The search only proposes a basis; `blp::certify_basis` proves or rejects
//! it exactly against the input LP. Nothing the search computes is trusted.
//!
//! * Data. `blp`'s integer-scaled rows and costs (for primal simplex) or
//!   direct dyadic conversion (for bounded-variable dual simplex), in `Dy<P>`.
//! * Primal method (`search_primal`). Bounded-variable revised simplex with
//!   `blp`'s conventions: slacks start basic, structurals nonbasic at Lower,
//!   Upper or Interior (start value 0 when lower < 0 < upper), bound flips,
//!   Dantzig pricing with Bland's rule after 50 consecutive degenerate steps.
//! * Dual method (`search_dual`). Bounded-variable Dual Simplex with the
//!   Bound-Flipping Ratio Test (BFRT) in `Dy<P>`: all `m` slacks start basic
//!   (`π = 0`) and each structural variable starts at its dual-feasible bound
//!   (`Upper` when `c_j < 0`, `Lower` when `c_j >= 0`), eliminating primal
//!   degeneracy at `δ = 0`. When `lp` carries `bb::relaxation`'s 3-row
//!   fractional disjunction blocks with `q0 = ±7`, `search` projects them onto
//!   the guard space, solves the projected LP with `search_dual`, and lifts
//!   the optimal basis back to the full relaxation.
//! * Basis-free Lagrangian dual bound (`certify_lagrangian_dual_bound`) and
//!   warm-started `Dy<P>` LP-guided discrete legal search
//!   (`lp_guided_discrete_solve`).

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

use crate::blp::{self, Basis, Pos, Scaled};
use crate::dyadic::Dy;
use crate::lp::{Lp, LpResult};
use crate::problem::Problem;

const REPORT_EVERY: u64 = 25;
const DEGENERATE_LIMIT: u32 = 50;
const REFACTOR_EVERY: u64 = 50;

/// Progress counters. `objective_bits` is `⌈log2 |objective|⌉` roughly (the
/// position above the top set bit), 0 for a zero objective.
#[derive(Debug, Clone, Default)]
pub struct SearchStats {
    pub pivots: u64,
    pub flips: u64,
    pub degenerate_pivots: u64,
    pub bland_switches: u64,
    pub phase: u8,
    pub objective_sign: i8,
    pub objective_bits: i64,
}

/// A proposed basis, with the search's own (untrusted) point and objective.
#[derive(Debug, Clone)]
pub struct Found<const P: u32> {
    pub basis: Basis,
    pub x: Vec<Dy<P>>,
    pub objective: Dy<P>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Basic,
    Lower,
    Upper,
    Interior,
}

enum Block<const P: u32> {
    Own(Dy<P>),
    Row(usize, bool, Dy<P>),
}

fn shift<const P: u32>() -> i32 {
    (P as i32 - 30).max(P as i32 / 2)
}

fn negligible<const P: u32>(v: Dy<P>, scale: Dy<P>) -> bool {
    v.abs() < scale.ldexp(-shift::<P>())
}

fn bits<const P: u32>(v: Dy<P>) -> i64 {
    if v.is_zero() {
        0
    } else {
        i64::from(v.exponent()) + i64::from(128 - v.mantissa().unsigned_abs().leading_zeros())
    }
}

type Progress<'a> = Option<&'a mut dyn FnMut(&SearchStats)>;

struct Search<'a, const P: u32> {
    n: usize,
    m: usize,
    cols: Vec<Vec<(usize, Dy<P>)>>,
    b: Vec<Dy<P>>,
    lower: Vec<Dy<P>>,
    upper: Vec<Option<Dy<P>>>,
    x: Vec<Dy<P>>,
    state: Vec<State>,
    basis: Vec<usize>,
    binv: Vec<Vec<Dy<P>>>,
    report_cost: Vec<Dy<P>>,
    since_refactor: u64,
    max_pivots: usize,
    stats: SearchStats,
    progress: Progress<'a>,
}

impl<const P: u32> Search<'_, P> {
    fn refactor(&mut self) -> Result<(), String> {
        let m = self.m;
        let mut a = vec![vec![Dy::<P>::ZERO; m]; m];
        for (r, &j) in self.basis.iter().enumerate() {
            for &(i, v) in &self.cols[j] {
                a[i][r] = v;
            }
        }
        let mut inv = vec![vec![Dy::<P>::ZERO; m]; m];
        for (i, row) in inv.iter_mut().enumerate() {
            row[i] = Dy::ONE;
        }
        for k in 0..m {
            let p = (k..m)
                .max_by(|&i, &j| a[i][k].abs().cmp(&a[j][k].abs()))
                .expect("k < m");
            if a[p][k].is_zero() {
                return Err("singular basis at refactor".into());
            }
            a.swap(k, p);
            inv.swap(k, p);
            let piv = a[k][k];
            for v in &mut a[k][k..m] {
                *v = *v / piv;
            }
            for v in &mut inv[k] {
                *v = *v / pv_nonzero(piv);
            }
            let (ak, ik) = (a[k].clone(), inv[k].clone());
            for i in (0..m).filter(|&i| i != k) {
                let f = a[i][k];
                if f.is_zero() {
                    continue;
                }
                for c in k..m {
                    a[i][c] = a[i][c] - f * ak[c];
                }
                for c in 0..m {
                    if !ik[c].is_zero() {
                        inv[i][c] = inv[i][c] - f * ik[c];
                    }
                }
            }
        }
        self.binv = inv;
        self.since_refactor = 0;
        // Recompute the basic values from the nonbasic ones.
        let mut rhs = self.b.clone();
        for j in (0..self.cols.len()).filter(|&j| self.state[j] != State::Basic) {
            if !self.x[j].is_zero() {
                for &(i, v) in &self.cols[j] {
                    rhs[i] = rhs[i] - v * self.x[j];
                }
            }
        }
        for r in 0..m {
            let mut s = Dy::ZERO;
            for (k, &h) in rhs.iter().enumerate() {
                if !h.is_zero() {
                    s = s + self.binv[r][k] * h;
                }
            }
            self.x[self.basis[r]] = s;
        }
        Ok(())
    }

    fn duals(&self, cost: &[Dy<P>]) -> Vec<Dy<P>> {
        let mut y = vec![Dy::ZERO; self.m];
        for (r, &j) in self.basis.iter().enumerate() {
            let cb = cost[j];
            if cb.is_zero() {
                continue;
            }
            for (k, yk) in y.iter_mut().enumerate() {
                *yk = *yk + cb * self.binv[r][k];
            }
        }
        y
    }

    /// The entering variable and its direction (+1 up, −1 down).
    fn price(&self, cost: &[Dy<P>], y: &[Dy<P>], bland: bool) -> Option<(usize, i8)> {
        let mut best: Option<(usize, i8, Dy<P>)> = None;
        for (j, &cj) in cost.iter().enumerate().take(self.cols.len()) {
            let st = self.state[j];
            if st == State::Basic || self.upper[j] == Some(self.lower[j]) {
                continue;
            }
            let mut d = cj;
            let mut scale = cj.abs();
            for &(i, v) in &self.cols[j] {
                let t = y[i] * v;
                d = d - t;
                scale = scale.max(t.abs());
            }
            if d.is_zero() || negligible(d, scale) {
                continue;
            }
            let dir = match st {
                State::Lower if d.is_negative() => 1,
                State::Upper if d.is_positive() => -1,
                State::Interior => -d.sign(),
                _ => continue,
            };
            if bland {
                return Some((j, dir));
            }
            if best.is_none_or(|(_, _, bd)| d.abs() > bd) {
                best = Some((j, dir, d.abs()));
            }
        }
        best.map(|(j, dir, _)| (j, dir))
    }

    fn column(&self, j: usize) -> Vec<Dy<P>> {
        let mut alpha = vec![Dy::ZERO; self.m];
        for &(i, v) in &self.cols[j] {
            for (r, a) in alpha.iter_mut().enumerate() {
                let bi = self.binv[r][i];
                if !bi.is_zero() {
                    *a = *a + bi * v;
                }
            }
        }
        alpha
    }

    fn ratio(&self, j: usize, dir: i8, alpha: &[Dy<P>], bland: bool) -> Option<Block<P>> {
        let colmax = alpha.iter().map(|a| a.abs()).max().unwrap_or(Dy::ZERO);
        let own = if dir > 0 {
            self.upper[j].map(|u| u - self.x[j])
        } else {
            Some(self.x[j] - self.lower[j])
        }
        .map(|t| t.max(Dy::ZERO));
        let mut best: Option<(Dy<P>, usize, bool, Dy<P>)> = None;
        for (r, &a) in alpha.iter().enumerate() {
            if a.is_zero() || negligible(a, colmax) {
                continue;
            }
            let rate = if dir > 0 { -a } else { a };
            let bv = self.basis[r];
            let (raw_gap, bound_scale, to_upper) = if rate.is_negative() {
                let gap = self.x[bv] - self.lower[bv];
                let sc = self.x[bv].abs().max(self.lower[bv].abs()).max(Dy::ONE);
                (gap, sc, false)
            } else {
                match self.upper[bv] {
                    Some(u) => {
                        let gap = u - self.x[bv];
                        let sc = self.x[bv].abs().max(u.abs()).max(Dy::ONE);
                        (gap, sc, true)
                    }
                    None => continue,
                }
            };
            // Clamp refactor rounding dust (~2^-110) at a tight bound to exact zero so
            // consecutive degeneracy and tie-breaking are not broken by refactor noise.
            let theta = if raw_gap <= Dy::ZERO || negligible(raw_gap, bound_scale) {
                Dy::ZERO
            } else if rate.is_negative() {
                raw_gap / (-rate)
            } else {
                raw_gap / rate
            };
            let better = match best {
                None => true,
                Some((bt, br, _, ba)) => {
                    theta < bt
                        || (theta == bt
                            && if bland {
                                bv < self.basis[br]
                            } else {
                                a.abs() > ba
                            })
                }
            };
            if better {
                best = Some((theta, r, to_upper, a.abs()));
            }
        }
        match (own, best) {
            (Some(o), Some((bt, ..))) if o <= bt => Some(Block::Own(o)),
            (_, Some((bt, r, up, _))) => Some(Block::Row(r, up, bt)),
            (Some(o), None) => Some(Block::Own(o)),
            (None, None) => None,
        }
    }

    fn pivot_inverse(&mut self, r: usize, alpha: &[Dy<P>]) {
        let piv = alpha[r];
        let row: Vec<Dy<P>> = self.binv[r].iter().map(|&v| v / piv).collect();
        for (i, &a) in alpha.iter().enumerate() {
            if i == r || a.is_zero() {
                continue;
            }
            for (k, &rv) in row.iter().enumerate() {
                if !rv.is_zero() {
                    self.binv[i][k] = self.binv[i][k] - a * rv;
                }
            }
        }
        self.binv[r] = row;
    }

    fn objective(&self, phase: u8) -> Dy<P> {
        let total = self.cols.len();
        let mut s = Dy::ZERO;
        if phase == 1 {
            for j in self.n + self.m..total {
                s = s + self.x[j];
            }
        } else {
            for j in 0..self.n {
                s = s + self.report_cost[j] * self.x[j];
            }
        }
        s
    }

    fn report(&mut self) {
        let obj = self.objective(self.stats.phase);
        self.stats.objective_sign = obj.sign();
        self.stats.objective_bits = bits(obj);
        if let Some(p) = self.progress.as_mut() {
            p(&self.stats);
        }
    }

    fn run(&mut self, cost: &[Dy<P>]) -> Result<(), String> {
        let mut degenerate = 0u32;
        loop {
            if (self.stats.pivots + self.stats.flips) as usize >= self.max_pivots {
                return Err(format!(
                    "out of pivots ({} pivots, {} flips)",
                    self.stats.pivots, self.stats.flips
                ));
            }
            let bland = degenerate >= DEGENERATE_LIMIT;
            let y = self.duals(cost);
            let Some((j, dir)) = self.price(cost, &y, bland) else {
                return Ok(());
            };
            let alpha = self.column(j);
            let block = self
                .ratio(j, dir, &alpha, bland)
                .ok_or_else(|| format!("unbounded direction at variable {j}"))?;
            let theta = match block {
                Block::Own(t) | Block::Row(_, _, t) => t,
            };
            if theta.is_zero() {
                self.stats.degenerate_pivots += 1;
                degenerate += 1;
                if degenerate == DEGENERATE_LIMIT {
                    self.stats.bland_switches += 1;
                }
            } else {
                degenerate = 0;
            }
            let step = if dir > 0 { theta } else { -theta };
            self.x[j] = self.x[j] + step;
            for (r, &a) in alpha.iter().enumerate() {
                if !a.is_zero() {
                    let bv = self.basis[r];
                    self.x[bv] = self.x[bv] - a * step;
                }
            }
            match block {
                Block::Own(_) => {
                    if dir > 0 {
                        self.state[j] = State::Upper;
                        self.x[j] = self.upper[j].expect("own upper bound");
                    } else {
                        self.state[j] = State::Lower;
                        self.x[j] = self.lower[j];
                    }
                    self.stats.flips += 1;
                }
                Block::Row(r, to_upper, _) => {
                    let out = self.basis[r];
                    if to_upper {
                        self.state[out] = State::Upper;
                        self.x[out] = self.upper[out].expect("leaving upper bound");
                    } else {
                        self.state[out] = State::Lower;
                        self.x[out] = self.lower[out];
                    }
                    self.state[j] = State::Basic;
                    self.basis[r] = j;
                    self.pivot_inverse(r, &alpha);
                    self.stats.pivots += 1;
                    self.since_refactor += 1;
                    if self.since_refactor >= REFACTOR_EVERY {
                        self.refactor()?;
                    }
                    if self.stats.pivots.is_multiple_of(REPORT_EVERY) {
                        self.report();
                    }
                }
            }
        }
    }

    /// Swap every basic artificial for the non-artificial nonbasic column
    /// with the largest entry in its basis row.
    fn drive_out(&mut self) -> Result<(), String> {
        let real = self.n + self.m;
        for r in 0..self.m {
            let art = self.basis[r];
            if art < real {
                continue;
            }
            let rho = self.binv[r].clone();
            let mut best: Option<(usize, Dy<P>)> = None;
            for j in (0..real).filter(|&j| self.state[j] != State::Basic) {
                let mut v = Dy::ZERO;
                for &(i, a) in &self.cols[j] {
                    v = v + rho[i] * a;
                }
                if !v.is_zero() && best.is_none_or(|(_, bv)| v.abs() > bv) {
                    best = Some((j, v.abs()));
                }
            }
            let (j, _) = best.ok_or("no column to replace a basic artificial")?;
            let alpha = self.column(j);
            self.pivot_inverse(r, &alpha);
            self.basis[r] = j;
            self.state[j] = State::Basic;
            self.state[art] = State::Lower;
            self.x[art] = Dy::ZERO;
        }
        self.refactor()
    }
}

fn pv_nonzero<const P: u32>(v: Dy<P>) -> Dy<P> {
    v
}

/// Bounded-variable primal revised simplex search in `Dy<P>`.
pub fn search_primal<const P: u32>(
    lp: &Lp,
    max_pivots: usize,
    progress: Progress<'_>,
) -> (Result<Found<P>, String>, SearchStats) {
    let n = lp.c.len();
    let m = lp.rows.len();
    let stats = SearchStats {
        phase: 1,
        ..SearchStats::default()
    };
    if lp.lower.iter().zip(&lp.upper).any(|(l, u)| l > u) {
        return (Err("infeasible: empty bounds".into()), stats);
    }
    let Scaled { cols, b, c } = blp::scaled(lp);
    let dy = |v: &BigRational| Dy::<P>::from_rational(v);
    let int = |v: &num_bigint::BigInt| Dy::<P>::from_bigint(v);

    // Start point, exactly as `blp`.
    let mut x0 = Vec::with_capacity(n);
    let mut state = Vec::with_capacity(n + 2 * m);
    for (l, u) in lp.lower.iter().zip(&lp.upper) {
        let (v, st) = if l > &BigRational::zero() || l == u {
            (l.clone(), State::Lower)
        } else if u <= &BigRational::zero() {
            (u.clone(), State::Upper)
        } else if l.is_zero() {
            (l.clone(), State::Lower)
        } else {
            (BigRational::zero(), State::Interior)
        };
        x0.push(v);
        state.push(st);
    }
    let mut slack: Vec<BigRational> = b
        .iter()
        .map(|v| BigRational::from_integer(v.clone()))
        .collect();
    for (j, col) in cols.iter().enumerate() {
        if !x0[j].is_zero() {
            for (k, a) in col {
                slack[*k] = &slack[*k] - BigRational::from_integer(a.clone()) * &x0[j];
            }
        }
    }
    let mut dcols: Vec<Vec<(usize, Dy<P>)>> = cols
        .iter()
        .map(|col| col.iter().map(|(i, v)| (*i, int(v))).collect())
        .collect();
    let mut lower: Vec<Dy<P>> = lp.lower.iter().map(dy).collect();
    let mut upper: Vec<Option<Dy<P>>> = lp.upper.iter().map(|u| Some(dy(u))).collect();
    let mut x: Vec<Dy<P>> = x0.iter().map(dy).collect();
    let mut basis = vec![0; m];
    let mut binv = vec![vec![Dy::<P>::ZERO; m]; m];
    let mut art = Vec::new();
    for (i, s) in slack.iter().enumerate() {
        dcols.push(vec![(i, Dy::ONE)]);
        lower.push(Dy::ZERO);
        upper.push(None);
        if s < &BigRational::zero() {
            x.push(Dy::ZERO);
            state.push(State::Lower);
            art.push((i, dy(&-s)));
        } else {
            x.push(dy(s));
            state.push(State::Basic);
            basis[i] = n + i;
            binv[i][i] = Dy::ONE;
        }
    }
    let n_art = art.len();
    for (i, v) in art {
        basis[i] = dcols.len();
        dcols.push(vec![(i, -Dy::<P>::ONE)]);
        lower.push(Dy::ZERO);
        upper.push(None);
        x.push(v);
        state.push(State::Basic);
        binv[i][i] = -Dy::<P>::ONE;
    }
    let total = dcols.len();
    let mut sx = Search {
        n,
        m,
        cols: dcols,
        b: b.iter().map(int).collect(),
        lower,
        upper,
        x,
        state,
        basis,
        binv,
        report_cost: lp.c.iter().map(dy).collect(),
        since_refactor: 0,
        max_pivots,
        stats,
        progress,
    };
    let outcome = (|| {
        if n_art > 0 {
            let mut cost1 = vec![Dy::<P>::ZERO; total];
            for c in cost1.iter_mut().skip(n + m) {
                *c = Dy::ONE;
            }
            sx.run(&cost1)?;
            let sum = sx.objective(1);
            let scale = sx.b.iter().map(|v| v.abs()).fold(Dy::<P>::ONE, Dy::max);
            if !negligible(sum, scale) {
                return Err(format!(
                    "phase 1: infeasible (artificial sum ~2^{})",
                    bits(sum)
                ));
            }
            sx.drive_out()?;
            for j in n + m..total {
                sx.upper[j] = Some(Dy::ZERO);
                sx.x[j] = Dy::ZERO;
            }
        }
        sx.stats.phase = 2;
        let mut cost2: Vec<Dy<P>> = c.iter().map(int).collect();
        cost2.resize(total, Dy::ZERO);
        sx.run(&cost2)?;
        sx.report();
        let nonbasic_pos = (0..n + m)
            .map(|j| match sx.state[j] {
                State::Upper => Pos::Upper,
                State::Interior => Pos::Interior,
                State::Lower | State::Basic => Pos::Lower,
            })
            .collect();
        Ok(Found {
            basis: Basis {
                basic: sx.basis.clone(),
                nonbasic_pos,
            },
            x: sx.x[..n].to_vec(),
            objective: sx.objective(2),
        })
    })();
    (outcome, sx.stats)
}

#[allow(clippy::too_many_arguments)]
fn dual_simplex_run<const P: u32>(
    n: usize,
    m: usize,
    cols: &[Vec<(usize, Dy<P>)>],
    cost: &[Dy<P>],
    lo: &[Dy<P>],
    up: &[Option<Dy<P>>],
    b_rhs: &[Dy<P>],
    basis: &mut [usize],
    st: &mut [State],
    x: &mut [Dy<P>],
    binv: &mut Vec<Vec<Dy<P>>>,
    pi: &mut [Dy<P>],
    max_pivots: usize,
    stats: &mut SearchStats,
    mut progress: Option<&mut Progress<'_>>,
) -> Result<(), String> {
    let recompute_xb =
        |basis: &[usize], st: &[State], x: &mut [Dy<P>], binv: &[Vec<Dy<P>>], b_rhs: &[Dy<P>]| {
            let mut rhs = b_rhs.to_vec();
            for j in 0..(n + m) {
                if st[j] != State::Basic && !x[j].is_zero() {
                    let xj = x[j];
                    for &(i, a) in &cols[j] {
                        rhs[i] = rhs[i] - a * xj;
                    }
                }
            }
            for r in 0..m {
                let mut s = Dy::<P>::ZERO;
                for (k, &rk) in rhs.iter().enumerate() {
                    if !rk.is_zero() && !binv[r][k].is_zero() {
                        s = s + binv[r][k] * rk;
                    }
                }
                x[basis[r]] = s;
            }
        };
    recompute_xb(basis, st, x, binv, b_rhs);

    let tol_exp = -((P as i32 / 2).min(38));
    let tol = Dy::<P>::ONE.ldexp(tol_exp);
    let calc_obj = |x: &[Dy<P>]| -> Dy<P> {
        let mut s = Dy::<P>::ZERO;
        for j in 0..n {
            if !cost[j].is_zero() && !x[j].is_zero() {
                s = s + cost[j] * x[j];
            }
        }
        s
    };

    let mut local_pivots = 0usize;
    loop {
        let mut best_r = None;
        let mut max_viol = tol;
        let mut leave_to_upper = false;
        for (r, &bv) in basis.iter().enumerate() {
            if x[bv] < lo[bv] - max_viol {
                max_viol = lo[bv] - x[bv];
                best_r = Some(r);
                leave_to_upper = false;
            } else if let Some(u) = up[bv] {
                if x[bv] > u + max_viol {
                    max_viol = x[bv] - u;
                    best_r = Some(r);
                    leave_to_upper = true;
                }
            }
        }
        pi.fill(Dy::ZERO);
        for (k, &bk) in basis.iter().enumerate() {
            let cb = cost[bk];
            if !cb.is_zero() {
                for (i, pii) in pi.iter_mut().enumerate() {
                    if !binv[k][i].is_zero() {
                        *pii = *pii + cb * binv[k][i];
                    }
                }
            }
        }
        let Some(r) = best_r else {
            break;
        };
        if (stats.pivots + stats.flips) as usize >= max_pivots {
            return Err(format!(
                "out of pivots ({} pivots, {} flips)",
                stats.pivots, stats.flips
            ));
        }

        let rho = &binv[r];
        let mut cands: Vec<(Dy<P>, usize, Dy<P>)> = Vec::new();
        for j in 0..(n + m) {
            if st[j] == State::Basic || up[j] == Some(lo[j]) {
                continue;
            }
            let mut arj = Dy::<P>::ZERO;
            let mut paj = Dy::<P>::ZERO;
            for &(i, a) in &cols[j] {
                if !rho[i].is_zero() {
                    arj = arj + rho[i] * a;
                }
                if !pi[i].is_zero() {
                    paj = paj + pi[i] * a;
                }
            }
            let dj = cost[j] - paj;
            let s_arj = if leave_to_upper { -arj } else { arj };
            if st[j] == State::Lower && s_arj < -tol {
                let ratio = dj.max(Dy::ZERO) / (-s_arj);
                cands.push((ratio, j, arj));
            } else if st[j] == State::Upper && s_arj > tol {
                let ratio = (-dj).max(Dy::ZERO) / s_arj;
                cands.push((ratio, j, arj));
            }
        }
        if cands.is_empty() {
            return Err(format!(
                "dual unbounded (primal infeasible) at basis row {r}"
            ));
        }
        cands.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.2.abs().cmp(&a.2.abs())));

        let mut rem_viol = max_viol;
        let mut enter_j = cands[0].1;
        let mut enter_ratio = cands[0].0;
        for &(ratio, j, arj) in &cands {
            enter_j = j;
            enter_ratio = ratio;
            if let Some(u) = up[j] {
                let width = u - lo[j];
                let gain = arj.abs() * width;
                if gain + tol < rem_viol {
                    if st[j] == State::Lower {
                        st[j] = State::Upper;
                        let dx = u - x[j];
                        x[j] = u;
                        for &(i, a) in &cols[j] {
                            let adx = a * dx;
                            for k in 0..m {
                                if !binv[k][i].is_zero() {
                                    x[basis[k]] = x[basis[k]] - binv[k][i] * adx;
                                }
                            }
                        }
                    } else {
                        st[j] = State::Lower;
                        let dx = lo[j] - x[j];
                        x[j] = lo[j];
                        for &(i, a) in &cols[j] {
                            let adx = a * dx;
                            for k in 0..m {
                                if !binv[k][i].is_zero() {
                                    x[basis[k]] = x[basis[k]] - binv[k][i] * adx;
                                }
                            }
                        }
                    }
                    rem_viol = rem_viol - gain;
                    stats.flips += 1;
                    continue;
                }
            }
            break;
        }
        if enter_ratio <= tol {
            stats.degenerate_pivots += 1;
        }

        let mut alpha = vec![Dy::<P>::ZERO; m];
        for &(i, a) in &cols[enter_j] {
            for (k, ak) in alpha.iter_mut().enumerate() {
                if !binv[k][i].is_zero() {
                    *ak = *ak + binv[k][i] * a;
                }
            }
        }
        let piv_val = alpha[r];
        let leave_bv = basis[r];
        let target_leave = if leave_to_upper {
            up[leave_bv].expect("leave to upper")
        } else {
            lo[leave_bv]
        };
        let dx_enter = (x[leave_bv] - target_leave) / piv_val;
        x[enter_j] = x[enter_j] + dx_enter;
        for k in 0..m {
            if k != r && !alpha[k].is_zero() {
                x[basis[k]] = x[basis[k]] - alpha[k] * dx_enter;
            }
        }
        x[leave_bv] = target_leave;
        st[leave_bv] = if leave_to_upper {
            State::Upper
        } else {
            State::Lower
        };
        st[enter_j] = State::Basic;
        basis[r] = enter_j;
        stats.pivots += 1;
        local_pivots += 1;

        if local_pivots.is_multiple_of(40) {
            let mut a_mat = vec![vec![Dy::<P>::ZERO; m]; m];
            let mut inv = vec![vec![Dy::<P>::ZERO; m]; m];
            for (i, row) in inv.iter_mut().enumerate() {
                row[i] = Dy::ONE;
            }
            for (col_idx, &bv) in basis.iter().enumerate() {
                for &(row_idx, v) in &cols[bv] {
                    a_mat[row_idx][col_idx] = v;
                }
            }
            for k in 0..m {
                let p_row = (k..m)
                    .max_by(|&i1, &i2| a_mat[i1][k].abs().cmp(&a_mat[i2][k].abs()))
                    .expect("k < m");
                if a_mat[p_row][k].is_zero() {
                    return Err("singular basis at dual refactor".into());
                }
                a_mat.swap(k, p_row);
                inv.swap(k, p_row);
                let pv = a_mat[k][k];
                for v in &mut a_mat[k][k..m] {
                    *v = *v / pv;
                }
                for v in &mut inv[k] {
                    *v = *v / pv;
                }
                let (ak, ik) = (a_mat[k].clone(), inv[k].clone());
                for i in 0..m {
                    if i == k {
                        continue;
                    }
                    let f = a_mat[i][k];
                    if f.is_zero() {
                        continue;
                    }
                    for c_idx in k..m {
                        if !ak[c_idx].is_zero() {
                            a_mat[i][c_idx] = a_mat[i][c_idx] - f * ak[c_idx];
                        }
                    }
                    for c_idx in 0..m {
                        if !ik[c_idx].is_zero() {
                            inv[i][c_idx] = inv[i][c_idx] - f * ik[c_idx];
                        }
                    }
                }
            }
            *binv = inv;
            recompute_xb(basis, st, x, binv, b_rhs);
        } else {
            let inv_row: Vec<Dy<P>> = binv[r].iter().map(|&v| v / piv_val).collect();
            for k in 0..m {
                if k == r {
                    continue;
                }
                let ak = alpha[k];
                if ak.is_zero() {
                    continue;
                }
                for c_idx in 0..m {
                    if !inv_row[c_idx].is_zero() {
                        binv[k][c_idx] = binv[k][c_idx] - ak * inv_row[c_idx];
                    }
                }
            }
            binv[r] = inv_row;
        }

        if let Some(p_opt) = progress.as_mut() {
            if stats.pivots.is_multiple_of(REPORT_EVERY) {
                let obj = calc_obj(x);
                stats.objective_sign = obj.sign();
                stats.objective_bits = bits(obj);
                if let Some(p) = p_opt.as_mut() {
                    p(stats);
                }
            }
        }
    }
    Ok(())
}

/// Bounded-variable Dual Simplex with Bound-Flipping Ratio Test (BFRT) in `Dy<P>`.
/// Starts with all `m` slacks basic (`π = 0`) and each structural variable at
/// its dual-feasible bound (`Upper` if `c_j < 0`, `Lower` if `c_j >= 0`).
pub fn search_dual<const P: u32>(
    lp: &Lp,
    max_pivots: usize,
    mut progress: Progress<'_>,
) -> (Result<Found<P>, String>, SearchStats) {
    let n = lp.c.len();
    let m = lp.rows.len();
    let mut stats = SearchStats {
        phase: 2,
        ..SearchStats::default()
    };
    if lp.lower.iter().zip(&lp.upper).any(|(l, u)| l > u) {
        return (Err("infeasible: empty bounds".into()), stats);
    }
    let dy = |v: &BigRational| Dy::<P>::from_rational(v);
    let mut cols: Vec<Vec<(usize, Dy<P>)>> = vec![Vec::new(); n + m];
    for (i, (row, _)) in lp.rows.iter().enumerate() {
        for (j, val) in row.iter().enumerate() {
            if !val.is_zero() {
                let a = dy(val);
                if !a.is_zero() {
                    cols[j].push((i, a));
                }
            }
        }
        cols[n + i].push((i, Dy::ONE));
    }
    let b_rhs: Vec<Dy<P>> = lp.rows.iter().map(|(_, bi)| dy(bi)).collect();
    let mut cost = vec![Dy::<P>::ZERO; n + m];
    let mut lo = vec![Dy::<P>::ZERO; n + m];
    let mut up: Vec<Option<Dy<P>>> = vec![None; n + m];
    for j in 0..n {
        cost[j] = dy(&lp.c[j]);
        lo[j] = dy(&lp.lower[j]);
        up[j] = Some(dy(&lp.upper[j]));
    }

    let mut st = vec![State::Lower; n + m];
    let mut x = vec![Dy::<P>::ZERO; n + m];
    for j in 0..n {
        if cost[j].is_negative() {
            st[j] = State::Upper;
            x[j] = up[j].expect("structural upper");
        } else {
            st[j] = State::Lower;
            x[j] = lo[j];
        }
    }
    let mut basis: Vec<usize> = (0..m).map(|i| n + i).collect();
    for i in 0..m {
        st[n + i] = State::Basic;
    }
    let mut binv = vec![vec![Dy::<P>::ZERO; m]; m];
    for (i, row) in binv.iter_mut().enumerate() {
        row[i] = Dy::ONE;
    }
    let mut pi = vec![Dy::<P>::ZERO; m];

    let outcome = dual_simplex_run::<P>(
        n,
        m,
        &cols,
        &cost,
        &lo,
        &up,
        &b_rhs,
        &mut basis,
        &mut st,
        &mut x,
        &mut binv,
        &mut pi,
        max_pivots,
        &mut stats,
        Some(&mut progress),
    )
    .map(|()| {
        let mut obj = Dy::<P>::ZERO;
        for j in 0..n {
            if !cost[j].is_zero() && !x[j].is_zero() {
                obj = obj + cost[j] * x[j];
            }
        }
        stats.objective_sign = obj.sign();
        stats.objective_bits = bits(obj);
        if let Some(p) = progress.as_mut() {
            p(&stats);
        }
        let nonbasic_pos = (0..(n + m))
            .map(|j| match st[j] {
                State::Upper => Pos::Upper,
                State::Interior => Pos::Interior,
                State::Lower | State::Basic => Pos::Lower,
            })
            .collect();
        Found {
            basis: Basis {
                basic: basis,
                nonbasic_pos,
            },
            x: x[..n].to_vec(),
            objective: obj,
        }
    });
    (outcome, stats)
}

#[derive(Debug, Clone)]
struct DisjSpec {
    coeff_idx: usize,
    q0: i64,
    four_shift: BigRational,
}

/// Detect `bb::relaxation`'s 3-row fractional disjunction blocks at the tail
/// of `lp` (when `q0 = ±7`), project them onto the `m0 × n0` structural guard
/// LP, and return `(proj_lp, specs)` for lifting the basis back to `lp`.
fn try_project_disjunction_lp(lp: &Lp) -> Option<(Lp, Vec<DisjSpec>)> {
    let nv = lp.c.len();
    let m = lp.rows.len();
    let one = BigRational::one();
    let neg_one = -BigRational::one();
    let zero = BigRational::zero();
    let fourteen = BigRational::from_integer(BigInt::from(14));
    let neg_fourteen = -fourteen.clone();

    // Count trailing 3-row disjunction triples from the end of `lp.rows`.
    let mut f_rev = Vec::new();
    let mut cur_m = m;
    let mut cur_n = nv;
    while cur_m >= 3 && cur_n >= 2 {
        let l_var = cur_n - 2;
        let r_var = cur_n - 1;
        if lp.lower[l_var] != zero
            || lp.upper[l_var] != one
            || lp.lower[r_var] != zero
            || lp.upper[r_var] != one
        {
            break;
        }
        let (a0, b0) = &lp.rows[cur_m - 3];
        let (a1, b1) = &lp.rows[cur_m - 2];
        let (a2, b2) = &lp.rows[cur_m - 1];
        if *b0 != one || !b1.is_zero() || !b2.is_zero() {
            break;
        }
        if a0[l_var] != one
            || a0[r_var] != one
            || a0
                .iter()
                .enumerate()
                .any(|(j, v)| j != l_var && j != r_var && !v.is_zero())
        {
            break;
        }
        if a1[r_var] != one || a2[l_var] != one {
            break;
        }
        // Find unique structural index `i < l_var` with `a1[i] == -1` and `a2[i] == 1`.
        let mut i_cand = None;
        let mut ok = true;
        for j in 0..nv {
            if j == l_var || j == r_var {
                continue;
            }
            if !a1[j].is_zero() || !a2[j].is_zero() {
                if i_cand.is_none() && j < l_var && a1[j] == neg_one && a2[j] == one {
                    i_cand = Some(j);
                } else {
                    ok = false;
                    break;
                }
            }
        }
        let Some(coeff_idx) = i_cand else {
            break;
        };
        if !ok {
            break;
        }
        let q0 = if a1[l_var] == neg_fourteen
            && a2[r_var].is_zero()
            && lp.lower[coeff_idx] == neg_fourteen
            && lp.upper[coeff_idx].is_zero()
        {
            7i64
        } else if a1[l_var].is_zero()
            && a2[r_var] == neg_fourteen
            && lp.lower[coeff_idx].is_zero()
            && lp.upper[coeff_idx] == fourteen
        {
            -7i64
        } else {
            break;
        };
        // Recover `four_shift = 4 * shift_f` from `c[l_var] / c[coeff_idx]` or a guard row.
        let four_shift = if !lp.c[coeff_idx].is_zero() {
            &lp.c[l_var] / &lp.c[coeff_idx]
        } else {
            let mut fs = None;
            for (arow, _) in &lp.rows[..cur_m - 3] {
                if !arow[coeff_idx].is_zero() {
                    fs = Some(&arow[l_var] / &arow[coeff_idx]);
                    break;
                }
            }
            fs?
        };
        if (q0 == 7 && four_shift.is_negative()) || (q0 == -7 && four_shift.is_positive()) {
            break;
        }
        f_rev.push(DisjSpec {
            coeff_idx,
            q0,
            four_shift,
        });
        cur_m -= 3;
        cur_n -= 2;
    }

    if f_rev.is_empty() {
        return None;
    }
    f_rev.reverse();
    let m0 = cur_m;
    let n0 = cur_n;
    let inv14 = BigRational::new(BigInt::from(1), BigInt::from(14));
    let neg_inv14 = -inv14.clone();

    let mut lower: Vec<BigRational> = lp.lower[..n0].to_vec();
    let mut upper: Vec<BigRational> = lp.upper[..n0].to_vec();
    for spec in &f_rev {
        let i = spec.coeff_idx;
        if spec.q0 == 7 {
            let edge = &neg_inv14 + &spec.four_shift;
            lower[i] = &neg_fourteen + &spec.four_shift;
            upper[i] = edge.max(zero.clone());
        } else {
            let edge = &inv14 + &spec.four_shift;
            lower[i] = edge.min(zero.clone());
            upper[i] = &fourteen + &spec.four_shift;
        }
    }
    let rows: Vec<(Vec<BigRational>, BigRational)> = lp.rows[..m0]
        .iter()
        .map(|(a, b)| (a[..n0].to_vec(), b.clone()))
        .collect();
    let proj_lp = Lp {
        c: lp.c[..n0].to_vec(),
        rows,
        lower,
        upper,
    };
    Some((proj_lp, f_rev))
}

fn lift_projected_found<const P: u32>(
    lp: &Lp,
    found_proj: Found<P>,
    specs: &[DisjSpec],
) -> Option<Found<P>> {
    let f_count = specs.len();
    let n0 = lp.c.len() - 2 * f_count;
    let m0 = lp.rows.len() - 3 * f_count;
    let nv = lp.c.len();
    let m = lp.rows.len();

    let mut is_frac = vec![false; n0];
    for spec in specs {
        is_frac[spec.coeff_idx] = true;
    }
    for &bv in &found_proj.basis.basic {
        if bv < n0 && is_frac[bv] {
            return None;
        }
    }

    let mut basic = Vec::with_capacity(m);
    let mut nonbasic_pos = vec![Pos::Lower; nv + m];
    for &bv in &found_proj.basis.basic {
        if bv < n0 {
            basic.push(bv);
        } else {
            basic.push(bv + 2 * f_count);
        }
    }
    for j in 0..n0 {
        if !is_frac[j] {
            nonbasic_pos[j] = found_proj.basis.nonbasic_pos[j];
        }
    }
    nonbasic_pos[nv..(nv + m0)].copy_from_slice(&found_proj.basis.nonbasic_pos[n0..(n0 + m0)]);

    let mut x_full = vec![Dy::<P>::ZERO; nv];
    x_full[..n0].copy_from_slice(&found_proj.x[..n0]);
    let inv14 = BigRational::new(BigInt::from(1), BigInt::from(14));
    let thirteen14 = BigRational::new(BigInt::from(13), BigInt::from(14));
    let dy = |v: &BigRational| Dy::<P>::from_rational(v);

    for (f, spec) in specs.iter().enumerate() {
        let i = spec.coeff_idx;
        let l_var = n0 + 2 * f;
        let r_var = n0 + 2 * f + 1;
        let s0 = nv + m0 + 3 * f;
        let s1 = nv + m0 + 3 * f + 1;
        let s2 = nv + m0 + 3 * f + 2;
        let at_lower = found_proj.basis.nonbasic_pos[i] == Pos::Lower;

        if spec.q0 == 7 && at_lower {
            basic.push(s0);
            basic.push(l_var);
            basic.push(s2);
            nonbasic_pos[i] = Pos::Lower;
            nonbasic_pos[l_var] = Pos::Lower;
            nonbasic_pos[r_var] = Pos::Lower;
            nonbasic_pos[s0] = Pos::Lower;
            nonbasic_pos[s1] = Pos::Lower;
            nonbasic_pos[s2] = Pos::Lower;
            x_full[i] = dy(&lp.lower[i]);
            x_full[l_var] = Dy::ONE;
            x_full[r_var] = Dy::ZERO;
        } else if spec.q0 == 7 && !at_lower {
            let edge = -&inv14 + &spec.four_shift;
            if edge.is_positive() {
                basic.push(i);
                basic.push(l_var);
                basic.push(r_var);
                nonbasic_pos[i] = Pos::Lower;
                x_full[i] = dy(&(-&inv14));
                x_full[l_var] = dy(&inv14);
                x_full[r_var] = dy(&thirteen14);
            } else {
                basic.push(s0);
                basic.push(l_var);
                basic.push(r_var);
                nonbasic_pos[i] = Pos::Upper;
                x_full[i] = Dy::ZERO;
                x_full[l_var] = Dy::ZERO;
                x_full[r_var] = Dy::ZERO;
            }
            nonbasic_pos[l_var] = Pos::Lower;
            nonbasic_pos[r_var] = Pos::Lower;
            nonbasic_pos[s0] = Pos::Lower;
            nonbasic_pos[s1] = Pos::Lower;
            nonbasic_pos[s2] = Pos::Lower;
        } else if spec.q0 == -7 && at_lower {
            let edge = &inv14 + &spec.four_shift;
            if edge.is_negative() {
                basic.push(i);
                basic.push(l_var);
                basic.push(r_var);
                x_full[i] = dy(&inv14);
                x_full[l_var] = dy(&thirteen14);
                x_full[r_var] = dy(&inv14);
            } else {
                basic.push(s0);
                basic.push(l_var);
                basic.push(r_var);
                x_full[i] = Dy::ZERO;
                x_full[l_var] = Dy::ZERO;
                x_full[r_var] = Dy::ZERO;
            }
            nonbasic_pos[i] = Pos::Lower;
            nonbasic_pos[l_var] = Pos::Lower;
            nonbasic_pos[r_var] = Pos::Lower;
            nonbasic_pos[s0] = Pos::Lower;
            nonbasic_pos[s1] = Pos::Lower;
            nonbasic_pos[s2] = Pos::Lower;
        } else {
            basic.push(s0);
            basic.push(s1);
            basic.push(r_var);
            nonbasic_pos[i] = Pos::Upper;
            nonbasic_pos[l_var] = Pos::Lower;
            nonbasic_pos[r_var] = Pos::Lower;
            nonbasic_pos[s0] = Pos::Lower;
            nonbasic_pos[s1] = Pos::Lower;
            nonbasic_pos[s2] = Pos::Lower;
            x_full[i] = dy(&lp.upper[i]);
            x_full[l_var] = Dy::ZERO;
            x_full[r_var] = Dy::ONE;
        }
    }

    Some(Found {
        basis: Basis {
            basic,
            nonbasic_pos,
        },
        x: x_full,
        objective: found_proj.objective,
    })
}

/// Run the search; `progress` is called every 25 pivots.
pub fn search<const P: u32>(
    lp: &Lp,
    max_pivots: usize,
    progress: Progress<'_>,
) -> (Result<Found<P>, String>, SearchStats) {
    if lp.rows.len() >= 64 {
        if let Some((proj_lp, specs)) = try_project_disjunction_lp(lp) {
            let (res, stats) = search_dual::<P>(&proj_lp, max_pivots, progress);
            if let Ok(found_proj) = res {
                if let Some(lifted) = lift_projected_found(lp, found_proj, &specs) {
                    return (Ok(lifted), stats);
                }
            }
            return search_primal::<P>(lp, max_pivots, None);
        }
        return search_dual::<P>(lp, max_pivots, progress);
    }
    search_primal::<P>(lp, max_pivots, progress)
}

/// Propose a basis in `blp`'s index space, or `None` when the search runs
/// out of pivots or finds the LP infeasible.
pub fn search_basis<const P: u32>(lp: &Lp, max_pivots: usize) -> Option<Basis> {
    search::<P>(lp, max_pivots, None).0.ok().map(|f| f.basis)
}

/// Search, then prove: the result is `certify_basis`'s, never the search's.
pub fn solve_search_certify<const P: u32>(lp: &Lp, max_pivots: usize) -> Result<LpResult, String> {
    let found = search::<P>(lp, max_pivots, None)
        .0
        .map_err(|e| format!("search: {e}"))?;
    blp::certify_basis(lp, &found.basis).map_err(|e| format!("certificate: {e}"))
}

/// Which path `solve_search_certify_repair` took.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairPath {
    /// The search's basis passed `certify_basis`.
    Certified,
    /// The certificate rejected the search's basis; `blp::solve_from_basis`
    /// finished from it (or fell back to its cold start, per `exact_start`).
    Repaired,
    /// The search proposed no basis; `blp` ran from its cold start.
    Cold,
}

/// `search_pivots` counts the search's basis changes; `exact_pivots` those of
/// the exact simplex (0 when the certificate passed). `exact_start` is how
/// the exact run began, `None` when it did not run.
#[derive(Debug, Clone)]
pub struct RepairStats {
    pub path: RepairPath,
    pub search_pivots: u64,
    pub exact_pivots: u64,
    pub exact_start: Option<blp::Start>,
}

/// Search, prove, and repair: a certified search basis is returned as is;
/// otherwise the exact simplex finishes from the search's basis, or from its
/// own cold start when the search found none. The answer is always exact.
pub fn solve_search_certify_repair<const P: u32>(
    lp: &Lp,
    max_pivots: usize,
) -> (LpResult, RepairStats) {
    let (found, sstats) = search::<P>(lp, max_pivots, None);
    let mut stats = RepairStats {
        path: RepairPath::Cold,
        search_pivots: sstats.pivots,
        exact_pivots: 0,
        exact_start: None,
    };
    let (result, exact) = match found {
        Ok(f) => {
            if let Ok(r) = blp::certify_basis(lp, &f.basis) {
                stats.path = RepairPath::Certified;
                return (r, stats);
            }
            stats.path = RepairPath::Repaired;
            blp::solve_from_basis_with_stats(lp, &f.basis, None)
        }
        Err(_) => blp::solve_with_stats(lp, None),
    };
    stats.exact_pivots = exact.pivots;
    stats.exact_start = Some(exact.start);
    (result, stats)
}

/// Convex-hull `δ`-relaxation of `p`: `minimise g·δ` subject to `-J·δ <= 0`
/// and `min_{q in -7..=7} δ_i(q) <= δ_i <= max_{q in -7..=7} δ_i(q)`.
pub fn delta_hull_lp(p: &Problem) -> Lp {
    let n = p.n();
    let cs = p.coeffs();
    let mut lower = vec![BigRational::zero(); n];
    let mut upper = vec![BigRational::zero(); n];
    for i in 0..n {
        for q in -7..=7 {
            let delta = if q == cs[i].q0 {
                BigRational::zero()
            } else {
                BigRational::new(BigInt::from(q - cs[i].q0), BigInt::from(4)) + &cs[i].shift
            };
            if delta < lower[i] {
                lower[i] = delta.clone();
            }
            if delta > upper[i] {
                upper[i] = delta;
            }
        }
    }
    let rows: Vec<(Vec<BigRational>, BigRational)> = p
        .guards
        .iter()
        .map(|row| {
            let neg_row: Vec<BigRational> = row.iter().map(|v| -v).collect();
            (neg_row, BigRational::zero())
        })
        .collect();
    Lp {
        c: p.g.clone(),
        rows,
        lower,
        upper,
    }
}

/// Basis-free exact rational Weak Duality certificate: for any non-negative
/// dual multiplier vector `y >= 0` on the guard rows `J·δ >= 0`, every guard-
/// feasible discrete assignment `δ` satisfies
/// `g·δ >= g·δ - y·(J·δ) >= Σ_i min_{q in -7..=7} (g_i - (J^T y)_i) δ_i(q)`.
pub fn certify_lagrangian_dual_bound(
    p: &Problem,
    y: &[BigRational],
) -> Result<BigRational, String> {
    if y.len() != p.guards.len() {
        return Err(format!(
            "dual multiplier length {} != {} guards",
            y.len(),
            p.guards.len()
        ));
    }
    for (r, yr) in y.iter().enumerate() {
        if yr.is_negative() {
            return Err(format!("dual multiplier {r} is negative"));
        }
    }
    let cs = p.coeffs();
    let mut bound = BigRational::zero();
    for (i, c) in cs.iter().enumerate() {
        let mut rc = p.g[i].clone();
        for (r, yr) in y.iter().enumerate() {
            if !yr.is_zero() && !p.guards[r][i].is_zero() {
                rc -= yr * &p.guards[r][i];
            }
        }
        let mut best_i = BigRational::zero();
        for q in -7..=7 {
            if q == c.q0 {
                continue;
            }
            let delta = BigRational::new(BigInt::from(q - c.q0), BigInt::from(4)) + &c.shift;
            let contrib = &rc * &delta;
            if contrib < best_i {
                best_i = contrib;
            }
        }
        bound += best_i;
    }
    Ok(bound)
}

/// Exact rational Reduced-Cost Bound Tightening (RCBT) certificate.
#[derive(Debug, Clone)]
pub struct ReducedCostFixing {
    /// Exact rational Lagrangian dual lower bound `L(y)`.
    pub lower_bound: BigRational,
    /// Exact rational optimality gap `upper_bound - lower_bound` (`>= 0`).
    pub gap: BigRational,
    /// Surviving legal destination codes `q in -7..=7` for each coefficient `i`.
    pub surviving_codes: Vec<Vec<i64>>,
    /// Number of coefficients `i` uniquely fixed (`surviving_codes[i].len() == 1`).
    pub fixed_coeffs: usize,
    /// Number of coefficients `i` restricted to at most 2 codes (`<= 2`).
    pub binary_or_fixed_coeffs: usize,
    /// Total `(i, q)` destination codes provably eliminated out of `15 * n`.
    pub eliminated_codes: usize,
    /// Total `(i, q)` destination codes before reduction (`15 * n`).
    pub total_codes: usize,
}

/// Prove in exact rational arithmetic which discrete codes `q in -7..=7` cannot
/// appear in any guard-feasible discrete assignment with objective `<= upper_bound`.
/// By Weak Duality, for any `y >= 0`, every guard-feasible discrete assignment `δ`
/// with `g·δ <= upper_bound` satisfies
/// `L(y) + Σ_i ((g_i - (J^T y)_i) δ_i(q_i) - min_{q'} (g_i - (J^T y)_i) δ_i(q')) <= upper_bound`.
/// Hence any code `q` whose exact reduced-cost penalty exceeds `upper_bound - L(y)`
/// is provably eliminated.
pub fn certify_reduced_cost_fixing(
    p: &Problem,
    y: &[BigRational],
    upper_bound: &BigRational,
) -> Result<ReducedCostFixing, String> {
    if y.len() != p.guards.len() {
        return Err(format!(
            "dual multiplier length {} != {} guards",
            y.len(),
            p.guards.len()
        ));
    }
    for (r, yr) in y.iter().enumerate() {
        if yr.is_negative() {
            return Err(format!("dual multiplier {r} is negative"));
        }
    }
    let cs = p.coeffs();
    let n = cs.len();
    let mut lower_bound = BigRational::zero();
    let mut per_coeff: Vec<(BigRational, Vec<(i64, BigRational)>)> = Vec::with_capacity(n);

    for (i, c) in cs.iter().enumerate() {
        let mut rc = p.g[i].clone();
        for (r, yr) in y.iter().enumerate() {
            if !yr.is_zero() && !p.guards[r][i].is_zero() {
                rc -= yr * &p.guards[r][i];
            }
        }
        let mut best_i = BigRational::zero();
        let mut contribs = Vec::with_capacity(15);
        for q in -7..=7 {
            let contrib = if q == c.q0 {
                BigRational::zero()
            } else {
                let delta = BigRational::new(BigInt::from(q - c.q0), BigInt::from(4)) + &c.shift;
                &rc * &delta
            };
            if contrib < best_i {
                best_i = contrib.clone();
            }
            contribs.push((q, contrib));
        }
        lower_bound += &best_i;
        per_coeff.push((best_i, contribs));
    }

    if upper_bound < &lower_bound {
        return Err(format!(
            "upper_bound {upper_bound} < lower_bound {lower_bound}"
        ));
    }
    let gap = upper_bound - &lower_bound;
    let total_codes = 15 * n;
    let mut surviving_codes = Vec::with_capacity(n);
    let mut fixed_coeffs = 0usize;
    let mut binary_or_fixed_coeffs = 0usize;
    let mut eliminated_codes = 0usize;

    for (best_i, contribs) in per_coeff {
        let mut surv = Vec::with_capacity(15);
        for (q, contrib) in contribs {
            let penalty = contrib - &best_i;
            if penalty <= gap {
                surv.push(q);
            } else {
                eliminated_codes += 1;
            }
        }
        if surv.len() == 1 {
            fixed_coeffs += 1;
        }
        if surv.len() <= 2 {
            binary_or_fixed_coeffs += 1;
        }
        surviving_codes.push(surv);
    }

    Ok(ReducedCostFixing {
        lower_bound,
        gap,
        surviving_codes,
        fixed_coeffs,
        binary_or_fixed_coeffs,
        eliminated_codes,
        total_codes,
    })
}

/// Result of `lp_guided_discrete_solve`.
#[derive(Debug, Clone)]
pub struct DiscreteLpResult {
    pub delta_lp_basis: Basis,
    pub dual_y: Vec<BigRational>,
    pub codes: Vec<i64>,
    pub deltas: Vec<BigRational>,
    pub objective: BigRational,
    pub guard_min: BigRational,
    pub active_guards: usize,
    pub moved_coeffs: usize,
    pub full_guard_vector_matches: bool,
    pub total_pivots: u64,
}

/// Warm-started `Dy<P>` Dual Simplex margin-tightening + exact rational
/// single-coefficient polish. Returns the `iter 0` basis and dual multipliers
/// of `delta_hull_lp(p)` together with an exact guard-feasible discrete
/// assignment verified by independent recomputation over `BigRational`.
pub fn lp_guided_discrete_solve<const P: u32>(p: &Problem) -> Result<DiscreteLpResult, String> {
    let n = p.n();
    let m = p.guards.len();
    let cs = p.coeffs();
    let dy = |v: &BigRational| Dy::<P>::from_rational(v);

    let mut legal_deltas: Vec<Vec<(i64, BigRational, Dy<P>)>> = Vec::with_capacity(n);
    let mut lower = vec![BigRational::zero(); n];
    let mut upper = vec![BigRational::zero(); n];
    for i in 0..n {
        let mut v = Vec::with_capacity(15);
        for q in -7..=7 {
            let delta = if q == cs[i].q0 {
                BigRational::zero()
            } else {
                BigRational::new(BigInt::from(q - cs[i].q0), BigInt::from(4)) + &cs[i].shift
            };
            if delta < lower[i] {
                lower[i] = delta.clone();
            }
            if delta > upper[i] {
                upper[i] = delta.clone();
            }
            let d_dy = dy(&delta);
            v.push((q, delta, d_dy));
        }
        legal_deltas.push(v);
    }

    let mut cols: Vec<Vec<(usize, Dy<P>)>> = vec![Vec::new(); n + m];
    let mut j_dy: Vec<Vec<Dy<P>>> = vec![vec![Dy::<P>::ZERO; n]; m];
    for (r, row) in p.guards.iter().enumerate() {
        for (j, val) in row.iter().enumerate() {
            if !val.is_zero() {
                let d = dy(val);
                j_dy[r][j] = d;
                let neg_d = -d;
                if !neg_d.is_zero() {
                    cols[j].push((r, neg_d));
                }
            }
        }
        cols[n + r].push((r, Dy::ONE));
    }
    let g_dy: Vec<Dy<P>> = p.g.iter().map(dy).collect();
    let mut cost = vec![Dy::<P>::ZERO; n + m];
    let mut lo = vec![Dy::<P>::ZERO; n + m];
    let mut up: Vec<Option<Dy<P>>> = vec![None; n + m];
    for j in 0..n {
        cost[j] = g_dy[j];
        lo[j] = dy(&lower[j]);
        up[j] = Some(dy(&upper[j]));
    }

    let mut st = vec![State::Lower; n + m];
    let mut x = vec![Dy::<P>::ZERO; n + m];
    for j in 0..n {
        if cost[j].is_negative() {
            st[j] = State::Upper;
            x[j] = up[j].expect("structural upper");
        } else {
            st[j] = State::Lower;
            x[j] = lo[j];
        }
    }
    let mut basis: Vec<usize> = (0..m).map(|i| n + i).collect();
    for i in 0..m {
        st[n + i] = State::Basic;
    }
    let mut binv = vec![vec![Dy::<P>::ZERO; m]; m];
    for (i, row) in binv.iter_mut().enumerate() {
        row[i] = Dy::ONE;
    }
    let b_rhs0 = vec![Dy::<P>::ZERO; m];
    let mut pi = vec![Dy::<P>::ZERO; m];
    let mut total_pivots = 0u64;

    let mut iter0_stats = SearchStats::default();
    dual_simplex_run::<P>(
        n,
        m,
        &cols,
        &cost,
        &lo,
        &up,
        &b_rhs0,
        &mut basis,
        &mut st,
        &mut x,
        &mut binv,
        &mut pi,
        100_000,
        &mut iter0_stats,
        None,
    )?;
    total_pivots += iter0_stats.pivots;

    let nonbasic_pos0 = (0..(n + m))
        .map(|j| match st[j] {
            State::Upper => Pos::Upper,
            State::Interior => Pos::Interior,
            State::Lower | State::Basic => Pos::Lower,
        })
        .collect();
    let basis_iter0 = Basis {
        basic: basis.clone(),
        nonbasic_pos: nonbasic_pos0,
    };
    let dual_y_iter0: Vec<BigRational> = pi
        .iter()
        .map(|&p_val| {
            let neg_p = -p_val;
            if neg_p.is_positive() {
                neg_p.to_rational()
            } else {
                BigRational::zero()
            }
        })
        .collect();

    let min_slack_dy = -Dy::<P>::ONE.ldexp(-35);
    let run_polish = |cur_q: &mut [i64],
                      disc_rat: &mut [BigRational],
                      disc_dy: &mut [Dy<P>],
                      exact_guards: &mut [BigRational],
                      guard_dy: &mut [Dy<P>],
                      exact_obj: &mut BigRational| {
        for _pass in 0..25 {
            let mut improved = false;
            for i in 0..n {
                let gi = g_dy[i];
                let cur_d = disc_dy[i];
                let mut best_cand: Option<(i64, BigRational, Dy<P>, Dy<P>)> = None;
                for &(q, ref d_rat, d_dy) in &legal_deltas[i] {
                    if q == cur_q[i] {
                        continue;
                    }
                    let diff_dy = d_dy - cur_d;
                    let gain_dy = gi * diff_dy;
                    if gain_dy.is_negative() {
                        if let Some((_, _, _, best_gain)) = best_cand {
                            if gain_dy >= best_gain {
                                continue;
                            }
                        }
                        let mut ok = true;
                        for r in 0..m {
                            if !j_dy[r][i].is_zero()
                                && guard_dy[r] + j_dy[r][i] * diff_dy < min_slack_dy
                            {
                                ok = false;
                                break;
                            }
                        }
                        if ok {
                            let diff_rat = d_rat - &disc_rat[i];
                            if (&p.g[i] * &diff_rat).is_negative()
                                && (0..m).all(|r| {
                                    !(&exact_guards[r] + &p.guards[r][i] * &diff_rat).is_negative()
                                })
                            {
                                best_cand = Some((q, d_rat.clone(), d_dy, gain_dy));
                            }
                        }
                    }
                }
                if let Some((q, d_rat, d_dy, _)) = best_cand {
                    let diff_rat = &d_rat - &disc_rat[i];
                    for r in 0..m {
                        if !p.guards[r][i].is_zero() {
                            exact_guards[r] += &p.guards[r][i] * &diff_rat;
                            guard_dy[r] = dy(&exact_guards[r]);
                        }
                    }
                    *exact_obj += &p.g[i] * &diff_rat;
                    cur_q[i] = q;
                    disc_rat[i] = d_rat;
                    disc_dy[i] = d_dy;
                    improved = true;
                }
            }
            if !improved {
                break;
            }
        }

        // 2-coefficient (pair) exact polish on blocked improving columns.
        for _pair_pass in 0..6 {
            let mut pair_improved = false;
            let mut blocked_i1: Vec<(usize, i64, BigRational, Dy<P>, Dy<P>)> = Vec::new();
            for i1 in 0..n {
                let gi1 = g_dy[i1];
                if gi1.is_zero() {
                    continue;
                }
                let step_q = if gi1.is_negative() {
                    cur_q[i1] + 1
                } else {
                    cur_q[i1] - 1
                };
                if !(-7..=7).contains(&step_q) {
                    continue;
                }
                let (q1, ref d1_rat, d1_dy) = legal_deltas[i1][(step_q + 7) as usize];
                let diff1_dy = d1_dy - disc_dy[i1];
                let gain1_dy = gi1 * diff1_dy;
                if gain1_dy.is_negative() {
                    blocked_i1.push((i1, q1, d1_rat.clone(), d1_dy, gain1_dy));
                }
            }
            blocked_i1.sort_by_key(|a| a.4);

            for (i1, q1, d1_rat, d1_dy, gain1_dy) in blocked_i1 {
                let diff1_dy = d1_dy - disc_dy[i1];
                let mut viol_rows = Vec::new();
                for r in 0..m {
                    if !j_dy[r][i1].is_zero() {
                        let new_s = guard_dy[r] + j_dy[r][i1] * diff1_dy;
                        if new_s.is_negative() {
                            viol_rows.push((r, new_s));
                        }
                    }
                }
                if viol_rows.is_empty() || viol_rows.len() > 8 {
                    continue;
                }
                let mut best_pair: Option<(usize, i64, BigRational, Dy<P>, Dy<P>)> = None;
                for i2 in 0..n {
                    if i2 == i1 {
                        continue;
                    }
                    let r0 = viol_rows[0].0;
                    if j_dy[r0][i2].is_zero() {
                        continue;
                    }
                    for step2 in [-1i64, 1i64] {
                        let q2 = cur_q[i2] + step2;
                        if !(-7..=7).contains(&q2) {
                            continue;
                        }
                        let (_, ref d2_rat, d2_dy) = legal_deltas[i2][(q2 + 7) as usize];
                        let diff2_dy = d2_dy - disc_dy[i2];
                        let net_gain_dy = gain1_dy + g_dy[i2] * diff2_dy;
                        if !net_gain_dy.is_negative() {
                            continue;
                        }
                        if let Some((_, _, _, _, best_g)) = best_pair {
                            if net_gain_dy >= best_g {
                                continue;
                            }
                        }
                        if viol_rows
                            .iter()
                            .any(|&(r, new_s)| new_s + j_dy[r][i2] * diff2_dy < min_slack_dy)
                        {
                            continue;
                        }
                        let mut all_ok = true;
                        for r in 0..m {
                            let s = guard_dy[r] + j_dy[r][i1] * diff1_dy + j_dy[r][i2] * diff2_dy;
                            if s < min_slack_dy {
                                all_ok = false;
                                break;
                            }
                        }
                        if all_ok {
                            let diff1_rat = &d1_rat - &disc_rat[i1];
                            let diff2_rat = d2_rat - &disc_rat[i2];
                            let net_rat = &p.g[i1] * &diff1_rat + &p.g[i2] * &diff2_rat;
                            if net_rat.is_negative()
                                && (0..m).all(|r| {
                                    !(&exact_guards[r]
                                        + &p.guards[r][i1] * &diff1_rat
                                        + &p.guards[r][i2] * &diff2_rat)
                                        .is_negative()
                                })
                            {
                                best_pair = Some((i2, q2, d2_rat.clone(), d2_dy, net_gain_dy));
                            }
                        }
                    }
                }
                if let Some((i2, q2, d2_rat, d2_dy, _)) = best_pair {
                    let diff1_rat = &d1_rat - &disc_rat[i1];
                    let diff2_rat = &d2_rat - &disc_rat[i2];
                    for r in 0..m {
                        if !p.guards[r][i1].is_zero() || !p.guards[r][i2].is_zero() {
                            exact_guards[r] +=
                                &p.guards[r][i1] * &diff1_rat + &p.guards[r][i2] * &diff2_rat;
                            guard_dy[r] = dy(&exact_guards[r]);
                        }
                    }
                    *exact_obj += &p.g[i1] * &diff1_rat + &p.g[i2] * &diff2_rat;
                    cur_q[i1] = q1;
                    disc_rat[i1] = d1_rat;
                    disc_dy[i1] = d1_dy;
                    cur_q[i2] = q2;
                    disc_rat[i2] = d2_rat;
                    disc_dy[i2] = d2_dy;
                    pair_improved = true;
                }
            }
            if !pair_improved {
                break;
            }
        }
    };

    let schedules: &[(i32, i64, i64)] = if m >= 64 {
        &[(-17, 17, 16), (-16, 9, 8), (-15, 9, 8), (-13, 5, 4)]
    } else {
        &[(-13, 5, 4)]
    };

    struct DiscreteCandidate {
        codes: Vec<i64>,
        deltas: Vec<BigRational>,
        guards: Vec<BigRational>,
        objective: BigRational,
    }

    let mut best_sol: Option<DiscreteCandidate> = None;

    for &(pad_exp, scale_num, scale_den) in schedules {
        let bump_pad = Dy::<P>::ONE.ldexp(pad_exp);
        let bump_scale = Dy::<P>::from_rational(&BigRational::new(
            BigInt::from(scale_num),
            BigInt::from(scale_den),
        ));
        let mut s_basis = basis.clone();
        let mut s_st = st.clone();
        let mut s_x = x.clone();
        let mut s_binv = binv.clone();
        let mut s_pi = pi.clone();
        let mut b_rhs = vec![Dy::<P>::ZERO; m];

        for iter in 0..60 {
            if iter > 0 {
                let mut iter_stats = SearchStats::default();
                if dual_simplex_run::<P>(
                    n,
                    m,
                    &cols,
                    &cost,
                    &lo,
                    &up,
                    &b_rhs,
                    &mut s_basis,
                    &mut s_st,
                    &mut s_x,
                    &mut s_binv,
                    &mut s_pi,
                    100_000,
                    &mut iter_stats,
                    None,
                )
                .is_err()
                {
                    break;
                }
                total_pivots += iter_stats.pivots;
            }

            let mut cur_q = vec![0i64; n];
            let mut disc_rat = Vec::with_capacity(n);
            let mut disc_dy = vec![Dy::<P>::ZERO; n];
            for i in 0..n {
                let xi = s_x[i];
                let best = legal_deltas[i]
                    .iter()
                    .min_by(|a, b| (a.2 - xi).abs().cmp(&(b.2 - xi).abs()))
                    .expect("15 legal deltas");
                cur_q[i] = best.0;
                disc_rat.push(best.1.clone());
                disc_dy[i] = best.2;
            }

            let mut n_viol = 0usize;
            let mut guard_dy = vec![Dy::<P>::ZERO; m];
            for r in 0..m {
                let mut s = Dy::<P>::ZERO;
                for i in 0..n {
                    if !disc_dy[i].is_zero() && !j_dy[r][i].is_zero() {
                        s = s + j_dy[r][i] * disc_dy[i];
                    }
                }
                guard_dy[r] = s;
                if s.is_negative() {
                    n_viol += 1;
                    let bump = (-s) * bump_scale + bump_pad;
                    b_rhs[r] = b_rhs[r] - bump;
                }
            }

            if n_viol == 0 {
                let mut exact_guards = vec![BigRational::zero(); m];
                for (r, eg) in exact_guards.iter_mut().enumerate() {
                    let mut s = BigRational::zero();
                    for (i, dr) in disc_rat.iter().enumerate() {
                        if !dr.is_zero() {
                            s += &p.guards[r][i] * dr;
                        }
                    }
                    *eg = s;
                }
                if exact_guards.iter().any(|g| g.is_negative()) {
                    for (r, eg) in exact_guards.iter().enumerate() {
                        if eg.is_negative() {
                            let s_dy = dy(eg);
                            let bump = (-s_dy) * bump_scale + bump_pad;
                            b_rhs[r] = b_rhs[r] - bump;
                        }
                    }
                    continue;
                }
                let mut exact_obj = BigRational::zero();
                for (i, dr) in disc_rat.iter().enumerate() {
                    if !dr.is_zero() {
                        exact_obj += &p.g[i] * dr;
                    }
                }
                run_polish(
                    &mut cur_q,
                    &mut disc_rat,
                    &mut disc_dy,
                    &mut exact_guards,
                    &mut guard_dy,
                    &mut exact_obj,
                );
                if exact_obj.is_positive() {
                    for (i, c) in cs.iter().enumerate() {
                        cur_q[i] = c.q0;
                        disc_rat[i] = BigRational::zero();
                        disc_dy[i] = Dy::ZERO;
                    }
                    exact_guards.fill(BigRational::zero());
                    guard_dy.fill(Dy::ZERO);
                    exact_obj = BigRational::zero();
                    run_polish(
                        &mut cur_q,
                        &mut disc_rat,
                        &mut disc_dy,
                        &mut exact_guards,
                        &mut guard_dy,
                        &mut exact_obj,
                    );
                }
                if best_sol
                    .as_ref()
                    .is_none_or(|cand| exact_obj < cand.objective)
                {
                    best_sol = Some(DiscreteCandidate {
                        codes: cur_q,
                        deltas: disc_rat,
                        guards: exact_guards,
                        objective: exact_obj,
                    });
                }
                break;
            }
        }
    }

    let Some(DiscreteCandidate {
        codes: cur_q,
        deltas: disc_rat,
        guards: exact_guards,
        objective: exact_obj,
    }) = best_sol
    else {
        return Err(
            "LP-guided margin loop did not reach a guard-feasible discrete assignment".into(),
        );
    };

    // Independent from-scratch recomputation over BigRational.
    let mut recomputed_guards = vec![BigRational::zero(); m];
    for (r, row) in p.guards.iter().enumerate() {
        let mut s = BigRational::zero();
        for (i, jv) in row.iter().enumerate() {
            if !disc_rat[i].is_zero() && !jv.is_zero() {
                s += jv * &disc_rat[i];
            }
        }
        if s.is_negative() {
            return Err(format!("independent guard check failed on row {r}"));
        }
        recomputed_guards[r] = s;
    }
    let full_guard_vector_matches = recomputed_guards == exact_guards;
    if !full_guard_vector_matches {
        return Err("independent guard vector differs from incremental".into());
    }
    let recomputed_obj =
        p.g.iter()
            .zip(&disc_rat)
            .fold(BigRational::zero(), |s, (gi, di)| s + gi * di);
    if recomputed_obj != exact_obj {
        return Err("independent objective differs from incremental".into());
    }
    let active_guards = recomputed_guards.iter().filter(|g| g.is_zero()).count();
    let guard_min = recomputed_guards
        .into_iter()
        .min()
        .unwrap_or_else(BigRational::zero);
    let moved_coeffs = disc_rat.iter().filter(|d| !d.is_zero()).count();

    Ok(DiscreteLpResult {
        delta_lp_basis: basis_iter0,
        dual_y: dual_y_iter0,
        codes: cur_q,
        deltas: disc_rat,
        objective: recomputed_obj,
        guard_min,
        active_guards,
        moved_coeffs,
        full_guard_vector_matches,
        total_pivots,
    })
}
