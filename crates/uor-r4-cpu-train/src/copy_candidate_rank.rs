//! Pairwise integer ranking over exact-copy candidates.
//!
//! This is a bounded D84 CPU-development screen. The fitter sees the exact
//! served query and exact-copy candidate occurrences, then learns a small
//! integer linear score. Inference never leaves the exact-copy candidate set.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uor_r4_integer::stack::{reply_stop, ReplyEnd};

use crate::copy_key_reply::{
    compare_copy_key_reply_rows, copy_key_reply_decision, exact_suffix_candidates, CopyKeyReplyArm,
    CopyKeyReplyMetric, CopyKeyReplyRow, COPY_KEY_REPLY_MAX_DEPTH,
};
use crate::reply_eval::{normalized_reference, GeneratedReply};
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const COPY_RANKER_FEATURES: usize = 7;
pub const COPY_RANKER_MAX_DEPTH: usize = 8;
pub const COPY_RANKER_EPOCHS: usize = 8;
pub const COPY_RANKER_MIN_GROUPS: usize = 640;
const LEARNING_RATE: i64 = 4;
const WEIGHT_LIMIT: i64 = 4_096;

pub const COPY_RANKER_FEATURE_NAMES: [&str; COPY_RANKER_FEATURES] = [
    "bias",
    "depth",
    "sign_agreement",
    "rotated_sign_agreement",
    "recency",
    "frequency",
    "writer_match",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CopyRankerTrajectory {
    Longest32,
    ReferencePrefix,
}

impl CopyRankerTrajectory {
    pub fn name(self) -> &'static str {
        match self {
            Self::Longest32 => "longest_32",
            Self::ReferencePrefix => "reference_prefix",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCandidateRanker {
    pub weights: [i64; COPY_RANKER_FEATURES],
    pub epochs: usize,
    pub learning_rate: i64,
    pub weight_limit: i64,
}

impl CopyCandidateRanker {
    fn zero() -> Self {
        Self {
            weights: [0; COPY_RANKER_FEATURES],
            epochs: COPY_RANKER_EPOCHS,
            learning_rate: LEARNING_RATE,
            weight_limit: WEIGHT_LIMIT,
        }
    }

    fn depth_only() -> Self {
        let mut ranker = Self::zero();
        ranker.weights[1] = 1;
        ranker
    }

    fn score(&self, features: &[i32; COPY_RANKER_FEATURES]) -> i64 {
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct RankedCandidate {
    token: u32,
    depth: usize,
    features: [i32; COPY_RANKER_FEATURES],
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CandidateGroup {
    row: usize,
    generated_step: usize,
    target: u32,
    candidates: Vec<RankedCandidate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCandidateRankTraining {
    pub trajectory: CopyRankerTrajectory,
    pub trained: CopyCandidateRanker,
    pub shuffled: CopyCandidateRanker,
    pub depth_only: CopyCandidateRanker,
    pub generated_rows: u64,
    pub supervised_groups: u64,
    pub candidate_rows: u64,
    pub target_present_groups: u64,
    pub shuffled_target_changes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RankedCopyStep {
    pub row: usize,
    pub generated_step: usize,
    pub writer_token: u32,
    pub longest_token: u32,
    pub selected_token: u32,
    pub selected_depth: Option<usize>,
    pub used_copy: bool,
    pub candidate_count: usize,
    pub selected_score: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RankedCopyReading {
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
    pub copy_decisions: u64,
    pub fallback_decisions: u64,
    pub selected_depth_counts: [u64; COPY_RANKER_MAX_DEPTH],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedRankedCopyReading {
    pub reading: RankedCopyReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<RankedCopyStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

pub fn derive_copy_candidate_rankers(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    seed: u64,
    trajectory: CopyRankerTrajectory,
) -> Result<CopyCandidateRankTraining> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("copy-candidate ranker training inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("copy-candidate ranker context: {error}")))?
        .context_length();
    let mut groups = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            training_groups_for_sample(artifact, sample, cap, context, row, trajectory)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    groups.sort_by_key(|group| (group.row, group.generated_step));
    if groups.len() < COPY_RANKER_MIN_GROUPS {
        return Err(invalid(format!(
            "copy-candidate ranker has {} supervised groups, fewer than {COPY_RANKER_MIN_GROUPS}",
            groups.len()
        )));
    }
    let generated_rows = groups
        .iter()
        .map(|group| group.row)
        .collect::<BTreeSet<_>>()
        .len();
    let candidate_rows = groups
        .iter()
        .map(|group| group.candidates.len() as u64)
        .sum();
    let trained = fit_ranker(&groups, seed, false)?;
    let shuffled = fit_ranker(&groups, seed, true)?;
    let shuffled_target_changes = shuffled_target_count(&groups, seed);
    if shuffled_target_changes == 0 {
        return Err(invalid("copy-candidate target shuffle is vacuous"));
    }
    Ok(CopyCandidateRankTraining {
        trajectory,
        trained,
        shuffled,
        depth_only: CopyCandidateRanker::depth_only(),
        generated_rows: generated_rows as u64,
        supervised_groups: groups.len() as u64,
        candidate_rows,
        target_present_groups: groups.len() as u64,
        shuffled_target_changes,
    })
}

pub fn evaluate_ranked_copy_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    ranker: &CopyCandidateRanker,
) -> Result<TimedRankedCopyReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("ranked-copy reply inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("ranked-copy context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply).to_vec();
            if sample.prompt.len() > context || reference.len() > cap {
                return Err(invalid("ranked-copy sample exceeds its context or cap"));
            }
            let (generated, steps) =
                generate_ranked_reply(artifact, &sample.prompt, cap, ranker, row)?;
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
            let copy_decisions = steps.iter().filter(|step| step.used_copy).count() as u64;
            let fallback_decisions = steps.len() as u64 - copy_decisions;
            let premature_bos = generated.stop == "bos" && generated.tokens.len() < reference.len();
            let non_premature_bos = generated.stop == "bos" && !premature_bos;
            let row = CopyKeyReplyRow {
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
                fallback_decisions,
                stop: generated.stop.to_owned(),
                reply_sha256: token_hash(&generated.tokens),
            };
            Ok((row, steps))
        })
        .collect::<Result<Vec<_>>>()?;
    let wall_elapsed_us = started.elapsed().as_micros().try_into().unwrap_or(u64::MAX);
    let (rows, steps): (Vec<_>, Vec<_>) = evaluated.into_iter().unzip();
    let steps = steps.into_iter().flatten().collect::<Vec<_>>();
    let reading = summarize(&rows, &steps)?;
    let reply_sha256 = reply_hash(&rows);
    let decision_sha256 = decision_hash(&steps);
    let generated_tokens_per_second = if wall_elapsed_us == 0 {
        0.0
    } else {
        reading.generated_tokens as f64 * 1_000_000.0 / wall_elapsed_us as f64
    };
    Ok(TimedRankedCopyReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

pub fn compare_ranked_copy_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<crate::CopyKeyReplyComparison> {
    compare_copy_key_reply_rows(candidate, control, metric)
}

fn training_groups_for_sample(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    cap: usize,
    context: usize,
    row: usize,
    trajectory: CopyRankerTrajectory,
) -> Result<Vec<CandidateGroup>> {
    match trajectory {
        CopyRankerTrajectory::Longest32 => {
            longest_training_groups_for_sample(artifact, sample, cap, context, row)
        }
        CopyRankerTrajectory::ReferencePrefix => {
            reference_training_groups_for_sample(artifact, sample, cap, context, row)
        }
    }
}

fn longest_training_groups_for_sample(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    cap: usize,
    context: usize,
    row: usize,
) -> Result<Vec<CandidateGroup>> {
    let reference = normalized_reference(&sample.reply).to_vec();
    if sample.prompt.is_empty() || sample.prompt.len() > context || reference.len() > cap {
        return Err(invalid(
            "copy-candidate ranker training sample exceeds its context or cap",
        ));
    }
    let (mut forward, mut stream, writer, query) =
        start_reply(artifact, &sample.prompt, cap, context, "training")?;
    let mut generator = (writer, query);
    let mut generated = 0usize;
    let mut groups = Vec::new();
    loop {
        let base = copy_key_reply_decision(
            &stream,
            generator.0,
            CopyKeyReplyArm::Longest32To1,
            0,
            row,
            generated,
        );
        let candidates = ranked_candidates(&forward, &generator.1, &stream, generator.0)?;
        if candidates.len() >= 2 {
            if let Some(target) = reference.get(generated).copied() {
                if candidates.iter().any(|candidate| candidate.token == target) {
                    groups.push(CandidateGroup {
                        row,
                        generated_step: generated,
                        target,
                        candidates,
                    });
                }
            }
        }
        let stop = reply_stop(base.token, generated, sample.prompt.len(), cap, context);
        if stop.is_some() {
            break;
        }
        stream.push(base.token);
        let step = forward
            .step(base.token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("copy-candidate ranker training step: {error}")))?;
        generator = (step.token, step.query_q12);
        generated += 1;
    }
    Ok(groups)
}

fn reference_training_groups_for_sample(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    cap: usize,
    context: usize,
    row: usize,
) -> Result<Vec<CandidateGroup>> {
    let reference = normalized_reference(&sample.reply).to_vec();
    if sample.prompt.is_empty() || sample.prompt.len() > context || reference.len() > cap {
        return Err(invalid(
            "copy-candidate reference training sample exceeds its context or cap",
        ));
    }
    let (mut forward, mut stream, writer, query) =
        start_reply(artifact, &sample.prompt, cap, context, "reference training")?;
    let mut generator = (writer, query);
    let mut generated = 0usize;
    let mut groups = Vec::new();
    while generated < reference.len()
        && generated < cap
        && sample.prompt.len().saturating_add(generated) < context
    {
        let target = reference[generated];
        let candidates = ranked_candidates(&forward, &generator.1, &stream, generator.0)?;
        if candidates.len() >= 2 && candidates.iter().any(|candidate| candidate.token == target) {
            groups.push(CandidateGroup {
                row,
                generated_step: generated,
                target,
                candidates,
            });
        }
        stream.push(target);
        let step = forward
            .step(target, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("copy-candidate reference training step: {error}")))?;
        generator = (step.token, step.query_q12);
        generated += 1;
    }
    Ok(groups)
}

fn generate_ranked_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    ranker: &CopyCandidateRanker,
    row: usize,
) -> Result<(GeneratedReply, Vec<RankedCopyStep>)> {
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("ranked-copy forward: {error}")))?
        .context_length();
    let (mut forward, mut stream, writer, query) =
        start_reply(artifact, prompt, cap, context, "reply")?;
    let mut generator = (writer, query);
    let mut tokens = Vec::with_capacity(cap);
    let mut steps = Vec::with_capacity(cap);
    loop {
        let base = copy_key_reply_decision(
            &stream,
            generator.0,
            CopyKeyReplyArm::Longest32To1,
            0,
            row,
            tokens.len(),
        );
        let candidates = ranked_candidates(&forward, &generator.1, &stream, generator.0)?;
        let (selected, depth, score) = choose_candidate(&candidates, ranker)
            .map(|candidate| {
                (
                    candidate.token,
                    Some(candidate.depth),
                    Some(ranker.score(&candidate.features)),
                )
            })
            .unwrap_or((base.token, base.depth, None));
        steps.push(RankedCopyStep {
            row,
            generated_step: tokens.len(),
            writer_token: generator.0,
            longest_token: base.token,
            selected_token: selected,
            selected_depth: depth,
            used_copy: !candidates.is_empty(),
            candidate_count: candidates.len(),
            selected_score: score,
        });
        let stop = reply_stop(selected, tokens.len(), prompt.len(), cap, context);
        if stop != Some(ReplyEnd::Bos) {
            tokens.push(selected);
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
        stream.push(selected);
        let step = forward
            .step(selected, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("ranked-copy generated step: {error}")))?;
        generator = (step.token, step.query_q12);
    }
}

fn ranked_candidates(
    forward: &CpuFbsForward,
    query: &[i32],
    stream: &[u32],
    writer: u32,
) -> Result<Vec<RankedCandidate>> {
    let source = exact_suffix_candidates(stream);
    let mut unique = BTreeMap::new();
    for depth in 1..=COPY_RANKER_MAX_DEPTH.min(COPY_KEY_REPLY_MAX_DEPTH) {
        let Some((token, source_position)) = source[depth - 1] else {
            continue;
        };
        if token == 0 {
            continue;
        }
        unique.insert(
            token,
            RankedCandidate {
                token,
                depth,
                features: candidate_features(
                    forward,
                    query,
                    stream,
                    token,
                    depth,
                    source_position,
                    writer,
                )?,
            },
        );
    }
    Ok(unique.into_values().collect())
}

fn candidate_features(
    forward: &CpuFbsForward,
    query: &[i32],
    stream: &[u32],
    token: u32,
    depth: usize,
    source: usize,
    writer: u32,
) -> Result<[i32; COPY_RANKER_FEATURES]> {
    let width = query.len();
    let row = forward
        .token_table()
        .get(token as usize * width..(token as usize + 1) * width)
        .ok_or_else(|| invalid("copy-candidate token row differs"))?;
    let agreement = query
        .iter()
        .zip(row)
        .filter(|(query, table)| (**query >= 0) == (**table >= 0))
        .count();
    let mut rotated = agreement;
    for eighth in 1..8 {
        let shift = width * eighth / 8;
        let count = query
            .iter()
            .enumerate()
            .filter(|(index, query)| {
                let table = row[(*index + shift) % width];
                (**query >= 0) == (table >= 0)
            })
            .count();
        rotated = rotated.max(count);
    }
    let age = stream.len().saturating_sub(source);
    let recency = 32i32.saturating_sub((age / 8).min(32) as i32);
    let frequency = stream
        .iter()
        .filter(|&&candidate| candidate == token)
        .count()
        .min(16) as i32;
    let writer_match = if writer == token { 1 } else { 0 };
    Ok([
        1,
        depth.min(COPY_RANKER_MAX_DEPTH) as i32,
        (agreement * 32 / width) as i32,
        (rotated * 32 / width) as i32,
        recency,
        frequency,
        writer_match,
    ])
}

fn choose_candidate<'a>(
    candidates: &'a [RankedCandidate],
    ranker: &CopyCandidateRanker,
) -> Option<&'a RankedCandidate> {
    candidates
        .iter()
        .min_by_key(|candidate| (Reverse(ranker.score(&candidate.features)), candidate.token))
}

fn fit_ranker(groups: &[CandidateGroup], seed: u64, shuffled: bool) -> Result<CopyCandidateRanker> {
    let mut ranker = CopyCandidateRanker::zero();
    for _ in 0..ranker.epochs {
        for group in groups {
            let target = if shuffled {
                shuffled_target(group, seed)?
            } else {
                group.target
            };
            let Some(chosen) = choose_candidate(&group.candidates, &ranker) else {
                return Err(invalid("copy-candidate training group is empty"));
            };
            if chosen.token == target {
                continue;
            }
            let target_candidate = group
                .candidates
                .iter()
                .find(|candidate| candidate.token == target)
                .ok_or_else(|| invalid("copy-candidate training target differs"))?;
            for index in 0..COPY_RANKER_FEATURES {
                let update = i64::from(target_candidate.features[index] - chosen.features[index])
                    * ranker.learning_rate;
                ranker.weights[index] = (ranker.weights[index] + update)
                    .clamp(-ranker.weight_limit, ranker.weight_limit);
            }
        }
    }
    Ok(ranker)
}

fn shuffled_target(group: &CandidateGroup, seed: u64) -> Result<u32> {
    if group.candidates.len() < 2 {
        return Err(invalid("copy-candidate shuffle needs two candidates"));
    }
    let target_index = group
        .candidates
        .iter()
        .position(|candidate| candidate.token == group.target)
        .ok_or_else(|| invalid("copy-candidate shuffle target differs"))?;
    let shift = 1 + seed as usize % (group.candidates.len() - 1);
    Ok(group.candidates[(target_index + shift) % group.candidates.len()].token)
}

fn shuffled_target_count(groups: &[CandidateGroup], seed: u64) -> u64 {
    groups
        .iter()
        .filter(|group| shuffled_target(group, seed).is_ok_and(|target| target != group.target))
        .count() as u64
}

fn start_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    context: usize,
    label: &str,
) -> Result<(CpuFbsForward, Vec<u32>, u32, Vec<i32>)> {
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(format!(
            "copy-candidate ranker {label} prompt, cap or context is invalid"
        )));
    }
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("copy-candidate ranker {label} forward: {error}")))?;
    let mut stream = Vec::with_capacity(context);
    let mut next = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward.step(token, CpuFbsHead::Writer).map_err(|error| {
            invalid(format!(
                "copy-candidate ranker {label} prompt step: {error}"
            ))
        })?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some((step.token, step.query_q12));
        }
    }
    let (writer, query) =
        next.ok_or_else(|| invalid(format!("copy-candidate ranker {label} prompt is empty")))?;
    Ok((forward, stream, writer, query))
}

