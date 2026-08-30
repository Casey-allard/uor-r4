//! Bounded recurrent state compiled from the qualified R4 softmax trace.
//!
//! The compiler in this module may use ordinary host floating point.  The
//! resulting runtime is nevertheless source-free: one transition consumes only
//! the prior 278-byte state, an observed token, a canonical H4 frame address,
//! and the frozen artifact.  It has no source-model, teacher-trace, target, or
//! future-token input.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::helm_d_r4_attention::{
    canonical_registered_h4_spin_frames, intrinsic_stable_softmax_into, R4RegisteredSpinFrame,
};
use crate::r4_softmax_trace_student::{
    R4SoftmaxTraceStudentArm, R4SoftmaxTraceStudentArtifact, R4SoftmaxTraceStudentError,
    TeacherTopDistributionQ16, R4_SOFTMAX_TRACE_Q16_TOTAL,
};

const ARTIFACT_MAGIC: [u8; 8] = *b"R4SSS001";
const ARTIFACT_VERSION: u32 = 1;
const ARTIFACT_HEADER_LEN: usize = 84;
const H4_FRAME_COUNT: usize = 120;
const RIDGE_SWEEPS: usize = 64;
const READOUT_STEPS: usize = 512;
const RIDGE_LAMBDA: f32 = 1.0 / 1024.0;
const READOUT_RATE: f32 = 1.0 / 32.0;

pub const R4_SOFTMAX_TRACE_STATE_STUDENT_SCHEMA: &str = "R4SoftmaxTraceStateStudentV1";
pub const R4_SOFTMAX_TRACE_STATE_BANKS: usize = 4;
pub const R4_SOFTMAX_TRACE_STATE_WIDTH: usize = 4;
pub const R4_SOFTMAX_TRACE_STATE_FLOATS: usize = 64;
pub const R4_SOFTMAX_TRACE_STATE_BYTES: usize = 278;
pub const R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM: usize = 120;
pub const R4_SOFTMAX_TRACE_STATE_FITTED_VALUES_PER_ARM: usize = 112;
pub const R4_SOFTMAX_TRACE_STATE_PARAMETER_BYTES_PER_ARM: usize = 480;
pub const R4_SOFTMAX_TRACE_STATE_READOUT_FEATURES: usize = 64;
pub const R4_SOFTMAX_TRACE_STATE_RHO: [f32; 4] = [0.10, 0.55, 0.90, 0.985];
pub const R4_SOFTMAX_TRACE_STATE_ETA: [f32; 4] = [1.00, 0.55, 0.20, 0.060];

type Vector4 = [f32; R4_SOFTMAX_TRACE_STATE_WIDTH];
type Matrix4 = [[f32; R4_SOFTMAX_TRACE_STATE_WIDTH]; R4_SOFTMAX_TRACE_STATE_WIDTH];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum R4SoftmaxTraceStateStudentError {
    Invalid(String),
    ArithmeticOverflow,
}

impl std::fmt::Display for R4SoftmaxTraceStateStudentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::ArithmeticOverflow => {
                formatter.write_str("R4 softmax trace state student arithmetic overflow")
            }
        }
    }
}

impl std::error::Error for R4SoftmaxTraceStateStudentError {}

