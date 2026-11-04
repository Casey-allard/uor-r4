//! Exact writer training on the model's own generated reply prefixes.
//!
//! A scan feeds generated tokens back through the complete CPU FBS and memory
//! path. The captured query and the reference token at the same reply offset
//! feed the existing K=1 rank-margin update. No serving path is opened here.

use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;
use uor_r4_integer::stack::{reply_stop, ReplyEnd};

use crate::reply_eval::normalized_reference;
use crate::train::{propose_writer_update, reading_from_forward, RankMarginConfig};
use crate::{
    audit_memory_weight_samples, invalid, CpuFbsForward, CpuFbsHead, CpuTrainError,
    MemoryWeightParityReading, MemoryWeightSample,
};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const SELF_PREFIX_SCHEMA: &str = "uor-r4.cpu-self-prefix/1";
const TARGET_CELLS: u32 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelfPrefixMode {
    Reference,
    StaticGenerated,
    DynamicGenerated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelfPrefixTargets {
    TrueLabels,
    SameCellShuffled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelfPrefixConfig {
    pub rank_margin: RankMarginConfig,
    pub reply_cap: usize,
    pub memory_enabled: bool,
    pub mode: SelfPrefixMode,
    pub targets: SelfPrefixTargets,
    pub shuffle_seed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelfPrefixDecision {
    pub sample: usize,
    pub reply_offset: usize,
    pub query_q12: Vec<i32>,
    pub generated_token: u32,
    pub reference_target: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelfPrefixReading {
    pub samples: u64,
    pub fixed_horizon_decisions: u64,
    pub observed_decisions: u64,
    pub missing_decisions: u64,
    pub generated_top1: u64,
    pub writer_top1: u64,
    pub target_admitted: u64,
    pub target_rank_sum: u64,
    pub target_distance_sum: u64,
    pub matched_prefix_depth_sum: u64,
    pub opening_correct: u64,
    pub exact_reference: u64,
    pub bos_stopped: u64,
    pub cap_stopped: u64,
    pub context_stopped: u64,
    pub generated_tokens: u64,
    pub distinct_non_empty: u64,
    pub repeated_4gram_replies: u64,
    pub first_divergence: BTreeMap<usize, u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelfPrefixEligibility {
    pub eligible_decisions: u64,
    pub prompts: u64,
    pub target_cells: u64,
    pub same_cell_shuffle_pairs: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelfPrefixPromptReading {
    pub sample: usize,
    pub matched_prefix_depth: usize,
    pub generated_top1: u64,
    pub observed_decisions: u64,
    pub opening_correct: bool,
    pub exact_reference: bool,
    pub stop: String,
    pub generated_tokens: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelfPrefixTrainStep {
    pub index: u64,
    pub updated: bool,
    pub sample: Option<usize>,
    pub reply_offset: Option<usize>,
    pub target: Option<u32>,
    pub coordinate: Option<usize>,
    pub before: SelfPrefixReading,
    pub after: SelfPrefixReading,
    pub parity: Option<MemoryWeightParityReading>,
}

#[derive(Clone)]
struct PrefixSampleScan {
    decisions: Vec<SelfPrefixDecision>,
    tokens: Vec<u32>,
    stop: &'static str,
}

#[derive(Clone)]
struct PrefixScan {
    samples: Vec<PrefixSampleScan>,
}

#[derive(Clone)]
pub struct CpuSelfPrefixTrainer {
    artifact: Vec<u8>,
    samples: Vec<MemoryWeightSample>,
    config: SelfPrefixConfig,
    frozen_scan: PrefixScan,
    attempts: u64,
    accepted: u64,
}

impl std::fmt::Debug for CpuSelfPrefixTrainer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CpuSelfPrefixTrainer")
            .field("artifact_bytes", &self.artifact.len())
            .field("samples", &self.samples.len())
            .field("config", &self.config)
            .field("attempts", &self.attempts)
            .field("accepted", &self.accepted)
            .finish()
    }
}

impl CpuSelfPrefixTrainer {
    pub fn from_artifact(
        artifact: &[u8],
        samples: Vec<MemoryWeightSample>,
        config: SelfPrefixConfig,
    ) -> Result<Self> {
        validate_config(config)?;
        if samples.is_empty() {
            return Err(invalid("self-prefix training needs samples"));
        }
        let frozen_scan = scan(artifact, &samples, &config)?;
        if frozen_scan.samples.len() != samples.len() {
            return Err(invalid("self-prefix scan lost samples"));
        }
        Ok(Self {
            artifact: artifact.to_vec(),
            samples,
            config,
            frozen_scan,
            attempts: 0,
            accepted: 0,
        })
    }

    pub fn artifact(&self) -> &[u8] {
        &self.artifact
    }

    pub fn attempts(&self) -> u64 {
        self.attempts
    }

    pub fn accepted(&self) -> u64 {
        self.accepted
    }

    pub fn reading(&self) -> Result<SelfPrefixReading> {
        let scan = scan(&self.artifact, &self.samples, &self.config)?;
        let targets = mapped_targets(&scan, &self.samples, self.config)?;
        reading_from_scan(
            &self.artifact,
            &scan,
            &self.samples,
            &targets,
            self.config.reply_cap,
        )
    }

    pub fn eligibility(&self) -> Result<SelfPrefixEligibility> {
        let scan = scan(&self.artifact, &self.samples, &self.config)?;
        eligibility(&self.artifact, &scan, &self.samples, &self.config)
    }

    pub fn train_steps(&mut self, count: usize) -> Result<Vec<SelfPrefixTrainStep>> {
        let mut steps = Vec::with_capacity(count);
        for _ in 0..count {
            steps.push(self.train_step()?);
        }
        Ok(steps)
    }

    pub fn train_step(&mut self) -> Result<SelfPrefixTrainStep> {
        let index = self.attempts;
        let source_scan = if self.config.mode == SelfPrefixMode::StaticGenerated {
            self.frozen_scan.clone()
        } else {
            scan(&self.artifact, &self.samples, &self.config)?
        };
        let source_targets = mapped_targets(&source_scan, &self.samples, self.config)?;
        let eligible = eligible(
            &self.artifact,
            &source_scan,
            &source_targets,
            self.config.rank_margin,
        )?;
        if eligible.is_empty() {
            self.attempts = self
                .attempts
                .checked_add(1)
                .ok_or_else(|| invalid("self-prefix attempt counter exhausted"))?;
            let reading = self.reading()?;
            return Ok(SelfPrefixTrainStep {
                index,
                updated: false,
                sample: None,
                reply_offset: None,
                target: None,
                coordinate: None,
                before: reading.clone(),
                after: reading,
                parity: None,
            });
        }
        let (sample, reply_offset, target) = eligible[(index as usize) % eligible.len()];
        let decision = source_scan.samples[sample]
            .decisions
            .iter()
            .find(|decision| decision.reply_offset == reply_offset)
            .ok_or_else(|| invalid("self-prefix eligible decision disappeared"))?;
        let before_scan = scan(&self.artifact, &self.samples, &self.config)?;
        let before = reading_from_scan(
            &self.artifact,
            &before_scan,
            &self.samples,
            &source_targets,
            self.config.reply_cap,
        )?;
        let forward = CpuFbsForward::from_artifact(&self.artifact)
            .map_err(|error| invalid(format!("self-prefix forward: {error}")))?;
        let proposal = propose_writer_update(
            &self.artifact,
            &forward,
            self.config.rank_margin,
            &decision.query_q12,
            target,
        )?;
        self.attempts = self
            .attempts
            .checked_add(1)
            .ok_or_else(|| invalid("self-prefix attempt counter exhausted"))?;
        let Some(proposal) = proposal else {
            return Ok(SelfPrefixTrainStep {
                index,
                updated: false,
                sample: Some(sample),
                reply_offset: Some(reply_offset),
                target: Some(target),
                coordinate: None,
                before: before.clone(),
                after: before,
                parity: None,
            });
        };
        let after_scan = scan(&proposal.artifact, &self.samples, &self.config)?;
        let after = reading_from_scan(
            &proposal.artifact,
            &after_scan,
            &self.samples,
            &source_targets,
            self.config.reply_cap,
        )?;
        let updated = strictly_better(&after, &before);
        if !updated {
            return Ok(SelfPrefixTrainStep {
                index,
                updated: false,
                sample: Some(sample),
                reply_offset: Some(reply_offset),
                target: Some(target),
                coordinate: proposal.coordinates.first().copied(),
                before: before.clone(),
                after: before,
                parity: None,
            });
        }
        let parity = audit_memory_weight_samples(
            &proposal.artifact,
            &self.samples,
            self.config.memory_enabled,
        )?;
        if !parity.is_exact() {
            return Err(invalid(
                "self-prefix update lost complete CPU/serving parity",
            ));
        }
        self.artifact = proposal.artifact;
        self.accepted = self
            .accepted
            .checked_add(1)
            .ok_or_else(|| invalid("self-prefix accepted counter exhausted"))?;
        Ok(SelfPrefixTrainStep {
            index,
            updated: true,
            sample: Some(sample),
            reply_offset: Some(reply_offset),
            target: Some(target),
            coordinate: proposal.coordinates.first().copied(),
            before,
            after,
            parity: Some(parity),
        })
    }
}

pub fn evaluate_self_prefix(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    config: SelfPrefixConfig,
) -> Result<SelfPrefixReading> {
    let scan = scan(artifact, samples, &config)?;
    let targets = mapped_targets(&scan, samples, config)?;
    reading_from_scan(artifact, &scan, samples, &targets, config.reply_cap)
}

pub fn evaluate_self_prefix_prompts(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    config: SelfPrefixConfig,
) -> Result<Vec<SelfPrefixPromptReading>> {
    let scan = scan(artifact, samples, &config)?;
    let targets = mapped_targets(&scan, samples, config)?;
    prompt_readings(artifact, &scan, samples, &targets, config.reply_cap)
}

fn validate_config(config: SelfPrefixConfig) -> Result<()> {
    let rank = config.rank_margin;
    if rank.margin_q12 <= 0
        || rank.step_q12 <= 0
        || rank.parameter_limit_q12 <= 0
        || rank.coordinates_per_step != 1
    {
        return Err(invalid(
            "self-prefix requires positive rank geometry and K=1",
        ));
    }
    if !(1..=64).contains(&config.reply_cap) {
        return Err(invalid("self-prefix reply cap is out of bounds"));
    }
    Ok(())
}

fn scan(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    config: &SelfPrefixConfig,
) -> Result<PrefixScan> {
    let rows = samples
        .par_iter()
        .enumerate()
        .map(|(sample, value)| scan_sample(artifact, sample, value, config))
        .collect::<Result<Vec<_>>>()?;
    Ok(PrefixScan { samples: rows })
}

fn scan_sample(
    artifact: &[u8],
    sample_index: usize,
    sample: &MemoryWeightSample,
    config: &SelfPrefixConfig,
) -> Result<PrefixSampleScan> {
    if sample.prompt.is_empty() {
        return Err(invalid("self-prefix sample prompt is empty"));
    }
    if sample.reply.is_empty() {
        return Err(invalid("self-prefix sample reply is empty"));
    }
    let reference = normalized_reference(&sample.reply).to_vec();
    if reference.is_empty() {
        return Err(invalid("self-prefix sample has no reference content"));
    }
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("self-prefix scan forward: {error}")))?;
    if !config.memory_enabled {
        forward.disable_memory();
    }
    let context = forward.context_length();
    if sample.prompt.len() > context {
        return Err(invalid("self-prefix prompt exceeds the context"));
    }
    let mut current = None;
    for (index, &token) in sample.prompt.iter().enumerate() {
        if index + 1 == sample.prompt.len() {
            forward.mark_reply_start();
        }
        current = Some(
            forward
                .step(token, CpuFbsHead::Writer)
                .map_err(|error| invalid(format!("self-prefix prompt step: {error}")))?,
        );
    }
    let mut current = current.ok_or_else(|| invalid("self-prefix sample prompt is empty"))?;
    let mut decisions = Vec::new();
    let mut tokens = Vec::new();
    if config.mode == SelfPrefixMode::Reference {
        let horizon = reference.len().min(config.reply_cap);
        for (offset, &target) in reference.iter().enumerate().take(horizon) {
            decisions.push(SelfPrefixDecision {
                sample: sample_index,
                reply_offset: offset,
                query_q12: current.query_q12.clone(),
                generated_token: current.token,
                reference_target: target,
            });
            if current.token != 0 {
                tokens.push(current.token);
            }
            if offset + 1 < horizon {
                current = forward
                    .step(target, CpuFbsHead::Writer)
                    .map_err(|error| invalid(format!("self-prefix reference step: {error}")))?;
            }
        }
        return Ok(PrefixSampleScan {
            decisions,
            tokens,
            stop: "cap",
        });
    }
    let mut offset = 0usize;
    let stop = loop {
        if let Some(reference_target) = reference.get(offset).copied() {
            decisions.push(SelfPrefixDecision {
                sample: sample_index,
                reply_offset: offset,
                query_q12: current.query_q12.clone(),
                generated_token: current.token,
                reference_target,
            });
        }
        let end = reply_stop(
            current.token,
            offset,
            sample.prompt.len(),
            config.reply_cap,
            context,
        );
        if end != Some(ReplyEnd::Bos) {
            tokens.push(current.token);
        }
        if let Some(end) = end {
            break match end {
                ReplyEnd::Bos => "bos",
                ReplyEnd::Length => "cap",
                ReplyEnd::Context => "context",
            };
        }
        current = forward
            .step(current.token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("self-prefix generated step: {error}")))?;
        offset += 1;
    };
    Ok(PrefixSampleScan {
        decisions,
        tokens,
        stop,
    })
}

fn mapped_targets(
    scan: &PrefixScan,
    samples: &[MemoryWeightSample],
    config: SelfPrefixConfig,
) -> Result<BTreeMap<(usize, usize), u32>> {
    let mut targets = BTreeMap::new();
    if scan.samples.len() != samples.len() {
        return Err(invalid("self-prefix target sample count differs"));
    }
    for (sample_index, sample) in samples.iter().enumerate() {
        let reference = normalized_reference(&sample.reply);
        for (offset, &target) in reference.iter().enumerate().take(config.reply_cap) {
            targets.insert((sample_index, offset), target);
        }
    }
    if targets.is_empty() {
        return Err(invalid("self-prefix target map is empty"));
    }
    for (sample_index, sample_scan) in scan.samples.iter().enumerate() {
        if sample_scan
            .decisions
            .iter()
            .any(|decision| decision.sample != sample_index)
        {
            return Err(invalid("self-prefix decision sample index differs"));
        }
    }
    if config.targets == SelfPrefixTargets::TrueLabels {
        return Ok(targets);
    }
    let mut cells = BTreeMap::<u32, Vec<((usize, usize), u32)>>::new();
    for (&key, &target) in &targets {
        cells
            .entry(target % TARGET_CELLS)
            .or_default()
            .push((key, target));
    }
    let mut changed = false;
    for rows in cells.values_mut() {
        if rows.len() < 2 {
            continue;
        }
        let shift = 1 + config.shuffle_seed as usize % (rows.len() - 1);
        let original = rows.iter().map(|&(_, target)| target).collect::<Vec<_>>();
        let mut shuffled = original.clone();
        shuffled.rotate_right(shift);
        for (row, target) in rows.iter_mut().zip(shuffled) {
            changed |= row.1 != target;
            row.1 = target;
        }
    }
    if !changed {
        return Err(invalid("same-cell target shuffle is vacuous"));
    }
    for (key, target) in cells.into_values().flatten() {
        targets.insert(key, target);
    }
    Ok(targets)
}

fn eligible(
    artifact: &[u8],
    scan: &PrefixScan,
    targets: &BTreeMap<(usize, usize), u32>,
    config: RankMarginConfig,
) -> Result<Vec<(usize, usize, u32)>> {
    let forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("self-prefix eligibility forward: {error}")))?;
    let mut rows = Vec::new();
    for sample in &scan.samples {
        for decision in &sample.decisions {
            let target = targets[&(decision.sample, decision.reply_offset)];
            let reading = reading_from_forward(&forward, config, &decision.query_q12, target)?;
            if reading.target_admitted && reading.margin_loss > 0 {
                rows.push((decision.sample, decision.reply_offset, target));
            }
        }
    }
    Ok(rows)
}

