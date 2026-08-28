//! Bounded higher-scope geometric attention for issue #973.
//!
//! `PriorSentenceCountRadiusR4V1` leaves the source-free table and the #953
//! count-radius overlay byte-for-byte unchanged. It can only rank the maximum-
//! count tie already admitted by #953. The fourth R4 coordinate is replaced by
//! exact candidate occupancy in the prefix before the final fitted period.
//! This is deliberately an exact lexical attention/copy mechanism, not a
//! semantic-similarity or general paragraph-attention claim.

use crate::source_free_table::{
    BackoffOrder, Continuation, ContinuationStop, MatchedGeometricPrediction,
    MultiscaleCountRadiusCandidate, MultiscaleCountRadiusCoordinates, MultiscaleCountRadiusR4V1,
    MultiscaleCountRadiusWork, SourceFreeTable, SourceFreeTableError, BOS_TOKEN, EOS_TOKEN,
    MAX_CONTINUATION_UNITS,
};

const OPERATOR_MAGIC: [u8; 8] = *b"SFTHSA01";
const OPERATOR_VERSION: u32 = 1;
const OPERATOR_HEADER_LEN: usize = 144;
const Q32_SCALE: u128 = 1_u128 << 32;
const TRIGRAM_DEPTH_Q32: u64 = 3_u64 << 30;
const BIGRAM_DEPTH_Q32: u64 = 2_u64 << 30;

pub const MAX_PRIOR_PREFIX_UNITS: usize = 64;
pub const MAX_PRIOR_TIED_CANDIDATES: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HigherScopeGeometricAttentionError {
    Invalid(String),
    SourceFree(String),
    ArithmeticOverflow,
}

impl std::fmt::Display for HigherScopeGeometricAttentionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::SourceFree(reason) => write!(formatter, "source-free table: {reason}"),
            Self::ArithmeticOverflow => {
                formatter.write_str("higher-scope attention arithmetic overflow")
            }
        }
    }
}

impl std::error::Error for HigherScopeGeometricAttentionError {}

