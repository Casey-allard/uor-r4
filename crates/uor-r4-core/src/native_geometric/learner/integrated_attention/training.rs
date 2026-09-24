//! Offline coordinated learning on the same hard causal path used by export.
//!
//! Credit is explicitly truncated: output/gates/energy use local CE, source
//! annotations supervise query/key alignment, and state actions receive a
//! one-step expected language-loss surrogate. There is no claim of BPTT or a
//! solved hard-routing optimization problem. Labels never enter Runtime.
use super::*;
use encoder::{EncoderInput, EncoderTrainer};
use output::OutputTrainer;
use training_support::{EnergyTrainer, SparseGateTrainer};

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

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct FitMetrics {
    pub positions: u64,
    pub surface_token_nll_nats: f64,
    pub labeled_sources: u64,
    pub source_admitted: u64,
    pub source_selected: u64,
    pub selected_reads: u64,
    pub correct_selected_tokens: u64,
    pub writes: u64,
    pub query_key_updates: u64,
    pub state_lane_updates: u64,
    pub energy_updates: u64,
    pub access: AccessCounts,
}
impl FitMetrics {
    pub fn add(&mut self, other: &Self) {
        self.positions += other.positions;
        self.surface_token_nll_nats += other.surface_token_nll_nats;
        self.labeled_sources += other.labeled_sources;
        self.source_admitted += other.source_admitted;
        self.source_selected += other.source_selected;
        self.selected_reads += other.selected_reads;
        self.correct_selected_tokens += other.correct_selected_tokens;
        self.writes += other.writes;
        self.query_key_updates += other.query_key_updates;
        self.state_lane_updates += other.state_lane_updates;
        self.energy_updates += other.energy_updates;
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
    key_input: EncoderInput,
    key: [u8; 16],
    record_id: Option<u64>,
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
        })
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
            digest: [0; 32],
        }
    }
    pub fn export(self) -> IntegratedModel {
        IntegratedModel {
            version: 1,
            config: self.config,
            binding: self.binding,
            algebra: self.algebra,
            encoder: self.encoder.export(),
            energy: self.energy.export(),
            output: self.output.export(),
            read_gate: self.read_gate.export(),
            write_gate: self.write_gate.export(),
            digest: [0; 32],
        }
    }
    fn frozen_episode_model(&self) -> IntegratedModel {
        IntegratedModel {
            version: 1,
            config: self.config.clone(),
            binding: self.binding.clone(),
            algebra: self.algebra.clone(),
            encoder: self.encoder.model().clone(),
            energy: self.energy.model().clone(),
            output: self.output.model().clone(),
            read_gate: self.read_gate.model().clone(),
            write_gate: self.write_gate.model().clone(),
            digest: [0; 32],
        }
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
        let mut session = Session::new(frozen.runtime(), episode.source_id)?;
        let mut observed = Vec::<ObservedTraining>::with_capacity(episode.tokens.len());
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
            if let Some(source_position) = episode.source_targets[position] {
                let source = observed[source_position];
                metrics.labeled_sources += 1;
                let admitted = source.record_id.and_then(|id| {
                    read.candidates[..read.candidate_count]
                        .iter()
                        .position(|c| c.record_id == id)
                });
                metrics.source_admitted += u64::from(admitted.is_some());
                metrics.source_selected +=
                    u64::from(admitted.is_some() && admitted == read.selected);
                self.encoder.supervise(
                    read.input,
                    &source.key[..self.config.lanes],
                    fit.encoder_rate,
                )?;
                self.encoder.supervise(
                    source.key_input,
                    &read.query[..self.config.lanes],
                    fit.encoder_rate * 0.1,
                )?;
                metrics.query_key_updates += 2;
                // Source insertion supplies fit-only credit even if a hard page was missed.
                // Export evaluation receives neither this source nor the source annotation.
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
                        &source.key[..self.config.lanes],
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
            // Missing annotations are not evidence that reading is useless. A negative
            // gate label is used only when an actual selected read worsens token loss.
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
                let useful = if episode.source_targets[position].is_some() {
                    right_source
                } else {
                    candidate_loss < no_read_loss
                };
                self.read_gate
                    .train(candidate_features.as_slice(), useful, fit.gate_rate)?;
            }
            self.output
                .train_token(feat.as_slice(), selected, target, fit.output_rate)?;
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
            if future_sources[position] {
                self.write_gate
                    .train(event.gate_features.as_slice(), true, fit.gate_rate)?;
            }
            metrics.writes += u64::from(event.record_id.is_some());
            metrics.access.add(event.access);
            observed.push(ObservedTraining {
                key_input: event.key_input,
                key: event.key,
                record_id: event.record_id,
            });
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
            let admitted = source_records[source].and_then(|id| {
                read.candidates[..read.candidate_count]
                    .iter()
                    .position(|c| c.record_id == id)
            });
            metrics.source_admitted += u64::from(admitted.is_some());
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
        source_records.push(event.record_id);
    }
    Ok(metrics)
}

#[cfg(test)]
mod tests {
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
