//! Fail-closed source-relative relation decisions over #1017 states and its
//! explicitly bound representation-adapted descendant.
//!
//! Each candidate is represented by the final normalized residual from the exact
//! relation input rendered by [`render_relation_input`]. The head scores those
//! width-288 states, collapses exact duplicate span text, and either selects one
//! original occurrence or returns an explicit non-answer terminal.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const RELATION_HEAD_ARTIFACT_SCHEMA: &str = "uor-r4.source-relative-relation-head/1";
pub const RELATION_HEAD_POLICY: &str = "R4SourceRelativeRelationHeadV1";
pub const RELATION_INPUT_POLICY: &str = "Evidence:\n<span>\nQuestion:\n<question>";
pub const RELATION_STATE_WIDTH: usize = 288;
pub const RELATION_HIDDEN_WIDTH: usize = 32;
pub const MAXIMUM_SOURCE_SPANS: usize = 8;
pub const EXPECTED_MODEL_WEIGHTS_CID: &str =
    "blake3:c5bf31aa97a567b3aaad4461ce2fac9cebc12b0a38becb6d02d21b43b493bf5d";
pub const EXPECTED_TOKENIZER_CID: &str =
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc";

/// C1-SB3's independently versioned fixed-verbalizer relation adapter.
///
/// This is deliberately not a relaxation of [`RELATION_HEAD_ARTIFACT_SCHEMA`].
/// The predecessor head remains frozen to #1017 for exact C1-SB2 replay. This
/// artifact binds either that unchanged checkpoint (the preflight short
/// circuit) or a distinct representation-adapted checkpoint and uses its fixed
/// tied vocabulary rows as the only readout.
pub const ATTENDED_RELATION_ADAPTER_SCHEMA: &str = "uor-r4.attended-relation-adapter/1";
pub const ATTENDED_RELATION_ADAPTER_POLICY: &str = "R4AttendedRelationAdapterV1";
pub const ATTENDED_RELATION_INPUT_POLICY: &str =
    "Evidence:\n<span>\nQuestion:\n<question>\nSupported:";
pub const ATTENDED_RELATION_SCORING_POLICY: &str =
    "dot(final_normalized_residual,tied_embedding[1771])-dot(final_normalized_residual,tied_embedding[542]);input-lanes-ascending-f32";
pub const ATTENDED_RELATION_RESEARCH_ADMISSION: &str = "research_only";
pub const ATTENDED_RELATION_UPDATE_NONE: &str = "none_frozen_readout";
pub const ATTENDED_RELATION_UPDATE_LORA: &str = "lora_qkvo_all_layers";
pub const ATTENDED_RELATION_SUPPORTED_TOKEN_ID: u32 = 1771;
pub const ATTENDED_RELATION_UNSUPPORTED_TOKEN_ID: u32 = 542;
pub const EXPECTED_MODEL_CONFIG_CID: &str =
    "blake3:1f1ddb6de22f5c81c04d3093eeff8e0991d63b79ee33bc8ff3cf7c68ef0a9497";

const FIRST_LAYER_WEIGHT_COUNT: usize = RELATION_HIDDEN_WIDTH * RELATION_STATE_WIDTH;

/// Render the sole relation input admitted by this head.
pub fn render_relation_input(span_text: &str, question: &str) -> String {
    format!("Evidence:\n{span_text}\nQuestion:\n{question}")
}

