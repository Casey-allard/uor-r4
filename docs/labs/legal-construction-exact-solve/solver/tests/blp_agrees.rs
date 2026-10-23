//! `blp::solve` against the tableau reference `lp::solve` on the root
//! relaxation of every control and random instance, and on bound-tightened
//! node LPs like branch-and-bound's (some infeasible). The generators are
//! copied from `controls.rs` and `random.rs` (test files are separate crates).

use legal_exact_solver::lp::{self, Lp, LpResult};
use legal_exact_solver::{bb, blp, problem::Problem};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

fn q(n: i64, d: i64) -> BigRational {
    BigRational::new(BigInt::from(n), BigInt::from(d))
}

fn rv(v: &[(i64, i64)]) -> Vec<BigRational> {
    v.iter().map(|&(n, d)| q(n, d)).collect()
}

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

/// Same generator as `random.rs`.
fn instance(seed: u64) -> Problem {
    let mut rng = Rng(seed);
    let n = rng.range(1, 5) as usize;
    let rows = rng.range(0, 4) as usize;
    let masters: Vec<BigRational> = (0..n)
        .map(|_| {
            if rng.range(0, 1) == 0 {
                q(rng.range(-7, 7), 4)
            } else {
                q(rng.range(-30, 30), 16)
            }
        })
        .collect();
    let g: Vec<BigRational> = (0..n).map(|_| q(rng.range(-8, 8), 4)).collect();
    let probe = Problem::new(masters.clone(), g.clone(), vec![]);
    let planted: Vec<BigRational> = probe
        .coeffs()
        .iter()
        .map(|c| {
            let (lo, hi) = (-7 - c.q0, 7 - c.q0);
            if !c.fractional {
                return q(rng.range(lo, hi), 4);
            }
            match rng.range(0, 2) {
                1 if lo <= -1 => q(rng.range(lo, -1), 4) + &c.shift,
                2 if hi >= 1 => q(rng.range(1, hi), 4) + &c.shift,
                _ => BigRational::zero(),
            }
        })
        .collect();
    let guards = (0..rows)
        .map(|_| {
            let row: Vec<BigRational> = (0..n).map(|_| q(rng.range(-4, 4), 2)).collect();
            let v = row
                .iter()
                .zip(&planted)
                .fold(BigRational::zero(), |s, (a, b)| s + a * b);
            if v.is_negative() {
                row.into_iter().map(|a| -a).collect()
            } else {
                row
            }
        })
        .collect();
    Problem::new(masters, g, guards)
}

/// The instances of `controls.rs`.
fn controls() -> Vec<Problem> {
    let mut forbid = Vec::new();
    for i in 0..3 {
        for s in [1, -1] {
            let mut row = rv(&[(0, 1), (0, 1), (0, 1)]);
            row[i] = q(s, 1);
            forbid.push(row);
        }
    }
    let geq = rv(&[(1, 2), (-3, 4), (1, 1)]);
    vec![
        Problem::new(
            rv(&[(0, 1), (1, 4)]),
            rv(&[(1, 1), (-2, 1)]),
            vec![rv(&[(1, 1), (-1, 1)])],
        ),
        Problem::new(
            rv(&[(3, 8), (0, 1)]),
            rv(&[(1, 1), (1, 1)]),
            vec![rv(&[(1, 1), (2, 1)])],
        ),
        Problem::new(rv(&[(5, 16)]), rv(&[(-1, 1)]), vec![]),
        Problem::new(rv(&[(7, 4), (-7, 4)]), rv(&[(1, 1), (-1, 1)]), vec![]),
        Problem::new(rv(&[(7, 4)]), rv(&[(-1, 1)]), vec![]),
        Problem::new(
            rv(&[(5, 16), (0, 1)]),
            rv(&[(1, 1), (1, 1)]),
            vec![rv(&[(1, 1), (0, 1)]), rv(&[(-1, 1), (0, 1)])],
        ),
        Problem::new(
            rv(&[(0, 1), (0, 1)]),
            rv(&[(1, 1), (-1, 1)]),
            vec![rv(&[(0, 1), (0, 1)]), rv(&[(0, 1), (0, 1)])],
        ),
        Problem::new(
            rv(&[(3, 8), (1, 4), (-5, 16)]),
            rv(&[(1, 1), (-1, 1), (2, 1)]),
            forbid,
        ),
        Problem::new(rv(&[(3, 8), (0, 1), (-5, 16)]), geq.clone(), vec![geq]),
    ]
}

#[derive(Default)]
struct Tally {
    optimal: usize,
    infeasible: usize,
    phase1: usize,
    flips: u64,
}

