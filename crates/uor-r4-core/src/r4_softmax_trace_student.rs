//! Bounded source-free suffix distillation from the qualified R4 softmax oracle.
//!
//! This module is deliberately a geometry-free baseline. The compiler consumes
//! construction-only token/teacher traces and emits three matched fixed-point
//! suffix tables: teacher-distilled, observed-count, and document-permuted
//! teacher control. The prediction/distribution/continuation path reads only
//! the artifact and token history and performs no source-model calls, softmax,
//! floating-point arithmetic, or sampling. The separate evaluation methods use
//! compiler-side floating point for cross-entropy.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

const ARTIFACT_MAGIC: [u8; 8] = *b"R4STU001";
const ARTIFACT_VERSION: u32 = 1;
const ARTIFACT_HEADER_LEN: usize = 72;
const ROW_PREFIX_LEN: usize = 24;
const CANDIDATE_LEN: usize = 12;
const MIN_CANDIDATE_CAP: u16 = 3;
const MAX_CANDIDATE_CAP: u16 = 256;
const MAX_CONTINUATION_TOKENS: usize = 4096;

pub const R4_SOFTMAX_TRACE_STUDENT_SCHEMA: &str = "R4SoftmaxTraceStudentV1";
pub const R4_SOFTMAX_TRACE_MAX_SUFFIX_DEPTH: u8 = 4;
pub const R4_SOFTMAX_TRACE_Q16_TOTAL: u16 = u16::MAX;
pub const R4_SOFTMAX_TRACE_DOCUMENT_PERMUTATION_POLICY: &str =
    "sort-document-id;donor=next-cyclic-document;position=floor(target-position*donor-length/target-length)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum R4SoftmaxTraceStudentError {
    Invalid(String),
    ArithmeticOverflow,
}

impl std::fmt::Display for R4SoftmaxTraceStudentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::ArithmeticOverflow => {
                formatter.write_str("R4 softmax trace student arithmetic overflow")
            }
        }
    }
}

impl std::error::Error for R4SoftmaxTraceStudentError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeacherTopTokenQ16 {
    pub token: u32,
    pub probability_q16: u16,
}

