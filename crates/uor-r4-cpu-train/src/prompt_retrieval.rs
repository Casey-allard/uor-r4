//! Discrete prompt-set retrieval of complete TRAIN replies.
//!
//! Retrieval ranks training prompts by exact token-set Jaccard overlap, then
//! resolves ties with the exact Q12 writer-query L1 distance. It emits the
//! selected complete reply and BOS in one decision. No matrix multiplication,
//! probabilistic normalization or floating-point serving arithmetic is used.

use std::collections::BTreeSet;

use rayon::prelude::*;
use serde::Serialize;

use crate::copy_key_reply::{
    compare_copy_key_reply_rows, CopyKeyReplyComparison, CopyKeyReplyMetric, CopyKeyReplyRow,
};
use crate::reply_eval::{normalized_reference, GeneratedReply};
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const PROMPT_RETRIEVAL_MAX_CAP: usize = 64;
pub const PROMPT_RETRIEVAL_MAX_CONTEXT: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptRetrievalArm {
    UnigramJaccardGeometryTie,
    UnigramJaccardOrdinalTie,
    BigramJaccardGeometryTie,
    UnigramJaccardShuffledReplies,
    QueryGeometryNearest,
    FixedOrdinal,
    ReferenceMembershipOracle,
}

impl PromptRetrievalArm {
    pub const EVALUATION_ARMS: [Self; 7] = [
        Self::UnigramJaccardGeometryTie,
        Self::UnigramJaccardOrdinalTie,
        Self::BigramJaccardGeometryTie,
        Self::UnigramJaccardShuffledReplies,
        Self::QueryGeometryNearest,
        Self::FixedOrdinal,
        Self::ReferenceMembershipOracle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::UnigramJaccardGeometryTie => "unigram_jaccard_geometry_tie",
            Self::UnigramJaccardOrdinalTie => "unigram_jaccard_ordinal_tie",
            Self::BigramJaccardGeometryTie => "bigram_jaccard_geometry_tie",
            Self::UnigramJaccardShuffledReplies => "unigram_jaccard_shuffled_replies",
            Self::QueryGeometryNearest => "query_geometry_nearest",
            Self::FixedOrdinal => "fixed_ordinal",
            Self::ReferenceMembershipOracle => "reference_membership_oracle",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PromptEntry {
    prompt_tokens: usize,
    unigram: BTreeSet<u32>,
    bigram: BTreeSet<(u32, u32)>,
    reply: Vec<u32>,
    query: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptRetrievalMemory {
    entries: Vec<PromptEntry>,
    context: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PromptRetrievalCensus {
    pub entries: u64,
    pub unique_replies: u64,
    pub unique_unigram_sets: u64,
    pub unique_bigram_sets: u64,
    pub prompt_tokens: u64,
    pub reply_tokens: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PromptRetrievalDecision {
    pub selected_ordinal: usize,
    pub similarity_numerator: u64,
    pub similarity_denominator: u64,
    pub best_matches: u64,
    pub geometry_l1: i64,
    pub used_geometry_tie: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PromptRetrievalStep {
    pub row: usize,
    pub arm: PromptRetrievalArm,
    pub selected_ordinal: usize,
    pub similarity_numerator: u64,
    pub similarity_denominator: u64,
    pub best_matches: u64,
    pub geometry_l1: i64,
    pub used_geometry_tie: bool,
    pub selected_tokens: u64,
    pub reply_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PromptRetrievalReading {
    pub arm: PromptRetrievalArm,
    pub replies: u64,
    pub reference_reachable: u64,
    pub exact_reference: u64,
    pub bos_complete: u64,
    pub opening_correct: u64,
    pub premature_bos: u64,
    pub correct_body_positions: u64,
    pub generated_tokens: u64,
    pub reference_tokens: u64,
    pub emitted_replies: u64,
    pub geometry_ties: u64,
    pub distinct_selected_replies: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TimedPromptRetrievalReading {
    pub reading: PromptRetrievalReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<PromptRetrievalStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

impl PromptRetrievalMemory {
    pub fn entries(&self) -> usize {
        self.entries.len()
    }

    pub fn contains_reply(&self, reply: &[u32]) -> bool {
        self.entries.iter().any(|entry| entry.reply == reply)
    }

    pub fn reply_at(&self, ordinal: usize) -> Option<&[u32]> {
        self.entries
            .get(ordinal)
            .map(|entry| entry.reply.as_slice())
    }

    pub fn census(&self) -> PromptRetrievalCensus {
        PromptRetrievalCensus {
            entries: self.entries.len() as u64,
            unique_replies: self
                .entries
                .iter()
                .map(|entry| entry.reply.as_slice())
                .collect::<BTreeSet<_>>()
                .len() as u64,
            unique_unigram_sets: self
                .entries
                .iter()
                .map(|entry| &entry.unigram)
                .collect::<BTreeSet<_>>()
                .len() as u64,
            unique_bigram_sets: self
                .entries
                .iter()
                .map(|entry| &entry.bigram)
                .collect::<BTreeSet<_>>()
                .len() as u64,
            prompt_tokens: self
                .entries
                .iter()
                .map(|entry| entry.prompt_tokens as u64)
                .sum(),
            reply_tokens: self
                .entries
                .iter()
                .map(|entry| entry.reply.len() as u64)
                .sum(),
        }
    }
}

pub fn derive_prompt_retrieval_memory(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
) -> Result<PromptRetrievalMemory> {
    if samples.is_empty() || cap == 0 || cap > PROMPT_RETRIEVAL_MAX_CAP {
        return Err(invalid("prompt retrieval samples or cap differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("prompt retrieval artifact: {error}")))?
        .context_length();
    if context == 0 || context > PROMPT_RETRIEVAL_MAX_CONTEXT {
        return Err(invalid("prompt retrieval context is outside its bound"));
    }
    let entries = samples
        .iter()
        .enumerate()
        .map(|(ordinal, sample)| {
            if sample.prompt.is_empty() || sample.prompt.len() > context {
                return Err(invalid(format!(
                    "prompt retrieval sample {ordinal} prompt is empty or too long"
                )));
            }
            let reply = normalized_reference(&sample.reply).to_vec();
            if reply.is_empty() || reply.len() > cap {
                return Err(invalid(format!(
                    "prompt retrieval sample {ordinal} reply is empty or too long"
                )));
            }
            Ok(PromptEntry {
                prompt_tokens: sample.prompt.len(),
                unigram: sample.prompt.iter().copied().collect(),
                bigram: sample
                    .prompt
                    .windows(2)
                    .map(|pair| (pair[0], pair[1]))
                    .collect(),
                reply,
                query: prompt_query(artifact, &sample.prompt)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(PromptRetrievalMemory { entries, context })
}

pub fn shuffle_prompt_retrieval_replies(
    memory: &PromptRetrievalMemory,
    seed: u64,
) -> Result<(PromptRetrievalMemory, u64)> {
    if memory.entries.len() < 2 {
        return Err(invalid("prompt retrieval shuffle needs two entries"));
    }
    let mut shuffled = memory.clone();
    let mut order = (0..shuffled.entries.len()).collect::<Vec<_>>();
    for index in (1..order.len()).rev() {
        let draw = splitmix64(seed ^ (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)) as usize
            % (index + 1);
        order.swap(index, draw);
        if order[index] == index {
            let alternate = (index + 1) % order.len();
            order.swap(index, alternate);
        }
    }
    let replies = shuffled
        .entries
        .iter()
        .map(|entry| entry.reply.clone())
        .collect::<Vec<_>>();
    let changed = order
        .iter()
        .enumerate()
        .filter(|(index, source)| **source != *index && replies[**source] != replies[*index])
        .count() as u64;
    for (entry, source) in shuffled.entries.iter_mut().zip(order) {
        entry.reply = replies[source].clone();
    }
    if changed == 0 {
        return Err(invalid("prompt retrieval shuffle did not move a reply"));
    }
    Ok((shuffled, changed))
}

pub fn evaluate_prompt_retrieval(
    artifact: &[u8],
    train: &PromptRetrievalMemory,
    shuffled: &PromptRetrievalMemory,
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: PromptRetrievalArm,
) -> Result<TimedPromptRetrievalReading> {
    if samples.is_empty()
        || cap == 0
        || cap > PROMPT_RETRIEVAL_MAX_CAP
        || train.entries() == 0
        || shuffled.entries() != train.entries()
    {
        return Err(invalid("prompt retrieval evaluation inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("prompt retrieval artifact: {error}")))?
        .context_length();
    if context != train.context || context == 0 || context > PROMPT_RETRIEVAL_MAX_CONTEXT {
        return Err(invalid("prompt retrieval evaluation context differs"));
    }
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            evaluate_row(artifact, train, shuffled, sample, row, cap, context, arm)
        })
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
    let generated_tokens_per_second = if wall_elapsed_us == 0 {
        0.0
    } else {
        reading.generated_tokens as f64 * 1_000_000.0 / wall_elapsed_us as f64
    };
    Ok(TimedPromptRetrievalReading {
        reply_sha256: reply_hash(&rows),
        decision_sha256: decision_hash(&steps),
        reading,
        rows,
        steps,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

pub fn compare_prompt_retrieval_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<CopyKeyReplyComparison> {
    compare_copy_key_reply_rows(candidate, control, metric)
}

#[allow(clippy::too_many_arguments)]
fn evaluate_row(
    artifact: &[u8],
    train: &PromptRetrievalMemory,
    shuffled: &PromptRetrievalMemory,
    sample: &MemoryWeightSample,
    row: usize,
    cap: usize,
    context: usize,
    arm: PromptRetrievalArm,
) -> Result<(CopyKeyReplyRow, PromptRetrievalStep, Vec<u32>)> {
    let reference = normalized_reference(&sample.reply);
    let reachable = reference_reachable(sample.prompt.len(), reference.len(), cap, context);
    let query = prompt_query(artifact, &sample.prompt)?;
    let decision = retrieve(train, shuffled, &sample.prompt, &query, arm)?;
    let selected = if arm == PromptRetrievalArm::ReferenceMembershipOracle {
        if train.contains_reply(reference) {
            reference.to_vec()
        } else {
            Vec::new()
        }
    } else {
        let source = if arm == PromptRetrievalArm::UnigramJaccardShuffledReplies {
            shuffled
        } else {
            train
        };
        source.entries[decision.selected_ordinal].reply.clone()
    };
    let selected_reachable = !selected.is_empty()
        && selected.len() < cap
        && sample
            .prompt
            .len()
            .checked_add(selected.len())
            .is_some_and(|total| total < context);
    let generated = if selected_reachable {
        GeneratedReply {
            tokens: selected,
            stop: "bos",
        }
    } else {
        GeneratedReply {
            tokens: Vec::new(),
            stop: "context",
        }
    };
    let opening_correct = matches!(
        (generated.tokens.first(), reference.first()),
        (Some(generated), Some(reference)) if generated == reference
    );
    let correct_body_positions = generated
        .tokens
        .iter()
        .zip(reference)
        .filter(|(generated, reference)| generated == reference)
        .count() as u64;
    let row_reading = CopyKeyReplyRow {
        row,
        reachable,
        exact_reference: generated.stop == "bos" && generated.tokens == reference,
        bos_complete: generated.stop == "bos",
        opening_correct,
        premature_bos: false,
        non_premature_bos: generated.stop == "bos",
        generated_tokens: generated.tokens.len() as u64,
        reference_tokens: reference.len() as u64,
        correct_body_positions,
        copy_decisions: u64::from(generated.stop == "bos"),
        fallback_decisions: u64::from(generated.stop != "bos"),
        stop: generated.stop.to_owned(),
        reply_sha256: token_hash(&generated.tokens),
    };
    let step = PromptRetrievalStep {
        row,
        arm,
        selected_ordinal: decision.selected_ordinal,
        similarity_numerator: decision.similarity_numerator,
        similarity_denominator: decision.similarity_denominator,
        best_matches: decision.best_matches,
        geometry_l1: decision.geometry_l1,
        used_geometry_tie: decision.used_geometry_tie,
        selected_tokens: generated.tokens.len() as u64,
        reply_sha256: row_reading.reply_sha256.clone(),
    };
    Ok((row_reading, step, generated.tokens))
}

fn retrieve(
    train: &PromptRetrievalMemory,
    shuffled: &PromptRetrievalMemory,
    prompt: &[u32],
    query: &[i32],
    arm: PromptRetrievalArm,
) -> Result<PromptRetrievalDecision> {
    if arm == PromptRetrievalArm::ReferenceMembershipOracle {
        return Ok(PromptRetrievalDecision {
            selected_ordinal: 0,
            similarity_numerator: 0,
            similarity_denominator: 0,
            best_matches: 0,
            geometry_l1: 0,
            used_geometry_tie: false,
        });
    }
    if arm == PromptRetrievalArm::FixedOrdinal {
        return Ok(PromptRetrievalDecision {
            selected_ordinal: 0,
            similarity_numerator: 0,
            similarity_denominator: 0,
            best_matches: train.entries.len() as u64,
            geometry_l1: query_l1(&train.entries[0].query, query),
            used_geometry_tie: false,
        });
    }
    if arm == PromptRetrievalArm::QueryGeometryNearest {
        let (selected_ordinal, geometry_l1) = train
            .entries
            .iter()
            .enumerate()
            .map(|(ordinal, entry)| (ordinal, query_l1(&entry.query, query)))
            .min_by_key(|&(ordinal, distance)| (distance, ordinal))
            .ok_or_else(|| invalid("prompt retrieval query table is empty"))?;
        return Ok(PromptRetrievalDecision {
            selected_ordinal,
            similarity_numerator: 0,
            similarity_denominator: 0,
            best_matches: 1,
            geometry_l1,
            used_geometry_tie: false,
        });
    }

    let source = if arm == PromptRetrievalArm::UnigramJaccardShuffledReplies {
        shuffled
    } else {
        train
    };
    let query_unigram = prompt.iter().copied().collect::<BTreeSet<_>>();
    let query_bigram = prompt
        .windows(2)
        .map(|pair| (pair[0], pair[1]))
        .collect::<BTreeSet<_>>();
    let geometry_tie = arm != PromptRetrievalArm::UnigramJaccardOrdinalTie;
    let mut best = None;
    for (ordinal, entry) in source.entries.iter().enumerate() {
        let (numerator, denominator) = match arm {
            PromptRetrievalArm::BigramJaccardGeometryTie => {
                intersection_union(&query_bigram, &entry.bigram)
            }
            PromptRetrievalArm::UnigramJaccardGeometryTie
            | PromptRetrievalArm::UnigramJaccardOrdinalTie
            | PromptRetrievalArm::UnigramJaccardShuffledReplies => {
                intersection_union(&query_unigram, &entry.unigram)
            }
            PromptRetrievalArm::QueryGeometryNearest
            | PromptRetrievalArm::FixedOrdinal
            | PromptRetrievalArm::ReferenceMembershipOracle => unreachable!("handled above"),
        };
        let distance = if geometry_tie {
            query_l1(&entry.query, query)
        } else {
            0
        };
        let replace = match best {
            None => true,
            Some(((best_num, best_den), best_ordinal, best_distance)) => {
                match compare_ratio(numerator, denominator, best_num, best_den) {
                    std::cmp::Ordering::Greater => true,
                    std::cmp::Ordering::Equal if geometry_tie => distance < best_distance,
                    std::cmp::Ordering::Equal => ordinal < best_ordinal,
                    std::cmp::Ordering::Less => false,
                }
            }
        };
        if replace {
            best = Some(((numerator, denominator), ordinal, distance));
        }
    }
    let ((similarity_numerator, similarity_denominator), selected_ordinal, geometry_l1) =
        best.ok_or_else(|| invalid("prompt retrieval table is empty"))?;
    let best_matches = source
        .entries
        .iter()
        .filter(|entry| {
            let (numerator, denominator) = match arm {
                PromptRetrievalArm::BigramJaccardGeometryTie => {
                    intersection_union(&query_bigram, &entry.bigram)
                }
                PromptRetrievalArm::UnigramJaccardGeometryTie
                | PromptRetrievalArm::UnigramJaccardOrdinalTie
                | PromptRetrievalArm::UnigramJaccardShuffledReplies => {
                    intersection_union(&query_unigram, &entry.unigram)
                }
                PromptRetrievalArm::QueryGeometryNearest
                | PromptRetrievalArm::FixedOrdinal
                | PromptRetrievalArm::ReferenceMembershipOracle => unreachable!("handled above"),
            };
            compare_ratio(
                numerator,
                denominator,
                similarity_numerator,
                similarity_denominator,
            ) == std::cmp::Ordering::Equal
        })
        .count() as u64;
    Ok(PromptRetrievalDecision {
        selected_ordinal,
        similarity_numerator,
        similarity_denominator,
        best_matches,
        geometry_l1,
        used_geometry_tie: geometry_tie && best_matches > 1,
    })
}

fn prompt_query(artifact: &[u8], prompt: &[u32]) -> Result<Vec<i32>> {
    let mut forward =
        CpuFbsForward::from_artifact(artifact).map_err(|error| invalid(error.to_string()))?;
    if prompt.is_empty() || prompt.len() > forward.context_length() {
        return Err(invalid("prompt retrieval prompt is empty or too long"));
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
        .ok_or_else(|| invalid("prompt retrieval prompt is empty"))
}

fn intersection_union<T: Ord>(left: &BTreeSet<T>, right: &BTreeSet<T>) -> (u64, u64) {
    (
        left.intersection(right).count() as u64,
        left.union(right).count() as u64,
    )
}

fn compare_ratio(
    left_num: u64,
    left_den: u64,
    right_num: u64,
    right_den: u64,
) -> std::cmp::Ordering {
    (u128::from(left_num) * u128::from(right_den))
        .cmp(&(u128::from(right_num) * u128::from(left_den)))
}

fn query_l1(left: &[i32], right: &[i32]) -> i64 {
    left.iter()
        .zip(right)
        .map(|(&left, &right)| i64::from(left.abs_diff(right)))
        .sum()
}

fn summarize(
    arm: PromptRetrievalArm,
    rows: &[CopyKeyReplyRow],
    steps: &[PromptRetrievalStep],
    replies: &[Vec<u32>],
) -> Result<PromptRetrievalReading> {
    let mut reading = PromptRetrievalReading {
        arm,
        replies: rows.len() as u64,
        reference_reachable: 0,
        exact_reference: 0,
        bos_complete: 0,
        opening_correct: 0,
        premature_bos: 0,
        correct_body_positions: 0,
        generated_tokens: 0,
        reference_tokens: 0,
        emitted_replies: 0,
        geometry_ties: 0,
        distinct_selected_replies: replies
            .iter()
            .filter(|reply| !reply.is_empty())
            .map(|reply| reply.as_slice())
            .collect::<BTreeSet<_>>()
            .len() as u64,
    };
    for row in rows {
        reading.reference_reachable =
            checked_add(reading.reference_reachable, u64::from(row.reachable))?;
        reading.exact_reference =
            checked_add(reading.exact_reference, u64::from(row.exact_reference))?;
        reading.bos_complete = checked_add(reading.bos_complete, u64::from(row.bos_complete))?;
        reading.opening_correct =
            checked_add(reading.opening_correct, u64::from(row.opening_correct))?;
        reading.premature_bos = checked_add(reading.premature_bos, u64::from(row.premature_bos))?;
        reading.correct_body_positions =
            checked_add(reading.correct_body_positions, row.correct_body_positions)?;
        reading.generated_tokens = checked_add(reading.generated_tokens, row.generated_tokens)?;
        reading.reference_tokens = checked_add(reading.reference_tokens, row.reference_tokens)?;
    }
    for step in steps {
        reading.emitted_replies =
            checked_add(reading.emitted_replies, u64::from(step.selected_tokens > 0))?;
        reading.geometry_ties =
            checked_add(reading.geometry_ties, u64::from(step.used_geometry_tie))?;
    }
    Ok(reading)
}

fn reference_reachable(
    prompt_len: usize,
    reference_len: usize,
    cap: usize,
    context: usize,
) -> bool {
    reference_len < cap
        && prompt_len
            .checked_add(reference_len)
            .is_some_and(|total| total < context)
}

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid("prompt retrieval counter overflow"))
}

fn token_hash(tokens: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(tokens.len() * 4);
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn reply_hash(rows: &[CopyKeyReplyRow]) -> String {
    let mut bytes = Vec::new();
    for row in rows {
        bytes.extend_from_slice(&(row.row as u64).to_le_bytes());
        bytes.push(u8::from(row.reachable));
        bytes.push(u8::from(row.exact_reference));
        bytes.push(u8::from(row.bos_complete));
        bytes.extend_from_slice(&row.generated_tokens.to_le_bytes());
        bytes.extend_from_slice(&row.reference_tokens.to_le_bytes());
        bytes.extend_from_slice(&row.correct_body_positions.to_le_bytes());
        bytes.extend_from_slice(row.reply_sha256.as_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn decision_hash(steps: &[PromptRetrievalStep]) -> String {
    let mut bytes = Vec::new();
    for step in steps {
        bytes.extend_from_slice(&(step.row as u64).to_le_bytes());
        bytes.extend_from_slice(&(step.selected_ordinal as u64).to_le_bytes());
        bytes.extend_from_slice(&step.similarity_numerator.to_le_bytes());
        bytes.extend_from_slice(&step.similarity_denominator.to_le_bytes());
        bytes.extend_from_slice(&step.best_matches.to_le_bytes());
        bytes.extend_from_slice(&step.geometry_l1.to_le_bytes());
        bytes.push(u8::from(step.used_geometry_tie));
        bytes.extend_from_slice(&step.selected_tokens.to_le_bytes());
        bytes.extend_from_slice(step.reply_sha256.as_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
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

    fn entry(prompt: &[u32], reply: &[u32], query: &[i32]) -> PromptEntry {
        PromptEntry {
            prompt_tokens: prompt.len(),
            unigram: prompt.iter().copied().collect(),
            bigram: prompt.windows(2).map(|pair| (pair[0], pair[1])).collect(),
            reply: reply.to_vec(),
            query: query.to_vec(),
        }
    }

    fn memory(entries: Vec<PromptEntry>) -> PromptRetrievalMemory {
        PromptRetrievalMemory {
            entries,
            context: 128,
        }
    }

    #[test]
    fn exact_ratio_order_does_not_round() {
        assert_eq!(compare_ratio(1, 3, 2, 7), std::cmp::Ordering::Greater);
        assert_eq!(compare_ratio(2, 6, 1, 3), std::cmp::Ordering::Equal);
    }

    #[test]
    fn geometry_resolves_equal_jaccard_scores() {
        let table = memory(vec![
            entry(&[1, 2, 3], &[10], &[0, 0, 4]),
            entry(&[1, 2, 4], &[11], &[0, 0, 1]),
        ]);
        let decision = retrieve(
            &table,
            &table,
            &[1, 2, 5],
            &[0, 0, 0],
            PromptRetrievalArm::UnigramJaccardGeometryTie,
        )
        .unwrap();
        assert_eq!(decision.selected_ordinal, 1);
        assert_eq!(decision.best_matches, 2);
        assert!(decision.used_geometry_tie);
    }

    #[test]
    fn ordinal_tie_uses_lower_index() {
        let table = memory(vec![
            entry(&[1, 2, 3], &[10], &[0, 0, 4]),
            entry(&[1, 2, 4], &[11], &[0, 0, 1]),
        ]);
        let decision = retrieve(
            &table,
            &table,
            &[1, 2, 5],
            &[0, 0, 0],
            PromptRetrievalArm::UnigramJaccardOrdinalTie,
        )
        .unwrap();
        assert_eq!(decision.selected_ordinal, 0);
        assert!(!decision.used_geometry_tie);
    }

    #[test]
    fn reply_shuffle_preserves_reply_set_and_changes_association() {
        let table = memory(vec![
            entry(&[1], &[10], &[0]),
            entry(&[2], &[11], &[0]),
            entry(&[3], &[12], &[0]),
        ]);
        let (shuffled, changed) = shuffle_prompt_retrieval_replies(&table, 7).unwrap();
        assert!(changed > 0);
        assert_ne!(
            table
                .entries
                .iter()
                .map(|entry| &entry.reply)
                .collect::<Vec<_>>(),
            shuffled
                .entries
                .iter()
                .map(|entry| &entry.reply)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            table
                .entries
                .iter()
                .map(|entry| entry.reply.clone())
                .collect::<BTreeSet<_>>(),
            shuffled
                .entries
                .iter()
                .map(|entry| entry.reply.clone())
                .collect::<BTreeSet<_>>()
        );
    }

    #[test]
    fn membership_checks_complete_reply_identity() {
        let table = memory(vec![entry(&[1], &[10, 11], &[0])]);
        assert!(table.contains_reply(&[10, 11]));
        assert!(!table.contains_reply(&[10]));
    }
}
