use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn actual_signatures_obey_bipolar_dot_identity_and_two_lane_distance() -> TestResult {
    let geometry = BoundGeometry::canonical()?;
    let metric = Metric::new(&geometry)?;
    assert_eq!(metric.geometry_digest(), geometry.identity_digest());
    for a in 0..ROOTS as u16 {
        let sa = geometry.signature(a)?;
        assert_eq!(metric.root_distance(a, a)?, 0);
        for b in 0..ROOTS as u16 {
            let sb = geometry.signature(b)?;
            // Independent coordinate arithmetic over the 120 actual predicates.
            let dot: i32 = (0..SIGNATURE_BITS)
                .map(|bit| {
                    let a = ((sa[bit >> 6] >> (bit & 63)) & 1) as i32;
                    let b = ((sb[bit >> 6] >> (bit & 63)) & 1) as i32;
                    (1 - 2 * a) * (1 - 2 * b)
                })
                .sum();
            let d = metric.root_distance(a, b)?;
            assert_eq!(dot, SIGNATURE_BITS as i32 - 2 * i32::from(d));
            assert_eq!(d, metric.root_distance(b, a)?);
            assert!(d <= 120);
            assert_eq!(metric.distance([a, b], [b, a])?, d + d);
            assert_eq!(metric.distance([a, b], [a, a])?, d);
        }
    }
    Ok(())
}

#[test]
fn root_domains_and_signature_padding_fail_closed() -> TestResult {
    let geometry = BoundGeometry::canonical()?;
    let metric = Metric::new(&geometry)?;
    for invalid in [120, u16::MAX] {
        assert_eq!(
            metric.root_distance(invalid, 0),
            Err(MetricError::RootOutOfRange)
        );
        assert_eq!(
            metric.root_distance(0, invalid),
            Err(MetricError::RootOutOfRange)
        );
        for (q, k) in [
            ([invalid, 0], [0, 0]),
            ([0, invalid], [0, 0]),
            ([0, 0], [invalid, 0]),
            ([0, 0], [0, invalid]),
        ] {
            assert_eq!(metric.distance(q, k), Err(MetricError::RootOutOfRange));
        }
    }
    for padding_bit in 56..64 {
        let mut signatures = metric.signatures;
        signatures[119][1] |= 1u64 << padding_bit;
        assert_eq!(
            Metric::from_signatures(signatures, metric.geometry_digest()),
            Err(MetricError::SignaturePadding)
        );
    }
    Ok(())
}

#[test]
fn census_counts_complete_domains_and_ranking_ties_without_semantic_claims() -> TestResult {
    let geometry = BoundGeometry::canonical()?;
    let report = census(&geometry)?;
    assert_eq!(report.roots, 120);
    assert_eq!(report.signature_bits, 120);
    assert_eq!(report.ordered_root_pairs, 14_400);
    assert!(report.reflexive && report.symmetric);
    assert_eq!(report.distance_histogram.iter().sum::<usize>(), 14_400);
    assert_eq!(
        report.signature_population_histogram.iter().sum::<usize>(),
        120
    );
    assert_eq!(
        report.distance_histogram[0],
        120 + report.colliding_ordered_distinct_root_pairs
    );
    assert!(report.distinct_signatures <= 120);
    assert_eq!(
        report
            .minimum_angular_partner_hamming_histogram
            .iter()
            .sum::<usize>(),
        report.minimum_angular_partner_pairs
    );
    assert!(report.unique_minimum_angular_partner_for_every_root);
    assert!(report.minimum_angular_score_negates_self_for_every_root);
    assert_eq!(report.minimum_angular_partner_pairs, 120);
    let r = &report.ranking;
    assert_eq!(r.query_candidate_pair_comparisons, 856_800);
    assert_eq!(
        r.same_strict_preference
            + r.opposite_strict_preference
            + r.both_tied
            + r.angular_tied_hamming_strict
            + r.hamming_tied_angular_strict,
        r.query_candidate_pair_comparisons
    );
    let bytes = serde_json::to_vec(&report)?;
    let decoded: MetricCensus = serde_json::from_slice(&bytes)?;
    assert_eq!(decoded, report);
    Ok(())
}
