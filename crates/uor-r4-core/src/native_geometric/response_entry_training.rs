//! Offline response entry and continuation over actual native model state.
//! Raw targets supervise token scores; they never create a serving anchor.
use super::completion_training::{fit_sparse_frames, top_counts, Frame, ValueCompletionFitConfig};
use super::completion_types::CompletionWork;
use super::response_entry_types::*;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseEntryFitConfig {
    pub epochs: usize,
    pub learning_rate: f64,
    pub max_positions: usize,
}
impl Default for ResponseEntryFitConfig {
    fn default() -> Self {
        Self {
            epochs: 24,
            learning_rate: 0.1,
            max_positions: RESPONSE_ENTRY_POSITIONS,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEntryFitReport {
    pub schema: String,
    pub baseline_artifact: String,
    pub artifact_cid: String,
    pub examples: usize,
    pub numeric_examples: usize,
    pub matched_numeric: usize,
    pub upstream_failures: usize,
    pub no_write_examples: usize,
    pub unexpected_value_writes: usize,
    pub overlong_responses: usize,
    pub position_limit_skips: usize,
    pub entry_positions: usize,
    pub entry_target_in_candidates: usize,
    pub entry_fit_correct: usize,
    pub entered_rollouts: usize,
    pub entry_rollout_failures: usize,
    pub continuation_positions: usize,
    pub continuation_target_in_candidates: usize,
    pub continuation_fit_correct: usize,
    pub final_entry_correct: usize,
    pub final_exact_responses: usize,
    pub learned_rows: usize,
    pub learned_associations: usize,
    pub dropped_row_events: usize,
    pub dropped_association_events: usize,
    pub entry_fit_loss: f64,
    pub continuation_fit_loss: Option<f64>,
    pub selected_entry_epoch: usize,
    pub selected_continuation_epoch: usize,
    pub config: ResponseEntryFitConfig,
    pub tokenization_law: String,
}

impl ResponseEntryModel {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let associations = self.rows.iter().map(|row| row.scores.len()).sum::<usize>();
        let valid_token =
            |token: u32| token != BOS && (token as usize) < model.geometry.tokens.len();
        if self.schema != RESPONSE_ENTRY_SCHEMA
            || model.completion.is_none()
            || self.rows.len() > RESPONSE_ENTRY_ROWS
            || associations > RESPONSE_ENTRY_ASSOCIATIONS
            || self.global_postings.len() > RESPONSE_ENTRY_CANDIDATES
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
                row.feature.kind >= (RESPONSE_ENTRY_FEATURES * 2) as u8
                    || row.default_score != 0
                    || row.postings.len() > RESPONSE_ENTRY_POSTINGS
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
            || self.fit_positions > RESPONSE_ENTRY_POSITIONS
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
                    || !(1..=RESPONSE_ENTRY_POSITIONS as u64).contains(&self.fit_config[2])
                    || self.fit_config[3] == 0
                    || self.fit_config[3] > self.fit_config[0]
                    || self.fit_config[4] > self.fit_config[0]))
        {
            return Err(Error("invalid learned response-entry artifact".into()));
        }
        let mut baseline = model.clone();
        baseline.response_entry = None;
        baseline.refresh_identity()?;
        if baseline.artifact_cid != self.baseline_artifact {
            return Err(Error(
                "response-entry artifact differs from its frozen completion baseline".into(),
            ));
        }
        Ok(())
    }
}

fn response_session(model: &Model, prompt: &str) -> Result<Session> {
    let mut session = model.session(Control::Full)?;
    session.observe(model, BOS)?;
    for token in model.encode(prompt)? {
        session.observe(model, token)?;
    }
    session.begin_response(model)?;
    Ok(session)
}

fn frame(session: &Session, model: &Model, target: u32, baseline: u32) -> Result<Frame> {
    let state = session
        .response_entry
        .as_ref()
        .ok_or_else(|| Error("response entry state absent during fit".into()))?;
    let values = session
        .values
        .as_ref()
        .ok_or_else(|| Error("typed state absent during entry fit".into()))?;
    let (features, len) =
        state.features(model, values, Control::Full, &mut CompletionWork::default());
    if len == 0 {
        return Err(Error("response entry frame has no causal boundary".into()));
    }
    Ok(Frame {
        features,
        len,
        target,
        baseline,
    })
}

impl Model {
    pub fn response_entry_version(&self) -> Option<&str> {
        self.response_entry
            .as_ref()
            .map(|head| head.schema.as_str())
    }
    pub fn response_entry_training(&self) -> &[DocumentReceipt] {
        self.response_entry
            .as_ref()
            .map_or(&[], |head| head.training.as_slice())
    }