impl TeacherTopTokenQ16 {
    pub const fn new(token: u32, probability_q16: u16) -> Self {
        Self {
            token,
            probability_q16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeacherTopDistributionQ16 {
    pub entries: Vec<TeacherTopTokenQ16>,
}

impl TeacherTopDistributionQ16 {
    pub fn new(entries: Vec<TeacherTopTokenQ16>) -> Result<Self, R4SoftmaxTraceStudentError> {
        validate_teacher_distribution(&entries, MAX_CANDIDATE_CAP)?;
        Ok(Self { entries })
    }

    pub fn top_token(&self) -> Result<u32, R4SoftmaxTraceStudentError> {
        teacher_top_token(&self.entries)
    }
}

/// One construction or evaluation sequence.
///
/// Event `i` uses `input_tokens[..=i]` as its causal history and aligns it with
/// `actual_next_tokens[i]` and `teacher_top_distributions[i]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct R4SoftmaxTraceSequence {
    pub document_id: String,
    pub input_tokens: Vec<u32>,
    pub actual_next_tokens: Vec<u32>,
    pub teacher_top_distributions: Vec<TeacherTopDistributionQ16>,
}

impl R4SoftmaxTraceSequence {
    pub fn new(
        document_id: impl Into<String>,
        input_tokens: Vec<u32>,
        actual_next_tokens: Vec<u32>,
        teacher_top_distributions: Vec<TeacherTopDistributionQ16>,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            input_tokens,
            actual_next_tokens,
            teacher_top_distributions,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct R4SoftmaxTraceStudentConfig {
    pub candidate_cap: u16,
}

impl R4SoftmaxTraceStudentConfig {
    pub fn new(candidate_cap: usize) -> Result<Self, R4SoftmaxTraceStudentError> {
        let candidate_cap = u16::try_from(candidate_cap)
            .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
        validate_candidate_cap(candidate_cap)?;
        Ok(Self { candidate_cap })
    }
}

impl Default for R4SoftmaxTraceStudentConfig {
    fn default() -> Self {
        Self { candidate_cap: 32 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum R4SoftmaxTraceStudentArm {
    TeacherDistilled,
    ObservedCount,
    DocumentPermutedControl,
}

impl R4SoftmaxTraceStudentArm {
    const fn index(self) -> usize {
        match self {
            Self::TeacherDistilled => 0,
            Self::ObservedCount => 1,
            Self::DocumentPermutedControl => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SuffixKey {
    depth: u8,
    tokens: [u32; R4_SOFTMAX_TRACE_MAX_SUFFIX_DEPTH as usize],
}

impl SuffixKey {
    fn from_history(history: &[u32], depth: usize) -> Result<Self, R4SoftmaxTraceStudentError> {
        if depth > usize::from(R4_SOFTMAX_TRACE_MAX_SUFFIX_DEPTH) || depth > history.len() {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace suffix depth exceeds its history".to_owned(),
            ));
        }
        let mut tokens = [0; R4_SOFTMAX_TRACE_MAX_SUFFIX_DEPTH as usize];
        if depth != 0 {
            tokens[..depth].copy_from_slice(&history[history.len() - depth..]);
        }
        Ok(Self {
            depth: u8::try_from(depth)
                .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?,
            tokens,
        })
    }

    fn validate(self) -> Result<(), R4SoftmaxTraceStudentError> {
        let depth = usize::from(self.depth);
        if depth > usize::from(R4_SOFTMAX_TRACE_MAX_SUFFIX_DEPTH)
            || self.tokens[depth..].iter().any(|&token| token != 0)
        {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace suffix key is not canonical".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StudentCandidate {
    token: u32,
    weights_q16: [u16; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StudentRow {
    candidates: Vec<StudentCandidate>,
}

impl StudentRow {
    fn distribution(&self, arm: R4SoftmaxTraceStudentArm) -> Vec<R4SoftmaxTraceStudentScore> {
        let index = arm.index();
        self.candidates
            .iter()
            .map(|candidate| R4SoftmaxTraceStudentScore {
                token: candidate.token,
                weight_q16: candidate.weights_q16[index],
            })
            .collect()
    }

    fn predict(
        &self,
        arm: R4SoftmaxTraceStudentArm,
        depth: u8,
    ) -> Result<R4SoftmaxTraceStudentPrediction, R4SoftmaxTraceStudentError> {
        let index = arm.index();
        let mut winner = None;
        for candidate in &self.candidates {
            let score = candidate.weights_q16[index];
            if winner.is_none_or(|(_, best_score)| score > best_score) {
                winner = Some((candidate.token, score));
            }
        }
        let (token, weight_q16) = winner.ok_or_else(|| {
            R4SoftmaxTraceStudentError::Invalid("R4 softmax trace student row is empty".to_owned())
        })?;
        Ok(R4SoftmaxTraceStudentPrediction {
            token,
            weight_q16,
            suffix_depth: depth,
            candidate_count: u16::try_from(self.candidates.len())
                .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R4SoftmaxTraceStudentArtifact {
    candidate_cap: u16,
    construction_document_count: u32,
    construction_position_count: u64,
    construction_digest: [u8; 32],
    rows: BTreeMap<SuffixKey, StudentRow>,
}

impl R4SoftmaxTraceStudentArtifact {
    pub fn candidate_cap(&self) -> u16 {
        self.candidate_cap
    }

    pub fn construction_document_count(&self) -> u32 {
        self.construction_document_count
    }

    pub fn construction_position_count(&self) -> u64 {
        self.construction_position_count
    }

    pub fn construction_digest(&self) -> [u8; 32] {
        self.construction_digest
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn rows_at_depth(&self, depth: u8) -> usize {
        self.rows.keys().filter(|key| key.depth == depth).count()
    }

    pub fn artifact_cid(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.to_bytes()).to_hex())
    }

    /// Canonical manual little-endian representation.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(
            ARTIFACT_HEADER_LEN
                + self.rows.len() * ROW_PREFIX_LEN
                + self
                    .rows
                    .values()
                    .map(|row| row.candidates.len() * CANDIDATE_LEN)
                    .sum::<usize>(),
        );
        bytes.extend_from_slice(&ARTIFACT_MAGIC);
        push_u32(&mut bytes, ARTIFACT_VERSION);
        push_u32(&mut bytes, u32::from(R4_SOFTMAX_TRACE_MAX_SUFFIX_DEPTH));
        push_u32(&mut bytes, u32::from(self.candidate_cap));
        push_u32(&mut bytes, self.construction_document_count);
        push_u64(&mut bytes, self.construction_position_count);
        push_u32(
            &mut bytes,
            u32::try_from(self.rows.len()).expect("validated trace row count exceeds u32"),
        );
        push_u32(&mut bytes, 0);
        bytes.extend_from_slice(&self.construction_digest);
        debug_assert_eq!(bytes.len(), ARTIFACT_HEADER_LEN);

        for (key, row) in &self.rows {
            bytes.push(key.depth);
            bytes.extend_from_slice(&[0; 3]);
            for &token in &key.tokens {
                push_u32(&mut bytes, token);
            }
            push_u32(
                &mut bytes,
                u32::try_from(row.candidates.len())
                    .expect("validated trace candidate count exceeds u32"),
            );
            for candidate in &row.candidates {
                push_u32(&mut bytes, candidate.token);
                for &weight in &candidate.weights_q16 {
                    push_u16(&mut bytes, weight);
                }
                push_u16(&mut bytes, 0);
            }
        }
        bytes
    }

    /// Fail-closed loader for the canonical artifact representation.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, R4SoftmaxTraceStudentError> {
        if bytes.len() < ARTIFACT_HEADER_LEN || bytes[..8] != ARTIFACT_MAGIC {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student magic/header is invalid".to_owned(),
            ));
        }
        let mut cursor = Cursor::new(bytes);
        cursor.take(8)?;
        if cursor.u32()? != ARTIFACT_VERSION {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student version is unsupported".to_owned(),
            ));
        }
        if cursor.u32()? != u32::from(R4_SOFTMAX_TRACE_MAX_SUFFIX_DEPTH) {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student suffix depth is unsupported".to_owned(),
            ));
        }
        let candidate_cap = u16::try_from(cursor.u32()?)
            .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
        validate_candidate_cap(candidate_cap)?;
        let construction_document_count = cursor.u32()?;
        let construction_position_count = cursor.u64()?;
        let row_count = cursor.count("R4 softmax trace row")?;
        if cursor.u32()? != 0 {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student reserved header field is nonzero".to_owned(),
            ));
        }
        let construction_digest: [u8; 32] = cursor
            .take(32)?
            .try_into()
            .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
        if row_count == 0 || row_count > cursor.remaining() / ROW_PREFIX_LEN {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student row count is invalid".to_owned(),
            ));
        }

        let mut rows = BTreeMap::new();
        let mut previous_key = None;
        for _ in 0..row_count {
            let depth = cursor.u8()?;
            if cursor.take(3)? != [0; 3] {
                return Err(R4SoftmaxTraceStudentError::Invalid(
                    "R4 softmax trace row reserved bytes are nonzero".to_owned(),
                ));
            }
            let mut tokens = [0; R4_SOFTMAX_TRACE_MAX_SUFFIX_DEPTH as usize];
            for token in &mut tokens {
                *token = cursor.u32()?;
            }
            let key = SuffixKey { depth, tokens };
            key.validate()?;
            if previous_key.is_some_and(|previous| previous >= key) {
                return Err(R4SoftmaxTraceStudentError::Invalid(
                    "R4 softmax trace rows are not strictly sorted".to_owned(),
                ));
            }
            previous_key = Some(key);
            let candidate_count = cursor.count("R4 softmax trace candidate")?;
            if candidate_count == 0
                || candidate_count > usize::from(candidate_cap)
                || candidate_count > cursor.remaining() / CANDIDATE_LEN
            {
                return Err(R4SoftmaxTraceStudentError::Invalid(
                    "R4 softmax trace row candidate count is invalid".to_owned(),
                ));
            }
            let mut candidates = Vec::with_capacity(candidate_count);
            let mut previous_token = None;
            for _ in 0..candidate_count {
                let token = cursor.u32()?;
                if previous_token.is_some_and(|previous| previous >= token) {
                    return Err(R4SoftmaxTraceStudentError::Invalid(
                        "R4 softmax trace candidates are not strictly sorted".to_owned(),
                    ));
                }
                previous_token = Some(token);
                let weights_q16 = [cursor.u16()?, cursor.u16()?, cursor.u16()?];
                if cursor.u16()? != 0 {
                    return Err(R4SoftmaxTraceStudentError::Invalid(
                        "R4 softmax trace candidate reserved field is nonzero".to_owned(),
                    ));
                }
                candidates.push(StudentCandidate { token, weights_q16 });
            }
            rows.insert(key, StudentRow { candidates });
        }
        if !cursor.is_finished() {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student has trailing bytes".to_owned(),
            ));
        }

        let artifact = Self {
            candidate_cap,
            construction_document_count,
            construction_position_count,
            construction_digest,
            rows,
        };
        artifact.validate()?;
        if artifact.to_bytes() != bytes {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student is not canonical".to_owned(),
            ));
        }
        Ok(artifact)
    }

    pub fn runtime(&self) -> R4SoftmaxTraceStudentRuntime {
        R4SoftmaxTraceStudentRuntime {
            artifact: self.clone(),
        }
    }

    pub fn evaluate(
        &self,
        sequences: &[R4SoftmaxTraceSequence],
    ) -> Result<R4SoftmaxTraceStudentEvaluation, R4SoftmaxTraceStudentError> {
        self.runtime().evaluate(sequences)
    }

    fn validate(&self) -> Result<(), R4SoftmaxTraceStudentError> {
        validate_candidate_cap(self.candidate_cap)?;
        if self.construction_document_count < 2 || self.construction_position_count == 0 {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student construction census is invalid".to_owned(),
            ));
        }
        if self.rows.is_empty() {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student has no rows".to_owned(),
            ));
        }
        let mut depth_zero_rows = 0_u8;
        for (key, row) in &self.rows {
            key.validate()?;
            if key.depth == 0 {
                depth_zero_rows = depth_zero_rows
                    .checked_add(1)
                    .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
            }
            validate_student_row(row, self.candidate_cap)?;
        }
        if depth_zero_rows != 1 {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student requires one depth-zero row".to_owned(),
            ));
        }
        Ok(())
    }

    fn row_for_history(&self, history: &[u32]) -> Option<(u8, &StudentRow)> {
        let max_depth = history
            .len()
            .min(usize::from(R4_SOFTMAX_TRACE_MAX_SUFFIX_DEPTH));
        for depth in (0..=max_depth).rev() {
            let key = SuffixKey::from_history(history, depth).ok()?;
            if let Some(row) = self.rows.get(&key) {
                return Some((key.depth, row));
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct R4SoftmaxTraceStudentPrediction {
    pub token: u32,
    pub weight_q16: u16,
    pub suffix_depth: u8,
    pub candidate_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct R4SoftmaxTraceStudentScore {
    pub token: u32,
    pub weight_q16: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct R4SoftmaxTraceStudentDistribution {
    pub suffix_depth: u8,
    pub scores: Vec<R4SoftmaxTraceStudentScore>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct R4SoftmaxTraceArmEvaluation {
    pub positions: u64,
    pub row_covered_positions: u64,
    pub nonzero_suffix_positions: u64,
    pub suffix_depth_histogram: [u64; 5],
    pub actual_token_covered_positions: u64,
    pub teacher_mass_total_q16: u64,
    pub teacher_mass_covered_q16: u64,
    pub teacher_top1_agreements: u64,
    pub actual_token_top1_correct: u64,
    /// Full bounded-teacher cross-entropy in natural-log units. `None` means
    /// at least one positive teacher token was outside the capped row support.
    pub teacher_cross_entropy_nats: Option<f64>,
    /// Cross-entropy over the teacher mass retained by the shared row support.
    pub covered_teacher_cross_entropy_nats: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct R4SoftmaxTraceStudentEvaluation {
    pub teacher_distilled: R4SoftmaxTraceArmEvaluation,
    pub observed_count: R4SoftmaxTraceArmEvaluation,
    pub document_permuted_control: R4SoftmaxTraceArmEvaluation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R4SoftmaxTraceStudentRuntime {
    artifact: R4SoftmaxTraceStudentArtifact,
}

impl R4SoftmaxTraceStudentRuntime {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, R4SoftmaxTraceStudentError> {
        Ok(Self {
            artifact: R4SoftmaxTraceStudentArtifact::from_bytes(bytes)?,
        })
    }

    pub fn artifact_cid(&self) -> String {
        self.artifact.artifact_cid()
    }

    pub fn predict(
        &self,
        history: &[u32],
        arm: R4SoftmaxTraceStudentArm,
    ) -> Result<R4SoftmaxTraceStudentPrediction, R4SoftmaxTraceStudentError> {
        let (depth, row) = self.artifact.row_for_history(history).ok_or_else(|| {
            R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student has no suffix row for prediction".to_owned(),
            )
        })?;
        row.predict(arm, depth)
    }

    pub fn distribution(
        &self,
        history: &[u32],
        arm: R4SoftmaxTraceStudentArm,
    ) -> Result<R4SoftmaxTraceStudentDistribution, R4SoftmaxTraceStudentError> {
        let (depth, row) = self.artifact.row_for_history(history).ok_or_else(|| {
            R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student has no suffix row for distribution".to_owned(),
            )
        })?;
        Ok(R4SoftmaxTraceStudentDistribution {
            suffix_depth: depth,
            scores: row.distribution(arm),
        })
    }

    pub fn continue_tokens(
        &self,
        history: &[u32],
        arm: R4SoftmaxTraceStudentArm,
        max_new_tokens: usize,
    ) -> Result<Vec<u32>, R4SoftmaxTraceStudentError> {
        if max_new_tokens > MAX_CONTINUATION_TOKENS {
            return Err(R4SoftmaxTraceStudentError::Invalid(format!(
                "R4 softmax trace continuation exceeds {MAX_CONTINUATION_TOKENS} tokens"
            )));
        }
        let mut context = history.to_vec();
        let mut generated = Vec::with_capacity(max_new_tokens);
        for _ in 0..max_new_tokens {
            let token = self.predict(&context, arm)?.token;
            generated.push(token);
            context.push(token);
        }
        Ok(generated)
    }

    pub fn evaluate(
        &self,
        sequences: &[R4SoftmaxTraceSequence],
    ) -> Result<R4SoftmaxTraceStudentEvaluation, R4SoftmaxTraceStudentError> {
        validate_sequences(sequences, self.artifact.candidate_cap, false)?;
        Ok(R4SoftmaxTraceStudentEvaluation {
            teacher_distilled: self
                .evaluate_arm(sequences, R4SoftmaxTraceStudentArm::TeacherDistilled)?,
            observed_count: self
                .evaluate_arm(sequences, R4SoftmaxTraceStudentArm::ObservedCount)?,
            document_permuted_control: self
                .evaluate_arm(sequences, R4SoftmaxTraceStudentArm::DocumentPermutedControl)?,
        })
    }

    fn evaluate_arm(
        &self,
        sequences: &[R4SoftmaxTraceSequence],
        arm: R4SoftmaxTraceStudentArm,
    ) -> Result<R4SoftmaxTraceArmEvaluation, R4SoftmaxTraceStudentError> {
        let mut accumulator = EvaluationAccumulator::default();
        for sequence in sorted_sequences(sequences) {
            for position in 0..sequence.input_tokens.len() {
                let history = &sequence.input_tokens[..=position];
                let (depth, row) = self.artifact.row_for_history(history).ok_or_else(|| {
                    R4SoftmaxTraceStudentError::Invalid(
                        "R4 softmax trace student has no evaluation row".to_owned(),
                    )
                })?;
                let prediction = row.predict(arm, depth)?;
                let teacher = &sequence.teacher_top_distributions[position];
                let actual = sequence.actual_next_tokens[position];
                accumulator.record(row, arm, prediction, teacher, actual)?;
            }
        }
        accumulator.finish()
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct RawCandidate {
    teacher: u64,
    count: u64,
    permuted: u64,
}

impl RawCandidate {
    fn score(self, arm: R4SoftmaxTraceStudentArm) -> u64 {
        match arm {
            R4SoftmaxTraceStudentArm::TeacherDistilled => self.teacher,
            R4SoftmaxTraceStudentArm::ObservedCount => self.count,
            R4SoftmaxTraceStudentArm::DocumentPermutedControl => self.permuted,
        }
    }

    fn combined(self) -> u128 {
        u128::from(self.teacher) + u128::from(self.count) + u128::from(self.permuted)
    }
}

type RawRow = BTreeMap<u32, RawCandidate>;

/// Compile all three matched source-free suffix arms in one artifact.
pub fn compile_r4_softmax_trace_student(
    config: R4SoftmaxTraceStudentConfig,
    construction: &[R4SoftmaxTraceSequence],
) -> Result<R4SoftmaxTraceStudentArtifact, R4SoftmaxTraceStudentError> {
    validate_candidate_cap(config.candidate_cap)?;
    let construction_position_count = validate_sequences(construction, config.candidate_cap, true)?;
    let documents = sorted_sequences(construction);
    let construction_digest = construction_digest(config, &documents)?;
    let mut raw_rows: BTreeMap<SuffixKey, RawRow> = BTreeMap::new();

    for (document_index, sequence) in documents.iter().enumerate() {
        let donor = documents[(document_index + 1) % documents.len()];
        for position in 0..sequence.input_tokens.len() {
            let donor_position = permuted_position(
                position,
                sequence.input_tokens.len(),
                donor.input_tokens.len(),
            )?;
            let teacher = &sequence.teacher_top_distributions[position];
            let permuted = &donor.teacher_top_distributions[donor_position];
            let actual = sequence.actual_next_tokens[position];
            let history = &sequence.input_tokens[..=position];
            let max_depth = history
                .len()
                .min(usize::from(R4_SOFTMAX_TRACE_MAX_SUFFIX_DEPTH));
            for depth in 0..=max_depth {
                let key = SuffixKey::from_history(history, depth)?;
                let row = raw_rows.entry(key).or_default();
                for entry in &teacher.entries {
                    checked_add_to(
                        &mut row.entry(entry.token).or_default().teacher,
                        u64::from(entry.probability_q16),
                    )?;
                }
                checked_add_to(
                    &mut row.entry(actual).or_default().count,
                    u64::from(R4_SOFTMAX_TRACE_Q16_TOTAL),
                )?;
                for entry in &permuted.entries {
                    checked_add_to(
                        &mut row.entry(entry.token).or_default().permuted,
                        u64::from(entry.probability_q16),
                    )?;
                }
            }
        }
    }

    let rows = raw_rows
        .into_iter()
        .map(|(key, raw)| Ok((key, compile_row(&raw, config.candidate_cap)?)))
        .collect::<Result<BTreeMap<_, _>, R4SoftmaxTraceStudentError>>()?;
    let artifact = R4SoftmaxTraceStudentArtifact {
        candidate_cap: config.candidate_cap,
        construction_document_count: u32::try_from(documents.len())
            .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?,
        construction_position_count,
        construction_digest,
        rows,
    };
    artifact.validate()?;
    Ok(artifact)
}

fn compile_row(raw: &RawRow, candidate_cap: u16) -> Result<StudentRow, R4SoftmaxTraceStudentError> {
    if raw.is_empty() {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "cannot compile an empty R4 softmax trace row".to_owned(),
        ));
    }
    let cap = usize::from(candidate_cap);
    let mut selected = BTreeSet::new();
    for arm in [
        R4SoftmaxTraceStudentArm::TeacherDistilled,
        R4SoftmaxTraceStudentArm::ObservedCount,
        R4SoftmaxTraceStudentArm::DocumentPermutedControl,
    ] {
        selected.insert(raw_winner(raw, arm)?);
    }
    if selected.len() > cap {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "R4 softmax trace candidate cap cannot retain all arm winners".to_owned(),
        ));
    }
    let mut ranked = raw
        .iter()
        .map(|(&token, &weights)| (token, weights.combined()))
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    for (token, _) in ranked {
        if selected.len() == cap {
            break;
        }
        selected.insert(token);
    }
    let tokens = selected.into_iter().collect::<Vec<_>>();
    let mut normalized = [Vec::new(), Vec::new(), Vec::new()];
    for arm in [
        R4SoftmaxTraceStudentArm::TeacherDistilled,
        R4SoftmaxTraceStudentArm::ObservedCount,
        R4SoftmaxTraceStudentArm::DocumentPermutedControl,
    ] {
        let values = tokens
            .iter()
            .map(|token| raw.get(token).copied().unwrap_or_default().score(arm))
            .collect::<Vec<_>>();
        normalized[arm.index()] = normalize_q16(&tokens, &values)?;
    }
    let candidates = tokens
        .into_iter()
        .enumerate()
        .map(|(index, token)| StudentCandidate {
            token,
            weights_q16: [
                normalized[0][index],
                normalized[1][index],
                normalized[2][index],
            ],
        })
        .collect::<Vec<_>>();
    let row = StudentRow { candidates };
    validate_student_row(&row, candidate_cap)?;
    Ok(row)
}

fn raw_winner(
    row: &RawRow,
    arm: R4SoftmaxTraceStudentArm,
) -> Result<u32, R4SoftmaxTraceStudentError> {
    let mut winner = None;
    for (&token, &candidate) in row {
        let score = candidate.score(arm);
        if score != 0 && winner.is_none_or(|(_, best_score)| score > best_score) {
            winner = Some((token, score));
        }
    }
    winner.map(|(token, _)| token).ok_or_else(|| {
        R4SoftmaxTraceStudentError::Invalid(
            "R4 softmax trace arm has no positive construction evidence".to_owned(),
        )
    })
}

/// Normalize raw evidence to exactly 65535 while assigning the same one-unit
/// Q16 floor to every member of the shared candidate support.
fn normalize_q16(tokens: &[u32], raw: &[u64]) -> Result<Vec<u16>, R4SoftmaxTraceStudentError> {
    if tokens.is_empty() || tokens.len() != raw.len() || tokens.len() > usize::from(u16::MAX) {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "R4 softmax trace Q16 normalization shape is invalid".to_owned(),
        ));
    }
    let total = raw.iter().try_fold(0_u128, |sum, &value| {
        sum.checked_add(u128::from(value))
            .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)
    })?;
    if total == 0 {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "R4 softmax trace Q16 normalization has zero evidence".to_owned(),
        ));
    }
    let floor_total =
        u32::try_from(tokens.len()).map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
    let distributable = u32::from(R4_SOFTMAX_TRACE_Q16_TOTAL)
        .checked_sub(floor_total)
        .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
    let mut normalized = Vec::with_capacity(raw.len());
    let mut remainders = Vec::with_capacity(raw.len());
    let mut assigned = floor_total;
    for (index, (&token, &value)) in tokens.iter().zip(raw).enumerate() {
        let numerator = u128::from(value)
            .checked_mul(u128::from(distributable))
            .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
        let quotient = u32::try_from(numerator / total)
            .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
        assigned = assigned
            .checked_add(quotient)
            .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
        normalized.push(
            u16::try_from(quotient + 1)
                .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?,
        );
        remainders.push((numerator % total, token, index));
    }
    let remaining = u32::from(R4_SOFTMAX_TRACE_Q16_TOTAL)
        .checked_sub(assigned)
        .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
    if usize::try_from(remaining).map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?
        > remainders.len()
    {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "R4 softmax trace Q16 remainder is invalid".to_owned(),
        ));
    }
    remainders.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    for &(_, _, index) in remainders.iter().take(
        usize::try_from(remaining).map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?,
    ) {
        normalized[index] = normalized[index]
            .checked_add(1)
            .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
    }
    let normalized_total = normalized
        .iter()
        .map(|&value| u32::from(value))
        .sum::<u32>();
    if normalized_total != u32::from(R4_SOFTMAX_TRACE_Q16_TOTAL) {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "R4 softmax trace Q16 normalization did not close".to_owned(),
        ));
    }
    Ok(normalized)
}

