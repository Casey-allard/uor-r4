//! Host-only fitting of a sparse categorical pointer/read operator.
//! Targets supervise copied token identity; no grammar or answer vocabulary
//! participates in the model or in memory candidate admission.
use super::memory_types::*;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

const ABSENT: usize = usize::MAX;
struct Alternative {
    token: u32,
    constant: i32,
    features: Option<[usize; MEMORY_FEATURE_COUNT]>,
}
struct Example {
    target: u32,
    alternatives: Vec<Alternative>,
    groups: Vec<Vec<usize>>,
}

pub(super) fn compile_cue_aliases(model: &Model) -> Result<CueAliases> {
    if model.lexical_pieces.len() > model.config.max_lexical_pieces
        || model.vocabulary_size() != LEXICAL_BASE as usize + model.lexical_pieces.len()
    {
        return Err(Error("cue alias vocabulary shape is invalid".into()));
    }
    let mut representatives: Vec<u32> = (0..model.vocabulary_size() as u32).collect();
    let mut classes = BTreeMap::<String, Vec<(u32, bool)>>::new();
    // Reuse the canonical lexical-run definition: case-sensitive Unicode
    // alphanumeric/underscore words, with leading Unicode whitespace owned
    // by the exact output piece. Punctuation, whitespace-only pieces, special
    // tokens and raw-byte fallback tokens keep their own identities.
    for (index, bytes) in model.lexical_pieces.iter().enumerate() {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| Error("cue alias lexical piece is not UTF-8".into()))?;
        let word = text.trim_start_matches(char::is_whitespace);
        if word.is_empty() || !word.chars().all(|c| c.is_alphanumeric() || c == '_') {
            continue;
        }
        classes
            .entry(word.into())
            .or_default()
            .push((LEXICAL_BASE + index as u32, text.len() == word.len()));
    }
    for members in classes.values() {
        // Prefer the exact bare-word lexical prime. If the vocabulary has no
        // bare form, its first deterministic existing member represents the
        // equivalence class; no synthetic semantic hash or new prime is used.
        let representative = members
            .iter()
            .find(|(_, bare)| *bare)
            .unwrap_or(&members[0])
            .0;
        for &(token, _) in members {
            representatives[token as usize] = representative;
        }
    }
    Ok(CueAliases {
        schema: CUE_SCHEMA.into(),
        representatives,
    })
}

impl MemoryModel {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.config.validate(model.vocabulary_size())?;
        let cue_schema_valid = match (&self.cue_aliases, self.schema.as_str()) {
            (None, LEGACY_MEMORY_SCHEMA) => true,
            (Some(aliases), MEMORY_SCHEMA) => *aliases == compile_cue_aliases(model)?,
            _ => false,
        };
        if !cue_schema_valid
            || self.baseline_artifact.is_empty()
            || self.source_shift
                != self
                    .config
                    .source_offsets
                    .next_power_of_two()
                    .trailing_zeros() as u8
            || self.posting_shift
                != self
                    .config
                    .postings_per_address
                    .next_power_of_two()
                    .trailing_zeros() as u8
            || self.training.is_empty()
            || self.fit_positions == 0
            || self.fit_positions > self.config.max_positions
            || self.rows.len() > self.config.max_features
            || self
                .rows
                .windows(2)
                .any(|pair| pair[0].feature >= pair[1].feature)
            || self.rows.iter().any(|row| {
                row.feature.kind >= MEMORY_FEATURE_COUNT as u8
                    || !(-8192..=8192).contains(&row.score)
            })
        {
            return Err(Error(
                "memory-read operator shape, scores or configuration invalid".into(),
            ));
        }
        let mut ids = BTreeSet::new();
        for receipt in &self.training {
            if receipt.id.trim().is_empty()
                || !ids.insert(&receipt.id)
                || model
                    .construction
                    .iter()
                    .any(|known| known.id == receipt.id || known.text_cid == receipt.text_cid)
                || model
                    .readout_training
                    .iter()
                    .any(|known| known.id == receipt.id && known != receipt)
            {
                return Err(Error("memory-read training provenance overlaps count data or changes a readout-fit ID".into()));
            }
        }
        Ok(())
    }
}

