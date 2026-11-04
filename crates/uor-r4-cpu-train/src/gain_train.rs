//! Exact distance training of the final served FBS gain.
//!
//! The final gain changes every served query without changing the
//! teacher-forced recurrent trajectory. The cached path stores the exact
//! normalization immediately before that gain, then replays the gain, Q12
//! conversion, writer decision, turn-memory movement and point fallback.

use rayon::prelude::*;
use uor_r4_integer::stack::{read_memory_uorlut01, IntegerStackModel, POINT_KEY_BOS};

use crate::fbs::CpuServedTrace;
use crate::memory_train::{checked_sample_totals, validate_samples};
use crate::{
    audit_memory_weight_samples, invalid, CpuFbsForward, CpuFbsHead, CpuTrainError,
    MemoryWeightParityReading, MemoryWeightReading, MemoryWeightSample,
};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const CPU_SERVED_GAIN_TRAINER_SCHEMA: &str = "uor-r4.cpu-served-gain-trainer/1";
pub const CPU_TERMINATION_GAIN_TRAINER_SCHEMA: &str = "uor-r4.cpu-termination-gain-trainer/1";
const CORPUS_EOS: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServedGainConfig {
    pub step_code: i16,
}

impl Default for ServedGainConfig {
    fn default() -> Self {
        Self { step_code: 1 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServedGainStep {
    pub index: u64,
    pub updated: bool,
    pub coordinate: Option<usize>,
    pub before_code: Option<i16>,
    pub after_code: Option<i16>,
    pub before: MemoryWeightReading,
    pub after: MemoryWeightReading,
    pub parity: Option<MemoryWeightParityReading>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServedGainMove {
    pub coordinate: usize,
    pub before_code: i16,
    pub after_code: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminationGainConfig {
    pub step_code: i16,
}

impl Default for TerminationGainConfig {
    fn default() -> Self {
        Self { step_code: 1 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminationGainReading {
    pub artifact_sha256: String,
    pub samples: usize,
    pub decisions: u64,
    pub terminal_decisions: u64,
    pub nonterminal_decisions: u64,
    pub terminal_bos_top1: u64,
    pub terminal_bos_admitted: u64,
    pub terminal_bos_top8: u64,
    pub terminal_bos_rank_sum: u64,
    pub terminal_bos_distance_sum: u64,
    pub terminal_content_top1: u64,
    pub nonterminal_bos_top1: u64,
    pub nonterminal_content_top1: u64,
    pub nonterminal_bos_distance_sum: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminationGainEvaluation {
    pub reading: TerminationGainReading,
    pub terminal_rows: Vec<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminationGainStep {
    pub index: u64,
    pub updated: bool,
    pub coordinate: Option<usize>,
    pub before_code: Option<i16>,
    pub after_code: Option<i16>,
    pub before: TerminationGainReading,
    pub after: TerminationGainReading,
    pub parity: Option<MemoryWeightParityReading>,
}

#[derive(Clone)]
pub struct CpuServedGainTrainer {
    initial_artifact: Vec<u8>,
    artifact: Vec<u8>,
    samples: Vec<MemoryWeightSample>,
    config: ServedGainConfig,
    steps: u64,
    trajectory: Option<CpuGainTrajectory>,
}

impl std::fmt::Debug for CpuServedGainTrainer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CpuServedGainTrainer")
            .field("artifact_bytes", &self.artifact.len())
            .field("samples", &self.samples.len())
            .field("step_code", &self.config.step_code)
            .field("steps", &self.steps)
            .field("cached_trajectory", &self.trajectory.is_some())
            .finish()
    }
}

impl CpuServedGainTrainer {
    pub fn from_artifact(
        artifact: &[u8],
        samples: Vec<MemoryWeightSample>,
        config: ServedGainConfig,
    ) -> Result<Self> {
        if config.step_code == 0 {
            return Err(invalid("served-gain step code must be nonzero"));
        }
        validate_samples(artifact, &samples)?;
        validate_artifact(artifact)?;
        let reading = evaluate(artifact, &samples)?;
        if reading.decisions == 0 {
            return Err(invalid("served-gain samples contain no decision"));
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
        config: ServedGainConfig,
    ) -> Result<Self> {
        let mut trainer = Self::from_artifact(artifact, samples, config)?;
        trainer.trajectory = Some(CpuGainTrajectory::build(artifact, &trainer.samples)?);
        Ok(trainer)
    }

    pub fn artifact(&self) -> &[u8] {
        &self.artifact
    }

    pub fn initial_artifact(&self) -> &[u8] {
        &self.initial_artifact
    }

    pub fn config(&self) -> ServedGainConfig {
        self.config
    }

    pub fn step_count(&self) -> u64 {
        self.steps
    }

    pub fn reading(&self) -> Result<MemoryWeightReading> {
        if let Some(trajectory) = &self.trajectory {
            trajectory.evaluate(&self.artifact)
        } else {
            evaluate(&self.artifact, &self.samples)
        }
    }

    pub fn train_step(&mut self) -> Result<ServedGainStep> {
        let index = self.steps;
        let before = self.reading()?;
        let gain = read_gain(&self.artifact)?;
        let mut candidates = Vec::with_capacity(gain.len() * 2);
        for (coordinate, &original) in gain.iter().enumerate() {
            for delta in [self.config.step_code, -self.config.step_code] {
                let Some(after) = original.checked_add(delta) else {
                    continue;
                };
                if after != original {
                    candidates.push((coordinate, original, after));
                }
            }
        }
        let evaluated = candidates
            .par_chunks(16)
            .map(|chunk| {
                let mut best: Option<GainCandidate> = None;
                for &(coordinate, original, after) in chunk {
                    let mut trial = gain.clone();
                    trial[coordinate] = after;
                    let artifact = patch_gain(&self.artifact, &trial)?;
                    let reading = match &self.trajectory {
                        Some(trajectory) => trajectory.evaluate(&artifact)?,
                        None => evaluate(&artifact, &self.samples)?,
                    };
                    if !strict_improvement(&reading, &before) {
                        continue;
                    }
                    let candidate = GainCandidate {
                        coordinate,
                        before_code: original,
                        after_code: after,
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
            .ok_or_else(|| invalid("served-gain step counter exhausted"))?;
        let Some(best) = best else {
            return Ok(ServedGainStep {
                index,
                updated: false,
                coordinate: None,
                before_code: None,
                after_code: None,
                before: before.clone(),
                after: before,
                parity: None,
            });
        };
        let parity = audit_memory_weight_samples(&best.artifact, &self.samples, true)?;
        if !parity.is_exact() {
            return Err(invalid(
                "served-gain update lost complete CPU/serving parity",
            ));
        }
        self.artifact = best.artifact;
        Ok(ServedGainStep {
            index,
            updated: true,
            coordinate: Some(best.coordinate),
            before_code: Some(best.before_code),
            after_code: Some(best.after_code),
            before,
            after: best.reading,
            parity: Some(parity),
        })
    }
}

/// Derive one terminal label for each teacher-forced decision. Every training
/// reply must end in corpus EOS; all other decisions are nonterminal.
pub fn termination_gain_labels(samples: &[MemoryWeightSample]) -> Result<Vec<bool>> {
    if samples.is_empty() {
        return Err(invalid("termination training needs samples"));
    }
    let mut labels = Vec::new();
    for (sample_index, sample) in samples.iter().enumerate() {
        if sample.prompt.is_empty() || sample.reply.is_empty() {
            return Err(invalid(format!(
                "termination sample {sample_index} has an empty prompt or reply"
            )));
        }
        let tokens = sample
            .prompt
            .iter()
            .chain(&sample.reply)
            .copied()
            .collect::<Vec<_>>();
        if tokens.len() < 2 {
            return Err(invalid(format!(
                "termination sample {sample_index} has no decision"
            )));
        }
        if sample.reply.last() != Some(&CORPUS_EOS) {
            return Err(invalid(format!(
                "termination sample {sample_index} reply does not end in EOS"
            )));
        }
        if tokens[..tokens.len() - 1].contains(&CORPUS_EOS) {
            return Err(invalid(format!(
                "termination sample {sample_index} has EOS before the final reply token"
            )));
        }
        let mut sample_labels = Vec::with_capacity(tokens.len() - 1);
        let mut terminal = 0usize;
        for &target in &tokens[1..] {
            let is_terminal = target == CORPUS_EOS;
            terminal += usize::from(is_terminal);
            sample_labels.push(is_terminal);
        }
        if terminal != 1 {
            return Err(invalid(format!(
                "termination sample {sample_index} has {terminal} EOS targets"
            )));
        }
        if sample_labels.iter().all(|terminal| *terminal) {
            return Err(invalid(format!(
                "termination sample {sample_index} has no nonterminal decision"
            )));
        }
        labels.extend(sample_labels);
    }
    if labels.iter().all(|terminal| *terminal) || labels.iter().all(|terminal| !*terminal) {
        return Err(invalid(
            "termination training needs both terminal and nonterminal decisions",
        ));
    }
    Ok(labels)
}

/// Conservatively rotate terminal labels to different decisions while keeping
/// the exact terminal/nonterminal counts.
pub fn shuffle_termination_gain_labels(labels: &[bool], seed: u64) -> Result<Vec<bool>> {
    if labels.is_empty() {
        return Err(invalid("termination label shuffle needs labels"));
    }
    let terminal = labels
        .iter()
        .enumerate()
        .filter_map(|(index, &terminal)| terminal.then_some(index))
        .collect::<Vec<_>>();
    if terminal.is_empty() || terminal.len() == labels.len() {
        return Err(invalid("termination label shuffle needs both classes"));
    }
    let shift = 1 + (seed as usize % (labels.len() - 1));
    let mut shuffled = vec![false; labels.len()];
    for index in terminal {
        shuffled[(index + shift) % labels.len()] = true;
    }
    if shuffled == labels {
        return Err(invalid(
            "termination label shuffle did not change the subset",
        ));
    }
    Ok(shuffled)
}

#[derive(Clone)]
struct TerminationDecisionOutcome {
    terminal: bool,
    target: u32,
    chosen: u32,
    bos: crate::WriterTokenDiagnostic,
}

/// Evaluate the true held-out BOS objective through either the cached final
/// gain replay or a fresh complete forward.
pub fn evaluate_termination_gain_samples(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    labels: &[bool],
    cached: bool,
) -> Result<TerminationGainEvaluation> {
    validate_samples(artifact, samples)?;
    if labels.len() != decision_count(samples)? {
        return Err(invalid(
            "termination labels do not cover every training decision",
        ));
    }
    let outcomes = if cached {
        let trajectory = CpuGainTrajectory::build(artifact, samples)?;
        trajectory.termination_outcomes(artifact, labels)?
    } else {
        termination_outcomes_full(artifact, samples, labels)?
    };
    summarize_termination(artifact, samples.len(), &outcomes)
}

fn decision_count(samples: &[MemoryWeightSample]) -> Result<usize> {
    samples.iter().try_fold(0usize, |total, sample| {
        let tokens = sample
            .prompt
            .len()
            .checked_add(sample.reply.len())
            .ok_or_else(|| invalid("termination sample token count overflows"))?;
        if tokens < 2 {
            return Err(invalid("termination sample has no decision"));
        }
        total
            .checked_add(tokens - 1)
            .ok_or_else(|| invalid("termination decision count overflows"))
    })
}

fn terminate_full_sample(
    artifact: &[u8],
    sample: &MemoryWeightSample,
) -> Result<Vec<TerminationDecisionOutcome>> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("termination full forward: {error}")))?;
    let tokens = sample
        .prompt
        .iter()
        .chain(&sample.reply)
        .copied()
        .collect::<Vec<_>>();
    let mut outcomes = Vec::with_capacity(tokens.len() - 1);
    for index in 0..tokens.len() - 1 {
        if index + 1 == sample.prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(tokens[index], CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("termination full forward: {error}")))?;
        let bos = forward
            .writer_token_diagnostic(&step.query_q12, POINT_KEY_BOS)
            .map_err(|error| invalid(format!("termination BOS diagnostic: {error}")))?;
        outcomes.push(TerminationDecisionOutcome {
            terminal: false,
            target: tokens[index + 1],
            chosen: step.token,
            bos,
        });
    }
    Ok(outcomes)
}

fn termination_outcomes_full(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    labels: &[bool],
) -> Result<Vec<TerminationDecisionOutcome>> {
    let mut outcomes = samples
        .iter()
        .map(|sample| terminate_full_sample(artifact, sample))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if outcomes.len() != labels.len() {
        return Err(invalid("termination full outcomes differ from labels"));
    }
    for (outcome, &terminal) in outcomes.iter_mut().zip(labels) {
        outcome.terminal = terminal;
    }
    Ok(outcomes)
}

fn summarize_termination(
    artifact: &[u8],
    samples: usize,
    outcomes: &[TerminationDecisionOutcome],
) -> Result<TerminationGainEvaluation> {
    let mut reading = TerminationGainReading {
        artifact_sha256: uor_r4_discrete::corpus::sha256(artifact),
        samples,
        decisions: 0,
        terminal_decisions: 0,
        nonterminal_decisions: 0,
        terminal_bos_top1: 0,
        terminal_bos_admitted: 0,
        terminal_bos_top8: 0,
        terminal_bos_rank_sum: 0,
        terminal_bos_distance_sum: 0,
        terminal_content_top1: 0,
        nonterminal_bos_top1: 0,
        nonterminal_content_top1: 0,
        nonterminal_bos_distance_sum: 0,
    };
    let mut terminal_rows = Vec::new();
    for outcome in outcomes {
        reading.decisions = checked_increment(reading.decisions, "termination decision count")?;
        let distance = u64::try_from(outcome.bos.target_distance)
            .map_err(|_| invalid("termination BOS distance is negative"))?;
        if outcome.terminal {
            reading.terminal_decisions =
                checked_increment(reading.terminal_decisions, "terminal decision count")?;
            reading.terminal_bos_top1 += u64::from(outcome.chosen == POINT_KEY_BOS);
            reading.terminal_bos_admitted += u64::from(outcome.bos.admitted);
            reading.terminal_bos_top8 += u64::from(outcome.bos.top8);
            reading.terminal_bos_rank_sum = reading
                .terminal_bos_rank_sum
                .checked_add(
                    outcome
                        .bos
                        .rank
                        .and_then(|rank| u64::try_from(rank).ok())
                        .unwrap_or(crate::VOCABULARY as u64 + 1),
                )
                .ok_or_else(|| invalid("terminal BOS rank sum overflows"))?;
            reading.terminal_bos_distance_sum = reading
                .terminal_bos_distance_sum
                .checked_add(distance)
                .ok_or_else(|| invalid("terminal BOS distance sum overflows"))?;
            reading.terminal_content_top1 += u64::from(outcome.chosen == outcome.target);
            terminal_rows.push(outcome.chosen == POINT_KEY_BOS);
        } else {
            reading.nonterminal_decisions =
                checked_increment(reading.nonterminal_decisions, "nonterminal decision count")?;
            reading.nonterminal_bos_top1 += u64::from(outcome.chosen == POINT_KEY_BOS);
            reading.nonterminal_content_top1 += u64::from(outcome.chosen == outcome.target);
            reading.nonterminal_bos_distance_sum = reading
                .nonterminal_bos_distance_sum
                .checked_add(distance)
                .ok_or_else(|| invalid("nonterminal BOS distance sum overflows"))?;
        }
    }
    if reading.terminal_decisions == 0 || reading.nonterminal_decisions == 0 {
        return Err(invalid(
            "termination evaluation needs both terminal and nonterminal decisions",
        ));
    }
    Ok(TerminationGainEvaluation {
        reading,
        terminal_rows,
    })
}

fn checked_increment(value: u64, name: &str) -> Result<u64> {
    value
        .checked_add(1)
        .ok_or_else(|| invalid(format!("{name} overflows")))
}

#[derive(Clone)]
pub struct CpuTerminationGainTrainer {
    initial_artifact: Vec<u8>,
    artifact: Vec<u8>,
    samples: Vec<MemoryWeightSample>,
    labels: Vec<bool>,
    config: TerminationGainConfig,
    steps: u64,
    trajectory: Option<CpuGainTrajectory>,
}

impl std::fmt::Debug for CpuTerminationGainTrainer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CpuTerminationGainTrainer")
            .field("artifact_bytes", &self.artifact.len())
            .field("samples", &self.samples.len())
            .field("decisions", &self.labels.len())
            .field("step_code", &self.config.step_code)
            .field("steps", &self.steps)
            .field("cached_trajectory", &self.trajectory.is_some())
            .finish()
    }
}

impl CpuTerminationGainTrainer {
    pub fn from_artifact(
        artifact: &[u8],
        samples: Vec<MemoryWeightSample>,
        labels: Vec<bool>,
        config: TerminationGainConfig,
    ) -> Result<Self> {
        if config.step_code == 0 {
            return Err(invalid("termination-gain step code must be nonzero"));
        }
        validate_samples(artifact, &samples)?;
        validate_artifact(artifact)?;
        if labels.len() != decision_count(&samples)? {
            return Err(invalid(
                "termination-gain labels do not cover every decision",
            ));
        }
        if labels.iter().all(|terminal| *terminal) || labels.iter().all(|terminal| !*terminal) {
            return Err(invalid(
                "termination-gain training needs both terminal and nonterminal decisions",
            ));
        }
        let reading =
            evaluate_termination_gain_samples(artifact, &samples, &labels, false)?.reading;
        if reading.decisions == 0 {
            return Err(invalid("termination-gain samples contain no decision"));
        }
        Ok(Self {
            initial_artifact: artifact.to_vec(),
            artifact: artifact.to_vec(),
            samples,
            labels,
            config,
            steps: 0,
            trajectory: None,
        })
    }

    pub fn from_artifact_cached(
        artifact: &[u8],
        samples: Vec<MemoryWeightSample>,
        labels: Vec<bool>,
        config: TerminationGainConfig,
    ) -> Result<Self> {
        let mut trainer = Self::from_artifact(artifact, samples, labels, config)?;
        trainer.trajectory = Some(CpuGainTrajectory::build(artifact, &trainer.samples)?);
        Ok(trainer)
    }

    pub fn artifact(&self) -> &[u8] {
        &self.artifact
    }

    pub fn initial_artifact(&self) -> &[u8] {
        &self.initial_artifact
    }

    pub fn labels(&self) -> &[bool] {
        &self.labels
    }

    pub fn config(&self) -> TerminationGainConfig {
        self.config
    }

    pub fn step_count(&self) -> u64 {
        self.steps
    }

    pub fn reading(&self) -> Result<TerminationGainEvaluation> {
        self.evaluate_artifact(&self.artifact)
    }

    pub fn train_step(&mut self) -> Result<TerminationGainStep> {
        let index = self.steps;
        let before = self.reading()?.reading;
        let gain = read_gain(&self.artifact)?;
        let mut candidates = Vec::with_capacity(gain.len() * 2);
        for (coordinate, &original) in gain.iter().enumerate() {
            for delta in [self.config.step_code, -self.config.step_code] {
                let Some(after) = original.checked_add(delta) else {
                    continue;
                };
                if after != original {
                    candidates.push((coordinate, original, after));
                }
            }
        }
        let evaluated = candidates
            .par_chunks(16)
            .map(|chunk| {
                let mut best: Option<TerminationGainCandidate> = None;
                for &(coordinate, original, after) in chunk {
                    let mut trial = gain.clone();
                    trial[coordinate] = after;
                    let artifact = patch_gain(&self.artifact, &trial)?;
                    let reading = self.evaluate_artifact(&artifact)?.reading;
                    if !termination_candidate_safe(&before, &reading)
                        || !termination_improves(&reading, &before)
                    {
                        continue;
                    }
                    let candidate = TerminationGainCandidate {
                        coordinate,
                        before_code: original,
                        after_code: after,
                        artifact,
                        reading,
                    };
                    if best.as_ref().is_none_or(|current| {
                        termination_improves(&candidate.reading, &current.reading)
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
                if termination_improves(&candidate.reading, &current.reading) {
                    candidate
                } else {
                    current
                }
            });
        self.steps = self
            .steps
            .checked_add(1)
            .ok_or_else(|| invalid("termination-gain step counter exhausted"))?;
        let Some(best) = best else {
            return Ok(TerminationGainStep {
                index,
                updated: false,
                coordinate: None,
                before_code: None,
                after_code: None,
                before: before.clone(),
                after: before,
                parity: None,
            });
        };
        let parity = audit_memory_weight_samples(&best.artifact, &self.samples, true)?;
        if !parity.is_exact() {
            return Err(invalid(
                "termination-gain update lost complete CPU/serving parity",
            ));
        }
        if best.reading.artifact_sha256 == before.artifact_sha256 {
            return Err(invalid(
                "termination-gain accepted move did not change the artifact",
            ));
        }
        self.artifact = best.artifact;
        Ok(TerminationGainStep {
            index,
            updated: true,
            coordinate: Some(best.coordinate),
            before_code: Some(best.before_code),
            after_code: Some(best.after_code),
            before,
            after: best.reading,
            parity: Some(parity),
        })
    }

    fn evaluate_artifact(&self, artifact: &[u8]) -> Result<TerminationGainEvaluation> {
        let Some(trajectory) = &self.trajectory else {
            return evaluate_termination_gain_samples(artifact, &self.samples, &self.labels, false);
        };
        let outcomes = trajectory.termination_outcomes(artifact, &self.labels)?;
        summarize_termination(artifact, self.samples.len(), &outcomes)
    }
}

struct TerminationGainCandidate {
    coordinate: usize,
    before_code: i16,
    after_code: i16,
    artifact: Vec<u8>,
    reading: TerminationGainReading,
}

fn termination_candidate_safe(
    before: &TerminationGainReading,
    after: &TerminationGainReading,
) -> bool {
    after.nonterminal_bos_top1 <= before.nonterminal_bos_top1
        && after.nonterminal_content_top1 >= before.nonterminal_content_top1
}

fn termination_improves(after: &TerminationGainReading, before: &TerminationGainReading) -> bool {
    let ordered = [
        after.terminal_bos_top1.cmp(&before.terminal_bos_top1),
        after
            .terminal_bos_admitted
            .cmp(&before.terminal_bos_admitted),
        after.terminal_bos_top8.cmp(&before.terminal_bos_top8),
        before
            .terminal_bos_rank_sum
            .cmp(&after.terminal_bos_rank_sum),
        before
            .terminal_bos_distance_sum
            .cmp(&after.terminal_bos_distance_sum),
        after
            .nonterminal_content_top1
            .cmp(&before.nonterminal_content_top1),
    ];
    for improvement in ordered {
        if improvement.is_gt() {
            return true;
        }
        if improvement.is_lt() {
            return false;
        }
    }
    false
}

struct GainCandidate {
    coordinate: usize,
    before_code: i16,
    after_code: i16,
    artifact: Vec<u8>,
    reading: MemoryWeightReading,
}

#[derive(Clone)]
struct CachedGainDecision {
    target: u32,
    norm_before_final_gain: Vec<i64>,
    memory: Option<crate::memory::MemoryMove>,
}

#[derive(Clone)]
struct CpuGainTrajectory {
    forward: CpuFbsForward,
    decisions: Vec<CachedGainDecision>,
    sample_ranges: Vec<std::ops::Range<usize>>,
    opening_weights: Vec<i32>,
    reply_weights: Option<Vec<i32>>,
    samples: usize,
}

impl CpuGainTrajectory {
    fn build(artifact: &[u8], samples: &[MemoryWeightSample]) -> Result<Self> {
        let forward = CpuFbsForward::from_artifact(artifact)
            .map_err(|error| invalid(format!("served-gain forward: {error}")))?;
        if !forward.supports_gain_replay() {
            return Err(invalid(
                "served-gain replay does not yet support a Hamming readout",
            ));
        }
        let memory = read_memory_uorlut01(artifact)
            .map_err(|error| invalid(format!("served-gain memory: {error}")))?
            .ok_or_else(|| invalid("served-gain training requires turn memory"))?;
        let opening_weights = memory.opening.weight;
        let reply_weights = memory.reply.map(|reply| reply.weight);
        let sample_decisions = samples
            .par_iter()
            .map(|sample| {
                let mut forward = CpuFbsForward::from_artifact(artifact)
                    .map_err(|error| invalid(format!("served-gain forward: {error}")))?;
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
                        _,
                        CpuServedTrace {
                            norm_before_final_gain,
                            norm_before_memory: _,
                            memory,
                        },
                    ) = forward
                        .step_with_trace(tokens[index], CpuFbsHead::Writer)
                        .map_err(|error| invalid(format!("served-gain forward: {error}")))?;
                    decisions.push(CachedGainDecision {
                        target: tokens[index + 1],
                        norm_before_final_gain,
                        memory,
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
            opening_weights,
            reply_weights,
            samples: samples.len(),
        })
    }

    fn evaluate(&self, artifact: &[u8]) -> Result<MemoryWeightReading> {
        let gain = read_gain(artifact)?;
        let width = self.forward.width();
        let vocab = self.forward.vocabulary();
        let table = self.forward.token_table();
        let readings =
            self.sample_ranges
                .iter()
                .map(|range| {
                    let mut sample_top1 = 0u64;
                    let mut sample_distance = 0u64;
                    let mut sample_decisions = 0u64;
                    for decision in &self.decisions[range.clone()] {
                        if decision.target as usize >= vocab {
                            return Err(invalid("served-gain target is outside vocabulary"));
                        }
                        let (query, token) = self
                            .forward
                            .replay_final_gain(
                                &decision.norm_before_final_gain,
                                &gain,
                                decision.memory,
                                &self.opening_weights,
                                self.reply_weights.as_deref(),
                            )
                            .map_err(|error| invalid(format!("served-gain replay: {error}")))?;
                        let target_row = &table[decision.target as usize * width..][..width];
                        let distance = query.iter().zip(target_row).try_fold(
                            0u64,
                            |sum, (&query, &point)| {
                                sum.checked_add(u64::from(query.abs_diff(i32::from(point))))
                                    .ok_or_else(|| {
                                        invalid("served-gain decision distance overflows")
                                    })
                            },
                        )?;
                        sample_top1 = sample_top1
                            .checked_add(u64::from(token == decision.target))
                            .ok_or_else(|| invalid("served-gain top-1 count overflows"))?;
                        sample_distance = sample_distance
                            .checked_add(distance)
                            .ok_or_else(|| invalid("served-gain distance sum overflows"))?;
                        sample_decisions = sample_decisions
                            .checked_add(1)
                            .ok_or_else(|| invalid("served-gain decision count overflows"))?;
                    }
                    Ok((sample_top1, sample_distance, sample_decisions))
                })
                .collect::<Result<Vec<_>>>()?;
        let (top1_correct, target_distance_sum, decisions) = checked_sample_totals(readings)?;
        Ok(MemoryWeightReading {
            artifact_sha256: uor_r4_discrete::corpus::sha256(artifact),
            samples: self.samples,
            decisions,
            top1_correct,
            target_distance_sum,
        })
    }

    fn termination_outcomes(
        &self,
        artifact: &[u8],
        labels: &[bool],
    ) -> Result<Vec<TerminationDecisionOutcome>> {
        if labels.len() != self.decisions.len() {
            return Err(invalid(
                "termination labels do not cover every cached decision",
            ));
        }
        let gain = read_gain(artifact)?;
        self.decisions
            .iter()
            .zip(labels)
            .map(|(decision, &terminal)| {
                let (query, chosen) = self
                    .forward
                    .replay_final_gain(
                        &decision.norm_before_final_gain,
                        &gain,
                        decision.memory,
                        &self.opening_weights,
                        self.reply_weights.as_deref(),
                    )
                    .map_err(|error| invalid(format!("termination gain replay: {error}")))?;
                let bos = self
                    .forward
                    .writer_token_diagnostic(&query, POINT_KEY_BOS)
                    .map_err(|error| invalid(format!("termination BOS diagnostic: {error}")))?;
                Ok(TerminationDecisionOutcome {
                    terminal,
                    target: decision.target,
                    chosen,
                    bos,
                })
            })
            .collect()
    }
}

fn evaluate(artifact: &[u8], samples: &[MemoryWeightSample]) -> Result<MemoryWeightReading> {
    crate::evaluate_memory_weight_samples(artifact, samples, true)
}

fn strict_improvement(after: &MemoryWeightReading, before: &MemoryWeightReading) -> bool {
    after.target_distance_sum < before.target_distance_sum
        || (after.target_distance_sum == before.target_distance_sum
            && after.top1_correct > before.top1_correct)
}

fn validate_artifact(artifact: &[u8]) -> Result<()> {
    let forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("served-gain forward: {error}")))?;
    if !forward.supports_gain_replay() {
        return Err(invalid(
            "served-gain replay does not yet support a Hamming readout",
        ));
    }
    if read_memory_uorlut01(artifact)
        .map_err(|error| invalid(format!("served-gain memory: {error}")))?
        .is_none()
    {
        return Err(invalid("served-gain training requires turn memory"));
    }
    let _ = read_gain(artifact)?;
    Ok(())
}

pub fn read_gain(artifact: &[u8]) -> Result<Vec<i16>> {
    let model = IntegerStackModel::parse(artifact)
        .map_err(|error| invalid(format!("served-gain artifact: {error}")))?;
    let image = model
        .fbs_trunk_image()
        .ok_or_else(|| invalid("served-gain artifact has no FBS trunk"))?;
    Ok(image.final_gain.to_vec())
}

pub fn replace_final_gain(artifact: &[u8], gain: &[i16]) -> Result<Vec<u8>> {
    patch_gain(artifact, gain)
}

pub fn random_gain_control(
    artifact: &[u8],
    moves: usize,
    seed: u64,
) -> Result<(Vec<u8>, Vec<ServedGainMove>)> {
    let mut gain = read_gain(artifact)?;
    if gain.is_empty() {
        return Err(invalid("final gain is empty"));
    }
    let mut state = seed.max(1);
    let mut records = Vec::with_capacity(moves);
    for _ in 0..moves {
        let mut moved = None;
        for _ in 0..gain.len().saturating_mul(2).max(1) {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let coordinate = (state as usize) % gain.len();
            let delta = if state >> 63 == 0 { 1i16 } else { -1i16 };
            let before = gain[coordinate];
            let after = before.checked_add(delta);
            if let Some(after) = after {
                if after != before {
                    gain[coordinate] = after;
                    moved = Some(ServedGainMove {
                        coordinate,
                        before_code: before,
                        after_code: after,
                    });
                    break;
                }
            }
        }
        records.push(
            moved.ok_or_else(|| invalid("random served-gain control found no movable code"))?,
        );
    }
    Ok((patch_gain(artifact, &gain)?, records))
}

fn patch_gain(artifact: &[u8], gain: &[i16]) -> Result<Vec<u8>> {
    if artifact.len() < 16 || &artifact[..8] != b"UORLUT01" {
        return Err(invalid("served-gain artifact magic differs"));
    }
    let header_len = u64::from_le_bytes(
        artifact[8..16]
            .try_into()
            .map_err(|_| invalid("served-gain header length differs"))?,
    );
    let header_end = 16usize
        .checked_add(
            usize::try_from(header_len).map_err(|_| invalid("served-gain header is too large"))?,
        )
        .filter(|&end| end <= artifact.len())
        .ok_or_else(|| invalid("served-gain header is outside the file"))?;
    let data_start = header_end.div_ceil(64) * 64;
    if data_start > artifact.len() {
        return Err(invalid("served-gain data start is outside the file"));
    }
    let header: serde_json::Value = serde_json::from_slice(&artifact[16..header_end])
        .map_err(|error| invalid(format!("served-gain header: {error}")))?;
    let entry = header["tables"]
        .as_array()
        .and_then(|tables| {
            tables
                .iter()
                .find(|entry| entry["name"].as_str() == Some("final_norm_gain"))
        })
        .ok_or_else(|| invalid("served-gain artifact has no final_norm_gain table"))?;
    if entry["kind"].as_str() != Some("i16") || entry["len"].as_u64() != Some(gain.len() as u64) {
        return Err(invalid("served-gain final_norm_gain shape differs"));
    }
    let offset = usize::try_from(
        entry["span"]["offset"]
            .as_u64()
            .ok_or_else(|| invalid("served-gain final gain offset is missing"))?,
    )
    .map_err(|_| invalid("served-gain final gain offset exceeds usize"))?;
    let bytes = usize::try_from(
        entry["span"]["bytes"]
            .as_u64()
            .ok_or_else(|| invalid("served-gain final gain bytes are missing"))?,
    )
    .map_err(|_| invalid("served-gain final gain bytes exceed usize"))?;
    if bytes != gain.len() * 2 {
        return Err(invalid("served-gain final gain byte length differs"));
    }
    let start = data_start
        .checked_add(offset)
        .ok_or_else(|| invalid("served-gain final gain offset overflows"))?;
    let end = start
        .checked_add(bytes)
        .filter(|&end| end <= artifact.len())
        .ok_or_else(|| invalid("served-gain final gain is outside the file"))?;
    let mut out = artifact.to_vec();
    for (slot, &value) in out[start..end].chunks_exact_mut(2).zip(gain) {
        slot.copy_from_slice(&value.to_le_bytes());
    }
    IntegerStackModel::parse(&out)
        .map_err(|error| invalid(format!("patched served-gain artifact: {error}")))?;
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

    fn fixture() -> Vec<u8> {
        fixture_with_hamming(false)
    }

    fn fixture_with_hamming(hamming: bool) -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            947,
            FbsSynth {
                vocab: crate::VOCABULARY,
                d: WIDTH,
                mlp: 64,
                layers: 2,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.25,
                decay_step: 0.0625,
            },
        );
        if hamming {
            snapshot.hamming = Some(uor_r4_integer::stack::synthetic::synthetic_hamming(
                snapshot.width,
                snapshot.layers.len(),
                4,
                947,
            ));
        }
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 3,
            key_window: 16,
            opening: vec![FbsMemoryEntry {
                key: vec![10, 11, 12],
                value: 42,
            }],
            opening_weight: vec![0.125; WIDTH],
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
                weight: vec![0.25; WIDTH],
            }),
        });
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn samples() -> Vec<MemoryWeightSample> {
        vec![
            MemoryWeightSample {
                prompt: vec![10, 11, 12],
                reply: vec![42, 77, 99],
            },
            MemoryWeightSample {
                prompt: vec![20, 21, 22],
                reply: vec![50, 51],
            },
        ]
    }

    fn termination_samples() -> Vec<MemoryWeightSample> {
        vec![
            MemoryWeightSample {
                prompt: vec![10, 11, 12],
                reply: vec![42, 77, CORPUS_EOS],
            },
            MemoryWeightSample {
                prompt: vec![20, 21, 22],
                reply: vec![50, CORPUS_EOS],
            },
            MemoryWeightSample {
                prompt: vec![30, 31, 32],
                reply: vec![60, 61, 62, CORPUS_EOS],
            },
        ]
    }

    fn termination_reading() -> TerminationGainReading {
        TerminationGainReading {
            artifact_sha256: "00".repeat(32),
            samples: 1,
            decisions: 10,
            terminal_decisions: 1,
            nonterminal_decisions: 9,
            terminal_bos_top1: 0,
            terminal_bos_admitted: 0,
            terminal_bos_top8: 0,
            terminal_bos_rank_sum: 20,
            terminal_bos_distance_sum: 100,
            terminal_content_top1: 0,
            nonterminal_bos_top1: 0,
            nonterminal_content_top1: 8,
            nonterminal_bos_distance_sum: 100,
        }
    }

    #[test]
    fn termination_labels_require_one_final_eos_and_keep_other_content() {
        let samples = termination_samples();
        let labels = termination_gain_labels(&samples).unwrap();
        assert_eq!(labels.len(), 15);
        assert_eq!(
            labels.iter().filter(|terminal| **terminal).count(),
            samples.len()
        );
        assert_eq!(
            labels,
            vec![
                false, false, false, false, true, false, false, false, true, false, false, false,
                false, false, true,
            ]
        );

        let mut no_eos = samples;
        no_eos[1].reply.pop();
        assert!(termination_gain_labels(&no_eos).is_err());

        let mut internal_eos = termination_samples();
        internal_eos[0].reply = vec![42, CORPUS_EOS, 77, CORPUS_EOS];
        assert!(termination_gain_labels(&internal_eos).is_err());
    }

    #[test]
    fn termination_label_shuffle_preserves_counts_and_changes_the_subset() {
        let labels = termination_gain_labels(&termination_samples()).unwrap();
        let shuffled = shuffle_termination_gain_labels(&labels, 17).unwrap();
        assert_eq!(
            shuffled.iter().filter(|terminal| **terminal).count(),
            labels.iter().filter(|terminal| **terminal).count()
        );
        assert_ne!(shuffled, labels);
        assert_eq!(
            shuffled,
            shuffle_termination_gain_labels(&labels, 17).unwrap()
        );
        assert!(shuffle_termination_gain_labels(&[true, true], 1).is_err());
    }

    #[test]
    fn cached_termination_evaluation_matches_the_full_forward() {
        let artifact = fixture();
        let samples = termination_samples();
        let labels = termination_gain_labels(&samples).unwrap();
        let full = evaluate_termination_gain_samples(&artifact, &samples, &labels, false).unwrap();
        let cached = evaluate_termination_gain_samples(&artifact, &samples, &labels, true).unwrap();
        assert_eq!(cached, full);
        assert_eq!(cached.reading.terminal_decisions, 3);
        assert_eq!(cached.terminal_rows.len(), 3);
    }

    #[test]
    fn termination_trainer_cached_and_full_steps_agree() {
        let artifact = fixture();
        let samples = termination_samples();
        let labels = termination_gain_labels(&samples).unwrap();
        let config = TerminationGainConfig::default();
        let mut full = CpuTerminationGainTrainer::from_artifact(
            &artifact,
            samples.clone(),
            labels.clone(),
            config,
        )
        .unwrap();
        let mut cached =
            CpuTerminationGainTrainer::from_artifact_cached(&artifact, samples, labels, config)
                .unwrap();
        assert_eq!(cached.reading().unwrap(), full.reading().unwrap());
        assert_eq!(cached.train_step().unwrap(), full.train_step().unwrap());
        assert_eq!(cached.reading().unwrap(), full.reading().unwrap());
        assert_eq!(cached.artifact(), full.artifact());
    }

    #[test]
    fn termination_trainer_refuses_missing_classes_and_hamming() {
        let samples = termination_samples();
        let labels = termination_gain_labels(&samples).unwrap();
        assert!(CpuTerminationGainTrainer::from_artifact_cached(
            &fixture(),
            samples.clone(),
            vec![false; labels.len()],
            TerminationGainConfig::default(),
        )
        .is_err());
        assert!(CpuTerminationGainTrainer::from_artifact_cached(
            &fixture_with_hamming(true),
            samples,
            labels,
            TerminationGainConfig::default(),
        )
        .is_err());
    }

    #[test]
    fn termination_objective_is_lexicographic_and_protects_content() {
        let before = termination_reading();
        let mut admitted = before.clone();
        admitted.terminal_bos_admitted = 1;
        assert!(termination_candidate_safe(&before, &admitted));
        assert!(termination_improves(&admitted, &before));

        let mut unsafe_candidate = admitted.clone();
        unsafe_candidate.nonterminal_content_top1 -= 1;
        assert!(!termination_candidate_safe(&before, &unsafe_candidate));

        let mut false_bos = admitted;
        false_bos.nonterminal_bos_top1 = 1;
        assert!(!termination_candidate_safe(&before, &false_bos));
    }

    fn assert_cached_matches_full(artifact: &[u8], steps: usize) {
        let config = ServedGainConfig::default();
        let mut full = CpuServedGainTrainer::from_artifact(artifact, samples(), config).unwrap();
        let mut cached =
            CpuServedGainTrainer::from_artifact_cached(artifact, samples(), config).unwrap();
        for _ in 0..steps {
            assert_eq!(cached.train_step().unwrap(), full.train_step().unwrap());
            assert_eq!(cached.reading().unwrap(), full.reading().unwrap());
        }
        assert_eq!(cached.artifact(), full.artifact());
    }

    #[test]
    fn cached_gain_replay_matches_the_full_forward() {
        assert_cached_matches_full(&fixture(), 3);
    }

    #[test]
    fn cached_gain_replay_matches_point_fallback() {
        let artifact = fixture();
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
        let artifact = add_point_key_uorlut01(
            &artifact,
            &PointKeyImage {
                width: WIDTH,
                points: query
                    .expect("the prompt has a final query")
                    .into_iter()
                    .map(|value| i16::try_from(value).expect("the query is Q12-clamped"))
                    .collect(),
                targets: vec![40],
                vote_k: 1,
            },
        )
        .unwrap();
        let mut point = CpuFbsForward::from_artifact(&artifact).unwrap();
        let mut token = None;
        for (index, input) in [30, 31, 32].into_iter().enumerate() {
            if index + 1 == 3 {
                point.mark_reply_start();
            }
            token = Some(point.step(input, CpuFbsHead::Writer).unwrap().token);
        }
        assert_eq!(token, Some(40));

        let point_samples = vec![
            samples()[0].clone(),
            MemoryWeightSample {
                prompt: vec![30, 31, 32],
                reply: vec![40],
            },
        ];
        let config = ServedGainConfig::default();
        let mut full =
            CpuServedGainTrainer::from_artifact(&artifact, point_samples.clone(), config).unwrap();
        let mut cached =
            CpuServedGainTrainer::from_artifact_cached(&artifact, point_samples, config).unwrap();
        for _ in 0..3 {
            assert_eq!(cached.train_step().unwrap(), full.train_step().unwrap());
        }
        assert_eq!(cached.artifact(), full.artifact());
    }

    #[test]
    fn final_gain_patch_changes_only_its_named_table() {
        let artifact = fixture();
        let mut gain = read_gain(&artifact).unwrap();
        gain[7] = gain[7].saturating_add(1);
        let patched = replace_final_gain(&artifact, &gain).unwrap();
        let changed = artifact
            .iter()
            .zip(&patched)
            .filter(|(left, right)| left != right)
            .count();
        assert_eq!(changed, 1);
    }

    #[test]
    fn random_gain_control_moves_exactly_the_requested_count() {
        let artifact = fixture();
        let original = read_gain(&artifact).unwrap();
        let (patched, moves) = random_gain_control(&artifact, 8, 1).unwrap();
        assert_eq!(moves.len(), 8);
        assert!(moves
            .iter()
            .all(|movement| movement.before_code != movement.after_code));
        assert_ne!(read_gain(&patched).unwrap(), original);
    }

    #[test]
    fn the_gain_trainer_refuses_a_hamming_readout() {
        let with_hamming = fixture_with_hamming(true);
        let error = CpuServedGainTrainer::from_artifact_cached(
            &with_hamming,
            samples(),
            ServedGainConfig::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("Hamming"));
    }
}