impl From<R4SoftmaxTraceStudentError> for R4SoftmaxTraceStateStudentError {
    fn from(error: R4SoftmaxTraceStudentError) -> Self {
        Self::Invalid(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum R4SoftmaxTraceStateArm {
    Suffix,
    PlainRecurrent,
    GeometricRecurrent,
    TransportPermutedControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R4SoftmaxTraceReductionRole {
    Query,
    Key,
    Value,
}

impl R4SoftmaxTraceReductionRole {
    const fn tag(self) -> u8 {
        match self {
            Self::Query => 0,
            Self::Key => 1,
            Self::Value => 2,
        }
    }
}

/// Fold exactly nine heads of sixteen R4 blocks into one signed R4 feature.
///
/// Signs are fixed by BLAKE3 over `(role, head, block)` and the declared
/// scale is exactly `1/sqrt(144) = 1/12`.  The input iteration order is part of
/// the representation contract. One scalar sign applies to every lane of a
/// block, so the reduction commutes with any linear R4 frame action.
pub fn signed_reduce_final_layer_r4(
    role: R4SoftmaxTraceReductionRole,
    blocks: &[[f32; 4]],
) -> Result<[f32; 4], R4SoftmaxTraceStateStudentError> {
    if blocks.len() != 9 * 16 || blocks.iter().flatten().any(|value| !value.is_finite()) {
        return Err(R4SoftmaxTraceStateStudentError::Invalid(
            "state-student reduction requires 9 heads x 16 finite R4 blocks".to_owned(),
        ));
    }
    let mut output = [0.0_f32; 4];
    for (flat_block, block) in blocks.iter().enumerate() {
        let head = flat_block / 16;
        let within_head = flat_block % 16;
        let address = [
            role.tag(),
            u8::try_from(head).map_err(|_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?,
            u8::try_from(within_head)
                .map_err(|_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?,
        ];
        let sign = if blake3::hash(&address).as_bytes()[0] & 1 == 0 {
            1.0
        } else {
            -1.0
        };
        for lane in 0..4 {
            output[lane] += sign * block[lane] / 12.0;
        }
    }
    require_finite_vector(output, "signed trace reduction")?;
    Ok(output)
}

/// A stable, table-free token representation available at compile and runtime.
pub fn deterministic_token_r4(token: u32) -> [f32; 4] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4-state-token/");
    hasher.update(&token.to_le_bytes());
    let digest = hasher.finalize();
    let mut vector = [0.0_f32; 4];
    for (lane, value) in vector.iter_mut().enumerate() {
        let start = lane * 4;
        let raw = i32::from_le_bytes(
            digest.as_bytes()[start..start + 4]
                .try_into()
                .expect("four-byte BLAKE3 lane"),
        );
        *value = raw as f32 / i32::MAX as f32;
    }
    normalize_or_axis(vector)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct R4SoftmaxTraceStateFitEvent {
    pub position: u32,
    pub observed_token: u32,
    pub actual_next_token: u32,
    pub frame_table_offset: u16,
    pub query_trace_r4: [f32; 4],
    pub key_trace_r4: [f32; 4],
    pub value_trace_r4: [f32; 4],
    pub teacher_top_distribution: TeacherTopDistributionQ16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct R4SoftmaxTraceStateFitSequence {
    pub document_id: String,
    pub events: Vec<R4SoftmaxTraceStateFitEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct R4SoftmaxTraceStateFitConfig {
    pub maximum_token_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct R4SoftmaxTraceStateParameters {
    key_map: Matrix4,
    value_map: Matrix4,
    query_map: Matrix4,
    readout_maps: [Matrix4; R4_SOFTMAX_TRACE_STATE_BANKS],
    rho: [f32; R4_SOFTMAX_TRACE_STATE_BANKS],
    eta: [f32; R4_SOFTMAX_TRACE_STATE_BANKS],
}

impl R4SoftmaxTraceStateParameters {
    pub fn key_map(&self) -> Matrix4 {
        self.key_map
    }

    pub fn value_map(&self) -> Matrix4 {
        self.value_map
    }

    pub fn query_map(&self) -> Matrix4 {
        self.query_map
    }

    pub fn readout_maps(&self) -> [Matrix4; R4_SOFTMAX_TRACE_STATE_BANKS] {
        self.readout_maps
    }

    pub fn rho(&self) -> [f32; R4_SOFTMAX_TRACE_STATE_BANKS] {
        self.rho
    }

    pub fn eta(&self) -> [f32; R4_SOFTMAX_TRACE_STATE_BANKS] {
        self.eta
    }

    pub fn parameter_value_count(&self) -> usize {
        R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM
    }

    fn new(key_map: Matrix4, value_map: Matrix4, query_map: Matrix4) -> Self {
        Self {
            key_map,
            value_map,
            query_map,
            readout_maps: [[[0.0; 4]; 4]; 4],
            rho: R4_SOFTMAX_TRACE_STATE_RHO,
            eta: R4_SOFTMAX_TRACE_STATE_ETA,
        }
    }

    fn values(&self) -> [f32; R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM] {
        let mut values = [0.0; R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM];
        let mut offset = 0;
        for matrix in [&self.key_map, &self.value_map, &self.query_map] {
            for row in matrix {
                for &value in row {
                    values[offset] = value;
                    offset += 1;
                }
            }
        }
        for matrix in &self.readout_maps {
            for row in matrix {
                for &value in row {
                    values[offset] = value;
                    offset += 1;
                }
            }
        }
        values[offset..offset + 4].copy_from_slice(&self.rho);
        offset += 4;
        values[offset..offset + 4].copy_from_slice(&self.eta);
        debug_assert_eq!(offset + 4, values.len());
        values
    }

    fn from_values(
        values: [f32; R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM],
    ) -> Result<Self, R4SoftmaxTraceStateStudentError> {
        if values.iter().any(|value| !value.is_finite()) {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student parameters contain a non-finite value".to_owned(),
            ));
        }
        let mut offset = 0;
        let mut next_matrix = || {
            let mut matrix = [[0.0; 4]; 4];
            for row in &mut matrix {
                for value in row {
                    *value = values[offset];
                    offset += 1;
                }
            }
            matrix
        };
        let key_map = next_matrix();
        let value_map = next_matrix();
        let query_map = next_matrix();
        let mut readout_maps = [[[0.0; 4]; 4]; 4];
        for matrix in &mut readout_maps {
            *matrix = next_matrix();
        }
        let mut rho = [0.0; 4];
        rho.copy_from_slice(&values[offset..offset + 4]);
        offset += 4;
        let mut eta = [0.0; 4];
        eta.copy_from_slice(&values[offset..offset + 4]);
        offset += 4;
        debug_assert_eq!(offset, values.len());
        let parameters = Self {
            key_map,
            value_map,
            query_map,
            readout_maps,
            rho,
            eta,
        };
        parameters.validate()?;
        Ok(parameters)
    }

    fn validate(&self) -> Result<(), R4SoftmaxTraceStateStudentError> {
        let values = self.values();
        if values.iter().any(|value| !value.is_finite()) {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student parameters contain a non-finite value".to_owned(),
            ));
        }
        if self.rho.map(f32::to_bits) != R4_SOFTMAX_TRACE_STATE_RHO.map(f32::to_bits)
            || self.eta.map(f32::to_bits) != R4_SOFTMAX_TRACE_STATE_ETA.map(f32::to_bits)
        {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student retention/write policies differ from the frozen basis".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct R4SoftmaxTraceStateStudentArtifact {
    maximum_token_id: u32,
    construction_document_count: u32,
    construction_position_count: u64,
    construction_digest: [u8; 32],
    geometric: R4SoftmaxTraceStateParameters,
    plain: R4SoftmaxTraceStateParameters,
    suffix: R4SoftmaxTraceStudentArtifact,
}

impl R4SoftmaxTraceStateStudentArtifact {
    pub fn maximum_token_id(&self) -> u32 {
        self.maximum_token_id
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

    pub fn suffix_artifact(&self) -> &R4SoftmaxTraceStudentArtifact {
        &self.suffix
    }

    pub fn geometric_parameters(&self) -> &R4SoftmaxTraceStateParameters {
        &self.geometric
    }

    pub fn plain_parameters(&self) -> &R4SoftmaxTraceStateParameters {
        &self.plain
    }

    pub fn fitted_parameter_values_per_arm(&self) -> usize {
        R4_SOFTMAX_TRACE_STATE_FITTED_VALUES_PER_ARM
    }

    pub fn parameter_values_per_arm(&self) -> usize {
        R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM
    }

    pub fn payload_bytes_before_headers(&self) -> usize {
        self.suffix.to_bytes().len() + 2 * R4_SOFTMAX_TRACE_STATE_PARAMETER_BYTES_PER_ARM
    }

    pub fn artifact_cid(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.to_bytes()).to_hex())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let suffix = self.suffix.to_bytes();
        let mut bytes = Vec::with_capacity(
            ARTIFACT_HEADER_LEN + 2 * R4_SOFTMAX_TRACE_STATE_PARAMETER_BYTES_PER_ARM + suffix.len(),
        );
        bytes.extend_from_slice(&ARTIFACT_MAGIC);
        push_u32(&mut bytes, ARTIFACT_VERSION);
        push_u32(&mut bytes, R4_SOFTMAX_TRACE_STATE_BANKS as u32);
        push_u32(&mut bytes, R4_SOFTMAX_TRACE_STATE_WIDTH as u32);
        push_u32(&mut bytes, R4_SOFTMAX_TRACE_STATE_BYTES as u32);
        push_u32(
            &mut bytes,
            R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM as u32,
        );
        push_u32(&mut bytes, self.maximum_token_id);
        push_u32(&mut bytes, self.construction_document_count);
        push_u64(&mut bytes, self.construction_position_count);
        bytes.extend_from_slice(&self.construction_digest);
        push_u64(
            &mut bytes,
            u64::try_from(suffix.len()).expect("validated suffix artifact length exceeds u64"),
        );
        debug_assert_eq!(bytes.len(), ARTIFACT_HEADER_LEN);
        for parameters in [&self.geometric, &self.plain] {
            for value in parameters.values() {
                push_u32(&mut bytes, value.to_bits());
            }
        }
        bytes.extend_from_slice(&suffix);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, R4SoftmaxTraceStateStudentError> {
        let mut cursor = Cursor::new(bytes);
        if cursor.take(8)? != ARTIFACT_MAGIC {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student artifact magic is invalid".to_owned(),
            ));
        }
        if cursor.u32()? != ARTIFACT_VERSION
            || cursor.u32()? != R4_SOFTMAX_TRACE_STATE_BANKS as u32
            || cursor.u32()? != R4_SOFTMAX_TRACE_STATE_WIDTH as u32
            || cursor.u32()? != R4_SOFTMAX_TRACE_STATE_BYTES as u32
            || cursor.u32()? != R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM as u32
        {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student artifact contract/version is unsupported".to_owned(),
            ));
        }
        let maximum_token_id = cursor.u32()?;
        let construction_document_count = cursor.u32()?;
        let construction_position_count = cursor.u64()?;
        let construction_digest: [u8; 32] = cursor
            .take(32)?
            .try_into()
            .map_err(|_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?;
        let suffix_length = usize::try_from(cursor.u64()?).map_err(|_| {
            R4SoftmaxTraceStateStudentError::Invalid(
                "state-student suffix length exceeds this host".to_owned(),
            )
        })?;
        let geometric = read_parameters(&mut cursor)?;
        let plain = read_parameters(&mut cursor)?;
        let suffix = R4SoftmaxTraceStudentArtifact::from_bytes(cursor.take(suffix_length)?)?;
        if !cursor.is_finished() {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student artifact has trailing bytes".to_owned(),
            ));
        }
        let artifact = Self {
            maximum_token_id,
            construction_document_count,
            construction_position_count,
            construction_digest,
            geometric,
            plain,
            suffix,
        };
        artifact.validate()?;
        if artifact.to_bytes() != bytes {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student artifact is not canonical".to_owned(),
            ));
        }
        Ok(artifact)
    }

    pub fn from_bytes_with_expected_cid(
        bytes: &[u8],
        expected_cid: &str,
    ) -> Result<Self, R4SoftmaxTraceStateStudentError> {
        let observed = format!("blake3:{}", blake3::hash(bytes).to_hex());
        if observed != expected_cid {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(format!(
                "state-student artifact CID mismatch: expected {expected_cid}, observed {observed}"
            )));
        }
        Self::from_bytes(bytes)
    }

    pub fn runtime(
        &self,
        arm: R4SoftmaxTraceStateArm,
    ) -> Result<R4SoftmaxTraceStateRuntime, R4SoftmaxTraceStateStudentError> {
        R4SoftmaxTraceStateRuntime::new(self.clone(), arm)
    }

    fn validate(&self) -> Result<(), R4SoftmaxTraceStateStudentError> {
        if self.construction_document_count == 0 || self.construction_position_count == 0 {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student construction census is empty".to_owned(),
            ));
        }
        if self.construction_document_count != self.suffix.construction_document_count()
            || self.construction_position_count != self.suffix.construction_position_count()
        {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student and embedded suffix construction censuses differ".to_owned(),
            ));
        }
        self.geometric.validate()?;
        self.plain.validate()?;
        Ok(())
    }
}

/// The entire recurrent state. Its canonical representation is exactly 278 bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct R4SoftmaxTraceState {
    banks: [Matrix4; R4_SOFTMAX_TRACE_STATE_BANKS],
    frame_table_offset: u16,
    observation_count: u32,
    token_ring: [u32; 4],
}

impl Default for R4SoftmaxTraceState {
    fn default() -> Self {
        Self {
            banks: [[[0.0; 4]; 4]; 4],
            frame_table_offset: 0,
            observation_count: 0,
            token_ring: [0; 4],
        }
    }
}

impl R4SoftmaxTraceState {
    pub fn banks(&self) -> &[Matrix4; R4_SOFTMAX_TRACE_STATE_BANKS] {
        &self.banks
    }

    pub fn frame_table_offset(&self) -> u16 {
        self.frame_table_offset
    }

    pub fn observation_count(&self) -> u32 {
        self.observation_count
    }

    pub fn token_ring(&self) -> [u32; 4] {
        self.token_ring
    }

