//! Integer pairwise ranking over a geometric candidate union.
//!
//! This is a bounded D84 CPU-development screen. The candidate set combines
//! the artifact writer's top geometric candidates with exact-copy history and
//! its selected fallback. Inference never reads a reference token.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uor_r4_integer::stack::{reply_stop, ReplyEnd};

use crate::copy_key_reply::{
    compare_copy_key_reply_rows, copy_key_reply_decision, exact_suffix_candidates, CopyKeyReplyArm,
    CopyKeyReplyMetric, CopyKeyReplyRow, COPY_KEY_REPLY_MAX_DEPTH,
};
use crate::reply_eval::normalized_reference;
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const GEOMETRIC_CANDIDATE_FEATURES: usize = 8;
pub const GEOMETRIC_CANDIDATE_TOP_WRITER: usize = 128;
pub const GEOMETRIC_CANDIDATE_EPOCHS: usize = 8;
pub const GEOMETRIC_CANDIDATE_MIN_GROUPS: usize = 2_048;
pub const GEOMETRIC_CANDIDATE_MIN_TARGETS: usize = 64;
const LEARNING_RATE: i64 = 4;
const WEIGHT_LIMIT: i64 = 4_096;

pub const GEOMETRIC_CANDIDATE_FEATURE_NAMES: [&str; GEOMETRIC_CANDIDATE_FEATURES] = [
    "bias",
    "exact_depth",
    "sign_agreement",
    "rotated_sign_agreement",
    "recency",
    "frequency",
    "writer_match",
    "writer_rank",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometricCandidateArm {
    Trained,
    Shuffled,
    DepthOnly,
    GeometryOnly,
}

impl GeometricCandidateArm {
    pub fn name(self) -> &'static str {
        match self {
            Self::Trained => "trained",
            Self::Shuffled => "shuffled",
            Self::DepthOnly => "depth_only",
            Self::GeometryOnly => "geometry_only",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometricCandidateRanker {
    pub weights: [i64; GEOMETRIC_CANDIDATE_FEATURES],
    pub epochs: usize,
    pub learning_rate: i64,
    pub weight_limit: i64,
}

impl GeometricCandidateRanker {
    fn zero() -> Self {
        Self {
            weights: [0; GEOMETRIC_CANDIDATE_FEATURES],
            epochs: GEOMETRIC_CANDIDATE_EPOCHS,
            learning_rate: LEARNING_RATE,
            weight_limit: WEIGHT_LIMIT,
        }
    }

    fn depth_only() -> Self {
        let mut ranker = Self::zero();
        ranker.weights[1] = 1;
        ranker
    }

    fn geometry_only() -> Self {
        let mut ranker = Self::zero();
        ranker.weights[3] = 1;
        ranker
    }

    fn score(&self, features: &[i32; GEOMETRIC_CANDIDATE_FEATURES]) -> i64 {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometricCandidateTraining {
    pub trajectory: String,
    pub candidate_set: String,
    pub writer_candidates: usize,
    pub exact_depth: usize,
    pub trained: GeometricCandidateRanker,
    pub shuffled: GeometricCandidateRanker,
    pub depth_only: GeometricCandidateRanker,
    pub geometry_only: GeometricCandidateRanker,
    pub generated_rows: u64,
    pub supervised_groups: u64,
    pub candidate_rows: u64,
    pub target_present_groups: u64,
    pub target_values: u64,
    pub shuffled_target_changes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometricCandidateStep {
    pub row: usize,
    pub generated_step: usize,
    pub writer_token: u32,
    pub longest_token: u32,
    pub selected_token: u32,
    pub selected_exact_depth: Option<usize>,
    pub selected_writer_rank: Option<usize>,
    pub selected_score: Option<i64>,
    pub candidate_count: usize,
    pub used_exact_copy: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometricCandidateReading {
    pub arm: GeometricCandidateArm,
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
    pub exact_copy_decisions: u64,
    pub non_copy_decisions: u64,
    pub selected_depth_counts: [u64; COPY_KEY_REPLY_MAX_DEPTH],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedGeometricCandidateReading {
    pub reading: GeometricCandidateReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<GeometricCandidateStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CandidateSeed {
    token: u32,
    exact_depth: Option<usize>,
    source: Option<usize>,
    writer_rank: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GeometricCandidate {
    token: u32,
    exact_depth: Option<usize>,
    writer_rank: Option<usize>,
    features: [i32; GEOMETRIC_CANDIDATE_FEATURES],
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CandidateGroup {
    row: usize,
    generated_step: usize,
    target: u32,
    candidates: Vec<GeometricCandidate>,
}

pub fn derive_geometric_candidate_rankers(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    seed: u64,
) -> Result<GeometricCandidateTraining> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("geometric candidate ranker training inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("geometric candidate ranker context: {error}")))?
        .context_length();
    let mut groups = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| candidate_groups_for_sample(artifact, sample, cap, context, row))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    groups.sort_by_key(|group| (group.row, group.generated_step));
    let target_values = validate_training_power(&groups)?;
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
    let shuffled_target_changes = groups
        .iter()
        .filter(|group| shuffled_target(group, seed).is_ok_and(|target| target != group.target))
        .count() as u64;
    if shuffled_target_changes == 0 {
        return Err(invalid("geometric candidate target shuffle is vacuous"));
    }
    Ok(GeometricCandidateTraining {
        trajectory: "reference_prefix".to_owned(),
        candidate_set: "writer_top128_nonbos_exact32_fallback".to_owned(),
        writer_candidates: GEOMETRIC_CANDIDATE_TOP_WRITER,
        exact_depth: COPY_KEY_REPLY_MAX_DEPTH,
        trained,
        shuffled,
        depth_only: GeometricCandidateRanker::depth_only(),
        geometry_only: GeometricCandidateRanker::geometry_only(),
        generated_rows: generated_rows as u64,
        supervised_groups: groups.len() as u64,
        candidate_rows,
        target_present_groups: groups.len() as u64,
        target_values: target_values as u64,
        shuffled_target_changes,
    })
}

fn validate_training_power(groups: &[CandidateGroup]) -> Result<usize> {
    if groups.len() < GEOMETRIC_CANDIDATE_MIN_GROUPS {
        return Err(invalid(format!(
            "geometric candidate ranker has {} supervised groups, fewer than {}",
            groups.len(),
            GEOMETRIC_CANDIDATE_MIN_GROUPS
        )));
    }
    let target_values = groups
        .iter()
        .map(|group| group.target)
        .collect::<BTreeSet<_>>()
        .len();
    if target_values < GEOMETRIC_CANDIDATE_MIN_TARGETS {
        return Err(invalid(format!(
            "geometric candidate ranker has {target_values} target tokens, fewer than {}",
            GEOMETRIC_CANDIDATE_MIN_TARGETS
        )));
    }
    Ok(target_values)
}

pub fn evaluate_geometric_candidate_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: GeometricCandidateArm,
    ranker: &GeometricCandidateRanker,
) -> Result<TimedGeometricCandidateReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("geometric candidate reply inputs differ"));
    }
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| evaluate_one(artifact, sample, cap, row, ranker))
        .collect::<Result<Vec<_>>>()?;
    let wall_elapsed_us = started.elapsed().as_micros().try_into().unwrap_or(u64::MAX);
    let (rows, steps): (Vec<_>, Vec<_>) = evaluated.into_iter().unzip();
    let steps = steps.into_iter().flatten().collect::<Vec<_>>();
    let reading = summarize(arm, &rows, &steps)?;
    let generated_tokens_per_second = if wall_elapsed_us == 0 {
        0.0
    } else {
        reading.generated_tokens as f64 * 1_000_000.0 / wall_elapsed_us as f64
    };
    Ok(TimedGeometricCandidateReading {
        reply_sha256: reply_hash(&rows),
        decision_sha256: decision_hash(&steps),
        reading,
        rows,
        steps,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

pub fn compare_geometric_candidate_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<crate::CopyKeyReplyComparison> {
    compare_copy_key_reply_rows(candidate, control, metric)
}

fn candidate_groups_for_sample(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    cap: usize,
    context: usize,
    row: usize,
) -> Result<Vec<CandidateGroup>> {
    let reference = normalized_reference(&sample.reply).to_vec();
    if sample.prompt.is_empty() || sample.prompt.len() > context || reference.len() > cap {
        return Err(invalid(
            "geometric candidate training sample exceeds its context or cap",
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
        let candidates = candidate_union(&forward, &generator.1, &stream, generator.0)?;
        if candidates.iter().any(|candidate| candidate.token == target) {
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
            .map_err(|error| invalid(format!("geometric candidate reference step: {error}")))?;
        generator = (step.token, step.query_q12);
        generated += 1;
    }
    Ok(groups)
}

fn evaluate_one(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    cap: usize,
    row: usize,
    ranker: &GeometricCandidateRanker,
) -> Result<(CopyKeyReplyRow, Vec<GeometricCandidateStep>)> {
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("geometric candidate reply context: {error}")))?
        .context_length();
    let reference = normalized_reference(&sample.reply).to_vec();
    if sample.prompt.len() > context || reference.len() > cap {
        return Err(invalid(
            "geometric candidate reply sample exceeds its cap or context",
        ));
    }
    let (mut forward, mut stream, writer, query) =
        start_reply(artifact, &sample.prompt, cap, context, "reply")?;
    let mut generator = (writer, query);
    let mut tokens = Vec::with_capacity(cap);
    let mut steps = Vec::with_capacity(cap);
    let stop = loop {
        let base = copy_key_reply_decision(
            &stream,
            generator.0,
            CopyKeyReplyArm::Longest32To1,
            0,
            row,
            tokens.len(),
        );
        let candidates = candidate_union(&forward, &generator.1, &stream, generator.0)?;
        let selected = choose_candidate(&candidates, ranker).ok_or_else(|| {
            invalid("geometric candidate ranker produced an empty candidate union")
        })?;
        steps.push(GeometricCandidateStep {
            row,
            generated_step: tokens.len(),
            writer_token: generator.0,
            longest_token: base.token,
            selected_token: selected.token,
            selected_exact_depth: selected.exact_depth,
            selected_writer_rank: selected.writer_rank,
            selected_score: Some(ranker.score(&selected.features)),
            candidate_count: candidates.len(),
            used_exact_copy: selected.exact_depth.is_some(),
        });
        if let Some(stop) = reply_stop(
            selected.token,
            tokens.len(),
            sample.prompt.len(),
            cap,
            context,
        ) {
            break stop;
        }
        tokens.push(selected.token);
        stream.push(selected.token);
        let step = forward
            .step(selected.token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("geometric candidate generated step: {error}")))?;
        generator = (step.token, step.query_q12);
    };
    let generated = GeneratedReply { tokens, stop };
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
    let exact_copy_decisions = steps.iter().filter(|step| step.used_exact_copy).count() as u64;
    let fallback_decisions = steps.len() as u64 - exact_copy_decisions;
    let premature_bos = generated.stop == ReplyEnd::Bos && generated.tokens.len() < reference.len();
    let non_premature_bos = generated.stop == ReplyEnd::Bos && !premature_bos;
    let row = CopyKeyReplyRow {
        row,
        reachable,
        exact_reference: generated.stop == ReplyEnd::Bos && generated.tokens == reference,
        bos_complete: generated.stop == ReplyEnd::Bos,
        opening_correct,
        premature_bos,
        non_premature_bos,
        generated_tokens: generated.tokens.len() as u64,
        reference_tokens: reference.len() as u64,
        correct_body_positions,
        copy_decisions: exact_copy_decisions,
        fallback_decisions,
        stop: match generated.stop {
            ReplyEnd::Bos => "bos",
            ReplyEnd::Length => "cap",
            ReplyEnd::Context => "context",
        }
        .to_owned(),
        reply_sha256: token_hash(&generated.tokens),
    };
    Ok((row, steps))
}

struct GeneratedReply {
    tokens: Vec<u32>,
    stop: ReplyEnd,
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
            "geometric candidate {label} prompt, cap or context is invalid"
        )));
    }
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("geometric candidate {label} forward: {error}")))?;
    let mut stream = Vec::with_capacity(context);
    let mut next = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward.step(token, CpuFbsHead::Writer).map_err(|error| {
            invalid(format!("geometric candidate {label} prompt step: {error}"))
        })?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some((step.token, step.query_q12));
        }
    }
    let (writer, query) =
        next.ok_or_else(|| invalid(format!("geometric candidate {label} prompt is empty")))?;
    Ok((forward, stream, writer, query))
}