fn validate_student_row(
    row: &StudentRow,
    candidate_cap: u16,
) -> Result<(), R4SoftmaxTraceStudentError> {
    if row.candidates.is_empty() || row.candidates.len() > usize::from(candidate_cap) {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "R4 softmax trace student row shape is invalid".to_owned(),
        ));
    }
    if row
        .candidates
        .windows(2)
        .any(|pair| pair[0].token >= pair[1].token)
    {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "R4 softmax trace student candidates are not canonical".to_owned(),
        ));
    }
    for arm_index in 0..3 {
        let mut total = 0_u32;
        for candidate in &row.candidates {
            let weight = candidate.weights_q16[arm_index];
            if weight == 0 {
                return Err(R4SoftmaxTraceStudentError::Invalid(
                    "R4 softmax trace student candidate has zero Q16 support".to_owned(),
                ));
            }
            total = total
                .checked_add(u32::from(weight))
                .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
        }
        if total != u32::from(R4_SOFTMAX_TRACE_Q16_TOTAL) {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student arm is not Q16-normalized".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_candidate_cap(candidate_cap: u16) -> Result<(), R4SoftmaxTraceStudentError> {
    if !(MIN_CANDIDATE_CAP..=MAX_CANDIDATE_CAP).contains(&candidate_cap) {
        return Err(R4SoftmaxTraceStudentError::Invalid(format!(
            "R4 softmax trace candidate cap must be in {MIN_CANDIDATE_CAP}..={MAX_CANDIDATE_CAP}"
        )));
    }
    Ok(())
}

fn validate_sequences(
    sequences: &[R4SoftmaxTraceSequence],
    candidate_cap: u16,
    require_permutation: bool,
) -> Result<u64, R4SoftmaxTraceStudentError> {
    if sequences.is_empty() || (require_permutation && sequences.len() < 2) {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "R4 softmax trace compilation requires at least two construction documents".to_owned(),
        ));
    }
    let mut document_ids = BTreeSet::new();
    let mut positions = 0_u64;
    for sequence in sequences {
        if sequence.document_id.is_empty() || sequence.document_id.len() > 4096 {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace document id is empty or oversized".to_owned(),
            ));
        }
        if !document_ids.insert(sequence.document_id.as_str()) {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace document ids are duplicated".to_owned(),
            ));
        }
        let event_count = sequence.input_tokens.len();
        if event_count == 0
            || sequence.actual_next_tokens.len() != event_count
            || sequence.teacher_top_distributions.len() != event_count
        {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace sequence shapes are not aligned".to_owned(),
            ));
        }
        positions = positions
            .checked_add(
                u64::try_from(event_count)
                    .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?,
            )
            .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
        for distribution in &sequence.teacher_top_distributions {
            validate_teacher_distribution(&distribution.entries, candidate_cap)?;
        }
    }
    Ok(positions)
}

