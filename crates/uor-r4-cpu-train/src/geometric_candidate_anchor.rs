//! Longest-32-anchored ranking over an expanded CPU-only geometric cell set.

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

pub const ANCHORED_CANDIDATE_FEATURES: usize = 12;
pub const ANCHORED_CANDIDATE_EPOCHS: usize = 8;
pub const ANCHORED_CANDIDATE_MIN_GROUPS: usize = 2_048;
pub const ANCHORED_CANDIDATE_MIN_TARGETS: usize = 64;
pub const ANCHORED_CANDIDATE_MIN_BASELINE_WRONG: usize = 256;
pub const ANCHORED_CANDIDATE_CELL_SWEEP: [usize; 5] = [8, 12, 16, 24, 32];
const LEARNING_RATE: i64 = 4;
const WEIGHT_LIMIT: i64 = 4_096;

pub const ANCHORED_CANDIDATE_FEATURE_NAMES: [&str; ANCHORED_CANDIDATE_FEATURES] = [
    "bias",
    "exact_depth",
    "sign_agreement",
    "rotated_sign_agreement",
    "recency",
    "frequency",
    "writer_match",
    "writer_rank",
    "is_longest_32",
    "depth_gap",
    "sign_agreement_gain",
    "rotated_agreement_gain",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchoredCandidateArm {
    Trained,
    Shuffled,
    GeometryOnly,
    Unanchored,
}

impl AnchoredCandidateArm {
    pub fn name(self) -> &'static str {
        match self {
            Self::Trained => "anchored_trained",
            Self::Shuffled => "anchored_shuffled",
            Self::GeometryOnly => "anchored_geometry_only",
            Self::Unanchored => "unanchored_trained",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchoredCandidateRanker {
    pub weights: [i64; ANCHORED_CANDIDATE_FEATURES],
    pub epochs: usize,
    pub learning_rate: i64,
    pub weight_limit: i64,
}

impl AnchoredCandidateRanker {
    fn zero() -> Self {
        Self {
            weights: [0; ANCHORED_CANDIDATE_FEATURES],
            epochs: ANCHORED_CANDIDATE_EPOCHS,
            learning_rate: LEARNING_RATE,
            weight_limit: WEIGHT_LIMIT,
        }
    }

    fn geometry_only() -> Self {
        let mut ranker = Self::zero();
        ranker.weights[3] = 1;
        ranker
    }

    fn score(&self, features: &[i32; ANCHORED_CANDIDATE_FEATURES]) -> i64 {
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
pub struct ExpandedCandidateCalibration {
    pub cells: usize,
    pub writer_candidate_limit: usize,
    pub writer_candidate_rows: u64,
    pub min_writer_candidates_per_group: usize,
    pub max_writer_candidates_per_group: usize,
    pub generated_rows: u64,
    pub target_present_groups: u64,
    pub target_values: u64,
    pub baseline_correct_groups: u64,
    pub baseline_wrong_groups: u64,
    pub candidate_rows: u64,
}

impl ExpandedCandidateCalibration {
    pub fn clears_floors(&self) -> bool {
        self.target_present_groups >= ANCHORED_CANDIDATE_MIN_GROUPS as u64
            && self.target_values >= ANCHORED_CANDIDATE_MIN_TARGETS as u64
            && self.baseline_wrong_groups >= ANCHORED_CANDIDATE_MIN_BASELINE_WRONG as u64
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchoredCandidateTraining {
    pub candidate_set: String,
    pub cells: usize,
    pub writer_candidate_limit: usize,
    pub writer_candidate_rows: u64,
    pub min_writer_candidates_per_group: usize,
    pub max_writer_candidates_per_group: usize,
    pub exact_depth: usize,
    pub trained: AnchoredCandidateRanker,
    pub shuffled: AnchoredCandidateRanker,
    pub geometry_only: AnchoredCandidateRanker,
    pub unanchored: AnchoredCandidateRanker,
    pub generated_rows: u64,
    pub supervised_groups: u64,
    pub baseline_correct_groups: u64,
    pub baseline_wrong_groups: u64,
    pub candidate_rows: u64,
    pub target_values: u64,
    pub shuffled_target_changes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchoredCandidateReading {
    pub arm: AnchoredCandidateArm,
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
    pub baseline_kept_decisions: u64,
    pub baseline_overrides: u64,
    pub selected_depth_counts: [u64; COPY_KEY_REPLY_MAX_DEPTH],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedAnchoredCandidateReading {
    pub reading: AnchoredCandidateReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<crate::GeometricCandidateStep>,
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
struct AnchoredCandidate {
    token: u32,
    exact_depth: Option<usize>,
    writer_rank: Option<usize>,
    features: [i32; ANCHORED_CANDIDATE_FEATURES],
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CandidateGroup {
    row: usize,
    generated_step: usize,
    target: u32,
    baseline: u32,
    candidates: Vec<AnchoredCandidate>,
}

struct FeatureContext<'a> {
    forward: &'a CpuFbsForward,
    query: &'a [i32],
    stream: &'a [u32],
    writer: u32,
    baseline: u32,
    baseline_depth: usize,
    writer_limit: usize,
}

pub fn calibrate_expanded_candidate_union(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    cells: &[usize],
) -> Result<Vec<ExpandedCandidateCalibration>> {
    validate_inputs(samples, cap)?;
    if cells.is_empty() || cells.iter().any(|&count| count == 0 || count > 256) {
        return Err(invalid("expanded candidate calibration settings differ"));
    }
    let mut unique = BTreeSet::new();
    if cells.iter().any(|&count| !unique.insert(count)) {
        return Err(invalid(
            "expanded candidate calibration repeats a cell count",
        ));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("expanded candidate context: {error}")))?
        .context_length();
    cells
        .iter()
        .map(|&count| {
            let groups = collect_groups(artifact, samples, cap, context, count)?;
            calibration_from_groups(count, &groups)
        })
        .collect()
}

pub fn derive_anchored_candidate_rankers(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    seed: u64,
    cells: usize,
) -> Result<AnchoredCandidateTraining> {
    validate_inputs(samples, cap)?;
    if cells == 0 || cells > 256 {
        return Err(invalid("anchored candidate cell count differs"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("anchored candidate context: {error}")))?
        .context_length();
    let groups = collect_groups(artifact, samples, cap, context, cells)?;
    let calibration = calibration_from_groups(cells, &groups)?;
    if !calibration.clears_floors() {
        return Err(invalid(format!(
            "anchored candidate calibration has {} groups, {} target tokens and {} baseline-wrong groups; requires {}, {} and {}",
            calibration.target_present_groups,
            calibration.target_values,
            calibration.baseline_wrong_groups,
            ANCHORED_CANDIDATE_MIN_GROUPS,
            ANCHORED_CANDIDATE_MIN_TARGETS,
            ANCHORED_CANDIDATE_MIN_BASELINE_WRONG
        )));
    }
    if calibration.writer_candidate_rows == 0
        || calibration.min_writer_candidates_per_group == 0
        || calibration.max_writer_candidates_per_group > calibration.writer_candidate_limit
    {
        return Err(invalid("anchored candidate writer-candidate counts differ"));
    }
    let trained = fit_anchored(&groups, seed, false)?;
    let shuffled = fit_anchored(&groups, seed, true)?;
    let unanchored = fit_unanchored(&groups, seed)?;
    let shuffled_target_changes = groups
        .iter()
        .filter(|group| shuffled_target(group, seed).is_ok_and(|target| target != group.target))
        .count() as u64;
    if shuffled_target_changes == 0 {
        return Err(invalid("anchored candidate target shuffle is vacuous"));
    }
    Ok(AnchoredCandidateTraining {
        candidate_set: "expanded_writer_cells_exact32_anchored".to_owned(),
        cells,
        writer_candidate_limit: cells.saturating_mul(16),
        writer_candidate_rows: calibration.writer_candidate_rows,
        min_writer_candidates_per_group: calibration.min_writer_candidates_per_group,
        max_writer_candidates_per_group: calibration.max_writer_candidates_per_group,
        exact_depth: COPY_KEY_REPLY_MAX_DEPTH,
        trained,
        shuffled,
        geometry_only: AnchoredCandidateRanker::geometry_only(),
        unanchored,
        generated_rows: calibration.generated_rows,
        supervised_groups: calibration.target_present_groups,
        baseline_correct_groups: calibration.baseline_correct_groups,
        baseline_wrong_groups: calibration.baseline_wrong_groups,
        candidate_rows: calibration.candidate_rows,
        target_values: calibration.target_values,
        shuffled_target_changes,
    })
}

pub fn evaluate_anchored_candidate_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: AnchoredCandidateArm,
    ranker: &AnchoredCandidateRanker,
    cells: usize,
) -> Result<TimedAnchoredCandidateReading> {
    validate_inputs(samples, cap)?;
    if cells == 0 || cells > 256 {
        return Err(invalid("anchored candidate evaluation cells differ"));
    }
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| evaluate_one(artifact, sample, cap, row, arm, ranker, cells))
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
    Ok(TimedAnchoredCandidateReading {
        reply_sha256: reply_hash(&rows),
        decision_sha256: decision_hash(&steps),
        reading,
        rows,
        steps,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

pub fn compare_anchored_candidate_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<crate::CopyKeyReplyComparison> {
    compare_copy_key_reply_rows(candidate, control, metric)
}

fn validate_inputs(samples: &[MemoryWeightSample], cap: usize) -> Result<()> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("anchored candidate inputs differ"));
    }
    Ok(())
}

fn collect_groups(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    context: usize,
    cells: usize,
) -> Result<Vec<CandidateGroup>> {
    let mut groups = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            candidate_groups_for_sample(artifact, sample, cap, context, row, cells)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    groups.sort_by_key(|group| (group.row, group.generated_step));
    Ok(groups)
}

fn calibration_from_groups(
    cells: usize,
    groups: &[CandidateGroup],
) -> Result<ExpandedCandidateCalibration> {
    let baseline_wrong_groups = groups
        .iter()
        .filter(|group| group.baseline != group.target)
        .count() as u64;
    let writer_candidate_counts = groups
        .iter()
        .map(|group| {
            group
                .candidates
                .iter()
                .filter(|candidate| candidate.writer_rank.is_some())
                .count()
        })
        .collect::<Vec<_>>();
    Ok(ExpandedCandidateCalibration {
        cells,
        writer_candidate_limit: cells.saturating_mul(16),
        writer_candidate_rows: groups
            .iter()
            .map(|group| {
                group
                    .candidates
                    .iter()
                    .filter(|candidate| candidate.writer_rank.is_some())
                    .count() as u64
            })
            .sum(),
        min_writer_candidates_per_group: writer_candidate_counts.iter().copied().min().unwrap_or(0),
        max_writer_candidates_per_group: writer_candidate_counts.iter().copied().max().unwrap_or(0),
        generated_rows: groups
            .iter()
            .map(|group| group.row)
            .collect::<BTreeSet<_>>()
            .len() as u64,
        target_present_groups: groups.len() as u64,
        target_values: groups
            .iter()
            .map(|group| group.target)
            .collect::<BTreeSet<_>>()
            .len() as u64,
        baseline_correct_groups: groups.len() as u64 - baseline_wrong_groups,
        baseline_wrong_groups,
        candidate_rows: groups
            .iter()
            .map(|group| group.candidates.len() as u64)
            .sum(),
    })
}

fn candidate_groups_for_sample(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    cap: usize,
    context: usize,
    row: usize,
    cells: usize,
) -> Result<Vec<CandidateGroup>> {
    let reference = normalized_reference(&sample.reply).to_vec();
    if sample.prompt.is_empty() || sample.prompt.len() > context || reference.len() > cap {
        return Err(invalid(
            "anchored candidate training sample exceeds its context or cap",
        ));
    }
    let (mut forward, mut stream, writer, query) =
        start_reply(artifact, &sample.prompt, cap, context, "training")?;
    let mut generator = (writer, query);
    let mut generated = 0usize;
    let mut groups = Vec::new();
    while generated < reference.len()
        && generated < cap
        && sample.prompt.len().saturating_add(generated) < context
    {
        let target = reference[generated];
        let baseline = baseline_token(&stream, generator.0);
        let candidates = candidate_union(&forward, &generator.1, &stream, generator.0, cells)?;
        if baseline != 0 && candidates.iter().any(|candidate| candidate.token == target) {
            groups.push(CandidateGroup {
                row,
                generated_step: generated,
                target,
                baseline,
                candidates,
            });
        }
        stream.push(target);
        let step = forward
            .step(target, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("anchored candidate reference step: {error}")))?;
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
    arm: AnchoredCandidateArm,
    ranker: &AnchoredCandidateRanker,
    cells: usize,
) -> Result<(CopyKeyReplyRow, Vec<crate::GeometricCandidateStep>)> {
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("anchored candidate reply context: {error}")))?
        .context_length();
    let reference = normalized_reference(&sample.reply).to_vec();
    if sample.prompt.len() > context || reference.len() > cap {
        return Err(invalid(
            "anchored candidate reply sample exceeds its cap or context",
        ));
    }
    let (mut forward, mut stream, writer, query) =
        start_reply(artifact, &sample.prompt, cap, context, "reply")?;
    let mut generator = (writer, query);
    let mut tokens = Vec::with_capacity(cap);
    let mut steps = Vec::with_capacity(cap);
    let stop = loop {
        let baseline = copy_key_reply_decision(
            &stream,
            generator.0,
            CopyKeyReplyArm::Longest32To1,
            0,
            row,
            tokens.len(),
        );
        let candidates = candidate_union(&forward, &generator.1, &stream, generator.0, cells)?;
        let selected = match choose_generated(&candidates, arm, ranker, baseline.token)
            .ok_or_else(|| invalid("anchored candidate ranker produced an empty candidate union"))?
        {
            CandidateChoice::Stop => {
                steps.push(crate::GeometricCandidateStep {
                    row,
                    generated_step: tokens.len(),
                    writer_token: generator.0,
                    longest_token: 0,
                    selected_token: 0,
                    selected_exact_depth: None,
                    selected_writer_rank: None,
                    selected_score: None,
                    candidate_count: candidates.len(),
                    used_exact_copy: false,
                });
                break reply_stop(0, tokens.len(), sample.prompt.len(), cap, context)
                    .ok_or_else(|| invalid("writer BOS candidate did not stop"))?;
            }
            CandidateChoice::Token(selected) => selected,
        };
        steps.push(crate::GeometricCandidateStep {
            row,
            generated_step: tokens.len(),
            writer_token: generator.0,
            longest_token: baseline.token,
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
            .map_err(|error| invalid(format!("anchored candidate generated step: {error}")))?;
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
    Ok((
        CopyKeyReplyRow {
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
        },
        steps,
    ))
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
            "anchored candidate {label} prompt, cap or context is invalid"
        )));
    }
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("anchored candidate {label} forward: {error}")))?;
    let mut stream = Vec::with_capacity(context);
    let mut next = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("anchored candidate {label} prompt step: {error}")))?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some((step.token, step.query_q12));
        }
    }
    let (writer, query) =
        next.ok_or_else(|| invalid(format!("anchored candidate {label} prompt is empty")))?;
    Ok((forward, stream, writer, query))
}

