//! Fail-closed, source-bound answers over the working #1017 local generator.
//!
//! The model may propose text, `ABSTAIN`, or `CONTRADICTION`, but this surface
//! serves ordinary text only when it is an exact, case-sensitive contiguous
//! byte span of the caller-supplied source. The complete underlying generation
//! report remains nested for audit without exposing unsupported text on stdout.

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::r4_softmax_local_generation::{
    run_r4_softmax_local_generation, R4SoftmaxLocalGenerationError, R4SoftmaxLocalGenerationReport,
    R4SoftmaxLocalGeneratorConfig, MAX_NEW_TOKENS,
};

pub const REPORT_SCHEMA: &str = "uor-r4.grounded-answer/1";
pub const PROMPT_POLICY: &str = "R4GroundedExtractivePromptV1";
pub const DEFAULT_MAX_NEW_TOKENS: usize = 32;
pub const MAX_SOURCE_BYTES: usize = 4 * 1024;
pub const MAX_QUESTION_BYTES: usize = 1024;

pub const PROMPT_POLICY_TEMPLATE: &str = concat!(
    "Use only the context. Copy one exact contiguous answer span from the context. ",
    "If the context does not answer the question, write ABSTAIN. ",
    "If the context gives conflicting answers, write CONTRADICTION.",
    "\nContext:\n{source}\nQuestion:\n{question}\nAnswer:\n",
);

const PROMPT_POLICY_INSTRUCTION: &str = concat!(
    "Use only the context. Copy one exact contiguous answer span from the context. ",
    "If the context does not answer the question, write ABSTAIN. ",
    "If the context gives conflicting answers, write CONTRADICTION.",
);

#[derive(Clone, Debug)]
pub struct GroundedAnswerConfig {
    pub model: PathBuf,
    pub source_file: PathBuf,
    pub question: String,
    pub max_new_tokens: usize,
    pub workers: NonZeroUsize,
    pub seed: Option<u64>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroundedAbstentionReason {
    EmptyGeneratedText,
    InsufficientEvidence,
    UnsupportedGeneratedText,
}

impl GroundedAbstentionReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyGeneratedText => "empty_generated_text",
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::UnsupportedGeneratedText => "unsupported_generated_text",
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
    prompt_policy: &'static str,
    prompt_policy_cid: &'a str,
    assembled_prompt_cid: &'a str,
    source_cid: &'a str,
    source_byte_length: usize,
    question: &'a str,
    inner_decision_cid: &'a str,
    inner_output_cid: &'a str,
    inner_audit_cid: &'a str,
    outcome: &'a GroundedAnswerOutcome,
}

#[derive(Serialize)]
pub struct GroundedAnswerReport {
    pub schema: String,
    pub decision_cid: String,
    pub claim_scope: String,
    pub prompt_policy: String,
    pub prompt_policy_cid: String,
    pub assembled_prompt_cid: String,
    pub source: GroundingSourceBinding,
    pub question: String,
    pub outcome: GroundedAnswerOutcome,
    /// The full #1017 generation report, including the raw candidate and every
    /// causal-attention, execution, checkpoint, and decode audit.
    pub generation: R4SoftmaxLocalGenerationReport,
    pub nonclaims: Vec<String>,
}

#[derive(Debug)]
pub enum GroundedAnswerError {
    InvalidRequest(String),
    InvalidSource(String),
    Generation(R4SoftmaxLocalGenerationError),
    Audit(String),
    Io(io::Error),
}

impl fmt::Display for GroundedAnswerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(reason) => write!(formatter, "invalid grounded answer: {reason}"),
            Self::InvalidSource(reason) => write!(formatter, "invalid grounding source: {reason}"),
            Self::Generation(error) => error.fmt(formatter),
            Self::Audit(reason) => write!(formatter, "grounded answer audit failed: {reason}"),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for GroundedAnswerError {}