impl Model {
    pub fn memory_read_version(&self) -> Option<&str> {
        self.memory_read
            .as_ref()
            .map(|memory| memory.schema.as_str())
    }
    pub fn memory_read_training(&self) -> &[DocumentReceipt] {
        self.memory_read
            .as_ref()
            .map(|memory| memory.training.as_slice())
            .unwrap_or(&[])
    }
    pub fn memory_read_config(&self) -> Option<&MemoryReadFitConfig> {
        self.memory_read.as_ref().map(|memory| &memory.config)
    }
    /// Effective cue identity comes from the bound operator schema/map,
    /// including artifacts fitted before word cues became an explicit option.
    pub fn memory_cue_identity(&self) -> Option<&str> {
        self.memory_read.as_ref().map(|memory| {
            memory
                .cue_aliases
                .as_ref()
                .map(|aliases| aliases.schema.as_str())
                .unwrap_or(EXACT_CUE_SCHEMA)
        })
    }
    /// Declared session backing buffers and fixed structure, before allocation.
    /// Excludes allocator bookkeeping; live buffer capacities remain in state().
    pub fn session_storage_bytes(&self) -> usize {
        let mut bytes = std::mem::size_of::<Session>()
            + self.artifact_cid.len()
            + self.config.context_tokens * std::mem::size_of::<u32>()
            + self.config.candidate_limit * std::mem::size_of::<Candidate>();
        if let Some(memory) = &self.memory_read {
            bytes += self.config.context_tokens * std::mem::size_of::<MemoryEntry>()
                + (self.vocabulary_size() << memory.source_shift << memory.posting_shift)
                    * std::mem::size_of::<MemoryReference>()
                + memory.config.candidate_limit * std::mem::size_of::<MemoryCandidate>();
        }
        bytes
    }

    /// Train an optional query-relative read head while preserving this model.
    /// Reusing its separate readout-fit corpus is explicitly permitted; count
    /// construction and later development labels remain separate. Only actual
    /// admitted memory values and base-model candidates enter the objective.
    pub fn fit_memory_read(
        &self,
        documents: &[Document],
        config: MemoryReadFitConfig,
    ) -> Result<(Model, MemoryReadFitReport)> {
        self.fit_memory_read_impl(documents, config, false)
    }

    /// Explicit experimental cue equivalence. Exact output and geometric token
    /// identities are preserved; only memory read/write keys use the alias map.
    pub fn fit_memory_read_with_word_cues(
        &self,
        documents: &[Document],
        config: MemoryReadFitConfig,
    ) -> Result<(Model, MemoryReadFitReport)> {
        self.fit_memory_read_impl(documents, config, true)
    }

