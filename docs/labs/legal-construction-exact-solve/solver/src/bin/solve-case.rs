//! Solve the saved legal-construction case (`input245`) in exact arithmetic.
//!
//! This is the unit-3 instrument: it loads the saved case (mapping from commit
//! `ebd6ecaa`, see `case.rs`), records the exact zero-move check, and runs the
//! unit-2 exact LP and branch-and-bound with the recorded 4,096-node cap.
//! It reports what actually happened: an optimum, or a cap/instrument blocker.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use num_rational::BigRational;
use num_traits::Zero;

use legal_exact_solver::{bb, case, check, lp};

fn frac(r: &BigRational) -> String {
    format!("{}/{}", r.numer(), r.denom())
}

fn arg<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = arg(&args, "--case")
        .map(PathBuf::from)
        .unwrap_or_else(case::default_case_dir);
    let stage = arg(&args, "--stage").unwrap_or("all");
    if !matches!(stage, "load" | "identity" | "lp" | "bb" | "all") {
        eprintln!("--stage must be load, identity, lp, bb or all");
        return ExitCode::FAILURE;
    }
    let node_cap: usize = arg(&args, "--node-cap")
        .and_then(|s| s.parse().ok())
        .unwrap_or(4096);

    let t = Instant::now();
    let case = match case::load(&dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("load failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    let n = case.problem.n();
    println!(
        "load case={} seconds={:.3} n={} guards={} fractional={}",
        dir.display(),
        t.elapsed().as_secs_f64(),
        n,
        case.guard_rows,
        case.fractional
    );

    let zero = bb::Assignment::zero(n);
    let zero_report = check::check(&case.problem, &zero);
    println!(
        "zero_move legal={} guards_hold={} guard_value_min={} objective={}",
        zero_report.legal,
        zero_report.guards_hold,
        zero_report
            .guard_values
            .iter()
            .min()
            .map(frac)
            .unwrap_or_else(|| "none".into()),
        frac(&zero_report.objective)
    );
    if !zero_report.legal || !zero_report.guards_hold {
        eprintln!("the zero move is not legal/guarded; the loader mapping is wrong");
        return ExitCode::FAILURE;
    }

    if matches!(stage, "identity" | "all") {
        let t = Instant::now();
        match case::term_identity(&dir) {
            Ok(families) => {
                for (family, matches) in &families {
                    println!(
                        "identity family={family} f32_term_sum_bit_equal={matches} seconds={:.3}",
                        t.elapsed().as_secs_f64()
                    );
                }
                if families.iter().any(|(_, matches)| !matches) {
                    eprintln!("the f32 term sum does not reproduce the saved aggregate bits");
                    return ExitCode::FAILURE;
                }
            }
            Err(e) => {
                eprintln!("identity failed: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    if matches!(stage, "lp" | "all") {
        let t = Instant::now();
        let root = lp::solve(&bb::relaxation(&case.problem));
        let seconds = t.elapsed().as_secs_f64();
        match root {
            lp::LpResult::Infeasible => println!("root_lp status=infeasible seconds={seconds:.3}"),
            lp::LpResult::Optimal { objective, .. } => println!(
                "root_lp status=optimal seconds={seconds:.3} objective={}",
                frac(&objective)
            ),
        }
    }

    if matches!(stage, "bb" | "all") {
        let t = Instant::now();
        let outcome = bb::solve(&case.problem, node_cap);
        let seconds = t.elapsed().as_secs_f64();
        let report = check::check(&case.problem, &outcome.assignment);
        let mut moved = 0usize;
        for i in 0..n {
            if !outcome.assignment.k[i].is_zero()
                || outcome.assignment.l[i]
                || outcome.assignment.r[i]
            {
                moved += 1;
            }
        }
        println!(
            "bb status={:?} nodes={} seconds={:.3} objective={} best_bound={} moved={}",
            outcome.status,
            outcome.nodes,
            seconds,
            frac(&outcome.objective),
            frac(&outcome.best_bound),
            moved
        );
        println!(
            "bb_check legal={} guards_hold={} guard_value_min={} objective={}",
            report.legal,
            report.guards_hold,
            report
                .guard_values
                .iter()
                .min()
                .map(frac)
                .unwrap_or_else(|| "none".into()),
            frac(&report.objective)
        );
    }
    ExitCode::SUCCESS
}
