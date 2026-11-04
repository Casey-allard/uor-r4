//! Deterministic complete-reply evaluation over the exact CPU-served FBS path.
//!
//! This module deliberately scores token identity, stopping behavior and simple
//! repetition diagnostics. Exact-reference agreement is not a semantic or
//! useful-chat measurement.

use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;
use uor_r4_integer::stack::{read_memory_uorlut01, reply_stop, ReplyEnd, POINT_KEY_BOS};

use crate::memory_train::memory_reply_key;
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplyReading {
    pub replies: u64,
    pub bos_complete: u64,
    pub cap_stopped: u64,
    pub context_stopped: u64,
    pub exact_reference: u64,
    pub reference_reachable: u64,
    pub opening_correct: u64,
    pub non_empty: u64,
    pub distinct_non_empty: u64,
    pub repeated_4gram: u64,
    pub generated_tokens: u64,
    pub reference_tokens: u64,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedReplyReading {
    pub reading: ReplyReading,
    pub rows: Vec<ReplyRowReading>,
    pub reply_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplyRowReading {
    pub row: usize,
    pub reachable: bool,
    pub exact: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminationStepReading {
    pub row: usize,
    pub generated_step: usize,
    pub generated_token: u32,
    pub writer_token: u32,
    pub bos_admitted: bool,
    pub bos_rank: Option<usize>,
    pub bos_top8: bool,
    pub candidate_count: usize,
    pub bos_distance: i64,
    pub chosen_distance: i64,
    pub bos_minus_chosen_distance: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminationClass {
    NoDecisions,
    AllChosenBos,
    NeverAdmitted,
    AdmittedNotTop1,
    Top1Suppressed,
    Mixed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminationReading {
    pub rows: u64,
    pub decisions: u64,
    pub chosen_bos: u64,
    pub bos_admitted: u64,
    pub bos_top8: u64,
    pub bos_top1: u64,
    pub never_admitted: u64,
    pub admitted_not_top1: u64,
    pub top1_suppressed: u64,
    pub min_bos_rank: Option<usize>,
    pub bos_distance_sum: u64,
    pub chosen_distance_sum: u64,
    pub classification: Option<TerminationClass>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedTerminationReading {
    pub reading: TerminationReading,
    pub steps: Vec<TerminationStepReading>,
    pub reply_sha256: String,
    pub wall_elapsed_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedReply {
    pub tokens: Vec<u32>,
    pub stop: &'static str,
}

/// Generate one bounded greedy reply from an exact exported artifact.
///
/// Every prompt token is stepped once. Reply start is marked before the final
/// prompt token, whose output is the first generated reply token. Generated
/// tokens are then fed back until the shared integer `reply_stop` contract
/// returns BOS, the cap or context exhaustion.
pub fn generate_reply(artifact: &[u8], prompt: &[u32], cap: usize) -> Result<GeneratedReply> {
    generate_reply_with_memory(artifact, prompt, cap, true)
}

/// Generate one bounded reply after disabling the artifact's turn-memory and
/// point-key reads while retaining its trunk and artifact identity.
pub fn generate_reply_memory_off(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
) -> Result<GeneratedReply> {
    generate_reply_with_memory(artifact, prompt, cap, false)
}

fn generate_reply_with_memory(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    memory_enabled: bool,
) -> Result<GeneratedReply> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("complete-reply forward: {error}")))?;
    if !memory_enabled {
        forward.disable_memory();
    }
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "complete-reply prompt, cap or artifact context is invalid",
        ));
    }
    let mut first = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("complete-reply prompt step: {error}")))?;
        if index + 1 == prompt.len() {
            first = Some(step.token);
        }
    }
    bounded_reply(
        first.ok_or_else(|| invalid("complete-reply prompt is empty"))?,
        prompt.len(),
        cap,
        context,
        |token| {
            forward
                .step(token, CpuFbsHead::Writer)
                .map(|step| step.token)
                .map_err(|error| invalid(format!("complete-reply generated step: {error}")))
        },
    )
}

/// Generate and summarize complete replies for every sample.
pub fn evaluate_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
) -> Result<TimedReplyReading> {
    evaluate_generated(artifact, samples, cap, |prompt, cap| {
        generate_reply(artifact, prompt, cap)
    })
}

