//! Bounded G1 fitting and qualification for issue #951.
//!
//! The command is intentionally one vertical experiment. It consumes the
//! retained G0 control, freezes one small train/held-out split before fitting,
//! updates only the existing layer-29 mixer, and emits the matched real,
//! coordinate-permuted, memory-permuted, and disabled evidence.

use std::collections::HashSet;
use std::io;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use uor_r4_core::transformerless::hf_bpe::{HfBpeTokenizer, TokenizerAdapter};
use uor_r4_model_source::geometric_decoder::{
    GeometricMixer, GeometricOperatorTrace, GeometryIntervention,
};
use uor_r4_model_source::geometric_training::{
    load_passing_preflight, run_mixer_preflight, GeometricMixerCheckpoint,
    GeometricMixerCheckpointBinding, MixerLossSummary, MixerPreflightReport, MixerSpecificTrainer,
    MixerTrainingConfig, MixerTrainingExample, MixerTrainingRound, SourceTrainingTracePoint,
    TrainingMemoryInput, TrainingPrefixKind, TrainingSupportSource, TrainingSupportTarget,
    LOSS_FORMULA,
};
use uor_r4_model_source::{HuggingFaceLlamaOracle, TeacherExecutionConfig, UOR_MATMUL_REVISION};
use uor_r4_router::UorR4Router;

use crate::geometric_decoder::{
    geometry_context_from_router, greedy_token, render_chat_prompt, transcript, validate_source,
    GeometricSpikeReport, RolloutTranscript, FROZEN_PROMPTS, GENERATED_TOKENS,
    PINNED_CHAT_TEMPLATE, PINNED_SOURCE_CID, PINNED_TOKENIZER_CID, SOURCE_REPOSITORY, TARGET_LAYER,
};

pub const QUALIFICATION_SCHEMA: &str = "uor-r4.geometric-mixer-qualification/1";
pub const REVIEW_SCHEMA: &str = "uor-r4.geometric-mixer-operator-review/1";
pub const DATASET_SCHEMA: &str = "uor-r4.geometric-mixer-dataset/1";
pub const DEFAULT_SEED: u64 = 95_120_260_826;
pub const MAX_TRAIN_POSITIONS: usize = 4_096;
pub const MAX_HELD_OUT_POSITIONS: usize = 512;
pub const MAX_ROUNDS: usize = 3;
pub const ROUND_WALL_SECONDS: u64 = 3_600;
pub const MAX_STEPS_PER_ROUND: usize = 80;