impl From<SourceFreeTableError> for HigherScopeGeometricAttentionError {
    fn from(error: SourceFreeTableError) -> Self {
        Self::SourceFree(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PriorSentenceCountRadiusArm {
    Real,
    ScopeDisabled,
    CandidatePermuted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PriorSentenceCountRadiusAbstention {
    LocalGeometryIneligible,
    MissingSentenceBoundary,
    NoPriorCandidateOccurrence,
    RadiusTie,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PriorSentenceCountRadiusCandidateEvidence {
    pub token: u32,
    pub count: u64,
    pub local_coordinates: MultiscaleCountRadiusCoordinates,
    pub local_radius: u128,
    pub prior_count: u32,
    pub real_prior_q32: u64,
    pub real_radius: u128,
    pub disabled_prior_q32: u64,
    pub disabled_radius: u128,
    pub permuted_prior_count: u32,
    pub permuted_prior_q32: u64,
    pub permuted_radius: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct PriorSentenceCountRadiusWork {
    pub local: MultiscaleCountRadiusWork,
    pub prior_prefix_units_scanned: u64,
    pub candidate_membership_checks: u64,
    pub normalization_table_reads: u64,
    pub square_table_reads: u64,
    pub coordinate_replacements: u64,
    pub radius_comparisons: u64,
    pub final_choice_operations: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PriorSentenceCountRadiusDecision {
    pub arm: PriorSentenceCountRadiusArm,
    pub token: u32,
    pub unique_radius_winner: Option<u32>,
    pub support_tokens: Vec<u32>,
    pub work: PriorSentenceCountRadiusWork,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MatchedPriorSentenceCountRadiusPrediction {
    pub local: MatchedGeometricPrediction,
    pub sentence_boundary_token: u32,
    pub sentence_boundary_index: Option<usize>,
    pub prior_prefix_units: usize,
    pub prior_candidate_occurrences: u32,
    pub candidate_evidence: Vec<PriorSentenceCountRadiusCandidateEvidence>,
    pub real: PriorSentenceCountRadiusDecision,
    pub scope_disabled: PriorSentenceCountRadiusDecision,
    pub candidate_permuted: PriorSentenceCountRadiusDecision,
    pub operator_abstention: Option<PriorSentenceCountRadiusAbstention>,
    pub support_matched: bool,
    pub work_matched: bool,
    pub teacher_calls: u64,
    pub provider_calls: u64,
    pub source_weight_reads: u64,
    pub future_unit_reads: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MatchedPriorSentenceCountRadiusContinuation {
    pub first_decision: MatchedPriorSentenceCountRadiusPrediction,
    pub real: Continuation,
    pub scope_disabled: Continuation,
    pub candidate_permuted: Continuation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NormalizationCell {
    count: u32,
    q32: u64,
    square: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizationRow {
    total: u32,
    cells: Vec<NormalizationCell>,
}

/// Canonical, construction-bound lookup artifact for the frozen Gate 0
/// higher-scope operator. Multiplication, division, and fixed-point squaring
/// happen only in [`Self::compile`]; prediction reads the stored cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorSentenceCountRadiusR4V1 {
    table_artifact_hash: [u8; 32],
    base_overlay_artifact_hash: [u8; 32],
    sentence_boundary_token: u32,
    max_prior_prefix_units: u32,
    max_tied_candidates: u32,
    trigram_depth_square: u128,
    bigram_depth_square: u128,
    normalization_rows: Vec<NormalizationRow>,
}

impl PriorSentenceCountRadiusR4V1 {
    pub fn compile(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
    ) -> Result<Self, HigherScopeGeometricAttentionError> {
        if base_overlay.table_artifact_cid() != table.artifact_cid() {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "#953 overlay table binding mismatches".to_owned(),
            ));
        }
        let boundary = table.encode_text(b".")?;
        if boundary.len() != 1
            || table.decode_tokens(&boundary)? != b"."
            || !table.is_fitted_lexical_token(boundary[0])
        {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "the fitted period sentence-boundary token is unavailable".to_owned(),
            ));
        }
        let mut normalization_rows = Vec::with_capacity(MAX_PRIOR_PREFIX_UNITS);
        for total in 1..=u32::try_from(MAX_PRIOR_PREFIX_UNITS)
            .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?
        {
            let mut cells = Vec::with_capacity(
                usize::try_from(total)
                    .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?
                    .checked_add(1)
                    .ok_or(HigherScopeGeometricAttentionError::ArithmeticOverflow)?,
            );
            for count in 0..=total {
                let numerator = u128::from(count)
                    .checked_mul(Q32_SCALE)
                    .ok_or(HigherScopeGeometricAttentionError::ArithmeticOverflow)?;
                let q32 = u64::try_from(numerator / u128::from(total))
                    .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?;
                let square = u128::from(q32)
                    .checked_mul(u128::from(q32))
                    .ok_or(HigherScopeGeometricAttentionError::ArithmeticOverflow)?;
                cells.push(NormalizationCell { count, q32, square });
            }
            normalization_rows.push(NormalizationRow { total, cells });
        }
        Ok(Self {
            table_artifact_hash: cid_digest(&table.artifact_cid())?,
            base_overlay_artifact_hash: cid_digest(&base_overlay.artifact_cid())?,
            sentence_boundary_token: boundary[0],
            max_prior_prefix_units: u32::try_from(MAX_PRIOR_PREFIX_UNITS)
                .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?,
            max_tied_candidates: u32::try_from(MAX_PRIOR_TIED_CANDIDATES)
                .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?,
            trigram_depth_square: compile_square(TRIGRAM_DEPTH_Q32)?,
            bigram_depth_square: compile_square(BIGRAM_DEPTH_Q32)?,
            normalization_rows,
        })
    }

    pub fn table_artifact_cid(&self) -> String {
        format!("blake3:{}", hex::encode(self.table_artifact_hash))
    }

    pub fn base_overlay_artifact_cid(&self) -> String {
        format!("blake3:{}", hex::encode(self.base_overlay_artifact_hash))
    }

    pub fn artifact_cid(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.to_bytes()).to_hex())
    }

    pub fn sentence_boundary_token(&self) -> u32 {
        self.sentence_boundary_token
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&OPERATOR_MAGIC);
        push_u32(&mut bytes, OPERATOR_VERSION);
        bytes.extend_from_slice(&self.table_artifact_hash);
        bytes.extend_from_slice(&self.base_overlay_artifact_hash);
        push_u32(&mut bytes, self.sentence_boundary_token);
        push_u32(&mut bytes, self.max_prior_prefix_units);
        push_u32(&mut bytes, self.max_tied_candidates);
        push_u32(&mut bytes, usize_u32(self.normalization_rows.len()));
        push_u64(&mut bytes, TRIGRAM_DEPTH_Q32);
        push_u128(&mut bytes, self.trigram_depth_square);
        push_u64(&mut bytes, BIGRAM_DEPTH_Q32);
        push_u128(&mut bytes, self.bigram_depth_square);
        push_u32(&mut bytes, 0);
        debug_assert_eq!(bytes.len(), OPERATOR_HEADER_LEN);
        for row in &self.normalization_rows {
            push_u32(&mut bytes, row.total);
            push_u32(&mut bytes, usize_u32(row.cells.len()));
            for cell in &row.cells {
                push_u32(&mut bytes, cell.count);
                push_u64(&mut bytes, cell.q32);
                push_u128(&mut bytes, cell.square);
            }
        }
        bytes
    }

    pub fn from_bytes(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        bytes: &[u8],
    ) -> Result<Self, HigherScopeGeometricAttentionError> {
        if bytes.len() < OPERATOR_HEADER_LEN || bytes[..8] != OPERATOR_MAGIC {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "higher-scope operator magic/header is invalid".to_owned(),
            ));
        }
        let mut cursor = Cursor::new(bytes);
        cursor.take(8)?;
        if cursor.u32()? != OPERATOR_VERSION {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "higher-scope operator version is unsupported".to_owned(),
            ));
        }
        let table_artifact_hash = cursor.array_32()?;
        let base_overlay_artifact_hash = cursor.array_32()?;
        let sentence_boundary_token = cursor.u32()?;
        let max_prior_prefix_units = cursor.u32()?;
        let max_tied_candidates = cursor.u32()?;
        let row_count = cursor.count("normalization row", MAX_PRIOR_PREFIX_UNITS)?;
        let trigram_depth_q32 = cursor.u64()?;
        let trigram_depth_square = cursor.u128()?;
        let bigram_depth_q32 = cursor.u64()?;
        let bigram_depth_square = cursor.u128()?;
        if cursor.u32()? != 0 {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "higher-scope operator reserved field is nonzero".to_owned(),
            ));
        }
        if trigram_depth_q32 != TRIGRAM_DEPTH_Q32 || bigram_depth_q32 != BIGRAM_DEPTH_Q32 {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "higher-scope operator depth coordinates drifted".to_owned(),
            ));
        }
        let mut normalization_rows = Vec::with_capacity(row_count);
        for expected_total in 1..=row_count {
            let total = cursor.u32()?;
            if usize::try_from(total).ok() != Some(expected_total) {
                return Err(HigherScopeGeometricAttentionError::Invalid(
                    "normalization totals are not canonical".to_owned(),
                ));
            }
            let maximum_cells = expected_total
                .checked_add(1)
                .ok_or(HigherScopeGeometricAttentionError::ArithmeticOverflow)?;
            let cell_count = cursor.count("normalization cell", maximum_cells)?;
            if cell_count != maximum_cells {
                return Err(HigherScopeGeometricAttentionError::Invalid(
                    "normalization row is incomplete".to_owned(),
                ));
            }
            let mut cells = Vec::with_capacity(cell_count);
            for expected_count in 0..cell_count {
                let count = cursor.u32()?;
                if usize::try_from(count).ok() != Some(expected_count) {
                    return Err(HigherScopeGeometricAttentionError::Invalid(
                        "normalization counts are not canonical".to_owned(),
                    ));
                }
                cells.push(NormalizationCell {
                    count,
                    q32: cursor.u64()?,
                    square: cursor.u128()?,
                });
            }
            normalization_rows.push(NormalizationRow { total, cells });
        }
        if !cursor.is_finished() {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "higher-scope operator has trailing bytes".to_owned(),
            ));
        }
        let operator = Self {
            table_artifact_hash,
            base_overlay_artifact_hash,
            sentence_boundary_token,
            max_prior_prefix_units,
            max_tied_candidates,
            trigram_depth_square,
            bigram_depth_square,
            normalization_rows,
        };
        let expected = Self::compile(table, base_overlay)?;
        if operator != expected || operator.to_bytes() != bytes {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "higher-scope operator is non-canonical or does not reproduce".to_owned(),
            ));
        }
        Ok(operator)
    }

    pub fn predict_matched(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        context: &[u32],
    ) -> Result<MatchedPriorSentenceCountRadiusPrediction, HigherScopeGeometricAttentionError> {
        self.ensure_bound(table, base_overlay)?;
        let local = table.predict_multiscale_count_radius(context, base_overlay)?;
        if local.baseline_support_tokens != local.geometric_support_tokens {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "#953 support differs across matched arms".to_owned(),
            ));
        }
        if local.baseline_work != local.geometric_work {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "#953 declared work differs across matched arms".to_owned(),
            ));
        }
        if local.max_count_tie_tokens.len() > MAX_PRIOR_TIED_CANDIDATES {
            return Err(HigherScopeGeometricAttentionError::Invalid(format!(
                "max-count tie exceeds the {}-candidate operator bound",
                MAX_PRIOR_TIED_CANDIDATES
            )));
        }
        let boundary_index = context
            .iter()
            .rposition(|token| *token == self.sentence_boundary_token);
        let prefix_start = usize::from(context.first() == Some(&BOS_TOKEN));
        let prior_prefix = match boundary_index {
            Some(index) if index >= prefix_start => &context[prefix_start..index],
            _ => &[],
        };
        if prior_prefix.len() > MAX_PRIOR_PREFIX_UNITS {
            return Err(HigherScopeGeometricAttentionError::Invalid(format!(
                "prior prefix exceeds the {}-unit operator bound",
                MAX_PRIOR_PREFIX_UNITS
            )));
        }
        if !local.geometry_reachable || local.max_count_tie_tokens.len() < 2 {
            return Ok(abstaining_prediction(
                local,
                self.sentence_boundary_token,
                boundary_index,
                0,
                0,
                zero_higher_scope_work(),
                PriorSentenceCountRadiusAbstention::LocalGeometryIneligible,
            ));
        }
        if local.tie_evidence.len() != local.max_count_tie_tokens.len()
            || local
                .tie_evidence
                .iter()
                .map(|candidate| candidate.token)
                .ne(local.max_count_tie_tokens.iter().copied())
        {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "#953 tie evidence does not match canonical max-count support".to_owned(),
            ));
        }
        let mut prior_counts = vec![0_u32; local.max_count_tie_tokens.len()];
        let mut membership_checks = 0_u64;
        for &observed in prior_prefix {
            for (index, &candidate) in local.max_count_tie_tokens.iter().enumerate() {
                membership_checks = checked_increment(membership_checks)?;
                if observed == candidate {
                    prior_counts[index] = prior_counts[index]
                        .checked_add(1)
                        .ok_or(HigherScopeGeometricAttentionError::ArithmeticOverflow)?;
                }
            }
        }
        let prior_total = prior_counts.iter().try_fold(0_u32, |total, count| {
            total
                .checked_add(*count)
                .ok_or(HigherScopeGeometricAttentionError::ArithmeticOverflow)
        })?;
        if usize::try_from(prior_total)
            .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?
            > prior_prefix.len()
        {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "candidate occupancy exceeds the prior prefix".to_owned(),
            ));
        }

        let work = PriorSentenceCountRadiusWork {
            local: local.geometric_work,
            prior_prefix_units_scanned: u64::try_from(prior_prefix.len())
                .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?,
            candidate_membership_checks: membership_checks,
            normalization_table_reads: if prior_total == 0 {
                0
            } else {
                u64::try_from(local.max_count_tie_tokens.len())
                    .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?
            },
            square_table_reads: if prior_total == 0 {
                0
            } else {
                u64::try_from(local.max_count_tie_tokens.len())
                    .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?
            },
            coordinate_replacements: if prior_total == 0 {
                0
            } else {
                u64::try_from(local.max_count_tie_tokens.len())
                    .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?
            },
            radius_comparisons: if prior_total == 0 {
                0
            } else {
                u64::try_from(local.max_count_tie_tokens.len())
                    .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?
            },
            final_choice_operations: 1,
        };

        if boundary_index.is_none() {
            return Ok(abstaining_prediction(
                local,
                self.sentence_boundary_token,
                None,
                prior_prefix.len(),
                prior_total,
                work,
                PriorSentenceCountRadiusAbstention::MissingSentenceBoundary,
            ));
        }
        if prior_total == 0 {
            return Ok(abstaining_prediction(
                local,
                self.sentence_boundary_token,
                boundary_index,
                prior_prefix.len(),
                prior_total,
                work,
                PriorSentenceCountRadiusAbstention::NoPriorCandidateOccurrence,
            ));
        }

        let real_scores = self.score_arm(
            &local.tie_evidence,
            &prior_counts,
            prior_total,
            local.order,
            PriorSentenceCountRadiusArm::Real,
        )?;
        let disabled_scores = self.score_arm(
            &local.tie_evidence,
            &prior_counts,
            prior_total,
            local.order,
            PriorSentenceCountRadiusArm::ScopeDisabled,
        )?;
        let permuted_scores = self.score_arm(
            &local.tie_evidence,
            &prior_counts,
            prior_total,
            local.order,
            PriorSentenceCountRadiusArm::CandidatePermuted,
        )?;
        let real_winner = unique_radius_winner(&real_scores);
        let disabled_winner = unique_radius_winner(&disabled_scores);
        let permuted_winner = unique_radius_winner(&permuted_scores);
        let fallback = local.geometric_token;

        let mut candidate_evidence = Vec::with_capacity(local.tie_evidence.len());
        for index in 0..local.tie_evidence.len() {
            let local_candidate = &local.tie_evidence[index];
            let real = &real_scores[index];
            let disabled = &disabled_scores[index];
            let permuted = &permuted_scores[index];
            if local_candidate.token != real.token
                || real.token != disabled.token
                || disabled.token != permuted.token
            {
                return Err(HigherScopeGeometricAttentionError::Invalid(
                    "matched arm candidate order drifted".to_owned(),
                ));
            }
            candidate_evidence.push(PriorSentenceCountRadiusCandidateEvidence {
                token: local_candidate.token,
                count: local_candidate.count,
                local_coordinates: local_candidate.coordinates,
                local_radius: local_candidate.radius,
                prior_count: prior_counts[index],
                real_prior_q32: real.prior_q32,
                real_radius: real.radius,
                disabled_prior_q32: disabled.prior_q32,
                disabled_radius: disabled.radius,
                permuted_prior_count: permuted.prior_count,
                permuted_prior_q32: permuted.prior_q32,
                permuted_radius: permuted.radius,
            });
        }

        let support = local.max_count_tie_tokens.clone();
        let real = decision(
            PriorSentenceCountRadiusArm::Real,
            real_winner.unwrap_or(fallback),
            real_winner,
            support.clone(),
            work,
        );
        let scope_disabled = decision(
            PriorSentenceCountRadiusArm::ScopeDisabled,
            fallback,
            disabled_winner,
            support.clone(),
            work,
        );
        let candidate_permuted = decision(
            PriorSentenceCountRadiusArm::CandidatePermuted,
            permuted_winner.unwrap_or(fallback),
            permuted_winner,
            support.clone(),
            work,
        );
        let support_matched = real.support_tokens == scope_disabled.support_tokens
            && real.support_tokens == candidate_permuted.support_tokens;
        let work_matched = real.work == scope_disabled.work
            && real.work == candidate_permuted.work
            && local.baseline_work == local.geometric_work;

        Ok(MatchedPriorSentenceCountRadiusPrediction {
            local,
            sentence_boundary_token: self.sentence_boundary_token,
            sentence_boundary_index: boundary_index,
            prior_prefix_units: prior_prefix.len(),
            prior_candidate_occurrences: prior_total,
            candidate_evidence,
            real,
            scope_disabled,
            candidate_permuted,
            operator_abstention: real_winner
                .is_none()
                .then_some(PriorSentenceCountRadiusAbstention::RadiusTie),
            support_matched,
            work_matched,
            teacher_calls: 0,
            provider_calls: 0,
            source_weight_reads: 0,
            future_unit_reads: 0,
        })
    }

    pub fn continue_matched(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        seed: &[u8],
        max_units: usize,
    ) -> Result<MatchedPriorSentenceCountRadiusContinuation, HigherScopeGeometricAttentionError>
    {
        if max_units == 0 || max_units > MAX_CONTINUATION_UNITS {
            return Err(HigherScopeGeometricAttentionError::Invalid(format!(
                "continuation bound must be 1..={MAX_CONTINUATION_UNITS}"
            )));
        }
        let mut initial_context = vec![BOS_TOKEN];
        initial_context.extend(table.encode_text(seed)?);
        let first_decision = self.predict_matched(table, base_overlay, &initial_context)?;
        let mut real = ContinuationState::new(initial_context.clone());
        let mut disabled = ContinuationState::new(initial_context.clone());
        let mut permuted = ContinuationState::new(initial_context);
        real.accept(first_decision.real.token);
        disabled.accept(first_decision.scope_disabled.token);
        permuted.accept(first_decision.candidate_permuted.token);

        while real.can_step(max_units)
            || disabled.can_step(max_units)
            || permuted.can_step(max_units)
        {
            if real.can_step(max_units) {
                let prediction =
                    table.predict_multiscale_count_radius(&real.context, base_overlay)?;
                real.accept(prediction.geometric_token);
            }
            if disabled.can_step(max_units) {
                let prediction =
                    table.predict_multiscale_count_radius(&disabled.context, base_overlay)?;
                disabled.accept(prediction.geometric_token);
            }
            if permuted.can_step(max_units) {
                let prediction =
                    table.predict_multiscale_count_radius(&permuted.context, base_overlay)?;
                permuted.accept(prediction.geometric_token);
            }
        }

        Ok(MatchedPriorSentenceCountRadiusContinuation {
            first_decision,
            real: real.finish(table)?,
            scope_disabled: disabled.finish(table)?,
            candidate_permuted: permuted.finish(table)?,
        })
    }

    fn ensure_bound(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
    ) -> Result<(), HigherScopeGeometricAttentionError> {
        if self.table_artifact_hash != cid_digest(&table.artifact_cid())?
            || self.base_overlay_artifact_hash != cid_digest(&base_overlay.artifact_cid())?
            || base_overlay.table_artifact_cid() != table.artifact_cid()
        {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "higher-scope operator artifact binding mismatches".to_owned(),
            ));
        }
        let boundary = table.encode_text(b".")?;
        if boundary.as_slice() != [self.sentence_boundary_token]
            || !table.is_fitted_lexical_token(self.sentence_boundary_token)
        {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "higher-scope operator sentence-boundary binding mismatches".to_owned(),
            ));
        }
        Ok(())
    }

    fn score_arm(
        &self,
        local_candidates: &[MultiscaleCountRadiusCandidate],
        prior_counts: &[u32],
        prior_total: u32,
        order: BackoffOrder,
        arm: PriorSentenceCountRadiusArm,
    ) -> Result<Vec<ScoredCandidate>, HigherScopeGeometricAttentionError> {
        let mut scored = Vec::with_capacity(local_candidates.len());
        for (index, candidate) in local_candidates.iter().enumerate() {
            let source_index = match arm {
                PriorSentenceCountRadiusArm::Real | PriorSentenceCountRadiusArm::ScopeDisabled => {
                    index
                }
                PriorSentenceCountRadiusArm::CandidatePermuted => {
                    let next = index
                        .checked_add(1)
                        .ok_or(HigherScopeGeometricAttentionError::ArithmeticOverflow)?;
                    if next == prior_counts.len() {
                        0
                    } else {
                        next
                    }
                }
            };
            let prior_count = prior_counts[source_index];
            let cell = if prior_total == 0 {
                NormalizationCell {
                    count: 0,
                    q32: 0,
                    square: 0,
                }
            } else {
                self.normalization_cell(prior_count, prior_total)?
            };
            let depth_square = match (order, candidate.coordinates.depth_q32) {
                (BackoffOrder::Trigram, TRIGRAM_DEPTH_Q32) => self.trigram_depth_square,
                (BackoffOrder::Bigram, BIGRAM_DEPTH_Q32) => self.bigram_depth_square,
                _ => {
                    return Err(HigherScopeGeometricAttentionError::Invalid(
                        "#953 candidate depth coordinate is not typed".to_owned(),
                    ))
                }
            };
            let base_radius = candidate.radius.checked_sub(depth_square).ok_or_else(|| {
                HigherScopeGeometricAttentionError::Invalid(
                    "#953 radius is smaller than its depth contribution".to_owned(),
                )
            })?;
            let (prior_q32, contribution) = match arm {
                PriorSentenceCountRadiusArm::ScopeDisabled => (0, 0),
                PriorSentenceCountRadiusArm::Real
                | PriorSentenceCountRadiusArm::CandidatePermuted => (cell.q32, cell.square),
            };
            let radius = base_radius
                .checked_add(contribution)
                .ok_or(HigherScopeGeometricAttentionError::ArithmeticOverflow)?;
            scored.push(ScoredCandidate {
                token: candidate.token,
                prior_count,
                prior_q32,
                radius,
            });
        }
        Ok(scored)
    }

    fn normalization_cell(
        &self,
        count: u32,
        total: u32,
    ) -> Result<NormalizationCell, HigherScopeGeometricAttentionError> {
        if total == 0 || count > total || total > self.max_prior_prefix_units {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "prior-prefix normalization coordinates are out of bounds".to_owned(),
            ));
        }
        let row_index = usize::try_from(total - 1)
            .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?;
        let cell_index = usize::try_from(count)
            .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?;
        let row = self.normalization_rows.get(row_index).ok_or_else(|| {
            HigherScopeGeometricAttentionError::Invalid(
                "prior-prefix normalization row is absent".to_owned(),
            )
        })?;
        let cell = row.cells.get(cell_index).copied().ok_or_else(|| {
            HigherScopeGeometricAttentionError::Invalid(
                "prior-prefix normalization cell is absent".to_owned(),
            )
        })?;
        if row.total != total || cell.count != count {
            return Err(HigherScopeGeometricAttentionError::Invalid(
                "prior-prefix normalization lookup identity drifted".to_owned(),
            ));
        }
        Ok(cell)
    }
}

