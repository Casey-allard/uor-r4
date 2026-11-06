//! Completion-level continuation memory for the CPU exact-content decoder.
//!
//! The table maps a full prompt-plus-generated suffix to complete remaining
//! reply sequences. A confident closure is emitted atomically, so termination
//! is part of the continuation decision rather than a separate per-token stop
//! classifier.

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
use crate::suffix_memory::{SuffixMemoryTable, SUFFIX_MEMORY_MAX_DEPTH};
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const CONTINUATION_CLOSURE_MIN_DEPTH: usize = 4;
pub const CONTINUATION_CLOSURE_MAX_DEPTH: usize = SUFFIX_MEMORY_MAX_DEPTH;
pub const CONTINUATION_CLOSURE_MIN_OBSERVATIONS: u64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuationClosureArm {
    SourceClosure,
    TokenwiseContent,
    SourceClosureShuffled,
    ClosureOnly,
    ContentNoBoundary,
    LocalReplyBoundary,
    WriterBoundary,
    ContentReferenceOracle,
}

impl ContinuationClosureArm {
    pub const EVALUATION_ARMS: [Self; 8] = [
        Self::SourceClosure,
        Self::TokenwiseContent,
        Self::SourceClosureShuffled,
        Self::ClosureOnly,
        Self::ContentNoBoundary,
        Self::LocalReplyBoundary,
        Self::WriterBoundary,
        Self::ContentReferenceOracle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::SourceClosure => "source_closure",
            Self::TokenwiseContent => "tokenwise_content",
            Self::SourceClosureShuffled => "source_closure_shuffled",
            Self::ClosureOnly => "closure_only",
            Self::ContentNoBoundary => "content_no_boundary",
            Self::LocalReplyBoundary => "local_reply_boundary",
            Self::WriterBoundary => "writer_boundary",
            Self::ContentReferenceOracle => "content_reference_oracle",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ContinuationCounts {
    counts: BTreeMap<Vec<u32>, u64>,
}

impl ContinuationCounts {
    fn observations(&self) -> u64 {
        self.counts.values().copied().sum()
    }

    fn prediction(&self) -> Option<(&[u32], u64)> {
        let mut winner = None;
        for (continuation, &count) in &self.counts {
            if winner.is_none_or(|(best, best_count): (&Vec<u32>, u64)| {
                count > best_count || (count == best_count && continuation < best)
            }) {
                winner = Some((continuation, count));
            }
        }
        winner.map(|(continuation, count)| (continuation.as_slice(), count))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContinuationClosureMemory {
    entries: BTreeMap<Vec<u32>, ContinuationCounts>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ContinuationClosurePrediction {
    pub depth: usize,
    pub observations: u64,
    pub continuation_count: u64,
    pub distinct_continuations: u64,
    pub continuation: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ContinuationClosureCensus {
    pub suffixes: u64,
    pub observations: u64,
    pub continuation_tokens: u64,
    pub empty_continuations: u64,
    pub suffixes_by_depth: [u64; CONTINUATION_CLOSURE_MAX_DEPTH],
    pub observations_by_depth: [u64; CONTINUATION_CLOSURE_MAX_DEPTH],
    pub continuation_tokens_by_depth: [u64; CONTINUATION_CLOSURE_MAX_DEPTH],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ContinuationClosureStep {
    pub row: usize,
    pub generated_step: usize,
    pub writer_token: u32,
    pub local_token: Option<u32>,
    pub global_token: Option<u32>,
    pub source_token: u32,
    pub selected_token: u32,
    pub used_global: bool,
    pub local_copy_depth: Option<usize>,
    pub global_depth: Option<usize>,
    pub closure_depth: Option<usize>,
    pub closure_observations: u64,
    pub closure_count: u64,
    pub closure_distinct: u64,
    pub closure_tokens: u64,
    pub continuation_sha256: String,
    pub stop_kind: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ContinuationClosureReading {
    pub arm: ContinuationClosureArm,
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
    pub local_decisions: u64,
    pub global_decisions: u64,
    pub writer_decisions: u64,
    pub closure_decisions: u64,
    pub closure_changes: u64,
    pub closure_tokens: u64,
    pub reply_boundary_stops: u64,
    pub oracle_stops: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TimedContinuationClosureReading {
    pub reading: ContinuationClosureReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<ContinuationClosureStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

impl ContinuationClosureMemory {
    pub fn entries(&self) -> usize {
        self.entries.len()
    }

    pub fn census(&self) -> ContinuationClosureCensus {
        let mut census = ContinuationClosureCensus {
            suffixes: self.entries.len() as u64,
            observations: 0,
            continuation_tokens: 0,
            empty_continuations: 0,
            suffixes_by_depth: [0; CONTINUATION_CLOSURE_MAX_DEPTH],
            observations_by_depth: [0; CONTINUATION_CLOSURE_MAX_DEPTH],
            continuation_tokens_by_depth: [0; CONTINUATION_CLOSURE_MAX_DEPTH],
        };
        for (suffix, counts) in &self.entries {
            let depth = suffix.len();
            if !(CONTINUATION_CLOSURE_MIN_DEPTH..=CONTINUATION_CLOSURE_MAX_DEPTH).contains(&depth) {
                continue;
            }
            let observations = counts.observations();
            let tokens = counts
                .counts
                .iter()
                .map(|(continuation, count)| continuation.len() as u64 * count)
                .sum::<u64>();
            census.suffixes_by_depth[depth - 1] += 1;
            census.observations_by_depth[depth - 1] += observations;
            census.continuation_tokens_by_depth[depth - 1] += tokens;
            census.observations += observations;
            census.continuation_tokens += tokens;
            census.empty_continuations += counts.counts.get(&Vec::new()).copied().unwrap_or(0);
        }
        census
    }

    pub fn predict_confident(
        &self,
        stream: &[u32],
        minimum_observations: u64,
    ) -> Option<ContinuationClosurePrediction> {
        if minimum_observations < 2 {
            return None;
        }
        let maximum = stream.len().min(CONTINUATION_CLOSURE_MAX_DEPTH);
        for depth in (CONTINUATION_CLOSURE_MIN_DEPTH..=maximum).rev() {
            let suffix = &stream[stream.len() - depth..];
            let Some(counts) = self.entries.get(suffix) else {
                continue;
            };
            let observations = counts.observations();
            let (continuation, continuation_count) = counts.prediction()?;
            if observations >= minimum_observations
                && continuation_count.saturating_mul(2) > observations
            {
                return Some(ContinuationClosurePrediction {
                    depth,
                    observations,
                    continuation_count,
                    distinct_continuations: counts.counts.len() as u64,
                    continuation: continuation.to_vec(),
                });
            }
        }
        None
    }
}

pub fn derive_continuation_closure(
    samples: &[MemoryWeightSample],
) -> Result<ContinuationClosureMemory> {
    if samples.is_empty() {
        return Err(invalid(
            "continuation-closure derivation needs training samples",
        ));
    }
    let mut entries = BTreeMap::<Vec<u32>, ContinuationCounts>::new();
    for sample in samples {
        let reply = normalized_reference(&sample.reply);
        if sample.prompt.is_empty() || reply.is_empty() {
            return Err(invalid(
                "continuation-closure derivation saw an empty prompt or reply",
            ));
        }
        let mut history = sample.prompt.clone();
        history.extend_from_slice(reply);
        for end in sample.prompt.len()..=history.len() {
            let continuation = history[end..].to_vec();
            let maximum = end.min(CONTINUATION_CLOSURE_MAX_DEPTH);
            for depth in CONTINUATION_CLOSURE_MIN_DEPTH..=maximum {
                let count = entries
                    .entry(history[end - depth..end].to_vec())
                    .or_default()
                    .counts
                    .entry(continuation.clone())
                    .or_default();
                *count = count
                    .checked_add(1)
                    .ok_or_else(|| invalid("continuation-closure count overflows"))?;
            }
        }
    }
    if entries.is_empty() {
        return Err(invalid(
            "continuation-closure derivation found no eligible suffixes",
        ));
    }
    Ok(ContinuationClosureMemory { entries })
}

pub fn shuffle_continuation_closure(
    original: &ContinuationClosureMemory,
    seed: u64,
) -> Result<(ContinuationClosureMemory, u64)> {
    for attempt in 0..64u64 {
        let mut shuffled = original.clone();
        for depth in CONTINUATION_CLOSURE_MIN_DEPTH..=CONTINUATION_CLOSURE_MAX_DEPTH {
            let keys = shuffled
                .entries
                .iter()
                .filter(|(suffix, _)| suffix.len() == depth)
                .map(|(suffix, counts)| (suffix.clone(), counts.observations()))
                .collect::<Vec<_>>();
            if keys.is_empty() {
                continue;
            }
            let mut continuations = Vec::new();
            for (suffix, observations) in &keys {
                let counts = shuffled.entries.get(suffix).expect("suffix exists");
                if counts.observations() != *observations {
                    return Err(invalid("continuation-closure shuffle support differs"));
                }
                for (continuation, count) in &counts.counts {
                    for _ in 0..*count {
                        continuations.push(continuation.clone());
                    }
                }
            }
            for index in (1..continuations.len()).rev() {
                let draw = splitmix64(
                    seed ^ (attempt << 48)
                        ^ (depth as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                        ^ (index as u64).wrapping_mul(0xd1b5_4a32_d192_ed03),
                ) as usize
                    % (index + 1);
                continuations.swap(index, draw);
            }
            let mut cursor = 0usize;
            for (suffix, observations) in keys {
                let count = usize::try_from(observations)
                    .map_err(|_| invalid("continuation-closure observation count is too large"))?;
                let counts = shuffled.entries.get_mut(&suffix).expect("suffix exists");
                counts.counts.clear();
                for continuation in &continuations[cursor..cursor + count] {
                    let value = counts.counts.entry(continuation.clone()).or_default();
                    *value = value
                        .checked_add(1)
                        .ok_or_else(|| invalid("continuation-closure shuffle count overflows"))?;
                }
                cursor += count;
            }
            if cursor != continuations.len() {
                return Err(invalid("continuation-closure shuffle assignment differs"));
            }
        }
        let changed = shuffled
            .entries
            .iter()
            .filter(|(suffix, counts)| {
                original
                    .entries
                    .get(*suffix)
                    .is_some_and(|old| old.counts != counts.counts)
            })
            .count() as u64;
        if changed > 0 {
            return Ok((shuffled, changed));
        }
    }
    Err(invalid(
        "continuation-closure shuffle could not change a continuation",
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn generate_continuation_closure_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    reference_length: usize,
    arm: ContinuationClosureArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    closure_memory: Option<&ContinuationClosureMemory>,
    reply_boundary: &ReplyBoundaryMemory,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<ContinuationClosureStep>)> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("continuation-closure forward: {error}")))?;
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "continuation-closure prompt, cap or artifact context is invalid",
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
            .map_err(|error| invalid(format!("continuation-closure prompt step: {error}")))?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some(step.token);
        }
    }
    let mut writer_token = next.ok_or_else(|| invalid("continuation-closure prompt is empty"))?;
    let mut tokens = Vec::with_capacity(cap);
    let mut steps = Vec::with_capacity(cap);

    loop {
        if arm == ContinuationClosureArm::ContentReferenceOracle && tokens.len() == reference_length
        {
            steps.push(step_without_source(
                row,
                tokens.len(),
                writer_token,
                POINT_KEY_BOS,
                "oracle",
            ));
            return Ok((
                GeneratedReply {
                    tokens,
                    stop: "bos",
                },
                steps,
            ));
        }

        let boundary_hit = if matches!(
            arm,
            ContinuationClosureArm::TokenwiseContent
                | ContinuationClosureArm::LocalReplyBoundary
                | ContinuationClosureArm::WriterBoundary
        ) {
            reply_boundary.stop_decision(&tokens)
        } else {
            None
        };
        if boundary_hit.is_some() {
            steps.push(step_without_source(
                row,
                tokens.len(),
                writer_token,
                POINT_KEY_BOS,
                "reply",
            ));
            return Ok((
                GeneratedReply {
                    tokens,
                    stop: "bos",
                },
                steps,
            ));
        }

        let use_content = matches!(
            arm,
            ContinuationClosureArm::SourceClosure
                | ContinuationClosureArm::TokenwiseContent
                | ContinuationClosureArm::SourceClosureShuffled
                | ContinuationClosureArm::ContentNoBoundary
                | ContinuationClosureArm::ContentReferenceOracle
        );
        let use_local = use_content || matches!(arm, ContinuationClosureArm::LocalReplyBoundary);
        let local = use_local.then(|| {
            copy_key_reply_decision(
                &stream,
                writer_token,
                CopyKeyReplyArm::Longest32To1,
                seed,
                row,
                tokens.len(),
            )
        });
        let local_depth = local.as_ref().and_then(|decision| decision.depth);
        let global = if use_content {
            suffix_memory
                .ok_or_else(|| invalid("content arm has no suffix table"))?
                .predict_confident(
                    &stream,
                    SUFFIX_MEMORY_MAX_DEPTH,
                    CONTINUATION_CLOSURE_MIN_OBSERVATIONS,
                )
        } else {
            None
        };
        let used_global = use_content
            && global.is_some_and(|prediction| {
                local_depth.is_none_or(|local_depth| prediction.depth > local_depth)
            });
        let source_token = if matches!(
            arm,
            ContinuationClosureArm::ClosureOnly | ContinuationClosureArm::WriterBoundary
        ) {
            writer_token
        } else if used_global {
            global.expect("used_global requires global").token
        } else if local_depth.is_some() {
            local.as_ref().expect("local depth requires local").token
        } else {
            writer_token
        };

        let closure = if matches!(
            arm,
            ContinuationClosureArm::SourceClosure
                | ContinuationClosureArm::SourceClosureShuffled
                | ContinuationClosureArm::ClosureOnly
        ) {
            closure_memory
                .ok_or_else(|| invalid("closure arm has no closure table"))?
                .predict_confident(&stream, CONTINUATION_CLOSURE_MIN_OBSERVATIONS)
                .filter(|closure| {
                    closure_eligible(
                        arm,
                        used_global,
                        global.map(|prediction| prediction.depth),
                        global.map(|prediction| prediction.token),
                        closure.depth,
                        &closure.continuation,
                    )
                })
        } else {
            None
        };
        if let Some(prediction) = closure {
            let total = tokens
                .len()
                .checked_add(prediction.continuation.len())
                .ok_or_else(|| invalid("continuation-closure length overflows"))?;
            if total < cap && prompt.len().checked_add(total).is_some_and(|n| n < context) {
                let selected_token = prediction
                    .continuation
                    .first()
                    .copied()
                    .unwrap_or(POINT_KEY_BOS);
                steps.push(ContinuationClosureStep {
                    row,
                    generated_step: tokens.len(),
                    writer_token,
                    local_token: local.as_ref().map(|decision| decision.token),
                    global_token: global.map(|prediction| prediction.token),
                    source_token,
                    selected_token,
                    used_global,
                    local_copy_depth: local_depth,
                    global_depth: global.map(|prediction| prediction.depth),
                    closure_depth: Some(prediction.depth),
                    closure_observations: prediction.observations,
                    closure_count: prediction.continuation_count,
                    closure_distinct: prediction.distinct_continuations,
                    closure_tokens: prediction.continuation.len() as u64,
                    continuation_sha256: token_hash(&prediction.continuation),
                    stop_kind: Some("closure".to_owned()),
                });
                tokens.extend_from_slice(&prediction.continuation);
                return Ok((
                    GeneratedReply {
                        tokens,
                        stop: "bos",
                    },
                    steps,
                ));
            }
        }

        steps.push(ContinuationClosureStep {
            row,
            generated_step: tokens.len(),
            writer_token,
            local_token: local.as_ref().map(|decision| decision.token),
            global_token: global.map(|prediction| prediction.token),
            source_token,
            selected_token: source_token,
            used_global,
            local_copy_depth: local_depth,
            global_depth: global.map(|prediction| prediction.depth),
            closure_depth: None,
            closure_observations: 0,
            closure_count: 0,
            closure_distinct: 0,
            closure_tokens: 0,
            continuation_sha256: String::new(),
            stop_kind: None,
        });
        let stop = reply_stop(source_token, tokens.len(), prompt.len(), cap, context);
        if stop != Some(ReplyEnd::Bos) {
            tokens.push(source_token);
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
        stream.push(source_token);
        let step = forward
            .step(source_token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("continuation-closure generated step: {error}")))?;
        writer_token = step.token;
    }
}

fn closure_eligible(
    arm: ContinuationClosureArm,
    used_global: bool,
    content_depth: Option<usize>,
    content_token: Option<u32>,
    closure_depth: usize,
    continuation: &[u32],
) -> bool {
    match arm {
        ContinuationClosureArm::SourceClosure | ContinuationClosureArm::SourceClosureShuffled => {
            used_global
                && content_depth == Some(closure_depth)
                && continuation
                    .first()
                    .is_none_or(|token| content_token == Some(*token))
        }
        ContinuationClosureArm::ClosureOnly => true,
        ContinuationClosureArm::TokenwiseContent
        | ContinuationClosureArm::ContentNoBoundary
        | ContinuationClosureArm::LocalReplyBoundary
        | ContinuationClosureArm::WriterBoundary
        | ContinuationClosureArm::ContentReferenceOracle => false,
    }
}

fn step_without_source(
    row: usize,
    generated_step: usize,
    writer_token: u32,
    selected_token: u32,
    stop_kind: &str,
) -> ContinuationClosureStep {
    ContinuationClosureStep {
        row,
        generated_step,
        writer_token,
        local_token: None,
        global_token: None,
        source_token: selected_token,
        selected_token,
        used_global: false,
        local_copy_depth: None,
        global_depth: None,
        closure_depth: None,
        closure_observations: 0,
        closure_count: 0,
        closure_distinct: 0,
        closure_tokens: 0,
        continuation_sha256: String::new(),
        stop_kind: Some(stop_kind.to_owned()),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_continuation_closure_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: ContinuationClosureArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    closure_memory: Option<&ContinuationClosureMemory>,
    reply_boundary: &ReplyBoundaryMemory,
    seed: u64,
) -> Result<TimedContinuationClosureReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("continuation-closure reply inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("continuation-closure context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply).to_vec();
            if sample.prompt.len() > context || reference.len() > cap {
                return Err(invalid(
                    "continuation-closure sample exceeds its context or cap",
                ));
            }
            let (generated, steps) = generate_continuation_closure_reply(
                artifact,
                &sample.prompt,
                cap,
                reference.len(),
                arm,
                suffix_memory,
                closure_memory,
                reply_boundary,
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
            let premature_bos = generated.stop == "bos" && generated.tokens.len() < reference.len();
            let non_premature_bos = generated.stop == "bos" && !premature_bos;
            let copy_decisions = steps
                .iter()
                .filter(|step| {
                    step.closure_depth.is_none()
                        && !step.used_global
                        && step.local_copy_depth.is_some()
                })
                .count() as u64;
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
                copy_decisions,
                fallback_decisions: steps.len() as u64 - copy_decisions,
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
    Ok(TimedContinuationClosureReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

pub fn compare_continuation_closure_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<crate::copy_key_reply::CopyKeyReplyComparison> {
    compare_copy_key_reply_rows(candidate, control, metric)
}

pub fn continuation_closure_changes(
    candidate: &TimedContinuationClosureReading,
    control: &TimedContinuationClosureReading,
) -> u64 {
    let candidate_reachable = candidate
        .rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .collect::<BTreeSet<_>>();
    let reachable_rows = control
        .rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .filter(|row| candidate_reachable.contains(row))
        .collect::<BTreeSet<_>>();
    let hashes = |reading: &TimedContinuationClosureReading| {
        reading
            .rows
            .iter()
            .filter(|row| reachable_rows.contains(&row.row))
            .map(|row| (row.row, (row.reply_sha256.clone(), row.stop.clone())))
            .collect::<BTreeMap<_, _>>()
    };
    let candidate = hashes(candidate);
    let control = hashes(control);
    candidate
        .iter()
        .filter(|(row, value)| control.get(row).is_some_and(|other| other != *value))
        .count() as u64
}

pub fn changed_closure_rows(
    candidate: &TimedContinuationClosureReading,
    controls: &[&TimedContinuationClosureReading],
) -> u64 {
    let closure_rows = candidate
        .steps
        .iter()
        .filter(|step| step.stop_kind.as_deref() == Some("closure"))
        .map(|step| step.row)
        .collect::<BTreeSet<_>>();
    let candidate_rows = candidate
        .rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| (row.row, row))
        .collect::<BTreeMap<_, _>>();
    let control_rows = controls
        .iter()
        .map(|control| {
            control
                .rows
                .iter()
                .filter(|row| row.reachable)
                .map(|row| (row.row, row))
                .collect::<BTreeMap<_, _>>()
        })
        .collect::<Vec<_>>();
    closure_rows
        .into_iter()
        .filter(|row| {
            candidate_rows.get(row).is_some_and(|candidate| {
                control_rows.iter().any(|control| {
                    control
                        .get(row)
                        .is_none_or(|control| !reply_matches(candidate, control))
                })
            })
        })
        .count() as u64
}

fn reply_matches(left: &CopyKeyReplyRow, right: &CopyKeyReplyRow) -> bool {
    left.reply_sha256 == right.reply_sha256 && left.stop == right.stop
}

fn summarize(
    arm: ContinuationClosureArm,
    rows: &[CopyKeyReplyRow],
    steps: &[ContinuationClosureStep],
) -> Result<ContinuationClosureReading> {
    let mut reading = ContinuationClosureReading {
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
        local_decisions: 0,
        global_decisions: 0,
        writer_decisions: 0,
        closure_decisions: 0,
        closure_changes: 0,
        closure_tokens: 0,
        reply_boundary_stops: 0,
        oracle_stops: 0,
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
        match step.stop_kind.as_deref() {
            Some("closure") => {
                reading.closure_decisions = checked_add(reading.closure_decisions, 1)?;
                reading.closure_changes = checked_add(
                    reading.closure_changes,
                    u64::from(step.selected_token != step.source_token),
                )?;
                reading.closure_tokens = checked_add(reading.closure_tokens, step.closure_tokens)?;
            }
            Some("reply") => {
                reading.reply_boundary_stops = checked_add(reading.reply_boundary_stops, 1)?;
            }
            Some("oracle") => {
                reading.oracle_stops = checked_add(reading.oracle_stops, 1)?;
            }
            Some(_) => return Err(invalid("unknown continuation-closure stop kind")),
            None if step.used_global => {
                reading.global_decisions = checked_add(reading.global_decisions, 1)?;
            }
            None if step.local_copy_depth.is_some() => {
                reading.local_decisions = checked_add(reading.local_decisions, 1)?;
            }
            None => {
                reading.writer_decisions = checked_add(reading.writer_decisions, 1)?;
            }
        }
    }
    let accounted = reading
        .local_decisions
        .checked_add(reading.global_decisions)
        .and_then(|count| count.checked_add(reading.writer_decisions))
        .and_then(|count| count.checked_add(reading.closure_decisions))
        .and_then(|count| count.checked_add(reading.reply_boundary_stops))
        .and_then(|count| count.checked_add(reading.oracle_stops))
        .ok_or_else(|| invalid("continuation-closure accounting overflows"))?;
    let reachable_decisions = steps
        .iter()
        .filter(|step| reachable_rows.contains(&step.row))
        .count() as u64;
    if accounted != reachable_decisions {
        return Err(invalid(format!(
            "continuation-closure accounting differs: {accounted} != {reachable_decisions}"
        )));
    }
    Ok(reading)
}

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid("continuation-closure counter overflows"))
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
        hasher.update(row.stop.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn decision_hash(steps: &[ContinuationClosureStep]) -> String {
    let mut hasher = blake3::Hasher::new();
    for step in steps {
        hasher.update(&(step.row as u64).to_le_bytes());
        hasher.update(&(step.generated_step as u64).to_le_bytes());
        hasher.update(&step.writer_token.to_le_bytes());
        hasher.update(&step.local_token.unwrap_or(u32::MAX).to_le_bytes());
        hasher.update(&step.global_token.unwrap_or(u32::MAX).to_le_bytes());
        hasher.update(&step.source_token.to_le_bytes());
        hasher.update(&step.selected_token.to_le_bytes());
        hasher.update(&[u8::from(step.used_global)]);
        hasher.update(&(step.local_copy_depth.unwrap_or(0) as u16).to_le_bytes());
        hasher.update(&(step.global_depth.unwrap_or(0) as u16).to_le_bytes());
        hasher.update(&(step.closure_depth.unwrap_or(0) as u16).to_le_bytes());
        hasher.update(&step.closure_observations.to_le_bytes());
        hasher.update(&step.closure_count.to_le_bytes());
        hasher.update(&step.closure_distinct.to_le_bytes());
        hasher.update(&step.closure_tokens.to_le_bytes());
        hasher.update(step.continuation_sha256.as_bytes());
        hasher.update(step.stop_kind.as_deref().unwrap_or_default().as_bytes());
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
            202,
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

    #[test]
    fn closure_requires_strict_majority_of_complete_continuations() {
        let memory = derive_continuation_closure(&[
            sample(&[1, 2, 3, 4], &[10, 11, 12]),
            sample(&[1, 2, 3, 4], &[10, 11, 12]),
            sample(&[1, 2, 3, 4], &[10, 11, 13]),
        ])
        .unwrap();
        let prediction = memory.predict_confident(&[1, 2, 3, 4], 2).unwrap();
        assert_eq!(prediction.depth, 4);
        assert_eq!(prediction.observations, 3);
        assert_eq!(prediction.continuation_count, 2);
        assert_eq!(prediction.continuation, vec![10, 11, 12]);

        let tied = derive_continuation_closure(&[
            sample(&[1, 2, 3, 4], &[10, 11, 12]),
            sample(&[1, 2, 3, 4], &[10, 11, 13]),
        ])
        .unwrap();
        assert!(tied.predict_confident(&[1, 2, 3, 4], 2).is_none());
    }

    #[test]
    fn closure_contains_the_natural_empty_endpoint() {
        let prompt = [1, 2, 3, 4];
        let reply = [10, 11, 12, 13];
        let memory = derive_continuation_closure(&[sample(&prompt, &reply)]).unwrap();
        assert_eq!(
            memory.predict_confident(&[1, 2, 3, 4, 10, 11, 12, 13], 2),
            None,
            "one observation does not meet the confidence floor"
        );
        let doubled =
            derive_continuation_closure(&[sample(&prompt, &reply), sample(&prompt, &reply)])
                .unwrap();
        let prediction = doubled
            .predict_confident(&[1, 2, 3, 4, 10, 11, 12, 13], 2)
            .unwrap();
        assert!(prediction.continuation.is_empty());
    }

    #[test]
    fn shuffle_preserves_each_keys_observation_count_and_depth_tokens() {
        let memory = derive_continuation_closure(&[
            sample(&[1, 2, 3, 4], &[10, 11, 12]),
            sample(&[5, 6, 7, 8], &[20, 21, 22, 23]),
        ])
        .unwrap();
        let (shuffled, changed) = shuffle_continuation_closure(&memory, 202).unwrap();
        assert!(changed > 0);
        for (suffix, counts) in &memory.entries {
            assert_eq!(
                counts.observations(),
                shuffled.entries[suffix].observations()
            );
        }
        let before = memory.census();
        let after = shuffled.census();
        assert_eq!(before.continuation_tokens, after.continuation_tokens);
        assert_eq!(
            before.continuation_tokens_by_depth,
            after.continuation_tokens_by_depth
        );
    }

    #[test]
    fn source_closure_emits_the_confident_remaining_reply_and_stops() {
        let training = [
            sample(&[1, 2, 3, 4], &[10, 11, 12, 13]),
            sample(&[1, 2, 3, 4], &[10, 11, 12, 13]),
        ];
        let content = derive_full_sequence_suffix_content_memory(&training).unwrap();
        let closure = derive_continuation_closure(&training).unwrap();
        let boundary = derive_reply_boundary_memory(&training).unwrap();
        let (reply, steps) = generate_continuation_closure_reply(
            &artifact(),
            &[1, 2, 3, 4],
            64,
            4,
            ContinuationClosureArm::SourceClosure,
            Some(&content),
            Some(&closure),
            &boundary,
            202,
            0,
        )
        .unwrap();
        assert_eq!(reply.tokens, [10, 11, 12, 13]);
        assert_eq!(reply.stop, "bos");
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].stop_kind.as_deref(), Some("closure"));
        assert_eq!(steps[0].closure_tokens, 4);
        assert_eq!(steps[0].continuation_sha256, token_hash(&[10, 11, 12, 13]));
    }

    #[test]
    fn source_closure_requires_matching_content_depth() {
        assert!(closure_eligible(
            ContinuationClosureArm::SourceClosure,
            true,
            Some(12),
            Some(10),
            12,
            &[10, 11],
        ));
        assert!(!closure_eligible(
            ContinuationClosureArm::SourceClosure,
            true,
            Some(16),
            Some(10),
            12,
            &[10, 11],
        ));
        assert!(!closure_eligible(
            ContinuationClosureArm::SourceClosure,
            false,
            Some(12),
            Some(10),
            12,
            &[10, 11],
        ));
        assert!(!closure_eligible(
            ContinuationClosureArm::SourceClosure,
            true,
            Some(12),
            Some(9),
            12,
            &[10, 11],
        ));
        assert!(closure_eligible(
            ContinuationClosureArm::SourceClosure,
            true,
            Some(12),
            Some(9),
            12,
            &[],
        ));
        assert!(closure_eligible(
            ContinuationClosureArm::ClosureOnly,
            false,
            None,
            None,
            12,
            &[10, 11],
        ));
        assert!(!closure_eligible(
            ContinuationClosureArm::ContentNoBoundary,
            true,
            Some(12),
            Some(10),
            12,
            &[10, 11],
        ));
    }

    #[test]
    fn closure_longer_than_cap_falls_back_tokenwise() {
        let training = [
            sample(&[1, 2, 3, 4], &[10, 11, 12, 13]),
            sample(&[1, 2, 3, 4], &[10, 11, 12, 13]),
        ];
        let content = derive_full_sequence_suffix_content_memory(&training).unwrap();
        let closure = derive_continuation_closure(&training).unwrap();
        let boundary = derive_reply_boundary_memory(&training).unwrap();
        let (reply, steps) = generate_continuation_closure_reply(
            &artifact(),
            &[1, 2, 3, 4],
            3,
            4,
            ContinuationClosureArm::SourceClosure,
            Some(&content),
            Some(&closure),
            &boundary,
            202,
            0,
        )
        .unwrap();
        assert_eq!(reply.stop, "cap");
        assert!(steps
            .iter()
            .all(|step| step.stop_kind.as_deref() != Some("closure")));
    }

    #[test]
    fn closure_beyond_context_falls_back_tokenwise() {
        let prompt = (1..=30).collect::<Vec<_>>();
        let reply = [100, 101, 102, 103];
        let training = [sample(&prompt, &reply), sample(&prompt, &reply)];
        let content = derive_full_sequence_suffix_content_memory(&training).unwrap();
        let closure = derive_continuation_closure(&training).unwrap();
        let boundary = derive_reply_boundary_memory(&training).unwrap();
        let (reply, steps) = generate_continuation_closure_reply(
            &artifact(),
            &prompt,
            64,
            reply.len(),
            ContinuationClosureArm::SourceClosure,
            Some(&content),
            Some(&closure),
            &boundary,
            202,
            0,
        )
        .unwrap();
        assert_ne!(reply.stop, "bos");
        assert!(steps
            .iter()
            .all(|step| step.stop_kind.as_deref() != Some("closure")));
    }

    #[test]
    fn closure_change_accounting_compares_the_preclosure_source_token() {
        let row = CopyKeyReplyRow {
            row: 0,
            reachable: true,
            exact_reference: false,
            bos_complete: true,
            opening_correct: false,
            premature_bos: false,
            non_premature_bos: true,
            generated_tokens: 4,
            reference_tokens: 4,
            correct_body_positions: 0,
            copy_decisions: 0,
            fallback_decisions: 1,
            stop: "bos".to_owned(),
            reply_sha256: String::new(),
        };
        let step = ContinuationClosureStep {
            row: 0,
            generated_step: 0,
            writer_token: 9,
            local_token: Some(5),
            global_token: Some(5),
            source_token: 5,
            selected_token: 10,
            used_global: true,
            local_copy_depth: None,
            global_depth: Some(8),
            closure_depth: Some(8),
            closure_observations: 2,
            closure_count: 2,
            closure_distinct: 1,
            closure_tokens: 4,
            continuation_sha256: token_hash(&[10, 11, 12, 13]),
            stop_kind: Some("closure".to_owned()),
        };
        let reading = summarize(ContinuationClosureArm::SourceClosure, &[row], &[step]).unwrap();
        assert_eq!(reading.closure_decisions, 1);
        assert_eq!(reading.closure_changes, 1);
    }

    #[test]
    fn reply_identity_includes_the_stop_marker() {
        let base = CopyKeyReplyRow {
            row: 0,
            reachable: true,
            exact_reference: false,
            bos_complete: true,
            opening_correct: false,
            premature_bos: false,
            non_premature_bos: true,
            generated_tokens: 1,
            reference_tokens: 2,
            correct_body_positions: 1,
            copy_decisions: 0,
            fallback_decisions: 1,
            stop: "bos".to_owned(),
            reply_sha256: "same".to_owned(),
        };
        let mut capped = base.clone();
        capped.stop = "cap".to_owned();
        assert!(!reply_matches(&base, &capped));
    }
}