fn baseline_token(stream: &[u32], writer: u32) -> u32 {
    copy_key_reply_decision(stream, writer, CopyKeyReplyArm::Longest32To1, 0, 0, 0).token
}

fn candidate_union(
    forward: &CpuFbsForward,
    query: &[i32],
    stream: &[u32],
    writer: u32,
    cells: usize,
) -> Result<Vec<AnchoredCandidate>> {
    let seeds = candidate_seeds(forward, query, stream, writer, cells)?;
    let baseline = copy_key_reply_decision(stream, writer, CopyKeyReplyArm::Longest32To1, 0, 0, 0);
    let baseline_depth = baseline.depth.unwrap_or(0);
    let baseline = baseline.token;
    let context = FeatureContext {
        forward,
        query,
        stream,
        writer,
        baseline,
        baseline_depth,
        writer_limit: cells.saturating_mul(16),
    };
    seeds
        .into_values()
        .map(|candidate| {
            Ok(AnchoredCandidate {
                token: candidate.token,
                exact_depth: candidate.exact_depth,
                writer_rank: candidate.writer_rank,
                features: candidate_features(&context, &candidate)?,
            })
        })
        .collect()
}

fn candidate_seeds(
    forward: &CpuFbsForward,
    query: &[i32],
    stream: &[u32],
    writer: u32,
    cells: usize,
) -> Result<BTreeMap<u32, CandidateSeed>> {
    let writer_limit = cells.saturating_mul(16);
    let mut seeds = BTreeMap::new();
    let writer_order = forward
        .writer_candidate_order_with_cells(query, cells, writer_limit + 1)
        .map_err(|error| invalid(format!("expanded writer candidates: {error}")))?;
    for (index, &(token, _)) in writer_order
        .iter()
        .filter(|(token, _)| *token != 0)
        .take(writer_limit)
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
    let baseline = baseline_token(stream, writer);
    if baseline != 0 {
        seeds.entry(baseline).or_insert(CandidateSeed {
            token: baseline,
            exact_depth: None,
            source: None,
            writer_rank: None,
        });
    }
    Ok(seeds)
}

