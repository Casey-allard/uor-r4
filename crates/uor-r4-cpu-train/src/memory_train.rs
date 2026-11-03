//! Exact full-path coordinate training of turn-memory movement weights.
//!
//! A step patches one exported Q24 memory-weight coordinate, reparses the whole
//! artifact, and runs the complete teacher-forced FBS path with opening and
//! reply rows active. The accepted direction is the one that improves served
//! top-1 first and then lowers the L1 distance from each complete query to its
//! actual next-token writer row.

use std::collections::BTreeMap;

use rayon::prelude::*;
use uor_r4_integer::stack::{
    read_memory_uorlut01, FbsMemoryEntry, FbsMemorySnapshot, FbsReplyTable, IntegerStackModel,
    StackTurnMemory, MEMORY_PAD, MEMORY_WEIGHT_EXP, STACK_FBS_MEMORY_SCHEMA,
};

use crate::fbs::CpuServedTrace;
use crate::memory::move_point_with_weight;
use crate::{
    audit_served_forward, invalid, CpuFbsForward, CpuFbsHead, CpuTrainError,
    ServedForwardAuditConfig,
};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const CPU_MEMORY_WEIGHT_TRAINER_SCHEMA: &str = "uor-r4.cpu-memory-weight-trainer/1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryWeightConfig {
    /// A candidate moves one Q24 weight coordinate by this many fixed-point units.
    pub step_q24: i32,
    /// Bidirectional absolute bound for a trained movement weight.
    pub parameter_limit_q24: i32,
    pub train_opening: bool,
    pub train_reply: bool,
}

impl Default for MemoryWeightConfig {
    fn default() -> Self {
        Self {
            step_q24: 1 << 20,
            parameter_limit_q24: 1 << 24,
            train_opening: true,
            train_reply: true,
        }
    }
}

