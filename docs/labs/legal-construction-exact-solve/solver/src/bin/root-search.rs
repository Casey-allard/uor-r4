//! `root-search <dir>`: load a saved legal-construction case, propose a basis
//! for its root LP relaxation with the `Dy<120>` search, then prove or reject
//! it with the exact `blp::certify_basis`. Progress goes to stderr every 25
//! pivots; a JSON summary to stdout.

use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use legal_exact_solver::lp::LpResult;
use legal_exact_solver::search::{self, SearchStats};
use legal_exact_solver::{bb, blp, case};
use serde_json::json;

const MAX_PIVOTS: usize = 10_000_000;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: root-search <dir>");
        return ExitCode::from(2);
    }
    let p = match case::load_verified(Path::new(&args[1])) {
        Ok((case, _verified)) => case.problem,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let lp = bb::relaxation(&p);
    eprintln!("root LP: {} vars, {} rows", lp.c.len(), lp.rows.len());
    let start = Instant::now();
    let mut report = |s: &SearchStats| {
        let sign = match s.objective_sign {
            -1 => "-",
            1 => "+",
            _ => "0",
        };
        eprintln!(
            "pivots {} phase {} flips {} elapsed_s {} objective sign {} bits {}",
            s.pivots,
            s.phase,
            s.flips,
            start.elapsed().as_secs(),
            sign,
            s.objective_bits
        );
    };
    let (found, stats) = search::search::<120>(&lp, MAX_PIVOTS, Some(&mut report));
    let search_ms = start.elapsed().as_millis() as u64;
    let (certify_ms, certified, objective, reason) = match found {
        Err(e) => (None, false, None, Some(format!("search: {e}"))),
        Ok(f) => {
            let t = Instant::now();
            let got = blp::certify_basis(&lp, &f.basis);
            let ms = t.elapsed().as_millis() as u64;
            match got {
                Ok(LpResult::Optimal { objective, .. }) => {
                    (Some(ms), true, Some(objective.to_string()), None)
                }
                Ok(LpResult::Infeasible) => (
                    Some(ms),
                    false,
                    None,
                    Some("certificate: infeasible".into()),
                ),
                Err(e) => (Some(ms), false, None, Some(format!("certificate: {e}"))),
            }
        }
    };
    let out = json!({
        "vars": lp.c.len(),
        "rows": lp.rows.len(),
        "search_pivots": stats.pivots,
        "search_flips": stats.flips,
        "search_ms": search_ms,
        "certify_ms": certify_ms,
        "certified": certified,
        "objective": objective,
        "reason": reason,
    });
    println!("{}", serde_json::to_string_pretty(&out).expect("json"));
    ExitCode::SUCCESS
}