/// Render the exact C1-SB3 input. The final byte is the `:` in `Supported:`;
/// there is no trailing newline or generated verbalizer token.
pub fn render_attended_relation_input(span_text: &str, question: &str) -> String {
    format!("Evidence:\n{span_text}\nQuestion:\n{question}\nSupported:")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRelationHeadArtifact {
    pub schema: String,
    pub policy: String,
    pub issue: u32,
    pub model_weights_cid: String,
    pub tokenizer_cid: String,
    pub hidden_size: usize,
    pub hidden_width: usize,
    /// Row-major `[hidden_width, hidden_size]` dense affine weights.
    pub first_layer_weights: Vec<f32>,
    pub first_layer_biases: Vec<f32>,
    pub output_weights: Vec<f32>,
    pub output_bias: f32,
    pub threshold: f32,
    pub maximum_source_spans: usize,
    pub relation_input_policy: String,
    pub dataset_cid: String,
    pub split_policy_cid: String,
    pub run_contract_cid: String,
    pub training_result_cid: String,
    pub preflight: serde_json::Value,
    pub development_metrics: serde_json::Value,
    pub product_probe_commitments: Vec<String>,
    pub artifact_cid: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRelationHeadBinding {
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

pub struct LoadedSourceRelationHead {
    pub artifact: SourceRelationHeadArtifact,
    pub binding: SourceRelationHeadBinding,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttendedRelationAdapterArtifact {
    pub schema: String,
    pub policy: String,
    pub issue: u32,
    /// Only `research_only` is admitted until a positive frozen tuple is
    /// reviewed and registered in source.
    pub admission: String,
    /// The sole representation update used to produce `model_weights_cid`.
    /// A frozen #1017 short-circuit and the predeclared all-layer QKVO LoRA
    /// path are distinct, exhaustive research mechanisms.
    pub representation_update: String,
    pub predecessor_model_weights_cid: String,
    pub model_weights_cid: String,
    pub checkpoint_tree_cid: String,
    pub config_cid: String,
    pub tokenizer_cid: String,
    pub hidden_size: usize,
    pub supported_token_id: u32,
    pub unsupported_token_id: u32,
    pub threshold: f32,
    pub maximum_source_spans: usize,
    pub relation_input_policy: String,
    pub dataset_cid: String,
    pub split_policy_cid: String,
    pub run_contract_cid: String,
    pub training_result_cid: String,
    pub product_probe_commitments: Vec<String>,
    pub artifact_cid: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttendedRelationAdapterBinding {
    pub path: String,
    pub artifact_cid: String,
    pub policy: String,
    pub admission: String,
    pub representation_update: String,
    pub predecessor_model_weights_cid: String,
    pub model_weights_cid: String,
    pub checkpoint_tree_cid: String,
    pub config_cid: String,
    pub tokenizer_cid: String,
    pub supported_token_id: u32,
    pub unsupported_token_id: u32,
    pub dataset_cid: String,
    pub split_policy_cid: String,
    pub run_contract_cid: String,
    pub training_result_cid: String,
}

pub struct LoadedAttendedRelationAdapter {
    pub artifact: AttendedRelationAdapterArtifact,
    pub binding: AttendedRelationAdapterBinding,
}

#[derive(Clone, Copy, Debug)]
pub struct SourceRelationCandidate<'a> {
    pub span_text: &'a str,
    pub byte_start: usize,
    pub final_relation_state: &'a [f32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRelationHeadDecision {
    Answer { candidate_index: usize },
    Abstain,
    Contradiction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceRelationHeadEvaluation {
    pub candidate_logits: Vec<f32>,
    pub decision: SourceRelationHeadDecision,
}

#[derive(Debug)]
pub enum SourceRelationHeadError {
    InvalidArtifact(String),
    InvalidInput(String),
    Io(std::io::Error),
}

impl fmt::Display for SourceRelationHeadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArtifact(reason) => {
                write!(formatter, "invalid source-relative relation head: {reason}")
            }
            Self::InvalidInput(reason) => {
                write!(
                    formatter,
                    "invalid source-relative relation input: {reason}"
                )
            }
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for SourceRelationHeadError {}

impl From<std::io::Error> for SourceRelationHeadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Load one canonical, self-addressed head bound to the exact #1017 checkpoint.
pub fn load_source_relation_head(
    path: &Path,
    loaded_model_weights_cid: &str,
    loaded_tokenizer_cid: &str,
) -> Result<LoadedSourceRelationHead, SourceRelationHeadError> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() {
        return Err(SourceRelationHeadError::InvalidArtifact(format!(
            "{} is not a regular non-symlink file",
            path.display()
        )));
    }
    let bytes = std::fs::read(path)?;
    let artifact =
        decode_artifact_bytes(&bytes, path, loaded_model_weights_cid, loaded_tokenizer_cid)?;
    let binding = SourceRelationHeadBinding {
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
    Ok(LoadedSourceRelationHead { artifact, binding })
}

/// Load one canonical C1-SB3 adapter and bind it to the exact checkpoint that
/// produced the candidate states. Only the explicit research admission is
/// accepted in this revision; a future qualified artifact requires a reviewed
/// repository trust anchor rather than a producer-supplied assertion.
pub fn load_attended_relation_adapter(
    path: &Path,
    loaded_model_weights_cid: &str,
    loaded_checkpoint_tree_cid: &str,
    loaded_config_cid: &str,
    loaded_tokenizer_cid: &str,
) -> Result<LoadedAttendedRelationAdapter, SourceRelationHeadError> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() {
        return Err(SourceRelationHeadError::InvalidArtifact(format!(
            "{} is not a regular non-symlink file",
            path.display()
        )));
    }
    let bytes = std::fs::read(path)?;
    let artifact = decode_attended_adapter_bytes(
        &bytes,
        path,
        loaded_model_weights_cid,
        loaded_checkpoint_tree_cid,
        loaded_config_cid,
        loaded_tokenizer_cid,
    )?;
    let binding = AttendedRelationAdapterBinding {
        path: PathBuf::from(path).display().to_string(),
        artifact_cid: artifact.artifact_cid.clone(),
        policy: artifact.policy.clone(),
        admission: artifact.admission.clone(),
        representation_update: artifact.representation_update.clone(),
        predecessor_model_weights_cid: artifact.predecessor_model_weights_cid.clone(),
        model_weights_cid: artifact.model_weights_cid.clone(),
        checkpoint_tree_cid: artifact.checkpoint_tree_cid.clone(),
        config_cid: artifact.config_cid.clone(),
        tokenizer_cid: artifact.tokenizer_cid.clone(),
        supported_token_id: artifact.supported_token_id,
        unsupported_token_id: artifact.unsupported_token_id,
        dataset_cid: artifact.dataset_cid.clone(),
        split_policy_cid: artifact.split_policy_cid.clone(),
        run_contract_cid: artifact.run_contract_cid.clone(),
        training_result_cid: artifact.training_result_cid.clone(),
    };
    Ok(LoadedAttendedRelationAdapter { artifact, binding })
}

/// Score all candidates and apply the fixed duplicate-aware safety decision.
pub fn evaluate_source_relation_head(
    artifact: &SourceRelationHeadArtifact,
    candidates: &[SourceRelationCandidate<'_>],
) -> Result<SourceRelationHeadEvaluation, SourceRelationHeadError> {
    validate_artifact(artifact, EXPECTED_MODEL_WEIGHTS_CID, EXPECTED_TOKENIZER_CID)?;
    if !(2..=MAXIMUM_SOURCE_SPANS).contains(&candidates.len()) {
        return Err(SourceRelationHeadError::InvalidInput(format!(
            "candidate span count must be in 2..={MAXIMUM_SOURCE_SPANS}, observed {}",
            candidates.len()
        )));
    }

    let mut candidate_logits = Vec::with_capacity(candidates.len());
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        if candidate.span_text.is_empty() {
            return Err(SourceRelationHeadError::InvalidInput(format!(
                "candidate {candidate_index} has empty exact span text"
            )));
        }
        if candidate.final_relation_state.len() != RELATION_STATE_WIDTH
            || candidate
                .final_relation_state
                .iter()
                .any(|value| !value.is_finite())
        {
            return Err(SourceRelationHeadError::InvalidInput(format!(
                "candidate {candidate_index} state is not {RELATION_STATE_WIDTH} finite lanes"
            )));
        }
        candidate_logits.push(score_candidate(artifact, candidate.final_relation_state)?);
    }

    // One representative is retained for every exact positive span string. Its
    // occurrence is the highest-logit candidate, then earliest byte, then index.
    let mut positive_representatives = Vec::<usize>::new();
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        if candidate_logits[candidate_index] <= artifact.threshold {
            continue;
        }
        if let Some(slot) = positive_representatives
            .iter()
            .position(|&representative| candidates[representative].span_text == candidate.span_text)
        {
            let prior = positive_representatives[slot];
            if occurrence_is_better(candidate_index, prior, candidates, &candidate_logits) {
                positive_representatives[slot] = candidate_index;
            }
        } else {
            positive_representatives.push(candidate_index);
        }
    }

    let decision = match positive_representatives.as_slice() {
        [] => SourceRelationHeadDecision::Abstain,
        [candidate_index] => SourceRelationHeadDecision::Answer {
            candidate_index: *candidate_index,
        },
        [_, _, ..] => SourceRelationHeadDecision::Contradiction,
    };
    Ok(SourceRelationHeadEvaluation {
        candidate_logits,
        decision,
    })
}

/// Apply C1-SB3's fixed tied-vocabulary readout. The two rows must come from
/// the already-bound adapted checkpoint. For each final normalized residual,
/// the score is exactly `dot(state, supported_row) - dot(state,
/// unsupported_row)` in ascending lane order. No learned Rust-side head exists.
pub fn evaluate_attended_relation_adapter(
    artifact: &AttendedRelationAdapterArtifact,
    candidates: &[SourceRelationCandidate<'_>],
    supported_token_row: &[f32],
    unsupported_token_row: &[f32],
) -> Result<SourceRelationHeadEvaluation, SourceRelationHeadError> {
    validate_attended_adapter_shape(artifact)?;
    if supported_token_row.len() != RELATION_STATE_WIDTH
        || unsupported_token_row.len() != RELATION_STATE_WIDTH
        || supported_token_row.iter().any(|value| !value.is_finite())
        || unsupported_token_row.iter().any(|value| !value.is_finite())
    {
        return Err(SourceRelationHeadError::InvalidInput(format!(
            "fixed verbalizer rows must each contain {RELATION_STATE_WIDTH} finite lanes"
        )));
    }
    if !(2..=MAXIMUM_SOURCE_SPANS).contains(&candidates.len()) {
        return Err(SourceRelationHeadError::InvalidInput(format!(
            "candidate span count must be in 2..={MAXIMUM_SOURCE_SPANS}, observed {}",
            candidates.len()
        )));
    }

    let mut candidate_logits = Vec::with_capacity(candidates.len());
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        if candidate.span_text.is_empty() {
            return Err(SourceRelationHeadError::InvalidInput(format!(
                "candidate {candidate_index} has empty exact span text"
            )));
        }
        if candidate.final_relation_state.len() != RELATION_STATE_WIDTH
            || candidate
                .final_relation_state
                .iter()
                .any(|value| !value.is_finite())
        {
            return Err(SourceRelationHeadError::InvalidInput(format!(
                "candidate {candidate_index} state is not {RELATION_STATE_WIDTH} finite lanes"
            )));
        }
        candidate_logits.push(fixed_verbalizer_logit(
            candidate.final_relation_state,
            supported_token_row,
            unsupported_token_row,
        )?);
    }

    let mut positive_representatives = Vec::<usize>::new();
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        if candidate_logits[candidate_index] <= artifact.threshold {
            continue;
        }
        if let Some(slot) = positive_representatives
            .iter()
            .position(|&representative| candidates[representative].span_text == candidate.span_text)
        {
            let prior = positive_representatives[slot];
            if occurrence_is_better(candidate_index, prior, candidates, &candidate_logits) {
                positive_representatives[slot] = candidate_index;
            }
        } else {
            positive_representatives.push(candidate_index);
        }
    }
    let decision = match positive_representatives.as_slice() {
        [] => SourceRelationHeadDecision::Abstain,
        [candidate_index] => SourceRelationHeadDecision::Answer {
            candidate_index: *candidate_index,
        },
        [_, _, ..] => SourceRelationHeadDecision::Contradiction,
    };
    Ok(SourceRelationHeadEvaluation {
        candidate_logits,
        decision,
    })
}

fn fixed_verbalizer_logit(
    state: &[f32],
    supported_token_row: &[f32],
    unsupported_token_row: &[f32],
) -> Result<f32, SourceRelationHeadError> {
    let mut supported_logit = 0.0_f32;
    let mut unsupported_logit = 0.0_f32;
    for ((&state_value, &supported_weight), &unsupported_weight) in state
        .iter()
        .zip(supported_token_row)
        .zip(unsupported_token_row)
    {
        supported_logit += state_value * supported_weight;
        unsupported_logit += state_value * unsupported_weight;
        if !supported_logit.is_finite() || !unsupported_logit.is_finite() {
            return Err(SourceRelationHeadError::InvalidInput(
                "fixed verbalizer dot product overflowed or became nonfinite".to_owned(),
            ));
        }
    }
    let relation_logit = supported_logit - unsupported_logit;
    if !relation_logit.is_finite() {
        return Err(SourceRelationHeadError::InvalidInput(
            "fixed verbalizer relation logit overflowed or became nonfinite".to_owned(),
        ));
    }
    Ok(relation_logit)
}

fn score_candidate(
    artifact: &SourceRelationHeadArtifact,
    state: &[f32],
) -> Result<f32, SourceRelationHeadError> {
    let mut hidden = [0.0_f32; RELATION_HIDDEN_WIDTH];
    for (row, hidden_value) in hidden.iter_mut().enumerate() {
        let row_start = row * RELATION_STATE_WIDTH;
        let weights = &artifact.first_layer_weights[row_start..row_start + RELATION_STATE_WIDTH];
        let mut value = artifact.first_layer_biases[row];
        for (&weight, &state_value) in weights.iter().zip(state) {
            value += weight * state_value;
        }
        if !value.is_finite() {
            return Err(SourceRelationHeadError::InvalidInput(format!(
                "hidden relation lane {row} overflowed or became nonfinite"
            )));
        }
        *hidden_value = value.max(0.0);
    }

    let mut logit = artifact.output_bias;
    for (&weight, hidden_value) in artifact.output_weights.iter().zip(hidden) {
        logit += weight * hidden_value;
    }
    if !logit.is_finite() {
        return Err(SourceRelationHeadError::InvalidInput(
            "relation logit overflowed or became nonfinite".to_owned(),
        ));
    }
    Ok(logit)
}

fn occurrence_is_better(
    candidate_index: usize,
    prior_index: usize,
    candidates: &[SourceRelationCandidate<'_>],
    logits: &[f32],
) -> bool {
    match logits[candidate_index].total_cmp(&logits[prior_index]) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => {
            candidates[candidate_index].byte_start < candidates[prior_index].byte_start
                || (candidates[candidate_index].byte_start == candidates[prior_index].byte_start
                    && candidate_index < prior_index)
        }
    }
}

fn decode_artifact_bytes(
    bytes: &[u8],
    path: &Path,
    loaded_model_weights_cid: &str,
    loaded_tokenizer_cid: &str,
) -> Result<SourceRelationHeadArtifact, SourceRelationHeadError> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|error| {
        SourceRelationHeadError::InvalidArtifact(format!("{} is not JSON: {error}", path.display()))
    })?;
    verify_canonical_json_layout(bytes).map_err(|reason| {
        SourceRelationHeadError::InvalidArtifact(format!(
            "{} is not canonical JSON: {reason}",
            path.display()
        ))
    })?;
    let embedded_cid = value
        .as_object()
        .ok_or_else(|| {
            SourceRelationHeadError::InvalidArtifact(
                "artifact root is not a JSON object".to_owned(),
            )
        })?
        .get("artifact_cid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            SourceRelationHeadError::InvalidArtifact(
                "artifact_cid is absent or is not a string".to_owned(),
            )
        })?;
    verify_self_cid(bytes, embedded_cid)?;
    let artifact: SourceRelationHeadArtifact = serde_json::from_value(value).map_err(|error| {
        SourceRelationHeadError::InvalidArtifact(format!("artifact fields are invalid: {error}"))
    })?;
    validate_artifact(&artifact, loaded_model_weights_cid, loaded_tokenizer_cid)?;
    Ok(artifact)
}

