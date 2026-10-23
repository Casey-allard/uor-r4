//! Exact single-coefficient legality census, best single move, and a greedy joint pass with exact
//! rational slack for the saved legal-construction case (#2828, deepseek-57).
//!
//! If only coefficient i moves by delta_i, every guard sees J_j . delta = J[j][i] * delta_i, so a
//! positive delta_i is legal exactly when every entry of column i is >= 0 and a negative one when
//! every entry is <= 0: an exact sign test with no solver numerics. The joint pass may re-destination
//! any coefficient, so a stop means no single-coefficient improvement remains (a local optimum in
//! single-coefficient space, not a global one). Every accepted move is checked on all 380 guard rows
//! and the returned assignment is recomputed from scratch from g and J: objective and the full guard
//! vector must equal the incremental ones exactly.
//!
//! It never claims an optimum unless it proves one. A feasible improving move is progress recorded
//! with its objective; the item stays open.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

use legal_exact_solver::case;
use legal_exact_solver::problem::Coeff;

fn arg<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn frac(r: &BigRational) -> String {
    format!("{}/{}", r.numer(), r.denom())
}

/// Every legal destination for one coefficient: (q, delta), with keep first.
fn destinations(c: &Coeff) -> Vec<(i64, BigRational)> {
    let mut out = Vec::with_capacity(15);
    out.push((c.q0, BigRational::zero()));
    for q in -7..c.q0 {
        let delta = BigRational::new(BigInt::from(q - c.q0), BigInt::from(4)) + &c.shift;
        out.push((q, delta));
    }
    for q in (c.q0 + 1)..=7 {
        let delta = BigRational::new(BigInt::from(q - c.q0), BigInt::from(4)) + &c.shift;
        out.push((q, delta));
    }
    out
}

fn column_has(g: &[Vec<BigRational>], i: usize) -> (bool, bool) {
    let mut has_pos = false;
    let mut has_neg = false;
    for row in g {
        let v = &row[i];
        if v.is_positive() {
            has_pos = true;
        } else if v.is_negative() {
            has_neg = true;
        }
    }
    (has_pos, has_neg)
}

#[derive(Default)]
struct Census {
    pos_only: usize,
    neg_only: usize,
    both_signs: usize,
    all_zero: usize,
    movable: usize,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = arg(&args, "--case")
        .map(PathBuf::from)
        .unwrap_or_else(case::default_case_dir);
    let max_moves: usize = arg(&args, "--max-moves")
        .and_then(|s| s.parse().ok())
        .unwrap_or(256);