fn candidate_features(
    context: &FeatureContext<'_>,
    candidate: &CandidateSeed,
) -> Result<[i32; ANCHORED_CANDIDATE_FEATURES]> {
    let (agreement, rotated) = agreement_features(context.forward, context.query, candidate.token)?;
    let (baseline_agreement, baseline_rotated) =
        agreement_features(context.forward, context.query, context.baseline)?;
    let source = candidate.source.or_else(|| {
        context
            .stream
            .iter()
            .rposition(|&token| token == candidate.token)
    });
    let recency = source.map_or(0, |source| {
        let age = context.stream.len().saturating_sub(source);
        32i32.saturating_sub((age / 8).min(32) as i32)
    });
    let frequency = context
        .stream
        .iter()
        .filter(|&&token| token == candidate.token)
        .count()
        .min(16) as i32;
    let writer_match = i32::from(context.writer == candidate.token);
    let writer_rank = candidate.writer_rank.map_or(0, |rank| {
        let rank = rank.min(context.writer_limit.max(1));
        32i32 - (((rank - 1) * 32) / context.writer_limit.max(1)).min(31) as i32
    });
    let baseline_depth = context.baseline_depth.min(32) as i32;
    let depth = candidate.exact_depth.unwrap_or(0).min(32) as i32;
    let width = context.query.len();
    Ok([
        1,
        depth,
        (agreement * 32 / width) as i32,
        (rotated * 32 / width) as i32,
        recency,
        frequency,
        writer_match,
        writer_rank,
        i32::from(candidate.token == context.baseline),
        (depth - baseline_depth).clamp(-32, 32),
        ((agreement as i32 - baseline_agreement as i32) * 32 / width as i32).clamp(-32, 32),
        ((rotated as i32 - baseline_rotated as i32) * 32 / width as i32).clamp(-32, 32),
    ])
}

