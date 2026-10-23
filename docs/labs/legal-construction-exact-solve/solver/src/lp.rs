//! Exact two-phase tableau simplex with Bland's rule.
//!
//! Problem form: minimise `c·x` subject to `a·x <= b` for every row and
//! finite bounds `lower <= x <= upper`. Variables are shifted to
//! `y = x - lower >= 0`; each upper bound becomes a row `y <= upper - lower`.
//! Rows whose shifted right-hand side is negative get an artificial
//! variable, and phase 1 drives the artificials to zero. Bland's rule
//! (smallest entering index, ties in the ratio test broken by smallest
//! basic index) guarantees termination without cycling.

use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

#[derive(Debug, Clone)]
pub struct Lp {
    pub c: Vec<BigRational>,
    pub rows: Vec<(Vec<BigRational>, BigRational)>,
    pub lower: Vec<BigRational>,
    pub upper: Vec<BigRational>,
}

#[derive(Debug, Clone)]
pub enum LpResult {
    Infeasible,
    Optimal {
        x: Vec<BigRational>,
        objective: BigRational,
    },
}

struct Tableau {
    t: Vec<Vec<BigRational>>,
    rhs: Vec<BigRational>,
    basis: Vec<usize>,
    d: Vec<BigRational>,
}

impl Tableau {
    fn pivot(&mut self, r: usize, j: usize) {
        let p = self.t[r][j].clone();
        if !p.is_one() {
            for v in self.t[r].iter_mut().filter(|v| !v.is_zero()) {
                *v = &*v / &p;
            }
            self.rhs[r] = &self.rhs[r] / &p;
        }
        let prow = self.t[r].clone();
        let prhs = self.rhs[r].clone();
        for i in 0..self.t.len() {
            if i == r {
                continue;
            }
            let f = self.t[i][j].clone();
            if f.is_zero() {
                continue;
            }
            for (v, pv) in self.t[i].iter_mut().zip(&prow) {
                if !pv.is_zero() {
                    *v = &*v - &f * pv;
                }
            }
            self.rhs[i] = &self.rhs[i] - &f * &prhs;
        }
        let f = self.d[j].clone();
        if !f.is_zero() {
            for (v, pv) in self.d.iter_mut().zip(&prow) {
                if !pv.is_zero() {
                    *v = &*v - &f * pv;
                }
            }
        }
        self.basis[r] = j;
    }

    /// Reduced costs `d_j = c_j - sum_i c_B(i) t[i][j]`.
    fn price(&mut self, cost: &[BigRational]) {
        self.d = cost.to_vec();
        for (row, &b) in self.t.iter().zip(&self.basis) {
            let cb = &cost[b];
            if cb.is_zero() {
                continue;
            }
            for (dj, a) in self.d.iter_mut().zip(row) {
                if !a.is_zero() {
                    *dj = &*dj - cb * a;
                }
            }
        }
    }

    /// Run primal simplex over columns `0..allowed`. Returns false if unbounded.
    fn run(&mut self, allowed: usize) -> bool {
        loop {
            let Some(j) = (0..allowed).find(|&j| self.d[j].is_negative()) else {
                return true;
            };
            let mut best: Option<(usize, BigRational)> = None;
            for i in 0..self.t.len() {
                let a = &self.t[i][j];
                if !a.is_positive() {
                    continue;
                }
                let ratio = &self.rhs[i] / a;
                let better = match &best {
                    None => true,
                    Some((bi, br)) => {
                        ratio < *br || (ratio == *br && self.basis[i] < self.basis[*bi])
                    }
                };
                if better {
                    best = Some((i, ratio));
                }
            }
            let Some((r, _)) = best else {
                return false;
            };
            self.pivot(r, j);
        }
    }
}

fn dot(a: &[BigRational], x: &[BigRational]) -> BigRational {
    a.iter()
        .zip(x)
        .filter(|(ai, _)| !ai.is_zero())
        .fold(BigRational::zero(), |s, (ai, xi)| s + ai * xi)
}

pub fn solve(lp: &Lp) -> LpResult {
    let n = lp.c.len();
    let zero = BigRational::zero();
    let one = BigRational::one();

    let mut rows: Vec<(Vec<BigRational>, BigRational)> = Vec::with_capacity(lp.rows.len() + n);
    for (a, b) in &lp.rows {
        rows.push((a.clone(), b - dot(a, &lp.lower)));
    }
    for j in 0..n {
        let width = &lp.upper[j] - &lp.lower[j];
        if width.is_negative() {
            return LpResult::Infeasible;
        }
        let mut a = vec![zero.clone(); n];
        a[j] = one.clone();
        rows.push((a, width));
    }
    let m = rows.len();
    let n_art = rows.iter().filter(|(_, b)| b.is_negative()).count();
    let width = n + m + n_art;

    let mut tab = Tableau {
        t: Vec::with_capacity(m),
        rhs: Vec::with_capacity(m),
        basis: Vec::with_capacity(m),
        d: Vec::new(),
    };
    let mut art = 0;
    for (i, (a, b)) in rows.into_iter().enumerate() {
        let mut row = vec![zero.clone(); width];
        if b.is_negative() {
            for (dst, src) in row.iter_mut().zip(&a) {
                *dst = -src;
            }
            row[n + i] = -one.clone();
            row[n + m + art] = one.clone();
            tab.basis.push(n + m + art);
            tab.rhs.push(-b);
            art += 1;
        } else {
            row[..n].clone_from_slice(&a);
            row[n + i] = one.clone();
            tab.basis.push(n + i);
            tab.rhs.push(b);
        }
        tab.t.push(row);
    }

    if n_art > 0 {
        let mut cost1 = vec![zero.clone(); width];
        for c in cost1.iter_mut().skip(n + m) {
            *c = one.clone();
        }
        tab.price(&cost1);
        let bounded = tab.run(width);
        assert!(bounded, "phase 1 is bounded below by 0");
        let infeasible = tab
            .basis
            .iter()
            .zip(&tab.rhs)
            .any(|(&b, v)| b >= n + m && v.is_positive());
        if infeasible {
            return LpResult::Infeasible;
        }
        // Drive zero-level artificials out of the basis; drop redundant rows.
        let mut i = 0;
        while i < tab.t.len() {
            if tab.basis[i] >= n + m {
                if let Some(j) = (0..n + m).find(|&j| !tab.t[i][j].is_zero()) {
                    tab.pivot(i, j);
                    i += 1;
                } else {
                    tab.t.remove(i);
                    tab.rhs.remove(i);
                    tab.basis.remove(i);
                }
            } else {
                i += 1;
            }
        }
        for row in tab.t.iter_mut() {
            row.truncate(n + m);
        }
    }

    let mut cost2 = lp.c.clone();
    cost2.resize(n + m, zero.clone());
    tab.price(&cost2);
    let bounded = tab.run(n + m);
    assert!(bounded, "a box-bounded LP cannot be unbounded");

    let mut x = lp.lower.clone();
    for (&b, v) in tab.basis.iter().zip(&tab.rhs) {
        if b < n {
            x[b] = &x[b] + v;
        }
    }
    let objective = dot(&lp.c, &x);
    LpResult::Optimal { x, objective }
}