fn validate_teacher_distribution(
    entries: &[TeacherTopTokenQ16],
    candidate_cap: u16,
) -> Result<(), R4SoftmaxTraceStudentError> {
    if entries.is_empty() || entries.len() > usize::from(candidate_cap) {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "bounded teacher top-token distribution shape is invalid".to_owned(),
        ));
    }
    let mut tokens = BTreeSet::new();
    let mut total = 0_u32;
    for entry in entries {
        if entry.probability_q16 == 0 || !tokens.insert(entry.token) {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "bounded teacher top-token distribution has zero or duplicate entries".to_owned(),
            ));
        }
        total = total
            .checked_add(u32::from(entry.probability_q16))
            .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
    }
    if total != u32::from(R4_SOFTMAX_TRACE_Q16_TOTAL) {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "bounded teacher top-token distribution is not Q16-normalized".to_owned(),
        ));
    }
    Ok(())
}

fn teacher_top_token(entries: &[TeacherTopTokenQ16]) -> Result<u32, R4SoftmaxTraceStudentError> {
    let mut winner = None;
    for entry in entries {
        if winner.is_none_or(|(token, best)| {
            entry.probability_q16 > best || (entry.probability_q16 == best && entry.token < token)
        }) {
            winner = Some((entry.token, entry.probability_q16));
        }
    }
    winner.map(|(token, _)| token).ok_or_else(|| {
        R4SoftmaxTraceStudentError::Invalid(
            "bounded teacher top-token distribution is empty".to_owned(),
        )
    })
}

