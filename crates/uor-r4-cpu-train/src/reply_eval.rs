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
const CORPUS_EOS: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplyReading {
    pub replies: u64,
    pub bos_complete: u64,
    pub cap_stopped: u64,
    pub context_stopped: u64,
    pub exact_reference: u64,
    pub reference_reachable: u64,
    pub premature_stop: u64,
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminationPointGate {
    pub width: usize,
    pub points: Vec<Vec<i16>>,
    pub radius: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminationGateTraining {
    pub trained: TerminationPointGate,
    pub random: TerminationPointGate,
    pub terminal_points_before_collision: usize,
    pub removed_collision_points: usize,
    pub max_terminal_neighbor_distance: u64,
    pub nearest_nonterminal_distance: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminationCellGate {
    pub width: usize,
    pub cells: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminationCellTraining {
    pub trained: TerminationCellGate,
    pub random: TerminationCellGate,
    pub terminal_cell_counts: BTreeMap<u16, u64>,
    pub nonterminal_cell_counts: BTreeMap<u16, u64>,
    pub excluded_ties: usize,
    pub excluded_nonterminal_majority: usize,
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

/// Derive a geometric termination-point gate from training examples and a
/// matched random nonterminal-point control.
///
/// Training replies end in corpus EOS. The query immediately before that EOS is
/// retained as the terminal point and an accepted match names the shared
/// decoder BOS stop sentinel. Terminal points that exactly equal a nonterminal
/// training query are removed as ambiguous. The radius is bounded by the
/// farthest terminal point's nearest terminal neighbour and one unit below the
/// nearest nonterminal training query.
pub fn derive_termination_point_gates(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    memory_enabled: bool,
    random_seed: u64,
) -> Result<TerminationGateTraining> {
    if samples.is_empty() {
        return Err(invalid("termination gate needs training samples"));
    }
    let model = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("termination gate artifact: {error}")))?;
    let width = model.width();
    let mut terminal = Vec::<Vec<i32>>::new();
    let mut nonterminal = Vec::<Vec<i32>>::new();
    for sample in samples {
        if sample.prompt.is_empty()
            || sample.reply.len() < 2
            || sample.reply.last() != Some(&CORPUS_EOS)
        {
            return Err(invalid(
                "termination gate training reply must end in corpus EOS",
            ));
        }
        let sequence = sample
            .prompt
            .iter()
            .chain(sample.reply[..sample.reply.len() - 1].iter())
            .copied()
            .collect::<Vec<_>>();
        let mut forward = CpuFbsForward::from_artifact(artifact)
            .map_err(|error| invalid(format!("termination gate forward: {error}")))?;
        if !memory_enabled {
            forward.disable_memory();
        }
        for (index, token) in sequence.iter().copied().enumerate() {
            if index + 1 == sample.prompt.len() {
                forward.mark_reply_start();
            }
            let step = forward
                .step(token, CpuFbsHead::Writer)
                .map_err(|error| invalid(format!("termination gate training step: {error}")))?;
            if index + 1 >= sample.prompt.len() {
                if index + 1 == sequence.len() {
                    terminal.push(step.query_q12);
                } else {
                    nonterminal.push(step.query_q12);
                }
            }
        }
    }
    let (terminal, removed_collision_points) =
        remove_nonterminal_collisions(terminal, &nonterminal);
    let terminal_points_before_collision = terminal.len() + removed_collision_points;
    let (radius, max_terminal_neighbor_distance, nearest_nonterminal_distance) =
        termination_gate_radius(&terminal, &nonterminal);
    let trained = TerminationPointGate {
        width,
        points: terminal
            .iter()
            .map(|query| gate_point(query, width))
            .collect::<Result<Vec<_>>>()?,
        radius,
    };
    let mut state = random_seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let random = TerminationPointGate {
        width,
        points: (0..trained.points.len())
            .map(|_| {
                if nonterminal.is_empty() {
                    Ok(vec![0; width])
                } else {
                    state ^= state >> 12;
                    state ^= state << 25;
                    state ^= state >> 27;
                    let draw = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
                    let row = usize::try_from(draw % nonterminal.len() as u64)
                        .map_err(|_| invalid("termination control row exceeds usize"))?;
                    gate_point(&nonterminal[row], width)
                }
            })
            .collect::<Result<Vec<_>>>()?,
        radius,
    };
    Ok(TerminationGateTraining {
        trained,
        random,
        terminal_points_before_collision,
        removed_collision_points,
        max_terminal_neighbor_distance,
        nearest_nonterminal_distance,
    })
}

fn remove_nonterminal_collisions(
    mut terminal: Vec<Vec<i32>>,
    nonterminal: &[Vec<i32>],
) -> (Vec<Vec<i32>>, usize) {
    let before = terminal.len();
    terminal.retain(|query| !nonterminal.contains(query));
    let removed = before - terminal.len();
    (terminal, removed)
}

fn termination_gate_radius(
    terminal: &[Vec<i32>],
    nonterminal: &[Vec<i32>],
) -> (u64, u64, Option<u64>) {
    let max_terminal_neighbor_distance = terminal
        .iter()
        .enumerate()
        .map(|(index, query)| {
            terminal
                .iter()
                .enumerate()
                .filter(|(other, _)| *other != index)
                .map(|(_, other)| point_l1(query, other))
                .min()
                .unwrap_or(0)
        })
        .max()
        .unwrap_or(0);
    let nearest_nonterminal_distance = nonterminal
        .iter()
        .flat_map(|query| terminal.iter().map(move |point| point_l1(query, point)))
        .min();
    let radius = max_terminal_neighbor_distance.min(
        nearest_nonterminal_distance
            .unwrap_or(u64::MAX)
            .saturating_sub(1),
    );
    (
        radius,
        max_terminal_neighbor_distance,
        nearest_nonterminal_distance,
    )
}

fn gate_point(query: &[i32], width: usize) -> Result<Vec<i16>> {
    if query.len() != width {
        return Err(invalid(
            "termination gate query width differs from the model",
        ));
    }
    query
        .iter()
        .map(|&value| {
            i16::try_from(value)
                .map_err(|_| invalid("termination gate query coordinate exceeds i16"))
        })
        .collect()
}

pub fn evaluate_termination_gate_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
    gate: &TerminationPointGate,
) -> Result<TimedReplyReading> {
    evaluate_generated(artifact, samples, cap, |prompt, cap| {
        generate_termination_gate_reply(artifact, prompt, cap, memory_enabled, gate)
    })
}

/// Derive a writer-cell termination gate and an equal-size deterministic control
/// from training examples only.
pub fn derive_termination_cell_gates(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    memory_enabled: bool,
    random_seed: u64,
) -> Result<TerminationCellTraining> {
    if samples.is_empty() {
        return Err(invalid("termination cell gate needs training samples"));
    }
    let model = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("termination cell gate artifact: {error}")))?;
    let width = model.width();
    let mut terminal_cell_counts = BTreeMap::<u16, u64>::new();
    let mut nonterminal_cell_counts = BTreeMap::<u16, u64>::new();
    for sample in samples {
        if sample.prompt.is_empty()
            || sample.reply.len() < 2
            || sample.reply.last() != Some(&CORPUS_EOS)
        {
            return Err(invalid(
                "termination cell gate training reply must end in corpus EOS",
            ));
        }
        let sequence = sample
            .prompt
            .iter()
            .chain(sample.reply[..sample.reply.len() - 1].iter())
            .copied()
            .collect::<Vec<_>>();
        let mut forward = CpuFbsForward::from_artifact(artifact)
            .map_err(|error| invalid(format!("termination cell gate forward: {error}")))?;
        if !memory_enabled {
            forward.disable_memory();
        }
        for (index, token) in sequence.iter().copied().enumerate() {
            if index + 1 == sample.prompt.len() {
                forward.mark_reply_start();
            }
            let step = forward.step(token, CpuFbsHead::Writer).map_err(|error| {
                invalid(format!("termination cell gate training step: {error}"))
            })?;
            if index + 1 >= sample.prompt.len() {
                let cell = model
                    .writer_query_cell(&step.query_q12)
                    .map_err(|error| invalid(format!("termination cell gate cell: {error}")))?;
                let counts = if index + 1 == sequence.len() {
                    &mut terminal_cell_counts
                } else {
                    &mut nonterminal_cell_counts
                };
                let count = counts.entry(cell).or_default();
                *count = count
                    .checked_add(1)
                    .ok_or_else(|| invalid("termination cell count overflow"))?;
            }
        }
    }
    let training = learn_termination_cells(
        width,
        terminal_cell_counts,
        nonterminal_cell_counts,
        random_seed,
    )?;
    Ok(training)
}