    pub fn fit_response_entry(
        &self,
        documents: &[ValueExample],
        config: ResponseEntryFitConfig,
    ) -> Result<(Model, ResponseEntryFitReport)> {
        let source_bytes = documents.iter().try_fold(0_usize, |sum, document| {
            sum.checked_add(document.prompt.len())?
                .checked_add(document.response.len())
        });
        if self.response_entry.is_some()
            || self.completion.is_none()
            || documents.is_empty()
            || documents.len() > 4096
            || source_bytes.is_none_or(|bytes| bytes > 16 * 1024 * 1024)
            || !(1..=64).contains(&config.epochs)
            || !config.learning_rate.is_finite()
            || !(0.0001..=1.0).contains(&config.learning_rate)
            || !(1..=RESPONSE_ENTRY_POSITIONS).contains(&config.max_positions)
        {
            return Err(Error(
                "invalid response-entry source, configuration or baseline".into(),
            ));
        }
        let mut model = self.clone();
        model.response_entry = Some(ResponseEntryModel {
            schema: RESPONSE_ENTRY_SCHEMA.into(),
            baseline_artifact: self.artifact_cid.clone(),
            rows: Vec::new(),
            global_postings: Vec::new(),
            fit_config: [0; 5],
            fit_positions: 0,
            training: Vec::new(),
        });
        model.refresh_identity()?;
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut prompts = BTreeMap::new();
        let mut entry_frames = Vec::new();
        let mut admitted = Vec::new();
        let mut reserved_positions = 0_usize;
        let mut numeric_examples = 0;
        let mut matched_numeric = 0;
        let mut upstream_failures = 0;
        let mut no_write_examples = 0;
        let mut unexpected_value_writes = 0;
        let mut overlong_responses = 0;
        let mut position_limit_skips = 0;
        for (index, document) in documents.iter().enumerate() {
            if document.id.trim().is_empty()
                || !ids.insert(&document.id)
                || prompts
                    .insert(&document.prompt, &document.response)
                    .is_some_and(|known| known != &document.response)
            {
                return Err(Error(
                    "response-entry source IDs or raw targets conflict".into(),
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
                    "entry source overlaps ordinary model construction".into(),
                ));
            }
            receipts.push(receipt);
            // Classification is training-only raw numeral syntax. All supplied
            // numeric examples are checked through the actual frozen full path.
            let numeric = document
                .response
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_digit() || matches!(*byte, b'+' | b'-'));
            if numeric {
                numeric_examples += 1;
                let generated = self.generate(&document.prompt, 64, Control::Full)?;
                if generated.bytes == document.response.as_bytes() {
                    matched_numeric += 1;
                } else {
                    upstream_failures += 1;
                }
                continue;
            }
            no_write_examples += 1;
            let mut session = response_session(&model, &document.prompt)?;
            session.predict(&model)?;
            if session.value_decision().is_some() {
                unexpected_value_writes += 1;
                continue;
            }
            let mut targets = model.encode(&document.response)?;
            targets.push(EOS);
            if targets.len() > usize::from(RESPONSE_ENTRY_STEPS) {
                overlong_responses += 1;
                continue;
            }
            if reserved_positions.saturating_add(targets.len()) > config.max_positions {
                position_limit_skips += 1;
                continue;
            }
            if session
                .response_entry
                .as_ref()
                .is_none_or(|state| state.boundary.is_none())
            {
                upstream_failures += 1;
                continue;
            }
            // Correct ordinary Base output must still learn Enter, because
            // observation of Base cannot create an entry anchor.
            entry_frames.push(frame(&session, &model, targets[0], u32::MAX)?);
            reserved_positions += targets.len();
            admitted.push((index, targets));
        }
        if entry_frames.is_empty() {
            return Err(Error(
                "entry fit has no bounded no-write response targets".into(),
            ));
        }
        let optimizer_config = ValueCompletionFitConfig {
            epochs: config.epochs,
            learning_rate: config.learning_rate,
            max_positions: config.max_positions,
        };
        let entry_fit = fit_sparse_frames(
            &entry_frames,
            optimizer_config,
            RESPONSE_ENTRY_ROWS,
            RESPONSE_ENTRY_ASSOCIATIONS,
            0,
        )?;
        let entry_row_count = entry_fit.rows.len();
        let entry_association_count = entry_fit.learned_associations;
        let head = model
            .response_entry
            .as_mut()
            .ok_or_else(|| Error("entry component missing".into()))?;
        head.rows = entry_fit.rows;
        head.global_postings = entry_fit.global_postings;
        model.refresh_identity()?;
        let mut continuation_frames = Vec::new();
        let mut entered_rollouts = 0;
        let mut entry_rollout_failures = 0;
        for (index, targets) in &admitted {
            let mut session = response_session(&model, &documents[*index].prompt)?;
            let prediction = session.predict(&model)?;
            let entered = session.response_entry_decision().is_some_and(|decision| {
                decision.action == ResponseEntryAction::Enter && decision.token == targets[0]
            });
            if prediction.token != targets[0] || !entered {
                entry_rollout_failures += 1;
                continue;
            }
            session.observe(&model, prediction.token)?;
            entered_rollouts += 1;
            for &target in &targets[1..] {
                let baseline = session.predict(&model)?.token;
                if session
                    .response_entry
                    .as_ref()
                    .is_none_or(|state| !state.active)
                {
                    return Err(Error(
                        "entry anchor ended before whole training response".into(),
                    ));
                }
                continuation_frames.push(frame(&session, &model, target, baseline)?);
                // Teacher forcing starts only after a real selected Enter was
                // observed. Later mismatches update actual state without hidden
                // target provenance; all targets and EOS stay trainer-side.
                session.observe(&model, target)?;
            }
        }
        let continuation_fit = if continuation_frames.is_empty() {
            None
        } else {
            Some(fit_sparse_frames(
                &continuation_frames,
                optimizer_config,
                RESPONSE_ENTRY_ROWS.saturating_sub(entry_row_count),
                RESPONSE_ENTRY_ASSOCIATIONS.saturating_sub(entry_association_count),
                RESPONSE_ENTRY_FEATURES as u8,
            )?)
        };
        let mut globals = BTreeMap::<u32, u64>::new();
        for item in entry_frames.iter().chain(&continuation_frames) {
            *globals.entry(item.target).or_default() += 1;
        }
        let head = model
            .response_entry
            .as_mut()
            .ok_or_else(|| Error("entry component missing".into()))?;
        if let Some(fit) = &continuation_fit {
            head.rows.extend(fit.rows.clone());
        }
        head.rows.sort_by_key(|row| row.feature);
        head.global_postings = top_counts(&globals, RESPONSE_ENTRY_CANDIDATES);
        head.fit_positions = entry_frames.len() + continuation_frames.len();
        head.fit_config = [
            config.epochs as u64,
            config.learning_rate.to_bits(),
            config.max_positions as u64,
            entry_fit.selected_epoch as u64,
            continuation_fit
                .as_ref()
                .map_or(0, |fit| fit.selected_epoch as u64),
        ];
        head.training = receipts;
        let learned_rows = head.rows.len();
        let learned_associations = head.rows.iter().map(|row| row.scores.len()).sum();
        model.refresh_identity()?;
        model.validate()?;
        // Shared global postings can change admission. Recheck actual entry
        // decisions and complete free-running bytes using the exported model.
        let mut final_entry_correct = 0;
        let mut final_exact_responses = 0;
        for (index, targets) in &admitted {
            let document = &documents[*index];
            let mut session = response_session(&model, &document.prompt)?;
            let prediction = session.predict(&model)?;
            final_entry_correct += usize::from(
                prediction.token == targets[0]
                    && session
                        .response_entry_decision()
                        .is_some_and(|decision| decision.action == ResponseEntryAction::Enter),
            );
            let output = model.generate(
                &document.prompt,
                usize::from(RESPONSE_ENTRY_STEPS),
                Control::Full,
            )?;
            final_exact_responses += usize::from(output.bytes == document.response.as_bytes());
        }
        let report = ResponseEntryFitReport {
            schema: RESPONSE_ENTRY_SCHEMA.into(), baseline_artifact: self.artifact_cid.clone(), artifact_cid: model.artifact_cid.clone(), examples: documents.len(),
            numeric_examples, matched_numeric, upstream_failures, no_write_examples, unexpected_value_writes, overlong_responses, position_limit_skips,
            entry_positions: entry_frames.len(), entry_target_in_candidates: entry_fit.target_in_candidates, entry_fit_correct: entry_fit.correct,
            entered_rollouts, entry_rollout_failures, continuation_positions: continuation_frames.len(),
            continuation_target_in_candidates: continuation_fit.as_ref().map_or(0, |fit| fit.target_in_candidates),
            continuation_fit_correct: continuation_fit.as_ref().map_or(0, |fit| fit.correct), final_entry_correct, final_exact_responses,
            learned_rows, learned_associations,
            dropped_row_events: entry_fit.dropped_row_events + continuation_fit.as_ref().map_or(0, |fit| fit.dropped_row_events),
            dropped_association_events: entry_fit.dropped_association_events + continuation_fit.as_ref().map_or(0, |fit| fit.dropped_association_events),
            entry_fit_loss: entry_fit.loss, continuation_fit_loss: continuation_fit.as_ref().map(|fit| fit.loss),
            selected_entry_epoch: entry_fit.selected_epoch, selected_continuation_epoch: continuation_fit.as_ref().map_or(0, |fit| fit.selected_epoch), config,
            tokenization_law: "Canonical frozen model.encode(response) plus EOS; an overlong response is skipped whole. Entry action is supervised separately from equal-token Base; continuation frames require actual quantized Enter selection and observation. Final shared-posting entry and free-running response checks are reported.".into(),
        };
        Ok((model, report))
    }
}
