//! Fail-closed exact source-span answers selected from #1017 R4/Spin states.
//!
//! This surface does not ask the language-model decoder to invent an answer.
//! The current policy encodes every admitted source sentence relative to the
//! exact question through the established six-layer coherent R4/Spin
//! causal-softmax executor, applies one learned relation head, and either copies
//! one original UTF-8 byte span or returns a typed abstention/conflict terminal.
//! The failed historical cosine pointer remains readable only for replay of its
//! already-recorded evidence.

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uor_r4_model_source::{
    HuggingFaceLlamaOracle, TeacherExecutionConfig, TeacherExecutionSnapshot, TeacherOracle,
};

use crate::r4_softmax_local_generation::{
    encode_r4_softmax_local_states, LocalCheckpointBinding, R4SoftmaxLocalGenerationError,
    R4SoftmaxLocalStateEncodingConfig, R4SoftmaxLocalStateSequenceAudit, SourceReadAudit,
};
use crate::r4_softmax_reference_generation::ModelShape;
use crate::r4_source_relation_head::{
    evaluate_attended_relation_adapter, evaluate_source_relation_head,
    load_attended_relation_adapter, load_source_relation_head, render_attended_relation_input,
    render_relation_input, AttendedRelationAdapterBinding, SourceRelationCandidate,
    SourceRelationHeadBinding, SourceRelationHeadDecision, SourceRelationHeadError,
    SourceRelationHeadEvaluation, ATTENDED_RELATION_ADAPTER_POLICY,
    ATTENDED_RELATION_ADAPTER_SCHEMA, ATTENDED_RELATION_INPUT_POLICY,
    ATTENDED_RELATION_SCORING_POLICY, ATTENDED_RELATION_UPDATE_NONE, RELATION_HEAD_ARTIFACT_SCHEMA,
    RELATION_HEAD_POLICY, RELATION_INPUT_POLICY,
};
use crate::r4_source_span_pointer::{
    evaluate_source_span_pointer, load_source_span_pointer, SourceSpanPointerBinding,
    SourceSpanPointerDecision, SourceSpanPointerError, SourceSpanPointerEvaluation,
    POINTER_ARTIFACT_SCHEMA, POINTER_POLICY,
};

pub const POINTER_REPORT_SCHEMA: &str = "uor-r4.grounded-answer/2";
pub const RELATION_REPORT_SCHEMA: &str = "uor-r4.grounded-answer/3";
pub const ATTENDED_RELATION_REPORT_SCHEMA: &str = "uor-r4.grounded-answer/4";
pub const FIXED_VERBALIZER_AUDIT_SCHEMA: &str = "uor-r4.fixed-verbalizer-audit/1";
pub const MAX_SOURCE_BYTES: usize = 4 * 1024;
pub const MAX_QUESTION_BYTES: usize = 1024;
pub const MAX_SOURCE_SPANS: usize = 8;