fn sorted_sequences(sequences: &[R4SoftmaxTraceSequence]) -> Vec<&R4SoftmaxTraceSequence> {
    let mut sorted = sequences.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.document_id.cmp(&right.document_id));
    sorted
}

fn permuted_position(
    target_position: usize,
    target_length: usize,
    donor_length: usize,
) -> Result<usize, R4SoftmaxTraceStudentError> {
    if target_length == 0 || donor_length == 0 || target_position >= target_length {
        return Err(R4SoftmaxTraceStudentError::Invalid(
            "R4 softmax trace permutation shape is invalid".to_owned(),
        ));
    }
    let numerator = (target_position as u128)
        .checked_mul(donor_length as u128)
        .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
    usize::try_from(numerator / target_length as u128)
        .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)
}

fn construction_digest(
    config: R4SoftmaxTraceStudentConfig,
    documents: &[&R4SoftmaxTraceSequence],
) -> Result<[u8; 32], R4SoftmaxTraceStudentError> {
    let mut bytes = Vec::new();
    push_length_prefixed(&mut bytes, R4_SOFTMAX_TRACE_STUDENT_SCHEMA.as_bytes())?;
    push_length_prefixed(
        &mut bytes,
        R4_SOFTMAX_TRACE_DOCUMENT_PERMUTATION_POLICY.as_bytes(),
    )?;
    push_u16(&mut bytes, config.candidate_cap);
    push_u16(&mut bytes, 0);
    push_u32(
        &mut bytes,
        u32::try_from(documents.len())
            .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?,
    );
    for sequence in documents {
        push_length_prefixed(&mut bytes, sequence.document_id.as_bytes())?;
        push_u32(
            &mut bytes,
            u32::try_from(sequence.input_tokens.len())
                .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?,
        );
        for position in 0..sequence.input_tokens.len() {
            push_u32(&mut bytes, sequence.input_tokens[position]);
            push_u32(&mut bytes, sequence.actual_next_tokens[position]);
            let mut entries = sequence.teacher_top_distributions[position].entries.clone();
            entries.sort_by_key(|entry| entry.token);
            push_u16(
                &mut bytes,
                u16::try_from(entries.len())
                    .map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?,
            );
            push_u16(&mut bytes, 0);
            for entry in entries {
                push_u32(&mut bytes, entry.token);
                push_u16(&mut bytes, entry.probability_q16);
                push_u16(&mut bytes, 0);
            }
        }
    }
    Ok(*blake3::hash(&bytes).as_bytes())
}

