//! Learned, fail-closed source-span selection over #1017 R4/Spin states.
//!
//! The pointer never decodes an answer token. It scores exact source sentence
//! spans from the final normalized token states produced by the established
//! all-layer R4/Spin causal-softmax executor, then selects either one original
//! byte span or an explicit abstention/conflict terminal.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const POINTER_ARTIFACT_SCHEMA: &str = "uor-r4.source-span-pointer/1";
pub const POINTER_POLICY: &str = "R4SourceSpanPointerV1";
pub const QUESTION_POLICY: &str = "Where is the <subject>?";
pub const SENTENCE_POLICY: &str = "exact .!? terminated UTF-8 byte spans";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpanPointerArtifact {
    pub schema: String,
    pub policy: String,
    pub issue: u32,
    pub model_weights_cid: String,
    pub tokenizer_cid: String,
    pub hidden_size: usize,
    pub state_weights: Vec<f32>,
    pub score_scale: f32,
    pub answer_bias: f32,
    pub abstain_bias: f32,
    pub conflict_bias: f32,
    pub maximum_source_spans: usize,
    pub question_policy: String,
    pub sentence_policy: String,
    pub dataset_cid: String,
    pub split_policy_cid: String,
    pub run_contract_cid: String,
    pub training_result_cid: String,
    pub preflight: serde_json::Value,
    pub development_metrics: serde_json::Value,
    pub product_probe_commitments: Vec<String>,
    pub artifact_cid: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceSpanPointerScores {
    pub candidate_scores: Vec<f32>,
    pub ranked_candidate_indices: Vec<usize>,
    pub answer_logit: f32,
    pub abstain_logit: f32,
    pub conflict_logit: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSpanPointerDecision {
    Answer { candidate_index: usize },
    Abstain,
    Contradiction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceSpanPointerEvaluation {
    pub scores: SourceSpanPointerScores,
    pub decision: SourceSpanPointerDecision,
}

#[derive(Debug)]
pub enum SourceSpanPointerError {
    InvalidArtifact(String),
    InvalidStates(String),
    Io(std::io::Error),
}

impl fmt::Display for SourceSpanPointerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArtifact(reason) => {
                write!(formatter, "invalid source-span pointer: {reason}")
            }
            Self::InvalidStates(reason) => write!(formatter, "invalid R4 pointer states: {reason}"),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for SourceSpanPointerError {}

impl From<std::io::Error> for SourceSpanPointerError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpanPointerBinding {
    pub path: String,
    pub artifact_cid: String,
    pub policy: String,
    pub model_weights_cid: String,
    pub tokenizer_cid: String,
    pub dataset_cid: String,
    pub split_policy_cid: String,
    pub run_contract_cid: String,
    pub training_result_cid: String,
}

pub struct LoadedSourceSpanPointer {
    pub artifact: SourceSpanPointerArtifact,
    pub binding: SourceSpanPointerBinding,
}

pub fn load_source_span_pointer(
    path: &Path,
    expected_model_weights_cid: &str,
    expected_tokenizer_cid: &str,
) -> Result<LoadedSourceSpanPointer, SourceSpanPointerError> {
    let bytes = std::fs::read(path)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        SourceSpanPointerError::InvalidArtifact(format!("{} is not JSON: {error}", path.display()))
    })?;
    let embedded_cid = value
        .get("artifact_cid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            SourceSpanPointerError::InvalidArtifact("artifact_cid is absent".to_owned())
        })?;
    verify_embedded_cid(&bytes, embedded_cid)?;
    let artifact: SourceSpanPointerArtifact = serde_json::from_value(value).map_err(|error| {
        SourceSpanPointerError::InvalidArtifact(format!("artifact fields are invalid: {error}"))
    })?;
    validate_artifact(
        &artifact,
        expected_model_weights_cid,
        expected_tokenizer_cid,
    )?;
    let binding = SourceSpanPointerBinding {
        path: PathBuf::from(path).display().to_string(),
        artifact_cid: artifact.artifact_cid.clone(),
        policy: artifact.policy.clone(),
        model_weights_cid: artifact.model_weights_cid.clone(),
        tokenizer_cid: artifact.tokenizer_cid.clone(),
        dataset_cid: artifact.dataset_cid.clone(),
        split_policy_cid: artifact.split_policy_cid.clone(),
        run_contract_cid: artifact.run_contract_cid.clone(),
        training_result_cid: artifact.training_result_cid.clone(),
    };
    Ok(LoadedSourceSpanPointer { artifact, binding })
}