    fn fit_memory_read_impl(
        &self,
        documents: &[Document],
        config: MemoryReadFitConfig,
        word_cues: bool,
    ) -> Result<(Model, MemoryReadFitReport)> {
        config.validate(self.vocabulary_size())?;
        if self.memory_read.is_some() || self.training.target_positions == 0 || documents.is_empty()
        {
            return Err(Error("memory-read fitting needs a fitted baseline without a memory-read head and nonempty fit documents".into()));
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
                || self
                    .readout_training
                    .iter()
                    .any(|known| known.id == receipt.id && known != &receipt)
            {
                return Err(Error("memory fit documents overlap count construction, repeat IDs or change readout-fit IDs".into()));
            }
            receipts.push(receipt);
        }
        receipts.sort_by(|a, b| a.id.cmp(&b.id));
        let mut memory = MemoryModel {
            schema: if word_cues {
                MEMORY_SCHEMA
            } else {
                LEGACY_MEMORY_SCHEMA
            }
            .into(),
            baseline_artifact: self.artifact_cid.clone(),
            cue_aliases: if word_cues {
                Some(compile_cue_aliases(self)?)
            } else {
                None
            },
            config,
            source_shift: config.source_offsets.next_power_of_two().trailing_zeros() as u8,
            posting_shift: config
                .postings_per_address
                .next_power_of_two()
                .trailing_zeros() as u8,
            training: receipts,
            rows: vec![MemoryWeight {
                feature: MemoryFeature { kind: 0, value: 0 },
                score: -1024,
            }],
            fit_positions: 0,
        };
        let mut addresses = BTreeMap::from([(MemoryFeature { kind: 0, value: 0 }, 0_usize)]);
        let mut examples = Vec::new();
        let mut observed = 0;
        let mut candidate_positions = 0;
        let mut target_in_memory = 0;
        let mut dropped = 0;
        let mut memory_bytes = 0;
        let mut sampled_documents = 0;
        let mut tail_positions = 0;
        for (document_index, document) in documents.iter().enumerate() {
            let mut session = self.session(Control::Full)?;
            let mut state = MemoryState::new(self, &memory);
            let view = state.state();
            memory_bytes =
                view.ring_storage_bytes + view.index_storage_bytes + view.candidate_storage_bytes;
            let mut work = Work::default();
            session.observe(self, BOS)?;
            state.observe(self, &memory, BOS, &mut work);
            let mut tokens = self.encode(&document.text)?;
            tokens.push(EOS);
            let length = tokens.len();
            let quota = (config.max_positions / documents.len()
                + usize::from(document_index < config.max_positions % documents.len()))
            .min(length);
            if quota == 0 {
                continue;
            }
            sampled_documents += 1;
            tail_positions += quota.min(8);
            let mut sampled = 0;
            for (position, target) in tokens.into_iter().enumerate() {
                // Generic end-of-document supervision ensures the final
                // continuation is represented without inspecting its label or
                // parsing any query grammar. Remaining targets span the body.
                let tail = quota.min(8);
                let body_quota = quota - tail;
                let body_length = length - tail;
                let selected = if sampled >= body_quota {
                    body_length + sampled - body_quota
                } else {
                    ((sampled as u128 * body_length as u128) / body_quota as u128) as usize
                };
                if sampled < quota && position == selected {
                    session.predict(self)?;
                    state.collect(self, &memory, Control::Full, &mut work);
                    target_in_memory += usize::from(
                        state
                            .candidates
                            .iter()
                            .any(|candidate| candidate.token == target),
                    );
                    candidate_positions += state.candidates.len();
                    let mut alternatives: Vec<_> = session
                        .candidates()
                        .iter()
                        .map(|candidate| Alternative {
                            token: candidate.token,
                            constant: candidate.score as i32,
                            features: None,
                        })
                        .collect();
                    for candidate in &state.candidates {
                        let mut features = [ABSENT; MEMORY_FEATURE_COUNT];
                        for (index, feature) in candidate.features.iter().enumerate() {
                            let next = addresses.len();
                            if let Some(&address) = addresses.get(feature) {
                                features[index] = address;
                            } else if next < config.max_features {
                                addresses.insert(*feature, next);
                                features[index] = next;
                            } else {
                                dropped += 1;
                            }
                        }
                        alternatives.push(Alternative {
                            token: candidate.token,
                            constant: self.prior_scores[candidate.token as usize],
                            features: Some(features),
                        });
                    }
                    let mut groups = BTreeMap::<u32, Vec<usize>>::new();
                    for (index, alternative) in alternatives.iter().enumerate() {
                        groups.entry(alternative.token).or_default().push(index);
                    }
                    examples.push(Example {
                        target,
                        alternatives,
                        groups: groups.into_values().collect(),
                    });
                    sampled += 1;
                }
                session.observe(self, target)?;
                state.observe(self, &memory, target, &mut work);
                observed += 1;
            }
        }
        let reachable = examples
            .iter()
            .filter(|example| {
                example
                    .alternatives
                    .iter()
                    .any(|alternative| alternative.token == example.target)
            })
            .count();
        if target_in_memory == 0 || reachable == 0 {
            return Err(Error(
                "memory fit has no target values in its bounded memory candidates".into(),
            ));
        }
        let mut weights = vec![0.0; addresses.len()];
        weights[0] = -4.0;
        let (before_correct, before_loss) = metrics(&examples, &weights);
        let (pointer_before_correct, pointer_before_loss) = pointer_metrics(&examples, &weights);
        let mut gradient = vec![0.0; weights.len()];
        let mut marks = vec![0_usize; weights.len()];
        let mut touched = Vec::with_capacity(config.candidate_limit * MEMORY_FEATURE_COUNT);
        let mut route_scores =
            Vec::with_capacity(config.candidate_limit + self.config.candidate_limit);
        let mut stamp = 0;
        // First learn the copy operation independently of the fixed base
        // scores. Otherwise a strong base model can extinguish every pointer
        // gradient before query/source discrimination is learned. This uses
        // only already-admitted memory routes and only reachable fit targets.
        for _ in 0..config.epochs {
            for example in &examples {
                stamp += 1;
                route_scores.clear();
                route_scores.extend(example.alternatives.iter().map(|alternative| {
                    if alternative.features.is_some() {
                        score(alternative, &weights)
                    } else {
                        f64::NEG_INFINITY
                    }
                }));
                if !example.alternatives.iter().any(|alternative| {
                    alternative.token == example.target && alternative.features.is_some()
                }) {
                    continue;
                }
                let maximum = route_scores
                    .iter()
                    .copied()
                    .fold(f64::NEG_INFINITY, f64::max);
                let denominator: f64 = route_scores
                    .iter()
                    .map(|score| libm::exp(*score - maximum))
                    .sum();
                let target_maximum = example
                    .alternatives
                    .iter()
                    .zip(&route_scores)
                    .filter(|(alternative, _)| alternative.token == example.target)
                    .map(|(_, score)| *score)
                    .fold(f64::NEG_INFINITY, f64::max);
                let target_denominator: f64 = example
                    .alternatives
                    .iter()
                    .zip(&route_scores)
                    .filter(|(alternative, _)| alternative.token == example.target)
                    .map(|(_, score)| libm::exp(*score - target_maximum))
                    .sum();
                touched.clear();
                // Marginalize latent source routes to the same target token.
                for (alternative, score) in example.alternatives.iter().zip(&route_scores) {
                    let mass = libm::exp(*score - maximum);
                    let coefficient = mass / denominator
                        - if alternative.token == example.target {
                            libm::exp(*score - target_maximum) / target_denominator
                        } else {
                            0.0
                        };
                    if let Some(features) = &alternative.features {
                        for &feature in features {
                            if feature == ABSENT {
                                continue;
                            }
                            if marks[feature] != stamp {
                                marks[feature] = stamp;
                                gradient[feature] = 0.0;
                                touched.push(feature);
                            }
                            gradient[feature] += coefficient;
                        }
                    }
                }
                for &feature in &touched {
                    let anchor = if feature == 0 { -4.0 } else { 0.0 };
                    weights[feature] = (weights[feature]
                        - 0.1 * (gradient[feature] + 0.0001 * (weights[feature] - anchor)))
                        .clamp(-16.0, 16.0);
                }
            }
        }
        // Calibrate the pointer's additive bias against the actual deployed
        // max-route objective on these same fit examples, then refine that
        // objective. No development labels enter either operation.
        calibrate_bias(&examples, &mut weights);
        let calibrated_bias = weights[0];
        let mut winners = Vec::with_capacity(config.candidate_limit + self.config.candidate_limit);
        let mut best_weights = weights.clone();
        let mut best_loss = metrics(&examples, &weights).1;
        for _ in 0..config.epochs {
            for example in &examples {
                stamp += 1;
                winner_scores(example, &weights, &mut winners);
                let Some(&(target_index, _)) = winners
                    .iter()
                    .find(|&&(index, _)| example.alternatives[index].token == example.target)
                else {
                    continue;
                };
                let maximum = winners
                    .iter()
                    .map(|(_, score)| *score)
                    .fold(f64::NEG_INFINITY, f64::max);
                let denominator: f64 = winners
                    .iter()
                    .map(|(_, score)| libm::exp(*score - maximum))
                    .sum();
                touched.clear();
                for &(index, score) in &winners {
                    let coefficient =
                        libm::exp(score - maximum) / denominator - f64::from(index == target_index);
                    if let Some(features) = &example.alternatives[index].features {
                        for &feature in features {
                            if feature == ABSENT {
                                continue;
                            }
                            if marks[feature] != stamp {
                                marks[feature] = stamp;
                                gradient[feature] = 0.0;
                                touched.push(feature);
                            }
                            gradient[feature] += coefficient;
                        }
                    }
                }
                for &feature in &touched {
                    let anchor = if feature == 0 { calibrated_bias } else { 0.0 };
                    weights[feature] = (weights[feature]
                        - 0.05 * (gradient[feature] + 0.0001 * (weights[feature] - anchor)))
                        .clamp(-16.0, 16.0);
                }
            }
            let loss = metrics(&examples, &weights).1;
            if loss < best_loss {
                best_loss = loss;
                best_weights.clone_from(&weights);
            }
        }
        weights = best_weights;
        // Fit diagnostics use the same quantized values that will be deployed.
        for weight in &mut weights {
            *weight = libm::round(*weight * SCORE_SCALE) / SCORE_SCALE;
        }
        let (after_correct, after_loss) = metrics(&examples, &weights);
        let (pointer_after_correct, pointer_after_loss) = pointer_metrics(&examples, &weights);
        memory.rows = addresses
            .into_iter()
            .map(|(feature, index)| MemoryWeight {
                feature,
                score: libm::round(weights[index] * SCORE_SCALE) as i32,
            })
            .collect();
        memory.fit_positions = examples.len();
        let report = MemoryReadFitReport {
            schema: memory.schema.clone(),
            cue_identity: if word_cues { CUE_SCHEMA } else { EXACT_CUE_SCHEMA }.into(),
            aliased_lexical_tokens: memory.cue_aliases.as_ref().map(|aliases| {
                aliases.representatives.iter().enumerate()
                    .filter(|(token, representative)| **representative as usize != *token).count()
            }).unwrap_or(0),
            objective: "pointer_only_token_marginal_then_fit_bias_grid_then_max_route_ce_v2; best_epoch_on_fit_ce; diagnostics_quantized_max_route"
                .into(),
            pointer_pretrain_epochs: config.epochs,
            max_route_refinement_epochs: config.epochs,
            calibrated_bias_score: libm::round(calibrated_bias * SCORE_SCALE) as i32,
            pointer_fit_correct_before: pointer_before_correct,
            pointer_fit_correct_after: pointer_after_correct,
            pointer_cross_entropy_before: pointer_before_loss,
            pointer_cross_entropy_after: pointer_after_loss,
            sampling: "equal_document_quota_uniform_body_plus_final_min_8_positions_v1".into(),
            documents: sampled_documents,
            positions: examples.len(),
            observed_context_positions: observed,
            tail_positions_per_document_limit: 8,
            tail_positions,
            body_positions: examples.len() - tail_positions,
            target_in_candidates: reachable,
            target_in_memory,
            candidate_positions,
            fit_correct_before: before_correct,
            fit_correct_after: after_correct,
            candidate_cross_entropy_before: before_loss,
            candidate_cross_entropy_after: after_loss,
            learned_features: memory.rows.len(),
            dropped_feature_events: dropped,
            epochs: config.epochs,
            session_memory_bytes: memory_bytes,
        };
        let mut learned = self.clone();
        learned.memory_read = Some(memory);
        learned.refresh_identity()?;
        learned.validate()?;
        Ok((learned, report))
    }
}