fn candidate_union(
    forward: &CpuFbsForward,
    query: &[i32],
    stream: &[u32],
    writer: u32,
) -> Result<Vec<GeometricCandidate>> {
    let seeds = candidate_seeds(forward, query, stream, writer)?;
    seeds
        .into_values()
        .map(|candidate| {
            Ok(GeometricCandidate {
                token: candidate.token,
                exact_depth: candidate.exact_depth,
                writer_rank: candidate.writer_rank,
                features: candidate_features(forward, query, stream, &candidate, writer)?,
            })
        })
        .collect()
}

fn candidate_seeds(
    forward: &CpuFbsForward,
    query: &[i32],
    stream: &[u32],
    writer: u32,
) -> Result<BTreeMap<u32, CandidateSeed>> {
    let mut seeds = BTreeMap::new();
    let writer_order = forward
        .writer_candidate_order(query, GEOMETRIC_CANDIDATE_TOP_WRITER + 1)
        .map_err(|error| invalid(format!("geometric writer candidates: {error}")))?;
    for (index, &(token, _)) in writer_order
        .iter()
        .filter(|(token, _)| *token != 0)
        .take(GEOMETRIC_CANDIDATE_TOP_WRITER)
        .enumerate()
    {
        seeds.entry(token).or_insert(CandidateSeed {
            token,
            exact_depth: None,
            source: None,
            writer_rank: Some(index + 1),
        });
    }
    let exact = exact_suffix_candidates(stream);
    for depth in 1..=COPY_KEY_REPLY_MAX_DEPTH {
        let Some((token, source)) = exact[depth - 1] else {
            continue;
        };
        if token == 0 {
            continue;
        }
        let candidate = seeds.entry(token).or_insert(CandidateSeed {
            token,
            exact_depth: None,
            source: None,
            writer_rank: None,
        });
        candidate.exact_depth = Some(depth);
        candidate.source = Some(source);
    }
    if writer != 0 {
        seeds.entry(writer).or_insert(CandidateSeed {
            token: writer,
            exact_depth: None,
            source: None,
            writer_rank: None,
        });
    }
    Ok(seeds)
}

