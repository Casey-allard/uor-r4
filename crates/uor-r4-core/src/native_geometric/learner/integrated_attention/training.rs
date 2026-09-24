//! Offline coordinated learning on the same hard causal path used by export.
//!
//! Credit is explicitly truncated: output/gates/energy use local CE, source
//! annotations supervise query/key alignment, and state actions receive a
//! one-step expected language-loss surrogate. There is no claim of BPTT or a
//! solved hard-routing optimization problem. Labels never enter Runtime.
use super::*;
use encoder::{EncoderInput, EncoderTrainer};
use output::OutputTrainer;
use read_action::{ReadActionCandidate, ReadActionExample, ReadActionTrainConfig};
use std::collections::HashMap;
use training_support::{EnergyTrainer, SparseGateTrainer};

const A2_MAX_NEGATIVES: usize = 16;
const A2_MAX_ENERGY_NEGATIVES: usize = 8;
const A2_HEAD_WARMUP_EXPOSURES: u64 = 128;
const A2_UTILITY_MARGIN_NATS: f64 = 0.01;
const A2_ENERGY_TEMPERATURE: f64 = 8.0;

#[derive(Clone, Debug)]
pub struct TrainingEpisode {
    pub name: String,
    pub source_id: u64,
    pub tokens: Vec<u16>,
    pub token_bytes: Vec<Vec<u8>>,
    /// Fit-only source position for predicting each token; always strictly earlier.
    pub source_targets: Vec<Option<usize>>,
    /// Evaluation generates after this raw prefix; target suffix never enters serving.
    pub prompt_len: Option<usize>,
}
impl TrainingEpisode {
    pub fn validate(&self, config: &ModelConfig) -> Result<(), ModelError> {
        let n = self.tokens.len();
        if n == 0
            || self.token_bytes.len() != n
            || self.source_targets.len() != n
            || self.prompt_len.is_some_and(|p| p == 0 || p > n)
        {
            return Err(ModelError::Configuration("episode lengths"));
        }
        for (i, (&token, source)) in self.tokens.iter().zip(&self.source_targets).enumerate() {
            if usize::from(token) >= config.vocabulary
                || source.is_some_and(|s| s >= i)
                || self.token_bytes[i].len() > config.memory.max_token_bytes
            {
                return Err(ModelError::Configuration(
                    "episode token, byte or causal source bound",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FitConfig {
    pub output_rate: f64,
    pub encoder_rate: f64,
    pub gate_rate: f64,
    pub energy_rate: f64,
    pub state_credit_every: usize,
}
impl Default for FitConfig {
    fn default() -> Self {
        Self {
            output_rate: 1.0,
            encoder_rate: 0.03,
            gate_rate: 0.03,
            energy_rate: 0.1,
            state_credit_every: 32,
        }
    }
}

/// Prospective A4 fit policy. Both treatment and continuation control use
/// the same answer/Stop repetitions; only a version-3 model has a selector.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct A4FitConfig {
    pub answers_head_repeats: u8,
    pub natural_selector_stride: usize,
    pub selector_margin: i32,
}
impl Default for A4FitConfig {
    fn default() -> Self {
        Self {
            answers_head_repeats: 8,
            natural_selector_stride: 256,
            selector_margin: 2,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FitMetrics {
    pub positions: u64,
    pub surface_token_nll_nats: f64,
    pub labeled_sources: u64,
    pub source_admitted: u64,
    /// Annotated source has an owned semantic record. Its coarse posting may
    /// still be absent if that page filled; `source_admitted` is measured later.
    pub source_committed: u64,
    pub source_ranked_before_gate: u64,
    pub source_selected: u64,
    pub selected_reads: u64,
    pub correct_selected_tokens: u64,
    pub writes: u64,
    pub query_key_updates: u64,
    pub contrastive_updates: u64,
    pub contrastive_skipped_no_source: u64,
    pub contrastive_skipped_no_negative: u64,
    pub fine_energy_query_updates: u64,
    pub fine_energy_key_updates: u64,
    pub fine_energy_flat_skips: u64,
    pub forced_source_head_updates: u64,
    pub forced_source_head_nll_nats: f64,
    pub forced_source_no_read_nll_nats: f64,
    pub strong_source_beats_alternatives: u64,
    pub strong_source_language_margin_nats: f64,
    pub read_gate_updates: u64,
    pub state_lane_updates: u64,
    pub energy_updates: u64,
    pub answer_head_extra_updates: u64,
    pub answer_forced_head_extra_updates: u64,
    pub answer_no_read_extra_updates: u64,
    pub answer_stop_extra_updates: u64,
    pub read_action_updates: u64,
    pub read_action_integer_edits: u64,
    pub read_action_target_eligible: u64,
    pub read_action_target_selected: u64,
    pub read_action_target_not_admitted: u64,
    pub read_action_skipped_unidentifiable: u64,
    pub read_action_utility_examples: u64,
    pub read_action_natural_utility_examples: u64,
    pub read_action_answer_utility_examples: u64,
    pub access: AccessCounts,
}
impl FitMetrics {
    pub fn add(&mut self, other: &Self) {
        self.positions += other.positions;
        self.surface_token_nll_nats += other.surface_token_nll_nats;
        self.labeled_sources += other.labeled_sources;
        self.source_admitted += other.source_admitted;
        self.source_committed += other.source_committed;
        self.source_ranked_before_gate += other.source_ranked_before_gate;
        self.source_selected += other.source_selected;
        self.selected_reads += other.selected_reads;
        self.correct_selected_tokens += other.correct_selected_tokens;
        self.writes += other.writes;
        self.query_key_updates += other.query_key_updates;
        self.contrastive_updates += other.contrastive_updates;
        self.contrastive_skipped_no_source += other.contrastive_skipped_no_source;
        self.contrastive_skipped_no_negative += other.contrastive_skipped_no_negative;
        self.fine_energy_query_updates += other.fine_energy_query_updates;
        self.fine_energy_key_updates += other.fine_energy_key_updates;
        self.fine_energy_flat_skips += other.fine_energy_flat_skips;
        self.forced_source_head_updates += other.forced_source_head_updates;
        self.forced_source_head_nll_nats += other.forced_source_head_nll_nats;
        self.forced_source_no_read_nll_nats += other.forced_source_no_read_nll_nats;
        self.strong_source_beats_alternatives += other.strong_source_beats_alternatives;
        self.strong_source_language_margin_nats += other.strong_source_language_margin_nats;
        self.read_gate_updates += other.read_gate_updates;
        self.state_lane_updates += other.state_lane_updates;
        self.energy_updates += other.energy_updates;
        self.answer_head_extra_updates += other.answer_head_extra_updates;
        self.answer_forced_head_extra_updates += other.answer_forced_head_extra_updates;
        self.answer_no_read_extra_updates += other.answer_no_read_extra_updates;
        self.answer_stop_extra_updates += other.answer_stop_extra_updates;
        self.read_action_updates += other.read_action_updates;
        self.read_action_integer_edits += other.read_action_integer_edits;
        self.read_action_target_eligible += other.read_action_target_eligible;
        self.read_action_target_selected += other.read_action_target_selected;
        self.read_action_target_not_admitted += other.read_action_target_not_admitted;
        self.read_action_skipped_unidentifiable += other.read_action_skipped_unidentifiable;
        self.read_action_utility_examples += other.read_action_utility_examples;
        self.read_action_natural_utility_examples += other.read_action_natural_utility_examples;
        self.read_action_answer_utility_examples += other.read_action_answer_utility_examples;
        self.access.add(other.access);
    }
    pub fn bits_per_token(&self) -> Option<f64> {
        (self.positions > 0)
            .then(|| self.surface_token_nll_nats / self.positions as f64 / std::f64::consts::LN_2)
    }
}

/// Exact for A1's one-token Copy: Copy and Generate of the same token both
/// enter the identical observe(token, evidence) transition, without a cursor.
/// This does not establish tractable marginalization for a future span-copy model.
pub fn token_nll_offline(
    output: &SparseOutput,
    features: &[u16],
    selected: Option<u16>,
    target: u16,
) -> Result<f64, ModelError> {
    let generate = output
        .score_action(features, selected.is_some(), Action::Generate(target))?
        .action_ce_offline();
    if selected == Some(target) {
        let copy = output
            .score_action(features, true, Action::Copy)?
            .action_ce_offline();
        let min = generate.min(copy);
        Ok(min - ((min - generate).exp() + (min - copy).exp()).ln())
    } else {
        Ok(generate)
    }
}

#[derive(Clone, Copy)]
struct ObservedTraining {
    key_input: Option<EncoderInput>,
    key: Option<[u8; 16]>,
    record_id: Option<u64>,
}

/// Fit-only competitors have already been committed to the causal index. Hard
/// admitted candidates come first; earlier committed records provide a route
/// to learn from a positive missed by the current page. A natural equal-bigram
/// pointer is weak: only a different next token after the same predecessor is
/// used as a negative for that auxiliary next-token relation. This does not
/// label arbitrary occurrence IDs as semantically inequivalent.
fn contrastive_negatives(
    episode: &TrainingEpisode,
    observed: &[ObservedTraining],
    record_positions: &HashMap<u64, usize>,
    read: &ReadTrace,
    source_position: usize,
    strong_source: bool,
) -> Vec<EncoderInput> {
    let Some(positive_input) = observed[source_position].key_input else {
        return Vec::new();
    };
    let mut negatives = Vec::with_capacity(A2_MAX_NEGATIVES);
    let source_value = episode.tokens[source_position];
    let source_predecessor = source_position
        .checked_sub(1)
        .map(|index| episode.tokens[index]);
    let hard_candidates = read.candidates[..read.candidate_count]
        .iter()
        .filter_map(|candidate| record_positions.get(&candidate.record_id).copied());
    for index in hard_candidates.chain((0..observed.len()).rev()) {
        if negatives.len() == A2_MAX_NEGATIVES {
            break;
        }
        if index == source_position || observed[index].record_id.is_none() {
            continue;
        }
        if !strong_source
            && (episode.tokens[index] == source_value
                || index == 0
                || source_predecessor != Some(episode.tokens[index - 1]))
        {
            continue;
        }
        let Some(input) = observed[index].key_input else {
            continue;
        };
        if input != positive_input && !negatives.contains(&input) {
            negatives.push(input);
        }
    }
    negatives
}

/// Frozen one-lane intervention for the finite relative-energy classifier.
/// This is offline source supervision over a bounded list, not a derivative
/// through the hard page, changed state trajectory, gate or language head.
/// Lane zero is reserved for A2's separate admission contrastive loss.
fn fine_energy_choice_losses(
    frozen: &IntegratedModel,
    query: &[u8; 16],
    positive_key: &[u8; 16],
    negative_keys: &[[u8; 16]],
    lane: usize,
    vary_query: bool,
) -> Result<[f64; 120], ModelError> {
    let mut losses = [0.0; 120];
    let actions = [frozen.algebra.identity(); 16];
    let mut relative = [0u8; 16];
    for action in 0..120u8 {
        let mut alternate_query = *query;
        let mut alternate_key = *positive_key;
        if vary_query {
            alternate_query[lane] = action;
        } else {
            alternate_key[lane] = action;
        }
        let mut logits = [0.0; A2_MAX_ENERGY_NEGATIVES + 1];
        for (index, key) in std::iter::once(&alternate_key)
            .chain(negative_keys.iter())
            .enumerate()
        {
            frozen.algebra.relative_lanes(
                &alternate_query[..frozen.config.lanes],
                &actions[..frozen.config.lanes],
                &key[..frozen.config.lanes],
                &mut relative[..frozen.config.lanes],
            )?;
            let mut reads = geometry::EnergyReadCounts::default();
            let energy = frozen
                .energy
                .score(&relative[..frozen.config.lanes], &mut reads)?;
            logits[index] = -f64::from(energy) / A2_ENERGY_TEMPERATURE;
        }
        let count = negative_keys.len() + 1;
        let max_logit = logits[..count]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let normalizer = logits[..count]
            .iter()
            .map(|logit| (logit - max_logit).exp())
            .sum::<f64>();
        losses[usize::from(action)] = max_logit + normalizer.ln() - logits[0];
    }
    Ok(losses)
}

pub struct JointTrainer {
    pub config: ModelConfig,
    pub binding: ArtifactBinding,
    pub algebra: FiniteAlgebra,
    pub encoder: EncoderTrainer,
    pub energy: EnergyTrainer,
    pub output: OutputTrainer,
    pub read_gate: SparseGateTrainer,
    pub write_gate: SparseGateTrainer,
    pub read_action: Option<ReadActionModel>,
    a2_head_exposures: u64,
    coarse_address_frozen: bool,
    a4_fit: Option<A4FitConfig>,
}
impl JointTrainer {
    pub fn from_model(model: IntegratedModel) -> Result<Self, ModelError> {
        model.validate()?;
        Ok(Self {
            config: model.config,
            binding: model.binding,
            algebra: model.algebra,
            encoder: EncoderTrainer::from_model(model.encoder)?,
            energy: EnergyTrainer::new(model.energy)?,
            output: OutputTrainer::from_model(model.output)?,
            read_gate: SparseGateTrainer::from_model(model.read_gate)?,
            write_gate: SparseGateTrainer::from_model(model.write_gate)?,
            read_action: model.read_action,
            a2_head_exposures: 0,
            coarse_address_frozen: false,
            a4_fit: None,
        })
    }
    /// Preserve an offline-fitted hard coarse router during subsequent joint
    /// language/state/fine-energy learning. This disables only A2's lane-zero
    /// contrastive update; fine contextual lanes and State remain trainable.
    /// The choice is an offline training policy and must be bound in the
    /// exported artifact's credit-assignment description.
    pub fn freeze_context_address_learning(&mut self, frozen: bool) {
        self.coarse_address_frozen = frozen;
    }
    pub fn configure_a4(&mut self, config: A4FitConfig) -> Result<(), ModelError> {
        if !(1..=32).contains(&config.answers_head_repeats)
            || config.natural_selector_stride == 0
            || !(1..=32).contains(&config.selector_margin)
            || !self.encoder.model().context_addressing
        {
            return Err(ModelError::Configuration("A4 training policy bounds"));
        }
        self.a4_fit = Some(config);
        Ok(())
    }
    pub fn runtime(&self) -> Runtime<'_> {
        Runtime {
            config: &self.config,
            algebra: &self.algebra,
            encoder: self.encoder.model(),
            energy: self.energy.model(),
            output: self.output.model(),
            read_gate: self.read_gate.model(),
            write_gate: self.write_gate.model(),
            read_action: self.read_action.as_ref(),
            digest: [0; 32],
        }
    }
    pub fn export(self) -> IntegratedModel {
        IntegratedModel {
            version: if self.read_action.is_some() {
                3
            } else if self.encoder.model().context_addressing {
                2
            } else {
                1
            },
            config: self.config,
            binding: self.binding,
            algebra: self.algebra,
            encoder: self.encoder.export(),
            energy: self.energy.export(),
            output: self.output.export(),
            read_gate: self.read_gate.export(),
            write_gate: self.write_gate.export(),
            read_action: self.read_action,
            digest: [0; 32],
        }
    }
    fn frozen_episode_model(&self) -> IntegratedModel {
        IntegratedModel {
            version: if self.read_action.is_some() {
                3
            } else if self.encoder.model().context_addressing {
                2
            } else {
                1
            },
            config: self.config.clone(),
            binding: self.binding.clone(),
            algebra: self.algebra.clone(),
            encoder: self.encoder.model().clone(),
            energy: self.energy.model().clone(),
            output: self.output.model().clone(),
            read_gate: self.read_gate.model().clone(),
            write_gate: self.write_gate.model().clone(),
            read_action: self.read_action.clone(),
            digest: [0; 32],
        }
    }

    /// Fit labels are applied only to candidates admitted by the frozen causal
    /// forward episode. A missing annotated source is not a NoRead example.
    /// Natural and later-answer utility labels may accept multiple equally
    /// useful actions, so equal values do not acquire conflicting lexical labels.
    fn fit_read_action_step(
        &mut self,
        frozen: &IntegratedModel,
        session: &Session,
        read: &ReadTrace,
        episode: &TrainingEpisode,
        observed: &[ObservedTraining],
        position: usize,
        target: u16,
        no_read_loss: f64,
        metrics: &mut FitMetrics,
    ) -> Result<(), ModelError> {
        let (Some(policy), Some(selector)) = (self.a4_fit, self.read_action.as_mut()) else {
            return Ok(());
        };
        let first_answer = episode.prompt_len == Some(position);
        let utility_step = episode
            .prompt_len
            .map_or(position % policy.natural_selector_stride == 0, |prompt| {
                position > prompt
            });
        if !first_answer && !utility_step {
            return Ok(());
        }
        let none = features(&self.config, session.last_token, &session.state, None);
        let candidates: Vec<ReadActionCandidate> = read.candidates[..read.candidate_count]
            .iter()
            .map(|candidate| ReadActionCandidate {
                features: features(
                    &self.config,
                    session.last_token,
                    &session.state,
                    Some(candidate.token),
                )
                .as_slice()
                .to_vec(),
                relative: candidate.relative[..self.config.lanes].to_vec(),
            })
            .collect();
        let mut example = ReadActionExample {
            no_read_features: none.as_slice().to_vec(),
            candidates,
            target: None,
        };
        let acceptable = if first_answer {
            let source_record = episode.source_targets[position]
                .and_then(|source| observed.get(source))
                .and_then(|source| source.record_id);
            let Some(index) = source_record.and_then(|id| {
                read.candidates[..read.candidate_count]
                    .iter()
                    .position(|c| c.record_id == id)
            }) else {
                metrics.read_action_target_not_admitted += 1;
                return Ok(());
            };
            // Exact occurrence supervision cannot resolve equal served inputs.
            if example
                .candidates
                .iter()
                .enumerate()
                .any(|(other, candidate)| {
                    other != index
                        && candidate.features == example.candidates[index].features
                        && candidate.relative == example.candidates[index].relative
                })
            {
                metrics.read_action_skipped_unidentifiable += 1;
                return Ok(());
            }
            metrics.read_action_target_eligible += 1;
            example.target = Some(index);
            vec![Some(index)]
        } else {
            // Conditional gold-token loss is offline supervision, not a served
            // feature. No future-target-derived recurrence pointer labels a read.
            let mut token_losses = HashMap::<u16, f64>::new();
            let mut losses = Vec::with_capacity(read.candidate_count + 1);
            losses.push((None, no_read_loss));
            for (index, candidate) in read.candidates[..read.candidate_count].iter().enumerate() {
                let loss = if let Some(&loss) = token_losses.get(&candidate.token) {
                    loss
                } else {
                    let loss = token_nll_offline(
                        &frozen.output,
                        &example.candidates[index].features,
                        Some(candidate.token),
                        target,
                    )?;
                    token_losses.insert(candidate.token, loss);
                    loss
                };
                losses.push((Some(index), loss));
            }
            let best = losses
                .iter()
                .map(|(_, loss)| *loss)
                .fold(f64::INFINITY, f64::min);
            metrics.read_action_utility_examples += 1;
            if episode.prompt_len.is_some() {
                metrics.read_action_answer_utility_examples += 1;
            } else {
                metrics.read_action_natural_utility_examples += 1;
            }
            losses
                .into_iter()
                .filter_map(|(action, loss)| {
                    (loss <= best + A2_UTILITY_MARGIN_NATS).then_some(action)
                })
                .collect()
        };
        let report = selector.train_acceptable(
            &example,
            &acceptable,
            &ReadActionTrainConfig {
                margin: policy.selector_margin,
                max_steps: 4,
            },
        )?;
        metrics.read_action_updates += 1;
        metrics.read_action_integer_edits += report.coefficient_edits;
        if first_answer {
            metrics.read_action_target_selected +=
                u64::from(acceptable.contains(&report.selected_after));
        }
        Ok(())
    }
    pub fn fit_episode(
        &mut self,
        episode: &TrainingEpisode,
        fit: FitConfig,
    ) -> Result<FitMetrics, ModelError> {
        episode.validate(&self.config)?;
        if fit.state_credit_every == 0 {
            return Err(ModelError::Configuration("state credit interval is zero"));
        }
        // One fixed forward model per episode. Updates below affect only the next
        // episode, so retained keys, queries and hard state use one codebook version.
        let frozen = self.frozen_episode_model();
        let a2 = frozen.encoder.context_addressing;
        // Gate decisions are trained only after source-conditioned head credit
        // has reached a later frozen episode. A read that is harmful solely
        // because the head is still untrained is not a useful negative label.
        let gate_ready = !a2 || self.a2_head_exposures >= A2_HEAD_WARMUP_EXPOSURES;
        let mut session = Session::new(frozen.runtime(), episode.source_id)?;
        let mut observed = Vec::<ObservedTraining>::with_capacity(episode.tokens.len());
        let mut event_positions = HashMap::<u64, usize>::with_capacity(episode.tokens.len());
        let mut record_positions = HashMap::<u64, usize>::with_capacity(episode.tokens.len());
        let mut future_sources = vec![false; episode.tokens.len()];
        for source in episode.source_targets.iter().flatten() {
            future_sources[*source] = true;
        }
        let mut metrics = FitMetrics::default();
        for (position, &target) in episode.tokens.iter().enumerate() {
            let read = session.read(frozen.runtime(), true)?;
            let selected = read.selected_token();
            let feat = features(&self.config, session.last_token, &session.state, selected);
            let no_read_features = features(&self.config, session.last_token, &session.state, None);
            let loss = token_nll_offline(&frozen.output, feat.as_slice(), selected, target)?;
            let no_read_loss =
                token_nll_offline(&frozen.output, no_read_features.as_slice(), None, target)?;
            metrics.positions += 1;
            metrics.surface_token_nll_nats += loss;
            metrics.selected_reads += u64::from(selected.is_some());
            metrics.correct_selected_tokens += u64::from(selected == Some(target));
            metrics.access.add(read.access);

            // Credit to the preceding state action holds the selected source fixed.
            // It is an explicit one-step intervention, not a gradient through search.
            if position > 0 && position % fit.state_credit_every == 0 {
                if let Some((input, _)) = session.last_transition {
                    for lane in 0..self.config.lanes {
                        let mut losses = [0.0; 120];
                        let mut alternate_state = session.state;
                        for action in 0..120u8 {
                            alternate_state[lane] =
                                self.algebra.compose(input.previous[lane], action)?;
                            let altered = features(
                                &self.config,
                                session.last_token,
                                &alternate_state,
                                selected,
                            );
                            losses[usize::from(action)] = token_nll_offline(
                                &frozen.output,
                                altered.as_slice(),
                                selected,
                                target,
                            )?;
                        }
                        self.encoder
                            .train_lane_losses(input, lane, &losses, fit.encoder_rate)?;
                        metrics.state_lane_updates += 1;
                    }
                }
            }
            self.output
                .train_token(feat.as_slice(), selected, target, fit.output_rate)?;
            let answer_repeats = if episode.prompt_len.is_some_and(|prompt| position >= prompt) {
                self.a4_fit.map_or(1, |policy| policy.answers_head_repeats)
            } else {
                1
            };
            for _ in 1..answer_repeats {
                self.output
                    .train_token(feat.as_slice(), selected, target, fit.output_rate)?;
                metrics.answer_head_extra_updates += 1;
            }
            if let Some(source_position) = episode.source_targets[position] {
                let source = observed[source_position];
                metrics.labeled_sources += 1;
                metrics.source_committed += u64::from(source.record_id.is_some());
                let admitted = source.record_id.and_then(|id| {
                    read.candidates[..read.candidate_count]
                        .iter()
                        .position(|c| c.record_id == id)
                });
                metrics.source_admitted += u64::from(admitted.is_some());
                metrics.source_ranked_before_gate +=
                    u64::from(admitted.is_some() && admitted == read.ranked);
                metrics.source_selected +=
                    u64::from(admitted.is_some() && admitted == read.selected);
                if a2 {
                    let mut source_language_loss = None;
                    if source.record_id.is_some() {
                        let source_token = episode.tokens[source_position];
                        let source_features = features(
                            &self.config,
                            session.last_token,
                            &session.state,
                            Some(source_token),
                        );
                        let forced_loss = token_nll_offline(
                            &frozen.output,
                            source_features.as_slice(),
                            Some(source_token),
                            target,
                        )?;
                        source_language_loss = Some(forced_loss);
                        metrics.forced_source_head_nll_nats += forced_loss;
                        metrics.forced_source_no_read_nll_nats += no_read_loss;
                        metrics.forced_source_head_updates += 1;
                        // A NoRead example preserves ordinary prediction while
                        // the sourced example teaches the same head to use an
                        // already observed value. These are fit-only causal
                        // interventions; neither source ID nor answer is served.
                        self.output.train_token(
                            no_read_features.as_slice(),
                            None,
                            target,
                            fit.output_rate * 0.5,
                        )?;
                        self.output.train_token(
                            source_features.as_slice(),
                            Some(source_token),
                            target,
                            fit.output_rate,
                        )?;
                        for _ in 1..answer_repeats {
                            self.output.train_token(
                                no_read_features.as_slice(),
                                None,
                                target,
                                fit.output_rate * 0.5,
                            )?;
                            self.output.train_token(
                                source_features.as_slice(),
                                Some(source_token),
                                target,
                                fit.output_rate,
                            )?;
                            metrics.answer_no_read_extra_updates += 1;
                            metrics.answer_forced_head_extra_updates += 1;
                        }
                        if episode.prompt_len.is_some_and(|p| position >= p) {
                            self.a2_head_exposures += 1;
                        }
                    }
                    // In a correction, the first answer token is the
                    // discriminating consequence. Requiring the same record
                    // on every later name/punctuation token would turn a
                    // grounding hint into false repeated-read supervision.
                    // Natural equal-bigram pointers were chosen using the eventual target.
                    // They are useful head/copy hints, but do not identify a source
                    // from the query. Only grounded first decisions supervise addresses.
                    let address_step = episode.prompt_len.is_some_and(|p| position == p);
                    if address_step {
                        if let (Some(_), Some(positive_input), Some(source_key)) =
                            (source.record_id, source.key_input, source.key)
                        {
                            let strong = episode.prompt_len.is_some();
                            let negatives = contrastive_negatives(
                                episode,
                                &observed,
                                &record_positions,
                                &read,
                                source_position,
                                strong,
                            );
                            if self.coarse_address_frozen {
                                // A3 learns integer admission before this
                                // joint stage. Soft overlap must not silently
                                // replace those exported coarse decisions.
                            } else if negatives.is_empty() {
                                metrics.contrastive_skipped_no_negative += 1;
                            } else {
                                let contrastive_loss = self.encoder.train_contrastive(
                                    read.input,
                                    positive_input,
                                    &negatives,
                                    fit.encoder_rate,
                                )?;
                                if contrastive_loss == 0.0 {
                                    metrics.contrastive_skipped_no_negative += 1;
                                } else {
                                    metrics.contrastive_updates += 1;
                                    metrics.query_key_updates += 1;
                                }
                            }
                            if strong && self.read_action.is_none() {
                                // The annotation chooses a source for this
                                // authored correction; candidate-conditioned
                                // language loss scales (rather than replaces)
                                // its ranking credit after head exposure.
                                let mut energy_rate = fit.energy_rate;
                                if gate_ready {
                                    if let Some(source_loss) = source_language_loss {
                                        let mut alternative_loss = no_read_loss;
                                        for candidate in
                                            read.candidates[..read.candidate_count].iter()
                                        {
                                            if Some(candidate.record_id) == source.record_id {
                                                continue;
                                            }
                                            let candidate_features = features(
                                                &self.config,
                                                session.last_token,
                                                &session.state,
                                                Some(candidate.token),
                                            );
                                            let loss = token_nll_offline(
                                                &frozen.output,
                                                candidate_features.as_slice(),
                                                Some(candidate.token),
                                                target,
                                            )?;
                                            alternative_loss = alternative_loss.min(loss);
                                        }
                                        let margin = alternative_loss - source_loss;
                                        metrics.strong_source_language_margin_nats += margin;
                                        metrics.strong_source_beats_alternatives +=
                                            u64::from(margin > A2_UTILITY_MARGIN_NATS);
                                        let bounded = margin.clamp(-20.0, 20.0);
                                        let utility_weight = 0.25 + 0.75 / (1.0 + (-bounded).exp());
                                        energy_rate *= utility_weight;
                                    }
                                }
                                // Source insertion is fit-only when the hard
                                // page misses. It cannot count as admission.
                                let mut relatives: Vec<Vec<u8>> = read.candidates
                                    [..read.candidate_count]
                                    .iter()
                                    .map(|c| c.relative[..self.config.lanes].to_vec())
                                    .collect();
                                let positive = if let Some(index) = admitted {
                                    index
                                } else {
                                    let actions = [self.algebra.identity(); 16];
                                    let mut relative = vec![0; self.config.lanes];
                                    self.algebra.relative_lanes(
                                        &read.query[..self.config.lanes],
                                        &actions[..self.config.lanes],
                                        &source_key[..self.config.lanes],
                                        &mut relative,
                                    )?;
                                    let index = relatives.len();
                                    relatives.push(relative);
                                    index
                                };
                                if relatives.len() > 1 {
                                    self.energy
                                        .train_choice(&relatives, positive, energy_rate)?;
                                    metrics.energy_updates += 1;
                                }
                                // Truncated credit from the frozen relative
                                // energy into the fine query and positive-key
                                // lanes. The hard index and gate remain fixed.
                                let mut negative_keys = Vec::<[u8; 16]>::new();
                                for candidate in read.candidates[..read.candidate_count].iter() {
                                    if Some(candidate.record_id) == source.record_id
                                        || candidate.key[..self.config.lanes]
                                            == source_key[..self.config.lanes]
                                        || negative_keys.iter().any(|key| {
                                            key[..self.config.lanes]
                                                == candidate.key[..self.config.lanes]
                                        })
                                    {
                                        continue;
                                    }
                                    negative_keys.push(candidate.key);
                                    if negative_keys.len() == A2_MAX_ENERGY_NEGATIVES {
                                        break;
                                    }
                                }
                                if !negative_keys.is_empty() {
                                    for lane in 1..self.config.lanes {
                                        for vary_query in [true, false] {
                                            let losses = fine_energy_choice_losses(
                                                &frozen,
                                                &read.query,
                                                &source_key,
                                                &negative_keys,
                                                lane,
                                                vary_query,
                                            )?;
                                            let minimum = losses
                                                .iter()
                                                .copied()
                                                .fold(f64::INFINITY, f64::min);
                                            let maximum = losses
                                                .iter()
                                                .copied()
                                                .fold(f64::NEG_INFINITY, f64::max);
                                            if maximum - minimum <= 1.0e-10 {
                                                metrics.fine_energy_flat_skips += 1;
                                                continue;
                                            }
                                            let input = if vary_query {
                                                read.input
                                            } else {
                                                positive_input
                                            };
                                            self.encoder.train_lane_losses(
                                                input,
                                                lane,
                                                &losses,
                                                fit.encoder_rate,
                                            )?;
                                            if vary_query {
                                                metrics.fine_energy_query_updates += 1;
                                            } else {
                                                metrics.fine_energy_key_updates += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            metrics.contrastive_skipped_no_source += 1;
                        }
                    }
                } else {
                    let source_key = source
                        .key
                        .ok_or(ModelError::Artifact("A1 source key missing"))?;
                    let source_input = source
                        .key_input
                        .ok_or(ModelError::Artifact("A1 source input missing"))?;
                    self.encoder.supervise(
                        read.input,
                        &source_key[..self.config.lanes],
                        fit.encoder_rate,
                    )?;
                    self.encoder.supervise(
                        source_input,
                        &read.query[..self.config.lanes],
                        fit.encoder_rate * 0.1,
                    )?;
                    metrics.query_key_updates += 2;
                    let mut relatives: Vec<Vec<u8>> = read.candidates[..read.candidate_count]
                        .iter()
                        .map(|c| c.relative[..self.config.lanes].to_vec())
                        .collect();
                    let positive = if let Some(index) = admitted {
                        index
                    } else {
                        let actions = [self.algebra.identity(); 16];
                        let mut relative = vec![0; self.config.lanes];
                        self.algebra.relative_lanes(
                            &read.query[..self.config.lanes],
                            &actions[..self.config.lanes],
                            &source_key[..self.config.lanes],
                            &mut relative,
                        )?;
                        let index = relatives.len();
                        relatives.push(relative);
                        index
                    };
                    if relatives.len() > 1 {
                        self.energy
                            .train_choice(&relatives, positive, fit.energy_rate)?;
                        metrics.energy_updates += 1;
                    }
                }
            }
            // Missing annotations are not evidence that reading is useless. A negative
            // gate label is used only when an actual selected read worsens token loss.
            self.fit_read_action_step(
                &frozen,
                &session,
                &read,
                episode,
                &observed,
                position,
                target,
                no_read_loss,
                &mut metrics,
            )?;
            if gate_ready && self.read_action.is_none() {
                if let Some(best) = read.ranked {
                    let candidate = read.candidates[best];
                    let candidate_features = features(
                        &self.config,
                        session.last_token,
                        &session.state,
                        Some(candidate.token),
                    );
                    let candidate_loss = token_nll_offline(
                        &frozen.output,
                        candidate_features.as_slice(),
                        Some(candidate.token),
                        target,
                    )?;
                    let right_source = episode.source_targets[position]
                        .is_some_and(|s| observed[s].record_id == Some(candidate.record_id));
                    let useful = if a2 {
                        candidate_loss + A2_UTILITY_MARGIN_NATS < no_read_loss
                    } else if episode.source_targets[position].is_some() {
                        right_source
                    } else {
                        candidate_loss < no_read_loss
                    };
                    self.read_gate
                        .train(candidate_features.as_slice(), useful, fit.gate_rate)?;
                    metrics.read_gate_updates += 1;
                }
            }
            // Provenance credit belongs to query/key, energy and read-gate updates.
            // This head sees the selected token, not the record ID: equal-valued
            // sources must not receive contradictory Copy/Generate supervision here.

            let event = session.observe(
                frozen.runtime(),
                u32::from(target),
                &episode.token_bytes[position],
                selected,
            )?;
            // Source annotations are incomplete: unmentioned events are unknown,
            // not unwanted writes. During this positive-unlabeled warmup only
            // witnessed useful sources receive credit. The initially enabled
            // gate therefore retains all indexed occurrences at this stage.
            // Selective writes need a later storage-pressure/utility objective.
            metrics.writes += u64::from(event.record_id.is_some());
            metrics.access.add(event.access);
            observed.push(ObservedTraining {
                key_input: (!a2).then_some(event.key_input),
                key: (!a2).then_some(event.key),
                record_id: (!a2).then_some(event.record_id).flatten(),
            });
            event_positions.insert(event.event_id, position);
            let indexed_position = if let Some(source_event_id) = event.indexed_source_event_id {
                Some(
                    *event_positions
                        .get(&source_event_id)
                        .ok_or(ModelError::Artifact("indexed source event is not observed"))?,
                )
            } else if !a2 && event.record_id.is_some() {
                Some(position)
            } else {
                None
            };
            if let Some(index) = indexed_position {
                observed[index].key_input = Some(event.key_input);
                observed[index].key = Some(event.key);
                observed[index].record_id = event.record_id;
                if let Some(id) = event.record_id {
                    record_positions.insert(id, index);
                }
                if future_sources[index] {
                    self.write_gate
                        .train(event.gate_features.as_slice(), true, fit.gate_rate)?;
                }
            }
            self.binding.training_updates += 1;
        }
        let read = session.read(frozen.runtime(), true)?;
        let feat = features(
            &self.config,
            session.last_token,
            &session.state,
            read.selected_token(),
        );
        self.output.train_action(
            feat.as_slice(),
            read.selected.is_some(),
            Action::Stop,
            fit.output_rate,
        )?;
        if episode.prompt_len.is_some() {
            for _ in 1..self.a4_fit.map_or(1, |policy| policy.answers_head_repeats) {
                self.output.train_action(
                    feat.as_slice(),
                    read.selected.is_some(),
                    Action::Stop,
                    fit.output_rate,
                )?;
                metrics.answer_stop_extra_updates += 1;
            }
        }
        Ok(metrics)
    }
}

pub fn evaluate_episode(
    model: &IntegratedModel,
    episode: &TrainingEpisode,
    reads_enabled: bool,
) -> Result<FitMetrics, ModelError> {
    episode.validate(&model.config)?;
    let mut metrics = FitMetrics::default();
    let mut session = Session::new(model.runtime(), episode.source_id)?;
    let mut source_records = Vec::<Option<u64>>::with_capacity(episode.tokens.len());
    let mut event_positions = HashMap::<u64, usize>::with_capacity(episode.tokens.len());
    for (position, &target) in episode.tokens.iter().enumerate() {
        let read = session.read(model.runtime(), reads_enabled)?;
        let selected = read.selected_token();
        let feat = features(&model.config, session.last_token, &session.state, selected);
        let nll = token_nll_offline(&model.output, feat.as_slice(), selected, target)?;
        let generated = model.output.score_action(
            feat.as_slice(),
            selected.is_some(),
            Action::Generate(target),
        )?;
        metrics.access.output_coefficients += u64::from(generated.coefficient_reads);
        if selected == Some(target) {
            metrics.access.output_coefficients += u64::from(
                model
                    .output
                    .score_action(feat.as_slice(), true, Action::Copy)?
                    .coefficient_reads,
            );
        }
        metrics.positions += 1;
        metrics.surface_token_nll_nats += nll;
        metrics.selected_reads += u64::from(selected.is_some());
        metrics.correct_selected_tokens += u64::from(selected == Some(target));
        metrics.access.add(read.access);
        if let Some(source) = episode.source_targets[position] {
            metrics.labeled_sources += 1;
            metrics.source_committed += u64::from(source_records[source].is_some());
            let admitted = source_records[source].and_then(|id| {
                read.candidates[..read.candidate_count]
                    .iter()
                    .position(|c| c.record_id == id)
            });
            metrics.source_admitted += u64::from(admitted.is_some());
            metrics.source_ranked_before_gate +=
                u64::from(admitted.is_some() && admitted == read.ranked);
            metrics.source_selected += u64::from(admitted.is_some() && admitted == read.selected);
        }
        let event = session.observe(
            model.runtime(),
            u32::from(target),
            &episode.token_bytes[position],
            selected,
        )?;
        metrics.writes += u64::from(event.record_id.is_some());
        metrics.access.add(event.access);
        source_records.push(None);
        event_positions.insert(event.event_id, position);
        let indexed_position = if let Some(source_event_id) = event.indexed_source_event_id {
            Some(
                *event_positions
                    .get(&source_event_id)
                    .ok_or(ModelError::Artifact("indexed source event is not observed"))?,
            )
        } else if !model.encoder.context_addressing && event.record_id.is_some() {
            Some(position)
        } else {
            None
        };
        if let Some(index) = indexed_position {
            source_records[index] = event.record_id;
        }
    }
    Ok(metrics)
}

#[cfg(test)]
mod tests {
    #[test]
    fn frozen_coarse_router_survives_joint_language_learning() -> Result<(), ModelError> {
        let mut model =
            IntegratedModel::new(ModelConfig::pilot(64, 4)?, AlgebraKind::Cyclic120, 17)?;
        model.enable_context_addressing()?;
        let coarse = |encoder: &CodeEncoder| -> Vec<i8> {
            encoder
                .address_coefficients()
                .chunks_exact(4 * encoder::ROW_STRIDE)
                .flat_map(|rows| rows[..encoder::ROW_STRIDE].iter().copied())
                .collect()
        };
        let before_coarse = coarse(&model.encoder);
        let before_output = model.output.coefficients().to_vec();
        let mut trainer = JointTrainer::from_model(model)?;
        trainer.freeze_context_address_learning(true);
        let tokens: Vec<u16> = (0..24).collect();
        let mut source_targets = vec![None; tokens.len()];
        source_targets[20] = Some(0);
        let episode = TrainingEpisode {
            name: "coarse-freeze-integration".into(),
            source_id: 17,
            token_bytes: vec![vec![b'x']; tokens.len()],
            tokens,
            source_targets,
            prompt_len: Some(20),
        };
        let metrics = trainer.fit_episode(&episode, FitConfig::default())?;
        assert_eq!(metrics.source_committed, 1);
        assert_eq!(metrics.contrastive_updates, 0);
        assert_eq!(coarse(trainer.encoder.model()), before_coarse);
        assert_ne!(trainer.output.model().coefficients(), before_output);
        Ok(())
    }

    use super::*;
    #[test]
    fn one_token_copy_generate_marginalization_normalizes() {
        let head = SparseOutput::new(8, 16, 2).unwrap();
        let features = [1, 7];
        let token_mass: f64 = (0..8)
            .map(|v| (-token_nll_offline(&head, &features, Some(3), v).unwrap()).exp())
            .sum();
        let stop = (-head
            .score_action(&features, true, Action::Stop)
            .unwrap()
            .action_ce_offline())
        .exp();
        assert!((token_mass + stop - 1.0).abs() < 1e-12);
    }
    #[test]
    fn loaded_bundle_and_snapshot_replay_same_causal_prediction() {
        let config = ModelConfig::pilot(8, 1).unwrap();
        let mut model = IntegratedModel::new(config, AlgebraKind::Cyclic120, 11).unwrap();
        model.binding = ArtifactBinding {
            source_commit: "unit-test fixture".into(),
            tokenizer_sha256: "0".repeat(64),
            training_data_blake3: "0".repeat(64),
            credit_assignment: "untrained fixture".into(),
            ..Default::default()
        };
        let bytes = model.to_bytes().unwrap();
        let loaded = IntegratedModel::from_bytes(&bytes).unwrap();
        let mut session = Session::new(loaded.runtime(), 9).unwrap();
        for token in [1, 2, 1, 3] {
            session
                .observe(loaded.runtime(), token, &[b'a' + token as u8], None)
                .unwrap();
        }
        let read = session.read(loaded.runtime(), true).unwrap();
        let before = session.output(loaded.runtime(), &read, None).unwrap();
        let resumed = Session::restore(loaded.runtime(), session.snapshot()).unwrap();
        let read = resumed.read(loaded.runtime(), true).unwrap();
        assert_eq!(
            before,
            resumed.output(loaded.runtime(), &read, None).unwrap()
        );
        assert_eq!(resumed.memory.retained_event_count(), 4);
        let mut corrupt = bytes;
        corrupt[50] ^= 1;
        assert!(IntegratedModel::from_bytes(&corrupt).is_err());
    }
}