fn score(alternative: &Alternative, weights: &[f64]) -> f64 {
    f64::from(alternative.constant) / SCORE_SCALE
        + alternative
            .features
            .as_ref()
            .map(|features| {
                features
                    .iter()
                    .filter(|&&index| index != ABSENT)
                    .map(|&index| weights[index])
                    .sum::<f64>()
            })
            .unwrap_or(0.0)
}
fn winner_scores(example: &Example, weights: &[f64], winners: &mut Vec<(usize, f64)>) {
    winners.clear();
    for group in &example.groups {
        let mut best = (group[0], score(&example.alternatives[group[0]], weights));
        for &index in &group[1..] {
            let candidate = score(&example.alternatives[index], weights);
            if candidate > best.1 {
                best = (index, candidate);
            }
        }
        winners.push(best);
    }
}

fn pointer_metrics(examples: &[Example], weights: &[f64]) -> (usize, f64) {
    let mut reachable = 0;
    let mut correct = 0;
    let mut loss = 0.0;
    let mut winners = Vec::new();
    for example in examples {
        winners.clear();
        for group in &example.groups {
            let best = group
                .iter()
                .copied()
                .filter(|&index| example.alternatives[index].features.is_some())
                .map(|index| {
                    (
                        example.alternatives[index].token,
                        score(&example.alternatives[index], weights),
                    )
                })
                .max_by(|a, b| a.1.total_cmp(&b.1));
            if let Some(best) = best {
                winners.push(best);
            }
        }
        let Some(&(_, target_score)) = winners.iter().find(|&&(token, _)| token == example.target)
        else {
            continue;
        };
        let Some(&(best_token, maximum)) = winners
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
        else {
            continue;
        };
        correct += usize::from(best_token == example.target);
        let denominator: f64 = winners
            .iter()
            .map(|(_, score)| libm::exp(*score - maximum))
            .sum();
        loss += libm::log(denominator) + maximum - target_score;
        reachable += 1;
    }
    (correct, loss / reachable as f64)
}

