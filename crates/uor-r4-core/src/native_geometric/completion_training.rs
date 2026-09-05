//! Offline sparse completion learning from actual frozen typed-value rollouts.
//! Only individual next-byte/EOS associations enter learned rows and postings.
use super::completion_runtime;
use super::completion_types::*;
use super::value_types::LEXEME_VALUE_SCHEMA;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueCompletionFitConfig {
    pub epochs: usize,
    pub learning_rate: f64,
    pub max_positions: usize,
}
impl Default for ValueCompletionFitConfig {
    fn default() -> Self {
        Self {
            epochs: 24,
            learning_rate: 0.1,
            max_positions: COMPLETION_POSITIONS,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueCompletionFitReport {
    pub schema: String,
    pub baseline_artifact: String,
    pub artifact_cid: String,
    pub examples: usize,
    pub matched_numeric: usize,
    pub skipped_no_write: usize,
    pub upstream_failures: usize,
    pub position_limit_skips: usize,
    pub overlong_responses: usize,
    pub positions: usize,
    pub target_in_candidates: usize,
    pub eligible_positions: usize,
    pub learned_rows: usize,
    pub learned_associations: usize,
    pub dropped_row_events: usize,
    pub dropped_association_events: usize,
    pub fit_correct: usize,
    pub fit_loss: f64,
    pub selected_epoch: usize,
    pub config: ValueCompletionFitConfig,
}

pub(super) struct Frame<const N: usize = COMPLETION_FEATURES> {
    pub(super) features: [Feature; N],
    pub(super) len: usize,
    pub(super) target: u32,
    pub(super) baseline: u32,
}
struct Alternative {
    token: u32,
    weights: Vec<usize>,
    correct: bool,
}
struct Example {
    alternatives: Vec<Alternative>,
    baseline_correct: bool,
}

fn valid_token(token: u32) -> bool {
    token == EOS || (2..LEXICAL_BASE).contains(&token)
}

/// Host-only exact numeric-prefix label. It never chooses an inference value
/// or operand; a shorter emitted number cannot be repaired as a suffix.
fn canonical_prefix_len(response: &str) -> Option<usize> {
    let bytes = response.as_bytes();
    let mut end = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let first_digit = end;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == first_digit
        || bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        || (bytes.get(end) == Some(&b'.')
            && bytes
                .get(end + 1)
                .is_some_and(|byte| !byte.is_ascii_whitespace()))
    {
        return None;
    }
    let value = response[..end].parse::<i64>().ok()?;
    let numeral =
        super::numeral::Numeral::from_zphi(crate::prime_route_attention::ZPhi::new(value, 0))?;
    (usize::from(numeral.len) == end
        && numeral
            .as_tokens()
            .iter()
            .zip(bytes)
            .all(|(token, byte)| *token == u32::from(*byte) + 2))
    .then_some(end)
}

impl CompletionModel {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let associations = self.rows.iter().map(|row| row.scores.len()).sum::<usize>();
        if self.schema != COMPLETION_SCHEMA
            || !model
                .values
                .as_ref()
                .is_some_and(|values| values.schema == LEXEME_VALUE_SCHEMA)
            || self.rows.len() > COMPLETION_ROWS
            || associations > COMPLETION_ASSOCIATIONS
            || self.global_postings.len() > COMPLETION_CANDIDATES
            || self
                .global_postings
                .iter()
                .any(|&token| !valid_token(token))
            || self.global_postings.iter().collect::<BTreeSet<_>>().len()
                != self.global_postings.len()
            || self
                .rows
                .windows(2)
                .any(|pair| pair[0].feature >= pair[1].feature)
            || self.rows.iter().any(|row| {
                row.feature.kind >= COMPLETION_FEATURES as u8
                    || row.default_score != 0
                    || row.postings.len() > COMPLETION_POSTINGS
                    || row.postings.iter().collect::<BTreeSet<_>>().len() != row.postings.len()
                    || row
                        .scores
                        .windows(2)
                        .any(|pair| pair[0].token >= pair[1].token)
                    || row.scores.iter().any(|entry| {
                        !valid_token(entry.token)
                            || !(-1_000_000..=1_000_000).contains(&entry.score)
                    })
                    || row.postings.iter().any(|token| {
                        row.scores
                            .binary_search_by_key(token, |entry| entry.token)
                            .is_err()
                    })
            })
            || self.fit_positions > COMPLETION_POSITIONS
            || self
                .training
                .iter()
                .any(|receipt| receipt.id.trim().is_empty())
            || self
                .training
                .iter()
                .map(|receipt| &receipt.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.training.len()
            || (!self.training.is_empty()
                && (self.fit_positions == 0
                    || !(1..=64).contains(&self.fit_config[0])
                    || !(0.0001..=1.0).contains(&f64::from_bits(self.fit_config[1]))
                    || !(1..=COMPLETION_POSITIONS as u64).contains(&self.fit_config[2])
                    || self.fit_config[3] == 0
                    || self.fit_config[3] > self.fit_config[0]))
        {
            return Err(Error("invalid sparse value-completion artifact".into()));
        }
        let mut baseline = model.clone();
        baseline.completion = None;
        baseline.response_entry = None;
        baseline.refresh_identity()?;
        if baseline.artifact_cid != self.baseline_artifact {
            return Err(Error(
                "completion artifact differs from its frozen typed baseline".into(),
            ));
        }
        Ok(())
    }
}

impl Model {
    pub fn value_completion_version(&self) -> Option<&str> {
        self.completion.as_ref().map(|head| head.schema.as_str())
    }

    pub fn value_completion_training(&self) -> &[DocumentReceipt] {
        self.completion
            .as_ref()
            .map_or(&[], |head| head.training.as_slice())
    }

    pub fn fit_value_completion(
        &self,
        documents: &[ValueExample],
        config: ValueCompletionFitConfig,
    ) -> Result<(Model, ValueCompletionFitReport)> {
        let source_bytes = documents.iter().try_fold(0_usize, |sum, document| {
            sum.checked_add(document.prompt.len())?
                .checked_add(document.response.len())
        });
        if self.completion.is_some()
            || !self
                .values
                .as_ref()
                .is_some_and(|values| values.schema == LEXEME_VALUE_SCHEMA)
            || documents.is_empty()
            || documents.len() > 4096
            || source_bytes.is_none_or(|bytes| bytes > 16 * 1024 * 1024)
            || !(1..=64).contains(&config.epochs)
            || !config.learning_rate.is_finite()
            || !(0.0001..=1.0).contains(&config.learning_rate)
            || !(1..=COMPLETION_POSITIONS).contains(&config.max_positions)
        {
            return Err(Error(
                "invalid completion source, configuration or frozen value baseline".into(),
            ));
        }
        let mut model = self.clone();
        model.completion = Some(CompletionModel {
            schema: COMPLETION_SCHEMA.into(),
            baseline_artifact: self.artifact_cid.clone(),
            rows: Vec::new(),
            global_postings: Vec::new(),
            fit_config: [0; 4],
            fit_positions: 0,
            training: Vec::new(),
        });
        model.refresh_identity()?;
        let mut frames = Vec::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut prompts = BTreeMap::new();
        let mut matched_numeric = 0;
        let mut skipped_no_write = 0;
        let mut upstream_failures = 0;
        let mut position_limit_skips = 0;
        let mut overlong_responses = 0;
        for document in documents {
            if document.id.trim().is_empty()
                || !ids.insert(&document.id)
                || prompts
                    .insert(&document.prompt, &document.response)
                    .is_some_and(|known| known != &document.response)
            {
                return Err(Error(
                    "completion source IDs or raw prompt targets conflict".into(),
                ));
            }
            let receipt = super::training::receipt(&Document {
                id: document.id.clone(),
                text: serde_json::to_string(&(&document.prompt, &document.response))
                    .map_err(|error| Error(error.to_string()))?,
            });
            let whole = super::training::receipt(&Document {
                id: document.id.clone(),
                text: format!("{}{}", document.prompt, document.response),
            });
            if self
                .construction
                .iter()
                .chain(self.readout_training())
                .chain(self.memory_read_training())
                .any(|known| {
                    known.id == receipt.id
                        || known.text_cid == receipt.text_cid
                        || known.text_cid == whole.text_cid
                })
            {
                return Err(Error(
                    "completion source overlaps ordinary model construction".into(),
                ));
            }
            receipts.push(receipt);
            let target_prefix_len = canonical_prefix_len(&document.response);
            let mut session = model.session(Control::Full)?;
            session.observe(&model, BOS)?;
            for token in model.encode(&document.prompt)? {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)?;
            let mut offset = 0;
            let mut upstream_ok = false;
            let mut no_write = false;
            for _ in 0..20 {
                let prediction = session.predict(&model)?;
                if session.value_decision().is_none() {
                    no_write = offset == 0;
                    break;
                }
                let expected = document
                    .response
                    .as_bytes()
                    .get(offset)
                    .map(|byte| u32::from(*byte) + 2);
                if expected != Some(prediction.token) {
                    break;
                }
                session.observe(&model, prediction.token)?;
                offset += 1;
                if session
                    .completion
                    .as_ref()
                    .is_some_and(|state| state.active)
                {
                    upstream_ok = target_prefix_len == Some(offset);
                    break;
                }
            }
            if !upstream_ok {
                let numeric_start = document
                    .response
                    .as_bytes()
                    .first()
                    .is_some_and(|byte| byte.is_ascii_digit() || matches!(*byte, b'+' | b'-'));
                if no_write && !numeric_start {
                    skipped_no_write += 1;
                } else {
                    upstream_failures += 1;
                }
                continue;
            }
            matched_numeric += 1;
            let remaining = &document.response.as_bytes()[offset..];
            if remaining.len().saturating_add(1) > usize::from(COMPLETION_STEPS) {
                overlong_responses += 1;
            }
            for target in remaining
                .iter()
                .map(|byte| u32::from(*byte) + 2)
                .chain(std::iter::once(EOS))
                .take(usize::from(COMPLETION_STEPS))
            {
                if frames.len() == config.max_positions {
                    position_limit_skips += 1;
                    break;
                }
                let baseline = session.predict(&model)?.token;
                let state = session.completion.as_ref().ok_or_else(|| {
                    Error("completion state missing during actual rollout".into())
                })?;
                let values = session
                    .values
                    .as_ref()
                    .ok_or_else(|| Error("typed state missing during completion rollout".into()))?;
                if !state.active {
                    return Err(Error(
                        "completion frame ended before its declared step bound".into(),
                    ));
                }
                let (features, len) = state.features(
                    &model,
                    values,
                    Control::Full,
                    &mut CompletionWork::default(),
                );
                frames.push(Frame {
                    features,
                    len,
                    target,
                    baseline,
                });
                session.observe(&model, target)?;
            }
        }
        if frames.is_empty() {
            return Err(Error(
                "completion source has no matched emitted numeral and suffix positions".into(),
            ));
        }
        let fit = fit_sparse_frames(&frames, config, COMPLETION_ROWS, COMPLETION_ASSOCIATIONS, 0)?;
        let head = model
            .completion
            .as_mut()
            .ok_or_else(|| Error("completion component unavailable".into()))?;
        head.rows = fit.rows;
        head.global_postings = fit.global_postings;
        let selected_epoch = fit.selected_epoch;
        head.fit_positions = frames.len();
        head.fit_config = [
            config.epochs as u64,
            config.learning_rate.to_bits(),
            config.max_positions as u64,
            selected_epoch as u64,
        ];
        head.training = receipts;
        let learned_rows = head.rows.len();
        model.refresh_identity()?;
        model.validate()?;
        let report = ValueCompletionFitReport {
            schema: COMPLETION_SCHEMA.into(),
            baseline_artifact: self.artifact_cid.clone(),
            artifact_cid: model.artifact_cid.clone(),
            examples: documents.len(),
            matched_numeric,
            skipped_no_write,
            upstream_failures,
            position_limit_skips,
            overlong_responses,
            positions: frames.len(),
            target_in_candidates: fit.target_in_candidates,
            eligible_positions: fit.eligible_positions,
            learned_rows,
            learned_associations: fit.learned_associations,
            dropped_row_events: fit.dropped_row_events,
            dropped_association_events: fit.dropped_association_events,
            fit_correct: fit.correct,
            fit_loss: fit.loss,
            selected_epoch,
            config,
        };
        Ok((model, report))
    }
}

/// Shared host sparse optimizer. A baseline ID absent from the token domain
/// makes Base ineligible when learning a state-creating action.
pub(super) struct SparseFit {
    pub(super) rows: Vec<ScoreRow>,
    pub(super) global_postings: Vec<u32>,
    pub(super) target_in_candidates: usize,
    pub(super) eligible_positions: usize,
    pub(super) learned_associations: usize,
    pub(super) dropped_row_events: usize,
    pub(super) dropped_association_events: usize,
    pub(super) correct: usize,
    pub(super) loss: f64,
    pub(super) selected_epoch: usize,
}

pub(super) fn fit_sparse_frames<const N: usize>(
    frames: &[Frame<N>],
    config: ValueCompletionFitConfig,
    max_rows: usize,
    max_associations: usize,
    bias_kind: u8,
) -> Result<SparseFit> {
    let mut counts = BTreeMap::<Feature, BTreeMap<u32, u64>>::new();
    let mut global = BTreeMap::<u32, u64>::new();
    let mut association_count = 0;
    let mut dropped_row_events = 0;
    let mut dropped_association_events = 0;
    for frame in frames {
        *global.entry(frame.target).or_default() += 1;
        for &feature in &frame.features[..frame.len] {
            if !counts.contains_key(&feature) && counts.len() == max_rows {
                dropped_row_events += 1;
                continue;
            }
            let row = counts.entry(feature).or_default();
            if !row.contains_key(&frame.target) {
                if association_count == max_associations {
                    dropped_association_events += 1;
                    continue;
                }
                association_count += 1;
            }
            *row.entry(frame.target).or_default() += 1;
        }
    }
    let mut registry = BTreeMap::<(Feature, u32), usize>::new();
    let mut weights = Vec::new();
    let rows = counts
        .into_iter()
        .map(|(feature, counts)| {
            let postings = top_counts(&counts, COMPLETION_POSTINGS);
            let scores = counts
                .keys()
                .map(|&token| {
                    registry.insert((feature, token), weights.len());
                    let weight = if feature.kind == bias_kind { -2.0 } else { 0.0 };
                    weights.push(weight);
                    TokenScore {
                        token,
                        score: (weight * 256.0) as i32,
                    }
                })
                .collect();
            ScoreRow {
                feature,
                default_score: 0,
                scores,
                postings,
            }
        })
        .collect();
    let mut rows: Vec<ScoreRow> = rows;
    let global_postings = top_counts(&global, COMPLETION_CANDIDATES);
    let mut examples = Vec::new();
    let mut target_in_candidates = 0;
    for frame in frames {
        let (tokens, len, _, _) = completion_runtime::candidate_rows_bounded::<N>(
            &rows,
            &global_postings,
            &frame.features[..frame.len],
            &mut CompletionWork::default(),
        );
        let target_present = tokens[..len].contains(&frame.target);
        target_in_candidates += usize::from(target_present);
        if !target_present && frame.baseline != frame.target {
            continue;
        }
        let alternatives = tokens[..len]
            .iter()
            .map(|&token| Alternative {
                token,
                correct: token == frame.target,
                weights: frame.features[..frame.len]
                    .iter()
                    .filter_map(|&feature| registry.get(&(feature, token)).copied())
                    .collect(),
            })
            .collect();
        examples.push(Example {
            alternatives,
            baseline_correct: frame.baseline == frame.target,
        });
    }
    if examples.is_empty() {
        return Err(Error(
            "completion postings admit no fitting target or correct Base action".into(),
        ));
    }
    let mut best_weights = weights.clone();
    let mut best_correct = 0;
    let mut best_loss = f64::INFINITY;
    let mut selected_epoch = 0;
    for epoch in 0..config.epochs {
        for example in &examples {
            update(example, &mut weights, config.learning_rate)?;
        }
        let quantized: Vec<f64> = weights
            .iter()
            .map(|weight| (weight * 256.0).round() / 256.0)
            .collect();
        let (correct, loss) = measure(&examples, &quantized);
        if !loss.is_finite() {
            return Err(Error(
                "completion fitting produced a nonfinite objective".into(),
            ));
        }
        if correct > best_correct || (correct == best_correct && loss < best_loss) {
            best_correct = correct;
            best_loss = loss;
            selected_epoch = epoch + 1;
            best_weights = quantized;
        }
    }
    for row in &mut rows {
        for entry in &mut row.scores {
            let index = registry
                .get(&(row.feature, entry.token))
                .ok_or_else(|| Error("completion export association missing".into()))?;
            entry.score = (best_weights[*index] * 256.0).round() as i32;
        }
    }
    Ok(SparseFit {
        rows,
        global_postings,
        target_in_candidates,
        eligible_positions: examples.len(),
        learned_associations: weights.len(),
        dropped_row_events,
        dropped_association_events,
        correct: best_correct,
        loss: best_loss,
        selected_epoch,
    })
}

pub(super) fn top_counts(counts: &BTreeMap<u32, u64>, cap: usize) -> Vec<u32> {
    let mut ranked: Vec<_> = counts
        .iter()
        .map(|(&token, &count)| (token, count))
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(cap);
    ranked.into_iter().map(|(token, _)| token).collect()
}

fn logits(example: &Example, weights: &[f64]) -> Vec<f64> {
    example
        .alternatives
        .iter()
        .map(|alternative| {
            alternative
                .weights
                .iter()
                .map(|&index| weights[index])
                .sum()
        })
        .collect()
}

fn update(example: &Example, weights: &mut [f64], rate: f64) -> Result<()> {
    let scores = logits(example, weights);
    let maximum = scores.iter().copied().fold(0.0, f64::max);
    let total = (-maximum).exp()
        + scores
            .iter()
            .map(|score| (score - maximum).exp())
            .sum::<f64>();
    let positive_max = example
        .alternatives
        .iter()
        .zip(&scores)
        .filter(|(alternative, _)| alternative.correct)
        .map(|(_, score)| *score)
        .fold(
            if example.baseline_correct {
                0.0
            } else {
                f64::NEG_INFINITY
            },
            f64::max,
        );
    let positive = (if example.baseline_correct {
        (-positive_max).exp()
    } else {
        0.0
    }) + example
        .alternatives
        .iter()
        .zip(&scores)
        .filter(|(alternative, _)| alternative.correct)
        .map(|(_, score)| (score - positive_max).exp())
        .sum::<f64>();
    for (alternative, score) in example.alternatives.iter().zip(scores) {
        let desired = if alternative.correct {
            (score - positive_max).exp() / positive
        } else {
            0.0
        };
        let gradient = desired - (score - maximum).exp() / total;
        let delta = rate * gradient / (alternative.weights.len().max(1) as f64).sqrt();
        if !delta.is_finite() {
            return Err(Error("completion gradient is nonfinite".into()));
        }
        for &index in &alternative.weights {
            weights[index] = (weights[index] + delta).clamp(-3900.0, 3900.0);
        }
    }
    Ok(())
}

fn measure(examples: &[Example], weights: &[f64]) -> (usize, f64) {
    let mut correct = 0;
    let mut loss = 0.0;
    for example in examples {
        let scores = logits(example, weights);
        let mut best = 0.0;
        let mut selected = None;
        let mut right = example.baseline_correct;
        for (alternative, &score) in example.alternatives.iter().zip(&scores) {
            if score > best
                || (score == best
                    && score > 0.0
                    && selected.is_some_and(|token| alternative.token < token))
            {
                best = score;
                selected = Some(alternative.token);
                right = alternative.correct;
            }
        }
        correct += usize::from(right);
        let positive_max = example
            .alternatives
            .iter()
            .zip(&scores)
            .filter(|(alternative, _)| alternative.correct)
            .map(|(_, score)| *score)
            .fold(
                if example.baseline_correct {
                    0.0
                } else {
                    f64::NEG_INFINITY
                },
                f64::max,
            );
        let total = (-best).exp() + scores.iter().map(|score| (score - best).exp()).sum::<f64>();
        let positive = (if example.baseline_correct {
            (-positive_max).exp()
        } else {
            0.0
        }) + example
            .alternatives
            .iter()
            .zip(&scores)
            .filter(|(alternative, _)| alternative.correct)
            .map(|(_, score)| (score - positive_max).exp())
            .sum::<f64>();
        loss += best + total.ln() - positive_max - positive.ln();
    }
    (correct, loss / examples.len() as f64)
}
