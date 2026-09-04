//! Exact HTTP, configuration, and private IPC wire vocabulary from #1105.

use serde::{Deserialize, Serialize};
use std::fmt;
use uor_r4_api::learned_reference::ExpectedBinding;

pub const UINT53_MAX: u64 = 9_007_199_254_740_991;
pub const CONFIGURED_MODEL_ID: &str =
    "r4lr:sha256:2c209590a64cae16a4140fd43adc1cb1f87b357c02e3d4959f1e37f4ab8cd5ab";
pub const ARTIFACT_SHA256: &str =
    "2c209590a64cae16a4140fd43adc1cb1f87b357c02e3d4959f1e37f4ab8cd5ab";
pub const ARTIFACT_BYTES: u64 = 2_172_252;
pub const NATIVE_STATE_SHA256: &str =
    "4f453da12a9346356e64b6c16abfbaad1ca99e3966173cd79e9ddbc8c2d9341b";
pub const NATIVE_CONTRACT_SHA256: &str =
    "e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115";
pub const OPERATOR_PROFILE: &str = "cpu-scalar-f32-f64-1086/1";
pub const ORIGINAL_EXPORT_RELEASE_SHA256: &str =
    "2c3c2f73eb6cf804eb69b2afb0f979ae623a512ca0492e47df2af70d6cbaca8b";
pub const HISTORICAL_BINARY_SHA256: &str =
    "d423d8d3c3acd2d1c6215c21206e1bec7583e4dd37e84f30f70f79e77c40d53f";
pub const HISTORICAL_QUALIFICATION_SHA256: &str =
    "61d29aa80e6bcd3d163b2ff2a6da4faab04414ea9f4284d80b798c4e46cf5369";
pub const FIRST_TARGET: &str = "aarch64-apple-darwin";
pub const BIND_HOST: &str = "127.0.0.1";
pub const OPERATION_ID: &str = "answer_four_fact_raw_text/v1";
pub const RAW_INPUT_SCHEMA: &str = "uor-r4.text-to-clauses/1";
pub const MODEL_RESULT_SCHEMA: &str = "uor-r4.text-binding-result/1";
pub const REFUSAL_RESULT_SCHEMA: &str = "uor-r4.text-to-clauses-result/1";
pub const RAW_DECODED_MAX_BYTES: usize = 8_192;
pub const CORE_INPUT_POLICY_MAX_BYTES: u64 = 4_096;

pub const CAPABILITIES_SCHEMA: &str = "uor-r4.workbench-capabilities/1";
pub const MODEL_SCHEMA: &str = "uor-r4.workbench-model/1";
pub const LOAD_SCHEMA: &str = "uor-r4.workbench-load/1";
pub const UNLOAD_SCHEMA: &str = "uor-r4.workbench-unload/1";
pub const REQUEST_SCHEMA: &str = "uor-r4.workbench-request/1";
pub const CANCEL_SCHEMA: &str = "uor-r4.workbench-cancel/1";
pub const ERROR_SCHEMA: &str = "uor-r4.workbench-error/1";
pub const JOB_SCHEMA: &str = "uor-r4.workbench-job/1";
pub const IPC_SCHEMA: &str = "uor-r4.workbench-ipc/1";
pub const CONFIG_SCHEMA: &str = "uor-r4.workbench-config/1";
pub const ASSET_SCHEMA: &str = "uor-r4.workbench-assets/1";
pub const HOST_ACCEPTANCE_SCHEMA: &str = "uor-r4.workbench-host-acceptance/1";
pub const NATIVE_QUALIFICATION_SCHEMA: &str = "uor-r4.native-reference-qualification/1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireError {
    message: String,
}

impl WireError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for WireError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for WireError {}

pub fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub fn is_blake3_cid(value: &str) -> bool {
    value
        .strip_prefix("blake3:")
        .is_some_and(|digest| is_hex(digest, 64))
}

pub fn is_job_id(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 16
        || value.starts_with('0')
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return false;
    }
    value
        .parse::<u64>()
        .is_ok_and(|parsed| (1..=UINT53_MAX).contains(&parsed))
}

