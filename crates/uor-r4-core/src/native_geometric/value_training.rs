//! Offline Rust learning for the optional typed operand/action selector.
//! Targets label output bytes only. Operand identity and operation are latent;
//! all positive credit goes to naturally admitted complete-value alternatives.
use super::numeral::NUMERAL_CODEC;
use super::value_runtime::execute;
use super::value_types::*;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueExample {
    pub id: String,
    pub prompt: String,
    pub response: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueFitConfig {
    pub epochs: usize,
    pub learning_rate: f64,
    pub max_features: usize,
}
impl Default for ValueFitConfig {
    fn default() -> Self {
        Self {
            epochs: 24,
            learning_rate: 0.1,
            max_features: 65_536,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueFitReport {
    pub schema: String,
    pub examples: usize,
    pub eligible_examples: usize,
    pub numeric_targets: usize,
    pub reachable_numeric_targets: usize,
    pub no_write_targets: usize,
    pub learned_features: usize,
    pub dropped_features: usize,
    pub epochs: usize,
    pub selected_epoch: usize,
    pub fit_correct: usize,
    pub continuation_positions: usize,
    pub loss: f64,
    pub config: ValueFitConfig,
    pub artifact_cid: String,
}
struct Alternative {
    features: Vec<usize>,
    correct: bool,
}
struct Example {
    alternatives: Vec<Alternative>,
    baseline_correct: bool,
}

fn target_integer(text: &str) -> Option<(i64, usize)> {
    let bytes = text.as_bytes();
    let mut end = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let start = end;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == start
        || bytes
            .get(end)
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
        || (bytes.get(end) == Some(&b'.')
            && bytes.get(end + 1).is_some_and(|b| !b.is_ascii_whitespace()))
    {
        return None;
    }
    text[..end].parse().ok().map(|n| (n, end))
}
impl ValueModel {
    pub(super) fn validate(&self) -> Result<()> {
        if ![VALUE_SCHEMA, LEXEME_VALUE_SCHEMA].contains(&self.schema.as_str())
            || self.codec != NUMERAL_CODEC
            || self.capacity != VALUES
            || self.rows.len() > 262_144
            || self.rows.windows(2).any(|p| p[0].feature >= p[1].feature)
            || self.rows.iter().any(|r| {
                r.feature.kind
                    > if self.schema == LEXEME_VALUE_SCHEMA {
                        16
                    } else {
                        15
                    }
                    || !(-1_000_000..=1_000_000).contains(&r.weight)
            })
            || !(0..=1_000_000).contains(&self.continuation_score)
            || self
                .training
                .iter()
                .map(|r| &r.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.training.len()
            || (!self.training.is_empty()
                && (!(1..=256).contains(&self.fit_config[0])
                    || !(0.0001..=1.0).contains(&f64::from_bits(self.fit_config[1]))
                    || !(1..=262_144).contains(&self.fit_config[2])
                    || self.fit_config[3] == 0
                    || self.fit_config[3] > self.fit_config[0]))
        {
            return Err(Error("invalid typed-value operator artifact".into()));
        }
        Ok(())
    }
}
impl Model {
    pub fn value_operator_version(&self) -> Option<&str> {
        self.values.as_ref().map(|v| v.schema.as_str())
    }
    pub fn value_training(&self) -> &[DocumentReceipt] {
        self.values.as_ref().map_or(&[], |v| v.training.as_slice())
    }
    pub fn fit_values(
        &self,
        documents: &[ValueExample],
        config: ValueFitConfig,
    ) -> Result<(Model, ValueFitReport)> {
        self.fit_values_impl(documents, config, false)
    }
    /// Learn the versioned whole-lexeme relation while preserving `/1` fitting.
    pub fn fit_values_with_lexeme_cues(
        &self,
        documents: &[ValueExample],
        config: ValueFitConfig,
    ) -> Result<(Model, ValueFitReport)> {
        self.fit_values_impl(documents, config, true)
    }
    fn fit_values_impl(
        &self,
        documents: &[ValueExample],
        config: ValueFitConfig,
        lexeme_cues: bool,
    ) -> Result<(Model, ValueFitReport)> {
        if self.values.is_some()
            || documents.is_empty()
            || documents.len() > 4096
            || !(1..=256).contains(&config.epochs)
            || !config.learning_rate.is_finite()
            || !(0.0001..=1.0).contains(&config.learning_rate)
            || !(1..=262_144).contains(&config.max_features)
            || documents
                .iter()
                .map(|d| d.prompt.len() + d.response.len())
                .sum::<usize>()
                > 16 * 1024 * 1024
        {
            return Err(Error(
                "invalid typed-value fit configuration/source or already fitted component".into(),
            ));
        }
        let mut model = self.clone();
        model.values = Some(ValueModel {
            schema: if lexeme_cues {
                LEXEME_VALUE_SCHEMA
            } else {
                VALUE_SCHEMA
            }
            .into(),
            codec: NUMERAL_CODEC.into(),
            capacity: VALUES,
            rows: Vec::new(),
            continuation_score: 0,
            fit_config: [0; 4],
            training: Vec::new(),
        });
        model.refresh_identity()?;
        let mut registry = BTreeMap::<ValueFeature, usize>::new();
        let mut weights = Vec::<f64>::new();
        let mut examples = Vec::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut prompts = BTreeMap::new();
        let mut numeric = 0;
        let mut reachable = 0;
        let mut no_write = 0;
        let mut dropped = 0;
        let mut continuation_positions = 0;
        for document in documents {
            if document.id.trim().is_empty()
                || !ids.insert(&document.id)
                || prompts
                    .insert(&document.prompt, &document.response)
                    .is_some_and(|prior| prior != &document.response)
            {
                return Err(Error(
                    "typed-value source IDs or prompt targets conflict".into(),
                ));
            }
            let combined = Document {
                id: document.id.clone(),
                text: serde_json::to_string(&(&document.prompt, &document.response))
                    .map_err(|error| Error(error.to_string()))?,
            };
            let receipt = super::training::receipt(&combined);
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
                    known.id == document.id
                        || known.text_cid == receipt.text_cid
                        || known.text_cid == whole.text_cid
                })
            {
                return Err(Error(
                    "typed-value fitting source overlaps previous model training".into(),
                ));
            }
            receipts.push(receipt);
            let mut session = model.session(Control::Full)?;
            session.observe(&model, BOS)?;
            for token in model.encode(&document.prompt)? {
                session.observe(&model, token)?;
            }
            session.begin_response(&model)?;
            let target = target_integer(&document.response);
            numeric += usize::from(target.is_some());
            no_write += usize::from(target.is_none());
            let state = session
                .values
                .as_ref()
                .ok_or_else(|| Error("typed state unavailable".into()))?;
            let mut alternatives = Vec::new();
            let mut any_correct = false;
            for index in 0..272 {
                let Some((action, a, b)) = state.proposal(index) else {
                    continue;
                };
                let mut work = ValueWork::default();
                let Some(value) = execute(action, a.value, b.value, &mut work) else {
                    continue;
                };
                let correct = target.is_some_and(|t| {
                    t.0 == value
                        && super::numeral::Numeral::from_zphi(
                            crate::prime_route_attention::ZPhi::new(value, 0),
                        )
                        .is_some_and(|n| {
                            n.len as usize == t.1
                                && n.as_tokens()
                                    .iter()
                                    .zip(document.response.as_bytes())
                                    .all(|(token, byte)| *token == u32::from(*byte) + 2)
                        })
                });
                any_correct |= correct;
                let (features, len) =
                    state.features(&model, action, a, b, Control::Full, &mut work);
                let mut indices = Vec::with_capacity(len);
                for &feature in &features[..len] {
                    let index = if let Some(&index) = registry.get(&feature) {
                        Some(index)
                    } else if weights.len() < config.max_features {
                        let index = weights.len();
                        registry.insert(feature, index);
                        weights.push(if feature.kind == 0 { -2.0 } else { 0.0 });
                        Some(index)
                    } else {
                        dropped += 1;
                        None
                    };
                    if let Some(index) = index {
                        indices.push(index);
                    }
                }
                alternatives.push(Alternative {
                    features: indices,
                    correct,
                });
            }
            if target.is_some() && !any_correct {
                continue;
            }
            if any_correct {
                reachable += 1;
                continuation_positions += target.map_or(0, |(_, n)| n.saturating_sub(1));
            }
            examples.push(Example {
                alternatives,
                baseline_correct: target.is_none(),
            });
        }
        if examples.is_empty() {
            return Err(Error(
                "typed-value source has no admitted learning targets".into(),
            ));
        }
        let mut best_weights = weights.clone();
        let mut best_correct = 0;
        let mut best_loss = f64::INFINITY;
        let mut best_epoch = 0;
        for epoch in 0..config.epochs {
            for example in &examples {
                let logits: Vec<f64> = example
                    .alternatives
                    .iter()
                    .map(|a| a.features.iter().map(|&i| weights[i]).sum())
                    .collect();
                let maximum = logits.iter().copied().fold(0.0, f64::max);
                let baseline = (-maximum).exp();
                let exp: Vec<f64> = logits.iter().map(|&n| (n - maximum).exp()).collect();
                let total = baseline + exp.iter().sum::<f64>();
                let positive = if example.baseline_correct {
                    baseline
                } else {
                    0.0
                } + example
                    .alternatives
                    .iter()
                    .zip(&exp)
                    .filter(|(a, _)| a.correct)
                    .map(|(_, p)| *p)
                    .sum::<f64>();
                for (alternative, p) in example.alternatives.iter().zip(exp) {
                    let gradient = (if alternative.correct {
                        p / positive
                    } else {
                        0.0
                    }) - p / total;
                    // Normalize update magnitude for a bounded feature list;
                    // repeated cues retain their declared repeated contributions.
                    let delta = config.learning_rate * gradient
                        / (alternative.features.len().max(1) as f64).sqrt();
                    for &index in &alternative.features {
                        weights[index] = (weights[index] + delta).clamp(-3900.0, 3900.0);
                    }
                }
            }
            let (correct, loss) = measure(&examples, &weights);
            if correct > best_correct || (correct == best_correct && loss < best_loss) {
                best_correct = correct;
                best_loss = loss;
                best_epoch = epoch + 1;
                best_weights.clone_from(&weights);
            }
        }
        // Preserve actual SGD source order in the artifact; this order can
        // change learned weights even when the source set is identical.
        let mut continuation = 0.0_f64;
        for _ in 0..config.epochs {
            for _ in 0..continuation_positions {
                continuation += config.learning_rate / (1.0 + continuation.exp());
            }
        }
        let head = model
            .values
            .as_mut()
            .ok_or_else(|| Error("typed head unavailable".into()))?;
        head.rows = registry
            .into_iter()
            .map(|(feature, index)| ValueRow {
                feature,
                weight: (best_weights[index] * 256.0).round() as i32,
            })
            .collect();
        head.continuation_score = (continuation * 256.0).round() as i32;
        head.training = receipts;
        head.fit_config = [
            config.epochs as u64,
            config.learning_rate.to_bits(),
            config.max_features as u64,
            best_epoch as u64,
        ];
        let quantized: Vec<f64> = best_weights
            .iter()
            .map(|w| (w * 256.0).round() / 256.0)
            .collect();
        let (best_correct, best_loss) = measure(&examples, &quantized);
        model.refresh_identity()?;
        model.validate()?;
        let report = ValueFitReport {
            schema: if lexeme_cues {
                LEXEME_VALUE_SCHEMA
            } else {
                VALUE_SCHEMA
            }
            .into(),
            examples: documents.len(),
            eligible_examples: examples.len(),
            numeric_targets: numeric,
            reachable_numeric_targets: reachable,
            no_write_targets: no_write,
            learned_features: head_len(&model),
            dropped_features: dropped,
            epochs: config.epochs,
            selected_epoch: best_epoch,
            fit_correct: best_correct,
            continuation_positions,
            loss: best_loss,
            config,
            artifact_cid: model.artifact_cid.clone(),
        };
        Ok((model, report))
    }
}
fn head_len(model: &Model) -> usize {
    model.values.as_ref().map_or(0, |h| h.rows.len())
}
fn measure(examples: &[Example], weights: &[f64]) -> (usize, f64) {
    let mut correct = 0;
    let mut loss = 0.0;
    for e in examples {
        let mut best = 0.0;
        let mut right = e.baseline_correct;
        let logits: Vec<f64> = e
            .alternatives
            .iter()
            .map(|a| a.features.iter().map(|&i| weights[i]).sum())
            .collect();
        for (a, &logit) in e.alternatives.iter().zip(&logits) {
            if logit > best {
                best = logit;
                right = a.correct;
            }
        }
        correct += usize::from(right);
        let baseline = (-best).exp();
        let mut total = baseline;
        let mut positive = if e.baseline_correct { baseline } else { 0.0 };
        for (a, logit) in e.alternatives.iter().zip(logits) {
            let p = (logit - best).exp();
            total += p;
            if a.correct {
                positive += p;
            }
        }
        loss += (total / positive).ln();
    }
    (correct, loss / examples.len() as f64)
}