#[derive(Debug, Clone, Copy)]
struct ScoredCandidate {
    token: u32,
    prior_count: u32,
    prior_q32: u64,
    radius: u128,
}

fn unique_radius_winner(candidates: &[ScoredCandidate]) -> Option<u32> {
    let mut winner = None;
    let mut maximum = 0_u128;
    let mut tied = false;
    for candidate in candidates {
        if winner.is_none() || candidate.radius > maximum {
            winner = Some(candidate.token);
            maximum = candidate.radius;
            tied = false;
        } else if candidate.radius == maximum {
            tied = true;
        }
    }
    if tied {
        None
    } else {
        winner
    }
}

fn decision(
    arm: PriorSentenceCountRadiusArm,
    token: u32,
    unique_radius_winner: Option<u32>,
    support_tokens: Vec<u32>,
    work: PriorSentenceCountRadiusWork,
) -> PriorSentenceCountRadiusDecision {
    PriorSentenceCountRadiusDecision {
        arm,
        token,
        unique_radius_winner,
        support_tokens,
        work,
    }
}

fn zero_higher_scope_work() -> PriorSentenceCountRadiusWork {
    PriorSentenceCountRadiusWork {
        local: MultiscaleCountRadiusWork::default(),
        prior_prefix_units_scanned: 0,
        candidate_membership_checks: 0,
        normalization_table_reads: 0,
        square_table_reads: 0,
        coordinate_replacements: 0,
        radius_comparisons: 0,
        final_choice_operations: 1,
    }
}