fn checked_add_to(target: &mut u64, value: u64) -> Result<(), R4SoftmaxTraceStudentError> {
    *target = target
        .checked_add(value)
        .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
    Ok(())
}

#[derive(Default)]
struct EvaluationAccumulator {
    positions: u64,
    row_covered_positions: u64,
    nonzero_suffix_positions: u64,
    suffix_depth_histogram: [u64; 5],
    actual_token_covered_positions: u64,
    teacher_mass_total_q16: u64,
    teacher_mass_covered_q16: u64,
    teacher_top1_agreements: u64,
    actual_token_top1_correct: u64,
    covered_cross_entropy_nats: f64,
    all_teacher_mass_covered: bool,
}

impl EvaluationAccumulator {
    fn record(
        &mut self,
        row: &StudentRow,
        arm: R4SoftmaxTraceStudentArm,
        prediction: R4SoftmaxTraceStudentPrediction,
        teacher: &TeacherTopDistributionQ16,
        actual: u32,
    ) -> Result<(), R4SoftmaxTraceStudentError> {
        if self.positions == 0 {
            self.all_teacher_mass_covered = true;
        }
        self.positions = increment(self.positions)?;
        self.row_covered_positions = increment(self.row_covered_positions)?;
        if prediction.suffix_depth != 0 {
            self.nonzero_suffix_positions = increment(self.nonzero_suffix_positions)?;
        }
        let depth_index = usize::from(prediction.suffix_depth);
        self.suffix_depth_histogram[depth_index] =
            increment(self.suffix_depth_histogram[depth_index])?;
        if row
            .candidates
            .binary_search_by_key(&actual, |candidate| candidate.token)
            .is_ok()
        {
            self.actual_token_covered_positions = increment(self.actual_token_covered_positions)?;
        }
        if prediction.token == teacher.top_token()? {
            self.teacher_top1_agreements = increment(self.teacher_top1_agreements)?;
        }
        if prediction.token == actual {
            self.actual_token_top1_correct = increment(self.actual_token_top1_correct)?;
        }

        let arm_index = arm.index();
        for entry in &teacher.entries {
            self.teacher_mass_total_q16 = self
                .teacher_mass_total_q16
                .checked_add(u64::from(entry.probability_q16))
                .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
            let candidate = row
                .candidates
                .binary_search_by_key(&entry.token, |candidate| candidate.token)
                .ok()
                .map(|index| row.candidates[index]);
            let Some(candidate) = candidate else {
                self.all_teacher_mass_covered = false;
                continue;
            };
            self.teacher_mass_covered_q16 = self
                .teacher_mass_covered_q16
                .checked_add(u64::from(entry.probability_q16))
                .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
            let teacher_probability =
                f64::from(entry.probability_q16) / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL);
            let student_probability =
                f64::from(candidate.weights_q16[arm_index]) / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL);
            self.covered_cross_entropy_nats -= teacher_probability * student_probability.ln();
        }
        Ok(())
    }

    fn finish(self) -> Result<R4SoftmaxTraceArmEvaluation, R4SoftmaxTraceStudentError> {
        if self.positions == 0 || self.teacher_mass_total_q16 == 0 {
            return Err(R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace evaluation is empty".to_owned(),
            ));
        }
        let full_cross_entropy = self
            .all_teacher_mass_covered
            .then_some(self.covered_cross_entropy_nats / self.positions as f64);
        let covered_cross_entropy = (self.teacher_mass_covered_q16 != 0).then_some(
            self.covered_cross_entropy_nats
                / (self.teacher_mass_covered_q16 as f64 / f64::from(R4_SOFTMAX_TRACE_Q16_TOTAL)),
        );
        Ok(R4SoftmaxTraceArmEvaluation {
            positions: self.positions,
            row_covered_positions: self.row_covered_positions,
            nonzero_suffix_positions: self.nonzero_suffix_positions,
            suffix_depth_histogram: self.suffix_depth_histogram,
            actual_token_covered_positions: self.actual_token_covered_positions,
            teacher_mass_total_q16: self.teacher_mass_total_q16,
            teacher_mass_covered_q16: self.teacher_mass_covered_q16,
            teacher_top1_agreements: self.teacher_top1_agreements,
            actual_token_top1_correct: self.actual_token_top1_correct,
            teacher_cross_entropy_nats: full_cross_entropy,
            covered_teacher_cross_entropy_nats: covered_cross_entropy,
        })
    }
}

