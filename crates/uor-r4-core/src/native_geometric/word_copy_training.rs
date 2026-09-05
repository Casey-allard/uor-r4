//! Offline latent occurrence fitting. Targets label complete retained spellings;
//! neither target indices nor response buffers enter the serving artifact.
use super::completion_training::{fit_sparse_frames, Frame, ValueCompletionFitConfig};
use super::completion_types::CompletionWork;
use super::response_entry_types::*;
use super::value_lexemes::{LexemeState, WordAtom, WORD_BYTES, WORD_QUERY};
use super::value_types::{ValueEntry, ValueFeature, ValueRow, ValueWork, LEXEME_VALUE_SCHEMA};
use super::word_copy_runtime;
use super::word_copy_types::*;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

fn source_error(error: impl std::fmt::Display) -> Error {
    Error(error.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEntryCopyFitReport {
    pub schema: String,
    pub baseline_artifact: String,
    pub artifact_cid: String,
    pub examples: usize,
    pub numeric_examples: usize,
    pub matched_numeric: usize,
    pub upstream_failures: usize,
    pub eligible_examples: usize,
    pub copy_targets: usize,
    pub reachable_copy_targets: usize,
    pub no_copy_targets: usize,
    pub unreachable_targets: usize,
    pub overlong_responses: usize,
    pub position_limit_skips: usize,
    pub dictionary_words: usize,
    pub dictionary_omitted_words: usize,
    pub dictionary_omitted_occurrences: u64,
    pub learned_features: usize,
    pub dropped_feature_events: usize,
    pub selector_fit_correct: usize,
    pub selector_fit_loss: f64,
    pub selected_selector_epoch: usize,
    pub selected_copies: usize,
    pub committed_complete_copies: usize,
    pub copy_rollout_failures: usize,
    pub false_copies: usize,
    pub continuation_positions: usize,
    pub continuation_target_in_candidates: usize,
    pub continuation_fit_correct: usize,
    pub continuation_fit_loss: Option<f64>,
    pub selected_continuation_epoch: usize,
    pub continuation_rows: usize,
    pub continuation_associations: usize,
    pub dropped_row_events: usize,
    pub dropped_association_events: usize,
    pub final_copy_correct: usize,
    pub final_no_copy_correct: usize,
    pub final_exact_responses: usize,
    pub config: ResponseEntryFitConfig,
    pub target_law: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suffix_law: Option<String>,
}

struct Alternative {
    features: Vec<usize>,
    correct: bool,
}
struct Example {
    document: usize,
    alternatives: Vec<Alternative>,
    baseline_score: f64,
    baseline_correct: bool,
    prefix_len: Option<usize>,
}

fn identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn target_prefix(response: &str) -> Option<usize> {
    let bytes = response.as_bytes();
    if !bytes
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
    {
        return None;
    }
    let len = bytes
        .iter()
        .take_while(|&&byte| identifier_byte(byte))
        .count();
    // The source scanner rejects an entire run containing non-ASCII bytes;
    // do not label its shorter ASCII prefix as a complete retained word.
    (!bytes.get(len).is_some_and(|byte| !byte.is_ascii())).then_some(len)
}

fn matches_target(word: &WordAtom, response: &str) -> bool {
    let len = usize::from(word.len);
    len != 0
        && target_prefix(response) == Some(len)
        && response.as_bytes().starts_with(&word.bytes[..len])
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

/// The exact existing scanner supplies dictionary words, including words that
/// later leave the sixteen-word ring. Only construction prompts contribute.
fn dictionary(documents: &[ValueExample]) -> Result<(Vec<WordCopyAddress>, usize, u64)> {
    let mut counts = BTreeMap::<Vec<u8>, u64>::new();
    for document in documents {
        let mut state = LexemeState::default();
        for &byte in document.prompt.as_bytes() {
            let prior = state.recent[0];
            state.feed(byte, ValueEntry::default(), &mut ValueWork::default());
            if state.recent[0] != prior {
                let word = state.recent[0];
                *counts
                    .entry(word.bytes[..usize::from(word.len)].to_vec())
                    .or_default() += 1;
            }
        }
        let prior = state.recent[0];
        state.finish(&mut ValueWork::default());
        if state.recent[0] != prior {
            let word = state.recent[0];
            *counts
                .entry(word.bytes[..usize::from(word.len)].to_vec())
                .or_default() += 1;
        }
    }
    let mut ranked: Vec<_> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let omitted_words = ranked.len().saturating_sub(WORD_COPY_DICTIONARY);
    let omitted_occurrences = ranked
        .iter()
        .skip(WORD_COPY_DICTIONARY)
        .map(|(_, n)| n)
        .sum();
    ranked.truncate(WORD_COPY_DICTIONARY);
    ranked.sort_by(|a, b| a.0.cmp(&b.0));
    let primes =
        crate::corpus_induced_spin_placement::first_primes(ranked.len()).map_err(source_error)?;
    let words = ranked
        .into_iter()
        .zip(primes)
        .map(|((word, _), prime)| {
            let mut bytes = [0; WORD_BYTES];
            bytes[..word.len()].copy_from_slice(&word);
            Ok(WordCopyAddress {
                bytes,
                len: word.len() as u8,
                prime: u32::try_from(prime).map_err(source_error)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((words, omitted_words, omitted_occurrences))
}

impl WordCopyModel {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        if model
            .values
            .as_ref()
            .is_none_or(|values| values.schema != LEXEME_VALUE_SCHEMA)
        {
            return Err(Error(
                "retained-word copying requires the whole-word typed-value schema".into(),
            ));
        }
        if self.dictionary.len() > WORD_COPY_DICTIONARY {
            return Err(Error(
                "retained-word dictionary exceeds its fixed capacity".into(),
            ));
        }
        let valid_token =
            |token: u32| token != BOS && (token as usize) < model.geometry.tokens.len();
        let primes = crate::corpus_induced_spin_placement::first_primes(self.dictionary.len())
            .map_err(source_error)?;
        if self.dictionary.len() > WORD_COPY_DICTIONARY
            || self.dictionary.iter().zip(primes).any(|(word, prime)| {
                let len = usize::from(word.len);
                len == 0
                    || len > WORD_BYTES
                    || u64::from(word.prime) != prime
                    || !(word.bytes[0].is_ascii_alphabetic() || word.bytes[0] == b'_')
                    || !word.bytes[..len].iter().all(|&byte| identifier_byte(byte))
                    || word.bytes[len..].iter().any(|&byte| byte != 0)
            })
            || self.dictionary.windows(2).any(|pair| {
                pair[0].bytes[..usize::from(pair[0].len)]
                    >= pair[1].bytes[..usize::from(pair[1].len)]
            })
            || self.rows.len() > WORD_COPY_ROWS
            || self
                .rows
                .windows(2)
                .any(|pair| pair[0].feature >= pair[1].feature)
            || self.rows.iter().any(|row| {
                row.feature.kind >= WORD_COPY_FEATURES as u8
                    || !(-1_000_000..=1_000_000).contains(&row.weight)
            })
            || self.continuation_rows.len() > RESPONSE_ENTRY_ROWS
            || self
                .continuation_rows
                .iter()
                .map(|row| row.scores.len())
                .sum::<usize>()
                > RESPONSE_ENTRY_ASSOCIATIONS
            || self.continuation_postings.len() > RESPONSE_ENTRY_CANDIDATES
            || self
                .continuation_postings
                .iter()
                .any(|&token| !valid_token(token))
            || self
                .continuation_postings
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                != self.continuation_postings.len()
            || self
                .continuation_rows
                .windows(2)
                .any(|pair| pair[0].feature >= pair[1].feature)
            || self.continuation_rows.iter().any(|row| {
                !(16..32).contains(&row.feature.kind)
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
            return Err(Error("invalid retained-word copy artifact".into()));
        }
        let mut parent = model.clone();
        let entry = parent
            .response_entry
            .as_mut()
            .ok_or_else(|| Error("copy parent entry missing".into()))?;
        entry.copy = None;
        entry.schema = RESPONSE_ENTRY_SCHEMA.into();
        parent.refresh_identity()?;
        if parent.artifact_cid != self.baseline_artifact {
            return Err(Error(
                "copy extension differs from its frozen response-entry parent".into(),
            ));
        }
        Ok(())
    }
}

impl Model {
    pub fn word_copy_version(&self) -> Option<&str> {
        self.response_entry
            .as_ref()
            .filter(|entry| entry.copy.is_some())
            .map(|entry| entry.schema.as_str())
    }

    pub fn word_copy_training(&self) -> &[DocumentReceipt] {
        self.response_entry
            .as_ref()
            .and_then(|entry| entry.copy.as_ref())
            .map_or(&[], |copy| copy.training.as_slice())
    }

    pub fn fit_response_entry_copy(
        &self,
        documents: &[ValueExample],
        config: ResponseEntryFitConfig,
    ) -> Result<(Model, ResponseEntryCopyFitReport)> {
        self.fit_response_entry_copy_impl(documents, config, false)
    }

    /// Fit suffix transitions relative to the observed end of a copied word.
    /// Initial occurrence construction, selection and quantization are the
    /// same as `fit_response_entry_copy`; no selector refit rule is changed.
    pub fn fit_response_entry_copy_completed_word(
        &self,
        documents: &[ValueExample],
        config: ResponseEntryFitConfig,
    ) -> Result<(Model, ResponseEntryCopyFitReport)> {
        self.fit_response_entry_copy_impl(documents, config, true)
    }

    fn fit_response_entry_copy_impl(
        &self,
        documents: &[ValueExample],
        config: ResponseEntryFitConfig,
        completed_word_suffix: bool,
    ) -> Result<(Model, ResponseEntryCopyFitReport)> {
        let source_bytes = documents.iter().try_fold(0_usize, |sum, document| {
            sum.checked_add(document.prompt.len())?
                .checked_add(document.response.len())
        });
        if self
            .response_entry
            .as_ref()
            .is_none_or(|entry| entry.schema != RESPONSE_ENTRY_SCHEMA || entry.copy.is_some())
            || self
                .values
                .as_ref()
                .is_none_or(|values| values.schema != LEXEME_VALUE_SCHEMA)
            || documents.is_empty()
            || documents.len() > 4096
            || source_bytes.is_none_or(|bytes| bytes > 16 * 1024 * 1024)
            || !(1..=64).contains(&config.epochs)
            || !config.learning_rate.is_finite()
            || !(0.0001..=1.0).contains(&config.learning_rate)
            || !(1..=RESPONSE_ENTRY_POSITIONS).contains(&config.max_positions)
        {
            return Err(Error(
                "invalid retained-word fitting source, configuration or entry parent".into(),
            ));
        }
        let (dictionary, omitted_words, omitted_occurrences) = dictionary(documents)?;
        let dictionary_words = dictionary.len();
        let mut model = self.clone();
        let entry = model
            .response_entry
            .as_mut()
            .ok_or_else(|| Error("entry parent missing".into()))?;
        entry.schema = RESPONSE_COPY_SCHEMA.into();
        entry.copy = Some(WordCopyModel {
            completed_word_suffix,
            baseline_artifact: self.artifact_cid.clone(),
            dictionary,
            rows: Vec::new(),
            continuation_rows: Vec::new(),
            continuation_postings: Vec::new(),
            fit_config: [0; 5],
            fit_positions: 0,
            training: Vec::new(),
        });
        model.refresh_identity()?;
        let mut ids = BTreeSet::new();
        let mut prompts = BTreeMap::new();
        let mut receipts = Vec::new();
        let mut registry = BTreeMap::<ValueFeature, usize>::new();
        let mut weights = Vec::new();
        let mut examples = Vec::new();
        let mut reserved_positions = 0_usize;
        let mut numeric_examples = 0;
        let mut matched_numeric = 0;
        let mut upstream_failures = 0;
        let mut copy_targets = 0;
        let mut no_copy_targets = 0;
        let mut unreachable_targets = 0;
        let mut overlong_responses = 0;
        let mut position_limit_skips = 0;
        let mut dropped_feature_events = 0;
        for (index, document) in documents.iter().enumerate() {
            if document.id.trim().is_empty()
                || !ids.insert(&document.id)
                || prompts
                    .insert(&document.prompt, &document.response)
                    .is_some_and(|prior| prior != &document.response)
            {
                return Err(Error(
                    "retained-word source IDs or raw targets conflict".into(),
                ));
            }
            let receipt = super::training::receipt(&Document {
                id: document.id.clone(),
                text: serde_json::to_string(&(&document.prompt, &document.response))
                    .map_err(source_error)?,
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
                    "retained-word source overlaps ordinary construction".into(),
                ));
            }
            receipts.push(receipt);
            let numeric = document
                .response
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_digit() || matches!(*byte, b'+' | b'-'));
            if numeric {
                numeric_examples += 1;
                if self.generate(&document.prompt, 64, Control::Full)?.bytes
                    == document.response.as_bytes()
                {
                    matched_numeric += 1;
                } else {
                    upstream_failures += 1;
                }
                continue;
            }
            let prefix = target_prefix(&document.response);
            copy_targets += usize::from(prefix.is_some());
            no_copy_targets += usize::from(prefix.is_none());
            let mut session = response_session(&model, &document.prompt)?;
            session.predict(&model)?;
            let values = session
                .values
                .as_ref()
                .ok_or_else(|| Error("copy fit typed state missing".into()))?;
            let entry = session
                .response_entry
                .as_ref()
                .ok_or_else(|| Error("copy fit entry state missing".into()))?;
            if !word_copy_runtime::eligible(entry, values, Control::Full) {
                upstream_failures += 1;
                continue;
            }
            let positions = if let Some(prefix_len) = prefix {
                prefix_len
                    .saturating_add(model.encode(&document.response[prefix_len..])?.len())
                    .saturating_add(1)
            } else {
                1
            };
            if positions > usize::from(RESPONSE_ENTRY_STEPS) {
                overlong_responses += 1;
                continue;
            }
            if reserved_positions.saturating_add(positions) > config.max_positions {
                position_limit_skips += 1;
                continue;
            }
            let mut lexical = *entry;
            let threshold = lexical
                .offer(
                    &model,
                    values,
                    Candidate {
                        token: BOS,
                        score: 0,
                    },
                    Control::Full,
                    &mut CompletionWork::default(),
                )
                .map_or(0, |candidate| candidate.score);
            let context = word_copy_runtime::context(
                &model,
                values,
                Control::Full,
                &mut WordCopyWork::default(),
            );
            let mut alternatives = Vec::new();
            let mut reachable = false;
            let words = values
                .lexemes
                .as_ref()
                .ok_or_else(|| Error("copy fit word state missing".into()))?;
            for word_index in 0..words.query_len.min(WORD_QUERY) {
                let word = words.queries[word_index];
                if word.len == 0 || usize::from(word.len) + 1 > usize::from(RESPONSE_ENTRY_STEPS) {
                    continue;
                }
                let correct = matches_target(&word, &document.response);
                reachable |= correct;
                let (features, len) = word_copy_runtime::features(
                    &model,
                    values,
                    &context,
                    word_index,
                    Control::Full,
                    &mut WordCopyWork::default(),
                );
                let mut indices = Vec::with_capacity(len);
                for feature in &features[..len] {
                    let known = registry.get(feature).copied();
                    let feature_index = if known.is_some() {
                        known
                    } else if weights.len() < WORD_COPY_ROWS {
                        let next = weights.len();
                        registry.insert(*feature, next);
                        weights.push(if feature.kind == 0 { -2.0 } else { 0.0 });
                        Some(next)
                    } else {
                        dropped_feature_events += 1;
                        None
                    };
                    if let Some(feature_index) = feature_index {
                        indices.push(feature_index);
                    }
                }
                alternatives.push(Alternative {
                    features: indices,
                    correct,
                });
            }
            if prefix.is_some() && !reachable {
                unreachable_targets += 1;
                continue;
            }
            reserved_positions += positions;
            examples.push(Example {
                document: index,
                alternatives,
                baseline_score: threshold as f64 / 256.0,
                baseline_correct: prefix.is_none(),
                prefix_len: prefix,
            });
        }
        if examples.is_empty() || !examples.iter().any(|example| !example.baseline_correct) {
            return Err(Error(
                "copy source has no reachable complete retained-word target".into(),
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
            let quantized: Vec<_> = weights
                .iter()
                .map(|weight| (weight * 256.0).round() / 256.0)
                .collect();
            let (correct, loss) = measure(&examples, &quantized)?;
            if correct > best_correct || (correct == best_correct && loss < best_loss) {
                best_correct = correct;
                best_loss = loss;
                selected_epoch = epoch + 1;
                best_weights = quantized;
            }
        }
        copy_mut(&mut model)?.rows = registry
            .into_iter()
            .map(|(feature, index)| ValueRow {
                feature,
                weight: (best_weights[index] * 256.0).round() as i32,
            })
            .collect();
        model.refresh_identity()?;
        let mut continuation_frames = Vec::new();
        let mut selected_copies = 0;
        let mut committed_complete_copies = 0;
        let mut copy_rollout_failures = 0;
        let mut false_copies = 0;
        for example in &examples {
            let document = &documents[example.document];
            let mut session = response_session(&model, &document.prompt)?;
            let first = session.predict(&model)?;
            let Some(prefix_len) = example.prefix_len else {
                false_copies += usize::from(
                    session
                        .word_copy_decision()
                        .is_some_and(|decision| decision.action == WordCopyAction::Start),
                );
                continue;
            };
            let selected = session.word_copy_decision().is_some_and(|decision| {
                decision.action == WordCopyAction::Start
                    && session
                        .values
                        .as_ref()
                        .and_then(|values| values.lexemes.as_ref())
                        .is_some_and(|words| {
                            usize::from(decision.word_index) < words.query_len
                                && matches_target(
                                    &words.queries[usize::from(decision.word_index)],
                                    &document.response,
                                )
                        })
            });
            if !selected || first.token != u32::from(document.response.as_bytes()[0]) + 2 {
                copy_rollout_failures += 1;
                continue;
            }
            selected_copies += 1;
            let mut complete = true;
            for (byte_index, &byte) in document.response.as_bytes()[..prefix_len]
                .iter()
                .enumerate()
            {
                let prediction = if byte_index == 0 {
                    first.clone()
                } else {
                    session.predict(&model)?
                };
                if prediction.token != u32::from(byte) + 2 {
                    complete = false;
                    break;
                }
                session.observe(&model, prediction.token)?;
            }
            if !complete
                || session
                    .word_copy
                    .as_ref()
                    .is_none_or(|state| state.progress != WordCopyProgress::Complete)
            {
                copy_rollout_failures += 1;
                continue;
            }
            committed_complete_copies += 1;
            let mut targets = model.encode(&document.response[prefix_len..])?;
            targets.push(EOS);
            for target in targets {
                let baseline = session.predict(&model)?.token;
                let entry = session
                    .response_entry
                    .as_ref()
                    .ok_or_else(|| Error("copy continuation entry missing".into()))?;
                let values = session
                    .values
                    .as_ref()
                    .ok_or_else(|| Error("copy continuation typed state missing".into()))?;
                if !entry.active {
                    return Err(Error(
                        "copy entry ended before complete suffix trajectory".into(),
                    ));
                }
                let copy = session
                    .word_copy
                    .as_ref()
                    .ok_or_else(|| Error("copy continuation origin missing".into()))?;
                let (features, len) = copy.continuation_features(
                    &model,
                    entry,
                    values,
                    Control::Full,
                    &mut WordCopyWork::default(),
                );
                if len == 0 {
                    return Err(Error(
                        "copy continuation features lack actual completed-word evidence".into(),
                    ));
                }
                continuation_frames.push(Frame {
                    features,
                    len,
                    target,
                    baseline,
                });
                session.observe(&model, target)?;
            }
        }
        let continuation_fit = if continuation_frames.is_empty() {
            None
        } else {
            Some(fit_sparse_frames(
                &continuation_frames,
                ValueCompletionFitConfig {
                    epochs: config.epochs,
                    learning_rate: config.learning_rate,
                    max_positions: config.max_positions,
                },
                RESPONSE_ENTRY_ROWS,
                RESPONSE_ENTRY_ASSOCIATIONS,
                RESPONSE_ENTRY_FEATURES as u8,
            )?)
        };
        let head = copy_mut(&mut model)?;
        if let Some(fit) = &continuation_fit {
            head.continuation_rows = fit.rows.clone();
            head.continuation_postings = fit.global_postings.clone();
        }
        head.fit_config = [
            config.epochs as u64,
            config.learning_rate.to_bits(),
            config.max_positions as u64,
            selected_epoch as u64,
            continuation_fit
                .as_ref()
                .map_or(0, |fit| fit.selected_epoch as u64),
        ];
        head.fit_positions = examples.len() + continuation_frames.len();
        head.training = receipts;
        let learned_features = head.rows.len();
        let continuation_rows = head.continuation_rows.len();
        let continuation_associations = head
            .continuation_rows
            .iter()
            .map(|row| row.scores.len())
            .sum();
        model.refresh_identity()?;
        model.validate()?;
        let mut final_copy_correct = 0;
        let mut final_no_copy_correct = 0;
        let mut final_exact_responses = 0;
        for example in &examples {
            let document = &documents[example.document];
            let mut session = response_session(&model, &document.prompt)?;
            session.predict(&model)?;
            let selected = session
                .word_copy_decision()
                .filter(|decision| decision.action == WordCopyAction::Start);
            if example.baseline_correct {
                final_no_copy_correct += usize::from(selected.is_none());
            } else if let Some(decision) = selected {
                final_copy_correct += usize::from(
                    session
                        .values
                        .as_ref()
                        .and_then(|values| values.lexemes.as_ref())
                        .is_some_and(|words| {
                            usize::from(decision.word_index) < words.query_len
                                && matches_target(
                                    &words.queries[usize::from(decision.word_index)],
                                    &document.response,
                                )
                        }),
                );
            }
            final_exact_responses += usize::from(
                model.generate(&document.prompt, 64, Control::Full)?.bytes
                    == document.response.as_bytes(),
            );
        }
        let report = ResponseEntryCopyFitReport {
            schema: RESPONSE_COPY_SCHEMA.into(), baseline_artifact: self.artifact_cid.clone(), artifact_cid: model.artifact_cid.clone(), examples: documents.len(), numeric_examples, matched_numeric, upstream_failures,
            eligible_examples: examples.len(), copy_targets, reachable_copy_targets: examples.iter().filter(|example| !example.baseline_correct).count(), no_copy_targets, unreachable_targets, overlong_responses, position_limit_skips,
            dictionary_words, dictionary_omitted_words: omitted_words, dictionary_omitted_occurrences: omitted_occurrences,
            learned_features, dropped_feature_events, selector_fit_correct: best_correct, selector_fit_loss: best_loss, selected_selector_epoch: selected_epoch,
            selected_copies, committed_complete_copies, copy_rollout_failures, false_copies, continuation_positions: continuation_frames.len(),
            continuation_target_in_candidates: continuation_fit.as_ref().map_or(0, |fit| fit.target_in_candidates), continuation_fit_correct: continuation_fit.as_ref().map_or(0, |fit| fit.correct), continuation_fit_loss: continuation_fit.as_ref().map(|fit| fit.loss), selected_continuation_epoch: continuation_fit.as_ref().map_or(0, |fit| fit.selected_epoch),
            continuation_rows, continuation_associations, dropped_row_events: continuation_fit.as_ref().map_or(0, |fit| fit.dropped_row_events), dropped_association_events: continuation_fit.as_ref().map_or(0, |fit| fit.dropped_association_events), final_copy_correct, final_no_copy_correct, final_exact_responses, config,
            target_law: "Offline complete identifier-prefix matching marks all matching retained occurrences positive. Copy must strictly beat the actual inherited lexical/Base score. Only actual quantized first-copy selection and complete matching observations create suffix frames; suffix uses canonical encoding after byte-token copied history plus EOS. Whole trajectories above32observations or positioncap are skipped, not truncated. No target index or answer buffer is serialized.".into(),
            suffix_law: completed_word_suffix.then(|| "Observed-completed-word boundary: H4/phase origin comes from the actual final copied byte in retained history; progress and last-two-token features contain only actual suffix observations, with BOS for unavailable suffix context. Original occurrence provenance and query prime remain. Selector dictionary, feature construction and fitting law are unchanged.".into()),
        };
        Ok((model, report))
    }
}

fn copy_mut(model: &mut Model) -> Result<&mut WordCopyModel> {
    model
        .response_entry
        .as_mut()
        .and_then(|entry| entry.copy.as_mut())
        .ok_or_else(|| Error("copy fitting component missing".into()))
}

fn logits(example: &Example, weights: &[f64]) -> Vec<f64> {
    example
        .alternatives
        .iter()
        .map(|alternative| {
            alternative
                .features
                .iter()
                .map(|&index| weights[index])
                .sum()
        })
        .collect()
}

fn update(example: &Example, weights: &mut [f64], rate: f64) -> Result<()> {
    let scores = logits(example, weights);
    let maximum = scores
        .iter()
        .copied()
        .fold(example.baseline_score, f64::max);
    let total = (example.baseline_score - maximum).exp()
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
                example.baseline_score
            } else {
                f64::NEG_INFINITY
            },
            f64::max,
        );
    let positive = if example.baseline_correct {
        (example.baseline_score - positive_max).exp()
    } else {
        0.0
    } + example
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
        let delta = rate * (desired - (score - maximum).exp() / total)
            / (alternative.features.len().max(1) as f64).sqrt();
        if !delta.is_finite() {
            return Err(Error("copy selector gradient is nonfinite".into()));
        }
        for &index in &alternative.features {
            weights[index] = (weights[index] + delta).clamp(-3900.0, 3900.0);
        }
    }
    Ok(())
}

fn measure(examples: &[Example], weights: &[f64]) -> Result<(usize, f64)> {
    let mut correct = 0;
    let mut loss = 0.0;
    for example in examples {
        let scores = logits(example, weights);
        let mut best = example.baseline_score;
        let mut right = example.baseline_correct;
        for (alternative, &score) in example.alternatives.iter().zip(&scores) {
            if score > best {
                best = score;
                right = alternative.correct;
            }
        }
        correct += usize::from(right);
        let total = (example.baseline_score - best).exp()
            + scores.iter().map(|score| (score - best).exp()).sum::<f64>();
        // A frozen lexical head can have a much larger finite increment than
        // newly initialized copy rows. Normalize positive mass independently;
        // exp(positive_score - best) can underflow even though NLL is finite.
        let positive_max = example
            .alternatives
            .iter()
            .zip(&scores)
            .filter(|(alternative, _)| alternative.correct)
            .map(|(_, score)| *score)
            .fold(
                if example.baseline_correct {
                    example.baseline_score
                } else {
                    f64::NEG_INFINITY
                },
                f64::max,
            );
        let positive = if example.baseline_correct {
            (example.baseline_score - positive_max).exp()
        } else {
            0.0
        } + example
            .alternatives
            .iter()
            .zip(&scores)
            .filter(|(alternative, _)| alternative.correct)
            .map(|(_, score)| (score - positive_max).exp())
            .sum::<f64>();
        let item_loss = (best - positive_max) + total.ln() - positive.ln();
        if !item_loss.is_finite() {
            return Err(Error("copy selector objective is nonfinite".into()));
        }
        loss += item_loss;
    }
    Ok((correct, loss / examples.len() as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_large_frozen_baseline_and_copy_logits_keep_finite_objective() {
        let copy_target = Example {
            document: 0,
            alternatives: vec![Alternative {
                features: vec![0],
                correct: true,
            }],
            baseline_score: 2000.0,
            baseline_correct: false,
            prefix_len: Some(1),
        };
        let (correct, loss) = measure(&[copy_target], &[0.0]).unwrap();
        assert_eq!(correct, 0);
        assert_eq!(loss, 2000.0);

        let no_copy_target = Example {
            document: 0,
            alternatives: vec![Alternative {
                features: vec![0],
                correct: false,
            }],
            baseline_score: 0.0,
            baseline_correct: true,
            prefix_len: None,
        };
        let (correct, loss) = measure(&[no_copy_target], &[2000.0]).unwrap();
        assert_eq!(correct, 0);
        assert_eq!(loss, 2000.0);
    }
}