/// Same variant and objective as the reference; blp's x feasible.
fn agree(lp: &Lp, what: &str, t: &mut Tally) {
    let (got, stats) = blp::solve_with_stats(lp, None);
    t.flips += stats.flips;
    t.phase1 += usize::from(stats.phase1_pivots > 0);
    match (lp::solve(lp), got) {
        (LpResult::Infeasible, LpResult::Infeasible) => t.infeasible += 1,
        (
            LpResult::Optimal {
                objective: want, ..
            },
            LpResult::Optimal { x, objective },
        ) => {
            assert_eq!(objective, want, "{what}: objective");
            for (k, (a, b)) in lp.rows.iter().enumerate() {
                let v = a
                    .iter()
                    .zip(&x)
                    .fold(BigRational::zero(), |s, (ai, xi)| s + ai * xi);
                assert!(v <= *b, "{what}: row {k} violated");
            }
            for (j, v) in x.iter().enumerate() {
                assert!(
                    *v >= lp.lower[j] && *v <= lp.upper[j],
                    "{what}: bound {j} violated"
                );
            }
            let obj =
                lp.c.iter()
                    .zip(&x)
                    .fold(BigRational::zero(), |s, (c, v)| s + c * v);
            assert_eq!(obj, objective, "{what}: objective is not c·x");
            t.optimal += 1;
        }
        (want, got) => panic!("{what}: variant differs: reference {want:?}, blp {got:?}"),
    }
}

/// Node LPs: each k forced to <= -1 or >= 1 (away from 0), one at a time
/// and all together, plus an empty box.
fn tightened(p: &Problem, what: &str, t: &mut Tally) {
    let root = bb::relaxation(p);
    for v in 0..p.n() {
        for up in [false, true] {
            let mut lp = root.clone();
            if up {
                lp.lower[v] = q(1, 1).max(lp.lower[v].clone());
            } else {
                lp.upper[v] = q(-1, 1).min(lp.upper[v].clone());
            }
            agree(
                &lp,
                &format!("{what} k{v} {}", if up { ">= 1" } else { "<= -1" }),
                t,
            );
        }
    }
    let mut all = root.clone();
    for v in 0..p.n() {
        all.lower[v] = q(1, 1).max(all.lower[v].clone());
    }
    agree(&all, &format!("{what} all k >= 1"), t);
    let mut empty = root;
    empty.lower[0] = &empty.upper[0] + q(1, 1);
    agree(&empty, &format!("{what} empty box"), t);
}

#[test]
fn controls_agree_with_tableau() {
    let mut t = Tally::default();
    for (i, p) in controls().iter().enumerate() {
        agree(&bb::relaxation(p), &format!("control {i}"), &mut t);
        tightened(p, &format!("control {i}"), &mut t);
    }
    // The forbid-every-move control must give infeasible tightened nodes.
    assert!(
        t.infeasible > 9 && t.optimal > 9 && t.phase1 > 0,
        "optimal {} infeasible {}",
        t.optimal,
        t.infeasible
    );
}

#[test]
fn random_agree_with_tableau() {
    let mut t = Tally::default();
    for seed in 0..240 {
        let p = instance(seed);
        agree(&bb::relaxation(&p), &format!("seed {seed} root"), &mut t);
        tightened(&p, &format!("seed {seed}"), &mut t);
    }
    eprintln!(
        "random: {} optimal, {} infeasible, {} with phase 1, {} flips",
        t.optimal, t.infeasible, t.phase1, t.flips
    );
    assert!(t.optimal > 500 && t.infeasible > 240 + 50 && t.phase1 > 100 && t.flips > 100);
}

/// `blp` threads once the LP has at least 64 rows (`m >= 64` in `blp.rs`);
/// every other test here is smaller, so this is the one that runs the
/// threaded pricing and inverse update.
#[test]
fn agrees_with_threaded_path() {
    const THRESHOLD: usize = 64;
    for seed in 0..5u64 {
        let mut rng = Rng(0x7a11_0000 + seed);
        let n = 8;
        let masters: Vec<BigRational> = (0..n).map(|_| q(rng.range(-7, 7), 4)).collect();
        let g: Vec<BigRational> = (0..n).map(|_| q(rng.range(-8, 8), 4)).collect();
        let guards: Vec<Vec<BigRational>> = (0..72)
            .map(|_| (0..n).map(|_| q(rng.range(-4, 4), 2)).collect())
            .collect();
        let p = Problem::new(masters, g, guards);
        let lp = bb::relaxation(&p);
        assert!(
            lp.rows.len() >= THRESHOLD + 8,
            "seed {seed}: only {} rows",
            lp.rows.len()
        );
        let mut t = Tally::default();
        agree(&lp, &format!("threaded seed {seed}"), &mut t);
        assert_eq!(t.optimal + t.infeasible, 1);
    }
}