    pub fn checksum(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.to_bytes()).to_hex())
    }

    pub fn to_bytes(&self) -> [u8; R4_SOFTMAX_TRACE_STATE_BYTES] {
        let mut bytes = [0_u8; R4_SOFTMAX_TRACE_STATE_BYTES];
        let mut offset = 0;
        for bank in &self.banks {
            for row in bank {
                for value in row {
                    bytes[offset..offset + 4].copy_from_slice(&value.to_bits().to_le_bytes());
                    offset += 4;
                }
            }
        }
        bytes[offset..offset + 2].copy_from_slice(&self.frame_table_offset.to_le_bytes());
        offset += 2;
        bytes[offset..offset + 4].copy_from_slice(&self.observation_count.to_le_bytes());
        offset += 4;
        for token in self.token_ring {
            bytes[offset..offset + 4].copy_from_slice(&token.to_le_bytes());
            offset += 4;
        }
        debug_assert_eq!(offset, R4_SOFTMAX_TRACE_STATE_BYTES);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, R4SoftmaxTraceStateStudentError> {
        if bytes.len() != R4_SOFTMAX_TRACE_STATE_BYTES {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student runtime state must contain exactly 278 bytes".to_owned(),
            ));
        }
        let mut cursor = Cursor::new(bytes);
        let mut banks = [[[0.0; 4]; 4]; 4];
        for bank in &mut banks {
            for row in bank {
                for value in row {
                    *value = f32::from_bits(cursor.u32()?);
                    if !value.is_finite() {
                        return Err(R4SoftmaxTraceStateStudentError::Invalid(
                            "state-student runtime state contains a non-finite value".to_owned(),
                        ));
                    }
                }
            }
        }
        let frame_table_offset = cursor.u16()?;
        let observation_count = cursor.u32()?;
        let mut token_ring = [0; 4];
        for token in &mut token_ring {
            *token = cursor.u32()?;
        }
        let state = Self {
            banks,
            frame_table_offset,
            observation_count,
            token_ring,
        };
        state.validate()?;
        if state.to_bytes().as_slice() != bytes {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student runtime state is not canonical".to_owned(),
            ));
        }
        Ok(state)
    }

    pub fn from_bytes_with_expected_checksum(
        bytes: &[u8],
        expected_checksum: &str,
    ) -> Result<Self, R4SoftmaxTraceStateStudentError> {
        let observed = format!("blake3:{}", blake3::hash(bytes).to_hex());
        if observed != expected_checksum {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student runtime state checksum mismatch".to_owned(),
            ));
        }
        Self::from_bytes(bytes)
    }

    fn history(&self) -> Vec<u32> {
        let count = usize::try_from(self.observation_count)
            .unwrap_or(usize::MAX)
            .min(4);
        self.token_ring[4 - count..].to_vec()
    }

    fn push_token(&mut self, token: u32) {
        self.token_ring.rotate_left(1);
        self.token_ring[3] = token;
    }

    fn validate(&self) -> Result<(), R4SoftmaxTraceStateStudentError> {
        if usize::from(self.frame_table_offset) >= H4_FRAME_COUNT {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student frame offset is outside the H4 registry".to_owned(),
            ));
        }
        let unused = 4_usize.saturating_sub(
            usize::try_from(self.observation_count)
                .unwrap_or(usize::MAX)
                .min(4),
        );
        if self.token_ring[..unused].iter().any(|token| *token != 0) {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student token-ring padding is nonzero".to_owned(),
            ));
        }
        if self.observation_count == 0
            && (self.frame_table_offset != 0
                || self
                    .banks
                    .iter()
                    .flatten()
                    .flatten()
                    .any(|value| value.to_bits() != 0))
        {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "zero-observation state differs from the canonical initial state".to_owned(),
            ));
        }
        Ok(())
    }

    fn validate_for_artifact(
        &self,
        maximum_token_id: u32,
    ) -> Result<(), R4SoftmaxTraceStateStudentError> {
        let populated = usize::try_from(self.observation_count)
            .unwrap_or(usize::MAX)
            .min(self.token_ring.len());
        if self.token_ring[self.token_ring.len() - populated..]
            .iter()
            .any(|token| *token > maximum_token_id)
        {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student replay history is outside the artifact namespace".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct R4SoftmaxTraceStateRuntimeAudit {
    pub prior_state_reads: u64,
    pub observed_token_reads: u64,
    pub canonical_frame_reads: u64,
    pub artifact_reads: u64,
    pub state_transports: u64,
    pub transport_permutations: u64,
    pub source_model_forwards: u64,
    pub source_trace_reads: u64,
    pub teacher_distribution_reads: u64,
    pub target_reads: u64,
    pub future_token_reads: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct R4SoftmaxTraceStateScore {
    pub token: u32,
    pub probability: f32,
    pub logit: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct R4SoftmaxTraceStatePrediction {
    pub token: u32,
    pub probability: f32,
    pub suffix_depth: u8,
    pub candidates: Vec<R4SoftmaxTraceStateScore>,
    pub state_checksum: String,
}

/// One candidate's read-only view of the existing recurrent readout.
///
/// This is compiler-side observability only. It exposes the natural feature
/// tensor and exact logit decomposition already used by the runtime; it does
/// not add a new score, parameter, or runtime input.
#[derive(Debug, Clone, PartialEq)]
pub struct R4SoftmaxTraceStateDiagnosticCandidate {
    pub token: u32,
    pub readout_features: [f32; R4_SOFTMAX_TRACE_STATE_READOUT_FEATURES],
    pub base_logit: f32,
    pub residual_logit: f32,
    pub total_logit: f32,
}

/// Immutable diagnostic snapshot of one already-observed recurrent state.
#[derive(Debug, Clone, PartialEq)]
pub struct R4SoftmaxTraceStateDiagnosticSnapshot {
    pub arm: R4SoftmaxTraceStateArm,
    pub suffix_depth: u8,
    pub state_checksum: String,
    pub candidates: Vec<R4SoftmaxTraceStateDiagnosticCandidate>,
}

#[derive(Debug, Clone)]
pub struct R4SoftmaxTraceStateRuntime {
    artifact: R4SoftmaxTraceStateStudentArtifact,
    arm: R4SoftmaxTraceStateArm,
    state: R4SoftmaxTraceState,
    frames: Vec<R4RegisteredSpinFrame>,
    audit: R4SoftmaxTraceStateRuntimeAudit,
}

impl R4SoftmaxTraceStateRuntime {
    fn new(
        artifact: R4SoftmaxTraceStateStudentArtifact,
        arm: R4SoftmaxTraceStateArm,
    ) -> Result<Self, R4SoftmaxTraceStateStudentError> {
        let frames = registered_frames()?;
        Ok(Self {
            artifact,
            arm,
            state: R4SoftmaxTraceState::default(),
            frames,
            audit: R4SoftmaxTraceStateRuntimeAudit::default(),
        })
    }

    pub fn from_artifact_and_state_bytes(
        artifact_bytes: &[u8],
        expected_artifact_cid: &str,
        state_bytes: &[u8],
        expected_state_checksum: &str,
        arm: R4SoftmaxTraceStateArm,
    ) -> Result<Self, R4SoftmaxTraceStateStudentError> {
        let artifact = R4SoftmaxTraceStateStudentArtifact::from_bytes_with_expected_cid(
            artifact_bytes,
            expected_artifact_cid,
        )?;
        let state = R4SoftmaxTraceState::from_bytes_with_expected_checksum(
            state_bytes,
            expected_state_checksum,
        )?;
        state.validate_for_artifact(artifact.maximum_token_id)?;
        let mut runtime = Self::new(artifact, arm)?;
        runtime.state = state;
        Ok(runtime)
    }

    pub fn arm(&self) -> R4SoftmaxTraceStateArm {
        self.arm
    }

    pub fn state(&self) -> &R4SoftmaxTraceState {
        &self.state
    }

    pub fn audit(&self) -> R4SoftmaxTraceStateRuntimeAudit {
        self.audit
    }

    /// Inspect the current recurrent readout without changing state or
    /// provenance counters.
    ///
    /// The runtime must already have consumed at least one observation. The
    /// candidate support, base logits, 64-value features, residual logits, and
    /// total logits are reconstructed from that immutable current state using
    /// the exact existing mechanism.
    pub fn diagnostic_readout_snapshot(
        &self,
    ) -> Result<R4SoftmaxTraceStateDiagnosticSnapshot, R4SoftmaxTraceStateStudentError> {
        let parameters = match self.arm {
            R4SoftmaxTraceStateArm::PlainRecurrent => &self.artifact.plain,
            R4SoftmaxTraceStateArm::GeometricRecurrent
            | R4SoftmaxTraceStateArm::TransportPermutedControl => &self.artifact.geometric,
            R4SoftmaxTraceStateArm::Suffix => {
                return Err(R4SoftmaxTraceStateStudentError::Invalid(
                    "state-student diagnostic snapshot requires a recurrent arm".to_owned(),
                ));
            }
        };
        if self.state.observation_count == 0 {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student diagnostic snapshot requires an observed state".to_owned(),
            ));
        }
        self.state
            .validate_for_artifact(self.artifact.maximum_token_id)?;
        let current_frame = self.frames[usize::from(self.state.frame_table_offset)];
        let observed_token = self.state.token_ring[3];
        let model_key = matrix_vector(&parameters.key_map, deterministic_token_r4(observed_token));
        let geometric = matches!(
            self.arm,
            R4SoftmaxTraceStateArm::GeometricRecurrent
                | R4SoftmaxTraceStateArm::TransportPermutedControl
        );
        let key = normalize_or_axis(if geometric {
            encode(current_frame, model_key)?
        } else {
            model_key
        });
        let current_frame = geometric.then_some(current_frame);
        let suffix_distribution = self.artifact.suffix.runtime().distribution(
            &self.state.history(),
            R4SoftmaxTraceStudentArm::TeacherDistilled,
        )?;
        let weights = flattened_readout_weights(parameters);
        let mut candidates = Vec::with_capacity(suffix_distribution.scores.len());
        for score in suffix_distribution.scores {
            let base_logit = (f32::from(score.weight_q16) / f32::from(R4_SOFTMAX_TRACE_Q16_TOTAL))
                .max(f32::MIN_POSITIVE)
                .ln();
            let readout_features = readout_features(
                &self.state.banks,
                parameters,
                key,
                score.token,
                current_frame,
            )?;
            let residual_logit = dot64(weights, readout_features);
            let total_logit = base_logit + residual_logit;
            if !total_logit.is_finite() {
                return Err(R4SoftmaxTraceStateStudentError::Invalid(
                    "state-student diagnostic snapshot produced a non-finite logit".to_owned(),
                ));
            }
            candidates.push(R4SoftmaxTraceStateDiagnosticCandidate {
                token: score.token,
                readout_features,
                base_logit,
                residual_logit,
                total_logit,
            });
        }
        Ok(R4SoftmaxTraceStateDiagnosticSnapshot {
            arm: self.arm,
            suffix_depth: suffix_distribution.suffix_depth,
            state_checksum: self.state.checksum(),
            candidates,
        })
    }

    /// Consume one causal observation and predict the following token.
    ///
    /// There is intentionally no teacher/source/target/future argument.
    pub fn observe_and_predict(
        &mut self,
        observed_token: u32,
        canonical_frame_offset: u16,
    ) -> Result<R4SoftmaxTraceStatePrediction, R4SoftmaxTraceStateStudentError> {
        if observed_token > self.artifact.maximum_token_id
            || usize::from(canonical_frame_offset) >= self.frames.len()
        {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student runtime observation is outside the frozen namespace".to_owned(),
            ));
        }
        self.audit.prior_state_reads = increment(self.audit.prior_state_reads)?;
        self.audit.observed_token_reads = increment(self.audit.observed_token_reads)?;
        self.audit.canonical_frame_reads = increment(self.audit.canonical_frame_reads)?;
        self.audit.artifact_reads = increment(self.audit.artifact_reads)?;

        let parameters = match self.arm {
            R4SoftmaxTraceStateArm::PlainRecurrent => &self.artifact.plain,
            R4SoftmaxTraceStateArm::GeometricRecurrent
            | R4SoftmaxTraceStateArm::TransportPermutedControl => &self.artifact.geometric,
            R4SoftmaxTraceStateArm::Suffix => &self.artifact.plain,
        };
        let current_frame = self.frames[usize::from(canonical_frame_offset)];
        let model_key = matrix_vector(&parameters.key_map, deterministic_token_r4(observed_token));
        let key = normalize_or_axis(match self.arm {
            R4SoftmaxTraceStateArm::GeometricRecurrent
            | R4SoftmaxTraceStateArm::TransportPermutedControl => encode(current_frame, model_key)?,
            _ => model_key,
        });
        if self.arm != R4SoftmaxTraceStateArm::Suffix {
            let model_value = matrix_vector(
                &parameters.value_map,
                deterministic_token_r4(observed_token),
            );
            let value = match self.arm {
                R4SoftmaxTraceStateArm::GeometricRecurrent
                | R4SoftmaxTraceStateArm::TransportPermutedControl => {
                    encode(current_frame, model_value)?
                }
                _ => model_value,
            };
            if self.arm != R4SoftmaxTraceStateArm::PlainRecurrent
                && self.state.observation_count != 0
            {
                let source_offset = match self.arm {
                    R4SoftmaxTraceStateArm::TransportPermutedControl => {
                        self.audit.transport_permutations =
                            increment(self.audit.transport_permutations)?;
                        (usize::from(self.state.frame_table_offset) + 1) % self.frames.len()
                    }
                    _ => usize::from(self.state.frame_table_offset),
                };
                transport_banks(
                    &mut self.state.banks,
                    self.frames[source_offset],
                    self.frames[usize::from(canonical_frame_offset)],
                )?;
                self.audit.state_transports = increment(self.audit.state_transports)?;
            }
            update_banks(&mut self.state.banks, parameters, key, value)?;
        }
        self.state.frame_table_offset = canonical_frame_offset;
        self.state.observation_count = self
            .state
            .observation_count
            .checked_add(1)
            .ok_or(R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?;
        self.state.push_token(observed_token);
        let history = self.state.history();
        let suffix_distribution = self
            .artifact
            .suffix
            .runtime()
            .distribution(&history, R4SoftmaxTraceStudentArm::TeacherDistilled)?;
        let mut logits = Vec::with_capacity(suffix_distribution.scores.len());
        for score in suffix_distribution.scores {
            let base = (f32::from(score.weight_q16) / f32::from(R4_SOFTMAX_TRACE_Q16_TOTAL))
                .max(f32::MIN_POSITIVE)
                .ln();
            let recurrent = if self.arm == R4SoftmaxTraceStateArm::Suffix {
                0.0
            } else {
                recurrent_score(
                    &self.state.banks,
                    parameters,
                    key,
                    score.token,
                    match self.arm {
                        R4SoftmaxTraceStateArm::GeometricRecurrent
                        | R4SoftmaxTraceStateArm::TransportPermutedControl => Some(current_frame),
                        _ => None,
                    },
                )?
            };
            let logit = base + recurrent;
            if !logit.is_finite() {
                return Err(R4SoftmaxTraceStateStudentError::Invalid(
                    "state-student runtime produced a non-finite logit".to_owned(),
                ));
            }
            logits.push((score.token, logit));
        }
        let candidates = stable_distribution(&logits)?;
        let winner = candidates
            .iter()
            .max_by(|left, right| {
                left.probability
                    .total_cmp(&right.probability)
                    .then_with(|| right.token.cmp(&left.token))
            })
            .ok_or_else(|| {
                R4SoftmaxTraceStateStudentError::Invalid(
                    "state-student suffix support is empty".to_owned(),
                )
            })?;
        Ok(R4SoftmaxTraceStatePrediction {
            token: winner.token,
            probability: winner.probability,
            suffix_depth: suffix_distribution.suffix_depth,
            candidates,
            state_checksum: self.state.checksum(),
        })
    }
}

