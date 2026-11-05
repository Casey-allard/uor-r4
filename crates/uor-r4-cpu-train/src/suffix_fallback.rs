//! Confident content-suffix fallback that preserves local exact copy.
//!
//! The table contains reply continuations only, never decoder BOS. Local
//! `Longest32To1` keeps priority; a global suffix may supply a token only when
//! local copy has no candidate and the suffix has a strict-majority successor.
//! Stopping remains the independent true reply-boundary memory.

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

pub const SUFFIX_FALLBACK_MIN_OBSERVATIONS: u64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuffixFallbackArm {
    LocalThenContent32,
    LocalThenContentDepth1,
    LocalThenShuffledContent32,
    Content32Only,
    LocalLongest32Boundary,
    WriterBoundary,
    ReferenceLengthOracle,
}

impl SuffixFallbackArm {
    pub const EVALUATION_ARMS: [Self; 7] = [
        Self::LocalThenContent32,
        Self::LocalThenContentDepth1,
        Self::LocalThenShuffledContent32,
        Self::Content32Only,
        Self::LocalLongest32Boundary,
        Self::WriterBoundary,
        Self::ReferenceLengthOracle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::LocalThenContent32 => "local_then_content_32",
            Self::LocalThenContentDepth1 => "local_then_content_depth_1",
            Self::LocalThenShuffledContent32 => "local_then_shuffled_content_32",
            Self::Content32Only => "content_32_only",
            Self::LocalLongest32Boundary => "local_longest_32_boundary",
            Self::WriterBoundary => "writer_boundary",
            Self::ReferenceLengthOracle => "reference_length_oracle",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SuffixFallbackStep {
    pub row: usize,
    pub generated_step: usize,
    pub writer_token: u32,
    pub selected_token: u32,
    pub local_copy_depth: Option<usize>,
    pub suffix_depth: Option<usize>,
    pub suffix_observations: u64,
    pub suffix_successor_count: u64,
    pub suffix_distinct_targets: u64,
    pub boundary_depth: Option<usize>,
    pub boundary_terminal: u64,
    pub boundary_nonterminal: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SuffixFallbackReading {
    pub arm: SuffixFallbackArm,
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
    pub local_copy_decisions: u64,
    pub suffix_decisions: u64,
    pub writer_decisions: u64,
    pub boundary_stops: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TimedSuffixFallbackReading {
    pub reading: SuffixFallbackReading,
    pub rows: Vec<CopyKeyReplyRow>,
    pub steps: Vec<SuffixFallbackStep>,
    pub reply_sha256: String,
    pub decision_sha256: String,
    pub wall_elapsed_us: u64,
    pub generated_tokens_per_second: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SuffixFallbackChanges {
    pub common_steps: u64,
    pub changed_steps: u64,
    pub divergent_rows: u64,
    pub first_changed_step: Option<(usize, usize)>,
}

#[allow(clippy::too_many_arguments)]
pub fn generate_suffix_fallback_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    reference_length: usize,
    arm: SuffixFallbackArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    boundary: &ReplyBoundaryMemory,
    seed: u64,
    row: usize,
) -> Result<(GeneratedReply, Vec<SuffixFallbackStep>)> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("suffix-fallback forward: {error}")))?;
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "suffix-fallback prompt, cap or artifact context is invalid",
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
            .map_err(|error| invalid(format!("suffix-fallback prompt step: {error}")))?;
        stream.push(token);
        if index + 1 == prompt.len() {
            next = Some(step.token);
        }
    }
    let mut writer_token = next.ok_or_else(|| invalid("suffix-fallback prompt is empty"))?;
    let mut tokens = Vec::with_capacity(cap);
    let mut steps = Vec::with_capacity(cap);