/// Generate and summarize complete replies with turn memory and point-key
/// reads disabled.
pub fn evaluate_replies_memory_off(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
) -> Result<TimedReplyReading> {
    evaluate_generated(artifact, samples, cap, |prompt, cap| {
        generate_reply_memory_off(artifact, prompt, cap)
    })
}

/// Record BOS treatment by the writer at every generated decision while
/// reproducing the same bounded greedy reply sequence.
pub fn evaluate_termination_diagnostics(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
) -> Result<TimedTerminationReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("termination diagnostic inputs differ"));
    }
    let started = std::time::Instant::now();
    let rows = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let (generated, steps) = generate_reply_with_diagnostics(
                artifact,
                &sample.prompt,
                cap,
                memory_enabled,
                row,
            )?;
            let reference = normalized_reference(&sample.reply).to_vec();
            let context = CpuFbsForward::from_artifact(artifact)
                .map_err(|error| invalid(format!("termination context: {error}")))?
                .context_length();
            let reachable = reference_reachable(sample.prompt.len(), reference.len(), cap, context);
            Ok((generated, reference, reachable, steps))
        })
        .collect::<Result<Vec<_>>>()?;
    let reply_sha256 = reply_sequence_sha256(
        &rows
            .iter()
            .map(|(reply, reference, reachable, _)| (reply.clone(), reference.clone(), *reachable))
            .collect::<Vec<_>>(),
    );
    let steps = rows
        .into_iter()
        .flat_map(|(_, _, _, steps)| steps)
        .collect::<Vec<_>>();
    let reading = summarize_termination(&steps)?;
    Ok(TimedTerminationReading {
        reading,
        steps,
        reply_sha256,
        wall_elapsed_us: started.elapsed().as_micros().try_into().unwrap_or(u64::MAX),
    })
}

fn generate_reply_with_diagnostics(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    memory_enabled: bool,
    row: usize,
) -> Result<(GeneratedReply, Vec<TerminationStepReading>)> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("termination forward: {error}")))?;
    if !memory_enabled {
        forward.disable_memory();
    }
    let context = forward.context_length();
    if prompt.is_empty() || prompt.len() > context {
        return Err(invalid("termination prompt or context differs"));
    }
    let mut next = None;
    let mut diagnosis = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("termination prompt step: {error}")))?;
        if index + 1 == prompt.len() {
            diagnosis = Some(
                forward
                    .writer_token_diagnostic(&step.query_q12, POINT_KEY_BOS)
                    .map_err(|error| invalid(format!("termination writer diagnostic: {error}")))?,
            );
            next = Some(step.token);
        }
    }
    let mut next = next.ok_or_else(|| invalid("termination prompt is empty"))?;
    let mut tokens = Vec::with_capacity(cap);
    let mut steps = Vec::with_capacity(cap);
    loop {
        let diagnostic = diagnosis
            .take()
            .ok_or_else(|| invalid("termination diagnostic lost its writer reading"))?;
        steps.push(TerminationStepReading {
            row,
            generated_step: tokens.len(),
            generated_token: next,
            writer_token: diagnostic.chosen_token,
            bos_admitted: diagnostic.admitted,
            bos_rank: diagnostic.rank,
            bos_top8: diagnostic.top8,
            candidate_count: diagnostic.candidate_count,
            bos_distance: diagnostic.target_distance,
            chosen_distance: diagnostic.chosen_distance,
            bos_minus_chosen_distance: diagnostic.target_minus_chosen_distance,
        });
        let stop = reply_stop(next, tokens.len(), prompt.len(), cap, context);
        if stop != Some(ReplyEnd::Bos) {
            tokens.push(next);
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
        let step = forward
            .step(next, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("termination generated step: {error}")))?;
        diagnosis = Some(
            forward
                .writer_token_diagnostic(&step.query_q12, POINT_KEY_BOS)
                .map_err(|error| invalid(format!("termination writer diagnostic: {error}")))?,
        );
        next = step.token;
    }
}

