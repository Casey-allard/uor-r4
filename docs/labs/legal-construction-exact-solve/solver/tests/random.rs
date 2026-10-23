//! Seeded random dyadic instances against the brute-force oracle.

use std::time::Instant;

use legal_exact_solver::bb::{self, Status};
use legal_exact_solver::{brute, check, lp, problem::Problem};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

const SEEDS: u64 = 240;

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

/// N <= 5, R <= 4, dyadic masters (half on the quarter grid, half on a
/// sixteenth grid so many are fractional), guards planted so that a random
/// legal move keeps every one of them.
fn instance(seed: u64, n_fixed: Option<usize>) -> Problem {
    let mut rng = Rng(seed);
    let n = n_fixed.unwrap_or_else(|| rng.range(1, 5) as usize);
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
    // A random legal move δ*.
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

#[test]
fn random_dyadic_agree_with_brute_force() {
    let start = Instant::now();
    let (mut improving, mut frac_moves, mut with_guards, mut total_nodes) = (0, 0, 0, 0);
    for seed in 0..SEEDS {
        let p = instance(seed, None);
        let o = bb::solve(&p, 100_000);
        assert_eq!(o.status, Status::Optimal, "seed {seed}");
        let b = brute::solve(&p);
        assert_eq!(
            o.objective, b.objective,
            "seed {seed}: solver vs brute force"
        );
        let c = check::check(&p, &o.assignment);
        assert!(
            c.legal,
            "seed {seed}: illegal assignment {:?}",
            o.assignment
        );
        assert!(
            c.guards_hold,
            "seed {seed}: guard broken {:?}",
            c.guard_values
        );
        assert_eq!(c.objective, o.objective, "seed {seed}: check objective");
        let cb = check::check(&p, &b.assignment);
        assert!(
            cb.legal && cb.guards_hold && cb.objective == b.objective,
            "seed {seed}: oracle"
        );
        improving += usize::from(o.objective.is_negative());
        frac_moves += usize::from(o.assignment.l.iter().chain(&o.assignment.r).any(|&x| x));
        with_guards += usize::from(!p.guards.is_empty() && o.objective.is_negative());
        total_nodes += o.nodes;
    }
    eprintln!(
        "random suite: {SEEDS} seeds in {:?}; {improving} improving, {frac_moves} with a fractional move, {with_guards} improving under guards, {total_nodes} B&B nodes",
        start.elapsed()
    );
    // Non-vacuity: the controls must exercise real moves, fractional ones and guards.
    assert!(improving > SEEDS as usize / 2);
    assert!(frac_moves > 20);
    assert!(with_guards > 50);
}

#[test]
fn timing_one_n5_lp_solve() {
    let p = instance(7, Some(5));
    let root = bb::relaxation(&p);
    let start = Instant::now();
    let r = lp::solve(&root);
    let lp_time = start.elapsed();
    assert!(matches!(r, lp::LpResult::Optimal { .. }));
    let start = Instant::now();
    let o = bb::solve(&p, 100_000);
    eprintln!(
        "N=5 R={}: one root LP solve {:?} ({} vars, {} rows); full B&B {:?} over {} nodes",
        p.guards.len(),
        lp_time,
        root.c.len(),
        root.rows.len(),
        start.elapsed(),
        o.nodes
    );
}