#[derive(Clone, Debug)]
pub struct GroundedAnswerConfig {
    pub model: PathBuf,
    pub head: PathBuf,
    pub source_file: PathBuf,
    pub question: String,
    pub workers: NonZeroUsize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundingSourceBinding {
    pub path: String,
    pub source_cid: String,
    pub byte_length: usize,
    pub regular_non_symlink: bool,
    pub utf8: bool,
    pub reads: u64,
    pub unchanged_after_run: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpanCandidateBinding {
    pub candidate_index: usize,
    pub source_span: SourceSpan,
    pub text_cid: String,
    pub state_cid: String,
    pub content_token_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_input_text_cid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_state_cid: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroundedStateEncodingAudit {
    pub schema: String,
    pub audit_cid: String,
    pub model_weights_cid: String,
    pub tokenizer_cid: String,
    pub hidden_size: usize,
    pub checkpoint: LocalCheckpointBinding,
    pub model_shape: ModelShape,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_text_cid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_state_cid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_content_token_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_input_policy: Option<String>,
    pub sequence_audits: Vec<R4SoftmaxLocalStateSequenceAudit>,
    pub source_read_audit: SourceReadAudit,
    pub execution: TeacherExecutionSnapshot,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroundedPointerLogits {
    pub answer: f32,
    pub abstain: f32,
    pub conflict: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroundedPointerEvaluation {
    pub candidate_scores: Vec<f32>,
    pub ranked_candidate_indices: Vec<usize>,
    pub logits: GroundedPointerLogits,
    pub decision: String,
    pub selected_span_index: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroundedRelationEvaluation {
    pub candidate_logits: Vec<f32>,
    pub positive_candidate_indices: Vec<usize>,
    pub positive_unique_span_count: usize,
    pub decision: String,
    pub selected_span_index: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundedFixedVerbalizerAudit {
    pub schema: String,
    pub scoring_policy: String,
    pub source_model_weights_cid: String,
    pub supported_token_id: u32,
    pub unsupported_token_id: u32,
    pub row_width: usize,
    pub supported_token_row_cid: String,
    pub unsupported_token_row_cid: String,
    pub oracle_loads: u64,
    pub checkpoint_identity_reads: u64,
    pub checkpoint_identity_unchanged: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroundedAbstentionReason {
    PointerAbstained,
    RelationNoSupport,
}

impl GroundedAbstentionReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PointerAbstained => "pointer_abstained",
            Self::RelationNoSupport => "relation_no_support",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum GroundedAnswerOutcome {
    Answered {
        answer: String,
        source_span: SourceSpan,
    },
    Contradiction,
    Abstained {
        reason: GroundedAbstentionReason,
    },
}

#[derive(Serialize)]
struct GroundedDecisionIdentity<'a> {
    schema: &'static str,
    pointer_policy: &'static str,
    source_cid: &'a str,
    source_byte_length: usize,
    question: &'a str,
    subject: &'a str,
    pointer_artifact_cid: &'a str,
    state_encoding_audit_cid: &'a str,
    candidate_spans: &'a [SourceSpanCandidateBinding],
    pointer_evaluation: &'a GroundedPointerEvaluation,
    outcome: &'a GroundedAnswerOutcome,
}

#[derive(Serialize)]
struct GroundedRelationDecisionIdentity<'a> {
    schema: &'static str,
    relation_policy: &'static str,
    relation_input_policy: &'static str,
    source_cid: &'a str,
    source_byte_length: usize,
    question: &'a str,
    relation_artifact_cid: &'a str,
    state_encoding_audit_cid: &'a str,
    candidate_spans: &'a [SourceSpanCandidateBinding],
    relation_evaluation: &'a GroundedRelationEvaluation,
    outcome: &'a GroundedAnswerOutcome,
}

#[derive(Serialize)]
struct GroundedAttendedRelationDecisionIdentity<'a> {
    schema: &'static str,
    relation_policy: &'static str,
    relation_input_policy: &'static str,
    source_cid: &'a str,
    source_byte_length: usize,
    question: &'a str,
    relation_artifact_cid: &'a str,
    relation_admission: &'a str,
    representation_update: &'a str,
    checkpoint_tree_cid: &'a str,
    model_weights_cid: &'a str,
    config_cid: &'a str,
    tokenizer_cid: &'a str,
    state_encoding_audit_cid: &'a str,
    fixed_verbalizer: &'a GroundedFixedVerbalizerAudit,
    candidate_spans: &'a [SourceSpanCandidateBinding],
    relation_evaluation: &'a GroundedRelationEvaluation,
    outcome: &'a GroundedAnswerOutcome,
}

#[derive(Serialize)]
pub struct GroundedAnswerReport {
    pub schema: String,
    pub decision_cid: String,
    pub claim_scope: String,
    pub source: GroundingSourceBinding,
    pub question: String,
    pub subject: String,
    pub candidate_spans: Vec<SourceSpanCandidateBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<SourceSpanPointerBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer_evaluation: Option<GroundedPointerEvaluation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<SourceRelationHeadBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attended_relation: Option<AttendedRelationAdapterBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_evaluation: Option<GroundedRelationEvaluation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_verbalizer: Option<GroundedFixedVerbalizerAudit>,
    /// Compact audit only. Raw width-288 state vectors are deliberately not
    /// repeated in the public report, but their content CIDs remain bound.
    pub state_encoding: GroundedStateEncodingAudit,
    pub outcome: GroundedAnswerOutcome,
    pub nonclaims: Vec<String>,
}

#[derive(Debug)]
pub enum GroundedAnswerError {
    InvalidRequest(String),
    InvalidSource(String),
    StateEncoding(R4SoftmaxLocalGenerationError),
    Pointer(SourceSpanPointerError),
    Relation(SourceRelationHeadError),
    Audit(String),
    Io(io::Error),
}

impl fmt::Display for GroundedAnswerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(reason) => write!(formatter, "invalid grounded answer: {reason}"),
            Self::InvalidSource(reason) => write!(formatter, "invalid grounding source: {reason}"),
            Self::StateEncoding(error) => error.fmt(formatter),
            Self::Pointer(error) => error.fmt(formatter),
            Self::Relation(error) => error.fmt(formatter),
            Self::Audit(reason) => write!(formatter, "grounded answer audit failed: {reason}"),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for GroundedAnswerError {}

impl From<R4SoftmaxLocalGenerationError> for GroundedAnswerError {
    fn from(error: R4SoftmaxLocalGenerationError) -> Self {
        Self::StateEncoding(error)
    }
}

impl From<SourceSpanPointerError> for GroundedAnswerError {
    fn from(error: SourceSpanPointerError) -> Self {
        Self::Pointer(error)
    }
}

impl From<SourceRelationHeadError> for GroundedAnswerError {
    fn from(error: SourceRelationHeadError) -> Self {
        Self::Relation(error)
    }
}

impl From<io::Error> for GroundedAnswerError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn run_grounded_answer(
    config: &GroundedAnswerConfig,
) -> Result<GroundedAnswerReport, GroundedAnswerError> {
    validate_request(config)?;
    let head_bytes = std::fs::read(&config.head).map_err(|error| {
        GroundedAnswerError::InvalidRequest(format!(
            "cannot read answer head {}: {error}",
            config.head.display()
        ))
    })?;
    let head_value: serde_json::Value = serde_json::from_slice(&head_bytes).map_err(|error| {
        GroundedAnswerError::InvalidRequest(format!(
            "cannot decode answer head {}: {error}",
            config.head.display()
        ))
    })?;
    match head_value.get("schema").and_then(serde_json::Value::as_str) {
        Some(ATTENDED_RELATION_ADAPTER_SCHEMA) => run_attended_relation_grounded_answer(config),
        Some(RELATION_HEAD_ARTIFACT_SCHEMA) => run_relation_grounded_answer(config),
        Some(POINTER_ARTIFACT_SCHEMA) => run_pointer_grounded_answer(config),
        Some(schema) => Err(GroundedAnswerError::InvalidRequest(format!(
            "unsupported answer-head schema {schema:?}"
        ))),
        None => Err(GroundedAnswerError::InvalidRequest(
            "answer head has no string schema".to_owned(),
        )),
    }
}

fn run_pointer_grounded_answer(
    config: &GroundedAnswerConfig,
) -> Result<GroundedAnswerReport, GroundedAnswerError> {
    validate_request(config)?;
    let source_before = read_source_file(&config.source_file)?;
    let source_text = std::str::from_utf8(&source_before).map_err(|error| {
        GroundedAnswerError::InvalidSource(format!(
            "{} is not exact UTF-8: {error}",
            config.source_file.display()
        ))
    })?;
    let head_before = std::fs::read(&config.head).map_err(|error| {
        GroundedAnswerError::InvalidRequest(format!(
            "cannot read pointer head {}: {error}",
            config.head.display()
        ))
    })?;
    let subject = parse_subject(&config.question)?;
    let sentence_spans = split_sentence_spans(source_text)?;

    let mut sequences = Vec::with_capacity(sentence_spans.len() + 1);
    sequences.push(subject.clone());
    sequences.extend(sentence_spans.iter().map(|(_, text)| (*text).to_owned()));
    let encoded = encode_r4_softmax_local_states(&R4SoftmaxLocalStateEncodingConfig {
        model: config.model.clone(),
        sequences,
        workers: config.workers,
    })?;
    if encoded.sequences.len() != sentence_spans.len() + 1 {
        return Err(GroundedAnswerError::Audit(
            "state encoder returned the wrong independent-sequence count".to_owned(),
        ));
    }

    let pointer = load_source_span_pointer(
        &config.head,
        &encoded.checkpoint.weights_cid,
        &encoded.checkpoint.tokenizer_cid,
    )?;
    if pointer.artifact.hidden_size != encoded.model_shape.dimension {
        return Err(GroundedAnswerError::Audit(format!(
            "pointer width {} differs from model width {}",
            pointer.artifact.hidden_size, encoded.model_shape.dimension
        )));
    }

    let subject_states = encoded.sequences[0].final_normalized_residuals.clone();
    let candidate_states = encoded.sequences[1..]
        .iter()
        .map(|sequence| sequence.final_normalized_residuals.clone())
        .collect::<Vec<_>>();
    let internal_evaluation =
        evaluate_source_span_pointer(&pointer.artifact, &subject_states, &candidate_states)?;
    let pointer_evaluation = public_pointer_evaluation(&internal_evaluation);

    let candidate_spans = sentence_spans
        .iter()
        .zip(&encoded.sequences[1..])
        .enumerate()
        .map(|(candidate_index, ((source_span, text), sequence))| {
            if sequence.text_cid != raw_cid(text.as_bytes()) {
                return Err(GroundedAnswerError::Audit(format!(
                    "candidate {candidate_index} state text CID does not match its exact source span"
                )));
            }
            Ok(SourceSpanCandidateBinding {
                candidate_index,
                source_span: *source_span,
                text_cid: sequence.text_cid.clone(),
                state_cid: sequence.audit.state_cid.clone(),
                content_token_count: sequence.audit.content_token_count,
                relation_input_text_cid: None,
                terminal_state_cid: None,
            })
        })
        .collect::<Result<Vec<_>, GroundedAnswerError>>()?;

    let outcome = match internal_evaluation.decision {
        SourceSpanPointerDecision::Answer { candidate_index } => {
            let candidate = sentence_spans.get(candidate_index).ok_or_else(|| {
                GroundedAnswerError::Audit("pointer selected an absent source span".to_owned())
            })?;
            GroundedAnswerOutcome::Answered {
                answer: candidate.1.to_owned(),
                source_span: candidate.0,
            }
        }
        SourceSpanPointerDecision::Abstain => GroundedAnswerOutcome::Abstained {
            reason: GroundedAbstentionReason::PointerAbstained,
        },
        SourceSpanPointerDecision::Contradiction => GroundedAnswerOutcome::Contradiction,
    };

    let source_after = read_source_file(&config.source_file)?;
    let source_cid = raw_cid(&source_before);
    if source_before != source_after {
        return Err(GroundedAnswerError::Audit(format!(
            "source {} changed during selection (before {}, after {})",
            config.source_file.display(),
            source_cid,
            raw_cid(&source_after)
        )));
    }
    let head_after = std::fs::read(&config.head).map_err(|error| {
        GroundedAnswerError::Audit(format!(
            "cannot rescan pointer head {}: {error}",
            config.head.display()
        ))
    })?;
    if head_before != head_after {
        return Err(GroundedAnswerError::Audit(format!(
            "pointer head {} changed during selection",
            config.head.display()
        )));
    }

    let source = GroundingSourceBinding {
        path: config.source_file.display().to_string(),
        source_cid,
        byte_length: source_before.len(),
        regular_non_symlink: true,
        utf8: true,
        reads: 2,
        unchanged_after_run: true,
    };
    let sequence_audits = encoded
        .sequences
        .iter()
        .map(|sequence| sequence.audit.clone())
        .collect::<Vec<_>>();
    let state_encoding = GroundedStateEncodingAudit {
        schema: encoded.schema,
        audit_cid: encoded.audit_cid,
        model_weights_cid: encoded.checkpoint.weights_cid.clone(),
        tokenizer_cid: encoded.checkpoint.tokenizer_cid.clone(),
        hidden_size: encoded.model_shape.dimension,
        checkpoint: encoded.checkpoint,
        model_shape: encoded.model_shape,
        subject_text_cid: Some(encoded.sequences[0].text_cid.clone()),
        subject_state_cid: Some(encoded.sequences[0].audit.state_cid.clone()),
        subject_content_token_count: Some(encoded.sequences[0].audit.content_token_count),
        relation_input_policy: None,
        sequence_audits,
        source_read_audit: encoded.source_read_audit,
        execution: encoded.execution,
    };
    let decision_cid = cid_serializable(&GroundedDecisionIdentity {
        schema: POINTER_REPORT_SCHEMA,
        pointer_policy: POINTER_POLICY,
        source_cid: &source.source_cid,
        source_byte_length: source.byte_length,
        question: &config.question,
        subject: &subject,
        pointer_artifact_cid: &pointer.binding.artifact_cid,
        state_encoding_audit_cid: &state_encoding.audit_cid,
        candidate_spans: &candidate_spans,
        pointer_evaluation: &pointer_evaluation,
        outcome: &outcome,
    })?;

    Ok(GroundedAnswerReport {
        schema: POINTER_REPORT_SCHEMA.to_owned(),
        decision_cid,
        claim_scope: "one learned source-span pointer over frozen #1017 all-layer coherent R4/Spin causal-softmax states; output is an exact original source sentence or a typed non-answer".to_owned(),
        source,
        question: config.question.clone(),
        subject,
        candidate_spans,
        pointer: Some(pointer.binding),
        pointer_evaluation: Some(pointer_evaluation),
        relation: None,
        attended_relation: None,
        relation_evaluation: None,
        fixed_verbalizer: None,
        state_encoding,
        outcome,
        nonclaims: vec![
            "This bounded pointer result does not establish open-domain question answering, reasoning, or general semantic entailment.".to_owned(),
            "The learned head and #1017 executor remain source-backed, floating-point, multiplication-using, and ordinary-softmax based.".to_owned(),
            "This result does not establish the final source-free exact geometric runtime or a browser product surface.".to_owned(),
        ],
    })
}

fn run_relation_grounded_answer(
    config: &GroundedAnswerConfig,
) -> Result<GroundedAnswerReport, GroundedAnswerError> {
    let source_before = read_source_file(&config.source_file)?;
    let source_text = std::str::from_utf8(&source_before).map_err(|error| {
        GroundedAnswerError::InvalidSource(format!(
            "{} is not exact UTF-8: {error}",
            config.source_file.display()
        ))
    })?;
    let head_before = std::fs::read(&config.head).map_err(|error| {
        GroundedAnswerError::InvalidRequest(format!(
            "cannot read source-relative relation head {}: {error}",
            config.head.display()
        ))
    })?;
    let subject = parse_subject(&config.question)?;
    let sentence_spans = split_sentence_spans(source_text)?;
    if sentence_spans.len() < 2 {
        return Err(GroundedAnswerError::InvalidSource(
            "source-relative relation selection requires 2..=8 sentence spans".to_owned(),
        ));
    }

    let sequences = sentence_spans
        .iter()
        .map(|(_, text)| render_relation_input(text, &config.question))
        .collect::<Vec<_>>();
    let encoded = encode_r4_softmax_local_states(&R4SoftmaxLocalStateEncodingConfig {
        model: config.model.clone(),
        sequences,
        workers: config.workers,
    })?;
    if encoded.sequences.len() != sentence_spans.len() {
        return Err(GroundedAnswerError::Audit(
            "relation state encoder returned the wrong independent-sequence count".to_owned(),
        ));
    }

    let relation = load_source_relation_head(
        &config.head,
        &encoded.checkpoint.weights_cid,
        &encoded.checkpoint.tokenizer_cid,
    )?;
    if relation.artifact.hidden_size != encoded.model_shape.dimension {
        return Err(GroundedAnswerError::Audit(format!(
            "relation-head width {} differs from model width {}",
            relation.artifact.hidden_size, encoded.model_shape.dimension
        )));
    }

    let terminal_states = encoded
        .sequences
        .iter()
        .enumerate()
        .map(|(candidate_index, sequence)| {
            sequence
                .final_normalized_residuals
                .last()
                .map(Vec::as_slice)
                .ok_or_else(|| {
                    GroundedAnswerError::Audit(format!(
                        "candidate {candidate_index} relation input emitted no terminal state"
                    ))
                })
        })
        .collect::<Result<Vec<_>, GroundedAnswerError>>()?;
    let relation_candidates = sentence_spans
        .iter()
        .zip(&terminal_states)
        .map(|((source_span, text), state)| SourceRelationCandidate {
            span_text: text,
            byte_start: source_span.byte_start,
            final_relation_state: state,
        })
        .collect::<Vec<_>>();
    let internal_evaluation =
        evaluate_source_relation_head(&relation.artifact, &relation_candidates)?;
    let relation_evaluation = public_relation_evaluation(
        &internal_evaluation,
        &sentence_spans,
        relation.artifact.threshold,
    );

    let candidate_spans = sentence_spans
        .iter()
        .zip(&encoded.sequences)
        .zip(&terminal_states)
        .enumerate()
        .map(
            |(candidate_index, (((source_span, text), sequence), terminal_state))| {
                let relation_input = render_relation_input(text, &config.question);
                let relation_input_text_cid = raw_cid(relation_input.as_bytes());
                if sequence.text_cid != relation_input_text_cid {
                    return Err(GroundedAnswerError::Audit(format!(
                        "candidate {candidate_index} state text CID does not match the exact relation input"
                    )));
                }
                Ok(SourceSpanCandidateBinding {
                    candidate_index,
                    source_span: *source_span,
                    text_cid: raw_cid(text.as_bytes()),
                    state_cid: sequence.audit.state_cid.clone(),
                    content_token_count: sequence.audit.content_token_count,
                    relation_input_text_cid: Some(relation_input_text_cid),
                    terminal_state_cid: Some(cid_serializable(terminal_state)?),
                })
            },
        )
        .collect::<Result<Vec<_>, GroundedAnswerError>>()?;

    let outcome = match internal_evaluation.decision {
        SourceRelationHeadDecision::Answer { candidate_index } => {
            let candidate = sentence_spans.get(candidate_index).ok_or_else(|| {
                GroundedAnswerError::Audit(
                    "relation head selected an absent source span".to_owned(),
                )
            })?;
            GroundedAnswerOutcome::Answered {
                answer: candidate.1.to_owned(),
                source_span: candidate.0,
            }
        }
        SourceRelationHeadDecision::Abstain => GroundedAnswerOutcome::Abstained {
            reason: GroundedAbstentionReason::RelationNoSupport,
        },
        SourceRelationHeadDecision::Contradiction => GroundedAnswerOutcome::Contradiction,
    };

    let source_after = read_source_file(&config.source_file)?;
    let source_cid = raw_cid(&source_before);
    if source_before != source_after {
        return Err(GroundedAnswerError::Audit(format!(
            "source {} changed during selection (before {}, after {})",
            config.source_file.display(),
            source_cid,
            raw_cid(&source_after)
        )));
    }
    let head_after = std::fs::read(&config.head).map_err(|error| {
        GroundedAnswerError::Audit(format!(
            "cannot rescan source-relative relation head {}: {error}",
            config.head.display()
        ))
    })?;
    if head_before != head_after {
        return Err(GroundedAnswerError::Audit(format!(
            "source-relative relation head {} changed during selection",
            config.head.display()
        )));
    }

    let source = GroundingSourceBinding {
        path: config.source_file.display().to_string(),
        source_cid,
        byte_length: source_before.len(),
        regular_non_symlink: true,
        utf8: true,
        reads: 2,
        unchanged_after_run: true,
    };
    let sequence_audits = encoded
        .sequences
        .iter()
        .map(|sequence| sequence.audit.clone())
        .collect::<Vec<_>>();
    let state_encoding = GroundedStateEncodingAudit {
        schema: encoded.schema,
        audit_cid: encoded.audit_cid,
        model_weights_cid: encoded.checkpoint.weights_cid.clone(),
        tokenizer_cid: encoded.checkpoint.tokenizer_cid.clone(),
        hidden_size: encoded.model_shape.dimension,
        checkpoint: encoded.checkpoint,
        model_shape: encoded.model_shape,
        subject_text_cid: None,
        subject_state_cid: None,
        subject_content_token_count: None,
        relation_input_policy: Some(RELATION_INPUT_POLICY.to_owned()),
        sequence_audits,
        source_read_audit: encoded.source_read_audit,
        execution: encoded.execution,
    };
    let decision_cid = cid_serializable(&GroundedRelationDecisionIdentity {
        schema: RELATION_REPORT_SCHEMA,
        relation_policy: RELATION_HEAD_POLICY,
        relation_input_policy: RELATION_INPUT_POLICY,
        source_cid: &source.source_cid,
        source_byte_length: source.byte_length,
        question: &config.question,
        relation_artifact_cid: &relation.binding.artifact_cid,
        state_encoding_audit_cid: &state_encoding.audit_cid,
        candidate_spans: &candidate_spans,
        relation_evaluation: &relation_evaluation,
        outcome: &outcome,
    })?;

    Ok(GroundedAnswerReport {
        schema: RELATION_REPORT_SCHEMA.to_owned(),
        decision_cid,
        claim_scope: "one learned source-relative relation head over frozen #1017 all-layer coherent R4/Spin causal-softmax states; output is an exact original source sentence or a typed non-answer".to_owned(),
        source,
        question: config.question.clone(),
        subject,
        candidate_spans,
        pointer: None,
        pointer_evaluation: None,
        relation: Some(relation.binding),
        attended_relation: None,
        relation_evaluation: Some(relation_evaluation),
        fixed_verbalizer: None,
        state_encoding,
        outcome,
        nonclaims: vec![
            "This bounded source-relative result does not establish open-domain question answering, reasoning, or general semantic entailment.".to_owned(),
            "The learned head and #1017 executor remain source-backed, floating-point, multiplication-using, and ordinary-softmax based.".to_owned(),
            "This result does not establish the final source-free exact geometric runtime or a browser product surface.".to_owned(),
        ],
    })
}

fn run_attended_relation_grounded_answer(
    config: &GroundedAnswerConfig,
) -> Result<GroundedAnswerReport, GroundedAnswerError> {
    let source_before = read_source_file(&config.source_file)?;
    let source_text = std::str::from_utf8(&source_before).map_err(|error| {
        GroundedAnswerError::InvalidSource(format!(
            "{} is not exact UTF-8: {error}",
            config.source_file.display()
        ))
    })?;
    let head_before = std::fs::read(&config.head).map_err(|error| {
        GroundedAnswerError::InvalidRequest(format!(
            "cannot read attended relation adapter {}: {error}",
            config.head.display()
        ))
    })?;
    let subject = parse_subject(&config.question)?;
    let sentence_spans = split_sentence_spans(source_text)?;
    if sentence_spans.len() < 2 {
        return Err(GroundedAnswerError::InvalidSource(
            "attended relation selection requires 2..=8 sentence spans".to_owned(),
        ));
    }

    let sequences = sentence_spans
        .iter()
        .map(|(_, text)| render_attended_relation_input(text, &config.question))
        .collect::<Vec<_>>();
    let encoded = encode_r4_softmax_local_states(&R4SoftmaxLocalStateEncodingConfig {
        model: config.model.clone(),
        sequences,
        workers: config.workers,
    })?;
    if encoded.sequences.len() != sentence_spans.len() {
        return Err(GroundedAnswerError::Audit(
            "attended relation state encoder returned the wrong independent-sequence count"
                .to_owned(),
        ));
    }

    let relation = load_attended_relation_adapter(
        &config.head,
        &encoded.checkpoint.weights_cid,
        &encoded.checkpoint.checkpoint_tree_cid,
        &encoded.checkpoint.config_cid,
        &encoded.checkpoint.tokenizer_cid,
    )?;
    if relation.artifact.hidden_size != encoded.model_shape.dimension {
        return Err(GroundedAnswerError::Audit(format!(
            "attended relation width {} differs from model width {}",
            relation.artifact.hidden_size, encoded.model_shape.dimension
        )));
    }
    let (supported_token_row, unsupported_token_row, fixed_verbalizer) =
        load_fixed_verbalizer_rows(
            config,
            &encoded.checkpoint,
            &encoded.model_shape,
            relation.artifact.supported_token_id,
            relation.artifact.unsupported_token_id,
        )?;

    let terminal_states = encoded
        .sequences
        .iter()
        .enumerate()
        .map(|(candidate_index, sequence)| {
            sequence
                .final_normalized_residuals
                .last()
                .map(Vec::as_slice)
                .ok_or_else(|| {
                    GroundedAnswerError::Audit(format!(
                        "candidate {candidate_index} attended relation input emitted no terminal state"
                    ))
                })
        })
        .collect::<Result<Vec<_>, GroundedAnswerError>>()?;
    let relation_candidates = sentence_spans
        .iter()
        .zip(&terminal_states)
        .map(|((source_span, text), state)| SourceRelationCandidate {
            span_text: text,
            byte_start: source_span.byte_start,
            final_relation_state: state,
        })
        .collect::<Vec<_>>();
    let internal_evaluation = evaluate_attended_relation_adapter(
        &relation.artifact,
        &relation_candidates,
        &supported_token_row,
        &unsupported_token_row,
    )?;
    let relation_evaluation = public_relation_evaluation(
        &internal_evaluation,
        &sentence_spans,
        relation.artifact.threshold,
    );

    let candidate_spans = sentence_spans
        .iter()
        .zip(&encoded.sequences)
        .zip(&terminal_states)
        .enumerate()
        .map(
            |(candidate_index, (((source_span, text), sequence), terminal_state))| {
                let relation_input = render_attended_relation_input(text, &config.question);
                let relation_input_text_cid = raw_cid(relation_input.as_bytes());
                if sequence.text_cid != relation_input_text_cid {
                    return Err(GroundedAnswerError::Audit(format!(
                        "candidate {candidate_index} state text CID does not match the exact attended relation input"
                    )));
                }
                Ok(SourceSpanCandidateBinding {
                    candidate_index,
                    source_span: *source_span,
                    text_cid: raw_cid(text.as_bytes()),
                    state_cid: sequence.audit.state_cid.clone(),
                    content_token_count: sequence.audit.content_token_count,
                    relation_input_text_cid: Some(relation_input_text_cid),
                    terminal_state_cid: Some(cid_serializable(terminal_state)?),
                })
            },
        )
        .collect::<Result<Vec<_>, GroundedAnswerError>>()?;

    let outcome = match internal_evaluation.decision {
        SourceRelationHeadDecision::Answer { candidate_index } => {
            let candidate = sentence_spans.get(candidate_index).ok_or_else(|| {
                GroundedAnswerError::Audit(
                    "attended relation adapter selected an absent source span".to_owned(),
                )
            })?;
            GroundedAnswerOutcome::Answered {
                answer: candidate.1.to_owned(),
                source_span: candidate.0,
            }
        }
        SourceRelationHeadDecision::Abstain => GroundedAnswerOutcome::Abstained {
            reason: GroundedAbstentionReason::RelationNoSupport,
        },
        SourceRelationHeadDecision::Contradiction => GroundedAnswerOutcome::Contradiction,
    };

    let source_after = read_source_file(&config.source_file)?;
    let source_cid = raw_cid(&source_before);
    if source_before != source_after {
        return Err(GroundedAnswerError::Audit(format!(
            "source {} changed during selection (before {}, after {})",
            config.source_file.display(),
            source_cid,
            raw_cid(&source_after)
        )));
    }
    let head_after = std::fs::read(&config.head).map_err(|error| {
        GroundedAnswerError::Audit(format!(
            "cannot rescan attended relation adapter {}: {error}",
            config.head.display()
        ))
    })?;
    if head_before != head_after {
        return Err(GroundedAnswerError::Audit(format!(
            "attended relation adapter {} changed during selection",
            config.head.display()
        )));
    }

    let source = GroundingSourceBinding {
        path: config.source_file.display().to_string(),
        source_cid,
        byte_length: source_before.len(),
        regular_non_symlink: true,
        utf8: true,
        reads: 2,
        unchanged_after_run: true,
    };
    let sequence_audits = encoded
        .sequences
        .iter()
        .map(|sequence| sequence.audit.clone())
        .collect::<Vec<_>>();
    let state_encoding = GroundedStateEncodingAudit {
        schema: encoded.schema,
        audit_cid: encoded.audit_cid,
        model_weights_cid: encoded.checkpoint.weights_cid.clone(),
        tokenizer_cid: encoded.checkpoint.tokenizer_cid.clone(),
        hidden_size: encoded.model_shape.dimension,
        checkpoint: encoded.checkpoint,
        model_shape: encoded.model_shape,
        subject_text_cid: None,
        subject_state_cid: None,
        subject_content_token_count: None,
        relation_input_policy: Some(ATTENDED_RELATION_INPUT_POLICY.to_owned()),
        sequence_audits,
        source_read_audit: encoded.source_read_audit,
        execution: encoded.execution,
    };
    let decision_cid = cid_serializable(&GroundedAttendedRelationDecisionIdentity {
        schema: ATTENDED_RELATION_REPORT_SCHEMA,
        relation_policy: ATTENDED_RELATION_ADAPTER_POLICY,
        relation_input_policy: ATTENDED_RELATION_INPUT_POLICY,
        source_cid: &source.source_cid,
        source_byte_length: source.byte_length,
        question: &config.question,
        relation_artifact_cid: &relation.binding.artifact_cid,
        relation_admission: &relation.binding.admission,
        representation_update: &relation.binding.representation_update,
        checkpoint_tree_cid: &state_encoding.checkpoint.checkpoint_tree_cid,
        model_weights_cid: &state_encoding.checkpoint.weights_cid,
        config_cid: &state_encoding.checkpoint.config_cid,
        tokenizer_cid: &state_encoding.checkpoint.tokenizer_cid,
        state_encoding_audit_cid: &state_encoding.audit_cid,
        fixed_verbalizer: &fixed_verbalizer,
        candidate_spans: &candidate_spans,
        relation_evaluation: &relation_evaluation,
        outcome: &outcome,
    })?;

    let claim_scope = if relation.binding.representation_update == ATTENDED_RELATION_UPDATE_NONE {
        "research-only fixed-verbalizer relation evaluation over the frozen #1017 six-layer coherent R4/Spin causal-softmax checkpoint; output is an exact original source sentence or a typed non-answer"
    } else {
        "research-only fixed-verbalizer relation evaluation over one representation-trained six-layer coherent R4/Spin causal-softmax checkpoint; output is an exact original source sentence or a typed non-answer"
    };

    Ok(GroundedAnswerReport {
        schema: ATTENDED_RELATION_REPORT_SCHEMA.to_owned(),
        decision_cid,
        claim_scope: claim_scope.to_owned(),
        source,
        question: config.question.clone(),
        subject,
        candidate_spans,
        pointer: None,
        pointer_evaluation: None,
        relation: None,
        attended_relation: Some(relation.binding),
        relation_evaluation: Some(relation_evaluation),
        fixed_verbalizer: Some(fixed_verbalizer),
        state_encoding,
        outcome,
        nonclaims: vec![
            "This artifact is admitted only for research parity; no qualified/product C1-SB3 tuple is registered.".to_owned(),
            "This bounded source-relative result does not establish open-domain question answering, reasoning, or general semantic entailment.".to_owned(),
            "The selected checkpoint remains source-backed, floating-point, multiplication-using, and ordinary-softmax based.".to_owned(),
            "This result does not establish the final source-free exact geometric runtime or a browser product surface.".to_owned(),
        ],
    })
}

fn load_fixed_verbalizer_rows(
    config: &GroundedAnswerConfig,
    checkpoint: &LocalCheckpointBinding,
    model_shape: &ModelShape,
    supported_token_id: u32,
    unsupported_token_id: u32,
) -> Result<(Vec<f32>, Vec<f32>, GroundedFixedVerbalizerAudit), GroundedAnswerError> {
    verify_bound_checkpoint_identity_files(config, checkpoint)?;
    let oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
        &config.model,
        1,
        TeacherExecutionConfig::fixed_workers(config.workers),
    )
    .map_err(|error| {
        GroundedAnswerError::Audit(format!(
            "cannot load the bound checkpoint's fixed verbalizer rows: {error}"
        ))
    })?;
    if oracle.source_cid() != checkpoint.weights_cid {
        return Err(GroundedAnswerError::Audit(
            "fixed verbalizer rows came from different weights than the encoded relation states"
                .to_owned(),
        ));
    }
    if oracle.cfg().vocab != model_shape.vocabulary
        || oracle.cfg().dim != model_shape.dimension
        || supported_token_id as usize >= oracle.cfg().vocab
        || unsupported_token_id as usize >= oracle.cfg().vocab
    {
        return Err(GroundedAnswerError::Audit(
            "fixed verbalizer token IDs or row width lie outside the bound checkpoint".to_owned(),
        ));
    }
    let mut supported_token_row = vec![0.0_f32; model_shape.dimension];
    let mut unsupported_token_row = vec![0.0_f32; model_shape.dimension];
    TeacherOracle::embedding(
        &oracle,
        supported_token_id as usize,
        &mut supported_token_row,
    );
    TeacherOracle::embedding(
        &oracle,
        unsupported_token_id as usize,
        &mut unsupported_token_row,
    );
    if supported_token_row.iter().any(|value| !value.is_finite())
        || unsupported_token_row.iter().any(|value| !value.is_finite())
    {
        return Err(GroundedAnswerError::Audit(
            "fixed verbalizer rows contain nonfinite lanes".to_owned(),
        ));
    }
    verify_bound_checkpoint_identity_files(config, checkpoint)?;
    let audit = GroundedFixedVerbalizerAudit {
        schema: FIXED_VERBALIZER_AUDIT_SCHEMA.to_owned(),
        scoring_policy: ATTENDED_RELATION_SCORING_POLICY.to_owned(),
        source_model_weights_cid: checkpoint.weights_cid.clone(),
        supported_token_id,
        unsupported_token_id,
        row_width: model_shape.dimension,
        supported_token_row_cid: cid_serializable(&supported_token_row)?,
        unsupported_token_row_cid: cid_serializable(&unsupported_token_row)?,
        oracle_loads: 1,
        checkpoint_identity_reads: 4,
        checkpoint_identity_unchanged: true,
    };
    Ok((supported_token_row, unsupported_token_row, audit))
}

fn verify_bound_checkpoint_identity_files(
    config: &GroundedAnswerConfig,
    checkpoint: &LocalCheckpointBinding,
) -> Result<(), GroundedAnswerError> {
    for (name, expected_cid) in [
        ("config.json", checkpoint.config_cid.as_str()),
        ("tokenizer.json", checkpoint.tokenizer_cid.as_str()),
    ] {
        let path = config.model.join(name);
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            GroundedAnswerError::Audit(format!(
                "cannot inspect bound checkpoint identity file {}: {error}",
                path.display()
            ))
        })?;
        if !metadata.file_type().is_file() {
            return Err(GroundedAnswerError::Audit(format!(
                "bound checkpoint identity file {} is not a regular non-symlink file",
                path.display()
            )));
        }
        let bytes = std::fs::read(&path).map_err(|error| {
            GroundedAnswerError::Audit(format!(
                "cannot read bound checkpoint identity file {}: {error}",
                path.display()
            ))
        })?;
        let observed_cid = raw_cid(&bytes);
        if observed_cid != expected_cid {
            return Err(GroundedAnswerError::Audit(format!(
                "bound checkpoint identity file {name} changed after state encoding: expected {expected_cid}, observed {observed_cid}"
            )));
        }
    }
    Ok(())
}

fn public_relation_evaluation(
    evaluation: &SourceRelationHeadEvaluation,
    sentence_spans: &[(SourceSpan, &str)],
    threshold: f32,
) -> GroundedRelationEvaluation {
    let positive_candidate_indices = evaluation
        .candidate_logits
        .iter()
        .enumerate()
        .filter_map(|(candidate_index, &logit)| (logit > threshold).then_some(candidate_index))
        .collect::<Vec<_>>();
    let mut unique_positive_texts = Vec::<&str>::new();
    for &candidate_index in &positive_candidate_indices {
        let text = sentence_spans[candidate_index].1;
        if !unique_positive_texts.contains(&text) {
            unique_positive_texts.push(text);
        }
    }
    let (decision, selected_span_index) = match evaluation.decision {
        SourceRelationHeadDecision::Answer { candidate_index } => {
            ("answer".to_owned(), Some(candidate_index))
        }
        SourceRelationHeadDecision::Abstain => ("abstain".to_owned(), None),
        SourceRelationHeadDecision::Contradiction => ("conflict".to_owned(), None),
    };
    GroundedRelationEvaluation {
        candidate_logits: evaluation.candidate_logits.clone(),
        positive_candidate_indices,
        positive_unique_span_count: unique_positive_texts.len(),
        decision,
        selected_span_index,
    }
}

fn public_pointer_evaluation(
    evaluation: &SourceSpanPointerEvaluation,
) -> GroundedPointerEvaluation {
    let (decision, selected_span_index) = match evaluation.decision {
        SourceSpanPointerDecision::Answer { candidate_index } => {
            ("answer".to_owned(), Some(candidate_index))
        }
        SourceSpanPointerDecision::Abstain => ("abstain".to_owned(), None),
        SourceSpanPointerDecision::Contradiction => ("conflict".to_owned(), None),
    };
    GroundedPointerEvaluation {
        candidate_scores: evaluation.scores.candidate_scores.clone(),
        ranked_candidate_indices: evaluation.scores.ranked_candidate_indices.clone(),
        logits: GroundedPointerLogits {
            answer: evaluation.scores.answer_logit,
            abstain: evaluation.scores.abstain_logit,
            conflict: evaluation.scores.conflict_logit,
        },
        decision,
        selected_span_index,
    }
}

pub fn write_json_report(
    path: &Path,
    report: &GroundedAnswerReport,
) -> Result<(), GroundedAnswerError> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| GroundedAnswerError::Io(io::Error::other(error)))?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Reject an audit output that already names the same file as the bound source.
pub fn require_distinct_output_path(
    source: &Path,
    head: &Path,
    model: &Path,
    output: &Path,
) -> Result<(), GroundedAnswerError> {
    let source_metadata = std::fs::metadata(source).map_err(|error| {
        GroundedAnswerError::InvalidSource(format!(
            "cannot inspect {} before selecting audit output: {error}",
            source.display()
        ))
    })?;
    let canonical_model = std::fs::canonicalize(model).map_err(|error| {
        GroundedAnswerError::InvalidRequest(format!(
            "cannot resolve model directory {}: {error}",
            model.display()
        ))
    })?;
    let resolved_output = resolve_output_path(output)?;
    if resolved_output.starts_with(&canonical_model) {
        return Err(GroundedAnswerError::InvalidRequest(
            "--json-output must remain outside the immutable model directory".to_owned(),
        ));
    }
    let output_metadata = match std::fs::metadata(output) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(GroundedAnswerError::InvalidRequest(format!(
                "cannot inspect --json-output {}: {error}",
                output.display()
            )))
        }
    };
    if same_file_identity(source, &source_metadata, output, &output_metadata)? {
        return Err(GroundedAnswerError::InvalidRequest(
            "--json-output must not name or alias --source-file".to_owned(),
        ));
    }
    let head_metadata = std::fs::metadata(head).map_err(|error| {
        GroundedAnswerError::InvalidRequest(format!(
            "cannot inspect pointer head {}: {error}",
            head.display()
        ))
    })?;
    if same_file_identity(head, &head_metadata, output, &output_metadata)? {
        return Err(GroundedAnswerError::InvalidRequest(
            "--json-output must not name or alias --head".to_owned(),
        ));
    }
    reject_checkpoint_hardlink(model, output, &output_metadata)?;
    Ok(())
}