fn summarize(rows: &[CopyKeyReplyRow], steps: &[RankedCopyStep]) -> Result<RankedCopyReading> {
    let mut reading = RankedCopyReading {
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
        copy_decisions: 0,
        fallback_decisions: 0,
        selected_depth_counts: [0; COPY_RANKER_MAX_DEPTH],
    };
    for row in rows {
        reading.reference_reachable += u64::from(row.reachable);
        reading.exact_reference += u64::from(row.exact_reference);
        reading.bos_complete += u64::from(row.bos_complete);
        reading.opening_correct += u64::from(row.opening_correct);
        reading.premature_bos += u64::from(row.premature_bos);
        reading.non_premature_bos += u64::from(row.non_premature_bos);
        reading.generated_tokens += row.generated_tokens;
        reading.reference_tokens += row.reference_tokens;
        reading.correct_body_positions += row.correct_body_positions;
    }
    for step in steps {
        reading.copy_decisions += u64::from(step.used_copy);
        reading.fallback_decisions += u64::from(!step.used_copy);
        if let Some(depth) = step.selected_depth {
            let slot = reading
                .selected_depth_counts
                .get_mut(depth - 1)
                .ok_or_else(|| invalid("ranked-copy selected depth differs"))?;
            *slot += 1;
        }
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
            .is_some_and(|length| length < context)
}

fn reply_hash(rows: &[CopyKeyReplyRow]) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(rows.len() as u64).to_le_bytes());
    for row in rows {
        bytes.extend_from_slice(&(row.row as u64).to_le_bytes());
        bytes.extend_from_slice(&row.generated_tokens.to_le_bytes());
        bytes.extend_from_slice(row.reply_sha256.as_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn decision_hash(steps: &[RankedCopyStep]) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(steps.len() as u64).to_le_bytes());
    for step in steps {
        bytes.extend_from_slice(&(step.row as u64).to_le_bytes());
        bytes.extend_from_slice(&(step.generated_step as u64).to_le_bytes());
        bytes.extend_from_slice(&step.writer_token.to_le_bytes());
        bytes.extend_from_slice(&step.longest_token.to_le_bytes());
        bytes.extend_from_slice(&step.selected_token.to_le_bytes());
        bytes.push(step.selected_depth.map_or(0, |depth| depth as u8));
        bytes.extend_from_slice(&step.selected_score.unwrap_or_default().to_le_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn token_hash(tokens: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(tokens.len() * 4);
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    fn artifact() -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            173,
            FbsSynth {
                vocab: crate::VOCABULARY,
                d: 64,
                mlp: 64,
                layers: 1,
                context: 32,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.25,
                decay_step: 0.0625,
            },
        );
        snapshot.memory = None;
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn candidate(token: u32, depth: usize, depth_feature: i32, geometry: i32) -> RankedCandidate {
        RankedCandidate {
            token,
            depth,
            features: [1, depth_feature, geometry, geometry, 4, 2, 0],
        }
    }

    #[test]
    fn pairwise_fit_learns_to_override_deeper_distance_only_choice() {
        let group = CandidateGroup {
            row: 0,
            generated_step: 0,
            target: 7,
            candidates: vec![candidate(3, 8, 8, 0), candidate(7, 1, 1, 32)],
        };
        let trained = fit_ranker(std::slice::from_ref(&group), 3, false).unwrap();
        let depth_only = CopyCandidateRanker::depth_only();
        assert_eq!(
            choose_candidate(&group.candidates, &depth_only)
                .unwrap()
                .token,
            3
        );
        assert_eq!(
            choose_candidate(&group.candidates, &trained).unwrap().token,
            7
        );
        assert_ne!(trained.weights, [0; COPY_RANKER_FEATURES]);
    }

    #[test]
    fn target_shuffle_changes_every_eligible_group() {
        let groups = [
            CandidateGroup {
                row: 0,
                generated_step: 0,
                target: 7,
                candidates: vec![candidate(7, 1, 1, 32), candidate(9, 2, 2, 0)],
            },
            CandidateGroup {
                row: 1,
                generated_step: 3,
                target: 11,
                candidates: vec![
                    candidate(11, 3, 3, 32),
                    candidate(12, 2, 2, 8),
                    candidate(13, 1, 1, 0),
                ],
            },
        ];
        assert_eq!(shuffled_target_count(&groups, 163), 2);
        assert!(groups
            .iter()
            .all(|group| shuffled_target(group, 163).unwrap() != group.target));
    }

    #[test]
    fn candidate_features_are_bounded_and_deterministic() {
        let artifact = artifact();
        let stream = [5, 6, 9, 5, 8, 6, 7, 5, 6];
        let (forward, stream, writer, query) =
            start_reply(&artifact, &stream, 8, 32, "features").unwrap();
        let Some((token, source)) = exact_suffix_candidates(&stream)[1] else {
            panic!("the fixture has a depth-two candidate");
        };
        let first =
            candidate_features(&forward, &query, &stream, token, 2, source, writer).unwrap();
        let repeat =
            candidate_features(&forward, &query, &stream, token, 2, source, writer).unwrap();
        assert_eq!(first, repeat);
        assert_eq!(first[0], 1);
        assert!((1..=COPY_RANKER_MAX_DEPTH as i32).contains(&first[1]));
        assert!((0..=32).contains(&first[2]));
        assert!((0..=32).contains(&first[3]));
        assert!((0..=32).contains(&first[4]));
        assert!((0..=16).contains(&first[5]));
        assert!((0..=1).contains(&first[6]));
    }

    #[test]
    fn ranked_reply_replay_is_byte_identical_apart_from_timing() {
        let artifact = artifact();
        let samples = vec![MemoryWeightSample {
            prompt: vec![3, 4, 5, 4],
            reply: vec![5, 4, 5, 0],
        }];
        let ranker = CopyCandidateRanker::depth_only();
        let first = evaluate_ranked_copy_replies(&artifact, &samples, 8, &ranker).unwrap();
        let repeat = evaluate_ranked_copy_replies(&artifact, &samples, 8, &ranker).unwrap();
        assert_eq!(first.reading, repeat.reading);
        assert_eq!(first.rows, repeat.rows);
        assert_eq!(first.steps, repeat.steps);
        assert_eq!(first.reply_sha256, repeat.reply_sha256);
        assert_eq!(first.decision_sha256, repeat.decision_sha256);
        assert!(first
            .steps
            .iter()
            .all(|step| step.candidate_count > 0 || step.selected_token == step.writer_token));
    }

    #[test]
    fn reference_prefix_training_is_deterministic_and_never_read_by_inference() {
        let artifact = artifact();
        let sample = MemoryWeightSample {
            prompt: vec![3, 4, 5, 4, 5, 4],
            reply: vec![5, 4, 5, 4, 6, 7],
        };
        let first = reference_training_groups_for_sample(&artifact, &sample, 8, 32, 11).unwrap();
        let repeat = reference_training_groups_for_sample(&artifact, &sample, 8, 32, 11).unwrap();
        assert_eq!(first, repeat);
        assert!(first
            .iter()
            .all(|group| group.row == 11 && group.generated_step < sample.reply.len()));

        let ranker = CopyCandidateRanker::depth_only();
        let generated =
            evaluate_ranked_copy_replies(&artifact, std::slice::from_ref(&sample), 8, &ranker)
                .unwrap();
        assert!(generated
            .steps
            .iter()
            .all(|step| { step.used_copy || step.selected_token == step.longest_token }));
    }

    #[test]
    fn trajectory_names_are_stable() {
        assert_eq!(CopyRankerTrajectory::Longest32.name(), "longest_32");
        assert_eq!(
            CopyRankerTrajectory::ReferencePrefix.name(),
            "reference_prefix"
        );
    }

    #[test]
    fn weight_digest_binds_every_feature() {
        let ranker = CopyCandidateRanker::zero();
        let mut changed = ranker.clone();
        for index in 0..COPY_RANKER_FEATURES {
            let mut current = changed.clone();
            current.weights[index] += 1;
            assert_ne!(ranker.weights_sha256(), current.weights_sha256());
        }
        changed.weights[0] += 1;
        assert_ne!(ranker.weights_sha256(), changed.weights_sha256());
    }
}