impl MemoryWeightConfig {
    fn validate(self) -> Result<()> {
        if self.step_q24 <= 0 || self.parameter_limit_q24 <= 0 {
            return Err(invalid(
                "memory-weight step and parameter limit must be positive",
            ));
        }
        if self.step_q24 > self.parameter_limit_q24 {
            return Err(invalid(
                "memory-weight step exceeds its bidirectional parameter limit",
            ));
        }
        if !self.train_opening && !self.train_reply {
            return Err(invalid("memory-weight training selected no table"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryWeightSample {
    /// The final prompt token predicts `reply[0]`; later tokens are teacher-forced.
    pub prompt: Vec<u32>,
    pub reply: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryWeightTable {
    Opening,
    Reply,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryWeightReading {
    pub artifact_sha256: String,
    pub samples: usize,
    pub decisions: u64,
    pub top1_correct: u64,
    pub target_distance_sum: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryWeightParityReading {
    pub samples: usize,
    pub positions: u64,
    pub query_coordinates: u64,
    pub query_coordinate_mismatches: u64,
    pub token_decisions: u64,
    pub token_mismatches: u64,
    pub kept_cells: u64,
    pub kept_cell_mismatches: u64,
}

impl MemoryWeightParityReading {
    pub fn is_exact(&self) -> bool {
        self.query_coordinate_mismatches == 0
            && self.token_mismatches == 0
            && self.kept_cell_mismatches == 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryWeightStep {
    pub index: u64,
    pub updated: bool,
    pub table: Option<MemoryWeightTable>,
    pub coordinate: Option<usize>,
    pub before_weight_q24: Option<i32>,
    pub after_weight_q24: Option<i32>,
    pub before: MemoryWeightReading,
    pub after: MemoryWeightReading,
    pub parity: Option<MemoryWeightParityReading>,
}

#[derive(Clone)]
pub struct CpuMemoryWeightTrainer {
    initial_artifact: Vec<u8>,
    artifact: Vec<u8>,
    samples: Vec<MemoryWeightSample>,
    config: MemoryWeightConfig,
    steps: u64,
    trajectory: Option<CpuMemoryTrajectory>,
}

impl std::fmt::Debug for CpuMemoryWeightTrainer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CpuMemoryWeightTrainer")
            .field("artifact_bytes", &self.artifact.len())
            .field("samples", &self.samples.len())
            .field("config", &self.config)
            .field("steps", &self.steps)
            .field("cached_trajectory", &self.trajectory.is_some())
            .finish()
    }
}

impl CpuMemoryWeightTrainer {
    pub fn from_artifact(
        artifact: &[u8],
        samples: Vec<MemoryWeightSample>,
        config: MemoryWeightConfig,
    ) -> Result<Self> {
        config.validate()?;
        let memory = read_memory_uorlut01(artifact)
            .map_err(|error| invalid(format!("memory-weight artifact: {error}")))?
            .ok_or_else(|| invalid("memory-weight training requires the turn memory"))?;
        let width = IntegerStackModel::parse(artifact)
            .map_err(|error| invalid(format!("memory-weight serving model: {error}")))?
            .shape()
            .width;
        validate_memory_weights(&memory, width, config)?;
        validate_samples(artifact, &samples)?;
        let reading = evaluate(artifact, &samples)?;
        if reading.decisions == 0 {
            return Err(invalid(
                "memory-weight samples contain no next-token decision",
            ));
        }
        Ok(Self {
            initial_artifact: artifact.to_vec(),
            artifact: artifact.to_vec(),
            samples,
            config,
            steps: 0,
            trajectory: None,
        })
    }

    pub fn from_artifact_cached(
        artifact: &[u8],
        samples: Vec<MemoryWeightSample>,
        config: MemoryWeightConfig,
    ) -> Result<Self> {
        let mut trainer = Self::from_artifact(artifact, samples, config)?;
        trainer.trajectory = Some(CpuMemoryTrajectory::build(artifact, &trainer.samples)?);
        Ok(trainer)
    }

    pub fn artifact(&self) -> &[u8] {
        &self.artifact
    }

    pub fn initial_artifact(&self) -> &[u8] {
        &self.initial_artifact
    }

    pub fn config(&self) -> MemoryWeightConfig {
        self.config
    }

    pub fn step_count(&self) -> u64 {
        self.steps
    }

    pub fn reading(&self) -> Result<MemoryWeightReading> {
        if let Some(trajectory) = &self.trajectory {
            let memory = read_memory_uorlut01(&self.artifact)
                .map_err(|error| invalid(error.to_string()))?
                .ok_or_else(|| invalid("memory trajectory lost turn memory"))?;
            trajectory.evaluate(
                &self.artifact,
                &memory.opening.weight,
                memory.reply.as_ref().map(|reply| reply.weight.as_slice()),
            )
        } else {
            evaluate(&self.artifact, &self.samples)
        }
    }

    pub fn evaluate_samples(
        &self,
        samples: &[MemoryWeightSample],
        memory_enabled: bool,
    ) -> Result<MemoryWeightReading> {
        evaluate_with_memory(&self.artifact, samples, memory_enabled)
    }

    pub fn train_step(&mut self) -> Result<MemoryWeightStep> {
        let index = self.steps;
        let before = self.reading()?;
        let memory = read_memory_uorlut01(&self.artifact)
            .map_err(|error| invalid(format!("memory-weight artifact: {error}")))?
            .ok_or_else(|| invalid("memory-weight training lost its turn memory"))?;
        let opening_weights = memory.opening.weight.clone();
        let reply_weights = memory.reply.as_ref().map(|reply| reply.weight.clone());
        let mut candidates = Vec::new();
        if self.config.train_opening {
            candidates.push((MemoryWeightTable::Opening, memory.opening.weight));
        }
        if self.config.train_reply {
            if let Some(reply) = memory.reply {
                candidates.push((MemoryWeightTable::Reply, reply.weight));
            }
        }
        let mut candidates_to_try = Vec::new();
        for (table, weights) in candidates {
            for (coordinate, &original) in weights.iter().enumerate() {
                for delta in [self.config.step_q24, -self.config.step_q24] {
                    let proposed = original.saturating_add(delta).clamp(
                        -self.config.parameter_limit_q24,
                        self.config.parameter_limit_q24,
                    );
                    if proposed == original {
                        continue;
                    }
                    candidates_to_try.push(WeightCandidateInput {
                        table,
                        coordinate,
                        before_weight: original,
                        after_weight: proposed,
                        weights: weights.clone(),
                        opening_weights: opening_weights.clone(),
                        reply_weights: reply_weights.clone(),
                    });
                }
            }
        }
        let evaluated = candidates_to_try
            .par_chunks(16)
            .map(|chunk| {
                let mut best: Option<WeightCandidate> = None;
                for candidate in chunk {
                    let mut trial_weights = candidate.weights.clone();
                    trial_weights[candidate.coordinate] = candidate.after_weight;
                    let artifact =
                        patch_memory_weight_table(&self.artifact, candidate.table, &trial_weights)?;
                    let reading = match &self.trajectory {
                        Some(trajectory) => {
                            let mut opening = candidate.opening_weights.clone();
                            let mut reply = candidate.reply_weights.clone();
                            match candidate.table {
                                MemoryWeightTable::Opening => {
                                    opening[candidate.coordinate] = candidate.after_weight;
                                }
                                MemoryWeightTable::Reply => {
                                    reply.as_mut().ok_or_else(|| {
                                        invalid("memory candidate lost its reply table")
                                    })?[candidate.coordinate] = candidate.after_weight;
                                }
                            }
                            trajectory.evaluate(&artifact, &opening, reply.as_deref())?
                        }
                        None => evaluate(&artifact, &self.samples)?,
                    };
                    if !strict_improvement(&reading, &before) {
                        continue;
                    }
                    let candidate = WeightCandidate {
                        table: candidate.table,
                        coordinate: candidate.coordinate,
                        before_weight: candidate.before_weight,
                        after_weight: candidate.after_weight,
                        artifact,
                        reading,
                    };
                    if best.as_ref().is_none_or(|current| {
                        strict_improvement(&candidate.reading, &current.reading)
                    }) {
                        best = Some(candidate);
                    }
                }
                Ok::<_, CpuTrainError>(best)
            })
            .collect::<Result<Vec<_>>>()?;
        let best = evaluated
            .into_iter()
            .flatten()
            .reduce(|current, candidate| {
                if strict_improvement(&candidate.reading, &current.reading) {
                    candidate
                } else {
                    current
                }
            });
        self.steps = self
            .steps
            .checked_add(1)
            .ok_or_else(|| invalid("memory-weight step counter exhausted"))?;
        let Some(best) = best else {
            return Ok(MemoryWeightStep {
                index,
                updated: false,
                table: None,
                coordinate: None,
                before_weight_q24: None,
                after_weight_q24: None,
                before: before.clone(),
                after: before,
                parity: None,
            });
        };
        let parity = audit_memory_weight_samples(&best.artifact, &self.samples, true)?;
        if !parity.is_exact() {
            return Err(invalid(
                "memory-weight update lost complete CPU/serving parity",
            ));
        }
        self.artifact = best.artifact;
        Ok(MemoryWeightStep {
            index,
            updated: true,
            table: Some(best.table),
            coordinate: Some(best.coordinate),
            before_weight_q24: Some(best.before_weight),
            after_weight_q24: Some(best.after_weight),
            before,
            after: best.reading,
            parity: Some(parity),
        })
    }
}

struct WeightCandidate {
    table: MemoryWeightTable,
    coordinate: usize,
    before_weight: i32,
    after_weight: i32,
    artifact: Vec<u8>,
    reading: MemoryWeightReading,
}

struct WeightCandidateInput {
    table: MemoryWeightTable,
    coordinate: usize,
    before_weight: i32,
    after_weight: i32,
    weights: Vec<i32>,
    opening_weights: Vec<i32>,
    reply_weights: Option<Vec<i32>>,
}

#[derive(Clone)]
struct CachedDecision {
    target: u32,
    norm_before_memory: Vec<i64>,
    memory: Option<CachedMemoryMove>,
    query_q12: Vec<i32>,
    token: u32,
}

#[derive(Clone, Copy)]
struct CachedMemoryMove {
    opening: bool,
    stored: Option<u32>,
}

#[derive(Clone)]
struct CpuMemoryTrajectory {
    forward: CpuFbsForward,
    decisions: Vec<CachedDecision>,
    sample_ranges: Vec<std::ops::Range<usize>>,
    samples: usize,
}

impl CpuMemoryTrajectory {
    fn build(artifact: &[u8], samples: &[MemoryWeightSample]) -> Result<Self> {
        let forward = CpuFbsForward::from_artifact(artifact)
            .map_err(|error| invalid(format!("memory trajectory forward: {error}")))?;
        let sample_decisions = samples
            .par_iter()
            .map(|sample| {
                let mut forward = CpuFbsForward::from_artifact(artifact)
                    .map_err(|error| invalid(format!("memory trajectory forward: {error}")))?;
                let tokens = sample
                    .prompt
                    .iter()
                    .chain(&sample.reply)
                    .copied()
                    .collect::<Vec<_>>();
                let mut decisions = Vec::with_capacity(tokens.len().saturating_sub(1));
                for index in 0..tokens.len() - 1 {
                    if index + 1 == sample.prompt.len() {
                        forward.mark_reply_start();
                    }
                    let (
                        step,
                        CpuServedTrace {
                            norm_before_final_gain: _,
                            norm_before_memory,
                            memory,
                        },
                    ) = forward
                        .step_with_trace(tokens[index], CpuFbsHead::Writer)
                        .map_err(|error| invalid(format!("memory trajectory forward: {error}")))?;
                    decisions.push(CachedDecision {
                        target: tokens[index + 1],
                        norm_before_memory,
                        memory: memory.map(|memory| CachedMemoryMove {
                            opening: memory.opening,
                            stored: memory.stored,
                        }),
                        query_q12: step.query_q12,
                        token: step.token,
                    });
                }
                Ok(decisions)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut decisions = Vec::new();
        let mut sample_ranges = Vec::with_capacity(sample_decisions.len());
        for sample in sample_decisions {
            let start = decisions.len();
            decisions.extend(sample);
            sample_ranges.push(start..decisions.len());
        }
        Ok(Self {
            forward,
            decisions,
            sample_ranges,
            samples: samples.len(),
        })
    }

    fn evaluate(
        &self,
        artifact: &[u8],
        opening_weights: &[i32],
        reply_weights: Option<&[i32]>,
    ) -> Result<MemoryWeightReading> {
        let width = self.forward.width();
        let vocab = self.forward.vocabulary();
        let table = self.forward.token_table();
        if opening_weights.len() != width || reply_weights.is_some_and(|w| w.len() != width) {
            return Err(invalid("memory trajectory weight width differs"));
        }
        let sample_readings = self
            .sample_ranges
            .iter()
            .map(|range| {
                let mut sample_top1 = 0u64;
                let mut sample_distance = 0u64;
                let mut sample_decisions = 0u64;
                for decision in &self.decisions[range.clone()] {
                    if decision.target as usize >= vocab {
                        return Err(invalid("memory trajectory target is outside vocabulary"));
                    }
                    let (query, token) = match decision.memory {
                        Some(memory) if memory.stored.is_some() => {
                            let weights = if memory.opening {
                                opening_weights
                            } else {
                                reply_weights.ok_or_else(|| {
                                    invalid("memory trajectory lost its reply weight table")
                                })?
                            };
                            let mut norm = decision.norm_before_memory.clone();
                            move_point_with_weight(
                                memory.stored.unwrap(),
                                table,
                                weights,
                                &mut norm,
                            );
                            let query = self.forward.query_from_norm(&norm).map_err(|error| {
                                invalid(format!("memory trajectory query: {error}"))
                            })?;
                            let token = self
                                .forward
                                .writer_token_from_query(&query, false)
                                .map_err(|error| {
                                    invalid(format!("memory trajectory decision: {error}"))
                                })?;
                            (query, token)
                        }
                        _ => (decision.query_q12.clone(), decision.token),
                    };
                    let target_row = &table[decision.target as usize * width..][..width];
                    let distance =
                        query
                            .iter()
                            .zip(target_row)
                            .try_fold(0u64, |sum, (&query, &point)| {
                                sum.checked_add(u64::from(query.abs_diff(i32::from(point))))
                                    .ok_or_else(|| {
                                        invalid("memory trajectory decision distance overflows")
                                    })
                            })?;
                    sample_top1 = sample_top1
                        .checked_add(u64::from(token == decision.target))
                        .ok_or_else(|| invalid("memory trajectory top-1 count overflows"))?;
                    sample_distance = sample_distance
                        .checked_add(distance)
                        .ok_or_else(|| invalid("memory trajectory distance sum overflows"))?;
                    sample_decisions = sample_decisions
                        .checked_add(1)
                        .ok_or_else(|| invalid("memory trajectory decision count overflows"))?;
                }
                Ok((sample_top1, sample_distance, sample_decisions))
            })
            .collect::<Result<Vec<_>>>()?;
        let (top1_correct, target_distance_sum, decisions) =
            checked_sample_totals(sample_readings)?;
        Ok(MemoryWeightReading {
            artifact_sha256: uor_r4_discrete::corpus::sha256(artifact),
            samples: self.samples,
            decisions,
            top1_correct,
            target_distance_sum,
        })
    }
}

pub(crate) fn checked_sample_totals(
    readings: impl IntoIterator<Item = (u64, u64, u64)>,
) -> Result<(u64, u64, u64)> {
    let mut top1_correct = 0u64;
    let mut target_distance_sum = 0u64;
    let mut decisions = 0u64;
    for (sample_top1, sample_distance, sample_decisions) in readings {
        top1_correct = top1_correct
            .checked_add(sample_top1)
            .ok_or_else(|| invalid("memory-weight top-1 count overflows"))?;
        target_distance_sum = target_distance_sum
            .checked_add(sample_distance)
            .ok_or_else(|| invalid("memory-weight distance sum overflows"))?;
        decisions = decisions
            .checked_add(sample_decisions)
            .ok_or_else(|| invalid("memory-weight decision count overflows"))?;
    }
    Ok((top1_correct, target_distance_sum, decisions))
}

fn strict_improvement(after: &MemoryWeightReading, before: &MemoryWeightReading) -> bool {
    after.top1_correct > before.top1_correct
        || (after.top1_correct == before.top1_correct
            && after.target_distance_sum < before.target_distance_sum)
}

fn validate_memory_weights(
    memory: &uor_r4_integer::stack::MemoryImage,
    width: usize,
    config: MemoryWeightConfig,
) -> Result<()> {
    if memory.opening.weight.len() != width {
        return Err(invalid(
            "opening memory-weight width differs from the trunk",
        ));
    }
    if let Some(reply) = &memory.reply {
        if reply.weight.len() != width {
            return Err(invalid("reply memory-weight width differs from the trunk"));
        }
    } else if config.train_reply && !config.train_opening {
        return Err(invalid("reply-only training needs a reply memory table"));
    }
    Ok(())
}

pub(crate) fn validate_samples(artifact: &[u8], samples: &[MemoryWeightSample]) -> Result<()> {
    if samples.is_empty() {
        return Err(invalid("memory-weight training needs at least one sample"));
    }
    let forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("memory-weight CPU forward: {error}")))?;
    let vocab = forward.vocabulary();
    let context = forward.context_length();
    for (index, sample) in samples.iter().enumerate() {
        if sample.prompt.is_empty() || sample.reply.is_empty() {
            return Err(invalid(format!(
                "memory-weight sample {index} needs a non-empty prompt and reply"
            )));
        }
        let total = sample
            .prompt
            .len()
            .checked_add(sample.reply.len())
            .ok_or_else(|| invalid("memory-weight sample length overflows"))?;
        if total > context {
            return Err(invalid(format!(
                "memory-weight sample {index} exceeds the artifact context"
            )));
        }
        if sample
            .prompt
            .iter()
            .chain(&sample.reply)
            .any(|&token| token as usize >= vocab)
        {
            return Err(invalid(format!(
                "memory-weight sample {index} has a token outside the vocabulary"
            )));
        }
    }
    Ok(())
}

pub fn evaluate_memory_weight_samples(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    memory_enabled: bool,
) -> Result<MemoryWeightReading> {
    evaluate_with_memory(artifact, samples, memory_enabled)
}

fn evaluate(artifact: &[u8], samples: &[MemoryWeightSample]) -> Result<MemoryWeightReading> {
    evaluate_with_memory(artifact, samples, true)
}

fn evaluate_with_memory(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    memory_enabled: bool,
) -> Result<MemoryWeightReading> {
    let model = IntegerStackModel::parse(artifact)
        .map_err(|error| invalid(format!("memory-weight serving model: {error}")))?;
    let forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("memory-weight CPU forward: {error}")))?;
    let width = model.shape().width;
    let vocab = model.shape().vocab;
    let table = forward.token_table();
    if table.len() != vocab.saturating_mul(width) {
        return Err(invalid("memory-weight writer table shape differs"));
    }
    let readings = samples
        .par_iter()
        .map(|sample| evaluate_memory_sample(artifact, sample, memory_enabled, table, width))
        .collect::<Result<Vec<_>>>()?;
    let (top1_correct, target_distance_sum, decisions) = checked_sample_totals(readings)?;
    Ok(MemoryWeightReading {
        artifact_sha256: uor_r4_discrete::corpus::sha256(artifact),
        samples: samples.len(),
        decisions,
        top1_correct,
        target_distance_sum,
    })
}

fn evaluate_memory_sample(
    artifact: &[u8],
    sample: &MemoryWeightSample,
    memory_enabled: bool,
    table: &[i16],
    width: usize,
) -> Result<(u64, u64, u64)> {
    let mut top1_correct = 0u64;
    let mut target_distance_sum = 0u64;
    let mut decisions = 0u64;
    {
        let mut cpu = CpuFbsForward::from_artifact(artifact)
            .map_err(|error| invalid(format!("memory-weight CPU forward: {error}")))?;
        let tokens = sample
            .prompt
            .iter()
            .chain(&sample.reply)
            .copied()
            .collect::<Vec<_>>();
        for index in 0..tokens.len() - 1 {
            if memory_enabled && index + 1 == sample.prompt.len() {
                cpu.mark_reply_start();
            }
            let step = cpu
                .step(tokens[index], CpuFbsHead::Writer)
                .map_err(|error| invalid(format!("memory-weight forward: {error}")))?;
            let target = tokens[index + 1];
            let row = &table[target as usize * width..][..width];
            let distance =
                step.query_q12
                    .iter()
                    .zip(row)
                    .try_fold(0u64, |sum, (&query, &point)| {
                        sum.checked_add(u64::from(query.abs_diff(i32::from(point))))
                            .ok_or_else(|| invalid("memory-weight decision distance overflows"))
                    })?;
            top1_correct = top1_correct
                .checked_add(u64::from(step.token == target))
                .ok_or_else(|| invalid("memory-weight top-1 count overflows"))?;
            target_distance_sum = target_distance_sum
                .checked_add(distance)
                .ok_or_else(|| invalid("memory-weight distance sum overflows"))?;
            decisions = decisions
                .checked_add(1)
                .ok_or_else(|| invalid("memory-weight decision count overflows"))?;
        }
    }
    Ok((top1_correct, target_distance_sum, decisions))
}

/// Build opening and reply memory tables from teacher-forced prompt/reply
/// samples. Both movement weight rows start at zero so training changes only
/// what the exact served path proves useful.
pub fn derive_turn_memory(
    samples: &[MemoryWeightSample],
    width: usize,
    k: usize,
    key_window: usize,
    reply_turn: usize,
) -> Result<FbsMemorySnapshot> {
    if samples.is_empty() || width == 0 || !(1..=16).contains(&k) {
        return Err(invalid("turn-memory derivation inputs differ"));
    }
    if key_window == 0 || !(1..=16).contains(&reply_turn) {
        return Err(invalid("turn-memory derivation dimensions differ"));
    }
    let mut opening = BTreeMap::<Vec<u32>, BTreeMap<u32, usize>>::new();
    let mut reply = BTreeMap::<Vec<u32>, BTreeMap<u32, usize>>::new();
    for sample in samples {
        if sample.prompt.is_empty() || sample.reply.is_empty() {
            return Err(invalid(
                "turn-memory derivation needs non-empty prompt and reply",
            ));
        }
        let opening_key = sample.prompt.iter().copied().take(k).collect::<Vec<_>>();
        *opening
            .entry(opening_key)
            .or_default()
            .entry(sample.reply[0])
            .or_default() += 1;

        let turn = sample
            .prompt
            .iter()
            .copied()
            .take(reply_turn)
            .collect::<Vec<_>>();
        let tokens = sample
            .prompt
            .iter()
            .chain(&sample.reply)
            .copied()
            .collect::<Vec<_>>();
        let reply_start = sample.prompt.len();
        for offset in 0..sample.reply.len().saturating_sub(1) {
            let row = reply_start + offset;
            let block_start = row / key_window * key_window;
            let visible = block_start.max(reply_start);
            let length = row + 1 - visible;
            let mut key = turn.clone();
            key.extend([
                MEMORY_PAD,
                u32::try_from(length).map_err(|_| invalid("reply key length exceeds u32"))?,
                MEMORY_PAD,
                tokens[row],
            ]);
            *reply
                .entry(key)
                .or_default()
                .entry(sample.reply[offset + 1])
                .or_default() += 1;
        }
    }
    Ok(FbsMemorySnapshot {
        k,
        key_window,
        opening: commonest_entries(opening),
        opening_weight: vec![0.0; width],
        reply: Some(FbsReplyTable {
            turn: reply_turn,
            entries: commonest_entries(reply),
            weight: vec![0.0; width],
        }),
    })
}

fn commonest_entries(counts: BTreeMap<Vec<u32>, BTreeMap<u32, usize>>) -> Vec<FbsMemoryEntry> {
    counts
        .into_iter()
        .map(|(key, targets)| {
            let value = targets
                .into_iter()
                .min_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)))
                .map(|(target, _)| target)
                .expect("a derived memory key has a target");
            FbsMemoryEntry { key, value }
        })
        .collect()
}

/// Add deterministic turn-memory tables to a parsed UORLUT artifact whose
/// existing bytes stay unchanged. The result is parsed and read back before it
/// is returned.
pub fn add_turn_memory_uorlut01(artifact: &[u8], memory: &FbsMemorySnapshot) -> Result<Vec<u8>> {
    let model = IntegerStackModel::parse(artifact)
        .map_err(|error| invalid(format!("turn-memory attach artifact: {error}")))?;
    let shape = model.shape();
    if !shape.projection_fbs() {
        return Err(invalid(
            "turn-memory attach requires a SpiralCore/FBS artifact",
        ));
    }
    if read_memory_uorlut01(artifact)
        .map_err(|error| invalid(format!("turn-memory attach artifact: {error}")))?
        .is_some()
    {
        return Err(invalid("artifact already carries turn memory"));
    }
    if memory.k == 0
        || memory.k > shape.vocab
        || memory.key_window == 0
        || memory.key_window > shape.context
        || memory.opening_weight.len() != shape.width
    {
        return Err(invalid(
            "turn-memory attach snapshot differs from the artifact",
        ));
    }
    let opening = memory_table(
        &memory.opening,
        memory.k,
        shape.width,
        shape.vocab,
        &memory.opening_weight,
    )?;
    let reply = memory
        .reply
        .as_ref()
        .map(|table| memory_table(&table.entries, 20, shape.width, shape.vocab, &table.weight))
        .transpose()?;
    if let Some(table) = &memory.reply {
        if table.turn == 0 || table.turn > 16 || table.weight.len() != shape.width {
            return Err(invalid("reply turn-memory attach snapshot differs"));
        }
    }
    if artifact.len() < 16 || &artifact[..8] != b"UORLUT01" {
        return Err(invalid("turn-memory attach artifact magic differs"));
    }
    let header_len = u64::from_le_bytes(
        artifact[8..16]
            .try_into()
            .map_err(|_| invalid("turn-memory attach header length differs"))?,
    );
    let header_end = 16usize
        .checked_add(
            usize::try_from(header_len)
                .map_err(|_| invalid("turn-memory attach header is too large"))?,
        )
        .filter(|&end| end <= artifact.len())
        .ok_or_else(|| invalid("turn-memory attach header is outside the file"))?;
    let data_start = header_end.div_ceil(64) * 64;
    if data_start > artifact.len() {
        return Err(invalid("turn-memory attach data start is outside the file"));
    }
    let mut header: serde_json::Value = serde_json::from_slice(&artifact[16..header_end])
        .map_err(|error| invalid(format!("turn-memory attach header: {error}")))?;
    let mut tables = header["tables"]
        .as_array()
        .cloned()
        .ok_or_else(|| invalid("turn-memory attach artifact has no table list"))?;
    let mut data = artifact[data_start..].to_vec();
    let append = |name: &str,
                  kind: &str,
                  bytes: Vec<u8>,
                  len: usize,
                  tables: &mut Vec<serde_json::Value>,
                  data: &mut Vec<u8>| {
        let aligned = data.len().div_ceil(64) * 64;
        data.resize(aligned, 0);
        let offset = data.len() as u64;
        data.extend_from_slice(&bytes);
        tables.push(serde_json::json!({
            "name": name,
            "kind": kind,
            "len": len,
            "span": {"offset": offset, "bytes": bytes.len() as u64},
        }));
    };
    append(
        "memory.opening.keys",
        "u32",
        u32_bytes(&opening.keys),
        opening.keys.len(),
        &mut tables,
        &mut data,
    );
    append(
        "memory.opening.values",
        "u32",
        u32_bytes(&opening.values),
        opening.values.len(),
        &mut tables,
        &mut data,
    );
    append(
        "memory.opening.weight",
        "i32",
        i32_bytes(&opening.weight),
        opening.weight.len(),
        &mut tables,
        &mut data,
    );
    if let (Some(_), Some(reply)) = (&memory.reply, reply.as_ref()) {
        append(
            "memory.reply.keys",
            "u32",
            u32_bytes(&reply.keys),
            reply.keys.len(),
            &mut tables,
            &mut data,
        );
        append(
            "memory.reply.values",
            "u32",
            u32_bytes(&reply.values),
            reply.values.len(),
            &mut tables,
            &mut data,
        );
        append(
            "memory.reply.weight",
            "i32",
            i32_bytes(&reply.weight),
            reply.weight.len(),
            &mut tables,
            &mut data,
        );
    }
    header["tables"] = serde_json::Value::Array(tables);
    header["schema"] = serde_json::Value::String(STACK_FBS_MEMORY_SCHEMA.to_owned());
    header["memory"] = serde_json::to_value(StackTurnMemory {
        k: memory.k,
        key_window: memory.key_window,
        opening: memory.opening.len(),
        reply: memory.reply.as_ref().map(|table| table.entries.len()),
        reply_turn: memory.reply.as_ref().map(|table| table.turn),
        reply_key_len: 20,
        weight_exp: MEMORY_WEIGHT_EXP,
    })
    .map_err(|error| invalid(format!("turn-memory header: {error}")))?;
    let header_bytes = serde_json::to_vec(&header)
        .map_err(|error| invalid(format!("turn-memory header: {error}")))?;
    let mut out = Vec::with_capacity(16 + header_bytes.len() + 64 + data.len());
    out.extend_from_slice(b"UORLUT01");
    out.extend_from_slice(&(header_bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(&header_bytes);
    out.resize(out.len().div_ceil(64) * 64, 0);
    out.extend_from_slice(&data);
    let image = read_memory_uorlut01(&out)
        .map_err(|error| invalid(format!("attached turn memory does not read: {error}")))?
        .ok_or_else(|| invalid("attached turn memory is absent"))?;
    if image.record.k != memory.k || image.record.key_window != memory.key_window {
        return Err(invalid("attached turn-memory record differs"));
    }
    let opening_keys = memory
        .opening
        .iter()
        .map(|entry| entry.key.clone())
        .collect::<Vec<_>>();
    if image.opening.keys != opening_keys
        || image.opening.values != opening.values
        || image.opening.weight != opening.weight
    {
        return Err(invalid("attached opening memory differs"));
    }
    match (&image.reply, &reply) {
        (Some(actual), Some(expected))
            if memory.reply.as_ref().is_some_and(|table| {
                actual.keys
                    == table
                        .entries
                        .iter()
                        .map(|entry| entry.key.clone())
                        .collect::<Vec<_>>()
            }) && actual.values == expected.values
                && actual.weight == expected.weight => {}
        (None, None) => {}
        _ => return Err(invalid("attached reply memory differs")),
    }
    Ok(out)
}

struct EncodedMemoryTable {
    keys: Vec<u32>,
    values: Vec<u32>,
    weight: Vec<i32>,
}

fn memory_table(
    entries: &[FbsMemoryEntry],
    row: usize,
    width: usize,
    vocab: usize,
    weights: &[f32],
) -> Result<EncodedMemoryTable> {
    if row == 0 || width == 0 || weights.len() != width {
        return Err(invalid("turn-memory attach table dimensions differ"));
    }
    let mut keys = Vec::with_capacity(entries.len().saturating_mul(row));
    let mut values = Vec::with_capacity(entries.len());
    let mut previous: Option<&[u32]> = None;
    for entry in entries {
        if entry.key.is_empty()
            || entry.key.len() > row
            || entry.value as usize >= vocab
            || entry
                .key
                .iter()
                .any(|&token| token as usize >= vocab && token != MEMORY_PAD)
            || previous.is_some_and(|prior| prior >= entry.key.as_slice())
        {
            return Err(invalid("turn-memory attach entry differs"));
        }
        previous = Some(&entry.key);
        keys.extend(
            entry
                .key
                .iter()
                .copied()
                .chain(std::iter::repeat(MEMORY_PAD))
                .take(row),
        );
        values.push(entry.value);
    }
    let weight = weights
        .iter()
        .map(|&value| {
            let scaled = (f64::from(value) * 2f64.powi(-MEMORY_WEIGHT_EXP)).round();
            if !scaled.is_finite() || scaled.abs() > f64::from(i32::MAX) {
                Err(invalid("turn-memory attach weight overflows"))
            } else {
                Ok(scaled as i32)
            }
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(EncodedMemoryTable {
        keys,
        values,
        weight,
    })
}

fn u32_bytes(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn i32_bytes(values: &[i32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

pub fn audit_memory_weight_samples(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    memory_enabled: bool,
) -> Result<MemoryWeightParityReading> {
    let parity = samples
        .par_iter()
        .map(|sample| {
            let tokens = sample
                .prompt
                .iter()
                .chain(&sample.reply)
                .copied()
                .collect::<Vec<_>>();
            audit_served_forward(
                artifact,
                &ServedForwardAuditConfig {
                    tokens,
                    token_source: None,
                    expected_artifact_sha256: None,
                    reply_start: memory_enabled.then_some(sample.prompt.len() - 1),
                    measure_timing: false,
                },
            )
            .map(|audit| audit.parity)
            .map_err(|error| invalid(format!("memory-weight served audit: {error}")))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut total = MemoryWeightParityReading::default();
    for parity in parity {
        if parity.instrument_fault {
            return Err(invalid(
                "memory-weight served audit reports a CPU/serving instrument fault",
            ));
        }
        let parity = MemoryWeightParityReading {
            samples: 1,
            positions: u64::try_from(parity.positions_checked)
                .map_err(|_| invalid("memory-weight parity position count exceeds u64"))?,
            query_coordinates: parity.query_coordinates_compared,
            query_coordinate_mismatches: parity.query_coordinate_mismatches,
            token_decisions: parity.token_decisions_compared,
            token_mismatches: parity.token_mismatches,
            kept_cells: parity.kept_cells_compared,
            kept_cell_mismatches: parity.kept_cell_mismatches,
        };
        checked_add_memory_weight_parity(&mut total, &parity)?;
    }
    Ok(total)
}

fn checked_add_memory_weight_parity(
    total: &mut MemoryWeightParityReading,
    parity: &MemoryWeightParityReading,
) -> Result<()> {
    fn add(current: &mut u64, increment: u64, field: &'static str) -> Result<()> {
        *current = current
            .checked_add(increment)
            .ok_or_else(|| invalid(format!("memory-weight parity {field} count overflows")))?;
        Ok(())
    }

    total.samples = total
        .samples
        .checked_add(parity.samples)
        .ok_or_else(|| invalid("memory-weight parity samples count overflows"))?;
    add(&mut total.positions, parity.positions, "positions")?;
    add(
        &mut total.query_coordinates,
        parity.query_coordinates,
        "query coordinates",
    )?;
    add(
        &mut total.query_coordinate_mismatches,
        parity.query_coordinate_mismatches,
        "query coordinate mismatch",
    )?;
    add(
        &mut total.token_decisions,
        parity.token_decisions,
        "token decisions",
    )?;
    add(
        &mut total.token_mismatches,
        parity.token_mismatches,
        "token mismatch",
    )?;
    add(&mut total.kept_cells, parity.kept_cells, "kept cells")?;
    add(
        &mut total.kept_cell_mismatches,
        parity.kept_cell_mismatches,
        "kept cell mismatch",
    )?;
    Ok(())
}

pub fn replace_memory_weight_table(
    artifact: &[u8],
    table: MemoryWeightTable,
    weights: &[i32],
) -> Result<Vec<u8>> {
    patch_memory_weight_table(artifact, table, weights)
}

fn patch_memory_weight_table(
    artifact: &[u8],
    table: MemoryWeightTable,
    weights: &[i32],
) -> Result<Vec<u8>> {
    let name = match table {
        MemoryWeightTable::Opening => "memory.opening.weight",
        MemoryWeightTable::Reply => "memory.reply.weight",
    };
    patch_i32_table(artifact, name, weights)
}

fn patch_i32_table(artifact: &[u8], name: &str, values: &[i32]) -> Result<Vec<u8>> {
    if artifact.len() < 16 || &artifact[..8] != b"UORLUT01" {
        return Err(invalid("memory-weight artifact magic differs"));
    }
    let header_len = u64::from_le_bytes(
        artifact[8..16]
            .try_into()
            .map_err(|_| invalid("memory-weight artifact header length differs"))?,
    );
    let header_end = 16usize
        .checked_add(
            usize::try_from(header_len)
                .map_err(|_| invalid("memory-weight artifact header is too large"))?,
        )
        .filter(|&end| end <= artifact.len())
        .ok_or_else(|| invalid("memory-weight artifact header is outside the file"))?;
    let data_start = header_end.div_ceil(64) * 64;
    if data_start > artifact.len() {
        return Err(invalid(
            "memory-weight artifact data start is outside the file",
        ));
    }
    let header: serde_json::Value =
        serde_json::from_slice(&artifact[16..header_end]).map_err(|error| {
            invalid(format!(
                "memory-weight artifact header is invalid JSON: {error}"
            ))
        })?;
    let entry = header["tables"]
        .as_array()
        .and_then(|tables| {
            tables
                .iter()
                .find(|entry| entry["name"].as_str() == Some(name))
        })
        .ok_or_else(|| invalid(format!("artifact has no {name} table")))?;
    if entry["kind"].as_str() != Some("i32") || entry["len"].as_u64() != Some(values.len() as u64) {
        return Err(invalid(format!("artifact {name} shape differs")));
    }
    let offset = usize::try_from(
        entry["span"]["offset"]
            .as_u64()
            .ok_or_else(|| invalid(format!("{name} offset is missing")))?,
    )
    .map_err(|_| invalid(format!("{name} offset exceeds usize")))?;
    let bytes = usize::try_from(
        entry["span"]["bytes"]
            .as_u64()
            .ok_or_else(|| invalid(format!("{name} byte length is missing")))?,
    )
    .map_err(|_| invalid(format!("{name} byte length exceeds usize")))?;
    if bytes != values.len() * 4 {
        return Err(invalid(format!("{name} byte length differs")));
    }
    let start = data_start
        .checked_add(offset)
        .ok_or_else(|| invalid(format!("{name} offset overflows")))?;
    let end = start
        .checked_add(bytes)
        .filter(|&end| end <= artifact.len())
        .ok_or_else(|| invalid(format!("{name} is outside the file")))?;
    let mut out = artifact.to_vec();
    for (slot, &value) in out[start..end].chunks_exact_mut(4).zip(values) {
        slot.copy_from_slice(&value.to_le_bytes());
    }
    IntegerStackModel::parse(&out)
        .map_err(|error| invalid(format!("patched artifact does not parse: {error}")))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};
    use uor_r4_integer::stack::{
        add_point_key_uorlut01, export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot,
        FbsReplyTable, PointKeyImage, MEMORY_PAD,
    };

    const WIDTH: usize = 64;

    fn fixture(opening_weight: f32, reply_weight: f32) -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            931,
            FbsSynth {
                vocab: crate::VOCABULARY,
                d: WIDTH,
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
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 3,
            key_window: 16,
            opening: vec![FbsMemoryEntry {
                key: vec![10, 11, 12],
                value: 42,
            }],
            opening_weight: vec![opening_weight; WIDTH],
            reply: Some(FbsReplyTable {
                turn: 3,
                entries: vec![
                    FbsMemoryEntry {
                        key: vec![10, 11, 12, MEMORY_PAD, 1, MEMORY_PAD, 42],
                        value: 77,
                    },
                    FbsMemoryEntry {
                        key: vec![10, 11, 12, MEMORY_PAD, 2, MEMORY_PAD, 77],
                        value: 99,
                    },
                ],
                weight: vec![reply_weight; WIDTH],
            }),
        });
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn bare_fixture() -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            931,
            FbsSynth {
                vocab: crate::VOCABULARY,
                d: WIDTH,
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

    fn samples() -> Vec<MemoryWeightSample> {
        vec![MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, 77, 99],
        }]
    }

    fn point_fixture() -> Vec<u8> {
        let artifact = fixture(0.0, 1.0);
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let mut query = None;
        for (index, token) in [30, 31, 32].into_iter().enumerate() {
            if index + 1 == 3 {
                forward.mark_reply_start();
            }
            let step = forward.step(token, CpuFbsHead::Writer).unwrap();
            if index + 1 == 3 {
                query = Some(step.query_q12);
            }
        }
        add_point_key_uorlut01(
            &artifact,
            &PointKeyImage {
                width: WIDTH,
                points: query
                    .expect("the prompt has a final query")
                    .into_iter()
                    .map(|value| i16::try_from(value).expect("the query is Q12-clamped"))
                    .collect(),
                targets: vec![40],
            },
        )
        .unwrap()
    }

    fn hit_miss_point_samples() -> Vec<MemoryWeightSample> {
        let mut edge_samples = samples();
        edge_samples.push(MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, 77, 49],
        });
        edge_samples.push(MemoryWeightSample {
            prompt: vec![30, 31, 32],
            reply: vec![40],
        });
        edge_samples
    }

    fn assert_audit(artifact: &[u8]) {
        let parity = audit_memory_weight_samples(artifact, &samples(), true).unwrap();
        assert!(parity.is_exact());
        assert!(parity.positions > 0);
        assert!(parity.query_coordinates > 0);
        assert!(parity.token_decisions > 0);
    }

    #[test]
    fn opening_weights_train_through_the_complete_served_path() {
        let artifact = fixture(0.0, 1.0);
        let config = MemoryWeightConfig {
            train_opening: true,
            train_reply: false,
            ..MemoryWeightConfig::default()
        };
        let mut trainer =
            CpuMemoryWeightTrainer::from_artifact(&artifact, samples(), config).unwrap();
        let before = trainer.reading().unwrap();
        let step = trainer.train_step().unwrap();
        assert!(step.updated);
        assert_eq!(step.table, Some(MemoryWeightTable::Opening));
        assert_eq!(step.after.top1_correct, before.top1_correct);
        assert!(step.after.target_distance_sum < before.target_distance_sum);
        assert!(step.parity.as_ref().unwrap().is_exact());
        assert_ne!(trainer.artifact(), artifact);
        assert_audit(trainer.artifact());
        eprintln!(
            "{}",
            serde_json::json!({
                "arm": "opening",
                "initial_sha256": uor_r4_discrete::corpus::sha256(&artifact),
                "final_sha256": uor_r4_discrete::corpus::sha256(trainer.artifact()),
                "before": before,
                "after": step.after,
                "table": step.table,
                "coordinate": step.coordinate,
                "before_weight_q24": step.before_weight_q24,
                "after_weight_q24": step.after_weight_q24,
                "parity": step.parity,
            })
        );
    }

    #[test]
    fn reply_weights_train_through_the_complete_served_path() {
        let artifact = fixture(1.0, 0.0);
        let config = MemoryWeightConfig {
            train_opening: false,
            train_reply: true,
            ..MemoryWeightConfig::default()
        };
        let mut trainer =
            CpuMemoryWeightTrainer::from_artifact(&artifact, samples(), config).unwrap();
        let before = trainer.reading().unwrap();
        let step = trainer.train_step().unwrap();
        assert!(step.updated);
        assert_eq!(step.table, Some(MemoryWeightTable::Reply));
        assert_eq!(step.after.top1_correct, before.top1_correct);
        assert!(step.after.target_distance_sum < before.target_distance_sum);
        assert!(step.parity.as_ref().unwrap().is_exact());
        assert_audit(trainer.artifact());
        eprintln!(
            "{}",
            serde_json::json!({
                "arm": "reply",
                "initial_sha256": uor_r4_discrete::corpus::sha256(&artifact),
                "final_sha256": uor_r4_discrete::corpus::sha256(trainer.artifact()),
                "before": before,
                "after": step.after,
                "table": step.table,
                "coordinate": step.coordinate,
                "before_weight_q24": step.before_weight_q24,
                "after_weight_q24": step.after_weight_q24,
                "parity": step.parity,
            })
        );
    }

    #[test]
    fn independent_constructions_are_byte_identical() {
        let artifact = fixture(0.0, 0.0);
        let config = MemoryWeightConfig::default();
        let mut first =
            CpuMemoryWeightTrainer::from_artifact(&artifact, samples(), config).unwrap();
        let mut second =
            CpuMemoryWeightTrainer::from_artifact(&artifact, samples(), config).unwrap();
        let one = first.train_step().unwrap();
        let two = second.train_step().unwrap();
        assert_eq!(one, two);
        assert_eq!(first.artifact(), second.artifact());
    }

    #[test]
    fn cached_trajectory_reproduces_full_forward_training_exactly() {
        let artifact = fixture(0.0, 0.0);
        let config = MemoryWeightConfig::default();
        let mut full = CpuMemoryWeightTrainer::from_artifact(&artifact, samples(), config).unwrap();
        let mut cached =
            CpuMemoryWeightTrainer::from_artifact_cached(&artifact, samples(), config).unwrap();
        for _ in 0..3 {
            let expected = full.train_step().unwrap();
            let actual = cached.train_step().unwrap();
            assert_eq!(actual, expected);
            assert_eq!(cached.reading().unwrap(), full.reading().unwrap());
        }
        assert_eq!(cached.artifact(), full.artifact());
    }

    #[test]
    fn cached_trajectory_matches_single_table_and_saturation_edges() {
        for (table, config, expect_accepted) in [
            (
                MemoryWeightTable::Opening,
                MemoryWeightConfig {
                    train_opening: true,
                    train_reply: false,
                    ..MemoryWeightConfig::default()
                },
                true,
            ),
            (
                MemoryWeightTable::Reply,
                MemoryWeightConfig {
                    train_opening: false,
                    train_reply: true,
                    ..MemoryWeightConfig::default()
                },
                false,
            ),
            (
                MemoryWeightTable::Opening,
                MemoryWeightConfig {
                    step_q24: 1 << 20,
                    parameter_limit_q24: 1 << 20,
                    train_opening: true,
                    train_reply: false,
                },
                true,
            ),
        ] {
            let artifact = fixture(0.0, 1.0);
            let mut full =
                CpuMemoryWeightTrainer::from_artifact(&artifact, samples(), config).unwrap();
            let mut cached =
                CpuMemoryWeightTrainer::from_artifact_cached(&artifact, samples(), config).unwrap();
            let mut accepted = 0;
            for _ in 0..2 {
                let expected = full.train_step().unwrap();
                let actual = cached.train_step().unwrap();
                assert_eq!(actual, expected);
                if let Some(selected) = actual.table {
                    assert_eq!(selected, table);
                }
                if actual.updated {
                    accepted += 1;
                    if config.parameter_limit_q24 == 1 << 20 {
                        assert_eq!(
                            actual
                                .after_weight_q24
                                .expect("an accepted step has its new weight")
                                .abs(),
                            1 << 20
                        );
                    }
                }
            }
            assert_eq!(accepted > 0, expect_accepted);
            assert_eq!(cached.artifact(), full.artifact());
        }
    }

    #[test]
    fn cached_trajectory_matches_hit_miss_and_point_fallback() {
        let artifact = point_fixture();
        let mut point = CpuFbsForward::from_artifact(&artifact).unwrap();
        let mut token = None;
        for (index, input) in [30, 31, 32].into_iter().enumerate() {
            if index + 1 == 3 {
                point.mark_reply_start();
            }
            token = Some(point.step(input, CpuFbsHead::Writer).unwrap().token);
        }
        assert_eq!(token, Some(40));

        let config = MemoryWeightConfig::default();
        let mut full =
            CpuMemoryWeightTrainer::from_artifact(&artifact, hit_miss_point_samples(), config)
                .unwrap();
        let mut cached = CpuMemoryWeightTrainer::from_artifact_cached(
            &artifact,
            hit_miss_point_samples(),
            config,
        )
        .unwrap();
        for _ in 0..3 {
            let expected = full.train_step().unwrap();
            let actual = cached.train_step().unwrap();
            assert_eq!(actual, expected);
        }
        assert_eq!(cached.artifact(), full.artifact());
    }

    #[test]
    fn cached_parallel_constructions_are_byte_identical() {
        let artifact = point_fixture();
        let config = MemoryWeightConfig::default();
        let mut first = CpuMemoryWeightTrainer::from_artifact_cached(
            &artifact,
            hit_miss_point_samples(),
            config,
        )
        .unwrap();
        let mut second = CpuMemoryWeightTrainer::from_artifact_cached(
            &artifact,
            hit_miss_point_samples(),
            config,
        )
        .unwrap();
        for _ in 0..3 {
            assert_eq!(first.train_step().unwrap(), second.train_step().unwrap());
        }
        assert_eq!(first.artifact(), second.artifact());
    }

    #[test]
    fn cached_sample_aggregation_refuses_every_overflow_class() {
        for readings in [
            [(u64::MAX, 0, 0), (1, 0, 0)],
            [(0, u64::MAX, 0), (0, 1, 0)],
            [(0, 0, u64::MAX), (0, 0, 1)],
        ] {
            let error = checked_sample_totals(readings).unwrap_err();
            assert!(error.to_string().contains("overflows"));
        }
    }

    #[test]
    fn parity_aggregation_refuses_every_overflow_class() {
        for field in 0..8 {
            let mut total = MemoryWeightParityReading::default();
            let mut parity = MemoryWeightParityReading::default();
            match field {
                0 => {
                    total.samples = usize::MAX;
                    parity.samples = 1;
                }
                1 => {
                    total.positions = u64::MAX;
                    parity.positions = 1;
                }
                2 => {
                    total.query_coordinates = u64::MAX;
                    parity.query_coordinates = 1;
                }
                3 => {
                    total.query_coordinate_mismatches = u64::MAX;
                    parity.query_coordinate_mismatches = 1;
                }
                4 => {
                    total.token_decisions = u64::MAX;
                    parity.token_decisions = 1;
                }
                5 => {
                    total.token_mismatches = u64::MAX;
                    parity.token_mismatches = 1;
                }
                6 => {
                    total.kept_cells = u64::MAX;
                    parity.kept_cells = 1;
                }
                7 => {
                    total.kept_cell_mismatches = u64::MAX;
                    parity.kept_cell_mismatches = 1;
                }
                _ => unreachable!(),
            }
            let error = checked_add_memory_weight_parity(&mut total, &parity).unwrap_err();
            assert!(error.to_string().contains("overflows"));
        }
    }

    #[test]
    fn a_weight_patch_changes_only_its_named_table() {
        let artifact = fixture(0.0, 0.0);
        let before = read_memory_uorlut01(&artifact).unwrap().unwrap();
        let mut opening = before.opening.weight.clone();
        let coordinate = 7;
        opening[coordinate] = 1 << 20;
        let patched =
            patch_memory_weight_table(&artifact, MemoryWeightTable::Opening, &opening).unwrap();
        let after = read_memory_uorlut01(&patched).unwrap().unwrap();
        assert_eq!(after.opening.weight, opening);
        assert_eq!(after.opening.keys, before.opening.keys);
        assert_eq!(after.opening.values, before.opening.values);
        assert_eq!(after.reply, before.reply);
        assert_eq!(after.record, before.record);
        assert_eq!(
            artifact
                .iter()
                .zip(&patched)
                .filter(|(left, right)| left != right)
                .count(),
            1,
            "the selected Q24 byte is the only byte that changes"
        );
    }

    #[test]
    fn an_unseen_memory_key_is_a_byte_identical_noop() {
        let artifact = fixture(0.0, 0.0);
        for cached in [false, true] {
            let unseen = vec![MemoryWeightSample {
                prompt: vec![20, 21, 22],
                reply: vec![50, 51, 52],
            }];
            let mut trainer = if cached {
                CpuMemoryWeightTrainer::from_artifact_cached(
                    &artifact,
                    unseen,
                    MemoryWeightConfig::default(),
                )
            } else {
                CpuMemoryWeightTrainer::from_artifact(
                    &artifact,
                    unseen,
                    MemoryWeightConfig::default(),
                )
            }
            .unwrap();
            let step = trainer.train_step().unwrap();
            assert!(!step.updated);
            assert_eq!(trainer.artifact(), artifact);
            assert!(step.parity.is_none());
        }
    }

    #[test]
    fn training_refuses_missing_memory_and_invalid_samples() {
        let mut snapshot = synthetic_fbs_trunk_with(
            932,
            FbsSynth {
                vocab: crate::VOCABULARY,
                d: WIDTH,
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
        let bare = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let error =
            CpuMemoryWeightTrainer::from_artifact(&bare, samples(), MemoryWeightConfig::default())
                .unwrap_err();
        assert!(error.to_string().contains("requires the turn memory"));

        let artifact = fixture(1.0, 1.0);
        let invalid = vec![MemoryWeightSample {
            prompt: Vec::new(),
            reply: vec![42],
        }];
        let error = CpuMemoryWeightTrainer::from_artifact(
            &artifact,
            invalid,
            MemoryWeightConfig::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("non-empty prompt and reply"));
    }

    #[test]
    fn derived_memory_can_be_attached_and_trained() {
        let memory = derive_turn_memory(&samples(), WIDTH, 3, 16, 3).unwrap();
        assert_eq!(memory.opening.len(), 1);
        assert_eq!(memory.opening[0].key, [10, 11, 12]);
        assert_eq!(memory.opening[0].value, 42);
        assert_eq!(memory.reply.as_ref().unwrap().entries.len(), 2);
        let artifact = add_turn_memory_uorlut01(&bare_fixture(), &memory).unwrap();
        let read = read_memory_uorlut01(&artifact).unwrap().unwrap();
        assert_eq!(read.opening.keys, vec![vec![10, 11, 12]]);
        assert_eq!(read.opening.values, vec![42]);
        assert_eq!(read.opening.weight, vec![0; WIDTH]);
        assert_eq!(read.reply.as_ref().unwrap().weight, vec![0; WIDTH]);
        let mut trainer = CpuMemoryWeightTrainer::from_artifact(
            &artifact,
            samples(),
            MemoryWeightConfig::default(),
        )
        .unwrap();
        let step = trainer.train_step().unwrap();
        assert!(step.updated);
        assert!(step.parity.as_ref().unwrap().is_exact());
    }
}