pub fn evaluate_source_span_pointer(
    artifact: &SourceSpanPointerArtifact,
    subject_states: &[Vec<f32>],
    candidate_states: &[Vec<Vec<f32>>],
) -> Result<SourceSpanPointerEvaluation, SourceSpanPointerError> {
    if subject_states.is_empty() {
        return Err(SourceSpanPointerError::InvalidStates(
            "subject encoded to zero content-token states".to_owned(),
        ));
    }
    if !(2..=artifact.maximum_source_spans).contains(&candidate_states.len()) {
        return Err(SourceSpanPointerError::InvalidStates(format!(
            "candidate span count must be in 2..={}, observed {}",
            artifact.maximum_source_spans,
            candidate_states.len()
        )));
    }
    require_state_shape(artifact, subject_states, "subject")?;
    for (index, states) in candidate_states.iter().enumerate() {
        if states.is_empty() {
            return Err(SourceSpanPointerError::InvalidStates(format!(
                "candidate {index} encoded to zero content-token states"
            )));
        }
        require_state_shape(artifact, states, "candidate")?;
    }

    let mut candidate_scores = Vec::with_capacity(candidate_states.len());
    for states in candidate_states {
        let mut sum = 0.0_f32;
        for subject in subject_states {
            let best = states
                .iter()
                .map(|candidate| weighted_cosine(&artifact.state_weights, subject, candidate))
                .try_fold(f32::NEG_INFINITY, |best, score| {
                    score.map(|score| best.max(score))
                })?;
            sum += best;
        }
        candidate_scores.push(sum / subject_states.len() as f32);
    }

    let mut ranked_candidate_indices = (0..candidate_scores.len()).collect::<Vec<_>>();
    ranked_candidate_indices.sort_by(|left, right| {
        candidate_scores[*right]
            .total_cmp(&candidate_scores[*left])
            .then_with(|| left.cmp(right))
    });
    let first = ranked_candidate_indices[0];
    let answer_logit = artifact.score_scale * candidate_scores[first] + artifact.answer_bias;
    let abstain_logit = artifact.abstain_bias;
    let second = ranked_candidate_indices[1];
    let conflict_logit = artifact.score_scale * candidate_scores[second] + artifact.conflict_bias;

    // Safety precedence for exact ties: contradiction, then abstention, then
    // an answer. This cannot turn an ambiguous equality into served text.
    let mut class_scores = vec![(1_u8, abstain_logit)];
    class_scores.push((0_u8, conflict_logit));
    class_scores.push((2_u8, answer_logit));
    class_scores.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    let decision = match class_scores[0].0 {
        0 => SourceSpanPointerDecision::Contradiction,
        1 => SourceSpanPointerDecision::Abstain,
        2 => SourceSpanPointerDecision::Answer {
            candidate_index: first,
        },
        _ => {
            return Err(SourceSpanPointerError::InvalidArtifact(
                "pointer class ranking escaped the fixed three-class set".to_owned(),
            ))
        }
    };

    Ok(SourceSpanPointerEvaluation {
        scores: SourceSpanPointerScores {
            candidate_scores,
            ranked_candidate_indices,
            answer_logit,
            abstain_logit,
            conflict_logit,
        },
        decision,
    })
}

fn validate_artifact(
    artifact: &SourceSpanPointerArtifact,
    expected_model_weights_cid: &str,
    expected_tokenizer_cid: &str,
) -> Result<(), SourceSpanPointerError> {
    let invalid = |reason: &str| SourceSpanPointerError::InvalidArtifact(reason.to_owned());
    if artifact.schema != POINTER_ARTIFACT_SCHEMA
        || artifact.policy != POINTER_POLICY
        || artifact.issue != 954
    {
        return Err(invalid("schema, policy, or issue does not match C1-SB1"));
    }
    if artifact.model_weights_cid != expected_model_weights_cid
        || artifact.tokenizer_cid != expected_tokenizer_cid
    {
        return Err(invalid(
            "head is not bound to the loaded #1017 weights/tokenizer",
        ));
    }
    if artifact.hidden_size == 0 || artifact.state_weights.len() != artifact.hidden_size {
        return Err(invalid("state-weight width does not match hidden_size"));
    }
    if artifact
        .state_weights
        .iter()
        .any(|weight| !weight.is_finite() || *weight <= 0.0)
    {
        return Err(invalid(
            "state weights must be finite and strictly positive",
        ));
    }
    if !artifact.score_scale.is_finite() || artifact.score_scale <= 0.0 {
        return Err(invalid("score_scale must be finite and positive"));
    }
    if [
        artifact.answer_bias,
        artifact.abstain_bias,
        artifact.conflict_bias,
    ]
    .iter()
    .any(|value| !value.is_finite())
    {
        return Err(invalid("pointer class biases must be finite"));
    }
    if artifact.maximum_source_spans != 8
        || artifact.question_policy != QUESTION_POLICY
        || artifact.sentence_policy != SENTENCE_POLICY
    {
        return Err(invalid(
            "bounded source/question policy differs from the frozen contract",
        ));
    }
    if artifact.product_probe_commitments.len() != 3
        || artifact
            .product_probe_commitments
            .iter()
            .any(|cid| !is_blake3_cid(cid))
    {
        return Err(invalid(
            "exactly three valid product commitments are required",
        ));
    }
    for (label, cid) in [
        ("artifact", artifact.artifact_cid.as_str()),
        ("model weights", artifact.model_weights_cid.as_str()),
        ("tokenizer", artifact.tokenizer_cid.as_str()),
        ("dataset", artifact.dataset_cid.as_str()),
        ("split policy", artifact.split_policy_cid.as_str()),
        ("run contract", artifact.run_contract_cid.as_str()),
        ("training result", artifact.training_result_cid.as_str()),
    ] {
        if !is_blake3_cid(cid) {
            return Err(invalid(&format!("{label} CID is invalid")));
        }
    }
    Ok(())
}

