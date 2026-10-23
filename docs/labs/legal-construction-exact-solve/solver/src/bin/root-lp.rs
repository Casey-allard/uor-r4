//! `root-lp <dir>`: load a saved legal-construction case and solve its root
//! LP relaxation (`bb::relaxation`) with the exact bounded simplex `blp`.
//! Progress goes to stderr every 25 pivots; a JSON summary to stdout.

use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use legal_exact_solver::lp::LpResult;
use legal_exact_solver::{bb, blp, case};
use num_traits::Signed;
use serde_json::json;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: root-lp <dir>");
        return ExitCode::from(2);
    }
    let (p, _verified) = match case::load_verified(Path::new(&args[1])) {
        Ok((case, verified)) => (case.problem, verified),
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let lp = bb::relaxation(&p);
    eprintln!("root LP: {} vars, {} rows", lp.c.len(), lp.rows.len());
    let start = Instant::now();
    let mut report = |s: &blp::Stats| {
        let sign = if s.objective.is_negative() {
            "-"
        } else if s.objective.is_positive() {
            "+"
        } else {
            "0"
        };
        eprintln!(
            "pivots {} flips {} phase {} d_bits {} (max {}) elapsed_s {} objective sign {} num_bits {} den_bits {}",
            s.pivots,
            s.flips,
            s.phase,
            s.d_bits,
            s.d_bits_max,
            start.elapsed().as_secs(),
            sign,
            s.objective.numer().bits(),
            s.objective.denom().bits()
        );
    };
    let (result, stats) = blp::solve_with_stats(&lp, Some(&mut report));
    let wall_ms = start.elapsed().as_millis();
    let (status, objective) = match &result {
        LpResult::Infeasible => ("infeasible", None),
        LpResult::Optimal { objective, .. } => ("optimal", Some(objective.clone())),
    };
    let out = json!({
        "vars": lp.c.len(),
        "rows": lp.rows.len(),
        "status": status,
        "objective": objective.as_ref().map(|o| o.to_string()),
        "objective_is_zero": objective.as_ref().map(|o| o == &num_rational::BigRational::from_integer(0.into())),
        "pivots": stats.pivots,
        "flips": stats.flips,
        "phase1_pivots": stats.phase1_pivots,
        "d_bits_max": stats.d_bits_max,
        "wall_ms": wall_ms as u64,
    });
    println!("{}", serde_json::to_string_pretty(&out).expect("json"));
    ExitCode::SUCCESS
}