fn agreement_features(
    forward: &CpuFbsForward,
    query: &[i32],
    token: u32,
) -> Result<(usize, usize)> {
    let width = query.len();
    let row = forward
        .token_table()
        .get(token as usize * width..(token as usize + 1) * width)
        .ok_or_else(|| invalid("anchored candidate token row differs"))?;
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
    Ok((agreement, rotated))
}

fn choose_unanchored<'a>(
    candidates: &'a [AnchoredCandidate],
    ranker: &AnchoredCandidateRanker,
) -> Option<&'a AnchoredCandidate> {
    candidates
        .iter()
        .min_by_key(|candidate| (Reverse(ranker.score(&candidate.features)), candidate.token))
}

fn choose_anchored<'a>(
    candidates: &'a [AnchoredCandidate],
    ranker: &AnchoredCandidateRanker,
    baseline: u32,
) -> Option<&'a AnchoredCandidate> {
    let base = candidates
        .iter()
        .find(|candidate| candidate.token == baseline)?;
    let contender = choose_unanchored(candidates, ranker)?;
    if ranker.score(&contender.features) > ranker.score(&base.features) {
        Some(contender)
    } else {
        Some(base)
    }
}

enum CandidateChoice<'a> {
    Stop,
    Token(&'a AnchoredCandidate),
}