fn eligibility(
    artifact: &[u8],
    scan: &PrefixScan,
    samples: &[MemoryWeightSample],
    config: &SelfPrefixConfig,
) -> Result<SelfPrefixEligibility> {
    let targets = mapped_targets(scan, samples, *config)?;
    let rows = eligible(artifact, scan, &targets, config.rank_margin)?;
    let prompts = rows.iter().map(|row| row.0).collect::<BTreeSet<_>>().len();
    let target_cells = rows
        .iter()
        .map(|row| row.2 % TARGET_CELLS)
        .collect::<BTreeSet<_>>()
        .len();
    let mut per_cell = BTreeMap::<u32, u64>::new();
    for row in &rows {
        *per_cell.entry(row.2 % TARGET_CELLS).or_default() += 1;
    }
    let same_cell_shuffle_pairs = per_cell
        .values()
        .filter(|&&count| count >= 2)
        .map(|count| count / 2)
        .sum();
    Ok(SelfPrefixEligibility {
        eligible_decisions: rows.len() as u64,
        prompts: prompts as u64,
        target_cells: target_cells as u64,
        same_cell_shuffle_pairs,
    })
}

fn reading_from_scan(
    artifact: &[u8],
    scan: &PrefixScan,
    samples: &[MemoryWeightSample],
    targets: &BTreeMap<(usize, usize), u32>,
    cap: usize,
) -> Result<SelfPrefixReading> {
    if scan.samples.len() != samples.len() {
        return Err(invalid("self-prefix reading sample count differs"));
    }
    let forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("self-prefix reading forward: {error}")))?;
    let mut reading = SelfPrefixReading {
        samples: samples.len() as u64,
        ..SelfPrefixReading::default()
    };
    let mut distinct = BTreeSet::<Vec<u32>>::new();
    for (sample_index, (sample, scan)) in samples.iter().zip(&scan.samples).enumerate() {
        let reference = normalized_reference(&sample.reply);
        let horizon = reference.len().min(cap);
        reading.fixed_horizon_decisions = checked_add(
            reading.fixed_horizon_decisions,
            horizon as u64,
            "self-prefix horizon decisions",
        )?;
        let mut matched = 0usize;
        for offset in 0..horizon {
            let Some(decision) = scan
                .decisions
                .iter()
                .find(|decision| decision.reply_offset == offset)
            else {
                reading.missing_decisions =
                    checked_add(reading.missing_decisions, 1, "self-prefix missing")?;
                reading.target_rank_sum = checked_add(
                    reading.target_rank_sum,
                    forward.vocabulary() as u64 + 1,
                    "self-prefix missing rank",
                )?;
                reading.target_distance_sum = checked_add(
                    reading.target_distance_sum,
                    i64::MAX as u64 / 4,
                    "self-prefix missing distance",
                )?;
                continue;
            };
            reading.observed_decisions =
                checked_add(reading.observed_decisions, 1, "self-prefix observed")?;
            let target = targets[&(sample_index, offset)];
            let diagnostic = reading_from_forward(
                &forward,
                RankMarginConfig::default(),
                &decision.query_q12,
                target,
            )?;
            if decision.generated_token == target {
                reading.generated_top1 =
                    checked_add(reading.generated_top1, 1, "self-prefix generated top1")?;
            }
            if diagnostic.top1 == target {
                reading.writer_top1 =
                    checked_add(reading.writer_top1, 1, "self-prefix writer top1")?;
            }
            if diagnostic.target_admitted {
                reading.target_admitted =
                    checked_add(reading.target_admitted, 1, "self-prefix admitted")?;
            }
            reading.target_rank_sum = checked_add(
                reading.target_rank_sum,
                diagnostic.target_rank as u64,
                "self-prefix rank sum",
            )?;
            reading.target_distance_sum = checked_add(
                reading.target_distance_sum,
                diagnostic.target_distance as u64,
                "self-prefix distance sum",
            )?;
            if decision.generated_token
                == reference
                    .get(offset)
                    .copied()
                    .ok_or_else(|| invalid("self-prefix reference horizon differs"))?
                && matched == offset
            {
                matched += 1;
            }
        }
        reading.matched_prefix_depth_sum = checked_add(
            reading.matched_prefix_depth_sum,
            matched as u64,
            "self-prefix prefix depth",
        )?;
        *reading.first_divergence.entry(matched).or_default() += 1;
        if scan.tokens.first() == reference.first() {
            reading.opening_correct =
                checked_add(reading.opening_correct, 1, "self-prefix opening")?;
        }
        if scan.tokens == reference && scan.stop == "bos" {
            reading.exact_reference = checked_add(reading.exact_reference, 1, "self-prefix exact")?;
        }
        match scan.stop {
            "bos" => reading.bos_stopped += 1,
            "cap" => reading.cap_stopped += 1,
            "context" => reading.context_stopped += 1,
            _ => return Err(invalid("self-prefix stop label differs")),
        }
        reading.generated_tokens = checked_add(
            reading.generated_tokens,
            scan.tokens.len() as u64,
            "self-prefix generated tokens",
        )?;
        if !scan.tokens.is_empty() {
            distinct.insert(scan.tokens.clone());
        }
        if repeated_4gram(&scan.tokens) {
            reading.repeated_4gram_replies = checked_add(
                reading.repeated_4gram_replies,
                1,
                "self-prefix repeated replies",
            )?;
        }
    }
    reading.distinct_non_empty = distinct.len() as u64;
    Ok(reading)
}

