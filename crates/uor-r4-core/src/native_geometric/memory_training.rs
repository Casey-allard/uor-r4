//! Host-only fitting of a sparse categorical pointer/read operator.
//! Targets supervise copied token identity; no grammar or answer vocabulary
//! participates in the model or in memory candidate admission.
use super::memory_types::*;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

mod resumable;
pub use resumable::{
    MemoryReadDocumentExposure, MemoryReadDocumentSupervision, MemoryReadSchedule,
    MemoryReadStreamProgress, MemoryReadStreamReport, MemoryReadSupervision, MemoryReadTokenSpan,
    MemoryReadTrainer,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryReadDiagnostic {
    pub predicted_token: u32,
    pub candidate_routes: usize,
    pub target_routes: usize,
    pub query_context_routes_with_registered_row: usize,
    pub query_context_routes_without_registered_row: usize,
}

impl Session {
    /// Host diagnostic that recomputes prediction for the current prefix. Query
    /// context rows can be absent while other feature rows are present. Row
    /// registration alone does not imply a nonzero gradient or useful fit.
    /// This reports currently admitted routes and their registered /3 query
    /// rows; it neither admits candidates nor changes learned scores.
    pub fn memory_read_diagnostic(
        &mut self,
        model: &Model,
        target: u32,
    ) -> Result<Option<MemoryReadDiagnostic>> {
        if target as usize >= model.vocabulary_size() {
            return Err(Error(
                "memory diagnostic target is outside the vocabulary".into(),
            ));
        }
        let prediction = self.predict(model)?;
        if self.control == Control::MemoryDisabled {
            return Ok(None);
        }
        let (Some(state), Some(operator)) = (&self.memory, &model.memory_read) else {
            return Ok(None);
        };
        if !matches!(
            operator.schema.as_str(),
            QUERY_CONTEXT_MEMORY_SCHEMA | OCCURRENCE_MEMORY_SCHEMA
        ) {
            return Err(Error(
                "query-context coverage diagnostic requires the /3 or /4 reader".into(),
            ));
        }
        let registered = state
            .candidates
            .iter()
            .filter(|candidate| {
                operator
                    .rows
                    .binary_search_by_key(&candidate.features[16], |row| row.feature)
                    .is_ok()
            })
            .count();
        Ok(Some(MemoryReadDiagnostic {
            predicted_token: prediction.token,
            candidate_routes: state.candidates.len(),
            target_routes: state
                .candidates
                .iter()
                .filter(|candidate| candidate.token == target)
                .count(),
            query_context_routes_with_registered_row: registered,
            query_context_routes_without_registered_row: state.candidates.len() - registered,
        }))
    }
}

const ABSENT: usize = usize::MAX;
struct Alternative {
    token: u32,
    constant: i32,
    features: Option<Vec<usize>>,
}
struct Example {
    target: u32,
    alternatives: Vec<Alternative>,
    groups: Vec<Vec<usize>>,
}

// Schema /3 warmup mixes two fitting objectives equally: target-token
// marginal NLL and mean per-route NLL over all admitted target-token routes.
// The uniform component keeps a currently low-scoring correct route from
// losing all learning responsibility. It does not identify a source location.
// Callers supply a nonzero target_routes count whenever target_posterior exists.
fn pointer_coefficient(
    probability: f64,
    target_posterior: Option<f64>,
    target_routes: usize,
    uniform_half: bool,
) -> f64 {
    probability
        - target_posterior
            .map(|posterior| {
                if uniform_half {
                    0.5 * posterior + 0.5 / target_routes as f64
                } else {
                    posterior
                }
            })
            .unwrap_or(0.0)
}

fn validate_query_context_primes(model: &Model) -> Result<()> {
    if model
        .geometry
        .tokens
        .iter()
        .any(|token| token.prime >= QUERY_CONTEXT_PRIME_LIMIT)
    {
        return Err(Error(
            "query-context selector requires complete prime identities below 2^24".into(),
        ));
    }
    Ok(())
}

fn memory_feature_names(query_context: bool) -> Vec<String> {
    let mut names = vec![
        "pointer_bias",
        "query_distance_and_source_offset",
        "last_prime_and_offsets",
        "logarithmic_age",
        "last_prime_and_age",
        "relative_h4_transport",
        "relative_h4_and_offsets",
        "relative_h4_signed_orientation",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<_>>();
    names.extend((0..PHASE_CHANNELS).map(|channel| format!("relative_fixed_zeta_phase_{channel}")));
    if query_context {
        names.extend(
            [
                "ordered_query_prime_pair",
                "ordered_query_prime_pair_offsets_and_posting_rank",
            ]
            .map(String::from),
        );
    } else {
        names.extend(
            [
                "last_prime_and_value_predecessor_prime",
                "value_predecessor_prime",
            ]
            .map(String::from),
        );
    }
    names
}

fn occurrence_feature_names() -> Vec<String> {
    let mut names = memory_feature_names(true);
    names[5] = "local_source_to_query_h4_path_transport".into();
    names[6] = "local_h4_path_transport_and_offsets".into();
    names[7] = "local_h4_path_signed_orientation".into();
    for channel in 0..PHASE_CHANNELS {
        names[8 + channel] = format!("local_query_minus_source_fixed_zeta_phase_{channel}");
    }
    names
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
            (
                None,
                LEGACY_MEMORY_SCHEMA | QUERY_CONTEXT_MEMORY_SCHEMA | OCCURRENCE_MEMORY_SCHEMA,
            ) => true,
            (
                Some(aliases),
                MEMORY_SCHEMA | QUERY_CONTEXT_MEMORY_SCHEMA | OCCURRENCE_MEMORY_SCHEMA,
            ) => *aliases == compile_cue_aliases(model)?,
            _ => false,
        };
        if matches!(
            self.schema.as_str(),
            QUERY_CONTEXT_MEMORY_SCHEMA | OCCURRENCE_MEMORY_SCHEMA
        ) {
            validate_query_context_primes(model)?;
        }
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
            || self.fit_positions
                > self
                    .fit_schedule
                    .as_ref()
                    .map(|lineage| lineage.schedule.total_positions)
                    .unwrap_or(self.config.max_positions)
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
        if self.schema == OCCURRENCE_MEMORY_SCHEMA && self.fit_schedule.is_none() {
            return Err(Error(
                "occurrence composition requires its bound stream schedule".into(),
            ));
        }
        if let Some(lineage) = &self.fit_schedule {
            lineage.schedule.validate(&self.config)?;
            if !matches!(
                self.schema.as_str(),
                QUERY_CONTEXT_MEMORY_SCHEMA | OCCURRENCE_MEMORY_SCHEMA
            ) || lineage.schema != resumable::REPORT_SCHEMA
                || lineage.ordered_source_cid.len() != 71
                || !lineage.ordered_source_cid.starts_with("blake3:")
                || lineage.configuration_cid
                    != resumable::configuration_identity_for_operator(
                        self.config,
                        lineage.schedule,
                        self.cue_aliases.is_some(),
                        lineage.supervision_cid.as_deref(),
                        self.schema == OCCURRENCE_MEMORY_SCHEMA,
                    )?
                || lineage
                    .supervision_cid
                    .as_ref()
                    .is_some_and(|cid| cid.len() != 71 || !cid.starts_with("blake3:"))
            {
                return Err(Error(
                    "resumable memory schedule requires the query-context reader".into(),
                ));
            }
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
    pub fn memory_read_feature_layout(&self) -> Option<&str> {
        self.memory_read.as_ref().map(|memory| {
            if memory.schema == OCCURRENCE_MEMORY_SCHEMA {
                OCCURRENCE_FEATURE_LAYOUT
            } else if memory.schema == QUERY_CONTEXT_MEMORY_SCHEMA {
                QUERY_CONTEXT_FEATURE_LAYOUT
            } else {
                LEGACY_FEATURE_LAYOUT
            }
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
            if memory.schema == OCCURRENCE_MEMORY_SCHEMA {
                bytes += memory.config.candidate_limit * std::mem::size_of::<ComposedCandidate>()
                    + memory.config.candidate_limit
                        * MEMORY_FEATURE_COUNT
                        * std::mem::size_of::<MemoryFeature>();
            }
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
        self.fit_memory_read_impl(documents, config, false, false)
    }

    /// Explicit experimental cue equivalence. Exact output and geometric token
    /// identities are preserved; only memory read/write keys use the alias map.
    pub fn fit_memory_read_with_word_cues(
        &self,
        documents: &[Document],
        config: MemoryReadFitConfig,
    ) -> Result<(Model, MemoryReadFitReport)> {
        self.fit_memory_read_impl(documents, config, true, false)
    }

    /// Fit an explicit successor that conditions selection on the two latest
    /// complete query-prime identities and on the retrieved occurrence rank.
    /// `word_cues` chooses the existing optional cue equivalence; it does not
    /// change output identities. Pointer warmup mixes target-token marginal
    /// and uniform target-route objectives equally. Existing ordered-query
    /// bias rows are calibrated after the shared global bias; the final
    /// max-route refinement is shared with /1 and /2.
    pub fn fit_memory_read_with_query_context(
        &self,
        documents: &[Document],
        config: MemoryReadFitConfig,
        word_cues: bool,
    ) -> Result<(Model, MemoryReadFitReport)> {
        self.fit_memory_read_impl(documents, config, word_cues, true)
    }

    fn fit_memory_read_impl(
        &self,
        documents: &[Document],
        config: MemoryReadFitConfig,
        word_cues: bool,
        query_context: bool,
    ) -> Result<(Model, MemoryReadFitReport)> {
        config.validate(self.vocabulary_size())?;
        if query_context {
            validate_query_context_primes(self)?;
        }
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
            schema: if query_context {
                QUERY_CONTEXT_MEMORY_SCHEMA
            } else if word_cues {
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
            fit_schedule: None,
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
                            features: Some(features.to_vec()),
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
                let target_routes = example
                    .alternatives
                    .iter()
                    .filter(|alternative| {
                        alternative.token == example.target && alternative.features.is_some()
                    })
                    .count();
                if target_routes == 0 {
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
                // Legacy /1 and /2 retain their pure latent-route marginal.
                // /3 adds uniform responsibility among already admitted
                // target-token routes during this warmup only. The following
                // bias calibration and max-route refinement stay unchanged.
                for (alternative, score) in example.alternatives.iter().zip(&route_scores) {
                    let mass = libm::exp(*score - maximum);
                    let target_posterior =
                        if alternative.token == example.target && alternative.features.is_some() {
                            Some(libm::exp(*score - target_maximum) / target_denominator)
                        } else {
                            None
                        };
                    let coefficient = pointer_coefficient(
                        mass / denominator,
                        target_posterior,
                        target_routes,
                        query_context,
                    );
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
        let query_calibration = if query_context {
            calibrate_query_biases(&examples, &mut weights)
        } else {
            QueryBiasCalibration::default()
        };
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
            feature_layout: if query_context { QUERY_CONTEXT_FEATURE_LAYOUT } else { LEGACY_FEATURE_LAYOUT }.into(),
            feature_names: memory_feature_names(query_context),
            cue_identity: if word_cues { CUE_SCHEMA } else { EXACT_CUE_SCHEMA }.into(),
            aliased_lexical_tokens: memory.cue_aliases.as_ref().map(|aliases| {
                aliases.representatives.iter().enumerate()
                    .filter(|(token, representative)| **representative as usize != *token).count()
            }).unwrap_or(0),
            objective: if query_context {
                "pointer_half_target_marginal_half_uniform_target_route_nll_then_global_and_query_pair_bias_grids_then_max_route_ce_v4; best_epoch_on_fit_ce; diagnostics_quantized_max_route"
            } else {
                "pointer_only_token_marginal_then_fit_bias_grid_then_max_route_ce_v2; best_epoch_on_fit_ce; diagnostics_quantized_max_route"
            }.into(),
            pointer_pretrain_epochs: config.epochs,
            max_route_refinement_epochs: config.epochs,
            calibrated_bias_score: libm::round(calibrated_bias * SCORE_SCALE) as i32,
            query_bias_contexts: query_calibration.contexts,
            query_bias_changed_contexts: query_calibration.changed_contexts,
            query_bias_positions: query_calibration.positions,
            query_bias_cross_entropy_before: query_calibration.before,
            query_bias_cross_entropy_after: query_calibration.after,
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

#[derive(Default)]
struct QueryBiasCalibration {
    contexts: usize,
    changed_contexts: usize,
    positions: usize,
    before: f64,
    after: f64,
}

struct QueryBiasExample {
    target: usize,
    // Per-token maxima for routes without / with the selected query-bias row.
    scores: Vec<(f64, f64)>,
}

fn query_bias_loss(cache: &[QueryBiasExample], bias: f64) -> f64 {
    let mut total = 0.0;
    for example in cache {
        let target = example.scores[example.target];
        let target_score = target.0.max(target.1 + bias);
        let maximum = example
            .scores
            .iter()
            .map(|&(fixed, selected)| fixed.max(selected + bias))
            .fold(f64::NEG_INFINITY, f64::max);
        let denominator: f64 = example
            .scores
            .iter()
            .map(|&(fixed, selected)| libm::exp(fixed.max(selected + bias) - maximum))
            .sum();
        total += libm::log(denominator) + maximum - target_score;
    }
    total
}

fn calibrate_query_biases(examples: &[Example], weights: &mut [f64]) -> QueryBiasCalibration {
    // A query's feature16 is common to its memory routes. Group once by that
    // existing address, so grids touch only their own examples. Missing rows
    // remain absent, and no fit target creates a new feature or candidate.
    let mut grouped = BTreeMap::<usize, Vec<&Example>>::new();
    for example in examples {
        if let Some(address) = example.alternatives.iter().find_map(|alternative| {
            alternative
                .features
                .as_ref()
                .map(|features| features[16])
                .filter(|&address| address != ABSENT)
        }) {
            grouped.entry(address).or_default().push(example);
        }
    }
    let mut report = QueryBiasCalibration::default();
    for (address, group) in grouped {
        let cache: Vec<_> = group
            .into_iter()
            .filter_map(|example| {
                let target = example
                    .groups
                    .iter()
                    .position(|routes| example.alternatives[routes[0]].token == example.target)?;
                let scores = example
                    .groups
                    .iter()
                    .map(|routes| {
                        let mut fixed = f64::NEG_INFINITY;
                        let mut selected = f64::NEG_INFINITY;
                        for &index in routes {
                            let alternative = &example.alternatives[index];
                            let value = score(alternative, weights);
                            if alternative.features.as_ref().map(|features| features[16])
                                == Some(address)
                            {
                                selected = selected.max(value - weights[address]);
                            } else {
                                // This includes memory routes without the
                                // selected row, not just base-model routes.
                                fixed = fixed.max(value);
                            }
                        }
                        (fixed, selected)
                    })
                    .collect();
                Some(QueryBiasExample { target, scores })
            })
            .collect();
        if cache.is_empty() {
            continue;
        }
        let mut best_bias = weights[address];
        let before = query_bias_loss(&cache, best_bias);
        let mut best_loss = before;
        // Same bounded grid as global calibration. Strict improvement retains
        // the existing weight on ties, including completely inactive routes.
        for half in -32..=32 {
            let bias = f64::from(half) * 0.5;
            let loss = query_bias_loss(&cache, bias);
            if loss < best_loss {
                best_bias = bias;
                best_loss = loss;
            }
        }
        report.contexts += 1;
        report.changed_contexts += usize::from(best_bias != weights[address]);
        report.positions += cache.len();
        report.before += before;
        report.after += best_loss;
        weights[address] = best_bias;
    }
    if report.positions > 0 {
        report.before /= report.positions as f64;
        report.after /= report.positions as f64;
    }
    report
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

#[cfg(test)]
mod pointer_warmup_tests {
    use super::*;

    #[test]
    fn native_uniform_target_responsibility_reaches_a_starved_correct_route() {
        // Two admitted routes return the correct token; a third returns a
        // different token. The second correct route starts 40 nats below the
        // first, as can happen when a frequent shared route dominates fitting.
        let masses = [1.0, libm::exp(-40.0), libm::exp(-20.0)];
        let total: f64 = masses.iter().sum();
        let correct_total = masses[0] + masses[1];
        let probabilities = masses.map(|mass| mass / total);
        let targets = [
            Some(masses[0] / correct_total),
            Some(masses[1] / correct_total),
            None,
        ];
        let legacy = std::array::from_fn::<_, 3, _>(|index| {
            pointer_coefficient(probabilities[index], targets[index], 2, false)
        });
        let mixed = std::array::from_fn::<_, 3, _>(|index| {
            pointer_coefficient(probabilities[index], targets[index], 2, true)
        });
        assert_eq!(legacy[1], probabilities[1] - targets[1].unwrap());
        assert!(legacy[1].abs() < 1e-12);
        // The route-logit derivative now provides a substantial positive
        // learning signal. Shared-feature updates can couple several routes.
        assert!(-0.1 * mixed[1] > 0.0249);
        assert!(mixed[0] > 0.24);
        assert_eq!(mixed[2], legacy[2]);
        assert!(mixed[2] > 0.0);
        assert!(mixed.iter().sum::<f64>().abs() < 1e-12);
    }

    #[test]
    fn native_query_bias_calibration_separates_opposing_contexts_and_skips_absent_rows() {
        let example = |target, address| {
            let mut selected = [ABSENT; MEMORY_FEATURE_COUNT];
            selected[0] = 0;
            selected[16] = address;
            let mut fixed_memory = [ABSENT; MEMORY_FEATURE_COUNT];
            fixed_memory[0] = 0;
            Example {
                target,
                alternatives: vec![
                    Alternative {
                        token: 0,
                        constant: -1024,
                        features: None,
                    },
                    Alternative {
                        token: 1,
                        constant: 0,
                        features: None,
                    },
                    Alternative {
                        token: 0,
                        constant: 0,
                        features: Some(selected.to_vec()),
                    },
                    Alternative {
                        token: 1,
                        constant: 256,
                        features: Some(fixed_memory.to_vec()),
                    },
                ],
                groups: vec![vec![0, 2], vec![1, 3]],
            }
        };
        // Identical alternatives have opposite targets in different query
        // contexts. A single global pointer bias cannot make both correct.
        let mut examples = vec![example(0, 1), example(1, 2)];
        let mut weights = vec![-4.0, 0.0, 0.0];
        calibrate_bias(&examples, &mut weights);
        assert_eq!(metrics(&examples, &weights).0, 1);
        let fixed_score = score(&examples[0].alternatives[3], &weights);
        let absent = example(1, ABSENT);
        let absent_score = score(&absent.alternatives[2], &weights);
        examples.push(absent);
        let report = calibrate_query_biases(&examples, &mut weights);
        assert_eq!(report.contexts, 2);
        assert_eq!(report.changed_contexts, 2);
        assert_eq!(report.positions, 2);
        assert_eq!(weights.len(), 3);
        assert!(weights[1] > weights[2]);
        assert_eq!(metrics(&examples[..2], &weights).0, 2);
        assert!(report.after < report.before);
        // Cached loss must agree with the real max-route scorer, including
        // memory alternatives that do not contain the calibrated row.
        assert!((report.after - metrics(&examples[..2], &weights).1).abs() < 1e-12);
        assert_eq!(score(&examples[0].alternatives[3], &weights), fixed_score);
        assert_eq!(score(&examples[2].alternatives[2], &weights), absent_score);
    }
}