/// Deterministically compile the two matched 120-value recurrent arms while
/// embedding the already-frozen suffix artifact as their shared support/base.
pub fn compile_r4_softmax_trace_state_student(
    config: R4SoftmaxTraceStateFitConfig,
    suffix: &R4SoftmaxTraceStudentArtifact,
    construction: &[R4SoftmaxTraceStateFitSequence],
) -> Result<R4SoftmaxTraceStateStudentArtifact, R4SoftmaxTraceStateStudentError> {
    let documents = validate_and_sort_fit_sequences(config, suffix, construction)?;
    let frames = registered_frames()?;
    let construction_digest = fit_digest(config, &documents)?;

    // Both matched arms fit identical token inputs to identical model-frame
    // decoded targets. Geometry enters only through encode/transport/update.
    let geometric = fit_role_parameters(&documents, &frames)?;
    let plain = fit_role_parameters(&documents, &frames)?;
    let mut artifact = R4SoftmaxTraceStateStudentArtifact {
        maximum_token_id: config.maximum_token_id,
        construction_document_count: u32::try_from(documents.len())
            .map_err(|_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?,
        construction_position_count: suffix.construction_position_count(),
        construction_digest,
        geometric,
        plain,
        suffix: suffix.clone(),
    };
    fit_readouts(
        &mut artifact.geometric,
        &documents,
        suffix,
        &frames,
        R4SoftmaxTraceStateArm::GeometricRecurrent,
    )?;
    fit_readouts(
        &mut artifact.plain,
        &documents,
        suffix,
        &frames,
        R4SoftmaxTraceStateArm::PlainRecurrent,
    )?;
    artifact.validate()?;
    Ok(artifact)
}

fn validate_and_sort_fit_sequences<'a>(
    config: R4SoftmaxTraceStateFitConfig,
    suffix: &R4SoftmaxTraceStudentArtifact,
    construction: &'a [R4SoftmaxTraceStateFitSequence],
) -> Result<Vec<&'a R4SoftmaxTraceStateFitSequence>, R4SoftmaxTraceStateStudentError> {
    if construction.is_empty() {
        return Err(R4SoftmaxTraceStateStudentError::Invalid(
            "state-student construction set is empty".to_owned(),
        ));
    }
    let mut documents = construction.iter().collect::<Vec<_>>();
    documents.sort_by(|left, right| left.document_id.cmp(&right.document_id));
    let mut ids = BTreeSet::new();
    let mut positions = 0_u64;
    for sequence in &documents {
        if sequence.document_id.is_empty()
            || !ids.insert(sequence.document_id.as_str())
            || sequence.events.is_empty()
        {
            return Err(R4SoftmaxTraceStateStudentError::Invalid(
                "state-student construction document identity/shape is invalid".to_owned(),
            ));
        }
        positions = positions
            .checked_add(
                u64::try_from(sequence.events.len())
                    .map_err(|_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?,
            )
            .ok_or(R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?;
        for (position, event) in sequence.events.iter().enumerate() {
            if usize::try_from(event.position).ok() != Some(position)
                || event.observed_token > config.maximum_token_id
                || event.actual_next_token > config.maximum_token_id
                || usize::from(event.frame_table_offset) >= H4_FRAME_COUNT
            {
                return Err(R4SoftmaxTraceStateStudentError::Invalid(
                    "state-student construction event is non-causal or outside the namespace"
                        .to_owned(),
                ));
            }
            require_finite_vector(event.query_trace_r4, "query trace feature")?;
            require_finite_vector(event.key_trace_r4, "key trace feature")?;
            require_finite_vector(event.value_trace_r4, "value trace feature")?;
            // Re-run the public constructor to enforce canonical nonzero,
            // unique, normalized Q16 teacher support.
            TeacherTopDistributionQ16::new(event.teacher_top_distribution.entries.clone())?;
        }
    }
    if documents.len()
        != usize::try_from(suffix.construction_document_count())
            .map_err(|_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?
        || positions != suffix.construction_position_count()
    {
        return Err(R4SoftmaxTraceStateStudentError::Invalid(
            "state-student construction census differs from the frozen suffix artifact".to_owned(),
        ));
    }
    Ok(documents)
}

fn fit_role_parameters(
    documents: &[&R4SoftmaxTraceStateFitSequence],
    frames: &[R4RegisteredSpinFrame],
) -> Result<R4SoftmaxTraceStateParameters, R4SoftmaxTraceStateStudentError> {
    let mut inputs = Vec::new();
    let mut queries = Vec::new();
    let mut keys = Vec::new();
    let mut values = Vec::new();
    for sequence in documents {
        for event in &sequence.events {
            inputs.push(deterministic_token_r4(event.observed_token));
            let frame = frames[usize::from(event.frame_table_offset)];
            queries.push(decode(frame, event.query_trace_r4)?);
            keys.push(decode(frame, event.key_trace_r4)?);
            values.push(decode(frame, event.value_trace_r4)?);
        }
    }
    Ok(R4SoftmaxTraceStateParameters::new(
        ridge_map(&inputs, &keys)?,
        ridge_map(&inputs, &values)?,
        ridge_map(&inputs, &queries)?,
    ))
}

#[derive(Clone)]
struct ReadoutCandidate {
    features: [f32; 64],
    base_logit: f32,
    teacher_probability: f32,
}

#[derive(Clone)]
struct ReadoutRow {
    candidates: Vec<ReadoutCandidate>,
}

fn fit_readouts(
    parameters: &mut R4SoftmaxTraceStateParameters,
    documents: &[&R4SoftmaxTraceStateFitSequence],
    suffix: &R4SoftmaxTraceStudentArtifact,
    frames: &[R4RegisteredSpinFrame],
    arm: R4SoftmaxTraceStateArm,
) -> Result<(), R4SoftmaxTraceStateStudentError> {
    let mut rows = Vec::new();
    for sequence in documents {
        let mut state = R4SoftmaxTraceState::default();
        let mut history = Vec::new();
        for event in &sequence.events {
            let current_frame = frames[usize::from(event.frame_table_offset)];
            let model_key = matrix_vector(
                &parameters.key_map,
                deterministic_token_r4(event.observed_token),
            );
            let model_value = matrix_vector(
                &parameters.value_map,
                deterministic_token_r4(event.observed_token),
            );
            let key = normalize_or_axis(if arm == R4SoftmaxTraceStateArm::GeometricRecurrent {
                encode(current_frame, model_key)?
            } else {
                model_key
            });
            let value = if arm == R4SoftmaxTraceStateArm::GeometricRecurrent {
                encode(current_frame, model_value)?
            } else {
                model_value
            };
            if arm == R4SoftmaxTraceStateArm::GeometricRecurrent && state.observation_count != 0 {
                transport_banks(
                    &mut state.banks,
                    frames[usize::from(state.frame_table_offset)],
                    frames[usize::from(event.frame_table_offset)],
                )?;
            }
            update_banks(&mut state.banks, parameters, key, value)?;
            state.frame_table_offset = event.frame_table_offset;
            state.observation_count = state
                .observation_count
                .checked_add(1)
                .ok_or(R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?;
            state.push_token(event.observed_token);
            history.push(event.observed_token);
            let suffix_row = suffix
                .runtime()
                .distribution(&history, R4SoftmaxTraceStudentArm::TeacherDistilled)?;
            let overlap_mass = suffix_row
                .scores
                .iter()
                .filter_map(|candidate| {
                    event
                        .teacher_top_distribution
                        .entries
                        .iter()
                        .find(|entry| entry.token == candidate.token)
                })
                .map(|entry| u32::from(entry.probability_q16))
                .sum::<u32>();
            if overlap_mass == 0 {
                return Err(R4SoftmaxTraceStateStudentError::Invalid(
                    "state-student readout row has zero teacher/support overlap".to_owned(),
                ));
            }
            let mut candidates = Vec::with_capacity(suffix_row.scores.len());
            for candidate in &suffix_row.scores {
                let teacher_q16 = event
                    .teacher_top_distribution
                    .entries
                    .iter()
                    .find(|entry| entry.token == candidate.token)
                    .map_or(0, |entry| entry.probability_q16);
                let base_logit = (f32::from(candidate.weight_q16)
                    / f32::from(R4_SOFTMAX_TRACE_Q16_TOTAL))
                .max(f32::MIN_POSITIVE)
                .ln();
                candidates.push(ReadoutCandidate {
                    features: readout_features(
                        &state.banks,
                        parameters,
                        key,
                        candidate.token,
                        (arm == R4SoftmaxTraceStateArm::GeometricRecurrent)
                            .then_some(current_frame),
                    )?,
                    base_logit,
                    teacher_probability: f32::from(teacher_q16) / overlap_mass as f32,
                });
            }
            rows.push(ReadoutRow { candidates });
        }
    }
    if rows.is_empty() {
        return Err(R4SoftmaxTraceStateStudentError::Invalid(
            "state-student readout training set is empty".to_owned(),
        ));
    }
    let mut weights = [0.0_f32; 64];
    let inverse_count = 1.0 / rows.len() as f32;
    for _ in 0..READOUT_STEPS {
        let mut gradient = [0.0_f32; 64];
        for row in &rows {
            let mut logits = row
                .candidates
                .iter()
                .map(|candidate| {
                    f64::from(candidate.base_logit + dot64(weights, candidate.features))
                })
                .collect::<Vec<_>>();
            let mut probabilities = vec![0.0_f32; logits.len()];
            intrinsic_stable_softmax_into(&mut logits, &mut probabilities)
                .map_err(|error| R4SoftmaxTraceStateStudentError::Invalid(error.to_string()))?;
            for (candidate, model_probability) in row.candidates.iter().zip(probabilities) {
                let error = model_probability - candidate.teacher_probability;
                for (gradient_value, feature) in gradient.iter_mut().zip(candidate.features) {
                    *gradient_value += error * feature;
                }
            }
        }
        for index in 0..64 {
            gradient[index] = gradient[index] * inverse_count + RIDGE_LAMBDA * weights[index];
            weights[index] -= READOUT_RATE * gradient[index];
            if !weights[index].is_finite() {
                return Err(R4SoftmaxTraceStateStudentError::Invalid(
                    "state-student readout fitting diverged".to_owned(),
                ));
            }
        }
    }
    for bank in 0..4 {
        for row in 0..4 {
            for column in 0..4 {
                parameters.readout_maps[bank][row][column] = weights[bank * 16 + row * 4 + column];
            }
        }
    }
    Ok(())
}

fn ridge_map(
    inputs: &[Vector4],
    targets: &[Vector4],
) -> Result<Matrix4, R4SoftmaxTraceStateStudentError> {
    if inputs.is_empty() || inputs.len() != targets.len() {
        return Err(R4SoftmaxTraceStateStudentError::Invalid(
            "state-student ridge shapes are invalid".to_owned(),
        ));
    }
    let mut normal = [[0.0_f32; 4]; 4];
    let mut cross = [[0.0_f32; 4]; 4];
    for (input, target) in inputs.iter().zip(targets) {
        for row in 0..4 {
            for column in 0..4 {
                normal[row][column] += input[row] * input[column];
                cross[row][column] += target[row] * input[column];
            }
        }
    }
    for (lane, row) in normal.iter_mut().enumerate() {
        row[lane] += RIDGE_LAMBDA;
    }
    let mut output = [[0.0_f32; 4]; 4];
    for target_lane in 0..4 {
        for _ in 0..RIDGE_SWEEPS {
            for lane in 0..4 {
                let mut residual = cross[target_lane][lane];
                for other in 0..4 {
                    if other != lane {
                        residual -= normal[lane][other] * output[target_lane][other];
                    }
                }
                output[target_lane][lane] = residual / normal[lane][lane];
            }
        }
    }
    require_finite_matrix(&output, "ridge map")?;
    Ok(output)
}

fn update_banks(
    banks: &mut [Matrix4; 4],
    parameters: &R4SoftmaxTraceStateParameters,
    key: Vector4,
    value: Vector4,
) -> Result<(), R4SoftmaxTraceStateStudentError> {
    for (bank_index, bank_matrix) in banks.iter_mut().enumerate() {
        let prediction = matrix_vector(bank_matrix, key);
        let mut error = [0.0; 4];
        for lane in 0..4 {
            error[lane] = value[lane] - prediction[lane];
        }
        for row in 0..4 {
            for column in 0..4 {
                bank_matrix[row][column] = parameters.rho[bank_index] * bank_matrix[row][column]
                    + parameters.eta[bank_index] * error[row] * key[column];
            }
        }
        require_finite_matrix(bank_matrix, "recurrent bank update")?;
    }
    Ok(())
}

fn transport_banks(
    banks: &mut [Matrix4; 4],
    source: R4RegisteredSpinFrame,
    destination: R4RegisteredSpinFrame,
) -> Result<(), R4SoftmaxTraceStateStudentError> {
    let connection = connection_matrix(source, destination)?;
    let transpose = transpose(connection);
    for bank in banks {
        *bank = matrix_multiply(matrix_multiply(connection, *bank), transpose);
        require_finite_matrix(bank, "transported recurrent bank")?;
    }
    Ok(())
}

fn connection_matrix(
    source: R4RegisteredSpinFrame,
    destination: R4RegisteredSpinFrame,
) -> Result<Matrix4, R4SoftmaxTraceStateStudentError> {
    let mut connection = [[0.0; 4]; 4];
    for column in 0..4 {
        let mut basis = [0.0_f64; 4];
        basis[column] = 1.0;
        let model = source
            .decode_local_block(basis)
            .map_err(|error| R4SoftmaxTraceStateStudentError::Invalid(error.to_string()))?;
        let local = destination
            .encode_model_block(model)
            .map_err(|error| R4SoftmaxTraceStateStudentError::Invalid(error.to_string()))?;
        for row in 0..4 {
            connection[row][column] = local[row] as f32;
        }
    }
    require_finite_matrix(&connection, "H4 connection matrix")?;
    Ok(connection)
}

fn decode(
    frame: R4RegisteredSpinFrame,
    local: Vector4,
) -> Result<Vector4, R4SoftmaxTraceStateStudentError> {
    let model = frame
        .decode_local_block(local.map(f64::from))
        .map_err(|error| R4SoftmaxTraceStateStudentError::Invalid(error.to_string()))?;
    let output = model.map(|value| value as f32);
    require_finite_vector(output, "H4 model-frame decoding")?;
    Ok(output)
}

fn encode(
    frame: R4RegisteredSpinFrame,
    model: Vector4,
) -> Result<Vector4, R4SoftmaxTraceStateStudentError> {
    let local = frame
        .encode_model_block(model.map(f64::from))
        .map_err(|error| R4SoftmaxTraceStateStudentError::Invalid(error.to_string()))?;
    let output = local.map(|value| value as f32);
    require_finite_vector(output, "H4 local-frame encoding")?;
    Ok(output)
}

fn readout_features(
    banks: &[Matrix4; 4],
    parameters: &R4SoftmaxTraceStateParameters,
    current_key: Vector4,
    candidate_token: u32,
    current_frame: Option<R4RegisteredSpinFrame>,
) -> Result<[f32; 64], R4SoftmaxTraceStateStudentError> {
    let model_candidate_query = matrix_vector(
        &parameters.query_map,
        deterministic_token_r4(candidate_token),
    );
    let candidate_query = if let Some(frame) = current_frame {
        encode(frame, model_candidate_query)?
    } else {
        model_candidate_query
    };
    let mut features = [0.0; 64];
    for bank in 0..4 {
        let response = matrix_vector(&banks[bank], current_key);
        for row in 0..4 {
            for column in 0..4 {
                features[bank * 16 + row * 4 + column] = response[row] * candidate_query[column];
            }
        }
    }
    Ok(features)
}

fn recurrent_score(
    banks: &[Matrix4; 4],
    parameters: &R4SoftmaxTraceStateParameters,
    current_key: Vector4,
    candidate_token: u32,
    current_frame: Option<R4RegisteredSpinFrame>,
) -> Result<f32, R4SoftmaxTraceStateStudentError> {
    let features = readout_features(
        banks,
        parameters,
        current_key,
        candidate_token,
        current_frame,
    )?;
    let weights = flattened_readout_weights(parameters);
    Ok(dot64(weights, features))
}

fn flattened_readout_weights(
    parameters: &R4SoftmaxTraceStateParameters,
) -> [f32; R4_SOFTMAX_TRACE_STATE_READOUT_FEATURES] {
    let mut weights = [0.0; R4_SOFTMAX_TRACE_STATE_READOUT_FEATURES];
    for bank in 0..4 {
        for row in 0..4 {
            for column in 0..4 {
                weights[bank * 16 + row * 4 + column] = parameters.readout_maps[bank][row][column];
            }
        }
    }
    weights
}

fn fit_digest(
    config: R4SoftmaxTraceStateFitConfig,
    documents: &[&R4SoftmaxTraceStateFitSequence],
) -> Result<[u8; 32], R4SoftmaxTraceStateStudentError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(R4_SOFTMAX_TRACE_STATE_STUDENT_SCHEMA.as_bytes());
    hasher.update(&config.maximum_token_id.to_le_bytes());
    hasher.update(
        &u32::try_from(documents.len())
            .map_err(|_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?
            .to_le_bytes(),
    );
    for sequence in documents {
        let length = u32::try_from(sequence.document_id.len())
            .map_err(|_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?;
        hasher.update(&length.to_le_bytes());
        hasher.update(sequence.document_id.as_bytes());
        hasher.update(
            &u32::try_from(sequence.events.len())
                .map_err(|_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?
                .to_le_bytes(),
        );
        for event in &sequence.events {
            hasher.update(&event.position.to_le_bytes());
            hasher.update(&event.observed_token.to_le_bytes());
            hasher.update(&event.actual_next_token.to_le_bytes());
            hasher.update(&event.frame_table_offset.to_le_bytes());
            for vector in [
                event.query_trace_r4,
                event.key_trace_r4,
                event.value_trace_r4,
            ] {
                for value in vector {
                    hasher.update(&value.to_bits().to_le_bytes());
                }
            }
            hasher.update(
                &u32::try_from(event.teacher_top_distribution.entries.len())
                    .map_err(|_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?
                    .to_le_bytes(),
            );
            for entry in &event.teacher_top_distribution.entries {
                hasher.update(&entry.token.to_le_bytes());
                hasher.update(&entry.probability_q16.to_le_bytes());
            }
        }
    }
    Ok(*hasher.finalize().as_bytes())
}

fn registered_frames() -> Result<Vec<R4RegisteredSpinFrame>, R4SoftmaxTraceStateStudentError> {
    let frames = canonical_registered_h4_spin_frames()
        .map_err(|error| R4SoftmaxTraceStateStudentError::Invalid(error.to_string()))?;
    if frames.len() != H4_FRAME_COUNT
        || frames
            .iter()
            .enumerate()
            .any(|(index, frame)| usize::from(frame.h4_table_offset()) != index)
    {
        return Err(R4SoftmaxTraceStateStudentError::Invalid(
            "canonical H4 registry is incomplete or noncanonical".to_owned(),
        ));
    }
    Ok(frames)
}

fn stable_distribution(
    logits: &[(u32, f32)],
) -> Result<Vec<R4SoftmaxTraceStateScore>, R4SoftmaxTraceStateStudentError> {
    if logits.is_empty() {
        return Err(R4SoftmaxTraceStateStudentError::Invalid(
            "cannot normalize an empty state-student row".to_owned(),
        ));
    }
    let mut softmax_logits = logits
        .iter()
        .map(|(_, logit)| f64::from(*logit))
        .collect::<Vec<_>>();
    let mut probabilities = vec![0.0_f32; logits.len()];
    intrinsic_stable_softmax_into(&mut softmax_logits, &mut probabilities)
        .map_err(|error| R4SoftmaxTraceStateStudentError::Invalid(error.to_string()))?;
    Ok(logits
        .iter()
        .zip(probabilities)
        .map(|((token, logit), probability)| R4SoftmaxTraceStateScore {
            token: *token,
            probability,
            logit: *logit,
        })
        .collect())
}

fn matrix_vector(matrix: &Matrix4, vector: Vector4) -> Vector4 {
    let mut output = [0.0; 4];
    for row in 0..4 {
        for column in 0..4 {
            output[row] += matrix[row][column] * vector[column];
        }
    }
    output
}

fn matrix_multiply(left: Matrix4, right: Matrix4) -> Matrix4 {
    let mut output = [[0.0; 4]; 4];
    for row in 0..4 {
        for column in 0..4 {
            for inner in 0..4 {
                output[row][column] += left[row][inner] * right[inner][column];
            }
        }
    }
    output
}

fn transpose(matrix: Matrix4) -> Matrix4 {
    let mut output = [[0.0; 4]; 4];
    for row in 0..4 {
        for column in 0..4 {
            output[row][column] = matrix[column][row];
        }
    }
    output
}

fn normalize_or_axis(mut vector: Vector4) -> Vector4 {
    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm.is_finite() && norm > 1.0e-12 {
        for value in &mut vector {
            *value /= norm;
        }
        vector
    } else {
        [1.0, 0.0, 0.0, 0.0]
    }
}

fn dot64(left: [f32; 64], right: [f32; 64]) -> f32 {
    left.iter()
        .zip(right)
        .map(|(left, right)| *left * right)
        .sum()
}

fn require_finite_vector(
    vector: Vector4,
    label: &str,
) -> Result<(), R4SoftmaxTraceStateStudentError> {
    if vector.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(R4SoftmaxTraceStateStudentError::Invalid(format!(
            "{label} contains a non-finite value"
        )))
    }
}

fn require_finite_matrix(
    matrix: &Matrix4,
    label: &str,
) -> Result<(), R4SoftmaxTraceStateStudentError> {
    if matrix.iter().flatten().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(R4SoftmaxTraceStateStudentError::Invalid(format!(
            "{label} contains a non-finite value"
        )))
    }
}

fn read_parameters(
    cursor: &mut Cursor<'_>,
) -> Result<R4SoftmaxTraceStateParameters, R4SoftmaxTraceStateStudentError> {
    let mut values = [0.0; R4_SOFTMAX_TRACE_STATE_PARAMETER_VALUES_PER_ARM];
    for value in &mut values {
        *value = f32::from_bits(cursor.u32()?);
    }
    R4SoftmaxTraceStateParameters::from_values(values)
}

fn increment(value: u64) -> Result<u64, R4SoftmaxTraceStateStudentError> {
    value
        .checked_add(1)
        .ok_or(R4SoftmaxTraceStateStudentError::ArithmeticOverflow)
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
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

    fn take(&mut self, length: usize) -> Result<&'a [u8], R4SoftmaxTraceStateStudentError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(R4SoftmaxTraceStateStudentError::ArithmeticOverflow)?;
        let value = self.bytes.get(self.offset..end).ok_or_else(|| {
            R4SoftmaxTraceStateStudentError::Invalid(
                "state-student canonical bytes are truncated".to_owned(),
            )
        })?;
        self.offset = end;
        Ok(value)
    }

    fn u16(&mut self) -> Result<u16, R4SoftmaxTraceStateStudentError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().map_err(
            |_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow,
        )?))
    }

    fn u32(&mut self) -> Result<u32, R4SoftmaxTraceStateStudentError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().map_err(
            |_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow,
        )?))
    }

    fn u64(&mut self) -> Result<u64, R4SoftmaxTraceStateStudentError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().map_err(
            |_| R4SoftmaxTraceStateStudentError::ArithmeticOverflow,
        )?))
    }

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r4_softmax_trace_student::{
        compile_r4_softmax_trace_student, R4SoftmaxTraceSequence, R4SoftmaxTraceStudentConfig,
        TeacherTopTokenQ16,
    };

    fn distribution(primary: u32, secondary: u32) -> TeacherTopDistributionQ16 {
        TeacherTopDistributionQ16::new(vec![
            TeacherTopTokenQ16::new(primary, 50_000),
            TeacherTopTokenQ16::new(secondary, 15_535),
        ])
        .expect("distribution")
    }

    fn fixture() -> (
        R4SoftmaxTraceStudentArtifact,
        Vec<R4SoftmaxTraceStateFitSequence>,
    ) {
        let suffix_sequences = vec![
            R4SoftmaxTraceSequence::new(
                "a",
                vec![1, 2],
                vec![3, 4],
                vec![distribution(3, 4), distribution(4, 3)],
            ),
            R4SoftmaxTraceSequence::new(
                "b",
                vec![2, 3],
                vec![4, 5],
                vec![distribution(4, 5), distribution(5, 4)],
            ),
        ];
        let suffix = compile_r4_softmax_trace_student(
            R4SoftmaxTraceStudentConfig::default(),
            &suffix_sequences,
        )
        .expect("suffix");
        let fit = suffix_sequences
            .iter()
            .enumerate()
            .map(|(document, sequence)| R4SoftmaxTraceStateFitSequence {
                document_id: sequence.document_id.clone(),
                events: sequence
                    .input_tokens
                    .iter()
                    .enumerate()
                    .map(|(position, token)| {
                        let feature = deterministic_token_r4(*token);
                        R4SoftmaxTraceStateFitEvent {
                            position: position as u32,
                            observed_token: *token,
                            actual_next_token: sequence.actual_next_tokens[position],
                            frame_table_offset: (document * 3 + position + 1) as u16,
                            query_trace_r4: feature,
                            key_trace_r4: feature,
                            value_trace_r4: feature,
                            teacher_top_distribution: sequence.teacher_top_distributions[position]
                                .clone(),
                        }
                    })
                    .collect(),
            })
            .collect();
        (suffix, fit)
    }

    #[test]
    fn state_is_exactly_278_bytes_and_round_trips() {
        let state = R4SoftmaxTraceState::default();
        let bytes = state.to_bytes();
        assert_eq!(bytes.len(), 278);
        assert_eq!(
            R4SoftmaxTraceState::from_bytes(&bytes).expect("reload"),
            state
        );
        assert!(R4SoftmaxTraceState::from_bytes(&bytes[..277]).is_err());
    }

    #[test]
    fn compiler_is_deterministic_and_budgets_are_exact() {
        let (suffix, fit) = fixture();
        let config = R4SoftmaxTraceStateFitConfig {
            maximum_token_id: 10,
        };
        let first =
            compile_r4_softmax_trace_state_student(config, &suffix, &fit).expect("first compile");
        let second =
            compile_r4_softmax_trace_state_student(config, &suffix, &fit).expect("second compile");
        assert_eq!(first.to_bytes(), second.to_bytes());
        assert_eq!(first.parameter_values_per_arm(), 120);
        assert_eq!(first.fitted_parameter_values_per_arm(), 112);
        assert_eq!(
            first.payload_bytes_before_headers(),
            suffix.to_bytes().len() + 960
        );
        let bytes = first.to_bytes();
        assert_eq!(
            R4SoftmaxTraceStateStudentArtifact::from_bytes(&bytes)
                .expect("artifact reload")
                .to_bytes(),
            bytes
        );
        let cid = first.artifact_cid();
        assert!(
            R4SoftmaxTraceStateStudentArtifact::from_bytes_with_expected_cid(&bytes, &cid).is_ok()
        );
        assert!(
            R4SoftmaxTraceStateStudentArtifact::from_bytes_with_expected_cid(
                &bytes,
                "blake3:0000000000000000000000000000000000000000000000000000000000000000"
            )
            .is_err()
        );
    }

    #[test]
    fn runtime_is_source_free_replayable_and_controls_transport() {
        let (suffix, fit) = fixture();
        let artifact = compile_r4_softmax_trace_state_student(
            R4SoftmaxTraceStateFitConfig {
                maximum_token_id: 10,
            },
            &suffix,
            &fit,
        )
        .expect("compile");
        let mut geometric = artifact
            .runtime(R4SoftmaxTraceStateArm::GeometricRecurrent)
            .expect("geometric runtime");
        let first = geometric.observe_and_predict(1, 1).expect("first");
        let artifact_bytes = artifact.to_bytes();
        let artifact_cid = artifact.artifact_cid();
        let state_bytes = geometric.state().to_bytes();
        let state_checksum = geometric.state().checksum();
        let mut replay = R4SoftmaxTraceStateRuntime::from_artifact_and_state_bytes(
            &artifact_bytes,
            &artifact_cid,
            &state_bytes,
            &state_checksum,
            R4SoftmaxTraceStateArm::GeometricRecurrent,
        )
        .expect("replay runtime");
        let second = geometric.observe_and_predict(2, 2).expect("second");
        let replayed = replay.observe_and_predict(2, 2).expect("replayed second");
        assert_eq!(second, replayed);
        assert_ne!(first.state_checksum, second.state_checksum);
        let audit = geometric.audit();
        assert_eq!(audit.source_model_forwards, 0);
        assert_eq!(audit.source_trace_reads, 0);
        assert_eq!(audit.teacher_distribution_reads, 0);
        assert_eq!(audit.target_reads, 0);
        assert_eq!(audit.future_token_reads, 0);
        assert_eq!(audit.state_transports, 1);

        let mut permuted = artifact
            .runtime(R4SoftmaxTraceStateArm::TransportPermutedControl)
            .expect("permuted runtime");
        permuted.observe_and_predict(1, 1).expect("first permuted");
        let permuted_second = permuted.observe_and_predict(2, 2).expect("second permuted");
        assert_eq!(permuted.audit().transport_permutations, 1);
        assert_ne!(second.state_checksum, permuted_second.state_checksum);

        assert!(R4SoftmaxTraceStateRuntime::from_artifact_and_state_bytes(
            &artifact_bytes,
            "blake3:0000000000000000000000000000000000000000000000000000000000000000",
            &state_bytes,
            &state_checksum,
            R4SoftmaxTraceStateArm::GeometricRecurrent,
        )
        .is_err());
        assert!(R4SoftmaxTraceStateRuntime::from_artifact_and_state_bytes(
            &artifact_bytes,
            &artifact_cid,
            &state_bytes,
            "blake3:0000000000000000000000000000000000000000000000000000000000000000",
            R4SoftmaxTraceStateArm::GeometricRecurrent,
        )
        .is_err());

        let mut out_of_namespace = geometric.state().clone();
        out_of_namespace.token_ring[3] = 11;
        let out_of_namespace_bytes = out_of_namespace.to_bytes();
        assert!(R4SoftmaxTraceStateRuntime::from_artifact_and_state_bytes(
            &artifact_bytes,
            &artifact_cid,
            &out_of_namespace_bytes,
            &out_of_namespace.checksum(),
            R4SoftmaxTraceStateArm::GeometricRecurrent,
        )
        .is_err());

        let mut noncanonical_initial = R4SoftmaxTraceState::default();
        noncanonical_initial.banks[0][0][0] = 1.0;
        assert!(R4SoftmaxTraceState::from_bytes(&noncanonical_initial.to_bytes()).is_err());
    }

    #[test]
    fn diagnostic_snapshot_is_exact_and_read_only_for_all_recurrent_arms() {
        let (suffix, fit) = fixture();
        let artifact = compile_r4_softmax_trace_state_student(
            R4SoftmaxTraceStateFitConfig {
                maximum_token_id: 10,
            },
            &suffix,
            &fit,
        )
        .expect("compile");

        for arm in [
            R4SoftmaxTraceStateArm::PlainRecurrent,
            R4SoftmaxTraceStateArm::GeometricRecurrent,
            R4SoftmaxTraceStateArm::TransportPermutedControl,
        ] {
            let mut runtime = artifact.runtime(arm).expect("runtime");
            assert!(runtime.diagnostic_readout_snapshot().is_err());
            runtime.observe_and_predict(1, 1).expect("first prediction");
            let prediction = runtime
                .observe_and_predict(2, 2)
                .expect("second prediction");
            let state_before = runtime.state().to_bytes();
            let audit_before = runtime.audit();

            let snapshot = runtime
                .diagnostic_readout_snapshot()
                .expect("diagnostic snapshot");
            assert_eq!(
                snapshot,
                runtime
                    .diagnostic_readout_snapshot()
                    .expect("replayed diagnostic snapshot")
            );
            assert_eq!(runtime.state().to_bytes(), state_before);
            assert_eq!(runtime.audit(), audit_before);
            assert_eq!(snapshot.arm, arm);
            assert_eq!(snapshot.suffix_depth, prediction.suffix_depth);
            assert_eq!(snapshot.state_checksum, prediction.state_checksum);
            assert_eq!(snapshot.candidates.len(), prediction.candidates.len());

            let parameters = if arm == R4SoftmaxTraceStateArm::PlainRecurrent {
                artifact.plain_parameters()
            } else {
                artifact.geometric_parameters()
            };
            let weights = flattened_readout_weights(parameters);
            for (diagnostic, predicted) in snapshot.candidates.iter().zip(&prediction.candidates) {
                assert_eq!(diagnostic.token, predicted.token);
                assert_eq!(
                    diagnostic.readout_features.len(),
                    R4_SOFTMAX_TRACE_STATE_READOUT_FEATURES
                );
                assert!(diagnostic
                    .readout_features
                    .iter()
                    .all(|value| value.is_finite()));
                assert_eq!(
                    diagnostic.residual_logit.to_bits(),
                    dot64(weights, diagnostic.readout_features).to_bits()
                );
                assert_eq!(
                    diagnostic.total_logit.to_bits(),
                    (diagnostic.base_logit + diagnostic.residual_logit).to_bits()
                );
                assert_eq!(diagnostic.total_logit.to_bits(), predicted.logit.to_bits());
            }
        }

        let mut suffix_runtime = artifact
            .runtime(R4SoftmaxTraceStateArm::Suffix)
            .expect("suffix runtime");
        suffix_runtime
            .observe_and_predict(2, 2)
            .expect("suffix prediction");
        assert!(suffix_runtime.diagnostic_readout_snapshot().is_err());
    }

    #[test]
    fn same_causal_prefix_has_identical_prediction() {
        let (suffix, fit) = fixture();
        let artifact = compile_r4_softmax_trace_state_student(
            R4SoftmaxTraceStateFitConfig {
                maximum_token_id: 10,
            },
            &suffix,
            &fit,
        )
        .expect("compile");
        let mut left = artifact
            .runtime(R4SoftmaxTraceStateArm::PlainRecurrent)
            .expect("left");
        let mut right = artifact
            .runtime(R4SoftmaxTraceStateArm::PlainRecurrent)
            .expect("right");
        assert_eq!(
            left.observe_and_predict(1, 1).expect("left prefix"),
            right.observe_and_predict(1, 1).expect("right prefix")
        );
    }

    #[test]
    fn signed_reduction_is_finite_and_role_separated() {
        let blocks = vec![[1.0, -2.0, 3.0, -4.0]; 144];
        let query = signed_reduce_final_layer_r4(R4SoftmaxTraceReductionRole::Query, &blocks)
            .expect("query reduction");
        let key = signed_reduce_final_layer_r4(R4SoftmaxTraceReductionRole::Key, &blocks)
            .expect("key reduction");
        assert_ne!(query, key);
        assert!(
            signed_reduce_final_layer_r4(R4SoftmaxTraceReductionRole::Value, &blocks[..143])
                .is_err()
        );
    }

    #[test]
    fn signed_reduction_commutes_with_registered_h4_frame() {
        let blocks = (0..144)
            .map(|index| {
                let scale = index as f32 + 1.0;
                [scale, -0.5 * scale, 0.25 * scale, -0.125 * scale]
            })
            .collect::<Vec<_>>();
        let frame = registered_frames().expect("frames")[17];
        let reduced_model =
            signed_reduce_final_layer_r4(R4SoftmaxTraceReductionRole::Query, &blocks)
                .expect("model reduction");
        let encoded_blocks = blocks
            .iter()
            .map(|block| encode(frame, *block).expect("encode block"))
            .collect::<Vec<_>>();
        let reduced_local =
            signed_reduce_final_layer_r4(R4SoftmaxTraceReductionRole::Query, &encoded_blocks)
                .expect("local reduction");
        let encoded_reduction = encode(frame, reduced_model).expect("encode reduction");
        for lane in 0..4 {
            assert!(
                (reduced_local[lane] - encoded_reduction[lane]).abs() <= 2.0e-4,
                "lane {lane}: local={} encoded={}",
                reduced_local[lane],
                encoded_reduction[lane]
            );
        }
    }
}
