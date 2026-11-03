//! Exact full-path coordinate training of turn-memory movement weights.
//!
//! A step patches one exported Q24 memory-weight coordinate, reparses the whole
//! artifact, and runs the complete teacher-forced FBS path with opening and
//! reply rows active. The accepted direction is the one that improves served
//! top-1 first and then lowers the L1 distance from each complete query to its
//! actual next-token writer row.

use uor_r4_integer::stack::{read_memory_uorlut01, IntegerStackModel};

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

#[derive(Clone, Debug)]
pub struct CpuMemoryWeightTrainer {
    initial_artifact: Vec<u8>,
    artifact: Vec<u8>,
    samples: Vec<MemoryWeightSample>,
    config: MemoryWeightConfig,
    steps: u64,
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
        })
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
        evaluate(&self.artifact, &self.samples)
    }

    pub fn train_step(&mut self) -> Result<MemoryWeightStep> {
        let index = self.steps;
        let before = self.reading()?;
        let memory = read_memory_uorlut01(&self.artifact)
            .map_err(|error| invalid(format!("memory-weight artifact: {error}")))?
            .ok_or_else(|| invalid("memory-weight training lost its turn memory"))?;
        let mut candidates = Vec::new();
        if self.config.train_opening {
            candidates.push((MemoryWeightTable::Opening, memory.opening.weight));
        }
        if self.config.train_reply {
            if let Some(reply) = memory.reply {
                candidates.push((MemoryWeightTable::Reply, reply.weight));
            }
        }
        let mut best: Option<WeightCandidate> = None;
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
                    let mut trial_weights = weights.clone();
                    trial_weights[coordinate] = proposed;
                    let artifact =
                        patch_memory_weight_table(&self.artifact, table, &trial_weights)?;
                    let reading = evaluate(&artifact, &self.samples)?;
                    if !strict_improvement(&reading, &before) {
                        continue;
                    }
                    let candidate = WeightCandidate {
                        table,
                        coordinate,
                        before_weight: original,
                        after_weight: proposed,
                        artifact,
                        reading,
                    };
                    if best.as_ref().is_none_or(|current| {
                        strict_improvement(&candidate.reading, &current.reading)
                    }) {
                        best = Some(candidate);
                    }
                }
            }
        }
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
        let parity = exact_parity(&best.artifact, &self.samples)?;
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

fn validate_samples(artifact: &[u8], samples: &[MemoryWeightSample]) -> Result<()> {
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

fn evaluate(artifact: &[u8], samples: &[MemoryWeightSample]) -> Result<MemoryWeightReading> {
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
    let mut top1_correct = 0u64;
    let mut target_distance_sum = 0u64;
    let mut decisions = 0u64;
    for sample in samples {
        let mut cpu = CpuFbsForward::from_artifact(artifact)
            .map_err(|error| invalid(format!("memory-weight CPU forward: {error}")))?;
        let tokens = sample
            .prompt
            .iter()
            .chain(&sample.reply)
            .copied()
            .collect::<Vec<_>>();
        for index in 0..tokens.len() - 1 {
            if index + 1 == sample.prompt.len() {
                cpu.mark_reply_start();
            }
            let step = cpu
                .step(tokens[index], CpuFbsHead::Writer)
                .map_err(|error| invalid(format!("memory-weight forward: {error}")))?;
            let target = tokens[index + 1];
            let row = &table[target as usize * width..][..width];
            let distance = step
                .query_q12
                .iter()
                .zip(row)
                .fold(0u64, |sum, (&query, &point)| {
                    sum.saturating_add(u64::from(query.abs_diff(i32::from(point))))
                });
            top1_correct += u64::from(step.token == target);
            target_distance_sum = target_distance_sum.saturating_add(distance);
            decisions = decisions
                .checked_add(1)
                .ok_or_else(|| invalid("memory-weight decision count overflows"))?;
        }
    }
    Ok(MemoryWeightReading {
        artifact_sha256: uor_r4_discrete::corpus::sha256(artifact),
        samples: samples.len(),
        decisions,
        top1_correct,
        target_distance_sum,
    })
}

fn exact_parity(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
) -> Result<MemoryWeightParityReading> {
    let mut total = MemoryWeightParityReading::default();
    for sample in samples {
        let tokens = sample
            .prompt
            .iter()
            .chain(&sample.reply)
            .copied()
            .collect::<Vec<_>>();
        let audit = audit_served_forward(
            artifact,
            &ServedForwardAuditConfig {
                tokens,
                token_source: None,
                expected_artifact_sha256: None,
                reply_start: Some(sample.prompt.len() - 1),
                measure_timing: false,
            },
        )
        .map_err(|error| invalid(format!("memory-weight served audit: {error}")))?;
        let parity = audit.parity;
        total.samples += 1;
        total.positions = total
            .positions
            .saturating_add(parity.positions_checked as u64);
        total.query_coordinates = total
            .query_coordinates
            .saturating_add(parity.query_coordinates_compared);
        total.query_coordinate_mismatches = total
            .query_coordinate_mismatches
            .saturating_add(parity.query_coordinate_mismatches);
        total.token_decisions = total
            .token_decisions
            .saturating_add(parity.token_decisions_compared);
        total.token_mismatches = total
            .token_mismatches
            .saturating_add(parity.token_mismatches);
        total.kept_cells = total.kept_cells.saturating_add(parity.kept_cells_compared);
        total.kept_cell_mismatches = total
            .kept_cell_mismatches
            .saturating_add(parity.kept_cell_mismatches);
        if parity.instrument_fault {
            return Err(invalid(
                "memory-weight served audit reports a CPU/serving instrument fault",
            ));
        }
    }
    Ok(total)
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
        export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot, FbsReplyTable, MEMORY_PAD,
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

    fn samples() -> Vec<MemoryWeightSample> {
        vec![MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, 77, 99],
        }]
    }

    fn assert_audit(artifact: &[u8]) {
        let parity = exact_parity(artifact, &samples()).unwrap();
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
        let unseen = vec![MemoryWeightSample {
            prompt: vec![20, 21, 22],
            reply: vec![50, 51, 52],
        }];
        let mut trainer =
            CpuMemoryWeightTrainer::from_artifact(&artifact, unseen, MemoryWeightConfig::default())
                .unwrap();
        let step = trainer.train_step().unwrap();
        assert!(!step.updated);
        assert_eq!(trainer.artifact(), artifact);
        assert!(step.parity.is_none());
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
}