fn increment(value: u64) -> Result<u64, R4SoftmaxTraceStudentError> {
    value
        .checked_add(1)
        .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)
}

fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_length_prefixed(
    bytes: &mut Vec<u8>,
    value: &[u8],
) -> Result<(), R4SoftmaxTraceStudentError> {
    push_u32(
        bytes,
        u32::try_from(value.len()).map_err(|_| R4SoftmaxTraceStudentError::ArithmeticOverflow)?,
    );
    bytes.extend_from_slice(value);
    Ok(())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], R4SoftmaxTraceStudentError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(R4SoftmaxTraceStudentError::ArithmeticOverflow)?;
        let value = self.bytes.get(self.offset..end).ok_or_else(|| {
            R4SoftmaxTraceStudentError::Invalid(
                "R4 softmax trace student artifact is truncated".to_owned(),
            )
        })?;
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, R4SoftmaxTraceStudentError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, R4SoftmaxTraceStudentError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().map_err(
            |_| R4SoftmaxTraceStudentError::ArithmeticOverflow,
        )?))
    }

    fn u32(&mut self) -> Result<u32, R4SoftmaxTraceStudentError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().map_err(
            |_| R4SoftmaxTraceStudentError::ArithmeticOverflow,
        )?))
    }

    fn u64(&mut self) -> Result<u64, R4SoftmaxTraceStudentError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().map_err(
            |_| R4SoftmaxTraceStudentError::ArithmeticOverflow,
        )?))
    }

    fn count(&mut self, label: &str) -> Result<usize, R4SoftmaxTraceStudentError> {
        usize::try_from(self.u32()?).map_err(|_| {
            R4SoftmaxTraceStudentError::Invalid(format!("{label} count exceeds this host"))
        })
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn distribution(primary: u32, secondary: u32) -> TeacherTopDistributionQ16 {
        TeacherTopDistributionQ16::new(vec![
            TeacherTopTokenQ16::new(primary, 50_000),
            TeacherTopTokenQ16::new(secondary, 15_535),
        ])
        .expect("valid teacher distribution")
    }

    fn fixture() -> Vec<R4SoftmaxTraceSequence> {
        vec![
            R4SoftmaxTraceSequence::new(
                "document-a",
                vec![10, 11],
                vec![20, 21],
                vec![distribution(20, 30), distribution(21, 31)],
            ),
            R4SoftmaxTraceSequence::new(
                "document-b",
                vec![10, 12],
                vec![40, 41],
                vec![distribution(40, 50), distribution(41, 51)],
            ),
        ]
    }

    #[test]
    fn compilation_roundtrip_and_runtime_replay_are_exact() {
        let artifact = compile_r4_softmax_trace_student(
            R4SoftmaxTraceStudentConfig::new(8).expect("config"),
            &fixture(),
        )
        .expect("compile");
        assert_eq!(artifact.rows_at_depth(0), 1);
        assert!(artifact.rows_at_depth(1) >= 1);
        assert!(artifact.rows_at_depth(2) >= 1);
        assert_eq!(artifact.rows_at_depth(3), 0);
        assert_eq!(artifact.rows_at_depth(4), 0);

        let bytes = artifact.to_bytes();
        let loaded = R4SoftmaxTraceStudentArtifact::from_bytes(&bytes).expect("roundtrip");
        assert_eq!(loaded.to_bytes(), bytes);
        assert_eq!(loaded.artifact_cid(), artifact.artifact_cid());

        let runtime = R4SoftmaxTraceStudentRuntime::from_bytes(&bytes).expect("runtime");
        assert_eq!(
            runtime
                .predict(&[10, 11], R4SoftmaxTraceStudentArm::TeacherDistilled)
                .expect("teacher prediction")
                .token,
            21
        );
        assert_eq!(
            runtime
                .predict(&[10, 11], R4SoftmaxTraceStudentArm::ObservedCount)
                .expect("count prediction")
                .token,
            21
        );
        assert_eq!(
            runtime
                .predict(&[10, 11], R4SoftmaxTraceStudentArm::DocumentPermutedControl,)
                .expect("control prediction")
                .token,
            41
        );
        let first = runtime
            .continue_tokens(&[10, 11], R4SoftmaxTraceStudentArm::TeacherDistilled, 6)
            .expect("continue");
        let replay = R4SoftmaxTraceStudentRuntime::from_bytes(&bytes)
            .expect("replay runtime")
            .continue_tokens(&[10, 11], R4SoftmaxTraceStudentArm::TeacherDistilled, 6)
            .expect("replay continue");
        assert_eq!(first, replay);
    }

    #[test]
    fn document_order_and_teacher_entry_order_do_not_change_bytes() {
        let config = R4SoftmaxTraceStudentConfig::new(8).expect("config");
        let first = fixture();
        let mut reordered = fixture();
        reordered.reverse();
        for sequence in &mut reordered {
            for distribution in &mut sequence.teacher_top_distributions {
                distribution.entries.reverse();
            }
        }
        let first = compile_r4_softmax_trace_student(config, &first).expect("first");
        let reordered = compile_r4_softmax_trace_student(config, &reordered).expect("reordered");
        assert_eq!(first.to_bytes(), reordered.to_bytes());
        assert_eq!(first.artifact_cid(), reordered.artifact_cid());
    }

    #[test]
    fn all_arms_share_support_and_evaluate_without_source_state() {
        let fixture = fixture();
        let artifact = compile_r4_softmax_trace_student(
            R4SoftmaxTraceStudentConfig::new(8).expect("config"),
            &fixture,
        )
        .expect("compile");
        let runtime = artifact.runtime();
        let histories = [&[10_u32, 11][..], &[10_u32, 12][..]];
        for history in histories {
            let teacher = runtime
                .distribution(history, R4SoftmaxTraceStudentArm::TeacherDistilled)
                .expect("teacher distribution");
            let count = runtime
                .distribution(history, R4SoftmaxTraceStudentArm::ObservedCount)
                .expect("count distribution");
            let control = runtime
                .distribution(history, R4SoftmaxTraceStudentArm::DocumentPermutedControl)
                .expect("control distribution");
            let support = |distribution: &R4SoftmaxTraceStudentDistribution| {
                distribution
                    .scores
                    .iter()
                    .map(|score| score.token)
                    .collect::<Vec<_>>()
            };
            assert_eq!(support(&teacher), support(&count));
            assert_eq!(support(&teacher), support(&control));
            for distribution in [teacher, count, control] {
                assert_eq!(
                    distribution
                        .scores
                        .iter()
                        .map(|score| u32::from(score.weight_q16))
                        .sum::<u32>(),
                    u32::from(R4_SOFTMAX_TRACE_Q16_TOTAL)
                );
            }
        }

        let evaluation = runtime.evaluate(&fixture).expect("evaluation");
        assert_eq!(evaluation.teacher_distilled.positions, 4);
        assert_eq!(evaluation.teacher_distilled.row_covered_positions, 4);
        assert_eq!(
            evaluation.teacher_distilled.actual_token_covered_positions,
            4
        );
        assert_eq!(evaluation.teacher_distilled.teacher_top1_agreements, 3);
        assert_eq!(evaluation.teacher_distilled.actual_token_top1_correct, 3);
        assert_eq!(
            evaluation
                .teacher_distilled
                .suffix_depth_histogram
                .iter()
                .sum::<u64>(),
            4
        );
        assert!(evaluation
            .teacher_distilled
            .teacher_cross_entropy_nats
            .is_some());
        assert!(evaluation
            .observed_count
            .teacher_cross_entropy_nats
            .is_some());
        assert!(evaluation
            .document_permuted_control
            .teacher_cross_entropy_nats
            .is_some());
        assert!(
            evaluation.teacher_distilled.teacher_cross_entropy_nats
                < evaluation
                    .document_permuted_control
                    .teacher_cross_entropy_nats
        );
    }

    #[test]
    fn malformed_shapes_and_bytes_fail_closed() {
        let config = R4SoftmaxTraceStudentConfig::new(8).expect("config");
        assert!(compile_r4_softmax_trace_student(config, &fixture()[..1]).is_err());
        let mut malformed = fixture();
        malformed[0].actual_next_tokens.pop();
        assert!(compile_r4_softmax_trace_student(config, &malformed).is_err());
        let mut malformed = fixture();
        malformed[0].teacher_top_distributions[0].entries[0].probability_q16 = 49_999;
        assert!(compile_r4_softmax_trace_student(config, &malformed).is_err());
        let mut malformed = fixture();
        malformed[1].document_id = malformed[0].document_id.clone();
        assert!(compile_r4_softmax_trace_student(config, &malformed).is_err());

        let artifact = compile_r4_softmax_trace_student(config, &fixture()).expect("compile");
        let bytes = artifact.to_bytes();
        assert!(R4SoftmaxTraceStudentArtifact::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(R4SoftmaxTraceStudentArtifact::from_bytes(&trailing).is_err());
        let mut bad_magic = bytes.clone();
        bad_magic[0] ^= 1;
        assert!(R4SoftmaxTraceStudentArtifact::from_bytes(&bad_magic).is_err());
        let mut bad_reserved = bytes;
        bad_reserved[36] = 1;
        assert!(R4SoftmaxTraceStudentArtifact::from_bytes(&bad_reserved).is_err());
    }

    #[test]
    fn shared_cap_preserves_each_arm_winner() {
        let artifact = compile_r4_softmax_trace_student(
            R4SoftmaxTraceStudentConfig::new(3).expect("config"),
            &fixture(),
        )
        .expect("compile");
        let runtime = artifact.runtime();
        assert_eq!(
            runtime
                .predict(&[10, 11], R4SoftmaxTraceStudentArm::TeacherDistilled)
                .expect("teacher")
                .token,
            21
        );
        assert_eq!(
            runtime
                .predict(&[10, 11], R4SoftmaxTraceStudentArm::ObservedCount)
                .expect("count")
                .token,
            21
        );
        assert_eq!(
            runtime
                .predict(&[10, 11], R4SoftmaxTraceStudentArm::DocumentPermutedControl,)
                .expect("control")
                .token,
            41
        );
    }
}