fn choose_generated<'a>(
    candidates: &'a [AnchoredCandidate],
    arm: AnchoredCandidateArm,
    ranker: &AnchoredCandidateRanker,
    baseline: u32,
) -> Option<CandidateChoice<'a>> {
    if baseline == 0 {
        return Some(CandidateChoice::Stop);
    }
    match arm {
        AnchoredCandidateArm::Trained
        | AnchoredCandidateArm::Shuffled
        | AnchoredCandidateArm::GeometryOnly => {
            choose_anchored(candidates, ranker, baseline).map(CandidateChoice::Token)
        }
        AnchoredCandidateArm::Unanchored => {
            choose_unanchored(candidates, ranker).map(CandidateChoice::Token)
        }
    }
}

fn fit_anchored(
    groups: &[CandidateGroup],
    seed: u64,
    shuffled: bool,
) -> Result<AnchoredCandidateRanker> {
    let mut ranker = AnchoredCandidateRanker::zero();
    for _ in 0..ranker.epochs {
        for group in groups {
            let target = if shuffled {
                shuffled_target(group, seed)?
            } else {
                group.target
            };
            if target == group.baseline {
                continue;
            }
            let baseline = group
                .candidates
                .iter()
                .find(|candidate| candidate.token == group.baseline)
                .ok_or_else(|| invalid("anchored candidate baseline is missing"))?;
            let target_candidate = group
                .candidates
                .iter()
                .find(|candidate| candidate.token == target)
                .ok_or_else(|| invalid("anchored candidate training target differs"))?;
            if ranker.score(&target_candidate.features) <= ranker.score(&baseline.features) {
                update_weights(&mut ranker, target_candidate, baseline);
            }
        }
    }
    Ok(ranker)
}

