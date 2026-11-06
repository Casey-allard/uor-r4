//! Full prompt-plus-reply boundary memory aligned to the full-sequence content
//! selector.
//!
//! Internal reply positions contribute nonterminal observations; only a final
//! reply suffix contributes terminal observations. At generation the longest
//! known suffix of the full prompt-plus-generated stream decides. A suffix with
//! any observed continuation blocks BOS.

use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uor_r4_integer::stack::{reply_stop, ReplyEnd, POINT_KEY_BOS};

use crate::copy_key_reply::{
    compare_copy_key_reply_rows, copy_key_reply_decision, CopyKeyReplyArm, CopyKeyReplyMetric,
    CopyKeyReplyRow,
};
use crate::reply_boundary::{ReplyBoundaryHit, ReplyBoundaryMemory};
use crate::reply_eval::{normalized_reference, GeneratedReply};
use crate::suffix_memory::{SuffixMemoryTable, SUFFIX_MEMORY_MAX_DEPTH};
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const FULL_BOUNDARY_MIN_DEPTH: usize = 4;
pub const FULL_BOUNDARY_MAX_DEPTH: usize = SUFFIX_MEMORY_MAX_DEPTH;
pub const FULL_BOUNDARY_MIN_OBSERVATIONS: u64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FullBoundaryArm {
    ContentFullBoundary,
    ContentReplyBoundary,
    ContentShuffledFullBoundary,
    ContentNoBoundary,
    LocalFullBoundary,
    LocalReplyBoundary,
    WriterFullBoundary,
    ContentReferenceOracle,
}

