//! Hand-solved instances, edge cases and the exact input conversion.

use legal_exact_solver::bb::{self, Status};
use legal_exact_solver::exact::{f32_to_rational, from_f32, problem_from_f32_bits, ExactError};
use legal_exact_solver::{brute, check, problem::Problem};
use num_bigint::BigInt;
use num_rational::BigRational;

fn q(n: i64, d: i64) -> BigRational {
    BigRational::new(BigInt::from(n), BigInt::from(d))
}

fn rv(v: &[(i64, i64)]) -> Vec<BigRational> {
    v.iter().map(|&(n, d)| q(n, d)).collect()
}

fn ks(o: &bb::Outcome) -> Vec<i64> {
    o.assignment
        .k
        .iter()
        .map(|k| i64::try_from(k).unwrap())
        .collect()
}

/// Solve, require a proved optimum that passes the independent check and
/// agrees with brute force.
fn solve_checked(p: &Problem) -> bb::Outcome {
    let o = bb::solve(p, 4096);
    assert_eq!(o.status, Status::Optimal);
    let c = check::check(p, &o.assignment);
    assert!(
        c.legal,
        "solver assignment is not legal: {:?}",
        o.assignment
    );
    assert!(
        c.guards_hold,
        "solver assignment breaks a guard: {:?}",
        c.guard_values
    );
    assert_eq!(c.objective, o.objective);
    assert_eq!(brute::solve(p).objective, o.objective);
    o
}

/// Hand-solved 1. m = (0, 1/4): both on grid, q0 = (0, 1), so
/// k0 in [-7, 7], k1 in [-8, 6], δ = k/4. g = (1, -2), guard δ0 - δ1 >= 0,
/// i.e. k0 >= k1. Objective (k0 - 2 k1)/4 >= (k1 - 2 k1)/4 = -k1/4 >= -6/4,
/// with equality only at k1 = 6, k0 = 6. Optimum -3/2 at k = (6, 6).
#[test]
fn hand_two_on_grid() {
    let p = Problem::new(
        rv(&[(0, 1), (1, 4)]),
        rv(&[(1, 1), (-2, 1)]),
        vec![rv(&[(1, 1), (-1, 1)])],
    );
    let o = solve_checked(&p);
    assert_eq!(o.objective, q(-3, 2));
    assert_eq!(ks(&o), vec![6, 6]);
}

/// Hand-solved 2. m0 = 3/8: 4m = 3/2, q0 = 2 (ties away), shift = 2/4 - 3/8 = 1/8.
/// δ0 is 0 (keep), k/4 + 1/8 for k in [-9, -1] (lower: -17/8 .. -1/8) or for
/// k in [1, 5] (higher: 3/8 .. 11/8). m1 = 0 on grid, δ1 = k1/4, k1 in [-7, 7].
/// g = (1, 1), guard δ0 + 2 δ1 >= 0. For δ0 > 0 the objective is >= δ0/2 > 0.
/// For lower moves, δ1 = smallest quarter >= -δ0/2:
///   δ0 = -17/8 -> δ1 = 5/4, sum -7/8;  δ0 = -15/8 -> δ1 = 1, sum -7/8;
///   δ0 = -13/8 -> δ1 = 1, sum -5/8;    ... all others are larger.
/// Optimum -7/8 (two optimal points; either is accepted).
#[test]
fn hand_fractional_lower() {
    let p = Problem::new(
        rv(&[(3, 8), (0, 1)]),
        rv(&[(1, 1), (1, 1)]),
        vec![rv(&[(1, 1), (2, 1)])],
    );
    let o = solve_checked(&p);
    assert_eq!(o.objective, q(-7, 8));
    assert!(o.assignment.l[0] && !o.assignment.r[0]);
    assert!(
        ks(&o) == vec![-9, 5] || ks(&o) == vec![-8, 4],
        "{:?}",
        ks(&o)
    );
}

/// Hand-solved 3. m = 5/16: 4m = 5/4, q0 = 1, shift = 1/4 - 5/16 = -1/16.
/// No guards, g = (-1): maximise δ. Higher moves k in [1, 6] give
/// δ = k/4 - 1/16, best k = 6 (q = 7): δ = 23/16, objective -23/16.
#[test]
fn hand_fractional_higher_to_endpoint() {
    let p = Problem::new(rv(&[(5, 16)]), rv(&[(-1, 1)]), vec![]);
    let o = solve_checked(&p);
    assert_eq!(o.objective, q(-23, 16));
    assert_eq!(ks(&o), vec![6]);
    assert!(o.assignment.r[0] && !o.assignment.l[0]);
}

/// Legal endpoints. m = 7/4 (q0 = 7) with g = 1 goes to q = -7 (k = -14);
/// m = -7/4 (q0 = -7) with g = -1 goes to q = 7 (k = 14). Objective -7/2 each.
#[test]
fn endpoints_q_pm7() {
    let p = Problem::new(rv(&[(7, 4), (-7, 4)]), rv(&[(1, 1), (-1, 1)]), vec![]);
    let o = solve_checked(&p);
    assert_eq!(ks(&o), vec![-14, 14]);
    assert_eq!(o.objective, q(-7, 1));
    // At the top endpoint the only move is down: g = -1 there leaves it at 7.
    let p = Problem::new(rv(&[(7, 4)]), rv(&[(-1, 1)]), vec![]);
    let o = solve_checked(&p);
    assert_eq!(ks(&o), vec![0]);
    assert_eq!(o.objective, q(0, 1));
}