fn prompt_readings(
    _artifact: &[u8],
    scan: &PrefixScan,
    samples: &[MemoryWeightSample],
    targets: &BTreeMap<(usize, usize), u32>,
    cap: usize,
) -> Result<Vec<SelfPrefixPromptReading>> {
    let mut readings = Vec::with_capacity(samples.len());
    for (sample_index, (sample, scan)) in samples.iter().zip(&scan.samples).enumerate() {
        let reference = normalized_reference(&sample.reply);
        let horizon = reference.len().min(cap);
        let mut matched = 0usize;
        let mut generated_top1 = 0u64;
        let mut observed_decisions = 0u64;
        for offset in 0..horizon {
            let Some(decision) = scan
                .decisions
                .iter()
                .find(|decision| decision.reply_offset == offset)
            else {
                continue;
            };
            observed_decisions += 1;
            let target = targets[&(sample_index, offset)];
            generated_top1 += u64::from(decision.generated_token == target);
            if decision.generated_token == reference[offset] && matched == offset {
                matched += 1;
            }
        }
        readings.push(SelfPrefixPromptReading {
            sample: sample_index,
            matched_prefix_depth: matched,
            generated_top1,
            observed_decisions,
            opening_correct: scan.tokens.first() == reference.first(),
            exact_reference: scan.tokens == reference && scan.stop == "bos",
            stop: scan.stop.to_owned(),
            generated_tokens: scan.tokens.len() as u64,
        });
    }
    Ok(readings)
}