fn resolve_output_path(path: &Path) -> Result<PathBuf, GroundedAnswerError> {
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(GroundedAnswerError::InvalidRequest(
            "--json-output must not contain `..` path traversal".to_owned(),
        ));
    }
    if path.exists() {
        return std::fs::canonicalize(path).map_err(GroundedAnswerError::Io);
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut ancestor = absolute.as_path();
    let mut suffix = Vec::new();
    while !ancestor.exists() {
        let name = ancestor.file_name().ok_or_else(|| {
            GroundedAnswerError::InvalidRequest(
                "--json-output has no resolvable existing ancestor".to_owned(),
            )
        })?;
        suffix.push(name.to_owned());
        ancestor = ancestor.parent().ok_or_else(|| {
            GroundedAnswerError::InvalidRequest(
                "--json-output has no resolvable existing ancestor".to_owned(),
            )
        })?;
    }
    let mut resolved = std::fs::canonicalize(ancestor)?;
    for component in suffix.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn reject_checkpoint_hardlink(
    directory: &Path,
    output: &Path,
    output_metadata: &std::fs::Metadata,
) -> Result<(), GroundedAnswerError> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            reject_checkpoint_hardlink(&entry.path(), output, output_metadata)?;
        } else if file_type.is_file() {
            let metadata = entry.metadata()?;
            if same_file_identity(&entry.path(), &metadata, output, output_metadata)? {
                return Err(GroundedAnswerError::InvalidRequest(
                    "--json-output must not name or alias a checkpoint file".to_owned(),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn same_file_identity(
    _left_path: &Path,
    left: &std::fs::Metadata,
    _right_path: &Path,
    right: &std::fs::Metadata,
) -> Result<bool, GroundedAnswerError> {
    use std::os::unix::fs::MetadataExt;
    Ok(left.dev() == right.dev() && left.ino() == right.ino())
}

#[cfg(not(unix))]
fn same_file_identity(
    left_path: &Path,
    _left: &std::fs::Metadata,
    right_path: &Path,
    _right: &std::fs::Metadata,
) -> Result<bool, GroundedAnswerError> {
    Ok(std::fs::canonicalize(left_path)? == std::fs::canonicalize(right_path)?)
}

fn validate_request(config: &GroundedAnswerConfig) -> Result<(), GroundedAnswerError> {
    if config.question.trim().is_empty() {
        return Err(GroundedAnswerError::InvalidRequest(
            "--question must not be empty or whitespace-only".to_owned(),
        ));
    }
    if config.question.len() > MAX_QUESTION_BYTES {
        return Err(GroundedAnswerError::InvalidRequest(format!(
            "--question is {} bytes; maximum is {MAX_QUESTION_BYTES}",
            config.question.len()
        )));
    }
    Ok(())
}

fn parse_subject(question: &str) -> Result<String, GroundedAnswerError> {
    const PREFIX: &str = "Where is the ";
    let subject = question
        .strip_prefix(PREFIX)
        .and_then(|rest| rest.strip_suffix('?'))
        .ok_or_else(|| {
            GroundedAnswerError::InvalidRequest(
                "--question must match exactly `Where is the <subject>?`".to_owned(),
            )
        })?;
    if subject.is_empty() || subject != subject.trim() || subject.contains(['?', '\n', '\r']) {
        return Err(GroundedAnswerError::InvalidRequest(
            "--question must match exactly `Where is the <subject>?`".to_owned(),
        ));
    }
    Ok(subject.to_owned())
}

fn split_sentence_spans(source: &str) -> Result<Vec<(SourceSpan, &str)>, GroundedAnswerError> {
    let mut spans = Vec::new();
    let mut byte_start = None;
    for (offset, character) in source.char_indices() {
        if byte_start.is_none() {
            if character.is_whitespace() {
                continue;
            }
            byte_start = Some(offset);
        }
        if !matches!(character, '.' | '!' | '?') {
            continue;
        }
        let Some(start) = byte_start else {
            return Err(GroundedAnswerError::InvalidSource(
                "source contains an empty punctuation-only sentence".to_owned(),
            ));
        };
        let end = offset + character.len_utf8();
        let text = &source[start..end];
        if text[..text.len() - character.len_utf8()].trim().is_empty() {
            return Err(GroundedAnswerError::InvalidSource(
                "source contains an empty punctuation-only sentence".to_owned(),
            ));
        }
        spans.push((
            SourceSpan {
                byte_start: start,
                byte_end: end,
            },
            text,
        ));
        if spans.len() > MAX_SOURCE_SPANS {
            return Err(GroundedAnswerError::InvalidSource(format!(
                "source exceeds {MAX_SOURCE_SPANS} punctuation-terminated sentence spans"
            )));
        }
        byte_start = None;
    }
    if byte_start.is_some() {
        return Err(GroundedAnswerError::InvalidSource(
            "source has a non-whitespace suffix without .!? termination".to_owned(),
        ));
    }
    if spans.is_empty() {
        return Err(GroundedAnswerError::InvalidSource(
            "source has no punctuation-terminated sentence".to_owned(),
        ));
    }
    Ok(spans)
}

fn read_source_file(path: &Path) -> Result<Vec<u8>, GroundedAnswerError> {
    let file = open_source_no_follow(path)?;
    let metadata = file.metadata().map_err(|error| {
        GroundedAnswerError::InvalidSource(format!("cannot inspect {}: {error}", path.display()))
    })?;
    if !metadata.is_file() {
        return Err(GroundedAnswerError::InvalidSource(format!(
            "{} is not a regular non-symlink file",
            path.display()
        )));
    }
    if metadata.len() > MAX_SOURCE_BYTES as u64 {
        return Err(GroundedAnswerError::InvalidSource(format!(
            "{} is {} bytes; maximum is {MAX_SOURCE_BYTES}",
            path.display(),
            metadata.len()
        )));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_SOURCE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            GroundedAnswerError::InvalidSource(format!("cannot read {}: {error}", path.display()))
        })?;
    if bytes.is_empty() {
        return Err(GroundedAnswerError::InvalidSource(format!(
            "{} is empty",
            path.display()
        )));
    }
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err(GroundedAnswerError::InvalidSource(format!(
            "{} grew past the {MAX_SOURCE_BYTES}-byte maximum while being read",
            path.display()
        )));
    }
    let source = std::str::from_utf8(&bytes).map_err(|error| {
        GroundedAnswerError::InvalidSource(format!(
            "{} is not exact UTF-8: {error}",
            path.display()
        ))
    })?;
    if source.trim().is_empty() {
        return Err(GroundedAnswerError::InvalidSource(format!(
            "{} is whitespace-only",
            path.display()
        )));
    }
    Ok(bytes)
}

#[cfg(unix)]
fn open_source_no_follow(path: &Path) -> Result<File, GroundedAnswerError> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| {
            GroundedAnswerError::InvalidSource(format!(
                "cannot open {} as a regular non-symlink file: {error}",
                path.display()
            ))
        })
}

