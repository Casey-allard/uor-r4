//! `search::solve_search_certify` on every control and random LP: the
//! certificate, not the bounded-precision search, decides. At `P = 120`
//! every optimal LP is certified with `blp`'s exact objective; infeasible
//! LPs are errors; at `P = 12` an `Optimal` must still be exact.
//! The generators are copied from `certify_basis.rs`.

use legal_exact_solver::lp::{Lp, LpResult};
use legal_exact_solver::{bb, problem::Problem};
use legal_exact_solver::{blp, search};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

const MAX_PIVOTS: usize = 10_000;

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

fn suite() -> Vec<(String, Lp)> {
    let mut problems = controls();
    problems.extend((0..240).map(instance));
    let mut out = Vec::new();
    for (i, p) in problems.iter().enumerate() {
        for (k, lp) in lps(p).into_iter().enumerate() {
            out.push((format!("problem {i} lp {k}"), lp));
        }
    }
    out
}

/// (optimal LPs, certified, infeasible LPs). Panics on any wrong optimum.
fn run<const P: u32>() -> (usize, usize, usize) {
    let (mut optimal, mut certified, mut infeasible) = (0, 0, 0);
    for (what, lp) in suite() {
        let got = search::solve_search_certify::<P>(&lp, MAX_PIVOTS);
        match blp::solve(&lp) {
            LpResult::Optimal {
                objective: want, ..
            } => {
                optimal += 1;
                match got {
                    Ok(LpResult::Optimal { objective, .. }) => {
                        assert_eq!(objective, want, "P={P} {what}: wrong optimum");
                        certified += 1;
                    }
                    Ok(other) => panic!("P={P} {what}: unexpected {other:?}"),
                    Err(_) => {}
                }
            }
            LpResult::Infeasible => {
                infeasible += 1;
                assert!(got.is_err(), "P={P} {what}: infeasible LP gave {got:?}");
            }
        }
    }
    eprintln!("P={P}: certified {certified} of {optimal} optimal LPs; {infeasible} infeasible LPs all Err");
    (optimal, certified, infeasible)
}

#[test]
fn p120_certifies_every_optimal_lp_exactly() {
    let (optimal, certified, infeasible) = run::<120>();
    assert!(optimal > 500, "only {optimal} optimal LPs");
    assert!(infeasible > 0, "no infeasible LP in the suite");
    assert_eq!(certified, optimal, "certified {certified} of {optimal}");
}

#[test]
fn infeasible_lp_is_an_error() {
    // x <= -1 with 0 <= x <= 1.
    let lp = Lp {
        c: vec![q(1, 1)],
        rows: vec![(vec![q(1, 1)], q(-1, 1))],
        lower: vec![q(0, 1)],
        upper: vec![q(1, 1)],
    };
    assert!(search::search_basis::<120>(&lp, MAX_PIVOTS).is_none());
    assert!(search::solve_search_certify::<120>(&lp, MAX_PIVOTS).is_err());
}

/// Planted low precision: the search may fail, but never produce a wrong
/// optimum, because the certificate decides.
#[test]
fn p12_never_returns_a_wrong_optimum() {
    let (optimal, _certified, _) = run::<12>();
    assert!(optimal > 500, "only {optimal} optimal LPs");
}

/// `solve_search_certify_repair` on the whole suite must equal `blp`:
/// the same exact objective on every optimal LP, `Infeasible` on every
/// infeasible one. Returns (certified, repaired, cold) path counts.
fn run_repair<const P: u32>() -> (usize, usize, usize) {
    let (mut certified, mut repaired, mut cold, mut exact_pivots) = (0, 0, 0, 0u64);
    for (what, lp) in suite() {
        let (got, stats) = search::solve_search_certify_repair::<P>(&lp, MAX_PIVOTS);
        match (blp::solve(&lp), got) {
            (
                LpResult::Optimal {
                    objective: want, ..
                },
                LpResult::Optimal { objective, .. },
            ) => assert_eq!(objective, want, "P={P} {what}: wrong optimum"),
            (LpResult::Infeasible, LpResult::Infeasible) => {}
            (want, got) => panic!("P={P} {what}: blp gave {want:?}, repair gave {got:?}"),
        }
        exact_pivots += stats.exact_pivots;
        match stats.path {
            search::RepairPath::Certified => certified += 1,
            search::RepairPath::Repaired => repaired += 1,
            search::RepairPath::Cold => cold += 1,
        }
    }
    eprintln!(
        "repair P={P}: certified {certified}, repaired {repaired}, cold {cold}; {exact_pivots} exact pivots"
    );
    (certified, repaired, cold)
}

#[test]
fn repair_p120_equals_blp_on_every_lp() {
    let (certified, repaired, cold) = run_repair::<120>();
    assert!(certified + repaired + cold > 500);
    assert!(certified > 0);
}

/// Low precision only costs exact pivots: never a wrong answer.
#[test]
fn repair_p12_equals_blp_on_every_lp() {
    let (certified, repaired, cold) = run_repair::<12>();
    assert!(certified + repaired + cold > 500);
}

