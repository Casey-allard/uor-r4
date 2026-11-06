//! Bounded integer ranking over complete TRAIN reply neighbors.
//!
//! The fit sees only leave-one-out TRAIN prompt/reply rows. Inference ranks at
//! most 16 exact prompt-set neighbors using eight bounded integer features.
//! No dense matrix operation, float scoring or generated token is used.

use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::copy_key_reply::{CopyKeyReplyMetric, CopyKeyReplyRow};
use crate::reply_eval::normalized_reference;
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const PROMPT_REPLY_RANK_TOP_K: usize = 16;
pub const PROMPT_REPLY_RANK_FEATURES: usize = 8;
pub const PROMPT_REPLY_RANK_EPOCHS: usize = 8;
pub const PROMPT_REPLY_RANK_LEARNING_RATE: i64 = 1;
pub const PROMPT_REPLY_RANK_WEIGHT_LIMIT: i64 = 65_536;
pub const PROMPT_REPLY_RANK_MIN_GROUPS: usize = 96;
pub const PROMPT_REPLY_RANK_MIN_BASELINE_WRONG: usize = 24;
pub const PROMPT_REPLY_RANK_SHUFFLE_SEED: u64 = 213;

pub const PROMPT_REPLY_RANK_FEATURE_NAMES: [&str; PROMPT_REPLY_RANK_FEATURES] = [
    "bias",
    "unigram_jaccard_q16",
    "query_coverage_q16",
    "candidate_coverage_q16",
    "reply_prompt_overlap_q16",
    "writer_distance_advantage",
    "reply_frequency",
    "reply_length_delta",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptReplyRanker {
    pub weights: [i64; PROMPT_REPLY_RANK_FEATURES],
    pub epochs: usize,
    pub learning_rate: i64,
    pub weight_limit: i64,
}

impl PromptReplyRanker {
    pub fn zero() -> Self {
        Self {
            weights: [0; PROMPT_REPLY_RANK_FEATURES],
            epochs: PROMPT_REPLY_RANK_EPOCHS,
            learning_rate: PROMPT_REPLY_RANK_LEARNING_RATE,
            weight_limit: PROMPT_REPLY_RANK_WEIGHT_LIMIT,
        }
    }

    pub fn score(&self, features: &[i32; PROMPT_REPLY_RANK_FEATURES]) -> i64 {
        self.weights
            .iter()
            .zip(features)
            .map(|(&weight, &feature)| weight * i64::from(feature))
            .sum()
    }

    pub fn weights_sha256(&self) -> String {
        let mut bytes = Vec::new();
        for weight in self.weights {
            bytes.extend_from_slice(&weight.to_le_bytes());
        }
        uor_r4_discrete::corpus::sha256(&bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptReplyRankArm {
    LearnedRanker,
    RawUnigramJaccard,
    ShuffledRanker,
    QueryGeometry,
    FixedOrdinal,
    ReferenceMembershipOracle,
}

impl PromptReplyRankArm {
    pub const EVALUATION_ARMS: [Self; 6] = [
        Self::LearnedRanker,
        Self::RawUnigramJaccard,
        Self::ShuffledRanker,
        Self::QueryGeometry,
        Self::FixedOrdinal,
        Self::ReferenceMembershipOracle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::LearnedRanker => "learned_ranker",
            Self::RawUnigramJaccard => "raw_unigram_jaccard",
            Self::ShuffledRanker => "shuffled_ranker",
            Self::QueryGeometry => "query_geometry",
            Self::FixedOrdinal => "fixed_ordinal",
            Self::ReferenceMembershipOracle => "reference_membership_oracle",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RankEntry {
    prompt_len: usize,
    prompt_unigram: BTreeSet<u32>,
    reply: Vec<u32>,
    reply_unigram: BTreeSet<u32>,
    query: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptReplyRankMemory {
    artifact: Vec<u8>,
    entries: Vec<RankEntry>,
    reply_frequency: BTreeMap<Vec<u32>, u32>,
    mean_reply_len: i32,
    context: usize,
}

impl PromptReplyRankMemory {
    pub fn entries(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RankedCandidate {
    ordinal: usize,
    raw_numerator: u64,
    raw_denominator: u64,
    raw_rank: usize,
    features: [i32; PROMPT_REPLY_RANK_FEATURES],
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TrainingGroup {
    row: usize,
    reference: Vec<u32>,
    candidates: Vec<RankedCandidate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptReplyRankTraining {
    pub trained: PromptReplyRanker,
    pub shuffled: PromptReplyRanker,
    pub rows: u64,
    pub supervised_groups: u64,
    pub baseline_wrong_groups: u64,
    pub candidate_rows: u64,
    pub positive_rows: u64,
    pub shuffled_label_changes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptReplyRankStep {
    pub row: usize,
    pub arm: PromptReplyRankArm,
    pub selected_ordinal: usize,
    pub candidate_count: usize,
    pub candidate_correct_present: bool,
    pub ranker_score: Option<i64>,
    pub raw_numerator: u64,
    pub raw_denominator: u64,
    pub reply_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptReplyRankReading {
    pub arm: PromptReplyRankArm,
    pub replies: u64,
    pub reference_reachable: u64,
    pub emitted: u64,
    pub exact_reference: u64,
    pub opening_correct: u64,
    pub correct_body_positions: u64,
    pub generated_tokens: u64,
    pub reference_tokens: u64,
    pub premature_bos: u64,
    pub candidate_correct_present: u64,
    pub distinct_selected_replies: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedPromptReplyRankReading {
    pub reading: PromptReplyRankReading,
    #[serde(skip)]
    pub rows: Vec<CopyKeyReplyRow>,
    #[serde(skip)]
    pub steps: Vec<PromptReplyRankStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub replies_per_second: f64,
}

pub fn derive_prompt_reply_rank_memory(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
) -> Result<PromptReplyRankMemory> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("prompt-reply ranker samples or cap differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("prompt-reply ranker artifact: {error}")))?
        .context_length();
    if context == 0 || context > 256 {
        return Err(invalid("prompt-reply ranker context is outside 1..=256"));
    }
    let entries = samples
        .iter()
        .enumerate()
        .map(|(row, sample)| {
            if sample.prompt.is_empty() || sample.prompt.len() > context {
                return Err(invalid(format!(
                    "prompt-reply ranker row {row} prompt is empty or too long"
                )));
            }
            let reply = normalized_reference(&sample.reply).to_vec();
            if reply.is_empty() || reply.len() >= cap {
                return Err(invalid(format!(
                    "prompt-reply ranker row {row} reply is empty or unreachable"
                )));
            }
            Ok(RankEntry {
                prompt_len: sample.prompt.len(),
                prompt_unigram: sample.prompt.iter().copied().collect(),
                reply_unigram: reply.iter().copied().collect(),
                reply,
                query: writer_query(artifact, &sample.prompt)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut reply_frequency = BTreeMap::new();
    for entry in &entries {
        *reply_frequency.entry(entry.reply.clone()).or_insert(0) += 1;
    }
    let mean_reply_len =
        entries.iter().map(|entry| entry.reply.len()).sum::<usize>() / entries.len();
    Ok(PromptReplyRankMemory {
        artifact: artifact.to_vec(),
        entries,
        reply_frequency,
        mean_reply_len: i32::try_from(mean_reply_len)
            .map_err(|_| invalid("prompt-reply ranker mean length overflow"))?,
        context,
    })
}

pub fn fit_prompt_reply_ranker(memory: &PromptReplyRankMemory) -> Result<PromptReplyRankTraining> {
    if memory.entries.len() < 2 {
        return Err(invalid("prompt-reply ranker needs at least two entries"));
    }
    let groups = (0..memory.entries.len())
        .map(|row| training_group(memory, row))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let baseline_wrong_groups = groups
        .iter()
        .filter(|group| group.candidates[0].ordinal != candidate_positive(memory, group))
        .count();
    if groups.len() < PROMPT_REPLY_RANK_MIN_GROUPS {
        return Err(invalid(format!(
            "prompt-reply ranker has {} supervised groups, fewer than {PROMPT_REPLY_RANK_MIN_GROUPS}",
            groups.len()
        )));
    }
    if baseline_wrong_groups < PROMPT_REPLY_RANK_MIN_BASELINE_WRONG {
        return Err(invalid(format!(
            "prompt-reply ranker has {baseline_wrong_groups} baseline-wrong groups, fewer than {PROMPT_REPLY_RANK_MIN_BASELINE_WRONG}"
        )));
    }
    let (trained, _) = fit_weights(memory, &groups, false);
    let (shuffled, shuffled_label_changes) = fit_weights(memory, &groups, true);
    if shuffled_label_changes == 0 {
        return Err(invalid("prompt-reply ranker target shuffle is vacuous"));
    }
    if trained.weights == shuffled.weights {
        return Err(invalid(
            "prompt-reply ranker true and shuffled weights are identical",
        ));
    }
    let candidate_rows = groups
        .iter()
        .map(|group| group.candidates.len() as u64)
        .sum();
    let positive_rows = groups
        .iter()
        .map(|group| {
            group
                .candidates
                .iter()
                .filter(|candidate| memory.entries[candidate.ordinal].reply == group.reference)
                .count() as u64
        })
        .sum();
    Ok(PromptReplyRankTraining {
        trained,
        shuffled,
        rows: memory.entries.len() as u64,
        supervised_groups: groups.len() as u64,
        baseline_wrong_groups: baseline_wrong_groups as u64,
        candidate_rows,
        positive_rows,
        shuffled_label_changes,
    })
}

pub fn evaluate_prompt_reply_ranking(
    memory: &PromptReplyRankMemory,
    training: &PromptReplyRankTraining,
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: PromptReplyRankArm,
) -> Result<TimedPromptReplyRankReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("prompt-reply ranker evaluation inputs differ"));
    }
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| evaluate_row(memory, training, sample, row, cap, arm))
        .collect::<Result<Vec<_>>>()?;
    let wall_elapsed_us = started.elapsed().as_micros().try_into().unwrap_or(u64::MAX);
    let mut rows = Vec::with_capacity(evaluated.len());
    let mut steps = Vec::with_capacity(evaluated.len());
    let mut replies = Vec::with_capacity(evaluated.len());
    for (row, step, reply) in evaluated {
        rows.push(row);
        steps.push(step);
        replies.push(reply);
    }
    let reading = summarize(arm, &rows, &steps, &replies)?;
    let replies_per_second = if wall_elapsed_us == 0 {
        0.0
    } else {
        rows.len() as f64 * 1_000_000.0 / wall_elapsed_us as f64
    };
    let reply_sha256 = row_hash(&steps);
    let decision_sha256 = decision_hash(&steps);
    Ok(TimedPromptReplyRankReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        replies_per_second,
    })
}

pub fn compare_prompt_reply_rank_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<crate::copy_key_reply::CopyKeyReplyComparison> {
    crate::copy_key_reply::compare_copy_key_reply_rows(candidate, control, metric)
}

fn training_group(memory: &PromptReplyRankMemory, row: usize) -> Result<Option<TrainingGroup>> {
    let entry = &memory.entries[row];
    let candidates = candidates_for(memory, &entry.prompt_unigram, &entry.query, Some(row))?;
    if !candidates
        .iter()
        .any(|candidate| memory.entries[candidate.ordinal].reply == entry.reply)
    {
        return Ok(None);
    }
    Ok(Some(TrainingGroup {
        row,
        reference: entry.reply.clone(),
        candidates,
    }))
}

fn candidates_for(
    memory: &PromptReplyRankMemory,
    query_unigram: &BTreeSet<u32>,
    query: &[i32],
    excluded: Option<usize>,
) -> Result<Vec<RankedCandidate>> {
    let mut ranked = memory
        .entries
        .iter()
        .enumerate()
        .filter(|(ordinal, _)| excluded != Some(*ordinal))
        .map(|(ordinal, entry)| {
            let intersection = query_unigram.intersection(&entry.prompt_unigram).count() as u64;
            let denominator = query_unigram.union(&entry.prompt_unigram).count() as u64;
            (ordinal, intersection, denominator)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(
        |&(left_ordinal, left_num, left_den), &(right_ordinal, right_num, right_den)| {
            compare_ratio(right_num, right_den, left_num, left_den)
                .then(left_ordinal.cmp(&right_ordinal))
        },
    );
    ranked.truncate(PROMPT_REPLY_RANK_TOP_K);
    if ranked.is_empty() {
        return Err(invalid("prompt-reply ranker has no candidates"));
    }
    let min_distance = ranked
        .iter()
        .map(|(ordinal, _, _)| query_distance(query, &memory.entries[*ordinal].query))
        .min()
        .ok_or_else(|| invalid("prompt-reply ranker has no query distance"))?;
    Ok(ranked
        .into_iter()
        .enumerate()
        .map(|(raw_rank, (ordinal, raw_numerator, raw_denominator))| {
            let entry = &memory.entries[ordinal];
            let candidate_len = entry.prompt_unigram.len() as u64;
            let query_len = query_unigram.len() as u64;
            let reply_overlap = entry.reply_unigram.intersection(query_unigram).count() as u64;
            let distance = query_distance(query, &entry.query);
            let features = [
                1,
                q16(raw_numerator, raw_denominator),
                q16(raw_numerator, query_len),
                q16(raw_numerator, candidate_len),
                q16(reply_overlap, entry.reply_unigram.len() as u64),
                clamp_i32(min_distance - distance, -65_535, 0),
                clamp_i32(
                    i64::from(
                        memory
                            .reply_frequency
                            .get(&entry.reply)
                            .copied()
                            .unwrap_or(0),
                    ),
                    0,
                    65_535,
                ),
                clamp_i32(
                    i64::from(memory.mean_reply_len - entry.reply.len() as i32),
                    -65_535,
                    65_535,
                ),
            ];
            RankedCandidate {
                ordinal,
                raw_numerator,
                raw_denominator,
                raw_rank,
                features,
            }
        })
        .collect())
}

fn fit_weights(
    memory: &PromptReplyRankMemory,
    groups: &[TrainingGroup],
    shuffle: bool,
) -> (PromptReplyRanker, u64) {
    let mut ranker = PromptReplyRanker::zero();
    let mut label_changes = 0u64;
    let labels = groups
        .iter()
        .map(|group| {
            let positives = group
                .candidates
                .iter()
                .map(|candidate| memory.entries[candidate.ordinal].reply == group.reference)
                .collect::<Vec<_>>();
            if !shuffle {
                return positives;
            }
            let selected = shuffled_positive_indices(
                group.row,
                group.candidates.len(),
                positives.iter().filter(|value| **value).count(),
            );
            let shuffled = (0..group.candidates.len())
                .map(|index| selected.contains(&index))
                .collect::<Vec<_>>();
            label_changes += positives
                .iter()
                .zip(&shuffled)
                .filter(|(left, right)| left != right)
                .count() as u64;
            shuffled
        })
        .collect::<Vec<_>>();
    for _ in 0..ranker.epochs {
        for (group, positives) in groups.iter().zip(&labels) {
            let Some(negative) = group
                .candidates
                .iter()
                .enumerate()
                .filter(|(index, _)| !positives[*index])
                .max_by_key(|(_, candidate)| {
                    (
                        ranker.score(&candidate.features),
                        Reverse(candidate.raw_rank),
                    )
                })
            else {
                continue;
            };
            let negative_score = ranker.score(&negative.1.features);
            for (index, candidate) in group.candidates.iter().enumerate() {
                if positives[index] && ranker.score(&candidate.features) <= negative_score {
                    for feature in 0..PROMPT_REPLY_RANK_FEATURES {
                        ranker.weights[feature] = (ranker.weights[feature]
                            + ranker.learning_rate
                                * i64::from(
                                    candidate.features[feature] - negative.1.features[feature],
                                ))
                        .clamp(-ranker.weight_limit, ranker.weight_limit);
                    }
                }
            }
        }
    }
    (ranker, label_changes)
}

fn shuffled_positive_indices(
    row: usize,
    candidate_count: usize,
    positive_count: usize,
) -> BTreeSet<usize> {
    let mut indices = (0..candidate_count).collect::<Vec<_>>();
    let mut state =
        PROMPT_REPLY_RANK_SHUFFLE_SEED ^ (row as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    for index in (1..indices.len()).rev() {
        state = splitmix64(state);
        let draw = state as usize % (index + 1);
        indices.swap(index, draw);
    }
    indices.into_iter().take(positive_count).collect()
}

fn candidate_positive(memory: &PromptReplyRankMemory, group: &TrainingGroup) -> usize {
    group
        .candidates
        .iter()
        .find(|candidate| memory.entries[candidate.ordinal].reply == group.reference)
        .map(|candidate| candidate.ordinal)
        .unwrap_or(usize::MAX)
}

fn evaluate_row(
    memory: &PromptReplyRankMemory,
    training: &PromptReplyRankTraining,
    sample: &MemoryWeightSample,
    row: usize,
    cap: usize,
    arm: PromptReplyRankArm,
) -> Result<(CopyKeyReplyRow, PromptReplyRankStep, Vec<u32>)> {
    let reference = normalized_reference(&sample.reply);
    let reachable = reference.len() < cap
        && sample
            .prompt
            .len()
            .checked_add(reference.len())
            .is_some_and(|total| total < memory.context);
    let query_unigram = sample.prompt.iter().copied().collect::<BTreeSet<_>>();
    let query = writer_query_from_entry(memory, &sample.prompt)?;
    let candidates = candidates_for(memory, &query_unigram, &query, None)?;
    let candidate_correct_present = candidates
        .iter()
        .any(|candidate| memory.entries[candidate.ordinal].reply == reference);
    let selected = select(memory, training, &candidates, &query, reference, arm)?;
    let emitted = selected.0.len() < cap
        && sample
            .prompt
            .len()
            .checked_add(selected.0.len())
            .is_some_and(|total| total < memory.context);
    let tokens = if emitted {
        selected.0.clone()
    } else {
        Vec::new()
    };
    let stop = if emitted { "bos" } else { "context" };
    let opening_correct = matches!(
        (tokens.first(), reference.first()),
        (Some(left), Some(right)) if left == right
    );
    let correct_body_positions = tokens
        .iter()
        .zip(reference)
        .filter(|(left, right)| left == right)
        .count() as u64;
    let reply_sha256 = token_hash(&tokens);
    let row_reading = CopyKeyReplyRow {
        row,
        reachable,
        exact_reference: emitted && tokens.as_slice() == reference,
        bos_complete: emitted,
        opening_correct,
        premature_bos: false,
        non_premature_bos: emitted,
        generated_tokens: tokens.len() as u64,
        reference_tokens: reference.len() as u64,
        correct_body_positions,
        copy_decisions: u64::from(emitted),
        fallback_decisions: u64::from(!emitted),
        stop: stop.to_owned(),
        reply_sha256: reply_sha256.clone(),
    };
    let step = PromptReplyRankStep {
        row,
        arm,
        selected_ordinal: selected.1,
        candidate_count: candidates.len(),
        candidate_correct_present,
        ranker_score: selected.2,
        raw_numerator: selected.3,
        raw_denominator: selected.4,
        reply_sha256,
    };
    Ok((row_reading, step, tokens))
}

fn select(
    memory: &PromptReplyRankMemory,
    training: &PromptReplyRankTraining,
    candidates: &[RankedCandidate],
    query: &[i32],
    reference: &[u32],
    arm: PromptReplyRankArm,
) -> Result<(Vec<u32>, usize, Option<i64>, u64, u64)> {
    if arm == PromptReplyRankArm::ReferenceMembershipOracle {
        if let Some((ordinal, _)) = memory
            .entries
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.reply == reference)
        {
            return Ok((reference.to_vec(), ordinal, None, 0, 0));
        }
        let candidate = &candidates[0];
        return Ok((
            memory.entries[candidate.ordinal].reply.clone(),
            candidate.ordinal,
            None,
            candidate.raw_numerator,
            candidate.raw_denominator,
        ));
    }
    if arm == PromptReplyRankArm::FixedOrdinal {
        let entry = &memory.entries[0];
        return Ok((entry.reply.clone(), 0, None, 0, 0));
    }
    if arm == PromptReplyRankArm::QueryGeometry {
        let (ordinal, _) = memory
            .entries
            .iter()
            .enumerate()
            .map(|(ordinal, entry)| (ordinal, query_distance(query, &entry.query)))
            .min_by_key(|&(ordinal, distance)| (distance, ordinal))
            .ok_or_else(|| invalid("prompt-reply ranker geometry table is empty"))?;
        return Ok((memory.entries[ordinal].reply.clone(), ordinal, None, 0, 0));
    }
    let ranker = match arm {
        PromptReplyRankArm::LearnedRanker => &training.trained,
        PromptReplyRankArm::ShuffledRanker => &training.shuffled,
        PromptReplyRankArm::RawUnigramJaccard => {
            let candidate = &candidates[0];
            return Ok((
                memory.entries[candidate.ordinal].reply.clone(),
                candidate.ordinal,
                None,
                candidate.raw_numerator,
                candidate.raw_denominator,
            ));
        }
        PromptReplyRankArm::QueryGeometry
        | PromptReplyRankArm::FixedOrdinal
        | PromptReplyRankArm::ReferenceMembershipOracle => unreachable!("handled above"),
    };
    let selected = candidates
        .iter()
        .max_by_key(|candidate| {
            (
                ranker.score(&candidate.features),
                usize::MAX - candidate.raw_rank,
            )
        })
        .ok_or_else(|| invalid("prompt-reply ranker candidate set is empty"))?;
    Ok((
        memory.entries[selected.ordinal].reply.clone(),
        selected.ordinal,
        Some(ranker.score(&selected.features)),
        selected.raw_numerator,
        selected.raw_denominator,
    ))
}

fn summarize(
    arm: PromptReplyRankArm,
    rows: &[CopyKeyReplyRow],
    steps: &[PromptReplyRankStep],
    replies: &[Vec<u32>],
) -> Result<PromptReplyRankReading> {
    let mut reading = PromptReplyRankReading {
        arm,
        replies: rows.len() as u64,
        reference_reachable: 0,
        emitted: 0,
        exact_reference: 0,
        opening_correct: 0,
        correct_body_positions: 0,
        generated_tokens: 0,
        reference_tokens: 0,
        premature_bos: 0,
        candidate_correct_present: 0,
        distinct_selected_replies: replies
            .iter()
            .filter(|reply| !reply.is_empty())
            .collect::<BTreeSet<_>>()
            .len() as u64,
    };
    for (row, step) in rows.iter().zip(steps) {
        reading.reference_reachable += u64::from(row.reachable);
        reading.emitted += u64::from(row.bos_complete);
        reading.exact_reference += u64::from(row.exact_reference);
        reading.opening_correct += u64::from(row.opening_correct);
        reading.correct_body_positions += row.correct_body_positions;
        reading.generated_tokens += row.generated_tokens;
        reading.reference_tokens += row.reference_tokens;
        reading.premature_bos += u64::from(row.premature_bos);
        reading.candidate_correct_present += u64::from(step.candidate_correct_present);
    }
    Ok(reading)
}

fn writer_query_from_entry(memory: &PromptReplyRankMemory, prompt: &[u32]) -> Result<Vec<i32>> {
    if prompt.is_empty() || prompt.len() > memory.context {
        return Err(invalid(
            "prompt-reply ranker query prompt is empty or too long",
        ));
    }
    let mut forward = CpuFbsForward::from_artifact(&memory.artifact)
        .map_err(|error| invalid(format!("prompt-reply ranker query artifact: {error}")))?;
    let mut step = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        step = Some(
            forward
                .step(token, CpuFbsHead::Writer)
                .map_err(|error| invalid(error.to_string()))?,
        );
    }
    step.map(|step| step.query_q12)
        .ok_or_else(|| invalid("prompt-reply ranker query is empty"))
}

fn writer_query(artifact: &[u8], prompt: &[u32]) -> Result<Vec<i32>> {
    let mut forward =
        CpuFbsForward::from_artifact(artifact).map_err(|error| invalid(error.to_string()))?;
    if prompt.is_empty() || prompt.len() > forward.context_length() {
        return Err(invalid("prompt-reply ranker prompt is empty or too long"));
    }
    let mut step = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        step = Some(
            forward
                .step(token, CpuFbsHead::Writer)
                .map_err(|error| invalid(error.to_string()))?,
        );
    }
    step.map(|step| step.query_q12)
        .ok_or_else(|| invalid("prompt-reply ranker query is empty"))
}

fn q16(numerator: u64, denominator: u64) -> i32 {
    if denominator == 0 {
        return 0;
    }
    let value = (u128::from(numerator) * 65_536) / u128::from(denominator);
    i32::try_from(value.min(65_536)).unwrap_or(65_536)
}

fn clamp_i32(value: i64, minimum: i32, maximum: i32) -> i32 {
    value.clamp(i64::from(minimum), i64::from(maximum)) as i32
}

fn query_distance(left: &[i32], right: &[i32]) -> i64 {
    left.iter()
        .zip(right)
        .map(|(&left, &right)| i64::from(left.abs_diff(right)))
        .sum()
}

fn compare_ratio(left_num: u64, left_den: u64, right_num: u64, right_den: u64) -> Ordering {
    (u128::from(left_num) * u128::from(right_den))
        .cmp(&(u128::from(right_num) * u128::from(left_den)))
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn token_hash(tokens: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(tokens.len() * 4);
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn row_hash(steps: &[PromptReplyRankStep]) -> String {
    let mut hasher = blake3::Hasher::new();
    for step in steps {
        hasher.update(&(step.row as u64).to_le_bytes());
        hasher.update(step.reply_sha256.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn decision_hash(steps: &[PromptReplyRankStep]) -> String {
    let mut hasher = blake3::Hasher::new();
    for step in steps {
        hasher.update(&(step.row as u64).to_le_bytes());
        hasher.update(&(step.selected_ordinal as u64).to_le_bytes());
        hasher.update(&(step.ranker_score.unwrap_or(i64::MIN)).to_le_bytes());
        hasher.update(&step.raw_numerator.to_le_bytes());
        hasher.update(&step.raw_denominator.to_le_bytes());
        hasher.update(step.reply_sha256.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(ordinal: usize, raw_rank: usize, feature: i32) -> RankedCandidate {
        RankedCandidate {
            ordinal,
            raw_numerator: 1,
            raw_denominator: 2,
            raw_rank,
            features: [1, feature, 0, 0, 0, 0, 0, 0],
        }
    }

    fn memory(entries: Vec<RankEntry>) -> PromptReplyRankMemory {
        let mut reply_frequency = BTreeMap::new();
        for entry in &entries {
            *reply_frequency.entry(entry.reply.clone()).or_insert(0) += 1;
        }
        PromptReplyRankMemory {
            artifact: Vec::new(),
            entries,
            reply_frequency,
            mean_reply_len: 2,
            context: 8,
        }
    }

    #[test]
    fn q16_is_exact_capped_and_zero_safe() {
        assert_eq!(q16(0, 4), 0);
        assert_eq!(q16(1, 2), 32_768);
        assert_eq!(q16(4, 4), 65_536);
        assert_eq!(q16(1, 0), 0);
    }

    #[test]
    fn bounded_feature_helpers_clip_without_overflow() {
        assert_eq!(clamp_i32(i64::MIN, -65_535, 0), -65_535);
        assert_eq!(clamp_i32(i64::MAX, 0, 65_535), 65_535);
        assert_eq!(query_distance(&[1, -2], &[3, 2]), 6);
    }

    #[test]
    fn pairwise_fit_moves_the_separating_feature_and_shuffle_preserves_counts() {
        let entries = vec![
            RankEntry {
                prompt_len: 1,
                prompt_unigram: BTreeSet::from([3]),
                reply: vec![10, 11],
                reply_unigram: BTreeSet::from([10, 11]),
                query: vec![0],
            },
            RankEntry {
                prompt_len: 1,
                prompt_unigram: BTreeSet::from([3]),
                reply: vec![10, 11],
                reply_unigram: BTreeSet::from([10, 11]),
                query: vec![0],
            },
            RankEntry {
                prompt_len: 1,
                prompt_unigram: BTreeSet::from([4]),
                reply: vec![12, 13],
                reply_unigram: BTreeSet::from([12, 13]),
                query: vec![0],
            },
        ];
        let memory = memory(entries);
        let groups = vec![TrainingGroup {
            row: 0,
            reference: vec![10, 11],
            candidates: vec![candidate(2, 0, 100), candidate(1, 1, 0)],
        }];
        let (trained, changes) = fit_weights(&memory, &groups, false);
        assert_eq!(changes, 0);
        assert!(trained.weights[1] < 0);
        let selected = shuffled_positive_indices(0, 16, 3);
        assert_eq!(selected.len(), 3);
        assert!(selected.iter().all(|index| *index < 16));
    }

    #[test]
    fn membership_oracle_falls_back_to_raw_when_the_reply_is_absent() {
        let entries = vec![RankEntry {
            prompt_len: 1,
            prompt_unigram: BTreeSet::from([3]),
            reply: vec![10, 11],
            reply_unigram: BTreeSet::from([10, 11]),
            query: vec![0],
        }];
        let memory = memory(entries);
        let training = PromptReplyRankTraining {
            trained: PromptReplyRanker::zero(),
            shuffled: PromptReplyRanker::zero(),
            rows: 1,
            supervised_groups: 0,
            baseline_wrong_groups: 0,
            candidate_rows: 1,
            positive_rows: 0,
            shuffled_label_changes: 0,
        };
        let candidates = vec![candidate(0, 0, 0)];
        let selected = select(
            &memory,
            &training,
            &candidates,
            &[0],
            &[99],
            PromptReplyRankArm::ReferenceMembershipOracle,
        )
        .unwrap();
        assert_eq!(selected.0, [10, 11]);
    }
}