fn learn_termination_cells(
    width: usize,
    terminal_cell_counts: BTreeMap<u16, u64>,
    nonterminal_cell_counts: BTreeMap<u16, u64>,
    random_seed: u64,
) -> Result<TerminationCellTraining> {
    let mut cells = Vec::new();
    let (mut excluded_ties, mut excluded_nonterminal_majority) = (0usize, 0usize);
    for (&cell, &terminal) in &terminal_cell_counts {
        let nonterminal = nonterminal_cell_counts.get(&cell).copied().unwrap_or(0);
        if terminal > nonterminal {
            cells.push(cell);
        } else if terminal == nonterminal {
            excluded_ties = excluded_ties
                .checked_add(1)
                .ok_or_else(|| invalid("termination excluded-cell count overflow"))?;
        } else {
            excluded_nonterminal_majority = excluded_nonterminal_majority
                .checked_add(1)
                .ok_or_else(|| invalid("termination excluded-cell count overflow"))?;
        }
    }
    cells.sort_unstable();
    cells.dedup();
    let trained = TerminationCellGate {
        width,
        cells: cells.clone(),
    };
    let learned = cells.iter().copied().collect::<BTreeSet<_>>();
    let mut pool = terminal_cell_counts
        .keys()
        .chain(nonterminal_cell_counts.keys())
        .copied()
        .filter(|cell| !learned.contains(cell))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if pool.len() < cells.len() {
        return Err(invalid(
            "termination shuffled control has fewer distinct unselected cells than the trained gate",
        ));
    }
    let mut state = random_seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    for end in (1..pool.len()).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let draw = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        pool.swap(
            end,
            usize::try_from(draw % (end as u64 + 1))
                .map_err(|_| invalid("termination shuffled-control index exceeds usize"))?,
        );
    }
    pool.truncate(cells.len());
    pool.sort_unstable();
    Ok(TerminationCellTraining {
        trained,
        random: TerminationCellGate { width, cells: pool },
        terminal_cell_counts,
        nonterminal_cell_counts,
        excluded_ties,
        excluded_nonterminal_majority,
    })
}