fn fit_unanchored(groups: &[CandidateGroup], seed: u64) -> Result<AnchoredCandidateRanker> {
    let _ = seed;
    let mut ranker = AnchoredCandidateRanker::zero();
    for _ in 0..ranker.epochs {
        for group in groups {
            let chosen = choose_unanchored(&group.candidates, &ranker)
                .ok_or_else(|| invalid("anchored candidate training group is empty"))?;
            if chosen.token == group.target {
                continue;
            }
            let target = group
                .candidates
                .iter()
                .find(|candidate| candidate.token == group.target)
                .ok_or_else(|| invalid("unanchored candidate training target differs"))?;
            update_weights(&mut ranker, target, chosen);
        }
    }
    Ok(ranker)
}

fn update_weights(
    ranker: &mut AnchoredCandidateRanker,
    target: &AnchoredCandidate,
    chosen: &AnchoredCandidate,
) {
    for index in 0..ANCHORED_CANDIDATE_FEATURES {
        let update =
            i64::from(target.features[index] - chosen.features[index]) * ranker.learning_rate;
        ranker.weights[index] =
            (ranker.weights[index] + update).clamp(-ranker.weight_limit, ranker.weight_limit);
    }
}

fn shuffled_target(group: &CandidateGroup, seed: u64) -> Result<u32> {
    if group.candidates.len() < 2 {
        return Err(invalid("anchored candidate shuffle needs two candidates"));
    }
    let target_index = group
        .candidates
        .iter()
        .position(|candidate| candidate.token == group.target)
        .ok_or_else(|| invalid("anchored candidate shuffle target differs"))?;
    let shift = 1 + seed as usize % (group.candidates.len() - 1);
    Ok(group.candidates[(target_index + shift) % group.candidates.len()].token)
}