fn summarize_termination(steps: &[TerminationStepReading]) -> Result<TerminationReading> {
    let mut reading = TerminationReading {
        rows: steps
            .iter()
            .map(|step| step.row)
            .collect::<BTreeSet<_>>()
            .len() as u64,
        ..TerminationReading::default()
    };
    for step in steps {
        reading.decisions = add_count(reading.decisions, 1)?;
        reading.chosen_bos = add_count(reading.chosen_bos, u64::from(step.generated_token == 0))?;
        reading.bos_admitted = add_count(reading.bos_admitted, u64::from(step.bos_admitted))?;
        reading.bos_top8 = add_count(reading.bos_top8, u64::from(step.bos_top8))?;
        reading.bos_top1 = add_count(reading.bos_top1, u64::from(step.bos_rank == Some(1)))?;
        if let Some(rank) = step.bos_rank {
            reading.min_bos_rank = Some(reading.min_bos_rank.map_or(rank, |old| old.min(rank)));
        }
        reading.bos_distance_sum = reading
            .bos_distance_sum
            .checked_add(
                step.bos_distance
                    .try_into()
                    .map_err(|_| invalid("termination BOS distance is negative or too large"))?,
            )
            .ok_or_else(|| invalid("termination BOS distance sum overflows"))?;
        reading.chosen_distance_sum = reading
            .chosen_distance_sum
            .checked_add(
                step.chosen_distance
                    .try_into()
                    .map_err(|_| invalid("termination chosen distance is negative or too large"))?,
            )
            .ok_or_else(|| invalid("termination chosen distance sum overflows"))?;
    }
    reading.never_admitted = reading
        .decisions
        .checked_sub(reading.bos_admitted)
        .ok_or_else(|| invalid("termination admitted count exceeds decisions"))?;
    reading.admitted_not_top1 = reading
        .bos_admitted
        .checked_sub(reading.bos_top1)
        .ok_or_else(|| invalid("termination top-1 count exceeds admitted"))?;
    reading.top1_suppressed = reading
        .bos_top1
        .checked_sub(reading.chosen_bos)
        .ok_or_else(|| invalid("termination chosen count exceeds top-1"))?;
    reading.classification = Some(if reading.decisions == 0 {
        TerminationClass::NoDecisions
    } else if reading.never_admitted > 0
        && reading.admitted_not_top1 == 0
        && reading.top1_suppressed == 0
        && reading.chosen_bos == 0
    {
        TerminationClass::NeverAdmitted
    } else if reading.never_admitted == 0
        && reading.admitted_not_top1 > 0
        && reading.top1_suppressed == 0
        && reading.chosen_bos == 0
    {
        TerminationClass::AdmittedNotTop1
    } else if reading.never_admitted == 0
        && reading.admitted_not_top1 == 0
        && reading.top1_suppressed > 0
        && reading.chosen_bos == 0
    {
        TerminationClass::Top1Suppressed
    } else if reading.chosen_bos == reading.decisions {
        TerminationClass::AllChosenBos
    } else {
        TerminationClass::Mixed
    });
    Ok(reading)
}

/// Generate complete replies with a direct exact-memory target override.
///
/// A hit on the opening key names the first reply token. After each generated
/// token is fed through the exact CPU forward, a hit on the same reply key the
/// served memory uses replaces the writer's fallback token with the stored
/// target. Misses retain the forward token. The CPU forward still applies the
/// artifact's memory-weight query movement when producing fallback tokens; only
/// the chosen token is overridden on a hit.
pub fn evaluate_memory_target_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
) -> Result<TimedReplyReading> {
    evaluate_generated(artifact, samples, cap, |prompt, cap| {
        generate_memory_target_reply(artifact, prompt, cap)
    })
}