impl From<R4SoftmaxLocalGenerationError> for GroundedAnswerError {
    fn from(error: R4SoftmaxLocalGenerationError) -> Self {
        Self::Generation(error)
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
    let source_before = read_source_file(&config.source_file)?;
    let source_text = std::str::from_utf8(&source_before).map_err(|error| {
        GroundedAnswerError::InvalidSource(format!(
            "{} is not exact UTF-8: {error}",
            config.source_file.display()
        ))
    })?;
    let source_cid = raw_cid(&source_before);
    let prompt = assemble_prompt(source_text, &config.question);
    let prompt_policy_cid = raw_cid(PROMPT_POLICY_TEMPLATE.as_bytes());
    let assembled_prompt_cid = raw_cid(prompt.as_bytes());

    let generation = run_r4_softmax_local_generation(&R4SoftmaxLocalGeneratorConfig {
        model: config.model.clone(),
        prompt,
        max_new_tokens: config.max_new_tokens,
        workers: config.workers,
        attention_off: false,
        seed: config.seed,
    })?;

    let source_after = read_source_file(&config.source_file)?;
    if source_before != source_after {
        return Err(GroundedAnswerError::Audit(format!(
            "source {} changed during generation (before {}, after {})",
            config.source_file.display(),
            source_cid,
            raw_cid(&source_after)
        )));
    }

    let outcome = if generation.transcript.utf8_decodable {
        classify_candidate(source_text, &generation.transcript.response_text)
    } else {
        GroundedAnswerOutcome::Abstained {
            reason: GroundedAbstentionReason::UnsupportedGeneratedText,
        }
    };
    let source = GroundingSourceBinding {
        path: config.source_file.display().to_string(),
        source_cid,
        byte_length: source_before.len(),
        regular_non_symlink: true,
        utf8: true,
        reads: 2,
        unchanged_after_run: true,
    };
    let decision_cid = cid_serializable(&GroundedDecisionIdentity {
        schema: REPORT_SCHEMA,
        prompt_policy: PROMPT_POLICY,
        prompt_policy_cid: &prompt_policy_cid,
        assembled_prompt_cid: &assembled_prompt_cid,
        source_cid: &source.source_cid,
        source_byte_length: source.byte_length,
        question: &config.question,
        inner_decision_cid: &generation.decision_cid,
        inner_output_cid: &generation.output_cid,
        inner_audit_cid: &generation.audit_cid,
        outcome: &outcome,
    })?;

    Ok(GroundedAnswerReport {
        schema: REPORT_SCHEMA.to_owned(),
        decision_cid,
        claim_scope: "fail-closed local #1017 answer serving with exact source-byte provenance and exact case-sensitive contiguous-span admission".to_owned(),
        prompt_policy: PROMPT_POLICY.to_owned(),
        prompt_policy_cid,
        assembled_prompt_cid,
        source,
        question: config.question.clone(),
        outcome,
        generation,
        nonclaims: vec![
            "Exact source-span membership establishes provenance, not semantic entailment or general correctness.".to_owned(),
            "A CONTRADICTION result is the model's typed response to the fixed prompt; this wrapper does not independently prove a semantic conflict.".to_owned(),
            "This #1017 path remains source-backed, floating-point, multiplication-using, and ordinary-softmax based.".to_owned(),
        ],
    })
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
/// This check runs before generation so report emission cannot overwrite the
/// source after the unchanged-source audit has completed.
pub fn require_distinct_output_path(
    source: &Path,
    output: &Path,
) -> Result<(), GroundedAnswerError> {
    let source_metadata = std::fs::metadata(source).map_err(|error| {
        GroundedAnswerError::InvalidSource(format!(
            "cannot inspect {} before selecting audit output: {error}",
            source.display()
        ))
    })?;
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
    if config.max_new_tokens == 0 || config.max_new_tokens > MAX_NEW_TOKENS {
        return Err(GroundedAnswerError::InvalidRequest(format!(
            "--max-new-tokens must be in 1..={MAX_NEW_TOKENS}"
        )));
    }
    Ok(())
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

fn assemble_prompt(source: &str, question: &str) -> String {
    format!("{PROMPT_POLICY_INSTRUCTION}\nContext:\n{source}\nQuestion:\n{question}\nAnswer:\n")
}

fn classify_candidate(source: &str, candidate: &str) -> GroundedAnswerOutcome {
    let candidate = candidate.trim();
    if candidate.is_empty() {
        return GroundedAnswerOutcome::Abstained {
            reason: GroundedAbstentionReason::EmptyGeneratedText,
        };
    }
    if candidate == "ABSTAIN" {
        return GroundedAnswerOutcome::Abstained {
            reason: GroundedAbstentionReason::InsufficientEvidence,
        };
    }
    if candidate == "CONTRADICTION" {
        return GroundedAnswerOutcome::Contradiction;
    }
    if let Some(byte_start) = source.find(candidate) {
        return GroundedAnswerOutcome::Answered {
            answer: candidate.to_owned(),
            source_span: SourceSpan {
                byte_start,
                byte_end: byte_start + candidate.len(),
            },
        };
    }
    GroundedAnswerOutcome::Abstained {
        reason: GroundedAbstentionReason::UnsupportedGeneratedText,
    }
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
    fn classifier_serves_only_exact_case_sensitive_source_spans() {
        let source = "The silver key opens the north door.";
        assert_eq!(
            classify_candidate(source, "silver key"),
            GroundedAnswerOutcome::Answered {
                answer: "silver key".to_owned(),
                source_span: SourceSpan {
                    byte_start: 4,
                    byte_end: 14,
                },
            }
        );
        assert_eq!(
            classify_candidate(source, "Silver key"),
            GroundedAnswerOutcome::Abstained {
                reason: GroundedAbstentionReason::UnsupportedGeneratedText,
            }
        );
    }

    #[test]
    fn classifier_types_every_non_answer_terminal() {
        assert_eq!(
            classify_candidate("source", "  "),
            GroundedAnswerOutcome::Abstained {
                reason: GroundedAbstentionReason::EmptyGeneratedText,
            }
        );
        assert_eq!(
            classify_candidate("source", " ABSTAIN "),
            GroundedAnswerOutcome::Abstained {
                reason: GroundedAbstentionReason::InsufficientEvidence,
            }
        );
        assert_eq!(
            classify_candidate("source", " CONTRADICTION "),
            GroundedAnswerOutcome::Contradiction
        );
        assert_eq!(
            classify_candidate("source", "unsupported"),
            GroundedAnswerOutcome::Abstained {
                reason: GroundedAbstentionReason::UnsupportedGeneratedText,
            }
        );
    }

    #[test]
    fn prompt_assembly_matches_the_frozen_policy() {
        assert_eq!(
            assemble_prompt("exact source", "exact question"),
            PROMPT_POLICY_TEMPLATE
                .replace("{source}", "exact source")
                .replace("{question}", "exact question")
        );
    }
}