fn abstaining_prediction(
    local: MatchedGeometricPrediction,
    sentence_boundary_token: u32,
    sentence_boundary_index: Option<usize>,
    prior_prefix_units: usize,
    prior_candidate_occurrences: u32,
    mut work: PriorSentenceCountRadiusWork,
    reason: PriorSentenceCountRadiusAbstention,
) -> MatchedPriorSentenceCountRadiusPrediction {
    work.local = local.geometric_work;
    let support = local.max_count_tie_tokens.clone();
    let fallback = local.geometric_token;
    MatchedPriorSentenceCountRadiusPrediction {
        local,
        sentence_boundary_token,
        sentence_boundary_index,
        prior_prefix_units,
        prior_candidate_occurrences,
        candidate_evidence: Vec::new(),
        real: decision(
            PriorSentenceCountRadiusArm::Real,
            fallback,
            None,
            support.clone(),
            work,
        ),
        scope_disabled: decision(
            PriorSentenceCountRadiusArm::ScopeDisabled,
            fallback,
            None,
            support.clone(),
            work,
        ),
        candidate_permuted: decision(
            PriorSentenceCountRadiusArm::CandidatePermuted,
            fallback,
            None,
            support,
            work,
        ),
        operator_abstention: Some(reason),
        support_matched: true,
        work_matched: true,
        teacher_calls: 0,
        provider_calls: 0,
        source_weight_reads: 0,
        future_unit_reads: 0,
    }
}