fn summarize(
    arm: AnchoredCandidateArm,
    rows: &[CopyKeyReplyRow],
    steps: &[crate::GeometricCandidateStep],
) -> Result<AnchoredCandidateReading> {
    let mut reading = AnchoredCandidateReading {
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
        baseline_kept_decisions: 0,
        baseline_overrides: 0,
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
        reading.baseline_kept_decisions += u64::from(step.selected_token == step.longest_token);
        reading.baseline_overrides += u64::from(step.selected_token != step.longest_token);
        if let Some(depth) = step.selected_exact_depth {
            let slot = reading
                .selected_depth_counts
                .get_mut(depth - 1)
                .ok_or_else(|| invalid("anchored candidate selected depth differs"))?;
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

fn decision_hash(steps: &[crate::GeometricCandidateStep]) -> String {
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

    fn candidate(token: u32, depth: usize, agreement: i32, longest: bool) -> AnchoredCandidate {
        AnchoredCandidate {
            token,
            exact_depth: (depth > 0).then_some(depth),
            writer_rank: None,
            features: [
                1,
                depth as i32,
                agreement,
                agreement,
                0,
                0,
                0,
                0,
                i32::from(longest),
                0,
                0,
                0,
            ],
        }
    }

    #[test]
    fn expanded_cells_add_deterministic_candidates_without_changing_the_served_eight() {
        let artifact = artifact();
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        forward.mark_reply_start();
        let step = forward.step(3, CpuFbsHead::Writer).unwrap();
        let served = forward
            .writer_candidate_order_with_cells(&step.query_q12, 8, 129)
            .unwrap();
        let expanded = forward
            .writer_candidate_order_with_cells(&step.query_q12, 16, 257)
            .unwrap();
        assert_eq!(served.len(), 128);
        assert_eq!(expanded.len(), 256);
        let served_tokens = served
            .iter()
            .map(|candidate| candidate.0)
            .collect::<BTreeSet<_>>();
        let expanded_tokens = expanded
            .iter()
            .map(|candidate| candidate.0)
            .collect::<BTreeSet<_>>();
        assert!(served_tokens.is_subset(&expanded_tokens));
        assert!(expanded_tokens.difference(&served_tokens).count() > 0);

        let first = candidate_union(&forward, &step.query_q12, &[3], step.token, 16).unwrap();
        let repeat = candidate_union(&forward, &step.query_q12, &[3], step.token, 16).unwrap();
        assert_eq!(first, repeat);
        assert!(first.len() > 128);
        assert!(first.len() <= 256 + COPY_KEY_REPLY_MAX_DEPTH + 1);
        assert!(first.iter().all(|candidate| candidate.token != 0));
        assert!(first
            .iter()
            .any(|candidate| candidate.token == baseline_token(&[3], step.token)));
    }

    #[test]
    fn expanded_union_retains_depth_32_deepest_source_fallback_and_anchor() {
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
        let stream = [99]
            .into_iter()
            .chain(first_block)
            .chain(second_block)
            .chain([1])
            .collect::<Vec<_>>();
        let exact = exact_suffix_candidates(&stream);
        assert_eq!(exact[COPY_KEY_REPLY_MAX_DEPTH - 1], Some((1, 33)));
        let seeds = candidate_seeds(&forward, &step.query_q12, &stream, step.token, 12).unwrap();
        assert_eq!(seeds[&1].exact_depth, Some(COPY_KEY_REPLY_MAX_DEPTH));
        assert_eq!(seeds[&1].source, Some(33));
        assert_eq!(
            seeds[&baseline_token(&stream, step.token)].token,
            baseline_token(&stream, step.token)
        );

        let admitted = forward
            .writer_candidate_order_with_cells(&step.query_q12, 12, 193)
            .unwrap()
            .into_iter()
            .map(|(token, _)| token)
            .collect::<BTreeSet<_>>();
        let fallback = (1..forward.vocabulary() as u32)
            .find(|token| !admitted.contains(token))
            .unwrap();
        let seeds = candidate_seeds(&forward, &step.query_q12, &[3], fallback, 12).unwrap();
        assert_eq!(seeds[&fallback].exact_depth, None);
        assert_eq!(seeds[&fallback].writer_rank, None);
    }

    #[test]
    fn baseline_relative_features_are_bounded_and_identify_the_anchor() {
        let artifact = artifact();
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        forward.mark_reply_start();
        let step = forward.step(3, CpuFbsHead::Writer).unwrap();
        let candidates =
            candidate_union(&forward, &step.query_q12, &[3, 4, 3], step.token, 8).unwrap();
        let baseline = baseline_token(&[3, 4, 3], step.token);
        for candidate in &candidates {
            assert_eq!(candidate.features[0], 1);
            assert!((0..=32).contains(&candidate.features[1]));
            assert!((0..=32).contains(&candidate.features[2]));
            assert!((0..=32).contains(&candidate.features[3]));
            assert!((0..=32).contains(&candidate.features[4]));
            assert!((0..=16).contains(&candidate.features[5]));
            assert!((0..=1).contains(&candidate.features[6]));
            assert!((0..=32).contains(&candidate.features[7]));
            assert!((0..=1).contains(&candidate.features[8]));
            assert!((-32..=32).contains(&candidate.features[9]));
            assert!((-32..=32).contains(&candidate.features[10]));
            assert!((-32..=32).contains(&candidate.features[11]));
        }
        let anchor = candidates
            .iter()
            .find(|candidate| candidate.token == baseline)
            .unwrap();
        assert_eq!(anchor.features[8], 1);
        assert_eq!(anchor.features[9..], [0, 0, 0]);
    }

    #[test]
    fn anchored_choice_keeps_longest_on_ties_and_negative_margins() {
        let baseline = candidate(7, 32, 20, true);
        let challenger = candidate(9, 1, 20, false);
        let tied = [challenger.clone(), baseline.clone()];
        assert_eq!(
            choose_anchored(&tied, &AnchoredCandidateRanker::zero(), baseline.token)
                .unwrap()
                .token,
            baseline.token
        );

        let mut ranker = AnchoredCandidateRanker::zero();
        ranker.weights[2] = 1;
        let mut worse = challenger;
        worse.features[2] = 19;
        let negative = [worse, baseline.clone()];
        assert_eq!(
            choose_anchored(&negative, &ranker, baseline.token)
                .unwrap()
                .token,
            baseline.token
        );

        let mut better = candidate(9, 1, 21, false);
        better.features[2] = 21;
        let positive = [better, baseline.clone()];
        assert_eq!(
            choose_anchored(&positive, &ranker, baseline.token)
                .unwrap()
                .token,
            9
        );

        for arm in [
            AnchoredCandidateArm::Trained,
            AnchoredCandidateArm::Shuffled,
            AnchoredCandidateArm::GeometryOnly,
            AnchoredCandidateArm::Unanchored,
        ] {
            assert!(matches!(
                choose_generated(&positive, arm, &ranker, 0),
                Some(CandidateChoice::Stop)
            ));
        }
    }

    #[test]
    fn anchored_fit_skips_baseline_correct_groups_and_learns_relative_updates() {
        let baseline = candidate(7, 32, 20, true);
        let target = candidate(9, 1, 21, false);
        let correct = CandidateGroup {
            row: 0,
            generated_step: 0,
            target: 7,
            baseline: 7,
            candidates: vec![baseline.clone(), target.clone()],
        };
        assert_eq!(
            fit_anchored(std::slice::from_ref(&correct), 1, false).unwrap(),
            AnchoredCandidateRanker::zero()
        );

        let wrong = CandidateGroup {
            row: 1,
            target: 9,
            ..correct
        };
        let ranker = fit_anchored(std::slice::from_ref(&wrong), 1, false).unwrap();
        assert_ne!(ranker, AnchoredCandidateRanker::zero());
        assert_eq!(
            choose_anchored(&wrong.candidates, &ranker, 7)
                .unwrap()
                .token,
            9
        );
    }

    #[test]
    fn shuffled_targets_are_deterministic_and_non_vacuous() {
        let group = CandidateGroup {
            row: 0,
            generated_step: 0,
            target: 3,
            baseline: 7,
            candidates: vec![
                candidate(3, 3, 30, false),
                candidate(5, 1, 10, false),
                candidate(7, 32, 20, true),
            ],
        };
        let first = shuffled_target(&group, 0).unwrap();
        let repeat = shuffled_target(&group, 0).unwrap();
        assert_eq!(first, repeat);
        assert_ne!(first, group.target);
        let ranker = fit_anchored(std::slice::from_ref(&group), 0, true).unwrap();
        assert_ne!(ranker, AnchoredCandidateRanker::zero());
        assert_eq!(
            ranker,
            fit_anchored(std::slice::from_ref(&group), 0, true).unwrap()
        );
    }

    #[test]
    fn weighted_digest_binds_every_feature() {
        let ranker = AnchoredCandidateRanker::zero();
        let digest = ranker.weights_sha256();
        for index in 0..ANCHORED_CANDIDATE_FEATURES {
            let mut changed = ranker.clone();
            changed.weights[index] += 1;
            assert_ne!(changed.weights_sha256(), digest);
        }
    }

    #[test]
    fn registered_calibration_floors_are_exact() {
        let passing = ExpandedCandidateCalibration {
            cells: 12,
            writer_candidate_limit: 192,
            writer_candidate_rows: 10,
            min_writer_candidates_per_group: 1,
            max_writer_candidates_per_group: 2,
            generated_rows: 400,
            target_present_groups: ANCHORED_CANDIDATE_MIN_GROUPS as u64,
            target_values: ANCHORED_CANDIDATE_MIN_TARGETS as u64,
            baseline_correct_groups: 1_792,
            baseline_wrong_groups: ANCHORED_CANDIDATE_MIN_BASELINE_WRONG as u64,
            candidate_rows: 1,
        };
        assert!(passing.clears_floors());

        for below in [
            ExpandedCandidateCalibration {
                target_present_groups: ANCHORED_CANDIDATE_MIN_GROUPS as u64 - 1,
                ..passing.clone()
            },
            ExpandedCandidateCalibration {
                target_values: ANCHORED_CANDIDATE_MIN_TARGETS as u64 - 1,
                ..passing.clone()
            },
            ExpandedCandidateCalibration {
                baseline_wrong_groups: ANCHORED_CANDIDATE_MIN_BASELINE_WRONG as u64 - 1,
                ..passing
            },
        ] {
            assert!(!below.clears_floors());
        }
    }

    #[test]
    fn generated_replies_repeat_and_ignore_reference_content() {
        let artifact = artifact();
        let first_sample = MemoryWeightSample {
            prompt: vec![3, 4, 5, 4, 5, 4],
            reply: vec![5, 4, 5, 4, 6, 7],
        };
        let second_sample = MemoryWeightSample {
            reply: vec![6, 7, 8, 9, 10, 11],
            ..first_sample.clone()
        };
        let ranker = AnchoredCandidateRanker::geometry_only();
        let first = evaluate_anchored_candidate_replies(
            &artifact,
            std::slice::from_ref(&first_sample),
            8,
            AnchoredCandidateArm::GeometryOnly,
            &ranker,
            8,
        )
        .unwrap();
        let repeat = evaluate_anchored_candidate_replies(
            &artifact,
            std::slice::from_ref(&first_sample),
            8,
            AnchoredCandidateArm::GeometryOnly,
            &ranker,
            8,
        )
        .unwrap();
        let changed = evaluate_anchored_candidate_replies(
            &artifact,
            std::slice::from_ref(&second_sample),
            8,
            AnchoredCandidateArm::GeometryOnly,
            &ranker,
            8,
        )
        .unwrap();
        assert_eq!(first.steps, repeat.steps);
        assert_eq!(first.reply_sha256, repeat.reply_sha256);
        assert_eq!(first.decision_sha256, repeat.decision_sha256);
        assert_eq!(first.steps, changed.steps);
        assert_eq!(first.reply_sha256, changed.reply_sha256);
        assert!(first.steps.iter().all(|step| step.selected_token != 0));
    }
}
