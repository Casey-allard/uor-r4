//! Learned geometric arbitration among exact-copy candidates.
//!
//! Training and evaluation use generated longest-32 trajectories. The learned
//! readout can only choose a token already present in the exact-copy candidate
//! set; otherwise the arm falls back to longest-32.

use std::collections::BTreeSet;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uor_r4_integer::stack::{reply_stop, ReplyEnd};

use crate::content_readout::{
    fit_content_readout, pack_content_signature, predict_content_target, ContentExample,
    ContentReadout, ContentReadoutTraining, ContentSignatureMode, CONTENT_MIN_TARGET_VALUES,
    CONTENT_MIN_TRAINING_ROWS,
};
use crate::copy_key_reply::{
    copy_key_reply_decision, CopyKeyReplyArm, CopyKeyReplyRow, COPY_KEY_REPLY_MAX_DEPTH,
};
use crate::reply_eval::{normalized_reference, GeneratedReply};
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const LEARNED_COPY_MAX_TRAINING_ROWS: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearnedCopyArm {
    LearnedRead32,
    ShuffledRead32,
    LookaheadRead32,
    ShuffledLookahead32,
}

impl LearnedCopyArm {
    pub fn name(self) -> &'static str {
        match self {
            Self::LearnedRead32 => "learned_read_32",
            Self::ShuffledRead32 => "shuffled_read_32",
            Self::LookaheadRead32 => "lookahead_read_32",
            Self::ShuffledLookahead32 => "shuffled_lookahead_32",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LearnedCopyStep {
    pub row: usize,
    pub generated_step: usize,
    pub writer_token: u32,
    pub longest_token: u32,
    pub selected_token: u32,
    pub selected_depth: Option<usize>,
    pub used_copy: bool,
    pub readout_target: Option<u32>,
    pub readout_distance: Option<u32>,
    pub readout_ambiguous: bool,
    pub readout_in_candidates: bool,
    pub learned_arbitrated: bool,
    pub lookahead_target: Option<u32>,
    pub lookahead_distance: Option<u32>,
    pub lookahead_ambiguous: bool,
    pub lookahead_evaluated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LearnedCopyReading {
    pub arm: LearnedCopyArm,
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
    pub learned_arbitrations: u64,
    pub learned_candidate_targets: u64,
    pub learned_distance_sum: u64,
    pub lookahead_decisions: u64,
    pub lookahead_distance_sum: u64,
    pub selected_depth_counts: [u64; COPY_KEY_REPLY_MAX_DEPTH],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedLearnedCopyReading {
    pub reading: LearnedCopyReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<LearnedCopyStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LearnedCopyTraining {
    pub readout: ContentReadoutTraining,
    pub generated_rows: u64,
    pub retained_rows: u64,
}

pub fn derive_learned_copy_readout(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
) -> Result<LearnedCopyTraining> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("learned-copy training inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("learned-copy training context: {error}")))?
        .context_length();
    let rows = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| training_rows_for_sample(artifact, sample, cap, context, row))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if rows.len() < CONTENT_MIN_TRAINING_ROWS {
        return Err(invalid("learned-copy readout has fewer than 640 rows"));
    }
    let generated_rows = rows.len();
    let rows = evenly_spaced_rows(rows, LEARNED_COPY_MAX_TRAINING_ROWS);
    let target_values = rows
        .iter()
        .map(|row| row.target)
        .collect::<BTreeSet<_>>()
        .len();
    if target_values < CONTENT_MIN_TARGET_VALUES {
        return Err(invalid(
            "learned-copy readout target diversity is too small",
        ));
    }
    let retained_rows = rows.len() as u64;
    let readout = fit_content_readout(rows, target_values, ContentSignatureMode::Code, false)?;
    Ok(LearnedCopyTraining {
        readout,
        generated_rows: generated_rows as u64,
        retained_rows,
    })
}

fn training_rows_for_sample(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    cap: usize,
    context: usize,
    row: usize,
) -> Result<Vec<ContentExample>> {
    let reference = normalized_reference(&sample.reply).to_vec();
    if sample.prompt.is_empty() || sample.prompt.len() > context || reference.len() > cap {
        return Err(invalid(
            "learned-copy training sample exceeds its context or cap",
        ));
    }
    let (mut forward, mut stream, writer, query) =
        start_reply(artifact, &sample.prompt, cap, context, "training")?;
    let mut generator = (writer, query);
    let mut generated = 0usize;
    let mut rows = Vec::with_capacity(reference.len().min(cap));
    loop {
        let longest = copy_key_reply_decision(
            &stream,
            generator.0,
            CopyKeyReplyArm::Longest32To1,
            0,
            row,
            generated,
        );
        if let Some(target) = reference.get(generated).copied() {
            rows.push(ContentExample {
                signature: pack_content_signature(&generator.1, ContentSignatureMode::Code)?,
                target,
            });
        }
        let stop = reply_stop(longest.token, generated, sample.prompt.len(), cap, context);
        if stop.is_some() {
            break;
        }
        stream.push(longest.token);
        let step = forward
            .step(longest.token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("learned-copy training step: {error}")))?;
        generator = (step.token, step.query_q12);
        generated += 1;
    }
    Ok(rows)
}

fn evenly_spaced_rows(rows: Vec<ContentExample>, limit: usize) -> Vec<ContentExample> {
    if rows.len() <= limit {
        return rows;
    }
    (0..limit)
        .map(|index| index * rows.len() / limit)
        .map(|index| rows[index].clone())
        .collect()
}

pub fn evaluate_learned_copy_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: LearnedCopyArm,
    readout: &ContentReadout,
    seed: u64,
) -> Result<TimedLearnedCopyReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("learned-copy reply inputs differ"));
    }
    if readout.mode != ContentSignatureMode::Code {
        return Err(invalid("learned-copy reply requires a modulo-code readout"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("learned-copy context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply).to_vec();
            if sample.prompt.len() > context || reference.len() > cap {
                return Err(invalid("learned-copy sample exceeds its context or cap"));
            }
            let (generated, steps) = generate_learned_copy_reply(
                artifact,
                &sample.prompt,
                cap,
                arm,
                readout,
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
            let copy_decisions = steps.iter().filter(|step| step.used_copy).count() as u64;
            let fallback_decisions = steps.len() as u64 - copy_decisions;
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
                copy_decisions,
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
    Ok(TimedLearnedCopyReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

fn generate_learned_copy_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    arm: LearnedCopyArm,
    readout: &ContentReadout,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<LearnedCopyStep>)> {
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("learned-copy forward: {error}")))?
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
            seed,
            row,
            tokens.len(),
        );
        let decision = match arm {
            LearnedCopyArm::LearnedRead32 | LearnedCopyArm::ShuffledRead32 => {
                learned_decision(&base, &generator.1, readout)?
            }
            LearnedCopyArm::LookaheadRead32 | LearnedCopyArm::ShuffledLookahead32 => {
                lookahead_decision(&base, &forward, readout)?
            }
        };
        steps.push(LearnedCopyStep {
            row,
            generated_step: tokens.len(),
            writer_token: generator.0,
            longest_token: base.token,
            selected_token: decision.token,
            selected_depth: decision.depth,
            used_copy: decision.depth.is_some(),
            readout_target: Some(decision.readout_target),
            readout_distance: Some(decision.readout_distance),
            readout_ambiguous: decision.readout_ambiguous,
            readout_in_candidates: decision.readout_in_candidates,
            learned_arbitrated: decision.learned_arbitrated,
            lookahead_target: decision.lookahead_target,
            lookahead_distance: decision.lookahead_distance,
            lookahead_ambiguous: decision.lookahead_ambiguous,
            lookahead_evaluated: decision.lookahead_evaluated,
        });
        let stop = reply_stop(decision.token, tokens.len(), prompt.len(), cap, context);
        if stop != Some(ReplyEnd::Bos) {
            tokens.push(decision.token);
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
        stream.push(decision.token);
        let step = forward
            .step(decision.token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("learned-copy generated step: {error}")))?;
        generator = (step.token, step.query_q12);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LearnedDecision {
    token: u32,
    depth: Option<usize>,
    readout_target: u32,
    readout_distance: u32,
    readout_ambiguous: bool,
    readout_in_candidates: bool,
    learned_arbitrated: bool,
    lookahead_target: Option<u32>,
    lookahead_distance: Option<u32>,
    lookahead_ambiguous: bool,
    lookahead_evaluated: bool,
}

fn learned_decision(
    base: &crate::CopyKeyReplyDecision,
    query: &[i32],
    readout: &ContentReadout,
) -> Result<LearnedDecision> {
    let signature = pack_content_signature(query, readout.mode)?;
    let prediction = predict_content_target(readout, &signature)?;
    let predicted_depth = (1..=COPY_KEY_REPLY_MAX_DEPTH)
        .rev()
        .find(|&depth| base.matches[depth - 1] == Some(prediction.target));
    let arbitrated = prediction.within_threshold && predicted_depth.is_some();
    Ok(if arbitrated {
        LearnedDecision {
            token: prediction.target,
            depth: predicted_depth,
            readout_target: prediction.target,
            readout_distance: prediction.distance,
            readout_ambiguous: prediction.ambiguous,
            readout_in_candidates: true,
            learned_arbitrated: true,
            lookahead_target: None,
            lookahead_distance: None,
            lookahead_ambiguous: false,
            lookahead_evaluated: false,
        }
    } else {
        LearnedDecision {
            token: base.token,
            depth: base.depth,
            readout_target: prediction.target,
            readout_distance: prediction.distance,
            readout_ambiguous: prediction.ambiguous,
            readout_in_candidates: predicted_depth.is_some(),
            learned_arbitrated: false,
            lookahead_target: None,
            lookahead_distance: None,
            lookahead_ambiguous: false,
            lookahead_evaluated: false,
        }
    })
}

fn lookahead_decision(
    base: &crate::CopyKeyReplyDecision,
    forward: &CpuFbsForward,
    readout: &ContentReadout,
) -> Result<LearnedDecision> {
    let mut candidates = BTreeSet::new();
    for depth in 1..=COPY_KEY_REPLY_MAX_DEPTH.min(4) {
        if let Some(token) = base.matches[depth - 1] {
            candidates.insert(token);
        }
    }
    let mut best = None;
    for token in candidates {
        let depth = (1..=COPY_KEY_REPLY_MAX_DEPTH.min(4))
            .rev()
            .find(|&depth| base.matches[depth - 1] == Some(token))
            .ok_or_else(|| invalid("lookahead candidate depth differs"))?;
        let mut next_forward = forward.clone();
        let next = next_forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("lookahead candidate step: {error}")))?;
        let signature = pack_content_signature(&next.query_q12, readout.mode)?;
        let prediction = predict_content_target(readout, &signature)?;
        let candidate = (
            prediction.distance,
            std::cmp::Reverse(depth),
            token,
            prediction.target,
            prediction.ambiguous,
            prediction.within_threshold,
        );
        if best.is_none_or(
            |current: (u32, std::cmp::Reverse<usize>, u32, u32, bool, bool)| {
                (candidate.0, candidate.1, candidate.2) < (current.0, current.1, current.2)
            },
        ) {
            best = Some(candidate);
        }
    }
    Ok(match best {
        Some((distance, std::cmp::Reverse(depth), token, target, ambiguous, true)) => {
            LearnedDecision {
                token,
                depth: Some(depth),
                readout_target: target,
                readout_distance: distance,
                readout_ambiguous: ambiguous,
                readout_in_candidates: false,
                learned_arbitrated: true,
                lookahead_target: Some(target),
                lookahead_distance: Some(distance),
                lookahead_ambiguous: ambiguous,
                lookahead_evaluated: true,
            }
        }
        Some((distance, _, _, target, ambiguous, false)) => LearnedDecision {
            token: base.token,
            depth: base.depth,
            readout_target: target,
            readout_distance: distance,
            readout_ambiguous: ambiguous,
            readout_in_candidates: false,
            learned_arbitrated: false,
            lookahead_target: Some(target),
            lookahead_distance: Some(distance),
            lookahead_ambiguous: ambiguous,
            lookahead_evaluated: true,
        },
        None => LearnedDecision {
            token: base.token,
            depth: base.depth,
            readout_target: 0,
            readout_distance: 0,
            readout_ambiguous: false,
            readout_in_candidates: false,
            learned_arbitrated: false,
            lookahead_target: None,
            lookahead_distance: None,
            lookahead_ambiguous: false,
            lookahead_evaluated: false,
        },
    })
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
            "learned-copy {label} prompt, cap or context is invalid"
        )));
    }
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("learned-copy {label} forward: {error}")))?;
    let mut stream = Vec::with_capacity(context);
    let mut next = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("learned-copy {label} prompt step: {error}")))?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some((step.token, step.query_q12));
        }
    }
    let (writer, query) =
        next.ok_or_else(|| invalid(format!("learned-copy {label} prompt is empty")))?;
    Ok((forward, stream, writer, query))
}

