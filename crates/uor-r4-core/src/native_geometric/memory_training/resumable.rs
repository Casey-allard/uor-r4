//! Explicit replay schedule for the /3 reader. Only a bounded batch of route
//! examples is live; every stage visits the same source-bound population.
//! Checkpoints keep unquantized optimizer state and calibration sums. A resumed
//! document replays its prefix once to recover the exact geometric/cue state.
use super::*;
use std::time::{Duration, Instant};

const CHECKPOINT_SCHEMA: &str = "uor-r4.native-memory-fit-checkpoint/1";
pub(super) const REPORT_SCHEMA: &str = "uor-r4.native-memory-stream-fit/1";
const MAX_SOURCE_BYTES: usize = 128 * 1024 * 1024;
const MAX_SOURCE_TOKENS: usize = 8_388_608;
const MAX_CHECKPOINT_BYTES: usize = 256 * 1024 * 1024;
const MAX_REPLAY_TOKENS_PER_BATCH: usize = 8192;
const SUPERVISION_SCHEMA: &str = "uor-r4.native-memory-token-span-supervision/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryReadTokenSpan {
    /// Inclusive index in encode(full_document_text), with BOS excluded.
    pub start: usize,
    /// Exclusive index. EOS is the target at encode(full_document_text).len().
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryReadDocumentSupervision {
    pub source: DocumentReceipt,
    /// Nonempty, sorted, nonoverlapping target intervals. No text is removed
    /// from causal context; these intervals select host training losses only.
    pub spans: Vec<MemoryReadTokenSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryReadSupervision {
    pub schema: String,
    /// Binds the tokenizer and all baseline state used to interpret positions.
    pub baseline_artifact: String,
    /// Must match the complete ordered fit source list exactly.
    pub documents: Vec<MemoryReadDocumentSupervision>,
}
impl MemoryReadSupervision {
    pub fn new(
        baseline: &Model,
        documents: &[Document],
        spans: Vec<Vec<MemoryReadTokenSpan>>,
    ) -> Result<Self> {
        if documents.len() != spans.len() || documents.is_empty() {
            return Err(Error(
                "memory supervision needs one span list per ordered source document".into(),
            ));
        }
        if documents
            .iter()
            .try_fold(0usize, |sum, document| sum.checked_add(document.text.len()))
            .is_none_or(|sum| sum > MAX_SOURCE_BYTES)
        {
            return Err(Error("memory supervision sources exceed 128 MiB".into()));
        }
        let receipts: Vec<_> = documents
            .iter()
            .map(super::super::training::receipt)
            .collect();
        let lengths = documents
            .iter()
            .map(|document| {
                baseline
                    .encode(&document.text)
                    .map(|tokens| tokens.len() + 1)
            })
            .collect::<Result<Vec<_>>>()?;
        if lengths.iter().sum::<usize>() > MAX_SOURCE_TOKENS {
            return Err(Error(
                "memory supervision sources exceed 8388608 token positions".into(),
            ));
        }
        let supervision = Self {
            schema: SUPERVISION_SCHEMA.into(),
            baseline_artifact: baseline.artifact_cid.clone(),
            documents: receipts
                .iter()
                .cloned()
                .zip(spans)
                .map(|(source, spans)| MemoryReadDocumentSupervision { source, spans })
                .collect(),
        };
        supervision.validate(baseline, &receipts, &lengths)?;
        Ok(supervision)
    }

    pub fn identity(&self) -> Result<String> {
        identity(self)
    }
    pub fn cid(&self) -> Result<String> {
        self.identity()
    }

    fn validate(
        &self,
        baseline: &Model,
        receipts: &[DocumentReceipt],
        lengths: &[usize],
    ) -> Result<Vec<usize>> {
        if self.schema != SUPERVISION_SCHEMA
            || self.baseline_artifact != baseline.artifact_cid
            || self.documents.len() != receipts.len()
            || self.documents.is_empty()
            || self
                .documents
                .iter()
                .try_fold(0usize, |sum, doc| sum.checked_add(doc.spans.len()))
                .is_none_or(|count| count > 65_536)
        {
            return Err(Error("memory supervision schema, tokenizer baseline, source count or span bound mismatch".into()));
        }
        let mut eligible = Vec::with_capacity(receipts.len());
        for ((document, receipt), &length) in self.documents.iter().zip(receipts).zip(lengths) {
            if document.source != *receipt
                || document.spans.is_empty()
                || document
                    .spans
                    .iter()
                    .any(|span| span.start >= span.end || span.end > length)
                || document
                    .spans
                    .windows(2)
                    .any(|pair| pair[0].end > pair[1].start)
            {
                return Err(Error("memory supervision has missing/changed sources or empty, overlapping, unordered or out-of-bounds token spans".into()));
            }
            eligible.push(
                document
                    .spans
                    .iter()
                    .map(|span| span.end - span.start)
                    .sum(),
            );
        }
        Ok(eligible)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryReadSchedule {
    /// Maximum distinct targets across the whole fit population, not per epoch.
    pub total_positions: usize,
    /// Live route-example buffer. Must equal the legacy config's max_positions.
    pub batch_positions: usize,
}
impl MemoryReadSchedule {
    pub(super) fn validate(&self, config: &MemoryReadFitConfig) -> Result<()> {
        if !(1..=1_048_576).contains(&self.total_positions)
            || self.batch_positions != config.max_positions
            || self.batch_positions > self.total_positions
        {
            return Err(Error("memory schedule needs 1..=1048576 total positions and a matching bounded batch capacity".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Stage {
    Discover,
    Pointer,
    GlobalCalibration,
    QueryCalibration,
    InitialSelection,
    Refine,
    SelectEpoch,
    FinalMetrics,
    Complete,
}
impl Stage {
    fn name(self) -> &'static str {
        match self {
            Self::Discover => "discover",
            Self::Pointer => "pointer",
            Self::GlobalCalibration => "global_calibration",
            Self::QueryCalibration => "query_calibration",
            Self::InitialSelection => "initial_selection",
            Self::Refine => "refine",
            Self::SelectEpoch => "select_epoch",
            Self::FinalMetrics => "final_metrics",
            Self::Complete => "complete",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MemoryReadDocumentExposure {
    pub id: String,
    pub token_positions: usize,
    pub supervised_positions: usize,
    pub target_in_candidates: usize,
    pub target_in_memory: usize,
    pub missing_query_context_positions: usize,
    pub tail_positions: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eligible_positions: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryReadStreamProgress {
    pub stage: String,
    /// Zero-based epoch within pointer or refinement stages.
    pub epoch: usize,
    pub document_cursor: usize,
    pub token_cursor: usize,
    pub sampled_cursor_in_document: usize,
    pub distinct_supervised_positions: usize,
    pub planned_supervised_positions: usize,
    pub replayed_context_positions: u64,
    pub processed_example_visits: u64,
    pub batches_completed: u64,
    pub learned_features: usize,
    pub is_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryReadStreamReport {
    pub schema: String,
    pub baseline_artifact: String,
    pub configuration_cid: String,
    pub ordered_source_cid: String,
    pub schedule: MemoryReadSchedule,
    pub fit: MemoryReadFitReport,
    pub progress: MemoryReadStreamProgress,
    pub document_exposure: Vec<MemoryReadDocumentExposure>,
    pub missing_query_context_positions: usize,
    pub unsupervised_query_feature_events: usize,
    pub calibration_population_positions: usize,
    pub peak_live_examples: usize,
    pub peak_live_alternatives: usize,
    pub replay_token_limit_per_batch: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supervision_cid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eligible_positions: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct MetricSum {
    correct: usize,
    reachable: usize,
    loss: f64,
    pointer_correct: usize,
    pointer_reachable: usize,
    pointer_loss: f64,
}
impl MetricSum {
    fn mean(&self) -> f64 {
        self.loss / self.reachable.max(1) as f64
    }
    fn pointer_mean(&self) -> f64 {
        self.pointer_loss / self.pointer_reachable.max(1) as f64
    }
    fn add(&mut self, example: &Example, weights: &[f64]) {
        let examples = std::slice::from_ref(example);
        let (correct, loss) = metrics(examples, weights);
        self.correct += correct;
        if loss.is_finite() {
            self.reachable += 1;
            self.loss += loss;
        }
        let (correct, loss) = pointer_metrics(examples, weights);
        self.pointer_correct += correct;
        if loss.is_finite() {
            self.pointer_reachable += 1;
            self.pointer_loss += loss;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BiasGrid {
    positions: usize,
    original: f64,
    original_loss: f64,
    losses: Vec<f64>,
}
impl BiasGrid {
    fn new(original: f64) -> Self {
        Self {
            positions: 0,
            original,
            original_loss: 0.0,
            losses: vec![0.0; 65],
        }
    }
    fn add(&mut self, example: &QueryBiasExample) {
        self.positions += 1;
        self.original_loss += query_bias_loss(std::slice::from_ref(example), self.original);
        for (index, loss) in self.losses.iter_mut().enumerate() {
            *loss += query_bias_loss(std::slice::from_ref(example), index as f64 * 0.5 - 16.0);
        }
    }
    fn best(&self) -> (f64, f64) {
        let mut best = (self.original, self.original_loss);
        for (index, &loss) in self.losses.iter().enumerate() {
            if loss < best.1 {
                best = (index as f64 * 0.5 - 16.0, loss);
            }
        }
        best
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointState {
    baseline_artifact: String,
    configuration_cid: String,
    ordered_source_cid: String,
    ordered_sources: Vec<DocumentReceipt>,
    config: MemoryReadFitConfig,
    schedule: MemoryReadSchedule,
    word_cues: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    compose_occurrences: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    supervision: Option<MemoryReadSupervision>,
    quotas: Vec<usize>,
    token_lengths: Vec<usize>,
    stage: Stage,
    epoch: usize,
    document: usize,
    position: usize,
    sampled: usize,
    // Index order is first observation order, not sorted feature order.
    registry: Vec<MemoryFeature>,
    weights: Vec<f64>,
    best_weights: Vec<f64>,
    best_loss: Option<f64>,
    calibrated_bias: f64,
    global_grid: Option<BiasGrid>,
    query_grids: BTreeMap<usize, BiasGrid>,
    query_contexts: usize,
    query_changed: usize,
    query_positions: usize,
    query_before: f64,
    query_after: f64,
    before: MetricSum,
    after: MetricSum,
    selection: MetricSum,
    exposure: Vec<MemoryReadDocumentExposure>,
    candidate_positions: usize,
    dropped_feature_events: usize,
    unsupervised_query_feature_events: usize,
    replayed_context_positions: u64,
    processed_example_visits: u64,
    batches_completed: u64,
    peak_live_examples: usize,
    peak_live_alternatives: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    schema: String,
    state_cid: String,
    state: CheckpointState,
}

struct Replay {
    session: Session,
    memory: MemoryState,
    work: Work,
}

/// Host-only resumable /3 fitting. Source token buffers are bounded separately
/// from live examples. Model/data/configuration identity is checked on restore.
pub struct MemoryReadTrainer {
    baseline: Model,
    memory: MemoryModel,
    tokens: Vec<Vec<u32>>,
    selected: Vec<Vec<usize>>,
    addresses: BTreeMap<MemoryFeature, usize>,
    state: CheckpointState,
    replay: Option<Replay>,
    scratch: OptimizerScratch,
}

fn identity<T: Serialize>(value: &T) -> Result<String> {
    let bytes = serde_json::to_vec(value).map_err(|error| Error(error.to_string()))?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}

pub(super) fn configuration_identity(
    config: MemoryReadFitConfig,
    schedule: MemoryReadSchedule,
    word_cues: bool,
) -> Result<String> {
    identity(&(REPORT_SCHEMA, config, schedule, word_cues))
}

pub(super) fn configuration_identity_with_supervision(
    config: MemoryReadFitConfig,
    schedule: MemoryReadSchedule,
    word_cues: bool,
    supervision_cid: Option<&str>,
) -> Result<String> {
    match supervision_cid {
        None => configuration_identity(config, schedule, word_cues),
        Some(cid) => identity(&(
            REPORT_SCHEMA,
            SUPERVISION_SCHEMA,
            config,
            schedule,
            word_cues,
            cid,
        )),
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

pub(super) fn configuration_identity_for_operator(
    config: MemoryReadFitConfig,
    schedule: MemoryReadSchedule,
    word_cues: bool,
    supervision_cid: Option<&str>,
    compose_occurrences: bool,
) -> Result<String> {
    let legacy =
        configuration_identity_with_supervision(config, schedule, word_cues, supervision_cid)?;
    if compose_occurrences {
        identity(&(OCCURRENCE_MEMORY_SCHEMA, legacy))
    } else {
        Ok(legacy)
    }
}

fn select_population(
    lengths: &[usize],
    quotas: &[usize],
    supervision: Option<&MemoryReadSupervision>,
) -> Vec<Vec<usize>> {
    lengths
        .iter()
        .zip(quotas)
        .enumerate()
        .map(|(document, (&length, &quota))| {
            let ranks = (0..quota).map(|sampled| selected_position(length, quota, sampled));
            match supervision {
                None => ranks.collect(),
                Some(supervision) => {
                    let spans = &supervision.documents[document].spans;
                    let mut span = 0;
                    let mut skipped = 0;
                    ranks
                        .map(|rank| {
                            while rank >= skipped + spans[span].end - spans[span].start {
                                skipped += spans[span].end - spans[span].start;
                                span += 1;
                            }
                            spans[span].start + rank - skipped
                        })
                        .collect()
                }
            }
        })
        .collect()
}

// Water-fill equal document quotas, redistributing only unused capacity from
// short documents. This realizes min(total_positions, total eligible positions).
fn quotas(lengths: &[usize], limit: usize) -> Vec<usize> {
    let mut result = vec![0; lengths.len()];
    let mut remaining = limit.min(lengths.iter().sum());
    while remaining > 0 {
        let active = lengths
            .iter()
            .zip(&result)
            .filter(|(length, count)| count < length)
            .count();
        if active == 0 {
            break;
        }
        let share = (remaining / active).max(1);
        for (&length, count) in lengths.iter().zip(&mut result) {
            let add = share.min(length - *count).min(remaining);
            *count += add;
            remaining -= add;
            if remaining == 0 {
                break;
            }
        }
    }
    result
}

fn selected_position(length: usize, quota: usize, sampled: usize) -> usize {
    if sampled >= quota {
        return length;
    }
    let tail = quota.min(8);
    let body_quota = quota - tail;
    let body_length = length - tail;
    if sampled >= body_quota {
        body_length + sampled - body_quota
    } else {
        ((sampled as u128 * body_length as u128) / body_quota as u128) as usize
    }
}

impl MemoryReadTrainer {
    pub fn new(
        baseline: &Model,
        documents: &[Document],
        config: MemoryReadFitConfig,
        schedule: MemoryReadSchedule,
        word_cues: bool,
    ) -> Result<Self> {
        Self::new_impl(
            baseline, documents, config, schedule, word_cues, None, false,
        )
    }

    pub fn new_with_supervision(
        baseline: &Model,
        documents: &[Document],
        config: MemoryReadFitConfig,
        schedule: MemoryReadSchedule,
        word_cues: bool,
        supervision: MemoryReadSupervision,
    ) -> Result<Self> {
        Self::new_impl(
            baseline,
            documents,
            config,
            schedule,
            word_cues,
            Some(supervision),
            false,
        )
    }

    /// Fit local H4/zeta path comparisons and combine distinct feature evidence
    /// reaching the same retained occurrence. No source-location labels are used.
    pub fn new_with_occurrence_composition(
        baseline: &Model,
        documents: &[Document],
        config: MemoryReadFitConfig,
        schedule: MemoryReadSchedule,
        word_cues: bool,
        supervision: Option<MemoryReadSupervision>,
    ) -> Result<Self> {
        Self::new_impl(
            baseline,
            documents,
            config,
            schedule,
            word_cues,
            supervision,
            true,
        )
    }

    pub fn composes_occurrences(&self) -> bool {
        self.state.compose_occurrences
    }

    fn new_impl(
        baseline: &Model,
        documents: &[Document],
        config: MemoryReadFitConfig,
        schedule: MemoryReadSchedule,
        word_cues: bool,
        supervision: Option<MemoryReadSupervision>,
        compose_occurrences: bool,
    ) -> Result<Self> {
        config.validate(baseline.vocabulary_size())?;
        schedule.validate(&config)?;
        validate_query_context_primes(baseline)?;
        if baseline.memory_read.is_some()
            || baseline.training.target_positions == 0
            || documents.is_empty()
        {
            return Err(Error("stream memory fitting needs a fitted baseline without a memory head and nonempty fit documents".into()));
        }
        let bytes = documents
            .iter()
            .try_fold(0usize, |sum, doc| sum.checked_add(doc.text.len()));
        if bytes.is_none_or(|sum| sum > MAX_SOURCE_BYTES) {
            return Err(Error("stream memory sources exceed 128 MiB".into()));
        }
        let mut ids = BTreeSet::new();
        let mut ordered_sources = Vec::new();
        let mut tokens = Vec::new();
        let mut token_count = 0;
        for document in documents {
            let receipt = super::super::training::receipt(document);
            if receipt.id.trim().is_empty()
                || !ids.insert(receipt.id.clone())
                || baseline
                    .construction
                    .iter()
                    .any(|known| known.id == receipt.id || known.text_cid == receipt.text_cid)
                || baseline
                    .readout_training
                    .iter()
                    .any(|known| known.id == receipt.id && known != &receipt)
            {
                return Err(Error("memory fit documents overlap count construction, repeat IDs or change readout-fit IDs".into()));
            }
            let mut encoded = baseline.encode(&document.text)?;
            encoded.push(EOS);
            token_count += encoded.len();
            if token_count > MAX_SOURCE_TOKENS {
                return Err(Error(
                    "stream memory sources exceed 8388608 token positions".into(),
                ));
            }
            tokens.push(encoded);
            ordered_sources.push(receipt);
        }
        let token_lengths: Vec<_> = tokens.iter().map(Vec::len).collect();
        let eligible_lengths = match &supervision {
            Some(supervision) => {
                supervision.validate(baseline, &ordered_sources, &token_lengths)?
            }
            None => token_lengths.clone(),
        };
        let supervision_cid = supervision
            .as_ref()
            .map(MemoryReadSupervision::identity)
            .transpose()?;
        let configuration_cid = configuration_identity_for_operator(
            config,
            schedule,
            word_cues,
            supervision_cid.as_deref(),
            compose_occurrences,
        )?;
        let quotas = quotas(&eligible_lengths, schedule.total_positions);
        let selected = select_population(&eligible_lengths, &quotas, supervision.as_ref());
        let exposure = ordered_sources
            .iter()
            .zip(&token_lengths)
            .zip(&eligible_lengths)
            .map(
                |((source, &length), &eligible)| MemoryReadDocumentExposure {
                    id: source.id.clone(),
                    token_positions: length,
                    eligible_positions: supervision.as_ref().map(|_| eligible),
                    ..MemoryReadDocumentExposure::default()
                },
            )
            .collect();
        let mut receipts = ordered_sources.clone();
        receipts.sort_by(|a, b| a.id.cmp(&b.id));
        let memory = MemoryModel {
            schema: if compose_occurrences {
                OCCURRENCE_MEMORY_SCHEMA
            } else {
                QUERY_CONTEXT_MEMORY_SCHEMA
            }
            .into(),
            baseline_artifact: baseline.artifact_cid.clone(),
            cue_aliases: if word_cues {
                Some(compile_cue_aliases(baseline)?)
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
            fit_schedule: Some(MemoryFitLineage {
                schema: REPORT_SCHEMA.into(),
                schedule,
                ordered_source_cid: identity(&ordered_sources)?,
                configuration_cid: configuration_cid.clone(),
                supervision_cid,
            }),
        };
        let state = CheckpointState {
            baseline_artifact: baseline.artifact_cid.clone(),
            configuration_cid,
            ordered_source_cid: identity(&ordered_sources)?,
            ordered_sources,
            config,
            schedule,
            word_cues,
            compose_occurrences,
            supervision,
            quotas,
            token_lengths,
            stage: Stage::Discover,
            epoch: 0,
            document: 0,
            position: 0,
            sampled: 0,
            registry: vec![MemoryFeature { kind: 0, value: 0 }],
            weights: vec![-4.0],
            best_weights: Vec::new(),
            best_loss: None,
            calibrated_bias: -4.0,
            global_grid: None,
            query_grids: BTreeMap::new(),
            query_contexts: 0,
            query_changed: 0,
            query_positions: 0,
            query_before: 0.0,
            query_after: 0.0,
            before: MetricSum::default(),
            after: MetricSum::default(),
            selection: MetricSum::default(),
            exposure,
            candidate_positions: 0,
            dropped_feature_events: 0,
            unsupervised_query_feature_events: 0,
            replayed_context_positions: 0,
            processed_example_visits: 0,
            batches_completed: 0,
            peak_live_examples: 0,
            peak_live_alternatives: 0,
        };
        Ok(Self {
            baseline: baseline.clone(),
            memory,
            tokens,
            selected,
            addresses: BTreeMap::from([(MemoryFeature { kind: 0, value: 0 }, 0)]),
            state,
            replay: None,
            scratch: OptimizerScratch::default(),
        })
    }

    pub fn restore(baseline: &Model, documents: &[Document], bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(Error("memory checkpoint exceeds 256 MiB".into()));
        }
        let checkpoint: Checkpoint =
            serde_json::from_slice(bytes).map_err(|error| Error(error.to_string()))?;
        if checkpoint.schema != CHECKPOINT_SCHEMA
            || checkpoint.state_cid != identity(&checkpoint.state)?
        {
            return Err(Error(
                "memory checkpoint schema or content identity mismatch".into(),
            ));
        }
        let state = checkpoint.state;
        let mut trainer = Self::new_impl(
            baseline,
            documents,
            state.config,
            state.schedule,
            state.word_cues,
            state.supervision.clone(),
            state.compose_occurrences,
        )?;
        if state.baseline_artifact != trainer.state.baseline_artifact
            || state.configuration_cid != trainer.state.configuration_cid
            || state.ordered_source_cid != trainer.state.ordered_source_cid
            || state.ordered_sources != trainer.state.ordered_sources
            || state.quotas != trainer.state.quotas
            || state.token_lengths != trainer.state.token_lengths
            || state
                .exposure
                .iter()
                .map(|doc| doc.eligible_positions)
                .collect::<Vec<_>>()
                != trainer
                    .state
                    .exposure
                    .iter()
                    .map(|doc| doc.eligible_positions)
                    .collect::<Vec<_>>()
        {
            return Err(Error(
                "memory checkpoint baseline, ordered source or configuration mismatch".into(),
            ));
        }
        validate_state(&state, &trainer.selected)?;
        trainer.addresses = state
            .registry
            .iter()
            .enumerate()
            .map(|(index, feature)| (*feature, index))
            .collect();
        trainer.state = state;
        // Context reconstruction is deferred to advance(), so its observed
        // work is charged to the same bounded call and cumulative counters.
        Ok(trainer)
    }

    pub fn checkpoint(&self) -> Result<Vec<u8>> {
        let checkpoint = Checkpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            state_cid: identity(&self.state)?,
            state: self.state.clone(),
        };
        let bytes = serde_json::to_vec(&checkpoint).map_err(|error| Error(error.to_string()))?;
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(Error("memory checkpoint exceeds 256 MiB".into()));
        }
        Ok(bytes)
    }

    pub fn is_complete(&self) -> bool {
        self.state.stage == Stage::Complete
    }

    pub fn supervision_cid(&self) -> Option<&str> {
        self.memory
            .fit_schedule
            .as_ref()
            .and_then(|lineage| lineage.supervision_cid.as_deref())
    }

    pub fn progress(&self) -> MemoryReadStreamProgress {
        MemoryReadStreamProgress {
            stage: self.state.stage.name().into(),
            epoch: self.state.epoch,
            document_cursor: self.state.document,
            token_cursor: self.state.position,
            sampled_cursor_in_document: self.state.sampled,
            distinct_supervised_positions: self
                .state
                .exposure
                .iter()
                .map(|doc| doc.supervised_positions)
                .sum(),
            planned_supervised_positions: self.state.quotas.iter().sum(),
            replayed_context_positions: self.state.replayed_context_positions,
            processed_example_visits: self.state.processed_example_visits,
            batches_completed: self.state.batches_completed,
            learned_features: self.state.registry.len(),
            is_complete: self.is_complete(),
        }
    }

    /// Perform at most max_batches bounded batches. The time limit is checked
    /// between examples during replay and between atomic optimizer batches; a
    /// single batch may overrun it. No stage owns an unbounded example cache.
    pub fn advance(
        &mut self,
        max_batches: usize,
        max_duration: Duration,
    ) -> Result<MemoryReadStreamProgress> {
        if max_batches == 0 || max_duration.is_zero() {
            return Err(Error(
                "memory fit advance needs a positive batch/time allowance".into(),
            ));
        }
        let started = Instant::now();
        for _ in 0..max_batches {
            if self.is_complete() || started.elapsed() >= max_duration {
                break;
            }
            let examples = self.batch(started, max_duration)?;
            self.state.peak_live_examples = self.state.peak_live_examples.max(examples.len());
            self.state.peak_live_alternatives = self.state.peak_live_alternatives.max(
                examples
                    .iter()
                    .map(|example| example.alternatives.len())
                    .sum(),
            );
            for example in &examples {
                self.process(example);
            }
            self.state.processed_example_visits += examples.len() as u64;
            self.state.batches_completed += 1;
            if self.state.document == self.tokens.len() {
                self.complete_pass()?;
            }
        }
        Ok(self.progress())
    }

    fn initialize_replay(&mut self) -> Result<()> {
        if self.replay.is_some() {
            return Ok(());
        }
        let mut session = self.baseline.session(Control::Full)?;
        let mut memory = MemoryState::new(&self.baseline, &self.memory);
        let mut work = Work::default();
        session.observe(&self.baseline, BOS)?;
        memory.observe(&self.baseline, &self.memory, BOS, &mut work);
        self.replay = Some(Replay {
            session,
            memory,
            work,
        });
        Ok(())
    }

    fn batch(&mut self, started: Instant, allowance: Duration) -> Result<Vec<Example>> {
        let mut examples = Vec::new();
        let mut observed = 0;
        while self.state.document < self.tokens.len()
            && examples.len() < self.state.schedule.batch_positions
            && observed < MAX_REPLAY_TOKENS_PER_BATCH
            && started.elapsed() < allowance
        {
            let document = self.state.document;
            if (self.state.quotas[document] == 0 && self.state.supervision.is_none())
                || self.state.position == self.tokens[document].len()
            {
                self.state.document += 1;
                self.state.position = 0;
                self.state.sampled = 0;
                self.replay = None;
                continue;
            }
            self.initialize_replay()?;
            let replay = self
                .replay
                .as_mut()
                .ok_or_else(|| Error("memory replay unavailable".into()))?;
            // A restored checkpoint has no transient session. Rebuild its
            // prefix in bounded increments, without changing the saved cursor.
            // session.work counts BOS plus every causal observation exactly.
            let reconstructed = (replay.session.work.observed_tokens as usize).saturating_sub(1);
            if reconstructed < self.state.position {
                let token = self.tokens[document][reconstructed];
                replay.session.observe(&self.baseline, token)?;
                replay
                    .memory
                    .observe(&self.baseline, &self.memory, token, &mut replay.work);
                self.state.replayed_context_positions += 1;
                observed += 1;
                continue;
            }
            let position = self.state.position;
            let target = self.tokens[document][position];
            if self.selected[document].get(self.state.sampled) == Some(&position) {
                replay.session.predict(&self.baseline)?;
                replay.memory.collect(
                    &self.baseline,
                    &self.memory,
                    Control::Full,
                    &mut replay.work,
                );
                let mut alternatives: Vec<_> = replay
                    .session
                    .candidates()
                    .iter()
                    .map(|candidate| Alternative {
                        token: candidate.token,
                        constant: candidate.score as i32,
                        features: None,
                    })
                    .collect();
                let target_in_memory = replay
                    .memory
                    .candidates
                    .iter()
                    .any(|candidate| candidate.token == target);
                let mut missing_query = false;
                // /4 learns the exact same unique-feature occurrence reduction
                // used by serving; /3 retains original route and feature order.
                let route_features: Vec<_> = if self.state.compose_occurrences {
                    replay
                        .memory
                        .composed
                        .iter()
                        .map(|candidate| {
                            (
                                candidate.token,
                                &replay.memory.composition_features[candidate.feature_start
                                    ..candidate.feature_start + candidate.feature_count],
                            )
                        })
                        .collect()
                } else {
                    replay
                        .memory
                        .candidates
                        .iter()
                        .map(|candidate| (candidate.token, candidate.features.as_slice()))
                        .collect()
                };
                for (token, candidate_features) in route_features {
                    let mut features = vec![ABSENT; candidate_features.len()];
                    let query_slot = candidate_features
                        .iter()
                        .position(|feature| feature.kind == 16)
                        .ok_or_else(|| {
                            Error("memory alternative lacks its query context".into())
                        })?;
                    for (slot, feature) in candidate_features.iter().enumerate() {
                        if let Some(&index) = self.addresses.get(feature) {
                            features[slot] = index;
                        } else if self.state.stage == Stage::Discover
                            && self.addresses.len() < self.state.config.max_features
                        {
                            let index = self.addresses.len();
                            self.addresses.insert(*feature, index);
                            self.state.registry.push(*feature);
                            self.state.weights.push(0.0);
                            features[slot] = index;
                        } else if self.state.stage == Stage::Discover {
                            self.state.dropped_feature_events += 1;
                            if feature.kind >= 16 {
                                self.state.unsupervised_query_feature_events += 1;
                            }
                        }
                    }
                    // Bias calibration shares one query row per occurrence. Its
                    // legacy slot stays explicit even for a larger feature union.
                    if features.len() <= 16 {
                        return Err(Error(
                            "memory occurrence feature union is incomplete".into(),
                        ));
                    }
                    features.swap(16, query_slot);
                    missing_query |= features[16] == ABSENT;
                    alternatives.push(Alternative {
                        token,
                        constant: self.baseline.prior_scores[token as usize],
                        features: Some(features),
                    });
                }
                if self.state.stage == Stage::Discover {
                    let exposure = &mut self.state.exposure[document];
                    exposure.supervised_positions += 1;
                    exposure.target_in_memory += usize::from(target_in_memory);
                    exposure.target_in_candidates += usize::from(
                        alternatives
                            .iter()
                            .any(|candidate| candidate.token == target),
                    );
                    exposure.missing_query_context_positions +=
                        usize::from(missing_query || replay.memory.candidates.is_empty());
                    exposure.tail_positions += usize::from(
                        self.state.sampled
                            >= self.state.quotas[document] - self.state.quotas[document].min(8),
                    );
                    self.state.candidate_positions += replay.memory.candidates.len();
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
                self.state.sampled += 1;
            }
            replay.session.observe(&self.baseline, target)?;
            replay
                .memory
                .observe(&self.baseline, &self.memory, target, &mut replay.work);
            self.state.position += 1;
            self.state.replayed_context_positions += 1;
            observed += 1;
        }
        // Completing the final token at a batch boundary completes the pass
        // without requiring an extra no-work batch.
        while self.state.document < self.tokens.len()
            && ((self.state.quotas[self.state.document] == 0 && self.state.supervision.is_none())
                || self.state.position == self.tokens[self.state.document].len())
        {
            self.state.document += 1;
            self.state.position = 0;
            self.state.sampled = 0;
            self.replay = None;
        }
        Ok(examples)
    }

    fn process(&mut self, example: &Example) {
        let state = &mut self.state;
        match state.stage {
            Stage::Discover => state.before.add(example, &state.weights),
            Stage::Pointer => self.scratch.update(example, &mut state.weights, true, -4.0),
            Stage::GlobalCalibration => {
                if let Some(cached) = bias_example(example, &state.weights, 0, false) {
                    state
                        .global_grid
                        .get_or_insert_with(|| BiasGrid::new(state.weights[0]))
                        .add(&cached);
                }
            }
            Stage::QueryCalibration => {
                if let Some(address) = example.alternatives.iter().find_map(|alternative| {
                    alternative
                        .features
                        .as_ref()
                        .map(|features| features[16])
                        .filter(|&address| address != ABSENT)
                }) {
                    if let Some(cached) = bias_example(example, &state.weights, address, true) {
                        state
                            .query_grids
                            .entry(address)
                            .or_insert_with(|| BiasGrid::new(state.weights[address]))
                            .add(&cached);
                    }
                }
            }
            Stage::InitialSelection | Stage::SelectEpoch => {
                state.selection.add(example, &state.weights)
            }
            Stage::Refine => {
                self.scratch
                    .update(example, &mut state.weights, false, state.calibrated_bias)
            }
            Stage::FinalMetrics => state.after.add(example, &state.weights),
            Stage::Complete => {}
        }
    }

    fn complete_pass(&mut self) -> Result<()> {
        let state = &mut self.state;
        state.stage = match state.stage {
            Stage::Discover => {
                if state.before.pointer_reachable == 0 {
                    return Err(Error(
                        "memory fit has no target values in its bounded memory candidates".into(),
                    ));
                }
                Stage::Pointer
            }
            Stage::Pointer => {
                state.epoch += 1;
                if state.epoch == state.config.epochs {
                    state.epoch = 0;
                    Stage::GlobalCalibration
                } else {
                    Stage::Pointer
                }
            }
            Stage::GlobalCalibration => {
                let grid = state
                    .global_grid
                    .take()
                    .ok_or_else(|| Error("memory calibration population is empty".into()))?;
                state.calibrated_bias = grid.best().0;
                state.weights[0] = state.calibrated_bias;
                Stage::QueryCalibration
            }
            Stage::QueryCalibration => {
                for (&address, grid) in &state.query_grids {
                    let (bias, loss) = grid.best();
                    state.query_contexts += 1;
                    state.query_changed += usize::from(bias != state.weights[address]);
                    state.query_positions += grid.positions;
                    state.query_before += grid.original_loss;
                    state.query_after += loss;
                    state.weights[address] = bias;
                }
                state.query_grids.clear();
                Stage::InitialSelection
            }
            Stage::InitialSelection => {
                state.best_loss = Some(state.selection.mean());
                state.best_weights.clone_from(&state.weights);
                state.selection = MetricSum::default();
                Stage::Refine
            }
            Stage::Refine => Stage::SelectEpoch,
            Stage::SelectEpoch => {
                let loss = state.selection.mean();
                if state.best_loss.is_none_or(|best| loss < best) {
                    state.best_loss = Some(loss);
                    state.best_weights.clone_from(&state.weights);
                }
                state.selection = MetricSum::default();
                state.epoch += 1;
                if state.epoch < state.config.epochs {
                    Stage::Refine
                } else {
                    state.weights.clone_from(&state.best_weights);
                    for weight in &mut state.weights {
                        *weight = libm::round(*weight * SCORE_SCALE) / SCORE_SCALE;
                    }
                    Stage::FinalMetrics
                }
            }
            Stage::FinalMetrics => Stage::Complete,
            Stage::Complete => Stage::Complete,
        };
        state.document = 0;
        state.position = 0;
        state.sampled = 0;
        self.replay = None;
        Ok(())
    }

    pub fn finish(&self) -> Result<(Model, MemoryReadStreamReport)> {
        if !self.is_complete() {
            return Err(Error(
                "memory fit is not complete; checkpoint and resume remaining stages".into(),
            ));
        }
        let state = &self.state;
        let mut memory = self.memory.clone();
        memory.rows = self
            .addresses
            .iter()
            .map(|(feature, &index)| MemoryWeight {
                feature: *feature,
                score: libm::round(state.weights[index] * SCORE_SCALE) as i32,
            })
            .collect();
        memory.fit_positions = state.quotas.iter().sum();
        let mut learned = self.baseline.clone();
        learned.memory_read = Some(memory.clone());
        learned.refresh_identity()?;
        learned.validate()?;
        let view = MemoryState::new(&self.baseline, &memory).state();
        let tail_positions: usize = state.exposure.iter().map(|doc| doc.tail_positions).sum();
        let positions = memory.fit_positions;
        let fit = MemoryReadFitReport {
            schema: memory.schema.clone(), feature_layout: if state.compose_occurrences { OCCURRENCE_FEATURE_LAYOUT } else { QUERY_CONTEXT_FEATURE_LAYOUT }.into(), feature_names: if state.compose_occurrences { occurrence_feature_names() } else { memory_feature_names(true) },
            cue_identity: if state.word_cues { CUE_SCHEMA } else { EXACT_CUE_SCHEMA }.into(),
            aliased_lexical_tokens: memory.cue_aliases.as_ref().map(|aliases| aliases.representatives.iter().enumerate().filter(|(token, representative)| **representative as usize != *token).count()).unwrap_or(0),
            objective: if state.compose_occurrences { "population_replay_unique_feature_occurrence_union; half_target_marginal_half_uniform_target_occurrence_nll_then_global_and_query_bias_grids_then_max_occurrence_ce/1; best_epoch_on_population_ce; diagnostics_quantized_max_occurrence" } else { "population_replay_pointer_half_target_marginal_half_uniform_target_route_nll_then_population_global_and_query_bias_grids_then_max_route_ce/1; best_epoch_on_population_ce; diagnostics_quantized_max_route" }.into(),
            pointer_pretrain_epochs: state.config.epochs, max_route_refinement_epochs: state.config.epochs,
            calibrated_bias_score: libm::round(state.calibrated_bias * SCORE_SCALE) as i32,
            query_bias_contexts: state.query_contexts, query_bias_changed_contexts: state.query_changed,
            query_bias_positions: state.query_positions,
            query_bias_cross_entropy_before: state.query_before / state.query_positions.max(1) as f64,
            query_bias_cross_entropy_after: state.query_after / state.query_positions.max(1) as f64,
            pointer_fit_correct_before: state.before.pointer_correct, pointer_fit_correct_after: state.after.pointer_correct,
            pointer_cross_entropy_before: state.before.pointer_mean(), pointer_cross_entropy_after: state.after.pointer_mean(),
            sampling: if state.supervision.is_some() {
                "equal_document_waterfill_quota_uniform_eligible_body_plus_final_min_8_eligible_positions/1; source_bound_token_span_population_replayed_in_source_order"
            } else {
                "equal_document_waterfill_quota_uniform_body_plus_final_min_8_positions/1; fixed_population_replayed_in_source_order"
            }.into(),
            documents: state.quotas.iter().filter(|&&quota| quota > 0).count(), positions,
            observed_context_positions: state.token_lengths.iter().zip(&state.quotas).filter(|(_, quota)| state.supervision.is_some() || **quota > 0).map(|(length, _)| *length).sum(),
            tail_positions_per_document_limit: 8, tail_positions, body_positions: positions - tail_positions,
            target_in_candidates: state.before.reachable, target_in_memory: state.before.pointer_reachable,
            candidate_positions: state.candidate_positions,
            fit_correct_before: state.before.correct, fit_correct_after: state.after.correct,
            candidate_cross_entropy_before: state.before.mean(), candidate_cross_entropy_after: state.after.mean(),
            learned_features: state.registry.len(), dropped_feature_events: state.dropped_feature_events, epochs: state.config.epochs,
            session_memory_bytes: view.ring_storage_bytes + view.index_storage_bytes + view.candidate_storage_bytes + view.composed_candidate_storage_bytes + view.composition_feature_storage_bytes,
        };
        let report = MemoryReadStreamReport {
            schema: REPORT_SCHEMA.into(),
            baseline_artifact: state.baseline_artifact.clone(),
            configuration_cid: state.configuration_cid.clone(),
            ordered_source_cid: state.ordered_source_cid.clone(),
            schedule: state.schedule,
            fit,
            progress: self.progress(),
            document_exposure: state.exposure.clone(),
            missing_query_context_positions: state
                .exposure
                .iter()
                .map(|doc| doc.missing_query_context_positions)
                .sum(),
            unsupervised_query_feature_events: state.unsupervised_query_feature_events,
            calibration_population_positions: positions,
            peak_live_examples: state.peak_live_examples,
            peak_live_alternatives: state.peak_live_alternatives,
            replay_token_limit_per_batch: MAX_REPLAY_TOKENS_PER_BATCH,
            supervision_cid: self.supervision_cid().map(String::from),
            eligible_positions: state.supervision.as_ref().map(|_| {
                state
                    .exposure
                    .iter()
                    .filter_map(|document| document.eligible_positions)
                    .sum()
            }),
        };
        Ok((learned, report))
    }
}

// Scores have no held example references; calibration retains 65 loss sums per
// already-discovered query row. Every grid sees frozen weights for its stage.
fn bias_example(
    example: &Example,
    weights: &[f64],
    address: usize,
    query: bool,
) -> Option<QueryBiasExample> {
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
                let applies = if query {
                    alternative.features.as_ref().map(|features| features[16]) == Some(address)
                } else {
                    alternative.features.is_some()
                };
                if applies {
                    selected = selected.max(value - weights[address]);
                } else {
                    fixed = fixed.max(value);
                }
            }
            (fixed, selected)
        })
        .collect();
    Some(QueryBiasExample { target, scores })
}

#[derive(Default)]
struct OptimizerScratch {
    gradient: Vec<f64>,
    marks: Vec<usize>,
    touched: Vec<usize>,
    stamp: usize,
    scores: Vec<f64>,
    winners: Vec<(usize, f64)>,
}
impl OptimizerScratch {
    fn add(&mut self, alternative: &Alternative, coefficient: f64) {
        if let Some(features) = &alternative.features {
            for &feature in features {
                if feature == ABSENT {
                    continue;
                }
                if self.marks[feature] != self.stamp {
                    self.marks[feature] = self.stamp;
                    self.gradient[feature] = 0.0;
                    self.touched.push(feature);
                }
                self.gradient[feature] += coefficient;
            }
        }
    }
    fn update(&mut self, example: &Example, weights: &mut [f64], pointer: bool, bias_anchor: f64) {
        self.gradient.resize(weights.len(), 0.0);
        self.marks.resize(weights.len(), 0);
        self.stamp += 1;
        self.touched.clear();
        if pointer {
            self.scores.clear();
            self.scores
                .extend(example.alternatives.iter().map(|alternative| {
                    if alternative.features.is_some() {
                        score(alternative, weights)
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
                return;
            }
            let maximum = self
                .scores
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            let denominator: f64 = self
                .scores
                .iter()
                .map(|value| libm::exp(*value - maximum))
                .sum();
            let target_maximum = example
                .alternatives
                .iter()
                .zip(&self.scores)
                .filter(|(alternative, _)| alternative.token == example.target)
                .map(|(_, value)| *value)
                .fold(f64::NEG_INFINITY, f64::max);
            let target_denominator: f64 = example
                .alternatives
                .iter()
                .zip(&self.scores)
                .filter(|(alternative, _)| alternative.token == example.target)
                .map(|(_, value)| libm::exp(*value - target_maximum))
                .sum();
            for (index, alternative) in example.alternatives.iter().enumerate() {
                let value = self.scores[index];
                let posterior =
                    if alternative.token == example.target && alternative.features.is_some() {
                        Some(libm::exp(value - target_maximum) / target_denominator)
                    } else {
                        None
                    };
                self.add(
                    alternative,
                    pointer_coefficient(
                        libm::exp(value - maximum) / denominator,
                        posterior,
                        target_routes,
                        true,
                    ),
                );
            }
        } else {
            winner_scores(example, weights, &mut self.winners);
            let Some(&(target, _)) = self
                .winners
                .iter()
                .find(|&&(index, _)| example.alternatives[index].token == example.target)
            else {
                return;
            };
            let maximum = self
                .winners
                .iter()
                .map(|(_, score)| *score)
                .fold(f64::NEG_INFINITY, f64::max);
            let denominator: f64 = self
                .winners
                .iter()
                .map(|(_, score)| libm::exp(*score - maximum))
                .sum();
            for slot in 0..self.winners.len() {
                let (index, value) = self.winners[slot];
                self.add(
                    &example.alternatives[index],
                    libm::exp(value - maximum) / denominator - f64::from(index == target),
                );
            }
        }
        for &feature in &self.touched {
            let anchor = if feature == 0 { bias_anchor } else { 0.0 };
            let rate = if pointer { 0.1 } else { 0.05 };
            weights[feature] = (weights[feature]
                - rate * (self.gradient[feature] + 0.0001 * (weights[feature] - anchor)))
                .clamp(-16.0, 16.0);
        }
    }
}

fn validate_state(state: &CheckpointState, selected: &[Vec<usize>]) -> Result<()> {
    let size = state.registry.len();
    let valid_weights = |weights: &[f64]| {
        weights
            .iter()
            .all(|value| value.is_finite() && (-16.0..=16.0).contains(value))
    };
    let valid_grid = |grid: &BiasGrid| {
        grid.losses.len() == 65
            && grid.original.is_finite()
            && (-16.0..=16.0).contains(&grid.original)
            && grid.original_loss.is_finite()
            && grid
                .losses
                .iter()
                .all(|value| value.is_finite() && *value >= 0.0)
    };
    if size == 0
        || size > state.config.max_features
        || state.weights.len() != size
        || state.registry[0] != (MemoryFeature { kind: 0, value: 0 })
        || state
            .registry
            .iter()
            .any(|feature| feature.kind >= MEMORY_FEATURE_COUNT as u8)
        || state
            .registry
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len()
            != size
        || !valid_weights(&state.weights)
        || (!state.best_weights.is_empty()
            && (state.best_weights.len() != size || !valid_weights(&state.best_weights)))
        || state
            .best_loss
            .is_some_and(|value| !value.is_finite() || value < 0.0)
        || !state.calibrated_bias.is_finite()
        || !(-16.0..=16.0).contains(&state.calibrated_bias)
        || state.document >= state.quotas.len()
        || state.position > state.token_lengths[state.document]
        || state.sampled > state.quotas[state.document]
        || state.epoch > state.config.epochs
        || state.exposure.len() != state.quotas.len()
        || state
            .exposure
            .iter()
            .zip(&state.quotas)
            .any(|(exposure, quota)| exposure.supervised_positions > *quota)
        || state
            .global_grid
            .as_ref()
            .is_some_and(|grid| !valid_grid(grid))
        || state.query_grids.iter().any(|(&address, grid)| {
            address >= size || state.registry[address].kind != 16 || !valid_grid(grid)
        })
    {
        return Err(Error(
            "memory checkpoint optimizer, registry, cursor or calibration state is invalid".into(),
        ));
    }
    let cursor_consistent = (state.sampled == 0
        || selected[state.document][state.sampled - 1] < state.position)
        && selected[state.document]
            .get(state.sampled)
            .copied()
            .unwrap_or(state.token_lengths[state.document])
            >= state.position;
    let best_required = matches!(
        state.stage,
        Stage::Refine | Stage::SelectEpoch | Stage::FinalMetrics | Stage::Complete
    );
    let epoch_valid = match state.stage {
        Stage::Pointer | Stage::Refine | Stage::SelectEpoch => state.epoch < state.config.epochs,
        Stage::FinalMetrics | Stage::Complete => state.epoch == state.config.epochs,
        _ => state.epoch == 0,
    };
    let exposure_valid = state.exposure.iter().enumerate().all(|(index, exposure)| {
        let expected = if state.stage != Stage::Discover || index < state.document {
            state.quotas[index]
        } else if index == state.document {
            state.sampled
        } else {
            0
        };
        exposure.id == state.ordered_sources[index].id
            && exposure.token_positions == state.token_lengths[index]
            && exposure.supervised_positions == expected
            && exposure.target_in_memory <= exposure.target_in_candidates
            && exposure.target_in_candidates <= exposure.supervised_positions
            && exposure.missing_query_context_positions <= exposure.supervised_positions
            && exposure.tail_positions <= exposure.supervised_positions.min(8)
    });
    let valid_metrics = |metric: &MetricSum| {
        metric.loss.is_finite()
            && metric.pointer_loss.is_finite()
            && metric.reachable <= state.schedule.total_positions
            && metric.pointer_reachable <= metric.reachable
            && metric.correct <= state.schedule.total_positions
            && metric.pointer_correct <= metric.pointer_reachable
    };
    if !cursor_consistent
        || !epoch_valid
        || !exposure_valid
        || (best_required && (state.best_weights.len() != size || state.best_loss.is_none()))
        || (!best_required && (!state.best_weights.is_empty() || state.best_loss.is_some()))
        || (state.stage != Stage::GlobalCalibration && state.global_grid.is_some())
        || (state.stage != Stage::QueryCalibration && !state.query_grids.is_empty())
        || !valid_metrics(&state.before)
        || !valid_metrics(&state.after)
        || !valid_metrics(&state.selection)
        || !state.query_before.is_finite()
        || !state.query_after.is_finite()
        || state.query_changed > state.query_contexts
        || state.query_positions > state.schedule.total_positions
        || state.before.reachable
            != state
                .exposure
                .iter()
                .map(|exposure| exposure.target_in_candidates)
                .sum::<usize>()
        || state.before.pointer_reachable
            != state
                .exposure
                .iter()
                .map(|exposure| exposure.target_in_memory)
                .sum::<usize>()
    {
        return Err(Error(
            "memory checkpoint stage, population or cursor semantics are invalid".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Model, Vec<Document>) {
        let catalog = vec![Document { id: "stream-catalog".into(), text: "The orb is red. The cube is blue. Now the orb is blue. What is the orb? red blue. let x = 1; x = 2; assert_eq!(x, 2);".into() }];
        let mut trainer = Trainer::new(
            Config {
                context_tokens: 32,
                candidate_limit: 8,
                max_lexical_pieces: 128,
                ..Config::default()
            },
            &catalog,
        )
        .unwrap();
        trainer.train_documents(&catalog).unwrap();
        let documents = (0..4).map(|index| Document { id: format!("stream-fit-{index}"), text: format!("The orb is {}. The cube is blue. Now the orb is {}. What is the orb? {}. let x = 1; x = 2; assert_eq!(x, 2);", if index % 2 == 0 { "red" } else { "blue" }, if index < 2 { "red" } else { "blue" }, if index < 2 { "red" } else { "blue" }) }).collect();
        (trainer.compile().unwrap(), documents)
    }

    fn config(batch: usize) -> MemoryReadFitConfig {
        MemoryReadFitConfig {
            max_positions: batch,
            candidate_limit: 16,
            max_features: 4096,
            epochs: 1,
            ..MemoryReadFitConfig::default()
        }
    }

    fn complete(trainer: &mut MemoryReadTrainer) {
        for _ in 0..10_000 {
            if trainer.is_complete() {
                return;
            }
            trainer.advance(32, Duration::from_secs(10)).unwrap();
        }
        panic!("bounded fixture did not complete");
    }

    #[test]
    fn native_occurrence_stream_restores_calibration_and_matches_integer_reduction() {
        let (baseline, documents) = fixture();
        let schedule = MemoryReadSchedule {
            total_positions: 96,
            batch_positions: 7,
        };
        let make = || {
            MemoryReadTrainer::new_with_occurrence_composition(
                &baseline,
                &documents,
                config(7),
                schedule,
                true,
                None,
            )
            .unwrap()
        };
        let mut continuous = make();
        complete(&mut continuous);
        let mut resumed = make();
        let mut stages = BTreeSet::new();
        for _ in 0..10000 {
            if resumed.is_complete() {
                break;
            }
            resumed.advance(1, Duration::from_secs(10)).unwrap();
            if resumed.state.position > 0 && stages.insert(resumed.state.stage.name()) {
                resumed = MemoryReadTrainer::restore(
                    &baseline,
                    &documents,
                    &resumed.checkpoint().unwrap(),
                )
                .unwrap();
                assert!(resumed.composes_occurrences());
            }
        }
        assert!(resumed.is_complete());
        assert_eq!(stages.len(), 8);
        let (model, report) = resumed.finish().unwrap();
        assert_eq!(
            model.to_bytes().unwrap(),
            continuous.finish().unwrap().0.to_bytes().unwrap()
        );
        assert_eq!(model.memory_read_version(), Some(OCCURRENCE_MEMORY_SCHEMA));
        assert_eq!(report.fit.feature_layout, OCCURRENCE_FEATURE_LAYOUT);
        let roundtrip = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
        let mut session = roundtrip.session(Control::Full).unwrap();
        for token in baseline.encode(&documents[0].text).unwrap() {
            session.observe(&roundtrip, token).unwrap();
            session.predict(&roundtrip).unwrap();
            let memory = session.memory.as_ref().unwrap();
            let learned = model.memory_read.as_ref().unwrap();
            for candidate in &memory.composed {
                let features = &memory.composition_features
                    [candidate.feature_start..candidate.feature_start + candidate.feature_count];
                assert!(features.windows(2).all(|pair| pair[0] < pair[1]));
                assert_eq!(features.iter().filter(|f| f.kind == 0).count(), 1);
                assert_eq!(features.iter().filter(|f| f.kind == 16).count(), 1);
                let expected = i64::from(model.prior_scores[candidate.token as usize])
                    + features
                        .iter()
                        .filter_map(|f| {
                            learned
                                .rows
                                .binary_search_by_key(f, |r| r.feature)
                                .ok()
                                .map(|i| i64::from(learned.rows[i].score))
                        })
                        .sum::<i64>();
                assert_eq!(candidate.score, expected);
            }
        }
        assert!(session.work.memory_composed_candidates > 0);
        let old = MemoryReadTrainer::new(&baseline, &documents, config(7), schedule, true).unwrap();
        assert_ne!(old.state.configuration_cid, resumed.state.configuration_cid);
        let old_checkpoint: serde_json::Value =
            serde_json::from_slice(&old.checkpoint().unwrap()).unwrap();
        assert!(old_checkpoint["state"].get("compose_occurrences").is_none());
    }

    #[test]
    fn native_stream_sampling_fills_distinct_population_and_keeps_tail() {
        let lengths = [2, 100, 3, 100];
        let quotas = quotas(&lengths, 39);
        assert_eq!(quotas.iter().sum::<usize>(), 39);
        assert_eq!(quotas[0], 2);
        assert_eq!(quotas[2], 3);
        for (&length, &quota) in lengths.iter().zip(&quotas) {
            let selected: Vec<_> = (0..quota)
                .map(|index| selected_position(length, quota, index))
                .collect();
            assert!(selected.windows(2).all(|pair| pair[0] < pair[1]));
            assert_eq!(selected.last(), Some(&(length - 1)));
            assert_eq!(
                &selected[selected.len() - quota.min(8)..],
                &(length - quota.min(8)..length).collect::<Vec<_>>()
            );
        }
        assert_eq!(super::quotas(&lengths, 999), lengths);
        assert!(MemoryReadSchedule {
            total_positions: 100_000,
            batch_positions: 128
        }
        .validate(&config(128))
        .is_ok());
        assert!(config(8192).validate(300).is_err());
    }

    #[test]
    fn native_stream_resume_preserves_floats_registry_calibration_and_selected_epoch() {
        let (baseline, documents) = fixture();
        let schedule = MemoryReadSchedule {
            total_positions: 96,
            batch_positions: 7,
        };
        let mut continuous =
            MemoryReadTrainer::new(&baseline, &documents, config(7), schedule, true).unwrap();
        complete(&mut continuous);
        let mut resumed =
            MemoryReadTrainer::new(&baseline, &documents, config(7), schedule, true).unwrap();
        let mut stages = BTreeSet::new();
        for _ in 0..10_000 {
            if resumed.is_complete() {
                break;
            }
            resumed.advance(1, Duration::from_secs(10)).unwrap();
            // Save in the middle of every stage, including nonempty bias grids.
            if resumed.state.position > 0 && stages.insert(resumed.state.stage.name()) {
                let bytes = resumed.checkpoint().unwrap();
                let restored = MemoryReadTrainer::restore(&baseline, &documents, &bytes).unwrap();
                assert_eq!(restored.state, resumed.state);
                assert_eq!(restored.checkpoint().unwrap(), bytes);
                resumed = restored;
            }
        }
        assert!(resumed.is_complete());
        assert_eq!(stages.len(), 8);
        let (expected, expected_report) = continuous.finish().unwrap();
        let (actual, actual_report) = resumed.finish().unwrap();
        assert_eq!(actual.to_bytes().unwrap(), expected.to_bytes().unwrap());
        assert_eq!(actual_report.fit, expected_report.fit);
        assert_eq!(
            actual_report.document_exposure,
            expected_report.document_exposure
        );
        assert_eq!(actual_report.fit.positions, 96);
        assert!(actual_report.peak_live_examples <= 7);
        assert!(
            actual_report.progress.replayed_context_positions
                > expected_report.progress.replayed_context_positions
        );
        assert_eq!(actual.memory_read.as_ref().unwrap().fit_positions, 96);
        assert_eq!(actual.memory_read_config().unwrap().max_positions, 7);
        assert!(Model::from_bytes(&actual.to_bytes().unwrap()).is_ok());
        let restored =
            MemoryReadTrainer::restore(&baseline, &documents, &resumed.checkpoint().unwrap())
                .unwrap();
        assert!(restored.is_complete());
        assert_eq!(
            restored.finish().unwrap().0.to_bytes().unwrap(),
            actual.to_bytes().unwrap()
        );
    }

    #[test]
    fn native_stream_checkpoint_rejects_changed_sources_order_baseline_and_invalid_stage() {
        let (baseline, documents) = fixture();
        let mut trainer = MemoryReadTrainer::new(
            &baseline,
            &documents,
            config(8),
            MemoryReadSchedule {
                total_positions: 64,
                batch_positions: 8,
            },
            false,
        )
        .unwrap();
        trainer.advance(1, Duration::from_secs(10)).unwrap();
        let bytes = trainer.checkpoint().unwrap();
        let mut changed = documents.clone();
        changed[0].text.push_str(" changed");
        assert!(MemoryReadTrainer::restore(&baseline, &changed, &bytes).is_err());
        let mut reordered = documents.clone();
        reordered.swap(0, 1);
        assert!(MemoryReadTrainer::restore(&baseline, &reordered, &bytes).is_err());
        let mut other_baseline = baseline.clone();
        other_baseline.artifact_cid.push('x');
        assert!(MemoryReadTrainer::restore(&other_baseline, &documents, &bytes).is_err());
        // An integrity hash is not permission to skip semantic validation.
        let mut invalid = trainer.state.clone();
        invalid.sampled = 0;
        let malformed = Checkpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            state_cid: identity(&invalid).unwrap(),
            state: invalid,
        };
        assert!(MemoryReadTrainer::restore(
            &baseline,
            &documents,
            &serde_json::to_vec(&malformed).unwrap()
        )
        .is_err());
        complete(&mut trainer);
        let mut invalid = trainer.state.clone();
        invalid.best_weights.clear();
        let malformed = Checkpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            state_cid: identity(&invalid).unwrap(),
            state: invalid,
        };
        assert!(MemoryReadTrainer::restore(
            &baseline,
            &documents,
            &serde_json::to_vec(&malformed).unwrap()
        )
        .is_err());
    }

    #[test]
    fn native_stream_calibration_accumulates_population_before_applying_bias() {
        let example = |target| {
            let mut features = [ABSENT; MEMORY_FEATURE_COUNT];
            features[0] = 0;
            features[16] = 1;
            Example {
                target,
                alternatives: vec![
                    Alternative {
                        token: 0,
                        constant: -512,
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
                        features: Some(features.to_vec()),
                    },
                ],
                groups: vec![vec![0, 2], vec![1]],
            }
        };
        let examples = vec![example(0), example(0), example(0), example(1)];
        let weights = vec![-4.0, 0.0];
        let mut grid = BiasGrid::new(weights[0]);
        for row in &examples[..3] {
            grid.add(&bias_example(row, &weights, 0, false).unwrap());
        }
        let bytes = serde_json::to_vec(&grid).unwrap();
        let mut grid: BiasGrid = serde_json::from_slice(&bytes).unwrap();
        grid.add(&bias_example(&examples[3], &weights, 0, false).unwrap());
        let mut expected = weights.clone();
        calibrate_bias(&examples, &mut expected);
        assert_eq!(grid.best().0, expected[0]);
        let mut last_batch = weights.clone();
        calibrate_bias(&examples[3..], &mut last_batch);
        assert_ne!(grid.best().0, last_batch[0]);
        assert_eq!(grid.positions, 4);
        let mut expected_query = expected.clone();
        let expected_report = calibrate_query_biases(&examples, &mut expected_query);
        let mut query = BiasGrid::new(expected[1]);
        for row in &examples {
            query.add(&bias_example(row, &expected, 1, true).unwrap());
        }
        assert_eq!(query.best().0, expected_query[1]);
        assert!((query.best().1 / 4.0 - expected_report.after).abs() < 1e-12);
    }

    #[test]
    fn native_stream_batch_capacity_preserves_legacy_query_reader_objective() {
        let (baseline, documents) = fixture();
        let positions = 96;
        let (legacy, legacy_report) = baseline
            .fit_memory_read_with_query_context(&documents, config(positions), true)
            .unwrap();
        let mut small = MemoryReadTrainer::new(
            &baseline,
            &documents,
            config(7),
            MemoryReadSchedule {
                total_positions: positions,
                batch_positions: 7,
            },
            true,
        )
        .unwrap();
        let mut large = MemoryReadTrainer::new(
            &baseline,
            &documents,
            config(32),
            MemoryReadSchedule {
                total_positions: positions,
                batch_positions: 32,
            },
            true,
        )
        .unwrap();
        complete(&mut small);
        complete(&mut large);
        let (small, small_report) = small.finish().unwrap();
        let (large, large_report) = large.finish().unwrap();
        assert_eq!(
            small.memory_read.as_ref().unwrap().rows,
            large.memory_read.as_ref().unwrap().rows
        );
        assert_eq!(
            small.memory_read.as_ref().unwrap().rows,
            legacy.memory_read.as_ref().unwrap().rows
        );
        assert_eq!(
            small_report.fit.candidate_cross_entropy_after,
            large_report.fit.candidate_cross_entropy_after
        );
        assert_eq!(
            small_report.fit.candidate_cross_entropy_after,
            legacy_report.candidate_cross_entropy_after
        );
        assert_eq!(
            small_report.fit.query_bias_cross_entropy_after,
            legacy_report.query_bias_cross_entropy_after
        );
        for prompt in [
            "The orb is red. Now the orb is blue. What is the orb?",
            "let x = 1; x = 2; assert_eq!(x,",
        ] {
            let mut legacy_session = legacy.session(Control::Full).unwrap();
            let mut small_session = small.session(Control::Full).unwrap();
            for session_and_model in [(&mut legacy_session, &legacy), (&mut small_session, &small)]
            {
                session_and_model
                    .0
                    .observe(session_and_model.1, BOS)
                    .unwrap();
                for token in session_and_model.1.encode(prompt).unwrap() {
                    session_and_model
                        .0
                        .observe(session_and_model.1, token)
                        .unwrap();
                }
            }
            assert_eq!(
                small_session.predict(&small).unwrap(),
                legacy_session.predict(&legacy).unwrap()
            );
            let target = small.encode(" red").unwrap()[0];
            let diagnostic = small_session
                .memory_read_diagnostic(&small, target)
                .unwrap()
                .unwrap();
            assert_eq!(
                diagnostic.candidate_routes,
                diagnostic.query_context_routes_with_registered_row
                    + diagnostic.query_context_routes_without_registered_row
            );
            assert!(diagnostic.target_routes <= diagnostic.candidate_routes);
        }
        // The added schedule fields remain absent from historical artifacts.
        let legacy_value: serde_json::Value =
            serde_json::from_slice(&legacy.to_bytes().unwrap()).unwrap();
        assert!(legacy_value["memory_read"].get("fit_schedule").is_none());
    }

    #[test]
    fn native_stream_span_supervision_observes_full_context_and_resumes_exact_population() {
        let (baseline, documents) = fixture();
        let baseline_bytes = baseline.to_bytes().unwrap();
        let lengths: Vec<_> = documents
            .iter()
            .map(|doc| baseline.encode(&doc.text).unwrap().len() + 1)
            .collect();
        let spans: Vec<_> = lengths
            .iter()
            .map(|&length| {
                vec![MemoryReadTokenSpan {
                    start: length - 6,
                    end: length,
                }]
            })
            .collect();
        let mask = MemoryReadSupervision::new(&baseline, &documents, spans).unwrap();
        let mask_cid = mask.cid().unwrap();
        let schedule = MemoryReadSchedule {
            total_positions: 96,
            batch_positions: 3,
        };
        let make = || {
            MemoryReadTrainer::new_with_supervision(
                &baseline,
                &documents,
                config(3),
                schedule,
                true,
                mask.clone(),
            )
            .unwrap()
        };
        let mut checked = make();
        assert_eq!(checked.state.quotas, vec![6; 4]);
        assert_eq!(
            checked.selected[0],
            (lengths[0] - 6..lengths[0]).collect::<Vec<_>>()
        );
        assert_eq!(checked.tokens[0][lengths[0] - 1], EOS);
        let examples = checked
            .batch(Instant::now(), Duration::from_secs(10))
            .unwrap();
        assert_eq!(
            examples
                .iter()
                .map(|example| example.target)
                .collect::<Vec<_>>(),
            checked.tokens[0][lengths[0] - 6..lengths[0] - 3]
        );
        // Independent causal replay observes the complete prefix, including
        // excluded prompt tokens and eviction, while collecting only span loss
        // positions. Those observations determine the identical feature set.
        let mut session = baseline.session(Control::Full).unwrap();
        let mut memory = MemoryState::new(&baseline, &checked.memory);
        let mut work = Work::default();
        session.observe(&baseline, BOS).unwrap();
        memory.observe(&baseline, &checked.memory, BOS, &mut work);
        let mut registered = BTreeSet::from([MemoryFeature { kind: 0, value: 0 }]);
        for (position, &token) in checked.tokens[0][..checked.state.position]
            .iter()
            .enumerate()
        {
            if checked.selected[0][..3].contains(&position) {
                memory.collect(&baseline, &checked.memory, Control::Full, &mut work);
                for candidate in &memory.candidates {
                    registered.extend(candidate.features);
                }
            }
            session.observe(&baseline, token).unwrap();
            memory.observe(&baseline, &checked.memory, token, &mut work);
        }
        assert_eq!(
            session.state(),
            checked.replay.as_ref().unwrap().session.state()
        );
        assert!(session.work.evictions > 0);
        assert_eq!(registered, checked.state.registry.iter().copied().collect());
        let mut continuous = make();
        complete(&mut continuous);
        let mut resumed = make();
        let mut stages = BTreeSet::new();
        for _ in 0..10_000 {
            if resumed.is_complete() {
                break;
            }
            resumed.advance(1, Duration::from_secs(10)).unwrap();
            if resumed.state.position > 0 && stages.insert(resumed.state.stage.name()) {
                resumed = MemoryReadTrainer::restore(
                    &baseline,
                    &documents,
                    &resumed.checkpoint().unwrap(),
                )
                .unwrap();
            }
        }
        assert_eq!(stages.len(), 8);
        let (expected, report) = continuous.finish().unwrap();
        let (actual, resumed_report) = resumed.finish().unwrap();
        assert_eq!(actual.to_bytes().unwrap(), expected.to_bytes().unwrap());
        assert_eq!(report.fit, resumed_report.fit);
        assert_eq!(report.fit.positions, 24);
        assert_eq!(report.eligible_positions, Some(24));
        assert_eq!(report.calibration_population_positions, 24);
        assert_eq!(report.progress.processed_example_visits, 24 * 8);
        assert_eq!(
            report.fit.observed_context_positions,
            lengths.iter().sum::<usize>()
        );
        assert_eq!(report.supervision_cid.as_deref(), Some(mask_cid.as_str()));
        assert!(report
            .document_exposure
            .iter()
            .all(|doc| doc.eligible_positions == Some(6) && doc.supervised_positions == 6));
        assert_eq!(
            actual
                .memory_read
                .as_ref()
                .unwrap()
                .fit_schedule
                .as_ref()
                .unwrap()
                .supervision_cid,
            Some(mask_cid)
        );
        assert_eq!(actual.geometry, baseline.geometry);
        assert_eq!(actual.lexical_pieces, baseline.lexical_pieces);
        assert_eq!(baseline.to_bytes().unwrap(), baseline_bytes);
        assert!(Model::from_bytes(&actual.to_bytes().unwrap()).is_ok());
    }

    #[test]
    fn native_stream_span_supervision_rejects_invalid_missing_or_changed_masks() {
        let (baseline, documents) = fixture();
        let lengths: Vec<_> = documents
            .iter()
            .map(|doc| baseline.encode(&doc.text).unwrap().len() + 1)
            .collect();
        let all: Vec<_> = lengths
            .iter()
            .map(|&length| {
                vec![MemoryReadTokenSpan {
                    start: 0,
                    end: length,
                }]
            })
            .collect();
        let mask = MemoryReadSupervision::new(&baseline, &documents, all.clone()).unwrap();
        let schedule = MemoryReadSchedule {
            total_positions: 96,
            batch_positions: 7,
        };
        let plain =
            MemoryReadTrainer::new(&baseline, &documents, config(7), schedule, false).unwrap();
        let mut masked = MemoryReadTrainer::new_with_supervision(
            &baseline,
            &documents,
            config(7),
            schedule,
            false,
            mask.clone(),
        )
        .unwrap();
        assert_eq!(plain.selected, masked.selected);
        assert_eq!(plain.state.quotas, masked.state.quotas);
        let plain_json: serde_json::Value =
            serde_json::from_slice(&plain.checkpoint().unwrap()).unwrap();
        assert!(plain_json["state"].get("supervision").is_none());
        assert!(plain_json["state"]["exposure"][0]
            .get("eligible_positions")
            .is_none());
        for invalid_spans in [
            Vec::new(),
            vec![MemoryReadTokenSpan { start: 2, end: 2 }],
            vec![MemoryReadTokenSpan {
                start: 0,
                end: lengths[0] + 1,
            }],
            vec![
                MemoryReadTokenSpan { start: 1, end: 4 },
                MemoryReadTokenSpan { start: 3, end: 5 },
            ],
            vec![
                MemoryReadTokenSpan { start: 4, end: 5 },
                MemoryReadTokenSpan { start: 1, end: 3 },
            ],
        ] {
            let mut bad = mask.clone();
            bad.documents[0].spans = invalid_spans;
            assert!(MemoryReadTrainer::new_with_supervision(
                &baseline,
                &documents,
                config(7),
                schedule,
                false,
                bad
            )
            .is_err());
        }
        let mut missing = mask.clone();
        missing.documents.pop();
        assert!(MemoryReadTrainer::new_with_supervision(
            &baseline,
            &documents,
            config(7),
            schedule,
            false,
            missing
        )
        .is_err());
        let mut changed = mask.clone();
        changed.documents[0].source.text_cid.push('x');
        assert!(MemoryReadTrainer::new_with_supervision(
            &baseline,
            &documents,
            config(7),
            schedule,
            false,
            changed
        )
        .is_err());
        let mut changed = mask.clone();
        changed.baseline_artifact.push('x');
        assert!(MemoryReadTrainer::new_with_supervision(
            &baseline,
            &documents,
            config(7),
            schedule,
            false,
            changed
        )
        .is_err());
        masked.advance(1, Duration::from_secs(10)).unwrap();
        for omitted in [false, true] {
            let mut state = masked.state.clone();
            if omitted {
                state.supervision = None;
            } else {
                state.supervision.as_mut().unwrap().documents[0].spans[0].start += 1;
            }
            let checkpoint = Checkpoint {
                schema: CHECKPOINT_SCHEMA.into(),
                state_cid: identity(&state).unwrap(),
                state,
            };
            assert!(MemoryReadTrainer::restore(
                &baseline,
                &documents,
                &serde_json::to_vec(&checkpoint).unwrap()
            )
            .is_err());
        }
    }
}