    let t = Instant::now();
    let (case, verified) = match case::load_verified(&dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("load failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    let problem = &case.problem;
    let n = problem.n();
    let guards = problem.guards.len();
    println!(
        "load case={} seconds={:.3} n={} guards={} fractional={} hashes_verified={}",
        dir.display(),
        t.elapsed().as_secs_f64(),
        n,
        guards,
        case.fractional,
        verified.len()
    );

    let coeffs = problem.coeffs();
    let mut census = Census::default();
    let mut best_single: Option<(usize, i64, BigRational, BigRational)> = None;
    let t = Instant::now();
    for i in 0..n {
        let (has_pos, has_neg) = column_has(&problem.guards, i);
        let pos_ok = !has_neg;
        let neg_ok = !has_pos;
        match (has_pos, has_neg) {
            (false, false) => census.all_zero += 1,
            (true, false) => census.pos_only += 1,
            (false, true) => census.neg_only += 1,
            (true, true) => census.both_signs += 1,
        }
        if pos_ok || neg_ok {
            census.movable += 1;
        }
        for (q, delta) in destinations(&coeffs[i]) {
            if delta.is_zero() {
                continue;
            }
            let legal = (delta.is_positive() && pos_ok) || (delta.is_negative() && neg_ok);
            if !legal {
                continue;
            }
            let contribution = &problem.g[i] * &delta;
            if contribution.is_negative()
                && best_single
                    .as_ref()
                    .is_none_or(|b| contribution < b.3)
            {
                best_single = Some((i, q, delta, contribution));
            }
        }
    }
    println!(
        "census seconds={:.3} columns={} movable_single={} pos_only={} neg_only={} both_signs={} all_zero={}",
        t.elapsed().as_secs_f64(),
        n,
        census.movable,
        census.pos_only,
        census.neg_only,
        census.both_signs,
        census.all_zero
    );
    match &best_single {
        Some((i, q, delta, contribution)) => {
            let mut guard_min = BigRational::zero();
            for (j, row) in problem.guards.iter().enumerate() {
                let v = &row[*i] * delta;
                if j == 0 || v < guard_min {
                    guard_min = v;
                }
            }
            println!(
                "single_best i={} q={} delta={} contribution={} exact_guard_min={}",
                i,
                q,
                frac(delta),
                frac(contribution),
                frac(&guard_min)
            );
        }
        None => println!("single_best none (no legal single-coefficient change lowers g)"),
    }

    // Greedy joint pass with exact slack. Every coefficient may be re-destined; a stop means no
    // single-coefficient re-destination improves the objective (single-coefficient local optimum).
    let mut chosen: Vec<Option<(i64, BigRational)>> = vec![None; n];
    let mut slack: Vec<BigRational> = vec![BigRational::zero(); guards];
    let mut objective = BigRational::zero();
    let mut accepted = 0usize;
    let t = Instant::now();
    let mut stopped_at_cap = true;
    for _pass in 0..max_moves {
        let mut best: Option<(usize, i64, BigRational, BigRational)> = None;
        for i in 0..n {
            let current: BigRational = chosen[i]
                .as_ref()
                .map(|(_, d)| d.clone())
                .unwrap_or_else(BigRational::zero);
            let mut low: Option<BigRational> = None;
            let mut high: Option<BigRational> = None;
            let mut infeasible = false;
            for j in 0..guards {
                let a = &problem.guards[j][i];
                if a.is_zero() {
                    if slack[j].is_negative() {
                        infeasible = true;
                        break;
                    }
                    continue;
                }
                let bound = -&slack[j] / a;
                if a.is_positive() {
                    if low.as_ref().is_none_or(|l| bound > *l) {
                        low = Some(bound);
                    }
                } else if high.as_ref().is_none_or(|h| bound < *h) {
                    high = Some(bound);
                }
            }
            if infeasible {
                continue;
            }
            for (q, destination) in destinations(&coeffs[i]) {
                let change = &destination - &current;
                if change.is_zero() {
                    continue;
                }
                if low.as_ref().is_some_and(|l| change < *l) {
                    continue;
                }
                if high.as_ref().is_some_and(|h| change > *h) {
                    continue;
                }
                let contribution = &problem.g[i] * &change;
                if contribution.is_negative()
                    && best.as_ref().is_none_or(|b| contribution < b.3)
                {
                    best = Some((i, q, destination, contribution));
                }
            }
        }
        let Some((i, q, destination, contribution)) = best else {
            stopped_at_cap = false;
            println!("greedy stop: no improving single-coefficient re-destination remains");
            break;
        };
        let current = chosen[i]
            .as_ref()
            .map(|(_, d)| d.clone())
            .unwrap_or_else(BigRational::zero);
        let change = &destination - &current;
        for j in 0..guards {
            slack[j] += &problem.guards[j][i] * &change;
        }
        objective += &contribution;
        chosen[i] = Some((q, destination));
        accepted += 1;
        println!(
            "greedy move={} i={} q={} contribution={} objective={}",
            accepted,
            i,
            q,
            frac(&contribution),
            frac(&objective)
        );
    }

    // Independent recomputation from the returned assignment: objective from g, and all 380 guard
    // rows from J, with no use of the incremental slack. The full vector must match exactly.
    let mut objective2 = BigRational::zero();
    for (i, c) in chosen.iter().enumerate() {
        let Some((_, delta)) = c else { continue };
        objective2 += &problem.g[i] * delta;
    }
    let mut guard2: Vec<BigRational> = Vec::with_capacity(guards);
    for row in &problem.guards {
        let mut v = BigRational::zero();
        for (i, c) in chosen.iter().enumerate() {
            if let Some((_, delta)) = c {
                v += &row[i] * delta;
            }
        }
        guard2.push(v);
    }
    let matches = objective2 == objective
        && guard2.len() == slack.len()
        && guard2.iter().zip(slack.iter()).all(|(a, b)| a == b);
    let guard_min2 = guard2
        .iter()
        .cloned()
        .min()
        .unwrap_or_else(BigRational::zero);
    let active = guard2.iter().filter(|v| v.is_zero()).count();
    println!(
        "greedy summary seconds={:.3} moves={} cap_reached={} objective={} exact_guard_min={}",
        t.elapsed().as_secs_f64(),
        accepted,
        stopped_at_cap && accepted == max_moves,
        frac(&objective),
        frac(&guard_min2)
    );
    println!(
        "independent objective={} exact_guard_min={} full_guard_vector_matches={matches}",
        frac(&objective2),
        frac(&guard_min2)
    );
    if !matches {
        eprintln!("independent recomputation disagrees with the incremental pass");
        return ExitCode::FAILURE;
    }
    println!("active_guards={} of {}", active, guards);
    let moved: Vec<String> = chosen
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            c.as_ref().map(|(q, delta)| {
                format!("{{\"i\":{i},\"q\":{q},\"delta\":\"{}\"}}", frac(delta))
            })
        })
        .collect();
    println!("assignment_moved={} {}", moved.len(), moved.join(","));
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use legal_exact_solver::problem::Problem;

    fn q(v: i64) -> BigRational {
        BigRational::new(BigInt::from(v), BigInt::from(1))
    }

    #[test]
    fn destinations_include_keep_and_quarter_endpoints() {
        let p = Problem::new(
            vec![BigRational::new(BigInt::from(1), BigInt::from(2))],
            vec![q(1)],
            vec![vec![q(1)]],
        );
        let c = p.coeffs()[0].clone();
        let d = destinations(&c);
        assert_eq!(d.len(), 15, "keep plus 14 nonzero destinations");
        assert!(d.iter().any(|(q, _)| *q == 7));
        assert!(d.iter().any(|(q, _)| *q == -7));
    }

    #[test]
    fn sign_census_separates_directions() {
        let p = Problem::new(
            vec![q(0), q(0), q(0), q(0)],
            vec![q(-1), q(-1), q(-1), q(-1)],
            vec![
                vec![q(1), q(-1), q(0), q(1)],
                vec![q(2), q(-2), q(0), q(-1)],
            ],
        );
        let (mut pos_only, mut neg_only, mut both, mut zero) = (0, 0, 0, 0);
        for i in 0..p.n() {
            match column_has(&p.guards, i) {
                (false, false) => zero += 1,
                (true, false) => pos_only += 1,
                (false, true) => neg_only += 1,
                (true, true) => both += 1,
            }
        }
        assert_eq!((pos_only, neg_only, both, zero), (1, 1, 1, 1));
    }
}