fn decode_attended_adapter_bytes(
    bytes: &[u8],
    path: &Path,
    loaded_model_weights_cid: &str,
    loaded_checkpoint_tree_cid: &str,
    loaded_config_cid: &str,
    loaded_tokenizer_cid: &str,
) -> Result<AttendedRelationAdapterArtifact, SourceRelationHeadError> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|error| {
        SourceRelationHeadError::InvalidArtifact(format!("{} is not JSON: {error}", path.display()))
    })?;
    verify_canonical_json_layout(bytes).map_err(|reason| {
        SourceRelationHeadError::InvalidArtifact(format!(
            "{} is not canonical JSON: {reason}",
            path.display()
        ))
    })?;
    let embedded_cid = value
        .as_object()
        .and_then(|object| object.get("artifact_cid"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            SourceRelationHeadError::InvalidArtifact(
                "artifact_cid is absent or is not a string".to_owned(),
            )
        })?;
    verify_self_cid(bytes, embedded_cid)?;
    let artifact: AttendedRelationAdapterArtifact =
        serde_json::from_value(value).map_err(|error| {
            SourceRelationHeadError::InvalidArtifact(format!(
                "attended-adapter fields are invalid: {error}"
            ))
        })?;
    validate_attended_adapter(
        &artifact,
        loaded_model_weights_cid,
        loaded_checkpoint_tree_cid,
        loaded_config_cid,
        loaded_tokenizer_cid,
    )?;
    Ok(artifact)
}

