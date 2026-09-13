use super::*;
use crate::native_geometric::addressed_attention::objects::{
    Action, ObjectSession, Reference, Symbol,
};
use crate::native_geometric::Geometry;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn close(a: f64, b: f64, tolerance: f64) {
    assert!((a - b).abs() <= tolerance, "{a:.17} != {b:.17}");
}
fn key(event: u64) -> EventKey {
    EventKey {
        seed: 973,
        batch: 2,
        particle: 0,
        document: 1,
        event,
        phase: 0,
        kind: EventKind::Gate,
        slot: 3,
    }
}
fn logit(p: f64) -> f64 {
    libm::log(p / (1.0 - p))
}

#[test]
fn invocation_rng_masks_normalized_ce_and_invalid_inputs() -> TestResult {
    let original = key(0);
    let u = uniform01(original);
    assert_eq!(u, uniform01(original));
    assert!((0.0..1.0).contains(&u));
    for changed in [
        EventKey {
            seed: 974,
            ..original
        },
        EventKey {
            batch: 3,
            ..original
        },
        EventKey {
            particle: 1,
            ..original
        },
        EventKey {
            document: 2,
            ..original
        },
        EventKey {
            event: 1,
            ..original
        },
        EventKey {
            phase: 1,
            ..original
        },
        EventKey {
            kind: EventKind::Head,
            ..original
        },
        EventKey {
            slot: 4,
            ..original
        },
    ] {
        assert_ne!(u, uniform01(changed));
    }
    let draws = (0..32)
        .map(|i| bernoulli(0.0, key(i)))
        .collect::<TrainingResult<Vec<_>>>()?;
    assert!(draws.iter().any(|d| d.outcome));
    assert!(draws.iter().any(|d| !d.outcome));
    for d in &draws {
        close(d.probability, 0.5, 0.0);
        close(d.score, f64::from(u8::from(d.outcome)) - 0.5, 0.0);
    }
    // A shared row is sampled again on a new invocation, never cached globally.
    assert_eq!(
        draws.iter().map(|d| d.score).sum::<f64>(),
        draws.iter().filter(|d| d.outcome).count() as f64 - 16.0
    );
    assert!(bernoulli(f64::NAN, key(0)).is_err());
    let logits = [1000.0, 999.0, -1000.0];
    let legal = [false, true, true];
    let p = categorical_probabilities(&logits, &legal)?;
    close(p.iter().sum(), 1.0, 1e-15);
    close(p[0], 0.0, 0.0);
    for event in 0..32 {
        let d = categorical(
            &logits,
            &legal,
            EventKey {
                kind: EventKind::Head,
                ..key(event)
            },
        )?;
        assert!(legal[d.outcome]);
        close(d.score[0], 0.0, 0.0);
        close(d.score.iter().sum(), 0.0, 1e-14);
    }
    let ce = cross_entropy(&[0.0, logit(0.2)], &[true, true], 1, 2)?;
    close(ce.loss, -libm::log(0.2) / 2.0, 1e-14);
    close(ce.direct[1], -0.4, 1e-14);
    close(ce.direct.iter().sum(), 0.0, 1e-14);
    assert_eq!(
        cross_entropy(&[0.0], &[true], 0, 0),
        Err(TrainingError::InvalidNormalizer)
    );
    assert_eq!(
        categorical_probabilities(&[0.0], &[false]),
        Err(TrainingError::EmptyLegalSupport)
    );
    assert_eq!(
        categorical_probabilities(&[0.0], &[]),
        Err(TrainingError::Dimensions)
    );
    assert_eq!(
        categorical_probabilities(&[], &[]),
        Err(TrainingError::Dimensions)
    );
    assert_eq!(
        categorical_probabilities(&[f64::INFINITY], &[true]),
        Err(TrainingError::NonFinite)
    );
    assert_eq!(
        cross_entropy(&[0.0, 1.0], &[true, false], 1, 1),
        Err(TrainingError::InvalidTarget)
    );
    assert_eq!(
        cross_entropy(&[0.0], &[true], 1, 1),
        Err(TrainingError::InvalidTarget)
    );
    Ok(())
}

