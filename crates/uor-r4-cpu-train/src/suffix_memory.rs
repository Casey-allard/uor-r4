//! Trained discrete reply-suffix continuation memory.
//!
//! TRAIN replies contribute suffix-to-next-token counts, including decoder BOS
//! after the last reply token. Generation backs off from the longest known
//! generated reply suffix to the shortest and selects that suffix's most
//! frequent successor. The table is exact, integer-only CPU research state and
//! is not part of the serving artifact.

use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uor_r4_integer::stack::{reply_stop, ReplyEnd, POINT_KEY_BOS};

use crate::copy_key_reply::{
    compare_copy_key_reply_rows, copy_key_reply_decision, CopyKeyReplyArm, CopyKeyReplyMetric,
    CopyKeyReplyRow,
};
use crate::reply_boundary::ReplyBoundaryMemory;
use crate::reply_eval::{normalized_reference, GeneratedReply};
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const SUFFIX_MEMORY_MAX_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuffixMemoryArm {
    TrainedSuffix32,
    TrainedSuffixDepth1,
    ShuffledSuffix32,
    LocalLongest32Boundary,
    WriterBoundary,
    ReferenceLengthOracle,
}

impl SuffixMemoryArm {
    pub const EVALUATION_ARMS: [Self; 6] = [
        Self::TrainedSuffix32,
        Self::TrainedSuffixDepth1,
        Self::ShuffledSuffix32,
        Self::LocalLongest32Boundary,
        Self::WriterBoundary,
        Self::ReferenceLengthOracle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::TrainedSuffix32 => "trained_suffix_32",
            Self::TrainedSuffixDepth1 => "trained_suffix_depth_1",
            Self::ShuffledSuffix32 => "shuffled_suffix_32",
            Self::LocalLongest32Boundary => "local_longest_32_boundary",
            Self::WriterBoundary => "writer_boundary",
            Self::ReferenceLengthOracle => "reference_length_oracle",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SuccessorCounts {
    counts: BTreeMap<u32, u64>,
}

impl SuccessorCounts {
    fn observations(&self) -> u64 {
        self.counts.values().copied().sum()
    }

    fn prediction(&self) -> Option<(u32, u64)> {
        let mut winner = None;
        for (&token, &count) in &self.counts {
            if winner.is_none_or(|(best_token, best_count)| {
                count > best_count || (count == best_count && token < best_token)
            }) {
                winner = Some((token, count));
            }
        }
        winner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuffixMemoryTable {
    entries: BTreeMap<Vec<u32>, SuccessorCounts>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SuffixMemoryPrediction {
    pub depth: usize,
    pub token: u32,
    pub observations: u64,
    pub successor_count: u64,
    pub distinct_targets: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SuffixMemoryCensus {
    pub suffixes: u64,
    pub transitions: u64,
    pub suffixes_by_depth: [u64; SUFFIX_MEMORY_MAX_DEPTH],
    pub transitions_by_depth: [u64; SUFFIX_MEMORY_MAX_DEPTH],
    pub distinct_targets_by_depth: [u64; SUFFIX_MEMORY_MAX_DEPTH],
    pub ambiguous_suffixes: u64,
    pub bos_transitions: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SuffixMemoryStep {
    pub row: usize,
    pub generated_step: usize,
    pub writer_token: u32,
    pub selected_token: u32,
    pub suffix_depth: Option<usize>,
    pub suffix_observations: u64,
    pub suffix_successor_count: u64,
    pub suffix_distinct_targets: u64,
    pub boundary_depth: Option<usize>,
    pub boundary_terminal: u64,
    pub boundary_nonterminal: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SuffixMemoryReading {
    pub arm: SuffixMemoryArm,
    pub replies: u64,
    pub reference_reachable: u64,
    pub exact_reference: u64,
    pub bos_complete: u64,
    pub opening_correct: u64,
    pub premature_bos: u64,
    pub non_premature_bos: u64,
    pub generated_tokens: u64,
    pub reference_tokens: u64,
    pub correct_body_positions: u64,
    pub decisions: u64,
    pub suffix_decisions: u64,
    pub fallback_decisions: u64,
    pub boundary_stops: u64,
    pub selected_depth_counts: [u64; SUFFIX_MEMORY_MAX_DEPTH],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TimedSuffixMemoryReading {
    pub reading: SuffixMemoryReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<SuffixMemoryStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SuffixDecisionChanges {
    pub common_steps: u64,
    pub changed_steps: u64,
    pub divergent_rows: u64,
    pub first_changed_step: Option<(usize, usize)>,
}

impl SuffixMemoryTable {
    pub fn entries(&self) -> usize {
        self.entries.len()
    }

    pub fn census(&self) -> SuffixMemoryCensus {
        let mut census = SuffixMemoryCensus {
            suffixes: self.entries.len() as u64,
            transitions: 0,
            suffixes_by_depth: [0; SUFFIX_MEMORY_MAX_DEPTH],
            transitions_by_depth: [0; SUFFIX_MEMORY_MAX_DEPTH],
            distinct_targets_by_depth: [0; SUFFIX_MEMORY_MAX_DEPTH],
            ambiguous_suffixes: 0,
            bos_transitions: 0,
        };
        let mut targets_by_depth: [BTreeSet<u32>; SUFFIX_MEMORY_MAX_DEPTH] =
            std::array::from_fn(|_| BTreeSet::new());
        for (suffix, successors) in &self.entries {
            let depth = suffix.len();
            if !(1..=SUFFIX_MEMORY_MAX_DEPTH).contains(&depth) {
                continue;
            }
            let observations = successors.observations();
            census.suffixes_by_depth[depth - 1] += 1;
            census.transitions_by_depth[depth - 1] =
                census.transitions_by_depth[depth - 1].saturating_add(observations);
            targets_by_depth[depth - 1].extend(successors.counts.keys().copied());
            census.transitions = census.transitions.saturating_add(observations);
            census.ambiguous_suffixes += u64::from(successors.counts.len() > 1);
            census.bos_transitions = census
                .bos_transitions
                .saturating_add(successors.counts.get(&POINT_KEY_BOS).copied().unwrap_or(0));
        }
        for (depth, targets) in targets_by_depth.iter().enumerate() {
            census.distinct_targets_by_depth[depth] = targets.len() as u64;
        }
        census
    }

    /// Use the longest generated-reply suffix present in the TRAIN table.
    pub fn predict(
        &self,
        generated_reply: &[u32],
        maximum_depth: usize,
    ) -> Option<SuffixMemoryPrediction> {
        let maximum = generated_reply.len().min(maximum_depth);
        for depth in (1..=maximum).rev() {
            if let Some(prediction) = self.predict_at_depth(generated_reply, depth) {
                return Some(prediction);
            }
        }
        None
    }

    /// Back off to the longest suffix with enough observations and a strict
    /// majority successor.
    pub fn predict_confident(
        &self,
        generated_reply: &[u32],
        maximum_depth: usize,
        minimum_observations: u64,
    ) -> Option<SuffixMemoryPrediction> {
        if minimum_observations < 2 {
            return None;
        }
        let maximum = generated_reply.len().min(maximum_depth);
        for depth in (1..=maximum).rev() {
            let Some(prediction) = self.predict_at_depth(generated_reply, depth) else {
                continue;
            };
            if prediction.observations >= minimum_observations
                && prediction.successor_count.saturating_mul(2) > prediction.observations
            {
                return Some(prediction);
            }
        }
        None
    }

    fn predict_at_depth(
        &self,
        generated_reply: &[u32],
        depth: usize,
    ) -> Option<SuffixMemoryPrediction> {
        if depth == 0 || depth > generated_reply.len() {
            return None;
        }
        let suffix = &generated_reply[generated_reply.len() - depth..];
        let successors = self.entries.get(suffix)?;
        let (token, successor_count) = successors.prediction()?;
        Some(SuffixMemoryPrediction {
            depth,
            token,
            observations: successors.observations(),
            successor_count,
            distinct_targets: successors.counts.len() as u64,
        })
    }
}

/// Derive a reply-only suffix continuation table from TRAIN samples.
pub fn derive_suffix_memory(samples: &[MemoryWeightSample]) -> Result<SuffixMemoryTable> {
    derive_suffix_memory_with_terminal(samples, true)
}

/// Derive the same table without decoder BOS as a target.
pub fn derive_suffix_content_memory(samples: &[MemoryWeightSample]) -> Result<SuffixMemoryTable> {
    derive_suffix_memory_with_terminal(samples, false)
}

/// Derive reply-target counts over full prompt-plus-reply histories. Keys may
/// include prompt tokens, so this table reads the same stream surface as local
/// exact copy. Terminal BOS transitions are excluded.
pub fn derive_full_sequence_suffix_content_memory(
    samples: &[MemoryWeightSample],
) -> Result<SuffixMemoryTable> {
    if samples.is_empty() {
        return Err(invalid(
            "full-sequence suffix derivation needs training samples",
        ));
    }
    let mut entries = BTreeMap::<Vec<u32>, SuccessorCounts>::new();
    for sample in samples {
        let reply = normalized_reference(&sample.reply);
        if reply.is_empty() || sample.prompt.is_empty() {
            return Err(invalid(
                "full-sequence suffix derivation saw an empty prompt or reply",
            ));
        }
        let mut history = sample.prompt.clone();
        for &target in reply {
            let maximum = history.len().min(SUFFIX_MEMORY_MAX_DEPTH);
            for depth in 1..=maximum {
                add_successor(&mut entries, &history[history.len() - depth..], target)?;
            }
            history.push(target);
        }
    }
    if entries.is_empty() {
        return Err(invalid(
            "full-sequence suffix derivation found no transitions",
        ));
    }
    Ok(SuffixMemoryTable { entries })
}

fn derive_suffix_memory_with_terminal(
    samples: &[MemoryWeightSample],
    include_terminal: bool,
) -> Result<SuffixMemoryTable> {
    if samples.is_empty() {
        return Err(invalid("suffix-memory derivation needs training samples"));
    }
    let mut entries = BTreeMap::<Vec<u32>, SuccessorCounts>::new();
    for sample in samples {
        let reply = normalized_reference(&sample.reply);
        if reply.is_empty() {
            return Err(invalid("suffix-memory derivation saw an empty reply"));
        }
        for end in 0..reply.len() {
            let target = if end + 1 < reply.len() {
                Some(reply[end + 1])
            } else if include_terminal {
                Some(POINT_KEY_BOS)
            } else {
                None
            };
            let Some(target) = target else {
                continue;
            };
            let maximum = (end + 1).min(SUFFIX_MEMORY_MAX_DEPTH);
            for depth in 1..=maximum {
                add_successor(&mut entries, &reply[end + 1 - depth..=end], target)?;
            }
        }
    }
    if entries.is_empty() {
        return Err(invalid("suffix-memory derivation found no transitions"));
    }
    Ok(SuffixMemoryTable { entries })
}

fn add_successor(
    entries: &mut BTreeMap<Vec<u32>, SuccessorCounts>,
    suffix: &[u32],
    target: u32,
) -> Result<()> {
    let count = entries
        .entry(suffix.to_vec())
        .or_default()
        .counts
        .entry(target)
        .or_default();
    *count = count
        .checked_add(1)
        .ok_or_else(|| invalid("suffix-memory successor count overflows"))?;
    Ok(())
}

/// Redistribute successor labels among equal-depth occurrences while
/// preserving every suffix's observation count and the target multiset.
pub fn shuffle_suffix_memory(
    original: &SuffixMemoryTable,
    seed: u64,
) -> Result<(SuffixMemoryTable, u64)> {
    for attempt in 0..64u64 {
        let mut shuffled = original.clone();
        for depth in 1..=SUFFIX_MEMORY_MAX_DEPTH {
            let keys = shuffled
                .entries
                .iter()
                .filter(|(suffix, _)| suffix.len() == depth)
                .map(|(suffix, successors)| (suffix.clone(), successors.observations()))
                .collect::<Vec<_>>();
            if keys.is_empty() {
                continue;
            }
            let mut targets = Vec::new();
            for (suffix, observations) in &keys {
                let successors = shuffled.entries.get(suffix).expect("suffix exists");
                for (&target, &count) in &successors.counts {
                    for _ in 0..count {
                        targets.push(target);
                    }
                }
                if successors.observations() != *observations {
                    return Err(invalid("suffix-memory shuffle support differs"));
                }
            }
            for index in (1..targets.len()).rev() {
                let draw = splitmix64(
                    seed ^ (attempt << 48)
                        ^ (depth as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                        ^ (index as u64).wrapping_mul(0xd1b5_4a32_d192_ed03),
                ) as usize
                    % (index + 1);
                targets.swap(index, draw);
            }
            let mut cursor = 0usize;
            for (suffix, observations) in keys {
                let count = usize::try_from(observations)
                    .map_err(|_| invalid("suffix-memory observation count is too large"))?;
                let successor_counts = shuffled.entries.get_mut(&suffix).expect("suffix exists");
                successor_counts.counts.clear();
                for &target in &targets[cursor..cursor + count] {
                    let entry = successor_counts.counts.entry(target).or_default();
                    *entry = entry
                        .checked_add(1)
                        .ok_or_else(|| invalid("suffix-memory shuffle count overflows"))?;
                }
                cursor += count;
            }
            if cursor != targets.len() {
                return Err(invalid("suffix-memory shuffle assignment differs"));
            }
        }
        let changed = shuffled
            .entries
            .iter()
            .filter(|(suffix, successors)| {
                original
                    .entries
                    .get(*suffix)
                    .is_some_and(|old| old.counts != successors.counts)
            })
            .count() as u64;
        if changed > 0 {
            return Ok((shuffled, changed));
        }
    }
    Err(invalid(
        "suffix-memory shuffle could not change a successor association",
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn generate_suffix_memory_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    reference_length: usize,
    arm: SuffixMemoryArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    boundary: Option<&ReplyBoundaryMemory>,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<SuffixMemoryStep>)> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("suffix-memory forward: {error}")))?;
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "suffix-memory prompt, cap or artifact context is invalid",
        ));
    }

    let mut stream = Vec::with_capacity(context);
    let mut next = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("suffix-memory prompt step: {error}")))?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some(step.token);
        }
    }
    let mut writer_token = next.ok_or_else(|| invalid("suffix-memory prompt is empty"))?;
    let mut tokens = Vec::with_capacity(cap);
    let mut steps = Vec::with_capacity(cap);

    loop {
        let prediction = match arm {
            SuffixMemoryArm::TrainedSuffix32 => suffix_memory
                .ok_or_else(|| invalid("trained suffix arm has no table"))?
                .predict(&tokens, SUFFIX_MEMORY_MAX_DEPTH),
            SuffixMemoryArm::TrainedSuffixDepth1 => suffix_memory
                .ok_or_else(|| invalid("depth-one suffix arm has no table"))?
                .predict(&tokens, 1),
            SuffixMemoryArm::ShuffledSuffix32 => suffix_memory
                .ok_or_else(|| invalid("shuffled suffix arm has no table"))?
                .predict(&tokens, SUFFIX_MEMORY_MAX_DEPTH),
            SuffixMemoryArm::LocalLongest32Boundary
            | SuffixMemoryArm::WriterBoundary
            | SuffixMemoryArm::ReferenceLengthOracle => None,
        };

        let boundary_hit = match arm {
            SuffixMemoryArm::TrainedSuffix32
            | SuffixMemoryArm::TrainedSuffixDepth1
            | SuffixMemoryArm::ShuffledSuffix32
                if prediction.is_none() =>
            {
                boundary
                    .ok_or_else(|| invalid("suffix-memory arm has no boundary table"))?
                    .stop_decision(&tokens)
            }
            SuffixMemoryArm::LocalLongest32Boundary | SuffixMemoryArm::WriterBoundary => boundary
                .ok_or_else(|| invalid("local copy arm has no boundary table"))?
                .stop_decision(&tokens),
            SuffixMemoryArm::ReferenceLengthOracle => (tokens.len() == reference_length).then_some(
                crate::reply_boundary::ReplyBoundaryHit {
                    depth: 0,
                    terminal: 1,
                    nonterminal: 0,
                },
            ),
            _ => None,
        };
        if let Some(hit) = boundary_hit {
            steps.push(SuffixMemoryStep {
                row,
                generated_step: tokens.len(),
                writer_token,
                selected_token: POINT_KEY_BOS,
                suffix_depth: None,
                suffix_observations: 0,
                suffix_successor_count: 0,
                suffix_distinct_targets: 0,
                boundary_depth: Some(hit.depth),
                boundary_terminal: hit.terminal,
                boundary_nonterminal: hit.nonterminal,
            });
            return Ok((
                GeneratedReply {
                    tokens,
                    stop: "bos",
                },
                steps,
            ));
        }

        let decision = match arm {
            SuffixMemoryArm::TrainedSuffix32
            | SuffixMemoryArm::TrainedSuffixDepth1
            | SuffixMemoryArm::ShuffledSuffix32 => {
                let token = prediction.map_or(writer_token, |hit| hit.token);
                (token, prediction)
            }
            SuffixMemoryArm::LocalLongest32Boundary | SuffixMemoryArm::ReferenceLengthOracle => {
                let decision = copy_key_reply_decision(
                    &stream,
                    writer_token,
                    CopyKeyReplyArm::Longest32To1,
                    seed,
                    row,
                    tokens.len(),
                );
                (decision.token, None)
            }
            SuffixMemoryArm::WriterBoundary => (writer_token, None),
        };
        let (selected_token, prediction) = decision;
        steps.push(SuffixMemoryStep {
            row,
            generated_step: tokens.len(),
            writer_token,
            selected_token,
            suffix_depth: prediction.map(|hit| hit.depth),
            suffix_observations: prediction.map_or(0, |hit| hit.observations),
            suffix_successor_count: prediction.map_or(0, |hit| hit.successor_count),
            suffix_distinct_targets: prediction.map_or(0, |hit| hit.distinct_targets),
            boundary_depth: None,
            boundary_terminal: 0,
            boundary_nonterminal: 0,
        });
        let stop = reply_stop(selected_token, tokens.len(), prompt.len(), cap, context);
        if stop != Some(ReplyEnd::Bos) {
            tokens.push(selected_token);
        }
        if let Some(stop) = stop {
            return Ok((
                GeneratedReply {
                    tokens,
                    stop: match stop {
                        ReplyEnd::Bos => "bos",
                        ReplyEnd::Length => "cap",
                        ReplyEnd::Context => "context",
                    },
                },
                steps,
            ));
        }
        stream.push(selected_token);
        let step = forward
            .step(selected_token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("suffix-memory generated step: {error}")))?;
        writer_token = step.token;
    }
}

pub fn evaluate_suffix_memory_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: SuffixMemoryArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    boundary: Option<&ReplyBoundaryMemory>,
    seed: u64,
) -> Result<TimedSuffixMemoryReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("suffix-memory reply inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("suffix-memory context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply).to_vec();
            if sample.prompt.len() > context || reference.len() > cap {
                return Err(invalid("suffix-memory sample exceeds its context or cap"));
            }
            let (generated, steps) = generate_suffix_memory_reply(
                artifact,
                &sample.prompt,
                cap,
                reference.len(),
                arm,
                suffix_memory,
                boundary,
                seed,
                row,
            )?;
            let reachable = reference_reachable(sample.prompt.len(), reference.len(), cap, context);
            let opening_correct = matches!(
                (generated.tokens.first(), reference.first()),
                (Some(generated), Some(reference)) if generated == reference
            );
            let correct_body_positions = generated
                .tokens
                .iter()
                .zip(&reference)
                .filter(|(generated, reference)| generated == reference)
                .count() as u64;
            let suffix_decisions = steps
                .iter()
                .filter(|step| step.suffix_depth.is_some())
                .count() as u64;
            let fallback_decisions = steps.len() as u64 - suffix_decisions;
            let premature_bos = generated.stop == "bos" && generated.tokens.len() < reference.len();
            let non_premature_bos = generated.stop == "bos" && !premature_bos;
            let row_reading = CopyKeyReplyRow {
                row,
                reachable,
                exact_reference: generated.stop == "bos" && generated.tokens == reference,
                bos_complete: generated.stop == "bos",
                opening_correct,
                premature_bos,
                non_premature_bos,
                generated_tokens: generated.tokens.len() as u64,
                reference_tokens: reference.len() as u64,
                correct_body_positions,
                copy_decisions: suffix_decisions,
                fallback_decisions,
                stop: generated.stop.to_owned(),
                reply_sha256: token_hash(&generated.tokens),
            };
            Ok((row_reading, steps))
        })
        .collect::<Result<Vec<_>>>()?;
    let wall_elapsed_us = started.elapsed().as_micros().try_into().unwrap_or(u64::MAX);
    let (rows, steps): (Vec<_>, Vec<_>) = evaluated.into_iter().unzip();
    let steps = steps.into_iter().flatten().collect::<Vec<_>>();
    let reading = summarize(arm, &rows, &steps)?;
    let reply_sha256 = reply_hash(&rows);
    let decision_sha256 = decision_hash(&steps);
    let generated_tokens_per_second = if wall_elapsed_us == 0 {
        0.0
    } else {
        reading.generated_tokens as f64 * 1_000_000.0 / wall_elapsed_us as f64
    };
    Ok(TimedSuffixMemoryReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

pub fn compare_suffix_memory_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<crate::copy_key_reply::CopyKeyReplyComparison> {
    compare_copy_key_reply_rows(candidate, control, metric)
}

pub fn suffix_decision_changes(
    candidate: &TimedSuffixMemoryReading,
    control: &TimedSuffixMemoryReading,
) -> SuffixDecisionChanges {
    let reachable_rows = candidate
        .rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .collect::<BTreeSet<_>>();
    let candidate_steps = candidate
        .steps
        .iter()
        .filter(|step| reachable_rows.contains(&step.row))
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let control_steps = control
        .steps
        .iter()
        .filter(|step| reachable_rows.contains(&step.row))
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let changed = candidate_steps
        .iter()
        .filter_map(|(key, token)| {
            control_steps
                .get(key)
                .filter(|control_token| *control_token != token)
                .map(|_| *key)
        })
        .collect::<Vec<_>>();
    let divergent_rows = changed
        .iter()
        .map(|(row, _)| *row)
        .collect::<BTreeSet<_>>()
        .len() as u64;
    SuffixDecisionChanges {
        common_steps: candidate_steps
            .keys()
            .filter(|key| control_steps.contains_key(key))
            .count() as u64,
        changed_steps: changed.len() as u64,
        divergent_rows,
        first_changed_step: changed.first().copied(),
    }
}

fn summarize(
    arm: SuffixMemoryArm,
    rows: &[CopyKeyReplyRow],
    steps: &[SuffixMemoryStep],
) -> Result<SuffixMemoryReading> {
    let mut reading = SuffixMemoryReading {
        arm,
        replies: rows.len() as u64,
        reference_reachable: 0,
        exact_reference: 0,
        bos_complete: 0,
        opening_correct: 0,
        premature_bos: 0,
        non_premature_bos: 0,
        generated_tokens: 0,
        reference_tokens: 0,
        correct_body_positions: 0,
        decisions: steps.len() as u64,
        suffix_decisions: 0,
        fallback_decisions: 0,
        boundary_stops: 0,
        selected_depth_counts: [0; SUFFIX_MEMORY_MAX_DEPTH],
    };
    for row in rows {
        reading.reference_reachable += u64::from(row.reachable);
        reading.exact_reference += u64::from(row.reachable && row.exact_reference);
        reading.bos_complete += u64::from(row.reachable && row.bos_complete);
        reading.opening_correct += u64::from(row.reachable && row.opening_correct);
        reading.premature_bos += u64::from(row.reachable && row.premature_bos);
        reading.non_premature_bos += u64::from(row.reachable && row.non_premature_bos);
        reading.generated_tokens = checked_add(reading.generated_tokens, row.generated_tokens)?;
        reading.reference_tokens = checked_add(reading.reference_tokens, row.reference_tokens)?;
        reading.correct_body_positions =
            checked_add(reading.correct_body_positions, row.correct_body_positions)?;
    }
    let reachable_rows = rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .collect::<BTreeSet<_>>();
    for step in steps
        .iter()
        .filter(|step| reachable_rows.contains(&step.row))
    {
        if let Some(depth) = step.suffix_depth {
            reading.suffix_decisions = checked_add(reading.suffix_decisions, 1)?;
            if depth <= SUFFIX_MEMORY_MAX_DEPTH {
                reading.selected_depth_counts[depth - 1] += 1;
            }
        } else {
            reading.fallback_decisions = checked_add(reading.fallback_decisions, 1)?;
        }
        reading.boundary_stops = checked_add(
            reading.boundary_stops,
            u64::from(step.boundary_depth.is_some()),
        )?;
    }
    Ok(reading)
}

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid("suffix-memory counter overflows"))
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
        hasher.update(&row.copy_decisions.to_le_bytes());
        hasher.update(&row.fallback_decisions.to_le_bytes());
        hasher.update(row.stop.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn decision_hash(steps: &[SuffixMemoryStep]) -> String {
    let mut hasher = blake3::Hasher::new();
    for step in steps {
        hasher.update(&(step.row as u64).to_le_bytes());
        hasher.update(&(step.generated_step as u64).to_le_bytes());
        hasher.update(&step.writer_token.to_le_bytes());
        hasher.update(&step.selected_token.to_le_bytes());
        hasher.update(&(step.suffix_depth.unwrap_or(0) as u16).to_le_bytes());
        hasher.update(&step.suffix_observations.to_le_bytes());
        hasher.update(&step.suffix_successor_count.to_le_bytes());
        hasher.update(&step.suffix_distinct_targets.to_le_bytes());
        hasher.update(&(step.boundary_depth.unwrap_or(0) as u16).to_le_bytes());
        hasher.update(&step.boundary_terminal.to_le_bytes());
        hasher.update(&step.boundary_nonterminal.to_le_bytes());
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
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    fn sample(reply: &[u32]) -> MemoryWeightSample {
        MemoryWeightSample {
            prompt: vec![9],
            reply: reply.to_vec(),
        }
    }

    fn artifact() -> Vec<u8> {
        export_fbs_trunk_uorlut01(&synthetic_fbs_trunk_with(
            193,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 3,
                context: 32,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        ))
        .unwrap()
    }

    fn boundary() -> ReplyBoundaryMemory {
        crate::reply_boundary::derive_reply_boundary_memory(&[sample(&[40, 10, 20, 30, 1])])
            .unwrap()
    }

    fn writer_prefix(artifact: &[u8]) -> Vec<u32> {
        generate_suffix_memory_reply(
            artifact,
            &[10, 20, 30, 40, 10, 20, 30],
            16,
            8,
            SuffixMemoryArm::WriterBoundary,
            None,
            Some(&boundary()),
            193,
            0,
        )
        .unwrap()
        .0
        .tokens
    }

    #[test]
    fn longest_known_suffix_backs_off_to_plurality() {
        let table = derive_suffix_memory(&[
            sample(&[10, 20, 30, 40, 1]),
            sample(&[50, 20, 30, 41, 1]),
            sample(&[60, 20, 30, 41, 1]),
        ])
        .unwrap();
        let hit = table.predict(&[20, 30], SUFFIX_MEMORY_MAX_DEPTH).unwrap();
        assert_eq!(hit.depth, 2);
        assert_eq!(hit.token, 41);
        assert_eq!(hit.successor_count, 2);
        assert_eq!(hit.observations, 3);
    }

    #[test]
    fn unseen_context_backs_off_to_a_known_shorter_suffix() {
        let table = derive_suffix_memory(&[sample(&[10, 20, 30, 40, 1])]).unwrap();
        let hit = table.predict(&[99, 20], SUFFIX_MEMORY_MAX_DEPTH).unwrap();
        assert_eq!(hit.depth, 1);
        assert_eq!(hit.token, 30);
    }

    #[test]
    fn terminal_suffix_can_predict_bos() {
        let table = derive_suffix_memory(&[sample(&[10, 20, 30, 40, 1])]).unwrap();
        let hit = table
            .predict(&[10, 20, 30, 40], SUFFIX_MEMORY_MAX_DEPTH)
            .unwrap();
        assert_eq!(hit.depth, 4);
        assert_eq!(hit.token, POINT_KEY_BOS);
    }

    #[test]
    fn content_only_table_has_no_terminal_target() {
        let table = derive_suffix_content_memory(&[sample(&[10, 20, 30, 40, 1])]).unwrap();
        assert_eq!(
            table.predict(&[10, 20, 30, 40], SUFFIX_MEMORY_MAX_DEPTH),
            None
        );
        assert_eq!(table.census().bos_transitions, 0);
        assert_eq!(
            table.predict(&[20], SUFFIX_MEMORY_MAX_DEPTH).unwrap().token,
            30
        );
    }

    #[test]
    fn full_sequence_table_learns_prompt_conditioned_reply_starts() {
        let table = derive_full_sequence_suffix_content_memory(&[MemoryWeightSample {
            prompt: vec![10, 20],
            reply: vec![30, 40, 1],
        }])
        .unwrap();
        assert_eq!(
            table
                .predict(&[10, 20], SUFFIX_MEMORY_MAX_DEPTH)
                .unwrap()
                .token,
            30
        );
        assert_eq!(
            table
                .predict(&[10, 20, 30], SUFFIX_MEMORY_MAX_DEPTH)
                .unwrap()
                .token,
            40
        );
        assert_eq!(table.census().bos_transitions, 0);
    }

    #[test]
    fn confident_prediction_requires_support_and_strict_majority() {
        let table = derive_suffix_content_memory(&[
            sample(&[10, 20, 30, 40, 1]),
            sample(&[50, 20, 30, 41, 1]),
            sample(&[60, 20, 30, 41, 1]),
        ])
        .unwrap();
        let hit = table
            .predict_confident(&[20, 30], SUFFIX_MEMORY_MAX_DEPTH, 2)
            .unwrap();
        assert_eq!(hit.depth, 2);
        assert_eq!(hit.token, 41);
        assert_eq!(hit.observations, 3);
        assert_eq!(hit.successor_count, 2);

        let tied = derive_suffix_content_memory(&[
            sample(&[10, 20, 30, 40, 1]),
            sample(&[50, 20, 30, 41, 1]),
        ])
        .unwrap();
        assert_eq!(
            tied.predict_confident(&[20, 30], SUFFIX_MEMORY_MAX_DEPTH, 2),
            None
        );
    }

    #[test]
    fn equal_counts_tie_to_the_smaller_token() {
        let table = derive_suffix_memory(&[sample(&[10, 40, 1]), sample(&[10, 30, 1])]).unwrap();
        let hit = table.predict(&[10], SUFFIX_MEMORY_MAX_DEPTH).unwrap();
        assert_eq!(hit.token, 30);
        assert_eq!(hit.successor_count, 1);
    }

    #[test]
    fn shuffled_table_preserves_support_and_target_multiset() {
        let table = derive_suffix_memory(&[
            sample(&[10, 20, 30, 40, 1]),
            sample(&[50, 20, 30, 41, 1]),
            sample(&[60, 21, 31, 42, 1]),
        ])
        .unwrap();
        let (shuffled, changed) = shuffle_suffix_memory(&table, 193).unwrap();
        assert!(changed > 0);
        let before = table.census();
        let after = shuffled.census();
        assert_eq!(before.suffixes, after.suffixes);
        assert_eq!(before.transitions, after.transitions);
        assert_eq!(before.suffixes_by_depth, after.suffixes_by_depth);
        assert_eq!(before.transitions_by_depth, after.transitions_by_depth);
        for (suffix, successors) in &table.entries {
            assert_eq!(
                successors.observations(),
                shuffled.entries[suffix].observations()
            );
        }
        let target_sum_by_depth = |memory: &SuffixMemoryTable| {
            let mut totals = std::array::from_fn::<_, SUFFIX_MEMORY_MAX_DEPTH, _>(|_| {
                BTreeMap::<u32, u64>::new()
            });
            for (suffix, successors) in &memory.entries {
                for (&target, &count) in &successors.counts {
                    *totals[suffix.len() - 1].entry(target).or_default() += count;
                }
            }
            totals
        };
        assert_eq!(target_sum_by_depth(&table), target_sum_by_depth(&shuffled));
    }

    #[test]
    fn complete_reply_integration_overrides_writer_predicts_bos_and_falls_back() {
        let artifact = artifact();
        let prefix = writer_prefix(&artifact);
        assert!(prefix.len() >= 2);
        let writer_first = prefix[0];
        let writer_second = prefix[1];
        let predicted = (10..512).find(|token| *token != writer_second).unwrap();
        let override_table =
            derive_suffix_memory(&[sample(&[writer_first, predicted, 1])]).unwrap();
        let trained = generate_suffix_memory_reply(
            &artifact,
            &[10, 20, 30, 40, 10, 20, 30],
            16,
            8,
            SuffixMemoryArm::TrainedSuffix32,
            Some(&override_table),
            Some(&boundary()),
            193,
            0,
        )
        .unwrap();
        assert_eq!(trained.0.tokens[0], writer_first);
        assert_eq!(trained.0.tokens[1], predicted);
        assert_eq!(trained.1[1].suffix_depth, Some(1));

        let terminal_table = derive_suffix_memory(&[sample(&[writer_first])]).unwrap();
        let stopped = generate_suffix_memory_reply(
            &artifact,
            &[10, 20, 30, 40, 10, 20, 30],
            16,
            8,
            SuffixMemoryArm::TrainedSuffix32,
            Some(&terminal_table),
            Some(&boundary()),
            193,
            0,
        )
        .unwrap();
        assert_eq!(stopped.0.tokens, [writer_first]);
        assert_eq!(stopped.0.stop, "bos");
        assert_eq!(stopped.1[1].suffix_depth, Some(1));
        assert_eq!(stopped.1[1].selected_token, POINT_KEY_BOS);

        let other = (10..512)
            .find(|token| *token != writer_first && *token != predicted)
            .unwrap();
        let unknown_table = derive_suffix_memory(&[sample(&[other, predicted, 1])]).unwrap();
        let fallback = generate_suffix_memory_reply(
            &artifact,
            &[10, 20, 30, 40, 10, 20, 30],
            16,
            8,
            SuffixMemoryArm::TrainedSuffix32,
            Some(&unknown_table),
            Some(&boundary()),
            193,
            0,
        )
        .unwrap();
        assert_eq!(fallback.0.tokens[0], writer_first);
        assert_eq!(fallback.0.tokens[1], writer_second);
        assert_eq!(fallback.1[1].suffix_depth, None);
    }

    #[test]
    fn shuffled_complete_reply_takes_a_different_trajectory() {
        let artifact = artifact();
        let prefix = writer_prefix(&artifact);
        let writer_first = prefix[0];
        let other_a = (10..512).find(|token| *token != writer_first).unwrap();
        let other_b = (10..512)
            .find(|token| *token != writer_first && *token != other_a)
            .unwrap();
        let predicted = (10..512)
            .find(|token| *token != other_a && *token != other_b)
            .unwrap();
        let table = derive_suffix_memory(&[
            sample(&[writer_first, predicted, 1]),
            sample(&[other_a, other_b, 1]),
        ])
        .unwrap();
        let original = table
            .predict(&[writer_first], SUFFIX_MEMORY_MAX_DEPTH)
            .unwrap();
        let (shuffled, _) = (0..64)
            .find_map(|seed| {
                let (control, changed) = shuffle_suffix_memory(&table, seed).unwrap();
                (changed > 0
                    && control.predict(&[writer_first], SUFFIX_MEMORY_MAX_DEPTH) != Some(original))
                .then_some((control, seed))
            })
            .unwrap();
        let true_reply = generate_suffix_memory_reply(
            &artifact,
            &[10, 20, 30, 40, 10, 20, 30],
            16,
            8,
            SuffixMemoryArm::TrainedSuffix32,
            Some(&table),
            Some(&boundary()),
            193,
            0,
        )
        .unwrap();
        let shuffled_reply = generate_suffix_memory_reply(
            &artifact,
            &[10, 20, 30, 40, 10, 20, 30],
            16,
            8,
            SuffixMemoryArm::ShuffledSuffix32,
            Some(&shuffled),
            Some(&boundary()),
            193,
            0,
        )
        .unwrap();
        assert_ne!(true_reply.0.tokens, shuffled_reply.0.tokens);
        assert_ne!(
            true_reply.1[1].selected_token,
            shuffled_reply.1[1].selected_token
        );
    }
}