fn candidate_features(
    forward: &CpuFbsForward,
    query: &[i32],
    stream: &[u32],
    candidate: &CandidateSeed,
    writer: u32,
) -> Result<[i32; GEOMETRIC_CANDIDATE_FEATURES]> {
    let width = query.len();
    let row = forward
        .token_table()
        .get(candidate.token as usize * width..(candidate.token as usize + 1) * width)
        .ok_or_else(|| invalid("geometric candidate token row differs"))?;
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
    let source = candidate
        .source
        .or_else(|| stream.iter().rposition(|&token| token == candidate.token));
    let recency = source.map_or(0, |source| {
        let age = stream.len().saturating_sub(source);
        32i32.saturating_sub((age / 8).min(32) as i32)
    });
    let frequency = stream
        .iter()
        .filter(|&&token| token == candidate.token)
        .count()
        .min(16) as i32;
    let writer_match = i32::from(writer == candidate.token);
    let writer_rank = candidate.writer_rank.map_or(0, |rank| {
        let rank = rank.min(GEOMETRIC_CANDIDATE_TOP_WRITER);
        32i32 - (((rank - 1) * 32) / GEOMETRIC_CANDIDATE_TOP_WRITER).min(31) as i32
    });
    Ok([
        1,
        candidate.exact_depth.unwrap_or(0).min(32) as i32,
        (agreement * 32 / width) as i32,
        (rotated * 32 / width) as i32,
        recency,
        frequency,
        writer_match,
        writer_rank,
    ])
}