fn evaluate_generated(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    generate: impl Fn(&[u32], usize) -> Result<GeneratedReply> + Sync,
) -> Result<TimedReplyReading> {
    if samples.is_empty() {
        return Err(invalid(
            "complete-reply evaluation needs at least one sample",
        ));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("complete-reply context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let rows = samples
        .par_iter()
        .map(|sample| {
            let generated = generate(&sample.prompt, cap)?;
            let reference = normalized_reference(&sample.reply).to_vec();
            let reachable = reference_reachable(sample.prompt.len(), reference.len(), cap, context);
            Ok((generated, reference, reachable))
        })
        .collect::<Result<Vec<_>>>()?;
    let row_readings = rows
        .iter()
        .enumerate()
        .map(|(row, (generated, reference, reachable))| ReplyRowReading {
            row,
            reachable: *reachable,
            exact: generated.tokens == normalized_reference(reference),
        })
        .collect::<Vec<_>>();
    let reply_sha256 = reply_sequence_sha256(&rows);
    let reading = summarize_replies(rows)?;
    let elapsed = started.elapsed();
    Ok(TimedReplyReading {
        reading,
        rows: row_readings,
        reply_sha256,
        wall_elapsed_us: elapsed.as_micros().try_into().unwrap_or(u64::MAX),
        generated_tokens_per_second: reading.generated_tokens as f64
            / elapsed.as_secs_f64().max(1e-12),
    })
}

fn generate_memory_target_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
) -> Result<GeneratedReply> {
    let memory = read_memory_uorlut01(artifact)
        .map_err(|error| invalid(format!("complete-reply memory: {error}")))?
        .ok_or_else(|| invalid("memory-target replies require turn memory"))?;
    let k = memory.record.k;
    let key_window = memory.record.key_window;
    let reply_turn = memory.record.reply_turn;
    let opening = memory
        .opening
        .keys
        .into_iter()
        .zip(memory.opening.values)
        .collect::<BTreeMap<_, _>>();
    let reply = memory.reply.map(|table| {
        table
            .keys
            .into_iter()
            .zip(table.values)
            .collect::<BTreeMap<_, _>>()
    });
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("memory-target forward: {error}")))?;
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "memory-target reply prompt, cap or artifact context is invalid",
        ));
    }
    let mut first = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("memory-target prompt step: {error}")))?;
        if index + 1 == prompt.len() {
            first = Some(step.token);
        }
    }
    let opening_key = prompt.iter().copied().take(k).collect::<Vec<_>>();
    let mut next = opening
        .get(&opening_key)
        .copied()
        .or(first)
        .ok_or_else(|| invalid("memory-target prompt is empty"))?;
    let mut tokens = Vec::with_capacity(cap);
    loop {
        let stop = reply_stop(next, tokens.len(), prompt.len(), cap, context);
        if stop != Some(ReplyEnd::Bos) {
            tokens.push(next);
        }
        if let Some(stop) = stop {
            return Ok(GeneratedReply {
                tokens,
                stop: match stop {
                    ReplyEnd::Bos => "bos",
                    ReplyEnd::Length => "cap",
                    ReplyEnd::Context => "context",
                },
            });
        }
        let fallback = forward
            .step(next, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("memory-target generated step: {error}")))?
            .token;
        next = if let (Some(reply), Some(turn)) = (&reply, reply_turn) {
            let sample = MemoryWeightSample {
                prompt: prompt.to_vec(),
                reply: tokens.clone(),
            };
            let key = memory_reply_key(&sample, tokens.len() - 1, turn, key_window)?;
            reply.get(&key).copied().unwrap_or(fallback)
        } else {
            fallback
        };
    }
}

fn summarize_replies(rows: Vec<(GeneratedReply, Vec<u32>, bool)>) -> Result<ReplyReading> {
    let mut reading = ReplyReading {
        replies: rows.len() as u64,
        ..ReplyReading::default()
    };
    let mut distinct = BTreeSet::new();
    for (generated, reference, reference_reachable) in rows {
        let reference = normalized_reference(&reference);
        match generated.stop {
            "bos" => reading.bos_complete = add_count(reading.bos_complete, 1)?,
            "cap" => reading.cap_stopped = add_count(reading.cap_stopped, 1)?,
            "context" => reading.context_stopped = add_count(reading.context_stopped, 1)?,
            other => return Err(invalid(format!("unknown complete-reply stop {other}"))),
        }
        reading.exact_reference = add_count(
            reading.exact_reference,
            u64::from(generated.tokens == reference),
        )?;
        reading.reference_reachable =
            add_count(reading.reference_reachable, u64::from(reference_reachable))?;
        reading.opening_correct = add_count(
            reading.opening_correct,
            u64::from(matches!(
                (generated.tokens.first(), reference.first()),
                (Some(generated), Some(reference)) if generated == reference
            )),
        )?;
        let non_empty = !generated.tokens.is_empty();
        reading.non_empty = add_count(reading.non_empty, u64::from(non_empty))?;
        if non_empty {
            distinct.insert(generated.tokens.clone());
        }
        reading.repeated_4gram = add_count(
            reading.repeated_4gram,
            u64::from(has_repeated_4gram(&generated.tokens)),
        )?;
        reading.generated_tokens =
            add_count(reading.generated_tokens, generated.tokens.len() as u64)?;
        reading.reference_tokens = add_count(reading.reference_tokens, reference.len() as u64)?;
    }
    reading.distinct_non_empty = distinct.len() as u64;
    Ok(reading)
}

