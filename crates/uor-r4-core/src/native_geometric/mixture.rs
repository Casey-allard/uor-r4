//! Construction-only discriminative fitting of a small geometric readout.
//! This module may use floating point. Its exported artifact contains seven
//! bounded integer gates and supported last-prime query gates; the kernel
//! applies them with shifts/additions. Development labels never enter fitting.

use super::*;
use std::collections::{BTreeMap, BTreeSet};

const GROUPS: usize = 7;
const MIN_QUERY_SUPPORT: usize = 8;
const FIXED: &str = "fixed_v1";
const LEARNED: &str = "learned_mixture_v1";

/// Order: lexical; ordered H4/current paired root; orientation/heatmap;
/// H4-zeta interaction; exact radial; fixed-zeta channels; paired window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct QueryGate {
    pub prime: u32,
    pub weights: [u8; GROUPS],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Readout {
    pub version: String,
    pub global: [u8; GROUPS],
    pub queries: Vec<QueryGate>,
    pub fit_positions: usize,
    pub epochs: usize,
}
impl Default for Readout {
    fn default() -> Self {
        Self {
            version: FIXED.into(),
            global: [8; GROUPS],
            queries: Vec::new(),
            fit_positions: 0,
            epochs: 0,
        }
    }
}
impl Readout {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        if self.global.iter().any(|&weight| weight > 16)
            || self.queries.len() > 4096
            || self
                .queries
                .windows(2)
                .any(|pair| pair[0].prime >= pair[1].prime)
            || self.queries.iter().any(|query| {
                query.weights.iter().any(|&weight| weight > 16)
                    || model
                        .geometry
                        .tokens
                        .binary_search_by_key(&query.prime, |token| token.prime)
                        .is_err()
            })
        {
            return Err(Error(
                "readout mixture gate bounds/address order invalid".into(),
            ));
        }
        match self.version.as_str() {
            FIXED
                if self.global == [8; GROUPS]
                    && self.queries.is_empty()
                    && self.fit_positions == 0
                    && self.epochs == 0
                    && model.readout_training.is_empty() => {}
            LEARNED
                if self.fit_positions > 0
                    && self.fit_positions <= 16384
                    && (1..=64).contains(&self.epochs)
                    && !model.readout_training.is_empty() => {}
            _ => return Err(Error("readout version/fit provenance mismatch".into())),
        }
        let mut ids = BTreeSet::new();
        for receipt in &model.readout_training {
            if receipt.id.trim().is_empty()
                || !ids.insert(&receipt.id)
                || model
                    .construction
                    .iter()
                    .any(|known| known.id == receipt.id || known.text_cid == receipt.text_cid)
            {
                return Err(Error(
                    "readout fit receipts overlap count construction or repeat IDs".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadoutFitConfig {
    pub max_positions: usize,
    pub epochs: usize,
    pub max_queries: usize,
}
impl Default for ReadoutFitConfig {
    fn default() -> Self {
        Self {
            max_positions: 4096,
            epochs: 8,
            max_queries: 256,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadoutFitReport {
    pub group_names: Vec<String>,
    pub sampling: String,
    pub documents_sampled: usize,
    pub observed_context_positions: usize,
    pub positions: usize,
    pub target_in_shortlist: usize,
    pub fit_correct_before: usize,
    pub fit_correct_after: usize,
    /// Candidate-conditional fitting objective, only for reachable targets.
    pub candidate_cross_entropy_before: f64,
    pub candidate_cross_entropy_after: f64,
    pub global_weights_eighths: [u8; GROUPS],
    pub query_weights_eighths: BTreeMap<u32, [u8; GROUPS]>,
    pub query_gate_count: usize,
    pub epochs: usize,
}

struct FitCandidate {
    token: u32,
    prior: i32,
    groups: [i64; GROUPS],
}
struct Example {
    query: u32,
    target: u32,
    candidates: Vec<FitCandidate>,
}

impl Model {
    pub fn readout_version(&self) -> &str {
        &self.readout.version
    }
    pub fn readout_training(&self) -> &[DocumentReceipt] {
        &self.readout_training
    }

    /// Fits a new artifact, preserving this fixed-readout baseline. The fit
    /// corpus must be disjoint from count construction by ID and exact bytes.
    /// It becomes training provenance and is refused by subsequent evaluate.
    /// Missing targets are counted, never inserted into candidate lists.
    /// Target quotas are divided evenly across documents and sampled uniformly
    /// through each document. Intervening tokens are observed so long-context
    /// state remains real; max_positions bounds fitted targets, not input bytes.
    pub fn fit_readout(
        &self,
        documents: &[Document],
        config: ReadoutFitConfig,
    ) -> Result<(Model, ReadoutFitReport)> {
        if self.readout.version != FIXED
            || self.training.target_positions == 0
            || self.memory_read.is_some()
        {
            return Err(Error(
                "readout fitting requires a fitted fixed_v1 baseline".into(),
            ));
        }
        if documents.is_empty()
            || !(1..=16384).contains(&config.max_positions)
            || !(1..=64).contains(&config.epochs)
            || config.max_queries > 4096
            || config
                .max_positions
                .saturating_mul(self.config.candidate_limit)
                > 1_048_576
        {
            return Err(Error(
                "readout fit exceeds position/epoch/query/candidate-memory bounds".into(),
            ));
        }
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        for document in documents {
            let receipt = super::training::receipt(document);
            if receipt.id.trim().is_empty()
                || !ids.insert(receipt.id.clone())
                || self
                    .construction
                    .iter()
                    .any(|known| known.id == receipt.id || known.text_cid == receipt.text_cid)
            {
                return Err(Error(
                    "readout fit corpus overlaps count construction or repeats IDs".into(),
                ));
            }
            receipts.push(receipt);
        }
        receipts.sort_by(|a, b| a.id.cmp(&b.id));
        let mut examples = Vec::new();
        let mut documents_sampled = 0;
        let mut observed_context_positions = 0;
        for (document_index, document) in documents.iter().enumerate() {
            let mut session = self.session(Control::Full)?;
            session.observe(self, BOS)?;
            let mut tokens = self.encode(&document.text)?;
            tokens.push(EOS);
            let token_count = tokens.len();
            let quota = (config.max_positions / documents.len()
                + usize::from(document_index < config.max_positions % documents.len()))
            .min(token_count);
            if quota == 0 {
                continue;
            }
            documents_sampled += 1;
            let mut sampled = 0;
            for (position, target) in tokens.into_iter().enumerate() {
                let selected = if quota == 1 {
                    token_count / 2
                } else {
                    ((sampled as u128 * (token_count - 1) as u128) / (quota - 1) as u128) as usize
                };
                if sampled >= quota || position != selected {
                    session.observe(self, target)?;
                    observed_context_positions += 1;
                    continue;
                }
                session.predict(self)?;
                let features = session.features(self);
                let rows: Vec<_> = features
                    .iter()
                    .filter_map(|feature| {
                        self.rows
                            .binary_search_by_key(feature, |row| row.feature)
                            .ok()
                    })
                    .collect();
                let candidates = session
                    .candidates()
                    .iter()
                    .map(|candidate| {
                        let prior = self.prior_scores[candidate.token as usize];
                        let mut groups = [0_i64; GROUPS];
                        for &index in &rows {
                            let row = &self.rows[index];
                            let conditional = row
                                .scores
                                .binary_search_by_key(&candidate.token, |item| item.token)
                                .map(|index| row.scores[index].score)
                                .unwrap_or(row.default_score);
                            groups[row.feature.group()] +=
                                (i64::from(conditional) - i64::from(prior)) >> row.feature.shift();
                        }
                        FitCandidate {
                            token: candidate.token,
                            prior,
                            groups,
                        }
                    })
                    .collect();
                examples.push(Example {
                    query: features[0].value as u32,
                    target,
                    candidates,
                });
                sampled += 1;
                session.observe(self, target)?;
                observed_context_positions += 1;
            }
        }
        let target_in_shortlist = examples
            .iter()
            .filter(|example| {
                example
                    .candidates
                    .iter()
                    .any(|candidate| candidate.token == example.target)
            })
            .count();
        if target_in_shortlist == 0 {
            return Err(Error(
                "readout fit has no reachable labels in the baseline shortlist".into(),
            ));
        }
        let mut global = [1.0_f64; GROUPS];
        for _ in 0..config.epochs {
            for example in &examples {
                update(example, &mut global, &[1.0; GROUPS], 0.002);
            }
        }
        let mut frequencies = BTreeMap::<u32, usize>::new();
        for example in &examples {
            if example
                .candidates
                .iter()
                .any(|candidate| candidate.token == example.target)
            {
                *frequencies.entry(example.query).or_default() += 1;
            }
        }
        let mut frequent: Vec<_> = frequencies
            .into_iter()
            .filter(|(_, count)| *count >= MIN_QUERY_SUPPORT)
            .collect();
        frequent.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        frequent.truncate(config.max_queries);
        let mut queries: BTreeMap<_, _> = frequent
            .into_iter()
            .map(|(prime, _)| (prime, global))
            .collect();
        for _ in 0..config.epochs {
            for example in &examples {
                if let Some(weights) = queries.get_mut(&example.query) {
                    update(example, weights, &global, 0.05);
                }
            }
        }
        let readout = Readout {
            version: LEARNED.into(),
            global: quantize(global),
            queries: queries
                .into_iter()
                .map(|(prime, weights)| QueryGate {
                    prime,
                    weights: quantize(weights),
                })
                .collect(),
            fit_positions: examples.len(),
            epochs: config.epochs,
        };
        let (fit_correct_before, candidate_cross_entropy_before) =
            metrics(&examples, &Readout::default());
        let (fit_correct_after, candidate_cross_entropy_after) = metrics(&examples, &readout);
        let report = ReadoutFitReport {
            group_names: [
                "lexical",
                "ordered_h4_current_paired_root",
                "signed_orientation_heatmap",
                "h4_zeta_interaction",
                "exact_additive_radial",
                "fixed_zeta_channels",
                "exact_paired_window",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            sampling: "equal_document_quota_uniform_positions_including_ends_v1".into(),
            documents_sampled,
            observed_context_positions,
            positions: examples.len(),
            target_in_shortlist,
            fit_correct_before,
            fit_correct_after,
            candidate_cross_entropy_before,
            candidate_cross_entropy_after,
            global_weights_eighths: readout.global,
            query_weights_eighths: readout
                .queries
                .iter()
                .map(|query| (query.prime, query.weights))
                .collect(),
            query_gate_count: readout.queries.len(),
            epochs: config.epochs,
        };
        let mut learned = self.clone();
        learned.readout = readout;
        learned.readout_training = receipts;
        learned.refresh_identity()?;
        learned.validate()?;
        Ok((learned, report))
    }
}

fn quantize(weights: [f64; GROUPS]) -> [u8; GROUPS] {
    weights.map(|value| libm::round(value.clamp(0.0, 2.0) * 8.0) as u8)
}
fn logit(candidate: &FitCandidate, weights: &[f64; GROUPS]) -> f64 {
    (f64::from(candidate.prior)
        + candidate
            .groups
            .iter()
            .zip(weights)
            .map(|(&value, &weight)| value as f64 * weight)
            .sum::<f64>())
        / SCORE_SCALE
}
fn update(example: &Example, weights: &mut [f64; GROUPS], anchor: &[f64; GROUPS], shrinkage: f64) {
    let Some(target) = example
        .candidates
        .iter()
        .find(|candidate| candidate.token == example.target)
    else {
        return;
    };
    let maximum = example
        .candidates
        .iter()
        .map(|candidate| logit(candidate, weights))
        .fold(f64::NEG_INFINITY, f64::max);
    let mut denominator = 0.0;
    let mut expected = [0.0; GROUPS];
    for candidate in &example.candidates {
        let mass = libm::exp(logit(candidate, weights) - maximum);
        denominator += mass;
        for (value, &group) in expected.iter_mut().zip(&candidate.groups) {
            *value += mass * group as f64 / SCORE_SCALE;
        }
    }
    for group in 0..GROUPS {
        let gradient = expected[group] / denominator - target.groups[group] as f64 / SCORE_SCALE
            + shrinkage * (weights[group] - anchor[group]);
        weights[group] = (weights[group] - 0.025 * gradient.clamp(-8.0, 8.0)).clamp(0.0, 2.0);
    }
}
fn metrics(examples: &[Example], readout: &Readout) -> (usize, f64) {
    let mut correct = 0;
    let mut loss = 0.0;
    let mut reachable = 0;
    for example in examples {
        let weights = readout
            .queries
            .binary_search_by_key(&example.query, |query| query.prime)
            .map(|index| readout.queries[index].weights)
            .unwrap_or(readout.global);
        // Integer rounding matches the deployed group-gating operation.
        let values: Vec<_> = example
            .candidates
            .iter()
            .map(|candidate| {
                let score = i64::from(candidate.prior)
                    + candidate
                        .groups
                        .iter()
                        .zip(weights)
                        .map(|(&value, weight)| (value * i64::from(weight)) >> 3)
                        .sum::<i64>();
                (candidate.token, score)
            })
            .collect();
        let Some(&(best, maximum)) = values
            .iter()
            .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
        else {
            continue;
        };
        correct += usize::from(best == example.target);
        if let Some((_, target_score)) = values.iter().find(|(token, _)| *token == example.target) {
            let denominator: f64 = values
                .iter()
                .map(|(_, score)| libm::exp((*score - maximum) as f64 / SCORE_SCALE))
                .sum();
            loss += libm::log(denominator) + (maximum - target_score) as f64 / SCORE_SCALE;
            reachable += 1;
        }
    }
    (correct, loss / reachable as f64)
}
