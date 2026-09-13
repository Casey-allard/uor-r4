//! Offline, per-invocation stochastic primitives. No optimizer or fit loop.
//! All scores refer to the actual visited event, never a globally sampled LUT.
use std::fmt;

/// Bound from the versioned first addressed-attention parameter layout.
pub const MAX_PARAMETERS: usize = 665_216;
/// Largest legal categorical head: all bytes and EOS.
pub const MAX_OUTCOMES: usize = 257;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainingError {
    Dimensions,
    NonFinite,
    EmptyLegalSupport,
    InvalidTarget,
    InvalidNormalizer,
    FrozenParametersChanged,
    ParticleCount,
}
impl fmt::Display for TrainingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for TrainingError {}
pub type TrainingResult<T> = Result<T, TrainingError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Gate,
    Head,
}
/// Each invocation owns a unique event key; repeated predict reuses its offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventKey {
    pub seed: u64,
    pub batch: u64,
    pub particle: u64,
    pub document: u64,
    pub event: u64,
    pub phase: u8,
    pub kind: EventKind,
    pub slot: u32,
}
/// Fixed-width domain-separated encoding; independent of host byte order.
/// This is an offline counter RNG, not an identity metric or serving primitive.
pub fn uniform01(key: EventKey) -> f64 {
    let mut hash = blake3::Hasher::new();
    hash.update(b"uor-r4.addressed-attention-event-rng/1");
    for value in [key.seed, key.batch, key.particle, key.document, key.event] {
        hash.update(&value.to_le_bytes());
    }
    hash.update(&[
        key.phase,
        match key.kind {
            EventKind::Gate => 0,
            EventKind::Head => 1,
        },
    ]);
    hash.update(&key.slot.to_le_bytes());
    let result = hash.finalize();
    let mut word = [0_u8; 8];
    word.copy_from_slice(&result.as_bytes()[..8]);
    (u64::from_le_bytes(word) >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
}
pub fn bernoulli_probability(logit: f64) -> TrainingResult<f64> {
    if !logit.is_finite() {
        return Err(TrainingError::NonFinite);
    }
    Ok(if logit >= 0.0 {
        1.0 / (1.0 + libm::exp(-logit))
    } else {
        let e = libm::exp(logit);
        e / (1.0 + e)
    })
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BernoulliDraw {
    pub outcome: bool,
    pub probability: f64,
    pub score: f64,
}
pub fn bernoulli(logit: f64, key: EventKey) -> TrainingResult<BernoulliDraw> {
    let probability = bernoulli_probability(logit)?;
    let outcome = uniform01(key) < probability;
    Ok(BernoulliDraw {
        outcome,
        probability,
        score: f64::from(u8::from(outcome)) - probability,
    })
}
fn categorical_weights(logits: &[f64], legal: &[bool]) -> TrainingResult<(Vec<f64>, f64, f64)> {
    if logits.is_empty() || logits.len() > MAX_OUTCOMES || logits.len() != legal.len() {
        return Err(TrainingError::Dimensions);
    }
    if logits.iter().any(|v| !v.is_finite()) {
        return Err(TrainingError::NonFinite);
    }
    let maximum = logits
        .iter()
        .zip(legal)
        .filter_map(|(&x, &allowed)| allowed.then_some(x))
        .reduce(f64::max)
        .ok_or(TrainingError::EmptyLegalSupport)?;
    let weights: Vec<_> = logits
        .iter()
        .zip(legal)
        .map(|(&x, &allowed)| if allowed { libm::exp(x - maximum) } else { 0.0 })
        .collect();
    let sum = weights.iter().sum::<f64>();
    if !sum.is_finite() || sum <= 0.0 {
        return Err(TrainingError::NonFinite);
    }
    Ok((weights, maximum, sum))
}
/// The caller supplies a causal legal mask. Invalid choices have exactly zero mass.
pub fn categorical_probabilities(logits: &[f64], legal: &[bool]) -> TrainingResult<Vec<f64>> {
    let (mut weights, _, sum) = categorical_weights(logits, legal)?;
    for w in &mut weights {
        *w /= sum;
    }
    Ok(weights)
}
#[derive(Debug, Clone, PartialEq)]
pub struct CategoricalDraw {
    pub outcome: usize,
    pub probabilities: Vec<f64>,
    pub score: Vec<f64>,
}
pub fn categorical(
    logits: &[f64],
    legal: &[bool],
    key: EventKey,
) -> TrainingResult<CategoricalDraw> {
    let probabilities = categorical_probabilities(logits, legal)?;
    let u = uniform01(key);
    let mut sum = 0.0;
    let mut selected = None;
    let mut last_positive = None;
    for (i, &p) in probabilities.iter().enumerate() {
        if p > 0.0 {
            last_positive = Some(i);
            sum += p;
            if u < sum {
                selected = Some(i);
                break;
            }
        }
    }
    // Absorb only floating summation residue into a positive-mass legal outcome.
    let outcome = selected
        .or(last_positive)
        .ok_or(TrainingError::EmptyLegalSupport)?;
    let score = probabilities
        .iter()
        .enumerate()
        .map(|(i, &p)| f64::from(u8::from(i == outcome)) - p)
        .collect();
    Ok(CategoricalDraw {
        outcome,
        probabilities,
        score,
    })
}
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalLoss {
    pub loss: f64,
    pub direct: Vec<f64>,
}
/// One target's contribution to the declared mean objective. The same fixed
/// normalizer must be used for all direct terms and the trajectory cost.
pub fn cross_entropy(
    logits: &[f64],
    legal: &[bool],
    target: usize,
    normalizer: usize,
) -> TrainingResult<ConditionalLoss> {
    if normalizer == 0 {
        return Err(TrainingError::InvalidNormalizer);
    }
    let (weights, maximum, sum) = categorical_weights(logits, legal)?;
    if target >= logits.len() || !legal[target] {
        return Err(TrainingError::InvalidTarget);
    }
    let loss = ((maximum - logits[target]) + libm::log(sum)) / normalizer as f64;
    if !loss.is_finite() || loss < 0.0 {
        return Err(TrainingError::NonFinite);
    }
    let direct = weights
        .iter()
        .enumerate()
        .map(|(i, &w)| (w / sum - f64::from(u8::from(i == target))) / normalizer as f64)
        .collect();
    Ok(ConditionalLoss { loss, direct })
}
fn parameter_digest(parameters: &[f64]) -> TrainingResult<blake3::Hash> {
    if parameters.is_empty() || parameters.len() > MAX_PARAMETERS {
        return Err(TrainingError::Dimensions);
    }
    if parameters.iter().any(|v| !v.is_finite()) {
        return Err(TrainingError::NonFinite);
    }
    let mut hash = blake3::Hasher::new();
    hash.update(b"uor-r4.addressed-attention-frozen-parameters/1");
    hash.update(&(parameters.len() as u64).to_le_bytes());
    for &v in parameters {
        hash.update(&v.to_bits().to_le_bytes());
    }
    Ok(hash.finalize())
}
/// Sufficient statistics for serial independent particles at identical frozen
/// parameters/data: A=sum S, B=sum L*S, C=sum direct, U=sum L.
/// No parameter update or optimizer configuration is provided by this type.
#[derive(Debug)]
pub struct SerialLoo {
    frozen: blake3::Hash,
    expected_particles: usize,
    completed_particles: usize,
    score_sum: Vec<f64>,
    weighted_score_sum: Vec<f64>,
    direct_sum: Vec<f64>,
    cost_sum: f64,
}
impl SerialLoo {
    pub fn new(parameters: &[f64], particles: usize) -> TrainingResult<Self> {
        if !(2..=4).contains(&particles) {
            return Err(TrainingError::ParticleCount);
        }
        let frozen = parameter_digest(parameters)?;
        Ok(Self {
            frozen,
            expected_particles: particles,
            completed_particles: 0,
            score_sum: vec![0.0; parameters.len()],
            weighted_score_sum: vec![0.0; parameters.len()],
            direct_sum: vec![0.0; parameters.len()],
            cost_sum: 0.0,
        })
    }
    fn validate_parameters(&self, parameters: &[f64]) -> TrainingResult<()> {
        if parameters.len() != self.score_sum.len() {
            return Err(TrainingError::Dimensions);
        }
        if parameter_digest(parameters)? != self.frozen {
            return Err(TrainingError::FrozenParametersChanged);
        }
        Ok(())
    }
    pub fn completed_particles(&self) -> usize {
        self.completed_particles
    }
    /// `path_score` already sums EVERY event score, including repeated row uses
    /// and offered-symbol events. `direct_mean_gradient` and `mean_loss` use the
    /// same normalization. The caller must replay a complete independent path.
    pub fn add_particle(
        &mut self,
        parameters: &[f64],
        mean_loss: f64,
        path_score: &[f64],
        direct_mean_gradient: &[f64],
    ) -> TrainingResult<()> {
        self.validate_parameters(parameters)?;
        if self.completed_particles >= self.expected_particles {
            return Err(TrainingError::ParticleCount);
        }
        if path_score.len() != self.score_sum.len()
            || direct_mean_gradient.len() != self.score_sum.len()
        {
            return Err(TrainingError::Dimensions);
        }
        if !mean_loss.is_finite() || mean_loss < 0.0 || !(self.cost_sum + mean_loss).is_finite() {
            return Err(TrainingError::NonFinite);
        }
        // Validate all arithmetic before mutation, so a rejected particle leaves
        // the accumulator unchanged and cannot create a partially charged row.
        for i in 0..self.score_sum.len() {
            let values = [
                self.score_sum[i] + path_score[i],
                self.weighted_score_sum[i] + mean_loss * path_score[i],
                self.direct_sum[i] + direct_mean_gradient[i],
            ];
            if values.iter().any(|x| !x.is_finite()) {
                return Err(TrainingError::NonFinite);
            }
        }
        self.cost_sum += mean_loss;
        for i in 0..self.score_sum.len() {
            self.score_sum[i] += path_score[i];
            self.weighted_score_sum[i] += mean_loss * path_score[i];
            self.direct_sum[i] += direct_mean_gradient[i];
        }
        self.completed_particles += 1;
        Ok(())
    }
    /// Consume the batch and reuse B's allocation for the final gradient.
    pub fn finish(mut self, parameters: &[f64]) -> TrainingResult<Vec<f64>> {
        self.validate_parameters(parameters)?;
        if self.completed_particles != self.expected_particles {
            return Err(TrainingError::ParticleCount);
        }
        let p = self.expected_particles as f64;
        for i in 0..self.weighted_score_sum.len() {
            let g = self.weighted_score_sum[i] / (p - 1.0)
                - self.cost_sum * self.score_sum[i] / (p * (p - 1.0))
                + self.direct_sum[i] / p;
            if !g.is_finite() {
                return Err(TrainingError::NonFinite);
            }
            self.weighted_score_sum[i] = g;
        }
        Ok(self.weighted_score_sum)
    }
}

#[cfg(test)]
#[path = "training_tests.rs"]
mod tests;