/// Planted: a certified basis with one basic index swapped for a nonbasic
/// one, so the certificate rejects it. `solve_from_basis` from that basis
/// must reach `blp`'s exact optimum. Counts the repairs that started warm.
#[test]
fn planted_corrupted_basis_is_repaired_exactly() {
    let (mut planted, mut warm, mut warm_pivots) = (0usize, 0usize, 0u64);
    for (what, lp) in suite() {
        let (
            LpResult::Optimal {
                objective: want, ..
            },
            Some(good),
        ) = blp::solve_with_basis(&lp)
        else {
            continue;
        };
        assert!(
            blp::certify_basis(&lp, &good).is_ok(),
            "{what}: good basis rejected"
        );
        let total = good.nonbasic_pos.len();
        let mut is_basic = vec![false; total];
        for &j in &good.basic {
            is_basic[j] = true;
        }
        let bad = (0..good.basic.len())
            .flat_map(|r| (0..total).filter(|&j| !is_basic[j]).map(move |j| (r, j)))
            .map(|(r, j)| {
                let mut b = good.clone();
                b.nonbasic_pos[b.basic[r]] = blp::Pos::Lower;
                b.basic[r] = j;
                b
            })
            .find(|b| blp::certify_basis(&lp, b).is_err());
        let Some(bad) = bad else { continue };
        planted += 1;
        let (got, stats) = blp::solve_from_basis_with_stats(&lp, &bad, None);
        match got {
            LpResult::Optimal { objective, .. } => {
                assert_eq!(
                    objective, want,
                    "{what}: repair from {bad:?} missed the optimum"
                )
            }
            other => panic!("{what}: repair gave {other:?}"),
        }
        if stats.start == blp::Start::Warm {
            warm += 1;
            warm_pivots += stats.pivots + stats.flips;
        }
    }
    eprintln!("planted: {planted} corrupted bases repaired, {warm} started warm ({warm_pivots} pivots+flips from warm starts)");
    assert!(planted > 100, "only {planted} planted repairs");
    assert!(warm > 0 && warm_pivots > 0, "no warm start did any work");
}

#[test]
fn lagrangian_dual_bound_rejects_negative_multiplier_and_wrong_length() {
    let p = &controls()[0];
    assert!(search::certify_lagrangian_dual_bound(p, &[q(-1, 4)]).is_err());
    assert!(search::certify_lagrangian_dual_bound(p, &[]).is_err());
    let zero_bound = search::certify_lagrangian_dual_bound(p, &[q(0, 1)]).expect("y=0 valid");
    assert!(zero_bound <= BigRational::zero());
}

#[test]
fn reduced_cost_fixing_rejects_negative_multiplier_and_invalid_upper_bound() {
    let p = &controls()[0];
    let u = BigRational::zero();
    assert!(search::certify_reduced_cost_fixing(p, &[q(-1, 4)], &u).is_err());
    assert!(search::certify_reduced_cost_fixing(p, &[], &u).is_err());
    let zero_bound = search::certify_lagrangian_dual_bound(p, &[q(0, 1)]).expect("y=0 valid");
    let invalid_u = &zero_bound - q(1, 4);
    assert!(search::certify_reduced_cost_fixing(p, &[q(0, 1)], &invalid_u).is_err());
    let valid_rcbt =
        search::certify_reduced_cost_fixing(p, &[q(0, 1)], &u).expect("y=0, u=0 valid");
    assert_eq!(valid_rcbt.lower_bound, zero_bound);
    assert_eq!(valid_rcbt.total_codes, p.n() * 15);
    assert_eq!(valid_rcbt.constrained_guards, 0);
    assert_eq!(valid_rcbt.guard_max_slack.len(), p.guards.len());
    let q0_codes: Vec<i64> = p.coeffs().iter().map(|c| c.q0).collect();
    assert!(search::certify_gap_decomposition(p, &[q(-1, 4)], &q0_codes).is_err());
    assert!(search::certify_gap_decomposition(p, &[q(0, 1)], &[0]).is_err());
    assert!(search::certify_gap_decomposition(p, &[q(0, 1)], &[99, 0]).is_err());
    let decomp = search::certify_gap_decomposition(p, &[q(0, 1)], &q0_codes).expect("valid q0");
    assert!(decomp.identity_holds);
    assert_eq!(decomp.primal_objective, BigRational::zero());
    assert_eq!(decomp.dual_lower_bound, zero_bound);
    assert_eq!(decomp.complementary_slackness_loss, BigRational::zero());
    assert_eq!(decomp.reduced_cost_loss, decomp.gap);
}