#[cfg(not(unix))]
fn open_source_no_follow(path: &Path) -> Result<File, GroundedAnswerError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        GroundedAnswerError::InvalidSource(format!("cannot inspect {}: {error}", path.display()))
    })?;
    if !metadata.file_type().is_file() {
        return Err(GroundedAnswerError::InvalidSource(format!(
            "{} is not a regular non-symlink file",
            path.display()
        )));
    }
    File::open(path).map_err(|error| {
        GroundedAnswerError::InvalidSource(format!("cannot open {}: {error}", path.display()))
    })
}

fn raw_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn cid_serializable<T: Serialize>(value: &T) -> Result<String, GroundedAnswerError> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        GroundedAnswerError::Audit(format!("cannot serialize decision identity: {error}"))
    })?;
    Ok(raw_cid(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn question_and_sentence_policies_match_the_frozen_python_boundary() {
        assert_eq!(
            parse_subject("Where is the copper compass?").unwrap(),
            "copper compass"
        );
        assert!(parse_subject("where is the copper compass?").is_err());

        let source = "  Alpha is here.  Beta is there!\n";
        let spans = split_sentence_spans(source).unwrap();
        assert_eq!(
            spans[0],
            (
                SourceSpan {
                    byte_start: 2,
                    byte_end: 16,
                },
                "Alpha is here."
            )
        );
        assert_eq!(
            spans[1],
            (
                SourceSpan {
                    byte_start: 18,
                    byte_end: 32,
                },
                "Beta is there!"
            )
        );
    }

    #[test]
    fn attended_decision_identity_binds_admission_update_and_checkpoint() {
        let fixed_verbalizer = GroundedFixedVerbalizerAudit {
            schema: FIXED_VERBALIZER_AUDIT_SCHEMA.to_owned(),
            scoring_policy: ATTENDED_RELATION_SCORING_POLICY.to_owned(),
            source_model_weights_cid: raw_cid(b"weights"),
            supported_token_id: 1771,
            unsupported_token_id: 542,
            row_width: 288,
            supported_token_row_cid: raw_cid(b"yes-row"),
            unsupported_token_row_cid: raw_cid(b"no-row"),
            oracle_loads: 1,
            checkpoint_identity_reads: 4,
            checkpoint_identity_unchanged: true,
        };
        let candidate_spans = vec![SourceSpanCandidateBinding {
            candidate_index: 0,
            source_span: SourceSpan {
                byte_start: 0,
                byte_end: 9,
            },
            text_cid: raw_cid(b"Evidence."),
            state_cid: raw_cid(b"state"),
            content_token_count: 1,
            relation_input_text_cid: Some(raw_cid(b"input")),
            terminal_state_cid: Some(raw_cid(b"terminal")),
        }];
        let relation_evaluation = GroundedRelationEvaluation {
            candidate_logits: vec![1.0],
            positive_candidate_indices: vec![0],
            positive_unique_span_count: 1,
            decision: "answer".to_owned(),
            selected_span_index: Some(0),
        };
        let outcome = GroundedAnswerOutcome::Answered {
            answer: "Evidence.".to_owned(),
            source_span: candidate_spans[0].source_span,
        };
        let artifact_cid = raw_cid(b"adapter");
        let tree_cid = raw_cid(b"tree");
        let weights_cid = raw_cid(b"weights");
        let config_cid = raw_cid(b"config");
        let tokenizer_cid = raw_cid(b"tokenizer");
        let audit_cid = raw_cid(b"audit");
        let identity = |admission: &str, representation_update: &str| {
            cid_serializable(&GroundedAttendedRelationDecisionIdentity {
                schema: ATTENDED_RELATION_REPORT_SCHEMA,
                relation_policy: ATTENDED_RELATION_ADAPTER_POLICY,
                relation_input_policy: ATTENDED_RELATION_INPUT_POLICY,
                source_cid: "blake3:source",
                source_byte_length: 9,
                question: "Where is the evidence?",
                relation_artifact_cid: &artifact_cid,
                relation_admission: admission,
                representation_update,
                checkpoint_tree_cid: &tree_cid,
                model_weights_cid: &weights_cid,
                config_cid: &config_cid,
                tokenizer_cid: &tokenizer_cid,
                state_encoding_audit_cid: &audit_cid,
                fixed_verbalizer: &fixed_verbalizer,
                candidate_spans: &candidate_spans,
                relation_evaluation: &relation_evaluation,
                outcome: &outcome,
            })
            .expect("attended decision CID")
        };
        let lora = identity("research_only", "lora_qkvo_all_layers");
        assert_ne!(lora, identity("qualified", "lora_qkvo_all_layers"));
        assert_ne!(lora, identity("research_only", "none_frozen_readout"));
    }
}