fn strictly_better(candidate: &SelfPrefixReading, control: &SelfPrefixReading) -> bool {
    if candidate.missing_decisions > control.missing_decisions {
        return false;
    }
    (
        candidate.generated_top1,
        std::cmp::Reverse(candidate.target_rank_sum),
        std::cmp::Reverse(candidate.target_distance_sum),
    ) > (
        control.generated_top1,
        std::cmp::Reverse(control.target_rank_sum),
        std::cmp::Reverse(control.target_distance_sum),
    )
}

fn repeated_4gram(tokens: &[u32]) -> bool {
    let mut seen = BTreeSet::new();
    tokens
        .windows(4)
        .any(|window| !seen.insert(window.to_vec()))
}

fn checked_add(left: u64, right: u64, name: &str) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid(format!("{name} overflows")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_discrete::normalization_mechanism::orbit_root_coords;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};
    use uor_r4_integer::stack::{
        export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot, FbsReplyTable,
        FbsTrunkSnapshot, IntegerStackModel, MEMORY_PAD,
    };

    fn snapshot() -> FbsTrunkSnapshot {
        let mut snapshot = synthetic_fbs_trunk_with(
            151,
            FbsSynth {
                vocab: 4096,
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
        snapshot.embed.clear();
        for token in 0..snapshot.vocab {
            for block in 0..snapshot.width / 8 {
                let root = match block {
                    0 => token & 15,
                    7 => ((token >> 4) ^ (token >> 8)) & 15,
                    _ => {
                        ((token >> 8) * 7 + ((token >> 4) & 15) * 3 + (token & 15) * 5 + block) & 15
                    }
                };
                snapshot.embed.extend(
                    orbit_root_coords(root as u8)
                        .unwrap()
                        .map(|value| value as f32),
                );
            }
        }
        snapshot
    }

    fn artifact_with_memory() -> Vec<u8> {
        let mut snapshot = snapshot();
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 4,
            key_window: snapshot.context,
            opening: vec![FbsMemoryEntry {
                key: vec![3, 4],
                value: 7,
            }],
            opening_weight: vec![1.0; snapshot.width],
            reply: None,
        });
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn artifact_with_reply_memory(first_generated: u32) -> Vec<u8> {
        let mut snapshot = snapshot();
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 4,
            key_window: snapshot.context,
            opening: vec![FbsMemoryEntry {
                key: vec![42, 43],
                value: 8,
            }],
            opening_weight: vec![0.0; snapshot.width],
            reply: Some(FbsReplyTable {
                turn: 3,
                entries: vec![FbsMemoryEntry {
                    key: vec![3, 4, MEMORY_PAD, 1, MEMORY_PAD, first_generated],
                    value: 9,
                }],
                weight: vec![1.0; snapshot.width],
            }),
        });
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn config(mode: SelfPrefixMode, memory_enabled: bool) -> SelfPrefixConfig {
        SelfPrefixConfig {
            rank_margin: RankMarginConfig::default(),
            reply_cap: 8,
            memory_enabled,
            mode,
            targets: SelfPrefixTargets::TrueLabels,
            shuffle_seed: 17,
        }
    }

    fn sample() -> MemoryWeightSample {
        MemoryWeightSample {
            prompt: vec![0, 3, 4],
            reply: vec![7, 8, 9, 1],
        }
    }

    #[test]
    fn scan_marks_reply_start_and_applies_opening_memory() {
        let artifact = artifact_with_memory();
        let memory = uor_r4_integer::stack::read_memory_uorlut01(&artifact)
            .unwrap()
            .unwrap();
        assert_eq!(memory.record.k, 4);
        assert_eq!(memory.opening.keys, vec![vec![3, 4]]);
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        assert!(forward.has_memory());
        forward.step(0, CpuFbsHead::Writer).unwrap();
        forward.step(3, CpuFbsHead::Writer).unwrap();
        forward.mark_reply_start();
        assert_eq!(forward.reply_start(), Some(3));
        let (_, trace) = forward.step_with_trace(4, CpuFbsHead::Writer).unwrap();
        assert_eq!(forward.memory_anchors().opening, vec![3, 4]);
        assert_eq!(trace.memory.unwrap().stored, Some(7));
        let on = scan_sample(
            &artifact,
            0,
            &sample(),
            &config(SelfPrefixMode::DynamicGenerated, true),
        )
        .unwrap();
        let off = scan_sample(
            &artifact,
            0,
            &sample(),
            &config(SelfPrefixMode::DynamicGenerated, false),
        )
        .unwrap();
        assert_eq!(on.decisions[0].reference_target, 7);
        assert_eq!(on.decisions[1].reference_target, 8);
        assert_ne!(on.decisions[0].query_q12, off.decisions[0].query_q12);
    }

    #[test]
    fn generated_feedback_reaches_a_state_reference_feedback_does_not() {
        let artifact = export_fbs_trunk_uorlut01(&snapshot()).unwrap();
        let generated = scan_sample(
            &artifact,
            0,
            &sample(),
            &config(SelfPrefixMode::DynamicGenerated, false),
        )
        .unwrap();
        let alternative = generated
            .tokens
            .iter()
            .map(|&token| (token + 1) % 4096)
            .chain([1])
            .collect::<Vec<_>>();
        let reference = scan_sample(
            &artifact,
            0,
            &MemoryWeightSample {
                prompt: vec![0, 3, 4],
                reply: alternative,
            },
            &config(SelfPrefixMode::Reference, false),
        )
        .unwrap();
        assert_ne!(
            generated.decisions[1].query_q12,
            reference.decisions[1].query_q12
        );
    }

    #[test]
    fn generated_scan_uses_reply_memory_after_the_opening() {
        let plain = export_fbs_trunk_uorlut01(&snapshot()).unwrap();
        let first = scan_sample(
            &plain,
            0,
            &sample(),
            &config(SelfPrefixMode::DynamicGenerated, false),
        )
        .unwrap()
        .decisions[0]
            .generated_token;
        let artifact = artifact_with_reply_memory(first);
        let on = scan_sample(
            &artifact,
            0,
            &sample(),
            &config(SelfPrefixMode::DynamicGenerated, true),
        )
        .unwrap();
        let off = scan_sample(
            &artifact,
            0,
            &sample(),
            &config(SelfPrefixMode::DynamicGenerated, false),
        )
        .unwrap();
        assert_ne!(on.decisions[1].query_q12, off.decisions[1].query_q12);
    }

    #[test]
    fn early_bos_counts_the_missing_fixed_horizon_as_wrong() {
        let artifact = export_fbs_trunk_uorlut01(&snapshot()).unwrap();
        let fabricated = PrefixScan {
            samples: vec![PrefixSampleScan {
                decisions: vec![SelfPrefixDecision {
                    sample: 0,
                    reply_offset: 0,
                    query_q12: vec![0; 64],
                    generated_token: 0,
                    reference_target: 7,
                }],
                tokens: vec![],
                stop: "bos",
            }],
        };
        let samples = vec![MemoryWeightSample {
            prompt: vec![0, 1, 2],
            reply: vec![7, 8, 1],
        }];
        let targets = mapped_targets(
            &fabricated,
            &samples,
            config(SelfPrefixMode::DynamicGenerated, false),
        )
        .unwrap();
        assert_eq!(targets.len(), 2);
        let reading = reading_from_scan(&artifact, &fabricated, &samples, &targets, 8).unwrap();
        assert_eq!(reading.fixed_horizon_decisions, 2);
        assert_eq!(reading.observed_decisions, 1);
        assert_eq!(reading.missing_decisions, 1);
        assert!(reading.target_rank_sum >= 4097);
    }

    #[test]
    fn same_cell_shuffle_preserves_cells_and_changes_targets() {
        let scan = PrefixScan {
            samples: vec![PrefixSampleScan {
                decisions: vec![
                    SelfPrefixDecision {
                        sample: 0,
                        reply_offset: 0,
                        query_q12: Vec::new(),
                        generated_token: 1,
                        reference_target: 10,
                    },
                    SelfPrefixDecision {
                        sample: 0,
                        reply_offset: 1,
                        query_q12: Vec::new(),
                        generated_token: 1,
                        reference_target: 266,
                    },
                    SelfPrefixDecision {
                        sample: 0,
                        reply_offset: 2,
                        query_q12: Vec::new(),
                        generated_token: 1,
                        reference_target: 20,
                    },
                    SelfPrefixDecision {
                        sample: 0,
                        reply_offset: 3,
                        query_q12: Vec::new(),
                        generated_token: 1,
                        reference_target: 276,
                    },
                ],
                tokens: Vec::new(),
                stop: "cap",
            }],
        };
        let config = SelfPrefixConfig {
            targets: SelfPrefixTargets::SameCellShuffled,
            ..config(SelfPrefixMode::StaticGenerated, false)
        };
        let samples = [MemoryWeightSample {
            prompt: vec![0, 3, 4],
            reply: vec![10, 266, 20, 276, 1],
        }];
        let targets = mapped_targets(&scan, &samples, config).unwrap();
        assert_eq!(targets[&(0, 0)] % TARGET_CELLS, 10);
        assert_eq!(targets[&(0, 0)], 266);
        assert_eq!(targets[&(0, 1)], 10);
        assert_eq!(targets[&(0, 2)], 276);
        assert_eq!(targets[&(0, 3)], 20);
    }

    #[test]
    fn scan_and_fixed_horizon_reading_are_deterministic() {
        let artifact = artifact_with_memory();
        let config = config(SelfPrefixMode::DynamicGenerated, true);
        let first = scan(&artifact, &[sample()], &config).unwrap();
        let second = scan(&artifact, &[sample()], &config).unwrap();
        let targets = mapped_targets(&first, &[sample()], config).unwrap();
        let left =
            reading_from_scan(&artifact, &first, &[sample()], &targets, config.reply_cap).unwrap();
        let right =
            reading_from_scan(&artifact, &second, &[sample()], &targets, config.reply_cap).unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn trainer_executes_an_admitted_generated_query_and_reparses_its_artifact() {
        let artifact = artifact_with_memory();
        let mut sample = sample();
        sample.reply = vec![7, 1];
        let mut scan_config = config(SelfPrefixMode::DynamicGenerated, true);
        scan_config.reply_cap = 1;
        let scanned = scan(&artifact, std::slice::from_ref(&sample), &scan_config).unwrap();
        let query = &scanned.samples[0].decisions[0].query_q12;
        let forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let target = (2..4096)
            .find(|&token| {
                reading_from_forward(&forward, RankMarginConfig::default(), query, token)
                    .is_ok_and(|reading| reading.target_admitted && reading.margin_loss > 0)
            })
            .expect("synthetic artifact has an admitted training target");
        sample.reply = vec![target, 1];
        let mut trainer =
            CpuSelfPrefixTrainer::from_artifact(&artifact, vec![sample], scan_config).unwrap();
        assert!(trainer.eligibility().unwrap().eligible_decisions > 0);
        let step = trainer.train_steps(1).unwrap().remove(0);
        assert_eq!(step.index, 0);
        assert_eq!(trainer.attempts(), 1);
        IntegerStackModel::parse(trainer.artifact()).unwrap();
        if step.updated {
            assert_eq!(trainer.accepted(), 1);
            assert!(step.parity.unwrap().is_exact());
        }
    }

    #[test]
    fn reference_mode_continues_past_a_predicted_bos() {
        let artifact = artifact_with_memory();
        let mut early = None;
        for token in 2..512 {
            let candidate = MemoryWeightSample {
                prompt: vec![0, 3, token],
                reply: vec![token + 1, token + 2, token + 3, 1],
            };
            let generated = scan_sample(
                &artifact,
                0,
                &candidate,
                &config(SelfPrefixMode::DynamicGenerated, true),
            )
            .unwrap();
            if generated.stop == "bos" {
                early = Some(candidate);
                break;
            }
        }
        let sample = early.expect("synthetic writer predicts BOS for at least one prompt");
        let reference = scan_sample(
            &artifact,
            0,
            &sample,
            &config(SelfPrefixMode::Reference, true),
        )
        .unwrap();
        assert_eq!(reference.decisions.len(), 3);
        assert_eq!(reference.stop, "cap");
    }

    #[test]
    fn malformed_config_vacuous_shuffle_and_no_eligible_rows_are_refused_or_empty() {
        let artifact = artifact_with_memory();
        let mut invalid = config(SelfPrefixMode::DynamicGenerated, true);
        invalid.rank_margin.coordinates_per_step = 2;
        assert!(CpuSelfPrefixTrainer::from_artifact(&artifact, vec![sample()], invalid).is_err());

        let fabricated = PrefixScan {
            samples: vec![PrefixSampleScan {
                decisions: vec![SelfPrefixDecision {
                    sample: 0,
                    reply_offset: 0,
                    query_q12: Vec::new(),
                    generated_token: 1,
                    reference_target: 10,
                }],
                tokens: Vec::new(),
                stop: "cap",
            }],
        };
        let shuffled = SelfPrefixConfig {
            targets: SelfPrefixTargets::SameCellShuffled,
            ..config(SelfPrefixMode::StaticGenerated, false)
        };
        let vacuous_samples = [MemoryWeightSample {
            prompt: vec![0, 3, 4],
            reply: vec![10, 1],
        }];
        assert!(mapped_targets(&fabricated, &vacuous_samples, shuffled).is_err());

        let mut one = config(SelfPrefixMode::DynamicGenerated, true);
        one.reply_cap = 1;
        let probe = MemoryWeightSample {
            prompt: vec![0, 3, 4],
            reply: vec![7, 1],
        };
        let scanned = scan(&artifact, std::slice::from_ref(&probe), &one).unwrap();
        let query = &scanned.samples[0].decisions[0].query_q12;
        let forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let top1 = reading_from_forward(&forward, RankMarginConfig::default(), query, 2)
            .unwrap()
            .top1;
        let no_eligible = MemoryWeightSample {
            prompt: vec![0, 3, 4],
            reply: vec![top1, 1],
        };
        let trainer =
            CpuSelfPrefixTrainer::from_artifact(&artifact, vec![no_eligible], one).unwrap();
        assert_eq!(trainer.eligibility().unwrap().eligible_decisions, 0);
    }

    #[test]
    fn early_bos_cannot_win_acceptance_by_increasing_missing_rows() {
        let control = SelfPrefixReading {
            generated_top1: 1,
            target_rank_sum: 20,
            target_distance_sum: 30,
            missing_decisions: 2,
            ..SelfPrefixReading::default()
        };
        let candidate = SelfPrefixReading {
            generated_top1: 2,
            target_rank_sum: 1,
            target_distance_sum: 1,
            missing_decisions: 3,
            ..SelfPrefixReading::default()
        };
        assert!(!strictly_better(&candidate, &control));
    }

    #[test]
    fn static_training_keeps_its_frozen_query_corpus() {
        let artifact = artifact_with_memory();
        let mut config = config(SelfPrefixMode::StaticGenerated, true);
        config.reply_cap = 1;
        let mut trainer =
            CpuSelfPrefixTrainer::from_artifact(&artifact, vec![sample()], config).unwrap();
        let frozen = trainer.frozen_scan.clone();
        let _ = trainer.train_steps(2).unwrap();
        assert_eq!(
            trainer.frozen_scan.samples[0].decisions,
            frozen.samples[0].decisions
        );
    }
}
