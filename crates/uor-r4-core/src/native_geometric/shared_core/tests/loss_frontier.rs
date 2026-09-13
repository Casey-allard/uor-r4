//! Host-only exhaustive likelihood/error frontier; never writes model parameters.
use super::*;

#[derive(Clone, Copy)]
struct Choice {
    branch: CapBranch,
    errors: usize,
    nll: f64,
}

// Tail[y][a][b] counts raw angular scores >= a-4 and >= b-4.
// Index 9 is the empty tail, index 0 includes the whole [-4,4] domain.
type Tail = [[[usize; 10]; 10]; 2];

fn tails(samples: &[(i16, i16, bool)]) -> Tail {
    let mut tail = [[[0usize; 10]; 10]; 2];
    for &(a, b, y) in samples {
        assert!((-4..=4).contains(&a) && (-4..=4).contains(&b));
        tail[usize::from(y)][(a + 4) as usize][(b + 4) as usize] += 1;
    }
    for tag in &mut tail {
        for a in (0..9).rev() {
            for b in (0..9).rev() {
                tag[a][b] += tag[a + 1][b] + tag[a][b + 1] - tag[a + 1][b + 1];
            }
        }
    }
    tail
}

fn loss(score: i16, right: bool) -> f64 {
    let margin = f64::from(score) / 4.0;
    libm::log1p(libm::exp(if right { -margin } else { margin }))
}

fn both_stats(tail: &Tail, thresholds: [i16; 2], deltas: &[[f64; 18]; 2]) -> [(usize, f64); 2] {
    let base = tail[0][0][0] as f64 * loss(-9, false) + tail[1][0][0] as f64 * loss(-9, true);
    let mut nll = [base; 2];
    let mut errors = [0usize; 2];
    for k in -8i16..=9 {
        let a = (k + thresholds[0] + 4).clamp(0, 9) as usize;
        let b = (k + thresholds[1] + 4).clamp(0, 9) as usize;
        for tag in 0..2 {
            let intersection = tail[tag][a][b];
            let union = tail[tag][a][0] + tail[tag][0][b] - intersection;
            for (op, count) in [intersection, union].into_iter().enumerate() {
                nll[op] += count as f64 * deltas[tag][(k + 8) as usize];
                if k == 1 {
                    errors[op] += if tag == 0 {
                        count
                    } else {
                        tail[tag][0][0] - count
                    };
                }
            }
        }
    }
    [(errors[0], nll[0]), (errors[1], nll[1])]
}

fn deltas() -> [[f64; 18]; 2] {
    std::array::from_fn(|tag| {
        std::array::from_fn(|i| {
            let k = i as i16 - 8;
            loss(k, tag == 1) - loss(k - 1, tag == 1)
        })
    })
}

fn choose(slot: &mut Option<Choice>, candidate: Choice) {
    if slot.is_none_or(|old| candidate.nll < old.nll) {
        *slot = Some(candidate);
    }
}

fn replay(
    model: &SharedCore,
    rows: &[([u16; LANES], bool)],
    depth: usize,
    choice: Choice,
) -> AuditResult<Value> {
    let mut nll = 0.0;
    let mut errors = 0;
    for &(roots, target) in rows {
        let score = model.cap_score(roots, choice.branch, depth, &mut Work::default());
        nll += loss(score, target);
        errors += usize::from((score > 0) != target);
    }
    if errors != choice.errors || (nll - choice.nll).abs() > 1e-8 {
        return Err("Tail-sum optimum disagrees with actual branch replay".into());
    }
    Ok(
        json!({"landmarks":choice.branch.landmarks,"thresholds":choice.branch.thresholds,
        "union":choice.branch.union,"errors":errors,"host_nll":nll,
        "enumeration_nll":choice.nll,"replay_tolerance":1e-8}),
    )
}

