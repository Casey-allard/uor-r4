use super::*;
use crate::native_geometric::addressed_attention::{
    artifact::BoundGeometry,
    objects::{Action, ObjectSession, Symbol},
    policy::Parameters,
    stability,
    training::cross_entropy,
};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "{actual:.17} != {expected:.17}"
    );
}

#[test]
fn causal_same_boundary_loo_excludes_prior_loss_and_own_baseline() -> TestResult {
    let losses = vec![
        vec![1.0, 2.0],
        vec![3.0, 4.0],
        vec![5.0, 6.0],
        vec![7.0, 8.0],
    ];
    let weights = suffix_loo_weights(&losses)?;
    close(weights[0][0], -2.0);
    close(weights[0][1], -1.0);
    for row in &weights {
        assert_eq!(row[2], 0.0);
    }
    // The late event cannot inherit any particle's already completed loss.
    let mut changed_prior = losses.clone();
    for (i, row) in changed_prior.iter_mut().enumerate() {
        row[0] += 100.0 * (i + 1) as f64;
    }
    let changed = suffix_loo_weights(&changed_prior)?;
    for i in 0..4 {
        assert_eq!(weights[i][1], changed[i][1]);
    }
    // Own cost has coefficient 1/4; each independent other-particle cost has
    // coefficient -1/12. Including self in its baseline would fail this check.
    let mut changed_own = losses.clone();
    changed_own[0][1] += 12.0;
    let changed = suffix_loo_weights(&changed_own)?;
    close(changed[0][1] - weights[0][1], 3.0);
    for i in 1..4 {
        close(changed[i][1] - weights[i][1], -1.0);
    }
    // Different event counts across paths do not change cost-boundary alignment.
    let tapes = vec![
        ScoreTape {
            events: vec![ScoreEvent {
                parameter: 0,
                score: 1.0,
            }],
            boundaries: vec![0, 0, 1],
        },
        ScoreTape {
            events: vec![],
            boundaries: vec![0, 0, 0],
        },
        ScoreTape {
            events: vec![
                ScoreEvent {
                    parameter: 1,
                    score: 7.0,
                },
                ScoreEvent {
                    parameter: 1,
                    score: -7.0,
                },
            ],
            boundaries: vec![0, 2, 2],
        },
        ScoreTape {
            events: vec![],
            boundaries: vec![0, 0, 0],
        },
    ];
    let reduced = reduce_causal_score(&tapes, &losses, 2)?;
    let without_prior = reduce_causal_score(&tapes, &changed_prior, 2)?;
    close(reduced[0], -1.0);
    assert_eq!(reduced[0], without_prior[0]);
    close(reduced[1], 0.0);
    Ok(())
}

#[test]
fn causal_pre_ce_event_keeps_current_loss_analytic_expectation() -> TestResult {
    let p = 0.25;
    let mut expectation = 0.0;
    // Exhaust all four independent particles, not a selected Monte Carlo draw.
    for outcomes in 0u8..16 {
        let mut probability = 1.0;
        let mut losses = Vec::new();
        let mut tapes = Vec::new();
        for particle in 0..4 {
            let selected = outcomes & (1 << particle) != 0;
            probability *= if selected { p } else { 1.0 - p };
            // Unequal prior costs make accidental old full-cost centering visible
            // in each draw; they are independent of the event being scored.
            losses.push(vec![
                100.0 + particle as f64,
                if selected { 1.0 } else { 5.0 },
            ]);
            tapes.push(ScoreTape {
                events: vec![ScoreEvent {
                    parameter: 0,
                    score: f64::from(u8::from(selected)) - p,
                }],
                boundaries: vec![0, 0, 1],
            });
        }
        expectation += probability * reduce_causal_score(&tapes, &losses, 1)?[0];
    }
    // d/dlogit E[current cost] = p(1-p)(cost_true-cost_false).
    close(expectation, p * (1.0 - p) * (1.0 - 5.0));
    assert!(expectation.abs() > 0.5, "dropping current CE must not pass");
    Ok(())
}

fn publication_future_loss(
    offered_correctly: bool,
) -> std::result::Result<f64, Box<dyn std::error::Error>> {
    let mut session = ObjectSession::new([1; 32], [2; 32], 7);
    session.observe_input(b'1', [0; 2], [0; 4])?;
    session.observe_input(b'2', [0; 2], [0; 4])?;
    let a = session.acquire_occurrence(7, 0, 1)?;
    let b = session.acquire_occurrence(7, 1, 1)?;
    let offer = session.cache_offer(
        Symbol::Byte(if offered_correctly { b'3' } else { b'4' }),
        Action::AddAB,
        Some(a),
        Some(b),
    )?;
    assert!(session.result(7, 1).is_err());
    session.acknowledge(Symbol::Byte(b'3'), offer.id)?;
    assert!(session.result(7, 1).is_err(), "publication waits for KEY");
    session.commit_key([0; 2], [0; 4])?;
    assert_eq!(
        session.publications_this_turn(),
        u8::from(offered_correctly)
    );
    if offered_correctly {
        assert_eq!(session.result(7, 1)?.lease.payload(), b"3");
        Ok(0.5)
    } else {
        assert!(session.result(7, 1).is_err());
        Ok(2.0)
    }
}

