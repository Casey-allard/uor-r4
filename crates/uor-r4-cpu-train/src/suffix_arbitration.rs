//! Same-surface arbitration between local exact copy and a confident global
//! suffix memory trained over prompt-plus-reply histories.
//!
//! The treatment accepts a global continuation only when its matched suffix is
//! deeper than the local exact-copy candidate or no local candidate exists.
//! Ties remain local. Stopping is the independent true reply-boundary memory.

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

pub const SUFFIX_ARBITRATION_MIN_OBSERVATIONS: u64 = 2;
pub const SUFFIX_ARBITRATION_SHALLOW_LOCAL_DEPTH: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuffixArbitrationArm {
    DeeperFullSequence32,
    DeeperFullSequenceShuffled,
    ShallowLocalGlobal4,
    FullSequence32Only,
    LocalLongest32Boundary,
    WriterBoundary,
    ReferenceLengthOracle,
}

impl SuffixArbitrationArm {
    pub const EVALUATION_ARMS: [Self; 7] = [
        Self::DeeperFullSequence32,
        Self::DeeperFullSequenceShuffled,
        Self::ShallowLocalGlobal4,
        Self::FullSequence32Only,
        Self::LocalLongest32Boundary,
        Self::WriterBoundary,
        Self::ReferenceLengthOracle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::DeeperFullSequence32 => "deeper_full_sequence_32",
            Self::DeeperFullSequenceShuffled => "deeper_full_sequence_shuffled",
            Self::ShallowLocalGlobal4 => "shallow_local_global_4",
            Self::FullSequence32Only => "full_sequence_32_only",
            Self::LocalLongest32Boundary => "local_longest_32_boundary",
            Self::WriterBoundary => "writer_boundary",
            Self::ReferenceLengthOracle => "reference_length_oracle",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SuffixArbitrationStep {
    pub row: usize,
    pub generated_step: usize,
    pub writer_token: u32,
    pub local_token: Option<u32>,
    pub global_token: Option<u32>,
    pub selected_token: u32,
    pub used_global: bool,
    pub local_copy_depth: Option<usize>,
    pub global_depth: Option<usize>,
    pub global_observations: u64,
    pub global_successor_count: u64,
    pub global_distinct_targets: u64,
    pub boundary_depth: Option<usize>,
    pub boundary_terminal: u64,
    pub boundary_nonterminal: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SuffixArbitrationReading {
    pub arm: SuffixArbitrationArm,
    pub replies: u64,
    pub reference_reachable: u64,
    pub exact_reference: u64,
    pub bos_complete: u64,
    pub length_complete: u64,
    pub retrieved_endpoint_complete: u64,
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
    pub boundary_stops: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TimedSuffixArbitrationReading {
    pub reading: SuffixArbitrationReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<SuffixArbitrationStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SuffixArbitrationChanges {
    pub common_steps: u64,
    pub changed_steps: u64,
    pub divergent_rows: u64,
    pub first_changed_step: Option<(usize, usize)>,
}

#[allow(clippy::too_many_arguments)]
pub fn generate_suffix_arbitration_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    reference_length: usize,
    arm: SuffixArbitrationArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    boundary: &ReplyBoundaryMemory,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<SuffixArbitrationStep>)> {
    generate_suffix_arbitration_reply_with_endpoint(
        artifact,
        prompt,
        cap,
        Endpoint::Boundary(reference_length),
        arm,
        suffix_memory,
        boundary,
        seed,
        row,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn generate_suffix_arbitration_reply_with_aligned_endpoint(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    retrieved_reply: &[u32],
    arm: SuffixArbitrationArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    boundary: &ReplyBoundaryMemory,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<SuffixArbitrationStep>)> {
    let branches = vec![retrieved_reply.to_vec()];
    generate_suffix_arbitration_reply_with_branch_endpoint(
        artifact,
        prompt,
        cap,
        &branches,
        arm,
        suffix_memory,
        boundary,
        seed,
        row,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn generate_suffix_arbitration_reply_with_branch_endpoint(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    retrieved_replies: &[Vec<u32>],
    arm: SuffixArbitrationArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    boundary: &ReplyBoundaryMemory,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<SuffixArbitrationStep>)> {
    generate_suffix_arbitration_reply_with_endpoint(
        artifact,
        prompt,
        cap,
        Endpoint::AlignedBranches(retrieved_replies),
        arm,
        suffix_memory,
        boundary,
        seed,
        row,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn generate_suffix_arbitration_reply_to_length(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    length: usize,
    arm: SuffixArbitrationArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<SuffixArbitrationStep>)> {
    generate_suffix_arbitration_reply_with_endpoint(
        artifact,
        prompt,
        cap,
        Endpoint::Length(length),
        arm,
        suffix_memory,
        &ReplyBoundaryMemory::default(),
        seed,
        row,
    )
}

#[derive(Clone, Copy)]
enum Endpoint<'a> {
    Boundary(usize),
    Length(usize),
    AlignedBranches(&'a [Vec<u32>]),
}

#[allow(clippy::too_many_arguments)]
fn generate_suffix_arbitration_reply_with_endpoint(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    endpoint: Endpoint<'_>,
    arm: SuffixArbitrationArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    boundary: &ReplyBoundaryMemory,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<SuffixArbitrationStep>)> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("suffix-arbitration forward: {error}")))?;
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "suffix-arbitration prompt, cap or artifact context is invalid",
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
            .map_err(|error| invalid(format!("suffix-arbitration prompt step: {error}")))?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some(step.token);
        }
    }
    let mut writer_token = next.ok_or_else(|| invalid("suffix-arbitration prompt is empty"))?;
    let mut tokens = Vec::with_capacity(cap);
    let mut steps = Vec::with_capacity(cap);

    loop {
        let suppress_boundary = match endpoint {
            Endpoint::Length(length) => {
                if length == 0 || length > cap {
                    return Err(invalid("suffix-arbitration stored length exceeds cap"));
                }
                if tokens.len() == length {
                    return Ok((
                        GeneratedReply {
                            tokens,
                            stop: "length",
                        },
                        steps,
                    ));
                }
                true
            }
            Endpoint::AlignedBranches(retrieved) => {
                if retrieved.is_empty()
                    || retrieved
                        .iter()
                        .any(|reply| reply.is_empty() || reply.len() > cap)
                {
                    return Err(invalid("suffix-arbitration retrieved branches are invalid"));
                }
                let mut extends = false;
                for reply in retrieved {
                    if reply.get(..tokens.len()) == Some(tokens.as_slice()) {
                        if reply.len() == tokens.len() {
                            return Ok((
                                GeneratedReply {
                                    tokens,
                                    stop: "retrieved_endpoint",
                                },
                                steps,
                            ));
                        }
                        extends = true;
                    }
                }
                extends
            }
            Endpoint::Boundary(_) => false,
        };
        if !suppress_boundary {
            let boundary_hit = match endpoint {
                Endpoint::Length(_) => unreachable!("stored length suppresses boundary"),
                Endpoint::Boundary(reference_length)
                    if arm == SuffixArbitrationArm::ReferenceLengthOracle =>
                {
                    (tokens.len() == reference_length).then_some(
                        crate::reply_boundary::ReplyBoundaryHit {
                            depth: 0,
                            terminal: 1,
                            nonterminal: 0,
                        },
                    )
                }
                Endpoint::Boundary(_) | Endpoint::AlignedBranches(_) => {
                    boundary.stop_decision(&tokens)
                }
            };
            if let Some(hit) = boundary_hit {
                steps.push(SuffixArbitrationStep {
                    row,
                    generated_step: tokens.len(),
                    writer_token,
                    local_token: None,
                    global_token: None,
                    selected_token: POINT_KEY_BOS,
                    used_global: false,
                    local_copy_depth: None,
                    global_depth: None,
                    global_observations: 0,
                    global_successor_count: 0,
                    global_distinct_targets: 0,
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
        }

        let local = matches!(
            arm,
            SuffixArbitrationArm::DeeperFullSequence32
                | SuffixArbitrationArm::DeeperFullSequenceShuffled
                | SuffixArbitrationArm::ShallowLocalGlobal4
                | SuffixArbitrationArm::LocalLongest32Boundary
                | SuffixArbitrationArm::ReferenceLengthOracle
        )
        .then(|| {
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

        let global = if matches!(
            arm,
            SuffixArbitrationArm::DeeperFullSequence32
                | SuffixArbitrationArm::DeeperFullSequenceShuffled
                | SuffixArbitrationArm::ShallowLocalGlobal4
                | SuffixArbitrationArm::FullSequence32Only
        ) {
            suffix_memory
                .ok_or_else(|| invalid("suffix-arbitration arm has no table"))?
                .predict_confident(
                    &stream,
                    SUFFIX_MEMORY_MAX_DEPTH,
                    SUFFIX_ARBITRATION_MIN_OBSERVATIONS,
                )
        } else {
            None
        };

        let (selected_token, used_global) = choose_token(arm, local.as_ref(), global, writer_token);

        steps.push(SuffixArbitrationStep {
            row,
            generated_step: tokens.len(),
            writer_token,
            local_token: local.as_ref().map(|decision| decision.token),
            global_token: global.map(|prediction| prediction.token),
            selected_token,
            used_global,
            local_copy_depth: local_depth,
            global_depth: global.map(|prediction| prediction.depth),
            global_observations: global.map_or(0, |prediction| prediction.observations),
            global_successor_count: global.map_or(0, |prediction| prediction.successor_count),
            global_distinct_targets: global.map_or(0, |prediction| prediction.distinct_targets),
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
            .map_err(|error| invalid(format!("suffix-arbitration generated step: {error}")))?;
        writer_token = step.token;
    }
}

fn choose_token(
    arm: SuffixArbitrationArm,
    local: Option<&crate::copy_key_reply::CopyKeyReplyDecision>,
    global: Option<crate::suffix_memory::SuffixMemoryPrediction>,
    writer_token: u32,
) -> (u32, bool) {
    let local_depth = local.and_then(|decision| decision.depth);
    let use_global = match (arm, global) {
        (
            SuffixArbitrationArm::DeeperFullSequence32
            | SuffixArbitrationArm::DeeperFullSequenceShuffled,
            Some(global),
        ) => local_depth.is_none_or(|depth| global.depth > depth),
        (SuffixArbitrationArm::ShallowLocalGlobal4, Some(_)) => {
            local_depth.is_none_or(|depth| depth <= SUFFIX_ARBITRATION_SHALLOW_LOCAL_DEPTH)
        }
        (SuffixArbitrationArm::FullSequence32Only, Some(_)) => true,
        _ => false,
    };
    let token = if use_global {
        global
            .expect("use_global requires a global prediction")
            .token
    } else if local_depth.is_some() {
        local.expect("local depth requires a local decision").token
    } else {
        writer_token
    };
    (token, use_global)
}

pub fn evaluate_suffix_arbitration_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: SuffixArbitrationArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    boundary: &ReplyBoundaryMemory,
    seed: u64,
) -> Result<TimedSuffixArbitrationReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("suffix-arbitration reply inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("suffix-arbitration context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply).to_vec();
            if sample.prompt.len() > context || reference.len() > cap {
                return Err(invalid(
                    "suffix-arbitration sample exceeds its context or cap",
                ));
            }
            let (generated, steps) = generate_suffix_arbitration_reply(
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
            let premature_bos = generated.stop == "bos" && generated.tokens.len() < reference.len();
            let non_premature_bos = generated.stop == "bos" && !premature_bos;
            let local_decisions = steps
                .iter()
                .filter(|step| {
                    step.boundary_depth.is_none()
                        && !step.used_global
                        && step.local_copy_depth.is_some()
                })
                .count() as u64;
            let row_reading = CopyKeyReplyRow {
                row,
                reachable,
                exact_reference: generated.tokens == reference,
                bos_complete: generated.stop == "bos",
                opening_correct,
                premature_bos,
                non_premature_bos,
                generated_tokens: generated.tokens.len() as u64,
                reference_tokens: reference.len() as u64,
                correct_body_positions,
                copy_decisions: local_decisions,
                fallback_decisions: steps.len() as u64 - local_decisions,
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
    Ok(TimedSuffixArbitrationReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

pub fn evaluate_suffix_arbitration_replies_to_length(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    lengths: &[usize],
    cap: usize,
    arm: SuffixArbitrationArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    seed: u64,
) -> Result<TimedSuffixArbitrationReading> {
    if samples.is_empty() || lengths.len() != samples.len() || cap == 0 || cap > 64 {
        return Err(invalid("suffix-arbitration length inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("suffix-arbitration context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .zip(lengths)
        .enumerate()
        .map(|(row, (sample, &length))| {
            let reference = normalized_reference(&sample.reply).to_vec();
            if sample.prompt.len() > context || reference.len() > cap || length == 0 || length > cap
            {
                return Err(invalid(
                    "suffix-arbitration sample exceeds its context, cap or stored length",
                ));
            }
            let (generated, steps) = generate_suffix_arbitration_reply_to_length(
                artifact,
                &sample.prompt,
                cap,
                length,
                arm,
                suffix_memory,
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
            let local_decisions = steps
                .iter()
                .filter(|step| !step.used_global && step.local_copy_depth.is_some())
                .count() as u64;
            let row_reading = CopyKeyReplyRow {
                row,
                reachable,
                exact_reference: generated.tokens == reference,
                bos_complete: generated.stop == "bos",
                opening_correct,
                premature_bos,
                non_premature_bos,
                generated_tokens: generated.tokens.len() as u64,
                reference_tokens: reference.len() as u64,
                correct_body_positions,
                copy_decisions: local_decisions,
                fallback_decisions: steps.len() as u64 - local_decisions,
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
    Ok(TimedSuffixArbitrationReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_suffix_arbitration_replies_with_aligned_endpoint(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    retrieved_replies: &[Vec<u32>],
    cap: usize,
    arm: SuffixArbitrationArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    boundary: &ReplyBoundaryMemory,
    seed: u64,
) -> Result<TimedSuffixArbitrationReading> {
    let branches = retrieved_replies
        .iter()
        .map(|reply| vec![reply.clone()])
        .collect::<Vec<_>>();
    evaluate_suffix_arbitration_replies_with_branch_endpoint(
        artifact,
        samples,
        &branches,
        cap,
        arm,
        suffix_memory,
        boundary,
        seed,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_suffix_arbitration_replies_with_branch_endpoint(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    retrieved_replies: &[Vec<Vec<u32>>],
    cap: usize,
    arm: SuffixArbitrationArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    boundary: &ReplyBoundaryMemory,
    seed: u64,
) -> Result<TimedSuffixArbitrationReading> {
    if samples.is_empty() || retrieved_replies.len() != samples.len() || cap == 0 || cap > 64 {
        return Err(invalid(
            "suffix-arbitration retrieved endpoint inputs differ",
        ));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("suffix-arbitration context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .zip(retrieved_replies)
        .enumerate()
        .map(|(row, (sample, retrieved_branches))| {
            let reference = normalized_reference(&sample.reply).to_vec();
            if sample.prompt.len() > context
                || reference.len() > cap
                || retrieved_branches.is_empty()
                || retrieved_branches
                    .iter()
                    .any(|retrieved| retrieved.is_empty() || retrieved.len() > cap)
            {
                return Err(invalid(
                    "suffix-arbitration sample or retrieved branches exceed their bound",
                ));
            }
            let (generated, steps) = generate_suffix_arbitration_reply_with_branch_endpoint(
                artifact,
                &sample.prompt,
                cap,
                retrieved_branches,
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
            let premature_bos = generated.stop == "bos" && generated.tokens.len() < reference.len();
            let non_premature_bos = generated.stop == "bos" && !premature_bos;
            let local_decisions = steps
                .iter()
                .filter(|step| !step.used_global && step.local_copy_depth.is_some())
                .count() as u64;
            let row_reading = CopyKeyReplyRow {
                row,
                reachable,
                exact_reference: generated.tokens == reference,
                bos_complete: generated.stop == "bos",
                opening_correct,
                premature_bos,
                non_premature_bos,
                generated_tokens: generated.tokens.len() as u64,
                reference_tokens: reference.len() as u64,
                correct_body_positions,
                copy_decisions: local_decisions,
                fallback_decisions: steps.len() as u64 - local_decisions,
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
    Ok(TimedSuffixArbitrationReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

pub fn compare_suffix_arbitration_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<crate::copy_key_reply::CopyKeyReplyComparison> {
    compare_copy_key_reply_rows(candidate, control, metric)
}

pub fn suffix_arbitration_changes(
    candidate: &TimedSuffixArbitrationReading,
    control: &TimedSuffixArbitrationReading,
) -> SuffixArbitrationChanges {
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
    SuffixArbitrationChanges {
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
    arm: SuffixArbitrationArm,
    rows: &[CopyKeyReplyRow],
    steps: &[SuffixArbitrationStep],
) -> Result<SuffixArbitrationReading> {
    let mut reading = SuffixArbitrationReading {
        arm,
        replies: rows.len() as u64,
        reference_reachable: 0,
        exact_reference: 0,
        bos_complete: 0,
        length_complete: 0,
        retrieved_endpoint_complete: 0,
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
        boundary_stops: 0,
    };
    for row in rows {
        reading.reference_reachable += u64::from(row.reachable);
        reading.exact_reference += u64::from(row.reachable && row.exact_reference);
        reading.bos_complete += u64::from(row.reachable && row.bos_complete);
        reading.length_complete += u64::from(row.reachable && row.stop == "length");
        reading.retrieved_endpoint_complete +=
            u64::from(row.reachable && row.stop == "retrieved_endpoint");
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
        if step.boundary_depth.is_some() {
            reading.boundary_stops = checked_add(reading.boundary_stops, 1)?;
        } else if step.used_global {
            reading.global_decisions = checked_add(reading.global_decisions, 1)?;
        } else if step.local_copy_depth.is_some() {
            reading.local_decisions = checked_add(reading.local_decisions, 1)?;
        } else {
            reading.writer_decisions = checked_add(reading.writer_decisions, 1)?;
        }
    }
    Ok(reading)
}

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid("suffix-arbitration counter overflows"))
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

fn decision_hash(steps: &[SuffixArbitrationStep]) -> String {
    let mut hasher = blake3::Hasher::new();
    for step in steps {
        hasher.update(&(step.row as u64).to_le_bytes());
        hasher.update(&(step.generated_step as u64).to_le_bytes());
        hasher.update(&step.writer_token.to_le_bytes());
        hasher.update(&step.local_token.unwrap_or(u32::MAX).to_le_bytes());
        hasher.update(&step.global_token.unwrap_or(u32::MAX).to_le_bytes());
        hasher.update(&step.selected_token.to_le_bytes());
        hasher.update(&[u8::from(step.used_global)]);
        hasher.update(&(step.local_copy_depth.unwrap_or(0) as u16).to_le_bytes());
        hasher.update(&(step.global_depth.unwrap_or(0) as u16).to_le_bytes());
        hasher.update(&step.global_observations.to_le_bytes());
        hasher.update(&step.global_successor_count.to_le_bytes());
        hasher.update(&step.global_distinct_targets.to_le_bytes());
        hasher.update(&(step.boundary_depth.unwrap_or(0) as u16).to_le_bytes());
        hasher.update(&step.boundary_terminal.to_le_bytes());
        hasher.update(&step.boundary_nonterminal.to_le_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reply_boundary::derive_reply_boundary_memory;
    use crate::suffix_memory::{derive_full_sequence_suffix_content_memory, shuffle_suffix_memory};
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
            198,
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
        derive_reply_boundary_memory(&[sample(&[9], &[40, 10, 20, 30, 1])]).unwrap()
    }

    #[test]
    fn deeper_global_replaces_shallow_local_but_ties_stay_local() {
        let stream = [10, 20, 30, 40, 10, 20, 30];
        let local = copy_key_reply_decision(&stream, 99, CopyKeyReplyArm::Longest32To1, 198, 0, 0);
        let local_depth = local.depth.unwrap();
        let global = |depth, token| crate::suffix_memory::SuffixMemoryPrediction {
            depth,
            token,
            observations: 2,
            successor_count: 2,
            distinct_targets: 1,
        };
        assert_eq!(
            choose_token(
                SuffixArbitrationArm::DeeperFullSequence32,
                Some(&local),
                Some(global(local_depth + 1, 77)),
                99,
            )
            .0,
            77
        );
        assert_eq!(
            choose_token(
                SuffixArbitrationArm::DeeperFullSequence32,
                Some(&local),
                Some(global(local_depth, 77)),
                99,
            )
            .0,
            local.token
        );
    }

    #[test]
    fn shallow_control_can_replace_local_at_depth_four() {
        let local = crate::copy_key_reply::CopyKeyReplyDecision {
            token: 55,
            depth: Some(4),
            matches: [None; SUFFIX_MEMORY_MAX_DEPTH],
            sign_hamming: None,
        };
        let global = crate::suffix_memory::SuffixMemoryPrediction {
            depth: 1,
            token: 77,
            observations: 2,
            successor_count: 2,
            distinct_targets: 1,
        };
        assert_eq!(
            choose_token(
                SuffixArbitrationArm::ShallowLocalGlobal4,
                Some(&local),
                Some(global),
                99,
            )
            .0,
            77
        );
        assert_eq!(
            choose_token(
                SuffixArbitrationArm::DeeperFullSequence32,
                Some(&local),
                Some(global),
                99,
            )
            .0,
            55
        );
    }

    #[test]
    fn full_sequence_table_and_shuffle_produce_different_complete_trajectories() {
        let training = [
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[41, 11, 21, 31]),
        ];
        let table = derive_full_sequence_suffix_content_memory(&training).unwrap();
        let evaluation = [sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30, 1])];
        let true_reply = evaluate_suffix_arbitration_replies(
            &artifact(),
            &evaluation,
            16,
            SuffixArbitrationArm::DeeperFullSequence32,
            Some(&table),
            &boundary(),
            198,
        )
        .unwrap();
        assert!(true_reply.reading.global_decisions > 0);
        let shuffled_reply = (0..64)
            .find_map(|seed| {
                let (shuffled, _) = shuffle_suffix_memory(&table, seed).unwrap();
                let reading = evaluate_suffix_arbitration_replies(
                    &artifact(),
                    &evaluation,
                    16,
                    SuffixArbitrationArm::DeeperFullSequenceShuffled,
                    Some(&shuffled),
                    &boundary(),
                    seed,
                )
                .unwrap();
                (reading.reply_sha256 != true_reply.reply_sha256).then_some(reading)
            })
            .expect("shuffle needs to change a complete trajectory");
        assert_ne!(true_reply.decision_sha256, shuffled_reply.decision_sha256);
    }

    #[test]
    fn stored_length_completes_exactly_without_a_boundary_signal() {
        let training = [
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[41, 11, 21, 31]),
        ];
        let table = derive_full_sequence_suffix_content_memory(&training).unwrap();
        let evaluation = [sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30])];
        let reading = evaluate_suffix_arbitration_replies_to_length(
            &artifact(),
            &evaluation,
            &[4],
            16,
            SuffixArbitrationArm::DeeperFullSequence32,
            Some(&table),
            198,
        )
        .unwrap();
        assert_eq!(reading.reading.reference_reachable, 1);
        assert_eq!(reading.reading.length_complete, 1);
        assert_eq!(reading.reading.bos_complete, 0);
        assert_eq!(reading.reading.exact_reference, 1);
        assert_eq!(reading.rows[0].stop, "length");
    }

    #[test]
    fn stored_length_refuses_zero_and_a_budget_above_the_cap() {
        let artifact = artifact();
        let evaluation = [sample(&[10, 11, 12], &[20])];
        for length in [0, 5] {
            assert!(evaluate_suffix_arbitration_replies_to_length(
                &artifact,
                &evaluation,
                &[length],
                4,
                SuffixArbitrationArm::WriterBoundary,
                None,
                198,
            )
            .is_err());
        }
    }

    #[test]
    fn aligned_retrieved_endpoint_stops_without_injecting_content() {
        let training = [
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[41, 11, 21, 31]),
        ];
        let table = derive_full_sequence_suffix_content_memory(&training).unwrap();
        let evaluation = [sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30])];
        let reading = evaluate_suffix_arbitration_replies_with_aligned_endpoint(
            &artifact(),
            &evaluation,
            &[vec![40, 10, 20, 30]],
            16,
            SuffixArbitrationArm::DeeperFullSequence32,
            Some(&table),
            &ReplyBoundaryMemory::default(),
            225,
        )
        .unwrap();
        assert_eq!(reading.reading.length_complete, 0);
        assert_eq!(reading.reading.retrieved_endpoint_complete, 1);
        assert_eq!(reading.reading.bos_complete, 0);
        assert_eq!(reading.reading.exact_reference, 1);
        assert_eq!(reading.rows[0].stop, "retrieved_endpoint");
    }

    #[test]
    fn aligned_retrieved_endpoint_refuses_empty_and_diverges_without_stopping() {
        let artifact = artifact();
        let evaluation = [sample(&[10, 11, 12], &[20])];
        assert!(evaluate_suffix_arbitration_replies_with_aligned_endpoint(
            &artifact,
            &evaluation,
            &[Vec::new()],
            4,
            SuffixArbitrationArm::WriterBoundary,
            None,
            &ReplyBoundaryMemory::default(),
            225,
        )
        .is_err());
        let reading = evaluate_suffix_arbitration_replies_with_aligned_endpoint(
            &artifact,
            &evaluation,
            &[vec![99]],
            4,
            SuffixArbitrationArm::WriterBoundary,
            None,
            &ReplyBoundaryMemory::default(),
            225,
        )
        .unwrap();
        assert_eq!(reading.reading.retrieved_endpoint_complete, 0);
        assert_ne!(reading.rows[0].stop, "retrieved_endpoint");
    }

    #[test]
    fn one_extending_branch_defers_boundary_until_its_complete_reply() {
        let training = [
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30]),
            sample(&[10, 20, 30, 40, 50, 60], &[41, 11, 21, 31]),
        ];
        let table = derive_full_sequence_suffix_content_memory(&training).unwrap();
        let evaluation = [sample(&[10, 20, 30, 40, 50, 60], &[40, 10, 20, 30])];
        let reading = evaluate_suffix_arbitration_replies_with_branch_endpoint(
            &artifact(),
            &evaluation,
            &[vec![vec![99], vec![40, 10, 20, 30]]],
            16,
            SuffixArbitrationArm::DeeperFullSequence32,
            Some(&table),
            &ReplyBoundaryMemory::default(),
            227,
        )
        .unwrap();
        assert_eq!(reading.reading.retrieved_endpoint_complete, 1);
        assert_eq!(reading.reading.exact_reference, 1);
        assert_eq!(reading.rows[0].stop, "retrieved_endpoint");
    }
}
