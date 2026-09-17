//! Offline categorical search over shared parameters held fixed throughout each
//! complete hard trajectory. Probabilities and gradients are never served.
use super::*;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TiedFitConfig {
    pub indices: [usize; 2],
    pub seed: u64,
    pub batches: usize,
    pub batch_size: usize,
    pub step_size: f64,
    pub max_seconds: u64,
}

impl TiedFitConfig {
    pub(super) fn validate(self) -> Result<()> {
        let allowed = |i| (TRANSITION..QUERY).contains(&i) || (READ..PHASE).contains(&i);
        if self.indices[0] == self.indices[1]
            || !self.indices.into_iter().all(allowed)
            || self.seed == 0
            || !(1..=64).contains(&self.batches)
            || !(2..=128).contains(&self.batch_size)
            || self.batches.saturating_mul(self.batch_size) > 8192
            || !(1..=120).contains(&self.max_seconds)
            || !self.step_size.is_finite()
            || self.step_size <= 0.0
            || self.step_size > 100.0
        {
            return Err(CoreError::InvalidInput("tied categorical fit bounds"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(super) enum SearchMode {
    Learned,
    Fixed,
    Coordinate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchReport {
    pub mode: String,
    pub best_roots: [u16; 2],
    pub before_cost: f64,
    pub after_cost: f64,
    pub initial_probabilities: [Vec<f64>; 2],
    /// For Coordinate these remain the initial reference probabilities; that
    /// baseline draws one uniform coordinate, not a factorized distribution.
    pub final_probabilities: [Vec<f64>; 2],
    pub calls: usize,
    pub batches: usize,
    pub clipping_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiedFitReport {
    pub config: TiedFitConfig,
    pub before: Metrics,
    pub after: Metrics,
    pub search: SearchReport,
    pub data_digest: String,
    pub elapsed_ms: u128,
}

fn deadline(start: Instant, max_seconds: u64) -> Result<()> {
    if start.elapsed().as_secs() >= max_seconds {
        return Err(CoreError::InvalidInput(
            "tied fit time limit; no partial artifact",
        ));
    }
    Ok(())
}

fn softmax(logits: &[f64]) -> Result<Vec<f64>> {
    if logits.is_empty() || logits.iter().any(|v| !v.is_finite()) {
        return Err(CoreError::InvalidInput("nonfinite categorical logits"));
    }
    let maximum = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut result: Vec<_> = logits.iter().map(|&v| libm::exp(v - maximum)).collect();
    let total: f64 = result.iter().sum();
    if !total.is_finite() || total <= 0.0 {
        return Err(CoreError::InvalidInput("invalid categorical normalization"));
    }
    for p in &mut result {
        *p /= total;
    }
    Ok(result)
}

/// Independent minibatch draws permit the mean of all other costs as an
/// action-independent baseline. Dividing by B (not B-1) averages the B terms.
fn loo_gradient(
    probabilities: &[Vec<f64>; 2],
    samples: &[([u16; 2], f64)],
) -> Result<[Vec<f64>; 2]> {
    if samples.len() < 2 || samples.iter().any(|(_, c)| !c.is_finite()) {
        return Err(CoreError::InvalidInput("invalid categorical minibatch"));
    }
    let total: f64 = samples.iter().map(|(_, cost)| cost).sum();
    if !total.is_finite() {
        return Err(CoreError::InvalidInput("nonfinite minibatch cost sum"));
    }
    let n = samples.len() as f64;
    let mut gradient: [Vec<f64>; 2] = std::array::from_fn(|d| vec![0.0; probabilities[d].len()]);
    for &(pair, cost) in samples {
        let advantage = cost - (total - cost) / (n - 1.0);
        for d in 0..2 {
            if usize::from(pair[d]) >= probabilities[d].len() {
                return Err(CoreError::InvalidInput("categorical sample outside domain"));
            }
            for (j, value) in gradient[d].iter_mut().enumerate() {
                *value += advantage
                    * (f64::from(u8::from(j == usize::from(pair[d]))) - probabilities[d][j])
                    / n;
            }
        }
    }
    if gradient.iter().flatten().any(|v| !v.is_finite()) {
        return Err(CoreError::InvalidInput("nonfinite categorical gradient"));
    }
    Ok(gradient)
}

// SplitMix64: deterministic local sampling, separate from model initialization.
fn next(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_add(0x9e3779b97f4a7c15);
    let mut x = *seed;
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

fn draw(probabilities: &[f64], seed: &mut u64) -> u16 {
    let uniform = (next(seed) >> 11) as f64 / 9_007_199_254_740_992.0;
    let mut cumulative = 0.0;
    for (i, p) in probabilities.iter().enumerate() {
        cumulative += p;
        if uniform < cumulative {
            return i as u16;
        }
    }
    // The final positive category also absorbs floating-point summation error.
    (probabilities.len() - 1) as u16
}

fn uniform_root(seed: &mut u64) -> u16 {
    // Rejection avoids modulo bias in the matched coordinate baseline.
    let threshold = (ROOTS as u64).wrapping_neg() % ROOTS as u64;
    loop {
        let x = next(seed);
        if x >= threshold {
            return (x % ROOTS as u64) as u16;
        }
    }
}

pub(super) fn search(
    initial: [u16; 2],
    config: TiedFitConfig,
    mode: SearchMode,
    mut callback: impl FnMut([u16; 2]) -> Result<f64>,
) -> Result<SearchReport> {
    config.validate()?;
    if initial.iter().any(|&v| usize::from(v) >= ROOTS) {
        return Err(CoreError::InvalidInput("initial tied roots outside domain"));
    }
    let start = Instant::now();
    let mut calls = 0;
    let mut evaluate = |pair| {
        deadline(start, config.max_seconds)?;
        let cost = callback(pair)?;
        calls += 1;
        deadline(start, config.max_seconds)?;
        if !cost.is_finite() {
            return Err(CoreError::InvalidInput("nonfinite hard trajectory cost"));
        }
        Ok(cost)
    };
    let mut logits: [Vec<f64>; 2] = std::array::from_fn(|d| {
        let mut row = vec![libm::log(0.5 / (ROOTS - 1) as f64); ROOTS];
        row[usize::from(initial[d])] = libm::log(0.5);
        row
    });
    let initial_probabilities = [softmax(&logits[0])?, softmax(&logits[1])?];
    let mut probabilities = initial_probabilities.clone();
    let before_cost = evaluate(initial)?;
    let (mut best, mut best_cost) = (initial, before_cost);
    let mut rng = config.seed;
    let mut clipping_count = 0;
    for batch in 0..config.batches {
        deadline(start, config.max_seconds)?;
        let mut samples = Vec::with_capacity(config.batch_size);
        for sample in 0..config.batch_size {
            let pair = if matches!(mode, SearchMode::Coordinate) {
                let mut proposal = best;
                proposal[(batch * config.batch_size + sample) & 1] = uniform_root(&mut rng);
                proposal
            } else {
                [
                    draw(&probabilities[0], &mut rng),
                    draw(&probabilities[1], &mut rng),
                ]
            };
            let cost = evaluate(pair)?;
            if cost < best_cost {
                best = pair;
                best_cost = cost;
            }
            samples.push((pair, cost));
        }
        if matches!(mode, SearchMode::Learned) {
            let gradient = loo_gradient(&probabilities, &samples)?;
            for d in 0..2 {
                for (value, derivative) in logits[d].iter_mut().zip(&gradient[d]) {
                    *value -= config.step_size * derivative;
                }
                let center = logits[d].iter().sum::<f64>() / ROOTS as f64;
                if !center.is_finite() || logits[d].iter().any(|v| !v.is_finite()) {
                    return Err(CoreError::InvalidInput("nonfinite categorical update"));
                }
                for value in &mut logits[d] {
                    *value -= center;
                    if *value < -30.0 || *value > 30.0 {
                        clipping_count += 1;
                        *value = value.clamp(-30.0, 30.0);
                    }
                }
                probabilities[d] = softmax(&logits[d])?;
            }
        }
        deadline(start, config.max_seconds)?;
    }
    let after_cost = evaluate(best)?;
    if (after_cost - best_cost).abs() > 1e-12 {
        return Err(CoreError::InvalidInput(
            "tied best trajectory replay mismatch",
        ));
    }
    deadline(start, config.max_seconds)?;
    Ok(SearchReport {
        mode: match mode {
            SearchMode::Learned => "learned_factorized_categorical",
            SearchMode::Fixed => "fixed_initial_distribution_joint",
            SearchMode::Coordinate => "greedy_uniform_coordinate",
        }
        .into(),
        best_roots: best,
        before_cost,
        after_cost,
        initial_probabilities,
        final_probabilities: probabilities,
        calls,
        batches: config.batches,
        clipping_count,
    })
}

impl SharedCore {
    /// Sample one pair for each complete hard trajectory, learn the factorized
    /// distribution from total mean NLL, and export the best observed hard pair.
    /// All repeated uses of either selected parameter share its sampled value.
    pub fn fit_tied(
        &self,
        documents: &[Vec<u8>],
        config: TiedFitConfig,
    ) -> Result<(Self, TiedFitReport)> {
        if self.artifact.angular_tree.is_some() {
            return Err(CoreError::InvalidInput(
                "angular emission requires its own training provenance",
            ));
        }
        config.validate()?;
        training::validate_data(documents)?;
        let start = Instant::now();
        let mut model = self.clone();
        let initial = config.indices.map(|i| model.artifact.parameters[i]);
        let mut before = None;
        let mut after = None;
        let search = search(initial, config, SearchMode::Learned, |pair| {
            deadline(start, config.max_seconds)?;
            for d in 0..2 {
                model.artifact.parameters[config.indices[d]] = pair[d];
            }
            let metrics = model.evaluate(documents, Intervention::Full)?;
            deadline(start, config.max_seconds)?;
            if before.is_none() {
                before = Some(metrics.clone());
            }
            let cost = metrics.mean_nll;
            after = Some(metrics);
            Ok(cost)
        })?;
        let before = before.ok_or(CoreError::InvalidInput("missing tied initial metrics"))?;
        let after = after.ok_or(CoreError::InvalidInput("missing tied final metrics"))?;
        if before.mean_nll != search.before_cost || after.mean_nll != search.after_cost {
            return Err(CoreError::InvalidInput("tied report metrics mismatch"));
        }
        let mut hash = blake3::Hasher::new();
        hash.update(b"uor-r4.shared-core-tied-training/1");
        for document in documents {
            hash.update(&(document.len() as u64).to_le_bytes());
            hash.update(document);
        }
        let data_digest = format!("blake3:{}", hash.finalize());
        model.artifact.implementation = Self::implementation_digest();
        model.artifact.training_digest = Some(data_digest.clone());
        model.artifact.training_parent = Some(self.cid.clone());
        model.artifact.fit_config = None;
        model.artifact.tied_fit_config = Some(config);
        model.refresh_identity()?;
        deadline(start, config.max_seconds)?;
        Ok((
            model,
            TiedFitReport {
                config,
                before,
                after,
                search,
                data_digest,
                elapsed_ms: start.elapsed().as_millis(),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TiedFitConfig {
        TiedFitConfig {
            indices: [TRANSITION, READ],
            seed: 7341,
            batches: 2,
            batch_size: 4,
            step_size: 40.0,
            max_seconds: 1,
        }
    }

    #[test]
    fn stable_softmax_and_invalid_config() -> Result<()> {
        let p = softmax(&[1000.0, 1001.0])?;
        assert!((p.iter().sum::<f64>() - 1.0).abs() < 1e-14);
        assert!((p[1] - 0.7310585786300049).abs() < 1e-14);
        assert!(softmax(&[f64::NAN]).is_err());
        assert!(softmax(&[]).is_err());
        config().validate()?;
        for indices in [[TRANSITION, TRANSITION], [QUERY, READ], [READ, PHASE]] {
            assert!(TiedFitConfig {
                indices,
                ..config()
            }
            .validate()
            .is_err());
        }
        for step_size in [0.0, -1.0, 100.1, f64::INFINITY, f64::NAN] {
            assert!(TiedFitConfig {
                step_size,
                ..config()
            }
            .validate()
            .is_err());
        }
        for (batches, batch_size) in [(0, 4), (65, 4), (1, 1), (1, 129)] {
            assert!(TiedFitConfig {
                batches,
                batch_size,
                ..config()
            }
            .validate()
            .is_err());
        }
        for max_seconds in [0, 121] {
            assert!(TiedFitConfig {
                max_seconds,
                ..config()
            }
            .validate()
            .is_err());
        }
        assert!(TiedFitConfig {
            seed: 0,
            ..config()
        }
        .validate()
        .is_err());
        Ok(())
    }

    #[test]
    fn expected_leave_one_out_gradient_matches_nonseparable_finite_difference() -> Result<()> {
        let logits = [vec![0.3, -0.2], vec![-0.1, 0.4]];
        let p = [softmax(&logits[0])?, softmax(&logits[1])?];
        // Interaction changes both magnitude and sign; this is not additive.
        let costs = [[1.0, 4.0], [3.0, -2.0]];
        let mut expected = [[0.0; 2]; 2];
        for a in 0..2 {
            for b in 0..2 {
                for c in 0..2 {
                    for d in 0..2 {
                        let weight = p[0][a] * p[1][b] * p[0][c] * p[1][d];
                        let gradient = loo_gradient(
                            &p,
                            &[
                                ([a as u16, b as u16], costs[a][b]),
                                ([c as u16, d as u16], costs[c][d]),
                            ],
                        )?;
                        for side in 0..2 {
                            for j in 0..2 {
                                expected[side][j] += weight * gradient[side][j];
                            }
                        }
                    }
                }
            }
        }
        let objective = |l: &[Vec<f64>; 2]| -> Result<f64> {
            let q = [softmax(&l[0])?, softmax(&l[1])?];
            let mut cost = 0.0;
            for a in 0..2 {
                for b in 0..2 {
                    cost += q[0][a] * q[1][b] * costs[a][b];
                }
            }
            Ok(cost)
        };
        for side in 0..2 {
            for j in 0..2 {
                let mut plus = logits.clone();
                let mut minus = logits.clone();
                plus[side][j] += 1e-5;
                minus[side][j] -= 1e-5;
                let finite_difference = (objective(&plus)? - objective(&minus)?) / 2e-5;
                assert!((finite_difference - expected[side][j]).abs() < 1e-9);
            }
        }
        Ok(())
    }

    #[test]
    fn matched_search_calls_replay_best_and_reject_nonfinite_cost() -> Result<()> {
        let initial = [11, 71];
        for mode in [
            SearchMode::Learned,
            SearchMode::Fixed,
            SearchMode::Coordinate,
        ] {
            let mut calls = 0;
            let report = search(initial, config(), mode, |pair| {
                calls += 1;
                Ok(f64::from(pair[0]) + f64::from(pair[1]))
            })?;
            assert_eq!(calls, 10);
            assert_eq!(report.calls, calls);
            assert!(report.after_cost <= report.before_cost);
            assert_eq!(
                report.after_cost,
                f64::from(report.best_roots[0]) + f64::from(report.best_roots[1])
            );
            for row in &report.final_probabilities {
                assert!((row.iter().sum::<f64>() - 1.0).abs() < 1e-12);
            }
        }
        assert!(search(initial, config(), SearchMode::Learned, |_| Ok(f64::NAN)).is_err());
        let repeat = || {
            search(initial, config(), SearchMode::Learned, |pair| {
                Ok(f64::from(pair[0] ^ pair[1]))
            })
        };
        assert_eq!(
            serde_json::to_vec(&repeat()?).map_err(|e| CoreError::Host(e.to_string()))?,
            serde_json::to_vec(&repeat()?).map_err(|e| CoreError::Host(e.to_string()))?
        );
        Ok(())
    }
}
