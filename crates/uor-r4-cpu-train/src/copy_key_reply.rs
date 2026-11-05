//! Complete-reply comparison of exact hierarchical copy selection.
//!
//! The selector runs before each decoded token and may replace only the token
//! already chosen by the artifact's writer. It does not alter model state,
//! memory, point-key behaviour or the shared stopping contract.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use uor_r4_integer::stack::{reply_stop, ReplyEnd};

use crate::copy_key_depth::COPY_KEY_MAX_DEPTH;
use crate::reply_eval::{normalized_reference, GeneratedReply};
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;
const COPY_KEY_REPLY_WINDOW: usize = 192;
pub const COPY_KEY_REPLY_MAX_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CopyKeyReplyArm {
    GeometricCopy32,
    Longest32To1,
    Longest4To1,
    K2BackoffK1,
    FixedDepth1,
    RandomAvailableDepth,
    RandomAvailableDepth32,
    Writer,
}

impl CopyKeyReplyArm {
    pub const ALL: [Self; 5] = [
        Self::Longest4To1,
        Self::K2BackoffK1,
        Self::FixedDepth1,
        Self::RandomAvailableDepth,
        Self::Writer,
    ];
    pub const EVALUATION_ARMS: [Self; 8] = [
        Self::GeometricCopy32,
        Self::Longest32To1,
        Self::Longest4To1,
        Self::K2BackoffK1,
        Self::FixedDepth1,
        Self::RandomAvailableDepth,
        Self::RandomAvailableDepth32,
        Self::Writer,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::GeometricCopy32 => "geometric_copy_32",
            Self::Longest32To1 => "longest_32_to_1",
            Self::Longest4To1 => "longest_4_to_1",
            Self::K2BackoffK1 => "k2_backoff_k1",
            Self::FixedDepth1 => "fixed_depth_1",
            Self::RandomAvailableDepth => "random_available_depth",
            Self::RandomAvailableDepth32 => "random_available_depth_32",
            Self::Writer => "writer",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CopyKeyReplyMetric {
    ExactReference,
    NonPrematureBos,
    OpeningCorrect,
    CorrectBodyPositions,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyKeyReplyDecision {
    pub token: u32,
    pub depth: Option<usize>,
    pub matches: [Option<u32>; COPY_KEY_REPLY_MAX_DEPTH],
    pub sign_hamming: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyKeyReplyStep {
    pub row: usize,
    pub generated_step: usize,
    pub writer_token: u32,
    pub selected_token: u32,
    pub selected_depth: Option<usize>,
    pub selected_sign_hamming: Option<u32>,
    pub used_copy: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyKeyReplyRow {
    pub row: usize,
    pub reachable: bool,
    pub exact_reference: bool,
    pub bos_complete: bool,
    pub opening_correct: bool,
    pub premature_bos: bool,
    pub non_premature_bos: bool,
    pub generated_tokens: u64,
    pub reference_tokens: u64,
    pub correct_body_positions: u64,
    pub copy_decisions: u64,
    pub fallback_decisions: u64,
    pub stop: String,
    pub reply_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyKeyReplyReading {
    pub arm: CopyKeyReplyArm,
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
    pub selected_depth_counts: [u64; COPY_KEY_REPLY_MAX_DEPTH],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedCopyKeyReplyReading {
    pub reading: CopyKeyReplyReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<CopyKeyReplyStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyKeyReplyComparison {
    pub metric: CopyKeyReplyMetric,
    pub candidate_only: u64,
    pub control_only: u64,
    pub candidate_sum: u64,
    pub control_sum: u64,
    pub threshold: u64,
    pub passes: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct DecisionCounts {
    decisions: u64,
    copy_decisions: u64,
    fallback_decisions: u64,
    selected_depth_counts: [u64; COPY_KEY_REPLY_MAX_DEPTH],
}

/// Select one next token from the current prompt and generated-token stream.
///
/// This is a pure deterministic decision used by the complete-reply decoder
/// and by finite tests. The writer token is returned unchanged when the policy
/// has no exact match.
pub fn copy_key_reply_decision(
    stream: &[u32],
    writer_token: u32,
    arm: CopyKeyReplyArm,
    seed: u64,
    row: usize,
    generated_step: usize,
) -> CopyKeyReplyDecision {
    let matches = exact_suffix_matches(stream);
    let depth = match arm {
        CopyKeyReplyArm::GeometricCopy32 => (1..=COPY_KEY_REPLY_MAX_DEPTH)
            .rev()
            .find(|&depth| matches[depth - 1].is_some()),
        CopyKeyReplyArm::Longest32To1 => (1..=COPY_KEY_REPLY_MAX_DEPTH)
            .rev()
            .find(|&depth| matches[depth - 1].is_some()),
        CopyKeyReplyArm::Longest4To1 => (1..=COPY_KEY_MAX_DEPTH)
            .rev()
            .find(|&depth| matches[depth - 1].is_some()),
        CopyKeyReplyArm::K2BackoffK1 => [2usize, 1]
            .into_iter()
            .find(|&depth| matches[depth - 1].is_some()),
        CopyKeyReplyArm::FixedDepth1 => matches[0].is_some().then_some(1),
        CopyKeyReplyArm::RandomAvailableDepth => {
            let available = (1..=COPY_KEY_MAX_DEPTH)
                .filter(|&depth| matches[depth - 1].is_some())
                .collect::<Vec<_>>();
            if available.is_empty() {
                None
            } else {
                let draw = splitmix64(
                    seed ^ (row as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                        ^ (generated_step as u64).wrapping_mul(0xd1b5_4a32_d192_ed03),
                ) as usize
                    % available.len();
                Some(available[draw])
            }
        }
        CopyKeyReplyArm::RandomAvailableDepth32 => {
            let available = (1..=COPY_KEY_REPLY_MAX_DEPTH)
                .filter(|&depth| matches[depth - 1].is_some())
                .collect::<Vec<_>>();
            if available.is_empty() {
                None
            } else {
                let draw = splitmix64(
                    seed ^ (row as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                        ^ (generated_step as u64).wrapping_mul(0xd1b5_4a32_d192_ed03),
                ) as usize
                    % available.len();
                Some(available[draw])
            }
        }
        CopyKeyReplyArm::Writer => None,
    };
    CopyKeyReplyDecision {
        token: depth
            .and_then(|depth| matches[depth - 1])
            .unwrap_or(writer_token),
        depth,
        matches,
        sign_hamming: None,
    }
}

fn geometric_copy_key_reply_decision(
    mut decision: CopyKeyReplyDecision,
    query: &[i32],
    forward: &CpuFbsForward,
) -> Result<CopyKeyReplyDecision> {
    let mut candidates = Vec::new();
    for depth in (1..=COPY_KEY_REPLY_MAX_DEPTH).rev() {
        let Some(token) = decision.matches[depth - 1] else {
            continue;
        };
        let distance = forward
            .token_sign_hamming(query, token)
            .map_err(|error| invalid(format!("geometric copy candidate: {error}")))?;
        candidates.push((token, depth, distance));
    }
    if let Some((token, depth, distance)) = select_geometric_candidate(candidates) {
        decision.token = token;
        decision.depth = Some(depth);
        decision.sign_hamming = Some(distance);
    }
    Ok(decision)
}

fn select_geometric_candidate(
    candidates: impl IntoIterator<Item = (u32, usize, u32)>,
) -> Option<(u32, usize, u32)> {
    candidates
        .into_iter()
        .min_by_key(|&candidate| geometric_candidate_key(candidate))
}

fn geometric_candidate_key(
    (token, depth, distance): (u32, usize, u32),
) -> (u32, Reverse<usize>, u32) {
    (distance, Reverse(depth), token)
}

/// Generate one bounded reply under a fixed copy-selection arm.
pub fn generate_copy_key_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    arm: CopyKeyReplyArm,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<CopyKeyReplyStep>)> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("copy-key reply forward: {error}")))?;
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "copy-key reply prompt, cap or artifact context is invalid",
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
            .map_err(|error| invalid(format!("copy-key prompt step: {error}")))?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some((step.token, step.query_q12));
        }
    }
    let mut next = next.ok_or_else(|| invalid("copy-key reply prompt is empty"))?;
    let mut tokens = Vec::with_capacity(cap);
    let mut steps = Vec::with_capacity(cap);
    loop {
        let mut decision = copy_key_reply_decision(&stream, next.0, arm, seed, row, tokens.len());
        if arm == CopyKeyReplyArm::GeometricCopy32 {
            decision = geometric_copy_key_reply_decision(decision, &next.1, &forward)?;
        }
        steps.push(CopyKeyReplyStep {
            row,
            generated_step: tokens.len(),
            writer_token: next.0,
            selected_token: decision.token,
            selected_depth: decision.depth,
            selected_sign_hamming: decision.sign_hamming,
            used_copy: decision.depth.is_some(),
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
            .map_err(|error| invalid(format!("copy-key generated step: {error}")))?;
        next = (step.token, step.query_q12);
    }
}

/// Evaluate one copy-selection arm over fixed complete-reply examples.
pub fn evaluate_copy_key_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: CopyKeyReplyArm,
    seed: u64,
) -> Result<TimedCopyKeyReplyReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("copy-key reply inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("copy-key reply context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply).to_vec();
            if sample.prompt.len() > context || reference.len() > cap {
                return Err(invalid("copy-key reply sample exceeds its context or cap"));
            }
            let (generated, steps) =
                generate_copy_key_reply(artifact, &sample.prompt, cap, arm, seed, row)?;
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
    Ok(TimedCopyKeyReplyReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

pub fn compare_copy_key_reply_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<CopyKeyReplyComparison> {
    if candidate.len() != control.len() {
        return Err(invalid("copy-key reply comparison lengths differ"));
    }
    let mut candidate_only = 0u64;
    let mut control_only = 0u64;
    let mut candidate_sum = 0u64;
    let mut control_sum = 0u64;
    for (candidate, control) in candidate.iter().zip(control) {
        if candidate.row != control.row {
            return Err(invalid("copy-key reply comparison rows differ"));
        }
        if !candidate.reachable || !control.reachable {
            continue;
        }
        let candidate = metric_value(candidate, metric);
        let control = metric_value(control, metric);
        candidate_sum = checked_add(candidate_sum, candidate)?;
        control_sum = checked_add(control_sum, control)?;
        candidate_only = checked_add(candidate_only, u64::from(candidate > control))?;
        control_only = checked_add(control_only, u64::from(control > candidate))?;
    }
    let threshold = ceil_two_sqrt(control_only).max(2);
    Ok(CopyKeyReplyComparison {
        metric,
        candidate_only,
        control_only,
        candidate_sum,
        control_sum,
        threshold,
        passes: candidate_sum > control_sum && candidate_only >= threshold,
    })
}

fn summarize(
    arm: CopyKeyReplyArm,
    rows: &[CopyKeyReplyRow],
    steps: &[CopyKeyReplyStep],
) -> Result<CopyKeyReplyReading> {
    let mut counts = DecisionCounts::default();
    for step in steps {
        counts.decisions = checked_add(counts.decisions, 1)?;
        counts.copy_decisions = checked_add(counts.copy_decisions, u64::from(step.used_copy))?;
        counts.fallback_decisions =
            checked_add(counts.fallback_decisions, u64::from(!step.used_copy))?;
        if let Some(depth) = step.selected_depth {
            let slot = counts
                .selected_depth_counts
                .get_mut(depth - 1)
                .ok_or_else(|| {
                    invalid(format!(
                        "copy-key reply selected depth is outside 1..={COPY_KEY_REPLY_MAX_DEPTH}"
                    ))
                })?;
            *slot = checked_add(*slot, 1)?;
        }
    }
    let mut reading = CopyKeyReplyReading {
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
        decisions: counts.decisions,
        copy_decisions: counts.copy_decisions,
        fallback_decisions: counts.fallback_decisions,
        selected_depth_counts: counts.selected_depth_counts,
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
        reading.non_premature_bos =
            checked_add(reading.non_premature_bos, u64::from(row.non_premature_bos))?;
        reading.generated_tokens = checked_add(reading.generated_tokens, row.generated_tokens)?;
        reading.reference_tokens = checked_add(reading.reference_tokens, row.reference_tokens)?;
        reading.correct_body_positions =
            checked_add(reading.correct_body_positions, row.correct_body_positions)?;
    }
    Ok(reading)
}

fn exact_suffix_matches(stream: &[u32]) -> [Option<u32>; COPY_KEY_REPLY_MAX_DEPTH] {
    exact_suffix_candidates(stream).map(|candidate| candidate.map(|(token, _)| token))
}

pub(crate) fn exact_suffix_candidates(
    stream: &[u32],
) -> [Option<(u32, usize)>; COPY_KEY_REPLY_MAX_DEPTH] {
    let window_start = stream.len().saturating_sub(COPY_KEY_REPLY_WINDOW);
    let window = &stream[window_start..];
    let Some(current) = window.len().checked_sub(1) else {
        return [None; COPY_KEY_REPLY_MAX_DEPTH];
    };
    let mut matches = [None; COPY_KEY_REPLY_MAX_DEPTH];
    for depth in 1..=COPY_KEY_REPLY_MAX_DEPTH {
        matches[depth - 1] = (1..current).rev().find_map(|candidate| {
            (0..depth)
                .all(|back| {
                    window[candidate.saturating_sub(back + 1)]
                        == window[current.saturating_sub(back)]
                })
                .then_some((window[candidate], window_start + candidate))
        });
    }
    matches
}

fn metric_value(row: &CopyKeyReplyRow, metric: CopyKeyReplyMetric) -> u64 {
    match metric {
        CopyKeyReplyMetric::ExactReference => u64::from(row.exact_reference),
        CopyKeyReplyMetric::NonPrematureBos => u64::from(row.non_premature_bos),
        CopyKeyReplyMetric::OpeningCorrect => u64::from(row.opening_correct),
        CopyKeyReplyMetric::CorrectBodyPositions => row.correct_body_positions,
    }
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
        bytes.push(match row.stop.as_str() {
            "bos" => 0,
            "cap" => 1,
            "context" => 2,
            _ => 3,
        });
        bytes.extend_from_slice(row.reply_sha256.as_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn decision_hash(steps: &[CopyKeyReplyStep]) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(steps.len() as u64).to_le_bytes());
    for step in steps {
        bytes.extend_from_slice(&(step.row as u64).to_le_bytes());
        bytes.extend_from_slice(&(step.generated_step as u64).to_le_bytes());
        bytes.extend_from_slice(&step.writer_token.to_le_bytes());
        bytes.extend_from_slice(&step.selected_token.to_le_bytes());
        bytes.push(step.selected_depth.map_or(0, |depth| depth as u8));
        bytes.push(u8::from(step.selected_sign_hamming.is_some()));
        bytes.extend_from_slice(&step.selected_sign_hamming.unwrap_or_default().to_le_bytes());
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

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid("copy-key reply counter overflow"))
}

fn ceil_two_sqrt(value: u64) -> u64 {
    let value = u128::from(value);
    let mut root = 0u128;
    while root * root < 4 * value {
        root += 1;
    }
    root.try_into().unwrap_or(u64::MAX)
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
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};
    use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, POINT_KEY_BOS};

    fn artifact() -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            153,
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

    #[test]
    fn latest_exact_suffix_and_depth_policies_are_fixed() {
        let stream = [9, 6, 7, 8, 5, 7, 6, 7];
        let matches = exact_suffix_matches(&stream);
        assert_eq!(matches[..4], [Some(6), Some(8), None, None]);
        assert!(matches[4..].iter().all(Option::is_none));
        let longest_32 =
            copy_key_reply_decision(&stream, 77, CopyKeyReplyArm::Longest32To1, 1, 0, 0);
        assert_eq!(longest_32.token, 8);
        assert_eq!(longest_32.depth, Some(2));
        let longest = copy_key_reply_decision(&stream, 77, CopyKeyReplyArm::Longest4To1, 1, 0, 0);
        assert_eq!(longest.token, 8);
        assert_eq!(longest.depth, Some(2));
        let k2 = copy_key_reply_decision(&stream, 77, CopyKeyReplyArm::K2BackoffK1, 1, 0, 0);
        assert_eq!(k2.token, 8);
        assert_eq!(k2.depth, Some(2));
        let d1 = copy_key_reply_decision(&stream, 77, CopyKeyReplyArm::FixedDepth1, 1, 0, 0);
        assert_eq!(d1.token, 6);
        assert_eq!(d1.depth, Some(1));
        let writer = copy_key_reply_decision(&stream, 77, CopyKeyReplyArm::Writer, 1, 0, 0);
        assert_eq!(writer.token, 77);
        assert_eq!(writer.depth, None);
    }

    #[test]
    fn exact_suffix_candidates_report_absolute_source_positions() {
        let stream = [9, 6, 7, 8, 5, 7, 6, 7];
        let candidates = exact_suffix_candidates(&stream);
        assert_eq!(candidates[0], Some((6, 6)));
        assert_eq!(stream[candidates[0].unwrap().1 + 1], 7);
    }

    #[test]
    fn copy_arm_serde_names_are_stable() {
        assert_eq!(
            serde_json::to_string(&CopyKeyReplyArm::Longest4To1).unwrap(),
            "\"longest4_to1\""
        );
        assert_eq!(
            serde_json::to_string(&CopyKeyReplyArm::Longest32To1).unwrap(),
            "\"longest32_to1\""
        );
        assert_eq!(
            serde_json::to_string(&CopyKeyReplyArm::RandomAvailableDepth32).unwrap(),
            "\"random_available_depth32\""
        );
        assert_eq!(
            serde_json::to_string(&CopyKeyReplyArm::GeometricCopy32).unwrap(),
            "\"geometric_copy32\""
        );
    }

    #[test]
    fn missing_history_repeats_the_first_token() {
        let stream = [5, 5, 5];
        let matches = exact_suffix_matches(&stream);
        assert_eq!(
            matches,
            [Some(5); COPY_KEY_REPLY_MAX_DEPTH],
            "startup history repeats the first token at every depth"
        );
        let none = copy_key_reply_decision(&[5], 9, CopyKeyReplyArm::Longest4To1, 0, 0, 0);
        assert_eq!(none.token, 9);
        assert_eq!(none.depth, None);
    }

    #[test]
    fn longest_32_prefers_the_maximum_depth_match() {
        let stream = (1..=COPY_KEY_REPLY_MAX_DEPTH as u32)
            .chain(1..=COPY_KEY_REPLY_MAX_DEPTH as u32)
            .collect::<Vec<_>>();
        let matches = exact_suffix_matches(&stream);
        assert_eq!(
            matches[COPY_KEY_REPLY_MAX_DEPTH - 1],
            Some(1),
            "{matches:?}"
        );
        let longest_32 =
            copy_key_reply_decision(&stream, 77, CopyKeyReplyArm::Longest32To1, 1, 0, 0);
        assert_eq!(longest_32.depth, Some(COPY_KEY_REPLY_MAX_DEPTH));
        assert_eq!(longest_32.token, 1);
    }

    #[test]
    fn geometric_arbitration_breaks_equal_distance_toward_depth() {
        let artifact = artifact();
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        forward.mark_reply_start();
        let step = forward.step(7, CpuFbsHead::Writer).unwrap();
        let mut matches = [None; COPY_KEY_REPLY_MAX_DEPTH];
        matches[0] = Some(7);
        matches[COPY_KEY_REPLY_MAX_DEPTH - 1] = Some(7);
        let decision = CopyKeyReplyDecision {
            token: 8,
            depth: None,
            matches,
            sign_hamming: None,
        };
        let selected =
            geometric_copy_key_reply_decision(decision, &step.query_q12, &forward).unwrap();
        assert_eq!(selected.token, 7);
        assert_eq!(selected.depth, Some(COPY_KEY_REPLY_MAX_DEPTH));
        assert_eq!(
            selected.sign_hamming,
            Some(forward.token_sign_hamming(&step.query_q12, 7).unwrap())
        );
    }

    #[test]
    fn geometric_ranking_tie_breaks_by_depth_then_token() {
        let candidates = [
            (9, 4, 7),
            (3, 2, 3),
            (8, 4, 7),
            (8, COPY_KEY_REPLY_MAX_DEPTH, 3),
        ];
        let best = select_geometric_candidate(candidates).unwrap();
        assert_eq!(best, (8, COPY_KEY_REPLY_MAX_DEPTH, 3));

        let same_depth = [(9, 5, 3), (3, 5, 3)];
        let best = select_geometric_candidate(same_depth).unwrap();
        assert_eq!(best, (3, 5, 3));
    }

    #[test]
    fn geometric_ranking_uses_the_smallest_measured_distance() {
        let candidates = [(3, 1, 12), (9, 7, 4), (7, 2, 8)];
        assert_eq!(
            select_geometric_candidate(candidates),
            Some((9, 7, 4)),
            "the nearer distinct token must win regardless of candidate order"
        );
    }

    #[test]
    fn decision_hash_binds_selected_sign_hamming() {
        let step = CopyKeyReplyStep {
            row: 2,
            generated_step: 5,
            writer_token: 17,
            selected_token: 11,
            selected_depth: Some(4),
            selected_sign_hamming: Some(0),
            used_copy: true,
        };
        let mut changed = step.clone();
        changed.selected_sign_hamming = Some(1);
        assert_ne!(
            decision_hash(std::slice::from_ref(&step)),
            decision_hash(std::slice::from_ref(&changed))
        );
    }

    #[test]
    fn k2_backoff_uses_depth_one_when_depth_two_is_absent() {
        let stream = [1, 2, 3, 5, 3];
        let decision = copy_key_reply_decision(&stream, 77, CopyKeyReplyArm::K2BackoffK1, 1, 0, 0);
        assert_eq!(decision.depth, Some(1));
        assert_eq!(decision.token, 5);
    }

    #[test]
    fn candidates_do_not_cross_the_fixed_copy_window() {
        let mut stream = vec![1; COPY_KEY_REPLY_WINDOW + 2];
        stream[0] = 99;
        stream[1] = 88;
        stream[2] = 5;
        stream[3] = 7;
        let last = stream.len() - 1;
        stream[last - 1] = 5;
        stream[last] = 5;
        let matches = exact_suffix_matches(&stream);
        assert_eq!(matches[1], Some(7), "{matches:?}");
    }

    #[test]
    fn bos_completion_requires_one_unit_of_cap_and_context_headroom() {
        assert!(!reference_reachable(10, 8, 8, 20));
        assert!(!reference_reachable(10, 8, 9, 18));
        assert!(reference_reachable(10, 7, 8, 18));
        assert!(reference_reachable(10, 8, 9, 19));
    }

    #[test]
    fn random_depth_is_deterministic_and_never_invents_a_match() {
        let stream = [4, 1, 2, 7, 1, 2];
        let first =
            copy_key_reply_decision(&stream, 88, CopyKeyReplyArm::RandomAvailableDepth, 7, 2, 3);
        let repeat =
            copy_key_reply_decision(&stream, 88, CopyKeyReplyArm::RandomAvailableDepth, 7, 2, 3);
        assert_eq!(first, repeat);
        assert!(first.depth.is_some());
        let none =
            copy_key_reply_decision(&[1], 88, CopyKeyReplyArm::RandomAvailableDepth, 7, 2, 3);
        assert_eq!(none.token, 88);
        assert_eq!(none.depth, None);
    }

    #[test]
    fn random_depth_32_is_deterministic_covers_its_domain_and_never_invents_a_match() {
        let stream = (1..=COPY_KEY_REPLY_MAX_DEPTH as u32)
            .chain(1..=COPY_KEY_REPLY_MAX_DEPTH as u32)
            .collect::<Vec<_>>();
        let matches = exact_suffix_matches(&stream);
        assert!(matches.iter().all(Option::is_some), "{matches:?}");

        let mut depths = std::collections::BTreeSet::new();
        for seed in 0..64 {
            let decision = copy_key_reply_decision(
                &stream,
                88,
                CopyKeyReplyArm::RandomAvailableDepth32,
                seed,
                0,
                0,
            );
            let repeat = copy_key_reply_decision(
                &stream,
                88,
                CopyKeyReplyArm::RandomAvailableDepth32,
                seed,
                0,
                0,
            );
            assert_eq!(decision, repeat);
            let depth = decision.depth.expect("every depth is available");
            depths.insert(depth);
            assert_eq!(decision.token, matches[depth - 1].unwrap());
        }
        assert_eq!(depths.first(), Some(&1));
        assert_eq!(depths.last(), Some(&COPY_KEY_REPLY_MAX_DEPTH));

        let none =
            copy_key_reply_decision(&[1], 88, CopyKeyReplyArm::RandomAvailableDepth32, 7, 2, 3);
        assert_eq!(none.token, 88);
        assert_eq!(none.depth, None);
    }

    #[test]
    fn generated_reply_feeds_the_selected_copy_token_and_stops() {
        let artifact = artifact();
        let forward_arm = CpuFbsForward::from_artifact(&artifact).unwrap();
        let context = forward_arm.context_length();
        assert!(context > 8);
        drop(forward_arm);
        let prompt = vec![3, 4, 5, 4, 5];
        let (reply, steps) =
            generate_copy_key_reply(&artifact, &prompt, 8, CopyKeyReplyArm::Longest4To1, 153, 0)
                .unwrap();
        assert!(reply.tokens.len() <= 8);
        assert_eq!(
            steps.len(),
            reply.tokens.len() + usize::from(reply.stop == "bos")
        );
        assert!(steps
            .iter()
            .all(|step| step.used_copy || step.selected_token == step.writer_token));
        assert_ne!(steps[0].selected_token, POINT_KEY_BOS);
    }

    #[test]
    fn generated_reply_honors_bos_cap_and_context_stops() {
        let artifact = artifact();
        let context = CpuFbsForward::from_artifact(&artifact)
            .unwrap()
            .context_length();

        let (bos, _) = generate_copy_key_reply(
            &artifact,
            &[0, 0, 0],
            8,
            CopyKeyReplyArm::Longest4To1,
            153,
            0,
        )
        .unwrap();
        assert_eq!(bos.stop, "bos");
        assert!(bos.tokens.is_empty());

        let (cap, _) =
            generate_copy_key_reply(&artifact, &[3, 4, 5], 1, CopyKeyReplyArm::Writer, 153, 0)
                .unwrap();
        assert_eq!(cap.stop, "cap");
        assert_eq!(cap.tokens.len(), 1);

        let (context_stop, _) = generate_copy_key_reply(
            &artifact,
            &vec![2; context - 1],
            8,
            CopyKeyReplyArm::Writer,
            153,
            0,
        )
        .unwrap();
        assert_eq!(context_stop.stop, "context");
        assert!(context_stop.tokens.len() <= 1, "{context_stop:?}");
    }

    #[test]
    fn paired_comparison_uses_the_registered_two_sigma_rule() {
        let row = |row, exact_reference| CopyKeyReplyRow {
            row,
            reachable: true,
            exact_reference,
            bos_complete: exact_reference,
            opening_correct: exact_reference,
            premature_bos: false,
            non_premature_bos: exact_reference,
            generated_tokens: 1,
            reference_tokens: 1,
            correct_body_positions: u64::from(exact_reference),
            copy_decisions: 1,
            fallback_decisions: 0,
            stop: if exact_reference { "bos" } else { "cap" }.to_owned(),
            reply_sha256: token_hash(&[u32::from(exact_reference)]),
        };
        let candidate = vec![row(0, true), row(1, true)];
        let control = vec![row(0, false), row(1, false)];
        let comparison =
            compare_copy_key_reply_rows(&candidate, &control, CopyKeyReplyMetric::ExactReference)
                .unwrap();
        assert_eq!(comparison.candidate_sum, 2);
        assert_eq!(comparison.threshold, 2);
        assert!(comparison.passes);
    }

    #[test]
    fn paired_comparison_excludes_unreachable_replies() {
        let reachable = CopyKeyReplyRow {
            row: 0,
            reachable: true,
            exact_reference: true,
            bos_complete: true,
            opening_correct: true,
            premature_bos: false,
            non_premature_bos: true,
            generated_tokens: 1,
            reference_tokens: 1,
            correct_body_positions: 1,
            copy_decisions: 1,
            fallback_decisions: 0,
            stop: "bos".to_owned(),
            reply_sha256: token_hash(&[1]),
        };
        let mut unreachable = reachable.clone();
        unreachable.row = 1;
        unreachable.reachable = false;
        let comparison = compare_copy_key_reply_rows(
            &[reachable.clone(), unreachable.clone()],
            &[reachable, unreachable],
            CopyKeyReplyMetric::ExactReference,
        )
        .unwrap();
        assert_eq!(comparison.candidate_sum, 1);
        assert_eq!(comparison.control_sum, 1);
        assert_eq!(comparison.candidate_only, 0);
        assert_eq!(comparison.control_only, 0);
    }

    #[test]
    fn same_artifact_replay_is_byte_identical_apart_from_timing() {
        let artifact = artifact();
        let samples = vec![MemoryWeightSample {
            prompt: vec![3, 4, 5, 4],
            reply: vec![5, 4, 5, POINT_KEY_BOS],
        }];
        let first =
            evaluate_copy_key_replies(&artifact, &samples, 8, CopyKeyReplyArm::Longest4To1, 153)
                .unwrap();
        let repeat =
            evaluate_copy_key_replies(&artifact, &samples, 8, CopyKeyReplyArm::Longest4To1, 153)
                .unwrap();
        assert_eq!(first.reading, repeat.reading);
        assert_eq!(first.rows, repeat.rows);
        assert_eq!(first.steps, repeat.steps);
        assert_eq!(first.reply_sha256, repeat.reply_sha256);
        assert_eq!(first.decision_sha256, repeat.decision_sha256);
    }

    #[test]
    fn geometric_arm_replay_binds_query_state_and_selected_distance() {
        let artifact = artifact();
        let samples = vec![MemoryWeightSample {
            prompt: vec![3, 4, 5, 4],
            reply: vec![5, 4, 5, POINT_KEY_BOS],
        }];
        let first = evaluate_copy_key_replies(
            &artifact,
            &samples,
            8,
            CopyKeyReplyArm::GeometricCopy32,
            153,
        )
        .unwrap();
        let repeat = evaluate_copy_key_replies(
            &artifact,
            &samples,
            8,
            CopyKeyReplyArm::GeometricCopy32,
            153,
        )
        .unwrap();
        assert!(first
            .steps
            .iter()
            .any(|step| step.used_copy && step.selected_sign_hamming.is_some()));
        assert_eq!(first.reading, repeat.reading);
        assert_eq!(first.rows, repeat.rows);
        assert_eq!(first.steps, repeat.steps);
        assert_eq!(first.reply_sha256, repeat.reply_sha256);
        assert_eq!(first.decision_sha256, repeat.decision_sha256);
    }
}