    loop {
        let boundary_hit = if arm == SuffixFallbackArm::ReferenceLengthOracle {
            (tokens.len() == reference_length).then_some(crate::reply_boundary::ReplyBoundaryHit {
                depth: 0,
                terminal: 1,
                nonterminal: 0,
            })
        } else {
            boundary.stop_decision(&tokens)
        };
        if let Some(hit) = boundary_hit {
            steps.push(SuffixFallbackStep {
                row,
                generated_step: tokens.len(),
                writer_token,
                selected_token: POINT_KEY_BOS,
                local_copy_depth: None,
                suffix_depth: None,
                suffix_observations: 0,
                suffix_successor_count: 0,
                suffix_distinct_targets: 0,
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

        let local = matches!(
            arm,
            SuffixFallbackArm::LocalThenContent32
                | SuffixFallbackArm::LocalThenContentDepth1
                | SuffixFallbackArm::LocalThenShuffledContent32
                | SuffixFallbackArm::LocalLongest32Boundary
                | SuffixFallbackArm::ReferenceLengthOracle
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

        let prediction = match arm {
            SuffixFallbackArm::LocalThenContent32
            | SuffixFallbackArm::LocalThenContentDepth1
            | SuffixFallbackArm::LocalThenShuffledContent32
                if local_depth.is_none() =>
            {
                let maximum = if arm == SuffixFallbackArm::LocalThenContentDepth1 {
                    1
                } else {
                    SUFFIX_MEMORY_MAX_DEPTH
                };
                suffix_memory
                    .ok_or_else(|| invalid("suffix-fallback arm has no table"))?
                    .predict_confident(&tokens, maximum, SUFFIX_FALLBACK_MIN_OBSERVATIONS)
            }
            SuffixFallbackArm::Content32Only => suffix_memory
                .ok_or_else(|| invalid("content-only arm has no table"))?
                .predict_confident(
                    &tokens,
                    SUFFIX_MEMORY_MAX_DEPTH,
                    SUFFIX_FALLBACK_MIN_OBSERVATIONS,
                ),
            _ => None,
        };

        let selected_token = if local_depth.is_some() {
            local
                .as_ref()
                .expect("local depth requires a local decision")
                .token
        } else {
            prediction.map_or(writer_token, |hit| hit.token)
        };

        steps.push(SuffixFallbackStep {
            row,
            generated_step: tokens.len(),
            writer_token,
            selected_token,
            local_copy_depth: local_depth,
            suffix_depth: prediction.map(|hit| hit.depth),
            suffix_observations: prediction.map_or(0, |hit| hit.observations),
            suffix_successor_count: prediction.map_or(0, |hit| hit.successor_count),
            suffix_distinct_targets: prediction.map_or(0, |hit| hit.distinct_targets),
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
            .map_err(|error| invalid(format!("suffix-fallback generated step: {error}")))?;
        writer_token = step.token;
    }
}

pub fn evaluate_suffix_fallback_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    arm: SuffixFallbackArm,
    suffix_memory: Option<&SuffixMemoryTable>,
    boundary: &ReplyBoundaryMemory,
    seed: u64,
) -> Result<TimedSuffixFallbackReading> {
    if samples.is_empty() || cap == 0 || cap > 64 {
        return Err(invalid("suffix-fallback reply inputs differ"));
    }
    let context = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("suffix-fallback context: {error}")))?
        .context_length();
    let started = std::time::Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply).to_vec();
            if sample.prompt.len() > context || reference.len() > cap {
                return Err(invalid("suffix-fallback sample exceeds its context or cap"));
            }
            let (generated, steps) = generate_suffix_fallback_reply(
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
            let local_copy_decisions = steps
                .iter()
                .filter(|step| step.local_copy_depth.is_some())
                .count() as u64;
            let fallback_decisions = steps.len() as u64 - local_copy_decisions;
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
                copy_decisions: local_copy_decisions,
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
    Ok(TimedSuffixFallbackReading {
        reading,
        rows,
        steps,
        reply_sha256,
        decision_sha256,
        wall_elapsed_us,
        generated_tokens_per_second,
    })
}

pub fn compare_suffix_fallback_rows(
    candidate: &[CopyKeyReplyRow],
    control: &[CopyKeyReplyRow],
    metric: CopyKeyReplyMetric,
) -> Result<crate::copy_key_reply::CopyKeyReplyComparison> {
    compare_copy_key_reply_rows(candidate, control, metric)
}

pub fn suffix_fallback_changes(
    candidate: &TimedSuffixFallbackReading,
    control: &TimedSuffixFallbackReading,
) -> SuffixFallbackChanges {
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
    SuffixFallbackChanges {
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
    arm: SuffixFallbackArm,
    rows: &[CopyKeyReplyRow],
    steps: &[SuffixFallbackStep],
) -> Result<SuffixFallbackReading> {
    let mut reading = SuffixFallbackReading {
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
        local_copy_decisions: 0,
        suffix_decisions: 0,
        writer_decisions: 0,
        boundary_stops: 0,
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
        if step.local_copy_depth.is_some() {
            reading.local_copy_decisions = checked_add(reading.local_copy_decisions, 1)?;
        } else if step.suffix_depth.is_some() {
            reading.suffix_decisions = checked_add(reading.suffix_decisions, 1)?;
        } else if step.boundary_depth.is_none() {
            reading.writer_decisions = checked_add(reading.writer_decisions, 1)?;
        }
        reading.boundary_stops = checked_add(
            reading.boundary_stops,
            u64::from(step.boundary_depth.is_some()),
        )?;
    }
    Ok(reading)
}

fn checked_add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid("suffix-fallback counter overflows"))
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

fn decision_hash(steps: &[SuffixFallbackStep]) -> String {
    let mut hasher = blake3::Hasher::new();
    for step in steps {
        hasher.update(&(step.row as u64).to_le_bytes());
        hasher.update(&(step.generated_step as u64).to_le_bytes());
        hasher.update(&step.writer_token.to_le_bytes());
        hasher.update(&step.selected_token.to_le_bytes());
        hasher.update(&(step.local_copy_depth.unwrap_or(0) as u16).to_le_bytes());
        hasher.update(&(step.suffix_depth.unwrap_or(0) as u16).to_le_bytes());
        hasher.update(&step.suffix_observations.to_le_bytes());
        hasher.update(&step.suffix_successor_count.to_le_bytes());
        hasher.update(&step.suffix_distinct_targets.to_le_bytes());
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
    use crate::suffix_memory::{derive_suffix_content_memory, shuffle_suffix_memory};
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    fn sample(reply: &[u32]) -> MemoryWeightSample {
        MemoryWeightSample {
            prompt: vec![9],
            reply: reply.to_vec(),
        }
    }

    fn artifact() -> Vec<u8> {
        export_fbs_trunk_uorlut01(&synthetic_fbs_trunk_with(
            196,
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
        derive_reply_boundary_memory(&[sample(&[40, 10, 20, 30, 1])]).unwrap()
    }

    #[test]
    fn content_fallback_never_selects_bos_as_a_body_token() {
        let table = derive_suffix_content_memory(&[
            sample(&[10, 20, 30, 40, 1]),
            sample(&[50, 20, 30, 41, 1]),
        ])
        .unwrap();
        assert_eq!(table.census().bos_transitions, 0);
        let body_tokens = generate_suffix_fallback_reply(
            &artifact(),
            &[10, 20, 30, 40, 10, 20, 30],
            16,
            8,
            SuffixFallbackArm::Content32Only,
            Some(&table),
            &boundary(),
            196,
            0,
        )
        .unwrap()
        .1
        .into_iter()
        .filter(|step| step.boundary_depth.is_none())
        .map(|step| step.selected_token);
        assert!(body_tokens.into_iter().all(|token| token != POINT_KEY_BOS));
    }

    #[test]
    fn treatment_preserves_local_copy_when_it_has_a_candidate() {
        let artifact = artifact();
        let local = evaluate_suffix_fallback_replies(
            &artifact,
            &[MemoryWeightSample {
                prompt: vec![10, 20, 30, 40, 10, 20, 30],
                reply: vec![40, 10, 20, 30, 1],
            }],
            16,
            SuffixFallbackArm::LocalLongest32Boundary,
            None,
            &boundary(),
            196,
        )
        .unwrap();
        let table = derive_suffix_content_memory(&[
            sample(&[10, 20, 30, 40, 1]),
            sample(&[10, 20, 30, 41, 1]),
            sample(&[10, 20, 30, 41, 1]),
        ])
        .unwrap();
        let treatment = evaluate_suffix_fallback_replies(
            &artifact,
            &[MemoryWeightSample {
                prompt: vec![10, 20, 30, 40, 10, 20, 30],
                reply: vec![40, 10, 20, 30, 1],
            }],
            16,
            SuffixFallbackArm::LocalThenContent32,
            Some(&table),
            &boundary(),
            196,
        )
        .unwrap();
        let local_steps = local
            .steps
            .iter()
            .filter(|step| step.local_copy_depth.is_some())
            .map(|step| ((step.row, step.generated_step), step.selected_token))
            .collect::<BTreeMap<_, _>>();
        assert!(!local_steps.is_empty());
        let treatment_steps = treatment
            .steps
            .iter()
            .map(|step| ((step.row, step.generated_step), step))
            .collect::<BTreeMap<_, _>>();
        let mut compared = 0;
        for (key, local_token) in local_steps {
            let Some(step) = treatment_steps.get(&key) else {
                continue;
            };
            compared += 1;
            assert!(step.local_copy_depth.is_some());
            assert_eq!(
                step.selected_token, local_token,
                "local candidate was displaced at {key:?}"
            );
        }
        assert!(compared > 0);
    }

    #[test]
    fn treatment_uses_confident_suffix_after_an_uncovered_writer_step_and_shuffle_diverges() {
        let artifact = artifact();
        let boundary = boundary();
        let mut fixture = None;
        for base in (10u32..400).step_by(16) {
            let prompt = (0..7).map(|offset| base + offset).collect::<Vec<_>>();
            let single = [MemoryWeightSample {
                prompt: prompt.clone(),
                reply: vec![400, 401, 402, 403, 1],
            }];
            let writer = evaluate_suffix_fallback_replies(
                &artifact,
                &single,
                16,
                SuffixFallbackArm::WriterBoundary,
                None,
                &boundary,
                196,
            )
            .unwrap();
            let local = evaluate_suffix_fallback_replies(
                &artifact,
                &single,
                16,
                SuffixFallbackArm::LocalLongest32Boundary,
                None,
                &boundary,
                196,
            )
            .unwrap();
            let Some(writer_first) = writer.steps.iter().find(|step| step.generated_step == 0)
            else {
                continue;
            };
            let Some(writer_second) = writer.steps.iter().find(|step| step.generated_step == 1)
            else {
                continue;
            };
            let Some(local_step) = local.steps.iter().find(|step| step.generated_step == 1) else {
                continue;
            };
            if local_step.local_copy_depth.is_none() {
                fixture = Some((
                    prompt,
                    writer_first.selected_token,
                    writer_second.writer_token,
                ));
                break;
            }
        }
        let (prompt, first, writer_second) =
            fixture.expect("synthetic artifact needs an uncovered second step");
        let target = (10..512)
            .find(|token| *token != first && *token != writer_second)
            .expect("fallback target");
        let first_reply = [
            MemoryWeightSample {
                prompt: prompt.clone(),
                reply: vec![first, target, 403, 404],
            },
            MemoryWeightSample {
                prompt: prompt.clone(),
                reply: vec![first, target, 403, 404],
            },
        ];
        let other = (10..512)
            .find(|token| *token != first && *token != target)
            .expect("second suffix key");
        let other_target = (10..512)
            .find(|token| *token != first && *token != other && *token != target)
            .expect("shuffle target");
        let mut training = first_reply.to_vec();
        training.push(MemoryWeightSample {
            prompt: prompt.clone(),
            reply: vec![other, other_target, 411, 412],
        });
        training.push(MemoryWeightSample {
            prompt: prompt.clone(),
            reply: vec![other, other_target, 411, 412],
        });
        let table = derive_suffix_content_memory(&training).unwrap();
        let prediction = table.predict(&[first], SUFFIX_MEMORY_MAX_DEPTH).unwrap();
        assert_eq!(prediction.token, target);
        let (shuffled, seed) = (0..64)
            .find_map(|seed| {
                let (control, _) = shuffle_suffix_memory(&table, seed).unwrap();
                let prediction = control.predict(&[first], SUFFIX_MEMORY_MAX_DEPTH)?;
                (prediction.token != target).then_some((control, seed))
            })
            .expect("shuffle needs a changed first-suffix target");
        let sample = [MemoryWeightSample {
            prompt: prompt.clone(),
            reply: vec![400, 401, target, 403, 404, 1],
        }];
        let writer = evaluate_suffix_fallback_replies(
            &artifact,
            &sample,
            16,
            SuffixFallbackArm::WriterBoundary,
            None,
            &boundary,
            seed,
        )
        .unwrap();
        let treatment = evaluate_suffix_fallback_replies(
            &artifact,
            &sample,
            16,
            SuffixFallbackArm::LocalThenContent32,
            Some(&table),
            &boundary,
            seed,
        )
        .unwrap();
        let shuffled_reply = evaluate_suffix_fallback_replies(
            &artifact,
            &sample,
            16,
            SuffixFallbackArm::LocalThenShuffledContent32,
            Some(&shuffled),
            &boundary,
            seed,
        )
        .unwrap();
        let writer_step = writer
            .steps
            .iter()
            .find(|step| step.generated_step == 1)
            .unwrap();
        let treatment_step = treatment
            .steps
            .iter()
            .find(|step| step.generated_step == 1)
            .unwrap();
        let shuffled_step = shuffled_reply
            .steps
            .iter()
            .find(|step| step.generated_step == 1)
            .unwrap();
        assert_eq!(treatment_step.selected_token, target);
        assert_ne!(treatment_step.selected_token, writer_step.writer_token);
        assert_eq!(treatment_step.local_copy_depth, None);
        assert_eq!(treatment_step.suffix_depth, Some(1));
        assert_ne!(treatment_step.selected_token, shuffled_step.selected_token);
    }
}