#[derive(Debug, Clone)]
struct ContinuationState {
    context: Vec<u32>,
    generated: Vec<u32>,
    stop: ContinuationStop,
}

impl ContinuationState {
    fn new(context: Vec<u32>) -> Self {
        Self {
            context,
            generated: Vec::new(),
            stop: ContinuationStop::Bound,
        }
    }

    fn can_step(&self, max_units: usize) -> bool {
        self.stop == ContinuationStop::Bound && self.generated.len() < max_units
    }

    fn accept(&mut self, token: u32) {
        if token == EOS_TOKEN {
            self.stop = ContinuationStop::EndOfDocument;
            return;
        }
        if self.generated.last() == Some(&token) {
            self.stop = ContinuationStop::PeriodOneCycle;
            return;
        }
        if self.generated.len() >= 3
            && self.generated[self.generated.len() - 2] == token
            && self.generated[self.generated.len() - 3] == self.generated[self.generated.len() - 1]
        {
            self.stop = ContinuationStop::PeriodTwoCycle;
            return;
        }
        self.generated.push(token);
        self.context.push(token);
    }

    fn finish(
        self,
        table: &SourceFreeTable,
    ) -> Result<Continuation, HigherScopeGeometricAttentionError> {
        Ok(Continuation {
            decoded: table.decode_tokens(&self.generated)?,
            tokens: self.generated,
            stop: self.stop,
        })
    }
}