fn verify_self_cid(bytes: &[u8], embedded_cid: &str) -> Result<(), SourceRelationHeadError> {
    if !is_blake3_cid(embedded_cid) {
        return Err(SourceRelationHeadError::InvalidArtifact(
            "artifact_cid is not a lowercase BLAKE3 CID".to_owned(),
        ));
    }
    // Remove the exact canonical member while retaining every producer-supplied
    // numeric lexeme; Python and Rust use different valid exponent spellings
    // for some finite floats, but the self-CID must bind the producer's exact
    // bytes. C1-SB2 sorts `artifact_cid` first. C1-SB3's `admission` field sorts
    // before it, so both canonical positions must reproduce without changing
    // the predecessor artifact's byte/CID rule.
    let member = format!("\"artifact_cid\":\"{embedded_cid}\"");
    let position = bytes
        .windows(member.len())
        .position(|window| window == member.as_bytes())
        .ok_or_else(|| {
            SourceRelationHeadError::InvalidArtifact(
                "canonical artifact_cid member is absent".to_owned(),
            )
        })?;
    let member_end = position + member.len();
    let mut unsigned = Vec::with_capacity(bytes.len() - member.len() - 1);
    if position == 1 && bytes.get(member_end) == Some(&b',') {
        unsigned.extend_from_slice(&bytes[..position]);
        unsigned.extend_from_slice(&bytes[member_end + 1..]);
    } else if position > 1 && bytes.get(position - 1) == Some(&b',') {
        unsigned.extend_from_slice(&bytes[..position - 1]);
        unsigned.extend_from_slice(&bytes[member_end..]);
    } else {
        return Err(SourceRelationHeadError::InvalidArtifact(
            "artifact_cid is not one canonical top-level member".to_owned(),
        ));
    }
    let observed = raw_cid(&unsigned);
    if embedded_cid != observed {
        return Err(SourceRelationHeadError::InvalidArtifact(format!(
            "artifact_cid does not reproduce: embedded {embedded_cid}, observed {observed}"
        )));
    }
    Ok(())
}

/// Verify the Python/Rust shared canonical envelope without normalizing number
/// spellings. Objects are recursively key-sorted, strings use their shortest
/// JSON escapes, no structural whitespace is allowed, and exactly one terminal
/// newline is required. The full JSON parser separately enforces number syntax.
fn verify_canonical_json_layout(bytes: &[u8]) -> Result<(), String> {
    let body = bytes
        .strip_suffix(b"\n")
        .ok_or_else(|| "one terminal newline is required".to_owned())?;
    if body.is_empty() || body.ends_with(b"\n") {
        return Err("exactly one terminal newline is required".to_owned());
    }
    let mut scanner = CanonicalJsonScanner {
        bytes: body,
        position: 0,
    };
    scanner.parse_value()?;
    if scanner.position != body.len() {
        return Err("trailing bytes follow the canonical JSON value".to_owned());
    }
    Ok(())
}

struct CanonicalJsonScanner<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl CanonicalJsonScanner<'_> {
    fn parse_value(&mut self) -> Result<(), String> {
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(|_| ()),
            Some(b't') => self.parse_literal(b"true"),
            Some(b'f') => self.parse_literal(b"false"),
            Some(b'n') => self.parse_literal(b"null"),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(byte) => Err(format!(
                "unexpected structural byte 0x{byte:02x} at {}",
                self.position
            )),
            None => Err("JSON value ended unexpectedly".to_owned()),
        }
    }

    fn parse_object(&mut self) -> Result<(), String> {
        self.expect(b'{')?;
        if self.consume(b'}') {
            return Ok(());
        }
        let mut previous_key: Option<String> = None;
        loop {
            let key = self.parse_string()?;
            if previous_key
                .as_ref()
                .is_some_and(|previous| previous >= &key)
            {
                return Err(format!(
                    "object key {key:?} is duplicate or not strictly sorted"
                ));
            }
            previous_key = Some(key);
            self.expect(b':')?;
            self.parse_value()?;
            if self.consume(b'}') {
                return Ok(());
            }
            self.expect(b',')?;
        }
    }

    fn parse_array(&mut self) -> Result<(), String> {
        self.expect(b'[')?;
        if self.consume(b']') {
            return Ok(());
        }
        loop {
            self.parse_value()?;
            if self.consume(b']') {
                return Ok(());
            }
            self.expect(b',')?;
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        let start = self.position;
        self.expect(b'"')?;
        loop {
            match self.peek() {
                Some(b'"') => {
                    self.position += 1;
                    break;
                }
                Some(b'\\') => {
                    self.position += 1;
                    let escaped = self
                        .peek()
                        .ok_or_else(|| "JSON string ended after an escape prefix".to_owned())?;
                    self.position += 1;
                    if escaped == b'u' {
                        self.position = self
                            .position
                            .checked_add(4)
                            .ok_or_else(|| "JSON string escape position overflowed".to_owned())?;
                        if self.position > self.bytes.len() {
                            return Err("JSON unicode escape ended early".to_owned());
                        }
                    }
                }
                Some(_) => self.position += 1,
                None => return Err("JSON string is unterminated".to_owned()),
            }
        }
        let token = &self.bytes[start..self.position];
        let decoded: String = serde_json::from_slice(token)
            .map_err(|error| format!("JSON string is invalid: {error}"))?;
        let canonical = serde_json::to_vec(&decoded)
            .map_err(|error| format!("JSON string cannot be canonicalized: {error}"))?;
        if canonical != token {
            return Err("JSON string does not use canonical escaping".to_owned());
        }
        Ok(decoded)
    }

    fn parse_literal(&mut self, literal: &[u8]) -> Result<(), String> {
        if self.bytes.get(self.position..self.position + literal.len()) != Some(literal) {
            return Err(format!("invalid JSON literal at byte {}", self.position));
        }
        self.position += literal.len();
        Ok(())
    }

    fn parse_number(&mut self) -> Result<(), String> {
        let start = self.position;
        while self
            .peek()
            .is_some_and(|byte| !matches!(byte, b',' | b']' | b'}'))
        {
            self.position += 1;
        }
        let token = &self.bytes[start..self.position];
        if token.iter().any(u8::is_ascii_whitespace) {
            return Err("JSON number contains structural whitespace".to_owned());
        }
        serde_json::from_slice::<serde_json::Number>(token)
            .map_err(|error| format!("JSON number is invalid: {error}"))?;
        Ok(())
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: u8) -> Result<(), String> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(format!(
                "expected structural byte {:?} at {}",
                char::from(expected),
                self.position
            ))
        }
    }
}