impl FullBoundaryArm {
    pub const EVALUATION_ARMS: [Self; 8] = [
        Self::ContentFullBoundary,
        Self::ContentReplyBoundary,
        Self::ContentShuffledFullBoundary,
        Self::ContentNoBoundary,
        Self::LocalFullBoundary,
        Self::LocalReplyBoundary,
        Self::WriterFullBoundary,
        Self::ContentReferenceOracle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::ContentFullBoundary => "content_full_boundary",
            Self::ContentReplyBoundary => "content_reply_boundary",
            Self::ContentShuffledFullBoundary => "content_shuffled_full_boundary",
            Self::ContentNoBoundary => "content_no_boundary",
            Self::LocalFullBoundary => "local_full_boundary",
            Self::LocalReplyBoundary => "local_reply_boundary",
            Self::WriterFullBoundary => "writer_full_boundary",
            Self::ContentReferenceOracle => "content_reference_oracle",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct FullBoundaryCounts {
    pub terminal: u64,
    pub nonterminal: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullSequenceBoundaryMemory {
    entries: BTreeMap<Vec<u32>, FullBoundaryCounts>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct FullBoundaryStop {
    pub depth: usize,
    pub terminal: u64,
    pub nonterminal: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FullBoundaryCensus {
    pub suffixes: u64,
    pub terminal_occurrences: u64,
    pub nonterminal_occurrences: u64,
    pub terminal_only_suffixes: u64,
    pub ambiguous_suffixes: u64,
    pub suffixes_by_depth: [u64; FULL_BOUNDARY_MAX_DEPTH],
    pub terminal_by_depth: [u64; FULL_BOUNDARY_MAX_DEPTH],
    pub nonterminal_by_depth: [u64; FULL_BOUNDARY_MAX_DEPTH],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FullBoundaryStep {
    pub row: usize,
    pub generated_step: usize,
    pub writer_token: u32,
    pub local_token: Option<u32>,
    pub global_token: Option<u32>,
    pub selected_token: u32,
    pub used_global: bool,
    pub local_copy_depth: Option<usize>,
    pub global_depth: Option<usize>,
    pub stop_kind: Option<String>,
    pub stop_depth: Option<usize>,
    pub stop_terminal: u64,
    pub stop_nonterminal: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FullBoundaryReading {
    pub arm: FullBoundaryArm,
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
    pub full_boundary_stops: u64,
    pub reply_boundary_stops: u64,
    pub oracle_stops: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TimedFullBoundaryReading {
    pub reading: FullBoundaryReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<FullBoundaryStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FullBoundaryChanges {
    pub common_steps: u64,
    pub changed_steps: u64,
    pub divergent_rows: u64,
    pub first_changed_step: Option<(usize, usize)>,
}

impl FullSequenceBoundaryMemory {
    pub fn entries(&self) -> usize {
        self.entries.len()
    }

    pub fn census(&self) -> FullBoundaryCensus {
        let mut census = FullBoundaryCensus {
            suffixes: self.entries.len() as u64,
            terminal_occurrences: 0,
            nonterminal_occurrences: 0,
            terminal_only_suffixes: 0,
            ambiguous_suffixes: 0,
            suffixes_by_depth: [0; FULL_BOUNDARY_MAX_DEPTH],
            terminal_by_depth: [0; FULL_BOUNDARY_MAX_DEPTH],
            nonterminal_by_depth: [0; FULL_BOUNDARY_MAX_DEPTH],
        };
        for (suffix, counts) in &self.entries {
            let depth = suffix.len();
            if !(FULL_BOUNDARY_MIN_DEPTH..=FULL_BOUNDARY_MAX_DEPTH).contains(&depth) {
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

    pub fn stop_decision(&self, full_stream: &[u32]) -> Option<FullBoundaryStop> {
        let maximum = full_stream.len().min(FULL_BOUNDARY_MAX_DEPTH);
        for depth in (FULL_BOUNDARY_MIN_DEPTH..=maximum).rev() {
            let suffix = &full_stream[full_stream.len() - depth..];
            let Some(counts) = self.entries.get(suffix) else {
                continue;
            };
            if counts.terminal > 0 && counts.nonterminal == 0 {
                return Some(FullBoundaryStop {
                    depth,
                    terminal: counts.terminal,
                    nonterminal: counts.nonterminal,
                });
            }
            return None;
        }
        None
    }
}

pub fn derive_full_sequence_boundary(
    samples: &[MemoryWeightSample],
) -> Result<FullSequenceBoundaryMemory> {
    if samples.is_empty() {
        return Err(invalid("full-boundary derivation needs training samples"));
    }
    let mut entries = BTreeMap::<Vec<u32>, FullBoundaryCounts>::new();
    for sample in samples {
        let reply = normalized_reference(&sample.reply);
        if sample.prompt.is_empty() || reply.is_empty() {
            return Err(invalid(
                "full-boundary derivation saw an empty prompt or reply",
            ));
        }
        let mut history = sample.prompt.clone();
        history.extend_from_slice(reply);
        for end in sample.prompt.len() + 1..=history.len() {
            let terminal = end == history.len();
            let maximum = end.min(FULL_BOUNDARY_MAX_DEPTH);
            for depth in FULL_BOUNDARY_MIN_DEPTH..=maximum {
                let counts = entries
                    .entry(history[end - depth..end].to_vec())
                    .or_default();
                if terminal {
                    counts.terminal = counts
                        .terminal
                        .checked_add(1)
                        .ok_or_else(|| invalid("full-boundary terminal count overflows"))?;
                } else {
                    counts.nonterminal = counts
                        .nonterminal
                        .checked_add(1)
                        .ok_or_else(|| invalid("full-boundary nonterminal count overflows"))?;
                }
            }
        }
    }
    if entries.is_empty() {
        return Err(invalid(
            "full-boundary derivation found no suffixes of depth at least four",
        ));
    }
    Ok(FullSequenceBoundaryMemory { entries })
}

pub fn shuffle_full_boundary(
    original: &FullSequenceBoundaryMemory,
    seed: u64,
) -> Result<(FullSequenceBoundaryMemory, u64)> {
    for attempt in 0..64u64 {
        let mut shuffled = original.clone();
        for depth in FULL_BOUNDARY_MIN_DEPTH..=FULL_BOUNDARY_MAX_DEPTH {
            let keys = shuffled
                .entries
                .iter()
                .filter(|(suffix, _)| suffix.len() == depth)
                .map(|(suffix, counts)| (suffix.clone(), counts.terminal))
                .collect::<Vec<_>>();
            let terminal_total = keys.iter().map(|(_, count)| *count).sum::<u64>();
            if terminal_total == 0 {
                continue;
            }
            for (suffix, _) in &keys {
                shuffled
                    .entries
                    .get_mut(suffix)
                    .expect("suffix exists")
                    .terminal = 0;
            }
            for event in 0..terminal_total {
                let draw = splitmix64(
                    seed ^ (attempt << 48)
                        ^ (depth as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                        ^ event.wrapping_mul(0xd1b5_4a32_d192_ed03),
                ) as usize
                    % keys.len();
                shuffled
                    .entries
                    .get_mut(&keys[draw].0)
                    .expect("suffix exists")
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
        "full-boundary shuffle could not change a terminal suffix",
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn generate_full_boundary_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    reference_length: usize,
    arm: FullBoundaryArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    full_boundary: Option<&FullSequenceBoundaryMemory>,
    reply_boundary: &ReplyBoundaryMemory,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<FullBoundaryStep>)> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("full-boundary forward: {error}")))?;
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "full-boundary prompt, cap or artifact context is invalid",
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
            .map_err(|error| invalid(format!("full-boundary prompt step: {error}")))?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some(step.token);
        }
    }
    let mut writer_token = next.ok_or_else(|| invalid("full-boundary prompt is empty"))?;
    let mut tokens = Vec::with_capacity(cap);
    let mut steps = Vec::with_capacity(cap);

    loop {
        let stop = stop_hit(
            arm,
            &stream,
            &tokens,
            reference_length,
            full_boundary,
            reply_boundary,
        );
        if let Some((kind, depth, terminal, nonterminal)) = stop {
            steps.push(FullBoundaryStep {
                row,
                generated_step: tokens.len(),
                writer_token,
                local_token: None,
                global_token: None,
                selected_token: POINT_KEY_BOS,
                used_global: false,
                local_copy_depth: None,
                global_depth: None,
                stop_kind: Some(kind.to_owned()),
                stop_depth: Some(depth),
                stop_terminal: terminal,
                stop_nonterminal: nonterminal,
            });
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
            FullBoundaryArm::ContentFullBoundary
                | FullBoundaryArm::ContentReplyBoundary
                | FullBoundaryArm::ContentShuffledFullBoundary
                | FullBoundaryArm::ContentNoBoundary
                | FullBoundaryArm::ContentReferenceOracle
        );
        let use_local = use_content
            || matches!(
                arm,
                FullBoundaryArm::LocalFullBoundary | FullBoundaryArm::LocalReplyBoundary
            );
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
                    FULL_BOUNDARY_MIN_OBSERVATIONS,
                )
        } else {
            None
        };
        let used_global = use_content
            && global.is_some_and(|prediction| {
                local_depth.is_none_or(|local_depth| prediction.depth > local_depth)
            });
        let selected_token = if used_global {
            global.expect("used_global requires global").token
        } else if local_depth.is_some() {
            local.as_ref().expect("local depth requires local").token
        } else {
            writer_token
        };
        steps.push(FullBoundaryStep {
            row,
            generated_step: tokens.len(),
            writer_token,
            local_token: local.as_ref().map(|decision| decision.token),
            global_token: global.map(|prediction| prediction.token),
            selected_token,
            used_global,
            local_copy_depth: local_depth,
            global_depth: global.map(|prediction| prediction.depth),
            stop_kind: None,
            stop_depth: None,
            stop_terminal: 0,
            stop_nonterminal: 0,
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
            .map_err(|error| invalid(format!("full-boundary generated step: {error}")))?;
        writer_token = step.token;
    }
}

fn stop_hit<'a>(
    arm: FullBoundaryArm,
    stream: &[u32],
    generated: &[u32],
    reference_length: usize,
    full_boundary: Option<&'a FullSequenceBoundaryMemory>,
    reply_boundary: &'a ReplyBoundaryMemory,
) -> Option<(&'static str, usize, u64, u64)> {
    match arm {
        FullBoundaryArm::ContentFullBoundary
        | FullBoundaryArm::LocalFullBoundary
        | FullBoundaryArm::WriterFullBoundary => full_boundary
            .expect("full-boundary arm has a table")
            .stop_decision(stream)
            .map(|hit| ("full", hit.depth, hit.terminal, hit.nonterminal)),
        FullBoundaryArm::ContentShuffledFullBoundary => full_boundary
            .expect("shuffled full-boundary arm has a table")
            .stop_decision(stream)
            .map(|hit| ("shuffled_full", hit.depth, hit.terminal, hit.nonterminal)),
        FullBoundaryArm::ContentReplyBoundary | FullBoundaryArm::LocalReplyBoundary => {
            reply_boundary
                .stop_decision(generated)
                .map(|hit: ReplyBoundaryHit| ("reply", hit.depth, hit.terminal, hit.nonterminal))
        }
        FullBoundaryArm::ContentReferenceOracle => {
            (generated.len() == reference_length).then_some(("oracle", 0, 1, 0))
        }
        FullBoundaryArm::ContentNoBoundary => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_full_boundary_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: FullBoundaryArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    full_boundary: Option<&FullSequenceBoundaryMemory>,
    reply_boundary: &ReplyBoundaryMemory,
    seed: u64,
) -> Result<TimedFullBoundaryReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("full-boundary reply inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("full-boundary context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply).to_vec();
            if sample.prompt.len() > context || reference.len() > cap {
                return Err(invalid("full-boundary sample exceeds its context or cap"));
            }
            let (generated, steps) = generate_full_boundary_reply(
                artifact,
                &sample.prompt,
                cap,
                reference.len(),
                arm,
                suffix_memory,
                full_boundary,
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
                    step.stop_kind.is_none() && !step.used_global && step.local_copy_depth.is_some()
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
    Ok(TimedFullBoundaryReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

pub fn compare_full_boundary_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<crate::copy_key_reply::CopyKeyReplyComparison> {
    compare_copy_key_reply_rows(candidate, control, metric)
}

pub fn full_boundary_changes(
    candidate: &TimedFullBoundaryReading,
    control: &TimedFullBoundaryReading,
) -> FullBoundaryChanges {
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
        .keys()
        .chain(control_steps.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|key| candidate_steps.get(*key) != control_steps.get(*key))
        .copied()
        .collect::<Vec<_>>();
    let divergent_rows = changed
        .iter()
        .map(|(row, _)| *row)
        .collect::<BTreeSet<_>>()
        .len() as u64;
    FullBoundaryChanges {
        common_steps: candidate_steps
            .keys()
            .filter(|key| control_steps.contains_key(key))
            .count() as u64,
        changed_steps: changed.len() as u64,
        divergent_rows,
        first_changed_step: changed.first().copied(),
    }
}

pub fn full_boundary_stop_changes(
    left: &TimedFullBoundaryReading,
    right: &TimedFullBoundaryReading,
) -> u64 {
    let stops = |reading: &TimedFullBoundaryReading| {
        reading
            .rows
            .iter()
            .filter(|row| row.reachable)
            .map(|row| (row.row, (row.stop == "bos").then_some(row.generated_tokens)))
            .collect::<BTreeMap<_, _>>()
    };
    let left = stops(left);
    let right = stops(right);
    left.keys()
        .chain(right.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|row| {
            left.get(row)
                .zip(right.get(row))
                .is_some_and(|(left, right)| left != right)
        })
        .count() as u64
}

fn summarize(
    arm: FullBoundaryArm,
    rows: &[CopyKeyReplyRow],
    steps: &[FullBoundaryStep],
) -> Result<FullBoundaryReading> {
    let mut reading = FullBoundaryReading {
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
        full_boundary_stops: 0,
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
            Some("full") | Some("shuffled_full") => {
                reading.full_boundary_stops = checked_add(reading.full_boundary_stops, 1)?;
            }
            Some("reply") => {
                reading.reply_boundary_stops = checked_add(reading.reply_boundary_stops, 1)?;
            }
            Some("oracle") => {
                reading.oracle_stops = checked_add(reading.oracle_stops, 1)?;
            }
            Some(_) => return Err(invalid("unknown full-boundary stop kind")),
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
    let accounted_decisions = reading
        .local_decisions
        .checked_add(reading.global_decisions)
        .and_then(|count| count.checked_add(reading.writer_decisions))
        .and_then(|count| count.checked_add(reading.full_boundary_stops))
        .and_then(|count| count.checked_add(reading.reply_boundary_stops))
        .and_then(|count| count.checked_add(reading.oracle_stops))
        .ok_or_else(|| invalid("full-boundary decision accounting overflows"))?;
    let reachable_decisions = steps
        .iter()
        .filter(|step| reachable_rows.contains(&step.row))
        .count() as u64;
    if accounted_decisions != reachable_decisions {
        return Err(invalid(format!(
            "full-boundary decision accounting differs: {accounted_decisions} != {reachable_decisions}"
        )));
    }
    Ok(reading)
}

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid("full-boundary counter overflows"))
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

fn decision_hash(steps: &[FullBoundaryStep]) -> String {
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
        hasher.update(step.stop_kind.as_deref().unwrap_or_default().as_bytes());
        hasher.update(&(step.stop_depth.unwrap_or(0) as u16).to_le_bytes());
        hasher.update(&step.stop_terminal.to_le_bytes());
        hasher.update(&step.stop_nonterminal.to_le_bytes());
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

    fn sample(prompt: &[u32], reply: &[u32]) -> MemoryWeightSample {
        MemoryWeightSample {
            prompt: prompt.to_vec(),
            reply: reply.to_vec(),
        }
    }

    #[test]
    fn full_history_distinguishes_terminal_from_internal_reply_tail() {
        let boundary = derive_full_sequence_boundary(&[
            sample(&[1, 2, 3, 4], &[10, 11, 12, 13]),
            sample(&[1, 2, 3, 4, 10], &[11, 12, 13, 14]),
            sample(&[20, 21, 22, 23], &[30, 31, 40, 41, 42, 43]),
        ])
        .unwrap();
        assert_eq!(
            boundary.stop_decision(&[1, 2, 3, 4, 10, 11, 12, 13]),
            None,
            "the same full-history prefix is terminal once and internal once"
        );
        assert!(boundary
            .stop_decision(&[20, 21, 22, 23, 30, 31, 40, 41, 42, 43])
            .is_some());
        assert!(boundary.stop_decision(&[1, 2, 3, 4, 10, 11]).is_none());
    }

    #[test]
    fn full_boundary_shuffle_preserves_depth_conditioned_totals() {
        let boundary = derive_full_sequence_boundary(&[
            sample(&[1, 2, 3, 4], &[10, 11, 12, 13, 14]),
            sample(&[5, 6, 7, 8], &[20, 21, 22, 23, 24]),
            sample(&[9, 10, 11, 12], &[30, 31, 32, 33, 34]),
        ])
        .unwrap();
        let (shuffled, changed) = shuffle_full_boundary(&boundary, 200).unwrap();
        assert!(changed > 0);
        let original_census = boundary.census();
        let shuffled_census = shuffled.census();
        assert_eq!(
            original_census.suffixes_by_depth,
            shuffled_census.suffixes_by_depth
        );
        assert_eq!(
            original_census.terminal_by_depth,
            shuffled_census.terminal_by_depth
        );
        assert_eq!(
            original_census.nonterminal_by_depth,
            shuffled_census.nonterminal_by_depth
        );
        assert_eq!(
            boundary.entries.keys().collect::<Vec<_>>(),
            shuffled.entries.keys().collect::<Vec<_>>()
        );
        for (suffix, counts) in &boundary.entries {
            assert_eq!(
                counts.nonterminal, shuffled.entries[suffix].nonterminal,
                "nonterminal count changed for {suffix:?}"
            );
        }
    }

    fn reply_row(
        row: usize,
        reachable: bool,
        generated_tokens: u64,
        stop: &str,
    ) -> CopyKeyReplyRow {
        CopyKeyReplyRow {
            row,
            reachable,
            exact_reference: false,
            bos_complete: stop == "bos",
            opening_correct: false,
            premature_bos: false,
            non_premature_bos: stop == "bos",
            generated_tokens,
            reference_tokens: generated_tokens,
            correct_body_positions: 0,
            copy_decisions: 0,
            fallback_decisions: 0,
            stop: stop.to_owned(),
            reply_sha256: String::new(),
        }
    }

    fn reading(rows: Vec<CopyKeyReplyRow>) -> TimedFullBoundaryReading {
        TimedFullBoundaryReading {
            reading: FullBoundaryReading {
                arm: FullBoundaryArm::ContentFullBoundary,
                replies: rows.len() as u64,
                reference_reachable: rows.iter().filter(|row| row.reachable).count() as u64,
                exact_reference: 0,
                bos_complete: 0,
                opening_correct: 0,
                premature_bos: 0,
                non_premature_bos: 0,
                generated_tokens: 0,
                reference_tokens: 0,
                correct_body_positions: 0,
                decisions: 0,
                local_decisions: 0,
                global_decisions: 0,
                writer_decisions: 0,
                full_boundary_stops: 0,
                reply_boundary_stops: 0,
                oracle_stops: 0,
            },
            rows,
            steps: Vec::new(),
            reply_sha256: String::new(),
            decision_sha256: String::new(),
            wall_elapsed_us: 0,
            generated_tokens_per_second: 0.0,
        }
    }

    #[test]
    fn stop_changes_ignore_source_label_when_behavior_matches() {
        let learned = reading(vec![reply_row(7, true, 12, "bos")]);
        let natural = reading(vec![reply_row(7, true, 12, "bos")]);
        assert_eq!(full_boundary_stop_changes(&learned, &natural), 0);
    }

    #[test]
    fn stop_changes_detect_timing_and_stop_presence() {
        let early = reading(vec![reply_row(7, true, 12, "bos")]);
        let late = reading(vec![reply_row(7, true, 13, "bos")]);
        assert_eq!(full_boundary_stop_changes(&early, &late), 1);
        let no_stop = reading(vec![reply_row(7, true, 13, "cap")]);
        assert_eq!(full_boundary_stop_changes(&early, &no_stop), 1);
    }

    #[test]
    fn stop_changes_exclude_rows_unreachable_in_either_arm() {
        let reachable_stop = reading(vec![reply_row(7, true, 12, "bos")]);
        let unreachable_change = reading(vec![
            reply_row(7, true, 12, "bos"),
            reply_row(9, false, 13, "cap"),
        ]);
        assert_eq!(
            full_boundary_stop_changes(&reachable_stop, &unreachable_change),
            0
        );
    }
}
