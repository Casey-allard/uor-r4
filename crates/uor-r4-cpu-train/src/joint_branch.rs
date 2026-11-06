//! Learned selection over complete branches cut from one geometric content spine.
//!
//! The spine never commits a stop. Every unchanged reply-boundary hit along it
//! becomes a complete candidate. A small integer pairwise ranker chooses among
//! those candidates using evidence from the whole generated prefix and the
//! proposed endpoint. Evaluation references affect scoring only after selection.

use std::cmp::Reverse;
use std::collections::BTreeSet;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uor_r4_integer::stack::{reply_stop, ReplyEnd};

use crate::copy_key_reply::{
    compare_copy_key_reply_rows, copy_key_reply_decision, CopyKeyReplyArm, CopyKeyReplyComparison,
    CopyKeyReplyMetric, CopyKeyReplyRow,
};
use crate::corpus::CpuConversationSample;
use crate::reply_boundary::{ReplyBoundaryHit, ReplyBoundaryMemory};
use crate::reply_eval::normalized_reference;
use crate::suffix_memory::{
    derive_full_sequence_suffix_content_memory, SuffixMemoryPrediction, SuffixMemoryTable,
    SUFFIX_MEMORY_MAX_DEPTH,
};
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const JOINT_BRANCH_FEATURES: usize = 15;
pub const JOINT_BRANCH_EPOCHS: usize = 8;
pub const JOINT_BRANCH_LEARNING_RATE: i64 = 1;
pub const JOINT_BRANCH_WEIGHT_LIMIT: i64 = 65_536;
pub const JOINT_BRANCH_MIN_OBSERVATIONS: u64 = 2;
pub const JOINT_BRANCH_SHUFFLE_SEED: u64 = 231;
pub const JOINT_BRANCH_TAIL: usize = 8;
pub const JOINT_BRANCH_FOLDS: usize = 4;