#[test]
fn dual_search_and_lp_guided_discrete_solve_respect_exact_bounds() {
    let mut problems = controls();
    problems.extend((0..60).map(instance));
    let mut certified_delta_hull = 0usize;
    let mut discrete_checked = 0usize;
    let mut total_eliminated_codes = 0usize;
    for (idx, p) in problems.iter().enumerate() {
        let dlp = search::delta_hull_lp(p);
        let (found_dual, _stats) = search::search_dual::<120>(&dlp, MAX_PIVOTS, None);
        let found_dual = found_dual.unwrap_or_else(|e| panic!("problem {idx} dual search: {e}"));
        let (cert, pi) = blp::certify_basis_with_dual(&dlp, &found_dual.basis)
            .unwrap_or_else(|e| panic!("problem {idx} delta_hull_lp certify: {e}"));
        let LpResult::Optimal {
            objective: dlp_obj, ..
        } = cert
        else {
            panic!("problem {idx}: delta_hull_lp not optimal");
        };
        let LpResult::Optimal {
            objective: want_dlp,
            ..
        } = blp::solve(&dlp)
        else {
            panic!("problem {idx}: blp delta_hull_lp not optimal");
        };
        assert_eq!(dlp_obj, want_dlp, "problem {idx}: delta_hull_lp mismatch");
        let exact_y: Vec<BigRational> = pi.into_iter().map(|v| -v).collect();
        let exact_lag = search::certify_lagrangian_dual_bound(p, &exact_y)
            .unwrap_or_else(|e| panic!("problem {idx} exact_y lagrangian bound: {e}"));
        assert_eq!(
            exact_lag, want_dlp,
            "problem {idx}: exact Lagrangian dual bound {exact_lag} != delta_hull_lp {want_dlp}"
        );
        certified_delta_hull += 1;

        let bb_res = bb::solve(p, 100_000);
        assert_eq!(
            bb_res.status,
            bb::Status::Optimal,
            "problem {idx}: bb status {:?}",
            bb_res.status
        );
        assert!(
            dlp_obj <= bb_res.objective,
            "problem {idx}: delta_hull_lp {dlp_obj} > discrete optimum {}",
            bb_res.objective
        );

        if let Ok(disc) = search::lp_guided_discrete_solve::<120>(p) {
            assert!(disc.full_guard_vector_matches);
            assert!(!disc.guard_min.is_negative());
            assert!(disc.objective <= BigRational::zero());
            assert!(
                disc.objective >= bb_res.objective,
                "problem {idx}: discrete solve {} < exact optimum {}",
                disc.objective,
                bb_res.objective
            );
            let lag = search::certify_lagrangian_dual_bound(p, &disc.dual_y)
                .expect("non-negative dual multipliers");
            assert!(
                lag <= bb_res.objective,
                "problem {idx}: lagrangian bound {lag} > discrete optimum {}",
                bb_res.objective
            );
            let (iter0_cert, iter0_pi) = blp::certify_basis_with_dual(&dlp, &disc.delta_lp_basis)
                .expect("iter0 delta_lp basis must certify");
            if let LpResult::Optimal { objective, .. } = iter0_cert {
                assert_eq!(objective, want_dlp);
            }
            let iter0_y: Vec<BigRational> = iter0_pi.into_iter().map(|v| -v).collect();
            let rcbt = search::certify_reduced_cost_fixing(p, &iter0_y, &disc.objective)
                .unwrap_or_else(|e| panic!("problem {idx} rcbt: {e}"));
            assert_eq!(rcbt.lower_bound, want_dlp);
            assert_eq!(rcbt.guard_max_slack.len(), p.guards.len());
            let decomp = search::certify_gap_decomposition(p, &iter0_y, &disc.codes)
                .unwrap_or_else(|e| panic!("problem {idx} gap_decomp: {e}"));
            assert!(decomp.identity_holds);
            assert_eq!(decomp.primal_objective, disc.objective);
            assert_eq!(decomp.dual_lower_bound, want_dlp);
            assert_eq!(decomp.gap, rcbt.gap);
            assert!(!decomp.reduced_cost_loss.is_negative());
            assert!(!decomp.complementary_slackness_loss.is_negative());
            total_eliminated_codes += rcbt.eliminated_codes;
            let mut opt_deltas = Vec::with_capacity(p.n());
            for (i, c) in p.coeffs().iter().enumerate() {
                let opt_k = i64::try_from(&bb_res.assignment.k[i]).expect("i64 step");
                let opt_q = c.q0 + opt_k;
                assert!(
                    rcbt.surviving_codes[i].contains(&opt_q),
                    "problem {idx} coeff {i}: optimal code {opt_q} eliminated by rcbt {:?}",
                    rcbt.surviving_codes[i]
                );
                let opt_d = if opt_k == 0 {
                    BigRational::zero()
                } else {
                    q(opt_k, 4) + &c.shift
                };
                opt_deltas.push(opt_d);
            }
            for (r, max_s_opt) in rcbt.guard_max_slack.iter().enumerate() {
                if let Some(max_s) = max_s_opt {
                    let s = p.guards[r]
                        .iter()
                        .zip(&opt_deltas)
                        .fold(BigRational::zero(), |acc, (a, d)| acc + a * d);
                    assert!(
                        !s.is_negative() && &s <= max_s,
                        "problem {idx} guard {r}: optimal guard slack {s} outside [0, {max_s}]"
                    );
                }
            }
            discrete_checked += 1;
        }
    }
    assert_eq!(certified_delta_hull, problems.len());
    assert!(
        discrete_checked > 50,
        "only {discrete_checked} discrete solves"
    );
    assert!(
        total_eliminated_codes > 0,
        "rcbt eliminated 0 codes across suite"
    );
}