/// Noop branch: the fractional coefficient 0 is pinned to δ0 = 0 by the
/// guards ±e0, so it must take (l, r) = (0, 0) with k = 0, while the free
/// on-grid coefficient 1 moves to q = -7.
#[test]
fn noop_branch() {
    let p = Problem::new(
        rv(&[(5, 16), (0, 1)]),
        rv(&[(1, 1), (1, 1)]),
        vec![rv(&[(1, 1), (0, 1)]), rv(&[(-1, 1), (0, 1)])],
    );
    let o = solve_checked(&p);
    assert!(!o.assignment.l[0] && !o.assignment.r[0]);
    assert_eq!(ks(&o), vec![0, -7]);
    assert_eq!(o.objective, q(-7, 4));
}

/// All-zero guard rows hold trivially and constrain nothing.
#[test]
fn all_zero_guard_rows() {
    let p = Problem::new(
        rv(&[(0, 1), (0, 1)]),
        rv(&[(1, 1), (-1, 1)]),
        vec![rv(&[(0, 1), (0, 1)]), rv(&[(0, 1), (0, 1)])],
    );
    let o = solve_checked(&p);
    assert_eq!(ks(&o), vec![-7, 7]);
    assert_eq!(o.objective, q(-7, 2));
}

/// Guards ±e_i forbid every non-zero move: the answer is δ = 0, objective 0.
#[test]
fn guard_forbids_every_move() {
    let m = rv(&[(3, 8), (1, 4), (-5, 16)]);
    let g = rv(&[(1, 1), (-1, 1), (2, 1)]);
    let mut guards = Vec::new();
    for i in 0..3 {
        for s in [1, -1] {
            let mut row = rv(&[(0, 1), (0, 1), (0, 1)]);
            row[i] = q(s, 1);
            guards.push(row);
        }
    }
    let p = Problem::new(m, g, guards);
    let o = solve_checked(&p);
    assert_eq!(o.objective, q(0, 1));
    assert_eq!(o.assignment, bb::Assignment::zero(3));
    assert!(check::check(&p, &o.assignment)
        .delta
        .iter()
        .all(|d| *d == q(0, 1)));
}

/// A guard equal to g forbids every strictly improving move.
#[test]
fn guard_equal_to_objective() {
    let g = rv(&[(1, 2), (-3, 4), (1, 1)]);
    let p = Problem::new(rv(&[(3, 8), (0, 1), (-5, 16)]), g.clone(), vec![g]);
    let o = solve_checked(&p);
    assert_eq!(o.objective, q(0, 1));
}

#[test]
fn node_cap_is_reported() {
    let p = Problem::new(
        rv(&[(3, 8), (0, 1)]),
        rv(&[(1, 1), (1, 1)]),
        vec![rv(&[(1, 1), (2, 1)])],
    );
    let o = bb::solve(&p, 1);
    assert_eq!(o.status, Status::CapReached);
    assert_eq!(o.nodes, 1);
    assert!(o.best_bound <= o.objective);
}

#[test]
fn f32_conversion_is_bit_exact() {
    // Smallest subnormal: 2^-149.
    let tiny = f32_to_rational(1).unwrap();
    assert_eq!(
        tiny,
        BigRational::new(BigInt::from(1), BigInt::from(1) << 149usize)
    );
    assert_eq!(from_f32(f32::from_bits(1)).unwrap(), tiny);
    // Largest subnormal: (2^23 - 1) · 2^-149.
    assert_eq!(
        f32_to_rational(0x007f_ffff).unwrap(),
        BigRational::new(BigInt::from((1u32 << 23) - 1), BigInt::from(1) << 149usize)
    );
    // Negative values.
    assert_eq!(from_f32(-1.5).unwrap(), q(-3, 2));
    assert_eq!(from_f32(-0.3125).unwrap(), q(-5, 16));
    assert_eq!(from_f32(-0.0).unwrap(), q(0, 1));
    // 0.1f32 is exactly 13421773 / 2^27.
    assert_eq!(
        from_f32(0.1).unwrap(),
        BigRational::new(BigInt::from(13_421_773), BigInt::from(1) << 27usize)
    );
    // Large normal: 2^127 · (2 - 2^-23).
    assert_eq!(
        from_f32(f32::MAX).unwrap(),
        BigRational::from_integer(BigInt::from((1u32 << 24) - 1) << 104usize)
    );
}

#[test]
fn f32_nan_and_infinity_refused() {
    for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            from_f32(x),
            Err(ExactError::NotFinite { bits: x.to_bits() })
        );
    }
    assert!(f32_to_rational(0x7fc0_0001).is_err());
    let bad = problem_from_f32_bits(&[0.25f32.to_bits()], &[f32::NAN.to_bits()], &[]);
    assert!(bad.is_err());
    let ok = problem_from_f32_bits(
        &[0.25f32.to_bits()],
        &[(-1.0f32).to_bits()],
        &[vec![0.5f32.to_bits()]],
    )
    .unwrap();
    assert_eq!(ok.masters, vec![q(1, 4)]);
    assert_eq!(ok.guards, vec![vec![q(1, 2)]]);
}