pub const JOINT_BRANCH_FEATURE_NAMES: [&str; JOINT_BRANCH_FEATURES] = [
    "bias",
    "branch_length",
    "boundary_depth",
    "boundary_terminal",
    "local_decisions",
    "global_decisions",
    "global_depth_sum",
    "global_observations_sum",
    "global_successor_margin_sum",
    "stronger_continuations",
    "writer_fallbacks",
    "tail_local_decisions",
    "tail_global_decisions",
    "repeated_four_grams",
    "endpoint_global_advantage",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JointBranchArm {
    Learned,
    ShuffledLabels,
    FirstBoundary,
    LongestSpine,
    BoundaryTerminalVolume,
    ReferenceEndpointOracle,
}

impl JointBranchArm {
    pub const EVALUATION_ARMS: [Self; 6] = [
        Self::Learned,
        Self::ShuffledLabels,
        Self::FirstBoundary,
        Self::LongestSpine,
        Self::BoundaryTerminalVolume,
        Self::ReferenceEndpointOracle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Learned => "learned_whole_branch",
            Self::ShuffledLabels => "shuffled_label_whole_branch",
            Self::FirstBoundary => "first_boundary",
            Self::LongestSpine => "longest_spine",
            Self::BoundaryTerminalVolume => "boundary_terminal_volume",
            Self::ReferenceEndpointOracle => "reference_endpoint_oracle",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointBranchSelector {
    pub weights: [i64; JOINT_BRANCH_FEATURES],
    pub epochs: usize,
    pub learning_rate: i64,
    pub weight_limit: i64,
}

impl JointBranchSelector {
    pub fn zero() -> Self {
        Self {
            weights: [0; JOINT_BRANCH_FEATURES],
            epochs: JOINT_BRANCH_EPOCHS,
            learning_rate: JOINT_BRANCH_LEARNING_RATE,
            weight_limit: JOINT_BRANCH_WEIGHT_LIMIT,
        }
    }

    pub fn score(&self, features: &[i32; JOINT_BRANCH_FEATURES]) -> i64 {
        self.weights
            .iter()
            .zip(features)
            .map(|(&weight, &feature)| weight * i64::from(feature))
            .sum()
    }

    pub fn weights_sha256(&self) -> String {
        let mut bytes = Vec::with_capacity(self.weights.len() * 8);
        for weight in self.weights {
            bytes.extend_from_slice(&weight.to_le_bytes());
        }
        uor_r4_discrete::corpus::sha256(&bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct JointBranchCandidate {
    pub length: usize,
    pub boundary: ReplyBoundaryHit,
    pub features: [i32; JOINT_BRANCH_FEATURES],
    pub endpoint_global_advantage: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointBranchSpine {
    pub tokens: Vec<u32>,
    pub stop: String,
    pub candidates: Vec<JointBranchCandidate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointBranchTraining {
    pub trained: JointBranchSelector,
    pub shuffled: JointBranchSelector,
    pub rows: u64,
    pub supervised_groups: u64,
    pub baseline_wrong_groups: u64,
    pub candidate_rows: u64,
    pub positive_rows: u64,
    pub shuffled_label_changes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct JointBranchStep {
    pub row: usize,
    pub arm: JointBranchArm,
    pub candidate_count: usize,
    pub selected_index: Option<usize>,
    pub selected_length: usize,
    pub selected_score: Option<i64>,
    pub first_length: Option<usize>,
    pub longest_length: Option<usize>,
    pub exact_candidate_present: bool,
    pub selected_boundary_depth: Option<usize>,
    pub selected_boundary_terminal: u64,
    pub premature_stop: bool,
    pub reply_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct JointBranchReading {
    pub arm: JointBranchArm,
    pub replies: u64,
    pub reference_reachable: u64,
    pub exact_reference: u64,
    pub bos_complete: u64,
    pub opening_correct: u64,
    pub premature_bos: u64,
    pub generated_tokens: u64,
    pub reference_tokens: u64,
    pub correct_body_positions: u64,
    pub rows_with_candidates: u64,
    pub exact_candidate_rows: u64,
    pub selected_candidates: u64,
    pub changed_from_first: u64,
    pub changed_from_longest: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TimedJointBranchReading {
    pub reading: JointBranchReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<JointBranchStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct JointBranchEvaluation<'a> {
    pub samples: &'a [MemoryWeightSample],
    pub suffix_memory: &'a SuffixMemoryTable,
    pub boundary: &'a ReplyBoundaryMemory,
    pub training: &'a JointBranchTraining,
    pub cap: usize,
    pub arm: JointBranchArm,
    pub seed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TrainingGroup {
    row: usize,
    candidates: Vec<JointBranchCandidate>,
    positives: Vec<bool>,
    baseline_wrong: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecisionKind {
    Local,
    Global(SuffixMemoryPrediction),
    Writer,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct BranchEvidence {
    local_decisions: u64,
    global_decisions: u64,
    writer_fallbacks: u64,
    global_depth_sum: u64,
    global_observations_sum: u64,
    global_successor_margin_sum: u64,
    stronger_continuations: u64,
    tail: Vec<DecisionKind>,
}

pub fn generate_joint_branch_spine(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    suffix_memory: &SuffixMemoryTable,
    boundary: &ReplyBoundaryMemory,
    seed: u64,
    row: usize,
) -> Result<JointBranchSpine> {
    if cap == 0 || cap > 64 || prompt.is_empty() {
        return Err(invalid("joint-branch spine inputs differ"));
    }
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("joint-branch forward: {error}")))?;
    let context = forward.context_length();
    if prompt.len() > context {
        return Err(invalid("joint-branch prompt exceeds context"));
    }

    let mut stream = Vec::with_capacity(context);
    let mut next = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("joint-branch prompt step: {error}")))?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some(step.token);
        }
    }
    let mut writer_token = next.ok_or_else(|| invalid("joint-branch prompt is empty"))?;
    let mut tokens = Vec::with_capacity(cap);
    let mut candidates = Vec::new();
    let mut evidence = BranchEvidence::default();

    loop {
        let local = copy_key_reply_decision(
            &stream,
            writer_token,
            CopyKeyReplyArm::Longest32To1,
            seed,
            row,
            tokens.len(),
        );
        let global = suffix_memory.predict_confident(
            &stream,
            SUFFIX_MEMORY_MAX_DEPTH,
            JOINT_BRANCH_MIN_OBSERVATIONS,
        );
        let local_depth = local.depth;
        let deeper_global =
            global.filter(|prediction| local_depth.is_none_or(|depth| prediction.depth > depth));
        let endpoint_advantage = match (deeper_global, local_depth) {
            (Some(prediction), Some(depth)) => prediction.depth.saturating_sub(depth),
            (Some(prediction), None) => prediction.depth,
            (None, _) => 0,
        };

        if let Some(hit) = boundary.stop_decision(&tokens) {
            candidates.push(JointBranchCandidate {
                length: tokens.len(),
                boundary: hit,
                features: branch_features(&tokens, hit, &evidence, endpoint_advantage)?,
                endpoint_global_advantage: i32::try_from(endpoint_advantage).unwrap_or(i32::MAX),
            });
        }

        let (selected_token, kind) = if let Some(prediction) = deeper_global {
            (prediction.token, DecisionKind::Global(prediction))
        } else if local_depth.is_some() {
            (local.token, DecisionKind::Local)
        } else {
            (writer_token, DecisionKind::Writer)
        };
        evidence.record(kind)?;
        let stop = reply_stop(selected_token, tokens.len(), prompt.len(), cap, context);
        if stop != Some(ReplyEnd::Bos) {
            tokens.push(selected_token);
        }
        if let Some(stop) = stop {
            return Ok(JointBranchSpine {
                tokens,
                stop: match stop {
                    ReplyEnd::Bos => "bos",
                    ReplyEnd::Length => "cap",
                    ReplyEnd::Context => "context",
                }
                .to_owned(),
                candidates,
            });
        }
        stream.push(selected_token);
        let step = forward
            .step(selected_token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("joint-branch generated step: {error}")))?;
        writer_token = step.token;
    }
}

fn branch_features(
    tokens: &[u32],
    boundary: ReplyBoundaryHit,
    evidence: &BranchEvidence,
    endpoint_global_advantage: usize,
) -> Result<[i32; JOINT_BRANCH_FEATURES]> {
    let tail_local = evidence
        .tail
        .iter()
        .filter(|kind| matches!(kind, DecisionKind::Local))
        .count();
    let tail_global = evidence
        .tail
        .iter()
        .filter(|kind| matches!(kind, DecisionKind::Global(_)))
        .count();
    Ok([
        1,
        bounded_i32(tokens.len() as u64, "joint-branch length")?,
        bounded_i32(boundary.depth as u64, "joint-branch boundary depth")?,
        bounded_i32(boundary.terminal, "joint-branch boundary terminal")?,
        bounded_i32(evidence.local_decisions, "joint-branch local decisions")?,
        bounded_i32(evidence.global_decisions, "joint-branch global decisions")?,
        bounded_i32(evidence.global_depth_sum, "joint-branch depth sum")?,
        bounded_i32(
            evidence.global_observations_sum,
            "joint-branch observation sum",
        )?,
        bounded_i32(
            evidence.global_successor_margin_sum,
            "joint-branch successor margin sum",
        )?,
        bounded_i32(
            evidence.stronger_continuations,
            "joint-branch stronger continuations",
        )?,
        bounded_i32(evidence.writer_fallbacks, "joint-branch writer fallbacks")?,
        bounded_i32(tail_local as u64, "joint-branch tail local")?,
        bounded_i32(tail_global as u64, "joint-branch tail global")?,
        bounded_i32(repeated_four_grams(tokens), "joint-branch repeats")?,
        bounded_i32(
            endpoint_global_advantage as u64,
            "joint-branch endpoint advantage",
        )?,
    ])
}

impl BranchEvidence {
    fn record(&mut self, kind: DecisionKind) -> Result<()> {
        match kind {
            DecisionKind::Local => {
                self.local_decisions = checked_add(self.local_decisions, 1, "local decisions")?
            }
            DecisionKind::Global(prediction) => {
                self.global_decisions = checked_add(self.global_decisions, 1, "global decisions")?;
                self.stronger_continuations =
                    checked_add(self.stronger_continuations, 1, "stronger continuations")?;
                self.global_depth_sum = checked_add(
                    self.global_depth_sum,
                    prediction.depth as u64,
                    "global depth sum",
                )?;
                self.global_observations_sum = checked_add(
                    self.global_observations_sum,
                    prediction.observations,
                    "global observations",
                )?;
                self.global_successor_margin_sum = checked_add(
                    self.global_successor_margin_sum,
                    prediction
                        .successor_count
                        .saturating_mul(2)
                        .saturating_sub(prediction.observations),
                    "global successor margin",
                )?;
            }
            DecisionKind::Writer => {
                self.writer_fallbacks = checked_add(self.writer_fallbacks, 1, "writer fallbacks")?
            }
        }
        self.tail.push(kind);
        if self.tail.len() > JOINT_BRANCH_TAIL {
            self.tail.remove(0);
        }
        Ok(())
    }
}

pub fn fit_joint_branch_selector(
    samples: &[CpuConversationSample],
    cap: usize,
    seed: u64,
) -> Result<JointBranchTraining> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("joint-branch training inputs differ"));
    }
    let groups = (0..JOINT_BRANCH_FOLDS)
        .into_par_iter()
        .map(|fold| training_groups_for_fold(samples, cap, seed, fold))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    finish_training(groups, samples.len() as u64)
}

pub fn fit_joint_branch_selector_on_panels(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    suffix_memory: &SuffixMemoryTable,
    boundary: &ReplyBoundaryMemory,
    cap: usize,
    seed: u64,
) -> Result<JointBranchTraining> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("joint-branch calibration inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("joint-branch calibration context: {error}")))?
        .context_length();
    let groups = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply);
            if reference.is_empty()
                || reference.len() >= cap
                || sample.prompt.len() > context
                || sample
                    .prompt
                    .len()
                    .checked_add(reference.len())
                    .is_none_or(|total| total >= context)
            {
                return Ok(None);
            }
            let spine = generate_joint_branch_spine(
                artifact,
                &sample.prompt,
                cap,
                suffix_memory,
                boundary,
                seed,
                row,
            )?;
            if spine.candidates.len() < 2 {
                return Ok(None);
            }
            let positives = spine
                .candidates
                .iter()
                .map(|candidate| {
                    candidate.length == reference.len()
                        && spine.tokens.get(..candidate.length) == Some(reference)
                })
                .collect::<Vec<_>>();
            if !positives.iter().any(|positive| *positive) {
                return Ok(None);
            }
            let first_exact = spine.candidates.first().is_some_and(|candidate| {
                candidate.length == reference.len()
                    && spine.tokens.get(..candidate.length) == Some(reference)
            });
            Ok(Some(TrainingGroup {
                row,
                candidates: spine.candidates,
                positives,
                baseline_wrong: !first_exact,
            }))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    finish_training(groups, samples.len() as u64)
}

fn finish_training(groups: Vec<TrainingGroup>, rows: u64) -> Result<JointBranchTraining> {
    if groups.is_empty() {
        return Ok(JointBranchTraining {
            trained: JointBranchSelector::zero(),
            shuffled: JointBranchSelector::zero(),
            rows,
            supervised_groups: 0,
            baseline_wrong_groups: 0,
            candidate_rows: 0,
            positive_rows: 0,
            shuffled_label_changes: 0,
        });
    }
    let (trained, _) = fit_weights(&groups, false);
    let (shuffled, shuffled_label_changes) = fit_weights(&groups, true);
    if trained == shuffled && shuffled_label_changes > 0 {
        return Err(invalid(
            "joint-branch shuffled labels produced the same selector",
        ));
    }
    Ok(JointBranchTraining {
        trained,
        shuffled,
        rows,
        supervised_groups: groups.len() as u64,
        baseline_wrong_groups: groups.iter().filter(|group| group.baseline_wrong).count() as u64,
        candidate_rows: groups
            .iter()
            .map(|group| group.candidates.len() as u64)
            .sum(),
        positive_rows: groups
            .iter()
            .map(|group| group.positives.iter().filter(|positive| **positive).count() as u64)
            .sum(),
        shuffled_label_changes,
    })
}

fn training_groups_for_fold(
    samples: &[CpuConversationSample],
    cap: usize,
    seed: u64,
    fold: usize,
) -> Result<Vec<TrainingGroup>> {
    let training = samples
        .iter()
        .filter(|sample| sample.document_ordinal % JOINT_BRANCH_FOLDS != fold)
        .map(|sample| MemoryWeightSample {
            prompt: sample.prompt.clone(),
            reply: sample.reply.clone(),
        })
        .collect::<Vec<_>>();
    let validation = samples
        .iter()
        .filter(|sample| sample.document_ordinal % JOINT_BRANCH_FOLDS == fold)
        .collect::<Vec<_>>();
    if training.is_empty() || validation.is_empty() {
        return Ok(Vec::new());
    }
    let suffix_memory = derive_full_sequence_suffix_content_memory(&training)?;
    let boundary = crate::reply_boundary::derive_reply_boundary_memory(&training)?;
    validation
        .into_par_iter()
        .map(|sample| {
            let row = sample.document_ordinal;
            let reference = normalized_reference(&sample.reply);
            if reference.is_empty() || reference.len() >= cap {
                return Ok(None);
            }
            let candidates = training_candidates_from_reference(
                &sample.prompt,
                reference,
                cap,
                &suffix_memory,
                &boundary,
                seed,
                row,
            )?;
            if candidates.len() < 2 {
                return Ok(None);
            }
            let positives = candidates
                .iter()
                .map(|candidate| candidate.length == reference.len())
                .collect::<Vec<_>>();
            if !positives.iter().any(|positive| *positive) {
                return Ok(None);
            }
            let first_exact = candidates
                .first()
                .is_some_and(|candidate| candidate.length == reference.len());
            Ok(Some(TrainingGroup {
                row,
                candidates,
                positives,
                baseline_wrong: !first_exact,
            }))
        })
        .collect::<Result<Vec<_>>>()
        .map(|groups| groups.into_iter().flatten().collect())
}

fn training_candidates_from_reference(
    prompt: &[u32],
    reference: &[u32],
    cap: usize,
    suffix_memory: &SuffixMemoryTable,
    boundary: &ReplyBoundaryMemory,
    seed: u64,
    row: usize,
) -> Result<Vec<JointBranchCandidate>> {
    if reference.is_empty() || reference.len() >= cap {
        return Err(invalid("joint-branch training reference differs"));
    }
    let mut stream = prompt.to_vec();
    let mut evidence = BranchEvidence::default();
    let mut candidates = Vec::new();
    for length in 0..=reference.len() {
        let local =
            copy_key_reply_decision(&stream, 0, CopyKeyReplyArm::Longest32To1, seed, row, length);
        let global = suffix_memory.predict_confident(
            &stream,
            SUFFIX_MEMORY_MAX_DEPTH,
            JOINT_BRANCH_MIN_OBSERVATIONS,
        );
        let local_depth = local.depth;
        let deeper_global =
            global.filter(|prediction| local_depth.is_none_or(|depth| prediction.depth > depth));
        let endpoint_advantage = match (deeper_global, local_depth) {
            (Some(prediction), Some(depth)) => prediction.depth.saturating_sub(depth),
            (Some(prediction), None) => prediction.depth,
            (None, _) => 0,
        };
        if let Some(hit) = boundary.stop_decision(&reference[..length]) {
            candidates.push(JointBranchCandidate {
                length,
                boundary: hit,
                features: branch_features(
                    &reference[..length],
                    hit,
                    &evidence,
                    endpoint_advantage,
                )?,
                endpoint_global_advantage: i32::try_from(endpoint_advantage).unwrap_or(i32::MAX),
            });
        }
        if length == reference.len() {
            break;
        }
        let kind = if let Some(prediction) = deeper_global {
            DecisionKind::Global(prediction)
        } else if local_depth.is_some() {
            DecisionKind::Local
        } else {
            DecisionKind::Writer
        };
        evidence.record(kind)?;
        stream.push(reference[length]);
    }
    Ok(candidates)
}

fn fit_weights(groups: &[TrainingGroup], shuffle: bool) -> (JointBranchSelector, u64) {
    let mut selector = JointBranchSelector::zero();
    let mut label_changes = 0u64;
    let labels = groups
        .iter()
        .map(|group| {
            if !shuffle {
                return group.positives.clone();
            }
            let positive_count = group.positives.iter().filter(|positive| **positive).count();
            let selected =
                shuffled_positive_indices(group.row, group.candidates.len(), positive_count);
            let labels = (0..group.candidates.len())
                .map(|index| selected.contains(&index))
                .collect::<Vec<_>>();
            label_changes += group
                .positives
                .iter()
                .zip(&labels)
                .filter(|(left, right)| left != right)
                .count() as u64;
            labels
        })
        .collect::<Vec<_>>();
    for _ in 0..selector.epochs {
        for (group, positives) in groups.iter().zip(&labels) {
            let Some((_, negative)) = group
                .candidates
                .iter()
                .enumerate()
                .filter(|(index, _)| !positives[*index])
                .max_by_key(|(_, candidate)| {
                    (
                        selector.score(&candidate.features),
                        Reverse(candidate.length),
                    )
                })
            else {
                continue;
            };
            let negative_score = selector.score(&negative.features);
            for (index, candidate) in group.candidates.iter().enumerate() {
                if positives[index] && selector.score(&candidate.features) <= negative_score {
                    for feature in 0..JOINT_BRANCH_FEATURES {
                        selector.weights[feature] = (selector.weights[feature]
                            + selector.learning_rate
                                * i64::from(
                                    candidate.features[feature] - negative.features[feature],
                                ))
                        .clamp(-selector.weight_limit, selector.weight_limit);
                    }
                }
            }
        }
    }
    (selector, label_changes)
}

fn shuffled_positive_indices(row: usize, count: usize, positives: usize) -> BTreeSet<usize> {
    let mut indices = (0..count).collect::<Vec<_>>();
    let mut state = JOINT_BRANCH_SHUFFLE_SEED ^ (row as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    for index in (1..indices.len()).rev() {
        state = splitmix64(state);
        let draw = state as usize % (index + 1);
        indices.swap(index, draw);
    }
    indices.into_iter().take(positives).collect()
}

pub fn evaluate_joint_branch_selection(
    artifact: &[u8],
    config: JointBranchEvaluation<'_>,
) -> Result<TimedJointBranchReading> {
    if config.samples.is_empty() || config.cap == 0 || config.cap > 64 {
        return Err(invalid("joint-branch evaluation inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("joint-branch evaluation context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = config
        .samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply);
            if reference.is_empty()
                || reference.len() >= config.cap
                || sample.prompt.len() > context
                || sample
                    .prompt
                    .len()
                    .checked_add(reference.len())
                    .is_none_or(|total| total >= context)
            {
                return Ok((
                    CopyKeyReplyRow {
                        row,
                        reachable: false,
                        exact_reference: false,
                        bos_complete: false,
                        opening_correct: false,
                        premature_bos: false,
                        non_premature_bos: false,
                        generated_tokens: 0,
                        reference_tokens: reference.len() as u64,
                        correct_body_positions: 0,
                        copy_decisions: 0,
                        fallback_decisions: 0,
                        stop: "unreachable".to_owned(),
                        reply_sha256: token_hash(&[]),
                    },
                    JointBranchStep {
                        row,
                        arm: config.arm,
                        candidate_count: 0,
                        selected_index: None,
                        selected_length: 0,
                        selected_score: None,
                        first_length: None,
                        longest_length: None,
                        exact_candidate_present: false,
                        selected_boundary_depth: None,
                        selected_boundary_terminal: 0,
                        premature_stop: false,
                        reply_sha256: token_hash(&[]),
                    },
                ));
            }
            let spine = generate_joint_branch_spine(
                artifact,
                &sample.prompt,
                config.cap,
                config.suffix_memory,
                config.boundary,
                config.seed,
                row,
            )?;
            let selected_index = select_candidate(&spine, config.arm, config.training, reference);
            let tokens = selected_index
                .and_then(|index| spine.tokens.get(..spine.candidates[index].length))
                .unwrap_or(spine.tokens.as_slice());
            let stop = if selected_index.is_some() {
                "bos".to_owned()
            } else {
                spine.stop.clone()
            };
            let selected_len = tokens.len();
            let exact_candidate_present = spine.candidates.iter().any(|candidate| {
                candidate.length == reference.len()
                    && spine.tokens.get(..candidate.length) == Some(reference)
            });
            let selected_boundary = selected_index.map(|index| spine.candidates[index].boundary);
            let premature_stop = selected_len < reference.len();
            let row_reading = CopyKeyReplyRow {
                row,
                reachable: true,
                exact_reference: tokens == reference,
                bos_complete: stop == "bos",
                opening_correct: matches!(
                    (tokens.first(), reference.first()),
                    (Some(generated), Some(reference)) if generated == reference
                ),
                premature_bos: stop == "bos" && premature_stop,
                non_premature_bos: stop == "bos" && !premature_stop,
                generated_tokens: selected_len as u64,
                reference_tokens: reference.len() as u64,
                correct_body_positions: tokens
                    .iter()
                    .zip(reference)
                    .filter(|(generated, reference)| generated == reference)
                    .count() as u64,
                copy_decisions: 0,
                fallback_decisions: 0,
                stop: stop.clone(),
                reply_sha256: token_hash(tokens),
            };
            let first_length = spine.candidates.first().map(|candidate| candidate.length);
            let longest_length = spine.candidates.last().map(|candidate| candidate.length);
            let step = JointBranchStep {
                row,
                arm: config.arm,
                candidate_count: spine.candidates.len(),
                selected_index,
                selected_length: selected_len,
                selected_score: selected_index.map(|index| {
                    let selector = match config.arm {
                        JointBranchArm::Learned => &config.training.trained,
                        JointBranchArm::ShuffledLabels => &config.training.shuffled,
                        _ => return i64::MIN,
                    };
                    selector.score(&spine.candidates[index].features)
                }),
                first_length,
                longest_length,
                exact_candidate_present,
                selected_boundary_depth: selected_boundary.map(|hit| hit.depth),
                selected_boundary_terminal: selected_boundary.map_or(0, |hit| hit.terminal),
                premature_stop,
                reply_sha256: token_hash(tokens),
            };
            Ok((row_reading, step))
        })
        .collect::<Result<Vec<_>>>()?;
    let wall_elapsed_us = started.elapsed().as_micros().try_into().unwrap_or(u64::MAX);
    let (rows, steps): (Vec<_>, Vec<_>) = evaluated.into_iter().unzip();
    let reading = summarize(config.arm, &rows, &steps)?;
    Ok(TimedJointBranchReading {
        reply_sha256: reply_hash(&rows),
        decision_sha256: decision_hash(&steps),
        generated_tokens_per_second: if wall_elapsed_us == 0 {
            0.0
        } else {
            reading.generated_tokens as f64 * 1_000_000.0 / wall_elapsed_us as f64
        },
        reading,
        rows,
        steps,
        wall_elapsed_us,
    })
}

fn select_candidate(
    spine: &JointBranchSpine,
    arm: JointBranchArm,
    training: &JointBranchTraining,
    reference: &[u32],
) -> Option<usize> {
    if spine.candidates.is_empty() {
        return None;
    }
    match arm {
        JointBranchArm::Learned => select_by_score(spine, &training.trained),
        JointBranchArm::ShuffledLabels => select_by_score(spine, &training.shuffled),
        JointBranchArm::FirstBoundary => Some(0),
        JointBranchArm::LongestSpine => Some(spine.candidates.len() - 1),
        JointBranchArm::BoundaryTerminalVolume => spine
            .candidates
            .iter()
            .enumerate()
            .max_by_key(|(_, candidate)| {
                (
                    candidate.boundary.terminal.saturating_mul(
                        u64::try_from(candidate.boundary.depth).unwrap_or(u64::MAX),
                    ),
                    candidate.boundary.terminal,
                    candidate.boundary.depth,
                    Reverse(candidate.length),
                )
            })
            .map(|(index, _)| index),
        JointBranchArm::ReferenceEndpointOracle => spine
            .candidates
            .iter()
            .position(|candidate| {
                candidate.length == reference.len()
                    && spine.tokens.get(..candidate.length) == Some(reference)
            })
            .or(Some(0)),
    }
}

fn select_by_score(spine: &JointBranchSpine, selector: &JointBranchSelector) -> Option<usize> {
    spine
        .candidates
        .iter()
        .enumerate()
        .max_by_key(|(_, candidate)| {
            (
                selector.score(&candidate.features),
                Reverse(candidate.length),
            )
        })
        .map(|(index, _)| index)
}

fn summarize(
    arm: JointBranchArm,
    rows: &[CopyKeyReplyRow],
    steps: &[JointBranchStep],
) -> Result<JointBranchReading> {
    let mut reading = JointBranchReading {
        arm,
        replies: rows.len() as u64,
        reference_reachable: 0,
        exact_reference: 0,
        bos_complete: 0,
        opening_correct: 0,
        premature_bos: 0,
        generated_tokens: 0,
        reference_tokens: 0,
        correct_body_positions: 0,
        rows_with_candidates: 0,
        exact_candidate_rows: 0,
        selected_candidates: 0,
        changed_from_first: 0,
        changed_from_longest: 0,
    };
    for row in rows {
        reading.reference_reachable += u64::from(row.reachable);
        reading.exact_reference += u64::from(row.reachable && row.exact_reference);
        reading.bos_complete += u64::from(row.reachable && row.bos_complete);
        reading.opening_correct += u64::from(row.reachable && row.opening_correct);
        reading.premature_bos += u64::from(row.reachable && row.premature_bos);
        reading.generated_tokens = checked_add(
            reading.generated_tokens,
            row.generated_tokens,
            "generated tokens",
        )?;
        reading.reference_tokens = checked_add(
            reading.reference_tokens,
            row.reference_tokens,
            "reference tokens",
        )?;
        reading.correct_body_positions = checked_add(
            reading.correct_body_positions,
            row.correct_body_positions,
            "correct body positions",
        )?;
    }
    for step in steps {
        reading.rows_with_candidates += u64::from(step.candidate_count > 0);
        reading.exact_candidate_rows += u64::from(step.exact_candidate_present);
        reading.selected_candidates += u64::from(step.selected_index.is_some());
        reading.changed_from_first +=
            u64::from(step.selected_length != step.first_length.unwrap_or(step.selected_length));
        reading.changed_from_longest +=
            u64::from(step.selected_length != step.longest_length.unwrap_or(step.selected_length));
    }
    Ok(reading)
}

pub fn compare_joint_branch_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<CopyKeyReplyComparison> {
    compare_copy_key_reply_rows(candidate, control, metric)
}

fn bounded_i32(value: u64, name: &str) -> Result<i32> {
    i32::try_from(value).map_err(|_| invalid(format!("{name} exceeds i32")))
}

fn checked_add(left: u64, right: u64, name: &str) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid(format!("{name} overflows")))
}

fn repeated_four_grams(tokens: &[u32]) -> u64 {
    if tokens.len() < 5 {
        return 0;
    }
    let mut seen = BTreeSet::new();
    let mut repeats = 0u64;
    for gram in tokens.windows(4) {
        if !seen.insert(gram.to_vec()) {
            repeats += 1;
        }
    }
    repeats
}

fn token_hash(tokens: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(tokens.len() * 4);
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn reply_hash(rows: &[CopyKeyReplyRow]) -> String {
    let mut hasher = blake3::Hasher::new();
    for row in rows {
        hasher.update(&(row.row as u64).to_le_bytes());
        hasher.update(row.reply_sha256.as_bytes());
        hasher.update(&[
            u8::from(row.reachable),
            u8::from(row.exact_reference),
            u8::from(row.bos_complete),
            u8::from(row.opening_correct),
            u8::from(row.premature_bos),
            u8::from(row.non_premature_bos),
        ]);
        hasher.update(&row.generated_tokens.to_le_bytes());
        hasher.update(&row.reference_tokens.to_le_bytes());
        hasher.update(&row.correct_body_positions.to_le_bytes());
        hasher.update(row.stop.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn decision_hash(steps: &[JointBranchStep]) -> String {
    let mut hasher = blake3::Hasher::new();
    for step in steps {
        hasher.update(&(step.row as u64).to_le_bytes());
        hasher.update(&(step.selected_length as u64).to_le_bytes());
        hasher.update(&(step.candidate_count as u64).to_le_bytes());
        hasher.update(&step.selected_index.unwrap_or(usize::MAX).to_le_bytes());
        hasher.update(step.reply_sha256.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reply_boundary::derive_reply_boundary_memory;
    use crate::suffix_memory::derive_full_sequence_suffix_content_memory;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    fn sample(prompt: &[u32], reply: &[u32]) -> MemoryWeightSample {
        MemoryWeightSample {
            prompt: prompt.to_vec(),
            reply: reply.to_vec(),
        }
    }

    fn artifact() -> Vec<u8> {
        export_fbs_trunk_uorlut01(&synthetic_fbs_trunk_with(
            231,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 3,
                context: 64,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        ))
        .unwrap()
    }

    #[test]
    fn spine_collects_complete_boundary_branches_without_committing_one() {
        let training = [
            sample(&[9], &[40, 10, 20, 30, 1]),
            sample(&[9], &[40, 10, 20, 30, 1]),
            sample(&[9], &[40, 10, 20, 30, 1]),
            sample(&[9], &[41, 11, 21, 31, 1]),
        ];
        let suffix = derive_full_sequence_suffix_content_memory(&training).unwrap();
        let boundary = derive_reply_boundary_memory(&training).unwrap();
        let spine =
            generate_joint_branch_spine(&artifact(), &[9], 16, &suffix, &boundary, 231, 0).unwrap();
        assert!(!spine.candidates.is_empty());
        assert!(spine
            .candidates
            .windows(2)
            .all(|pair| pair[0].length < pair[1].length));
    }

    #[test]
    fn selector_weights_are_deterministic_and_bounded() {
        let group = TrainingGroup {
            row: 0,
            candidates: vec![
                JointBranchCandidate {
                    length: 4,
                    boundary: ReplyBoundaryHit {
                        depth: 4,
                        terminal: 1,
                        nonterminal: 0,
                    },
                    features: [1, 4, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    endpoint_global_advantage: 0,
                },
                JointBranchCandidate {
                    length: 6,
                    boundary: ReplyBoundaryHit {
                        depth: 6,
                        terminal: 2,
                        nonterminal: 0,
                    },
                    features: [1, 6, 6, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    endpoint_global_advantage: 0,
                },
            ],
            positives: vec![false, true],
            baseline_wrong: true,
        };
        let first = fit_weights(&[group.clone()], false).0;
        let second = fit_weights(&[group], false).0;
        assert_eq!(first, second);
        assert_ne!(first, JointBranchSelector::zero());
        assert!(first
            .weights
            .iter()
            .all(|weight| weight.abs() <= first.weight_limit));
    }

    #[test]
    fn shuffled_labels_preserve_positive_count_and_change_the_selector() {
        let group = TrainingGroup {
            row: 7,
            candidates: (1..=5)
                .map(|length| JointBranchCandidate {
                    length,
                    boundary: ReplyBoundaryHit {
                        depth: length + 3,
                        terminal: length as u64,
                        nonterminal: 0,
                    },
                    features: [
                        1,
                        length as i32,
                        (length + 3) as i32,
                        length as i32,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                    ],
                    endpoint_global_advantage: 0,
                })
                .collect(),
            positives: vec![false, true, false, false, false],
            baseline_wrong: true,
        };
        let (trained, changes) = fit_weights(&[group.clone()], false);
        let (shuffled, label_changes) = fit_weights(&[group], true);
        assert_eq!(changes, 0);
        assert!(label_changes > 0);
        assert_ne!(trained, shuffled);
    }

    #[test]
    fn reference_oracle_selects_the_exact_candidate() {
        let spine = JointBranchSpine {
            tokens: vec![40, 10, 20, 30, 99],
            stop: "cap".to_owned(),
            candidates: [4, 5]
                .into_iter()
                .map(|length| JointBranchCandidate {
                    length,
                    boundary: ReplyBoundaryHit {
                        depth: 4,
                        terminal: 1,
                        nonterminal: 0,
                    },
                    features: [0; JOINT_BRANCH_FEATURES],
                    endpoint_global_advantage: 0,
                })
                .collect(),
        };
        let training = JointBranchTraining {
            trained: JointBranchSelector::zero(),
            shuffled: JointBranchSelector::zero(),
            rows: 0,
            supervised_groups: 0,
            baseline_wrong_groups: 0,
            candidate_rows: 0,
            positive_rows: 0,
            shuffled_label_changes: 0,
        };
        assert_eq!(
            select_candidate(
                &spine,
                JointBranchArm::ReferenceEndpointOracle,
                &training,
                &[40, 10, 20, 30],
            ),
            Some(0)
        );
    }

    #[test]
    fn reference_preflight_records_the_complete_terminal_branch() {
        let training = [
            sample(&[9], &[40, 10, 20, 30, 1]),
            sample(&[9], &[40, 10, 20, 30, 1]),
            sample(&[9], &[41, 11, 21, 31, 1]),
        ];
        let suffix = derive_full_sequence_suffix_content_memory(&training).unwrap();
        let boundary = derive_reply_boundary_memory(&training).unwrap();
        let candidates = training_candidates_from_reference(
            &[9],
            &[40, 10, 20, 30],
            16,
            &suffix,
            &boundary,
            231,
            0,
        )
        .unwrap();
        assert!(candidates.iter().any(|candidate| candidate.length == 4));
        assert!(candidates.iter().all(|candidate| candidate.length <= 4));
    }

    #[test]
    fn panel_fit_uses_generated_complete_branches_and_is_deterministic() {
        let training = [
            sample(&[9], &[40, 10, 20, 30, 1]),
            sample(&[9], &[40, 10, 20, 30, 1]),
            sample(&[9], &[41, 11, 21, 31, 1]),
        ];
        let suffix = derive_full_sequence_suffix_content_memory(&training).unwrap();
        let boundary = derive_reply_boundary_memory(&training).unwrap();
        let calibration = [sample(&[9], &[40, 10, 20, 30, 1])];
        let first = fit_joint_branch_selector_on_panels(
            &artifact(),
            &calibration,
            &suffix,
            &boundary,
            16,
            233,
        )
        .unwrap();
        let second = fit_joint_branch_selector_on_panels(
            &artifact(),
            &calibration,
            &suffix,
            &boundary,
            16,
            233,
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.rows, 1);
        assert!(first.supervised_groups <= 1);
    }

    #[test]
    fn invalid_prompt_and_cap_are_refused() {
        let training = [
            sample(&[9], &[40, 10, 20, 30, 1]),
            sample(&[9], &[41, 11, 21, 31, 1]),
        ];
        let suffix = derive_full_sequence_suffix_content_memory(&training).unwrap();
        let boundary = derive_reply_boundary_memory(&training).unwrap();
        assert!(
            generate_joint_branch_spine(&artifact(), &[], 16, &suffix, &boundary, 231, 0,).is_err()
        );
        assert!(
            generate_joint_branch_spine(&artifact(), &[1], 0, &suffix, &boundary, 231, 0,).is_err()
        );
    }
}
