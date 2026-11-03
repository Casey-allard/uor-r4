//! Exact distance training of the final served FBS gain.
//!
//! The final gain changes every served query without changing the
//! teacher-forced recurrent trajectory. The cached path stores the exact
//! normalization immediately before that gain, then replays the gain, Q12
//! conversion, writer decision, turn-memory movement and point fallback.

use rayon::prelude::*;
use uor_r4_integer::stack::{read_memory_uorlut01, IntegerStackModel};

use crate::fbs::CpuServedTrace;
use crate::memory_train::{checked_sample_totals, validate_samples};
use crate::{
    audit_memory_weight_samples, invalid, CpuFbsForward, CpuFbsHead, CpuTrainError,
    MemoryWeightParityReading, MemoryWeightReading, MemoryWeightSample,
};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const CPU_SERVED_GAIN_TRAINER_SCHEMA: &str = "uor-r4.cpu-served-gain-trainer/1";

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