pub(super) fn audit_node(
    model: &SharedCore,
    rows: &[([u16; LANES], bool)],
    node: usize,
    depth: usize,
    expected_errors: usize,
) -> AuditResult<Value> {
    let start = std::time::Instant::now();
    let current = model
        .artifact
        .calibrated
        .as_ref()
        .ok_or("Calibrated emitter")?
        .branches[node];
    let gate_max_errors = if node == 0 { 12 } else { 215 };
    let lane = depth & 3;
    let scores: [Vec<Vec<i16>>; 2] = std::array::from_fn(|side| {
        (0..120)
            .map(|landmark| {
                rows.iter()
                    .map(|&(roots, _)| {
                        model.relative_score(
                            roots[(lane + side) & 3],
                            landmark,
                            &mut Work::default(),
                        )
                    })
                    .collect()
            })
            .collect()
    });
    let current_samples: Vec<_> = rows
        .iter()
        .enumerate()
        .map(|(i, &(_, y))| {
            (
                scores[0][current.landmarks[0] as usize][i],
                scores[1][current.landmarks[1] as usize][i],
                y,
            )
        })
        .collect();
    let delta = deltas();
    let current_stats = both_stats(&tails(&current_samples), current.thresholds, &delta)
        [usize::from(current.union)];
    if current_stats.0 != expected_errors {
        return Err("Current error count disagrees with sealed calibration".into());
    }
    let current_report = replay(
        model,
        rows,
        depth,
        Choice {
            branch: current,
            errors: current_stats.0,
            nll: current_stats.1,
        },
    )?;
    let mut by_errors = vec![None; rows.len() + 1];
    let mut global = None;
    let mut admissible = None;
    let mut settings = 0usize;
    for first in 0..120 {
        if start.elapsed().as_secs() >= 25 {
            return Err("Frontier node exceeded 25-second internal cap".into());
        }
        for second in 0..120 {
            let samples: Vec<_> = rows
                .iter()
                .enumerate()
                .map(|(i, &(_, y))| (scores[0][first][i], scores[1][second][i], y))
                .collect();
            let tail = tails(&samples);
            for t0 in -5..=5 {
                for t1 in -5..=5 {
                    for (op, (errors, nll)) in
                        both_stats(&tail, [t0, t1], &delta).into_iter().enumerate()
                    {
                        settings += 1;
                        let candidate = Choice {
                            branch: CapBranch {
                                landmarks: [first as u16, second as u16],
                                thresholds: [t0, t1],
                                union: op == 1,
                            },
                            errors,
                            nll,
                        };
                        choose(&mut global, candidate);
                        choose(&mut by_errors[errors], candidate);
                        if errors <= gate_max_errors {
                            choose(&mut admissible, candidate);
                        }
                    }
                }
            }
        }
    }
    let best = global.ok_or("No global candidate")?;
    let allowed = admissible.ok_or("No gate-compatible candidate")?;
    if settings != 3_484_800 || best.nll > current_stats.1 + 1e-8 {
        return Err("Incomplete search or current setting excluded".into());
    }
    let min_errors = by_errors
        .iter()
        .position(Option::is_some)
        .ok_or("No errors frontier")?;
    if min_errors != if node == 0 { 10 } else { 202 } {
        return Err("Hard-error minimum disagrees with prior exact audit".into());
    }
    let frontier: Vec<_> = by_errors
        .into_iter()
        .flatten()
        .map(|choice| replay(model, rows, depth, choice))
        .collect::<AuditResult<_>>()?;
    Ok(
        json!({"node":node,"depth":depth,"positions":rows.len(),"gate_max_errors":gate_max_errors,
        "settings_examined":settings,"exact_minimum_hard_errors":min_errors,
        "current":current_report,"minimum_loss":replay(model,rows,depth,best)?,
        "minimum_gate_compatible_loss":replay(model,rows,depth,allowed)?,
        "lower_loss_admissible_setting_exists":allowed.nll < current_stats.1 - 1e-8,
        "global_minimum_meets_gate":best.errors <= gate_max_errors,
        "loss_error_frontier":frontier,"elapsed_ms":start.elapsed().as_millis(),
        "scope":"Complete finite parameter enumeration with f64 logistic loss; exact hard-error counts, numerically evaluated loss minima, fixed saved states. No hard-pattern deduplication, model mutation or fresh evaluation."}),
    )
}

#[test]
fn tail_sum_likelihood_matches_direct_min_max_with_extreme_thresholds() {
    let samples: Vec<_> = (-4i16..=4)
        .flat_map(|a| (-4i16..=4).flat_map(move |b| [(a, b, false), (a, b, true), (a, b, a > b)]))
        .collect();
    let tail = tails(&samples);
    for t0 in -5..=5 {
        for t1 in -5..=5 {
            for (op, (errors, nll)) in both_stats(&tail, [t0, t1], &deltas())
                .into_iter()
                .enumerate()
            {
                let (mut direct_errors, mut direct_loss) = (0, 0.0);
                for &(a, b, y) in &samples {
                    let score = if op == 0 {
                        (a - t0).min(b - t1)
                    } else {
                        (a - t0).max(b - t1)
                    };
                    direct_errors += usize::from((score > 0) != y);
                    direct_loss += loss(score, y);
                }
                assert_eq!(errors, direct_errors);
                assert!((nll - direct_loss).abs() < 1e-9);
            }
        }
    }
}