fn require_state_shape(
    artifact: &SourceSpanPointerArtifact,
    states: &[Vec<f32>],
    label: &str,
) -> Result<(), SourceSpanPointerError> {
    for (index, state) in states.iter().enumerate() {
        if state.len() != artifact.hidden_size || state.iter().any(|value| !value.is_finite()) {
            return Err(SourceSpanPointerError::InvalidStates(format!(
                "{label} state {index} is not {} finite lanes",
                artifact.hidden_size
            )));
        }
    }
    Ok(())
}

fn weighted_cosine(
    weights: &[f32],
    left: &[f32],
    right: &[f32],
) -> Result<f32, SourceSpanPointerError> {
    let mut dot = 0.0_f32;
    let mut left_norm = 0.0_f32;
    let mut right_norm = 0.0_f32;
    for ((&weight, &left), &right) in weights.iter().zip(left).zip(right) {
        dot += weight * left * right;
        left_norm += weight * left * left;
        right_norm += weight * right * right;
    }
    let denominator = left_norm.sqrt() * right_norm.sqrt();
    if !denominator.is_finite() || denominator <= 0.0 {
        return Err(SourceSpanPointerError::InvalidStates(
            "weighted cosine has a nonpositive or nonfinite norm".to_owned(),
        ));
    }
    let score = dot / denominator;
    if !score.is_finite() {
        return Err(SourceSpanPointerError::InvalidStates(
            "weighted cosine is nonfinite".to_owned(),
        ));
    }
    Ok(score.clamp(-1.0, 1.0))
}

fn verify_embedded_cid(bytes: &[u8], expected: &str) -> Result<(), SourceSpanPointerError> {
    let text = std::str::from_utf8(bytes).map_err(|error| {
        SourceSpanPointerError::InvalidArtifact(format!("artifact JSON is not UTF-8: {error}"))
    })?;
    let needle = format!("\"artifact_cid\":\"{expected}\"");
    let start = text.find(&needle).ok_or_else(|| {
        SourceSpanPointerError::InvalidArtifact(
            "artifact does not contain its canonical artifact_cid".to_owned(),
        )
    })?;
    if text[start + needle.len()..].contains(&needle) {
        return Err(SourceSpanPointerError::InvalidArtifact(
            "artifact contains duplicate artifact_cid values".to_owned(),
        ));
    }
    let end = start + needle.len();
    let mut unsigned = bytes.to_vec();
    if start > 0 && unsigned[start - 1] == b',' {
        unsigned.drain(start - 1..end);
    } else if unsigned.get(end) == Some(&b',') {
        unsigned.drain(start..=end);
    } else {
        return Err(SourceSpanPointerError::InvalidArtifact(
            "artifact_cid is not one field of canonical JSON".to_owned(),
        ));
    }
    let observed = format!("blake3:{}", blake3::hash(&unsigned).to_hex());
    if observed != expected {
        return Err(SourceSpanPointerError::InvalidArtifact(
            "artifact_cid does not reproduce".to_owned(),
        ));
    }
    Ok(())
}

fn is_blake3_cid(value: &str) -> bool {
    value.len() == 71
        && value
            .strip_prefix("blake3:")
            .is_some_and(|hex| hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}