#[test]
fn causal_post_ce_offer_excludes_current_but_keeps_actual_publication_future() -> TestResult {
    let p: f64 = 0.25;
    let ce = cross_entropy(&[0.0, (p / (1.0 - p)).ln()], &[true, true], 1, 2)?;
    let future = [
        publication_future_loss(false)?,
        publication_future_loss(true)?,
    ];
    let mut expected_causal = 0.0;
    let mut expected_old = 0.0;
    for outcomes in 0u8..16 {
        let mut probability = 1.0;
        let mut losses = Vec::new();
        let mut tapes = Vec::new();
        let mut scores = Vec::new();
        for particle in 0..4 {
            let selected = outcomes & (1 << particle) != 0;
            probability *= if selected { p } else { 1.0 - p };
            let score = f64::from(u8::from(selected)) - p;
            scores.push(score);
            losses.push(vec![ce.loss, future[usize::from(selected)] / 2.0]);
            // Score 0 is the offered-symbol event after CE_0 and before CE_1.
            // Score 1 stands for a final offered-symbol/KEY event after CE_1:
            // it has no remaining target cost and must contribute exactly zero.
            tapes.push(ScoreTape {
                events: vec![
                    ScoreEvent {
                        parameter: 0,
                        score,
                    },
                    ScoreEvent {
                        parameter: 1,
                        score: 11.0,
                    },
                ],
                boundaries: vec![0, 0, 1],
            });
        }
        let reduced = reduce_causal_score(&tapes, &losses, 2)?;
        close(reduced[1], 0.0);
        let mut changed_current = losses.clone();
        for (i, loss) in changed_current.iter_mut().enumerate() {
            loss[0] += 10.0 * i as f64;
        }
        assert_eq!(
            reduced,
            reduce_causal_score(&tapes, &changed_current, 2)?,
            "post-CE score cannot include any current CE baseline"
        );
        expected_causal += probability * reduced[0];
        let totals: Vec<f64> = losses.iter().map(|row| row.iter().sum()).collect();
        for i in 0..4 {
            let baseline = totals
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, x)| x)
                .sum::<f64>()
                / 3.0;
            expected_old += probability * (totals[i] - baseline) * scores[i] / 4.0;
        }
    }
    let analytic_future = p * (1.0 - p) * (0.5 - 2.0) / 2.0;
    close(expected_causal, analytic_future);
    close(expected_old, analytic_future);
    close(
        ce.direct[1] + expected_causal,
        (p - 1.0) / 2.0 + analytic_future,
    );
    assert!(
        expected_causal.abs() > 0.1,
        "dropping publication credit must fail"
    );
    Ok(())
}

#[test]
fn causal_paired_batch_preserves_actual_old_paths_losses_direct_and_gradient() -> TestResult {
    let parameters = Parameters::seeded(7341)?;
    let before = parameters.digest();
    let geometry = BoundGeometry::canonical()?;
    let document = [u16::from(b'A'), 256];
    let old = stability::instrumented_batch(&parameters, &geometry, &document, 1, 1973, 0)?;
    let paired = paired_batch(&parameters, &geometry, &document, 1, 1973, 0)?;
    assert_eq!(old.particles, paired.particles);
    assert_eq!(old.mean_ce.to_bits(), paired.mean_ce.to_bits());
    assert_eq!(old.prompt_ce.to_bits(), paired.prompt_ce.to_bits());
    assert_eq!(old.response_ce.to_bits(), paired.response_ce.to_bits());
    assert_eq!(old.gradient.len(), paired.old_gradient.len());
    assert_eq!(old.direct.len(), paired.direct.len());
    for (a, b) in old.gradient.iter().zip(&paired.old_gradient) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
    for (a, b) in old.direct.iter().zip(&paired.direct) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
    assert_eq!(old.counts.scored_positions, paired.counts.scored_positions);
    assert_eq!(old.counts.gate_events, paired.counts.gate_events);
    assert_eq!(old.counts.head_events, paired.counts.head_events);
    assert_eq!(old.counts.circuit_calls, paired.counts.circuit_calls);
    assert_eq!(paired.causal_gradient.len(), old.gradient.len());
    assert!(paired.causal_gradient.iter().all(|value| value.is_finite()));
    assert_eq!(parameters.digest(), before);
    Ok(())
}
