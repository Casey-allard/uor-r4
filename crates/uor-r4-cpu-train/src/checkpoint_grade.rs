//! Bounded CPU grading for exported geometric checkpoints.
//!
//! This module joins the exact served forward, held-out row evaluation,
//! complete-reply generation and CPU/serving audit. It is development evidence
//! on the D84 host and does not replace GPU qualification.

use std::time::Instant;

use rayon::prelude::*;
use serde::Serialize;
use uor_r4_integer::stack::IntegerStackModel;

use crate::corpus::{validate_corpus_manifest, CpuMaskedCorpus};
use crate::{
    audit_served_forward, evaluate_replies, read_uort_token_slice, CpuFbsForward, CpuFbsHead,
    CpuPanelEvaluator, CpuTrainError, MemoryWeightSample, PanelExportReading, RankMarginConfig,
    RankMarginSample, ServedForwardAuditConfig, TimedReplyReading,
};

pub const CHECKPOINT_GRADE_SCHEMA: &str = "uor-r4.cpu-checkpoint-grade/1";
const SAMPLE_FINGERPRINT_SCHEMA: &[u8] = b"uor-r4.cpu-checkpoint-grade-samples/1\0";

type Result<T> = std::result::Result<T, CpuTrainError>;

fn invalid(message: impl Into<String>) -> CpuTrainError {
    CpuTrainError::Invalid(message.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckpointGradeConfig {
    pub max_evaluation_rows: usize,
    pub max_reply_examples: usize,
    pub reply_tokens: usize,
    pub margin_q12: i32,
}

impl Default for CheckpointGradeConfig {
    fn default() -> Self {
        Self {
            max_evaluation_rows: 4_096,
            max_reply_examples: 16,
            reply_tokens: 16,
            margin_q12: RankMarginConfig::default().margin_q12,
        }
    }
}

impl CheckpointGradeConfig {
    fn validate(self) -> Result<Self> {
        if self.max_evaluation_rows == 0
            || self.max_reply_examples == 0
            || self.reply_tokens == 0
            || self.reply_tokens > 64
            || self.margin_q12 <= 0
        {
            return Err(invalid(
                "checkpoint-grade row, reply or margin bounds are invalid",
            ));
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CheckpointGradeCorpus {
    pub split: String,
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub tokenizer_sha256: String,
    pub tokens: usize,
    pub response_targets: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CheckpointGradeSettings {
    pub max_evaluation_rows: usize,
    pub max_reply_examples: usize,
    pub reply_tokens: usize,
    pub margin_q12: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CheckpointGradeEvaluation {
    pub head: String,
    pub samples: usize,
    pub selected_from: usize,
    pub sample_fingerprint: String,
    pub top1_correct: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top8_correct: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_admitted: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<PanelExportReading>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CheckpointGradeBaseline {
    pub artifact_sha256: String,
    pub evaluation: CheckpointGradeEvaluation,
    pub replies: TimedReplyReading,
    pub top1_delta: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top1_only_candidate: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top1_only_baseline: Option<usize>,
    pub top8_delta: Option<i64>,
    pub target_admitted_delta: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CheckpointGradeAudit {
    pub positions_checked: usize,
    pub query_coordinates_compared: u64,
    pub query_coordinate_mismatches: u64,
    pub token_decisions_compared: u64,
    pub token_mismatches: u64,
    pub kept_cells_compared: u64,
    pub kept_cell_mismatches: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CheckpointGradeReport {
    pub schema: &'static str,
    pub substrate: &'static str,
    pub qualification: &'static str,
    pub artifact_sha256: String,
    pub vocab: usize,
    pub context: usize,
    pub corpus: CheckpointGradeCorpus,
    pub settings: CheckpointGradeSettings,
    pub evaluation: CheckpointGradeEvaluation,
    pub reply_sample_fingerprint: String,
    pub replies: TimedReplyReading,
    pub audit: CheckpointGradeAudit,
    pub baseline: Option<CheckpointGradeBaseline>,
    pub wall_elapsed_us: u64,
}

struct RowGrade {
    evaluation: CheckpointGradeEvaluation,
    top1_rows: Vec<bool>,
}

fn sample_fingerprint(samples: &[RankMarginSample]) -> String {
    let mut hash = blake3::Hasher::new();
    hash.update(SAMPLE_FINGERPRINT_SCHEMA);
    hash.update(&(samples.len() as u64).to_le_bytes());
    for sample in samples {
        hash.update(&(sample.input.len() as u64).to_le_bytes());
        for token in &sample.input {
            hash.update(&token.to_le_bytes());
        }
        hash.update(&sample.target.to_le_bytes());
    }
    hex::encode(hash.finalize().as_bytes())
}

fn reply_sample_fingerprint(samples: &[MemoryWeightSample]) -> String {
    let mut hash = blake3::Hasher::new();
    hash.update(b"uor-r4.cpu-checkpoint-grade-replies/1\0");
    hash.update(&(samples.len() as u64).to_le_bytes());
    for sample in samples {
        hash.update(&(sample.prompt.len() as u64).to_le_bytes());
        for token in &sample.prompt {
            hash.update(&token.to_le_bytes());
        }
        hash.update(&(sample.reply.len() as u64).to_le_bytes());
        for token in &sample.reply {
            hash.update(&token.to_le_bytes());
        }
    }
    hex::encode(hash.finalize().as_bytes())
}

fn grade_rows(
    artifact: &[u8],
    model: &IntegerStackModel,
    samples: &[RankMarginSample],
    selected_from: usize,
    margin_q12: i32,
) -> Result<RowGrade> {
    let shape = model.shape();
    let fingerprint = sample_fingerprint(samples);
    if shape.head_nearest() {
        let forward = CpuFbsForward::from_artifact(artifact)
            .map_err(|error| invalid(format!("nearest grading forward: {error}")))?;
        let rows = samples
            .par_iter()
            .map_init(
                || forward.clone(),
                |forward, sample| {
                    forward.reset();
                    let mut step = None;
                    for (index, &token) in sample.input.iter().enumerate() {
                        if index + 1 == sample.input.len() {
                            step = Some(forward.step(token, CpuFbsHead::Nearest).map_err(
                                |error| invalid(format!("nearest grading step: {error}")),
                            )?);
                        } else {
                            forward.step(token, CpuFbsHead::Writer).map_err(|error| {
                                invalid(format!("nearest grading step: {error}"))
                            })?;
                        }
                    }
                    let chosen = step
                        .ok_or_else(|| invalid("nearest grading sample has an empty input"))?
                        .token;
                    Ok(chosen == sample.target)
                },
            )
            .collect::<Result<Vec<_>>>()?;
        let top1_correct = rows.iter().filter(|&&hit| hit).count();
        Ok(RowGrade {
            evaluation: CheckpointGradeEvaluation {
                head: "nearest".to_owned(),
                samples: samples.len(),
                selected_from,
                sample_fingerprint: fingerprint,
                top1_correct,
                top8_correct: None,
                target_admitted: None,
                export: None,
            },
            top1_rows: rows,
        })
    } else if shape.head_native_vector() {
        let mut panel = CpuPanelEvaluator::from_artifact(artifact, samples.to_vec(), margin_q12)?;
        let reading = panel.evaluate()?;
        if !reading.export.is_exact() {
            return Err(invalid(
                "native checkpoint-grade CPU/serving export differs",
            ));
        }
        Ok(RowGrade {
            evaluation: CheckpointGradeEvaluation {
                head: "native_writer".to_owned(),
                samples: reading.samples,
                selected_from,
                sample_fingerprint: fingerprint,
                top1_correct: reading.top1_correct,
                top8_correct: Some(reading.top8_correct),
                target_admitted: Some(reading.target_admitted),
                export: Some(reading.export),
            },
            top1_rows: Vec::new(),
        })
    } else {
        Err(invalid(
            "checkpoint grading supports only nearest and native-writer heads",
        ))
    }
}

fn grade_replies(
    artifact: &[u8],
    corpus: &CpuMaskedCorpus,
    config: CheckpointGradeConfig,
) -> Result<(TimedReplyReading, Vec<MemoryWeightSample>)> {
    let samples = corpus
        .conversation_examples_capped(config.max_reply_examples)?
        .into_iter()
        .map(|example| MemoryWeightSample {
            prompt: example.prompt,
            reply: example.reply,
        })
        .collect::<Vec<_>>();
    let reading = evaluate_replies(artifact, &samples, config.reply_tokens)?;
    Ok((reading, samples))
}

fn delta(after: usize, before: usize) -> i64 {
    i64::try_from(after)
        .unwrap_or(i64::MAX)
        .saturating_sub(i64::try_from(before).unwrap_or(i64::MAX))
}

/// Grade one exported artifact, optionally against a baseline artifact on the
/// exact same sampled rows and replies.
pub fn grade_checkpoint_artifact(
    artifact: &[u8],
    baseline: Option<&[u8]>,
    token_bytes: &[u8],
    mask: &[u8],
    manifest: &[u8],
    config: CheckpointGradeConfig,
) -> Result<CheckpointGradeReport> {
    let started = Instant::now();
    let config = config.validate()?;
    let identity = validate_corpus_manifest(manifest, token_bytes, mask)?;
    let model = IntegerStackModel::parse(artifact)
        .map_err(|error| invalid(format!("artifact: {error}")))?;
    let shape = model.shape();
    if shape.vocab != identity.vocab || identity.bos != 0 {
        return Err(invalid(
            "grading corpus vocabulary or BOS differs from the artifact",
        ));
    }
    identity.validate_context(shape.context)?;
    let corpus =
        CpuMaskedCorpus::from_bytes(token_bytes, mask, shape.vocab, shape.context, identity.bos)?;
    let samples = corpus.causal_samples_capped(config.max_evaluation_rows)?;
    let selected_from = corpus.causal_target_count();
    let graded = grade_rows(artifact, &model, &samples, selected_from, config.margin_q12)?;

    let audit_count = identity.tokens.min(shape.context);
    let audit_tokens = read_uort_token_slice(token_bytes, 0, audit_count)
        .map_err(|error| invalid(format!("grading audit tokens: {error}")))?;
    let artifact_sha256 = uor_r4_discrete::corpus::sha256(artifact);
    let audit = audit_served_forward(
        artifact,
        &ServedForwardAuditConfig {
            tokens: audit_tokens.tokens,
            token_source: Some(audit_tokens.source),
            expected_artifact_sha256: Some(artifact_sha256.clone()),
            reply_start: None,
            measure_timing: false,
        },
    )
    .map_err(|error| invalid(format!("grading audit: {error}")))?;
    if audit.parity.instrument_fault
        || audit.parity.query_coordinate_mismatches != 0
        || audit.parity.token_mismatches != 0
        || audit.parity.kept_cell_mismatches != 0
    {
        return Err(invalid("checkpoint grading served-forward parity differs"));
    }

    let (replies, reply_samples) = grade_replies(artifact, &corpus, config)?;
    let reply_sample_fingerprint = reply_sample_fingerprint(&reply_samples);
    let comparison = if let Some(bytes) = baseline {
        let baseline_model = IntegerStackModel::parse(bytes)
            .map_err(|error| invalid(format!("baseline artifact: {error}")))?;
        let baseline_shape = baseline_model.shape();
        if baseline_shape.vocab != shape.vocab || baseline_shape.context != shape.context {
            return Err(invalid(
                "baseline artifact shape differs from the graded artifact",
            ));
        }
        let baseline_rows = grade_rows(
            bytes,
            &baseline_model,
            &samples,
            selected_from,
            config.margin_q12,
        )?;
        if baseline_rows.evaluation.sample_fingerprint != graded.evaluation.sample_fingerprint {
            return Err(invalid(
                "baseline evaluation did not use the selected sample rows",
            ));
        }
        let paired_top1 = if !graded.top1_rows.is_empty() && !baseline_rows.top1_rows.is_empty() {
            if graded.top1_rows.len() != baseline_rows.top1_rows.len() {
                return Err(invalid("baseline top-1 row alignment differs"));
            }
            Some((
                graded
                    .top1_rows
                    .iter()
                    .zip(&baseline_rows.top1_rows)
                    .filter(|(after, before)| **after && !**before)
                    .count(),
                graded
                    .top1_rows
                    .iter()
                    .zip(&baseline_rows.top1_rows)
                    .filter(|(after, before)| !**after && **before)
                    .count(),
            ))
        } else {
            None
        };
        let baseline_replies = evaluate_replies(bytes, &reply_samples, config.reply_tokens)?;
        Some(CheckpointGradeBaseline {
            artifact_sha256: uor_r4_discrete::corpus::sha256(bytes),
            top1_delta: delta(
                graded.evaluation.top1_correct,
                baseline_rows.evaluation.top1_correct,
            ),
            top1_only_candidate: paired_top1.map(|(candidate, _)| candidate),
            top1_only_baseline: paired_top1.map(|(_, baseline)| baseline),
            top8_delta: graded
                .evaluation
                .top8_correct
                .zip(baseline_rows.evaluation.top8_correct)
                .map(|(after, before)| delta(after, before)),
            target_admitted_delta: graded
                .evaluation
                .target_admitted
                .zip(baseline_rows.evaluation.target_admitted)
                .map(|(after, before)| delta(after, before)),
            evaluation: baseline_rows.evaluation,
            replies: baseline_replies,
        })
    } else {
        None
    };

    Ok(CheckpointGradeReport {
        schema: CHECKPOINT_GRADE_SCHEMA,
        substrate: "cpu-development-research",
        qualification: "gpu-required",
        artifact_sha256,
        vocab: shape.vocab,
        context: shape.context,
        corpus: CheckpointGradeCorpus {
            split: identity.split,
            tokens_sha256: identity.tokens_sha256,
            mask_sha256: identity.mask_sha256,
            tokenizer_sha256: identity.tokenizer_sha256,
            tokens: identity.tokens,
            response_targets: identity.response_targets,
        },
        settings: CheckpointGradeSettings {
            max_evaluation_rows: config.max_evaluation_rows,
            max_reply_examples: config.max_reply_examples,
            reply_tokens: config.reply_tokens,
            margin_q12: config.margin_q12,
        },
        evaluation: graded.evaluation,
        reply_sample_fingerprint,
        replies,
        audit: CheckpointGradeAudit {
            positions_checked: audit.parity.positions_checked,
            query_coordinates_compared: audit.parity.query_coordinates_compared,
            query_coordinate_mismatches: audit.parity.query_coordinate_mismatches,
            token_decisions_compared: audit.parity.token_decisions_compared,
            token_mismatches: audit.parity.token_mismatches,
            kept_cells_compared: audit.parity.kept_cells_compared,
            kept_cell_mismatches: audit.parity.kept_cell_mismatches,
        },
        baseline: comparison,
        wall_elapsed_us: u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    const SYNTH: FbsSynth = FbsSynth {
        vocab: 256,
        d: 32,
        mlp: 32,
        layers: 1,
        context: 8,
        kept: 2,
        gain_lo: 0.75,
        gain_span: 1.5,
        table: 0.5,
        decay_step: 0.25,
    };

    fn corpus(vocab: u32, context: usize) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let tokens = [0u16, 2, 3, 4, 5, 0, 6, 7, 8, 9];
        let mask = [0u8, 0, 1, 1, 1, 0, 0, 1, 1, 1];
        let token_bytes = uort(&tokens, vocab);
        let manifest = serde_json::to_vec(&serde_json::json!({
            "schema": "uor-r4-chat-corpus/v1",
            "mask_schema": "uor-r4-response-mask/u8/v1",
            "split": "dev",
            "tokens": tokens.len(),
            "tokens_bytes": token_bytes.len(),
            "tokens_sha256": uor_r4_discrete::corpus::sha256(&token_bytes),
            "mask_bytes": mask.len(),
            "mask_sha256": uor_r4_discrete::corpus::sha256(&mask),
            "response_tokens": mask.iter().filter(|&&value| value == 1).count(),
            "max_tokens": context,
            "tokenizer": {
                "vocab_size": vocab,
                "bos_id": 0,
                "sha256": "00".repeat(32)
            }
        }))
        .unwrap();
        (token_bytes, mask.to_vec(), manifest)
    }

    fn uort(tokens: &[u16], vocab: u32) -> Vec<u8> {
        let mut bytes = vec![0u8; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&vocab.to_le_bytes());
        for token in tokens {
            bytes.extend_from_slice(&token.to_le_bytes());
        }
        bytes
    }

    fn artifact(vocab: usize, nearest: bool) -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(941, FbsSynth { vocab, ..SYNTH });
        snapshot.readout_nearest = nearest;
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn config() -> CheckpointGradeConfig {
        CheckpointGradeConfig {
            max_evaluation_rows: 3,
            max_reply_examples: 2,
            reply_tokens: 2,
            ..CheckpointGradeConfig::default()
        }
    }

    #[test]
    fn nearest_grading_is_bounded_and_repeats() {
        let artifact = artifact(256, true);
        let (tokens, mask, manifest) = corpus(256, 8);
        let first = grade_checkpoint_artifact(
            &artifact,
            Some(&artifact),
            &tokens,
            &mask,
            &manifest,
            config(),
        )
        .unwrap();
        let second = grade_checkpoint_artifact(
            &artifact,
            Some(&artifact),
            &tokens,
            &mask,
            &manifest,
            config(),
        )
        .unwrap();
        assert_eq!(first.schema, CHECKPOINT_GRADE_SCHEMA);
        assert_eq!(first.evaluation.head, "nearest");
        assert_eq!(first.evaluation.samples, 3);
        assert_eq!(first.evaluation.selected_from, 6);
        assert_eq!(
            first.settings,
            CheckpointGradeSettings {
                max_evaluation_rows: 3,
                max_reply_examples: 2,
                reply_tokens: 2,
                margin_q12: CheckpointGradeConfig::default().margin_q12,
            }
        );
        assert_eq!(first.evaluation.top8_correct, None);
        assert_eq!(first.audit.query_coordinate_mismatches, 0);
        assert_eq!(first.audit.token_mismatches, 0);
        assert_eq!(first.audit.kept_cell_mismatches, 0);
        let baseline = first.baseline.as_ref().unwrap();
        assert_eq!(baseline.top1_delta, 0);
        assert_eq!(baseline.top1_only_candidate, Some(0));
        assert_eq!(baseline.top1_only_baseline, Some(0));
        assert_eq!(baseline.top8_delta, None);
        assert_eq!(baseline.target_admitted_delta, None);
        assert_eq!(first.replies.reading, second.replies.reading);
        assert_eq!(first.replies.reply_sha256, second.replies.reply_sha256);
        assert_eq!(
            first.reply_sample_fingerprint,
            second.reply_sample_fingerprint
        );
        assert_eq!(first.evaluation, second.evaluation);
    }

    #[test]
    fn native_writer_grading_carries_top8_and_admission() {
        let artifact = artifact(4_096, false);
        let (tokens, mask, manifest) = corpus(4_096, 8);
        let report =
            grade_checkpoint_artifact(&artifact, None, &tokens, &mask, &manifest, config())
                .unwrap();
        assert_eq!(report.evaluation.head, "native_writer");
        assert_eq!(report.evaluation.samples, 3);
        assert!(report.evaluation.top8_correct.is_some());
        assert!(report.evaluation.target_admitted.is_some());
        assert!(report.evaluation.export.as_ref().unwrap().is_exact());
        assert_eq!(report.audit.query_coordinate_mismatches, 0);
        assert_eq!(report.audit.token_mismatches, 0);
        assert_eq!(report.audit.kept_cell_mismatches, 0);
        assert!(report.baseline.is_none());
    }

    #[test]
    fn invalid_bounds_are_refused_before_grading() {
        let artifact = artifact(256, true);
        let (tokens, mask, manifest) = corpus(256, 8);
        let error = grade_checkpoint_artifact(
            &artifact,
            None,
            &tokens,
            &mask,
            &manifest,
            CheckpointGradeConfig {
                max_evaluation_rows: 0,
                ..config()
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("bounds are invalid"));
    }
}