fn require(condition: bool, message: &'static str) -> Result<(), WireError> {
    condition
        .then_some(())
        .ok_or_else(|| WireError::new(message))
}

fn require_uint53(value: u64) -> Result<(), WireError> {
    require(value <= UINT53_MAX, "integer exceeds uint53")
}

fn require_hex64(value: &str) -> Result<(), WireError> {
    require(is_hex(value, 64), "expected lowercase hex64")
}

fn require_hex32(value: &str) -> Result<(), WireError> {
    require(is_hex(value, 32), "expected lowercase hex32")
}

fn require_literal(actual: &str, expected: &'static str) -> Result<(), WireError> {
    require(actual == expected, "wire literal does not match contract")
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelState {
    Unavailable,
    Unloaded,
    Loading,
    Ready,
    Running,
    Stopping,
    Unloading,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Accepted,
    Loading,
    Running,
    Stopping,
    Unloading,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    Load,
    Answer,
    Unload,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    UserCancel,
    Deadline,
    WorkerFailure,
    Shutdown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStage {
    Idle,
    ReadingArtifact,
    Validating,
    Qualifying,
    Ready,
    Inference,
    Terminating,
    Reaping,
    Complete,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceErrorTag {
    BadRequest,
    UnsupportedSchema,
    UnsupportedOperation,
    InvalidBase64,
    OriginRejected,
    HostRejected,
    NotFound,
    ModelNotFound,
    JobNotFound,
    MethodNotAllowed,
    StaleInstance,
    StaleModel,
    Busy,
    NotReady,
    AlreadyLoaded,
    AlreadyUnloaded,
    AlreadyTerminal,
    NotCancellable,
    BodyTooLarge,
    RawInputTooLarge,
    UnsupportedMediaType,
    UnavailableNativeQualification,
    UnsupportedRuntime,
    UnavailableArtifact,
    ArtifactRejected,
    NativeFailure,
    WorkerFailure,
    WorkerProtocolFailure,
    TerminationUnconfirmed,
    DeadlineExceeded,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NativeErrorTag {
    ContainerLimit,
    InvalidContainer,
    ArtifactIdentityMismatch,
    UnsupportedManifest,
    UnsupportedProfile,
    SourceBindingMismatch,
    InvalidComponent,
    InvalidTensor,
    InvalidCodecPolicy,
    InvalidFrameTable,
    StateIdentityMismatch,
    UnavailableArtifact,
    UnavailableNativeQualification,
    Busy,
    NumericalFailure,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RefusalStatus {
    UnsupportedSchema,
    UnavailableArtifact,
    InputLimit,
    InvalidEncoding,
    UnknownLexeme,
    UnsupportedBoundary,
    UnsupportedSyntax,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IpcCommand {
    Load,
    Answer,
    Unload,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IpcReplyKind {
    Ready,
    Progress,
    Result,
    Failure,
    Unloaded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeError {
    pub tag: NativeErrorTag,
    pub component: Option<String>,
    pub offset: Option<u64>,
}

impl NativeError {
    pub fn validate(&self) -> Result<(), WireError> {
        if let Some(offset) = self.offset {
            require_uint53(offset)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ServiceError {
    pub tag: ServiceErrorTag,
    pub message: String,
    pub native: Option<NativeError>,
}

impl ServiceError {
    pub fn validate(&self) -> Result<(), WireError> {
        require(
            self.message.len() <= 512 && !self.message.chars().any(char::is_control),
            "service error message is not a bounded safe string",
        )?;
        if let Some(native) = &self.native {
            native.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostIdentity {
    pub native_binary_sha256: String,
    pub runtime_receipt_sha256: Option<String>,
    pub target: String,
    pub operator_profile: String,
    pub service_contract_sha256: String,
    pub asset_manifest_sha256: String,
    pub host_acceptance_sha256: Option<String>,
    pub qualification_receipt_sha256: Option<String>,
}

impl HostIdentity {
    pub fn validate(&self) -> Result<(), WireError> {
        require_hex64(&self.native_binary_sha256)?;
        for value in [
            self.runtime_receipt_sha256.as_deref(),
            self.host_acceptance_sha256.as_deref(),
            self.qualification_receipt_sha256.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            require_hex64(value)?;
        }
        require(!self.target.is_empty(), "host target must not be empty")?;
        require_literal(&self.operator_profile, OPERATOR_PROFILE)?;
        require_hex64(&self.service_contract_sha256)?;
        require_hex64(&self.asset_manifest_sha256)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactIdentity {
    pub model_id: String,
    pub artifact_sha256: String,
    pub artifact_bytes: u64,
    pub native_state_sha256: String,
    pub codec_cid: String,
    pub policy_sha256: String,
    pub reader_file_cid: String,
    pub core_file_cid: String,
    pub frame_tree_cid: String,
    pub original_export_release_sha256: String,
}

impl ArtifactIdentity {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.model_id, CONFIGURED_MODEL_ID)?;
        require_literal(&self.artifact_sha256, ARTIFACT_SHA256)?;
        require(
            self.artifact_bytes == ARTIFACT_BYTES,
            "artifact byte length does not match configured artifact",
        )?;
        require_literal(&self.native_state_sha256, NATIVE_STATE_SHA256)?;
        require(is_blake3_cid(&self.codec_cid), "invalid codec CID")?;
        require_hex64(&self.policy_sha256)?;
        require(
            is_blake3_cid(&self.reader_file_cid),
            "invalid reader file CID",
        )?;
        require(is_blake3_cid(&self.core_file_cid), "invalid core file CID")?;
        require(
            is_blake3_cid(&self.frame_tree_cid),
            "invalid frame tree CID",
        )?;
        require_literal(
            &self.original_export_release_sha256,
            ORIGINAL_EXPORT_RELEASE_SHA256,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub stage: ProgressStage,
    pub completed: Option<u64>,
    pub total: Option<u64>,
    pub unit: Option<String>,
    pub fraction: Option<u64>,
    pub eta_ms: Option<u64>,
}

impl Progress {
    pub fn validate(&self) -> Result<(), WireError> {
        for value in [self.completed, self.total, self.fraction, self.eta_ms]
            .into_iter()
            .flatten()
        {
            require_uint53(value)?;
        }
        require(
            self.fraction.is_none() && self.eta_ms.is_none(),
            "fraction and eta_ms must remain null",
        )?;
        if self.stage == ProgressStage::ReadingArtifact {
            require(
                self.unit.as_deref() == Some("bytes")
                    && self.total == Some(ARTIFACT_BYTES)
                    && self.completed.is_some_and(|value| value <= ARTIFACT_BYTES),
                "reading_artifact progress requires bounded byte denominator",
            )
        } else {
            require(
                self.completed.is_none() && self.total.is_none() && self.unit.is_none(),
                "non-reading progress counters must be null",
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HistoricalReference {
    pub issue: String,
    pub terminal: String,
    pub binary_sha256: String,
    pub qualification_sha256: String,
    pub scope: String,
    pub applies_to_current_host: bool,
}

impl HistoricalReference {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.issue, "1102")?;
        require_literal(&self.terminal, "NATIVE_REFERENCE_PRESERVED")?;
        require_literal(&self.binary_sha256, HISTORICAL_BINARY_SHA256)?;
        require_literal(&self.qualification_sha256, HISTORICAL_QUALIFICATION_SHA256)?;
        require_literal(&self.scope, "known-authoring-four-fact-reference")?;
        require(
            !self.applies_to_current_host,
            "historical qualification cannot apply to current host",
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub id: String,
    pub input_schema: String,
    pub result_schemas: Vec<String>,
    pub enabled: bool,
    pub unavailable_reason: Option<ServiceError>,
    pub stateless: bool,
    pub input_policy_max_bytes: u64,
    pub decoded_transport_max_bytes: u64,
    pub general_generation: bool,
    pub general_context: bool,
    pub coding: bool,
    pub final_integer_kernel: bool,
}

impl Operation {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.id, OPERATION_ID)?;
        require_literal(&self.input_schema, RAW_INPUT_SCHEMA)?;
        require(
            self.result_schemas
                == [
                    MODEL_RESULT_SCHEMA.to_owned(),
                    REFUSAL_RESULT_SCHEMA.to_owned(),
                ],
            "result schemas must be the exact ordered accepted pair",
        )?;
        require(
            self.enabled == self.unavailable_reason.is_none(),
            "operation availability and reason disagree",
        )?;
        if let Some(error) = &self.unavailable_reason {
            error.validate()?;
        }
        require(self.stateless, "operation must be stateless")?;
        require(
            self.input_policy_max_bytes == CORE_INPUT_POLICY_MAX_BYTES,
            "operation input policy cap changed",
        )?;
        require(
            self.decoded_transport_max_bytes == RAW_DECODED_MAX_BYTES as u64,
            "operation transport cap changed",
        )?;
        require(
            !self.general_generation
                && !self.general_context
                && !self.coding
                && !self.final_integer_kernel,
            "operation scope exceeds the accepted research reference",
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub schema: String,
    pub instance_id: String,
    pub revision: u64,
    pub provider: String,
    pub execution: String,
    pub host: HostIdentity,
    pub configured_artifact: ArtifactIdentity,
    pub model_state: ModelState,
    pub operations: Vec<Operation>,
    pub historical_reference: HistoricalReference,
    pub active_job_id: Option<String>,
    pub last_job_id: Option<String>,
}

impl Capabilities {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, CAPABILITIES_SCHEMA)?;
        require_hex32(&self.instance_id)?;
        require_uint53(self.revision)?;
        require_literal(&self.provider, "native")?;
        require_literal(&self.execution, "cpu-floating-point-research-reference")?;
        self.host.validate()?;
        self.configured_artifact.validate()?;
        require(
            self.operations.len() == 1,
            "exactly one operation is exposed",
        )?;
        self.operations[0].validate()?;
        let operation = &self.operations[0];
        let expected_unavailable_tag = match self.model_state {
            ModelState::Ready => None,
            ModelState::Loading
            | ModelState::Running
            | ModelState::Stopping
            | ModelState::Unloading => Some(ServiceErrorTag::Busy),
            ModelState::Unloaded => Some(ServiceErrorTag::NotReady),
            ModelState::Unavailable | ModelState::Error => {
                require(
                    operation.unavailable_reason.is_some(),
                    "unavailable or error model requires an operation reason",
                )?;
                operation.unavailable_reason.as_ref().map(|error| error.tag)
            }
        };
        require(
            operation.enabled == (self.model_state == ModelState::Ready)
                && operation.unavailable_reason.as_ref().map(|error| error.tag)
                    == expected_unavailable_tag,
            "operation availability does not match model state",
        )?;
        self.historical_reference.validate()?;
        validate_job_ids(&self.active_job_id, &self.last_job_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModelSnapshot {
    pub schema: String,
    pub instance_id: String,
    pub revision: u64,
    pub model_id: String,
    pub model_generation: u64,
    pub state: ModelState,
    pub verified_artifact: Option<ArtifactIdentity>,
    pub qualification_receipt_sha256: Option<String>,
    pub active_job_id: Option<String>,
    pub last_job_id: Option<String>,
    pub progress: Progress,
    pub error: Option<ServiceError>,
}

impl ModelSnapshot {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, MODEL_SCHEMA)?;
        require_hex32(&self.instance_id)?;
        require_uint53(self.revision)?;
        require_literal(&self.model_id, CONFIGURED_MODEL_ID)?;
        require_uint53(self.model_generation)?;
        if let Some(artifact) = &self.verified_artifact {
            artifact.validate()?;
        }
        if let Some(digest) = &self.qualification_receipt_sha256 {
            require_hex64(digest)?;
        }
        validate_job_ids(&self.active_job_id, &self.last_job_id)?;
        self.progress.validate()?;
        if let Some(error) = &self.error {
            error.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LoadRequest {
    pub schema: String,
    pub instance_id: String,
    pub model_id: String,
}

impl LoadRequest {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, LOAD_SCHEMA)?;
        require_hex32(&self.instance_id)?;
        require_literal(&self.model_id, CONFIGURED_MODEL_ID)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnloadRequest {
    pub schema: String,
    pub instance_id: String,
    pub model_id: String,
    pub expected_generation: u64,
}

impl UnloadRequest {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, UNLOAD_SCHEMA)?;
        require_hex32(&self.instance_id)?;
        require_literal(&self.model_id, CONFIGURED_MODEL_ID)?;
        require_uint53(self.expected_generation)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RawInput {
    pub schema: String,
    pub encoding: String,
    pub bytes_b64: String,
}

impl RawInput {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, RAW_INPUT_SCHEMA)?;
        require_literal(&self.encoding, "base64")?;
        crate::base64::decode_canonical(&self.bytes_b64, RAW_DECODED_MAX_BYTES)
            .map(|_| ())
            .map_err(|error| WireError::new(error.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AnswerRequest {
    pub schema: String,
    pub instance_id: String,
    pub model_id: String,
    pub expected_generation: u64,
    pub operation: String,
    pub input: RawInput,
}

impl AnswerRequest {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, REQUEST_SCHEMA)?;
        require_hex32(&self.instance_id)?;
        require_literal(&self.model_id, CONFIGURED_MODEL_ID)?;
        require_uint53(self.expected_generation)?;
        require_literal(&self.operation, OPERATION_ID)?;
        self.input.validate()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CancelRequest {
    pub schema: String,
    pub instance_id: String,
}

impl CancelRequest {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, CANCEL_SCHEMA)?;
        require_hex32(&self.instance_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ErrorResponse {
    pub schema: String,
    pub instance_id: String,
    pub revision: u64,
    pub error: ServiceError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub forward_count: Option<u64>,
    pub forward_upper_bound: u64,
    pub elapsed_ms: Option<u64>,
}

impl Work {
    pub fn validate(&self) -> Result<(), WireError> {
        require(
            self.forward_count
                .is_none_or(|value| value <= self.forward_upper_bound)
                && self.forward_upper_bound <= 1,
            "forward counters must be zero or one",
        )?;
        if let Some(elapsed_ms) = self.elapsed_ms {
            require_uint53(elapsed_ms)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModelToken {
    pub schema: String,
    pub status: String,
    pub policy_sha256: String,
    pub raw_text_sha256: String,
    pub derived_input_sha256: String,
    pub reader_file_cid: String,
    pub core_file_cid: String,
    pub frame_tree_cid: String,
    pub token_id: u64,
    pub token: String,
}

impl ModelToken {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, "uor-r4.text-binding-result/1")?;
        require_literal(&self.status, "MODEL_TOKEN")?;
        require_hex64(&self.policy_sha256)?;
        require_hex64(&self.raw_text_sha256)?;
        require_hex64(&self.derived_input_sha256)?;
        require(
            is_blake3_cid(&self.reader_file_cid),
            "invalid reader file CID",
        )?;
        require(is_blake3_cid(&self.core_file_cid), "invalid core file CID")?;
        require(
            is_blake3_cid(&self.frame_tree_cid),
            "invalid frame tree CID",
        )?;
        require(self.token_id <= 4095, "token ID exceeds fixed vocabulary")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Refusal {
    pub schema: String,
    pub status: RefusalStatus,
    pub byte_offset: Option<u64>,
}

impl Refusal {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, "uor-r4.text-to-clauses-result/1")?;
        if let Some(offset) = self.byte_offset {
            require_uint53(offset)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum NativeTextResult {
    Model(ModelToken),
    Refusal(Refusal),
}

impl NativeTextResult {
    pub fn validate(&self) -> Result<(), WireError> {
        match self {
            Self::Model(value) => value.validate(),
            Self::Refusal(value) => value.validate(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct JobSnapshot {
    pub schema: String,
    pub instance_id: String,
    pub revision: u64,
    pub job_id: String,
    pub kind: JobKind,
    pub state: JobState,
    pub model_id: String,
    pub admitted_generation: u64,
    pub raw_text_sha256: Option<String>,
    pub progress: Progress,
    pub stop_reason: Option<StopReason>,
    pub result: Option<NativeTextResult>,
    pub error: Option<ServiceError>,
    pub work: Work,
    pub host: HostIdentity,
    pub artifact: ArtifactIdentity,
}

impl JobSnapshot {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, JOB_SCHEMA)?;
        require_hex32(&self.instance_id)?;
        require_uint53(self.revision)?;
        require(is_job_id(&self.job_id), "invalid job ID")?;
        require_literal(&self.model_id, CONFIGURED_MODEL_ID)?;
        require_uint53(self.admitted_generation)?;
        if let Some(digest) = &self.raw_text_sha256 {
            require_hex64(digest)?;
        }
        self.progress.validate()?;
        if let Some(result) = &self.result {
            result.validate()?;
        }
        if let Some(error) = &self.error {
            error.validate()?;
        }
        self.work.validate()?;
        self.host.validate()?;
        self.artifact.validate()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FileIdentity {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

impl FileIdentity {
    pub fn validate_shape(&self) -> Result<(), WireError> {
        require_uint53(self.bytes)?;
        require_hex64(&self.sha256)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AssetFile {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
    pub mime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AssetManifest {
    pub schema: String,
    pub files: Vec<AssetFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalConfiguration {
    pub schema: String,
    pub artifact_path: String,
    pub expected_binding: ExpectedBinding,
    pub host_acceptance: Option<FileIdentity>,
    pub trusted_host_acceptance_sha256: Option<String>,
    pub asset_manifest: FileIdentity,
    pub bind_host: String,
    pub port: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostAcceptance {
    pub schema: String,
    pub service_contract_sha256: String,
    pub native_binary_sha256: String,
    pub operator_profile: String,
    pub target: String,
    pub runtime_receipt: FileIdentity,
    pub qualification: FileIdentity,
    pub comparison_result: FileIdentity,
    pub accepted_result_review: FileIdentity,
    pub fresh_execution_release: FileIdentity,
    pub original_export_release_sha256: String,
    pub artifact_sha256: String,
    pub native_state_sha256: String,
}

/// The canonical twelve-field #1086 qualification record referenced by an
/// independently adopted workbench HostAcceptance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeQualification {
    pub schema: String,
    pub terminal: String,
    pub accepted_review_sha256: String,
    pub comparison_result_sha256: String,
    pub artifact_sha256: String,
    pub native_state_sha256: String,
    pub native_binary_sha256: String,
    pub runtime_receipt_sha256: String,
    pub contract_sha256: String,
    pub operator_profile: String,
    pub request_schema: String,
    pub result_schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IpcLoad {
    pub configuration_path: String,
    pub configuration_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum IpcRequestPayload {
    Load(IpcLoad),
    Answer(RawInput),
    Empty(()),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IpcRequest {
    pub schema: String,
    pub instance_id: String,
    pub job_id: String,
    pub worker_generation: u64,
    pub command: IpcCommand,
    pub payload: IpcRequestPayload,
}

impl IpcRequest {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, IPC_SCHEMA)?;
        require_hex32(&self.instance_id)?;
        require(is_job_id(&self.job_id), "invalid job ID")?;
        require_uint53(self.worker_generation)?;
        match (&self.command, &self.payload) {
            (IpcCommand::Load, IpcRequestPayload::Load(load)) => {
                require_hex64(&load.configuration_sha256)
            }
            (IpcCommand::Answer, IpcRequestPayload::Answer(input)) => input.validate(),
            (IpcCommand::Unload, IpcRequestPayload::Empty(())) => Ok(()),
            _ => Err(WireError::new("IPC command/payload mismatch")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkerReady {
    pub host: HostIdentity,
    pub artifact: ArtifactIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum IpcResponsePayload {
    Ready(WorkerReady),
    Progress(Progress),
    Result(NativeTextResult),
    Failure(ServiceError),
    Empty(()),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IpcResponse {
    pub schema: String,
    pub instance_id: String,
    pub job_id: String,
    pub worker_generation: u64,
    pub kind: IpcReplyKind,
    pub payload: IpcResponsePayload,
}

impl IpcResponse {
    pub fn validate(&self) -> Result<(), WireError> {
        require_literal(&self.schema, IPC_SCHEMA)?;
        require_hex32(&self.instance_id)?;
        require(is_job_id(&self.job_id), "invalid job ID")?;
        require_uint53(self.worker_generation)?;
        match (&self.kind, &self.payload) {
            (IpcReplyKind::Ready, IpcResponsePayload::Ready(ready)) => {
                ready.host.validate()?;
                ready.artifact.validate()
            }
            (IpcReplyKind::Progress, IpcResponsePayload::Progress(progress)) => progress.validate(),
            (IpcReplyKind::Result, IpcResponsePayload::Result(result)) => result.validate(),
            (IpcReplyKind::Failure, IpcResponsePayload::Failure(error)) => error.validate(),
            (IpcReplyKind::Unloaded, IpcResponsePayload::Empty(())) => Ok(()),
            _ => Err(WireError::new("IPC reply kind/payload mismatch")),
        }
    }
}

fn validate_job_ids(active: &Option<String>, last: &Option<String>) -> Result<(), WireError> {
    for value in [active.as_deref(), last.as_deref()].into_iter().flatten() {
        require(is_job_id(value), "invalid job ID")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strict_json;

    const INSTANCE: &str = "00000000000000000000000000000001";

    fn progress(stage: ProgressStage) -> Progress {
        Progress {
            stage,
            completed: None,
            total: None,
            unit: None,
            fraction: None,
            eta_ms: None,
        }
    }

    fn host() -> HostIdentity {
        HostIdentity {
            native_binary_sha256: "1".repeat(64),
            runtime_receipt_sha256: None,
            target: FIRST_TARGET.to_owned(),
            operator_profile: OPERATOR_PROFILE.to_owned(),
            service_contract_sha256: "2".repeat(64),
            asset_manifest_sha256: "3".repeat(64),
            host_acceptance_sha256: None,
            qualification_receipt_sha256: None,
        }
    }

    fn artifact() -> ArtifactIdentity {
        ArtifactIdentity {
            model_id: CONFIGURED_MODEL_ID.to_owned(),
            artifact_sha256: ARTIFACT_SHA256.to_owned(),
            artifact_bytes: ARTIFACT_BYTES,
            native_state_sha256: NATIVE_STATE_SHA256.to_owned(),
            codec_cid: format!("blake3:{}", "4".repeat(64)),
            policy_sha256: "5".repeat(64),
            reader_file_cid: format!("blake3:{}", "6".repeat(64)),
            core_file_cid: format!("blake3:{}", "7".repeat(64)),
            frame_tree_cid: format!("blake3:{}", "8".repeat(64)),
            original_export_release_sha256: ORIGINAL_EXPORT_RELEASE_SHA256.to_owned(),
        }
    }

    #[test]
    fn request_shapes_are_exact_and_literals_are_validated() {
        let request: LoadRequest = strict_json::from_slice(
            format!(
                r#"{{"schema":"{LOAD_SCHEMA}","instance_id":"{INSTANCE}","model_id":"{CONFIGURED_MODEL_ID}"}}"#
            )
            .as_bytes(),
        )
        .unwrap();
        request.validate().unwrap();

        let unknown = format!(
            r#"{{"schema":"{LOAD_SCHEMA}","instance_id":"{INSTANCE}","model_id":"{CONFIGURED_MODEL_ID}","extra":null}}"#
        );
        assert!(strict_json::from_slice::<LoadRequest>(unknown.as_bytes()).is_err());

        let mut wrong = request;
        wrong.schema = "uor-r4.workbench-load/2".to_owned();
        assert!(wrong.validate().is_err());
    }

    #[test]
    fn enums_have_only_the_frozen_http_and_ipc_spellings() {
        assert_eq!(
            serde_json::to_string(&ModelState::Unavailable).unwrap(),
            "\"unavailable\""
        );
        assert_eq!(
            serde_json::to_string(&StopReason::UserCancel).unwrap(),
            "\"user_cancel\""
        );
        assert_eq!(
            serde_json::to_string(&ServiceErrorTag::Busy).unwrap(),
            "\"BUSY\""
        );

        let commands = [IpcCommand::Load, IpcCommand::Answer, IpcCommand::Unload];
        assert_eq!(
            serde_json::to_value(commands).unwrap(),
            serde_json::json!(["load", "answer", "unload"])
        );
        let replies = [
            IpcReplyKind::Ready,
            IpcReplyKind::Progress,
            IpcReplyKind::Result,
            IpcReplyKind::Failure,
            IpcReplyKind::Unloaded,
        ];
        assert_eq!(
            serde_json::to_value(replies).unwrap(),
            serde_json::json!(["ready", "progress", "result", "failure", "unloaded"])
        );
    }

    #[test]
    fn ipc_discriminants_reject_wrong_payload_shapes() {
        let request = IpcRequest {
            schema: IPC_SCHEMA.to_owned(),
            instance_id: INSTANCE.to_owned(),
            job_id: "1".to_owned(),
            worker_generation: 1,
            command: IpcCommand::Unload,
            payload: IpcRequestPayload::Answer(RawInput {
                schema: RAW_INPUT_SCHEMA.to_owned(),
                encoding: "base64".to_owned(),
                bytes_b64: String::new(),
            }),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn snapshot_serialization_keeps_required_null_fields() {
        let model = ModelSnapshot {
            schema: MODEL_SCHEMA.to_owned(),
            instance_id: INSTANCE.to_owned(),
            revision: 0,
            model_id: CONFIGURED_MODEL_ID.to_owned(),
            model_generation: 0,
            state: ModelState::Unloaded,
            verified_artifact: None,
            qualification_receipt_sha256: None,
            active_job_id: None,
            last_job_id: None,
            progress: progress(ProgressStage::Idle),
            error: None,
        };
        model.validate().unwrap();
        let value = serde_json::to_value(&model).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 12);
        for key in [
            "verified_artifact",
            "qualification_receipt_sha256",
            "active_job_id",
            "last_job_id",
            "error",
        ] {
            assert!(value[key].is_null(), "{key}");
        }

        let job = JobSnapshot {
            schema: JOB_SCHEMA.to_owned(),
            instance_id: INSTANCE.to_owned(),
            revision: 1,
            job_id: "1".to_owned(),
            kind: JobKind::Load,
            state: JobState::Accepted,
            model_id: CONFIGURED_MODEL_ID.to_owned(),
            admitted_generation: 0,
            raw_text_sha256: None,
            progress: progress(ProgressStage::Idle),
            stop_reason: None,
            result: None,
            error: None,
            work: Work {
                forward_count: None,
                forward_upper_bound: 0,
                elapsed_ms: None,
            },
            host: host(),
            artifact: artifact(),
        };
        job.validate().unwrap();
        let value = serde_json::to_value(job).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 16);
        for key in ["raw_text_sha256", "stop_reason", "result", "error"] {
            assert!(value[key].is_null(), "{key}");
        }
    }

    #[test]
    fn primitive_bounds_match_the_machine_contract() {
        assert!(is_job_id("1"));
        assert!(is_job_id(&UINT53_MAX.to_string()));
        assert!(!is_job_id("0"));
        assert!(!is_job_id("01"));
        assert!(!is_job_id("9007199254740992"));
        assert!(is_hex(&"a".repeat(64), 64));
        assert!(!is_hex(&"A".repeat(64), 64));
    }

    #[test]
    fn forward_count_cannot_exceed_its_upper_bound() {
        assert!(Work {
            forward_count: Some(1),
            forward_upper_bound: 0,
            elapsed_ms: None,
        }
        .validate()
        .is_err());
        assert!(Work {
            forward_count: Some(1),
            forward_upper_bound: 1,
            elapsed_ms: None,
        }
        .validate()
        .is_ok());
    }
}
