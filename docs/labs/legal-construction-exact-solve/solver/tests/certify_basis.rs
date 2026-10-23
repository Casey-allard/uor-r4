//! `blp::certify_basis` proves `blp`'s own final basis optimal on every
//! control and random LP (root and bound-tightened nodes), and rejects
//! planted bad bases. The generators are copied from `blp_agrees.rs`.

use legal_exact_solver::blp::{self, Basis, Pos};
use legal_exact_solver::lp::{Lp, LpResult};
use legal_exact_solver::{bb, problem::Problem};
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

/// Root relaxation plus the node LPs of `blp_agrees.rs`.
fn lps(p: &Problem) -> Vec<Lp> {
    let root = bb::relaxation(p);
    let mut out = vec![root.clone()];
    for v in 0..p.n() {
        let mut up = root.clone();
        up.lower[v] = q(1, 1).max(up.lower[v].clone());
        out.push(up);
        let mut down = root.clone();
        down.upper[v] = q(-1, 1).min(down.upper[v].clone());
        out.push(down);
    }
    let mut all = root.clone();
    for v in 0..p.n() {
        all.lower[v] = q(1, 1).max(all.lower[v].clone());
    }
    out.push(all);
    out
}

/// Certify blp's basis; then swap each nonbasic index into basis row 0 and
/// require an `Err` or the same optimum. Returns (certified, swap errors).
fn check(lp: &Lp, what: &str) -> (usize, usize) {
    let (got, basis) = blp::solve_with_basis(lp);
    let LpResult::Optimal {
        objective: want, ..
    } = got
    else {
        assert!(basis.is_none(), "{what}: basis without an optimum");
        return (0, 0);
    };
    let basis = basis.expect("optimal result carries a basis");
    match blp::certify_basis(lp, &basis) {
        Ok(LpResult::Optimal { objective, .. }) => assert_eq!(objective, want, "{what}"),
        other => panic!("{what}: blp's own basis not certified: {other:?}"),
    }
    let mut errs = 0;
    let total = basis.nonbasic_pos.len();
    for j in (0..total).filter(|j| !basis.basic.contains(j)) {
        if basis.basic.is_empty() {
            break;
        }
        let mut bad = basis.clone();
        let out = bad.basic[0];
        bad.basic[0] = j;
        bad.nonbasic_pos[out] = Pos::Lower;
        match blp::certify_basis(lp, &bad) {
            Err(_) => errs += 1,
            Ok(LpResult::Optimal { objective, .. }) => {
                assert_eq!(
                    objective, want,
                    "{what}: swap {out}->{j} gave a wrong optimum"
                )
            }
            Ok(other) => panic!("{what}: unexpected {other:?}"),
        }
    }
    (1, errs)
}

#[test]
fn certifies_blp_basis_on_controls_and_random() {
    let (mut ok, mut errs) = (0, 0);
    let mut problems = controls();
    problems.extend((0..240).map(instance));
    for (i, p) in problems.iter().enumerate() {
        for (k, lp) in lps(p).iter().enumerate() {
            let (a, e) = check(lp, &format!("problem {i} lp {k}"));
            ok += a;
            errs += e;
        }
    }
    eprintln!("certified {ok} bases; {errs} swapped bases rejected");
    assert!(ok > 500 && errs > 500, "certified {ok}, rejected {errs}");
}

fn tiny() -> Lp {
    // minimise -x subject to x <= 1, 0 <= x <= 2: optimum x = 1.
    Lp {
        c: vec![q(-1, 1)],
        rows: vec![(vec![q(1, 1)], q(1, 1))],
        lower: vec![q(0, 1)],
        upper: vec![q(2, 1)],
    }
}

#[test]
fn planted_wrong_sign_is_rejected() {
    let lp = tiny();
    let slack = Basis {
        basic: vec![1],
        nonbasic_pos: vec![Pos::Lower, Pos::Lower],
    };
    let err = blp::certify_basis(&lp, &slack).expect_err("x at Lower is not optimal");
    assert!(err.contains("wrong sign"), "{err}");
    let right = Basis {
        basic: vec![0],
        nonbasic_pos: vec![Pos::Lower, Pos::Lower],
    };
    match blp::certify_basis(&lp, &right) {
        Ok(LpResult::Optimal { objective, .. }) => assert_eq!(objective, q(-1, 1)),
        other => panic!("optimal basis not certified: {other:?}"),
    }
}

#[test]
fn bad_bases_are_errors_not_panics() {
    // x + y <= 1 and 2x + 2y <= 2 scale to the same row: {x, y} is singular.
    let lp = Lp {
        c: vec![q(-1, 1), q(-1, 1)],
        rows: vec![
            (vec![q(1, 1), q(1, 1)], q(1, 1)),
            (vec![q(2, 1), q(2, 1)], q(2, 1)),
        ],
        lower: vec![q(0, 1), q(0, 1)],
        upper: vec![q(1, 1), q(1, 1)],
    };
    let pos = vec![Pos::Lower; 4];
    let singular = Basis {
        basic: vec![0, 1],
        nonbasic_pos: pos.clone(),
    };
    assert_eq!(
        blp::certify_basis(&lp, &singular).expect_err("singular"),
        "singular basis"
    );
    for basic in [vec![0, 0], vec![0, 9], vec![0]] {
        let b = Basis {
            basic,
            nonbasic_pos: pos.clone(),
        };
        assert!(blp::certify_basis(&lp, &b).is_err(), "{b:?}");
    }
    let upper_slack = Basis {
        basic: vec![0, 1],
        nonbasic_pos: vec![Pos::Lower, Pos::Lower, Pos::Upper, Pos::Lower],
    };
    assert!(blp::certify_basis(&lp, &upper_slack).is_err());
}