fn choose_candidate<'a>(
    candidates: &'a [GeometricCandidate],
    ranker: &GeometricCandidateRanker,
) -> Option<&'a GeometricCandidate> {
    candidates
        .iter()
        .min_by_key(|candidate| (Reverse(ranker.score(&candidate.features)), candidate.token))
}

fn fit_ranker(
    groups: &[CandidateGroup],
    seed: u64,
    shuffled: bool,
) -> Result<GeometricCandidateRanker> {
    let mut ranker = GeometricCandidateRanker::zero();
    for _ in 0..ranker.epochs {
        for group in groups {
            let target = if shuffled {
                shuffled_target(group, seed)?
            } else {
                group.target
            };
            let chosen = choose_candidate(&group.candidates, &ranker)
                .ok_or_else(|| invalid("geometric candidate training group is empty"))?;
            if chosen.token == target {
                continue;
            }
            let target_candidate = group
                .candidates
                .iter()
                .find(|candidate| candidate.token == target)
                .ok_or_else(|| invalid("geometric candidate training target differs"))?;
            for index in 0..GEOMETRIC_CANDIDATE_FEATURES {
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
        return Err(invalid("geometric candidate shuffle needs two candidates"));
    }
    let target_index = group
        .candidates
        .iter()
        .position(|candidate| candidate.token == group.target)
        .ok_or_else(|| invalid("geometric candidate shuffle target differs"))?;
    let shift = 1 + seed as usize % (group.candidates.len() - 1);
    Ok(group.candidates[(target_index + shift) % group.candidates.len()].token)
}

fn summarize(
    arm: GeometricCandidateArm,
    rows: &[CopyKeyReplyRow],
    steps: &[GeometricCandidateStep],
) -> Result<GeometricCandidateReading> {
    let mut reading = GeometricCandidateReading {
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
        exact_copy_decisions: 0,
        non_copy_decisions: 0,
        selected_depth_counts: [0; COPY_KEY_REPLY_MAX_DEPTH],
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
        reading.exact_copy_decisions += u64::from(step.used_exact_copy);
        reading.non_copy_decisions += u64::from(!step.used_exact_copy);
        if let Some(depth) = step.selected_exact_depth {
            let slot = reading
                .selected_depth_counts
                .get_mut(depth - 1)
                .ok_or_else(|| invalid("geometric candidate selected depth differs"))?;
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

fn decision_hash(steps: &[GeometricCandidateStep]) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(steps.len() as u64).to_le_bytes());
    for step in steps {
        bytes.extend_from_slice(&(step.row as u64).to_le_bytes());
        bytes.extend_from_slice(&(step.generated_step as u64).to_le_bytes());
        bytes.extend_from_slice(&step.writer_token.to_le_bytes());
        bytes.extend_from_slice(&step.longest_token.to_le_bytes());
        bytes.extend_from_slice(&step.selected_token.to_le_bytes());
        bytes.push(step.selected_exact_depth.map_or(0, |depth| depth as u8));
        bytes.push(u8::from(step.selected_writer_rank.is_some()));
        bytes.extend_from_slice(&step.selected_writer_rank.unwrap_or_default().to_le_bytes());
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

    fn candidate(token: u32, depth: usize, geometry: i32) -> GeometricCandidate {
        GeometricCandidate {
            token,
            exact_depth: (depth > 0).then_some(depth),
            writer_rank: None,
            features: [1, depth as i32, geometry, geometry, 0, 0, 0, 0],
        }
    }

    #[test]
    fn candidate_union_is_deterministic_unique_and_excludes_bos() {
        let artifact = artifact();
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        forward.mark_reply_start();
        let step = forward.step(3, CpuFbsHead::Writer).unwrap();
        let writer_order = forward
            .writer_candidate_order(&step.query_q12, GEOMETRIC_CANDIDATE_TOP_WRITER)
            .unwrap();
        assert_eq!(
            writer_order.first().map(|candidate| candidate.0),
            Some(step.token)
        );
        assert!(
            writer_order
                .windows(2)
                .all(|pair| pair[0].1 > pair[1].1
                    || (pair[0].1 == pair[1].1 && pair[0].0 < pair[1].0))
        );
        let first = candidate_union(&forward, &step.query_q12, &[3], step.token).unwrap();
        let repeat = candidate_union(&forward, &step.query_q12, &[3], step.token).unwrap();
        assert_eq!(first, repeat);
        assert!(!first.is_empty());
        assert!(first.iter().all(|candidate| candidate.token != 0));
        assert!(first.windows(2).all(|pair| pair[0].token < pair[1].token));
        assert!(first.len() <= GEOMETRIC_CANDIDATE_TOP_WRITER + COPY_KEY_REPLY_MAX_DEPTH + 1);

        let expected_writer = forward
            .writer_candidate_order(&step.query_q12, GEOMETRIC_CANDIDATE_TOP_WRITER + 1)
            .unwrap()
            .into_iter()
            .filter(|(token, _)| *token != 0)
            .take(GEOMETRIC_CANDIDATE_TOP_WRITER)
            .collect::<Vec<_>>();
        assert_eq!(expected_writer.len(), GEOMETRIC_CANDIDATE_TOP_WRITER);
        let seeds = candidate_seeds(&forward, &step.query_q12, &[3], step.token).unwrap();
        for (rank, &(token, _)) in expected_writer.iter().enumerate() {
            assert_eq!(seeds[&token].writer_rank, Some(rank + 1));
        }
        assert_eq!(seeds[&step.token].token, step.token);

        let admitted = expected_writer
            .iter()
            .map(|(token, _)| *token)
            .collect::<BTreeSet<_>>();
        let fallback_token = (1..forward.vocabulary() as u32)
            .find(|token| !admitted.contains(token))
            .unwrap();
        let fallback = candidate_seeds(&forward, &step.query_q12, &[3], fallback_token).unwrap();
        assert_eq!(fallback[&fallback_token].token, fallback_token);
        assert_eq!(fallback[&fallback_token].exact_depth, None);
        assert_eq!(fallback[&fallback_token].writer_rank, None);
    }

    #[test]
    fn exact_copy_retains_depth_32_and_its_deepest_source() {
        let artifact = artifact();
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        forward.mark_reply_start();
        let step = forward.step(3, CpuFbsHead::Writer).unwrap();
        let first_block = (1..=28).chain([1, 1, 1, 1]).collect::<Vec<_>>();
        let second_block = [1]
            .into_iter()
            .chain(first_block[..30].iter().copied())
            .chain([1])
            .collect::<Vec<_>>();
        assert_eq!(first_block.len(), COPY_KEY_REPLY_MAX_DEPTH);
        assert_eq!(second_block.len(), COPY_KEY_REPLY_MAX_DEPTH);
        let stream = [99]
            .into_iter()
            .chain(first_block)
            .chain(second_block)
            .chain([1])
            .collect::<Vec<_>>();
        let exact = exact_suffix_candidates(&stream);
        assert_eq!(exact[0], Some((1, 64)));
        assert_eq!(exact[COPY_KEY_REPLY_MAX_DEPTH - 1], Some((1, 33)));

        let seeds = candidate_seeds(&forward, &step.query_q12, &stream, step.token).unwrap();
        let candidate = &seeds[&1];
        assert_eq!(candidate.exact_depth, Some(COPY_KEY_REPLY_MAX_DEPTH));
        assert_eq!(candidate.source, Some(33));
    }

    #[test]
    fn candidate_features_are_bounded() {
        let artifact = artifact();
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        forward.mark_reply_start();
        let step = forward.step(3, CpuFbsHead::Writer).unwrap();
        let candidates =
            candidate_union(&forward, &step.query_q12, &[3, 4, 3], step.token).unwrap();
        for candidate in candidates {
            assert_eq!(candidate.features[0], 1);
            assert!((0..=32).contains(&candidate.features[1]));
            assert!((0..=32).contains(&candidate.features[2]));
            assert!((0..=32).contains(&candidate.features[3]));
            assert!((0..=32).contains(&candidate.features[4]));
            assert!((0..=16).contains(&candidate.features[5]));
            assert!((0..=1).contains(&candidate.features[6]));
            assert!((0..=32).contains(&candidate.features[7]));
        }
    }

    #[test]
    fn pairwise_fit_can_override_writer_rank() {
        let mut target = candidate(9, 1, 20);
        target.features[7] = 0;
        let mut control = candidate(7, 0, 31);
        control.features[2] = 31;
        control.features[3] = 31;
        control.features[7] = 32;
        let group = CandidateGroup {
            row: 0,
            generated_step: 0,
            target: 9,
            candidates: vec![target, control],
        };
        let ranker = fit_ranker(&[group], 17, false).unwrap();
        let candidates = [
            GeometricCandidate {
                exact_depth: Some(1),
                ..candidate(9, 1, 20)
            },
            GeometricCandidate {
                exact_depth: None,
                ..candidate(7, 0, 31)
            },
        ];
        let chosen = choose_candidate(&candidates, &ranker).unwrap();
        assert_eq!(chosen.token, 9);
    }

    #[test]
    fn geometry_only_and_ties_are_deterministic() {
        let geometry_only = GeometricCandidateRanker::geometry_only();
        let candidates = [candidate(9, 0, 31), candidate(7, 32, 10)];
        assert_eq!(
            choose_candidate(&candidates, &geometry_only).unwrap().token,
            9
        );

        let tied = [candidate(9, 4, 20), candidate(7, 4, 20)];
        assert_eq!(
            choose_candidate(&tied, &GeometricCandidateRanker::zero())
                .unwrap()
                .token,
            7
        );
    }

    #[test]
    fn shuffled_targets_are_deterministic_and_non_vacuous() {
        let group = CandidateGroup {
            row: 0,
            generated_step: 0,
            target: 3,
            candidates: vec![
                candidate(3, 3, 30),
                candidate(5, 1, 10),
                candidate(7, 0, 20),
            ],
        };
        let first_target = shuffled_target(&group, 29).unwrap();
        let repeat_target = shuffled_target(&group, 29).unwrap();
        assert_eq!(first_target, repeat_target);
        assert_ne!(first_target, group.target);

        let first_ranker = fit_ranker(std::slice::from_ref(&group), 29, true).unwrap();
        let repeat_ranker = fit_ranker(std::slice::from_ref(&group), 29, true).unwrap();
        assert_eq!(first_ranker, repeat_ranker);

        let singleton = CandidateGroup {
            candidates: vec![candidate(3, 3, 30)],
            ..group
        };
        assert!(shuffled_target(&singleton, 29).is_err());
    }

    #[test]
    fn generated_reply_is_deterministic_and_excludes_bos() {
        let artifact = artifact();
        let sample = MemoryWeightSample {
            prompt: vec![3, 4, 5, 4, 5, 4],
            reply: vec![5, 4, 5, 4, 6, 7],
        };
        let ranker = GeometricCandidateRanker::geometry_only();
        let first = evaluate_geometric_candidate_replies(
            &artifact,
            std::slice::from_ref(&sample),
            8,
            GeometricCandidateArm::GeometryOnly,
            &ranker,
        )
        .unwrap();
        let repeat = evaluate_geometric_candidate_replies(
            &artifact,
            std::slice::from_ref(&sample),
            8,
            GeometricCandidateArm::GeometryOnly,
            &ranker,
        )
        .unwrap();
        assert_eq!(first.steps, repeat.steps);
        assert_eq!(first.reply_sha256, repeat.reply_sha256);
        assert_eq!(first.decision_sha256, repeat.decision_sha256);
        assert!(first.steps.iter().all(|step| step.selected_token != 0));
        assert!(first.steps.iter().all(|step| step.candidate_count > 0));
    }

    #[test]
    fn reference_training_is_deterministic_and_separate_from_inference() {
        let artifact = artifact();
        let sample = MemoryWeightSample {
            prompt: vec![3, 4, 5, 4, 5, 4],
            reply: vec![5, 4, 5, 4, 6, 7],
        };
        let first = candidate_groups_for_sample(&artifact, &sample, 8, 32, 11).unwrap();
        let repeat = candidate_groups_for_sample(&artifact, &sample, 8, 32, 11).unwrap();
        assert_eq!(first, repeat);
        assert!(first.iter().all(|group| group.row == 11));
        let ranker = GeometricCandidateRanker::geometry_only();
        let generated = evaluate_geometric_candidate_replies(
            &artifact,
            std::slice::from_ref(&sample),
            8,
            GeometricCandidateArm::GeometryOnly,
            &ranker,
        )
        .unwrap();
        assert!(generated.steps.iter().all(|step| step.selected_token != 0));

        let mut changed_reference = sample.clone();
        changed_reference.reply = vec![6, 7, 8, 9, 10, 11];
        let changed = evaluate_geometric_candidate_replies(
            &artifact,
            std::slice::from_ref(&changed_reference),
            8,
            GeometricCandidateArm::GeometryOnly,
            &ranker,
        )
        .unwrap();
        assert_eq!(generated.steps, changed.steps);
        assert_eq!(generated.reply_sha256, changed.reply_sha256);
        assert_eq!(generated.decision_sha256, changed.decision_sha256);
    }

    #[test]
    fn registered_power_floors_are_exact() {
        let groups = |count: usize, targets: u32| {
            (0..count)
                .map(|index| CandidateGroup {
                    row: index,
                    generated_step: 0,
                    target: index as u32 % targets,
                    candidates: Vec::new(),
                })
                .collect::<Vec<_>>()
        };

        let below_groups = groups(GEOMETRIC_CANDIDATE_MIN_GROUPS - 1, 64);
        let groups_error = validate_training_power(&below_groups).unwrap_err();
        assert!(
            groups_error.to_string().contains("fewer than 2048"),
            "{groups_error}"
        );
        assert_eq!(
            validate_training_power(&groups(GEOMETRIC_CANDIDATE_MIN_GROUPS, 64)).unwrap(),
            64
        );

        let below_targets = groups(GEOMETRIC_CANDIDATE_MIN_GROUPS, 63);
        let targets_error = validate_training_power(&below_targets).unwrap_err();
        assert!(
            targets_error.to_string().contains("fewer than 64"),
            "{targets_error}"
        );
        assert_eq!(
            validate_training_power(&groups(GEOMETRIC_CANDIDATE_MIN_GROUPS, 64)).unwrap(),
            64
        );
    }

    #[test]
    fn reachable_reply_floor_excludes_cap_and_context_boundaries() {
        assert!(reference_reachable(1, 62, 64, 64));
        assert!(!reference_reachable(1, 63, 64, 64));
        assert!(!reference_reachable(1, 64, 64, 65));
        assert!(reference_reachable(1, 63, 65, 65));
    }

    #[test]
    fn weight_digest_binds_every_feature() {
        let ranker = GeometricCandidateRanker::zero();
        let digest = ranker.weights_sha256();
        for index in 0..GEOMETRIC_CANDIDATE_FEATURES {
            let mut changed = ranker.clone();
            changed.weights[index] += 1;
            assert_ne!(changed.weights_sha256(), digest);
        }
    }
}
