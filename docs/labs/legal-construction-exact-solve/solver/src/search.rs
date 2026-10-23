//! Bounded-precision basis search (#2828).
//!
//! The search only proposes a basis; `blp::certify_basis` proves or rejects
//! it exactly against the input LP. Nothing the search computes is trusted.
//!
//! * Data. `blp`'s integer-scaled rows and costs, converted to `Dy<P>`.
//! * Method. Bounded-variable revised simplex with `blp`'s conventions:
//!   slacks start basic, structurals nonbasic at Lower, Upper or Interior
//!   (start value 0 when lower < 0 < upper), bound flips, Dantzig pricing
//!   with Bland's rule after 50 consecutive degenerate steps.
//! * Phase 1. Rows the start point violates get an artificial with column
//!   `−e_i`, as in `blp`. After phase 1 every basic artificial is swapped
//!   for the non-artificial column with the largest entry in its basis row.
//! * Basis inverse. Dense `B⁻¹` in `Dy<P>`, rank-one (product-form) updates,
//!   refactored by Gauss-Jordan elimination every 50 pivots.
//! * Tolerance, for pivot choice only: a value below `2^-s` times the
//!   magnitude it is compared with counts as zero, `s = max(P − 30, P / 2)`.

use num_rational::BigRational;
use num_traits::Zero;

use crate::blp::{self, Basis, Pos, Scaled};
use crate::dyadic::Dy;
use crate::lp::{Lp, LpResult};

const REPORT_EVERY: u64 = 25;
const DEGENERATE_LIMIT: u32 = 50;
const REFACTOR_EVERY: u64 = 50;

/// Progress counters. `objective_bits` is `⌈log2 |objective|⌉` roughly (the
/// position above the top set bit), 0 for a zero objective.
#[derive(Debug, Clone, Default)]
pub struct SearchStats {
    pub pivots: u64,
    pub flips: u64,
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
                *v = *v / piv;
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
            let (theta, to_upper) = if rate.is_negative() {
                ((self.x[bv] - self.lower[bv]) / (-rate), false)
            } else {
                match self.upper[bv] {
                    Some(u) => ((u - self.x[bv]) / rate, true),
                    None => continue,
                }
            };
            let theta = theta.max(Dy::ZERO);
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
            degenerate = if theta.is_zero() { degenerate + 1 } else { 0 };
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

/// Run the search; `progress` is called every 25 pivots.
pub fn search<const P: u32>(
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