struct H4Fixture {
    geometry: Geometry,
    ranks: Vec<i16>,
    minus: u16,
    objects: ObjectSession,
    sources: [Reference; 2],
}
impl H4Fixture {
    fn new() -> TestResult<Self> {
        let geometry = crate::native_geometric::training::geometry(256, 256)?;
        let identity = geometry.anchors.rows[usize::from(geometry.identity)].root_scaled_zphi;
        let negative = identity.map(|v| [-v[0], -v[1]]);
        let minus = geometry
            .anchors
            .rows
            .iter()
            .find(|r| r.root_scaled_zphi == negative)
            .ok_or("exact central minus-I missing")?
            .root_index;
        let mut values: Vec<_> = geometry
            .anchors
            .rows
            .iter()
            .map(|r| r.root_scaled_zphi[0])
            .collect();
        values.sort_by(|a, b| {
            crate::native_geometric::training::exact_sign([a[0] - b[0], a[1] - b[1]]).cmp(&0)
        });
        values.dedup();
        let zero = values
            .iter()
            .position(|v| *v == [0, 0])
            .ok_or("angular zero")? as i16;
        let ranks = geometry
            .anchors
            .rows
            .iter()
            .map(|r| {
                values
                    .iter()
                    .position(|v| *v == r.root_scaled_zphi[0])
                    .map(|i| i as i16 - zero)
                    .ok_or("angular rank")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut objects = ObjectSession::new([1; 32], [2; 32], 7);
        for (byte, root) in [(b'A', geometry.identity), (b'B', minus)] {
            objects.observe_input(byte, [root; 2], [root; 4])?;
        }
        let a = objects.acquire_occurrence(7, 0, 1)?;
        let b = objects.acquire_occurrence(7, 1, 1)?;
        assert_eq!(a.payload(), b"A");
        assert_eq!(b.payload(), b"B");
        let sources = [a.reference(), b.reference()];
        assert_ne!(sources[0], sources[1]);
        let result = Self {
            geometry,
            ranks,
            minus,
            objects,
            sources,
        };
        assert_ne!(result.minus, result.geometry.identity);
        assert_eq!(result.product(minus, minus), result.geometry.identity);
        for r in 0..120 {
            assert_eq!(result.product(minus, r), result.product(r, minus));
        }
        assert_eq!(result.source(result.geometry.identity)?, 0);
        assert_eq!(result.source(minus)?, 1);
        Ok(result)
    }
    fn product(&self, a: u16, b: u16) -> u16 {
        self.geometry.products[self.geometry.row_bases[usize::from(a)] + usize::from(b)]
    }
    fn source(&self, query: u16) -> TestResult<usize> {
        let mut scores = [0i16; 2];
        for (index, reference) in self.sources.iter().enumerate() {
            let Reference::Occurrence {
                epoch,
                turn,
                start,
                end,
            } = *reference
            else {
                return Err("source must be exact occurrence".into());
            };
            let occurrence = self.objects.occurrence(epoch, start)?;
            assert_eq!(occurrence.turn, turn);
            assert_eq!(end, start + 1);
            let lease = self.objects.acquire_occurrence(epoch, start, 1)?;
            assert_eq!(lease.reference(), *reference);
            assert_eq!(lease.payload(), &[occurrence.byte]);
            // Both signed lanes come from the actual stored occurrence.
            scores[index] = occurrence
                .keys
                .iter()
                .map(|key| {
                    self.ranks[usize::from(
                        self.product(query, self.geometry.inverses[usize::from(*key)]),
                    )]
                })
                .sum();
        }
        assert_ne!(
            scores[0], scores[1],
            "toy matching source must win strictly"
        );
        let selected = self.sources[usize::from(scores[1] > scores[0])];
        self.sources
            .iter()
            .position(|reference| *reference == selected)
            .ok_or_else(|| "selected full source identity missing".into())
    }
    fn path(&self, actions: [bool; 2], p: f64, q: f64) -> TestResult<Path> {
        self.path_with_row_control(actions, p, q, false)
    }
    fn path_with_row_control(
        &self,
        actions: [bool; 2],
        p: f64,
        q: f64,
        freeze_second_row: bool,
    ) -> TestResult<Path> {
        let mut state = self.geometry.identity;
        state = self.product(
            state,
            if actions[0] {
                self.minus
            } else {
                self.geometry.identity
            },
        );
        let first = self.source(state)?;
        let row = if freeze_second_row { 0 } else { first };
        let second_probability = if row == 0 { p } else { q };
        state = self.product(
            state,
            if actions[1] {
                self.minus
            } else {
                self.geometry.identity
            },
        );
        let final_source = self.source(state)?;
        let mut score = [f64::from(u8::from(actions[0])) - p, 0.0];
        score[row] += f64::from(u8::from(actions[1])) - second_probability;
        Ok(Path {
            actions,
            first_source: first,
            second_row: row,
            final_source,
            cost: f64::from(u8::from(final_source != 1)),
            probability: (if actions[0] { p } else { 1.0 - p })
                * (if actions[1] {
                    second_probability
                } else {
                    1.0 - second_probability
                }),
            score,
        })
    }
    fn paths(&self, p: f64, q: f64) -> TestResult<Vec<Path>> {
        (0..4)
            .map(|i| self.path([i & 1 != 0, i & 2 != 0], p, q))
            .collect()
    }
}
#[derive(Clone)]
struct Path {
    actions: [bool; 2],
    first_source: usize,
    second_row: usize,
    final_source: usize,
    cost: f64,
    probability: f64,
    score: [f64; 2],
}
fn exact_cost(paths: &[Path]) -> f64 {
    paths.iter().map(|p| p.probability * p.cost).sum()
}
fn reweighted_cost(paths: &[Path], p: f64, q: f64) -> f64 {
    paths
        .iter()
        .map(|path| {
            let second = if path.second_row == 0 { p } else { q };
            (if path.actions[0] { p } else { 1.0 - p })
                * (if path.actions[1] {
                    second
                } else {
                    1.0 - second
                })
                * path.cost
        })
        .sum()
}

#[test]
fn hard_h4_changed_source_shared_row_exact_credit_and_exports() -> TestResult {
    let h4 = H4Fixture::new()?;
    for (p, q, expected_cost, expected_gradient) in [
        (0.2, 0.7, 0.78, [-0.144, 0.042]),
        (0.65, 0.25, 0.285, [-0.102375, 0.121875]),
    ] {
        let paths = h4.paths(p, q)?;
        close(paths.iter().map(|r| r.probability).sum(), 1.0, 1e-14);
        close(exact_cost(&paths), expected_cost, 1e-14);
        close(exact_cost(&paths), 1.0 - 2.0 * p + p * p + p * q, 1e-14);
        for path in &paths {
            assert_eq!(path.first_source, usize::from(path.actions[0]));
            assert_eq!(path.second_row, path.first_source);
            assert_eq!(
                path.final_source,
                usize::from(path.actions[0] ^ path.actions[1])
            );
        }
        let expected = [(-2.0 + 2.0 * p + q) * p * (1.0 - p), p * q * (1.0 - q)];
        let actual: [f64; 2] = std::array::from_fn(|r| {
            paths
                .iter()
                .map(|v| v.probability * v.cost * v.score[r])
                .sum()
        });
        for row in 0..2 {
            close(actual[row], expected[row], 1e-14);
            close(actual[row], expected_gradient[row], 1e-14);
            let mut logits = [logit(p), logit(q)];
            logits[row] += 1e-5;
            let plus = reweighted_cost(
                &paths,
                bernoulli_probability(logits[0])?,
                bernoulli_probability(logits[1])?,
            );
            logits[row] -= 2e-5;
            let minus = reweighted_cost(
                &paths,
                bernoulli_probability(logits[0])?,
                bernoulli_probability(logits[1])?,
            );
            close((plus - minus) / (2e-5), actual[row], 1e-7);
        }
        // Exact expectation of the two-particle LOO estimator over 16 pairs.
        let mut expected_loo = [0.0; 2];
        for a in &paths {
            for b in &paths {
                let parameters = [logit(p), logit(q)];
                let mut batch = SerialLoo::new(&parameters, 2)?;
                batch.add_particle(&parameters, a.cost, &a.score, &[0.0; 2])?;
                batch.add_particle(&parameters, b.cost, &b.score, &[0.0; 2])?;
                let gradient = batch.finish(&parameters)?;
                for r in 0..2 {
                    expected_loo[r] += a.probability * b.probability * gradient[r];
                }
            }
        }
        for r in 0..2 {
            close(expected_loo[r], actual[r], 1e-14);
        }
        // Negative control: freezing the second source/address to j0 loses q.
        let wrong_paths = (0..4)
            .map(|i| h4.path_with_row_control([i & 1 != 0, i & 2 != 0], p, q, true))
            .collect::<TestResult<Vec<_>>>()?;
        let wrong_q: f64 = wrong_paths
            .iter()
            .map(|v| v.probability * v.cost * v.score[1])
            .sum();
        close(wrong_q, 0.0, 0.0);
        assert!((actual[1] - wrong_q).abs() > 0.01);
        assert!(wrong_paths
            .iter()
            .any(|v| v.first_source == 1 && v.second_row == 0));
        let wrong_single_visit: f64 = paths
            .iter()
            .map(|v| v.probability * v.cost * (f64::from(u8::from(v.actions[0])) - p))
            .sum();
        assert!((wrong_single_visit - actual[0]).abs() > 1e-3);
        // Enumerate all four deterministic shared-row exports independently.
        for export in 0..4 {
            let table = [export & 1 != 0, export & 2 != 0];
            let first = table[0];
            let state = h4.product(
                h4.geometry.identity,
                if first {
                    h4.minus
                } else {
                    h4.geometry.identity
                },
            );
            let second = table[h4.source(state)?];
            let path = h4.path([first, second], p, q)?;
            assert_eq!(path.actions[1], table[path.second_row]);
            let final_state = h4.product(
                state,
                if second {
                    h4.minus
                } else {
                    h4.geometry.identity
                },
            );
            assert_eq!(h4.source(final_state)?, path.final_source);
            assert_eq!(
                paths
                    .iter()
                    .find(|v| v.actions == path.actions)
                    .ok_or("enumerated export path")?
                    .cost,
                path.cost
            );
        }
    }
    Ok(())
}

#[test]
fn late_offered_symbol_direct_and_future_score_have_same_mean_normalization() -> TestResult {
    let p = 0.2;
    let logits = [0.0, logit(p)];
    let direct = cross_entropy(&logits, &[true, true], 1, 2)?;
    let mut future = 0.0;
    let mut full_score = 0.0;
    let mut expected_cost = direct.loss;
    for offered in [false, true] {
        let probability = if offered { p } else { 1.0 - p };
        // Category 1 means byte '1'. Actual input arrives after the learned
        // offer; the real ordinary lease acknowledgment determines next CE.
        let mut session = ObjectSession::new([1; 32], [2; 32], 7);
        session.observe_input(b'1', [0; 2], [0; 4])?;
        session.observe_input(b'x', [0; 2], [0; 4])?;
        let lease = session.acquire_occurrence(7, 0, 2)?;
        let offer = session.cache_offer(
            Symbol::Byte(if offered { b'1' } else { b'0' }),
            Action::AcquireA,
            Some(lease),
            None,
        )?;
        assert_eq!(
            offer
                .active()
                .ok_or("selected ordinary lease")?
                .lease()
                .reference(),
            lease.reference()
        );
        session.acknowledge(Symbol::Byte(b'1'), offer.id)?;
        assert!(session.key_pending());
        session.commit_key([0; 2], [0; 4])?;
        assert!(!session.key_pending());
        assert!(session.pending().is_none());
        let acknowledged = session.active().is_some_and(|active| active.acknowledged());
        assert_eq!(acknowledged, offered);
        if offered {
            assert_eq!(
                session
                    .active()
                    .ok_or("acknowledged lease")?
                    .lease()
                    .reference(),
                lease.reference()
            );
        } else {
            assert!(
                session.active().is_none(),
                "mismatch must cancel the selected lease"
            );
        }
        assert_eq!(session.occurrence(7, 2)?.byte, b'1');
        assert_eq!(
            session.publications_this_turn(),
            0,
            "ordinary leases do not publish arithmetic results"
        );
        let next_ce = if acknowledged { 0.5 } else { 2.0 };
        let score = f64::from(u8::from(offered)) - p;
        future += probability * (next_ce / 2.0) * score;
        full_score += probability * (direct.loss + next_ce / 2.0) * score;
        expected_cost += probability * next_ce / 2.0;
    }
    close(direct.direct[1], -0.4, 1e-14);
    close(future, -0.12, 1e-14);
    close(full_score, future, 1e-14);
    close(direct.direct[1] + future, -0.52, 1e-14);
    close(
        expected_cost,
        (-libm::log(p) + (1.0 - p) * 2.0 + p * 0.5) / 2.0,
        1e-14,
    );
    let objective = |x: f64| -> TrainingResult<f64> {
        let q = bernoulli_probability(x)?;
        Ok((-libm::log(q) + (1.0 - q) * 2.0 + q * 0.5) / 2.0)
    };
    close(
        (objective(logit(p) + 1e-5)? - objective(logit(p) - 1e-5)?) / 2e-5,
        -0.52,
        1e-7,
    );
    assert!(
        (direct.direct[1] - (-0.52)).abs() > 0.1,
        "missing late score must fail"
    );
    Ok(())
}

#[test]
fn serial_four_particle_loo_matches_explicit_and_rejects_mutated_parameters() -> TestResult {
    let parameters = [0.2, -0.3, 0.7];
    let losses = [0.2, 1.3, 0.7, 2.0];
    let scores = [
        [0.8, -0.2, 0.3],
        [-0.2, 0.6, -0.7],
        [0.4, 0.1, -0.1],
        [-0.1, -0.5, 0.9],
    ];
    let direct = [
        [0.1, -0.2, 0.1],
        [0.2, -0.3, 0.1],
        [-0.1, 0.0, 0.1],
        [0.3, -0.4, 0.1],
    ];
    let mut accumulator = SerialLoo::new(&parameters, 4)?;
    let mut changed = parameters;
    changed[0] += 1e-6;
    assert_eq!(
        accumulator.add_particle(&changed, losses[0], &scores[0], &direct[0]),
        Err(TrainingError::FrozenParametersChanged)
    );
    assert_eq!(accumulator.completed_particles(), 0);
    assert_eq!(
        accumulator.add_particle(&parameters, 0.1, &[f64::NAN, 0.0, 0.0], &direct[0]),
        Err(TrainingError::NonFinite)
    );
    assert_eq!(accumulator.completed_particles(), 0);
    for i in 0..4 {
        accumulator.add_particle(&parameters, losses[i], &scores[i], &direct[i])?;
    }
    assert_eq!(
        accumulator.add_particle(&parameters, 0.1, &scores[0], &direct[0]),
        Err(TrainingError::ParticleCount)
    );
    let gradient = accumulator.finish(&parameters)?;
    for r in 0..3 {
        let expected = (0..4)
            .map(|i| {
                let baseline = (0..4).filter(|&j| j != i).map(|j| losses[j]).sum::<f64>() / 3.0;
                (losses[i] - baseline) * scores[i][r] + direct[i][r]
            })
            .sum::<f64>()
            / 4.0;
        close(gradient[r], expected, 1e-14);
    }
    assert_eq!(
        SerialLoo::new(&parameters, 4)?.finish(&parameters),
        Err(TrainingError::ParticleCount)
    );
    let mut complete = SerialLoo::new(&parameters, 2)?;
    complete.add_particle(&parameters, 0.0, &[0.0; 3], &[0.0; 3])?;
    complete.add_particle(&parameters, 0.0, &[0.0; 3], &[0.0; 3])?;
    assert_eq!(
        complete.finish(&changed),
        Err(TrainingError::FrozenParametersChanged)
    );
    assert!(SerialLoo::new(&[f64::NAN], 4).is_err());
    assert!(SerialLoo::new(&[], 4).is_err());
    assert!(SerialLoo::new(&parameters, 1).is_err());
    Ok(())
}