pub fn evaluate_termination_cell_gate_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
    gate: &TerminationCellGate,
) -> Result<TimedReplyReading> {
    evaluate_generated(artifact, samples, cap, |prompt, cap| {
        generate_termination_cell_gate_reply(artifact, prompt, cap, memory_enabled, gate)
    })
}

fn generate_termination_cell_gate_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    memory_enabled: bool,
    gate: &TerminationCellGate,
) -> Result<GeneratedReply> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("termination cell gate forward: {error}")))?;
    if !memory_enabled {
        forward.disable_memory();
    }
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "termination cell gate prompt, cap or artifact context is invalid",
        ));
    }
    let mut next = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("termination cell gate prompt step: {error}")))?;
        if index + 1 == prompt.len() {
            next = Some(cell_name_or_writer(&forward, gate, &step)?);
        }
    }
    let mut next = next.ok_or_else(|| invalid("termination cell gate prompt is empty"))?;
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
        let step = forward
            .step(next, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("termination cell gate generated step: {error}")))?;
        next = cell_name_or_writer(&forward, gate, &step)?;
    }
}

fn cell_name_or_writer(
    forward: &CpuFbsForward,
    gate: &TerminationCellGate,
    step: &crate::CpuFbsStep,
) -> Result<u32> {
    if step.query_q12.len() != gate.width {
        return Err(invalid(
            "termination cell gate width differs from the served query",
        ));
    }
    Ok(
        if gate
            .cells
            .binary_search(
                &forward
                    .writer_query_cell(&step.query_q12)
                    .map_err(|error| invalid(format!("termination cell gate cell: {error}")))?,
            )
            .is_ok()
        {
            POINT_KEY_BOS
        } else {
            step.token
        },
    )
}