fn summarize(
    arm: LearnedCopyArm,
    rows: &[CopyKeyReplyRow],
    steps: &[LearnedCopyStep],
) -> Result<LearnedCopyReading> {
    let mut reading = LearnedCopyReading {
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
        copy_decisions: 0,
        fallback_decisions: 0,
        learned_arbitrations: 0,
        learned_candidate_targets: 0,
        learned_distance_sum: 0,
        lookahead_decisions: 0,
        lookahead_distance_sum: 0,
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
        reading.copy_decisions += u64::from(step.used_copy);
        reading.fallback_decisions += u64::from(!step.used_copy);
        reading.learned_arbitrations += u64::from(step.learned_arbitrated);
        reading.learned_candidate_targets += u64::from(step.readout_in_candidates);
        reading.learned_distance_sum += u64::from(step.readout_distance.unwrap_or_default());
        reading.lookahead_decisions += u64::from(step.lookahead_evaluated);
        reading.lookahead_distance_sum += u64::from(step.lookahead_distance.unwrap_or_default());
        if let Some(depth) = step.selected_depth {
            let slot = reading
                .selected_depth_counts
                .get_mut(depth - 1)
                .ok_or_else(|| invalid("learned-copy selected depth differs"))?;
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

fn decision_hash(steps: &[LearnedCopyStep]) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(steps.len() as u64).to_le_bytes());
    for step in steps {
        bytes.extend_from_slice(&(step.row as u64).to_le_bytes());
        bytes.extend_from_slice(&(step.generated_step as u64).to_le_bytes());
        bytes.extend_from_slice(&step.writer_token.to_le_bytes());
        bytes.extend_from_slice(&step.longest_token.to_le_bytes());
        bytes.extend_from_slice(&step.selected_token.to_le_bytes());
        bytes.push(step.selected_depth.map_or(0, |depth| depth as u8));
        bytes.extend_from_slice(&step.readout_target.unwrap_or_default().to_le_bytes());
        bytes.extend_from_slice(&step.readout_distance.unwrap_or_default().to_le_bytes());
        bytes.push(
            u8::from(step.readout_ambiguous)
                | (u8::from(step.readout_in_candidates) << 1)
                | (u8::from(step.learned_arbitrated) << 2),
        );
        bytes.extend_from_slice(&step.lookahead_target.unwrap_or_default().to_le_bytes());
        bytes.extend_from_slice(&step.lookahead_distance.unwrap_or_default().to_le_bytes());
        bytes.push(u8::from(step.lookahead_ambiguous) | (u8::from(step.lookahead_evaluated) << 1));
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
    use crate::content_readout::ContentSignature;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    fn artifact() -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            181,
            FbsSynth {
                vocab: 512,
                d: 256,
                mlp: 64,
                layers: 1,
                context: 16,
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

    fn signature(bit: usize) -> ContentSignature {
        let mut signature = vec![0u64; ContentSignatureMode::Code.bits() / 64];
        let bit = bit % ContentSignatureMode::Code.bits();
        signature[bit / 64] |= 1u64 << (bit % 64);
        signature
    }

    fn make_readout(rows: Vec<ContentExample>) -> ContentReadout {
        ContentReadout {
            mode: ContentSignatureMode::Code,
            signature_bits: ContentSignatureMode::Code.bits(),
            includes_reference_end: false,
            rows,
            max_distance: 0,
        }
    }

    #[test]
    fn learned_choice_requires_a_candidate_and_uses_its_deepest_occurrence() {
        let readout = make_readout(vec![ContentExample {
            signature: signature(0),
            target: 7,
        }]);
        let mut matches = [None; COPY_KEY_REPLY_MAX_DEPTH];
        matches[0] = Some(9);
        matches[1] = Some(7);
        matches[COPY_KEY_REPLY_MAX_DEPTH - 1] = Some(7);
        let base = crate::CopyKeyReplyDecision {
            token: 9,
            depth: Some(1),
            matches,
            sign_hamming: None,
        };
        let mut query = [0; 256];
        query[0] = 1;
        let selected = learned_decision(&base, &query, &readout).unwrap();
        assert_eq!(selected.token, 7);
        assert_eq!(selected.depth, Some(COPY_KEY_REPLY_MAX_DEPTH));
        assert!(selected.learned_arbitrated);

        let absent = make_readout(vec![ContentExample {
            signature: signature(0),
            target: 11,
        }]);
        let selected = learned_decision(&base, &query, &absent).unwrap();
        assert_eq!(selected.token, 9);
        assert_eq!(selected.depth, Some(1));
        assert!(!selected.readout_in_candidates);
        assert!(!selected.learned_arbitrated);
    }

    #[test]
    fn threshold_miss_falls_back_to_longest_even_when_target_is_available() {
        let mut readout = make_readout(vec![ContentExample {
            signature: signature(0),
            target: 7,
        }]);
        readout.max_distance = 0;
        let mut matches = [None; COPY_KEY_REPLY_MAX_DEPTH];
        matches[0] = Some(9);
        matches[1] = Some(7);
        let base = crate::CopyKeyReplyDecision {
            token: 9,
            depth: Some(1),
            matches,
            sign_hamming: None,
        };
        let query = {
            let mut query = vec![0; 256];
            query[7] = 1;
            query
        };
        let selected = learned_decision(&base, &query, &readout).unwrap();
        assert_eq!(selected.token, 9);
        assert!(selected.readout_in_candidates);
        assert!(!selected.learned_arbitrated);
    }

    #[test]
    fn code_readout_uses_exact_modulo_signature_distance() {
        let readout = make_readout(vec![
            ContentExample {
                signature: signature(0),
                target: 7,
            },
            ContentExample {
                signature: signature(1),
                target: 9,
            },
        ]);
        let prediction = predict_content_target(&readout, &signature(1)).unwrap();
        assert_eq!(prediction.target, 9);
        assert_eq!(prediction.distance, 0);
        assert!(prediction.within_threshold);
    }

    #[test]
    fn lookahead_ranks_candidate_conditioned_next_states() {
        let artifact = artifact();
        let (forward, stream, writer, _) =
            start_reply(&artifact, &[3, 4, 5], 8, 16, "lookahead-test").unwrap();
        let base =
            copy_key_reply_decision(&stream, writer, CopyKeyReplyArm::Longest32To1, 163, 0, 0);
        let mut matches = [None; COPY_KEY_REPLY_MAX_DEPTH];
        matches[0] = Some(7);
        matches[1] = Some(9);
        let mut base = base;
        base.token = 9;
        base.depth = Some(2);
        base.matches = matches;

        let mut next = forward.clone();
        let next_seven = next.step(7, CpuFbsHead::Writer).unwrap();
        let readout = make_readout(vec![ContentExample {
            signature: pack_content_signature(&next_seven.query_q12, ContentSignatureMode::Code)
                .unwrap(),
            target: 42,
        }]);
        let selected = lookahead_decision(&base, &forward, &readout).unwrap();
        assert_eq!(selected.token, 7);
        assert_eq!(selected.depth, Some(1));
        assert!(selected.lookahead_evaluated);
        assert_eq!(selected.lookahead_target, Some(42));
        assert_eq!(selected.lookahead_distance, Some(0));
    }

    #[test]
    fn lookahead_threshold_miss_and_missing_near_candidates_fall_back() {
        let artifact = artifact();
        let (forward, stream, writer, _) =
            start_reply(&artifact, &[3, 4, 5], 8, 16, "lookahead-test").unwrap();
        let mut base =
            copy_key_reply_decision(&stream, writer, CopyKeyReplyArm::Longest32To1, 163, 0, 0);
        base.matches = [None; COPY_KEY_REPLY_MAX_DEPTH];
        base.token = 11;
        base.depth = Some(5);
        base.matches[4] = Some(11);
        let readout = make_readout(vec![ContentExample {
            signature: signature(0),
            target: 42,
        }]);
        let selected = lookahead_decision(&base, &forward, &readout).unwrap();
        assert_eq!(selected.token, 11);
        assert_eq!(selected.depth, Some(5));
        assert!(!selected.lookahead_evaluated);
    }

    #[test]
    fn lookahead_with_no_candidates_preserves_writer_fallback() {
        let artifact = artifact();
        let (forward, _, _, _) =
            start_reply(&artifact, &[3, 4, 5], 8, 16, "lookahead-test").unwrap();
        let base = crate::CopyKeyReplyDecision {
            token: 77,
            depth: None,
            matches: [None; COPY_KEY_REPLY_MAX_DEPTH],
            sign_hamming: None,
        };
        let readout = make_readout(vec![ContentExample {
            signature: signature(0),
            target: 42,
        }]);
        let selected = lookahead_decision(&base, &forward, &readout).unwrap();
        assert_eq!(selected.token, 77);
        assert_eq!(selected.depth, None);
        assert!(!selected.lookahead_evaluated);
    }

    #[test]
    fn training_rows_follow_longest_copy_and_align_reference_targets() {
        let artifact = artifact();
        let sample = MemoryWeightSample {
            prompt: vec![3, 4, 5],
            reply: vec![4, 5, 6, 7],
        };
        let first = training_rows_for_sample(&artifact, &sample, 8, 16, 0).unwrap();
        let repeat = training_rows_for_sample(&artifact, &sample, 8, 16, 0).unwrap();
        assert_eq!(first, repeat);
        assert_eq!(
            first.iter().map(|row| row.target).collect::<Vec<_>>(),
            [4, 5, 6, 7]
        );
        assert!(first
            .iter()
            .all(|row| row.signature.len() == ContentSignatureMode::Code.bits() / 64));
    }

    #[test]
    fn decision_hash_binds_readout_state() {
        let step = LearnedCopyStep {
            row: 1,
            generated_step: 2,
            writer_token: 3,
            longest_token: 4,
            selected_token: 5,
            selected_depth: Some(6),
            used_copy: true,
            readout_target: Some(5),
            readout_distance: Some(7),
            readout_ambiguous: false,
            readout_in_candidates: true,
            learned_arbitrated: true,
            lookahead_target: None,
            lookahead_distance: None,
            lookahead_ambiguous: false,
            lookahead_evaluated: false,
        };
        let mut changed = step.clone();
        changed.lookahead_distance = Some(8);
        assert_ne!(
            decision_hash(std::slice::from_ref(&step)),
            decision_hash(std::slice::from_ref(&changed))
        );
    }

    #[test]
    fn learned_copy_evaluation_repeats_byte_identically_apart_from_timing() {
        let artifact = artifact();
        let samples = vec![MemoryWeightSample {
            prompt: vec![3, 4, 5, 4],
            reply: vec![5, 4, 5, 0],
        }];
        let readout = ContentReadout {
            mode: ContentSignatureMode::Code,
            signature_bits: ContentSignatureMode::Code.bits(),
            includes_reference_end: false,
            rows: vec![ContentExample {
                signature: signature(0),
                target: 4,
            }],
            max_distance: ContentSignatureMode::Code.bits() as u32,
        };
        let first = evaluate_learned_copy_replies(
            &artifact,
            &samples,
            8,
            LearnedCopyArm::LearnedRead32,
            &readout,
            163,
        )
        .unwrap();
        let repeat = evaluate_learned_copy_replies(
            &artifact,
            &samples,
            8,
            LearnedCopyArm::LearnedRead32,
            &readout,
            163,
        )
        .unwrap();
        assert_eq!(first.reading, repeat.reading);
        assert_eq!(first.rows, repeat.rows);
        assert_eq!(first.steps, repeat.steps);
        assert_eq!(first.reply_sha256, repeat.reply_sha256);
        assert_eq!(first.decision_sha256, repeat.decision_sha256);
    }

    #[test]
    fn evenly_spaced_training_rows_are_deterministic_and_bounded() {
        let rows = (0..2500)
            .map(|target| ContentExample {
                signature: signature(target as usize),
                target,
            })
            .collect::<Vec<_>>();
        let retained = evenly_spaced_rows(rows, LEARNED_COPY_MAX_TRAINING_ROWS);
        assert_eq!(retained.len(), LEARNED_COPY_MAX_TRAINING_ROWS);
        assert_eq!(retained[0].target, 0);
        assert_eq!(retained.last().unwrap().target, 2497);
    }
}