fn bounded_reply(
    first: u32,
    prompt_len: usize,
    cap: usize,
    context: usize,
    mut step: impl FnMut(u32) -> Result<u32>,
) -> Result<GeneratedReply> {
    let mut tokens = Vec::with_capacity(cap);
    let mut next = first;
    loop {
        let stop = reply_stop(next, tokens.len(), prompt_len, cap, context);
        if stop != Some(ReplyEnd::Bos) {
            tokens.push(next);
        }
        if let Some(stop) = stop {
            return Ok(GeneratedReply {
                tokens,
                stop: match stop {
                    ReplyEnd::Bos => "bos",
                    ReplyEnd::Length => "cap",
                    ReplyEnd::Context => "context",
                },
            });
        }
        next = step(next)?;
    }
}

fn has_repeated_4gram(tokens: &[u32]) -> bool {
    let mut seen = BTreeSet::new();
    tokens.windows(4).any(|window| !seen.insert(window))
}

fn reply_sequence_sha256(rows: &[(GeneratedReply, Vec<u32>, bool)]) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(rows.len() as u64).to_le_bytes());
    for (reply, _, _) in rows {
        let stop = match reply.stop {
            "bos" => 0u8,
            "cap" => 1,
            "context" => 2,
            _ => 3,
        };
        bytes.push(stop);
        bytes.extend_from_slice(&(reply.tokens.len() as u64).to_le_bytes());
        for token in &reply.tokens {
            bytes.extend_from_slice(&token.to_le_bytes());
        }
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn normalized_reference(reference: &[u32]) -> &[u32] {
    reference
        .strip_suffix(&[POINT_KEY_BOS])
        .unwrap_or(reference)
}

fn reference_reachable(
    prompt_len: usize,
    reference_len: usize,
    cap: usize,
    context: usize,
) -> bool {
    reference_len <= cap
        && prompt_len
            .checked_add(reference_len)
            .is_some_and(|length| length <= context)
}

fn add_count(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid("complete-reply counter overflow"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{add_turn_memory_uorlut01, derive_turn_memory};
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};
    use uor_r4_integer::stack::{
        add_point_key_uorlut01, export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot,
        PointKeyImage,
    };

    fn artifact() -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            931,
            FbsSynth {
                vocab: crate::VOCABULARY,
                d: 64,
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

    #[test]
    fn bounded_reply_feeds_only_generated_tokens_and_stops_at_bos() {
        let mut fed = Vec::new();
        let reply = bounded_reply(5, 3, 8, 20, |token| {
            fed.push(token);
            Ok(match token {
                5 => 7,
                _ => POINT_KEY_BOS,
            })
        })
        .unwrap();
        assert_eq!(reply.tokens, vec![5, 7]);
        assert_eq!(reply.stop, "bos");
        assert_eq!(fed, vec![5, 7]);
    }

    #[test]
    fn bounded_reply_stops_at_its_cap_or_context_deterministically() {
        let capped = bounded_reply(5, 3, 3, 20, |token| Ok(token + 1)).unwrap();
        assert_eq!(capped.tokens, vec![5, 6, 7]);
        assert_eq!(capped.stop, "cap");

        let context = bounded_reply(5, 15, 8, 17, |token| Ok(token + 1)).unwrap();
        assert_eq!(context.tokens, vec![5, 6]);
        assert_eq!(context.stop, "context");
    }

    #[test]
    fn summary_counts_exact_opening_distinct_and_repeated_replies() {
        let rows = vec![
            (
                GeneratedReply {
                    tokens: vec![1, 2, 3, 4, 1, 2, 3, 4],
                    stop: "bos",
                },
                vec![1, 2, 3, 4, 1, 2, 3, 4, POINT_KEY_BOS],
                true,
            ),
            (
                GeneratedReply {
                    tokens: vec![9, 8],
                    stop: "cap",
                },
                vec![9, 7],
                true,
            ),
            (
                GeneratedReply {
                    tokens: vec![9, 8],
                    stop: "context",
                },
                vec![1],
                false,
            ),
        ];
        let reading = summarize_replies(rows).unwrap();
        assert_eq!(reading.replies, 3);
        assert_eq!(reading.bos_complete, 1);
        assert_eq!(reading.cap_stopped, 1);
        assert_eq!(reading.context_stopped, 1);
        assert_eq!(reading.exact_reference, 1);
        assert_eq!(reading.reference_reachable, 2);
        assert_eq!(reading.opening_correct, 2);
        assert_eq!(reading.non_empty, 3);
        assert_eq!(reading.distinct_non_empty, 2);
        assert_eq!(reading.repeated_4gram, 1);
        assert_eq!(reading.generated_tokens, 12);
        assert_eq!(reading.reference_tokens, 11);
    }

    #[test]
    fn reference_reachability_has_exact_cap_and_context_boundaries() {
        assert!(reference_reachable(10, 6, 6, 16), "exactly cap and context");
        assert!(!reference_reachable(10, 7, 6, 20), "cap plus one");
        assert!(!reference_reachable(10, 7, 8, 16), "context plus one");
    }

    #[test]
    fn reply_digest_and_normalization_are_deterministic() {
        let rows = vec![(
            GeneratedReply {
                tokens: vec![4, 5],
                stop: "bos",
            },
            vec![4, 5, POINT_KEY_BOS],
            true,
        )];
        let first = reply_sequence_sha256(&rows);
        assert_eq!(first, reply_sequence_sha256(&rows));
        assert_eq!(normalized_reference(&rows[0].1), &[4, 5]);

        let mut changed_token = rows.clone();
        changed_token[0].0.tokens[1] = 6;
        assert_ne!(first, reply_sequence_sha256(&changed_token));

        let mut changed_stop = rows;
        changed_stop[0].0.stop = "cap";
        assert_ne!(first, reply_sequence_sha256(&changed_stop));
    }

    #[test]
    fn complete_reply_evaluation_runs_the_exported_cpu_path_deterministically() {
        let artifact = artifact();
        let samples = vec![MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, 77, 99],
        }];
        let first = evaluate_replies(&artifact, &samples, 4).unwrap();
        let second = evaluate_replies(&artifact, &samples, 4).unwrap();
        assert_eq!(first.reading, second.reading);
        assert_eq!(first.reply_sha256, second.reply_sha256);
        assert_eq!(first.reading.replies, 1);
        assert_eq!(first.reading.non_empty, 1);
        let generated = generate_reply(&artifact, &samples[0].prompt, 4).unwrap();
        assert_eq!(
            generated.tokens.len() as u64,
            first.reading.generated_tokens
        );
    }

    #[test]
    fn exact_memory_targets_drive_a_complete_reply_with_forward_fallback() {
        let samples = vec![MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, 77, POINT_KEY_BOS],
        }];
        let memory = derive_turn_memory(&samples, 64, 3, 16, 3).unwrap();
        let artifact = add_turn_memory_uorlut01(&artifact(), &memory).unwrap();
        let reading = evaluate_memory_target_replies(&artifact, &samples, 4).unwrap();
        assert_eq!(reading.reading.replies, 1);
        assert_eq!(reading.reading.bos_complete, 1);
        assert_eq!(reading.reading.exact_reference, 1);
        assert_eq!(reading.reading.reference_reachable, 1);
        assert_eq!(reading.reading.opening_correct, 1);
    }

    #[test]
    fn memory_off_complete_reply_ignores_an_artifact_point_key() {
        let artifact = add_turn_memory_uorlut01(
            &artifact(),
            &FbsMemorySnapshot {
                k: 3,
                key_window: 16,
                opening: vec![FbsMemoryEntry {
                    key: vec![1, 2, 3],
                    value: 4,
                }],
                opening_weight: vec![0.0; 64],
                reply: None,
            },
        )
        .unwrap();
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let mut query = None;
        for (index, token) in [10, 11, 12].into_iter().enumerate() {
            if index + 1 == 3 {
                forward.mark_reply_start();
            }
            let step = forward.step(token, CpuFbsHead::Writer).unwrap();
            if index + 1 == 3 {
                query = Some(step.query_q12);
            }
        }
        let artifact = add_point_key_uorlut01(
            &artifact,
            &PointKeyImage {
                width: 64,
                points: query
                    .unwrap()
                    .into_iter()
                    .map(|value| i16::try_from(value).unwrap())
                    .collect(),
                targets: vec![99],
                vote_k: 1,
            },
        )
        .unwrap();
        let on = generate_reply(&artifact, &[10, 11, 12], 2).unwrap();
        let off = generate_reply_memory_off(&artifact, &[10, 11, 12], 2).unwrap();
        assert_eq!(on.tokens.first(), Some(&99));
        assert_ne!(off.tokens.first(), Some(&99));
        let evaluated = evaluate_replies_memory_off(
            &artifact,
            &[MemoryWeightSample {
                prompt: vec![10, 11, 12],
                reply: off.tokens.clone(),
            }],
            2,
        )
        .unwrap();
        assert_eq!(evaluated.reading.exact_reference, 1);
    }

    #[test]
    fn termination_diagnostics_preserve_reply_sequences_and_explain_bos() {
        let artifact = artifact();
        let samples = vec![MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, 77, 99],
        }];
        let ordinary = evaluate_replies(&artifact, &samples, 3).unwrap();
        let diagnostic = evaluate_termination_diagnostics(&artifact, &samples, 3, true).unwrap();
        assert_eq!(diagnostic.reply_sha256, ordinary.reply_sha256);
        assert_eq!(diagnostic.reading.rows, 1);
        assert!(!diagnostic.steps.is_empty());
        for (index, step) in diagnostic.steps.iter().enumerate() {
            assert_eq!(step.row, 0);
            assert_eq!(step.generated_step, index);
            assert!(step.bos_distance >= 0);
            assert!(step.chosen_distance >= 0);
            assert_eq!(
                step.bos_minus_chosen_distance,
                step.bos_distance - step.chosen_distance
            );
            assert_eq!(step.bos_admitted, step.bos_rank.is_some());
            assert!(step
                .bos_rank
                .is_none_or(|rank| (1..=step.candidate_count).contains(&rank)));
        }
    }

    #[test]
    fn termination_classification_partitions_mixed_steps_exactly() {
        let step =
            |row, generated_step, bos_admitted, bos_rank, chosen_bos| TerminationStepReading {
                row,
                generated_step,
                generated_token: chosen_bos,
                writer_token: 0,
                bos_admitted,
                bos_rank,
                bos_top8: bos_rank.is_some_and(|rank| rank <= 8),
                candidate_count: 8,
                bos_distance: 10,
                chosen_distance: 9,
                bos_minus_chosen_distance: 1,
            };
        let reading = summarize_termination(&[
            step(0, 0, true, Some(1), 7),
            step(0, 1, true, Some(2), 7),
            step(1, 0, false, None, 7),
            step(1, 1, false, None, 7),
        ])
        .unwrap();
        assert_eq!(reading.decisions, 4);
        assert_eq!(reading.bos_admitted, 2);
        assert_eq!(reading.bos_top1, 1);
        assert_eq!(reading.chosen_bos, 0);
        assert_eq!(reading.never_admitted, 2);
        assert_eq!(reading.admitted_not_top1, 1);
        assert_eq!(reading.top1_suppressed, 1);
        assert_eq!(reading.classification, Some(TerminationClass::Mixed));
    }

    #[test]
    fn termination_classification_covers_every_terminal_branch() {
        let step = |row, admitted, rank, generated| TerminationStepReading {
            row,
            generated_step: 0,
            generated_token: generated,
            writer_token: 0,
            bos_admitted: admitted,
            bos_rank: rank,
            bos_top8: rank.is_some_and(|rank| rank <= 8),
            candidate_count: 8,
            bos_distance: 10,
            chosen_distance: 9,
            bos_minus_chosen_distance: 1,
        };
        assert_eq!(
            summarize_termination(&[]).unwrap().classification,
            Some(TerminationClass::NoDecisions)
        );
        assert_eq!(
            summarize_termination(&[step(0, false, None, 7), step(1, false, None, 7)])
                .unwrap()
                .classification,
            Some(TerminationClass::NeverAdmitted)
        );
        assert_eq!(
            summarize_termination(&[step(0, true, Some(2), 7), step(1, true, Some(3), 7)])
                .unwrap()
                .classification,
            Some(TerminationClass::AdmittedNotTop1)
        );
        assert_eq!(
            summarize_termination(&[step(0, true, Some(1), 7), step(1, true, Some(1), 7)])
                .unwrap()
                .classification,
            Some(TerminationClass::Top1Suppressed)
        );
        assert_eq!(
            summarize_termination(&[step(0, true, Some(1), 0), step(1, true, Some(1), 0)])
                .unwrap()
                .classification,
            Some(TerminationClass::AllChosenBos)
        );
    }
}