fn compile_square(value: u64) -> Result<u128, HigherScopeGeometricAttentionError> {
    u128::from(value)
        .checked_mul(u128::from(value))
        .ok_or(HigherScopeGeometricAttentionError::ArithmeticOverflow)
}

fn checked_increment(value: u64) -> Result<u64, HigherScopeGeometricAttentionError> {
    value
        .checked_add(1)
        .ok_or(HigherScopeGeometricAttentionError::ArithmeticOverflow)
}

fn cid_digest(cid: &str) -> Result<[u8; 32], HigherScopeGeometricAttentionError> {
    let hex_digest = cid.strip_prefix("blake3:").ok_or_else(|| {
        HigherScopeGeometricAttentionError::Invalid("artifact CID is not BLAKE3".to_owned())
    })?;
    let bytes = hex::decode(hex_digest).map_err(|_| {
        HigherScopeGeometricAttentionError::Invalid("artifact CID hex is invalid".to_owned())
    })?;
    bytes.try_into().map_err(|_| {
        HigherScopeGeometricAttentionError::Invalid("artifact CID length is invalid".to_owned())
    })
}

fn usize_u32(value: usize) -> u32 {
    u32::try_from(value).expect("validated higher-scope artifact length exceeds u32")
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u128(bytes: &mut Vec<u8>, value: u128) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], HigherScopeGeometricAttentionError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(HigherScopeGeometricAttentionError::ArithmeticOverflow)?;
        let value = self.bytes.get(self.offset..end).ok_or_else(|| {
            HigherScopeGeometricAttentionError::Invalid(
                "higher-scope operator is truncated".to_owned(),
            )
        })?;
        self.offset = end;
        Ok(value)
    }

    fn u32(&mut self) -> Result<u32, HigherScopeGeometricAttentionError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().map_err(
            |_| HigherScopeGeometricAttentionError::ArithmeticOverflow,
        )?))
    }

    fn u64(&mut self) -> Result<u64, HigherScopeGeometricAttentionError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().map_err(
            |_| HigherScopeGeometricAttentionError::ArithmeticOverflow,
        )?))
    }

    fn u128(&mut self) -> Result<u128, HigherScopeGeometricAttentionError> {
        Ok(u128::from_le_bytes(self.take(16)?.try_into().map_err(
            |_| HigherScopeGeometricAttentionError::ArithmeticOverflow,
        )?))
    }

    fn array_32(&mut self) -> Result<[u8; 32], HigherScopeGeometricAttentionError> {
        self.take(32)?
            .try_into()
            .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)
    }

    fn count(
        &mut self,
        label: &str,
        maximum: usize,
    ) -> Result<usize, HigherScopeGeometricAttentionError> {
        let count = usize::try_from(self.u32()?)
            .map_err(|_| HigherScopeGeometricAttentionError::ArithmeticOverflow)?;
        if count > maximum {
            return Err(HigherScopeGeometricAttentionError::Invalid(format!(
                "{label} count exceeds its bound"
            )));
        }
        Ok(count)
    }

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
