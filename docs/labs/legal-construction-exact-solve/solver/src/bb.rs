//! Depth-first branch-and-bound over the integer codes `k` and the `l`/`r`
//! binaries, on the exact LP relaxation, from the incumbent `δ = 0`.
//!
//! Variable layout: `k_i` at index `i`; for the f-th fractional coefficient,
//! `l` at `N + 2f` and `r` at `N + 2f + 1`.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};

use crate::lp::{self, Lp, LpResult};
use crate::problem::Problem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub k: Vec<BigInt>,
    pub l: Vec<bool>,
    pub r: Vec<bool>,
}

impl Assignment {
    /// The noop move: every `k = 0`, every `l = r = 0`, so `δ = 0`.
    pub fn zero(n: usize) -> Self {
        Assignment {
            k: vec![BigInt::zero(); n],
            l: vec![false; n],
            r: vec![false; n],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The tree was closed: the objective is a proved optimum.
    Optimal,
    /// The node cap fired with open nodes; not an optimality claim.
    CapReached,
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub status: Status,
    pub assignment: Assignment,
    pub objective: BigRational,
    /// A proved lower bound on the optimum (equal to `objective` when Optimal).
    pub best_bound: BigRational,
    /// LP relaxations solved.
    pub nodes: usize,
}

fn int(v: i64) -> BigRational {
    BigRational::from_integer(BigInt::from(v))
}

/// Index of the `l` variable for each fractional coefficient (`r` is next).
fn binary_index(p: &Problem) -> (Vec<Option<usize>>, usize) {
    let mut next = p.n();
    let idx = p
        .coeffs()
        .iter()
        .map(|c| {
            if c.fractional {
                next += 2;
                Some(next - 2)
            } else {
                None
            }
        })
        .collect();
    (idx, next)
}

/// The root LP relaxation: `k` continuous in its legal bounds, `l, r` in [0, 1].
pub fn relaxation(p: &Problem) -> Lp {
    let n = p.n();
    let cs = p.coeffs();
    let (bin, nv) = binary_index(p);
    let quarter = BigRational::new(BigInt::from(1), BigInt::from(4));
    let zero = BigRational::zero();

    // δ_i as a sparse linear form over the variables.
    let delta: Vec<Vec<(usize, BigRational)>> = (0..n)
        .map(|i| {
            let mut t = vec![(i, quarter.clone())];
            if let Some(l) = bin[i] {
                t.push((l, cs[i].shift.clone()));
                t.push((l + 1, cs[i].shift.clone()));
            }
            t
        })
        .collect();

    let mut c = vec![zero.clone(); nv];
    for (gi, terms) in p.g.iter().zip(&delta) {
        for (v, coef) in terms {
            c[*v] = &c[*v] + gi * coef;
        }
    }

    let mut lower = vec![zero.clone(); nv];
    let mut upper = vec![BigRational::one(); nv];
    for (i, ci) in cs.iter().enumerate() {
        lower[i] = int(-7 - ci.q0);
        upper[i] = int(7 - ci.q0);
    }

    let mut rows = Vec::new();
    // Guards: J·δ >= 0, written as -J·δ <= 0.
    for row in &p.guards {
        let mut a = vec![zero.clone(); nv];
        for (jv, terms) in row.iter().zip(&delta) {
            for (v, coef) in terms {
                a[*v] = &a[*v] - jv * coef;
            }
        }
        rows.push((a, zero.clone()));
    }
    // Disjunction rows for each fractional coefficient.
    for (i, ci) in cs.iter().enumerate() {
        let Some(l) = bin[i] else { continue };
        let r = l + 1;
        // l + r <= 1
        let mut a = vec![zero.clone(); nv];
        a[l] = int(1);
        a[r] = int(1);
        rows.push((a, int(1)));
        // k >= r - (q0+7) l   ->   -k + r - (q0+7) l <= 0
        let mut a = vec![zero.clone(); nv];
        a[i] = int(-1);
        a[r] = int(1);
        a[l] = int(-(ci.q0 + 7));
        rows.push((a, zero.clone()));
        // k <= -l + (7-q0) r   ->   k + l - (7-q0) r <= 0
        let mut a = vec![zero.clone(); nv];
        a[i] = int(1);
        a[l] = int(1);
        a[r] = int(-(7 - ci.q0));
        rows.push((a, zero.clone()));
    }
    Lp {
        c,
        rows,
        lower,
        upper,
    }
}

struct Node {
    lower: Vec<BigRational>,
    upper: Vec<BigRational>,
    bound: Option<BigRational>,
}

pub fn solve(p: &Problem, node_cap: usize) -> Outcome {
    assert!(node_cap >= 1, "node cap must allow the root");
    let n = p.n();
    let (bin, nv) = binary_index(p);
    let mut lp = relaxation(p);

    let mut best_obj = BigRational::zero();
    let mut best = Assignment::zero(n);
    let mut stack = vec![Node {
        lower: lp.lower.clone(),
        upper: lp.upper.clone(),
        bound: None,
    }];
    let mut nodes = 0usize;

    while let Some(node) = stack.pop() {
        if node.bound.as_ref().is_some_and(|b| *b >= best_obj) {
            continue;
        }
        if nodes == node_cap {
            stack.push(node);
            break;
        }
        nodes += 1;
        lp.lower.clone_from(&node.lower);
        lp.upper.clone_from(&node.upper);
        let (x, obj) = match lp::solve(&lp) {
            LpResult::Infeasible => continue,
            LpResult::Optimal { x, objective } => (x, objective),
        };
        if obj >= best_obj {
            continue;
        }
        // Branch on a fractional binary first, then on a fractional k.
        match (n..nv).chain(0..n).find(|&v| !x[v].is_integer()) {
            None => {
                let mut a = Assignment::zero(n);
                for i in 0..n {
                    a.k[i] = x[i].to_integer();
                    if let Some(l) = bin[i] {
                        a.l[i] = x[l].is_one();
                        a.r[i] = x[l + 1].is_one();
                    }
                }
                best = a;
                best_obj = obj;
            }
            Some(v) => {
                let f = x[v].floor();
                let mut up_lower = node.lower.clone();
                up_lower[v] = &f + BigRational::one();
                stack.push(Node {
                    lower: up_lower,
                    upper: node.upper.clone(),
                    bound: Some(obj.clone()),
                });
                let mut down_upper = node.upper;
                down_upper[v] = f;
                stack.push(Node {
                    lower: node.lower,
                    upper: down_upper,
                    bound: Some(obj),
                });
            }
        }
    }

    stack.retain(|nd| nd.bound.as_ref().is_none_or(|b| *b < best_obj));
    if stack.is_empty() {
        return Outcome {
            status: Status::Optimal,
            assignment: best,
            best_bound: best_obj.clone(),
            objective: best_obj,
            nodes,
        };
    }
    let best_bound = stack
        .iter()
        .filter_map(|nd| nd.bound.clone())
        .fold(best_obj.clone(), |acc, b| if b < acc { b } else { acc });
    Outcome {
        status: Status::CapReached,
        assignment: best,
        objective: best_obj,
        best_bound,
        nodes,
    }
}