#[derive(Clone, Debug)]
pub struct QualificationConfig {
    pub source: PathBuf,
    pub source_revision: String,
    pub g0_report: PathBuf,
    pub preflight_report: PathBuf,
    pub output: PathBuf,
    pub checkpoint: PathBuf,
    pub identity: String,
    pub workers: NonZeroUsize,
    pub seed: u64,
    pub steps_per_round: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualificationSourceBinding {
    pub repository: String,
    pub revision: String,
    pub weights_cid: String,
    pub tokenizer: TokenizerAdapter,
    pub tokenizer_cid: String,
    pub chat_template_cid: String,
    pub base_checkpoint_identity: String,
    pub base_memory_adapter_identity: String,
    pub projection_owner: String,
    pub source_parameters_frozen: bool,
    pub g0_report_cid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatasetExampleRecord {
    pub id: String,
    pub prefix_kind: TrainingPrefixKind,
    pub split: String,
    pub position: usize,
    pub prefix_positions: usize,
    pub memory_examples: usize,
    pub target_token: u32,
    pub q_cid: String,
    pub k_cid: String,
    pub v_cid: String,
    pub logit_cid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualificationDataset {
    pub schema: String,
    pub dataset_cid: String,
    pub split_rule: String,
    pub selected_position_rule: String,
    pub training_positions: usize,
    pub held_out_positions: usize,
    pub teacher_training_positions: usize,
    pub student_training_positions: usize,
    pub teacher_held_out_positions: usize,
    pub student_held_out_positions: usize,
    pub memory_training_positions: usize,
    pub memory_held_out_positions: usize,
    pub maximum_training_positions: usize,
    pub maximum_held_out_positions: usize,
    pub examples: Vec<DatasetExampleRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchedHeldOutResult {
    pub disabled: MixerLossSummary,
    pub real: MixerLossSummary,
    pub permuted_coordinates: MixerLossSummary,
    pub permuted_memory: MixerLossSummary,
    pub relative_real_advantage: f64,
    pub required_relative_advantage: f64,
    pub memory_probability_delta: Option<f64>,
    pub teacher_prefix_real: MixerLossSummary,
    pub teacher_prefix_permuted: MixerLossSummary,
    pub student_prefix_real: MixerLossSummary,
    pub student_prefix_permuted: MixerLossSummary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RolloutArm {
    pub intervention: GeometryIntervention,
    pub transcript: RolloutTranscript,
    pub trace_entries: usize,
    pub mixer_checkpoint_identity: String,
    pub distinct_prefix_support_positions: usize,
    pub distinct_memory_support_spans: usize,
    pub mean_memory_support_weight: f64,
    pub all_support_bounded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperatorPromptReview {
    pub prompt_id: String,
    pub grammatical: bool,
    pub prompt_responsive: bool,
    pub rationale: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperatorReviewFile {
    pub schema: String,
    pub reviews: Vec<OperatorPromptReview>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperatorReviewResult {
    pub schema: String,
    pub reviews: Vec<OperatorPromptReview>,
    pub passing_prompts: usize,
    pub required_passing_prompts: usize,
    pub verdict: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualificationGates {
    pub preflight_passed: bool,
    pub checkpoint_round_trip_passed: bool,
    pub held_out_advantage_passed: bool,
    pub bounded_support_diverse: bool,
    pub memory_load_bearing: bool,
    pub teacher_and_student_reported_separately: bool,
    pub student_prefix_no_short_cycle: bool,
    pub five_real_rollouts: bool,
    pub five_sequences_distinct: bool,
    pub all_real_rollouts_no_short_cycle: bool,
    pub frozen_g0_rubric_preserved: Option<bool>,
    pub machine_verdict: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeometricMixerQualificationReport {
    pub schema: String,
    pub issue: u32,
    pub claim_scope: String,
    pub source: QualificationSourceBinding,
    pub preflight: MixerPreflightReport,
    pub dataset: QualificationDataset,
    pub loss_formula: String,
    pub training_rounds: Vec<MixerTrainingRound>,
    pub held_out: MatchedHeldOutResult,
    pub accepted_checkpoint_identity: String,
    pub accepted_checkpoint_digest: String,
    pub accepted_memory_adapter_identity: String,
    pub real_rollouts: Vec<RolloutArm>,
    pub matched_rollout_controls: Vec<RolloutArm>,
    pub operator_review: Option<OperatorReviewResult>,
    pub gates: QualificationGates,
    pub final_verdict: Option<String>,
    pub report_digest: String,
}

#[derive(Debug)]
pub enum QualificationError {
    Io(io::Error),
    Invalid(String),
    Source(String),
    Training(String),
    Decoder(String),
    Router(String),
}

impl std::fmt::Display for QualificationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Invalid(reason) => write!(formatter, "invalid G1 qualification input: {reason}"),
            Self::Source(reason) => write!(formatter, "G1 source unavailable: {reason}"),
            Self::Training(reason) => write!(formatter, "G1 fitting unavailable: {reason}"),
            Self::Decoder(reason) => write!(formatter, "G1 rollout unavailable: {reason}"),
            Self::Router(reason) => write!(formatter, "G1 memory unavailable: {reason}"),
        }
    }
}

impl std::error::Error for QualificationError {}

impl From<io::Error> for QualificationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn run_preflight_only(
    seed: u64,
    report: &Path,
    checkpoint: &Path,
) -> Result<MixerPreflightReport, QualificationError> {
    run_mixer_preflight(seed, report, checkpoint)
        .map_err(|error| QualificationError::Training(error.to_string()))
}

pub fn run_qualification(
    config: &QualificationConfig,
) -> Result<GeometricMixerQualificationReport, QualificationError> {
    if config.steps_per_round == 0 || config.steps_per_round > MAX_STEPS_PER_ROUND {
        return Err(QualificationError::Invalid(format!(
            "steps per round {} is outside the fixed 1..={MAX_STEPS_PER_ROUND} bound",
            config.steps_per_round
        )));
    }
    if config.identity.trim().is_empty() {
        return Err(QualificationError::Invalid(
            "qualification identity is empty".to_owned(),
        ));
    }
    // This check precedes source validation, tokenizer loading, and every
    // source trace access by construction.
    let preflight = load_passing_preflight(&config.preflight_report)
        .map_err(|error| QualificationError::Training(error.to_string()))?;
    if preflight.seed != config.seed {
        return Err(QualificationError::Invalid(format!(
            "preflight seed {} != requested {}",
            preflight.seed, config.seed
        )));
    }
    validate_source(&config.source, &config.source_revision)
        .map_err(|error| QualificationError::Source(error.to_string()))?;
    let tokenizer = HfBpeTokenizer::from_dir(&config.source)
        .map_err(|error| QualificationError::Source(error.to_string()))?;
    let tokenizer_cid = tokenizer.address();
    if tokenizer_cid != PINNED_TOKENIZER_CID {
        return Err(QualificationError::Invalid(format!(
            "tokenizer {tokenizer_cid} != pinned {PINNED_TOKENIZER_CID}"
        )));
    }
    let g0_bytes = std::fs::read(&config.g0_report)?;
    let g0: GeometricSpikeReport = serde_json::from_slice(&g0_bytes)
        .map_err(|error| QualificationError::Invalid(format!("G0 report: {error}")))?;
    validate_g0(&g0, &config.source_revision, &tokenizer)?;
    let rendered = FROZEN_PROMPTS
        .iter()
        .map(|prompt| render_chat_prompt(prompt))
        .collect::<Vec<_>>();
    let inputs = rendered
        .iter()
        .map(|prompt| tokenizer.encode(prompt))
        .collect::<Vec<_>>();
    let horizon = inputs
        .iter()
        .map(|input| input.len() + GENERATED_TOKENS)
        .max()
        .ok_or_else(|| QualificationError::Invalid("frozen prompt set is empty".to_owned()))?;
    let oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
        &config.source,
        horizon,
        TeacherExecutionConfig::fixed_workers(config.workers),
    )
    .map_err(|error| QualificationError::Source(error.to_string()))?;
    if oracle.source_cid() != PINNED_SOURCE_CID || oracle.cfg().vocab != tokenizer.vocab_size() {
        return Err(QualificationError::Invalid(
            "source weights/vocabulary do not match the frozen G0 binding".to_owned(),
        ));
    }
    let mut mixer_seed = Vec::new();
    mixer_seed.extend_from_slice(oracle.source_cid().as_bytes());
    mixer_seed.extend_from_slice(tokenizer_cid.as_bytes());
    mixer_seed.extend_from_slice(b"issue-950-one-layer-r4-spike");
    let base_mixer = GeometricMixer::deterministic(TARGET_LAYER, oracle.cfg().dim, &mixer_seed)
        .map_err(|error| QualificationError::Decoder(error.to_string()))?;
    let base_identity = base_mixer.checkpoint_identity();
    if g0
        .treatment
        .first()
        .is_none_or(|treatment| treatment.mixer_checkpoint_identity != base_identity)
    {
        return Err(QualificationError::Invalid(
            "reconstructed base checkpoint differs from G0".to_owned(),
        ));
    }
    let base_adapter = base_mixer.memory_adapter_identity(oracle.source_cid(), &tokenizer_cid);
    if base_adapter != g0.memory_adapter_identity {
        return Err(QualificationError::Invalid(
            "reconstructed base memory adapter differs from G0".to_owned(),
        ));
    }

    let (train, held_out, dataset) = build_dataset(
        &oracle,
        &tokenizer,
        &inputs,
        &g0,
        &base_adapter,
        config.seed,
    )?;
    if train.len() > MAX_TRAIN_POSITIONS || held_out.len() > MAX_HELD_OUT_POSITIONS {
        return Err(QualificationError::Invalid(
            "frozen split exceeds the hard G1 position caps".to_owned(),
        ));
    }
    let training_config = MixerTrainingConfig::issue_951(config.seed);
    let mut trainer =
        MixerSpecificTrainer::new(base_mixer.clone(), training_config.clone(), config.workers)
            .map_err(|error| QualificationError::Training(error.to_string()))?;
    let mut rounds = Vec::new();
    for _round_index in 0..MAX_ROUNDS {
        let round = trainer
            .train(
                &train,
                config.steps_per_round,
                Duration::from_secs(ROUND_WALL_SECONDS),
            )
            .map_err(|error| QualificationError::Training(error.to_string()))?;
        rounds.push(round);
        let matched = evaluate_matched(&trainer, &held_out)?;
        if held_out_advantages_pass(&matched)
            && matched
                .memory_probability_delta
                .is_some_and(|delta| delta > 0.0)
        {
            break;
        }
    }
    let held_out_result = evaluate_matched(&trainer, &held_out)?;
    let checkpoint_binding = GeometricMixerCheckpointBinding {
        source_cid: oracle.source_cid().to_owned(),
        tokenizer_cid: tokenizer_cid.clone(),
        base_checkpoint_identity: base_identity.clone(),
        dataset_cid: dataset.dataset_cid.clone(),
        seed: config.seed,
        training_config,
        projection_owner: format!("uor-matmul exact GEMM@{UOR_MATMUL_REVISION}"),
    };
    let checkpoint = GeometricMixerCheckpoint::new(checkpoint_binding, trainer.mixer().clone())
        .map_err(|error| QualificationError::Training(error.to_string()))?;
    checkpoint
        .save(&config.checkpoint)
        .map_err(|error| QualificationError::Training(error.to_string()))?;
    let reloaded = GeometricMixerCheckpoint::load(&config.checkpoint)
        .map_err(|error| QualificationError::Training(error.to_string()))?;
    let checkpoint_round_trip_passed = reloaded.content_digest == checkpoint.content_digest
        && reloaded.mixer.checkpoint_identity() == checkpoint.mixer.checkpoint_identity();
    let accepted_adapter = checkpoint
        .mixer
        .memory_adapter_identity(oracle.source_cid(), &tokenizer_cid);
    let (real_rollouts, matched_rollout_controls) = run_rollouts(
        &oracle,
        &tokenizer,
        &inputs,
        &checkpoint.mixer,
        &accepted_adapter,
        &config.identity,
    )?;
    let gates = machine_gates(
        &preflight,
        checkpoint_round_trip_passed,
        &held_out_result,
        &real_rollouts,
    );
    let final_verdict = (gates.machine_verdict != "PASS_PENDING_OPERATOR_REVIEW")
        .then_some("REDESIGN_REPRESENTATION".to_owned());
    let mut report = GeometricMixerQualificationReport {
        schema: QUALIFICATION_SCHEMA.to_owned(),
        issue: 951,
        claim_scope: "bounded empirical qualification of one experimental layer-29 mixer and tokenizer-bound memory adapter; no all-layer transformerless, production, broad quality, performance, or multiplication-free claim".to_owned(),
        source: QualificationSourceBinding {
            repository: SOURCE_REPOSITORY.to_owned(),
            revision: config.source_revision.clone(),
            weights_cid: oracle.source_cid().to_owned(),
            tokenizer: tokenizer.adapter(),
            tokenizer_cid: tokenizer_cid.clone(),
            chat_template_cid: format!("blake3:{}", blake3::hash(PINNED_CHAT_TEMPLATE.as_bytes()).to_hex()),
            base_checkpoint_identity: base_identity,
            base_memory_adapter_identity: base_adapter,
            projection_owner: format!("uor-matmul exact GEMM@{UOR_MATMUL_REVISION}"),
            source_parameters_frozen: true,
            g0_report_cid: format!("blake3:{}", blake3::hash(&g0_bytes).to_hex()),
        },
        preflight,
        dataset,
        loss_formula: LOSS_FORMULA.to_owned(),
        training_rounds: rounds,
        held_out: held_out_result,
        accepted_checkpoint_identity: checkpoint.mixer.checkpoint_identity(),
        accepted_checkpoint_digest: checkpoint.content_digest,
        accepted_memory_adapter_identity: accepted_adapter,
        real_rollouts,
        matched_rollout_controls,
        operator_review: None,
        gates,
        final_verdict,
        report_digest: String::new(),
    };
    report.report_digest = report_digest(&report)?;
    write_json(&config.output, &report)?;
    Ok(report)
}

fn validate_g0(
    g0: &GeometricSpikeReport,
    revision: &str,
    tokenizer: &HfBpeTokenizer,
) -> Result<(), QualificationError> {
    if g0.issue != 950
        || g0.gates.verdict != "PASS"
        || g0.source.repository != SOURCE_REPOSITORY
        || g0.source.revision != revision
        || g0.source.weights_cid != PINNED_SOURCE_CID
        || g0.source.tokenizer.tokenizer_cid != PINNED_TOKENIZER_CID
        || g0.source.tokenizer != tokenizer.adapter()
        || g0.control.len() != FROZEN_PROMPTS.len()
        || g0.treatment.len() != 1
        || g0.treatment[0].rollout.prompt_id != "G0-P1"
    {
        return Err(QualificationError::Invalid(
            "G0 report is not the promoted frozen control/treatment record".to_owned(),
        ));
    }
    Ok(())
}

#[derive(Clone)]
struct SequenceInput {
    id: String,
    kind: TrainingPrefixKind,
    prompt_index: usize,
    input_tokens: usize,
    tokens: Vec<u32>,
}

#[allow(clippy::too_many_arguments)]
fn build_dataset(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &HfBpeTokenizer,
    inputs: &[Vec<u32>],
    g0: &GeometricSpikeReport,
    base_adapter: &str,
    seed: u64,
) -> Result<
    (
        Vec<MixerTrainingExample>,
        Vec<MixerTrainingExample>,
        QualificationDataset,
    ),
    QualificationError,
> {
    let mut sequences = Vec::new();
    for (prompt_index, input) in inputs.iter().take(2).enumerate() {
        let mut tokens = input.clone();
        tokens.extend_from_slice(&g0.control[prompt_index].generated_token_ids);
        sequences.push(SequenceInput {
            id: format!("teacher-G0-P{}", prompt_index + 1),
            kind: TrainingPrefixKind::Teacher,
            prompt_index,
            input_tokens: input.len(),
            tokens,
        });
    }
    let mut student_tokens = inputs[0].clone();
    student_tokens.extend_from_slice(&g0.treatment[0].rollout.generated_token_ids);
    sequences.push(SequenceInput {
        id: "student-G0-P1".to_owned(),
        kind: TrainingPrefixKind::Student,
        prompt_index: 0,
        input_tokens: inputs[0].len(),
        tokens: student_tokens,
    });

    let mut train = Vec::new();
    let mut held_out = Vec::new();
    let mut records = Vec::new();
    for sequence in sequences {
        let trace = oracle
            .capture_geometric_training_sequence(&sequence.tokens, TARGET_LAYER)
            .map_err(|error| QualificationError::Source(error.to_string()))?;
        let memory =
            persistent_training_memory(oracle, tokenizer, sequence.prompt_index, base_adapter)?;
        let positions = selected_positions(sequence.input_tokens, trace.len());
        for (selection_index, position) in positions.into_iter().enumerate() {
            let point = trace.get(position).ok_or_else(|| {
                QualificationError::Invalid(format!(
                    "{} selected missing trace position {position}",
                    sequence.id
                ))
            })?;
            let target_token = sequence.tokens[position + 1];
            let candidate_tokens = sampled_token_candidates(
                target_token,
                oracle.cfg().vocab,
                &sequence.id,
                position,
                seed,
            );
            let next_token_candidate_embeddings =
                oracle
                    .source_embedding_rows(&candidate_tokens)
                    .map_err(|error| QualificationError::Source(error.to_string()))?;
            let support_target = support_target(point, 0.20)?;
            let id = format!("{}-pos-{position}", sequence.id);
            let example = MixerTrainingExample {
                id: id.clone(),
                prefix_kind: sequence.kind.clone(),
                normalized_prefix: trace[..=position]
                    .iter()
                    .map(|item| item.normalized_residual.clone())
                    .collect(),
                session_route_state: memory.1,
                memories: vec![memory.0.clone()],
                target_attention_output: point.attention_output.clone(),
                support_target,
                next_token_candidate_embeddings,
            };
            example
                .validate(oracle.cfg().dim)
                .map_err(|error| QualificationError::Invalid(error.to_string()))?;
            // Every fourth selected position in each source sequence is held
            // out. This is frozen before the first update and guarantees both
            // teacher and student rows in each partition.
            let split = if selection_index.is_multiple_of(4) {
                held_out.push(example);
                "held_out"
            } else {
                train.push(example);
                "train"
            };
            records.push(DatasetExampleRecord {
                id,
                prefix_kind: sequence.kind.clone(),
                split: split.to_owned(),
                position,
                prefix_positions: position + 1,
                memory_examples: 1,
                target_token,
                q_cid: point.q_cid.clone(),
                k_cid: point.k_cid.clone(),
                v_cid: point.v_cid.clone(),
                logit_cid: logits_cid(&point.logits),
            });
        }
    }
    if train.is_empty() || held_out.is_empty() {
        return Err(QualificationError::Invalid(
            "frozen dataset split produced an empty partition".to_owned(),
        ));
    }
    let dataset_cid = dataset_cid(&train, &held_out, &records);
    let count = |split: &str, kind: TrainingPrefixKind| {
        records
            .iter()
            .filter(|record| record.split == split && record.prefix_kind == kind)
            .count()
    };
    let dataset = QualificationDataset {
        schema: DATASET_SCHEMA.to_owned(),
        dataset_cid,
        split_rule: "within each frozen source sequence, selected-position index modulo 4 equals 0 -> held_out; all other selected positions -> train"
            .to_owned(),
        selected_position_rule: "last two prompt decisions plus generated-prefix offsets 3,7,11,15,19,23,27, clipped to the recorded 32-token horizon"
            .to_owned(),
        training_positions: train.len(),
        held_out_positions: held_out.len(),
        teacher_training_positions: count("train", TrainingPrefixKind::Teacher),
        student_training_positions: count("train", TrainingPrefixKind::Student),
        teacher_held_out_positions: count("held_out", TrainingPrefixKind::Teacher),
        student_held_out_positions: count("held_out", TrainingPrefixKind::Student),
        memory_training_positions: train.len(),
        memory_held_out_positions: held_out.len(),
        maximum_training_positions: MAX_TRAIN_POSITIONS,
        maximum_held_out_positions: MAX_HELD_OUT_POSITIONS,
        examples: records,
    };
    Ok((train, held_out, dataset))
}

fn selected_positions(input_tokens: usize, trace_positions: usize) -> Vec<usize> {
    let mut positions = vec![
        input_tokens.saturating_sub(2),
        input_tokens.saturating_sub(1),
    ];
    for offset in [3usize, 7, 11, 15, 19, 23, 27] {
        positions.push(input_tokens.saturating_sub(1).saturating_add(offset));
    }
    positions.retain(|position| *position < trace_positions);
    positions.sort_unstable();
    positions.dedup();
    positions
}

fn persistent_training_memory(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &HfBpeTokenizer,
    prompt_index: usize,
    adapter: &str,
) -> Result<(TrainingMemoryInput, [f32; 4]), QualificationError> {
    let mut router = UorR4Router::new(0.5);
    let prompt = FROZEN_PROMPTS[prompt_index];
    let tokens = tokenizer.encode(prompt);
    router
        .commit_tokenizer_bound_turn(
            "issue-951-training-memory",
            "user",
            prompt,
            &tokens,
            PINNED_TOKENIZER_CID,
            adapter,
            oracle.source_cid(),
        )
        .map_err(|error| QualificationError::Router(error.to_string()))?;
    let spans = router
        .latest_tokenizer_bound_turns(
            "issue-951-training-memory",
            PINNED_TOKENIZER_CID,
            adapter,
            oracle.source_cid(),
            1,
            256,
        )
        .map_err(|error| QualificationError::Router(error.to_string()))?;
    let span = spans.first().ok_or_else(|| {
        QualificationError::Router("persistent training memory was not retained".to_owned())
    })?;
    let rows = oracle
        .source_embedding_rows(&span.token_ids)
        .map_err(|error| QualificationError::Source(error.to_string()))?;
    let mut mean = vec![0.0f32; oracle.cfg().dim];
    for row in rows {
        for (sum, value) in mean.iter_mut().zip(row) {
            *sum += value;
        }
    }
    for value in &mut mean {
        *value /= span.token_ids.len() as f32;
    }
    let coordinates = span.r4_coordinates.map(|value| value as f32);
    Ok((
        TrainingMemoryInput {
            span_index: 0,
            mean_embedding: mean,
            r4_coordinates: coordinates,
        },
        coordinates,
    ))
}

fn support_target(
    point: &SourceTrainingTracePoint,
    memory_probability: f32,
) -> Result<Vec<TrainingSupportTarget>, QualificationError> {
    if point.mean_attention_support.is_empty() {
        return Err(QualificationError::Invalid(
            "source trace has no attention support".to_owned(),
        ));
    }
    let mut order = (0..point.mean_attention_support.len()).collect::<Vec<_>>();
    order.sort_by(|&left, &right| {
        point.mean_attention_support[right]
            .total_cmp(&point.mean_attention_support[left])
            .then_with(|| left.cmp(&right))
    });
    order.truncate(4.min(order.len()));
    let selected_sum = order
        .iter()
        .map(|&index| point.mean_attention_support[index])
        .sum::<f32>()
        .max(f32::MIN_POSITIVE);
    let mut targets = order
        .into_iter()
        .map(|index| TrainingSupportTarget {
            source: TrainingSupportSource::Prefix,
            index,
            probability: (1.0 - memory_probability) * point.mean_attention_support[index]
                / selected_sum,
        })
        .collect::<Vec<_>>();
    targets.push(TrainingSupportTarget {
        source: TrainingSupportSource::Memory,
        index: 0,
        probability: memory_probability,
    });
    Ok(targets)
}

fn sampled_token_candidates(
    target: u32,
    vocabulary: usize,
    sequence: &str,
    position: usize,
    seed: u64,
) -> Vec<u32> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.issue-951-sampled-token-candidates/1");
    hasher.update(&seed.to_le_bytes());
    hasher.update(sequence.as_bytes());
    hasher.update(&(position as u64).to_le_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest.as_bytes()[..8]);
    let mut state = u64::from_le_bytes(bytes).max(1);
    let mut output = vec![target];
    let mut seen = HashSet::from([target]);
    while output.len() < 16 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let candidate = (state % vocabulary as u64) as u32;
        if seen.insert(candidate) {
            output.push(candidate);
        }
    }
    output
}

fn logits_cid(logits: &[f32]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.issue-951-source-logits/1");
    for value in logits {
        hasher.update(&value.to_bits().to_le_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn dataset_cid(
    train: &[MixerTrainingExample],
    held_out: &[MixerTrainingExample],
    records: &[DatasetExampleRecord],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(DATASET_SCHEMA.as_bytes());
    for (split, examples) in [("train", train), ("held_out", held_out)] {
        hasher.update(split.as_bytes());
        for example in examples {
            hasher.update(example.id.as_bytes());
            hasher.update(match example.prefix_kind {
                TrainingPrefixKind::Teacher => b"teacher",
                TrainingPrefixKind::Student => b"student",
            });
            for value in &example.target_attention_output {
                hasher.update(&value.to_bits().to_le_bytes());
            }
            for target in &example.support_target {
                hasher.update(match target.source {
                    TrainingSupportSource::Prefix => b"prefix",
                    TrainingSupportSource::Memory => b"memory",
                });
                hasher.update(&(target.index as u64).to_le_bytes());
                hasher.update(&target.probability.to_bits().to_le_bytes());
            }
            for embedding in &example.next_token_candidate_embeddings {
                for value in embedding {
                    hasher.update(&value.to_bits().to_le_bytes());
                }
            }
        }
    }
    for record in records {
        hasher.update(record.q_cid.as_bytes());
        hasher.update(record.k_cid.as_bytes());
        hasher.update(record.v_cid.as_bytes());
        hasher.update(record.logit_cid.as_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn evaluate_matched(
    trainer: &MixerSpecificTrainer,
    held_out: &[MixerTrainingExample],
) -> Result<MatchedHeldOutResult, QualificationError> {
    let disabled = trainer
        .evaluate_disabled_reference(held_out)
        .map_err(|error| QualificationError::Training(error.to_string()))?;
    let real = trainer
        .evaluate(held_out, GeometryIntervention::Real)
        .map_err(|error| QualificationError::Training(error.to_string()))?;
    let permuted_coordinates = trainer
        .evaluate(held_out, GeometryIntervention::PermutedCoordinates)
        .map_err(|error| QualificationError::Training(error.to_string()))?;
    let permuted_memory = trainer
        .evaluate(held_out, GeometryIntervention::PermutedMemory)
        .map_err(|error| QualificationError::Training(error.to_string()))?;
    let teacher = held_out
        .iter()
        .filter(|example| example.prefix_kind == TrainingPrefixKind::Teacher)
        .cloned()
        .collect::<Vec<_>>();
    let student = held_out
        .iter()
        .filter(|example| example.prefix_kind == TrainingPrefixKind::Student)
        .cloned()
        .collect::<Vec<_>>();
    if teacher.is_empty() || student.is_empty() {
        return Err(QualificationError::Invalid(
            "held-out split lacks teacher or student prefixes".to_owned(),
        ));
    }
    Ok(MatchedHeldOutResult {
        relative_real_advantage: relative_advantage(real.total, permuted_coordinates.total),
        required_relative_advantage: 0.05,
        memory_probability_delta: probability_delta(&real, &permuted_memory),
        teacher_prefix_real: trainer
            .evaluate(&teacher, GeometryIntervention::Real)
            .map_err(|error| QualificationError::Training(error.to_string()))?,
        teacher_prefix_permuted: trainer
            .evaluate(&teacher, GeometryIntervention::PermutedCoordinates)
            .map_err(|error| QualificationError::Training(error.to_string()))?,
        student_prefix_real: trainer
            .evaluate(&student, GeometryIntervention::Real)
            .map_err(|error| QualificationError::Training(error.to_string()))?,
        student_prefix_permuted: trainer
            .evaluate(&student, GeometryIntervention::PermutedCoordinates)
            .map_err(|error| QualificationError::Training(error.to_string()))?,
        disabled,
        real,
        permuted_coordinates,
        permuted_memory,
    })
}

fn relative_advantage(real: f64, permuted: f64) -> f64 {
    if permuted > f64::EPSILON {
        (permuted - real) / permuted
    } else {
        0.0
    }
}

fn probability_delta(real: &MixerLossSummary, permuted: &MixerLossSummary) -> Option<f64> {
    real.mean_target_memory_probability
        .zip(permuted.mean_target_memory_probability)
        .map(|(left, right)| left - right)
}

fn held_out_advantages_pass(held_out: &MatchedHeldOutResult) -> bool {
    let required = held_out.required_relative_advantage;
    held_out.relative_real_advantage >= required
        && relative_advantage(
            held_out.teacher_prefix_real.total,
            held_out.teacher_prefix_permuted.total,
        ) >= required
        && relative_advantage(
            held_out.student_prefix_real.total,
            held_out.student_prefix_permuted.total,
        ) >= required
}

fn run_rollouts(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &HfBpeTokenizer,
    inputs: &[Vec<u32>],
    mixer: &GeometricMixer,
    adapter: &str,
    identity: &str,
) -> Result<(Vec<RolloutArm>, Vec<RolloutArm>), QualificationError> {
    let mut real = Vec::with_capacity(FROZEN_PROMPTS.len());
    for (index, input) in inputs.iter().enumerate() {
        real.push(rollout_arm(
            oracle,
            tokenizer,
            mixer,
            adapter,
            &format!("{identity}-P{}", index + 1),
            &format!("G1-P{}", index + 1),
            FROZEN_PROMPTS[index],
            input,
            GeometryIntervention::Real,
        )?);
    }
    let mut controls = Vec::new();
    for intervention in [
        GeometryIntervention::Disabled,
        GeometryIntervention::PermutedCoordinates,
        GeometryIntervention::PermutedMemory,
    ] {
        controls.push(rollout_arm(
            oracle,
            tokenizer,
            mixer,
            adapter,
            &format!("{identity}-P1"),
            "G1-P1",
            FROZEN_PROMPTS[0],
            &inputs[0],
            intervention,
        )?);
    }
    Ok((real, controls))
}

#[allow(clippy::too_many_arguments)]
fn rollout_arm(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &HfBpeTokenizer,
    mixer: &GeometricMixer,
    adapter: &str,
    identity: &str,
    prompt_id: &str,
    prompt: &str,
    input: &[u32],
    intervention: GeometryIntervention,
) -> Result<RolloutArm, QualificationError> {
    if input.is_empty() {
        return Err(QualificationError::Invalid(
            "rollout prompt tokenized to an empty sequence".to_owned(),
        ));
    }
    let mut router = UorR4Router::new(0.5);
    let prompt_tokens = tokenizer.encode(prompt);
    router
        .commit_tokenizer_bound_turn(
            identity,
            "user",
            prompt,
            &prompt_tokens,
            PINNED_TOKENIZER_CID,
            adapter,
            oracle.source_cid(),
        )
        .map_err(|error| QualificationError::Router(error.to_string()))?;
    let context = geometry_context_from_router(
        &router,
        identity,
        PINNED_TOKENIZER_CID,
        adapter,
        oracle.source_cid(),
    )
    .map_err(|error| QualificationError::Router(error.to_string()))?;
    let mut session = oracle
        .new_geometric_session(
            mixer.clone(),
            context,
            intervention,
            input.len() + GENERATED_TOKENS,
        )
        .map_err(|error| QualificationError::Decoder(error.to_string()))?;
    let mut logits = vec![0.0f32; oracle.cfg().vocab];
    for (position, &token) in input[..input.len() - 1].iter().enumerate() {
        oracle
            .step_geometric(&mut session, token as usize, position, &mut logits)
            .map_err(|error| QualificationError::Decoder(error.to_string()))?;
    }
    session.clear_traces();
    let decision_position = input.len() - 1;
    oracle
        .step_geometric(
            &mut session,
            input[decision_position] as usize,
            decision_position,
            &mut logits,
        )
        .map_err(|error| QualificationError::Decoder(error.to_string()))?;
    let mut generated = Vec::with_capacity(GENERATED_TOKENS);
    generated.push(
        greedy_token(&logits).map_err(|error| QualificationError::Decoder(error.to_string()))?,
    );
    for offset in 1..GENERATED_TOKENS {
        let position = input.len() - 1 + offset;
        oracle
            .step_geometric(
                &mut session,
                generated[offset - 1] as usize,
                position,
                &mut logits,
            )
            .map_err(|error| QualificationError::Decoder(error.to_string()))?;
        generated.push(
            greedy_token(&logits)
                .map_err(|error| QualificationError::Decoder(error.to_string()))?,
        );
    }
    let traces = session.traces();
    let (prefix, memory, mean_memory_weight, bounded) = support_stats(traces);
    Ok(RolloutArm {
        intervention,
        transcript: transcript(tokenizer, prompt_id, prompt, input.len(), generated),
        trace_entries: traces.len(),
        mixer_checkpoint_identity: session.checkpoint_identity(),
        distinct_prefix_support_positions: prefix,
        distinct_memory_support_spans: memory,
        mean_memory_support_weight: mean_memory_weight,
        all_support_bounded: bounded,
    })
}

fn support_stats(traces: &[GeometricOperatorTrace]) -> (usize, usize, f64, bool) {
    let mut prefix = HashSet::new();
    let mut memory = HashSet::new();
    let mut memory_weight = 0.0f64;
    let mut memory_entries = 0usize;
    let bounded = traces.iter().all(|trace| {
        !trace.selected_support.is_empty()
            && trace.selected_support.len() <= trace.support_budget
            && trace.support_budget == 4
    });
    for trace in traces {
        for support in &trace.selected_support {
            if support.source == "prefix" {
                prefix.insert(support.index);
            } else if support.source == "memory" {
                memory.insert(support.index);
                memory_weight += f64::from(support.weight);
                memory_entries += 1;
            }
        }
    }
    (
        prefix.len(),
        memory.len(),
        if memory_entries > 0 {
            memory_weight / memory_entries as f64
        } else {
            0.0
        },
        bounded,
    )
}

fn machine_gates(
    preflight: &MixerPreflightReport,
    checkpoint_round_trip_passed: bool,
    held_out: &MatchedHeldOutResult,
    real_rollouts: &[RolloutArm],
) -> QualificationGates {
    let held_out_advantage_passed = held_out_advantages_pass(held_out);
    let bounded_support_diverse = held_out.real.distinct_selected_prefix_positions > 1
        && real_rollouts
            .iter()
            .all(|rollout| rollout.all_support_bounded)
        && real_rollouts
            .iter()
            .any(|rollout| rollout.distinct_prefix_support_positions > 1)
        && real_rollouts
            .iter()
            .filter(|rollout| rollout.distinct_memory_support_spans > 0)
            .count()
            >= 2;
    let memory_load_bearing = held_out
        .memory_probability_delta
        .is_some_and(|delta| delta > 0.0)
        && held_out.real.distinct_selected_memories > 0;
    let teacher_and_student_reported_separately =
        held_out.teacher_prefix_real.examples > 0 && held_out.student_prefix_real.examples > 0;
    let student_prefix_no_short_cycle = real_rollouts
        .first()
        .is_some_and(|rollout| rollout.transcript.short_cycle_period.is_none());
    let five_real_rollouts = real_rollouts.len() == FROZEN_PROMPTS.len();
    let five_sequences_distinct = real_rollouts
        .iter()
        .map(|rollout| rollout.transcript.generated_token_ids.as_slice())
        .collect::<HashSet<_>>()
        .len()
        == FROZEN_PROMPTS.len();
    let all_real_rollouts_no_short_cycle = real_rollouts
        .iter()
        .all(|rollout| rollout.transcript.short_cycle_period.is_none());
    let pass = preflight.verdict == "PASS"
        && checkpoint_round_trip_passed
        && held_out_advantage_passed
        && bounded_support_diverse
        && memory_load_bearing
        && teacher_and_student_reported_separately
        && student_prefix_no_short_cycle
        && five_real_rollouts
        && five_sequences_distinct
        && all_real_rollouts_no_short_cycle;
    QualificationGates {
        preflight_passed: preflight.verdict == "PASS",
        checkpoint_round_trip_passed,
        held_out_advantage_passed,
        bounded_support_diverse,
        memory_load_bearing,
        teacher_and_student_reported_separately,
        student_prefix_no_short_cycle,
        five_real_rollouts,
        five_sequences_distinct,
        all_real_rollouts_no_short_cycle,
        frozen_g0_rubric_preserved: None,
        machine_verdict: if pass {
            "PASS_PENDING_OPERATOR_REVIEW"
        } else {
            "FAIL"
        }
        .to_owned(),
    }
}

pub fn finalize_operator_review(
    report_path: &Path,
    review_path: &Path,
) -> Result<GeometricMixerQualificationReport, QualificationError> {
    let report_bytes = std::fs::read(report_path)?;
    let mut report: GeometricMixerQualificationReport = serde_json::from_slice(&report_bytes)
        .map_err(|error| QualificationError::Invalid(format!("qualification report: {error}")))?;
    if report.schema != QUALIFICATION_SCHEMA || report.report_digest != report_digest(&report)? {
        return Err(QualificationError::Invalid(
            "qualification report schema/digest does not validate".to_owned(),
        ));
    }
    if report.operator_review.is_some() || report.final_verdict.is_some() {
        return Err(QualificationError::Invalid(
            "qualification report is already terminal".to_owned(),
        ));
    }
    let review_bytes = std::fs::read(review_path)?;
    let review: OperatorReviewFile = serde_json::from_slice(&review_bytes)
        .map_err(|error| QualificationError::Invalid(format!("operator review: {error}")))?;
    if review.schema != REVIEW_SCHEMA || review.reviews.len() != FROZEN_PROMPTS.len() {
        return Err(QualificationError::Invalid(
            "operator review must contain exactly the frozen five prompts".to_owned(),
        ));
    }
    let transcript_ids = report
        .real_rollouts
        .iter()
        .map(|rollout| rollout.transcript.prompt_id.clone())
        .collect::<HashSet<_>>();
    let review_ids = review
        .reviews
        .iter()
        .map(|item| item.prompt_id.clone())
        .collect::<HashSet<_>>();
    if transcript_ids != review_ids
        || review
            .reviews
            .iter()
            .any(|item| item.rationale.trim().is_empty())
    {
        return Err(QualificationError::Invalid(
            "operator review IDs/rationales do not bind the retained transcripts".to_owned(),
        ));
    }
    let passing_prompts = review
        .reviews
        .iter()
        .filter(|item| item.grammatical && item.prompt_responsive)
        .count();
    let rubric_passed = passing_prompts >= 4;
    report.gates.frozen_g0_rubric_preserved = Some(rubric_passed);
    let promote = report.gates.machine_verdict == "PASS_PENDING_OPERATOR_REVIEW" && rubric_passed;
    let final_verdict = if promote {
        "PROMOTE_TO_ALL_LAYERS"
    } else {
        "REDESIGN_REPRESENTATION"
    };
    report.operator_review = Some(OperatorReviewResult {
        schema: REVIEW_SCHEMA.to_owned(),
        reviews: review.reviews,
        passing_prompts,
        required_passing_prompts: 4,
        verdict: if rubric_passed { "PASS" } else { "FAIL" }.to_owned(),
    });
    report.final_verdict = Some(final_verdict.to_owned());
    report.report_digest.clear();
    report.report_digest = report_digest(&report)?;
    write_json(report_path, &report)?;
    Ok(report)
}

fn report_digest(report: &GeometricMixerQualificationReport) -> Result<String, QualificationError> {
    let mut content = report.clone();
    content.report_digest.clear();
    let bytes = serde_json::to_vec(&content)
        .map_err(|error| QualificationError::Invalid(error.to_string()))?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), QualificationError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| QualificationError::Invalid(error.to_string()))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_dataset_positions_are_bounded_and_deterministic() {
        let positions = selected_positions(40, 71);
        assert_eq!(positions, vec![38, 39, 42, 46, 50, 54, 58, 62, 66]);
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn sampled_candidates_bind_true_token_at_row_zero_without_duplicates() {
        let candidates = sampled_token_candidates(17, 128, "teacher-G0-P1", 39, DEFAULT_SEED);
        assert_eq!(candidates.len(), 16);
        assert_eq!(candidates[0], 17);
        assert_eq!(candidates.iter().copied().collect::<HashSet<_>>().len(), 16);
    }

    #[test]
    fn positive_gate_requires_teacher_and_student_advantage() {
        let summary = |examples, total| MixerLossSummary {
            examples,
            total,
            ..MixerLossSummary::default()
        };
        let held_out = MatchedHeldOutResult {
            disabled: summary(2, 0.0),
            real: summary(2, 0.90),
            permuted_coordinates: summary(2, 1.0),
            permuted_memory: summary(2, 1.0),
            relative_real_advantage: 0.10,
            required_relative_advantage: 0.05,
            memory_probability_delta: Some(0.1),
            teacher_prefix_real: summary(1, 0.90),
            teacher_prefix_permuted: summary(1, 1.0),
            student_prefix_real: summary(1, 0.97),
            student_prefix_permuted: summary(1, 1.0),
        };
        assert!(!held_out_advantages_pass(&held_out));
    }
}
