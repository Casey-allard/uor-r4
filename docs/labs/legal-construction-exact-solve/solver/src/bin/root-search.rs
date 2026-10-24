//! `root-search <dir>`: load a saved legal-construction case, propose a basis
//! for its root LP relaxation (`bb::relaxation`, 440 rows, 1,960 vars) and its
//! convex-hull `δ`-relaxation (`delta_hull_lp`, 380 rows, 1,920 vars) with the
//! `Dy<120>` search, prove both with the exact `blp::certify_basis`, certify a
//! basis-free Lagrangian dual bound, and run the warm-started `Dy<120>`
//! LP-guided discrete solver. Progress goes to stderr every 25 pivots; a JSON
//! summary to stdout.

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
    let (loaded, verified_files) = match case::load_verified(Path::new(&args[1])) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let p = loaded.problem;
    let lp = bb::relaxation(&p);
    eprintln!(
        "root LP: {} vars, {} rows ({} sha256-verified files)",
        lp.c.len(),
        lp.rows.len(),
        verified_files.len()
    );
    let start = Instant::now();
    let mut report = |s: &SearchStats| {
        let sign = match s.objective_sign {
            -1 => "-",
            1 => "+",
            _ => "0",
        };
        eprintln!(
            "pivots {} phase {} flips {} degen {} bland {} elapsed_s {} objective sign {} bits {}",
            s.pivots,
            s.phase,
            s.flips,
            s.degenerate_pivots,
            s.bland_switches,
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

    let delta_lp = search::delta_hull_lp(&p);
    let t_disc = Instant::now();
    let disc = search::lp_guided_discrete_solve::<120>(&p);
    let disc_ms = t_disc.elapsed().as_millis() as u64;

    let (delta_lp_json, lagrangian_bound_str, rcbt_json, discrete_json) = match disc {
        Ok(d) => {
            let t_dlp = Instant::now();
            let dlp_cert = blp::certify_basis_with_dual(&delta_lp, &d.delta_lp_basis);
            let dlp_ms = t_dlp.elapsed().as_millis() as u64;
            let (dlp_ok, dlp_obj, dlp_err, exact_y) = match dlp_cert {
                Ok((LpResult::Optimal { objective, .. }, pi)) => (
                    true,
                    Some(objective.to_string()),
                    None,
                    Some(pi.into_iter().map(|v| -v).collect::<Vec<_>>()),
                ),
                Ok((LpResult::Infeasible, _)) => (
                    false,
                    None,
                    Some("certificate: infeasible".to_string()),
                    None,
                ),
                Err(e) => (false, None, Some(format!("certificate: {e}")), None),
            };
            let dual_y_ref = exact_y.as_deref().unwrap_or(&d.dual_y);
            let lag_bound = search::certify_lagrangian_dual_bound(&p, dual_y_ref)
                .ok()
                .map(|b| b.to_string());
            let gap_decomp = search::certify_gap_decomposition(&p, dual_y_ref, &d.codes).ok();
            let rcbt = match search::certify_reduced_cost_fixing(&p, dual_y_ref, &d.objective) {
                Ok(r) => json!({
                    "fixed_coeffs": r.fixed_coeffs,
                    "binary_or_fixed_coeffs": r.binary_or_fixed_coeffs,
                    "eliminated_codes": r.eliminated_codes,
                    "total_codes": r.total_codes,
                    "constrained_guards": r.constrained_guards,
                    "gap": r.gap.to_string(),
                    "reduced_cost_loss": gap_decomp.as_ref().map(|g| g.reduced_cost_loss.to_string()),
                    "complementary_slackness_loss": gap_decomp.as_ref().map(|g| g.complementary_slackness_loss.to_string()),
                    "zero_reduced_cost_coeffs": gap_decomp.as_ref().map(|g| g.zero_reduced_cost_coeffs),
                    "gap_identity_holds": gap_decomp.as_ref().map(|g| g.identity_holds),
                    "dual_bound_matches_delta_hull_lp": lag_bound == dlp_obj,
                }),
                Err(e) => json!({ "error": e }),
            };
            let changed: Vec<serde_json::Value> = d
                .codes
                .iter()
                .zip(&d.deltas)
                .enumerate()
                .filter(|(_, (_, delta))| {
                    *delta != &num_rational::BigRational::from_integer(0.into())
                })
                .map(|(i, (&q, delta))| json!({ "i": i, "q": q, "delta": delta.to_string() }))
                .collect();
            (
                json!({
                    "vars": delta_lp.c.len(),
                    "rows": delta_lp.rows.len(),
                    "certified": dlp_ok,
                    "certify_ms": dlp_ms,
                    "objective": dlp_obj,
                    "reason": dlp_err,
                }),
                lag_bound,
                rcbt,
                json!({
                    "solve_ms": disc_ms,
                    "total_pivots": d.total_pivots,
                    "moved_coeffs": d.moved_coeffs,
                    "active_guards": d.active_guards,
                    "guard_min": d.guard_min.to_string(),
                    "objective": d.objective.to_string(),
                    "full_guard_vector_matches": d.full_guard_vector_matches,
                    "changed_count": changed.len(),
                }),
            )
        }
        Err(e) => (
            json!({ "certified": false, "reason": e }),
            None,
            json!({ "error": e }),
            json!({ "error": e }),
        ),
    };

    let out = json!({
        "verified_files": verified_files.len(),
        "vars": lp.c.len(),
        "rows": lp.rows.len(),
        "search_pivots": stats.pivots,
        "search_flips": stats.flips,
        "search_degenerate_pivots": stats.degenerate_pivots,
        "search_bland_switches": stats.bland_switches,
        "search_ms": search_ms,
        "certify_ms": certify_ms,
        "certified": certified,
        "objective": objective,
        "reason": reason,
        "delta_hull_lp": delta_lp_json,
        "lagrangian_dual_bound": lagrangian_bound_str,
        "reduced_cost_fixing": rcbt_json,
        "discrete_solution": discrete_json,
    });
    println!("{}", serde_json::to_string_pretty(&out).expect("json"));
    ExitCode::SUCCESS
}
