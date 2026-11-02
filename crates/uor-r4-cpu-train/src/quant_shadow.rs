//! Quantization-aware CPU checkpoint and finite float-shadow comparison.
//!
//! The CPU trainer edits the exact Q12 integer artifact. This module keeps a
//! paired offline `FbsTrunkSnapshot` shadow in step with those accepted integer
//! coordinate moves, then measures what the continuous source and exact served
//! artifact do on one identical finite panel. It is component evidence only.

use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01, FbsReferenceSnaps, FbsTrunkSnapshot, FbsWriterOutput,
    IntegerStackModel,
};

use crate::panel::{PanelExportReading, PanelReading};
use crate::train::{
    sample_fingerprint, CpuRankMarginTrainer, RankMarginConfig, RankMarginSample, RankMarginStep,
};
use crate::{invalid, CpuTrainError, TOP_K};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const CPU_QUANTIZATION_CHECKPOINT_SCHEMA: &str = "uor-r4.cpu-quantization-checkpoint/1";
const CPU_SUBSTRATE: &str = "cpu-development-research";
const CPU_QUALIFICATION: &str = "gpu-required";
const Q12_SCALE: f64 = 4096.0;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpuFloatPanelReading {
    pub samples: usize,
    pub target_admitted: usize,
    pub top1_correct: usize,
    pub top8_correct: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpuPanelGap {
    pub query_coordinates_checked: u64,
    pub query_coordinate_mismatches: u64,
    pub token_mismatches: u64,
    pub kept_cells_checked: u64,
    pub kept_cell_mismatches: u64,
    pub target_admission_mismatches: u64,
    pub top1_mismatches: u64,
    pub top8_mismatches: u64,
}

impl CpuPanelGap {
    pub fn is_exact(&self) -> bool {
        self.query_coordinate_mismatches == 0
            && self.token_mismatches == 0
            && self.kept_cell_mismatches == 0
            && self.target_admission_mismatches == 0
            && self.top1_mismatches == 0
            && self.top8_mismatches == 0
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpuQuantizationPanelReading {
    pub sample_fingerprint: [u8; 32],
    pub exact: PanelReading,
    pub plain: CpuFloatPanelReading,
    pub quantized: CpuFloatPanelReading,
    pub plain_gap: CpuPanelGap,
    pub quantized_gap: CpuPanelGap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantizationGate {
    Accept,
    NoChange,
    RejectAdmission,
    RejectExportGap,
    ExportLimited,
}

pub fn quantization_gate(
    before: &CpuQuantizationPanelReading,
    after: &CpuQuantizationPanelReading,
) -> Result<QuantizationGate> {
    if before.sample_fingerprint != after.sample_fingerprint
        || before.exact.samples != after.exact.samples
        || before.exact.margin_q12 != after.exact.margin_q12
        || before.exact.export.query_coordinates_checked
            != after.exact.export.query_coordinates_checked
        || before.exact.export.kept_cells_checked != after.exact.export.kept_cells_checked
    {
        return Err(invalid(
            "quantization comparison panel, margin or export shape differs",
        ));
    }
    if after.exact.target_admitted < before.exact.target_admitted {
        return Ok(QuantizationGate::RejectAdmission);
    }
    if !before.exact.export.is_exact() || !after.exact.export.is_exact() {
        return Ok(QuantizationGate::RejectExportGap);
    }

    let admission_lost = after.plain.target_admitted > before.plain.target_admitted
        && after.exact.target_admitted <= before.exact.target_admitted;
    let top1_lost = after.plain.top1_correct > before.plain.top1_correct
        && after.exact.top1_correct <= before.exact.top1_correct;
    let top8_lost = after.plain.top8_correct > before.plain.top8_correct
        && after.exact.top8_correct <= before.exact.top8_correct;
    if admission_lost || top1_lost || top8_lost {
        return Ok(QuantizationGate::ExportLimited);
    }
    if after == before {
        return Ok(QuantizationGate::NoChange);
    }
    Ok(QuantizationGate::Accept)
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpuQuantizationCheckpoint {
    pub schema: String,
    pub substrate: String,
    pub qualification: String,
    pub source_snapshot_sha256: String,
    pub initial_artifact_sha256: String,
    pub artifact_sha256: String,
    pub shadow_sha256: String,
    pub trainer_checkpoint_sha256: String,
    pub config: RankMarginConfig,
    pub training_samples: Vec<RankMarginSample>,
    pub panel_samples: Vec<RankMarginSample>,
    pub trainer_checkpoint: Vec<u8>,
    pub shadow: FbsTrunkSnapshot,
    pub panel: CpuQuantizationPanelReading,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CpuQuantizationStep {
    pub step: RankMarginStep,
    pub before: CpuQuantizationPanelReading,
    pub after: CpuQuantizationPanelReading,
    pub gate: QuantizationGate,
    pub committed: bool,
}

#[derive(Clone)]
pub struct CpuQuantizationShadow {
    source_snapshot_sha256: String,
    initial_artifact_sha256: String,
    shadow: FbsTrunkSnapshot,
    trainer: CpuRankMarginTrainer,
    training_samples: Vec<RankMarginSample>,
    panel_samples: Vec<RankMarginSample>,
}

impl CpuQuantizationShadow {
    pub fn from_snapshot(
        snapshot: FbsTrunkSnapshot,
        training_samples: Vec<RankMarginSample>,
        panel_samples: Vec<RankMarginSample>,
        config: RankMarginConfig,
        seed: u64,
    ) -> Result<Self> {
        validate_shadow_snapshot(&snapshot)?;
        let artifact = export_fbs_trunk_uorlut01(&snapshot)
            .map_err(|error| invalid(format!("quantization shadow export: {error}")))?;
        let source_snapshot_sha256 = canonical_sha256(&snapshot)?;
        let initial_artifact_sha256 = sha256(&artifact);
        let trainer =
            CpuRankMarginTrainer::from_artifact(&artifact, training_samples.clone(), config, seed)?;
        if trainer.artifact() != artifact {
            return Err(invalid(
                "quantization shadow trainer did not retain its source artifact",
            ));
        }
        let mut shadow = Self {
            source_snapshot_sha256,
            initial_artifact_sha256,
            shadow: snapshot,
            trainer,
            training_samples,
            panel_samples,
        };
        let panel = shadow.panel_reading()?;
        if !panel.exact.export.is_exact() {
            return Err(invalid(
                "quantization shadow baseline loses exact CPU/serving parity",
            ));
        }
        Ok(shadow)
    }

    pub fn shadow(&self) -> &FbsTrunkSnapshot {
        &self.shadow
    }

    pub fn trainer(&self) -> &CpuRankMarginTrainer {
        &self.trainer
    }

    pub fn artifact(&self) -> &[u8] {
        self.trainer.artifact()
    }

    pub fn source_snapshot_sha256(&self) -> &str {
        &self.source_snapshot_sha256
    }

    pub fn initial_artifact_sha256(&self) -> &str {
        &self.initial_artifact_sha256
    }

    pub fn panel_reading(&mut self) -> Result<CpuQuantizationPanelReading> {
        evaluate_panel(&self.shadow, &mut self.trainer, &self.panel_samples)
    }

    pub fn train_step_checked(&mut self) -> Result<CpuQuantizationStep> {
        let before = self.panel_reading()?;
        let mut trial = self.clone();
        let old_table = trial.trainer.table().to_vec();
        let step = trial.trainer.train_step()?;
        if step.updated && old_table == trial.trainer.table() {
            return Err(invalid(
                "quantized trainer reported an update without changing its table",
            ));
        }
        sync_shadow(&mut trial.shadow, &old_table, trial.trainer.table())?;
        let exported = export_fbs_trunk_uorlut01(&trial.shadow)
            .map_err(|error| invalid(format!("quantization shadow export: {error}")))?;
        if exported != trial.trainer.artifact() {
            return Err(invalid(
                "quantization shadow does not export the trained integer artifact",
            ));
        }
        if !step.updated && old_table != trial.trainer.table() {
            return Err(invalid(
                "quantization shadow table changed on a refused trainer step",
            ));
        }
        let after = trial.panel_reading()?;
        self.finish_trial(trial, before, after, step)
    }

    fn finish_trial(
        &mut self,
        trial: Self,
        before: CpuQuantizationPanelReading,
        after: CpuQuantizationPanelReading,
        step: RankMarginStep,
    ) -> Result<CpuQuantizationStep> {
        let gate = quantization_gate(&before, &after)?;
        let committed = matches!(gate, QuantizationGate::Accept | QuantizationGate::NoChange);
        if committed {
            *self = trial;
        }
        Ok(CpuQuantizationStep {
            step,
            before,
            after,
            gate,
            committed,
        })
    }

    pub fn checkpoint(&mut self) -> Result<CpuQuantizationCheckpoint> {
        let artifact = self.trainer.artifact().to_vec();
        let exported = export_fbs_trunk_uorlut01(&self.shadow)
            .map_err(|error| invalid(format!("quantization shadow export: {error}")))?;
        if exported != artifact {
            return Err(invalid(
                "quantization shadow does not export the current trainer artifact",
            ));
        }
        let trainer_checkpoint = self.trainer.checkpoint()?;
        Ok(CpuQuantizationCheckpoint {
            schema: CPU_QUANTIZATION_CHECKPOINT_SCHEMA.into(),
            substrate: CPU_SUBSTRATE.into(),
            qualification: CPU_QUALIFICATION.into(),
            source_snapshot_sha256: self.source_snapshot_sha256.clone(),
            initial_artifact_sha256: self.initial_artifact_sha256.clone(),
            artifact_sha256: sha256(&artifact),
            shadow_sha256: canonical_sha256(&self.shadow)?,
            trainer_checkpoint_sha256: sha256(&trainer_checkpoint),
            config: self.trainer.config(),
            training_samples: self.training_samples.clone(),
            panel_samples: self.panel_samples.clone(),
            trainer_checkpoint,
            shadow: self.shadow.clone(),
            panel: self.panel_reading()?,
        })
    }

    pub fn checkpoint_bytes(&mut self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self.checkpoint()?).map_err(|error| invalid(error.to_string()))
    }

    pub fn checkpoint_sha256(&mut self) -> Result<String> {
        Ok(sha256(&self.checkpoint_bytes()?))
    }

    pub fn resume(
        bytes: &[u8],
        expected_source_snapshot_sha256: &str,
        expected_initial_artifact_sha256: &str,
        expected_checkpoint_sha256: &str,
    ) -> Result<Self> {
        uor_r4_discrete::corpus::validate_sha256(expected_checkpoint_sha256)
            .map_err(|error| invalid(error.to_string()))?;
        if sha256(bytes) != expected_checkpoint_sha256 {
            return Err(invalid("quantization checkpoint envelope hash differs"));
        }
        let checkpoint: CpuQuantizationCheckpoint =
            serde_json::from_slice(bytes).map_err(|error| invalid(error.to_string()))?;
        if checkpoint.schema != CPU_QUANTIZATION_CHECKPOINT_SCHEMA
            || checkpoint.substrate != CPU_SUBSTRATE
            || checkpoint.qualification != CPU_QUALIFICATION
        {
            return Err(invalid(
                "quantization checkpoint schema or substrate differs",
            ));
        }
        for hash in [
            expected_source_snapshot_sha256,
            expected_initial_artifact_sha256,
            &checkpoint.source_snapshot_sha256,
            &checkpoint.initial_artifact_sha256,
            &checkpoint.artifact_sha256,
            &checkpoint.shadow_sha256,
            &checkpoint.trainer_checkpoint_sha256,
        ] {
            uor_r4_discrete::corpus::validate_sha256(hash)
                .map_err(|error| invalid(error.to_string()))?;
        }
        if checkpoint.source_snapshot_sha256 != expected_source_snapshot_sha256
            || checkpoint.initial_artifact_sha256 != expected_initial_artifact_sha256
        {
            return Err(invalid("quantization checkpoint lineage differs"));
        }
        validate_shadow_snapshot(&checkpoint.shadow)?;
        if canonical_sha256(&checkpoint.shadow)? != checkpoint.shadow_sha256 {
            return Err(invalid("quantization checkpoint shadow hash differs"));
        }
        if sha256(&checkpoint.trainer_checkpoint) != checkpoint.trainer_checkpoint_sha256 {
            return Err(invalid(
                "quantization checkpoint trainer state hash differs",
            ));
        }
        let trainer = CpuRankMarginTrainer::resume(
            &checkpoint.trainer_checkpoint,
            checkpoint.training_samples.clone(),
        )?;
        if trainer.config() != checkpoint.config {
            return Err(invalid("quantization checkpoint trainer config differs"));
        }
        if sha256(trainer.artifact()) != checkpoint.artifact_sha256 {
            return Err(invalid("quantization checkpoint artifact hash differs"));
        }
        let exported = export_fbs_trunk_uorlut01(&checkpoint.shadow)
            .map_err(|error| invalid(format!("quantization shadow export: {error}")))?;
        if exported != trainer.artifact() {
            return Err(invalid(
                "quantization checkpoint shadow and trainer artifact differ",
            ));
        }
        let mut restored = Self {
            source_snapshot_sha256: checkpoint.source_snapshot_sha256,
            initial_artifact_sha256: checkpoint.initial_artifact_sha256,
            shadow: checkpoint.shadow,
            trainer,
            training_samples: checkpoint.training_samples,
            panel_samples: checkpoint.panel_samples,
        };
        if restored.panel_reading()? != checkpoint.panel {
            return Err(invalid("quantization checkpoint panel reading differs"));
        }
        Ok(restored)
    }
}

fn validate_shadow_snapshot(snapshot: &FbsTrunkSnapshot) -> Result<()> {
    if snapshot.hamming.is_some() || snapshot.memory.is_some() {
        return Err(invalid(
            "quantization shadow supports only no-Hamming, no-memory trunks",
        ));
    }
    if snapshot.readout_nearest {
        return Err(invalid(
            "quantization shadow requires the native cell-writer head",
        ));
    }
    Ok(())
}

fn sync_shadow(shadow: &mut FbsTrunkSnapshot, old_table: &[i16], new_table: &[i16]) -> Result<()> {
    if old_table.len() != new_table.len() || shadow.embed.len() != new_table.len() {
        return Err(invalid("quantization shadow table length differs"));
    }
    let mut changed = Vec::new();
    for (index, (&old, &new)) in old_table.iter().zip(new_table).enumerate() {
        if old != new {
            changed.push(index);
            let delta = f64::from(i32::from(new) - i32::from(old)) / Q12_SCALE;
            shadow.embed[index] = (f64::from(shadow.embed[index]) + delta) as f32;
        }
    }
    if changed.len() > 2 {
        return Err(invalid(
            "one quantized trainer step changed more than two coordinates",
        ));
    }
    Ok(())
}

fn evaluate_panel(
    shadow: &FbsTrunkSnapshot,
    trainer: &mut CpuRankMarginTrainer,
    samples: &[RankMarginSample],
) -> Result<CpuQuantizationPanelReading> {
    if samples.is_empty() {
        return Err(invalid("quantization panel needs samples"));
    }
    let model = IntegerStackModel::parse(trainer.artifact())
        .map_err(|error| invalid(format!("quantization serving model: {error}")))?;
    let inputs = samples
        .iter()
        .map(|sample| sample.input.as_slice())
        .collect::<Vec<_>>();
    let plain_outputs = shadow
        .forward_f32_each(&inputs, false, FbsReferenceSnaps::FLOAT)
        .map_err(|error| invalid(format!("plain float panel: {error}")))?;
    let quantized_outputs = shadow
        .forward_f32_each(&inputs, true, FbsReferenceSnaps::INTEGER)
        .map_err(|error| invalid(format!("quantized panel: {error}")))?;
    if plain_outputs.len() != samples.len() || quantized_outputs.len() != samples.len() {
        return Err(invalid("quantization panel output count differs"));
    }

    let mut exact = PanelReading {
        artifact_digest: *blake3::hash(trainer.artifact()).as_bytes(),
        sample_fingerprint: sample_fingerprint(samples),
        margin_q12: trainer.config().margin_q12,
        samples: samples.len(),
        target_admitted: 0,
        top1_correct: 0,
        top8_correct: 0,
        margin_loss_sum: 0,
        target_rank_sum: 0,
        export: PanelExportReading {
            query_coordinates_checked: 0,
            query_coordinate_mismatches: 0,
            token_mismatches: 0,
            kept_cells_checked: 0,
            kept_cell_mismatches: 0,
        },
    };
    let mut plain = CpuFloatPanelReading {
        samples: samples.len(),
        target_admitted: 0,
        top1_correct: 0,
        top8_correct: 0,
    };
    let mut quantized = plain.clone();
    let mut plain_gap = CpuPanelGap::default();
    let mut quantized_gap = CpuPanelGap::default();

    for ((sample, plain_outputs), quantized_outputs) in
        samples.iter().zip(&plain_outputs).zip(&quantized_outputs)
    {
        let plain_output = plain_outputs
            .last()
            .ok_or_else(|| invalid("plain float panel sample has no output"))?;
        let quantized_output = quantized_outputs
            .last()
            .ok_or_else(|| invalid("quantized panel sample has no output"))?;
        let cpu_query = trainer.query_for(sample)?;
        let cpu_reading = trainer.reading(&cpu_query, sample.target)?;
        let mut session = model.session();
        let mut served = None;
        for &token in &sample.input {
            served = Some(
                session
                    .step_native(token)
                    .map_err(|error| invalid(format!("quantization panel serving: {error}")))?,
            );
        }
        let served = served.ok_or_else(|| invalid("quantization panel sample has no step"))?;

        exact.target_admitted += usize::from(cpu_reading.target_admitted);
        exact.top1_correct += usize::from(cpu_reading.top1 == sample.target);
        exact.top8_correct += usize::from(cpu_reading.top8.contains(&sample.target));
        exact.margin_loss_sum = exact
            .margin_loss_sum
            .saturating_add(cpu_reading.margin_loss);
        exact.target_rank_sum = exact
            .target_rank_sum
            .saturating_add(u64::try_from(cpu_reading.target_rank).unwrap_or(u64::MAX));
        exact.export.query_coordinates_checked +=
            u64::try_from(cpu_query.len()).unwrap_or(u64::MAX);
        exact.export.query_coordinate_mismatches += u64::try_from(
            cpu_query
                .iter()
                .zip(&served.query_q12)
                .filter(|(cpu, served)| cpu != served)
                .count()
                + cpu_query.len().abs_diff(served.query_q12.len()),
        )
        .unwrap_or(u64::MAX);
        exact.export.token_mismatches += u64::from(cpu_reading.top1 != served.token);
        exact.export.kept_cells_checked +=
            u64::try_from(cpu_reading.admitted_cells.len()).unwrap_or(u64::MAX);
        exact.export.kept_cell_mismatches += u64::try_from(
            cpu_reading
                .admitted_cells
                .iter()
                .zip(&served.kept_cells)
                .filter(|(cpu, served)| **cpu != **served)
                .count()
                + cpu_reading
                    .admitted_cells
                    .len()
                    .abs_diff(served.kept_cells.len()),
        )
        .unwrap_or(u64::MAX);

        accumulate_reference(
            plain_output,
            &cpu_reading.top8,
            cpu_reading.top1,
            cpu_reading.target_admitted,
            sample.target,
            &served.query_q12,
            served.token,
            &served.kept_cells,
            &mut plain,
            &mut plain_gap,
        )?;
        accumulate_reference(
            quantized_output,
            &cpu_reading.top8,
            cpu_reading.top1,
            cpu_reading.target_admitted,
            sample.target,
            &served.query_q12,
            served.token,
            &served.kept_cells,
            &mut quantized,
            &mut quantized_gap,
        )?;
    }

    Ok(CpuQuantizationPanelReading {
        sample_fingerprint: exact.sample_fingerprint,
        exact,
        plain,
        quantized,
        plain_gap,
        quantized_gap,
    })
}

#[allow(clippy::too_many_arguments)]
fn accumulate_reference(
    output: &FbsWriterOutput,
    exact_top8: &[u32],
    exact_top1: u32,
    exact_target_admitted: bool,
    target: u32,
    served_query: &[i32],
    served_token: u32,
    served_kept_cells: &[u32],
    reading: &mut CpuFloatPanelReading,
    gap: &mut CpuPanelGap,
) -> Result<()> {
    if output.query.len() != served_query.len() {
        return Err(invalid("quantization reference query length differs"));
    }
    let top8 = reference_top8(output)?;
    let target_admitted = output.candidates.iter().any(|(token, _)| *token == target);
    reading.target_admitted += usize::from(target_admitted);
    reading.top1_correct += usize::from(output.token == target);
    reading.top8_correct += usize::from(top8.contains(&target));

    gap.query_coordinates_checked += u64::try_from(output.query.len()).unwrap_or(u64::MAX);
    gap.query_coordinate_mismatches += u64::try_from(
        output
            .query
            .iter()
            .zip(served_query)
            .filter(|(value, exact)| q12_i32(**value) != **exact)
            .count(),
    )
    .unwrap_or(u64::MAX);
    gap.token_mismatches += u64::from(output.token != served_token);
    gap.kept_cells_checked += u64::try_from(output.kept_cells.len()).unwrap_or(u64::MAX);
    gap.kept_cell_mismatches += u64::try_from(
        output
            .kept_cells
            .iter()
            .zip(served_kept_cells)
            .filter(|(reference, served)| **reference != **served)
            .count()
            + output.kept_cells.len().abs_diff(served_kept_cells.len()),
    )
    .unwrap_or(u64::MAX);
    gap.target_admission_mismatches += u64::from(target_admitted != exact_target_admitted);
    gap.top1_mismatches += u64::from(output.token != exact_top1);
    gap.top8_mismatches += u64::try_from(
        top8.iter()
            .zip(exact_top8)
            .filter(|(reference, exact)| **reference != **exact)
            .count()
            + top8.len().abs_diff(exact_top8.len()),
    )
    .unwrap_or(u64::MAX);
    Ok(())
}

fn reference_top8(output: &FbsWriterOutput) -> Result<Vec<u32>> {
    if output
        .candidates
        .iter()
        .any(|(_, score)| !score.is_finite())
    {
        return Err(invalid("quantization reference produced a nonfinite score"));
    }
    let mut candidates = output.candidates.clone();
    candidates.sort_by(|(left_token, left_score), (right_token, right_score)| {
        left_score
            .total_cmp(right_score)
            .then_with(|| left_token.cmp(right_token))
    });
    Ok(candidates
        .into_iter()
        .take(TOP_K)
        .map(|(token, _)| token)
        .collect())
}

fn q12_i32(value: f64) -> i32 {
    (value * Q12_SCALE).round().clamp(-32768.0, 32767.0) as i32
}

fn canonical_sha256<T: serde::Serialize>(value: &T) -> Result<String> {
    let bytes = serde_json::to_vec(value).map_err(|error| invalid(error.to_string()))?;
    Ok(sha256(&bytes))
}

fn sha256(bytes: &[u8]) -> String {
    uor_r4_discrete::corpus::sha256(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use uor_r4_discrete::normalization_mechanism::orbit_root_coords;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth};

    use crate::VOCABULARY;

    fn snapshot() -> FbsTrunkSnapshot {
        let mut snapshot = synthetic_fbs_trunk_with(
            307,
            FbsSynth {
                vocab: VOCABULARY,
                d: 64,
                mlp: 64,
                layers: 1,
                context: 8,
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
                let root =
                    ((token >> 8) * 7 + ((token >> 4) & 15) * 3 + (token & 15) * 5 + block) & 15;
                snapshot.embed.extend(
                    orbit_root_coords(root as u8)
                        .unwrap()
                        .map(|value| value as f32),
                );
            }
        }
        snapshot
    }

    fn accepted_sample(artifact: &[u8]) -> RankMarginSample {
        accepted_sample_with_config(artifact, RankMarginConfig::default())
    }

    fn accepted_sample_with_config(artifact: &[u8], config: RankMarginConfig) -> RankMarginSample {
        let mut probe = CpuRankMarginTrainer::from_artifact(
            artifact,
            vec![RankMarginSample {
                input: vec![0],
                target: 0,
            }],
            config,
            42,
        )
        .unwrap();
        for input in 0u32..VOCABULARY as u32 {
            let reading = probe
                .evaluate_sample(&RankMarginSample {
                    input: vec![input],
                    target: 0,
                })
                .unwrap();
            for target in reading.top8.iter().rev().copied() {
                if target == reading.top1 {
                    continue;
                }
                let sample = RankMarginSample {
                    input: vec![input],
                    target,
                };
                let mut trainer =
                    CpuRankMarginTrainer::from_artifact(artifact, vec![sample.clone()], config, 42)
                        .unwrap();
                if trainer.train_steps(1).unwrap()[0].updated {
                    return sample;
                }
            }
        }
        panic!("no accepted tied-table update");
    }

    fn no_change_sample(artifact: &[u8]) -> RankMarginSample {
        let mut probe = CpuRankMarginTrainer::from_artifact(
            artifact,
            vec![RankMarginSample {
                input: vec![0],
                target: 0,
            }],
            RankMarginConfig::default(),
            42,
        )
        .unwrap();
        for input in 0u32..VOCABULARY as u32 {
            for target in 0u32..256 {
                let sample = RankMarginSample {
                    input: vec![input],
                    target,
                };
                if !probe.evaluate_sample(&sample).unwrap().target_admitted {
                    return sample;
                }
            }
        }
        panic!("no refused sample");
    }

    fn find_admission_loss(
        artifact: &[u8],
        config: RankMarginConfig,
    ) -> (RankMarginSample, Vec<RankMarginSample>) {
        let seed_sample = RankMarginSample {
            input: vec![0],
            target: 0,
        };
        let mut probe =
            CpuRankMarginTrainer::from_artifact(artifact, vec![seed_sample.clone()], config, 42)
                .unwrap();
        for input in 0u32..VOCABULARY as u32 {
            let reading = probe
                .evaluate_sample(&RankMarginSample {
                    input: vec![input],
                    target: 0,
                })
                .unwrap();
            for target in reading.top8.iter().rev().copied() {
                if target == reading.top1 {
                    continue;
                }
                let sample = RankMarginSample {
                    input: vec![input],
                    target,
                };
                let mut candidate =
                    CpuRankMarginTrainer::from_artifact(artifact, vec![sample.clone()], config, 42)
                        .unwrap();
                let old_table = candidate.table().to_vec();
                let step = candidate.train_step().unwrap();
                if !step.updated {
                    continue;
                }
                assert_ne!(old_table, candidate.table());
                let Some(rival) = step.rival else {
                    continue;
                };
                let panel = vec![
                    sample.clone(),
                    RankMarginSample {
                        input: vec![step.target],
                        target: step.target,
                    },
                    RankMarginSample {
                        input: vec![rival],
                        target: rival,
                    },
                ];
                let mut baseline = CpuRankMarginTrainer::from_artifact(
                    artifact,
                    vec![seed_sample.clone()],
                    config,
                    42,
                )
                .unwrap();
                if exact_admission(&mut candidate, &panel) < exact_admission(&mut baseline, &panel)
                {
                    return (sample, panel);
                }
            }
        }
        panic!("no real accepted update loses whole-panel admission");
    }

    fn exact_admission(trainer: &mut CpuRankMarginTrainer, samples: &[RankMarginSample]) -> usize {
        samples
            .iter()
            .map(|sample| {
                let query = trainer.query_for(sample).unwrap();
                usize::from(
                    trainer
                        .reading(&query, sample.target)
                        .unwrap()
                        .target_admitted,
                )
            })
            .sum()
    }

    fn shadow(sample: RankMarginSample) -> (CpuQuantizationShadow, FbsTrunkSnapshot) {
        let snapshot = snapshot();
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let mut panel = vec![sample.clone()];
        panel.extend((0u32..7).map(|input| RankMarginSample {
            input: vec![input],
            target: input & 255,
        }));
        let shadow = CpuQuantizationShadow::from_snapshot(
            snapshot.clone(),
            vec![sample],
            panel,
            RankMarginConfig::default(),
            42,
        )
        .unwrap();
        assert_eq!(shadow.artifact(), artifact);
        (shadow, snapshot)
    }

    #[test]
    fn accepted_q12_update_tracks_shadow_and_re_exports_exactly() {
        let initial = snapshot();
        let artifact = export_fbs_trunk_uorlut01(&initial).unwrap();
        let sample = accepted_sample(&artifact);
        let (mut shadow, initial) = shadow(sample);
        let old_table = shadow.trainer.table().to_vec();
        let old_artifact = shadow.artifact().to_vec();
        let outcome = shadow.train_step_checked().unwrap();
        assert!(outcome.step.updated);
        assert_eq!(outcome.gate, QuantizationGate::Accept);
        assert!(outcome.committed);
        assert!(outcome.after.exact.export.is_exact());
        assert!(outcome.after.quantized_gap.query_coordinate_mismatches > 0);
        assert_eq!(outcome.after.quantized_gap.token_mismatches, 0);
        assert_eq!(outcome.after.quantized_gap.kept_cell_mismatches, 0);
        assert_ne!(shadow.artifact(), old_artifact);
        assert_eq!(
            export_fbs_trunk_uorlut01(shadow.shadow()).unwrap(),
            shadow.artifact()
        );
        for (index, (&old, &new)) in old_table.iter().zip(shadow.trainer.table()).enumerate() {
            if old == new {
                continue;
            }
            let shadow_delta = (f64::from(shadow.shadow().embed[index])
                - f64::from(initial.embed[index]))
                * Q12_SCALE;
            assert!((shadow_delta - f64::from(new - old)).abs() < 1e-4);
        }
        for _ in 0..8 {
            shadow.train_step_checked().unwrap();
            assert_eq!(
                export_fbs_trunk_uorlut01(shadow.shadow()).unwrap(),
                shadow.artifact()
            );
        }
        let before = shadow.checkpoint().unwrap();
        let before_bytes = serde_json::to_vec(&before).unwrap();
        let checkpoint_hash = sha256(&before_bytes);
        let mut resumed = CpuQuantizationShadow::resume(
            &before_bytes,
            &before.source_snapshot_sha256,
            &before.initial_artifact_sha256,
            &checkpoint_hash,
        )
        .unwrap();
        assert_eq!(
            resumed.checkpoint_bytes().unwrap(),
            shadow.checkpoint_bytes().unwrap()
        );
        let uninterrupted_next = shadow.train_step_checked().unwrap();
        let resumed_next = resumed.train_step_checked().unwrap();
        assert_eq!(resumed_next, uninterrupted_next);
        assert_eq!(
            resumed.checkpoint_sha256().unwrap(),
            shadow.checkpoint_sha256().unwrap()
        );
    }

    #[test]
    fn admission_loss_rolls_back_the_complete_trial_state() {
        let initial = snapshot();
        let artifact = export_fbs_trunk_uorlut01(&initial).unwrap();
        let config = RankMarginConfig {
            step_q12: 1_024,
            ..RankMarginConfig::default()
        };
        let (sample, panel) = find_admission_loss(&artifact, config);
        let mut shadow =
            CpuQuantizationShadow::from_snapshot(initial, vec![sample], panel, config, 42).unwrap();
        let before = shadow.panel_reading().unwrap();
        assert!(before.exact.target_admitted > 0);
        let before_checkpoint = shadow.checkpoint_bytes().unwrap();
        let outcome = shadow.train_step_checked().unwrap();
        assert!(outcome.step.updated);
        assert_eq!(outcome.gate, QuantizationGate::RejectAdmission);
        assert!(!outcome.committed);
        assert_eq!(shadow.checkpoint_bytes().unwrap(), before_checkpoint);
    }

    #[test]
    fn no_change_step_is_deterministic_and_keeps_shadow_bytes() {
        let initial = snapshot();
        let artifact = export_fbs_trunk_uorlut01(&initial).unwrap();
        let sample = no_change_sample(&artifact);
        let (mut shadow, _) = shadow(sample);
        let before_artifact = shadow.artifact().to_vec();
        let before_shadow = serde_json::to_vec(shadow.shadow()).unwrap();
        let outcome = shadow.train_step_checked().unwrap();
        assert!(!outcome.step.updated);
        assert_eq!(outcome.gate, QuantizationGate::NoChange);
        assert!(outcome.committed);
        assert_eq!(shadow.artifact(), before_artifact);
        assert_eq!(serde_json::to_vec(shadow.shadow()).unwrap(), before_shadow);
        assert_eq!(shadow.trainer.step_count(), 1);
    }

    #[test]
    fn gate_separates_admission_export_and_float_only_gains() {
        let initial = snapshot();
        let artifact = export_fbs_trunk_uorlut01(&initial).unwrap();
        let sample = accepted_sample(&artifact);
        let (mut shadow, _) = shadow(sample);
        let before = shadow.panel_reading().unwrap();
        assert!(before.exact.target_admitted > 0);
        assert_eq!(
            quantization_gate(&before, &before).unwrap(),
            QuantizationGate::NoChange
        );

        let mut admission_loss = before.clone();
        admission_loss.exact.target_admitted -= 1;
        assert_eq!(
            quantization_gate(&before, &admission_loss).unwrap(),
            QuantizationGate::RejectAdmission
        );

        let mut export_loss = before.clone();
        export_loss.exact.export.token_mismatches = 1;
        assert_eq!(
            quantization_gate(&before, &export_loss).unwrap(),
            QuantizationGate::RejectExportGap
        );
        let mut baseline_loss = before.clone();
        baseline_loss.exact.export.kept_cell_mismatches = 1;
        assert_eq!(
            quantization_gate(&baseline_loss, &before).unwrap(),
            QuantizationGate::RejectExportGap
        );

        let mut float_only = before.clone();
        float_only.plain.top1_correct += 1;
        assert_eq!(
            quantization_gate(&before, &float_only).unwrap(),
            QuantizationGate::ExportLimited
        );

        let mut mixed_metric = before.clone();
        mixed_metric.plain.top1_correct += 1;
        mixed_metric.exact.top8_correct += 1;
        assert_eq!(
            quantization_gate(&before, &mixed_metric).unwrap(),
            QuantizationGate::ExportLimited
        );
    }

    #[test]
    fn checkpoint_refuses_tamper_stale_lineage_and_trailing_bytes() {
        let initial = snapshot();
        let artifact = export_fbs_trunk_uorlut01(&initial).unwrap();
        let sample = accepted_sample(&artifact);
        let (mut shadow, _) = shadow(sample);
        let checkpoint = shadow.checkpoint().unwrap();
        let source = checkpoint.source_snapshot_sha256.clone();
        let initial_artifact = checkpoint.initial_artifact_sha256.clone();
        let bytes = serde_json::to_vec(&checkpoint).unwrap();
        let checkpoint_hash = sha256(&bytes);
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        let refusal = |value: Value| {
            let bytes = serde_json::to_vec(&value).unwrap();
            let hash = sha256(&bytes);
            assert!(
                CpuQuantizationShadow::resume(&bytes, &source, &initial_artifact, &hash).is_err()
            );
        };
        for mutate in [
            |value: &mut Value| value["schema"] = json!("wrong"),
            |value: &mut Value| value["substrate"] = json!("gpu"),
            |value: &mut Value| value["qualification"] = json!("qualified"),
            |value: &mut Value| value["shadow_sha256"] = json!("00".repeat(32)),
            |value: &mut Value| value["artifact_sha256"] = json!("00".repeat(32)),
            |value: &mut Value| value["trainer_checkpoint_sha256"] = json!("00".repeat(32)),
            |value: &mut Value| value["shadow"]["readout_nearest"] = json!(true),
            |value: &mut Value| value["panel"]["exact"]["target_admitted"] = json!(0),
            |value: &mut Value| value["unexpected"] = json!(true),
            |value: &mut Value| {
                value["trainer_checkpoint"].as_array_mut().unwrap().pop();
            },
        ] {
            let mut bad = value.clone();
            mutate(&mut bad);
            refusal(bad);
        }
        assert!(CpuQuantizationShadow::resume(
            &bytes,
            &"00".repeat(32),
            &initial_artifact,
            &checkpoint_hash
        )
        .is_err());
        assert!(
            CpuQuantizationShadow::resume(&bytes, &source, &"00".repeat(32), &checkpoint_hash)
                .is_err()
        );

        let mut trainer_tamper = value.clone();
        let trainer = trainer_tamper["trainer_checkpoint"].as_array_mut().unwrap();
        trainer[32] = json!(trainer[32].as_u64().unwrap() ^ 1);
        let trainer_tamper_bytes = serde_json::to_vec(&trainer_tamper).unwrap();
        assert!(CpuQuantizationShadow::resume(
            &trainer_tamper_bytes,
            &source,
            &initial_artifact,
            &checkpoint_hash
        )
        .is_err());
        let coherent_outer = sha256(&trainer_tamper_bytes);
        let error = match CpuQuantizationShadow::resume(
            &trainer_tamper_bytes,
            &source,
            &initial_artifact,
            &coherent_outer,
        ) {
            Ok(_) => panic!("coherently rehashed trainer state must be refused"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("trainer state hash"));

        let mut trailing = bytes;
        trailing.push(b'0');
        let trailing_hash = sha256(&trailing);
        assert!(CpuQuantizationShadow::resume(
            &trailing,
            &source,
            &initial_artifact,
            &trailing_hash
        )
        .is_err());
    }

    #[test]
    fn hamming_and_memory_snapshots_are_outside_this_unit() {
        let mut snapshot = snapshot();
        snapshot.layers.push(snapshot.layers[0].clone());
        let mut hamming = synthetic_hamming(snapshot.width, snapshot.layers.len(), 4, 152);
        hamming
            .reads
            .iter_mut()
            .for_each(|read| read.weight.fill(1.0));
        snapshot.hamming = Some(hamming);
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let sample = RankMarginSample {
            input: vec![0],
            target: 0,
        };
        assert!(CpuQuantizationShadow::from_snapshot(
            snapshot,
            vec![sample.clone()],
            vec![sample],
            RankMarginConfig::default(),
            42,
        )
        .is_err());
        assert!(!artifact.is_empty());
    }

    #[test]
    fn malformed_training_and_panel_samples_are_refused() {
        let snapshot = snapshot();
        let sample = RankMarginSample {
            input: vec![0],
            target: 0,
        };
        assert!(CpuQuantizationShadow::from_snapshot(
            snapshot.clone(),
            vec![sample.clone()],
            Vec::new(),
            RankMarginConfig::default(),
            42,
        )
        .is_err());
        assert!(CpuQuantizationShadow::from_snapshot(
            snapshot.clone(),
            vec![sample.clone()],
            vec![RankMarginSample {
                input: vec![VOCABULARY as u32],
                target: 0,
            }],
            RankMarginConfig::default(),
            42,
        )
        .is_err());
        assert!(CpuQuantizationShadow::from_snapshot(
            snapshot,
            Vec::new(),
            vec![sample],
            RankMarginConfig::default(),
            42,
        )
        .is_err());
    }
}