fn generate_termination_gate_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    memory_enabled: bool,
    gate: &TerminationPointGate,
) -> Result<GeneratedReply> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("termination gate forward: {error}")))?;
    if !memory_enabled {
        forward.disable_memory();
    }
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "termination gate prompt, cap or artifact context is invalid",
        ));
    }
    let mut next = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("termination gate prompt step: {error}")))?;
        if index + 1 == prompt.len() {
            next = Some(gate_name_or_writer(gate, &step)?);
        }
    }
    let mut next = next.ok_or_else(|| invalid("termination gate prompt is empty"))?;
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
        let step = forward
            .step(next, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("termination gate generated step: {error}")))?;
        next = gate_name_or_writer(gate, &step)?;
    }
}

fn gate_name_or_writer(gate: &TerminationPointGate, step: &crate::CpuFbsStep) -> Result<u32> {
    if step.query_q12.len() != gate.width || gate.points.iter().any(|row| row.len() != gate.width) {
        return Err(invalid(
            "termination gate width differs from the served query",
        ));
    }
    let nearest = gate
        .points
        .iter()
        .map(|point| {
            step.query_q12
                .iter()
                .zip(point)
                .fold(0u64, |sum, (&query, &point)| {
                    sum.saturating_add(u64::from(query.abs_diff(i32::from(point))))
                })
        })
        .min();
    Ok(if nearest.is_some_and(|distance| distance <= gate.radius) {
        POINT_KEY_BOS
    } else {
        step.token
    })
}

