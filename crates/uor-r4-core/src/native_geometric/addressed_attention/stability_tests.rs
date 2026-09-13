use super::*;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
#[test]
fn instrumented_credit_matches_original_batch_and_target_cannot_change_its_context() -> TestResult {
    let parameters = Parameters::seeded(7341)?;
    let geometry = BoundGeometry::canonical()?;
    let document = [u16::from(b'A'), 256];
    let original = super::super::learner::batch(&parameters, &geometry, &document, 1973, 0)?;
    let diagnostic = instrumented_batch(&parameters, &geometry, &document, 1, 1973, 0)?;
    assert_eq!(original.mean_ce.to_bits(), diagnostic.mean_ce.to_bits());
    assert!(original
        .gradient
        .iter()
        .zip(&diagnostic.gradient)
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    assert_eq!(
        original.counts.scored_positions,
        diagnostic.counts.scored_positions
    );
    assert_eq!(original.counts.gate_events, diagnostic.counts.gate_events);
    assert_eq!(original.counts.head_events, diagnostic.counts.head_events);
    assert_eq!(
        original.counts.circuit_calls,
        diagnostic.counts.circuit_calls
    );
    assert!(
        ((diagnostic.prompt_ce + diagnostic.response_ce) / 2.0 - diagnostic.mean_ce).abs() < 1e-12
    );
    assert_eq!(diagnostic.particles.len(), 4);
    for trace in &diagnostic.particles {
        assert_eq!(trace.emission_contexts.len(), 2);
        assert_eq!(trace.offered_symbols.len(), 2);
        assert_eq!(trace.roots_before.len(), 2);
        assert_eq!(trace.selected.len(), 2);
    }
    for i in 0..PARAMETER_COUNT {
        assert!(
            (diagnostic.direct[i] + diagnostic.score_credit[i] - diagnostic.gradient[i]).abs()
                < 1e-12
        );
    }
    let changed = instrumented_batch(&parameters, &geometry, &[u16::from(b'B'), 256], 1, 1973, 0)?;
    for (a, b) in diagnostic.particles.iter().zip(&changed.particles) {
        assert_eq!(a.emission_contexts[0], b.emission_contexts[0]);
        assert_eq!(a.offered_symbols[0], b.offered_symbols[0]);
        assert_eq!(a.roots_before[0], b.roots_before[0]);
        assert_eq!(a.selected[0], b.selected[0]);
    }
    let compiled = parameters.compile()?;
    let a = deterministic_trace(&parameters, &compiled, &geometry, &document, 1)?;
    let b = deterministic_trace(
        &parameters,
        &compiled,
        &geometry,
        &[u16::from(b'B'), 256],
        1,
    )?;
    assert_eq!(a.trace.emission_contexts[0], b.trace.emission_contexts[0]);
    assert_eq!(a.trace.offered_symbols[0], b.trace.offered_symbols[0]);
    assert_eq!(a.trace.roots_before[0], b.trace.roots_before[0]);
    assert_eq!(a.trace.selected[0], b.trace.selected[0]);
    assert!(((a.prompt_ce + a.response_ce) / 2.0 - a.mean_ce).abs() < 1e-12);
    assert!(instrumented_batch(&parameters, &geometry, &document, 0, 1973, 0).is_err());
    assert!(deterministic_trace(&parameters, &compiled, &geometry, &document, 2).is_err());
    Ok(())
}