fn validate_artifact(
    artifact: &SourceRelationHeadArtifact,
    loaded_model_weights_cid: &str,
    loaded_tokenizer_cid: &str,
) -> Result<(), SourceRelationHeadError> {
    let invalid = |reason: &str| SourceRelationHeadError::InvalidArtifact(reason.to_owned());
    if artifact.schema != RELATION_HEAD_ARTIFACT_SCHEMA
        || artifact.policy != RELATION_HEAD_POLICY
        || artifact.issue != 954
    {
        return Err(invalid("schema, policy, or issue does not match C1-SB2"));
    }
    if loaded_model_weights_cid != EXPECTED_MODEL_WEIGHTS_CID
        || loaded_tokenizer_cid != EXPECTED_TOKENIZER_CID
        || artifact.model_weights_cid != EXPECTED_MODEL_WEIGHTS_CID
        || artifact.tokenizer_cid != EXPECTED_TOKENIZER_CID
    {
        return Err(invalid(
            "head and loaded checkpoint must match the immutable #1017 weights/tokenizer",
        ));
    }
    if artifact.hidden_size != RELATION_STATE_WIDTH
        || artifact.hidden_width != RELATION_HIDDEN_WIDTH
    {
        return Err(invalid(
            "relation head width differs from the frozen 288x32 shape",
        ));
    }
    if artifact.first_layer_weights.len() != FIRST_LAYER_WEIGHT_COUNT
        || artifact.first_layer_biases.len() != RELATION_HIDDEN_WIDTH
        || artifact.output_weights.len() != RELATION_HIDDEN_WIDTH
    {
        return Err(invalid(
            "relation head tensors differ from row-major 32x288, 32, and 32",
        ));
    }
    if artifact
        .first_layer_weights
        .iter()
        .chain(&artifact.first_layer_biases)
        .chain(&artifact.output_weights)
        .any(|value| !value.is_finite())
        || !artifact.output_bias.is_finite()
    {
        return Err(invalid("relation head parameters must all be finite"));
    }
    if artifact.threshold.to_bits() != 0.0_f32.to_bits() {
        return Err(invalid("relation threshold must be exact positive 0.0"));
    }
    if artifact.maximum_source_spans != MAXIMUM_SOURCE_SPANS
        || artifact.relation_input_policy != RELATION_INPUT_POLICY
    {
        return Err(invalid(
            "maximum span count or exact relation input policy differs",
        ));
    }
    if artifact.product_probe_commitments.len() != 4
        || artifact
            .product_probe_commitments
            .iter()
            .any(|cid| !is_blake3_cid(cid))
        || artifact
            .product_probe_commitments
            .iter()
            .enumerate()
            .any(|(index, cid)| artifact.product_probe_commitments[..index].contains(cid))
    {
        return Err(invalid(
            "exactly four distinct valid product-probe commitments are required",
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

fn validate_attended_adapter(
    artifact: &AttendedRelationAdapterArtifact,
    loaded_model_weights_cid: &str,
    loaded_checkpoint_tree_cid: &str,
    loaded_config_cid: &str,
    loaded_tokenizer_cid: &str,
) -> Result<(), SourceRelationHeadError> {
    validate_attended_adapter_shape(artifact)?;
    let invalid = |reason: &str| SourceRelationHeadError::InvalidArtifact(reason.to_owned());
    if artifact.model_weights_cid != loaded_model_weights_cid
        || artifact.checkpoint_tree_cid != loaded_checkpoint_tree_cid
        || artifact.config_cid != loaded_config_cid
        || artifact.tokenizer_cid != loaded_tokenizer_cid
    {
        return Err(invalid(
            "adapter checkpoint tree/config/tokenizer/weights do not exactly match the loaded state producer",
        ));
    }
    Ok(())
}

fn validate_attended_adapter_shape(
    artifact: &AttendedRelationAdapterArtifact,
) -> Result<(), SourceRelationHeadError> {
    let invalid = |reason: &str| SourceRelationHeadError::InvalidArtifact(reason.to_owned());
    if artifact.schema != ATTENDED_RELATION_ADAPTER_SCHEMA
        || artifact.policy != ATTENDED_RELATION_ADAPTER_POLICY
        || artifact.issue != 954
    {
        return Err(invalid("schema, policy, or issue does not match C1-SB3"));
    }
    if artifact.admission != ATTENDED_RELATION_RESEARCH_ADMISSION {
        return Err(invalid(
            "only explicit research_only C1-SB3 admission is available; no qualified tuple is registered",
        ));
    }
    if artifact.predecessor_model_weights_cid != EXPECTED_MODEL_WEIGHTS_CID
        || artifact.tokenizer_cid != EXPECTED_TOKENIZER_CID
        || artifact.config_cid != EXPECTED_MODEL_CONFIG_CID
    {
        return Err(invalid(
            "adapter must retain the immutable #1017 predecessor/config/tokenizer identity",
        ));
    }
    match artifact.representation_update.as_str() {
        ATTENDED_RELATION_UPDATE_NONE => {
            if artifact.model_weights_cid != EXPECTED_MODEL_WEIGHTS_CID {
                return Err(invalid(
                    "none_frozen_readout must use the exact #1017 predecessor weights",
                ));
            }
        }
        ATTENDED_RELATION_UPDATE_LORA => {
            if artifact.model_weights_cid == EXPECTED_MODEL_WEIGHTS_CID {
                return Err(invalid(
                    "lora_qkvo_all_layers weights must differ from the #1017 predecessor",
                ));
            }
        }
        _ => {
            return Err(invalid(
                "representation_update must be none_frozen_readout or lora_qkvo_all_layers",
            ));
        }
    }
    if artifact.hidden_size != RELATION_STATE_WIDTH {
        return Err(invalid(
            "attended relation residual width differs from the frozen width 288",
        ));
    }
    if artifact.supported_token_id != ATTENDED_RELATION_SUPPORTED_TOKEN_ID
        || artifact.unsupported_token_id != ATTENDED_RELATION_UNSUPPORTED_TOKEN_ID
        || artifact.supported_token_id == artifact.unsupported_token_id
    {
        return Err(invalid(
            "fixed supported/unsupported verbalizer token IDs differ from 1771/542",
        ));
    }
    if artifact.threshold.to_bits() != 0.0_f32.to_bits() {
        return Err(invalid("relation threshold must be exact positive 0.0"));
    }
    if artifact.maximum_source_spans != MAXIMUM_SOURCE_SPANS
        || artifact.relation_input_policy != ATTENDED_RELATION_INPUT_POLICY
    {
        return Err(invalid(
            "maximum span count or exact attended relation input policy differs",
        ));
    }
    if artifact.product_probe_commitments.len() != 4
        || artifact
            .product_probe_commitments
            .iter()
            .any(|cid| !is_blake3_cid(cid))
        || artifact
            .product_probe_commitments
            .iter()
            .enumerate()
            .any(|(index, cid)| artifact.product_probe_commitments[..index].contains(cid))
    {
        return Err(invalid(
            "exactly four distinct valid product-probe commitments are required",
        ));
    }
    for (label, cid) in [
        ("artifact", artifact.artifact_cid.as_str()),
        (
            "predecessor model weights",
            artifact.predecessor_model_weights_cid.as_str(),
        ),
        ("adapted model weights", artifact.model_weights_cid.as_str()),
        ("checkpoint tree", artifact.checkpoint_tree_cid.as_str()),
        ("config", artifact.config_cid.as_str()),
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

#[cfg(test)]
fn canonical_json_bytes(value: &serde_json::Value) -> Result<Vec<u8>, SourceRelationHeadError> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| {
        SourceRelationHeadError::InvalidArtifact(format!(
            "artifact cannot be encoded as canonical JSON: {error}"
        ))
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn raw_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn is_blake3_cid(value: &str) -> bool {
    value.len() == 71
        && value.strip_prefix("blake3:").is_some_and(|hex| {
            hex.bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cid(label: &str) -> String {
        raw_cid(label.as_bytes())
    }

    fn valid_artifact() -> SourceRelationHeadArtifact {
        let mut first_layer_weights = vec![0.0; FIRST_LAYER_WEIGHT_COUNT];
        first_layer_weights[0] = 1.0;
        let mut output_weights = vec![0.0; RELATION_HIDDEN_WIDTH];
        output_weights[0] = 1.0;
        SourceRelationHeadArtifact {
            schema: RELATION_HEAD_ARTIFACT_SCHEMA.to_owned(),
            policy: RELATION_HEAD_POLICY.to_owned(),
            issue: 954,
            model_weights_cid: EXPECTED_MODEL_WEIGHTS_CID.to_owned(),
            tokenizer_cid: EXPECTED_TOKENIZER_CID.to_owned(),
            hidden_size: RELATION_STATE_WIDTH,
            hidden_width: RELATION_HIDDEN_WIDTH,
            first_layer_weights,
            first_layer_biases: vec![0.0; RELATION_HIDDEN_WIDTH],
            output_weights,
            output_bias: -0.5,
            threshold: 0.0,
            maximum_source_spans: MAXIMUM_SOURCE_SPANS,
            relation_input_policy: RELATION_INPUT_POLICY.to_owned(),
            dataset_cid: test_cid("dataset"),
            split_policy_cid: test_cid("split"),
            run_contract_cid: test_cid("run"),
            training_result_cid: test_cid("result"),
            preflight: serde_json::json!({"status": "PASS"}),
            development_metrics: serde_json::json!({"status": "PASS"}),
            product_probe_commitments: (0..4)
                .map(|index| test_cid(&format!("probe-{index}")))
                .collect(),
            artifact_cid: test_cid("artifact-placeholder"),
        }
    }

    fn valid_attended_adapter() -> AttendedRelationAdapterArtifact {
        AttendedRelationAdapterArtifact {
            schema: ATTENDED_RELATION_ADAPTER_SCHEMA.to_owned(),
            policy: ATTENDED_RELATION_ADAPTER_POLICY.to_owned(),
            issue: 954,
            admission: ATTENDED_RELATION_RESEARCH_ADMISSION.to_owned(),
            representation_update: ATTENDED_RELATION_UPDATE_LORA.to_owned(),
            predecessor_model_weights_cid: EXPECTED_MODEL_WEIGHTS_CID.to_owned(),
            model_weights_cid: test_cid("adapted-weights"),
            checkpoint_tree_cid: test_cid("adapted-tree"),
            config_cid: EXPECTED_MODEL_CONFIG_CID.to_owned(),
            tokenizer_cid: EXPECTED_TOKENIZER_CID.to_owned(),
            hidden_size: RELATION_STATE_WIDTH,
            supported_token_id: ATTENDED_RELATION_SUPPORTED_TOKEN_ID,
            unsupported_token_id: ATTENDED_RELATION_UNSUPPORTED_TOKEN_ID,
            threshold: 0.0,
            maximum_source_spans: MAXIMUM_SOURCE_SPANS,
            relation_input_policy: ATTENDED_RELATION_INPUT_POLICY.to_owned(),
            dataset_cid: test_cid("adapter-dataset"),
            split_policy_cid: test_cid("adapter-split"),
            run_contract_cid: test_cid("adapter-run"),
            training_result_cid: test_cid("adapter-result"),
            product_probe_commitments: (0..4)
                .map(|index| test_cid(&format!("adapter-probe-{index}")))
                .collect(),
            artifact_cid: test_cid("adapter-artifact-placeholder"),
        }
    }

    fn state(first_lane: f32) -> Vec<f32> {
        let mut state = vec![0.0; RELATION_STATE_WIDTH];
        state[0] = first_lane;
        state
    }

    fn signed_value(mut value: serde_json::Value) -> Vec<u8> {
        let object = value.as_object_mut().expect("artifact object");
        object.remove("artifact_cid");
        let artifact_cid = raw_cid(&canonical_json_bytes(&value).expect("unsigned JSON"));
        value.as_object_mut().expect("artifact object").insert(
            "artifact_cid".to_owned(),
            serde_json::Value::String(artifact_cid),
        );
        canonical_json_bytes(&value).expect("signed JSON")
    }

    fn signed_artifact(artifact: &SourceRelationHeadArtifact) -> Vec<u8> {
        signed_value(serde_json::to_value(artifact).expect("artifact value"))
    }

    fn signed_attended_adapter(artifact: &AttendedRelationAdapterArtifact) -> Vec<u8> {
        signed_value(serde_json::to_value(artifact).expect("adapter value"))
    }

    #[test]
    fn relation_input_policy_ends_on_the_question_without_a_trailing_newline() {
        assert_eq!(
            render_relation_input(
                "The opal astrolabe is beneath the maple stair.",
                "Where is the opal astrolabe?"
            ),
            "Evidence:\nThe opal astrolabe is beneath the maple stair.\nQuestion:\nWhere is the opal astrolabe?"
        );
        assert_eq!(
            RELATION_INPUT_POLICY,
            "Evidence:\n<span>\nQuestion:\n<question>"
        );
    }

    #[test]
    fn attended_relation_input_ends_on_the_fixed_colon_without_a_trailing_newline() {
        assert_eq!(
            render_attended_relation_input(
                "The opal astrolabe is beneath the maple stair.",
                "Where is the opal astrolabe?"
            ),
            "Evidence:\nThe opal astrolabe is beneath the maple stair.\nQuestion:\nWhere is the opal astrolabe?\nSupported:"
        );
        assert_eq!(
            ATTENDED_RELATION_INPUT_POLICY,
            "Evidence:\n<span>\nQuestion:\n<question>\nSupported:"
        );
    }

    #[test]
    fn fixed_verbalizer_retains_exact_copy_abstain_and_conflict_decisions() {
        let artifact = valid_attended_adapter();
        let mut supported_row = vec![0.0; RELATION_STATE_WIDTH];
        supported_row[0] = 1.0;
        let unsupported_row = vec![0.0; RELATION_STATE_WIDTH];
        let positive = state(2.0);
        let zero = state(0.0);
        let duplicate = state(1.0);

        let candidates = [
            SourceRelationCandidate {
                span_text: "same supported span.",
                byte_start: 20,
                final_relation_state: &positive,
            },
            SourceRelationCandidate {
                span_text: "same supported span.",
                byte_start: 5,
                final_relation_state: &duplicate,
            },
            SourceRelationCandidate {
                span_text: "irrelevant.",
                byte_start: 50,
                final_relation_state: &zero,
            },
        ];
        let answer = evaluate_attended_relation_adapter(
            &artifact,
            &candidates,
            &supported_row,
            &unsupported_row,
        )
        .expect("fixed verbalizer answer");
        assert_eq!(answer.candidate_logits, vec![2.0, 1.0, 0.0]);
        assert_eq!(
            answer.decision,
            SourceRelationHeadDecision::Answer { candidate_index: 0 }
        );

        let abstaining = [
            SourceRelationCandidate {
                span_text: "zero.",
                byte_start: 0,
                final_relation_state: &zero,
            },
            SourceRelationCandidate {
                span_text: "also zero.",
                byte_start: 10,
                final_relation_state: &zero,
            },
        ];
        assert_eq!(
            evaluate_attended_relation_adapter(
                &artifact,
                &abstaining,
                &supported_row,
                &unsupported_row,
            )
            .expect("zero abstains")
            .decision,
            SourceRelationHeadDecision::Abstain
        );

        let conflicting = [
            SourceRelationCandidate {
                span_text: "first value.",
                byte_start: 0,
                final_relation_state: &positive,
            },
            SourceRelationCandidate {
                span_text: "second value.",
                byte_start: 20,
                final_relation_state: &duplicate,
            },
        ];
        assert_eq!(
            evaluate_attended_relation_adapter(
                &artifact,
                &conflicting,
                &supported_row,
                &unsupported_row,
            )
            .expect("distinct positives conflict")
            .decision,
            SourceRelationHeadDecision::Contradiction
        );
    }

    #[test]
    fn attended_adapter_checkpoint_bindings_and_research_admission_fail_closed() {
        let artifact = valid_attended_adapter();
        validate_attended_adapter(
            &artifact,
            &artifact.model_weights_cid,
            &artifact.checkpoint_tree_cid,
            EXPECTED_MODEL_CONFIG_CID,
            EXPECTED_TOKENIZER_CID,
        )
        .expect("exact research binding");
        assert!(validate_attended_adapter(
            &artifact,
            &test_cid("different weights"),
            &artifact.checkpoint_tree_cid,
            EXPECTED_MODEL_CONFIG_CID,
            EXPECTED_TOKENIZER_CID,
        )
        .is_err());

        let mut qualified = valid_attended_adapter();
        qualified.admission = "qualified".to_owned();
        assert!(validate_attended_adapter_shape(&qualified).is_err());

        let mut unchanged_lora = valid_attended_adapter();
        unchanged_lora.model_weights_cid = EXPECTED_MODEL_WEIGHTS_CID.to_owned();
        assert!(validate_attended_adapter_shape(&unchanged_lora).is_err());

        let mut frozen = valid_attended_adapter();
        frozen.representation_update = ATTENDED_RELATION_UPDATE_NONE.to_owned();
        frozen.model_weights_cid = EXPECTED_MODEL_WEIGHTS_CID.to_owned();
        validate_attended_adapter_shape(&frozen).expect("exact frozen-readout short circuit");

        let mut changed_frozen = frozen;
        changed_frozen.model_weights_cid = test_cid("changed-frozen-weights");
        assert!(validate_attended_adapter_shape(&changed_frozen).is_err());

        let mut unknown_update = valid_attended_adapter();
        unknown_update.representation_update = "full_finetune".to_owned();
        assert!(validate_attended_adapter_shape(&unknown_update).is_err());
    }

    #[test]
    fn attended_adapter_canonical_self_cid_and_unknown_fields_fail_closed() {
        let artifact = valid_attended_adapter();
        let bytes = signed_attended_adapter(&artifact);
        let decoded = decode_attended_adapter_bytes(
            &bytes,
            Path::new("adapter.json"),
            &artifact.model_weights_cid,
            &artifact.checkpoint_tree_cid,
            EXPECTED_MODEL_CONFIG_CID,
            EXPECTED_TOKENIZER_CID,
        )
        .expect("canonical attended adapter");
        assert_eq!(decoded.policy, ATTENDED_RELATION_ADAPTER_POLICY);

        let mut tampered: serde_json::Value =
            serde_json::from_slice(&bytes).expect("adapter value");
        tampered["supported_token_id"] = serde_json::json!(42);
        let tampered = canonical_json_bytes(&tampered).expect("tampered adapter JSON");
        assert!(decode_attended_adapter_bytes(
            &tampered,
            Path::new("tampered-adapter.json"),
            &artifact.model_weights_cid,
            &artifact.checkpoint_tree_cid,
            EXPECTED_MODEL_CONFIG_CID,
            EXPECTED_TOKENIZER_CID,
        )
        .is_err());

        let mut unknown = serde_json::to_value(valid_attended_adapter()).expect("adapter value");
        unknown
            .as_object_mut()
            .expect("adapter object")
            .insert("learned_head".to_owned(), serde_json::json!([1.0]));
        let unknown = signed_value(unknown);
        assert!(decode_attended_adapter_bytes(
            &unknown,
            Path::new("unknown-adapter.json"),
            &artifact.model_weights_cid,
            &artifact.checkpoint_tree_cid,
            EXPECTED_MODEL_CONFIG_CID,
            EXPECTED_TOKENIZER_CID,
        )
        .is_err());
    }

    #[test]
    fn dense_relu_affine_logits_use_row_major_weights() {
        let artifact = valid_artifact();
        let positive = state(2.0);
        let zero = state(0.5);
        let candidates = [
            SourceRelationCandidate {
                span_text: "positive.",
                byte_start: 0,
                final_relation_state: &positive,
            },
            SourceRelationCandidate {
                span_text: "zero.",
                byte_start: 10,
                final_relation_state: &zero,
            },
        ];
        let evaluation =
            evaluate_source_relation_head(&artifact, &candidates).expect("valid evaluation");
        assert_eq!(evaluation.candidate_logits, vec![1.5, 0.0]);
        assert_eq!(
            evaluation.decision,
            SourceRelationHeadDecision::Answer { candidate_index: 0 }
        );
    }

    #[test]
    fn exact_duplicates_count_once_and_choose_best_then_earliest_occurrence() {
        let artifact = valid_artifact();
        let one = state(1.0);
        let two = state(2.0);
        let negative = state(0.0);
        let candidates = [
            SourceRelationCandidate {
                span_text: "The jade sextant is inside the linen drawer.",
                byte_start: 40,
                final_relation_state: &one,
            },
            SourceRelationCandidate {
                span_text: "The jade sextant is inside the linen drawer.",
                byte_start: 5,
                final_relation_state: &two,
            },
            SourceRelationCandidate {
                span_text: "The jade sextant was calibrated yesterday.",
                byte_start: 90,
                final_relation_state: &negative,
            },
        ];
        let evaluation =
            evaluate_source_relation_head(&artifact, &candidates).expect("deduplicated answer");
        assert_eq!(
            evaluation.decision,
            SourceRelationHeadDecision::Answer { candidate_index: 1 }
        );

        let tied = [
            SourceRelationCandidate {
                span_text: "duplicate.",
                byte_start: 20,
                final_relation_state: &one,
            },
            SourceRelationCandidate {
                span_text: "duplicate.",
                byte_start: 5,
                final_relation_state: &one,
            },
        ];
        let evaluation =
            evaluate_source_relation_head(&artifact, &tied).expect("earliest duplicate");
        assert_eq!(
            evaluation.decision,
            SourceRelationHeadDecision::Answer { candidate_index: 1 }
        );
    }

    #[test]
    fn zero_never_authorizes_text_and_distinct_positives_conflict() {
        let artifact = valid_artifact();
        let zero = state(0.5);
        let negative = state(0.0);
        let abstaining = [
            SourceRelationCandidate {
                span_text: "zero.",
                byte_start: 0,
                final_relation_state: &zero,
            },
            SourceRelationCandidate {
                span_text: "negative.",
                byte_start: 10,
                final_relation_state: &negative,
            },
        ];
        let evaluation =
            evaluate_source_relation_head(&artifact, &abstaining).expect("safe abstention");
        assert_eq!(evaluation.candidate_logits, vec![0.0, -0.5]);
        assert_eq!(evaluation.decision, SourceRelationHeadDecision::Abstain);

        let one = state(1.0);
        let two = state(2.0);
        let conflicting = [
            SourceRelationCandidate {
                span_text: "first location.",
                byte_start: 0,
                final_relation_state: &one,
            },
            SourceRelationCandidate {
                span_text: "second location.",
                byte_start: 20,
                final_relation_state: &two,
            },
        ];
        let evaluation =
            evaluate_source_relation_head(&artifact, &conflicting).expect("safe conflict");
        assert_eq!(
            evaluation.decision,
            SourceRelationHeadDecision::Contradiction
        );
    }

    #[test]
    fn malformed_states_and_unsafe_candidate_counts_fail_closed() {
        let artifact = valid_artifact();
        let short = vec![0.0; RELATION_STATE_WIDTH - 1];
        let valid = state(0.0);
        let candidates = [
            SourceRelationCandidate {
                span_text: "short.",
                byte_start: 0,
                final_relation_state: &short,
            },
            SourceRelationCandidate {
                span_text: "valid.",
                byte_start: 10,
                final_relation_state: &valid,
            },
        ];
        assert!(matches!(
            evaluate_source_relation_head(&artifact, &candidates),
            Err(SourceRelationHeadError::InvalidInput(_))
        ));

        let singleton = [SourceRelationCandidate {
            span_text: "single.",
            byte_start: 0,
            final_relation_state: &valid,
        }];
        assert!(matches!(
            evaluate_source_relation_head(&artifact, &singleton),
            Err(SourceRelationHeadError::InvalidInput(_))
        ));

        let mut nonfinite = state(0.0);
        nonfinite[17] = f32::NAN;
        let candidates = [
            SourceRelationCandidate {
                span_text: "nonfinite.",
                byte_start: 0,
                final_relation_state: &nonfinite,
            },
            SourceRelationCandidate {
                span_text: "valid.",
                byte_start: 10,
                final_relation_state: &valid,
            },
        ];
        assert!(matches!(
            evaluate_source_relation_head(&artifact, &candidates),
            Err(SourceRelationHeadError::InvalidInput(_))
        ));
    }

    #[test]
    fn artifact_shape_threshold_and_checkpoint_bindings_fail_closed() {
        let mut artifact = valid_artifact();
        artifact.first_layer_weights.pop();
        assert!(validate_artifact(
            &artifact,
            EXPECTED_MODEL_WEIGHTS_CID,
            EXPECTED_TOKENIZER_CID
        )
        .is_err());

        let mut artifact = valid_artifact();
        artifact.threshold = -0.0;
        assert!(validate_artifact(
            &artifact,
            EXPECTED_MODEL_WEIGHTS_CID,
            EXPECTED_TOKENIZER_CID
        )
        .is_err());

        let artifact = valid_artifact();
        assert!(validate_artifact(
            &artifact,
            &test_cid("different weights"),
            EXPECTED_TOKENIZER_CID
        )
        .is_err());
    }

    #[test]
    fn canonical_self_cid_loader_rejects_reformatting_tampering_and_unknown_fields() {
        let artifact = valid_artifact();
        let bytes = signed_artifact(&artifact);
        let decoded = decode_artifact_bytes(
            &bytes,
            Path::new("fixture.json"),
            EXPECTED_MODEL_WEIGHTS_CID,
            EXPECTED_TOKENIZER_CID,
        )
        .expect("canonical artifact");
        assert_eq!(decoded.policy, RELATION_HEAD_POLICY);

        let mut pretty = serde_json::to_vec_pretty(
            &serde_json::from_slice::<serde_json::Value>(&bytes).expect("fixture value"),
        )
        .expect("pretty JSON");
        pretty.push(b'\n');
        assert!(decode_artifact_bytes(
            &pretty,
            Path::new("pretty.json"),
            EXPECTED_MODEL_WEIGHTS_CID,
            EXPECTED_TOKENIZER_CID,
        )
        .is_err());

        let mut tampered: serde_json::Value =
            serde_json::from_slice(&bytes).expect("fixture value");
        tampered["output_bias"] = serde_json::json!(-0.25);
        let tampered = canonical_json_bytes(&tampered).expect("tampered JSON");
        assert!(decode_artifact_bytes(
            &tampered,
            Path::new("tampered.json"),
            EXPECTED_MODEL_WEIGHTS_CID,
            EXPECTED_TOKENIZER_CID,
        )
        .is_err());

        let mut unknown: serde_json::Value =
            serde_json::to_value(valid_artifact()).expect("artifact value");
        unknown
            .as_object_mut()
            .expect("artifact object")
            .insert("unexpected".to_owned(), serde_json::json!(true));
        let unknown = signed_value(unknown);
        assert!(decode_artifact_bytes(
            &unknown,
            Path::new("unknown.json"),
            EXPECTED_MODEL_WEIGHTS_CID,
            EXPECTED_TOKENIZER_CID,
        )
        .is_err());
    }

    #[test]
    fn canonical_layout_accepts_python_exponents_but_rejects_spacing_and_key_reordering() {
        verify_canonical_json_layout(b"{\"a\":1e-07,\"b\":3.2000000000000005e-05}\n")
            .expect("Python number spellings remain bound as raw bytes");
        assert!(verify_canonical_json_layout(b"{\"a\": 1}\n").is_err());
        assert!(verify_canonical_json_layout(b"{\"a\":1 ,\"b\":2}\n").is_err());
        assert!(verify_canonical_json_layout(b"{\"b\":1,\"a\":2}\n").is_err());
        assert!(verify_canonical_json_layout(b"{\"a\":1}\n\n").is_err());
    }
}