fn calibrate_bias(examples: &[Example], weights: &mut [f64]) {
    // Cache the best base/pointer score per actual token. The pointer score
    // excludes its shared bias, so this bounded grid does not rescore features.
    let cache: Vec<_> = examples
        .iter()
        .map(|example| {
            let groups: Vec<_> = example
                .groups
                .iter()
                .map(|group| {
                    let mut base = f64::NEG_INFINITY;
                    let mut pointer = f64::NEG_INFINITY;
                    for &index in group {
                        let alternative = &example.alternatives[index];
                        let value = score(alternative, weights);
                        if alternative.features.is_some() {
                            pointer = pointer.max(value - weights[0]);
                        } else {
                            base = base.max(value);
                        }
                    }
                    (example.alternatives[group[0]].token, base, pointer)
                })
                .collect();
            (example.target, groups)
        })
        .collect();
    let loss = |bias: f64| -> f64 {
        let mut total = 0.0;
        for (target, groups) in &cache {
            let Some((_, base, pointer)) = groups.iter().find(|&&(token, _, _)| token == *target)
            else {
                continue;
            };
            let target_score = base.max(pointer + bias);
            let maximum = groups
                .iter()
                .map(|(_, base, pointer)| base.max(pointer + bias))
                .fold(f64::NEG_INFINITY, f64::max);
            let denominator: f64 = groups
                .iter()
                .map(|(_, base, pointer)| libm::exp(base.max(pointer + bias) - maximum))
                .sum();
            total += libm::log(denominator) + maximum - target_score;
        }
        total
    };
    let mut best_bias = weights[0];
    let mut best_loss = loss(best_bias);
    for half in -32..=32 {
        let bias = f64::from(half) * 0.5;
        let candidate_loss = loss(bias);
        if candidate_loss < best_loss {
            best_loss = candidate_loss;
            best_bias = bias;
        }
    }
    weights[0] = best_bias;
}

fn metrics(examples: &[Example], weights: &[f64]) -> (usize, f64) {
    let mut winners = Vec::new();
    let mut correct = 0;
    let mut reachable = 0;
    let mut loss = 0.0;
    for example in examples {
        winner_scores(example, weights, &mut winners);
        let Some(&(best, maximum)) = winners.iter().max_by(|a, b| {
            a.1.total_cmp(&b.1).then_with(|| {
                example.alternatives[b.0]
                    .token
                    .cmp(&example.alternatives[a.0].token)
            })
        }) else {
            continue;
        };
        correct += usize::from(example.alternatives[best].token == example.target);
        if let Some((_, target_score)) = winners
            .iter()
            .find(|&&(index, _)| example.alternatives[index].token == example.target)
        {
            let denominator: f64 = winners
                .iter()
                .map(|(_, value)| libm::exp(*value - maximum))
                .sum();
            loss += libm::log(denominator) + maximum - target_score;
            reachable += 1;
        }
    }
    (correct, loss / reachable as f64)
}
