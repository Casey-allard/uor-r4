//! Exact reply-boundary suffix memory for the complete-reply copy path.
//!
//! A generated suffix that occurred only at the end of a TRAIN reply may emit
//! decoder BOS. A suffix seen with any continuation belongs to content and may
//! not stop. The longest known suffix decides; shorter suffixes cannot override
//! a known continuation.

use std::collections::BTreeMap;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uor_r4_integer::stack::{reply_stop, ReplyEnd};

use crate::copy_key_reply::{
    copy_key_reply_decision, CopyKeyReplyArm, CopyKeyReplyMetric, CopyKeyReplyRow,
};
use crate::reply_eval::{normalized_reference, GeneratedReply};
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const REPLY_BOUNDARY_MIN_DEPTH: usize = 4;
pub const REPLY_BOUNDARY_MAX_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplyBoundaryArm {
    Longest32NoBoundary,
    Longest32TrueBoundary,
    Longest32ShuffledBoundary,
    ReferenceLengthOracle,
}

impl ReplyBoundaryArm {
    pub const EVALUATION_ARMS: [Self; 4] = [
        Self::Longest32NoBoundary,
        Self::Longest32TrueBoundary,
        Self::Longest32ShuffledBoundary,
        Self::ReferenceLengthOracle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Longest32NoBoundary => "longest32_no_boundary",
            Self::Longest32TrueBoundary => "longest32_true_boundary",
            Self::Longest32ShuffledBoundary => "longest32_shuffled_boundary",
            Self::ReferenceLengthOracle => "reference_length_oracle",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ReplyBoundaryCounts {
    pub terminal: u64,
    pub nonterminal: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReplyBoundaryMemory {
    entries: BTreeMap<Vec<u32>, ReplyBoundaryCounts>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ReplyBoundaryHit {
    pub depth: usize,
    pub terminal: u64,
    pub nonterminal: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReplyBoundaryCensus {
    pub suffixes: u64,
    pub terminal_occurrences: u64,
    pub nonterminal_occurrences: u64,
    pub terminal_only_suffixes: u64,
    pub ambiguous_suffixes: u64,
    pub suffixes_by_depth: [u64; REPLY_BOUNDARY_MAX_DEPTH],
    pub terminal_by_depth: [u64; REPLY_BOUNDARY_MAX_DEPTH],
    pub nonterminal_by_depth: [u64; REPLY_BOUNDARY_MAX_DEPTH],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReplyBoundaryStep {
    pub row: usize,
    pub generated_step: usize,
    pub writer_token: u32,
    pub selected_token: u32,
    pub copy_depth: Option<usize>,
    pub boundary_depth: Option<usize>,
    pub boundary_terminal: u64,
    pub boundary_nonterminal: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReplyBoundaryReading {
    pub arm: ReplyBoundaryArm,
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
    pub boundary_stops: u64,
    pub boundary_stop_depths: [u64; REPLY_BOUNDARY_MAX_DEPTH],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TimedReplyBoundaryReading {
    pub reading: ReplyBoundaryReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<ReplyBoundaryStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReplyBoundaryComparison {
    pub metric: CopyKeyReplyMetric,
    pub candidate_only: u64,
    pub control_only: u64,
    pub candidate_sum: u64,
    pub control_sum: u64,
    pub threshold: u64,
    pub passes: bool,
}

impl ReplyBoundaryMemory {
    pub fn entries(&self) -> usize {
        self.entries.len()
    }

    pub fn census(&self) -> ReplyBoundaryCensus {
        let mut census = ReplyBoundaryCensus {
            suffixes: self.entries.len() as u64,
            terminal_occurrences: 0,
            nonterminal_occurrences: 0,
            terminal_only_suffixes: 0,
            ambiguous_suffixes: 0,
            suffixes_by_depth: [0; REPLY_BOUNDARY_MAX_DEPTH],
            terminal_by_depth: [0; REPLY_BOUNDARY_MAX_DEPTH],
            nonterminal_by_depth: [0; REPLY_BOUNDARY_MAX_DEPTH],
        };
        for (suffix, counts) in &self.entries {
            let depth = suffix.len();
            if !(REPLY_BOUNDARY_MIN_DEPTH..=REPLY_BOUNDARY_MAX_DEPTH).contains(&depth) {
                continue;
            }
            census.suffixes_by_depth[depth - 1] += 1;
            census.terminal_by_depth[depth - 1] =
                census.terminal_by_depth[depth - 1].saturating_add(counts.terminal);
            census.nonterminal_by_depth[depth - 1] =
                census.nonterminal_by_depth[depth - 1].saturating_add(counts.nonterminal);
            census.terminal_occurrences =
                census.terminal_occurrences.saturating_add(counts.terminal);
            census.nonterminal_occurrences = census
                .nonterminal_occurrences
                .saturating_add(counts.nonterminal);
            census.terminal_only_suffixes +=
                u64::from(counts.terminal > 0 && counts.nonterminal == 0);
            census.ambiguous_suffixes += u64::from(counts.terminal > 0 && counts.nonterminal > 0);
        }
        census
    }

    /// The longest known generated-reply suffix decides. A terminal-only suffix
    /// emits BOS; a suffix with any continuation observation blocks stopping.
    pub fn stop_decision(&self, generated: &[u32]) -> Option<ReplyBoundaryHit> {
        let maximum = generated.len().min(REPLY_BOUNDARY_MAX_DEPTH);
        for depth in (REPLY_BOUNDARY_MIN_DEPTH..=maximum).rev() {
            let suffix = &generated[generated.len() - depth..];
            if let Some(counts) = self.entries.get(suffix) {
                if counts.terminal > 0 && counts.nonterminal == 0 {
                    return Some(ReplyBoundaryHit {
                        depth,
                        terminal: counts.terminal,
                        nonterminal: counts.nonterminal,
                    });
                }
                return None;
            }
        }
        None
    }
}

/// Build terminal/nonterminal suffix counts from TRAIN replies only.
pub fn derive_reply_boundary_memory(samples: &[MemoryWeightSample]) -> Result<ReplyBoundaryMemory> {
    if samples.is_empty() {
        return Err(invalid("reply-boundary derivation needs training samples"));
    }
    let mut entries = BTreeMap::<Vec<u32>, ReplyBoundaryCounts>::new();
    for sample in samples {
        let reference = normalized_reference(&sample.reply);
        if reference.is_empty() {
            return Err(invalid("reply-boundary derivation saw an empty reply"));
        }
        let maximum = reference.len().min(REPLY_BOUNDARY_MAX_DEPTH);
        for depth in REPLY_BOUNDARY_MIN_DEPTH..=maximum {
            add_boundary_count(&mut entries, &reference[reference.len() - depth..], true)?;
        }
        for end in REPLY_BOUNDARY_MIN_DEPTH..reference.len() {
            let maximum = end.min(REPLY_BOUNDARY_MAX_DEPTH);
            for depth in REPLY_BOUNDARY_MIN_DEPTH..=maximum {
                add_boundary_count(&mut entries, &reference[end - depth..end], false)?;
            }
        }
    }
    if entries.is_empty() {
        return Err(invalid(
            "reply-boundary derivation found no suffix of at least four tokens",
        ));
    }
    Ok(ReplyBoundaryMemory { entries })
}

fn add_boundary_count(
    entries: &mut BTreeMap<Vec<u32>, ReplyBoundaryCounts>,
    suffix: &[u32],
    terminal: bool,
) -> Result<()> {
    let counts = entries.entry(suffix.to_vec()).or_default();
    if terminal {
        counts.terminal = counts
            .terminal
            .checked_add(1)
            .ok_or_else(|| invalid("reply-boundary terminal count overflows"))?;
    } else {
        counts.nonterminal = counts
            .nonterminal
            .checked_add(1)
            .ok_or_else(|| invalid("reply-boundary nonterminal count overflows"))?;
    }
    Ok(())
}

/// Deterministically move terminal counts across suffix keys at the same
/// depth. Nonterminal counts stay fixed, so the control still treats the same
/// continuations as content.
pub fn shuffle_reply_boundary_memory(
    original: &ReplyBoundaryMemory,
    seed: u64,
) -> Result<(ReplyBoundaryMemory, u64)> {
    for attempt in 0..64u64 {
        let mut shuffled = original.clone();
        for depth in REPLY_BOUNDARY_MIN_DEPTH..=REPLY_BOUNDARY_MAX_DEPTH {
            let candidates = shuffled
                .entries
                .iter()
                .filter(|(suffix, _)| suffix.len() == depth)
                .map(|(suffix, _)| suffix.clone())
                .collect::<Vec<_>>();
            if candidates.is_empty() {
                continue;
            }
            let terminal_total = candidates
                .iter()
                .map(|suffix| shuffled.entries[suffix].terminal)
                .sum::<u64>();
            if terminal_total == 0 {
                continue;
            }
            for suffix in &candidates {
                shuffled
                    .entries
                    .get_mut(suffix)
                    .expect("candidate exists")
                    .terminal = 0;
            }
            for event in 0..terminal_total {
                let draw = splitmix64(
                    seed ^ (attempt << 48)
                        ^ (depth as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                        ^ event.wrapping_mul(0xd1b5_4a32_d192_ed03),
                ) as usize
                    % candidates.len();
                shuffled
                    .entries
                    .get_mut(&candidates[draw])
                    .expect("candidate exists")
                    .terminal += 1;
            }
        }
        let changed = shuffled
            .entries
            .iter()
            .filter(|(suffix, counts)| {
                original
                    .entries
                    .get(*suffix)
                    .is_some_and(|old| old.terminal != counts.terminal)
            })
            .count() as u64;
        if changed > 0 {
            return Ok((shuffled, changed));
        }
    }
    Err(invalid(
        "reply-boundary shuffle could not change a terminal suffix",
    ))
}

/// Generate one bounded reply from the fixed longest-copy body and a boundary
/// arm. The oracle stops at the normalized reference length and is diagnostic.
#[allow(clippy::too_many_arguments)]
pub fn generate_boundary_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    reference_length: usize,
    arm: ReplyBoundaryArm,
    boundary: Option<&ReplyBoundaryMemory>,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<ReplyBoundaryStep>)> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("reply-boundary forward: {error}")))?;
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "reply-boundary prompt, cap or artifact context is invalid",
        ));
    }
    let table = match arm {
        ReplyBoundaryArm::Longest32NoBoundary | ReplyBoundaryArm::ReferenceLengthOracle => None,
        ReplyBoundaryArm::Longest32TrueBoundary | ReplyBoundaryArm::Longest32ShuffledBoundary => {
            Some(boundary.ok_or_else(|| invalid("reply-boundary arm has no table"))?)
        }
    };

    let mut stream = Vec::with_capacity(context);
    let mut next = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("reply-boundary prompt step: {error}")))?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some(step.token);
        }
    }
    let mut writer_token = next.ok_or_else(|| invalid("reply-boundary prompt is empty"))?;
    let mut tokens = Vec::with_capacity(cap);
    let mut steps = Vec::with_capacity(cap);
    loop {
        let hit = match arm {
            ReplyBoundaryArm::Longest32NoBoundary => None,
            ReplyBoundaryArm::Longest32TrueBoundary
            | ReplyBoundaryArm::Longest32ShuffledBoundary => table
                .expect("arms with boundaries carry a table")
                .stop_decision(&tokens),
            ReplyBoundaryArm::ReferenceLengthOracle => (tokens.len() == reference_length)
                .then_some(ReplyBoundaryHit {
                    depth: 0,
                    terminal: 1,
                    nonterminal: 0,
                }),
        };
        if let Some(hit) = hit {
            steps.push(ReplyBoundaryStep {
                row,
                generated_step: tokens.len(),
                writer_token,
                selected_token: 0,
                copy_depth: None,
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

        let decision = copy_key_reply_decision(
            &stream,
            writer_token,
            CopyKeyReplyArm::Longest32To1,
            seed,
            row,
            tokens.len(),
        );
        steps.push(ReplyBoundaryStep {
            row,
            generated_step: tokens.len(),
            writer_token,
            selected_token: decision.token,
            copy_depth: decision.depth,
            boundary_depth: None,
            boundary_terminal: 0,
            boundary_nonterminal: 0,
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
            .map_err(|error| invalid(format!("reply-boundary generated step: {error}")))?;
        writer_token = step.token;
    }
}

pub fn evaluate_boundary_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: ReplyBoundaryArm,
    boundary: Option<&ReplyBoundaryMemory>,
    seed: u64,
) -> Result<TimedReplyBoundaryReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("reply-boundary reply inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("reply-boundary context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply).to_vec();
            if sample.prompt.len() > context || reference.len() > cap {
                return Err(invalid("reply-boundary sample exceeds its context or cap"));
            }
            let (generated, steps) = generate_boundary_reply(
                artifact,
                &sample.prompt,
                cap,
                reference.len(),
                arm,
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
            let copy_decisions = steps
                .iter()
                .filter(|step| step.copy_depth.is_some())
                .count() as u64;
            let fallback_decisions = steps
                .iter()
                .filter(|step| step.boundary_depth.is_none() && step.copy_depth.is_none())
                .count() as u64;
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
    Ok(TimedReplyBoundaryReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

fn summarize(
    arm: ReplyBoundaryArm,
    rows: &[CopyKeyReplyRow],
    steps: &[ReplyBoundaryStep],
) -> Result<ReplyBoundaryReading> {
    let mut reading = ReplyBoundaryReading {
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
        boundary_stops: 0,
        boundary_stop_depths: [0; REPLY_BOUNDARY_MAX_DEPTH],
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
        reading.copy_decisions = checked_add(reading.copy_decisions, row.copy_decisions)?;
        reading.fallback_decisions =
            checked_add(reading.fallback_decisions, row.fallback_decisions)?;
    }
    for step in steps {
        if let Some(depth) = step.boundary_depth {
            reading.boundary_stops = checked_add(reading.boundary_stops, 1)?;
            if depth > 0 && depth <= REPLY_BOUNDARY_MAX_DEPTH {
                reading.boundary_stop_depths[depth - 1] += 1;
            }
        }
    }
    Ok(reading)
}

pub fn compare_boundary_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<ReplyBoundaryComparison> {
    if candidate.len() != control.len() {
        return Err(invalid("reply-boundary comparison lengths differ"));
    }
    let mut candidate_only = 0u64;
    let mut control_only = 0u64;
    let mut candidate_sum = 0u64;
    let mut control_sum = 0u64;
    for (candidate, control) in candidate.iter().zip(control) {
        if candidate.row != control.row {
            return Err(invalid("reply-boundary comparison rows differ"));
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
    let threshold = ceil_two_sqrt(
        candidate_only
            .checked_add(control_only)
            .ok_or_else(|| invalid("reply-boundary discordant count overflows"))?,
    )
    .max(2);
    Ok(ReplyBoundaryComparison {
        metric,
        candidate_only,
        control_only,
        candidate_sum,
        control_sum,
        threshold,
        passes: candidate_sum > control_sum && candidate_only >= threshold,
    })
}

pub fn boundary_stop_changes(
    left: &TimedReplyBoundaryReading,
    right: &TimedReplyBoundaryReading,
) -> u64 {
    let key =
        |step: &ReplyBoundaryStep| (step.row, step.generated_step, step.boundary_depth.unwrap());
    let left = left
        .steps
        .iter()
        .filter(|step| step.boundary_depth.is_some())
        .map(key)
        .collect::<std::collections::BTreeSet<_>>();
    let right = right
        .steps
        .iter()
        .filter(|step| step.boundary_depth.is_some())
        .map(key)
        .collect::<std::collections::BTreeSet<_>>();
    left.symmetric_difference(&right).count() as u64
}

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid("reply-boundary count overflows"))
}

fn ceil_two_sqrt(value: u64) -> u64 {
    let value = u128::from(value);
    let mut root = 0u128;
    while root * root < 4 * value {
        root += 1;
    }
    root.try_into().unwrap_or(u64::MAX)
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

fn decision_hash(steps: &[ReplyBoundaryStep]) -> String {
    let mut hasher = blake3::Hasher::new();
    for step in steps {
        hasher.update(&(step.row as u64).to_le_bytes());
        hasher.update(&(step.generated_step as u64).to_le_bytes());
        hasher.update(&step.writer_token.to_le_bytes());
        hasher.update(&step.selected_token.to_le_bytes());
        hasher.update(&(step.copy_depth.unwrap_or(0) as u16).to_le_bytes());
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

    fn sample(prompt: &[u32], reply: &[u32]) -> MemoryWeightSample {
        MemoryWeightSample {
            prompt: prompt.to_vec(),
            reply: reply.to_vec(),
        }
    }

    #[test]
    fn terminal_and_internal_suffixes_are_distinguished() {
        let training = vec![
            sample(&[10], &[20, 21, 22, 23, 1]),
            sample(&[11], &[20, 21, 22, 23, 24, 1]),
        ];
        let memory = derive_reply_boundary_memory(&training).unwrap();
        assert_eq!(memory.stop_decision(&[20, 21, 22, 23]), None);
        assert_eq!(memory.stop_decision(&[21, 22, 23, 24]).unwrap().depth, 4);
        assert_eq!(
            memory.census().terminal_only_suffixes,
            memory
                .entries
                .values()
                .filter(|counts| counts.terminal > 0 && counts.nonterminal == 0)
                .count() as u64
        );
    }

    #[test]
    fn a_continuation_blocks_stopping() {
        let training = vec![
            sample(&[10], &[6, 7, 8, 9, 1]),
            sample(&[11], &[1, 2, 3, 4, 5, 9, 1]),
        ];
        let memory = derive_reply_boundary_memory(&training).unwrap();
        assert_eq!(memory.stop_decision(&[6, 7, 8, 9]).unwrap().depth, 4);
        assert_eq!(memory.stop_decision(&[5, 1, 2, 3, 4]), None);
        assert_eq!(memory.stop_decision(&[1, 2, 3, 4, 5]), None);
    }

    #[test]
    fn shuffled_counts_keep_depth_totals_and_change_keys() {
        let training = (0..20)
            .map(|i| {
                let mut reply = (0..8).map(|j| 10 + i + j).collect::<Vec<_>>();
                reply.push(1);
                sample(&[i], &reply)
            })
            .collect::<Vec<_>>();
        let memory = derive_reply_boundary_memory(&training).unwrap();
        let (shuffled, changed) = shuffle_reply_boundary_memory(&memory, 191).unwrap();
        assert!(changed > 0);
        let original = memory.census();
        let control = shuffled.census();
        assert_eq!(original.terminal_by_depth, control.terminal_by_depth);
        assert_eq!(original.nonterminal_by_depth, control.nonterminal_by_depth);
    }

    #[test]
    fn reachability_matches_the_strict_existing_boundaries() {
        assert!(!reference_reachable(100, 4, 4, 200));
        assert!(!reference_reachable(100, 3, 64, 103));
        assert!(reference_reachable(100, 3, 64, 104));
        assert!(reference_reachable(100, 4, 5, 105));
    }

    #[test]
    fn terminal_boundary_stops_and_controls_do_not() {
        let artifact = export_fbs_trunk_uorlut01(&synthetic_fbs_trunk_with(
            191,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 3,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        ))
        .unwrap();
        let memory = derive_reply_boundary_memory(&[sample(&[9], &[40, 10, 20, 30, 1])]).unwrap();
        let prompt = [10, 20, 30, 40, 10, 20, 30];
        let true_reply = generate_boundary_reply(
            &artifact,
            &prompt,
            64,
            4,
            ReplyBoundaryArm::Longest32TrueBoundary,
            Some(&memory),
            191,
            0,
        )
        .unwrap();
        assert_eq!(true_reply.0.tokens, [40, 10, 20, 30]);
        assert_eq!(true_reply.0.stop, "bos");
        assert_eq!(true_reply.1.last().unwrap().boundary_depth, Some(4));

        let no_boundary = generate_boundary_reply(
            &artifact,
            &prompt,
            64,
            4,
            ReplyBoundaryArm::Longest32NoBoundary,
            None,
            191,
            0,
        )
        .unwrap();
        assert!(no_boundary.0.stop != "bos" || no_boundary.0.tokens.len() > 4);

        let oracle = generate_boundary_reply(
            &artifact,
            &prompt,
            64,
            4,
            ReplyBoundaryArm::ReferenceLengthOracle,
            None,
            191,
            0,
        )
        .unwrap();
        assert_eq!(oracle.0.tokens, true_reply.0.tokens);
        assert_eq!(oracle.0.stop, "bos");
    }

    #[test]
    fn stop_change_digest_ignores_identical_and_detects_position_or_depth() {
        let left = timed_with_stops(&[(0, 4, 4), (1, 8, 5)]);
        assert_eq!(boundary_stop_changes(&left, &left), 0);
        assert_eq!(
            boundary_stop_changes(&left, &timed_with_stops(&[(0, 5, 4), (1, 8, 5)])),
            2
        );
        assert_eq!(
            boundary_stop_changes(&left, &timed_with_stops(&[(0, 4, 5), (1, 8, 5)])),
            2
        );
    }

    #[test]
    fn paired_threshold_uses_both_discordant_arms() {
        let candidate = (0..4).map(|row| row_fixture(row, true)).collect::<Vec<_>>();
        let control = (0..4)
            .map(|row| row_fixture(row, false))
            .collect::<Vec<_>>();
        let comparison =
            compare_boundary_rows(&candidate, &control, CopyKeyReplyMetric::ExactReference)
                .unwrap();
        assert_eq!(comparison.candidate_only, 4);
        assert_eq!(comparison.control_only, 0);
        assert_eq!(comparison.threshold, 4);
        assert!(comparison.passes);
    }

    #[test]
    fn digests_bind_reply_length_stop_and_boundary_counts() {
        let row = row_fixture(0, true);
        let mut changed_row = row.clone();
        changed_row.generated_tokens += 1;
        assert_ne!(reply_hash(&[row]), reply_hash(&[changed_row]));

        let step = ReplyBoundaryStep {
            row: 0,
            generated_step: 4,
            writer_token: 7,
            selected_token: 0,
            copy_depth: None,
            boundary_depth: Some(4),
            boundary_terminal: 2,
            boundary_nonterminal: 0,
        };
        let mut changed_step = step.clone();
        changed_step.boundary_nonterminal = 1;
        assert_ne!(decision_hash(&[step]), decision_hash(&[changed_step]));
    }

    fn row_fixture(row: usize, exact: bool) -> CopyKeyReplyRow {
        CopyKeyReplyRow {
            row,
            reachable: true,
            exact_reference: exact,
            bos_complete: exact,
            opening_correct: exact,
            premature_bos: false,
            non_premature_bos: exact,
            generated_tokens: 4,
            reference_tokens: 4,
            correct_body_positions: 4,
            copy_decisions: 4,
            fallback_decisions: 0,
            stop: if exact { "bos" } else { "cap" }.to_owned(),
            reply_sha256: String::new(),
        }
    }

    fn timed_with_stops(stops: &[(usize, usize, usize)]) -> TimedReplyBoundaryReading {
        TimedReplyBoundaryReading {
            reading: ReplyBoundaryReading {
                arm: ReplyBoundaryArm::Longest32TrueBoundary,
                replies: 2,
                reference_reachable: 2,
                exact_reference: 0,
                bos_complete: stops.len() as u64,
                opening_correct: 0,
                premature_bos: 0,
                non_premature_bos: stops.len() as u64,
                generated_tokens: 0,
                reference_tokens: 0,
                correct_body_positions: 0,
                decisions: 0,
                copy_decisions: 0,
                fallback_decisions: 0,
                boundary_stops: stops.len() as u64,
                boundary_stop_depths: [0; REPLY_BOUNDARY_MAX_DEPTH],
            },
            rows: Vec::new(),
            steps: stops
                .iter()
                .map(|&(row, generated_step, depth)| ReplyBoundaryStep {
                    row,
                    generated_step,
                    writer_token: 0,
                    selected_token: 0,
                    copy_depth: None,
                    boundary_depth: Some(depth),
                    boundary_terminal: 1,
                    boundary_nonterminal: 0,
                })
                .collect(),
            reply_sha256: String::new(),
            decision_sha256: String::new(),
            wall_elapsed_us: 0,
            generated_tokens_per_second: 0.0,
        }
    }
}
