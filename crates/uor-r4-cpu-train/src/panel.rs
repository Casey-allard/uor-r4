//! Panel-level rank-margin evaluation and exact CPU-versus-serving export checks.

use rayon::prelude::*;
use uor_r4_integer::stack::IntegerStackModel;

use crate::train::{
    sample_fingerprint, validate_training_model, CpuRankMarginEvaluator, CpuRankMarginTrainer,
    RankMarginConfig, RankMarginSample,
};
use crate::CpuTrainError;

type Result<T> = std::result::Result<T, CpuTrainError>;

fn invalid(message: impl Into<String>) -> CpuTrainError {
    CpuTrainError::Invalid(message.into())
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelExportReading {
    pub query_coordinates_checked: u64,
    pub query_coordinate_mismatches: u64,
    pub token_mismatches: u64,
    pub kept_cells_checked: u64,
    pub kept_cell_mismatches: u64,
}

impl PanelExportReading {
    pub fn is_exact(&self) -> bool {
        self.query_coordinate_mismatches == 0
            && self.token_mismatches == 0
            && self.kept_cell_mismatches == 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelReading {
    pub artifact_digest: [u8; 32],
    pub sample_fingerprint: [u8; 32],
    pub margin_q12: i32,
    pub samples: usize,
    pub target_admitted: usize,
    pub top1_correct: usize,
    pub top8_correct: usize,
    pub margin_loss_sum: i64,
    pub target_rank_sum: u64,
    pub export: PanelExportReading,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelDelta {
    pub target_admitted: i64,
    pub top1_correct: i64,
    pub top8_correct: i64,
    pub margin_loss_sum: i64,
    pub query_coordinate_mismatches: i64,
    pub token_mismatches: i64,
    pub kept_cell_mismatches: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelGateDecision {
    Accept(PanelDelta),
    RejectAdmission(PanelDelta),
    RejectExportGap(PanelDelta),
}

impl PanelDelta {
    pub fn between(before: &PanelReading, after: &PanelReading) -> Result<Self> {
        if before.samples != after.samples {
            return Err(invalid("panel comparison sample counts differ"));
        }
        if before.sample_fingerprint != after.sample_fingerprint {
            return Err(invalid("panel comparison sample sets differ"));
        }
        if before.margin_q12 != after.margin_q12 {
            return Err(invalid("panel comparison margin configurations differ"));
        }
        if before.export.query_coordinates_checked != after.export.query_coordinates_checked
            || before.export.kept_cells_checked != after.export.kept_cells_checked
        {
            return Err(invalid("panel comparison export shapes differ"));
        }
        let difference = |left: usize, right: usize| -> i64 {
            i64::try_from(right)
                .unwrap_or(i64::MAX)
                .saturating_sub(i64::try_from(left).unwrap_or(i64::MAX))
        };
        let difference_u64 = |left: u64, right: u64| -> i64 {
            i64::try_from(right)
                .unwrap_or(i64::MAX)
                .saturating_sub(i64::try_from(left).unwrap_or(i64::MAX))
        };
        Ok(Self {
            target_admitted: difference(before.target_admitted, after.target_admitted),
            top1_correct: difference(before.top1_correct, after.top1_correct),
            top8_correct: difference(before.top8_correct, after.top8_correct),
            margin_loss_sum: after.margin_loss_sum.saturating_sub(before.margin_loss_sum),
            query_coordinate_mismatches: difference_u64(
                before.export.query_coordinate_mismatches,
                after.export.query_coordinate_mismatches,
            ),
            token_mismatches: difference_u64(
                before.export.token_mismatches,
                after.export.token_mismatches,
            ),
            kept_cell_mismatches: difference_u64(
                before.export.kept_cell_mismatches,
                after.export.kept_cell_mismatches,
            ),
        })
    }
}

pub fn panel_admission_gate(
    before: &PanelReading,
    after: &PanelReading,
) -> Result<PanelGateDecision> {
    let delta = PanelDelta::between(before, after)?;
    if after.target_admitted < before.target_admitted {
        Ok(PanelGateDecision::RejectAdmission(delta))
    } else if !after.export.is_exact() {
        Ok(PanelGateDecision::RejectExportGap(delta))
    } else {
        Ok(PanelGateDecision::Accept(delta))
    }
}

pub struct CpuPanelEvaluator {
    artifact: Vec<u8>,
    model: IntegerStackModel,
    evaluator: CpuRankMarginEvaluator,
    samples: Vec<RankMarginSample>,
}

struct PanelRowReading {
    target_admitted: usize,
    top1_correct: usize,
    top8_correct: usize,
    margin_loss_sum: i64,
    target_rank_sum: u64,
    query_coordinates_checked: u64,
    query_coordinate_mismatches: u64,
    token_mismatches: u64,
    kept_cells_checked: u64,
    kept_cell_mismatches: u64,
}

impl CpuPanelEvaluator {
    pub fn from_artifact(
        artifact: &[u8],
        samples: Vec<RankMarginSample>,
        margin_q12: i32,
    ) -> Result<Self> {
        if samples.is_empty() {
            return Err(invalid("panel evaluation needs samples"));
        }
        validate_training_model(artifact)?;
        let model = IntegerStackModel::parse(artifact)
            .map_err(|error| invalid(format!("panel model: {error}")))?;
        let evaluator = CpuRankMarginTrainer::from_artifact(
            artifact,
            samples.clone(),
            RankMarginConfig {
                margin_q12,
                ..RankMarginConfig::default()
            },
            0,
        )?;
        let evaluator = evaluator.evaluator();
        Ok(Self {
            artifact: artifact.to_vec(),
            model,
            evaluator,
            samples,
        })
    }

    pub fn artifact(&self) -> &[u8] {
        &self.artifact
    }

    pub fn samples(&self) -> &[RankMarginSample] {
        &self.samples
    }

    pub fn evaluate(&mut self) -> Result<PanelReading> {
        let rows = self
            .samples
            .par_iter()
            .map(|sample| self.evaluate_row(sample))
            .collect::<Result<Vec<_>>>()?;
        let mut reading = PanelReading {
            artifact_digest: *blake3::hash(&self.artifact).as_bytes(),
            sample_fingerprint: sample_fingerprint(&self.samples),
            margin_q12: self.evaluator.config().margin_q12,
            samples: self.samples.len(),
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
        for row in rows {
            reading.target_admitted += row.target_admitted;
            reading.top1_correct += row.top1_correct;
            reading.top8_correct += row.top8_correct;
            reading.margin_loss_sum = reading.margin_loss_sum.saturating_add(row.margin_loss_sum);
            reading.target_rank_sum = reading.target_rank_sum.saturating_add(row.target_rank_sum);
            reading.export.query_coordinates_checked += row.query_coordinates_checked;
            reading.export.query_coordinate_mismatches += row.query_coordinate_mismatches;
            reading.export.token_mismatches += row.token_mismatches;
            reading.export.kept_cells_checked += row.kept_cells_checked;
            reading.export.kept_cell_mismatches += row.kept_cell_mismatches;
        }
        Ok(reading)
    }

    fn evaluate_row(&self, sample: &RankMarginSample) -> Result<PanelRowReading> {
        let mut evaluator = self.evaluator.clone();
        let cpu_query = evaluator.query_for(sample)?;
        let cpu_reading = evaluator.reading(&cpu_query, sample.target)?;
        let mut session = self.model.session();
        let mut served = None;
        for &token in &sample.input {
            served = Some(
                session
                    .step_native(token)
                    .map_err(|error| invalid(format!("serving panel step: {error}")))?,
            );
        }
        let served = served.ok_or_else(|| invalid("panel sample has no serving step"))?;

        Ok(PanelRowReading {
            target_admitted: usize::from(cpu_reading.target_admitted),
            top1_correct: usize::from(cpu_reading.top1 == sample.target),
            top8_correct: usize::from(cpu_reading.top8.contains(&sample.target)),
            margin_loss_sum: cpu_reading.margin_loss,
            target_rank_sum: u64::try_from(cpu_reading.target_rank).unwrap_or(u64::MAX),
            query_coordinates_checked: u64::try_from(cpu_query.len()).unwrap_or(u64::MAX),
            query_coordinate_mismatches: u64::try_from(
                cpu_query
                    .iter()
                    .zip(&served.query_q12)
                    .filter(|(cpu, served)| cpu != served)
                    .count()
                    + cpu_query.len().abs_diff(served.query_q12.len()),
            )
            .unwrap_or(u64::MAX),
            token_mismatches: u64::from(cpu_reading.top1 != served.token),
            kept_cells_checked: u64::try_from(cpu_reading.admitted_cells.len()).unwrap_or(u64::MAX),
            kept_cell_mismatches: u64::try_from(
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
            .unwrap_or(u64::MAX),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VOCABULARY;
    use uor_r4_discrete::normalization_mechanism::orbit_root_coords;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    fn artifact() -> Vec<u8> {
        artifact_with_nearest(false)
    }

    fn artifact_with_nearest(nearest: bool) -> Vec<u8> {
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
        snapshot.readout_nearest = nearest;
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
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn samples() -> Vec<RankMarginSample> {
        (0u32..8)
            .map(|input| RankMarginSample {
                input: vec![input],
                target: input & 255,
            })
            .collect()
    }

    fn accepted_trainer(artifact: &[u8]) -> (CpuRankMarginTrainer, RankMarginSample) {
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
                let mut trainer = CpuRankMarginTrainer::from_artifact(
                    artifact,
                    vec![sample.clone()],
                    RankMarginConfig::default(),
                    42,
                )
                .unwrap();
                if trainer.train_steps(1).unwrap()[0].updated {
                    return (trainer, sample);
                }
            }
        }
        panic!("no accepted tied-table update");
    }

    #[test]
    fn finite_panel_matches_the_serving_integer_session_exactly() {
        let artifact = artifact();
        let mut evaluator = CpuPanelEvaluator::from_artifact(&artifact, samples(), 1).unwrap();
        let left = evaluator.evaluate().unwrap();
        let right = evaluator.evaluate().unwrap();
        assert_eq!(left, right);
        assert_eq!(left.samples, 8);
        assert!(left.export.is_exact());
        assert_eq!(left.export.query_coordinates_checked, 8 * 64);
        assert_eq!(left.export.kept_cells_checked, 8 * 8);
    }

    #[test]
    fn panel_reading_is_identical_across_bounded_worker_counts() {
        let artifact = artifact();
        let samples = samples();
        let evaluate = |threads| {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            pool.install(|| {
                CpuPanelEvaluator::from_artifact(&artifact, samples.clone(), 1)
                    .unwrap()
                    .evaluate()
                    .unwrap()
            })
        };

        let expected = evaluate(1);
        for threads in [2, 3, 5, 8, 16, 32] {
            assert_eq!(evaluate(threads), expected, "{threads} workers");
        }
    }

    #[test]
    fn checkpoint_and_resume_produce_the_same_panel_reading() {
        let artifact = artifact();
        let (trainer, sample) = accepted_trainer(&artifact);
        let panel = vec![sample];
        let checkpoint = trainer.checkpoint().unwrap();
        let resumed = CpuRankMarginTrainer::resume(&checkpoint, panel.clone()).unwrap();
        assert_eq!(trainer.artifact(), resumed.artifact());
        let mut left =
            CpuPanelEvaluator::from_artifact(trainer.artifact(), panel.clone(), 1).unwrap();
        let mut right =
            CpuPanelEvaluator::from_artifact(resumed.artifact(), panel.clone(), 1).unwrap();
        let left = left.evaluate().unwrap();
        let right = right.evaluate().unwrap();
        assert_eq!(left, right);
        assert!(left.export.is_exact());

        let mut baseline = CpuPanelEvaluator::from_artifact(&artifact, panel, 1).unwrap();
        let before = baseline.evaluate().unwrap();
        assert!(matches!(
            panel_admission_gate(&before, &left).unwrap(),
            PanelGateDecision::Accept(_)
        ));
    }

    #[test]
    fn matched_gate_accepts_no_change_and_rejects_admission_or_export_loss() {
        let artifact = artifact();
        let mut evaluator = CpuPanelEvaluator::from_artifact(&artifact, samples(), 1).unwrap();
        let before = evaluator.evaluate().unwrap();
        assert_eq!(
            panel_admission_gate(&before, &before).unwrap(),
            PanelGateDecision::Accept(PanelDelta {
                target_admitted: 0,
                top1_correct: 0,
                top8_correct: 0,
                margin_loss_sum: 0,
                query_coordinate_mismatches: 0,
                token_mismatches: 0,
                kept_cell_mismatches: 0,
            })
        );

        let mut admission_loss = before.clone();
        admission_loss.target_admitted -= 1;
        assert!(matches!(
            panel_admission_gate(&before, &admission_loss).unwrap(),
            PanelGateDecision::RejectAdmission(_)
        ));

        let mut export_gap = before.clone();
        export_gap.export.token_mismatches = 1;
        assert!(matches!(
            panel_admission_gate(&before, &export_gap).unwrap(),
            PanelGateDecision::RejectExportGap(_)
        ));

        let mut different = samples();
        different[0].target = 1;
        let mut different_evaluator =
            CpuPanelEvaluator::from_artifact(&artifact, different, 1).unwrap();
        assert!(PanelDelta::between(&before, &different_evaluator.evaluate().unwrap()).is_err());

        let mut margin_evaluator =
            CpuPanelEvaluator::from_artifact(&artifact, samples(), 2).unwrap();
        assert!(PanelDelta::between(&before, &margin_evaluator.evaluate().unwrap()).is_err());
    }

    #[test]
    fn empty_malformed_and_unsupported_panels_are_refused() {
        let artifact = artifact();
        assert!(CpuPanelEvaluator::from_artifact(&artifact, Vec::new(), 1).is_err());
        assert!(CpuPanelEvaluator::from_artifact(
            &artifact,
            vec![RankMarginSample {
                input: Vec::new(),
                target: 0,
            }],
            1,
        )
        .is_err());
        assert!(CpuPanelEvaluator::from_artifact(
            &artifact,
            vec![RankMarginSample {
                input: vec![0],
                target: VOCABULARY as u32,
            }],
            1,
        )
        .is_err());
        let mut truncated = artifact;
        truncated.truncate(truncated.len() - 1);
        assert!(CpuPanelEvaluator::from_artifact(&truncated, samples(), 1).is_err());
        assert!(
            CpuPanelEvaluator::from_artifact(&artifact_with_nearest(true), samples(), 1,).is_err()
        );
    }
}