fn point_l1(left: &[i32], right: &[i32]) -> u64 {
    left.iter().zip(right).fold(0u64, |sum, (&left, &right)| {
        sum.saturating_add(u64::from(left.abs_diff(right)))
    })
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
        reading.premature_stop = add_count(
            reading.premature_stop,
            u64::from(generated.stop == "bos" && generated.tokens != reference),
        )?;
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

/// Remove one decoder stop marker from a complete-reply reference. Corpus
/// references use EOS id 1; retaining BOS removal keeps synthetic and older
/// callers compatible.
pub fn normalized_reference(reference: &[u32]) -> &[u32] {
    reference
        .strip_suffix(&[CORPUS_EOS])
        .or_else(|| reference.strip_suffix(&[POINT_KEY_BOS]))
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
    use crate::{add_turn_memory_uorlut01, derive_turn_memory, CpuFbsStep};
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
                vec![5],
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
        assert_eq!(normalized_reference(&[4, 5, CORPUS_EOS]), &[4, 5]);

        let mut changed_token = rows.clone();
        changed_token[0].0.tokens[1] = 6;
        assert_ne!(first, reply_sequence_sha256(&changed_token));

        let mut changed_stop = rows;
        changed_stop[0].0.stop = "cap";
        assert_ne!(first, reply_sequence_sha256(&changed_stop));
    }

    #[test]
    fn termination_gate_derivation_uses_reply_endpoints_and_removes_collisions() {
        let artifact = artifact();
        let samples = vec![
            MemoryWeightSample {
                prompt: vec![10, 11, 12],
                reply: vec![42, CORPUS_EOS],
            },
            MemoryWeightSample {
                prompt: vec![10, 11, 12],
                reply: vec![42, 77, CORPUS_EOS],
            },
        ];
        let training = derive_termination_point_gates(&artifact, &samples, true, 1).unwrap();
        assert_eq!(training.terminal_points_before_collision, 2);
        assert_eq!(training.removed_collision_points, 1);
        assert_eq!(training.trained.points.len(), 1);
        assert_eq!(training.random.points.len(), 1);
        assert_eq!(training.trained.width, 64);

        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let mut nonterminal = Vec::new();
        let mut terminal = None;
        for (index, token) in [10, 11, 12, 42, 77].into_iter().enumerate() {
            if index + 1 == 3 {
                forward.mark_reply_start();
            }
            let step = forward.step(token, CpuFbsHead::Writer).unwrap();
            if index + 1 >= 3 {
                if index + 1 == 5 {
                    terminal = Some(step.query_q12);
                } else {
                    nonterminal.push(step.query_q12);
                }
            }
        }
        let terminal = terminal.unwrap();
        assert_eq!(
            training.trained.points[0],
            terminal
                .iter()
                .map(|&value| i16::try_from(value).unwrap())
                .collect::<Vec<_>>()
        );
        assert!(
            training.random.points.iter().all(|point| {
                nonterminal.iter().any(|query| {
                    query
                        .iter()
                        .map(|&value| i16::try_from(value).unwrap())
                        .collect::<Vec<_>>()
                        == *point
                })
            }),
            "random={:?} nonterminal={nonterminal:?}",
            training.random.points
        );
        assert_ne!(training.trained, training.random);
        let bad = [MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, POINT_KEY_BOS],
        }];
        assert!(derive_termination_point_gates(&artifact, &bad, true, 1).is_err());
    }

    #[test]
    fn termination_radius_is_bounded_by_terminal_neighbors_and_nonterminals() {
        let terminal = vec![vec![10, 0], vec![20, 0]];
        let nonterminal = vec![vec![15, 0], vec![100, 0]];
        assert_eq!(
            termination_gate_radius(&terminal, &nonterminal),
            (4, 10, Some(5))
        );

        let one_terminal = vec![vec![10, 0]];
        assert_eq!(
            termination_gate_radius(&one_terminal, &nonterminal),
            (0, 0, Some(5))
        );
        assert_eq!(termination_gate_radius(&terminal, &[]), (10, 10, None));
    }

    #[test]
    fn termination_gate_names_bos_only_inside_the_radius() {
        let gate = TerminationPointGate {
            width: 4,
            points: vec![vec![10, 20, 30, 40]],
            radius: 1,
        };
        let exact = CpuFbsStep {
            token: 7,
            kept_cells: vec![1],
            query_q12: vec![10, 20, 30, 40],
        };
        assert_eq!(gate_name_or_writer(&gate, &exact).unwrap(), POINT_KEY_BOS);

        let adjacent = CpuFbsStep {
            query_q12: vec![11, 20, 30, 40],
            ..exact.clone()
        };
        assert_eq!(
            gate_name_or_writer(&gate, &adjacent).unwrap(),
            POINT_KEY_BOS
        );

        let outside = CpuFbsStep {
            query_q12: vec![12, 20, 30, 40],
            ..exact
        };
        assert_eq!(gate_name_or_writer(&gate, &outside).unwrap(), 7);
    }

    #[test]
    fn nonempty_termination_gate_stops_the_real_reply_loop_at_bos() {
        let artifact = artifact();
        let prompt = [10, 11, 12];
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let mut query = None;
        for (index, token) in prompt.into_iter().enumerate() {
            if index + 1 == prompt.len() {
                forward.mark_reply_start();
            }
            let step = forward.step(token, CpuFbsHead::Writer).unwrap();
            if index + 1 == prompt.len() {
                query = Some(step.query_q12);
            }
        }
        let gate = TerminationPointGate {
            width: 64,
            points: vec![query
                .unwrap()
                .into_iter()
                .map(|value| i16::try_from(value).unwrap())
                .collect()],
            radius: 0,
        };
        let reply = generate_termination_gate_reply(&artifact, &prompt, 4, true, &gate).unwrap();
        assert_eq!(reply.stop, "bos");
        assert!(reply.tokens.is_empty());
    }

    #[test]
    fn empty_termination_gate_preserves_the_no_gate_reply_digest() {
        let artifact = artifact();
        let samples = vec![MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, 77, 99],
        }];
        let gate = TerminationPointGate {
            width: 64,
            points: Vec::new(),
            radius: 0,
        };
        let no_gate = evaluate_replies(&artifact, &samples, 4).unwrap();
        let empty_gate =
            evaluate_termination_gate_replies(&artifact, &samples, 4, true, &gate).unwrap();
        assert_eq!(no_gate.reply_sha256, empty_gate.reply_sha256);
        assert_eq!(no_gate.reading, empty_gate.reading);
    }

    #[test]
    fn writer_cell_uses_the_first_and_last_orbit_roots() {
        let artifact = artifact();
        let forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let roots = forward.writer_roots();
        let mut query = vec![0i32; 64];
        query[..8].copy_from_slice(&roots[3]);
        query[56..].copy_from_slice(&roots[7]);
        assert_eq!(forward.writer_query_cell(&query).unwrap(), 55);
    }

    #[test]
    fn termination_cells_require_strict_terminal_density_and_build_a_shuffled_control() {
        let terminal = BTreeMap::from([(1, 3), (2, 2), (3, 4)]);
        let nonterminal = BTreeMap::from([(1, 2), (2, 2), (3, 6), (4, 1)]);
        let first = learn_termination_cells(64, terminal, nonterminal, 1).unwrap();
        assert_eq!(first.trained.cells, [1]);
        assert_eq!(first.excluded_ties, 1);
        assert_eq!(first.excluded_nonterminal_majority, 1);
        assert_eq!(first.random.cells.len(), 1);
        assert!(!first.random.cells.contains(&1));
        let second = learn_termination_cells(
            64,
            BTreeMap::from([(1, 3), (2, 2), (3, 4)]),
            BTreeMap::from([(1, 2), (2, 2), (3, 6), (4, 1)]),
            1,
        )
        .unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn termination_cell_gate_names_bos_only_inside_a_learned_cell() {
        let artifact = artifact();
        let forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let roots = forward.writer_roots();
        let mut query = vec![0i32; 64];
        query[..8].copy_from_slice(&roots[3]);
        query[56..].copy_from_slice(&roots[7]);
        let cell = forward.writer_query_cell(&query).unwrap();
        let gate = TerminationCellGate {
            width: 64,
            cells: vec![cell],
        };
        let inside = CpuFbsStep {
            token: 7,
            kept_cells: vec![1],
            query_q12: query.clone(),
        };
        assert_eq!(
            cell_name_or_writer(&forward, &gate, &inside).unwrap(),
            POINT_KEY_BOS
        );

        query[..8].copy_from_slice(&roots[4]);
        let outside = CpuFbsStep {
            query_q12: query,
            ..inside
        };
        assert_eq!(cell_name_or_writer(&forward, &gate, &outside).unwrap(), 7);
    }

    #[test]
    fn nonempty_termination_cell_gate_stops_the_real_reply_loop_at_bos() {
        let artifact = artifact();
        let prompt = [10, 11, 12];
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let mut query = None;
        for (index, token) in prompt.into_iter().enumerate() {
            if index + 1 == prompt.len() {
                forward.mark_reply_start();
            }
            let step = forward.step(token, CpuFbsHead::Writer).unwrap();
            if index + 1 == prompt.len() {
                query = Some(step.query_q12);
            }
        }
        let gate = TerminationCellGate {
            width: 64,
            cells: vec![forward.writer_query_cell(&query.unwrap()).unwrap()],
        };
        let reply =
            generate_termination_cell_gate_reply(&artifact, &prompt, 4, true, &gate).unwrap();
        assert_eq!(reply.stop, "bos");
        assert!(reply.tokens.is_empty());
    }

    #[test]
    fn empty_termination_cell_gate_preserves_the_no_gate_reply_digest() {
        let artifact = artifact();
        let samples = vec![MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, 77, 99],
        }];
        let gate = TerminationCellGate {
            width: 64,
            cells: Vec::new(),
        };
        let no_gate = evaluate_replies(&artifact, &samples, 4).unwrap();
        let empty_gate =
            evaluate_termination_cell_gate_replies(&artifact, &samples, 4, true, &gate).unwrap();
        assert_eq!(no_gate.reply_sha256, empty_gate.reply_sha256);
        assert_eq!(no_gate.reading, empty_gate.reading);
    }

    #[test]
    fn termination_cell_training_uses_only_eos_training_rows() {
        let artifact = artifact();
        let samples = vec![
            MemoryWeightSample {
                prompt: vec![10, 11, 12],
                reply: vec![42, CORPUS_EOS],
            },
            MemoryWeightSample {
                prompt: vec![10, 11, 12],
                reply: vec![42, 77, CORPUS_EOS],
            },
        ];
        let first = derive_termination_cell_gates(&artifact, &samples, true, 1).unwrap();
        let second = derive_termination_cell_gates(&artifact, &samples, true, 1).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.trained.width, 64);
        assert_eq!(first.trained.cells.len(), first.random.cells.len());
        assert_eq!(first.terminal_cell_counts.values().sum::<u64>(), 2);
        let bad = [MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, POINT_KEY_BOS],
        }];
        assert!(derive_termination_cell_gates(&artifact, &bad, true, 1).is_err());
    }

    #[test]
    fn premature_stop_counts_nonmatching_bos_replies_separately() {
        let reading = summarize_replies(vec![(
            GeneratedReply {
                tokens: vec![4, 5],
                stop: "bos",
            },
            vec![4, 6, POINT_KEY_BOS],
            true,
        )])
        .unwrap();
        assert_eq!(reading.bos_complete, 1);
        assert_eq!(reading.exact_reference, 0);
        assert_eq!(reading.premature_stop, 1);
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
